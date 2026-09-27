//! The tile layout (docs/GUI.md §4, U8): outliner left, plots centre, inspector and registry
//! right, timeline and log below. It persists in `layout.ron`, beside `session.ron`.

use egui_tiles::{Tiles, Tree};
use serde::{Deserialize, Serialize};

/// A panel in the tile layout. The toolbar sits above the tiles; the editor and compare join
/// at G0.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pane {
    /// Tape entities by key.
    Outliner,
    /// The plotted series.
    Plots,
    /// The selection in detail.
    Inspector,
    /// Every param.
    Registry,
    /// The scrubber, events and ring checkpoints.
    Timeline,
    /// The log.
    Log,
}

impl Pane {
    /// Every pane, in the order a new layout places them.
    pub const ALL: [Pane; 6] = [
        Pane::Outliner,
        Pane::Plots,
        Pane::Inspector,
        Pane::Registry,
        Pane::Timeline,
        Pane::Log,
    ];

    /// Its tab's title.
    pub fn title(self) -> &'static str {
        match self {
            Pane::Outliner => "Outliner",
            Pane::Plots => "Plots",
            Pane::Inspector => "Inspector",
            Pane::Registry => "Registry",
            Pane::Timeline => "Timeline",
            Pane::Log => "Log",
        }
    }
}

/// The layout of a new session.
pub fn default_tree() -> Tree<Pane> {
    let mut tiles = Tiles::default();
    let outliner = tiles.insert_pane(Pane::Outliner);
    let plots = tiles.insert_pane(Pane::Plots);
    let inspector = tiles.insert_pane(Pane::Inspector);
    let registry = tiles.insert_pane(Pane::Registry);
    let timeline = tiles.insert_pane(Pane::Timeline);
    let log = tiles.insert_pane(Pane::Log);
    let right = tiles.insert_tab_tile(vec![inspector, registry]);
    let top = tiles.insert_horizontal_tile(vec![outliner, plots, right]);
    let below = tiles.insert_tab_tile(vec![timeline, log]);
    let root = tiles.insert_vertical_tile(vec![top, below]);
    Tree::new("layout", root, tiles)
}

/// The layout as RON.
pub fn to_ron(tree: &Tree<Pane>) -> String {
    ron::ser::to_string_pretty(tree, ron::ser::PrettyConfig::new()).expect("a layout serialises")
}

/// A layout from RON: one that parses and holds every pane.
pub fn from_ron(text: &str) -> Result<Tree<Pane>, String> {
    let tree: Tree<Pane> = ron::from_str(text).map_err(|e| e.to_string())?;
    let missing: Vec<&str> = Pane::ALL
        .iter()
        .filter(|p| tree.tiles.find_pane(p).is_none())
        .map(|p| p.title())
        .collect();
    if missing.is_empty() {
        Ok(tree)
    } else {
        Err(format!("the layout lacks {}", missing.join(", ")))
    }
}
