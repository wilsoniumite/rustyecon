//! Export (docs/GUI.md §4): what a run's export directory holds, as text. `platform` writes
//! it; nothing here touches a file.
//!
//! - `series.csv`: exactly the plotted series at full resolution, one row per report tick, a
//!   value written as the shortest text that reads back to the same f64, a gap left empty. Its
//!   `#` lines give the export's stamp ("run, draft" or "experiment, draft"), the run key as
//!   the cli's `run …` line gives it, the ticks, the origin, the lineage file and whether the
//!   ledger changed.
//! - `manifest.ron`: a [`GuiManifest`], an envelope around certify's `Manifest`, so certify's
//!   type does not change: the run's record (its key, genesis, start, any checkpoint it resumed
//!   from and its hash stream by digest), with the origin, the draft mark, the lineage's file
//!   name and "ledger changed".
//! - `tape.ron`: the run's tape, canonical, which the cli runs unchanged (E1).
//! - For an experiment, `tape.lineage.ron`, and `ancestor.ron`: the tape of the nearest
//!   ancestor on disk that the lineage names, when this session holds it.
//!
//! The manifest is begun from a `Sim` built from the tape at genesis, as the cli begins one,
//! read and dropped: only a Runner keeps a `Sim` (U1).

use super::Lineage;
use crate::run::{Origin, SeriesKey, Store};
use certify::{Hex, Manifest, ResumedFrom};
use rustyecon_engine::prelude::*;
use serde::{Deserialize, Serialize};

/// The series file.
pub const SERIES: &str = "series.csv";
/// The manifest envelope.
pub const MANIFEST: &str = "manifest.ron";
/// The run's tape.
pub const TAPE: &str = "tape.ron";
/// The run's lineage.
pub const LINEAGE: &str = "tape.lineage.ron";
/// The tape of the nearest ancestor on disk.
pub const ANCESTOR: &str = "ancestor.ron";

/// The manifest of a GUI export: certify's `Manifest`, unchanged, in an envelope (§4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuiManifest {
    /// The run's record, as certify writes one.
    pub manifest: Manifest,
    /// Run or experiment (D5, U3).
    pub origin: Origin,
    /// Always true: no GUI export is citable before G6–7.
    pub draft: bool,
    /// The lineage's file name, beside this one, for an experiment.
    #[serde(deserialize_with = "required")]
    pub lineage: Option<String>,
    /// Whether the tape's ledger tolerances differ from its parent's (§5.1 item 2).
    pub ledger_changed: bool,
}

/// Read an `Option` field that must be written.
fn required<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

impl GuiManifest {
    /// The envelope as RON, opening `GuiManifest(`.
    pub fn to_ron(&self) -> String {
        let pretty = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .new_line("\n".to_string());
        let mut s = ron::ser::to_string_pretty(self, pretty).expect("a manifest serialises");
        s.push('\n');
        s
    }

    /// An envelope from RON.
    pub fn from_ron(text: &str) -> Result<GuiManifest, String> {
        ron::from_str(text).map_err(|e| e.to_string())
    }

    /// "run, draft" or "experiment, draft".
    pub fn stamp(&self) -> String {
        stamp(self.origin)
    }
}

/// An export's stamp: "run, draft" or "experiment, draft".
pub fn stamp(origin: Origin) -> String {
    format!("{origin}, draft")
}

/// What an export reads.
#[derive(Debug, Clone, Copy)]
pub struct Source<'a> {
    /// The run's record.
    pub store: &'a Store,
    /// The plotted series, in the order plotted; those of another world are left out.
    pub plots: &'a [SeriesKey],
    /// The run's origin.
    pub origin: Origin,
    /// The checkpoint the run resumed from, if it did.
    pub resumed_from: Option<&'a ResumedFrom>,
    /// The run's lineage, for an experiment.
    pub lineage: Option<&'a Lineage>,
    /// The tape of the nearest ancestor on disk, when this session holds it.
    pub ancestor: Option<&'a Tape>,
    /// Whether the ledger's tolerances differ from the parent's.
    pub ledger_changed: bool,
}

/// The run's manifest envelope: begun at genesis as the cli begins one, moved to the tick the
/// run started from, and fed every tick's hash the run recorded.
pub fn manifest(s: &Source<'_>) -> Result<GuiManifest, String> {
    let store = s.store;
    let run = store.run().ok_or("the run has not loaded")?.clone();
    let tape = store.tape().ok_or("the run has not loaded")?;
    let genesis = Sim::new(tape).map_err(|e| format!("the tape does not load: {e}"))?;
    let mut m = Manifest::begin(run, genesis.hash(), &genesis, s.resumed_from.cloned());
    drop(genesis);
    m.from = store.start();
    m.until = store.start();
    for (t, h) in store.reports().zip(store.hashes()) {
        m.tick(t + 1, *h);
    }
    Ok(GuiManifest {
        manifest: m,
        origin: s.origin,
        draft: true,
        lineage: s.lineage.map(|_| LINEAGE.to_string()),
        ledger_changed: s.ledger_changed,
    })
}

/// A CSV field, quoted when it holds a comma, a quote or a line break.
fn field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// The plotted series at full resolution, with its `#` lines.
pub fn csv(s: &Source<'_>, m: &GuiManifest) -> Result<String, String> {
    let store = s.store;
    let w = store.world().ok_or("the run has not loaded")?;
    let keys: Vec<&SeriesKey> = s.plots.iter().filter(|k| k.in_world(w)).collect();
    let ticks = store.reports();
    let date = |t: u64| {
        w.clock
            .date_of(t)
            .map_or_else(String::new, |d| d.to_string())
    };
    let mut out = String::new();
    out.push_str(&format!("# rustyecon-gui export: {}\n", m.stamp()));
    out.push_str(&format!("# {}\n", m.manifest.run_line()));
    if ticks.is_empty() {
        out.push_str("# report ticks: none ran\n");
    } else {
        out.push_str(&format!(
            "# report ticks {} to {} ({} to {}), {} rows; each value is stamped with the tick \
             that ran, holdings and params with the state it left\n",
            ticks.start,
            ticks.end - 1,
            date(ticks.start),
            date(ticks.end - 1),
            ticks.end - ticks.start
        ));
    }
    out.push_str(&format!("# origin {}\n", m.origin));
    match &m.lineage {
        Some(f) => out.push_str(&format!("# lineage {f}\n")),
        None => out.push_str("# lineage none\n"),
    }
    out.push_str(&format!(
        "# ledger changed: {}\n",
        if m.ledger_changed { "yes" } else { "no" }
    ));
    let mut header = vec!["tick".to_string(), "date".to_string()];
    header.extend(keys.iter().map(|k| field(&k.to_string())));
    out.push_str(&header.join(","));
    out.push('\n');
    let series: Vec<_> = keys.iter().map(|k| store.series(k)).collect();
    for t in ticks {
        let mut row = vec![t.to_string(), date(t)];
        for s in &series {
            row.push(
                s.and_then(|s| s.at(t))
                    .map_or_else(String::new, |v| v.to_string()),
            );
        }
        out.push_str(&row.join(","));
        out.push('\n');
    }
    Ok(out)
}

/// Every file of the export: its name in the export directory and its text.
pub fn files(s: &Source<'_>) -> Result<Vec<(String, String)>, String> {
    let m = manifest(s)?;
    let tape = s.store.tape().ok_or("the run has not loaded")?;
    let mut out = vec![
        (SERIES.to_string(), csv(s, &m)?),
        (MANIFEST.to_string(), m.to_ron()),
        (TAPE.to_string(), tape.to_ron()),
    ];
    if let Some(l) = s.lineage {
        out.push((LINEAGE.to_string(), l.to_ron()));
        if let Some(a) = s.ancestor {
            if certify::tape_hash(a) == l.parent.tape_hash.0 {
                out.push((ANCESTOR.to_string(), a.to_ron()));
            }
        }
    }
    Ok(out)
}

/// The record a resume from a ring checkpoint leaves in the manifest: the checkpoint's tick
/// and digest, the hash of the state it holds, and the run that took it. That hash is the
/// resumed run's hash at its load, `store`'s start. `None` when the store did not start from
/// the checkpoint (its resume was refused and it reran from genesis) or the bytes do not
/// decode.
pub fn resumed_from(cp: &crate::run::RingCheckpoint, store: &Store) -> Option<ResumedFrom> {
    if store.run().is_none() || store.start() != cp.tick() || cp.tick() == 0 {
        return None;
    }
    let c = cp.checkpoint().ok()?;
    Some(ResumedFrom {
        tick: cp.tick(),
        digest: Hex(c.digest()),
        state_hash: Hex(store.start_hash()),
        parent: cp.run().clone(),
    })
}
