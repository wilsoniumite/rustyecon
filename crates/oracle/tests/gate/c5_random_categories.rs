//! C5: random multi-category economies (docs/unit-1b.md §8): the fork identity and the
//! category bounds on random instances, and the income identity to 1e-12 (PLAN Phase 1's
//! gate), with every identity of §4, the residuals, the root and single crossing.

use oracle::{Category, CategoryEconomy, CategoryParams, Eq1b, PowerSchedule, Regime};

use crate::g5_random_economies::{SplitMix64, MAX_DRAWS, MAX_LAG, SEED, WANTED};
use crate::support_1b::*;

#[derive(Clone, Copy, Debug)]
enum Set {
    /// (ρ, δ, J_b) = (0, 1, 1).
    Flow,
    /// ρ ~ U(0, 0.12), δ ~ U(0.03, 1), J_b = 1.
    Durable,
    /// As `Durable`, with J_b drawn from 1..=5.
    BuildLag,
}

/// The first seed of unit 1b's sets: 1a's G5 seed, check_macro.py's 923 (docs/unit-1b.md
/// §8). The draws are 1b's own, so they share nothing with 1a's but the generator.
const SEED_1B: u64 = SEED;

/// True with probability `p`.
fn chance(rng: &mut SplitMix64, p: f64) -> bool {
    rng.uniform(0.0, 1.0) < p
}

/// One draw from docs/unit-1b.md §8's table. Fields are drawn in the order written.
fn draw(rng: &mut SplitMix64, set: Set) -> CategoryParams {
    // 1a's G5 ranges for the scalars, without h.
    let workers = rng.uniform(1.0, 5.0);
    let land = rng.uniform(2.0, 20.0);
    let a = rng.uniform(0.05, 0.5);
    let b = rng.uniform(0.1, 1.0);
    let lam = rng.uniform(0.0, 0.1);
    let schedule = PowerSchedule {
        eta: 1.0,
        g0: rng.uniform(0.01, 0.5),
        g1: rng.uniform(0.2, 1.5),
        k: rng.uniform(0.5, 6.0),
    };
    let chi_max = rng.uniform(0.5, 3.0);
    let (rho, delta, build_lag) = match set {
        Set::Flow => (0.0, 1.0, 1),
        Set::Durable => (rng.uniform(0.0, 0.12), rng.uniform(0.03, 1.0), 1),
        Set::BuildLag => (
            rng.uniform(0.0, 0.12),
            rng.uniform(0.03, 1.0),
            1 + (rng.next_u64() % MAX_LAG) as u32,
        ),
    };
    // The task line: S in 1..=4 segments with widths proportional to U(0.5, 1.5).
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
    // C in 2..=5 categories, and a site with probability 0.5.
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
        categories.push(Category {
            weight: rng.uniform(0.2, 1.5),
            direct_land: 1.0,
            density: vec![0.0; segments],
        });
    }
    CategoryParams {
        workers,
        land,
        a,
        lam,
        b,
        schedule,
        work_cost: oracle::UniformWorkCost { chi_max },
        rho,
        delta,
        build_lag,
        edges,
        categories,
    }
}

struct Sample {
    economy: CategoryEconomy,
    eq: Box<Eq1b>,
}

/// What a set's draws gave besides its interior economies.
#[derive(Debug, Default)]
struct Tally {
    invalid: usize,
    boundary: usize,
    not_viable: usize,
    no_interior_at_zero: usize,
}

/// The first WANTED interior economies of a set, and the tally of the other draws.
fn sample(set: Set, seed: u64) -> (Vec<Sample>, Tally) {
    let mut rng = SplitMix64(seed);
    let mut out = Vec::with_capacity(WANTED);
    let mut tally = Tally::default();
    let mut draws = 0;
    while out.len() < WANTED {
        assert!(draws < MAX_DRAWS, "{set:?}: only {} interior", out.len());
        draws += 1;
        let params = draw(&mut rng, set);
        let Ok(economy) = CategoryEconomy::new(params.clone()) else {
            tally.invalid += 1;
            continue;
        };
        match economy.solve() {
            Ok(Regime::Interior(eq)) => out.push(Sample { economy, eq }),
            Ok(Regime::BoundaryNoMargin { .. }) => tally.boundary += 1,
            Ok(Regime::NotViable { .. }) => tally.not_viable += 1,
            Ok(Regime::NoInteriorAtZero { .. }) => tally.no_interior_at_zero += 1,
            Err(e) => panic!("{params:?}: {e}"),
        }
    }
    (out, tally)
}

fn all_sets() -> Vec<Sample> {
    [
        (Set::Flow, SEED_1B),
        (Set::Durable, SEED_1B + 1),
        (Set::BuildLag, SEED_1B + 2),
    ]
    .into_iter()
    .flat_map(|(set, seed)| sample(set, seed).0)
    .collect()
}

#[test]
fn identities() {
    // Income as Y P_s and as sum p_j y_j; land, services and the user cost; eq 11; P_s in
    // both forms; the basket count; interest = rho W_K (SSRN App. C, eq 11, §5).
    for s in all_sets() {
        check_identities_1b(&s.economy, &s.eq);
    }
}

#[test]
fn fork_identity_and_category_bounds() {
    // For every category, p_j = v L_j* + b_j and p_j = v lt_j + bt_j (main.tex:459; SSRN eq
    // 12); the chain b_j <= bt_j^q <= bt_j <= p_j <= v Lbar_j + b_j, L_j* <= Lbar_j, the
    // pair, and v/p_j >= 1/Lbar_j when b_j = 0; the basket's v/P_s <= v/B_s (SSRN eq 13,
    // 19; main.tex:465-474).
    let samples = all_sets();
    for s in &samples {
        check_fork_and_bounds(&s.economy, &s.eq);
    }
    // The draws include categories whose every task is machine work. (A category whose
    // every task is hand work, on its upper bound, is C3's care.)
    let categories = || samples.iter().flat_map(|s| s.eq.categories.iter());
    assert!(categories().any(|c| c.machine > 0.0 && c.human == 0.0));
}

#[test]
fn residuals_recompute() {
    // Each residual equals its recomputation bit for bit (check_identities_1b), and each is
    // nonzero in some economy, so the comparison has teeth.
    let samples = all_sets();
    let nonzero = |pick: fn(&oracle::Residuals1b) -> f64| {
        samples
            .iter()
            .filter(|s| pick(&s.eq.residuals) > 0.0)
            .count()
    };
    for (name, count) in [
        ("labour", nonzero(|r| r.labor)),
        ("income", nonzero(|r| r.income)),
        ("land", nonzero(|r| r.land)),
        ("services", nonzero(|r| r.services)),
        ("user cost", nonzero(|r| r.user_cost)),
        ("fork", nonzero(|r| r.fork)),
        ("totals", nonzero(|r| r.totals)),
        ("expenditure", nonzero(|r| r.expenditure)),
    ] {
        assert!(
            count > 0,
            "the {name} residual is 0 in all {} economies",
            samples.len()
        );
    }
    for s in &samples {
        assert_eq!(recomputed_1b(&s.economy, &s.eq), s.eq.residuals);
    }
}

#[test]
fn root_and_grid() {
    // x* is the double where f changes sign, and n_D is nonincreasing, v/P_s increasing and
    // f single-crossing on the grid (docs/unit-1b.md §5.3).
    for s in all_sets() {
        assert_root_1b(&s.economy, s.eq.x_star);
        check_single_crossing_1b(&s.economy);
    }
}

#[test]
fn margin_flag() {
    // margin_active is whether a basket category has tasks on the segment holding x*,
    // recomputed from the parameters; both values occur.
    let samples = all_sets();
    for s in &samples {
        assert_eq!(
            s.eq.margin_active,
            margin_is_active(s.economy.params(), s.eq.x_star),
            "{:?}",
            s.economy.params()
        );
    }
    assert!(samples.iter().any(|s| s.eq.margin_active));
}

#[test]
fn unused_categories_are_priced() {
    // A category with z_j = 0 is not bought (y_j = 0) but has its cost price, and the bounds
    // hold for it: they are cost identities whether or not it is produced (docs/unit-1b.md
    // §4.2). check_fork_and_bounds covers every category; here the draws must contain some.
    let samples = all_sets();
    let mut unused = 0;
    for s in &samples {
        for (c, cat) in s.eq.categories.iter().zip(&s.economy.params().categories) {
            if cat.weight == 0.0 {
                unused += 1;
                assert_eq!((c.output, c.share, c.final_hours), (0.0, 0.0, 0.0));
                assert!(c.price > 0.0 && c.price.is_finite());
            }
        }
    }
    assert!(unused > 0, "no draw has an unused category");
}

#[test]
fn the_draws_cover_the_regimes() {
    // The sets are drawn to be mostly interior, with some draws failing validation (a basket
    // with no direct land) and some at the boundary; the tally is recorded, not gated,
    // beyond each kind occurring in the three sets together.
    let mut total = Tally::default();
    let mut lags = [false; MAX_LAG as usize];
    for (set, seed) in [
        (Set::Flow, SEED_1B),
        (Set::Durable, SEED_1B + 1),
        (Set::BuildLag, SEED_1B + 2),
    ] {
        let (samples, tally) = sample(set, seed);
        total.invalid += tally.invalid;
        total.boundary += tally.boundary;
        total.not_viable += tally.not_viable;
        total.no_interior_at_zero += tally.no_interior_at_zero;
        for s in &samples {
            lags[s.economy.params().build_lag as usize - 1] = true;
        }
    }
    assert!(total.invalid > 0 && total.boundary > 0, "{total:?}");
    assert!(
        lags.iter().all(|&seen| seen),
        "a build lag in 1..=5 was never drawn"
    );
}
