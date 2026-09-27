//! Fixtures and checks shared by unit 1g's tests (docs/unit-1g.md §3.3 and §8): the chains S1,
//! S2 and S2h, A0, CHAIN's horse, and L2 with its plants, written as goldens/generate_1g.py
//! writes them.

use oracle::{
    Category, ChainCategory, ChainEconomy, ChainEq, Eq1c, GoodsChain, GoodsRecipe, Machine,
    MachineParams, MachineType, Material, Plant, PlantRecipe, PowerSchedule, Recipe, Regime,
    UniformWorkCost,
};
use rustyecon_core::{Clock, CompoundPerYear, Date, FractionPerYear, Years};

use crate::support::*;
use crate::support_1c::*;

/// The weekly clock of decision 121: 52 ticks a year.
pub fn weekly() -> Clock {
    Clock {
        start: Date::new(1750, 1, 1).expect("a date"),
        ticks_per_year: 52,
    }
}

/// δ a tick for an annual fraction, by `Clock::fraction`.
pub fn per_week(delta_year: f64) -> f64 {
    weekly().fraction(FractionPerYear(delta_year))
}

/// ρ a tick for an annual rate, by `Clock::compound`.
pub fn rho_per_week(rho_year: f64) -> f64 {
    weekly().compound(CompoundPerYear(rho_year))
}

/// A lag in years as whole ticks, by `Clock::ticks`.
pub fn ticks(years: f64) -> u32 {
    weekly().ticks(Years(years)).expect("a lag")
}

/// A recipe over goods by key.
pub fn goods(inputs: &[(&str, f64)], labor: f64, land: f64) -> GoodsRecipe {
    GoodsRecipe {
        inputs: inputs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        labor,
        land,
    }
}

/// A material.
pub fn material(key: &str, recipe: GoodsRecipe) -> Material {
    Material {
        key: key.to_string(),
        recipe,
    }
}

/// A machine: its good, build recipe, hours, κ, θ, operating recipe, δ and J.
#[allow(clippy::too_many_arguments)]
pub fn machine(
    key: &str,
    build: GoodsRecipe,
    hours: &str,
    hours_per_period: f64,
    task_efficiency: f64,
    operating: GoodsRecipe,
    delta: f64,
    build_lag: u32,
) -> Machine {
    Machine {
        key: key.to_string(),
        build,
        hours: hours.to_string(),
        hours_per_period,
        task_efficiency,
        operating,
        delta,
        build_lag,
    }
}

/// The good (μ 1 on one segment) and space (h 1), Appendix B's categories, keyed GOOD and SPACE.
pub fn good_space() -> Vec<ChainCategory> {
    vec![
        ChainCategory {
            key: "GOOD".into(),
            category: Category {
                weight: 1.0,
                direct_land: 0.0,
                density: vec![1.0],
            },
            inputs: vec![],
        },
        ChainCategory {
            key: "SPACE".into(),
            category: Category {
                weight: 1.0,
                direct_land: 1.0,
                density: vec![0.0],
            },
            inputs: vec![],
        },
    ]
}

/// A chain on one segment with the good and space: N, T, the schedule, χ_max and ρ.
pub fn chain_of(
    workers: f64,
    schedule: PowerSchedule,
    chi_max: f64,
    rho: f64,
    materials: Vec<Material>,
    machines: Vec<Machine>,
) -> GoodsChain {
    GoodsChain {
        workers,
        land: 10.0,
        schedule,
        work_cost: UniformWorkCost { chi_max },
        rho,
        edges: vec![0.0, 1.0],
        categories: good_space(),
        materials,
        machines,
    }
}

/// S1's four solved coefficients (docs/unit-1g.md §3.3), as the doubles nearest the fractions:
/// engine-hour labour, engine-good labour, coal's seams, the engine's site.
pub const S1_SOLVED: [(f64, f64); 4] = [
    (855257.0, 7735400.0),
    (14560827.0, 37903460.0),
    (14836821.0, 15470800.0),
    (186348663.0, 3032276800.0),
];

/// S1's materials and engine (θ as given).
pub fn s1_goods(theta: f64) -> (Vec<Material>, Machine) {
    let [lab_op, lab_i, seams, site] = S1_SOLVED.map(|(n, d)| n / d);
    let materials = vec![
        material("COAL", goods(&[("ENGINE_HOURS", 0.1)], 0.2, seams)),
        material("IRON", goods(&[("COAL", 0.5)], 0.5, 0.1)),
    ];
    let engine = machine(
        "ENGINE",
        goods(&[("IRON", 0.1)], lab_i, site),
        "ENGINE_HOURS",
        1.0,
        theta,
        goods(&[("COAL", 0.4)], lab_op, 0.0),
        0.1,
        3,
    );
    (materials, engine)
}

/// S1, the steam chain (docs/unit-1g.md §3.3), at ρ: M3's household on γ = 1 + 2x.
pub fn s1(rho: f64) -> GoodsChain {
    let (materials, engine) = s1_goods(1.0);
    chain_of(
        4.0,
        linear(1.0, 1.0, 2.0),
        1.0,
        rho,
        materials,
        vec![engine],
    )
}

/// S2 (the engine at the margin: θ 1.3, γ = 0.5 + 1.5x, N 10) or S2H (the horse at the margin:
/// θ 1.2, γ = 0.5 + x, N 16), at ρ.
pub fn s2(horse_at_the_margin: bool, rho: f64) -> GoodsChain {
    let (theta, g1, workers) = if horse_at_the_margin {
        (1.2, 1.0, 16.0)
    } else {
        (1.3, 1.5, 10.0)
    };
    let (mut materials, engine) = s1_goods(theta);
    materials.push(material("FODDER", goods(&[], 0.3, 0.5)));
    let horse = machine(
        "HORSE",
        goods(&[("FODDER", 2.0)], 1.5, 0.0),
        "HORSE_HOURS",
        1.0,
        1.0,
        goods(&[("FODDER", 0.1)], 0.15, 0.0),
        0.08,
        4,
    );
    chain_of(
        workers,
        linear(1.0, 0.5, g1),
        1.0,
        rho,
        materials,
        vec![horse, engine],
    )
}

/// A0's ω: the share of the flow machine's land its horse runs on, as fodder.
pub const A0_OMEGA: f64 = 0.5;

/// A0 (docs/unit-1g.md §3.3): Appendix B's flow machine (0.3, 0.05, 0.4) as a durable good at
/// 52 ticks a year, δ 10% a year, J 1, ω 1/2, on Appendix B's household, at ρ a tick.
pub fn a0(rho: f64) -> GoodsChain {
    let d = per_week(0.1);
    let (a, lam, b) = (0.3, 0.05, 0.4);
    chain_of(
        4.0,
        linear(1.0, 0.2, 0.8),
        1.0,
        rho,
        vec![material("FODDER", goods(&[], 0.0, A0_OMEGA * b))],
        vec![machine(
            "HORSE",
            goods(&[("HORSE_DAYS", a / d)], lam / d, (1.0 - A0_OMEGA) * b / d),
            "HORSE_DAYS",
            1.0,
            1.0,
            goods(&[("FODDER", 1.0)], 0.0, 0.0),
            d,
            1,
        )],
    )
}

/// A0's machine as one unit-1c type: operating (0; 0; ω·b), build (a/δ; λ/δ; (1 − ω)·b/δ).
pub fn a0_type() -> MachineType {
    let d = per_week(0.1);
    let (a, lam, b) = (0.3, 0.05, 0.4);
    machine_type(
        1.0,
        recipe(&[0.0], 0.0, A0_OMEGA * b),
        recipe(&[a / d], lam / d, (1.0 - A0_OMEGA) * b / d),
        d,
        1,
    )
}

/// A0 as one type on Appendix B's household, at ρ a tick.
pub fn a0_one_type(rho: f64) -> MachineParams {
    appendix_b_household(linear(1.0, 0.2, 0.8), rho, 1.0, vec![a0_type()])
}

/// CHAIN's horse at weekly periods (docs/unit-1g.md §3.3), on its constructed county: N 12,
/// T 10, h 1, χ_max 1/20, γ = 0.2 + 0.8x, ρ 0.
pub fn horse() -> GoodsChain {
    chain_of(
        12.0,
        linear(1.0, 0.2, 0.8),
        0.05,
        0.0,
        vec![material("FODDER", goods(&[("HORSE_DAYS", 2.0)], 8.0, 1.0))],
        vec![machine(
            "HORSE",
            goods(&[("FODDER", 8.0)], 20.0, 3.0),
            "HORSE_DAYS",
            250.0 / 52.0,
            1.0,
            goods(&[("FODDER", 0.0176)], 0.1, 0.0),
            per_week(0.08),
            ticks(3.0),
        )],
    )
}

/// The markets probe's L2: Appendix B's household with M4's engine and power as flow types, each
/// buying the other's service.
pub fn l2(rho: f64) -> MachineParams {
    appendix_b_household(
        linear(1.0, 0.2, 0.8),
        rho,
        1.0,
        vec![
            machine_type(2.0, recipe(&[0.1, 0.5], 0.12, 0.4), Recipe::zero(2), 1.0, 1),
            machine_type(0.0, recipe(&[0.1, 0.0], 0.12, 0.6), Recipe::zero(2), 1.0, 1),
        ],
    )
}

/// The plants' θ: CAPACITY.md's 0.8.
pub const PLANT_THETA: f64 = 0.8;

/// A plant at θ 0.8, δ 10% a year a week, J 1, of the recipe given.
pub fn plant(recipe: PlantRecipe) -> Plant {
    Plant {
        bundle_share: PLANT_THETA,
        delta: per_week(0.1),
        build_lag: 1,
        recipe,
    }
}

/// A bundle plant of size s (s1 when `None`).
pub fn bundle_plant(size: Option<f64>) -> Plant {
    let s = size.unwrap_or_else(|| oracle::s1_size(PLANT_THETA, per_week(0.1)));
    plant(PlantRecipe::Bundle { size: s })
}

/// P2's plant: labour 0.5 and land 0.5 a unit.
pub fn labour_land_plant() -> Plant {
    plant(PlantRecipe::Fixed(recipe(&[0.0, 0.0], 0.5, 0.5)))
}

/// A chain's economy, known valid.
pub fn chain_economy(chain: GoodsChain) -> ChainEconomy {
    ChainEconomy::new(chain.clone()).unwrap_or_else(|e| panic!("{chain:?}: {e}"))
}

/// A chain's interior equilibrium, with unit 1c's identities checked on it.
pub fn solved_chain(chain: GoodsChain) -> (ChainEconomy, Box<ChainEq>) {
    let e = chain_economy(chain);
    match e.solve() {
        Ok(Regime::Interior(eq)) => {
            check_identities_1c(e.economy(), &eq.eq);
            (e, eq)
        }
        other => panic!("not interior: {other:?}"),
    }
}

/// The aggregates of a unit-1c equilibrium against goldens (x*, 1 − x*, v, P_s, Y, N_a, income).
pub fn aggregates(what: &str, eq: &Eq1c, want: [f64; 7]) {
    let got = [
        eq.x_star,
        eq.one_minus_x_star,
        eq.v,
        eq.p_s,
        eq.y,
        eq.n_a,
        eq.income,
    ];
    let names = ["x*", "1 - x*", "v", "P_s", "Y", "N_a", "income"];
    for ((name, g), w) in names.iter().zip(got).zip(want) {
        close(&format!("{what}: {name}"), g, w);
    }
}

/// A good's price and output against goldens (an output of 0 exactly when the golden is 0).
pub fn good_is(what: &str, eq: &ChainEq, key: &str, price: f64, output: f64) {
    let g = eq
        .good(key)
        .unwrap_or_else(|| panic!("{what}: no good {key}"));
    close(&format!("{what}: {key}'s price"), g.price, price);
    if output == 0.0 {
        assert_eq!(g.output, 0.0, "{what}: {key}'s output");
    } else {
        close(&format!("{what}: {key}'s output"), g.output, output);
    }
}

/// The largest modulus of an eigenvalue of a nonnegative n×n matrix (row-major), by power
/// iteration on I + A, whose Perron root is 1 + ρ(A) and which is primitive when A is
/// irreducible.
pub fn spectral_radius(n: usize, a: &[f64]) -> f64 {
    let mut x = vec![1.0; n];
    let mut rate = 0.0;
    for _ in 0..20000 {
        let mut y = x.clone();
        for i in 0..n {
            for j in 0..n {
                y[i] += a[i * n + j] * x[j];
            }
        }
        let norm = y.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        rate = norm / x.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        x = y.iter().map(|v| v / norm).collect();
    }
    rate - 1.0
}

/// A^op + A^I and A^op + Δ·A^I of a unit-1c economy's types, row-major.
pub fn physical_and_per_period(types: &[MachineType]) -> (Vec<f64>, Vec<f64>) {
    let n = types.len();
    let mut physical = vec![0.0; n * n];
    let mut per_period = vec![0.0; n * n];
    for k in 0..n {
        for l in 0..n {
            let (op, build) = (types[k].operating.machines[l], types[k].build.machines[l]);
            physical[k * n + l] = op + build;
            per_period[k * n + l] = op + types[k].delta * build;
        }
    }
    (physical, per_period)
}

/// Unit 1c's rule before D-G10, reimplemented: I − (A^op + A^I) a nonsingular M-matrix (every
/// pivot of elimination without pivoting positive) and (I − (A^op + A^I))⁻¹(b^op + b^I) > 0.
pub fn accepted_by_1c(types: &[MachineType]) -> bool {
    let n = types.len();
    let (physical, _) = physical_and_per_period(types);
    let mut g: Vec<f64> = (0..n * n)
        .map(|i| {
            let (k, l) = (i / n, i % n);
            if k == l {
                1.0 - physical[i]
            } else {
                -physical[i]
            }
        })
        .collect();
    let mut r: Vec<f64> = types
        .iter()
        .map(|t| t.operating.land + t.build.land)
        .collect();
    for k in 0..n {
        let pivot = g[k * n + k];
        if pivot.is_nan() || pivot <= 0.0 {
            return false;
        }
        for i in k + 1..n {
            let m = g[i * n + k] / pivot;
            for j in k + 1..n {
                g[i * n + j] -= m * g[k * n + j];
            }
            r[i] -= m * r[k];
        }
    }
    let mut z = vec![0.0; n];
    for i in (0..n).rev() {
        let mut s = r[i];
        for j in i + 1..n {
            s -= g[i * n + j] * z[j];
        }
        z[i] = s / g[i * n + i];
    }
    z.iter().all(|&v| v > 0.0)
}
