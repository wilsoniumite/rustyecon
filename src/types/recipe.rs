use crate::types::ids::{ComponentId, GoodId, RecipeId};
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    pub good: GoodId,
    /// Quantity consumed per unit of effective recipe size per tick.
    pub qty_per_unit: f64,
    pub scaling: InputScaling,
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
