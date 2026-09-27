//! U4: a GUI run hashes equal to the cli's run of the same tape, on the same platform and
//! build (docs/GUI.md §8.1). The GUI's path is the whole seam: commands through a
//! `ThreadDriver` to its worker's Runner, observations back over the channel into a Store.
//!
//! When `RUSTYECON_GUI_HASHES` names a directory, the tests write `gate.hashes`, `appb.hashes`
//! and `demo-gb.hashes` there, one `{t} 0x{hash:016x}` line per tick with
//! `t = report.tick + 1` (the cli's convention, P0.5 amendment 8), and `scripts/gui.sh` compares
//! them with the body of `rustyecon run <tape> --until <T> --hashes`.

mod common;

use certify::Manifest;
use common::{
    driver, reference_hashes, sim_at, tape_of, wait_for, wait_paused, APPB, APPB_TICKS, GATE,
    GATE_TICKS,
};
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Driver;
use rustyecon_gui::run::{Breakpoint, Catalogue, Cmd, Obs, PauseReason, Store};
use std::time::Duration;

/// Run `t` through a `ThreadDriver` by a fixed script of pauses, speed caps, steps of 1 and 7,
/// run-untils, snapshots and the breakpoint on error, to `until`; every observation, in order.
fn scripted(t: &Tape, until: u64) -> Vec<Obs> {
    let mut d = driver();
    let mut log = Vec::new();
    let run = |until: Option<u64>, max_tps: Option<u32>| Cmd::Run { until, max_tps };
    d.send(Cmd::Breakpoints(vec![Breakpoint::OnError]));
    d.send(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
    wait_for(&mut d, &mut log, |o| matches!(o, Obs::Loaded { .. }));
    for (cmd, want) in [
        (Cmd::Step(1), (1, PauseReason::Stepped)),
        (Cmd::Step(7), (8, PauseReason::Stepped)),
        (Cmd::Step(1), (9, PauseReason::Stepped)),
        (run(Some(100), None), (100, PauseReason::Reached(100))),
    ] {
        d.send(cmd);
        assert_eq!(wait_paused(&mut d, &mut log), want);
    }
    // A snapshot ahead, reached under a speed cap of 4,000 ticks a second.
    d.send(Cmd::Snapshot(300));
    d.send(run(Some(400), Some(4_000)));
    assert_eq!(
        wait_paused(&mut d, &mut log),
        (400, PauseReason::Reached(400))
    );
    // A snapshot behind, from the ring.
    d.send(Cmd::Snapshot(250));
    wait_for(
        &mut d,
        &mut log,
        |o| matches!(o, Obs::Snapshot(s) if s.tick == 250),
    );
    // Run on at a cap, and pause wherever it has got to.
    d.send(run(None, Some(2_000)));
    std::thread::sleep(Duration::from_millis(30));
    d.send(Cmd::Pause);
    let (at, why) = wait_paused(&mut d, &mut log);
    assert_eq!(why, PauseReason::Asked);
    assert!(at >= 400, "{at}");
    for k in 1..=3 {
        d.send(Cmd::Step(7));
        assert_eq!(
            wait_paused(&mut d, &mut log),
            (at + 7 * k, PauseReason::Stepped)
        );
    }
    // A run-until behind the state pauses at once; then to the end, uncapped.
    d.send(run(Some(at), None));
    assert_eq!(
        wait_paused(&mut d, &mut log),
        (at + 21, PauseReason::Reached(at))
    );
    d.send(run(Some(until), None));
    assert_eq!(
        wait_paused(&mut d, &mut log),
        (until, PauseReason::Reached(until))
    );
    drop(d);
    log
}

/// Check one tape's GUI path against the engine's own run, and write its hash lines.
fn check(name: &str, text: &str, until: u64) {
    let t = tape_of(text);
    let log = scripted(&t, until);
    // The observations, as the Runner sent them: each tick once, in order.
    let mut ticks = Vec::new();
    for o in &log {
        if let Obs::Batch(b) = o {
            ticks.extend(b.rows.iter().map(|r| (r.tick, r.hash)));
        }
    }
    assert_eq!(ticks.len() as u64, until, "{name}: one row a tick");
    assert!(ticks.iter().enumerate().all(|(i, r)| r.0 == i as u64));
    // The store's record of them.
    let mut store = Store::default();
    for o in log.iter().cloned() {
        store.ingest(o).expect("every observation ingests");
    }
    assert_eq!(store.tick(), until);
    let reference = reference_hashes(&t, until);
    let got: Vec<u64> = ticks.iter().map(|r| r.1).collect();
    assert_eq!(got, reference, "{name}: the GUI's hashes are the engine's");
    assert_eq!(store.hashes(), reference.as_slice());
    assert_eq!(store.hash(), reference[reference.len() - 1]);
    // No failure, and the breakpoint never fired.
    assert!(store.failure().is_none());
    assert!(!log.iter().any(|o| matches!(
        o,
        Obs::Paused {
            why: PauseReason::Breakpoint(_),
            ..
        }
    )));
    // The snapshots equal the state at their ticks.
    for tick in [300, 250] {
        let snap = store.snapshot(tick).expect("the snapshot arrived");
        let sim = sim_at(&t, tick);
        assert_eq!(snap.hash, sim.hash(), "{name}: snapshot {tick}");
        assert_eq!(
            snap.holdings,
            sim.observe_holdings(),
            "{name}: snapshot {tick}"
        );
    }
    // The ring: every checkpoint's state is the run's at its tick.
    assert!(!store.ring().is_empty());
    for cp in store.ring() {
        let state = cp.checkpoint().expect("it decodes");
        let want = match cp.tick() {
            0 => Sim::new(&t).unwrap().hash(),
            k => reference[k as usize - 1],
        };
        assert_eq!(Sim::resume(&t, &state).unwrap().hash(), want, "{name}");
    }
    if let Some(dir) = std::env::var_os("RUSTYECON_GUI_HASHES") {
        let mut body = String::new();
        for (tick, hash) in &ticks {
            body.push_str(&Manifest::line(tick + 1, *hash));
        }
        let path = std::path::Path::new(&dir).join(format!("{name}.hashes"));
        std::fs::write(&path, body).expect("the hash file is written");
    }
}

#[test]
fn gui_equals_cli() {
    check("gate", GATE, GATE_TICKS);
    check("appb", APPB, APPB_TICKS);
}

/// The demo world's run in the gate: through the first tick of 1901, when its last steps fire
/// (worldgen's `demo_runs_to_1901`).
const DEMO_TICKS: u64 = 7_852;

#[test]
fn gui_equals_cli_demo_gb() {
    // U4 on the demo tape (D.4, 2026-09-27): tapes/demo-gb.ron through a ThreadDriver with the
    // lean catalogue the model sends a world of 93 nodes, by steps of 1 and 7 and a run-until,
    // hashes as the engine does at every tick; `scripts/gui.sh` diffs the written
    // `demo-gb.hashes` against `rustyecon run tapes/demo-gb.ron --until 7852 --hashes`.
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tapes/demo-gb.ron"
    ))
    .expect("tapes/demo-gb.ron");
    let t = tape_of(&text);
    assert_eq!(Catalogue::for_nodes(t.nodes.len()), Catalogue::Lean);
    let mut d = driver();
    let mut log = Vec::new();
    d.send(Cmd::Catalogue(Catalogue::Lean));
    d.send(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
    wait_for(&mut d, &mut log, |o| matches!(o, Obs::Loaded { .. }));
    d.send(Cmd::Step(1));
    assert_eq!(wait_paused(&mut d, &mut log), (1, PauseReason::Stepped));
    d.send(Cmd::Step(7));
    assert_eq!(wait_paused(&mut d, &mut log), (8, PauseReason::Stepped));
    d.send(Cmd::Run {
        until: Some(DEMO_TICKS),
        max_tps: None,
    });
    assert_eq!(
        wait_paused(&mut d, &mut log),
        (DEMO_TICKS, PauseReason::Reached(DEMO_TICKS))
    );
    drop(d);
    let mut ticks = Vec::new();
    for o in &log {
        if let Obs::Batch(b) = o {
            ticks.extend(b.rows.iter().map(|r| (r.tick, r.hash)));
        }
    }
    assert_eq!(ticks.len() as u64, DEMO_TICKS, "one row a tick");
    assert!(ticks.iter().enumerate().all(|(i, r)| r.0 == i as u64));
    let mut store = Store::default();
    for o in log {
        store.ingest(o).expect("every observation ingests");
    }
    assert_eq!(store.tick(), DEMO_TICKS);
    let reference = reference_hashes(&t, DEMO_TICKS);
    let got: Vec<u64> = ticks.iter().map(|r| r.1).collect();
    assert_eq!(got, reference, "the GUI's demo-gb hashes are the engine's");
    assert_eq!(store.hashes(), reference.as_slice());
    if let Some(dir) = std::env::var_os("RUSTYECON_GUI_HASHES") {
        let mut body = String::new();
        for (tick, hash) in &ticks {
            body.push_str(&Manifest::line(tick + 1, *hash));
        }
        let path = std::path::Path::new(&dir).join("demo-gb.hashes");
        std::fs::write(&path, body).expect("the hash file is written");
    }
}
