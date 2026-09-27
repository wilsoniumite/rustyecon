//! The plots (docs/GUI.md §4, §3.4): every plotted series, stacked one panel per unit, the y
//! axis naming the unit, the x axis the report tick labelled by its date. The panels share
//! their x axis and their hover cursor, and each draws the model's cursor; a click sets it.
//!
//! The plot cache thins each series once with [`Decimator`], at about two points per pixel
//! column, extends it as ticks arrive, and lends egui the points it keeps
//! (`PlotPoints::Borrowed`). Every vertex lent is a recorded `(tick, value)`, each column
//! keeps its true least and greatest values, and a gap in the record splits the line, so a
//! gap is never bridged. The cache records what it lent each frame, so a test can check
//! every drawn vertex against the store (`every_drawn_vertex_is_recorded`).

use crate::model::{Cursor, Intent};
use crate::run::{Decimator, RunId, SeriesKey, Store};
use crate::vm::plots::{tick_label, year_marks, PlotsVm};
use egui::Color32;
use egui_plot::{
    log_grid_spacer, GridInput, GridMark, HoverPosition, Legend, Line, Plot, PlotPoint, PlotPoints,
    VLine,
};
use std::collections::BTreeMap;

/// A neutral palette: each line of a panel takes the next, and no colour means good or bad. A
/// test tells the plots' lines from other painted paths by it.
pub const PALETTE: [Color32; 8] = [
    Color32::from_rgb(0x4c, 0x78, 0xa8),
    Color32::from_rgb(0xf5, 0x85, 0x18),
    Color32::from_rgb(0x72, 0xb7, 0xb2),
    Color32::from_rgb(0xb2, 0x79, 0xa2),
    Color32::from_rgb(0x9d, 0x75, 0x5d),
    Color32::from_rgb(0x8c, 0x8c, 0x8c),
    Color32::from_rgb(0x54, 0xa2, 0x4b),
    Color32::from_rgb(0xba, 0xb0, 0xac),
];

/// One cached line.
struct Cached {
    width: u64,
    decimator: Decimator,
    /// The points of the series already taken.
    seen: usize,
    /// The thinned line, one list per unbroken stretch of the record.
    segments: Vec<Vec<PlotPoint>>,
    /// The frame that last drew it.
    drawn: u64,
}

/// One decimator per plotted series of the focused run.
#[derive(Default)]
pub struct PlotCache {
    /// The run and the store's load the lines were taken from.
    source: Option<(RunId, u64)>,
    lines: BTreeMap<SeriesKey, Cached>,
    /// Each panel's x range, as it was last drawn: (lo, hi) in ticks.
    bounds: BTreeMap<String, (f64, f64)>,
    frame: u64,
    /// What this frame lent egui, line by line, read from the very points each `Line` was
    /// made of ([`lend`]).
    lent: Vec<DrawnLine>,
}

/// A line as it was lent to egui in the last frame.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnLine {
    /// The series.
    pub key: SeriesKey,
    /// Its segments' vertices, `[x, y]`, as egui received them: one per `Line` handed to the
    /// plot, in the order they were handed.
    pub segments: Vec<Vec<[f64; 2]>>,
}

/// Lend egui one segment of a line and record what it received. The record is read from the
/// `PlotPoints` the `Line` is made of, after they are made, so what is recorded is what is
/// lent; the line is handed to the plot before it is recorded.
fn lend<'a>(
    pui: &mut egui_plot::PlotUi<'a>,
    lent: &mut Vec<DrawnLine>,
    key: &SeriesKey,
    label: &str,
    colour: Color32,
    seg: &'a [PlotPoint],
) {
    let points = PlotPoints::Borrowed(seg);
    let received: Vec<[f64; 2]> = points.points().iter().map(|p| [p.x, p.y]).collect();
    pui.line(Line::new(label, points).color(colour));
    match lent.last_mut() {
        Some(l) if l.key == *key => l.segments.push(received),
        _ => lent.push(DrawnLine {
            key: key.clone(),
            segments: vec![received],
        }),
    }
}

/// Columns of `2^k` ticks, the fewest that keep a span of `span` ticks within `columns`
/// columns. Widths double as a record grows, so a decimator is rebuilt a few times at most.
fn width_for(span: f64, columns: f64) -> u64 {
    let per = (span / columns.max(1.0)).ceil();
    let per = if per.is_finite() && per >= 1.0 {
        per as u64
    } else {
        1
    };
    per.next_power_of_two()
}

impl PlotCache {
    /// A new frame begins: nothing is lent yet.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
        self.lent.clear();
    }

    /// The lines the last frame lent egui, in the order it drew them, each as egui received
    /// it. Not the cache: what the plots handed over.
    pub fn drawn(&self) -> Vec<DrawnLine> {
        self.lent.clone()
    }

    /// Bring the line of `key` up to date with the store, at columns of `width` ticks.
    fn update(&mut self, store: &Store, key: &SeriesKey, width: u64) {
        let origin = store.start();
        let stale = self
            .lines
            .get(key)
            .is_none_or(|c| c.width != width || store.series(key).map_or(0, |s| s.len()) < c.seen);
        if stale {
            self.lines.insert(
                key.clone(),
                Cached {
                    width,
                    decimator: Decimator::new(origin, width),
                    seen: 0,
                    segments: Vec::new(),
                    drawn: 0,
                },
            );
        }
        let c = self.lines.get_mut(key).expect("inserted above");
        c.drawn = self.frame;
        let Some(series) = store.series(key) else {
            return;
        };
        if series.len() == c.seen {
            return;
        }
        c.decimator.extend(series);
        c.seen = series.len();
        c.segments = c
            .decimator
            .segments()
            .into_iter()
            .map(|s| {
                s.into_iter()
                    .map(|(t, v)| PlotPoint::new(t as f64, v))
                    .collect()
            })
            .collect();
    }
}

/// Draw the plots of the focused run.
pub fn show(
    ui: &mut egui::Ui,
    run: RunId,
    store: &Store,
    vm: &PlotsVm,
    cache: &mut PlotCache,
    out: &mut Vec<Intent>,
) {
    if cache.source != Some((run, store.generation())) {
        cache.source = Some((run, store.generation()));
        cache.lines.clear();
        cache.bounds.clear();
    }
    if !vm.absent.is_empty() {
        let keys: Vec<String> = vm.absent.iter().map(ToString::to_string).collect();
        ui.weak(format!(
            "plotted, but not in this run's world: {}",
            keys.join(", ")
        ));
    }
    if vm.panels.is_empty() {
        ui.weak("nothing plotted: plot a series from the outliner or the inspector");
        return;
    }
    let Some(clock) = store.world().map(|w| w.clock) else {
        return;
    };
    // About two points per pixel column: a decimator keeps each column's least and greatest.
    let columns = f64::from(ui.available_width() * ui.ctx().pixels_per_point());
    let record = (vm.now.saturating_sub(vm.start)).max(1) as f64;
    for p in &vm.panels {
        let span = cache
            .bounds
            .get(&p.unit)
            .map_or(record, |(lo, hi)| (hi - lo).clamp(1.0, record.max(1.0)));
        let width = width_for(span, columns);
        for l in &p.lines {
            cache.update(store, &l.key, width);
        }
    }
    // Drop the lines no longer plotted.
    let frame = cache.frame;
    cache.lines.retain(|_, c| c.drawn == frame);
    let n = vm.panels.len() as f32;
    let height = ((ui.available_height() - 8.0 * n) / n).max(140.0);
    let cursor = vm.cursor;
    let mut bounds = Vec::new();
    let mut clicked = None;
    let lines = &cache.lines;
    let lent = &mut cache.lent;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for p in &vm.panels {
            // Gridlines on year starts, labelled by their year; below two years, egui's own
            // marks, labelled by date.
            let fallback = log_grid_spacer(10);
            let spacer = move |input: GridInput| {
                let (lo, hi) = input.bounds;
                match year_marks(&clock, lo, hi, input.base_step_size) {
                    Some(marks) => marks
                        .into_iter()
                        .map(|(value, step_size)| GridMark { value, step_size })
                        .collect(),
                    None => fallback(input),
                }
            };
            let resp = Plot::new(("plot", p.unit.as_str()))
                .height(height)
                .link_axis("plots", [true, false])
                .link_cursor("plots", [true, false])
                .y_axis_label(p.unit.clone())
                .x_grid_spacer(spacer)
                .x_axis_formatter(move |mark, _| tick_label(&clock, mark.value, mark.step_size))
                .label_formatter({
                    let unit = p.unit.clone();
                    move |h: &HoverPosition<'_>| {
                        let (name, v) = match h {
                            HoverPosition::NearDataPoint {
                                plot_name,
                                position,
                                ..
                            } => (*plot_name, *position),
                            HoverPosition::Elsewhere { position } => ("", *position),
                        };
                        let t = v.x.round().max(0.0) as u64;
                        let date = clock.date_of(t).map_or_else(String::new, |d| d.to_string());
                        let head = if name.is_empty() {
                            String::new()
                        } else {
                            format!("{name}\n")
                        };
                        Some(format!(
                            "{head}{date} (tick {t})\n{} {unit}",
                            super::fmt(v.y)
                        ))
                    }
                })
                .legend(Legend::default())
                .show(ui, |pui| {
                    for (i, l) in p.lines.iter().enumerate() {
                        let colour = PALETTE[i % PALETTE.len()];
                        let Some(c) = lines.get(&l.key) else {
                            continue;
                        };
                        for seg in &c.segments {
                            lend(pui, lent, &l.key, &l.label, colour, seg);
                        }
                    }
                    if let Some(t) = cursor {
                        pui.vline(VLine::new("cursor", t as f64).color(Color32::GRAY));
                    }
                });
            let b = resp.transform.bounds();
            bounds.push((p.unit.clone(), (b.min()[0], b.max()[0])));
            if resp.response.clicked() {
                if let Some(pos) = resp.response.interact_pointer_pos() {
                    clicked = Some(resp.transform.value_from_position(pos).x);
                }
            }
        }
    });
    cache.bounds = bounds.into_iter().collect();
    if let Some(x) = clicked {
        if x.is_finite() && x >= 0.0 {
            out.push(Intent::Cursor(Cursor::At(x.round() as u64)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::width_for;

    #[test]
    fn widths_are_powers_of_two_that_fit_the_columns() {
        assert_eq!(width_for(100.0, 800.0), 1);
        assert_eq!(width_for(800.0, 800.0), 1);
        assert_eq!(width_for(801.0, 800.0), 2);
        assert_eq!(width_for(20_000.0, 1_000.0), 32);
        assert_eq!(width_for(0.0, 0.0), 1);
        assert_eq!(width_for(f64::NAN, 10.0), 1);
    }
}
