//! What the other charts lend egui (G1, after its verification): the lab's field over x and its
//! sweep, and a market's log waterfall. Each chart records every item it hands its plot, read
//! from the very values the item was made of, in the order handed over, and the transform the
//! plot painted them with. A test then holds what was lent to the chart's view-model bit for
//! bit, and what was painted to what was lent, as `every_drawn_vertex_is_recorded` holds the
//! plots' lines through [`super::plots::DrawnLine`].

use egui::Color32;
use egui_plot::{Bar, BarChart, HLine, Line, Plot, PlotPoints, PlotTransform, PlotUi, VLine};

/// A bar as the plot received it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LentBar {
    /// Its place on the x axis.
    pub argument: f64,
    /// Its height, from its base.
    pub value: f64,
    /// Where it starts: 0 unless stacked.
    pub base: f64,
    /// Its width.
    pub width: f64,
}

/// One item handed to a chart's plot.
#[derive(Debug, Clone, PartialEq)]
pub enum Lent {
    /// A line: its name and its vertices, `[x, y]`, as the plot received them.
    Line {
        /// Its name in the legend.
        name: String,
        /// Its colour.
        colour: Color32,
        /// Its vertices.
        points: Vec<[f64; 2]>,
    },
    /// A vertical line at `x`.
    VLine {
        /// Its name in the legend.
        name: String,
        /// Its colour.
        colour: Color32,
        /// Where.
        x: f64,
    },
    /// A horizontal line at `y`.
    HLine {
        /// Its name in the legend.
        name: String,
        /// Its colour.
        colour: Color32,
        /// Where.
        y: f64,
    },
    /// A chart of bars.
    Bars {
        /// Its name in the legend.
        name: String,
        /// Its colour: each bar is stroked in it.
        colour: Color32,
        /// Its bars, in order.
        bars: Vec<LentBar>,
    },
}

/// A chart as the last frame drew it.
#[derive(Debug, Clone)]
pub struct Chart {
    /// The plot's id: `lab-curve-plot`, `lab-sweep-plot` or `waterfall`.
    pub plot: String,
    /// What was handed to it, in order.
    pub items: Vec<Lent>,
    /// The transform its items were painted with, from the plot's response.
    pub transform: PlotTransform,
}

/// The charts the last frame drew.
#[derive(Default)]
pub struct Charts {
    frame: Vec<Chart>,
}

impl Charts {
    /// A new frame begins: no chart is drawn yet.
    pub fn begin_frame(&mut self) {
        self.frame.clear();
    }

    /// The charts the last frame drew, in the order drawn.
    pub fn drawn(&self) -> Vec<Chart> {
        self.frame.clone()
    }
}

/// A chart's plot while it is built: each item lent through it is handed to the plot and
/// recorded. The plot keeps no empty line or bar chart, and neither does the record.
pub struct Lender {
    items: Vec<Lent>,
}

impl Lender {
    /// Lend a line of `points`. What is recorded is read from the `PlotPoints` the line is made
    /// of, after they are made.
    pub fn line(&mut self, pui: &mut PlotUi<'_>, name: &str, points: Vec<[f64; 2]>, c: Color32) {
        let points = PlotPoints::from(points);
        let received: Vec<[f64; 2]> = points.points().iter().map(|p| [p.x, p.y]).collect();
        pui.line(Line::new(name, points).color(c));
        if !received.is_empty() {
            self.items.push(Lent::Line {
                name: name.to_string(),
                colour: c,
                points: received,
            });
        }
    }

    /// Lend a vertical line at `x`.
    pub fn vline(&mut self, pui: &mut PlotUi<'_>, name: &str, x: f64, c: Color32) {
        pui.vline(VLine::new(name, x).color(c));
        self.items.push(Lent::VLine {
            name: name.to_string(),
            colour: c,
            x,
        });
    }

    /// Lend a horizontal line at `y`.
    pub fn hline(&mut self, pui: &mut PlotUi<'_>, name: &str, y: f64, c: Color32) {
        pui.hline(HLine::new(name, y).color(c));
        self.items.push(Lent::HLine {
            name: name.to_string(),
            colour: c,
            y,
        });
    }

    /// Lend a chart of `bars`. What is recorded is read from each bar as made.
    pub fn bars(&mut self, pui: &mut PlotUi<'_>, name: &str, bars: Vec<Bar>, c: Color32) {
        let lent: Vec<LentBar> = bars
            .iter()
            .map(|b| LentBar {
                argument: b.argument,
                value: b.value,
                base: b.base_offset.unwrap_or(0.0),
                width: b.bar_width,
            })
            .collect();
        pui.bar_chart(BarChart::new(name, bars).color(c));
        if !lent.is_empty() {
            self.items.push(Lent::Bars {
                name: name.to_string(),
                colour: c,
                bars: lent,
            });
        }
    }
}

/// Show `plot`, named `id`, built by `build` through a [`Lender`], and record what it lent and
/// the transform it painted with.
pub fn show<'a>(
    charts: &mut Charts,
    id: &str,
    plot: Plot<'a>,
    ui: &mut egui::Ui,
    build: impl FnOnce(&mut PlotUi<'a>, &mut Lender) + 'a,
) {
    let r = plot.show(ui, move |pui| {
        let mut lender = Lender { items: Vec::new() };
        build(pui, &mut lender);
        lender.items
    });
    charts.frame.push(Chart {
        plot: id.to_string(),
        items: r.inner,
        transform: r.transform,
    });
}
