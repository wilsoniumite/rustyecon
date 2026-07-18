use crate::types::{
    building::Building,
    channel::ChannelState,
    ids::{BuildingId, GoodId, InventoryId, MarketNodeId, MagicProducerId, RecipeInstanceId},
    inventory::Inventory,
    magic_producer::MagicProducer,
    pop_group::{PopGroup, PopPair},
    recipe_instance::RecipeInstance,
};
use serde::{Deserialize, Serialize};

/// The complete runtime state of the simulation at a single point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimState {
    pub tick: u64,

    /// Prices: flat array indexed by node_id * num_goods + good_id.
    prices: Vec<f64>,

    /// Aggregated sell volume per (node, good) after clearing; used by price_update.
    supply: Vec<f64>,

    /// Aggregated buy volume per (node, good) after clearing; used by price_update.
    demand: Vec<f64>,

    /// One-year EMA of posted prices. Same flat layout as prices. Updated each tick
    /// by price_update. Decisions and metrics that need medium-term price signals
    /// should read this rather than the raw posted price.
    price_ema: Vec<f64>,

    num_goods: usize,
    num_nodes: usize,

    pub buildings: Vec<Building>,
    #[serde(default)]
    pub recipe_instances: Vec<RecipeInstance>,
    pub pop_groups: Vec<PopGroup>,
    /// Centralised inventory store. Buildings and PopGroups hold an InventoryId
    /// that indexes into this Vec.
    #[serde(default)]
    pub inventories: Vec<Inventory>,
    /// Pairs employed and unemployed PopGroups. Populated on scenario load;
    /// used by pop_update for labour redistribution each tick.
    #[serde(default)]
    pub pop_pairs: Vec<PopPair>,
    pub channels: Vec<ChannelState>,
    #[serde(default)]
    pub magic_producers: Vec<MagicProducer>,
}

impl SimState {
    pub fn new(num_goods: usize, num_nodes: usize) -> Self {
        let size = num_goods * num_nodes;
        Self {
            tick: 0,
            prices: vec![1.0; size],
            supply: vec![0.0; size],
            demand: vec![0.0; size],
            price_ema: vec![1.0; size],
            num_goods,
            num_nodes,
            buildings: Vec::new(),
            recipe_instances: Vec::new(),
            pop_groups: Vec::new(),
            inventories: Vec::new(),
            pop_pairs: Vec::new(),
            channels: Vec::new(),
            magic_producers: Vec::new(),
        }
    }

    /// Construct from resolved prices (e.g. from a starting_state.ron load).
    /// Imbalances start at zero — they're computed fresh each tick by clearing.
    pub fn from_prices(tick: u64, num_goods: usize, num_nodes: usize, prices: Vec<f64>) -> Self {
        let size = num_goods * num_nodes;
        Self {
            tick,
            price_ema: prices.clone(),
            prices,
            supply: vec![0.0; size],
            demand: vec![0.0; size],
            num_goods,
            num_nodes,
            buildings: Vec::new(),
            recipe_instances: Vec::new(),
            pop_groups: Vec::new(),
            inventories: Vec::new(),
            pop_pairs: Vec::new(),
            channels: Vec::new(),
            magic_producers: Vec::new(),
        }
    }

    pub fn inventory(&self, id: InventoryId) -> &Inventory {
        &self.inventories[id.idx()]
    }

    pub fn inventory_mut(&mut self, id: InventoryId) -> &mut Inventory {
        &mut self.inventories[id.idx()]
    }

    pub fn magic_producer_mut(&mut self, id: MagicProducerId) -> &mut MagicProducer {
        &mut self.magic_producers[id.idx()]
    }

    fn idx(&self, node: MarketNodeId, good: GoodId) -> usize {
        node.idx() * self.num_goods + good.idx()
    }

    pub fn price(&self, node: MarketNodeId, good: GoodId) -> f64 {
        self.prices[self.idx(node, good)]
    }

    pub fn set_price(&mut self, node: MarketNodeId, good: GoodId, price: f64) {
        let i = self.idx(node, good);
        self.prices[i] = price;
    }

    pub fn supply(&self, node: MarketNodeId, good: GoodId) -> f64 {
        self.supply[self.idx(node, good)]
    }

    pub fn demand(&self, node: MarketNodeId, good: GoodId) -> f64 {
        self.demand[self.idx(node, good)]
    }

    pub fn set_supply(&mut self, node: MarketNodeId, good: GoodId, qty: f64) {
        let i = self.idx(node, good);
        self.supply[i] = qty;
    }

    pub fn set_demand(&mut self, node: MarketNodeId, good: GoodId, qty: f64) {
        let i = self.idx(node, good);
        self.demand[i] = qty;
    }

    /// Computed from stored supply and demand: (demand - supply) / max(supply, demand).
    pub fn imbalance(&self, node: MarketNodeId, good: GoodId) -> f64 {
        let s = self.supply[self.idx(node, good)];
        let d = self.demand[self.idx(node, good)];
        if s <= 0.0 && d <= 0.0 { return 0.0; }
        (d - s) / s.max(d)
    }

    pub fn price_ema(&self, node: MarketNodeId, good: GoodId) -> f64 {
        self.price_ema[self.idx(node, good)]
    }

    pub fn set_price_ema(&mut self, node: MarketNodeId, good: GoodId, ema: f64) {
        let i = self.idx(node, good);
        self.price_ema[i] = ema;
    }

    pub fn building(&self, id: BuildingId) -> &Building {
        &self.buildings[id.idx()]
    }

    pub fn building_mut(&mut self, id: BuildingId) -> &mut Building {
        &mut self.buildings[id.idx()]
    }

    pub fn recipe_instance(&self, id: RecipeInstanceId) -> &RecipeInstance {
        &self.recipe_instances[id.idx()]
    }

    pub fn recipe_instance_mut(&mut self, id: RecipeInstanceId) -> &mut RecipeInstance {
        &mut self.recipe_instances[id.idx()]
    }
}
