//! Shared helpers for the engine's gate tests: the gate tape and its variants, ids by key, and
//! runs that collect reports and hashes.

// Each test file is its own crate and uses part of this module.
#![allow(dead_code)]

use rustyecon_core::{apply, resolve, state_hash, Ledger};
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::{Agents, Cast};
use rustyecon_engine::rustyecon_markets::{admit, clear, Line};

pub mod scan;

/// The gate world (docs/ENGINE.md §10).
pub const GATE: &str = include_str!("../../../../tapes/gate.ron");

/// The gate run's length: 40 years of 52 ticks.
pub const TICKS: u64 = 2080;

/// A tape text, parsed.
pub fn tape_of(text: &str) -> Tape {
    Tape::from_ron(text).expect("the tape parses")
}

/// The gate tape.
pub fn tape() -> Tape {
    tape_of(GATE)
}

/// Replace exactly one occurrence of `from` in `text`.
pub fn edit_text(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "{from:?} must occur once");
    text.replacen(from, to, 1)
}

/// Replace exactly one occurrence of `from` in the gate tape.
pub fn edit(from: &str, to: &str) -> String {
    edit_text(GATE, from, to)
}

/// A `Sim` of a tape.
pub fn sim_of(t: &Tape) -> Sim {
    Sim::new(t).expect("the tape loads")
}

/// Step `sim` until its tick is `until`, returning every report.
pub fn reports(sim: &mut Sim, until: u64) -> Vec<TickReport> {
    let mut out = Vec::new();
    sim.run_until(until, &mut |r| out.push(r.clone()))
        .expect("the run succeeds");
    out
}

/// The hash after each tick of a run of `t` from genesis to `until`.
pub fn hashes(t: &Tape, until: u64) -> Vec<u64> {
    let mut sim = sim_of(t);
    let mut out = Vec::new();
    sim.run_until(until, &mut |r| out.push(r.hash))
        .expect("the run succeeds");
    out
}

pub fn actor(w: &World, key: &str) -> ActorId {
    w.id_of::<ActorId>(key).expect("a gate actor")
}

pub fn good(w: &World, key: &str) -> GoodId {
    w.id_of::<GoodId>(key).expect("a gate good")
}

pub fn node(w: &World, key: &str) -> NodeId {
    w.id_of::<NodeId>(key).expect("a gate node")
}

pub fn class(w: &World, key: &str) -> ClassId {
    w.id_of::<ClassId>(key).expect("a gate class")
}

pub fn param(w: &World, key: &str) -> ParamId {
    w.id_of::<ParamId>(key).expect("a gate param")
}

/// The tick a date falls in.
pub fn tick_of(w: &World, date: &str) -> u64 {
    w.clock
        .tick_of(Date::parse(date).expect("a date"))
        .expect("a date after the start")
}

/// The net declared quantity of (good, provenance) in a report's audit, 0 if none.
pub fn audit_line(r: &TickReport, g: GoodId, p: Provenance) -> f64 {
    r.audit
        .lines
        .iter()
        .find(|(good, prov, _)| *good == g && *prov == p)
        .map_or(0.0, |(_, _, q)| *q)
}

/// One tick of a run, rebuilt beside it from each phase's start state (E6): what the hooks
/// return on the state their phase began with, and the lines admission makes of those orders.
pub struct Rebuilt {
    /// The tick's report.
    pub report: TickReport,
    /// Every delta the tick applied.
    pub trace: Trace,
    /// The `decide` deltas of every actor on the state after phase 0, in `ActorId` order.
    pub decided: Vec<StateDelta<Agents>>,
    /// The admitted lines of the `decide` orders on that state, in canonical order.
    pub lines: Vec<Line>,
    /// `clear`'s volumes for those lines.
    pub volumes: Vec<StateDelta<Agents>>,
    /// The `produce` deltas of every actor on the state after phase 3, in `ActorId` order.
    pub produced: Vec<StateDelta<Agents>>,
}

impl Rebuilt {
    /// The trace's deltas of one phase, in order.
    pub fn applied(&self, phase: Phase) -> Vec<StateDelta<Agents>> {
        self.trace
            .0
            .iter()
            .filter(|e| e.phase == phase)
            .map(|e| e.delta.clone())
            .collect()
    }

    /// The admitted line of an actor's order in a market, on one side.
    pub fn line(&self, a: ActorId, n: NodeId, g: GoodId, side: SideTag) -> Option<&Line> {
        self.lines.iter().find(|l| {
            let o = &l.order;
            (o.actor, o.node, o.good, o.side.tag()) == (a, n, g, side)
        })
    }
}

/// Run `t` until its tick is `until`, rebuilding each tick beside the run: a shadow state
/// applies the trace phase by phase, and at the start of phases 1, 2 and 4 the hooks and
/// admission run on it through the crates' public functions. The shadow's hash must equal the
/// run's after every tick.
pub fn rebuild(t: &Tape, until: u64, mut each: impl FnMut(&World, &Rebuilt)) {
    let mut sim = sim_of(t);
    let w = sim.world().clone();
    let cast = Cast::new(&w).expect("the cast builds");
    let (_, mut shadow) = resolve(t).expect("the tape resolves");
    let phases = [
        Phase::Events,
        Phase::Decisions,
        Phase::Clearing,
        Phase::Settlement,
        Phase::Production,
        Phase::Upkeep,
        Phase::Prices,
    ];
    while sim.tick() < until {
        let (report, trace) = sim.step_traced().expect("the run succeeds");
        let mut l = Ledger::open(&shadow, &w).expect("the ledger opens");
        let (mut decided, mut orders, mut produced) = (Vec::new(), Vec::new(), Vec::new());
        let (mut lines, mut volumes) = (Vec::new(), Vec::new());
        for phase in phases {
            match phase {
                Phase::Decisions => {
                    for a in cast.actors() {
                        let d = cast.decide(a, &shadow, &w).expect("decide runs");
                        decided.extend(d.deltas);
                        orders.extend(d.orders);
                    }
                }
                Phase::Clearing => {
                    lines = admit(orders.clone(), &shadow, &w).expect("the orders are admitted");
                    volumes = clear(&lines, &w).expect("the lines clear").0;
                }
                Phase::Production => {
                    for a in cast.actors() {
                        produced.extend(cast.produce(a, &shadow, &w).expect("produce runs"));
                    }
                }
                _ => {}
            }
            for e in trace.0.iter().filter(|e| e.phase == phase) {
                apply(
                    &mut shadow,
                    &w,
                    phase,
                    std::slice::from_ref(&e.delta),
                    &mut l,
                )
                .expect("the traced delta applies");
            }
        }
        assert_eq!(state_hash(&shadow), report.hash, "tick {}", report.tick);
        each(
            &w,
            &Rebuilt {
                report,
                trace,
                decided,
                lines,
                volumes,
                produced,
            },
        );
    }
}
