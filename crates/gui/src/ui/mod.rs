//! egui only (docs/GUI.md §3.2): panels draw view-models and return intents; the tile layout.
//! Nothing here changes the model: every act is an [`Intent`] for `reduce`.
//!
//! Drawing trigonometry is allowed here and nowhere else (D12), under
//! `#[expect(clippy::disallowed_methods, reason = "display only")]`. G0.1 needs none.
//!
//! The toolbar sits above the tiles; the tiles hold the outliner, the plots, the inspector, the
//! registry, the editor and compare, the timeline and the log (§4). Space runs or pauses the
//! focused run and `.` steps it one tick, unless a text field has the keyboard. Every panel
//! reads the focused run at the model's cursor; compare reads it against its parent.

pub mod compare;
pub mod editor;
pub mod inspector;
pub mod layout;
pub mod log;
pub mod outliner;
pub mod plots;
pub mod registry;
pub mod timeline;
pub mod toolbar;

use crate::model::{Cursor, Intent, Model};
use crate::run::SeriesKey;
use crate::vm;
use layout::Pane;
use std::collections::BTreeSet;

/// What the panels keep between frames: the plot cache, the toolbar's and the editor's text,
/// and which panes the last frame drew.
#[derive(Default)]
pub struct State {
    /// The plot cache: one decimator per plotted series.
    pub plots: plots::PlotCache,
    toolbar: toolbar::ToolbarState,
    /// The editor's form and file fields.
    pub editor: editor::EditorState,
    drawn: BTreeSet<Pane>,
}

impl State {
    /// The panes the last frame drew.
    pub fn panes_drawn(&self) -> &BTreeSet<Pane> {
        &self.drawn
    }
}

/// The model's cursor as the view-models take it: a report tick, `None` for live.
pub fn cursor(m: &Model) -> Option<u64> {
    match m.cursor() {
        Cursor::Live => None,
        Cursor::At(t) => Some(t),
    }
}

/// A number for display: six significant digits, in exponent form outside [1e-3, 1e6). The
/// full value is on hover wherever this is used.
pub fn fmt(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return format!("{v}");
    }
    let a = v.abs();
    if !(1e-3..1e6).contains(&a) {
        return format!("{v:.5e}");
    }
    let mut decimals = 5_usize;
    let mut x = a;
    while x >= 10.0 && decimals > 0 {
        x /= 10.0;
        decimals -= 1;
    }
    while x < 1.0 {
        x *= 10.0;
        decimals += 1;
    }
    let s = format!("{v:.decimals$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// A number and its unit as one label, the full value on hover.
pub fn value_label(ui: &mut egui::Ui, v: Option<f64>, unit: &str) -> egui::Response {
    match v {
        Some(v) => ui
            .label(format!("{} {unit}", fmt(v)))
            .on_hover_text(format!("{v}")),
        None => ui.weak(format!("– {unit}")),
    }
}

/// A small "plot" or "unplot" button for a series. It reads "plot"; its accessible name, and
/// its hover text, name the series, so a screen reader or a script can tell one row's button
/// from another's.
pub fn plot_button(ui: &mut egui::Ui, s: &SeriesKey, plotted: bool, out: &mut Vec<Intent>) {
    let (text, act) = if plotted {
        ("unplot", Intent::Unplot(s.clone()))
    } else {
        ("plot", Intent::Plot(s.clone()))
    };
    let r = ui.small_button(text).on_hover_text(s.to_string());
    r.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("{text} {s}"))
    });
    if r.clicked() {
        out.push(act);
    }
}

/// Draw the model into `ui` and return what the user did.
pub fn draw(
    ui: &mut egui::Ui,
    m: &Model,
    tree: &mut egui_tiles::Tree<Pane>,
    state: &mut State,
) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !ui.ctx().egui_wants_keyboard_input() {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                intents.push(Intent::RunPause);
            }
            if i.key_pressed(egui::Key::Period) {
                intents.push(Intent::Step(1));
            }
        });
    }
    state.drawn.clear();
    state.plots.begin_frame();
    egui::Panel::top("toolbar").show(ui, |ui| {
        toolbar::show(ui, m, &mut state.toolbar, &mut intents);
    });
    egui::CentralPanel::default().show(ui, |ui| {
        let mut panes = Panes {
            m,
            state,
            intents: &mut intents,
        };
        tree.ui(&mut panes, ui);
    });
    intents
}

/// The tiles' behaviour: each pane draws its view-model.
struct Panes<'a> {
    m: &'a Model,
    state: &'a mut State,
    intents: &'a mut Vec<Intent>,
}

impl egui_tiles::Behavior<Pane> for Panes<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.title().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        self.state.drawn.insert(*pane);
        let m = self.m;
        let Some(run) = m.focused() else {
            ui.weak("no tape open: Open, or rustyecon-gui <tape.ron>");
            return egui_tiles::UiResponse::None;
        };
        let store = &run.store;
        let at = cursor(m);
        let out = &mut *self.intents;
        match pane {
            Pane::Outliner => {
                match vm::outliner::build(store, m.selection(), &m.session.pins, &m.session.plots) {
                    Some(v) => outliner::show(ui, &v, out),
                    None => loading(ui),
                }
            }
            Pane::Plots => match vm::plots::build(store, &m.session.plots, at) {
                Some(v) => plots::show(ui, run.id, store, &v, &mut self.state.plots, out),
                None => loading(ui),
            },
            Pane::Inspector => match m.selection() {
                None => {
                    ui.weak("select an entity in the outliner");
                }
                Some(sel) => match vm::inspector::build(store, sel, at) {
                    Some(v) => inspector::show(ui, &v, &m.session.plots, out),
                    None => loading(ui),
                },
            },
            Pane::Registry => match vm::registry::build(store, at) {
                Some(v) => registry::show(ui, &v, out),
                None => loading(ui),
            },
            Pane::Timeline => match vm::timeline::build(store, at) {
                Some(v) => timeline::show(ui, &v, out),
                None => loading(ui),
            },
            Pane::Log => {
                let on = m
                    .session
                    .breakpoints
                    .contains(&crate::run::Breakpoint::OnError);
                log::show(ui, &vm::log::build(m.log()), on, out);
            }
            Pane::Editor => editor::show(ui, m, &mut self.state.editor, out),
            Pane::Compare => match m.parent_of(run.id) {
                None => {
                    ui.weak(
                        "the focused run is not a branch with its parent open: compare shows a \
                         branch against its parent",
                    );
                }
                Some(parent) => {
                    let lineage = run.lineage.as_ref().map(|l| l.lines()).unwrap_or_default();
                    let earlier = m.earlier(parent.id);
                    let p = vm::compare::Side {
                        store: &parent.store,
                        earlier: &earlier,
                        origin: parent.origin,
                    };
                    let c = vm::compare::Side {
                        store,
                        earlier: &[],
                        origin: run.origin,
                    };
                    match vm::compare::build(p, c, &lineage, &m.session.plots, at) {
                        Some(v) => compare::show(ui, &v),
                        None => loading(ui),
                    }
                }
            },
        }
        egui_tiles::UiResponse::None
    }
}

fn loading(ui: &mut egui::Ui) {
    ui.weak("the tape is loading");
}

#[cfg(test)]
mod tests {
    use super::fmt;

    #[test]
    fn numbers_show_six_significant_digits() {
        assert_eq!(fmt(0.0), "0");
        assert_eq!(fmt(2.0), "2");
        assert_eq!(fmt(18.345678), "18.3457");
        assert_eq!(fmt(-0.0123456789), "-0.0123457");
        assert_eq!(fmt(123456.7), "123457");
        assert_eq!(fmt(3.6e-5), "3.60000e-5");
        assert_eq!(fmt(2.5e7), "2.50000e7");
    }
}
