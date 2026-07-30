use crate::certify::ConservationLedger;
use crate::state::sim_state::SimState;
use crate::types::delta::StateDelta;
use crate::types::provenance::Provenance;
use crate::types::recipe_instance::StrategyState;

/// The only place SimState is mutated. All systems produce StateDelta values;
/// this function applies them. Never call this from within a system function.
pub fn apply_state_deltas(state: &mut SimState, deltas: &[StateDelta]) {
    apply_with_ledger(state, deltas, None);
}

/// Apply deltas while posting every inventory movement to the conservation
/// ledger, so the tick can be checked against `observed == declared` (R3).
pub fn apply_state_deltas_ledgered(
    state: &mut SimState,
    deltas: &[StateDelta],
    ledger: &mut ConservationLedger,
) {
    apply_with_ledger(state, deltas, Some(ledger));
}

fn apply_with_ledger(
    state: &mut SimState,
    deltas: &[StateDelta],
    mut ledger: Option<&mut ConservationLedger>,
) {
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
            AddToInventory { inv, good, qty, life, prov } => {
                state.inventories[inv.idx()].add(*good, *qty, *life);
                if let Some(l) = ledger.as_deref_mut() {
                    l.record_add(*good, *qty, *prov);
                }
            }
            RemoveFromInventory { inv, good, qty, prov } => {
                // The actual removed quantity, not the requested one: a clamp must
                // never be silent — it becomes a shortfall ledger line instead.
                let removed = state.inventories[inv.idx()].remove(*good, *qty);
                if let Some(l) = ledger.as_deref_mut() {
                    l.record_remove(*good, *qty, removed, *prov);
                }
            }
            SpoilInventory { inv } => {
                let destroyed = state.inventories[inv.idx()].spoil_lots();
                if let Some(l) = ledger.as_deref_mut() {
                    for (good, qty) in destroyed {
                        l.record_remove(good, qty, qty, Provenance::Spoilage);
                    }
                }
            }
            SetChosenSize { instance, size } => {
                state.recipe_instance_mut(*instance).chosen_size = *size;
            }
            SetRecipeSize { instance, size } => {
                state.recipe_instance_mut(*instance).recipe_size = *size;
            }
            SetEfficiency { instance, efficiency } => {
                if let Some(cs) = state.recipe_instance_mut(*instance).strategy_state.capacity_control_mut() {
                    cs.efficiency = *efficiency;
                }
            }
            SetTransferTarget { instance, target } => {
                state.recipe_instance_mut(*instance).transfer_target = *target;
            }
            RemoveRecipeInstance { instance } => {
                let ri = state.recipe_instance_mut(*instance);
                ri.recipe_size = 0.0;
                ri.chosen_size = 0.0;
            }
            AdjustInstanceBalance { instance, amount } => {
                if let Some(cs) = state.recipe_instance_mut(*instance).strategy_state.capacity_control_mut() {
                    cs.balance += amount;
                }
            }
            SetInstanceLastMargin { instance, margin } => {
                if let Some(cs) = state.recipe_instance_mut(*instance).strategy_state.capacity_control_mut() {
                    cs.last_margin = *margin;
                }
            }
            SetInstanceLastThroughput { instance, throughput } => {
                if let Some(cs) = state.recipe_instance_mut(*instance).strategy_state.capacity_control_mut() {
                    cs.last_throughput = *throughput;
                }
            }
            SetDividendSmoothedCost { instance, cost } => {
                if let StrategyState::DividendPayout(s) = &mut state.recipe_instance_mut(*instance).strategy_state {
                    s.smoothed_input_cost = *cost;
                }
            }
            SetDeskFill { instance, fill } => {
                state.recipe_instance_mut(*instance).last_fill = *fill;
            }
            SetPopWealth { pop, wealth } => {
                state.pop_groups[pop.idx()].wealth = *wealth;
            }
            SetPopSubState { pop, sub_state } => {
                state.pop_groups[pop.idx()].sub_state = sub_state.clone();
            }
            SetPopEmaState { pop, ema_spending, ema_balance } => {
                state.pop_groups[pop.idx()].ema_spending = *ema_spending;
                state.pop_groups[pop.idx()].ema_balance = *ema_balance;
            }
            SetPopPrevSpendError { pop, spend_error } => {
                state.pop_groups[pop.idx()].prev_spend_error = *spend_error;
            }
            SetPopSize { pop, size } => {
                state.pop_groups[pop.idx()].size = *size;
            }
            SetPopLabourFillRate { pop, fill_rate } => {
                state.pop_groups[pop.idx()].last_labour_fill_rate = *fill_rate;
            }
            RedistributePopPair { pair: _, employed, unemployed, fill_rate } => {
                let e_idx = employed.idx();
                let u_idx = unemployed.idx();
                let e_size = state.pop_groups[e_idx].size;
                let u_size = state.pop_groups[u_idx].size;
                let total = e_size + u_size;
                // Per-delta skip: a degenerate pair must not drop the rest of the batch.
                if total <= 0.0 { continue; }

                let new_e = fill_rate * total;
                let new_u = total - new_e;

                let e_inv = state.pop_groups[e_idx].inventory;
                let u_inv = state.pop_groups[u_idx].inventory;

                // Redistribute total inventory proportionally to new sizes.
                // Not posted to the conservation ledger: every good is split as
                // target_e + target_u == total_qty, so the pair's adds and removes
                // cancel exactly and Σ inventory is unchanged.
                // Collect all goods held across both halves.
                let goods: Vec<_> = {
                    let mut g: std::collections::BTreeMap<crate::types::ids::GoodId, f64> =
                        Default::default();
                    for (good, qty) in state.inventories[e_inv.idx()].goods() {
                        *g.entry(good).or_insert(0.0) += qty;
                    }
                    for (good, qty) in state.inventories[u_inv.idx()].goods() {
                        *g.entry(good).or_insert(0.0) += qty;
                    }
                    g.into_iter().collect()
                };
                // Clear both inventories and set new proportional amounts.
                for &(good, total_qty) in &goods {
                    let target_e = if total > 0.0 { total_qty * new_e / total } else { 0.0 };
                    let target_u = total_qty - target_e;
                    let current_e = state.inventories[e_inv.idx()].get(good);
                    let current_u = state.inventories[u_inv.idx()].get(good);
                    // Adjust employed half.
                    if target_e > current_e + 1e-12 {
                        state.inventories[e_inv.idx()].add(good, target_e - current_e, None);
                    } else if current_e > target_e + 1e-12 {
                        state.inventories[e_inv.idx()].remove(good, current_e - target_e);
                    }
                    // Adjust unemployed half.
                    if target_u > current_u + 1e-12 {
                        state.inventories[u_inv.idx()].add(good, target_u - current_u, None);
                    } else if current_u > target_u + 1e-12 {
                        state.inventories[u_inv.idx()].remove(good, current_u - target_u);
                    }
                }
                state.pop_groups[e_idx].size = new_e;
                state.pop_groups[u_idx].size = new_u;
                state.pop_groups[u_idx].last_labour_fill_rate = *fill_rate;
            }
            SetPriceEma { node, good, ema } => {
                state.set_price_ema(*node, *good, *ema);
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
