//! The equilibrium: prices and quantities at a threshold (spec §3.1-3.2), the root
//! solve (spec §4), and the reported outputs (spec §3.4-3.5 and §5).

use std::fmt;

use rustyecon_core::num;

use crate::closure::viability;
use crate::params::{wealth_factor, Economy};
use crate::schedule::Schedule;

/// Left end of the root bracket, x = 1e-12.
///
/// The equilibrium threshold is interior, x* ∈ (0, 1) (SSRN Lemma B.1, p.29), so the
/// bracket starts just inside the task line. The value is laborformal's
/// (`paths/code/macro.py:102` at 31b3482), so [`Regime::NoInteriorAtZero`] is decided
/// at the same point as in the reference implementation.
pub const BRACKET_LO: f64 = 1e-12;

/// Right end of the root bracket, x = 1: the end of the task line, where SSRN Lemma B.1
/// states its conditions (eq 25) and viability is checked.
pub const BRACKET_HI: f64 = 1.0;

/// Cap on bisection steps, a guard that valid input never reaches.
///
/// Bisection stops when the floating midpoint equals an endpoint, that is when lo and
/// hi are adjacent doubles. Doubles in [1e-12, 1] are never closer than 2^-92 (the
/// spacing just above 2^-40 < 1e-12), and each step halves a bracket that starts
/// narrower than 1, so at most 92 steps (plus one or two for rounding of the midpoint)
/// are needed wherever the root lies. The cap leaves a margin above that bound; reaching
/// it means an invariant broke, and [`Economy::solve`] then returns
/// [`SolveError::NoConvergence`] rather than looping.
pub const MAX_BISECTION_STEPS: u32 = 128;

/// The safety net on an interior result: [`Economy::solve`] returns
/// [`SolveError::LaborNotCleared`] rather than [`Regime::Interior`] when the labour
/// residual |N_a − n_S(x*)| ([`Residuals::labor`]) exceeds this fraction of N_a.
///
/// Bisection finds where the f64 excess demand f = n_D − n_S changes sign between
/// adjacent doubles. For a continuous f (the [`Schedule`] contract) that is a root, and the
/// reported equilibrium clears the labour market to the rounding in f, a few ulps of N_a,
/// plus the change in n_S across one double of x. That change is about max(k, 1)·2^-52
/// relative for a [`PowerSchedule`](crate::PowerSchedule), under 2.3e-13 at
/// [`CURVATURE_CEIL`](crate::CURVATURE_CEIL), times n_S's sensitivity to γ, which is of
/// order 1 away from the viability edge. Over 109,281 random interior economies with
/// every parameter spread over several decades and k up to the ceiling, the largest
/// residual was 1.0e-13 of N_a. A sign change that is not a root, a jump in f, leaves a
/// residual of the jump's size: at k = 1e20 (above the ceiling) γ jumps from 0.2 to 1
/// across the last double below 1, and the residual was 0.23 of N_a.
///
/// The net is 1e-9, four orders of magnitude above the largest residual of a root away from
/// the viability edge, so no economy of the gate or the paper comes near it. It is a
/// safety net, not a tolerance: it never changes a result, it only refuses one whose labour
/// market does not clear to a part in 10^9. Two things trip it:
///
/// - a jump in f, from a custom [`Schedule`] that breaks the contract (G8 constructs one)
///   or from a failure mode not yet known;
/// - the viability edge, where n_S's sensitivity to γ grows like 1/D, so that n_S can move
///   by more than 1e-9 of itself between adjacent doubles of x. The equilibrium is then
///   not resolved in f64 to that level, and neither are the prices. In a probe of 188,336
///   interior economies drawn close to the edge (D(1) log-uniform in [1e-12, 0.5]), the
///   net refused 3,419, all with D(x*) < 5e-5, and in each n_S moved across the final
///   bracket by at least twice the residual.
pub const LABOR_RESIDUAL_NET: f64 = 1e-9;

/// Prices and quantities at a candidate threshold x (spec §3.1-3.2), with r = 1.
///
/// Machines perform the tasks in [0, x) and people those in (x, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// The threshold x.
    pub x: f64,
    /// γ(x).
    pub gamma: f64,
    /// J(x) = ∫₀ˣ γ, the capability integral (SSRN eq 21).
    pub j: f64,
    /// D(x) = 1 − u(a + λγ(x)), the viability denominator.
    pub d: f64,
    /// V_m = b/D, the cost of building one machine: a·p_m + λ·v + b (SSRN A.4, p.27).
    pub v_m: f64,
    /// p_m = u·V_m, the machine-service price (main.tex's c) (SSRN A.4, p.28).
    pub p_m: f64,
    /// v = γ(x)·p_m, the task margin (SSRN eq 21).
    pub v: f64,
    /// p = v(1 − x) + p_m·J(x), the good's price (SSRN eq 22).
    pub p: f64,
    /// P_s = p + h, the basket's price (SSRN eq 22).
    pub p_s: f64,
    /// Y = T/(h + b·δ·J/(1 − aδ)), baskets, from land clearing (SSRN eq 23 at δ = 1).
    pub y: f64,
    /// K = Y·J/(1 − aδ), installed machines, equal to machine services per period.
    pub k: f64,
    /// Y(1 − x), hours at final tasks.
    pub final_hours: f64,
    /// λ·δ·K, hours building machines.
    pub machine_hours: f64,
    /// n_D = final_hours + machine_hours, labour demand (SSRN eq 24).
    pub n_d: f64,
    /// n_S = N·F(ln(1 + v/P_s)), labour supply (SSRN eq 10 and 24).
    pub n_s: f64,
}

impl Point {
    /// f(x) = n_D − n_S, the function whose root is x*.
    pub fn excess_demand(&self) -> f64 {
        self.n_d - self.n_s
    }
}

/// The cost-system view at u = 1 (spec §3.5; SSRN eq 2-4, p.8, and the display on
/// p.29). Rows are ordered (good, machine services, space).
///
/// With A = [[0, J, 0], [0, a, 0], [0, 0, 0]], λ = (1 − x, λ, 0), b = (0, b, 1) and
/// z = (1, 0, h): λ̃ = (I − A)⁻¹λ, b̃ = (I − A)⁻¹b, L_s = zᵀλ̃, B_s = zᵀb̃, and
/// P_s = v·L_s + B_s. Y = T/B_s and n_D = T·L_s/B_s (SSRN eq 11) hold when δ = 1
/// as well; at u = 1 with δ < 1 only the price side is this system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CostSystem {
    /// λ̃, total hours per unit of each row's output. The good's row uses
    /// [`Eq1a::one_minus_x_star`] for 1 − x.
    pub lambda_tilde: [f64; 3],
    /// b̃, total land services per unit of each row's output.
    pub b_tilde: [f64; 3],
    /// L_s = 1 − x + λJ/(1 − a), total hours per basket.
    pub l_s: f64,
    /// B_s = h + bJ/(1 − a), total land services per basket.
    pub b_s: f64,
}

/// How far the solution is from the equations it must satisfy (spec §4 step 5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Residuals {
    /// |N_a − n_S(x*)|, in hours: labour demand at the reported equilibrium (N_a, with
    /// the carried 1 − x*) against labour supply at the reported prices (at the double
    /// x*). An `Interior` result has it at most [`LABOR_RESIDUAL_NET`]·N_a.
    ///
    /// It is not |f| at the double x*: near full automation x* can round to 1.0, where
    /// n_D(1.0) misses all of Y(1 − x*) and |f| is N_a itself (G3 at η = 1e-20).
    pub labor: f64,
    /// |Y·P_s − I|/I: the income identity, which is checked, not imposed.
    pub income: f64,
    /// |h·Y + b·δ·K − T|/T: land clearing.
    pub land: f64,
    /// |K − Y·J − a·δ·K|/K: machine-service clearing.
    pub services: f64,
    /// |p_m − u(a·p_m + λ·v + b)|/p_m: the user-cost price equation.
    pub user_cost: f64,
}

/// The interior equilibrium of unit 1a (spec §5), in r = 1 units.
///
/// **Precision.** Away from the ends of the task line every field is within a few ulps of
/// the exact equilibrium of the f64 inputs (the gate compares at 1e-12 relative).
///
/// - Near full automation, x* is a double and the doubles near 1 are 1.1e-16 apart, so
///   1.0 − x_star is only good to about 1e-16 absolute. The solve therefore carries
///   1 − x* separately, in [`one_minus_x_star`](Eq1a::one_minus_x_star), and every output
///   proportional to 1 − x* is computed from it: `final_hours`, `n_a`, `participation`,
///   `labor_share`, `worker_baskets`, the good's row of λ̃ and `l_s`. They keep their
///   relative precision however close x* is to 1.
/// - Near x* = 0 the outputs proportional to x* (`j_star`, `k`, `machine_hours`,
///   `interest`, `capital_share`) are limited by the conditioning of the root: f is
///   known to about 2^-53·n_D, so x* is known to about 2^-53·n_D/|f'| absolute. That is
///   beyond 1e-12 relative once x* ≲ 1e-6, for any representation of x.
/// - Near the viability edge the prices scale as 1/D. D is computed to about
///   2^-53·(1 − u·a)/D relative (see `viability` in `closure.rs`), and u's own error,
///   which grows with the build lag (see [`user_cost`](crate::user_cost)), is amplified
///   by u(a + λγ)/D. x* is a double, so γ(x*) is off by up to about max(k, 1)·2^-53
///   relative, and D carries that times uλγ/D: the outputs are within about
///   max(k, 1)·2^-53·(1 + uλγ/D) relative of the exact equilibrium. Near the edge the
///   labour market clears only as well as the doubles of x resolve n_S: an `Interior`
///   result with D(x*) within about 1e-5 can carry a labour residual up to
///   [`LABOR_RESIDUAL_NET`] of N_a, and closer still the solve refuses
///   ([`SolveError::LaborNotCleared`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Eq1a {
    /// x*, the equilibrium threshold: of the two adjacent doubles between which the f64
    /// excess demand n_D − n_S changes sign, the one with the smaller |n_D − n_S|.
    ///
    /// It lies in [[`BRACKET_LO`], [`BRACKET_HI`]], and it can equal either end. When the
    /// true root lies within half an ulp of 1 (1 − x* < 2^-54 ≈ 5.6e-17), 1.0 is the
    /// nearest double and x_star is exactly 1.0. The economy is still interior (f(1) < 0),
    /// and [`one_minus_x_star`](Eq1a::one_minus_x_star) is positive.
    pub x_star: f64,
    /// 1 − x*, to full relative precision even where x* rounds to 1.
    ///
    /// Bisection ends with adjacent doubles lo < hi and f(lo) > 0 > f(hi). The root of
    /// the straight line through (lo, f(lo)) and (hi, f(hi)) is hi − θ·(hi − lo) with
    /// θ = −f(hi)/(f(lo) − f(hi)) ∈ [0, 1], so 1 − x* = (1 − hi) + θ·(hi − lo): 1 − hi is
    /// exact for hi ≥ 1/2 and hi − lo is exact, so nothing cancels. The value lies in
    /// [1 − hi, 1 − lo] up to rounding, so it differs from the double 1.0 − x_star by at
    /// most the spacing of doubles at x* plus that at 1 − x*. Where f's rounding noise
    /// swamps its slope over one spacing, it is no worse than 1.0 − x_star. If bisection
    /// lands on an exact zero of f, it is 1.0 − x_star.
    pub one_minus_x_star: f64,
    /// γ(x*).
    pub gamma_star: f64,
    /// J(x*).
    pub j_star: f64,
    /// u = (ρ + δ)(1 + ρ)^(J_b − 1).
    pub u: f64,
    /// v = w/r.
    pub v: f64,
    /// p_m, the machine-service price.
    pub p_m: f64,
    /// V_m, the cost of building one machine.
    pub v_m: f64,
    /// p, the good's price.
    pub p: f64,
    /// P_s, the basket's price.
    pub p_s: f64,
    /// Y, baskets, equal to the good's output.
    pub y: f64,
    /// K, installed machines.
    pub k: f64,
    /// Hours at final tasks, Y·(1 − x*), with 1 − x* = `one_minus_x_star`.
    pub final_hours: f64,
    /// Hours building machines, λδK.
    pub machine_hours: f64,
    /// N_a = final_hours + machine_hours = n_D at the root, hours worked.
    pub n_a: f64,
    /// N_a/N, capped at 1. Where supply is saturated at the root (F = 1, so n_S = N),
    /// n_D(x*) can exceed N by a few ulps; the cap keeps the share a share. `n_a` itself
    /// is not capped.
    pub participation: f64,
    /// I = v·N_a + T + interest (SSRN eq 15 when interest = 0; macro.py:109).
    pub income: f64,
    /// (u − δ)·V_m·K, the machine sector's net cash: ρ·V_m·K when J_b = 1, and ρ·W_K
    /// in general (check_dynamics.py L1-L2, :155-169 at 31b3482). It is computed as
    /// ρ·W_K, which has no cancellation. It goes to the provider.
    pub interest: f64,
    /// v·N_a/I.
    pub labor_share: f64,
    /// interest/I.
    pub capital_share: f64,
    /// v/P_s = w/P_s, the wage in baskets.
    pub real_wage: f64,
    /// N·P_s, the cost of one basket for every potential worker.
    pub support_cost: f64,
    /// N + v·N_a/P_s: support plus wage baskets.
    pub worker_baskets: f64,
    /// (T + interest)/P_s − N, the provider's own baskets.
    pub provider_baskets: f64,
    /// provider_baskets > 0: the provider can fund support, T + interest > N·P_s
    /// (macro.py:113; spec §3.4).
    ///
    /// It is computed from `provider_baskets`, so the two never disagree. The comparison
    /// T + interest > N·P_s rounds differently in f64: within a rounding of the boundary it
    /// can read false while `provider_baskets` is 2^-51 > 0 (a probe of 2039 boundaries in
    /// N found 366 such points within four ulps of them). There the sign is not decidable
    /// in f64, as for the regimes. At an exact tie, N·P_s = T + interest in f64 and
    /// `provider_baskets` = 0, `funded` is false (G8).
    pub funded: bool,
    /// n_S(1) > n_D(1) and T > N·P_s(1), SSRN eq 25.
    pub lemma_b1: bool,
    /// φ_w = λγ(x*)/(1 − a), labour's share of p_m (three-taxes), when u = 1.
    pub phi_w: Option<f64>,
    /// φ_r = 1 − φ_w, rent's share of p_m, when u = 1.
    pub phi_r: Option<f64>,
    /// The cost-system view, when u = 1.
    pub cost_system: Option<CostSystem>,
    /// The residuals of spec §4 step 5.
    pub residuals: Residuals,
    /// Bisection steps taken, always below [`MAX_BISECTION_STEPS`].
    pub bisection_steps: u32,
}

/// One reported output of an interior solve, as [`Eq1a::outputs`] lists it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    /// A number.
    Float(f64),
    /// A number that is reported only when u = 1, and `None` otherwise.
    FlowOnly(Option<f64>),
    /// A flag.
    Flag(bool),
    /// A count.
    Count(u32),
}

impl Eq1a {
    /// Every output, with the key the dump prints it under, in the dump's order (the
    /// field order, with each cost-system and residual component under its own key).
    ///
    /// This is the one list of outputs: the dump prints it, and [`Economy::solve`] checks
    /// every number in it for finiteness.
    pub fn outputs(&self) -> Vec<(&'static str, Output)> {
        use Output::{Count, Flag, Float, FlowOnly};
        let cs = self.cost_system;
        let r = &self.residuals;
        vec![
            ("x_star", Float(self.x_star)),
            ("one_minus_x_star", Float(self.one_minus_x_star)),
            ("gamma_star", Float(self.gamma_star)),
            ("j_star", Float(self.j_star)),
            ("u", Float(self.u)),
            ("v", Float(self.v)),
            ("p_m", Float(self.p_m)),
            ("v_m", Float(self.v_m)),
            ("p", Float(self.p)),
            ("p_s", Float(self.p_s)),
            ("y", Float(self.y)),
            ("k", Float(self.k)),
            ("final_hours", Float(self.final_hours)),
            ("machine_hours", Float(self.machine_hours)),
            ("n_a", Float(self.n_a)),
            ("participation", Float(self.participation)),
            ("income", Float(self.income)),
            ("interest", Float(self.interest)),
            ("labor_share", Float(self.labor_share)),
            ("capital_share", Float(self.capital_share)),
            ("real_wage", Float(self.real_wage)),
            ("support_cost", Float(self.support_cost)),
            ("worker_baskets", Float(self.worker_baskets)),
            ("provider_baskets", Float(self.provider_baskets)),
            ("funded", Flag(self.funded)),
            ("lemma_b1", Flag(self.lemma_b1)),
            ("phi_w", FlowOnly(self.phi_w)),
            ("phi_r", FlowOnly(self.phi_r)),
            ("lambda_tilde_good", FlowOnly(cs.map(|c| c.lambda_tilde[0]))),
            (
                "lambda_tilde_machine",
                FlowOnly(cs.map(|c| c.lambda_tilde[1])),
            ),
            (
                "lambda_tilde_space",
                FlowOnly(cs.map(|c| c.lambda_tilde[2])),
            ),
            ("b_tilde_good", FlowOnly(cs.map(|c| c.b_tilde[0]))),
            ("b_tilde_machine", FlowOnly(cs.map(|c| c.b_tilde[1]))),
            ("b_tilde_space", FlowOnly(cs.map(|c| c.b_tilde[2]))),
            ("l_s", FlowOnly(cs.map(|c| c.l_s))),
            ("b_s", FlowOnly(cs.map(|c| c.b_s))),
            ("res_labor", Float(r.labor)),
            ("res_income", Float(r.income)),
            ("res_land", Float(r.land)),
            ("res_services", Float(r.services)),
            ("res_user_cost", Float(r.user_cost)),
            ("bisection_steps", Count(self.bisection_steps)),
        ]
    }
}

/// The outcome of a solve (spec §5).
///
/// The regime is decided on the f64 evaluation of D(1), f(1) and f([`BRACKET_LO`]), with
/// the spec's convention at exact equality: D(1) = 0 is `NotViable`, f(1) = 0 is
/// `BoundaryNoMargin` and f(lo) = 0 is `NoInteriorAtZero`. Where one of these is within a
/// few ulps of zero (relative to 1 for D, to n_D for f), the f64 sign can differ from the
/// exact sign on the same inputs, and so can another implementation's; the regime is
/// then not decidable in f64, and a comparison of regimes must exclude such cases.
#[derive(Clone, Debug, PartialEq)]
pub enum Regime {
    /// A unique interior threshold with n_D = n_S. In exact arithmetic x* ∈ (0, 1); the
    /// reported double can round to 1.0 (see [`Eq1a::x_star`]).
    Interior(Box<Eq1a>),
    /// f(1) ≥ 0: labour holds no machine-contestable task, x* would be 1 and workers
    /// would only build machines. SSRN §3.1's boundary case, which unit 1d solves.
    BoundaryNoMargin {
        /// f(1) = n_D(1) − n_S(1).
        f_at_1: f64,
    },
    /// D(1) ≤ 0: the machine-service price is undefined at x = 1.
    NotViable {
        /// D(1) = 1 − u(a + λγ(1)).
        d_at_1: f64,
    },
    /// f(lo) ≤ 0 at lo = [`BRACKET_LO`]: labour supply already meets demand at the
    /// bracket's left end.
    ///
    /// This does not say that no root exists. One can lie in (0, lo), where the solve does
    /// not look, as in the reference implementation (macro.py:102). At x = 0 the case
    /// cannot arise when T/h > N, since n_D(0) = T/h and n_S ≤ N. At lo it can: to first
    /// order in lo, n_D(0) − n_D(lo) = (T/h)·lo·[1 + δγ(0)(b/h − λ)/(1 − aδ)], which grows
    /// without bound as aδ → 1. G8 has a case with T/h − N = 5e-12, and one at
    /// a = 1 − 2^-53 where n_D(lo) is 0.014 against T/h = 10.
    NoInteriorAtZero {
        /// f at x = [`BRACKET_LO`].
        f_at_0: f64,
    },
}

impl Regime {
    /// The variant's name, as the dump example prints it.
    pub fn name(&self) -> &'static str {
        match self {
            Regime::Interior(_) => "Interior",
            Regime::BoundaryNoMargin { .. } => "BoundaryNoMargin",
            Regime::NotViable { .. } => "NotViable",
            Regime::NoInteriorAtZero { .. } => "NoInteriorAtZero",
        }
    }

    /// The equilibrium, if interior.
    pub fn interior(&self) -> Option<&Eq1a> {
        match self {
            Regime::Interior(eq) => Some(eq.as_ref()),
            _ => None,
        }
    }
}

/// A numerical failure. Valid parameters of ordinary size never produce one.
#[derive(Clone, Debug, PartialEq)]
pub enum SolveError {
    /// A quantity overflowed or became NaN: a bracket value, so no regime can be
    /// decided, or a reported output of an interior solve.
    NonFinite {
        /// What was not finite: a bracket value, or the key (as [`Eq1a::outputs`] and the
        /// dump name it) of the first output in that list that was.
        what: &'static str,
    },
    /// Bisection hit [`MAX_BISECTION_STEPS`].
    NoConvergence {
        /// The steps taken.
        steps: u32,
    },
    /// Bisection closed on a sign change of the f64 excess demand, but the labour market
    /// at the reported equilibrium does not clear: [`Residuals::labor`] is above
    /// [`LABOR_RESIDUAL_NET`]·N_a. Either f jumps there (a schedule that breaks the
    /// [`Schedule`] contract), or the root is not resolved by the doubles of x (very near
    /// the viability edge). See [`LABOR_RESIDUAL_NET`].
    LaborNotCleared {
        /// The double x* where bisection ended.
        x_star: f64,
        /// |N_a − n_S(x*)|/N_a.
        relative: f64,
    },
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveError::NonFinite { what } => write!(f, "{what} is not finite"),
            SolveError::NoConvergence { steps } => {
                write!(f, "bisection did not converge in {steps} steps")
            }
            SolveError::LaborNotCleared { x_star, relative } => write!(
                f,
                "the labour market does not clear at x* = {x_star:?}: |N_a - n_S| is \
                 {relative:e} of N_a, above LABOR_RESIDUAL_NET = {LABOR_RESIDUAL_NET:e}"
            ),
        }
    }
}

impl std::error::Error for SolveError {}

fn finite(what: &'static str, value: f64) -> Result<f64, SolveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(SolveError::NonFinite { what })
    }
}

impl<S: Schedule> Economy<S> {
    /// Prices and quantities at a candidate threshold x in [0, 1] (spec §3.1-3.2).
    ///
    /// Outside [0, 1], or where D(x) ≤ 0, the formulas are evaluated as written and
    /// mean nothing.
    pub fn at(&self, x: f64) -> Point {
        let p = self.params();
        let u = self.user_cost();
        let gamma = p.schedule.gamma(x);
        let j = p.schedule.integral(x);
        // Price block (spec §3.1), in the operation order of macro.py:79-86 except D,
        // which is formed without rounding a + λγ first (see `viability`).
        let d = viability(u, p.a, p.lam, gamma);
        let v_m = p.b / d;
        let p_m = u * v_m;
        let v = gamma * p_m;
        let price = v * (1.0 - x) + p_m * j;
        let p_s = price + p.space;
        // Quantity block (spec §3.2), macro.py:87-93, with 1 − aδ from one fused
        // multiply-add, so that it keeps its relative precision as aδ → 1.
        let s_k = num::fma(-p.a, p.delta, 1.0);
        let y = p.land / (p.space + p.b * p.delta * j / s_k);
        let k = y * j / s_k;
        let final_hours = y * (1.0 - x);
        let machine_hours = p.lam * p.delta * k;
        let n_d = final_hours + machine_hours;
        let n_s = p.workers * p.work_cost.cdf(num::ln1p(v / p_s));
        Point {
            x,
            gamma,
            j,
            d,
            v_m,
            p_m,
            v,
            p: price,
            p_s,
            y,
            k,
            final_hours,
            machine_hours,
            n_d,
            n_s,
        }
    }

    /// Classifies the economy and, when interior, solves n_D(x*) = n_S(x*) (spec §4).
    ///
    /// The checks run in the spec's order: viability D(1) > 0, then f(1) < 0, then
    /// f(lo) > 0 with lo = [`BRACKET_LO`]; a value exactly 0 fails its check (see
    /// [`Regime`]). An interior root is then unique (SSRN Lemma B.1, p.29, extended to u
    /// by spec §4) and is found by bisection to adjacent doubles; 1 − x* is then
    /// resolved between them (see [`Eq1a::one_minus_x_star`]).
    ///
    /// Returns [`SolveError::NonFinite`] if a bracket value or any reported output is
    /// not finite, so an `Interior` result never carries NaN or infinity. D(1) = −∞, from
    /// u·λγ(1) overflowing, is not an error but [`Regime::NotViable`]: D(1) is then
    /// certainly negative. Returns [`SolveError::LaborNotCleared`] if the reported
    /// equilibrium does not clear the labour market to [`LABOR_RESIDUAL_NET`], so an
    /// `Interior` result is always a root.
    pub fn solve(&self) -> Result<Regime, SolveError> {
        let at_hi = self.at(BRACKET_HI);
        // D = (1 − u·a) − u·λγ with u finite and a < 1: 1 − u·a is finite, so D is finite
        // or −∞ (u·λγ overflowed), and −∞ is not viable. NaN would be undecidable.
        let d_at_1 = at_hi.d;
        if d_at_1.is_nan() {
            return Err(SolveError::NonFinite { what: "D(1)" });
        }
        if d_at_1 <= 0.0 {
            return Ok(Regime::NotViable { d_at_1 });
        }
        let f_hi = finite("n_D(1) - n_S(1)", at_hi.excess_demand())?;
        if f_hi >= 0.0 {
            return Ok(Regime::BoundaryNoMargin { f_at_1: f_hi });
        }
        let f_lo = finite(
            "n_D - n_S at BRACKET_LO",
            self.at(BRACKET_LO).excess_demand(),
        )?;
        if f_lo <= 0.0 {
            return Ok(Regime::NoInteriorAtZero { f_at_0: f_lo });
        }
        let root = bisect(
            |x| self.at(x).excess_demand(),
            (BRACKET_LO, f_lo),
            (BRACKET_HI, f_hi),
        )?;
        let eq = self.report(root, &at_hi);
        if let Some(what) = first_non_finite(&eq) {
            return Err(SolveError::NonFinite { what });
        }
        // The safety net: a sign change of f that is not a root (LABOR_RESIDUAL_NET).
        if eq.residuals.labor > LABOR_RESIDUAL_NET * eq.n_a {
            return Err(SolveError::LaborNotCleared {
                x_star: eq.x_star,
                relative: eq.residuals.labor / eq.n_a,
            });
        }
        Ok(Regime::Interior(Box::new(eq)))
    }

    /// Spec §3.4-3.5 and §4 step 5 at x*.
    fn report(&self, root: Root, at_one: &Point) -> Eq1a {
        let p = self.params();
        let u = self.user_cost();
        let q = self.at(root.x);
        // The outputs proportional to 1 − x* take it from the root, not from 1.0 − x*
        // (Eq1a's precision note). Everything else is evaluated at the double x*.
        let one_minus_x = root.one_minus_x;
        let final_hours = q.y * one_minus_x;
        let n_a = final_hours + q.machine_hours;
        let interest = p.rho * wealth_factor(p.rho, p.delta, p.build_lag) * q.v_m * q.k;
        let wages = q.v * n_a;
        let income = wages + p.land + interest;
        let support_cost = p.workers * q.p_s;
        let provider_baskets = (p.land + interest) / q.p_s - p.workers;
        // φ and the cost system are the flow price system's; report them only at u = 1.
        let flow_prices = u == 1.0;
        let phi_w = flow_prices.then(|| p.lam * q.gamma / (1.0 - p.a));
        let cost_system = flow_prices.then(|| {
            let lam_m = p.lam / (1.0 - p.a);
            let b_m = p.b / (1.0 - p.a);
            let lambda_tilde = [one_minus_x + q.j * lam_m, lam_m, 0.0];
            let b_tilde = [q.j * b_m, b_m, 1.0];
            let z = [1.0, 0.0, p.space];
            let dot = |t: &[f64; 3]| z[0] * t[0] + z[1] * t[1] + z[2] * t[2];
            CostSystem {
                lambda_tilde,
                b_tilde,
                l_s: dot(&lambda_tilde),
                b_s: dot(&b_tilde),
            }
        });
        let residuals = Residuals {
            labor: (n_a - q.n_s).abs(),
            income: (q.y * q.p_s - income).abs() / income,
            land: (p.space * q.y + p.b * p.delta * q.k - p.land).abs() / p.land,
            services: (q.k - q.y * q.j - p.a * p.delta * q.k).abs() / q.k,
            user_cost: (q.p_m - u * (p.a * q.p_m + p.lam * q.v + p.b)).abs() / q.p_m,
        };
        Eq1a {
            x_star: root.x,
            one_minus_x_star: one_minus_x,
            gamma_star: q.gamma,
            j_star: q.j,
            u,
            v: q.v,
            p_m: q.p_m,
            v_m: q.v_m,
            p: q.p,
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
            cost_system,
            residuals,
            bisection_steps: root.steps,
        }
    }
}

/// The key of the first number in [`Eq1a::outputs`] that is NaN or infinite.
fn first_non_finite(eq: &Eq1a) -> Option<&'static str> {
    eq.outputs()
        .into_iter()
        .find_map(|(key, output)| match output {
            Output::Float(v) | Output::FlowOnly(Some(v)) if !v.is_finite() => Some(key),
            _ => None,
        })
}

/// Where bisection leaves the root.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Root {
    /// The double reported as x*.
    x: f64,
    /// 1 − x*, resolved below the spacing of doubles (see [`Eq1a::one_minus_x_star`]).
    one_minus_x: f64,
    /// Bisection steps taken.
    steps: u32,
}

impl Root {
    /// The root between adjacent doubles lo < hi with f(lo) > 0 > f(hi): x is the end
    /// with the smaller |f| (lo on a tie), and 1 − x* comes from the straight line
    /// through the two ends.
    fn between((lo, f_lo): (f64, f64), (hi, f_hi): (f64, f64), steps: u32) -> Root {
        let x = if f_hi.abs() < f_lo.abs() { hi } else { lo };
        // f_lo > 0 > f_hi, so the denominator has no cancellation and θ ∈ [0, 1]. An
        // overflowing denominator gives θ = 0, that is 1 − hi.
        let theta = -f_hi / (f_lo - f_hi);
        Root {
            x,
            one_minus_x: (1.0 - hi) + theta * (hi - lo),
            steps,
        }
    }

    /// An exact zero of f at x.
    fn exact(x: f64, steps: u32) -> Root {
        Root {
            x,
            one_minus_x: 1.0 - x,
            steps,
        }
    }
}

/// Deterministic bisection on a decreasing f with f(lo) > 0 > f(hi).
///
/// Stops when the floating midpoint equals an endpoint, so lo and hi are adjacent
/// doubles (see [`Root::between`]). There is no tolerance to tune, and the result is the
/// same on every run. An exact zero at a midpoint is returned at once.
fn bisect(
    f: impl Fn(f64) -> f64,
    (mut lo, mut f_lo): (f64, f64),
    (mut hi, mut f_hi): (f64, f64),
) -> Result<Root, SolveError> {
    for step in 0..MAX_BISECTION_STEPS {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            return Ok(Root::between((lo, f_lo), (hi, f_hi), step));
        }
        let f_mid = f(mid);
        if f_mid > 0.0 {
            (lo, f_lo) = (mid, f_mid);
        } else if f_mid < 0.0 {
            (hi, f_hi) = (mid, f_mid);
        } else if f_mid == 0.0 {
            return Ok(Root::exact(mid, step + 1));
        } else {
            return Err(SolveError::NonFinite {
                what: "n_D - n_S inside the bracket",
            });
        }
    }
    Err(SolveError::NoConvergence {
        steps: MAX_BISECTION_STEPS,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: impl Fn(f64) -> f64) -> Result<Root, SolveError> {
        bisect(&f, (BRACKET_LO, f(BRACKET_LO)), (BRACKET_HI, f(BRACKET_HI)))
    }

    #[test]
    fn bisection_lands_on_adjacent_doubles() {
        for root in [0.3, 0.863150418162437, 0.999_999_9, 2e-12, 1.5e-12] {
            let r = run(|x| root - x).unwrap();
            assert!(r.steps < MAX_BISECTION_STEPS);
            // x is within one double of the root: the root lies between x and a neighbour.
            let (below, above) = (r.x.next_down(), r.x.next_up());
            assert!(
                below <= root && root <= above,
                "root {root:e}, got {:e}",
                r.x
            );
            // f is linear and exact here, so 1 - x* is 1 - root to rounding.
            let want = 1.0 - root;
            assert!(
                (r.one_minus_x - want).abs() <= 2.0 * f64::EPSILON * want,
                "1 - x* for root {root:e}: got {:e}",
                r.one_minus_x
            );
        }
    }

    #[test]
    fn one_minus_x_is_resolved_below_the_spacing_of_doubles() {
        // Roots at 1 - s for s far below the spacing of doubles near 1 (1.1e-16). x* is a
        // double, so it is 1 - 2^-53 or 1.0 and 1.0 - x* is 1.1e-16 or 0; the interpolated
        // 1 - x* is s to within rounding.
        for s in [3e-17, 1e-20, 2.3e-31] {
            let r = run(|x| (1.0 - x) - s).unwrap();
            assert!(r.x == 1.0 || r.x == 1.0f64.next_down(), "{:e}", r.x);
            assert!(
                (r.one_minus_x - s).abs() <= 4.0 * f64::EPSILON * s,
                "s = {s:e}: got {:e}",
                r.one_minus_x
            );
        }
        // Between two doubles away from 1 the same interpolation holds: the root 0.3 + d
        // with d a third of the spacing there.
        let spacing = 0.3f64.next_up() - 0.3;
        let root_gap = spacing / 3.0;
        let r = run(|x| (0.3 - x) + root_gap).unwrap();
        assert_eq!(r.x, 0.3);
        let want = (1.0 - 0.3) - root_gap;
        assert!((r.one_minus_x - want).abs() <= 2.0 * f64::EPSILON * want);
    }

    #[test]
    fn bisection_worst_case_stays_under_the_cap() {
        // A root at the bracket's left end needs the most halvings: the doubles there
        // are 2^-92 apart. The cap's documented bound is 92 plus rounding.
        let root = BRACKET_LO.next_up();
        let r = run(|x| if x <= root { 1.0 } else { -1.0 }).unwrap();
        assert!(r.steps <= 94, "{} steps", r.steps);
        assert!(r.x == root || r.x == root.next_up(), "{:e}", r.x);
    }

    #[test]
    fn bisection_returns_an_exact_zero() {
        // 0.5 is the first midpoint of [BRACKET_LO, 1] only approximately, so place the
        // zero at the exact first midpoint.
        let first = 0.5 * (BRACKET_LO + BRACKET_HI);
        let r = run(|x| first - x).unwrap();
        assert_eq!(r, Root::exact(first, 1));
        assert_eq!(r.one_minus_x, 1.0 - first);
    }

    #[test]
    fn bisection_reports_nan() {
        let err = run(|x| {
            if x == BRACKET_LO {
                1.0
            } else if x == BRACKET_HI {
                -1.0
            } else {
                f64::NAN
            }
        })
        .unwrap_err();
        assert!(matches!(err, SolveError::NonFinite { .. }));
    }

    /// An `Eq1a` whose numbers are 1, 2, 3, ... in field order, with the `nan_at`-th
    /// replaced by NaN (none when `nan_at` is 0). A struct literal must name every field,
    /// so a field added to `Eq1a` does not compile here until it is given a number.
    fn numbered(nan_at: usize) -> Eq1a {
        let mut next = 0;
        let mut n = || {
            next += 1;
            if next == nan_at {
                f64::NAN
            } else {
                next as f64
            }
        };
        Eq1a {
            x_star: n(),
            one_minus_x_star: n(),
            gamma_star: n(),
            j_star: n(),
            u: n(),
            v: n(),
            p_m: n(),
            v_m: n(),
            p: n(),
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
            cost_system: Some(CostSystem {
                lambda_tilde: [n(), n(), n()],
                b_tilde: [n(), n(), n()],
                l_s: n(),
                b_s: n(),
            }),
            residuals: Residuals {
                labor: n(),
                income: n(),
                land: n(),
                services: n(),
                user_cost: n(),
            },
            bisection_steps: 7,
        }
    }

    fn numbers(eq: &Eq1a) -> Vec<(&'static str, f64)> {
        eq.outputs()
            .into_iter()
            .filter_map(|(key, output)| match output {
                Output::Float(v) | Output::FlowOnly(Some(v)) => Some((key, v)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn outputs_list_every_field_once() {
        // Every number of Eq1a appears in outputs() exactly once, in field order, and
        // every key is distinct.
        let eq = numbered(0);
        let listed = numbers(&eq);
        let values: Vec<f64> = listed.iter().map(|(_, v)| *v).collect();
        let want: Vec<f64> = (1..=listed.len()).map(|i| i as f64).collect();
        assert_eq!(values, want);
        assert_eq!(listed.len(), 39);
        let mut keys: Vec<&str> = eq.outputs().iter().map(|(k, _)| *k).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), eq.outputs().len());
        let flags: Vec<(&str, Output)> = eq
            .outputs()
            .into_iter()
            .filter(|(_, o)| matches!(o, Output::Flag(_) | Output::Count(_)))
            .collect();
        assert_eq!(
            flags,
            [
                ("funded", Output::Flag(true)),
                ("lemma_b1", Output::Flag(false)),
                ("bisection_steps", Output::Count(7)),
            ]
        );
    }

    #[test]
    fn every_output_is_checked_for_finiteness() {
        assert_eq!(first_non_finite(&numbered(0)), None);
        let keys: Vec<&str> = numbers(&numbered(0)).iter().map(|(k, _)| *k).collect();
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(
                first_non_finite(&numbered(i + 1)),
                Some(*key),
                "number {}",
                i + 1
            );
        }
        // A flow-only output that is absent is not a number and is skipped.
        let mut eq = numbered(0);
        eq.phi_w = None;
        eq.residuals.land = f64::INFINITY;
        assert_eq!(first_non_finite(&eq), Some("res_land"));
    }

    #[test]
    fn the_quantity_block_keeps_one_minus_a_delta() {
        // a = 1 - 3·2^-30 and delta = 1 - 5·2^-30: a·delta needs 60 bits, so the plain
        // 1 - a·delta rounds it near 1 and is off by up to 2^-54/2^-27 = 7.5e-9 relative.
        // Exactly, 1 - a·delta = 2^-27 - 15·2^-60, which is a double; the fused form gets it,
        // and Y = T/(h + b·delta·J/(1 - a·delta)) follows to rounding.
        use crate::{pow2, Params, PowerSchedule, UniformWorkCost};
        let (a, delta) = (1.0 - 3.0 * pow2(-30), 1.0 - 5.0 * pow2(-30));
        let s_k = pow2(-27) - 15.0 * pow2(-60);
        assert_ne!(1.0 - a * delta, s_k);
        let economy = Economy::new(Params {
            workers: 4.0,
            land: 10.0,
            space: 1.0,
            a,
            lam: 0.0,
            b: 0.4,
            schedule: PowerSchedule {
                eta: 1.0,
                g0: 0.2,
                g1: 0.8,
                k: 1.0,
            },
            work_cost: UniformWorkCost { chi_max: 1.0 },
            rho: 0.0,
            delta,
            build_lag: 1,
        })
        .unwrap();
        let q = economy.at(0.5);
        let y = 10.0 / (1.0 + 0.4 * delta * q.j / s_k);
        let k = y * q.j / s_k;
        for (name, got, want) in [("Y", q.y, y), ("K", q.k, k)] {
            assert!(
                (got - want).abs() <= 4.0 * f64::EPSILON * want,
                "{name}: got {got:e}, want {want:e}"
            );
        }
    }

    #[test]
    fn bisection_is_repeatable() {
        let f = |x: f64| num::exp(-x) - x.sqrt();
        let first = run(f).unwrap();
        for _ in 0..3 {
            assert_eq!(run(f).unwrap(), first);
        }
    }
}
