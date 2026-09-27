//! The log's lines (docs/GUI.md §4, the log panel): loads, runs, pauses, fired events, rationing
//! onsets by class, refusals and errors, made from what a run reports. A failed run's lines give
//! its ledger line and its last good tick, as the cli's stderr does.
//!
//! A rationing onset is the first tick of a load in which a class's line in a market (R12) was
//! filled below what it requested: the report's own numbers, compared exactly, so the GUI
//! defines no tolerance, and one line for each class line, so a market at rest whose fills
//! differ from its requests by rounding does not fill the log. Later episodes are in the class
//! lines' series, which the inspector shows and plots. A tolerance on a fill is a criterion's
//! registered bar (certify's `rationed_below`), not the log's.

use super::{At, Measure, Obs, ObsBatch, RunId, SeriesKey};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// How a line reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Level {
    /// A load, a run, a pause.
    Info,
    /// A tape event that fired.
    Event,
    /// A class's line began to be rationed.
    Rationing,
    /// An error: a failed step, a refusal, a file that could not be read or written.
    Error,
}

/// Watches every class line of a run for the onset of rationing. One per run, fed each batch
/// before the store records it, and started afresh by a load.
#[derive(Debug, Clone, Default)]
pub struct RationWatch {
    /// The class lines already filled below their request once.
    rationed: BTreeSet<At>,
}

impl RationWatch {
    /// The onset lines of a batch of run `run`, whose series `batch.known..` are the batch's
    /// new ones and whose earlier ones are `catalogue`'s.
    pub fn onsets(&mut self, run: RunId, batch: &ObsBatch, catalogue: &[SeriesKey]) -> Vec<Entry> {
        let key = |i: u32| -> Option<&SeriesKey> {
            let i = i as usize;
            let known = batch.known as usize;
            if i < known {
                catalogue.get(i)
            } else {
                batch.new_series.get(i - known)
            }
        };
        let mut out = Vec::new();
        for row in &batch.rows {
            let mut lines: BTreeMap<&At, (Option<f64>, Option<f64>)> = BTreeMap::new();
            for &(i, v) in &row.cells {
                let Some(k) = key(i) else { continue };
                match k.measure {
                    Measure::Requested => lines.entry(&k.at).or_default().0 = Some(v),
                    Measure::Filled => lines.entry(&k.at).or_default().1 = Some(v),
                    _ => {}
                }
            }
            for (at, line) in lines {
                let (Some(requested), Some(filled)) = line else {
                    continue;
                };
                if filled < requested && self.rationed.insert(at.clone()) {
                    out.push(Entry {
                        run: Some(run),
                        tick: Some(row.tick),
                        level: Level::Rationing,
                        text: onset_text(at, requested, filled),
                    });
                }
            }
        }
        out
    }
}

fn onset_text(at: &At, requested: f64, filled: f64) -> String {
    match at {
        At::Class {
            node,
            good,
            class,
            side,
        } => {
            let side = match side {
                rustyecon_engine::prelude::SideTag::Buy => "buy",
                rustyecon_engine::prelude::SideTag::Sell => "sell",
            };
            format!(
                "rationing onset: {class} ({side}) at {node}/{good}, filled {filled} of {requested} \
                 requested"
            )
        }
        other => format!("rationing onset at {other:?}: filled {filled} of {requested} requested"),
    }
}

/// One line of the log.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entry {
    /// The run it concerns, if one.
    pub run: Option<RunId>,
    /// The tick it concerns, if one.
    pub tick: Option<u64>,
    /// How it reads.
    pub level: Level,
    /// The text.
    pub text: String,
}

impl Entry {
    /// A line about no run in particular.
    pub fn note(level: Level, text: impl Into<String>) -> Entry {
        Entry {
            run: None,
            tick: None,
            level,
            text: text.into(),
        }
    }
}

/// The lines an observation of run `run` adds to the log. Ring checkpoints and batches without
/// events add none.
pub fn entries(run: RunId, obs: &Obs) -> Vec<Entry> {
    let line = |tick: Option<u64>, level: Level, text: String| Entry {
        run: Some(run),
        tick,
        level,
        text,
    };
    match obs {
        Obs::Loaded {
            run: key,
            world,
            tick,
            hash,
            ..
        } => vec![line(
            Some(*tick),
            Level::Info,
            format!(
                "loaded {} at tick {tick}: tape_hash {}, world_id {}, state 0x{hash:016x}",
                world.name, key.tape_hash, key.world_id
            ),
        )],
        Obs::Running { tick } => vec![line(
            Some(*tick),
            Level::Info,
            format!("running from tick {tick}"),
        )],
        Obs::Batch(b) => b
            .rows
            .iter()
            .flat_map(|r| r.events.iter().map(move |e| (r.tick, e)))
            .map(|(tick, e)| {
                let from = e
                    .source
                    .as_ref()
                    .map(|s| format!(", from {s}"))
                    .unwrap_or_default();
                let nth = if e.occurrence > 0 {
                    format!(" (occurrence {})", e.occurrence)
                } else {
                    String::new()
                };
                line(
                    Some(tick),
                    Level::Event,
                    format!("event {}{nth}: {}{from}", e.key, e.action.name()),
                )
            })
            .collect(),
        Obs::Paused { tick, why } => vec![line(
            Some(*tick),
            Level::Info,
            format!("{why} at tick {tick}"),
        )],
        Obs::Checkpointed(_) => Vec::new(),
        Obs::Snapshot(s) => vec![line(
            Some(s.tick),
            Level::Info,
            format!("snapshot of tick {}", s.tick),
        )],
        Obs::Failed { error, last } => {
            let good = match last {
                Some(r) => format!(
                    "the last good tick is {} (state tick {})",
                    r.tick,
                    r.tick + 1
                ),
                None => format!("no tick completed; the run started at tick {}", error.tick),
            };
            vec![
                line(
                    Some(error.tick),
                    Level::Error,
                    format!("run error: {error}"),
                ),
                line(Some(error.tick), Level::Error, good),
            ]
        }
        Obs::Refused(r) => vec![line(None, Level::Error, r.to_string())],
    }
}
