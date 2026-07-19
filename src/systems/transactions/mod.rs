use std::collections::HashMap;

use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::{GoodId, InventoryId, OwnerId},
    order::{MarketFills, Order, OrderSide},
    provenance::Provenance,
};

/// Phase 3: Transfer goods and settle currency payments.
///
/// Currency is looked up per market node from MarketNodeDef.currency_good.
/// If a node has no currency, goods on that node transfer freely with no payment.
pub fn run(
    state: &SimState,
    game_data: &GameData,
    orders: &[Order],
    fills: &MarketFills,
) -> Vec<StateDelta> {
    let mut deltas = Vec::new();

    // Currency of each order's node; None means free transfer on that node.
    let node_currency = |node| -> Option<GoodId> {
        game_data.market_node(node).currency_good
    };

    // Remaining currency balance per pop, keyed by (pop_id, currency_good_id).
    // Pops only buy on their own region's node, so one currency per pop in practice.
    let mut pop_currency: HashMap<(u32, u32), f64> = HashMap::new();
    for pop in &state.pop_groups {
        let node = game_data.region(pop.region).market_node;
        if let Some(cid) = node_currency(node) {
            let balance = state.inventory(pop.inventory).get(cid).max(0.0);
            pop_currency.insert((pop.id.0, cid.0), balance);
        }
    }

    // Resolve order owner to the InventoryId that receives/loses goods.
    // Returns None for owners with no real inventory (Government, MagicProducer).
    let owner_inv = |owner: OwnerId| -> Option<InventoryId> {
        match owner {
            OwnerId::Building(bid) => Some(state.building(bid).inventory),
            OwnerId::RecipeInstance(rid) => Some(state.recipe_instance(rid).input_inv),
            OwnerId::PopGroup(pid) => Some(state.pop_groups[pid.idx()].inventory),
            OwnerId::Government(_) | OwnerId::MagicProducer(_) => None,
        }
    };

    // Goods that pops supply as labour. Pops hold no labour stock: it is minted
    // against their headcount at the point of sale (Provenance::LabourMint) and
    // burned the same tick by production or by Instant spoilage. Membership is
    // only ever queried, never iterated, so the hash order never reaches a delta.
    let labour_goods: std::collections::HashSet<GoodId> =
        state.pop_groups.iter().filter_map(|p| p.labour_good).collect();

    // Wages earned by pops from sell orders, keyed by (pop_id, currency_good_id).
    let mut pop_sell_revenue: HashMap<(u32, u32), f64> = HashMap::new();

    let mut instance_currency_delta: HashMap<(u32, u32), f64> = HashMap::new();
    let mut instance_sell_revenue: HashMap<(u32, u32), f64> = HashMap::new();
    let mut instance_buy_cost: HashMap<(u32, u32), f64> = HashMap::new();

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
        let cid = node_currency(order.node);

        match order.side {
            OrderSide::Buy => {
                let actual_qty = match order.owner {
                    OwnerId::PopGroup(pid) => {
                        if let Some(c) = cid {
                            let key = (pid.0, c.0);
                            let remaining = pop_currency.get(&key).copied().unwrap_or(0.0);
                            let affordable = if price > 1e-9 { remaining / price } else { cleared_qty };
                            let qty = cleared_qty.min(affordable.max(0.0));
                            if let Some(r) = pop_currency.get_mut(&key) {
                                *r = (*r - price * qty).max(0.0);
                            }
                            qty
                        } else {
                            cleared_qty
                        }
                    }
                    OwnerId::Building(bid) => {
                        if let Some(c) = cid {
                            let key = (bid.0, c.0);
                            let cost = price * cleared_qty;
                            *instance_currency_delta.entry(key).or_insert(0.0) -= cost;
                            *instance_buy_cost.entry(key).or_insert(0.0) += cost;
                        }
                        cleared_qty
                    }
                    OwnerId::RecipeInstance(rid) => {
                        if let Some(c) = cid {
                            let key = (rid.0, c.0);
                            let cost = price * cleared_qty;
                            *instance_currency_delta.entry(key).or_insert(0.0) -= cost;
                            *instance_buy_cost.entry(key).or_insert(0.0) += cost;
                        }
                        cleared_qty
                    }
                    _ => cleared_qty,
                };
                if actual_qty > 0.0 {
                    if let Some(inv) = owner_inv(order.owner) {
                        deltas.push(StateDelta::AddToInventory {
                            inv,
                            good: order.good,
                            qty: actual_qty,
                            life: game_data.good(order.good).shelf_life.initial_life(),
                            // Buying labour mints it; every other purchase is the
                            // receiving half of a conserved transfer.
                            prov: if labour_goods.contains(&order.good) {
                                Provenance::LabourMint
                            } else {
                                Provenance::Transfer
                            },
                        });
                    }
                }
            }
            OrderSide::Sell => {
                match order.owner {
                    OwnerId::Building(bid) => {
                        if let Some(c) = cid {
                            let key = (bid.0, c.0);
                            let revenue = price * cleared_qty;
                            *instance_currency_delta.entry(key).or_insert(0.0) += revenue;
                            *instance_sell_revenue.entry(key).or_insert(0.0) += revenue;
                        }
                    }
                    OwnerId::RecipeInstance(rid) => {
                        if let Some(c) = cid {
                            let key = (rid.0, c.0);
                            let revenue = price * cleared_qty;
                            *instance_currency_delta.entry(key).or_insert(0.0) += revenue;
                            *instance_sell_revenue.entry(key).or_insert(0.0) += revenue;
                        }
                    }
                    OwnerId::PopGroup(pid) => {
                        if let Some(c) = cid {
                            *pop_sell_revenue.entry((pid.0, c.0)).or_insert(0.0) += price * cleared_qty;
                        }
                    }
                    _ => {}
                }
                // A pop selling its labour has no stock to give up — the units are
                // minted straight into the buyer above. Emitting a removal here
                // would clamp to zero and log a phantom shortfall every tick.
                let pop_labour_sale = matches!(order.owner, OwnerId::PopGroup(_))
                    && labour_goods.contains(&order.good);
                if !pop_labour_sale {
                    if let Some(inv) = owner_inv(order.owner) {
                        deltas.push(StateDelta::RemoveFromInventory {
                            inv,
                            good: order.good,
                            qty: cleared_qty,
                            prov: Provenance::Transfer,
                        });
                    }
                }
            }
        }
    }

    // Settle pop currency payments and wage receipts.
    for pop in &state.pop_groups {
        let node = game_data.region(pop.region).market_node;
        if let Some(cid) = node_currency(node) {
            let key = (pop.id.0, cid.0);
            let start = state.inventory(pop.inventory).get(cid).max(0.0);
            let remaining = pop_currency.get(&key).copied().unwrap_or(start);
            let spent = start - remaining;
            if spent > 1e-12 {
                deltas.push(StateDelta::RemoveFromInventory {
                    inv: pop.inventory,
                    good: cid,
                    qty: spent,
                    prov: Provenance::Transfer,
                });
            }
            let wages = pop_sell_revenue.get(&key).copied().unwrap_or(0.0);
            if wages > 1e-12 {
                deltas.push(StateDelta::AddToInventory {
                    inv: pop.inventory,
                    good: cid,
                    qty: wages,
                    life: None, // currency
                    prov: Provenance::Transfer,
                });
            }
        }
    }

    // Settle recipe instance currency receipts/payments.
    // Ordered iteration: HashSet order is nondeterministic and would make the
    // emitted delta stream vary run-to-run, breaking replay/golden-hash equality.
    let all_ikeys: std::collections::BTreeSet<(u32, u32)> =
        instance_currency_delta.keys().copied().collect();
    for (rid_u32, cid_u32) in all_ikeys {
        use crate::types::ids::{GoodId, RecipeInstanceId};
        let rid = RecipeInstanceId(rid_u32);
        let cid = GoodId(cid_u32);
        let delta    = instance_currency_delta.get(&(rid_u32, cid_u32)).copied().unwrap_or(0.0);
        let revenue  = instance_sell_revenue.get(&(rid_u32, cid_u32)).copied().unwrap_or(0.0);
        let buy_cost = instance_buy_cost.get(&(rid_u32, cid_u32)).copied().unwrap_or(0.0);

        let cs = state.recipe_instance(rid).strategy_state.capacity_control();
        let old_balance = cs.map(|s| s.balance).unwrap_or(0.0);
        let new_balance = (old_balance + revenue).min(0.0) - buy_cost;
        let balance_delta = new_balance - old_balance;
        if balance_delta.abs() > 1e-12 {
            deltas.push(StateDelta::AdjustInstanceBalance { instance: rid, amount: balance_delta });
        }
        let inv = state.recipe_instance(rid).input_inv;
        if delta > 1e-12 {
            deltas.push(StateDelta::AddToInventory {
                inv,
                good: cid,
                qty: delta,
                life: None, // currency
                prov: Provenance::Transfer,
            });
        } else if delta < -1e-12 {
            deltas.push(StateDelta::RemoveFromInventory {
                inv,
                good: cid,
                qty: -delta,
                prov: Provenance::Transfer,
            });
        }
    }

    deltas
}
