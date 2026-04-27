use crate::types::ids::{MarketNodeId, RegionId};
use serde::{Deserialize, Serialize};

/// Static definition of a market node (in GameData).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketNodeDef {
    pub id: MarketNodeId,
    pub tier: MarketTier,
    /// Only set for Regional nodes.
    pub region: Option<RegionId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketTier {
    Regional,
    National,
    World,
}
