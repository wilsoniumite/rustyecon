//! The watchlist's view-model (docs/GUI.md §4, the outliner at G1): each watched series by key
//! (U7), its value at the cursor with its unit, the tick before's, and the change, so a few
//! numbers can be followed while the run streams without plotting them. A watched key of
//! another world is listed apart, with no unit made up for it.

use super::{report_tick, unit_of};
use crate::run::{SeriesKey, Store};
use serde::Serialize;

/// One watched series.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WatchRowVm {
    /// The series.
    pub key: SeriesKey,
    /// Its name.
    pub label: String,
    /// Its unit.
    pub unit: String,
    /// Its value at the cursor's report tick, if it has one there.
    pub value: Option<f64>,
    /// Its value at the tick before, if it has one there.
    pub before: Option<f64>,
    /// The change from the tick before, where both are recorded.
    pub change: Option<f64>,
    /// Whether it is plotted too.
    pub plotted: bool,
}

/// The watchlist.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WatchVm {
    /// The cursor's report tick, once a tick has run.
    pub tick: Option<u64>,
    /// Each watched series of this run's world, in the order watched.
    pub rows: Vec<WatchRowVm>,
    /// The watched series this run's world has nothing of.
    pub absent: Vec<SeriesKey>,
}

/// The watchlist of `watch` at report tick `cursor` (`None`: live).
pub fn build(
    store: &Store,
    watch: &[SeriesKey],
    plots: &[SeriesKey],
    cursor: Option<u64>,
) -> Option<WatchVm> {
    let w = store.world()?;
    let tick = report_tick(store, cursor);
    let mut rows = Vec::new();
    let mut absent = Vec::new();
    for key in watch {
        if !key.in_world(w) {
            absent.push(key.clone());
            continue;
        }
        let s = store.series(key);
        let at = |t: Option<u64>| s.and_then(|s| s.at(t?));
        let value = at(tick);
        let before = at(tick.and_then(|t| t.checked_sub(1)));
        rows.push(WatchRowVm {
            key: key.clone(),
            label: key.to_string(),
            unit: unit_of(w, key),
            value,
            before,
            change: value.zip(before).map(|(v, b)| v - b),
            plotted: plots.contains(key),
        });
    }
    Some(WatchVm { tick, rows, absent })
}
