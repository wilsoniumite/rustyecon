//! The oracle: the static equilibrium of the pinning paper's economy.
//!
//! Unit 1a (docs/unit-1a.md) solves one category: one final good, one machine type and one
//! land input, with durability and interest entering through the scalar user cost
//! u = (ρ + δ)(1 + ρ)^(J_b − 1). At (ρ, δ, J_b) = (0, 1, 1) it is the SSRN Appendix B
//! economy (SSRN 7226858, pp.28-30), and it reproduces that appendix's published numbers.
//!
//! Unit 1b (docs/unit-1b.md) extends it to many categories bought in a fixed basket, on one
//! task line: [`CategoryEconomy`], with the fork identity, the category bounds and the
//! purchasing-power pair at every equilibrium ([`Eq1b`]). A one-category economy solves to
//! 1a's equilibrium bit for bit. The price block alone, for categories given as task cells
//! at any prices, and the CES share of SSRN eq 26, are [`cell_cost`] and [`ces_share`].
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

mod categories;
mod closure;
pub mod dump;
mod fork;
mod params;
mod schedule;
mod solve;

pub use categories::{
    Category, CategoryEconomy, CategoryEq, CategoryParams, CategoryPoint, Eq1b, Output1b,
    OutputKey, Residuals1b,
};
pub use closure::{closure, Closure, ClosureError};
pub use fork::{cell_cost, ces_share, CategoryCost, Cell};
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
