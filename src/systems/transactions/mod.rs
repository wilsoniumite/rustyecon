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

    // How a seller sources what it sells. Classified per *order*, not per good:
    // whether units are delivered from stock or created at the point of sale is a
    // property of the seller, and keying it on the good alone would both mis-tag a
    // stock-backed sale of a labour good and miss a phantom seller of an ordinary one.
    #[derive(Clone, Copy, PartialEq)]
    enum Source {
        /// Delivered from the seller's own inventory — a conserved transfer.
        Stock,
        /// A pop selling its own labour. Pops hold no labour stock; it is minted
        /// against their headcount and burned the same tick by production or by
        /// `Instant` spoilage.
        Labour,
        /// A seller with no inventory at all (MagicProducer, Government): phantom
        /// supply, conjured for test scenarios.
        Phantom,
    }

    let seller_source = |owner: OwnerId, good: GoodId| -> Source {
        match owner {
            OwnerId::PopGroup(pid)
                if state.pop_groups[pid.idx()].labour_good == Some(good) =>
            {
                Source::Labour
            }
            other if owner_inv(other).is_none() => Source::Phantom,
            _ => Source::Stock,
        }
    };

    /// Cleared sell volume at one market, split by how the sellers source it.
    #[derive(Default, Clone, Copy)]
    struct Supply {
        stock: f64,
        labour: f64,
        phantom: f64,
    }
    impl Supply {
        fn total(&self) -> f64 {
            self.stock + self.labour + self.phantom
        }
    }

    // ── Pass 0: what is on offer, and where it comes from ─────────────────────
    let mut supply: HashMap<(u32, u32), Supply> = HashMap::new();
    for order in orders {
        if order.side != OrderSide::Sell {
            continue;
        }
        let cleared = order.qty * fills.seller_fill(order.node, order.good);
        if cleared <= 0.0 {
            continue;
        }
        let e = supply.entry((order.node.0, order.good.0)).or_default();
        match seller_source(order.owner, order.good) {
            Source::Stock => e.stock += cleared,
            Source::Labour => e.labour += cleared,
            Source::Phantom => e.phantom += cleared,
        }
    }

    /// How much of `cleared_qty` a buyer drawing on `inv` can actually pay for,
    /// debiting the shared purse. No currency means free transfer on that node.
    fn afford(
        purse: &mut HashMap<(u32, u32), f64>,
        state: &SimState,
        inv: InventoryId,
        cid: Option<GoodId>,
        price: f64,
        cleared_qty: f64,
    ) -> f64 {
        let Some(c) = cid else { return cleared_qty };
        let key = (inv.0, c.0);
        let remaining = *purse
            .entry(key)
            .or_insert_with(|| state.inventory(inv).get(c).max(0.0));
        let affordable = if price > 1e-9 { remaining / price } else { cleared_qty };
        let qty = cleared_qty.min(affordable.max(0.0));
        if let Some(r) = purse.get_mut(&key) {
            *r = (*r - price * qty).max(0.0);
        }
        qty
    }

    // Wages earned by pops from sell orders, keyed by (pop_id, currency_good_id).
    let mut pop_sell_revenue: HashMap<(u32, u32), f64> = HashMap::new();

    let mut instance_currency_delta: HashMap<(u32, u32), f64> = HashMap::new();
    let mut instance_sell_revenue: HashMap<(u32, u32), f64> = HashMap::new();
    let mut instance_buy_cost: HashMap<(u32, u32), f64> = HashMap::new();

    // Currency still unspent this tick, per (inventory, currency good).
    //
    // Keyed by *inventory*, not by owner: several recipe instances are wired to
    // the same building inventory, so they share one purse. Sizing each of them
    // against the full opening balance let them collectively commit to more than
    // existed, and the currency removal then clamped — money credited to sellers
    // that was never actually paid.
    let mut purse: HashMap<(u32, u32), f64> = HashMap::new();

    // Goods actually received by buyers, and goods offered by sellers at the
    // cleared rate, per (node, good).
    //
    // A buyer's cash cap can hold it below its cleared quantity, so how much
    // really changes hands is not known until every buyer has been resolved.
    // Sellers are therefore settled in a second pass against these totals —
    // previously they shipped and were paid for the full cleared quantity while
    // the buyer received less, and the difference simply vanished.
    let mut bought: HashMap<(u32, u32), f64> = HashMap::new();

    // Currency paid to sellers that have no inventory to receive it, keyed by
    // (buyer inventory, currency good).
    //
    // A phantom seller mints the goods it sells *and* swallows the payment: the
    // money leaves the economy at the point of sale. That is a real burn and has
    // to be declared as one, exactly as the matching goods mint is declared —
    // otherwise the buyer's currency simply disappears with no provenance line.
    let mut phantom_paid: HashMap<(u32, u32), f64> = HashMap::new();

    // ── Pass 1: buyers ────────────────────────────────────────────────────────
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
        let market = (order.node.0, order.good.0);

        match order.side {
            OrderSide::Sell => {}
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
                        let inv = state.building(bid).inventory;
                        let qty = afford(&mut purse, state, inv, cid, price, cleared_qty);
                        if let Some(c) = cid {
                            let key = (bid.0, c.0);
                            let cost = price * qty;
                            *instance_currency_delta.entry(key).or_insert(0.0) -= cost;
                            *instance_buy_cost.entry(key).or_insert(0.0) += cost;
                        }
                        qty
                    }
                    OwnerId::RecipeInstance(rid) => {
                        let inv = state.recipe_instance(rid).input_inv;
                        let qty = afford(&mut purse, state, inv, cid, price, cleared_qty);
                        if let Some(c) = cid {
                            let key = (rid.0, c.0);
                            let cost = price * qty;
                            *instance_currency_delta.entry(key).or_insert(0.0) -= cost;
                            *instance_buy_cost.entry(key).or_insert(0.0) += cost;
                        }
                        qty
                    }
                    _ => cleared_qty,
                };
                if actual_qty > 0.0 {
                    if let Some(inv) = owner_inv(order.owner) {
                        // Attribute the receipt across the market's sources in the
                        // proportions they supplied it. Units traceable to stock are
                        // the receiving half of a transfer; units from a pop's labour
                        // or a phantom seller come into existence here and must say so,
                        // or the ledger sees creation with no provenance line.
                        let s = supply.get(&market).copied().unwrap_or_default();
                        let total = s.total();
                        let life = game_data.good(order.good).shelf_life.initial_life();
                        let mut push = |qty: f64, prov: Provenance| {
                            if qty > 0.0 {
                                deltas.push(StateDelta::AddToInventory {
                                    inv,
                                    good: order.good,
                                    qty,
                                    life,
                                    prov,
                                });
                            }
                        };
                        if total > 0.0 {
                            push(actual_qty * s.stock / total, Provenance::Transfer);
                            push(actual_qty * s.labour / total, Provenance::LabourMint);
                            push(actual_qty * s.phantom / total, Provenance::Magic);
                            // The share of the payment that went to a seller with
                            // nowhere to put it leaves the economy with the goods.
                            if let Some(c) = cid {
                                let lost = price * actual_qty * s.phantom / total;
                                if lost > 0.0 {
                                    *phantom_paid.entry((inv.0, c.0)).or_insert(0.0) += lost;
                                }
                            }
                        } else {
                            push(actual_qty, Provenance::Transfer);
                        }
                        // Only count what a buyer with somewhere to put it actually
                        // received; that is precisely what sellers may ship.
                        *bought.entry(market).or_insert(0.0) += actual_qty;
                    }
                }
            }
        }
    }

    // ── Pass 2: sellers ───────────────────────────────────────────────────────
    // Ship and get paid for exactly what buyers took. When buyers were cash-short
    // the market did not clear at the posted terms, and the unsold remainder stays
    // with the seller — a rationing outcome, not evaporated stock (R8).
    for order in orders {
        if order.side != OrderSide::Sell {
            continue;
        }
        let cleared_qty = order.qty * fills.seller_fill(order.node, order.good);
        if cleared_qty <= 0.0 {
            continue;
        }
        let market = (order.node.0, order.good.0);
        let offered_qty = supply.get(&market).copied().unwrap_or_default().total();
        let taken_qty = bought.get(&market).copied().unwrap_or(0.0);
        // Pro-rata across sellers of this good at this node.
        //
        // Guard only against a non-positive denominator, never an absolute
        // quantity epsilon: a market can carry a large *value* in a tiny
        // quantity once prices diverge (a collapsed region reaches price ~1e18
        // against quantities ~1e-24), and skipping the seller there would leave
        // the buyer's payment with no recipient.
        let fill = if offered_qty > 0.0 {
            (taken_qty / offered_qty).min(1.0)
        } else {
            0.0
        };
        let shipped = cleared_qty * fill;
        if shipped <= 0.0 {
            continue;
        }
        let price = state.price(order.node, order.good);
        let cid = node_currency(order.node);
        let revenue = price * shipped;

        match order.owner {
            OwnerId::Building(bid) => {
                if let Some(c) = cid {
                    let key = (bid.0, c.0);
                    *instance_currency_delta.entry(key).or_insert(0.0) += revenue;
                    *instance_sell_revenue.entry(key).or_insert(0.0) += revenue;
                }
            }
            OwnerId::RecipeInstance(rid) => {
                if let Some(c) = cid {
                    let key = (rid.0, c.0);
                    *instance_currency_delta.entry(key).or_insert(0.0) += revenue;
                    *instance_sell_revenue.entry(key).or_insert(0.0) += revenue;
                }
            }
            OwnerId::PopGroup(pid) => {
                if let Some(c) = cid {
                    *pop_sell_revenue.entry((pid.0, c.0)).or_insert(0.0) += revenue;
                }
            }
            _ => {}
        }

        // Only a stock-backed seller gives anything up. A pop selling its labour,
        // or a seller with no inventory at all, created the units at the point of
        // sale — they were already tagged as minted on the buyer's side. Emitting
        // a removal here would clamp to zero and log a phantom shortfall each tick.
        if seller_source(order.owner, order.good) == Source::Stock {
            if let Some(inv) = owner_inv(order.owner) {
                deltas.push(StateDelta::RemoveFromInventory {
                    inv,
                    good: order.good,
                    qty: shipped,
                    prov: Provenance::Transfer,
                });
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
            // Split off whatever went to a phantom seller: that part is burned,
            // not transferred, and must carry its own provenance.
            let lost = phantom_paid
                .remove(&(pop.inventory.0, cid.0))
                .unwrap_or(0.0)
                .min(spent);
            if lost > 1e-12 {
                deltas.push(StateDelta::RemoveFromInventory {
                    inv: pop.inventory,
                    good: cid,
                    qty: lost,
                    prov: Provenance::Magic,
                });
            }
            let transferred = spent - lost;
            if transferred > 1e-12 {
                deltas.push(StateDelta::RemoveFromInventory {
                    inv: pop.inventory,
                    good: cid,
                    qty: transferred,
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
        // Money paid to a phantom seller is burned, so it leaves separately with
        // its own provenance; `delta` is the net of revenue and *all* costs, so
        // adding the burned part back leaves the genuine transfer component.
        //
        // Taken, not read: instances can share one input inventory, and the entry
        // covers every payment out of it. Whichever instance settles first carries
        // the whole burn, and the nets still sum to the inventory's true change.
        let lost = phantom_paid.remove(&(inv.0, cid.0)).unwrap_or(0.0);
        if lost > 1e-12 {
            deltas.push(StateDelta::RemoveFromInventory {
                inv,
                good: cid,
                qty: lost,
                prov: Provenance::Magic,
            });
        }
        let transferred = delta + lost;
        if transferred > 1e-12 {
            deltas.push(StateDelta::AddToInventory {
                inv,
                good: cid,
                qty: transferred,
                life: None, // currency
                prov: Provenance::Transfer,
            });
        } else if transferred < -1e-12 {
            deltas.push(StateDelta::RemoveFromInventory {
                inv,
                good: cid,
                qty: -transferred,
                prov: Provenance::Transfer,
            });
        }
    }

    deltas
}
