//! The timeline's view-model (docs/GUI.md §4): the scrubber's span, the cursor, the fired and
//! the scheduled events, and the ring checkpoints.
//!
//! The span runs from the record's first state tick to the later of the state's tick and the
//! last dated event, so an event still to come has a place. A fired event is one the reports
//! carried; a scheduled one is still to run. Each year's first tick is marked with its year.

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
    /// Every firing in the span, fired and scheduled, in firing order.
    pub events: Vec<EventMark>,
    /// The state ticks of the ring's checkpoints.
    pub ring: Vec<u64>,
}

/// The timeline of a run, with the cursor at report tick `cursor` (`None`: live).
pub fn build(store: &Store, cursor: Option<u64>) -> Option<TimelineVm> {
    let w = store.world()?;
    let start = store.start();
    let now = store.tick();
    let last_dated = w.schedule.once().last().map_or(0, |f| f.tick + 1);
    let end = now.max(last_dated).max(start + 1);
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
    let cursor = report_tick(store, cursor).map(|t| CursorVm {
        tick: t,
        date: date(w, t),
        live: cursor.is_none(),
    });
    Some(TimelineVm {
        start,
        now,
        end,
        ticks_per_year: w.clock.ticks_per_year,
        cursor,
        years,
        events,
        ring: store.ring().iter().map(|c| c.tick()).collect(),
    })
}
