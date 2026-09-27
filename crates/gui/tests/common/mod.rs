//! Helpers for the GUI's tests: the two worlds, text edits of a tape, a Runner whose sink
//! collects, reference runs through the engine alone, and waits on a driver.

// Each test file is its own crate and uses part of this module.
#![allow(dead_code)]

pub mod paint;
pub mod scan;
pub mod sync;

use certify::Build;
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::{Driver, Host, ThreadDriver};
use rustyecon_gui::model::Model;
use rustyecon_gui::run::{Obs, PauseReason, Runner};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The date the editor's tests stamp their experiments with.
pub fn today() -> Date {
    Date::parse("2026-09-27").expect("a date")
}

/// A key.
pub fn key(s: &str) -> Key {
    Key::new(s).expect("a key")
}

/// The tick a date falls in, on the gate's calendar.
pub fn gate_tick(date: &str) -> u64 {
    let clock = tape_of(GATE);
    let clock = Clock {
        start: clock.header.start,
        ticks_per_year: clock.header.ticks_per_year,
    };
    clock
        .tick_of(Date::parse(date).expect("a date"))
        .expect("a tick")
}

/// Pump `host` into `m` until `done` holds of the model. Fails after [`PATIENCE`].
pub fn pump_until(host: &mut Host, m: &mut Model, what: &str, done: impl Fn(&Model) -> bool) {
    let t0 = Instant::now();
    while !done(m) {
        host.pump(m);
        assert!(t0.elapsed() < PATIENCE, "the run did not get there: {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// The gate world (docs/ENGINE.md §10) and its length, 40 years of 52 ticks.
pub const GATE: &str = include_str!("../../../../tapes/gate.ron");
pub const GATE_TICKS: u64 = 2080;

/// The Appendix B world (docs/probe/RULES.md) and the cli gate's run of it.
pub const APPB: &str = include_str!("../../../../tapes/appb.ron");
pub const APPB_TICKS: u64 = 20_000;

/// How long a test waits on a worker before it fails rather than hangs.
pub const PATIENCE: Duration = Duration::from_secs(300);

/// A tape text, parsed.
pub fn tape_of(text: &str) -> Tape {
    Tape::from_ron(text).expect("the tape parses")
}

/// Replace exactly one occurrence of `from` in `text`.
pub fn edit(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "{from:?} must occur once");
    text.replacen(from, to, 1)
}

/// The gate with a theft: a burn of 1e9 coin from the workers on 1751-06-01, tick 73, which
/// they do not hold. The cli's `shortfall_stops_the_run` makes the same variant.
pub fn theft() -> String {
    edit(
        GATE,
        "    events: [\n",
        "    events: [\n        (key: \"theft\", at: \"1751-06-01\", basis: Assumed(\"test\"), \
         act: Burn(holder: \"workers\", good: \"coin\", amount: Qty(1e9))),\n",
    )
}

/// The theft, with a gift of 1,000 coin to the workers on the same date, which fires first:
/// tick 73's events apply the gift and then fail on the burn, so the state the failure leaves
/// differs from every state a tick of the tape left.
pub fn gift_then_theft() -> String {
    edit(
        GATE,
        "    events: [\n",
        "    events: [\n        (key: \"a.gift\", at: \"1751-06-01\", basis: Assumed(\"test\"), \
         act: Mint(holder: \"workers\", good: \"coin\", qty: 1000.0)),\n        \
         (key: \"theft\", at: \"1751-06-01\", basis: Assumed(\"test\"), \
         act: Burn(holder: \"workers\", good: \"coin\", amount: Qty(1e9))),\n",
    )
}

/// A build for tests: this binary's stamp.
pub fn build() -> Build {
    rustyecon_gui::build()
}

/// A Runner whose sink collects into the returned list.
pub fn collecting() -> (Runner, Arc<Mutex<Vec<Obs>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let runner = Runner::new(
        build(),
        Box::new(move |o| sink.lock().unwrap().push(o)),
        Box::new(|| {}),
    );
    (runner, seen)
}

/// Take what a collecting sink holds.
pub fn drain(seen: &Arc<Mutex<Vec<Obs>>>) -> Vec<Obs> {
    std::mem::take(&mut *seen.lock().unwrap())
}

/// `TickReport::hash` of every tick of a run of `t` from genesis until the state's tick is
/// `until`, through the engine alone: `Sim::new` and `run_until`.
pub fn reference_hashes(t: &Tape, until: u64) -> Vec<u64> {
    let mut sim = Sim::new(t).expect("the tape loads");
    let mut out = Vec::new();
    sim.run_until(until, &mut |r| out.push(r.hash))
        .expect("the run succeeds");
    out
}

/// A `Sim` of `t` at state tick `tick`, from genesis.
pub fn sim_at(t: &Tape, tick: u64) -> Sim {
    let mut sim = Sim::new(t).expect("the tape loads");
    sim.run_until(tick, &mut |_| {}).expect("the run succeeds");
    sim
}

/// A driver with an empty Runner and no wake.
pub fn driver() -> ThreadDriver {
    ThreadDriver::spawn(build(), Arc::new(|| {}))
}

/// Poll `d` into `log` until an observation satisfies `done`, and return it. Fails after
/// [`PATIENCE`].
pub fn wait_for(d: &mut ThreadDriver, log: &mut Vec<Obs>, done: impl Fn(&Obs) -> bool) -> Obs {
    let t0 = Instant::now();
    let mut from = log.len();
    loop {
        d.poll(log);
        if let Some(o) = log[from..].iter().find(|o| done(o)) {
            return o.clone();
        }
        from = log.len();
        assert!(t0.elapsed() < PATIENCE, "the driver did not answer in time");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Poll `d` until it reports a pause, and return the tick and the reason.
pub fn wait_paused(d: &mut ThreadDriver, log: &mut Vec<Obs>) -> (u64, PauseReason) {
    match wait_for(d, log, |o| matches!(o, Obs::Paused { .. })) {
        Obs::Paused { tick, why } => (tick, why),
        _ => unreachable!(),
    }
}
