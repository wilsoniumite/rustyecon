//! The type switch at the wall (P2.4; O97's rule, decisions 409–411; the switch scan's §3,
//! docs/probe/switch/SPEC.md; docs/probe/SWITCH-RULES.md): a worker type with reserved tasks and
//! pool efficiency ε > 0 sells a share a of its hours to the pool, at ε efficiency hours an hour,
//! and the rest on its own labour market. Each tick, before it offers, it reads two posted wages,
//! the pool's wage for an hour of its own, e = ε·w, and its own, w_i, and moves a toward the
//! market that pays more (the migration rule):
//!
//! ```text
//! g  = ln(e/w_i)
//! a' = a + share(k·g)·(1 − a)    if g > 0     (a share of its own market's hours moves to the pool)
//! a' = a·exp(k·g)                if g < 0     (a share of its pool hours moves back)
//! a' = a                         if g = 0
//! ```
//!
//! with share(x) = −expm1(−x) and k = rate/tpy. Its participation reads the wage its offered hours
//! earn, v = w_i + a'·(e − w_i); its hours are n = N·min(ln1p(v/P_s)/χ_max, 1); it offers
//! (1 − a')·n on its own market and ε·(a'·n) efficiency hours to the pool.
//!
//! **Its evaluation order is the scan's §3.3, normative**, and the scan's mirror (`wms.py`) makes
//! the same operations: [`switch_split`] is steps 3–6, which the rule and the harness's readout
//! share, so the two cannot part.
//!
//! **What it reads** (R13): the two posted wages, the basket's prices, its own params (ε, k, N,
//! χ_max, the spending rate) and its own state a. No volume, fill, other actor or oracle.
//!
//! **No clamp** (R3): for g > 0, a' is a convex combination of a and 1 with weight share(k·g) in
//! [0, 1); for g < 0, a' = a·e^(k·g) with the factor in (0, 1); so a' stays in [0, 1] by
//! construction, and v is a convex combination of the two posted wages. The only min is F's, the
//! support of χ, as in every workers rule. Both corners are open: at a = 0 and g > 0, a' > 0; at
//! a = 1 and g < 0, a' < 1.
//!
//! **Off.** Without a pool the pop is a `BasketWorkers` and never reaches this module (P2.3's
//! role bit for bit). With ε = 0 the switch is structurally off: the pop decides exactly as P2.3's
//! workers, its state holding a unchanged, and no pool order is placed. At a' = 0 with g < 0 it
//! offers P2.3's hours on its own market and places no pool order, so a type at its wall makes
//! P2.3's orders and deltas but for its state.

use crate::behaviour::{AgentError, Behaviour, Decision, View};
use crate::ext::{ActorState, SwitchWorkersState};
use crate::roles::many::rules::{basket_orders, basket_price, eat};
use crate::roles::many::spec::{BasketWorkers, Pool};
use crate::roles::rules::{param, price, sell, set, Delta};
use rustyecon_core::num;
use rustyecon_core::{Holder, Provenance, StateDelta};

/// g = ln(ε·w/w_i), the log gap between the pool's wage for an hour of the type and its own
/// wage at posted prices (the scan's §3.3 step 3): above 0 the pool pays more. The harness reads a
/// switch pop's gap through this function.
pub fn switch_gap(efficiency: f64, pool_wage: f64, own_wage: f64) -> f64 {
    num::ln((efficiency * pool_wage) / own_wage)
}

/// The pool share's move, the migration rule (the scan's §3.3 step 3): from a at the gap g and
/// the rate k a tick, a + share(k·g)·(1 − a) for g > 0, a·e^(k·g) for g < 0, and a at g = 0.
pub fn switch_move(a: f64, g: f64, k: f64) -> f64 {
    if g > 0.0 {
        a + (-num::expm1(-(k * g))) * (1.0 - a)
    } else if g < 0.0 {
        a * num::exp(k * g)
    } else {
        a
    }
}

/// A switch pop's split at posted prices (the scan's §3.3 steps 3–6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Split {
    /// g = ln(ε·w/w_i).
    pub gap: f64,
    /// a', the pool share after the move.
    pub pool: f64,
    /// v = w_i + a'·(ε·w − w_i), the wage its offered hours earn.
    pub wage: f64,
    /// F = min(ln1p(v/P_s)/χ_max, 1), its participation share.
    pub share: f64,
    /// (1 − a')·n, the hours it offers on its own market, n = N·F.
    pub own: f64,
    /// ε·(a'·n), the efficiency hours it offers to the pool.
    pub pooled: f64,
}

/// The split from numbers, in the scan's §3.3 order: a the pool share it holds, ε, w the pool's
/// wage, w_i its own, k the rate a tick, P_s the basket's price, χ_max and N its heads' hours a
/// tick. The rule calls it with its params and posted prices.
#[allow(clippy::too_many_arguments)]
pub fn switch_split(
    a: f64,
    efficiency: f64,
    pool_wage: f64,
    own_wage: f64,
    k: f64,
    basket_price: f64,
    chi_max: f64,
    heads: f64,
) -> Split {
    let e = efficiency * pool_wage;
    let g = switch_gap(efficiency, pool_wage, own_wage);
    let a1 = switch_move(a, g, k);
    let v = own_wage + a1 * (e - own_wage);
    let f = (num::ln1p(v / basket_price) / chi_max).min(1.0);
    let n = heads * f;
    Split {
        gap: g,
        pool: a1,
        wage: v,
        share: f,
        own: (1.0 - a1) * n,
        pooled: efficiency * (a1 * n),
    }
}

/// A switch pop (a Pop; decision 411): the basket workers with a pool.
#[derive(Debug, Clone)]
pub struct SwitchWorkers {
    workers: BasketWorkers,
    pool: Pool,
}

impl SwitchWorkers {
    /// The switch pop of `workers`, whose pool is `pool`.
    pub fn new(workers: &BasketWorkers, pool: &Pool) -> SwitchWorkers {
        SwitchWorkers {
            workers: workers.clone(),
            pool: *pool,
        }
    }
}

impl Behaviour for SwitchWorkers {
    type Own = SwitchWorkersState;

    fn decide(&self, v: &View<'_, SwitchWorkersState>) -> Result<Decision, AgentError> {
        let p = &self.workers;
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        // Step 1: the posted wages, the basket's price, ε and the pool share held.
        let w = price(v, self.pool.good)?;
        let wi = price(v, p.labour)?;
        let ps = basket_price(v, &p.basket)?;
        let eps = param(v, self.pool.efficiency)?;
        let a = v.own_state.pool;
        let chi = param(v, p.chi_max)?;
        let heads = param(v, p.heads)?;
        let (share, pool, own, pooled) = if eps == 0.0 {
            // Step 2: off. P2.3's workers' hours, N·min(ln1p(w_i/P_s)/χ_max, 1), all on its own
            // market; the pool share held as it was.
            let s = (num::ln1p(wi / ps) / chi).min(1.0);
            (s, a, heads * s, 0.0)
        } else {
            // Steps 3–6.
            let k = param(v, self.pool.rate)?;
            let x = switch_split(a, eps, w, wi, k, ps, chi, heads);
            (x.share, x.pool, x.own, x.pooled)
        };
        // Step 7: its own hours minted where positive and offered, as P2.3's are; the pool's
        // efficiency hours minted and offered only where positive, so a type at its wall places
        // no pool order.
        let me = Holder::Actor(v.me);
        if own > 0.0 {
            out.deltas.push(StateDelta::Mint {
                to: me,
                good: p.labour,
                qty: own,
                prov: Provenance::Endowment,
            });
        }
        out.orders.push(sell(v, p.labour, own));
        if pooled > 0.0 {
            out.deltas.push(StateDelta::Mint {
                to: me,
                good: self.pool.good,
                qty: pooled,
                prov: Provenance::Endowment,
            });
            out.orders.push(sell(v, self.pool.good, pooled));
        }
        // Step 8: the baskets, P2.3's.
        let budget = param(v, p.spend)? * dry.get(v.currency);
        basket_orders(v, &p.basket, ps, budget, &mut dry, &mut out)?;
        // Step 9.
        out.deltas.push(set(
            v,
            ActorState::SwitchWorkers(SwitchWorkersState { share, pool }),
        ));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, SwitchWorkersState>) -> Result<Vec<Delta>, AgentError> {
        eat(v, &self.workers.basket)
    }

    fn upkeep(&self, _: &View<'_, SwitchWorkersState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}
