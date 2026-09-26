//! G7: the three-taxes resolution shares on the G6 instance (check_three_taxes.py:27-61
//! at 31b3482; SSRN p.9).

use oracle::closure;

use crate::goldens::*;
use crate::support::*;

#[test]
fn shares_on_the_worked_instance() {
    let (a, lam, g, b, r) = (0.5, 0.1, 3.0, 0.2, 1.0);
    let c = closure(a, lam, g, b, r, 1.0).unwrap();
    let (phi_w, phi_r) = (c.phi_w.expect("u = 1"), c.phi_r.expect("u = 1"));
    close("phi_w", phi_w, G7_PHI_W);
    close("phi_r", phi_r, G7_PHI_R);
    // T1's ledger: phi_w = w lambda-tilde_m / c and phi_r = r b-tilde_m / c, with
    // lambda-tilde_m = lambda/(1 - a) and b-tilde_m = b/(1 - a) (check_three_taxes.py:40-50).
    close("phi_w = w lt_m / c", c.w * lam / (1.0 - a) / c.p_m, phi_w);
    close("phi_r = r bt_m / c", r * b / (1.0 - a) / c.p_m, phi_r);
}

#[test]
fn lambda_zero_corner() {
    // check_three_taxes.py:52-53: lambda -> 0 gives (phi_w, phi_r) = (0, 1).
    let c = closure(0.5, 0.0, 3.0, 0.2, 1.0, 1.0).unwrap();
    assert_eq!(c.phi_w, Some(G7_LAM0_PHI_W));
    assert_eq!(c.phi_r, Some(G7_LAM0_PHI_R));
}
