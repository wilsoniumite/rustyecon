//! e5: idle land at zero rent (docs/unit-1e.md §2.8 and §4.6): 1d's `LaborShort` economies W2,
//! W3 and E6 have their equilibria on the idle stretch, with the pool's wage the numeraire; I4
//! holds free plots on idle land. SSRN A.1 ("unused fixed inputs have zero rent"), App. C;
//! check_enclosure N-i.

use oracle::{Eq1e, LandMarket, Margin, ParcelEconomy, Regime, SolveError};

use rustyecon_core::num;

use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;

fn idle(tag: &str) -> (ParcelEconomy, Box<Eq1e>) {
    let params = match tag {
        "I1" => e0(goodspace(0.25, 0.05, 1.0)),
        "I2" => e0(goodspace(4.0, 0.6, 3.0)),
        "I3" => e0(entrant_trained(8.0, 0.5, 1.0, 1.0)),
        "I4" => i4(0.5),
        _ => unreachable!(),
    };
    checked_1e(params)
}

#[test]
fn idle_goldens() {
    for tag in ["I1", "I2", "I3", "I4"] {
        let (e, eq) = idle(tag);
        let b = &eq.base;
        assert_eq!(eq.land_market, LandMarket::Idle, "{tag}");
        assert_eq!((eq.rent, b.v, b.x_star), (0.0, 1.0, 1.0), "{tag}");
        assert!(eq.land.idle > 0.0, "{tag}");
        // rent income is 0: I = v·N_a + interest (SSRN App. C)
        close(&format!("{tag} I"), b.income, b.wage_bill + b.interest);
        // v/P_s is 1d's ceiling ω_∞
        if tag != "I3" {
            let omega = e.workers().wall_end().omega.unwrap();
            close_to(&format!("{tag} ω_∞"), b.real_wage, omega, 1e-15);
        }
    }
    let (_, i1) = idle("I1");
    for (name, got, want) in [
        ("Y", i1.base.y, I1_Y),
        ("T_m", i1.land.market, I1_T_M),
        ("T_idle", i1.land.idle, I1_IDLE),
        ("N_a", i1.base.n_a, I1_N_A),
        ("P_s", i1.base.p_s, I1_P_S),
        ("v/P_s", i1.base.real_wage, I1_REAL_WAGE),
        ("f_∞", i1.f_end.unwrap(), I1_F_END),
    ] {
        close_to(&format!("I1 {name}"), got, want, 1e-14);
    }
    let (_, i2) = idle("I2");
    for (name, got, want) in [
        ("Y", i2.base.y, I2_Y),
        ("T_m", i2.land.market, I2_T_M),
        ("T_idle", i2.land.idle, I2_IDLE),
        ("N_a", i2.base.n_a, I2_N_A),
        ("P_s", i2.base.p_s, I2_P_S),
        ("v/P_s", i2.base.real_wage, I2_REAL_WAGE),
        ("f_∞", i2.f_end.unwrap(), I2_F_END),
    ] {
        close_to(&format!("I2 {name}"), got, want, 1e-14);
    }
    // N_a = (4/3)·ln(53/18) at W3 (χ_max 3, supply unsaturated at the ceiling)
    close("I2 N_a", i2.base.n_a, 4.0 / 3.0 * num::ln(53.0 / 18.0));
    let (_, i4) = idle("I4");
    for (name, got, want) in [
        ("Y", i4.base.y, I4_Y),
        ("T_m", i4.land.market, I4_T_M),
        ("T_p", i4.land.rented_plots, I4_T_P),
        ("N_a", i4.base.n_a, I4_N_A),
        ("part", i4.base.participation, I4_PARTICIPATION),
        ("e", i4.workers[0].exit_value, I4_EXIT_VALUE),
    ] {
        close_to(&format!("I4 {name}"), got, want, 1e-14);
    }
    // free plots: rent in kind 0, the plots on idle enclosed land
    assert_eq!(i4.home.rent_in_kind, 0.0);
    assert_eq!(i4.plot_rent, 0.0);
}

#[test]
fn the_wall_at_zero_rent() {
    // §2.8: at r = 0 machines cost labour alone, and every delivered machine cost is below
    // labour's at every task: x* = 1 and γ(1)·π < v.
    for tag in ["I1", "I2", "I3", "I4"] {
        let (_, eq) = idle(tag);
        let b = &eq.base;
        assert_eq!(b.margin, Margin::Wall, "{tag}");
        assert!(b.replacement_top < b.v, "{tag}");
        for t in &b.types {
            if let Some(cost) = t.delivered_cost {
                assert!(cost * b.gamma_star < b.v, "{tag}");
            }
        }
        // prices at r = 0 are labour totals: p_j = v·λ̃_j + R_j, the land content unpriced
        for c in &b.categories {
            close_to(
                &format!("{tag} p_j"),
                c.price,
                b.v * c.lambda_tilde + c.reserved_cost,
                1e-14,
            );
        }
        // a good made of land alone is free, its real wage absent from the outputs
        let space = b.categories.len() - 1;
        assert_eq!(b.categories[space].price, 0.0, "{tag}");
        let bits = bits_1e(&eq);
        assert_eq!(bits.get(&format!("cat{space}.real_wage")), Some(&None));
    }
}

#[test]
fn worst_parcels_idle_first() {
    // §2.1's convention: land in use fills the best parcels first. I1's LAND is used 47/60; at
    // I4 HEATH (quality 0.5) idles while FIELDS (1.5) is used 0.5198.
    let (_, i1) = idle("I1");
    close("I1 LAND used", i1.parcels[0].used, 47.0 / 60.0);
    assert_eq!(i1.parcels[0].rent_per_acre, 0.0);
    let (_, fields) = idle("I4");
    close("FIELDS used", fields.parcels[0].used, I4_FIELDS_USED);
    assert_eq!(fields.parcels[1].used, 0.0);
    // the totals are what count: listing HEATH first does not change the equilibrium, only
    // which parcel is reported used
    let mut p = i4(0.5);
    p.parcels.reverse();
    let (_, swapped) = checked_1e(p);
    close("the same Y", swapped.base.y, fields.base.y);
    assert_eq!(swapped.parcels[0].used, 0.0);
    close("FIELDS used", swapped.parcels[1].used, I4_FIELDS_USED);
    // a parcel of quality 0 supplies nothing and always idles
    let mut p = i4(0.5);
    p.parcels.push(enclosed(3.0, 0.0));
    let (_, zero) = checked_1e(p);
    assert_eq!(zero.parcels[2].used, 0.0);
    assert_eq!(zero.base.y.to_bits(), fields.base.y.to_bits());
}

#[test]
fn edge_on_the_idle_stretch() {
    // I3 (1d's E6, 1d §12 item 16): the trained is short at the wall's end with the land fully
    // used; on the idle stretch its reserved demand 0.12·Y falls to N_T, where its supply is
    // vertical and its wage κ_T·ν·P_s clears the pool.
    let (_, eq) = idle("I3");
    let b = &eq.base;
    let t = &b.workers[1];
    assert!(t.edge && !t.pooled);
    close("D_T = N_T", t.reserved_hours, 0.5);
    close("hours N_T", t.hours, 0.5);
    for (name, got, want) in [
        ("Y", b.y, I3_Y),
        ("T_m", eq.land.market, I3_T_M),
        ("κ_T", t.clearing_real_wage, I3_TRAINED_CLEARING),
        ("trained wage", t.wage, I3_TRAINED_WAGE),
        ("P_s", b.p_s, I3_P_S),
        ("entrant hours", b.workers[0].hours, I3_ENTRANT_HOURS),
        ("participation", b.participation, I3_PARTICIPATION),
    ] {
        close_to(&format!("I3 {name}"), got, want, 1e-12);
    }
    // the wall's end is short: f_end is absent
    assert_eq!(eq.f_end, None);
    close("Y = 25/6", b.y, 25.0 / 6.0);
}

#[test]
fn no_market() {
    // I4 with exit (2, 0, 0.5): the exit life is worth p_g·s₀ = 2·18/35 > 1 pool wage at the
    // ceiling, so no one works at any wage.
    let e = economy_1e(i4(2.0));
    match e.solve() {
        Err(SolveError::NoMarket { f_end }) => close("f_∞", f_end, I5_F_END),
        other => panic!("I4 with exit 2 gave {other:?}"),
    }
    let technique = e.workers().machines().envelope().last();
    let rest = e.at_idle(0.0, technique);
    assert_eq!(rest.point.n_s, 0.0);
    assert!(rest.exit_values[0] > 1.0);
    // one below the ceiling: a market
    assert!(matches!(
        economy_1e(i4(1.9)).solve(),
        Ok(Regime::Interior(_))
    ));
}
