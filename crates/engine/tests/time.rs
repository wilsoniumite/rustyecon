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

/// What a run of the gate world at `tpy` ticks a year shows, until 1771.
struct Run {
    /// The 1750 endowments.
    year: Year,
    /// The tick each dated event fired in (its first occurrence), with the tick of its date.
    fired: Vec<(String, u64, u64)>,
    /// The tick of every occurrence of the yearly pension.
    pensions: Vec<u64>,
    /// The ticks, from mint to spoilage, that the longest-lived bread lot lasts.
    bread_life: u32,
    world: World,
}

fn run(tpy: u32) -> Run {
    let text = edit("ticks_per_year: 52,", &format!("ticks_per_year: {tpy},"));
    let t = tape_of(&text);
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let (grain, fuel) = (good(&w, "grain"), good(&w, "fuel"));
    let bread = good(&w, "bread");
    let mut year = Year {
        grain: 0.0,
        fuel: 0.0,
        ticks: 0,
    };
    let mut fired = Vec::new();
    let mut pensions = Vec::new();
    // The longest life any bread lot carries after ageing, plus the tick it was minted in.
    let mut bread_life = 0;
    loop {
        let r = sim.step().expect("the run succeeds");
        if r.date.y == 1750 {
            year.grain += audit_line(&r, grain, Provenance::Endowment);
            year.fuel += audit_line(&r, fuel, Provenance::Endowment);
            year.ticks += 1;
        }
        for e in &r.events {
            if e.key.as_str() == "pension" {
                pensions.push(r.tick);
            }
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
        for a in &w.actors {
            let inv = sim.holding(Holder::Actor(a.id)).expect("an inventory");
            for lot in inv.lots(bread) {
                bread_life = bread_life.max(lot.life.expect("bread spoils") + 1);
            }
        }
        if r.date.y > 1770 {
            break;
        }
    }
    Run {
        year,
        fired,
        pensions,
        bread_life,
        world: w,
    }
}

#[test]
fn a13_annual_quantities_invariant() {
    let (weekly, monthly) = (run(52), run(12));
    let (w52, w12) = (&weekly.world, &monthly.world);
    assert_eq!((weekly.year.ticks, monthly.year.ticks), (52, 12));
    for (a, b) in [
        (weekly.year.grain, monthly.year.grain),
        (weekly.year.fuel, monthly.year.fuel),
    ] {
        assert!(a > 0.0);
        assert!(
            (a - b).abs() <= REL * a,
            "1750 endowment {a} at 52 ticks, {b} at 12"
        );
    }
    // Each is the registered annual capacity.
    assert!((weekly.year.grain - 104.0).abs() <= REL * 104.0);
    assert!((weekly.year.fuel - 52.0).abs() <= REL * 52.0);
    // Every event fires in the tick its date falls in, at either tick length.
    for events in [&weekly.fired, &monthly.fired] {
        assert_eq!(events.len(), 5, "{events:?}");
        for (key, tick, expected) in events {
            assert_eq!(tick, expected, "{key}");
        }
    }
    // The dates of those ticks agree to within one tick (§6): at most a month apart.
    let days = |w: &World, t: u64| w.clock.date_of(t).unwrap().days();
    for ((key, t52, _), (other, t12, _)) in weekly.fired.iter().zip(&monthly.fired) {
        assert_eq!(key, other);
        let (d52, d12) = (days(w52, *t52), days(w12, *t12));
        assert!((d52 - d12).abs() <= 31, "{key}: day {d52} and day {d12}");
    }
    // A recurring period is a span in years (§6: `ticks`): the yearly pension fires once a
    // year at either tick length, every occurrence within a month of the other run's.
    assert_eq!(weekly.pensions.len(), 21, "1751 to 1771");
    assert_eq!(weekly.pensions.len(), monthly.pensions.len());
    for (k, (t52, t12)) in weekly.pensions.iter().zip(&monthly.pensions).enumerate() {
        let (d52, d12) = (days(w52, *t52), days(w12, *t12));
        assert!(
            (d52 - d12).abs() <= 31,
            "pension {k}: day {d52} and day {d12}"
        );
    }
    // A shelf life is a span in years too: three weeks is 3 weekly ticks and 1 monthly tick,
    // and a bread lot lasts that long, to within half a tick of 0.0577 years, at either length.
    assert_eq!((weekly.bread_life, monthly.bread_life), (3, 1));
    for (life, tpy) in [(weekly.bread_life, 52.0), (monthly.bread_life, 12.0)] {
        assert!(
            (f64::from(life) - 0.0577 * tpy).abs() <= 0.5,
            "{life} ticks at {tpy}"
        );
    }
}

#[test]
fn same_tick_events_fire_in_date_order() {
    // O9 (P0.9), A13, E8: two dated events that fall in one tick fire in date order, so what a
    // tape means does not change with its tick length or with how its keys are spelled. The
    // gate tape gains a cut of the mine on 1760-11-01, keyed "z.cut", and a restore twenty days
    // later, keyed "a.restore". At 52 or 365 ticks a year they fall in two ticks and the later
    // restore wins; at 12 they share a tick, where key order put the cut last and left the mine
    // cut for nine and a half years, until the tape's own 1770 restore. Now the restore wins at
    // every tick length, and at 12 the report lists the two in date order.
    let text = edit(
        "    events: [\n",
        "    events: [\n        (key: \"z.cut\", at: \"1760-11-01\", basis: Assumed(\"test\"), \
         act: SetParam(param: \"mine.capacity\", to: \"mine.capacity.cut\")),\n        \
         (key: \"a.restore\", at: \"1760-11-21\", basis: Assumed(\"test\"), \
         act: SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")),\n",
    );
    for tpy in [12u32, 52, 365] {
        let t = tape_of(&edit_text(
            &text,
            "ticks_per_year: 52,",
            &format!("ticks_per_year: {tpy},"),
        ));
        let mut sim = sim_of(&t);
        let w = sim.world().clone();
        let capacity = param(&w, "mine.capacity");
        let (cut, restore) = (tick_of(&w, "1760-11-01"), tick_of(&w, "1760-11-21"));
        assert_eq!(
            cut == restore,
            tpy == 12,
            "{tpy} ticks a year: {cut} and {restore}"
        );
        let mut order = Vec::new();
        sim.run_until(restore + 1, &mut |r| {
            for e in &r.events {
                if ["z.cut", "a.restore"].contains(&e.key.as_str()) {
                    order.push((r.tick, e.key.to_string()));
                }
            }
        })
        .expect("the run succeeds");
        assert_eq!(
            order,
            [
                (cut, "z.cut".to_string()),
                (restore, "a.restore".to_string())
            ],
            "{tpy} ticks a year"
        );
        assert_eq!(sim.param(capacity), Some(52.0), "{tpy} ticks a year");
        // And it stays restored until the tape's next event on the mine, the 1770 restore.
        sim.run_until(tick_of(&w, "1765-01-01"), &mut |_| {})
            .expect("the run succeeds");
        assert_eq!(
            sim.param(capacity),
            Some(52.0),
            "{tpy} ticks a year, in 1765"
        );
    }
}
