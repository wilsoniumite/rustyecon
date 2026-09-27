//! The registry (docs/GUI.md §4): every param, as `rustyecon registry` lists it, with its
//! unit, use, genesis and current values, and its basis; each of its uses with its conversion
//! and per-tick value; and beside the current value, the basis of the param the last
//! `SetParam` copied, with that event's key and date. Below, the inline numbers.

use super::fmt;
use crate::model::Intent;
use crate::run::Entity;
use crate::vm::registry::RegistryVm;
use egui_extras::{Column, TableBuilder};

/// Draw the registry.
pub fn show(ui: &mut egui::Ui, vm: &RegistryVm, out: &mut Vec<Intent>) {
    if let Some(e) = &vm.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
        return;
    }
    match vm.tick {
        Some(t) => ui.label(format!(
            "{} params and {} inline numbers; current values after tick {t}",
            vm.params.len(),
            vm.inline.len()
        )),
        None => ui.label(format!(
            "{} params and {} inline numbers; no tick has run",
            vm.params.len(),
            vm.inline.len()
        )),
    };
    let line = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let height = ui.available_height() * 0.7;
    TableBuilder::new(ui)
        .id_salt("registry")
        .striped(true)
        .resizable(true)
        .max_scroll_height(height)
        .column(Column::auto().at_least(120.0))
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(160.0))
        .column(Column::remainder().at_least(160.0))
        .header(line, |mut h| {
            for title in [
                "param", "unit", "use", "genesis", "current", "uses", "basis",
            ] {
                h.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|mut body| {
            for p in &vm.params {
                let lines = p.sites.len().max(1) + usize::from(p.copied.is_some());
                body.row(line * lines as f32, |mut row| {
                    row.col(|ui| {
                        if ui.link(&p.key).clicked() {
                            if let Ok(k) = rustyecon_engine::prelude::Key::new(p.key.clone()) {
                                out.push(Intent::Select(Some(Entity::Param(k))));
                            }
                        }
                    });
                    row.col(|ui| {
                        ui.label(&p.unit);
                    });
                    row.col(|ui| {
                        ui.label(&p.use_);
                    });
                    row.col(|ui| {
                        ui.label(fmt(p.genesis))
                            .on_hover_text(format!("{}", p.genesis));
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            match p.current {
                                Some(v) => ui.label(fmt(v)).on_hover_text(format!("{v}")),
                                None => ui.weak("–"),
                            };
                            if let Some(c) = &p.copied {
                                ui.weak(format!("from {} by {} on {}", c.source, c.event, c.date))
                                    .on_hover_text(&c.basis);
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            if p.sites.is_empty() {
                                ui.weak("no use: a SetParam's target or a source only");
                            }
                            for s in &p.sites {
                                let now = s
                                    .per_tick_now
                                    .map_or_else(String::new, |v| format!(" (now {})", fmt(v)));
                                ui.label(format!(
                                    "{} {}{now} at {}",
                                    s.method,
                                    fmt(s.per_tick),
                                    s.path
                                ));
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.label(&p.basis);
                            if let Some(c) = &p.copied {
                                ui.weak(&c.basis);
                            }
                        });
                    });
                });
            }
        });
    ui.separator();
    egui::CollapsingHeader::new(format!("Inline numbers ({})", vm.inline.len()))
        .id_salt("registry-inline")
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("registry-inline-scroll")
                .show(ui, |ui| {
                    egui::Grid::new("registry-inline-grid")
                        .striped(true)
                        .num_columns(3)
                        .show(ui, |ui| {
                            for n in &vm.inline {
                                ui.label(&n.path);
                                ui.label(fmt(n.value)).on_hover_text(format!("{}", n.value));
                                ui.weak(&n.basis);
                                ui.end_row();
                            }
                        });
                });
        });
}
