use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta, provenance::Provenance, recipe::InputScaling,
    recipe_instance::StrategyState,
};

/// True when `good` appears on both sides of the recipe — it is routed through
/// the transformation rather than created or destroyed by it.
///
/// The conservation ledger cannot police stoichiometry across *different* goods
/// (turning wheat into flour is a modelling choice, not an accounting identity),
/// so cross-good transforms are self-declared. A good flowing in and back out is
/// the one case it can police, and must: `dividend` (GBP in, GBP out) and
/// `flour_transport` (flour in, flour out) are pure movements wearing a recipe's
/// clothes, and an unbalanced ratio there would be money printing.
fn passes_through(recipe: &crate::types::recipe::RecipeDef, good: crate::types::ids::GoodId) -> bool {
    recipe.inputs.iter().any(|i| i.good == good) && recipe.outputs.iter().any(|o| o.good == good)
}

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
                    // A good on both sides of the recipe is passing through, not
                    // being transformed: ledger it as a transfer so the two halves
                    // must balance. Otherwise a currency-in/currency-out recipe
                    // (dividend, transport) could mint at any ratio and, because
                    // mint/burn lines are self-declared, drift would stay zero.
                    prov: if passes_through(recipe, input.good) {
                        Provenance::Transfer
                    } else {
                        Provenance::Production
                    },
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
                    // See the input side: a pass-through good is a transfer, so
                    // outputting more of it than was taken in shows up as drift.
                    prov: if passes_through(recipe, output.good) {
                        Provenance::Transfer
                    } else {
                        Provenance::Production
                    },
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
