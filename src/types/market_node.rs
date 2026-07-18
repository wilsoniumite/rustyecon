use crate::types::ids::{GoodId, MarketNodeId, RegionId};
use serde::{Deserialize, Serialize};

/// Static definition of a market node (in GameData).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketNodeDef {
    pub id: MarketNodeId,
    pub tier: MarketTier,
    /// Only set for Regional nodes.
    pub region: Option<RegionId>,
    /// The good used as currency on this node.
    #[serde(default)]
    pub currency_good: Option<GoodId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketTier {
    Regional,
    National,
    World,
}
