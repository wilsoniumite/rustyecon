use crate::state::sim_state::SimState;
use crate::types::{delta::StateDelta, ids::OwnerId};

/// The only place SimState is mutated. All systems produce StateDelta values;
/// this function applies them. Never call this from within a system function.
pub fn apply_state_deltas(state: &mut SimState, deltas: &[StateDelta]) {
    use StateDelta::*;
    for delta in deltas {
        match delta {
            SetPrice { node, good, price } => {
                state.set_price(*node, *good, *price);
            }
            SetMarketVolumes { node, good, supply, demand } => {
                state.set_supply(*node, *good, *supply);
                state.set_demand(*node, *good, *demand);
            }
            AddToInventory { owner, good, qty } => {
                match owner {
                    OwnerId::Building(id) => state.building_mut(*id).inventory.add(*good, *qty),
                    OwnerId::PopGroup(id) => state.pop_groups[id.idx()].inventory.add(*good, *qty),
                    OwnerId::Government(_) => { /* stub: government inventory */ }
                    OwnerId::MagicProducer(_) => { /* no inventory — goods come from nothing */ }
                }
            }
            RemoveFromInventory { owner, good, qty } => {
                match owner {
                    OwnerId::Building(id) => { state.building_mut(*id).inventory.remove(*good, *qty); }
                    OwnerId::PopGroup(id) => { state.pop_groups[id.idx()].inventory.remove(*good, *qty); }
                    OwnerId::Government(_) => { /* stub */ }
                    OwnerId::MagicProducer(_) => { /* no inventory to remove */ }
                }
            }
            SetChosenSize { building, size } => {
                state.building_mut(*building).chosen_size = *size;
            }
            SetRecipeSize { building, size } => {
                state.building_mut(*building).recipe_size = *size;
            }
            SetEfficiency { building, efficiency } => {
                state.building_mut(*building).efficiency = *efficiency;
            }
            SetTransferTarget { building, target } => {
                state.building_mut(*building).transfer_target = *target;
            }
            RemoveBuilding { building } => {
                // Mark as size 0; actual removal would need index compaction.
                // For now, zero-out the building so it produces nothing.
                state.building_mut(*building).recipe_size = 0.0;
                state.building_mut(*building).chosen_size = 0.0;
            }
            AdjustBuildingBalance { building, amount } => {
                state.building_mut(*building).balance += amount;
            }
            SetBuildingLastMargin { building, margin } => {
                state.building_mut(*building).last_margin = *margin;
            }
            SetBuildingLastThroughput { building, throughput } => {
                state.building_mut(*building).last_throughput = *throughput;
            }
            SetPopWealth { pop, wealth } => {
                state.pop_groups[pop.idx()].wealth = *wealth;
            }
            SetPopSubState { pop, sub_state } => {
                state.pop_groups[pop.idx()].sub_state = sub_state.clone();
            }
            SetMagicProducerQty { id, qty } => {
                state.magic_producer_mut(*id).qty_per_tick = *qty;
            }
            AdvanceTick => {
                state.tick += 1;
            }
        }
        // NOTE: #[non_exhaustive] on StateDelta means external crates can't
        // match without a wildcard, but within this crate the compiler enforces
        // exhaustiveness. Adding a new variant will produce a compile error here
        // — intentional, so we can't forget to handle it.
    }
}
