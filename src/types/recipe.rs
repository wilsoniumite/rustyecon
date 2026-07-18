use crate::types::ids::{ComponentId, GoodId, RecipeId};
use serde::{Deserialize, Serialize};

/// Determines how a recipe instance sets its chosen_size each tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyKind {
    /// PD capacity controller: follows demand, exits when unprofitable.
    CapacityControl,
    /// Pays out inventory surplus above a multiple of smoothed input cost.
    DividendPayout { reserve_multiple: f64 },
    /// Runs at recipe_size every tick with no decision logic.
    AlwaysRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDef {
    pub id: RecipeId,
    pub name: String,
    pub inputs: Vec<RecipeInput>,
    /// Multiple outputs at fixed ratios — ratios do not respond to prices.
    pub outputs: Vec<RecipeOutput>,
    /// Components that must be held at full recipe size for this recipe to operate.
    pub component_reqs: Vec<ComponentReq>,
    /// If true, can run at negative chosen_size (inputs/outputs swap).
    pub reversible: bool,
    /// Strategy that governs how instances of this recipe make capacity decisions.
    #[serde(default = "default_capacity_control")]
    pub strategy: StrategyKind,
}

fn default_capacity_control() -> StrategyKind { StrategyKind::CapacityControl }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    pub good: GoodId,
    /// Quantity consumed per unit of effective recipe size per tick.
    pub qty_per_unit: f64,
    pub scaling: InputScaling,
}

impl RecipeInput {
    /// Desired quantity of this input given the current chosen and recipe sizes.
    pub fn desired(&self, chosen: f64, recipe_size: f64) -> f64 {
        match self.scaling {
            InputScaling::Variable => self.qty_per_unit * chosen,
            InputScaling::Fixed => self.qty_per_unit * recipe_size,
            InputScaling::SemiVariable { floor, slope } => floor + slope * chosen,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputScaling {
    /// Scales linearly with chosen recipe size.
    Variable,
    /// Paid at recipe_size regardless of chosen_size.
    Fixed,
    /// floor + slope * chosen_size.
    SemiVariable { floor: f64, slope: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOutput {
    pub good: GoodId,
    pub qty_per_unit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentReq {
    pub component: ComponentId,
    pub qty_per_unit_size: f64,
}
