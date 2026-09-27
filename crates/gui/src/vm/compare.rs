//! The compare panel's view-model (docs/GUI.md §4, §5.1 item 5): a branch against its parent.
//!
//! - **Identities:** each run's key (build, `world_id`, `tape_hash`), its origin and where it
//!   started.
//! - **The first differing hash:** the first state tick whose hash differs, over the ticks
//!   both runs recorded, and the report tick that left it. A branch shares its parent's past
//!   up to its resume tick, so a difference at or before the state it resumed from is flagged
//!   ([`CompareVm::before_resume`]). A parent that itself resumed from its own parent's ring
//!   holds no record before its start; its earlier states are that run's, so the comparison
//!   reads them there ([`Side::earlier`]). When the records this session holds do not reach
//!   back far enough, a difference at the first state both hold says it may begin earlier
//!   ([`FirstDifferenceVm::may_be_earlier`]).
//! - **The tape diff by key:** every entry added, removed or changed, section by section, each
//!   written as the tape writes it.
//! - **The lineage:** its lines, as the caller gives them (the lineage is the editor's; this
//!   builder reads run types only, D13).
//! - **The difference of the plotted series:** at the cursor, the parent's value, the
//!   branch's and branch minus parent; over the ticks both recorded, the largest absolute
//!   difference and its tick, and the first tick they differ. Differencing is a display
//!   transform U6 allows.

use super::{date, report_tick, unit_of};
use crate::run::{Origin, SeriesKey, Store};
use serde::Serialize;

/// One run as compare reads it.
#[derive(Debug, Clone, Copy)]
pub struct Side<'a> {
    /// Its record.
    pub store: &'a Store,
    /// The records of the runs it resumed from, nearest first, as far back as the caller holds
    /// them: a run resumed from its parent's ring checkpoint has no record before its start,
    /// and its states there are its parent's. Compare reads the parent's side only: a branch's
    /// own states before its start are its parent's, so nothing there can differ.
    pub earlier: &'a [&'a Store],
    /// Run or experiment (U3).
    pub origin: Origin,
}

impl Side<'_> {
    /// The record that holds state tick or report tick `t`: its own from its start, else the
    /// first earlier one that starts at or before `t`.
    fn at(&self, t: u64) -> Option<&Store> {
        std::iter::once(self.store)
            .chain(self.earlier.iter().copied())
            .find(|s| s.run().is_some() && s.start() <= t)
    }

    /// The first state tick its records hold.
    fn first(&self) -> u64 {
        std::iter::once(self.store)
            .chain(self.earlier.iter().copied())
            .filter(|s| s.run().is_some())
            .map(Store::start)
            .min()
            .unwrap_or(self.store.start())
    }

    fn state_hash_at(&self, s: u64) -> Option<u64> {
        self.at(s)?.state_hash_at(s)
    }

    fn value_at(&self, key: &SeriesKey, t: u64) -> Option<f64> {
        self.at(t)?.series(key)?.at(t)
    }
}

/// One run's identity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IdentityVm {
    /// The tape's name.
    pub name: String,
    /// Run or experiment (U3).
    pub origin: Origin,
    /// The build's commit.
    pub commit: String,
    /// Whether the build's sources differed from it.
    pub dirty: bool,
    /// `world_id`, as `0x%016x`.
    pub world_id: String,
    /// `tape_hash`, as `0x%016x`.
    pub tape_hash: String,
    /// The state tick its record starts from: 0 from genesis, the resume tick otherwise.
    pub start: u64,
    /// The latest state tick recorded.
    pub tick: u64,
}

/// Where the two runs' hashes first differ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FirstDifferenceVm {
    /// The first state tick whose hash differs.
    pub state_tick: u64,
    /// The report tick that left it, `state_tick - 1`, when a tick left it.
    pub report_tick: Option<u64>,
    /// That report tick's first date.
    pub date: String,
    /// The parent's hash there, `0x%016x`.
    pub parent: String,
    /// The branch's.
    pub child: String,
    /// Whether this is the first state both records hold, past genesis and past the branch's
    /// start: the parent's records begin there, so the hashes may differ earlier, in states
    /// this session does not hold.
    pub may_be_earlier: bool,
}

/// How an entry changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Change {
    /// The branch has it and the parent does not.
    Added,
    /// The parent has it and the branch does not.
    Removed,
    /// Both have it, written differently.
    Changed,
}

/// One entry of the tape diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLineVm {
    /// The tape's section: `header`, `params`, `events`, …
    pub section: String,
    /// The entry's key, or the header's field.
    pub key: String,
    /// How it changed.
    pub change: Change,
    /// The parent's entry, as the tape writes it.
    pub before: Option<String>,
    /// The branch's.
    pub after: Option<String>,
}

/// One plotted series, branch against parent.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SeriesDiffVm {
    /// The series.
    pub key: SeriesKey,
    /// Its name.
    pub label: String,
    /// Its unit.
    pub unit: String,
    /// At the cursor's report tick: the parent's value, the branch's, branch minus parent.
    pub at_cursor: (Option<f64>, Option<f64>, Option<f64>),
    /// Over the ticks both recorded: the largest absolute difference, its tick and the
    /// difference there, branch minus parent. `None` when they never differ.
    pub largest: Option<(u64, f64)>,
    /// The first tick both recorded where the values differ.
    pub first_differs: Option<u64>,
}

/// The compare panel.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompareVm {
    /// The parent.
    pub parent: IdentityVm,
    /// The branch.
    pub child: IdentityVm,
    /// The state ticks compared: from the later of the two records' first states (the
    /// parent's reaching back through the runs it resumed from) to the earlier latest tick.
    pub compared: (u64, u64),
    /// The first differing hash, if any in the ticks compared.
    pub first_difference: Option<FirstDifferenceVm>,
    /// Whether a branch that resumed from a checkpoint (its start is past genesis) first
    /// differs at or before the state it resumed from, which a checkpoint the two share cannot
    /// give: an error. A branch rerun from genesis may differ from the start, when its edit
    /// changed the world.
    pub before_resume: bool,
    /// The tape diff, by section and key.
    pub tape: Vec<DiffLineVm>,
    /// The branch's lineage, in lines.
    pub lineage: Vec<String>,
    /// The cursor's report tick in the branch, once a tick ran.
    pub cursor: Option<u64>,
    /// The plotted series of the branch's world, branch against parent.
    pub series: Vec<SeriesDiffVm>,
}

fn identity(side: &Side<'_>) -> Option<IdentityVm> {
    let store = side.store;
    let k = store.run()?;
    let w = store.world()?;
    Some(IdentityVm {
        name: w.name.clone(),
        origin: side.origin,
        commit: k.build.commit.clone(),
        dirty: k.build.dirty,
        world_id: k.world_id.to_string(),
        tape_hash: k.tape_hash.to_string(),
        start: store.start(),
        tick: store.tick(),
    })
}

/// An entry as RON on one line.
fn ron_of<T: Serialize>(v: &T) -> String {
    ron::to_string(v).unwrap_or_else(|e| format!("({e})"))
}

/// The diff of one keyed section.
fn keyed<T: Serialize>(
    out: &mut Vec<DiffLineVm>,
    section: &str,
    before: &[T],
    after: &[T],
    key: impl Fn(&T) -> String,
) {
    use std::collections::BTreeMap;
    let b: BTreeMap<String, String> = before.iter().map(|e| (key(e), ron_of(e))).collect();
    let a: BTreeMap<String, String> = after.iter().map(|e| (key(e), ron_of(e))).collect();
    let keys: std::collections::BTreeSet<&String> = b.keys().chain(a.keys()).collect();
    for k in keys {
        let (x, y) = (b.get(k), a.get(k));
        let change = match (x, y) {
            (None, Some(_)) => Change::Added,
            (Some(_), None) => Change::Removed,
            (Some(x), Some(y)) if x != y => Change::Changed,
            _ => continue,
        };
        out.push(DiffLineVm {
            section: section.to_string(),
            key: k.clone(),
            change,
            before: x.cloned(),
            after: y.cloned(),
        });
    }
}

/// The tape diff of `after` against `before`, by section and key, each read in its canonical
/// order, so a list written in another order is no change (as `tape_hash` reads it).
pub fn tape_diff(
    before: &rustyecon_engine::prelude::Tape,
    after: &rustyecon_engine::prelude::Tape,
) -> Vec<DiffLineVm> {
    let (before, after) = (&before.canonical(), &after.canonical());
    let mut out = Vec::new();
    let (hb, ha) = (&before.header, &after.header);
    let header = [
        ("name", ron_of(&hb.name), ron_of(&ha.name)),
        ("start", ron_of(&hb.start), ron_of(&ha.start)),
        (
            "ticks_per_year",
            ron_of(&hb.ticks_per_year),
            ron_of(&ha.ticks_per_year),
        ),
        ("market", ron_of(&hb.market), ron_of(&ha.market)),
        ("ledger", ron_of(&hb.ledger), ron_of(&ha.ledger)),
    ];
    for (k, x, y) in header {
        if x != y {
            out.push(DiffLineVm {
                section: "header".to_string(),
                key: k.to_string(),
                change: Change::Changed,
                before: Some(x),
                after: Some(y),
            });
        }
    }
    if before.schema != after.schema {
        out.push(DiffLineVm {
            section: "schema".to_string(),
            key: "schema".to_string(),
            change: Change::Changed,
            before: Some(before.schema.to_string()),
            after: Some(after.schema.to_string()),
        });
    }
    keyed(&mut out, "params", &before.params, &after.params, |p| {
        p.key.to_string()
    });
    keyed(&mut out, "goods", &before.goods, &after.goods, |g| {
        g.key.to_string()
    });
    keyed(&mut out, "nodes", &before.nodes, &after.nodes, |n| {
        n.key.to_string()
    });
    keyed(
        &mut out,
        "channels",
        &before.channels,
        &after.channels,
        |c| c.key.to_string(),
    );
    keyed(&mut out, "classes", &before.classes, &after.classes, |c| {
        c.to_string()
    });
    keyed(&mut out, "actors", &before.actors, &after.actors, |a| {
        a.key.to_string()
    });
    let (gb, ga) = (&before.genesis, &after.genesis);
    if gb.basis != ga.basis {
        out.push(DiffLineVm {
            section: "genesis".to_string(),
            key: "basis".to_string(),
            change: Change::Changed,
            before: Some(ron_of(&gb.basis)),
            after: Some(ron_of(&ga.basis)),
        });
    }
    keyed(&mut out, "genesis.prices", &gb.prices, &ga.prices, |p| {
        format!("{}/{}", p.node, p.good)
    });
    keyed(
        &mut out,
        "genesis.holdings",
        &gb.holdings,
        &ga.holdings,
        |h| h.holder.to_string(),
    );
    keyed(&mut out, "events", &before.events, &after.events, |e| {
        e.key.to_string()
    });
    keyed(
        &mut out,
        "recurring",
        &before.recurring,
        &after.recurring,
        |r| r.key.to_string(),
    );
    out
}

/// The first state tick in `lo..=hi` whose hash differs between the two records.
fn first_difference(parent: &Side<'_>, child: &Store, lo: u64, hi: u64) -> Option<u64> {
    (lo..=hi).find(|&s| {
        let (p, c) = (parent.state_hash_at(s), child.state_hash_at(s));
        p.is_some() && c.is_some() && p != c
    })
}

fn series_diff(
    parent: &Side<'_>,
    child: &Store,
    key: &SeriesKey,
    unit: String,
    cursor: Option<u64>,
) -> SeriesDiffVm {
    let c = child.series(key);
    let pv = cursor.and_then(|t| parent.value_at(key, t));
    let cv = c.and_then(|s| s.at(cursor?));
    let mut largest: Option<(u64, f64)> = None;
    let mut first = None;
    if let Some(c) = c {
        for (&t, &y) in c.ticks().iter().zip(c.values()) {
            let Some(x) = parent.value_at(key, t) else {
                continue;
            };
            if x.to_bits() != y.to_bits() && first.is_none() {
                first = Some(t);
            }
            let d = y - x;
            if d != 0.0 && largest.is_none_or(|(_, l)| d.abs() > l.abs()) {
                largest = Some((t, d));
            }
        }
    }
    SeriesDiffVm {
        key: key.clone(),
        label: key.to_string(),
        unit,
        at_cursor: (pv, cv, pv.zip(cv).map(|(x, y)| y - x)),
        largest,
        first_differs: first,
    }
}

/// The compare panel of the branch `child` against `parent`, with the branch's lineage in
/// lines, the plotted series and the cursor at report tick `cursor` of the branch (`None`:
/// live). `None` until both have loaded. The parent's earlier records are read for the
/// states before its start; the branch's are not.
pub fn build(
    parent: Side<'_>,
    child: Side<'_>,
    lineage: &[String],
    plots: &[SeriesKey],
    cursor: Option<u64>,
) -> Option<CompareVm> {
    let (pi, ci) = (identity(&parent)?, identity(&child)?);
    let (ps, cs) = (parent.store, child.store);
    let w = cs.world()?;
    let lo = parent.first().max(cs.start());
    let hi = ps.tick().min(cs.tick());
    let first = if lo <= hi {
        first_difference(&parent, cs, lo, hi)
    } else {
        None
    };
    let first_difference = first.map(|s| {
        let report = s.checked_sub(1);
        FirstDifferenceVm {
            state_tick: s,
            report_tick: report,
            date: report.map_or_else(String::new, |t| date(w, t)),
            parent: parent
                .state_hash_at(s)
                .map_or_else(String::new, |h| certify::Hex(h).to_string()),
            child: cs
                .state_hash_at(s)
                .map_or_else(String::new, |h| certify::Hex(h).to_string()),
            may_be_earlier: s == lo && lo > 0 && lo > cs.start(),
        }
    });
    let before_resume = cs.start() > 0 && first.is_some_and(|s| s <= cs.start());
    let tape = match (ps.tape(), cs.tape()) {
        (Some(a), Some(b)) => tape_diff(a, b),
        _ => Vec::new(),
    };
    let cursor = report_tick(cs, cursor);
    let series = plots
        .iter()
        .filter(|k| k.in_world(w))
        .map(|k| series_diff(&parent, cs, k, unit_of(w, k), cursor))
        .collect();
    Some(CompareVm {
        parent: pi,
        child: ci,
        compared: (lo, hi),
        first_difference,
        before_resume,
        tape,
        lineage: lineage.to_vec(),
        cursor,
        series,
    })
}
