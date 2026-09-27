//! f10: validation, exact zeros, units and permutation (docs/unit-1f.md §3.2, §8).

use oracle::{
    Basket, Budget, Government, HouseholdEconomy, HouseholdParams, ParamError, ParcelParams,
    Program, Regime, Requirement, SolveError, TransferMode, SIGMA_CEIL,
};

use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;
use crate::support_1f::*;

/// The error a government field gives.
fn government_error(params: HouseholdParams) -> (&'static str, ParamError) {
    match HouseholdEconomy::new(params) {
        Err(ParamError::Item { kind, index, error }) => {
            assert_eq!((kind, index), ("government", 0));
            (error.name(), *error)
        }
        other => panic!("{other:?}"),
    }
}

fn invalid(params: HouseholdParams) -> &'static str {
    match HouseholdEconomy::new(params) {
        Err(ParamError::Invalid { name, .. }) => name,
        other => panic!("{other:?}"),
    }
}

#[test]
fn validation() {
    let g1 = g1_parcels(4.0, 0.05, 1.0);
    // σ in [SCALE_FLOOR, SIGMA_CEIL], finite.
    for sigma in [
        0.0,
        -1.0,
        1e-31,
        SIGMA_CEIL.next_up(),
        1e3,
        f64::NAN,
        f64::INFINITY,
    ] {
        let p = HouseholdParams {
            economy: g1.clone(),
            basket: Basket::Ces { sigma },
            government: Government::none(),
        };
        let err = HouseholdEconomy::new(p).unwrap_err();
        assert_eq!(err.name(), "sigma", "{sigma}");
        if sigma.is_finite() {
            assert!(matches!(
                err,
                ParamError::OutOfRange {
                    requirement: Requirement::Elasticity,
                    ..
                }
            ));
        }
    }
    for sigma in [1e-30, SIGMA_CEIL, 0.5, 1.0] {
        let p = HouseholdParams {
            economy: g1.clone(),
            basket: Basket::Ces { sigma },
            government: Government::none(),
        };
        assert!(HouseholdEconomy::new(p).is_ok(), "{sigma}");
    }
    // A CES basket only with every type without an exit value and no reserved hours.
    let with = |economy: ParcelParams| HouseholdParams {
        economy,
        basket: Basket::Ces { sigma: 0.5 },
        government: Government::none(),
    };
    assert_eq!(invalid(with(k1())), "basket");
    assert_eq!(invalid(with(e2_parcels())), "basket");
    let mut off = g1.clone();
    off.exits = vec![priced(0.0, 0.0, 1.0)];
    assert!(HouseholdEconomy::new(with(off)).is_ok());
    // The government's fields.
    for (g, name) in [
        (payroll(1.0), "payroll"),
        (payroll(-0.1), "payroll"),
        (payroll(f64::NAN), "payroll"),
        (consumption(-1.0), "consumption"),
        (consumption(1e31), "consumption"),
        (consumption(f64::INFINITY), "consumption"),
        (rent_rate(1e-31), "dividend"),
        (rent_rate(-1.0), "dividend"),
        (dividend(1.5), "rent_tax"),
        (dividend(-0.1), "rent_tax"),
        (program(1e31, 0.0), "work"),
        (program(0.0, -2.0), "exit"),
    ] {
        let (got, _) = government_error(g1_with(g));
        assert_eq!(got, name, "{g:?}");
    }
    for g in [
        payroll(0.999),
        consumption(1e30),
        rent_rate(1e-30),
        dividend(1.0),
        dividend(0.0),
    ] {
        assert!(HouseholdEconomy::new(g1_with(g)).is_ok(), "{g:?}");
    }
    // −0.0 is stored as +0.0.
    let e = economy_1f(g1_with(government(|g| {
        g.payroll = -0.0;
        g.consumption = -0.0;
        g.program = Program {
            work: -0.0,
            exit: -0.0,
        };
        g.budget = Budget::RentRate { dividend: -0.0 };
    })));
    let g = e.params().government;
    for v in [g.payroll, g.consumption, g.program.work, g.program.exit] {
        assert_eq!(v.to_bits(), 0.0_f64.to_bits());
    }
    assert_eq!(g.budget, Budget::RentRate { dividend: 0.0 });
    // The Dividend closure only without reserved hours, without plot takers and with a uniform
    // program (§2.6).
    assert_eq!(invalid(household(e2_parcels(), dividend(0.5))), "budget");
    assert_eq!(invalid(household(k1(), dividend(0.5))), "budget");
    assert_eq!(
        invalid(g1_with(government(|g| {
            g.budget = Budget::Dividend { rent_tax: 0.5 };
            g.program = Program {
                work: 0.1,
                exit: 0.0,
            };
        }))),
        "budget"
    );
    assert!(HouseholdEconomy::new(g1_with(government(|g| {
        g.budget = Budget::Dividend { rent_tax: 0.5 };
        g.program = Program {
            work: 0.1,
            exit: 0.1,
        };
    })))
    .is_ok());
    // a priced type that takes no plot admits the Dividend closure
    let mut floor = g1.clone();
    floor.exits = vec![priced(0.0, 0.5, 0.1)];
    assert!(HouseholdEconomy::new(household(floor, dividend(0.5))).is_ok());
    // With reserved hours μ_w ≤ μ_e (§2.8).
    assert_eq!(
        invalid(household(e2_parcels(), program(0.2, 0.1))),
        "program"
    );
    assert!(HouseholdEconomy::new(household(e2_parcels(), program(0.1, 0.1))).is_ok());
    // With the fixed basket and no government, 1e's rules: its rejected rows keep 1e's errors
    // (f1::refusals_nest).
    let mut p = g1;
    p.parcels.clear();
    assert_eq!(
        HouseholdEconomy::new(HouseholdParams::from_parcels(p.clone())).unwrap_err(),
        oracle::ParcelEconomy::new(p).unwrap_err()
    );
}

#[test]
fn exact_zeros() {
    // L_R = 0 exactly without a government: within the rent.
    let (_, eq) = checked_1f(g1_with(Government::none()));
    assert_eq!(eq.government.levy, Some(0.0));
    assert_eq!(eq.government.within_rent, Some(true));
    // d = ν·P^c exactly in Replace mode: the kink, where both branches give A = P^c.
    let (_, ga) = checked_1f(g1_with(replacing(rent_rate(1.0))));
    assert_eq!(
        ga.government.dividend.to_bits(),
        ga.basket.consumer_price.to_bits()
    );
    assert_eq!(ga.accounts.workers[0].support, 0.0);
    assert_eq!(
        ga.accounts.workers[0].unearned.to_bits(),
        ga.basket.consumer_price.to_bits()
    );
    // μ_w = μ_e with reserved hours is allowed, and a walled type stays walled.
    let (_, eq) = checked_1f(household(e2_parcels(), program(0.1, 0.1)));
    assert!(!eq.base.base.workers[1].pooled);
    // f_0 = 0 exactly is not positive: SurplusLabour. N 10 at χ_max 0.05 with μ_w 0.1 draws all
    // 10 into work at a zero wage, against n_D(0) = 10.
    let e = economy_1f(household(g1_parcels(10.0, 0.05, 0.05), program(0.1, 0.0)));
    assert_eq!(e.start().unwrap(), 0.0);
    assert_eq!(e.solve(), Err(SolveError::SurplusLabour { f_start: 0.0 }));
    // with a benefit too small to draw all 10 in at a zero wage the start is positive
    let e = economy_1f(household(g1_parcels(10.0, 0.05, 0.05), program(0.01, 0.0)));
    assert!(e.start().unwrap() > 0.0);
    assert!(matches!(e.solve(), Ok(Regime::Interior(_))));
}

#[test]
fn units() {
    // SSRN p.11: a proportional rescaling of every price leaves the choice unchanged. Land service
    // in other units (1e's e9, c = 4 and 3) with a government in composites: the same x*, Y,
    // hours, participation and κ; every price, v and d times c. Bit for bit at c = 4.
    let governments = [
        rent_rate(0.5),
        government(|g| {
            g.payroll = 0.1;
            g.consumption = 0.2;
            g.program = Program {
                work: 0.05,
                exit: 0.1,
            };
        }),
        replacing(rent_rate(1.2)),
    ];
    for p in [k1(), k3(), race(60.0, 2.0)] {
        for g in governments {
            for c in [4.0, 3.0] {
                let a = solved_1f(household(p.clone(), g));
                let b = solved_1f(household(
                    crate::e9_regimes_and_validation::in_units(p.clone(), c),
                    g,
                ));
                let tol = if c == 4.0 { 0.0 } else { 1e-12 };
                let same = |name: &str, x: f64, y: f64| {
                    if tol == 0.0 {
                        assert_eq!(x.to_bits(), y.to_bits(), "{name} at c {c}");
                    } else {
                        close_to(name, y, x, tol);
                    }
                };
                same("x*", a.base.base.x_star, b.base.base.x_star);
                same("Y", a.base.base.y, b.base.base.y);
                same("N_a", a.base.base.n_a, b.base.base.n_a);
                same("κ", a.base.coverage, b.base.coverage);
                same("v", a.base.base.v * c, b.base.base.v);
                same("P", a.basket.price * c, b.basket.price);
                same("d", a.government.dividend * c, b.government.dividend);
                same(
                    "the provider's composites",
                    a.accounts.provider.composites,
                    b.accounts.provider.composites,
                );
            }
        }
    }
}

#[test]
fn permutation() {
    // Two pooled types in either order: the same equilibrium, each type's own outputs swapped.
    let mut p = g1_parcels(4.0, 0.05, 1.0);
    p.worker_types = vec![worker(3.0, 1.0, 1.0, 1.0), worker(1.5, 0.7, 1.4, 1.3)];
    p.reserved = vec![vec![0.0, 0.0]; 2];
    p.exits = vec![oracle::ExitForm::Dependence; 2];
    let mut q = p.clone();
    q.worker_types.reverse();
    let g = government(|g| {
        g.payroll = 0.1;
        g.budget = Budget::RentRate { dividend: 0.3 };
        g.program = Program {
            work: 0.1,
            exit: 0.0,
        };
        g.mode = TransferMode::Replace;
    });
    let (_, a) = checked_1f(household(p, g));
    let (_, b) = checked_1f(household(q, g));
    close("x*", a.base.base.x_star, b.base.base.x_star);
    close("v", a.base.base.v, b.base.base.v);
    close("N_a", a.base.base.n_a, b.base.base.n_a);
    for i in 0..2 {
        let (x, y) = (&a.accounts.workers[i], &b.accounts.workers[1 - i]);
        close("A_i", x.unearned, y.unearned);
        close("spending", x.spending, y.spending);
    }
}

#[test]
fn certification_under_a_government() {
    // §5.4: 1e's Proposition 5 with κ_w·ε_i in place of ε_i, and only with μ_w = μ_e where a type
    // takes plots. K3 with the good given 0.3 of direct land: h·ℓ₀ = 0.1/1.3 = 1/13, so the
    // economy is certified while κ_w = 1/(1 + t_c) ≥ 1/13.
    let mut p = k3();
    p.categories[0].direct_land = 0.3;
    let certified = |g: Government| economy_1f(household(p.clone(), g)).parcels().certified();
    assert!(certified(Government::none()));
    assert!(certified(consumption(11.0)));
    assert!(!certified(consumption(13.0)));
    assert!(!certified(payroll(1.0 - 1.0 / 14.0)));
    assert!(!certified(program(0.1, 0.0)));
    assert!(certified(program(0.1, 0.1)));
}

#[test]
fn the_scan_where_monotonicity_is_not_proved() {
    // §5.3 step 4: at ρ > 0 a CES basket and a Dividend transfer that moves with the point scan
    // every piece of the path, since Proposition F reaches ρ = 0 only; the equilibrium does not
    // depend on the scan, bit for bit. At ρ = 0, or with every rate 0, or under RentRate, nothing
    // is scanned (the fixed basket's pieces have no plot taker here).
    let rho = |r: f64| {
        from_1a_1e(oracle::Params {
            rho: r,
            ..appendix_b()
        })
    };
    let scan = |p: HouseholdParams| {
        let e = economy_1f(p);
        let Ok(Regime::Interior(a)) = e.solve() else {
            panic!("an equilibrium")
        };
        let Ok(Regime::Interior(b)) = e.solve_scanned(0) else {
            panic!("an equilibrium")
        };
        let (mut x, mut y) = (bits_1f(&a), bits_1f(&b));
        x.remove("scan_points");
        y.remove("scan_points");
        assert_eq!(x, y);
        assert_eq!(b.base.scan_points, 0);
        a.base.scan_points
    };
    assert!(scan(ces(rho(0.05), 0.5)) > 0);
    assert!(scan(household(rho(0.05), dividend(0.5))) > 0);
    assert!(
        scan(household(
            rho(0.05),
            government(|g| {
                g.budget = Budget::Dividend { rent_tax: 0.0 };
                g.payroll = 0.1;
            })
        )) > 0
    );
    assert_eq!(scan(ces(rho(0.0), 0.5)), 0);
    assert_eq!(scan(household(rho(0.0), dividend(0.5))), 0);
    assert_eq!(scan(household(rho(0.05), dividend(0.0))), 0);
    assert_eq!(scan(household(rho(0.05), rent_rate(0.5))), 0);
}
