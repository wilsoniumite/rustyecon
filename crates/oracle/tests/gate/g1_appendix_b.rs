//! G1: the SSRN Appendix B instance (SSRN pp.28-30).

use oracle::{Output, Regime, BRACKET_HI, BRACKET_LO, MAX_BISECTION_STEPS};

use crate::goldens::*;
use crate::support::*;

#[test]
fn published_figures_to_five_decimals() {
    // SSRN p.30, gated as in check_macro.py P1 (:29-34 at 31b3482).
    let eq = interior(appendix_b());
    for (name, got, want) in [
        ("x*", eq.x_star, PUB_G1_X_STAR),
        ("v", eq.v, PUB_G1_V),
        ("Y", eq.y, PUB_G1_Y),
        ("N_a", eq.n_a, PUB_G1_N_A),
        ("final-task hours", eq.final_hours, PUB_G1_FINAL_HOURS),
        (
            "machine-sector hours",
            eq.machine_hours,
            PUB_G1_MACHINE_HOURS,
        ),
        ("N P_s", eq.support_cost, PUB_G1_SUPPORT_COST),
    ] {
        near(name, got, want, PUBLISHED);
    }
}

#[test]
fn full_precision() {
    let eq = interior(appendix_b());
    for (name, got, want) in [
        ("x*", eq.x_star, G1_X_STAR),
        ("1 - x*", eq.one_minus_x_star, 1.0 - G1_X_STAR),
        ("gamma(x*)", eq.gamma_star, G1_GAMMA_STAR),
        ("J(x*)", eq.j_star, G1_J_STAR),
        ("v", eq.v, G1_V),
        ("p_m", eq.p_m, G1_P_M),
        ("V_m", eq.v_m, G1_P_M),
        ("p", eq.p, G1_P),
        ("P_s", eq.p_s, G1_P_S),
        ("Y", eq.y, G1_Y),
        ("K", eq.k, G1_K),
        ("final-task hours", eq.final_hours, G1_FINAL_HOURS),
        ("machine-sector hours", eq.machine_hours, G1_MACHINE_HOURS),
        ("N_a", eq.n_a, G1_N_A),
        ("N_a / N", eq.participation, G1_PARTICIPATION),
        ("N P_s", eq.support_cost, G1_SUPPORT_COST),
        ("I", eq.income, G1_INCOME),
        ("labour share", eq.labor_share, G1_LABOR_SHARE),
        ("w / P_s", eq.real_wage, G1_REAL_WAGE),
        ("worker baskets", eq.worker_baskets, G1_WORKER_BASKETS),
        ("provider baskets", eq.provider_baskets, G1_PROVIDER_BASKETS),
        (
            "coverage T / (N P_s)",
            appendix_b().land / eq.support_cost,
            G1_COVERAGE,
        ),
    ] {
        close(name, got, want);
    }
    // The flow benchmark: u = 1 exactly, so there is no interest.
    assert_eq!(eq.u, 1.0);
    assert_eq!(eq.interest, 0.0);
    assert_eq!(eq.capital_share, 0.0);
    assert_eq!(eq.funded, G1_FUNDED);
    assert_eq!(eq.lemma_b1, G1_LEMMA_B1);
}

#[test]
fn cost_system_and_shares() {
    // SSRN eq 2-4 (p.8) and the three-row display on p.29.
    let params = appendix_b();
    let eq = interior(params.clone());
    let cs = eq.cost_system.expect("u = 1 reports the cost system");
    close(
        "lambda-tilde good",
        cs.lambda_tilde[0],
        G1_LAMBDA_TILDE_GOOD,
    );
    close(
        "lambda-tilde machine",
        cs.lambda_tilde[1],
        G1_LAMBDA_TILDE_MACHINE,
    );
    close("b-tilde good", cs.b_tilde[0], G1_B_TILDE_GOOD);
    close("b-tilde machine", cs.b_tilde[1], G1_B_TILDE_MACHINE);
    assert_eq!(cs.lambda_tilde[2], 0.0);
    assert_eq!(cs.b_tilde[2], 1.0);
    close("L_s", cs.l_s, G1_L_S);
    close("B_s", cs.b_s, G1_B_S);
    // p = (I - A)^-1 (lambda w + b r), row by row, with r = 1.
    close(
        "p = w lt + bt",
        eq.v * cs.lambda_tilde[0] + cs.b_tilde[0],
        eq.p,
    );
    close(
        "p_m = w lt_m + bt_m",
        eq.v * cs.lambda_tilde[1] + cs.b_tilde[1],
        eq.p_m,
    );
    close("P_s = w L_s + r B_s", eq.v * cs.l_s + cs.b_s, eq.p_s);
    // Y = T/B_s and n_D = T L_s / B_s (SSRN eq 11).
    close("Y = T/B_s", params.land / cs.b_s, eq.y);
    close("n_D = T L_s / B_s", params.land * cs.l_s / cs.b_s, eq.n_a);
    // Three-taxes shares at the equilibrium margin (check_three_taxes.py:48 at 31b3482).
    let (phi_w, phi_r) = (eq.phi_w.expect("u = 1"), eq.phi_r.expect("u = 1"));
    close("phi_w", phi_w, G1_PHI_W);
    close("phi_r", phi_r, G1_PHI_R);
    close(
        "phi_w = w lt_m / p_m",
        eq.v * cs.lambda_tilde[1] / eq.p_m,
        phi_w,
    );
    close("phi_r = r bt_m / p_m", cs.b_tilde[1] / eq.p_m, phi_r);
}

#[test]
fn bracket_values() {
    let e = economy(appendix_b());
    close("f(1e-12)", e.at(BRACKET_LO).excess_demand(), G1_F_AT_LO);
    let one = e.at(BRACKET_HI);
    close("n_D(1)", one.n_d, G1_N_D_AT_1);
    close("n_S(1)", one.n_s, G1_N_S_AT_1);
    close("f(1)", one.excess_demand(), G1_F_AT_1);
    close("P_s(1)", one.p_s, G1_P_S_AT_1);
    close(
        "N P_s(1)",
        appendix_b().workers * one.p_s,
        G1_SUPPORT_COST_AT_1,
    );
    close("v(1)", one.v, G1_V_AT_1);
    close("D(1)", one.d, G1_D_AT_1);
}

#[test]
fn solution_quality() {
    let e = economy(appendix_b());
    let eq = interior(appendix_b());
    assert_root(&e, eq.x_star);
    let r = eq.residuals;
    for (name, value) in [
        ("income", r.income),
        ("land", r.land),
        ("services", r.services),
        ("user cost", r.user_cost),
        ("labour, relative to N_a", r.labor / eq.n_a),
    ] {
        assert!(value <= FULL, "{name} residual {value:e}");
    }
    assert!(eq.bisection_steps < MAX_BISECTION_STEPS);
    // Goods clear through Walras' law (SSRN p.30): worker and provider baskets sum to Y.
    close(
        "worker + provider baskets = Y",
        eq.worker_baskets + eq.provider_baskets,
        eq.y,
    );
}

/// Every output of an interior solve as bits, so that 0.0 and -0.0 (equal under `==`)
/// differ.
fn bits(regime: &Regime) -> Vec<(&'static str, Option<u64>)> {
    let eq = regime.interior().expect("interior");
    eq.outputs()
        .into_iter()
        .map(|(key, output)| {
            let bits = match output {
                Output::Float(v) | Output::FlowOnly(Some(v)) => Some(v.to_bits()),
                Output::FlowOnly(None) => None,
                Output::Flag(b) => Some(u64::from(b)),
                Output::Count(n) => Some(u64::from(n)),
            };
            (key, bits)
        })
        .collect()
}

#[test]
fn solve_is_bit_reproducible() {
    let e = economy(appendix_b());
    let first = bits(&e.solve().unwrap());
    for _ in 0..3 {
        assert_eq!(bits(&e.solve().unwrap()), first);
    }
}
