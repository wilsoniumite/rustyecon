//! The plots' view-model (docs/GUI.md §4, §3.4): the plotted series, stacked one panel per
//! unit, so every axis names its unit and no two units share one. The x axis is the report
//! tick, labelled by its date. Each line gives what the run recorded of it: its number of
//! points, its first, last, least and greatest points, and its value at the cursor.
//!
//! The points drawn are not here: the plot cache in `ui/` thins each series with
//! [`run::Decimator`](crate::run::Decimator) and lends egui the recorded points it keeps.

use super::{report_tick, unit_of};
use crate::run::{SeriesKey, Store};
use rustyecon_engine::prelude::*;
use serde::Serialize;

/// One plotted series.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LineVm {
    /// The series.
    pub key: SeriesKey,
    /// Its name, as the legend shows it.
    pub label: String,
    /// The points recorded.
    pub points: usize,
    /// The first point recorded.
    pub first: Option<(u64, f64)>,
    /// The last point recorded.
    pub last: Option<(u64, f64)>,
    /// The least value and the first tick it was recorded at.
    pub min: Option<(u64, f64)>,
    /// The greatest value and the first tick it was recorded at.
    pub max: Option<(u64, f64)>,
    /// The value at the cursor's report tick, if the series has one there.
    pub at_cursor: Option<f64>,
}

/// One panel of the stack: every plotted series in one unit.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PanelVm {
    /// The unit, the y axis's label.
    pub unit: String,
    /// Its series, in the order they were plotted.
    pub lines: Vec<LineVm>,
}

/// The plots.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlotsVm {
    /// The record's first report tick.
    pub start: u64,
    /// The state's tick: report ticks run to `now - 1`.
    pub now: u64,
    /// The cursor's report tick, once a tick has run.
    pub cursor: Option<u64>,
    /// Ticks per year, for the date labels.
    pub ticks_per_year: u32,
    /// The panels, in the order their units were first plotted.
    pub panels: Vec<PanelVm>,
}

fn line(store: &Store, key: &SeriesKey, cursor: Option<u64>) -> LineVm {
    let s = store.series(key);
    let points = s.map_or(0, |s| s.len());
    let mut min: Option<(u64, f64)> = None;
    let mut max: Option<(u64, f64)> = None;
    if let Some(s) = s {
        for (&t, &v) in s.ticks().iter().zip(s.values()) {
            if min.is_none_or(|m| v < m.1) {
                min = Some((t, v));
            }
            if max.is_none_or(|m| v > m.1) {
                max = Some((t, v));
            }
        }
    }
    LineVm {
        key: key.clone(),
        label: key.to_string(),
        points,
        first: s.and_then(|s| Some((*s.ticks().first()?, *s.values().first()?))),
        last: s.and_then(|s| s.last()),
        min,
        max,
        at_cursor: s.and_then(|s| s.at(cursor?)),
    }
}

/// The plots of `plots`, with the cursor at report tick `cursor` (`None`: live).
pub fn build(store: &Store, plots: &[SeriesKey], cursor: Option<u64>) -> Option<PlotsVm> {
    let w = store.world()?;
    let cursor = report_tick(store, cursor);
    let mut panels: Vec<PanelVm> = Vec::new();
    for key in plots {
        let unit = unit_of(w, key);
        let l = line(store, key, cursor);
        match panels.iter_mut().find(|p| p.unit == unit) {
            Some(p) => p.lines.push(l),
            None => panels.push(PanelVm {
                unit,
                lines: vec![l],
            }),
        }
    }
    Some(PlotsVm {
        start: store.start(),
        now: store.tick(),
        cursor,
        ticks_per_year: w.clock.ticks_per_year,
        panels,
    })
}

/// The label of a tick on a date-labelled axis: the year when the marks are a year or more
/// apart, else the date. A mark that is not a tick of the calendar has none.
pub fn tick_label(clock: &Clock, x: f64, step: f64) -> String {
    if !(x >= 0.0 && x.fract() == 0.0 && x < 9.0e15) {
        return String::new();
    }
    let Some(d) = clock.date_of(x as u64) else {
        return String::new();
    };
    if step >= f64::from(clock.ticks_per_year) {
        d.y.to_string()
    } else {
        d.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_labels_are_dates() {
        let clock = Clock {
            start: Date::parse("1750-01-01").unwrap(),
            ticks_per_year: 52,
        };
        // Tick 520 begins 3,653 days after 1750-01-01 (ceil(520 × 365.2425 / 52)).
        assert_eq!(tick_label(&clock, 520.0, 520.0), "1760");
        assert_eq!(tick_label(&clock, 520.0, 13.0), "1760-01-02");
        assert_eq!(tick_label(&clock, 0.0, 52.0), "1750");
        assert_eq!(tick_label(&clock, 0.5, 1.0), "");
        assert_eq!(tick_label(&clock, -52.0, 52.0), "");
    }
}
