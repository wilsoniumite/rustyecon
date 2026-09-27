//! Unit 1g: machines as goods (docs/unit-1g.md §2.2-2.5, §3, §4.1-4.4 and §5.2).
//!
//! A chain of goods: categories made on the task line, as in units 1b and 1c; materials made by
//! fixed recipes (fodder, coal, iron); and machines, each a durable good held as a stock (a
//! horse, an engine), built from goods by its build recipe, whose hours of use (horse-days,
//! engine-hours) are the paper's machine services, run on goods by its operating recipe. When no
//! material, build or operating recipe uses a category (E1), and categories use materials and
//! hours only through their tasks (E2), the chain is a unit-1c economy with more rows
//! (ORACLE-GOODS §1.3): each material and machine good a flow type, each machine's hours a type
//! built from its good. [`GoodsChain::to_machine_params`] builds that economy, and
//! [`ChainEconomy`] solves it and reads the equilibrium back per good.
//!
//! Everything is per period: δ, ρ and J are per period, and a tape's yearly values are
//! converted where tapes are built, with core's `Clock` (docs/unit-1g.md §2.3).

use std::collections::BTreeMap;
use std::fmt;

use crate::categories::Category;
use crate::machine_block::{MachineType, Recipe};
use crate::machines::{Eq1c, MachineEconomy, MachineParams};
use crate::params::{self, ParamError, UniformWorkCost};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{Regime, SolveError};

/// A recipe over goods, per unit made (docs/unit-1g.md §3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct GoodsRecipe {
    /// (a good's key, units of it per unit made). Each key at most once; a material's, a machine
    /// good's or a machine's hours, never a category's (E1).
    pub inputs: Vec<(String, f64)>,
    /// Hours of pool labour (decision 140).
    pub labor: f64,
    /// Land services.
    pub land: f64,
}

/// A good made on the task line: one of unit 1b's categories, with the categories it uses.
#[derive(Clone, Debug, PartialEq)]
pub struct ChainCategory {
    /// Its key.
    pub key: String,
    /// Its weight in the basket, direct land and task densities (unit 1b).
    pub category: Category,
    /// (a category's key, units of it per unit made): unit 1c's intermediate inputs. Only
    /// categories (E2).
    pub inputs: Vec<(String, f64)>,
}

/// A material: a good made by a fixed recipe, with no task margin (fodder, coal, iron).
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// Its key.
    pub key: String,
    /// Its recipe, per unit.
    pub recipe: GoodsRecipe,
}

/// A machine: a durable good held as a stock, and the hours of its use (docs/unit-1g.md §2.5).
#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    /// The good's key (the stock: a head, an engine).
    pub key: String,
    /// The build recipe, per unit of stock.
    pub build: GoodsRecipe,
    /// The key of its hours (its service: horse-days, engine-hours).
    pub hours: String,
    /// κ, hours a period per unit of stock. Scale.
    pub hours_per_period: f64,
    /// θ, task units per hour relative to the line's γ, as unit 1c's `task_efficiency`; 0 for a
    /// machine that does no tasks.
    pub task_efficiency: f64,
    /// The operating recipe, per hour.
    pub operating: GoodsRecipe,
    /// δ, the stock's wear per period.
    pub delta: f64,
    /// J, periods from the start of a build to its first hour.
    pub build_lag: u32,
}

/// A chain of goods (docs/unit-1g.md §3.1): unit 1c's economy with its machine types written as
/// materials and machines over goods by key. Build a [`ChainEconomy`] from it to validate.
#[derive(Clone, Debug, PartialEq)]
pub struct GoodsChain<S = PowerSchedule> {
    /// N, potential workers.
    pub workers: f64,
    /// T, the land-service endowment per period.
    pub land: f64,
    /// γ(x), relative human productivity on the one task line.
    pub schedule: S,
    /// F, the distribution of the work cost χ.
    pub work_cost: UniformWorkCost,
    /// ρ, the interest rate per period.
    pub rho: f64,
    /// The task line's segments' edges (unit 1b).
    pub edges: Vec<f64>,
    /// The categories, in order.
    pub categories: Vec<ChainCategory>,
    /// The materials, in order.
    pub materials: Vec<Material>,
    /// The machines, in order.
    pub machines: Vec<Machine>,
}

/// Where a good sits in the unit-1c economy (docs/unit-1g.md §4.2): a category's index, or a
/// machine type's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// Category j of [`MachineParams::categories`].
    Category(usize),
    /// A material: a flow type.
    Material(usize),
    /// A machine good: a flow type whose output is the goods made a period.
    MachineGood(usize),
    /// A machine's hours: a type built from 1/κ of its good.
    Hours(usize),
}

/// Why a chain was refused (docs/unit-1g.md §3.2).
#[derive(Clone, Debug, PartialEq)]
pub enum ChainError {
    /// Two goods share a key.
    DuplicateKey {
        /// The key.
        key: String,
    },
    /// A recipe names a good the chain does not have.
    UnknownGood {
        /// The good whose recipe it is.
        good: String,
        /// The unknown key.
        input: String,
    },
    /// A recipe names an input twice.
    RepeatedInput {
        /// The good whose recipe it is.
        good: String,
        /// The key named twice.
        input: String,
    },
    /// E1 (decision 67 reworded, D-G1): a material's recipe, a machine good's build recipe or a
    /// machine's operating recipe uses a category.
    CategoryInMachineRecipe {
        /// The material, machine good or hours whose recipe it is.
        good: String,
        /// The category.
        input: String,
    },
    /// E2: a category uses a material, a machine good or hours other than through its tasks
    /// (GOODS-CHAIN's addendum G1, which is not built).
    DirectUse {
        /// The category.
        category: String,
        /// The good it names.
        input: String,
    },
    /// A parameter failed: a machine's κ, or unit 1c's validation of the mapped economy, named
    /// by the good its item is (`None` for the economy's own parameters).
    Param {
        /// The good the error names, if any.
        good: Option<String>,
        /// The error.
        error: ParamError,
    },
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainError::DuplicateKey { key } => write!(f, "two goods are keyed {key:?}"),
            ChainError::UnknownGood { good, input } => {
                write!(
                    f,
                    "{good:?}'s recipe names {input:?}, which is not a good of the chain"
                )
            }
            ChainError::RepeatedInput { good, input } => {
                write!(f, "{good:?}'s recipe names {input:?} twice")
            }
            ChainError::CategoryInMachineRecipe { good, input } => write!(
                f,
                "{good:?}'s recipe uses the category {input:?}: machine-side recipes use \
                 materials, machine goods, hours, labour and land, never a category (E1)"
            ),
            ChainError::DirectUse { category, input } => write!(
                f,
                "the category {category:?} uses {input:?} directly: categories use materials and \
                 hours only through their tasks (E2)"
            ),
            ChainError::Param {
                good: Some(good),
                error,
            } => write!(f, "{good:?}: {error}"),
            ChainError::Param { good: None, error } => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChainError {}

impl<S: Clone> GoodsChain<S> {
    /// Every good's key and its row in the unit-1c economy, in the order categories, materials,
    /// machine goods, hours (docs/unit-1g.md §4.2); an error if two goods share a key.
    pub fn rows(&self) -> Result<Vec<(String, Row)>, ChainError> {
        let (materials, machines) = (self.materials.len(), self.machines.len());
        let mut rows = Vec::with_capacity(self.categories.len() + materials + 2 * machines);
        for (j, c) in self.categories.iter().enumerate() {
            rows.push((c.key.clone(), Row::Category(j)));
        }
        for (f, m) in self.materials.iter().enumerate() {
            rows.push((m.key.clone(), Row::Material(f)));
        }
        for (i, m) in self.machines.iter().enumerate() {
            rows.push((m.key.clone(), Row::MachineGood(materials + i)));
        }
        for (i, m) in self.machines.iter().enumerate() {
            rows.push((m.hours.clone(), Row::Hours(materials + machines + i)));
        }
        let mut seen = BTreeMap::new();
        for (key, _) in &rows {
            if seen.insert(key.as_str(), ()).is_some() {
                return Err(ChainError::DuplicateKey { key: key.clone() });
            }
        }
        Ok(rows)
    }

    /// Unit 1c's parameters for the chain (docs/unit-1g.md §4.2 and §5.2), after §3.2's checks 1
    /// to 5: keys, inputs, E1, E2 and κ. The machine types are the materials, then the machine
    /// goods, then each machine's hours: a material or a machine good is a flow type (θ 0, its
    /// recipe as the operating recipe, no build recipe, δ 1, J 1); a machine's hours are a type
    /// with its θ, its operating recipe, a build recipe of 1/κ of its good, and its δ and J. The
    /// categories' inputs are unit 1c's intermediate inputs. Unit 1c validates the rest
    /// ([`ChainEconomy::new`]).
    pub fn to_machine_params(&self) -> Result<MachineParams<S>, ChainError> {
        let rows = self.rows()?;
        let index: BTreeMap<&str, Row> = rows.iter().map(|(k, r)| (k.as_str(), *r)).collect();
        let count = self.materials.len() + 2 * self.machines.len();
        // A machine-side recipe as a vector over the types (E1).
        let machine_side = |good: &str, recipe: &GoodsRecipe| -> Result<Recipe, ChainError> {
            let mut machines = vec![0.0; count];
            let mut named = BTreeMap::new();
            for (input, units) in &recipe.inputs {
                if named.insert(input.as_str(), ()).is_some() {
                    return Err(ChainError::RepeatedInput {
                        good: good.to_string(),
                        input: input.clone(),
                    });
                }
                match index.get(input.as_str()) {
                    None => {
                        return Err(ChainError::UnknownGood {
                            good: good.to_string(),
                            input: input.clone(),
                        })
                    }
                    Some(Row::Category(_)) => {
                        return Err(ChainError::CategoryInMachineRecipe {
                            good: good.to_string(),
                            input: input.clone(),
                        })
                    }
                    Some(Row::Material(k) | Row::MachineGood(k) | Row::Hours(k)) => {
                        machines[*k] = *units;
                    }
                }
            }
            Ok(Recipe {
                machines,
                labor: recipe.labor,
                land: recipe.land,
            })
        };
        let mut types = Vec::with_capacity(count);
        for m in &self.materials {
            types.push(MachineType {
                task_efficiency: 0.0,
                operating: machine_side(&m.key, &m.recipe)?,
                build: Recipe::zero(count),
                delta: 1.0,
                build_lag: 1,
            });
        }
        for m in &self.machines {
            types.push(MachineType {
                task_efficiency: 0.0,
                operating: machine_side(&m.key, &m.build)?,
                build: Recipe::zero(count),
                delta: 1.0,
                build_lag: 1,
            });
        }
        for (i, m) in self.machines.iter().enumerate() {
            let kappa = params::scale("hours_per_period", m.hours_per_period).map_err(|error| {
                ChainError::Param {
                    good: Some(m.key.clone()),
                    error,
                }
            })?;
            let mut build = Recipe::zero(count);
            build.machines[self.materials.len() + i] = 1.0 / kappa;
            types.push(MachineType {
                task_efficiency: m.task_efficiency,
                operating: machine_side(&m.hours, &m.operating)?,
                build,
                delta: m.delta,
                build_lag: m.build_lag,
            });
        }
        // The categories' inputs (E2).
        let categories = self.categories.len();
        let mut intermediate = vec![vec![0.0; categories]; categories];
        for (j, c) in self.categories.iter().enumerate() {
            let mut named = BTreeMap::new();
            for (input, units) in &c.inputs {
                if named.insert(input.as_str(), ()).is_some() {
                    return Err(ChainError::RepeatedInput {
                        good: c.key.clone(),
                        input: input.clone(),
                    });
                }
                match index.get(input.as_str()) {
                    None => {
                        return Err(ChainError::UnknownGood {
                            good: c.key.clone(),
                            input: input.clone(),
                        })
                    }
                    Some(Row::Category(l)) => intermediate[j][*l] = *units,
                    Some(_) => {
                        return Err(ChainError::DirectUse {
                            category: c.key.clone(),
                            input: input.clone(),
                        })
                    }
                }
            }
        }
        Ok(MachineParams {
            workers: self.workers,
            land: self.land,
            schedule: self.schedule.clone(),
            work_cost: self.work_cost,
            rho: self.rho,
            machine_types: types,
            edges: self.edges.clone(),
            categories: self.categories.iter().map(|c| c.category.clone()).collect(),
            intermediate,
        })
    }
}

/// A validated chain: its unit-1c economy, and where each good sits in it
/// (docs/unit-1g.md §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct ChainEconomy<S = PowerSchedule> {
    chain: GoodsChain<S>,
    economy: MachineEconomy<S>,
    rows: Vec<(String, Row)>,
}

impl<S: Schedule + Clone> ChainEconomy<S> {
    /// Maps the chain ([`GoodsChain::to_machine_params`]) and validates the result with
    /// [`MachineEconomy::new`] (unit 1c §3.2 with D-G10's rule, docs/unit-1g.md §2.1), naming a
    /// machine type's or a category's error by its good.
    pub fn new(chain: GoodsChain<S>) -> Result<Self, ChainError> {
        let params = chain.to_machine_params()?;
        let rows = chain.rows()?;
        let economy = MachineEconomy::new(params).map_err(|error| {
            let good = match &error {
                ParamError::Item { kind, index, .. } => rows
                    .iter()
                    .find(|(_, row)| match (row, *kind) {
                        (Row::Category(j), "category") => j == index,
                        (
                            Row::Material(k) | Row::MachineGood(k) | Row::Hours(k),
                            "machine type",
                        ) => k == index,
                        _ => false,
                    })
                    .map(|(key, _)| key.clone()),
                _ => None,
            };
            ChainError::Param { good, error }
        })?;
        Ok(ChainEconomy {
            chain,
            economy,
            rows,
        })
    }

    /// The chain, as given.
    pub fn chain(&self) -> &GoodsChain<S> {
        &self.chain
    }

    /// The unit-1c economy the chain maps to.
    pub fn economy(&self) -> &MachineEconomy<S> {
        &self.economy
    }

    /// Every good's key and row, in the order categories, materials, machine goods, hours.
    pub fn rows(&self) -> &[(String, Row)] {
        &self.rows
    }

    /// The row of the good keyed `key`.
    pub fn row(&self, key: &str) -> Option<Row> {
        self.rows.iter().find(|(k, _)| k == key).map(|(_, r)| *r)
    }

    /// Unit 1c's solve of the economy, its interior equilibrium read per good
    /// ([`ChainEconomy::readout`]); a boundary regime or an error is 1c's.
    pub fn solve(&self) -> Result<Regime<ChainEq>, SolveError> {
        Ok(match self.economy.solve()? {
            Regime::Interior(eq) => Regime::Interior(Box::new(self.readout(*eq))),
            Regime::BoundaryNoMargin { f_at_1 } => Regime::BoundaryNoMargin { f_at_1 },
            Regime::NotViable { d_at_1 } => Regime::NotViable { d_at_1 },
            Regime::NoInteriorAtZero { f_at_0 } => Regime::NoInteriorAtZero { f_at_0 },
        })
    }

    /// A unit-1c equilibrium of the economy read per good (docs/unit-1g.md §4.4).
    pub fn readout(&self, eq: Eq1c) -> ChainEq {
        let goods = self
            .rows
            .iter()
            .map(|(key, row)| {
                let (price, output) = match *row {
                    Row::Category(j) => {
                        let c = &eq.categories[j];
                        (c.price, c.gross_output)
                    }
                    Row::Material(k) | Row::MachineGood(k) | Row::Hours(k) => {
                        let t = &eq.types[k];
                        (t.price, t.services)
                    }
                };
                GoodEq {
                    key: key.clone(),
                    row: *row,
                    price,
                    output,
                }
            })
            .collect();
        let offset = self.chain.materials.len();
        let count = self.chain.machines.len();
        let machines = self
            .chain
            .machines
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let good = &eq.types[offset + i];
                let hours = &eq.types[offset + count + i];
                let kappa = m.hours_per_period;
                MachineEq {
                    key: m.key.clone(),
                    hours_key: m.hours.clone(),
                    price: good.price,
                    made: good.services,
                    stock: hours.services / kappa,
                    hour_price: hours.price,
                    operating_cost: hours.operating_cost,
                    build_cost: hours.build_cost,
                    hours: hours.services,
                    user_cost: hours.user_cost,
                    wealth: hours.wealth,
                    interest: hours.interest,
                }
            })
            .collect();
        ChainEq {
            eq: Box::new(eq),
            goods,
            machines,
        }
    }
}

/// One good at the equilibrium (docs/unit-1g.md §4.4).
#[derive(Clone, Debug, PartialEq)]
pub struct GoodEq {
    /// Its key.
    pub key: String,
    /// Its row in the unit-1c economy.
    pub row: Row,
    /// Its price: a category's p_j, a material's or machine good's unit cost (a machine good's
    /// per unit of stock), a machine's hour price p = O + uV.
    pub price: f64,
    /// Its gross output a period: a category's, a material's, the machine goods made (δX/κ when
    /// nothing else uses them), a machine's hours X.
    pub output: f64,
}

/// One machine at the equilibrium (docs/unit-1g.md §4.4), in r = 1 units.
#[derive(Clone, Debug, PartialEq)]
pub struct MachineEq {
    /// The good's key.
    pub key: String,
    /// The hours' key.
    pub hours_key: String,
    /// The good's price per unit of stock, κ·V.
    pub price: f64,
    /// Units of the good made a period (its flow type's output).
    pub made: f64,
    /// Units of the good installed, X/κ.
    pub stock: f64,
    /// An hour's price, p = O + uV.
    pub hour_price: f64,
    /// An hour's operating cost O.
    pub operating_cost: f64,
    /// V, the good's cost per unit of capacity (one hour a period): price/κ.
    pub build_cost: f64,
    /// Hours X a period.
    pub hours: f64,
    /// u.
    pub user_cost: f64,
    /// W = ω·V·X.
    pub wealth: f64,
    /// ρ·W.
    pub interest: f64,
}

/// A chain's interior equilibrium: unit 1c's, and each good's and machine's readout.
#[derive(Clone, Debug, PartialEq)]
pub struct ChainEq {
    /// The unit-1c equilibrium.
    pub eq: Box<Eq1c>,
    /// Every good, in [`ChainEconomy::rows`]' order.
    pub goods: Vec<GoodEq>,
    /// Every machine, in the chain's order.
    pub machines: Vec<MachineEq>,
}

impl ChainEq {
    /// The good keyed `key`.
    pub fn good(&self, key: &str) -> Option<&GoodEq> {
        self.goods.iter().find(|g| g.key == key)
    }

    /// The machine whose good is keyed `key`.
    pub fn machine(&self, key: &str) -> Option<&MachineEq> {
        self.machines.iter().find(|m| m.key == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(inputs: &[(&str, f64)], labor: f64, land: f64) -> GoodsRecipe {
        GoodsRecipe {
            inputs: inputs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            labor,
            land,
        }
    }

    fn chain() -> GoodsChain {
        GoodsChain {
            workers: 4.0,
            land: 10.0,
            schedule: PowerSchedule {
                eta: 1.0,
                g0: 0.2,
                g1: 0.8,
                k: 1.0,
            },
            work_cost: UniformWorkCost { chi_max: 1.0 },
            rho: 0.0,
            edges: vec![0.0, 1.0],
            categories: vec![
                ChainCategory {
                    key: "good".into(),
                    category: Category {
                        weight: 1.0,
                        direct_land: 0.0,
                        density: vec![1.0],
                    },
                    inputs: vec![],
                },
                ChainCategory {
                    key: "space".into(),
                    category: Category {
                        weight: 1.0,
                        direct_land: 1.0,
                        density: vec![0.0],
                    },
                    inputs: vec![],
                },
            ],
            materials: vec![Material {
                key: "fodder".into(),
                recipe: recipe(&[], 0.0, 0.2),
            }],
            machines: vec![Machine {
                key: "horse".into(),
                build: recipe(&[("horse-days", 3.0)], 0.5, 2.0),
                hours: "horse-days".into(),
                hours_per_period: 4.0,
                task_efficiency: 1.0,
                operating: recipe(&[("fodder", 1.0)], 0.0, 0.0),
                delta: 0.1,
                build_lag: 1,
            }],
        }
    }

    #[test]
    fn the_embedding_writes_each_good_as_a_type() {
        let c = chain();
        let p = c.to_machine_params().unwrap();
        assert_eq!(p.machine_types.len(), 3);
        let (fodder, horse, days) = (
            &p.machine_types[0],
            &p.machine_types[1],
            &p.machine_types[2],
        );
        assert_eq!(fodder.operating.land, 0.2);
        assert_eq!((fodder.delta, fodder.build_lag), (1.0, 1));
        assert_eq!(horse.task_efficiency, 0.0);
        assert_eq!(horse.operating.machines, vec![0.0, 0.0, 3.0]);
        assert_eq!(horse.build, Recipe::zero(3));
        assert_eq!(days.operating.machines, vec![1.0, 0.0, 0.0]);
        assert_eq!(days.build.machines, vec![0.0, 0.25, 0.0]);
        assert_eq!((days.delta, days.build_lag), (0.1, 1));
        assert_eq!(
            c.rows().unwrap(),
            vec![
                ("good".to_string(), Row::Category(0)),
                ("space".to_string(), Row::Category(1)),
                ("fodder".to_string(), Row::Material(0)),
                ("horse".to_string(), Row::MachineGood(1)),
                ("horse-days".to_string(), Row::Hours(2)),
            ]
        );
    }

    #[test]
    fn errors_name_their_goods() {
        let mut c = chain();
        c.machines[0].hours = "fodder".into();
        assert_eq!(
            c.to_machine_params(),
            Err(ChainError::DuplicateKey {
                key: "fodder".into()
            })
        );
        let mut c = chain();
        c.materials[0].recipe.inputs.push(("space".into(), 1.0));
        assert!(matches!(
            c.to_machine_params(),
            Err(ChainError::CategoryInMachineRecipe { good, input }) if good == "fodder" && input == "space"
        ));
        let mut c = chain();
        c.categories[0].inputs.push(("fodder".into(), 1.0));
        assert!(matches!(
            c.to_machine_params(),
            Err(ChainError::DirectUse { category, input }) if category == "good" && input == "fodder"
        ));
        let mut c = chain();
        c.machines[0].delta = 0.0;
        match ChainEconomy::new(c) {
            Err(ChainError::Param {
                good: Some(good), ..
            }) => assert_eq!(good, "horse-days"),
            other => panic!("{other:?}"),
        }
        for e in [
            ChainError::DuplicateKey { key: "a".into() },
            ChainError::DirectUse {
                category: "a".into(),
                input: "b".into(),
            },
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
