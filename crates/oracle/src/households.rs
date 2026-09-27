//! Unit 1f: households and government (docs/unit-1f.md).
//!
//! Unit 1e's economy with a government and a basket that can respond to prices. The government
//! has SSRN A.1's instruments: a payroll tax on gross wages, a uniform tax on final purchases, a
//! tax on market rent, a uniform transfer to every person and a transfer program paying (m_w,
//! m_e) in work and in exit (§2.1-2.2). Its budget balances at every equilibrium, closed by the
//! owners' levy (the default, [`Budget::RentRate`]) or by the uniform transfer
//! ([`Budget::Dividend`]) (§2.5). Producers pay gross prices, so the government enters the
//! equilibrium only through who offers hours: one participation rule, SSRN eq 8 with the support,
//! the exit life and the government, covers SSRN eq 9, p.16, eq 27 and units 1d and 1e as cases
//! (§2.3, §4.4). The basket is 1b's fixed one ([`Basket::Fixed`], decision 59) or a CES over the
//! categories ([`Basket::Ces`], SSRN eq 26), whose unit-elasticity case is check_pinning's and
//! check_dynamics' land-share household (§2.10).
//!
//! The evaluation order is normative (§5.1): with [`Government::none`] and the fixed basket every
//! new operation is an exact no-op, and the solve repeats unit 1e's floating-point operations bit
//! for bit (§2.14, §9).

use rustyecon_core::num;

use crate::categories::Output1b;
use crate::machines::{Item, OutputKey1c};
use crate::params::{self, ParamError};
use crate::parcels::{
    EnclosurePoint, EnclosureSide, Eq1e, ParcelEconomy, ParcelParams, ParcelPoint,
};
use crate::schedule::{PowerSchedule, Schedule};
use crate::solve::{Regime, SolveError};
use crate::workers::{Edge, WorkerType};

/// The participation rule with a government, as the evaluation reads it (docs/unit-1f.md §2.3,
/// §4.3-4.5): the consumer price's factor, the net wage's, the program in composites, how a
/// transfer meets the support, and the budget's closure. [`Rule::none`] is units 1a-1e's rule,
/// with which every operation below is an exact no-op on IEEE doubles (§9).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rule {
    /// 1 + t_c: a composite costs a household P^c = (1 + t_c)·P.
    pub(crate) consumer: f64,
    /// 1 − τ_w: a worker keeps (1 − τ_w)·v_i.
    pub(crate) net: f64,
    /// μ_w, the program's payment in work, in composites at consumer prices.
    pub(crate) work: f64,
    /// μ_e, its payment in exit.
    pub(crate) exit: f64,
    /// Δμ = μ_w − μ_e.
    pub(crate) gap: f64,
    /// Replace mode: A_i = max(ν_i·P^c, d) (§2.4); Supplement otherwise, A_i = ν_i·P^c + d.
    pub(crate) replace: bool,
    /// The budget's closure (§2.5).
    pub(crate) budget: Closure,
}

/// The budget's closure as the evaluation reads it (docs/unit-1f.md §2.5 and §4.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Closure {
    /// The owners' levy balances the budget: the uniform transfer is d̂ composites per person.
    RentRate {
        /// d̂.
        dividend: f64,
    },
    /// Every rate is given and the uniform transfer is the residual, with a uniform program μ.
    Dividend {
        /// τ_w.
        payroll: f64,
        /// τ_R.
        rent_tax: f64,
        /// t_c.
        consumption: f64,
        /// μ = μ_w = μ_e.
        program: f64,
        /// N = Σ_i N_i, the persons the transfer goes to (§2.7).
        people: f64,
    },
}

impl Rule {
    /// Units 1a-1e's rule: no tax, no transfer, no program, Supplement (docs/unit-1f.md §2.14).
    pub(crate) fn none() -> Rule {
        // 1 + t_c, 1 − τ_w and μ_w − μ_e at zero rates and payments, exactly.
        Rule {
            consumer: 1.0,
            net: 1.0,
            work: 0.0,
            exit: 0.0,
            gap: 0.0,
            replace: false,
            budget: Closure::RentRate { dividend: 0.0 },
        }
    }

    /// d, the uniform transfer in money at a point whose composite costs P (§4.3): d̂·P^c under
    /// RentRate; under Dividend (τ_w·W + τ_R·R + t_c·C − μ·P^c·N)/N from the point's gross wage
    /// bill W, market rent R and consumption C = Y·P at producer value.
    pub(crate) fn transfer(&self, p: f64, wages: f64, rent: f64, consumption: f64) -> f64 {
        let p_c = self.consumer * p;
        match self.budget {
            Closure::RentRate { dividend } => dividend * p_c,
            Closure::Dividend {
                payroll,
                rent_tax,
                consumption: rate,
                program,
                people,
            } => {
                let revenue = (payroll * wages + rent_tax * rent) + rate * consumption;
                (revenue - (program * p_c) * people) / people
            }
        }
    }

    /// A_i, a type's unearned resources in both states (§2.4): ν_i·P^c + d in Supplement mode,
    /// max(ν_i·P^c, d) in Replace mode, the product formed first.
    pub(crate) fn unearned(&self, support: f64, p_c: f64, transfer: f64) -> f64 {
        let own = support * p_c;
        if self.replace {
            own.max(transfer)
        } else {
            own + transfer
        }
    }

    /// s_i, the provider's support per person (§4.3): ν_i·P^c, or (ν_i·P^c − d)⁺ in Replace mode.
    pub(crate) fn support(&self, support: f64, p_c: f64, transfer: f64) -> f64 {
        let own = support * p_c;
        if self.replace {
            (own - transfer).max(0.0)
        } else {
            own
        }
    }

    /// (num_i, den_i) of §4.4's point form at the wage v_i, the composite's producer price P,
    /// the exit value e_i and the transfer d: num = (m_w − m_e) + ((1 − τ_w)·v_i − ê_i),
    /// den = (A_i + m_e) + ê_i, ê_i = (1 + t_c)·e_i. With [`Rule::none`] they are unit 1e's
    /// v_i − e_i and ν_i·P + e_i bit for bit (§9 item 5).
    pub(crate) fn terms(
        &self,
        t: &WorkerType,
        wage: f64,
        p: f64,
        exit: f64,
        transfer: f64,
    ) -> (f64, f64) {
        let p_c = self.consumer * p;
        let a = self.unearned(t.support, p_c, transfer);
        let (m_w, m_e) = (self.work * p_c, self.exit * p_c);
        let home = self.consumer * exit;
        let num = (m_w - m_e) + (self.net * wage - home);
        let den = (a + m_e) + home;
        (num, den)
    }

    /// ln1p(num_i/den_i), the marginal worker's work cost (§4.4).
    pub(crate) fn marginal_cost(
        &self,
        t: &WorkerType,
        wage: f64,
        p: f64,
        exit: f64,
        transfer: f64,
    ) -> f64 {
        let (num, den) = self.terms(t, wage, p, exit, transfer);
        num::ln1p(num / den)
    }

    /// n_S,i = N_i·F_i(ln1p(num_i/den_i)), SSRN eq 8 and 10 with the government (§4.4).
    pub(crate) fn supply(
        &self,
        t: &WorkerType,
        wage: f64,
        p: f64,
        exit: f64,
        transfer: f64,
    ) -> f64 {
        t.workers
            * t.work_cost
                .cdf(self.marginal_cost(t, wage, p, exit, transfer))
    }

    /// â_i, a type's unearned composites at a corner where the transfer is δ composites (§4.4):
    /// ν_i + δ, or max(ν_i, δ) in Replace mode.
    pub(crate) fn composites(&self, support: f64, delta: f64) -> f64 {
        if self.replace {
            support.max(delta)
        } else {
            support + delta
        }
    }

    /// ratio_i(ω) = ((μ_w − μ_e) + ((1 − τ_w)·(ε_i·ω))/(1 + t_c))/(â_i + μ_e), §4.4's corner form
    /// for a type without an exit value; +∞ at ω = +∞, where supply saturates. With
    /// [`Rule::none`] it is unit 1d's (ε_i·ω)/ν_i bit for bit (§9 item 5).
    pub(crate) fn ratio(&self, t: &WorkerType, omega: f64, delta: f64) -> f64 {
        if omega == f64::INFINITY {
            return f64::INFINITY;
        }
        let paid = (self.net * (t.efficiency * omega)) / self.consumer;
        (self.gap + paid) / (self.composites(t.support, delta) + self.exit)
    }

    /// c_i, a walled type's wage per unit of P at its wall (§4.5): ((â_i + μ_e)·ζ_i −
    /// (μ_w − μ_e))·(1 + t_c)/(1 − τ_w), where ζ_i is its clearing value of num/den. With
    /// [`Rule::none`] it is unit 1d's ζ_i·ν_i bit for bit (§9 item 6).
    pub(crate) fn walled_rate(&self, support: f64, clearing: f64, delta: f64) -> f64 {
        let owed = (self.composites(support, delta) + self.exit) * clearing - self.gap;
        (owed * self.consumer) / self.net
    }

    /// δ where it does not depend on the point: d̂ under RentRate. Under Dividend there are no
    /// walled types (§2.6), and the value only fills the walk's rates of types that cannot be
    /// walled, so 0.
    pub(crate) fn fixed_delta(&self) -> f64 {
        match self.budget {
            Closure::RentRate { dividend } => dividend,
            Closure::Dividend { .. } => 0.0,
        }
    }

    /// Whether the transfer moves with T_m on the idle stretch, where R = 0 and prices are fixed:
    /// the Dividend closure with a payroll or a consumption tax (docs/unit-1f.md §5.3 step 3).
    pub(crate) fn moving_on_idle_land(&self) -> bool {
        match self.budget {
            Closure::RentRate { .. } => false,
            Closure::Dividend {
                payroll,
                consumption,
                ..
            } => payroll > 0.0 || consumption > 0.0,
        }
    }

    /// Whether the transfer moves with the point: the Dividend closure with a positive rate
    /// (docs/unit-1f.md §14).
    pub(crate) fn moving_transfer(&self) -> bool {
        match self.budget {
            Closure::RentRate { .. } => false,
            Closure::Dividend {
                payroll,
                rent_tax,
                consumption,
                ..
            } => payroll > 0.0 || rent_tax > 0.0 || consumption > 0.0,
        }
    }
}

/// A CES basket over the categories (docs/unit-1f.md §2.10 and §4.2): the fixed basket's weights
/// z_j, their total Z and shares s̄_j = z_j/Z, and one elasticity σ.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ces {
    /// σ.
    pub(crate) sigma: f64,
    /// z_j.
    pub(crate) weights: Vec<f64>,
    /// s̄_j = z_j/Z.
    pub(crate) shares: Vec<f64>,
    /// Z = Σ_j z_j.
    pub(crate) total: f64,
}

impl Ces {
    /// The CES basket with weights z and elasticity σ.
    pub(crate) fn new(sigma: f64, weights: &[f64]) -> Ces {
        let mut total = 0.0;
        for &z in weights {
            total += z;
        }
        Ces {
            sigma,
            weights: weights.to_vec(),
            shares: weights.iter().map(|&z| z / total).collect(),
            total,
        }
    }

    /// ln M at the prices p (§5.1 step 2): Σ_j s̄_j·ln p_j at σ = 1; otherwise, with the prices
    /// scaled by the largest weighted one when σ < 1 and by the smallest when σ > 1, so that
    /// every exponent x_j = (1 − σ)·(ln p_j − ln p_scale) is ≤ 0, the sum S = Σ_j s̄_j·expm1(x_j)
    /// in (−1, 0], and ln M = ln p_scale + ln1p(S)/(1 − σ) where S ≥ [`LN1P_FLOOR`], else
    /// ln p_scale + ln(Σ_j s̄_j·exp(x_j))/(1 − σ). Sums run over the weighted categories in index
    /// order.
    pub(crate) fn log_mean(&self, logs: &[f64]) -> f64 {
        let weighted = || {
            logs.iter()
                .zip(&self.shares)
                .zip(&self.weights)
                .filter(|(_, &z)| z > 0.0)
                .map(|((&l, &s), _)| (l, s))
        };
        if self.sigma == 1.0 {
            let mut sum = 0.0;
            for (l, s) in weighted() {
                sum += s * l;
            }
            return sum;
        }
        let one_minus = 1.0 - self.sigma;
        let mut scale: Option<f64> = None;
        for (l, _) in weighted() {
            scale = Some(match scale {
                None => l,
                Some(s) if one_minus > 0.0 => s.max(l),
                Some(s) => s.min(l),
            });
        }
        let scale = scale.expect("a validated basket weighs some category");
        let mut sum = 0.0;
        for (l, s) in weighted() {
            sum += s * num::expm1(one_minus * (l - scale));
        }
        if sum >= LN1P_FLOOR {
            return scale + num::ln1p(sum) / one_minus;
        }
        // 1 + S is small: the sum of positive terms, which does not cancel.
        let mut total = 0.0;
        for (l, s) in weighted() {
            total += s * num::exp(one_minus * (l - scale));
        }
        scale + num::ln(total) / one_minus
    }

    /// The composite's producer price P = Z·M and its content c_j = z_j·exp(−σ·(ln p_j − ln M))
    /// at the category prices p (§4.2); c_j = 0 for an unweighted category.
    pub(crate) fn at(&self, prices: &[f64]) -> (f64, Vec<f64>) {
        let logs: Vec<f64> = prices.iter().map(|&p| num::ln(p)).collect();
        let ln_m = self.log_mean(&logs);
        let price = self.total * num::exp(ln_m);
        let content = (0..prices.len())
            .map(|j| {
                if self.weights[j] > 0.0 {
                    self.weights[j] * num::exp(-(self.sigma * (logs[j] - ln_m)))
                } else {
                    0.0
                }
            })
            .collect();
        (price, content)
    }

    /// Z·M(λ̃), the composite's labour at the wall's end, whose inverse is the limit of ω there
    /// (§2.12, §5.3 step 3); `None` for σ ≥ 1, where ω grows without bound, and where no
    /// weighted category embodies labour.
    pub(crate) fn labour_at_the_end(&self, labour: &[f64]) -> Option<f64> {
        if self.sigma >= 1.0 {
            return None;
        }
        let any = (0..labour.len()).any(|j| self.weights[j] > 0.0 && labour[j] > 0.0);
        if !any {
            return None;
        }
        let logs: Vec<f64> = labour.iter().map(|&l| num::ln(l)).collect();
        Some(self.total * num::exp(self.log_mean(&logs)))
    }
}

/// The least S = Σ_j s̄_j·expm1(x_j) at which [`Ces::log_mean`] takes ln1p(S) (docs/unit-1f.md
/// §5.1 step 2, §5.5).
///
/// Every x_j ≤ 0, so S ∈ (−1, 0] and 1 + S is the sum of the positive terms s̄_j·exp(x_j).
/// Where S ≥ −1/2, 1 + S ≥ 1/2 carries S's rounding with no more than twice its relative size,
/// and ln1p keeps ln M's precision as σ → 1, where S → 0. Below, 1 + S cancels down to the
/// weights of the categories near p_scale (W1 at σ = 20 with eq 26's weights: space's weight
/// 4.4e-8, 1 + S = 1.5e-7, and ln M would lose about 2^-53/(1.5e-7·19) ≈ 4e-11), while the
/// direct sum of positive terms carries a few ulps and its logarithm, at least ln 2 in size, a
/// few ulps of itself.
const LN1P_FLOOR: f64 = -0.5;

/// The largest elasticity of substitution a CES basket accepts (docs/unit-1f.md §3.2).
///
/// The CES is evaluated in logs, and a category's content is z_j·exp(−σ·(ln p_j − ln M)): with
/// σ = 64 an exponent reaches exp's overflow near 710 when one price is about 2^16 times
/// another (64·ln 2^16 ≈ 710), so a category's content stays finite for price ratios up to
/// about 2^16. Beyond, one category's content is below 2^-1022 or above 2^1023 of another's and
/// the point is a limit, not an evaluation (§2.12). The paper's σ are 0.5-2.
pub const SIGMA_CEIL: f64 = 64.0;

/// The basket every household buys (docs/unit-1f.md §2.10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Basket {
    /// Unit 1b's fixed basket z (SSRN eq 7; decision 59): the default, and the historical runs'.
    Fixed,
    /// A CES aggregate of the categories with the fixed basket's weights z_j and one elasticity
    /// σ in [[`SCALE_FLOOR`](crate::SCALE_FLOOR), [`SIGMA_CEIL`]]: P = Z·M, c_j =
    /// z_j·(p_j/M)^(−σ), so one composite is the basket z wherever the prices are equal. Only
    /// with every type without an exit value and no reserved hours (§2.11).
    Ces {
        /// σ.
        sigma: f64,
    },
}

/// How a uniform transfer meets the provider's support (docs/unit-1f.md §2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferMode {
    /// A_i = ν_i·P^c + d: the transfer adds to the support, and raises the reservation wage to
    /// (e^χ − 1)(P_s + d) (SSRN p.16). The default.
    Supplement,
    /// A_i = max(ν_i·P^c, d): the provider tops each person up to its support and pays
    /// (ν_i·P^c − d)⁺, "maintaining total support and prices" (SSRN p.16).
    Replace,
}

/// The budget's closure (docs/unit-1f.md §2.5): which instrument is the residual.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Budget {
    /// The transfers and the rates τ_w, t_c are given, and the owners pay the levy L_R that
    /// balances the budget: the rent tax's rate is L_R/R, reported (SSRN eq 16 is this closure's
    /// output at d̂ = 1). The default.
    RentRate {
        /// d̂, the uniform transfer per person in composites at consumer prices: 0 or scale.
        dividend: f64,
    },
    /// Every rate is given and the uniform transfer d is the residual: SSRN A.1's "returned as a
    /// uniform transfer d = τ_R·R/N". Only where the budget at a point does not depend on who
    /// works: no reserved hours, no plot-taking type and a uniform program (§2.6).
    Dividend {
        /// τ_R, the tax on market rent, in [0, 1].
        rent_tax: f64,
    },
}

/// A transfer program (SSRN A.1 and D.1): a payment per person in work and one in exit, in
/// composites at consumer prices, each 0 or scale, in addition to the supported basket.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Program {
    /// μ_w.
    pub work: f64,
    /// μ_e.
    pub exit: f64,
}

/// The government of SSRN A.1 (docs/unit-1f.md §2.1-2.5, §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Government {
    /// τ_w, the payroll tax, a share of the gross wage of every hour sold, in [0, 1).
    pub payroll: f64,
    /// t_c, the uniform tax on final purchases at producer value, in [0, SCALE_CEIL].
    pub consumption: f64,
    /// The budget's closure, with its given instrument.
    pub budget: Budget,
    /// The transfer program.
    pub program: Program,
    /// How a uniform transfer meets the support.
    pub mode: TransferMode,
}

impl Government {
    /// No government: every rate 0, RentRate with d̂ = 0, no program, Supplement. With it and the
    /// fixed basket, unit 1f is unit 1e bit for bit (docs/unit-1f.md §2.14).
    pub fn none() -> Government {
        Government {
            payroll: 0.0,
            consumption: 0.0,
            budget: Budget::RentRate { dividend: 0.0 },
            program: Program {
                work: 0.0,
                exit: 0.0,
            },
            mode: TransferMode::Supplement,
        }
    }
}

/// The parameters of unit 1f (docs/unit-1f.md §3.1): unit 1e's [`ParcelParams`], the basket and
/// the government. Build a [`HouseholdEconomy`] from them to validate.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdParams<S = PowerSchedule> {
    /// Unit 1e's economy.
    pub economy: ParcelParams<S>,
    /// The basket.
    pub basket: Basket,
    /// The government.
    pub government: Government,
}

impl<S> HouseholdParams<S> {
    /// Unit 1e's economy in household form (docs/unit-1f.md §3.3, H0): the fixed basket and
    /// [`Government::none`]. It solves to 1e's equilibrium bit for bit, and every error is 1e's.
    pub fn from_parcels(economy: ParcelParams<S>) -> Self {
        HouseholdParams {
            economy,
            basket: Basket::Fixed,
            government: Government::none(),
        }
    }
}

/// A validated unit-1f economy (docs/unit-1f.md §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdEconomy<S = PowerSchedule> {
    params: HouseholdParams<S>,
    /// Unit 1e's economy, with the rule and the basket set underneath.
    parcels: ParcelEconomy<S>,
    /// Σ_i N_i, the persons the transfers go to (§2.7).
    people: f64,
}

/// The error for a field of the government.
fn in_government(error: ParamError) -> ParamError {
    ParamError::Item {
        kind: "government",
        index: 0,
        error: Box::new(error),
    }
}

impl<S: Schedule + Clone> HouseholdEconomy<S> {
    /// Validates the parameters (docs/unit-1f.md §3.2) and sets the rule and the basket.
    ///
    /// In order: unit 1e's checks in 1e's order ([`ParcelEconomy::new`]); under a CES basket σ
    /// in [[`SCALE_FLOOR`](crate::SCALE_FLOOR), [`SIGMA_CEIL`]] (`Invalid`-free
    /// `OutOfRange { name: "sigma" }`), and every type without an exit value and no reserved
    /// hours (`Invalid { name: "basket" }`, §2.11); the government's fields (`Item { kind:
    /// "government", .. }` with the field's name: `payroll` in [0, 1), `consumption` in [0,
    /// SCALE_CEIL], `dividend`, `work` and `exit` each 0 or scale, `rent_tax` in [0, 1]); the
    /// Dividend closure only without reserved hours, without a plot-taking type and with
    /// μ_w = μ_e (`Invalid { name: "budget" }`, §2.6); and with reserved hours μ_w ≤ μ_e
    /// (`Invalid { name: "program" }`, §2.8). −0.0 is stored as +0.0. With the fixed basket and
    /// [`Government::none`] these are 1e's rules.
    pub fn new(mut params: HouseholdParams<S>) -> Result<Self, ParamError> {
        let mut parcels = ParcelEconomy::new(params.economy.clone())?;
        params.economy = parcels.params().clone();
        let reserved = params.economy.reserved.iter().flatten().any(|&r| r > 0.0);
        if let Basket::Ces { sigma } = &mut params.basket {
            *sigma = params::elasticity("sigma", *sigma)?;
            if !parcels.exit_free() || reserved {
                return Err(ParamError::Invalid {
                    name: "basket",
                    reason: "a CES basket needs every worker type without an exit value (the \
                             dependence form, or s0 = s_ = 0) and no reserved hours: the walk and \
                             the plots would move with the basket (docs/unit-1f.md §2.11)",
                });
            }
        }
        let g = &mut params.government;
        g.payroll = params::below_one("payroll", g.payroll).map_err(in_government)?;
        g.consumption = params::nonnegative("consumption", g.consumption).map_err(in_government)?;
        match &mut g.budget {
            Budget::RentRate { dividend } => {
                *dividend = params::zero_or_scale("dividend", *dividend).map_err(in_government)?;
            }
            Budget::Dividend { rent_tax } => {
                *rent_tax = params::unit_interval("rent_tax", *rent_tax).map_err(in_government)?;
            }
        }
        g.program.work = params::zero_or_scale("work", g.program.work).map_err(in_government)?;
        g.program.exit = params::zero_or_scale("exit", g.program.exit).map_err(in_government)?;
        let takers = !parcels.plot_takers().is_empty();
        if let Budget::Dividend { .. } = g.budget {
            if reserved || takers || g.program.work != g.program.exit {
                return Err(ParamError::Invalid {
                    name: "budget",
                    reason: "the Dividend closure needs a budget that does not depend on who \
                             works at a point: no reserved hours, no plot-taking type and a \
                             uniform program (docs/unit-1f.md §2.6)",
                });
            }
        }
        if reserved && g.program.work > g.program.exit {
            return Err(ParamError::Invalid {
                name: "program",
                reason: "with reserved hours the program's payment in work must not exceed its \
                         payment in exit: a walled type would supply its hours at no wage \
                         (docs/unit-1f.md §2.8)",
            });
        }
        let mut people = 0.0;
        for t in &params.economy.worker_types {
            people += t.workers;
        }
        let rule = Rule {
            consumer: 1.0 + g.consumption,
            net: 1.0 - g.payroll,
            work: g.program.work,
            exit: g.program.exit,
            gap: g.program.work - g.program.exit,
            replace: g.mode == TransferMode::Replace,
            budget: match g.budget {
                Budget::RentRate { dividend } => Closure::RentRate { dividend },
                Budget::Dividend { rent_tax } => Closure::Dividend {
                    payroll: g.payroll,
                    rent_tax,
                    consumption: g.consumption,
                    program: g.program.work,
                    people,
                },
            },
        };
        let ces = match params.basket {
            Basket::Fixed => None,
            Basket::Ces { sigma } => Some(Ces::new(sigma, parcels.workers().weights())),
        };
        parcels.set_households(rule, ces);
        Ok(HouseholdEconomy {
            params,
            parcels,
            people,
        })
    }

    /// The validated parameters.
    pub fn params(&self) -> &HouseholdParams<S> {
        &self.params
    }

    /// The unit-1e economy underneath, with the rule and the basket set: its evaluations and its
    /// solve are unit 1f's.
    pub fn parcels(&self) -> &ParcelEconomy<S> {
        &self.parcels
    }

    /// κ_w = (1 − τ_w)/(1 + t_c), the share of the gross wage a worker keeps in producer-priced
    /// composites (docs/unit-1f.md §2.3).
    pub fn net_factor(&self) -> f64 {
        let rule = self.parcels.workers().rule();
        rule.net / rule.consumer
    }

    fn wrap(&self, point: ParcelPoint) -> HouseholdPoint {
        let w = self.parcels.workers();
        let rule = w.rule();
        let p = point.point.p_s;
        let consumer_price = rule.consumer * p;
        let transfer = point.point.transfer;
        let unearned = self
            .params
            .economy
            .worker_types
            .iter()
            .map(|t| rule.unearned(t.support, consumer_price, transfer))
            .collect();
        HouseholdPoint {
            content: w.content_of(&point.point.basket).to_vec(),
            price: p,
            consumer_price,
            transfer,
            unearned,
            point,
        }
    }

    /// The point at x on the line under the envelope's technique there.
    pub fn at(&self, x: f64) -> HouseholdPoint {
        self.wrap(self.parcels.at(x))
    }

    /// The point at x on the line under `technique`, r = 1 (docs/unit-1f.md §5.1).
    pub fn at_with(&self, x: f64, technique: usize) -> HouseholdPoint {
        self.wrap(self.parcels.at_with(x, technique))
    }

    /// The point at x with the pool's wage v given (a corner's), r = 1.
    pub fn at_wage(&self, x: f64, v: f64, technique: usize) -> HouseholdPoint {
        self.wrap(self.parcels.at_wage(x, v, technique))
    }

    /// A point of the idle stretch with `market_land` in use (docs/unit-1e.md §4.6).
    pub fn at_idle(&self, market_land: f64, technique: usize) -> HouseholdPoint {
        self.wrap(self.parcels.at_idle(market_land, technique))
    }

    /// A point of the idle stretch with a type at the edge of its reserved shortage.
    pub fn at_idle_edge(&self, market_land: f64, technique: usize, edge: Edge) -> HouseholdPoint {
        self.wrap(self.parcels.at_idle_edge(market_land, technique, edge))
    }

    /// The evaluation at an enclosure point on one of its sides (docs/unit-1e.md §4.7).
    pub fn at_enclosure(&self, point: &EnclosurePoint, side: EnclosureSide) -> HouseholdPoint {
        self.wrap(self.parcels.at_enclosure(point, side))
    }

    /// f_0 = n_D(x = 0) − S(v = 0), the path's start (docs/unit-1f.md §2.9): positive in every
    /// economy without an in-work benefit, +∞ under a CES basket that weighs a category free at
    /// v = 0.
    pub fn start(&self) -> Result<f64, SolveError> {
        self.parcels.start_value()
    }

    /// Classifies the economy and solves it (docs/unit-1f.md §5.3), with 1e's scan of
    /// [`EXIT_SCAN`](crate::EXIT_SCAN) points per piece where a type takes plots, and every piece
    /// at ρ > 0 under a CES basket or a Dividend closure whose transfer moves with the point.
    ///
    /// 1e's solve with the participation rule of §4.4 and the basket at the point's prices, from
    /// the evaluated start: a start that is not in excess demand with no change of side after it
    /// is [`SolveError::SurplusLabour`]. The equilibrium is reported with the basket, the
    /// government's accounts, the households' and the three-taxes ledger ([`Eq1f`]). With the
    /// fixed basket and [`Government::none`] it is unit 1e's solve bit for bit, every equilibrium,
    /// error and count (§2.14).
    pub fn solve(&self) -> Result<Regime<Eq1f>, SolveError> {
        self.solve_scanned(crate::EXIT_SCAN)
    }

    /// [`solve`](Self::solve) with `scan` interior points per scanned piece, for the gate: the
    /// equilibrium does not depend on it bit for bit, only `scan_points` does.
    pub fn solve_scanned(&self, scan: usize) -> Result<Regime<Eq1f>, SolveError> {
        match self.parcels.solve_scanned(scan)? {
            Regime::Interior(eq) => {
                let f_start = self.start()?;
                let eq = self.report(*eq, f_start);
                if let Some(error) = first_non_finite(&eq) {
                    return Err(error);
                }
                Ok(Regime::Interior(Box::new(eq)))
            }
            Regime::NotViable { d_at_1 } => Ok(Regime::NotViable { d_at_1 }),
            Regime::BoundaryNoMargin { f_at_1 } => Ok(Regime::BoundaryNoMargin { f_at_1 }),
            Regime::NoInteriorAtZero { f_at_0 } => Ok(Regime::NoInteriorAtZero { f_at_0 }),
        }
    }

    /// docs/unit-1f.md §4.6-4.9 at an equilibrium of unit 1e's report under the rule: the
    /// basket, the budget, the accounts and the ledger.
    fn report(&self, base: Eq1e, f_start: f64) -> Eq1f {
        let w = self.parcels.workers();
        let rule = w.rule();
        let g = &self.params.government;
        let b = &base.base;
        let types = &self.params.economy.worker_types;
        let kinds = types.len();
        let people = self.people;
        // The basket at the equilibrium's prices (§4.2).
        let price = b.p_s;
        let consumer_price = rule.consumer * price;
        let content = w.content_of(&b.basket).to_vec();
        let mut euler = 0.0;
        for (c, cat) in content.iter().zip(&b.categories) {
            euler += c * cat.price;
        }
        let shares: Vec<f64> = b.categories.iter().map(|c| c.share).collect();
        // The budget (§4.6).
        let wages = b.wage_bill;
        let rent = base.rent * base.land.market;
        let consumption = b.y * price;
        let transfer = b.transfer;
        let (m_w, m_e) = (rule.work * consumer_price, rule.exit * consumer_price);
        let (mut working, mut exiting) = (0.0, 0.0);
        for (i, eq) in base.workers.iter().enumerate() {
            working += b.workers[i].supply;
            exiting += eq.exiters;
        }
        let program_cost = m_w * working + m_e * exiting;
        let revenue_payroll = g.payroll * wages;
        let revenue_consumption = g.consumption * consumption;
        let transfers = people * transfer;
        let (levy, revenue_rent, rent_tax) = match g.budget {
            Budget::RentRate { .. } => {
                let levy = ((transfers + program_cost) - revenue_payroll) - revenue_consumption;
                (Some(levy), levy, (rent > 0.0).then(|| levy / rent))
            }
            Budget::Dividend { rent_tax } => (None, rent_tax * rent, Some(rent_tax)),
        };
        let within_rent = levy.map(|l| (0.0..=rent).contains(&l));
        let government = GovernmentEq {
            payroll: g.payroll,
            consumption: g.consumption,
            rent_tax,
            levy,
            dividend: transfer,
            dividend_composites: transfer / consumer_price,
            program_work: m_w,
            program_exit: m_e,
            revenue_payroll,
            revenue_rent,
            revenue_consumption,
            consumption_wage_leg: g.consumption * wages,
            consumption_rent_leg: g.consumption * rent,
            consumption_interest_leg: g.consumption * b.interest,
            transfers,
            program_cost,
            within_rent,
            rate_for_one_composite: (base.coverage > 0.0).then(|| 1.0 / base.coverage),
        };
        // The accounts (§4.7), at consumer prices.
        let mut accounts_workers = Vec::with_capacity(kinds);
        let (mut spending, mut support) = (0.0, 0.0);
        for (i, t) in types.iter().enumerate() {
            let unearned = rule.unearned(t.support, consumer_price, transfer);
            let own = rule.support(t.support, consumer_price, transfer);
            let net_wage = rule.net * b.workers[i].wage;
            let (n, e) = (b.workers[i].supply, base.workers[i].exiters);
            let spent = n * ((unearned + m_w) + net_wage) + e * (unearned + m_e);
            spending += spent;
            support += t.workers * own;
            accounts_workers.push(TypeAccount {
                unearned,
                support: own,
                net_wage,
                working: n,
                exiting: e,
                spending: spent,
                composites: spent / consumer_price,
            });
        }
        let receipts = (rent - revenue_rent) + b.interest;
        let provider_spending = receipts - support;
        let provider_composites = provider_spending / consumer_price;
        let provider = ProviderAccount {
            receipts,
            support,
            spending: provider_spending,
            composites: provider_composites,
            funded: provider_composites > 0.0,
        };
        spending += provider_spending;
        let composites = spending / consumer_price;
        let accounts = Accounts {
            workers: accounts_workers,
            provider,
            spending,
            composites,
        };
        // The ledger (§4.9).
        let mut reserved = 0.0;
        for (i, r) in w.reserved_per_basket().iter().enumerate() {
            reserved += b.workers[i].wage * r;
        }
        let flow = b.types.iter().all(|t| t.user_cost == 1.0);
        let rent_share = (base.rent * b.b_s_q) / price;
        let collected = revenue_rent;
        let ledger = Ledger {
            basket_wage_share: (b.v * b.l_s + reserved) / price,
            basket_rent_share: flow.then(|| (base.rent * b.b_s) / price),
            rent_share,
            recirculated_base: rent_share * (consumption - collected),
            multiplier: (rent > 0.0).then(|| 1.0 / (1.0 - rent_share * (collected / rent))),
        };
        let budget = (((revenue_payroll + revenue_rent) + revenue_consumption)
            - (transfers + program_cost))
            .abs()
            / consumption;
        let gross = rule.consumer * consumption;
        let residuals = Residuals1f {
            budget,
            spending: (spending - gross).abs() / gross,
            composites: (composites - b.y).abs() / b.y,
            rent_share: if rent > 0.0 {
                (rent - rent_share * consumption).abs() / rent
            } else {
                0.0
            },
            euler: (euler - price).abs() / price,
        };
        Eq1f {
            basket: BasketEq {
                price,
                consumer_price,
                content,
                shares,
                sigma: w.ces().map(|c| c.sigma),
            },
            government,
            accounts,
            ledger,
            f_start: f_start.is_finite().then_some(f_start),
            residuals,
            base,
        }
    }
}

/// A point of unit 1f's path (docs/unit-1f.md §5.1): unit 1e's point, with the basket at its
/// prices and the households' side.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdPoint {
    /// Unit 1e's point, its supplies under the participation rule.
    pub point: ParcelPoint,
    /// c_j, the composite's content: z, or a CES basket's at the point's prices.
    pub content: Vec<f64>,
    /// P, the composite's producer price.
    pub price: f64,
    /// P^c = (1 + t_c)·P.
    pub consumer_price: f64,
    /// d, the uniform transfer in money.
    pub transfer: f64,
    /// A_i per worker type, its unearned resources in both states.
    pub unearned: Vec<f64>,
}

impl HouseholdPoint {
    /// f = n_D − S, or +∞ where the point is short.
    pub fn excess_demand(&self) -> f64 {
        self.point.excess_demand()
    }
}

/// The basket at an equilibrium (docs/unit-1f.md §4.2).
#[derive(Clone, Debug, PartialEq)]
pub struct BasketEq {
    /// P, the composite's producer price (1e's P_s).
    pub price: f64,
    /// P^c = (1 + t_c)·P.
    pub consumer_price: f64,
    /// c_j, units of category j per composite at the equilibrium's prices.
    pub content: Vec<f64>,
    /// s_j = c_j·p_j/P, each category's share of the composite's cost (SSRN eq 26's for a CES
    /// basket over the good and space).
    pub shares: Vec<f64>,
    /// σ, for a CES basket.
    pub sigma: Option<f64>,
}

/// The government at an equilibrium (docs/unit-1f.md §4.6), in money in the numeraire's units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GovernmentEq {
    /// τ_w.
    pub payroll: f64,
    /// t_c.
    pub consumption: f64,
    /// τ_R: given under Dividend; L_R/R under RentRate, `None` where R = 0.
    pub rent_tax: Option<f64>,
    /// L_R = N·d + M − G_w − G_c, the owners' levy under RentRate.
    pub levy: Option<f64>,
    /// d, the uniform transfer per person.
    pub dividend: f64,
    /// d/P^c.
    pub dividend_composites: f64,
    /// m_w = μ_w·P^c.
    pub program_work: f64,
    /// m_e = μ_e·P^c.
    pub program_exit: f64,
    /// G_w = τ_w·W, W the gross wage bill.
    pub revenue_payroll: f64,
    /// G_R: τ_R·R, or the levy.
    pub revenue_rent: f64,
    /// G_c = t_c·C, C = Y·P.
    pub revenue_consumption: f64,
    /// t_c·W, the consumption tax paid from wages (App. C).
    pub consumption_wage_leg: f64,
    /// t_c·R, from rent.
    pub consumption_rent_leg: f64,
    /// t_c·interest.
    pub consumption_interest_leg: f64,
    /// N·d.
    pub transfers: f64,
    /// M = m_w·Σ_i n_S,i + m_e·Σ_i E_i.
    pub program_cost: f64,
    /// 0 ≤ L_R ≤ R, under RentRate.
    pub within_rent: Option<bool>,
    /// 1/κ, the rent tax that funds one composite per person (SSRN eq 16); `None` at κ = 0.
    pub rate_for_one_composite: Option<f64>,
}

/// One worker type's account (docs/unit-1f.md §4.7), in money at consumer prices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeAccount {
    /// A_i, unearned resources per person in both states.
    pub unearned: f64,
    /// s_i, the provider's support per person.
    pub support: f64,
    /// (1 − τ_w)·v_i.
    pub net_wage: f64,
    /// n_S,i.
    pub working: f64,
    /// E_i = N_i − n_S,i.
    pub exiting: f64,
    /// n_S,i·(A_i + m_w + (1 − τ_w)·v_i) + E_i·(A_i + m_e).
    pub spending: f64,
    /// spending/P^c.
    pub composites: f64,
}

/// The provider's account (docs/unit-1f.md §2.7 and §4.7): it owns the land and the machines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProviderAccount {
    /// R − G_R + interest.
    pub receipts: f64,
    /// Σ_i N_i·s_i.
    pub support: f64,
    /// receipts − support, its own consumption.
    pub spending: f64,
    /// spending/P^c.
    pub composites: f64,
    /// composites > 0: the provider funds the support (1a's flag).
    pub funded: bool,
}

/// The households' accounts (docs/unit-1f.md §4.7).
#[derive(Clone, Debug, PartialEq)]
pub struct Accounts {
    /// Every worker type, in order.
    pub workers: Vec<TypeAccount>,
    /// The provider.
    pub provider: ProviderAccount,
    /// Every household's spending: (1 + t_c)·Y·P at an equilibrium (I3).
    pub spending: f64,
    /// spending/P^c: Y (I4).
    pub composites: f64,
}

/// Three-taxes' ledger at the equilibrium (docs/unit-1f.md §4.9).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ledger {
    /// (v·Lᵖ_s + Σ_i v_i·R_ŷi)/P, the basket's labour claims, price side (T1's φ_w).
    pub basket_wage_share: f64,
    /// r·Bᵖ_s/P, when every u_k = 1 (T1's φ_r).
    pub basket_rent_share: Option<f64>,
    /// φ^q_r = r·B^q_s/P: R = φ^q_r·C (T5).
    pub rent_share: f64,
    /// R₀ = φ^q_r·(C − G_R), the rent the rest of spending generates.
    pub recirculated_base: f64,
    /// 1/(1 − φ^q_r·G_R/R), T5's circular-flow multiplier: R = R₀ times it; `None` at R = 0.
    pub multiplier: Option<f64>,
}

/// How far unit 1f's accounts are from their identities (docs/unit-1f.md §6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Residuals1f {
    /// |G_w + G_R + G_c − N·d − M|/C.
    pub budget: f64,
    /// |Σ spending − (1 + t_c)·C|/((1 + t_c)·C).
    pub spending: f64,
    /// |Σ composites − Y|/Y.
    pub composites: f64,
    /// |R − φ^q_r·C|/R where R > 0.
    pub rent_share: f64,
    /// |Σ_j c_j·p_j − P|/P.
    pub euler: f64,
}

/// An equilibrium of unit 1f (docs/unit-1f.md §6).
#[derive(Clone, Debug, PartialEq)]
pub struct Eq1f {
    /// Unit 1e's equilibrium under the participation rule, with the basket at its prices. Its
    /// `worker_baskets`, `provider_baskets` and `funded` are units 1a-1e's accounts without a
    /// government, kept for comparison (decision 116); [`Eq1f::accounts`] has the households'.
    pub base: Eq1e,
    /// The basket.
    pub basket: BasketEq,
    /// The government.
    pub government: GovernmentEq,
    /// The households' accounts.
    pub accounts: Accounts,
    /// Three-taxes' ledger.
    pub ledger: Ledger,
    /// f_0, the path's start; `None` where it is +∞.
    pub f_start: Option<f64>,
    /// The residuals unit 1f adds.
    pub residuals: Residuals1f,
}

impl Eq1f {
    /// Every output: unit 1e's keys in 1e's order, then unit 1f's economy keys, each category's
    /// content (`cat<j>.content`) and each worker type's account (under `worker<i>.`).
    ///
    /// This is the one list of outputs: [`HouseholdEconomy::solve`] checks every number in it for
    /// finiteness.
    pub fn outputs(&self) -> Vec<(OutputKey1c, Output1b)> {
        use Output1b::{Flag, Float, Optional};
        let mut list = self.base.outputs();
        let top = |name| OutputKey1c {
            item: Item::Economy,
            name,
        };
        let g = &self.government;
        let a = &self.accounts;
        let l = &self.ledger;
        let r = &self.residuals;
        list.extend([
            (top("basket_price"), Float(self.basket.price)),
            (top("consumer_price"), Float(self.basket.consumer_price)),
            (top("sigma"), Optional(self.basket.sigma)),
            (top("payroll"), Float(g.payroll)),
            (top("consumption"), Float(g.consumption)),
            (top("rent_tax"), Optional(g.rent_tax)),
            (top("levy"), Optional(g.levy)),
            (top("dividend"), Float(g.dividend)),
            (top("dividend_composites"), Float(g.dividend_composites)),
            (top("program_work"), Float(g.program_work)),
            (top("program_exit"), Float(g.program_exit)),
            (top("revenue_payroll"), Float(g.revenue_payroll)),
            (top("revenue_rent"), Float(g.revenue_rent)),
            (top("revenue_consumption"), Float(g.revenue_consumption)),
            (top("consumption_wage_leg"), Float(g.consumption_wage_leg)),
            (top("consumption_rent_leg"), Float(g.consumption_rent_leg)),
            (
                top("consumption_interest_leg"),
                Float(g.consumption_interest_leg),
            ),
            (top("transfers"), Float(g.transfers)),
            (top("program_cost"), Float(g.program_cost)),
            (
                top("within_rent"),
                Optional(g.within_rent.map(|w| if w { 1.0 } else { 0.0 })),
            ),
            (
                top("rate_for_one_composite"),
                Optional(g.rate_for_one_composite),
            ),
            (top("provider_receipts"), Float(a.provider.receipts)),
            (top("provider_support"), Float(a.provider.support)),
            (top("provider_spending"), Float(a.provider.spending)),
            (top("provider_composites"), Float(a.provider.composites)),
            (top("provider_funded"), Flag(a.provider.funded)),
            (top("spending"), Float(a.spending)),
            (top("composites"), Float(a.composites)),
            (top("basket_wage_share"), Float(l.basket_wage_share)),
            (top("basket_rent_share"), Optional(l.basket_rent_share)),
            (top("rent_share"), Float(l.rent_share)),
            (top("recirculated_base"), Float(l.recirculated_base)),
            (top("multiplier"), Optional(l.multiplier)),
            (top("f_start"), Optional(self.f_start)),
            (top("res_budget"), Float(r.budget)),
            (top("res_spending"), Float(r.spending)),
            (top("res_composites"), Float(r.composites)),
            (top("res_rent_share"), Float(r.rent_share)),
            (top("res_euler"), Float(r.euler)),
        ]);
        for (j, &c) in self.basket.content.iter().enumerate() {
            list.push((
                OutputKey1c {
                    item: Item::Category(j),
                    name: "content",
                },
                Float(c),
            ));
        }
        for (i, w) in self.accounts.workers.iter().enumerate() {
            let key = |name| OutputKey1c {
                item: Item::Worker(i),
                name,
            };
            list.extend([
                (key("unearned"), Float(w.unearned)),
                (key("support"), Float(w.support)),
                (key("net_wage"), Float(w.net_wage)),
                (key("spending"), Float(w.spending)),
                (key("composites"), Float(w.composites)),
            ]);
        }
        list
    }
}

/// The error for the first number in [`Eq1f::outputs`] that is NaN or infinite.
fn first_non_finite(eq: &Eq1f) -> Option<SolveError> {
    eq.outputs()
        .into_iter()
        .find_map(|(key, output)| match output {
            Output1b::Float(v) | Output1b::Optional(Some(v)) if !v.is_finite() => {
                Some(match key.item {
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
                    _ => SolveError::NonFinite { what: key.name },
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
    use crate::schedule::PowerSchedule;
    use crate::workers::WorkerParams;

    fn g1() -> HouseholdParams {
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
        HouseholdParams::from_parcels(ParcelParams::from_workers(WorkerParams::from_machines(
            MachineParams::from_categories(CategoryParams::from_one_category(params)),
        )))
    }

    fn worker(support: f64, efficiency: f64) -> WorkerType {
        WorkerType {
            workers: 3.0,
            work_cost: UniformWorkCost { chi_max: 0.7 },
            efficiency,
            support,
        }
    }

    #[test]
    fn the_zero_rule_is_units_1d_and_1e_bit_for_bit() {
        // §9 items 5 and 6: with Rule::none every new operation is an exact no-op.
        let rule = Rule::none();
        for (support, wage, p, exit) in [
            (1.0, 0.5434, 1.3615, 0.0),
            (0.37, 12.3, 7.69, 0.25),
            (2.0, 0.0, 3.0, 0.0),
            (1.2, 1e-300, 1e300, 1e-10),
        ] {
            let t = worker(support, 1.5);
            let (num, den) = rule.terms(&t, wage, p, exit, 0.0);
            assert_eq!(num.to_bits(), (wage - exit).to_bits());
            assert_eq!(den.to_bits(), (support * p + exit).to_bits());
            let omega = wage / p;
            assert_eq!(
                rule.ratio(&t, omega, rule.fixed_delta()).to_bits(),
                ((t.efficiency * omega) / support).to_bits()
            );
            for zeta in [0.0, 0.3, 1.7] {
                assert_eq!(
                    rule.walled_rate(support, zeta, 0.0).to_bits(),
                    (zeta * support).to_bits()
                );
            }
            assert_eq!(rule.transfer(p, 3.0, 10.0, 7.0), 0.0);
            assert_eq!(
                rule.unearned(support, p, 0.0).to_bits(),
                (support * p).to_bits()
            );
        }
        assert_eq!(
            rule.ratio(&worker(1.0, 1.0), f64::INFINITY, 0.0),
            f64::INFINITY
        );
        assert!(!rule.moving_transfer() && !rule.moving_on_idle_land());
    }

    #[test]
    fn the_rule_reads_every_instrument() {
        // §4.4: num = (m_w − m_e) + ((1 − τ_w)·v − (1 + t_c)·e), den = (A + m_e) + (1 + t_c)·e.
        let rule = Rule {
            consumer: 1.25,
            net: 0.9,
            work: 0.2,
            exit: 0.1,
            gap: 0.1,
            replace: false,
            budget: Closure::RentRate { dividend: 0.5 },
        };
        let t = worker(1.2, 1.0);
        let (p, v, e) = (2.0, 3.0, 0.4);
        let d = rule.transfer(p, 0.0, 0.0, 0.0);
        assert_eq!(d, 0.5 * 2.5);
        let (num, den) = rule.terms(&t, v, p, e, d);
        assert_eq!(num, (0.2 * 2.5 - 0.1 * 2.5) + (0.9 * 3.0 - 1.25 * 0.4));
        assert_eq!(den, ((1.2 * 2.5 + d) + 0.1 * 2.5) + 1.25 * 0.4);
        // Replace tops the support up to d, and the provider pays the rest
        let replace = Rule {
            replace: true,
            ..rule.clone()
        };
        assert_eq!(replace.unearned(1.2, 2.5, 1.0), 3.0);
        assert_eq!(replace.unearned(1.2, 2.5, 4.0), 4.0);
        assert_eq!(replace.support(1.2, 2.5, 1.0), 2.0);
        assert_eq!(replace.support(1.2, 2.5, 4.0), 0.0);
        assert_eq!(rule.support(1.2, 2.5, 1.0), 3.0);
        // the corner form and the walled rate agree with the point form: at ratio = ζ the
        // walled wage's supply is its reserved demand
        let zeta = 0.8;
        let c = rule.walled_rate(t.support, zeta, rule.fixed_delta());
        let (num, den) = rule.terms(&t, c * p, p, 0.0, d);
        assert!((num / den - zeta).abs() < 1e-15);
        let omega = 1.7;
        let (num, den) = rule.terms(&t, omega * p, p, 0.0, d);
        assert!((rule.ratio(&t, omega, 0.5) - num / den).abs() < 1e-15);
        // the Dividend closure's residual transfer
        let dividend = Rule {
            budget: Closure::Dividend {
                payroll: 0.1,
                rent_tax: 0.5,
                consumption: 0.25,
                program: 0.1,
                people: 4.0,
            },
            work: 0.1,
            exit: 0.1,
            gap: 0.0,
            ..rule
        };
        let d = dividend.transfer(2.0, 12.0, 8.0, 20.0);
        assert_eq!(
            d,
            (((0.1 * 12.0 + 0.5 * 8.0) + 0.25 * 20.0) - (0.1 * 2.5) * 4.0) / 4.0
        );
        assert!(dividend.moving_transfer() && dividend.moving_on_idle_land());
    }

    #[test]
    fn the_ces_basket() {
        // §4.2: at equal prices the composite is z and P = Σ z_j·p; Euler's P = Σ c_j·p_j and the
        // shares z_j·p_j^(1−σ)/Σ z_k·p_k^(1−σ) at any prices; σ = 1 is the limit of σ → 1; an
        // unweighted category has no content.
        let z = [0.7, 0.0, 0.3, 1.1];
        for sigma in [0.1, 0.5, 1.0, 2.0, 10.0, SIGMA_CEIL] {
            let ces = Ces::new(sigma, &z);
            let (price, content) = ces.at(&[2.0; 4]);
            assert!(
                (price - 2.0 * 2.1).abs() <= 4.0 * f64::EPSILON * price,
                "{sigma}"
            );
            for (c, w) in content.iter().zip(&z) {
                assert!((c - w).abs() <= 8.0 * f64::EPSILON, "{sigma}");
            }
            let p = [1.3, 5.0, 0.2, 7.7];
            let (price, content) = ces.at(&p);
            assert_eq!(content[1], 0.0);
            let mut euler = 0.0;
            for (c, q) in content.iter().zip(&p) {
                euler += c * q;
            }
            assert!((euler - price).abs() <= 1e-13 * price, "{sigma}");
            let mut den = 0.0;
            for (w, q) in z.iter().zip(&p) {
                if *w > 0.0 {
                    den += w * num::pow(*q, 1.0 - sigma);
                }
            }
            for j in [0, 2, 3] {
                let share = content[j] * p[j] / price;
                let want = z[j] * num::pow(p[j], 1.0 - sigma) / den;
                assert!(
                    (share - want).abs() <= 1e-12 * want.max(1e-300),
                    "{sigma} {j}"
                );
            }
        }
        let p = [1.3, 5.0, 0.2, 7.7];
        let at_one = Ces::new(1.0, &z).at(&p);
        for sigma in [1.0 - 1e-9, 1.0 + 1e-9] {
            let near = Ces::new(sigma, &z).at(&p);
            assert!((near.0 - at_one.0).abs() <= 1e-8 * at_one.0);
        }
        // SIGMA_CEIL: at σ = 64 a price ratio of 2^16 keeps every content finite
        let ces = Ces::new(SIGMA_CEIL, &[1.0, 1.0]);
        let (price, content) = ces.at(&[1.0, 65536.0]);
        assert!(price.is_finite() && content.iter().all(|c| c.is_finite() && *c >= 0.0));
        // the labour at the wall's end: the power mean of order 1 − σ, zero labour included
        let ces = Ces::new(0.5, &[0.7, 0.3]);
        let l = ces.labour_at_the_end(&[0.4, 0.0]).unwrap();
        let want = num::pow(0.7 * num::pow(0.4, 0.5), 2.0);
        assert!((l - want).abs() <= 1e-15 * want);
        assert_eq!(ces.labour_at_the_end(&[0.0, 0.0]), None);
        assert_eq!(
            Ces::new(1.0, &[0.7, 0.3]).labour_at_the_end(&[0.4, 0.0]),
            None
        );
    }

    /// An `Eq1f` whose own numbers are 1, 2, 3, … in output order after unit 1e's, with the
    /// `nan_at`-th replaced by NaN (none when 0). A struct literal names every field, so a field
    /// added to `Eq1f` or its parts does not compile here until it is given a number.
    fn numbered(nan_at: usize) -> Eq1f {
        let e = HouseholdEconomy::new(g1()).unwrap();
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
        let (price, consumer_price, sigma) = (n(), n(), n());
        let government = GovernmentEq {
            payroll: n(),
            consumption: n(),
            rent_tax: Some(n()),
            levy: Some(n()),
            dividend: n(),
            dividend_composites: n(),
            program_work: n(),
            program_exit: n(),
            revenue_payroll: n(),
            revenue_rent: n(),
            revenue_consumption: n(),
            consumption_wage_leg: n(),
            consumption_rent_leg: n(),
            consumption_interest_leg: n(),
            transfers: n(),
            program_cost: n(),
            within_rent: Some(true),
            rate_for_one_composite: Some(n()),
        };
        let provider = ProviderAccount {
            receipts: n(),
            support: n(),
            spending: n(),
            composites: n(),
            funded: true,
        };
        let (spending, composites) = (n(), n());
        let ledger = Ledger {
            basket_wage_share: n(),
            basket_rent_share: Some(n()),
            rent_share: n(),
            recirculated_base: n(),
            multiplier: Some(n()),
        };
        let f_start = Some(n());
        let residuals = Residuals1f {
            budget: n(),
            spending: n(),
            composites: n(),
            rent_share: n(),
            euler: n(),
        };
        let content = vec![n(), n()];
        let workers = vec![TypeAccount {
            unearned: n(),
            support: n(),
            net_wage: n(),
            working: 0.0,
            exiting: 0.0,
            spending: n(),
            composites: n(),
        }];
        Eq1f {
            base: real.base.clone(),
            basket: BasketEq {
                price,
                consumer_price,
                content,
                shares: vec![0.5; 2],
                sigma: Some(sigma),
            },
            government,
            accounts: Accounts {
                workers,
                provider,
                spending,
                composites,
            },
            ledger,
            f_start,
            residuals,
        }
    }

    /// The numbers unit 1f adds to `outputs()`, without the within-rent flag.
    fn numbers(eq: &Eq1f) -> Vec<(OutputKey1c, f64)> {
        let skip = eq.base.outputs().len();
        eq.outputs()
            .into_iter()
            .skip(skip)
            .filter(|(key, _)| key.name != "within_rent")
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
        assert_eq!(listed.len(), 3 + 17 + 4 + 2 + 5 + 1 + 5 + 2 + 5);
        let mut keys: Vec<String> = eq.outputs().iter().map(|(k, _)| k.to_string()).collect();
        let all = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), all, "a key is listed twice");
        for key in [
            "cat1.content",
            "worker0.net_wage",
            "levy",
            "res_euler",
            "f_start",
        ] {
            assert!(keys.contains(&key.to_string()), "{key}");
        }
        let flags: Vec<(String, Output1b)> = eq
            .outputs()
            .into_iter()
            .skip(eq.base.outputs().len())
            .filter(|(k, o)| matches!(o, Output1b::Flag(_)) || k.name == "within_rent")
            .map(|(k, o)| (k.to_string(), o))
            .collect();
        assert_eq!(
            flags,
            vec![
                ("within_rent".to_string(), Output1b::Optional(Some(1.0))),
                ("provider_funded".to_string(), Output1b::Flag(true)),
            ]
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
                Item::Category(category) => SolveError::NonFiniteInCategory {
                    category,
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
        // an absent start (+∞) is not a number; 1e's own outputs are checked through it
        let mut eq = numbered(0);
        eq.f_start = None;
        assert_eq!(first_non_finite(&eq), None);
        eq.base.base.v = f64::NAN;
        assert_eq!(
            first_non_finite(&eq),
            Some(SolveError::NonFinite { what: "v" })
        );
    }

    #[test]
    fn the_household_form_is_units_1e_economy() {
        // from_parcels is the fixed basket and no government; the rule set underneath is none.
        let p = g1();
        assert_eq!(p.basket, Basket::Fixed);
        assert_eq!(p.government, Government::none());
        let e = HouseholdEconomy::new(p).unwrap();
        assert_eq!(*e.parcels().workers().rule(), Rule::none());
        assert!(e.parcels().workers().ces().is_none());
        assert_eq!(e.net_factor(), 1.0);
    }
}
