//! The tile layout (docs/GUI.md §4, U8): outliner left, the map and the plots as tabs in the
//! centre, inspector, registry, editor and compare right, timeline and log below. It persists
//! in `layout.ron`, beside `session.ron`. A layout saved before a pane existed gains it beside
//! the inspector; one saved before the map (D.3) gains the map as a tab beside the plots.

use egui_tiles::{Tile, Tiles, Tree};
use serde::{Deserialize, Serialize};

/// A panel in the tile layout. The toolbar sits above the tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// Tape edits, branches, saving and export (G0.2).
    Editor,
    /// A branch against its parent (G0.2).
    Compare,
    /// The map and its lenses (D.3; docs/GUI.md §4, §6).
    Map,
}

impl Pane {
    /// Every pane, in the order a new layout places them.
    pub const ALL: [Pane; 9] = [
        Pane::Outliner,
        Pane::Map,
        Pane::Plots,
        Pane::Inspector,
        Pane::Registry,
        Pane::Editor,
        Pane::Compare,
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
            Pane::Editor => "Editor",
            Pane::Compare => "Compare",
            Pane::Map => "Map",
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
    let editor = tiles.insert_pane(Pane::Editor);
    let compare = tiles.insert_pane(Pane::Compare);
    let timeline = tiles.insert_pane(Pane::Timeline);
    let log = tiles.insert_pane(Pane::Log);
    let map = tiles.insert_pane(Pane::Map);
    // The plots come first, so a tape with no map opens on them; one with a map is brought
    // forward on the map (`bring_forward`).
    let centre = tiles.insert_tab_tile(vec![plots, map]);
    let right = tiles.insert_tab_tile(vec![inspector, registry, editor, compare]);
    let top = tiles.insert_horizontal_tile(vec![outliner, centre, right]);
    let below = tiles.insert_tab_tile(vec![timeline, log]);
    let root = tiles.insert_vertical_tile(vec![top, below]);
    // D.3: the centre, where the map and the plots are, takes half the width and the top row
    // two thirds of the height; each pane was a third before.
    for (container, shares) in [
        (top, vec![(outliner, 1.0), (centre, 2.4), (right, 1.4)]),
        (root, vec![(top, 2.2), (below, 1.0)]),
    ] {
        if let Some(Tile::Container(egui_tiles::Container::Linear(l))) = tiles.get_mut(container) {
            for (tile, share) in shares {
                l.shares.set_share(tile, share);
            }
        }
    }
    Tree::new("layout", root, tiles)
}

/// Bring a pane's tab to the front: the map when a tape with a map opens, the plots when one
/// without does. Returns whether a tab changed.
pub fn bring_forward(tree: &mut Tree<Pane>, pane: Pane) -> bool {
    tree.make_active(|_, tile| matches!(tile, Tile::Pane(p) if *p == pane))
}

/// Put the map beside the plots, as a tab: in the plots' tabs, or in new tabs that take the
/// plots' place.
fn place_map(tree: &mut Tree<Pane>, map: egui_tiles::TileId) -> bool {
    let Some(plots) = tree.tiles.find_pane(&Pane::Plots) else {
        return false;
    };
    let Some(parent) = tree.tiles.parent_of(plots) else {
        return false;
    };
    if let Some(Tile::Container(egui_tiles::Container::Tabs(t))) = tree.tiles.get_mut(parent) {
        t.add_child(map);
        return true;
    }
    let tabs = tree.tiles.insert_tab_tile(vec![plots, map]);
    match tree.tiles.get_mut(parent) {
        Some(Tile::Container(c)) => c.replace_child(plots, tabs).is_some(),
        _ => false,
    }
}

/// The layout as RON.
pub fn to_ron(tree: &Tree<Pane>) -> String {
    ron::ser::to_string_pretty(tree, ron::ser::PrettyConfig::new()).expect("a layout serialises")
}

/// A layout from RON: one that parses. A pane it lacks joins the container that holds the
/// inspector, or the root's, and the map joins the plots as a tab; a layout with no container
/// to hold it does not read.
pub fn from_ron(text: &str) -> Result<Tree<Pane>, String> {
    let mut tree: Tree<Pane> = ron::from_str(text).map_err(|e| e.to_string())?;
    let missing: Vec<Pane> = Pane::ALL
        .iter()
        .copied()
        .filter(|p| tree.tiles.find_pane(p).is_none())
        .collect();
    for p in missing {
        if p == Pane::Map {
            let id = tree.tiles.insert_pane(p);
            if place_map(&mut tree, id) {
                continue;
            }
            tree.tiles.remove(id);
        }
        let home = tree
            .tiles
            .find_pane(&Pane::Inspector)
            .and_then(|i| tree.tiles.parent_of(i))
            .or_else(|| tree.root());
        let id = tree.tiles.insert_pane(p);
        match home.and_then(|h| tree.tiles.get_mut(h)) {
            Some(Tile::Container(c)) => c.add_child(id),
            _ => return Err(format!("the layout has no place for {}", p.title())),
        }
    }
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_older_layout_gains_the_new_panes() {
        let mut tiles = Tiles::default();
        let panes: Vec<_> = [
            Pane::Outliner,
            Pane::Plots,
            Pane::Inspector,
            Pane::Registry,
            Pane::Timeline,
            Pane::Log,
        ]
        .into_iter()
        .map(|p| tiles.insert_pane(p))
        .collect();
        let right = tiles.insert_tab_tile(panes[2..4].to_vec());
        let top = tiles.insert_horizontal_tile(vec![panes[0], panes[1], right]);
        let below = tiles.insert_tab_tile(panes[4..].to_vec());
        let root = tiles.insert_vertical_tile(vec![top, below]);
        let old = Tree::new("layout", root, tiles);
        let tree = from_ron(&to_ron(&old)).expect("it reads");
        for p in Pane::ALL {
            assert!(tree.tiles.find_pane(&p).is_some(), "{p:?}");
        }
        let editor = tree.tiles.find_pane(&Pane::Editor).unwrap();
        assert_eq!(tree.tiles.parent_of(editor), Some(right));
        // D.3: the map joins the plots as a tab, in tabs that take the plots' place.
        let map = tree.tiles.find_pane(&Pane::Map).unwrap();
        let plots = tree.tiles.find_pane(&Pane::Plots).unwrap();
        let centre = tree.tiles.parent_of(map).expect("the map has a parent");
        assert_eq!(tree.tiles.parent_of(plots), Some(centre));
        assert!(matches!(
            tree.tiles.get(centre),
            Some(Tile::Container(egui_tiles::Container::Tabs(_)))
        ));
        assert_eq!(tree.tiles.parent_of(centre), Some(top));
        assert!(from_ron(&to_ron(&default_tree())).is_ok());
        let mut fresh = default_tree();
        assert!(bring_forward(&mut fresh, Pane::Map));
    }
}
