use std::collections::HashMap;

use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::OwnerId,
    order::{MarketFills, Order, OrderSide},
};

/// Phase 3: Transfer goods and settle currency payments.
///
/// If the scenario defines a currency good, pops pay from their currency inventory
/// and buildings receive currency. Pop delivery is capped at affordable quantity
/// (currency_balance / price) — sequential across basket goods so essentials are
/// funded before luxuries.
/// If no currency good is defined, goods transfer freely with no payment.
pub fn run(
    state: &SimState,
    game_data: &GameData,
    orders: &[Order],
    fills: &MarketFills,
) -> Vec<StateDelta> {
    let mut deltas = Vec::new();
    let currency = game_data.currency_good;

    // Remaining currency per pop this tick (sequential deduction across goods).
    let mut pop_currency: HashMap<u32, f64> = state
        .pop_groups
        .iter()
        .map(|p| {
            let balance = currency.map(|c| p.inventory.get(c)).unwrap_or(f64::INFINITY);
            (p.id.0, balance.max(0.0))
        })
        .collect();

    // Remaining cash per building (for buy orders — capped in Phase 1, but track here too).
    let mut building_currency_delta: HashMap<u32, f64> = HashMap::new();
    let mut building_sell_revenue: HashMap<u32, f64> = HashMap::new();
    let mut building_buy_cost: HashMap<u32, f64> = HashMap::new();

    for order in orders {
        let rate = match order.side {
            OrderSide::Buy => fills.buyer_fill(order.node, order.good),
            OrderSide::Sell => fills.seller_fill(order.node, order.good),
        };
        let cleared_qty = order.qty * rate;
        if cleared_qty <= 0.0 {
            continue;
        }
        let price = state.price(order.node, order.good);

        match order.side {
            OrderSide::Buy => {
                let actual_qty = match order.owner {
                    OwnerId::PopGroup(pid) => {
                        if let Some(_) = currency {
                            let remaining = pop_currency.get_mut(&pid.0).copied().unwrap_or(0.0);
                            let affordable = if price > 1e-9 { remaining / price } else { cleared_qty };
                            let qty = cleared_qty.min(affordable.max(0.0));
                            if let Some(r) = pop_currency.get_mut(&pid.0) {
                                *r = (*r - price * qty).max(0.0);
                            }
                            qty
                        } else {
                            cleared_qty
                        }
                    }
                    OwnerId::Building(bid) => {
                        if currency.is_some() {
                            let cost = price * cleared_qty;
                            *building_currency_delta.entry(bid.0).or_insert(0.0) -= cost;
                            *building_buy_cost.entry(bid.0).or_insert(0.0) += cost;
                        }
                        cleared_qty
                    }
                    _ => cleared_qty,
                };
                if actual_qty > 0.0 {
                    deltas.push(StateDelta::AddToInventory {
                        owner: order.owner,
                        good: order.good,
                        qty: actual_qty,
                    });
                }
            }
            OrderSide::Sell => {
                if let OwnerId::Building(bid) = order.owner {
                    if currency.is_some() {
                        let revenue = price * cleared_qty;
                        *building_currency_delta.entry(bid.0).or_insert(0.0) += revenue;
                        *building_sell_revenue.entry(bid.0).or_insert(0.0) += revenue;
                    }
                }
                deltas.push(StateDelta::RemoveFromInventory {
                    owner: order.owner,
                    good: order.good,
                    qty: cleared_qty,
                });
            }
        }
    }

    // Settle pop currency payments.
    if let Some(cid) = currency {
        for pop in &state.pop_groups {
            let start = pop.inventory.get(cid).max(0.0);
            let remaining = pop_currency.get(&pop.id.0).copied().unwrap_or(start);
            let spent = start - remaining;
            if spent > 1e-12 {
                deltas.push(StateDelta::RemoveFromInventory {
                    owner: OwnerId::PopGroup(pop.id),
                    good: cid,
                    qty: spent,
                });
            }
        }

        // Settle building currency receipts/payments.
        // Balance semantics: sell revenue recovers toward 0; buy cost then reduces unconditionally.
        // new_balance = min(old_balance + sell_revenue, 0) - buy_cost
        let all_bids: std::collections::HashSet<u32> = building_currency_delta.keys().copied().collect();
        for bid_u32 in all_bids {
            use crate::types::ids::BuildingId;
            let bid = BuildingId(bid_u32);
            let delta    = building_currency_delta.get(&bid_u32).copied().unwrap_or(0.0);
            let revenue  = building_sell_revenue.get(&bid_u32).copied().unwrap_or(0.0);
            let buy_cost = building_buy_cost.get(&bid_u32).copied().unwrap_or(0.0);

            let old_balance  = state.building(bid).balance;
            let new_balance  = (old_balance + revenue).min(0.0) - buy_cost;
            let balance_delta = new_balance - old_balance;
            if balance_delta.abs() > 1e-12 {
                deltas.push(StateDelta::AdjustBuildingBalance { building: bid, amount: balance_delta });
            }

            // Cash (GBP inventory) moves by the net amount regardless.
            if delta > 1e-12 {
                deltas.push(StateDelta::AddToInventory {
                    owner: OwnerId::Building(bid),
                    good: cid,
                    qty: delta,
                });
            } else if delta < -1e-12 {
                deltas.push(StateDelta::RemoveFromInventory {
                    owner: OwnerId::Building(bid),
                    good: cid,
                    qty: -delta,
                });
            }
        }
    }

    deltas
}
