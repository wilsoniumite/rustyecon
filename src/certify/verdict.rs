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

/// One statistic that was computed, recorded whether or not it fired.
///
/// A verdict is a threshold applied to a number, and only the number can be
/// compared across two runs. Phase 3.5 left the corpus at 0/72 regions, so a
/// pass count can no longer tell an improved kernel from an unchanged one — an
/// A/B on a count is satisfied by an arm that also scores zero. These are the
/// continuous readings that comparison needs (PLAN Phase 4).
///
/// Recorded for **every** class × metric pair, including after a `short_circuit`
/// class has fired. The short-circuit governs the *verdict* — nothing about a
/// dead region is meaningful — but suppressing the measurement too would make
/// the two arms' tables differently shaped exactly where they differ most, and
/// a mean over differently-composed sets compares nothing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Failure class this statistic belongs to, e.g. `DRIFTING`.
    pub class: String,
    /// Metric it was computed on, lowercased, e.g. `realwage`.
    pub metric: String,
    /// Short name of the statistic itself, e.g. `range`, `osc`, `frac`.
    pub stat: String,
    /// Rendered rather than numeric: JSON has no NaN and no Infinity, and
    /// serde_json flattens both to `null`, which would erase the difference
    /// between "uncomputable" and "unboundedly bad". Both round-trip through
    /// Rust's `str::parse::<f64>` and Python's `float()`.
    pub value: String,
    /// Whether the rule tripped on this reading. Not the same as appearing in
    /// [`RegionVerdict::issues`]: a reading taken after a short-circuit trips
    /// here but does not count toward the verdict.
    pub tripped: bool,
    /// Whether this reading counted toward the region's verdict.
    pub counted: bool,
}

impl Measurement {
    /// The reading as a number, with the non-finite cases preserved.
    pub fn as_f64(&self) -> f64 {
        self.value.parse().unwrap_or(f64::NAN)
    }
}

/// One region's stability outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionVerdict {
    pub region: String,
    /// Rendered issues, e.g. `UNSTABLE(velocity CV=0.34)`. Empty means clean.
    pub issues: Vec<String>,
    /// False when any *fatal* class fired; warnings do not fail a region.
    pub passed: bool,
    /// Every statistic computed for this region. Reported so a verdict can be
    /// audited and so two runs can be compared on more than pass/fail.
    #[serde(default)]
    pub measurements: Vec<Measurement>,
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

/// A detrended half flat enough that its spread carries no information.
///
/// The residuals this guards are logarithmic, so the number is a *relative*
/// deviation: a series constant to one part in a billion is constant for any
/// economic purpose. [`FLAT`] cannot serve here — it is an absolute bound on a
/// raw level, and means something different at price 1e3 than at price 1e-9.
const FLAT_REL: f64 = 1e-9;

/// OLS fit of ln(value) against sample index over strictly-positive finite
/// samples, returning the slope and the residuals paired with their index.
///
/// Non-positive samples are dropped rather than clamped: a metric at exactly
/// zero has no logarithm, and inventing one would put a fabricated point into
/// the fit. Dropping them can leave too little to judge, which the callers turn
/// into NaN and the registered policy fails closed on.
fn log_fit(values: &[f64]) -> Option<(f64, Vec<(usize, f64)>)> {
    let pts: Vec<(usize, f64)> = values
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_finite() && **v > 0.0)
        .map(|(i, v)| (i, v.ln()))
        .collect();
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|(i, _)| *i as f64).sum::<f64>() / n;
    let my = pts.iter().map(|(_, y)| *y).sum::<f64>() / n;
    let den: f64 = pts.iter().map(|(i, _)| (*i as f64 - mx).powi(2)).sum();
    if den <= 0.0 {
        return None;
    }
    let slope: f64 = pts.iter().map(|(i, y)| (*i as f64 - mx) * (y - my)).sum::<f64>() / den;
    if !slope.is_finite() {
        return None;
    }
    let resid = pts
        .iter()
        .map(|(i, y)| (*i, y - (my + slope * (*i as f64 - mx))))
        .collect();
    Some((slope, resid))
}

/// Linear-interpolated percentile of an already-sorted slice, matching numpy's
/// default so the Rust and Python scorers agree.
fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let pos = p / 100.0 * (n - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

/// How wide a band the level occupied: the ratio of its 95th to its 5th
/// percentile over the window.
///
/// This asks the question the class actually cares about — *did the metric stay
/// put* — and it is deliberately blind to the *shape* of any excursion, because
/// shape is not what makes a level unstable. The percentiles rather than the
/// extremes keep one freak sample from deciding a verdict.
///
/// It replaces a trend-fit (OLS slope of ln × window span), which measured
/// something else while claiming this. That statistic answers "what monotone
/// trend best fits", so it scored a metric that collapsed a millionfold and came
/// back as perfectly stable — the fit through a V is flat — and it inflated a
/// one-off level shift by `exp(Δ/2)`, tripping a "2×" bar at a true shift of
/// 1.59×. Both were measured on the shipped corpus, not hypothesised.
pub fn level_range(values: &[f64]) -> f64 {
    let mut live: Vec<f64> = values.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    if live.len() < 3 {
        return f64::NAN;
    }
    live.sort_by(|a, b| a.partial_cmp(b).expect("filtered to finite"));
    let (lo, hi) = (percentile_sorted(&live, 5.0), percentile_sorted(&live, 95.0));
    if lo <= 0.0 {
        return f64::INFINITY;
    }
    let r = hi / lo;
    if !r.is_finite() {
        f64::INFINITY
    } else {
        r
    }
}

/// Interquartile range — a spread measure that one outlier cannot move.
fn iqr(values: &[f64]) -> f64 {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).expect("caller filtered to finite"));
    percentile_sorted(&v, 75.0) - percentile_sorted(&v, 25.0)
}

/// Spread of the detrended relative residual in the second half over the first.
///
/// Above 1 the oscillation is growing. Flat throughout scores 0.0; flat and then
/// moving scores infinite, for the reason given on [`damping_ratio`].
///
/// Spread is the interquartile range, not the standard deviation. With σ a
/// single sample out of 851 could decide the class — one tick's 2× dip moved
/// this ratio from 0.998 to 1.499, and on the shipped corpus removing one sample
/// took `lr_01`/Leeds from 2.18 (firing) to 0.73 (clean). The IQR moves by 0.03%
/// on the same perturbation. A stability verdict that one tick can flip is not
/// measuring stability.
pub fn residual_damping(values: &[f64], split: usize) -> f64 {
    let Some((_, resid)) = log_fit(values) else {
        return f64::NAN;
    };
    let first: Vec<f64> = resid.iter().filter(|(i, _)| *i < split).map(|(_, r)| *r).collect();
    let second: Vec<f64> = resid.iter().filter(|(i, _)| *i >= split).map(|(_, r)| *r).collect();
    if first.len() <= 5 || second.len() <= 5 {
        return f64::NAN;
    }
    let (s1, s2) = (iqr(&first), iqr(&second));
    if !s1.is_finite() || !s2.is_finite() {
        return f64::NAN;
    }
    if s1 < FLAT_REL {
        return if s2 < FLAT_REL { 0.0 } else { f64::INFINITY };
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

/// One named statistic and its value, as computed by a rule.
struct Reading {
    stat: &'static str,
    value: f64,
}

/// Apply one class to one metric.
///
/// Returns the issue text if it fires, and the reading(s) behind that decision
/// either way — a rule that passes is still evidence, and the number is what
/// survives comparison between two runs.
fn check(
    criteria: &Criteria,
    class: &FailureClass,
    metric: Metric,
    values: &[f64],
) -> (Option<String>, Vec<Reading>) {
    let label = format!("{metric:?}").to_lowercase();
    let fail_closed = matches!(criteria.policy_for(class), NanPolicy::FailClosed);
    // A NaN statistic means the detector could not be computed. Under the
    // registered policy that is a failure, not a skip.
    let uncomputable = |name: &str| {
        fail_closed.then(|| format!("{}({label} {name}=NaN)", class.name))
    };
    let one = |stat, value| vec![Reading { stat, value }];

    match class.rule {
        Rule::FracBelow { thresh, frac } => {
            let f = frac_below(values, thresh);
            let r = one("frac", f);
            if f.is_nan() {
                return (uncomputable("frac"), r);
            }
            ((f > frac).then(|| format!("{}({label} frac={f:.2})", class.name)), r)
        }
        Rule::RollingCv { window, max_cv } => {
            let cv = rolling_cv(values, window as usize);
            let r = one("cv", cv);
            if cv.is_nan() {
                return (uncomputable("CV"), r);
            }
            ((cv > max_cv).then(|| format!("{}({label} CV={cv:.2})", class.name)), r)
        }
        Rule::Damping { max_ratio } => {
            let mid = criteria.analysis_mid().saturating_sub(criteria.transient) as usize;
            let dr = damping_ratio(values, mid);
            let r = one("damp", dr);
            if dr.is_nan() {
                return (uncomputable("damp"), r);
            }
            ((dr > max_ratio).then(|| format!("{}({label} damp={dr:.2})", class.name)), r)
        }
        Rule::LevelRange { max_ratio } => {
            let v = level_range(values);
            let r = one("range", v);
            if v.is_nan() {
                return (uncomputable("range"), r);
            }
            ((v > max_ratio).then(|| format!("{}({label} range={v:.2}x)", class.name)), r)
        }
        Rule::ResidualDamping { max_ratio } => {
            let mid = criteria.analysis_mid().saturating_sub(criteria.transient) as usize;
            let v = residual_damping(values, mid);
            let r = one("osc", v);
            if v.is_nan() {
                return (uncomputable("osc"), r);
            }
            ((v > max_ratio).then(|| format!("{}({label} osc={v:.2})", class.name)), r)
        }
        Rule::MeanAndSlope { mean_min, slope_min } => {
            let s = finite(values);
            if s.is_empty() {
                return (uncomputable("mean"), one("mean", f64::NAN));
            }
            let m = mean(&s);
            let half = s.len() / 2;
            let slope = linear_slope(&s[half..]);
            // Both conditions: a high level that is still trending up. Both are
            // reported, because either one alone explains nothing.
            let r = vec![
                Reading { stat: "mean", value: m },
                Reading { stat: "slope", value: slope },
            ];
            let issue = (m > mean_min && slope > slope_min)
                .then(|| format!("{}({label} mean={m:.2}, slope={slope:.4})", class.name));
            (issue, r)
        }
    }
}

/// Score one region against the criteria.
///
/// Two passes over the same classes, not one: the verdict stops at the first
/// `short_circuit` hit — once a region is dead, nothing else about it is
/// meaningful — while the measurement table keeps going to the end. Scoring
/// stops; measuring does not. `scoring` is the flag that separates them, so the
/// verdict logic below is the same logic it has always been.
pub fn evaluate_region(criteria: &Criteria, series: &RegionSeries) -> RegionVerdict {
    let mut issues = Vec::new();
    let mut measurements = Vec::new();
    let mut fatal = false;
    let mut scoring = true;

    for class in &criteria.classes {
        let mut fired = false;
        for &metric in &class.metrics {
            let label = format!("{metric:?}").to_lowercase();
            let values = slice(series.get(metric), criteria, class.window);
            if values.is_empty() {
                // Nothing recorded at all is a missing metric, and under the
                // fail-closed policy that is an issue rather than a pass.
                let fail_closed = matches!(criteria.policy_for(class), NanPolicy::FailClosed);
                measurements.push(Measurement {
                    class: class.name.clone(),
                    metric: label,
                    stat: "no-data".into(),
                    value: render(f64::NAN),
                    tripped: fail_closed,
                    counted: scoring && fail_closed,
                });
                if scoring && fail_closed {
                    issues.push(format!("{}({:?} no-data)", class.name, metric));
                    fired = true;
                    fatal |= class.fatal;
                }
                continue;
            }
            let (issue, readings) = check(criteria, class, metric, values);
            let tripped = issue.is_some();
            for r in readings {
                measurements.push(Measurement {
                    class: class.name.clone(),
                    metric: label.clone(),
                    stat: r.stat.into(),
                    value: render(r.value),
                    tripped,
                    counted: scoring && tripped,
                });
            }
            if scoring {
                if let Some(issue) = issue {
                    issues.push(issue);
                    fired = true;
                    fatal |= class.fatal;
                }
            }
        }
        // DEAD short-circuits: once a region is dead nothing else is meaningful.
        if scoring && fired && class.short_circuit {
            scoring = false;
        }
    }

    RegionVerdict {
        region: series.name.clone(),
        issues,
        passed: !fatal,
        measurements,
    }
}

/// Render a statistic so JSON can carry it.
///
/// JSON has no NaN and no Infinity, and serde_json writes both as `null` — which
/// would erase the difference between a detector that could not be computed and
/// one that returned an unbounded reading. Both are failures, but they are not
/// the same failure, and the A/B ranks them differently.
fn render(v: f64) -> String {
    format!("{v:e}")
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

    /// Criteria with a short-circuiting DEAD in front of a LevelRange class, so
    /// the interaction between "stop scoring" and "keep measuring" is exercised.
    fn two_class_criteria() -> Criteria {
        use crate::certify::criteria::*;
        Criteria {
            version: 4,
            date: "2026-07-31".into(),
            transient: 0,
            analysis_end: 60,
            staple_good: "flour".into(),
            labour_good: "labour".into(),
            nan_policy: NanPolicy::FailClosed,
            classes: vec![
                FailureClass {
                    name: "DEAD".into(),
                    fatal: true,
                    metrics: vec![Metric::Velocity],
                    rule: Rule::FracBelow { thresh: 0.003, frac: 0.80 },
                    window: Window::Analysis,
                    short_circuit: true,
                    nan_policy: None,
                },
                FailureClass {
                    name: "DRIFTING".into(),
                    fatal: true,
                    metrics: vec![Metric::RealWage],
                    rule: Rule::LevelRange { max_ratio: 2.2 },
                    window: Window::Analysis,
                    short_circuit: false,
                    nan_policy: None,
                },
            ],
        }
    }

    fn series_with(velocity: Vec<f64>, real_wage: Vec<f64>) -> crate::certify::metrics::RegionSeries {
        let n = velocity.len();
        crate::certify::metrics::RegionSeries {
            region: crate::types::ids::RegionId(0),
            name: "Somewhere".into(),
            velocity,
            employment: vec![1.0; n],
            real_wage,
            concentration: vec![0.0; n],
            bld_util: vec![1.0; n],
            real_income: vec![1.0; n],
            building_ids: Vec::new(),
            building_util: Vec::new(),
        }
    }

    #[test]
    fn a_dead_region_still_reports_the_band_it_never_got_scored_on() {
        // The whole point of the measurement table. A dead region short-circuits,
        // so DRIFTING never counts against it — but the band is exactly what the
        // A/B compares, and a table that stopped here would be missing precisely
        // the regions the two arms differ on.
        let dead = series_with(vec![0.0; 60], (0..60).map(|i| 1.0 + i as f64).collect());
        let v = evaluate_region(&two_class_criteria(), &dead);

        assert!(!v.passed);
        assert_eq!(v.issues.len(), 1, "only DEAD counts: {:?}", v.issues);
        assert!(v.issues[0].starts_with("DEAD"));

        let band = v.measurements.iter().find(|m| m.stat == "range").expect("band measured");
        assert!(band.as_f64() > 2.2, "the wage really did drift: {}", band.value);
        assert!(band.tripped, "the rule tripped");
        assert!(!band.counted, "but it did not count — DEAD short-circuited");
    }

    #[test]
    fn a_live_region_counts_what_it_trips() {
        let live = series_with(vec![1.0; 60], (0..60).map(|i| 1.0 + i as f64).collect());
        let v = evaluate_region(&two_class_criteria(), &live);
        assert!(!v.passed);
        assert_eq!(v.issues.len(), 1, "{:?}", v.issues);
        assert!(v.issues[0].starts_with("DRIFTING"));
        let band = v.measurements.iter().find(|m| m.stat == "range").unwrap();
        assert!(band.tripped && band.counted);
    }

    #[test]
    fn non_finite_readings_survive_the_json_round_trip() {
        // serde_json writes NaN and Infinity as `null`, which would make an
        // uncomputable detector and an unbounded one indistinguishable in the
        // persisted certificate. Both are failures; they are not the same one.
        for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, 2.2, -1.5e-9] {
            let text = render(v);
            let back: f64 = text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"));
            assert_eq!(back.is_nan(), v.is_nan(), "{text}");
            if !v.is_nan() {
                assert_eq!(back, v, "{text}");
            }
        }
        assert_ne!(render(f64::INFINITY), render(f64::NAN));
    }

    #[test]
    fn linear_slope_detects_a_rising_trend() {
        let v: Vec<f64> = (0..10).map(|i| i as f64 * 2.0).collect();
        assert!((linear_slope(&v) - 2.0).abs() < 1e-9);
        assert!(linear_slope(&[1.0, 1.0, 1.0]).abs() < 1e-12);
    }
}
