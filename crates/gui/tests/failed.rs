//! A failed run (docs/GUI.md §3.3, §8.1; E5, R2): through the whole seam, a run that cannot
//! take what a tape event burns pauses on the error breakpoint, and its toolbar and log show
//! `Poisoned`, its ledger line and its last good tick, as the cli's stderr does.

mod common;

use common::{build, theft, PATIENCE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Host;
use rustyecon_gui::model::{Intent, Model};
use rustyecon_gui::run::{Breakpoint, PauseReason, RunStatus};
use rustyecon_gui::vm;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Pump `host` into `m` until `done` holds of the model.
fn pump_until(host: &mut Host, m: &mut Model, done: impl Fn(&Model) -> bool) {
    let t0 = Instant::now();
    while !done(m) {
        host.pump(m);
        assert!(t0.elapsed() < PATIENCE, "the run did not answer in time");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn failed_run_shows_its_ledger_line() {
    // A shortfall variant of the gate, made by text substitution as the cli's
    // shortfall_stops_the_run makes one: on 1751-06-01, tick 73, an event burns 1e9 coin the
    // workers do not hold.
    let mut m = Model::default();
    let mut host = Host::new(build(), Arc::new(|| {}), None);
    host.act(&mut m, Intent::BreakOnError(true));
    host.act(
        &mut m,
        Intent::TapeRead {
            path: "theft.ron".to_string(),
            text: Ok(theft()),
        },
    );
    pump_until(&mut host, &mut m, |m| {
        m.focused().is_some_and(|r| r.store.run().is_some())
    });
    host.act(&mut m, Intent::Run { until: Some(2080) });
    pump_until(&mut host, &mut m, |m| {
        m.focused()
            .is_some_and(|r| r.store.paused().is_some_and(|p| p.0 > 0))
    });
    let run = m.focused().expect("the run");
    let store = &run.store;
    // It paused on the error breakpoint, poisoned in tick 73's events.
    assert_eq!(
        store.paused(),
        Some((73, PauseReason::Breakpoint(Breakpoint::OnError)))
    );
    assert_eq!(
        store.status(),
        RunStatus::Poisoned {
            tick: 73,
            phase: Phase::Events
        }
    );
    assert!(!store.can_run());
    assert_eq!(store.hashes().len(), 73, "ticks 0 to 72 are recorded");
    // The toolbar: Poisoned, the ledger line, the last good tick.
    let t = vm::toolbar::build(store, run.origin);
    assert_eq!(t.health.status, vm::toolbar::Status::Poisoned);
    let line = t.health.ledger_line.expect("the ledger line");
    assert!(line.starts_with("tick 73, phase 0 (events): "), "{line}");
    assert!(
        line.contains("shortfall at tick 73") && line.contains("a Event burn"),
        "{line}"
    );
    assert!(line.contains("asked pop#1 for 1e9 of good#1"), "{line}");
    assert_eq!(t.health.last_good_tick, Some(72));
    assert_eq!(
        t.health.paused.as_deref(),
        Some("breakpoint on error at tick 73")
    );
    // The log: the same line, and the last good tick in the cli's words.
    let log = vm::log::build(m.log());
    let errors: Vec<&str> = log
        .lines
        .iter()
        .filter(|l| l.error)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(
        errors,
        [
            format!("run error: {line}").as_str(),
            "the last good tick is 72 (state tick 73)"
        ]
    );
    // A poisoned run takes no more ticks: Space, a step and a run send nothing.
    let hashes = store.hashes().len();
    host.act(&mut m, Intent::RunPause);
    host.act(&mut m, Intent::Step(1));
    host.act(&mut m, Intent::Run { until: None });
    std::thread::sleep(Duration::from_millis(50));
    host.pump(&mut m);
    assert_eq!(m.focused().unwrap().store.hashes().len(), hashes);
}
