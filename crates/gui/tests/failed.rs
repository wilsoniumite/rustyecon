//! A failed run (docs/GUI.md §3.3, §8.1; E5, R2): through the whole seam, a run that cannot
//! take what a tape event burns pauses on the error breakpoint, and its toolbar and log show
//! `Poisoned`, its ledger line and its last good tick, as the cli's stderr does. The actor
//! inspector then reads the state tick 72 left, never the state the failure left.

mod common;

use common::{build, gift_then_theft, reference_hashes, tape_of, PATIENCE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Host;
use rustyecon_gui::model::{Intent, Model};
use rustyecon_gui::run::{Breakpoint, Entity, LedgerCheck, PauseReason, RunStatus};
use rustyecon_gui::vm;
use rustyecon_gui::vm::inspector::InspectorVm;
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
    // workers do not hold. A gift of 1,000 coin, dated the same day, fires first, so the
    // failed tick leaves the workers holding coin that no tick of the tape left them.
    let text = gift_then_theft();
    let mut m = Model::default();
    let mut host = Host::new(build(), Arc::new(|| {}), None);
    host.act(&mut m, Intent::BreakOnError(true));
    host.act(
        &mut m,
        Intent::TapeRead {
            path: "theft.ron".to_string(),
            text: Ok(text.clone()),
            lineage: None,
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
    let t = vm::toolbar::build(store, run.origin, LedgerCheck::NoParent);
    assert_eq!(t.health.status, vm::toolbar::Status::Poisoned);
    let line = t.health.ledger_line.clone().expect("the ledger line");
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
    // The ledger line is the engine's, as the cli prints it, with dense ids; beside it, the
    // same line by key (U7), and the tape events due in the tick's events phase.
    let failure = store.failure().expect("the failure");
    let RunErrorKind::Core(CoreError::Shortfall(short)) = &failure.error.kind else {
        panic!("a shortfall: {}", failure.error);
    };
    assert!(
        short.held > 1000.0,
        "the gift was applied before the burn failed"
    );
    assert_eq!(
        t.health.ledger_keys,
        [
            format!(
                "by key: workers was asked for 1e9 coin and held {:e}",
                short.held
            ),
            "tick 73's events, in firing order: a.gift (Mint 1000 coin to workers), theft \
             (Burn 1000000000 coin from workers)"
                .to_string(),
        ]
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
    // Select the workers: the model asks the run for a snapshot of the state tick 72 left,
    // state tick 73. That is the state the cli's run leaves at tick 73, whose hash is tick 72's
    // report hash, and not the state the failed tick left, which holds the gift.
    host.act(
        &mut m,
        Intent::Select(Some(Entity::Actor(Key::new("workers").unwrap()))),
    );
    pump_until(&mut host, &mut m, |m| {
        m.focused().is_some_and(|r| r.store.snapshot(73).is_some())
    });
    let store = &m.focused().unwrap().store;
    let snap = store.snapshot(73).unwrap();
    let reference = reference_hashes(&tape_of(&text), 73);
    assert_eq!(snap.hash, reference[72], "the snapshot is state tick 73");
    assert_eq!(
        snap.hash,
        store.hashes()[72],
        "the record's hash of state 73"
    );
    // The inspector shows, for state tick 73, lots that add up to the holdings the record
    // gives for that state, bit for bit, as the inventory sums them.
    let Some(InspectorVm::Actor(a)) = vm::inspector::build(store, m.selection().unwrap(), None)
    else {
        panic!("the workers' inspector");
    };
    assert_eq!((a.state_tick, a.snapshot), (73, true));
    let mut checked = 0;
    for h in &a.holdings {
        let lots = h.lots.as_ref().expect("the lots of each holding");
        let sum = lots.iter().fold(0.0, |acc, l| acc + l.qty);
        assert_eq!(
            Some(sum.to_bits()),
            h.held.value.map(f64::to_bits),
            "{}: lots {sum} beside a holding of {:?}",
            h.good,
            h.held.value
        );
        checked += 1;
    }
    assert!(checked > 0, "the workers hold something");
    // A poisoned run takes no more ticks: Space, a step and a run send nothing.
    let hashes = store.hashes().len();
    host.act(&mut m, Intent::RunPause);
    host.act(&mut m, Intent::Step(1));
    host.act(&mut m, Intent::Run { until: None });
    std::thread::sleep(Duration::from_millis(50));
    host.pump(&mut m);
    assert_eq!(m.focused().unwrap().store.hashes().len(), hashes);
}
