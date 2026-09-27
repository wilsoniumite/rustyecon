//! The outliner (docs/GUI.md §4): the tape's entities by key. A click selects one; "pin" keeps
//! it at the top; "plot" plots its own series. Markets and actors open by default.

use crate::model::Intent;
use crate::vm::outliner::{OutlinerVm, RowVm};

/// Draw the outliner.
pub fn show(ui: &mut egui::Ui, vm: &OutlinerVm, out: &mut Vec<Intent>) {
    egui::ScrollArea::vertical()
        .id_salt("outliner")
        .show(ui, |ui| {
            if !vm.pinned.is_empty() {
                egui::CollapsingHeader::new("Pinned")
                    .default_open(true)
                    .show(ui, |ui| rows(ui, "pinned", &vm.pinned, out));
            }
            for s in &vm.sections {
                let open = s.title == "Markets" || s.title == "Actors";
                egui::CollapsingHeader::new(format!("{} ({})", s.title, s.rows.len()))
                    .id_salt(&s.title)
                    .default_open(open)
                    .show(ui, |ui| rows(ui, &s.title, &s.rows, out));
            }
        });
}

fn rows(ui: &mut egui::Ui, section: &str, rows: &[RowVm], out: &mut Vec<Intent>) {
    egui::Grid::new(("outliner", section))
        .num_columns(4)
        .show(ui, |ui| {
            for r in rows {
                let label = ui
                    .selectable_label(r.selected, &r.label)
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
                    Some(s) => {
                        let (text, act) = if r.plotted {
                            ("unplot", Intent::Unplot(s.clone()))
                        } else {
                            ("plot", Intent::Plot(s.clone()))
                        };
                        if ui.small_button(text).on_hover_text(s.to_string()).clicked() {
                            out.push(act);
                        }
                    }
                    None => {
                        ui.label("");
                    }
                }
                ui.end_row();
            }
        });
}
