//! f9: random economies with a government and a basket (docs/unit-1f.md §8), drawn with
//! SplitMix64 from seeds 951 to 955 on unit 1e's e8 and unit 1d's d7 tables (1e's were 941-948):
//! every identity of §4.8-4.9, the residuals, the consumption tax as a wage tax, a replacing
//! transfer's neutrality, the count against a scan 16 times finer, and the regimes.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use oracle::{
    Basket, Budget, Eq1f, ExitForm, Government, HouseholdEconomy, HouseholdParams, LandMarket,
    ParcelEconomy, ParcelParams, Program, Regime, SolveError, TransferMode, EXIT_SCAN,
};

use crate::e8_random_parcels::{draw as draw_1e, Set as Set1e};
use crate::g5_random_economies::SplitMix64;
use crate::support_1e::{bits_1e, priced};
use crate::support_1f::*;

/// The sets of docs/unit-1f.md §8 that are drawn; (e) and (f) re-solve (a), (d) and (b).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Set {
    /// (a) RentRate on 1e's draws: τ_w, t_c ~ U(0, 0.5), d̂ ~ U(0, 1), μ_w and μ_e each U(0, 0.3)
    /// with probability 1/2, the mode at random.
    Rent,
    /// (b) Dividend on the eligible draws (no plot taker): τ_R ~ U(0, 1), τ_w and t_c as (a), a
    /// uniform program with probability 1/2, the mode at random.
    Dividend,
    /// (c) CES on the eligible draws (the dependence form): σ log-uniform on [0.1, 10], 1 with
    /// probability 1/4, with and without interest, RentRate with probability 1/2.
    Ces,
    /// (d) 1d's d7 draws with reserved hours under RentRate, μ_w ≤ μ_e.
    Reserved,
    /// (g) in-work benefits μ_w ~ U(0, 1) > μ_e on (a)'s economies.
    InWork,
}

const SETS: [Set; 5] = [
    Set::Rent,
    Set::Dividend,
    Set::Ces,
    Set::Reserved,
    Set::InWork,
];

/// The first seed; the sets take 951 to 955.
const SEED_1F: u64 = 951;

/// Equilibria wanted per set.
const WANTED_1F: usize = 60;

/// The cap on draws per set.
const MAX_DRAWS_1F: usize = 2_000;

fn chance(rng: &mut SplitMix64, p: f64) -> bool {
    rng.uniform(0.0, 1.0) < p
}

fn mode(rng: &mut SplitMix64) -> TransferMode {
    if chance(rng, 0.5) {
        TransferMode::Replace
    } else {
        TransferMode::Supplement
    }
}

/// A 1e draw of sets (a), (b), (c) or (g), cycling 1e's tables.
fn economy(rng: &mut SplitMix64, k: usize) -> ParcelParams {
    let set = [Set1e::One, Set1e::Types, Set1e::Interest, Set1e::Goods][k % 4];
    draw_1e(rng, set)
}

/// One draw of `set`, or `None` when the set has no more (d).
fn draw(
    rng: &mut SplitMix64,
    set: Set,
    k: usize,
    reserved: &mut dyn Iterator<Item = ParcelParams>,
) -> Option<HouseholdParams> {
    let params = match set {
        Set::Rent => {
            let economy = economy(rng, k);
            let g = government(|g| {
                g.payroll = rng.uniform(0.0, 0.5);
                g.consumption = rng.uniform(0.0, 0.5);
                g.budget = Budget::RentRate {
                    dividend: rng.uniform(0.0, 1.0),
                };
                if chance(rng, 0.5) {
                    g.program.work = rng.uniform(0.0, 0.3);
                }
                if chance(rng, 0.5) {
                    g.program.exit = rng.uniform(0.0, 0.3);
                }
                g.mode = mode(rng);
            });
            household(economy, g)
        }
        Set::Dividend => {
            let mut economy = economy(rng, k % 3);
            for e in &mut economy.exits {
                *e = if chance(rng, 2.0 / 3.0) {
                    ExitForm::Dependence
                } else {
                    priced(0.0, rng.uniform(0.0, 1.0), 0.1)
                };
            }
            let g = government(|g| {
                g.payroll = rng.uniform(0.0, 0.5);
                g.consumption = rng.uniform(0.0, 0.5);
                g.budget = Budget::Dividend {
                    rent_tax: rng.uniform(0.0, 1.0),
                };
                if chance(rng, 0.5) {
                    let mu = rng.uniform(0.0, 0.3);
                    g.program = Program { work: mu, exit: mu };
                }
                g.mode = mode(rng);
            });
            household(economy, g)
        }
        Set::Ces => {
            let mut economy = economy(rng, 1 + k % 2);
            economy.exits = vec![ExitForm::Dependence; economy.worker_types.len()];
            let sigma = if chance(rng, 0.25) {
                1.0
            } else {
                let span = rustyecon_core::num::ln(10.0);
                rustyecon_core::num::exp(rng.uniform(-span, span))
            };
            let g = if chance(rng, 0.5) {
                Government::none()
            } else {
                government(|g| {
                    g.payroll = rng.uniform(0.0, 0.5);
                    g.consumption = rng.uniform(0.0, 0.5);
                    g.budget = Budget::RentRate {
                        dividend: rng.uniform(0.0, 1.0),
                    };
                })
            };
            HouseholdParams {
                economy,
                basket: Basket::Ces { sigma },
                government: g,
            }
        }
        Set::Reserved => {
            let economy = reserved.next()?;
            let g = government(|g| {
                g.payroll = rng.uniform(0.0, 0.5);
                g.consumption = rng.uniform(0.0, 0.5);
                g.budget = Budget::RentRate {
                    dividend: rng.uniform(0.0, 1.0),
                };
                g.program.exit = rng.uniform(0.0, 0.3);
                if chance(rng, 0.5) {
                    g.program.work = rng.uniform(0.0, g.program.exit);
                }
                g.mode = mode(rng);
            });
            household(economy, g)
        }
        Set::InWork => {
            let economy = economy(rng, k);
            let g = government(|g| {
                g.program.work = rng.uniform(0.0, 1.0);
                g.program.exit = rng.uniform(0.0, g.program.work);
                if chance(rng, 0.5) {
                    g.payroll = rng.uniform(0.0, 0.5);
                }
                g.mode = mode(rng);
            });
            household(economy, g)
        }
    };
    Some(params)
}

/// One draw and what 1f gave.
struct Sample {
    set: Set,
    economy: HouseholdEconomy,
    result: Result<Regime<Eq1f>, SolveError>,
}

impl Sample {
    fn equilibrium(&self) -> Option<&Eq1f> {
        match &self.result {
            Ok(Regime::Interior(eq)) => Some(eq),
            _ => None,
        }
    }
}

type Tally = BTreeMap<String, usize>;

fn label(economy: &HouseholdEconomy, result: &Result<Regime<Eq1f>, SolveError>) -> Vec<String> {
    let g = economy.params().government;
    let mut out = vec![
        match g.budget {
            Budget::RentRate { .. } => "RentRate".to_string(),
            Budget::Dividend { .. } => "Dividend".to_string(),
        },
        format!("{:?}", g.mode),
    ];
    match result {
        Ok(Regime::Interior(eq)) => {
            out.push(format!("{:?}", eq.base.base.margin));
            out.push(format!("{:?}", eq.base.land_market));
            if eq.base.base.tie.is_some() || eq.base.enclosure.is_some() {
                out.push("tie".to_string());
            }
            if let Some(w) = eq.government.within_rent {
                out.push(format!("within_rent {w}"));
            }
            if eq.base.land_market == LandMarket::Scarce {
                out.push(format!("kappa >= 1 {}", eq.base.coverage >= 1.0));
            }
        }
        Ok(Regime::NotViable { .. }) => out.push("NotViable".to_string()),
        Ok(other) => out.push(other.name().to_string()),
        Err(SolveError::NoMarket { .. }) => out.push("NoMarket".to_string()),
        Err(SolveError::MultipleEquilibria { .. }) => out.push("MultipleEquilibria".to_string()),
        Err(SolveError::SurplusLabour { .. }) => out.push("SurplusLabour".to_string()),
        Err(e) => out.push(format!("error {e}")),
    }
    out
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
        })
        .map(|s| crate::support_1e::e0(s.economy.params().clone()));
    while found < WANTED_1F {
        assert!(draws < MAX_DRAWS_1F, "{set:?}: {tally:?}");
        draws += 1;
        let Some(params) = draw(&mut rng, set, draws, &mut reserved) else {
            break;
        };
        let Ok(economy) = HouseholdEconomy::new(params) else {
            *tally.entry("invalid".to_string()).or_default() += 1;
            continue;
        };
        let result = economy.solve();
        if let Err(e) = &result {
            assert!(
                matches!(
                    e,
                    SolveError::NoMarket { .. }
                        | SolveError::MultipleEquilibria { .. }
                        | SolveError::SurplusLabour { .. }
                ),
                "{set:?} draw {draws}: {e}"
            );
        }
        found += usize::from(matches!(result, Ok(Regime::Interior(_))));
        for l in label(&economy, &result) {
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

fn all_sets() -> &'static [(Vec<Sample>, Tally)] {
    static SAMPLES: OnceLock<Vec<(Vec<Sample>, Tally)>> = OnceLock::new();
    SAMPLES.get_or_init(|| {
        SETS.into_iter()
            .zip(SEED_1F..)
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
            check_identities_1f(&s.economy, eq);
            n += 1;
        }
    }
    assert!(n >= 4 * WANTED_1F, "{n}");
}

#[test]
fn residuals_recompute() {
    // Each new residual is recomputed bit for bit by check_identities_1f; each is nonzero
    // somewhere, so none is a constant.
    let mut any = [false; 5];
    for s in samples() {
        if let Some(eq) = s.equilibrium() {
            let r = &eq.residuals;
            for (k, v) in [r.budget, r.spending, r.composites, r.rent_share, r.euler]
                .into_iter()
                .enumerate()
            {
                any[k] |= v > 0.0;
            }
        }
    }
    assert_eq!(any, [true; 5]);
}

#[test]
fn paired_equivalence() {
    // (e): every (a) and (d) equilibrium re-solved at (1 − κ_w, 0), the rest of its government
    // the same: the allocation within 1e-12 (§4.10 (f)).
    let mut n = 0;
    for s in samples() {
        if !matches!(s.set, Set::Rent | Set::Reserved) {
            continue;
        }
        let Some(eq) = s.equilibrium() else { continue };
        let mut params = s.economy.params().clone();
        let g = params.government;
        params.government.payroll = 1.0 - (1.0 - g.payroll) / (1.0 + g.consumption);
        params.government.consumption = 0.0;
        let other = match economy_1f(params).solve() {
            Ok(Regime::Interior(eq)) => eq,
            other => panic!("{:?}: {other:?}", s.set),
        };
        same_allocation(&format!("{:?}", s.set), eq, &other, 1e-12);
        n += 1;
    }
    assert!(n >= 2 * WANTED_1F, "{n}");
}

#[test]
fn replace_neutrality() {
    // (f): (b)'s draws in Replace mode with the rent tax alone, scaled so that d = τ_R·r·T/N stays
    // below every ν_i·P^c along the path (P is least at the start, v = 0): bit for bit the draw
    // without a government, but the scan's count at ρ > 0.
    let mut n = 0;
    for s in samples() {
        if s.set != Set::Dividend {
            continue;
        }
        let economy = &s.economy.params().economy;
        let parcels = ParcelEconomy::new(economy.clone()).unwrap();
        let first = parcels.workers().machines().envelope().first;
        let p_min = parcels.at_wage(0.0, 0.0, first).point.p_s;
        let nu = economy
            .worker_types
            .iter()
            .map(|t| t.support)
            .fold(f64::INFINITY, f64::min);
        let people: f64 = economy.worker_types.iter().map(|t| t.workers).sum();
        let t = parcels.enclosed_land();
        let tau = (0.5 * nu * p_min * people / t).min(1.0);
        let replace = economy_1f(household(economy.clone(), replacing(dividend(tau))));
        let none = economy_1f(HouseholdParams::from_parcels(economy.clone()));
        match (none.solve(), replace.solve()) {
            (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) => {
                let (mut x, mut y) = (bits_1e(&a.base), bits_1e(&b.base));
                x.remove("scan_points");
                y.remove("scan_points");
                assert_eq!(x, y, "τ_R {tau}");
                assert!(b.government.dividend <= nu * b.basket.consumer_price);
                n += 1;
            }
            (a, b) => assert_eq!(format!("{a:?}"), format!("{b:?}")),
        }
    }
    assert!(n >= WANTED_1F, "{n}");
}

#[test]
fn count_against_a_fine_scan() {
    // §5.3 step 4: where the count scans (a plot taker; a CES basket or a moving Dividend transfer
    // at ρ > 0), a scan 16 times finer gives the same result, the equilibrium bit for bit.
    let mut compared = 0;
    for (samples, _) in all_sets() {
        let mut taken = 0;
        for s in samples {
            let scanned = s.equilibrium().is_some_and(|eq| eq.base.scan_points > 0);
            if !scanned || taken == 30 {
                continue;
            }
            taken += 1;
            let fine = s.economy.solve_scanned(16 * EXIT_SCAN);
            match (&s.result, &fine) {
                (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) => {
                    let (mut x, mut y) = (bits_1f(a), bits_1f(b));
                    x.remove("scan_points");
                    y.remove("scan_points");
                    assert_eq!(x, y, "{:?}", s.set);
                }
                (a, b) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{:?}", s.set),
            }
            compared += 1;
        }
    }
    assert!(compared >= 60, "{compared}");
}

#[test]
fn the_draws_cover_the_regimes() {
    // The line, the wall, the all-human corner, idle land, a tie, SurplusLabour, both closures
    // and modes, within_rent both ways and κ on both sides of 1 (decision 176 records the
    // tallies).
    let mut all = Tally::new();
    for (set, (_, tally)) in SETS.iter().zip(all_sets()) {
        println!("f9 {set:?}: {tally:?}");
        for (k, v) in tally {
            *all.entry(k.clone()).or_default() += v;
        }
    }
    // The tallies of docs/unit-1f.md §14 (decision 176), exactly: the draws are fixed by their
    // seeds.
    let want: [(&str, usize); 18] = [
        ("AllHuman", 3),
        ("Contestable", 142),
        ("Dividend", 62),
        ("Idle", 51),
        ("NotViable", 7),
        ("RentRate", 248),
        ("Replace", 120),
        ("Scarce", 249),
        ("Supplement", 190),
        ("SurplusLabour", 3),
        ("Wall", 155),
        ("draws", 341),
        ("invalid", 31),
        ("kappa >= 1 false", 204),
        ("kappa >= 1 true", 45),
        ("tie", 2),
        ("within_rent false", 138),
        ("within_rent true", 102),
    ];
    for (key, n) in want {
        assert_eq!(all.get(key).copied(), Some(n), "{key}: {all:?}");
    }
    for key in [
        "Contestable",
        "Wall",
        "AllHuman",
        "Idle",
        "tie",
        "SurplusLabour",
        "RentRate",
        "Dividend",
        "Replace",
        "Supplement",
        "within_rent true",
        "within_rent false",
        "kappa >= 1 true",
        "kappa >= 1 false",
    ] {
        assert!(all.contains_key(key), "no {key}: {all:?}");
    }
}
