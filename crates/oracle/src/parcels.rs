//! Unit 1e: parcels, the idle margin and the priced exit (docs/unit-1e.md).
//!
//! Unit 1d's economy with its land cut into parcels and the exit life priced. A parcel has an
//! acreage, a quality (land service per acre) and an access: enclosed parcels are rented on the
//! market at r per unit of service, open ones are a commons, free to exit households and
//! closed to production (§2.1-2.2). Each worker type names its exit form: the dependence form
//! of SSRN eq 9 (units 1a-1d), or main.tex's priced form s(q) = max(s₀ − q·h, s̲), in units of
//! one exit good, both under SSRN eq 8's participation rule with the exit life's value added
//! (§2.3). An exit plot takes h units of land service: on the commons while it has room, at a
//! shadow rent when it is full, and rented on enclosed land, which it takes from production,
//! when it spills (§2.6-2.7, §4.4). Idle enclosed land earns zero rent: the path continues past
//! the wall's end onto a fourth stretch at r = 0, with the pool's wage as numeraire, where every
//! economy unit 1d called `LaborShort` has its equilibrium (§2.8, §4.6).
//!
//! The evaluation order is normative (§5.1): with every type in the dependence form (or with
//! its exit option switched off, s₀ = s̲ = 0), every new operation is an exact no-op, and the
//! solve repeats unit 1d's floating-point operations bit for bit wherever 1d has an equilibrium
//! (§2.12).

use rustyecon_core::num;

use crate::categories::{Category, Output1b};
use crate::exit::PricedExit;
use crate::households::{Ces, Closure, Rule};
use crate::machine_block::MachineType;
use crate::machines::{Item, OutputKey1c, Tie};
use crate::params::{self, ParamError, SCALE_CEIL, SCALE_FLOOR};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{bisect, bisect_bits, labor_net, Regime, Root, SolveError};
use crate::solve::{BRACKET_HI, BRACKET_LO};
use crate::workers::{
    Context, Edge, Eq1d, Kind, Margin, Market, Path, Reported, WorkerEconomy, WorkerParams,
    WorkerPoint, WorkerType,
};

/// Interior points at which the solve reads the excess demand in every piece of the path, in
/// an economy with a type that can take a plot (docs/unit-1e.md §2.11 and §5.3).
///
/// Under the priced exit a plot-taking type's supply can fall as the wage rises (Lemma 5, §5.4:
/// where its plot, at the rent it pays, is more land-intensive than the market's version of its
/// exit goods), so the excess demand can rise within a piece, and two equilibria can lie
/// between two values of the sequence whose sides agree (M has three, two of them 0.36 apart on
/// one region of the line). The scan finds them when they are more than one cell apart. Where
/// §5.4's certification holds ([`ParcelEconomy::certified`]) the excess demand is nonincreasing
/// and the count is exact; elsewhere a pair of equilibria inside one cell of width 1/257 of its
/// piece is missed. The root itself is found by bisection on the piece's own ends, so its bits
/// do not depend on this number. 256 is the draft's: e8 measures the count against a scan 16
/// times finer.
pub const EXIT_SCAN: usize = 256;

/// A parcel's access (docs/unit-1e.md §2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// Rented on the market at r per unit of land service.
    Enclosed,
    /// A commons: free to exit plots, closed to production.
    Open,
}

/// One parcel (docs/unit-1e.md §2.1 and §3.1): A_z acres of quality Q_z, which supply A_z·Q_z
/// units of land service per period.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parcel {
    /// A_z, acres. Scale.
    pub acreage: f64,
    /// Q_z, land service per acre per period. 0 (the truly idle margin) or scale.
    pub quality: f64,
    /// Enclosed or open.
    pub access: Access,
}

/// A worker type's exit form (docs/unit-1e.md §2.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExitForm {
    /// SSRN eq 9: exit is dependence on the support basket, e_i = 0 (units 1a-1d).
    Dependence,
    /// main.tex's s(q), in units of the exit good: e_i = p_g·s(q_o).
    Priced(PricedExit),
}

/// The parameters of unit 1e (docs/unit-1e.md §3.1): unit 1d's [`WorkerParams`] with `land`
/// replaced by the parcels, plus each type's exit form and the exit good. Build a
/// [`ParcelEconomy`] from them to validate.
#[derive(Clone, Debug, PartialEq)]
pub struct ParcelParams<S = PowerSchedule> {
    /// The parcels, in the order every sum over parcels runs; at least one.
    pub parcels: Vec<Parcel>,
    /// γ(x), relative human productivity on the one task line, for efficiency 1.
    pub schedule: S,
    /// ρ, the interest rate.
    pub rho: f64,
    /// The machine types (unit 1c).
    pub machine_types: Vec<MachineType>,
    /// The segments' edges on the task line (unit 1b).
    pub edges: Vec<f64>,
    /// The categories (unit 1b).
    pub categories: Vec<Category>,
    /// The categories' intermediate inputs (unit 1c).
    pub intermediate: Vec<Vec<f64>>,
    /// The worker types (unit 1d).
    pub worker_types: Vec<WorkerType>,
    /// The common human-required hours (unit 1d).
    pub human_required: Vec<f64>,
    /// The reserved hours (unit 1d); all 0 in an economy with a priced type (§2.9).
    pub reserved: Vec<Vec<f64>>,
    /// Each worker type's exit form, in type order.
    pub exits: Vec<ExitForm>,
    /// g, the category in whose units s₀ and s̲ are measured; q = r/p_g (§2.5).
    pub exit_good: usize,
}

impl<S> ParcelParams<S> {
    /// Unit 1d's economy in parcel form (docs/unit-1e.md §3.3, E0): one enclosed parcel of T
    /// acres of quality 1, every type in the dependence form, exit good 0. Wherever 1d has an
    /// equilibrium it solves to 1d's bit for bit.
    pub fn from_workers(params: WorkerParams<S>) -> Self {
        let kinds = params.worker_types.len();
        ParcelParams {
            parcels: vec![Parcel {
                acreage: params.land,
                quality: 1.0,
                access: Access::Enclosed,
            }],
            schedule: params.schedule,
            rho: params.rho,
            machine_types: params.machine_types,
            edges: params.edges,
            categories: params.categories,
            intermediate: params.intermediate,
            worker_types: params.worker_types,
            human_required: params.human_required,
            reserved: params.reserved,
            exits: vec![ExitForm::Dependence; kinds],
            exit_good: 0,
        }
    }
}

/// Which land market holds an equilibrium (docs/unit-1e.md §2.8), and so the numeraire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandMarket {
    /// Enclosed land fully used: r = 1 is the numeraire (units 1a-1d's units).
    Scarce,
    /// Enclosed land idle at r = 0: the pool's wage is the numeraire, v = 1.
    Idle,
}

impl LandMarket {
    /// 0 or 1, as [`Eq1e::outputs`] reports it.
    pub fn code(self) -> u32 {
        match self {
            LandMarket::Scarce => 0,
            LandMarket::Idle => 1,
        }
    }
}

/// Where the exit plots stand (docs/unit-1e.md §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitLand {
    /// No one asks for a plot: G(0) = 0.
    Unused,
    /// The commons has room: plots are free, r_o = 0.
    Commons,
    /// The commons is full and rations plots by a shadow rent r_o in (0, r) that nobody
    /// receives.
    Crowded,
    /// Plots spill onto enclosed land at r_o = r and leave production: no suitable land idles
    /// (§0.2, Prop exit (ii)). Only while enclosed land is scarce, r = 1.
    Enclosed,
    /// At r = 0: plots fill the commons and spill onto idle enclosed land, where they stand
    /// free. Suitable land still idles, so this is Prop exit (i)'s commons in all but access,
    /// not enclosure (§0.2, §4.4; docs/unit-1e.md §12 item 17).
    Idle,
}

impl ExitLand {
    /// 0 to 4, as [`Eq1e::outputs`] reports it.
    pub fn code(self) -> u32 {
        match self {
            ExitLand::Unused => 0,
            ExitLand::Commons => 1,
            ExitLand::Crowded => 2,
            ExitLand::Enclosed => 3,
            ExitLand::Idle => 4,
        }
    }
}

/// A worker type's exit branch (docs/unit-1e.md §4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    /// The dependence form: no exit life.
    Dependence,
    /// A plot, s = s₀ − q_o·h.
    Plot,
    /// The floor, s = s̲ (at q_o = q_enc the type counts as on its floor).
    Floor,
}

impl Branch {
    /// 0 to 2, as [`Eq1e::outputs`] reports it.
    pub fn code(self) -> u32 {
        match self {
            Branch::Dependence => 0,
            Branch::Plot => 1,
            Branch::Floor => 2,
        }
    }
}

/// An equilibrium at an enclosure point (docs/unit-1e.md §2.10 and §4.7): the type's exiters
/// are indifferent between a rented plot and the floor, q = q_enc, and a share of them rents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnclosureTie {
    /// The worker type at its enclosure point.
    pub worker: usize,
    /// ψ, the share of its exiters on plots (on the commons and rented), in [0, 1].
    pub share: f64,
}

/// A point of the path where a plot-taking type's q crosses its q_enc (docs/unit-1e.md §4.7):
/// the largest double of the path's parameter at which the type is on its floor at the market
/// rent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnclosurePoint {
    /// The worker type.
    pub worker: usize,
    /// The stretch: the line, the all-human corner or the wall.
    pub margin: Margin,
    /// x (0 at the all-human corner, 1 at the wall).
    pub x: f64,
    /// The pool's wage at the point (on the line, the margin's).
    pub v: f64,
    /// The technique.
    pub technique: usize,
}

/// A side of an enclosure point (docs/unit-1e.md §4.7).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnclosureSide {
    /// The type on its floor at the market rent.
    Below,
    /// The type renting plots at the market rent.
    Above,
    /// ψ of the type's exiters on plots, the rest on the floor, at the market rent, the type's
    /// supply from its floor value.
    Share(f64),
}

/// How the exit sub-problem treats one type at an enclosure point.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Force {
    None,
    Above(usize),
    Share(usize, f64),
}

/// One type's exit form, with the quantities derived at construction (§5.2).
#[derive(Clone, Copy, Debug, PartialEq)]
struct TypeExit {
    priced: Option<PricedExit>,
    /// Δ = s₀ − s̲.
    advantage: f64,
    /// q_enc = Δ/h, for a plot-taking type.
    threshold: Option<f64>,
    /// Whether the type is in 𝒫: priced, h > 0 and Δ > 0.
    takes_plots: bool,
}

/// The exit sub-problem at one point (docs/unit-1e.md §4.4 and §5.1 step 2).
#[derive(Clone, Debug, PartialEq)]
struct ExitState {
    regime: ExitLand,
    plot_rent: f64,
    branches: Vec<Branch>,
    values: Vec<f64>,
    goods: Vec<f64>,
    households: Vec<f64>,
    plots: Vec<f64>,
    demand: f64,
    commons_occupied: f64,
    rented: f64,
}

/// The per-type results of one trial plot rent r_o.
struct Trial {
    demand: f64,
    branches: Vec<Branch>,
    values: Vec<f64>,
    goods: Vec<f64>,
    households: Vec<f64>,
    plots: Vec<f64>,
}

/// Prices and quantities at one point of the path (docs/unit-1e.md §5.1): unit 1d's point on
/// the market's land, and the exit sub-problem.
#[derive(Clone, Debug, PartialEq)]
pub struct ParcelPoint {
    /// Unit 1d's point, on the market's land T_m, with each type's supply from its exit value.
    pub point: WorkerPoint,
    /// r: 1 while enclosed land is scarce (prices per unit of rent), 0 on the idle stretch
    /// (prices per unit of the pool's wage).
    pub rent: f64,
    /// T_m, the market's land in use.
    pub market_land: f64,
    /// p_g, the exit good's price.
    pub exit_good_price: f64,
    /// q = r/p_g; at r = 0 its limit at the wall's end, 0 where the exit good embodies labour
    /// and 1/b̃_g where it does not (docs/unit-1e.md §12 item 16).
    pub q: f64,
    /// Where the exit plots stand.
    pub exit_land: ExitLand,
    /// r_o, the rent an exit plot pays per unit of service.
    pub plot_rent: f64,
    /// Each type's branch.
    pub branches: Vec<Branch>,
    /// e_i, each type's exit value in money.
    pub exit_values: Vec<f64>,
    /// s_i, each type's exit value in the exit good.
    pub exit_goods: Vec<f64>,
    /// Each type's exiters on plots.
    pub plot_households: Vec<f64>,
    /// Each type's plot land, on the commons and rented.
    pub plots: Vec<f64>,
    /// G, the plot land asked for at r_o (a crowded commons' split type counted with its share).
    pub plot_demand: f64,
    /// T_oc, the commons occupied.
    pub commons_occupied: f64,
    /// T_p, the enclosed land in exit plots.
    pub rented_plots: f64,
    /// The market in the form unit 1d's evaluation takes.
    market: Market,
}

impl ParcelPoint {
    /// f = n_D − S, or +∞ where the point is short.
    pub fn excess_demand(&self) -> f64 {
        self.point.excess_demand()
    }
}

/// A validated unit-1e economy with every quantity that does not depend on the path
/// (docs/unit-1e.md §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct ParcelEconomy<S = PowerSchedule> {
    params: ParcelParams<S>,
    /// Unit 1d's economy on the enclosed land T.
    workers: WorkerEconomy<S>,
    /// T = Σ_{enclosed} A_z·Q_z.
    enclosed: f64,
    /// T_o = Σ_{open} A_z·Q_z.
    commons: f64,
    exits: Vec<TypeExit>,
    /// 𝒫, the plot-taking types.
    plot_takers: Vec<usize>,
    /// Every type in the dependence form or with s₀ = s̲ = 0: unit 1d's operations.
    exit_free: bool,
    /// b̄_g, the exit good's land through the chain at x = 0.
    exit_good_land: f64,
    /// ℓ₀, n_D per unit of market land at x = 0.
    demand_per_land: f64,
    /// §5.4's certification.
    certified: bool,
    /// The enclosed parcels, best first (quality descending, ties to the lower index).
    enclosed_order: Vec<usize>,
    /// The open parcels, best first.
    open_order: Vec<usize>,
}

/// The error for a parameter of parcel `index`.
fn in_parcel(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "parcel",
        index,
        error: Box::new(error),
    }
}

/// The error for a parameter of worker type `index`.
fn in_worker(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "worker type",
        index,
        error: Box::new(error),
    }
}

/// The parcels of one access, best first: quality descending, ties to the lower index.
fn best_first(parcels: &[Parcel], access: Access) -> Vec<usize> {
    let mut order: Vec<usize> = (0..parcels.len())
        .filter(|&z| parcels[z].access == access)
        .collect();
    order.sort_by(|&a, &b| {
        parcels[b]
            .quality
            .total_cmp(&parcels[a].quality)
            .then(a.cmp(&b))
    });
    order
}

impl<S: Schedule + Clone> ParcelEconomy<S> {
    /// Validates the parameters (docs/unit-1e.md §3.2) and computes every quantity that does
    /// not depend on the path (§5.2).
    ///
    /// In order: the parcels in unit 1d's place of `land` (at least one; each `acreage` scale
    /// and `quality` 0 or scale, as `ParamError::Item { kind: "parcel", .. }`; T in
    /// [[`SCALE_FLOOR`], [`SCALE_CEIL`]] and T_o at most [`SCALE_CEIL`], as `Invalid { name:
    /// "parcels" }`), then unit 1d's checks in 1d's order on T, then one exit form per worker
    /// type (`Invalid { name: "exits" }`), each priced form's `gross`, `floor` and `plot` 0 or
    /// scale (`Item { kind: "worker type" }`), `exit_good` a category, no reserved hours with a
    /// priced type (`Invalid { name: "reserved" }`), and land for every plot, T + T_o >
    /// Σ_{i∈𝒫} h_i·N_i (`Invalid { name: "parcels" }`). −0.0 is stored as +0.0. With one
    /// enclosed parcel of quality 1 and every type in the dependence form these are 1d's rules.
    pub fn new(mut params: ParcelParams<S>) -> Result<Self, ParamError> {
        // The parcels, in land's place.
        if params.parcels.is_empty() {
            return Err(ParamError::Invalid {
                name: "parcels",
                reason: "the economy needs at least one parcel",
            });
        }
        for (index, parcel) in params.parcels.iter_mut().enumerate() {
            let item = in_parcel(index);
            parcel.acreage = params::scale("acreage", parcel.acreage).map_err(&item)?;
            parcel.quality = params::zero_or_scale("quality", parcel.quality).map_err(&item)?;
        }
        let (mut enclosed, mut commons) = (0.0, 0.0);
        for parcel in &params.parcels {
            match parcel.access {
                Access::Enclosed => enclosed += parcel.acreage * parcel.quality,
                Access::Open => commons += parcel.acreage * parcel.quality,
            }
        }
        if !(SCALE_FLOOR..=SCALE_CEIL).contains(&enclosed) {
            return Err(ParamError::Invalid {
                name: "parcels",
                reason: "the enclosed parcels' land service T = Σ A·Q must be in \
                         [SCALE_FLOOR, SCALE_CEIL]",
            });
        }
        if commons > SCALE_CEIL {
            return Err(ParamError::Invalid {
                name: "parcels",
                reason: "the commons' land service Σ A·Q over open parcels must be at most \
                         SCALE_CEIL",
            });
        }
        // Unit 1d's checks on T.
        let workers = WorkerEconomy::new(WorkerParams {
            land: enclosed,
            schedule: params.schedule.clone(),
            rho: params.rho,
            machine_types: std::mem::take(&mut params.machine_types),
            edges: std::mem::take(&mut params.edges),
            categories: std::mem::take(&mut params.categories),
            intermediate: std::mem::take(&mut params.intermediate),
            worker_types: std::mem::take(&mut params.worker_types),
            human_required: std::mem::take(&mut params.human_required),
            reserved: std::mem::take(&mut params.reserved),
        })?;
        {
            let w = workers.params();
            params.rho = w.rho;
            params.machine_types = w.machine_types.clone();
            params.edges = w.edges.clone();
            params.categories = w.categories.clone();
            params.intermediate = w.intermediate.clone();
            params.worker_types = w.worker_types.clone();
            params.human_required = w.human_required.clone();
            params.reserved = w.reserved.clone();
        }
        // The exit forms.
        let kinds = params.worker_types.len();
        if params.exits.len() != kinds {
            return Err(ParamError::Invalid {
                name: "exits",
                reason: "the exit forms need one entry per worker type",
            });
        }
        for (index, form) in params.exits.iter_mut().enumerate() {
            if let ExitForm::Priced(priced) = form {
                *priced = priced.validated().map_err(in_worker(index))?;
            }
        }
        if params.exit_good >= params.categories.len() {
            return Err(ParamError::Invalid {
                name: "exit_good",
                reason: "the exit good must name a category",
            });
        }
        let any_priced = params
            .exits
            .iter()
            .any(|e| matches!(e, ExitForm::Priced(_)));
        if any_priced && params.reserved.iter().flatten().any(|&r| r > 0.0) {
            return Err(ParamError::Invalid {
                name: "reserved",
                reason: "an economy with a priced exit form has no reserved hours: the land and \
                         participation loop with walled types has no solution method yet \
                         (docs/unit-1e.md §2.9)",
            });
        }
        let exits: Vec<TypeExit> = params
            .exits
            .iter()
            .map(|form| match *form {
                ExitForm::Dependence => TypeExit {
                    priced: None,
                    advantage: 0.0,
                    threshold: None,
                    takes_plots: false,
                },
                ExitForm::Priced(p) => {
                    let advantage = p.gross - p.floor;
                    let takes_plots = p.plot > 0.0 && advantage > 0.0;
                    TypeExit {
                        priced: Some(p),
                        advantage,
                        threshold: takes_plots.then(|| advantage / p.plot),
                        takes_plots,
                    }
                }
            })
            .collect();
        let plot_takers: Vec<usize> = (0..kinds).filter(|&i| exits[i].takes_plots).collect();
        let mut need = 0.0;
        for &i in &plot_takers {
            let plot = exits[i].priced.map_or(0.0, |p| p.plot);
            need += plot * params.worker_types[i].workers;
        }
        if enclosed + commons <= need {
            return Err(ParamError::Invalid {
                name: "parcels",
                reason: "the parcels must leave land for production when everyone exits: \
                         T + T_o > Σ h_i·N_i over the plot-taking types",
            });
        }
        let exit_free = exits
            .iter()
            .all(|e| e.priced.is_none_or(|p| p.gross == 0.0 && p.floor == 0.0));
        let m = workers.machines();
        let g = params.exit_good;
        let exit_good_land = m.chain_land()[g];
        let demand_per_land =
            (m.basket_all_human_hours() + workers.human_required_per_basket()) / m.basket_land();
        let certified = params.rho == 0.0
            && (plot_takers.len() <= 1 || commons == 0.0)
            && plot_takers.iter().all(|&i| {
                let p = exits[i].priced.expect("a plot taker is priced");
                p.plot <= p.gross * exit_good_land
                    && p.plot * demand_per_land <= params.worker_types[i].efficiency
            });
        let enclosed_order = best_first(&params.parcels, Access::Enclosed);
        let open_order = best_first(&params.parcels, Access::Open);
        Ok(ParcelEconomy {
            params,
            workers,
            enclosed,
            commons,
            exits,
            plot_takers,
            exit_free,
            exit_good_land,
            demand_per_land,
            certified,
            enclosed_order,
            open_order,
        })
    }

    /// The validated parameters.
    pub fn params(&self) -> &ParcelParams<S> {
        &self.params
    }

    /// Sets unit 1f's participation rule and basket (docs/unit-1f.md §5.2) on the worker economy
    /// underneath, and recomputes §5.4's certification as unit 1f states it: 1e's Proposition 5
    /// with κ_w·ε_i in place of ε_i, κ_w = (1 − τ_w)/(1 + t_c), which needs μ_w = μ_e where a
    /// type takes plots (docs/unit-1f.md §5.4). With [`Rule::none`] it is 1e's bit for bit.
    pub(crate) fn set_households(&mut self, rule: Rule, ces: Option<Ces>) {
        let kappa = rule.net / rule.consumer;
        let even = rule.gap == 0.0 || self.plot_takers.is_empty();
        self.certified = self.params.rho == 0.0
            && even
            && (self.plot_takers.len() <= 1 || self.commons == 0.0)
            && self.plot_takers.iter().all(|&i| {
                let p = self.exits[i].priced.expect("a plot taker is priced");
                p.plot <= p.gross * self.exit_good_land
                    && p.plot * self.demand_per_land
                        <= kappa * self.params.worker_types[i].efficiency
            });
        self.workers.set_households(rule, ces);
    }

    /// Whether every type is without an exit value: in the dependence form, or priced with
    /// s₀ = s̲ = 0 (docs/unit-1e.md §2.12).
    pub(crate) fn exit_free(&self) -> bool {
        self.exit_free
    }

    /// Unit 1d's economy on the enclosed land T underneath.
    pub fn workers(&self) -> &WorkerEconomy<S> {
        &self.workers
    }

    /// T = Σ_{enclosed} A_z·Q_z, the market's land-service endowment.
    pub fn enclosed_land(&self) -> f64 {
        self.enclosed
    }

    /// T_o = Σ_{open} A_z·Q_z, the commons.
    pub fn commons(&self) -> f64 {
        self.commons
    }

    /// p*_g,i = r·h_i/Δ_i at r = 1, the exit good's price at each type's enclosure point;
    /// `None` for a type that takes no plot.
    pub fn enclosure_targets(&self) -> Vec<Option<f64>> {
        self.exits
            .iter()
            .map(|e| match (e.takes_plots, e.priced) {
                (true, Some(p)) => Some(p.plot / e.advantage),
                _ => None,
            })
            .collect()
    }

    /// q_enc,i per type, `None` for a type that takes no plot.
    pub fn thresholds(&self) -> Vec<Option<f64>> {
        self.exits.iter().map(|e| e.threshold).collect()
    }

    /// 𝒫, the plot-taking types: priced, with h_i > 0 and s₀,i > s̲_i.
    pub fn plot_takers(&self) -> &[usize] {
        &self.plot_takers
    }

    /// Whether §5.4's Proposition 5 certifies the economy: ρ = 0, every plot-taking type with
    /// h_i ≤ s₀,i·b̄_g and h_i·ℓ₀ ≤ ε_i, and at most one type taking plots (or no commons).
    /// The excess demand is then nonincreasing along the path and the count exact.
    pub fn certified(&self) -> bool {
        self.certified
    }

    /// b̄_g, the exit good's land through the chain at x = 0, and ℓ₀ = (L̄_ŷ + L^H_ŷ)/B_ŷ, the
    /// pool's hours per unit of market land at x = 0 (§5.4).
    pub fn certification_totals(&self) -> (f64, f64) {
        (self.exit_good_land, self.demand_per_land)
    }

    /// Prices and quantities at x on the line under the envelope's technique there.
    pub fn at(&self, x: f64) -> ParcelPoint {
        self.at_with(x, self.workers.technique_at(x))
    }

    /// Prices and quantities at x on the line with `technique` doing the machine tasks and the
    /// task margin setting v (docs/unit-1e.md §5.1), r = 1. In unit 1d's form, the point is
    /// 1d's [`WorkerEconomy::at_with`] bit for bit.
    pub fn at_with(&self, x: f64, technique: usize) -> ParcelPoint {
        let block = self.workers.block_at(x, technique);
        self.evaluate(x, technique, block, 1.0, None, Force::None, None)
    }

    /// Prices and quantities at x with the pool's wage v given (a corner's evaluation), r = 1.
    pub fn at_wage(&self, x: f64, v: f64, technique: usize) -> ParcelPoint {
        let block = self.workers.block_at_wage(v, 1.0, technique);
        self.evaluate(x, technique, block, 1.0, None, Force::None, None)
    }

    /// A point of the idle stretch (docs/unit-1e.md §4.6): x = 1 under `technique`, r = 0,
    /// the pool's wage the numeraire (v = 1), and `market_land` of enclosed land in use.
    pub fn at_idle(&self, market_land: f64, technique: usize) -> ParcelPoint {
        self.idle_point(market_land, technique, None)
    }

    /// [`at_idle`](Self::at_idle) with a type at the edge of its reserved shortage.
    pub fn at_idle_edge(&self, market_land: f64, technique: usize, edge: Edge) -> ParcelPoint {
        self.idle_point(market_land, technique, Some(edge))
    }

    fn idle_point(&self, market_land: f64, technique: usize, edge: Option<Edge>) -> ParcelPoint {
        let block = self.workers.block_at_wage(1.0, 0.0, technique);
        self.evaluate(
            BRACKET_HI,
            technique,
            block,
            0.0,
            Some(market_land),
            Force::None,
            edge,
        )
    }

    /// The evaluation at an enclosure point on one of its sides (docs/unit-1e.md §4.7).
    pub fn at_enclosure(&self, point: &EnclosurePoint, side: EnclosureSide) -> ParcelPoint {
        let force = match side {
            EnclosureSide::Below => Force::None,
            EnclosureSide::Above => Force::Above(point.worker),
            EnclosureSide::Share(share) => Force::Share(point.worker, share),
        };
        let block = match point.margin {
            Margin::Contestable => self.workers.block_at(point.x, point.technique),
            _ => self.workers.block_at_wage(point.v, 1.0, point.technique),
        };
        self.evaluate(point.x, point.technique, block, 1.0, None, force, None)
    }

    /// §5.1: the price side at the rent r, the exit sub-problem at its prices, and 1d's
    /// quantities and supplies on the market's land (T − T_p while land is scarce, or given).
    #[allow(clippy::too_many_arguments)]
    fn evaluate(
        &self,
        x: f64,
        technique: usize,
        block: crate::machine_block::BlockPrices,
        rent: f64,
        land: Option<f64>,
        force: Force,
        edge: Option<Edge>,
    ) -> ParcelPoint {
        let prices = self.workers.price_side(x, technique, block, rent);
        // At r = 0 a free exit good (one that embodies no labour) leaves q = r/p_g at 0/0; its
        // limit at the wall's end is 1/b̃_g, and the plots are decided there (§12 item 16).
        let free_good = rent == 0.0 && prices.base_prices[self.params.exit_good] == 0.0;
        let wall = free_good.then(|| self.wall_price(technique));
        let exit = if self.exit_free {
            self.free_state()
        } else {
            let p_g = prices.base_prices[self.params.exit_good];
            // d at the point (docs/unit-1f.md §4.3): d̂·P^c under RentRate, and under Dividend
            // on the whole endowment, since the Dividend closure has no plot-taking type (§2.6).
            let market = Market {
                rent,
                land: land.unwrap_or(self.enclosed),
                exit: Vec::new(),
            };
            let transfer = self.workers.transfer_at(&prices, &market);
            let point = (prices.block.v, prices.base_p_s, transfer);
            self.exit_state(point, p_g, rent, force, wall)
        };
        let market_land = land.unwrap_or(self.enclosed - exit.rented);
        let market = Market {
            rent,
            land: market_land,
            exit: exit.values.clone(),
        };
        let point = self.workers.finish(prices, edge, &market);
        let exit_good_price = point.prices[self.params.exit_good];
        let q = match wall {
            _ if rent != 0.0 => rent / exit_good_price,
            Some(p_wall) if exit_good_price == 0.0 => 1.0 / p_wall,
            _ => 0.0,
        };
        ParcelPoint {
            point,
            rent,
            market_land,
            exit_good_price,
            q,
            exit_land: exit.regime,
            plot_rent: exit.plot_rent,
            branches: exit.branches,
            exit_values: exit.values,
            exit_goods: exit.goods,
            plot_households: exit.households,
            plots: exit.plots,
            plot_demand: exit.demand,
            commons_occupied: exit.commons_occupied,
            rented_plots: exit.rented,
            market,
        }
    }

    /// The exit sub-problem of an economy without exit values: no plots, e_i = 0.
    fn free_state(&self) -> ExitState {
        let kinds = self.exits.len();
        ExitState {
            regime: ExitLand::Unused,
            plot_rent: 0.0,
            branches: self
                .exits
                .iter()
                .map(|e| match e.priced {
                    None => Branch::Dependence,
                    Some(_) => Branch::Floor,
                })
                .collect(),
            values: vec![0.0; kinds],
            goods: vec![0.0; kinds],
            households: vec![0.0; kinds],
            plots: vec![0.0; kinds],
            demand: 0.0,
            commons_occupied: 0.0,
            rented: 0.0,
        }
    }

    /// Each type's branch, exit value and supply at a trial plot rent r_o, in type order, and
    /// G, the plot land asked for, from 0.0 (docs/unit-1e.md §5.1 step 2).
    ///
    /// `wall` is `Some(p_wall)` at r = 0 with a free exit good (§12 item 16): the branch and
    /// the exit goods are then decided as at the wall's end, in rent units, where the exit good
    /// costs p_wall = b̃_g per unit of rent and `plot_rent` is the trial rent in [0, 1] there,
    /// while the exit value in money is p_g·s = 0.
    fn trial(
        &self,
        plot_rent: f64,
        (v, p_s, transfer): (f64, f64, f64),
        p_g: f64,
        force: Force,
        wall: Option<f64>,
    ) -> Trial {
        // (the price that decides the branch and the goods, the plot rent in money)
        let (price, money_rent) = match wall {
            None => (p_g, plot_rent),
            Some(p_wall) => (p_wall, 0.0),
        };
        let types = &self.params.worker_types;
        let kinds = types.len();
        let mut t = Trial {
            demand: 0.0,
            branches: Vec::with_capacity(kinds),
            values: Vec::with_capacity(kinds),
            goods: Vec::with_capacity(kinds),
            households: Vec::with_capacity(kinds),
            plots: Vec::with_capacity(kinds),
        };
        for (i, t_i) in types.iter().enumerate() {
            let wage = t_i.efficiency * v;
            let exit = &self.exits[i];
            let (branch, value, goods, share) = match exit.priced {
                None => (Branch::Dependence, 0.0, 0.0, 0.0),
                Some(p) => match force {
                    Force::Share(k, share) if k == i => {
                        (Branch::Floor, p_g * p.floor, p.floor, share)
                    }
                    _ => {
                        let plot = match force {
                            Force::Above(k) if k == i => true,
                            _ => plot_rent * p.plot < price * exit.advantage,
                        };
                        if plot {
                            let value = num::fma(p_g, p.gross, -(money_rent * p.plot));
                            let goods = if plot_rent == 0.0 {
                                p.gross
                            } else {
                                num::fma(-(plot_rent / price), p.plot, p.gross)
                            };
                            (Branch::Plot, value, goods, 1.0)
                        } else {
                            (Branch::Floor, p_g * p.floor, p.floor, 0.0)
                        }
                    }
                },
            };
            let supply = self.workers.rule().supply(t_i, wage, p_s, value, transfer);
            let households = share * (t_i.workers - supply);
            let plot = exit.priced.map_or(0.0, |p| p.plot) * households;
            if share > 0.0 {
                t.demand += plot;
            }
            t.branches.push(branch);
            t.values.push(value);
            t.goods.push(goods);
            t.households.push(households);
            t.plots.push(plot);
        }
        t
    }

    /// §4.4's regime and the plot rent at the point's prices, at the market rent r.
    ///
    /// At r = 0 the plots are free: past the commons they stand on idle enclosed land
    /// ([`ExitLand::Idle`]). With a free exit good (`wall` is `Some`, §12 item 16) the regime and
    /// the branches are decided in the wall's end's rent units, trial rents in [0, 1], and every
    /// rent reported in money is 0.
    fn exit_state(
        &self,
        point: (f64, f64, f64),
        p_g: f64,
        rent: f64,
        force: Force,
        wall: Option<f64>,
    ) -> ExitState {
        let t_o = self.commons;
        // the rent in the units the plots are decided in, and a rent reported in money
        let top = if wall.is_some() { 1.0 } else { rent };
        let money = |r: f64| if wall.is_some() { 0.0 } else { r };
        let trial = |r: f64| self.trial(r, point, p_g, force, wall);
        let state = |t: Trial, regime, plot_rent, commons_occupied, rented| ExitState {
            regime,
            plot_rent,
            branches: t.branches,
            values: t.values,
            goods: t.goods,
            households: t.households,
            plots: t.plots,
            demand: t.demand,
            commons_occupied,
            rented,
        };
        if let Force::Share(..) = force {
            let t = trial(top);
            let rented = (t.demand - t_o).max(0.0);
            let occupied = t.demand.min(t_o);
            return state(t, ExitLand::Enclosed, money(rent), occupied, rented);
        }
        let t0 = trial(0.0);
        if t0.demand == 0.0 {
            return state(t0, ExitLand::Unused, 0.0, 0.0, 0.0);
        }
        if t0.demand <= t_o {
            let occupied = t0.demand;
            return state(t0, ExitLand::Commons, 0.0, occupied, 0.0);
        }
        let t1 = trial(top);
        if t1.demand >= t_o {
            let rented = t1.demand - t_o;
            // at r = 0 the spill stands free on idle enclosed land: not enclosure (§0.2)
            let regime = if rent == 0.0 {
                ExitLand::Idle
            } else {
                ExitLand::Enclosed
            };
            return state(t1, regime, money(rent), t_o, rented);
        }
        // Crowded: the least double r_o in [0, r] with G(r_o) ≤ T_o, by bisection on bit
        // patterns.
        let (mut lo, mut hi) = (0.0_f64, top);
        while hi.to_bits() > lo.to_bits() + 1 {
            let mid = f64::from_bits(lo.to_bits() + (hi.to_bits() - lo.to_bits()) / 2);
            if trial(mid).demand > t_o {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let below = trial(lo);
        let mut t = trial(hi);
        // A type whose plot demand drops between the two splits between commons plots and its
        // floor, with the same supply either way (§4.4).
        if let Some(i) = (0..t.branches.len())
            .find(|&i| below.branches[i] == Branch::Plot && t.branches[i] == Branch::Floor)
        {
            let plot = self.exits[i].priced.map_or(0.0, |p| p.plot);
            let exiters = self.params.worker_types[i].workers - self.point_supply(&t, i, point);
            let land = (t_o - t.demand).max(0.0).min(plot * exiters);
            t.plots[i] = land;
            t.households[i] = land / plot;
            t.demand += land;
        }
        state(t, ExitLand::Crowded, money(hi), t_o, 0.0)
    }

    /// p_g per unit of rent at the wall's end under `technique`, where the exit good embodies
    /// no labour: its price at x = 1 with v = 0 and r = 1, which is b̃_g. Every point of the
    /// wall's last piece repeats it bit for bit, since v·0 = 0 and the chain's elimination adds
    /// nonnegative terms only, so that a structural zero stays 0 (leontief.rs).
    fn wall_price(&self, technique: usize) -> f64 {
        let block = self.workers.block_at_wage(0.0, 1.0, technique);
        self.workers
            .price_side(BRACKET_HI, technique, block, 1.0)
            .base_prices[self.params.exit_good]
    }

    /// n_S,i at a trial's exit value, with the point's wage, P_s and transfer.
    fn point_supply(&self, t: &Trial, i: usize, (v, p_s, transfer): (f64, f64, f64)) -> f64 {
        let ty = &self.params.worker_types[i];
        self.workers
            .rule()
            .supply(ty, ty.efficiency * v, p_s, t.values[i], transfer)
    }
}

/// A value of the path's sequence (docs/unit-1e.md §5.3 step 2).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Station {
    /// v → 0: f_0 = n_D(0) − S(0), positive when > 0 (docs/unit-1f.md §2.9), which it is in every
    /// economy without an in-work benefit.
    Start,
    /// An enclosure point, the type on its floor (index into the path's points).
    EnclosureBelow(usize),
    /// The same point, the type renting.
    EnclosureAbove(usize),
    Zero,
    Lo,
    SwitchBelow(usize),
    SwitchAbove(usize),
    One,
    WallBelow(usize),
    WallAbove(usize),
    /// f_∞, the wall's end and the idle stretch's start.
    End,
    /// −S_∞, the idle stretch's end.
    Rest,
}

impl Station {
    /// §5.3 step 3: a value is on the positive side when > 0, f(1) when ≥ 0, and the idle
    /// stretch's end when ≥ 0 (S_∞ = 0: no production is no equilibrium). The start is
    /// evaluated (docs/unit-1f.md §2.9), and > 0 in every economy without an in-work benefit.
    fn positive(self, f: f64) -> bool {
        match self {
            Station::One | Station::Rest => f >= 0.0,
            _ => f > 0.0,
        }
    }
}

/// How a piece of the path is parametrized (docs/unit-1e.md §5.3 step 2).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Span {
    /// The all-human corner in ω = v/P_s, x = 0, with the corner's (L_s, B_s).
    AllHuman {
        technique: usize,
        l_s: f64,
        b_s: f64,
    },
    /// [0, lo] on the line, not scanned.
    Bottom { technique: usize },
    /// A region of the line in x.
    Line { technique: usize },
    /// A piece of the wall in ω, x = 1, with the corner's (L_s, B_s) and the exit good's λ̃.
    Wall {
        technique: usize,
        l_s: f64,
        b_s: f64,
        lambda_g: f64,
    },
    /// The idle stretch in T_m, not scanned.
    Idle { technique: usize },
}

impl Span {
    /// The pool's wage at ω on a corner: v = ω·B_s/(1 − ω·L_s), 1d's with no reserved hours.
    fn wage(self, omega: f64) -> f64 {
        match self {
            Span::AllHuman { l_s, b_s, .. } | Span::Wall { l_s, b_s, .. } => {
                (omega * b_s) / ((1.0 - 0.0) - omega * l_s)
            }
            _ => f64::NAN,
        }
    }

    fn scanned(self) -> bool {
        matches!(
            self,
            Span::AllHuman { .. } | Span::Line { .. } | Span::Wall { .. }
        )
    }
}

/// A piece of the path between two values of the sequence.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Piece {
    span: Span,
    lo: f64,
    hi: f64,
    /// The pool's wage at each end, on a corner.
    v_lo: f64,
    v_hi: f64,
}

/// The sequence of a priced economy (docs/unit-1e.md §5.3 step 2).
struct PricedPath {
    stations: Vec<(Station, f64)>,
    /// pieces[k] lies between stations[k − 1] and stations[k]; `None` for the two values of a
    /// switch or an enclosure point.
    pieces: Vec<Option<Piece>>,
    enclosures: Vec<EnclosurePoint>,
}

impl PricedPath {
    fn push(&mut self, piece: Option<Piece>, station: (Station, f64)) {
        self.pieces.push(piece);
        self.stations.push(station);
    }
}

/// The value of the sequence at a point: +∞ where it is short, else f = n_D − S, finite.
fn value(what: &'static str, q: &ParcelPoint) -> Result<f64, SolveError> {
    if q.point.short.is_some() {
        return Ok(f64::INFINITY);
    }
    let f = q.point.n_d - q.point.n_s;
    if f.is_finite() {
        Ok(f)
    } else {
        Err(SolveError::NonFinite { what })
    }
}

/// `Some(f)` for a finite value, `None` for +∞.
fn finite_or_none(f: f64) -> Option<f64> {
    f.is_finite().then_some(f)
}

/// What the report of an equilibrium needs besides unit 1d's point.
struct Found {
    q: ParcelPoint,
    one_minus_x: f64,
    steps: u32,
    margin: Margin,
    technique: usize,
    tie: Option<Tie>,
    edge: Option<Edge>,
    enclosure: Option<EnclosureTie>,
}

impl<S: Schedule + Clone> ParcelEconomy<S> {
    /// Classifies the economy and solves it (docs/unit-1e.md §5.3), with the scan of
    /// [`EXIT_SCAN`] points per piece.
    ///
    /// 1d's viability test comes first. Then the excess demand is read along the whole path:
    /// the all-human corner, [0, lo], the line's regions, the wall's pieces, the end of the wall
    /// f_∞, and the idle stretch's end −S_∞, with each enclosure point (§4.7) as a pair of
    /// values and, in an economy with a plot-taking type, `EXIT_SCAN` interior points of every
    /// piece of the corner, the line on [lo, 1] and the wall. The start v → 0 is evaluated, f_0 =
    /// n_D(0) − S(0), positive without an in-work benefit (docs/unit-1f.md §2.9); a value is
    /// positive when > 0, f(1) and −S_∞ when ≥ 0. No change of side is
    /// [`SolveError::NoMarket`], or [`SolveError::SurplusLabour`] from a start that is not
    /// positive; more than one is [`SolveError::MultipleEquilibria`]. One is the
    /// equilibrium, inside [`Regime::Interior`]: on the line, a corner or the idle stretch, at a
    /// technique tie or an enclosure tie. In an economy without exit values (every type in the
    /// dependence form, or priced with s₀ = s̲ = 0) the sequence is unit 1d's with −S_∞
    /// appended and every equilibrium 1d has is 1d's bit for bit (§2.12).
    ///
    /// Returns [`SolveError::NonFinite`] (or the per-item variants) if a value the solve decides
    /// on, or any reported output, is not finite, and [`SolveError::LaborNotCleared`] if the
    /// pool's residual exceeds [`LABOR_RESIDUAL_NET`](crate::LABOR_RESIDUAL_NET) of n_pool.
    pub fn solve(&self) -> Result<Regime<Eq1e>, SolveError> {
        self.solve_scanned(EXIT_SCAN)
    }

    /// [`solve`](Self::solve) with `scan` interior points per piece in place of
    /// [`EXIT_SCAN`], for the gate (docs/unit-1e.md §8, e7 and e8): 0 is the count of 1a-1d,
    /// which reads f only at the sequence's values. The equilibrium, when there is one, does not
    /// depend on it bit for bit, only `scan_points` does.
    pub fn solve_scanned(&self, scan: usize) -> Result<Regime<Eq1e>, SolveError> {
        // A CES basket evaluates whole points at the corners (docs/unit-1f.md §5.3 step 2).
        let eq = if self.exit_free && self.workers.ces().is_none() {
            match self.solve_exit_free(scan)? {
                Ok(eq) => eq,
                Err(d_at_1) => return Ok(Regime::NotViable { d_at_1 }),
            }
        } else {
            match self.solve_priced(scan)? {
                Ok(eq) => eq,
                Err(d_at_1) => return Ok(Regime::NotViable { d_at_1 }),
            }
        };
        if let Some(error) = first_non_finite(&eq) {
            return Err(error);
        }
        labor_net(eq.base.x_star, eq.base.residuals.labor, eq.base.n_pool)?;
        Ok(Regime::Interior(Box::new(eq)))
    }

    /// §5.3 for an economy without exit values: 1d's sequence and location, with the idle
    /// stretch after the wall's end, from the evaluated start (docs/unit-1f.md §2.9), and, where
    /// the Dividend closure's transfer moves at ρ > 0, `scan` interior points of each piece
    /// (§5.3 step 4).
    fn solve_exit_free(&self, scan: usize) -> Result<Result<Eq1e, f64>, SolveError> {
        let path = match self.workers.path()? {
            Ok(path) => path,
            Err(d_at_1) => return Ok(Err(d_at_1)),
        };
        let tau_e = path.end.technique;
        let rest = self.at_idle(0.0, tau_e);
        let f_rest = value("n_D - n_S at the end of the idle stretch", &rest)?;
        let start = path.f_start > 0.0;
        let (gaps, scan_points) = if scan > 0 && self.scans_everywhere() {
            self.exit_free_scan(&path, scan)?
        } else {
            (vec![Vec::new(); path.sequence.len()], 0)
        };
        let mut sides = Vec::with_capacity(path.sequence.len() + 2);
        sides.push(start);
        for &(kind, f) in &path.sequence {
            sides.push(match kind {
                Kind::One => f >= 0.0,
                _ => f > 0.0,
            });
        }
        sides.push(f_rest >= 0.0);
        let c = match count_changes(&sides, &gaps) {
            Ok(c) => c,
            Err(0) if !start => {
                return Err(SolveError::SurplusLabour {
                    f_start: path.f_start,
                })
            }
            Err(0) => {
                return Err(SolveError::NoMarket {
                    f_end: path.end.excess,
                })
            }
            Err(counted) => {
                return Err(SolveError::MultipleEquilibria {
                    sign_changes: counted,
                    switches: path.points,
                })
            }
        };
        let n = path.sequence.len();
        let f_end = path.end.excess;
        let junction = c + 1 == n && f_end == 0.0 && path.last_start != 0.0;
        if c < n && !junction {
            let eq = self.workers.locate(&path, c)?;
            let side = self.free_side(&eq);
            return Ok(Ok(self.extend(eq, &side, None, f_end, scan_points)));
        }
        let found = self.idle_equilibrium(tau_e, self.enclosed, junction)?;
        let context = self.exit_free_context(&path);
        self.report(found, &context, f_end, scan_points).map(Ok)
    }

    /// Whether the count scans every piece of the path, not only where a type takes plots
    /// (docs/unit-1f.md §5.3 step 4): at ρ > 0 with a CES basket, or with the Dividend
    /// closure's transfer moving with the point, where §5.4's monotonicity does not reach.
    fn scans_everywhere(&self) -> bool {
        self.params.rho > 0.0
            && (self.workers.ces().is_some() || self.workers.rule().moving_transfer())
    }

    /// The scan of an exit-free economy's path (docs/unit-1f.md §5.3 step 4), gap by gap of 1d's
    /// sequence (the gap before `sequence[k]`): `scan` interior points of the all-human corner
    /// and of each piece of the wall in ω, by the corner form, and of each region of the line in
    /// x, each on the positive side when > 0; [0, lo] is not scanned. With the count of points.
    fn exit_free_scan(
        &self,
        path: &Path,
        scan: usize,
    ) -> Result<(Vec<Vec<bool>>, u32), SolveError> {
        let w = &self.workers;
        let env = w.machines().envelope();
        let side = |f: f64| -> Result<bool, SolveError> {
            if f.is_nan() {
                Err(SolveError::NonFinite {
                    what: "n_D - n_S at a scan point",
                })
            } else {
                Ok(f > 0.0)
            }
        };
        let corner_sides = |x: f64, technique: usize, lo: f64, hi: f64| {
            let corner = w.corner(x, technique);
            let mut out = Vec::new();
            if hi > lo {
                for s in scan_grid(lo, hi, scan) {
                    out.push(side(corner.n_d - w.supply_at(&corner, s))?);
                }
            }
            Ok::<Vec<bool>, SolveError>(out)
        };
        let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
            .chain(path.points.iter().copied())
            .chain(std::iter::once(BRACKET_HI))
            .collect();
        let omega_at = |v: f64, technique: usize| v / w.at_wage(BRACKET_HI, v, technique).p_s;
        let mut gaps = Vec::with_capacity(path.sequence.len());
        let mut count = 0u32;
        let mut region = 0;
        let mut piece = 0;
        for &(kind, _) in &path.sequence {
            let gap = match kind {
                Kind::Zero => corner_sides(0.0, env.first, 0.0, path.at_zero.v / path.at_zero.p_s)?,
                Kind::Lo | Kind::SwitchAbove(_) | Kind::WallAbove(_) => Vec::new(),
                Kind::SwitchBelow(_) | Kind::One => {
                    let t = path.techniques[region];
                    let (lo, hi) = (bounds[region], bounds[region + 1]);
                    region += 1;
                    let mut out = Vec::new();
                    if hi > lo {
                        for x in scan_grid(lo, hi, scan) {
                            out.push(side(w.at_with(x, t).excess_demand())?);
                        }
                    }
                    out
                }
                Kind::WallBelow(_) | Kind::End => {
                    let technique = match piece {
                        0 => env.last(),
                        s => w.wall_switches()[s - 1].above,
                    };
                    let lo = match piece {
                        0 => path.at_one.v / path.at_one.p_s,
                        s => omega_at(w.wall_switches()[s - 1].wage, technique),
                    };
                    let hi = match w.wall_switches().get(piece) {
                        Some(sw) => omega_at(sw.wage, technique),
                        None => path.end.omega.unwrap_or(f64::NAN),
                    };
                    piece += 1;
                    if hi.is_nan() {
                        Vec::new()
                    } else {
                        corner_sides(BRACKET_HI, technique, lo, hi)?
                    }
                }
            };
            count += gap.len() as u32;
            gaps.push(gap);
        }
        Ok((gaps, count))
    }

    /// f_0, the start of the path (docs/unit-1f.md §2.9 and §5.3 step 1): n_D(x = 0) − S(v = 0),
    /// by 1d's corner form in an economy without exit values and a fixed basket, else at the
    /// point v = 0 of the all-human corner; +∞ under a CES basket that weighs a category made
    /// of labour alone at x = 0, free at v = 0, whose content, and n_D, grow without bound
    /// there. Positive in every economy without an in-work benefit.
    pub(crate) fn start_value(&self) -> Result<f64, SolveError> {
        let w = &self.workers;
        let env = w.machines().envelope();
        if self.exit_free && w.ces().is_none() {
            let start = w.corner(0.0, env.first);
            let f = start.n_d - w.supply_at(&start, 0.0);
            return if f.is_nan() {
                Err(SolveError::NonFinite {
                    what: "n_D - n_S at the start of the path",
                })
            } else {
                Ok(f)
            };
        }
        if w.ces().is_some() {
            let land = w.machines().chain_land();
            let weights = w.weights();
            if (0..land.len()).any(|j| weights[j] > 0.0 && land[j] == 0.0) {
                return Ok(f64::INFINITY);
            }
        }
        value(
            "n_D - n_S at the start of the path",
            &self.at_wage(0.0, 0.0, env.first),
        )
    }

    /// Whether a CES basket weighs a category that embodies no labour under the wall's last
    /// technique, free relative to the rest as v/r → ∞ (docs/unit-1f.md §2.12).
    fn free_at_the_end(&self, technique: usize) -> bool {
        let w = &self.workers;
        if w.ces().is_none() {
            return false;
        }
        let corner = w.corner(BRACKET_HI, technique);
        let weights = w.weights();
        (0..weights.len()).any(|j| weights[j] > 0.0 && corner.lambda_tilde[j] == 0.0)
    }

    /// S_∞, the pool's supply at the wall's end under a CES basket with a free weighted category
    /// (docs/unit-1f.md §2.12 and §5.3 step 3): ω_∞ = 1/(Z·M(λ̃)) for σ < 1, +∞ otherwise, where
    /// n_D → 0, W, R and C over P vanish, and the transfer tends to d̂ composites (RentRate) or
    /// −μ (Dividend). No reserved hours with a CES basket, so every type is pooled.
    fn supply_at_the_end(&self, technique: usize) -> f64 {
        let w = &self.workers;
        let ces = w.ces().expect("a CES basket");
        let corner = w.corner(BRACKET_HI, technique);
        let omega = ces
            .labour_at_the_end(&corner.lambda_tilde)
            .map_or(f64::INFINITY, |l| 1.0 / l);
        let rule = w.rule();
        let delta = match rule.budget {
            Closure::RentRate { dividend } => dividend,
            Closure::Dividend { program, .. } => -program,
        };
        let mut s = 0.0;
        for t in &self.params.worker_types {
            if t.efficiency > 0.0 {
                let r = rule.ratio(t, omega, delta);
                s += t.efficiency * (t.workers * t.work_cost.cdf(num::ln1p(r)));
            }
        }
        s
    }

    /// 1d's report context for an exit-free economy.
    fn exit_free_context<'a>(&self, path: &'a Path) -> Context<'a> {
        Context {
            points: &path.points,
            at_one: &path.at_one,
            f_zero: path.f_zero,
            f_lo: path.f_lo,
            f_one: path.f_one,
            f_end: path.end.excess,
            land_at_one: self.enclosed,
        }
    }

    /// The equilibrium on the idle stretch under τ_e (docs/unit-1e.md §4.6), whose start has
    /// T_m,∞ of market land: at the junction T_m,∞ itself; without reserved hours S is fixed and
    /// T_m* = T_m,∞·S/n_D(T_m,∞); with them, bisection on the bit patterns of T_m, f rising
    /// with T_m, and the edge of a reserved shortage where it closes on a short point.
    fn idle_equilibrium(
        &self,
        technique: usize,
        start: f64,
        junction: bool,
    ) -> Result<Found, SolveError> {
        let found = |q: ParcelPoint, steps, edge| Found {
            q,
            one_minus_x: 0.0,
            steps,
            margin: Margin::Wall,
            technique,
            tie: None,
            edge,
            enclosure: None,
        };
        if junction {
            return Ok(found(self.at_idle(start, technique), 0, None));
        }
        let reserved = self.workers.reserved_per_basket().iter().any(|&r| r > 0.0);
        // Supply is fixed on the idle stretch unless the Dividend closure's transfer moves with
        // T_m (docs/unit-1f.md §5.3 step 3), which bisects as with reserved hours.
        if !reserved && !self.workers.rule().moving_on_idle_land() {
            let q = self.at_idle(start, technique);
            let market_land = start * (q.point.n_s / q.point.n_d);
            return Ok(found(self.at_idle(market_land, technique), 0, None));
        }
        let f = |t: f64| self.at_idle(t, technique).excess_demand();
        let (mut lo, mut f_lo) = (0.0_f64, f(0.0));
        let (mut hi, mut f_hi) = (start, f(start));
        if f_hi <= 0.0 {
            // f_∞ > 0 in 1d's form is within rounding of 0 here: the junction.
            return Ok(found(self.at_idle(start, technique), 0, None));
        }
        let mut steps = 0;
        while hi.to_bits() > lo.to_bits() + 1 {
            if steps >= crate::MAX_BISECTION_STEPS {
                return Err(SolveError::NoConvergence { steps });
            }
            steps += 1;
            let mid = f64::from_bits(lo.to_bits() + (hi.to_bits() - lo.to_bits()) / 2);
            let f_mid = f(mid);
            if f_mid < 0.0 {
                (lo, f_lo) = (mid, f_mid);
            } else if f_mid > 0.0 {
                (hi, f_hi) = (mid, f_mid);
            } else if f_mid == 0.0 {
                return Ok(found(self.at_idle(mid, technique), steps, None));
            } else {
                return Err(SolveError::NonFinite {
                    what: "n_D - n_S on the idle stretch",
                });
            }
        }
        if f_hi == f64::INFINITY {
            // The edge of a reserved shortage on the idle stretch (1d §12 item 16).
            let worker = self
                .workers
                .short_type(self.at_idle(hi, technique).point.short)?;
            let at = self.at_idle(lo, technique);
            let (edge, more) =
                self.workers
                    .edge_clearing(worker, (at.point.clearing[worker], f_lo), |kappa| {
                        let edge = Edge {
                            worker,
                            clearing: kappa,
                        };
                        self.at_idle_edge(lo, technique, edge).excess_demand()
                    })?;
            let q = self.at_idle_edge(lo, technique, edge);
            return Ok(found(q, steps + more, Some(edge)));
        }
        let t = if f_hi.abs() < f_lo.abs() { hi } else { lo };
        Ok(found(self.at_idle(t, technique), steps, None))
    }

    /// §5.3 for an economy with exit values.
    fn solve_priced(&self, scan: usize) -> Result<Result<Eq1e, f64>, SolveError> {
        let w = &self.workers;
        let env = w.machines().envelope();
        let at_one = self.at_with(BRACKET_HI, env.last());
        // Step 1: 1d's viability test.
        if at_one.point.d.is_nan() {
            return Err(SolveError::NonFinite { what: "D(1)" });
        }
        if at_one.point.d <= 0.0 {
            return Ok(Err(at_one.point.d));
        }
        // Step 2: the sequence.
        let f_one = value("n_D(1) - n_S(1)", &at_one)?;
        let f_lo = value(
            "n_D - n_S at BRACKET_LO",
            &self.at_with(BRACKET_LO, env.first),
        )?;
        let at_zero = self.at_with(0.0, env.first);
        let f_zero = value("n_D - n_S at 0", &at_zero)?;
        let points = w.switch_points()?;
        let techniques: Vec<usize> = std::iter::once(env.first)
            .chain(env.switches.iter().map(|s| s.above))
            .collect();
        let f_start = self.start_value()?;
        let mut path = PricedPath {
            stations: vec![(Station::Start, f_start)],
            pieces: vec![None],
            enclosures: Vec::new(),
        };
        // The all-human corner, ω in [0, ω(0)], ω = v/P_z with the fixed basket's P_z
        // (docs/unit-1f.md §5.3 step 2; P_s itself for the fixed basket without reserved hours).
        let c0 = w.corner(0.0, env.first);
        let span = Span::AllHuman {
            technique: env.first,
            l_s: c0.l_s,
            b_s: c0.b_s,
        };
        let omega_zero = at_zero.point.v / at_zero.point.reference_p_s;
        self.push_piece(
            &mut path,
            span,
            (0.0, 0.0),
            (omega_zero, at_zero.point.v),
            false,
            (Station::Zero, f_zero),
        )?;
        // [0, lo].
        let bottom = Piece {
            span: Span::Bottom {
                technique: env.first,
            },
            lo: 0.0,
            hi: BRACKET_LO,
            v_lo: f64::NAN,
            v_hi: f64::NAN,
        };
        path.push(Some(bottom), (Station::Lo, f_lo));
        // The line's regions.
        let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
            .chain(points.iter().copied())
            .chain(std::iter::once(BRACKET_HI))
            .collect();
        for (r, &t) in techniques.iter().enumerate() {
            let end = if r < points.len() {
                let what = "n_D - n_S at a switch";
                (
                    Station::SwitchBelow(r),
                    value(what, &self.at_with(points[r], t))?,
                )
            } else {
                (Station::One, f_one)
            };
            let span = Span::Line { technique: t };
            let ends = ((bounds[r], f64::NAN), (bounds[r + 1], f64::NAN));
            self.push_piece(&mut path, span, ends.0, ends.1, false, end)?;
            if r < points.len() {
                let what = "n_D - n_S at a switch";
                let above = value(what, &self.at_with(points[r], techniques[r + 1]))?;
                path.push(None, (Station::SwitchAbove(r), above));
            }
        }
        // The wall's pieces.
        let g = self.params.exit_good;
        let wall_span = |technique: usize| {
            let c = w.corner(BRACKET_HI, technique);
            Span::Wall {
                technique,
                l_s: c.l_s,
                b_s: c.b_s,
                lambda_g: c.lambda_tilde[g],
            }
        };
        let mut technique = env.last();
        let (mut v_lo, mut omega_lo) =
            (at_one.point.v, at_one.point.v / at_one.point.reference_p_s);
        for (s, sw) in w.wall_switches().iter().enumerate() {
            let what = "n_D - n_S at a wall switch";
            let below = self.at_wage(BRACKET_HI, sw.wage, sw.below);
            let omega_hi = sw.wage / below.point.reference_p_s;
            let end = (Station::WallBelow(s), value(what, &below)?);
            self.push_piece(
                &mut path,
                wall_span(technique),
                (omega_lo, v_lo),
                (omega_hi, sw.wage),
                false,
                end,
            )?;
            let above = self.at_wage(BRACKET_HI, sw.wage, sw.above);
            path.push(None, (Station::WallAbove(s), value(what, &above)?));
            technique = sw.above;
            v_lo = sw.wage;
            omega_lo = sw.wage / above.point.reference_p_s;
        }
        // The wall's last piece, to ω_∞ = 1/L_s, and the idle stretch's start.
        let last = wall_span(technique);
        let omega_end = match last {
            Span::Wall { l_s, .. } => 1.0 / l_s,
            _ => f64::NAN,
        };
        let (start, f_end) = if self.free_at_the_end(technique) {
            // A CES basket with a free weighted category: n_D → 0 on the wall's last piece, so
            // f_∞ = −S_∞, land never idles and the idle stretch is empty (docs/unit-1f.md §2.12).
            // The wall's end is then the path's end, positive when ≥ 0 as the idle stretch's is.
            let f_end = -self.supply_at_the_end(technique);
            self.push_piece(
                &mut path,
                last,
                (omega_lo, v_lo),
                (omega_end, f64::INFINITY),
                true,
                (Station::Rest, f_end),
            )?;
            (self.enclosed, f_end)
        } else {
            let plots_at_rest = self.at_idle(self.enclosed, technique).rented_plots;
            let start = self.enclosed - plots_at_rest;
            let f_end = value(
                "n_D - n_S at the end of the wall",
                &self.at_idle(start, technique),
            )?;
            self.push_piece(
                &mut path,
                last,
                (omega_lo, v_lo),
                (omega_end, f64::INFINITY),
                true,
                (Station::End, f_end),
            )?;
            // The idle stretch.
            let f_rest = value(
                "n_D - n_S at the end of the idle stretch",
                &self.at_idle(0.0, technique),
            )?;
            let idle = Piece {
                span: Span::Idle { technique },
                lo: 0.0,
                hi: start,
                v_lo: f64::NAN,
                v_hi: f64::NAN,
            };
            path.push(Some(idle), (Station::Rest, f_rest));
            (start, f_end)
        };
        // Step 3: the sides, with the scan.
        let scanning = scan > 0 && (!self.plot_takers.is_empty() || self.scans_everywhere());
        let mut sides = Vec::new();
        let mut scan_points = 0u32;
        for (k, &(station, f)) in path.stations.iter().enumerate() {
            if let (true, Some(piece)) = (scanning, path.pieces[k]) {
                if piece.span.scanned() && piece.hi > piece.lo {
                    for s in scan_grid(piece.lo, piece.hi, scan) {
                        let q = self.at_span(piece.span, s);
                        sides.push(value("n_D - n_S at a scan point", &q)? > 0.0);
                        scan_points += 1;
                    }
                }
            }
            sides.push(station.positive(f));
        }
        let changes = (0..sides.len() - 1)
            .filter(|&i| sides[i] != sides[i + 1])
            .count();
        if changes == 0 {
            if !Station::Start.positive(f_start) {
                return Err(SolveError::SurplusLabour { f_start });
            }
            return Err(SolveError::NoMarket { f_end });
        }
        if changes > 1 {
            return Err(SolveError::MultipleEquilibria {
                sign_changes: changes,
                switches: points,
            });
        }
        // Step 4: the one change lies between two consecutive values of the sequence.
        let k = (1..path.stations.len())
            .find(|&k| {
                let (a, fa) = path.stations[k - 1];
                let (b, fb) = path.stations[k];
                a.positive(fa) && !b.positive(fb)
            })
            .expect("one change of side lies between two values of the sequence");
        let found = self.locate_priced(&path, k, &techniques, &points, start)?;
        let context = Context {
            points: &points,
            at_one: &at_one.point,
            f_zero,
            f_lo,
            f_one,
            f_end,
            land_at_one: at_one.market_land,
        };
        self.report(found, &context, f_end, scan_points).map(Ok)
    }

    /// Step 4 for a priced economy: the change between stations k − 1 and k.
    fn locate_priced(
        &self,
        path: &PricedPath,
        k: usize,
        techniques: &[usize],
        points: &[f64],
        start: f64,
    ) -> Result<Found, SolveError> {
        let w = &self.workers;
        let (before, f_before) = path.stations[k - 1];
        let (after, f_after) = path.stations[k];
        let plain = |q: ParcelPoint, one_minus_x, steps, margin, technique| Found {
            q,
            one_minus_x,
            steps,
            margin,
            technique,
            tie: None,
            edge: None,
            enclosure: None,
        };
        let Some(piece) = path.pieces[k] else {
            return match (before, after) {
                (Station::EnclosureBelow(e), _) => self.enclosure_tie(&path.enclosures[e]),
                (Station::SwitchBelow(i), _) => {
                    let (below, above) = (techniques[i], techniques[i + 1]);
                    let x = points[i];
                    let q = self.at_with(x, below);
                    let (share, edge) = w.tie_share(&q.point, below, above, &q.market)?;
                    let gamma = w.machines().envelope().switches[i].gamma;
                    Ok(Found {
                        tie: Some(Tie {
                            above,
                            share,
                            gamma,
                        }),
                        edge,
                        ..plain(q, 1.0 - x, 0, Margin::Contestable, below)
                    })
                }
                (Station::WallBelow(s), _) => {
                    let sw = w.wall_switches()[s];
                    let q = self.at_wage(BRACKET_HI, sw.wage, sw.below);
                    let (share, edge) = w.tie_share(&q.point, sw.below, sw.above, &q.market)?;
                    Ok(Found {
                        tie: Some(Tie {
                            above: sw.above,
                            share,
                            gamma: sw.gamma,
                        }),
                        edge,
                        ..plain(q, 0.0, 0, Margin::Wall, sw.below)
                    })
                }
                _ => unreachable!("only a switch or an enclosure point has two values"),
            };
        };
        // The start's value is the evaluated f_0 (docs/unit-1f.md §2.9).
        let f_lo = f_before;
        match piece.span {
            Span::Idle { technique } => self.idle_equilibrium(technique, start, false),
            Span::Bottom { technique } => {
                let root = if f_after == 0.0 {
                    Root::exact(BRACKET_LO, 0)
                } else {
                    bisect_bits(
                        |x| self.at_with(x, technique).excess_demand(),
                        (0.0, f_lo),
                        (BRACKET_LO, f_after),
                    )?
                };
                let q = self.at_with(root.x, technique);
                Ok(plain(
                    q,
                    root.one_minus_x,
                    root.steps,
                    Margin::Contestable,
                    technique,
                ))
            }
            Span::Line { technique } => {
                let root = if f_after == 0.0 {
                    Root::exact(piece.hi, 0)
                } else if f_lo == 0.0 {
                    Root::exact(piece.lo, 0)
                } else {
                    bisect(
                        |x| self.at_with(x, technique).excess_demand(),
                        (piece.lo, f_lo),
                        (piece.hi, f_after),
                    )?
                };
                let q = self.at_with(root.x, technique);
                Ok(plain(
                    q,
                    root.one_minus_x,
                    root.steps,
                    Margin::Contestable,
                    technique,
                ))
            }
            Span::AllHuman { technique, .. } | Span::Wall { technique, .. } => {
                let wall = matches!(piece.span, Span::Wall { .. });
                let (v, steps) = if f_lo == 0.0 {
                    (piece.v_lo, 0)
                } else if after == Station::End && f_after == 0.0 {
                    // The junction of the wall and the idle stretch (§5.3 step 3).
                    return self.idle_equilibrium(technique, start, true);
                } else if f_after == 0.0 {
                    (piece.v_hi, 0)
                } else if w.ces().is_some() {
                    // A CES basket's corners are bisected on the bit patterns of the pool's wage
                    // itself (docs/unit-1f.md §5.3 step 2): its P is not v·L_z + B_z, so v is
                    // well conditioned, while ω_z resolves v only to about 2^-52·v·L_z/B_z near
                    // ω_z = 1/L_z. f moves with v as with ω_z, which rises with v.
                    let x = if wall { BRACKET_HI } else { 0.0 };
                    let root = bisect_bits(
                        |v| self.at_wage(x, v, technique).excess_demand(),
                        (piece.v_lo, f_lo),
                        (piece.v_hi, f_after),
                    )?;
                    (root.x, root.steps)
                } else {
                    let root = bisect_bits(
                        |omega| self.at_span(piece.span, omega).excess_demand(),
                        (piece.lo, f_lo),
                        (piece.hi, f_after),
                    )?;
                    (piece.span.wage(root.x), root.steps)
                };
                if wall {
                    let q = self.at_wage(BRACKET_HI, v, technique);
                    Ok(plain(q, 0.0, steps, Margin::Wall, technique))
                } else {
                    let technique = w.cheapest_at(v);
                    let q = self.at_wage(0.0, v, technique);
                    Ok(plain(q, 1.0, steps, Margin::AllHuman, technique))
                }
            }
        }
    }

    /// The equilibrium between an enclosure point's two values (docs/unit-1e.md §4.7): f is
    /// linear in T_p at the point's prices, so T_m* = S/(n_D per unit of market land), and
    /// ψ = (T − T_m* + T_o − G_{−i})/(h_i·E_i).
    fn enclosure_tie(&self, point: &EnclosurePoint) -> Result<Found, SolveError> {
        let i = point.worker;
        let zero = self.at_enclosure(point, EnclosureSide::Share(0.0));
        let market_land = zero.market_land * (zero.point.n_s / zero.point.n_d);
        let others = zero.plot_demand;
        let t = &self.params.worker_types[i];
        let exiters = t.workers - zero.point.supply[i];
        let plot = self.exits[i].priced.map_or(0.0, |p| p.plot);
        let share = (((self.enclosed - market_land) + self.commons - others) / (plot * exiters))
            .clamp(0.0, 1.0);
        let q = self.at_enclosure(point, EnclosureSide::Share(share));
        let one_minus_x = 1.0 - point.x;
        Ok(Found {
            q,
            one_minus_x,
            steps: 0,
            margin: point.margin,
            technique: point.technique,
            tie: None,
            edge: None,
            enclosure: Some(EnclosureTie { worker: i, share }),
        })
    }

    /// A point on a span at its parameter: ω on a corner, x on the line, T_m on the idle
    /// stretch.
    fn at_span(&self, span: Span, s: f64) -> ParcelPoint {
        match span {
            Span::AllHuman { technique, .. } => self.at_wage(0.0, span.wage(s), technique),
            Span::Bottom { technique } | Span::Line { technique } => self.at_with(s, technique),
            Span::Wall { technique, .. } => self.at_wage(BRACKET_HI, span.wage(s), technique),
            Span::Idle { technique } => self.at_idle(s, technique),
        }
    }

    /// p_g at a span's parameter, from the price side alone (r = 1).
    fn exit_good_price_at(&self, span: Span, s: f64) -> f64 {
        let (x, technique, block) = match span {
            Span::AllHuman { technique, .. } => (
                0.0,
                technique,
                self.workers.block_at_wage(span.wage(s), 1.0, technique),
            ),
            Span::Wall { technique, .. } => (
                BRACKET_HI,
                technique,
                self.workers.block_at_wage(span.wage(s), 1.0, technique),
            ),
            Span::Bottom { technique } | Span::Line { technique } | Span::Idle { technique } => {
                (s, technique, self.workers.block_at(s, technique))
            }
        };
        self.workers
            .price_side(x, technique, block, 1.0)
            .base_prices[self.params.exit_good]
    }

    /// Whether plot-taking type i is on its floor at the market rent r = 1 at a span's
    /// parameter: r·h_i ≥ p_g·Δ_i, the complement of §5.1's branch test.
    fn on_floor(&self, span: Span, s: f64, i: usize) -> bool {
        let e = &self.exits[i];
        let p = e.priced.expect("a plot taker is priced");
        let p_g = self.exit_good_price_at(span, s);
        let plot = 1.0 * p.plot < p_g * e.advantage;
        !plot
    }

    /// Appends a piece of the path from `lo` to `hi` (each with the pool's wage there on a
    /// corner) and its end value, with each enclosure point inside it as a pair of values
    /// (§4.7): where type i is on its floor at `lo` and renting at `hi` (at the wall's end,
    /// renting when the exit good's price grows without bound), the largest double of the
    /// parameter at which it is on its floor, by bisection on bit patterns.
    fn push_piece(
        &self,
        path: &mut PricedPath,
        span: Span,
        (lo, v_lo): (f64, f64),
        (hi, v_hi): (f64, f64),
        to_the_end: bool,
        end: (Station, f64),
    ) -> Result<(), SolveError> {
        let mut found: Vec<(f64, usize)> = Vec::new();
        if span.scanned() {
            for &i in &self.plot_takers {
                let floor_lo = self.on_floor(span, lo, i);
                let floor_hi = match span {
                    Span::Wall { lambda_g, .. } if to_the_end => floor_lo && lambda_g <= 0.0,
                    _ => self.on_floor(span, hi, i),
                };
                if floor_lo && !floor_hi {
                    let (mut a, mut b) = (lo, hi);
                    while b.to_bits() > a.to_bits() + 1 {
                        let mid = f64::from_bits(a.to_bits() + (b.to_bits() - a.to_bits()) / 2);
                        if self.on_floor(span, mid, i) {
                            a = mid;
                        } else {
                            b = mid;
                        }
                    }
                    found.push((a, i));
                }
            }
            found.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        }
        let (mut from, mut v_from) = (lo, v_lo);
        for (s, i) in found {
            let point = match span {
                Span::Line { technique } => EnclosurePoint {
                    worker: i,
                    margin: Margin::Contestable,
                    x: s,
                    v: self.workers.block_at(s, technique).v,
                    technique,
                },
                Span::AllHuman { technique, .. } => EnclosurePoint {
                    worker: i,
                    margin: Margin::AllHuman,
                    x: 0.0,
                    v: span.wage(s),
                    technique,
                },
                Span::Wall { technique, .. } => EnclosurePoint {
                    worker: i,
                    margin: Margin::Wall,
                    x: BRACKET_HI,
                    v: span.wage(s),
                    technique,
                },
                _ => unreachable!("only the corners and the line are searched"),
            };
            let what = "n_D - n_S at an enclosure point";
            let below = value(what, &self.at_enclosure(&point, EnclosureSide::Below))?;
            let above = value(what, &self.at_enclosure(&point, EnclosureSide::Above))?;
            let e = path.enclosures.len();
            path.enclosures.push(point);
            let piece = Piece {
                span,
                lo: from,
                hi: s,
                v_lo: v_from,
                v_hi: point.v,
            };
            path.push(Some(piece), (Station::EnclosureBelow(e), below));
            path.push(None, (Station::EnclosureAbove(e), above));
            (from, v_from) = (s, point.v);
        }
        let piece = Piece {
            span,
            lo: from,
            hi,
            v_lo: v_from,
            v_hi,
        };
        path.push(Some(piece), end);
        Ok(())
    }

    /// Every enclosure point along the path of a priced economy, in path order
    /// (docs/unit-1e.md §4.7); empty for an economy without plot-taking types, and for one that
    /// is not viable.
    pub fn enclosure_points(&self) -> Result<Vec<EnclosurePoint>, SolveError> {
        if self.plot_takers.is_empty() {
            return Ok(Vec::new());
        }
        let w = &self.workers;
        let env = w.machines().envelope();
        let d = self.at_with(BRACKET_HI, env.last()).point.d;
        if d.is_nan() || d <= 0.0 {
            return Ok(Vec::new());
        }
        let mut path = PricedPath {
            stations: Vec::new(),
            pieces: Vec::new(),
            enclosures: Vec::new(),
        };
        let at_zero = self.at_with(0.0, env.first);
        let c0 = w.corner(0.0, env.first);
        let span = Span::AllHuman {
            technique: env.first,
            l_s: c0.l_s,
            b_s: c0.b_s,
        };
        let omega_zero = at_zero.point.v / at_zero.point.p_s;
        self.push_piece(
            &mut path,
            span,
            (0.0, 0.0),
            (omega_zero, at_zero.point.v),
            false,
            (Station::Zero, 0.0),
        )?;
        let points = w.switch_points()?;
        let techniques: Vec<usize> = std::iter::once(env.first)
            .chain(env.switches.iter().map(|s| s.above))
            .collect();
        let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
            .chain(points.iter().copied())
            .chain(std::iter::once(BRACKET_HI))
            .collect();
        for (r, &t) in techniques.iter().enumerate() {
            let ends = ((bounds[r], f64::NAN), (bounds[r + 1], f64::NAN));
            self.push_piece(
                &mut path,
                Span::Line { technique: t },
                ends.0,
                ends.1,
                false,
                (Station::One, 0.0),
            )?;
        }
        let g = self.params.exit_good;
        let wall_span = |technique: usize| {
            let c = w.corner(BRACKET_HI, technique);
            Span::Wall {
                technique,
                l_s: c.l_s,
                b_s: c.b_s,
                lambda_g: c.lambda_tilde[g],
            }
        };
        let at_one = self.at_with(BRACKET_HI, env.last());
        let mut technique = env.last();
        let mut omega_lo = at_one.point.v / at_one.point.p_s;
        for sw in w.wall_switches() {
            let below = self.at_wage(BRACKET_HI, sw.wage, sw.below);
            let omega_hi = sw.wage / below.point.p_s;
            self.push_piece(
                &mut path,
                wall_span(technique),
                (omega_lo, f64::NAN),
                (omega_hi, sw.wage),
                false,
                (Station::One, 0.0),
            )?;
            technique = sw.above;
            omega_lo = sw.wage / self.at_wage(BRACKET_HI, sw.wage, sw.above).point.p_s;
        }
        let last = wall_span(technique);
        let omega_end = match last {
            Span::Wall { l_s, .. } => 1.0 / l_s,
            _ => f64::NAN,
        };
        self.push_piece(
            &mut path,
            last,
            (omega_lo, f64::NAN),
            (omega_end, f64::INFINITY),
            true,
            (Station::End, 0.0),
        )?;
        Ok(path.enclosures)
    }
}

/// The top of a scan on bit patterns, where a wall piece ends at ω = +∞ (the basket embodies
/// no labour at the wall's end): 2^512. Above it the pool's wage ω·B_s and the prices it sets
/// can overflow, while f is already within rounding of its limit f_∞.
const OMEGA_SCAN_TOP: f64 = f64::from_bits((1023 + 512) << 52);

/// `scan` points strictly inside [lo, hi], equally spaced, or on the bit patterns of
/// nonnegative doubles up to [`OMEGA_SCAN_TOP`] when hi is +∞ (docs/unit-1e.md §5.3 step 2).
fn scan_grid(lo: f64, hi: f64, scan: usize) -> Vec<f64> {
    let cells = (scan + 1) as f64;
    if hi.is_finite() {
        (1..=scan)
            .map(|k| lo + (hi - lo) * (k as f64 / cells))
            .collect()
    } else {
        let (a, b) = ((lo + 0.0).to_bits(), OMEGA_SCAN_TOP.max(lo).to_bits());
        let span = (b - a) as f64;
        (1..=scan)
            .map(|k| f64::from_bits(a + (span * (k as f64 / cells)) as u64))
            .collect()
    }
}

/// The count of an exit-free economy's path (docs/unit-1f.md §5.3 steps 3-4): `sides` holds the
/// side of the start, of each value of 1d's sequence and of the idle stretch's end, and
/// `gaps[k]` the scanned sides between `sides[k]` and `sides[k + 1]` (none after the last value
/// of the sequence). `Ok(c)` where the whole counted sequence, the scan's sides among it,
/// changes side once, between `sides[c]` and `sides[c + 1]`; else `Err` with its number of
/// changes, 0 or more than one. The scan only adds points inside the gaps, so a single change
/// of the counted sequence is a single change of `sides`.
fn count_changes(sides: &[bool], gaps: &[Vec<bool>]) -> Result<usize, usize> {
    let mut counted = vec![sides[0]];
    for (k, &side) in sides.iter().enumerate().skip(1) {
        if let Some(gap) = gaps.get(k - 1) {
            counted.extend(gap);
        }
        counted.push(side);
    }
    let changes = (0..counted.len() - 1)
        .filter(|&i| counted[i] != counted[i + 1])
        .count();
    if changes != 1 {
        return Err(changes);
    }
    Ok((0..sides.len() - 1)
        .find(|&i| sides[i] != sides[i + 1])
        .expect("the counted sequence's one change is one of the sides'"))
}

/// The land and exit side of an equilibrium, as [`ParcelEconomy::extend`] reads it.
struct LandSide {
    rent: f64,
    market_land: f64,
    exit_good_price: f64,
    q: f64,
    exit_land: ExitLand,
    plot_rent: f64,
    branches: Vec<Branch>,
    values: Vec<f64>,
    goods: Vec<f64>,
    households: Vec<f64>,
    plots: Vec<f64>,
    commons_occupied: f64,
    rented: f64,
}

impl ParcelPoint {
    fn land_side(&self) -> LandSide {
        LandSide {
            rent: self.rent,
            market_land: self.market_land,
            exit_good_price: self.exit_good_price,
            q: self.q,
            exit_land: self.exit_land,
            plot_rent: self.plot_rent,
            branches: self.branches.clone(),
            values: self.exit_values.clone(),
            goods: self.exit_goods.clone(),
            households: self.plot_households.clone(),
            plots: self.plots.clone(),
            commons_occupied: self.commons_occupied,
            rented: self.rented_plots,
        }
    }
}

impl<S: Schedule + Clone> ParcelEconomy<S> {
    /// The report of a found equilibrium: unit 1d's on the point's market, extended.
    fn report(
        &self,
        found: Found,
        context: &Context,
        f_end: f64,
        scan_points: u32,
    ) -> Result<Eq1e, SolveError> {
        let at = Reported {
            q: &found.q.point,
            one_minus_x: found.one_minus_x,
            steps: found.steps,
            margin: found.margin,
        };
        let eq = self.workers.report_at(
            &at,
            found.technique,
            found.tie,
            found.edge,
            context,
            &found.q.market,
        )?;
        let side = found.q.land_side();
        Ok(self.extend(eq, &side, found.enclosure, f_end, scan_points))
    }

    /// The land side of an exit-free equilibrium, which unit 1d's report does not carry: the
    /// whole endowment in use at r = 1, no plots.
    fn free_side(&self, eq: &Eq1d) -> LandSide {
        let state = self.free_state();
        LandSide {
            rent: 1.0,
            market_land: self.enclosed,
            exit_good_price: eq.categories[self.params.exit_good].price,
            q: 1.0 / eq.categories[self.params.exit_good].price,
            exit_land: state.regime,
            plot_rent: state.plot_rent,
            branches: state.branches,
            values: state.values,
            goods: state.goods,
            households: state.households,
            plots: state.plots,
            commons_occupied: 0.0,
            rented: 0.0,
        }
    }

    /// docs/unit-1e.md §4.8: unit 1d's report with the land, the parcels, the exit per type,
    /// the home account and coverage.
    fn extend(
        &self,
        base: Eq1d,
        side: &LandSide,
        enclosure: Option<EnclosureTie>,
        f_end: f64,
        scan_points: u32,
    ) -> Eq1e {
        let types = &self.params.worker_types;
        let (t, t_o) = (self.enclosed, self.commons);
        let rent = side.rent;
        let scarce = rent == 1.0;
        let idle = if scarce {
            0.0
        } else {
            (t - side.market_land) - side.rented
        };
        let land = LandEq {
            enclosed: t,
            commons: t_o,
            market: side.market_land,
            rented_plots: side.rented,
            idle,
            commons_occupied: side.commons_occupied,
        };
        // The parcels, best first (§2.1): enclosed ones take T_m + T_p, open ones T_oc.
        let mut parcels = vec![
            ParcelEq {
                rent_per_acre: 0.0,
                shadow_rent_per_acre: 0.0,
                used: 0.0,
            };
            self.params.parcels.len()
        ];
        let fill = |order: &[usize], total: f64, whole: bool, parcels: &mut Vec<ParcelEq>| {
            let mut left = total;
            for &z in order {
                let parcel = &self.params.parcels[z];
                let capacity = parcel.acreage * parcel.quality;
                parcels[z].used = if capacity == 0.0 {
                    0.0
                } else if whole {
                    1.0
                } else {
                    let take = capacity.min(left.max(0.0));
                    left -= take;
                    take / capacity
                };
            }
        };
        fill(
            &self.enclosed_order,
            side.market_land + side.rented,
            scarce,
            &mut parcels,
        );
        fill(
            &self.open_order,
            side.commons_occupied,
            side.commons_occupied == t_o,
            &mut parcels,
        );
        for (z, parcel) in self.params.parcels.iter().enumerate() {
            match parcel.access {
                Access::Enclosed => parcels[z].rent_per_acre = rent * parcel.quality,
                Access::Open => {
                    parcels[z].shadow_rent_per_acre = side.plot_rent * parcel.quality;
                }
            }
        }
        // Each type's exit, and the home account (§4.8).
        let p_g = side.exit_good_price;
        let mut total_plots = 0.0;
        for &plot in &side.plots {
            total_plots += plot;
        }
        let rented_share = if total_plots > 0.0 {
            side.rented / total_plots
        } else {
            0.0
        };
        let (mut output, mut floor) = (0.0, 0.0);
        let mut workers = Vec::with_capacity(types.len());
        for (i, ty) in types.iter().enumerate() {
            let exiters = ty.workers - base.workers[i].supply;
            let e = &self.exits[i];
            if let Some(p) = e.priced {
                output += p_g * p.gross * side.households[i];
                floor += p_g * p.floor * (exiters - side.households[i]);
            }
            workers.push(WorkerEq1e {
                branch: side.branches[i],
                exit_value: side.values[i],
                exit_goods: side.goods[i],
                threshold: e.threshold,
                exiters,
                plot_households: side.households[i],
                plot_land: side.plots[i],
                rented_land: side.plots[i] * rented_share,
            });
        }
        let home = HomeAccount {
            output,
            rent_in_kind: rent * side.rented,
            floor,
            commons_shadow_rent: side.plot_rent * side.commons_occupied,
        };
        let mut people = 0.0;
        for ty in types {
            people += ty.workers;
        }
        let coverage = (rent * t) / (people * base.p_s);
        let partition = (((t - land.market) - land.rented_plots) - land.idle).abs() / t;
        let commons = if side.exit_land == ExitLand::Crowded {
            (total_plots - t_o).abs() / t_o
        } else {
            0.0
        };
        Eq1e {
            land_market: if scarce {
                LandMarket::Scarce
            } else {
                LandMarket::Idle
            },
            rent,
            exit_land: side.exit_land,
            plot_rent: side.plot_rent,
            q: side.q,
            exit_good_price: p_g,
            land,
            parcels,
            coverage,
            enclosure,
            home,
            workers,
            f_end: finite_or_none(f_end),
            scan_points,
            certified: self.certified,
            residuals: Residuals1e { partition, commons },
            base,
        }
    }
}

/// The land at an equilibrium (docs/unit-1e.md §4.1 and §4.8), in units of land service.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LandEq {
    /// T, the enclosed parcels' services.
    pub enclosed: f64,
    /// T_o, the commons.
    pub commons: f64,
    /// T_m, the market's land in use: Y·B^q.
    pub market: f64,
    /// T_p, enclosed land in exit plots.
    pub rented_plots: f64,
    /// T_idle = T − T_m − T_p, enclosed land idle at zero rent (0 while land is scarce).
    pub idle: f64,
    /// T_oc, the commons occupied by exit plots.
    pub commons_occupied: f64,
}

/// One parcel at an equilibrium (docs/unit-1e.md §2.1 and §4.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParcelEq {
    /// r·Q_z for an enclosed parcel (0 on idle land at r = 0); 0 for an open one.
    pub rent_per_acre: f64,
    /// r_o·Q_z for an open parcel, the commons' shadow rent, which nobody receives; 0 for an
    /// enclosed one.
    pub shadow_rent_per_acre: f64,
    /// The share of its services in use, best parcels first: a convention where land idles
    /// (§2.1), whose totals are what count.
    pub used: f64,
}

/// The exit life's accounts, in kind and outside the market's (docs/unit-1e.md §2.6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HomeAccount {
    /// Σ p_g·s₀,i over the plot households.
    pub output: f64,
    /// r·T_p, the rent plots pay in kind.
    pub rent_in_kind: f64,
    /// Σ p_g·s̲_i over the priced types' floor households.
    pub floor: f64,
    /// r_o·T_oc, the commons' shadow rent.
    pub commons_shadow_rent: f64,
}

/// One worker type's exit at an equilibrium (docs/unit-1e.md §4.8); the rest of the type is
/// in [`Eq1d::workers`] of [`Eq1e::base`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkerEq1e {
    /// Its branch (at an enclosure tie, the type's is `Floor`, with ψ of its exiters on plots).
    pub branch: Branch,
    /// e_i, the exit life's money value.
    pub exit_value: f64,
    /// s_i, the exit life's value in the exit good.
    pub exit_goods: f64,
    /// q_enc,i, for a plot-taking type.
    pub threshold: Option<f64>,
    /// E_i = N_i − n_S,i.
    pub exiters: f64,
    /// Its exiters on plots.
    pub plot_households: f64,
    /// Its plot land, on the commons and rented.
    pub plot_land: f64,
    /// Its plot land on enclosed land, in proportion to its plots.
    pub rented_land: f64,
}

/// What unit 1e adds to unit 1d's residuals (docs/unit-1e.md §6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Residuals1e {
    /// |T − T_m − T_p − T_idle|/T.
    pub partition: f64,
    /// |Σ plots − T_o|/T_o in a crowded commons, 0 otherwise.
    pub commons: f64,
}

/// An equilibrium of unit 1e (docs/unit-1e.md §4.8): unit 1d's outputs in the numeraire's
/// units (per unit of rent while land is scarce, per unit of the pool's wage on idle land), on
/// the market's land, and the land, the parcels and the exit.
#[derive(Clone, Debug, PartialEq)]
pub struct Eq1e {
    /// Unit 1d's equilibrium on the market's land. Its `f_end` is +∞ where the wall's end is
    /// short; [`Eq1e::f_end`] is the output.
    pub base: Eq1d,
    /// Scarce (r = 1, the rent the numeraire) or idle (r = 0, the pool's wage).
    pub land_market: LandMarket,
    /// r, 1 or 0.
    pub rent: f64,
    /// Where the exit plots stand.
    pub exit_land: ExitLand,
    /// r_o, the rent an exit plot pays per unit of service.
    pub plot_rent: f64,
    /// q = r/p_g; at r = 0 its limit at the wall's end (0, or 1/b̃_g for an exit good that
    /// embodies no labour: docs/unit-1e.md §12 item 16).
    pub q: f64,
    /// p_g.
    pub exit_good_price: f64,
    /// The land.
    pub land: LandEq,
    /// Every parcel, in order.
    pub parcels: Vec<ParcelEq>,
    /// κ = r·T/(Σ_i N_i·P_s), coverage with the whole enclosed rent base (SSRN eq 16).
    pub coverage: f64,
    /// The enclosure tie, when the equilibrium sits at an enclosure point.
    pub enclosure: Option<EnclosureTie>,
    /// The exit life's accounts in kind.
    pub home: HomeAccount,
    /// Every worker type's exit, in order.
    pub workers: Vec<WorkerEq1e>,
    /// f_∞, the excess demand at the wall's end; `None` where it is short.
    pub f_end: Option<f64>,
    /// How many scan points the count used.
    pub scan_points: u32,
    /// Whether §5.4 certifies the count.
    pub certified: bool,
    /// The residuals unit 1e adds; unit 1d's are [`Eq1d::residuals`].
    pub residuals: Residuals1e,
}

impl Eq1e {
    /// Every output: unit 1d's keys in 1d's order (with `f_end` optional), then unit 1e's
    /// economy keys, each parcel's (`parcel<z>.`), and each worker type's exit (under
    /// `worker<i>.`).
    ///
    /// This is the one list of outputs: [`ParcelEconomy::solve`] checks every number in it for
    /// finiteness.
    pub fn outputs(&self) -> Vec<(OutputKey1c, Output1b)> {
        use Output1b::{Count, Flag, Float, Optional};
        // On the idle stretch a machine type whose recipes use no labour is free: the pool's
        // wage in machine-task units, and labour's share of its price, are then undefined.
        let machines_free = self.base.types[self.base.technique].price == 0.0;
        let mut list: Vec<(OutputKey1c, Output1b)> = self
            .base
            .outputs()
            .into_iter()
            .map(|(key, output)| {
                let free = match key.item {
                    Item::Economy => machines_free && matches!(key.name, "g" | "phi_w" | "phi_r"),
                    Item::Category(j) => {
                        self.base.categories[j].price == 0.0
                            && matches!(key.name, "real_wage" | "wage_floor" | "phi_w" | "phi_r")
                    }
                    Item::Type(k) => self.base.types[k].price == 0.0 && key.name == "phi_w",
                    _ => false,
                };
                if key.item == Item::Economy && key.name == "f_end" {
                    (key, Optional(self.f_end))
                } else if free {
                    (key, Optional(None))
                } else {
                    (key, output)
                }
            })
            .collect();
        let top = |name| OutputKey1c {
            item: Item::Economy,
            name,
        };
        let l = &self.land;
        let h = &self.home;
        list.extend([
            (top("land_market"), Count(self.land_market.code())),
            (top("rent"), Float(self.rent)),
            (top("exit_land"), Count(self.exit_land.code())),
            (top("plot_rent"), Float(self.plot_rent)),
            (top("q"), Float(self.q)),
            (top("exit_good_price"), Float(self.exit_good_price)),
            (top("land_enclosed"), Float(l.enclosed)),
            (top("land_commons"), Float(l.commons)),
            (top("market_land"), Float(l.market)),
            (top("rented_plots"), Float(l.rented_plots)),
            (top("idle_land"), Float(l.idle)),
            (top("commons_occupied"), Float(l.commons_occupied)),
            (top("coverage"), Float(self.coverage)),
            (top("enclosure"), Flag(self.enclosure.is_some())),
            (
                top("enclosure_worker"),
                Optional(self.enclosure.map(|e| e.worker as f64)),
            ),
            (
                top("enclosure_share"),
                Optional(self.enclosure.map(|e| e.share)),
            ),
            (top("home_output"), Float(h.output)),
            (top("home_rent_in_kind"), Float(h.rent_in_kind)),
            (top("home_floor"), Float(h.floor)),
            (
                top("home_commons_shadow_rent"),
                Float(h.commons_shadow_rent),
            ),
            (top("scan_points"), Count(self.scan_points)),
            (top("certified"), Flag(self.certified)),
            (top("res_partition"), Float(self.residuals.partition)),
            (top("res_commons"), Float(self.residuals.commons)),
        ]);
        for (z, p) in self.parcels.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Parcel(z),
                name,
            };
            list.extend([
                (key("rent_per_acre"), Float(p.rent_per_acre)),
                (key("shadow_rent_per_acre"), Float(p.shadow_rent_per_acre)),
                (key("used"), Float(p.used)),
            ]);
        }
        for (i, w) in self.workers.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Worker(i),
                name,
            };
            list.extend([
                (key("branch"), Count(w.branch.code())),
                (key("exit_value"), Float(w.exit_value)),
                (key("exit_goods"), Float(w.exit_goods)),
                (key("threshold"), Optional(w.threshold)),
                (key("exiters"), Float(w.exiters)),
                (key("plot_households"), Float(w.plot_households)),
                (key("plot_land"), Float(w.plot_land)),
                (key("rented_land"), Float(w.rented_land)),
            ]);
        }
        list
    }
}

/// The error for the first number in [`Eq1e::outputs`] that is NaN or infinite.
fn first_non_finite(eq: &Eq1e) -> Option<SolveError> {
    eq.outputs()
        .into_iter()
        .find_map(|(key, output)| match output {
            Output1b::Float(v) | Output1b::Optional(Some(v)) if !v.is_finite() => {
                Some(match key.item {
                    Item::Economy | Item::Switch(_) | Item::WallSwitch(_) | Item::Parcel(_) => {
                        SolveError::NonFinite { what: key.name }
                    }
                    Item::Type(machine_type) => SolveError::NonFiniteInType {
                        machine_type,
                        what: key.name,
                    },
                    Item::Category(category) => SolveError::NonFiniteInCategory {
                        category,
                        what: key.name,
                    },
                    Item::Worker(worker) => SolveError::NonFiniteInWorker {
                        worker,
                        what: key.name,
                    },
                })
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::categories::CategoryParams;
    use crate::machines::MachineParams;
    use crate::params::{Params, UniformWorkCost};

    /// G1 in parcel form (docs/unit-1e.md §3.3) with the parcels and exit given.
    fn g1(parcels: Vec<Parcel>, exit: ExitForm) -> ParcelEconomy {
        let params = Params {
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
        };
        let mut p = ParcelParams::from_workers(WorkerParams::from_machines(
            MachineParams::from_categories(CategoryParams::from_one_category(params)),
        ));
        p.parcels = parcels;
        p.exits = vec![exit];
        ParcelEconomy::new(p).unwrap()
    }

    fn land(acreage: f64, access: Access) -> Parcel {
        Parcel {
            acreage,
            quality: 1.0,
            access,
        }
    }

    fn priced(gross: f64, floor: f64, plot: f64) -> ExitForm {
        ExitForm::Priced(PricedExit { gross, floor, plot })
    }

    /// G1 with a commons of the given services and exit (s₀, s̲, h).
    fn with_commons(commons: f64, exit: ExitForm) -> ParcelEconomy {
        g1(
            vec![land(10.0, Access::Enclosed), land(commons, Access::Open)],
            exit,
        )
    }

    #[test]
    fn the_exit_sub_problem_has_four_regimes() {
        // At fixed prices G(r_o) = h·E(r_o): no plots when every exiter works; the commons with
        // room; crowded; spilling onto enclosed land.
        let (v, p_s, p_g) = (0.55, 1.36, 0.36);
        let exit = priced(0.5, 0.0, 0.1);
        let big = with_commons(1.0, exit);
        let t0 = big.trial(0.0, (v, p_s, 0.0), p_g, Force::None, None);
        let t1 = big.trial(1.0, (v, p_s, 0.0), p_g, Force::None, None);
        assert!(t1.demand < t0.demand && t0.demand < 0.4);
        let s = big.exit_state((v, p_s, 0.0), p_g, 1.0, Force::None, None);
        assert_eq!(
            (s.regime, s.plot_rent, s.rented),
            (ExitLand::Commons, 0.0, 0.0)
        );
        assert_eq!(s.commons_occupied, t0.demand);
        let crowded = with_commons(0.5 * (t0.demand + t1.demand), exit);
        let s = crowded.exit_state((v, p_s, 0.0), p_g, 1.0, Force::None, None);
        assert_eq!(s.regime, ExitLand::Crowded);
        assert!(s.plot_rent > 0.0 && s.plot_rent < 1.0);
        // r_o is the least double with G ≤ T_o
        let t_o = crowded.commons();
        assert!(
            crowded
                .trial(s.plot_rent, (v, p_s, 0.0), p_g, Force::None, None)
                .demand
                <= t_o
        );
        let before = s.plot_rent.next_down();
        assert!(
            crowded
                .trial(before, (v, p_s, 0.0), p_g, Force::None, None)
                .demand
                > t_o
        );
        let small = with_commons(0.5 * t1.demand, exit);
        let s = small.exit_state((v, p_s, 0.0), p_g, 1.0, Force::None, None);
        assert_eq!((s.regime, s.plot_rent), (ExitLand::Enclosed, 1.0));
        assert_eq!(s.rented, t1.demand - small.commons());
        // everyone works at a high enough wage: no plot is asked for
        let s = big.exit_state((50.0, p_s, 0.0), p_g, 1.0, Force::None, None);
        assert_eq!((s.regime, s.demand), (ExitLand::Unused, 0.0));
        // at r = 0 plots are free everywhere, the excess over the commons on idle land: not
        // enclosure, since suitable land idles
        let s = small.exit_state((v, p_s, 0.0), p_g, 0.0, Force::None, None);
        assert_eq!((s.regime, s.plot_rent), (ExitLand::Idle, 0.0));
        assert_eq!(s.rented, t0.demand - small.commons());
    }

    #[test]
    fn a_free_exit_good_is_decided_at_the_wall() {
        // §12 item 16: at r = 0 an exit good that embodies no labour costs 0, and q = r/p_g is
        // 0/0. The plots are decided as at the wall's end, where the good costs p_wall = b̃_g
        // per unit of rent (q = 1/p_wall), while every exit value in money is 0.
        let (v, p_s, p_wall) = (1.0, 2.0, 1.0);
        // a plot is worth its land there (h = 0.1 < p_wall·Δ = 0.5): the type rents, and the
        // spill past the commons stands free on idle land
        let rents = with_commons(0.01, priced(0.5, 0.0, 0.1));
        let s = rents.exit_state((v, p_s, 0.0), 0.0, 0.0, Force::None, Some(p_wall));
        assert_eq!((s.regime, s.plot_rent), (ExitLand::Idle, 0.0));
        assert_eq!((s.branches[0], s.values[0]), (Branch::Plot, 0.0));
        assert_eq!(s.goods[0], num::fma(-(1.0 / p_wall), 0.1, 0.5));
        assert_eq!(s.rented, s.demand - 0.01);
        // without the frame the type would stand on its floor at p_g = 0 (0 < 0 is false),
        // which is not where the wall's last piece leaves it
        let plain = rents.exit_state((v, p_s, 0.0), 0.0, 0.0, Force::None, None);
        assert_eq!(plain.branches[0], Branch::Floor);
        // a plot not worth its land there (h = 1 ≥ 0.5): the floor at the wall's rent, a plot
        // on the commons at a low enough shadow rent, so a small commons is crowded, rationed
        // at the type's drop ρ = p_wall·Δ/h = 0.5 in the wall's units, and 0 in money
        let floor = with_commons(0.01, priced(0.5, 0.0, 1.0));
        let s = floor.exit_state((v, p_s, 0.0), 0.0, 0.0, Force::None, Some(p_wall));
        assert_eq!(
            (s.regime, s.plot_rent, s.rented),
            (ExitLand::Crowded, 0.0, 0.0)
        );
        assert_eq!((s.branches[0], s.values[0]), (Branch::Floor, 0.0));
        assert_eq!(s.plots[0], 0.01);
        // with no commons every exiter stands on the floor, as on the wall's last piece
        let none = g1(vec![land(10.0, Access::Enclosed)], priced(0.5, 0.0, 1.0));
        let s = none.exit_state((v, p_s, 0.0), 0.0, 0.0, Force::None, Some(p_wall));
        assert_eq!(
            (s.branches[0], s.rented, s.demand),
            (Branch::Floor, 0.0, 0.0)
        );
    }

    #[test]
    fn a_crowded_commons_splits_a_type_at_its_drop() {
        // With q_enc inside [0, r] (p_g·Δ/h < 1), G drops to 0 at r_o* = p_g·Δ/h; a commons
        // between the two limits is filled by a share of the type's exiters on plots, the rest
        // on the floor, at r_o = r_o* (to a double or two), the supply the same either way.
        let (v, p_s, p_g) = (0.55, 1.36, 0.36);
        let exit = priced(0.5, 0.2, 0.5);
        let drop: f64 = p_g * 0.3 / 0.5;
        let probe = with_commons(1.0, exit);
        let left = probe
            .trial(
                drop.next_down().next_down(),
                (v, p_s, 0.0),
                p_g,
                Force::None,
                None,
            )
            .demand;
        assert!(left > 0.0);
        let right = probe
            .trial(
                drop.next_up().next_up(),
                (v, p_s, 0.0),
                p_g,
                Force::None,
                None,
            )
            .demand;
        assert_eq!(right, 0.0);
        let split = with_commons(0.5 * left, exit);
        let s = split.exit_state((v, p_s, 0.0), p_g, 1.0, Force::None, None);
        assert_eq!(s.regime, ExitLand::Crowded);
        assert!((s.plot_rent - drop).abs() <= 4.0 * f64::EPSILON * drop);
        assert_eq!(s.branches[0], Branch::Floor);
        assert_eq!(s.plots[0], split.commons());
        assert_eq!(s.demand, split.commons());
        assert!((s.households[0] * 0.5 - split.commons()).abs() <= 1e-16);
    }

    #[test]
    fn the_enclosure_tie_is_the_bisection_root() {
        // §4.7: f is linear in T_p at the point's prices, so T_m* = S/(n_D per unit of market
        // land) is the root of f(ψ), which bisection finds too (Q2).
        let e = g1(vec![land(100.0, Access::Enclosed)], priced(1.5, 0.0, 1.0));
        let mut p = e.params().clone();
        p.worker_types[0].workers = 60.0;
        p.schedule.eta = 2.0;
        let e = ParcelEconomy::new(p).unwrap();
        let points = e.enclosure_points().unwrap();
        assert_eq!(points.len(), 1);
        let found = e.enclosure_tie(&points[0]).unwrap();
        let share = found.enclosure.unwrap().share;
        let f = |psi: f64| {
            e.at_enclosure(&points[0], EnclosureSide::Share(psi))
                .excess_demand()
        };
        let root = bisect(f, (0.0, f(0.0)), (1.0, f(1.0))).unwrap();
        assert!((root.x - share).abs() <= 1e-13, "{} {}", root.x, share);
    }

    #[test]
    fn the_idle_equilibrium_is_the_bisection_root() {
        // §4.6: without walled types S is fixed on the idle stretch and f linear in T_m (W2).
        let e = g1(vec![land(10.0, Access::Enclosed)], ExitForm::Dependence);
        let mut p = e.params().clone();
        p.worker_types[0].workers = 0.25;
        let e = ParcelEconomy::new(p).unwrap();
        let t = e.workers().wall_end().technique;
        let found = e.idle_equilibrium(t, 10.0, false).unwrap();
        let f = |m: f64| -e.at_idle(m, t).excess_demand();
        let root = bisect_bits(f, (0.0, f(0.0)), (10.0, f(10.0))).unwrap();
        let closed = found.q.market_land;
        assert!((root.x - closed).abs() <= 4.0 * f64::EPSILON * closed);
        assert!((closed - 47.0 / 6.0).abs() <= 1e-14 * closed);
    }

    #[test]
    fn the_scan_grid() {
        // equally spaced strictly inside a finite piece
        assert_eq!(scan_grid(1.0, 2.0, 3), vec![1.25, 1.5, 1.75]);
        // on bit patterns up to 2^512 where the piece ends at +∞
        let grid = scan_grid(1.0, f64::INFINITY, 7);
        assert_eq!(grid.len(), 7);
        assert!(grid.windows(2).all(|w| w[0] < w[1]));
        assert!(grid[0] > 1.0 && grid[6] < OMEGA_SCAN_TOP);
        assert_eq!(OMEGA_SCAN_TOP, crate::pow2(512));
        // equally spaced bits: each step multiplies ω by about the same factor
        let steps: Vec<u64> = grid
            .windows(2)
            .map(|w| w[1].to_bits() - w[0].to_bits())
            .collect();
        assert!(steps.iter().all(|&d| d.abs_diff(steps[0]) <= 1));
    }

    #[test]
    fn the_exit_free_count_reads_the_scan() {
        // docs/unit-1f.md §5.3 step 4, §14 item 19: the sequence alone changes side once, from
        // the start's value to the next; a scanned pair of changes inside that gap makes three,
        // which are MultipleEquilibria, not the sequence's one.
        let sides = [true, true, false];
        assert_eq!(count_changes(&sides, &[vec![]]), Ok(1));
        assert_eq!(count_changes(&sides, &[vec![true, true]]), Ok(1));
        assert_eq!(count_changes(&sides, &[vec![false, true]]), Err(3));
        // the change between the start and the first value, and a gap before it
        assert_eq!(count_changes(&[true, false, false], &[vec![true]]), Ok(0));
        // no change, and a pair where the sequence has none
        assert_eq!(
            count_changes(&[false, false, false], &[vec![false]]),
            Err(0)
        );
        assert_eq!(count_changes(&[true, true, true], &[vec![false]]), Err(2));
        // the change after the last value of the sequence, which has no gap
        assert_eq!(
            count_changes(&[true, true, true, false], &[vec![], vec![true]]),
            Ok(2)
        );
    }

    /// An `Eq1e` whose own numbers are 1, 2, 3, … in output order after unit 1d's, with the
    /// `nan_at`-th replaced by NaN (none when 0). A struct literal names every field, so a
    /// field added to `Eq1e`, `LandEq`, `ParcelEq`, `HomeAccount` or `WorkerEq1e` does not
    /// compile here until it is given a number.
    fn numbered(nan_at: usize) -> Eq1e {
        let e = g1(vec![land(10.0, Access::Enclosed)], ExitForm::Dependence);
        let Ok(Regime::Interior(real)) = e.solve() else {
            panic!("G1 solves")
        };
        let mut next = 0;
        let mut n = || {
            next += 1;
            if next == nan_at {
                f64::NAN
            } else {
                next as f64
            }
        };
        let mut eq = Eq1e {
            base: real.base.clone(),
            land_market: LandMarket::Idle,
            rent: n(),
            exit_land: ExitLand::Crowded,
            plot_rent: n(),
            q: n(),
            exit_good_price: n(),
            land: LandEq {
                enclosed: n(),
                commons: n(),
                market: n(),
                rented_plots: n(),
                idle: n(),
                commons_occupied: n(),
            },
            parcels: Vec::new(),
            coverage: n(),
            enclosure: None,
            home: HomeAccount {
                output: 0.0,
                rent_in_kind: 0.0,
                floor: 0.0,
                commons_shadow_rent: 0.0,
            },
            workers: Vec::new(),
            f_end: Some(1000.0),
            scan_points: 3,
            certified: true,
            residuals: Residuals1e {
                partition: 0.0,
                commons: 0.0,
            },
        };
        eq.enclosure = Some(EnclosureTie {
            worker: 1,
            share: n(),
        });
        eq.home = HomeAccount {
            output: n(),
            rent_in_kind: n(),
            floor: n(),
            commons_shadow_rent: n(),
        };
        eq.residuals = Residuals1e {
            partition: n(),
            commons: n(),
        };
        for _ in 0..2 {
            eq.parcels.push(ParcelEq {
                rent_per_acre: n(),
                shadow_rent_per_acre: n(),
                used: n(),
            });
        }
        eq.workers.push(WorkerEq1e {
            branch: Branch::Plot,
            exit_value: n(),
            exit_goods: n(),
            threshold: Some(n()),
            exiters: n(),
            plot_households: n(),
            plot_land: n(),
            rented_land: n(),
        });
        eq
    }

    /// The numbers unit 1e adds to `outputs()`, without the enclosure's type index.
    fn numbers(eq: &Eq1e) -> Vec<(OutputKey1c, f64)> {
        let skip = eq.base.outputs().len();
        eq.outputs()
            .into_iter()
            .skip(skip)
            .filter(|(key, _)| key.name != "enclosure_worker")
            .filter_map(|(key, output)| match output {
                Output1b::Float(v) | Output1b::Optional(Some(v)) => Some((key, v)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn outputs_list_every_field_once() {
        let eq = numbered(0);
        let listed = numbers(&eq);
        let values: Vec<f64> = listed.iter().map(|(_, v)| *v).collect();
        let want: Vec<f64> = (1..=listed.len()).map(|i| i as f64).collect();
        assert_eq!(values, want);
        assert_eq!(listed.len(), 11 + 1 + 4 + 2 + 2 * 3 + 7);
        let mut keys: Vec<String> = eq.outputs().iter().map(|(k, _)| k.to_string()).collect();
        let all = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), all);
        for key in [
            "parcel1.used",
            "worker0.plot_land",
            "market_land",
            "res_commons",
        ] {
            assert!(keys.contains(&key.to_string()), "{key}");
        }
        // 1d's f_end becomes optional
        let f_end = eq
            .outputs()
            .into_iter()
            .find(|(k, _)| k.name == "f_end")
            .unwrap();
        assert_eq!(f_end.1, Output1b::Optional(Some(1000.0)));
        let flags: Vec<(String, Output1b)> = eq
            .outputs()
            .into_iter()
            .skip(eq.base.outputs().len())
            .filter(|(_, o)| matches!(o, Output1b::Flag(_) | Output1b::Count(_)))
            .map(|(k, o)| (k.to_string(), o))
            .collect();
        let want: Vec<(String, Output1b)> = [
            ("land_market", Output1b::Count(1)),
            ("exit_land", Output1b::Count(2)),
            ("enclosure", Output1b::Flag(true)),
            ("scan_points", Output1b::Count(3)),
            ("certified", Output1b::Flag(true)),
            ("worker0.branch", Output1b::Count(1)),
        ]
        .into_iter()
        .map(|(k, o)| (k.to_string(), o))
        .collect();
        assert_eq!(flags, want);
        assert_eq!(
            [Branch::Dependence, Branch::Plot, Branch::Floor].map(Branch::code),
            [0, 1, 2]
        );
        assert_eq!(
            [
                ExitLand::Unused,
                ExitLand::Commons,
                ExitLand::Crowded,
                ExitLand::Enclosed,
                ExitLand::Idle
            ]
            .map(ExitLand::code),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            [LandMarket::Scarce, LandMarket::Idle].map(LandMarket::code),
            [0, 1]
        );
    }

    #[test]
    fn every_output_is_checked_for_finiteness() {
        assert_eq!(first_non_finite(&numbered(0)), None);
        let keys: Vec<OutputKey1c> = numbers(&numbered(0)).iter().map(|(k, _)| *k).collect();
        for (i, key) in keys.iter().enumerate() {
            let want = match key.item {
                Item::Worker(worker) => SolveError::NonFiniteInWorker {
                    worker,
                    what: key.name,
                },
                _ => SolveError::NonFinite { what: key.name },
            };
            assert_eq!(
                first_non_finite(&numbered(i + 1)),
                Some(want),
                "number {}",
                i + 1
            );
        }
        // an infinite f_∞ is absent, not a number; 1d's own outputs are checked through it
        let mut eq = numbered(0);
        eq.f_end = None;
        eq.base.f_end = f64::INFINITY;
        assert_eq!(first_non_finite(&eq), None);
        eq.base.v = f64::NAN;
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFinite { what: "v" })
        );
        // a free good's ratios at r = 0 are absent, and so are the task margin's where machines
        // are free
        let mut eq = numbered(0);
        let j = eq.base.categories.len() - 1;
        eq.base.categories[j].price = 0.0;
        eq.base.categories[j].real_wage = f64::INFINITY;
        eq.base.categories[j].wage_floor = f64::INFINITY;
        eq.base.categories[j].phi_w = Some(f64::NAN);
        eq.base.categories[j].phi_r = Some(f64::NAN);
        assert_eq!(first_non_finite(&eq), None);
        eq.base.categories[j].price = 1.0;
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFiniteInCategory {
                category: j,
                what: "real_wage"
            })
        );
        let mut eq = numbered(0);
        let t = eq.base.technique;
        eq.base.types[t].price = 0.0;
        eq.base.types[t].phi_w = Some(f64::NAN);
        eq.base.g = f64::INFINITY;
        eq.base.phi_w = Some(f64::NAN);
        eq.base.phi_r = Some(f64::NAN);
        assert_eq!(first_non_finite(&eq), None);
        eq.base.types[t].price = 1.0;
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFinite { what: "phi_w" })
        );
        eq.base.phi_w = Some(0.5);
        eq.base.phi_r = Some(0.5);
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFinite { what: "g" })
        );
    }
}
