//! C3: the fork economy (constructed 2026-09-27; docs/unit-1b.md §3.3 and §7), in flow,
//! durable and ρ = 0 versions. Four categories on three segments of the task line:
//! manufactures (no direct land, tasks low on the line), food, care (tasks high on the line)
//! and shelter (mostly land).

use oracle::{CategoryParams, BRACKET_HI, BRACKET_LO};

use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;

/// The durable version: (ρ, δ, J_b) = (0.04, 0.35, 2), u = 0.39 · 1.04 = 0.4056.
fn durable() -> CategoryParams {
    CategoryParams {
        rho: 0.04,
        delta: 0.35,
        build_lag: 2,
        ..fork_economy()
    }
}

/// The ρ = 0 version: (ρ, δ, J_b) = (0, 0.1, 1).
fn zero_interest() -> CategoryParams {
    CategoryParams {
        delta: 0.1,
        ..fork_economy()
    }
}

#[test]
fn flow_goldens() {
    let e = economy_1b(fork_economy());
    let eq = interior_1b(fork_economy());
    assert_eq!(eq.u, 1.0);
    for (name, got, want) in [
        ("x*", eq.x_star, C3_X_STAR),
        ("1 - x*", eq.one_minus_x_star, C3_ONE_MINUS_X_STAR),
        ("gamma(x*)", eq.gamma_star, C3_GAMMA_STAR),
        ("v", eq.v, C3_V),
        ("p_m", eq.p_m, C3_P_M),
        ("P_s", eq.p_s, C3_P_S),
        ("Y", eq.y, C3_Y),
        ("K", eq.k, C3_K),
        ("M_s", eq.m_s, C3_M_S),
        ("H_s", eq.h_s, C3_H_S),
        ("final hours", eq.final_hours, C3_FINAL_HOURS),
        ("machine hours", eq.machine_hours, C3_MACHINE_HOURS),
        ("N_a", eq.n_a, C3_N_A),
        ("N_a / N", eq.participation, C3_PARTICIPATION),
        ("I", eq.income, C3_INCOME),
        ("labour share", eq.labor_share, C3_LABOR_SHARE),
        ("v / P_s", eq.real_wage, C3_REAL_WAGE),
        ("N P_s", eq.support_cost, C3_SUPPORT_COST),
        ("worker baskets", eq.worker_baskets, C3_WORKER_BASKETS),
        ("provider baskets", eq.provider_baskets, C3_PROVIDER_BASKETS),
        ("L_s*", eq.l_star_s, C3_L_STAR_S),
        ("L_s", eq.l_s, C3_L_S),
        ("B_s", eq.b_s, C3_B_S),
        ("v / B_s", eq.rent_ceiling, C3_RENT_CEILING),
        (
            "lambda-tilde machine",
            eq.lambda_tilde_machine,
            C3_LAMBDA_TILDE_MACHINE,
        ),
        ("b-tilde machine", eq.b_tilde_machine, C3_B_TILDE_MACHINE),
    ] {
        close(name, got, want);
    }
    // Flow benchmark: the clearing side's totals are the price side's.
    close("L_s^q = L_s", eq.l_s_q, eq.l_s);
    close("B_s^q = B_s", eq.b_s_q, eq.b_s);
    assert_eq!(eq.b_d, e.basket_direct_land());
    close("B_d", eq.b_d, 1.42);
    close("L-bar_s", e.basket_all_human_hours(), 1.127);
    // The root lies in the middle segment, where food and shelter have tasks: the margin is
    // active.
    assert!(0.4 < eq.x_star && eq.x_star < 0.75);
    assert!(eq.margin_active);
    let per_category = [
        (
            C3_MANUFACTURES_P,
            C3_MANUFACTURES_REAL_WAGE,
            C3_MANUFACTURES_L_STAR,
            C3_MANUFACTURES_H,
            C3_MANUFACTURES_M,
            C3_MANUFACTURES_LAMBDA_TILDE,
            C3_MANUFACTURES_B_TILDE,
            C3_MANUFACTURES_SHARE,
            C3_MANUFACTURES_WAGE_FLOOR,
            C3_MANUFACTURES_PHI_W,
            C3_MANUFACTURES_PHI_R,
        ),
        (
            C3_FOOD_P,
            C3_FOOD_REAL_WAGE,
            C3_FOOD_L_STAR,
            C3_FOOD_H,
            C3_FOOD_M,
            C3_FOOD_LAMBDA_TILDE,
            C3_FOOD_B_TILDE,
            C3_FOOD_SHARE,
            C3_FOOD_WAGE_FLOOR,
            C3_FOOD_PHI_W,
            C3_FOOD_PHI_R,
        ),
        (
            C3_CARE_P,
            C3_CARE_REAL_WAGE,
            C3_CARE_L_STAR,
            C3_CARE_H,
            C3_CARE_M,
            C3_CARE_LAMBDA_TILDE,
            C3_CARE_B_TILDE,
            C3_CARE_SHARE,
            C3_CARE_WAGE_FLOOR,
            C3_CARE_PHI_W,
            C3_CARE_PHI_R,
        ),
        (
            C3_SHELTER_P,
            C3_SHELTER_REAL_WAGE,
            C3_SHELTER_L_STAR,
            C3_SHELTER_H,
            C3_SHELTER_M,
            C3_SHELTER_LAMBDA_TILDE,
            C3_SHELTER_B_TILDE,
            C3_SHELTER_SHARE,
            C3_SHELTER_WAGE_FLOOR,
            C3_SHELTER_PHI_W,
            C3_SHELTER_PHI_R,
        ),
    ];
    for ((name, c), want) in FORK_NAMES.iter().zip(&eq.categories).zip(per_category) {
        let (p, wage, l_star, h, m, lt, bt, share, floor, phi_w, phi_r) = want;
        let tag = |what: &str| format!("{what} of {name}");
        close(&tag("p"), c.price, p);
        close(&tag("v/p"), c.real_wage, wage);
        close(&tag("L*"), c.l_star, l_star);
        if h == 0.0 {
            assert_eq!(c.human, 0.0, "{}", tag("H"));
        } else {
            close(&tag("H"), c.human, h);
        }
        if m == 0.0 {
            assert_eq!(c.machine, 0.0, "{}", tag("M"));
        } else {
            close(&tag("M"), c.machine, m);
        }
        close(&tag("lambda-tilde"), c.lambda_tilde, lt);
        close(&tag("b-tilde"), c.b_tilde, bt);
        close(&tag("share"), c.share, share);
        close(&tag("floor"), c.wage_floor, floor);
        close(&tag("phi_w"), c.phi_w.expect("u = 1"), phi_w);
        close(&tag("phi_r"), c.phi_r.expect("u = 1"), phi_r);
    }
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
}

#[test]
fn bracket_values() {
    let e = economy_1b(fork_economy());
    close("f(1e-12)", e.at(BRACKET_LO).excess_demand(), C3_F_AT_LO);
    let one = e.at(BRACKET_HI);
    close("f(1)", one.excess_demand(), C3_F_AT_1);
    close("n_S(1)", one.n_s, C3_N_S_AT_1);
    close("n_D(1)", one.n_d, C3_N_D_AT_1);
    close("P_s(1)", one.p_s, C3_P_S_AT_1);
    let eq = interior_1b(fork_economy());
    assert_eq!((eq.lemma_b1, eq.funded), (C3_LEMMA_B1, C3_FUNDED));
    assert!(eq.lemma_b1 && eq.funded);
    // SSRN eq 25 as written: both conditions hold, strictly and with room.
    assert!(one.n_s > one.n_d + 0.5);
    assert!(fork_economy().land > fork_economy().workers * one.p_s + 1.0);
}

#[test]
fn durable_goldens() {
    let e = economy_1b(durable());
    let eq = interior_1b(durable());
    close("u", eq.u, 0.39 * 1.04);
    for (name, got, want) in [
        ("x*", eq.x_star, C3D_X_STAR),
        ("1 - x*", eq.one_minus_x_star, C3D_ONE_MINUS_X_STAR),
        ("v", eq.v, C3D_V),
        ("p_m", eq.p_m, C3D_P_M),
        ("P_s", eq.p_s, C3D_P_S),
        ("Y", eq.y, C3D_Y),
        ("K", eq.k, C3D_K),
        ("N_a", eq.n_a, C3D_N_A),
        ("I", eq.income, C3D_INCOME),
        ("interest", eq.interest, C3D_INTEREST),
        ("labour share", eq.labor_share, C3D_LABOR_SHARE),
        ("capital share", eq.capital_share, C3D_CAPITAL_SHARE),
        ("v / P_s", eq.real_wage, C3D_REAL_WAGE),
        (
            "provider baskets",
            eq.provider_baskets,
            C3D_PROVIDER_BASKETS,
        ),
        ("L_s", eq.l_s, C3D_L_S),
        ("B_s", eq.b_s, C3D_B_S),
        ("L_s^q", eq.l_s_q, C3D_L_S_Q),
        ("B_s^q", eq.b_s_q, C3D_B_S_Q),
    ] {
        close(name, got, want);
    }
    // The root lies in the top segment, so 1 - x* is carried there.
    assert!(eq.x_star > 0.75);
    let per_category = [
        (
            C3D_MANUFACTURES_P,
            C3D_MANUFACTURES_REAL_WAGE,
            C3D_MANUFACTURES_LAMBDA_TILDE,
            C3D_MANUFACTURES_B_TILDE,
            C3D_MANUFACTURES_LAMBDA_Q,
            C3D_MANUFACTURES_B_Q,
        ),
        (
            C3D_FOOD_P,
            C3D_FOOD_REAL_WAGE,
            C3D_FOOD_LAMBDA_TILDE,
            C3D_FOOD_B_TILDE,
            C3D_FOOD_LAMBDA_Q,
            C3D_FOOD_B_Q,
        ),
        (
            C3D_CARE_P,
            C3D_CARE_REAL_WAGE,
            C3D_CARE_LAMBDA_TILDE,
            C3D_CARE_B_TILDE,
            C3D_CARE_LAMBDA_Q,
            C3D_CARE_B_Q,
        ),
        (
            C3D_SHELTER_P,
            C3D_SHELTER_REAL_WAGE,
            C3D_SHELTER_LAMBDA_TILDE,
            C3D_SHELTER_B_TILDE,
            C3D_SHELTER_LAMBDA_Q,
            C3D_SHELTER_B_Q,
        ),
    ];
    for ((name, c), (p, wage, lt, bt, lq, bq)) in
        FORK_NAMES.iter().zip(&eq.categories).zip(per_category)
    {
        let tag = |what: &str| format!("{what} of {name}, durable");
        close(&tag("p"), c.price, p);
        close(&tag("v/p"), c.real_wage, wage);
        close(&tag("lambda-tilde"), c.lambda_tilde, lt);
        close(&tag("b-tilde"), c.b_tilde, bt);
        close(&tag("lambda-tilde^q"), c.lambda_tilde_q, lq);
        close(&tag("b-tilde^q"), c.b_tilde_q, bq);
        assert_eq!((c.phi_w, c.phi_r), (None, None), "{}", tag("phi"));
    }
    assert_eq!((eq.phi_w, eq.phi_r), (None, None));
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
}

#[test]
fn rho_zero_is_the_scaled_flow_economy() {
    // unit-1a.md G4's nesting, per category: at rho = 0 the durable economy is the flow
    // economy with (a, lambda, b) scaled by delta (generate_1b.py asserts it at 70 digits).
    // In f64 they round differently, so they are compared at FULL.
    let e = economy_1b(zero_interest());
    let eq = interior_1b(zero_interest());
    for (name, got, want) in [
        ("x*", eq.x_star, C3Z_X_STAR),
        ("v", eq.v, C3Z_V),
        ("Y", eq.y, C3Z_Y),
        ("N_a", eq.n_a, C3Z_N_A),
        ("P_s", eq.p_s, C3Z_P_S),
        ("v / P_s", eq.real_wage, C3Z_REAL_WAGE),
    ] {
        close(name, got, want);
    }
    for (c, want) in eq.categories.iter().zip([
        C3Z_MANUFACTURES_REAL_WAGE,
        C3Z_FOOD_REAL_WAGE,
        C3Z_CARE_REAL_WAGE,
        C3Z_SHELTER_REAL_WAGE,
    ]) {
        close("v/p_j", c.real_wage, want);
    }
    assert_eq!(eq.interest, 0.0);
    let flow = interior_1b(CategoryParams {
        a: 0.03,
        lam: 0.005,
        b: 0.04,
        ..fork_economy()
    });
    for (name, got, want) in [
        ("x*", eq.x_star, flow.x_star),
        ("v", eq.v, flow.v),
        ("p_m", eq.p_m, flow.p_m),
        ("P_s", eq.p_s, flow.p_s),
        ("Y", eq.y, flow.y),
        ("K", eq.k, flow.k),
        ("N_a", eq.n_a, flow.n_a),
        ("I", eq.income, flow.income),
        ("L_s", eq.l_s, flow.l_s),
        ("B_s", eq.b_s, flow.b_s),
        ("L_s^q", eq.l_s_q, flow.l_s_q),
        ("B_s^q", eq.b_s_q, flow.b_s_q),
    ] {
        close(&format!("{name}, durable vs scaled flow"), got, want);
    }
    for (a, b) in eq.categories.iter().zip(&flow.categories) {
        for (name, got, want) in [
            ("p", a.price, b.price),
            ("L*", a.l_star, b.l_star),
            ("lambda-tilde", a.lambda_tilde, b.lambda_tilde),
            ("b-tilde", a.b_tilde, b.b_tilde),
            ("lambda-tilde^q", a.lambda_tilde_q, b.lambda_tilde_q),
            ("b-tilde^q", a.b_tilde_q, b.b_tilde_q),
        ] {
            close(&format!("{name}, durable vs scaled flow"), got, want);
        }
    }
    // At u = delta the price side's totals are the clearing side's.
    close("L_s = L_s^q", eq.l_s, eq.l_s_q);
    close("B_s = B_s^q", eq.b_s, eq.b_s_q);
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
}

#[test]
fn care_sits_on_its_human_bound() {
    // Care's tasks are all above x*: L* = L-bar and p = v L-bar + b bit for bit, and v/p is
    // the lower end of the pair (main.tex eq category-bounds :465, eq fork-pair :469).
    let eq = interior_1b(fork_economy());
    let care = &eq.categories[2];
    assert_eq!(care.machine, 0.0);
    assert_eq!(care.l_star.to_bits(), care.l_bar.to_bits());
    assert_eq!(care.l_bar, 0.25);
    assert_eq!(care.price.to_bits(), (eq.v * care.l_bar + 0.1).to_bits());
    close("v/p = floor", care.real_wage, care.wage_floor);
    assert_eq!(care.b_tilde, 0.1);
    assert_eq!(care.lambda_tilde, 0.25);
    // Without its direct land, care uses no land at all (b-tilde = 0): the pair has no
    // ceiling, and v/p is 1/L-bar = 4, the floor, exactly as main.tex:474 has it.
    let mut landless = fork_economy();
    landless.categories[2].direct_land = 0.0;
    let e = economy_1b(landless.clone());
    let eq = interior_1b(landless);
    let care = &eq.categories[2];
    assert_eq!((care.b_tilde, care.wage_ceiling), (0.0, None));
    assert_eq!(care.wage_floor, 4.0);
    close("v/p = 1/L-bar", care.real_wage, 4.0);
    check_fork_and_bounds(&e, &eq);
}

#[test]
fn manufactures_is_fully_automated() {
    // All of manufactures' tasks lie below x*: no hours by hand, L* = M/gamma(x*), and
    // M = 2 J(0.4) (docs/unit-1b.md §4.1).
    let e = economy_1b(fork_economy());
    let eq = interior_1b(fork_economy());
    let m = &eq.categories[0];
    assert_eq!(m.human.to_bits(), 0.0f64.to_bits());
    assert_eq!(m.l_star.to_bits(), (m.machine / eq.gamma_star).to_bits());
    let j = |x: f64| {
        let s = &e.params().schedule;
        oracle::Schedule::integral(s, x)
    };
    assert_eq!(m.machine, 2.0 * (j(0.4) - j(0.0)));
    // No direct land, yet land through machines: v/p is above 1/L-bar and below v/b-tilde.
    assert!(m.real_wage > 1.0 / m.l_bar && m.real_wage < m.wage_ceiling.unwrap());
}

#[test]
fn durable_price_and_clearing_totals_differ() {
    // With u != delta the price side scales the machine row by u, the clearing side by
    // delta: lambda-tilde^q < lambda-tilde and b-tilde^q < b-tilde strictly for every
    // category with machine services, and equal for those without (SSRN eq 11 against eq
    // 12; docs/unit-1b.md §10).
    let e = economy_1b(durable());
    let eq = interior_1b(durable());
    assert!(eq.u > e.params().delta);
    for (name, c) in FORK_NAMES.iter().zip(&eq.categories) {
        if c.machine > 0.0 {
            assert!(c.lambda_tilde_q < c.lambda_tilde, "{name}");
            assert!(c.b_tilde_q < c.b_tilde, "{name}");
            // The chain b_j <= bt^q <= bt <= p holds strictly here, with room.
            let b_j = e.params().categories[FORK_NAMES.iter().position(|n| n == name).unwrap()]
                .direct_land;
            assert!(b_j < c.b_tilde_q && c.b_tilde < c.price, "{name}");
        } else {
            assert_eq!(c.lambda_tilde_q, c.lambda_tilde, "{name}");
            assert_eq!(c.b_tilde_q, c.b_tilde, "{name}");
        }
    }
    assert!(eq.l_s_q < eq.l_s - 1e-3 && eq.b_s_q < eq.b_s - 1e-3);
}

#[test]
fn single_crossing_on_a_grid() {
    // docs/unit-1b.md §5.3, for the three versions.
    for params in [fork_economy(), durable(), zero_interest()] {
        check_single_crossing_1b(&economy_1b(params));
    }
}

#[test]
fn cost_system_rows_hold() {
    // docs/unit-1b.md §4.6. Rows (categories 1..C, machine services): A has M_j in row j's
    // machine column and u a in the machine row; lambda = (H_1, ..., H_C, u lambda);
    // b = (b_1, ..., b_C, u b). p = A p + lambda v + b row by row (SSRN eq 3), by
    // multiplication, at the double x* (prices are evaluated there).
    for params in [fork_economy(), durable(), zero_interest()] {
        let e = economy_1b(params.clone());
        let eq = interior_1b(params.clone());
        let q = e.at(eq.x_star);
        for (j, cat) in params.categories.iter().enumerate() {
            close(
                &format!("row {j}"),
                q.machine[j] * q.p_m + q.human[j] * q.v + cat.direct_land,
                q.prices[j],
            );
        }
        close(
            "machine row",
            eq.u * params.a * eq.p_m + eq.u * params.lam * eq.v + eq.u * params.b,
            eq.p_m,
        );
    }
    // At rho = 0 (u = delta), with gross outputs y = (Y z, K): f = (I - A^T) y = (Y z, 0),
    // lambda^T y = N_a, b^T y = T and p^T f = v N_a + T (SSRN A.1 and App. C).
    let params = zero_interest();
    let eq = interior_1b(params.clone());
    let u = eq.u;
    assert_eq!(u, params.delta);
    let y: Vec<f64> = eq.categories.iter().map(|c| c.output).collect();
    let net_machine = eq.k
        - eq.categories
            .iter()
            .zip(&y)
            .map(|(c, y)| c.machine * y)
            .sum::<f64>()
        - u * params.a * eq.k;
    near("f_machine = 0", net_machine, 0.0, FULL * eq.k);
    let hours: f64 = eq
        .categories
        .iter()
        .zip(&y)
        .map(|(c, y)| c.human * y)
        .sum::<f64>()
        + u * params.lam * eq.k;
    close("lambda^T y = N_a", hours, eq.n_a);
    let land: f64 = params
        .categories
        .iter()
        .zip(&y)
        .map(|(c, y)| c.direct_land * y)
        .sum::<f64>()
        + u * params.b * eq.k;
    close("b^T y = T", land, params.land);
    let value: f64 = eq.categories.iter().zip(&y).map(|(c, y)| c.price * y).sum();
    close("p^T f = v N_a + T", value, eq.v * eq.n_a + params.land);
}
