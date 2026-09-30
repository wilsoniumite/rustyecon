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
//!   grey; a region with no value is plain grey, and its hover card says why. The atlas's
//!   credit (data/atlas/ATTRIBUTION, ODbL) sits in the lower right corner, always, with the
//!   whole attribution on its hover.
//! - **Fitted clear.** The fitted view leaves the legend's box and the credit's panel clear of
//!   every region: where a region would fall under them, the map fits the canvas above their
//!   tops (O39, D2.1). Until someone pans or zooms, a canvas of another size is fitted afresh
//!   (D2.5). The ranked table's county names are clipped before its values are.
//! - **The sidebar** (D2.5). Below the selector and the lens's heading, the selected county's
//!   card and what the lens is scroll together above the ranked table, which keeps at least 40%
//!   of the height; every line wraps to the sidebar, and a ranked value is one line.
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
use rustyecon_worldgen::lens as wl;
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
        View::fit_within(bounds, rect, rect)
    }

    /// The view on the canvas `rect` that fits `bounds` into `area`, a part of the canvas, with
    /// a margin: the centre of `bounds` lands on the centre of `area`.
    pub fn fit_within(bounds: [f64; 4], rect: Rect, area: Rect) -> View {
        let w = (bounds[2] - bounds[0]).max(1.0);
        let h = (bounds[3] - bounds[1]).max(1.0);
        let scale = (f64::from(area.width()) / w).min(f64::from(area.height()) / h) * 0.94;
        let scale = scale.max(1e-9);
        let shift = area.center() - rect.center();
        View {
            centre: [
                (bounds[0] + bounds[2]) / 2.0 - f64::from(shift.x) / scale,
                (bounds[1] + bounds[3]) / 2.0 + f64::from(shift.y) / scale,
            ],
            scale,
        }
    }

    /// The fitted view of `geo` on the canvas `rect` that leaves the legend's box and the
    /// credit's panel (`clear`) clear of every region (O39, D2.1): the whole canvas when no
    /// region's box falls under them, else the canvas above the highest of their tops. On a
    /// wide canvas the map sits between the two; on a narrower one Cornwall and Devon, at the
    /// map's south-west corner, would fall under the legend.
    pub fn fit_clear(geo: &Geo, rect: Rect, clear: &[Rect]) -> View {
        let whole = View::fit(geo.bounds, rect);
        if !geo.fills.iter().any(|f| {
            let r = whole.box_on_screen(rect, f.bbox);
            clear.iter().any(|c| c.intersects(r))
        }) {
            return whole;
        }
        let top = clear.iter().map(|c| c.min.y).fold(rect.max.y, f32::min) - 6.0;
        let area = Rect::from_min_max(rect.min, pos2(rect.max.x, top.max(rect.min.y + 1.0)));
        View::fit_within(geo.bounds, rect, area)
    }

    /// A grid box (west, south, east, north) on screen.
    pub fn box_on_screen(&self, rect: Rect, b: [f64; 4]) -> Rect {
        Rect::from_two_pos(
            self.to_screen(rect, [b[0], b[1]]),
            self.to_screen(rect, [b[2], b[3]]),
        )
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
    /// The box its painted triangles cover on screen, read from the frame's mesh: every vertex
    /// of its span. `None` for a region with no vertex.
    pub rect: Option<Rect>,
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
    /// The legend as drawn.
    pub legend: LegendFrame,
    /// The atlas's credit, as painted on the map (data/atlas/ATTRIBUTION, ODbL).
    pub credit: Vec<String>,
    /// Where the credit was painted.
    pub credit_rect: Option<Rect>,
    /// Where the scale bar was painted, its label included (D2.5): clear of the legend and the
    /// credit.
    pub scale_bar: Option<Rect>,
    /// The canvas and the sidebar beside it (D2.5).
    pub canvas: Option<Rect>,
    /// The sidebar: the selector, the lens, the selected county and the ranked table.
    pub sidebar: Option<Rect>,
}

/// The legend as the last frame drew it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LegendFrame {
    /// The bar's segments: where each starts and ends on the scale, from 0 to 1, and its colour.
    pub segments: Vec<(f64, f64, Color32)>,
    /// Each mark: its value, and where along the bar it was drawn, from 0 to 1.
    pub marks: Vec<(f64, f64)>,
    /// Where along the bar the reference's mark was drawn, when it was.
    pub reference: Option<f64>,
    /// The header's text.
    pub head: String,
    /// Its box, once drawn.
    pub rect: Option<Rect>,
    /// Where along the bar each further named mark was drawn (the maker's reservation ψ on
    /// the horse's price), from 0 to 1.
    pub named: Vec<f64>,
    /// The marks whose values were written under the bar, by value (D2.5): the domain's ends
    /// always, then the reference, then on a log scale each power of ten, then the rest where
    /// they fit.
    pub labelled: Vec<f64>,
}

/// The map's state between frames.
pub struct MapState {
    geo: Option<Result<Geo, String>>,
    lenses: Option<(wl::Kind, Result<Vec<Lens>, String>)>,
    /// The lens shown, by key.
    pub lens: String,
    view: Option<View>,
    /// The canvas the view was last fitted to, while nobody has panned or zoomed since: a
    /// canvas of another size is then fitted afresh (D2.5). `None` once the view was moved.
    fit: Option<Rect>,
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
    /// Each region's painted box, from its span's vertices, in the atlas's order.
    rects: Vec<Option<Rect>>,
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
            fit: None,
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

    /// v1's lenses, read on first use.
    pub fn lenses(&mut self) -> Result<&[Lens], String> {
        self.lenses_for(wl::V1_TAPE)
    }

    /// The lenses of the tape named `tape` (`lens::for_tape`: the second pass's for its tape,
    /// v1's for any other), read when the table changes.
    pub fn lenses_for(&mut self, tape: &str) -> Result<&[Lens], String> {
        let kind = wl::Kind::of_tape(tape);
        if self.lenses.as_ref().map(|(k, _)| *k) != Some(kind) {
            self.lenses = Some((kind, wl::for_tape(tape).map_err(|e| e.to_string())));
        }
        match &self.lenses {
            Some((_, Ok(l))) => Ok(l),
            Some((_, Err(e))) => Err(e.clone()),
            None => Err("no lens table".to_string()),
        }
    }

    /// Read the atlas and v1's lenses, once.
    pub fn ready(&mut self) -> Result<(), String> {
        self.ready_for(wl::V1_TAPE)
    }

    /// Read the atlas and the lenses of the tape named `tape`.
    pub fn ready_for(&mut self, tape: &str) -> Result<(), String> {
        self.geo()?;
        self.lenses_for(tape)?;
        Ok(())
    }

    /// The atlas and the lenses, once read.
    pub fn parts(&self) -> Option<(&Geo, &[Lens])> {
        match (&self.geo, &self.lenses) {
            (Some(Ok(g)), Some((_, Ok(l)))) => Some((g, l)),
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

    /// Whether the view is the fitted one, untouched since: the map then follows its canvas.
    pub fn is_fitted(&self) -> bool {
        self.view.is_none() || self.fit.is_some()
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
    state.frame.canvas = Some(canvas_rect);
    state.frame.sidebar = Some(side);
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
    let p = ui.painter_at(rect);
    // The legend's box and the credit's panel, which the fitted map leaves clear (O39).
    let legend_at = legend_box(rect, legend_row(&p));
    let clear = [legend_at, credit_layout(&p, rect, legend_at, &ink).1];
    let fitted = || View::fit_clear(geo, rect, &clear);
    // The fitted view follows its canvas: until someone pans or zooms, a canvas of another size
    // (a resized window, a moved divider) is fitted afresh, so Cornwall never lies under the
    // legend after a resize (D2.5; the map review's fifth minor).
    let refit = match (state.view, state.fit) {
        (None, _) => true,
        (Some(_), Some(at)) => at != rect,
        (Some(_), None) => false,
    };
    if refit {
        state.view = Some(fitted());
        state.fit = Some(rect);
    }
    let mut view = state.view.unwrap_or_else(fitted);
    let mut moved = false;
    // Pan, zoom about the pointer, fit on a double click.
    if resp.dragged() {
        let d = resp.drag_delta();
        view.centre[0] -= f64::from(d.x) / view.scale;
        view.centre[1] += f64::from(d.y) / view.scale;
        moved |= d != Vec2::ZERO;
    }
    if let Some(p) = resp.hover_pos() {
        let (scroll, zoom) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
        let factor = f64::from(zoom) * (1.0 + f64::from(scroll) * 0.0015).clamp(0.5, 2.0);
        if (factor - 1.0).abs() > 1e-6 {
            let fit = fitted().scale;
            let before = view.to_grid(rect, p);
            view.scale = (view.scale * factor).clamp(fit * 0.5, fit * 60.0);
            let after = view.to_grid(rect, p);
            view.centre[0] += before[0] - after[0];
            view.centre[1] += before[1] - after[1];
            moved = true;
        }
    }
    if moved {
        state.fit = None;
    }
    if resp.double_clicked() {
        view = fitted();
        state.fit = Some(rect);
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
            let mut rects: Vec<Option<Rect>> = vec![None; geo.fills.len()];
            for &(i, first, n) in &spans {
                for v in &mesh.vertices[first..first + n] {
                    rects[i]
                        .get_or_insert(Rect::from_min_max(v.pos, v.pos))
                        .extend_with(v.pos);
                }
            }
            Painted {
                key,
                colours: colours.clone(),
                mesh: std::sync::Arc::new(mesh),
                spans,
                rects,
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
    let rects = g.rects.clone();
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
            rect: rects[i],
        });
    }
    overlay_title(&p, rect, lens, &ink);
    let (legend, legend_rect) = legend(&p, rect, lens, &ink);
    let (credit, credit_rect) = credit(&p, rect, legend_rect, &ink);
    // The scale bar sits above the credit in the corner, or in the corner itself when a narrow
    // canvas puts the credit above the legend.
    let bar_y = if credit_rect.max.y >= rect.max.y - 5.0 {
        credit_rect.min.y - 6.0
    } else {
        rect.max.y - 18.0
    };
    let scale = scale_bar(&p, rect, bar_y, &[legend_rect, credit_rect], &view, &ink);
    state.frame.scale_bar = Some(scale);
    // The whole attribution and licence, on the credit's hover (data/atlas/ATTRIBUTION).
    ui.interact(credit_rect, ui.id().with("map-credit"), Sense::hover())
        .on_hover_text(rustyecon_worldgen::atlas::ATTRIBUTION);
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
    state.frame.legend = legend;
    state.frame.credit = credit;
    state.frame.credit_rect = Some(credit_rect);
}

/// The atlas's credit (data/atlas/ATTRIBUTION; ODbL), in the canvas's lower right corner on a
/// pale panel, or just above the legend (`avoid`) when the canvas is too narrow for both side
/// by side: the lines painted and the panel's rect. Each line wraps to the canvas, so a narrow
/// one clips none of it (G1, after D.4's re-check, O26), and the panel stays inside the canvas.
fn credit(p: &egui::Painter, rect: Rect, avoid: Rect, ink: &Ink) -> (Vec<String>, Rect) {
    let (galleys, r) = credit_layout(p, rect, avoid, ink);
    p.rect_filled(r, 3.0, ink.panel);
    let mut y = r.min.y + 3.0;
    let mut lines = Vec::with_capacity(galleys.len());
    for g in galleys {
        let hgt = g.size().y;
        lines.push(g.text().to_string());
        p.galley(pos2(r.min.x + 5.0, y), g, ink.weak);
        y += hgt;
    }
    (lines, r)
}

/// The credit's lines laid out, and the panel [`credit`] paints them on, without painting.
fn credit_layout(
    p: &egui::Painter,
    rect: Rect,
    avoid: Rect,
    ink: &Ink,
) -> (Vec<std::sync::Arc<egui::Galley>>, Rect) {
    let font = FontId::proportional(10.0);
    let wrap = (rect.width() - 18.0).max(40.0);
    let galleys: Vec<_> = rustyecon_worldgen::atlas::CREDIT
        .iter()
        .map(|t| p.layout((*t).to_string(), font.clone(), ink.weak, wrap))
        .collect();
    let w = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max) + 10.0;
    let h = galleys.iter().map(|g| g.size().y).sum::<f32>() + 6.0;
    let mut r = Rect::from_min_size(pos2(rect.max.x - w - 4.0, rect.max.y - h - 4.0), vec2(w, h));
    if r.intersects(avoid) {
        let x = avoid.min.x.min(rect.max.x - w - 4.0).max(rect.min.x + 4.0);
        r = Rect::from_min_size(pos2(x, avoid.min.y - h - 4.0), vec2(w, h));
    }
    (galleys, r)
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

/// A rounded panel with lines of text, placed at `at` and kept inside `rect`. Each line wraps to
/// the canvas, so a narrow canvas cuts none of it (D2.5: the title's unit line was cut at a
/// 1,024 × 768 window).
fn panel(
    p: &egui::Painter,
    rect: Rect,
    at: Pos2,
    lines: &[(String, FontId, Color32)],
    ink: &Ink,
) -> Rect {
    let wrap = (rect.width() - 28.0).max(60.0);
    let galleys: Vec<_> = lines
        .iter()
        .map(|(t, f, c)| p.layout(t.clone(), f.clone(), *c, wrap))
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

/// The legend's bar's width on a canvas: 42% of the canvas's, from 180 to 380 points.
fn legend_bar(rect: Rect) -> f32 {
    (rect.width() * 0.42).clamp(180.0, 380.0)
}

/// The legend's font.
fn legend_font() -> FontId {
    FontId::proportional(11.0)
}

/// The height of one row of the legend's text, as the painter lays it out.
fn legend_row(p: &egui::Painter) -> f32 {
    p.layout_no_wrap("Hg".to_string(), legend_font(), Color32::WHITE)
        .size()
        .y
}

/// Where the legend's bar starts below the box's top: the head's two rows, then a row for the
/// named marks (ψ) above the bar (D2.5: the head no longer runs past the box, and ψ is no longer
/// painted over it).
fn legend_bar_top(row: f32) -> f32 {
    5.0 + 2.0 * row + row.max(12.0) + 1.0
}

/// The legend's box on a canvas, in its lower left corner: the same height for every lens, so
/// the fitted view does not move when the lens does.
fn legend_box(rect: Rect, row: f32) -> Rect {
    let box_h = legend_bar_top(row) + 12.0 + 5.0 + row + 6.0;
    Rect::from_min_size(
        pos2(rect.min.x + 10.0, rect.max.y - box_h - 10.0),
        vec2(legend_bar(rect) + 24.0, box_h),
    )
}

/// A value on the legend is a power of ten, read from its label (`1`, `10`, `0.01`).
fn is_decade(label: &str) -> bool {
    let digits: String = label
        .chars()
        .filter(|c| c.is_ascii_digit() && *c != '0')
        .collect();
    digits == "1" && !label.contains('e') && !label.starts_with('-')
}

/// The legend: the scale as a bar with its marks, the unit, the reference and the domain; and
/// what it drew, with its box.
fn legend(p: &egui::Painter, rect: Rect, lens: &LensVm, ink: &Ink) -> (LegendFrame, Rect) {
    let mut drawn = LegendFrame::default();
    let bar_w = legend_bar(rect);
    let row = legend_row(p);
    let r = legend_box(rect, row);
    p.rect(
        r,
        6.0,
        ink.panel,
        Stroke::new(1.0, ink.county),
        StrokeKind::Inside,
    );
    let bar = Rect::from_min_size(r.min + vec2(12.0, legend_bar_top(row)), vec2(bar_w, 12.0));
    // Where a point of the bar sits on the scale, from 0 to 1, as drawn.
    let along = |x: f32| f64::from((x - bar.min.x) / bar.width());
    let steps = 64;
    for k in 0..steps {
        let a = k as f32 / steps as f32;
        let b = (k + 1) as f32 / steps as f32;
        let seg = Rect::from_min_max(
            pos2(bar.min.x + a * bar.width(), bar.min.y),
            pos2(bar.min.x + b * bar.width() + 0.5, bar.max.y),
        );
        let c = colour(&lens.scale, f64::from((a + b) / 2.0));
        p.rect_filled(seg, 0.0, c);
        drawn.segments.push((f64::from(a), f64::from(b), c));
    }
    p.rect_stroke(bar, 0.0, Stroke::new(1.0, ink.county), StrokeKind::Outside);
    let font = legend_font();
    let reference = lens.reference.as_ref().filter(|rf| rf.at.is_some());
    // Every mark is ticked; its value is written where it fits, by rank (D2.5: the gap lens's 10
    // and its domain's top went unwritten): the domain's ends always, the reference, each power
    // of ten on a log scale, then the rest, none within 4 points of another. The ends are
    // written inward from the bar's ends, so they stay inside the box.
    let last = lens.legend.len().saturating_sub(1);
    let mut ranked: Vec<(u8, usize)> = Vec::with_capacity(lens.legend.len());
    for (k, t) in lens.legend.iter().enumerate() {
        let x = bar.min.x + t.at as f32 * bar.width();
        p.line_segment(
            [pos2(x, bar.max.y), pos2(x, bar.max.y + 4.0)],
            Stroke::new(1.0, ink.text),
        );
        drawn.marks.push((t.value, along(x)));
        let label = fmt(t.value);
        let rank = if k == 0 || k == last {
            0
        } else if reference.is_some_and(|rf| rf.value == t.value) {
            1
        } else if lens.scale.contains("log") && is_decade(&label) {
            2
        } else {
            3
        };
        ranked.push((rank, k));
    }
    ranked.sort();
    let mut taken: Vec<(f32, f32)> = Vec::new();
    let mut written: Vec<usize> = Vec::new();
    for (_, k) in ranked {
        let t = &lens.legend[k];
        let x = bar.min.x + t.at as f32 * bar.width();
        let g = p.layout_no_wrap(fmt(t.value), font.clone(), ink.text);
        let w = g.size().x;
        let lo = if k == 0 {
            x - 1.0
        } else if k == last {
            x + 1.0 - w
        } else {
            x - w / 2.0
        };
        let span = (lo, lo + w);
        if taken
            .iter()
            .any(|&(a, b)| span.0 < b + 4.0 && a < span.1 + 4.0)
        {
            continue;
        }
        taken.push(span);
        written.push(k);
        p.galley(pos2(lo, bar.max.y + 5.0), g, ink.text);
    }
    written.sort_unstable();
    drawn.labelled = written.iter().map(|&k| lens.legend[k].value).collect();
    // The reference is marked, and named in the header, only where it lies on the scale.
    if let Some(at) = reference.and_then(|rf| rf.at) {
        let x = bar.min.x + at as f32 * bar.width();
        p.line_segment(
            [pos2(x, bar.min.y - 4.0), pos2(x, bar.max.y)],
            Stroke::new(2.0, ink.text),
        );
        drawn.reference = Some(along(x));
    }
    // The header names the unit and the reference, whose mark is the bar's heavy line, `|`,
    // a glyph the bundled font has (D2.5: `▏` was painted as a box). It wraps to the bar's
    // width, two rows at most, so it never runs past the box.
    let head = match (reference, lens.counts.below + lens.counts.above) {
        (Some(rf), 0) => format!("{} · | {}", lens.unit, rf.text),
        (Some(rf), n) => format!("{} · | {} · {n} beyond the scale", lens.unit, rf.text),
        (None, 0) => format!("{} · {}", lens.unit, lens.scale),
        (None, n) => format!("{} · {} · {n} beyond the scale", lens.unit, lens.scale),
    };
    let mut job = egui::text::LayoutJob::simple(head.clone(), font, ink.text, bar_w);
    job.wrap.max_rows = 2;
    p.galley(r.min + vec2(12.0, 5.0), p.layout_job(job), ink.text);
    // Further named marks, in the row between the header and the bar: the maker's
    // reservation ψ on the horse's price.
    for m in &lens.marks {
        if let Some(at) = m.at {
            let x = bar.min.x + at as f32 * bar.width();
            p.line_segment(
                [pos2(x, bar.min.y - 4.0), pos2(x, bar.max.y)],
                Stroke::new(1.0, ink.text),
            );
            p.text(
                pos2(x + 2.0, bar.min.y - 2.0),
                Align2::LEFT_BOTTOM,
                "ψ",
                FontId::proportional(10.0),
                ink.text,
            );
            drawn.named.push(along(x));
        }
    }
    drawn.head = head;
    drawn.rect = Some(r);
    (drawn, r)
}

/// A scale bar of a round number of kilometres, about 120 points long, its line at `y`, or
/// above any of `avoid` it would cross (D2.5: on a narrow canvas it lay over the legend's
/// labels). Returns what it covers, its label included.
fn scale_bar(
    p: &egui::Painter,
    rect: Rect,
    y: f32,
    avoid: &[Rect],
    view: &View,
    ink: &Ink,
) -> Rect {
    let per_km = view.scale * 1000.0;
    let mut km = 1.0;
    for k in [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0] {
        if k * per_km <= 140.0 {
            km = k;
        }
    }
    let len = (km * per_km) as f32;
    let font = FontId::proportional(11.0);
    let label = p.layout_no_wrap(format!("{km} km"), font, ink.text);
    let size = label.size();
    let x0 = rect.max.x - len - 20.0;
    let covers = |y: f32| {
        let mid = x0 + len / 2.0;
        Rect::from_min_max(
            pos2((x0 - 1.0).min(mid - size.x / 2.0), y - 4.0 - 6.0 - size.y),
            pos2((x0 + len + 1.0).max(mid + size.x / 2.0), y),
        )
    };
    let mut y = y;
    for _ in 0..avoid.len() + 1 {
        match avoid
            .iter()
            .filter(|a| a.intersects(covers(y)))
            .map(|a| a.min.y)
            .reduce(f32::min)
        {
            Some(top) => y = top - 4.0,
            None => break,
        }
    }
    let a = pos2(x0, y - 4.0);
    let b = a + vec2(len, 0.0);
    p.line_segment([a, b], Stroke::new(2.0, ink.text));
    for q in [a, b] {
        p.line_segment(
            [q - vec2(0.0, 4.0), q + vec2(0.0, 4.0)],
            Stroke::new(2.0, ink.text),
        );
    }
    p.galley(
        a + vec2(len / 2.0 - size.x / 2.0, -6.0 - size.y),
        label,
        ink.text,
    );
    covers(y)
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
        // A long lens name is cut short in the button, never widening the sidebar past its
        // pane (O39, D2.1): the button is laid out in the width left for it, less the next
        // button's, and the whole name is in the selector and the heading below.
        let w = (ui.available_width() - 34.0).max(40.0);
        let h = ui.spacing().interact_size.y;
        ui.allocate_ui(vec2(w, h), |ui| {
            egui::ComboBox::from_id_salt("map-lens")
                .selected_text(label)
                .truncate()
                .width(w)
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
    ui.label(when(lens)).on_hover_text(run_line(lens));
    if let Some(why) = &lens.unavailable {
        ui.colored_label(ui.visuals().warn_fg_color, why);
    }
    // Below the heading the sidebar has two parts (D2.5; the map review's first major): the
    // selected county's card and what the lens is, which scroll together and take at most what
    // leaves the ranked table its rows; and the ranked table, in the rest. Nothing in either is
    // laid out wider than the sidebar, so nothing is cut at the pane's edge.
    let avail = ui.available_height();
    let line = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let share = if vm.county.is_some() { 0.4 } else { 0.5 };
    let table = (avail * share).min(line * 11.0);
    let top = (avail - table - 12.0).max(line * 3.0);
    egui::ScrollArea::vertical()
        .id_salt("map-side")
        .max_height(top)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if let Some(c) = &vm.county {
                county_card(ui, c, lens, state, out);
                ui.separator();
            }
            about(ui, lens);
        });
    ui.separator();
    ranked(ui, lens, state, out);
}

/// What the lens is: its measure and unit, its scale and domain, its named marks, its note, the
/// run its values belong to (U3), and the hatched regions.
fn about(ui: &mut egui::Ui, lens: &LensVm) {
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
    for m in &lens.marks {
        ui.weak(&m.text);
    }
    if !lens.note.is_empty() {
        ui.small(&lens.note);
    }
    let [run, ids] = run_lines(lens);
    ui.small(run);
    ui.small(ids);
    if lens.counts.in_tape < lens.counts.regions {
        ui.small(format!(
            "{} of the atlas's {} regions are this tape's nodes; the rest are hatched",
            lens.counts.in_tape, lens.counts.regions
        ));
    }
}

fn county_card(
    ui: &mut egui::Ui,
    c: &crate::vm::map::CountyVm,
    lens: &LensVm,
    state: &mut MapState,
    out: &mut Vec<Intent>,
) {
    // The header wraps rather than running past the pane (O26's card note; D2.5), and its
    // button is `×`, a glyph the bundled font has (`✕` was painted as a box).
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(&c.name).strong().size(14.0));
        ui.weak(format!("{} · {}", c.key, c.country));
        if ui
            .small_button("×")
            .on_hover_text("clear the selection")
            .clicked()
        {
            out.push(Intent::Select(None));
        }
    });
    // Its value under the lens named in the heading just above, which the card leaves out to
    // keep its lines few on a small window (D2.5).
    let here = lens.regions.iter().find(|r| r.key == c.key);
    if let Some(r) = here {
        match (r.value, &r.why) {
            (Some(v), _) => ui.label(format!(
                "{}{}",
                value_text(v, &lens.unit, r.beyond),
                r.rank
                    .map(|k| format!(", {} of {}", ordinal(k), lens.ranked.len()))
                    .unwrap_or_default()
            )),
            (None, Some(why)) => ui.weak(format!("no value: {why}")),
            (None, None) => ui.weak("no value"),
        };
    }
    // What carries its gap to its equilibrium, beside the herd's own readings (WORLD-V2 §8,
    // item 2): a county short of horses shows a positive markup, the rent that buys more.
    if !c.causes.is_empty() {
        ui.weak("Its gap, ln(observed / equilibrium):");
        for cause in &c.causes {
            ui.label(format!("  {} {:+.3}", cause.observable, cause.gap));
        }
        for key in ["horses.vs.oracle", "horses.vs.plan", "hday.markup"] {
            if let Some(l) = c.lenses.iter().find(|l| l.key == key) {
                match l.value {
                    Some(v) => ui.label(format!("  {}: {v:+.3}", l.name)),
                    None => ui.weak(format!("  {}: {}", l.name, l.why.as_deref().unwrap_or("–"))),
                };
            }
        }
    }
    // Each lens and each input on its own line, which wraps to the sidebar: a name and a value
    // with its unit are wider than the pane together (D2.5: in a grid the values ran past it).
    egui::CollapsingHeader::new("Every lens")
        .id_salt("map-county-lenses")
        .show(ui, |ui| {
            for l in &c.lenses {
                ui.horizontal_wrapped(|ui| {
                    if ui.link(&l.name).on_hover_text(&l.key).clicked() {
                        state.lens = l.key.clone();
                    }
                    match l.value {
                        Some(v) => ui.label(format!("{} {}", fmt(v), l.unit)),
                        None => ui.weak(l.why.as_deref().unwrap_or("–")),
                    };
                });
            }
        });
    egui::CollapsingHeader::new("Recorded inputs")
        .id_salt("map-county-inputs")
        .show(ui, |ui| {
            for i in &c.inputs {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&i.label);
                    match i.value {
                        Some(v) => ui.label(format!("{} {}", fmt(v), i.unit)),
                        None => ui.weak(format!("– {}", i.unit)),
                    };
                    plot(ui, &i.series, out);
                });
            }
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
        "Ranked: {} counties, largest first",
        lens.ranked.len()
    ))
    .on_hover_text(format!("the counties with a value, in {}", lens.unit));
    let line = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    let mut rows = Vec::new();
    let mut hot = None;
    let height = ui.available_height();
    TableBuilder::new(ui)
        .id_salt("map-ranked")
        .striped(true)
        .max_scroll_height(height)
        // The county's name takes what is left and is clipped there, so the value column keeps
        // its width on a narrow pane (O39, D2.1): a long name is cut, never a value.
        .column(Column::exact(28.0))
        .column(Column::exact(16.0))
        .column(Column::remainder().at_least(24.0).clip(true))
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
                    // One line: a value too wide for the column widens the column rather than
                    // wrapping (D2.5: `2.22045e-16` was painted over two lines).
                    ui.add(egui::Label::new(format!("{text}{mark}")).extend())
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

#[cfg(test)]
mod tests {
    use super::{credit, ink, legend_box, legend_row};
    use egui::{pos2, vec2, Rect};

    /// The credit's panel on a canvas of `w` × 600 beside the legend, as the canvas places it,
    /// with the legend's box and the credit's width.
    fn placed(ctx: &egui::Context, w: f32) -> (Rect, Rect, Rect) {
        let canvas = Rect::from_min_size(pos2(20.0, 30.0), vec2(w, 600.0));
        let mut got = None;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let avoid = legend_box(canvas, legend_row(ui.painter()));
            got = Some((avoid, credit(ui.painter(), canvas, avoid, &ink(false)).1));
        });
        let (avoid, credit) = got.expect("the credit was placed");
        (canvas, avoid, credit)
    }

    #[test]
    fn the_credit_moved_above_the_legend_stays_on_the_canvas() {
        // G1's verification (A5): no test decided the credit's x once it moves above the legend.
        // On a canvas narrow enough that the credit meets the legend, the credit starts at the
        // legend's left edge unless that would run it past the canvas's right edge, where it
        // starts as far right as fits; never left of the canvas's own edge. Every width from 60
        // to 1,000 points is tried, and at some of them the right edge decides.
        let ctx = egui::Context::default();
        let mut decided = Vec::new();
        for w in 60..=1000 {
            let (canvas, avoid, r) = placed(&ctx, w as f32);
            let moved = r.max.y <= avoid.min.y;
            if !moved {
                continue;
            }
            assert!(!r.intersects(avoid), "at {w}: {r:?} over {avoid:?}");
            assert!(r.min.x >= canvas.min.x + 4.0 - 1e-3, "at {w}: {r:?}");
            assert!(r.min.x <= avoid.min.x + 1e-3, "at {w}: {r:?}");
            if r.width() <= canvas.width() - 8.0 {
                assert!(
                    r.max.x <= canvas.max.x - 4.0 + 1e-3,
                    "at {w}: {r:?} runs past {canvas:?}"
                );
            }
            if r.min.x < avoid.min.x - 1e-3 {
                decided.push(w);
            }
        }
        println!(
            "the right edge decides the credit's x at {} widths",
            decided.len()
        );
        assert!(!decided.is_empty(), "no width tried moves the credit left");
    }
}
