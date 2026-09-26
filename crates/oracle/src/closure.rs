//! The price block on its own, at a given margin (spec §3.6).

use std::fmt;

use rustyecon_core::num;

use crate::params::{self, ParamError};

/// The replacement closure at a given margin γ* (spec §3.6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Closure {
    /// p_m = u·b·r/D, the machine-service price (main.tex's c).
    pub p_m: f64,
    /// w = γ*·p_m, the wage at the margin.
    pub w: f64,
    /// D = 1 − u(a + λγ*), positive on the viable set.
    pub d: f64,
    /// φ_w = λγ*/(1 − a) = w·λ̃_m/p_m, labour's share of p_m, when u = 1
    /// (check_three_taxes.py:48 at 31b3482).
    pub phi_w: Option<f64>,
    /// φ_r = 1 − φ_w = r·b̃_m/p_m, rent's share of p_m, when u = 1.
    pub phi_r: Option<f64>,
}

/// Why [`closure`] returned no prices.
#[derive(Clone, Debug, PartialEq)]
pub enum ClosureError {
    /// An input is outside its range.
    Invalid(ParamError),
    /// D = 1 − u(a + λγ*) ≤ 0: no positive price solves the recursion.
    NotViable {
        /// D.
        d: f64,
    },
    /// The inputs are valid and viable, but a price overflowed: u·b·r/D is beyond f64.
    NonFinite {
        /// The output that was not finite: `"p_m"` or `"w"`.
        what: &'static str,
    },
}

impl fmt::Display for ClosureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClosureError::Invalid(e) => write!(f, "invalid input: {e}"),
            ClosureError::NotViable { d } => write!(f, "not viable: D = {d:?} <= 0"),
            ClosureError::NonFinite { what } => write!(f, "{what} is not finite"),
        }
    }
}

impl std::error::Error for ClosureError {}

impl From<ParamError> for ClosureError {
    fn from(e: ParamError) -> Self {
        ClosureError::Invalid(e)
    }
}

/// D = 1 − u(a + λγ), the viability denominator (spec §3.1), shared by [`closure`] and
/// the solve.
///
/// It is evaluated as (1 − u·a) − u·λ·γ, with 1 − u·a from one fused multiply-add (core's
/// `num::fma`, correctly rounded, so the same on every platform). The plain form
/// 1 − u·(a + λγ) rounds a + λγ, of order 1, before the cancellation, which
/// costs up to 1.1e-16/D relative: near the viability edge (D → 0) that reaches the
/// prices, since p_m = u·b/D. The fused form rounds 1 − u·a once, at its own magnitude,
/// so the error is about 2^-53·(1 − u·a)/D relative, and none when λγ = 0. At u = 1 and
/// a ≥ 0.5, 1 − a is exact. The review of 2026-09-25 found p_m off by 8.1e-3 relative at
/// D(x*) ≈ 5.6e-15 with the plain form; G4 case G gates the fused one at D(x*) ≈ 8.4e-7.
pub(crate) fn viability(u: f64, a: f64, lam: f64, gamma: f64) -> f64 {
    num::fma(-u, a, 1.0) - u * lam * gamma
}

/// Solves p_m = u(a·p_m + λ·w + b·r) with the margin w = γ*·p_m:
/// p_m = u·b·r/(1 − u(a + λγ*)) and w = γ*·p_m.
///
/// At u = 1 this is the static closure of main.tex:254-258 (laborformal at 31b3482).
/// With u = 1 + ρ or u = ρ + δ and λ = 0 it gives SSRN A.4's two displays (p.28), and
/// its λ > 0 form is checked in check_pinning.py:269-296. There is no equilibrium solve.
/// D is computed as in the solve (see `viability` in the source).
///
/// Requires 0 ≤ a < 1, 0 ≤ λ ≤ [`SCALE_CEIL`](crate::SCALE_CEIL), and γ*, b, r, u finite
/// and positive; a negative zero in a or λ is read as +0.0. Returns
/// [`ClosureError::NonFinite`] if p_m or w overflows, so an `Ok` result is always finite.
pub fn closure(
    a: f64,
    lam: f64,
    gamma_star: f64,
    b: f64,
    r: f64,
    u: f64,
) -> Result<Closure, ClosureError> {
    let a = params::machine_share("a", a)?;
    let lam = params::nonnegative("lam", lam)?;
    let gamma_star = params::positive("gamma_star", gamma_star)?;
    let b = params::positive("b", b)?;
    let r = params::positive("r", r)?;
    let u = params::positive("u", u)?;
    let d = viability(u, a, lam, gamma_star);
    if d <= 0.0 {
        return Err(ClosureError::NotViable { d });
    }
    let p_m = u * b * r / d;
    let w = gamma_star * p_m;
    for (what, value) in [("p_m", p_m), ("w", w)] {
        if !value.is_finite() {
            return Err(ClosureError::NonFinite { what });
        }
    }
    let phi_w = (u == 1.0).then(|| lam * gamma_star / (1.0 - a));
    Ok(Closure {
        p_m,
        w,
        d,
        phi_w,
        phi_r: phi_w.map(|w| 1.0 - w),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pow2;

    #[test]
    fn invalid_inputs_are_errors() {
        for (args, name) in [
            ((1.0, 0.1, 3.0, 0.2, 1.0, 1.0), "a"),
            ((0.5, -0.1, 3.0, 0.2, 1.0, 1.0), "lam"),
            ((0.5, 0.1, 0.0, 0.2, 1.0, 1.0), "gamma_star"),
            ((0.5, 0.1, 3.0, f64::NAN, 1.0, 1.0), "b"),
            ((0.5, 0.1, 3.0, 0.2, -1.0, 1.0), "r"),
            ((0.5, 0.1, 3.0, 0.2, 1.0, 0.0), "u"),
        ] {
            let (a, lam, g, b, r, u) = args;
            match closure(a, lam, g, b, r, u) {
                Err(ClosureError::Invalid(e)) => assert_eq!(e.name(), name),
                other => panic!("{args:?} gave {other:?}"),
            }
        }
    }

    #[test]
    fn overflowing_prices_are_errors() {
        // Valid and viable (D = 1), but p_m = b r / D = 1e600 overflows (review of
        // 2026-09-25: this returned Ok with infinite prices).
        assert_eq!(
            closure(0.0, 0.0, 1.0, 1e300, 1e300, 1.0),
            Err(ClosureError::NonFinite { what: "p_m" })
        );
        // p_m is finite, w = gamma* p_m is not.
        assert_eq!(
            closure(0.0, 0.0, 1e300, 1e300, 1e8, 1.0),
            Err(ClosureError::NonFinite { what: "w" })
        );
        assert_eq!(
            ClosureError::NonFinite { what: "p_m" }.to_string(),
            "p_m is not finite"
        );
    }

    #[test]
    fn negative_zero_inputs_read_as_positive_zero() {
        let c = closure(-0.0, -0.0, 3.0, 0.2, 1.0, 1.0).unwrap();
        assert_eq!(c.phi_w.map(f64::to_bits), Some(0.0f64.to_bits()));
        assert_eq!(c, closure(0.0, 0.0, 3.0, 0.2, 1.0, 1.0).unwrap());
    }

    #[test]
    fn viability_is_accurate_near_the_edge() {
        // u a = 1 - 2^-19 and u lambda gamma = 2^-20 gamma, both exact, so
        // D = 2^-20 (2 - gamma), which is exact in f64 too (2 - gamma by Sterbenz). The
        // fused form gets it exactly. The plain form rounds a + lambda gamma, which needs
        // bits down to 2^-72, to the doubles near 1, 2^-53 apart.
        let (a, lam, gamma) = (1.0 - pow2(-19), pow2(-20), 1.4);
        let exact = (2.0 - gamma) * pow2(-20);
        assert_eq!(viability(1.0, a, lam, gamma), exact);
        let plain = 1.0 - (a + lam * gamma);
        // Off by 3.9e-11 relative: beyond the gate's 1e-12.
        assert!((plain - exact).abs() > 1e-12 * exact, "{plain:e}");
        // The same through u != 1: u = 2, a = (1 - 2^-19)/2 exactly.
        assert_eq!(viability(2.0, a / 2.0, lam / 2.0, gamma), exact);
        // lambda = 0: D = 1 - u a with one rounding, exact at u = 1 for a >= 0.5.
        assert_eq!(viability(1.0, 0.75, 0.0, 9.0), 0.25);
    }

    #[test]
    fn shares_only_at_u_one() {
        let c = closure(0.5, 0.1, 3.0, 0.2, 1.0, 1.05).unwrap();
        assert_eq!((c.phi_w, c.phi_r), (None, None));
        let c = closure(0.5, 0.1, 3.0, 0.2, 1.0, 1.0).unwrap();
        assert!(c.phi_w.is_some() && c.phi_r.is_some());
    }
}
