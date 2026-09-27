//! e4: the commons (docs/unit-1e.md §2.7 and §4.4): G1's economy with exit (0.5, 0, 0.1) and a
//! commons that the equilibrium does not fill (K1), fills (K2), or none (K3); WASTE enclosed by
//! law (K4) and recut (K5). main.tex:375-385; check_enclosure N-i and N-v.

use oracle::{Branch, Eq1e, ExitLand, LandMarket};

use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1e::*;

fn goldens(tag: &str, eq: &Eq1e, want: [f64; 14]) {
    let b = &eq.base;
    let got = [
        b.x_star,
        b.v,
        b.p_s,
        b.y,
        b.n_a,
        b.participation,
        b.income,
        eq.q,
        eq.coverage,
        eq.exit_good_price,
        eq.plot_rent,
        eq.land.commons_occupied,
        eq.land.rented_plots,
        eq.land.market,
    ];
    let names = [
        "x*", "v", "P_s", "Y", "N_a", "part", "I", "q", "κ", "p_g", "r_o", "T_oc", "T_p", "T_m",
    ];
    for ((name, g), w) in names.iter().zip(got).zip(want) {
        if w == 0.0 {
            assert_eq!(g, 0.0, "{tag} {name}");
        } else {
            close_to(&format!("{tag} {name}"), g, w, 1e-13);
        }
    }
}

#[test]
fn commons_goldens() {
    #[rustfmt::skip]
    let table = [
        ("K1", k1(), ExitLand::Commons, [K1_X_STAR, K1_V, K1_P_S, K1_Y, K1_N_A, K1_PARTICIPATION, K1_INCOME, K1_Q, K1_KAPPA, K1_P_G, K1_PLOT_RENT, K1_T_OC, K1_T_P, K1_T_M]),
        ("K2", k2(), ExitLand::Crowded, [K2_X_STAR, K2_V, K2_P_S, K2_Y, K2_N_A, K2_PARTICIPATION, K2_INCOME, K2_Q, K2_KAPPA, K2_P_G, K2_PLOT_RENT, K2_T_OC, K2_T_P, K2_T_M]),
        ("K3", k3(), ExitLand::Enclosed, [K3_X_STAR, K3_V, K3_P_S, K3_Y, K3_N_A, K3_PARTICIPATION, K3_INCOME, K3_Q, K3_KAPPA, K3_P_G, K3_PLOT_RENT, K3_T_OC, K3_T_P, K3_T_M]),
        ("K4", k4(), ExitLand::Enclosed, [K4_X_STAR, K4_V, K4_P_S, K4_Y, K4_N_A, K4_PARTICIPATION, K4_INCOME, K4_Q, K4_KAPPA, K4_P_G, K4_PLOT_RENT, K4_T_OC, K4_T_P, K4_T_M]),
    ];
    for (tag, params, regime, want) in table {
        let (_, eq) = checked_1e(params);
        assert_eq!(eq.exit_land, regime, "{tag}");
        assert_eq!(eq.land_market, LandMarket::Scarce, "{tag}");
        assert_eq!(eq.workers[0].branch, Branch::Plot, "{tag}");
        goldens(tag, &eq, want);
    }
    // K1 (Prop exit (i)): the exit plot is free while suitable land idles, s = s₀; the commons is
    // partly idle, and an open parcel earns no rent, shadow or market.
    let (_, k1) = checked_1e(k1());
    assert_eq!(k1.workers[0].exit_goods, 0.5);
    assert!(k1.land.commons_occupied < 1.0);
    close("WASTE used", k1.parcels[1].used, K1_WASTE_USED);
    assert_eq!(
        (
            k1.parcels[1].rent_per_acre,
            k1.parcels[1].shadow_rent_per_acre
        ),
        (0.0, 0.0)
    );
    // K2: crowded, the shadow rent in (0, r); the type's supply is vertical at N − T_o/h = 1.
    let (_, k2) = checked_1e(k2());
    assert!(k2.plot_rent > 0.0 && k2.plot_rent < 1.0);
    close_to("N − T_o/h", k2.base.n_a, 4.0 - 0.3 / 0.1, 1e-15);
    close("K2 e", k2.workers[0].exit_value, K2_EXIT_VALUE);
    close(
        "the commons' shadow rent in kind",
        k2.home.commons_shadow_rent,
        k2.plot_rent * 0.3,
    );
    // K3 (Prop exit (ii)): the plot pays the ruling rent, s = s(q) = s₀ − q·h.
    let (_, k3) = checked_1e(k3());
    close("K3 s(q)", k3.workers[0].exit_goods, K3_EXIT_GOODS);
    close(
        "s(q) = s0 - q h",
        k3.workers[0].exit_goods,
        0.5 - k3.q * 0.1,
    );
    assert_eq!(
        k3.land.market.to_bits(),
        (10.0 - k3.land.rented_plots).to_bits()
    );
    close(
        "the rent in kind",
        k3.home.rent_in_kind,
        k3.land.rented_plots,
    );
    // A commons too small for the plots at the market rent: it is full and the rest is rented,
    // each type's plots shared between the two in proportion (§12).
    let (_, spill) = checked_1e(commons(vec![enclosed(10.0, 1.0), open(0.1, 1.0)]));
    assert_eq!(spill.exit_land, ExitLand::Enclosed);
    assert_eq!(spill.land.commons_occupied, 0.1);
    let w = &spill.workers[0];
    close(
        "the rented share",
        w.rented_land,
        w.plot_land * spill.land.rented_plots / (spill.land.rented_plots + 0.1),
    );
    assert!(w.rented_land < w.plot_land);
}

#[test]
fn enclosure_manufactures_labour() {
    // main.tex:376 (iii) and :385, check_enclosure N-v's sign: enclosing the commons by price
    // (K1 → K2 → K3) and by law (K1 → K4, T up 10%) raises participation and lowers the wage.
    let eqs: Vec<Box<Eq1e>> = [k1(), k2(), k3(), k4()]
        .into_iter()
        .map(solved_1e)
        .collect();
    for (a, b) in [(0, 1), (1, 2), (0, 3)] {
        assert!(
            eqs[b].base.participation > eqs[a].base.participation,
            "participation {a} → {b}"
        );
        assert!(eqs[b].base.v < eqs[a].base.v, "v {a} → {b}");
    }
    close("K4 T", economy_1e(k4()).enclosed_land(), 11.0);
}

#[test]
fn quality_is_efficiency() {
    // §2.1 and main.tex:692: K5 recuts WASTE as 5 acres of quality 0.2, the same services (5 ×
    // 0.2 is 1.0 in f64 too), and is K1 bit for bit, its parcel's share used included.
    assert_eq!(5.0 * 0.2, 1.0);
    let (a, b) = (solved_1e(k1()), solved_1e(k5()));
    assert_eq!(bits_1e(&a), bits_1e(&b));
    // Rent per acre is r·Q_z on enclosed land, the shadow rent r_o·Q_z on the commons.
    let (_, crowded) = checked_1e(k2());
    assert_eq!(crowded.parcels[0].rent_per_acre, 1.0);
    assert_eq!(crowded.parcels[1].shadow_rent_per_acre, crowded.plot_rent);
    let mut p = k2();
    p.parcels[1] = open(1.5, 0.2);
    let (_, recut) = checked_1e(p);
    assert_eq!(recut.parcels[1].shadow_rent_per_acre, recut.plot_rent * 0.2);
    close("the same shadow rent", recut.plot_rent, crowded.plot_rent);
    let mut p = k3();
    p.parcels[0] = enclosed(5.0, 2.0);
    let (_, rich) = checked_1e(p);
    assert_eq!(rich.parcels[0].rent_per_acre, 2.0);
    close("the same equilibrium", rich.base.x_star, K3_X_STAR);
}

#[test]
fn rented_plots_leave_production() {
    // main.tex:380 and §2.6: plots rented on enclosed land leave production, Y·B^q = T − T_p. A
    // unit that left them in production (A2) has Y 7.797, not K3's 7.586.
    let (_, k3) = checked_1e(k3());
    close(
        "Y B^q = T − T_p",
        k3.base.y * k3.base.b_s_q,
        10.0 - k3.land.rented_plots,
    );
    assert!((k3.base.y - A2_Y).abs() > 0.2);
    assert!((k3.base.x_star - A1_X_STAR).abs() > 3e-3);
    // K1's charged-commons unit (A1) is the wrong answer for K1 and K2 alike.
    let (k1, k2) = (solved_1e(k1()), solved_1e(k2()));
    assert!((k1.base.x_star - A1_X_STAR).abs() > 0.03);
    assert!((k1.base.v - A1_V).abs() > 0.01);
    assert!((k1.base.participation - A1_PARTICIPATION).abs() > 0.05);
    assert!((k2.base.x_star - A1_X_STAR).abs() > 0.01);
}
