use crate::state::{GameData, SimState};
use crate::types::{delta::StateDelta, ids::{GoodId, MarketNodeId}};
use crate::systems::clearing::{price_next, price_next_ratio, PriceRule};

/// EMA alpha for the one-year smoothed price (52-tick span at weekly ticks).
const EMA_ALPHA: f64 = 2.0 / 53.0;

/// Phase 6: Apply the imbalance formula to produce next tick's posted prices,
/// and update the one-year EMA of each price. Parallelisable per (good, node).
pub fn run(state: &SimState, game_data: &GameData, rule: PriceRule) -> Vec<StateDelta> {
    let mut deltas = Vec::new();
    for node_idx in 0..game_data.num_nodes() {
        let node = MarketNodeId(node_idx as u32);
        for good_idx in 0..game_data.num_goods() {
            let good = GoodId(good_idx as u32);
            let alpha = game_data.good(good).alpha;
            let current_price = state.price(node, good);
            let next_price = match rule {
                PriceRule::Imbalance => {
                    price_next(current_price, alpha, state.imbalance(node, good))
                }
                PriceRule::Ratio => price_next_ratio(
                    current_price,
                    state.supply(node, good),
                    state.demand(node, good),
                ),
            };
            if (next_price - current_price).abs() > 1e-12 {
                deltas.push(StateDelta::SetPrice { node, good, price: next_price });
            }
            let new_ema = EMA_ALPHA * current_price + (1.0 - EMA_ALPHA) * state.price_ema(node, good);
            if (new_ema - state.price_ema(node, good)).abs() > 1e-12 {
                deltas.push(StateDelta::SetPriceEma { node, good, ema: new_ema });
            }
        }
    }
    deltas
}
