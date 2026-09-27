//! Unit 1g: plants as machine types, the capacity damper's long run (docs/unit-1g.md §2.6,
//! §3.1-3.2, §4.5 and §5.3; D:/rustyecon-loops/capacity/CAPACITY.md).
//!
//! A plant desk makes y = K^(1−θ)·z^θ from its recipe's bundle z and a plant K, built from a
//! recipe R and worn at δ. At its long run the plant has the size that minimises the cost of its
//! output, and the desk's price row is unit 1c's p = O + u·V with, per unit of capacity, the
//! operating recipe ζ times the bundle and the build recipe κ times R, where ζ^θ·κ^(1−θ) = 1 and
//! κ/ζ = (1 − θ)·c_f/(θ·u·P_K). So a plant rewrites a flow type ([`PlantEconomy::long_run`]).
//! With R a size s of the desk's own bundle the ratio is free of prices, and at ρ = 0 and
//! s = s1 ([`s1_size`]) the long run is the flow economy exactly; any other R makes the long run
//! a fixed point in the ratios ([`PlantEconomy::solve`]).

use std::fmt;

use rustyecon_core::num;

use crate::machine_block::{MachineType, Recipe};
use crate::machines::{Eq1c, MachineEconomy, MachineParams};
use crate::params::{self, user_cost, ParamError};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{Regime, SolveError};

/// The fixed point's tolerance: the largest |ln r′ − ln r| over the plants at which the ratios
/// count as fixed (docs/unit-1g.md §2.6).
pub const PLANT_TOL: f64 = 1e-13;

/// The most steps the fixed point takes before it refuses the plants.
pub const MAX_PLANT_STEPS: u32 = 200;

/// What a plant is built from, per plant unit.
#[derive(Clone, Debug, PartialEq)]
pub enum PlantRecipe {
    /// `size` bundles of its type's own recipe (CAPACITY.md's default): its ratio κ/ζ is free of
    /// prices. Scale.
    Bundle {
        /// s, bundles per plant unit.
        size: f64,
    },
    /// Any fixed recipe over machine services, labour and land: its ratio moves with prices.
    Fixed(Recipe),
}

/// A plant on a flow type (docs/unit-1g.md §3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct Plant {
    /// θ, the bundle's exponent in y = K^(1−θ)·z^θ, its share of the price at the long run. In
    /// [`SCALE_FLOOR`](crate::SCALE_FLOOR), 1]; 1 is no plant.
    pub bundle_share: f64,
    /// δ, the plant's wear per period. In [`SCALE_FLOOR`](crate::SCALE_FLOOR), 1].
    pub delta: f64,
    /// J, periods from the start of a build to its first use. At least 1.
    pub build_lag: u32,
    /// What it is built from.
    pub recipe: PlantRecipe,
}

/// s1 = θ^(θ/(1−θ))·(1 − θ)/δ, the bundle plant's size at which its long run at ρ = 0 is the
/// flow economy (docs/unit-1g.md §4.5): 40.47 bundles a plant unit at θ 0.8 and δ 10% a year per
/// week. θ in [SCALE_FLOOR, 1); δ positive.
pub fn s1_size(bundle_share: f64, delta: f64) -> f64 {
    let theta = bundle_share;
    (num::pow(theta, theta / (1.0 - theta)) * (1.0 - theta)) / delta
}

/// A unit-1c economy with plants on some of its flow types, validated (docs/unit-1g.md §3.2).
#[derive(Clone, Debug, PartialEq)]
pub struct PlantEconomy<S = PowerSchedule> {
    base: MachineParams<S>,
    plants: Vec<(usize, Plant)>,
    /// R per plant unit, per plant: s times the bundle, or the fixed recipe.
    recipes: Vec<Recipe>,
    /// u = (ρ + δ)(1 + ρ)^(J − 1) per plant.
    user_costs: Vec<f64>,
}

/// Why plants were refused, or their long run not found (docs/unit-1g.md §5.3).
#[derive(Clone, Debug, PartialEq)]
pub enum PlantError {
    /// The long run's unit-1c economy at a step failed validation.
    Param(ParamError),
    /// The solve of the unplanted economy or of a step failed.
    Solve(SolveError),
    /// The fixed point needs an interior equilibrium at every step (its prices set the next
    /// ratios); the unplanted economy (step 0) or a step's long run was a boundary regime.
    NotInterior {
        /// The step, 0 for the unplanted economy.
        step: u32,
        /// The regime's name.
        regime: &'static str,
    },
    /// The ratios did not settle within the steps allowed.
    NoFixedPoint {
        /// The steps taken.
        steps: u32,
        /// The largest |ln r′ − ln r| at the last step.
        gap: f64,
    },
}

impl fmt::Display for PlantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlantError::Param(e) => write!(f, "the plants' long run is not a valid economy: {e}"),
            PlantError::Solve(e) => write!(f, "{e}"),
            PlantError::NotInterior { step, regime } => write!(
                f,
                "the plants' fixed point needs an interior equilibrium at every step; step {step} \
                 is {regime}"
            ),
            PlantError::NoFixedPoint { steps, gap } => write!(
                f,
                "the plants' ratios did not settle in {steps} steps: the last moved ln(kappa/zeta) \
                 by {gap:e}"
            ),
        }
    }
}

impl std::error::Error for PlantError {}

/// A recipe scaled by c.
fn scaled(recipe: &Recipe, c: f64) -> Recipe {
    Recipe {
        machines: recipe.machines.iter().map(|a| a * c).collect(),
        labor: recipe.labor * c,
        land: recipe.land * c,
    }
}

/// A recipe's cost at the type prices, the wage and r = 1: Σ_l a_l·p_l + λ·v + b.
fn cost(recipe: &Recipe, prices: &[f64], v: f64) -> f64 {
    let mut sum = 0.0;
    for (a, p) in recipe.machines.iter().zip(prices) {
        sum += a * p;
    }
    (sum + recipe.labor * v) + recipe.land
}

impl<S: Schedule + Clone> PlantEconomy<S> {
    /// Validates the unplanted economy ([`MachineEconomy::new`]) and each plant, in order: its
    /// type in range and planted once, a flow type (no build recipe), θ and δ in
    /// [SCALE_FLOOR, 1], J ≥ 1 with a finite u, a bundle's size scale, a fixed recipe's
    /// machines one entry per type in [0, SCALE_CEIL], labour in [0, SCALE_CEIL] and land 0 or
    /// scale, not all zero. A plant's error is `ParamError::Item { kind: "plant", .. }`.
    pub fn new(
        base: MachineParams<S>,
        mut plants: Vec<(usize, Plant)>,
    ) -> Result<Self, ParamError> {
        let base = MachineEconomy::new(base)?.params().clone();
        let count = base.machine_types.len();
        let mut planted = vec![false; count];
        let mut recipes = Vec::with_capacity(plants.len());
        let mut user_costs = Vec::with_capacity(plants.len());
        for (index, (k, plant)) in plants.iter_mut().enumerate() {
            let item = |error: ParamError| ParamError::Item {
                kind: "plant",
                index,
                error: Box::new(error),
            };
            if *k >= count || planted[*k] {
                return Err(item(ParamError::Invalid {
                    name: "machine_type",
                    reason: "a plant names a machine type of the economy, each type at most once",
                }));
            }
            planted[*k] = true;
            let flow = &base.machine_types[*k];
            let build = &flow.build;
            if build.machines.iter().any(|&a| a != 0.0) || build.labor != 0.0 || build.land != 0.0 {
                return Err(item(ParamError::Invalid {
                    name: "machine_type",
                    reason: "a plant goes on a flow type, one with no build recipe",
                }));
            }
            plant.bundle_share =
                params::depreciation("bundle_share", plant.bundle_share).map_err(item)?;
            plant.delta = params::depreciation("delta", plant.delta).map_err(item)?;
            if plant.build_lag == 0 {
                return Err(item(ParamError::BuildLag {
                    value: plant.build_lag,
                }));
            }
            let u = user_cost(base.rho, plant.delta, plant.build_lag);
            if !u.is_finite() {
                return Err(item(ParamError::UserCostNotFinite { u }));
            }
            user_costs.push(u);
            let recipe = match &mut plant.recipe {
                PlantRecipe::Bundle { size } => {
                    *size = params::scale("size", *size).map_err(item)?;
                    scaled(&flow.operating, *size)
                }
                PlantRecipe::Fixed(recipe) => {
                    if recipe.machines.len() != count {
                        return Err(item(ParamError::Invalid {
                            name: "recipe.machines",
                            reason: "a plant's recipe needs one entry per machine type",
                        }));
                    }
                    for a in &mut recipe.machines {
                        *a = params::nonnegative("recipe.machines", *a).map_err(item)?;
                    }
                    recipe.labor =
                        params::nonnegative("recipe.labor", recipe.labor).map_err(item)?;
                    recipe.land =
                        params::zero_or_scale("recipe.land", recipe.land).map_err(item)?;
                    if recipe.machines.iter().all(|&a| a == 0.0)
                        && recipe.labor == 0.0
                        && recipe.land == 0.0
                    {
                        return Err(item(ParamError::Invalid {
                            name: "recipe",
                            reason: "a plant's recipe must use machine services, labour or land",
                        }));
                    }
                    recipe.clone()
                }
            };
            recipes.push(recipe);
        }
        Ok(PlantEconomy {
            base,
            plants,
            recipes,
            user_costs,
        })
    }

    /// The unplanted economy's parameters, validated.
    pub fn base(&self) -> &MachineParams<S> {
        &self.base
    }

    /// The plants, validated, with their types.
    pub fn plants(&self) -> &[(usize, Plant)] {
        &self.plants
    }

    /// The long run's unit-1c parameters at the ratios κ/ζ, one per plant (docs/unit-1g.md
    /// §4.5): each planted type's operating recipe ζ times its bundle, its build recipe κ times
    /// R, ζ = r^(θ−1) and κ = r^θ, and the plant's δ and J. A plant with θ = 1 leaves its type
    /// as it is, bit for bit.
    pub fn long_run(&self, ratios: &[f64]) -> MachineParams<S> {
        assert_eq!(ratios.len(), self.plants.len(), "one ratio per plant");
        let mut params = self.base.clone();
        for (((k, plant), recipe), &r) in self.plants.iter().zip(&self.recipes).zip(ratios) {
            let theta = plant.bundle_share;
            if theta == 1.0 {
                continue;
            }
            let (zeta, kappa) = (num::pow(r, theta - 1.0), num::pow(r, theta));
            let flow = &self.base.machine_types[*k];
            params.machine_types[*k] = MachineType {
                task_efficiency: flow.task_efficiency,
                operating: scaled(&flow.operating, zeta),
                build: scaled(recipe, kappa),
                delta: plant.delta,
                build_lag: plant.build_lag,
            };
        }
        params
    }

    /// The ratios κ/ζ that an equilibrium's prices give (docs/unit-1g.md §4.5): a bundle plant's
    /// (1 − θ)/(θ·u·s), free of prices; a fixed recipe's (1 − θ)·c_f/(θ·u·P_K), with c_f the
    /// bundle's cost and P_K the plant's at the equilibrium's type prices and wage. 1.0 for a
    /// plant with θ = 1, which [`PlantEconomy::long_run`] ignores.
    pub fn ratios_at(&self, eq: &Eq1c) -> Vec<f64> {
        let prices: Vec<f64> = eq.types.iter().map(|t| t.price).collect();
        self.ratios_with(Some((&prices, eq.v)))
    }

    /// The ratios at the prices (`None`: each bundle plant's, the fixed ones' 1.0).
    fn ratios_with(&self, prices: Option<(&[f64], f64)>) -> Vec<f64> {
        self.plants
            .iter()
            .zip(&self.recipes)
            .zip(&self.user_costs)
            .map(|(((k, plant), recipe), &u)| {
                let theta = plant.bundle_share;
                if theta == 1.0 {
                    return 1.0;
                }
                match (&plant.recipe, prices) {
                    (PlantRecipe::Bundle { size }, _) => (1.0 - theta) / ((theta * u) * size),
                    (PlantRecipe::Fixed(_), Some((p, v))) => {
                        let bundle = cost(&self.base.machine_types[*k].operating, p, v);
                        ((1.0 - theta) * bundle) / ((theta * u) * cost(recipe, p, v))
                    }
                    (PlantRecipe::Fixed(_), None) => 1.0,
                }
            })
            .collect()
    }

    /// Whether every plant's ratio is free of prices (a bundle, or θ = 1).
    fn closed_form(&self) -> bool {
        self.plants.iter().all(|(_, plant)| {
            plant.bundle_share == 1.0 || matches!(plant.recipe, PlantRecipe::Bundle { .. })
        })
    }

    /// The plants' long run (docs/unit-1g.md §5.3), within [`MAX_PLANT_STEPS`].
    pub fn solve(&self) -> Result<Regime<PlantEq>, PlantError> {
        self.solve_within(MAX_PLANT_STEPS)
    }

    /// The plants' long run within `max_steps` steps. With only bundle plants, the ratios in
    /// closed form and one solve, whose regime is returned. Otherwise a fixed point: from the
    /// unplanted economy's equilibrium, each step solves the long run at the ratios, which must
    /// be interior, and moves each ln r half way to what its prices give, until the largest
    /// move is at most [`PLANT_TOL`]; that step's equilibrium is returned.
    pub fn solve_within(&self, max_steps: u32) -> Result<Regime<PlantEq>, PlantError> {
        let solve = |ratios: &[f64]| -> Result<Regime<Eq1c>, PlantError> {
            let economy = MachineEconomy::new(self.long_run(ratios)).map_err(PlantError::Param)?;
            economy.solve().map_err(PlantError::Solve)
        };
        if self.closed_form() {
            let ratios = self.ratios_with(None);
            return Ok(match solve(&ratios)? {
                Regime::Interior(eq) => {
                    Regime::Interior(Box::new(self.plant_eq(*eq, ratios, 0, 0.0)))
                }
                Regime::BoundaryNoMargin { f_at_1 } => Regime::BoundaryNoMargin { f_at_1 },
                Regime::NotViable { d_at_1 } => Regime::NotViable { d_at_1 },
                Regime::NoInteriorAtZero { f_at_0 } => Regime::NoInteriorAtZero { f_at_0 },
            });
        }
        let base = MachineEconomy::new(self.base.clone())
            .map_err(PlantError::Param)?
            .solve()
            .map_err(PlantError::Solve)?;
        let Regime::Interior(base) = base else {
            return Err(PlantError::NotInterior {
                step: 0,
                regime: base.name(),
            });
        };
        let mut logs: Vec<f64> = self.ratios_at(&base).into_iter().map(num::ln).collect();
        let mut gap = f64::INFINITY;
        for step in 1..=max_steps {
            let ratios: Vec<f64> = logs.iter().map(|&s| num::exp(s)).collect();
            let eq = match solve(&ratios)? {
                Regime::Interior(eq) => eq,
                other => {
                    return Err(PlantError::NotInterior {
                        step,
                        regime: other.name(),
                    })
                }
            };
            let moves: Vec<f64> = self
                .ratios_at(&eq)
                .into_iter()
                .zip(&logs)
                .map(|(r, &s)| num::ln(r) - s)
                .collect();
            gap = moves.iter().fold(0.0, |g: f64, m| g.max(m.abs()));
            if gap <= PLANT_TOL {
                return Ok(Regime::Interior(Box::new(
                    self.plant_eq(*eq, ratios, step, gap),
                )));
            }
            for (s, m) in logs.iter_mut().zip(&moves) {
                *s += 0.5 * m;
            }
        }
        Err(PlantError::NoFixedPoint {
            steps: max_steps,
            gap,
        })
    }

    /// The readouts of docs/unit-1g.md §4.5 at an equilibrium of the long run at `ratios`.
    fn plant_eq(&self, eq: Eq1c, ratios: Vec<f64>, steps: u32, gap: f64) -> PlantEq {
        let prices: Vec<f64> = eq.types.iter().map(|t| t.price).collect();
        let plants = self
            .plants
            .iter()
            .zip(&self.recipes)
            .zip(&ratios)
            .map(|(((k, plant), recipe), &r)| {
                let theta = plant.bundle_share;
                let (zeta, kappa) = if theta == 1.0 {
                    (1.0, 0.0)
                } else {
                    (num::pow(r, theta - 1.0), num::pow(r, theta))
                };
                let t = &eq.types[*k];
                PlantReadout {
                    machine_type: *k,
                    ratio: r,
                    zeta,
                    kappa,
                    stock: kappa * t.services,
                    bundles: zeta * t.services,
                    bundle_cost: cost(&self.base.machine_types[*k].operating, &prices, eq.v),
                    plant_price: cost(recipe, &prices, eq.v),
                    capital_share: (t.user_cost * t.build_cost) / t.price,
                }
            })
            .collect();
        PlantEq {
            eq: Box::new(eq),
            ratios,
            plants,
            steps,
            gap,
        }
    }
}

/// One plant at the long run (docs/unit-1g.md §4.5), in r = 1 units.
#[derive(Clone, Debug, PartialEq)]
pub struct PlantReadout {
    /// The planted type's index.
    pub machine_type: usize,
    /// r = κ/ζ.
    pub ratio: f64,
    /// ζ, bundles per unit of service.
    pub zeta: f64,
    /// κ, plant units per unit of capacity.
    pub kappa: f64,
    /// K* = κ·X, the plant installed.
    pub stock: f64,
    /// z = ζ·X, bundles a period.
    pub bundles: f64,
    /// c_f, the bundle's cost at the equilibrium's prices.
    pub bundle_cost: f64,
    /// P_K, a plant unit's cost at the equilibrium's prices (V = κ·P_K).
    pub plant_price: f64,
    /// u·V/p, the plant's user cost as a share of the price: 1 − θ at the long run.
    pub capital_share: f64,
}

/// The plants' long run: its unit-1c equilibrium and each plant's readout.
#[derive(Clone, Debug, PartialEq)]
pub struct PlantEq {
    /// The long run's unit-1c equilibrium.
    pub eq: Box<Eq1c>,
    /// The ratios κ/ζ, one per plant.
    pub ratios: Vec<f64>,
    /// Each plant, in order.
    pub plants: Vec<PlantReadout>,
    /// The fixed point's steps (0 in closed form).
    pub steps: u32,
    /// The last step's largest |ln r′ − ln r| (0 in closed form).
    pub gap: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s1_is_the_size_that_makes_zeta_theta() {
        // At ρ = 0, u = δ and r = (1 − θ)/(θ·δ·s1) = θ^(−1/(1−θ)), so ζ = r^(θ−1) = θ.
        for (theta, delta) in [(0.8, 0.002), (0.7, 0.1), (0.5, 1.0)] {
            let s = s1_size(theta, delta);
            let r = (1.0 - theta) / ((theta * delta) * s);
            let zeta = num::pow(r, theta - 1.0);
            assert!(
                (zeta - theta).abs() <= 4.0 * f64::EPSILON,
                "{theta}: {zeta}"
            );
        }
        // θ 0.8: s1·δ = 0.8^4·0.2 = 0.08192.
        assert!((s1_size(0.8, 1.0) - 0.08192).abs() < 1e-16);
    }

    #[test]
    fn a_recipe_costs_in_index_order() {
        let r = Recipe {
            machines: vec![0.5, 2.0],
            labor: 0.25,
            land: 1.0,
        };
        assert_eq!(cost(&r, &[1.0, 0.5], 4.0), (0.5 + 1.0 + 1.0) + 1.0);
        assert_eq!(scaled(&r, 2.0).machines, vec![1.0, 4.0]);
    }
}
