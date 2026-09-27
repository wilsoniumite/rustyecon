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
pub mod lab;
pub mod layout;
pub mod log;
pub mod map;
pub mod outliner;
pub mod plots;
pub mod registry;
pub mod timeline;
pub mod toolbar;

use crate::model::{Cursor, Intent, Model};
use crate::run::{Entity, SeriesKey};
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
    /// The map: the atlas triangulated once, the lens shown, the view (D.3).
    pub map: map::MapState,
    /// The oracle lab's form and what it last solved (G1).
    pub lab: lab::LabState,
    /// The log's breakpoint field (G1).
    break_at: String,
    drawn: BTreeSet<Pane>,
    /// The run and load whose tab was last brought forward: the map for a tape with one,
    /// the plots for a tape without.
    shown: Option<(crate::run::RunId, u64)>,
    outliner: OutlinerCache,
    /// The log's view-model and the number of lines it was made from: the log only grows, and
    /// a run of the demo world logs 30,078 fired events (D.3).
    log: Option<(usize, vm::log::LogVm)>,
    /// The map's view-model, what it was made from and when (D.3): kept while the run, its
    /// record, the cursor, the lens and the selection are unchanged, so a paused map remakes
    /// nothing, and while a run streams live remade at most 30 times a second.
    map_vm: Option<(MapKey, std::time::Instant, Option<vm::map::MapVm>)>,
    /// The registry's view-model, what it was made from and when (D.3). The demo world's 31,116
    /// params take about 75 ms to list, so while a run streams live the registry is remade at
    /// most four times a second; it names the tick it was made at.
    registry: Option<(
        RegistryKey,
        std::time::Instant,
        Option<vm::registry::RegistryVm>,
    )>,
    /// The selected market's log waterfall, what it was made from and when (G1): remade when
    /// its key changes, and at most four times a second while a run streams live.
    waterfall: Option<(
        WaterfallKey,
        std::time::Instant,
        Result<vm::pricestep::WaterfallVm, String>,
    )>,
}

/// What the map's view-model reads: the run, its load and its latest tick, the cursor, the lens
/// and the selection.
type MapKey = (
    crate::run::RunId,
    u64,
    u64,
    Option<u64>,
    String,
    Option<Entity>,
);

/// What a market's log waterfall reads: the run, its load and its latest tick, the cursor, and
/// the market.
type WaterfallKey = (
    crate::run::RunId,
    u64,
    u64,
    Option<u64>,
    rustyecon_engine::prelude::Key,
    rustyecon_engine::prelude::Key,
);

/// What the registry's view-model reads: the run, its load and its latest tick, and the
/// cursor.
type RegistryKey = (crate::run::RunId, u64, u64, Option<u64>);

/// What the outliner's view-model reads, the selection aside: the run and its load, the pins
/// and the plots. It reads no tick.
type OutlinerKey = (crate::run::RunId, u64, Vec<Entity>, Vec<SeriesKey>);

/// The outliner's view-model and filter, kept while its key is unchanged: listing the demo
/// world's 31,116 params and 30,078 events took about 60 ms, too slow for a frame (D.3). The
/// panel marks the selection itself, so a new selection does not remake it.
#[derive(Default)]
struct OutlinerCache {
    key: Option<OutlinerKey>,
    vm: Option<vm::outliner::OutlinerVm>,
    filter: String,
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

/// A small "watch" or "unwatch" button for a series (G1): the outliner's watchlist shows a
/// watched series at the cursor. Its accessible name, and its hover text, name the series.
pub fn watch_button(ui: &mut egui::Ui, s: &SeriesKey, watched: bool, out: &mut Vec<Intent>) {
    let (text, act) = if watched {
        ("unwatch", Intent::Unwatch(s.clone()))
    } else {
        ("watch", Intent::Watch(s.clone()))
    };
    let r = ui.small_button(text).on_hover_text(s.to_string());
    r.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("{text} {s}"))
    });
    if r.clicked() {
        out.push(act);
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
    // A tape with a map opens on the map, and one without on the plots, once per load.
    if let Some(run) = m.focused() {
        let load = (run.id, run.store.generation());
        if let (Some(w), true) = (run.store.world(), state.shown != Some(load)) {
            state.shown = Some(load);
            let mapped = state
                .map
                .geo()
                .is_ok_and(|g| vm::map::geography(w, &g.atlas).is_ok());
            layout::bring_forward(tree, if mapped { Pane::Map } else { Pane::Plots });
        }
    }
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
        // The lab needs no tape: it solves the oracle, outside any run (G1).
        if *pane == Pane::Lab {
            lab::show(ui, &mut self.state.lab);
            return egui_tiles::UiResponse::None;
        }
        let Some(run) = m.focused() else {
            ui.weak("no tape open: Open, or rustyecon-gui <tape.ron>");
            return egui_tiles::UiResponse::None;
        };
        let store = &run.store;
        let at = cursor(m);
        let out = &mut *self.intents;
        match pane {
            Pane::Outliner => {
                let key = (
                    run.id,
                    store.generation(),
                    m.session.pins.clone(),
                    m.session.plots.clone(),
                );
                let cache = &mut self.state.outliner;
                if cache.key.as_ref() != Some(&key) {
                    cache.vm = vm::outliner::build(
                        store,
                        m.selection(),
                        &m.session.pins,
                        &m.session.plots,
                    );
                    cache.key = Some(key);
                }
                let watch = vm::watch::build(store, &m.session.watch, &m.session.plots, at);
                match &cache.vm {
                    Some(v) => {
                        let ctx = outliner::Ctx {
                            selection: m.selection(),
                            watch: &m.session.watch,
                            watchlist: watch.as_ref(),
                        };
                        outliner::show(ui, v, &ctx, &mut cache.filter, out);
                    }
                    None => loading(ui),
                }
            }
            Pane::Plots => {
                match vm::plots::build_with(store, &m.session.plots, &m.session.log_axes, at) {
                    Some(v) => plots::show(ui, run.id, store, &v, &mut self.state.plots, out),
                    None => loading(ui),
                }
            }
            Pane::Map => match self.state.map.ready() {
                Err(e) => {
                    ui.colored_label(ui.visuals().error_fg_color, e);
                }
                Ok(()) => {
                    let key = (
                        run.id,
                        store.generation(),
                        store.tick(),
                        at,
                        self.state.map.lens.clone(),
                        m.selection().cloned(),
                    );
                    // While the run streams live, the map is remade at most every 33 ms,
                    // 30 times a second; its overlay names the tick it shows.
                    let live = at.is_none()
                        && matches!(store.status(), crate::run::RunStatus::Running { .. });
                    let fresh = match &self.state.map_vm {
                        None => false,
                        Some((k, _, _)) if *k == key => true,
                        Some((k, made, _)) => {
                            live && (k.0, k.1, k.3, &k.4, &k.5)
                                == (key.0, key.1, key.3, &key.4, &key.5)
                                && made.elapsed() < std::time::Duration::from_millis(33)
                        }
                    };
                    if !fresh {
                        let built = self.state.map.parts().and_then(|(geo, lenses)| {
                            vm::map::build(
                                store,
                                run.origin,
                                &geo.atlas,
                                lenses,
                                &self.state.map.lens,
                                m.selection(),
                                at,
                            )
                        });
                        self.state.map_vm = Some((key, std::time::Instant::now(), built));
                    }
                    match self.state.map_vm.as_ref().and_then(|(_, _, v)| v.as_ref()) {
                        Some(v) => map::show(ui, v, &mut self.state.map, out),
                        None => loading(ui),
                    }
                }
            },
            Pane::Inspector => match m.selection() {
                None => {
                    ui.weak("select an entity in the outliner");
                }
                Some(sel) => match vm::inspector::build(store, sel, at) {
                    Some(v) => {
                        let waterfall = match sel {
                            Entity::Market { node, good } => {
                                let key = (
                                    run.id,
                                    store.generation(),
                                    store.tick(),
                                    at,
                                    node.clone(),
                                    good.clone(),
                                );
                                let live = at.is_none()
                                    && matches!(
                                        store.status(),
                                        crate::run::RunStatus::Running { .. }
                                    );
                                let fresh = match &self.state.waterfall {
                                    None => false,
                                    Some((k, _, _)) if *k == key => true,
                                    // While the run streams live, a waterfall stands 250 ms.
                                    Some((k, made, _)) => {
                                        live && (&k.0, k.1, k.3, &k.4, &k.5)
                                            == (&key.0, key.1, key.3, &key.4, &key.5)
                                            && made.elapsed()
                                                < std::time::Duration::from_millis(250)
                                    }
                                };
                                if !fresh {
                                    let w = vm::pricestep::waterfall(store, node, good, at);
                                    self.state.waterfall =
                                        Some((key, std::time::Instant::now(), w));
                                }
                                self.state.waterfall.as_ref().map(|(_, _, w)| w)
                            }
                            _ => None,
                        };
                        let ctx = inspector::Ctx {
                            plots: &m.session.plots,
                            watch: &m.session.watch,
                            breakpoints: &m.session.breakpoints,
                            waterfall,
                        };
                        inspector::show(ui, &v, &ctx, out);
                    }
                    None => loading(ui),
                },
            },
            Pane::Registry => {
                let key = (run.id, store.generation(), store.tick(), at);
                let live =
                    at.is_none() && matches!(store.status(), crate::run::RunStatus::Running { .. });
                let fresh = match &self.state.registry {
                    None => false,
                    Some((k, _, _)) if *k == key => true,
                    // While the run streams live, the last listing stands for 250 ms.
                    Some((k, made, _)) => {
                        live && (k.0, k.1, k.3) == (key.0, key.1, key.3)
                            && made.elapsed() < std::time::Duration::from_millis(250)
                    }
                };
                if !fresh {
                    let v = vm::registry::build(store, at);
                    self.state.registry = Some((key, std::time::Instant::now(), v));
                }
                match self
                    .state
                    .registry
                    .as_ref()
                    .and_then(|(_, _, v)| v.as_ref())
                {
                    Some(v) => registry::show(ui, v, out),
                    None => loading(ui),
                }
            }
            Pane::Timeline => match vm::timeline::build(store, at) {
                Some(v) => timeline::show(ui, &v, out),
                None => loading(ui),
            },
            Pane::Log => {
                // The log only grows: the lines added since the last frame are appended.
                let all = m.log();
                let (n, v) = self
                    .state
                    .log
                    .get_or_insert_with(|| (0, vm::log::build(&[])));
                if *n > all.len() {
                    *v = vm::log::build(all);
                } else {
                    v.lines.extend(vm::log::build(&all[*n..]).lines);
                }
                *n = all.len();
                let field = &mut self.state.break_at;
                log::show(ui, v, &m.session.breakpoints, field, out);
            }
            Pane::Editor => editor::show(ui, m, &mut self.state.editor, out),
            // Drawn above, with or without a tape.
            Pane::Lab => {}
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
