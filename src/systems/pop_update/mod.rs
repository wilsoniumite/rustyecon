use crate::state::{GameData, SimState};
use crate::types::{delta::StateDelta, provenance::Provenance};

/// Phase 4 — Pop update.
/// 1. Consume all non-currency, non-labour goods received this tick (pops don't stockpile).
/// 2. Redistribute employed/unemployed sizes based on labour market fill rate.
pub fn run(state: &SimState, game_data: &GameData) -> Vec<StateDelta> {
    let mut deltas = Vec::new();

    for pop in &state.pop_groups {
        let node = game_data.region(pop.region).market_node;
        let currency = game_data.market_node(node).currency_good;
        for (good, qty) in state.inventory(pop.inventory).goods() {
            // Retain currency (held across ticks) and the pop's own labour good.
            if currency.map_or(false, |c| c == good) { continue; }
            if pop.labour_good.map_or(false, |l| l == good) { continue; }
            if qty > 0.0 {
                deltas.push(StateDelta::RemoveFromInventory {
                    inv: pop.inventory,
                    good,
                    qty,
                    // The demand sink: consumed goods leave existence.
                    prov: Provenance::Consumption,
                });
            }
        }
    }

    // Redistribute employed/unemployed sizes based on this tick's labour fill rate.
    for pair in &state.pop_pairs {
        let employed = &state.pop_groups[pair.employed.idx()];
        let labour_good = match employed.labour_good {
            Some(g) => g,
            None => continue,
        };
        let node = game_data.region(employed.region).market_node;
        let supply = state.supply(node, labour_good);
        let demand = state.demand(node, labour_good);
        let fill_rate = if supply > 1e-12 {
            (demand / supply).min(1.0)
        } else {
            1.0
        };
        deltas.push(StateDelta::RedistributePopPair {
            pair: pair.id,
            employed: pair.employed,
            unemployed: pair.unemployed,
            fill_rate,
        });
    }

    deltas
}
