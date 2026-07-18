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

impl ShelfLife {
    /// Convert shelf life to the initial lot life counter.
    /// `Instant` → `Some(0)`, `Ticks(n)` → `Some(n)`, `Indefinite` → `None`.
    pub fn initial_life(&self) -> Option<u32> {
        match self {
            ShelfLife::Instant => Some(0),
            ShelfLife::Ticks(n) => Some(*n),
            ShelfLife::Indefinite => None,
        }
    }
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
