use crate::scenario::{raw, EventSchedule};
use crate::state::SimState;
use std::error::Error;
use std::fs;
use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// A fully loaded scenario ready to hand to the runner.
pub struct Scenario {
    pub state: SimState,
    pub game_data: crate::state::GameData,
    pub events: EventSchedule,
}

/// Load a scenario from a directory. Expects three files:
///   game_data.ron      — good/recipe/wealth definitions (string-keyed, no numeric IDs)
///   starting_state.ron — SimState at tick 0 (string refs resolved against game_data)
///   events.ron         — scheduled StateDelta timeline (still uses numeric IDs)
///
/// A binary `starting_state.bin` takes precedence over the RON file if present.
pub fn load(dir: &Path) -> Result<Scenario> {
    // Phase 1: load and resolve game_data — builds the name→ID resolver.
    let raw_gd: raw::RawGameData = load_ron(&dir.join("game_data.ron"))?;
    let (game_data, resolver) = raw::resolve_game_data(raw_gd)?;

    // Phase 2: load starting state.
    let state = {
        let bin_path = dir.join("starting_state.bin");
        let ron_path = dir.join("starting_state.ron");
        if bin_path.exists() {
            let bytes = fs::read(&bin_path)?;
            bincode::deserialize(&bytes)?
        } else {
            let raw_state: raw::RawSimState = load_ron(&ron_path)?;
            raw::resolve_sim_state(raw_state, &resolver, &game_data)?
        }
    };

    let events: EventSchedule = load_ron(&dir.join("events.ron"))?;

    Ok(Scenario { state, game_data, events })
}

fn load_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    ron::from_str(&text)
        .map_err(|e| format!("{}: {e}", path.display()).into())
}
