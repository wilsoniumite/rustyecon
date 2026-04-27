use crate::types::ids::{GoodId, MagicProducerId, MarketNodeId};
use serde::{Deserialize, Serialize};

/// A test supply source that posts a fixed sell order every tick with no inventory.
/// Used in scenarios to provide controlled, clean supply without building agent noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicProducer {
    pub id: MagicProducerId,
    pub node: MarketNodeId,
    pub good: GoodId,
    pub qty_per_tick: f64,
}
