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
//!
//! The open-commons instances (P2.3; the commons frame, docs/probe/commons/SPEC.md §2) are I1
//! with one priced worker type whose exit good is food and a commons the workers hold
//! ([`Commons`]); unit 1e's `ParcelEconomy` solves them, with the enclosed land T and the commons
//! T_o as two parcels of quality 1 (decision 399). The paced instances (P2.4; the trap's remedy,
//! decisions 404–406) C1P, C2P and C1PN are C1, C2 and C1N with the workers' participation at a
//! rate; the pace does not move the point, so their oracle is the same.

use crate::setup::clock;
use oracle::{
    Access, Category as OCategory, ExitForm, MachineParams, MachineType, Parcel, ParcelEconomy,
    ParcelParams, PowerSchedule, PricedExit, Recipe, Regime, UniformWorkCost, WorkerEconomy,
    WorkerParams, WorkerType,
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
    /// L^H_j, the pool's hours per unit at tasks closed to machines (unit 1d's human-required
    /// tail; the wall frame's §2.1); 0 for none, as in every P2.1 instance.
    pub tail: f64,
    /// R_ji, each reserved worker type's hours per unit, in [`Instance::wtypes`] order; empty,
    /// or 0 for a type, for none.
    pub reserved: Vec<f64>,
}

/// A reserved-only worker type (unit 1d with ε 0; the wall frame's §2.1, decision 394): its own
/// pop `workers.<key>` in class `<key>_workers`, selling only its reserved hours on its own
/// labour market `labour.<key>`, with params `inst.<key>.workers` and `inst.<key>.chi_max`.
/// Its support is one basket a head, paid by the provider's `more` transfer.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerKind {
    /// Its key.
    pub key: String,
    /// N_i, potential hours per year.
    pub workers: f64,
    /// χ_max,i.
    pub chi_max: f64,
}

impl WorkerKind {
    /// Its labour market's good.
    pub fn market(&self) -> String {
        format!("labour.{}", self.key)
    }

    /// Its pop's actor.
    pub fn pop(&self) -> String {
        format!("workers.{}", self.key)
    }
}

/// The priced exit with a commons (unit 1e; the commons frame's §2.1, decision 398): the pool's
/// workers are one priced type whose exit yields s(q) = max(s₀ − q·h, s̲) of the exit good a head
/// on a plot of h land service, and they hold a commons T_o they never trade. Params
/// `inst.exit.gross`, `inst.exit.floor`, `inst.exit.plot` and `inst.commons`.
#[derive(Debug, Clone, PartialEq)]
pub struct Commons {
    /// The exit good: a category's key.
    pub good: String,
    /// s₀, the exit good a head yields at home.
    pub gross: f64,
    /// s̲, the floor.
    pub floor: f64,
    /// h, the land service a plot takes.
    pub plot: f64,
    /// T_o, the commons, per year.
    pub commons: f64,
    /// Whether the workers' participation moves at a rate (P2.4; O100's remedy, decisions
    /// 404–406; docs/probe/TRAP-RULES.md): the exit's `pace`, at the dial
    /// `adjust.participation.workers`, its genesis share the point's S/N. False at C1, C2 and C1N.
    pub paced: bool,
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
    /// The basis of the scalars (N, T, χ_max, the schedule) and of the worker types.
    pub scalar_basis: String,
    /// The reserved-only worker types, in order, beside the pool's workers (the wall, P2.3).
    /// Empty in every P2.1 instance.
    pub wtypes: Vec<WorkerKind>,
    /// Whether the oracle is unit 1d's `WorkerEconomy` (the wall instances, P2.3: worker types,
    /// the tail and reserved hours) rather than unit 1c's `MachineEconomy` (P2.1's, whose
    /// genesis and pins stay 1c's). It also sets the harness's form: each desk's threshold
    /// x_j = 1 − s_j observed in place of s_j, and each reserved wage (decision 396).
    pub worker_form: bool,
    /// The priced exit with a commons (the open-commons instances, P2.3), solved by unit 1e's
    /// `ParcelEconomy`; `None` in every other instance.
    pub exit: Option<Commons>,
}

/// The scalars' basis, and 1a's machine's.
pub const SCALARS: &str = "SSRN 7226858 App. B via laborformal 31b3482";

fn cat(key: &str, weight: f64, land: f64, density: &[f64]) -> Category {
    Category {
        key: key.into(),
        weight,
        land,
        density: density.to_vec(),
        tail: 0.0,
        reserved: Vec::new(),
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
/// The wall instances' basis: the frame's own calibration, which the tape writes as `Assumed`.
pub const WALL_BASIS: &str =
    "docs/probe/wall/SPEC.md §2.1: unit-1d.md §3.3 B with E7's three worker types (frame-wall)";
/// The open-commons instances' basis for the exit and the commons: the frame's own
/// calibration, which the tape writes as `Assumed`.
pub const COMMONS_BASIS: &str =
    "docs/probe/commons/SPEC.md §2.1: I1 with one priced worker type in food and a commons (frame-commons)";

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
            scalar_basis: SCALARS.into(),
            wtypes: Vec::new(),
            worker_form: false,
            exit: None,
        }
    }

    /// An open-commons instance (the commons frame's §2.1; decision 398): I1 (C3's categories,
    /// 1a's machine, N 208 and T 520 a year) at χ_max `chi_max`, with the pool's workers one
    /// priced type in food, exit (s₀ `gross`, s̲ 0, h `plot`) and a commons of `commons` a year.
    /// Its cost coefficients are I1's land.mach and b.food and the commons, at the frame's
    /// values (`values`: ×1.1, ×0.9, ×2, ×0.5 as `battery_c.run_list` writes them).
    fn commons(
        id: &str,
        title: &str,
        chi_max: f64,
        (gross, plot, commons): (f64, f64, f64),
        (base, values): (&str, [&str; 4]),
    ) -> Instance {
        let mut i = Instance::make(
            id,
            title,
            208.0,
            c3(),
            mach(),
            vec![
                land_mach(),
                b_food(),
                coef("commons", "inst.commons", base, values),
            ],
        );
        i.chi_max = chi_max;
        if chi_max != 1.0 {
            i.scalar_basis = COMMONS_BASIS.into();
        }
        i.exit = Some(Commons {
            good: "food".into(),
            gross,
            floor: 0.0,
            plot,
            commons,
            paced: false,
        });
        i
    }

    /// A paced open-commons instance (P2.4; the trap scan's §5.4 and §6; decision 405): the
    /// commons instance `of` with the workers' participation at a rate, and its own id and title.
    /// Its oracle, targets, genesis prices and coins are `of`'s; only the workers' block and the
    /// dial `adjust.participation.workers` are added.
    fn paced(of: Instance, id: &str, title: &str) -> Instance {
        let mut i = of;
        i.id = id.into();
        i.title = title.into();
        if let Some(x) = i.exit.as_mut() {
            x.paced = true;
        }
        i
    }

    /// Whether the workers' participation moves at a rate (a paced commons instance, P2.4).
    pub fn paced_exit(&self) -> bool {
        self.exit.as_ref().is_some_and(|x| x.paced)
    }

    /// The wall frame's IW1 (docs/probe/wall/SPEC.md §2.1; decision 394), with the entrant's
    /// χ_max given: unit 1d's B economy (services with a human-required tail, goods, space 1,
    /// Appendix B's machine, η 0.5) with E7's three worker types, the trained and the master
    /// reserved-only. At χ_max 1 it is IW1, at 0.25 the line control IC1 (decision 397).
    fn wall(id: &str, title: &str, chi_max: f64) -> Instance {
        let services = Category {
            tail: 0.1,
            reserved: vec![0.04, 0.0],
            ..cat("services", 1.0, 0.0, &[0.75])
        };
        let goods = Category {
            reserved: vec![0.0, 0.03],
            ..cat("goods", 1.0, 0.0, &[1.0])
        };
        let wkind = |key: &str, workers: f64| WorkerKind {
            key: key.into(),
            workers,
            chi_max: 2.0,
        };
        Instance {
            id: id.into(),
            title: title.into(),
            workers: 130.0,
            land: 520.0,
            chi_max,
            eta: 0.5,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
            edges: Vec::new(),
            categories: vec![services, goods],
            space: Some(1.0),
            types: mach().0,
            coefs: vec![
                land_mach(),
                coef(
                    "tail.services",
                    "inst.services.tail",
                    "0.1",
                    ["0.11", "0.09", "0.2", "0.05"],
                ),
                coef(
                    "res.services.trained",
                    "inst.services.reserved.trained",
                    "0.04",
                    ["0.044", "0.036", "0.08", "0.02"],
                ),
            ],
            category_basis: WALL_BASIS.into(),
            type_basis: MACH_BASIS.into(),
            scalar_basis: WALL_BASIS.into(),
            wtypes: vec![wkind("trained", 52.0), wkind("master", 26.0)],
            worker_form: true,
            exit: None,
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
            "iw1" => Instance::wall(
                "iw1",
                "unit 1d's B economy with E7's three worker types: a solved wall",
                1.0,
            ),
            "ic1" => Instance::wall(
                "ic1",
                "IW1 with the entrant's chi_max 0.25: the task margin active, a control",
                0.25,
            ),
            "c1" => Instance::commons(
                "c1",
                "I1 with one priced worker type in food and a commons, full (Crowded)",
                1.0,
                (0.3, 0.135, 24.3),
                ("24.3", ["26.73", "21.87", "48.6", "12.15"]),
            ),
            "c2" => Instance::commons(
                "c2",
                "I1 with one priced worker type in food and a commons with room (Commons)",
                1.0,
                (0.2, 0.09, 31.2),
                ("31.2", ["34.32", "28.08", "62.4", "15.6"]),
            ),
            "c1n" => Instance::commons(
                "c1n",
                "C1 with chi_max 0.25: the negative control",
                0.25,
                (0.3, 0.135, 24.3),
                ("24.3", ["26.73", "21.87", "48.6", "12.15"]),
            ),
            // The trap's remedy (P2.4; decision 405): C1, C2 and C1N with the workers'
            // participation at a rate.
            "c1p" => Instance::paced(
                Instance::named("c1")?,
                "c1p",
                "C1 with the workers' participation at a rate (the trap's remedy)",
            ),
            "c2p" => Instance::paced(
                Instance::named("c2")?,
                "c2p",
                "C2 with the workers' participation at a rate (the trap's remedy)",
            ),
            "c1pn" => Instance::paced(
                Instance::named("c1n")?,
                "c1pn",
                "C1N with the workers' participation at a rate: a stress control",
            ),
            _ => {
                return Err(format!(
                    "no instance {id}: i0, i1, i2, i3, l2, l3, g1, iw1, ic1, c1, c2, c1n, c1p, \
                     c2p or c1pn"
                ))
            }
        })
    }

    /// Every registered instance of the markets probe (P2.1), in MARKETS-SPEC §1.1's order.
    pub const IDS: [&'static str; 7] = ["i0", "i1", "i2", "i3", "l2", "l3", "g1"];

    /// Phase 2 proper's wall instances (P2.3): IW1 and its line control IC1.
    pub const WALL_IDS: [&'static str; 2] = ["iw1", "ic1"];

    /// Phase 2 proper's open-commons instances (P2.3): C1, C2 and C1's negative control.
    pub const COMMONS_IDS: [&'static str; 3] = ["c1", "c2", "c1n"];

    /// Phase 2 proper's paced commons instances (P2.4, the trap's remedy): C1P, C2P and the
    /// stress control C1PN.
    pub const PACED_IDS: [&'static str; 3] = ["c1p", "c2p", "c1pn"];

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
    /// good, then each reserved worker type's labour. For I0 this is P2.0's order, [labour,
    /// land, mach, good].
    pub fn markets(&self) -> Vec<String> {
        let mut m = vec!["labour".to_string(), "land".to_string()];
        m.extend(self.types.iter().map(|t| t.key.clone()));
        m.extend(self.categories.iter().map(|c| c.key.clone()));
        m.extend(self.wtypes.iter().map(WorkerKind::market));
        m
    }

    /// The households, in the harness's order: the provider, the workers, then each reserved
    /// worker type's pop.
    pub fn households(&self) -> Vec<String> {
        let mut h = vec!["provider".to_string(), "workers".to_string()];
        h.extend(self.wtypes.iter().map(WorkerKind::pop));
        h
    }

    /// The households' classes, in [`Instance::households`] order.
    pub fn household_classes(&self) -> Vec<String> {
        let mut c = vec!["owners".to_string(), "workers".to_string()];
        c.extend(self.wtypes.iter().map(|t| format!("{}_workers", t.key)));
        c
    }

    /// The desks, in the harness's order: each category's, then each type's, by key.
    pub fn desks(&self) -> Vec<String> {
        let mut d: Vec<String> = self.categories.iter().map(|c| c.key.clone()).collect();
        d.extend(self.types.iter().map(|t| t.key.clone()));
        d
    }

    /// The actors, in the harness's order: `desk.<key>` for each desk, then the households
    /// ([`Instance::households`]). For I0 this is P2.0's order.
    pub fn actors(&self) -> Vec<String> {
        let mut a: Vec<String> = self.desks().iter().map(|d| format!("desk.{d}")).collect();
        a.extend(self.households());
        a
    }

    /// The instance's params, as the tape registers them: (key, value, unit, basis), in the
    /// tape's order. Only params some rule reads are listed (an unreferenced param does not
    /// load): θ for the task type alone, and a type's bought input only where it is positive.
    pub fn params(&self) -> Result<Vec<(String, f64, &'static str, String)>, String> {
        let tau = self.task_type()?;
        let mut out = Vec::new();
        let scal = self.scalar_basis.clone();
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
        // The priced exit and the commons (the commons frame's §2.1).
        if let Some(x) = &self.exit {
            let b = COMMONS_BASIS.to_string();
            for (k, v) in [
                ("inst.exit.gross", x.gross),
                ("inst.exit.floor", x.floor),
                ("inst.exit.plot", x.plot),
            ] {
                out.push((k.into(), v, "Dimensionless", b.clone()));
            }
            out.push(("inst.commons".into(), x.commons, "FlowPerYear", b));
        }
        for t in &self.wtypes {
            out.push((
                format!("inst.{}.workers", t.key),
                t.workers,
                "FlowPerYear",
                scal.clone(),
            ));
            out.push((
                format!("inst.{}.chi_max", t.key),
                t.chi_max,
                "Dimensionless",
                scal.clone(),
            ));
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
            if c.tail > 0.0 {
                out.push((
                    format!("inst.{}.tail", c.key),
                    c.tail,
                    "Dimensionless",
                    b.clone(),
                ));
            }
            for (t, r) in self.wtypes.iter().zip(&c.reserved) {
                if *r > 0.0 {
                    out.push((
                        format!("inst.{}.reserved.{}", c.key, t.key),
                        *r,
                        "Dimensionless",
                        b.clone(),
                    ));
                }
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
            ["inst", "commons"] => self.exit.as_mut().ok_or_else(bad)?.commons = value,
            ["inst", "exit", field] => {
                let x = self.exit.as_mut().ok_or_else(bad)?;
                match field {
                    "gross" => x.gross = value,
                    "floor" => x.floor = value,
                    "plot" => x.plot = value,
                    _ => return Err(bad()),
                }
            }
            ["inst", who, field] => {
                if let Some(c) = self.categories.iter_mut().find(|c| c.key == who) {
                    match field {
                        "weight" => c.weight = value,
                        "land" => c.land = value,
                        "tail" => c.tail = value,
                        _ => return Err(bad()),
                    }
                } else if let Some(t) = self.wtypes.iter_mut().find(|t| t.key == who) {
                    match field {
                        "workers" => t.workers = value,
                        "chi_max" => t.chi_max = value,
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
            ["inst", who, "reserved", other] => {
                let i = self
                    .wtypes
                    .iter()
                    .position(|t| t.key == other)
                    .ok_or_else(bad)?;
                let c = self
                    .categories
                    .iter_mut()
                    .find(|c| c.key == who)
                    .ok_or_else(bad)?;
                *c.reserved.get_mut(i).ok_or_else(bad)? = value;
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

    /// Unit 1d's parameters at `tpy` ticks a year (the wall frame's §3.6), for a worker-form
    /// instance: [`Instance::machine_params`]'s categories, space, machine types and schedule,
    /// with the worker types in pop order, the pool's workers first (N, χ_max, ε 1, ν 1) and
    /// each reserved-only type after them (N_i, χ_max,i, ε 0, ν 1); the tail as each category's
    /// human-required hours; and each category's reserved hours in the type's column, the
    /// pool's column and space's row zero.
    pub fn worker_params(&self, tpy: u32) -> Result<WorkerParams, String> {
        let c = clock(tpy)?;
        let mut w = WorkerParams::from_machines(self.machine_params(tpy)?);
        let kind = |workers: f64, chi_max: f64, efficiency: f64| WorkerType {
            workers: c.flow(FlowPerYear(workers)),
            work_cost: UniformWorkCost { chi_max },
            efficiency,
            support: 1.0,
        };
        w.worker_types = vec![kind(self.workers, self.chi_max, 1.0)];
        w.worker_types
            .extend(self.wtypes.iter().map(|t| kind(t.workers, t.chi_max, 0.0)));
        let n_cats = w.categories.len();
        let n_workers = w.worker_types.len();
        w.human_required = vec![0.0; n_cats];
        w.reserved = vec![vec![0.0; n_workers]; n_cats];
        for (j, cat) in self.categories.iter().enumerate() {
            w.human_required[j] = cat.tail;
            for (i, r) in cat.reserved.iter().enumerate() {
                if i + 1 >= n_workers {
                    return Err(format!(
                        "{}: {} has reserved hours for {} worker types, and there are {}",
                        self.id,
                        cat.key,
                        cat.reserved.len(),
                        self.wtypes.len()
                    ));
                }
                w.reserved[j][i + 1] = *r;
            }
        }
        Ok(w)
    }

    /// The oracle's interior equilibrium at `tpy` ticks a year, per tick, every price relative
    /// to r = 1: unit 1c's (which is 1b's and 1a's where they apply), or unit 1d's
    /// `WorkerEconomy::solve` for a worker-form instance.
    pub fn point(&self, tpy: u32) -> Result<Point, String> {
        if self.worker_form {
            return self.worker_point(tpy);
        }
        if self.exit.is_some() {
            return self.parcel_point(tpy);
        }
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
        let services: Vec<f64> = q.types.iter().map(|t| t.services).collect();
        let traded = self.traded(&q.types, &services);
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
            pool: q.n_a,
            wage: Vec::new(),
            hours: vec![q.n_a],
            pop_baskets: vec![q.worker_baskets],
            margin: String::new(),
            commons: None,
        })
    }

    /// What each type's market clears: its task services sold to the category desks, and what
    /// the other types buy of it; a type's own input is kept, not traded.
    fn traded(&self, types: &[oracle::TypeEq], services: &[f64]) -> Vec<f64> {
        (0..types.len())
            .map(|k| {
                let mut v = types[k].task_services;
                for (l, t) in self.types.iter().enumerate() {
                    if l != k {
                        v += t.row[k] * services[l];
                    }
                }
                v
            })
            .collect()
    }

    /// Unit 1d's interior equilibrium (the wall frame's §3.6).
    fn worker_point(&self, tpy: u32) -> Result<Point, String> {
        let c = clock(tpy)?;
        let e = WorkerEconomy::new(self.worker_params(tpy)?)
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
        let nc = self.categories.len();
        let services: Vec<f64> = q.types.iter().map(|t| t.services).collect();
        let traded = self.traded(&q.types, &services);
        // Each worker pop's baskets: its support, N_i·ν_i with ν_i 1, and its wage bill over P_s.
        let heads: Vec<f64> = std::iter::once(self.workers)
            .chain(self.wtypes.iter().map(|t| t.workers))
            .map(|n| c.flow(FlowPerYear(n)))
            .collect();
        let pop_baskets = q
            .workers
            .iter()
            .zip(&heads)
            .map(|(w, n)| n + w.wage * w.hours / q.p_s)
            .collect();
        Ok(Point {
            x_star: q.x_star,
            one_minus_x: q.one_minus_x_star,
            v: q.v,
            p_s: q.p_s,
            y: q.y,
            n_a: q.n_a,
            cat_price: q.categories[..nc].iter().map(|j| j.price).collect(),
            cat_output: q.categories[..nc].iter().map(|j| j.output).collect(),
            type_price: q.types.iter().map(|t| t.price).collect(),
            type_services: services,
            type_traded: traded,
            worker_baskets: q.worker_baskets,
            provider_baskets: q.provider_baskets,
            margin_active: q.margin_active,
            tie: q.tie.is_some(),
            pool: q.n_pool,
            wage: q.workers[1..].iter().map(|w| w.wage).collect(),
            hours: q.workers.iter().map(|w| w.hours).collect(),
            pop_baskets,
            margin: format!("{:?}", q.margin),
            commons: None,
        })
    }

    /// Unit 1e's parameters at `tpy` ticks a year (the commons frame's §3.8; decision 399), for
    /// an open-commons instance: [`Instance::machine_params`] in 1d's form with one worker type
    /// (N, χ_max, ε 1, ν 1), the enclosed land T and the commons T_o as two parcels of quality 1
    /// (per tick), the one type's exit `ExitForm::Priced` (s₀, s̲, h), and the exit good the
    /// category the tape names.
    pub fn parcel_params(&self, tpy: u32) -> Result<ParcelParams, String> {
        let c = clock(tpy)?;
        let x = self
            .exit
            .as_ref()
            .ok_or_else(|| format!("{}: no exit", self.id))?;
        let g = self
            .categories
            .iter()
            .position(|j| j.key == x.good)
            .ok_or_else(|| format!("{}: the exit good {} is no category", self.id, x.good))?;
        let mut p =
            ParcelParams::from_workers(WorkerParams::from_machines(self.machine_params(tpy)?));
        p.parcels = vec![Parcel {
            acreage: c.flow(FlowPerYear(self.land)),
            quality: 1.0,
            access: Access::Enclosed,
        }];
        // A commons of 0 (all of it enclosed by law) is no parcel: unit 1e takes acreage at its
        // scale floor or above, and the plots then all stand on enclosed land.
        if x.commons > 0.0 {
            p.parcels.push(Parcel {
                acreage: c.flow(FlowPerYear(x.commons)),
                quality: 1.0,
                access: Access::Open,
            });
        }
        p.exits = vec![ExitForm::Priced(PricedExit {
            gross: x.gross,
            floor: x.floor,
            plot: x.plot,
        })];
        p.exit_good = g;
        Ok(p)
    }

    /// Unit 1e's interior equilibrium (the commons frame's §2.3, §3.8), relative to r = 1:
    /// labour clears the supply S at the point, and each household's baskets are its money
    /// accounts, the provider's (T_m + T_p)/P_s − N and the workers' N + (w·S − T_p)/P_s, which
    /// sum to Y (the plots pay rent in money, O102).
    fn parcel_point(&self, tpy: u32) -> Result<Point, String> {
        let e = ParcelEconomy::new(self.parcel_params(tpy)?)
            .map_err(|e| format!("{}: {e}", self.id))?;
        let certified = e.certified();
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
        let b = &q.base;
        let nc = self.categories.len();
        let services: Vec<f64> = b.types.iter().map(|t| t.services).collect();
        let traded = self.traded(&b.types, &services);
        let supply = b.workers[0].supply;
        let rented = q.land.rented_plots;
        let workers = b.worker_baskets - rented / b.p_s;
        Ok(Point {
            x_star: b.x_star,
            one_minus_x: b.one_minus_x_star,
            v: b.v,
            p_s: b.p_s,
            y: b.y,
            n_a: b.n_a,
            cat_price: b.categories[..nc].iter().map(|j| j.price).collect(),
            cat_output: b.categories[..nc].iter().map(|j| j.output).collect(),
            type_price: b.types.iter().map(|t| t.price).collect(),
            type_services: services,
            type_traded: traded,
            worker_baskets: workers,
            provider_baskets: b.provider_baskets + rented / b.p_s,
            margin_active: b.margin_active,
            tie: b.tie.is_some() || q.enclosure.is_some(),
            pool: supply,
            wage: Vec::new(),
            hours: vec![supply],
            pop_baskets: vec![workers],
            margin: format!("{:?}", b.margin),
            commons: Some(CommonsPoint {
                regime: format!("{:?}", q.exit_land),
                land_market: format!("{:?}", q.land_market),
                plot_rent: q.plot_rent,
                rented,
                occupied: q.land.commons_occupied,
                exit_value: q.workers[0].exit_value,
                funded: b.funded,
                certified,
            }),
        })
    }
}

/// Unit 1e's readouts at an open-commons instance's point (the commons frame's §2.3).
#[derive(Debug, Clone, PartialEq)]
pub struct CommonsPoint {
    /// Where the plots stand, `ExitLand` by name: `Commons`, `Crowded`, `Enclosed`, ...
    pub regime: String,
    /// The land market, `LandMarket` by name: `Scarce` (r = 1) or `Idle`.
    pub land_market: String,
    /// r_o, the rent a plot pays: 0 on a commons with room, the shadow rent when crowded, r when
    /// the plots spill onto enclosed land.
    pub plot_rent: f64,
    /// T_p, the enclosed land in plots.
    pub rented: f64,
    /// T_oc, the commons occupied.
    pub occupied: f64,
    /// e, the exit life's money value.
    pub exit_value: f64,
    /// Whether the provider's baskets are positive in the oracle's accounts.
    pub funded: bool,
    /// Whether unit-1e.md §5.4 certifies the count.
    pub certified: bool,
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
    /// The pool's hours, which the `labour` market clears: n_D in unit 1d, N_a in 1c.
    pub pool: f64,
    /// Each reserved worker type's wage v_i, in [`Instance::wtypes`] order (empty in 1c).
    pub wage: Vec<f64>,
    /// Hours worked by each worker pop, the pool's workers first (unit 1d's
    /// `WorkerEq::hours`; N_a alone in 1c).
    pub hours: Vec<f64>,
    /// Each worker pop's baskets, its support plus its wage bill over P_s, the pool's workers
    /// first (the workers' baskets alone in 1c).
    pub pop_baskets: Vec<f64>,
    /// Unit 1d's margin (`Wall`, `Line`, …), empty in 1c.
    pub margin: String,
    /// Unit 1e's readouts at an open-commons instance; `None` elsewhere.
    pub commons: Option<CommonsPoint>,
}
