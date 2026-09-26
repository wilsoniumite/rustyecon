//! G5's two general instances. The random sets check identities, and identities hold for
//! any γ, h or χ_max; these pin every output of one flow and one durable economy, with
//! every parameter off the paper's values, to 70-digit goldens. In particular k ≠ 1
//! (4.5 and 2.5), h, χ_max and η ≠ 1, and J_b = 2.

use oracle::{Eq1a, Params, PowerSchedule, UniformWorkCost};

use crate::goldens::*;
use crate::support::*;

/// The flow instance: N 5.2, T 12.5, h 0.7, a 0.22, λ 0.08, b 0.55,
/// γ = 2.3 (0.15 + 0.9 x^4.5), χ_max 1.6, (ρ, δ, J_b) = (0, 1, 1).
pub fn flow() -> Params {
    Params {
        workers: 5.2,
        land: 12.5,
        space: 0.7,
        a: 0.22,
        lam: 0.08,
        b: 0.55,
        schedule: PowerSchedule {
            eta: 2.3,
            g0: 0.15,
            g1: 0.9,
            k: 4.5,
        },
        work_cost: UniformWorkCost { chi_max: 1.6 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    }
}

/// The durable instance: the flow instance with k 2.5, ρ 0.04, δ 0.35 and J_b 2, so
/// u = 0.39 · 1.04 = 0.4056. Its fourteen parameters are distinct and none is 1.
pub fn durable() -> Params {
    let base = flow();
    Params {
        schedule: PowerSchedule {
            k: 2.5,
            ..base.schedule
        },
        rho: 0.04,
        delta: 0.35,
        build_lag: 2,
        ..base
    }
}

/// The outputs that both instances pin, with their goldens: (name, field, golden).
fn common(eq: &Eq1a, goldens: [f64; 23]) -> Vec<(&'static str, f64, f64)> {
    let fields = [
        ("x*", eq.x_star),
        ("1 - x*", eq.one_minus_x_star),
        ("gamma(x*)", eq.gamma_star),
        ("J(x*)", eq.j_star),
        ("v", eq.v),
        ("p_m", eq.p_m),
        ("V_m", eq.v_m),
        ("p", eq.p),
        ("P_s", eq.p_s),
        ("Y", eq.y),
        ("K", eq.k),
        ("final hours", eq.final_hours),
        ("machine hours", eq.machine_hours),
        ("N_a", eq.n_a),
        ("N_a / N", eq.participation),
        ("I", eq.income),
        ("interest", eq.interest),
        ("labour share", eq.labor_share),
        ("capital share", eq.capital_share),
        ("w / P_s", eq.real_wage),
        ("N P_s", eq.support_cost),
        ("worker baskets", eq.worker_baskets),
        ("provider baskets", eq.provider_baskets),
    ];
    fields
        .into_iter()
        .zip(goldens)
        .map(|((name, got), want)| (name, got, want))
        .collect()
}

#[test]
fn flow_instance() {
    let eq = interior(flow());
    assert_eq!(eq.u, 1.0);
    let goldens = [
        G5_FLOW_X_STAR,
        G5_FLOW_ONE_MINUS_X_STAR,
        G5_FLOW_GAMMA_STAR,
        G5_FLOW_J_STAR,
        G5_FLOW_V,
        G5_FLOW_P_M,
        G5_FLOW_V_M,
        G5_FLOW_P,
        G5_FLOW_P_S,
        G5_FLOW_Y,
        G5_FLOW_K,
        G5_FLOW_FINAL_HOURS,
        G5_FLOW_MACHINE_HOURS,
        G5_FLOW_N_A,
        G5_FLOW_PARTICIPATION,
        G5_FLOW_INCOME,
        G5_FLOW_INTEREST,
        G5_FLOW_LABOR_SHARE,
        G5_FLOW_CAPITAL_SHARE,
        G5_FLOW_REAL_WAGE,
        G5_FLOW_SUPPORT_COST,
        G5_FLOW_WORKER_BASKETS,
        G5_FLOW_PROVIDER_BASKETS,
    ];
    for (name, got, want) in common(&eq, goldens) {
        close(&format!("{name}, flow"), got, want);
    }
    assert_eq!(eq.funded, G5_FLOW_FUNDED);
    assert_eq!(eq.lemma_b1, G5_FLOW_LEMMA_B1);
    let cs = eq.cost_system.expect("u = 1");
    for (name, got, want) in [
        ("phi_w", eq.phi_w.expect("u = 1"), G5_FLOW_PHI_W),
        ("phi_r", eq.phi_r.expect("u = 1"), G5_FLOW_PHI_R),
        (
            "lambda-tilde good",
            cs.lambda_tilde[0],
            G5_FLOW_LAMBDA_TILDE_GOOD,
        ),
        (
            "lambda-tilde machine",
            cs.lambda_tilde[1],
            G5_FLOW_LAMBDA_TILDE_MACHINE,
        ),
        ("b-tilde good", cs.b_tilde[0], G5_FLOW_B_TILDE_GOOD),
        ("b-tilde machine", cs.b_tilde[1], G5_FLOW_B_TILDE_MACHINE),
        ("L_s", cs.l_s, G5_FLOW_L_S),
        ("B_s", cs.b_s, G5_FLOW_B_S),
    ] {
        close(&format!("{name}, flow"), got, want);
    }
    assert_eq!((cs.lambda_tilde[2], cs.b_tilde[2]), (0.0, 1.0));
}

#[test]
fn durable_instance() {
    let eq = interior(durable());
    close("u, durable", eq.u, G5_DURABLE_U);
    let goldens = [
        G5_DURABLE_X_STAR,
        G5_DURABLE_ONE_MINUS_X_STAR,
        G5_DURABLE_GAMMA_STAR,
        G5_DURABLE_J_STAR,
        G5_DURABLE_V,
        G5_DURABLE_P_M,
        G5_DURABLE_V_M,
        G5_DURABLE_P,
        G5_DURABLE_P_S,
        G5_DURABLE_Y,
        G5_DURABLE_K,
        G5_DURABLE_FINAL_HOURS,
        G5_DURABLE_MACHINE_HOURS,
        G5_DURABLE_N_A,
        G5_DURABLE_PARTICIPATION,
        G5_DURABLE_INCOME,
        G5_DURABLE_INTEREST,
        G5_DURABLE_LABOR_SHARE,
        G5_DURABLE_CAPITAL_SHARE,
        G5_DURABLE_REAL_WAGE,
        G5_DURABLE_SUPPORT_COST,
        G5_DURABLE_WORKER_BASKETS,
        G5_DURABLE_PROVIDER_BASKETS,
    ];
    for (name, got, want) in common(&eq, goldens) {
        close(&format!("{name}, durable"), got, want);
    }
    assert_eq!(eq.funded, G5_DURABLE_FUNDED);
    assert_eq!(eq.lemma_b1, G5_DURABLE_LEMMA_B1);
    assert_eq!((eq.phi_w, eq.phi_r), (None, None));
    assert!(eq.cost_system.is_none());
}
