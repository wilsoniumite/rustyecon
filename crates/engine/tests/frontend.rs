//! The frontend contract (docs/ENGINE.md §7 and §11, engine; E3, E4, E5): a frontend reads a
//! run through reports and accessors that cannot change it, can run it on a worker thread, and
//! meets a poisoned `Sim` after a failed step. Every comparison is exact.

mod common;

use common::scan::{
    api_violations, impl_violations, pub_fields, shipped, sources, strip, writer_reexports,
    CORE_READ_ONLY, ENGINE_PATH,
};
use common::*;
use rustyecon_core::state_hash;
use rustyecon_engine::prelude::*;
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
        assert_eq!(state_hash(sim.checkpoint().unwrap().state()), sim.hash());
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
    touched ^= sim.checkpoint().unwrap().prefix_id();
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
    // E1, E4: nothing a frontend can reach hands it a way to change a run. Only `step`,
    // `step_traced` and `run_until` take `&mut self` (so there is no setter on Sim); no public
    // signature in the engine, markets or agents names `&mut` to a Sim, Checkpoint, SimState,
    // World or Inventory, a free function's or a closure's parameters included; no return type
    // holds `&mut` anything; no impl of Sim, core's Checkpoint or core's SimState, inherent or
    // of a trait, hands out `&mut` (`DerefMut`, `AsMut`, `BorrowMut`, `IndexMut`, or a method
    // with `&mut self`); and none of the three has a public field. Checked on the source, the
    // scanner on regression fixtures first, and, for the accessors, by their types. Round 3 of
    // the review (O11) found five writers the P0.8 scan missed; each is a fixture here.
    const ALLOWED: [&str; 3] = ["step", "step_traced", "run_until"];
    const NONE: [&str; 0] = [];
    let impls: [(&str, &[&str]); 3] = [
        ("Sim", &ALLOWED),
        ("Checkpoint", &NONE),
        ("SimState", &NONE),
    ];
    let names = |flagged: &[String]| -> Vec<String> {
        flagged
            .iter()
            .map(|f| {
                let f = f.trim_start_matches("pub ").trim_start_matches("fn ");
                f.split(['(', '<', ':']).next().unwrap().trim().to_string()
            })
            .collect()
    };
    // Public functions.
    let fixtures = r"
        impl Sim {
            pub fn set_param(&mut self, p: ParamId, value: f64) -> Result<(), RunError> { todo!() }
            pub fn with_state<R>(&mut self, f: impl FnOnce(&mut SimState<Agents>) -> R) -> R { todo!() }
            pub fn state_mut(&self) -> &mut SimState<Agents> { todo!() }
            pub fn edit(&self, f: &mut dyn FnMut(&'a mut rustyecon_core::World)) { todo!() }
            pub fn step(&mut self) -> Result<TickReport, RunError> { todo!() }
            pub fn run_until(&mut self, until: u64, on_tick: &mut dyn FnMut(&TickReport)) { todo!() }
            pub fn holding(&self, h: Holder) -> Option<&Inventory> { todo!() }
            pub(crate) fn advance(&mut self) -> Result<TickReport, RunError> { todo!() }
        }
        pub fn set_param_of(sim: &mut Sim, p: ParamId, value: f64) { todo!() }
        pub fn all(sims: &mut Vec<Sim>) { todo!() }
        pub const fn peek(cp: &mut Option<Checkpoint>) { todo!() }
        pub fn read(sim: &Sim, cp: &Checkpoint, f: fn(&Sim) -> u64) -> u64 { todo!() }
    ";
    let mut n = 0;
    let flagged = api_violations(&shipped(&strip(fixtures)), &ALLOWED, &mut n);
    assert_eq!(n, 11);
    assert_eq!(
        names(&flagged),
        [
            "set_param",
            "with_state",
            "state_mut",
            "edit",
            "set_param_of",
            "all",
            "peek"
        ],
        "{flagged:#?}"
    );
    // Impls: the trait impl and the inherent methods of the review's mutants 1, 4 and 5, and
    // what may stay.
    let fixtures = r"
        impl std::ops::DerefMut for Sim {
            fn deref_mut(&mut self) -> &mut SimState<Agents> { todo!() }
        }
        impl<E: Ext> Checkpoint<E> {
            pub fn state(&self) -> &SimState<E> { todo!() }
            pub fn state_mut(&mut self) -> &mut SimState<E> { todo!() }
        }
        impl<E: Ext> SimState<E> {
            pub(crate) fn genesis(params: Vec<f64>) -> SimState<E> { todo!() }
            pub fn holdings(&self) -> &BTreeMap<Holder, Inventory> { todo!() }
            pub fn holdings_mut(&mut self) -> &mut BTreeMap<Holder, Inventory> { todo!() }
        }
        impl AsMut<SimState<Agents>> for Sim { fn as_mut(&mut self) -> &mut SimState<Agents> { todo!() } }
        impl Extend<StateDelta<Agents>> for Sim { fn extend<I>(&mut self, iter: I) { todo!() } }
        impl std::ops::Deref for Sim {
            type Target = World;
            fn deref(&self) -> &World { todo!() }
        }
        impl fmt::Display for Checkpoint { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() } }
        impl Sim {
            pub fn step(&mut self) -> Result<TickReport, RunError> { todo!() }
            fn advance(&mut self) -> Result<TickReport, RunError> { todo!() }
        }
        impl Inventory { pub fn put(&mut self, g: GoodId, lots: Vec<Lot>) {} }
        pub fn with(f: impl FnOnce(&Sim)) {}
    ";
    let mut seen = 0;
    let flagged = impl_violations(&shipped(&strip(fixtures)), &impls, &mut seen);
    assert_eq!(
        seen, 8,
        "the impls of Sim, Checkpoint and SimState: {flagged:#?}"
    );
    let expected = [
        "DerefMut for Sim: a trait that hands out &mut Sim",
        "fn deref_mut(&mut self)",
        "fn state_mut(&mut self)",
        "fn holdings_mut(&mut self)",
        "for Sim: a trait that hands out &mut Sim",
        "fn as_mut(&mut self)",
        "fn extend<I>(&mut self",
    ];
    assert_eq!(flagged.len(), expected.len(), "{flagged:#?}");
    for (f, want) in flagged.iter().zip(expected) {
        assert!(f.contains(want), "{want:?} in {f:?}");
    }
    // Fields.
    let fixtures = r"
        pub struct Sim { world: World, pub state: SimState<Agents> }
        pub struct Checkpoint<E: Ext>(pub SimState<E>, u64);
        pub struct SimState<E: Ext> { pub(crate) tick: u64, pub(crate) params: Vec<f64> }
        pub struct Report { pub tick: u64 }
    ";
    let mut seen = 0;
    let flagged = pub_fields(
        &shipped(&strip(fixtures)),
        &["Sim", "Checkpoint", "SimState"],
        &mut seen,
    );
    assert_eq!(seen, 3);
    assert_eq!(names(&flagged), ["Sim", "Checkpoint"], "{flagged:#?}");

    // The sources a frontend can reach: the engine's, markets' and agents' public functions,
    // every impl of the three types wherever it lives (core's Checkpoint and SimState
    // included), and their fields.
    let mut found = Vec::new();
    let (mut public, mut impls_seen, mut structs) = (0, 0, 0);
    for krate in ENGINE_PATH {
        for (path, text) in sources(krate) {
            let code = shipped(&strip(&text));
            if krate != "core" {
                let allowed: &[&str] = if krate == "engine" { &ALLOWED } else { &NONE };
                for f in api_violations(&code, allowed, &mut public) {
                    found.push(format!("{path}: {f}"));
                }
            }
            for f in impl_violations(&code, &impls, &mut impls_seen) {
                found.push(format!("{path}: {f}"));
            }
            for f in pub_fields(&code, &["Sim", "Checkpoint", "SimState"], &mut structs) {
                found.push(format!("{path}: {f}"));
            }
        }
    }
    assert!(
        public > 40,
        "the scan found the public functions ({public})"
    );
    assert_eq!(
        impls_seen, 3,
        "the inherent impls of Sim, Checkpoint and SimState"
    );
    assert_eq!(structs, 3, "the structs Sim, Checkpoint and SimState");
    assert!(
        found.is_empty(),
        "what could hand a frontend a writer: {found:#?}"
    );
    // The accessors' types: shared references and owned values only.
    let _: fn(&Sim) -> &World = Sim::world;
    let _: fn(&Sim, Holder) -> Option<&Inventory> = Sim::holding;
    let _: fn(&Sim, ActorId) -> Option<&ActorState> = Sim::actor_state;
    let _: fn(&Sim) -> Option<&TickReport> = Sim::last_report;
    let _: fn(&Sim) -> Result<Checkpoint, RunError> = Sim::checkpoint;
    let _: fn(&Sim) -> HoldingTotals = Sim::observe_holdings;
    let _: fn(&Sim, NodeId, GoodId) -> Option<f64> = Sim::price;
    // A checkpoint is read-only too: its state comes out as a shared reference (the fields
    // are private; core's doc test shows that `&mut cp.state` does not compile, and the
    // engine's that `state_mut`, `holdings_mut` and `&mut **sim` do not).
    let _: fn(
        &Checkpoint,
    ) -> &rustyecon_core::SimState<rustyecon_engine::rustyecon_agents::Agents> = Checkpoint::state;
}

#[test]
fn no_reexport_hands_out_core_writer() {
    // E1: a frontend that depends on the engine alone cannot reach core's writer. The engine
    // re-exports core's read-only types in its prelude and never core itself, and neither
    // markets nor agents, which the engine does re-export, re-exports core. A re-export of
    // anything from core but the read-only types is flagged (an allow-list, which the
    // prelude's list must equal), and so is core whole: bare, renamed, by glob, by `self` in a
    // group (the review's mutant 3, O11), through a private alias, or as `pub extern crate`.
    // The doc tests of the engine's lib.rs pin the same from outside:
    // `rustyecon_engine::rustyecon_core::apply` does not compile. Checked on the source, the
    // scanner on regression fixtures first.
    let fixtures = r"
        pub use rustyecon_core;
        pub use rustyecon_core as core;
        pub use rustyecon_core::*;
        pub use rustyecon_core::{Date, apply};
        pub use rustyecon_core::{
            Key,
            Ledger,
        };
        pub use rustyecon_core::{self as internals};
        pub use rustyecon_core::{Date, self};
        pub use ::rustyecon_core::resolve;
        pub use rustyecon_core::ledger::RunLedger;
        pub use rustyecon_core::inventory::Inventory;
        use rustyecon_core as c;
        pub use c::Resolver;
        pub extern crate rustyecon_core;
        pub use rustyecon_core::{Date, Key, SimState};
        pub use rustyecon_core::Inventory as Stock;
        pub(crate) use rustyecon_core::apply;
        use rustyecon_core::{apply, Ledger};
        pub use rustyecon_agents;
        pub use crate::prelude::*;
    ";
    let mut seen = 0;
    let flagged = writer_reexports(&shipped(&strip(fixtures)), &mut seen);
    assert_eq!(seen, 16);
    assert_eq!(
        flagged,
        [
            "pub use rustyecon_core",
            "pub use rustyecon_core as core",
            "pub use rustyecon_core::*",
            "pub use rustyecon_core::{Date, apply}",
            "pub use rustyecon_core::{ Key, Ledger, }",
            "pub use rustyecon_core::{self as internals}",
            "pub use rustyecon_core::{Date, self}",
            "pub use ::rustyecon_core::resolve",
            "pub use rustyecon_core::ledger::RunLedger",
            "pub use rustyecon_core::inventory::Inventory",
            "pub use c::Resolver",
            "pub extern crate rustyecon_core",
        ]
    );
    // The allow-list is exactly what the prelude re-exports from core.
    let prelude = sources("engine")
        .into_iter()
        .find(|(p, _)| p.replace('\\', "/").ends_with("engine/src/prelude.rs"))
        .expect("the prelude");
    let code = shipped(&strip(&prelude.1));
    let group = &code[code
        .find("pub use rustyecon_core::{")
        .expect("core's group")..];
    let group = &group[group.find('{').unwrap() + 1..group.find('}').unwrap()];
    let mut listed: Vec<&str> = group
        .split(',')
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .collect();
    listed.sort_unstable();
    assert_eq!(listed, CORE_READ_ONLY);
    let mut found = Vec::new();
    let mut seen = 0;
    for krate in ["engine", "markets", "agents"] {
        for (path, text) in sources(krate) {
            for stmt in writer_reexports(&shipped(&strip(&text)), &mut seen) {
                found.push(format!("{path}: {stmt}"));
            }
        }
    }
    assert!(seen > 10, "the scan found the re-exports ({seen})");
    assert!(found.is_empty(), "re-exports of core's writer: {found:#?}");
}
