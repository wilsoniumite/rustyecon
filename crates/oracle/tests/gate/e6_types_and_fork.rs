//! e6: two priced types sharing a commons (T) and the fork economy with a priced exit in food
//! (F) (docs/unit-1e.md §3.3).

use oracle::{Branch, ExitLand, ParcelEconomy, BRACKET_HI, BRACKET_LO};

use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1d::{appendix_b_machine, baumol_one, worker};
use crate::support_1e::*;

#[test]
fn two_types_share_a_commons() {
    // §4.4: the commons is full and one shadow rent rations it for both types; their plots sum
    // to T_o = 0.34.
    let (_, eq) = checked_1e(two_types());
    let b = &eq.base;
    assert_eq!(eq.exit_land, ExitLand::Crowded);
    assert!(eq.workers.iter().all(|w| w.branch == Branch::Plot));
    close(
        "plots = T_o",
        eq.workers[0].plot_land + eq.workers[1].plot_land,
        0.34,
    );
    for (name, got, want) in [
        ("x*", b.x_star, T_X_STAR),
        ("v", b.v, T_V),
        ("P_s", b.p_s, T_P_S),
        ("Y", b.y, T_Y),
        ("n_pool", b.n_pool, T_N_POOL),
        ("entrant hours", b.workers[0].hours, T_ENTRANT_HOURS),
        ("trained hours", b.workers[1].hours, T_TRAINED_HOURS),
        ("entrant exiters", eq.workers[0].exiters, T_ENTRANT_EXITERS),
        ("trained exiters", eq.workers[1].exiters, T_TRAINED_EXITERS),
        ("r_o", eq.plot_rent, T_PLOT_RENT),
    ] {
        close_to(&format!("T {name}"), got, want, 1e-12);
    }
    // both at the same r_o: each type's exit value is p_g·s₀ − r_o·h with its own s₀ and h
    let p_g = eq.exit_good_price;
    close(
        "entrant e",
        eq.workers[0].exit_value,
        p_g * 0.5 - eq.plot_rent * 0.1,
    );
    close(
        "trained e",
        eq.workers[1].exit_value,
        p_g * 1.2 - eq.plot_rent * 0.05,
    );
}

#[test]
fn fork_goldens() {
    // F: 1c's M4 with exit (0.1, 0, 0.05) in food, ρ 0.04, under the engine. F1 has a commons
    // it does not fill; F2 rents plots, which leave production. Food's price comes through the
    // chain, and the cost system holds over every row on the market's land (check_identities).
    for (tag, with, regime, want) in [
        (
            "F1",
            true,
            ExitLand::Commons,
            [
                F1_X_STAR,
                F1_V,
                F1_P_S,
                F1_Y,
                F1_N_A,
                F1_PARTICIPATION,
                F1_INCOME,
                F1_P_FOOD,
                F1_T_OC,
                F1_T_P,
            ],
        ),
        (
            "F2",
            false,
            ExitLand::Enclosed,
            [
                F2_X_STAR,
                F2_V,
                F2_P_S,
                F2_Y,
                F2_N_A,
                F2_PARTICIPATION,
                F2_INCOME,
                F2_P_FOOD,
                F2_T_OC,
                F2_T_P,
            ],
        ),
    ] {
        let (_, eq) = checked_1e(fork_1e(with));
        let b = &eq.base;
        assert_eq!(eq.exit_land, regime, "{tag}");
        assert_eq!(b.technique, 1, "{tag}: the engine");
        let got = [
            b.x_star,
            b.v,
            b.p_s,
            b.y,
            b.n_a,
            b.participation,
            b.income,
            eq.exit_good_price,
            eq.land.commons_occupied,
            eq.land.rented_plots,
        ];
        for (g, w) in got.into_iter().zip(want) {
            if w == 0.0 {
                assert_eq!(g, 0.0, "{tag}");
            } else {
                close_to(tag, g, w, 1e-12);
            }
        }
        assert_eq!(eq.exit_good_price, b.categories[1].price);
        assert!(b.interest > 0.0);
    }
}

/// f along every region of the line and of the wall on a grid of `n` points.
fn grid(e: &ParcelEconomy, n: usize) -> Vec<Vec<f64>> {
    let w = e.workers();
    let env = w.machines().envelope();
    let points = w.switch_points().unwrap();
    let techniques: Vec<usize> = std::iter::once(env.first)
        .chain(env.switches.iter().map(|s| s.above))
        .collect();
    let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
        .chain(points.iter().copied())
        .chain(std::iter::once(BRACKET_HI))
        .collect();
    let mut out = Vec::new();
    for (r, &t) in techniques.iter().enumerate() {
        let (a, b) = (bounds[r], bounds[r + 1]);
        out.push(
            (0..=n)
                .map(|k| {
                    e.at_with(a + (b - a) * (k as f64 / n as f64), t)
                        .excess_demand()
                })
                .collect(),
        );
    }
    let one = e.at_with(BRACKET_HI, env.last());
    let v1 = one.point.v;
    let tech = w.wall_technique_at(v1);
    out.push(
        (0..=n)
            .map(|k| {
                let v = v1 * (1.0 + 20.0 * (k as f64 / n as f64));
                e.at_wage(BRACKET_HI, v, tech).excess_demand()
            })
            .collect(),
    );
    out
}

#[test]
fn certified_is_monotone() {
    // §5.4: F meets the land conditions of Proposition 5 (h ≤ s₀·b̄_food = 0.06), at a ρ where
    // the Proposition is not proved; its f is nonincreasing on a 256-point grid of every
    // stretch. K1 and K3 are certified at ρ = 0 by the same conditions but for Appendix B's
    // good, which carries no land, and are not.
    for with in [true, false] {
        let e = economy_1e(fork_1e(with));
        let (b_food, _) = e.certification_totals();
        assert!(0.05 <= 0.1 * b_food, "the land condition");
        assert!(!e.certified(), "ρ > 0");
        for stretch in grid(&e, 256) {
            for w in stretch.windows(2) {
                assert!(w[1] <= w[0] + 1e-12 * w[0].abs().max(1.0), "{w:?}");
            }
        }
    }
    for p in [k1(), k3()] {
        let e = economy_1e(p);
        assert_eq!(e.certification_totals().0, 0.0);
        assert!(!e.certified());
    }
    // a certified economy: F's conditions at ρ = 0
    let mut p = fork_1e(false);
    p.rho = 0.0;
    let e = economy_1e(p.clone());
    assert!(e.certified());
    // two plot-taking types are certified without a commons, not with one (§5.4: a crowded
    // commons can shift its occupants toward the type that supplies less)
    let mut two = p.clone();
    two.worker_types.push(worker(2.0, 1.0, 1.0, 1.0));
    two.exits.push(priced(0.1, 0.0, 0.05));
    two.reserved = vec![vec![0.0, 0.0]; 4];
    assert!(economy_1e(two.clone()).certified());
    two.parcels.push(open(1.0, 1.0));
    assert!(!economy_1e(two).certified());
    for stretch in grid(&e, 64) {
        for w in stretch.windows(2) {
            assert!(w[1] <= w[0] + 1e-12 * w[0].abs().max(1.0));
        }
    }
}

#[test]
fn certification_needs_both_conditions() {
    // §5.4's Proposition 5 needs h_i ≤ s₀,i·b̄_g and h_i·ℓ₀ ≤ ε_i for every plot-taking type.
    // F at ρ = 0 meets both; a type whose efficiency is below h·ℓ₀ fails the second alone.
    let mut p = fork_1e(false);
    p.rho = 0.0;
    let e = economy_1e(p.clone());
    let (b_food, ell0) = e.certification_totals();
    assert!(0.05 <= 0.1 * b_food && e.certified());
    for (efficiency, certified) in [(2.0 * 0.05 * ell0, true), (0.5 * 0.05 * ell0, false)] {
        let mut q = p.clone();
        q.worker_types[0].efficiency = efficiency;
        let e = economy_1e(q);
        assert_eq!(e.certification_totals(), (b_food, ell0));
        assert_eq!(e.certified(), certified, "ε {efficiency}");
    }
    // ℓ₀ = (L̄_ŷ + L^H_ŷ)/B_ŷ is n_D per unit of market land at x = 0, where every task is
    // human and the basket's land is direct: with the common hours of B's services (L^H 0.25)
    // in a priced economy
    let mut b = e0(baumol_one(1.0, 8.0, 0.25));
    b.exits = vec![priced(0.5, 0.0, 0.1)];
    let e = economy_1e(b.clone());
    let first = e.workers().machines().envelope().first;
    let zero = e.at_with(0.0, first);
    close(
        "ℓ₀",
        e.certification_totals().1,
        zero.point.n_d / zero.market_land,
    );
    b.human_required = vec![0.0; 3];
    let without = economy_1e(b).certification_totals().1;
    assert!(e.certification_totals().1 > without);
}

#[test]
fn the_all_human_corner_takes_the_cheapest_type() {
    // SSRN A.1 at the all-human corner: no task is done by machine, and the technique reported
    // is the task type cheapest at the corner's wage, not the envelope's first. G1 with a
    // second, labour-intensive machine type (build 0.3 of itself, labour 0.5, land 0.05),
    // η 10, N 20, χ_max 0.05 and exit (0.2, 0, 0.1): the corner's wage is so low that the
    // second type is cheaper there, while the first leads the envelope at x = 0.
    let mut p = goodspace_1e(20.0, 10.0, 10.0, 0.05, 0.05);
    let m = appendix_b_machine();
    let mut a = m.clone();
    a.build.machines = vec![m.build.machines[0], 0.0];
    a.operating.machines = vec![0.0, 0.0];
    let mut b = a.clone();
    b.build.machines = vec![0.0, m.build.machines[0]];
    b.build.labor = 0.5;
    b.build.land = 0.05;
    p.machine_types = vec![a, b];
    p.exits = vec![priced(0.2, 0.0, 0.1)];
    let (e, eq) = checked_1e(p);
    assert_eq!(eq.base.margin, oracle::Margin::AllHuman);
    assert_eq!(e.workers().machines().envelope().first, 0);
    assert_eq!(eq.base.technique, 1);
    let t = &eq.base.types;
    assert!(t[1].price < t[0].price, "{} {}", t[1].price, t[0].price);
    assert_eq!(eq.base.residuals.cheapest, 0.0);
}
