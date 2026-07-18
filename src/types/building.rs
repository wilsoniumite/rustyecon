use crate::types::ids::{BuildingId, InventoryId, RegionId};
use serde::{Deserialize, Serialize};

/// An inventory-holding entity in a region.
/// Owns one inventory; recipe instances reference it by InventoryId.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingId,
    pub region: RegionId,
    pub inventory: InventoryId,
}
