//! m3: the two-recipe economy (docs/unit-1c.md §3.3, M3): check_dynamics' machine in SSRN
//! Appendix B's closure, with interest (ρ = 0.05) and without (M3z), its income identity with
//! interest (PLAN Phase 1's gate), and its corners, which are unit-1a economies.

use oracle::{Economy, Params, Recipe, Regime};

use crate::goldens_1c::*;
use crate::support::*;
use crate::support_1c::*;

#[test]
fn goldens() {
    for (what, rho, want) in [
        (
            "M3",
            0.05,
            [
                M3_X_STAR,
                M3_ONE_MINUS_X_STAR,
                M3_GAMMA_STAR,
                M3_V,
                M3_U,
                M3_OMEGA,
                M3_P_M,
                M3_OPERATING,
                M3_BUILD,
                M3_P_GOOD,
                M3_P_S,
                M3_Y,
                M3_SERVICES,
                M3_N_A,
                M3_FINAL_HOURS,
                M3_MACHINE_HOURS,
                M3_INCOME,
                M3_LAMBDA_TILDE,
                M3_LAMBDA_TILDE_Q,
                M3_B_TILDE,
                M3_B_TILDE_Q,
                M3_LABOR_SHARE,
                M3_REAL_WAGE,
            ],
        ),
        (
            "M3z",
            0.0,
            [
                M3Z_X_STAR,
                M3Z_ONE_MINUS_X_STAR,
                M3Z_GAMMA_STAR,
                M3Z_V,
                M3Z_U,
                M3Z_OMEGA,
                M3Z_P_M,
                M3Z_OPERATING,
                M3Z_BUILD,
                M3Z_P_GOOD,
                M3Z_P_S,
                M3Z_Y,
                M3Z_SERVICES,
                M3Z_N_A,
                M3Z_FINAL_HOURS,
                M3Z_MACHINE_HOURS,
                M3Z_INCOME,
                M3Z_LAMBDA_TILDE,
                M3Z_LAMBDA_TILDE_Q,
                M3Z_B_TILDE,
                M3Z_B_TILDE_Q,
                M3Z_LABOR_SHARE,
                M3Z_REAL_WAGE,
            ],
        ),
    ] {
        let eq = interior_1c(m3(rho));
        let t = &eq.types[0];
        let got = [
            eq.x_star,
            eq.one_minus_x_star,
            eq.gamma_star,
            eq.v,
            t.user_cost,
            t.wealth_factor,
            t.price,
            t.operating_cost,
            t.build_cost,
            eq.categories[0].price,
            eq.p_s,
            eq.y,
            t.services,
            eq.n_a,
            eq.final_hours,
            eq.machine_hours,
            eq.income,
            t.lambda_tilde,
            t.lambda_tilde_q,
            t.b_tilde,
            t.b_tilde_q,
            eq.labor_share,
            eq.real_wage,
        ];
        for (i, (g, w)) in got.iter().zip(want).enumerate() {
            close(&format!("{what} output {i}"), *g, w);
        }
        let e = economy_1c(m3(rho));
        check_identities_1c(&e, &eq);
        check_fork_and_bounds_1c(&e, &eq);
    }
    let eq = interior_1c(m3(0.05));
    close("interest", eq.interest, M3_INTEREST);
    close("W", eq.types[0].wealth, M3_WEALTH);
    close("capital share", eq.capital_share, M3_CAPITAL_SHARE);
    // Space is the pass-through row: p = r = 1.
    assert_eq!(eq.categories[1].price, 1.0);
}

#[test]
fn price_and_clearing_totals() {
    // docs/unit-1c.md §4.5: with ρ > 0 the price side's totals (at u) exceed the clearing
    // side's (at δ); at ρ = 0, u = δ exactly and they are bit-equal.
    let eq = interior_1c(m3(0.05));
    let t = &eq.types[0];
    assert!(t.lambda_tilde > t.lambda_tilde_q && t.b_tilde > t.b_tilde_q);
    assert!(eq.l_s > eq.l_s_q && eq.b_s > eq.b_s_q);
    let eq = interior_1c(m3(0.0));
    let t = &eq.types[0];
    assert_eq!(t.user_cost, 0.1);
    assert_eq!(t.lambda_tilde.to_bits(), t.lambda_tilde_q.to_bits());
    assert_eq!(t.b_tilde.to_bits(), t.b_tilde_q.to_bits());
    assert_eq!(eq.interest, 0.0);
    close("M3z totals", t.lambda_tilde, M3Z_LAMBDA_TILDE);
}

#[test]
fn income_with_interest() {
    // docs/unit-1c.md §4.6, SSRN App. C and check_dynamics L1-L2: I = vN_a + T + Σρω_kV_kX_k
    // = Y·P_s = pᵀf = Σ_j p_jz_jY, interest = Σ(u_k − δ_k)V_kX_k, and the baskets add to Y.
    let params = m3(0.05);
    let eq = interior_1c(params.clone());
    let t = &eq.types[0];
    let (rho, delta) = (0.05, 0.1);
    let interest = rho * t.wealth_factor * t.build_cost * t.services;
    close("interest = rho omega V X", eq.interest, interest);
    close(
        "interest = (u - delta) V X",
        (t.user_cost - delta) * t.build_cost * t.services,
        eq.interest,
    );
    close(
        "I = v N_a + T + interest",
        eq.v * eq.n_a + params.land + interest,
        eq.income,
    );
    close("I = Y P_s", eq.y * eq.p_s, eq.income);
    let spending: f64 = eq.categories.iter().map(|c| c.price * c.output).sum();
    close("I = sum p_j z_j Y", spending, eq.income);
    // p'f with f = (I − A^q')y: the good's and space's final outputs, and the machine row's
    // net output, which is 0.
    let net_machine = t.services - t.task_services - (0.5 + delta * 0.1) * t.services;
    near("machine row of f", net_machine, 0.0, FULL * t.services);
    let pf = eq.categories[0].price * eq.categories[0].output
        + eq.categories[1].price * eq.categories[1].output
        + t.price * net_machine;
    close("I = p'f", pf, eq.income);
    close("baskets", eq.worker_baskets + eq.provider_baskets, eq.y);
    close(
        "shares",
        eq.labor_share + eq.capital_share + params.land / eq.income,
        1.0,
    );
    assert!(eq.capital_share > 0.0);
    check_identities_1c(&economy_1c(params), &eq);
}

#[test]
fn corners() {
    // check_dynamics R4 and R5: M3's operating-only form is 1a's flow economy with
    // (a, λ, b) = (0.5, 0.1, 0.2) at u = 1, and its build-only form is 1a's durable economy
    // with (0.1, 0.2, 0.02) at (ρ, δ, J) = (0.05, 0.1, 3); both bit for bit.
    let one = |a, lam, b, rho, delta, build_lag| Params {
        a,
        lam,
        b,
        schedule: linear(1.0, 1.0, 2.0),
        rho,
        delta,
        build_lag,
        ..appendix_b()
    };
    let mut flow = m3(0.05);
    flow.machine_types[0].build = Recipe::zero(1);
    let eq = interior_1c(flow.clone());
    let a = interior(one(0.5, 0.1, 0.2, 0.0, 1.0, 1));
    for (name, c, want) in [
        ("x*", eq.x_star, a.x_star),
        ("1 - x*", eq.one_minus_x_star, a.one_minus_x_star),
        ("v", eq.v, a.v),
        ("p_m", eq.types[0].price, a.p_m),
        ("O = V_m", eq.types[0].operating_cost, a.v_m),
        ("P_s", eq.p_s, a.p_s),
        ("Y", eq.y, a.y),
        ("K", eq.types[0].services, a.k),
        ("N_a", eq.n_a, a.n_a),
        ("income", eq.income, a.income),
        (
            "res_user_cost",
            eq.residuals.user_cost,
            a.residuals.user_cost,
        ),
        ("res_services", eq.residuals.services, a.residuals.services),
    ] {
        assert_eq!(c.to_bits(), want.to_bits(), "{name}");
    }
    assert_eq!((eq.interest, eq.types[0].build_cost), (0.0, 0.0));
    check_identities_1c(&economy_1c(flow), &eq);
    // The build-only form: here labour holds no contestable task (f(1) > 0), in both.
    let mut capital = m3(0.05);
    capital.machine_types[0].operating = Recipe::zero(1);
    let typed = economy_1c(capital).solve().unwrap();
    let one_a = Economy::new(one(0.1, 0.2, 0.02, 0.05, 0.1, 3))
        .unwrap()
        .solve()
        .unwrap();
    match (typed, one_a) {
        (Regime::BoundaryNoMargin { f_at_1: c }, Regime::BoundaryNoMargin { f_at_1: a }) => {
            assert_eq!(c.to_bits(), a.to_bits());
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn single_crossing_on_a_grid() {
    // docs/unit-1c.md §5.5 Lemma 2 with one type: f is nonincreasing, one sign change.
    for rho in [0.0, 0.05] {
        let e = economy_1c(m3(rho));
        assert_eq!(check_regions(&e), 1);
        let eq = interior_1c(m3(rho));
        assert_root_1c(&e, eq.x_star, eq.technique);
    }
}
