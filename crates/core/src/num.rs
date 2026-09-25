//! The one module for transcendentals on the state path, and the exact budget helpers
//! (docs/ENGINE.md §2.5 and §9, ADDENDUM A5).
//!
//! The platform maths libraries disagree in the last bit on a fraction of inputs, so every
//! transcendental the state or verdict path needs is one call into the pure-Rust `libm` crate,
//! which computes the same bits on every platform. `clippy.toml` denies the inherent methods.
//!
//! The helpers answer "how much fits": the largest value whose rounded product or sum stays
//! within a bound. They search the bit patterns of the non-negative doubles, which are ordered
//! like the values, starting from the quotient or difference, so they return the exact largest
//! fitting value and cannot fail on precision. Nothing here panics.

use std::fmt;

/// `e^x`.
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// `e^x − 1`, accurate near zero.
pub fn expm1(x: f64) -> f64 {
    libm::expm1(x)
}

/// The natural logarithm.
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

/// `ln(1 + x)`, accurate near zero.
pub fn ln1p(x: f64) -> f64 {
    libm::log1p(x)
}

/// `x^y`.
pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

/// Whether `x` may sit on the state path: finite, with a clear sign bit (so not `-0.0`).
pub fn is_clean(x: f64) -> bool {
    x.is_finite() && !x.is_sign_negative()
}

/// Why a helper could not answer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumError {
    /// An input was non-finite or had its sign bit set.
    Invalid {
        /// Which input.
        what: &'static str,
        /// Its value.
        value: f64,
    },
    /// `max_remainder` was asked for what remains of `total` after spending more than it.
    Exceeded {
        /// The bound.
        total: f64,
        /// What was already spent.
        spent: f64,
    },
}

impl fmt::Display for NumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumError::Invalid { what, value } => write!(
                f,
                "{what} is {value:e}: helper inputs must be finite with a clear sign bit"
            ),
            NumError::Exceeded { total, spent } => {
                write!(f, "spent {spent:e} exceeds the total {total:e}")
            }
        }
    }
}

impl std::error::Error for NumError {}

fn clean(what: &'static str, value: f64) -> Result<f64, NumError> {
    if is_clean(value) {
        Ok(value)
    } else {
        Err(NumError::Invalid { what, value })
    }
}

/// The largest `q` with `fl(price·q) <= budget`.
///
/// A zero budget gives 0: nothing is bought with nothing. When `budget/price` is infinite (a
/// zero or tiny price, N6) the result is `+∞`, so any finite quantity binds instead.
pub fn max_qty(budget: f64, price: f64) -> Result<f64, NumError> {
    let budget = clean("budget", budget)?;
    let price = clean("price", price)?;
    if budget == 0.0 {
        return Ok(0.0);
    }
    let est = budget / price;
    if est.is_infinite() {
        return Ok(f64::INFINITY);
    }
    largest(est, |q| price * q <= budget)
}

/// The largest `x` with `fl(coef·x) <= held`.
///
/// Nothing held gives 0. A zero coefficient does not bind, so the result is `+∞`, as it is
/// whenever `held/coef` overflows.
pub fn max_scale(held: f64, coef: f64) -> Result<f64, NumError> {
    let held = clean("held", held)?;
    let coef = clean("coef", coef)?;
    if held == 0.0 {
        return Ok(0.0);
    }
    let est = held / coef;
    if est.is_infinite() {
        return Ok(f64::INFINITY);
    }
    largest(est, |x| coef * x <= held)
}

/// The largest `d` with `fl(spent + d) <= total`.
///
/// When `spent == total` nothing remains and the result is 0. Spending more than `total` is
/// [`NumError::Exceeded`].
pub fn max_remainder(total: f64, spent: f64) -> Result<f64, NumError> {
    let total = clean("total", total)?;
    let spent = clean("spent", spent)?;
    if spent > total {
        return Err(NumError::Exceeded { total, spent });
    }
    if spent == total {
        return Ok(0.0);
    }
    largest(total - spent, |d| spent + d <= total)
}

/// The largest non-negative finite double for which `fits` holds, given a finite, non-negative
/// estimate. `fits` must be monotone (true up to some value, false beyond it) and must hold at
/// zero; every caller's bound is a rounded product or sum, which is.
///
/// Non-negative doubles order like their bit patterns, so the search runs on `u64`: it gallops
/// from the estimate (normally within a few steps of the answer) until it brackets the
/// boundary, then bisects. At most about 130 evaluations, and usually 2 to 4.
fn largest(est: f64, fits: impl Fn(f64) -> bool) -> Result<f64, NumError> {
    let top = f64::MAX.to_bits();
    let at = |bits: u64| fits(f64::from_bits(bits));
    let start = clean("estimate", est)?.to_bits();
    // lo always fits; hi never does.
    let (mut lo, mut hi);
    if at(start) {
        lo = start;
        let mut step = 1u64;
        loop {
            let cand = lo.saturating_add(step).min(top);
            if cand == lo {
                return Ok(f64::from_bits(lo));
            }
            if at(cand) {
                lo = cand;
                step = step.saturating_mul(2);
            } else {
                hi = cand;
                break;
            }
        }
    } else {
        hi = start;
        let mut step = 1u64;
        loop {
            let cand = hi.saturating_sub(step);
            if at(cand) {
                lo = cand;
                break;
            }
            if cand == 0 {
                // Unreachable for the callers' bounds, which all hold at zero.
                return Err(NumError::Invalid {
                    what: "bound at zero",
                    value: est,
                });
            }
            hi = cand;
            step = step.saturating_mul(2);
        }
    }
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if at(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(f64::from_bits(lo))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small deterministic generator for test inputs; the engine itself has no randomness.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 11
        }
        /// A positive double spread over many binades.
        fn positive(&mut self) -> f64 {
            let mantissa = (self.next() % (1 << 52)) as f64 / (1u64 << 52) as f64;
            let exponent = (self.next() % 81) as i32 - 40;
            (1.0 + mantissa) * libm::pow(2.0, f64::from(exponent))
        }
    }

    #[test]
    fn num_matches_libm_bits() {
        let xs = [
            -700.0,
            -20.5,
            -1.0,
            -1e-9,
            -0.0,
            0.0,
            1e-12,
            0.1,
            0.5,
            1.0,
            std::f64::consts::E,
            7.0,
            88.5,
            700.0,
        ];
        for &x in &xs {
            assert_eq!(exp(x).to_bits(), libm::exp(x).to_bits(), "exp({x})");
            assert_eq!(expm1(x).to_bits(), libm::expm1(x).to_bits(), "expm1({x})");
            if x > -1.0 {
                assert_eq!(ln1p(x).to_bits(), libm::log1p(x).to_bits(), "ln1p({x})");
            }
            if x > 0.0 {
                assert_eq!(ln(x).to_bits(), libm::log(x).to_bits(), "ln({x})");
                assert_eq!(
                    pow(x, 1.7).to_bits(),
                    libm::pow(x, 1.7).to_bits(),
                    "pow({x})"
                );
            }
        }
        // Golden bits: libm is pure Rust, so these hold on every platform. A change here means
        // the implementation behind `num` changed, which moves every hash.
        // (libm's exp(1) is one ulp above the correctly rounded e; what matters is that it is
        // the same everywhere.)
        assert_eq!(exp(1.0).to_bits(), 0x4005_bf0a_8b14_576a);
        assert_eq!(ln(10.0).to_bits(), 0x4002_6bb1_bbb5_5516);
        assert_eq!(expm1(-0.1).to_bits(), 0xbfb8_5c93_3156_a62c);
        assert_eq!(pow(2.0, 0.5).to_bits(), 0x3ff6_a09e_667f_3bcd);
    }

    #[test]
    fn num_helpers_are_exact() {
        let mut g = Lcg(0x2026_0925);
        for _ in 0..20_000 {
            let (a, b) = (g.positive(), g.positive());

            let q = max_qty(a, b).unwrap();
            assert!(b * q <= a, "max_qty({a:e}, {b:e}) = {q:e} does not fit");
            assert!(
                b * q.next_up() > a,
                "max_qty({a:e}, {b:e}) = {q:e} is not the largest"
            );

            let x = max_scale(a, b).unwrap();
            assert!(
                b * x <= a && b * x.next_up() > a,
                "max_scale({a:e}, {b:e}) = {x:e}"
            );

            let (total, spent) = if a >= b { (a, b) } else { (b, a) };
            if spent < total {
                let d = max_remainder(total, spent).unwrap();
                assert!(
                    spent + d <= total,
                    "max_remainder({total:e}, {spent:e}) = {d:e}"
                );
                assert!(spent + d.next_up() > total, "max_remainder not the largest");
            }
        }
        // A remainder far smaller than the total: the case where stepping one ulp at a time from
        // the difference would take hundreds of steps.
        let d = max_remainder(1.0, 0.999).unwrap();
        assert!(0.999 + d <= 1.0 && 0.999 + d.next_up() > 1.0);
        // Nothing to spend, nothing held, nothing left: exactly zero.
        assert_eq!(max_qty(0.0, 0.5).unwrap().to_bits(), 0);
        assert_eq!(max_scale(0.0, 3.0).unwrap().to_bits(), 0);
        assert_eq!(max_remainder(2.5, 2.5).unwrap().to_bits(), 0);
        // An infinite quotient does not bind.
        assert_eq!(max_qty(1e300, 1e-300).unwrap(), f64::INFINITY);
        assert_eq!(max_qty(1.0, 0.0).unwrap(), f64::INFINITY);
        assert_eq!(max_scale(1.0, 0.0).unwrap(), f64::INFINITY);
        // Invalid inputs and overspending are errors, never panics.
        assert!(matches!(
            max_qty(f64::NAN, 1.0),
            Err(NumError::Invalid { .. })
        ));
        assert!(matches!(max_qty(1.0, -0.0), Err(NumError::Invalid { .. })));
        assert!(matches!(
            max_scale(-1.0, 1.0),
            Err(NumError::Invalid { .. })
        ));
        assert!(matches!(
            max_remainder(1.0, 1.5),
            Err(NumError::Exceeded { .. })
        ));
        // A price of 3 does not divide 1 exactly: the quotient rounds, and the answer is the
        // largest q whose cost still rounds to at most the budget.
        let q = max_qty(1.0, 3.0).unwrap();
        assert!(3.0 * q <= 1.0 && 3.0 * q.next_up() > 1.0);
    }
}
