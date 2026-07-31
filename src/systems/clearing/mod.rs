use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::{GoodId, MarketNodeId},
    order::{MarketFills, Order, OrderSide},
};

/// Phase 2: Aggregate orders per (node, good), compute fill rates and imbalances.
/// Nodes are independent — parallelisable with rayon if needed.
pub fn run(
    _state: &SimState,
    game_data: &GameData,
    orders: &[Order],
) -> (Vec<StateDelta>, MarketFills) {
    let num_nodes = game_data.num_nodes();
    let num_goods = game_data.num_goods();
    let slot_count = num_nodes * num_goods;

    let mut supply = vec![0.0f64; slot_count];
    let mut demand = vec![0.0f64; slot_count];

    for order in orders {
        let i = order.node.idx() * num_goods + order.good.idx();
        match order.side {
            OrderSide::Sell => supply[i] += order.qty,
            OrderSide::Buy => demand[i] += order.qty,
        }
    }

    let mut fills = MarketFills::new(num_nodes, num_goods);
    let mut deltas = Vec::with_capacity(slot_count);

    for node_idx in 0..num_nodes {
        let node = MarketNodeId(node_idx as u32);
        for good_idx in 0..num_goods {
            let good = GoodId(good_idx as u32);
            let i = node_idx * num_goods + good_idx;
            let s = supply[i];
            let d = demand[i];

            let buyer_rate = if d > 0.0 { (s / d).min(1.0) } else { 0.0 };
            let seller_rate = if s > 0.0 { (d / s).min(1.0) } else { 0.0 };
            fills.set_buyer_fill(node, good, buyer_rate);
            fills.set_seller_fill(node, good, seller_rate);

            deltas.push(StateDelta::SetMarketVolumes { node, good, supply: s, demand: d });
        }
    }

    (deltas, fills)
}

/// Which price rule a run uses. A/B'd like the agent arm, and for the same
/// reason: one of them is wrong and the certified suite has to say which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum PriceRule {
    /// `p ← p · (1 + α · clamp(imbalance, ±1))` — the shipped rule.
    #[default]
    Imbalance,
    /// `p ← p · d/s` — the clearing rule, with no free parameter.
    Ratio,
}

impl PriceRule {
    pub fn name(&self) -> &'static str {
        match self {
            PriceRule::Imbalance => "imbalance",
            PriceRule::Ratio => "ratio",
        }
    }
}

impl std::str::FromStr for PriceRule {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "imbalance" => Ok(PriceRule::Imbalance),
            "ratio" => Ok(PriceRule::Ratio),
            other => Err(format!("unknown price rule {other:?} (imbalance | ratio)")),
        }
    }
}

/// price_next = price_current * (1 + alpha * clip(imbalance, -1, 1))
///
/// **The normaliser destroys the magnitude the price needs.**
/// `(d − s)/max(d, s)` saturates at ±1, so a market short by 1000× and one
/// short by 10× produce the same step — `α` — and a market mispriced by 1000×
/// needs 73 ticks to get there while everything else moves around it. The same
/// class of mistake as v2 Phase 3.5's retired detectors: a transform that
/// compresses away the quantity being measured.
pub fn price_next(price_current: f64, alpha: f64, imbalance: f64) -> f64 {
    let clamped = imbalance.clamp(-1.0, 1.0);
    price_current * (1.0 + alpha * clamped)
}

/// `p ← p · d/s`, the clearing price for the demand side this engine actually
/// has. **No free parameter, and none is missing.**
///
/// A pop wanting quantity `Q` with cash `C` posts `min(Q, C/p)`, so once its
/// budget binds its demand is a rectangular hyperbola in price — unit elastic.
/// Against a supply posted from own stock, which is vertical, the market clears
/// where `C/p = s`, i.e. at `p* = C/s = p·d/s`. One step, exactly, from any
/// starting price:
///
/// ```text
///     p_{t+1} = p_t · d_t/s_t = p_t · (C/p_t)/s = C/s        (a fixed point)
/// ```
///
/// Where demand is still inelastic (`d = Q`, the pop's budget not yet binding)
/// the rule moves the price by `Q/s` per tick, monotonically, until the budget
/// does bind — so it converges from either regime rather than oscillating.
/// `α` is not tuned away here; it is *absent*, because the step size is
/// determined by the market data instead of by a dial.
///
/// **A market with no trade leaves its price alone.** One-sided markets are the
/// only case needing a decision, and this is a real choice, not an oversight:
/// `d/s` is undefined at `s = 0`, and a market where nothing changed hands
/// produced no evidence about what its price should be. The shipped rule
/// instead marks such a market down by α *every tick forever* — which is what
/// B7 catches as `services pinned at -1.000000`, a price falling without bound
/// against demand that was never there.
pub fn price_next_ratio(price_current: f64, supply: f64, demand: f64) -> f64 {
    if supply <= 0.0 || demand <= 0.0 {
        return price_current;
    }
    let next = price_current * (demand / supply);
    if next.is_finite() && next > 0.0 {
        next
    } else {
        price_current
    }
}

/// imbalance = (demand - supply) / max(demand, supply)
pub fn imbalance(supply: f64, demand: f64) -> f64 {
    if supply <= 0.0 && demand <= 0.0 {
        return 0.0;
    }
    (demand - supply) / supply.max(demand)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_rises_on_excess_demand() {
        let p = price_next(1.0, 0.1, 0.5);
        assert!(p > 1.0);
    }

    #[test]
    fn price_falls_on_excess_supply() {
        let p = price_next(1.0, 0.1, -0.5);
        assert!(p < 1.0);
    }

    #[test]
    fn balanced_market_stable() {
        let p = price_next(1.0, 0.1, 0.0);
        assert!((p - 1.0).abs() < 1e-12);
    }

    #[test]
    fn imbalance_excess_demand() {
        // demand=100, supply=50 -> imbalance = (100-50)/100 = 0.5
        assert!((imbalance(50.0, 100.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn imbalance_excess_supply() {
        // demand=50, supply=100 -> imbalance = (50-100)/100 = -0.5
        assert!((imbalance(100.0, 50.0) + 0.5).abs() < 1e-12);
    }
}
