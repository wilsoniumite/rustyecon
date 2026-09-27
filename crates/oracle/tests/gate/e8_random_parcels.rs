//! e8: random economies (docs/unit-1e.md §8): §4.8's identities, the residuals, the count
//! against a scan 16 times finer, Lemma 5's sign, and the regimes, on eight sets drawn with
//! SplitMix64 from seeds 941 to 948 (1d's were 931-935).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use oracle::{
    Branch, Category, EnclosureSide, Eq1e, ExitForm, ExitLand, LandMarket, MachineType, Margin,
    ParcelEconomy, ParcelParams, PowerSchedule, Recipe, Regime, SolveError, WorkerType, EXIT_SCAN,
};
use rustyecon_core::num;

use crate::g5_random_economies::SplitMix64;
use crate::support_1b::category;
use crate::support_1c::*;
use crate::support_1d::*;
use crate::support_1e::*;

/// The sets of docs/unit-1e.md §8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Set {
    /// (a) one priced type, one machine type, ρ = 0.
    One,
    /// (b) 1-3 types, some in the dependence form, K in 1..3, ρ = 0.
    Types,
    /// (c) ρ ~ U(0, 0.1).
    Interest,
    /// (d) 1d's d7 draws with reserved hours, in the dependence form: the idle stretch and its
    /// edges.
    Reserved,
    /// (e) certified draws: the exit good given direct land ≥ h/s₀.
    Certified,
    /// (f) enclosure ties: N set so that the equilibrium lies in an enclosure point's jump.
    Ties,
    /// (g) as (a), with the exit good drawn among the categories, the land-only site included
    /// (docs/unit-1e.md §12 item 12).
    Goods,
    /// (h) as (f), on (g)'s draws.
    GoodTies,
}

/// Every set, in seed order.
const SETS: [Set; 8] = [
    Set::One,
    Set::Types,
    Set::Interest,
    Set::Reserved,
    Set::Certified,
    Set::Ties,
    Set::Goods,
    Set::GoodTies,
];

/// The first seed; the sets take 941 to 948.
const SEED_1E: u64 = 941;

/// Equilibria wanted per set.
const WANTED_1E: usize = 60;

/// The cap on draws per set; docs/unit-1e.md §12 records what each set needed.
const MAX_DRAWS_1E: usize = 4_000;

fn chance(rng: &mut SplitMix64, p: f64) -> bool {
    rng.uniform(0.0, 1.0) < p
}

/// 1c's m6 recipe, as d7's.
fn draw_recipe(rng: &mut SplitMix64, k: usize) -> Recipe {
    let machines = (0..k)
        .map(|_| {
            if chance(rng, 0.6) {
                0.0
            } else {
                rng.uniform(0.02, 0.3)
            }
        })
        .collect();
    let labor = if chance(rng, 0.2) {
        0.0
    } else {
        rng.uniform(0.01, 0.5)
    };
    let land = if chance(rng, 0.2) {
        0.0
    } else {
        rng.uniform(0.05, 1.0)
    };
    Recipe {
        machines,
        labor,
        land,
    }
}

/// A priced exit form of §8's table: s₀ ~ U(0.05, 1.5), s̲ 0 or U(0, s₀), h ~ U(0.01, 0.4).
fn draw_exit(rng: &mut SplitMix64) -> ExitForm {
    let gross = rng.uniform(0.05, 1.5);
    let floor = if chance(rng, 0.5) {
        0.0
    } else {
        rng.uniform(0.0, gross)
    };
    priced(gross, floor, rng.uniform(0.01, 0.4))
}

/// Sets (a)-(c) and (e): d7's table for the scalars, segments, categories and machine types,
/// then §8's parcels, exits and supports.
fn draw(rng: &mut SplitMix64, set: Set) -> ParcelParams {
    let schedule = PowerSchedule {
        eta: 1.0,
        g0: rng.uniform(0.01, 0.5),
        g1: rng.uniform(0.2, 1.5),
        k: rng.uniform(0.5, 4.0),
    };
    let rho = if set == Set::Interest {
        rng.uniform(0.0, 0.1)
    } else {
        0.0
    };
    let segments = 1 + (rng.next_u64() % 3) as usize;
    let widths: Vec<f64> = (0..segments).map(|_| rng.uniform(0.5, 1.5)).collect();
    let total: f64 = widths.iter().sum();
    let mut edges = vec![0.0];
    let mut run = 0.0;
    for w in &widths[..segments - 1] {
        run += w;
        edges.push(run / total);
    }
    edges.push(1.0);
    let count = 2 + (rng.next_u64() % 2) as usize;
    let mut categories: Vec<Category> = Vec::with_capacity(count + 1);
    for j in 0..count {
        let mut density: Vec<f64> = (0..segments)
            .map(|_| {
                if chance(rng, 0.4) {
                    0.0
                } else {
                    rng.uniform(0.2, 3.0)
                }
            })
            .collect();
        if j == 0 && density.iter().all(|&m| m == 0.0) {
            density[0] = 1.0;
        }
        let mut direct_land = if j == 0 || chance(rng, 0.2) {
            0.0
        } else {
            rng.uniform(0.05, 1.5)
        };
        if density.iter().all(|&m| m == 0.0) && direct_land == 0.0 {
            direct_land = 1.0;
        }
        categories.push(category(rng.uniform(0.1, 1.5), direct_land, &density));
    }
    categories.push(site(rng.uniform(0.2, 1.5), segments));
    let c = categories.len();
    let mut intermediate = vec![vec![0.0; c]; c];
    for (j, row) in intermediate.iter_mut().enumerate() {
        for (l, a) in row.iter_mut().enumerate() {
            let zero = if j == l { 0.9 } else { 0.7 };
            if !chance(rng, zero) {
                *a = rng.uniform(0.02, 0.3);
            }
        }
    }
    let k = if matches!(set, Set::Types | Set::Interest) {
        1 + (rng.next_u64() % 3) as usize
    } else {
        1
    };
    let machine_types: Vec<MachineType> = (0..k)
        .map(|t| {
            let task_efficiency = if t == 0 {
                1.0
            } else if chance(rng, 0.3) {
                0.0
            } else {
                rng.uniform(0.5, 2.0)
            };
            let mut operating = draw_recipe(rng, k);
            let mut build = draw_recipe(rng, k);
            let which = rng.uniform(0.0, 1.0);
            if which < 0.25 {
                operating = Recipe::zero(k);
            } else if which < 0.5 {
                build = Recipe::zero(k);
            }
            MachineType {
                task_efficiency,
                operating,
                build,
                delta: rng.uniform(0.05, 1.0),
                build_lag: 1 + (rng.next_u64() % 4) as u32,
            }
        })
        .collect();
    // The worker types and their exits.
    let kinds = if matches!(set, Set::Types | Set::Interest) {
        1 + (rng.next_u64() % 3) as usize
    } else {
        1
    };
    let mut worker_types: Vec<WorkerType> = Vec::with_capacity(kinds);
    let mut exits = Vec::with_capacity(kinds);
    for i in 0..kinds {
        let support = if chance(rng, 0.5) {
            1.0
        } else {
            rng.uniform(0.05, 2.0)
        };
        let efficiency = if i == 0 { 1.0 } else { rng.uniform(0.5, 2.5) };
        worker_types.push(worker(
            rng.uniform(1.0, 10.0),
            rng.uniform(0.3, 3.0),
            efficiency,
            support,
        ));
        let dependence = matches!(set, Set::Types | Set::Interest) && !chance(rng, 0.7);
        exits.push(if dependence {
            ExitForm::Dependence
        } else {
            draw_exit(rng)
        });
    }
    if set == Set::Certified {
        if let ExitForm::Priced(x) = exits[0] {
            categories[0].direct_land = rng.uniform(1.0, 2.0) * x.plot / x.gross;
        }
    }
    let exit_good = if set == Set::Goods {
        (rng.next_u64() % c as u64) as usize
    } else {
        0
    };
    // The parcels.
    let mut parcels = vec![enclosed(rng.uniform(2.0, 12.0), 1.0)];
    if chance(rng, 0.5) {
        parcels.push(enclosed(rng.uniform(1.0, 5.0), rng.uniform(0.2, 2.0)));
    }
    if chance(rng, 0.65) {
        parcels.push(open(rng.uniform(0.02, 1.5), 1.0));
    }
    ParcelParams {
        parcels,
        schedule,
        rho,
        machine_types,
        edges,
        categories,
        intermediate,
        human_required: vec![0.0; c],
        reserved: vec![vec![0.0; kinds]; c],
        worker_types,
        exits,
        exit_good,
    }
}

/// Sets (f) and (h): a set (a) or (g) draw with N set between the two values of its first
/// enclosure point's zero crossings, so that the equilibrium lies in the jump; `None` when it
/// has no point.
fn draw_tie(rng: &mut SplitMix64, set: Set) -> Option<ParcelParams> {
    let params = draw(rng, set);
    let economy = ParcelEconomy::new(params.clone()).ok()?;
    let point = *economy.enclosure_points().ok()?.first()?;
    let (land, commons) = (economy.enclosed_land(), economy.commons());
    let plot = match params.exits[0] {
        ExitForm::Priced(x) => x.plot,
        ExitForm::Dependence => return None,
    };
    let at = |workers: f64, side: EnclosureSide| -> Option<f64> {
        let mut p = params.clone();
        p.worker_types[0].workers = workers;
        let e = ParcelEconomy::new(p).ok()?;
        Some(e.at_enclosure(&point, side).excess_demand())
    };
    let top = 0.999 * (land + commons) / plot;
    let root = |side: EnclosureSide| -> Option<f64> {
        let (mut lo, mut hi) = (1e-3, top);
        if at(lo, side)? <= 0.0 || at(hi, side)? >= 0.0 {
            return None;
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if at(mid, side)? > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(0.5 * (lo + hi))
    };
    let (below, above) = (root(EnclosureSide::Below)?, root(EnclosureSide::Above)?);
    if above >= below {
        return None;
    }
    let mut p = params;
    p.worker_types[0].workers = 0.5 * (above + below);
    Some(p)
}

/// One draw and what 1e gave.
struct Sample {
    set: Set,
    economy: ParcelEconomy,
    result: Result<Regime<Eq1e>, SolveError>,
}

impl Sample {
    fn equilibrium(&self) -> Option<&Eq1e> {
        match &self.result {
            Ok(Regime::Interior(eq)) => Some(eq),
            _ => None,
        }
    }
}

/// What a set's draws gave, by label.
type Tally = BTreeMap<String, usize>;

fn label(result: &Result<Regime<Eq1e>, SolveError>) -> Vec<String> {
    match result {
        Ok(Regime::Interior(eq)) => {
            let mut out = vec![
                format!("{:?}", eq.base.margin),
                format!("{:?}", eq.land_market),
                format!("exit {:?}", eq.exit_land),
            ];
            if eq.enclosure.is_some() {
                out.push(format!("tie {:?}", eq.base.margin));
            }
            if eq.base.tie.is_some() {
                out.push("technique tie".to_string());
            }
            if eq.base.workers.iter().any(|w| w.edge) {
                out.push("edge".to_string());
            }
            out
        }
        Ok(Regime::NotViable { .. }) => vec!["NotViable".to_string()],
        Ok(other) => vec![other.name().to_string()],
        Err(SolveError::NoMarket { .. }) => vec!["NoMarket".to_string()],
        Err(SolveError::MultipleEquilibria { .. }) => vec!["MultipleEquilibria".to_string()],
        Err(e) => vec![format!("error {e}")],
    }
}

fn sample(set: Set, seed: u64) -> (Vec<Sample>, Tally) {
    let mut rng = SplitMix64(seed);
    let mut out = Vec::new();
    let mut tally = Tally::new();
    let (mut found, mut draws) = (0, 0);
    let mut reserved = crate::d7_random_workers::all_sets()
        .iter()
        .flat_map(|(s, _)| s.iter())
        .filter(|s| {
            s.economy
                .params()
                .reserved
                .iter()
                .flatten()
                .any(|&r| r > 0.0)
        });
    while found < WANTED_1E {
        assert!(draws < MAX_DRAWS_1E, "{set:?}: {tally:?}");
        draws += 1;
        let params = match set {
            Set::Reserved => match reserved.next() {
                Some(s) => e0(s.economy.params().clone()),
                None => break,
            },
            Set::Ties | Set::GoodTies => match draw_tie(
                &mut rng,
                if set == Set::Ties {
                    Set::One
                } else {
                    Set::Goods
                },
            ) {
                Some(p) => p,
                None => {
                    *tally.entry("no tie".to_string()).or_default() += 1;
                    continue;
                }
            },
            _ => draw(&mut rng, set),
        };
        let Ok(economy) = ParcelEconomy::new(params.clone()) else {
            *tally.entry("invalid".to_string()).or_default() += 1;
            continue;
        };
        if set == Set::Certified && !economy.certified() {
            *tally.entry("not certified".to_string()).or_default() += 1;
            continue;
        }
        let result = economy.solve();
        if let Err(e) = &result {
            assert!(
                matches!(
                    e,
                    SolveError::NoMarket { .. } | SolveError::MultipleEquilibria { .. }
                ),
                "{set:?} draw {draws}: {e}"
            );
        }
        found += usize::from(result.is_ok() && matches!(result, Ok(Regime::Interior(_))));
        for l in label(&result) {
            *tally.entry(l).or_default() += 1;
        }
        out.push(Sample {
            set,
            economy,
            result,
        });
    }
    *tally.entry("draws".to_string()).or_default() += draws;
    (out, tally)
}

/// The six sets, drawn once for every test of this module.
fn all_sets() -> &'static [(Vec<Sample>, Tally)] {
    static SAMPLES: OnceLock<Vec<(Vec<Sample>, Tally)>> = OnceLock::new();
    SAMPLES.get_or_init(|| {
        SETS.into_iter()
            .zip(SEED_1E..)
            .map(|(set, seed)| sample(set, seed))
            .collect()
    })
}

fn samples() -> impl Iterator<Item = &'static Sample> {
    all_sets().iter().flat_map(|(s, _)| s.iter())
}

#[test]
fn identities() {
    let mut n = 0;
    for s in samples() {
        if let Some(eq) = s.equilibrium() {
            check_identities_1e(&s.economy, eq);
            n += 1;
        }
    }
    assert!(n >= 5 * WANTED_1E, "{n}");
}

#[test]
fn residuals_recompute() {
    // Each new residual equals its recomputation bit for bit (check_identities_1e), and each
    // is nonzero somewhere, so neither is a constant.
    let (mut partition, mut commons) = (false, false);
    for s in samples() {
        if let Some(eq) = s.equilibrium() {
            partition |= eq.residuals.partition > 0.0;
            commons |= eq.residuals.commons > 0.0;
        }
    }
    assert!(partition && commons, "{partition} {commons}");
}

#[test]
fn count_against_a_fine_scan() {
    // §5.4-5.5: the count equals a scan 16 times finer, or the miss is recorded (none on these
    // draws); in set (e) f is nonincreasing along the path and the equilibrium unique.
    let (mut compared, mut monotone) = (0, 0);
    for (samples, _) in all_sets() {
        for s in samples.iter().take(30) {
            if s.economy.plot_takers().is_empty() {
                continue;
            }
            let fine = s.economy.solve_scanned(16 * EXIT_SCAN);
            match (&s.result, &fine) {
                (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) => {
                    let (mut a, mut b) = (bits_1e(a), bits_1e(b));
                    a.remove("scan_points");
                    b.remove("scan_points");
                    assert_eq!(a, b, "{:?}", s.set);
                }
                (a, b) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{:?}", s.set),
            }
            if s.set == Set::Certified {
                assert!(s.economy.certified());
                assert!(!matches!(
                    s.result,
                    Err(SolveError::MultipleEquilibria { .. })
                ));
                // a viable economy's f is nonincreasing along the whole path (§5.4)
                let f = match s.result {
                    Ok(Regime::NotViable { .. }) => Vec::new(),
                    _ => path_values(&s.economy, 64),
                };
                for (k, w) in f.windows(2).enumerate() {
                    assert!(
                        w[1] <= w[0] + 1e-12 * w[0].abs().max(1.0),
                        "set (e): f rises at {k} of the path: {w:?}"
                    );
                }
                monotone += usize::from(!f.is_empty());
            }
            compared += 1;
        }
    }
    assert!(compared >= 100 && monotone >= 25, "{compared} {monotone}");
}

/// f along the whole path of a priced economy in path order (docs/unit-1e.md §5.3 step 2), `n`
/// points per piece: the all-human corner in v, each region of the line in x (both values at a
/// switch), each piece of the wall in v under its technique (both values at a wall switch),
/// the last piece to a million times its start, and the idle stretch in T_m from its start to
/// 0. Where Proposition 5 certifies the economy f is nonincreasing along all of it (§5.4).
fn path_values(e: &ParcelEconomy, n: usize) -> Vec<f64> {
    let w = e.workers();
    let env = w.machines().envelope();
    let mut f = Vec::new();
    let v0 = e.at_with(0.0, env.first).point.v;
    for k in 1..=n {
        let v = v0 * (k as f64 / n as f64);
        f.push(e.at_wage(0.0, v, env.first).excess_demand());
    }
    let points = w.switch_points().unwrap();
    let techniques: Vec<usize> = std::iter::once(env.first)
        .chain(env.switches.iter().map(|s| s.above))
        .collect();
    let bounds: Vec<f64> = std::iter::once(0.0)
        .chain(points.iter().copied())
        .chain(std::iter::once(1.0))
        .collect();
    for (r, &t) in techniques.iter().enumerate() {
        let (a, b) = (bounds[r], bounds[r + 1]);
        for k in 0..=n {
            f.push(
                e.at_with(a + (b - a) * (k as f64 / n as f64), t)
                    .excess_demand(),
            );
        }
    }
    let mut v_lo = e.at_with(1.0, env.last()).point.v;
    let mut technique = env.last();
    for sw in w.wall_switches() {
        for k in 0..=n {
            let v = v_lo + (sw.wage - v_lo) * (k as f64 / n as f64);
            f.push(e.at_wage(1.0, v, technique).excess_demand());
        }
        technique = sw.above;
        v_lo = sw.wage;
    }
    for k in 0..=n {
        let v = v_lo * num::pow(1e6, k as f64 / n as f64);
        f.push(e.at_wage(1.0, v, technique).excess_demand());
    }
    let start = e.enclosed_land() - e.at_idle(e.enclosed_land(), technique).rented_plots;
    for k in 0..=n {
        let t_m = start * (1.0 - k as f64 / n as f64);
        f.push(e.at_idle(t_m, technique).excess_demand());
    }
    f
}

#[test]
fn supply_slope() {
    // Lemma 5 at every equilibrium on the line with a plot-taking type: each type's marginal
    // cost moves with σ_i's sign as x rises, at the equilibrium's plot rent.
    let mut checked = 0;
    for s in samples() {
        let Some(eq) = s.equilibrium() else { continue };
        if eq.base.margin != Margin::Contestable || eq.enclosure.is_some() || eq.base.tie.is_some()
        {
            continue;
        }
        let e = &s.economy;
        let p = e.params();
        let (x, t) = (eq.base.x_star, eq.base.technique);
        let dx = 1e-7 * x.max(1e-3);
        let (a, b) = (e.at_with(x, t), e.at_with(x + dx, t));
        if a.branches != b.branches || a.exit_land != b.exit_land || x + dx >= 1.0 {
            continue;
        }
        let land = e.at_wage(x, 0.0, t);
        let (b_s, b_g) = (land.point.base_p_s, land.point.base_prices[p.exit_good]);
        for (i, ty) in p.worker_types.iter().enumerate() {
            let exit = |q: &oracle::ParcelPoint| match (p.exits[i], a.branches[i]) {
                (ExitForm::Priced(f), Branch::Plot) => {
                    num::fma(q.exit_good_price, f.gross, -(a.plot_rent * f.plot))
                }
                (ExitForm::Priced(f), _) => q.exit_good_price * f.floor,
                (ExitForm::Dependence, _) => 0.0,
            };
            let z = |q: &oracle::ParcelPoint| {
                let e_i = exit(q);
                num::ln1p((ty.efficiency * q.point.v - e_i) / (ty.support * q.point.p_s + e_i))
            };
            let dz = z(&b) - z(&a);
            let (v, p_s, e_i) = (a.point.v, a.point.p_s, exit(&a));
            let scale = ty.support * p_s + ty.efficiency * v;
            let sigma = ty.support * b_s * (ty.efficiency * v - e_i)
                + match (p.exits[i], a.branches[i]) {
                    (ExitForm::Priced(f), Branch::Plot) => {
                        (f.gross * b_g - a.plot_rent * f.plot) * scale
                    }
                    (ExitForm::Priced(f), _) => f.floor * b_g * scale,
                    (ExitForm::Dependence, _) => 0.0,
                };
            if dz.abs() > 1e-12 && sigma.abs() > 1e-9 {
                assert_eq!(dz > 0.0, sigma > 0.0, "{:?}: type {i}", s.set);
                checked += 1;
            }
        }
    }
    assert!(checked >= 50, "{checked}");
}

#[test]
fn the_draws_cover_the_regimes() {
    // Every exit-land regime (plots free on idle land among them), both land markets, every
    // margin, enclosure ties on the line, the wall and the all-human corner, NoMarket and
    // MultipleEquilibria; and in sets (g) and (h) exit goods other than category 0 on the wall,
    // at enclosure ties, and free at r = 0 (docs/unit-1e.md §12 items 12 and 16).
    let mut all = Tally::new();
    for (set, (_, tally)) in SETS.iter().zip(all_sets()) {
        println!("e8 {set:?}: {tally:?}");
        for (k, v) in tally {
            *all.entry(k.clone()).or_default() += v;
        }
    }
    for key in [
        "exit Unused",
        "exit Commons",
        "exit Crowded",
        "exit Enclosed",
        "exit Idle",
        "Scarce",
        "Idle",
        "Contestable",
        "Wall",
        "AllHuman",
        "tie Contestable",
        "tie Wall",
        "tie AllHuman",
        "NoMarket",
        "MultipleEquilibria",
    ] {
        assert!(all.contains_key(key), "no {key}: {all:?}");
    }
    let ties = all_sets()[5]
        .0
        .iter()
        .filter(|s| s.equilibrium().is_some_and(|eq| eq.enclosure.is_some()))
        .count();
    assert!(ties >= 40, "{ties}");
    // the exit good elsewhere than category 0
    let other = |set: usize, want: &dyn Fn(&Eq1e) -> bool| {
        all_sets()[set]
            .0
            .iter()
            .filter(|s| s.economy.params().exit_good != 0)
            .filter(|s| s.equilibrium().is_some_and(want))
            .count()
    };
    let on_the_wall = other(6, &|eq| eq.base.margin == Margin::Wall);
    let at_ties = other(7, &|eq| eq.enclosure.is_some());
    let wall_ties = other(7, &|eq| {
        eq.enclosure.is_some() && eq.base.margin == Margin::Wall
    });
    let free = other(6, &|eq| {
        eq.land_market == LandMarket::Idle && eq.exit_good_price == 0.0
    });
    println!("e8 other exit goods: {on_the_wall} on the wall, {at_ties} ties");
    println!("e8 other exit goods: {wall_ties} ties on the wall, {free} free on idle land");
    assert!(on_the_wall >= 5 && at_ties >= 20 && wall_ties >= 5 && free >= 1);
    let _ = ExitLand::Unused;
}
