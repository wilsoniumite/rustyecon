//! The oracle: the static equilibrium of the pinning paper's economy.
//!
//! This is unit 1a (docs/unit-1a.md). It solves one category: one final good, one
//! machine type and one land input, with durability and interest entering through the
//! scalar user cost u = (ρ + δ)(1 + ρ)^(J_b − 1). At (ρ, δ, J_b) = (0, 1, 1) it is the
//! SSRN Appendix B economy (SSRN 7226858, pp.28-30), and it reproduces that appendix's
//! published numbers.
//!
//! Every price is in units of the land rent r = 1, so v = w/r.
//!
//! The oracle shares types but not logic with the agents. No agent may read it
//! (PLAN R13).
//!
//! ```
//! use oracle::{Economy, Params, PowerSchedule, Regime, UniformWorkCost};
//!
//! // The SSRN Appendix B instance (SSRN p.30).
//! let params = Params {
//!     workers: 4.0, land: 10.0, space: 1.0, a: 0.3, lam: 0.05, b: 0.4,
//!     schedule: PowerSchedule { eta: 1.0, g0: 0.2, g1: 0.8, k: 1.0 },
//!     work_cost: UniformWorkCost { chi_max: 1.0 },
//!     rho: 0.0, delta: 1.0, build_lag: 1,
//! };
//! let economy = Economy::new(params).expect("valid parameters");
//! match economy.solve().expect("finite") {
//!     Regime::Interior(eq) => assert!((eq.x_star - 0.86315).abs() < 5e-6),
//!     other => panic!("expected an interior equilibrium, got {}", other.name()),
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod closure;
pub mod dump;
mod params;
mod schedule;
mod solve;

pub use closure::{closure, Closure, ClosureError};
pub use params::{
    user_cost, Economy, ParamError, Params, Requirement, UniformWorkCost, SCALE_CEIL, SCALE_FLOOR,
};
pub use schedule::{PowerSchedule, Schedule, CURVATURE_CEIL, VALIDATION_SAMPLES};
pub use solve::{
    CostSystem, Eq1a, Output, Point, Regime, Residuals, SolveError, BRACKET_HI, BRACKET_LO,
    LABOR_RESIDUAL_NET, MAX_BISECTION_STEPS,
};

/// 2^n exactly, for the unit tests' dyadic inputs (`powi` is denied: clippy.toml, A5). The
/// double with that exponent and a zero mantissa; n must be in the normal range.
#[cfg(test)]
pub(crate) fn pow2(n: i32) -> f64 {
    assert!((-1022..=1023).contains(&n), "2^{n} is not a normal double");
    f64::from_bits(((1023 + n) as u64) << 52)
}
