//! Unit 1b: many categories on one task line, bought in a fixed basket (docs/unit-1b.md).
//!
//! Every household consumes the categories in the fixed proportions z, the paper's own
//! closure (SSRN eq 7 and §5, pp.10-12). The categories share one task line [0, 1] with
//! one capability schedule γ; category j uses μ_js hours by hand per unit of output per
//! unit length of line on segment s. Machines do the tasks below the threshold x in every
//! category, people those above. Space is a category with no tasks and direct land 1
//! (SSRN p.8's pass-through row), so SSRN Appendix B is the basket z = (1, h) over the good
//! and space, and a one-category economy repeats unit 1a's arithmetic bit for bit
//! (docs/unit-1b.md §5.1).

use std::fmt;

use rustyecon_core::num;

use crate::closure::viability;
use crate::params::{self, user_cost, wealth_factor, ParamError, Params, UniformWorkCost};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{classify, labor_net, Regime, Root, SolveError, BRACKET_HI};

/// One category of final goods or services (docs/unit-1b.md §3.1).
///
/// "0 or scale" below means 0, or finite and in [[`SCALE_FLOOR`](crate::SCALE_FLOOR),
/// [`SCALE_CEIL`](crate::SCALE_CEIL)].
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    /// z_j, units of the category per basket. 0 or scale. A category with weight 0 is
    /// priced but not bought.
    pub weight: f64,
    /// b_j, land services per unit of the category, used directly (acreage, sites). 0 or
    /// scale.
    pub direct_land: f64,
    /// μ_js, hours by hand on segment s per unit of the category per unit length of task
    /// line: main.tex's 1/γ_Lj with the task quantities in the integration measure
    /// (main.tex:447). One entry per segment, each 0 or scale.
    pub density: Vec<f64>,
}

/// The parameters of unit 1b (docs/unit-1b.md §3.1): unit 1a's [`Params`] without `space`,
/// plus the task line's segments and the categories. Build a [`CategoryEconomy`] from them
/// to validate.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryParams<S = PowerSchedule> {
    /// N, potential workers. Scale.
    pub workers: f64,
    /// T, the land-service endowment per period. Scale.
    pub land: f64,
    /// a, machine services used per machine built. 0 ≤ a < 1.
    pub a: f64,
    /// λ, hours per machine built. 0 ≤ λ ≤ [`SCALE_CEIL`](crate::SCALE_CEIL).
    pub lam: f64,
    /// b, land services per machine built. Scale.
    pub b: f64,
    /// γ(x), relative human productivity on the one task line.
    pub schedule: S,
    /// F, the distribution of the work cost χ.
    pub work_cost: UniformWorkCost,
    /// ρ, the required return per period. 0 ≤ ρ ≤ [`SCALE_CEIL`](crate::SCALE_CEIL).
    pub rho: f64,
    /// δ, geometric depreciation per period. [`SCALE_FLOOR`](crate::SCALE_FLOOR) ≤ δ ≤ 1.
    pub delta: f64,
    /// J_b, periods from the start of a build to its first service. J_b ≥ 1.
    pub build_lag: u32,
    /// e_0 … e_S, the segments' edges on the task line: e_0 = 0 and e_S = 1 exactly,
    /// strictly increasing.
    pub edges: Vec<f64>,
    /// The categories, in the order every sum over categories runs.
    pub categories: Vec<Category>,
}

impl<S> CategoryParams<S> {
    /// Unit 1a's economy in category form (docs/unit-1b.md §3.3, C1): one segment, the good
    /// (z 1, b 0, μ 1) and space (z = h, b 1, μ 0). It solves to 1a's equilibrium bit for
    /// bit.
    pub fn from_one_category(params: Params<S>) -> Self {
        CategoryParams {
            workers: params.workers,
            land: params.land,
            a: params.a,
            lam: params.lam,
            b: params.b,
            schedule: params.schedule,
            work_cost: params.work_cost,
            rho: params.rho,
            delta: params.delta,
            build_lag: params.build_lag,
            edges: vec![0.0, 1.0],
            categories: vec![
                Category {
                    weight: 1.0,
                    direct_land: 0.0,
                    density: vec![1.0],
                },
                Category {
                    weight: params.space,
                    direct_land: 1.0,
                    density: vec![0.0],
                },
            ],
        }
    }
}

/// A validated unit-1b economy: parameters that passed every check in
/// [`CategoryEconomy::new`], with u and the quantities that do not depend on x.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryEconomy<S = PowerSchedule> {
    params: CategoryParams<S>,
    u: f64,
    /// J(e_s) at every edge; J(e_0) = 0.
    integral_at_edges: Vec<f64>,
    /// L̄_j for every category.
    all_human: Vec<f64>,
    /// B_d = Σ z_j·b_j, summed from 0.0 in category order.
    basket_land: f64,
    /// L̄_s = Σ z_j·L̄_j.
    basket_all_human: f64,
}

/// The error for a parameter of category `index`.
fn in_category(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "category",
        index,
        error: Box::new(error),
    }
}

/// L̄_j = Σ_s μ_js·(e_s − e_{s−1}), summed from 0.0 in segment order: the hours of the
/// all-human method (main.tex:445). The same terms, in the same order, as the hours at a
/// threshold below every task of the category, so the two are equal bit for bit.
pub(crate) fn all_human_hours(edges: &[f64], density: &[f64]) -> f64 {
    let mut hours = 0.0;
    for (s, &mu) in density.iter().enumerate() {
        hours += mu * (edges[s + 1] - edges[s]);
    }
    hours
}

/// The task line's checks (docs/unit-1b.md §3.2), shared by units 1b and 1c: at least two
/// edges, finite, the first exactly 0 and the last exactly 1, strictly increasing, with −0.0
/// stored as +0.0; J(0) exactly 0. Returns J at every edge.
pub(crate) fn validate_line<S: Schedule>(
    edges: &mut [f64],
    schedule: &S,
) -> Result<Vec<f64>, ParamError> {
    if edges.len() < 2 {
        return Err(ParamError::Invalid {
            name: "edges",
            reason: "the task line needs at least two edges, 0 and 1",
        });
    }
    for edge in edges.iter_mut() {
        *edge = params::finite("edges", *edge)? + 0.0;
    }
    let last = edges.len() - 1;
    if edges[0] != 0.0 || edges[last] != 1.0 {
        return Err(ParamError::Invalid {
            name: "edges",
            reason: "the first edge must be exactly 0 and the last exactly 1",
        });
    }
    if edges.windows(2).any(|w| w[1] <= w[0]) {
        return Err(ParamError::Invalid {
            name: "edges",
            reason: "the edges must strictly increase",
        });
    }
    if schedule.integral(0.0) != 0.0 {
        return Err(ParamError::Invalid {
            name: "J(0)",
            reason: "J(0) must be exactly 0: J is the integral of gamma from 0",
        });
    }
    let mut integral_at_edges = Vec::with_capacity(edges.len());
    for &edge in edges.iter() {
        integral_at_edges.push(params::finite("J(edge)", schedule.integral(edge))?);
    }
    Ok(integral_at_edges)
}

/// One category's own checks (docs/unit-1b.md §3.2), shared by units 1b and 1c: weight,
/// direct land and densities 0 or scale, one density per segment, −0.0 stored as +0.0.
pub(crate) fn validate_category(
    category: &mut Category,
    segments: usize,
) -> Result<(), ParamError> {
    category.weight = params::zero_or_scale("weight", category.weight)?;
    category.direct_land = params::zero_or_scale("direct_land", category.direct_land)?;
    if category.density.len() != segments {
        return Err(ParamError::Invalid {
            name: "density",
            reason: "a category needs one density per segment of the task line",
        });
    }
    for mu in &mut category.density {
        *mu = params::zero_or_scale("density", *mu)?;
    }
    Ok(())
}

/// (H_j, M_j) for one category at threshold x, with J(x) given (docs/unit-1b.md §4.1 and
/// §5.1 step 2): hours at human tasks and machine services per unit of the category, from
/// its densities on the task line's segments and J at their edges.
///
/// With `carried` = Some(1 − x*), the top segment's human length e_S − x is the carried
/// 1 − x* whenever x lies in the top segment, x = 1 included (§5.1 step 5).
pub(crate) fn tasks(
    edges: &[f64],
    integral_at_edges: &[f64],
    density: &[f64],
    x: f64,
    j_x: f64,
    carried: Option<f64>,
) -> (f64, f64) {
    let je = integral_at_edges;
    let top = edges.len() - 2;
    let (mut human, mut machine) = (0.0, 0.0);
    for (s, &mu) in density.iter().enumerate() {
        let (lo, hi) = (edges[s], edges[s + 1]);
        if x >= hi {
            machine += mu * (je[s + 1] - je[s]);
            if let (true, Some(one_minus_x)) = (s == top, carried) {
                human += mu * one_minus_x;
            }
        } else if x <= lo {
            human += mu * (hi - lo);
        } else {
            let length = match carried {
                Some(one_minus_x) if s == top => one_minus_x,
                _ => hi - x,
            };
            human += mu * length;
            machine += mu * (j_x - je[s]);
        }
    }
    (human, machine)
}

/// The segment holding x: e_s ≤ x < e_{s+1}, and the top segment for x ≥ 1.
pub(crate) fn segment_of(edges: &[f64], x: f64) -> usize {
    let above = edges.partition_point(|&e| e <= x);
    above.clamp(1, edges.len() - 1) - 1
}

impl<S: Schedule> CategoryEconomy<S> {
    /// Validates the parameters (docs/unit-1b.md §3.2) and computes u.
    ///
    /// Unit 1a's checks come first, in 1a's order and with 1a's names (N, T, a, λ, b, the
    /// schedule, χ_max, ρ, δ, J_b, u; there is no h). Then:
    /// - `edges`: at least two, finite, the first exactly 0 and the last exactly 1, strictly
    ///   increasing; J(0) must be exactly 0, as the one-category nesting needs;
    /// - at least one category; each category's `weight`, `direct_land` and `density`
    ///   entries 0 or scale, with one density per segment, and L̄_j > 0 or b_j > 0, so that
    ///   it is positively priced (SSRN Prop 4's premise);
    /// - the basket uses land directly, B_d = Σ z_j·b_j > 0 (1a's h > 0), and needs work,
    ///   L̄_s = Σ z_j·L̄_j > 0.
    ///
    /// Viability is not checked here: D(1) ≤ 0 solves to
    /// [`Regime::NotViable`](crate::Regime::NotViable), at the top of the line whether or not
    /// a basket category uses its top segment (1a's convention). A negative zero in a, λ, ρ,
    /// an edge, a weight, a direct land or a density is stored as +0.0.
    pub fn new(mut params: CategoryParams<S>) -> Result<Self, ParamError> {
        // Unit 1a's checks, in its order (params.rs, Economy::new).
        params::scale("workers", params.workers)?;
        params::scale("land", params.land)?;
        params.a = params::machine_share("a", params.a)?;
        params.lam = params::nonnegative("lam", params.lam)?;
        params::scale("b", params.b)?;
        params.schedule.validate()?;
        params::scale("chi_max", params.work_cost.chi_max)?;
        params.rho = params::nonnegative("rho", params.rho)?;
        params::depreciation("delta", params.delta)?;
        if params.build_lag == 0 {
            return Err(ParamError::BuildLag {
                value: params.build_lag,
            });
        }
        let u = user_cost(params.rho, params.delta, params.build_lag);
        if !u.is_finite() {
            return Err(ParamError::UserCostNotFinite { u });
        }
        // The task line.
        let integral_at_edges = validate_line(&mut params.edges, &params.schedule)?;
        // The categories.
        if params.categories.is_empty() {
            return Err(ParamError::Invalid {
                name: "categories",
                reason: "the economy needs at least one category",
            });
        }
        let segments = params.edges.len() - 1;
        let mut all_human = Vec::with_capacity(params.categories.len());
        for (index, category) in params.categories.iter_mut().enumerate() {
            let item = in_category(index);
            validate_category(category, segments).map_err(&item)?;
            let hours = all_human_hours(&params.edges, &category.density);
            if !(hours > 0.0 || category.direct_land > 0.0) {
                return Err(item(ParamError::Invalid {
                    name: "category",
                    reason: "a category needs tasks or direct land, so that its price is \
                             positive",
                }));
            }
            all_human.push(hours);
        }
        // The basket (docs/unit-1b.md §3.2).
        let (mut basket_land, mut basket_all_human) = (0.0, 0.0);
        for (category, hours) in params.categories.iter().zip(&all_human) {
            basket_land += category.weight * category.direct_land;
            basket_all_human += category.weight * hours;
        }
        // Sums of finite nonnegative products: never NaN, and 0 only when every term is.
        if basket_land <= 0.0 {
            return Err(ParamError::Invalid {
                name: "basket",
                reason: "the basket must use land directly: the sum of weight times \
                         direct_land must be positive",
            });
        }
        if basket_all_human <= 0.0 {
            return Err(ParamError::Invalid {
                name: "basket",
                reason: "the basket must need work: the sum of weight times all-human hours \
                         must be positive",
            });
        }
        Ok(CategoryEconomy {
            params,
            u,
            integral_at_edges,
            all_human,
            basket_land,
            basket_all_human,
        })
    }

    /// The validated parameters.
    pub fn params(&self) -> &CategoryParams<S> {
        &self.params
    }

    /// u = (ρ + δ)(1 + ρ)^(J_b − 1).
    pub fn user_cost(&self) -> f64 {
        self.u
    }

    /// L̄_j, the hours of each category's all-human method (main.tex:445), in category order.
    pub fn all_human_hours(&self) -> &[f64] {
        &self.all_human
    }

    /// B_d = Σ z_j·b_j, the basket's direct land.
    pub fn basket_direct_land(&self) -> f64 {
        self.basket_land
    }

    /// L̄_s = Σ z_j·L̄_j, the basket's all-human hours.
    pub fn basket_all_human_hours(&self) -> f64 {
        self.basket_all_human
    }

    /// (H_j, M_j) for one category at threshold x: the free function [`tasks`] on this
    /// economy's line.
    fn tasks(&self, category: &Category, x: f64, j_x: f64, carried: Option<f64>) -> (f64, f64) {
        tasks(
            &self.params.edges,
            &self.integral_at_edges,
            &category.density,
            x,
            j_x,
            carried,
        )
    }

    /// Prices and quantities at a candidate threshold x in [0, 1] (docs/unit-1b.md §4.1-4.3),
    /// in the evaluation order of §5.1, which for one category is unit 1a's
    /// [`Economy::at`](crate::Economy::at) operation for operation.
    ///
    /// Outside [0, 1], or where D(x) ≤ 0, the formulas are evaluated as written and mean
    /// nothing.
    pub fn at(&self, x: f64) -> CategoryPoint {
        let p = &self.params;
        let u = self.u;
        let gamma = p.schedule.gamma(x);
        let j = p.schedule.integral(x);
        // Step 1: the machine row and the task margin (1a's order).
        let d = viability(u, p.a, p.lam, gamma);
        let v_m = p.b / d;
        let p_m = u * v_m;
        let v = gamma * p_m;
        // Steps 2-3: each category's tasks and price, and the basket's sums from 0.0.
        let count = p.categories.len();
        let (mut prices, mut human, mut machine) = (
            Vec::with_capacity(count),
            Vec::with_capacity(count),
            Vec::with_capacity(count),
        );
        let (mut p_s, mut h_s, mut m_s) = (0.0, 0.0, 0.0);
        for category in &p.categories {
            let (h, m) = self.tasks(category, x, j, None);
            let price = v * h + p_m * m + category.direct_land;
            p_s += category.weight * price;
            h_s += category.weight * h;
            m_s += category.weight * m;
            prices.push(price);
            human.push(h);
            machine.push(m);
        }
        // Step 4: the quantity block (1a's, with h → B_d and J → M_s).
        let b_d = self.basket_land;
        let s_k = num::fma(-p.a, p.delta, 1.0);
        let y = p.land / (b_d + p.b * p.delta * m_s / s_k);
        let k = y * m_s / s_k;
        let final_hours = y * h_s;
        let machine_hours = p.lam * p.delta * k;
        let n_d = final_hours + machine_hours;
        let n_s = p.workers * p.work_cost.cdf(num::ln1p(v / p_s));
        CategoryPoint {
            x,
            gamma,
            j,
            d,
            v_m,
            p_m,
            v,
            prices,
            human,
            machine,
            p_s,
            h_s,
            m_s,
            b_d,
            y,
            k,
            final_hours,
            machine_hours,
            n_d,
            n_s,
        }
    }

    /// Classifies the economy and, when interior, solves n_D(x*) = n_S(x*)
    /// (docs/unit-1b.md §5), exactly as unit 1a's [`Economy::solve`](crate::Economy::solve)
    /// does: the same regime tests in the same order, the same bracket, bisection to adjacent
    /// doubles, and 1 − x* carried from the line through them.
    ///
    /// Returns [`SolveError::NonFinite`] or [`SolveError::NonFiniteInCategory`] if a bracket
    /// value or any reported output is not finite, and [`SolveError::LaborNotCleared`] if the
    /// reported equilibrium misses labour clearing by more than
    /// [`LABOR_RESIDUAL_NET`](crate::LABOR_RESIDUAL_NET) of N_a.
    pub fn solve(&self) -> Result<Regime<Eq1b>, SolveError> {
        let at_hi = self.at(BRACKET_HI);
        let root = match classify(at_hi.d, at_hi.excess_demand(), |x| {
            self.at(x).excess_demand()
        })? {
            Ok(root) => root,
            Err(regime) => return Ok(regime),
        };
        let eq = self.report(root, &at_hi);
        if let Some(error) = first_non_finite(&eq) {
            return Err(error);
        }
        labor_net(eq.x_star, eq.residuals.labor, eq.n_a)?;
        Ok(Regime::Interior(Box::new(eq)))
    }

    /// The segment holding x: e_s ≤ x < e_{s+1}, and the top segment for x ≥ 1.
    fn segment_of(&self, x: f64) -> usize {
        segment_of(&self.params.edges, x)
    }

    /// docs/unit-1b.md §4.2-4.4 and §5.1 step 5 at x*.
    fn report(&self, root: Root, at_one: &CategoryPoint) -> Eq1b {
        let p = &self.params;
        let u = self.u;
        let q = self.at(root.x);
        let one_minus_x = root.one_minus_x;
        // Hours-type outputs take the carried 1 − x* in the top segment; prices and L* are
        // evaluated at the double x* (§5.1 step 5, as 1a's Eq1a does).
        let mut human = Vec::with_capacity(p.categories.len());
        let mut h_s = 0.0;
        for category in &p.categories {
            let (h, _) = self.tasks(category, q.x, q.j, Some(one_minus_x));
            h_s += category.weight * h;
            human.push(h);
        }
        // 1a's report, with h → B_d and J → M_s, in 1a's order.
        let final_hours = q.y * h_s;
        let n_a = final_hours + q.machine_hours;
        let interest = p.rho * wealth_factor(p.rho, p.delta, p.build_lag) * q.v_m * q.k;
        let wages = q.v * n_a;
        let income = wages + p.land + interest;
        let support_cost = p.workers * q.p_s;
        let provider_baskets = (p.land + interest) / q.p_s - p.workers;
        let flow_prices = u == 1.0;
        let phi_w = flow_prices.then(|| p.lam * q.gamma / (1.0 - p.a));
        // The machine row's totals, price side (scaled by u, §4.2) and clearing side (by δ,
        // §4.3). At u = 1, u·λ and 1 − u·a are 1a's λ and 1.0 − a bit for bit.
        let one_minus_ua = num::fma(-u, p.a, 1.0);
        let lambda_tilde_machine = u * p.lam / one_minus_ua;
        let b_tilde_machine = u * p.b / one_minus_ua;
        let s_k = num::fma(-p.a, p.delta, 1.0);
        let lambda_q_machine = p.lam * p.delta / s_k;
        let b_q_machine = p.b * p.delta / s_k;
        let mut categories = Vec::with_capacity(p.categories.len());
        let (mut l_star_s, mut l_s, mut b_s, mut l_s_q, mut b_s_q) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut spending, mut fork, mut totals) = (0.0, 0.0, 0.0);
        for (index, category) in p.categories.iter().enumerate() {
            let (price, m, b_j, z) = (
                q.prices[index],
                q.machine[index],
                category.direct_land,
                category.weight,
            );
            let l_star = q.human[index] + m / q.gamma;
            let lambda_tilde = human[index] + m * lambda_tilde_machine;
            let b_tilde = b_j + m * b_tilde_machine;
            let output = z * q.y;
            let c = CategoryEq {
                price,
                real_wage: q.v / price,
                l_star,
                l_bar: self.all_human[index],
                human: human[index],
                machine: m,
                lambda_tilde,
                b_tilde,
                lambda_tilde_q: human[index] + m * lambda_q_machine,
                b_tilde_q: b_j + m * b_q_machine,
                output,
                final_hours: output * human[index],
                machine_services: output * m,
                share: z * price / q.p_s,
                wage_floor: 1.0 / (self.all_human[index] + b_j / q.v),
                wage_ceiling: (b_tilde > 0.0).then(|| q.v / b_tilde),
                phi_w: flow_prices.then(|| q.v * lambda_tilde / price),
                phi_r: flow_prices.then(|| b_tilde / price),
            };
            l_star_s += z * c.l_star;
            l_s += z * c.lambda_tilde;
            b_s += z * c.b_tilde;
            l_s_q += z * c.lambda_tilde_q;
            b_s_q += z * c.b_tilde_q;
            spending += price * output;
            fork = worse(fork, (price - (q.v * c.l_star + b_j)).abs() / price);
            totals = worse(
                totals,
                (price - (q.v * c.lambda_tilde + c.b_tilde)).abs() / price,
            );
            categories.push(c);
        }
        let segment = self.segment_of(q.x);
        let margin_active = p
            .categories
            .iter()
            .any(|c| c.weight > 0.0 && c.density[segment] > 0.0);
        let residuals = Residuals1b {
            labor: (n_a - q.n_s).abs(),
            income: (q.y * q.p_s - income).abs() / income,
            land: (q.b_d * q.y + p.b * p.delta * q.k - p.land).abs() / p.land,
            services: if q.k > 0.0 {
                (q.k - q.y * q.m_s - p.a * p.delta * q.k).abs() / q.k
            } else {
                0.0
            },
            user_cost: (q.p_m - u * (p.a * q.p_m + p.lam * q.v + p.b)).abs() / q.p_m,
            fork,
            totals,
            expenditure: (spending - income).abs() / income,
        };
        Eq1b {
            x_star: root.x,
            one_minus_x_star: one_minus_x,
            gamma_star: q.gamma,
            m_s: q.m_s,
            u,
            v: q.v,
            p_m: q.p_m,
            v_m: q.v_m,
            p_s: q.p_s,
            y: q.y,
            k: q.k,
            final_hours,
            machine_hours: q.machine_hours,
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
            lambda_tilde_machine,
            b_tilde_machine,
            h_s,
            b_d: q.b_d,
            l_star_s,
            l_s,
            b_s,
            l_s_q,
            b_s_q,
            rent_ceiling: q.v / b_s,
            margin_active,
            categories,
            residuals,
            bisection_steps: root.steps,
        }
    }
}

/// The larger of two residuals, NaN if either is, so that a NaN residual is not lost in a
/// maximum.
fn worse(so_far: f64, next: f64) -> f64 {
    if next.is_nan() || next > so_far {
        next
    } else {
        so_far
    }
}

/// Prices and quantities at a candidate threshold x (docs/unit-1b.md §4.1-4.3), with r = 1.
/// Per-category vectors are in category order.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryPoint {
    /// The threshold x.
    pub x: f64,
    /// γ(x).
    pub gamma: f64,
    /// J(x) = ∫₀ˣ γ.
    pub j: f64,
    /// D(x) = 1 − u(a + λγ(x)).
    pub d: f64,
    /// V_m = b/D, the cost of building one machine.
    pub v_m: f64,
    /// p_m = u·V_m, the machine-service price.
    pub p_m: f64,
    /// v = γ(x)·p_m, the task margin.
    pub v: f64,
    /// p_j = v·H_j + p_m·M_j + b_j, each task at its cheaper method (main.tex:477-490).
    pub prices: Vec<f64>,
    /// H_j, hours at human tasks per unit of category j.
    pub human: Vec<f64>,
    /// M_j, machine services per unit of category j.
    pub machine: Vec<f64>,
    /// P_s = Σ z_j·p_j, the basket's price (SSRN eq 7).
    pub p_s: f64,
    /// H_s = Σ z_j·H_j.
    pub h_s: f64,
    /// M_s = Σ z_j·M_j.
    pub m_s: f64,
    /// B_d = Σ z_j·b_j.
    pub b_d: f64,
    /// Y = T/(B_d + b·δ·M_s/(1 − aδ)), baskets, from land clearing.
    pub y: f64,
    /// K = Y·M_s/(1 − aδ), installed machines.
    pub k: f64,
    /// Y·H_s, hours at final tasks.
    pub final_hours: f64,
    /// λ·δ·K, hours building machines.
    pub machine_hours: f64,
    /// n_D, labour demand.
    pub n_d: f64,
    /// n_S = N·F(ln(1 + v/P_s)), labour supply (SSRN eq 10).
    pub n_s: f64,
}

impl CategoryPoint {
    /// f(x) = n_D − n_S, the function whose root is x*.
    pub fn excess_demand(&self) -> f64 {
        self.n_d - self.n_s
    }
}

/// One category at the equilibrium (docs/unit-1b.md §4.4), in r = 1 units.
///
/// Prices, `real_wage` and `l_star` are evaluated at the double x*; the hours-type fields
/// (`human`, `lambda_tilde`, `lambda_tilde_q`, `final_hours`) carry 1 − x* in the top
/// segment (§5.1 step 5).
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryEq {
    /// p_j, the category's price.
    pub price: f64,
    /// v/p_j, the wage in units of the category (SSRN eq 12).
    pub real_wage: f64,
    /// L_j* = H_j + M_j/γ(x*), the wage-equivalent task cost (main.tex eq effective-hours,
    /// :450). Not a count of hours.
    pub l_star: f64,
    /// L̄_j, the all-human method's hours.
    pub l_bar: f64,
    /// H_j, hours at human tasks per unit.
    pub human: f64,
    /// M_j, machine services per unit.
    pub machine: f64,
    /// λ̃_j = H_j + M_j·λ̃_m, total hours per unit, price side (SSRN eq 4 with the machine
    /// row scaled by u).
    pub lambda_tilde: f64,
    /// b̃_j = b_j + M_j·b̃_m, total land per unit, price side.
    pub b_tilde: f64,
    /// λ̃_j^q = H_j + M_j·λδ/(1 − aδ), total hours per unit, clearing side (§4.3).
    pub lambda_tilde_q: f64,
    /// b̃_j^q = b_j + M_j·bδ/(1 − aδ), total land per unit, clearing side.
    pub b_tilde_q: f64,
    /// y_j = z_j·Y, the category's output.
    pub output: f64,
    /// y_j·H_j, hours at the category's final tasks.
    pub final_hours: f64,
    /// y_j·M_j, machine services used in the category.
    pub machine_services: f64,
    /// z_j·p_j/P_s, the category's expenditure share.
    pub share: f64,
    /// 1/(L̄_j + b_j/v), the lower end of the purchasing-power pair (SSRN eq 13).
    pub wage_floor: f64,
    /// v/b̃_j, the upper end of the pair, when b̃_j > 0 (SSRN eq 13).
    pub wage_ceiling: Option<f64>,
    /// φ_w,j = v·λ̃_j/p_j, labour's share of the price, when u = 1 (three-taxes T1).
    pub phi_w: Option<f64>,
    /// φ_r,j = b̃_j/p_j, rent's share of the price, when u = 1.
    pub phi_r: Option<f64>,
}

/// How far a unit-1b solution is from the equations it must satisfy (docs/unit-1b.md §6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Residuals1b {
    /// |N_a − n_S(x*)|, in hours, as 1a's [`Residuals::labor`](crate::Residuals::labor).
    pub labor: f64,
    /// |Y·P_s − I|/I.
    pub income: f64,
    /// |B_d·Y + b·δ·K − T|/T: land clearing.
    pub land: f64,
    /// |K − Y·M_s − a·δ·K|/K: machine-service clearing; 0 when K = 0 (no basket task below
    /// x*, where both sides are exactly 0).
    pub services: f64,
    /// |p_m − u(a·p_m + λ·v + b)|/p_m: the user-cost price equation.
    pub user_cost: f64,
    /// The largest |p_j − (v·L_j* + b_j)|/p_j over the categories: the fork identity in
    /// main.tex's form (eq composites, :459).
    pub fork: f64,
    /// The largest |p_j − (v·λ̃_j + b̃_j)|/p_j: the fork identity in SSRN's form (eq 12).
    pub totals: f64,
    /// |Σ_j p_j·y_j − I|/I: income as expenditure on the categories (SSRN App. C, p.30).
    pub expenditure: f64,
}

/// The interior equilibrium of unit 1b (docs/unit-1b.md §4.4 and §6), in r = 1 units.
///
/// The aggregates carry unit 1a's [`Eq1a`](crate::Eq1a) names where they mean the same
/// thing, and the same precision notes apply: 1a's `j_star` is `m_s` here, 1a's `p` is the
/// first category's `price`, and 1a's cost-system rows are the categories' `lambda_tilde` and
/// `b_tilde` with the machine row's `lambda_tilde_machine` and `b_tilde_machine`.
///
/// **Interior edges.** Only the top segment carries its offset from x*
/// ([`one_minus_x_star`](Eq1b::one_minus_x_star)). Within a distance d of an interior edge,
/// an output made mostly of the sliver between x* and the edge is good to about
/// 2^-53·x*/d relative for hours (H_j) and 2^-53·(x* + 2·J(x*)/γ(x*))/d for machine
/// services (M_j), and so are the outputs built from them, K, M_s and interest included
/// when all machine use is in the sliver. Prices, v, x*, P_s, Y and N_a keep full
/// precision (docs/unit-1b.md §5.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Eq1b {
    /// x*, as [`Eq1a::x_star`](crate::Eq1a::x_star).
    pub x_star: f64,
    /// 1 − x*, to full relative precision, as [`Eq1a::one_minus_x_star`](crate::Eq1a).
    pub one_minus_x_star: f64,
    /// γ(x*) = w/p_m.
    pub gamma_star: f64,
    /// M_s = Σ z_j·M_j at x*, machine services per basket (1a's J(x*)).
    pub m_s: f64,
    /// u = (ρ + δ)(1 + ρ)^(J_b − 1).
    pub u: f64,
    /// v = w/r.
    pub v: f64,
    /// p_m, the machine-service price.
    pub p_m: f64,
    /// V_m, the cost of building one machine.
    pub v_m: f64,
    /// P_s, the basket's price.
    pub p_s: f64,
    /// Y, baskets.
    pub y: f64,
    /// K, installed machines.
    pub k: f64,
    /// Y·H_s, hours at final tasks, with the carried 1 − x*.
    pub final_hours: f64,
    /// λδK, hours building machines.
    pub machine_hours: f64,
    /// N_a = final_hours + machine_hours.
    pub n_a: f64,
    /// N_a/N, capped at 1.
    pub participation: f64,
    /// I = v·N_a + T + interest.
    pub income: f64,
    /// ρ·W_K, the machine sector's net cash.
    pub interest: f64,
    /// v·N_a/I.
    pub labor_share: f64,
    /// interest/I.
    pub capital_share: f64,
    /// v/P_s, the wage in baskets.
    pub real_wage: f64,
    /// N·P_s.
    pub support_cost: f64,
    /// N + v·N_a/P_s.
    pub worker_baskets: f64,
    /// (T + interest)/P_s − N.
    pub provider_baskets: f64,
    /// provider_baskets > 0.
    pub funded: bool,
    /// n_S(1) > n_D(1) and T > N·P_s(1), SSRN eq 25 as written.
    pub lemma_b1: bool,
    /// φ_w = λγ(x*)/(1 − a), labour's share of p_m, when u = 1.
    pub phi_w: Option<f64>,
    /// φ_r = 1 − φ_w, when u = 1.
    pub phi_r: Option<f64>,
    /// λ̃_m = uλ/(1 − ua), the machine row's total hours, price side.
    pub lambda_tilde_machine: f64,
    /// b̃_m = ub/(1 − ua), the machine row's total land, price side.
    pub b_tilde_machine: f64,
    /// H_s = Σ z_j·H_j, hours at final tasks per basket, with the carried 1 − x*.
    pub h_s: f64,
    /// B_d = Σ z_j·b_j, direct land per basket.
    pub b_d: f64,
    /// L_s* = Σ z_j·L_j*, so P_s = v·L_s* + B_d.
    pub l_star_s: f64,
    /// L_s = Σ z_j·λ̃_j, so P_s = v·L_s + B_s (SSRN eq 7 and §5).
    pub l_s: f64,
    /// B_s = Σ z_j·b̃_j.
    pub b_s: f64,
    /// L_s^q = Σ z_j·λ̃_j^q, so n_D = T·L_s^q/B_s^q (SSRN eq 11).
    pub l_s_q: f64,
    /// B_s^q = Σ z_j·b̃_j^q, so Y = T/B_s^q.
    pub b_s_q: f64,
    /// v/B_s, the basket's rent ceiling: v/P_s ≤ v/B_s (SSRN p.14; check_macro C3).
    pub rent_ceiling: f64,
    /// Whether some basket category has tasks on the segment holding x* (e_s ≤ x* < e_{s+1},
    /// the top segment for x* = 1). When false, x* lies in a gap: no produced task is at
    /// parity, and labour clearing, not a task, sets w/p_m = γ(x*) (docs/unit-1b.md §2.3).
    pub margin_active: bool,
    /// Every category, in order.
    pub categories: Vec<CategoryEq>,
    /// The residuals of docs/unit-1b.md §6.
    pub residuals: Residuals1b,
    /// Bisection steps taken.
    pub bisection_steps: u32,
}

/// One reported output of a unit-1b solve, as [`Eq1b::outputs`] lists it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output1b {
    /// A number.
    Float(f64),
    /// A number that is not always defined: φ when u ≠ 1, and v/b̃_j when b̃_j = 0.
    Optional(Option<f64>),
    /// A flag.
    Flag(bool),
    /// A count.
    Count(u32),
}

/// The key of an output: an economy-level name, or a category's index and field name,
/// printed `cat<j>.<name>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputKey {
    /// The category, or `None` for an economy-level output.
    pub category: Option<usize>,
    /// The field's name.
    pub name: &'static str,
}

impl fmt::Display for OutputKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.category {
            None => write!(f, "{}", self.name),
            Some(index) => write!(f, "cat{index}.{}", self.name),
        }
    }
}

impl Eq1b {
    /// Every output, economy-level first in field order (1a's names in 1a's order, then 1b's
    /// basket fields and residuals), then each category's in field order.
    ///
    /// This is the one list of outputs: [`CategoryEconomy::solve`] checks every number in it
    /// for finiteness.
    pub fn outputs(&self) -> Vec<(OutputKey, Output1b)> {
        use Output1b::{Count, Flag, Float, Optional};
        let top = |name| OutputKey {
            category: None,
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
            (top("p_m"), Float(self.p_m)),
            (top("v_m"), Float(self.v_m)),
            (top("p_s"), Float(self.p_s)),
            (top("y"), Float(self.y)),
            (top("k"), Float(self.k)),
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
            (
                top("lambda_tilde_machine"),
                Float(self.lambda_tilde_machine),
            ),
            (top("b_tilde_machine"), Float(self.b_tilde_machine)),
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
        ];
        for (index, c) in self.categories.iter().enumerate() {
            let key = |name| OutputKey {
                category: Some(index),
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
            ]);
        }
        list
    }
}

/// The error for the first number in [`Eq1b::outputs`] that is NaN or infinite.
fn first_non_finite(eq: &Eq1b) -> Option<SolveError> {
    eq.outputs()
        .into_iter()
        .find_map(|(key, output)| match output {
            Output1b::Float(v) | Output1b::Optional(Some(v)) if !v.is_finite() => {
                Some(match key.category {
                    None => SolveError::NonFinite { what: key.name },
                    Some(category) => SolveError::NonFiniteInCategory {
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

    /// An `Eq1b` with two categories whose numbers are 1, 2, 3, ... in output order, with the
    /// `nan_at`-th replaced by NaN (none when `nan_at` is 0). A struct literal must name every
    /// field, so a field added to `Eq1b` or `CategoryEq` does not compile here until it is
    /// given a number.
    fn numbered(nan_at: usize) -> Eq1b {
        let mut next = 0;
        let mut n = || {
            next += 1;
            if next == nan_at {
                f64::NAN
            } else {
                next as f64
            }
        };
        let mut eq = Eq1b {
            x_star: n(),
            one_minus_x_star: n(),
            gamma_star: n(),
            m_s: n(),
            u: n(),
            v: n(),
            p_m: n(),
            v_m: n(),
            p_s: n(),
            y: n(),
            k: n(),
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
            lambda_tilde_machine: n(),
            b_tilde_machine: n(),
            h_s: n(),
            b_d: n(),
            l_star_s: n(),
            l_s: n(),
            b_s: n(),
            l_s_q: n(),
            b_s_q: n(),
            rent_ceiling: n(),
            margin_active: true,
            categories: Vec::new(),
            residuals: Residuals1b {
                labor: n(),
                income: n(),
                land: n(),
                services: n(),
                user_cost: n(),
                fork: n(),
                totals: n(),
                expenditure: n(),
            },
            bisection_steps: 7,
        };
        for _ in 0..2 {
            eq.categories.push(CategoryEq {
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
            });
        }
        eq
    }

    fn numbers(eq: &Eq1b) -> Vec<(OutputKey, f64)> {
        eq.outputs()
            .into_iter()
            .filter_map(|(key, output)| match output {
                Output1b::Float(v) | Output1b::Optional(Some(v)) => Some((key, v)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn outputs_list_every_field_once() {
        // Every number of Eq1b appears in outputs() exactly once, in field order, and every
        // key is distinct.
        let eq = numbered(0);
        let listed = numbers(&eq);
        let values: Vec<f64> = listed.iter().map(|(_, v)| *v).collect();
        let want: Vec<f64> = (1..=listed.len()).map(|i| i as f64).collect();
        assert_eq!(values, want);
        assert_eq!(listed.len(), 43 + 2 * 18);
        let mut keys: Vec<String> = eq.outputs().iter().map(|(k, _)| k.to_string()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), eq.outputs().len());
        assert!(keys.contains(&"cat1.wage_ceiling".to_string()));
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
            ]
        );
    }

    #[test]
    fn every_output_is_checked_for_finiteness() {
        assert_eq!(first_non_finite(&numbered(0)), None);
        let keys: Vec<OutputKey> = numbers(&numbered(0)).iter().map(|(k, _)| *k).collect();
        for (i, key) in keys.iter().enumerate() {
            let want = match key.category {
                None => SolveError::NonFinite { what: key.name },
                Some(category) => SolveError::NonFiniteInCategory {
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
        eq.categories[1].wage_ceiling = None;
        eq.categories[1].phi_r = Some(f64::INFINITY);
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFiniteInCategory {
                category: 1,
                what: "phi_r"
            })
        );
        assert_eq!(
            SolveError::NonFiniteInCategory {
                category: 1,
                what: "phi_r"
            }
            .to_string(),
            "phi_r of category 1 is not finite"
        );
    }

    #[test]
    fn a_nan_residual_is_not_lost_in_the_maximum() {
        assert!(worse(0.0, f64::NAN).is_nan());
        assert!(worse(f64::NAN, 1.0).is_nan());
        assert_eq!(worse(1e-16, 3e-16), 3e-16);
        assert_eq!(worse(3e-16, 1e-16), 3e-16);
    }
}
