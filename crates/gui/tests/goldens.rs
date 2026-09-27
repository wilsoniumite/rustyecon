//! The view-model goldens (docs/GUI.md §8.1): every `vm::*` builder on the gate world, saved as
//! RON, at four points: tick 0, where bread rations; after the 1760 cut, where the registry
//! shows `mine.capacity` with `mine.capacity.cut`'s basis, `mine.cut` and its date; 1768, when
//! the oven opens; and tick 2,080. The Appendix B world has two: tick 0 and tick 20,000.
//!
//! Each point is a record made by a Runner stepped there, with a snapshot of the state its
//! last tick left, and a build named `golden` so a commit does not move the identity chip. The
//! files are `tests/golden/<tape>-<point>/<vm>.ron`, one per builder. `UPDATE_GOLDEN=1`
//! rewrites them, in the commit that retunes a world. They run under `scripts/gui.sh` only, so
//! a retune never fails an engine step on a GUI golden (D1).
//!
//! Besides the text, each point asserts what the design says it shows, so a rewrite cannot
//! pass a golden that has lost it.

mod common;

use certify::Build;
use common::{tape_of, APPB, GATE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::run::log::{self, Entry, RationWatch};
use rustyecon_gui::run::{
    At, Cmd, Entity, LedgerCheck, Measure, Obs, Origin, RunId, Runner, SeriesKey, Store,
};
use rustyecon_gui::vm;
use rustyecon_gui::vm::inspector::InspectorVm;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn key(s: &str) -> Key {
    Key::new(s).expect("a key")
}

/// Every view-model at one point.
struct Golden {
    toolbar: vm::toolbar::ToolbarVm,
    timeline: Option<vm::timeline::TimelineVm>,
    outliner: Option<vm::outliner::OutlinerVm>,
    plots: Option<vm::plots::PlotsVm>,
    inspector: Vec<(Entity, Option<InspectorVm>)>,
    registry: Option<vm::registry::RegistryVm>,
    log: vm::log::LogVm,
}

/// A Runner with a fixed build, its store and its log, stepped by hand.
struct Rig {
    runner: Runner,
    seen: Arc<Mutex<Vec<Obs>>>,
    store: Store,
    log: Vec<Entry>,
    watch: RationWatch,
}

impl Rig {
    fn new(tape: &Tape) -> Rig {
        let build = Build {
            commit: "golden".to_string(),
            dirty: false,
            target: "golden".to_string(),
            rustc: "golden".to_string(),
        };
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let runner = Runner::new(
            build,
            Box::new(move |o| sink.lock().unwrap().push(o)),
            Box::new(|| {}),
        );
        let mut rig = Rig {
            runner,
            seen,
            store: Store::default(),
            log: Vec::new(),
            watch: RationWatch::default(),
        };
        rig.cmd(Cmd::Load {
            tape: Box::new(tape.clone()),
            from: None,
        });
        rig
    }

    fn cmd(&mut self, c: Cmd) {
        self.runner.handle(c);
        while self.runner.advance(97).busy {}
        let obs: Vec<Obs> = std::mem::take(&mut *self.seen.lock().unwrap());
        for o in obs {
            self.log.extend(log::entries(RunId(0), &o));
            if let Obs::Batch(b) = &o {
                let onsets = self.watch.onsets(RunId(0), b, self.store.catalogue());
                self.log.extend(onsets);
            }
            self.store.ingest(o).expect("the record takes it");
        }
    }

    /// Step to state tick `to`, and take a snapshot of that state.
    fn to(&mut self, to: u64) {
        let n = to - self.store.tick();
        self.cmd(Cmd::Step(n));
        assert_eq!(self.store.tick(), to);
        self.cmd(Cmd::Snapshot(to));
        assert!(self.store.snapshot(to).is_some());
    }
}

fn every_price(w: &World) -> Vec<SeriesKey> {
    w.markets()
        .map(|(n, g)| SeriesKey {
            measure: Measure::Price,
            at: At::Market {
                node: w.key_of(n).unwrap().clone(),
                good: w.key_of(g).unwrap().clone(),
            },
        })
        .collect()
}

fn golden(rig: &Rig, selections: &[Entity], pins: &[Entity]) -> Golden {
    let s = &rig.store;
    let w = s.world().expect("loaded");
    let plots = every_price(w);
    Golden {
        toolbar: vm::toolbar::build(s, Origin::Run, LedgerCheck::NoParent),
        timeline: vm::timeline::build(s, None),
        outliner: vm::outliner::build(s, selections.first(), pins, &plots),
        plots: vm::plots::build(s, &plots, None),
        inspector: selections
            .iter()
            .map(|e| (e.clone(), vm::inspector::build(s, e, None)))
            .collect(),
        registry: vm::registry::build(s, None),
        log: vm::log::build(&rig.log),
    }
}

/// Compare every view-model of a point with its golden file, `tests/golden/<point>/<vm>.ron`,
/// or write them under `UPDATE_GOLDEN=1`. Each is pretty RON down to the depth where one
/// record (a year, an event, a param, a row, a log line) fits on one line.
fn check(point: &str, g: &Golden) {
    // G1: every market the point inspects explains its step, and the explainer's next price,
    // markets' own function of the recorded inputs, is the run's bit for bit.
    for (e, v) in &g.inspector {
        if let (Entity::Market { .. }, Some(InspectorVm::Market(m))) = (e, v) {
            let x = m.explainer.as_ref().expect("the explainer");
            assert_eq!(x.equal, Some(true), "{point} {e:?}");
            assert_eq!(x.recorded.map(f64::to_bits), Some(x.next.to_bits()));
        }
    }
    compare(point, "toolbar", &g.toolbar, 3);
    compare(point, "timeline", &g.timeline, 2);
    compare(point, "outliner", &g.outliner, 4);
    compare(point, "plots", &g.plots, 4);
    compare(point, "inspector", &g.inspector, 4);
    compare(point, "registry", &g.registry, 2);
    compare(point, "log", &g.log, 2);
}

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
            "{point}/{name}: the view-model differs from {} at line {}:\n  golden: {a}\n  \
             now:    {b}\n(UPDATE_GOLDEN=1 rewrites it, in the commit that retunes the world)",
            p.display(),
            n + 1
        );
    }
}

fn market<'a>(g: &'a Golden, node: &str, good: &str) -> &'a vm::inspector::MarketVm {
    g.inspector
        .iter()
        .find_map(|(e, v)| match (e, v) {
            (Entity::Market { node: n, good: gd }, Some(InspectorVm::Market(m)))
                if n.as_str() == node && gd.as_str() == good =>
            {
                Some(m.as_ref())
            }
            _ => None,
        })
        .expect("the market's inspector")
}

fn registry_row<'a>(g: &'a Golden, k: &str) -> &'a vm::registry::ParamRowVm {
    g.registry
        .as_ref()
        .unwrap()
        .params
        .iter()
        .find(|r| r.key == k)
        .expect("the param's row")
}

#[test]
fn gate_view_models_equal_their_goldens() {
    let tape = tape_of(GATE);
    let mut rig = Rig::new(&tape);
    let clock = rig.store.world().unwrap().clock;
    let cut = clock.tick_of(Date::parse("1760-03-01").unwrap()).unwrap();
    let oven = clock.tick_of(Date::parse("1768-04-01").unwrap()).unwrap();
    let selections = [
        Entity::Market {
            node: key("town"),
            good: key("bread"),
        },
        Entity::Market {
            node: key("village"),
            good: key("bread"),
        },
        Entity::Actor(key("mill")),
        Entity::Actor(key("oven")),
        Entity::Param(key("mine.capacity")),
        Entity::Param(key("mine.capacity.cut")),
        Entity::Event(key("mine.cut")),
        Entity::Event(key("pension")),
        Entity::Good(key("bread")),
        Entity::Node(key("village")),
        Entity::Class(key("pensioners")),
    ];
    let pins = [
        Entity::Market {
            node: key("town"),
            good: key("bread"),
        },
        Entity::Param(key("mine.capacity")),
    ];

    // Tick 0: bread rations. The pensioners' coin buys less bread than they ask for.
    rig.to(1);
    let g = golden(&rig, &selections, &pins);
    let rationed = ["town", "village"].iter().any(|n| {
        market(&g, n, "bread").rationing.iter().any(|r| {
            let v: Vec<f64> = r.values.iter().map(|x| x.value.unwrap()).collect();
            r.side == "buy" && v[2] < v[0]
        })
    });
    assert!(rationed, "bread rations at tick 0");
    let tb = market(&g, "town", "bread");
    assert_eq!(tb.tick, Some(0));
    assert_eq!(tb.values[0].unit, "coin per bread");
    assert!(tb.log_step.is_some() && tb.rate.as_ref().unwrap().key.as_str() == "rate.bread");
    check("gate-tick0", &g);

    // After the 1760 cut: the registry shows mine.capacity copied from mine.capacity.cut, with
    // its basis, the event mine.cut and its date.
    rig.to(cut + 1);
    let g = golden(&rig, &selections, &pins);
    let row = registry_row(&g, "mine.capacity");
    let c = row.copied.as_ref().expect("the cut copied a value in");
    assert_eq!(c.source.as_str(), "mine.capacity.cut");
    assert_eq!(c.basis, r#"Assumed("gate world: half")"#);
    assert_eq!(
        (c.event.as_str(), c.date.as_str(), c.tick),
        ("mine.cut", "1760-03-01", cut)
    );
    assert_eq!(row.current, Some(26.0));
    assert_eq!(row.genesis, 52.0);
    assert!(
        row.sites.iter().all(|s| s.per_tick_now == Some(0.5)),
        "{row:?}"
    );
    check("gate-cut", &g);

    // 1768: the oven opens.
    rig.to(oven + 1);
    let g = golden(&rig, &selections, &pins);
    let opened = g
        .timeline
        .as_ref()
        .unwrap()
        .events
        .iter()
        .any(|e| e.key.as_str() == "oven.opens" && e.fired && e.tick == oven);
    assert!(opened, "oven.opens has fired");
    check("gate-1768", &g);

    // Tick 2,080: the run's end, with the restoration copied back in.
    rig.to(2080);
    let g = golden(&rig, &selections, &pins);
    let c = registry_row(&g, "mine.capacity").copied.clone().unwrap();
    assert_eq!(c.source.as_str(), "mine.capacity.base");
    assert_eq!(g.toolbar.health.hash.as_deref(), Some("0x61f9c8529131ff17"));
    assert_eq!(g.timeline.as_ref().unwrap().ring.len(), 41);
    check("gate-2080", &g);
}

#[test]
fn appb_view_models_equal_their_goldens() {
    let tape = tape_of(APPB);
    let mut rig = Rig::new(&tape);
    let selections = [
        Entity::Market {
            node: key("home"),
            good: key("good"),
        },
        Entity::Actor(key("desk.good")),
        Entity::Actor(key("workers")),
        Entity::Param(key("spend.workers")),
        Entity::Good(key("labour")),
        Entity::Class(key("workers")),
    ];
    let pins = [Entity::Actor(key("desk.good"))];
    rig.to(1);
    let g = golden(&rig, &selections, &pins);
    assert_eq!(
        g.plots.as_ref().unwrap().panels.len(),
        4,
        "a unit per price"
    );
    check("appb-tick0", &g);
    rig.to(20_000);
    let g = golden(&rig, &selections, &pins);
    assert_eq!(g.toolbar.health.hash.as_deref(), Some("0xe1fa082b26995867"));
    check("appb-20000", &g);
}
