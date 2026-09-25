//! Tick length (docs/ENGINE.md §6 and §11, engine; A13): the tick length is registered on the
//! tape, and every dial carries a time unit, so changing only `ticks_per_year` keeps annual
//! quantities and event dates.
//!
//! The one bar here, `REL`, is relative: a year's endowment is a left fold of 52 (or 12) equal
//! per-tick mints `capacity/ticks_per_year`, each fold step rounding within 2⁻⁵³ of the running
//! sum, so the two totals differ by at most about 64 ulps of the total, some 1e-14 of it; 1e-12
//! leaves room and still fails on any per-tick misconversion.

mod common;

use common::*;
use rustyecon_engine::prelude::*;

/// The relative bar of this file (see the module comment).
const REL: f64 = 1e-12;

struct Year {
    grain: f64,
    fuel: f64,
    ticks: u64,
}

/// The 1750 endowments of the gate world at `tpy` ticks a year, and the tick each dated event
/// fired in, run until 1771.
fn run(tpy: u32) -> (Year, Vec<(String, u64, u64)>, World) {
    let text = edit("ticks_per_year: 52,", &format!("ticks_per_year: {tpy},"));
    let t = tape_of(&text);
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let (grain, fuel) = (good(&w, "grain"), good(&w, "fuel"));
    let mut year = Year {
        grain: 0.0,
        fuel: 0.0,
        ticks: 0,
    };
    let mut fired = Vec::new();
    loop {
        let r = sim.step().expect("the run succeeds");
        if r.date.y == 1750 {
            year.grain += audit_line(&r, grain, Provenance::Endowment);
            year.fuel += audit_line(&r, fuel, Provenance::Endowment);
            year.ticks += 1;
        }
        for e in &r.events {
            if e.occurrence == 0 {
                let date = t
                    .events
                    .iter()
                    .find(|x| x.key == e.key)
                    .map(|x| x.at)
                    .or_else(|| t.recurring.iter().find(|x| x.key == e.key).map(|x| x.first))
                    .expect("a tape event");
                let expected = w.clock.tick_of(date).expect("after the start");
                fired.push((e.key.to_string(), r.tick, expected));
            }
        }
        if r.date.y > 1770 {
            break;
        }
    }
    (year, fired, w)
}

#[test]
fn a13_annual_quantities_invariant() {
    let (weekly, weekly_events, w52) = run(52);
    let (monthly, monthly_events, w12) = run(12);
    assert_eq!((weekly.ticks, monthly.ticks), (52, 12));
    for (a, b) in [(weekly.grain, monthly.grain), (weekly.fuel, monthly.fuel)] {
        assert!(a > 0.0);
        assert!(
            (a - b).abs() <= REL * a,
            "1750 endowment {a} at 52 ticks, {b} at 12"
        );
    }
    // Each is the registered annual capacity.
    assert!((weekly.grain - 104.0).abs() <= REL * 104.0);
    assert!((weekly.fuel - 52.0).abs() <= REL * 52.0);
    // Every event fires in the tick its date falls in, at either tick length.
    for events in [&weekly_events, &monthly_events] {
        assert_eq!(events.len(), 5, "{events:?}");
        for (key, tick, expected) in events {
            assert_eq!(tick, expected, "{key}");
        }
    }
    // The dates of those ticks agree to within one tick (§6): at most a month apart.
    for ((key, t52, _), (other, t12, _)) in weekly_events.iter().zip(&monthly_events) {
        assert_eq!(key, other);
        let d52 = w52.clock.date_of(*t52).unwrap().days();
        let d12 = w12.clock.date_of(*t12).unwrap().days();
        assert!((d52 - d12).abs() <= 31, "{key}: day {d52} and day {d12}");
    }
}
