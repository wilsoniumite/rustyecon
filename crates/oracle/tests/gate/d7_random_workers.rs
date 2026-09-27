//! d7: random economies (docs/unit-1d.md §8): §4.7's identities and bounds, the residuals, the
//! path and the count, and the worker types' status, on five sets drawn with SplitMix64 from
//! seeds 931 to 935 (1c's were 926-929).

use std::sync::OnceLock;

use oracle::{
    Category, Eq1d, MachineType, Margin, PowerSchedule, Recipe, Regime, SolveError, WorkerEconomy,
    WorkerParams, WorkerType,
};

use crate::g5_random_economies::SplitMix64;
use crate::support_1b::category;
use crate::support_1c::*;
use crate::support_1d::*;

/// The sets of docs/unit-1d.md §8.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Set {
    /// (a) ρ = 0, one machine type.
    One,
    /// (b) ρ = 0, K in 1..3.
    Zero,
    /// (c) ρ ~ U(0, 0.1), K in 1..3.
    Interest,
    /// (d) abundant labour: type 0 with N ~ U(20, 60) and χ_max ~ U(0.02, 0.2), ρ = 0.
    Abundant,
    /// (e) the switch set: two task types whose crossing is drawn on the line (even draws) or
    /// on the wall (odd draws), ρ ~ U(0, 0.15), and half the time type 0's N set so that the
    /// pool's supply at the switch lies between the two one-sided demands.
    Switch,
}

/// The first seed; the sets take 931 to 935.
const SEED_1D: u64 = 931;

/// Equilibria wanted per set.
const WANTED_1D: usize = 60;

/// The cap on draws per set; docs/unit-1d.md §12 records what each set needed.
const MAX_DRAWS_1D: usize = 3_000;

fn chance(rng: &mut SplitMix64, p: f64) -> bool {
    rng.uniform(0.0, 1.0) < p
}

/// 1c's m6 recipe: machines 0 w.p. 0.6, else U(0.02, 0.3); labour 0 w.p. 0.2, else
/// U(0.01, 0.5); land 0 w.p. 0.2, else U(0.05, 1).
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

/// The worker types, human-required and reserved hours of §8's table, for C categories of
/// which the last `sites` have no tasks.
fn draw_workers(
    rng: &mut SplitMix64,
    set: Set,
    count: usize,
    sites: usize,
) -> (Vec<WorkerType>, Vec<f64>, Vec<Vec<f64>>) {
    let kinds = 1 + (rng.next_u64() % 3) as usize;
    let mut types = vec![if set == Set::Abundant {
        worker(rng.uniform(20.0, 60.0), rng.uniform(0.02, 0.2), 1.0, 1.0)
    } else {
        worker(rng.uniform(1.0, 10.0), rng.uniform(0.3, 3.0), 1.0, 1.0)
    }];
    for _ in 1..kinds {
        let workers = rng.uniform(0.5, 5.0);
        let chi_max = rng.uniform(0.3, 3.0);
        let efficiency = if chance(rng, 0.15) {
            0.0
        } else {
            rng.uniform(0.5, 2.5)
        };
        types.push(worker(workers, chi_max, efficiency, rng.uniform(0.5, 2.0)));
    }
    let required: Vec<f64> = (0..count)
        .map(|j| {
            if j >= count - sites || chance(rng, 0.6) {
                0.0
            } else {
                rng.uniform(0.05, 0.4)
            }
        })
        .collect();
    let mut reserved: Vec<Vec<f64>> = (0..count)
        .map(|j| {
            (0..kinds)
                .map(|i| {
                    let zero = if i == 0 { 0.8 } else { 0.5 };
                    if j >= count - sites || chance(rng, zero) {
                        0.0
                    } else {
                        rng.uniform(0.02, 0.3)
                    }
                })
                .collect()
        })
        .collect();
    for i in 1..kinds {
        if types[i].efficiency == 0.0 && reserved.iter().all(|row| row[i] == 0.0) {
            reserved[0][i] = rng.uniform(0.02, 0.3);
        }
    }
    (types, required, reserved)
}

/// Sets (a)-(d): 1c's m6 table for the scalars, segments, categories, intermediate inputs and
/// machine types, with T ~ U(2, 12), then the worker types.
fn draw_general(rng: &mut SplitMix64, set: Set) -> WorkerParams {
    let land = rng.uniform(2.0, 12.0);
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
    let count = 2 + (rng.next_u64() % 3) as usize;
    let mut categories: Vec<Category> = Vec::with_capacity(count + 1);
    for j in 0..count {
        let density: Vec<f64> = (0..segments)
            .map(|_| {
                if chance(rng, 0.4) {
                    0.0
                } else {
                    rng.uniform(0.2, 3.0)
                }
            })
            .collect();
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
    let k = if set == Set::One {
        1
    } else {
        1 + (rng.next_u64() % 3) as usize
    };
    let machine_types = (0..k)
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
    let (worker_types, human_required, reserved) = draw_workers(rng, set, c, 1);
    WorkerParams {
        land,
        schedule,
        rho,
        machine_types,
        edges,
        categories,
        intermediate,
        worker_types,
        human_required,
        reserved,
    }
}

/// Set (e): the good and space; a labour-using type 0 and a flatter type 1 whose build land
/// puts the crossing at γ(x) for x ~ U(0.15, 0.85) (even draws) or at g ~ U(1, 3)·γ(1) on the
/// wall (odd draws); then the worker types, and half the time type 0's N set so that the pool's
/// supply at the switch lies midway between the two one-sided demands.
fn draw_switch(rng: &mut SplitMix64, index: usize) -> Option<WorkerParams> {
    let rho = rng.uniform(0.0, 0.15);
    let schedule = linear(1.0, rng.uniform(0.1, 0.5), rng.uniform(0.5, 2.0));
    let space = rng.uniform(0.1, 1.5);
    let land = rng.uniform(5.0, 20.0);
    let (d0, lag0) = (rng.uniform(0.2, 1.0), 1 + (rng.next_u64() % 3) as u32);
    let (d1, lag1) = (rng.uniform(0.02, 0.3), 1 + (rng.next_u64() % 6) as u32);
    let first = machine_type(
        1.0,
        recipe(&[0.0, 0.0], rng.uniform(0.05, 0.4), rng.uniform(0.1, 0.8)),
        recipe(&[0.0, 0.0], rng.uniform(0.0, 0.2), rng.uniform(0.0, 0.3)),
        d0,
        lag0,
    );
    let theta1 = rng.uniform(0.5, 2.0);
    let lam1 = rng.uniform(0.0, 0.05);
    let gamma = if index.is_multiple_of(2) {
        oracle::Schedule::gamma(&schedule, rng.uniform(0.15, 0.85))
    } else {
        oracle::Schedule::gamma(&schedule, 1.0) * rng.uniform(1.0, 3.0)
    };
    let set_workers = chance(rng, 0.5);
    let u0 = oracle::user_cost(rho, d0, lag0);
    let u1 = oracle::user_cost(rho, d1, lag1);
    let (lt0, bt0) = (
        first.operating.labor + u0 * first.build.labor,
        first.operating.land + u0 * first.build.land,
    );
    let lt1 = u1 * lam1;
    if lt1 / theta1 >= lt0 {
        return None;
    }
    let room0 = 1.0 - gamma * lt0;
    if room0 <= 0.0 || theta1 - gamma * lt1 <= 0.0 {
        return None;
    }
    let bt1 = bt0 * (theta1 - gamma * lt1) / room0;
    let second = machine_type(
        theta1,
        Recipe::zero(2),
        recipe(&[0.0, 0.0], lam1, bt1 / u1),
        d1,
        lag1,
    );
    let categories = vec![category(1.0, 0.0, &[1.0]), site(space, 1)];
    let (worker_types, human_required, reserved) = draw_workers(rng, Set::Switch, 2, 1);
    let mut params = WorkerParams {
        land,
        schedule,
        rho,
        machine_types: vec![first, second],
        edges: vec![0.0, 1.0],
        categories,
        intermediate: no_inputs(2),
        worker_types,
        human_required,
        reserved,
    };
    if set_workers {
        let e = WorkerEconomy::new(params.clone()).ok()?;
        let (below, above) = if let Some(&x) = e.switch_points().ok()?.first() {
            let s = e.machines().envelope().switches[0];
            (e.at_with(x, s.below), e.at_with(x, s.above))
        } else {
            let s = *e.wall_switches().first()?;
            (
                e.at_wage(1.0, s.wage, s.below),
                e.at_wage(1.0, s.wage, s.above),
            )
        };
        if below.short.is_some() || below.walled[0] {
            return Some(params);
        }
        let target = 0.5 * (below.n_d + above.n_d);
        let others = below.n_s - (below.supply[0] - below.reserved_demand[0]);
        let t = &params.worker_types[0];
        let share = t.work_cost.cdf(rustyecon_core::num::ln1p(
            below.wages[0] / (t.support * below.p_s),
        ));
        let workers = ((target - others) / t.efficiency + below.reserved_demand[0]) / share;
        if workers.is_finite() && (0.01..1e4).contains(&workers) {
            params.worker_types[0].workers = workers;
        }
    }
    Some(params)
}

/// A draw and its outcome.
struct Sample {
    set: Set,
    economy: WorkerEconomy,
    result: Result<Regime<Eq1d>, SolveError>,
}

impl Sample {
    fn equilibrium(&self) -> Option<&Eq1d> {
        match &self.result {
            Ok(Regime::Interior(eq)) => Some(eq),
            _ => None,
        }
    }
}

/// What a set's draws gave.
#[derive(Debug, Default)]
struct Tally {
    draws: usize,
    invalid: usize,
    contestable: usize,
    wall: usize,
    all_human: usize,
    with_walled: usize,
    ties: usize,
    short_pool: usize,
    short_reserved: usize,
    not_viable: usize,
    multiple: usize,
}

fn sample(set: Set, seed: u64) -> (Vec<Sample>, Tally) {
    let mut rng = SplitMix64(seed);
    let mut out = Vec::new();
    let mut tally = Tally::default();
    let mut found = 0;
    while found < WANTED_1D {
        assert!(tally.draws < MAX_DRAWS_1D, "{set:?}: {tally:?}");
        tally.draws += 1;
        let params = match set {
            Set::Switch => match draw_switch(&mut rng, tally.draws) {
                Some(p) => p,
                None => {
                    tally.invalid += 1;
                    continue;
                }
            },
            _ => draw_general(&mut rng, set),
        };
        let Ok(economy) = WorkerEconomy::new(params.clone()) else {
            tally.invalid += 1;
            continue;
        };
        let result = economy.solve();
        match &result {
            Ok(Regime::Interior(eq)) => {
                found += 1;
                match eq.margin {
                    Margin::Contestable => tally.contestable += 1,
                    Margin::Wall => tally.wall += 1,
                    Margin::AllHuman => tally.all_human += 1,
                }
                tally.with_walled += usize::from(eq.workers.iter().any(|w| !w.pooled));
                tally.ties += usize::from(eq.tie.is_some());
            }
            Ok(Regime::NotViable { .. }) => tally.not_viable += 1,
            Err(SolveError::LaborShort { reserved: None, .. }) => tally.short_pool += 1,
            Err(SolveError::LaborShort {
                reserved: Some(_), ..
            }) => tally.short_reserved += 1,
            Err(SolveError::MultipleEquilibria { .. }) => tally.multiple += 1,
            Err(e) => panic!("{params:?}: {e}"),
            Ok(other) => panic!("1d returned {}", other.name()),
        }
        out.push(Sample {
            set,
            economy,
            result,
        });
    }
    (out, tally)
}

/// The five sets, drawn once for every test of this module.
fn all_sets() -> &'static [(Vec<Sample>, Tally)] {
    static SETS: OnceLock<Vec<(Vec<Sample>, Tally)>> = OnceLock::new();
    SETS.get_or_init(|| {
        [
            Set::One,
            Set::Zero,
            Set::Interest,
            Set::Abundant,
            Set::Switch,
        ]
        .into_iter()
        .zip(SEED_1D..)
        .map(|(set, seed)| sample(set, seed))
        .collect()
    })
}

fn samples() -> impl Iterator<Item = &'static Sample> {
    all_sets().iter().flat_map(|(s, _)| s.iter())
}

#[test]
fn identities() {
    // §4.7's identities and bounds at every equilibrium, the prices pinned to their evaluation.
    let mut n = 0;
    for s in samples() {
        if let Some(eq) = s.equilibrium() {
            check_identities_1d(&s.economy, eq);
            n += 1;
        }
    }
    assert_eq!(n, 5 * WANTED_1D);
}

#[test]
fn residuals_recompute() {
    // Each residual equals its recomputation bit for bit (in check_identities_1d too), and each
    // is nonzero somewhere, so none is a constant.
    let mut nonzero = [false; 15];
    for s in samples() {
        if let Some(eq) = s.equilibrium() {
            let r = recomputed_1d(&s.economy, eq);
            assert_eq!(r, eq.residuals);
            let values = [
                r.labor,
                r.income,
                r.land,
                r.services,
                r.user_cost,
                r.fork,
                r.totals,
                r.expenditure,
                r.leontief_price,
                r.leontief_quantity,
                r.closure,
                r.cheapest,
                r.corner,
                r.reserved,
                r.basket,
            ];
            for (seen, v) in nonzero.iter_mut().zip(values) {
                *seen |= v != 0.0;
            }
        }
    }
    // The cheapest type's residual and the corner's are 0 wherever the technique is the
    // cheapest and the corner's inequality holds, which is every equilibrium; the other
    // thirteen are rounding and appear.
    let names = [
        "labor",
        "income",
        "land",
        "services",
        "user cost",
        "fork",
        "totals",
        "expenditure",
        "leontief price",
        "leontief quantity",
        "closure",
        "cheapest",
        "corner",
        "reserved",
        "basket",
    ];
    for (i, name) in names.iter().enumerate() {
        if *name != "cheapest" && *name != "corner" {
            assert!(nonzero[i], "{name} is 0 at every equilibrium");
        }
    }
}

/// The Proposition of §5.4: at ρ = 0 a switch lowers the pool's demand, strictly when the
/// basket has machine tasks (with none, the two techniques' quantities are the same).
fn lowers(below: &oracle::WorkerPoint, above: &oracle::WorkerPoint) -> bool {
    if below.m_s > 0.0 {
        above.n_d < below.n_d
    } else {
        above.n_d == below.n_d
    }
}

#[test]
fn path_and_count() {
    // f nonincreasing on a grid of every stretch within each technique; the count equals the
    // result; at ρ = 0 every switch lowers labour demand and no economy has more than one
    // equilibrium (§5.4).
    for s in samples() {
        if matches!(s.result, Ok(Regime::NotViable { .. })) {
            continue;
        }
        check_path_1d(&s.economy);
        check_count_1d(&s.economy, &s.result);
        let p = s.economy.params();
        if p.rho == 0.0 {
            assert!(
                !matches!(s.result, Err(SolveError::MultipleEquilibria { .. })),
                "{p:?}"
            );
            let env = s.economy.machines().envelope();
            for (sw, &x) in env.switches.iter().zip(&s.economy.switch_points().unwrap()) {
                let (b, a) = (
                    s.economy.at_with(x, sw.below),
                    s.economy.at_with(x, sw.above),
                );
                assert!(
                    lowers(&b, &a),
                    "a switch raises labour demand at rho 0: {p:?}"
                );
            }
            for w in s.economy.wall_switches() {
                let (b, a) = (
                    s.economy.at_wage(1.0, w.wage, w.below),
                    s.economy.at_wage(1.0, w.wage, w.above),
                );
                assert!(
                    lowers(&b, &a),
                    "a wall switch raises labour demand at rho 0: {p:?}"
                );
            }
        }
    }
}

#[test]
fn type_status() {
    // Every walled type's wage is above ε_i·v and its supply clears its reserved demand; every
    // pooled type's supply covers its reserved demand (§4.3).
    for s in samples() {
        if let Some(eq) = s.equilibrium() {
            let p = s.economy.params();
            for (i, w) in eq.workers.iter().enumerate() {
                let t = &p.worker_types[i];
                if w.pooled {
                    assert!(w.supply >= w.reserved_hours * (1.0 - 1e-12));
                    assert_eq!(w.wage, t.efficiency * eq.v);
                } else {
                    assert!(w.wage > t.efficiency * eq.v);
                    assert!((w.supply - w.reserved_hours).abs() <= 1e-12 * w.reserved_hours);
                }
            }
        }
    }
}

#[test]
fn the_draws_cover_the_regimes() {
    // Every margin, walled and pooled types, a type with ε = 0, ties on the line and on the
    // wall, both causes of LaborShort, and MultipleEquilibria at ρ > 0 (recorded if absent).
    let (mut margins, mut walled, mut pooled_other, mut zero_eff) =
        ([false; 3], false, false, false);
    let (mut line_tie, mut wall_tie) = (false, false);
    let (mut short_pool, mut short_reserved, mut multiple) = (false, false, 0);
    for s in samples() {
        match &s.result {
            Ok(Regime::Interior(eq)) => {
                margins[eq.margin.code() as usize] = true;
                walled |= eq.workers.iter().any(|w| !w.pooled);
                pooled_other |= eq.workers.iter().skip(1).any(|w| w.pooled);
                zero_eff |= eq.workers.iter().any(|w| w.efficiency == 0.0);
                line_tie |= eq.tie.is_some() && eq.margin == Margin::Contestable;
                wall_tie |= eq.tie.is_some() && eq.margin == Margin::Wall;
            }
            Err(SolveError::LaborShort { reserved, .. }) => {
                short_pool |= reserved.is_none();
                short_reserved |= reserved.is_some();
            }
            Err(SolveError::MultipleEquilibria { .. }) => {
                assert!(s.economy.params().rho > 0.0);
                multiple += 1;
            }
            _ => {}
        }
    }
    assert_eq!(margins, [true; 3], "every margin");
    assert!(walled && pooled_other && zero_eff);
    assert!(
        line_tie && wall_tie,
        "ties: line {line_tie}, wall {wall_tie}"
    );
    assert!(short_pool && short_reserved);
    for (samples, tally) in all_sets() {
        println!("d7 {:?}: {tally:?}", samples[0].set);
    }
    println!("d7 MultipleEquilibria at rho > 0: {multiple}");
}
