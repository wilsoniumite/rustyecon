//! G4: durability and interest through u = (ρ + δ)(1 + ρ)^(J_b − 1) (spec §3.0;
//! check_dynamics.py:45 at 31b3482; SSRN A.4, pp.27-28).

use oracle::{Params, PowerSchedule};

use crate::goldens::*;
use crate::support::*;

fn durable(rho: f64, delta: f64, build_lag: u32) -> Params {
    Params {
        rho,
        delta,
        build_lag,
        ..appendix_b()
    }
}

#[test]
fn inputs_advanced_one_period() {
    // (rho, delta, J_b) = (0.05, 1, 1): u = 1 + rho, SSRN A.4's first display (p.27).
    let eq = interior(durable(0.05, 1.0, 1));
    close("u", eq.u, G4_A_U);
    close("x*", eq.x_star, G4_A_X_STAR);
    close("v", eq.v, G4_A_V);
    close("Y", eq.y, G4_A_Y);
    close("N_a", eq.n_a, G4_A_N_A);
    close("p_m", eq.p_m, G4_A_P_M);
    close("V_m", eq.v_m, G4_A_V_M);
    close("I", eq.income, G4_A_INCOME);
    close("capital share", eq.capital_share, G4_A_CAPITAL_SHARE);
    close("labour share", eq.labor_share, G4_A_LABOR_SHARE);
    close("w / P_s", eq.real_wage, G4_A_REAL_WAGE);
}

#[test]
fn durable_asset() {
    // (0.05, 0.1, 1): u = rho + delta, SSRN A.4's second display (p.28).
    let eq = interior(durable(0.05, 0.1, 1));
    close("u", eq.u, G4_B_U);
    close("x*", eq.x_star, G4_B_X_STAR);
    close("v", eq.v, G4_B_V);
    close("Y", eq.y, G4_B_Y);
    close("N_a", eq.n_a, G4_B_N_A);
    close("p_m", eq.p_m, G4_B_P_M);
    close("V_m", eq.v_m, G4_B_V_M);
    close("K", eq.k, G4_B_K);
    close("I", eq.income, G4_B_INCOME);
    close("capital share", eq.capital_share, G4_B_CAPITAL_SHARE);
    close("labour share", eq.labor_share, G4_B_LABOR_SHARE);
    close("w / P_s", eq.real_wage, G4_B_REAL_WAGE);
}

#[test]
fn zero_interest_nests_the_scaled_flow_benchmark() {
    // (0, 0.1, 1). At rho = 0 the durable economy is the flow benchmark with its
    // coefficients scaled by delta: (a, lambda, b) = (0.03, 0.005, 0.04). The two are
    // equal in exact arithmetic (generate.py asserts it at 70 digits); in f64 they
    // round differently, so they are compared at FULL.
    let eq = interior(durable(0.0, 0.1, 1));
    close("u", eq.u, G4_C_U);
    close("x*", eq.x_star, G4_C_X_STAR);
    close("v", eq.v, G4_C_V);
    close("Y", eq.y, G4_C_Y);
    close("N_a", eq.n_a, G4_C_N_A);
    assert_eq!(eq.interest, 0.0);
    let flow = interior(Params {
        a: 0.03,
        lam: 0.005,
        b: 0.04,
        ..appendix_b()
    });
    for (name, got, want) in [
        ("x*", eq.x_star, flow.x_star),
        ("v", eq.v, flow.v),
        ("Y", eq.y, flow.y),
        ("N_a", eq.n_a, flow.n_a),
        ("P_s", eq.p_s, flow.p_s),
        ("p_m", eq.p_m, flow.p_m),
        ("K", eq.k, flow.k),
    ] {
        close(&format!("{name}, durable vs scaled flow"), got, want);
    }
}

#[test]
fn build_lag() {
    // (0.05, 0.1, 3): u = 0.15 * 1.05^2. Derived for this closure; laborformal has no
    // number for it.
    let eq = interior(durable(0.05, 0.1, 3));
    close("u", eq.u, G4_D_U);
    close("x*", eq.x_star, G4_D_X_STAR);
    close("v", eq.v, G4_D_V);
    close("Y", eq.y, G4_D_Y);
    close("N_a", eq.n_a, G4_D_N_A);
    close("I", eq.income, G4_D_INCOME);
}

#[test]
fn interest_decides_funding() {
    // (N, rho, delta, J_b) = (7.3, 0.05, 1, 1): T < N P_s(x*) < T + interest, so the
    // provider can fund support only with its interest (macro.py:113).
    let params = Params {
        workers: 7.3,
        ..durable(0.05, 1.0, 1)
    };
    let eq = interior(params.clone());
    close("x*", eq.x_star, G4_E_X_STAR);
    close("N P_s", eq.support_cost, G4_E_SUPPORT_COST);
    close("interest", eq.interest, G4_E_INTEREST);
    close(
        "provider baskets",
        eq.provider_baskets,
        G4_E_PROVIDER_BASKETS,
    );
    assert!(eq.support_cost > params.land, "N P_s > T");
    assert!(
        eq.support_cost < params.land + eq.interest,
        "N P_s < T + interest"
    );
    assert!(eq.provider_baskets > 0.0);
    assert_eq!(eq.funded, G4_E_FUNDED);
    // With interest in income, v N_a / I is not v N_a / (v N_a + T).
    close("labour share", eq.labor_share, G4_E_LABOR_SHARE);
    close("w / P_s", eq.real_wage, G4_E_REAL_WAGE);
}

#[test]
fn interest_is_rho_times_machine_wealth() {
    // check_dynamics.py L1-L2 (:155-169 at 31b3482): machine-sector cash (u - delta) V_m K
    // equals rho W_K, with W_K = V_m K [(1+rho)^(J_b-1) + delta((1+rho)^(J_b-1) - 1)/rho].
    // ((1+rho)^n - 1)/rho is summed term by term here, so a tiny rho loses nothing.
    let cases = [
        (0.05, 1.0, 1),
        (0.05, 0.1, 1),
        (0.05, 0.1, 3),
        (0.08, 0.3, 6),
        (1e-9, 0.9, 1),
        (1e-9, 0.5, 4),
    ];
    for (rho, delta, lag) in cases {
        let eq = interior(durable(rho, delta, lag));
        let tag = format!("at (rho, delta, J_b) = ({rho}, {delta}, {lag})");
        let (mut carry, mut series) = (1.0, 0.0);
        for _ in 1..lag {
            series += carry;
            carry *= 1.0 + rho;
        }
        let wealth = eq.v_m * eq.k * (carry + delta * series);
        close(
            &format!("interest = rho W_K {tag}"),
            eq.interest,
            rho * wealth,
        );
        // The defining form, (u - delta) V_m K, cancels when rho is small against delta;
        // it agrees to FULL relative to income.
        let cash = (eq.u - delta) * eq.v_m * eq.k;
        near(
            &format!("interest = (u - delta) V_m K {tag}"),
            eq.interest,
            cash,
            FULL * eq.income,
        );
    }
}

#[test]
fn flow_only_outputs_are_absent_when_u_is_not_one() {
    for (rho, delta, lag) in [
        (0.05, 1.0, 1),
        (0.05, 0.1, 1),
        (0.0, 0.1, 1),
        (0.05, 0.1, 3),
    ] {
        let eq = interior(durable(rho, delta, lag));
        assert_ne!(eq.u, 1.0);
        assert_eq!((eq.phi_w, eq.phi_r), (None, None));
        assert!(eq.cost_system.is_none());
    }
}

#[test]
fn u_one_with_partial_depreciation() {
    // (rho, delta) = (0.5, 0.5) gives u = 1 exactly: the price side is the flow cost
    // system, but the clearing side scales the machine recipe by delta (spec §3.5), so
    // P_s = w L_s + r B_s holds while Y = T/B_s does not.
    let params = durable(0.5, 0.5, 1);
    let eq = interior(params.clone());
    assert_eq!(eq.u, 1.0);
    close("x*", eq.x_star, G4_F_X_STAR);
    close("Y", eq.y, G4_F_Y);
    let cs = eq.cost_system.expect("u = 1 reports the cost system");
    close("P_s = w L_s + r B_s", eq.v * cs.l_s + cs.b_s, eq.p_s);
    // The shares are the price side's: lambda gamma(x*) / (1 - a), with no delta in them.
    let (phi_w, phi_r) = (eq.phi_w.expect("u = 1"), eq.phi_r.expect("u = 1"));
    close("phi_w", phi_w, G4_F_PHI_W);
    close("phi_r", phi_r, G4_F_PHI_R);
    close(
        "phi_w = lambda gamma(x*) / (1 - a)",
        params.lam * eq.gamma_star / (1.0 - params.a),
        phi_w,
    );
    close(
        "phi_w = w lt_m / p_m",
        eq.v * cs.lambda_tilde[1] / eq.p_m,
        phi_w,
    );
    close("phi_r = r bt_m / p_m", cs.b_tilde[1] / eq.p_m, phi_r);
    let gap = (params.land / cs.b_s - eq.y).abs() / eq.y;
    assert!(
        gap > 1e-3,
        "Y = T/B_s should fail at delta < 1, gap {gap:e}"
    );
    let s_k = 1.0 - params.a * params.delta;
    let b_s_clearing = params.space + params.b * params.delta * eq.j_star / s_k;
    close(
        "Y = T / (h + b delta J / (1 - a delta))",
        params.land / b_s_clearing,
        eq.y,
    );
}

#[test]
fn a_higher_required_return_automates_fewer_tasks() {
    // check_macro.py A5 (:140-143 at 31b3482): fewer tasks automated, a higher v and
    // labour share.
    let low = interior(appendix_b());
    let high = interior(durable(0.05, 1.0, 1));
    assert!(high.x_star < low.x_star);
    assert!(high.v > low.v);
    assert!(high.labor_share > low.labor_share);
}

#[test]
fn near_the_viability_edge() {
    // Case G: u a = 1 - 2^-19 and D(x*) = 8.4e-7, so the prices are about 1/D = 1.2e6
    // times b. Every input is dyadic, so the f64 inputs are the golden's exactly. D is
    // formed as (1 - u a) - u lambda gamma; the form 1 - u (a + lambda gamma) rounds a sum
    // near 1 first and misses these goldens by up to 1e-10 (review of 2026-09-25).
    let base = appendix_b();
    let params = Params {
        a: 1.0 - pow2(-19),
        lam: pow2(-20),
        b: 0.375,
        schedule: PowerSchedule {
            g0: 0.5,
            g1: 1.0,
            ..base.schedule
        },
        rho: 0.5,
        delta: 0.5,
        ..base
    };
    // generate.py's decimals, 0.9999980926513671875 and 0.00000095367431640625.
    assert_eq!(params.a, 1.0 - 1.9073486328125e-6);
    assert_eq!(params.lam, 9.5367431640625e-7);
    let e = economy(params.clone());
    let eq = interior(params);
    assert_eq!(eq.u, 1.0);
    close("x*", eq.x_star, G4_G_X_STAR);
    close("D(x*)", e.at(eq.x_star).d, G4_G_D_STAR);
    close("p_m", eq.p_m, G4_G_P_M);
    close("v", eq.v, G4_G_V);
    close("P_s", eq.p_s, G4_G_P_S);
    close("I", eq.income, G4_G_INCOME);
    let r = eq.residuals;
    for (name, value) in [("income", r.income), ("user cost", r.user_cost)] {
        assert!(value <= FULL, "{name} residual {value:e}");
    }
}
