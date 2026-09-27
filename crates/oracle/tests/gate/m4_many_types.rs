//! m4: many types (docs/unit-1c.md §3.3, M4): the loom, the engine and power on unit 1b's fork
//! economy with intermediate inputs; the technique path under task automation, the Leontief
//! identities over every produced good, the fork through intermediate inputs, and the tie at
//! the switch.

use oracle::{MachineParams, Regime};

use crate::goldens_1c::*;
use crate::support::*;
use crate::support_1b::FORK_NAMES;
use crate::support_1c::*;

/// M4t: η 0.5, N 8, whose equilibrium is a tie at the switch.
fn m4t() -> MachineParams {
    MachineParams {
        workers: 8.0,
        ..m4(0.5)
    }
}

#[test]
fn three_type_goldens() {
    let eq = interior_1c(m4(1.0));
    assert_eq!(eq.technique, 1, "the engine is at the margin");
    assert_eq!(eq.tie, None);
    assert_eq!(eq.switches.len(), 1);
    let s = eq.switches[0];
    assert_eq!((s.below, s.above), (0, 1));
    close("switch gamma", s.gamma, M4_SWITCH_GAMMA);
    close("switch x", s.x, M4_SWITCH_X);
    for (name, got, want) in [
        ("x*", eq.x_star, M4_X_STAR),
        ("v", eq.v, M4_V),
        ("P_s", eq.p_s, M4_P_S),
        ("Y", eq.y, M4_Y),
        ("N_a", eq.n_a, M4_N_A),
        ("I", eq.income, M4_INCOME),
        ("interest", eq.interest, M4_INTEREST),
        ("v/P_s", eq.real_wage, M4_REAL_WAGE),
        ("L_s", eq.l_s, M4_L_S),
        ("B_s", eq.b_s, M4_B_S),
        ("L_s^q", eq.l_s_q, M4_L_S_Q),
        ("B_s^q", eq.b_s_q, M4_B_S_Q),
    ] {
        close(name, got, want);
    }
    let want = [
        [
            M4_LOOM_U,
            M4_LOOM_OMEGA,
            M4_LOOM_LAMBDA_TILDE,
            M4_LOOM_B_TILDE,
            M4_LOOM_LAMBDA_TILDE_Q,
            M4_LOOM_B_TILDE_Q,
            M4_LOOM_P,
            M4_LOOM_OPERATING,
            M4_LOOM_BUILD,
        ],
        [
            M4_ENGINE_U,
            M4_ENGINE_OMEGA,
            M4_ENGINE_LAMBDA_TILDE,
            M4_ENGINE_B_TILDE,
            M4_ENGINE_LAMBDA_TILDE_Q,
            M4_ENGINE_B_TILDE_Q,
            M4_ENGINE_P,
            M4_ENGINE_OPERATING,
            M4_ENGINE_BUILD,
        ],
        [
            M4_POWER_U,
            M4_POWER_OMEGA,
            M4_POWER_LAMBDA_TILDE,
            M4_POWER_B_TILDE,
            M4_POWER_LAMBDA_TILDE_Q,
            M4_POWER_B_TILDE_Q,
            M4_POWER_P,
            M4_POWER_OPERATING,
            M4_POWER_BUILD,
        ],
    ];
    for (k, (t, w)) in eq.types.iter().zip(want).enumerate() {
        let got = [
            t.user_cost,
            t.wealth_factor,
            t.lambda_tilde,
            t.b_tilde,
            t.lambda_tilde_q,
            t.b_tilde_q,
            t.price,
            t.operating_cost,
            t.build_cost,
        ];
        for (i, (g, w)) in got.iter().zip(w).enumerate() {
            close(&format!("{} field {i}", M4_NAMES[k]), *g, w);
        }
    }
    // Services: the loom is unused, the engine does the tasks, power supplies the engine.
    assert_eq!(eq.types[0].services, 0.0);
    close("X engine", eq.types[1].services, M4_ENGINE_SERVICES);
    close("X power", eq.types[2].services, M4_POWER_SERVICES);
    // Closure wages: the engine's is v; the unused loom's is above it (SSRN A.1's unused
    // methods); power does no tasks and has none.
    close(
        "engine's closure wage",
        eq.types[1].closure_wage.unwrap(),
        M4_ENGINE_CLOSURE_WAGE,
    );
    close(
        "loom's closure wage",
        eq.types[0].closure_wage.unwrap(),
        M4_LOOM_CLOSURE_WAGE,
    );
    assert!(eq.types[0].closure_wage.unwrap() > eq.v);
    assert_eq!(eq.types[2].closure_wage, None);
    for (j, c) in eq.categories.iter().enumerate() {
        let (p, w) = [
            (M4_MANUFACTURES_P, M4_MANUFACTURES_REAL_WAGE),
            (M4_FOOD_P, M4_FOOD_REAL_WAGE),
            (M4_CARE_P, M4_CARE_REAL_WAGE),
            (M4_SHELTER_P, M4_SHELTER_REAL_WAGE),
        ][j];
        close(&format!("p of {}", FORK_NAMES[j]), c.price, p);
        close(&format!("v/p of {}", FORK_NAMES[j]), c.real_wage, w);
    }
}

#[test]
fn task_automation_path() {
    // docs/unit-1c.md §7's η table: as η falls the switch moves up the line and v falls; the
    // loom is unused at x* and its closure wage is above v at each point.
    let (mut last_switch, mut last_v) = (0.0, f64::INFINITY);
    for (eta, switch, x_star, v) in [
        (2.0, M4_ETA2_SWITCH_X, M4_ETA2_X_STAR, M4_ETA2_V),
        (1.0, M4_ETA1_SWITCH_X, M4_ETA1_X_STAR, M4_ETA1_V),
        (0.5, M4_ETA05_SWITCH_X, M4_ETA05_X_STAR, M4_ETA05_V),
    ] {
        let eq = interior_1c(m4(eta));
        close(&format!("switch at eta {eta}"), eq.switches[0].x, switch);
        close(&format!("x* at eta {eta}"), eq.x_star, x_star);
        close(&format!("v at eta {eta}"), eq.v, v);
        assert_eq!(eq.technique, 1);
        assert_eq!(eq.types[0].services, 0.0);
        assert!(eq.types[0].closure_wage.unwrap() > eq.v);
        assert!(eq.switches[0].x > last_switch && eq.v < last_v);
        (last_switch, last_v) = (eq.switches[0].x, eq.v);
        let e = economy_1c(m4(eta));
        check_identities_1c(&e, &eq);
        check_fork_and_bounds_1c(&e, &eq);
    }
}

#[test]
fn leontief_identities() {
    // SSRN eq 3, A.1 and App. C over the C + K rows, at M4 and at the tie M4t: p = Ap + λv + b,
    // f = (I − A^q')y = (Yz, 0), N_a = λ^q'y, T = b^q'y and p'f = vN_a + T + interest, each by
    // multiplication (support_1c::check_identities_1c).
    for params in [m4(1.0), m4t(), m4(2.0)] {
        let e = economy_1c(params.clone());
        let eq = interior_1c(params);
        check_identities_1c(&e, &eq);
        assert!(eq.residuals.leontief_price <= FULL && eq.residuals.leontief_quantity <= FULL);
    }
}

#[test]
fn fork_through_intermediate_inputs() {
    // docs/unit-1c.md §4.4: both fork forms, the bounds and the pair with chain totals, for
    // every category; ŷ, L̄ and b̄ through the chain.
    let e = economy_1c(m4(1.0));
    for (j, (y, l, b)) in [
        (
            M4_MANUFACTURES_YHAT,
            M4_MANUFACTURES_L_BAR,
            M4_MANUFACTURES_B_BAR,
        ),
        (M4_FOOD_YHAT, M4_FOOD_L_BAR, M4_FOOD_B_BAR),
        (M4_CARE_YHAT, M4_CARE_L_BAR, M4_CARE_B_BAR),
        (M4_SHELTER_YHAT, M4_SHELTER_L_BAR, M4_SHELTER_B_BAR),
    ]
    .into_iter()
    .enumerate()
    {
        close("y-hat", e.basket_outputs()[j], y);
        close("L-bar", e.all_human_hours()[j], l);
        if b == 0.0 {
            assert_eq!(e.chain_land()[j], 0.0);
        } else {
            close("b-bar", e.chain_land()[j], b);
        }
    }
    // Manufactures has no land even through the chain: its wage is floored by 1/L̄ alone.
    let eq = interior_1c(m4(1.0));
    let manufactures = &eq.categories[0];
    assert!(manufactures.real_wage >= 1.0 / manufactures.l_bar);
    // Food, care and shelter carry land through the chain above their direct land.
    let fork = crate::support_1b::fork_economy();
    for j in 1..4 {
        assert!(e.chain_land()[j] >= fork.categories[j].direct_land);
    }
    assert!(e.chain_land()[2] > fork.categories[2].direct_land);
    check_fork_and_bounds_1c(&e, &eq);
    for params in [m4t(), m4(0.5), m4(2.0)] {
        let e = economy_1c(params.clone());
        check_fork_and_bounds_1c(&e, &interior_1c(params));
    }
}

#[test]
fn power_does_no_tasks() {
    // docs/unit-1c.md §2.4: θ = 0 marks a type that supplies other machines only. Power's
    // services equal the engine's and its own demand for them at the clearing coefficients.
    let params = m4(1.0);
    let eq = interior_1c(params.clone());
    let power = &eq.types[2];
    assert_eq!(power.task_services, 0.0);
    assert_eq!((power.delivered_cost, power.closure_wage), (None, None));
    let t = &params.machine_types;
    let demand: f64 = (0..3)
        .map(|l| {
            (t[l].operating.machines[2] + t[l].delta * t[l].build.machines[2])
                * eq.types[l].services
        })
        .sum();
    close("power's services", power.services, demand);
    assert!(power.services > 0.0 && power.hours > 0.0 && power.land > 0.0);
    // Its price is its recipes' cost at the equilibrium prices.
    close(
        "power's price",
        power.price,
        power.operating_cost + power.user_cost * power.build_cost,
    );
}

#[test]
fn tie_goldens() {
    // docs/unit-1c.md §4.7 and §7, M4t: the equilibrium sits on the switch, the loom below
    // and the engine above deliver tasks at the same cost, and labour clears by the split.
    let params = m4t();
    let e = economy_1c(params.clone());
    let eq = interior_1c(params);
    let tie = eq.tie.expect("a tie");
    assert_eq!((eq.technique, tie.above), (0, 1));
    assert_eq!(eq.x_star, eq.switches[0].x);
    assert_eq!(eq.bisection_steps, 0);
    for (name, got, want) in [
        ("x*", eq.x_star, M4T_X_STAR),
        ("gamma", tie.gamma, M4T_GAMMA),
        ("sigma", tie.share, M4T_SHARE),
        ("v", eq.v, M4T_V),
        ("P_s", eq.p_s, M4T_P_S),
        ("Y", eq.y, M4T_Y),
        ("N_a", eq.n_a, M4T_N_A),
        ("I", eq.income, M4T_INCOME),
        ("interest", eq.interest, M4T_INTEREST),
        ("L_s^q", eq.l_s_q, M4T_L_S_Q),
        ("B_s^q", eq.b_s_q, M4T_B_S_Q),
        ("X loom", eq.types[0].services, M4T_LOOM_SERVICES),
        ("X engine", eq.types[1].services, M4T_ENGINE_SERVICES),
        ("X power", eq.types[2].services, M4T_POWER_SERVICES),
    ] {
        close(name, got, want);
    }
    // The excess demand on each side of the switch, at the switch point.
    close(
        "f under the loom",
        e.at_with(eq.x_star, 0).excess_demand(),
        M4T_F_LOOM,
    );
    close(
        "f under the engine",
        e.at_with(eq.x_star, 1).excess_demand(),
        M4T_F_ENGINE,
    );
    // The two delivered costs are equal, and labour clears.
    close(
        "delivered costs",
        eq.types[0].delivered_cost.unwrap(),
        eq.types[1].delivered_cost.unwrap(),
    );
    assert!(eq.residuals.labor <= FULL * eq.n_a);
    check_identities_1c(&e, &eq);
    check_fork_and_bounds_1c(&e, &eq);
}

#[test]
fn tie_moves_with_workers() {
    // Across the tie's range of N, σ falls from 1 to 0 as N rises (more labour supply takes the
    // labour-using loom), and at the ends the tie meets the neighbouring regions' roots.
    let base = m4t();
    let e = economy_1c(base.clone());
    let x_sw = e.switch_points().unwrap()[0];
    let (loom, engine) = (e.at_with(x_sw, 0), e.at_with(x_sw, 1));
    // F at the switch, from the loom's prices (the same as the engine's to rounding).
    let f = loom.n_s / base.workers;
    let (n_engine, n_loom) = (engine.n_d / f, loom.n_d / f);
    assert!(n_engine < 8.0 && 8.0 < n_loom);
    let mut last = 1.0;
    for i in 1..20 {
        let workers = n_engine + (n_loom - n_engine) * i as f64 / 20.0;
        let eq = interior_1c(MachineParams {
            workers,
            ..base.clone()
        });
        let share = eq.tie.expect("inside the range, a tie").share;
        assert!(share < last && share > 0.0, "N {workers}: {share}");
        last = share;
    }
    // Just outside the range the root is in a region, next to the switch, and its outputs
    // meet the tie's at the end: σ → 1 at N_engine, σ → 0 at N_loom.
    for (workers_out, workers_in, share_at_end, technique) in [
        (n_engine * (1.0 - 1e-13), n_engine * (1.0 + 1e-13), 1.0, 1),
        (n_loom * (1.0 + 1e-13), n_loom * (1.0 - 1e-13), 0.0, 0),
    ] {
        let root = interior_1c(MachineParams {
            workers: workers_out,
            ..base.clone()
        });
        let tie = interior_1c(MachineParams {
            workers: workers_in,
            ..base.clone()
        });
        assert_eq!(root.tie, None);
        assert_eq!(root.technique, technique);
        near(
            "sigma at the end",
            tie.tie.unwrap().share,
            share_at_end,
            1e-9,
        );
        for (name, a, b) in [
            ("x*", root.x_star, tie.x_star),
            ("v", root.v, tie.v),
            ("Y", root.y, tie.y),
            ("N_a", root.n_a, tie.n_a),
        ] {
            close_to(name, a, b, FULL);
        }
        // The services move with σ, which moves by about 1e-13 of the range here: against the
        // economy's machine services, not each type's own (the engine's are tiny at σ → 0).
        let scale: f64 = tie.types.iter().map(|t| t.services).sum();
        for k in 0..3 {
            near(
                "X",
                root.types[k].services,
                tie.types[k].services,
                FULL * scale,
            );
        }
    }
}

#[test]
fn single_crossing_per_region() {
    // docs/unit-1c.md §5.5 Lemma 2: within each technique's region f is nonincreasing; the
    // sign sequence changes side once (at the root, or across the switch at the tie).
    for params in [m4(1.0), m4(2.0), m4(0.5), m4t()] {
        let e = economy_1c(params.clone());
        assert_eq!(check_regions(&e), 1, "{params:?}");
        let eq = interior_1c(params);
        if eq.tie.is_none() {
            assert_root_1c(&e, eq.x_star, eq.technique);
        }
    }
    // A regime row that is not interior has no solve to check, but its regions still slope.
    match economy_1c(m4(0.25)).solve().unwrap() {
        Regime::BoundaryNoMargin { f_at_1 } => {
            close("f(1) at eta 0.25", f_at_1, M4_REG_ETA025_F_AT_1)
        }
        other => panic!("{other:?}"),
    }
}
