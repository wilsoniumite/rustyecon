//! Scoring metric series against a scenario's registered criteria.
//!
//! The statistics are ported from `notebooks/07_stability_suite.py`, including
//! its fail-closed NaN handling: a detector that cannot be computed **fails**
//! rather than being skipped (engine.md, "NaN = FAIL"). The Python original used
//! `x is np.nan`, an identity test that never fired for computed NaNs, so broken
//! metrics passed silently; that was fixed in Phase 0 and the corrected semantics
//! are what is ported here.

use serde::{Deserialize, Serialize};

use crate::certify::criteria::{Criteria, FailureClass, Metric, NanPolicy, Rule, Window};
use crate::certify::metrics::RegionSeries;

/// One region's stability outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionVerdict {
    pub region: String,
    /// Rendered issues, e.g. `UNSTABLE(velocity CV=0.34)`. Empty means clean.
    pub issues: Vec<String>,
    /// False when any *fatal* class fired; warnings do not fail a region.
    pub passed: bool,
}

/// The scenario's stability outcome, scored against its dated criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityReport {
    pub criteria_date: String,
    pub criteria_version: u32,
    /// The window actually scored, which may be shorter than the run.
    pub window: [u64; 2],
    pub regions: Vec<RegionVerdict>,
    pub regions_passed: usize,
    pub regions_total: usize,
}

impl StabilityReport {
    pub fn passed(&self) -> bool {
        self.regions_passed == self.regions_total
    }

    pub fn summary(&self) -> String {
        let issues: usize = self.regions.iter().map(|r| r.issues.len()).sum();
        format!(
            "{}/{} regions pass over ticks {}..{} ({} issue(s), criteria {} v{})",
            self.regions_passed,
            self.regions_total,
            self.window[0],
            self.window[1],
            issues,
            self.criteria_date,
            self.criteria_version
        )
    }
}

// ── statistics ────────────────────────────────────────────────────────────────

fn finite(values: &[f64]) -> Vec<f64> {
    values.iter().copied().filter(|v| v.is_finite()).collect()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Sample standard deviation (ddof=1), matching pandas' `Series.std()`.
fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return f64::NAN;
    }
    let m = mean(values);
    let var = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    var.sqrt()
}

/// Fraction of finite values below `thresh`. NaN when there is nothing to judge.
pub fn frac_below(values: &[f64], thresh: f64) -> f64 {
    let s = finite(values);
    if s.is_empty() {
        return f64::NAN;
    }
    s.iter().filter(|v| **v < thresh).count() as f64 / s.len() as f64
}

/// Mean rolling coefficient of variation: `mean(rolling_std / |rolling_mean|)`.
/// NaN when there is less than two windows' worth of data.
pub fn rolling_cv(values: &[f64], window: usize) -> f64 {
    let s = finite(values);
    if window == 0 || s.len() < window * 2 {
        return f64::NAN;
    }
    let mut ratios = Vec::new();
    for w in s.windows(window) {
        let m = mean(w).abs();
        let sd = std_dev(w);
        if m > 0.0 && sd.is_finite() {
            ratios.push(sd / m);
        }
    }
    if ratios.is_empty() {
        return f64::NAN;
    }
    mean(&ratios)
}

/// A half flat enough that its spread carries no information — small enough that
/// dividing by it is meaningless, not merely small.
const FLAT: f64 = 1e-12;

/// `std(second half) / std(first half)` — below 1 means the series is settling.
///
/// NaN when either half is too short to measure.
///
/// The flat-first-half case is judged on the *second* half. A series flat
/// throughout is genuinely settled and still scores 0.0; a series that is flat
/// and then erupts is the exact opposite of settling, and this used to score it
/// 0.0 as well — returning early without ever looking at `s2`, so no amount of
/// later movement could fire SWINGING. Scored as infinite now, failing against
/// any finite `max_ratio`.
///
/// What this deliberately does *not* do is fail a world whose series never move
/// at all. A constant series is not swinging, and this class saying so is
/// correct. A run that is uninformative because nothing in it ever happens is a
/// scenario problem, not a detector problem, and is caught where it belongs —
/// see `tests/test_08_long_horizon.rs`, which asserts the tracer's prices are
/// still moving at the horizon.
pub fn damping_ratio(values: &[f64], split: usize) -> f64 {
    let first = finite(&values[..split.min(values.len())]);
    let second = finite(&values[split.min(values.len())..]);
    if first.len() <= 5 || second.len() <= 5 {
        return f64::NAN;
    }
    let (s1, s2) = (std_dev(&first), std_dev(&second));
    if !s1.is_finite() || !s2.is_finite() {
        return f64::NAN;
    }
    if s1 < FLAT {
        return if s2 < FLAT { 0.0 } else { f64::INFINITY };
    }
    s2 / s1
}

/// Ordinary least squares slope against sample index.
pub fn linear_slope(values: &[f64]) -> f64 {
    let s = finite(values);
    let n = s.len();
    if n < 3 {
        return 0.0;
    }
    let xs: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let (mx, my) = (mean(&xs), mean(&s));
    let num: f64 = xs.iter().zip(&s).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

// ── evaluation ────────────────────────────────────────────────────────────────

/// The slice of a series a class reads, given the criteria's windows.
///
/// Series are indexed from the first recorded tick, which is tick 1.
fn slice<'a>(series: &'a [f64], criteria: &Criteria, window: Window) -> &'a [f64] {
    let to_idx = |tick: u64| tick.saturating_sub(1) as usize;
    let lo = to_idx(criteria.transient);
    // A run can end before the analysis window even opens — scoring it then has
    // nothing to read, which callers treat as missing data rather than a pass.
    if lo >= series.len() {
        return &[];
    }
    let hi = to_idx(criteria.analysis_end).min(series.len() - 1);
    if hi < lo {
        return &[];
    }
    match window {
        Window::Analysis | Window::Halves => &series[lo..=hi],
        Window::PostTransient => &series[lo..],
    }
}

/// Apply one class to one metric, returning the issue text if it fires.
fn check(
    criteria: &Criteria,
    class: &FailureClass,
    metric: Metric,
    values: &[f64],
) -> Option<String> {
    let label = format!("{metric:?}").to_lowercase();
    let fail_closed = matches!(criteria.policy_for(class), NanPolicy::FailClosed);
    // A NaN statistic means the detector could not be computed. Under the
    // registered policy that is a failure, not a skip.
    let uncomputable = |name: &str| {
        fail_closed.then(|| format!("{}({label} {name}=NaN)", class.name))
    };

    match class.rule {
        Rule::FracBelow { thresh, frac } => {
            let f = frac_below(values, thresh);
            if f.is_nan() {
                return uncomputable("frac");
            }
            (f > frac).then(|| format!("{}({label} frac={f:.2})", class.name))
        }
        Rule::RollingCv { window, max_cv } => {
            let cv = rolling_cv(values, window as usize);
            if cv.is_nan() {
                return uncomputable("CV");
            }
            (cv > max_cv).then(|| format!("{}({label} CV={cv:.2})", class.name))
        }
        Rule::Damping { max_ratio } => {
            let mid = criteria.analysis_mid().saturating_sub(criteria.transient) as usize;
            let dr = damping_ratio(values, mid);
            if dr.is_nan() {
                return uncomputable("damp");
            }
            (dr > max_ratio).then(|| format!("{}({label} damp={dr:.2})", class.name))
        }
        Rule::MeanAndSlope { mean_min, slope_min } => {
            let s = finite(values);
            if s.is_empty() {
                return uncomputable("mean");
            }
            let m = mean(&s);
            let half = s.len() / 2;
            let slope = linear_slope(&s[half..]);
            // Both conditions: a high level that is still trending up.
            (m > mean_min && slope > slope_min)
                .then(|| format!("{}({label} mean={m:.2}, slope={slope:.4})", class.name))
        }
    }
}

/// Score one region against the criteria.
pub fn evaluate_region(criteria: &Criteria, series: &RegionSeries) -> RegionVerdict {
    let mut issues = Vec::new();
    let mut fatal = false;

    for class in &criteria.classes {
        let mut fired = false;
        for &metric in &class.metrics {
            let values = slice(series.get(metric), criteria, class.window);
            if values.is_empty() {
                // Nothing recorded at all is a missing metric, and under the
                // fail-closed policy that is an issue rather than a pass.
                if matches!(criteria.policy_for(class), NanPolicy::FailClosed) {
                    issues.push(format!("{}({:?} no-data)", class.name, metric));
                    fired = true;
                    fatal |= class.fatal;
                }
                continue;
            }
            if let Some(issue) = check(criteria, class, metric, values) {
                issues.push(issue);
                fired = true;
                fatal |= class.fatal;
            }
        }
        // DEAD short-circuits: once a region is dead nothing else is meaningful.
        if fired && class.short_circuit {
            break;
        }
    }

    RegionVerdict {
        region: series.name.clone(),
        issues,
        passed: !fatal,
    }
}

/// Score every region and summarise.
pub fn evaluate(criteria: &Criteria, all: &[RegionSeries]) -> StabilityReport {
    let regions: Vec<RegionVerdict> = all
        .iter()
        .map(|s| evaluate_region(criteria, s))
        .collect();
    let passed = regions.iter().filter(|r| r.passed).count();
    StabilityReport {
        criteria_date: criteria.date.clone(),
        criteria_version: criteria.version,
        window: [criteria.transient, criteria.analysis_end],
        regions_total: regions.len(),
        regions_passed: passed,
        regions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frac_below_ignores_nan_and_reports_share() {
        let v = [1.0, f64::NAN, 0.0, 0.0];
        assert!((frac_below(&v, 0.5) - 2.0 / 3.0).abs() < 1e-12);
        assert!(frac_below(&[f64::NAN, f64::NAN], 1.0).is_nan());
    }

    #[test]
    fn rolling_cv_is_zero_for_a_constant_series() {
        let v = vec![5.0; 40];
        assert!(rolling_cv(&v, 10).abs() < 1e-12);
    }

    #[test]
    fn rolling_cv_needs_two_windows() {
        assert!(rolling_cv(&[1.0, 2.0, 3.0], 10).is_nan());
    }

    #[test]
    fn damping_ratio_settles_and_swings() {
        // Second half quieter than the first -> ratio below 1.
        let mut v: Vec<f64> = (0..20).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        v.extend(std::iter::repeat(0.0).take(20));
        assert!(damping_ratio(&v, 20) < 1.0);

        // Constant throughout is genuinely settled, not uncomputable.
        let flat: Vec<f64> = vec![1.0; 40];
        assert_eq!(damping_ratio(&flat, 20), 0.0);

        // Too little data to judge -> NaN, which callers fail closed on.
        assert!(damping_ratio(&[1.0, 2.0, 3.0, 4.0], 2).is_nan());
    }

    #[test]
    fn a_quiet_opening_cannot_buy_a_pass_for_a_loud_ending() {
        // The regression this exists for: a flat first half used to return 0.0
        // without the second half being looked at, so SWINGING could not fire
        // however violently the series later moved.
        let mut v: Vec<f64> = vec![1.0; 20];
        v.extend((0..20).map(|i| if i % 2 == 0 { 5.0 } else { -5.0 }));
        let dr = damping_ratio(&v, 20);
        assert!(dr.is_infinite(), "flat then oscillating must not score as settled, got {dr}");
        assert!(!dr.is_nan(), "this is computable and failing, not uncomputable");
        assert!(dr > 0.72, "must fire against the registered SWINGING threshold");
    }

    #[test]
    fn a_run_shorter_than_the_transient_scores_as_no_data() {
        use crate::certify::criteria::*;
        let criteria = Criteria {
            version: 1,
            date: "2026-07-20".into(),
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
        };
        // 60 ticks recorded, window opens at 150: nothing to read, and it must
        // report that rather than panicking or silently passing.
        let series = crate::certify::metrics::RegionSeries {
            region: crate::types::ids::RegionId(0),
            name: "Nowhere".into(),
            velocity: vec![1.0; 60],
            employment: vec![1.0; 60],
            real_wage: vec![1.0; 60],
            concentration: vec![0.0; 60],
            bld_util: vec![1.0; 60],
            real_income: vec![1.0; 60],
            building_ids: Vec::new(),
            building_util: Vec::new(),
        };
        let v = evaluate_region(&criteria, &series);
        assert!(!v.passed, "fail-closed on missing data");
        assert!(v.issues.iter().any(|i| i.contains("no-data")), "{:?}", v.issues);
    }

    #[test]
    fn linear_slope_detects_a_rising_trend() {
        let v: Vec<f64> = (0..10).map(|i| i as f64 * 2.0).collect();
        assert!((linear_slope(&v) - 2.0).abs() < 1e-9);
        assert!(linear_slope(&[1.0, 1.0, 1.0]).abs() < 1e-12);
    }
}
