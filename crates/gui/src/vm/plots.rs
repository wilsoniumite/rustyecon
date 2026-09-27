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
    /// The plotted series this run's world has nothing of, such as another tape's: left out
    /// of the stack, with no unit made up for them.
    pub absent: Vec<SeriesKey>,
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

/// The plots of `plots`, with the cursor at report tick `cursor` (`None`: live). A plotted
/// series of another world is listed apart, in [`PlotsVm::absent`].
pub fn build(store: &Store, plots: &[SeriesKey], cursor: Option<u64>) -> Option<PlotsVm> {
    let w = store.world()?;
    let cursor = report_tick(store, cursor);
    let mut panels: Vec<PanelVm> = Vec::new();
    let mut absent = Vec::new();
    for key in plots {
        if !key.in_world(w) {
            absent.push(key.clone());
            continue;
        }
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
        absent,
    })
}

/// The label of a tick on a date-labelled axis. A mark a year or more from the next of its
/// weight is a year start ([`year_marks`]): the tick 1 January falls in, labelled by that
/// year, though the tick may begin in the old year (tick 51 begins on 1750-12-26 and holds
/// 1751-01-01). Any other mark is labelled by its tick's first date, so a year never labels a
/// tick that holds no 1 January. A mark that is not a tick of the calendar has none.
pub fn tick_label(clock: &Clock, x: f64, step: f64) -> String {
    if !(x >= 0.0 && x.fract() == 0.0 && x < 9.0e15) {
        return String::new();
    }
    let t = x as u64;
    let Some(d) = clock.date_of(t) else {
        return String::new();
    };
    if step >= f64::from(clock.ticks_per_year) {
        if let Some(y) = year_starting(clock, t) {
            return y.to_string();
        }
    }
    d.to_string()
}

/// The year whose 1 January falls in tick `t`, if one does.
fn year_starting(clock: &Clock, t: u64) -> Option<i32> {
    let d = clock.date_of(t)?;
    [d.y, d.y.checked_add(1)?].into_iter().find(|&y| {
        Date::new(y, 1, 1)
            .ok()
            .and_then(|jan1| clock.tick_of(jan1).ok())
            == Some(t)
    })
}

/// The steps between year marks, in years.
const YEAR_STEPS: [i32; 7] = [1, 5, 10, 50, 100, 500, 1000];

/// The grid marks of a date axis showing ticks `lo` to `hi`: the year starts, each the tick
/// its 1 January falls in (the ticks the timeline and the ring use), at the three smallest of
/// 1, 5, 10, 50, 100, 500 and 1,000 years that are at least `min_step` ticks apart. Each mark
/// is `(tick, step)`, its step the largest of the three its year is a multiple of, in ticks.
/// `None` when the axis shows less than two years, or more than it can mark: that axis takes
/// ordinary marks, labelled by date.
pub fn year_marks(clock: &Clock, lo: f64, hi: f64, min_step: f64) -> Option<Vec<(f64, f64)>> {
    let tpy = f64::from(clock.ticks_per_year);
    let finite = lo.is_finite() && hi.is_finite() && min_step.is_finite();
    if !finite || tpy < 1.0 || hi - lo < 2.0 * tpy || (hi - lo) / tpy > 1e6 {
        return None;
    }
    let first = YEAR_STEPS
        .iter()
        .position(|&s| f64::from(s) * tpy >= min_step)?;
    let steps = &YEAR_STEPS[first..(first + 3).min(YEAR_STEPS.len())];
    let s0 = steps[0];
    // Years are counted from the clock's start, a year of ticks at a time, with one to spare.
    let years = |x: f64| (x / tpy).floor() as i32;
    let start = clock.start.y;
    let y0 = start.saturating_add(years(lo.max(0.0))).saturating_sub(1);
    let y1 = start.saturating_add(years(hi.max(0.0))).saturating_add(1);
    let mut marks = Vec::new();
    let mut y = y0.max(start).div_euclid(s0) * s0;
    while y <= y1 {
        let t = Date::new(y, 1, 1).ok().and_then(|d| clock.tick_of(d).ok());
        if let Some(t) = t.map(|t| t as f64).filter(|t| (lo..=hi).contains(t)) {
            let step = steps.iter().rev().find(|&&s| y % s == 0).unwrap_or(&s0);
            marks.push((t, f64::from(*step) * tpy));
        }
        let Some(next) = y.checked_add(s0) else {
            break;
        };
        y = next;
    }
    Some(marks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate_clock() -> Clock {
        Clock {
            start: Date::parse("1750-01-01").unwrap(),
            ticks_per_year: 52,
        }
    }

    fn jan1(clock: &Clock, y: i32) -> u64 {
        clock.tick_of(Date::new(y, 1, 1).unwrap()).unwrap()
    }

    #[test]
    fn axis_labels_are_dates() {
        let clock = gate_clock();
        // 1751-01-01 falls in tick 51, which begins on 1750-12-26: a year mark there is 1751.
        assert_eq!(jan1(&clock, 1751), 51);
        assert_eq!(tick_label(&clock, 51.0, 52.0), "1751");
        assert_eq!(tick_label(&clock, 51.0, 13.0), "1750-12-26");
        // Tick 520 begins 3,653 days after 1750-01-01 (ceil(520 × 365.2425 / 52)), on
        // 1760-01-02; 1760-01-01 falls in tick 519. No year labels a tick without a 1 January.
        assert_eq!(jan1(&clock, 1760), 519);
        assert_eq!(tick_label(&clock, 519.0, 520.0), "1760");
        assert_eq!(tick_label(&clock, 520.0, 520.0), "1760-01-02");
        assert_eq!(tick_label(&clock, 520.0, 13.0), "1760-01-02");
        // Tick 1000 is 1769-03-26, mid-year: a date, never "1769".
        assert_eq!(tick_label(&clock, 1000.0, 1000.0), "1769-03-26");
        assert_eq!(tick_label(&clock, 0.0, 52.0), "1750");
        assert_eq!(tick_label(&clock, 0.5, 1.0), "");
        assert_eq!(tick_label(&clock, -52.0, 52.0), "");
    }

    #[test]
    fn year_marks_fall_on_year_starts() {
        let clock = gate_clock();
        // The gate's 40 years at about 100 ticks a mark: 5, 10 and 50 years.
        let marks = year_marks(&clock, 0.0, 2080.0, 100.0).unwrap();
        let years: Vec<i32> = (1750..=1790).step_by(5).collect();
        assert_eq!(
            marks.iter().map(|m| m.0).collect::<Vec<_>>(),
            years
                .iter()
                .map(|&y| jan1(&clock, y) as f64)
                .collect::<Vec<_>>()
        );
        for ((t, step), y) in marks.iter().zip(&years) {
            let want = if y % 50 == 0 {
                50
            } else if y % 10 == 0 {
                10
            } else {
                5
            };
            assert_eq!(*step, f64::from(want * 52), "{y}");
            assert_eq!(tick_label(&clock, *t, *step), y.to_string());
        }
        // Every year at a mark a tick apart; only the visible years.
        let marks = year_marks(&clock, 100.0, 400.0, 1.0).unwrap();
        let ticks: Vec<f64> = marks.iter().map(|m| m.0).collect();
        let want: Vec<f64> = (1752..=1757).map(|y| jan1(&clock, y) as f64).collect();
        assert_eq!(ticks, want);
        // Less than two years: ordinary marks, labelled by date.
        assert_eq!(year_marks(&clock, 0.0, 100.0, 1.0), None);
        assert_eq!(year_marks(&clock, 0.0, f64::INFINITY, 1.0), None);
        assert_eq!(year_marks(&clock, 0.0, 2080.0, 1e9), None);
        // An axis dragged far past any calendar year marks nothing, and does not overflow.
        assert_eq!(
            year_marks(&clock, 1e15, 1e15 + 2080.0, 100.0),
            Some(Vec::new())
        );
    }
}
