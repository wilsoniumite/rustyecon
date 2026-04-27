use crate::types::ids::{BuildingId, ChannelId, OwnerId, RecipeId, RegionId};
use crate::types::inventory::Inventory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingId,
    pub region: RegionId,
    pub recipe: RecipeId,

    /// Maximum throughput, set by components held.
    pub recipe_size: f64,

    /// Elected each tick: in [0, recipe_size].
    /// Negative if this is a reversible recipe running in reverse.
    pub chosen_size: f64,

    /// Output modifier. 1.0 = normal. Decreases with capital vintage decay.
    pub efficiency: f64,

    /// Notional P&L denominated in the currency good. Tracks accumulated input costs
    /// minus accumulated revenue from cleared orders. Decays by 0.95 each tick.
    /// Separate from inventory currency — this is an accounting ledger, not a cash balance.
    #[serde(default)]
    pub balance: f64,

    /// Margin per unit of recipe_size from the previous tick (output revenue minus input cost).
    /// Used by the D-term in the capacity controller to detect worsening/improving conditions.
    #[serde(default)]
    pub last_margin: f64,

    /// Actual throughput (chosen × input fill scale) from the previous tick.
    /// Used as the anchor for the P+D capacity controller. Initialised to recipe_size so
    /// the building starts at full capacity and discovers its operating point from there.
    /// Only updated when throughput > 0, so the last productive level persists if production stops.
    #[serde(default)]
    pub last_throughput: f64,

    pub inventory: Inventory,

    // TODO: component register — the non-market fixed assets held by this building.
    // Design not yet decided: what type represents a component and how is it stored?

    // TODO: decision persistence — we need some per-building state to track
    // sustained unprofitability (for exit) and sustained opportunity (for upsize).
    // The exact fields and logic are undecided.

    /// If set, released components transfer here instead of being dismantled.
    /// This is the "transition" mechanism: source building downsizes while target builds.
    pub transfer_target: Option<BuildingId>,

    /// Fractional ownership. Each entry is (owner, share); shares sum to 1.0.
    pub owners: Vec<(OwnerId, f64)>,

    /// If set, this building is a channel operator on the named channel.
    /// Its buy orders go to channel.from and sell orders go to channel.to.
    pub channel: Option<ChannelId>,
}
