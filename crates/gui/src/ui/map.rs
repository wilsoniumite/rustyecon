//! The map (docs/GUI.md §4 "Lenses", §6; D.3, 2026-09-27): the atlas's regions coloured by a
//! lens, with a legend, a lens selector with hotkeys, the ranked table and the selected county.
//!
//! - **Geometry, once.** Each region's parts are triangulated with earcut, holes included, when
//!   the map is first drawn ([`Geo::build`]). Each frame moves the triangles into the view and
//!   colours them as one mesh. Each shared border is stored once in the atlas and stroked once,
//!   thinned in screen space so that no two kept points lie within [`THIN_PX`] of each other.
//!   Coordinates are British National Grid metres; the view is a centre and a scale.
//! - **Looks.** A muted sea with a pale band along the coast, the regions in the lens's
//!   colours, thin county borders, heavier lines where England meets Wales and Scotland, the
//!   coast in dark ink, and the names wherever they fit. A region the tape lacks is hatched
//!   grey; a region with no value is plain grey, and its hover card says why.
//! - **Colours.** Neutral scales: viridis for a sequential lens, purple to orange through
//!   white for a diverging one, and nothing good or bad (U-rules). A value outside the fixed
//!   domain takes the end colour, and the legend and the card say so.
//! - **Hands.** Drag pans, the wheel zooms about the pointer, a double click fits the map; a
//!   click selects a county, which the inspector, the outliner and the plots then follow by
//!   key; `1`–`9` and `0` pick the first ten lenses and `[` and `]` step through all of them.
//! - **One view-model.** The map, the legend and the table draw the same [`LensVm`]: the
//!   table's rows index the map's regions, so the map's values are the table's (G4's gate).
//!   Each frame records what it drew ([`MapFrame`]) so a test can hold the two to each other.

use super::fmt;
use crate::model::Intent;
use crate::run::{Entity, SeriesKey};
use crate::vm::map::{Beyond, LensVm, MapVm, RegionVm};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2,
};
use egui_extras::{Column, TableBuilder};
use rustyecon_engine::prelude::Key;
use rustyecon_worldgen::atlas::{Atlas, Country, Part};
use rustyecon_worldgen::tables::Lens;
use std::collections::BTreeMap;

/// The lens a new map shows: the wage in land-service units, w/r, the paper's v and the
/// fork's falling branch.
pub const DEFAULT_LENS: &str = "wage.land";

/// No two points of a drawn border lie closer than this, in screen pixels.
pub const THIN_PX: f32 = 0.75;

/// The sidebar's width, in points.
const SIDEBAR: f32 = 270.0;

/// What a region's fill is made of: its triangles in grid metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    /// Every vertex of every part, rings without their closing point.
    pub points: Vec<[f64; 2]>,
    /// Triangles, three indices into `points` each.
    pub triangles: Vec<u32>,
    /// Its bounding box: west, south, east, north.
    pub bbox: [f64; 4],
}

impl Fill {
    /// The area its triangles cover, square metres.
    pub fn area(&self) -> f64 {
        self.triangles
            .chunks_exact(3)
            .map(|t| {
                let [a, b, c] = [t[0], t[1], t[2]].map(|i| self.points[i as usize]);
                ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
            })
            .sum()
    }
}

/// What a border separates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// A region from the sea.
    Coast,
    /// Two regions of one country.
    County,
    /// Two countries: England from Wales or from Scotland.
    Nation,
}

/// One border of the atlas, stored once.
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    /// Its points, grid metres.
    pub points: Vec<[f64; 2]>,
    /// What it separates.
    pub edge: Edge,
}

/// The atlas, triangulated once.
#[derive(Debug, Clone, PartialEq)]
pub struct Geo {
    /// The atlas.
    pub atlas: Atlas,
    /// Every region's fill, in the atlas's order.
    pub fills: Vec<Fill>,
    /// Every border, in the atlas's order.
    pub borders: Vec<Border>,
    /// The whole atlas's box: west, south, east, north.
    pub bounds: [f64; 4],
}

fn bbox(points: impl IntoIterator<Item = [f64; 2]>) -> [f64; 4] {
    let mut b = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
    for [x, y] in points {
        b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
    }
    b
}

/// Triangulate one part, holes included, into `fill`.
fn triangulate(part: &Part, cut: &mut earcut::Earcut<f64>, fill: &mut Fill) {
    let base = fill.points.len();
    let mut data = Vec::new();
    let mut holes: Vec<u32> = Vec::new();
    for (k, ring) in part.rings.iter().enumerate() {
        if k > 0 {
            holes.push(u32::try_from(data.len()).expect("a part of fewer than 2^32 points"));
        }
        // A ring is closed; earcut takes it open.
        data.extend_from_slice(&ring[..ring.len().saturating_sub(1)]);
    }
    let mut tris: Vec<u32> = Vec::new();
    cut.earcut(data.iter().copied(), &holes, &mut tris);
    let base = u32::try_from(base).expect("fewer than 2^32 points");
    fill.triangles.extend(tris.into_iter().map(|i| base + i));
    fill.points.extend(data);
}

impl Geo {
    /// Triangulate every region and classify every border.
    pub fn build(atlas: Atlas) -> Geo {
        let mut cut = earcut::Earcut::new();
        let fills: Vec<Fill> = atlas
            .regions
            .iter()
            .map(|r| {
                let mut fill = Fill {
                    points: Vec::new(),
                    triangles: Vec::new(),
                    bbox: [0.0; 4],
                };
                for part in &r.parts {
                    triangulate(part, &mut cut, &mut fill);
                }
                fill.bbox = bbox(fill.points.iter().copied());
                fill
            })
            .collect();
        let borders = atlas
            .borders
            .iter()
            .map(|b| {
                let edge = match b.right {
                    None => Edge::Coast,
                    Some(r) if atlas.regions[r].country != atlas.regions[b.left].country => {
                        // Northern Ireland shares no border with Great Britain.
                        let pair = [atlas.regions[r].country, atlas.regions[b.left].country];
                        if pair.contains(&Country::England) {
                            Edge::Nation
                        } else {
                            Edge::County
                        }
                    }
                    Some(_) => Edge::County,
                };
                Border {
                    points: b.points.clone(),
                    edge,
                }
            })
            .collect();
        let bounds = bbox(
            fills
                .iter()
                .flat_map(|f| [[f.bbox[0], f.bbox[1]], [f.bbox[2], f.bbox[3]]]),
        );
        Geo {
            atlas,
            fills,
            borders,
            bounds,
        }
    }

    /// The region under a point of the grid, if one is.
    pub fn region_at(&self, p: [f64; 2]) -> Option<usize> {
        self.fills.iter().enumerate().find_map(|(i, f)| {
            let b = f.bbox;
            let inside_box = p[0] >= b[0] && p[0] <= b[2] && p[1] >= b[1] && p[1] <= b[3];
            (inside_box && self.atlas.regions[i].contains(p)).then_some(i)
        })
    }
}

/// Where the map looks: a centre in grid metres and a scale in points a metre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// The grid point at the canvas's centre.
    pub centre: [f64; 2],
    /// Points on screen per metre of the grid.
    pub scale: f64,
}

impl View {
    /// The view that fits `bounds` into `rect` with a margin.
    pub fn fit(bounds: [f64; 4], rect: Rect) -> View {
        let w = (bounds[2] - bounds[0]).max(1.0);
        let h = (bounds[3] - bounds[1]).max(1.0);
        let scale = (f64::from(rect.width()) / w).min(f64::from(rect.height()) / h) * 0.94;
        View {
            centre: [(bounds[0] + bounds[2]) / 2.0, (bounds[1] + bounds[3]) / 2.0],
            scale: scale.max(1e-9),
        }
    }

    /// A grid point on screen. North is up.
    pub fn to_screen(&self, rect: Rect, p: [f64; 2]) -> Pos2 {
        let c = rect.center();
        pos2(
            c.x + ((p[0] - self.centre[0]) * self.scale) as f32,
            c.y - ((p[1] - self.centre[1]) * self.scale) as f32,
        )
    }

    /// A screen point on the grid.
    pub fn to_grid(&self, rect: Rect, q: Pos2) -> [f64; 2] {
        let c = rect.center();
        [
            self.centre[0] + f64::from(q.x - c.x) / self.scale,
            self.centre[1] - f64::from(q.y - c.y) / self.scale,
        ]
    }
}

/// A region as the last frame drew it.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnRegion {
    /// Its key.
    pub key: String,
    /// The value its colour stands for, from the view-model.
    pub value: Option<f64>,
    /// Its fill colour.
    pub colour: Color32,
    /// Whether it was hatched: a region the tape lacks.
    pub hatched: bool,
    /// How many times its triangles went into the frame's mesh.
    pub fills: u32,
    /// Its label point on screen.
    pub label: Pos2,
}

/// A row of the ranked table as the last frame drew it.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnRow {
    /// The region's key.
    pub key: String,
    /// Its rank.
    pub rank: usize,
    /// The value the row shows.
    pub value: f64,
    /// Its text.
    pub text: String,
    /// Its swatch's colour.
    pub colour: Color32,
}

/// What the map pane drew in the last frame, for the tests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapFrame {
    /// The lens shown.
    pub lens: Option<String>,
    /// The report tick the values are read at.
    pub tick: Option<u64>,
    /// Every region drawn, in the atlas's order.
    pub regions: Vec<DrawnRegion>,
    /// How many times each border was stroked, in the atlas's order.
    pub borders: Vec<u32>,
    /// The ranked table's rows drawn.
    pub rows: Vec<DrawnRow>,
    /// The hover card's lines, when one was drawn.
    pub hover: Option<Vec<String>>,
    /// The vertices of the fill mesh.
    pub vertices: usize,
    /// Why there was no map, when there was none.
    pub refused: Option<String>,
}

/// The map's state between frames.
pub struct MapState {
    geo: Option<Result<Geo, String>>,
    lenses: Option<Result<Vec<Lens>, String>>,
    /// The lens shown, by key.
    pub lens: String,
    view: Option<View>,
    /// The region the table's pointer is over, so the map can mark it.
    hot: Option<String>,
    frame: MapFrame,
    /// The last frame's geometry on screen, kept while the view, the canvas and the theme
    /// are: a paused map redraws the same mesh, and a running one recolours it in place.
    painted: Option<Painted>,
}

/// The map's geometry on screen for one view: the fill mesh with its spans, the colours it
/// has, and the borders' strokes.
struct Painted {
    key: ([u64; 3], [u32; 4], bool),
    colours: Vec<Color32>,
    mesh: std::sync::Arc<Mesh>,
    spans: Vec<(usize, usize, usize)>,
    shallows: Vec<Shape>,
    lines: Vec<Shape>,
}

impl Default for MapState {
    fn default() -> MapState {
        MapState {
            geo: None,
            lenses: None,
            lens: DEFAULT_LENS.to_string(),
            view: None,
            hot: None,
            frame: MapFrame::default(),
            painted: None,
        }
    }
}

impl MapState {
    /// The atlas, triangulated on first use.
    pub fn geo(&mut self) -> Result<&Geo, String> {
        let g = self
            .geo
            .get_or_insert_with(|| Atlas::gb().map(Geo::build).map_err(|e| e.to_string()));
        g.as_ref().map_err(Clone::clone)
    }

    /// The demo world's lenses, read on first use.
    pub fn lenses(&mut self) -> Result<&[Lens], String> {
        let l = self
            .lenses
            .get_or_insert_with(|| rustyecon_worldgen::lens::demo_gb().map_err(|e| e.to_string()));
        l.as_deref().map_err(Clone::clone)
    }

    /// Read the atlas and the lenses, once.
    pub fn ready(&mut self) -> Result<(), String> {
        self.geo()?;
        self.lenses()?;
        Ok(())
    }

    /// The atlas and the lenses, once read.
    pub fn parts(&self) -> Option<(&Geo, &[Lens])> {
        match (&self.geo, &self.lenses) {
            (Some(Ok(g)), Some(Ok(l))) => Some((g, l)),
            _ => None,
        }
    }

    /// What the last frame drew.
    pub fn frame(&self) -> &MapFrame {
        &self.frame
    }

    /// Fit the map to the canvas on the next frame.
    pub fn fit(&mut self) {
        self.view = None;
    }
}

/// The colour of a place on a lens's scale: viridis for a sequential scale, purple to orange
/// through white for a diverging one.
pub fn colour(scale: &str, at: f64) -> Color32 {
    let g = if scale == "diverging" {
        colorous::PURPLE_ORANGE
    } else {
        colorous::VIRIDIS
    };
    let c = g.eval_continuous(at.clamp(0.0, 1.0));
    Color32::from_rgb(c.r, c.g, c.b)
}

/// The palette the map draws with, for the light and dark themes.
struct Ink {
    sea: Color32,
    shallows: Color32,
    coast: Color32,
    county: Color32,
    nation: Color32,
    none: Color32,
    missing: Color32,
    hatch: Color32,
    panel: Color32,
    text: Color32,
    weak: Color32,
}

fn ink(dark: bool) -> Ink {
    if dark {
        Ink {
            sea: Color32::from_rgb(0x17, 0x22, 0x2e),
            shallows: Color32::from_rgba_unmultiplied(0x3c, 0x5a, 0x74, 0x90),
            coast: Color32::from_rgb(0x0b, 0x12, 0x1a),
            county: Color32::from_rgba_unmultiplied(0x10, 0x14, 0x1a, 0xa0),
            nation: Color32::from_rgba_unmultiplied(0x08, 0x0a, 0x0e, 0xe6),
            none: Color32::from_rgb(0x5c, 0x5f, 0x64),
            missing: Color32::from_rgb(0x44, 0x47, 0x4c),
            hatch: Color32::from_rgb(0x74, 0x77, 0x7c),
            panel: Color32::from_rgba_unmultiplied(0x12, 0x16, 0x1c, 0xe0),
            text: Color32::from_rgb(0xe8, 0xe8, 0xe8),
            weak: Color32::from_rgb(0xa8, 0xad, 0xb4),
        }
    } else {
        Ink {
            sea: Color32::from_rgb(0xb7, 0xcc, 0xdb),
            shallows: Color32::from_rgba_unmultiplied(0xe2, 0xec, 0xf3, 0xc0),
            coast: Color32::from_rgb(0x2a, 0x3a, 0x4a),
            county: Color32::from_rgba_unmultiplied(0x20, 0x24, 0x2a, 0x90),
            nation: Color32::from_rgba_unmultiplied(0x14, 0x16, 0x1a, 0xdc),
            none: Color32::from_rgb(0x9a, 0x9d, 0xa2),
            missing: Color32::from_rgb(0xc9, 0xcb, 0xce),
            hatch: Color32::from_rgb(0x8a, 0x8d, 0x92),
            panel: Color32::from_rgba_unmultiplied(0xf8, 0xf8, 0xf6, 0xe8),
            text: Color32::from_rgb(0x1c, 0x1c, 0x1c),
            weak: Color32::from_rgb(0x55, 0x5a, 0x60),
        }
    }
}

/// Black or white, whichever reads better on `c`.
fn on(c: Color32) -> Color32 {
    let l = 0.299 * f32::from(c.r()) + 0.587 * f32::from(c.g()) + 0.114 * f32::from(c.b());
    if l > 140.0 {
        Color32::from_rgb(0x14, 0x14, 0x14)
    } else {
        Color32::from_rgb(0xf4, 0xf4, 0xf4)
    }
}

/// A region's fill colour under a lens, and whether it is hatched.
fn fill_of(r: &RegionVm, scale: &str, ink: &Ink) -> (Color32, bool) {
    match (r.in_tape, r.at) {
        (false, _) => (ink.missing, true),
        (true, Some(at)) => (colour(scale, at), false),
        (true, None) => (ink.none, false),
    }
}

/// The fill mesh of every region in `colours` (a colour per region, in the atlas's order),
/// seen through `view` on `rect`.
pub fn fill_mesh(geo: &Geo, colours: &[Color32], view: &View, rect: Rect) -> Mesh {
    fill_mesh_spans(geo, colours, view, rect).0
}

/// [`fill_mesh`], with the span of the mesh's vertices each region's triangles took, in the
/// order they went in: `(region, first vertex, vertices)`.
fn fill_mesh_spans(
    geo: &Geo,
    colours: &[Color32],
    view: &View,
    rect: Rect,
) -> (Mesh, Vec<(usize, usize, usize)>) {
    let mut mesh = Mesh::default();
    let n: usize = geo.fills.iter().map(|f| f.points.len()).sum();
    mesh.reserve_vertices(n);
    let mut spans = Vec::with_capacity(geo.fills.len());
    for (i, (f, c)) in geo.fills.iter().zip(colours).enumerate() {
        let first = mesh.vertices.len();
        let base = u32::try_from(first).expect("fewer than 2^32 vertices");
        for &p in &f.points {
            mesh.colored_vertex(view.to_screen(rect, p), *c);
        }
        mesh.indices.extend(f.triangles.iter().map(|i| base + i));
        spans.push((i, first, f.points.len()));
    }
    (mesh, spans)
}

/// A border on screen, thinned: consecutive kept points at least [`THIN_PX`] apart, both ends
/// kept.
pub fn thinned(points: &[[f64; 2]], view: &View, rect: Rect) -> Vec<Pos2> {
    let mut out: Vec<Pos2> = Vec::with_capacity(points.len().min(64));
    let last = points.len().saturating_sub(1);
    for (i, &p) in points.iter().enumerate() {
        let q = view.to_screen(rect, p);
        match out.last() {
            Some(&l) if i != last && (q - l).length_sq() < THIN_PX * THIN_PX => {}
            _ => out.push(q),
        }
    }
    out
}

/// The strokes of every border, thinned, each once: the coast's band first, then the borders.
pub fn border_shapes(geo: &Geo, view: &View, rect: Rect, dark: bool) -> (Vec<Shape>, Vec<Shape>) {
    let ink = ink(dark);
    let mut shallows = Vec::new();
    let mut lines = Vec::new();
    let viewport = rect.expand(8.0);
    for b in &geo.borders {
        let pts = thinned(&b.points, view, rect);
        if pts.len() < 2 || !pts.iter().any(|p| viewport.contains(*p)) {
            // Off screen: not stroked, and kept in place so the list stays one per border.
            lines.push(Shape::Noop);
            continue;
        }
        if b.edge == Edge::Coast {
            shallows.push(Shape::line(pts.clone(), Stroke::new(7.0, ink.shallows)));
        }
        lines.push(Shape::line(pts, border_stroke(b.edge, dark)));
    }
    (shallows, lines)
}

/// The stroke a border takes: the coast in dark ink, county lines thin, the lines between
/// England and Wales or Scotland heavier.
pub fn border_stroke(edge: Edge, dark: bool) -> Stroke {
    let ink = ink(dark);
    match edge {
        Edge::Coast => Stroke::new(1.3, ink.coast),
        Edge::County => Stroke::new(0.8, ink.county),
        Edge::Nation => Stroke::new(2.2, ink.nation),
    }
}

/// Hatch lines across a region's triangles on screen, `gap` points apart.
fn hatch(fill: &Fill, view: &View, rect: Rect, gap: f32, stroke: Stroke) -> Vec<Shape> {
    let mut out = Vec::new();
    for t in fill.triangles.chunks_exact(3) {
        let q = [t[0], t[1], t[2]].map(|i| view.to_screen(rect, fill.points[i as usize]));
        // Lines x + y = c, crossing the triangle between its least and greatest c.
        let u = q.map(|p| p.x + p.y);
        let lo = u.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = u.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut c = (lo / gap).ceil() * gap;
        while c <= hi {
            let mut hits = Vec::with_capacity(2);
            for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                let (ua, ub) = (u[a], u[b]);
                if (ua - c) * (ub - c) <= 0.0 && ua != ub {
                    let s = (c - ua) / (ub - ua);
                    hits.push(q[a] + (q[b] - q[a]) * s);
                }
            }
            if hits.len() >= 2 {
                out.push(Shape::line_segment([hits[0], hits[1]], stroke));
            }
            c += gap;
        }
    }
    out
}

/// Handle the lens hotkeys: `1`–`9` and `0` pick the first ten lenses, `[` and `]` step.
fn hotkeys(ui: &egui::Ui, lenses: &[crate::vm::map::LensItemVm], state: &mut MapState) {
    if ui.ctx().egui_wants_keyboard_input() || lenses.is_empty() {
        return;
    }
    let digits = [
        egui::Key::Num1,
        egui::Key::Num2,
        egui::Key::Num3,
        egui::Key::Num4,
        egui::Key::Num5,
        egui::Key::Num6,
        egui::Key::Num7,
        egui::Key::Num8,
        egui::Key::Num9,
        egui::Key::Num0,
    ];
    let now = lenses.iter().position(|l| l.key == state.lens).unwrap_or(0);
    let mut pick = None;
    ui.input(|i| {
        for (k, key) in digits.iter().enumerate() {
            if i.key_pressed(*key) && k < lenses.len() {
                pick = Some(k);
            }
        }
        if i.key_pressed(egui::Key::CloseBracket) {
            pick = Some((now + 1) % lenses.len());
        }
        if i.key_pressed(egui::Key::OpenBracket) {
            pick = Some((now + lenses.len() - 1) % lenses.len());
        }
    });
    if let Some(k) = pick {
        state.lens = lenses[k].key.clone();
    }
}

/// The hotkey of the `k`th lens.
fn hotkey(k: usize) -> Option<&'static str> {
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]
        .get(k)
        .copied()
}

/// The run a value belongs to (U3), in two lines: the tape, its origin and the build; then
/// the `tape_hash` and the `world_id`.
fn run_lines(vm: &LensVm) -> [String; 2] {
    match &vm.run {
        Some(r) => [
            format!(
                "{}, {} of build {}{}",
                r.name,
                r.origin,
                r.commit.get(..7).unwrap_or(&r.commit),
                if r.dirty { " (dirty)" } else { "" },
            ),
            format!("tape_hash {} · world_id {}", r.tape_hash, r.world_id),
        ],
        None => ["no run".to_string(), String::new()],
    }
}

/// The run a value belongs to, on one line.
fn run_line(vm: &LensVm) -> String {
    let [a, b] = run_lines(vm);
    format!("{a} · {b}")
}

fn when(vm: &LensVm) -> String {
    match vm.tick {
        Some(t) => format!("report tick {t}, {}", vm.date),
        None => "no tick has run".to_string(),
    }
}

/// A value with its unit and where it sits against the domain.
fn value_text(v: f64, unit: &str, beyond: Beyond) -> String {
    let mark = match beyond {
        Beyond::Within => "",
        Beyond::Below => " (below the scale)",
        Beyond::Above => " (above the scale)",
    };
    format!("{} {unit}{mark}", fmt(v))
}

/// Draw the map pane: the canvas and the sidebar.
pub fn show(ui: &mut egui::Ui, vm: &MapVm, state: &mut MapState, out: &mut Vec<Intent>) {
    state.frame = MapFrame {
        refused: vm.refused.clone(),
        ..MapFrame::default()
    };
    if let Some(why) = &vm.refused {
        ui.add_space(8.0);
        ui.weak(why);
        return;
    }
    let Some(lens) = &vm.lens else {
        ui.weak("no lens to show");
        return;
    };
    hotkeys(ui, &vm.lenses, state);
    if let Err(e) = state.geo() {
        ui.colored_label(ui.visuals().error_fg_color, e);
        return;
    }
    // Held apart for the frame, so the canvas can write the rest of the state.
    let Some(Ok(geo)) = state.geo.take() else {
        return;
    };
    let full = ui.available_rect_before_wrap();
    let side_w = SIDEBAR.min(full.width() * 0.4);
    let canvas_rect = Rect::from_min_max(full.min, pos2(full.max.x - side_w - 6.0, full.max.y));
    let side = Rect::from_min_max(pos2(full.max.x - side_w, full.min.y), full.max);
    ui.allocate_rect(full, Sense::hover());
    let mut canvas_ui = ui.new_child(egui::UiBuilder::new().max_rect(canvas_rect));
    canvas(&mut canvas_ui, &geo, vm, lens, state, out);
    let mut side_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(side)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    sidebar(&mut side_ui, vm, lens, state, out);
    state.geo = Some(Ok(geo));
}

fn canvas(
    ui: &mut egui::Ui,
    geo: &Geo,
    vm: &MapVm,
    lens: &LensVm,
    state: &mut MapState,
    out: &mut Vec<Intent>,
) {
    let rect = ui.max_rect();
    let resp = ui.interact(rect, ui.id().with("map-canvas"), Sense::click_and_drag());
    let dark = ui.visuals().dark_mode;
    let ink = ink(dark);
    let mut view = *state
        .view
        .get_or_insert_with(|| View::fit(geo.bounds, rect));
    // Pan, zoom about the pointer, fit on a double click.
    if resp.dragged() {
        let d = resp.drag_delta();
        view.centre[0] -= f64::from(d.x) / view.scale;
        view.centre[1] += f64::from(d.y) / view.scale;
    }
    if resp.double_clicked() {
        view = View::fit(geo.bounds, rect);
    }
    if let Some(p) = resp.hover_pos() {
        let (scroll, zoom) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
        let factor = f64::from(zoom) * (1.0 + f64::from(scroll) * 0.0015).clamp(0.5, 2.0);
        if (factor - 1.0).abs() > 1e-6 {
            let fit = View::fit(geo.bounds, rect).scale;
            let before = view.to_grid(rect, p);
            view.scale = (view.scale * factor).clamp(fit * 0.5, fit * 60.0);
            let after = view.to_grid(rect, p);
            view.centre[0] += before[0] - after[0];
            view.centre[1] += before[1] - after[1];
        }
    }
    state.view = Some(view);
    let hovered = resp
        .hover_pos()
        .and_then(|p| geo.region_at(view.to_grid(rect, p)));
    if resp.clicked() {
        if let Some(i) = resp
            .interact_pointer_pos()
            .and_then(|p| geo.region_at(view.to_grid(rect, p)))
        {
            if let Ok(k) = Key::new(geo.atlas.regions[i].key.clone()) {
                out.push(Intent::Select(Some(Entity::Node(k))));
            }
        }
    }
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 0.0, ink.sea);
    // Colours, one per region, from the view-model.
    let mut colours = Vec::with_capacity(geo.fills.len());
    let mut hatched = Vec::with_capacity(geo.fills.len());
    for (i, r) in lens.regions.iter().enumerate() {
        let (c, h) = fill_of(r, &lens.scale, &ink);
        let c = if hovered == Some(i) || state.hot.as_deref() == Some(r.key.as_str()) {
            c.gamma_multiply(0.82).to_opaque()
        } else {
            c
        };
        colours.push(c);
        hatched.push(h);
    }
    // The geometry on screen: made again when the view, the canvas or the theme moves, and
    // recoloured in place when only the colours change.
    let key = (
        [
            view.centre[0].to_bits(),
            view.centre[1].to_bits(),
            view.scale.to_bits(),
        ],
        [rect.min.x, rect.min.y, rect.max.x, rect.max.y].map(f32::to_bits),
        dark,
    );
    let cached = state.painted.take().filter(|c| c.key == key);
    let mut g = match cached {
        Some(c) => c,
        None => {
            let (shallows, lines) = border_shapes(geo, &view, rect, dark);
            let (mesh, spans) = fill_mesh_spans(geo, &colours, &view, rect);
            Painted {
                key,
                colours: colours.clone(),
                mesh: std::sync::Arc::new(mesh),
                spans,
                shallows,
                lines,
            }
        }
    };
    if g.colours != colours {
        let m = std::sync::Arc::make_mut(&mut g.mesh);
        for &(i, first, n) in &g.spans {
            for v in &mut m.vertices[first..first + n] {
                v.color = colours[i];
            }
        }
        g.colours.clone_from(&colours);
    }
    for s in &g.shallows {
        p.add(s.clone());
    }
    let (mesh, spans) = (std::sync::Arc::clone(&g.mesh), g.spans.clone());
    let lines = g.lines.clone();
    state.painted = Some(g);
    let vertices = mesh.vertices.len();
    // What each region's fill went in as: how many times, and the colour its vertices took.
    let mut fills = vec![0_u32; geo.fills.len()];
    let mut painted = vec![Color32::TRANSPARENT; geo.fills.len()];
    for &(i, first, n) in &spans {
        fills[i] += 1;
        if n > 0 {
            painted[i] = mesh.vertices[first].color;
        }
    }
    p.add(Shape::mesh(mesh));
    // Hatch the regions the tape lacks, a line every 7 points.
    for (i, f) in geo.fills.iter().enumerate() {
        if hatched[i] {
            for s in hatch(f, &view, rect, 7.0, Stroke::new(1.0, ink.hatch)) {
                p.add(s);
            }
        }
    }
    // Each border's stroke, once; one off screen is not stroked.
    let mut stroked = vec![0_u32; geo.borders.len()];
    for (k, s) in lines.into_iter().enumerate() {
        if !matches!(s, Shape::Noop) {
            stroked[k] += 1;
            p.add(s);
        }
    }
    // The selected county and the hovered one, outlined.
    let selected = vm.county.as_ref().map(|c| c.key.clone());
    for (i, r) in geo.atlas.regions.iter().enumerate() {
        let sel = selected.as_deref() == Some(r.key.as_str());
        let hot = hovered == Some(i) || state.hot.as_deref() == Some(r.key.as_str());
        if !sel && !hot {
            continue;
        }
        for part in &r.parts {
            for ring in &part.rings {
                let pts = thinned(ring, &view, rect);
                if sel {
                    p.add(Shape::line(pts.clone(), Stroke::new(4.0, Color32::BLACK)));
                    p.add(Shape::line(pts, Stroke::new(2.2, Color32::WHITE)));
                } else {
                    p.add(Shape::line(pts, Stroke::new(1.6, ink.text)));
                }
            }
        }
    }
    // Names wherever they fit: the full name, else the Chapman code.
    let mut drawn = Vec::with_capacity(geo.fills.len());
    for (i, r) in geo.atlas.regions.iter().enumerate() {
        let at = view.to_screen(rect, r.label);
        let size = (r.area_km2.sqrt() * 1000.0 * view.scale) as f32;
        let font = FontId::proportional((size / 9.0).clamp(10.0, 13.0));
        let w = |s: &str| s.chars().count() as f32 * font.size * 0.52;
        let text = if w(&r.name) <= size * 1.25 {
            Some(r.name.as_str())
        } else if w(&r.chapman) <= size * 1.2 {
            Some(r.chapman.as_str())
        } else {
            None
        };
        if let (Some(t), true) = (text, rect.contains(at)) {
            let fg = on(colours[i]);
            let halo = if fg == Color32::WHITE || fg.r() > 128 {
                Color32::from_black_alpha(150)
            } else {
                Color32::from_white_alpha(150)
            };
            for d in [
                vec2(1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(0.0, 1.0),
                vec2(0.0, -1.0),
            ] {
                p.text(at + d, Align2::CENTER_CENTER, t, font.clone(), halo);
            }
            p.text(at, Align2::CENTER_CENTER, t, font, fg);
        }
        drawn.push(DrawnRegion {
            key: r.key.clone(),
            value: lens.regions[i].value,
            colour: painted[i],
            hatched: hatched[i],
            fills: fills[i],
            label: at,
        });
    }
    overlay_title(&p, rect, lens, &ink);
    legend(&p, rect, lens, &ink);
    scale_bar(&p, rect, &view, &ink);
    if lens.tick.is_none() {
        // Before the first tick there is no report, so no value (docs/GUI.md §4: a cursor is
        // a report tick).
        let lines = [
            (
                "No tick has run".to_string(),
                FontId::proportional(16.0),
                ink.text,
            ),
            (
                "Space runs, `.` steps a tick, the toolbar steps a year".to_string(),
                FontId::proportional(12.0),
                ink.weak,
            ),
        ];
        panel(&p, rect, rect.center() - vec2(150.0, 30.0), &lines, &ink);
    }
    // The hover card: the county, the value with its unit, the run and the tick.
    let mut card = None;
    if let (Some(i), Some(pointer)) = (hovered, resp.hover_pos()) {
        let r = &lens.regions[i];
        let value = match (r.value, &r.why) {
            (Some(v), _) => value_text(v, &lens.unit, r.beyond),
            (None, Some(why)) => format!("no value: {why}"),
            (None, None) => "no value".to_string(),
        };
        let rank = r
            .rank
            .map(|k| format!(", {} of {}", ordinal(k), lens.ranked.len()))
            .unwrap_or_default();
        let [run, ids] = run_lines(lens);
        let lines = vec![
            format!("{} ({}, {})", r.name, r.key, r.country),
            format!("{}: {value}{rank}", lens.name),
            when(lens),
            run,
            ids,
        ];
        hover_card(&p, rect, pointer, &lines, &ink);
        card = Some(lines);
    }
    state.frame.lens = Some(lens.key.clone());
    state.frame.tick = lens.tick;
    state.frame.regions = drawn;
    state.frame.borders = stroked;
    state.frame.hover = card;
    state.frame.vertices = vertices;
}

fn ordinal(k: usize) -> String {
    let suffix = match (k % 10, k % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{k}{suffix}")
}

/// A rounded panel with lines of text, placed at `at` and kept inside `rect`.
fn panel(
    p: &egui::Painter,
    rect: Rect,
    at: Pos2,
    lines: &[(String, FontId, Color32)],
    ink: &Ink,
) -> Rect {
    let galleys: Vec<_> = lines
        .iter()
        .map(|(t, f, c)| p.layout_no_wrap(t.clone(), f.clone(), *c))
        .collect();
    let w = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max) + 16.0;
    let h = galleys.iter().map(|g| g.size().y + 2.0).sum::<f32>() + 12.0;
    let mut min = at;
    min.x = min.x.min(rect.max.x - w - 4.0).max(rect.min.x + 4.0);
    min.y = min.y.min(rect.max.y - h - 4.0).max(rect.min.y + 4.0);
    let r = Rect::from_min_size(min, vec2(w, h));
    p.rect(
        r,
        6.0,
        ink.panel,
        Stroke::new(1.0, ink.county),
        StrokeKind::Inside,
    );
    let mut y = r.min.y + 6.0;
    for g in galleys {
        let hgt = g.size().y;
        p.galley(pos2(r.min.x + 8.0, y), g, ink.text);
        y += hgt + 2.0;
    }
    r
}

fn overlay_title(p: &egui::Painter, rect: Rect, lens: &LensVm, ink: &Ink) -> Rect {
    let lines = vec![
        (lens.name.clone(), FontId::proportional(16.0), ink.text),
        (
            format!("{} · {}", lens.unit, when(lens)),
            FontId::proportional(12.0),
            ink.weak,
        ),
    ];
    panel(p, rect, rect.min + vec2(10.0, 10.0), &lines, ink)
}

fn hover_card(p: &egui::Painter, rect: Rect, pointer: Pos2, lines: &[String], ink: &Ink) {
    let styled: Vec<(String, FontId, Color32)> = lines
        .iter()
        .enumerate()
        .map(|(k, t)| {
            let (size, c) = match k {
                0 => (14.0, ink.text),
                1 => (13.0, ink.text),
                _ => (11.0, ink.weak),
            };
            (t.clone(), FontId::proportional(size), c)
        })
        .collect();
    panel(p, rect, pointer + vec2(16.0, 16.0), &styled, ink);
}

/// The legend: the scale as a bar with its marks, the unit, the reference and the domain.
fn legend(p: &egui::Painter, rect: Rect, lens: &LensVm, ink: &Ink) {
    let bar_w = (rect.width() * 0.42).clamp(180.0, 380.0);
    let box_h = 74.0;
    let r = Rect::from_min_size(
        pos2(rect.min.x + 10.0, rect.max.y - box_h - 10.0),
        vec2(bar_w + 24.0, box_h),
    );
    p.rect(
        r,
        6.0,
        ink.panel,
        Stroke::new(1.0, ink.county),
        StrokeKind::Inside,
    );
    let bar = Rect::from_min_size(r.min + vec2(12.0, 22.0), vec2(bar_w, 12.0));
    let steps = 64;
    for k in 0..steps {
        let a = k as f32 / steps as f32;
        let b = (k + 1) as f32 / steps as f32;
        let seg = Rect::from_min_max(
            pos2(bar.min.x + a * bar.width(), bar.min.y),
            pos2(bar.min.x + b * bar.width() + 0.5, bar.max.y),
        );
        p.rect_filled(seg, 0.0, colour(&lens.scale, f64::from((a + b) / 2.0)));
    }
    p.rect_stroke(bar, 0.0, Stroke::new(1.0, ink.county), StrokeKind::Outside);
    let font = FontId::proportional(11.0);
    let mut last_x = f32::NEG_INFINITY;
    for t in &lens.legend {
        let x = bar.min.x + t.at as f32 * bar.width();
        p.line_segment(
            [pos2(x, bar.max.y), pos2(x, bar.max.y + 4.0)],
            Stroke::new(1.0, ink.text),
        );
        if x - last_x > 34.0 {
            p.text(
                pos2(x, bar.max.y + 5.0),
                Align2::CENTER_TOP,
                fmt(t.value),
                font.clone(),
                ink.text,
            );
            last_x = x;
        }
    }
    if let Some(rf) = &lens.reference {
        if let Some(at) = rf.at {
            let x = bar.min.x + at as f32 * bar.width();
            p.line_segment(
                [pos2(x, bar.min.y - 4.0), pos2(x, bar.max.y)],
                Stroke::new(2.0, ink.text),
            );
        }
    }
    let head = match (&lens.reference, lens.counts.below + lens.counts.above) {
        (Some(rf), 0) => format!("{} · ▏{}", lens.unit, rf.text),
        (Some(rf), n) => format!("{} · ▏{} · {n} beyond the scale", lens.unit, rf.text),
        (None, 0) => format!("{} · {}", lens.unit, lens.scale),
        (None, n) => format!("{} · {} · {n} beyond the scale", lens.unit, lens.scale),
    };
    p.text(
        r.min + vec2(12.0, 5.0),
        Align2::LEFT_TOP,
        head,
        font,
        ink.text,
    );
}

/// A scale bar of a round number of kilometres, about 120 points long.
fn scale_bar(p: &egui::Painter, rect: Rect, view: &View, ink: &Ink) {
    let per_km = view.scale * 1000.0;
    let mut km = 1.0;
    for k in [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0] {
        if k * per_km <= 140.0 {
            km = k;
        }
    }
    let len = (km * per_km) as f32;
    let a = pos2(rect.max.x - len - 20.0, rect.max.y - 22.0);
    let b = a + vec2(len, 0.0);
    p.line_segment([a, b], Stroke::new(2.0, ink.text));
    for q in [a, b] {
        p.line_segment(
            [q - vec2(0.0, 4.0), q + vec2(0.0, 4.0)],
            Stroke::new(2.0, ink.text),
        );
    }
    p.text(
        a + vec2(len / 2.0, -6.0),
        Align2::CENTER_BOTTOM,
        format!("{km} km"),
        FontId::proportional(11.0),
        ink.text,
    );
}

fn sidebar(
    ui: &mut egui::Ui,
    vm: &MapVm,
    lens: &LensVm,
    state: &mut MapState,
    out: &mut Vec<Intent>,
) {
    // The selector: the lens shown, the others by group with their hotkeys.
    let current = vm
        .lenses
        .iter()
        .position(|l| l.key == lens.key)
        .unwrap_or(0);
    ui.horizontal(|ui| {
        if ui
            .small_button("◀")
            .on_hover_text("previous lens ([)")
            .clicked()
        {
            let n = vm.lenses.len().max(1);
            state.lens = vm.lenses[(current + n - 1) % n].key.clone();
        }
        let label = format!(
            "{}{}",
            hotkey(current)
                .map(|h| format!("{h}  "))
                .unwrap_or_default(),
            lens.name
        );
        egui::ComboBox::from_id_salt("map-lens")
            .selected_text(label)
            .width(ui.available_width() - 34.0)
            .height(520.0)
            .show_ui(ui, |ui| {
                let mut group = "";
                for (k, l) in vm.lenses.iter().enumerate() {
                    if l.group != group {
                        group = &l.group;
                        ui.weak(group);
                    }
                    let text = format!(
                        "{}{}",
                        hotkey(k)
                            .map(|h| format!("{h}  "))
                            .unwrap_or_else(|| "    ".to_string()),
                        l.name
                    );
                    let r = ui.add_enabled(
                        l.unavailable.is_none(),
                        egui::Button::selectable(l.key == state.lens, text),
                    );
                    let r = match &l.unavailable {
                        Some(why) => r.on_disabled_hover_text(why),
                        None => r.on_hover_text(format!("{} ({})", l.key, l.unit)),
                    };
                    if r.clicked() {
                        state.lens = l.key.clone();
                    }
                }
            });
        if ui
            .small_button("▶")
            .on_hover_text("next lens (])")
            .clicked()
        {
            let n = vm.lenses.len().max(1);
            state.lens = vm.lenses[(current + 1) % n].key.clone();
        }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new(&lens.name).strong().size(15.0));
    ui.weak(format!("{} · {}", lens.measure, lens.unit));
    ui.weak(format!(
        "{} scale, fixed for the run: {} to {}{}",
        lens.scale,
        fmt(lens.domain.0),
        fmt(lens.domain.1),
        lens.reference
            .as_ref()
            .map(|r| format!("; reference {}", r.text))
            .unwrap_or_default()
    ));
    if let Some(why) = &lens.unavailable {
        ui.colored_label(ui.visuals().warn_fg_color, why);
    }
    ui.label(when(lens)).on_hover_text(run_line(lens));
    let [run, ids] = run_lines(lens);
    ui.small(run);
    ui.small(ids);
    if lens.counts.in_tape < lens.counts.regions {
        ui.small(format!(
            "{} of the atlas's {} regions are this tape's nodes; the rest are hatched",
            lens.counts.in_tape, lens.counts.regions
        ));
    }
    ui.separator();
    if let Some(c) = &vm.county {
        county_card(ui, c, lens, state, out);
        ui.separator();
    }
    ranked(ui, lens, state, out);
}

fn county_card(
    ui: &mut egui::Ui,
    c: &crate::vm::map::CountyVm,
    lens: &LensVm,
    state: &mut MapState,
    out: &mut Vec<Intent>,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&c.name).strong().size(14.0));
        ui.weak(format!("{} · {}", c.key, c.country));
        if ui
            .small_button("✕")
            .on_hover_text("clear the selection")
            .clicked()
        {
            out.push(Intent::Select(None));
        }
    });
    let here = lens.regions.iter().find(|r| r.key == c.key);
    if let Some(r) = here {
        match (r.value, &r.why) {
            (Some(v), _) => ui.label(format!(
                "{}: {}{}",
                lens.name,
                value_text(v, &lens.unit, r.beyond),
                r.rank
                    .map(|k| format!(", {} of {}", ordinal(k), lens.ranked.len()))
                    .unwrap_or_default()
            )),
            (None, Some(why)) => ui.weak(format!("{}: no value: {why}", lens.name)),
            (None, None) => ui.weak(format!("{}: no value", lens.name)),
        };
    }
    egui::CollapsingHeader::new("Every lens")
        .id_salt("map-county-lenses")
        .show(ui, |ui| {
            egui::Grid::new("map-county-lenses-grid")
                .striped(true)
                .show(ui, |ui| {
                    for l in &c.lenses {
                        if ui.link(&l.name).on_hover_text(&l.key).clicked() {
                            state.lens = l.key.clone();
                        }
                        match l.value {
                            Some(v) => ui.label(format!("{} {}", fmt(v), l.unit)),
                            None => ui.weak(l.why.as_deref().unwrap_or("–")),
                        };
                        ui.end_row();
                    }
                });
        });
    egui::CollapsingHeader::new("Recorded inputs")
        .id_salt("map-county-inputs")
        .show(ui, |ui| {
            egui::Grid::new("map-county-inputs-grid")
                .striped(true)
                .show(ui, |ui| {
                    for i in &c.inputs {
                        ui.label(&i.label);
                        match i.value {
                            Some(v) => ui.label(format!("{} {}", fmt(v), i.unit)),
                            None => ui.weak(format!("– {}", i.unit)),
                        };
                        plot(ui, &i.series, out);
                        ui.end_row();
                    }
                });
        });
}

fn plot(ui: &mut egui::Ui, s: &SeriesKey, out: &mut Vec<Intent>) {
    let r = ui.small_button("plot").on_hover_text(s.to_string());
    r.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("plot {s}"))
    });
    if r.clicked() {
        out.push(Intent::Plot(s.clone()));
    }
}

fn ranked(ui: &mut egui::Ui, lens: &LensVm, state: &mut MapState, out: &mut Vec<Intent>) {
    ui.label(format!(
        "Ranked: {} counties with a value, largest first ({})",
        lens.ranked.len(),
        lens.unit
    ));
    let line = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let mut rows = Vec::new();
    let mut hot = None;
    let height = ui.available_height();
    TableBuilder::new(ui)
        .id_salt("map-ranked")
        .striped(true)
        .max_scroll_height(height)
        .column(Column::exact(28.0))
        .column(Column::exact(16.0))
        .column(Column::remainder().at_least(120.0))
        .column(Column::auto().at_least(70.0))
        .header(line, |mut h| {
            for t in ["#", "", "county", "value"] {
                h.col(|ui| {
                    ui.strong(t);
                });
            }
        })
        .body(|body| {
            body.rows(line, lens.ranked.len(), |mut row| {
                let i = lens.ranked[row.index()];
                let r = &lens.regions[i];
                let Some(v) = r.value else { return };
                let c = r.at.map_or(Color32::GRAY, |at| colour(&lens.scale, at));
                let rank = r.rank.unwrap_or(0);
                let text = fmt(v);
                row.col(|ui| {
                    ui.weak(rank.to_string());
                });
                row.col(|ui| {
                    let (sw, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                    ui.painter().rect_filled(sw, 2.0, c);
                });
                let (_, name) = row.col(|ui| {
                    let sel = ui.selectable_label(false, &r.name).on_hover_text(&r.key);
                    if sel.clicked() {
                        if let Ok(k) = Key::new(r.key.clone()) {
                            out.push(Intent::Select(Some(Entity::Node(k))));
                        }
                    }
                });
                row.col(|ui| {
                    let mark = match r.beyond {
                        Beyond::Within => "",
                        Beyond::Below => " ↓",
                        Beyond::Above => " ↑",
                    };
                    ui.label(format!("{text}{mark}"))
                        .on_hover_text(format!("{v} {}", lens.unit));
                });
                if name.hovered() {
                    hot = Some(r.key.clone());
                }
                rows.push(DrawnRow {
                    key: r.key.clone(),
                    rank,
                    value: v,
                    text,
                    colour: c,
                });
            });
        });
    state.hot = hot;
    state.frame.rows = rows;
}

/// A frame's map shapes at `rect`, as the canvas makes them: the fill mesh of every region in
/// `colours` and every border's thinned strokes. For the headless timing (§6, "Measured").
pub fn frame_shapes(geo: &Geo, colours: &[Color32], rect: Rect) -> (Mesh, Vec<Shape>, Vec<Shape>) {
    let view = View::fit(geo.bounds, rect);
    let mesh = fill_mesh(geo, colours, &view, rect);
    let (a, b) = border_shapes(geo, &view, rect, true);
    (mesh, a, b)
}

/// Each region's colour under a lens, in the atlas's order.
pub fn colours(lens: &LensVm, dark: bool) -> Vec<Color32> {
    let ink = ink(dark);
    lens.regions
        .iter()
        .map(|r| fill_of(r, &lens.scale, &ink).0)
        .collect()
}

/// The fills and regions of an atlas, by key, for the tests.
pub fn index(geo: &Geo) -> BTreeMap<String, usize> {
    geo.atlas
        .regions
        .iter()
        .enumerate()
        .map(|(i, r)| (r.key.clone(), i))
        .collect()
}
