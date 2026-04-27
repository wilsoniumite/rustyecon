use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    ids::OwnerId,
    order::{Order, OrderSide},
    recipe::InputScaling,
};

pub fn run(state: &SimState, game_data: &GameData, deltas: &mut Vec<StateDelta>, orders: &mut Vec<Order>) {
    for building in &state.buildings {
        if building.recipe_size <= 0.0 {
            continue;
        }

        let recipe = game_data.recipe(building.recipe);

        // Channel operators buy from channel.from and sell to channel.to.
        // All other buildings buy and sell at their home region's market node.
        let (buy_node, sell_node) = match building.channel {
            Some(ch_id) => {
                let ch = game_data.channel(ch_id);
                (ch.from, ch.to)
            }
            None => {
                let home = game_data.region(building.region).market_node;
                (home, home)
            }
        };

        // --- Step 1: Sell proportion (raw undiscounted balance) ---
        // potential_revenue: what we'd receive if we sold all output inventory now.
        let potential_revenue: f64 = recipe.outputs.iter()
            .map(|o| building.inventory.get(o.good) * state.price(sell_node, o.good))
            .sum();
        let sell_proportion = if building.balance >= 0.0 {
            1.0_f64
        } else {
            // How much of the negative balance would full sales recover?
            let recovery_ratio = potential_revenue / (-building.balance);
            let sell_proportion = if recovery_ratio >= 0.98 {
                1.0
            } else if recovery_ratio <= 0.8 {
                0.0
            } else {
                (recovery_ratio - 0.8) / 0.18
            };
            println!("Building {} balance={:.2}, potential_revenue={:.2}, recovery_ratio={:.2}, sell_proportion={:.2}",
                building.id.0, building.balance, potential_revenue, recovery_ratio, sell_proportion);
            sell_proportion
        };

        // --- Step 2: Post sell orders ---
        for output in &recipe.outputs {
            let qty = building.inventory.get(output.good) * sell_proportion;
            if qty > 0.0 {
                orders.push(Order {
                    node: sell_node,
                    good: output.good,
                    side: OrderSide::Sell,
                    owner: OwnerId::Building(building.id),
                    qty,
                });
            }
        }

        // --- Step 3: Discount balance ---
        let discount_amount = building.balance * -0.02;
        if discount_amount.abs() > 1e-12 {
            deltas.push(StateDelta::AdjustBuildingBalance {
                building: building.id,
                amount: discount_amount,
            });
        }

        // --- Step 5: Capacity (P+D controller anchored to last throughput) ---
        // Margin per unit of recipe_size at current prices.
        let revenue_per_unit: f64 = recipe.outputs.iter()
            .map(|o| state.price(sell_node, o.good) * o.qty_per_unit)
            .sum();
        let cost_per_unit: f64 = recipe.inputs.iter()
            .map(|i| state.price(buy_node, i.good) * i.qty_per_unit)
            .sum();
        let margin = revenue_per_unit - cost_per_unit;
        let reference_price = ((revenue_per_unit + cost_per_unit) / 2.0).max(1e-9);

        const KP_UP: f64 = 0.6; // Overall scaling of P-term.
        const KP_DOWN: f64 = 0.6; // More aggressive when margin is negative.
        let kp = if margin >= 0.0 { KP_UP } else { KP_DOWN };
        const KD: f64 = 0.2;
        let margin_delta = margin - building.last_margin;
        let p_adj = kp * (margin / reference_price);
        let d_adj = KD * (margin_delta / reference_price).clamp(-0.5, 0.5);
        // Blend recipe_size into the anchor so the controller can recover from near-zero throughput.
        let anchor = 0.1 * building.recipe_size + 0.9 * building.last_throughput;
        let chosen = building.last_throughput + (anchor * (p_adj + d_adj))
            .clamp(0.0, building.recipe_size);

        deltas.push(StateDelta::SetBuildingLastMargin { building: building.id, margin });

        if (chosen - building.chosen_size).abs() > 1e-12 {
            deltas.push(StateDelta::SetChosenSize { building: building.id, size: chosen });
        }
        println!("Building {}: margin={:.3}, margin_delta={:.3}, p_adj={:.3}, d_adj={:.3}, last_throughput={:.2}, chosen={:.2}",
            building.id.0, margin, margin_delta, p_adj, d_adj, building.last_throughput, chosen);
        println!("--");

        // --- Step 6: Post buy orders ---
        // Currency inputs (e.g. GBP crossing cost) come from inventory — no market order posted.
        // Total cash budget covers both market buys and currency recipe inputs at face value.
        // Scale everything back if cash-constrained, then net non-currency buys against inventory.
        let currency = game_data.currency_good;

        let cash = currency
            .map(|c| building.inventory.get(c).max(0.0))
            .unwrap_or(f64::INFINITY);

        let total_cost: f64 = recipe.inputs.iter().map(|i| {
            let d = match i.scaling {
                InputScaling::Variable => i.qty_per_unit * chosen,
                InputScaling::Fixed => i.qty_per_unit * building.recipe_size,
                InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
            };
            if Some(i.good) == currency { d } else { d * state.price(buy_node, i.good) }
        }).sum();

        let scale = if total_cost > 1e-12 { (cash / total_cost).min(1.0) } else { 1.0 };

        // GBP reserved for recipe cost — subtracted from cash before market buys.
        let gbp_reserved: f64 = recipe.inputs.iter()
            .filter(|i| Some(i.good) == currency)
            .map(|i| {
                let d = match i.scaling {
                    InputScaling::Variable => i.qty_per_unit * chosen,
                    InputScaling::Fixed => i.qty_per_unit * building.recipe_size,
                    InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
                };
                d * scale
            })
            .sum();

        let mut cash_remaining = (cash - gbp_reserved).max(0.0);

        for input in &recipe.inputs {
            if Some(input.good) == currency {
                continue; // comes from inventory
            }
            let desired = match input.scaling {
                InputScaling::Variable => input.qty_per_unit * chosen,
                InputScaling::Fixed => input.qty_per_unit * building.recipe_size,
                InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
            } * scale;
            let in_inv = building.inventory.get(input.good);
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
                    owner: OwnerId::Building(building.id),
                    qty,
                });
            }
        }
    }
}
