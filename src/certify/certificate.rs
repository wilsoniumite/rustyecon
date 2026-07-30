//! The run certificate (engine.md, "The run certificate"; METHODOLOGY R5).
//!
//! Every run ends verdict-first: the batteries print PASS/FAIL *before* any
//! result is read, the certificate JSON is persisted to `results/`, and a failed
//! battery exits nonzero. A result without a certificate does not exist.
//!
//! The batteries are engine correctness, not economics. Stability lives in the
//! scenario's dated `criteria.ron` ([`crate::certify::criteria`]) and is scored
//! separately: a run can be perfectly conserved, deterministic and NaN-free while
//! its economy collapses, and those are different claims.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// One pass/fail check, with the number behind it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battery {
    /// Stable short id: B1..B5.
    pub id: String,
    /// What the battery checks, in words.
    pub label: String,
    pub pass: bool,
    /// The measurement, so a PASS is auditable rather than merely reassuring.
    pub detail: String,
}

impl Battery {
    pub fn new(id: &str, label: &str, pass: bool, detail: String) -> Self {
        Self { id: id.into(), label: label.into(), pass, detail }
    }
}

/// What identifies a run: same tape, same code, same seed → same run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIdentity {
    /// Derived from the other fields, so it is reproducible rather than random.
    pub run: String,
    pub git: String,
    /// Fingerprint of the scenario inputs (game_data + starting_state + events).
    pub tape_sha: String,
    pub seed: u64,
    pub scenario: String,
    pub ticks: u64,
    /// Which agent layer decided (PLAN Phase 4). Defaulted for certificates
    /// written before the kernel arm existed — every one of those was legacy,
    /// so the default states a fact rather than guessing one.
    #[serde(default = "legacy_arm")]
    pub agents: String,
}

fn legacy_arm() -> String { "legacy".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    #[serde(flatten)]
    pub identity: RunIdentity,
    pub batteries: Vec<Battery>,
    /// Stability scored against the scenario's dated criteria.ron, when it has
    /// one. Absent means the scenario registered no criteria — which is itself
    /// reported, so an unscored run cannot be mistaken for a clean one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stability: Option<crate::certify::verdict::StabilityReport>,
    /// PASS only if every battery passed *and* stability passed.
    pub verdict: String,
}

impl Certificate {
    pub fn new(identity: RunIdentity, batteries: Vec<Battery>) -> Self {
        Self::with_stability(identity, batteries, None)
    }

    pub fn with_stability(
        identity: RunIdentity,
        batteries: Vec<Battery>,
        stability: Option<crate::certify::verdict::StabilityReport>,
    ) -> Self {
        let batteries_ok = batteries.iter().all(|b| b.pass);
        let stable = stability.as_ref().map_or(true, |s| s.passed());
        let verdict = if batteries_ok && stable { "PASS" } else { "FAIL" };
        Self { identity, batteries, stability, verdict: verdict.into() }
    }

    pub fn passed(&self) -> bool {
        self.verdict == "PASS"
    }

    /// Render verdict-first, in the shape engine.md specifies.
    pub fn render(&self) -> String {
        let mut out = format!(
            "RUN CERTIFICATE  run={}  git={}  tape_sha={}  seed={}\n",
            self.identity.run, self.identity.git, self.identity.tape_sha, self.identity.seed,
        );
        out.push_str(&format!(
            "  scenario={}  ticks={}  agents={}\n",
            self.identity.scenario, self.identity.ticks, self.identity.agents
        ));
        for b in &self.batteries {
            let line = format!("{} {}: {}", b.id, b.label, b.detail);
            out.push_str(&format!(
                "{:<62} {}\n",
                line,
                if b.pass { "PASS" } else { "FAIL" }
            ));
        }
        match &self.stability {
            Some(s) => {
                out.push_str(&format!(
                    "{:<62} {}\n",
                    format!("STABILITY: {}", s.summary()),
                    if s.passed() { "PASS" } else { "FAIL" }
                ));
                for r in s.regions.iter().filter(|r| !r.issues.is_empty()) {
                    out.push_str(&format!(
                        "  {:<12} {:<4} {}\n",
                        r.region,
                        if r.passed { "warn" } else { "FAIL" },
                        r.issues.join(", ")
                    ));
                }
            }
            None => out.push_str("STABILITY: no criteria.ron registered — unscored\n"),
        }
        out.push_str(&format!("VERDICT: {}\n", self.verdict));
        out
    }

    /// Persist to `dir/<scenario>/certificate.json`.
    ///
    /// A stable path, not a timestamped one: verdicts are committed to the repo
    /// (R5), so a re-run of the same scenario should update its certificate in
    /// place and show up as a reviewable diff.
    pub fn persist(&self, dir: &Path) -> std::io::Result<std::path::PathBuf> {
        let scenario_dir = dir.join(&self.identity.scenario);
        std::fs::create_dir_all(&scenario_dir)?;
        let path = scenario_dir.join("certificate.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json + "\n")?;
        Ok(path)
    }
}

/// Best-effort git revision of the working tree.
///
/// Reported, never trusted for correctness: a run whose git sha cannot be read
/// still certifies, it just says so.
pub fn git_sha() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

/// Fingerprint of a scenario directory's inputs.
///
/// Covers the three files that define the run — the tape proper plus the world
/// it acts on — so a changed scenario cannot silently reuse an old verdict.
pub fn tape_sha(dir: &Path) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for name in ["game_data.ron", "starting_state.ron", "events.ron"] {
        if let Ok(bytes) = std::fs::read(dir.join(name)) {
            for b in bytes {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    format!("{hash:016x}")
}

/// Deterministic run id: the same inputs must name the same run.
pub fn run_id(git: &str, tape: &str, scenario: &str, agents: &str, ticks: u64, seed: u64) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    // The arm is part of the identity: the A/B runs one scenario twice, and two
    // runs that differ only in who decided must not share a run id.
    for part in [git, tape, scenario, agents] {
        for b in part.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    for v in [ticks, seed] {
        for b in v.to_le_bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident() -> RunIdentity {
        RunIdentity {
            run: "abc".into(),
            git: "deadbeef".into(),
            tape_sha: "0123".into(),
            seed: 0,
            scenario: "lr_00".into(),
            ticks: 100,
            agents: "legacy".into(),
        }
    }

    #[test]
    fn verdict_is_pass_only_when_every_battery_passes() {
        let ok = Certificate::new(
            ident(),
            vec![Battery::new("B1", "conservation", true, "max drift 0".into())],
        );
        assert!(ok.passed());

        let bad = Certificate::new(
            ident(),
            vec![
                Battery::new("B1", "conservation", true, "max drift 0".into()),
                Battery::new("B4", "metrics computable", false, "3 NaN".into()),
            ],
        );
        assert!(!bad.passed());
        assert_eq!(bad.verdict, "FAIL");
    }

    #[test]
    fn renders_verdict_and_every_battery() {
        let c = Certificate::new(
            ident(),
            vec![
                Battery::new("B1", "conservation", true, "max drift 0.0e0".into()),
                Battery::new("B5", "clamp/shortfall events", false, "7".into()),
            ],
        );
        let text = c.render();
        assert!(text.starts_with("RUN CERTIFICATE"));
        assert!(text.contains("B1 conservation"));
        assert!(text.contains("PASS"));
        assert!(text.contains("B5 clamp/shortfall events"));
        assert!(text.contains("FAIL"));
        assert!(text.trim_end().ends_with("VERDICT: FAIL"));
    }

    #[test]
    fn run_id_is_reproducible_and_input_sensitive() {
        let a = run_id("git1", "tape1", "lr_00", "legacy", 100, 0);
        assert_eq!(a, run_id("git1", "tape1", "lr_00", "legacy", 100, 0));
        assert_ne!(a, run_id("git1", "tape1", "lr_00", "legacy", 101, 0));
        assert_ne!(a, run_id("git2", "tape1", "lr_00", "legacy", 100, 0));
        assert_ne!(a, run_id("git1", "tape1", "lr_01", "legacy", 100, 0));
        // The A/B's two arms run the same tape, so this is the only field that
        // separates their certificates.
        assert_ne!(a, run_id("git1", "tape1", "lr_00", "kernel", 100, 0));
    }

    #[test]
    fn round_trips_through_json() {
        let c = Certificate::new(
            ident(),
            vec![Battery::new("B1", "conservation", true, "max drift 0".into())],
        );
        let text = serde_json::to_string(&c).unwrap();
        let back: Certificate = serde_json::from_str(&text).unwrap();
        assert_eq!(back.verdict, "PASS");
        assert_eq!(back.identity.scenario, "lr_00");
        assert_eq!(back.batteries.len(), 1);
    }
}
