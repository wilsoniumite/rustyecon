//! Detector validation against synthetic ground truth (v2 Phase 3.5).
//!
//! The stability rules are the instrument every verdict in this project is
//! measured with, and until Phase 3.5 nothing checked the instrument. Two rules
//! have now been retired for scoring the wrong thing, and no test could notice
//! either, because every test asked whether scenarios passed the detector rather
//! than whether the detector was right:
//!
//! - `Damping` read std(2nd)/std(1st) of a raw *level*, where standard deviation
//!   scales with the level — so it passed a monotone collapse and failed a
//!   healthy stationary economy. Backwards since it was ported.
//! - `LogDrift` fitted an OLS trend to ln(metric) and called it "did the level
//!   stay put". A fit through a V is flat, so a millionfold collapse-and-recovery
//!   scored 1.05 and passed clean; and a one-off step was inflated by
//!   exp(delta/2), tripping a "2x" bar at a true shift of 1.59x.
//!
//! So: series with a known answer by construction. A healthy world is one whose
//! level stays put and whose fluctuation does not grow; a sick one runs away in
//! level, or oscillates harder and harder. These cases are the definition, and
//! the registered thresholds must separate them with margin.
//!
//! This file is also the record of how the thresholds were chosen. They were
//! fixed from the separation *here*, before the corpus was scored — the whole
//! point of R6 is that a threshold picked to make particular scenarios pass is
//! not a threshold.

use rustyecon::certify::verdict::{damping_ratio, level_range, residual_damping};

/// The tracer's scored window, and the split `Rule::ResidualDamping` uses.
const N: usize = 11_550;
const SPLIT: usize = N / 2;

/// Registered in the generation-4 criteria; see data/scenarios/*/criteria.ron.
const MAX_LEVEL_RANGE: f64 = 2.2;
const MAX_OSC_RATIO: f64 = 1.5;

/// exp(x) sampled over the window, so a geometric process is linear in the log.
fn build(f: impl Fn(f64) -> f64) -> Vec<f64> {
    (0..N).map(|i| f(i as f64)).collect()
}

/// Deterministic pseudo-noise — a test that flakes on an RNG is not a test.
fn wobble(i: f64, scale: f64) -> f64 {
    // Three incommensurable frequencies: no period, but perfectly reproducible.
    scale * ((i * 0.7).sin() + (i * 0.113).sin() + (i * 0.0271).sin()) / 3.0
}

fn healthy() -> Vec<(&'static str, Vec<f64>)> {
    vec![
        ("perfectly constant", build(|_| 1.0)),
        ("stationary + 5% ripple", build(|i| (wobble(i, 0.05)).exp())),
        ("stationary + slow cycle", build(|i| (0.05 * (i / 200.0).sin()).exp())),
        ("stationary + settling ripple", build(|i| (0.2 * (-i / 2000.0).exp() * (i / 50.0).sin()).exp())),
        ("mild drift 1.2x over window", build(|i| (1.2f64.ln() * i / N as f64).exp())),
        ("level shifts once, then flat", build(|i| if i < N as f64 / 3.0 { 1.0 } else { 1.15 })),
        ("stationary at a tiny level", build(|i| 1e-9 * (wobble(i, 0.05)).exp())),
        ("stationary at a huge level", build(|i| 1e9 * (wobble(i, 0.05)).exp())),
        // Real economies have cycles. A rule that calls an ordinary business
        // cycle unstable would fail every world worth simulating.
        ("business cycle, +/-15% in logs", build(|i| (0.15 * (i / 900.0).sin()).exp())),
        ("business cycle, +/-30% in logs", build(|i| (0.30 * (i / 900.0).sin()).exp())),
        ("recovering from an early shock", build(|i| {
            (if i < 500.0 { 0.75 } else { 1.0 }) * (wobble(i, 0.02)).exp()
        })),
    ]
}

fn sick() -> Vec<(&'static str, Vec<f64>)> {
    vec![
        ("the tracer's 29.6x deflation", build(|i| 4.32 * (-29.6f64.ln() * i / N as f64).exp())),
        ("30x inflation", build(|i| 0.14 * (30.0f64.ln() * i / N as f64).exp())),
        ("hyperinflation 1e6x", build(|i| 1e-3 * (1e6f64.ln() * i / N as f64).exp())),
        ("collapse toward zero", build(|i| (-i / 1200.0).exp())),
        ("slow 3x drift", build(|i| (3.0f64.ln() * i / N as f64).exp())),
        ("oscillation growing without bound", build(|i| (0.02 * (i / 4000.0).exp() * (i / 50.0).sin()).exp())),
        // A permanent regime shift is not a wobble; the level genuinely moved.
        ("permanent 3x level shift", build(|i| if i < N as f64 / 2.0 { 1.0 } else { 3.0 })),
        // The shape LogDrift was blind to. These are the reason it was retired:
        // a trend fit through a V is flat, so it scored them perfectly stable.
        ("falls 33x and comes back", build(|i| {
            (-33.0f64.ln() * (std::f64::consts::PI * i / N as f64).sin().powi(2)).exp()
        })),
        ("falls 1e6x and comes back", build(|i| {
            (-1e6f64.ln() * (std::f64::consts::PI * i / N as f64).sin().powi(2)).exp()
        })),
        // Volatility doubling is the oscillation rule's own case.
        ("volatility doubles mid-window", build(|i| {
            (wobble(i, 0.03) * if i as usize >= SPLIT { 2.0 } else { 1.0 }).exp()
        })),
    ]
}

/// Does a case fire either fatal rule?
fn fires(v: &[f64]) -> bool {
    level_range(v) > MAX_LEVEL_RANGE || residual_damping(v, SPLIT) > MAX_OSC_RATIO
}

#[test]
fn a_healthy_economy_is_not_flagged() {
    for (name, v) in healthy() {
        assert!(
            !fires(&v),
            "'{name}' is stable by construction but fired: range={:.3}x osc={:.3}",
            level_range(&v),
            residual_damping(&v, SPLIT)
        );
    }
}

#[test]
fn a_sick_economy_is_always_flagged() {
    for (name, v) in sick() {
        assert!(
            fires(&v),
            "'{name}' is unstable by construction but passed: range={:.3}x osc={:.3}",
            level_range(&v),
            residual_damping(&v, SPLIT)
        );
    }
}

#[test]
fn the_thresholds_keep_real_margin_on_both_sides() {
    // Not just "it separates today" — it separates with room, so a scenario a few
    // percent either way cannot flip the verdict.
    //
    // The margin is ~1.2x per side on the level rule, tighter than the ~1.7x an
    // earlier draft claimed. That draft bought its room by only testing
    // textbook-clean cases. The real squeeze: a business cycle swinging +/-30% in
    // logs reads 1.81, and a slow 3x drift reads 2.69 — those are genuinely close,
    // and 2.2 is the geometric midpoint between them. Widening the bar passes the
    // drift; narrowing it fails the cycle. This is where the boundary honestly
    // is, and the assertion records it rather than flattering it.
    let worst_healthy_level = healthy().iter().map(|(_, v)| level_range(v)).fold(0.0, f64::max);
    let worst_healthy_osc = healthy()
        .iter()
        .map(|(_, v)| residual_damping(v, SPLIT))
        .filter(|r| r.is_finite())
        .fold(0.0, f64::max);
    // Only the level-sick cases; the oscillation ones are the other rule's job.
    let mildest_sick_level = sick()
        .iter()
        .map(|(_, v)| level_range(v))
        .filter(|d| *d > MAX_LEVEL_RANGE)
        .fold(f64::INFINITY, f64::min);
    let mildest_sick_osc = sick()
        .iter()
        .map(|(_, v)| residual_damping(v, SPLIT))
        .filter(|r| *r > MAX_OSC_RATIO)
        .fold(f64::INFINITY, f64::min);

    for (what, healthy_side, sick_side, bar) in [
        ("level", worst_healthy_level, mildest_sick_level, MAX_LEVEL_RANGE),
        ("oscillation", worst_healthy_osc, mildest_sick_osc, MAX_OSC_RATIO),
    ] {
        assert!(
            healthy_side < bar / 1.2,
            "{what}: healthiest case {healthy_side:.3} is too close below the {bar} bar"
        );
        assert!(
            sick_side > bar * 1.2,
            "{what}: mildest sick case {sick_side:.3} is too close above the {bar} bar"
        );
    }
}

#[test]
fn the_replacement_fixes_what_the_old_rule_got_backwards() {
    // The specific inversion Phase 3.5 exists to correct, pinned so it cannot
    // return: the legacy rule's verdict on these two is the wrong way round.
    let deflating = build(|i| 4.32 * (-29.6f64.ln() * i / N as f64).exp());
    let stationary = build(|i| (wobble(i, 0.05)).exp());

    // Legacy: passes the collapse, fails the healthy economy.
    assert!(damping_ratio(&deflating, SPLIT) <= 0.72, "the old rule passed the collapse");
    assert!(damping_ratio(&stationary, SPLIT) > 0.72, "the old rule failed the healthy economy");

    // Split rules: the level runaway is caught, the healthy economy is left alone.
    assert!(level_range(&deflating) > MAX_LEVEL_RANGE, "the collapse must be caught as drift");
    assert!(!fires(&stationary), "the healthy economy must not fire either rule");
}

#[test]
fn the_level_range_is_blind_to_scale_and_to_direction() {
    // A 3x fall and a 3x rise are the same magnitude of instability, and neither
    // should depend on whether the metric is denominated in millions or
    // billionths. For a monotone geometric ramp spanning a factor F the
    // 95th/5th-percentile band is F^0.9, since the percentiles trim 5% off each
    // end — so the statistic slightly under-reports a pure ramp, which is the
    // price of not letting one freak sample decide a verdict.
    let up = build(|i| 1e-7 * (3.0f64.ln() * i / N as f64).exp());
    let down = build(|i| 1e7 * (-3.0f64.ln() * i / N as f64).exp());
    let expected = 3.0f64.powf(0.9);
    for (name, v) in [("rise", &up), ("fall", &down)] {
        assert!(
            (level_range(v) - expected).abs() < 0.02,
            "{name}: got {:.4}, expected {expected:.4}",
            level_range(v)
        );
    }
}

#[test]
fn the_thresholds_hold_at_the_corpus_window_too() {
    // The corpus scores ticks 150..1000 — 850 samples, not 11,550. `LogDrift` is
    // registered as a total factor *across the window*, so a short run is held
    // to a looser per-tick rate on purpose: it is a claim about the span that
    // was actually observed. Check the separation survives the shorter span,
    // since the thresholds were picked at the long one.
    const M: usize = 850;
    let short = |f: &dyn Fn(f64) -> f64| -> Vec<f64> { (0..M).map(|i| f(i as f64)).collect() };

    let ok_flat = short(&|i| (wobble(i, 0.05)).exp());
    let ok_mild = short(&|i| (1.2f64.ln() * i / M as f64).exp());
    let bad_drift = short(&|i| (3.0f64.ln() * i / M as f64).exp());
    let bad_collapse = short(&|i| (-i / 120.0).exp());

    for (name, v) in [("stationary", &ok_flat), ("mild drift", &ok_mild)] {
        assert!(
            level_range(v) <= MAX_LEVEL_RANGE && residual_damping(v, M / 2) <= MAX_OSC_RATIO,
            "'{name}' fired at the corpus window: range={:.3} osc={:.3}",
            level_range(v),
            residual_damping(v, M / 2)
        );
    }
    for (name, v) in [("3x drift", &bad_drift), ("collapse", &bad_collapse)] {
        assert!(level_range(v) > MAX_LEVEL_RANGE, "'{name}' was missed at the corpus window");
    }
}

#[test]
fn an_uncomputable_series_is_uncomputable_not_stable() {
    // Fail-closed depends on these being NaN rather than a comfortable number:
    // a dead region must not read as a calm one (engine.md, "NaN = FAIL").
    let dead = vec![0.0; N];
    let absent = vec![f64::NAN; N];
    assert!(level_range(&dead).is_nan(), "an all-zero series has no drift to report");
    assert!(level_range(&absent).is_nan());
    assert!(residual_damping(&dead, SPLIT).is_nan());
    assert!(residual_damping(&absent, SPLIT).is_nan());

    // Alive for only half the window: too little to judge the quiet half.
    let half_dead: Vec<f64> = (0..N).map(|i| if i < SPLIT { 0.0 } else { 1.0 }).collect();
    assert!(residual_damping(&half_dead, SPLIT).is_nan());
}
