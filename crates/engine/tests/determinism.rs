//! Determinism on the gate world (docs/ENGINE.md §11, engine; R8, N11, E8): test_03's repeat,
//! resume and replay design (`v2p3: tests/test_03_determinism.rs`), re-targeted from lr_00 and
//! the July agents to the gate world, and run over all 2,080 ticks. Every comparison is of
//! hashes, so it is exact.

mod common;

use common::*;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::RawSpec;
use rustyecon_engine::rustyecon_core::{apply, resolve, state_hash, Ledger};

#[test]
fn gate_repeat_identical_hashes() {
    let t = tape();
    let a = hashes(&t, TICKS);
    let b = hashes(&t, TICKS);
    assert_eq!(a.len(), TICKS as usize);
    assert_eq!(
        a, b,
        "the same tape and code must give the same hash stream"
    );
    // Guard against the degenerate pass where nothing changes from tick to tick.
    assert!(
        a.windows(2).all(|w| w[0] != w[1]),
        "the state must evolve every tick"
    );
    let mut distinct = a.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(distinct.len(), a.len());
}

#[test]
fn gate_resume_from_checkpoints() {
    // Checkpoint at ticks 1, 520, 1,040 and 2,079 through both forms; each resumed run reloads
    // to an equal hash, and its tail equals the uninterrupted run tick for tick (lot lives
    // included, defect 5).
    let t = tape();
    let reference = hashes(&t, TICKS);
    for at in [1u64, 520, 1040, 2079] {
        let mut sim = sim_of(&t);
        sim.run_until(at, &mut |_| {}).unwrap();
        let cp = sim.checkpoint().unwrap();
        assert_eq!(cp.state().tick(), at);
        let from_bytes = Checkpoint::from_bytes(&cp.to_bytes()).expect("the bytes decode");
        let from_ron = Checkpoint::from_ron(&cp.to_ron()).expect("the text decodes");
        assert_eq!(from_bytes, cp);
        assert_eq!(from_ron, cp);
        for loaded in [from_bytes, from_ron] {
            let mut resumed = Sim::resume(&t, &loaded).expect("the checkpoint resumes");
            assert_eq!(resumed.hash(), sim.hash(), "the round trip is lossless");
            assert_eq!(resumed.tick(), at);
            let mut tail = Vec::new();
            resumed
                .run_until(TICKS, &mut |r| tail.push(r.hash))
                .unwrap();
            assert_eq!(
                tail.as_slice(),
                &reference[at as usize..],
                "the run resumed at {at} must track the uninterrupted run"
            );
        }
    }
}

#[test]
fn gate_replay_matches_every_tick() {
    // A shadow state that only applies each tick's trace through core::apply matches the live
    // run's hash after every tick: apply is the only writer.
    let t = tape();
    let reference = hashes(&t, TICKS);
    let end = audit_replay(&t, TICKS).expect("the replay matches every tick");
    assert_eq!(end, *reference.last().unwrap());
    // The comparison is sensitive: the whole trace reproduces tick 0, and the trace less any
    // single entry does not (except the ageing of a holder with no perishable lot, which
    // changes nothing).
    let mut live = sim_of(&t);
    let (report, trace) = live.step_traced().unwrap();
    // A shadow that cannot apply a later entry has diverged too: `None`.
    let shadow = |skip: Option<usize>| {
        let (w, mut s) = resolve(&t).unwrap();
        let mut l = Ledger::open(&s, &w).unwrap();
        for (i, e) in trace.0.iter().enumerate() {
            if Some(i) != skip {
                apply(&mut s, &w, e.phase, std::slice::from_ref(&e.delta), &mut l).ok()?;
            }
        }
        Some(state_hash(&s))
    };
    assert_eq!(shadow(None), Some(report.hash));
    let mill = Holder::Actor(actor(live.world(), "mill"));
    for (i, e) in trace.0.iter().enumerate() {
        let ages_bread = e.delta == StateDelta::Age { holder: mill };
        if ages_bread || !matches!(e.delta, StateDelta::Age { .. }) {
            assert_ne!(
                shadow(Some(i)),
                Some(report.hash),
                "entry {i} changed nothing"
            );
        }
    }
}

/// Reverse every list of a tape, the specs' own lists included.
fn reversed(t: &Tape) -> Tape {
    let mut r = t.clone();
    r.params.reverse();
    r.goods.reverse();
    r.nodes.reverse();
    r.channels.reverse();
    r.classes.reverse();
    r.actors.reverse();
    for a in &mut r.actors {
        let RawSpec::Scripted(s) = &mut a.spec;
        s.buy.reverse();
        s.sell.reverse();
        if let Some(rec) = &mut s.recipe {
            rec.inputs.reverse();
            rec.outputs.reverse();
        }
        if let Some(p) = &mut s.payout {
            p.to.reverse();
        }
    }
    r.genesis.prices.reverse();
    r.genesis.holdings.reverse();
    for h in &mut r.genesis.holdings {
        h.goods.reverse();
    }
    r.events.reverse();
    r.recurring.reverse();
    r
}

#[test]
fn file_order_is_irrelevant_to_the_hash_stream() {
    // E8: the loader orders everything by key, never by file position, so permuting every list
    // (and writing the canonical text) changes no id and no hash.
    let t = tape();
    let reference = hashes(&t, TICKS);
    let r = reversed(&t);
    assert_ne!(r, t);
    assert_eq!(hashes(&r, TICKS), reference);
    let canonical = tape_of(&t.to_ron());
    assert_eq!(hashes(&canonical, TICKS), reference);
}
