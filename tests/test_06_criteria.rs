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
        ticks,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: out.path().to_path_buf(),
        record: false,
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
    for b in &cert.batteries {
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
