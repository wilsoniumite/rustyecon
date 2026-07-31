use crate::state::SimState;
use crate::types::ids::{GoodId, MarketNodeId};

/// Where a non-finite value was found, for the certificate's B4 detail line.
#[derive(Debug, Clone, PartialEq)]
pub struct NonFinite {
    pub field: &'static str,
    pub index: usize,
}

/// Scan the whole state for NaN or infinity.
///
/// engine.md: **NaN = FAIL**. A metric that cannot be computed fails rather than
/// being quietly skipped, because a NaN that reaches a notebook is indisting-
/// uishable from a number. Infinities are included: they are equally a sign the
/// arithmetic stopped meaning anything, and they propagate just as silently.
///
/// A value *declared missing* under a registered convention is a different thing
/// and is not represented as NaN anywhere in state.
pub fn scan(state: &SimState, num_nodes: usize, num_goods: usize) -> Vec<NonFinite> {
    let mut found = Vec::new();

    for node_idx in 0..num_nodes {
        let node = MarketNodeId(node_idx as u32);
        for good_idx in 0..num_goods {
            let good = GoodId(good_idx as u32);
            let i = node_idx * num_goods + good_idx;
            for (field, v) in [
                ("price", state.price(node, good)),
                ("price_ema", state.price_ema(node, good)),
                ("supply", state.supply(node, good)),
                ("demand", state.demand(node, good)),
            ] {
                if !v.is_finite() {
                    found.push(NonFinite { field, index: i });
                }
            }
        }
    }

    for (i, inv) in state.inventories.iter().enumerate() {
        if inv.goods().iter().any(|(_, q)| !q.is_finite()) {
            found.push(NonFinite { field: "inventory", index: i });
        }
    }

    for (i, p) in state.pop_groups.iter().enumerate() {
        if !p.size.is_finite() || !p.wealth.is_finite() {
            found.push(NonFinite { field: "pop", index: i });
        }
    }

    for (i, ri) in state.recipe_instances.iter().enumerate() {
        if !ri.chosen_size.is_finite() || !ri.recipe_size.is_finite() {
            found.push(NonFinite { field: "recipe_instance", index: i });
        }
    }

    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_state_has_no_non_finite_values() {
        let state = SimState::new(3, 2);
        assert!(scan(&state, 2, 3).is_empty());
    }

    #[test]
    fn a_nan_price_is_found() {
        let mut state = SimState::new(3, 2);
        state.set_price(MarketNodeId(1), GoodId(2), f64::NAN);
        let found = scan(&state, 2, 3);
        assert!(found.iter().any(|f| f.field == "price"), "found: {found:?}");
    }

    #[test]
    fn an_infinite_pop_wealth_is_found() {
        let mut state = SimState::new(1, 1);
        state.pop_groups.push(crate::types::pop_group::PopGroup {
            id: crate::types::ids::PopGroupId(0),
            region: crate::types::ids::RegionId(0),
            size: 1.0,
            wealth: f64::INFINITY,
            inventory: crate::types::ids::InventoryId(0),
            savings_target: 0.0,
            sub_state: Vec::new(),
            ema_spending: 0.0,
            ema_balance: 0.0,
            prev_spend_error: 0.0,
            labour_good: None,
            is_employed: true,
            last_labour_fill_rate: 1.0,
            participation: 1.0,
            parity: None,
        });
        let found = scan(&state, 1, 1);
        assert!(found.iter().any(|f| f.field == "pop"), "found: {found:?}");
    }
}
