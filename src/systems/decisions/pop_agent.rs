use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::OwnerId,
    order::{Order, OrderSide},
};

/// Maximum wealth drift per tick in either direction (normal conditions).
const MAX_DRIFT: f64 = 5.0;
/// Controls how fast wealth drifts toward the savings target.
/// excess_ratio = remaining/target_remaining − 1 → drift = excess_ratio * DRIFT_RATE, clamped.
const DRIFT_RATE: f64 = 4.0;
/// Maximum fraction each substitution allocation can shift per tick.
const MAX_SUB_SHIFT: f64 = 0.05;

pub fn run(state: &SimState, game_data: &GameData, deltas: &mut Vec<StateDelta>, orders: &mut Vec<Order>) {
    let currency = game_data.currency_good;

    for pop in &state.pop_groups {
        let node = game_data.region(pop.region).market_node;
        let balance = currency.map(|c| pop.inventory.get(c)).unwrap_or(f64::INFINITY);

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

        // ── Step 5: wealth drift from savings target ──────────────────────────
        let new_wealth = if let Some(_) = currency {
            let sr = pop.savings_target.clamp(0.0, 0.9999);
            let actual_cost = basket_cost * scale;
            let remaining = (balance - actual_cost).max(0.0);

            let excess_ratio = if sr > 0.0 && actual_cost > 1e-9 {
                let target_remaining = actual_cost * sr / (1.0 - sr);
                remaining / target_remaining.max(1e-12) - 1.0
            } else if remaining > 1e-9 {
                1.0 // has money, no target → treat as excess
            } else {
                0.0
            };

            let drift = (excess_ratio * DRIFT_RATE).clamp(-MAX_DRIFT, MAX_DRIFT);
            let new_wealth = (wealth + drift).max(0.0);
            println!("Pop {} wealth drift: {:.3} -> {:.3} (excess_ratio={:.3}, drift={:.3}, balance={:.2}, cost={:.2}, sr={:.2})",
                pop.id.0, wealth, new_wealth, excess_ratio, drift, balance, actual_cost, sr);
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
