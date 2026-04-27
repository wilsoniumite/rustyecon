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

/// price_next = price_current * (1 + alpha * clip(imbalance, -1, 1))
pub fn price_next(price_current: f64, alpha: f64, imbalance: f64) -> f64 {
    let clamped = imbalance.clamp(-1.0, 1.0);
    price_current * (1.0 + alpha * clamped)
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
