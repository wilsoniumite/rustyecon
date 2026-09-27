//! Unit 1c's machine block on its own (docs/unit-1c.md §2.2-2.4, §4.0, §4.2-4.3 and §4.9):
//! machine types with an operating recipe and a build recipe each, over machine services,
//! labour and land; their user costs; the Leontief totals on the price side (at the user
//! costs u) and the clearing side (at the depreciation rates δ); the two-recipe system at a
//! given margin; and the envelope of the cheapest task type along the task line.
//!
//! Every price is in units of the land rent r = 1, so v = w/r.

use rustyecon_core::num;

use crate::closure::ClosureError;
use crate::leontief::{identity_minus, Factors};
use crate::params::{self, user_cost, wealth_factor, ParamError};

/// A recipe per unit of a machine type's service (operating) or capacity (build)
/// (docs/unit-1c.md §2.2 and §3.1).
///
/// Machine recipes use machine services, labour and land, not categories (§2.3).
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    /// a_kl: units of machine type l's service, one entry per machine type, each in
    /// [0, [`SCALE_CEIL`](crate::SCALE_CEIL)].
    pub machines: Vec<f64>,
    /// λ: hours. In [0, [`SCALE_CEIL`](crate::SCALE_CEIL)].
    pub labor: f64,
    /// b: land services. 0 or scale.
    pub land: f64,
}

impl Recipe {
    /// The zero recipe over `types` machine types.
    pub fn zero(types: usize) -> Recipe {
        Recipe {
            machines: vec![0.0; types],
            labor: 0.0,
            land: 0.0,
        }
    }
}

/// One machine type (docs/unit-1c.md §3.1).
///
/// Its service costs p_k = O_k + u_k·V_k: the operating recipe O_k, paid in the period, plus
/// the user cost u_k = (ρ + δ_k)(1 + ρ)^(J_k − 1) of the build recipe V_k per unit of
/// capacity (check_dynamics R1-R6). One unit of capacity delivers one unit of service per
/// period.
#[derive(Clone, Debug, PartialEq)]
pub struct MachineType {
    /// θ_k: task units per unit of the type's service, relative to the line's γ (γ_Mk =
    /// θ_k/γ). 0 for a type that does no tasks and only supplies other machines; else scale.
    pub task_efficiency: f64,
    /// The operating recipe, per unit of service, paid in the period.
    pub operating: Recipe,
    /// The build recipe, per unit of capacity built.
    pub build: Recipe,
    /// δ_k, depreciation per period. [`SCALE_FLOOR`](crate::SCALE_FLOOR) ≤ δ_k ≤ 1.
    pub delta: f64,
    /// J_k, periods from the start of a build to its first service. J_k ≥ 1.
    pub build_lag: u32,
}

/// The machine types and the interest rate, validated, with every quantity that does not
/// depend on the task line (docs/unit-1c.md §4.9 and §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct MachineBlock {
    rho: f64,
    types: Vec<MachineType>,
    user_cost: Vec<f64>,
    wealth_factor: Vec<f64>,
    /// Â = A^op + U·A^I, row-major K×K.
    a_hat: Vec<f64>,
    /// λ̂ = λ^op + U·λ^I.
    lambda_hat: Vec<f64>,
    /// b̂ = b^op + U·b^I.
    land_hat: Vec<f64>,
    /// The factors of I − Â, diagonal fma(−u_k, a^I_kk, 1 − a^op_kk).
    price_factors: Factors,
    /// λ̃_m, b̃_m and λ̃_m's back-substitution numerators, when I − Â is a nonsingular
    /// M-matrix.
    price_totals: Option<PriceTotals>,
    /// A^q_m = A^op + Δ·A^I, row-major K×K.
    a_q: Vec<f64>,
    /// λ^q = λ^op + Δ·λ^I.
    lambda_q: Vec<f64>,
    /// b^q = b^op + Δ·b^I.
    land_q: Vec<f64>,
    /// The factors of I − A^qᵀ_m, which clear machine services.
    clearing_t: Factors,
    lambda_tilde_q: Vec<f64>,
    b_tilde_q: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct PriceTotals {
    lambda_tilde: Vec<f64>,
    b_tilde: Vec<f64>,
    lambda_numerators: Vec<f64>,
}

/// The totals of the machine block (docs/unit-1c.md §4.2 and §4.5), per type.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockTotals {
    /// u_k = (ρ + δ_k)(1 + ρ)^(J_k − 1).
    pub user_cost: Vec<f64>,
    /// ω_k = (1 + ρ)^(J_k − 1) + δ_k·Σ_{i<J_k−1}(1 + ρ)^i, machine wealth per unit of
    /// installed build cost (check_dynamics L2).
    pub wealth_factor: Vec<f64>,
    /// λ̃_m = (I − Â)⁻¹λ̂, total hours per unit of service, price side; `None` when I − Â
    /// is not a nonsingular M-matrix (no technique is then viable).
    pub lambda_tilde: Option<Vec<f64>>,
    /// b̃_m = (I − Â)⁻¹b̂, total land per unit of service, price side.
    pub b_tilde: Option<Vec<f64>>,
    /// λ̃^q_m = (I − A^q_m)⁻¹λ^q, total hours per unit of service delivered, clearing side.
    pub lambda_tilde_q: Vec<f64>,
    /// b̃^q_m = (I − A^q_m)⁻¹b^q, total land per unit of service delivered, clearing side.
    pub b_tilde_q: Vec<f64>,
    /// The pivots of I − Â.
    pub price_pivots: Vec<f64>,
}

/// The machine block's prices at a margin γ* with technique τ (docs/unit-1c.md §5.1 step 2).
#[derive(Clone, Debug, PartialEq)]
pub struct BlockPrices {
    /// p_k = O_k + u_k·V_k, each type's service price.
    pub prices: Vec<f64>,
    /// O_k, the operating cost per unit of service.
    pub operating: Vec<f64>,
    /// V_k, the build cost per unit of capacity (p_K; 1a's V_m).
    pub build: Vec<f64>,
    /// v = γ*·p_τ/θ_τ, the wage at the margin (SSRN A.1).
    pub v: f64,
    /// π = p_τ/θ_τ, the delivered price of a machine task at efficiency 1.
    pub task_price: f64,
    /// The 2K pivots of the (O, V) system, in index order.
    pub pivots: Vec<f64>,
    /// The least pivot: the first that is not positive, or else the smallest. τ is viable
    /// at γ* when it is positive; for unit 1a's one type it is D = 1 − u(a + λγ*).
    pub d: f64,
}

/// A change of the cheapest task type along the line (docs/unit-1c.md §4.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Switch {
    /// γ_i, where the two types deliver tasks at the same cost.
    pub gamma: f64,
    /// The type below γ_i.
    pub below: usize,
    /// The type from γ_i up: an exact switch point belongs to the piece above.
    pub above: usize,
}

/// The lower envelope of the task types' closure wages over a range of γ
/// (docs/unit-1c.md §4.3 and §5.2): the first technique, then each switch in increasing γ.
#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    /// τ_0, the technique at the bottom of the range.
    pub first: usize,
    /// The switches, γ increasing; each type appears at most once (Lemma 1).
    pub switches: Vec<Switch>,
}

impl Envelope {
    /// The technique at γ: τ_i for γ_i ≤ γ < γ_{i+1}.
    pub fn technique_at(&self, gamma: f64) -> usize {
        let mut technique = self.first;
        for s in &self.switches {
            if gamma >= s.gamma {
                technique = s.above;
            } else {
                break;
            }
        }
        technique
    }

    /// τ_m, the technique at the top of the range.
    pub fn last(&self) -> usize {
        self.switches.last().map_or(self.first, |s| s.above)
    }
}

/// The error for a parameter of machine type `index`.
fn in_type(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "machine type",
        index,
        error: Box::new(error),
    }
}

/// The names of one recipe's parameters in errors.
struct RecipeNames {
    machines: &'static str,
    labor: &'static str,
    land: &'static str,
}

const OPERATING: RecipeNames = RecipeNames {
    machines: "operating.machines",
    labor: "operating.labor",
    land: "operating.land",
};

const BUILD: RecipeNames = RecipeNames {
    machines: "build.machines",
    labor: "build.labor",
    land: "build.land",
};

fn check_recipe(recipe: &mut Recipe, types: usize, names: &RecipeNames) -> Result<(), ParamError> {
    if recipe.machines.len() != types {
        return Err(ParamError::Invalid {
            name: names.machines,
            reason: "a recipe needs one entry per machine type",
        });
    }
    for a in &mut recipe.machines {
        *a = params::nonnegative(names.machines, *a)?;
    }
    recipe.labor = params::nonnegative(names.labor, recipe.labor)?;
    recipe.land = params::zero_or_scale(names.land, recipe.land)?;
    Ok(())
}

/// The reason given when I − (A^op + A^I) is not a nonsingular M-matrix.
const NOT_PRODUCTIVE: &str = "machine recipes are not productive: the spectral radius of the \
                              operating plus build machine inputs must be below 1";

impl MachineBlock {
    /// Validates the machine types and ρ (docs/unit-1c.md §3.2) and computes every x-free
    /// quantity (§5.2).
    ///
    /// Requires ρ in [0, [`SCALE_CEIL`](crate::SCALE_CEIL)]; at least one type, and at least
    /// one with θ > 0; per type, in order: `task_efficiency` 0 or scale, each recipe's
    /// `machines` one entry per type in [0, SCALE_CEIL], `labor` in [0, SCALE_CEIL], `land` 0
    /// or scale, δ in [SCALE_FLOOR, 1], J ≥ 1 and a finite u; then that the recipes are
    /// productive, I − (A^op + A^I) a nonsingular M-matrix (SSRN A.1; for one type with no
    /// operating recipe, 1a's a < 1); then that every type's chain reaches land,
    /// (I − (A^op + A^I))⁻¹(b^op + b^I) > 0 (1a's b > 0). −0.0 is stored as +0.0.
    ///
    /// Viability of the price side is not checked: a block whose I − Â is not a nonsingular
    /// M-matrix is valid, has no price totals, and prices no technique.
    pub fn new(mut types: Vec<MachineType>, rho: f64) -> Result<MachineBlock, ParamError> {
        let rho = params::nonnegative("rho", rho)?;
        let count = types.len();
        if count == 0 {
            return Err(ParamError::Invalid {
                name: "machine types",
                reason: "the economy needs at least one machine type",
            });
        }
        let mut u = Vec::with_capacity(count);
        for (index, t) in types.iter_mut().enumerate() {
            let item = in_type(index);
            t.task_efficiency =
                params::zero_or_scale("task_efficiency", t.task_efficiency).map_err(&item)?;
            check_recipe(&mut t.operating, count, &OPERATING).map_err(&item)?;
            check_recipe(&mut t.build, count, &BUILD).map_err(&item)?;
            params::depreciation("delta", t.delta).map_err(&item)?;
            if t.build_lag == 0 {
                return Err(item(ParamError::BuildLag { value: t.build_lag }));
            }
            let uk = user_cost(rho, t.delta, t.build_lag);
            if !uk.is_finite() {
                return Err(item(ParamError::UserCostNotFinite { u: uk }));
            }
            u.push(uk);
        }
        if !types.iter().any(|t| t.task_efficiency > 0.0) {
            return Err(ParamError::Invalid {
                name: "machine types",
                reason: "at least one machine type must do tasks: task_efficiency > 0",
            });
        }
        // Physical productivity and the chain to land (§3.2): pattern conditions, the same at
        // every u and δ.
        let physical: Vec<f64> = (0..count * count)
            .map(|i| {
                let (k, l) = (i / count, i % count);
                types[k].operating.machines[l] + types[k].build.machines[l]
            })
            .collect();
        let physical = Factors::new(count, identity_minus(count, &physical));
        if !physical.is_m_matrix() {
            return Err(ParamError::Invalid {
                name: "machine types",
                reason: NOT_PRODUCTIVE,
            });
        }
        let land: Vec<f64> = types
            .iter()
            .map(|t| t.operating.land + t.build.land)
            .collect();
        for (index, reach) in physical.solve(&land).into_iter().enumerate() {
            if reach.is_nan() || reach <= 0.0 {
                return Err(in_type(index)(ParamError::Invalid {
                    name: "land",
                    reason: "the type's recipes use no land, directly or through the machine \
                             services they use",
                }));
            }
        }
        let wealth: Vec<f64> = types
            .iter()
            .map(|t| wealth_factor(rho, t.delta, t.build_lag))
            .collect();
        // The price side (§4.2): Â = A^op + U·A^I, with I − Â's diagonal from one fma.
        let mut a_hat = vec![0.0; count * count];
        let mut price_matrix = vec![0.0; count * count];
        for k in 0..count {
            let t = &types[k];
            for l in 0..count {
                a_hat[k * count + l] = t.operating.machines[l] + u[k] * t.build.machines[l];
                price_matrix[k * count + l] = if k == l {
                    num::fma(-u[k], t.build.machines[k], 1.0 - t.operating.machines[k])
                } else {
                    -a_hat[k * count + l]
                };
            }
        }
        let lambda_hat: Vec<f64> = (0..count)
            .map(|k| types[k].operating.labor + u[k] * types[k].build.labor)
            .collect();
        let land_hat: Vec<f64> = (0..count)
            .map(|k| types[k].operating.land + u[k] * types[k].build.land)
            .collect();
        let price_factors = Factors::new(count, price_matrix);
        let price_totals = price_factors.is_m_matrix().then(|| {
            let mut r = lambda_hat.clone();
            price_factors.forward(&mut r);
            let (lambda_numerators, lambda_tilde) = price_factors.back(&r);
            PriceTotals {
                lambda_tilde,
                b_tilde: price_factors.solve(&land_hat),
                lambda_numerators,
            }
        });
        // The clearing side (§4.5): A^q = A^op + Δ·A^I.
        let mut a_q = vec![0.0; count * count];
        let mut clearing = vec![0.0; count * count];
        for k in 0..count {
            let t = &types[k];
            for l in 0..count {
                a_q[k * count + l] = t.operating.machines[l] + t.delta * t.build.machines[l];
                clearing[k * count + l] = if k == l {
                    num::fma(-t.delta, t.build.machines[k], 1.0 - t.operating.machines[k])
                } else {
                    -a_q[k * count + l]
                };
            }
        }
        let lambda_q: Vec<f64> = types
            .iter()
            .map(|t| t.operating.labor + t.delta * t.build.labor)
            .collect();
        let land_q: Vec<f64> = types
            .iter()
            .map(|t| t.operating.land + t.delta * t.build.land)
            .collect();
        let clearing_t = Factors::new(count, crate::leontief::transpose(count, &clearing));
        let clearing = Factors::new(count, clearing);
        // A^q ≤ A^op + A^I (δ ≤ 1), so both are M-matrices once the physical block is; a
        // rounding that says otherwise is reported as the same failure.
        if !clearing.is_m_matrix() || !clearing_t.is_m_matrix() {
            return Err(ParamError::Invalid {
                name: "machine types",
                reason: NOT_PRODUCTIVE,
            });
        }
        let lambda_tilde_q = clearing.solve(&lambda_q);
        let b_tilde_q = clearing.solve(&land_q);
        Ok(MachineBlock {
            rho,
            types,
            user_cost: u,
            wealth_factor: wealth,
            a_hat,
            lambda_hat,
            land_hat,
            price_factors,
            price_totals,
            a_q,
            lambda_q,
            land_q,
            clearing_t,
            lambda_tilde_q,
            b_tilde_q,
        })
    }

    /// The validated machine types.
    pub fn types(&self) -> &[MachineType] {
        &self.types
    }

    /// ρ.
    pub fn rho(&self) -> f64 {
        self.rho
    }

    /// K, the number of machine types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Never true: a block has at least one type.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// u_k per type.
    pub fn user_costs(&self) -> &[f64] {
        &self.user_cost
    }

    /// ω_k per type.
    pub fn wealth_factors(&self) -> &[f64] {
        &self.wealth_factor
    }

    /// Â_kl, the price-side machine coefficients.
    pub fn a_hat(&self, k: usize, l: usize) -> f64 {
        self.a_hat[k * self.len() + l]
    }

    /// λ̂_k.
    pub fn lambda_hat(&self) -> &[f64] {
        &self.lambda_hat
    }

    /// b̂_k.
    pub fn land_hat(&self) -> &[f64] {
        &self.land_hat
    }

    /// A^q_kl, the clearing-side machine coefficients: type l's service per unit of type k's
    /// service delivered, operating plus replacement builds.
    pub fn a_q(&self, k: usize, l: usize) -> f64 {
        self.a_q[k * self.len() + l]
    }

    /// λ^q_k = λ^op_k + δ_k·λ^I_k, hours per unit of service delivered.
    pub fn lambda_q(&self) -> &[f64] {
        &self.lambda_q
    }

    /// b^q_k = b^op_k + δ_k·b^I_k, land per unit of service delivered.
    pub fn land_q(&self) -> &[f64] {
        &self.land_q
    }

    /// The totals of §4.2 and §4.5.
    pub fn totals(&self) -> BlockTotals {
        BlockTotals {
            user_cost: self.user_cost.clone(),
            wealth_factor: self.wealth_factor.clone(),
            lambda_tilde: self.price_totals.as_ref().map(|t| t.lambda_tilde.clone()),
            b_tilde: self.price_totals.as_ref().map(|t| t.b_tilde.clone()),
            lambda_tilde_q: self.lambda_tilde_q.clone(),
            b_tilde_q: self.b_tilde_q.clone(),
            price_pivots: self.price_factors.pivots(),
        }
    }

    /// λ̃_m and b̃_m, when I − Â is a nonsingular M-matrix.
    pub(crate) fn price_totals(&self) -> Option<(&[f64], &[f64])> {
        self.price_totals
            .as_ref()
            .map(|t| (t.lambda_tilde.as_slice(), t.b_tilde.as_slice()))
    }

    /// λ̃^q_m.
    pub(crate) fn lambda_tilde_q(&self) -> &[f64] {
        &self.lambda_tilde_q
    }

    /// b̃^q_m.
    pub(crate) fn b_tilde_q(&self) -> &[f64] {
        &self.b_tilde_q
    }

    /// γλ̃_τ/θ_τ, labour's share of the technique's price, as (n_τ·γ)/(U_ττ·θ_τ) from the
    /// numerator and pivot of λ̃_τ's back substitution: for unit 1a, (λγ)/(1 − a) bit for bit
    /// (docs/unit-1c.md §4.8).
    pub(crate) fn labor_share_of_price(&self, gamma: f64, technique: usize) -> Option<f64> {
        let totals = self.price_totals.as_ref()?;
        let theta = self.types[technique].task_efficiency;
        Some(
            (totals.lambda_numerators[technique] * gamma)
                / (self.price_factors.pivot(technique) * theta),
        )
    }

    /// The (O, V) system of docs/unit-1c.md §5.1 step 2 at γ with technique τ, evaluated as
    /// written: outside the viable set the prices mean nothing, and `d` says so.
    pub(crate) fn prices_at(&self, gamma: f64, technique: usize) -> BlockPrices {
        let k = self.len();
        let n = 2 * k;
        let u = &self.user_cost;
        let theta = self.types[technique].task_efficiency;
        let u_tau = u[technique];
        let mut g = vec![0.0; n * n];
        let mut r = vec![0.0; n];
        for i in 0..k {
            let (op, build) = (&self.types[i].operating, &self.types[i].build);
            for l in 0..k {
                let mut oo = if i == l {
                    1.0 - op.machines[i]
                } else {
                    -op.machines[l]
                };
                let mut ov = -(op.machines[l] * u[l]);
                let mut vo = -build.machines[l];
                let mut vv = if i == l {
                    num::fma(-u[i], build.machines[i], 1.0)
                } else {
                    -(build.machines[l] * u[l])
                };
                if l == technique {
                    oo -= (op.labor * gamma) / theta;
                    ov -= ((u_tau * op.labor) * gamma) / theta;
                    vo -= (build.labor * gamma) / theta;
                    vv -= ((u_tau * build.labor) * gamma) / theta;
                }
                g[i * n + l] = oo;
                g[i * n + k + l] = ov;
                g[(k + i) * n + l] = vo;
                g[(k + i) * n + k + l] = vv;
            }
            r[i] = op.land;
            r[k + i] = build.land;
        }
        let factors = Factors::new(n, g);
        factors.forward(&mut r);
        let (_, z) = factors.back(&r);
        let operating = z[..k].to_vec();
        let build = z[k..].to_vec();
        let prices: Vec<f64> = (0..k).map(|i| operating[i] + u[i] * build[i]).collect();
        let v = (gamma * prices[technique]) / theta;
        let task_price = prices[technique] / theta;
        BlockPrices {
            v,
            task_price,
            pivots: factors.pivots(),
            d: factors.least_pivot(),
            prices,
            operating,
            build,
        }
    }

    /// The (O, V) system at a given wage (docs/unit-1d.md §5.1, a corner's evaluation): §5.1
    /// step 2's matrix without the margin column, so that labour enters the right-hand side at
    /// the wage. Its factors do not depend on the wage, the line or the technique; for one type
    /// with no operating recipe its pivots are 1.0 and 1 − u·a.
    pub(crate) fn wage_system(&self) -> Factors {
        let k = self.len();
        let n = 2 * k;
        let u = &self.user_cost;
        let mut g = vec![0.0; n * n];
        for i in 0..k {
            let (op, build) = (&self.types[i].operating, &self.types[i].build);
            for l in 0..k {
                g[i * n + l] = if i == l {
                    1.0 - op.machines[i]
                } else {
                    -op.machines[l]
                };
                g[i * n + k + l] = -(op.machines[l] * u[l]);
                g[(k + i) * n + l] = -build.machines[l];
                g[(k + i) * n + k + l] = if i == l {
                    num::fma(-u[i], build.machines[i], 1.0)
                } else {
                    -(build.machines[l] * u[l])
                };
            }
        }
        Factors::new(n, g)
    }

    /// The machine block's prices at the wage v, with the factors of
    /// [`MachineBlock::wage_system`] (docs/unit-1d.md §4.2 and §5.1): O = A^op·p + λ^op·v + b^op,
    /// V = A^I·p + λ^I·v + b^I and p_k = O_k + u_k·V_k, so p = v·λ̃ + b̃; π = p_τ/θ_τ for the
    /// technique τ. `d` is the system's least pivot. Evaluated as written: where I − Â is not a
    /// nonsingular M-matrix the prices mean nothing, and `d` says so.
    pub(crate) fn prices_at_wage(
        &self,
        factors: &Factors,
        v: f64,
        technique: usize,
    ) -> BlockPrices {
        let k = self.len();
        let u = &self.user_cost;
        let mut r = vec![0.0; 2 * k];
        for i in 0..k {
            let (op, build) = (&self.types[i].operating, &self.types[i].build);
            r[i] = op.land + op.labor * v;
            r[k + i] = build.land + build.labor * v;
        }
        factors.forward(&mut r);
        let (_, z) = factors.back(&r);
        let operating = z[..k].to_vec();
        let build = z[k..].to_vec();
        let prices: Vec<f64> = (0..k).map(|i| operating[i] + u[i] * build[i]).collect();
        let task_price = prices[technique] / self.types[technique].task_efficiency;
        BlockPrices {
            v,
            task_price,
            pivots: factors.pivots(),
            d: factors.least_pivot(),
            prices,
            operating,
            build,
        }
    }

    /// The machine block's prices at the margin γ* with task type τ doing the machine tasks
    /// (docs/unit-1c.md §4.2 and §4.9): p_k, O_k, V_k and v = γ*·p_τ/θ_τ.
    ///
    /// At one type with no operating recipe and θ = 1 it is unit 1a's
    /// [`closure`](crate::closure) at r = 1 up to association: V = b/D and p = u·V, where
    /// `closure` forms u·b/D. With a zero build recipe it is b/(1 − a − λγ*) at every u
    /// (check_dynamics R4), and in general check_dynamics R1-R3.
    ///
    /// Requires γ* finite and positive and τ a task type (θ_τ > 0). Returns
    /// [`ClosureError::NotViable`] with the least pivot when τ is not viable at γ*, and
    /// [`ClosureError::NonFinite`] if a price overflows.
    pub fn closure(&self, gamma_star: f64, technique: usize) -> Result<BlockPrices, ClosureError> {
        let gamma_star = params::positive("gamma_star", gamma_star)?;
        if technique >= self.len() || self.types[technique].task_efficiency <= 0.0 {
            return Err(ClosureError::Invalid(ParamError::Invalid {
                name: "technique",
                reason: "the technique must be a machine type that does tasks",
            }));
        }
        let prices = self.prices_at(gamma_star, technique);
        if prices.d.is_nan() || prices.d <= 0.0 {
            return Err(ClosureError::NotViable { d: prices.d });
        }
        let named = [("v", prices.v), ("task_price", prices.task_price)];
        for (what, value) in named {
            if !value.is_finite() {
                return Err(ClosureError::NonFinite { what });
            }
        }
        for (what, values) in [
            ("price", &prices.prices),
            ("operating", &prices.operating),
            ("build", &prices.build),
        ] {
            if values.iter().any(|x| !x.is_finite()) {
                return Err(ClosureError::NonFinite { what });
            }
        }
        Ok(prices)
    }

    /// v_t(γ) = γ·b̃_t/(θ_t − γ·λ̃_t), task type t's closure wage (SSRN A.1's closure, one
    /// fixed input), when t does tasks and is viable at γ (θ_t − γλ̃_t > 0).
    pub fn closure_wage(&self, gamma: f64, t: usize) -> Option<f64> {
        let (lt, bt) = self.price_totals()?;
        let theta = self.types[t].task_efficiency;
        let room = theta - gamma * lt[t];
        (theta > 0.0 && room > 0.0).then(|| (gamma * bt[t]) / room)
    }

    /// The lower envelope of the task types' closure wages on [γ_lo, γ_hi]
    /// (docs/unit-1c.md §4.3 and §5.2).
    ///
    /// τ_0 has the least closure wage at γ_lo; a tie goes to the type cheaper just above (the
    /// smaller λ̃_t/θ_t), then to the lower index. From each technique the next is the task
    /// type, not yet visited, whose crossing γ_τl = (b̃_l·θ_τ − b̃_τ·θ_l)/Δ_τl, with
    /// Δ_τl = b̃_l·λ̃_τ − b̃_τ·λ̃_l > 0, is the least in (γ_c, γ_hi] with l viable there; ties
    /// again to the type cheaper just above, then the lower index. When I − Â is not a
    /// nonsingular M-matrix, or no task type is viable at γ_lo, the envelope is the lowest
    /// task type alone.
    pub fn envelope(&self, gamma_lo: f64, gamma_hi: f64) -> Envelope {
        let tasks: Vec<usize> = (0..self.len())
            .filter(|&t| self.types[t].task_efficiency > 0.0)
            .collect();
        let alone = Envelope {
            first: tasks[0],
            switches: Vec::new(),
        };
        let Some((lt, bt)) = self.price_totals() else {
            return alone;
        };
        let theta = |t: usize| self.types[t].task_efficiency;
        // "t is cheaper than s just above a common point": λ̃_t/θ_t < λ̃_s/θ_s.
        let flatter = |t: usize, s: usize| lt[t] * theta(s) < lt[s] * theta(t);
        let mut first: Option<(usize, f64)> = None;
        for &t in &tasks {
            let Some(v) = self.closure_wage(gamma_lo, t) else {
                continue;
            };
            let better = match first {
                None => true,
                Some((s, w)) => v < w || (v == w && flatter(t, s)),
            };
            if better {
                first = Some((t, v));
            }
        }
        let Some((tau0, _)) = first else {
            return alone;
        };
        let mut current = tau0;
        let mut visited = vec![false; self.len()];
        visited[current] = true;
        let mut gamma_c = gamma_lo;
        let mut switches = Vec::new();
        loop {
            let mut next: Option<(usize, f64)> = None;
            for &l in &tasks {
                if visited[l] {
                    continue;
                }
                let delta = bt[l] * lt[current] - bt[current] * lt[l];
                if delta.is_nan() || delta <= 0.0 {
                    continue;
                }
                let crossing = (bt[l] * theta(current) - bt[current] * theta(l)) / delta;
                let counts =
                    gamma_c < crossing && crossing <= gamma_hi && theta(l) - crossing * lt[l] > 0.0;
                if !counts {
                    continue;
                }
                let better = match next {
                    None => true,
                    Some((s, g)) => crossing < g || (crossing == g && flatter(l, s)),
                };
                if better {
                    next = Some((l, crossing));
                }
            }
            let Some((l, crossing)) = next else {
                break;
            };
            switches.push(Switch {
                gamma: crossing,
                below: current,
                above: l,
            });
            visited[l] = true;
            current = l;
            gamma_c = crossing;
        }
        Envelope {
            first: tau0,
            switches,
        }
    }

    /// X = (I − A^qᵀ_m)⁻¹·t: gross machine services when t_k units of type k's service go to
    /// tasks (docs/unit-1c.md §4.5 and §4.9), by forward and back substitution.
    pub fn gross_services(&self, task_services: &[f64]) -> Vec<f64> {
        assert_eq!(
            task_services.len(),
            self.len(),
            "one entry per machine type"
        );
        let mut r = task_services.to_vec();
        self.clearing_t.forward(&mut r);
        self.clearing_t.back(&r).1
    }

    /// Forward substitution and the numerators of back substitution for (I − A^qᵀ_m)X = t,
    /// with the division deferred (§5.1 step 4): X_k = n_k/U_kk, returned as (n, U_kk).
    pub(crate) fn clearing_numerators(&self, task_services: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let mut r = task_services.to_vec();
        self.clearing_t.forward(&mut r);
        (self.clearing_t.back(&r).0, self.clearing_t.pivots())
    }
}
