//! Pre-registered scenario criteria (engine.md, "The run certificate"; METHODOLOGY R6).
//!
//! Thresholds are data, not code: a scenario's pass/fail bar lives in a dated
//! `criteria.ron` beside its `game_data.ron`, so a criterion cannot be quietly
//! retuned after seeing results — changing one means a new dated file.
//!
//! The taxonomy is ported from `notebooks/07_stability_suite.py`, where these
//! thresholds were global Python constants applied to every scenario.

use serde::{Deserialize, Serialize};

/// One of the per-region time series the failure classes are scored against.
///
/// Each is computed in-engine per tick from `SimState`; see `certify::metrics`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Metric {
    /// Σ(cleared × price) over non-currency goods, divided by regional currency held.
    Velocity,
    /// Employed pop size over total pop size.
    Employment,
    /// price(labour) / price(the reference staple).
    RealWage,
    /// Buildings' share of regional currency.
    Concentration,
    /// Mean clamp(chosen_size / recipe_size, 0, 1) over non-channel buildings.
    BldUtil,
    /// Employed currency per capita, deflated by the staple price.
    RealIncome,
}

/// Which slice of the run a rule reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    /// `[transient, analysis_end]` — the standard scoring window.
    Analysis,
    /// Everything from `transient` onward.
    PostTransient,
    /// The analysis window split at its midpoint, for before/after comparisons.
    Halves,
}

/// The detector applied to a metric series, with its registered thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Rule {
    /// Fails when the fraction of ticks below `thresh` exceeds `frac`.
    FracBelow { thresh: f64, frac: f64 },
    /// Fails when the mean rolling coefficient of variation exceeds `max_cv`.
    RollingCv { window: u32, max_cv: f64 },
    /// Fails when std(second half) / std(first half) exceeds `max_ratio`.
    ///
    /// **Superseded by [`Rule::LogDrift`] + [`Rule::ResidualDamping`] (v2 Phase
    /// 3.5). Retained so criteria registered before 2026-07-20 still load and
    /// their certificates stay reproducible; do not use it in new files.**
    ///
    /// It was intended to ask "is the series settling", but it reads the raw
    /// *level*, and standard deviation is homogeneous of degree one — so a
    /// series whose level shrinks scores low no matter how it behaves. Measured
    /// over an 11,550-tick window: monotone geometric deflation scores 0.149 and
    /// passes, while a stationary economy with ordinary ripple scores 1.009 and
    /// fails. It rewards the disease and punishes the cure. The replacement
    /// splits the two claims it was conflating.
    Damping { max_ratio: f64 },
    /// Fails when the level drifts by more than `max_factor` across the window,
    /// in either direction — the runaway half of the old `Damping`.
    ///
    /// Measured as the OLS slope of ln(value) against sample index, converted to
    /// the total factor across the window and folded so that a fall of 3× and a
    /// rise of 3× both score 3.0. Working in logs makes geometric drift linear,
    /// which is the shape the engine's multiplicative price rule produces.
    LogDrift { max_factor: f64 },
    /// Fails when the oscillation *grows*: std(second half) / std(first half) of
    /// the **detrended** relative residual exceeds `max_ratio`.
    ///
    /// This is what `Damping` was reaching for. Removing the fitted log-trend
    /// first means a series is judged on how it moves about its own path rather
    /// than on where that path went, so a stationary economy scores ~1.0 whether
    /// its level is high or low. The threshold therefore sits *above* 1.0: a
    /// real economy in equilibrium fluctuates persistently, and demanding decay
    /// would fail every healthy world.
    ResidualDamping { max_ratio: f64 },
    /// Fails when the mean exceeds `mean_min` *and* the second-half slope
    /// exceeds `slope_min` — a high level that is still trending up.
    MeanAndSlope { mean_min: f64, slope_min: f64 },
}

/// What to do when a rule's statistic cannot be computed.
///
/// engine.md: **NaN = FAIL**. A value *declared missing* under a registered
/// convention is a separate thing and is not represented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NanPolicy {
    /// An uncomputable statistic fails the class (the default).
    #[default]
    FailClosed,
    /// An uncomputable statistic is skipped. Only legitimate where the metric is
    /// genuinely inapplicable to the scenario, and must be justified in the file.
    Skip,
}

/// A named failure class: a detector over one or more metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureClass {
    /// Reported verbatim in the certificate, e.g. "DEAD", "UNSTABLE".
    pub name: String,
    /// Fatal classes decide the region's verdict; non-fatal ones are warnings
    /// that are reported but do not fail the run.
    pub fatal: bool,
    /// Every metric this class is applied to, each scored independently.
    pub metrics: Vec<Metric>,
    pub rule: Rule,
    pub window: Window,
    /// When set, a hit stops evaluation of the remaining classes for that region
    /// (the 07-suite short-circuits after DEAD — nothing else is meaningful).
    #[serde(default)]
    pub short_circuit: bool,
    /// Overrides the file-level policy for this class.
    #[serde(default)]
    pub nan_policy: Option<NanPolicy>,
}

/// A dated, pre-registered criteria set for one scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criteria {
    /// Bumped whenever the shape of this file changes.
    pub version: u32,
    /// ISO date this criteria set was registered. Retuning a threshold after
    /// seeing results requires a new dated file, never an edit in place (R6).
    pub date: String,
    /// Ticks discarded as startup transient before scoring begins.
    pub transient: u64,
    /// Last tick included in the analysis window.
    pub analysis_end: u64,
    /// Good used as the deflator for RealWage / RealIncome, by name.
    pub staple_good: String,
    /// Good supplied by pops as labour, by name — the RealWage numerator.
    pub labour_good: String,
    #[serde(default)]
    pub nan_policy: NanPolicy,
    pub classes: Vec<FailureClass>,
}

impl Criteria {
    /// Midpoint of the analysis window — the split point for [`Window::Halves`].
    pub fn analysis_mid(&self) -> u64 {
        self.transient + (self.analysis_end - self.transient) / 2
    }

    /// True when `tick` falls inside the scoring window.
    pub fn in_analysis(&self, tick: u64) -> bool {
        tick >= self.transient && tick <= self.analysis_end
    }

    /// The policy in force for a class: its override, else the file default.
    pub fn policy_for(&self, class: &FailureClass) -> NanPolicy {
        class.nan_policy.unwrap_or(self.nan_policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Criteria {
        Criteria {
            version: 1,
            date: "2026-07-19".into(),
            transient: 150,
            analysis_end: 1000,
            staple_good: "flour".into(),
            labour_good: "labour".into(),
            nan_policy: NanPolicy::FailClosed,
            classes: vec![FailureClass {
                name: "DEAD".into(),
                fatal: true,
                metrics: vec![Metric::Velocity],
                rule: Rule::FracBelow { thresh: 0.003, frac: 0.80 },
                window: Window::Analysis,
                short_circuit: true,
                nan_policy: None,
            }],
        }
    }

    #[test]
    fn analysis_window_matches_the_ported_suite() {
        let c = sample();
        // 07-suite: TRANSIENT=150, ANALYSIS_END=1000 -> MID=575.
        assert_eq!(c.analysis_mid(), 575);
        assert!(!c.in_analysis(149));
        assert!(c.in_analysis(150));
        assert!(c.in_analysis(1000));
        assert!(!c.in_analysis(1001));
    }

    #[test]
    fn class_inherits_file_nan_policy_unless_overridden() {
        let mut c = sample();
        assert_eq!(c.policy_for(&c.classes[0]), NanPolicy::FailClosed);
        c.classes[0].nan_policy = Some(NanPolicy::Skip);
        assert_eq!(c.policy_for(&c.classes[0]), NanPolicy::Skip);
    }

    #[test]
    fn round_trips_through_ron() {
        let c = sample();
        let text = ron::ser::to_string(&c).unwrap();
        let back: Criteria = ron::from_str(&text).unwrap();
        assert_eq!(back.transient, c.transient);
        assert_eq!(back.classes.len(), 1);
        assert_eq!(back.classes[0].name, "DEAD");
        assert!(back.classes[0].short_circuit);
    }
}
