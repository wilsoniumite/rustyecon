//! Criteria + stability scoring integration tests (v2 Phase 1).
//!
//! The Phase 1 gate: the engine batteries pass while the corpus fails on
//! *stability*. Those are separate claims and the certificate must keep them
//! separate — a conserved, deterministic, NaN-free run of a collapsing economy
//! is exactly what these scenarios are.

use std::path::PathBuf;

use rustyecon::{
    certify::{Certificate, Criteria},
    runner::{RunConfig, SimRunner},
    scenario::loader,
};

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/scenarios")
        .join(name)
}

fn certify(name: &str, ticks: u64, criteria: Option<Criteria>) -> Certificate {
    let dir = scenario_dir(name);
    let s = loader::load(&dir).expect("scenario loads");
    let out = tempfile::tempdir().unwrap();
    let config = RunConfig {
        agents: rustyecon::kernel::AgentArm::Legacy,
        ticks,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: out.path().to_path_buf(),
        record: false,
        telemetry_every: 1,
        certify: true,
        results_dir: out.path().join("results"),
    };
    let criteria = criteria.or(s.criteria);
    SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&dir)
        .with_criteria(criteria)
        .run()
        .expect("certificate")
}

#[test]
fn every_scenario_registers_criteria() {
    // An unscored run must never be mistakable for a clean one, so the corpus
    // is expected to carry criteria everywhere.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&root).unwrap().flatten() {
        let dir = entry.path();
        if dir.join("game_data.ron").exists() && !dir.join("criteria.ron").exists() {
            missing.push(dir.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(missing.is_empty(), "scenarios without criteria.ron: {missing:?}");
}

#[test]
fn criteria_load_and_carry_the_ported_window() {
    let s = loader::load(&scenario_dir("lr_00")).unwrap();
    let c = s.criteria.expect("lr_00 registers criteria");
    // The 07-suite window: TRANSIENT=150, ANALYSIS_END=1000 -> mid 575.
    assert_eq!(c.transient, 150);
    assert_eq!(c.analysis_end, 1000);
    assert_eq!(c.analysis_mid(), 575);
    assert!(!c.date.is_empty(), "criteria must be dated (R6)");
    let names: Vec<&str> = c.classes.iter().map(|k| k.name.as_str()).collect();
    for expected in ["DEAD", "UNSTABLE", "SWINGING", "CURRENCY_DRAIN", "POP_DESTITUTION"] {
        assert!(names.contains(&expected), "missing class {expected}");
    }
    // DEAD_BUILDING is a warning, not a region failure.
    let warn = c.classes.iter().find(|k| k.name == "DEAD_BUILDING").unwrap();
    assert!(!warn.fatal);
}

#[test]
fn engine_batteries_pass_while_stability_fails() {
    let cert = certify("lr_00", 1000, None);

    // Engine correctness is a separate claim from economic stability.
    //
    // B6 excepted: it checks an agent-design invariant rather than engine
    // correctness, and the legacy agent violates it — see test_10_invariants.
    // Excluded here rather than dropped, so the distinction stays visible.
    for b in cert.batteries.iter().filter(|b| b.id != "B6") {
        assert!(b.pass, "engine battery {} should pass: {}", b.id, b.detail);
    }
    let s = cert.stability.as_ref().expect("scored against criteria");
    assert!(!s.passed(), "lr_00 is expected to fail on stability");
    assert!(s.regions_total > 0);
    assert!(
        s.regions.iter().any(|r| !r.issues.is_empty()),
        "a failing region must say why"
    );

    // The overall verdict folds stability in, so a collapsing economy cannot
    // certify clean just because the engine behaved.
    assert_eq!(cert.verdict, "FAIL");
    let text = cert.render();
    assert!(text.contains("STABILITY:"));
    assert!(text.trim_end().ends_with("VERDICT: FAIL"));
}

#[test]
fn a_run_without_criteria_is_reported_unscored_not_clean() {
    // Deliberately drop the criteria: the certificate must say so rather than
    // quietly reporting a pass it never checked.
    let cert = certify("lr_00", 60, None);
    assert!(cert.stability.is_some(), "sanity: lr_00 does register criteria");

    let unscored = Certificate::new(cert.identity.clone(), cert.batteries.clone());
    assert!(unscored.stability.is_none());
    assert!(
        unscored.render().contains("no criteria.ron registered — unscored"),
        "an unscored run must say so"
    );
}

#[test]
fn scoring_is_reproducible() {
    let a = certify("lr_00", 300, None);
    let b = certify("lr_00", 300, None);
    let issues = |c: &Certificate| {
        c.stability
            .as_ref()
            .unwrap()
            .regions
            .iter()
            .map(|r| r.issues.join(","))
            .collect::<Vec<_>>()
    };
    assert_eq!(issues(&a), issues(&b), "same run must score identically");
}

#[test]
fn an_infinite_damping_ratio_survives_the_certificate_boundary() {
    // v2 Phase 3 made damping_ratio return INFINITY for a series that is flat and
    // then erupts, where it used to return 0.0 without looking at the second half.
    // serde_json cannot represent a non-finite f64, so the question is whether one
    // can reach a serialized field. It cannot — the ratio is formatted into the
    // issue string before it lands in RegionVerdict — and this pins that down, so
    // a later refactor that stores the raw ratio fails here rather than in the
    // field, at the end of a 225-year run, as an unwritable certificate.
    use rustyecon::certify::{criteria::*, metrics::RegionSeries, verdict};

    let criteria = Criteria {
        version: 1,
        date: "2026-07-20".into(),
        transient: 0,
        analysis_end: 40,
        staple_good: "g".into(),
        labour_good: "l".into(),
        nan_policy: NanPolicy::FailClosed,
        classes: vec![FailureClass {
            name: "SWINGING".into(),
            fatal: true,
            metrics: vec![Metric::Velocity],
            rule: Rule::Damping { max_ratio: 0.72 },
            window: Window::Halves,
            short_circuit: false,
            nan_policy: None,
        }],
    };

    let mut v: Vec<f64> = vec![1.0; 20];
    v.extend((0..20).map(|i| if i % 2 == 0 { 5.0 } else { -5.0 }));
    let series = RegionSeries {
        region: rustyecon::types::ids::RegionId(0),
        name: "Probe".into(),
        velocity: v.clone(),
        employment: v.clone(),
        real_wage: v.clone(),
        concentration: v.clone(),
        bld_util: v.clone(),
        real_income: v,
        building_ids: Vec::new(),
        building_util: Vec::new(),
    };

    let report = verdict::evaluate(&criteria, &[series]);
    assert!(!report.passed(), "a flat-then-erupting series must fail SWINGING");

    let json = serde_json::to_string(&report)
        .expect("the report must serialize even when a ratio was infinite");
    let back: verdict::StabilityReport =
        serde_json::from_str(&json).expect("and must round-trip");
    assert_eq!(back.regions[0].issues, report.regions[0].issues);
    assert!(
        report.regions[0].issues.iter().any(|i| i.contains("inf")),
        "the reader should see that the ratio was unbounded: {:?}",
        report.regions[0].issues
    );
}
