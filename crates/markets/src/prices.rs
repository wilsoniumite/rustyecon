//! The price update (docs/ENGINE.md §3.3; N5, N13, F8, A13), salvaged from `v2p3:
//! systems/price_update/mod.rs` and the rules in `v2p3: systems/clearing/mod.rs:53-141`.
//!
//! What changed as it moved:
//!
//! - The rule and the one-sided behaviour come from the tape (`World::market`), with no default;
//!   July's rule was a command-line switch with a code default (N13), and it treated one-sided
//!   markets by formula (F8).
//! - `Imbalance` is `p·exp(k·x)` with `k = clock.log_step(price_rate)`, which is tick-invariant
//!   (A13); July's `p·(1 + α·x)` agrees with it to first order in `α·x`. The price rate is a
//!   `LogStep` site (amended at S2.2), read through it.
//! - The EMA weight is `clock.weight(ema_time_constant)`, a registered span in years read at use
//!   time through its `Weight` site; July's `EMA_ALPHA = 2/53` was a literal tied to weekly
//!   ticks.
//! - A price or EMA is emitted when its bits change. July's `1e-12` change gates made a price
//!   floor near `1e-12/α` (N5) and a redenomination change real outcomes; they are gone.
//! - A non-finite or non-positive price is an error that stops the run. July's ratio rule kept
//!   the old price instead (`v2p3: clearing/mod.rs:124-132`); there is no floor, ceiling or
//!   fallback.

use rustyecon_core::num::{self, is_clean};
use rustyecon_core::{
    CoreError, Ext, GoodId, NodeId, OneSided, PriceRule, SimState, StateDelta, World,
};
use std::fmt;

/// A price update that cannot be made. It stops the run.
#[derive(Debug, Clone, PartialEq)]
pub enum PriceError {
    /// The next price is not finite or not positive.
    NonFinite {
        /// The node.
        node: NodeId,
        /// The good.
        good: GoodId,
        /// The posted price.
        from: f64,
        /// The price the rule gave.
        to: f64,
    },
    /// The next EMA is not finite or not positive.
    NonFiniteEma {
        /// The node.
        node: NodeId,
        /// The good.
        good: GoodId,
        /// The EMA.
        from: f64,
        /// The EMA the update gave.
        to: f64,
    },
    /// A param or book read that does not fit the world.
    Core(CoreError),
}

impl fmt::Display for PriceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceError::NonFinite {
                node,
                good,
                from,
                to,
            } => write!(
                f,
                "the price of {good} at {node} would move from {from:e} to {to:e}, outside the \
                 finite positive range"
            ),
            PriceError::NonFiniteEma {
                node,
                good,
                from,
                to,
            } => write!(
                f,
                "the price EMA of {good} at {node} would move from {from:e} to {to:e}, outside \
                 the finite positive range"
            ),
            PriceError::Core(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PriceError {}

impl From<CoreError> for PriceError {
    fn from(e: CoreError) -> PriceError {
        PriceError::Core(e)
    }
}

/// The imbalance `x = (D − S)/max(S, D)`, and 0 when both are 0. It lies in [−1, 1] and is
/// exactly ±1 when one side is empty.
pub fn imbalance(supply: f64, demand: f64) -> f64 {
    if supply == 0.0 && demand == 0.0 {
        0.0
    } else {
        (demand - supply) / supply.max(demand)
    }
}

/// One `Imbalance` step: `p·exp(k·x)`.
pub fn step(price: f64, k: f64, x: f64) -> f64 {
    price * num::exp(k * x)
}

/// The next price under a rule, from the posted price, the per-tick log step `k` and the
/// cleared volumes.
///
/// - `Imbalance`: `step(p, k, x)` when both sides or neither side traded. A one-sided market
///   (exactly one of `S` and `D` is 0) steps by `x = ±1` under `Saturate` and keeps `p` under
///   `Hold`.
/// - `Ratio`: `p·(D/S)` when both sides are positive (July's form), else `p`. It ignores `k`,
///   and holds a one-sided market whatever `one_sided` says (`Ratio` with `Saturate` does not
///   load).
pub fn next_price(
    rule: PriceRule,
    one_sided: OneSided,
    price: f64,
    k: f64,
    supply: f64,
    demand: f64,
) -> f64 {
    match rule {
        PriceRule::Imbalance => {
            let one_side = (supply > 0.0) != (demand > 0.0);
            if one_side && one_sided == OneSided::Hold {
                price
            } else {
                step(price, k, imbalance(supply, demand))
            }
        }
        PriceRule::Ratio => {
            if supply > 0.0 && demand > 0.0 {
                price * (demand / supply)
            } else {
                price
            }
        }
    }
}

/// Phase 6: the next posted price and EMA of every market, in (node, good) order, from this
/// tick's volumes. Every rate and the EMA's time constant are read from the current params now.
/// `SetPrice` and `SetEma` are emitted only when the bits change; a price or EMA that leaves the
/// finite positive range is an error, and then nothing is emitted.
pub fn update_prices<E: Ext>(
    s: &SimState<E>,
    w: &World<E>,
) -> Result<Vec<StateDelta<E>>, PriceError> {
    let params = s.params(&w.registry);
    let weight = w.market.ema_time_constant.per_tick(&params, &w.clock)?;
    let mut out = Vec::new();
    for (node, good) in w.markets() {
        let q = s
            .book()
            .quote(node, good)
            .ok_or_else(|| CoreError::Shape(format!("no book slot for ({node}, {good})")))?;
        let rate = w
            .good(good)
            .and_then(|g| g.price_rate)
            .ok_or_else(|| CoreError::Shape(format!("{good} has a market but no price rate")))?;
        let k = rate.per_tick(&params, &w.clock)?;
        let price = next_price(
            w.market.rule,
            w.market.one_sided,
            q.price,
            k,
            q.supply,
            q.demand,
        );
        if !(is_clean(price) && price > 0.0) {
            return Err(PriceError::NonFinite {
                node,
                good,
                from: q.price,
                to: price,
            });
        }
        if price.to_bits() != q.price.to_bits() {
            out.push(StateDelta::SetPrice { node, good, price });
        }
        // The EMA follows this tick's posted price.
        let ema = weight * q.price + (1.0 - weight) * q.ema;
        if !(is_clean(ema) && ema > 0.0) {
            return Err(PriceError::NonFiniteEma {
                node,
                good,
                from: q.ema,
                to: ema,
            });
        }
        if ema.to_bits() != q.ema.to_bits() {
            out.push(StateDelta::SetEma { node, good, ema });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    //! July's five clearing tests (`v2p3: systems/clearing/mod.rs:147-176`), names kept and
    //! retargeted on `p·exp(k·x)`. July's `abs() < 1e-12` checks are bit equality now:
    //! `exp(0) = 1` exactly, and the imbalances are exactly ±0.5.

    use super::*;

    /// July's per-tick alpha, 0.1, as a per-tick log step.
    const K: f64 = 0.1;

    #[test]
    fn price_rises_on_excess_demand() {
        let p = step(1.0, K, 0.5);
        assert!(p > 1.0);
        assert_eq!(p, num::exp(0.05));
    }

    #[test]
    fn price_falls_on_excess_supply() {
        let p = step(1.0, K, -0.5);
        assert!(p < 1.0);
        assert_eq!(p, num::exp(-0.05));
    }

    #[test]
    fn balanced_market_stable() {
        assert_eq!(step(1.0, K, 0.0).to_bits(), 1.0_f64.to_bits());
        assert_eq!(imbalance(7.0, 7.0), 0.0);
        assert_eq!(imbalance(0.0, 0.0), 0.0);
    }

    #[test]
    fn imbalance_excess_demand() {
        // demand 100, supply 50: (100 − 50)/100 = 0.5, exactly.
        assert_eq!(imbalance(50.0, 100.0), 0.5);
        assert_eq!(imbalance(0.0, 3.0), 1.0);
    }

    #[test]
    fn imbalance_excess_supply() {
        // demand 50, supply 100: (50 − 100)/100 = −0.5, exactly.
        assert_eq!(imbalance(100.0, 50.0), -0.5);
        assert_eq!(imbalance(3.0, 0.0), -1.0);
    }
}
