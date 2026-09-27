//! The Runner without a thread (docs/GUI.md §3.3): commands pause where they say, the ring
//! falls on each year's first tick and holds the run's own states, a resume from it continues
//! the run exactly, a refused resume reruns from genesis, and snapshots read the state of their
//! tick now, later or from the ring.

mod common;

use common::{collecting, drain, edit, reference_hashes, sim_at, tape_of, GATE, GATE_TICKS};
use rustyecon_engine::prelude::*;
use rustyecon_gui::run::{
    ring_tick_at_or_after, Cmd, Obs, PauseReason, Refusal, ResumeFrom, RingCheckpoint, Runner,
};

/// Advance until the run stops asking for slices, `slice` ticks at a time.
fn settle(r: &mut Runner, slice: u32) {
    while r.advance(slice).busy {}
}

fn load(r: &mut Runner, t: &Tape) {
    r.handle(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
}

fn pauses(obs: &[Obs]) -> Vec<(u64, PauseReason)> {
    obs.iter()
        .filter_map(|o| match o {
            Obs::Paused { tick, why } => Some((*tick, why.clone())),
            _ => None,
        })
        .collect()
}

fn hashes(obs: &[Obs]) -> Vec<(u64, u64)> {
    obs.iter()
        .filter_map(|o| match o {
            Obs::Batch(b) => Some(b.rows.iter().map(|r| (r.tick, r.hash)).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn ring(obs: &[Obs]) -> Vec<RingCheckpoint> {
    obs.iter()
        .filter_map(|o| match o {
            Obs::Checkpointed(cp) => Some(cp.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn commands_pause_where_they_say() {
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    load(&mut r, &t);
    let o = drain(&seen);
    assert!(matches!(o[0], Obs::Loaded { tick: 0, .. }), "{:?}", o[0]);
    assert!(!r.busy());
    // Step 3 in a slice of 100: three ticks, then the pause, in the same slice.
    r.handle(Cmd::Step(3));
    let p = r.advance(100);
    assert_eq!((p.ran, p.busy), (3, false));
    let o = drain(&seen);
    assert!(matches!(o[0], Obs::Running { tick: 0 }));
    assert_eq!(pauses(&o), [(3, PauseReason::Stepped)]);
    // Run until 10 in slices of 4: 4 ticks, still busy; then 3, and the pause.
    r.handle(Cmd::Run {
        until: Some(10),
        max_tps: Some(5),
    });
    let p = r.advance(4);
    assert_eq!((p.ran, p.busy), (4, true));
    let p = r.advance(4);
    assert_eq!((p.ran, p.busy), (3, false));
    assert_eq!(pauses(&drain(&seen)), [(10, PauseReason::Reached(10))]);
    // An until already reached pauses at once; a pause while paused says nothing.
    r.handle(Cmd::Run {
        until: Some(5),
        max_tps: None,
    });
    r.handle(Cmd::Pause);
    r.handle(Cmd::Step(0));
    assert_eq!(
        pauses(&drain(&seen)),
        [(10, PauseReason::Reached(5)), (10, PauseReason::Stepped)]
    );
    assert_eq!(r.advance(100).ran, 0);
    // Run on, pause between slices.
    r.handle(Cmd::Run {
        until: None,
        max_tps: None,
    });
    assert_eq!(r.advance(6).ran, 6);
    r.handle(Cmd::Pause);
    assert_eq!(r.advance(6).ran, 0);
    let o = drain(&seen);
    assert_eq!(pauses(&o), [(16, PauseReason::Asked)]);
    // Every tick so far, once, in order, as the engine alone runs them.
    let (mut r2, seen2) = collecting();
    load(&mut r2, &t);
    r2.handle(Cmd::Run {
        until: Some(16),
        max_tps: None,
    });
    settle(&mut r2, 1);
    let one_by_one = hashes(&drain(&seen2));
    let reference = reference_hashes(&t, 16);
    assert_eq!(
        one_by_one.iter().map(|x| x.1).collect::<Vec<_>>(),
        reference
    );
}

#[test]
fn the_ring_holds_each_years_first_state() {
    // D4: the ring is the state at the tick each 1 January falls in, genesis included, and each
    // checkpoint holds the run's own state there: its state hash is the uninterrupted run's.
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    load(&mut r, &t);
    r.handle(Cmd::Run {
        until: Some(GATE_TICKS),
        max_tps: None,
    });
    settle(&mut r, 97);
    let o = drain(&seen);
    let cps = ring(&o);
    let clock = Sim::new(&t).unwrap().world().clock;
    let mut expected = Vec::new();
    let mut at = Some(0);
    while let Some(k) = at.filter(|&k| k <= GATE_TICKS) {
        expected.push(k);
        at = ring_tick_at_or_after(&clock, k + 1);
    }
    assert_eq!(expected.len(), 41, "1750 to 1790");
    assert_eq!(cps.iter().map(|c| c.tick()).collect::<Vec<_>>(), expected);
    let reference = reference_hashes(&t, GATE_TICKS);
    let genesis = Sim::new(&t).unwrap().hash();
    let world_id = Sim::new(&t).unwrap().world().world_id;
    for cp in &cps {
        assert_eq!(cp.run().world_id.0, world_id);
        let decoded = cp.checkpoint().expect("a ring checkpoint decodes");
        let want = if cp.tick() == 0 {
            genesis
        } else {
            reference[cp.tick() as usize - 1]
        };
        assert_eq!(decoded.state().tick(), cp.tick());
        assert_eq!(
            rustyecon_engine::prelude::Sim::resume(&t, &decoded)
                .unwrap()
                .hash(),
            want
        );
    }
}

#[test]
fn a_resume_from_the_ring_continues_the_run() {
    // D4, E1: a branch's Runner loads its parent's ring checkpoint and continues from it; with
    // the same tape, its hashes are the uninterrupted run's tail.
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    load(&mut r, &t);
    r.handle(Cmd::Run {
        until: Some(900),
        max_tps: None,
    });
    settle(&mut r, 50);
    let cp = ring(&drain(&seen))
        .into_iter()
        .rfind(|c| c.tick() <= 800)
        .expect("a ring checkpoint by tick 800");
    let from = cp.tick();
    let (mut b, bseen) = collecting();
    b.handle(Cmd::Load {
        tape: Box::new(t.clone()),
        from: Some(ResumeFrom::Ring(cp)),
    });
    b.handle(Cmd::Run {
        until: Some(GATE_TICKS),
        max_tps: None,
    });
    settle(&mut b, 64);
    let o = drain(&bseen);
    assert!(matches!(o[0], Obs::Loaded { tick, .. } if tick == from));
    let got: Vec<u64> = hashes(&o).into_iter().map(|x| x.1).collect();
    let reference = reference_hashes(&t, GATE_TICKS);
    assert_eq!(got, reference[from as usize..]);
}

#[test]
fn a_refused_resume_reruns_from_genesis_and_a_bad_tape_is_refused() {
    // docs/GUI.md §3.3: when a check fails, the GUI reruns from genesis and logs why. A ring
    // checkpoint of the gate under a tape whose past differs is refused as WrongPrefix.
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    load(&mut r, &t);
    r.handle(Cmd::Run {
        until: Some(600),
        max_tps: None,
    });
    settle(&mut r, 100);
    let cp = ring(&drain(&seen)).pop().expect("a ring checkpoint");
    let past = tape_of(&edit(
        GATE,
        r#"(key: "mine.cut", at: "1760-03-01","#,
        r#"(key: "mine.cut", at: "1752-03-01","#,
    ));
    let (mut b, bseen) = collecting();
    b.handle(Cmd::Load {
        tape: Box::new(past),
        from: Some(ResumeFrom::Ring(cp)),
    });
    let o = drain(&bseen);
    assert!(
        matches!(
            &o[0],
            Obs::Refused(Refusal::Resume(ResumeError::WrongPrefix { .. }))
        ),
        "{:?}",
        o[0]
    );
    assert!(matches!(o[1], Obs::Loaded { tick: 0, .. }));
    // A tape that parses and does not resolve: refused, and no run.
    let salt = tape_of(&edit(
        GATE,
        r#"(node: "village", good: "grain", qty: "mill.buy.grain.village", weight: 0.375)"#,
        r#"(node: "village", good: "salt", qty: "mill.buy.grain.village", weight: 0.375)"#,
    ));
    load(&mut b, &salt);
    let o = drain(&bseen);
    assert_eq!(o.len(), 1);
    match &o[0] {
        Obs::Refused(Refusal::Load(e)) => {
            assert!(
                e.to_string()
                    .contains("actors[mill].spec.buy[village/salt].good"),
                "{e}"
            );
        }
        other => panic!("{other:?}"),
    }
    b.handle(Cmd::Step(5));
    assert_eq!(b.advance(10).ran, 0);
    assert!(drain(&bseen).is_empty());
}

#[test]
fn snapshots_read_the_state_of_their_tick() {
    // Deep inspection (docs/GUI.md §3.3): a snapshot of the current tick is taken now, of a
    // later one when the run reaches it, and of an earlier one from a scratch Sim resumed at the
    // latest ring checkpoint at or before it. Each equals the state an uninterrupted run has
    // there, and none moves the run.
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    load(&mut r, &t);
    r.handle(Cmd::Snapshot(0));
    r.handle(Cmd::Snapshot(300));
    r.handle(Cmd::Run {
        until: Some(400),
        max_tps: None,
    });
    settle(&mut r, 33);
    r.handle(Cmd::Snapshot(250));
    r.handle(Cmd::Snapshot(20));
    r.handle(Cmd::Run {
        until: Some(420),
        max_tps: None,
    });
    settle(&mut r, 33);
    let o = drain(&seen);
    let snaps: Vec<_> = o
        .iter()
        .filter_map(|x| match x {
            Obs::Snapshot(s) => Some(s.as_ref().clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        snaps.iter().map(|s| s.tick).collect::<Vec<_>>(),
        [0, 300, 250, 20]
    );
    for s in &snaps {
        let sim = sim_at(&t, s.tick);
        assert_eq!(s.hash, sim.hash(), "tick {}", s.tick);
        assert_eq!(s.holdings, sim.observe_holdings(), "tick {}", s.tick);
        assert_eq!(s.actors.len(), sim.world().actors.len());
        // The lots behind each total, as the state holds them (the actor inspector reads them).
        assert_eq!(s.lots.len(), s.holdings.0.len());
        for ((h, g, lots), (h2, g2, _)) in s.lots.iter().zip(&s.holdings.0) {
            assert_eq!((h, g), (h2, g2));
            assert_eq!(
                lots.as_slice(),
                sim.holding(*h).unwrap().lots(*g),
                "tick {}",
                s.tick
            );
        }
    }
    // Bread spoils, so some holding has lots of more than one life by tick 300.
    let s300 = &snaps[1];
    assert!(
        s300.lots.iter().any(|(_, _, l)| l.len() > 1),
        "{:?}",
        s300.lots
    );
    let got: Vec<u64> = hashes(&o).into_iter().map(|x| x.1).collect();
    assert_eq!(got, reference_hashes(&t, 420));
}
