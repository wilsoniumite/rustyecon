use crate::types::ids::{GoodId, InventoryId, MagicProducerId, MarketNodeId, PopGroupId, PopPairId, RecipeInstanceId};
use crate::types::provenance::Provenance;
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
        inv: InventoryId,
        good: GoodId,
        qty: f64,
        /// Initial lot life. `None` = indefinite. Defaults to `None` for backward-compat
        /// (e.g. events loaded from RON that predate the lots system).
        #[serde(default)]
        life: Option<u32>,
        /// Whether these units are newly minted, and by what mechanism.
        /// Defaults to `Transfer` (conserved) so pre-provenance RON still loads.
        #[serde(default)]
        prov: Provenance,
    },

    RemoveFromInventory {
        inv: InventoryId,
        good: GoodId,
        qty: f64,
        /// Whether these units leave existence, and by what mechanism.
        /// Defaults to `Transfer` (conserved) so pre-provenance RON still loads.
        #[serde(default)]
        prov: Provenance,
    },

    /// Advance shelf-life counters on one inventory and drop lots that expired.
    /// Emitted in the upkeep phase, one per inventory. Routed through the delta
    /// pipeline (rather than mutating inventories directly) so spoilage is part
    /// of the replayable, hashable delta stream like every other mutation.
    SpoilInventory {
        inv: InventoryId,
    },

    // --- RecipeInstance state ---

    SetChosenSize {
        instance: RecipeInstanceId,
        size: f64,
    },

    SetRecipeSize {
        instance: RecipeInstanceId,
        size: f64,
    },

    SetEfficiency {
        instance: RecipeInstanceId,
        efficiency: f64,
    },

    SetTransferTarget {
        instance: RecipeInstanceId,
        target: Option<RecipeInstanceId>,
    },

    /// Remove a recipe instance (on exit). Components should have been released first.
    RemoveRecipeInstance {
        instance: RecipeInstanceId,
    },

    /// Adjust a CapacityControl instance's notional P&L balance by the given signed amount.
    /// Positive = revenue received; negative = input cost incurred.
    AdjustInstanceBalance {
        instance: RecipeInstanceId,
        amount: f64,
    },

    /// Record the margin observed this tick. Only meaningful for CapacityControl.
    SetInstanceLastMargin {
        instance: RecipeInstanceId,
        margin: f64,
    },

    /// Record actual throughput from production this tick. Only meaningful for CapacityControl.
    SetInstanceLastThroughput {
        instance: RecipeInstanceId,
        throughput: f64,
    },

    /// Update smoothed_input_cost for a DividendPayout instance.
    SetDividendSmoothedCost {
        instance: RecipeInstanceId,
        cost: f64,
    },

    /// Set a pop's participation scale π — the fraction of the pair's hours
    /// actually offered to the market. Kernel arm only.
    SetPopParticipation { pop: PopGroupId, participation: f64 },

    /// Update a desk's EMA of its own realized sell fill. Kernel arm only.
    SetDeskFill {
        instance: RecipeInstanceId,
        fill: f64,
    },

    // --- Pop wealth and substitution state ---

    /// Update a pop's floating-point wealth level (drift up/down each tick).
    SetPopWealth { pop: PopGroupId, wealth: f64 },

    /// Replace a pop's substitution-state fractions for all need categories.
    SetPopSubState { pop: PopGroupId, sub_state: Vec<Vec<f64>> },

    /// Update the EMA of actual spending and EMA of balance used by the wealth PD controller.
    SetPopEmaState { pop: PopGroupId, ema_spending: f64, ema_balance: f64 },

    /// Store prev_spend_error for the D-term of the wealth PD controller.
    SetPopPrevSpendError { pop: PopGroupId, spend_error: f64 },

    /// Set the size of one half of a pop pair (employed or unemployed).
    SetPopSize { pop: PopGroupId, size: f64 },

    /// Record the labour fill rate for an unemployed half; used to scale next tick's supply.
    SetPopLabourFillRate { pop: PopGroupId, fill_rate: f64 },

    /// Redistribute sizes between the employed and unemployed halves of a pair
    /// based on the labour market fill rate at their node. Inventory is split proportionally.
    RedistributePopPair {
        pair: PopPairId,
        employed: PopGroupId,
        unemployed: PopGroupId,
        fill_rate: f64,
    },

    /// Update the one-year EMA of the posted price for a (node, good) pair.
    /// Emitted each tick by price_update alongside SetPrice.
    SetPriceEma {
        node: MarketNodeId,
        good: GoodId,
        ema: f64,
    },

    // --- Magic producer (test only) ---

    /// Change a magic producer's fixed supply quantity per tick.
    SetMagicProducerQty {
        id: MagicProducerId,
        qty: f64,
    },

    // --- Tick counter (applied last each tick) ---

    AdvanceTick,
}
