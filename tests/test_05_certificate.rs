//! Run certificate integration tests (v2 Phase 1, docs/architecture/engine.md).
//!
//! A result without a certificate does not exist (METHODOLOGY R5). These cover
//! the whole pipeline: run under `--certify`, get every battery, get a verdict,
//! and get an identity that names what was actually run.

use std::path::PathBuf;

use rustyecon::{
    runner::{RunConfig, SimRunner},
    scenario::loader,
};

fn certify(scenario: &str, ticks: u64) -> rustyecon::certify::Certificate {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/scenarios")
        .join(scenario);
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
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config).with_scenario_dir(&dir);
    runner.run().expect("certify mode yields a certificate")
}

#[test]
fn a_certified_run_reports_every_battery_and_a_verdict() {
    let cert = certify("lr_00", 120);

    let ids: Vec<&str> = cert.batteries.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, ["B1", "B2", "B3", "B4", "B5"], "all batteries present");
    assert!(cert.passed(), "lr_00 should certify clean: {}", cert.render());

    // Each battery carries its measurement, so a PASS is auditable rather than
    // merely reassuring.
    for b in &cert.batteries {
        assert!(!b.detail.is_empty(), "{} must report a measurement", b.id);
    }
    assert!(cert.render().starts_with("RUN CERTIFICATE"), "verdict-first");
}

#[test]
fn identity_names_the_run_and_is_reproducible() {
    let a = certify("lr_00", 60);
    let b = certify("lr_00", 60);

    assert_eq!(a.identity.scenario, "lr_00");
    assert_eq!(a.identity.ticks, 60);
    // Same tape, same code, same seed → same run id and same end state.
    assert_eq!(a.identity.run, b.identity.run, "run id must be reproducible");
    assert_eq!(a.identity.tape_sha, b.identity.tape_sha);
    let hash_of = |c: &rustyecon::certify::Certificate| {
        c.batteries.iter().find(|x| x.id == "B3").unwrap().detail.clone()
    };
    assert_eq!(hash_of(&a), hash_of(&b), "same inputs → same state hash");

    // A different span is a different run.
    let c = certify("lr_00", 61);
    assert_ne!(a.identity.run, c.identity.run);
}

#[test]
fn different_scenarios_have_different_tape_fingerprints() {
    let a = certify("lr_00", 30);
    let b = certify("lr_01", 30);
    assert_ne!(
        a.identity.tape_sha, b.identity.tape_sha,
        "the tape fingerprint must distinguish scenarios, or a stale verdict could be reused"
    );
}

#[test]
fn certificate_persists_as_json() {
    let cert = certify("lr_00", 30);
    let dir = tempfile::tempdir().unwrap();
    let path = cert.persist(dir.path()).expect("persists");
    assert!(path.ends_with("certificate.json"));

    let text = std::fs::read_to_string(&path).unwrap();
    let back: rustyecon::certify::Certificate = serde_json::from_str(&text).unwrap();
    assert_eq!(back.verdict, cert.verdict);
    assert_eq!(back.batteries.len(), 5);
}
