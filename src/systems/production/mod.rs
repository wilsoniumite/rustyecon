use crate::state::{GameData, SimState};
use crate::types::{delta::StateDelta, recipe::InputScaling, recipe_instance::StrategyState};

/// Phase 4: Recipe instances produce output based on chosen_size elected in Phase 1.
/// Inputs are consumed from input_inv. Output is placed into output_inv.
/// Output scales proportionally to input fill: if inputs are short, output is short too.
pub fn run(state: &SimState, game_data: &GameData) -> Vec<StateDelta> {
    let mut deltas = Vec::new();
    for ri in &state.recipe_instances {
        let chosen = ri.chosen_size;
        if chosen <= 0.0 {
            continue;
        }
        let recipe = game_data.recipe(ri.recipe);

        let efficiency = match &ri.strategy_state {
            StrategyState::CapacityControl(cs) => cs.efficiency,
            _ => 1.0,
        };

        // Determine how much of each input is available and compute a common scale factor.
        let mut scale = 1.0f64;
        for input in &recipe.inputs {
            let desired = match input.scaling {
                InputScaling::Variable => input.qty_per_unit * chosen,
                InputScaling::Fixed => input.qty_per_unit * ri.recipe_size,
                InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
            };
            if desired > 0.0 {
                let available = state.inventory(ri.input_inv).get(input.good);
                scale = scale.min(available / desired);
            }
        }
        if scale <= 0.0 {
            continue;
        }

        // Consume inputs from input_inv.
        for input in &recipe.inputs {
            let qty = match input.scaling {
                InputScaling::Variable => input.qty_per_unit * chosen,
                InputScaling::Fixed => input.qty_per_unit * ri.recipe_size,
                InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
            } * scale;
            if qty > 0.0 {
                deltas.push(StateDelta::RemoveFromInventory {
                    inv: ri.input_inv,
                    good: input.good,
                    qty,
                });
            }
        }

        // Produce outputs into output_inv.
        for output in &recipe.outputs {
            let qty = output.qty_per_unit * chosen * efficiency * scale;
            if qty > 0.0 {
                deltas.push(StateDelta::AddToInventory {
                    inv: ri.output_inv,
                    good: output.good,
                    qty,
                    life: game_data.good(output.good).shelf_life.initial_life(),
                });
            }
        }

        let throughput = chosen * scale;
        if throughput > 0.0 {
            deltas.push(StateDelta::SetInstanceLastThroughput {
                instance: ri.id,
                throughput,
            });
        }
    }
    deltas
}
