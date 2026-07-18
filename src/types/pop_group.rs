use crate::types::ids::{GoodId, InventoryId, PopGroupId, PopPairId, RegionId};
use serde::{Deserialize, Serialize};

/// One half of a PopPair — either the employed or unemployed cohort.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopGroup {
    pub id: PopGroupId,
    pub region: RegionId,
    pub size: f64,
    /// Floating-point wealth level. Integer floors map to defined wealth tiers in GameData;
    /// values between integers are linearly interpolated. Drifts each tick based on savings.
    pub wealth: f64,
    pub inventory: InventoryId,
    /// Desired savings ratio: after spending, pop wants remaining/spent ≈ savings_target/(1−savings_target).
    pub savings_target: f64,
    /// Substitution state: current allocation fractions within each need category.
    #[serde(default)]
    pub sub_state: Vec<Vec<f64>>,
    #[serde(default)]
    pub ema_spending: f64,
    #[serde(default)]
    pub ema_balance: f64,
    #[serde(default)]
    pub prev_spend_error: f64,
    /// The good this pop supplies as labour. If None, the pop posts no labour sell orders.
    /// Both halves of a pair share the same labour good.
    #[serde(default)]
    pub labour_good: Option<GoodId>,
    /// True for the employed half; false for the unemployed half.
    #[serde(default = "default_true")]
    pub is_employed: bool,
    /// Fill rate of this pop's labour sell orders last tick. Employed half keeps 1.0.
    /// Unemployed half scales its posted supply by this each tick (downward wage stickiness).
    #[serde(default = "default_one")]
    pub last_labour_fill_rate: f64,
}

fn default_true() -> bool { true }
fn default_one() -> f64 { 1.0 }

/// Links an employed PopGroup and its unemployed counterpart.
/// Sizes are redistributed between the two halves each tick after labour clearing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopPair {
    pub id: PopPairId,
    pub employed: PopGroupId,
    pub unemployed: PopGroupId,
}
