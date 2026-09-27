//! Unit 1c: many machine types, the Leontief inverse and the user cost per type
//! (docs/unit-1c.md).
//!
//! The categories of unit 1b, which may now use each other as intermediate inputs, are made
//! on one task line by people and by machine types. Each type has an operating recipe and a
//! build recipe over machine services, labour and land, its own depreciation and build lag,
//! and so its own user cost; one interest rate is common to all (check_dynamics R1-R6,
//! SSRN A.1 and A.4). The task-doing types share the line's capability shape, each with its
//! own task efficiency θ_k, so the cheapest type takes every machine task and one threshold
//! splits the line, as in units 1a and 1b (docs/unit-1c.md §2.4). Where the cheapest type
//! changes along the line, labour demand jumps; an equilibrium on the jump is a tie, solved
//! in closed form, and more than one equilibrium is refused (§2.5).
//!
//! The evaluation order is normative (§5.1): one type with no operating recipe, θ = 1 and no
//! intermediate inputs repeats unit 1b's (and so unit 1a's) floating-point operations bit
//! for bit.

use std::fmt;

use rustyecon_core::num;

use crate::categories::{
    all_human_hours, segment_of, tasks, validate_category, validate_line, Category, CategoryParams,
    Output1b,
};
use crate::leontief::{identity_minus, transpose, Factors};
use crate::machine_block::{Envelope, MachineBlock, MachineType, Recipe};
use crate::params::{self, ParamError, UniformWorkCost};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{
    bisect, finite, labor_net, regime_tests, Regime, Root, SolveError, BRACKET_HI, BRACKET_LO,
    MAX_BISECTION_STEPS,
};

/// The parameters of unit 1c (docs/unit-1c.md §3.1): unit 1b's
/// [`CategoryParams`] without the machine row (a, λ, b, δ, J_b), plus the machine types and
/// the categories' intermediate inputs. Build a [`MachineEconomy`] from them to validate.
#[derive(Clone, Debug, PartialEq)]
pub struct MachineParams<S = PowerSchedule> {
    /// N, potential workers. Scale.
    pub workers: f64,
    /// T, the land-service endowment per period. Scale.
    pub land: f64,
    /// γ(x), relative human productivity on the one task line.
    pub schedule: S,
    /// F, the distribution of the work cost χ.
    pub work_cost: UniformWorkCost,
    /// ρ, the interest rate, common to every machine type. In
    /// [0, [`SCALE_CEIL`](crate::SCALE_CEIL)].
    pub rho: f64,
    /// The machine types, in the order every sum over types runs.
    pub machine_types: Vec<MachineType>,
    /// e_0 … e_S, the segments' edges on the task line (unit 1b).
    pub edges: Vec<f64>,
    /// The categories, in the order every sum over categories runs (unit 1b).
    pub categories: Vec<Category>,
    /// a_jl: units of category l used per unit of category j, one row per category (row j),
    /// each entry in [0, [`SCALE_CEIL`](crate::SCALE_CEIL)].
    pub intermediate: Vec<Vec<f64>>,
}

impl<S> MachineParams<S> {
    /// Unit 1b's economy in machine-type form (docs/unit-1c.md §3.3, M1): one type with θ = 1,
    /// no operating recipe, the build recipe (a, λ, b), 1b's δ and J_b, and no intermediate
    /// inputs. It solves to 1b's equilibrium bit for bit.
    pub fn from_categories(params: CategoryParams<S>) -> Self {
        let count = params.categories.len();
        MachineParams {
            workers: params.workers,
            land: params.land,
            schedule: params.schedule,
            work_cost: params.work_cost,
            rho: params.rho,
            machine_types: vec![MachineType {
                task_efficiency: 1.0,
                operating: Recipe::zero(1),
                build: Recipe {
                    machines: vec![params.a],
                    labor: params.lam,
                    land: params.b,
                },
                delta: params.delta,
                build_lag: params.build_lag,
            }],
            edges: params.edges,
            categories: params.categories,
            intermediate: vec![vec![0.0; count]; count],
        }
    }
}

/// A validated unit-1c economy, with every quantity that does not depend on x
/// (docs/unit-1c.md §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct MachineEconomy<S = PowerSchedule> {
    params: MachineParams<S>,
    block: MachineBlock,
    integral_at_edges: Vec<f64>,
    /// The factors of I − A_cc.
    cc: Factors,
    /// The factors of I − A_ccᵀ, which give a basket's gross outputs (unit 1f's CES content at
    /// each point, docs/unit-1f.md §4.2).
    cc_t: Factors,
    /// ŷ = (I − A_ccᵀ)⁻¹z, the categories' gross outputs per basket.
    basket_outputs: Vec<f64>,
    /// L̄^dir_j, each category's own all-human hours.
    direct_hours: Vec<f64>,
    /// L̄ = (I − A_cc)⁻¹L̄^dir, the all-human method through the chain.
    all_human: Vec<f64>,
    /// b̄ = (I − A_cc)⁻¹b_c, land through the chain.
    chain_land: Vec<f64>,
    /// B_ŷ = Σ ŷ_j·b_j.
    basket_land: f64,
    /// Σ ŷ_j·L̄^dir_j.
    basket_all_human: f64,
    /// The technique along the bracket.
    envelope: Envelope,
}

/// The error for a parameter of category `index`.
pub(crate) fn in_category(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "category",
        index,
        error: Box::new(error),
    }
}

/// The error for category `index` when it has neither work nor land, directly or through the
/// chain, and so no positive price (docs/unit-1c.md §3.2; unit-1d.md §3.2 with its own hours).
pub(crate) fn unpriced_category(index: usize) -> ParamError {
    in_category(index)(ParamError::Invalid {
        name: "category",
        reason: "a category needs tasks or land, directly or through its \
                 intermediate inputs, so that its price is positive",
    })
}

/// The error when the basket needs no work by hand (docs/unit-1c.md §3.2; unit-1d.md §3.2
/// counts the common human-required hours too).
pub(crate) fn no_basket_work() -> ParamError {
    ParamError::Invalid {
        name: "basket",
        reason: "the basket must need work: the sum over categories of gross output per \
                 basket times all-human hours must be positive",
    }
}

impl<S: Schedule> MachineEconomy<S> {
    /// Validates the parameters (docs/unit-1c.md §3.2) and computes every x-free quantity
    /// (§5.2).
    ///
    /// In order: N, T, the schedule, χ_max and ρ as in unit 1b; the machine types
    /// ([`MachineBlock::new`]: each type's parameters, at least one task type, productive
    /// recipes, every chain reaching land); the task line and each category's own checks as
    /// in unit 1b; `intermediate`, C rows of C entries in [0, SCALE_CEIL] with I − A_cc a
    /// nonsingular M-matrix (SSRN p.8's spectral radius below one); each category priced
    /// through the chain (L̄_j > 0 or b̄_j > 0); the basket's chain land B_ŷ = Σ ŷ_j·b_j > 0
    /// and chain hours Σ ŷ_j·L̄^dir_j > 0. −0.0 is stored as +0.0.
    ///
    /// Price-side viability is not checked: an economy with no viable technique at x = 1
    /// solves to [`Regime::NotViable`], and so does one with a type whose price recursion
    /// diverges, even a type no technique would use (docs/unit-1c.md §12 item 15).
    pub fn new(params: MachineParams<S>) -> Result<Self, ParamError> {
        let economy = Self::assemble(params, true)?;
        for index in 0..economy.params.categories.len() {
            if !(economy.all_human[index] > 0.0 || economy.chain_land[index] > 0.0) {
                return Err(unpriced_category(index));
            }
        }
        economy.check_basket_land()?;
        if economy.basket_all_human <= 0.0 {
            return Err(no_basket_work());
        }
        Ok(economy)
    }

    /// [`MachineEconomy::new`]'s checks and x-free quantities without its last three checks
    /// (each category priced, the basket's land and its work), which unit 1d states with its
    /// own hours (docs/unit-1d.md §3.2). With `check_workers` false, N and χ_max are not
    /// checked either: unit 1d's worker types carry them, and the placeholders are unused.
    pub(crate) fn assemble(
        mut params: MachineParams<S>,
        check_workers: bool,
    ) -> Result<Self, ParamError> {
        if check_workers {
            params::scale("workers", params.workers)?;
        }
        params::scale("land", params.land)?;
        params.schedule.validate()?;
        if check_workers {
            params::scale("chi_max", params.work_cost.chi_max)?;
        }
        params.rho = params::nonnegative("rho", params.rho)?;
        let block = MachineBlock::new(params.machine_types.clone(), params.rho)?;
        params.machine_types = block.types().to_vec();
        let integral_at_edges = validate_line(&mut params.edges, &params.schedule)?;
        if params.categories.is_empty() {
            return Err(ParamError::Invalid {
                name: "categories",
                reason: "the economy needs at least one category",
            });
        }
        let segments = params.edges.len() - 1;
        for (index, category) in params.categories.iter_mut().enumerate() {
            validate_category(category, segments).map_err(in_category(index))?;
        }
        // Intermediate inputs (§3.2).
        let count = params.categories.len();
        if params.intermediate.len() != count
            || params.intermediate.iter().any(|r| r.len() != count)
        {
            return Err(ParamError::Invalid {
                name: "intermediate",
                reason: "the intermediate inputs need one row per category, each with one \
                         entry per category",
            });
        }
        for row in &mut params.intermediate {
            for a in row.iter_mut() {
                *a = params::nonnegative("intermediate", *a)?;
            }
        }
        let flat: Vec<f64> = params.intermediate.iter().flatten().copied().collect();
        let cc = Factors::new(count, identity_minus(count, &flat));
        let cc_t = Factors::new(count, identity_minus(count, &transpose(count, &flat)));
        if !cc.is_m_matrix() || !cc_t.is_m_matrix() {
            return Err(ParamError::Invalid {
                name: "intermediate",
                reason: "intermediate inputs are not productive: the spectral radius of the \
                         categories' input matrix must be below 1",
            });
        }
        let weights: Vec<f64> = params.categories.iter().map(|c| c.weight).collect();
        let basket_outputs = cc_t.solve(&weights);
        let direct_hours: Vec<f64> = params
            .categories
            .iter()
            .map(|c| all_human_hours(&params.edges, &c.density))
            .collect();
        let direct_land: Vec<f64> = params.categories.iter().map(|c| c.direct_land).collect();
        let all_human = cc.solve(&direct_hours);
        let chain_land = cc.solve(&direct_land);
        let (mut basket_land, mut basket_all_human) = (0.0, 0.0);
        for index in 0..count {
            basket_land += basket_outputs[index] * params.categories[index].direct_land;
            basket_all_human += basket_outputs[index] * direct_hours[index];
        }
        let envelope = block.envelope(
            params.schedule.gamma(BRACKET_LO),
            params.schedule.gamma(BRACKET_HI),
        );
        Ok(MachineEconomy {
            params,
            block,
            integral_at_edges,
            cc,
            cc_t,
            basket_outputs,
            direct_hours,
            all_human,
            chain_land,
            basket_land,
            basket_all_human,
            envelope,
        })
    }

    /// The validated parameters.
    pub fn params(&self) -> &MachineParams<S> {
        &self.params
    }

    /// The machine block: the types, ρ, their user costs and totals, the closure at any
    /// margin (docs/unit-1c.md §4.9).
    pub fn block(&self) -> &MachineBlock {
        &self.block
    }

    /// The technique envelope on [γ(`BRACKET_LO`), γ(1)] (docs/unit-1c.md §4.3).
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    /// ŷ = (I − A_ccᵀ)⁻¹z, the categories' gross outputs per basket (SSRN eq 7).
    pub fn basket_outputs(&self) -> &[f64] {
        &self.basket_outputs
    }

    /// L̄ = (I − A_cc)⁻¹L̄^dir, each category's all-human hours through the chain.
    pub fn all_human_hours(&self) -> &[f64] {
        &self.all_human
    }

    /// b̄ = (I − A_cc)⁻¹b_c, each category's land through the chain.
    pub fn chain_land(&self) -> &[f64] {
        &self.chain_land
    }

    /// B_ŷ = Σ ŷ_j·b_j, the basket's chain land.
    pub fn basket_land(&self) -> f64 {
        self.basket_land
    }

    /// Σ ŷ_j·L̄^dir_j, the basket's chain hours by hand.
    pub fn basket_all_human_hours(&self) -> f64 {
        self.basket_all_human
    }

    /// The basket's land check of [`MachineEconomy::new`]: B_ŷ = Σ ŷ_j·b_j > 0.
    pub(crate) fn check_basket_land(&self) -> Result<(), ParamError> {
        if self.basket_land <= 0.0 {
            return Err(ParamError::Invalid {
                name: "basket",
                reason: "the basket must use land directly: the sum over categories of gross \
                         output per basket times direct_land must be positive",
            });
        }
        Ok(())
    }

    /// The factors of I − A_cc.
    pub(crate) fn chain(&self) -> &Factors {
        &self.cc
    }

    /// (I − A_ccᵀ)⁻¹c, the gross outputs of a basket whose content is c (unit 1f's CES basket,
    /// docs/unit-1f.md §4.2); with c = z it is [`basket_outputs`](Self::basket_outputs) bit for
    /// bit.
    pub(crate) fn outputs_of(&self, content: &[f64]) -> Vec<f64> {
        self.cc_t.solve(content)
    }

    /// J at every edge of the task line.
    pub(crate) fn integral_at_edges(&self) -> &[f64] {
        &self.integral_at_edges
    }

    /// L̄^dir_j, each category's own all-human hours.
    pub(crate) fn direct_hours(&self) -> &[f64] {
        &self.direct_hours
    }

    /// The technique the envelope gives at x: τ_i for γ_i ≤ γ(x) < γ_{i+1}.
    pub fn technique_at(&self, x: f64) -> usize {
        self.envelope.technique_at(self.params.schedule.gamma(x))
    }

    /// Prices and quantities at x under the envelope's technique there
    /// (docs/unit-1c.md §5.1).
    pub fn at(&self, x: f64) -> MachinePoint {
        self.at_with(x, self.technique_at(x))
    }

    /// Prices and quantities at a candidate threshold x in [0, 1] with task type `technique`
    /// doing the machine tasks, in the evaluation order of docs/unit-1c.md §5.1, which for one
    /// type is unit 1b's [`CategoryEconomy::at`](crate::CategoryEconomy::at) operation for
    /// operation.
    ///
    /// Outside [0, 1], or where the technique is not viable (d ≤ 0), the formulas are
    /// evaluated as written and mean nothing. Panics if `technique` is out of range.
    pub fn at_with(&self, x: f64, technique: usize) -> MachinePoint {
        let p = &self.params;
        let gamma = p.schedule.gamma(x);
        let j = p.schedule.integral(x);
        // Step 2: the machine block.
        let block = self.block.prices_at(gamma, technique);
        let (v, task_price) = (block.v, block.task_price);
        // Step 3: the categories.
        let count = p.categories.len();
        let (mut human, mut machine, mut rhs) = (
            Vec::with_capacity(count),
            Vec::with_capacity(count),
            Vec::with_capacity(count),
        );
        for category in &p.categories {
            let (h, m) = tasks(
                &p.edges,
                &self.integral_at_edges,
                &category.density,
                x,
                j,
                None,
            );
            rhs.push(v * h + task_price * m + category.direct_land);
            human.push(h);
            machine.push(m);
        }
        let prices = self.cc.solve(&rhs);
        let (mut p_s, mut h_s, mut m_s) = (0.0, 0.0, 0.0);
        for index in 0..count {
            p_s += p.categories[index].weight * prices[index];
            h_s += self.basket_outputs[index] * human[index];
            m_s += self.basket_outputs[index] * machine[index];
        }
        // Step 4: the quantities.
        let theta = self.block.types()[technique].task_efficiency;
        let mut task = vec![0.0; self.block.len()];
        task[technique] = m_s / theta;
        let cleared = self.clear(&task);
        let final_hours = cleared.y * h_s;
        let n_d = final_hours + cleared.machine_hours;
        let n_s = p.workers * p.work_cost.cdf(num::ln1p(v / p_s));
        MachinePoint {
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
            prices,
            human,
            machine,
            p_s,
            h_s,
            m_s,
            b_d: self.basket_land,
            y: cleared.y,
            services: cleared.services,
            final_hours,
            machine_hours: cleared.machine_hours,
            n_d,
            n_s,
        }
    }

    /// §5.1 step 4 from the task services per basket t: forward substitution, back
    /// substitution with the division deferred, land clearing, services and machine hours.
    pub(crate) fn clear(&self, task: &[f64]) -> Cleared {
        self.clear_with(task, self.params.land)
    }

    /// [`clear`](Self::clear) with the market's land in use T_m in place of T (docs/unit-1e.md
    /// §4.5): Y = T_m/B^q. With T_m = T it is `clear` bit for bit.
    pub(crate) fn clear_with(&self, task: &[f64], land: f64) -> Cleared {
        self.clear_basket(task, land, self.basket_land)
    }

    /// [`clear_with`](Self::clear_with) for a basket whose chain land is B_ŷ = `basket_land` (unit
    /// 1f's CES content at a point, docs/unit-1f.md §4.2): Y = T_m/(B_ŷ + Σ_k b^q_k·x̂_k).
    pub(crate) fn clear_basket(&self, task: &[f64], land: f64, basket_land: f64) -> Cleared {
        let (numerators, pivots) = self.block.clearing_numerators(task);
        let land_q = self.block.land_q();
        let lambda_q = self.block.lambda_q();
        let mut land_per_basket = 0.0;
        for k in 0..task.len() {
            land_per_basket += (land_q[k] * numerators[k]) / pivots[k];
        }
        let y = land / (basket_land + land_per_basket);
        let services: Vec<f64> = (0..task.len())
            .map(|k| (y * numerators[k]) / pivots[k])
            .collect();
        let mut machine_hours = 0.0;
        for k in 0..task.len() {
            machine_hours += lambda_q[k] * services[k];
        }
        Cleared {
            y,
            services,
            machine_hours,
        }
    }

    /// The switch points x_i on the line (docs/unit-1c.md §5.3 step 2): for each switch γ_i
    /// of the envelope, the largest double x with γ(x) < γ_i, by bisection on [x_{i−1}, 1]
    /// for γ(x) ≥ γ_i until the ends are adjacent doubles (x_0 = [`BRACKET_LO`]).
    pub fn switch_points(&self) -> Result<Vec<f64>, SolveError> {
        let schedule = &self.params.schedule;
        let mut points = Vec::with_capacity(self.envelope.switches.len());
        let mut from = BRACKET_LO;
        for switch in &self.envelope.switches {
            let (mut lo, mut hi) = (from, BRACKET_HI);
            let mut steps = 0;
            loop {
                let mid = 0.5 * (lo + hi);
                if mid <= lo || mid >= hi {
                    break;
                }
                if steps == MAX_BISECTION_STEPS {
                    return Err(SolveError::NoConvergence { steps });
                }
                steps += 1;
                if schedule.gamma(mid) >= switch.gamma {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            points.push(lo);
            from = lo;
        }
        Ok(points)
    }

    /// Classifies the economy and, when interior, solves it (docs/unit-1c.md §5.3).
    ///
    /// The regime tests are unit 1a's, with the envelope's last technique at x = 1 (its least
    /// pivot is `d_at_1`) and its first at [`BRACKET_LO`]. Without a switch they decide the
    /// regime, and this is unit 1b's solve, step for step. With switches, `NotViable` stands,
    /// and the sign of the excess demand is read at both ends of each technique's region:
    /// f(1) ≥ 0 is the boundary's corner and f(lo) ≤ 0 the corner at lo, each one
    /// equilibrium; a change of side inside the bracket is a root inside a region, found by
    /// 1a's bisection with 1 − x* carried, or a tie at a switch, where labour clears by the
    /// share of machine tasks each type takes (§4.7). One equilibrium in all is the solve's
    /// answer; more is [`SolveError::MultipleEquilibria`], so that a boundary regime is
    /// returned only when no equilibrium hides inside the bracket (an upward jump of labour
    /// demand at a switch can put one there, with interest).
    ///
    /// Returns [`SolveError::NonFinite`], [`SolveError::NonFiniteInType`] or
    /// [`SolveError::NonFiniteInCategory`] if a value the solve decides on, or any reported
    /// output, is not finite, and [`SolveError::LaborNotCleared`] if the reported equilibrium
    /// misses labour clearing by more than [`LABOR_RESIDUAL_NET`](crate::LABOR_RESIDUAL_NET)
    /// of N_a.
    pub fn solve(&self) -> Result<Regime<Eq1c>, SolveError> {
        let envelope = &self.envelope;
        let at_hi = self.at_with(BRACKET_HI, envelope.last());
        let f_at_lo = || self.at_with(BRACKET_LO, envelope.first).excess_demand();
        // Step 1: 1a's regime tests. With a switch a boundary regime is only a candidate:
        // step 3's count decides, with the other end evaluated too.
        let (f_lo, f_hi) = match regime_tests(at_hi.d, at_hi.excess_demand(), f_at_lo)? {
            Ok(ends) => ends,
            Err(regime) if envelope.switches.is_empty() => return Ok(regime),
            Err(Regime::BoundaryNoMargin { f_at_1 }) => {
                (finite("n_D - n_S at BRACKET_LO", f_at_lo())?, f_at_1)
            }
            Err(Regime::NoInteriorAtZero { f_at_0 }) => (f_at_0, at_hi.excess_demand()),
            Err(regime) => return Ok(regime),
        };
        let points = self.switch_points()?;
        let techniques: Vec<usize> = std::iter::once(envelope.first)
            .chain(envelope.switches.iter().map(|s| s.above))
            .collect();
        let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
            .chain(points.iter().copied())
            .chain(std::iter::once(BRACKET_HI))
            .collect();
        let last = techniques.len() - 1;
        let at_switch = |x: f64, technique: usize| -> Result<f64, SolveError> {
            let f = self.at_with(x, technique).excess_demand();
            if f.is_finite() {
                Ok(f)
            } else {
                Err(SolveError::NonFinite {
                    what: "n_D - n_S at a switch",
                })
            }
        };
        // f_i(x_i), f_i(x_{i+1}) for each region i.
        let mut sequence = Vec::with_capacity(2 * techniques.len());
        for (i, &technique) in techniques.iter().enumerate() {
            sequence.push(if i == 0 {
                f_lo
            } else {
                at_switch(bounds[i], technique)?
            });
            sequence.push(if i == last {
                f_hi
            } else {
                at_switch(bounds[i + 1], technique)?
            });
        }
        // The sides: a value is on the positive side when > 0, except f(1), which is when ≥ 0
        // (1a's BoundaryNoMargin at f(1) = 0). A positive side before f(lo) and a nonpositive
        // one after f(1) stand for the corners: a change of side there is NoInteriorAtZero or
        // BoundaryNoMargin, so every equilibrium is one change and their number is odd.
        let top = sequence.len() - 1;
        let mut sides = Vec::with_capacity(sequence.len() + 2);
        sides.push(true);
        for (i, &f) in sequence.iter().enumerate() {
            sides.push(if i == top { f >= 0.0 } else { f > 0.0 });
        }
        sides.push(false);
        let changes: Vec<usize> = (0..sides.len() - 1)
            .filter(|&i| sides[i] != sides[i + 1])
            .collect();
        if changes.len() != 1 {
            return Err(SolveError::MultipleEquilibria {
                sign_changes: changes.len(),
                switches: points,
            });
        }
        // The change between sides i and i + 1 is between sequence[i − 1] and sequence[i].
        let at = match changes[0] {
            0 => return Ok(Regime::NoInteriorAtZero { f_at_0: f_lo }),
            i if i == sequence.len() => return Ok(Regime::BoundaryNoMargin { f_at_1: f_hi }),
            i => i - 1,
        };
        let region = at / 2;
        let eq = if at.is_multiple_of(2) {
            // Inside region i: 1a's bisection, or an exact zero at its upper end.
            let technique = techniques[region];
            let (lo, hi) = (bounds[region], bounds[region + 1]);
            let (f_at_lo, f_at_hi) = (sequence[at], sequence[at + 1]);
            let root = if f_at_hi == 0.0 {
                Root::exact(hi, 0)
            } else {
                bisect(
                    |x| self.at_with(x, technique).excess_demand(),
                    (lo, f_at_lo),
                    (hi, f_at_hi),
                )?
            };
            self.report(
                (root.x, root.one_minus_x, root.steps),
                technique,
                None,
                &points,
                &at_hi,
            )
        } else {
            // Between f_i(x_{i+1}) and f_{i+1}(x_{i+1}): a tie at the switch.
            let (below, above) = (techniques[region], techniques[region + 1]);
            let x = bounds[region + 1];
            let share = self.tie_share(x, below, above);
            let gamma = envelope.switches[region].gamma;
            self.report(
                (x, 1.0 - x, 0),
                below,
                Some(Tie {
                    above,
                    share,
                    gamma,
                }),
                &points,
                &at_hi,
            )
        };
        if let Some(error) = first_non_finite(&eq) {
            return Err(error);
        }
        labor_net(eq.x_star, eq.residuals.labor, eq.n_a)?;
        Ok(Regime::Interior(Box::new(eq)))
    }

    /// σ, the share of the machine tasks the type above takes at a tie at x
    /// (docs/unit-1c.md §4.7): σ = B_a·f_a/(B_a·f_a − B_b·f_b), with both techniques'
    /// quantities at x and the prices, and so n_S, of the type below. 1 when f_b ≥ 0, which
    /// happens only within rounding of a root of the upper technique at x.
    fn tie_share(&self, x: f64, below: usize, above: usize) -> f64 {
        let q = self.at_with(x, below);
        let theta_b = self.block.types()[above].task_efficiency;
        let mut task = vec![0.0; self.block.len()];
        task[above] = q.m_s / theta_b;
        let b = self.clear(&task);
        let land_a = self.params.land / q.y;
        let land_b = self.params.land / b.y;
        let f_a = q.n_d - q.n_s;
        let f_b = (b.y * q.h_s + b.machine_hours) - q.n_s;
        if f_b >= 0.0 {
            return 1.0;
        }
        (land_a * f_a) / (land_a * f_a - land_b * f_b)
    }

    /// docs/unit-1c.md §4.4-4.8 at x*, with technique τ, or at a tie with τ the type below.
    fn report(
        &self,
        (x, one_minus_x, steps): (f64, f64, u32),
        technique: usize,
        tie: Option<Tie>,
        points: &[f64],
        at_one: &MachinePoint,
    ) -> Eq1c {
        let p = &self.params;
        let block = &self.block;
        let types = block.types();
        let k_count = block.len();
        let q = self.at_with(x, technique);
        let theta = |t: usize| types[t].task_efficiency;
        // The task services per basket, by type: all to τ, or split at a tie.
        let (y, services, machine_hours) = match tie {
            None => (q.y, q.services.clone(), q.machine_hours),
            Some(t) => {
                let mut task = vec![0.0; k_count];
                task[technique] = (1.0 - t.share) * (q.m_s / theta(technique));
                task[t.above] = t.share * (q.m_s / theta(t.above));
                let c = self.clear(&task);
                (c.y, c.services, c.machine_hours)
            }
        };
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
        let mut human = Vec::with_capacity(count);
        let mut h_s = 0.0;
        for (index, category) in p.categories.iter().enumerate() {
            let (h, _) = tasks(
                &p.edges,
                &self.integral_at_edges,
                &category.density,
                q.x,
                q.j,
                Some(one_minus_x),
            );
            h_s += self.basket_outputs[index] * h;
            human.push(h);
        }
        // 1b's report, with the machine row per type, in 1b's order.
        let final_hours = y * h_s;
        let n_a = final_hours + machine_hours;
        let u = block.user_costs();
        let omega = block.wealth_factors();
        let mut interest = 0.0;
        let mut type_interest = Vec::with_capacity(k_count);
        for k in 0..k_count {
            let i = p.rho * omega[k] * q.build[k] * services[k];
            interest += i;
            type_interest.push(i);
        }
        let wages = q.v * n_a;
        let income = wages + p.land + interest;
        let support_cost = p.workers * q.p_s;
        let provider_baskets = (p.land + interest) / q.p_s - p.workers;
        let flow_prices = u.iter().all(|&uk| uk == 1.0);
        let phi_w = if flow_prices {
            block.labor_share_of_price(q.gamma, technique)
        } else {
            None
        };
        // An interior technique is viable, so I − Â is a nonsingular M-matrix in exact
        // arithmetic; should rounding say otherwise, the totals are NaN and the solve
        // reports them as not finite.
        let missing = vec![f64::NAN; k_count];
        let (lt_m, bt_m) = block
            .price_totals()
            .unwrap_or((missing.as_slice(), missing.as_slice()));
        let (ltq_m, btq_m) = (block.lambda_tilde_q(), block.b_tilde_q());
        // The machine part of each category's totals, per unit of task services: through τ, or
        // at a tie through the split, on both sides. At a tie the two delivered costs are
        // equal, so either type's decomposition of the price holds; the split's keeps the
        // price and clearing sides alike (docs/unit-1c.md §4.7).
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
        let chain = |rhs: Vec<f64>| self.cc.solve(&rhs);
        let l_star = chain(
            (0..count)
                .map(|j| q.human[j] + q.machine[j] / q.gamma)
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
            let (price, m, z) = (q.prices[index], q.machine[index], category.weight);
            let b_bar = self.chain_land[index];
            let output = z * y;
            let gross_output = self.basket_outputs[index] * y;
            let c = CategoryEq1c {
                price,
                real_wage: q.v / price,
                l_star: l_star[index],
                l_bar: self.all_human[index],
                human: human[index],
                machine: m,
                lambda_tilde: lambda_tilde[index],
                b_tilde: b_tilde[index],
                lambda_tilde_q: lambda_tilde_q[index],
                b_tilde_q: b_tilde_q[index],
                output,
                final_hours: gross_output * human[index],
                machine_services: gross_output * m,
                share: z * price / q.p_s,
                wage_floor: 1.0 / (self.all_human[index] + b_bar / q.v),
                wage_ceiling: (b_tilde[index] > 0.0).then(|| q.v / b_tilde[index]),
                phi_w: flow_prices.then(|| q.v * lambda_tilde[index] / price),
                phi_r: flow_prices.then(|| b_tilde[index] / price),
                chain_land: b_bar,
                gross_output,
            };
            l_star_s += z * c.l_star;
            l_s += z * c.lambda_tilde;
            b_s += z * c.b_tilde;
            l_s_q += z * c.lambda_tilde_q;
            b_s_q += z * c.b_tilde_q;
            spending += price * output;
            fork = worse(fork, (price - (q.v * c.l_star + b_bar)).abs() / price);
            totals = worse(
                totals,
                (price - (q.v * c.lambda_tilde + c.b_tilde)).abs() / price,
            );
            categories.push(c);
        }
        let segment = segment_of(&p.edges, q.x);
        let margin_active = p
            .categories
            .iter()
            .enumerate()
            .any(|(j, c)| self.basket_outputs[j] > 0.0 && c.density[segment] > 0.0);
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
                closure_wage: block.closure_wage(q.gamma, k),
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
        let residuals = self.residuals(&ResidualInputs {
            q: &q,
            technique,
            tie,
            y,
            services: &services,
            task_services: &task_services,
            n_a,
            income,
            spending,
            fork,
            totals,
        });
        Eq1c {
            x_star: x,
            one_minus_x_star: one_minus_x,
            gamma_star: q.gamma,
            m_s: q.m_s,
            u: u[technique],
            v: q.v,
            p_s: q.p_s,
            y,
            final_hours,
            machine_hours,
            n_a,
            participation: (n_a / p.workers).min(1.0),
            income,
            interest,
            labor_share: wages / income,
            capital_share: interest / income,
            real_wage: q.v / q.p_s,
            support_cost,
            worker_baskets: p.workers + wages / q.p_s,
            provider_baskets,
            funded: provider_baskets > 0.0,
            lemma_b1: at_one.n_s > at_one.n_d && p.land > p.workers * at_one.p_s,
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
            switches: self
                .envelope
                .switches
                .iter()
                .zip(points)
                .map(|(s, &x)| SwitchPoint {
                    gamma: s.gamma,
                    x,
                    below: s.below,
                    above: s.above,
                })
                .collect(),
            types: type_eqs,
            categories,
        }
    }

    /// docs/unit-1c.md §6's residuals, each recomputed from the recipes in a fixed order, so
    /// that for one type unit 1b's eight are its own bit for bit.
    fn residuals(&self, r: &ResidualInputs) -> Residuals1c {
        let p = &self.params;
        let block = &self.block;
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
        // The user-cost rows: |p_k − (O_k + u_k·V_k)| with O and V from the recipes.
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
        // The share of the machine tasks each type does: 1 for τ, or the tie's split.
        let task_share = |k: usize| -> f64 {
            match r.tie {
                None if k == r.technique => 1.0,
                Some(t) if k == r.technique => 1.0 - t.share,
                Some(t) if k == t.above => t.share,
                _ => 0.0,
            }
        };
        // p = Ap + λv + b over the C + K rows (SSRN eq 3; A.1).
        let count = p.categories.len();
        let mut leontief_price = 0.0;
        for j in 0..count {
            let mut inputs = 0.0;
            for l in 0..count {
                inputs += p.intermediate[j][l] * q.prices[l];
            }
            for (k, t) in types.iter().enumerate() {
                let share = task_share(k);
                if share > 0.0 {
                    inputs += (share * (q.machine[j] / t.task_efficiency)) * q.type_prices[k];
                }
            }
            let cost = inputs + q.human[j] * q.v + p.categories[j].direct_land;
            leontief_price = worse(leontief_price, (q.prices[j] - cost).abs() / q.prices[j]);
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
        // y = A^qᵀy + f over the rows with y_i > 0 (SSRN A.1: f = (I − Aᵀ)y).
        let gross: Vec<f64> = self.basket_outputs.iter().map(|b| b * r.y).collect();
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
        // SSRN A.1's closure for τ, and no task type cheaper than τ.
        let closure = match block.closure_wage(q.gamma, r.technique) {
            Some(w) => (q.v - w).abs() / q.v,
            None => f64::NAN,
        };
        let mut cheapest = 0.0;
        for (t, mt) in types.iter().enumerate() {
            if mt.task_efficiency > 0.0 {
                let undercut =
                    (q.task_price - q.type_prices[t] / mt.task_efficiency) / q.task_price;
                cheapest = worse(cheapest, undercut.max(0.0));
            }
        }
        Residuals1c {
            labor: (r.n_a - q.n_s).abs(),
            income: (r.y * q.p_s - r.income).abs() / r.income,
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
        }
    }
}

/// The inputs of [`MachineEconomy::residuals`].
struct ResidualInputs<'a> {
    q: &'a MachinePoint,
    technique: usize,
    tie: Option<Tie>,
    y: f64,
    services: &'a [f64],
    task_services: &'a [f64],
    n_a: f64,
    income: f64,
    spending: f64,
    fork: f64,
    totals: f64,
}

/// §5.1 step 4's quantities.
pub(crate) struct Cleared {
    pub(crate) y: f64,
    pub(crate) services: Vec<f64>,
    pub(crate) machine_hours: f64,
}

/// The larger of two residuals, NaN if either is, so that a NaN residual is not lost in a
/// maximum.
pub(crate) fn worse(so_far: f64, next: f64) -> f64 {
    if next.is_nan() || next > so_far {
        next
    } else {
        so_far
    }
}

/// Prices and quantities at a candidate threshold x with one technique (docs/unit-1c.md
/// §5.1), with r = 1. Per-type vectors are in type order, per-category ones in category
/// order.
#[derive(Clone, Debug, PartialEq)]
pub struct MachinePoint {
    /// The threshold x.
    pub x: f64,
    /// γ(x).
    pub gamma: f64,
    /// J(x) = ∫₀ˣ γ.
    pub j: f64,
    /// τ, the task type doing the machine tasks.
    pub technique: usize,
    /// The least pivot of the (O, V) system: τ is viable at x when d > 0; for unit 1a, D(x).
    pub d: f64,
    /// p_k = O_k + u_k·V_k, each type's service price.
    pub type_prices: Vec<f64>,
    /// O_k, the operating cost per unit of service.
    pub operating: Vec<f64>,
    /// V_k, the build cost per unit of capacity.
    pub build: Vec<f64>,
    /// v = γ(x)·p_τ/θ_τ, the task margin.
    pub v: f64,
    /// π = p_τ/θ_τ, the delivered price of a machine task at efficiency 1.
    pub task_price: f64,
    /// p_j = Σ_l a_jl·p_l + v·H_j + π·M_j + b_j, each category's price.
    pub prices: Vec<f64>,
    /// H_j, hours at human tasks per unit of category j.
    pub human: Vec<f64>,
    /// M_j, machine task services per unit of category j, at efficiency 1.
    pub machine: Vec<f64>,
    /// P_s = Σ z_j·p_j.
    pub p_s: f64,
    /// H_ŷ = Σ ŷ_j·H_j, hours at final tasks per basket.
    pub h_s: f64,
    /// M_ŷ = Σ ŷ_j·M_j, machine task services per basket.
    pub m_s: f64,
    /// B_ŷ = Σ ŷ_j·b_j.
    pub b_d: f64,
    /// Y = T/(B_ŷ + Σ_k b^q_k·x̂_k), baskets, from land clearing.
    pub y: f64,
    /// X_k, each type's services (= capacity).
    pub services: Vec<f64>,
    /// Y·H_ŷ.
    pub final_hours: f64,
    /// Σ_k λ^q_k·X_k.
    pub machine_hours: f64,
    /// n_D, labour demand.
    pub n_d: f64,
    /// n_S = N·F(ln(1 + v/P_s)).
    pub n_s: f64,
}

impl MachinePoint {
    /// f(x) = n_D − n_S.
    pub fn excess_demand(&self) -> f64 {
        self.n_d - self.n_s
    }
}

/// A tie at a technique switch (docs/unit-1c.md §4.7): the equilibrium sits on the jump of
/// labour demand, both types deliver tasks at the same cost, and labour clears by the split.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tie {
    /// The type above the switch; [`Eq1c::technique`] is the type below.
    pub above: usize,
    /// σ ∈ (0, 1], the share of the machine tasks the type above does.
    pub share: f64,
    /// γ_i, the switch.
    pub gamma: f64,
}

/// A switch of the cheapest task type on the line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchPoint {
    /// γ_i.
    pub gamma: f64,
    /// x_i, the largest double with γ(x) < γ_i.
    pub x: f64,
    /// The type below.
    pub below: usize,
    /// The type above.
    pub above: usize,
}

/// One machine type at the equilibrium (docs/unit-1c.md §4.8), in r = 1 units.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeEq {
    /// p_k, the price of one unit of the type's service.
    pub price: f64,
    /// O_k, the operating cost per unit of service.
    pub operating_cost: f64,
    /// V_k, the build cost per unit of capacity (p_K; 1a's V_m).
    pub build_cost: f64,
    /// u_k.
    pub user_cost: f64,
    /// ω_k.
    pub wealth_factor: f64,
    /// p_k/θ_k, the delivered cost of a task at efficiency 1, for a task type.
    pub delivered_cost: Option<f64>,
    /// v_k(x*) = γ*b̃_k/(θ_k − γ*λ̃_k), the wage were the type at the margin, for a task type
    /// viable at x*. At least v (SSRN A.1's unused methods).
    pub closure_wage: Option<f64>,
    /// λ̃_k, total hours per unit of service, price side.
    pub lambda_tilde: f64,
    /// b̃_k, total land per unit of service, price side.
    pub b_tilde: f64,
    /// λ̃^q_k, total hours per unit of service delivered, clearing side.
    pub lambda_tilde_q: f64,
    /// b̃^q_k, total land per unit of service delivered, clearing side.
    pub b_tilde_q: f64,
    /// X_k, services per period, equal to installed capacity.
    pub services: f64,
    /// The part of X_k that goes to tasks.
    pub task_services: f64,
    /// δ_k·X_k, capacity built per period.
    pub builds: f64,
    /// λ^q_k·X_k, hours operating and building the type.
    pub hours: f64,
    /// b^q_k·X_k, land operating and building the type.
    pub land: f64,
    /// W_k = ω_k·V_k·X_k, machine wealth (check_dynamics L2).
    pub wealth: f64,
    /// ρ·W_k, computed as ρ·ω_k·V_k·X_k.
    pub interest: f64,
    /// v·λ̃_k/p_k, labour's share of the price, when u_k = 1.
    pub phi_w: Option<f64>,
}

/// One category at the equilibrium (docs/unit-1c.md §4.8): unit 1b's
/// [`CategoryEq`](crate::CategoryEq) fields with the chain meanings of §4.4, plus the chain
/// land and the gross output.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryEq1c {
    /// p_j.
    pub price: f64,
    /// v/p_j (SSRN eq 12).
    pub real_wage: f64,
    /// L*_j = ((I − A_cc)⁻¹(H + M/γ))_j, at the double x*.
    pub l_star: f64,
    /// L̄_j through the chain.
    pub l_bar: f64,
    /// H_j, hours at the category's own human tasks per unit, with the carried 1 − x*.
    pub human: f64,
    /// M_j, machine task services per unit, at efficiency 1.
    pub machine: f64,
    /// λ̃_j, total hours per unit, price side.
    pub lambda_tilde: f64,
    /// b̃_j, total land per unit, price side.
    pub b_tilde: f64,
    /// λ̃^q_j, total hours per unit, clearing side.
    pub lambda_tilde_q: f64,
    /// b̃^q_j, total land per unit, clearing side.
    pub b_tilde_q: f64,
    /// z_j·Y, final output.
    pub output: f64,
    /// Gross output times H_j: hours at the category's own tasks.
    pub final_hours: f64,
    /// Gross output times M_j.
    pub machine_services: f64,
    /// z_j·p_j/P_s.
    pub share: f64,
    /// 1/(L̄_j + b̄_j/v), the lower end of the pair.
    pub wage_floor: f64,
    /// v/b̃_j, when b̃_j > 0.
    pub wage_ceiling: Option<f64>,
    /// v·λ̃_j/p_j, when every u_k = 1.
    pub phi_w: Option<f64>,
    /// b̃_j/p_j, when every u_k = 1.
    pub phi_r: Option<f64>,
    /// b̄_j, the category's land through the chain.
    pub chain_land: f64,
    /// ŷ_j·Y, gross output, final plus intermediate.
    pub gross_output: f64,
}

/// How far a unit-1c solution is from the equations it must satisfy (docs/unit-1c.md §6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Residuals1c {
    /// |N_a − n_S(x*)|, in hours.
    pub labor: f64,
    /// |Y·P_s − I|/I.
    pub income: f64,
    /// |(B_ŷ·Y + Σ_k b^q_k·X_k) − T|/T.
    pub land: f64,
    /// The largest |(X_k − task_k) − Σ_l A^q_lk·X_l|/X_k over types (0 where X_k = 0).
    pub services: f64,
    /// The largest |p_k − (O_k + u_k·V_k)|/p_k over types, with O and V from the recipes.
    pub user_cost: f64,
    /// The largest |p_j − (v·L*_j + b̄_j)|/p_j.
    pub fork: f64,
    /// The largest |p_j − (v·λ̃_j + b̃_j)|/p_j.
    pub totals: f64,
    /// |Σ_j p_j·z_j·Y − I|/I.
    pub expenditure: f64,
    /// The largest |p_i − (Ap + λv + b)_i|/p_i over the C + K rows.
    pub leontief_price: f64,
    /// The largest |y_i − (A^qᵀy + f)_i|/y_i over the rows with y_i > 0.
    pub leontief_quantity: f64,
    /// |v − γb̃_τ/(θ_τ − γλ̃_τ)|/v.
    pub closure: f64,
    /// The largest (π − p_t/θ_t)⁺/π over task types: 0 unless a type undercuts τ.
    pub cheapest: f64,
}

/// The interior equilibrium of unit 1c (docs/unit-1c.md §4.8), in r = 1 units.
///
/// The aggregates carry unit 1b's [`Eq1b`](crate::Eq1b) names where they mean the same
/// thing, with ŷ for z where the chain enters; 1b's machine row is `types[0]` (p_m is its
/// `price`, V_m its `build_cost`, K its `services`, λ̃_m and b̃_m its `lambda_tilde` and
/// `b_tilde`).
#[derive(Clone, Debug, PartialEq)]
pub struct Eq1c {
    /// x*, as [`Eq1a::x_star`](crate::Eq1a::x_star); at a tie, the switch point x_i.
    pub x_star: f64,
    /// 1 − x*, carried as in unit 1a; 1.0 − x_i at a tie.
    pub one_minus_x_star: f64,
    /// γ(x*).
    pub gamma_star: f64,
    /// M_ŷ at x*, machine task services per basket.
    pub m_s: f64,
    /// u of the technique at x*.
    pub u: f64,
    /// v = w/r.
    pub v: f64,
    /// P_s.
    pub p_s: f64,
    /// Y, baskets.
    pub y: f64,
    /// Y·H_ŷ, with the carried 1 − x*.
    pub final_hours: f64,
    /// Σ_k λ^q_k·X_k.
    pub machine_hours: f64,
    /// N_a = final_hours + machine_hours.
    pub n_a: f64,
    /// N_a/N, capped at 1.
    pub participation: f64,
    /// I = v·N_a + T + interest.
    pub income: f64,
    /// Σ_k ρ·W_k.
    pub interest: f64,
    /// v·N_a/I.
    pub labor_share: f64,
    /// interest/I.
    pub capital_share: f64,
    /// v/P_s.
    pub real_wage: f64,
    /// N·P_s.
    pub support_cost: f64,
    /// N + v·N_a/P_s.
    pub worker_baskets: f64,
    /// (T + interest)/P_s − N.
    pub provider_baskets: f64,
    /// provider_baskets > 0.
    pub funded: bool,
    /// n_S(1) > n_D(1) and T > N·P_s(1), with the envelope's technique at 1.
    pub lemma_b1: bool,
    /// φ_w = γλ̃_τ/θ_τ, labour's share of the technique's price, when every u_k = 1.
    pub phi_w: Option<f64>,
    /// φ_r = 1 − φ_w, when every u_k = 1.
    pub phi_r: Option<f64>,
    /// H_ŷ, with the carried 1 − x*.
    pub h_s: f64,
    /// B_ŷ = Σ ŷ_j·b_j.
    pub b_d: f64,
    /// L*_s = zᵀL*, so P_s = v·L*_s + zᵀb̄.
    pub l_star_s: f64,
    /// L_s = zᵀλ̃_c, so P_s = v·L_s + B_s.
    pub l_s: f64,
    /// B_s = zᵀb̃_c.
    pub b_s: f64,
    /// L_s^q = zᵀλ̃^q_c, so n_D = T·L_s^q/B_s^q (SSRN eq 11).
    pub l_s_q: f64,
    /// B_s^q = zᵀb̃^q_c, so Y = T/B_s^q.
    pub b_s_q: f64,
    /// v/B_s.
    pub rent_ceiling: f64,
    /// Whether some category with ŷ_j > 0 has tasks on the segment holding x*.
    pub margin_active: bool,
    /// The residuals of docs/unit-1c.md §6.
    pub residuals: Residuals1c,
    /// Bisection steps of the root; 0 at a tie.
    pub bisection_steps: u32,
    /// The least pivot of the technique's (O, V) system at x* (unit 1a's D(x*)).
    pub d: f64,
    /// τ, the type doing the machine tasks at x* (at a tie, the type below the switch).
    pub technique: usize,
    /// The tie, when the equilibrium sits on a switch.
    pub tie: Option<Tie>,
    /// The envelope's switches on the line.
    pub switches: Vec<SwitchPoint>,
    /// Every machine type, in order.
    pub types: Vec<TypeEq>,
    /// Every category, in order.
    pub categories: Vec<CategoryEq1c>,
}

/// What an output belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    /// The economy.
    Economy,
    /// A switch of the envelope, by position.
    Switch(usize),
    /// A machine type, by index.
    Type(usize),
    /// A category, by index.
    Category(usize),
    /// A worker type, by index (unit 1d).
    Worker(usize),
    /// A switch of the technique on the wall, by position (unit 1d).
    WallSwitch(usize),
    /// A parcel, by index (unit 1e).
    Parcel(usize),
}

/// The key of a unit-1c output, printed `name`, `switch<i>.name`, `type<k>.name` or
/// `cat<j>.name`; unit 1d adds `worker<i>.name` and `wall_switch<s>.name`, unit 1e
/// `parcel<z>.name`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputKey1c {
    /// What the output belongs to.
    pub item: Item,
    /// The field's name.
    pub name: &'static str,
}

impl fmt::Display for OutputKey1c {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.item {
            Item::Economy => write!(f, "{}", self.name),
            Item::Switch(i) => write!(f, "switch{i}.{}", self.name),
            Item::Type(k) => write!(f, "type{k}.{}", self.name),
            Item::Category(j) => write!(f, "cat{j}.{}", self.name),
            Item::Worker(i) => write!(f, "worker{i}.{}", self.name),
            Item::WallSwitch(s) => write!(f, "wall_switch{s}.{}", self.name),
            Item::Parcel(z) => write!(f, "parcel{z}.{}", self.name),
        }
    }
}

impl Eq1c {
    /// Every output: the economy's in unit 1b's order (without 1b's machine row, which is
    /// `type0`), then unit 1c's, then each switch's, each type's and each category's.
    ///
    /// This is the one list of outputs: [`MachineEconomy::solve`] checks every number in it
    /// for finiteness.
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
            ]);
        }
        list
    }
}

/// The error for the first number in [`Eq1c::outputs`] that is NaN or infinite.
fn first_non_finite(eq: &Eq1c) -> Option<SolveError> {
    eq.outputs()
        .into_iter()
        .find_map(|(key, output)| match output {
            Output1b::Float(v) | Output1b::Optional(Some(v)) if !v.is_finite() => {
                Some(match key.item {
                    Item::Economy | Item::Switch(_) | Item::WallSwitch(_) | Item::Parcel(_) => {
                        SolveError::NonFinite { what: key.name }
                    }
                    Item::Worker(worker) => SolveError::NonFiniteInWorker {
                        worker,
                        what: key.name,
                    },
                    Item::Type(machine_type) => SolveError::NonFiniteInType {
                        machine_type,
                        what: key.name,
                    },
                    Item::Category(category) => SolveError::NonFiniteInCategory {
                        category,
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

    /// An `Eq1c` with a tie, one switch, two types and two categories whose numbers are 1, 2,
    /// 3, ... in output order, with the `nan_at`-th replaced by NaN (none when `nan_at` is 0).
    /// A struct literal must name every field, so a field added to `Eq1c`, `TypeEq` or
    /// `CategoryEq1c` does not compile here until it is given a number.
    fn numbered(nan_at: usize) -> Eq1c {
        let mut next = 0;
        let mut n = || {
            next += 1;
            if next == nan_at {
                f64::NAN
            } else {
                next as f64
            }
        };
        let mut eq = Eq1c {
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
            residuals: Residuals1c {
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
            },
            bisection_steps: 7,
            d: 0.0,
            technique: 0,
            tie: None,
            switches: Vec::new(),
            types: Vec::new(),
            categories: Vec::new(),
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
            eq.categories.push(CategoryEq1c {
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
            });
        }
        eq
    }

    /// The numbers of `outputs()`, without the tie's type index (an index, not a number of
    /// the equilibrium; it is 1.0 in `numbered`).
    fn numbers(eq: &Eq1c) -> Vec<(OutputKey1c, f64)> {
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
        // Every number of Eq1c appears in outputs() exactly once, in output order, and every
        // key is distinct.
        let eq = numbered(0);
        let listed = numbers(&eq);
        let values: Vec<f64> = listed.iter().map(|(_, v)| *v).collect();
        let want: Vec<f64> = (1..=listed.len()).map(|i| i as f64).collect();
        assert_eq!(values, want);
        assert_eq!(listed.len(), 45 + 2 + 2 * 19 + 2 * 20);
        let mut keys: Vec<String> = eq.outputs().iter().map(|(k, _)| k.to_string()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), eq.outputs().len());
        for key in [
            "switch0.gamma",
            "type1.phi_w",
            "cat1.gross_output",
            "tie_share",
            "tie_above",
            "d",
        ] {
            assert!(keys.contains(&key.to_string()), "{key}");
        }
        let flags: Vec<(String, Output1b)> = eq
            .outputs()
            .into_iter()
            .filter(|(_, o)| matches!(o, Output1b::Flag(_) | Output1b::Count(_)))
            .map(|(k, o)| (k.to_string(), o))
            .collect();
        assert_eq!(
            flags,
            [
                ("funded".to_string(), Output1b::Flag(true)),
                ("lemma_b1".to_string(), Output1b::Flag(false)),
                ("margin_active".to_string(), Output1b::Flag(true)),
                ("bisection_steps".to_string(), Output1b::Count(7)),
                ("technique".to_string(), Output1b::Count(0)),
                ("tie".to_string(), Output1b::Flag(true)),
                ("switch0.below".to_string(), Output1b::Count(0)),
                ("switch0.above".to_string(), Output1b::Count(1)),
            ]
        );
        let tie_above: Vec<Output1b> = eq
            .outputs()
            .into_iter()
            .filter(|(k, _)| k.name == "tie_above")
            .map(|(_, o)| o)
            .collect();
        assert_eq!(tie_above, [Output1b::Optional(Some(1.0))]);
    }

    #[test]
    fn every_output_is_checked_for_finiteness() {
        assert_eq!(first_non_finite(&numbered(0)), None);
        let keys: Vec<OutputKey1c> = numbers(&numbered(0)).iter().map(|(k, _)| *k).collect();
        for (i, key) in keys.iter().enumerate() {
            let want = match key.item {
                Item::Economy | Item::Switch(_) | Item::WallSwitch(_) | Item::Parcel(_) => {
                    SolveError::NonFinite { what: key.name }
                }
                Item::Worker(worker) => SolveError::NonFiniteInWorker {
                    worker,
                    what: key.name,
                },
                Item::Type(machine_type) => SolveError::NonFiniteInType {
                    machine_type,
                    what: key.name,
                },
                Item::Category(category) => SolveError::NonFiniteInCategory {
                    category,
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
        // An absent optional output is not a number and is skipped.
        let mut eq = numbered(0);
        eq.types[1].closure_wage = None;
        eq.types[1].phi_w = Some(f64::INFINITY);
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFiniteInType {
                machine_type: 1,
                what: "phi_w"
            })
        );
        assert_eq!(
            SolveError::NonFiniteInType {
                machine_type: 1,
                what: "phi_w"
            }
            .to_string(),
            "phi_w of machine type 1 is not finite"
        );
    }

    #[test]
    fn keys_print_their_item() {
        for (item, text) in [
            (Item::Economy, "v"),
            (Item::Switch(2), "switch2.v"),
            (Item::Type(1), "type1.v"),
            (Item::Category(3), "cat3.v"),
        ] {
            assert_eq!(OutputKey1c { item, name: "v" }.to_string(), text);
        }
    }

    /// M4 (docs/unit-1c.md §3.3) at η and N.
    fn m4(eta: f64, workers: f64) -> MachineParams {
        let recipe = |machines: [f64; 3], labor, land| Recipe {
            machines: machines.to_vec(),
            labor,
            land,
        };
        let kind = |theta, operating, build, delta, build_lag| MachineType {
            task_efficiency: theta,
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
        MachineParams {
            workers,
            land: 10.0,
            schedule: PowerSchedule {
                eta,
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
        }
    }

    #[test]
    fn a_tie_share_is_one_when_the_type_above_does_not_clear_labour() {
        // docs/unit-1c.md §5.3 step 3: σ = B_a·f_a/(B_a·f_a − B_b·f_b), and 1 when f_b ≥ 0,
        // which the solve meets only within rounding of a root at the switch. At M4t's switch
        // (η 0.5, N 8) σ is interior; with N 2 labour is short under both techniques there,
        // f_b > 0, and the share is 1.
        let e = MachineEconomy::new(m4(0.5, 8.0)).unwrap();
        let x = e.switch_points().unwrap()[0];
        let s = e.envelope().switches[0];
        let share = e.tie_share(x, s.below, s.above);
        assert!(share > 0.0 && share < 1.0, "{share}");
        let e = MachineEconomy::new(m4(0.5, 2.0)).unwrap();
        let x = e.switch_points().unwrap()[0];
        let (below, above) = (e.at_with(x, s.below), e.at_with(x, s.above));
        assert!(above.n_d - below.n_s > 0.0 && below.n_d - below.n_s > 0.0);
        assert_eq!(e.tie_share(x, s.below, s.above), 1.0);
    }

    #[test]
    fn a_nan_residual_is_not_lost_in_the_maximum() {
        assert!(worse(0.0, f64::NAN).is_nan());
        assert!(worse(f64::NAN, 1.0).is_nan());
        assert_eq!(worse(1e-16, 3e-16), 3e-16);
        assert_eq!(worse(3e-16, 1e-16), 3e-16);
    }
}
