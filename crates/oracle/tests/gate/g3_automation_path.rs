//! G3: the automation path of SSRN p.30, λ = 0 and γ = η(1 + x), as laborformal runs it
//! (check_macro.py P2, :35-43 at 31b3482).

use oracle::{Params, PowerSchedule};

use crate::goldens::*;
use crate::support::*;

/// (η, N_a/N, v) along the path.
const PATH: [(f64, f64, f64); 5] = [
    (1.0, G3_ETA_1_PARTICIPATION, G3_ETA_1_V),
    (0.3, G3_ETA_0_3_PARTICIPATION, G3_ETA_0_3_V),
    (0.1, G3_ETA_0_1_PARTICIPATION, G3_ETA_0_1_V),
    (0.03, G3_ETA_0_03_PARTICIPATION, G3_ETA_0_03_V),
    (0.01, G3_ETA_0_01_PARTICIPATION, G3_ETA_0_01_V),
];

/// γ = η(1 + x): g0 = g1 = η with the schedule's own η at 1, as in check_macro.py:37.
fn at(eta: f64) -> Params {
    Params {
        lam: 0.0,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: eta,
            g1: eta,
            k: 1.0,
        },
        ..appendix_b()
    }
}

#[test]
fn path_values() {
    for (eta, participation, v) in PATH {
        let eq = interior(at(eta));
        close(
            &format!("N_a / N at eta = {eta}"),
            eq.participation,
            participation,
        );
        close(&format!("v at eta = {eta}"), eq.v, v);
    }
}

#[test]
fn the_papers_claims_along_the_path() {
    // SSRN p.30: 0 < v <= 2 b eta / (1 - a), the conditions (25) hold throughout,
    // support stays funded, and participation falls toward zero.
    let p = appendix_b();
    let mut last = f64::INFINITY;
    for (eta, _, _) in PATH {
        let eq = interior(at(eta));
        assert!(
            eq.v > 0.0 && eq.v <= 2.0 * p.b * eta / (1.0 - p.a),
            "v at eta = {eta}"
        );
        assert!(eq.lemma_b1, "Lemma B.1 at eta = {eta}");
        assert!(eq.funded, "funded at eta = {eta}");
        assert!(
            eq.participation < last,
            "participation rises at eta = {eta}"
        );
        last = eq.participation;
        // With lambda = 0 the three-taxes shares are (0, 1) (check_three_taxes.py:52).
        assert_eq!((eq.phi_w, eq.phi_r), (Some(0.0), Some(1.0)));
    }
}

#[test]
fn supply_uses_ln_1p() {
    // Deep along the path (eta = 1e-6), z = v/P_s is about 7e-7 (spec §4 step 4). ln(1 + z)
    // rounds 1 + z first and loses up to eps/z = 1.6e-10 relative, 1.2e-10 at x = 0.25;
    // ln_1p(z) does not. A point evaluation, with no root-finding between the formula and
    // the golden.
    let e = economy(at(1e-6));
    close("n_S(0.25)", e.at(0.25).n_s, G3_ETA_1E_6_N_S_AT_QUARTER);
}

#[test]
fn near_full_automation() {
    // Deep along the path 1 - x* falls below what the double x* resolves: at eta = 1e-6
    // it is 4.6e-7, where 1.0 - x* carries 1e-16/4.6e-7 = 2.4e-10 relative error, and at
    // eta = 1e-20 it is 4.6e-21, where x* rounds to 1.0 and 1.0 - x* is 0. The outputs
    // proportional to 1 - x* come from Eq1a::one_minus_x_star and keep full precision (the
    // review measured N_a off by 2.7e-11 at eta = 1e-6, and 0 for 2.3e-30, before it).
    let cases = [
        (
            1e-6,
            G3_ETA_1E_6_ONE_MINUS_X_STAR,
            G3_ETA_1E_6_N_A,
            G3_ETA_1E_6_PARTICIPATION,
            G3_ETA_1E_6_LABOR_SHARE,
        ),
        (
            1e-20,
            G3_ETA_1E_20_ONE_MINUS_X_STAR,
            G3_ETA_1E_20_N_A,
            G3_ETA_1E_20_PARTICIPATION,
            G3_ETA_1E_20_LABOR_SHARE,
        ),
    ];
    for (eta, one_minus_x, n_a, participation, labor_share) in cases {
        let eq = interior(at(eta));
        let tag = |what: &str| format!("{what} at eta = {eta:e}");
        close(&tag("1 - x*"), eq.one_minus_x_star, one_minus_x);
        close(&tag("N_a"), eq.n_a, n_a);
        // lambda = 0: every hour is at final tasks.
        assert_eq!(eq.machine_hours, 0.0);
        close(&tag("final hours"), eq.final_hours, n_a);
        close(&tag("N_a / N"), eq.participation, participation);
        close(&tag("labour share"), eq.labor_share, labor_share);
        close(
            &tag("lambda-tilde good = 1 - x*"),
            eq.cost_system.expect("u = 1").lambda_tilde[0],
            one_minus_x,
        );
        assert!(eq.lemma_b1 && eq.funded, "{}", tag("Lemma B.1"));
    }
    // At eta = 1e-20 the double x* is 1.0, and the result is still interior.
    let eq = interior(at(1e-20));
    assert_eq!(eq.x_star, 1.0);
    assert!(eq.one_minus_x_star > 0.0 && eq.final_hours > 0.0);
}
