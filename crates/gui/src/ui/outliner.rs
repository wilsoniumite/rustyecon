//! The outliner (docs/GUI.md §4): the tape's entities by key. A click selects one; "pin" keeps
//! it at the top; "plot" plots its own series. Markets and actors open by default.
//!
//! As built at D.3: a filter narrows every section to the keys that contain its text, and a
//! section shows at most [`SHOWN`] rows, saying how many more there are. The demo world has
//! 31,116 params and 30,078 events.
//!
//! The watchlist (G1) opens the pane: each watched series by key, its value at the cursor with
//! its unit and the change from the tick before; "watch" beside an entity's own series adds
//! it.

use super::{fmt, value_label};
use crate::model::Intent;
use crate::run::{Entity, SeriesKey};
use crate::vm::outliner::{OutlinerVm, RowVm};
use crate::vm::watch::WatchVm;

/// What the outliner reads besides its view-model, which is kept across frames.
pub struct Ctx<'a> {
    /// The selection, which the panel marks itself.
    pub selection: Option<&'a Entity>,
    /// The watched series.
    pub watch: &'a [SeriesKey],
    /// The watchlist at the cursor, made each frame.
    pub watchlist: Option<&'a WatchVm>,
}

/// The most rows a section lays out in a frame.
pub const SHOWN: usize = 300;

/// Draw the outliner, narrowed by `filter`, marking the selection: the view-model is kept
/// across frames and selections (D.3), so the panel marks the selection itself. The watchlist
/// comes first.
pub fn show(
    ui: &mut egui::Ui,
    vm: &OutlinerVm,
    ctx: &Ctx<'_>,
    filter: &mut String,
    out: &mut Vec<Intent>,
) {
    let selection = ctx.selection;
    if let Some(w) = ctx.watchlist {
        if !w.rows.is_empty() || !w.absent.is_empty() {
            watchlist(ui, w, out);
            ui.separator();
        }
    }
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
                        rows(
                            ui,
                            "pinned",
                            &vm.pinned,
                            (selection, ctx.watch),
                            &needle,
                            out,
                        )
                    });
            }
            for s in &vm.sections {
                // A long section opens when asked: the demo world has 372 markets and actors.
                let open = (s.title == "Markets" || s.title == "Actors") && s.rows.len() <= 100;
                egui::CollapsingHeader::new(format!("{} ({})", s.title, s.rows.len()))
                    .id_salt(&s.title)
                    .default_open(open)
                    .show(ui, |ui| {
                        rows(ui, &s.title, &s.rows, (selection, ctx.watch), &needle, out)
                    });
            }
        });
}

/// The watchlist: each watched series, its value at the cursor, the change from the tick
/// before, and its buttons.
fn watchlist(ui: &mut egui::Ui, w: &WatchVm, out: &mut Vec<Intent>) {
    egui::CollapsingHeader::new(format!("Watchlist ({})", w.rows.len()))
        .id_salt("watchlist")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("watchlist-grid")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    for r in &w.rows {
                        ui.label(&r.label);
                        value_label(ui, r.value, &r.unit);
                        match r.change {
                            Some(c) => ui
                                .weak(format!("{}{} since the tick before", sign(c), fmt(c)))
                                .on_hover_text(format!("{c}")),
                            None => ui.weak("–"),
                        };
                        super::plot_button(ui, &r.key, r.plotted, out);
                        super::watch_button(ui, &r.key, true, out);
                        ui.end_row();
                    }
                });
            if !w.absent.is_empty() {
                let keys: Vec<String> = w.absent.iter().map(ToString::to_string).collect();
                ui.weak(format!(
                    "watched, but not in this run's world: {}",
                    keys.join(", ")
                ));
            }
        });
}

fn sign(v: f64) -> &'static str {
    if v > 0.0 {
        "+"
    } else {
        ""
    }
}

fn rows(
    ui: &mut egui::Ui,
    section: &str,
    rows: &[RowVm],
    (selection, watch): (Option<&Entity>, &[SeriesKey]),
    needle: &str,
    out: &mut Vec<Intent>,
) {
    let hits = rows
        .iter()
        .filter(|r| needle.is_empty() || r.label.to_lowercase().contains(needle));
    let mut more = 0_usize;
    egui::Grid::new(("outliner", section))
        .num_columns(5)
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
                    Some(s) => {
                        super::plot_button(ui, s, r.plotted, out);
                        super::watch_button(ui, s, watch.contains(s), out);
                    }
                    None => {
                        ui.label("");
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
