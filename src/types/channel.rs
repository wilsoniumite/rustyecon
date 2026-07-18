use crate::types::ids::{ChannelId, MarketNodeId, RecipeInstanceId};
use serde::{Deserialize, Serialize};

/// Static channel definition (in GameData). Channels are directed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDef {
    pub id: ChannelId,
    pub from: MarketNodeId,
    pub to: MarketNodeId,
    pub channel_type: ChannelType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Carries physical goods; crossing cost scales with distance and infrastructure.
    Trade,
    /// Carries financial goods; crossing cost is information asymmetry, not transport.
    Capital,
}

/// Runtime channel state (in SimState).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    /// 0.0 = fully blocked, 1.0 = free flow. Modified by laws (tariffs, capital controls).
    pub regulatory_factor: f64,
    /// Which recipe instance currently occupies this channel slot, if any.
    pub occupant: Option<RecipeInstanceId>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self { regulatory_factor: 1.0, occupant: None }
    }
}
