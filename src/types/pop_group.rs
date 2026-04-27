use crate::types::ids::{PopGroupId, RegionId};
use crate::types::inventory::Inventory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopGroup {
    pub id: PopGroupId,
    pub region: RegionId,
    pub size: f64,
    /// Floating-point wealth level. Integer floors map to defined wealth tiers in GameData;
    /// values between integers are linearly interpolated. Drifts each tick based on savings.
    pub wealth: f64,
    pub inventory: Inventory,
    /// Desired savings ratio: after spending, pop wants remaining/spent ≈ savings_target/(1−savings_target).
    /// Deviation from this target drives the wealth drift each tick.
    #[serde(default)]
    pub savings_target: f64,
    /// Substitution state: current allocation fractions within each need category.
    /// `sub_state[cat_idx][entry_idx]` = fraction of that category's demand going to that entry.
    /// Sums to 1.0 within each category. Shifts toward target fractions at most `MAX_SUB_SHIFT` per tick.
    /// Defaults to empty; initialised from GameData on first use.
    #[serde(default)]
    pub sub_state: Vec<Vec<f64>>,
}
