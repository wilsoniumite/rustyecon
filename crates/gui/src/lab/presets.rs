//! The lab's presets: instances the oracle's goldens were computed on, two or more per unit,
//! each named by its goldens file and its prefix there, so the lab shows the oracle's outputs
//! beside the generator's numbers for the same economy.
//!
//! Each is built as the oracle's gate tests build it (`crates/oracle/tests/gate/support*.rs`),
//! through the oracle's own conversions (`CategoryParams::from_one_category`, …) where the
//! test does, and with its numbers as the unit's spec and the goldens file's heading give them.
//! `lab_presets_solve_to_their_goldens` solves every preset and holds every paired output to its
//! golden, so a preset that is not its golden's economy fails there.

use super::{Instance, OracleUnit};
use oracle::{
    Access, Basket, Budget, Category, CategoryParams, ExitForm, Government, HouseholdParams,
    MachineParams, MachineType, Params, Parcel, ParcelParams, PowerSchedule, PricedExit, Recipe,
    UniformWorkCost, WorkerParams, WorkerType,
};
use rustyecon_engine::num;

/// A preset.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    /// Its name in the lab: the goldens' prefix.
    pub id: &'static str,
    /// Its unit.
    pub unit: OracleUnit,
    /// The goldens file its numbers are in.
    pub file: &'static str,
    /// The prefix of its keys there.
    pub prefix: &'static str,
    /// What it is, in a line.
    pub title: &'static str,
    /// Its parameters.
    pub build: fn() -> Instance,
}

impl Preset {
    /// Its instance.
    pub fn instance(&self) -> Instance {
        (self.build)()
    }
}

/// The SSRN Appendix B instance (SSRN p.30): N 4, T 10, h 1, a 0.3, λ 0.05, b 0.4,
/// γ = 0.2 + 0.8x, χ ~ U[0, 1], (ρ, δ, J_b) = (0, 1, 1).
pub fn appendix_b() -> Params {
    Params {
        workers: 4.0,
        land: 10.0,
        space: 1.0,
        a: 0.3,
        lam: 0.05,
        b: 0.4,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
        },
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    }
}

/// γ = η(g0 + g1·x).
fn linear(eta: f64, g0: f64, g1: f64) -> PowerSchedule {
    PowerSchedule {
        eta,
        g0,
        g1,
        k: 1.0,
    }
}

fn g5_flow() -> Params {
    Params {
        workers: 5.2,
        land: 12.5,
        space: 0.7,
        a: 0.22,
        lam: 0.08,
        b: 0.55,
        schedule: PowerSchedule {
            eta: 2.3,
            g0: 0.15,
            g1: 0.9,
            k: 4.5,
        },
        work_cost: UniformWorkCost { chi_max: 1.6 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    }
}

fn category(weight: f64, direct_land: f64, density: &[f64]) -> Category {
    Category {
        weight,
        direct_land,
        density: density.to_vec(),
    }
}

/// G1's scalars on the given task line and categories (unit 1b's `g1_scalars`).
fn g1_scalars(edges: &[f64], categories: Vec<Category>) -> CategoryParams {
    let a = appendix_b();
    CategoryParams {
        edges: edges.to_vec(),
        categories,
        ..CategoryParams::from_one_category(a)
    }
}

/// 1b's fork economy: manufactures, food, care and shelter on edges (0, 0.4, 0.75, 1).
fn fork_economy() -> CategoryParams {
    g1_scalars(
        &[0.0, 0.4, 0.75, 1.0],
        vec![
            category(0.3, 0.0, &[2.0, 0.0, 0.0]),
            category(1.0, 0.6, &[0.5, 1.5, 0.0]),
            category(0.2, 0.1, &[0.0, 0.0, 1.0]),
            category(0.8, 1.0, &[0.0, 0.4, 0.0]),
        ],
    )
}

fn recipe(machines: &[f64], labor: f64, land: f64) -> Recipe {
    Recipe {
        machines: machines.to_vec(),
        labor,
        land,
    }
}

fn machine_type(
    task_efficiency: f64,
    operating: Recipe,
    build: Recipe,
    delta: f64,
    build_lag: u32,
) -> MachineType {
    MachineType {
        task_efficiency,
        operating,
        build,
        delta,
        build_lag,
    }
}

/// G1's household with the good and space on one segment, on the given schedule, ρ, h and
/// machine types (unit 1c's `appendix_b_household`).
fn appendix_b_household(
    schedule: PowerSchedule,
    rho: f64,
    space: f64,
    machine_types: Vec<MachineType>,
) -> MachineParams {
    MachineParams {
        schedule,
        rho,
        machine_types,
        categories: vec![category(1.0, 0.0, &[1.0]), category(space, 1.0, &[0.0])],
        intermediate: vec![vec![0.0; 2]; 2],
        edges: vec![0.0, 1.0],
        ..MachineParams::from_categories(CategoryParams::from_one_category(appendix_b()))
    }
}

/// M4: 1b's fork economy with the loom, the engine and power, intermediate inputs, ρ 0.04.
fn m4(eta: f64) -> MachineParams {
    let fork = fork_economy();
    MachineParams {
        schedule: linear(eta, 0.2, 0.8),
        rho: 0.04,
        machine_types: vec![
            machine_type(
                1.0,
                recipe(&[0.0, 0.0, 0.05], 0.3, 0.05),
                recipe(&[0.1, 0.0, 0.0], 1.0, 0.3),
                0.1,
                2,
            ),
            machine_type(
                2.0,
                recipe(&[0.0, 0.0, 0.5], 0.02, 0.0),
                recipe(&[0.0, 0.1, 0.0], 0.1, 0.4),
                0.05,
                3,
            ),
            machine_type(
                0.0,
                recipe(&[0.0, 0.0, 0.0], 0.02, 0.5),
                recipe(&[0.0, 0.1, 0.0], 0.1, 0.1),
                0.05,
                3,
            ),
        ],
        edges: fork.edges.clone(),
        categories: fork.categories.clone(),
        intermediate: vec![
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.1, 0.0, 0.0, 0.0],
            vec![0.0, 0.2, 0.0, 0.0],
            vec![0.15, 0.0, 0.0, 0.0],
        ],
        ..MachineParams::from_categories(fork)
    }
}

fn worker(workers: f64, chi_max: f64, efficiency: f64, support: f64) -> WorkerType {
    WorkerType {
        workers,
        work_cost: UniformWorkCost { chi_max },
        efficiency,
        support,
    }
}

/// Unit 1d's B: services (z 1, μ 0.75, L^H `required`), goods and space on one segment,
/// Appendix B's machine, T 10, γ = η(0.2 + 0.8x), ρ 0, with the worker types and reserved
/// hours given.
fn baumol(
    eta: f64,
    worker_types: Vec<WorkerType>,
    required: f64,
    reserved: Vec<Vec<f64>>,
) -> WorkerParams {
    WorkerParams {
        land: 10.0,
        schedule: linear(eta, 0.2, 0.8),
        rho: 0.0,
        machine_types: vec![machine_type(
            1.0,
            recipe(&[0.0], 0.0, 0.0),
            recipe(&[0.3], 0.05, 0.4),
            1.0,
            1,
        )],
        edges: vec![0.0, 1.0],
        categories: vec![
            category(1.0, 0.0, &[0.75]),
            category(1.0, 0.0, &[1.0]),
            category(1.0, 1.0, &[0.0]),
        ],
        intermediate: vec![vec![0.0; 3]; 3],
        worker_types,
        human_required: vec![required, 0.0, 0.0],
        reserved,
    }
}

/// G1 in parcel form with N, T, η, λ and χ_max changed (unit 1e's `goodspace_1e`).
fn goodspace(workers: f64, land: f64, eta: f64, lam: f64, chi_max: f64) -> ParcelParams {
    ParcelParams::from_workers(WorkerParams::from_machines(MachineParams::from_categories(
        CategoryParams::from_one_category(Params {
            workers,
            land,
            lam,
            schedule: linear(eta, 0.2, 0.8),
            work_cost: UniformWorkCost { chi_max },
            ..appendix_b()
        }),
    )))
}

fn priced(gross: f64, floor: f64, plot: f64) -> ExitForm {
    ExitForm::Priced(PricedExit { gross, floor, plot })
}

fn parcel(acreage: f64, quality: f64, access: Access) -> Parcel {
    Parcel {
        acreage,
        quality,
        access,
    }
}

fn household(economy: ParcelParams, government: Government) -> HouseholdParams {
    HouseholdParams {
        economy,
        basket: Basket::Fixed,
        government,
    }
}

/// Unit 1f's TX: N 4, T 8, h 1, a 0.5, λ 0.1, b 0.2, γ = 2 + 2x, χ_max 1/8, ν 1/4.
fn tx_parcels() -> ParcelParams {
    let mut p = ParcelParams::from_workers(WorkerParams::from_machines(
        MachineParams::from_categories(CategoryParams::from_one_category(Params {
            workers: 4.0,
            land: 8.0,
            a: 0.5,
            lam: 0.1,
            b: 0.2,
            schedule: linear(1.0, 2.0, 2.0),
            work_cost: UniformWorkCost { chi_max: 0.125 },
            ..appendix_b()
        })),
    ));
    p.worker_types[0].support = 0.25;
    p
}

/// Every preset, by unit.
pub fn all() -> Vec<Preset> {
    use OracleUnit::{U1a, U1b, U1c, U1d, U1e, U1f};
    vec![
        Preset {
            id: "G1",
            unit: U1a,
            file: "goldens.txt",
            prefix: "G1",
            title: "SSRN Appendix B (p.30): the paper's instance",
            build: || Instance::A(appendix_b()),
        },
        Preset {
            id: "G3 η 0.3",
            unit: U1a,
            file: "goldens.txt",
            prefix: "G3_ETA_0_3",
            title: "the automation path at η 0.3: λ 0, γ = η(1 + x)",
            build: || {
                Instance::A(Params {
                    lam: 0.0,
                    schedule: linear(1.0, 0.3, 0.3),
                    ..appendix_b()
                })
            },
        },
        Preset {
            id: "G5 flow",
            unit: U1a,
            file: "goldens.txt",
            prefix: "G5_FLOW",
            title: "a general flow instance, every parameter off the paper's",
            build: || Instance::A(g5_flow()),
        },
        Preset {
            id: "G5 durable",
            unit: U1a,
            file: "goldens.txt",
            prefix: "G5_DURABLE",
            title: "the flow instance with k 2.5, ρ 0.04, δ 0.35 and J_b 2",
            build: || {
                let base = g5_flow();
                Instance::A(Params {
                    schedule: PowerSchedule {
                        k: 2.5,
                        ..base.schedule
                    },
                    rho: 0.04,
                    delta: 0.35,
                    build_lag: 2,
                    ..base
                })
            },
        },
        Preset {
            id: "C3",
            unit: U1b,
            file: "goldens_1b.txt",
            prefix: "C3",
            title: "the fork economy: manufactures, food, care and shelter",
            build: || Instance::B(fork_economy()),
        },
        Preset {
            id: "C7 gap",
            unit: U1b,
            file: "goldens_1b.txt",
            prefix: "C7_GAP",
            title: "the gap economy: C3 with N 5, a middle segment no category uses",
            build: || {
                Instance::B(CategoryParams {
                    workers: 5.0,
                    ..g1_scalars(
                        &[0.0, 0.4, 0.6, 1.0],
                        vec![
                            category(0.3, 0.0, &[2.0, 0.0, 0.0]),
                            category(1.0, 0.6, &[0.5, 0.0, 0.2]),
                            category(0.2, 0.1, &[0.0, 0.0, 0.3]),
                            category(0.8, 1.0, &[0.0, 0.0, 0.1]),
                        ],
                    )
                })
            },
        },
        Preset {
            id: "M3",
            unit: U1c,
            file: "goldens_1c.txt",
            prefix: "M3",
            title: "check_dynamics' machine in Appendix B's closure, γ = 1 + 2x, ρ 0.05",
            build: || {
                Instance::C(appendix_b_household(
                    linear(1.0, 1.0, 2.0),
                    0.05,
                    1.0,
                    vec![machine_type(
                        1.0,
                        recipe(&[0.5], 0.1, 0.2),
                        recipe(&[0.1], 0.2, 0.02),
                        0.1,
                        3,
                    )],
                ))
            },
        },
        Preset {
            id: "M4",
            unit: U1c,
            file: "goldens_1c.txt",
            prefix: "M4",
            title: "the three-type fork economy (loom, engine, power), η 1, ρ 0.04",
            build: || Instance::C(m4(1.0)),
        },
        Preset {
            id: "B1",
            unit: U1d,
            file: "goldens_1d.txt",
            prefix: "B1",
            title: "the human-required economy, N 8, η 1: a contestable margin with the tail",
            build: || {
                Instance::D(baumol(
                    1.0,
                    vec![worker(8.0, 1.0, 1.0, 1.0)],
                    0.25,
                    vec![vec![0.0]; 3],
                ))
            },
        },
        Preset {
            id: "E1",
            unit: U1d,
            file: "goldens_1d.txt",
            prefix: "E1",
            title: "the entrant (N 8) and the trained (N 3), both pooled",
            build: || {
                Instance::D(baumol(
                    1.0,
                    vec![worker(8.0, 1.0, 1.0, 1.0), worker(3.0, 0.8, 1.5, 1.2)],
                    0.25,
                    vec![vec![0.0, 0.1], vec![0.0, 0.02], vec![0.0, 0.0]],
                ))
            },
        },
        Preset {
            id: "K1",
            unit: U1e,
            file: "goldens_1e.txt",
            prefix: "K1",
            title: "the commons: G1's economy with a waste of 1 open and exit (0.5, 0, 0.1)",
            build: || {
                Instance::E(ParcelParams {
                    parcels: vec![
                        parcel(10.0, 1.0, Access::Enclosed),
                        parcel(1.0, 1.0, Access::Open),
                    ],
                    exits: vec![priced(0.5, 0.0, 0.1)],
                    ..goodspace(4.0, 10.0, 1.0, 0.05, 1.0)
                })
            },
        },
        Preset {
            id: "Q1",
            unit: U1e,
            file: "goldens_1e.txt",
            prefix: "Q1",
            title: "the race: T 100, N 50, η 2.5, exit (1.5, 0, 1)",
            build: || {
                Instance::E(ParcelParams {
                    exits: vec![priced(1.5, 0.0, 1.0)],
                    ..goodspace(50.0, 100.0, 2.5, 0.05, 1.0)
                })
            },
        },
        Preset {
            id: "TX",
            unit: U1f,
            file: "goldens_1f.txt",
            prefix: "TX",
            title: "three-taxes' worked instance inside the closure, no government",
            build: || Instance::F(household(tx_parcels(), Government::none())),
        },
        Preset {
            id: "GB",
            unit: U1f,
            file: "goldens_1f.txt",
            prefix: "GB",
            title: "Appendix B with a basic income: the owners' levy, d̂ 1",
            build: || {
                let mut g = Government::none();
                g.budget = Budget::RentRate { dividend: 1.0 };
                Instance::F(household(goodspace(4.0, 10.0, 1.0, 0.05, 1.0), g))
            },
        },
        Preset {
            id: "C1",
            unit: U1f,
            file: "goldens_1f.txt",
            prefix: "C1",
            title: "the CES household on G1's economy, α 0.3, σ 0.5 (SSRN eq 26)",
            build: || {
                let (alpha, sigma) = (0.3, 0.5);
                let mut p = goodspace(4.0, 10.0, 1.0, 0.05, 1.0);
                p.categories[0].weight = num::pow(1.0 - alpha, sigma);
                p.categories[1].weight = num::pow(alpha, sigma);
                Instance::F(HouseholdParams {
                    economy: p,
                    basket: Basket::Ces { sigma },
                    government: Government::none(),
                })
            },
        },
    ]
}

/// The presets of one unit, in order.
pub fn of(unit: OracleUnit) -> Vec<Preset> {
    all().into_iter().filter(|p| p.unit == unit).collect()
}
