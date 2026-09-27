//! What a frame painted, read from its shapes (G1, after its verification): each text with its
//! colour, and each of the other charts' paths and bars against what the chart lent egui
//! (`ui::charts`), mapped to the screen by the transform the plot painted with.

use egui::epaint::{ClippedShape, Shape, TextShape};
use egui::{Color32, Pos2, Rect};
use egui_plot::PlotPoint;
use rustyecon_gui::ui::charts::{Chart, Lent};

/// A painted text: its string, its colour and where it sits.
#[derive(Debug, Clone)]
pub struct Text {
    /// What it says.
    pub text: String,
    /// The colour it is painted in.
    pub colour: Color32,
    /// Where.
    pub rect: Rect,
}

/// The colour a text shape paints in: its override, else its first section's colour, else the
/// painter's fallback where the section leaves it to the painter.
fn colour_of(t: &TextShape) -> Color32 {
    if let Some(c) = t.override_text_color {
        return c;
    }
    match t.galley.job.sections.first().map(|s| s.format.color) {
        Some(c) if c != Color32::PLACEHOLDER => c,
        _ => t.fallback_color,
    }
}

/// Every text the shapes paint, in paint order.
pub fn texts(shapes: &[ClippedShape]) -> Vec<Text> {
    fn walk(s: &Shape, out: &mut Vec<Text>) {
        match s {
            Shape::Text(t) => out.push(Text {
                text: t.galley.text().to_string(),
                colour: colour_of(t),
                rect: t.visual_bounding_rect(),
            }),
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in shapes {
        walk(&c.shape, &mut out);
    }
    out
}

/// How far a painted point may sit from where the transform puts what was lent, in points:
/// screen positions are f32.
const ON_SCREEN: f32 = 0.01;

fn near(a: Pos2, b: Pos2) -> bool {
    (a.x - b.x).abs() < ON_SCREEN && (a.y - b.y).abs() < ON_SCREEN
}

/// An open path painted: its points and its stroke's colour.
type PaintedPath = (Vec<Pos2>, Color32);
/// A rect painted: where, and its stroke's colour.
type PaintedRect = (Rect, Color32);

/// The open paths and the rects a chart's plot painted, in paint order: every shape clipped to
/// the plot's frame. egui_plot paints a line of two or more points as one open path, a vertical
/// or horizontal line as one of two, a bar as a rect, and its grid as line segments.
fn in_frame(shapes: &[ClippedShape], frame: Rect) -> (Vec<PaintedPath>, Vec<PaintedRect>) {
    fn walk(s: &Shape, paths: &mut Vec<PaintedPath>, rects: &mut Vec<PaintedRect>) {
        match s {
            Shape::Path(p) if !p.closed => {
                if let egui::epaint::ColorMode::Solid(c) = p.stroke.color {
                    paths.push((p.points.clone(), c));
                }
            }
            Shape::Rect(r) => rects.push((r.rect, r.stroke.color)),
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, paths, rects)),
            _ => {}
        }
    }
    let (mut paths, mut rects) = (Vec::new(), Vec::new());
    for c in shapes {
        if frame.expand(0.5).contains_rect(c.clip_rect) {
            walk(&c.shape, &mut paths, &mut rects);
        }
    }
    (paths, rects)
}

/// Everything `chart` lent its plot is what the plot painted, and nothing else is painted there
/// as a path: each line of two or more vertices one path of as many points, each point where
/// the plot's transform puts the vertex lent, in the line's colour; each vertical or horizontal
/// line a path of two points at its x or y; each bar a rect stroked in its chart's colour,
/// spanning its argument ± half its width and its base to its base plus its value. Returns how
/// many vertices, marks and bars were checked.
pub fn chart_is_painted(shapes: &[ClippedShape], chart: &Chart) -> usize {
    let t = &chart.transform;
    let at = |x: f64, y: f64| t.position_from_point(&PlotPoint::new(x, y));
    let (paths, rects) = in_frame(shapes, *t.frame());
    let mut paths = paths.into_iter();
    let mut checked = 0;
    for item in &chart.items {
        match item {
            Lent::Line {
                name,
                colour,
                points,
            } if points.len() >= 2 => {
                let (p, c) = paths
                    .next()
                    .unwrap_or_else(|| panic!("{}: line {name} lent and not painted", chart.plot));
                assert_eq!(c, *colour, "{}: line {name}'s colour", chart.plot);
                assert_eq!(
                    p.len(),
                    points.len(),
                    "{}: line {name} painted as lent",
                    chart.plot
                );
                for (q, v) in p.iter().zip(points) {
                    assert!(
                        near(*q, at(v[0], v[1])),
                        "{}: line {name}'s vertex {v:?} is painted at {q:?}, not {:?}",
                        chart.plot,
                        at(v[0], v[1])
                    );
                    checked += 1;
                }
            }
            Lent::Line { .. } => {}
            Lent::VLine { name, colour, x } => {
                let (p, c) = paths
                    .next()
                    .unwrap_or_else(|| panic!("{}: {name} lent and not painted", chart.plot));
                assert_eq!(c, *colour, "{}: {name}'s colour", chart.plot);
                let want = at(*x, 0.0).x;
                assert!(
                    p.len() == 2 && p.iter().all(|q| (q.x - want).abs() < ON_SCREEN),
                    "{}: {name} at x {x} is painted at {p:?}, not x {want}",
                    chart.plot
                );
                checked += 1;
            }
            Lent::HLine { name, colour, y } => {
                let (p, c) = paths
                    .next()
                    .unwrap_or_else(|| panic!("{}: {name} lent and not painted", chart.plot));
                assert_eq!(c, *colour, "{}: {name}'s colour", chart.plot);
                let want = at(0.0, *y).y;
                assert!(
                    p.len() == 2 && p.iter().all(|q| (q.y - want).abs() < ON_SCREEN),
                    "{}: {name} at y {y} is painted at {p:?}, not y {want}",
                    chart.plot
                );
                checked += 1;
            }
            Lent::Bars { name, colour, bars } => {
                let painted: Vec<Rect> = rects
                    .iter()
                    .filter(|(_, c)| c == colour)
                    .map(|(r, _)| *r)
                    .collect();
                assert_eq!(painted.len(), bars.len(), "{}: {name}'s bars", chart.plot);
                for (r, b) in painted.iter().zip(bars) {
                    let half = b.width / 2.0;
                    let want = t.rect_from_values(
                        &PlotPoint::new(b.argument - half, b.base),
                        &PlotPoint::new(b.argument + half, b.base + b.value),
                    );
                    assert!(
                        near(r.min, want.min) && near(r.max, want.max),
                        "{}: bar {b:?} is painted at {r:?}, not {want:?}",
                        chart.plot
                    );
                    checked += 1;
                }
            }
        }
    }
    let left: Vec<usize> = paths.map(|(p, _)| p.len()).collect();
    assert!(
        left.is_empty(),
        "{}: painted and not lent: {left:?}",
        chart.plot
    );
    checked
}
