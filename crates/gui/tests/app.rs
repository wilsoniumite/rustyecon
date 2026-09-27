//! The app headless (docs/GUI.md §8.1): G0's kittest scripts, egui_kittest with no GPU. Each
//! drives the whole app, a worker thread per run, through its panels as a user would: keys,
//! clicks and typed text, found by their accessible labels, and checks what the frames paint.
//!
//! - `rustyecon-gui tapes/gate.ron` opens paused at tick 0 with every price plotted, and one
//!   key press (Space) gives a live price plot.
//! - The gate script: set the speed cap, run, pause, step, step a year, lift the cap, run until
//!   a date; select (town, bread) and see its price and unit; select an actor and see its
//!   snapshot; the timeline's cursor; follow the registry's link to a param and see the basis
//!   its value was copied with; the log and its breakpoint on error.
//! - The appb script: open, run, pause, step, select (home, good) and see its inspector; the
//!   registry and the log draw.
//! - The theft script: a tape whose event burns what its holder lacks paints `Poisoned`, the
//!   ledger line, the same line by key and the last good tick, and takes no more ticks.
//! - A session made by another tape still gives this tape's prices a live plot.
//! - `every_drawn_vertex_is_recorded`: every vertex the plots lend egui is a recorded (tick,
//!   value) of the store, no gap in the record is bridged, and each line keeps its extremes;
//!   and every segment lent is painted.
//! - The editor's form (G0.2): an empty note, a malformed key and a malformed date are
//!   refused, and so are a key the tree has and a ledger tolerance; an act that does not read
//!   puts the parser's line and column on the raw pane.
//! - The branch script (G0.2): Apply makes a branch that resumes from the ring; compare shows
//!   the first differing hash, the tape diff and the lineage; export writes a CSV and a
//!   manifest that say "experiment", and the lineage; "Save tape as" writes the tape and its
//!   lineage.

mod common;

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use rustyecon_engine::prelude::*;
use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::edit::export::GuiManifest;
use rustyecon_gui::edit::{lineage_path, Lineage};
use rustyecon_gui::model::{Intent, Model};
use rustyecon_gui::platform::Files;
use rustyecon_gui::run::{
    At, Entity, LedgerCheck, Measure, Origin, PauseReason, RunId, RunStatus, Series, SeriesKey,
    Store,
};
use rustyecon_gui::ui::fmt;
use rustyecon_gui::ui::plots::{DrawnLine, PALETTE};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

fn key(s: &str) -> Key {
    Key::new(s).expect("a key")
}

/// The path of a tape of `tapes/`.
fn tape_path(tape: &str) -> String {
    format!("{}/../../tapes/{tape}.ron", env!("CARGO_MANIFEST_DIR"))
}

/// The app on the tape file `path`, in a 1600 × 1000 window, with its session in `files`.
fn harness_on<'a>(path: String, files: Option<Files>) -> Harness<'a, GuiApp> {
    Harness::builder()
        .with_size(egui::vec2(1600.0, 1000.0))
        .build_eframe(move |cc| {
            GuiApp::new(
                &cc.egui_ctx,
                Launch {
                    tape: Some(path),
                    files,
                    smoke: None,
                },
            )
        })
}

/// The app on a tape of `tapes/`, in a 1600 × 1000 window, with no session.
fn harness<'a>(tape: &str) -> Harness<'a, GuiApp> {
    harness_on(tape_path(tape), None)
}

/// The six prices of the gate world.
fn gate_prices() -> BTreeSet<SeriesKey> {
    ["town", "village"]
        .iter()
        .flat_map(|n| ["bread", "fuel", "grain"].map(|g| price(n, g)))
        .collect()
}

/// Step frames until `done` holds of the app.
fn step_until(h: &mut Harness<'_, GuiApp>, what: &str, done: impl Fn(&GuiApp) -> bool) {
    let t0 = Instant::now();
    while !done(h.state()) {
        h.step();
        assert!(
            t0.elapsed() < Duration::from_secs(60),
            "the app did not get there: {what}"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn store(app: &GuiApp) -> &Store {
    &app.model().focused().expect("a focused run").store
}

fn status(m: &Model) -> Option<RunStatus> {
    m.focused().map(|r| r.store.status())
}

fn paused(app: &GuiApp, why: PauseReason) -> bool {
    matches!(status(app.model()), Some(RunStatus::Paused { why: Some(w), .. }) if w == why)
}

fn price(node: &str, good: &str) -> SeriesKey {
    SeriesKey {
        measure: Measure::Price,
        at: At::Market {
            node: key(node),
            good: key(good),
        },
    }
}

/// Click the node labelled `label`, then take the pointer away as a hand leaves the mouse.
fn click(h: &mut Harness<'_, GuiApp>, label: &str) {
    h.get_by_label(label).click();
    h.step();
    h.remove_cursor();
    h.step();
}

/// Every text the last frame painted, in paint order.
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

/// The texts the last frame painted turned on their side: the plots' y-axis labels, which
/// egui_plot paints rotated a quarter turn. The outliner's rows paint the same units level.
fn axis_labels(h: &Harness<'_, GuiApp>) -> Vec<String> {
    fn walk(s: &egui::Shape, out: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) if t.angle != 0.0 => out.push(t.galley.text().to_string()),
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

/// Step until the plots' y axes name exactly `units`, one per panel in stack order, 20 frames
/// at most.
fn axes_name(h: &mut Harness<'_, GuiApp>, units: &[&str]) {
    for _ in 0..20 {
        if axis_labels(h) == units {
            return;
        }
        h.step();
    }
    assert_eq!(axis_labels(h), units, "the y axes' units");
}

/// Step frames until the frame paints a text that `hit` accepts, 20 frames at most. A panel's
/// text can take a frame or two: egui lays a new grid out invisibly on its first frame, and a
/// tab's scrolled rows a frame after the scroll moves. The scripts check what is painted, which
/// is what is on screen, and click through the accessible nodes.
fn paints(h: &mut Harness<'_, GuiApp>, what: &str, hit: impl Fn(&str) -> bool) {
    for _ in 0..20 {
        if painted(h).iter().any(|t| hit(t)) {
            return;
        }
        h.step();
    }
    let tail = what.split_once(' ').map_or(what, |x| x.1);
    let near: Vec<String> = painted(h)
        .into_iter()
        .filter(|t| t.contains(tail))
        .take(12)
        .collect();
    let m = h.state().model();
    let at = (
        m.cursor(),
        m.focused().map(|r| (r.store.tick(), r.store.status())),
    );
    panic!("the frame paints no {what:?}; texts with {tail:?}: {near:?}; cursor and run {at:?}");
}

/// Step until the frame paints `text`.
fn shows(h: &mut Harness<'_, GuiApp>, text: &str) {
    paints(h, text, |t| t == text);
}

/// Step until the frame paints a text containing `part`.
fn shows_part(h: &mut Harness<'_, GuiApp>, part: &str) {
    paints(h, part, |t| t.contains(part));
}

/// Step until an interactive node labelled `label` exists, 20 frames at most: a row laid out
/// below a table's fold is not painted, but its link can be clicked.
fn has(h: &mut Harness<'_, GuiApp>, label: &str) {
    for _ in 0..20 {
        if h.query_all_by_label(label).next().is_some() {
            return;
        }
        h.step();
    }
    panic!("no node labelled {label:?}");
}

fn vertices(lines: &[DrawnLine]) -> usize {
    lines.iter().flat_map(|l| &l.segments).map(Vec::len).sum()
}

/// Every open path the last frame painted, with its stroke's colour. egui_plot paints each
/// `Line` of two or more points as one such path, and a `VLine` as one of two.
fn painted_paths(h: &Harness<'_, GuiApp>) -> Vec<(Vec<egui::Pos2>, Option<egui::Color32>)> {
    fn walk(s: &egui::Shape, out: &mut Vec<(Vec<egui::Pos2>, Option<egui::Color32>)>) {
        match s {
            egui::Shape::Path(p) if !p.closed => {
                let colour = match &p.stroke.color {
                    egui::epaint::ColorMode::Solid(c) => Some(*c),
                    egui::epaint::ColorMode::UV(_) => None,
                };
                out.push((p.points.clone(), colour));
            }
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

/// A path the plots painted: a line, stroked in the plots' palette, or a panel's cursor, a grey
/// vertical line of two points.
enum PlotPath {
    Line(Vec<egui::Pos2>),
    Cursor(f64),
}

/// The plots' paths as the last frame painted them, in paint order. egui_plot paints each
/// `Line` of two or more points as one open path, in the order the lines were handed to it,
/// and then the panel's cursor, so each panel's paths end with its cursor.
fn plot_paths(h: &Harness<'_, GuiApp>) -> Vec<PlotPath> {
    painted_paths(h)
        .into_iter()
        .filter_map(|(p, c)| {
            let c = c?;
            if PALETTE.contains(&c) {
                Some(PlotPath::Line(p))
            } else if c == egui::Color32::GRAY && p.len() == 2 && p[0].x == p[1].x {
                Some(PlotPath::Cursor(f64::from(p[0].x)))
            } else {
                None
            }
        })
        .collect()
}

/// The plots' cursor lines the last frame painted, by their x on screen.
fn cursor_lines(h: &Harness<'_, GuiApp>) -> Vec<f64> {
    plot_paths(h)
        .into_iter()
        .filter_map(|p| match p {
            PlotPath::Cursor(x) => Some(x),
            PlotPath::Line(_) => None,
        })
        .collect()
}

/// How far a painted point may sit from where its value puts it, in points: screen positions
/// are f32.
const ON_SCREEN: f64 = 0.01;

/// What the last frame lent egui is what it painted. Each segment of two or more vertices is
/// painted as one path of as many points, in the order it was lent, and nothing else is
/// painted in the plots' palette. Within a panel every painted point sits at one affine map of
/// its tick, and the panel's grey cursor line at the same map of the cursor's tick, so a vertex
/// lent at another tick than the one recorded fails here; within a line, y is one affine map
/// of the value.
fn lent_is_painted(h: &Harness<'_, GuiApp>, lines: &[DrawnLine]) {
    let mut lent = lines.iter().flat_map(|l| {
        l.segments
            .iter()
            .filter(|s| s.len() >= 2)
            .map(move |s| (&l.key, s))
    });
    let store = store(h.state());
    let cursor = store
        .report_at(rustyecon_gui::ui::cursor(h.state().model()))
        .expect("a tick ran") as f64;
    // Each panel: its lines, then its cursor.
    let mut panel: Vec<Painted<'_>> = Vec::new();
    let mut panels = 0;
    for p in plot_paths(h) {
        match p {
            PlotPath::Line(path) => {
                let (key, seg) = lent.next().expect("a painted line was lent");
                assert_eq!(path.len(), seg.len(), "{key}: painted as lent");
                panel.push((key, seg, path));
            }
            PlotPath::Cursor(x) => {
                check_panel(&panel, cursor, x);
                panel.clear();
                panels += 1;
            }
        }
    }
    assert!(panel.is_empty(), "every panel ends with its cursor");
    assert!(panels > 0, "the plots draw the cursor");
    let left: Vec<_> = lent.map(|(k, s)| (k.to_string(), s.len())).collect();
    assert!(left.is_empty(), "lent and not painted: {left:?}");
}

/// A segment lent, with its series, and the path painted for it.
type Painted<'a> = (&'a SeriesKey, &'a Vec<[f64; 2]>, Vec<egui::Pos2>);

/// One panel's painted lines against what was lent, and its cursor at `x`.
fn check_panel(panel: &[Painted<'_>], cursor: f64, x: f64) {
    let span = |s: &Vec<[f64; 2]>| s[s.len() - 1][0] - s[0][0];
    let Some((_, seg, path)) = panel.iter().max_by(|a, b| span(a.1).total_cmp(&span(b.1))) else {
        return;
    };
    let (t0, t1) = (seg[0][0], seg[seg.len() - 1][0]);
    assert!(t1 > t0, "the widest segment spans ticks");
    let (x0, x1) = (f64::from(path[0].x), f64::from(path[path.len() - 1].x));
    let b = (x1 - x0) / (t1 - t0);
    let x_of = |t: f64| x0 + b * (t - t0);
    for (key, seg, path) in panel {
        for (v, p) in seg.iter().zip(path) {
            let off = f64::from(p.x) - x_of(v[0]);
            assert!(
                off.abs() < ON_SCREEN,
                "{key}: tick {} painted {off} off",
                v[0]
            );
        }
        // y: affine in the value, within the line.
        let lo = seg
            .iter()
            .zip(path)
            .min_by(|a, b| a.0[1].total_cmp(&b.0[1]));
        let hi = seg
            .iter()
            .zip(path)
            .max_by(|a, b| a.0[1].total_cmp(&b.0[1]));
        if let (Some((vl, pl)), Some((vh, ph))) = (lo, hi) {
            if vh[1] > vl[1] {
                let c = f64::from(ph.y - pl.y) / (vh[1] - vl[1]);
                for (v, p) in seg.iter().zip(path.iter()) {
                    let off = f64::from(p.y) - (f64::from(pl.y) + c * (v[1] - vl[1]));
                    assert!(
                        off.abs() < ON_SCREEN,
                        "{key}: value {} painted {off} off",
                        v[1]
                    );
                }
            }
        }
    }
    let off = x - x_of(cursor);
    assert!(
        off.abs() < ON_SCREEN,
        "the cursor at tick {cursor} is painted {off} off"
    );
}

/// Step until the frame paints `text` at least `n` times, 20 frames at most.
fn shows_times(h: &mut Harness<'_, GuiApp>, text: &str, n: usize) {
    for _ in 0..20 {
        if painted(h).iter().filter(|t| *t == text).count() >= n {
            return;
        }
        h.step();
    }
    let got = painted(h).iter().filter(|t| *t == text).count();
    panic!("the frame paints {text:?} {got} times, not {n}");
}

#[test]
fn one_key_press_gives_a_live_price_plot() {
    // docs/GUI.md §4 and §9: `rustyecon-gui tapes/gate.ron` opens paused at tick 0 with every
    // price plotted, and one key press gives a live price plot.
    let mut h = harness("gate");
    step_until(&mut h, "paused at tick 0", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    h.step();
    let plots = h.state().model().session.plots.clone();
    assert_eq!(
        plots.iter().cloned().collect::<BTreeSet<_>>(),
        gate_prices(),
        "every price"
    );
    // No tick has run, so nothing is lent to egui yet.
    assert!(h.state().drawn().is_empty(), "no tick has run");
    // Three panels, one per unit, each naming it.
    let vm = rustyecon_gui::vm::plots::build(store(h.state()), &plots, None).unwrap();
    let units: Vec<&str> = vm.panels.iter().map(|p| p.unit.as_str()).collect();
    assert_eq!(units, ["coin per bread", "coin per fuel", "coin per grain"]);
    h.key_press(egui::Key::Space);
    live_prices(&mut h);
}

/// After Space: a line for each of the gate's prices is lent to egui and painted, and it grows
/// while the run runs.
fn live_prices(h: &mut Harness<'_, GuiApp>) {
    step_until(h, "every price drawn", |a| {
        let d = a.drawn();
        d.len() == 6 && d.iter().all(|l| l.segments.iter().any(|s| s.len() > 20))
    });
    let drawn = h.state().drawn();
    let keys: BTreeSet<SeriesKey> = drawn.iter().map(|l| l.key.clone()).collect();
    assert_eq!(keys, gate_prices(), "a line for each price");
    lent_is_painted(h, &drawn);
    // Live: it grows while the run runs.
    let before = vertices(&drawn);
    let tick = store(h.state()).tick();
    step_until(h, "the plot grows", |a| {
        store(a).tick() > tick + 50 && vertices(&a.drawn()) > before
    });
    assert!(matches!(
        status(h.state().model()),
        Some(RunStatus::Running { .. })
    ));
}

#[test]
fn a_second_tapes_session_still_plots_every_price() {
    // docs/GUI.md §4 and §9 again, for a user whose session.ron another tape wrote: its plots
    // name the Appendix B world's prices, none of the gate's. The gate opens with its own
    // prices plotted, and Space gives a live price plot; appb's keys stay out of the stack.
    let dir = tempfile::tempdir().expect("a temporary directory");
    {
        let mut h = harness_on(tape_path("appb"), Some(Files::at(dir.path())));
        step_until(&mut h, "appb loaded", |a| {
            status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
        });
        assert_eq!(h.state().model().session.plots.len(), 4);
    }
    let session = std::fs::read_to_string(dir.path().join("session.ron")).expect("saved");
    assert!(
        session.contains("\"home\""),
        "appb's plots are saved: {session}"
    );
    let mut h = harness_on(tape_path("gate"), Some(Files::at(dir.path())));
    step_until(&mut h, "the gate loaded", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
            && store(a).world().is_some_and(|w| w.name == "gate")
    });
    let plots: BTreeSet<SeriesKey> = h.state().model().session.plots.iter().cloned().collect();
    assert!(plots.is_superset(&gate_prices()), "{plots:?}");
    h.key_press(egui::Key::Space);
    live_prices(&mut h);
    axes_name(
        &mut h,
        &["coin per bread", "coin per fuel", "coin per grain"],
    );
    assert!(
        !painted(&h).iter().any(|t| t.starts_with("currency per")),
        "no unit is made up for another world's key"
    );
    shows_part(
        &mut h,
        "plotted, but not in this run's world: price at home/",
    );
}

#[test]
fn the_theft_script_shows_a_failed_run() {
    // docs/GUI.md §3.3: a failed run shows Poisoned, its ledger line and its last good tick
    // (E5, R2). The gate with a theft, opened from a file: on 1751-06-01, tick 73, an event
    // burns 1e9 coin the workers do not hold. Space runs it into the failure; the toolbar
    // paints the engine's line, the same line by key, and the last good tick; Space and `.`
    // then take no tick.
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("theft.ron");
    std::fs::write(&path, common::gift_then_theft()).expect("the tape is written");
    let mut h = harness_on(path.display().to_string(), None);
    step_until(&mut h, "paused at tick 0", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    h.key_press(egui::Key::Space);
    step_until(&mut h, "poisoned", |a| {
        matches!(
            status(a.model()),
            Some(RunStatus::Poisoned {
                tick: 73,
                phase: Phase::Events
            })
        )
    });
    let store_now = store(h.state());
    let line = store_now.failure().expect("the failure").error.to_string();
    assert!(line.starts_with("tick 73, phase 0 (events): shortfall at tick 73"));
    let keys = rustyecon_gui::vm::toolbar::build(
        store_now,
        h.state().model().focused().unwrap().origin,
        LedgerCheck::NoParent,
    )
    .health
    .ledger_keys;
    assert_eq!(keys.len(), 2, "{keys:?}");
    paints(&mut h, "Poisoned", |t| t.starts_with("Poisoned · "));
    shows(&mut h, &line);
    for k in &keys {
        shows(&mut h, k);
    }
    shows_part(&mut h, "by key: workers was asked for 1e9 coin");
    shows(&mut h, "last good tick 72");
    // The log says it too, in the cli's words.
    click(&mut h, "Log");
    shows(&mut h, &format!("run 0 [73] run error: {line}"));
    shows(
        &mut h,
        "run 0 [72] the last good tick is 72 (state tick 73)",
    );
    // A poisoned run takes no more ticks.
    h.key_press(egui::Key::Space);
    h.key_press(egui::Key::Period);
    for _ in 0..10 {
        h.step();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(store(h.state()).tick(), 73);
    assert_eq!(store(h.state()).hashes().len(), 73);
}

#[test]
fn the_gate_script_runs_pauses_steps_and_inspects() {
    // G0's kittest script on the gate world (docs/GUI.md §8.1): open, run, pause, step; select
    // (town, bread) and see its inspector.
    let mut h = harness("gate");
    step_until(&mut h, "paused at tick 0", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    // Cap the speed at a model year a second, so the pause lands in the run's first years.
    click(&mut h, "speed cap");
    click(&mut h, "1 year/s");
    step_until(&mut h, "capped", |a| a.model().session.speed == Some(52));
    // Run, and pause.
    h.key_press(egui::Key::Space);
    step_until(&mut h, "running past tick 10", |a| store(a).tick() > 10);
    h.key_press(egui::Key::Space);
    step_until(&mut h, "paused", |a| paused(a, PauseReason::Asked));
    // Step one tick with `.`, then a year with the toolbar's button.
    let at = store(h.state()).tick();
    h.key_press(egui::Key::Period);
    step_until(&mut h, "stepped", |a| paused(a, PauseReason::Stepped));
    assert_eq!(store(h.state()).tick(), at + 1);
    click(&mut h, "Step a year");
    step_until(&mut h, "a year on", |a| {
        store(a).tick() > at + 1 && paused(a, PauseReason::Stepped)
    });
    assert_eq!(store(h.state()).tick(), at + 53);
    // Lift the cap, and run until a date: through the tick 1760-03-01 falls in, where mine.cut
    // fires.
    click(&mut h, "speed cap");
    click(&mut h, "no cap");
    step_until(&mut h, "uncapped", |a| a.model().session.speed.is_none());
    let clock = store(h.state()).world().unwrap().clock;
    let cut = clock.tick_of(Date::parse("1760-03-01").unwrap()).unwrap();
    assert!(
        at + 53 < cut,
        "the pause landed at tick {at}, before the cut's year"
    );
    h.get_by_label("until").focus();
    h.step();
    h.get_by_label("until").type_text("1760-03-01");
    h.step();
    click(&mut h, "Run until");
    step_until(&mut h, "reached", |a| {
        matches!(
            status(a.model()),
            Some(RunStatus::Paused {
                why: Some(PauseReason::Reached(_)),
                ..
            })
        )
    });
    assert!(paused(h.state(), PauseReason::Reached(cut + 1)));
    assert!(store(h.state())
        .events()
        .iter()
        .any(|(t, e)| *t == cut && e.key.as_str() == "mine.cut"));
    // U3: the identity chip names the build (its commit and dirty flag), the world_id and the
    // tape_hash, as the cli's run line and the certified manifest do.
    let b = rustyecon_gui::build();
    let commit: String = b.commit.chars().take(10).collect();
    let state = if b.dirty { "dirty" } else { "clean" };
    shows(
        &mut h,
        &format!(
            "gate · run · {commit} {state} · world 0x43628a8e0fd5f695 · tape 0x54066d053474846b"
        ),
    );
    // Units always shown: each of the three panels names its unit on its y axis and the years
    // on its x axis, and draws the cursor.
    axes_name(
        &mut h,
        &["coin per bread", "coin per fuel", "coin per grain"],
    );
    for year in ["1750", "1755", "1760"] {
        shows_times(&mut h, year, 3);
    }
    assert_eq!(cursor_lines(&h).len(), 3, "a cursor line in each panel");
    lent_is_painted(&h, &h.state().drawn());
    // Select (town, bread) in the outliner and see its inspector: the market's price at the
    // cursor, with its unit.
    click(&mut h, "town/bread");
    step_until(&mut h, "selected", |a| {
        a.model().selection()
            == Some(&Entity::Market {
                node: key("town"),
                good: key("bread"),
            })
    });
    shows(&mut h, "Market town/bread");
    let p = store(h.state())
        .series(&price("town", "bread"))
        .and_then(|s| s.at(cut))
        .expect("the price at the cut's tick");
    shows(&mut h, &format!("{} coin per bread", fmt(p)));
    shows(&mut h, &format!("tick {cut} (1760-02-27)"));
    // Plot from the inspector: the market's supply gets a line, in a panel of its own unit.
    let supply = SeriesKey {
        measure: Measure::Supply,
        at: At::Market {
            node: key("town"),
            good: key("bread"),
        },
    };
    plot_by_click(&mut h, "plot supply at town/bread", &supply);
    axes_name(
        &mut h,
        &[
            "coin per bread",
            "coin per fuel",
            "coin per grain",
            "bread per tick",
        ],
    );
    // Select the mill: the model asks the paused run for a snapshot of the state the cursor's
    // tick left, and the inspector reads the mill's lots and state from it.
    click(&mut h, "mill");
    step_until(&mut h, "the mill's snapshot", |a| {
        store(a).snapshot(cut + 1).is_some()
    });
    shows(&mut h, "Actor mill");
    shows_part(&mut h, &format!("(snapshot of state tick {})", cut + 1));
    // The timeline follows the run: its cursor is the cut's tick, live.
    shows(&mut h, &format!("cursor 1760-02-27 (tick {cut}), live"));
    // The registry lists mine.capacity. Its link selects the param, and the inspector names
    // where the current value came from: mine.capacity.cut, by mine.cut, on its date
    // (FiredEvent.source), with that param's basis.
    click(&mut h, "Registry");
    has(&mut h, "mine.capacity");
    h.get_by_label("mine.capacity").click_accesskit();
    step_until(&mut h, "the param selected", |a| {
        a.model().selection() == Some(&Entity::Param(key("mine.capacity")))
    });
    click(&mut h, "Inspector");
    shows(&mut h, "Param mine.capacity");
    shows_part(
        &mut h,
        &format!(
            "copied from mine.capacity.cut by mine.cut on 1760-03-01 (tick {cut}): \
             Assumed(\"gate world: half\")"
        ),
    );
    // The log, with the breakpoint on error: on in a new session, and a click turns it off. The
    // log opens at its latest line.
    click(&mut h, "Log");
    shows_part(
        &mut h,
        &format!("[{cut}] snapshot of state tick {}", cut + 1),
    );
    click(&mut h, "Break on error");
    step_until(&mut h, "the breakpoint off", |a| {
        a.model().session.breakpoints.is_empty()
    });
    // Plot from the outliner: the mill's row plots its coin.
    let coin = SeriesKey {
        measure: Measure::Held,
        at: At::Holding {
            holder: rustyecon_gui::run::HolderKey::Actor(key("mill")),
            good: key("coin"),
        },
    };
    plot_by_click(&mut h, "plot held of coin by mill", &coin);
}

/// Click the plot button whose accessible name is `label`, and see `key`'s line lent to egui
/// and painted.
fn plot_by_click(h: &mut Harness<'_, GuiApp>, label: &str, key: &SeriesKey) {
    has(h, label);
    h.get_by_label(label).click_accesskit();
    step_until(h, label, |a| {
        a.model().session.plots.contains(key)
            && a.drawn()
                .iter()
                .any(|l| l.key == *key && l.segments.iter().any(|s| s.len() > 20))
    });
    h.step();
    lent_is_painted(h, &h.state().drawn());
}

#[test]
fn the_appb_script_runs_pauses_steps_and_inspects() {
    // G0's kittest script on the Appendix B world: open, run, pause, step; select (home, good)
    // and see its inspector.
    let mut h = harness("appb");
    step_until(&mut h, "paused at tick 0", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    assert_eq!(h.state().model().session.plots.len(), 4, "every price");
    h.key_press(egui::Key::Space);
    step_until(&mut h, "running past tick 200", |a| store(a).tick() > 200);
    h.key_press(egui::Key::Space);
    step_until(&mut h, "paused", |a| paused(a, PauseReason::Asked));
    let at = store(h.state()).tick();
    h.key_press(egui::Key::Period);
    step_until(&mut h, "stepped", |a| paused(a, PauseReason::Stepped));
    assert_eq!(store(h.state()).tick(), at + 1);
    click(&mut h, "home/good");
    step_until(&mut h, "selected", |a| a.model().selection().is_some());
    shows(&mut h, "Market home/good");
    let p = store(h.state())
        .series(&price("home", "good"))
        .and_then(|s| s.at(at))
        .expect("the price at the cursor");
    shows(&mut h, &format!("{} coin per good", fmt(p)));
    assert!(vertices(&h.state().drawn()) > 0);
    // The registry and the log draw for this world too.
    click(&mut h, "Registry");
    has(&mut h, "spend.workers");
    click(&mut h, "Log");
    shows(&mut h, "Break on error");
}

/// Whether a series has a gap between its first and last points.
fn gapped(s: &Series) -> bool {
    s.ticks().windows(2).any(|w| w[1] != w[0] + 1)
}

/// Check every line drawn against the store; returns the vertices drawn.
fn check_drawn(store: &Store, lines: &[DrawnLine]) -> usize {
    let mut n = 0;
    for l in lines {
        let s = store.series(&l.key).expect("a drawn series is recorded");
        let mut all: Vec<[f64; 2]> = Vec::new();
        for seg in &l.segments {
            assert!(!seg.is_empty(), "{}: an empty segment", l.key);
            for &[x, y] in seg {
                assert!(x >= 0.0 && x.fract() == 0.0, "{}: x {x} is no tick", l.key);
                let t = x as u64;
                assert_eq!(
                    s.at(t).map(f64::to_bits),
                    Some(y.to_bits()),
                    "{}: the vertex ({t}, {y}) is not recorded",
                    l.key
                );
            }
            assert!(seg.windows(2).all(|w| w[0][0] < w[1][0]), "{}", l.key);
            // No gap is bridged: every tick between a segment's ends is recorded.
            let (a, b) = (seg[0][0] as u64, seg[seg.len() - 1][0] as u64);
            let (i, j) = s.range(a, b + 1);
            assert_eq!(
                (j - i) as u64,
                b - a + 1,
                "{}: {a}..={b} bridges a gap",
                l.key
            );
            all.extend(seg);
        }
        // One segment per unbroken stretch of the record.
        let stretches = 1 + s.ticks().windows(2).filter(|w| w[1] != w[0] + 1).count();
        assert_eq!(l.segments.len(), stretches, "{}", l.key);
        // The series' least and greatest values are drawn.
        let min = s.values().iter().copied().fold(f64::INFINITY, f64::min);
        let max = s.values().iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(all.iter().any(|p| p[1] == min), "{}: its minimum", l.key);
        assert!(all.iter().any(|p| p[1] == max), "{}: its maximum", l.key);
        n += all.len();
    }
    n
}

#[test]
fn every_drawn_vertex_is_recorded() {
    // docs/GUI.md §3.4: the plot cache thins each series and lends egui the points it keeps.
    // Every vertex egui receives is a recorded (tick, value), bit for bit; a gap in the record
    // splits the line; and each line keeps its least and greatest values. Checked on the gate
    // run to 2,080 with every price, a param that steps and a series with gaps, at two window
    // widths, the narrower thinning harder. The vertices are read where they are lent, from
    // the points each `Line` is made of, and each segment lent is painted as one path.
    let mut h = harness("gate");
    step_until(&mut h, "loaded", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    h.state_mut().act(Intent::Run { until: Some(2080) });
    step_until(&mut h, "the run's end", |a| {
        paused(a, PauseReason::Reached(2080))
    });
    let with_gaps = store(h.state())
        .catalogue()
        .iter()
        .find(|k| store(h.state()).series(k).is_some_and(gapped))
        .cloned()
        .expect("some series of the gate has a gap");
    let capacity = SeriesKey {
        measure: Measure::Param,
        at: At::Param(key("mine.capacity")),
    };
    h.state_mut().act(Intent::Plot(with_gaps.clone()));
    h.state_mut().act(Intent::Plot(capacity.clone()));
    h.step();
    h.step();
    let plotted: BTreeSet<SeriesKey> = h.state().model().session.plots.iter().cloned().collect();
    assert_eq!(plotted.len(), 8);
    let drawn = h.state().drawn();
    let keys: BTreeSet<SeriesKey> = drawn.iter().map(|l| l.key.clone()).collect();
    assert_eq!(keys, plotted, "every plotted series is drawn");
    let wide = check_drawn(store(h.state()), &drawn);
    lent_is_painted(&h, &drawn);
    let gap_line = drawn.iter().find(|l| l.key == with_gaps).unwrap();
    assert!(
        gap_line.segments.len() >= 2,
        "{with_gaps} is split at its gaps"
    );
    // A narrower window thins harder, and still draws only recorded points.
    h.set_size(egui::vec2(640.0, 1000.0));
    h.step();
    h.step();
    let drawn = h.state().drawn();
    let narrow = check_drawn(store(h.state()), &drawn);
    lent_is_painted(&h, &drawn);
    assert!(
        narrow < wide,
        "{narrow} vertices at 640 wide, {wide} at 1600"
    );
    let bread = drawn
        .iter()
        .find(|l| l.key == price("town", "bread"))
        .unwrap();
    let n: usize = bread.segments.iter().map(Vec::len).sum();
    assert!(n < 2080, "the price of bread is thinned: {n} vertices");
}

/// The act that restores the mine's capacity from its base value.
const RESTORE: &str = "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")";

/// Put `text` in the field labelled `label`, in place of what it holds: focus it, select all,
/// type.
fn fill(h: &mut Harness<'_, GuiApp>, label: &str, text: &str) {
    h.get_by_label(label).focus();
    h.step();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.step();
    h.get_by_label(label).type_text(text);
    h.step();
    assert_eq!(
        h.get_by_label(label).value().as_deref(),
        Some(text),
        "{label}"
    );
}

/// Scroll the node labelled `label` into view, as a hand on the wheel would, so the frame
/// paints it.
fn scroll_to(h: &mut Harness<'_, GuiApp>, label: &str) {
    has(h, label);
    h.get_by_label(label).scroll_to_me();
    h.step();
    h.step();
}

/// Scroll the text labelled `text` into view and see the frame paint it: a pane's lines sit
/// higher or lower as its paths and its window wrap them.
fn see(h: &mut Harness<'_, GuiApp>, text: &str) {
    scroll_to(h, text);
    shows(h, text);
}

/// Press the button labelled `label`, through its accessible node.
fn press(h: &mut Harness<'_, GuiApp>, label: &str) {
    has(h, label);
    h.get_by_label(label).click_accesskit();
    h.step();
    h.step();
}

#[test]
fn the_editor_refuses_an_empty_note_a_malformed_key_and_a_malformed_date() {
    // docs/GUI.md §5.1 item 2, §8.1: the form checks first. An empty note, a malformed key and
    // a malformed date are refused, as are a key a tape of the session has and a ledger
    // tolerance, and each refusal is painted; an act that does not read puts the parser's line
    // and column on the raw pane. Nothing is staged until the form reads.
    let mut h = harness("gate");
    step_until(&mut h, "paused at tick 0", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    click(&mut h, "Editor");
    see(&mut h, "Branch run 0 (gate)");
    let staged = |h: &Harness<'_, GuiApp>| h.state().model().editor().staged.len();
    // An empty note.
    fill(&mut h, "edit key", "gui.1.1");
    fill(&mut h, "edit date", "1765-06-01");
    fill(&mut h, "edit act", RESTORE);
    press(&mut h, "Add edit");
    see(&mut h, "refused: a note is required");
    assert_eq!(staged(&h), 0);
    // A malformed key.
    fill(&mut h, "edit note", "restore the capacity early");
    fill(&mut h, "edit key", "Mine Cut");
    press(&mut h, "Add edit");
    see(
        &mut h,
        "refused: the key \"Mine Cut\" must match [a-z0-9_.-]+",
    );
    assert_eq!(staged(&h), 0);
    // A malformed date, with a minted key.
    press(&mut h, "Mint key");
    assert_eq!(
        h.get_by_label("edit key").value().as_deref(),
        Some("gui.1.1")
    );
    fill(&mut h, "edit date", "1765-13-45");
    press(&mut h, "Add edit");
    see(
        &mut h,
        "refused: the date does not parse: invalid date \"1765-13-45\": expected an existing \
         YYYY-MM-DD",
    );
    assert_eq!(staged(&h), 0);
    // An act that does not read: its line and column land on the raw pane, under the line.
    fill(&mut h, "edit date", "1765-06-01");
    let bad = "SetParam(param: \"mine.capacity\" to: \"mine.capacity.base\")";
    fill(&mut h, "edit act", bad);
    press(&mut h, "Add edit");
    see(
        &mut h,
        "refused: the act does not read at line 1, column 33: Expected comma",
    );
    see(&mut h, &format!("  1 | {bad}"));
    see(&mut h, &format!("    | {}^", " ".repeat(32)));
    see(&mut h, "line 1, column 33: Expected comma");
    assert_eq!(staged(&h), 0);
    // A key a tape of the session has.
    fill(&mut h, "edit act", RESTORE);
    fill(&mut h, "edit key", "mine.cut");
    press(&mut h, "Add edit");
    see(
        &mut h,
        "refused: the key mine.cut is taken by a tape of this session; a new entry needs a new \
         key (mint one)",
    );
    // A ledger tolerance.
    click(&mut h, "edit kind");
    click(&mut h, "SetGenesisParam");
    fill(&mut h, "edit key", "ledger.rel_flow");
    fill(&mut h, "edit value", "1e-10");
    press(&mut h, "Add edit");
    see(
        &mut h,
        "refused: ledger.rel_flow is a ledger tolerance (header.ledger), which is not editable",
    );
    assert_eq!(staged(&h), 0);
    // The form read: staged, and listed.
    click(&mut h, "edit kind");
    click(&mut h, "AddEvent");
    press(&mut h, "Mint key");
    press(&mut h, "Add edit");
    see(&mut h, "Staged edits (1)");
    let line = {
        let e = &h.state().model().editor().staged[0];
        format!("1. {} ({:?})", e.op, e.note)
    };
    assert!(line.starts_with("1. AddEvent gui.1.1 on 1765-06-01: SetParam("));
    see(&mut h, &line);
    assert_eq!(staged(&h), 1);
    assert_eq!(h.state().model().runs().count(), 1, "nothing applied yet");
}

#[test]
fn the_branch_script_applies_compares_exports_and_saves() {
    // docs/GUI.md §5.1, §4, §8.1: run the gate to its end, then set mine.capacity from
    // mine.capacity.base on 1765-06-01 through the editor. Apply makes a branch, which resumes
    // from the ring and whose identity chip says "experiment"; compare shows the first
    // differing hash at the edit's tick, both identities with their origins, the range
    // compared, the tape diff and the lineage; export writes a CSV and a manifest envelope
    // that say "experiment", and the lineage; "Save tape as" writes the tape and its lineage.
    // The saved tape, edited by hand and reopened, paints "ledger changed", and "ledger
    // unchecked" once its base is gone.
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("gate.ron");
    std::fs::write(&path, common::GATE).expect("the tape is written");
    let path = path.display().to_string();
    let mut h = harness_on(path.clone(), None);
    step_until(&mut h, "paused at tick 0", |a| {
        status(a.model()) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    let today = h.state().model().today().expect("today's date");
    fill(&mut h, "until", "2080");
    click(&mut h, "Run until");
    step_until(&mut h, "the gate's end", |a| {
        paused(a, PauseReason::Reached(2080))
    });
    click(&mut h, "Editor");
    press(&mut h, "Mint key");
    fill(&mut h, "edit date", "1765-06-01");
    fill(&mut h, "edit act", RESTORE);
    fill(&mut h, "edit note", "restore the capacity early");
    press(&mut h, "Add edit");
    see(&mut h, "Staged edits (1)");
    press(&mut h, "Apply");
    step_until(&mut h, "the branch loaded", |a| {
        a.model().focus() == Some(RunId(1)) && store(a).run().is_some()
    });
    let jan = common::gate_tick("1765-01-01");
    let edit = common::gate_tick("1765-06-01");
    assert_eq!(store(h.state()).start(), jan, "it resumed from the ring");
    let name = format!("gate [GUI experiment {today}]");
    let b = rustyecon_gui::build();
    let commit: String = b.commit.chars().take(10).collect();
    let tape = h.state().model().focused().unwrap().tape_hash;
    shows(
        &mut h,
        &format!(
            "{name} · experiment · {commit} {} · world 0x43628a8e0fd5f695 · tape {}",
            b.state(),
            certify::Hex(tape)
        ),
    );
    // Run the branch to the end: "until" still says 2080.
    click(&mut h, "Run until");
    step_until(&mut h, "the branch's end", |a| {
        paused(a, PauseReason::Reached(2080))
    });
    // Compare: the first differing hash is the edit's tick; the tape diff and the lineage.
    click(&mut h, "Compare");
    let clock = store(h.state()).world().unwrap().clock;
    let date = clock.date_of(edit).unwrap();
    shows_part(
        &mut h,
        &format!(
            "first differing hash: state tick {}, left by report tick {edit} ({date})",
            edit + 1
        ),
    );
    // Each identity names its origin (U3), and the range compared is shown.
    see(
        &mut h,
        &format!(
            "parent: gate · run · {commit} {} · world 0x43628a8e0fd5f695 · tape \
             0x54066d053474846b · state ticks 0 to 2080",
            b.state()
        ),
    );
    see(
        &mut h,
        &format!(
            "branch: {name} · experiment · {commit} {} · world 0x43628a8e0fd5f695 · tape {} · \
             state ticks {jan} to 2080",
            b.state(),
            certify::Hex(tape)
        ),
    );
    see(&mut h, &format!("compared: state ticks {jan} to 2080"));
    // Each line is scrolled to first: how far down it sits depends on how long the paths are.
    let from = format!("from {path} (tape_hash 0x54066d053474846b)");
    scroll_to(&mut h, &from);
    shows(&mut h, &from);
    let lines = h
        .state()
        .model()
        .focused()
        .unwrap()
        .lineage
        .as_ref()
        .unwrap()
        .lines();
    assert!(lines[1].starts_with(&format!(
        "1. {today}: AddEvent gui.1.1 on 1765-06-01: SetParam("
    )));
    scroll_to(&mut h, &lines[1]);
    shows(&mut h, &lines[1]);
    scroll_to(&mut h, "header name changed");
    shows(&mut h, "header name changed");
    scroll_to(&mut h, "events gui.1.1 added");
    shows(&mut h, "events gui.1.1 added");
    shows_part(
        &mut h,
        "+ (key:\"gui.1.1\",at:\"1765-06-01\",basis:Assumed(\"GUI experiment",
    );
    // Export: a CSV and a manifest that say "experiment", and the lineage.
    click(&mut h, "Editor");
    let out = dir.path().join("export");
    fill(&mut h, "export directory", &out.display().to_string());
    press(&mut h, "Export");
    let csv = std::fs::read_to_string(out.join("series.csv")).expect("the CSV is written");
    assert!(
        csv.starts_with("# rustyecon-gui export: experiment, draft\n"),
        "{csv}"
    );
    assert!(csv.contains("\n# origin experiment\n"));
    let manifest = std::fs::read_to_string(out.join("manifest.ron")).expect("the manifest");
    let g = GuiManifest::from_ron(&manifest).expect("the envelope reads");
    assert_eq!((g.origin, g.draft), (Origin::Experiment, true));
    assert_eq!(g.manifest.tape, name);
    let l = std::fs::read_to_string(out.join("tape.lineage.ron")).expect("the lineage");
    let l = Lineage::from_ron(&l).expect("the lineage reads");
    assert_eq!(l.parent.path, path);
    assert_eq!(l.edits[0].edit.note, "restore the capacity early");
    assert!(h
        .state()
        .model()
        .log()
        .iter()
        .any(|e| e.text.starts_with("run 1 exported to ")));
    // Save tape as: the tape, and its lineage beside it.
    let saved = dir.path().join("branch.ron").display().to_string();
    fill(&mut h, "save path", &saved);
    press(&mut h, "Save tape as");
    let text = std::fs::read_to_string(&saved).expect("the tape is saved");
    assert_eq!(certify::tape_hash(&Tape::from_ron(&text).unwrap()), tape);
    assert!(std::path::Path::new(&lineage_path(&saved)).exists());
    scroll_to(&mut h, &format!("on disk: {saved}"));
    shows(&mut h, &format!("on disk: {saved}"));
    // The saved tape, its tolerance edited by hand and its lineage beside it, opened in a new
    // window: the host reads the base its lineage names, and the chip paints "ledger
    // changed". With the base gone, the chip paints "ledger unchecked" and why.
    let mut t = Tape::from_ron(&text).unwrap();
    let flow = t
        .params
        .iter_mut()
        .find(|p| p.key.as_str() == "ledger.rel_flow")
        .unwrap();
    flow.value *= 2.0;
    let hand = dir.path().join("by-hand.ron").display().to_string();
    std::fs::write(&hand, t.to_ron()).unwrap();
    std::fs::copy(lineage_path(&saved), lineage_path(&hand)).unwrap();
    let mut changed = harness_on(hand.clone(), None);
    paints(&mut changed, " · ledger changed", |t| {
        t.contains(" · ledger changed")
    });
    std::fs::rename(&path, dir.path().join("gone.ron")).unwrap();
    let mut unchecked = harness_on(hand, None);
    paints(&mut unchecked, " · ledger unchecked", |t| {
        t.contains(" · ledger unchecked")
    });
    shows_part(
        &mut unchecked,
        &format!("ledger unchecked: its lineage's ancestor {path} cannot be read: "),
    );
}

/// The app with no tape and no session, in a 1600 × 1000 window.
fn harness_bare<'a>() -> Harness<'a, GuiApp> {
    Harness::builder()
        .with_size(egui::vec2(1600.0, 1000.0))
        .build_eframe(move |cc| {
            GuiApp::new(
                &cc.egui_ctx,
                Launch {
                    tape: None,
                    files: None,
                    smoke: None,
                },
            )
        })
}

#[test]
fn the_lab_script_shows_appendix_b_and_its_goldens() {
    // §9 G1's gate, on the screen: with no tape open, the Lab tab paints the SSRN Appendix B
    // instance's x*, v, Y and N_a as the oracle's own doubles (its outputs, called here
    // directly, printed with the shortest digits that read back), beside the paper's published
    // 0.86315, 0.54344, 7.88061 and 1.34338 and the generator's 70-digit goldens; its regime;
    // f over x with the root; an edited knob that drops the goldens; and a sweep.
    use oracle::{Economy, Regime};
    let eq = match Economy::new(rustyecon_gui::lab::presets::appendix_b())
        .expect("valid")
        .solve()
    {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{other:?}"),
    };
    let mut h = harness_bare();
    h.step();
    click(&mut h, "Lab");
    shows(&mut h, "regime Interior");
    for (value, published, golden) in [
        (eq.x_star, "0.86315", "0.863150418162437031916392798424"),
        (eq.v, "0.54344", "0.543435960696778328420453096896"),
        (eq.y, "7.88061", "7.88060552497290767677101216703"),
        (eq.n_a, "1.34338", "1.34338188009771734611380197514"),
    ] {
        let text = format!("{value:?}");
        see(&mut h, &text);
        let back: f64 = text.parse().unwrap();
        assert_eq!(back.to_bits(), value.to_bits());
        see(&mut h, published);
        see(&mut h, golden);
    }
    // The painted value is the lab's view of the oracle's own output.
    let vm = h
        .state()
        .ui_state()
        .lab
        .view()
        .cloned()
        .expect("the lab solved");
    let x = vm.outputs.iter().find(|o| o.key == "x_star").unwrap();
    assert_eq!(x.value, format!("{:?}", eq.x_star));
    assert_eq!(vm.origin, "oracle");
    // f over x: the root is named, and the bracket.
    see(
        &mut h,
        &format!("x* {:?}; bracket [1e-12, 1]; regime Interior", eq.x_star),
    );
    // An edit: N 5. The regime is solved again, and no golden is shown beside another
    // economy's output.
    press(&mut h, "Instance");
    scroll_to(&mut h, "workers");
    fill(&mut h, "workers", "5");
    h.key_press(egui::Key::Enter);
    h.step();
    h.step();
    shows_part(
        &mut h,
        "edited from its preset: its goldens are another economy's",
    );
    let vm = h
        .state()
        .ui_state()
        .lab
        .view()
        .cloned()
        .expect("solved again");
    assert!(vm.edited);
    let mut p = rustyecon_gui::lab::presets::appendix_b();
    p.workers = 5.0;
    let five = match Economy::new(p).unwrap().solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{other:?}"),
    };
    see(&mut h, &format!("{:?}", five.x_star));
    // A sweep of N over 200 points.
    press(&mut h, "Reset to preset");
    press(&mut h, "Run the sweep");
    let s = match h.state().ui_state().lab.swept() {
        Some(Ok(s)) => s.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!((s.xs.len(), s.xs[0], s.xs[199]), (200, 2.0, 6.0));
    assert_eq!(s.knob, "workers");
    see(&mut h, &rustyecon_gui::ui::lab::sweep_line(&s));
    // Another unit: 1c's M3, its x* beside the generator's 70-digit golden.
    press(&mut h, "unit");
    press(&mut h, "1c many machine types and the Leontief inverse");
    let vm = h.state().ui_state().lab.view().cloned();
    let vm = vm.expect("solved");
    assert_eq!((vm.unit.as_str(), vm.preset.as_deref()), ("1c", Some("M3")));
    let x = vm.outputs.iter().find(|o| o.key == "x_star").unwrap();
    assert!(x.golden.as_ref().is_some_and(|g| g.agrees));
    see(&mut h, &x.value);
    see(&mut h, "0.910574687993354806041937921365");
}
