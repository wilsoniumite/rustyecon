//! The two checks kernel.md says Phase 4 must make rather than assume
//! (v2 Phase 4, P4.5; docs/architecture/kernel.md, "Price formation").
//!
//! These are falsification tests before they are anything else. A guard that
//! cannot fail is not a guard, and this project has already shipped one — the
//! legacy `Rule::Damping` — so each battery here is shown firing on a defect
//! that is independently known to exist before it is trusted to say a run is
//! clean.
//!
//! The defect B6 fires on is the one v2 Phase 3 cost a phase to find: the legacy
//! layer posts `min(stock, 1.2 × demand)` and buys against `state.demand`, so
//! its postings are functions of other agents' orders. Phase 3 found that by
//! investigation. B6 finds it by construction, every run.

use std::path::PathBuf;

use rustyecon::{
    certify::Certificate,
    kernel::AgentArm,
    runner::{RunConfig, SimRunner},
    scenario::loader,
};

fn certify(scenario: &str, ticks: u64, agents: AgentArm) -> Certificate {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/scenarios")
        .join(scenario);
    let s = loader::load(&dir).expect("scenario loads");
    let out = tempfile::tempdir().unwrap();
    let config = RunConfig {
        agents,
        price_rule: rustyecon::systems::clearing::PriceRule::Imbalance,
        ticks,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: out.path().to_path_buf(),
        record: false,
        telemetry_every: 1,
        certify: true,
        results_dir: out.path().join("results"),
    };
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&dir)
        .with_criteria(s.criteria);
    runner.run().expect("certify mode yields a certificate")
}

fn battery<'a>(cert: &'a Certificate, id: &str) -> &'a rustyecon::certify::certificate::Battery {
    cert.batteries.iter().find(|b| b.id == id).unwrap_or_else(|| panic!("no {id}"))
}

#[test]
fn b6_fires_on_the_legacy_layer_which_is_known_to_read_foreign_demand() {
    // `building_agent.rs` caps posted supply at `demand * 1.2` and sizes its
    // capacity target from `state.supply` and `state.demand`. Whether that is a
    // defect is settled — engine.md, "The 225-year horizon" — so a battery that
    // did NOT fire here would be measuring nothing.
    let cert = certify("lr_00", 200, AgentArm::Legacy);
    let b6 = battery(&cert, "B6");
    assert!(!b6.pass, "B6 must catch the legacy layer: {}", b6.detail);
    assert!(
        b6.detail.contains("violation"),
        "and must say what moved, not merely that something did: {}",
        b6.detail
    );
}

#[test]
fn b6_passes_under_the_kernel_because_every_posting_names_only_own_state() {
    // Rule 1 posts from stock and scale, Rule 3 buys from stock, scale and
    // price. `last_fill` is read, but only by σ, to steer the scale — never to
    // size an offer. This is the assertion kernel.md asks for.
    let cert = certify("lr_00", 200, AgentArm::Kernel);
    let b6 = battery(&cert, "B6");
    assert!(b6.pass, "B6 must pass under the kernel: {}", b6.detail);
}

#[test]
fn b7_catches_the_exact_pin_phase_3_measured_by_hand() {
    // The number this reproduces: engine.md, "The 225-year horizon" records the
    // legacy sell cap holding imbalance at exactly -1/6 across ~10,100
    // consecutive ticks, found by investigation after a whole corpus of
    // certificates had passed over it. B7 must find the same thing by itself.
    //
    // It did not, at first. Whole-window constancy was the original test, and
    // the pin holds only *while the cap binds* — a long unbroken stretch inside
    // a series that also does other things — so "was it constant throughout"
    // answered no and the run passed. This is why the test is a run-length one.
    let cert = certify("tracer_2r", 11_700, AgentArm::Legacy);
    let b7 = battery(&cert, "B7");
    assert!(!b7.pass, "B7 must catch the tracer's pin: {}", b7.detail);
    assert!(
        b7.detail.contains("-0.166667"),
        "and must report the pinned VALUE, which is what identified the cap as \
         the cause — (d - 1.2d)/1.2d = -1/6 exactly, independent of price: {}",
        b7.detail
    );
    assert!(b7.detail.contains("unbroken"), "as a stretch, not a whole window: {}", b7.detail);
}

#[test]
fn b7_reports_a_number_rather_than_a_reassurance() {
    // Every battery in this engine carries the measurement behind it (P4.0a):
    // a PASS that says only "PASS" cannot be audited later.
    let cert = certify("lr_00", 400, AgentArm::Legacy);
    let b7 = battery(&cert, "B7");
    assert!(
        b7.detail.contains("imbalance") || b7.detail.contains("pinned"),
        "B7 must name the measurement: {}",
        b7.detail
    );
}

#[test]
fn both_new_batteries_are_present_and_the_verdict_accounts_for_them() {
    let cert = certify("lr_00", 200, AgentArm::Legacy);
    let ids: Vec<&str> = cert.batteries.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, ["B1", "B2", "B3", "B4", "B5", "B6", "B7"]);
    // Fail-closed: a failing battery must reach the verdict, or the certificate
    // reports a defect and then calls the run clean anyway.
    assert!(!cert.passed(), "B6 fails on legacy, so the run cannot PASS");
}

#[test]
fn the_arm_is_recorded_so_two_certificates_cannot_be_confused() {
    // The A/B runs one scenario twice under identical tapes. Without this the
    // two certificates differ in no field that says which is which.
    let legacy = certify("lr_00", 60, AgentArm::Legacy);
    let kernel = certify("lr_00", 60, AgentArm::Kernel);
    assert_eq!(legacy.identity.agents, "legacy+imbalance");
    assert_eq!(kernel.identity.agents, "kernel+imbalance");
    assert_eq!(legacy.identity.tape_sha, kernel.identity.tape_sha, "same tape");
    assert_ne!(legacy.identity.run, kernel.identity.run, "different runs");
}
