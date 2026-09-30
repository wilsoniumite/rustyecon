//! The map on the demo's second pass (docs/demo/WORLD-V2.md §7, §8; D2.3, 2026-09-30):
//! `tapes/demo-gb-v2.ron` with its own lens table, `worlds/demo-gb/lenses-v2a1.csv`.
//!
//! - `demo_v2_lens_values_equal_the_engine`: every lens of the table for every county at
//!   report ticks 0, 51 and 259, against the engine's own report, params and states, read from
//!   a Sim the test steps: the maker's markup against its rule's own `maker_reservation`, the
//!   horse-day's markup against the capacity desk's recipe, the oracle lenses against 1g solved
//!   at the params the Sim holds, and the windows' counts tick by tick.
//! - `price_horse_has_no_value_on_an_idle_tick`: a county whose capacity desk starts with three
//!   times its herd orders no horses, so no horse trades; the horse's price has no value there
//!   (O47, O54), its card says why, and the idle lens counts the ticks.
//! - `v2_map_values_equal_table`: the ranked table shows the map's values under every lens.
//! - `the_v2_map_is_fitted_clear_and_its_values_whole`: G1.6's and D2.1's layout checks on the
//!   second pass's store, under its longest lens name.
//! - `the_map_takes_the_lens_table_of_its_tape`: the pane's lens table follows the tape.

mod common;

use certify::Build;
use egui_kittest::Harness;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::{maker_reservation, Spec};
use rustyecon_gui::model::Intent;
use rustyecon_gui::run::{Catalogue, Cmd, Entity, Obs, Origin, Runner, Store};
use rustyecon_gui::ui::map::{self as ui_map, MapState};
use rustyecon_gui::vm::map::{self as vm_map, LensVm};
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::chain::{self, ChainParams, RuleA};
use rustyecon_worldgen::lens::{self as wl, NO_TRADE_WINDOW};
use rustyecon_worldgen::tables::Lens;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

fn v2_path() -> String {
    format!("{}/../../tapes/demo-gb-v2.ron", env!("CARGO_MANIFEST_DIR"))
}

fn v2_text() -> &'static str {
    static T: OnceLock<String> = OnceLock::new();
    T.get_or_init(|| std::fs::read_to_string(v2_path()).expect("tapes/demo-gb-v2.ron"))
}

fn v2_tape() -> &'static Tape {
    static T: OnceLock<Tape> = OnceLock::new();
    T.get_or_init(|| Tape::from_ron(v2_text()).expect("the v2 tape parses"))
}

fn atlas() -> &'static Atlas {
    static A: OnceLock<Atlas> = OnceLock::new();
    A.get_or_init(|| Atlas::gb().expect("the bundled atlas"))
}

fn lenses() -> &'static [Lens] {
    static L: OnceLock<Vec<Lens>> = OnceLock::new();
    L.get_or_init(|| wl::for_tape(wl::V2_TAPE).expect("the second pass's lenses"))
}

fn lens(key: &str) -> &'static Lens {
    lenses()
        .iter()
        .find(|l| l.key == key)
        .unwrap_or_else(|| panic!("no lens {key}"))
}

/// A Runner with a fixed build, stepped by hand, and the store of what it reported.
fn record(tape: &Tape, ticks: u64) -> Store {
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
    take(&mut runner, &mut store, Cmd::Catalogue(Catalogue::Lean));
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

/// The second pass run five years, with the lean catalogue a world of 93 nodes takes.
fn v2_store() -> &'static Store {
    static S: OnceLock<Store> = OnceLock::new();
    S.get_or_init(|| {
        assert_eq!(Catalogue::for_nodes(v2_tape().nodes.len()), Catalogue::Lean);
        record(v2_tape(), 260)
    })
}

fn lens_vm(store: &Store, l: &Lens, tick: u64) -> LensVm {
    vm_map::lens(store, Origin::Run, atlas(), l, Some(tick)).expect("a loaded run")
}

/// The engine's own numbers for one county at one report tick.
#[derive(Debug, Clone, Default)]
struct Truth {
    /// Posted prices and cleared volumes of labour, land, fodder, the horse, horse-days, the
    /// good; and whether each traded.
    price: [f64; 6],
    cleared: [f64; 6],
    trades: [bool; 6],
    /// The params in force, by column, as registered (N and T a year).
    param: BTreeMap<String, f64>,
    /// κ and δ per tick; a horse-day's fodder and labour; ψ.
    kappa: f64,
    delta: f64,
    run_fodder: f64,
    run_labour: f64,
    psi: f64,
    n_tick: f64,
    share: f64,
    used: f64,
    due: f64,
    paid: f64,
    out_good: f64,
    out_hours: f64,
    held: f64,
    target: f64,
    out_horse: f64,
    serving: f64,
    out_fodder: f64,
    /// The maker's net markup and whether it withheld, by its rule's own code.
    markup: f64,
    withheld: bool,
    /// The capacity desk's full cost of a horse-day at posted prices, O + δ·p_K/κ, from its
    /// running recipe as the tape resolved it.
    full: f64,
}

const GOODS: [&str; 6] = ["labour", "land", "fodder", "horse", "traction", "good"];

/// Every county's truth at every report tick up to the last of `ticks`, from a Sim of the v2
/// tape stepped here.
fn engine_truth(last: u64) -> Vec<BTreeMap<String, Truth>> {
    let mut sim = Sim::new(v2_tape()).expect("the v2 tape loads");
    let w = sim.world().clone();
    let goods = GOODS.map(|g| w.id_of::<GoodId>(g).expect("a good"));
    let mut index: BTreeMap<(NodeId, GoodId), usize> = BTreeMap::new();
    let mut out = Vec::new();
    let maker = |k: &str| {
        let a = w
            .actors
            .iter()
            .find(|a| w.key_of(a.id).map(Key::as_str) == Some(k))
            .expect("the maker");
        match &a.spec {
            Spec::Maker(m) => m.clone(),
            other => panic!("{k}: {other:?}"),
        }
    };
    let capacity = |k: &str| {
        let a = w
            .actors
            .iter()
            .find(|a| w.key_of(a.id).map(Key::as_str) == Some(k))
            .expect("the capacity desk");
        match &a.spec {
            Spec::CapacityDesk(c) => c.clone(),
            other => panic!("{k}: {other:?}"),
        }
    };
    let makers: BTreeMap<String, _> = w
        .nodes
        .iter()
        .map(|n| {
            (
                n.key.as_str().to_string(),
                maker(&format!("{}.desk.maker", n.key)),
            )
        })
        .collect();
    let desks: BTreeMap<String, _> = w
        .nodes
        .iter()
        .map(|n| {
            (
                n.key.as_str().to_string(),
                capacity(&format!("{}.desk.capacity", n.key)),
            )
        })
        .collect();
    for t in 0..=last {
        let r = sim.step().expect("the v2 tape steps");
        assert_eq!(r.tick, t);
        if t == 0 {
            index = r
                .markets
                .iter()
                .enumerate()
                .map(|(i, l)| ((l.node, l.good), i))
                .collect();
        }
        let par = |s: Site| -> Result<f64, AgentError> {
            let v = sim
                .param(s.param)
                .ok_or(AgentError::Core(CoreError::UnknownParam(s.param)))?;
            s.convert(&w.clock, v).map_err(AgentError::Core)
        };
        let mut per = BTreeMap::new();
        for n in &w.nodes {
            let k = n.key.as_str();
            let line = |g: GoodId| &r.markets[index[&(n.id, g)]];
            let mut x = Truth::default();
            for (i, &g) in goods.iter().enumerate() {
                x.price[i] = line(g).price;
                x.cleared[i] = line(g).cleared;
                x.trades[i] = line(g).trades();
            }
            let p = |c: &str| {
                sim.param(w.id_of::<ParamId>(c).unwrap_or_else(|| panic!("{c}")))
                    .expect("its value")
            };
            for c in [
                "workers",
                "land",
                "space",
                "eta",
                "g0",
                "g1",
                "k",
                "chi_max",
                "fodder.land",
                "horse.own_hours",
                "horse.labour",
                "horse.land",
            ] {
                x.param.insert(c.to_string(), p(&format!("{k}.{c}")));
            }
            x.kappa = ClockMethod::Flow
                .per_tick(&w.clock, p("horse.kappa"))
                .expect("a flow");
            x.delta = ClockMethod::Fraction
                .per_tick(&w.clock, p("horse.delta"))
                .expect("a fraction");
            x.run_fodder = p("horse.run.fodder");
            x.run_labour = p("horse.run.labour");
            x.psi = p("reserve.maker");
            x.n_tick = ClockMethod::Flow
                .per_tick(&w.clock, p(&format!("{k}.workers")))
                .expect("a flow");
            let a = |role: &str| {
                w.id_of::<ActorId>(&format!("{k}.{role}"))
                    .expect("an actor")
            };
            match sim.actor_state(a("desk.good")) {
                Some(ActorState::GoodDesk(s)) => {
                    (x.share, x.used, x.out_good) = (s.share, s.used, s.output);
                }
                other => panic!("{k}: {other:?}"),
            }
            match sim.actor_state(a("desk.capacity")) {
                Some(ActorState::Capacity(s)) => {
                    (x.held, x.target, x.out_hours) = (s.held, s.target, s.output);
                }
                other => panic!("{k}: {other:?}"),
            }
            match sim.actor_state(a("desk.maker")) {
                Some(ActorState::Maker(s)) => (x.serving, x.out_horse) = (s.serving, s.output),
                other => panic!("{k}: {other:?}"),
            }
            match sim.actor_state(a("desk.fodder")) {
                Some(ActorState::MachDesk(s)) => x.out_fodder = s.output,
                other => panic!("{k}: {other:?}"),
            }
            match sim.actor_state(a("provider")) {
                Some(ActorState::Provider(s)) => (x.due, x.paid) = (s.due, s.paid),
                other => panic!("{k}: {other:?}"),
            }
            // The maker's markup by its rule's own code, at this tick's posted prices.
            let pr = |g: GoodId| -> Result<f64, AgentError> { Ok(line(g).price) };
            let res = maker_reservation(&makers[k], &par, &pr).expect("the maker's markup");
            (x.markup, x.withheld) = (res.markup, res.withholds);
            // The capacity desk's full cost of a horse-day, from its running recipe.
            let d = &desks[k];
            let mut o = 0.0;
            for i in &d.running.goods {
                o += par(i.coef).expect("a coefficient") * line(i.good).price;
            }
            let o = o + par(d.running.labour).expect("labour") * line(d.labour).price;
            let full =
                o + par(d.delta).expect("δ") * line(d.stock).price / par(d.kappa).expect("κ");
            x.full = full;
            per.insert(k.to_string(), x);
        }
        out.push(per);
    }
    out
}

/// The oracle's point at the params the engine holds, solved here from the Sim's own params.
fn oracle(x: &Truth) -> chain::ChainPoint {
    let p = |c: &str| x.param[c];
    let clock = v2_tape_clock();
    chain::point(
        &ChainParams {
            workers: p("workers"),
            land: p("land"),
            space: p("space"),
            eta: p("eta"),
            g0: p("g0"),
            g1: p("g1"),
            k: p("k"),
            chi_max: p("chi_max"),
            rule: RuleA {
                fodder_land: p("fodder.land"),
                own_hours: p("horse.own_hours"),
                labour: p("horse.labour"),
                pasture: p("horse.land"),
            },
            kappa: x.kappa,
            delta: x.delta,
            fodder: true,
        },
        &clock,
    )
    .expect("an interior point")
}

fn v2_tape_clock() -> Clock {
    Clock {
        start: v2_tape().header.start,
        ticks_per_year: v2_tape().header.ticks_per_year,
    }
}

#[test]
fn demo_v2_lens_values_equal_the_engine() {
    // U3, U6 (WORLD-V2 §7): at report ticks 0, 51 and 259 every lens of the second pass's table,
    // for every county, equals its formula on the engine's own report, params and states, read
    // from a Sim this test steps. The maker's markup is its rule's own (`maker_reservation`);
    // the horse-day's markup is the capacity desk's full cost at posted prices; the oracle
    // lenses are 1g at the params the Sim holds; the windows count up to the cursor.
    let ticks = [0_u64, 51, 259];
    let truth = engine_truth(259);
    let store = v2_store();
    let ln = rustyecon_engine::num::ln;
    let window = |k: &str, t: u64, f: &dyn Fn(&Truth) -> bool| -> f64 {
        let lo = (t + 1).saturating_sub(NO_TRADE_WINDOW);
        (lo..=t).filter(|&s| f(&truth[s as usize][k])).count() as f64
    };
    let heads = |x: &Truth| (x.held + x.serving) / x.n_tick;
    let mut checked = 0;
    let mut wrong = Vec::new();
    let mut values = 0;
    for &t in &ticks {
        for l in lenses() {
            let vm = lens_vm(store, l, t);
            assert_eq!(vm.tick, Some(t));
            for r in &vm.regions {
                let k = r.key.as_str();
                let x = &truth[t as usize][k];
                let x0 = &truth[0][k];
                let (w, rr, pf, ph, p) =
                    (x.price[0], x.price[1], x.price[2], x.price[4], x.price[5]);
                let e = || oracle(x);
                let want: Option<f64> = match l.key.as_str() {
                    "horses.per.head" => Some(heads(x)),
                    "since.horses.per.head" => Some(ln(heads(x) / heads(x0))),
                    "horses.vs.oracle" => Some(ln((x.held + x.serving) / e().heads())),
                    "horses.vs.plan" => Some(ln(x.held / x.target)),
                    "price.fodder" => Some(pf / rr),
                    "price.horse" => x.trades[3].then_some(x.markup),
                    "hday.markup" => Some(ph / x.full - 1.0),
                    "land.to.fodder" => Some(x.param["fodder.land"] * x.out_fodder / x.cleared[1]),
                    "land.to.horses" => Some(
                        (x.param["fodder.land"] * x.out_fodder
                            + x.param["horse.land"] * x.out_horse)
                            / x.cleared[1],
                    ),
                    "reserve.ticks" => Some(window(k, t, &|y| y.withheld)),
                    "idle.horse" => Some(window(k, t, &|y| !y.trades[3])),
                    "wage.baskets" => Some(w / (p + x.param["space"] * rr)),
                    "wage.goods" => Some(w / p),
                    "wage.land" => Some(w / rr),
                    "rent.goods" => Some(rr / p),
                    "share.land" => {
                        let (wl_, rt) = (w * x.cleared[0], rr * x.cleared[1]);
                        Some(rt / (wl_ + rt))
                    }
                    "share.labour" => {
                        let (wl_, rt) = (w * x.cleared[0], rr * x.cleared[1]);
                        Some(wl_ / (wl_ + rt))
                    }
                    "frontier.x" => Some(1.0 - x.share),
                    "participation" => Some(x.cleared[0] / x.n_tick),
                    "output.per.head" => Some(x.cleared[5] / x.n_tick),
                    "price.good" => Some(p / rr),
                    "price.hday" => Some(ph / rr),
                    "relief.burden" => Some(x.due / (rr * x.cleared[1])),
                    "shortfall" => Some((x.due - x.paid) / x.due),
                    // The rationing lines are the report's; lens_values_equal_the_engine holds
                    // them on v1, and here the value must merely be one.
                    "rationing" => None,
                    "no.trade" => Some(window(k, t, &|y| {
                        !(y.trades[0] && y.trades[1] && y.trades[2] && y.trades[4] && y.trades[5])
                    })),
                    "gap.oracle" => {
                        Some(gap_terms(x).iter().map(|g| g.abs()).fold(0.0, f64::max) / 1e-3)
                    }
                    "gap.wage" => Some(ln((w / rr) / e().v)),
                    "since.wage.baskets" => {
                        let wb =
                            |y: &Truth| y.price[0] / (y.price[5] + y.param["space"] * y.price[1]);
                        Some(ln(wb(x) / wb(x0)))
                    }
                    "since.output.per.head" => {
                        Some(ln((x.cleared[5] / x.n_tick) / (x0.cleared[5] / x0.n_tick)))
                    }
                    "param.workers" => Some(x.param["workers"]),
                    "param.land" => Some(x.param["land"]),
                    "param.eta" => Some(x.param["eta"]),
                    "param.chi_max" => Some(x.param["chi_max"]),
                    "param.b" => {
                        Some(x.param["fodder.land"] + x.delta / x.kappa * x.param["horse.land"])
                    }
                    "param.lam" => Some(x.delta / x.kappa * x.param["horse.labour"]),
                    other => panic!("no truth for {other}"),
                };
                match (want, r.value) {
                    (Some(want), Some(got)) => {
                        if (got - want).abs() > 1e-12 * want.abs().max(1e-3) {
                            wrong.push(format!("{} {k} at {t}: map {got}, engine {want}", l.key));
                        }
                        values += 1;
                    }
                    (None, Some(_)) if l.key == "rationing" => values += 1,
                    (None, None) if l.key == "price.horse" => {}
                    (want, got) => {
                        wrong.push(format!(
                            "{} {k} at {t}: map {got:?} ({:?}), engine {want:?}",
                            l.key, r.why
                        ));
                    }
                }
                checked += 1;
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(12)]
    );
    assert_eq!(checked, 3 * 36 * 93);
    println!("{checked} lens values checked, {values} with a value");
    // The county card gives each county the same values, and names what carries its gap.
    for &t in &ticks[1..] {
        for k in ["county.lan", "county.gla", "county.sut"] {
            let sel = Entity::Node(common::key(k));
            let vm = vm_map::build(
                store,
                Origin::Run,
                atlas(),
                lenses(),
                "wage.land",
                Some(&sel),
                Some(t),
            )
            .expect("a loaded run");
            let card = vm.county.expect("the card");
            assert_eq!(card.lenses.len(), 36);
            for cl in &card.lenses {
                let map = lens_vm(store, lens(&cl.key), t);
                let r = map.regions.iter().find(|r| r.key == k).expect("the county");
                assert_eq!(cl.value, r.value, "{} {k} at {t}", cl.key);
            }
            // What carries its gap: the three observables furthest from 1g at the Sim's params.
            assert_eq!(card.causes.len(), 3, "{k} at {t}");
            let x = &truth[t as usize][k];
            let gaps = gap_terms(x);
            let mut largest: Vec<f64> = gaps.iter().map(|g| g.abs()).collect();
            largest.sort_by(|a, b| b.total_cmp(a));
            for (c, want) in card.causes.iter().zip(&largest) {
                assert!(
                    (c.gap.abs() - want).abs() <= 1e-12,
                    "{k} at {t}: {c:?} against {want}"
                );
            }
        }
    }
}

#[test]
fn price_horse_has_no_value_on_an_idle_tick() {
    // WORLD-V2 §7.2 (decision 333; O47, O54): an idle market's price is not a valuation. Here
    // Bedfordshire's capacity desk starts with three times its herd, so its target is below
    // what it holds and it orders none; its maker still offers, and no horse trades. On such a
    // tick the horse's price has no value and says why, the idle lens counts it, and every
    // other county still has its value.
    let text = v2_text();
    let from = "(holder: \"county.bdf.desk.capacity\", goods: [(\"coin\", ";
    let at = text.find(from).expect("Bedfordshire's capacity desk");
    let line_end = at + text[at..].find(')').expect("its coin") + 1;
    let rest = &text[line_end..];
    let horse = rest.find("(\"horse\", ").expect("its horses") + "(\"horse\", ".len();
    let end = horse + rest[horse..].find(')').expect("the number");
    let heads: f64 = rest[horse..end].parse().expect("a number");
    let mut edited = String::with_capacity(text.len());
    edited.push_str(&text[..line_end]);
    edited.push_str(&rest[..horse]);
    edited.push_str(&format!("{:?}", 3.0 * heads));
    edited.push_str(&rest[end..]);
    let tape = Tape::from_ron(&edited).expect("the edited tape parses");
    let store = record(&tape, 12);
    let l = lens("price.horse");
    let mut idle_ticks = 0;
    for t in 0..12 {
        let vm = lens_vm(&store, l, t);
        let bdf = vm
            .regions
            .iter()
            .find(|r| r.key == "county.bdf")
            .expect("Bedfordshire");
        let others = vm.regions.iter().filter(|r| r.key != "county.bdf");
        assert!(others.clone().all(|r| r.value.is_some()), "at {t}");
        if bdf.value.is_none() {
            idle_ticks += 1;
            let why = bdf.why.as_deref().unwrap_or_default();
            assert!(
                why.starts_with("idle: no horse traded this tick; the posted price is "),
                "{why}"
            );
            assert_eq!(bdf.at, None);
            assert!(vm.ranked.iter().all(|&i| vm.regions[i].key != "county.bdf"));
        }
    }
    assert!(idle_ticks > 0, "no idle tick in Bedfordshire");
    let idle = lens_vm(&store, lens("idle.horse"), 11);
    let bdf = idle
        .regions
        .iter()
        .find(|r| r.key == "county.bdf")
        .expect("Bedfordshire");
    assert_eq!(bdf.value, Some(f64::from(idle_ticks)));
    // The legend marks the reservation, read from the run.
    let vm = lens_vm(&store, l, 11);
    assert_eq!(vm.marks.len(), 1);
    assert_eq!(vm.marks[0].value, 0.25);
    assert_eq!(vm.marks[0].at, Some(0.125));
}

/// The map pane alone, drawing `store` under the second pass's lens table.
struct Pane1 {
    map: MapState,
    cursor: Option<u64>,
    selection: Option<Entity>,
    out: Vec<Intent>,
}

fn map_harness(store: &'static Store, size: egui::Vec2) -> Harness<'static, Pane1> {
    let mut map = MapState::default();
    map.ready_for(wl::V2_TAPE)
        .expect("the atlas and the lenses read");
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
fn the_map_takes_the_lens_table_of_its_tape() {
    // WORLD-V2 §7.2: the map shows the second pass's 36 lenses on its tape and v1's 25 on v1's;
    // the opening lens is the wage in land on both.
    let mut m = MapState::default();
    assert_eq!(m.lenses_for(wl::V2_TAPE).expect("v2").len(), 36);
    assert_eq!(m.lenses_for(wl::V1_TAPE).expect("v1").len(), 25);
    assert_eq!(m.lenses_for(&v2_tape().header.name).expect("v2").len(), 36);
    assert_eq!(m.lens, ui_map::DEFAULT_LENS);
    assert!(lenses().iter().any(|l| l.key == ui_map::DEFAULT_LENS));
    let mut h = map_harness(v2_store(), egui::vec2(1600.0, 900.0));
    h.state_mut().cursor = Some(51);
    h.run();
    assert_eq!(
        h.state().map.frame().lens.as_deref(),
        Some(ui_map::DEFAULT_LENS)
    );
}

#[test]
fn v2_map_values_equal_table() {
    // G4's gate on the second pass: under every lens at report ticks 0, 51 and 259, each row of
    // the ranked table shows the value, rank and colour of the region the map drew.
    let store = v2_store();
    let mut h = map_harness(store, egui::vec2(1600.0, 1200.0));
    for t in [0_u64, 51, 259] {
        for l in lenses() {
            h.state_mut().cursor = Some(t);
            h.state_mut().map.lens = l.key.clone();
            h.run();
            let f = h.state().map.frame().clone();
            assert_eq!(f.lens.as_deref(), Some(l.key.as_str()));
            let drawn: BTreeMap<&str, &ui_map::DrawnRegion> =
                f.regions.iter().map(|r| (r.key.as_str(), r)).collect();
            for row in &f.rows {
                let r = drawn[row.key.as_str()];
                assert_eq!(r.value, Some(row.value), "{} {} at {t}", l.key, row.key);
                assert_eq!(r.colour, row.colour, "{} {} at {t}", l.key, row.key);
            }
        }
    }
}

#[test]
fn the_v2_map_is_fitted_clear_and_its_values_whole() {
    // G1.6's and D2.1's layout checks on the second pass's store: every region's painted box
    // clear of the legend and the credit; under the table's longest name and unit, every value
    // the ranked table shows painted whole across, down to a 480-point window.
    let store = v2_store();
    let longest = lenses()
        .iter()
        .max_by_key(|l| l.name.len() + l.unit.len())
        .expect("a lens");
    for (w, hgt) in [
        (1600.0, 900.0),
        (1280.0, 800.0),
        (1024.0, 768.0),
        (700.0, 768.0),
        (480.0, 600.0),
    ] {
        let mut h = map_harness(store, egui::vec2(w, hgt));
        h.state_mut().cursor = Some(51);
        h.state_mut().map.lens = longest.key.clone();
        h.run();
        let f = h.state().map.frame().clone();
        let legend = f.legend.rect.expect("the legend");
        let credit = f.credit_rect.expect("the credit");
        for r in &f.regions {
            let b = r.rect.expect("a painted box");
            assert!(
                !b.intersects(legend) && !b.intersects(credit),
                "{} at {w}",
                r.key
            );
        }
        let texts = painted_texts(&h);
        let window = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, hgt));
        let mut whole = 0;
        for row in &f.rows {
            let Some((t, r, clip)) = texts.iter().find(|(t, _, _)| {
                t == &row.text || t.strip_suffix(" ↓").or(t.strip_suffix(" ↑")) == Some(&row.text)
            }) else {
                continue;
            };
            if r.min.y < clip.min.y - 0.5 || r.max.y > clip.max.y + 0.5 {
                continue;
            }
            whole += 1;
            let across = |a: egui::Rect| a.min.x <= r.min.x + 0.5 && r.max.x <= a.max.x + 0.5;
            assert!(
                across(*clip) && across(window),
                "at {w} under {:?}: {t:?} at {r:?}, clip {clip:?}",
                longest.name
            );
        }
        assert!(whole > 5, "at {w}: {whole} rows whole");
    }
}

/// The 17 signed gaps ln(o/o*) of a county's truth, in the gap lens's order.
fn gap_terms(x: &Truth) -> Vec<f64> {
    let e = oracle(x);
    let tl = ClockMethod::Flow
        .per_tick(&v2_tape_clock(), x.param["land"])
        .expect("a flow");
    let (w, rr, pf, pk, ph, p) = (
        x.price[0], x.price[1], x.price[2], x.price[3], x.price[4], x.price[5],
    );
    [
        (w / rr, e.v),
        (pf / rr, e.pf),
        (pk / rr, e.pk),
        (ph / rr, e.ph),
        (p / rr, e.p),
        (x.used, e.one_minus_x),
        (x.cleared[0], e.n_a),
        (x.cleared[1], tl),
        (x.cleared[2], e.qf),
        (x.cleared[4], e.task_hours),
        (x.cleared[5], e.good),
        (x.out_good, e.good),
        (x.out_hours, e.task_hours),
        (x.out_horse, e.made),
        (x.out_fodder, e.qf),
        (x.held, e.capacity),
        (x.serving, e.serving),
    ]
    .iter()
    .map(|(o, s)| rustyecon_engine::num::ln(o / s))
    .collect()
}

/// Every text shape the last frame painted: its string, its rect on screen and the rect it is
/// clipped to.
fn painted_texts(h: &Harness<'static, Pane1>) -> Vec<(String, egui::Rect, egui::Rect)> {
    fn walk(s: &egui::Shape, clip: egui::Rect, out: &mut Vec<(String, egui::Rect, egui::Rect)>) {
        match s {
            egui::Shape::Text(t) => {
                out.push((t.galley.text().to_string(), t.visual_bounding_rect(), clip));
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, clip, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in &h.output().shapes {
        walk(&c.shape, c.clip_rect, &mut out);
    }
    out
}
