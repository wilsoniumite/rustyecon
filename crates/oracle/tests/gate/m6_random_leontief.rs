//! m6: random economies (docs/unit-1c.md §8): the Leontief identities and the income identity
//! with interest on random instances (PLAN Phase 1's gate), the fork and bounds through
//! intermediate inputs, the closure and the cheapest type, the residuals, the root and the
//! technique regions, and ties and multiple equilibria on a set built to switch mid-line.

use oracle::{
    Category, Eq1c, MachineEconomy, MachineParams, MachineType, PowerSchedule, Recipe, Regime,
    SolveError, UniformWorkCost,
};

use crate::g5_random_economies::SplitMix64;
use crate::support::*;
use crate::support_1b::at_most;
use crate::support_1c::*;

/// The sets of docs/unit-1c.md §8.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Set {
    /// (a) ρ = 0.
    Zero,
    /// (b) ρ ~ U(0, 0.1).
    Interest,
    /// (c) ρ ~ U(0, 0.1), with at least one intermediate input.
    Chain,
    /// (d) Two task types with the switch drawn mid-line, and N set half the time so that n_S
    /// at the switch lies between the two one-sided n_D.
    Switch,
}

/// The first seed, 926 (1b's sets were 923-925); the sets take 926 to 929.
const SEED_1C: u64 = 926;

/// Interior economies (ties included) wanted per set.
const WANTED_1C: usize = 60;

/// The cap on draws per set. On 2026-09-27 the sets needed 213, 180, 217 and 149 draws
/// (docs/unit-1c.md §12).
const MAX_DRAWS_1C: usize = 2_000;

/// True with probability `p`.
fn chance(rng: &mut SplitMix64, p: f64) -> bool {
    rng.uniform(0.0, 1.0) < p
}

/// A recipe from §8's table over K types: machines 0 w.p. 0.6, else U(0.02, 0.3); labour 0 w.p.
/// 0.2, else U(0.01, 0.5); land 0 w.p. 0.2, else U(0.05, 1).
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

/// One draw of sets (a)-(c): 1b's C5 table for the scalars, segments and categories, then the
/// intermediate inputs and the machine types, in the order written.
fn draw_general(rng: &mut SplitMix64, set: Set) -> MachineParams {
    let workers = rng.uniform(1.0, 5.0);
    let land = rng.uniform(2.0, 20.0);
    let schedule = PowerSchedule {
        eta: 1.0,
        g0: rng.uniform(0.01, 0.5),
        g1: rng.uniform(0.2, 1.5),
        k: rng.uniform(0.5, 6.0),
    };
    let chi_max = rng.uniform(0.5, 3.0);
    let rho = if set == Set::Zero {
        0.0
    } else {
        rng.uniform(0.0, 0.1)
    };
    // The task line and the categories, as 1b's C5.
    let segments = 1 + (rng.next_u64() % 4) as usize;
    let widths: Vec<f64> = (0..segments).map(|_| rng.uniform(0.5, 1.5)).collect();
    let total: f64 = widths.iter().sum();
    let mut edges = vec![0.0];
    let mut run = 0.0;
    for w in &widths[..segments - 1] {
        run += w;
        edges.push(run / total);
    }
    edges.push(1.0);
    let count = 2 + (rng.next_u64() % 4) as usize;
    let mut categories = Vec::with_capacity(count + 1);
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
        let weight = if j >= 2 && chance(rng, 0.1) {
            0.0
        } else {
            rng.uniform(0.1, 1.5)
        };
        if density.iter().all(|&m| m == 0.0) && direct_land == 0.0 {
            direct_land = 1.0;
        }
        categories.push(Category {
            weight,
            direct_land,
            density,
        });
    }
    if chance(rng, 0.5) {
        categories.push(site(rng.uniform(0.2, 1.5), segments));
    }
    // Intermediate inputs: 0 w.p. 0.7 off the diagonal and 0.9 on it, else U(0.02, 0.3).
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
    if set == Set::Chain && (0..c).all(|j| (0..c).all(|l| j == l || intermediate[j][l] == 0.0)) {
        intermediate[1][0] = rng.uniform(0.02, 0.3);
    }
    // The machine types.
    let k = 1 + (rng.next_u64() % 4) as usize;
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
                build_lag: 1 + (rng.next_u64() % 5) as u32,
            }
        })
        .collect();
    MachineParams {
        workers,
        land,
        schedule,
        work_cost: UniformWorkCost { chi_max },
        rho,
        machine_types,
        edges,
        categories,
        intermediate,
    }
}

/// One draw of set (d): the good and space; a labour-using type 0 and a flatter type 1 whose
/// build land puts the crossing at γ(x) for x ~ U(0.15, 0.85); ρ = 0 on even draws and
/// U(0.02, 0.12) on odd ones; and, half the time, N such that n_S at the switch is midway
/// between the two one-sided n_D (after the prototype's switch scan, docs/unit-1c.md §2.5).
fn draw_switch(rng: &mut SplitMix64, index: usize) -> Option<MachineParams> {
    let rho = if index.is_multiple_of(2) {
        0.0
    } else {
        rng.uniform(0.02, 0.12)
    };
    let schedule = linear(1.0, rng.uniform(0.1, 0.5), rng.uniform(0.5, 2.0));
    let space = rng.uniform(0.1, 1.5);
    let workers = rng.uniform(1.0, 20.0);
    let land = rng.uniform(5.0, 20.0);
    let chi_max = rng.uniform(0.5, 3.0);
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
    let x_switch = rng.uniform(0.15, 0.85);
    let set_workers = chance(rng, 0.5);
    // Totals without machine inputs: λ̃ = λ^op + uλ^I, b̃ = b^op + u·b^I.
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
    // Equal closure wages at γ_sw: b̃_1 = b̃_0 (θ_1 − γλ̃_1)/(θ_0 − γλ̃_0).
    let gamma = oracle::Schedule::gamma(&schedule, x_switch);
    let room0 = 1.0 - gamma * lt0;
    if room0 <= 0.0 {
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
    let mut params = MachineParams {
        workers,
        land,
        work_cost: UniformWorkCost { chi_max },
        ..appendix_b_household(schedule, rho, space, vec![first, second])
    };
    if set_workers {
        let e = MachineEconomy::new(params.clone()).ok()?;
        let x = *e.switch_points().ok()?.first()?;
        let s = e.envelope().switches[0];
        let (below, above) = (e.at_with(x, s.below), e.at_with(x, s.above));
        let f = below.n_s / params.workers;
        let mid = 0.5 * (below.n_d + above.n_d) / f;
        if !(mid.is_finite() && mid > 0.0) {
            return None;
        }
        params.workers = mid;
    }
    Some(params)
}

/// A draw's outcome.
pub(crate) struct Sample {
    pub(crate) set: Set,
    pub(crate) economy: MachineEconomy,
    pub(crate) result: Result<Regime<Eq1c>, SolveError>,
}

impl Sample {
    fn interior(&self) -> Option<&Eq1c> {
        match &self.result {
            Ok(Regime::Interior(eq)) => Some(eq),
            _ => None,
        }
    }
}

/// What a set's draws gave.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Tally {
    pub(crate) draws: usize,
    pub(crate) invalid: usize,
    pub(crate) interior: usize,
    pub(crate) ties: usize,
    pub(crate) boundary: usize,
    pub(crate) not_viable: usize,
    pub(crate) no_interior_at_zero: usize,
    pub(crate) multiple: usize,
}

/// Every valid draw of a set until it has WANTED_1C interior economies, and the tally.
fn sample(set: Set, seed: u64) -> (Vec<Sample>, Tally) {
    let mut rng = SplitMix64(seed);
    let mut out = Vec::new();
    let mut tally = Tally::default();
    while tally.interior < WANTED_1C {
        assert!(tally.draws < MAX_DRAWS_1C, "{set:?}: {tally:?}");
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
        let Ok(economy) = MachineEconomy::new(params.clone()) else {
            tally.invalid += 1;
            continue;
        };
        let result = economy.solve();
        match &result {
            Ok(Regime::Interior(eq)) => {
                tally.interior += 1;
                if eq.tie.is_some() {
                    tally.ties += 1;
                }
            }
            Ok(Regime::BoundaryNoMargin { .. }) => tally.boundary += 1,
            Ok(Regime::NotViable { .. }) => tally.not_viable += 1,
            Ok(Regime::NoInteriorAtZero { .. }) => tally.no_interior_at_zero += 1,
            Err(SolveError::MultipleEquilibria { .. }) => tally.multiple += 1,
            Err(e) => panic!("{params:?}: {e}"),
        }
        out.push(Sample {
            set,
            economy,
            result,
        });
    }
    (out, tally)
}

/// The four sets.
fn all_sets() -> Vec<(Vec<Sample>, Tally)> {
    [Set::Zero, Set::Interest, Set::Chain, Set::Switch]
        .into_iter()
        .zip(SEED_1C..)
        .map(|(set, seed)| sample(set, seed))
        .collect()
}

/// Every interior sample of every set.
fn interiors() -> Vec<Sample> {
    all_sets()
        .into_iter()
        .flat_map(|(samples, _)| samples)
        .filter(|s| s.interior().is_some())
        .collect()
}

#[test]
fn identities() {
    // Income by Y·P_s, by Σ p_j z_j Y and by pᵀf, with interest; land, services and the
    // user-cost rows; eq 11; the basket count; interest = Σρω_kV_kX_k (§4.5-4.6).
    let samples = interiors();
    assert_eq!(samples.len(), 4 * WANTED_1C);
    for s in &samples {
        check_identities_1c(&s.economy, s.interior().unwrap());
    }
}

#[test]
fn leontief_identities() {
    // p = Ap + λv + b over every row, (I − Â)λ̃ = λ̂, f = (I − A^qᵀ)y, N_a = λ^qᵀy and
    // T = b^qᵀy, all by multiplication (SSRN eq 3-4, A.1).
    for s in &interiors() {
        check_leontief_1c(&s.economy, s.interior().unwrap());
    }
}

#[test]
fn fork_and_bounds() {
    // Both fork forms, the bounds and the pair for every category, bought or not, and
    // v/P_s ≤ v/B_s (§4.4).
    let mut unbought = 0;
    for s in &interiors() {
        let eq = s.interior().unwrap();
        check_fork_and_bounds_1c(&s.economy, eq);
        unbought += s
            .economy
            .params()
            .categories
            .iter()
            .filter(|c| c.weight == 0.0)
            .count();
    }
    assert!(unbought > 0, "no unbought category was checked");
}

#[test]
fn closure_and_cheapest() {
    // SSRN A.1's closure for the technique, and every other task type's delivered cost at
    // least the technique's (§4.3).
    let mut unused = 0;
    for s in &interiors() {
        let eq = s.interior().unwrap();
        let p = s.economy.params();
        let tau = &eq.types[eq.technique];
        let theta = p.machine_types[eq.technique].task_efficiency;
        close(
            "closure",
            eq.gamma_star * tau.b_tilde / (theta - eq.gamma_star * tau.lambda_tilde),
            eq.v,
        );
        let pi = tau.price / theta;
        for (k, t) in eq.types.iter().enumerate() {
            let theta_k = p.machine_types[k].task_efficiency;
            if theta_k > 0.0 {
                at_most("cheapest", pi, t.price / theta_k);
                if let Some(w) = t.closure_wage {
                    at_most("closure wage >= v", eq.v, w);
                }
                if t.task_services == 0.0 {
                    unused += 1;
                }
            }
        }
    }
    assert!(unused > 0);
}

#[test]
fn residuals_recompute() {
    // Each residual equals its recomputation bit for bit, and each is nonzero somewhere.
    let mut nonzero = [false; 12];
    for s in &interiors() {
        let eq = s.interior().unwrap();
        let r = eq.residuals;
        assert_eq!(recomputed_1c(&s.economy, eq), r);
        for (i, v) in [
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
        ]
        .iter()
        .enumerate()
        {
            nonzero[i] |= *v > 0.0;
        }
    }
    // `cheapest` is 0 unless a type undercuts the technique by rounding: at a tie below the
    // switch point the type above does, by an ulp.
    assert_eq!(nonzero, [true; 12], "labor, income, land, services, user cost, fork, totals, expenditure, leontief price, quantity, closure, cheapest");
}

#[test]
fn root_and_regions() {
    // x* is the double where f under the technique changes sign, or the tie's switch point;
    // f is nonincreasing in each region; the sign-change count is the regime (§5.3, §5.5).
    for (samples, _) in all_sets() {
        for s in &samples {
            // Beyond the viability edge the regions' prices mean nothing (1a's convention).
            if matches!(s.result, Ok(Regime::NotViable { .. })) {
                continue;
            }
            let changes = check_regions(&s.economy);
            match &s.result {
                Ok(Regime::Interior(eq)) => {
                    assert_eq!(changes, 1, "{:?}", s.economy.params());
                    match eq.tie {
                        None => assert_root_1c(&s.economy, eq.x_star, eq.technique),
                        Some(_) => assert!(eq.switches.iter().any(|w| w.x == eq.x_star)),
                    }
                }
                Err(SolveError::MultipleEquilibria { sign_changes, .. }) => {
                    assert_eq!(changes, *sign_changes);
                    assert!(changes >= 3 && changes % 2 == 1);
                }
                Ok(_) => assert_eq!(changes, 0),
                Err(e) => panic!("{e}"),
            }
        }
    }
}

#[test]
fn ties_and_multiplicity() {
    // In the switch set: every tie's identities hold; at ρ = 0 there is no MultipleEquilibria
    // and every jump is downward; at ρ > 0 each MultipleEquilibria has an upward jump, and an
    // odd number of sign changes above one on re-evaluation (§4.7, §5.5).
    let (samples, tally) = sample(Set::Switch, SEED_1C + 3);
    let (mut down_at_zero, mut up) = (0, 0);
    for s in &samples {
        let e = &s.economy;
        let rho = e.params().rho;
        for (w, x) in e.envelope().switches.iter().zip(e.switch_points().unwrap()) {
            let jump = e.at_with(x, w.above).n_d - e.at_with(x, w.below).n_d;
            if rho == 0.0 {
                assert!(jump < 0.0, "an upward jump at rho = 0: {:?}", e.params());
                down_at_zero += 1;
            } else if jump > 0.0 {
                up += 1;
            }
        }
        match &s.result {
            Ok(Regime::Interior(eq)) if eq.tie.is_some() => {
                check_identities_1c(e, eq);
                check_fork_and_bounds_1c(e, eq);
                let tie = eq.tie.unwrap();
                assert!(tie.share > 0.0 && tie.share <= 1.0);
            }
            Err(SolveError::MultipleEquilibria {
                sign_changes,
                switches,
            }) => {
                assert!(rho > 0.0, "multiple equilibria at rho = 0");
                assert!(*sign_changes >= 3 && sign_changes % 2 == 1);
                let x = switches[0];
                let w = e.envelope().switches[0];
                assert!(e.at_with(x, w.above).n_d > e.at_with(x, w.below).n_d);
                assert_eq!(check_regions(e), *sign_changes);
            }
            _ => {}
        }
    }
    assert!(tally.ties > 0 && tally.multiple > 0, "{tally:?}");
    assert!(down_at_zero > 20 && up > 0, "{down_at_zero} {up}");
}

#[test]
fn the_draws_cover_the_regimes() {
    // Every regime, ties, MultipleEquilibria, unused task types, θ = 0 types and every build
    // lag in 1..5 occur; the tallies are the ones docs/unit-1c.md §12 records.
    let sets = all_sets();
    let tallies: Vec<&Tally> = sets.iter().map(|(_, t)| t).collect();
    for t in &tallies {
        eprintln!("{t:?}");
    }
    let sum = |f: fn(&Tally) -> usize| tallies.iter().map(|t| f(t)).sum::<usize>();
    assert!(sum(|t| t.boundary) > 0);
    assert!(sum(|t| t.not_viable) > 0);
    assert!(sum(|t| t.no_interior_at_zero) > 0);
    assert!(sum(|t| t.multiple) > 0);
    assert!(sum(|t| t.ties) > 0);
    assert!(sum(|t| t.invalid) > 0);
    let (mut unused, mut power, mut lags, mut switches, mut chains) = (0, 0, [false; 6], 0, 0);
    for (samples, _) in &sets {
        for s in samples {
            let Some(eq) = s.interior() else { continue };
            let p = s.economy.params();
            for (k, t) in p.machine_types.iter().enumerate() {
                if t.task_efficiency > 0.0 && eq.types[k].task_services == 0.0 {
                    unused += 1;
                }
                if t.task_efficiency == 0.0 && eq.types[k].services > 0.0 {
                    power += 1;
                }
                lags[t.build_lag as usize - 1] = true;
            }
            switches += eq.switches.len();
            if p.intermediate.iter().flatten().any(|&a| a > 0.0) {
                chains += 1;
            }
        }
    }
    assert!(
        unused > 0 && power > 0 && switches > 0 && chains > 60,
        "{unused} {power} {switches} {chains}"
    );
    assert_eq!(lags[..5], [true; 5]);
    // A non-switch set's samples are drawn by the general table.
    assert!(sets[0]
        .0
        .iter()
        .all(|s| s.set == Set::Zero && s.economy.params().rho == 0.0));
}
