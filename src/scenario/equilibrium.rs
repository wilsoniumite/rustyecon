//! The registered closed-form equilibrium of a solvable scenario.
//!
//! A scenario's `equilibrium.ron` says where the prices are *supposed to go*.
//! It is read by `tests/test_12_solvable.rs`; see
//! `docs/design/solvable-scenarios.md` for the derivations and
//! `data/scenarios/solv_1g/equilibrium.ron` for a fully commented example.
//!
//! # Why this is a separate file from `criteria.ron`
//!
//! [`Rule`](crate::certify::criteria::Rule) cannot express "distance from a
//! target vector". Every variant it has reads one series and asks whether that
//! series *stayed put*: `LevelRange` measures a band, `ResidualDamping` measures
//! whether a wobble grew, `FracBelow` measures a floor. None of them can be
//! handed a number the series is supposed to equal, and none of them can be
//! handed a *vector*, which is what a chain's answer is. Bolting a target onto
//! that schema would have made one file answer two questions, and the whole
//! point of this family is that stability and correctness are different
//! questions with different answers — `solv_labour` passes every stability class
//! at a real wage of 2.0, which is the wrong answer for that world.
//!
//! # Why it is not wired into the run certificate
//!
//! B8 already reports the realised-versus-implied price distance on every
//! certified run, deliberately with **no threshold** (see
//! [`PriceGapWatch::report`](crate::certify::technology::PriceGapWatch::report)):
//! the corpus's numbers were known before B8 was written, so any bar picked then
//! would have been fitted to a result. These scenarios do have pre-registered
//! tolerances — but they were derived from the kernel's own dead-bands, which
//! only exists because these worlds have hand-derived fixed points, and no such
//! derivation is available for `lr_*` or `tracer_2r`. Turning that into a
//! certificate battery would apply a bar to scenarios it was never derived for.
//! So the target is registered as data, read by a test, and left out of the
//! certificate until a bar can be defended for the general case (R6).
//!
//! Nothing in the engine reads this module. That is intentional and is the
//! reason `Equilibrium` carries no behaviour beyond parsing: a target the engine
//! could see would be a target the engine could be tuned against.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The mode-B displacement a scenario registers: which genesis price to
/// multiply, by how much, and how close the run must then get.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Displacement {
    /// Good whose genesis price is scaled. Named, not indexed, so the file
    /// survives a reordering of `goods`.
    pub good: String,
    /// Multiplier applied to that good's genesis price.
    pub factor: f64,
    /// Allowed `|ln(realised/target)|` on the **median** over the analysis
    /// window. Derived from registered dials — never from a measurement — and
    /// the derivation belongs in the scenario's own `equilibrium.ron`.
    pub tol_log: f64,
}

/// One scenario's hand-derived competitive equilibrium.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equilibrium {
    /// Where the derivation lives, so a number can always be traced to an
    /// argument rather than to whoever typed it.
    pub derivation: String,
    /// ISO date this target was registered. Same discipline as `criteria.ron`:
    /// a target changed after seeing results needs a new dated registration,
    /// never an edit in place (R6).
    pub registered: String,
    /// The good every price below is expressed against.
    pub numeraire: String,
    /// `price(good) / price(numeraire)` at the equilibrium. The numeraire itself
    /// appears with value 1.0 so the file reads as a vector rather than as a
    /// vector with a hole in it.
    pub prices: HashMap<String, f64>,
    /// Units of each good changing hands per tick at the equilibrium.
    pub cleared: HashMap<String, f64>,
    /// `price(labour) / price(staple)`, the metric the certificate already
    /// computes. Registered separately from `prices` because it is the number
    /// the rest of the project talks in, not because it is independent of them.
    pub real_wage: f64,
    /// Equilibrium velocity, where the tape pins the price *level*.
    ///
    /// `None` is a finding, not an omission: with a single wealth tier the pop's
    /// cash rule is skipped, the excess-demand system is homogeneous of degree
    /// zero, and the equilibrium is a half-open *ray* of price levels. Velocity
    /// is proportional to the level at fixed money stock, so it has no single
    /// equilibrium value in such a tape and registering one would be registering
    /// the end of a ray as if it were a point.
    pub velocity: Option<f64>,
    /// Mode A tolerance: allowed `|ln(realised/target)|` at **every** tick when
    /// genesis already is the equilibrium.
    ///
    /// This is float slack, not economic slack. These tapes are written entirely
    /// in dyadic rationals so the fixed point is bit-exact in f64 and every
    /// rule's response at it is identically zero; any observed motion is a
    /// defect in the engine or in the derivation.
    pub fixed_point_tol_log: f64,
    /// Mode B, when the scenario registers one. `None` for control tapes whose
    /// only job is to be compared against another run from the same fixed point.
    pub displaced: Option<Displacement>,
}

impl Equilibrium {
    /// Read `equilibrium.ron` from a scenario directory.
    ///
    /// Absence is `Ok(None)` — most scenarios have no known right answer, and
    /// that is the normal case rather than an error. A file that exists but
    /// fails to parse IS an error: a malformed target must never read as
    /// "unscored", which is the same fail-closed reasoning the criteria loader
    /// uses.
    pub fn load(dir: &Path) -> crate::scenario::loader::Result<Option<Self>> {
        let path = dir.join("equilibrium.ron");
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let eq: Equilibrium =
            ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        eq.validate()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Some(eq))
    }

    /// Reject a target that cannot mean anything, at load time rather than in
    /// the middle of a comparison.
    fn validate(&self) -> Result<(), String> {
        match self.prices.get(&self.numeraire) {
            Some(v) if (*v - 1.0).abs() < 1e-12 => {}
            Some(v) => {
                return Err(format!(
                    "numeraire {:?} must have relative price 1.0, got {v}",
                    self.numeraire
                ))
            }
            None => {
                return Err(format!(
                    "numeraire {:?} is not listed in `prices`",
                    self.numeraire
                ))
            }
        }
        for (good, p) in &self.prices {
            if !(p.is_finite() && *p > 0.0) {
                return Err(format!("price target for {good:?} must be finite and positive, got {p}"));
            }
        }
        for (good, q) in &self.cleared {
            if !(q.is_finite() && *q > 0.0) {
                // Zero is rejected as well as negative: a market registered as
                // clearing nothing has no relative price to score, and the
                // honest way to say "this market does not trade here" is to
                // leave the good out of the file entirely.
                return Err(format!("cleared target for {good:?} must be finite and positive, got {q}"));
            }
        }
        for (name, tol) in [
            ("fixed_point_tol_log", self.fixed_point_tol_log),
            (
                "displaced.tol_log",
                self.displaced.as_ref().map_or(1.0, |d| d.tol_log),
            ),
        ] {
            if !(tol.is_finite() && tol > 0.0) {
                return Err(format!("{name} must be finite and positive, got {tol}"));
            }
        }
        if let Some(d) = &self.displaced {
            if !(d.factor.is_finite() && d.factor > 0.0) {
                return Err(format!("displacement factor must be finite and positive, got {}", d.factor));
            }
            if !self.prices.contains_key(&d.good) {
                return Err(format!(
                    "displaced good {:?} has no registered price target, so mode B has nothing to converge to",
                    d.good
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every solvable tape's target must load and validate. This is the guard
    /// that a hand-edited `equilibrium.ron` cannot quietly become unreadable and
    /// take the correctness test's coverage with it — a test that silently
    /// scores nothing is worse than no test.
    #[test]
    fn every_registered_equilibrium_loads() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios");
        let mut found = 0;
        for entry in std::fs::read_dir(&root).expect("scenario corpus exists") {
            let dir = entry.expect("readable entry").path();
            if !dir.is_dir() || !dir.join("equilibrium.ron").exists() {
                continue;
            }
            let eq = Equilibrium::load(&dir)
                .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
                .expect("the file exists, so load must return Some");
            assert_eq!(eq.registered, "2026-07-31", "{}", dir.display());
            found += 1;
        }
        assert!(found >= 5, "expected the solv_* family, found {found} equilibrium files");
    }

    /// Falsification: the validator must actually reject the mistakes it claims
    /// to. A validator that cannot fail is decoration, so each branch is shown
    /// firing on a defect rather than assumed to work.
    #[test]
    fn the_validator_rejects_targets_that_cannot_mean_anything() {
        let base = |body: &str| -> Result<Equilibrium, String> {
            let eq: Equilibrium = ron::from_str(body).map_err(|e| e.to_string())?;
            eq.validate()?;
            Ok(eq)
        };
        // The control: a well-formed target parses and validates.
        let ok = base(
            r#"(derivation: "d", registered: "2026-07-31", numeraire: "labour",
                prices: {"labour": 1.0, "grain": 0.5}, cleared: {"grain": 20.0},
                real_wage: 2.0, velocity: None, fixed_point_tol_log: 1e-12,
                displaced: Some((good: "grain", factor: 2.0, tol_log: 0.095)))"#,
        );
        assert!(ok.is_ok(), "{ok:?}");

        // A numeraire priced against itself at anything but 1 is a units error,
        // and it would silently rescale the whole vector under comparison.
        let e = base(
            r#"(derivation: "d", registered: "2026-07-31", numeraire: "labour",
                prices: {"labour": 2.0}, cleared: {}, real_wage: 2.0, velocity: None,
                fixed_point_tol_log: 1e-12, displaced: None)"#,
        )
        .unwrap_err();
        assert!(e.contains("relative price 1.0"), "{e}");

        // A displacement pointed at a good with no target converges to nothing.
        let e = base(
            r#"(derivation: "d", registered: "2026-07-31", numeraire: "labour",
                prices: {"labour": 1.0}, cleared: {}, real_wage: 2.0, velocity: None,
                fixed_point_tol_log: 1e-12,
                displaced: Some((good: "gold", factor: 2.0, tol_log: 0.095)))"#,
        )
        .unwrap_err();
        assert!(e.contains("mode B has nothing to converge to"), "{e}");

        // A non-positive tolerance is a gate nothing can pass, which would look
        // like a very strict test and behave like a broken one.
        let e = base(
            r#"(derivation: "d", registered: "2026-07-31", numeraire: "labour",
                prices: {"labour": 1.0}, cleared: {}, real_wage: 2.0, velocity: None,
                fixed_point_tol_log: 0.0, displaced: None)"#,
        )
        .unwrap_err();
        assert!(e.contains("fixed_point_tol_log"), "{e}");
    }
}
