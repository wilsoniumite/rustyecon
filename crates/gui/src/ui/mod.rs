//! egui only (docs/GUI.md §3.2): panels draw view-models and return intents; the tile layout.
//! Nothing here changes the model: every act is an [`Intent`] for `reduce`.
//!
//! Drawing trigonometry is allowed here and nowhere else (D12), under
//! `#[expect(clippy::disallowed_methods, reason = "display only")]`.
//!
//! G0.1's first part draws the seams only: a status line from the toolbar's view-model, the
//! keys Space (run or pause) and `.` (step one tick), and the tile layout with each panel's
//! place. The panels fill in with G0.1's second part.

pub mod layout;

use crate::model::{Intent, Model};
use crate::vm;
use layout::Pane;

/// Draw the model into `ui` and return what the user did.
pub fn draw(ui: &mut egui::Ui, m: &Model, tree: &mut egui_tiles::Tree<Pane>) -> Vec<Intent> {
    let mut intents = Vec::new();
    ui.input(|i| {
        if i.key_pressed(egui::Key::Space) {
            intents.push(Intent::RunPause);
        }
        if i.key_pressed(egui::Key::Period) {
            intents.push(Intent::Step(1));
        }
    });
    egui::Panel::top("toolbar").show(ui, |ui| {
        ui.label(status_line(m));
    });
    egui::CentralPanel::default().show(ui, |ui| {
        tree.ui(&mut Panes, ui);
    });
    intents
}

/// One line from the focused run's toolbar view-model.
fn status_line(m: &Model) -> String {
    let Some(run) = m.focused() else {
        return "no tape open: rustyecon-gui <tape.ron>".to_string();
    };
    let t = vm::toolbar::build(&run.store, run.origin);
    let mut parts = Vec::new();
    if let Some(id) = &t.identity {
        let state = if id.dirty { "dirty" } else { "clean" };
        let commit: String = id.commit.chars().take(10).collect();
        parts.push(format!(
            "{} · {commit} {state} · world {} · tape {} · {}",
            id.name, id.world_id, id.tape_hash, id.origin
        ));
    }
    if let Some(c) = &t.clock {
        parts.push(format!(
            "{} · tick {} · {} ticks/year",
            c.date, c.tick, c.ticks_per_year
        ));
    }
    parts.push(format!("{:?}", t.health.status));
    if let Some(line) = &t.health.ledger_line {
        parts.push(line.clone());
    }
    parts.join(" · ")
}

/// Each panel's place, until the panels arrive.
struct Panes;

impl egui_tiles::Behavior<Pane> for Panes {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.title().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        ui.label(pane.title());
        egui_tiles::UiResponse::None
    }
}
