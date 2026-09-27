//! The price-step explainer and the log waterfall (docs/GUI.md §4, "Why is this price 12.3?";
//! §9 G1's gate): the explainer, which calls markets' own `imbalance` and `next_price` on each
//! tick's recorded inputs, equals the engine's next price bit for bit on every tick of the
//! gate world, and of the Appendix B world; and the waterfall's Σ k·x and residual add up to
//! ln(p/p₀).

mod common;

use common::{driver, tape_of, wait_for, wait_paused, APPB, APPB_TICKS, GATE, GATE_TICKS};
use rustyecon_engine::num;
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Driver;
use rustyecon_gui::run::{Cmd, Obs, PauseReason, Store};
use rustyecon_gui::vm::pricestep::{explain, waterfall};

/// A store of `t` run to `until` through a `ThreadDriver`, and the engine's own reports of the
/// same run, from `Sim::new` and `run_until`.
fn record(t: &Tape, until: u64) -> (Store, Vec<TickReport>) {
    let mut d = driver();
    let mut log = Vec::new();
    d.send(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
    wait_for(&mut d, &mut log, |o| matches!(o, Obs::Loaded { .. }));
    d.send(Cmd::Run {
        until: Some(until),
        max_tps: None,
    });
    assert_eq!(
        wait_paused(&mut d, &mut log),
        (until, PauseReason::Reached(until))
    );
    drop(d);
    let mut store = Store::default();
    for o in log {
        store.ingest(o).expect("every observation ingests");
    }
    let mut sim = Sim::new(t).expect("the tape loads");
    let mut reports = Vec::new();
    sim.run_until(until, &mut |r| reports.push(r.clone()))
        .expect("the run succeeds");
    (store, reports)
}

/// Every tick and market of a run: the explainer's next price is the engine's, bit for bit.
fn every_tick(name: &str, text: &str, until: u64) -> Store {
    let t = tape_of(text);
    let (store, reports) = record(&t, until);
    let w = store.world().expect("loaded").clone();
    let mut checked = 0;
    for r in &reports {
        for m in &r.markets {
            let node = w.key_of(m.node).expect("a node key");
            let good = w.key_of(m.good).expect("a good key");
            let e = explain(&store, node, good, r.tick)
                .unwrap_or_else(|why| panic!("{name} {node}/{good} tick {}: {why}", r.tick));
            assert_eq!(
                e.next.to_bits(),
                m.next_price.to_bits(),
                "{name} {node}/{good} tick {}: {:?} against the engine's {:?}",
                r.tick,
                e.next,
                m.next_price
            );
            assert_eq!(e.recorded.map(f64::to_bits), Some(m.next_price.to_bits()));
            assert_eq!(e.equal, Some(true));
            assert_eq!(e.price.to_bits(), m.price.to_bits());
            checked += 1;
        }
    }
    assert_eq!(checked, reports.len() * w.markets().count(), "{name}");
    println!("{name}: the explainer equals next_price at {checked} (tick, market) pairs");
    store
}

#[test]
fn the_explainer_equals_next_price_on_every_gate_tick() {
    let store = every_tick("gate", GATE, GATE_TICKS);
    // The gate's rate is 5.2 a year, read as a log step: k = 5.2/52 = 0.1 per tick.
    let town = Key::new("town").unwrap();
    let bread = Key::new("bread").unwrap();
    let e = explain(&store, &town, &bread, 0).unwrap();
    assert_eq!(e.rate.as_str(), "rate.bread");
    assert_eq!(e.method, "log_step");
    assert_eq!(e.k, 5.2 / 52.0);
    assert_eq!(e.kx.to_bits(), (e.k * e.imbalance).to_bits());
    // A tick past the record has no inputs, and says so.
    assert!(explain(&store, &town, &bread, GATE_TICKS + 5)
        .unwrap_err()
        .contains("not recorded"));
}

#[test]
fn the_explainer_equals_next_price_on_every_appb_tick() {
    every_tick("appb", APPB, APPB_TICKS);
}

#[test]
fn the_waterfall_adds_up_to_the_log_price() {
    // ln(p_t/p₀) = Σ k·x + residual at the cursor, the bins' Σ k·x is the whole's, each bin's
    // level is ln(p/p₀) at its end, and the bins tile the span a year at a time. The gate holds
    // a one-sided market's price (`Hold`) and no event of it moves a price, so its residual is
    // what the holds did not step, −Σ k·x over the one-sided ticks, and rounding.
    let t = tape_of(GATE);
    let (store, reports) = record(&t, GATE_TICKS);
    let town = Key::new("town").unwrap();
    for good in ["bread", "fuel", "grain"] {
        let good = Key::new(good).unwrap();
        for cursor in [Some(1), Some(51), Some(800), None] {
            let w = waterfall(&store, &town, &good, cursor).unwrap();
            let tick = cursor.unwrap_or(GATE_TICKS - 1);
            assert_eq!(w.tick, tick);
            let p = |t: u64| {
                let r = &reports[t as usize];
                r.markets
                    .iter()
                    .find(|m| {
                        store.world().unwrap().key_of(m.node) == Some(&town)
                            && store.world().unwrap().key_of(m.good) == Some(&good)
                    })
                    .unwrap()
                    .price
            };
            assert_eq!(w.p0.to_bits(), p(0).to_bits());
            assert_eq!(w.p.to_bits(), p(tick).to_bits());
            assert_eq!(w.level.to_bits(), num::ln(w.p / w.p0).to_bits());
            assert_eq!(w.residual.to_bits(), (w.level - w.explained).to_bits());
            let sum: f64 = w.bins.iter().map(|b| b.kx).sum();
            assert!((sum - w.explained).abs() <= 1e-12, "{sum} {}", w.explained);
            let mut held = 0.0;
            let mut sides = 0;
            for t in 0..tick {
                let e = explain(&store, &town, &good, t).unwrap();
                if e.one_side {
                    held += e.kx;
                    sides += 1;
                }
            }
            assert_eq!(w.one_sided, sides);
            assert!(
                (w.residual + held).abs() < 1e-9,
                "{good} at {cursor:?}: residual {} against the holds' {}",
                w.residual,
                -held
            );
            assert_eq!(w.missing, 0);
            if let Some(last) = w.bins.last() {
                assert_eq!(last.to, tick);
                assert_eq!(last.level.to_bits(), w.level.to_bits());
            }
            let mut from = 0;
            for b in &w.bins {
                assert_eq!(b.from, from);
                from = b.to;
            }
            if tick >= 104 {
                assert!(w.bins.len() <= 42, "{} bins", w.bins.len());
            } else {
                assert_eq!(w.bins.len() as u64, tick);
            }
        }
    }
    // The events fired in the span are marked; none moves a gate price.
    let w = waterfall(&store, &town, &Key::new("bread").unwrap(), None).unwrap();
    assert!(w.events.iter().any(|e| e.key.as_str() == "mine.cut"));
    assert!(w.events.iter().all(|e| !e.moves));
}
