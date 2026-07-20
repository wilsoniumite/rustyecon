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
    assert!(
        stability.regions_total >= 2,
        "the scenario must score more than one region"
    );
    // NOTE, so this test is not read as more than it is: the two regions are
    // identical clones with no channel and the engine has no RNG, so Beta's
    // series are bit-identical to Alpha's. "2/2" is one trajectory scored twice,
    // not two samples. See engine.md, "The 225-year horizon".
}

/// What the tracer is doing in the back half of its scored window, measured.
struct LateBehaviour {
    price_lo: f64,
    price_hi: f64,
    price_direction_changes: usize,
    distinct_cleared_staple: usize,
}

/// Walk the run and characterise everything after `from`.
fn late_behaviour(from: u64) -> LateBehaviour {
    let dir = tracer_dir();
    let s = loader::load(&dir).expect("tracer_2r loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);
    let node = rustyecon::types::ids::MarketNodeId(0);
    let staple = rustyecon::types::ids::GoodId(0); // "grain"

    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut turns, mut last_dir) = (0usize, 0i8);
    let mut prev = state.price(node, staple);
    let mut cleared: Vec<u64> = Vec::new();

    for _ in 0..FULL_SPAN {
        run_tick(&mut state, &gd, &events);
        if state.tick < from {
            prev = state.price(node, staple);
            continue;
        }
        let p = state.price(node, staple);
        lo = lo.min(p);
        hi = hi.max(p);
        let dir = if p > prev { 1 } else if p < prev { -1 } else { 0 };
        if dir != 0 {
            if last_dir != 0 && dir != last_dir {
                turns += 1;
            }
            last_dir = dir;
        }
        prev = p;
        // Quantity actually changing hands, as the clearing rule defines it.
        let q = state.supply(node, staple).min(state.demand(node, staple));
        cleared.push(q.to_bits());
    }
    cleared.sort_unstable();
    cleared.dedup();
    LateBehaviour {
        price_lo: lo,
        price_hi: hi,
        price_direction_changes: turns,
        distinct_cleared_staple: cleared.len(),
    }
}

#[test]
fn the_tracer_prices_stay_inside_a_sane_band() {
    // A price that has run away toward 1e139 or collapsed into the
    // delta-suppression floor near 1e-11 is "moving" in the sense a fire is warm.
    let b = late_behaviour(150);
    assert!(
        b.price_lo > 1e-6 && b.price_hi < 1e6,
        "staple price left the sane band: {:.3e} to {:.3e}",
        b.price_lo,
        b.price_hi
    );
}

/// The tracer's real economy freezes after its transient; only the nominal price
/// level still moves, and it moves one way. Recorded as a test so the fact
/// cannot quietly stop being true, and so nobody reads the full-span PASS as
/// evidence of a live economy at the horizon.
///
/// This is *not* the property Phase 3 wanted. It is the property Phase 3 got, and
/// pinning it down is what stops the next draft from rediscovering it by
/// accident. See docs/architecture/engine.md, "The 225-year horizon".
#[test]
fn the_tracers_real_economy_is_frozen_after_the_transient() {
    let b = late_behaviour(1550);
    assert_eq!(
        b.distinct_cleared_staple, 1,
        "the tracer used to clear exactly one quantity of staple on every tick \
         after ~1550; it now clears {} distinct quantities. If the world has been \
         made genuinely live, delete this test and say so in engine.md.",
        b.distinct_cleared_staple
    );
    assert_eq!(
        b.price_direction_changes, 0,
        "the staple price used to fall monotonically after ~1550 with no turning \
         points; it now has {}. Same instruction as above.",
        b.price_direction_changes
    );
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
