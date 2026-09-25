//! The frontend contract (docs/ENGINE.md §7 and §11, engine; E3, E4, E5): a frontend reads a
//! run through reports and accessors that cannot change it, can run it on a worker thread, and
//! meets a poisoned `Sim` after a failed step. Every comparison is exact.

mod common;

use common::scan::{shipped, sources, strip};
use common::*;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_core::state_hash;
use std::sync::mpsc;

#[test]
fn observation_matches_accessors() {
    // E4: every report agrees with the accessors read after its step, on every gate tick.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let markets: Vec<(NodeId, GoodId)> = w.markets().collect();
    for _ in 0..TICKS {
        let r = sim.step().unwrap();
        assert_eq!(r.tick + 1, sim.tick());
        assert_eq!(Some(r.date), w.clock.date_of(r.tick));
        assert_eq!(r.hash, sim.hash());
        assert_eq!(state_hash(&sim.checkpoint().unwrap().state), sim.hash());
        assert_eq!(sim.last_report(), Some(&r));
        let order: Vec<(NodeId, GoodId)> = r.markets.iter().map(|m| (m.node, m.good)).collect();
        assert_eq!(order, markets);
        for m in &r.markets {
            assert_eq!(sim.price(m.node, m.good), Some(m.next_price));
            assert_eq!(sim.ema(m.node, m.good), Some(m.ema));
            assert_eq!(sim.supply(m.node, m.good), Some(m.supply));
            assert_eq!(sim.demand(m.node, m.good), Some(m.demand));
            let cleared = r
                .settlements
                .iter()
                .filter(|l| (l.node, l.good, l.side) == (m.node, m.good, SideTag::Buy))
                .fold(0.0, |acc, l| acc + l.qty);
            assert_eq!(m.cleared, cleared);
        }
    }
}

/// Call every read-only accessor.
fn observe(sim: &Sim) -> u64 {
    let w = sim.world();
    let mut touched = sim.tick() ^ sim.hash();
    for (n, g) in w.markets() {
        for v in [
            sim.price(n, g),
            sim.ema(n, g),
            sim.supply(n, g),
            sim.demand(n, g),
        ] {
            touched ^= v.unwrap().to_bits();
        }
    }
    for p in w.registry.params() {
        touched ^= sim.param(p.id).unwrap().to_bits();
    }
    for a in &w.actors {
        let h = Holder::Actor(a.id);
        touched ^= sim.holding(h).unwrap().lot_count() as u64;
        touched ^= u64::from(sim.actor_state(a.id).is_some());
    }
    for g in &w.goods {
        touched ^= sim.holdings_of(g.id).count() as u64;
    }
    touched ^= sim.observe_holdings().0.len() as u64;
    touched ^= u64::from(sim.status() == Status::Ready);
    touched ^= sim.last_report().map_or(0, |r| r.tick);
    touched ^= sim.checkpoint().unwrap().prefix_id;
    touched
}

#[test]
fn observing_changes_no_hash() {
    // E4: reading is free. A run observed through every accessor after every step, checkpoints
    // included, has the hash stream of a run nobody watched.
    let t = tape();
    let quiet = hashes(&t, TICKS);
    let mut sim = sim_of(&t);
    let mut watched = Vec::new();
    let mut touched = 0;
    for _ in 0..TICKS {
        touched ^= observe(&sim);
        watched.push(sim.step().unwrap().hash);
        touched ^= observe(&sim);
    }
    assert_eq!(watched, quiet);
    assert_ne!(touched, 0);
}

#[test]
fn engine_types_are_send() {
    // E3, at compile time.
    fn ok<T: Send + Sync + 'static>() {}
    ok::<Sim>();
    ok::<Tape>();
    ok::<World>();
    ok::<Checkpoint>();
    ok::<TickReport>();
    ok::<HoldingTotals>();
    ok::<Trace>();
    ok::<RunError>();
    ok::<LoadError>();
    ok::<ResumeError>();
    ok::<ReplayError>();
}

#[test]
fn engine_runs_on_a_worker_thread() {
    // E3 in use: a Sim built here runs on a worker thread and sends every report, and its
    // holdings every 520 ticks, over a channel; the frontend keeps its own World for key
    // lookups. What arrives is exactly what a run on this thread sees.
    enum Msg {
        Tick(Box<TickReport>),
        Holdings(u64, HoldingTotals),
        Done(Result<u64, RunError>),
    }
    let t = tape();
    let sim = sim_of(&t);
    let world = sim.world().clone();
    let (tx, rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut sim = sim;
        let result = (|| {
            while sim.tick() < TICKS {
                let r = sim.step()?;
                tx.send(Msg::Tick(Box::new(r)))
                    .expect("the frontend listens");
                if sim.tick().is_multiple_of(520) {
                    tx.send(Msg::Holdings(sim.tick(), sim.observe_holdings()))
                        .expect("the frontend listens");
                }
            }
            Ok(sim.hash())
        })();
        tx.send(Msg::Done(result)).expect("the frontend listens");
    });
    let mut local = sim_of(&t);
    let mut ticks = 0;
    let mut snapshots = 0;
    let bread = good(&world, "bread");
    loop {
        match rx.recv().expect("the worker reports") {
            Msg::Tick(r) => {
                assert_eq!(*r, local.step().unwrap());
                assert!(r.markets.iter().any(|m| m.good == bread));
                ticks += 1;
            }
            Msg::Holdings(tick, h) => {
                assert_eq!(tick, local.tick());
                assert_eq!(h, local.observe_holdings());
                snapshots += 1;
            }
            Msg::Done(result) => {
                assert_eq!(result, Ok(local.hash()));
                break;
            }
        }
    }
    worker.join().expect("the worker finishes");
    assert_eq!((ticks, snapshots), (TICKS, 4));
}

#[test]
fn failed_step_poisons_the_sim() {
    // E5: a shortfall stops the tick with its ledger line; the Sim then refuses to step, run or
    // checkpoint, still answers its accessors, and keeps the last good report. A rebuilt Sim
    // runs.
    let text = edit(
        "    events: [\n",
        "    events: [\n        (key: \"theft\", at: \"1751-06-01\", basis: Assumed(\"test\"), \
         act: Burn(holder: \"workers\", good: \"coin\", amount: Qty(1e9))),\n",
    );
    let t = tape_of(&text);
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let at = tick_of(&w, "1751-06-01");
    sim.run_until(at, &mut |_| {})
        .expect("the run is good until the theft");
    let e = sim.step().unwrap_err();
    assert_eq!((e.tick, e.phase), (at, Phase::Events));
    let RunErrorKind::Core(CoreError::Shortfall(line)) = &e.kind else {
        panic!("expected a shortfall, got {e}");
    };
    assert_eq!(line.holder, Holder::Actor(actor(&w, "workers")));
    assert_eq!(line.requested, 1e9);
    assert_eq!(line.prov, Some(Provenance::Event));
    let shown = e.to_string();
    assert!(
        shown.contains("shortfall") && shown.contains("nothing moved"),
        "{shown}"
    );
    let poisoned = Status::Poisoned {
        tick: at,
        phase: Phase::Events,
    };
    assert_eq!(sim.status(), poisoned);
    for err in [
        sim.step().unwrap_err(),
        sim.run_until(TICKS, &mut |_| {}).unwrap_err(),
        sim.checkpoint().unwrap_err(),
    ] {
        assert_eq!(err.kind, RunErrorKind::Poisoned);
        assert_eq!((err.tick, err.phase), (at, Phase::Events));
    }
    assert!(sim.step_traced().is_err());
    assert_eq!(sim.last_report().map(|r| r.tick), Some(at - 1));
    assert_eq!(sim.tick(), at, "the tick never advanced");
    let town = node(&w, "town");
    assert!(sim.price(town, good(&w, "bread")).is_some());
    assert!(sim.holding(Holder::Actor(actor(&w, "workers"))).is_some());
    // Rebuilt from the tape, it runs to the same failure.
    let mut again = sim_of(&t);
    again.run_until(at, &mut |_| {}).unwrap();
    assert_eq!(again.step().unwrap_err(), e);
}

#[test]
fn no_public_api_hands_out_mut_state() {
    // E4: no public engine function returns `&mut` anything, so nothing reaches the SimState or
    // the World to change it; the Sim's fields are private. Checked on the source and, for the
    // accessors, by their types.
    let mut found = Vec::new();
    let mut public = 0;
    for (path, text) in sources("engine") {
        let code = strip(&text);
        let code = shipped(&code);
        let mut rest = code;
        while let Some(k) = rest.find("pub fn ") {
            let sig_end = rest[k..].find(['{', ';']).map_or(rest.len(), |e| k + e);
            let sig = &rest[k..sig_end];
            public += 1;
            if let Some(ret) = sig.split("->").nth(1) {
                if ret.contains("mut") {
                    found.push(format!("{path}: {}", sig.trim()));
                }
            }
            rest = &rest[sig_end..];
        }
        // The Sim's fields: no `pub` inside its body.
        if let Some(k) = code.find("pub struct Sim {") {
            let body = &code[k + "pub struct Sim {".len()..];
            let body = &body[..body.find('}').unwrap()];
            assert!(!body.contains("pub"), "{path}: a public field on Sim");
        }
    }
    assert!(
        public > 20,
        "the scan found the public functions ({public})"
    );
    assert!(
        found.is_empty(),
        "public functions returning &mut: {found:#?}"
    );
    // The accessors' types: shared references and owned values only.
    let _: fn(&Sim) -> &World = Sim::world;
    let _: fn(&Sim, Holder) -> Option<&Inventory> = Sim::holding;
    let _: fn(&Sim, ActorId) -> Option<&ActorState> = Sim::actor_state;
    let _: fn(&Sim) -> Option<&TickReport> = Sim::last_report;
    let _: fn(&Sim) -> Result<Checkpoint, RunError> = Sim::checkpoint;
    let _: fn(&Sim) -> HoldingTotals = Sim::observe_holdings;
    let _: fn(&Sim, NodeId, GoodId) -> Option<f64> = Sim::price;
}
