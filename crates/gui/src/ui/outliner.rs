//! The outliner (docs/GUI.md §4): the tape's entities by key. A click selects one; "pin" keeps
//! it at the top; "plot" plots its own series. Markets and actors open by default.
//!
//! As built at D.3: a filter narrows every section to the keys that contain its text, and a
//! section shows at most [`SHOWN`] rows, saying how many more there are. The demo world has
//! 31,116 params and 30,078 events.

use crate::model::Intent;
use crate::run::Entity;
use crate::vm::outliner::{OutlinerVm, RowVm};

/// The most rows a section lays out in a frame.
pub const SHOWN: usize = 300;

/// Draw the outliner, narrowed by `filter`, marking `selection`: the view-model is kept across
/// frames and selections (D.3), so the panel marks the selection itself.
pub fn show(
    ui: &mut egui::Ui,
    vm: &OutlinerVm,
    selection: Option<&Entity>,
    filter: &mut String,
    out: &mut Vec<Intent>,
) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(filter)
                .hint_text("filter by key")
                .desired_width(ui.available_width() - 30.0),
        );
        if !filter.is_empty() && ui.small_button("✕").clicked() {
            filter.clear();
        }
    });
    let needle = filter.trim().to_lowercase();
    egui::ScrollArea::vertical()
        .id_salt("outliner")
        .show(ui, |ui| {
            if !vm.pinned.is_empty() {
                egui::CollapsingHeader::new("Pinned")
                    .default_open(true)
                    .show(ui, |ui| {
                        rows(ui, "pinned", &vm.pinned, selection, &needle, out)
                    });
            }
            for s in &vm.sections {
                // A long section opens when asked: the demo world has 372 markets and actors.
                let open = (s.title == "Markets" || s.title == "Actors") && s.rows.len() <= 100;
                egui::CollapsingHeader::new(format!("{} ({})", s.title, s.rows.len()))
                    .id_salt(&s.title)
                    .default_open(open)
                    .show(ui, |ui| {
                        rows(ui, &s.title, &s.rows, selection, &needle, out)
                    });
            }
        });
}

fn rows(
    ui: &mut egui::Ui,
    section: &str,
    rows: &[RowVm],
    selection: Option<&Entity>,
    needle: &str,
    out: &mut Vec<Intent>,
) {
    let hits = rows
        .iter()
        .filter(|r| needle.is_empty() || r.label.to_lowercase().contains(needle));
    let mut more = 0_usize;
    egui::Grid::new(("outliner", section))
        .num_columns(4)
        .show(ui, |ui| {
            for (k, r) in hits.enumerate() {
                if k >= SHOWN {
                    more += 1;
                    continue;
                }
                let label = ui
                    .selectable_label(selection == Some(&r.entity), &r.label)
                    .on_hover_text(&r.detail);
                if label.clicked() {
                    out.push(Intent::Select(Some(r.entity.clone())));
                }
                ui.weak(&r.detail);
                let (pin, act) = if r.pinned {
                    ("unpin", Intent::Unpin(r.entity.clone()))
                } else {
                    ("pin", Intent::Pin(r.entity.clone()))
                };
                if ui.small_button(pin).clicked() {
                    out.push(act);
                }
                match &r.series {
                    Some(s) => super::plot_button(ui, s, r.plotted, out),
                    None => {
                        ui.label("");
                    }
                }
                ui.end_row();
            }
        });
    if more > 0 {
        ui.weak(format!("… and {more} more: type in the filter to narrow"));
    }
}
