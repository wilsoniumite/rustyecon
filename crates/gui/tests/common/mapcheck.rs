//! The map's checks read from what a frame painted, shared by the map's tests on v1's store and
//! on the second pass's (D2.5; WORLD-V2 §10, decision 338): each region's and each legend
//! segment's colour as painted against its lens, every text with its clip rect and rows, and
//! every glyph in the font that paints it.

use egui::epaint::{ClippedShape, Shape};
use egui::{Color32, FontId, Rect};
use rustyecon_gui::run::{Origin, Store};
use rustyecon_gui::ui::map::{self as ui_map, Geo, LegendFrame, MapFrame};
use rustyecon_gui::vm::map::{self as vm_map, LensVm};
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::tables::Lens;

/// A painted text.
#[derive(Debug, Clone)]
pub struct Painted {
    /// What it says.
    pub text: String,
    /// Where it sits on screen.
    pub rect: Rect,
    /// The rect it is clipped to.
    pub clip: Rect,
    /// Whether it is seen: not transparent, not faded to nothing.
    pub seen: bool,
    /// Its galley's rows.
    pub rows: usize,
    /// Each section's font and text.
    pub sections: Vec<(FontId, String)>,
}

impl Painted {
    /// Whether it lies inside its clip rect from top to bottom: a row a scroll area shows only in
    /// part is cut there by design.
    pub fn whole_down(&self) -> bool {
        self.rect.min.y >= self.clip.min.y - 0.5 && self.rect.max.y <= self.clip.max.y + 0.5
    }

    /// Whether it lies inside `r` from left to right.
    pub fn across(&self, r: Rect) -> bool {
        r.min.x <= self.rect.min.x + 0.5 && self.rect.max.x <= r.max.x + 0.5
    }
}

/// Every text the shapes paint, in paint order.
pub fn texts(shapes: &[ClippedShape]) -> Vec<Painted> {
    fn walk(s: &Shape, clip: Rect, out: &mut Vec<Painted>) {
        match s {
            Shape::Text(t) => {
                let seen = t.opacity_factor > 0.0
                    && t.override_text_color.is_none_or(|c| c.a() > 0)
                    && t.galley.job.sections.iter().all(|s| s.format.color.a() > 0);
                let job = &t.galley.job;
                let sections = job
                    .sections
                    .iter()
                    .map(|s| {
                        (
                            s.format.font_id.clone(),
                            job.text[s.byte_range.start.0..s.byte_range.end.0].to_string(),
                        )
                    })
                    .collect();
                out.push(Painted {
                    text: t.galley.text().to_string(),
                    rect: t.visual_bounding_rect(),
                    clip,
                    seen,
                    rows: t.galley.rows.len(),
                    sections,
                });
            }
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, clip, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in shapes {
        walk(&c.shape, c.clip_rect, &mut out);
    }
    out
}

/// Every painted text with a character its font cannot draw, which egui paints as its
/// replacement box `◻` (D2.5: the legend's `▏` and the card's `✕`). A character is drawn as the
/// box exactly when its laid-out glyph is the box's glyph; `Fonts::has_glyph` cannot say, since
/// it answers no for every character of the face that holds the box, `◀` and `▶` among them.
pub fn missing_glyphs(ctx: &egui::Context, shapes: &[ClippedShape]) -> Vec<String> {
    let mut seen: std::collections::BTreeMap<(String, char), bool> = Default::default();
    let glyph = |font: &FontId, c: char| {
        let g = ctx.fonts_mut(|f| f.layout_no_wrap(c.to_string(), font.clone(), Color32::WHITE));
        g.rows
            .first()
            .and_then(|r| r.row.glyphs.first())
            .map(|g| g.uv_rect)
    };
    let mut out = Vec::new();
    for t in texts(shapes) {
        for (font, s) in &t.sections {
            let boxed = glyph(font, '◻');
            let mut lacking = String::new();
            for c in s.chars().filter(|c| !c.is_whitespace() && *c != '◻') {
                let key = (format!("{font:?}"), c);
                let missing = match seen.get(&key) {
                    Some(m) => *m,
                    None => {
                        let m = glyph(font, c) == boxed;
                        seen.insert(key, m);
                        m
                    }
                };
                if missing {
                    lacking.push(c);
                }
            }
            if !lacking.is_empty() {
                out.push(format!("{:?} lacks {lacking:?} in {font:?}", t.text));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Whether two colours are within `tol` in every channel.
pub fn near(a: Color32, b: Color32, tol: u8) -> bool {
    [(a.r(), b.r()), (a.g(), b.g()), (a.b(), b.b())]
        .iter()
        .all(|(x, y)| x.abs_diff(*y) <= tol)
}

/// The map's colours, and its legend, against the lens of the frame `f` in `lenses`: each region
/// painted in its lens colour, each legend segment in the colour of its place on the scale,
/// each region's colour inside the legend's segment at its value, each mark where the
/// view-model puts it, and the reference marked and named only where it lies on the scale; then
/// the mesh's vertices and the bar's segments as painted in `shapes`.
#[allow(clippy::too_many_arguments)]
pub fn check_colours(
    f: &MapFrame,
    shapes: &[ClippedShape],
    store: &Store,
    lenses: &[Lens],
    atlas: &Atlas,
    geo: &Geo,
    what: &str,
) {
    let key = f.lens.as_deref().expect("a lens was drawn");
    let l = lenses
        .iter()
        .find(|l| l.key == key)
        .unwrap_or_else(|| panic!("no lens {key}"));
    let vm = vm_map::lens(store, Origin::Run, atlas, l, Some(f.tick.expect("a tick")))
        .expect("a loaded run");
    let mut wrong = Vec::new();
    for (d, r) in f.regions.iter().zip(&vm.regions) {
        let at = r.at.expect("a value");
        if d.colour != ui_map::colour(&vm.scale, at) {
            wrong.push(r.key.clone());
        }
    }
    assert!(
        wrong.is_empty(),
        "{what}, {}: painted in another colour: {wrong:?}",
        l.key
    );
    let lg = &f.legend;
    assert_eq!(lg.segments.len(), 64, "{what}");
    for &(a, b, c) in &lg.segments {
        assert_eq!(
            c,
            ui_map::colour(&vm.scale, (a + b) / 2.0),
            "{what}, {}",
            l.key
        );
    }
    for (d, r) in f.regions.iter().zip(&vm.regions) {
        let at = r.at.expect("a value");
        let &(a, b, c) = lg
            .segments
            .iter()
            .find(|(a, b, _)| *a <= at && at <= *b)
            .expect("a segment holds every place on the scale");
        let (ca, cb) = (ui_map::colour(&vm.scale, a), ui_map::colour(&vm.scale, b));
        let spread = [(ca.r(), cb.r()), (ca.g(), cb.g()), (ca.b(), cb.b())]
            .iter()
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        assert!(
            near(d.colour, c, spread + 1),
            "{what}, {}: {} is {:?}, the legend at {at} {c:?}",
            l.key,
            r.key,
            d.colour
        );
    }
    assert_eq!(lg.marks.len(), vm.legend.len(), "{what}");
    for (&(value, along), t) in lg.marks.iter().zip(&vm.legend) {
        assert_eq!(value, t.value);
        assert!(
            (along - t.at).abs() < 1e-4,
            "{what}, {}: mark {value}",
            l.key
        );
    }
    let on_scale = vm.reference.as_ref().and_then(|r| r.at);
    assert_eq!(
        lg.reference.is_some(),
        on_scale.is_some(),
        "{what}, {}",
        l.key
    );
    if let (Some(a), Some(b)) = (lg.reference, on_scale) {
        assert!((a - b).abs() < 1e-4);
    }
    // The header names the reference, by its mark `|`, only where it lies on the scale.
    assert_eq!(
        lg.head.contains(" | "),
        on_scale.is_some(),
        "{}: {}",
        l.key,
        lg.head
    );
    painted_colours(shapes, geo, &vm, lg, what, &l.key);
}

/// The colours as painted, read from the frame's shapes rather than from what the painter
/// recorded (G1, O26's C6, C7 and C8): every vertex of the fill mesh, region by region in the
/// atlas's order, takes its region's lens colour; and the legend's bar is painted as 64
/// segments left to right, each the scale's colour at its middle.
fn painted_colours(
    shapes: &[ClippedShape],
    geo: &Geo,
    vm: &LensVm,
    lg: &LegendFrame,
    what: &str,
    key: &str,
) {
    let mut meshes = Vec::new();
    let mut rects = Vec::new();
    fn walk(
        s: &Shape,
        meshes: &mut Vec<std::sync::Arc<egui::Mesh>>,
        rects: &mut Vec<(Rect, Color32)>,
    ) {
        match s {
            Shape::Mesh(m) => meshes.push(std::sync::Arc::clone(m)),
            Shape::Rect(r) => rects.push((r.rect, r.fill)),
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, meshes, rects)),
            _ => {}
        }
    }
    for c in shapes {
        walk(&c.shape, &mut meshes, &mut rects);
    }
    let n: usize = geo.fills.iter().map(|f| f.points.len()).sum();
    let mesh = meshes
        .iter()
        .find(|m| m.vertices.len() == n)
        .expect("the fill mesh is painted");
    let mut first = 0;
    let mut wrong = Vec::new();
    for (f, r) in geo.fills.iter().zip(&vm.regions) {
        let want = ui_map::colour(&vm.scale, r.at.expect("a value"));
        let span = &mesh.vertices[first..first + f.points.len()];
        let bad = span.iter().filter(|v| v.color != want).count();
        if bad > 0 {
            wrong.push(format!("{}: {bad} of {}", r.key, span.len()));
        }
        first += f.points.len();
    }
    assert!(
        wrong.is_empty(),
        "{what}, {key}: vertices painted in another colour: {wrong:?}"
    );
    let legend = lg.rect.expect("the legend was drawn");
    let mut bar: Vec<(Rect, Color32)> = rects
        .into_iter()
        .filter(|(r, c)| c.a() > 0 && legend.contains_rect(*r) && (r.height() - 12.0).abs() < 0.01)
        .collect();
    bar.sort_by(|a, b| a.0.min.x.total_cmp(&b.0.min.x));
    assert_eq!(bar.len(), 64, "{what}, {key}: the bar's segments");
    for (k, (_, c)) in bar.iter().enumerate() {
        let at = (k as f64 + 0.5) / 64.0;
        assert_eq!(
            *c,
            ui_map::colour(&vm.scale, at),
            "{what}, {key}: segment {k} of the bar as painted"
        );
    }
}

/// Drag the map by `by` from `at`, and take the pointer away.
pub fn pan<S>(h: &mut egui_kittest::Harness<'_, S>, at: egui::Pos2, by: egui::Vec2) {
    h.drag_at(at);
    h.step();
    h.hover_at(at + by);
    h.step();
    h.drop_at(at + by);
    h.remove_cursor();
    h.run();
}
