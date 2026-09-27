//! The log's lines (docs/GUI.md §4, the log panel): loads, runs, pauses, fired events, refusals
//! and errors, made from what a run reports. A failed run's lines give its ledger line and its
//! last good tick, as the cli's stderr does.

use super::{Obs, RunId};
use serde::Serialize;

/// How a line reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Level {
    /// A load, a run, a pause.
    Info,
    /// A tape event that fired.
    Event,
    /// An error: a failed step, a refusal, a file that could not be read or written.
    Error,
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
