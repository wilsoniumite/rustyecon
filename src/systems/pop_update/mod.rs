use crate::state::{GameData, SimState};
use crate::types::{delta::StateDelta, ids::OwnerId};

/// Phase 4 — Pop update: consume all goods received this tick.
/// Pops don't stockpile — whatever settled into inventory is consumed now.
/// Wealth and sub_state are updated in pop_agent (decision phase).
pub fn run(state: &SimState, game_data: &GameData) -> Vec<StateDelta> {
    let mut deltas = Vec::new();
    let currency = game_data.currency_good;

    for pop in &state.pop_groups {
        for &(good, qty) in pop.inventory.goods() {
            // Don't drain currency — pops hold it across ticks.
            if currency.map_or(false, |c| c == good) { continue; }
            if qty > 0.0 {
                deltas.push(StateDelta::RemoveFromInventory {
                    owner: OwnerId::PopGroup(pop.id),
                    good,
                    qty,
                });
            }
        }
    }
    deltas
}
