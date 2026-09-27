//! d2: the corners in Appendix B's closure (docs/unit-1d.md §3.3's W; §8): 1a's G8 boundary
//! rows solved. The wall (SSRN §3.1, p.6; main.tex:147-162), labour shortage at the wall, the
//! all-human corner, and roots below the bracket.

use oracle::{
    Economy, Margin, Params, Regime, SolveError, UniformWorkCost, WorkerEconomy, BRACKET_LO,
};

use crate::g5_random_economies::SplitMix64;
use crate::goldens::*;
use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1d::*;

#[test]
fn wall_goldens() {
    // W1, λ 0.6: 1a's BoundaryNoMargin (f(1) = +0.719), solved at the wall. A unit that pinned
    // the wage at the top task's replacement value would give γ(1)·π = v(1), about 4.
    let (e, eq) = checked_1d(goodspace(4.0, 0.6, 1.0));
    assert_eq!(eq.margin, Margin::Wall);
    assert_eq!((eq.x_star, eq.one_minus_x_star), (1.0, 0.0));
    assert!(eq.v > eq.replacement_top && eq.g > 1.0);
    let v_line = e.at_with(1.0, 0).v;
    assert!(v_line < 4.1 && eq.v > 3.0 * v_line, "{v_line} {}", eq.v);
    for (name, got, want) in [
        ("v", eq.v, W1_V),
        ("g", eq.g, W1_G),
        ("p_m", eq.types[0].price, W1_P_M),
        ("replacement top", eq.replacement_top, W1_REPLACEMENT_TOP),
        ("P_s", eq.p_s, W1_P_S),
        ("Y", eq.y, W1_Y),
        ("N_a", eq.n_a, W1_N_A),
        ("I", eq.income, W1_INCOME),
        ("labour share", eq.labor_share, W1_LABOR_SHARE),
        ("real wage", eq.real_wage, W1_REAL_WAGE),
        ("provider baskets", eq.provider_baskets, W1_PROVIDER_BASKETS),
        ("f_end", eq.f_end, W1_F_END),
    ] {
        close(name, got, want);
    }
    assert!(!eq.funded);
    // f on the line at 1 is 1a's diagnostic, bit for bit, and 1a's golden.
    let one = Economy::new(Params {
        lam: 0.6,
        ..appendix_b()
    })
    .unwrap()
    .solve()
    .unwrap();
    match one {
        Regime::BoundaryNoMargin { f_at_1 } => {
            assert_eq!(eq.f_line_1.unwrap().to_bits(), f_at_1.to_bits())
        }
        other => panic!("{other:?}"),
    }
    close("f_line_1", eq.f_line_1.unwrap(), G8_LAM06_F_AT_1);
    close("f_line_1 (1d)", eq.f_line_1.unwrap(), W1_F_LINE_1);
    // The machine price at the wall is the recursion at the wall's own wage.
    close(
        "p_m = (lambda v + b)/(1 - a)",
        (0.6 * eq.v + 0.4) / 0.7,
        eq.types[0].price,
    );
    // d is the least pivot of the wage-given system that priced it (§6): with one flow type,
    // no operating recipe and u = 1 its pivots are 1 and 1 − a.
    close("d = 1 - a", eq.d, 1.0 - 0.3);
    // At the wall the technique's closure wage at g = v/π is v, and φ_w = g·λ̃/θ = v·λ̃/p_m.
    close("closure wage", eq.types[0].closure_wage.unwrap(), eq.v);
    close(
        "phi_w",
        eq.phi_w.unwrap(),
        eq.v * eq.types[0].lambda_tilde / eq.types[0].price,
    );
    check_path_1d(&e);
}

/// The one-type wall's closed form (§4.5): ζ = expm1(χ_max·n_D/N), v = ζ·B_s/(1 − ζ·L_s).
fn closed_form(params: &oracle::WorkerParams, eq: &oracle::Eq1d) -> f64 {
    let t = &params.worker_types[0];
    let zeta = rustyecon_core::num::expm1(t.work_cost.chi_max * eq.n_pool / t.workers);
    zeta * eq.b_s / (1.0 - zeta * eq.l_s)
}

#[test]
fn closed_form_at_the_wall() {
    // W1 and B2, and 60 random one-type economies at the wall.
    for params in [goodspace(4.0, 0.6, 1.0), baumol_one(1.0, 4.0, 0.25)] {
        let eq = solved_1d(params.clone());
        assert_eq!(eq.margin, Margin::Wall);
        close("v, closed form", closed_form(&params, &eq), eq.v);
    }
    let mut rng = SplitMix64(930);
    let (mut found, mut draws) = (0, 0);
    while found < 60 {
        assert!(draws < 2_000, "{found} of 60 in {draws} draws");
        draws += 1;
        let params = from_1a(Params {
            workers: rng.uniform(0.5, 6.0),
            land: rng.uniform(2.0, 20.0),
            space: rng.uniform(0.1, 2.0),
            a: rng.uniform(0.05, 0.5),
            b: rng.uniform(0.1, 1.0),
            lam: rng.uniform(0.2, 0.9),
            schedule: appendix_b_schedule(rng.uniform(0.2, 1.5)),
            work_cost: UniformWorkCost {
                chi_max: rng.uniform(0.3, 3.0),
            },
            ..appendix_b()
        });
        let Ok(e) = WorkerEconomy::new(params.clone()) else {
            continue;
        };
        if let Ok(Regime::Interior(eq)) = e.solve() {
            if eq.margin == Margin::Wall {
                found += 1;
                assert!(eq.f_line_1.unwrap() >= 0.0);
                close("v, closed form", closed_form(&params, &eq), eq.v);
                check_identities_1d(&e, &eq);
            }
        }
    }
}

#[test]
fn labor_short() {
    // W2, N 0.25: n_D(1) = 15/47 > N, supply saturates. W3, λ 0.6 and χ_max 3: the pool's
    // real wage reaches its ceiling 1/L_s before supply meets demand.
    for (params, excess, what) in [
        (goodspace(0.25, 0.05, 1.0), W2_EXCESS, "W2"),
        (goodspace(4.0, 0.6, 3.0), W3_EXCESS, "W3"),
    ] {
        let e = economy_1d(params);
        match e.solve() {
            Err(SolveError::LaborShort {
                excess: got,
                reserved,
            }) => {
                close(what, got, excess);
                assert_eq!(reserved, None, "{what}");
                assert_eq!(got.to_bits(), e.wall_end().excess.to_bits());
            }
            other => panic!("{what}: {other:?}"),
        }
        check_path_1d(&e);
        let (_, changes) = sequence_1d(&e);
        assert_eq!(changes, 0);
    }
    let e = economy_1d(goodspace(0.25, 0.05, 1.0));
    close(
        "W2 f_line_1",
        e.at_with(1.0, 0).excess_demand(),
        W2_F_LINE_1,
    );
    close(
        "W2 f_line_1 (1a)",
        e.at_with(1.0, 0).excess_demand(),
        G8_N025_F_AT_1,
    );
    close("15/47 - 0.25", W2_EXCESS, 15.0 / 47.0 - 0.25);
    let e = economy_1d(goodspace(4.0, 0.6, 3.0));
    close("W3 omega_end", e.wall_end().omega.unwrap(), W3_OMEGA_END);
    close("35/18", W3_OMEGA_END, 35.0 / 18.0);
    let message = SolveError::LaborShort {
        excess: 0.5,
        reserved: Some(1),
    }
    .to_string();
    assert!(message.contains("worker type 1"), "{message}");
}

#[test]
fn all_human_goldens() {
    // W4, N 20, χ_max 0.05: 1a's NoInteriorAtZero, solved at the all-human corner. A unit that
    // stopped at lo with the margin's wage would give v(lo), about 0.116.
    let (e, eq) = checked_1d(goodspace(20.0, 0.05, 0.05));
    assert_eq!(eq.margin, Margin::AllHuman);
    assert_eq!((eq.x_star, eq.one_minus_x_star), (0.0, 1.0));
    assert!(eq.v < eq.replacement_bottom);
    assert!(e.at_with(BRACKET_LO, 0).v > 4.0 * eq.v);
    assert_eq!(eq.types[0].services, 0.0);
    assert_eq!((eq.interest, eq.m_s), (0.0, 0.0));
    for (name, got, want) in [
        ("v", eq.v, W4_V),
        (
            "replacement bottom",
            eq.replacement_bottom,
            W4_REPLACEMENT_BOTTOM,
        ),
        ("pi", eq.types[0].price, W4_PI),
        ("P_s", eq.p_s, W4_P_S),
        ("Y", eq.y, W4_Y),
        ("N_a", eq.n_a, W4_N_A),
        ("I", eq.income, W4_INCOME),
        ("f_line_0", eq.f_line_0.unwrap(), W4_F_LINE_0),
        ("f_line_lo", eq.f_line_lo.unwrap(), W4_F_LINE_LO),
        ("f_line_lo (1a)", eq.f_line_lo.unwrap(), G8_N20_F_AT_LO),
    ] {
        close(name, got, want);
    }
    // The good's price is at its upper bound, the all-human method, bit for bit.
    let good = &eq.categories[0];
    assert_eq!(
        good.price.to_bits(),
        (eq.v * good.l_bar + good.chain_land + good.reserved_cost).to_bits()
    );
    check_path_1d(&e);
}

#[test]
fn roots_below_the_bracket() {
    // W5: T/h − N = 5e-12 with χ_max 1e-3 (the doubles); W6: a = 1 − 2^-53, λ 0. Both are
    // 1a's NoInteriorAtZero with a root in (0, 1e-12), found by bisection on bit patterns.
    let w5 = Params {
        workers: 10.0,
        land: 10.000000000005,
        work_cost: UniformWorkCost { chi_max: 1e-3 },
        ..appendix_b()
    };
    let w6 = Params {
        a: 1.0 - f64::EPSILON / 2.0,
        lam: 0.0,
        ..appendix_b()
    };
    for (what, params, x_gold, f0, flo, rest) in [
        (
            "W5",
            w5.clone(),
            W5_X_STAR,
            W5_F_LINE_0,
            W5_F_LINE_LO,
            vec![("v", W5_V), ("P_s", W5_P_S), ("Y", W5_Y), ("N_a", W5_N_A)],
        ),
        (
            "W6",
            w6,
            W6_X_STAR,
            W6_F_LINE_0,
            W6_F_LINE_LO,
            vec![("v", W6_V), ("Y", W6_Y), ("N_a", W6_N_A)],
        ),
    ] {
        let (_, eq) = checked_1d(from_1a(params));
        assert_eq!(eq.margin, Margin::Contestable, "{what}");
        assert!(eq.x_star > 0.0 && eq.x_star < BRACKET_LO, "{what}");
        assert!(eq.bisection_steps <= 64, "{what}: {}", eq.bisection_steps);
        // x* is resolved to about 2^-52·n_D/|f'| absolute (§5.5).
        let slope = (f0 - flo) / BRACKET_LO;
        let tolerance = 4.0 * f64::EPSILON * eq.n_pool / slope;
        near(&format!("{what} x*"), eq.x_star, x_gold, tolerance);
        // f on the line is n_D − n_S with n_D about 10 at both points: good to a few ulps of
        // n_D absolute, which is 1.6e-4 of W5's f.
        let absolute = 8.0 * f64::EPSILON * eq.n_pool;
        near(
            &format!("{what} f_line_0"),
            eq.f_line_0.unwrap(),
            f0,
            absolute,
        );
        near(
            &format!("{what} f_line_lo"),
            eq.f_line_lo.unwrap(),
            flo,
            absolute,
        );
        for (name, want) in rest {
            let got = match name {
                "v" => eq.v,
                "P_s" => eq.p_s,
                "Y" => eq.y,
                _ => eq.n_a,
            };
            close(&format!("{what} {name}"), got, want);
        }
    }
    close("W6 f_line_lo (1a)", W6_F_LINE_LO, G8_A_NEAR_1_F_AT_LO);
    // W5's inputs are the doubles: the decimal economy's root is 8.9e-5 away (§3.3), far
    // outside the tolerance above.
    assert!((W5_X_STAR - W5_X_STAR_DECIMAL).abs() > 8e-5 * W5_X_STAR);
}
