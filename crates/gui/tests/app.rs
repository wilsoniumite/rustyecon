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
//! - `every_drawn_vertex_is_recorded`: every vertex the plots lend egui is a recorded (tick,
//!   value) of the store, no gap in the record is bridged, and each line keeps its extremes.

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use rustyecon_engine::prelude::*;
use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::model::{Intent, Model};
use rustyecon_gui::run::{At, Entity, Measure, PauseReason, RunStatus, Series, SeriesKey, Store};
use rustyecon_gui::ui::fmt;
use rustyecon_gui::ui::plots::DrawnLine;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

fn key(s: &str) -> Key {
    Key::new(s).expect("a key")
}

/// The app on a tape of `tapes/`, in a 1600 × 1000 window, with no session.
fn harness<'a>(tape: &str) -> Harness<'a, GuiApp> {
    let path = format!("{}/../../tapes/{tape}.ron", env!("CARGO_MANIFEST_DIR"));
    Harness::builder()
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
        })
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
    let markets = ["town", "village"]
        .iter()
        .flat_map(|n| ["bread", "fuel", "grain"].map(|g| price(n, g)))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        plots.iter().cloned().collect::<BTreeSet<_>>(),
        markets,
        "every price"
    );
    let drawn = h.state().drawn();
    assert_eq!(drawn.len(), 6, "a line for each price");
    assert_eq!(vertices(&drawn), 0, "no tick has run");
    // Three panels, one per unit, each naming it.
    let vm = rustyecon_gui::vm::plots::build(store(h.state()), &plots, None).unwrap();
    let units: Vec<&str> = vm.panels.iter().map(|p| p.unit.as_str()).collect();
    assert_eq!(units, ["coin per bread", "coin per fuel", "coin per grain"]);
    h.key_press(egui::Key::Space);
    step_until(&mut h, "every price drawn", |a| {
        let d = a.drawn();
        d.len() == 6 && d.iter().all(|l| l.segments.iter().any(|s| s.len() > 20))
    });
    // Live: it grows while the run runs.
    let before = vertices(&h.state().drawn());
    let tick = store(h.state()).tick();
    step_until(&mut h, "the plot grows", |a| {
        store(a).tick() > tick + 50 && vertices(&a.drawn()) > before
    });
    assert!(matches!(
        status(h.state().model()),
        Some(RunStatus::Running { .. })
    ));
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
    shows_part(&mut h, &format!("snapshot of tick {}", cut + 1));
    click(&mut h, "Break on error");
    step_until(&mut h, "the breakpoint off", |a| {
        a.model().session.breakpoints.is_empty()
    });
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
    // widths, the narrower thinning harder.
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
