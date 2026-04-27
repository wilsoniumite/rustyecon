use crate::types::ids::{BuildingId, GoodId, MagicProducerId, MarketNodeId, OwnerId, PopGroupId};
use serde::{Deserialize, Serialize};

/// Every mutation the simulation can express. The only path to modifying SimState.
///
/// Systems return Vec<StateDelta>. apply_state_deltas() is the only place
/// SimState is written to. Adding a variant here is the correct way to extend
/// what the simulation can do.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateDelta {
    // --- Market ---

    /// Set the posted price for next tick.
    SetPrice {
        node: MarketNodeId,
        good: GoodId,
        price: f64,
    },

    /// Record the aggregated supply and demand volumes after clearing (used by price update).
    SetMarketVolumes {
        node: MarketNodeId,
        good: GoodId,
        supply: f64,
        demand: f64,
    },

    // --- Inventories ---

    AddToInventory {
        owner: OwnerId,
        good: GoodId,
        qty: f64,
    },

    RemoveFromInventory {
        owner: OwnerId,
        good: GoodId,
        qty: f64,
    },

    // --- Building state ---

    SetChosenSize {
        building: BuildingId,
        size: f64,
    },

    SetRecipeSize {
        building: BuildingId,
        size: f64,
    },

    SetEfficiency {
        building: BuildingId,
        efficiency: f64,
    },

    SetTransferTarget {
        building: BuildingId,
        target: Option<BuildingId>,
    },

    /// Remove a building entirely (on exit). Components should have been released first.
    RemoveBuilding {
        building: BuildingId,
    },

    /// Adjust a building's notional P&L balance by the given signed amount.
    /// Positive = revenue received; negative = input cost incurred.
    AdjustBuildingBalance {
        building: BuildingId,
        amount: f64,
    },

    /// Record the margin observed this tick (output price revenue minus input cost per unit of
    /// recipe_size). Read next tick as last_margin for the D-term capacity adjustment.
    SetBuildingLastMargin {
        building: BuildingId,
        margin: f64,
    },

    /// Record actual throughput (chosen × input fill scale) from production this tick.
    /// Only emitted when throughput > 0, so the last productive level persists across idle ticks.
    SetBuildingLastThroughput {
        building: BuildingId,
        throughput: f64,
    },

    // --- Pop wealth and substitution state ---

    /// Update a pop's floating-point wealth level (drift up/down each tick).
    SetPopWealth { pop: PopGroupId, wealth: f64 },

    /// Replace a pop's substitution-state fractions for all need categories.
    SetPopSubState { pop: PopGroupId, sub_state: Vec<Vec<f64>> },

    // --- Magic producer (test only) ---

    /// Change a magic producer's fixed supply quantity per tick.
    SetMagicProducerQty {
        id: MagicProducerId,
        qty: f64,
    },

    // --- Tick counter (applied last each tick) ---

    AdvanceTick,
}
