//! The timeline's view-model (docs/GUI.md §4): the scrubber's span, the cursor, the fired and
//! the scheduled events, and the ring checkpoints.
//!
//! The span runs from the record's first state tick to the later of the state's tick and the
//! last dated event, so an event still to come has a place. A fired event is one the reports
//! carried; a scheduled one is still to run. Each year's first tick is marked with its year.
//!
//! As built at D.3: a span of more than [`LISTED`] firings, such as the demo world's 30,078
//! dated steps, lists only the firings within a year of the cursor, and counts every firing of
//! the span a year at a time for the strip. A shorter schedule is listed whole, as before.

use super::{date, describe, firing_date, report_tick};
use crate::run::{ring_tick_at_or_after, Store};
use rustyecon_engine::prelude::*;
use serde::Serialize;
use std::collections::BTreeSet;

/// Where the cursor is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CursorVm {
    /// The report tick.
    pub tick: u64,
    /// Its first date.
    pub date: String,
    /// Whether it follows the latest tick.
    pub live: bool,
}

/// One firing on the timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventMark {
    /// The tick it fires in.
    pub tick: u64,
    /// Its date: a dated event's own, a recurring occurrence's tick's first day.
    pub date: String,
    /// The event's key.
    pub key: Key,
    /// 0 for a dated event; k for a recurring entry's k-th firing.
    pub occurrence: u32,
    /// Whether a report carried it. A firing still to run is scheduled.
    pub fired: bool,
    /// What it does, by key, with the param it copies from.
    pub what: String,
}

/// A year's first tick, labelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct YearMark {
    /// The tick 1 January falls in.
    pub tick: u64,
    /// The year.
    pub year: i32,
}

/// How many firings fall in one year of the span, for a schedule too long to list whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct YearCount {
    /// The tick 1 January falls in.
    pub tick: u64,
    /// The year.
    pub year: i32,
    /// Its firings before the state's tick: fired.
    pub fired: u32,
    /// Its firings from the state's tick on: scheduled.
    pub scheduled: u32,
}

/// A span with more firings than this lists only those within a year of the cursor, and counts
/// the rest a year at a time (D.3: the demo world's history is 30,078 dated steps).
pub const LISTED: usize = 2_000;

/// The timeline.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimelineVm {
    /// The record's first state tick.
    pub start: u64,
    /// The state's tick: every report tick before it has run.
    pub now: u64,
    /// The scrubber's end.
    pub end: u64,
    /// Ticks per year.
    pub ticks_per_year: u32,
    /// The cursor, once a tick has run.
    pub cursor: Option<CursorVm>,
    /// Each year's first tick in the span.
    pub years: Vec<YearMark>,
    /// Every firing in the span, fired and scheduled, in firing order; for a span of more than
    /// [`LISTED`], those within a year of the cursor.
    pub events: Vec<EventMark>,
    /// The state ticks of the ring's checkpoints.
    pub ring: Vec<u64>,
    /// For a span of more than [`LISTED`] firings, every firing in it, counted a year at a
    /// time. Empty otherwise.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub density: Vec<YearCount>,
    /// For a span of more than [`LISTED`] firings, how many it holds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
}

/// The ticks of a recurring entry's occurrences in `[lo, hi)`, with their occurrence numbers:
/// it fires at `first + k·period` up to `last`.
fn occurrences(first: u64, period: u64, last: Option<u64>, lo: u64, hi: u64) -> Vec<(u64, u32)> {
    let mut out = Vec::new();
    let period = period.max(1);
    let mut k = lo.saturating_sub(first).div_ceil(period);
    while let Some(t) = k.checked_mul(period).and_then(|d| first.checked_add(d)) {
        if t >= hi || last.is_some_and(|l| t > l) {
            break;
        }
        out.push((t, u32::try_from(k).unwrap_or(u32::MAX)));
        k += 1;
    }
    out
}

/// The timeline of a run, with the cursor at report tick `cursor` (`None`: live).
pub fn build(store: &Store, cursor: Option<u64>) -> Option<TimelineVm> {
    let w = store.world()?;
    let start = store.start();
    let now = store.tick();
    let last_dated = w.schedule.once().last().map_or(0, |f| f.tick + 1);
    let end = now.max(last_dated).max(start + 1);
    let once = w.schedule.once();
    let (lo, hi) = (
        once.partition_point(|f| f.tick < start),
        once.partition_point(|f| f.tick < end),
    );
    let recurring: usize = w
        .schedule
        .every()
        .iter()
        .map(|r| occurrences(r.first, r.period, r.last, start, end).len())
        .sum();
    let total = hi - lo + recurring;
    let mut years = Vec::new();
    let mut t = start;
    while let Some(r) = ring_tick_at_or_after(&w.clock, t) {
        if r > end {
            break;
        }
        // The tick 1 January of year Y falls in begins on that day or in the year before.
        let year = w
            .clock
            .date_of(r)
            .map_or(0, |d| if (d.m, d.d) == (1, 1) { d.y } else { d.y + 1 });
        years.push(YearMark { tick: r, year });
        t = r + 1;
    }
    let cursor_vm = report_tick(store, cursor).map(|t| CursorVm {
        tick: t,
        date: date(w, t),
        live: cursor.is_none(),
    });
    let (events, density, total) = if total <= LISTED {
        (every_firing(store, w, start, end), Vec::new(), None)
    } else {
        let tpy = u64::from(w.clock.ticks_per_year);
        let centre = cursor_vm.as_ref().map_or(now, |c| c.tick);
        let (wlo, whi) = (
            centre.saturating_sub(tpy).max(start),
            (centre + tpy).min(end),
        );
        (
            firings_between(store, w, wlo, whi),
            counts(w, &years, start, end, now),
            Some(total),
        )
    };
    Some(TimelineVm {
        start,
        now,
        end,
        ticks_per_year: w.clock.ticks_per_year,
        cursor: cursor_vm,
        years,
        events,
        ring: store.ring().iter().map(|c| c.tick()).collect(),
        density,
        total,
    })
}

/// Every firing in the span, from the schedule's own list.
fn every_firing(store: &Store, w: &World, start: u64, end: u64) -> Vec<EventMark> {
    let fired: BTreeSet<(u64, &Key, u32)> = store
        .events()
        .iter()
        .map(|(t, e)| (*t, &e.key, e.occurrence))
        .collect();
    let mut events = Vec::new();
    for f in w.schedule.firings_before(end) {
        if f.tick < start {
            continue;
        }
        let Some(key) = w.key_of(f.event) else {
            continue;
        };
        let mut what = describe(w, &f.action);
        if let Some(s) = &f.source {
            what.push_str(&format!(", from {s}"));
        }
        events.push(EventMark {
            tick: f.tick,
            date: firing_date(w, f.tick, key),
            key: key.clone(),
            occurrence: f.occurrence,
            fired: fired.contains(&(f.tick, key, f.occurrence)),
            what,
        });
    }
    events
}

/// The firings with ticks in `[lo, hi)`, in firing order: the dated ones found by bisection,
/// the recurring ones by arithmetic. Fired when a report carried them.
fn firings_between(store: &Store, w: &World, lo: u64, hi: u64) -> Vec<EventMark> {
    let recorded = store.events();
    let (a, b) = (
        recorded.partition_point(|(t, _)| *t < lo),
        recorded.partition_point(|(t, _)| *t < hi),
    );
    let fired: BTreeSet<(u64, &Key, u32)> = recorded[a..b]
        .iter()
        .map(|(t, e)| (*t, &e.key, e.occurrence))
        .collect();
    let once = w.schedule.once();
    let (i, j) = (
        once.partition_point(|f| f.tick < lo),
        once.partition_point(|f| f.tick < hi),
    );
    let mut marks: Vec<(u64, i64, &Key, EventMark)> = Vec::new();
    let mark = |tick: u64, day: i64, key: &Key, occurrence: u32, what: String| EventMark {
        tick,
        date: Date::from_days(day).map_or_else(|| date(w, tick), |d| d.to_string()),
        key: key.clone(),
        occurrence,
        fired: fired.contains(&(tick, key, occurrence)),
        what,
    };
    for f in &once[i..j] {
        let Some(key) = w.key_of(f.event) else {
            continue;
        };
        let mut what = describe(w, &f.action);
        if let Some(s) = &f.source {
            what.push_str(&format!(", from {s}"));
        }
        let m = mark(f.tick, f.day, key, f.occurrence, what);
        marks.push((f.tick, f.day, key, m));
    }
    for r in w.schedule.every() {
        let Some(key) = w.key_of(r.event) else {
            continue;
        };
        for (t, k) in occurrences(r.first, r.period, r.last, lo, hi) {
            let mut what = describe(w, &r.action);
            if let Some(s) = &r.source {
                what.push_str(&format!(", from {s}"));
            }
            let day = w.clock.first_day(t);
            let m = mark(t, day, key, k, what);
            marks.push((t, day, key, m));
        }
    }
    marks.sort_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)));
    marks.into_iter().map(|(_, _, _, m)| m).collect()
}

/// Every firing of the span, counted a year at a time: fired before the state's tick,
/// scheduled from it on.
fn counts(w: &World, years: &[YearMark], start: u64, end: u64, now: u64) -> Vec<YearCount> {
    let mut out: Vec<YearCount> = years
        .iter()
        .map(|y| YearCount {
            tick: y.tick,
            year: y.year,
            fired: 0,
            scheduled: 0,
        })
        .collect();
    if out.is_empty() {
        return out;
    }
    let mut add = |t: u64| {
        let k = out.partition_point(|y| y.tick <= t).saturating_sub(1);
        if t < now {
            out[k].fired += 1;
        } else {
            out[k].scheduled += 1;
        }
    };
    let once = w.schedule.once();
    let (i, j) = (
        once.partition_point(|f| f.tick < start),
        once.partition_point(|f| f.tick < end),
    );
    for f in &once[i..j] {
        add(f.tick);
    }
    for r in w.schedule.every() {
        for (t, _) in occurrences(r.first, r.period, r.last, start, end) {
            add(t);
        }
    }
    out
}
