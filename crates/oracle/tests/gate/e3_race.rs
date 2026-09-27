//! e3: the race inside equilibria (docs/unit-1e.md §4.9): check_enclosure N-iii and SSRN D.3 in
//! Appendix B's closure with T 100 and exit (1.5, 0, 1), along the automation path. κ =
//! qT/(N(1 + q)), q_enc = 1.5, q* = N/(100 − N) and N_crit = 60 hold at every equilibrium.

use oracle::{Branch, EnclosureSide, Eq1e, ExitLand, LandMarket, Margin};

use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1e::*;

/// Q1-Q5 (docs/unit-1e.md §3.3).
fn q(tag: &str) -> (f64, f64) {
    match tag {
        "Q1" => (50.0, 2.5),
        "Q2" => (60.0, 2.0),
        "Q3" => (80.0, 2.0),
        "Q4" => (80.0, 1.0),
        "Q5" => (80.0, 0.7),
        _ => unreachable!(),
    }
}

fn solve(tag: &str) -> Box<Eq1e> {
    let (people, eta) = q(tag);
    checked_1e(race(people, eta)).1
}

#[test]
fn race_goldens() {
    #[rustfmt::skip]
    let table: [(&str, [f64; 12]); 5] = [
        ("Q1", [Q1_X_STAR, Q1_V, Q1_P_S, Q1_Q, Q1_KAPPA, Q1_Y, Q1_N_A, Q1_T_P, Q1_PROVIDER_BASKETS, Q1_P_G, Q1_PARTICIPATION, Q1_INCOME]),
        ("Q2", [Q2_X_STAR, Q2_V, Q2_P_S, Q2_Q, Q2_KAPPA, Q2_Y, Q2_N_A, Q2_T_P, Q2_PROVIDER_BASKETS, Q2_P_G, Q2_PARTICIPATION, Q2_INCOME]),
        ("Q3", [Q3_X_STAR, Q3_V, Q3_P_S, Q3_Q, Q3_KAPPA, Q3_Y, Q3_N_A, Q3_T_P, Q3_PROVIDER_BASKETS, Q3_P_G, Q3_PARTICIPATION, Q3_INCOME]),
        ("Q4", [Q4_X_STAR, Q4_V, Q4_P_S, Q4_Q, Q4_KAPPA, Q4_Y, Q4_N_A, Q4_T_P, Q4_PROVIDER_BASKETS, Q4_P_G, Q4_PARTICIPATION, Q4_INCOME]),
        ("Q5", [Q5_X_STAR, Q5_V, Q5_P_S, Q5_Q, Q5_KAPPA, Q5_Y, Q5_N_A, Q5_T_P, Q5_PROVIDER_BASKETS, Q5_P_G, Q5_PARTICIPATION, Q5_INCOME]),
    ];
    for (tag, want) in table {
        let eq = solve(tag);
        let b = &eq.base;
        assert_eq!(b.margin, Margin::Contestable, "{tag}");
        assert_eq!(
            (eq.land_market, eq.exit_land),
            (LandMarket::Scarce, ExitLand::Enclosed)
        );
        let got = [
            b.x_star,
            b.v,
            b.p_s,
            eq.q,
            eq.coverage,
            b.y,
            b.n_a,
            eq.land.rented_plots,
            b.provider_baskets,
            eq.exit_good_price,
            b.participation,
            b.income,
        ];
        let names = [
            "x*", "v", "P_s", "q", "κ", "Y", "N_a", "T_p", "provider", "p_g", "part", "I",
        ];
        for ((name, g), w) in names.iter().zip(got).zip(want) {
            if w == 0.0 {
                assert_eq!(g, 0.0, "{tag} {name}");
            } else {
                close(&format!("{tag} {name}"), g, w);
            }
        }
        // `funded` is the provider's market budget: false at Q1-Q4, true only past q* at Q5.
        assert_eq!(b.funded, tag == "Q5", "{tag}");
    }
}

#[test]
fn coverage_identity() {
    // SSRN eq 16 and 28: κ = qT/(N(1 + q)) in Appendix B's basket, and κ ≥ 1 exactly when
    // q ≥ q* = N/(100 − N).
    for tag in ["Q1", "Q2", "Q3", "Q4", "Q5"] {
        let (people, _) = q(tag);
        let eq = solve(tag);
        close_to(
            &format!("{tag} κ"),
            eq.coverage,
            eq.q * 100.0 / (people * (1.0 + eq.q)),
            1e-14,
        );
        let q_star = people / (100.0 - people);
        if tag != "Q2" {
            assert_eq!(eq.coverage >= 1.0, eq.q >= q_star, "{tag}");
        }
    }
    // Q1 is safe with plots still rented; Q4 inside the gap; Q5 past q*.
    assert!(solve("Q1").coverage > 1.0 && solve("Q1").land.rented_plots > 0.0);
    assert!(solve("Q4").coverage < 1.0 && solve("Q4").q > P_Q_ENC);
    assert!(solve("Q5").q > P_Q_STAR_80);
}

#[test]
fn branch_flips_at_q_enc() {
    // main.tex:375-381: below q_enc every exiter rents a plot (Q1: T_p = E, h = 1), above it
    // every exiter is on the floor (Q4, Q5), where the form is the dependence form's.
    let eq = solve("Q1");
    let w = &eq.workers[0];
    assert!(eq.q < w.threshold.unwrap());
    assert_eq!(w.branch, Branch::Plot);
    close("every exiter rents", w.plot_households, w.exiters);
    close("T_p = E", eq.land.rented_plots, w.exiters);
    for tag in ["Q4", "Q5"] {
        let eq = solve(tag);
        let w = &eq.workers[0];
        assert!(eq.q > w.threshold.unwrap(), "{tag}");
        assert_eq!(w.branch, Branch::Floor, "{tag}");
        assert_eq!((w.plot_households, w.exit_value), (0.0, 0.0), "{tag}");
        assert_eq!(eq.land.rented_plots, 0.0, "{tag}");
    }
}

#[test]
fn the_tie_is_enclosure_by_price() {
    // main.tex:854-864 and §4.7: Q2 and Q3 sit at the enclosure point, q = q_enc = 1.5, with a
    // share ψ of the exiters renting. The prices at the tie are fixed by q = q_enc alone, so the
    // two share x*, v and P_s bit for bit; N moves only ψ and quantities. κ = 1 at Q2 (N_crit)
    // and 0.75 at Q3: the gap opens.
    let (q2, q3) = (solve("Q2"), solve("Q3"));
    for (tag, eq, share, below, above, kappa) in [
        ("Q2", &q2, Q2_SHARE, Q2_F_BELOW, Q2_F_ABOVE, Q2_KAPPA),
        ("Q3", &q3, Q3_SHARE, Q3_F_BELOW, Q3_F_ABOVE, Q3_KAPPA),
    ] {
        let tie = eq.enclosure.expect("an enclosure tie");
        assert_eq!(tie.worker, 0);
        close_to(&format!("{tag} ψ"), tie.share, share, 1e-13);
        close_to(&format!("{tag} q"), eq.q, P_Q_ENC, 1e-15);
        close_to(&format!("{tag} κ"), eq.coverage, kappa, 1e-15);
        assert_eq!(eq.workers[0].branch, Branch::Floor);
        close(
            "the renters' land",
            eq.workers[0].plot_land,
            tie.share * eq.workers[0].exiters,
        );
        // the point's two values, from the economy's own enclosure points
        let e = economy_1e(race(q(tag).0, q(tag).1));
        let points = e.enclosure_points().unwrap();
        assert_eq!(points.len(), 1, "{tag}");
        let point = points[0];
        assert_eq!(point.x.to_bits(), eq.base.x_star.to_bits(), "{tag}");
        let (f_b, f_a) = (
            e.at_enclosure(&point, EnclosureSide::Below).excess_demand(),
            e.at_enclosure(&point, EnclosureSide::Above).excess_demand(),
        );
        close_to(&format!("{tag} f below"), f_b, below, 1e-12);
        close_to(&format!("{tag} f above"), f_a, above, 1e-12);
        // f is linear in ψ between them: the tie's ψ is the root
        close_to(
            &format!("{tag} ψ linear"),
            tie.share,
            f_b / (f_b - f_a),
            1e-12,
        );
        let at = e.at_enclosure(&point, EnclosureSide::Share(tie.share));
        assert!(at.excess_demand().abs() <= 1e-12 * at.point.n_d);
    }
    assert_eq!(q2.base.x_star.to_bits(), q3.base.x_star.to_bits());
    assert_eq!(q2.base.v.to_bits(), q3.base.v.to_bits());
    assert_eq!(q2.base.p_s.to_bits(), q3.base.p_s.to_bits());
    close("P_s = 5/3", q2.base.p_s, 5.0 / 3.0);
}

#[test]
fn the_wrong_units_miss() {
    // §3.3: a unit that leaves rented plots in production (A3) solves Q1 at x* 0.8135 with Y
    // 62.09, not 0.7058 and 44.20; one that deflates the rent by P_s (A4) sends Q3 to the wall
    // with 3/224 participating, not to its tie.
    let q1 = solve("Q1");
    assert!((q1.base.x_star - A3_X_STAR).abs() > 0.1);
    assert!((q1.base.y - A3_Y).abs() > 15.0);
    let q3 = solve("Q3");
    assert_eq!(q3.base.margin, Margin::Contestable);
    assert!((q3.base.participation - A4_PARTICIPATION).abs() > 0.3);
    assert!((q3.base.y - A4_Y).abs() > 50.0);
    close("3/224", A4_PARTICIPATION, 3.0 / 224.0);
}
