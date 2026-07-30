use crate::types::ids::{ChannelId, InventoryId, RecipeId, RecipeInstanceId, RegionId};
use serde::{Deserialize, Serialize};

/// Per-instance mutable state for the capacity-control strategy.
/// Saved to checkpoints; defaulted to reasonable values on scenario load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityControlState {
    /// Output modifier. 1.0 = normal. Decreases with capital vintage decay.
    pub efficiency: f64,
    /// Notional P&L in currency units. Tracks accumulated revenue minus input costs.
    /// Decays by 0.95 each tick. Separate from inventory — this is an accounting ledger.
    pub balance: f64,
    /// Margin per unit of recipe_size from the previous tick.
    /// Used by the D-term in the capacity controller.
    pub last_margin: f64,
    /// Actual throughput (chosen × input fill scale) from the previous tick.
    /// Initialised to recipe_size so the building starts at full capacity.
    pub last_throughput: f64,
}

impl CapacityControlState {
    pub fn new(_recipe_size: f64) -> Self {
        Self {
            efficiency: 1.0,
            balance: 0.0,
            last_margin: 0.0,
            last_throughput: 0.0,
        }
    }
}

/// Per-instance mutable state for the dividend-payout strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendPayoutState {
    /// Smoothed (EMA) cost of a tick's worth of inputs at current prices.
    /// Used to compute the target reserve: reserve_multiple × smoothed_input_cost.
    #[serde(default)]
    pub smoothed_input_cost: f64,
}

/// Strategy-specific working memory for a recipe instance.
/// Variant must match the RecipeDef.strategy kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyState {
    CapacityControl(CapacityControlState),
    DividendPayout(DividendPayoutState),
    AlwaysRun,
}

impl StrategyState {
    pub fn capacity_control_mut(&mut self) -> Option<&mut CapacityControlState> {
        match self {
            StrategyState::CapacityControl(s) => Some(s),
            _ => None,
        }
    }

    pub fn capacity_control(&self) -> Option<&CapacityControlState> {
        match self {
            StrategyState::CapacityControl(s) => Some(s),
            _ => None,
        }
    }
}

/// A running instance of a recipe. The computational unit of the simulation.
///
/// Holds no goods directly — input and output inventories are referenced by ID.
/// For regular production buildings, input_inv == output_inv (the building's own inventory).
/// For dividend recipes, input_inv is the building's inventory and output_inv is a pop's inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInstance {
    pub id: RecipeInstanceId,
    pub region: RegionId,
    pub recipe: RecipeId,

    /// Inventory that inputs are consumed from.
    pub input_inv: InventoryId,
    /// Inventory that outputs are produced into.
    pub output_inv: InventoryId,

    /// Maximum throughput. Used differently per strategy:
    /// - CapacityControl: set by components held; constrains chosen_size.
    /// - DividendPayout: effectively unbounded; chosen_size is the payout amount.
    /// - AlwaysRun: chosen_size is pinned to recipe_size every tick.
    pub recipe_size: f64,

    /// Elected each tick by the decision phase. Always in [0, recipe_size]
    /// except for AlwaysRun which may exceed it.
    pub chosen_size: f64,

    /// If set, this instance operates on the named channel.
    /// Its buy orders go to channel.from and sell orders go to channel.to.
    pub channel: Option<ChannelId>,

    /// If set, released components transfer here instead of being dismantled.
    pub transfer_target: Option<RecipeInstanceId>,

    /// Per-strategy working memory.
    pub strategy_state: StrategyState,

    /// EMA of the fill this desk realized on what it posted, in [0, 1].
    ///
    /// Only the kernel arm reads or writes it. Rule 1 gives non-storable outputs
    /// no stock to smooth them, so this plays the buffer band's role — and it is
    /// the one place a posted quantity is keyed to a fill, which kernel.md's own
    /// invariant forbids while its Rule 1 specifies. See `kernel::rule_1_sell`.
    ///
    /// Initialised to 1.0 per kernel.md, and defaulted for tapes and checkpoints
    /// written before the kernel existed: a desk with no fill history assumes it
    /// sold everything, so it opens at full flow rather than silently muted.
    #[serde(default = "default_full_fill")]
    pub last_fill: f64,
}

fn default_full_fill() -> f64 { 1.0 }
