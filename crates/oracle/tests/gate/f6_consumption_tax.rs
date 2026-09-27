//! f6: the consumption tax (docs/unit-1f.md §2.1, §4.10 (f); SSRN D.5; three-taxes T6): under
//! RentRate a consumption tax t gives the allocation of a payroll tax t/(1 + t), for every exit
//! form and for walled types, because every transfer is in composites at consumer prices and
//! home output replaces purchases at consumer prices.

use oracle::{Government, HouseholdParams};

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1e::*;
use crate::support_1f::*;

/// (τ_w, t_c) and its equivalent (1 − κ_w, 0), each with `rest` of the government.
fn pair(tw: f64, tc: f64, rest: Government) -> (Government, Government) {
    let mut a = rest;
    a.payroll = tw;
    a.consumption = tc;
    let mut b = rest;
    b.payroll = 1.0 - (1.0 - tw) / (1.0 + tc);
    b.consumption = 0.0;
    (a, b)
}

#[test]
fn the_wage_leg_is_a_wage_tax() {
    // GT against GT′ (the dependence form), KT's pair (the priced exit, plots rented) and ER
    // against ER′ (a walled type): the same x*, v, P, n_S,i, Y, T_m and T_p within 1e-12. The
    // revenues differ by the rent and interest legs and the tax on the transfers' spending:
    // G_c − G_w′ = t_c·(R + interest) + t_c·τ_w′·W.
    let cases: [(&str, oracle::ParcelParams, Government); 3] = [
        ("GT", g1_parcels(4.0, 0.05, 1.0), consumption(0.25)),
        ("KT", k3(), consumption(0.25)),
        ("ER", e2_parcels(), er_government()),
    ];
    for (what, p, g) in cases {
        let (a, b) = pair(g.payroll, g.consumption, g);
        let (_, ea) = checked_1f(household(p.clone(), a));
        let (_, eb) = checked_1f(household(p, b));
        same_allocation(what, &ea, &eb, 1e-12);
        if a.payroll == 0.0 {
            let t = a.consumption;
            let w = ea.base.base.wage_bill;
            let rent = ea.base.rent * ea.base.land.market;
            close(
                &format!("{what}: G_c − G_w′"),
                ea.government.revenue_consumption - eb.government.revenue_payroll,
                t * (rent + ea.base.base.interest) + t * b.payroll * w,
            );
        }
    }
    let (_, kt) = checked_1f(household(k3(), consumption(0.25)));
    let b = &kt.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, KT_X_STAR),
        ("v", b.v, KT_V),
        ("P", kt.basket.price, KT_P),
        ("N_a", b.n_a, KT_N_A),
        ("T_p", kt.base.land.rented_plots, KT_T_P),
    ] {
        close(&format!("KT {name}"), got, want);
    }
}

#[test]
fn support_at_consumer_prices() {
    // §2.1, §2.3: the provider buys the support at P^c, so a consumption tax moves the
    // equilibrium (GT: x* 0.89016, not G1's 0.86315) as its payroll equivalent does.
    let (e, gt) = checked_1f(g1_with(consumption(0.25)));
    // κ_w = 1/(1 + t_c) = 0.8: the net wage against producer-priced composites
    assert_eq!(e.net_factor(), 0.8);
    let b = &gt.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, GT_X_STAR),
        ("v", b.v, GT_V),
        ("P", gt.basket.price, GT_P),
        ("N_a", b.n_a, GT_N_A),
        ("ω_net", omega_net(&consumption(0.25), &gt), GT_OMEGA_NET),
    ] {
        close(name, got, want);
    }
    close("P^c", gt.basket.consumer_price, 1.25 * gt.basket.price);
    let g1 = solved_1f(HouseholdParams::from_parcels(g1_parcels(4.0, 0.05, 1.0)));
    assert!(b.x_star - g1.base.base.x_star > 0.02);
}

#[test]
fn home_output_is_untaxed() {
    // §2.1: home output is in kind and untaxed; it replaces purchases at consumer prices, ê_i =
    // (1 + t_c)·e_i, which is what makes KT's pair hold. At KT's equilibrium each type's supply
    // with the home output valued at producer prices (ê_i = e_i) would differ by more than 1e-6.
    let g = consumption(0.25);
    let (_, kt) = checked_1f(household(k3(), g));
    let b = &kt.base.base;
    let w = &kt.base.workers[0];
    assert!(w.exit_value > 0.0);
    let t = &k3().worker_types[0];
    let right = rule_supply(&g, t, b.workers[0].wage, b.p_s, w.exit_value, 0.0);
    close("the supply with ê = (1 + t_c)e", b.workers[0].supply, right);
    let wrong = rule_supply(&g, t, b.workers[0].wage, b.p_s, w.exit_value / 1.25, 0.0);
    assert!((wrong - right).abs() > 1e-6 * t.workers, "{wrong} {right}");
}
