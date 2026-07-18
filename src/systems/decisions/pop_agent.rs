use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::OwnerId,
    order::{Order, OrderSide},
};

/// Units of labour supplied per unit of pop size. Scales with recipe demand side.
const LABOUR_RATE: f64 = 1.0;

/// Maximum wealth drift per tick in either direction (normal conditions).
const MAX_DRIFT: f64 = 3.0;
/// P-gain on balance_ratio = balance / target_balance - 1.
/// target_balance = ema_spending / (1 - sr) (savings target in cash terms).
const KP: f64 = 0.03;
/// D-gain on the tick-to-tick change in balance_ratio.
/// balance_ratio is already smooth (ema_spending denominator), so no further smoothing needed.
const KD: f64 = 0.2;
/// EMA factor for spending smoothing
const SPEND_ALPHA: f64 = 0.3;
const BALANCE_ALPHA: f64 = 0.02;
/// Maximum fraction each substitution allocation can shift per tick.
const MAX_SUB_SHIFT: f64 = 0.05;

pub fn run(state: &SimState, game_data: &GameData, deltas: &mut Vec<StateDelta>, orders: &mut Vec<Order>) {
    for pop in &state.pop_groups {
        let node = game_data.region(pop.region).market_node;
        let currency = game_data.market_node(node).currency_good;

        // ── Labour sell orders ────────────────────────────────────────────────
        if let Some(labour_good) = pop.labour_good {
            let supply = if pop.is_employed {
                pop.size * LABOUR_RATE
            } else {
                pop.size * LABOUR_RATE * pop.last_labour_fill_rate
            };
            if supply > 0.0 {
                orders.push(Order {
                    node,
                    good: labour_good,
                    side: OrderSide::Sell,
                    owner: OwnerId::PopGroup(pop.id),
                    qty: supply,
                });
            }
        }
        let balance = currency.map(|c| state.inventory(pop.inventory).get(c)).unwrap_or(f64::INFINITY);

        // ── Resolve sub_state (initialise from defaults if empty) ────────────
        let sub_state: Vec<Vec<f64>> = if pop.sub_state.len() == game_data.need_categories.len() {
            // Move each category's fractions toward the target, capped by MAX_SUB_SHIFT.
            pop.sub_state.iter().zip(game_data.need_categories.iter())
                .map(|(current, cat)| {
                    let target = cat.target_fractions();
                    if target.len() != current.len() {
                        return target;
                    }
                    let mut new_fracs: Vec<f64> = current.iter().zip(&target)
                        .map(|(&c, &t)| {
                            let shift = (t - c).clamp(-MAX_SUB_SHIFT, MAX_SUB_SHIFT);
                            c + shift
                        })
                        .collect();
                    let sum: f64 = new_fracs.iter().sum();
                    if sum > 1e-12 {
                        for f in new_fracs.iter_mut() { *f /= sum; }
                    }
                    new_fracs
                })
                .collect()
        } else {
            game_data.default_sub_state()
        };

        // ── Step 1: basket at current wealth ─────────────────────────────────
        let wealth = pop.wealth.clamp(0.0, game_data.max_wealth_tier());
        let (basket_qty, basket_cost) = compute_basket(game_data, wealth, pop.size, node, state);

        // ── Step 2: hard downgrade if unaffordable (rare — big events only) ──
        let (wealth, basket_qty, basket_cost) = if currency.is_some() && basket_cost > balance + 1e-9 {
            let mut w = wealth.floor(); // first try the floor integer
            loop {
                let (qty, cost) = compute_basket(game_data, w, pop.size, node, state);
                if cost <= balance + 1e-9 || w <= 0.0 {
                    break (w.max(0.0), qty, cost);
                }
                w -= 1.0;
            }
        } else {
            (wealth, basket_qty, basket_cost)
        };

        // ── Step 3: proportional scale if even wealth=0 basket unaffordable ──
        let scale = if currency.is_some() && basket_cost > balance + 1e-9 {
            balance / basket_cost.max(1e-9)
        } else {
            1.0
        };

        // ── Step 4: post buy orders ───────────────────────────────────────────
        for (cat_idx, cat) in game_data.need_categories.iter().enumerate() {
            let total_qty = basket_qty[cat_idx] * scale;
            print!("Pop {} category {}: wealth={:.2}, base_qty={:.2}, scale={:.2}, final_qty={:.2}",
                pop.id.0, cat.name, wealth, basket_qty[cat_idx], scale, total_qty);
            if total_qty <= 0.0 { continue; }
            for (entry_idx, entry) in cat.entries.iter().enumerate() {
                let qty = total_qty * sub_state[cat_idx][entry_idx];
                if qty > 0.0 {
                    orders.push(Order {
                        node,
                        good: entry.good,
                        side: OrderSide::Buy,
                        owner: OwnerId::PopGroup(pop.id),
                        qty,
                    });
                }
            }
        }

        // ── Step 5: wealth drift ──────────────────────────────────────────────
        let new_wealth = if let Some(_) = currency {
            let sr = pop.savings_target.clamp(0.0, 0.9999);

            let actual_cost = basket_cost * scale;

            // Update EMA of spending; seed from actual_cost on the very first tick.
            let new_ema = if pop.ema_spending < 1e-12 {
                actual_cost
            } else {
                SPEND_ALPHA * actual_cost + (1.0 - SPEND_ALPHA) * pop.ema_spending
            };
            // Update EMA of balance; seed from actual balance on the very first tick.
            let new_ema_balance = if pop.ema_balance < 1e-12 && balance > 1e-12 {
                balance
            } else {
                BALANCE_ALPHA * balance + (1.0 - BALANCE_ALPHA) * pop.ema_balance
            };

            let target_spend = new_ema_balance * (1.0 - sr);
            // No drift signal until the pop has a spending history to compare against.
            // Dividing by near-zero ema causes runaway wealth when a pop can't yet buy goods.
            let (relative_spend_error, drift) = if new_ema < 1e-9 {
                (0.0, 0.0)
            } else {
                let rse = (target_spend - new_ema) / new_ema;
                let d = (KP * rse + KD * (rse - pop.prev_spend_error)).clamp(-MAX_DRIFT, MAX_DRIFT);
                (rse, d)
            };
            let new_wealth = (wealth + drift).max(0.0);
            println!("Pop {} wealth drift: {:.3} -> {:.3} (rse={:.3}, drift={:.3}, balance={:.2}, target_spend={:.2}, ema_spend={:.2})",
                pop.id.0, wealth, new_wealth, relative_spend_error, drift, new_ema_balance, target_spend, new_ema);
            deltas.push(StateDelta::SetPopEmaState { pop: pop.id, ema_spending: new_ema, ema_balance: new_ema_balance });
            deltas.push(StateDelta::SetPopPrevSpendError { pop: pop.id, spend_error: relative_spend_error });
            new_wealth
        } else {
            wealth // no currency system → wealth is static
        };

        deltas.push(StateDelta::SetPopWealth { pop: pop.id, wealth: new_wealth });
        deltas.push(StateDelta::SetPopSubState { pop: pop.id, sub_state });
    }
}

/// Compute (qty_per_category, total_basket_cost) for a pop at the given wealth level.
/// qty[cat_idx] is the total units desired (all entries combined), not yet split by sub_state.
fn compute_basket(
    game_data: &GameData,
    wealth: f64,
    pop_size: f64,
    node: crate::types::ids::MarketNodeId,
    state: &SimState,
) -> (Vec<f64>, f64) {
    let qty_per_pop = game_data.interpolated_qty(wealth);
    let mut cost = 0.0;

    // Cost uses a weighted average price across entries in each category.
    // (Sub_state fractions determine which goods are bought, so we use them for cost estimation too,
    //  but sub_state is not available here — use uniform weight-proportional fractions.)
    for (cat_idx, cat) in game_data.need_categories.iter().enumerate() {
        let total_qty = qty_per_pop[cat_idx] * pop_size;
        if total_qty <= 0.0 { continue; }
        let target_fracs = cat.target_fractions();
        for (ei, entry) in cat.entries.iter().enumerate() {
            let price = state.price(node, entry.good).max(1e-9);
            cost += total_qty * target_fracs[ei] * price;
        }
    }

    (qty_per_pop.into_iter().map(|q| q * pop_size).collect(), cost)
}
