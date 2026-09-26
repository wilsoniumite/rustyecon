//! G6: the replacement closure, the price block alone (main.tex:325-336 and
//! check_pinning.py:59-73 at 31b3482).

use oracle::{closure, ClosureError};

use crate::goldens::*;
use crate::support::*;

/// (a, λ, γ*, b, r): the worked instance of main.tex:325-336.
const INSTANCE: (f64, f64, f64, f64, f64) = (0.5, 0.1, 3.0, 0.2, 1.0);

#[test]
fn worked_instance() {
    let (a, lam, g, b, r) = INSTANCE;
    let c = closure(a, lam, g, b, r, 1.0).unwrap();
    close("p_m (c)", c.p_m, G6_P_M);
    close("w", c.w, G6_W);
    close("D", c.d, G6_D);
    // The recipe balances: 1 = 0.5 * 1 + 0.1 * 3 + 0.2 * 1.
    close("recursion", a * c.p_m + lam * c.w + b * r, c.p_m);
}

#[test]
fn recursive_automation_cuts_the_wage() {
    let (a, _, g, b, r) = INSTANCE;
    let with_labour = closure(a, INSTANCE.1, g, b, r, 1.0).unwrap();
    let c = closure(a, 0.0, g, b, r, 1.0).unwrap();
    close("p_m at lambda = 0", c.p_m, G6_LAM0_P_M);
    close("w at lambda = 0", c.w, G6_LAM0_W);
    close("wage cut", 1.0 - c.w / with_labour.w, G6_LAM0_WAGE_CUT);
}

#[test]
fn not_viable() {
    let (a, _, g, b, r) = INSTANCE;
    match closure(a, 0.2, g, b, r, 1.0) {
        Err(ClosureError::NotViable { d }) => close("D", d, G6_NOT_VIABLE_D),
        other => panic!("expected NotViable, got {other:?}"),
    }
}

#[test]
fn zero_d_is_not_viable() {
    // D = 1 - (0.5 + 0.25 * 2) = 0 exactly in f64: the boundary of the viable set, where
    // p_m = u b r / D would be infinite. D <= 0 is NotViable (spec §3.6).
    match closure(0.5, 0.25, 2.0, 0.2, 1.0, 1.0) {
        Err(ClosureError::NotViable { d }) => assert_eq!(d, 0.0),
        other => panic!("expected NotViable at D = 0, got {other:?}"),
    }
    // The same edge reached through u: 1.25 * (0.3 + 0.1 * 5) rounds to 1 in f64.
    match closure(0.3, 0.1, 5.0, 0.2, 1.0, 1.25) {
        Err(ClosureError::NotViable { d }) => assert_eq!(d, 0.0),
        other => panic!("expected NotViable at D = 0, got {other:?}"),
    }
}

#[test]
fn user_cost_forms_reduce_to_the_static_closure() {
    // check_pinning.py:269-296: c = s b r / (1 - s(a + lambda gamma*)) with s = 1 + rho or
    // s = rho + delta; s = 1 is the static closure, and lambda = 0 gives SSRN A.4's two
    // displays (p.27-28).
    let (a, lam, g, b, r) = INSTANCE;
    let stat = closure(a, lam, g, b, r, 1.0).unwrap();
    close("static form", stat.p_m, b * r / (1.0 - a - lam * g));
    let (rho, delta) = (0.0, 1.0);
    assert_eq!(closure(a, lam, g, b, r, 1.0 + rho).unwrap(), stat);
    assert_eq!(closure(a, lam, g, b, r, rho + delta).unwrap(), stat);
    for (rho, delta) in [(0.05, 1.0), (0.03, 0.08), (0.1, 0.5)] {
        let s = 1.0 + rho;
        let c = closure(a, 0.0, g, b, r, s).unwrap();
        close("A.4 one-period display", c.p_m, s * b * r / (1.0 - a * s));
        let s = rho + delta;
        let c = closure(a, 0.0, g, b, r, s).unwrap();
        close("A.4 durable display", c.p_m, s * b * r / (1.0 - a * s));
        let c = closure(a, lam, g, b, r, s).unwrap();
        close(
            "lambda > 0 user-cost form",
            c.p_m,
            s * b * r / (1.0 - s * (a + lam * g)),
        );
        close("w = gamma* p_m", c.w, g * c.p_m);
        // The recursion itself, not the closed form: p_m = u (a p_m + lambda w + b r).
        close(
            "p_m = u (a p_m + lambda w + b r)",
            s * (a * c.p_m + lam * c.w + b * r),
            c.p_m,
        );
        close("D", c.d, 1.0 - s * (a + lam * g));
    }
}
