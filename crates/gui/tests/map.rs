//! The map and its lenses (docs/GUI.md §4, §6, §8.1's map row; D.3, 2026-09-27), on the demo
//! world `tapes/demo-gb.ron` (docs/demo/WORLD.md).
//!
//! - `the_atlas_triangulates_and_labels_hit_their_regions`: every node key of the demo tape is
//!   a region; each region's triangles cover its drawn area to a relative 1e-9; each label
//!   point lies in a triangle of its own region.
//! - `every_region_drawn_once`: each region's fill goes into the frame's mesh once, and each
//!   shared border is stroked once; a region the tape lacks is drawn hatched, once.
//! - `map_values_equal_table`: for every lens at report ticks 0, 51 and 259, each row of the
//!   ranked table shows the value, rank and colour of the region the map drew, and every region
//!   with a value has its row, because one view-model makes both.
//! - `lens_domains_are_fixed_for_the_run`: a lens's domain, centre, reference and legend are
//!   the table's and do not move as the run does, while its values do.
//! - `demo_lens_view_models_equal_their_goldens`: every lens's view-model at report tick 0,
//!   saved as RON (`UPDATE_GOLDEN=1` rewrites them).
//! - `the_demo_script_switches_lenses_runs_hovers_and_selects`: the app opens the demo tape on
//!   the map with the atlas's credit painted, every lens is switched to by its hotkeys, a year
//!   runs, a county's hover card names its value, unit, run and tick, and a click selects it
//!   for the inspector.
//! - `map_mesh_frame_time_is_recorded` (ignored): a frame's fill mesh and thinned borders,
//!   headless, median of 60.
//!
//! Added at D.4, after D.3's verification:
//!
//! - `lens_values_equal_the_engine`: sixteen lenses at report ticks 0, 51 and 259 for every
//!   county against the formula on the engine's own report, params and states, read from a Sim
//!   the test steps; `no.trade`'s window ends at the cursor.
//! - `the_scales_are_the_neutral_palettes`: viridis's ends and middle, and purple to orange
//!   through white.
//! - `rebuilt_mesh_colours_are_the_lens_colours`: the mesh built afresh (the first frame, a
//!   pan) paints each region in its lens colour, and the legend's segments, marks and
//!   reference agree with the scale.
//! - `every_triangle_hits_its_region` and `hover_and_click_name_every_part`: the hit test at
//!   every triangle, and the hover card and the click at a point of every part.
//! - `the_credit_is_painted_clear_of_the_legend`: the atlas's credit on every map, inside
//!   the canvas and clear of the legend, at four widths.

mod common;

use certify::Build;
use common::key;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use rustyecon_engine::prelude::*;
use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::model::Intent;
use rustyecon_gui::run::{Catalogue, Cmd, Entity, Obs, Origin, RunStatus, Runner, Store};
use rustyecon_gui::ui::layout::Pane;
use rustyecon_gui::ui::map::{self as ui_map, Edge, Geo, MapState};
use rustyecon_gui::vm::map::{self as vm_map, Beyond, LensVm};
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::tables::Lens;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The demo world's tape, `tapes/demo-gb.ron`.
fn demo_path() -> String {
    format!("{}/../../tapes/demo-gb.ron", env!("CARGO_MANIFEST_DIR"))
}

fn demo_tape() -> &'static Tape {
    static T: OnceLock<Tape> = OnceLock::new();
    T.get_or_init(|| {
        let text = std::fs::read_to_string(demo_path()).expect("tapes/demo-gb.ron");
        Tape::from_ron(&text).expect("the demo tape parses")
    })
}

fn atlas() -> &'static Atlas {
    static A: OnceLock<Atlas> = OnceLock::new();
    A.get_or_init(|| Atlas::gb().expect("the bundled atlas"))
}

fn lenses() -> &'static [Lens] {
    static L: OnceLock<Vec<Lens>> = OnceLock::new();
    L.get_or_init(|| rustyecon_worldgen::lens::demo_gb().expect("the demo world's lenses"))
}

/// A Runner with a fixed build, stepped by hand, and the store of what it reported.
fn record(tape: &Tape, catalogue: Catalogue, ticks: u64) -> Store {
    let build = Build {
        commit: "golden".to_string(),
        dirty: false,
        target: "golden".to_string(),
        rustc: "golden".to_string(),
    };
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let mut runner = Runner::new(
        build,
        Box::new(move |o| sink.lock().unwrap().push(o)),
        Box::new(|| {}),
    );
    let mut store = Store::default();
    let take = |runner: &mut Runner, store: &mut Store, c: Cmd| {
        runner.handle(c);
        while runner.advance(97).busy {}
        let obs: Vec<Obs> = std::mem::take(&mut *seen.lock().unwrap());
        for o in obs {
            store.ingest(o).expect("the record takes it");
        }
    };
    take(&mut runner, &mut store, Cmd::Catalogue(catalogue));
    take(
        &mut runner,
        &mut store,
        Cmd::Load {
            tape: Box::new(tape.clone()),
            from: None,
        },
    );
    if ticks > 0 {
        take(&mut runner, &mut store, Cmd::Step(ticks));
    }
    assert_eq!(store.tick(), ticks);
    store
}

/// The demo world run five years, with the catalogue the model gives a world of 93 nodes.
fn demo_store() -> &'static Store {
    static S: OnceLock<Store> = OnceLock::new();
    S.get_or_init(|| {
        let tape = demo_tape();
        assert_eq!(Catalogue::for_nodes(tape.nodes.len()), Catalogue::Lean);
        record(tape, Catalogue::Lean, 260)
    })
}

/// A tape with one county: the Appendix B world with its node called `county.lan`.
fn lancashire_alone() -> Tape {
    let text = common::APPB.replace("\"home\"", "\"county.lan\"");
    Tape::from_ron(&text).expect("the renamed tape parses")
}

fn lens(key: &str) -> &'static Lens {
    lenses()
        .iter()
        .find(|l| l.key == key)
        .unwrap_or_else(|| panic!("no lens {key}"))
}

fn lens_vm(store: &Store, l: &Lens, tick: u64) -> LensVm {
    vm_map::lens(store, Origin::Run, atlas(), l, Some(tick)).expect("a loaded run")
}

/// The map pane alone, drawing `store` at a report tick.
struct Pane1 {
    map: MapState,
    cursor: Option<u64>,
    selection: Option<Entity>,
    out: Vec<Intent>,
}

fn map_harness(store: &'static Store, height: f32) -> Harness<'static, Pane1> {
    map_harness_sized(store, egui::vec2(1600.0, height))
}

/// The map pane alone at `size`.
fn map_harness_sized(store: &'static Store, size: egui::Vec2) -> Harness<'static, Pane1> {
    let mut map = MapState::default();
    map.ready().expect("the atlas and the lenses read");
    Harness::builder().with_size(size).build_ui_state(
        move |ui, p: &mut Pane1| {
            let vm = {
                let (g, l) = p.map.parts().expect("ready");
                vm_map::build(
                    store,
                    Origin::Run,
                    &g.atlas,
                    l,
                    &p.map.lens,
                    p.selection.as_ref(),
                    p.cursor,
                )
                .expect("a loaded run")
            };
            ui_map::show(ui, &vm, &mut p.map, &mut p.out);
        },
        Pane1 {
            map,
            cursor: None,
            selection: None,
            out: Vec::new(),
        },
    )
}

#[test]
fn the_atlas_triangulates_and_labels_hit_their_regions() {
    let geo = Geo::build(atlas().clone());
    assert_eq!(geo.fills.len(), 93);
    for (i, (r, f)) in geo.atlas.regions.iter().zip(&geo.fills).enumerate() {
        let drawn = r.drawn_area_km2() * 1e6;
        let rel = (f.area() - drawn).abs() / drawn;
        assert!(
            rel <= 1e-9,
            "{}: triangles {} m², drawn {drawn} m²",
            r.key,
            f.area()
        );
        // The label point lies in a triangle of its own region.
        let p = r.label;
        let inside = f.triangles.chunks_exact(3).any(|t| {
            let [a, b, c] = [t[0], t[1], t[2]].map(|k| f.points[k as usize]);
            let side = |u: [f64; 2], v: [f64; 2]| {
                (v[0] - u[0]) * (p[1] - u[1]) - (v[1] - u[1]) * (p[0] - u[0])
            };
            let (s1, s2, s3) = (side(a, b), side(b, c), side(c, a));
            (s1 >= 0.0 && s2 >= 0.0 && s3 >= 0.0) || (s1 <= 0.0 && s2 <= 0.0 && s3 <= 0.0)
        });
        assert!(
            inside,
            "{}'s label point is in none of its triangles",
            r.key
        );
        assert_eq!(geo.region_at(p), Some(i), "{}'s label point hits it", r.key);
    }
    // Every node key of the demo tape is a region, and every region a node.
    let w = demo_store().world().expect("loaded");
    assert!(vm_map::geography(w, atlas()).is_ok());
    let nodes: Vec<&str> = w.nodes.iter().map(|n| n.key.as_str()).collect();
    let regions: Vec<&str> = atlas().regions.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(nodes, regions);
    // A tape with nodes the atlas lacks has no map, and says why.
    let gate = Tape::from_ron(common::GATE).unwrap();
    let gate_world = rustyecon_engine::Sim::new(&gate).unwrap().world().clone();
    let why = vm_map::geography(&gate_world, atlas()).unwrap_err();
    assert!(why.starts_with("no map"), "{why}");
}

#[test]
fn every_region_drawn_once() {
    let store = demo_store();
    let mut h = map_harness(store, 1000.0);
    h.state_mut().cursor = Some(51);
    h.run();
    let f = h.state().map.frame().clone();
    assert_eq!(f.lens.as_deref(), Some(ui_map::DEFAULT_LENS));
    assert_eq!(f.regions.len(), 93);
    let keys: Vec<&str> = f.regions.iter().map(|r| r.key.as_str()).collect();
    let atlas_keys: Vec<&str> = atlas().regions.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, atlas_keys, "each region once, in the atlas's order");
    assert!(f.regions.iter().all(|r| r.fills == 1 && !r.hatched));
    assert_eq!(f.borders.len(), atlas().borders.len());
    assert!(
        f.borders.iter().all(|&n| n == 1),
        "each border stroked once"
    );
    let points: usize = Geo::build(atlas().clone())
        .fills
        .iter()
        .map(|x| x.points.len())
        .sum();
    assert_eq!(f.vertices, points, "one mesh of every region's vertices");
    // What was painted: one mesh of that many vertices, and one path for each border, none
    // painted twice either way round (a region-by-region outline would stroke each shared
    // border twice).
    let dark = true;
    let strokes = [Edge::Coast, Edge::County, Edge::Nation].map(|e| ui_map::border_stroke(e, dark));
    let mut meshes = Vec::new();
    let mut paths: Vec<Vec<egui::Pos2>> = Vec::new();
    fn walk(
        s: &egui::Shape,
        strokes: &[egui::Stroke],
        meshes: &mut Vec<usize>,
        paths: &mut Vec<Vec<egui::Pos2>>,
    ) {
        match s {
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, strokes, meshes, paths)),
            egui::Shape::Mesh(m) => meshes.push(m.vertices.len()),
            egui::Shape::Path(p) => {
                let solid = |c: &egui::epaint::ColorMode| match c {
                    egui::epaint::ColorMode::Solid(c) => Some(*c),
                    _ => None,
                };
                let hit = strokes
                    .iter()
                    .any(|s| s.width == p.stroke.width && Some(s.color) == solid(&p.stroke.color));
                if hit && !p.closed {
                    paths.push(p.points.clone());
                }
            }
            _ => {}
        }
    }
    for c in &h.output().shapes {
        walk(&c.shape, &strokes, &mut meshes, &mut paths);
    }
    // egui paints small meshes of its own (a few vertices each); the map's fill is the one
    // large mesh.
    let large: Vec<usize> = meshes.into_iter().filter(|&n| n > 1_000).collect();
    assert_eq!(large, [points], "one fill mesh");
    assert_eq!(paths.len(), atlas().borders.len(), "a path per border");
    // No path of three points or more is painted twice, either way round. A border shorter
    // than the thinning thins to its two ends, and two such can share both.
    let mut seen = std::collections::BTreeSet::new();
    let mut twice = Vec::new();
    for p in paths.iter().filter(|p| p.len() > 2) {
        let key = |q: &[egui::Pos2]| {
            q.iter()
                .map(|x| (x.x.to_bits(), x.y.to_bits()))
                .collect::<Vec<_>>()
        };
        let mut back = p.clone();
        back.reverse();
        if seen.contains(&key(&back)) || !seen.insert(key(p)) {
            twice.push(p.len());
        }
    }
    assert!(twice.is_empty(), "borders painted twice: {twice:?}");
    assert!(
        seen.len() > 500,
        "{} borders of three points or more",
        seen.len()
    );

    // A tape with one county: the other 92 regions are drawn once each, hatched.
    static ONE: OnceLock<Store> = OnceLock::new();
    let one = ONE.get_or_init(|| record(&lancashire_alone(), Catalogue::Full, 3));
    let mut h = map_harness(one, 1000.0);
    h.run();
    let f = h.state().map.frame().clone();
    assert_eq!(f.regions.len(), 93);
    assert!(f.regions.iter().all(|r| r.fills == 1));
    let plain: Vec<&str> = f
        .regions
        .iter()
        .filter(|r| !r.hatched)
        .map(|r| r.key.as_str())
        .collect();
    assert_eq!(plain, ["county.lan"]);
    let hatched = f.regions.iter().filter(|r| r.hatched).count();
    assert_eq!(hatched, 92);
    let vm = lens_vm(one, lens("wage.land"), 2);
    let lan = vm.regions.iter().find(|r| r.key == "county.lan").unwrap();
    assert!(lan.in_tape && lan.value.is_some(), "{lan:?}");
    assert!(vm
        .regions
        .iter()
        .filter(|r| r.key != "county.lan")
        .all(|r| !r.in_tape && r.value.is_none()));
}

#[test]
fn map_values_equal_table() {
    let store = demo_store();
    // Tall enough that the ranked table draws every row.
    let mut h = map_harness(store, 3200.0);
    let mut checked = 0;
    for tick in [0, 51, 259] {
        for l in lenses() {
            h.state_mut().cursor = Some(tick);
            h.state_mut().map.lens = l.key.clone();
            h.run();
            let f = h.state().map.frame().clone();
            assert_eq!(f.lens.as_deref(), Some(l.key.as_str()));
            assert_eq!(f.tick, Some(tick));
            let vm = lens_vm(store, l, tick);
            // The map drew the view-model's values.
            for (d, r) in f.regions.iter().zip(&vm.regions) {
                assert_eq!(d.key, r.key);
                assert_eq!(d.value.map(f64::to_bits), r.value.map(f64::to_bits));
            }
            // The table drew one row per region with a value, in rank order, each with the
            // map's value and colour.
            assert_eq!(f.rows.len(), vm.ranked.len(), "{} at {tick}", l.key);
            for (k, row) in f.rows.iter().enumerate() {
                let d = f
                    .regions
                    .iter()
                    .find(|d| d.key == row.key)
                    .expect("a row's region is on the map");
                assert_eq!(Some(row.value.to_bits()), d.value.map(f64::to_bits));
                assert_eq!(row.colour, d.colour, "{} {}", l.key, row.key);
                assert_eq!(row.rank, k + 1);
                assert_eq!(row.text, rustyecon_gui::ui::fmt(row.value));
                checked += 1;
            }
            let valued = f.regions.iter().filter(|d| d.value.is_some()).count();
            assert_eq!(valued, f.rows.len());
            if l.source != rustyecon_worldgen::tables::LensSource::Observe {
                assert_eq!(valued, 93, "{} at {tick}: every county has a value", l.key);
            } else {
                assert_eq!(valued, 0, "{} waits for crates/observe", l.key);
            }
        }
    }
    assert!(checked > 5_000, "{checked} rows checked");
}

#[test]
fn lens_domains_are_fixed_for_the_run() {
    let store = demo_store();
    let mut moved = 0;
    for l in lenses() {
        let at: Vec<LensVm> = [0, 51, 259].map(|t| lens_vm(store, l, t)).into();
        for v in &at {
            // The domain is the table's, registered before any run; the legend and the
            // reference follow from it alone.
            assert_eq!(v.domain, l.domain, "{}", l.key);
            assert_eq!(v.legend, vm_map::legend(l), "{}", l.key);
            assert_eq!(v.centre, at[0].centre);
            assert_eq!(v.reference, at[0].reference);
            assert_eq!(v.scale, at[0].scale);
            assert!(v
                .legend
                .first()
                .is_some_and(|t| t.value == l.domain.0 && t.at == 0.0));
            assert!(v
                .legend
                .last()
                .is_some_and(|t| t.value == l.domain.1 && t.at == 1.0));
            assert!(v
                .legend
                .windows(2)
                .all(|w| w[0].value < w[1].value && w[0].at < w[1].at));
            for r in &v.regions {
                if let (Some(x), Some(a)) = (r.value, r.at) {
                    assert!((0.0..=1.0).contains(&a));
                    let want = if x < l.domain.0 {
                        Beyond::Below
                    } else if x > l.domain.1 {
                        Beyond::Above
                    } else {
                        Beyond::Within
                    };
                    assert_eq!(r.beyond, want, "{} {}", l.key, r.key);
                }
            }
        }
        if l.scale == rustyecon_worldgen::tables::Scale::Diverging {
            let c = at[0].centre.expect("a diverging lens has a centre");
            assert_eq!(
                vm_map::position(l, c).0,
                0.5,
                "{} centres on its reference",
                l.key
            );
        }
        let differs = at[0]
            .regions
            .iter()
            .zip(&at[2].regions)
            .any(|(a, b)| a.value.map(f64::to_bits) != b.value.map(f64::to_bits));
        moved += usize::from(differs);
    }
    // The run moves the values under the fixed scales: the map recolours as it advances.
    assert!(moved >= 15, "{moved} lenses moved");
}

/// Compare a view-model with its golden file, `tests/golden/<point>/<name>.ron`, or write it
/// under `UPDATE_GOLDEN=1`.
fn compare(point: &str, name: &str, value: &impl Serialize, depth: usize) {
    let pretty = ron::ser::PrettyConfig::new()
        .new_line("\n".to_string())
        .depth_limit(depth);
    let text = ron::ser::to_string_pretty(value, pretty).expect("a golden serialises") + "\n";
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(point)
        .join(format!("{name}.ron"));
    if std::env::var_os("UPDATE_GOLDEN").is_some_and(|v| v == "1") {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{}: {e}; run with UPDATE_GOLDEN=1", p.display()))
        .replace("\r\n", "\n");
    if want != text {
        let (n, (a, b)) = want
            .lines()
            .zip(text.lines())
            .enumerate()
            .find(|(_, (a, b))| a != b)
            .unwrap_or((
                want.lines().count().min(text.lines().count()),
                ("<end>", "<end>"),
            ));
        panic!(
            "{point}/{name}: the view-model differs from {} at line {}:\n  golden: {a}\n  now:    \
             {b}\n(UPDATE_GOLDEN=1 rewrites it, in the commit that retunes the world)",
            p.display(),
            n + 1
        );
    }
}

#[test]
fn demo_lens_view_models_equal_their_goldens() {
    let store = demo_store();
    for l in lenses() {
        let v = lens_vm(store, l, 0);
        // What each golden shows, asserted before it is compared.
        let run = v.run.as_ref().expect("the run is named");
        assert_eq!(run.tape_hash, "0x1bd56d66433d3b67");
        assert_eq!(run.name, "demo-gb [illustrative]");
        assert_eq!((v.tick, v.date.as_str()), (Some(0), "1750-01-01"));
        assert_eq!(v.regions.len(), 93);
        assert_eq!(v.counts.in_tape, 93);
        match l.source {
            rustyecon_worldgen::tables::LensSource::Observe => {
                assert!(v.unavailable.is_some() && v.counts.valued == 0, "{}", l.key)
            }
            _ => assert_eq!(v.counts.valued, 93, "{}", l.key),
        }
        if l.key.starts_with("since.") {
            assert!(
                v.regions.iter().all(|r| r.value == Some(0.0)),
                "{}: no change at the record's first tick",
                l.key
            );
        }
        compare("demo-gb-tick0", &format!("lens-{}", l.key), &v, 2);
    }
    // At genesis each county sits at its oracle point, so the wage in land is the oracle's v
    // (docs/demo/WORLD.md §3.3: Middlesex 1.87, Lancashire 1.52, the West Riding 1.34).
    let v = lens_vm(store, lens("wage.land"), 0);
    let at = |k: &str| {
        v.regions
            .iter()
            .find(|r| r.key == k)
            .and_then(|r| r.value)
            .expect("a value")
    };
    for (k, want) in [
        ("county.mdx", 1.87),
        ("county.lan", 1.52),
        ("county.wry", 1.34),
    ] {
        assert!((at(k) - want).abs() < 0.01, "{k}: {} against {want}", at(k));
    }
}

/// Step frames until `done` holds of the app, a minute at most.
fn step_until(h: &mut Harness<'_, GuiApp>, what: &str, done: impl Fn(&GuiApp) -> bool) {
    let t0 = Instant::now();
    while !done(h.state()) {
        h.step();
        assert!(
            t0.elapsed() < Duration::from_secs(300),
            "the app did not get there: {what}"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Every text the last frame painted.
fn painted(h: &Harness<'_, GuiApp>) -> Vec<String> {
    fn walk(s: &egui::Shape, out: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in &h.output().shapes {
        walk(&c.shape, &mut out);
    }
    out
}

/// Step until the frame paints a text that contains `text`, 20 frames at most.
fn paints(h: &mut Harness<'_, GuiApp>, text: &str) {
    for _ in 0..20 {
        if painted(h).iter().any(|t| t.contains(text)) {
            return;
        }
        h.step();
    }
    panic!("the frame paints no {text:?}");
}

#[test]
fn the_demo_script_switches_lenses_runs_hovers_and_selects() {
    let path = demo_path();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 1000.0))
        .build_eframe(move |cc| {
            GuiApp::new(
                &cc.egui_ctx,
                Launch {
                    tape: Some(path),
                    files: None,
                    smoke: None,
                },
            )
        });
    step_until(&mut h, "the demo tape loaded", |a| {
        a.model().focused().map(|r| r.store.status())
            == Some(RunStatus::Paused { tick: 0, why: None })
    });
    // It opens on the map, with nothing plotted and the lean catalogue logged.
    assert!(h.state().model().session.plots.is_empty());
    assert!(h.state().model().log().iter().any(|e| e
        .text
        .contains("93 nodes: the run records the lean catalogue")));
    h.step();
    h.step();
    assert!(h.state().ui_state().panes_drawn().contains(&Pane::Map));
    assert!(!h.state().ui_state().panes_drawn().contains(&Pane::Plots));
    paints(&mut h, "Wage in land");
    // The atlas's credit (data/atlas/ATTRIBUTION, ODbL 1.0), painted on the map.
    let credit = rustyecon_worldgen::atlas::CREDIT;
    assert!(credit[0].contains("Historic County Borders Project (county-borders.co.uk)"));
    assert!(credit[1].contains("© OpenStreetMap contributors, ODbL 1.0"));
    for line in credit {
        paints(&mut h, line);
    }
    assert_eq!(h.state().ui_state().map.frame().credit, credit);
    // Run a year: the map has values, named by the tick they are read at.
    click(&mut h, "Step a year");
    step_until(&mut h, "a year ran", |a| {
        a.model().focused().is_some_and(|r| r.store.tick() == 52)
    });
    let date = h
        .state()
        .model()
        .focused()
        .and_then(|r| r.store.world())
        .and_then(|w| w.clock.date_of(51))
        .expect("tick 51 has a date")
        .to_string();
    let when = format!("report tick 51, {date}");
    paints(&mut h, &when);
    // Every lens, by its hotkeys: `]` steps through all of them and the digits pick the first
    // ten; each paints its name and draws the map in its values.
    let n = lenses().len();
    let first = lenses()
        .iter()
        .position(|l| l.key == ui_map::DEFAULT_LENS)
        .expect("the default lens is listed");
    for k in 0..n {
        let want = &lenses()[(first + k + 1) % n];
        h.key_press(egui::Key::CloseBracket);
        h.step();
        h.step();
        let f = h.state().ui_state().map.frame().clone();
        assert_eq!(f.lens.as_deref(), Some(want.key.as_str()));
        assert_eq!(f.regions.len(), 93);
        paints(&mut h, &want.name);
    }
    h.key_press(egui::Key::Num3);
    h.step();
    h.step();
    assert_eq!(
        h.state().ui_state().map.frame().lens.as_deref(),
        Some(lenses()[2].key.as_str())
    );
    // Hover Lancashire: the card names the county, the value with its unit, the run and the
    // report tick.
    let lan = h
        .state()
        .ui_state()
        .map
        .frame()
        .regions
        .iter()
        .find(|r| r.key == "county.lan")
        .expect("Lancashire is drawn")
        .label;
    h.hover_at(lan);
    h.step();
    h.step();
    let card = h
        .state()
        .ui_state()
        .map
        .frame()
        .hover
        .clone()
        .expect("a hover card");
    let l = &lenses()[2];
    assert!(card[0].starts_with("Lancashire (county.lan"), "{card:?}");
    assert!(card[1].starts_with(&format!("{}: ", l.name)), "{card:?}");
    assert!(card[1].contains(&l.unit), "{card:?}");
    assert_eq!(card[2], when);
    assert!(
        card[3].starts_with("demo-gb [illustrative], run of build"),
        "{card:?}"
    );
    assert!(
        card[4].starts_with("tape_hash 0x1bd56d66433d3b67 · world_id 0x"),
        "{card:?}"
    );
    paints(&mut h, "Lancashire (county.lan, England)");
    // Click it: the county is selected by key, and the inspector and the card show it.
    h.event(egui::Event::PointerButton {
        pos: lan,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::PointerButton {
        pos: lan,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    step_until(&mut h, "Lancashire selected", |a| {
        a.model().selection() == Some(&Entity::Node(key("county.lan")))
    });
    h.remove_cursor();
    paints(&mut h, "node county.lan");
    paints(&mut h, "Every lens");
}

/// Click the node labelled `label`, then take the pointer away.
fn click(h: &mut Harness<'_, GuiApp>, label: &str) {
    h.get_by_label(label).click();
    h.step();
    h.remove_cursor();
    h.step();
}

#[test]
#[ignore = "a measurement: run by name, with --release"]
fn map_mesh_frame_time_is_recorded() {
    // docs/GUI.md §6, "Measured": a frame's map at 93 regions, the fill mesh of every region
    // and every border thinned and stroked, headless, median of 60 frames. No threshold.
    let geo = Geo::build(atlas().clone());
    let v = lens_vm(demo_store(), lens("wage.land"), 51);
    let colours = ui_map::colours(&v, true);
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1260.0, 900.0));
    let mut ms = Vec::new();
    let mut size = (0, 0);
    for _ in 0..60 {
        let t0 = Instant::now();
        let (mesh, shallows, lines) = ui_map::frame_shapes(&geo, &colours, rect);
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
        size = (mesh.vertices.len(), shallows.len() + lines.len());
    }
    ms.sort_by(f64::total_cmp);
    let t0 = Instant::now();
    let _ = Geo::build(atlas().clone());
    let build = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "map frame at 93 regions: median {:.3} ms (min {:.3}, max {:.3}) over 60; {} vertices, {} \
         strokes; triangulating the atlas once {build:.1} ms",
        ms[30], ms[0], ms[59], size.0, size.1
    );
}

// D.4 (2026-09-27): the verification of D.3 (verify-map-r1) found that the value path was
// checked only against itself and at tick 0, that the mesh built afresh and the legend were
// never held to the lens colours, that the palettes were unpinned, that the hit test was tried
// only at label points, and that the map showed no credit for its data. Each test below fails
// on the mutant or the omission it names.

/// One county's numbers after a report tick, read by the test from its own `Sim`, not from the
/// GUI's record.
#[derive(Debug, Clone, Default)]
struct Truth {
    /// w, r, p_m, p.
    prices: [f64; 4],
    /// Labour, land, machine services and the good cleared.
    cleared: [f64; 4],
    space: f64,
    eta: f64,
    /// N per tick, through the clock's flow conversion.
    n_tick: f64,
    share: f64,
    due: f64,
    paid: f64,
    /// Whether all four markets traded, at every report tick from 0 to this one.
    traded: Vec<bool>,
}

/// The engine's own numbers for every county at each report tick of `ticks`.
fn engine_truth(ticks: &[u64]) -> BTreeMap<u64, BTreeMap<String, Truth>> {
    let tape = demo_tape();
    let mut sim = Sim::new(tape).expect("the demo tape loads");
    let w = sim.world().clone();
    let goods: Vec<GoodId> = ["labour", "land", "mach", "good"]
        .iter()
        .map(|g| w.id_of::<GoodId>(g).expect("a good"))
        .collect();
    let last = *ticks.iter().max().expect("a tick");
    let mut index: BTreeMap<(NodeId, GoodId), usize> = BTreeMap::new();
    let mut traded: BTreeMap<String, Vec<bool>> = BTreeMap::new();
    let mut out = BTreeMap::new();
    for t in 0..=last {
        let r = sim.step().expect("the demo steps");
        assert_eq!(r.tick, t);
        if t == 0 {
            index = r
                .markets
                .iter()
                .enumerate()
                .map(|(i, l)| ((l.node, l.good), i))
                .collect();
        }
        let line = |n: NodeId, g: GoodId| {
            let l = &r.markets[index[&(n, g)]];
            assert_eq!((l.node, l.good), (n, g));
            l
        };
        for n in &w.nodes {
            let all = goods.iter().all(|&g| line(n.id, g).trades());
            traded
                .entry(n.key.as_str().to_string())
                .or_default()
                .push(all);
        }
        if !ticks.contains(&t) {
            continue;
        }
        let mut per = BTreeMap::new();
        for n in &w.nodes {
            let k = n.key.as_str();
            let mut x = Truth::default();
            for (i, &g) in goods.iter().enumerate() {
                x.prices[i] = line(n.id, g).price;
                x.cleared[i] = line(n.id, g).cleared;
            }
            let p = |c: &str| {
                sim.param(w.id_of::<ParamId>(&format!("{k}.{c}")).expect("a param"))
                    .expect("its value")
            };
            x.space = p("space");
            x.eta = p("eta");
            x.n_tick = ClockMethod::Flow
                .per_tick(&w.clock, p("workers"))
                .expect("a flow converts");
            let a = |role: &str| {
                w.id_of::<ActorId>(&format!("{k}.{role}"))
                    .expect("an actor")
            };
            match sim.actor_state(a("desk.good")) {
                Some(ActorState::GoodDesk(s)) => x.share = s.share,
                other => panic!("{k}: {other:?}"),
            }
            match sim.actor_state(a("provider")) {
                Some(ActorState::Provider(s)) => (x.due, x.paid) = (s.due, s.paid),
                other => panic!("{k}: {other:?}"),
            }
            x.traded = traded[k].clone();
            per.insert(k.to_string(), x);
        }
        out.insert(t, per);
    }
    out
}

#[test]
fn lens_values_equal_the_engine() {
    // U3, U6 (D.4): at report ticks 0, 51 and 259 (a cursor behind live, and live), every
    // county's value under sixteen lenses equals the lens's formula applied to the engine's own
    // report, params and actor states, read from a Sim this test steps. Two change lenses read
    // tick 0 as their base, whatever the cursor. `no.trade` counts over the trailing year up to
    // the cursor and no further, and the readings' window is exactly those ticks.
    let ticks = [0_u64, 51, 259];
    let truth = engine_truth(&ticks);
    let store = demo_store();
    assert_eq!(
        store.tick(),
        260,
        "the record runs past every cursor but the last"
    );
    let ln = rustyecon_core::num::ln;
    let wb = |x: &Truth| x.prices[0] / (x.prices[3] + x.space * x.prices[1]);
    let oph = |x: &Truth| x.cleared[3] / x.n_tick;
    let window = |x: &Truth, t: u64| {
        let lo = (t + 1).saturating_sub(rustyecon_worldgen::lens::NO_TRADE_WINDOW) as usize;
        x.traded[lo..=t as usize].to_vec()
    };
    type Formula = Box<dyn Fn(&Truth, &Truth, u64) -> f64>;
    let formulas: Vec<(&str, Formula)> = vec![
        ("wage.land", Box::new(|x, _, _| x.prices[0] / x.prices[1])),
        ("wage.goods", Box::new(|x, _, _| x.prices[0] / x.prices[3])),
        ("rent.goods", Box::new(|x, _, _| x.prices[1] / x.prices[3])),
        ("price.good", Box::new(|x, _, _| x.prices[3] / x.prices[1])),
        ("price.mach", Box::new(|x, _, _| x.prices[2] / x.prices[1])),
        ("wage.baskets", Box::new(move |x, _, _| wb(x))),
        (
            "share.land",
            Box::new(|x, _, _| {
                let (wl, rt) = (x.prices[0] * x.cleared[0], x.prices[1] * x.cleared[1]);
                rt / (wl + rt)
            }),
        ),
        ("frontier.x", Box::new(|x, _, _| 1.0 - x.share)),
        ("participation", Box::new(|x, _, _| x.cleared[0] / x.n_tick)),
        ("output.per.head", Box::new(move |x, _, _| oph(x))),
        ("param.eta", Box::new(|x, _, _| x.eta)),
        (
            "relief.burden",
            Box::new(|x, _, _| x.due / (x.prices[1] * x.cleared[1])),
        ),
        ("shortfall", Box::new(|x, _, _| (x.due - x.paid) / x.due)),
        (
            "since.wage.baskets",
            Box::new(move |x, x0, _| ln(wb(x) / wb(x0))),
        ),
        (
            "since.output.per.head",
            Box::new(move |x, x0, _| ln(oph(x) / oph(x0))),
        ),
        (
            "no.trade",
            Box::new(move |x, _, t| window(x, t).iter().filter(|a| !**a).count() as f64),
        ),
    ];
    let mut checked = 0;
    let mut wrong = Vec::new();
    for &t in &ticks {
        for (key, f) in &formulas {
            let vm = lens_vm(store, lens(key), t);
            assert_eq!(vm.tick, Some(t));
            for r in &vm.regions {
                let x = &truth[&t][&r.key];
                let want = f(x, &truth[&0][&r.key], t);
                let got = r.value.expect("a value");
                if (got - want).abs() > 1e-12 * want.abs().max(1e-3) {
                    wrong.push(format!("{key} {} at {t}: map {got}, engine {want}", r.key));
                }
                checked += 1;
            }
        }
        // The trailing window the record gives `no.trade`: the ticks up to the cursor, a year
        // at most, and not one past it.
        for r in &atlas().regions {
            let got = vm_map::readings(store, &r.key, t).traded;
            assert_eq!(got, window(&truth[&t][&r.key], t), "{} at {t}", r.key);
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(12)]
    );
    assert_eq!(checked, 3 * 16 * 93);
}

#[test]
fn the_scales_are_the_neutral_palettes() {
    // U-rules: viridis for a sequential lens, purple to orange through white for a diverging
    // one, nothing red against green. The ends and the middle are pinned, so a reversed or a
    // good/bad palette fails here even if the map, the table and the legend move together.
    let rgb = |s: &str, at: f64| {
        let c = ui_map::colour(s, at);
        [c.r(), c.g(), c.b()]
    };
    for s in ["sequential", "sequential, log"] {
        assert_eq!(
            rgb(s, 0.0),
            [0x44, 0x01, 0x54],
            "{s}: viridis starts dark purple"
        );
        assert_eq!(
            rgb(s, 0.5),
            [0x20, 0x90, 0x8c],
            "{s}: viridis's teal middle"
        );
        assert_eq!(rgb(s, 1.0), [0xfd, 0xe7, 0x25], "{s}: viridis ends yellow");
        // Out of range clamps to the ends.
        assert_eq!(rgb(s, -1.0), rgb(s, 0.0));
        assert_eq!(rgb(s, 2.0), rgb(s, 1.0));
    }
    let [r0, g0, b0] = rgb("diverging", 0.0);
    let [r1, g1, b1] = rgb("diverging", 1.0);
    let mid = rgb("diverging", 0.5);
    assert!(
        b0 > r0 && b0 > g0,
        "the low end is purple: {:?}",
        [r0, g0, b0]
    );
    assert!(
        r1 > g1 && g1 > b1,
        "the high end is orange-brown: {:?}",
        [r1, g1, b1]
    );
    // colorous's purple-orange is [243, 238, 234] at its centre: near white, no hue to speak of.
    assert!(
        mid.iter().all(|&c| c >= 0xe6)
            && mid.iter().max().unwrap() - mid.iter().min().unwrap() <= 12,
        "the centre is near white: {mid:?}"
    );
    // No end is green against red.
    for [r, g, b] in [rgb("sequential", 0.0), [r0, g0, b0], [r1, g1, b1]] {
        assert!(!(g > r && g > b), "an end is green: {:?}", [r, g, b]);
    }
}

/// Whether two colours are within `tol` in every channel.
fn near(a: egui::Color32, b: egui::Color32, tol: u8) -> bool {
    [(a.r(), b.r()), (a.g(), b.g()), (a.b(), b.b())]
        .iter()
        .all(|(x, y)| x.abs_diff(*y) <= tol)
}

/// The map's colours, and its legend, against the lens: each region painted in its lens colour,
/// each legend segment in the colour of its place on the scale, each region's colour inside the
/// legend's segment at its value, each mark where the view-model puts it, and the reference
/// marked and named only where it lies on the scale.
fn check_colours(h: &Harness<'static, Pane1>, store: &Store, what: &str) {
    let f = h.state().map.frame().clone();
    let l = lens(f.lens.as_deref().expect("a lens was drawn"));
    let vm = lens_vm(store, l, f.tick.expect("a tick"));
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
    assert_eq!(
        lg.head.contains('▏'),
        on_scale.is_some(),
        "{}: {}",
        l.key,
        lg.head
    );
}

/// Drag the map by `by` from the canvas's middle, and take the pointer away.
fn pan(h: &mut Harness<'static, Pane1>, by: egui::Vec2) {
    let at = egui::pos2(600.0, 500.0);
    h.drag_at(at);
    h.step();
    h.hover_at(at + by);
    h.step();
    h.drop_at(at + by);
    h.remove_cursor();
    h.run();
}

#[test]
fn rebuilt_mesh_colours_are_the_lens_colours() {
    // The canvas builds its fill mesh afresh on the first frame and whenever the view moves, and
    // recolours it in place otherwise (map_values_equal_table holds the second). Here the first
    // frame already has values, and each check follows a rebuild: the first frame, a pan, a
    // switch of lens and another pan. Then every lens, recoloured, against its legend.
    let store = demo_store();
    let mut h = map_harness(store, 1000.0);
    h.state_mut().cursor = Some(51);
    h.run();
    check_colours(&h, store, "the first frame");
    let before = h.state().map.frame().regions[0].label;
    pan(&mut h, egui::vec2(30.0, 10.0));
    let after = h.state().map.frame().regions[0].label;
    assert_ne!(before, after, "the pan moved the map");
    check_colours(&h, store, "after a pan");
    h.state_mut().map.lens = "since.output.per.head".to_string();
    h.run();
    pan(&mut h, egui::vec2(-50.0, 25.0));
    check_colours(&h, store, "a diverging lens after a pan");
    for l in lenses() {
        if l.source == rustyecon_worldgen::tables::LensSource::Observe {
            continue;
        }
        h.state_mut().map.lens = l.key.clone();
        h.run();
        check_colours(&h, store, "every lens");
    }
}

/// Each triangle of region `i`'s fill: its part, its centroid and its inradius, metres.
fn triangles(geo: &Geo, i: usize) -> Vec<(usize, [f64; 2], f64)> {
    let r = &geo.atlas.regions[i];
    let f = &geo.fills[i];
    // Each part's points were appended in turn: its rings without their closing points.
    let mut bases = Vec::new();
    let mut n = 0usize;
    for part in &r.parts {
        bases.push(n);
        n += part.rings.iter().map(|q| q.len() - 1).sum::<usize>();
    }
    assert_eq!(n, f.points.len(), "{}", r.key);
    let part_of = |k: usize| bases.iter().rposition(|&b| b <= k).expect("a part");
    f.triangles
        .chunks_exact(3)
        .map(|t| {
            let [a, b, c] = [t[0], t[1], t[2]].map(|k| f.points[k as usize]);
            let area = ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0;
            let d = |u: [f64; 2], v: [f64; 2]| {
                ((u[0] - v[0]) * (u[0] - v[0]) + (u[1] - v[1]) * (u[1] - v[1])).sqrt()
            };
            let per = d(a, b) + d(b, c) + d(c, a);
            let centre = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
            (part_of(t[0] as usize), centre, 2.0 * area / per)
        })
        .collect()
}

fn geo() -> &'static Geo {
    static G: OnceLock<Geo> = OnceLock::new();
    G.get_or_init(|| Geo::build(atlas().clone()))
}

#[test]
fn every_triangle_hits_its_region() {
    // The hit test (Geo::region_at, Region::contains) at every triangle's centroid, not only at
    // the label points: every part, detached parts, islands and the three ridings included, and
    // no point of an enclave (a hole) given to the county around it. Triangles thinner than a
    // metre are left out: their centroid can round onto a border.
    let g = geo();
    let (mut checked, mut thin) = (0, 0);
    let mut wrong = Vec::new();
    for i in 0..g.atlas.regions.len() {
        for (part, c, inradius) in triangles(g, i) {
            if inradius < 1.0 {
                thin += 1;
                continue;
            }
            checked += 1;
            let hit = g.region_at(c);
            if hit != Some(i) {
                wrong.push(format!(
                    "{} part {part} at {c:?}: {:?}",
                    g.atlas.regions[i].key,
                    hit.map(|h| g.atlas.regions[h].key.clone())
                ));
            }
        }
    }
    println!("triangles checked {checked}, thinner than a metre {thin}");
    assert!(checked > 100_000, "{checked}");
    assert!(
        wrong.is_empty(),
        "{} wrong: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(12)]
    );
}

#[test]
fn hover_and_click_name_every_part() {
    // Through the map pane: the pointer over one interior point of every part of every region
    // paints a hover card naming that region, and a click on each riding and on every detached
    // part of an English county selects its key.
    let store = demo_store();
    let mut h = map_harness(store, 1000.0);
    h.state_mut().cursor = Some(51);
    h.run();
    // The view, from the two label points farthest apart east to west.
    let f = h.state().map.frame().clone();
    let xs: Vec<f64> = atlas().regions.iter().map(|r| r.label[0]).collect();
    let a = (0..xs.len())
        .min_by(|&i, &j| xs[i].total_cmp(&xs[j]))
        .expect("a region");
    let b = (0..xs.len())
        .max_by(|&i, &j| xs[i].total_cmp(&xs[j]))
        .expect("a region");
    let (ga, gb) = (atlas().regions[a].label, atlas().regions[b].label);
    let (sa, sb) = (f.regions[a].label, f.regions[b].label);
    let scale = f64::from(sb.x - sa.x) / (gb[0] - ga[0]);
    let to_screen = |p: [f64; 2]| {
        egui::pos2(
            sa.x + ((p[0] - ga[0]) * scale) as f32,
            sa.y - ((p[1] - ga[1]) * scale) as f32,
        )
    };
    // One interior point a part: the centroid of its triangle of greatest inradius.
    let mut points: Vec<(usize, usize, egui::Pos2)> = Vec::new();
    for i in 0..atlas().regions.len() {
        let mut best: BTreeMap<usize, ([f64; 2], f64)> = BTreeMap::new();
        for (part, c, inradius) in triangles(geo(), i) {
            let e = best.entry(part).or_insert((c, inradius));
            if inradius > e.1 {
                *e = (c, inradius);
            }
        }
        for (part, (c, inradius)) in best {
            if inradius >= 1.0 {
                points.push((i, part, to_screen(c)));
            }
        }
    }
    assert!(points.len() > 1_100, "{} parts", points.len());
    let mut wrong = Vec::new();
    for &(i, part, s) in &points {
        let r = &atlas().regions[i];
        h.hover_at(s);
        h.step();
        let card = h.state().map.frame().hover.clone();
        let want = format!("{} ({}, ", r.name, r.key);
        if !card.as_ref().is_some_and(|c| c[0].starts_with(&want)) {
            wrong.push(format!(
                "{} part {part} at {s:?}: {:?}",
                r.key,
                card.map(|c| c[0].clone())
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(12)]
    );
    let mut clicked = 0;
    for &(i, part, s) in &points {
        let r = &atlas().regions[i];
        let riding = ["county.ery", "county.nry", "county.wry"].contains(&r.key.as_str());
        let detached = part > 0 && r.country == rustyecon_worldgen::atlas::Country::England;
        if !(riding && part == 0) && !detached {
            continue;
        }
        h.state_mut().out.clear();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton {
                pos: s,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        h.step();
        h.step();
        let got: Vec<String> = h
            .state()
            .out
            .iter()
            .filter_map(|o| match o {
                Intent::Select(Some(Entity::Node(k))) => Some(k.as_str().to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(got, [r.key.as_str()], "{} part {part}", r.key);
        clicked += 1;
        // Away, so the next press is not a double click.
        h.hover_at(egui::pos2(5.0, 5.0));
        h.step();
        std::thread::sleep(Duration::from_millis(5));
    }
    println!("hovered {} parts, clicked {clicked}", points.len());
    assert!(clicked > 200, "{clicked} parts clicked");
}

#[test]
fn the_credit_is_painted_clear_of_the_legend() {
    // data/atlas/ATTRIBUTION (ODbL 1.0; ruling 7): every map drawn from the atlas shows its
    // credit. It is painted on every frame with a map, inside the canvas and clear of the
    // legend: in the lower right corner on a wide pane, above the legend on a narrow one.
    let store = demo_store();
    for width in [1600.0, 1100.0, 900.0, 700.0] {
        let mut h = map_harness_sized(store, egui::vec2(width, 900.0));
        h.state_mut().cursor = Some(51);
        h.run();
        let f = h.state().map.frame().clone();
        assert_eq!(f.credit, rustyecon_worldgen::atlas::CREDIT, "at {width}");
        let c = f.credit_rect.expect("the credit was painted");
        let l = f.legend.rect.expect("the legend was drawn");
        assert!(
            !c.intersects(l),
            "at {width}: the credit {c:?} over the legend {l:?}"
        );
        assert!(
            c.min.x >= 0.0 && c.min.y >= 0.0 && c.max.y <= 900.0,
            "at {width}: {c:?}"
        );
    }
}
