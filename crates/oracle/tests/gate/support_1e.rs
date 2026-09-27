//! Fixtures and checks shared by unit 1e's tests (docs/unit-1e.md §8).

// The identities are sums over categories, machine types, worker types and parcels, and the
// checks index several vectors at once, as docs/unit-1e.md §4.8 writes them.
#![allow(clippy::needless_range_loop)]

use std::collections::BTreeMap;

use oracle::{
    Access, Branch, CategoryParams, EnclosurePoint, EnclosureSide, Eq1e, ExitForm, ExitLand,
    LandMarket, MachineParams, Margin, Output1b, Params, Parcel, ParcelEconomy, ParcelParams,
    ParcelPoint, PowerSchedule, PricedExit, Regime, UniformWorkCost, WorkerParams,
};
use rustyecon_core::num;

use crate::support::*;
use crate::support_1b::at_most;
use crate::support_1c::*;
use crate::support_1d::*;

/// A parcel (A, Q, access).
pub fn parcel(acreage: f64, quality: f64, access: Access) -> Parcel {
    Parcel {
        acreage,
        quality,
        access,
    }
}

/// An enclosed parcel.
pub fn enclosed(acreage: f64, quality: f64) -> Parcel {
    parcel(acreage, quality, Access::Enclosed)
}

/// An open parcel, a commons.
pub fn open(acreage: f64, quality: f64) -> Parcel {
    parcel(acreage, quality, Access::Open)
}

/// The priced exit form (s₀, s̲, h).
pub fn priced(gross: f64, floor: f64, plot: f64) -> ExitForm {
    ExitForm::Priced(PricedExit { gross, floor, plot })
}

/// E0: a unit-1d economy in parcel form.
pub fn e0(params: WorkerParams) -> ParcelParams {
    ParcelParams::from_workers(params)
}

/// A unit-1a economy in parcel form, through 1b's, 1c's and 1d's forms.
pub fn from_1a_1e(params: Params) -> ParcelParams {
    e0(WorkerParams::from_machines(MachineParams::from_categories(
        CategoryParams::from_one_category(params),
    )))
}

/// G1 in parcel form with N, T, η, λ and χ_max changed (docs/unit-1e.md §3.3).
pub fn goodspace_1e(workers: f64, land: f64, eta: f64, lam: f64, chi_max: f64) -> ParcelParams {
    from_1a_1e(Params {
        workers,
        land,
        lam,
        schedule: appendix_b_schedule(eta),
        work_cost: UniformWorkCost { chi_max },
        ..appendix_b()
    })
}

/// Q (docs/unit-1e.md §3.3): G1 with (LAND, 100, 1, enclosed), one type (N, 1, 1, 1) and exit
/// (1.5, 0, 1), at η.
pub fn race(workers: f64, eta: f64) -> ParcelParams {
    ParcelParams {
        exits: vec![priced(1.5, 0.0, 1.0)],
        ..goodspace_1e(workers, 100.0, eta, 0.05, 1.0)
    }
}

/// K (docs/unit-1e.md §3.3): G1's economy on the given parcels, exit (0.5, 0, 0.1).
pub fn commons(parcels: Vec<Parcel>) -> ParcelParams {
    ParcelParams {
        parcels,
        exits: vec![priced(0.5, 0.0, 0.1)],
        ..goodspace_1e(4.0, 10.0, 1.0, 0.05, 1.0)
    }
}

/// K1's parcels: (FIELDS, 10, 1, enclosed) and (WASTE, 1, 1, open).
pub fn k1() -> ParcelParams {
    commons(vec![enclosed(10.0, 1.0), open(1.0, 1.0)])
}

/// K2's: WASTE (0.3, 1, open).
pub fn k2() -> ParcelParams {
    commons(vec![enclosed(10.0, 1.0), open(0.3, 1.0)])
}

/// K3's: no commons.
pub fn k3() -> ParcelParams {
    commons(vec![enclosed(10.0, 1.0)])
}

/// K4's: K1 with WASTE enclosed by law.
pub fn k4() -> ParcelParams {
    commons(vec![enclosed(10.0, 1.0), enclosed(1.0, 1.0)])
}

/// K5's: K1 with WASTE recut as (5, 0.2, open).
pub fn k5() -> ParcelParams {
    commons(vec![enclosed(10.0, 1.0), open(5.0, 0.2)])
}

/// D: G1 with exit (1, 0, 1).
pub fn dead_exit() -> ParcelParams {
    ParcelParams {
        exits: vec![priced(1.0, 0.0, 1.0)],
        ..goodspace_1e(4.0, 10.0, 1.0, 0.05, 1.0)
    }
}

/// I4 (docs/unit-1e.md §3.3): W3 (λ 0.6, χ_max 3) with exit (s₀, 0, 0.5) and parcels
/// (FIELDS, 5, 1.5) and (HEATH, 5, 0.5), both enclosed.
pub fn i4(gross: f64) -> ParcelParams {
    ParcelParams {
        parcels: vec![enclosed(5.0, 1.5), enclosed(5.0, 0.5)],
        exits: vec![priced(gross, 0.0, 0.5)],
        ..goodspace_1e(4.0, 10.0, 1.0, 0.6, 3.0)
    }
}

/// T (docs/unit-1e.md §3.3): G1's economy with (WASTE, 0.34, 1, open), the entrant (4, 1, 1, 1)
/// with exit (0.5, 0, 0.1) and the trained (1, 0.8, 1.5, 1.2) with exit (1.2, 0, 0.05).
pub fn two_types() -> ParcelParams {
    let base = goodspace_1e(4.0, 10.0, 1.0, 0.05, 1.0);
    ParcelParams {
        parcels: vec![enclosed(10.0, 1.0), open(0.34, 1.0)],
        worker_types: vec![worker(4.0, 1.0, 1.0, 1.0), worker(1.0, 0.8, 1.5, 1.2)],
        reserved: vec![vec![0.0, 0.0]; 2],
        exits: vec![priced(0.5, 0.0, 0.1), priced(1.2, 0.0, 0.05)],
        ..base
    }
}

/// F (docs/unit-1e.md §3.3): 1c's M4 with one type (4, 1, 1, 1) and exit (0.1, 0, 0.05) in food,
/// with a commons (WASTE, 1, 1) or without.
pub fn fork_1e(with_commons: bool) -> ParcelParams {
    let mut parcels = vec![enclosed(10.0, 1.0)];
    if with_commons {
        parcels.push(open(1.0, 1.0));
    }
    ParcelParams {
        parcels,
        exits: vec![priced(0.1, 0.0, 0.05)],
        exit_good: 1,
        ..e0(WorkerParams::from_machines(m4(1.0)))
    }
}

/// M (docs/unit-1e.md §3.3): (LAND, 8.6, 1), h 1.24, a 0.075, λ 0.063, b 0.88,
/// γ = 1.16(0.41 + 1.88x), one type (7, 0.67, 1, 0.1) with exit (1.1, 0.17, 0.36).
pub fn m_economy() -> ParcelParams {
    let mut p = from_1a_1e(Params {
        workers: 7.0,
        land: 8.6,
        space: 1.24,
        a: 0.075,
        lam: 0.063,
        b: 0.88,
        schedule: PowerSchedule {
            eta: 1.16,
            g0: 0.41,
            g1: 1.88,
            k: 1.0,
        },
        work_cost: UniformWorkCost { chi_max: 0.67 },
        ..appendix_b()
    });
    p.worker_types[0].support = 0.1;
    p.exits = vec![priced(1.1, 0.17, 0.36)];
    p
}

/// Validates unit-1e parameters the test knows to be valid.
pub fn economy_1e(params: ParcelParams) -> ParcelEconomy {
    ParcelEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{params:?}: {e}"))
}

/// Solves a unit-1e economy the test knows to have one equilibrium.
pub fn solved_1e(params: ParcelParams) -> Box<Eq1e> {
    match economy_1e(params.clone()).solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{params:?} has no equilibrium: {other:?}"),
    }
}

/// Solves, checks every identity of §4.8, and returns the equilibrium and the economy.
pub fn checked_1e(params: ParcelParams) -> (ParcelEconomy, Box<Eq1e>) {
    let e = economy_1e(params.clone());
    let eq = solved_1e(params);
    check_identities_1e(&e, &eq);
    (e, eq)
}

/// Every output of a unit-1e equilibrium, by printed key, as bits.
pub fn bits_1e(eq: &Eq1e) -> BTreeMap<String, Option<u64>> {
    eq.outputs()
        .into_iter()
        .map(|(key, output)| {
            let bits = match output {
                Output1b::Float(v) | Output1b::Optional(Some(v)) => Some(v.to_bits()),
                Output1b::Optional(None) => None,
                Output1b::Flag(b) => Some(u64::from(b)),
                Output1b::Count(n) => Some(u64::from(n)),
            };
            (key.to_string(), bits)
        })
        .collect()
}

/// The enclosure point an enclosure tie sits at.
pub fn enclosure_point(eq: &Eq1e) -> Option<EnclosurePoint> {
    eq.enclosure.map(|t| EnclosurePoint {
        worker: t.worker,
        margin: eq.base.margin,
        x: eq.base.x_star,
        v: eq.base.v,
        technique: eq.base.technique,
    })
}

/// The evaluation that priced an equilibrium: on the idle stretch, at an enclosure tie, on the
/// line or at a corner (docs/unit-1e.md §5.1).
pub fn point_of_1e(economy: &ParcelEconomy, eq: &Eq1e) -> ParcelPoint {
    let b = &eq.base;
    if eq.land_market == LandMarket::Idle {
        return match edge_of(b) {
            Some(edge) => economy.at_idle_edge(eq.land.market, b.technique, edge),
            None => economy.at_idle(eq.land.market, b.technique),
        };
    }
    if let Some(point) = enclosure_point(eq) {
        let share = eq.enclosure.map_or(0.0, |t| t.share);
        return economy.at_enclosure(&point, EnclosureSide::Share(share));
    }
    match b.margin {
        Margin::Contestable => economy.at_with(b.x_star, b.technique),
        _ => economy.at_wage(b.x_star, b.v, b.technique),
    }
}

/// Every identity and bound of docs/unit-1e.md §4.8 at an equilibrium, recomputed from the
/// reported fields and the parameters, with the prices, P_s, v, each exit value and each supply
/// pinned to the evaluation that set them bit for bit (§8). An equilibrium without exit values
/// on scarce land is also checked as unit 1d's.
pub fn check_identities_1e(economy: &ParcelEconomy, eq: &Eq1e) {
    check_identities_1e_under(economy, eq, None);
}

/// A type's supply at its wage and exit value under unit 1f's participation rule
/// (docs/unit-1f.md §4.4): (worker type, wage, exit value) to n_S,i.
pub type SupplyRule<'a> = &'a dyn Fn(usize, f64, f64) -> f64;

/// [`check_identities_1e`] with each pooled type's supply from `rule` in place of 1e's formula
/// (unit 1f's economies, docs/unit-1f.md §8); 1d's own check, whose supply is 1d's, runs only
/// without one.
pub fn check_identities_1e_under(economy: &ParcelEconomy, eq: &Eq1e, rule: Option<SupplyRule>) {
    let p = economy.params();
    let b = &eq.base;
    let at = |what: &str| format!("{what} at {p:?}");
    let exit_free = eq.workers.iter().all(|w| w.exit_value == 0.0) && eq.home.output == 0.0;
    if eq.land_market == LandMarket::Scarce
        && exit_free
        && eq.land.rented_plots == 0.0
        && rule.is_none()
    {
        check_identities_1d(economy.workers(), b);
    }
    let q = point_of_1e(economy, eq);
    let (t, t_o) = (economy.enclosed_land(), economy.commons());
    let kinds = p.worker_types.len();
    // The prices, bit for bit.
    assert_eq!(b.v.to_bits(), q.point.v.to_bits(), "{}", at("v"));
    for (k, ty) in b.types.iter().enumerate() {
        assert_eq!(
            ty.price.to_bits(),
            q.point.type_prices[k].to_bits(),
            "{}",
            at("p_k")
        );
    }
    for (j, c) in b.categories.iter().enumerate() {
        assert_eq!(
            c.price.to_bits(),
            (q.point.base_prices[j] + c.reserved_cost).to_bits(),
            "{}",
            at("p_j")
        );
    }
    // With a type at the edge of its reserved shortage on scarce land, or reserved hours at a
    // tie (whose walk moves with σ), the point's own P_s is not the equilibrium's; unit 1d's
    // check covers those.
    let edge_on_scarce_land = eq.land_market == LandMarket::Scarce && edge_of(b).is_some();
    let no_reserved = p.reserved.iter().flatten().all(|&r| r == 0.0);
    let pinned = !edge_on_scarce_land && (b.tie.is_none() || no_reserved);
    if pinned {
        assert_eq!(
            b.p_s.to_bits(),
            q.point.p_s.to_bits(),
            "{} ({:?} {:?} {:?} {:?})",
            at("P_s"),
            b.margin,
            eq.land_market,
            b.tie,
            b.workers
                .iter()
                .map(|w| (w.pooled, w.edge))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(
        eq.exit_good_price.to_bits(),
        b.categories[p.exit_good].price.to_bits(),
        "{}",
        at("p_g")
    );
    for i in 0..kinds {
        assert_eq!(
            eq.workers[i].exit_value.to_bits(),
            q.exit_values[i].to_bits(),
            "{}",
            at("e_i")
        );
        if pinned && b.workers[i].pooled && b.tie.is_none() {
            assert_eq!(
                b.workers[i].supply.to_bits(),
                q.point.supply[i].to_bits(),
                "{}",
                at("n_S,i")
            );
        }
    }
    // The land (§4.1, §4.8): the numeraire, the partition, T_m = Y·B^q.
    let rent = eq.rent;
    // At r = 0 an exit good that embodies no labour is free, and q = r/p_g takes its limit at
    // the wall's end, 1/p_wall with p_wall = b̃_g, its price there per unit of rent (§12 item
    // 16): the price at x = 1 with v = 0 and r = 1.
    let wall_price = (rent == 0.0 && eq.exit_good_price == 0.0)
        .then(|| economy.at_wage(1.0, 0.0, b.technique).exit_good_price);
    match eq.land_market {
        LandMarket::Scarce => {
            assert_eq!(rent, 1.0);
            assert_eq!(eq.land.idle, 0.0);
            close("T = T_m + T_p", eq.land.market + eq.land.rented_plots, t);
        }
        LandMarket::Idle => {
            assert_eq!((rent, b.v, b.x_star), (0.0, 1.0, 1.0), "{}", at("idle"));
            assert!(eq.land.idle >= -1e-12 * t, "{}", at("T_idle"));
            assert_eq!(b.margin, Margin::Wall);
            assert_eq!(eq.coverage, 0.0);
            let q_end = wall_price.map_or(0.0, |p_wall| 1.0 / p_wall);
            assert_eq!(eq.q.to_bits(), q_end.to_bits(), "{}", at("q at r = 0"));
            // At r = 0 every delivered machine cost is below labour's at every task (§2.8).
            assert!(b.replacement_top < b.v, "{}", at("the wall at r = 0"));
        }
    }
    assert_eq!((eq.land.enclosed, eq.land.commons), (t, t_o));
    let residual = (((t - eq.land.market) - eq.land.rented_plots) - eq.land.idle).abs() / t;
    assert_eq!(eq.residuals.partition.to_bits(), residual.to_bits());
    assert!(eq.residuals.partition <= 1e-12, "{}", at("partition"));
    close("Y B^q = T_m", b.y * b.b_s_q, eq.land.market);
    close(
        "n_pool = T_m L^q/B^q",
        b.n_pool,
        eq.land.market * b.l_s_q / b.b_s_q,
    );
    // Income four ways with the market's rent income (SSRN App. C).
    let mut hours_bill = 0.0;
    for w in &b.workers {
        hours_bill += w.wage * w.hours;
    }
    close("I = Y P_s", b.y * b.p_s, b.income);
    close(
        "I = sum v_i h_i + r T_m + interest",
        hours_bill + rent * eq.land.market + b.interest,
        b.income,
    );
    let mut spent = 0.0;
    for c in &b.categories {
        spent += c.price * c.output;
    }
    close("I = sum p_j z_j Y", spent, b.income);
    close(
        "provider baskets",
        b.provider_baskets,
        (rent * eq.land.market + b.interest) / b.p_s - economy.workers().support(),
    );
    // The exit sub-problem (§4.3, §4.4): each type's value, supply and plots.
    let p_g = eq.exit_good_price;
    let mut plots = 0.0;
    let (mut output, mut floor) = (0.0, 0.0);
    for i in 0..kinds {
        let w = &eq.workers[i];
        let ty = &p.worker_types[i];
        let wage = b.workers[i].wage;
        let supply = b.workers[i].supply;
        match p.exits[i] {
            ExitForm::Dependence => {
                assert_eq!((w.branch, w.exit_value), (Branch::Dependence, 0.0));
            }
            ExitForm::Priced(x) => {
                let want = match w.branch {
                    Branch::Plot => num::fma(p_g, x.gross, -(eq.plot_rent * x.plot)),
                    _ => p_g * x.floor,
                };
                assert_eq!(w.exit_value.to_bits(), want.to_bits(), "{}", at("e_i"));
                output += p_g * x.gross * w.plot_households;
                floor += p_g * x.floor * (w.exiters - w.plot_households);
                close_to("plot land", w.plot_land, x.plot * w.plot_households, 1e-12);
                // The branch follows q_o against q_enc (§4.3), and s_i = s(q_o): at the plot
                // rent and p_g, or at r = 0 with a free exit good at the wall's end, where the
                // good costs p_wall per unit of rent and a plot past the commons pays 1 (§12
                // item 16). A crowded commons' own rent is not reported there.
                let decided = match wall_price {
                    None => Some((eq.plot_rent, p_g)),
                    Some(p_wall) => match eq.exit_land {
                        ExitLand::Idle => Some((1.0, p_wall)),
                        ExitLand::Crowded => None,
                        _ => Some((0.0, p_wall)),
                    },
                };
                if let Some((r_o, price)) = decided {
                    if eq.enclosure.is_none() && eq.exit_land != ExitLand::Crowded {
                        let plot = r_o * x.plot < price * (x.gross - x.floor);
                        assert_eq!(plot, w.branch == Branch::Plot, "{}", at("branch"));
                    }
                    let goods = match w.branch {
                        Branch::Plot if r_o == 0.0 => x.gross,
                        Branch::Plot => num::fma(-(r_o / price), x.plot, x.gross),
                        _ => x.floor,
                    };
                    assert_eq!(w.exit_goods.to_bits(), goods.to_bits(), "{}", at("s_i"));
                }
            }
        }
        if b.workers[i].pooled {
            let n = match rule {
                Some(f) => f(i, wage, w.exit_value),
                None => {
                    let z = num::ln1p((wage - w.exit_value) / (ty.support * b.p_s + w.exit_value));
                    ty.workers * ty.work_cost.cdf(z)
                }
            };
            close_to("supply", supply, n, 1e-12);
            assert!(
                (w.exiters - (ty.workers - supply)).abs() <= 1e-12 * ty.workers,
                "{}",
                at("exiters")
            );
        }
        plots += w.plot_land;
        assert!(w.rented_land <= w.plot_land * (1.0 + 1e-12) + 1e-300);
    }
    close_to("home output", eq.home.output, output, 1e-12);
    close_to("home floor", eq.home.floor, floor, 1e-12);
    assert_eq!(eq.home.rent_in_kind, rent * eq.land.rented_plots);
    assert_eq!(
        eq.home.commons_shadow_rent,
        eq.plot_rent * eq.land.commons_occupied
    );
    // Each regime's condition (§4.4).
    match eq.exit_land {
        ExitLand::Unused => {
            assert_eq!(plots, 0.0);
            assert_eq!((eq.land.rented_plots, eq.land.commons_occupied), (0.0, 0.0));
        }
        ExitLand::Commons => {
            assert_eq!(eq.plot_rent, 0.0);
            assert!(plots <= t_o, "{}", at("the commons has room"));
            close("T_oc = G", eq.land.commons_occupied, plots);
            assert_eq!(eq.land.rented_plots, 0.0);
        }
        ExitLand::Crowded => {
            if rent == 1.0 {
                assert!(eq.plot_rent > 0.0 && eq.plot_rent < rent, "{}", at("r_o"));
            } else {
                // at r = 0 only a free exit good can crowd the commons, rationed at the wall's
                // end's rent while every rent in money is 0 (§12 item 16)
                assert!(wall_price.is_some(), "{}", at("crowded at r = 0"));
                assert_eq!(eq.plot_rent, 0.0);
            }
            assert_eq!(eq.land.commons_occupied, t_o);
            assert_eq!(eq.land.rented_plots, 0.0);
            let residual = (plots - t_o).abs() / t_o;
            assert_eq!(eq.residuals.commons.to_bits(), residual.to_bits());
            assert!(residual <= 1e-9, "{}", at("the commons clears"));
        }
        ExitLand::Enclosed | ExitLand::Idle => {
            // Enclosed pays the ruling rent while land is scarce; at r = 0 the spill stands
            // free on idle enclosed land, which is not enclosure (§0.2)
            let want = if rent == 1.0 {
                ExitLand::Enclosed
            } else {
                ExitLand::Idle
            };
            assert_eq!(eq.exit_land, want, "{}", at("enclosed or idle"));
            assert_eq!(eq.plot_rent, rent);
            if t_o > 0.0 && plots > 0.0 {
                assert_eq!(eq.land.commons_occupied, plots.min(t_o));
            }
            close_to(
                "T_p = G - T_o",
                eq.land.rented_plots,
                (plots - t_o).max(0.0),
                1e-12,
            );
        }
    }
    if eq.exit_land != ExitLand::Crowded {
        assert_eq!(eq.residuals.commons, 0.0);
    }
    // Coverage (SSRN eq 16), q and the parcels (§2.1).
    let mut people = 0.0;
    for ty in &p.worker_types {
        people += ty.workers;
    }
    assert_eq!(
        eq.coverage.to_bits(),
        ((rent * t) / (people * b.p_s)).to_bits()
    );
    if rent == 1.0 {
        assert_eq!(eq.q.to_bits(), (1.0 / p_g).to_bits());
    }
    let (mut used_enclosed, mut used_open) = (0.0, 0.0);
    for (z, parcel) in p.parcels.iter().enumerate() {
        let r = &eq.parcels[z];
        assert!((0.0..=1.0).contains(&r.used));
        let services = parcel.acreage * parcel.quality;
        match parcel.access {
            Access::Enclosed => {
                assert_eq!(r.rent_per_acre, rent * parcel.quality);
                assert_eq!(r.shadow_rent_per_acre, 0.0);
                used_enclosed += r.used * services;
            }
            Access::Open => {
                assert_eq!(r.rent_per_acre, 0.0);
                assert_eq!(r.shadow_rent_per_acre, eq.plot_rent * parcel.quality);
                used_open += r.used * services;
            }
        }
    }
    close_to(
        "the enclosed parcels in use",
        used_enclosed,
        if rent == 1.0 {
            t
        } else {
            eq.land.market + eq.land.rented_plots
        },
        1e-12,
    );
    assert!(
        (used_open - eq.land.commons_occupied).abs() <= 1e-12 * t_o.max(1e-300),
        "{}",
        at("the commons in use")
    );
    // Each type's rented plot land, in proportion to its plots, sums to T_p.
    let mut rented = 0.0;
    for w in &eq.workers {
        rented += w.rented_land;
    }
    if eq.land.rented_plots > 0.0 {
        close("the rented plots", rented, eq.land.rented_plots);
    } else {
        assert_eq!(rented, 0.0, "{}", at("no plot rented"));
    }
    // 1d's residuals, every one, on the market's land with the land priced at the rent (§4.8):
    // the cost system, the quantities, the income, the corner and the cheapest task type.
    let r = &b.residuals;
    for (name, value) in [
        ("income", r.income),
        ("land", r.land),
        ("services", r.services),
        ("user cost", r.user_cost),
        ("fork", r.fork),
        ("totals", r.totals),
        ("expenditure", r.expenditure),
        ("leontief price", r.leontief_price),
        ("leontief quantity", r.leontief_quantity),
        ("closure", r.closure),
        ("cheapest", r.cheapest),
        ("corner", r.corner),
        ("reserved", r.reserved),
        ("basket", r.basket),
    ] {
        assert!(value <= FULL, "{name} residual {value:e} {}", at(""));
    }
    // No task type is cheaper than the technique's (SSRN A.1), at every margin: on the
    // all-human corner the technique is the cheapest at the wage.
    let types = &p.machine_types;
    let task_price = b.types[b.technique].price / types[b.technique].task_efficiency;
    for (k, mt) in types.iter().enumerate() {
        if mt.task_efficiency > 0.0 {
            at_most(
                &at("no task type is cheaper"),
                task_price,
                b.types[k].price / mt.task_efficiency,
            );
        }
    }
    // Each category's wage ceiling v/(r·b̃_j + R_j), wage floor 1/(L̄_j + (r·b̄_j + R_j)/v) and
    // shares φ_w = (v·λ̃_j + R_j)/p_j and φ_r = r·b̃_j/p_j, with the land priced at the rent
    // (a category free at r = 0 reports the last three absent, §12 item 4).
    for c in &b.categories {
        let fixed = rent * c.b_tilde + c.reserved_cost;
        let want = (fixed > 0.0).then(|| b.v / fixed);
        assert_eq!(
            c.wage_ceiling.map(f64::to_bits),
            want.map(f64::to_bits),
            "{}",
            at("wage ceiling")
        );
        if c.price > 0.0 {
            let floor = 1.0 / (c.l_bar + (rent * c.chain_land + c.reserved_cost) / b.v);
            assert_eq!(
                c.wage_floor.to_bits(),
                floor.to_bits(),
                "{}",
                at("wage floor")
            );
            at_most(&at("floor <= v/p"), c.wage_floor, c.real_wage);
        }
        if let (Some(w), Some(r)) = (c.phi_w, c.phi_r) {
            let want_w = (b.v * c.lambda_tilde + c.reserved_cost) / c.price;
            let want_r = (rent * c.b_tilde) / c.price;
            assert_eq!(w.to_bits(), want_w.to_bits(), "{}", at("phi_w"));
            assert_eq!(r.to_bits(), want_r.to_bits(), "{}", at("phi_r"));
            if c.price > 0.0 {
                close(&at("phi_w + phi_r"), w + r, 1.0);
            }
        }
    }
    // Lemma B.1's flag with the market's land at x = 1 (§4.8).
    let env = economy.workers().machines().envelope();
    let one = economy.at_with(1.0, env.last());
    let lemma = one.point.short.is_none()
        && one.point.n_s > one.point.n_d
        && one.market_land > economy.workers().support() * one.point.p_s;
    assert_eq!(b.lemma_b1, lemma, "{}", at("lemma_b1"));
    // The margin (§4.8).
    match b.margin {
        Margin::Contestable => close("v = γ(x*)π", b.v, b.gamma_star * (b.v / b.g)),
        Margin::Wall => assert!(b.v >= b.replacement_top * (1.0 - 1e-12)),
        Margin::AllHuman => assert!(b.v <= b.replacement_bottom * (1.0 + 1e-12)),
    }
    // The pool clears.
    assert!(b.residuals.labor <= 1e-9 * b.n_pool, "{}", at("labor"));
    // The enclosure tie: q = q_enc and ψ in [0, 1] (§4.7).
    if let Some(tie) = eq.enclosure {
        assert!((0.0..=1.0).contains(&tie.share));
        let threshold = eq.workers[tie.worker].threshold.expect("a plot taker");
        close_to("q = q_enc", eq.q, threshold, 1e-12);
    }
    // The finiteness of every output is the solve's; the bits of the outputs are the fields'.
    let bits = bits_1e(eq);
    assert_eq!(
        bits.get("market_land"),
        Some(&Some(eq.land.market.to_bits()))
    );
}

/// L (docs/unit-1e.md §12 item 16): W3's economy (λ 0.6, χ_max 3) with N workers on
/// (LAND, 10, 1, enclosed) and exit (1.5·h + 0.5, 0, h) in Appendix B's space, a good made of
/// land alone, which is free at r = 0.
pub fn land_good(workers: f64, plot: f64) -> ParcelParams {
    ParcelParams {
        exits: vec![priced(1.5 * plot + 0.5, 0.0, plot)],
        exit_good: 1,
        ..goodspace_1e(workers, 10.0, 1.0, 0.6, 3.0)
    }
}
