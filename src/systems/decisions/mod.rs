use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::OwnerId,
    order::{Order, OrderSide},
};

pub mod building_agent;
pub mod pop_agent;

/// Phase 1: All entities post orders simultaneously, reading only &SimState.
/// All reads are from last tick's prices — no intra-phase ordering dependency.
/// Returns state deltas (e.g. chosen_size updates) and the order book for this tick.
pub fn run(state: &SimState, game_data: &GameData) -> (Vec<StateDelta>, Vec<Order>) {
    let mut deltas = Vec::new();
    let mut orders = Vec::new();

    // Magic producers post a fixed sell order every tick of 105% of demand, up to their limit.
    for mp in &state.magic_producers {
        let order_qty = if mp.qty_per_tick > 0.0 {
            let demand = state.demand(mp.node, mp.good);
            (demand * 1.05).min(mp.qty_per_tick)
        } else {
            0.0
        };
        if order_qty > 0.0 {
            orders.push(Order {
                node: mp.node,
                good: mp.good,
                side: OrderSide::Sell,
                owner: OwnerId::MagicProducer(mp.id),
                qty: order_qty,
            });
        }
    }

    building_agent::run(state, game_data, &mut deltas, &mut orders);
    pop_agent::run(state, game_data, &mut deltas, &mut orders);
    (deltas, orders)
}
