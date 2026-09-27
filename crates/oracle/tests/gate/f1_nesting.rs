//! f1: units 1a to 1e in household form, exactly (docs/unit-1f.md §2.14, §9; PLAN Phase 1's
//! gate, R1). With the fixed basket and `Government::none()` every new operation of §5.1 is an
//! exact no-op, so every equilibrium, error and count of 1e is 1f's bit for bit; the producer
//! side does not move with a government (§4.1), and a replacing transfer moves nothing (§4.10 (c)).

use oracle::{
    Basket, Eq1e, Eq1f, Government, HouseholdEconomy, HouseholdParams, MachineParams, ParamError,
    ParcelEconomy, ParcelParams, Regime, WorkerParams,
};

use crate::support::*;
use crate::support_1c::*;
use crate::support_1d::*;
use crate::support_1e::*;
use crate::support_1f::*;

/// Unit 1e's golden instances (docs/unit-1e.md §3.3), in 1e's parameters.
pub(crate) fn instances_1e() -> Vec<(&'static str, ParcelParams)> {
    let mut q6 = race(14.0, 1.5);
    q6.worker_types[0].work_cost.chi_max = 3.0;
    let mut l2 = goodspace_1e(4.0, 10.0, 1.0, 0.6, 3.0);
    l2.exits = vec![priced(0.5, 0.0, 1.0)];
    l2.exit_good = 1;
    vec![
        ("Q1", race(50.0, 2.5)),
        ("Q2", race(60.0, 2.0)),
        ("Q3", race(80.0, 2.0)),
        ("Q4", race(80.0, 1.0)),
        ("Q5", race(80.0, 0.7)),
        ("Q6", q6),
        ("K1", k1()),
        ("K2", k2()),
        ("K3", k3()),
        ("K4", k4()),
        ("K5", k5()),
        ("D", dead_exit()),
        ("I1", e0(goodspace(0.25, 0.05, 1.0))),
        ("I2", e0(goodspace(4.0, 0.6, 3.0))),
        ("I3", e0(entrant_trained(8.0, 0.5, 1.0, 1.0))),
        ("I4", i4(0.5)),
        ("I5", i4(2.0)),
        ("L1", land_good(5.0, 1.8)),
        ("L2", l2),
        ("T", two_types()),
        ("F1", fork_1e(true)),
        ("F2", fork_1e(false)),
        ("M", m_economy()),
    ]
}

/// Every golden instance of units 1a to 1e in parcel form.
fn every_instance() -> Vec<(String, ParcelParams)> {
    let mut out: Vec<(String, ParcelParams)> = crate::c1_nesting::golden_instances()
        .into_iter()
        .map(|(what, p)| (what, from_1a_1e(p)))
        .collect();
    for (what, p) in crate::e1_nesting::instances_1d() {
        out.push((what.to_string(), e0(p)));
    }
    for (what, p) in [
        ("M3", m3(0.05)),
        ("M4", m4(1.0)),
        ("M4 eta 2", m4(2.0)),
        ("M5 rho 0.1", m5(0.1, 4.0, 1.0)),
    ] {
        out.push((what.to_string(), e0(WorkerParams::from_machines(p))));
    }
    for (what, p) in instances_1e() {
        out.push((what.to_string(), p));
    }
    out
}

/// What a unit-1e economy gave and what its household form gave.
#[derive(Debug, PartialEq)]
enum Nesting {
    Same,
    NotViable,
    Error(String),
    Rejected,
}

/// Every output of 1e's equilibrium equals the base's of 1f's bit for bit, with the count's
/// scan points and steps; f_0 is n_D at x = 0 and positive; the new residuals are 0 or within
/// rounding (§5.1).
fn assert_same(what: &str, household: &HouseholdEconomy, e: &Eq1e, f: &Eq1f) {
    assert_eq!(bits_1e(e), bits_1e(&f.base), "{what}");
    assert_eq!(e.scan_points, f.base.scan_points, "{what}");
    assert_eq!(
        e.base.bisection_steps, f.base.base.bisection_steps,
        "{what}"
    );
    let parcels = household.parcels();
    let first = parcels.workers().machines().envelope().first;
    let start = household.start().unwrap();
    let n_d = if parcels.plot_takers().is_empty() && e.workers.iter().all(|w| w.exit_value == 0.0) {
        parcels.at_with(0.0, first).point.n_d
    } else {
        parcels.at_wage(0.0, 0.0, first).point.n_d
    };
    if start.is_finite() {
        assert_eq!(start.to_bits(), n_d.to_bits(), "{what}: f_0 = n_D(0)");
    }
    assert!(start > 0.0, "{what}");
    assert_eq!(f.residuals.budget, 0.0, "{what}");
    assert_eq!(f.government.dividend, 0.0, "{what}");
    assert!(f.residuals.spending <= FULL && f.residuals.composites <= FULL);
    assert!(f.residuals.rent_share <= FULL && f.residuals.euler <= FULL);
    // the households' accounts are 1e's without a government (decision 116)
    close(
        &format!("{what}: provider composites"),
        f.accounts.provider.composites,
        e.base.provider_baskets,
    );
}

/// Solves a unit-1e economy in 1e and in household form and compares them.
fn nest(what: &str, params: ParcelParams) -> Nesting {
    let Ok(parcels) = ParcelEconomy::new(params.clone()) else {
        let f = HouseholdEconomy::new(HouseholdParams::from_parcels(params.clone()));
        assert_eq!(
            format!("{:?}", f.unwrap_err()),
            format!("{:?}", ParcelEconomy::new(params).unwrap_err()),
            "{what}: 1e's error"
        );
        return Nesting::Rejected;
    };
    let household = economy_1f(HouseholdParams::from_parcels(params));
    match (parcels.solve(), household.solve()) {
        (Ok(Regime::Interior(e)), Ok(Regime::Interior(f))) => {
            assert_same(what, &household, &e, &f);
            Nesting::Same
        }
        (Ok(Regime::NotViable { d_at_1: a }), Ok(Regime::NotViable { d_at_1: b })) => {
            assert_eq!(a.to_bits(), b.to_bits(), "{what}");
            Nesting::NotViable
        }
        (Err(a), Err(b)) if a == b => Nesting::Error(format!("{a:?}")),
        (a, b) => panic!("{what}: 1e gave {a:?} and 1f {b:?}"),
    }
}

#[test]
fn every_1e_golden_instance_is_bit_identical() {
    let mut same = 0;
    for (what, params) in every_instance() {
        match nest(&what, params) {
            Nesting::Same => same += 1,
            Nesting::Error(e) => assert!(matches!(what.as_str(), "M" | "I5"), "{what}: {e}"),
            other => panic!("{what}: {other:?}"),
        }
    }
    assert_eq!(same, 27 + 24 + 4 + 21);
    // the checked identities, on the household form of a few
    for params in [
        race(60.0, 2.0),
        k2(),
        fork_1e(true),
        e0(goodspace(0.25, 0.05, 1.0)),
    ] {
        checked_1f(HouseholdParams::from_parcels(params));
    }
}

#[test]
fn random_economies_are_bit_identical() {
    // 1d's d7 and 1e's e8 draws, every one: equilibrium, error and count.
    let mut tally = std::collections::BTreeMap::<String, usize>::new();
    for (samples, _) in crate::d7_random_workers::all_sets() {
        for (i, s) in samples.iter().enumerate() {
            let n = nest(
                &format!("d7 {:?} {i}", s.set),
                e0(s.economy.params().clone()),
            );
            *tally
                .entry(format!("{n:?}").split('(').next().unwrap().to_string())
                .or_default() += 1;
        }
    }
    for (samples, _) in crate::e8_random_parcels::all_sets() {
        for (i, s) in samples.iter().enumerate() {
            let household = economy_1f(HouseholdParams::from_parcels(s.economy.params().clone()));
            let what = format!("e8 {:?} {i}", s.set);
            match (&s.result, household.solve()) {
                (Ok(Regime::Interior(e)), Ok(Regime::Interior(f))) => {
                    assert_same(&what, &household, e, &f);
                    *tally.entry("Same".to_string()).or_default() += 1;
                }
                (a, b) => {
                    assert_eq!(format!("{a:?}"), format!("{b:?}"), "{what}");
                    *tally.entry("Error".to_string()).or_default() += 1;
                }
            }
        }
    }
    println!("f1 random tallies: {tally:?}");
    let want = [("Error", 17), ("NotViable", 9), ("Same", 871)];
    assert_eq!(
        tally,
        want.into_iter().map(|(k, n)| (k.to_string(), n)).collect(),
        "docs/unit-1f.md §14's tallies"
    );
}

#[test]
fn refusals_nest() {
    // NotViable with d_at_1 bit for bit; MultipleEquilibria (M) and NoMarket (I5) as 1e's.
    assert_eq!(
        nest("M4 eta 50", e0(WorkerParams::from_machines(m4(50.0)))),
        Nesting::NotViable
    );
    assert!(matches!(nest("M", m_economy()), Nesting::Error(e) if e.starts_with("Multiple")));
    assert!(matches!(nest("I5", i4(2.0)), Nesting::Error(e) if e.starts_with("NoMarket")));
    // Rejected rows keep 1e's errors.
    let base = e0(goodspace(4.0, 0.05, 1.0));
    for acreage in [0.0, f64::NAN, 1e31, -1.0] {
        let mut p = base.clone();
        p.parcels[0].acreage = acreage;
        assert_eq!(nest("acreage", p), Nesting::Rejected);
    }
    let mut p = base.clone();
    p.worker_types[0].support = 0.0;
    assert_eq!(nest("support", p), Nesting::Rejected);
    let mut p = base.clone();
    p.exits = vec![];
    assert_eq!(nest("exits", p), Nesting::Rejected);
    let mut p = k3();
    p.exit_good = 5;
    assert_eq!(nest("exit good", p), Nesting::Rejected);
    let mut p = e0(entrant_trained(8.0, 3.0, 1.0, 1.0));
    p.exits[0] = priced(0.5, 0.0, 0.1);
    let err = HouseholdEconomy::new(HouseholdParams::from_parcels(p.clone())).unwrap_err();
    assert_eq!(err, ParcelEconomy::new(p).unwrap_err());
    assert!(matches!(
        err,
        ParamError::Invalid {
            name: "reserved",
            ..
        }
    ));
}

#[test]
fn points_nest_on_a_grid() {
    // at(x) is 1e's at(x) bit for bit on a grid, on G1, M4, E1 and K1.
    for (what, params) in [
        ("G1", e0(goodspace(4.0, 0.05, 1.0))),
        ("M4", e0(WorkerParams::from_machines(m4(1.0)))),
        ("E1", e0(entrant_trained(8.0, 3.0, 1.0, 1.0))),
        ("K1", k1()),
    ] {
        let e = economy_1e(params.clone());
        let f = economy_1f(HouseholdParams::from_parcels(params));
        for i in 0..=100 {
            let x = f64::from(i) / 100.0;
            let (a, b) = (e.at(x), f.at(x));
            assert_eq!(a, b.point, "{what} at {x}");
            assert_eq!(b.price.to_bits(), a.point.p_s.to_bits());
            assert_eq!(b.consumer_price.to_bits(), a.point.p_s.to_bits());
            assert_eq!(b.transfer, 0.0);
            let z: Vec<f64> = e.params().categories.iter().map(|c| c.weight).collect();
            assert_eq!(b.content, z);
        }
    }
}

/// 1e's golden instances without plot-taking or walled types, which admit the Dividend closure.
fn dividend_instances() -> Vec<(String, ParcelParams)> {
    every_instance()
        .into_iter()
        .filter(|(_, p)| {
            let e = economy_1e(p.clone());
            e.plot_takers().is_empty() && p.reserved.iter().flatten().all(|&r| r == 0.0)
        })
        .collect()
}

#[test]
fn zero_government_in_either_closure() {
    // Dividend { rent_tax: 0 } in both modes, and RentRate with d̂ 0 in Replace mode, equal the
    // zero government bit for bit (§2.14, §4.3's δ(ω)).
    let (mut n, instances) = (0, dividend_instances());
    assert!(instances.len() >= 40, "{}", instances.len());
    let count = instances.len();
    for (what, params) in instances {
        let none = economy_1f(HouseholdParams::from_parcels(params.clone())).solve();
        for g in [
            dividend(0.0),
            replacing(dividend(0.0)),
            replacing(rent_rate(0.0)),
        ] {
            let other = economy_1f(household(params.clone(), g)).solve();
            match (&none, &other) {
                (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) => {
                    assert_eq!(bits_1e(&a.base), bits_1e(&b.base), "{what}: {g:?}");
                    assert_eq!(
                        a.accounts.provider.composites.to_bits(),
                        b.accounts.provider.composites.to_bits()
                    );
                    n += 1;
                }
                (a, b) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{what}"),
            }
        }
    }
    assert_eq!(n, 3 * count);
}

#[test]
fn the_producer_side_is_tax_free() {
    // §4.1 and §4.10 (a); Prop 6 (i): at every point of G1, B1 and M4 the prices, the totals
    // and the quantities are the same under GP's, GT's, GC's and GW's governments bit for bit.
    let governments = [
        payroll(0.1),
        consumption(0.25),
        dividend(0.5),
        program(0.2, 0.0),
    ];
    for (what, params) in [
        ("G1", e0(goodspace(4.0, 0.05, 1.0))),
        ("B1", e0(baumol_one(1.0, 8.0, 0.25))),
        ("M4", e0(WorkerParams::from_machines(m4(1.0)))),
    ] {
        let none = economy_1f(HouseholdParams::from_parcels(params.clone()));
        let t = none.parcels().workers().machines().envelope().first;
        for g in governments {
            let other = economy_1f(household(params.clone(), g));
            for i in 0..=20 {
                let x = f64::from(i) / 20.0;
                for (a, b) in [
                    (none.at(x), other.at(x)),
                    (
                        none.at_wage(1.0, 3.0 + x, t),
                        other.at_wage(1.0, 3.0 + x, t),
                    ),
                    (
                        none.at_wage(0.0, 0.1 * x, t),
                        other.at_wage(0.0, 0.1 * x, t),
                    ),
                ] {
                    let (a, b) = (&a.point.point, &b.point.point);
                    assert_eq!(a.v.to_bits(), b.v.to_bits(), "{what}");
                    assert_eq!(a.type_prices, b.type_prices, "{what}");
                    assert_eq!(a.prices, b.prices, "{what}");
                    assert_eq!(a.p_s.to_bits(), b.p_s.to_bits(), "{what}");
                    assert_eq!((a.h_s, a.m_s, a.b_d), (b.h_s, b.m_s, b.b_d), "{what}");
                    assert_eq!((a.y, a.n_d), (b.y, b.n_d), "{what}");
                    assert_eq!(a.services, b.services, "{what}");
                }
            }
        }
    }
}

#[test]
fn replacing_support_moves_nothing() {
    // SSRN p.16; §4.10 (c): GA (RentRate d̂ 1, Replace) and GR (Dividend τ_R 0.1, 0.25, 0.375,
    // Replace) are G1 bit for bit; the owners' after-tax rent and the provider's support fall by
    // the transfer, and the provider's composites do not move.
    let g1 = solved_1f(g1_with(Government::none()));
    for g in [
        replacing(rent_rate(1.0)),
        replacing(dividend(0.1)),
        replacing(dividend(0.25)),
        replacing(dividend(0.375)),
    ] {
        let (_, eq) = checked_1f(g1_with(g));
        assert_eq!(bits_1e(&g1.base), bits_1e(&eq.base), "{g:?}");
        let d = eq.government.dividend;
        assert!(d > 0.0 && d <= eq.basket.price);
        let rent = eq.base.rent * eq.base.land.market;
        close(
            "the owners' after-tax rent",
            eq.accounts.provider.receipts,
            rent - 4.0 * d,
        );
        close(
            "the provider's support",
            eq.accounts.provider.support,
            4.0 * (eq.basket.price - d),
        );
        close_to(
            "the provider's composites",
            eq.accounts.provider.composites,
            g1.accounts.provider.composites,
            1e-15,
        );
    }
}

#[test]
fn ces_approaches_the_fixed_basket() {
    // §2.10: σ → 0 is the fixed basket. G1 and W1 under Ces { sigma: 2^-40 } with G1's z.
    for (what, params) in [
        ("G1", g1_parcels(4.0, 0.05, 1.0)),
        ("W1", g1_parcels(4.0, 0.6, 1.0)),
    ] {
        let fixed = solved_1f(HouseholdParams::from_parcels(params.clone()));
        let ces = checked_1f(HouseholdParams {
            economy: params,
            basket: Basket::Ces {
                sigma: 1.0 / 1_099_511_627_776.0,
            },
            government: Government::none(),
        })
        .1;
        same_allocation(what, &fixed, &ces, 1e-8);
    }
}

#[test]
fn the_household_form_of_a_machine_economy() {
    // from_parcels and the machine forms compose: 1c's M4 through every unit is the same
    // equilibrium as in 1e.
    let p = e0(WorkerParams::from_machines(MachineParams::from_categories(
        crate::support_1b::fork_economy(),
    )));
    assert_eq!(nest("C3 through 1f", p), Nesting::Same);
}
