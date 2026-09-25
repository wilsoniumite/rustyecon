//! Shared helpers for the engine's gate tests: the gate tape and its variants, ids by key, and
//! runs that collect reports and hashes.

// Each test file is its own crate and uses part of this module.
#![allow(dead_code)]

use rustyecon_engine::prelude::*;

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
