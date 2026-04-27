use crate::state::{GameData, SimState};
use crate::types::{delta::StateDelta, ids::OwnerId, recipe::InputScaling};

/// Phase 4: Buildings produce output based on chosen_size elected in Phase 1.
/// Inputs are consumed from inventory (received via Phase 3 buy orders).
/// Output scales proportionally to input fill: if inputs are short, output is short too.
/// Parallelisable across buildings.
pub fn run(state: &SimState, game_data: &GameData) -> Vec<StateDelta> {
    let mut deltas = Vec::new();
    for building in &state.buildings {
        let chosen = building.chosen_size;
        if chosen <= 0.0 {
            continue;
        }
        let recipe = game_data.recipe(building.recipe);

        // Determine how much of each input is available and compute a common scale factor.
        // If a building has no inputs, scale stays 1.0 (full production).
        let mut scale = 1.0f64;
        for input in &recipe.inputs {
            let desired = match input.scaling {
                InputScaling::Variable => input.qty_per_unit * chosen,
                InputScaling::Fixed => input.qty_per_unit * building.recipe_size,
                InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
            };
            if desired > 0.0 {
                let available = building.inventory.get(input.good);
                scale = scale.min(available / desired);
            }
        }
        // scale is in [0, 1]: the fraction of desired inputs actually on hand.
        if scale <= 0.0 {
            continue;
        }

        // Consume inputs scaled by actual throughput.
        for input in &recipe.inputs {
            let qty = match input.scaling {
                InputScaling::Variable => input.qty_per_unit * chosen,
                InputScaling::Fixed => input.qty_per_unit * building.recipe_size,
                InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
            } * scale;
            if qty > 0.0 {
                deltas.push(StateDelta::RemoveFromInventory {
                    owner: OwnerId::Building(building.id),
                    good: input.good,
                    qty,
                });
            }
        }

        // Produce outputs scaled by the same factor.
        for output in &recipe.outputs {
            let qty = output.qty_per_unit * chosen * building.efficiency * scale;
            if qty > 0.0 {
                deltas.push(StateDelta::AddToInventory {
                    owner: OwnerId::Building(building.id),
                    good: output.good,
                    qty,
                });
            }
        }

        let throughput = chosen * scale;
        if throughput > 0.0 {
            deltas.push(StateDelta::SetBuildingLastThroughput {
                building: building.id,
                throughput,
            });
        }
    }
    deltas
}
