use crate::state::{GameData, SimState};
use crate::types::{delta::StateDelta, ids::{GoodId, MarketNodeId}};
use crate::systems::clearing::{price_next};

/// Phase 6: Apply the imbalance formula to produce next tick's posted prices.
/// Reads imbalances set during clearing. Parallelisable per (good, node).
pub fn run(state: &SimState, game_data: &GameData) -> Vec<StateDelta> {
    let mut deltas = Vec::new();
    for node_idx in 0..game_data.num_nodes() {
        let node = MarketNodeId(node_idx as u32);
        for good_idx in 0..game_data.num_goods() {
            let good = GoodId(good_idx as u32);
            let alpha = game_data.good(good).alpha;
            let current_price = state.price(node, good);
            let imbalance = state.imbalance(node, good);
            let next_price = price_next(current_price, alpha, imbalance);
            // Don't emit a delta if price is unchanged (common case).
            if (next_price - current_price).abs() > 1e-12 {
                deltas.push(StateDelta::SetPrice { node, good, price: next_price });
            }
        }
    }
    deltas
}
