//! Parameters, their validation, and the user-cost factor (spec §2 and §3.0).

use std::fmt;

use crate::schedule::{PowerSchedule, Schedule, CURVATURE_CEIL, VALIDATION_SAMPLES};

/// The work-cost distribution F: χ uniform on [0, χ_max] (SSRN p.30).
///
/// A worker works exactly when χ ≤ ln(1 + v/P_s) (SSRN eq 8-9, p.11), so labour
/// supply is n_S = N·F(ln(1 + v/P_s)) (SSRN eq 10 and 24).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformWorkCost {
    /// χ_max, the top of the support. In [[`SCALE_FLOOR`], [`SCALE_CEIL`]].
    pub chi_max: f64,
}

impl UniformWorkCost {
    /// F(z) = z/χ_max, clamped to [0, 1]. The clamp caps participation at N.
    pub fn cdf(&self, z: f64) -> f64 {
        (z / self.chi_max).clamp(0.0, 1.0)
    }
}

/// The parameters of unit 1a (spec §2). Build an [`Economy`] from them to validate.
///
/// "Scale" below means finite and in [[`SCALE_FLOOR`], [`SCALE_CEIL`]].
///
/// In the flow benchmark (ρ, δ, J_b) = (0, 1, 1), the recipe (a, λ, b) is per machine
/// service. With durability it is per machine built, and each machine yields one
/// service per period.
#[derive(Clone, Debug, PartialEq)]
pub struct Params<S = PowerSchedule> {
    /// N, potential workers. Each supplies at most one hour per period. Scale.
    pub workers: f64,
    /// T, the land-service endowment per period. Scale.
    pub land: f64,
    /// h, space per basket. A basket is one unit of the good and h units of space. Scale.
    pub space: f64,
    /// a, machine services used per machine built. 0 ≤ a < 1.
    pub a: f64,
    /// λ, hours per machine built. 0 ≤ λ ≤ [`SCALE_CEIL`].
    pub lam: f64,
    /// b, land services per machine built. Scale, so b > 0.
    pub b: f64,
    /// γ(x), relative human productivity. Positive and strictly increasing on [0, 1].
    pub schedule: S,
    /// F, the distribution of the work cost χ.
    pub work_cost: UniformWorkCost,
    /// ρ, the required return per period. An input, never solved for.
    /// 0 ≤ ρ ≤ [`SCALE_CEIL`].
    pub rho: f64,
    /// δ, geometric depreciation per period, applied after installation.
    /// [`SCALE_FLOOR`] ≤ δ ≤ 1.
    pub delta: f64,
    /// J_b, periods from the start of a build to its first service. J_b ≥ 1.
    pub build_lag: u32,
}

/// Why a parameter set was rejected.
#[derive(Clone, Debug, PartialEq)]
pub enum ParamError {
    /// A parameter is NaN or infinite.
    NotFinite {
        /// The parameter's name.
        name: &'static str,
        /// Its value.
        value: f64,
    },
    /// A parameter is finite but outside its range (SSRN p.28, and the scale bounds).
    OutOfRange {
        /// The parameter's name.
        name: &'static str,
        /// Its value.
        value: f64,
        /// The condition it fails.
        requirement: Requirement,
    },
    /// The build lag J_b is 0. It must be at least 1.
    BuildLag {
        /// The lag given.
        value: u32,
    },
    /// The user-cost factor u overflowed. This needs an absurdly long build lag.
    UserCostNotFinite {
        /// The value computed for u.
        u: f64,
    },
    /// A parameter of one item of a list failed: a category of unit 1b
    /// ([`CategoryParams`](crate::CategoryParams)) or a cell of the price block
    /// ([`cell_cost`](crate::cell_cost)).
    Item {
        /// What the list holds: `"category"` or `"cell"`.
        kind: &'static str,
        /// The item's position in its list, from 0.
        index: usize,
        /// Which of the item's parameters failed, and why.
        error: Box<ParamError>,
    },
    /// A condition on the shape or the combination of parameters failed: unit 1b's task
    /// line, its categories and its basket (docs/unit-1b.md §3.2).
    Invalid {
        /// What failed: `"edges"`, `"density"`, `"categories"`, `"category"`, `"basket"` or
        /// `"J(0)"`.
        name: &'static str,
        /// The condition, in words.
        reason: &'static str,
    },
}

impl ParamError {
    /// The name of the offending parameter (`"u"` for [`ParamError::UserCostNotFinite`];
    /// for [`ParamError::Item`], the item's parameter).
    pub fn name(&self) -> &'static str {
        match self {
            ParamError::NotFinite { name, .. }
            | ParamError::OutOfRange { name, .. }
            | ParamError::Invalid { name, .. } => name,
            ParamError::BuildLag { .. } => "build_lag",
            ParamError::UserCostNotFinite { .. } => "u",
            ParamError::Item { error, .. } => error.name(),
        }
    }
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::NotFinite { name, value } => write!(f, "{name} = {value:?} is not finite"),
            ParamError::OutOfRange {
                name,
                value,
                requirement,
            } => write!(f, "{name} = {value:?} is out of range: {requirement}"),
            ParamError::BuildLag { value } => {
                write!(f, "build_lag = {value} is out of range: must be >= 1")
            }
            ParamError::UserCostNotFinite { u } => {
                write!(
                    f,
                    "user cost u = {u:?} is not finite: the build lag is too long"
                )
            }
            ParamError::Item { kind, index, error } => write!(f, "{kind} {index}: {error}"),
            ParamError::Invalid { name, reason } => write!(f, "{name}: {reason}"),
        }
    }
}

impl std::error::Error for ParamError {}

/// The condition a rejected parameter fails. Its `Display` states the condition with the
/// current values of [`SCALE_FLOOR`] and [`SCALE_CEIL`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Requirement {
    /// [`SCALE_FLOOR`] ≤ value ≤ [`SCALE_CEIL`]: N, T, h, b, χ_max and a schedule's
    /// scale parameters.
    Scale,
    /// 0 ≤ value ≤ [`SCALE_CEIL`]: λ and ρ.
    NonNegative,
    /// 0 ≤ value < 1: a.
    MachineShare,
    /// [`SCALE_FLOOR`] ≤ value ≤ 1: δ.
    Depreciation,
    /// value = 0 or [`SCALE_FLOOR`] ≤ value ≤ [`SCALE_CEIL`]: unit 1b's basket weights,
    /// direct land and task densities, and a cell's weight and machine productivity.
    ZeroOrScale,
    /// 0 < value < 1: the CES weight α of [`ces_share`](crate::ces_share).
    OpenUnit,
    /// [`SCALE_FLOOR`] ≤ value ≤ [`CURVATURE_CEIL`]: the [`PowerSchedule`]'s k.
    Curvature,
    /// value > 0: the inputs of [`closure`](crate::closure).
    Positive,
    /// γ strictly increases at the points x = i/[`VALIDATION_SAMPLES`], as the default
    /// [`Schedule::validate`] checks it. Decided by sampling, so [`Requirement::admits`]
    /// is false for it.
    SampledIncrease,
    /// A condition that a [`Schedule`](crate::Schedule) states in words.
    Schedule(&'static str),
}

impl Requirement {
    /// Whether a finite value meets the condition. [`Requirement::SampledIncrease`] and
    /// [`Requirement::Schedule`] are decided by the schedule, so this is false for them.
    pub fn admits(&self, value: f64) -> bool {
        match self {
            Requirement::Scale => (SCALE_FLOOR..=SCALE_CEIL).contains(&value),
            Requirement::NonNegative => (0.0..=SCALE_CEIL).contains(&value),
            Requirement::MachineShare => (0.0..1.0).contains(&value),
            Requirement::Depreciation => (SCALE_FLOOR..=1.0).contains(&value),
            Requirement::ZeroOrScale => value == 0.0 || (SCALE_FLOOR..=SCALE_CEIL).contains(&value),
            Requirement::OpenUnit => value > 0.0 && value < 1.0,
            Requirement::Curvature => (SCALE_FLOOR..=CURVATURE_CEIL).contains(&value),
            Requirement::Positive => value > 0.0,
            Requirement::SampledIncrease | Requirement::Schedule(_) => false,
        }
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Requirement::Scale => write!(
                f,
                "must be in [SCALE_FLOOR, SCALE_CEIL] = [{SCALE_FLOOR:e}, {SCALE_CEIL:e}]"
            ),
            Requirement::NonNegative => {
                write!(f, "must be in [0, SCALE_CEIL] = [0, {SCALE_CEIL:e}]")
            }
            Requirement::MachineShare => write!(f, "must satisfy 0 <= a < 1"),
            Requirement::Depreciation => {
                write!(f, "must be in [SCALE_FLOOR, 1] = [{SCALE_FLOOR:e}, 1]")
            }
            Requirement::ZeroOrScale => write!(
                f,
                "must be 0 or in [SCALE_FLOOR, SCALE_CEIL] = [{SCALE_FLOOR:e}, {SCALE_CEIL:e}]"
            ),
            Requirement::OpenUnit => write!(f, "must satisfy 0 < value < 1"),
            Requirement::Curvature => write!(
                f,
                "must be in [SCALE_FLOOR, CURVATURE_CEIL] = [{SCALE_FLOOR:e}, {CURVATURE_CEIL}]"
            ),
            Requirement::Positive => write!(f, "must be > 0"),
            Requirement::SampledIncrease => write!(
                f,
                "gamma must strictly increase on [0, 1] (sampled at x = i/{VALIDATION_SAMPLES})"
            ),
            Requirement::Schedule(text) => write!(f, "{text}"),
        }
    }
}

pub(crate) fn finite(name: &'static str, value: f64) -> Result<f64, ParamError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ParamError::NotFinite { name, value })
    }
}

/// A finite value that meets `requirement`, returned with a negative zero made +0.0.
///
/// −0.0 passes every check that admits 0, and it would otherwise travel into the outputs
/// (ρ = −0.0 gives interest = −0.0): the same economy would print differently. Adding
/// +0.0 turns −0.0 into +0.0 and leaves every other value alone.
fn check(name: &'static str, value: f64, requirement: Requirement) -> Result<f64, ParamError> {
    let value = finite(name, value)?;
    if requirement.admits(value) {
        Ok(value + 0.0)
    } else {
        Err(ParamError::OutOfRange {
            name,
            value,
            requirement,
        })
    }
}

/// A finite value > 0.
pub(crate) fn positive(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::Positive)
}

/// The smallest value [`Economy::new`] accepts for a parameter that must be positive:
/// N, T, h, b, χ_max and δ, and the schedule's scale parameters (η, g0, g1 and k for
/// [`PowerSchedule`], γ(0) for any [`Schedule`]). k's ceiling is its own,
/// [`CURVATURE_CEIL`].
///
/// Below it the solver's comparisons can be decided by underflow. At δ = 5e-324 (the
/// smallest subnormal), n_D(1) = λδK and n_S(1) both round to 0, so f(1) = 0 reads as
/// `BoundaryNoMargin` although the exact f(1) is negative. [`SCALE_CEIL`] bounds the same
/// parameters, and λ and ρ, from above. The quantities the regime tests compare are
/// products and quotients of up to about six parameters, so with every parameter in
/// [1e-30, 1e30] they stay within about [1e-180, 1e180], far from underflow (below
/// 2.2e-308) and overflow. No meaningful instance comes near either bound: prices are in
/// units of r = 1, and the paper's scale parameters lie between 0.05 and 10.
///
/// The bounds do not bound u or D. A long build lag with a large ρ can make u huge and
/// overflow a price, and D(1) can be arbitrarily close to 0; [`Economy::solve`] then
/// returns [`SolveError::NonFinite`](crate::SolveError::NonFinite) rather than a regime.
pub const SCALE_FLOOR: f64 = 1e-30;

/// The largest value [`Economy::new`] accepts for N, T, h, b, χ_max, λ, ρ and the
/// schedule's scale parameters (η, g0 and g1, and γ(0) for any [`Schedule`]; the
/// [`PowerSchedule`]'s k has the lower [`CURVATURE_CEIL`]). See [`SCALE_FLOOR`].
///
/// Without it, huge parameters push quotients into underflow: N = h = 1e300 with
/// T = b = 1e-30 makes T/h = 1e-330 and v/P_s about 1.5e-330, so n_D(1) and n_S(1) round
/// to 0 and f(1) = 0 reads as `BoundaryNoMargin`, although the exact f(1) is about
/// −1.5e-30 and the economy is `NoInteriorAtZero` (review of 2026-09-25).
pub const SCALE_CEIL: f64 = 1e30;

/// A finite parameter in [[`SCALE_FLOOR`], [`SCALE_CEIL`]].
pub(crate) fn scale(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::Scale)
}

/// A finite curvature in [[`SCALE_FLOOR`], [`CURVATURE_CEIL`]].
pub(crate) fn curvature(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::Curvature)
}

/// A finite parameter in [0, [`SCALE_CEIL`]], with −0.0 made +0.0.
pub(crate) fn nonnegative(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::NonNegative)
}

/// 0 or a finite parameter in [[`SCALE_FLOOR`], [`SCALE_CEIL`]], with −0.0 made +0.0.
pub(crate) fn zero_or_scale(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::ZeroOrScale)
}

/// [`SCALE_FLOOR`] ≤ δ ≤ 1.
pub(crate) fn depreciation(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::Depreciation)
}

/// 0 ≤ a < 1: machine services must not use a whole machine service each (SSRN p.28).
/// −0.0 is made +0.0.
pub(crate) fn machine_share(name: &'static str, value: f64) -> Result<f64, ParamError> {
    check(name, value, Requirement::MachineShare)
}

/// The scalar user-cost factor u = (ρ + δ)(1 + ρ)^(J_b − 1) (spec §3.0).
///
/// It is derived from free entry in laborformal `dynamics/checks/check_dynamics.py`
/// (u_K at :45, U3 at :47-64, at 31b3482): the build cost is paid at the start of
/// period t, the first service comes in period t + J_b undepreciated, and survival is
/// (1 − δ) per service period after that. Its corners:
///
/// - (ρ, δ, J_b) = (0, 1, 1) gives u = 1, the SSRN Appendix B flow benchmark;
/// - J_b = 1 gives ρ + δ, the SSRN A.4 durable asset (p.28);
/// - J_b = 1 and δ = 1 give 1 + ρ, inputs advanced one period (SSRN A.4, p.27);
/// - J_b = 0 and δ = 1 give 1 (check_dynamics U4b). J_b = 0 in general gives
///   (ρ + δ)/(1 + ρ); [`Economy::new`] rejects J_b = 0, so only this function reaches it.
///
/// The power is taken by repeated squaring in this crate's own code, so u is the same
/// double on every platform.
///
/// **Accuracy.** 1 + ρ is rounded once and then raised to J_b − 1, and each squaring
/// doubles the relative error it inherits, so the error in u grows linearly with the
/// lag: at most about 2·J_b·2^-53 ≈ J_b·2.2e-16 relative, that is between J_b and 2·J_b
/// ulps. Measured against an 80-digit u (2026-09-25 review): at most 4.4 ulps for
/// J_b ≤ 4, 26 for J_b ≤ 20, 80 for J_b ≤ 60 and 298 for J_b ≤ 200. The bound reaches
/// the gate's 1e-12 relative at J_b ≈ 4500 (the review measured 3.4e-8 at
/// J_b = 2^32 − 1), and V_m = b/D amplifies it by u(a + λγ)/D near the viability edge
/// D → 0. A later unit that needs long lags there should compute u in double-double
/// arithmetic, which keeps the platform independence.
pub fn user_cost(rho: f64, delta: f64, build_lag: u32) -> f64 {
    match build_lag {
        0 => (rho + delta) / (1.0 + rho),
        lag => (rho + delta) * geometric(1.0 + rho, lag - 1).0,
    }
}

/// ω = (1 + ρ)^(J_b − 1) + δ·Σ_{i < J_b − 1} (1 + ρ)^i, machine wealth per unit of
/// installed build cost: W_K = ω·V_m·K (check_dynamics.py L2, :164-169 at 31b3482).
///
/// In exact arithmetic u − δ = ρ·ω, so interest (u − δ)·V_m·K is ρ·W_K. Computing it
/// as ρ·ω avoids the cancellation in u − δ when ρ is small against δ. J_b ≥ 1.
pub(crate) fn wealth_factor(rho: f64, delta: f64, build_lag: u32) -> f64 {
    let (carry, series) = geometric(1.0 + rho, build_lag.saturating_sub(1));
    carry + delta * series
}

/// (q^n, Σ_{i < n} q^i) by repeated squaring. Both are built from positive terms, so
/// neither suffers cancellation.
fn geometric(q: f64, mut n: u32) -> (f64, f64) {
    // The result so far covers a run of terms; `block` covers the next 2^j of them.
    let (mut pow, mut sum) = (1.0, 0.0);
    let (mut block_pow, mut block_sum) = (q, 1.0);
    while n > 0 {
        if n & 1 == 1 {
            sum += pow * block_sum;
            pow *= block_pow;
        }
        n >>= 1;
        if n > 0 {
            block_sum += block_pow * block_sum;
            block_pow *= block_pow;
        }
    }
    (pow, sum)
}

/// A validated economy: parameters that passed every check in [`Economy::new`], and
/// their user-cost factor u.
#[derive(Clone, Debug, PartialEq)]
pub struct Economy<S = PowerSchedule> {
    params: Params<S>,
    u: f64,
}

impl<S: Schedule> Economy<S> {
    /// Validates the parameters against the paper's assumptions (SSRN p.28 and A.4)
    /// and computes u.
    ///
    /// Requires N, T, h, b and χ_max finite and in [[`SCALE_FLOOR`], [`SCALE_CEIL`]];
    /// 0 ≤ a < 1; λ and ρ in [0, [`SCALE_CEIL`]]; [`SCALE_FLOOR`] ≤ δ ≤ 1; J_b ≥ 1; the
    /// schedule's own [`Schedule::validate`] (for [`PowerSchedule`], k ≤
    /// [`CURVATURE_CEIL`]); and a finite u. Viability,
    /// D(1) = 1 − u(a + λγ(1)) > 0, is not checked here: an economy that fails it is valid
    /// and solves to [`Regime::NotViable`](crate::Regime::NotViable).
    ///
    /// A negative zero in a, λ or ρ is stored as +0.0, so that one economy has one
    /// representation and prints the same whichever zero it was given.
    pub fn new(mut params: Params<S>) -> Result<Self, ParamError> {
        scale("workers", params.workers)?;
        scale("land", params.land)?;
        scale("space", params.space)?;
        params.a = machine_share("a", params.a)?;
        params.lam = nonnegative("lam", params.lam)?;
        scale("b", params.b)?;
        params.schedule.validate()?;
        scale("chi_max", params.work_cost.chi_max)?;
        params.rho = nonnegative("rho", params.rho)?;
        depreciation("delta", params.delta)?;
        if params.build_lag == 0 {
            return Err(ParamError::BuildLag {
                value: params.build_lag,
            });
        }
        let u = user_cost(params.rho, params.delta, params.build_lag);
        if !u.is_finite() {
            return Err(ParamError::UserCostNotFinite { u });
        }
        Ok(Economy { params, u })
    }

    /// The validated parameters.
    pub fn params(&self) -> &Params<S> {
        &self.params
    }

    /// u = (ρ + δ)(1 + ρ)^(J_b − 1).
    pub fn user_cost(&self) -> f64 {
        self.u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn appendix_b() -> Params {
        // SSRN p.30.
        Params {
            workers: 4.0,
            land: 10.0,
            space: 1.0,
            a: 0.3,
            lam: 0.05,
            b: 0.4,
            schedule: PowerSchedule {
                eta: 1.0,
                g0: 0.2,
                g1: 0.8,
                k: 1.0,
            },
            work_cost: UniformWorkCost { chi_max: 1.0 },
            rho: 0.0,
            delta: 1.0,
            build_lag: 1,
        }
    }

    #[test]
    fn appendix_b_is_valid_with_u_one() {
        let e = Economy::new(appendix_b()).unwrap();
        assert_eq!(e.user_cost(), 1.0);
        assert_eq!(e.params(), &appendix_b());
    }

    #[test]
    fn each_parameter_is_checked() {
        let base = appendix_b();
        let cases: [(Params, &str); 17] = [
            (
                Params {
                    workers: 0.0,
                    ..base.clone()
                },
                "workers",
            ),
            (
                Params {
                    workers: f64::NAN,
                    ..base.clone()
                },
                "workers",
            ),
            (
                Params {
                    land: -1.0,
                    ..base.clone()
                },
                "land",
            ),
            (
                Params {
                    space: 0.0,
                    ..base.clone()
                },
                "space",
            ),
            (
                Params {
                    a: 1.0,
                    ..base.clone()
                },
                "a",
            ),
            (
                Params {
                    a: -0.1,
                    ..base.clone()
                },
                "a",
            ),
            (
                Params {
                    lam: -0.01,
                    ..base.clone()
                },
                "lam",
            ),
            (
                Params {
                    lam: f64::INFINITY,
                    ..base.clone()
                },
                "lam",
            ),
            (
                Params {
                    b: 0.0,
                    ..base.clone()
                },
                "b",
            ),
            (
                Params {
                    schedule: PowerSchedule {
                        g1: -0.8,
                        ..base.schedule
                    },
                    ..base.clone()
                },
                "g1",
            ),
            (
                Params {
                    work_cost: UniformWorkCost { chi_max: 0.0 },
                    ..base.clone()
                },
                "chi_max",
            ),
            (
                Params {
                    rho: -0.01,
                    ..base.clone()
                },
                "rho",
            ),
            (
                Params {
                    delta: 0.0,
                    ..base.clone()
                },
                "delta",
            ),
            (
                Params {
                    delta: 1.5,
                    ..base.clone()
                },
                "delta",
            ),
            (
                Params {
                    delta: f64::NAN,
                    ..base.clone()
                },
                "delta",
            ),
            (
                Params {
                    build_lag: 0,
                    ..base.clone()
                },
                "build_lag",
            ),
            (
                Params {
                    rho: 1.0,
                    build_lag: 2000,
                    ..base.clone()
                },
                "u",
            ),
        ];
        for (params, name) in cases {
            let err = Economy::new(params.clone()).expect_err("invalid parameters accepted");
            assert_eq!(err.name(), name, "{params:?} gave {err}");
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn edges_that_are_valid() {
        let base = appendix_b();
        for params in [
            Params {
                a: 0.0,
                ..base.clone()
            },
            Params {
                lam: 0.0,
                ..base.clone()
            },
            Params {
                delta: 1.0,
                rho: 0.0,
                ..base.clone()
            },
            Params {
                build_lag: 40,
                rho: 0.05,
                delta: 0.1,
                ..base.clone()
            },
            // Not viable (D(1) = -0.1), but valid: solve() classifies it.
            Params {
                lam: 0.8,
                ..base.clone()
            },
        ] {
            assert!(Economy::new(params.clone()).is_ok(), "{params:?}");
        }
    }

    #[test]
    fn user_cost_corners() {
        // check_dynamics.py U4 (:65-68) and U4b at 31b3482.
        assert_eq!(user_cost(0.0, 1.0, 1), 1.0);
        for rho in [0.0, 0.01, 0.05, 0.12] {
            for delta in [0.03, 0.1, 0.5, 1.0] {
                assert_eq!(user_cost(rho, delta, 1), rho + delta);
            }
            assert_eq!(user_cost(rho, 1.0, 1), 1.0 + rho);
            assert_eq!(user_cost(rho, 1.0, 0), 1.0);
        }
        // (0.05, 0.1, 3): 0.15 · 1.05² = 0.165375.
        assert!((user_cost(0.05, 0.1, 3) / 0.165375 - 1.0).abs() < 1e-15);
        // At ρ = 0 the lag drops out: u = δ.
        for lag in [1, 2, 7, 100] {
            assert_eq!(user_cost(0.0, 0.1, lag), 0.1);
        }
        // J_b = 0 away from δ = 1: (ρ + δ)/(1 + ρ) = 0.15/1.05, one rounding each.
        let want: f64 = 0.15 / 1.05;
        let got = user_cost(0.05, 0.1, 0);
        assert!((got - want).abs() <= want * f64::EPSILON, "{got} vs {want}");
        assert!((got - 1.0 / 7.0).abs() < 1e-15);
    }

    #[test]
    fn scale_parameters_have_a_floor() {
        // δ = 5e-324 underflows λδK and n_S(1) to 0 (spec: SCALE_FLOOR's doc).
        let base = appendix_b();
        let tiny = SCALE_FLOOR / 2.0;
        let rejected = [
            (
                "delta",
                Params {
                    delta: 5e-324,
                    ..base.clone()
                },
            ),
            (
                "delta",
                Params {
                    delta: tiny,
                    ..base.clone()
                },
            ),
            (
                "workers",
                Params {
                    workers: tiny,
                    ..base.clone()
                },
            ),
            (
                "land",
                Params {
                    land: tiny,
                    ..base.clone()
                },
            ),
            (
                "space",
                Params {
                    space: tiny,
                    ..base.clone()
                },
            ),
            (
                "b",
                Params {
                    b: tiny,
                    ..base.clone()
                },
            ),
            (
                "chi_max",
                Params {
                    work_cost: UniformWorkCost { chi_max: tiny },
                    ..base.clone()
                },
            ),
            (
                "eta",
                Params {
                    schedule: PowerSchedule {
                        eta: 1e-300,
                        ..base.schedule
                    },
                    ..base.clone()
                },
            ),
            (
                "g0",
                Params {
                    schedule: PowerSchedule {
                        g0: tiny,
                        ..base.schedule
                    },
                    ..base.clone()
                },
            ),
            (
                "g1",
                Params {
                    schedule: PowerSchedule {
                        g1: tiny,
                        ..base.schedule
                    },
                    ..base.clone()
                },
            ),
            (
                "k",
                Params {
                    schedule: PowerSchedule {
                        k: tiny,
                        ..base.schedule
                    },
                    ..base.clone()
                },
            ),
        ];
        for (name, params) in rejected {
            let err = Economy::new(params.clone()).expect_err("below the floor accepted");
            assert_eq!(err.name(), name, "{params:?} gave {err}");
            assert!(err.to_string().contains("SCALE_FLOOR"), "{err}");
        }
        // The floor itself is accepted, and parameters that may be 0 have no floor.
        for params in [
            Params {
                delta: SCALE_FLOOR,
                ..base.clone()
            },
            Params {
                workers: SCALE_FLOOR,
                ..base.clone()
            },
            Params {
                rho: 5e-324,
                lam: 5e-324,
                a: 5e-324,
                ..base.clone()
            },
        ] {
            assert!(Economy::new(params.clone()).is_ok(), "{params:?}");
        }
    }

    #[test]
    fn scale_parameters_have_a_ceiling() {
        // N = h = 1e300 with T = b = 1e-30 underflows T/h and v/P_s, so f(1) reads 0 (the
        // review's case); the ceiling rejects it (SCALE_CEIL's doc).
        let base = appendix_b();
        let underflow = Params {
            workers: 1e300,
            land: 1e-30,
            space: 1e300,
            b: 1e-30,
            ..base.clone()
        };
        assert_eq!(Economy::new(underflow).unwrap_err().name(), "workers");
        let huge = SCALE_CEIL * 2.0;
        let schedule = |f: fn(&mut PowerSchedule)| {
            let mut s = base.schedule;
            f(&mut s);
            Params {
                schedule: s,
                ..base.clone()
            }
        };
        let rejected = [
            (
                "workers",
                Params {
                    workers: huge,
                    ..base.clone()
                },
            ),
            (
                "land",
                Params {
                    land: huge,
                    ..base.clone()
                },
            ),
            (
                "space",
                Params {
                    space: huge,
                    ..base.clone()
                },
            ),
            (
                "lam",
                Params {
                    lam: huge,
                    ..base.clone()
                },
            ),
            (
                "b",
                Params {
                    b: huge,
                    ..base.clone()
                },
            ),
            ("eta", schedule(|s| s.eta = 1e31)),
            ("g0", schedule(|s| s.g0 = 1e31)),
            ("g1", schedule(|s| s.g1 = 1e31)),
            (
                "chi_max",
                Params {
                    work_cost: UniformWorkCost { chi_max: huge },
                    ..base.clone()
                },
            ),
            (
                "rho",
                Params {
                    rho: huge,
                    ..base.clone()
                },
            ),
        ];
        for (name, params) in rejected {
            let err = Economy::new(params.clone()).expect_err("above the ceiling accepted");
            assert_eq!(err.name(), name, "{params:?} gave {err}");
            let text = err.to_string();
            assert!(
                text.contains("SCALE_CEIL") && text.contains("1e30"),
                "{text}"
            );
        }
        // The ceiling itself is accepted.
        for params in [
            Params {
                workers: SCALE_CEIL,
                lam: SCALE_CEIL,
                rho: SCALE_CEIL,
                ..base.clone()
            },
            schedule(|s| s.g1 = SCALE_CEIL),
        ] {
            assert!(Economy::new(params.clone()).is_ok(), "{params:?}");
        }
        // k has its own, lower ceiling (CURVATURE_CEIL's doc): k = 1e31 and k = SCALE_CEIL
        // are rejected with a message that names it, and k = CURVATURE_CEIL is accepted.
        for k in [1e31, SCALE_CEIL, CURVATURE_CEIL.next_up()] {
            let err = Economy::new(schedule_k(&base, k)).expect_err("steep k accepted");
            assert_eq!(err.name(), "k", "{err}");
            let text = err.to_string();
            assert!(
                text.contains("CURVATURE_CEIL") && text.contains("1024"),
                "{text}"
            );
        }
        assert!(Economy::new(schedule_k(&base, CURVATURE_CEIL)).is_ok());
    }

    fn schedule_k(base: &Params, k: f64) -> Params {
        Params {
            schedule: PowerSchedule { k, ..base.schedule },
            ..base.clone()
        }
    }

    #[test]
    fn messages_state_the_bounds_from_the_constants() {
        let base = appendix_b();
        let err = Economy::new(Params {
            b: 0.0,
            ..base.clone()
        })
        .unwrap_err();
        let floor = format!("{SCALE_FLOOR:e}");
        assert_eq!(
            err.to_string(),
            format!(
                "b = 0.0 is out of range: must be in [SCALE_FLOOR, SCALE_CEIL] = [{floor}, 1e30]"
            )
        );
        let err = Economy::new(Params {
            delta: 0.0,
            ..base.clone()
        })
        .unwrap_err();
        assert!(
            err.to_string().ends_with(&format!("= [{floor}, 1]")),
            "{err}"
        );
        // The build lag is an integer and prints as one.
        let err = Economy::new(Params {
            build_lag: 0,
            ..base.clone()
        })
        .unwrap_err();
        assert_eq!(err, ParamError::BuildLag { value: 0 });
        assert_eq!(
            err.to_string(),
            "build_lag = 0 is out of range: must be >= 1"
        );
    }

    #[test]
    fn negative_zero_is_stored_as_positive_zero() {
        // a, lambda and rho may be 0; -0.0 is the same economy and must print the same.
        let e = Economy::new(Params {
            a: -0.0,
            lam: -0.0,
            rho: -0.0,
            ..appendix_b()
        })
        .unwrap();
        for (name, value) in [
            ("a", e.params().a),
            ("lam", e.params().lam),
            ("rho", e.params().rho),
        ] {
            assert_eq!(value.to_bits(), 0.0f64.to_bits(), "{name}");
        }
        // Other values pass through unchanged, bit for bit.
        assert_eq!(Economy::new(appendix_b()).unwrap().params(), &appendix_b());
        assert_eq!(
            nonnegative("lam", 0.05).unwrap().to_bits(),
            0.05f64.to_bits()
        );
    }

    #[test]
    fn geometric_matches_the_naive_loop() {
        for q in [1.0, 1.05, 0.5, 1.123456789] {
            let (mut product, mut series) = (1.0, 0.0);
            for n in 0..40u32 {
                let (pow, sum) = geometric(q, n);
                assert!((pow - product).abs() <= 1e-14 * product, "{q}^{n}");
                assert!(
                    (sum - series).abs() <= 1e-14 * series,
                    "sum of {q}^i, i < {n}"
                );
                series += product;
                product *= q;
            }
        }
        assert_eq!(geometric(1.05, 2), (1.05 * 1.05, 1.0 + 1.05));
        assert_eq!(geometric(2.0, 10), (1024.0, 1023.0));
    }

    #[test]
    fn interest_factor_is_u_minus_delta_over_rho() {
        // u - delta = rho * omega (check_dynamics.py L1-L2).
        for (rho, delta, lag) in [
            (0.05, 1.0, 1),
            (0.05, 0.1, 3),
            (0.12, 0.03, 7),
            (0.01, 0.5, 40),
        ] {
            let omega = wealth_factor(rho, delta, lag);
            let u = user_cost(rho, delta, lag);
            assert!(
                (u - delta - rho * omega).abs() <= 1e-14 * u,
                "{rho} {delta} {lag}"
            );
        }
        assert_eq!(wealth_factor(0.07, 0.4, 1), 1.0);
    }

    #[test]
    fn work_cost_cdf_clamps() {
        let f = UniformWorkCost { chi_max: 2.0 };
        assert_eq!(f.cdf(-1.0), 0.0);
        assert_eq!(f.cdf(1.0), 0.5);
        assert_eq!(f.cdf(3.0), 1.0);
    }
}
