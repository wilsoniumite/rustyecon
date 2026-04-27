/// Test utilities: magic producers and consumers that peg prices for isolation testing.
/// Available under `#[cfg(any(test, feature = "test-utils"))]`.
///
/// A MagicProducer posts unlimited sell orders at a target price, keeping the
/// price from rising above it. A MagicConsumer posts unlimited buy orders at a
/// target price, keeping the price from falling below it. Together they pin a
/// price so you can test a specific system in isolation without the full sim running.

use crate::types::ids::{GoodId, MarketNodeId};

pub struct MagicProducer {
    pub node: MarketNodeId,
    pub good: GoodId,
    pub target_price: f64,
    pub volume_per_tick: f64,
}

pub struct MagicConsumer {
    pub node: MarketNodeId,
    pub good: GoodId,
    pub target_price: f64,
    pub volume_per_tick: f64,
}
