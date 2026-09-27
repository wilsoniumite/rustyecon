//! e9: regimes at exact equality, validation, permutation and units (docs/unit-1e.md §3.2,
//! §4.3, §4.4, §5.3 and §8).

use oracle::{
    Access, Branch, EnclosureSide, ExitForm, ExitLand, LandMarket, Margin, ParamError,
    ParcelEconomy, ParcelParams, PricedExit, Regime, UniformWorkCost, BRACKET_HI,
};

use crate::g8_regimes::saturated;

use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;

#[test]
fn exact_zeros() {
    // G(0) = T_o exactly is the Commons side of the boundary; one double less is Crowded.
    let x = 0.5;
    let big = economy_1e(k1());
    let t = big.workers().technique_at(x);
    let g0 = big.at_with(x, t).commons_occupied;
    assert_eq!(big.at_with(x, t).exit_land, ExitLand::Commons);
    let with = |services: f64| {
        let mut p = k1();
        p.parcels[1] = open(services, 1.0);
        economy_1e(p).at_with(x, t)
    };
    let at = with(g0);
    assert_eq!((at.exit_land, at.plot_rent), (ExitLand::Commons, 0.0));
    let at = with(g0.next_down());
    assert_eq!(at.exit_land, ExitLand::Crowded);
    assert!(at.plot_rent > 0.0 && at.plot_rent < 1e-12);
    // G(r) = T_o exactly is the Enclosed side, with no plot rented; one double more is Crowded.
    let gr = economy_1e(k3()).at_with(x, t).rented_plots;
    let at = with(gr);
    assert_eq!(at.exit_land, ExitLand::Enclosed);
    assert_eq!((at.rented_plots, at.plot_rent), (0.0, 1.0));
    let at = with(gr.next_up());
    assert_eq!(at.exit_land, ExitLand::Crowded);
    assert!(at.plot_rent < 1.0 && at.plot_rent > 1.0 - 1e-12);
    // f_∞ = 0 exactly: supply saturates between x = 1 and the wall's end, and N = n_D there;
    // the equilibrium is the junction of the wall and the idle stretch, no land idle.
    let e = economy_1e(e0(goodspace(4.0, 0.05, 0.9)));
    let technique = e.workers().machines().envelope().last();
    let n_d = e.at_with(BRACKET_HI, technique).point.n_d;
    let mut w = goodspace(n_d, 0.05, 0.9);
    w.worker_types[0].workers = n_d;
    let end = economy_1d(w.clone()).wall_end();
    assert_eq!(end.excess, 0.0, "f_∞ = 0 exactly");
    assert!(economy_1d(w.clone()).at(1.0).excess_demand() > 0.0);
    let (_, eq) = checked_1e(e0(w));
    assert_eq!(eq.land_market, LandMarket::Idle);
    assert_eq!(eq.land.market, eq.land.enclosed);
    assert_eq!(eq.land.idle, 0.0);
    // A type exactly at its q_enc is on its floor: h = p_g·Δ exactly.
    let e = economy_1e(k3());
    let p_g = e.at_with(x, t).exit_good_price;
    let at_threshold = |plot: f64| {
        let mut p = k3();
        p.exits = vec![priced(0.5, 0.0, plot)];
        economy_1e(p).at_with(x, t)
    };
    let h = p_g * 0.5;
    let at = at_threshold(h);
    assert_eq!(at.branches[0], Branch::Floor);
    assert_eq!(at.exit_values[0], 0.0);
    let below = at_threshold(h.next_down());
    assert_eq!(below.branches[0], Branch::Plot);
    // on either branch the supply is the same to rounding
    close_to(
        "the same supply",
        below.point.supply[0],
        at.point.supply[0],
        1e-14,
    );
}

#[test]
fn saturated_knife_edges() {
    // 1d's saturated knife edge in parcel form (1d §12 items 4 and 17): f is 0 on the whole wall
    // and at its end. The wall's piece starts at an exact zero, which is the equilibrium, not the
    // junction with the idle stretch (§5.3 step 3).
    let n_1 = economy_1d(from_1a(saturated(1.0))).at_with(1.0, 0).n_d;
    let (e, eq) = checked_1e(from_1a_1e(saturated(n_1)));
    let line = e.at_with(1.0, 0);
    assert_eq!(line.excess_demand(), 0.0);
    assert_eq!(
        (eq.land_market, eq.base.margin),
        (LandMarket::Scarce, Margin::Wall)
    );
    assert_eq!(eq.base.v.to_bits(), line.point.v.to_bits());
    assert_eq!(eq.f_end, Some(0.0));
    // The same with a priced type saturated from x = 1 on: no one exits there, no plot is asked
    // for, f(1) = n_D(1) − N = 0 exactly, and f(1) = 0 is on the positive side, so the
    // equilibrium is the wall's start rather than x = 1 on the line.
    let mut p = k3();
    p.worker_types[0].work_cost = UniformWorkCost { chi_max: 0.1 };
    let n_d = economy_1e(p.clone()).at_with(1.0, 0).point.n_d;
    p.worker_types[0].workers = n_d;
    let e = economy_1e(p.clone());
    let one = e.at_with(1.0, 0);
    assert_eq!(
        (one.excess_demand(), one.exit_land),
        (0.0, ExitLand::Unused)
    );
    let (_, eq) = checked_1e(p);
    assert_eq!(eq.base.margin, Margin::Wall);
    assert_eq!(eq.base.v.to_bits(), one.point.v.to_bits());
    assert_eq!((eq.base.bisection_steps, eq.f_end), (0, Some(0.0)));
}

#[test]
fn lemma_b1_uses_the_market_land() {
    // §4.8: Lemma B.1's flag compares the market's land at x = 1, T_m(1) = T − T_p(1), with the
    // support bill. At K3 with a support ν between 1.5 and 2 the bill lies between T_m(1) and T
    // while f(1) < 0: the flag is false, where one on T would be true.
    let chosen = (0..=50)
        .map(|k| 1.5 + 0.01 * f64::from(k))
        .find(|&support| {
            let mut p = k3();
            p.worker_types[0].support = support;
            let e = economy_1e(p);
            let one = e.at_with(1.0, 0);
            let bill = e.workers().support() * one.point.p_s;
            one.market_land < bill && bill < e.enclosed_land() && one.excess_demand() < 0.0
        });
    let mut p = k3();
    p.worker_types[0].support = chosen.expect("a support between T_m(1) and T");
    let (_, eq) = checked_1e(p);
    assert!(!eq.base.lemma_b1);
    assert!(eq.base.f_line_1.unwrap() < 0.0);
}

#[test]
fn validation() {
    let base = k1();
    let check = |p: ParcelParams, name: &str| match ParcelEconomy::new(p) {
        Err(e) => assert_eq!(e.name(), name, "{e}"),
        Ok(_) => panic!("{name} accepted"),
    };
    // parcels
    check(
        ParcelParams {
            parcels: vec![],
            ..base.clone()
        },
        "parcels",
    );
    for (acreage, quality, name) in [
        (0.0, 1.0, "acreage"),
        (f64::NAN, 1.0, "acreage"),
        (1e31, 1.0, "acreage"),
        (1.0, -1.0, "quality"),
        (1.0, 1e-31, "quality"),
        (1.0, f64::INFINITY, "quality"),
    ] {
        let mut p = base.clone();
        p.parcels[1] = open(acreage, quality);
        match ParcelEconomy::new(p) {
            Err(ParamError::Item { kind, index, error }) => {
                assert_eq!((kind, index, error.name()), ("parcel", 1, name));
            }
            other => panic!("{name}: {other:?}"),
        }
    }
    // T in range: an enclosed parcel of positive quality; T and T_o at most SCALE_CEIL
    let mut p = base.clone();
    p.parcels[0].quality = 0.0;
    check(p, "parcels");
    let mut p = base.clone();
    p.parcels[0] = enclosed(1e20, 1e20);
    check(p, "parcels");
    let mut p = base.clone();
    p.parcels[1] = open(1e20, 1e20);
    check(p, "parcels");
    // one exit form per type
    check(
        ParcelParams {
            exits: vec![],
            ..base.clone()
        },
        "exits",
    );
    // a priced form's fields
    for (form, name) in [
        (priced(-1.0, 0.0, 0.1), "gross"),
        (priced(0.5, f64::NAN, 0.1), "floor"),
        (priced(0.5, 0.0, 1e31), "plot"),
    ] {
        let mut p = base.clone();
        p.exits = vec![form];
        match ParcelEconomy::new(p) {
            Err(ParamError::Item { kind, index, error }) => {
                assert_eq!((kind, index, error.name()), ("worker type", 0, name));
            }
            other => panic!("{name}: {other:?}"),
        }
    }
    // the exit good names a category
    check(
        ParcelParams {
            exit_good: 2,
            ..base.clone()
        },
        "exit_good",
    );
    // no reserved hours with a priced form (§2.9)
    let mut p = e0(entrant_trained(8.0, 3.0, 1.0, 1.0));
    p.exits[0] = priced(0.5, 0.0, 0.1);
    check(p.clone(), "reserved");
    p.exits[0] = ExitForm::Dependence;
    assert!(ParcelEconomy::new(p).is_ok());
    // land for every plot: T + T_o > Σ h_i·N_i
    let mut p = base.clone();
    p.exits = vec![priced(0.5, 0.0, 11.0 / 4.0)];
    check(p.clone(), "parcels");
    p.exits = vec![priced(0.5, 0.0, 11.0 / 4.0 * (1.0 - 1e-15))];
    assert!(ParcelEconomy::new(p).is_ok());
    // a plot that never pays, or takes no land, needs none
    let mut p = base.clone();
    p.exits = vec![priced(0.5, 0.5, 100.0)];
    assert!(ParcelEconomy::new(p).is_ok());
    // support stays positive (1d's rule)
    let mut p = base.clone();
    p.worker_types[0].support = 0.0;
    check(p, "support");
    // −0.0 is stored as +0.0
    let mut p = base.clone();
    p.parcels.push(enclosed(1.0, -0.0));
    p.exits = vec![priced(0.5, -0.0, 0.1)];
    let e = ParcelEconomy::new(p).unwrap();
    assert_eq!(e.params().parcels[2].quality.to_bits(), 0.0_f64.to_bits());
    let ExitForm::Priced(PricedExit { floor, .. }) = e.params().exits[0] else {
        panic!()
    };
    assert_eq!(floor.to_bits(), 0.0_f64.to_bits());
}

#[test]
fn permutation() {
    // Parcels reversed: the same equilibrium within 1e-12, the per-parcel outputs permuted.
    for p in [k1(), k2(), i4(0.5)] {
        let a = solved_1e(p.clone());
        let mut q = p.clone();
        q.parcels.reverse();
        let b = solved_1e(q);
        for (name, x, y) in [
            ("x*", a.base.x_star, b.base.x_star),
            ("v", a.base.v, b.base.v),
            ("Y", a.base.y, b.base.y),
            ("T_m", a.land.market, b.land.market),
        ] {
            close(name, y, x);
        }
        let n = a.parcels.len();
        for z in 0..n {
            let (u, w) = (&a.parcels[z], &b.parcels[n - 1 - z]);
            close_to("used", w.used, u.used, 1e-12);
            assert_eq!(w.rent_per_acre, u.rent_per_acre);
        }
    }
    // Worker types reversed (T): the same equilibrium, the per-type outputs permuted.
    let a = solved_1e(two_types());
    let mut p = two_types();
    p.worker_types.reverse();
    p.exits.reverse();
    let b = solved_1e(p);
    close("x*", b.base.x_star, a.base.x_star);
    close("r_o", b.plot_rent, a.plot_rent);
    for i in 0..2 {
        close_to(
            "hours",
            b.base.workers[1 - i].hours,
            a.base.workers[i].hours,
            1e-12,
        );
        close_to(
            "plots",
            b.workers[1 - i].plot_land,
            a.workers[i].plot_land,
            1e-12,
        );
    }
}

/// An economy with its land service measured in units of 1/c: every Q_z, every direct land
/// requirement, every machine recipe's land and every plot times c.
pub(crate) fn in_units(mut p: ParcelParams, c: f64) -> ParcelParams {
    for z in &mut p.parcels {
        z.quality *= c;
    }
    for cat in &mut p.categories {
        cat.direct_land *= c;
    }
    for t in &mut p.machine_types {
        t.operating.land *= c;
        t.build.land *= c;
    }
    for e in &mut p.exits {
        if let ExitForm::Priced(x) = e {
            x.plot *= c;
        }
    }
    p
}

#[test]
fn units() {
    // The same x*, Y, hours and participation; every price and v times c (the rent per unit
    // is 1/c of what it was); q over c; κ unchanged. Bit for bit at c = 4.
    for p in [k1(), k2(), k3(), race(60.0, 2.0), i4(0.5)] {
        for c in [4.0, 3.0] {
            let a = solved_1e(p.clone());
            let b = solved_1e(in_units(p.clone(), c));
            let tol = if c == 4.0 { 0.0 } else { 1e-12 };
            let same = |name: &str, x: f64, y: f64| {
                if tol == 0.0 {
                    assert_eq!(x.to_bits(), y.to_bits(), "{name} at c {c}");
                } else {
                    close_to(name, y, x, tol);
                }
            };
            same("x*", a.base.x_star, b.base.x_star);
            same("Y", a.base.y, b.base.y);
            same("N_a", a.base.n_a, b.base.n_a);
            same("participation", a.base.participation, b.base.participation);
            same("κ", a.coverage, b.coverage);
            same("r_o", a.plot_rent, b.plot_rent);
            let scale = if a.land_market == LandMarket::Idle {
                1.0
            } else {
                c
            };
            same("v", a.base.v * scale, b.base.v);
            same("P_s", a.base.p_s * scale, b.base.p_s);
            same("p_g", a.exit_good_price * scale, b.exit_good_price);
            if a.land_market == LandMarket::Scarce {
                same("q", a.q / c, b.q);
            }
            same("T_m", a.land.market * c, b.land.market);
        }
    }
}

#[test]
fn enclosure_by_law_moves_services() {
    // Switching WASTE from open to enclosed changes T and T_o by its A·Q exactly.
    let (a, b) = (economy_1e(k1()), economy_1e(k4()));
    assert_eq!(b.enclosed_land() - a.enclosed_land(), 1.0);
    assert_eq!(a.commons() - b.commons(), 1.0);
    assert_eq!(b.commons(), 0.0);
    assert_eq!(k4().parcels[1].access, Access::Enclosed);
    // and it moves the equilibrium: the enclosed regime with plots rented
    let eq = solved_1e(k4());
    assert_eq!(eq.exit_land, ExitLand::Enclosed);
    assert!(matches!(economy_1e(k1()).solve(), Ok(Regime::Interior(_))));
    let _ = EnclosureSide::Below;
}
