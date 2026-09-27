//! The log's view-model (docs/GUI.md §4): its lines, oldest first, with the run and tick each
//! concerns and whether it is an error.

use crate::run::log::{Entry, Level};
use serde::Serialize;

/// One line.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LineVm {
    /// The run, by its number in this process.
    pub run: Option<u32>,
    /// The tick.
    pub tick: Option<u64>,
    /// Whether it is an error.
    pub error: bool,
    /// The text.
    pub text: String,
}

/// The log.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogVm {
    /// Every line, oldest first.
    pub lines: Vec<LineVm>,
}

/// The log of these lines.
pub fn build(entries: &[Entry]) -> LogVm {
    LogVm {
        lines: entries
            .iter()
            .map(|e| LineVm {
                run: e.run.map(|r| r.0),
                tick: e.tick,
                error: e.level == Level::Error,
                text: e.text.clone(),
            })
            .collect(),
    }
}
