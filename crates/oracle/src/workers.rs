//! Unit 1d: worker types, human-required tasks and the wall (docs/unit-1d.md).
//!
//! Unit 1c's economy with the tasks closed to machines in the equilibrium and several worker
//! types. The paper's human-required set H (SSRN §3.1, "a wall at the right edge") is hours per
//! unit of each category that any pooled worker can do; reserved tasks are hours that only one
//! worker type can do (the eras' "tasks closed to engines", main.tex:582-586). Types share the
//! contestable line's shape, each with an efficiency ε_i, so the types that sell hours there
//! are perfect substitutes in efficiency units and form the pool, with one wage v per
//! efficiency hour; a type whose reserved work takes all its hours at ε_i·v leaves the pool and
//! is paid a scarcity price at its own wall (docs/unit-1d.md §2.2 and §4.3).
//!
//! The pool's wage runs along one path in three stretches (§2.1): the all-human corner
//! (x = 0, the wage below its replacement value at the bottom task), 1a-1c's task line, and
//! the wall (x = 1, the wage above γ(1)·π), where "the machine comparison does not pin the
//! wage" (SSRN p.6) and labour clearing does. On the line the unknown is 1a's threshold, found
//! by 1a's bisection; at a corner it is the pool's real wage ω = v/P_s, found by bisection on
//! the bit patterns of doubles. 1c's count of sign changes runs over the whole path, and none
//! is [`SolveError::LaborShort`] (§5.3).
//!
//! The evaluation order is normative (§5.1): with one worker type of efficiency 1 and support
//! 1, and no human-required or reserved hours, every new operation is an exact no-op, and the
//! solve repeats unit 1c's floating-point operations bit for bit wherever 1c has an
//! equilibrium (§2.6).

use rustyecon_core::num;

use crate::categories::{segment_of, tasks, Category, Output1b};
use crate::leontief::Factors;
use crate::machine_block::{BlockPrices, MachineType};
use crate::machines::{
    in_category, no_basket_work, unpriced_category, worse, Item, MachineEconomy, MachineParams,
    OutputKey1c, SwitchPoint, Tie, TypeEq,
};
use crate::params::{self, ParamError, UniformWorkCost};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{
    bisect, bisect_bits, labor_net, Regime, Root, SolveError, BRACKET_HI, BRACKET_LO,
};

/// One worker type (docs/unit-1d.md §3.1).
///
/// "Scale" means finite and in [[`SCALE_FLOOR`](crate::SCALE_FLOOR),
/// [`SCALE_CEIL`](crate::SCALE_CEIL)].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkerType {
    /// N_i, potential workers of the type, one hour each per period. Scale.
    pub workers: f64,
    /// F_i, the distribution of the type's work cost χ (1a's uniform).
    pub work_cost: UniformWorkCost,
    /// ε_i, efficiency hours per hour at the contestable line and at the common
    /// human-required tasks: the type's schedule is ε_i·γ. 0, for a type that does only its
    /// reserved tasks, or scale.
    pub efficiency: f64,
    /// ν_i, baskets of support the provider gives each worker of the type, in work and out of
    /// it: its living cost (docs/unit-1d.md §2.3). Scale.
    pub support: f64,
}

/// The parameters of unit 1d (docs/unit-1d.md §3.1): unit 1c's [`MachineParams`] with N and
/// F moved into the worker types, plus the human-required and reserved hours. Build a
/// [`WorkerEconomy`] from them to validate.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerParams<S = PowerSchedule> {
    /// T, the land-service endowment per period. Scale.
    pub land: f64,
    /// γ(x), relative human productivity on the one task line, for efficiency 1.
    pub schedule: S,
    /// ρ, the interest rate, common to every machine type.
    pub rho: f64,
    /// The machine types (unit 1c), whose recipes use pool labour.
    pub machine_types: Vec<MachineType>,
    /// e_0 … e_S, the segments' edges on the task line (unit 1b).
    pub edges: Vec<f64>,
    /// The categories (unit 1b).
    pub categories: Vec<Category>,
    /// a_jl, the categories' intermediate inputs (unit 1c).
    pub intermediate: Vec<Vec<f64>>,
    /// The worker types, in the order every sum over types runs; at least one with ε > 0.
    pub worker_types: Vec<WorkerType>,
    /// L^H_j: efficiency hours per unit of category j at tasks closed to machines that any
    /// pooled worker can do (the paper's H). One entry per category, each 0 or scale.
    pub human_required: Vec<f64>,
    /// R_ji: hours of type i per unit of category j at tasks closed to machines and to every
    /// other type. One row per category, one entry per worker type, each 0 or scale.
    pub reserved: Vec<Vec<f64>>,
}

impl<S> WorkerParams<S> {
    /// Unit 1c's economy in worker form (docs/unit-1d.md §3.3, D0): one type with 1c's N and
    /// F, efficiency 1 and support 1, no human-required and no reserved hours. Where 1c has
    /// an equilibrium, it solves to 1c's bit for bit.
    pub fn from_machines(params: MachineParams<S>) -> Self {
        let count = params.categories.len();
        WorkerParams {
            land: params.land,
            schedule: params.schedule,
            rho: params.rho,
            machine_types: params.machine_types,
            edges: params.edges,
            categories: params.categories,
            intermediate: params.intermediate,
            worker_types: vec![WorkerType {
                workers: params.workers,
                work_cost: params.work_cost,
                efficiency: 1.0,
                support: 1.0,
            }],
            human_required: vec![0.0; count],
            reserved: vec![vec![0.0]; count],
        }
    }
}

/// A switch of the cheapest task type on the wall (docs/unit-1d.md §5.2): 1c's envelope
/// continued above γ(1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallSwitch {
    /// γ_s > γ(1), where the two types deliver tasks at the same cost at their closure wage.
    pub gamma: f64,
    /// v_s = γ_s·b̃_b/(θ_b − γ_s·λ̃_b), the pool's wage at the switch (the closure wage of the
    /// type below; NaN if it is not viable there).
    pub wage: f64,
    /// The type below v_s.
    pub below: usize,
    /// The type from v_s up.
    pub above: usize,
}

/// Which stretch of the path holds an equilibrium (docs/unit-1d.md §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Margin {
    /// On the task line: v = γ(x*)·π, 1a-1c's threshold (a root in (0, 10^-12) included).
    Contestable,
    /// At the wall: x* = 1 and v ≥ γ(1)·π, set by labour clearing (SSRN §3.1).
    Wall,
    /// At the all-human corner: x* = 0 and v ≤ γ(0)·π.
    AllHuman,
}

impl Margin {
    /// 0, 1 or 2, as [`Eq1d::outputs`] reports it.
    pub fn code(self) -> u32 {
        match self {
            Margin::Contestable => 0,
            Margin::Wall => 1,
            Margin::AllHuman => 2,
        }
    }
}

/// Why a point of the path has no finite excess demand (docs/unit-1d.md §4.3): its excess
/// demand is +∞.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortage {
    /// The first worker type whose reserved demand exceeds its workers.
    Reserved(usize),
    /// The walled types' wages cost at least the basket they buy: the walk has no fixed point.
    Ceiling,
}

/// A worker type at the edge of its reserved shortage (docs/unit-1d.md §12 items 2 and 16).
///
/// Where the type's reserved demand D_i reaches its workers N_i, its supply is vertical at N_i
/// for every real wage per support basket at or above expm1(χ_max,i), so its reserved market
/// clears at any such wage. The pool's clearing picks it: a higher wage raises P_s and lowers
/// the pool's real wage, so the excess demand rises with it, continuously, from its value at
/// the edge's finite side toward n_D. The type is then paid κ_i·ν_i·P_s, κ_i in place of ζ_i:
/// "a scarcity price … set by demand" (main.tex:584; SSRN §5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// The worker type whose reserved demand takes all its workers.
    pub worker: usize,
    /// κ_i, its real wage per support basket, in place of ζ_i.
    pub clearing: f64,
}

/// The end of the wall, v → ∞ under the wall's last technique (docs/unit-1d.md §4.5).
#[derive(Clone, Debug, PartialEq)]
pub struct WallEnd {
    /// τ_e, the wall's last technique.
    pub technique: usize,
    /// ω_∞ = lim v/P_s (+∞ when the basket's embodied labour vanishes), `None` where a point
    /// is short.
    pub omega: Option<f64>,
    /// f_∞ = n_D − S(ω_∞), +∞ where the end is short.
    pub excess: f64,
    /// The first type whose reserved demand exceeds its workers at x = 1.
    pub reserved: Option<usize>,
}

/// A validated unit-1d economy with every quantity that does not depend on the path
/// (docs/unit-1d.md §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerEconomy<S = PowerSchedule> {
    params: WorkerParams<S>,
    /// Unit 1c's x-free quantities; its N and F are unused placeholders.
    machines: MachineEconomy<S>,
    /// L̃^H = (I − A_cc)⁻¹L^H.
    required_chain: Vec<f64>,
    /// R̃_{·i} = (I − A_cc)⁻¹R_{·i}, by type.
    reserved_chain: Vec<Vec<f64>>,
    /// L^H_ŷ = Σ_j ŷ_j·L^H_j.
    required_per_basket: f64,
    /// R_ŷi = Σ_j ŷ_j·R_ji, by type.
    reserved_per_basket: Vec<f64>,
    /// ν = Σ_i ν_i·N_i.
    support: f64,
    /// Σ_i N_i.
    workers: f64,
    /// The factors of the (O, V) system at a given wage.
    wage_system: Factors,
    /// The switches of the envelope above γ(1).
    wall: Vec<WallSwitch>,
}

/// The error for a parameter of worker type `index`.
fn in_worker(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "worker type",
        index,
        error: Box::new(error),
    }
}

/// The worker types at one point of the path (docs/unit-1d.md §5.1 steps 5-7).
#[derive(Clone, Debug, PartialEq)]
struct WorkerState {
    demand: Vec<f64>,
    clearing: Vec<f64>,
    short: Option<Shortage>,
    p_s: f64,
    walled: Vec<bool>,
    wages: Vec<f64>,
    supply: Vec<f64>,
    pool_supply: f64,
}

/// The least fixed point of P = P⁰ + Σ_i max(e_i·r_i, c_i·r_i·P) (docs/unit-1d.md §4.3, the
/// walk), with the walled set, or `None` when a denominator is ≤ 0.
///
/// The candidates are the types with r_i > 0 and c_i > 0, visited in increasing order of their
/// thresholds e_i/c_i (ties to the lower index). Starting with every type pooled, each
/// candidate is walled while c_i·P > e_i, and P is recomputed after each as
/// (P⁰ + Σ_pooled e_i·r_i)/(1.0 − Σ_walled c_i·r_i), both sums from 0.0 in type order.
pub(crate) fn walk(
    base: f64,
    pooled_wage: &[f64],
    walled_rate: &[f64],
    per_basket: &[f64],
) -> Option<(f64, Vec<bool>)> {
    let n = per_basket.len();
    let mut order: Vec<usize> = (0..n)
        .filter(|&i| per_basket[i] > 0.0 && walled_rate[i] > 0.0)
        .collect();
    order.sort_by(|&a, &b| {
        let (ta, tb) = (
            pooled_wage[a] / walled_rate[a],
            pooled_wage[b] / walled_rate[b],
        );
        ta.total_cmp(&tb).then(a.cmp(&b))
    });
    let price = |walled: &[bool]| -> Option<f64> {
        let (mut pooled, mut rate) = (0.0, 0.0);
        for i in 0..n {
            if walled[i] {
                rate += walled_rate[i] * per_basket[i];
            } else {
                pooled += pooled_wage[i] * per_basket[i];
            }
        }
        let room = 1.0 - rate;
        if room <= 0.0 {
            None
        } else {
            Some((base + pooled) / room)
        }
    };
    let mut walled = vec![false; n];
    let mut p = price(&walled)?;
    for &i in &order {
        if walled_rate[i] * p > pooled_wage[i] {
            walled[i] = true;
            p = price(&walled)?;
        } else {
            break;
        }
    }
    Some((p, walled))
}

impl<S: Schedule + Clone> WorkerEconomy<S> {
    /// Validates the parameters (docs/unit-1d.md §3.2) and computes every quantity that does
    /// not depend on the path (§5.2).
    ///
    /// In order: unit 1c's checks in 1c's order without N and χ_max and without its last
    /// three (each category priced, and the basket's land and work), then the basket's land
    /// (1c's), then the worker types (at least one; each type's `workers`, `work_cost.chi_max`
    /// and `support` scale and `efficiency` 0 or scale, as `ParamError::Item { kind: "worker
    /// type", .. }`), at least one with ε > 0, `human_required` (C entries) and `reserved` (C
    /// rows of I entries), each 0 or scale, every type with work (ε_i > 0 or R_ŷi > 0), each
    /// category priced through the chain with the pool's and the reserved hours, and the
    /// basket needing pool work at x = 0, Σ_j ŷ_j(L̄^dir_j + L^H_j) > 0. −0.0 is stored as
    /// +0.0. With one type and no human-required or reserved hours these are 1c's rules.
    pub fn new(mut params: WorkerParams<S>) -> Result<Self, ParamError> {
        let machine_params = MachineParams {
            workers: 1.0,
            land: params.land,
            schedule: params.schedule.clone(),
            work_cost: UniformWorkCost { chi_max: 1.0 },
            rho: params.rho,
            machine_types: std::mem::take(&mut params.machine_types),
            edges: std::mem::take(&mut params.edges),
            categories: std::mem::take(&mut params.categories),
            intermediate: std::mem::take(&mut params.intermediate),
        };
        let machines = MachineEconomy::assemble(machine_params, false)?;
        machines.check_basket_land()?;
        {
            let m = machines.params();
            params.rho = m.rho;
            params.machine_types = m.machine_types.clone();
            params.edges = m.edges.clone();
            params.categories = m.categories.clone();
            params.intermediate = m.intermediate.clone();
        }
        // The worker types.
        if params.worker_types.is_empty() {
            return Err(ParamError::Invalid {
                name: "worker types",
                reason: "the economy needs at least one worker type",
            });
        }
        for (index, t) in params.worker_types.iter_mut().enumerate() {
            let item = in_worker(index);
            params::scale("workers", t.workers).map_err(&item)?;
            params::scale("chi_max", t.work_cost.chi_max).map_err(&item)?;
            t.efficiency = params::zero_or_scale("efficiency", t.efficiency).map_err(&item)?;
            params::scale("support", t.support).map_err(&item)?;
        }
        if !params.worker_types.iter().any(|t| t.efficiency > 0.0) {
            return Err(ParamError::Invalid {
                name: "worker types",
                reason: "at least one worker type must sell hours on the contestable line: \
                         efficiency > 0",
            });
        }
        let count = params.categories.len();
        let kinds = params.worker_types.len();
        if params.human_required.len() != count {
            return Err(ParamError::Invalid {
                name: "human_required",
                reason: "the human-required hours need one entry per category",
            });
        }
        for (index, h) in params.human_required.iter_mut().enumerate() {
            *h = params::zero_or_scale("human_required", *h).map_err(in_category(index))?;
        }
        if params.reserved.len() != count || params.reserved.iter().any(|r| r.len() != kinds) {
            return Err(ParamError::Invalid {
                name: "reserved",
                reason: "the reserved hours need one row per category, each with one entry per \
                         worker type",
            });
        }
        for (index, row) in params.reserved.iter_mut().enumerate() {
            for r in row.iter_mut() {
                *r = params::zero_or_scale("reserved", *r).map_err(in_category(index))?;
            }
        }
        // The x-free hours (§3.1): through the chain, and per basket.
        let yhat = machines.basket_outputs();
        let required_chain = machines.chain().solve(&params.human_required);
        let reserved_chain: Vec<Vec<f64>> = (0..kinds)
            .map(|i| {
                let column: Vec<f64> = params.reserved.iter().map(|row| row[i]).collect();
                machines.chain().solve(&column)
            })
            .collect();
        let mut required_per_basket = 0.0;
        for (y, h) in yhat.iter().zip(&params.human_required) {
            required_per_basket += y * h;
        }
        let reserved_per_basket: Vec<f64> = (0..kinds)
            .map(|i| {
                let mut total = 0.0;
                for (y, row) in yhat.iter().zip(&params.reserved) {
                    total += y * row[i];
                }
                total
            })
            .collect();
        for (index, t) in params.worker_types.iter().enumerate() {
            if !(t.efficiency > 0.0 || reserved_per_basket[index] > 0.0) {
                return Err(in_worker(index)(ParamError::Invalid {
                    name: "reserved",
                    reason: "a worker type with efficiency 0 needs reserved work in the basket",
                }));
            }
        }
        for index in 0..count {
            let mut reserved = 0.0;
            for chain in &reserved_chain {
                reserved += chain[index];
            }
            let hours = machines.all_human_hours()[index] + required_chain[index] + reserved;
            if !(hours > 0.0 || machines.chain_land()[index] > 0.0) {
                return Err(unpriced_category(index));
            }
        }
        let mut pool_work = 0.0;
        for ((y, l), h) in yhat
            .iter()
            .zip(machines.direct_hours())
            .zip(&params.human_required)
        {
            pool_work += y * (l + h);
        }
        if pool_work <= 0.0 {
            return Err(no_basket_work());
        }
        let (mut support, mut workers) = (0.0, 0.0);
        for t in &params.worker_types {
            support += t.support * t.workers;
            workers += t.workers;
        }
        // The envelope continued above γ(1) (§5.2): its switches up to γ(1) are 1c's.
        let block = machines.block();
        let full = block.envelope(params.schedule.gamma(BRACKET_LO), f64::INFINITY);
        let line = machines.envelope().switches.len();
        debug_assert_eq!(full.switches[..line], machines.envelope().switches[..]);
        let wall = full.switches[line..]
            .iter()
            .map(|s| WallSwitch {
                gamma: s.gamma,
                wage: block.closure_wage(s.gamma, s.below).unwrap_or(f64::NAN),
                below: s.below,
                above: s.above,
            })
            .collect();
        let wage_system = block.wage_system();
        Ok(WorkerEconomy {
            params,
            machines,
            required_chain,
            reserved_chain,
            required_per_basket,
            reserved_per_basket,
            support,
            workers,
            wage_system,
            wall,
        })
    }

    /// The validated parameters.
    pub fn params(&self) -> &WorkerParams<S> {
        &self.params
    }

    /// Unit 1c's economy underneath: the machine block, the envelope on the line, the chain
    /// and the basket's outputs. Its N and F are unused placeholders.
    pub fn machines(&self) -> &MachineEconomy<S> {
        &self.machines
    }

    /// L̃^H = (I − A_cc)⁻¹L^H, each category's common human-required hours through the chain.
    pub fn human_required_chain(&self) -> &[f64] {
        &self.required_chain
    }

    /// R̃_{·i} = (I − A_cc)⁻¹R_{·i}: type i's reserved hours per unit of each category,
    /// through the chain.
    pub fn reserved_chain(&self, worker: usize) -> &[f64] {
        &self.reserved_chain[worker]
    }

    /// L^H_ŷ = Σ_j ŷ_j·L^H_j, the common human-required hours per basket.
    pub fn human_required_per_basket(&self) -> f64 {
        self.required_per_basket
    }

    /// R_ŷi = Σ_j ŷ_j·R_ji, each type's reserved hours per basket.
    pub fn reserved_per_basket(&self) -> &[f64] {
        &self.reserved_per_basket
    }

    /// ν = Σ_i ν_i·N_i, the baskets of support the provider funds.
    pub fn support(&self) -> f64 {
        self.support
    }

    /// The switches of the technique on the wall, v increasing.
    pub fn wall_switches(&self) -> &[WallSwitch] {
        &self.wall
    }

    /// The switch points x_i on the line (unit 1c's).
    pub fn switch_points(&self) -> Result<Vec<f64>, SolveError> {
        self.machines.switch_points()
    }

    /// The technique on the line at x (unit 1c's envelope).
    pub fn technique_at(&self, x: f64) -> usize {
        self.machines.technique_at(x)
    }

    /// The technique on the wall at the pool's wage v: the line's last, then the type above
    /// each wall switch from its wage up.
    pub fn wall_technique_at(&self, v: f64) -> usize {
        let mut technique = self.machines.envelope().last();
        for s in &self.wall {
            if v >= s.wage {
                technique = s.above;
            } else {
                break;
            }
        }
        technique
    }

    /// Prices and quantities at x on the line under the envelope's technique there.
    pub fn at(&self, x: f64) -> WorkerPoint {
        self.at_with(x, self.technique_at(x))
    }

    /// Prices and quantities at a candidate threshold x on the line, with task type
    /// `technique` doing the machine tasks and the task margin setting v = γ(x)·π
    /// (docs/unit-1d.md §5.1). For one type of efficiency and support 1 without human-required
    /// or reserved hours, every field shared with unit 1c's
    /// [`MachineEconomy::at_with`](crate::MachineEconomy::at_with) is its value bit for bit.
    ///
    /// Outside [0, 1], or where the technique is not viable (d ≤ 0), the formulas are
    /// evaluated as written and mean nothing. Panics if `technique` is out of range.
    pub fn at_with(&self, x: f64, technique: usize) -> WorkerPoint {
        let gamma = self.params.schedule.gamma(x);
        let block = self.machines.block().prices_at(gamma, technique);
        self.evaluate(x, technique, block, None)
    }

    /// [`at_with`](Self::at_with) with one worker type at the edge of its reserved shortage:
    /// its clearing real wage per support basket is `edge.clearing` in place of
    /// expm1(χ_max·D/N) (docs/unit-1d.md §12 item 16). The evaluation that prices an
    /// equilibrium on the line at such an edge.
    pub fn at_edge(&self, x: f64, technique: usize, edge: Edge) -> WorkerPoint {
        let gamma = self.params.schedule.gamma(x);
        let block = self.machines.block().prices_at(gamma, technique);
        self.evaluate(x, technique, block, Some(edge))
    }

    /// Prices and quantities at x with the pool's wage v given (docs/unit-1d.md §5.1, a
    /// corner's evaluation): the machine prices from the wage-given (O, V) system, and the
    /// quantities at x with `technique` doing the machine tasks. The corners use x = 0 and 1.
    pub fn at_wage(&self, x: f64, v: f64, technique: usize) -> WorkerPoint {
        let block = self
            .machines
            .block()
            .prices_at_wage(&self.wage_system, v, technique);
        self.evaluate(x, technique, block, None)
    }

    /// §5.1 steps 1 and 3-8 with the machine block's prices given, and a type at its edge.
    fn evaluate(
        &self,
        x: f64,
        technique: usize,
        block: BlockPrices,
        edge: Option<Edge>,
    ) -> WorkerPoint {
        let p = &self.params;
        let m = &self.machines;
        let gamma = p.schedule.gamma(x);
        let j = p.schedule.integral(x);
        let (v, task_price) = (block.v, block.task_price);
        // Step 3: the categories, with the common human-required hours.
        let count = p.categories.len();
        let (mut human, mut machine, mut rhs) = (
            Vec::with_capacity(count),
            Vec::with_capacity(count),
            Vec::with_capacity(count),
        );
        for (index, category) in p.categories.iter().enumerate() {
            let (h, mm) = tasks(
                &p.edges,
                m.integral_at_edges(),
                &category.density,
                x,
                j,
                None,
            );
            let h = h + p.human_required[index];
            rhs.push(v * h + task_price * mm + category.direct_land);
            human.push(h);
            machine.push(mm);
        }
        let base_prices = m.chain().solve(&rhs);
        let yhat = m.basket_outputs();
        let (mut base_p_s, mut h_s, mut m_s) = (0.0, 0.0, 0.0);
        for index in 0..count {
            base_p_s += p.categories[index].weight * base_prices[index];
            h_s += yhat[index] * human[index];
            m_s += yhat[index] * machine[index];
        }
        // Step 4: the quantities.
        let theta = m.block().types()[technique].task_efficiency;
        let mut task = vec![0.0; m.block().len()];
        task[technique] = m_s / theta;
        let cleared = m.clear(&task);
        let final_hours = cleared.y * h_s;
        let n_d = final_hours + cleared.machine_hours;
        // Steps 5-8: the worker types, the walk, supply and the prices with reserved costs.
        let state = self.workers_at(v, base_p_s, cleared.y, edge);
        let (reserved_costs, prices) = self.with_reserved(&base_prices, &state.wages);
        WorkerPoint {
            x,
            gamma,
            j,
            technique,
            d: block.d,
            type_prices: block.prices,
            operating: block.operating,
            build: block.build,
            v,
            task_price,
            base_prices,
            reserved_costs,
            prices,
            human,
            machine,
            base_p_s,
            p_s: state.p_s,
            h_s,
            m_s,
            b_d: m.basket_land(),
            y: cleared.y,
            services: cleared.services,
            final_hours,
            machine_hours: cleared.machine_hours,
            n_d,
            reserved_demand: state.demand,
            clearing: state.clearing,
            walled: state.walled,
            wages: state.wages,
            supply: state.supply,
            n_s: state.pool_supply,
            short: state.short,
        }
    }

    /// Step 5: reserved demand D_i = Y·R_ŷi and ζ_i = expm1(χ_max,i·(D_i/N_i)), in type order,
    /// and the first type whose D_i exceeds N_i; a type at its edge takes the edge's κ_i in
    /// place of ζ_i.
    fn reserved_at(&self, y: f64, edge: Option<Edge>) -> (Vec<f64>, Vec<f64>, Option<Shortage>) {
        let kinds = self.params.worker_types.len();
        let (mut demand, mut clearing) = (Vec::with_capacity(kinds), Vec::with_capacity(kinds));
        let mut short = None;
        for (i, t) in self.params.worker_types.iter().enumerate() {
            let d = y * self.reserved_per_basket[i];
            if short.is_none() && d > t.workers {
                short = Some(Shortage::Reserved(i));
            }
            demand.push(d);
            clearing.push(match edge {
                Some(e) if e.worker == i => e.clearing,
                _ => num::expm1(t.work_cost.chi_max * (d / t.workers)),
            });
        }
        (demand, clearing, short)
    }

    /// Steps 5-7 at the pool's wage v, P⁰_s and Y (docs/unit-1d.md §5.1), with a type at its
    /// edge if one is given.
    fn workers_at(&self, v: f64, base_p_s: f64, y: f64, edge: Option<Edge>) -> WorkerState {
        let types = &self.params.worker_types;
        let kinds = types.len();
        let (demand, clearing, short) = self.reserved_at(y, edge);
        let unsolved = |short| WorkerState {
            demand: demand.clone(),
            clearing: clearing.clone(),
            short: Some(short),
            p_s: f64::NAN,
            walled: vec![false; kinds],
            wages: vec![f64::NAN; kinds],
            supply: vec![f64::NAN; kinds],
            pool_supply: f64::NAN,
        };
        if let Some(short) = short {
            return unsolved(short);
        }
        // Step 6: the walk.
        let pooled_wage: Vec<f64> = types.iter().map(|t| t.efficiency * v).collect();
        let walled_rate: Vec<f64> = clearing
            .iter()
            .zip(types)
            .map(|(z, t)| z * t.support)
            .collect();
        let Some((p_s, walled)) = walk(
            base_p_s,
            &pooled_wage,
            &walled_rate,
            &self.reserved_per_basket,
        ) else {
            return unsolved(Shortage::Ceiling);
        };
        let wages: Vec<f64> = (0..kinds)
            .map(|i| {
                if walled[i] {
                    walled_rate[i] * p_s
                } else {
                    pooled_wage[i]
                }
            })
            .collect();
        // Step 7: supply per type, and the pool's net supply in efficiency hours.
        let supply: Vec<f64> = types
            .iter()
            .zip(&wages)
            .map(|(t, &w)| t.workers * t.work_cost.cdf(num::ln1p(w / (t.support * p_s))))
            .collect();
        let mut pool_supply = 0.0;
        for i in 0..kinds {
            if !walled[i] {
                pool_supply += types[i].efficiency * (supply[i] - demand[i]);
            }
        }
        WorkerState {
            demand,
            clearing,
            short: None,
            p_s,
            walled,
            wages,
            supply,
            pool_supply,
        }
    }

    /// Step 8: R_j = Σ_i v_i·R̃_ji from 0.0 in type order, and p_j = p⁰_j + R_j.
    fn with_reserved(&self, base: &[f64], wages: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let mut costs = Vec::with_capacity(base.len());
        let mut prices = Vec::with_capacity(base.len());
        for (index, &p0) in base.iter().enumerate() {
            let mut r = 0.0;
            for (i, &w) in wages.iter().enumerate() {
                r += w * self.reserved_chain[i][index];
            }
            costs.push(r);
            prices.push(p0 + r);
        }
        (costs, prices)
    }

    /// The quantities of a corner at (x, technique), which do not depend on the wage, and the
    /// basket's price-side totals L_s and B_s there (docs/unit-1d.md §4.5).
    fn corner(&self, x: f64, technique: usize) -> Corner {
        let p = &self.params;
        let m = &self.machines;
        let block = m.block();
        let j_x = p.schedule.integral(x);
        let theta = block.types()[technique].task_efficiency;
        let (lt_coef, bt_coef) = match block.price_totals() {
            Some((lt, bt)) => (lt[technique] / theta, bt[technique] / theta),
            None => (f64::NAN, f64::NAN),
        };
        let yhat = m.basket_outputs();
        let count = p.categories.len();
        let (mut hours, mut land) = (Vec::with_capacity(count), Vec::with_capacity(count));
        let (mut h_s, mut m_s) = (0.0, 0.0);
        for (index, category) in p.categories.iter().enumerate() {
            let (h, mm) = tasks(
                &p.edges,
                m.integral_at_edges(),
                &category.density,
                x,
                j_x,
                None,
            );
            let h = h + p.human_required[index];
            hours.push(h + mm * lt_coef);
            land.push(category.direct_land + mm * bt_coef);
            h_s += yhat[index] * h;
            m_s += yhat[index] * mm;
        }
        let (lt_c, bt_c) = (m.chain().solve(&hours), m.chain().solve(&land));
        let (mut l_s, mut b_s) = (0.0, 0.0);
        for (index, category) in p.categories.iter().enumerate() {
            l_s += category.weight * lt_c[index];
            b_s += category.weight * bt_c[index];
        }
        let mut task = vec![0.0; block.len()];
        task[technique] = m_s / theta;
        let cleared = m.clear(&task);
        let n_d = cleared.y * h_s + cleared.machine_hours;
        let (demand, clearing, short) = self.reserved_at(cleared.y, None);
        Corner {
            n_d,
            demand,
            clearing,
            short,
            l_s,
            b_s,
        }
    }

    /// S(ω), the pool's net supply at the real wage ω at a corner (docs/unit-1d.md §4.5):
    /// Σ ε_i·(N_i·F_i(ln(1 + ε_i·ω/ν_i)) − D_i) over the types with ε_i·ω/ν_i ≥ ζ_i.
    fn supply_at(&self, c: &Corner, omega: f64) -> f64 {
        let mut s = 0.0;
        for (i, t) in self.params.worker_types.iter().enumerate() {
            if t.efficiency > 0.0 {
                let r = (t.efficiency * omega) / t.support;
                if r >= c.clearing[i] {
                    s += t.efficiency * (t.workers * t.work_cost.cdf(num::ln1p(r)) - c.demand[i]);
                }
            }
        }
        s
    }

    /// The pool's wage at the real wage ω at a corner (docs/unit-1d.md §4.5):
    /// v = ω·B/((1 − C) − ω·L), with L = L_s + Σ_pooled ε_i·R_ŷi and C = Σ_walled ζ_i·ν_i·R_ŷi.
    fn wage_at(&self, c: &Corner, omega: f64) -> f64 {
        let (mut hours, mut rate) = (c.l_s, 0.0);
        for (i, t) in self.params.worker_types.iter().enumerate() {
            let pooled = t.efficiency > 0.0 && (t.efficiency * omega) / t.support >= c.clearing[i];
            if pooled {
                hours += t.efficiency * self.reserved_per_basket[i];
            } else {
                rate += (c.clearing[i] * t.support) * self.reserved_per_basket[i];
            }
        }
        (omega * c.b_s) / ((1.0 - rate) - omega * hours)
    }

    /// The end of the wall (docs/unit-1d.md §4.5): ω_∞ = 1/ρ_∞ from the walk with P⁰ → L_s
    /// and e_i → ε_i under the wall's last technique, and f_∞ = n_D − S(ω_∞).
    pub fn wall_end(&self) -> WallEnd {
        let technique = self
            .wall
            .last()
            .map_or(self.machines.envelope().last(), |s| s.above);
        let c = self.corner(BRACKET_HI, technique);
        if let Some(Shortage::Reserved(i)) = c.short {
            return WallEnd {
                technique,
                omega: None,
                excess: f64::INFINITY,
                reserved: Some(i),
            };
        }
        let omega = self.omega_end(&c);
        let excess = match omega {
            Some(w) => c.n_d - self.supply_at(&c, w),
            None => f64::INFINITY,
        };
        WallEnd {
            technique,
            omega,
            excess,
            reserved: None,
        }
    }

    /// ω_∞ at a corner, `None` when the walk has no fixed point.
    fn omega_end(&self, c: &Corner) -> Option<f64> {
        let types = &self.params.worker_types;
        let efficiency: Vec<f64> = types.iter().map(|t| t.efficiency).collect();
        let rate: Vec<f64> = c
            .clearing
            .iter()
            .zip(types)
            .map(|(z, t)| z * t.support)
            .collect();
        walk(c.l_s, &efficiency, &rate, &self.reserved_per_basket).map(|(rho, _)| 1.0 / rho)
    }

    /// The cheapest task type at the pool's wage v, min p_t/θ_t with the wage-given prices,
    /// ties to the lower index (the all-human corner's report, docs/unit-1d.md §5.3).
    fn cheapest_at(&self, v: f64) -> usize {
        let block = self.machines.block();
        let first = self.machines.envelope().first;
        let prices = block.prices_at_wage(&self.wage_system, v, first).prices;
        let mut best: Option<(usize, f64)> = None;
        for (t, mt) in block.types().iter().enumerate() {
            if mt.task_efficiency > 0.0 {
                let cost = prices[t] / mt.task_efficiency;
                let better = match best {
                    None => true,
                    Some((_, c)) => cost < c,
                };
                if better {
                    best = Some((t, cost));
                }
            }
        }
        best.map_or(first, |(t, _)| t)
    }

    /// Classifies the economy and solves it (docs/unit-1d.md §5.3).
    ///
    /// 1c's viability test comes first (`NotViable`, decision 64's convention). Then the
    /// excess demand is read along the whole path, in order: at x = 0 and at
    /// [`BRACKET_LO`] under the first technique, at each switch of the line under the
    /// technique below and above, at x = 1, at each switch of the wall under the technique
    /// below and above (the corner's evaluation at the switch's wage), and at the end of the
    /// wall; a short point is +∞. A value is on the positive side when > 0, except f(1) and
    /// f_∞, which are when ≥ 0 (f_∞ = 0 is not when the wall's last piece starts at an exact
    /// zero). No change of side is [`SolveError::LaborShort`], more than one
    /// [`SolveError::MultipleEquilibria`] (its `switches` the line's). One is the
    /// equilibrium, reported inside [`Regime::Interior`] with [`Eq1d::margin`] saying where:
    /// the all-human corner, a root in [0, lo] (bisection on the doubles' bit patterns), a
    /// root or a tie on the line (unit 1c's step 3 unchanged), a root on a piece of the wall
    /// (bisection on ω) or a tie at a wall switch. Where bisection in x or in a tie's σ closes
    /// on a short point (+∞), the equilibrium is the edge of that reserved shortage: the short
    /// type's supply is vertical there, and its wage, found by a bisection of its own, clears
    /// the pool ([`Edge`], [`WorkerEq::edge`]). 1d never returns `BoundaryNoMargin` or
    /// `NoInteriorAtZero`.
    ///
    /// Returns [`SolveError::NonFinite`] (or the per-item variants) if a value the solve
    /// decides on, or any reported output, is not finite, and
    /// [`SolveError::LaborNotCleared`] if the pool's residual |n_pool − S| exceeds
    /// [`LABOR_RESIDUAL_NET`](crate::LABOR_RESIDUAL_NET) of n_pool.
    pub fn solve(&self) -> Result<Regime<Eq1d>, SolveError> {
        let env = self.machines.envelope();
        let at_one = self.at_with(BRACKET_HI, env.last());
        // Step 1: 1c's viability test.
        if at_one.d.is_nan() {
            return Err(SolveError::NonFinite { what: "D(1)" });
        }
        if at_one.d <= 0.0 {
            return Ok(Regime::NotViable { d_at_1: at_one.d });
        }
        // Step 2: the sequence along the path.
        let f_one = value("n_D(1) - n_S(1)", &at_one)?;
        let f_lo = value(
            "n_D - n_S at BRACKET_LO",
            &self.at_with(BRACKET_LO, env.first),
        )?;
        let at_zero = self.at_with(0.0, env.first);
        let f_zero = value("n_D - n_S at 0", &at_zero)?;
        let points = self.machines.switch_points()?;
        let techniques: Vec<usize> = std::iter::once(env.first)
            .chain(env.switches.iter().map(|s| s.above))
            .collect();
        let mut sequence = vec![(Kind::Zero, f_zero), (Kind::Lo, f_lo)];
        for (i, &x) in points.iter().enumerate() {
            let what = "n_D - n_S at a switch";
            sequence.push((
                Kind::SwitchBelow(i),
                value(what, &self.at_with(x, techniques[i]))?,
            ));
            sequence.push((
                Kind::SwitchAbove(i),
                value(what, &self.at_with(x, techniques[i + 1]))?,
            ));
        }
        sequence.push((Kind::One, f_one));
        for (s, w) in self.wall.iter().enumerate() {
            let what = "n_D - n_S at a wall switch";
            sequence.push((
                Kind::WallBelow(s),
                value(what, &self.at_wage(BRACKET_HI, w.wage, w.below))?,
            ));
            sequence.push((
                Kind::WallAbove(s),
                value(what, &self.at_wage(BRACKET_HI, w.wage, w.above))?,
            ));
        }
        let end = self.wall_end();
        if end.excess.is_nan() || end.excess == f64::NEG_INFINITY {
            return Err(SolveError::NonFinite {
                what: "n_D - n_S at the end of the wall",
            });
        }
        // The value that starts the wall's last piece: f_line(1), or the last wall switch's
        // value above.
        let last_start = sequence.last().map_or(f_one, |&(_, f)| f);
        sequence.push((Kind::End, end.excess));
        // Step 3: the sides, from a positive one for v → 0. f_∞ = 0 is on the positive side
        // (an equilibrium at v = ∞ is none), except when the last piece starts at an exact
        // zero: f is then 0 on the whole piece (Lemma 3'), and its start is the equilibrium by
        // the exact-zero rule (docs/unit-1d.md §12 item 17).
        let mut sides = Vec::with_capacity(sequence.len() + 1);
        sides.push(true);
        for &(kind, f) in &sequence {
            sides.push(match kind {
                Kind::One => f >= 0.0,
                Kind::End => f > 0.0 || (f == 0.0 && last_start != 0.0),
                _ => f > 0.0,
            });
        }
        let changes: Vec<usize> = (0..sides.len() - 1)
            .filter(|&i| sides[i] != sides[i + 1])
            .collect();
        if changes.is_empty() {
            return Err(SolveError::LaborShort {
                excess: end.excess,
                reserved: end.reserved,
            });
        }
        if changes.len() > 1 {
            return Err(SolveError::MultipleEquilibria {
                sign_changes: changes.len(),
                switches: points,
            });
        }
        let context = Context {
            points: &points,
            at_one: &at_one,
            f_zero,
            f_lo,
            f_one,
            f_end: end.excess,
        };
        let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
            .chain(points.iter().copied())
            .chain(std::iter::once(BRACKET_HI))
            .collect();
        // The change between sides c and c + 1 is between sequence[c − 1] (the start when
        // c = 0) and sequence[c].
        let c = changes[0];
        let eq = if c == 0 {
            // The all-human corner: ω in [0, ω_line(0)], f(0) = n_D(0).
            let (v, steps) = if f_zero == 0.0 {
                (at_zero.v, 0)
            } else {
                let corner = self.corner(0.0, env.first);
                // D_i and ζ_i are fixed at a corner, so f(ω) = n_D − S(ω) is finite on it,
                // and the walk's ceiling cannot bind below a point where it does not (§12).
                let root = bisect_bits(
                    |w| corner.n_d - self.supply_at(&corner, w),
                    (0.0, corner.n_d),
                    (at_zero.v / at_zero.p_s, f_zero),
                )?;
                (self.wage_at(&corner, root.x), root.steps)
            };
            let technique = self.cheapest_at(v);
            let at = Where::Corner {
                margin: Margin::AllHuman,
                v,
                steps,
            };
            self.report(at, technique, None, None, &context)?
        } else {
            let ((before, f_before), (after, f_after)) = (sequence[c - 1], sequence[c]);
            match (before, after) {
                (Kind::Zero, _) => {
                    // A root in [0, lo] under the first technique, or the edge of a reserved
                    // shortage there.
                    let t = env.first;
                    let root = if f_lo == 0.0 {
                        Root::exact(BRACKET_LO, 0)
                    } else {
                        bisect_bits(
                            |x| self.at_with(x, t).excess_demand(),
                            (0.0, f_zero),
                            (BRACKET_LO, f_lo),
                        )?
                    };
                    let (at, edge) = self.on_the_line(root, t)?;
                    self.report(at, t, None, edge, &context)?
                }
                (Kind::Lo, _) | (Kind::SwitchAbove(_), _) => {
                    // Inside a region of the line: 1a's bisection, or an exact zero at its
                    // upper end (unit 1c's step 3).
                    let region = match before {
                        Kind::SwitchAbove(i) => i + 1,
                        _ => 0,
                    };
                    let t = techniques[region];
                    let (lo, hi) = (bounds[region], bounds[region + 1]);
                    let root = if f_after == 0.0 {
                        Root::exact(hi, 0)
                    } else {
                        bisect(
                            |x| self.at_with(x, t).excess_demand(),
                            (lo, f_before),
                            (hi, f_after),
                        )?
                    };
                    let (at, edge) = self.on_the_line(root, t)?;
                    self.report(at, t, None, edge, &context)?
                }
                (Kind::SwitchBelow(i), _) => {
                    // A tie at a switch of the line.
                    let (below, above) = (techniques[i], techniques[i + 1]);
                    let x = points[i];
                    let (share, edge) = self.tie_share(&self.at_with(x, below), below, above)?;
                    let tie = Tie {
                        above,
                        share,
                        gamma: env.switches[i].gamma,
                    };
                    let at = Where::Line {
                        x,
                        one_minus_x: 1.0 - x,
                        steps: 0,
                    };
                    self.report(at, below, Some(tie), edge, &context)?
                }
                (Kind::One, _) | (Kind::WallAbove(_), _) => {
                    let piece = match before {
                        Kind::WallAbove(s) => s + 1,
                        _ => 0,
                    };
                    self.wall_piece(piece, (f_before, f_after), &end, &context)?
                }
                (Kind::WallBelow(s), _) => {
                    // A tie at a switch of the wall.
                    let w = self.wall[s];
                    let q = self.at_wage(BRACKET_HI, w.wage, w.below);
                    let (share, edge) = self.tie_share(&q, w.below, w.above)?;
                    let tie = Tie {
                        above: w.above,
                        share,
                        gamma: w.gamma,
                    };
                    let at = Where::Corner {
                        margin: Margin::Wall,
                        v: w.wage,
                        steps: 0,
                    };
                    self.report(at, w.below, Some(tie), edge, &context)?
                }
                (Kind::End, _) => unreachable!("the end of the wall is the last value"),
            }
        };
        if let Some(error) = first_non_finite(&eq) {
            return Err(error);
        }
        labor_net(eq.x_star, eq.residuals.labor, eq.n_pool)?;
        Ok(Regime::Interior(Box::new(eq)))
    }

    /// Where a root on the line is reported. A root whose bracket closed on a short point (+∞)
    /// is the edge of a reserved shortage (docs/unit-1d.md §12 item 16): x* is the bracket's
    /// finite end, where the short type's reserved demand is within rounding of its workers,
    /// and that type's κ is found by [`edge_clearing`](Self::edge_clearing). Its steps are
    /// added to the root's.
    fn on_the_line(
        &self,
        root: Root,
        technique: usize,
    ) -> Result<(Where, Option<Edge>), SolveError> {
        if !root.jump {
            return Ok((Where::line(root), None));
        }
        let worker = self.short_type(self.at_with(root.x.next_down(), technique).short)?;
        let at = self.at_with(root.x, technique);
        let (edge, steps) =
            self.edge_clearing(worker, (at.clearing[worker], at.excess_demand()), |kappa| {
                let edge = Edge {
                    worker,
                    clearing: kappa,
                };
                self.at_edge(root.x, technique, edge).excess_demand()
            })?;
        let at = Where::Line {
            x: root.x,
            one_minus_x: root.one_minus_x,
            steps: root.steps + steps,
        };
        Ok((at, Some(edge)))
    }

    /// The type whose reserved demand exceeds its workers on the short side of an edge. A
    /// change of side cannot close on the walk's ceiling: approaching it from the finite side,
    /// P_s grows without bound and f rises to n_D > 0 (docs/unit-1d.md §12 item 16).
    fn short_type(&self, short: Option<Shortage>) -> Result<usize, SolveError> {
        match short {
            Some(Shortage::Reserved(i)) => Ok(i),
            _ => Err(SolveError::NonFinite {
                what: "n_D - n_S beside a change of side",
            }),
        }
    }

    /// κ at the edge of type `worker`'s reserved shortage (docs/unit-1d.md §12 item 16): the
    /// real wage per support basket at which the pool clears, found by bisection on the bit
    /// patterns of doubles over [ζ_b, +∞], where ζ_b is the type's own ζ at the edge's finite
    /// end and f_b < 0 the excess demand there. `f(κ)` is the excess demand with κ in place of
    /// ζ: nondecreasing in κ (a higher walled wage raises P_s and lowers the pool's real wage),
    /// and +∞ at κ = +∞, where the walk has no fixed point.
    fn edge_clearing(
        &self,
        worker: usize,
        (zeta_b, f_b): (f64, f64),
        f: impl Fn(f64) -> f64,
    ) -> Result<(Edge, u32), SolveError> {
        let root = bisect_bits(
            |kappa| -f(kappa),
            (zeta_b, -f_b),
            (f64::INFINITY, -f(f64::INFINITY)),
        )?;
        let edge = Edge {
            worker,
            clearing: root.x,
        };
        Ok((edge, root.steps))
    }

    /// A root on piece `piece` of the wall (docs/unit-1d.md §5.3): bisection of f(ω) between
    /// the piece's ends, whose values are the sequence's, or an exact zero at an end.
    fn wall_piece(
        &self,
        piece: usize,
        (f_lo, f_hi): (f64, f64),
        end: &WallEnd,
        context: &Context,
    ) -> Result<Eq1d, SolveError> {
        let technique = match piece {
            0 => self.machines.envelope().last(),
            s => self.wall[s - 1].above,
        };
        let ends = |v: f64| {
            let q = self.at_wage(BRACKET_HI, v, technique);
            (v, v / q.p_s)
        };
        let (v_lo, omega_lo) = match piece {
            0 => (context.at_one.v, context.at_one.v / context.at_one.p_s),
            s => ends(self.wall[s - 1].wage),
        };
        let (v_hi, omega_hi) = match self.wall.get(piece) {
            Some(w) => ends(w.wage),
            None => (f64::INFINITY, end.omega.unwrap_or(f64::NAN)),
        };
        let (v, steps) = if f_lo == 0.0 {
            (v_lo, 0)
        } else if f_hi == 0.0 {
            (v_hi, 0)
        } else {
            let corner = self.corner(BRACKET_HI, technique);
            let root = bisect_bits(
                |w| corner.n_d - self.supply_at(&corner, w),
                (omega_lo, f_lo),
                (omega_hi, f_hi),
            )?;
            (self.wage_at(&corner, root.x), root.steps)
        };
        let at = Where::Corner {
            margin: Margin::Wall,
            v,
            steps,
        };
        self.report(at, technique, None, None, context)
    }

    /// σ, the share of the machine tasks the type above takes at a tie (docs/unit-1d.md §4.6),
    /// with the prices of `q`, the evaluation at the switch under the type below. When no type
    /// is walled under either pure technique, 1c's closed form with the pool's supply at each:
    /// σ = B_a·f_a/(B_a·f_a − B_b·f_b), 1 when f_b ≥ 0. Otherwise the walled wages move with
    /// Y(σ), and σ is found by bisection of f(σ) on [0, 1] on the doubles' bit patterns. When
    /// that bisection closes on a short mix (+∞), σ is the edge of a reserved shortage: the
    /// bracket's finite end, with the short type's κ from
    /// [`edge_clearing`](Self::edge_clearing) (docs/unit-1d.md §12 item 16).
    fn tie_share(
        &self,
        q: &WorkerPoint,
        below: usize,
        above: usize,
    ) -> Result<(f64, Option<Edge>), SolveError> {
        let types = self.machines.block().types();
        let mut task = vec![0.0; types.len()];
        task[above] = q.m_s / types[above].task_efficiency;
        let b = self.machines.clear(&task);
        let state_a = self.workers_at(q.v, q.base_p_s, q.y, None);
        let state_b = self.workers_at(q.v, q.base_p_s, b.y, None);
        let simple = [&state_a, &state_b]
            .iter()
            .all(|s| s.short.is_none() && !s.walled.contains(&true));
        if simple {
            let land_a = self.params.land / q.y;
            let land_b = self.params.land / b.y;
            let f_a = q.n_d - state_a.pool_supply;
            let f_b = (b.y * q.h_s + b.machine_hours) - state_b.pool_supply;
            if f_b >= 0.0 {
                return Ok((1.0, None));
            }
            return Ok(((land_a * f_a) / (land_a * f_a - land_b * f_b), None));
        }
        let f = |share: f64| self.mixed_excess(q, below, above, share, None);
        let f_one = f(1.0);
        if f_one >= 0.0 {
            return Ok((1.0, None));
        }
        let root = bisect_bits(f, (0.0, f(0.0)), (1.0, f_one))?;
        if !root.jump {
            return Ok((root.x, None));
        }
        // The edge of a reserved shortage in σ.
        let share = root.x;
        let worker = self.short_type(
            self.mixed_state(q, below, above, share.next_down(), None)
                .1
                .short,
        )?;
        let (n_d, state) = self.mixed_state(q, below, above, share, None);
        let (edge, _) = self.edge_clearing(
            worker,
            (state.clearing[worker], n_d - state.pool_supply),
            |kappa| {
                let edge = Edge {
                    worker,
                    clearing: kappa,
                };
                self.mixed_excess(q, below, above, share, Some(edge))
            },
        )?;
        Ok((share, Some(edge)))
    }

    /// f(σ) at a tie (docs/unit-1d.md §4.6): the excess demand when type `above` takes the
    /// share σ of the machine tasks at the point `q`, evaluated at a switch under the type
    /// below (its `technique`), with `q`'s prices and the worker types at the σ-mix's Y; +∞
    /// where the mix is short. σ = 0 is `q` itself.
    pub fn excess_at_share(&self, q: &WorkerPoint, above: usize, share: f64) -> f64 {
        self.mixed_excess(q, q.technique, above, share, None)
    }

    /// f(σ) at a tie: the σ-mix of the two techniques' quantities, the worker types at its Y
    /// (with a type at its edge if one is given), and the prices of `q`; +∞ where a point is
    /// short.
    fn mixed_excess(
        &self,
        q: &WorkerPoint,
        below: usize,
        above: usize,
        share: f64,
        edge: Option<Edge>,
    ) -> f64 {
        let (n_d, state) = self.mixed_state(q, below, above, share, edge);
        if state.short.is_some() {
            f64::INFINITY
        } else {
            n_d - state.pool_supply
        }
    }

    /// The pool's demand and the worker types at the σ-mix of a tie.
    fn mixed_state(
        &self,
        q: &WorkerPoint,
        below: usize,
        above: usize,
        share: f64,
        edge: Option<Edge>,
    ) -> (f64, WorkerState) {
        let types = self.machines.block().types();
        let mut task = vec![0.0; types.len()];
        task[below] = (1.0 - share) * (q.m_s / types[below].task_efficiency);
        task[above] = share * (q.m_s / types[above].task_efficiency);
        let c = self.machines.clear(&task);
        let state = self.workers_at(q.v, q.base_p_s, c.y, edge);
        (c.y * q.h_s + c.machine_hours, state)
    }

    /// docs/unit-1d.md §4.7 at the equilibrium: unit 1c's report in 1c's order of operations,
    /// with the worker types, the reserved costs and the corners, and a type at its edge if
    /// one is given.
    fn report(
        &self,
        at: Where,
        technique: usize,
        tie: Option<Tie>,
        edge: Option<Edge>,
        context: &Context,
    ) -> Result<Eq1d, SolveError> {
        let p = &self.params;
        let m = &self.machines;
        let block = m.block();
        let types = block.types();
        let k_count = block.len();
        let (q, one_minus_x, steps, margin) = match at {
            Where::Line {
                x,
                one_minus_x,
                steps,
            } => (
                self.at_with(x, technique),
                one_minus_x,
                steps,
                Margin::Contestable,
            ),
            Where::Corner { margin, v, steps } => {
                let x = if margin == Margin::AllHuman { 0.0 } else { 1.0 };
                (self.at_wage(x, v, technique), 1.0 - x, steps, margin)
            }
        };
        let on_line = margin == Margin::Contestable;
        let theta = |t: usize| types[t].task_efficiency;
        // The task services per basket, by type: all to τ, or split at a tie.
        let (y, services, machine_hours) = match tie {
            None => (q.y, q.services.clone(), q.machine_hours),
            Some(t) => {
                let mut task = vec![0.0; k_count];
                task[technique] = (1.0 - t.share) * (q.m_s / theta(technique));
                task[t.above] = t.share * (q.m_s / theta(t.above));
                let c = m.clear(&task);
                (c.y, c.services, c.machine_hours)
            }
        };
        // The worker types at the equilibrium's Y: at a tie D_i and the walk move with σ.
        let state = self.workers_at(q.v, q.base_p_s, y, edge);
        if let Some(short) = state.short {
            return Err(SolveError::NonFinite {
                what: match short {
                    Shortage::Reserved(_) => "reserved demand at the equilibrium",
                    Shortage::Ceiling => "the walk at the equilibrium",
                },
            });
        }
        let (reserved_costs, prices) = self.with_reserved(&q.base_prices, &state.wages);
        let p_s = state.p_s;
        let task_services: Vec<f64> = (0..k_count)
            .map(|k| match tie {
                None if k == technique => (y * q.m_s) / theta(k),
                Some(t) if k == technique => (1.0 - t.share) * ((y * q.m_s) / theta(k)),
                Some(t) if k == t.above => t.share * ((y * q.m_s) / theta(k)),
                _ => 0.0,
            })
            .collect();
        // Hours-type outputs take the carried 1 − x* in the top segment (1b §5.1 step 5).
        let count = p.categories.len();
        let yhat = m.basket_outputs();
        let mut human = Vec::with_capacity(count);
        let mut h_s = 0.0;
        for (index, category) in p.categories.iter().enumerate() {
            let (h, _) = tasks(
                &p.edges,
                m.integral_at_edges(),
                &category.density,
                q.x,
                q.j,
                Some(one_minus_x),
            );
            let h = h + p.human_required[index];
            h_s += yhat[index] * h;
            human.push(h);
        }
        let final_hours = y * h_s;
        let n_pool = final_hours + machine_hours;
        let u = block.user_costs();
        let omega = block.wealth_factors();
        let mut interest = 0.0;
        let mut type_interest = Vec::with_capacity(k_count);
        for k in 0..k_count {
            let i = p.rho * omega[k] * q.build[k] * services[k];
            interest += i;
            type_interest.push(i);
        }
        // The worker types' hours (§4.7): reserved demand, and the pool's demand split among
        // the pooled types by their net supplies.
        let worker_types = &p.worker_types;
        let kinds = worker_types.len();
        let pool_share = |i: usize| -> f64 {
            if state.walled[i] || state.pool_supply == 0.0 {
                0.0
            } else {
                let net = worker_types[i].efficiency * (state.supply[i] - state.demand[i]);
                n_pool * (net / state.pool_supply)
            }
        };
        let pool: Vec<f64> = (0..kinds).map(pool_share).collect();
        let hours: Vec<f64> = (0..kinds)
            .map(|i| {
                let e = worker_types[i].efficiency;
                if state.walled[i] || e == 0.0 {
                    state.demand[i]
                } else {
                    state.demand[i] + pool[i] / e
                }
            })
            .collect();
        let mut n_a = 0.0;
        let mut reserved_bill = 0.0;
        let mut reserved_hours = 0.0;
        for ((h, w), d) in hours.iter().zip(&state.wages).zip(&state.demand) {
            n_a += h;
            reserved_bill += w * d;
            reserved_hours += d;
        }
        let wage_bill = q.v * n_pool + reserved_bill;
        let income = wage_bill + p.land + interest;
        let support_cost = self.support * p_s;
        let provider_baskets = (p.land + interest) / p_s - self.support;
        let flow_prices = u.iter().all(|&uk| uk == 1.0);
        // γ(x*) on the line; at a corner the pool's wage in machine-task units, v/π.
        let g_margin = if on_line { q.gamma } else { q.v / q.task_price };
        let phi_w = if flow_prices {
            block.labor_share_of_price(g_margin, technique)
        } else {
            None
        };
        let missing = vec![f64::NAN; k_count];
        let (lt_m, bt_m) = block
            .price_totals()
            .unwrap_or((missing.as_slice(), missing.as_slice()));
        let (ltq_m, btq_m) = (block.lambda_tilde_q(), block.b_tilde_q());
        let per_task = |totals: &[f64]| -> f64 {
            match tie {
                None => totals[technique] / theta(technique),
                Some(t) => {
                    (1.0 - t.share) * (totals[technique] / theta(technique))
                        + t.share * (totals[t.above] / theta(t.above))
                }
            }
        };
        let (lt_coef, bt_coef) = (per_task(lt_m), per_task(bt_m));
        let (ltq_coef, btq_coef) = (per_task(ltq_m), per_task(btq_m));
        let chain = |rhs: Vec<f64>| m.chain().solve(&rhs);
        let l_star = chain(
            (0..count)
                .map(|j| q.human[j] + q.machine[j] / g_margin)
                .collect(),
        );
        let lambda_tilde = chain(
            (0..count)
                .map(|j| human[j] + q.machine[j] * lt_coef)
                .collect(),
        );
        let b_tilde = chain(
            (0..count)
                .map(|j| p.categories[j].direct_land + q.machine[j] * bt_coef)
                .collect(),
        );
        let lambda_tilde_q = chain(
            (0..count)
                .map(|j| human[j] + q.machine[j] * ltq_coef)
                .collect(),
        );
        let b_tilde_q = chain(
            (0..count)
                .map(|j| p.categories[j].direct_land + q.machine[j] * btq_coef)
                .collect(),
        );
        let mut categories = Vec::with_capacity(count);
        let (mut l_star_s, mut l_s, mut b_s, mut l_s_q, mut b_s_q) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut spending, mut fork, mut totals) = (0.0, 0.0, 0.0);
        for (index, category) in p.categories.iter().enumerate() {
            let (price, mm, z) = (prices[index], q.machine[index], category.weight);
            let r = reserved_costs[index];
            let b_bar = m.chain_land()[index];
            let l_bar = m.all_human_hours()[index] + self.required_chain[index];
            let output = z * y;
            let gross_output = yhat[index] * y;
            let fixed = b_tilde[index] + r;
            let c = CategoryEq1d {
                price,
                real_wage: q.v / price,
                l_star: l_star[index],
                l_bar,
                human: human[index],
                machine: mm,
                lambda_tilde: lambda_tilde[index],
                b_tilde: b_tilde[index],
                lambda_tilde_q: lambda_tilde_q[index],
                b_tilde_q: b_tilde_q[index],
                output,
                final_hours: gross_output * human[index],
                machine_services: gross_output * mm,
                share: z * price / p_s,
                wage_floor: 1.0 / (l_bar + (b_bar + r) / q.v),
                wage_ceiling: (fixed > 0.0).then(|| q.v / fixed),
                phi_w: flow_prices.then(|| (q.v * lambda_tilde[index] + r) / price),
                phi_r: flow_prices.then(|| b_tilde[index] / price),
                chain_land: b_bar,
                gross_output,
                human_required: self.required_chain[index],
                reserved_cost: r,
            };
            l_star_s += z * c.l_star;
            l_s += z * c.lambda_tilde;
            b_s += z * c.b_tilde;
            l_s_q += z * c.lambda_tilde_q;
            b_s_q += z * c.b_tilde_q;
            spending += price * output;
            fork = worse(fork, (price - ((q.v * c.l_star + b_bar) + r)).abs() / price);
            totals = worse(
                totals,
                (price - ((q.v * c.lambda_tilde + c.b_tilde) + r)).abs() / price,
            );
            categories.push(c);
        }
        let margin_active = on_line && {
            let segment = segment_of(&p.edges, q.x);
            p.categories
                .iter()
                .enumerate()
                .any(|(j, c)| yhat[j] > 0.0 && c.density[segment] > 0.0)
        };
        // The machine types.
        let mut type_eqs = Vec::with_capacity(k_count);
        for k in 0..k_count {
            let t = &types[k];
            type_eqs.push(TypeEq {
                price: q.type_prices[k],
                operating_cost: q.operating[k],
                build_cost: q.build[k],
                user_cost: u[k],
                wealth_factor: omega[k],
                delivered_cost: (t.task_efficiency > 0.0)
                    .then(|| q.type_prices[k] / t.task_efficiency),
                closure_wage: block.closure_wage(g_margin, k),
                lambda_tilde: lt_m[k],
                b_tilde: bt_m[k],
                lambda_tilde_q: ltq_m[k],
                b_tilde_q: btq_m[k],
                services: services[k],
                task_services: task_services[k],
                builds: t.delta * services[k],
                hours: block.lambda_q()[k] * services[k],
                land: block.land_q()[k] * services[k],
                wealth: omega[k] * q.build[k] * services[k],
                interest: type_interest[k],
                phi_w: (u[k] == 1.0).then(|| q.v * lt_m[k] / q.type_prices[k]),
            });
        }
        // The worker types' outputs.
        let workers: Vec<WorkerEq> = (0..kinds)
            .map(|i| {
                let t = &worker_types[i];
                let wage = state.wages[i];
                let pooled = !state.walled[i];
                WorkerEq {
                    wage,
                    real_wage: wage / p_s,
                    efficiency: t.efficiency,
                    pooled,
                    premium: (t.efficiency > 0.0).then(|| wage / (t.efficiency * q.v)),
                    hours: hours[i],
                    reserved_hours: state.demand[i],
                    pool_hours: if pooled && t.efficiency > 0.0 {
                        pool[i] / t.efficiency
                    } else {
                        0.0
                    },
                    supply: state.supply[i],
                    participation: (hours[i] / t.workers).min(1.0),
                    clearing_real_wage: state.clearing[i],
                    marginal_work_cost: num::ln1p(wage / (t.support * p_s)),
                    at_wall: !pooled || margin == Margin::Wall,
                    edge: edge.is_some_and(|e| e.worker == i),
                }
            })
            .collect();
        let gamma_top = p.schedule.gamma(BRACKET_HI);
        let gamma_bottom = p.schedule.gamma(0.0);
        let residuals = self.residuals(&ResidualInputs {
            q: &q,
            state: &state,
            prices: &prices,
            technique,
            tie,
            margin,
            y,
            services: &services,
            task_services: &task_services,
            n_pool,
            income,
            spending,
            fork,
            totals,
            replacement: (gamma_bottom * q.task_price, gamma_top * q.task_price),
        });
        let at_one = context.at_one;
        Ok(Eq1d {
            x_star: q.x,
            one_minus_x_star: one_minus_x,
            gamma_star: q.gamma,
            m_s: q.m_s,
            u: u[technique],
            v: q.v,
            p_s,
            y,
            final_hours,
            machine_hours,
            n_a,
            participation: (n_a / self.workers).min(1.0),
            income,
            interest,
            labor_share: wage_bill / income,
            capital_share: interest / income,
            real_wage: q.v / p_s,
            support_cost,
            worker_baskets: self.support + wage_bill / p_s,
            provider_baskets,
            funded: provider_baskets > 0.0,
            lemma_b1: at_one.short.is_none()
                && at_one.n_s > at_one.n_d
                && p.land > self.support * at_one.p_s,
            phi_w,
            phi_r: phi_w.map(|w| 1.0 - w),
            h_s,
            b_d: q.b_d,
            l_star_s,
            l_s,
            b_s,
            l_s_q,
            b_s_q,
            rent_ceiling: q.v / b_s,
            margin_active,
            residuals,
            bisection_steps: steps,
            d: q.d,
            technique,
            tie,
            switches: m
                .envelope()
                .switches
                .iter()
                .zip(context.points)
                .map(|(s, &x)| SwitchPoint {
                    gamma: s.gamma,
                    x,
                    below: s.below,
                    above: s.above,
                })
                .collect(),
            types: type_eqs,
            categories,
            margin,
            g: q.v / q.task_price,
            replacement_top: gamma_top * q.task_price,
            replacement_bottom: gamma_bottom * q.task_price,
            n_pool,
            required_hours: y * self.required_per_basket,
            reserved_hours,
            wage_bill,
            f_line_0: finite_or_none(context.f_zero),
            f_line_lo: finite_or_none(context.f_lo),
            f_line_1: finite_or_none(context.f_one),
            f_end: context.f_end,
            wall_switches: self.wall.clone(),
            workers,
        })
    }

    /// docs/unit-1d.md §6's residuals, each recomputed from the recipes in a fixed order, so
    /// that for one type 1c's twelve are its own bit for bit and the new three are 0.
    fn residuals(&self, r: &ResidualInputs) -> Residuals1d {
        let p = &self.params;
        let m = &self.machines;
        let block = m.block();
        let types = block.types();
        let q = r.q;
        let k_count = block.len();
        let u = block.user_costs();
        // Land: |(B_ŷ·Y + Σ_k b^q_k·X_k) − T|/T.
        let mut machine_land = 0.0;
        for k in 0..k_count {
            machine_land += block.land_q()[k] * r.services[k];
        }
        let land = (q.b_d * r.y + machine_land - p.land).abs() / p.land;
        // Services: |(X_k − task_k) − Σ_l A^q_lk·X_l|/X_k, 0 when X_k = 0.
        let mut services = 0.0;
        for k in 0..k_count {
            let x_k = r.services[k];
            let residual = if x_k > 0.0 {
                let mut used = 0.0;
                for l in 0..k_count {
                    used += block.a_q(l, k) * r.services[l];
                }
                ((x_k - r.task_services[k]) - used).abs() / x_k
            } else {
                0.0
            };
            services = worse(services, residual);
        }
        // The user-cost rows.
        let mut user_cost = 0.0;
        for k in 0..k_count {
            let (op, build) = (&types[k].operating, &types[k].build);
            let mut o = 0.0;
            let mut v_k = 0.0;
            for l in 0..k_count {
                o += op.machines[l] * q.type_prices[l];
                v_k += build.machines[l] * q.type_prices[l];
            }
            let o = o + op.labor * q.v + op.land;
            let v_k = v_k + build.labor * q.v + build.land;
            let price = q.type_prices[k];
            user_cost = worse(user_cost, (price - (o + u[k] * v_k)).abs() / price);
        }
        let task_share = |k: usize| -> f64 {
            match r.tie {
                None if k == r.technique => 1.0,
                Some(t) if k == r.technique => 1.0 - t.share,
                Some(t) if k == t.above => t.share,
                _ => 0.0,
            }
        };
        // p = Ap + λv + b + R over the C + K rows (SSRN eq 2-3; A.1), with each category's
        // direct reserved cost Σ_i v_i·R_ji.
        let count = p.categories.len();
        let mut leontief_price = 0.0;
        for j in 0..count {
            let mut inputs = 0.0;
            for l in 0..count {
                inputs += p.intermediate[j][l] * r.prices[l];
            }
            for (k, t) in types.iter().enumerate() {
                let share = task_share(k);
                if share > 0.0 {
                    inputs += (share * (q.machine[j] / t.task_efficiency)) * q.type_prices[k];
                }
            }
            let mut reserved = 0.0;
            for (i, &w) in r.state.wages.iter().enumerate() {
                reserved += w * p.reserved[j][i];
            }
            let cost = (inputs + q.human[j] * q.v + p.categories[j].direct_land) + reserved;
            leontief_price = worse(leontief_price, (r.prices[j] - cost).abs() / r.prices[j]);
        }
        for k in 0..k_count {
            let mut inputs = 0.0;
            for l in 0..k_count {
                inputs += block.a_hat(k, l) * q.type_prices[l];
            }
            let cost = inputs + block.lambda_hat()[k] * q.v + block.land_hat()[k];
            let price = q.type_prices[k];
            leontief_price = worse(leontief_price, (price - cost).abs() / price);
        }
        // y = A^qᵀy + f over the rows with y_i > 0.
        let gross: Vec<f64> = m.basket_outputs().iter().map(|b| b * r.y).collect();
        let mut leontief_quantity = 0.0;
        for (j, (&y_j, category)) in gross.iter().zip(&p.categories).enumerate() {
            if y_j > 0.0 {
                let mut used = 0.0;
                for (row, &y_l) in p.intermediate.iter().zip(&gross) {
                    used += row[j] * y_l;
                }
                let demand = used + category.weight * r.y;
                leontief_quantity = worse(leontief_quantity, (y_j - demand).abs() / y_j);
            }
        }
        for (k, (t, &x_k)) in types.iter().zip(r.services).enumerate() {
            if x_k > 0.0 {
                let mut used = 0.0;
                let share = task_share(k);
                if share > 0.0 {
                    for (&y_j, &m_j) in gross.iter().zip(&q.machine) {
                        used += y_j * (share * (m_j / t.task_efficiency));
                    }
                }
                for (l, &x_l) in r.services.iter().enumerate() {
                    used += block.a_q(l, k) * x_l;
                }
                leontief_quantity = worse(leontief_quantity, (x_k - used).abs() / x_k);
            }
        }
        // SSRN A.1's closure for τ on the line; a corner has no task margin.
        let closure = if r.margin == Margin::Contestable {
            match block.closure_wage(q.gamma, r.technique) {
                Some(w) => (q.v - w).abs() / q.v,
                None => f64::NAN,
            }
        } else {
            0.0
        };
        let mut cheapest = 0.0;
        for (t, mt) in types.iter().enumerate() {
            if mt.task_efficiency > 0.0 {
                let undercut =
                    (q.task_price - q.type_prices[t] / mt.task_efficiency) / q.task_price;
                cheapest = worse(cheapest, undercut.max(0.0));
            }
        }
        // The corner's inequality against the replacement value at its end of the line.
        let (bottom, top) = r.replacement;
        let corner = match r.margin {
            Margin::Contestable => 0.0,
            Margin::Wall => (top - q.v).max(0.0) / q.v,
            Margin::AllHuman => (q.v - bottom).max(0.0) / q.v,
        };
        // The walled types' reserved markets.
        let mut reserved = 0.0;
        for i in 0..r.state.walled.len() {
            if r.state.walled[i] {
                let d = r.state.demand[i];
                reserved = worse(reserved, (r.state.supply[i] - d).abs() / d);
            }
        }
        // The basket: P_s against Σ_j z_j·p_j.
        let mut spent = 0.0;
        for (category, &price) in p.categories.iter().zip(r.prices) {
            spent += category.weight * price;
        }
        let basket = (r.state.p_s - spent).abs() / r.state.p_s;
        Residuals1d {
            labor: (r.n_pool - r.state.pool_supply).abs(),
            income: (r.y * r.state.p_s - r.income).abs() / r.income,
            land,
            services,
            user_cost,
            fork: r.fork,
            totals: r.totals,
            expenditure: (r.spending - r.income).abs() / r.income,
            leontief_price,
            leontief_quantity,
            closure,
            cheapest,
            corner,
            reserved,
            basket,
        }
    }
}

/// The value of the sequence at a point: +∞ where it is short, else f = n_D − S, which must
/// be finite.
fn value(what: &'static str, q: &WorkerPoint) -> Result<f64, SolveError> {
    if q.short.is_some() {
        return Ok(f64::INFINITY);
    }
    let f = q.n_d - q.n_s;
    if f.is_finite() {
        Ok(f)
    } else {
        Err(SolveError::NonFinite { what })
    }
}

/// `Some(f)` for a finite value of the sequence, `None` for a short point (+∞).
fn finite_or_none(f: f64) -> Option<f64> {
    f.is_finite().then_some(f)
}

/// A value of the sequence of docs/unit-1d.md §5.3 step 2.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Zero,
    Lo,
    SwitchBelow(usize),
    SwitchAbove(usize),
    One,
    WallBelow(usize),
    WallAbove(usize),
    End,
}

/// Where the report is evaluated.
#[derive(Clone, Copy, Debug)]
enum Where {
    /// On the line at x, with the carried 1 − x*.
    Line {
        x: f64,
        one_minus_x: f64,
        steps: u32,
    },
    /// At a corner with the pool's wage v.
    Corner { margin: Margin, v: f64, steps: u32 },
}

impl Where {
    fn line(root: Root) -> Where {
        Where::Line {
            x: root.x,
            one_minus_x: root.one_minus_x,
            steps: root.steps,
        }
    }
}

/// What the report needs from the solve.
struct Context<'a> {
    points: &'a [f64],
    at_one: &'a WorkerPoint,
    f_zero: f64,
    f_lo: f64,
    f_one: f64,
    f_end: f64,
}

/// A corner's quantities (docs/unit-1d.md §4.5).
struct Corner {
    n_d: f64,
    demand: Vec<f64>,
    clearing: Vec<f64>,
    short: Option<Shortage>,
    l_s: f64,
    b_s: f64,
}

/// The inputs of [`WorkerEconomy::residuals`].
struct ResidualInputs<'a> {
    q: &'a WorkerPoint,
    state: &'a WorkerState,
    prices: &'a [f64],
    technique: usize,
    tie: Option<Tie>,
    margin: Margin,
    y: f64,
    services: &'a [f64],
    task_services: &'a [f64],
    n_pool: f64,
    income: f64,
    spending: f64,
    fork: f64,
    totals: f64,
    /// (γ(0)·π, γ(1)·π).
    replacement: (f64, f64),
}

/// Prices and quantities at one point of the path (docs/unit-1d.md §5.1), with r = 1.
/// Per-type vectors are in type order (machine types, then worker types where named),
/// per-category ones in category order.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerPoint {
    /// x.
    pub x: f64,
    /// γ(x).
    pub gamma: f64,
    /// J(x).
    pub j: f64,
    /// τ, the task type doing the machine tasks.
    pub technique: usize,
    /// The least pivot of the system that priced the machines: the margin's (O, V) system on
    /// the line, the wage-given one at a corner.
    pub d: f64,
    /// p_k per machine type.
    pub type_prices: Vec<f64>,
    /// O_k per machine type.
    pub operating: Vec<f64>,
    /// V_k per machine type.
    pub build: Vec<f64>,
    /// v, the pool's wage per efficiency hour.
    pub v: f64,
    /// π = p_τ/θ_τ.
    pub task_price: f64,
    /// p⁰_j, each category's price without the reserved costs.
    pub base_prices: Vec<f64>,
    /// R_j = Σ_i v_i·R̃_ji, each category's reserved costs through the chain.
    pub reserved_costs: Vec<f64>,
    /// p_j = p⁰_j + R_j.
    pub prices: Vec<f64>,
    /// h_j = H_j + L^H_j, efficiency hours at pool tasks per unit of category j.
    pub human: Vec<f64>,
    /// M_j, machine task services per unit of category j, at efficiency 1.
    pub machine: Vec<f64>,
    /// P⁰_s = Σ z_j·p⁰_j.
    pub base_p_s: f64,
    /// P_s, the walk's least fixed point (NaN where short).
    pub p_s: f64,
    /// H_ŷ = Σ ŷ_j·h_j, with the common human-required hours.
    pub h_s: f64,
    /// M_ŷ.
    pub m_s: f64,
    /// B_ŷ.
    pub b_d: f64,
    /// Y.
    pub y: f64,
    /// X_k per machine type.
    pub services: Vec<f64>,
    /// Y·H_ŷ.
    pub final_hours: f64,
    /// Σ_k λ^q_k·X_k.
    pub machine_hours: f64,
    /// n_D, the pool's demand in efficiency hours.
    pub n_d: f64,
    /// D_i = Y·R_ŷi per worker type.
    pub reserved_demand: Vec<f64>,
    /// ζ_i, the real wage per support basket that clears type i's reserved market.
    pub clearing: Vec<f64>,
    /// Whether each worker type is at its wall.
    pub walled: Vec<bool>,
    /// v_i per worker type (NaN where short).
    pub wages: Vec<f64>,
    /// n_S,i per worker type (NaN where short).
    pub supply: Vec<f64>,
    /// S, the pool's net supply in efficiency hours (NaN where short).
    pub n_s: f64,
    /// Why the point has no finite excess demand, if it has none.
    pub short: Option<Shortage>,
}

impl WorkerPoint {
    /// f = n_D − S, or +∞ where the point is short.
    pub fn excess_demand(&self) -> f64 {
        if self.short.is_some() {
            f64::INFINITY
        } else {
            self.n_d - self.n_s
        }
    }
}

/// One worker type at the equilibrium (docs/unit-1d.md §4.7).
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerEq {
    /// v_i, the wage per hour: ε_i·v when pooled, ζ_i·ν_i·P_s at its wall.
    pub wage: f64,
    /// v_i/P_s.
    pub real_wage: f64,
    /// ε_i.
    pub efficiency: f64,
    /// Whether the type sells hours to the pool (not at its own wall).
    pub pooled: bool,
    /// v_i/(ε_i·v), 1 when pooled; `None` at ε_i = 0.
    pub premium: Option<f64>,
    /// Hours worked: D_i plus the type's share of the pool's hours.
    pub hours: f64,
    /// D_i = Y·R_ŷi, hours at its reserved tasks.
    pub reserved_hours: f64,
    /// The type's hours at pool tasks, its share of n_pool over ε_i (0 when walled).
    pub pool_hours: f64,
    /// n_S,i = N_i·F_i(ln(1 + v_i/(ν_i·P_s))) at the reported prices.
    pub supply: f64,
    /// hours/N_i, capped at 1.
    pub participation: f64,
    /// ζ_i = expm1(χ_max,i·D_i/N_i); κ_i for a type at its edge.
    pub clearing_real_wage: f64,
    /// ln(1 + v_i/(ν_i·P_s)), the marginal worker's work cost, at which the exit value
    /// (e^χ − 1)·ν_i·P_s equals v_i.
    pub marginal_work_cost: f64,
    /// At its own wall, or pooled with the pool's margin at the wall.
    pub at_wall: bool,
    /// At the edge of its reserved shortage ([`Edge`]): its reserved demand takes all its
    /// workers (D_i = N_i to rounding), and its wage κ_i·ν_i·P_s is set by the pool's clearing,
    /// κ_i at least the ζ_i its demand implies (docs/unit-1d.md §12 item 16).
    pub edge: bool,
}

/// One category at the equilibrium (docs/unit-1d.md §4.7): unit 1c's
/// [`CategoryEq1c`](crate::CategoryEq1c) with the pool's hours and the reserved costs.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryEq1d {
    /// p_j, with the reserved costs.
    pub price: f64,
    /// v/p_j.
    pub real_wage: f64,
    /// L*_j = ((I − A_cc)⁻¹(h + M/g))_j, g = γ(x*) on the line, v/π at a corner.
    pub l_star: f64,
    /// L̄_j + L̃^H_j, the pool's all-human hours through the chain.
    pub l_bar: f64,
    /// h_j = H_j + L^H_j, with the carried 1 − x*.
    pub human: f64,
    /// M_j.
    pub machine: f64,
    /// λ̃_j, the pool's total hours per unit, price side.
    pub lambda_tilde: f64,
    /// b̃_j, total land per unit, price side.
    pub b_tilde: f64,
    /// λ̃^q_j, clearing side.
    pub lambda_tilde_q: f64,
    /// b̃^q_j, clearing side.
    pub b_tilde_q: f64,
    /// z_j·Y.
    pub output: f64,
    /// Gross output times h_j.
    pub final_hours: f64,
    /// Gross output times M_j.
    pub machine_services: f64,
    /// z_j·p_j/P_s.
    pub share: f64,
    /// 1/(L̄_j + L̃^H_j + (b̄_j + R_j)/v).
    pub wage_floor: f64,
    /// v/(b̃_j + R_j), when positive.
    pub wage_ceiling: Option<f64>,
    /// (v·λ̃_j + R_j)/p_j, labour's share of the price, pool and reserved, when every u_k = 1.
    pub phi_w: Option<f64>,
    /// b̃_j/p_j, when every u_k = 1.
    pub phi_r: Option<f64>,
    /// b̄_j.
    pub chain_land: f64,
    /// ŷ_j·Y.
    pub gross_output: f64,
    /// L̃^H_j, the common human-required hours through the chain.
    pub human_required: f64,
    /// R_j = Σ_i v_i·R̃_ji, the reserved costs through the chain.
    pub reserved_cost: f64,
}

/// How far a unit-1d solution is from the equations it must satisfy (docs/unit-1d.md §6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Residuals1d {
    /// |n_pool − S|, the pool's clearing in efficiency hours (1a's for one type).
    pub labor: f64,
    /// |Y·P_s − I|/I.
    pub income: f64,
    /// |(B_ŷ·Y + Σ_k b^q_k·X_k) − T|/T.
    pub land: f64,
    /// The largest |(X_k − task_k) − Σ_l A^q_lk·X_l|/X_k.
    pub services: f64,
    /// The largest |p_k − (O_k + u_k·V_k)|/p_k.
    pub user_cost: f64,
    /// The largest |p_j − (v·L*_j + b̄_j + R_j)|/p_j.
    pub fork: f64,
    /// The largest |p_j − (v·λ̃_j + b̃_j + R_j)|/p_j.
    pub totals: f64,
    /// |Σ_j p_j·z_j·Y − I|/I.
    pub expenditure: f64,
    /// The largest |p_i − (Ap + λv + b + R)_i|/p_i over the C + K rows.
    pub leontief_price: f64,
    /// The largest |y_i − (A^qᵀy + f)_i|/y_i.
    pub leontief_quantity: f64,
    /// |v − γb̃_τ/(θ_τ − γλ̃_τ)|/v on the line; 0 at a corner.
    pub closure: f64,
    /// The largest (π − p_t/θ_t)⁺/π over task types.
    pub cheapest: f64,
    /// (γ(1)·π − v)⁺/v at the wall, (v − γ(0)·π)⁺/v at the all-human corner, 0 on the line.
    pub corner: f64,
    /// The largest |n_S,i − D_i|/D_i over the walled types (0 if none).
    pub reserved: f64,
    /// |P_s − Σ_j z_j·p_j|/P_s.
    pub basket: f64,
}

/// An equilibrium of unit 1d (docs/unit-1d.md §4.7), in r = 1 units: unit 1c's
/// [`Eq1c`](crate::Eq1c) fields, with the meanings of §4.7 (the one-type case is 1c's), and
/// the worker types, the corners and the path.
#[derive(Clone, Debug, PartialEq)]
pub struct Eq1d {
    /// x*: the root or the switch point on the line; 1.0 at the wall, 0.0 at the all-human
    /// corner.
    pub x_star: f64,
    /// 1 − x*, carried on the line; exact at a corner.
    pub one_minus_x_star: f64,
    /// γ(x*).
    pub gamma_star: f64,
    /// M_ŷ at x*.
    pub m_s: f64,
    /// u of the technique.
    pub u: f64,
    /// v, the pool's wage per efficiency hour.
    pub v: f64,
    /// P_s.
    pub p_s: f64,
    /// Y.
    pub y: f64,
    /// Y·H_ŷ, with the human-required tail and the carried 1 − x*.
    pub final_hours: f64,
    /// Σ_k λ^q_k·X_k.
    pub machine_hours: f64,
    /// Σ_i hours_i, hours worked by every type.
    pub n_a: f64,
    /// n_a/Σ N_i, capped at 1.
    pub participation: f64,
    /// I = v·n_pool + Σ_i v_i·D_i + T + interest.
    pub income: f64,
    /// Σ_k ρ·W_k.
    pub interest: f64,
    /// wage bill/I.
    pub labor_share: f64,
    /// interest/I.
    pub capital_share: f64,
    /// v/P_s, the pool's real wage per efficiency hour.
    pub real_wage: f64,
    /// ν·P_s.
    pub support_cost: f64,
    /// ν + wage bill/P_s.
    pub worker_baskets: f64,
    /// (T + interest)/P_s − ν.
    pub provider_baskets: f64,
    /// provider_baskets > 0.
    pub funded: bool,
    /// f_line(1) < 0 and T > ν·P_s(1), SSRN eq 25 with the support ν.
    pub lemma_b1: bool,
    /// g·λ̃_τ/θ_τ, labour's share of the technique's price, when every u_k = 1.
    pub phi_w: Option<f64>,
    /// 1 − φ_w.
    pub phi_r: Option<f64>,
    /// H_ŷ with the tail.
    pub h_s: f64,
    /// B_ŷ.
    pub b_d: f64,
    /// zᵀL*.
    pub l_star_s: f64,
    /// L_s = zᵀλ̃_c, so P_s = v·L_s + B_s + Σ_i v_i·R_ŷi.
    pub l_s: f64,
    /// B_s = zᵀb̃_c.
    pub b_s: f64,
    /// L_s^q, so n_pool = T·L_s^q/B_s^q.
    pub l_s_q: f64,
    /// B_s^q, so Y = T/B_s^q.
    pub b_s_q: f64,
    /// v/B_s.
    pub rent_ceiling: f64,
    /// Whether a bought category has tasks on the segment holding x* (false at a corner).
    pub margin_active: bool,
    /// The residuals of docs/unit-1d.md §6.
    pub residuals: Residuals1d,
    /// Steps of the bisection that found the equilibrium, x's and κ's together at the edge of a
    /// reserved shortage on the line; 0 at a tie or an exact zero.
    pub bisection_steps: u32,
    /// The least pivot of the system that priced the machines.
    pub d: f64,
    /// τ: the technique, the type below at a tie, the cheapest type at the all-human corner.
    pub technique: usize,
    /// The tie, when the equilibrium sits on a switch of the line or of the wall.
    pub tie: Option<Tie>,
    /// The envelope's switches on the line.
    pub switches: Vec<SwitchPoint>,
    /// Every machine type, in order.
    pub types: Vec<TypeEq>,
    /// Every category, in order.
    pub categories: Vec<CategoryEq1d>,
    /// Which stretch holds the equilibrium.
    pub margin: Margin,
    /// g = v/π, the pool's wage in machine-task units: γ(x*) on the line, above γ(1) at the
    /// wall, below γ(0) at the all-human corner.
    pub g: f64,
    /// γ(1)·π, labour's replacement value at the top task.
    pub replacement_top: f64,
    /// γ(0)·π, labour's replacement value at the bottom task.
    pub replacement_bottom: f64,
    /// n_D, the pool's hours in efficiency units (equal to `n_a` for one type of efficiency 1).
    pub n_pool: f64,
    /// Y·L^H_ŷ, the common human-required hours.
    pub required_hours: f64,
    /// Σ_i D_i, the reserved hours.
    pub reserved_hours: f64,
    /// v·n_pool + Σ_i v_i·D_i.
    pub wage_bill: f64,
    /// f on the line at x = 0 under the first technique; `None` where it is short (+∞).
    pub f_line_0: Option<f64>,
    /// f on the line at [`BRACKET_LO`] (1a's f_at_0); `None` where it is short.
    pub f_line_lo: Option<f64>,
    /// f on the line at x = 1 (1a's f_at_1); `None` where it is short.
    pub f_line_1: Option<f64>,
    /// f_∞, the excess demand at the end of the wall.
    pub f_end: f64,
    /// The envelope's switches on the wall.
    pub wall_switches: Vec<WallSwitch>,
    /// Every worker type, in order.
    pub workers: Vec<WorkerEq>,
}

impl Eq1d {
    /// Every output: unit 1c's economy keys in 1c's order, then unit 1d's, then each line
    /// switch's, each machine type's and each category's (1c's keys, then 1d's), then each
    /// worker type's and each wall switch's.
    ///
    /// This is the one list of outputs: [`WorkerEconomy::solve`] checks every number in it for
    /// finiteness.
    pub fn outputs(&self) -> Vec<(OutputKey1c, Output1b)> {
        use Output1b::{Count, Flag, Float, Optional};
        let top = |name| OutputKey1c {
            item: Item::Economy,
            name,
        };
        let r = &self.residuals;
        let mut list = vec![
            (top("x_star"), Float(self.x_star)),
            (top("one_minus_x_star"), Float(self.one_minus_x_star)),
            (top("gamma_star"), Float(self.gamma_star)),
            (top("m_s"), Float(self.m_s)),
            (top("u"), Float(self.u)),
            (top("v"), Float(self.v)),
            (top("p_s"), Float(self.p_s)),
            (top("y"), Float(self.y)),
            (top("final_hours"), Float(self.final_hours)),
            (top("machine_hours"), Float(self.machine_hours)),
            (top("n_a"), Float(self.n_a)),
            (top("participation"), Float(self.participation)),
            (top("income"), Float(self.income)),
            (top("interest"), Float(self.interest)),
            (top("labor_share"), Float(self.labor_share)),
            (top("capital_share"), Float(self.capital_share)),
            (top("real_wage"), Float(self.real_wage)),
            (top("support_cost"), Float(self.support_cost)),
            (top("worker_baskets"), Float(self.worker_baskets)),
            (top("provider_baskets"), Float(self.provider_baskets)),
            (top("funded"), Flag(self.funded)),
            (top("lemma_b1"), Flag(self.lemma_b1)),
            (top("phi_w"), Optional(self.phi_w)),
            (top("phi_r"), Optional(self.phi_r)),
            (top("h_s"), Float(self.h_s)),
            (top("b_d"), Float(self.b_d)),
            (top("l_star_s"), Float(self.l_star_s)),
            (top("l_s"), Float(self.l_s)),
            (top("b_s"), Float(self.b_s)),
            (top("l_s_q"), Float(self.l_s_q)),
            (top("b_s_q"), Float(self.b_s_q)),
            (top("rent_ceiling"), Float(self.rent_ceiling)),
            (top("margin_active"), Flag(self.margin_active)),
            (top("res_labor"), Float(r.labor)),
            (top("res_income"), Float(r.income)),
            (top("res_land"), Float(r.land)),
            (top("res_services"), Float(r.services)),
            (top("res_user_cost"), Float(r.user_cost)),
            (top("res_fork"), Float(r.fork)),
            (top("res_totals"), Float(r.totals)),
            (top("res_expenditure"), Float(r.expenditure)),
            (top("bisection_steps"), Count(self.bisection_steps)),
            (top("d"), Float(self.d)),
            (top("technique"), Count(self.technique as u32)),
            (top("tie"), Flag(self.tie.is_some())),
            (top("tie_above"), Optional(self.tie.map(|t| t.above as f64))),
            (top("tie_share"), Optional(self.tie.map(|t| t.share))),
            (top("tie_gamma"), Optional(self.tie.map(|t| t.gamma))),
            (top("res_leontief_price"), Float(r.leontief_price)),
            (top("res_leontief_quantity"), Float(r.leontief_quantity)),
            (top("res_closure"), Float(r.closure)),
            (top("res_cheapest"), Float(r.cheapest)),
            // Unit 1d's.
            (top("margin"), Count(self.margin.code())),
            (top("g"), Float(self.g)),
            (top("replacement_top"), Float(self.replacement_top)),
            (top("replacement_bottom"), Float(self.replacement_bottom)),
            (top("n_pool"), Float(self.n_pool)),
            (top("required_hours"), Float(self.required_hours)),
            (top("reserved_hours"), Float(self.reserved_hours)),
            (top("wage_bill"), Float(self.wage_bill)),
            (top("f_line_0"), Optional(self.f_line_0)),
            (top("f_line_lo"), Optional(self.f_line_lo)),
            (top("f_line_1"), Optional(self.f_line_1)),
            (top("f_end"), Float(self.f_end)),
            (top("res_corner"), Float(r.corner)),
            (top("res_reserved"), Float(r.reserved)),
            (top("res_basket"), Float(r.basket)),
        ];
        for (i, s) in self.switches.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Switch(i),
                name,
            };
            list.extend([
                (key("gamma"), Float(s.gamma)),
                (key("x"), Float(s.x)),
                (key("below"), Count(s.below as u32)),
                (key("above"), Count(s.above as u32)),
            ]);
        }
        for (k, t) in self.types.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Type(k),
                name,
            };
            list.extend([
                (key("price"), Float(t.price)),
                (key("operating_cost"), Float(t.operating_cost)),
                (key("build_cost"), Float(t.build_cost)),
                (key("user_cost"), Float(t.user_cost)),
                (key("wealth_factor"), Float(t.wealth_factor)),
                (key("delivered_cost"), Optional(t.delivered_cost)),
                (key("closure_wage"), Optional(t.closure_wage)),
                (key("lambda_tilde"), Float(t.lambda_tilde)),
                (key("b_tilde"), Float(t.b_tilde)),
                (key("lambda_tilde_q"), Float(t.lambda_tilde_q)),
                (key("b_tilde_q"), Float(t.b_tilde_q)),
                (key("services"), Float(t.services)),
                (key("task_services"), Float(t.task_services)),
                (key("builds"), Float(t.builds)),
                (key("hours"), Float(t.hours)),
                (key("land"), Float(t.land)),
                (key("wealth"), Float(t.wealth)),
                (key("interest"), Float(t.interest)),
                (key("phi_w"), Optional(t.phi_w)),
            ]);
        }
        for (j, c) in self.categories.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Category(j),
                name,
            };
            list.extend([
                (key("price"), Float(c.price)),
                (key("real_wage"), Float(c.real_wage)),
                (key("l_star"), Float(c.l_star)),
                (key("l_bar"), Float(c.l_bar)),
                (key("human"), Float(c.human)),
                (key("machine"), Float(c.machine)),
                (key("lambda_tilde"), Float(c.lambda_tilde)),
                (key("b_tilde"), Float(c.b_tilde)),
                (key("lambda_tilde_q"), Float(c.lambda_tilde_q)),
                (key("b_tilde_q"), Float(c.b_tilde_q)),
                (key("output"), Float(c.output)),
                (key("final_hours"), Float(c.final_hours)),
                (key("machine_services"), Float(c.machine_services)),
                (key("share"), Float(c.share)),
                (key("wage_floor"), Float(c.wage_floor)),
                (key("wage_ceiling"), Optional(c.wage_ceiling)),
                (key("phi_w"), Optional(c.phi_w)),
                (key("phi_r"), Optional(c.phi_r)),
                (key("chain_land"), Float(c.chain_land)),
                (key("gross_output"), Float(c.gross_output)),
                (key("human_required"), Float(c.human_required)),
                (key("reserved_cost"), Float(c.reserved_cost)),
            ]);
        }
        for (i, w) in self.workers.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Worker(i),
                name,
            };
            list.extend([
                (key("wage"), Float(w.wage)),
                (key("real_wage"), Float(w.real_wage)),
                (key("efficiency"), Float(w.efficiency)),
                (key("pooled"), Flag(w.pooled)),
                (key("premium"), Optional(w.premium)),
                (key("hours"), Float(w.hours)),
                (key("reserved_hours"), Float(w.reserved_hours)),
                (key("pool_hours"), Float(w.pool_hours)),
                (key("supply"), Float(w.supply)),
                (key("participation"), Float(w.participation)),
                (key("clearing_real_wage"), Float(w.clearing_real_wage)),
                (key("marginal_work_cost"), Float(w.marginal_work_cost)),
                (key("at_wall"), Flag(w.at_wall)),
                (key("edge"), Flag(w.edge)),
            ]);
        }
        for (s, w) in self.wall_switches.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::WallSwitch(s),
                name,
            };
            list.extend([
                (key("gamma"), Float(w.gamma)),
                (key("wage"), Float(w.wage)),
                (key("below"), Count(w.below as u32)),
                (key("above"), Count(w.above as u32)),
            ]);
        }
        list
    }
}

/// The error for the first number in [`Eq1d::outputs`] that is NaN or infinite.
fn first_non_finite(eq: &Eq1d) -> Option<SolveError> {
    eq.outputs()
        .into_iter()
        .find_map(|(key, output)| match output {
            Output1b::Float(v) | Output1b::Optional(Some(v)) if !v.is_finite() => {
                Some(match key.item {
                    Item::Economy | Item::Switch(_) | Item::WallSwitch(_) => {
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
    use crate::machine_block::Recipe;
    use crate::pow2;

    #[test]
    fn walk_visits_types_in_the_order_of_their_thresholds() {
        // Type 0's threshold e/c is 4, type 1's 1. With every type pooled P = 1 + 1 + 0.25 =
        // 2.25, above type 1's threshold and below type 0's: the walk walls type 1 alone, and P
        // is the fixed point P = P0 + Σ max(e_i r_i, c_i r_i P). A walk in index order that
        // stopped at the first pooled type would have walled none.
        let (e, c, r) = ([4.0, 1.0], [1.0, 1.0], [0.25, 0.25]);
        let (p, walled) = walk(1.0, &e, &c, &r).unwrap();
        assert_eq!(walled, [false, true]);
        assert_eq!(p, (1.0 + 4.0 * 0.25) / (1.0 - 0.25));
        let fixed = 1.0 + (e[0] * r[0]).max(c[0] * r[0] * p) + (e[1] * r[1]).max(c[1] * r[1] * p);
        assert!((p - fixed).abs() <= 2.0 * f64::EPSILON * p);
        assert!(c[0] * 2.25 <= e[0]);
    }

    #[test]
    fn walk_recomputes_after_each_wall() {
        // With every type pooled P = 1.625 + 0.75 + 0.375 = 2.75, below type 0's threshold 3;
        // walling type 1 (threshold 1.5) raises P to 2.375/0.75 = 3.17, past it, so type 0 is
        // walled too and P = 1.625/(1 − 0.5) = 3.25, the fixed point. A walk that did not
        // recompute P after walling type 1 would leave type 0 pooled.
        let (e, c, r) = ([3.0, 1.5], [1.0, 1.0], [0.25, 0.25]);
        let (p, walled) = walk(1.625, &e, &c, &r).unwrap();
        assert_eq!(walled, [true, true]);
        assert_eq!(p, 3.25);
        assert!(1.625 + 0.75 + 0.375 < e[0] / c[0]);
        assert!((1.625 + 0.75) / 0.75 > e[0] / c[0]);
        assert_eq!(p, 1.625 + (c[0] * r[0] * p) + (c[1] * r[1] * p));
    }

    #[test]
    fn walk_keeps_a_type_at_its_threshold_pooled() {
        // With every type pooled P = 1 + 2·0.5 = 2, and c·P = 2 = e exactly: at equality the type
        // counts as pooled (docs/unit-1d.md §4.3), so the walk stops. Walling it would give the
        // same P, (1 + 0)/(1 − 0.5) = 2, with the status flipped.
        let (p, walled) = walk(1.0, &[2.0], &[1.0], &[0.5]).unwrap();
        assert_eq!((p, walled), (2.0, vec![false]));
        assert_eq!(1.0 * p, 2.0);
    }

    #[test]
    fn walk_has_a_ceiling() {
        // Walled shares Σ c_i r_i that reach 1 leave no finite P: None.
        assert_eq!(walk(1.0, &[0.0], &[2.0], &[0.5]), None);
        assert_eq!(walk(1.0, &[0.0, 0.0], &[1.0, 1.0], &[0.5, 0.75]), None);
        let (p, walled) = walk(1.0, &[0.0], &[2.0], &[pow2(-1).next_down()]).unwrap();
        assert_eq!(walled, [true]);
        assert!(p > 1e15);
    }

    #[test]
    fn walk_without_candidates_is_the_base_bit_for_bit() {
        // No type with reserved hours: P = (P0 + Σ e_i·0.0)/(1.0 − 0.0) = P0, the one-type
        // nesting's step 6 (docs/unit-1d.md §5.1).
        for base in [1.3615758317798092, 7.25e-300, 3.0e300, 0.1] {
            let (p, walled) = walk(base, &[0.5434, 2.0], &[0.0, 0.0], &[0.0, 0.0]).unwrap();
            assert_eq!(p.to_bits(), base.to_bits());
            assert_eq!(walled, [false, false]);
        }
        // A type with reserved hours but ζ = 0 is no candidate either.
        let (p, walled) = walk(2.0, &[1.0], &[0.0], &[0.5]).unwrap();
        assert_eq!((p, walled), (2.5, vec![false]));
    }

    /// 1c's M4t (docs/unit-1c.md §3.3) in one-type form: a tie at the line's switch.
    fn m4t() -> WorkerEconomy {
        let recipe = |machines: [f64; 3], labor, land| Recipe {
            machines: machines.to_vec(),
            labor,
            land,
        };
        let kind = |task_efficiency, operating, build, delta, build_lag| MachineType {
            task_efficiency,
            operating,
            build,
            delta,
            build_lag,
        };
        let category = |weight, direct_land, density: [f64; 3]| Category {
            weight,
            direct_land,
            density: density.to_vec(),
        };
        WorkerEconomy::new(WorkerParams::from_machines(MachineParams {
            workers: 8.0,
            land: 10.0,
            schedule: PowerSchedule {
                eta: 0.5,
                g0: 0.2,
                g1: 0.8,
                k: 1.0,
            },
            work_cost: UniformWorkCost { chi_max: 1.0 },
            rho: 0.04,
            machine_types: vec![
                kind(
                    1.0,
                    recipe([0.0, 0.0, 0.05], 0.3, 0.05),
                    recipe([0.1, 0.0, 0.0], 1.0, 0.3),
                    0.1,
                    2,
                ),
                kind(
                    2.0,
                    recipe([0.0, 0.0, 0.5], 0.02, 0.0),
                    recipe([0.0, 0.1, 0.0], 0.1, 0.4),
                    0.05,
                    3,
                ),
                kind(
                    0.0,
                    recipe([0.0, 0.0, 0.0], 0.02, 0.5),
                    recipe([0.0, 0.1, 0.0], 0.1, 0.1),
                    0.05,
                    3,
                ),
            ],
            edges: vec![0.0, 0.4, 0.75, 1.0],
            categories: vec![
                category(0.3, 0.0, [2.0, 0.0, 0.0]),
                category(1.0, 0.6, [0.5, 1.5, 0.0]),
                category(0.2, 0.1, [0.0, 0.0, 1.0]),
                category(0.8, 1.0, [0.0, 0.4, 0.0]),
            ],
            intermediate: vec![
                vec![0.0, 0.0, 0.0, 0.0],
                vec![0.1, 0.0, 0.0, 0.0],
                vec![0.0, 0.2, 0.0, 0.0],
                vec![0.15, 0.0, 0.0, 0.0],
            ],
        }))
        .unwrap()
    }

    #[test]
    fn a_tie_without_walled_types_takes_the_closed_form() {
        // With no walled type the prices do not move with σ and f(σ)·B(σ) is linear in σ, so
        // unit 1c's closed form is the root: bisection of f(σ) lands on it.
        let e = m4t();
        let x = e.switch_points().unwrap()[0];
        let s = e.machines().envelope().switches[0];
        let q = e.at_with(x, s.below);
        let (closed, edge) = e.tie_share(&q, s.below, s.above).unwrap();
        assert!(closed > 0.0 && closed < 1.0);
        assert_eq!(edge, None);
        let f = |share: f64| e.mixed_excess(&q, s.below, s.above, share, None);
        let bisected = bisect_bits(f, (0.0, f(0.0)), (1.0, f(1.0))).unwrap().x;
        assert!(
            (closed - bisected).abs() <= 1e-12 * closed,
            "{closed} {bisected}"
        );
        assert!(f(closed).abs() <= 1e-12 * q.n_d);
    }

    /// An `Eq1d` with a tie, one line switch, one wall switch, two machine types, two
    /// categories and two worker types, whose numbers are 1, 2, 3, ... in output order, with the
    /// `nan_at`-th replaced by NaN (none when `nan_at` is 0). A struct literal must name every
    /// field, so a field added to `Eq1d`, `CategoryEq1d` or `WorkerEq` does not compile here
    /// until it is given a number.
    fn numbered(nan_at: usize) -> Eq1d {
        let mut next = 0;
        let mut n = || {
            next += 1;
            if next == nan_at {
                f64::NAN
            } else {
                next as f64
            }
        };
        let mut eq = Eq1d {
            x_star: n(),
            one_minus_x_star: n(),
            gamma_star: n(),
            m_s: n(),
            u: n(),
            v: n(),
            p_s: n(),
            y: n(),
            final_hours: n(),
            machine_hours: n(),
            n_a: n(),
            participation: n(),
            income: n(),
            interest: n(),
            labor_share: n(),
            capital_share: n(),
            real_wage: n(),
            support_cost: n(),
            worker_baskets: n(),
            provider_baskets: n(),
            funded: true,
            lemma_b1: false,
            phi_w: Some(n()),
            phi_r: Some(n()),
            h_s: n(),
            b_d: n(),
            l_star_s: n(),
            l_s: n(),
            b_s: n(),
            l_s_q: n(),
            b_s_q: n(),
            rent_ceiling: n(),
            margin_active: true,
            residuals: Residuals1d {
                labor: n(),
                income: n(),
                land: n(),
                services: n(),
                user_cost: n(),
                fork: n(),
                totals: n(),
                expenditure: n(),
                leontief_price: 0.0,
                leontief_quantity: 0.0,
                closure: 0.0,
                cheapest: 0.0,
                corner: 0.0,
                reserved: 0.0,
                basket: 0.0,
            },
            bisection_steps: 7,
            d: 0.0,
            technique: 0,
            tie: None,
            switches: Vec::new(),
            types: Vec::new(),
            categories: Vec::new(),
            margin: Margin::Wall,
            g: 0.0,
            replacement_top: 0.0,
            replacement_bottom: 0.0,
            n_pool: 0.0,
            required_hours: 0.0,
            reserved_hours: 0.0,
            wage_bill: 0.0,
            f_line_0: None,
            f_line_lo: None,
            f_line_1: None,
            f_end: 0.0,
            wall_switches: Vec::new(),
            workers: Vec::new(),
        };
        eq.d = n();
        eq.tie = Some(Tie {
            above: 1,
            share: n(),
            gamma: n(),
        });
        eq.residuals.leontief_price = n();
        eq.residuals.leontief_quantity = n();
        eq.residuals.closure = n();
        eq.residuals.cheapest = n();
        eq.g = n();
        eq.replacement_top = n();
        eq.replacement_bottom = n();
        eq.n_pool = n();
        eq.required_hours = n();
        eq.reserved_hours = n();
        eq.wage_bill = n();
        eq.f_line_0 = Some(n());
        eq.f_line_lo = Some(n());
        eq.f_line_1 = Some(n());
        eq.f_end = n();
        eq.residuals.corner = n();
        eq.residuals.reserved = n();
        eq.residuals.basket = n();
        eq.switches.push(SwitchPoint {
            gamma: n(),
            x: n(),
            below: 0,
            above: 1,
        });
        for _ in 0..2 {
            eq.types.push(TypeEq {
                price: n(),
                operating_cost: n(),
                build_cost: n(),
                user_cost: n(),
                wealth_factor: n(),
                delivered_cost: Some(n()),
                closure_wage: Some(n()),
                lambda_tilde: n(),
                b_tilde: n(),
                lambda_tilde_q: n(),
                b_tilde_q: n(),
                services: n(),
                task_services: n(),
                builds: n(),
                hours: n(),
                land: n(),
                wealth: n(),
                interest: n(),
                phi_w: Some(n()),
            });
        }
        for _ in 0..2 {
            eq.categories.push(CategoryEq1d {
                price: n(),
                real_wage: n(),
                l_star: n(),
                l_bar: n(),
                human: n(),
                machine: n(),
                lambda_tilde: n(),
                b_tilde: n(),
                lambda_tilde_q: n(),
                b_tilde_q: n(),
                output: n(),
                final_hours: n(),
                machine_services: n(),
                share: n(),
                wage_floor: n(),
                wage_ceiling: Some(n()),
                phi_w: Some(n()),
                phi_r: Some(n()),
                chain_land: n(),
                gross_output: n(),
                human_required: n(),
                reserved_cost: n(),
            });
        }
        for _ in 0..2 {
            eq.workers.push(WorkerEq {
                wage: n(),
                real_wage: n(),
                efficiency: n(),
                pooled: true,
                premium: Some(n()),
                hours: n(),
                reserved_hours: n(),
                pool_hours: n(),
                supply: n(),
                participation: n(),
                clearing_real_wage: n(),
                marginal_work_cost: n(),
                at_wall: false,
                edge: false,
            });
        }
        eq.wall_switches.push(WallSwitch {
            gamma: n(),
            wage: n(),
            below: 0,
            above: 1,
        });
        eq
    }

    /// The numbers of `outputs()`, without the tie's type index.
    fn numbers(eq: &Eq1d) -> Vec<(OutputKey1c, f64)> {
        eq.outputs()
            .into_iter()
            .filter(|(key, _)| key.name != "tie_above")
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
        assert_eq!(listed.len(), 45 + 14 + 2 + 2 * 19 + 2 * 22 + 2 * 11 + 2);
        let mut keys: Vec<String> = eq.outputs().iter().map(|(k, _)| k.to_string()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), eq.outputs().len());
        for key in [
            "worker1.premium",
            "wall_switch0.wage",
            "cat1.reserved_cost",
            "f_line_lo",
            "res_basket",
            "margin",
        ] {
            assert!(keys.contains(&key.to_string()), "{key}");
        }
        let flags: Vec<(String, Output1b)> = eq
            .outputs()
            .into_iter()
            .filter(|(_, o)| matches!(o, Output1b::Flag(_) | Output1b::Count(_)))
            .map(|(k, o)| (k.to_string(), o))
            .collect();
        let want: Vec<(String, Output1b)> = [
            ("funded", Output1b::Flag(true)),
            ("lemma_b1", Output1b::Flag(false)),
            ("margin_active", Output1b::Flag(true)),
            ("bisection_steps", Output1b::Count(7)),
            ("technique", Output1b::Count(0)),
            ("tie", Output1b::Flag(true)),
            ("margin", Output1b::Count(1)),
            ("switch0.below", Output1b::Count(0)),
            ("switch0.above", Output1b::Count(1)),
            ("worker0.pooled", Output1b::Flag(true)),
            ("worker0.at_wall", Output1b::Flag(false)),
            ("worker0.edge", Output1b::Flag(false)),
            ("worker1.pooled", Output1b::Flag(true)),
            ("worker1.at_wall", Output1b::Flag(false)),
            ("worker1.edge", Output1b::Flag(false)),
            ("wall_switch0.below", Output1b::Count(0)),
            ("wall_switch0.above", Output1b::Count(1)),
        ]
        .into_iter()
        .map(|(k, o)| (k.to_string(), o))
        .collect();
        assert_eq!(flags, want);
        assert_eq!(
            [Margin::Contestable, Margin::Wall, Margin::AllHuman].map(Margin::code),
            [0, 1, 2]
        );
    }

    #[test]
    fn every_output_is_checked_for_finiteness() {
        assert_eq!(first_non_finite(&numbered(0)), None);
        let keys: Vec<OutputKey1c> = numbers(&numbered(0)).iter().map(|(k, _)| *k).collect();
        for (i, key) in keys.iter().enumerate() {
            let want = match key.item {
                Item::Economy | Item::Switch(_) | Item::WallSwitch(_) => {
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
            };
            assert_eq!(
                first_non_finite(&numbered(i + 1)),
                Some(want),
                "number {}",
                i + 1
            );
        }
        // A short point on the line is absent, not a number, and is skipped.
        let mut eq = numbered(0);
        eq.f_line_lo = None;
        eq.workers[1].premium = Some(f64::INFINITY);
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFiniteInWorker {
                worker: 1,
                what: "premium"
            })
        );
        assert_eq!(
            SolveError::NonFiniteInWorker {
                worker: 1,
                what: "premium"
            }
            .to_string(),
            "premium of worker type 1 is not finite"
        );
        assert_eq!(finite_or_none(f64::INFINITY), None);
        assert_eq!(finite_or_none(-2.5), Some(-2.5));
    }
}
