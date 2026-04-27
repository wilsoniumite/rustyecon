use crate::types::ids::GoodId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodDef {
    pub id: GoodId,
    pub name: String,
    /// Price adjustment speed per tick. High for financial goods, low for wages.
    pub alpha: f64,
    pub shelf_life: ShelfLife,
    pub movement_type: MovementType,
    /// Continuous (grain) vs discrete units (ships).
    pub divisible: bool,
    pub storage_cost_per_tick: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShelfLife {
    /// Destroyed at end of tick if unsold.
    Instant,
    /// Spoils after this many ticks.
    Ticks(u32),
    Indefinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementType {
    /// Transport cost scales with distance and infrastructure.
    Physical,
    /// Near-zero transport cost; subject to capital controls.
    Financial,
    /// Cannot cross regional borders (services).
    Local,
}
