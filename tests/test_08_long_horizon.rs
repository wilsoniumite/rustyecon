//! The 225-year tracer bullet (v2 Phase 3, spec docs/PLAN.md).
//!
//! The determinism suite in `test_03` runs 60 ticks. That is enough to prove the
//! three equalities hold *at all*, and says nothing about whether they survive
//! 11,700 of them — which is the horizon the project is actually aiming at, and
//! where float drift, accumulated lot vectors and checkpoint growth live.
//!
//! These tests run the full span. They are the executable form of the Phase 3
//! gate: if the long horizon ever stops certifying, this fails in CI rather than
//! being discovered by a 225-year run somebody launched in anger.

use std::path::PathBuf;

use rustyecon::{
    certify::state_hash,
    output::checkpoint::{self, SaveFormat},
    runner::{RunConfig, SimRunner},
    scenario::loader,
    systems::run_tick,
};

/// 225 years at weekly ticks — the horizon docs/PLAN.md Phase 3 names.
const FULL_SPAN: u64 = 11_700;

fn tracer_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios/tracer_2r")
}

#[test]
fn the_tracer_certifies_at_the_full_span() {
    let dir = tracer_dir();
    let s = loader::load(&dir).expect("tracer_2r loads");
    let out = tempfile::tempdir().unwrap();
    let results = tempfile::tempdir().unwrap();

    let config = RunConfig {
        ticks: FULL_SPAN,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: out.path().to_path_buf(),
        record: false,
        telemetry_every: 1,
        certify: true,
        results_dir: results.path().to_path_buf(),
    };
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&dir)
        .with_criteria(s.criteria);
    let cert = runner.run().expect("certify was requested");

    // Verdict first, and every battery named individually: a bare `passed()`
    // would report "something broke" when the useful information is which.
    for b in &cert.batteries {
        assert!(b.pass, "{} {} failed at {FULL_SPAN} ticks: {}", b.id, b.label, b.detail);
    }
    let stability = cert.stability.as_ref().expect("tracer_2r registers criteria");
    assert!(
        stability.passed(),
        "stability failed at the full span: {}",
        stability.summary()
    );
    assert_eq!(
        stability.window,
        [150, FULL_SPAN],
        "the tracer must be scored over the full span, not the corpus's 150..1000 window"
    );
    assert!(cert.passed(), "verdict was {}", cert.verdict);
}

#[test]
fn the_tracer_actually_moves_for_all_of_it() {
    // Guards the gate above against the degenerate pass. A frozen world
    // certifies trivially and tests nothing; an earlier draft of this scenario
    // did exactly that. The staple price must still be moving at the horizon,
    // not merely have moved once during the transient.
    let dir = tracer_dir();
    let s = loader::load(&dir).expect("tracer_2r loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);

    let node = rustyecon::types::ids::MarketNodeId(0);
    let staple = rustyecon::types::ids::GoodId(0); // "grain"

    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut moved_in_last_thousand = 0usize;
    let mut prev = state.price(node, staple);
    for _ in 0..FULL_SPAN {
        run_tick(&mut state, &gd, &events);
        let p = state.price(node, staple);
        lo = lo.min(p);
        hi = hi.max(p);
        if state.tick > FULL_SPAN - 1000 && p != prev {
            moved_in_last_thousand += 1;
        }
        prev = p;
    }

    assert!(hi / lo > 10.0, "staple price spanned only {:.3}x ({lo:.4} to {hi:.4})", hi / lo);
    assert!(
        moved_in_last_thousand > 900,
        "staple moved on only {moved_in_last_thousand}/1000 of the final ticks — \
         the world settles into a frozen state and stops testing anything"
    );
    // A price that has run away to 1e139 or collapsed into the delta-suppression
    // floor near 1e-11 is "moving" in the same sense a fire is warm.
    assert!(lo > 1e-6 && hi < 1e6, "staple price left the sane band: {lo:.3e} to {hi:.3e}");
}

#[test]
fn resume_is_lossless_at_the_full_span() {
    // test_03 proves this over 50 ticks. The long-horizon question is different:
    // by tick 11,700 inventories carry thousands of accumulated lots, and a
    // checkpoint that reordered or coalesced them would still round-trip the
    // totals while changing every subsequent float sum.
    const T: u64 = FULL_SPAN / 2;
    let dir = tracer_dir();
    let s = loader::load(&dir).expect("tracer_2r loads");
    let (genesis, gd, events) = (s.state, s.game_data, s.events);

    let mut reference = genesis.clone();
    for _ in 0..FULL_SPAN {
        run_tick(&mut reference, &gd, &events);
    }
    let end_hash = state_hash(&reference);

    let mut state = genesis;
    for _ in 0..T {
        run_tick(&mut state, &gd, &events);
    }

    let td = tempfile::tempdir().unwrap();
    let path = td.path().join("halfway.bin");
    checkpoint::save(&state, &SaveFormat::Binary, &path).unwrap();
    let mut resumed = checkpoint::load(&path).unwrap();
    assert_eq!(
        state_hash(&state),
        state_hash(&resumed),
        "checkpoint round-trip must be lossless at tick {T}, lot lives included"
    );

    for _ in 0..(FULL_SPAN - T) {
        run_tick(&mut resumed, &gd, &events);
    }
    assert_eq!(
        end_hash,
        state_hash(&resumed),
        "a run resumed at tick {T} must reach the same state at {FULL_SPAN} as one that never stopped"
    );
}
