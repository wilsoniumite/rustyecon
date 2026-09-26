//! Fixtures and comparisons shared by the gate's tests.

use oracle::{Economy, Eq1a, Params, PowerSchedule, Regime, UniformWorkCost};

/// Relative tolerance for full-precision goldens and identities: 1e-12 (ADDENDUM A7).
/// laborformal asserts its identities at the same level (check_macro.py:59,129 at
/// 31b3482); the PLAN asks only 1e-10.
pub const FULL: f64 = 1e-12;

/// Absolute tolerance for figures published to five decimals: half a unit in the last
/// place (check_macro.py:34 at 31b3482; ADDENDUM A7).
pub const PUBLISHED: f64 = 5e-6;

/// Absolute tolerance for figures published to two decimals, SSRN Figure 3's caption
/// (p.12): half a unit in the last place, the rule behind [`PUBLISHED`].
pub const CAPTION: f64 = 5e-3;

/// The SSRN Appendix B instance (SSRN p.30): N 4, T 10, h 1, a 0.3, b 0.4, λ 0.05,
/// γ = 0.2 + 0.8x, χ ~ U[0, 1], at the flow benchmark (ρ, δ, J_b) = (0, 1, 1).
pub fn appendix_b() -> Params {
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

/// Validates parameters the test knows to be valid.
pub fn economy(params: Params) -> Economy {
    Economy::new(params.clone()).unwrap_or_else(|e| panic!("{params:?}: {e}"))
}

/// Solves an economy the test knows to be interior.
pub fn interior(params: Params) -> Box<Eq1a> {
    match economy(params.clone()).solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{params:?} is not interior: {other:?}"),
    }
}

/// 2^n exactly, for dyadic inputs (`powi` is denied: clippy.toml, A5). The double with that
/// exponent and a zero mantissa; n must be in the normal range.
pub fn pow2(n: i32) -> f64 {
    assert!((-1022..=1023).contains(&n), "2^{n} is not a normal double");
    f64::from_bits(((1023 + n) as u64) << 52)
}

/// |got − want| ≤ tol·|want|.
pub fn close_to(name: &str, got: f64, want: f64, tol: f64) {
    let err = (got - want).abs();
    assert!(
        err <= tol * want.abs(),
        "{name}: got {got:e}, want {want:e}, rel err {:e} > {tol:e}",
        err / want.abs()
    );
}

/// Relative agreement at [`FULL`].
pub fn close(name: &str, got: f64, want: f64) {
    close_to(name, got, want, FULL);
}

/// |got − want| ≤ tol.
pub fn near(name: &str, got: f64, want: f64, tol: f64) {
    let err = (got - want).abs();
    assert!(
        err <= tol,
        "{name}: got {got}, want {want}, abs err {err:e} > {tol:e}"
    );
}

/// x is where f = n_D − n_S changes sign, to one double: f is positive on one side and
/// negative on the other, and x is the side with the smaller |f|.
pub fn assert_root(economy: &Economy, x: f64) {
    let f = |x: f64| economy.at(x).excess_demand();
    let fx = f(x);
    if fx == 0.0 {
        return;
    }
    let other = if fx > 0.0 { x.next_up() } else { x.next_down() };
    let f_other = f(other);
    assert!(
        f_other == 0.0 || f_other.signum() != fx.signum(),
        "no sign change between {x:e} (f = {fx:e}) and {other:e} (f = {f_other:e})"
    );
    assert!(
        fx.abs() <= f_other.abs(),
        "x* = {x:e} is not the better of its bracket"
    );
}
