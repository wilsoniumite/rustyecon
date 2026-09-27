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
//!   the map, every lens is switched to by its hotkeys, a year runs, a county's hover card
//!   names its value, unit, run and tick, and a click selects it for the inspector.
//! - `map_mesh_frame_time_is_recorded` (ignored): a frame's fill mesh and thinned borders,
//!   headless, median of 60.

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
    let mut map = MapState::default();
    map.ready().expect("the atlas and the lenses read");
    Harness::builder()
        .with_size(egui::vec2(1600.0, height))
        .build_ui_state(
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
