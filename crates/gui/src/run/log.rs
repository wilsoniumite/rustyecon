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

use super::{At, Measure, Obs, ObsBatch, PauseReason, RunId, SeriesKey};
use serde::Serialize;
use std::collections::BTreeMap;

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

/// One class line the watch knows: where its requested and filled series sit in the
/// catalogue, and whether its onset is logged.
#[derive(Debug, Clone, Copy, Default)]
struct Line {
    requested: Option<usize>,
    filled: Option<usize>,
    rationed: bool,
}

/// Watches every class line of a run for the onset of rationing. One per run, fed each batch
/// before the store records it, and started afresh by a load.
///
/// As built at D.3, it learns each class line's two series once, as the catalogue grows, and
/// reads each row through a scratch column by catalogue index: the demo world's rows hold
/// 5,970 cells and 1,116 class lines, and building a map of every row's lines cost the UI
/// thread about a millisecond a tick. The onsets, and their order within a tick (by line), are
/// as before.
#[derive(Debug, Clone, Default)]
pub struct RationWatch {
    /// Every class line seen, in line order.
    lines: BTreeMap<At, Line>,
    /// How much of the catalogue `lines` has read.
    read: usize,
    /// One row's values by catalogue index, NaN where the row has none.
    scratch: Vec<f64>,
}

impl RationWatch {
    fn learn(&mut self, i: usize, k: &SeriesKey) {
        match k.measure {
            Measure::Requested => self.lines.entry(k.at.clone()).or_default().requested = Some(i),
            Measure::Filled => self.lines.entry(k.at.clone()).or_default().filled = Some(i),
            _ => {}
        }
    }

    /// The onset lines of a batch of run `run`, whose series `batch.known..` are the batch's
    /// new ones and whose earlier ones are `catalogue`'s.
    pub fn onsets(&mut self, run: RunId, batch: &ObsBatch, catalogue: &[SeriesKey]) -> Vec<Entry> {
        let known = batch.known as usize;
        for (i, k) in catalogue.iter().enumerate().take(known).skip(self.read) {
            self.learn(i, k);
        }
        for (j, k) in batch.new_series.iter().enumerate() {
            self.learn(known + j, k);
        }
        self.read = self.read.max(known + batch.new_series.len());
        let n = self.read;
        let mut out = Vec::new();
        for row in &batch.rows {
            self.scratch.clear();
            self.scratch.resize(n, f64::NAN);
            for &(i, v) in &row.cells {
                if let Some(x) = self.scratch.get_mut(i as usize) {
                    *x = v;
                }
            }
            for (at, line) in &mut self.lines {
                let (false, Some(r), Some(f)) = (line.rationed, line.requested, line.filled) else {
                    continue;
                };
                let (requested, filled) = (self.scratch[r], self.scratch[f]);
                // A line the row lacks is NaN, and NaN is never below anything.
                if filled < requested {
                    line.rationed = true;
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
    /// The report tick it concerns, if one: the tick that ran, as the cursor counts (docs/GUI.md
    /// §4). A line about a state names the tick that left it, state tick − 1, and none for the
    /// state at tick 0, which no tick left. A click on the line moves the cursor there.
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
/// events add none. Every line's tick is a report tick ([`Entry::tick`]); a line about a state
/// says "state tick" in its text.
pub fn entries(run: RunId, obs: &Obs) -> Vec<Entry> {
    let line = |tick: Option<u64>, level: Level, text: String| Entry {
        run: Some(run),
        tick,
        level,
        text,
    };
    // The report tick that left a state: none for the state at tick 0.
    let left = |state: u64| state.checked_sub(1);
    match obs {
        Obs::Loaded {
            run: key,
            world,
            tick,
            hash,
            ..
        } => vec![line(
            left(*tick),
            Level::Info,
            format!(
                "loaded {} at state tick {tick}: tape_hash {}, world_id {}, state 0x{hash:016x}",
                world.name, key.tape_hash, key.world_id
            ),
        )],
        Obs::Running { tick } => vec![line(
            left(*tick),
            Level::Info,
            format!("running from state tick {tick}"),
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
        Obs::Paused { tick, why } => {
            let text = match why {
                PauseReason::Reached(u) => format!("reached state tick {u}"),
                _ => format!("{why} at state tick {tick}"),
            };
            vec![line(left(*tick), Level::Info, text)]
        }
        Obs::Checkpointed(_) => Vec::new(),
        Obs::Snapshot(s) => vec![line(
            left(s.tick),
            Level::Info,
            format!("snapshot of state tick {}", s.tick),
        )],
        Obs::Failed { error, last } => {
            // The failed tick ran, in part, and left no report; the last good tick did.
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
                line(last.as_ref().map(|r| r.tick), Level::Error, good),
            ]
        }
        Obs::Refused(r) => vec![line(None, Level::Error, r.to_string())],
        Obs::Ended => vec![line(
            None,
            Level::Error,
            "the run's worker ended without being told to stop (it panicked); the run can go \
             no further"
                .to_string(),
        )],
    }
}
