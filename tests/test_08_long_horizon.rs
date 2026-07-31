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

/// The engine batteries — conservation, delta-only mutation, determinism, no
/// NaN, no clamps — must hold across 225 years. That is the claim Phase 3 can
/// actually make, and it is separate from whether the *economy* is stable.
#[test]
fn the_engine_holds_together_across_the_full_span() {
    let dir = tracer_dir();
    let s = loader::load(&dir).expect("tracer_2r loads");
    let out = tempfile::tempdir().unwrap();
    let results = tempfile::tempdir().unwrap();

    let config = RunConfig {
        agents: rustyecon::kernel::AgentArm::Legacy,
        price_rule: rustyecon::systems::clearing::PriceRule::Imbalance,
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
    // B6 and B7 excepted. Both check agent design rather than engine
    // correctness, and the legacy agent this tracer runs under fails both by
    // construction: it posts `1.2 x demand`, which is the B6 violation, and
    // that cap pins grain's imbalance at -1/6 for 10,159 unbroken ticks, which
    // is the B7 one. Neither is news — engine.md, "The 225-year horizon",
    // records the pin, found by hand before either battery existed. What is new
    // is that a battery finds it now; test_10_invariants asserts that it does,
    // so excluding them here loses no coverage. Everything else must hold at
    // the 225-year horizon.
    for b in cert.batteries.iter().filter(|b| b.id != "B6" && b.id != "B7") {
        assert!(b.pass, "{} {} failed at {FULL_SPAN} ticks: {}", b.id, b.label, b.detail);
    }
    let stability = cert.stability.as_ref().expect("tracer_2r registers criteria");
    assert_eq!(
        stability.window,
        [150, FULL_SPAN],
        "the tracer must be scored over the full span, not the corpus's 150..1000 window"
    );
    assert!(
        stability.regions_total >= 2,
        "the scenario must score more than one region"
    );

    // Stability is deliberately NOT asserted here, and the certificate verdict
    // is deliberately FAIL. Under the criteria registered on 2026-07-20 this
    // scenario passed, and that PASS was the artifact the whole of Phase 3.1–3.5
    // exists to retract: the old Damping rule read a raw level, so it scored
    // this world's monotone deflation as "settling". Generation 3 splits the
    // rule and the deflation is caught as DRIFTING. Asserting a PASS here again
    // would re-import the bug as a test.
    assert!(
        !cert.passed(),
        "tracer_2r now certifies PASS. Either the economy was genuinely fixed — \
         in which case update engine.md's \"225-year horizon\" section and this \
         test together — or a detector regressed. Do not just flip this assert."
    );
    assert!(
        stability.regions.iter().all(|r| r.issues.iter().any(|i| i.starts_with("DRIFTING"))),
        "expected every region to fail on DRIFTING (the deflation ramp); got {:?}",
        stability.regions.iter().map(|r| &r.issues).collect::<Vec<_>>()
    );
    // NOTE, so this is not read as more than it is: the two regions are
    // identical clones with no channel and the engine has no RNG, so Beta's
    // series are bit-identical to Alpha's — one trajectory scored twice.
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
        run_tick(&mut state, &gd, &events, rustyecon::kernel::AgentArm::Legacy, rustyecon::systems::clearing::PriceRule::Imbalance);
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
        run_tick(&mut reference, &gd, &events, rustyecon::kernel::AgentArm::Legacy, rustyecon::systems::clearing::PriceRule::Imbalance);
    }
    let end_hash = state_hash(&reference);

    let mut state = genesis;
    for _ in 0..T {
        run_tick(&mut state, &gd, &events, rustyecon::kernel::AgentArm::Legacy, rustyecon::systems::clearing::PriceRule::Imbalance);
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
        run_tick(&mut resumed, &gd, &events, rustyecon::kernel::AgentArm::Legacy, rustyecon::systems::clearing::PriceRule::Imbalance);
    }
    assert_eq!(
        end_hash,
        state_hash(&resumed),
        "a run resumed at tick {T} must reach the same state at {FULL_SPAN} as one that never stopped"
    );
}
