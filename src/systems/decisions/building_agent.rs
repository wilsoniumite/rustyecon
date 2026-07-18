use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::OwnerId,
    order::{Order, OrderSide},
    recipe::StrategyKind,
    recipe_instance::StrategyState,
};

pub fn run(state: &SimState, game_data: &GameData, deltas: &mut Vec<StateDelta>, orders: &mut Vec<Order>) {
    for ri in &state.recipe_instances {
        if ri.recipe_size <= 0.0 {
            continue;
        }

        let recipe = game_data.recipe(ri.recipe);

        match &ri.strategy_state {
            StrategyState::CapacityControl(cs) => {
                // Channel operators buy from channel.from and sell to channel.to.
                // All other instances buy and sell at their home region's market node.
                let (buy_node, sell_node) = match ri.channel {
                    Some(ch_id) => {
                        let ch = game_data.channel(ch_id);
                        (ch.from, ch.to)
                    }
                    None => {
                        let home = game_data.region(ri.region).market_node;
                        (home, home)
                    }
                };

                // --- Step 1: Sell proportion (raw undiscounted balance) ---
                let potential_revenue: f64 = recipe.outputs.iter()
                    .map(|o| state.inventory(ri.output_inv).get(o.good) * state.price(sell_node, o.good))
                    .sum();
                let sell_proportion = if cs.balance >= 0.0 {
                    1.0_f64
                } else {
                    let recovery_ratio = potential_revenue / (-cs.balance);
                    if recovery_ratio >= 1.0 {
                        1.0
                    } else if recovery_ratio <= 0.9 {
                        0.0
                    } else {
                        (recovery_ratio - 0.9) / 0.1
                    }
                };

                // --- Step 2: Post sell orders, but never in excess of 120% of demand ---
                for output in &recipe.outputs {
                    let qty = state.inventory(ri.output_inv).get(output.good) * sell_proportion;
                    let demand = state.demand(sell_node, output.good);
                    let max_qty = demand * 1.2;
                    let qty = qty.min(max_qty);
                    if qty > 0.0 {
                        orders.push(Order {
                            node: sell_node,
                            good: output.good,
                            side: OrderSide::Sell,
                            owner: OwnerId::RecipeInstance(ri.id),
                            qty,
                        });
                    }
                }

                // --- Step 3: Discount balance ---
                let discount_amount = cs.balance * -0.02;
                if discount_amount.abs() > 1e-12 {
                    deltas.push(StateDelta::AdjustInstanceBalance {
                        instance: ri.id,
                        amount: discount_amount,
                    });
                }

                // --- Step 5: Capacity ---
                let revenue_per_unit: f64 = recipe.outputs.iter()
                    .map(|o| state.price(sell_node, o.good) * o.qty_per_unit)
                    .sum();
                let cost_per_unit: f64 = recipe.inputs.iter()
                    .map(|i| state.price(buy_node, i.good) * i.qty_per_unit)
                    .sum();
                let margin = revenue_per_unit - cost_per_unit;
                let reference_price = ((revenue_per_unit + cost_per_unit) / 2.0).max(1e-9);
                let relative_margin = margin / reference_price;

                deltas.push(StateDelta::SetInstanceLastMargin { instance: ri.id, margin });

                let chosen = if relative_margin >= 0.05 {
                    let target: f64 = if revenue_per_unit > 1e-12 {
                        recipe.outputs.iter().map(|o| {
                            let revenue_weight = (state.price(sell_node, o.good) * o.qty_per_unit) / revenue_per_unit;
                            let supply = state.supply(sell_node, o.good);
                            let raw_share = if supply < 1e-12 {
                                1.0
                            } else {
                                state.inventory(ri.output_inv).get(o.good) / supply
                            };
                            let market_share = if raw_share <= 1.0 {
                                raw_share.max(0.0)
                            } else if raw_share <= 1.1 {
                                1.0
                            } else if raw_share <= 2.2 {
                                (2.2 - raw_share) / 1.1
                            } else {
                                0.0
                            };
                            let demand = state.demand(sell_node, o.good);
                            let target_output = demand * (market_share + 0.05);
                            let target_recipe_units = if o.qty_per_unit > 1e-12 { target_output / o.qty_per_unit } else { 0.0 };
                            revenue_weight * target_recipe_units
                        }).sum()
                    } else {
                        0.0
                    };
                    (cs.last_throughput * 0.2 + target * 0.8).clamp(0.0, ri.recipe_size)
                } else {
                    const KP: f64 = 0.6;
                    const KD: f64 = 0.2;
                    let margin_delta = margin - cs.last_margin;
                    let p_adj = KP * (relative_margin);
                    let d_adj = KD * (margin_delta / reference_price).clamp(-0.5, 0.5);
                    let anchor = 0.1 * ri.recipe_size + 0.9 * cs.last_throughput;
                    (cs.last_throughput + (anchor * (p_adj + d_adj))).clamp(0.0, ri.recipe_size)
                };

                if (chosen - ri.chosen_size).abs() > 1e-12 {
                    deltas.push(StateDelta::SetChosenSize { instance: ri.id, size: chosen });
                }

                // --- Step 6: Post buy orders ---
                let currency = game_data.market_node(buy_node).currency_good;

                let cash = currency
                    .map(|c| state.inventory(ri.input_inv).get(c).max(0.0))
                    .unwrap_or(f64::INFINITY);

                let total_cost: f64 = recipe.inputs.iter().map(|i| {
                    let d = i.desired(chosen, ri.recipe_size);
                    if Some(i.good) == currency { d } else { d * state.price(buy_node, i.good) }
                }).sum();

                let scale = if total_cost > 1e-12 { (cash / total_cost).min(1.0) } else { 1.0 };

                let gbp_reserved: f64 = recipe.inputs.iter()
                    .filter(|i| Some(i.good) == currency)
                    .map(|i| i.desired(chosen, ri.recipe_size) * scale)
                    .sum();

                let mut cash_remaining = (cash - gbp_reserved).max(0.0);

                for input in &recipe.inputs {
                    if Some(input.good) == currency {
                        continue;
                    }
                    let desired = input.desired(chosen, ri.recipe_size) * scale;
                    let sold = recipe.outputs.iter()
                        .find(|o| o.good == input.good)
                        .map(|o| state.inventory(ri.output_inv).get(o.good) * sell_proportion)
                        .unwrap_or(0.0);
                    let in_inv = (state.inventory(ri.input_inv).get(input.good) - sold).max(0.0);
                    let need_to_buy = (desired - in_inv).max(0.0);
                    if need_to_buy <= 0.0 {
                        continue;
                    }
                    let price = state.price(buy_node, input.good);
                    let affordable = if price > 1e-9 { cash_remaining / price } else { need_to_buy };
                    let qty = need_to_buy.min(affordable);
                    if qty > 0.0 {
                        cash_remaining -= qty * price;
                        orders.push(Order {
                            node: buy_node,
                            good: input.good,
                            side: OrderSide::Buy,
                            owner: OwnerId::RecipeInstance(ri.id),
                            qty,
                        });
                    }
                }
            }

            StrategyState::DividendPayout(ds) => {
                let node = game_data.region(ri.region).market_node;
                let Some(gbp) = game_data.market_node(node).currency_good else { continue; };

                // Sum expected GBP input cost of all CapacityControl instances sharing this inventory.
                // These are the production recipe instances whose operating costs determine how large
                // a cash reserve the building needs.
                let input_cost: f64 = state.recipe_instances.iter()
                    .filter(|other| other.input_inv == ri.input_inv
                        && other.strategy_state.capacity_control().is_some())
                    .map(|other| {
                        let other_recipe = game_data.recipe(other.recipe);
                        other_recipe.inputs.iter()
                            .filter(|inp| inp.good != gbp)
                            .map(|inp| inp.desired(other.chosen_size, other.recipe_size) * state.price(node, inp.good))
                            .sum::<f64>()
                    })
                    .sum();

                // Update smoothed input cost EMA (α = 0.1 ≈ 10-week smoothing).
                let new_smoothed = ds.smoothed_input_cost * 0.9 + input_cost * 0.1;
                deltas.push(StateDelta::SetDividendSmoothedCost { instance: ri.id, cost: new_smoothed });

                let reserve_multiple = match &game_data.recipe(ri.recipe).strategy {
                    StrategyKind::DividendPayout { reserve_multiple } => *reserve_multiple,
                    _ => 26.0,
                };

                let gbp_held = state.inventory(ri.input_inv).get(gbp);
                let reserve = reserve_multiple * new_smoothed;
                let payout = (gbp_held - reserve).max(0.0);

                if (payout - ri.chosen_size).abs() > 1e-12 {
                    deltas.push(StateDelta::SetChosenSize { instance: ri.id, size: payout });
                }
            }

            StrategyState::AlwaysRun => {
                if (ri.chosen_size - ri.recipe_size).abs() > 1e-12 {
                    deltas.push(StateDelta::SetChosenSize { instance: ri.id, size: ri.recipe_size });
                }
            }
        }
    }
}
