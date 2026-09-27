//! The markets probe's instances (MARKETS-SPEC §1): economies with several final categories on
//! one task line (oracle unit 1b, decision 60) and several machine types (unit 1c), each in the
//! flow case (ρ, δ, J) = (0, 1, 1), and their known answers.
//!
//! An [`Instance`] is the economy as the tape registers it: every coefficient is a tape param
//! with a unit and a basis (R4), keyed as MARKETS-SPEC §2.9 suggests. The oracle sees it through
//! [`Instance::machine_params`]: every type's flow recipe as its operating recipe, a zero build
//! recipe, δ = 1, J = 1 and ρ = 0 (MARKETS-SPEC §1.2, M9), with a household's space as a
//! category with no tasks and direct land 1, last. For I0 that is unit 1a's G1 bit for bit
//! (`markets_i0_genesis_is_appb`). The oracle is solved here, in the harness, outside any `Sim`;
//! no agent reads it (R13).

use crate::setup::clock;
use oracle::{
    Category as OCategory, MachineParams, MachineType, PowerSchedule, Recipe, Regime,
    UniformWorkCost,
};
use rustyecon_core::FlowPerYear;

/// A final category with tasks, served by its own desk (unit-1b.md §3.1).
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    /// Its key: the good, `desk.<key>`, class `<key>_desks`, params `inst.<key>.*`.
    pub key: String,
    /// z_j, units per basket.
    pub weight: f64,
    /// b_j, land per unit, used directly.
    pub land: f64,
    /// μ_js, hours by hand per unit per unit length of line, one per segment.
    pub density: Vec<f64>,
}

/// A machine type, with its flow recipe (unit-1c.md §2; decision 67).
#[derive(Debug, Clone, PartialEq)]
pub struct MachineKind {
    /// Its key: the service good, `desk.<key>`, class `<key>_desks`, params `inst.<key>.*`.
    pub key: String,
    /// θ_k: task units per unit of its service; 0 for a type that does no tasks.
    pub theta: f64,
    /// a_kl: units of each type l's service per unit of this one, over every type in order; the
    /// entry at the type's own index is its own input, kept and not bought.
    pub row: Vec<f64>,
    /// λ_k, hours per unit.
    pub labour: f64,
    /// b_k, land per unit.
    pub land: f64,
}

/// A registered cost-shock target (MARKETS-SPEC §1.5): a coefficient, its param and its values,
/// written as the battery names them.
#[derive(Debug, Clone, PartialEq)]
pub struct Coef {
    /// Its run-grammar name, as MARKETS-SPEC §1.5 writes it (`land.mach`, `b.food`, ...).
    pub name: String,
    /// The tape param it is.
    pub param: String,
    /// Its registered value, as written.
    pub base: String,
    /// ×1.1, ×0.9, ×2 and ×0.5 of it, as written.
    pub values: [String; 4],
}

/// One of MARKETS-SPEC's economies, as the tape registers it.
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    /// Its id: `i0`, `i1`, `i2`, `i3`, `l2`, `l3` or `g1`.
    pub id: String,
    /// What it is, in a line.
    pub title: String,
    /// N, potential hours per year.
    pub workers: f64,
    /// T, land services per year.
    pub land: f64,
    /// χ_max.
    pub chi_max: f64,
    /// η.
    pub eta: f64,
    /// g0.
    pub g0: f64,
    /// g1.
    pub g1: f64,
    /// k.
    pub k: f64,
    /// The interior edges of the task line, in order (0 and 1 are structural).
    pub edges: Vec<f64>,
    /// The categories with tasks, in order: one desk and one good market each.
    pub categories: Vec<Category>,
    /// h, the land a basket holds as space, bought by households on the land market, if any.
    pub space: Option<f64>,
    /// The machine types, in order: one desk and one market each.
    pub types: Vec<MachineKind>,
    /// The registered cost-shock targets.
    pub coefs: Vec<Coef>,
    /// The basis of the categories' coefficients.
    pub category_basis: String,
    /// The basis of the machine types' coefficients.
    pub type_basis: String,
}

/// The scalars' basis, and 1a's machine's.
pub const SCALARS: &str = "SSRN 7226858 App. B via laborformal 31b3482";

fn cat(key: &str, weight: f64, land: f64, density: &[f64]) -> Category {
    Category {
        key: key.into(),
        weight,
        land,
        density: density.to_vec(),
    }
}

fn kind(key: &str, theta: f64, row: &[f64], labour: f64, land: f64) -> MachineKind {
    MachineKind {
        key: key.into(),
        theta,
        row: row.to_vec(),
        labour,
        land,
    }
}

fn coef(name: &str, param: &str, base: &str, values: [&str; 4]) -> Coef {
    Coef {
        name: name.into(),
        param: param.into(),
        base: base.into(),
        values: values.map(str::to_string),
    }
}

/// C1's categories: the good (z 1, b 0, μ (1)) and space (h 1), unit-1b.md §3.3.
fn c1() -> Line {
    (
        Vec::new(),
        vec![cat("good", 1.0, 0.0, &[1.0])],
        Some(1.0),
        C1_BASIS,
    )
}

/// C3's categories, the fork economy, on edges (0, 0.4, 0.75, 1), unit-1b.md §3.3.
fn c3() -> Line {
    (
        vec![0.4, 0.75],
        vec![
            cat("manufactures", 0.3, 0.0, &[2.0, 0.0, 0.0]),
            cat("food", 1.0, 0.6, &[0.5, 1.5, 0.0]),
            cat("care", 0.2, 0.1, &[0.0, 0.0, 1.0]),
            cat("shelter", 0.8, 1.0, &[0.0, 0.4, 0.0]),
        ],
        None,
        C3_BASIS,
    )
}

/// 1b's gap economy, on edges (0, 0.4, 0.6, 1), the middle segment unused (unit-1b.md §3.3).
fn gap() -> Line {
    (
        vec![0.4, 0.6],
        vec![
            cat("manufactures", 0.3, 0.0, &[2.0, 0.0, 0.0]),
            cat("food", 1.0, 0.6, &[0.5, 0.0, 0.2]),
            cat("care", 0.2, 0.1, &[0.0, 0.0, 0.3]),
            cat("shelter", 0.8, 1.0, &[0.0, 0.0, 0.1]),
        ],
        None,
        GAP_BASIS,
    )
}

/// 1a's machine: θ 1, a 0.3, λ 0.05, b 0.4.
fn mach() -> Kinds {
    (vec![kind("mach", 1.0, &[0.3], 0.05, 0.4)], MACH_BASIS)
}

/// M4's engine and power with their operating recipes only (unit-1c.md §3.3): a chain.
fn chain() -> Kinds {
    (
        vec![
            kind("engine", 2.0, &[0.0, 0.5], 0.02, 0.0),
            kind("power", 0.0, &[0.0, 0.0], 0.02, 0.5),
        ],
        CHAIN_BASIS,
    )
}

/// M4's engine and power with operating and build recipes at δ = J = 1, added: a loop.
fn cycle() -> Kinds {
    (
        vec![
            kind("engine", 2.0, &[0.1, 0.5], 0.12, 0.4),
            kind("power", 0.0, &[0.1, 0.0], 0.12, 0.6),
        ],
        LOOP_BASIS,
    )
}

fn land_mach() -> Coef {
    coef(
        "land.mach",
        "inst.mach.land",
        "0.4",
        ["0.44", "0.36", "0.8", "0.2"],
    )
}

fn b_food() -> Coef {
    coef(
        "b.food",
        "inst.food.land",
        "0.6",
        ["0.66", "0.54", "1.2", "0.3"],
    )
}

fn land_power_chain() -> Coef {
    coef(
        "land.power",
        "inst.power.land",
        "0.5",
        ["0.55", "0.45", "1", "0.25"],
    )
}

/// A line's interior edges, its categories, its space and their basis.
type Line = (Vec<f64>, Vec<Category>, Option<f64>, &'static str);

/// Machine types and their basis.
type Kinds = (Vec<MachineKind>, &'static str);

const C1_BASIS: &str = "crates/oracle/docs/unit-1b.md §3.3 C1: Appendix B in category form";
const C3_BASIS: &str = "crates/oracle/docs/unit-1b.md §3.3 C3, the fork economy";
const GAP_BASIS: &str = "crates/oracle/docs/unit-1b.md §3.3, the gap economy (C7)";
const MACH_BASIS: &str = "SSRN 7226858 App. B via laborformal 31b3482";
const CHAIN_BASIS: &str = "crates/oracle/docs/unit-1c.md §3.3 M4, operating recipes, flow";
const LOOP_BASIS: &str = "crates/oracle/docs/unit-1c.md §3.3 M4, operating + build at δ = J = 1";

impl Instance {
    fn make(
        id: &str,
        title: &str,
        workers: f64,
        (edges, categories, space, category_basis): Line,
        (types, type_basis): Kinds,
        coefs: Vec<Coef>,
    ) -> Instance {
        Instance {
            id: id.into(),
            title: title.into(),
            workers,
            land: 520.0,
            chi_max: 1.0,
            eta: 1.0,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
            edges,
            categories,
            space,
            types,
            coefs,
            category_basis: category_basis.into(),
            type_basis: type_basis.into(),
        }
    }

    /// The registered instance with this id (MARKETS-SPEC §1.1, §1.2, §1.5).
    pub fn named(id: &str) -> Result<Instance, String> {
        let land_engine = || {
            coef(
                "land.engine",
                "inst.engine.land",
                "0.4",
                ["0.44", "0.36", "0.8", "0.2"],
            )
        };
        Ok(match id {
            "i0" => Instance::make(
                "i0",
                "Appendix B through the many-market roles (control)",
                208.0,
                c1(),
                mach(),
                vec![land_mach()],
            ),
            "i1" => Instance::make(
                "i1",
                "1b's four-category fork economy (C3), 1a's machine",
                208.0,
                c3(),
                mach(),
                vec![land_mach(), b_food()],
            ),
            "i2" => Instance::make(
                "i2",
                "Appendix B's categories, M4's engine and power on operating recipes (a chain)",
                208.0,
                c1(),
                chain(),
                vec![
                    land_power_chain(),
                    coef(
                        "a.engine.power",
                        "inst.engine.in.power",
                        "0.5",
                        ["0.55", "0.45", "1", "0.25"],
                    ),
                ],
            ),
            "i3" => Instance::make(
                "i3",
                "C3's categories with I2's two machine types",
                208.0,
                c3(),
                chain(),
                vec![land_power_chain(), b_food()],
            ),
            "l2" => Instance::make(
                "l2",
                "Appendix B's categories, M4's engine and power with build recipes (a loop)",
                208.0,
                c1(),
                cycle(),
                vec![
                    land_engine(),
                    coef(
                        "land.power",
                        "inst.power.land",
                        "0.6",
                        ["0.66", "0.54", "1.2", "0.3"],
                    ),
                ],
            ),
            "l3" => Instance::make(
                "l3",
                "C3's categories with L2's two machine types (a loop)",
                208.0,
                c3(),
                cycle(),
                vec![land_engine(), b_food()],
            ),
            "g1" => Instance::make(
                "g1",
                "1b's gap economy at N 5, 1a's machine (optional)",
                260.0,
                gap(),
                mach(),
                vec![land_mach()],
            ),
            _ => return Err(format!("no instance {id}: i0, i1, i2, i3, l2, l3 or g1")),
        })
    }

    /// Every registered instance's id, in MARKETS-SPEC §1.1's order.
    pub const IDS: [&'static str; 7] = ["i0", "i1", "i2", "i3", "l2", "l3", "g1"];

    /// The number of segments of the task line.
    pub fn segments(&self) -> usize {
        self.edges.len() + 1
    }

    /// The task type: the one type with θ > 0.
    pub fn task_type(&self) -> Result<usize, String> {
        let tasks: Vec<usize> = (0..self.types.len())
            .filter(|&k| self.types[k].theta > 0.0)
            .collect();
        match tasks[..] {
            [k] => Ok(k),
            _ => Err(format!(
                "{}: {} task types; the roles take one (MARKETS-SPEC §2.4)",
                self.id,
                tasks.len()
            )),
        }
    }

    /// The markets, in the harness's order: labour, land, each type's service, each category's
    /// good. For I0 this is P2.0's order, [labour, land, mach, good].
    pub fn markets(&self) -> Vec<String> {
        let mut m = vec!["labour".to_string(), "land".to_string()];
        m.extend(self.types.iter().map(|t| t.key.clone()));
        m.extend(self.categories.iter().map(|c| c.key.clone()));
        m
    }

    /// The desks, in the harness's order: each category's, then each type's, by key.
    pub fn desks(&self) -> Vec<String> {
        let mut d: Vec<String> = self.categories.iter().map(|c| c.key.clone()).collect();
        d.extend(self.types.iter().map(|t| t.key.clone()));
        d
    }

    /// The actors, in the harness's order: `desk.<key>` for each desk, then the provider and
    /// the workers. For I0 this is P2.0's order.
    pub fn actors(&self) -> Vec<String> {
        let mut a: Vec<String> = self.desks().iter().map(|d| format!("desk.{d}")).collect();
        a.push("provider".into());
        a.push("workers".into());
        a
    }

    /// The instance's params, as the tape registers them: (key, value, unit, basis), in the
    /// tape's order. Only params some rule reads are listed (an unreferenced param does not
    /// load): θ for the task type alone, and a type's bought input only where it is positive.
    pub fn params(&self) -> Result<Vec<(String, f64, &'static str, String)>, String> {
        let tau = self.task_type()?;
        let mut out = Vec::new();
        let scal = SCALARS.to_string();
        out.push((
            "inst.workers".into(),
            self.workers,
            "FlowPerYear",
            scal.clone(),
        ));
        out.push(("inst.land".into(), self.land, "FlowPerYear", scal.clone()));
        for (k, v) in [
            ("inst.chi_max", self.chi_max),
            ("inst.eta", self.eta),
            ("inst.g0", self.g0),
            ("inst.g1", self.g1),
            ("inst.k", self.k),
        ] {
            out.push((k.into(), v, "Dimensionless", scal.clone()));
        }
        for (s, e) in self.edges.iter().enumerate() {
            out.push((
                format!("inst.edge.{}", s + 1),
                *e,
                "Dimensionless",
                self.category_basis.clone(),
            ));
        }
        for c in &self.categories {
            let b = self.category_basis.clone();
            out.push((
                format!("inst.{}.weight", c.key),
                c.weight,
                "Dimensionless",
                b.clone(),
            ));
            out.push((
                format!("inst.{}.land", c.key),
                c.land,
                "Dimensionless",
                b.clone(),
            ));
            for (s, mu) in c.density.iter().enumerate() {
                out.push((
                    format!("inst.{}.mu.{}", c.key, s + 1),
                    *mu,
                    "Dimensionless",
                    b.clone(),
                ));
            }
        }
        if let Some(h) = self.space {
            out.push((
                "inst.space.weight".into(),
                h,
                "Dimensionless",
                self.category_basis.clone(),
            ));
        }
        for (k, t) in self.types.iter().enumerate() {
            let b = self.type_basis.clone();
            if k == tau {
                out.push((
                    format!("inst.{}.theta", t.key),
                    t.theta,
                    "Dimensionless",
                    b.clone(),
                ));
            }
            out.push((
                format!("inst.{}.own", t.key),
                t.row[k],
                "Dimensionless",
                b.clone(),
            ));
            for (l, other) in self.types.iter().enumerate() {
                if l != k && t.row[l] > 0.0 {
                    out.push((
                        format!("inst.{}.in.{}", t.key, other.key),
                        t.row[l],
                        "Dimensionless",
                        b.clone(),
                    ));
                }
            }
            out.push((
                format!("inst.{}.labour", t.key),
                t.labour,
                "Dimensionless",
                b.clone(),
            ));
            out.push((format!("inst.{}.land", t.key), t.land, "Dimensionless", b));
        }
        Ok(out)
    }

    /// The value of the coefficient a param holds.
    pub fn get(&self, key: &str) -> Result<f64, String> {
        self.params()?
            .into_iter()
            .find(|(k, ..)| k == key)
            .map(|(_, v, ..)| v)
            .ok_or_else(|| format!("{}: no param {key}", self.id))
    }

    /// Set the coefficient a param holds (a cost shock's target, or any instance param).
    pub fn set(&mut self, key: &str, value: f64) -> Result<(), String> {
        self.get(key)?;
        let parts: Vec<&str> = key.split('.').collect();
        let bad = || format!("{}: {key} cannot be set", self.id);
        match parts[..] {
            ["inst", "workers"] => self.workers = value,
            ["inst", "land"] => self.land = value,
            ["inst", "chi_max"] => self.chi_max = value,
            ["inst", "eta"] => self.eta = value,
            ["inst", "g0"] => self.g0 = value,
            ["inst", "g1"] => self.g1 = value,
            ["inst", "k"] => self.k = value,
            ["inst", "edge", s] => {
                let s: usize = s.parse().map_err(|_| bad())?;
                *s.checked_sub(1)
                    .and_then(|i| self.edges.get_mut(i))
                    .ok_or_else(bad)? = value;
            }
            ["inst", "space", "weight"] => self.space = Some(value),
            ["inst", who, field] => {
                if let Some(c) = self.categories.iter_mut().find(|c| c.key == who) {
                    match field {
                        "weight" => c.weight = value,
                        "land" => c.land = value,
                        _ => return Err(bad()),
                    }
                } else {
                    let k = self
                        .types
                        .iter()
                        .position(|t| t.key == who)
                        .ok_or_else(bad)?;
                    let t = &mut self.types[k];
                    match field {
                        "theta" => t.theta = value,
                        "own" => t.row[k] = value,
                        "labour" => t.labour = value,
                        "land" => t.land = value,
                        _ => return Err(bad()),
                    }
                }
            }
            ["inst", who, "mu", s] => {
                let s: usize = s.parse().map_err(|_| bad())?;
                let c = self
                    .categories
                    .iter_mut()
                    .find(|c| c.key == who)
                    .ok_or_else(bad)?;
                *s.checked_sub(1)
                    .and_then(|i| c.density.get_mut(i))
                    .ok_or_else(bad)? = value;
            }
            ["inst", who, "in", other] => {
                let l = self
                    .types
                    .iter()
                    .position(|t| t.key == other)
                    .ok_or_else(bad)?;
                let t = self
                    .types
                    .iter_mut()
                    .find(|t| t.key == who)
                    .ok_or_else(bad)?;
                t.row[l] = value;
            }
            _ => return Err(bad()),
        }
        Ok(())
    }

    /// A registered cost-shock target by its run-grammar name, or by its param key.
    pub fn coef(&self, name: &str) -> Result<&Coef, String> {
        self.coefs
            .iter()
            .find(|c| c.name == name || c.param == name)
            .ok_or_else(|| {
                let names: Vec<&str> = self.coefs.iter().map(|c| c.name.as_str()).collect();
                format!(
                    "{}: no cost coefficient {name} ({})",
                    self.id,
                    names.join(", ")
                )
            })
    }

    /// The oracle's parameters at `tpy` ticks a year (MARKETS-SPEC §1.2, M9): N and T per
    /// tick, every type's flow recipe as its operating recipe with a zero build recipe, δ = 1,
    /// J = 1 and ρ = 0, and space as a category with no tasks and direct land 1, last.
    pub fn machine_params(&self, tpy: u32) -> Result<MachineParams, String> {
        let c = clock(tpy)?;
        let n_types = self.types.len();
        let mut categories: Vec<OCategory> = self
            .categories
            .iter()
            .map(|j| OCategory {
                weight: j.weight,
                direct_land: j.land,
                density: j.density.clone(),
            })
            .collect();
        if let Some(h) = self.space {
            categories.push(OCategory {
                weight: h,
                direct_land: 1.0,
                density: vec![0.0; self.segments()],
            });
        }
        let n_cats = categories.len();
        let mut edges = vec![0.0];
        edges.extend(self.edges.iter().copied());
        edges.push(1.0);
        let machine_types = self
            .types
            .iter()
            .map(|t| MachineType {
                task_efficiency: t.theta,
                operating: Recipe {
                    machines: t.row.clone(),
                    labor: t.labour,
                    land: t.land,
                },
                build: Recipe::zero(n_types),
                delta: 1.0,
                build_lag: 1,
            })
            .collect();
        Ok(MachineParams {
            workers: c.flow(FlowPerYear(self.workers)),
            land: c.flow(FlowPerYear(self.land)),
            schedule: PowerSchedule {
                eta: self.eta,
                g0: self.g0,
                g1: self.g1,
                k: self.k,
            },
            work_cost: UniformWorkCost {
                chi_max: self.chi_max,
            },
            rho: 0.0,
            machine_types,
            edges,
            categories,
            intermediate: vec![vec![0.0; n_cats]; n_cats],
        })
    }

    /// The oracle's interior equilibrium at `tpy` ticks a year, per tick, every price relative
    /// to r = 1 (unit 1c, which is 1b's and 1a's where they apply).
    pub fn point(&self, tpy: u32) -> Result<Point, String> {
        let e = oracle::MachineEconomy::new(self.machine_params(tpy)?)
            .map_err(|e| format!("{}: {e}", self.id))?;
        let q = match e.solve().map_err(|e| format!("{}: {e}", self.id))? {
            Regime::Interior(q) => q,
            other => {
                return Err(format!(
                    "{}: no interior equilibrium ({})",
                    self.id,
                    other.name()
                ))
            }
        };
        let c = self.categories.len();
        let n_types = self.types.len();
        let services: Vec<f64> = q.types.iter().map(|t| t.services).collect();
        // What each type's market clears: its task services sold to the category desks, and
        // what the other types buy of it; a type's own input is kept, not traded.
        let traded = (0..n_types)
            .map(|k| {
                let mut v = q.types[k].task_services;
                for (l, t) in self.types.iter().enumerate() {
                    if l != k {
                        v += t.row[k] * services[l];
                    }
                }
                v
            })
            .collect();
        Ok(Point {
            x_star: q.x_star,
            one_minus_x: q.one_minus_x_star,
            v: q.v,
            p_s: q.p_s,
            y: q.y,
            n_a: q.n_a,
            cat_price: q.categories[..c].iter().map(|j| j.price).collect(),
            cat_output: q.categories[..c].iter().map(|j| j.output).collect(),
            type_price: q.types.iter().map(|t| t.price).collect(),
            type_services: services,
            type_traded: traded,
            worker_baskets: q.worker_baskets,
            provider_baskets: q.provider_baskets,
            margin_active: q.margin_active,
            tie: q.tie.is_some(),
        })
    }
}

/// The oracle's equilibrium, as the harness reads it: per tick, relative to r = 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// x\*.
    pub x_star: f64,
    /// 1 − x\*, carried.
    pub one_minus_x: f64,
    /// v = w/r.
    pub v: f64,
    /// P_s.
    pub p_s: f64,
    /// Y, baskets.
    pub y: f64,
    /// N_a, hours worked.
    pub n_a: f64,
    /// p_j, each category with tasks.
    pub cat_price: Vec<f64>,
    /// z_j·Y, each category's output.
    pub cat_output: Vec<f64>,
    /// p_k, each type.
    pub type_price: Vec<f64>,
    /// X_k, each type's services made.
    pub type_services: Vec<f64>,
    /// What each type's market clears: X_k less its kept own input.
    pub type_traded: Vec<f64>,
    /// The workers' baskets, N + v·N_a/P_s.
    pub worker_baskets: f64,
    /// The provider's baskets, T/P_s − N.
    pub provider_baskets: f64,
    /// Whether some category has tasks on the segment holding x\* (false in a gap).
    pub margin_active: bool,
    /// Whether x\* sits at a tie between task types.
    pub tie: bool,
}
