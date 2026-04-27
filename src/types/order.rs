use crate::types::ids::{GoodId, MarketNodeId, OwnerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub node: MarketNodeId,
    pub good: GoodId,
    pub side: OrderSide,
    pub owner: OwnerId,
    pub qty: f64,
}

/// Per-(node, good) fill rates produced by the clearing phase.
/// Indexed the same way as SimState's price/imbalance arrays: node_idx * num_goods + good_idx.
pub struct MarketFills {
    /// Fraction of each buy order that was filled (1.0 when supply ≥ demand).
    pub buyer_fill_rates: Vec<f64>,
    /// Fraction of each sell order that was filled (1.0 when demand ≥ supply).
    pub seller_fill_rates: Vec<f64>,
    num_goods: usize,
}

impl MarketFills {
    pub fn new(num_nodes: usize, num_goods: usize) -> Self {
        let size = num_nodes * num_goods;
        Self {
            buyer_fill_rates: vec![0.0; size],
            seller_fill_rates: vec![0.0; size],
            num_goods,
        }
    }

    fn idx(&self, node: MarketNodeId, good: GoodId) -> usize {
        node.idx() * self.num_goods + good.idx()
    }

    pub fn buyer_fill(&self, node: MarketNodeId, good: GoodId) -> f64 {
        self.buyer_fill_rates[self.idx(node, good)]
    }

    pub fn seller_fill(&self, node: MarketNodeId, good: GoodId) -> f64 {
        self.seller_fill_rates[self.idx(node, good)]
    }

    pub fn set_buyer_fill(&mut self, node: MarketNodeId, good: GoodId, rate: f64) {
        let i = self.idx(node, good);
        self.buyer_fill_rates[i] = rate;
    }

    pub fn set_seller_fill(&mut self, node: MarketNodeId, good: GoodId, rate: f64) {
        let i = self.idx(node, good);
        self.seller_fill_rates[i] = rate;
    }
}
