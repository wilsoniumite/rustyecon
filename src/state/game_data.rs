use crate::types::{
    channel::ChannelDef,
    good::GoodDef,
    ids::{ChannelId, GoodId, MarketNodeId, RecipeId, RegionId},
    market_node::MarketNodeDef,
    need_category::NeedCategory,
    recipe::RecipeDef,
};
use serde::{Deserialize, Serialize};

/// Demand quantities for each need category at one integer wealth tier.
/// `qty_per_pop[cat_idx]` = units demanded per person per tick from that category.
/// Categories not listed for this tier have qty = 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthLevel {
    pub tier: u8,
    pub qty_per_pop: Vec<f64>, // length == game_data.need_categories.len()
}

/// Static definitions that do not change during a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub goods: Vec<GoodDef>,
    pub recipes: Vec<RecipeDef>,
    /// Substitution categories — defines what goods satisfy each need.
    #[serde(default)]
    pub need_categories: Vec<NeedCategory>,
    /// Integer wealth tiers in ascending order. Fractional wealth is interpolated.
    #[serde(default)]
    pub wealth_levels: Vec<WealthLevel>,
    pub market_nodes: Vec<MarketNodeDef>,
    #[serde(default)]
    pub channels: Vec<ChannelDef>,
    pub regions: Vec<RegionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDef {
    pub id: RegionId,
    pub name: String,
    pub market_node: MarketNodeId,
    #[serde(default)]
    pub position: Option<(f64, f64)>,
}

impl GameData {
    pub fn good(&self, id: GoodId) -> &GoodDef { &self.goods[id.idx()] }
    pub fn recipe(&self, id: RecipeId) -> &RecipeDef { &self.recipes[id.idx()] }
    pub fn market_node(&self, id: MarketNodeId) -> &MarketNodeDef { &self.market_nodes[id.idx()] }
    pub fn channel(&self, id: ChannelId) -> &ChannelDef { &self.channels[id.idx()] }
    pub fn region(&self, id: RegionId) -> &RegionDef { &self.regions[id.idx()] }
    pub fn num_goods(&self) -> usize { self.goods.len() }
    pub fn num_nodes(&self) -> usize { self.market_nodes.len() }

    /// Linear interpolation of qty_per_pop across integer wealth tiers.
    /// Returns a Vec indexed by need category.
    pub fn interpolated_qty(&self, wealth: f64) -> Vec<f64> {
        let n = self.need_categories.len();
        if self.wealth_levels.is_empty() || n == 0 {
            return vec![0.0; n];
        }
        let wealth = wealth.max(0.0);
        let floor_tier = wealth.floor() as u8;
        let frac = wealth.fract();

        // Find bracketing tiers; clamp at the edges.
        let lower = self.wealth_levels.iter()
            .filter(|wl| wl.tier <= floor_tier)
            .last()
            .unwrap_or(&self.wealth_levels[0]);
        let upper = self.wealth_levels.iter()
            .find(|wl| wl.tier > floor_tier)
            .unwrap_or(lower);

        (0..n).map(|i| {
            let lo = lower.qty_per_pop.get(i).copied().unwrap_or(0.0);
            let hi = upper.qty_per_pop.get(i).copied().unwrap_or(0.0);
            lo + frac * (hi - lo)
        }).collect()
    }

    /// Largest wealth tier defined. Useful for clamping.
    pub fn max_wealth_tier(&self) -> f64 {
        self.wealth_levels.last().map(|wl| wl.tier as f64).unwrap_or(0.0)
    }

    /// Weight-proportional default sub_state: one fractions-vector per category.
    pub fn default_sub_state(&self) -> Vec<Vec<f64>> {
        self.need_categories.iter()
            .map(|cat| cat.target_fractions())
            .collect()
    }
}
