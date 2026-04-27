use crate::state::{GameData, SimState};
use crate::types::delta::StateDelta;

/// Reserved for recipes that fire automatically each tick without agent decisions:
/// spoilage (ShelfLife::Ticks goods decay), maturity (aged goods transform), etc.
/// Pop consumption and target updates live in pop_update instead.
pub fn run(_state: &SimState, _game_data: &GameData) -> Vec<StateDelta> {
    // TODO: spoilage, maturity
    vec![]
}
