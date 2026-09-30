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
//!
//! Added at D2.5, after the second pass's verification (each fails on the mutant or the defect
//! it names; `demo_v2_lens_values_equal_the_engine` now holds the card's causes by name and
//! sign, and the layout test each ranked value to one line):
//!
//! - `the_v2_app_shows_every_lens_and_a_countys_causes`: the whole app on the v2 tape.
//! - `the_v2_sidebar_and_card_are_whole_at_the_apps_window_sizes`: 1,024 × 768 and 1,280 × 800.
//! - `the_v2_legend_title_and_scale_bar_are_whole`, `the_legend_marks_psi_where_the_run_puts_it`.
//! - `the_card_paints_its_causes_and_the_herds_readings`.
//! - `the_horse_price_and_its_windows_follow_the_market_tick_by_tick`: ψ raised to 1.2 from
//!   February to April 1750, so every maker withholds and trades again.
//! - `the_fitted_view_follows_a_resized_window`.
//! - `the_v2_map_paints_its_lens_colours_and_its_credit`: G1.6's checks on the v2 store.
//! - `every_glyph_the_map_paints_is_in_its_font`: v1's table and the second pass's.
//! - `lens_v2_domains_hold_the_engines_long_run` (ignored, run by name in scripts/gui.sh): every
//!   lens, every county, every tick to 1901, inside its domain.

mod common;

use certify::Build;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::{maker_reservation, Spec};
use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::model::Intent;
use rustyecon_gui::run::{Catalogue, Cmd, Entity, Obs, Origin, RunStatus, Runner, Store};
use rustyecon_gui::ui::map::{self as ui_map, MapState};
use rustyecon_gui::vm::map::{self as vm_map, LensVm};
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::chain::{self, ChainParams, RuleA};
use rustyecon_worldgen::lens::{self as wl, NO_TRADE_WINDOW};
use rustyecon_worldgen::tables::Lens;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

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

/// Every county's truth at every report tick up to `last`, from a Sim of the v2 tape stepped
/// here.
fn engine_truth(last: u64) -> Vec<BTreeMap<String, Truth>> {
    engine_truth_of(v2_tape(), last)
}

/// Every county's truth at every report tick up to `last`, from a Sim of `tape` stepped here.
fn engine_truth_of(tape: &Tape, last: u64) -> Vec<BTreeMap<String, Truth>> {
    let mut sim = Sim::new(tape).expect("the tape loads");
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
            // What carries its gap: the three observables furthest from 1g at the Sim's params,
            // each by its name and with its sign (D2.5: only |gap| was held, so a card with the
            // signs flipped or the names lost passed).
            assert_eq!(card.causes.len(), 3, "{k} at {t}");
            let x = &truth[t as usize][k];
            let mut named: Vec<(&str, f64)> = OBSERVED.iter().copied().zip(gap_terms(x)).collect();
            named.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
            for (c, (name, gap)) in card.causes.iter().zip(&named) {
                assert_eq!(c.observable, *name, "{k} at {t}: {c:?}");
                assert!(
                    (c.gap - gap).abs() <= 1e-12,
                    "{k} at {t}: {c:?} against {name} {gap}"
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
    // D2.5 adds the herd against its equilibrium at genesis, where every county rests at its
    // point and some values are rounding's (2.22045e-16): each value is painted on one line.
    let store = v2_store();
    let longest = lenses()
        .iter()
        .max_by_key(|l| l.name.len() + l.unit.len())
        .expect("a lens");
    for (key, tick) in [(longest.key.as_str(), 51), ("horses.vs.oracle", 0)] {
        for (w, hgt) in [
            (1600.0, 900.0),
            (1280.0, 800.0),
            (1024.0, 768.0),
            (700.0, 768.0),
            (480.0, 600.0),
        ] {
            let mut h = map_harness(store, egui::vec2(w, hgt));
            h.state_mut().cursor = Some(tick);
            h.state_mut().map.lens = key.to_string();
            h.run();
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
            let texts = common::mapcheck::texts(&h.output().shapes);
            let window = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, hgt));
            let is_row = |t: &str, row: &ui_map::DrawnRow| {
                t == row.text || t.strip_suffix(" ↓").or(t.strip_suffix(" ↑")) == Some(&row.text)
            };
            let mut whole = 0;
            for row in &f.rows {
                let Some(t) = texts.iter().find(|t| is_row(&t.text, row)) else {
                    continue;
                };
                if !t.whole_down() {
                    continue;
                }
                whole += 1;
                assert!(
                    t.across(t.clip) && t.across(window),
                    "at {w} under {key}: {t:?}"
                );
            }
            assert!(whole > 5, "at {w} under {key}: {whole} rows whole");
            for t in &texts {
                if f.rows.iter().any(|row| is_row(&t.text, row)) {
                    assert_eq!(
                        t.rows, 1,
                        "at {w} under {key}: {:?} over {} rows",
                        t.text, t.rows
                    );
                }
            }
        }
    }
}

/// P2.2a's observables as HORSES-SPEC §7.1 names them, but the horse market's volume, in the
/// order of [`gap_terms`].
const OBSERVED: [&str; 17] = [
    "v",
    "pi.fodder",
    "pi.horse",
    "pi.traction",
    "pi.good",
    "s.good",
    "vol.labour",
    "vol.land",
    "vol.fodder",
    "vol.traction",
    "vol.good",
    "y.good",
    "y.traction",
    "y.horse",
    "y.fodder",
    "heads.capacity",
    "heads.maker",
];

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

#[test]
#[ignore = "a measurement: run by name, with --release"]
fn oracle_gap_cost_is_recorded() {
    // O86 (WORLD-V2 §7.1): the gap lens's cost. The first map of gap.oracle at a report tick
    // solves 1g for each county whose params it has not seen; a second is served from the
    // memo. Each solve's cost is timed over every county at genesis.
    let store = v2_store();
    let l = lens("gap.oracle");
    let t0 = std::time::Instant::now();
    let cold = lens_vm(store, l, 259);
    let cold_ms = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = std::time::Instant::now();
    let warm = lens_vm(store, l, 259);
    let warm_ms = t1.elapsed().as_secs_f64() * 1e3;
    assert_eq!(cold, warm);
    let truth = engine_truth(0);
    let t2 = std::time::Instant::now();
    for x in truth[0].values() {
        let _ = oracle(x);
    }
    let per = t2.elapsed().as_secs_f64() * 1e6 / truth[0].len() as f64;
    println!(
        "gap.oracle at report tick 259: {cold_ms:.1} ms cold, {warm_ms:.1} ms from the memo; one \
         1g solve {per:.0} µs, so a run's 25,573 about {:.1} s",
        per * 25_573.0 / 1e6
    );
}

// D2.5 (2026-09-30): the fix round after the bounded verification of the map (the map review's
// three majors and its minors; docs/demo/WORLD-V2.md §16). Each test below fails on the mutant
// or the defect it names, checked with the fix undone.

fn geo() -> &'static ui_map::Geo {
    static G: OnceLock<ui_map::Geo> = OnceLock::new();
    G.get_or_init(|| ui_map::Geo::build(atlas().clone()))
}

fn v1_path() -> String {
    format!("{}/../../tapes/demo-gb.ron", env!("CARGO_MANIFEST_DIR"))
}

/// Step frames until `done` holds of the app, five minutes at most.
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

/// The whole app opened on the tape at `path` in a window of `size`, loaded and paused.
fn app(path: String, size: egui::Vec2) -> Harness<'static, GuiApp> {
    let mut h = Harness::builder().with_size(size).build_eframe(move |cc| {
        GuiApp::new(
            &cc.egui_ctx,
            Launch {
                tape: Some(path),
                files: None,
                smoke: None,
            },
        )
    });
    step_until(&mut h, "the tape loaded", |a| {
        a.model().focused().map(|r| r.store.status())
            == Some(RunStatus::Paused { tick: 0, why: None })
    });
    h.step();
    h.step();
    h
}

/// Run the app's run until `until` ticks have run, and draw.
fn run_to(h: &mut Harness<'_, GuiApp>, until: u64) {
    h.state_mut().act(Intent::Run { until: Some(until) });
    step_until(h, "the run reached its tick", |a| {
        matches!(
            a.model().focused().map(|r| r.store.status()),
            Some(RunStatus::Paused { tick, why: Some(_) }) if tick == until
        )
    });
    h.step();
    h.step();
}

/// Press `]` until the map shows the lens `key`.
fn show_lens(h: &mut Harness<'_, GuiApp>, key: &str) {
    for _ in 0..40 {
        if h.state().ui_state().map.frame().lens.as_deref() == Some(key) {
            break;
        }
        h.key_press(egui::Key::CloseBracket);
        h.step();
    }
    h.step();
    h.step();
    assert_eq!(h.state().ui_state().map.frame().lens.as_deref(), Some(key));
}

/// Step until the frame paints a text that contains `text`, 20 frames at most.
fn paints(h: &mut Harness<'_, GuiApp>, text: &str) {
    for _ in 0..20 {
        if common::mapcheck::texts(&h.output().shapes)
            .iter()
            .any(|t| t.text.contains(text))
        {
            return;
        }
        h.step();
    }
    panic!("the frame paints no {text:?}");
}

/// The selected county's card, as the view-model makes it from the app's own record.
fn card_of(h: &Harness<'_, GuiApp>, county: &str, lens: &str) -> vm_map::CountyVm {
    let run = h.state().model().focused().expect("a run");
    let sel = Entity::Node(common::key(county));
    vm_map::build(
        &run.store,
        run.origin,
        atlas(),
        lenses(),
        lens,
        Some(&sel),
        None,
    )
    .and_then(|v| v.county)
    .expect("the card")
}

/// The lines the card paints for what carries its gap and for the herd's three readings.
fn card_lines(card: &vm_map::CountyVm) -> Vec<String> {
    let mut lines: Vec<String> = card
        .causes
        .iter()
        .map(|c| format!("  {} {:+.3}", c.observable, c.gap))
        .collect();
    for key in ["horses.vs.oracle", "horses.vs.plan", "hday.markup"] {
        let l = card
            .lenses
            .iter()
            .find(|l| l.key == key)
            .expect("the herd's lens");
        lines.push(format!("  {}: {:+.3}", l.name, l.value.expect("a value")));
    }
    lines
}

#[test]
fn the_v2_app_shows_every_lens_and_a_countys_causes() {
    // The map review's third major (V8): the app takes the lens table of its tape's name, and no
    // test ran the whole app on the second pass's tape, so an app that always took v1's 25
    // lenses passed. Here the app opens the tape and holds its 36 lenses, the chain's group
    // first; a year runs; `]` steps through every lens, each painting its name over 93 counties
    // with a value; `1` picks the herd per head; and a click on Lancashire selects it, whose card
    // paints what carries its gap and the herd's three readings.
    let mut h = app(v2_path(), egui::vec2(1600.0, 1000.0));
    let (_, held) = h
        .state()
        .ui_state()
        .map
        .parts()
        .expect("the atlas and the lenses read");
    assert_eq!(held, lenses(), "the app holds the second pass's table");
    let items = vm_map::items(held);
    assert_eq!(items.len(), 36);
    assert!(items[..11].iter().all(|i| i.group == "Horses and fodder"));
    paints(&mut h, "Wage in land");
    run_to(&mut h, 52);
    let n = lenses().len();
    let first = lenses()
        .iter()
        .position(|l| l.key == ui_map::DEFAULT_LENS)
        .expect("the opening lens is listed");
    for k in 0..n {
        let want = &lenses()[(first + k + 1) % n];
        h.key_press(egui::Key::CloseBracket);
        h.step();
        h.step();
        let f = h.state().ui_state().map.frame().clone();
        assert_eq!(f.lens.as_deref(), Some(want.key.as_str()));
        assert_eq!(f.regions.len(), 93);
        let none: Vec<&str> = f
            .regions
            .iter()
            .filter(|r| r.value.is_none())
            .map(|r| r.key.as_str())
            .collect();
        assert!(none.is_empty(), "{}: no value at {none:?}", want.key);
        paints(&mut h, &want.name);
    }
    h.key_press(egui::Key::Num1);
    h.step();
    h.step();
    assert_eq!(
        h.state().ui_state().map.frame().lens.as_deref(),
        Some("horses.per.head")
    );
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
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: lan,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.step();
    step_until(&mut h, "Lancashire selected", |a| {
        a.model().selection() == Some(&Entity::Node(common::key("county.lan")))
    });
    h.remove_cursor();
    paints(&mut h, "Its gap, ln(observed / equilibrium):");
    let card = card_of(&h, "county.lan", "horses.per.head");
    assert_eq!(card.causes.len(), 3);
    for line in card_lines(&card) {
        paints(&mut h, &line);
    }
}

/// Every text the frame painted in the map's sidebar that is cut across: one that starts in the
/// sidebar and lies whole from top to bottom in its clip (a row a scroll area shows in part is
/// cut there by design), but not whole from left to right in its clip, the sidebar or the
/// window.
fn cut_in_sidebar(texts: &[common::mapcheck::Painted], side: egui::Rect, w: f32) -> Vec<String> {
    let within = egui::Rect::from_min_max(side.min, egui::pos2(side.max.x.min(w), side.max.y));
    let starts_in = |t: &common::mapcheck::Painted| {
        (side.min.x - 0.5..side.max.x).contains(&t.rect.min.x)
            && (side.min.y - 0.5..side.max.y).contains(&t.rect.min.y)
    };
    texts
        .iter()
        .filter(|t| starts_in(t) && t.whole_down())
        .filter(|t| !(t.across(t.clip) && t.across(within)))
        .map(|t| format!("{:?} at {:?} clip {:?}", t.text, t.rect, t.clip))
        .collect()
}

#[test]
fn the_v2_sidebar_and_card_are_whole_at_the_apps_window_sizes() {
    // The map review's first major: at the app's own window sizes the card's header and lines
    // ran past the sidebar, and the lens's note, the card's causes and the herd's readings
    // pushed the ranked table off the pane (O26's card note, left open by O39). Now the card and
    // the lens's description scroll above a ranked table that keeps its rows, and every line
    // wraps to the sidebar. Read from the shapes the whole app painted at 1,024 × 768 and
    // 1,280 × 800 with Lancashire selected: no text of the sidebar is cut across, under three
    // lenses and with the card's two lists opened; the card's header and its button are
    // painted; what carries the gap and the herd's three readings are painted whole; and the
    // ranked table shows rows whole.
    for (w, hgt) in [(1024.0, 768.0), (1280.0, 800.0)] {
        let mut h = app(v2_path(), egui::vec2(w, hgt));
        run_to(&mut h, 260);
        h.state_mut()
            .act(Intent::Select(Some(Entity::Node(common::key(
                "county.lan",
            )))));
        for _ in 0..4 {
            h.step();
        }
        for lens in ["price.horse", "horses.vs.oracle", "gap.oracle"] {
            show_lens(&mut h, lens);
            // The card's top in view: the scroll area keeps its offset from lens to lens.
            h.get_by_label("×").scroll_to_me();
            for _ in 0..3 {
                h.step();
            }
            let f = h.state().ui_state().map.frame().clone();
            let side = f.sidebar.expect("the sidebar");
            let texts = common::mapcheck::texts(&h.output().shapes);
            let cut = cut_in_sidebar(&texts, side, w);
            assert!(cut.is_empty(), "at {w} × {hgt} under {lens}: cut {cut:#?}");
            let shown = |texts: &[common::mapcheck::Painted], s: &str| {
                texts
                    .iter()
                    .any(|t| t.text == s && t.whole_down() && t.rect.min.x >= side.min.x - 1.0)
            };
            assert!(
                shown(&texts, "Lancashire"),
                "at {w} under {lens}: the card's name"
            );
            assert!(shown(&texts, "×"), "at {w} under {lens}: the card's button");
            // The ranked table keeps its rows below the card.
            let whole = f
                .rows
                .iter()
                .filter(|row| {
                    texts.iter().any(|t| {
                        (t.text == row.text || t.text.starts_with(&format!("{} ", row.text)))
                            && t.whole_down()
                            && t.rect.min.x >= side.min.x - 1.0
                    })
                })
                .count();
            println!("at {w} × {hgt} under {lens}: {whole} ranked rows whole");
            assert!(
                whole >= 4,
                "at {w} × {hgt} under {lens}: {whole} ranked rows whole"
            );
            // What carries the gap is in view with the card's top; the herd's readings are,
            // or are brought into view by scrolling the card, and are then whole.
            let lines = card_lines(&card_of(&h, "county.lan", lens));
            for line in &lines[..3] {
                assert!(
                    shown(&texts, line),
                    "at {w} × {hgt} under {lens}: {line:?} not shown whole"
                );
            }
            for line in &lines[3..] {
                if !shown(&texts, line) {
                    h.get_by_label_contains(line.trim()).scroll_to_me();
                    for _ in 0..3 {
                        h.step();
                    }
                }
                let texts = common::mapcheck::texts(&h.output().shapes);
                assert!(
                    shown(&texts, line),
                    "at {w} × {hgt} under {lens}: {line:?} not shown whole, scrolled to"
                );
                let cut = cut_in_sidebar(&texts, side, w);
                assert!(
                    cut.is_empty(),
                    "at {w} × {hgt} under {lens}, scrolled: cut {cut:#?}"
                );
            }
        }
        for list in ["Every lens", "Recorded inputs"] {
            h.get_by_label(list).click();
            for _ in 0..4 {
                h.step();
            }
        }
        h.remove_cursor();
        h.step();
        let f = h.state().ui_state().map.frame().clone();
        let texts = common::mapcheck::texts(&h.output().shapes);
        let cut = cut_in_sidebar(&texts, f.sidebar.expect("the sidebar"), w);
        assert!(
            cut.is_empty(),
            "at {w} × {hgt}, the lists open: cut {cut:#?}"
        );
    }
}

/// The legend, the title and the scale bar of a frame against its canvas: the legend's header
/// painted inside its box; ψ, where marked, in its row between the header and the bar; the
/// domain's ends written; every label inside the box; the title's lines inside the canvas; and
/// the scale bar clear of the legend and the credit.
fn check_legend(f: &ui_map::MapFrame, texts: &[common::mapcheck::Painted], what: &str) {
    let lg = &f.legend;
    let r = lg.rect.expect("the legend was drawn");
    let canvas = f.canvas.expect("the canvas");
    let inside = |a: egui::Rect, b: egui::Rect| a.expand(0.5).contains_rect(b);
    let head = texts
        .iter()
        .find(|t| t.text == lg.head && inside(r, t.rect))
        .unwrap_or_else(|| panic!("{what}: the header {:?} is not painted in its box", lg.head));
    assert!(
        inside(canvas, head.rect) && inside(head.clip, head.rect),
        "{what}: {head:?}"
    );
    if !lg.named.is_empty() {
        let psi = texts
            .iter()
            .find(|t| t.text == "ψ" && inside(r, t.rect))
            .unwrap_or_else(|| panic!("{what}: ψ is not painted in the legend"));
        assert!(!psi.rect.intersects(head.rect), "{what}: ψ over the header");
    }
    let (lo, hi) = (lg.marks[0].0, lg.marks[lg.marks.len() - 1].0);
    assert!(
        lg.labelled.contains(&lo) && lg.labelled.contains(&hi),
        "{what}: the ends {lo} and {hi} not both written: {:?}",
        lg.labelled
    );
    for v in &lg.labelled {
        let label = rustyecon_gui::ui::fmt(*v);
        let t = texts
            .iter()
            .find(|t| t.text == label && inside(r, t.rect))
            .unwrap_or_else(|| panic!("{what}: the label {label} is not painted in the box"));
        assert!(inside(t.clip, t.rect), "{what}: {t:?}");
    }
    let lens = f.lens.as_deref().expect("a lens");
    let l = lenses().iter().find(|l| l.key == lens).expect("the lens");
    let unit = format!("{} · report tick", l.unit);
    let title: Vec<_> = texts
        .iter()
        .filter(|t| t.rect.min.x < canvas.max.x)
        .filter(|t| t.text == l.name || t.text.starts_with(&unit))
        .collect();
    assert_eq!(title.len(), 2, "{what}: the title's two lines");
    for t in title {
        assert!(
            inside(canvas, t.rect) && inside(t.clip, t.rect),
            "{what}: {t:?}"
        );
    }
    let bar = f.scale_bar.expect("the scale bar");
    let credit = f.credit_rect.expect("the credit");
    assert!(
        !bar.intersects(r) && !bar.intersects(credit),
        "{what}: the scale bar {bar:?} over the legend {r:?} or the credit {credit:?}"
    );
}

#[test]
fn the_v2_legend_title_and_scale_bar_are_whole() {
    // The map review's fourth minor: at the app's sizes the legend's header ran past its box, ψ
    // was painted over it, the scale bar lay over the legend on a narrow canvas, the title's
    // unit line was cut, and the gap lens's 10 and its domain's top went unwritten. Read from
    // the shapes painted, in the whole app at 1,024 × 768 and 1,280 × 800 and in the map pane
    // alone down to a 480-point window, under the horse's price (ψ), the gap lens (a log scale
    // from 1 to 1,000), the herd against its equilibrium and population (a log scale to
    // 10,000).
    let keys = [
        "price.horse",
        "gap.oracle",
        "horses.vs.oracle",
        "param.workers",
    ];
    for (w, hgt) in [(1024.0, 768.0), (1280.0, 800.0)] {
        let mut h = app(v2_path(), egui::vec2(w, hgt));
        run_to(&mut h, 260);
        for key in keys {
            show_lens(&mut h, key);
            let f = h.state().ui_state().map.frame().clone();
            let texts = common::mapcheck::texts(&h.output().shapes);
            check_legend(&f, &texts, &format!("the app at {w} × {hgt}, {key}"));
        }
    }
    for (w, hgt) in [(1600.0, 900.0), (700.0, 768.0), (480.0, 600.0)] {
        let mut h = map_harness(v2_store(), egui::vec2(w, hgt));
        h.state_mut().cursor = Some(51);
        for key in keys {
            h.state_mut().map.lens = key.to_string();
            h.run();
            let f = h.state().map.frame().clone();
            let texts = common::mapcheck::texts(&h.output().shapes);
            check_legend(&f, &texts, &format!("the pane at {w} × {hgt}, {key}"));
            if key == "gap.oracle" {
                // §8 item 1's bands: the tolerance, 1%, 10% and 100% in log.
                for v in [1.0, 10.0, 100.0, 1000.0] {
                    assert!(
                        f.legend.labelled.contains(&v),
                        "at {w}: {v} is not written: {:?}",
                        f.legend.labelled
                    );
                }
            }
        }
    }
}

#[test]
fn the_legend_marks_psi_where_the_run_puts_it() {
    // The map review's third minor (V9, V10): nothing read the ψ mark as painted, so a mark
    // mirrored along the bar or not painted at all passed. ψ is 0.25 on a domain of 0 to 2, an
    // eighth along the bar; its letter is painted just right of the mark, in the legend's box.
    let mut h = map_harness(v2_store(), egui::vec2(1600.0, 900.0));
    h.state_mut().cursor = Some(51);
    h.state_mut().map.lens = "price.horse".to_string();
    h.run();
    let f = h.state().map.frame().clone();
    let lg = &f.legend;
    assert_eq!(lg.named.len(), 1, "{:?}", lg.named);
    assert!((lg.named[0] - 0.125).abs() < 1e-4, "{:?}", lg.named);
    let r = lg.rect.expect("the legend");
    let (bar_x, bar_w) = (r.min.x + 12.0, r.width() - 24.0);
    let psi = common::mapcheck::texts(&h.output().shapes)
        .into_iter()
        .find(|t| t.text == "ψ" && r.contains_rect(t.rect))
        .expect("ψ is painted in the legend");
    let at = bar_x + 0.125 * bar_w;
    assert!(
        psi.rect.min.x > at && psi.rect.min.x < at + 6.0,
        "ψ at {:?}, the mark at {at}",
        psi.rect
    );
    // Only the horse's price marks ψ.
    h.state_mut().map.lens = "horses.vs.oracle".to_string();
    h.run();
    assert!(h.state().map.frame().legend.named.is_empty());
}

#[test]
fn the_card_paints_its_causes_and_the_herds_readings() {
    // The map review's third minor (V11): nothing read the card's lines as painted, so a card
    // that dropped two of the herd's three readings passed. Lancashire's card, at report tick
    // 259, paints the three observables that carry its gap with their signs, and the herd
    // against its equilibrium, against the desk's target, and the horse-day's markup.
    let store = v2_store();
    let mut h = map_harness(store, egui::vec2(1600.0, 1200.0));
    h.state_mut().cursor = Some(259);
    h.state_mut().selection = Some(Entity::Node(common::key("county.lan")));
    h.run();
    h.run();
    let sel = Entity::Node(common::key("county.lan"));
    let card = vm_map::build(
        store,
        Origin::Run,
        atlas(),
        lenses(),
        ui_map::DEFAULT_LENS,
        Some(&sel),
        Some(259),
    )
    .and_then(|v| v.county)
    .expect("the card");
    let texts = common::mapcheck::texts(&h.output().shapes);
    let lines = card_lines(&card);
    assert_eq!(lines.len(), 6);
    for line in lines {
        assert!(
            texts.iter().any(|t| t.text == line && t.whole_down()),
            "{line:?} is not painted"
        );
    }
}

#[test]
fn the_horse_price_and_its_windows_follow_the_market_tick_by_tick() {
    // The map review's third minor (V2, V3): the horse's price was held with no value only where
    // the market idled throughout, and the reservation's window only where no maker withheld,
    // so a price read a tick early and a ψ scaled per tick passed. Here the tape raises ψ to 1.2
    // from February to April 1750: every maker withholds, no horse trades, and then the market
    // trades again. At every report tick to April, for every county, the horse's price has a
    // value exactly when the engine's horse market traded, and it is the maker's markup by its
    // rule; the idle lens counts the engine's idle ticks, and the reservation's lens the ticks
    // the maker's own rule withheld.
    let text = v2_text();
    let anchor = "(key: \"reserve.maker\", value: 0.25,";
    let at = text.find(anchor).expect("the reservation's param");
    let line = at + text[at..].find('\n').expect("its line") + 1;
    let events = text.find("    events: [\n").expect("the events") + "    events: [\n".len();
    assert!(line < events);
    let basis = "basis: Assumed(\"D2.5 test: psi raised and lowered\")";
    let mut edited = String::with_capacity(text.len() + 1024);
    edited.push_str(&text[..line]);
    for (k, v) in [("hi", "1.2"), ("lo", "0.25")] {
        edited.push_str(&format!(
            "        (key: \"reserve.maker.test.{k}\", value: {v}, unit: Dimensionless, {basis}),\n"
        ));
    }
    edited.push_str(&text[line..events]);
    for (k, date) in [("hi", "1750-02-01"), ("lo", "1750-04-01")] {
        edited.push_str(&format!(
            "        (key: \"test.psi.{k}\", at: \"{date}\", {basis}, act: SetParam(param: \
             \"reserve.maker\", to: \"reserve.maker.test.{k}\")),\n"
        ));
    }
    edited.push_str(&text[events..]);
    let tape = Tape::from_ron(&edited).expect("the edited tape parses");
    let last = 25;
    let store = record(&tape, last + 1);
    let truth = engine_truth_of(&tape, last);
    let window = |k: &str, t: u64, f: &dyn Fn(&Truth) -> bool| -> f64 {
        let lo = (t + 1).saturating_sub(NO_TRADE_WINDOW);
        (lo..=t).filter(|&s| f(&truth[s as usize][k])).count() as f64
    };
    let (mut stops, mut starts, mut withheld) = (0, 0, 0);
    let mut wrong = Vec::new();
    for t in 0..=last {
        let price = lens_vm(&store, lens("price.horse"), t);
        let idle = lens_vm(&store, lens("idle.horse"), t);
        let reserve = lens_vm(&store, lens("reserve.ticks"), t);
        for ((p, i), r) in price
            .regions
            .iter()
            .zip(&idle.regions)
            .zip(&reserve.regions)
        {
            let k = p.key.as_str();
            let x = &truth[t as usize][k];
            let want = x.trades[3].then_some(x.markup);
            match (p.value, want) {
                (Some(a), Some(b)) if (a - b).abs() <= 1e-12 * b.abs() => {}
                (None, None) => {}
                (got, want) => wrong.push(format!("price.horse {k} at {t}: {got:?}, {want:?}")),
            }
            let idle_want = window(k, t, &|y| !y.trades[3]);
            if i.value != Some(idle_want) {
                wrong.push(format!("idle.horse {k} at {t}: {:?}, {idle_want}", i.value));
            }
            let reserve_want = window(k, t, &|y| y.withheld);
            if r.value != Some(reserve_want) {
                wrong.push(format!(
                    "reserve.ticks {k} at {t}: {:?}, {reserve_want}",
                    r.value
                ));
            }
            if t > 0 {
                let before = truth[t as usize - 1][k].trades[3];
                stops += usize::from(before && !x.trades[3]);
                starts += usize::from(!before && x.trades[3]);
            }
            withheld += usize::from(x.withheld);
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong: {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(12)]
    );
    println!("{stops} county-ticks stop trading, {starts} start again; {withheld} withheld");
    assert!(
        stops >= 93 && starts >= 93,
        "{stops} stops, {starts} starts"
    );
    assert!(withheld > 0);
}

#[test]
fn the_fitted_view_follows_a_resized_window() {
    // The map review's fifth minor: the view was fitted at the first frame and never again, so
    // shrinking the window put Cornwall and Devon under the legend until a double click. Now a
    // view nobody has moved is fitted afresh to a canvas of another size; a panned view is the
    // user's and stays, until the map is fitted again.
    let store = v2_store();
    let mut h = map_harness(store, egui::vec2(1600.0, 1000.0));
    h.state_mut().cursor = Some(51);
    h.run();
    assert!(h.state().map.is_fitted());
    let clear = |h: &Harness<'static, Pane1>, what: &str| {
        let f = h.state().map.frame().clone();
        let legend = f.legend.rect.expect("the legend");
        let credit = f.credit_rect.expect("the credit");
        let under: Vec<&str> = f
            .regions
            .iter()
            .filter(|r| {
                r.rect
                    .is_some_and(|b| b.intersects(legend) || b.intersects(credit))
            })
            .map(|r| r.key.as_str())
            .collect();
        assert!(
            under.is_empty(),
            "{what}: under the legend or credit {under:?}"
        );
    };
    for size in [egui::vec2(1024.0, 768.0), egui::vec2(700.0, 768.0)] {
        h.set_size(size);
        h.run();
        h.run();
        assert!(h.state().map.is_fitted());
        clear(&h, &format!("resized to {size:?}"));
    }
    common::mapcheck::pan(&mut h, egui::pos2(250.0, 300.0), egui::vec2(40.0, 20.0));
    assert!(!h.state().map.is_fitted(), "a pan is the user's view");
    let before = h.state().map.frame().regions[0].label;
    h.set_size(egui::vec2(1024.0, 768.0));
    h.run();
    h.run();
    assert!(!h.state().map.is_fitted(), "a resize keeps the user's view");
    let after = h.state().map.frame().regions[0].label;
    // The same view on a canvas whose centre moved: the map moves with the centre only.
    let canvas_shift = |w: f32| (w - (w * 0.4).min(270.0) - 6.0) / 2.0;
    let shift = canvas_shift(1024.0) - canvas_shift(700.0);
    assert!(
        ((after.x - before.x) - shift).abs() < 1.0,
        "{before:?} to {after:?}, the canvas's centre moved {shift}"
    );
    h.state_mut().map.fit();
    h.run();
    h.run();
    assert!(h.state().map.is_fitted());
    clear(&h, "fitted again at 1,024 × 768");
}

#[test]
fn the_v2_map_paints_its_lens_colours_and_its_credit() {
    // The map review's third major: G1.6's colour and credit checks ran on v1's store only
    // (WORLD-V2 §10; decision 338). On the second pass's store: the mesh built afresh (the first
    // frame, a pan, a switch of lens and another pan) paints each region in its lens colour, and
    // the legend's segments, marks and reference agree with the scale, for every lens; and the
    // atlas's credit is painted whole, seen, inside the canvas and clear of the legend, down to
    // a 480-point window, under the horse's price, whose legend carries ψ.
    let store = v2_store();
    let check = |h: &Harness<'static, Pane1>, what: &str| {
        let f = h.state().map.frame().clone();
        common::mapcheck::check_colours(
            &f,
            &h.output().shapes,
            store,
            lenses(),
            atlas(),
            geo(),
            what,
        );
    };
    let mut h = map_harness(store, egui::vec2(1600.0, 1000.0));
    h.state_mut().cursor = Some(51);
    h.run();
    check(&h, "the first frame");
    common::mapcheck::pan(&mut h, egui::pos2(600.0, 500.0), egui::vec2(30.0, 10.0));
    check(&h, "after a pan");
    h.state_mut().map.lens = "horses.vs.oracle".to_string();
    h.run();
    common::mapcheck::pan(&mut h, egui::pos2(600.0, 500.0), egui::vec2(-50.0, 25.0));
    check(&h, "a diverging lens after a pan");
    for l in lenses() {
        h.state_mut().map.lens = l.key.clone();
        h.run();
        check(&h, "every lens");
    }
    for (w, hgt) in [
        (1600.0, 900.0),
        (1280.0, 800.0),
        (1024.0, 768.0),
        (700.0, 768.0),
        (480.0, 600.0),
    ] {
        let mut h = map_harness(store, egui::vec2(w, hgt));
        h.state_mut().cursor = Some(51);
        h.state_mut().map.lens = "price.horse".to_string();
        h.run();
        let f = h.state().map.frame().clone();
        assert_eq!(f.credit, rustyecon_worldgen::atlas::CREDIT, "at {w}");
        let c = f.credit_rect.expect("the credit");
        let l = f.legend.rect.expect("the legend");
        let canvas = f.canvas.expect("the canvas");
        assert!(
            !c.intersects(l),
            "at {w}: the credit {c:?} over the legend {l:?}"
        );
        assert!(canvas.expand(0.5).contains_rect(c), "at {w}: {c:?}");
        let texts = common::mapcheck::texts(&h.output().shapes);
        for line in rustyecon_worldgen::atlas::CREDIT {
            let t = texts
                .iter()
                .find(|t| t.text == line)
                .unwrap_or_else(|| panic!("at {w} × {hgt}: {line:?} is not painted"));
            assert!(t.seen, "at {w}: {line:?} is painted unseen");
            assert!(t.clip.expand(0.5).contains_rect(t.rect), "at {w}: {t:?}");
        }
    }
}

/// The map pane alone, drawing `store` under the lens table of the tape named `tape`.
fn map_harness_of(store: &'static Store, size: egui::Vec2, tape: &str) -> Harness<'static, Pane1> {
    let mut map = MapState::default();
    map.ready_for(tape).expect("the atlas and the lenses read");
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

fn v1_store() -> &'static Store {
    static S: OnceLock<Store> = OnceLock::new();
    S.get_or_init(|| {
        let text = std::fs::read_to_string(v1_path()).expect("tapes/demo-gb.ron");
        record(&Tape::from_ron(&text).expect("v1 parses"), 52)
    })
}

#[test]
fn every_glyph_the_map_paints_is_in_its_font() {
    // The map review's fourth minor: the legend's reference mark `▏` and the card's `✕` are not
    // in egui's bundled fonts, and were painted as boxes in every screenshot since D.3. Every
    // character of every text the map pane paints, under every lens of v1's table and of the
    // second pass's, with a county selected and its two lists open, is in the font that paints
    // it.
    for (store, tape) in [(v2_store(), wl::V2_TAPE), (v1_store(), wl::V1_TAPE)] {
        let table = wl::for_tape(tape).expect("the table");
        let mut h = map_harness_of(store, egui::vec2(1600.0, 1400.0), tape);
        h.state_mut().cursor = Some(51);
        h.state_mut().selection = Some(Entity::Node(common::key("county.lan")));
        h.run();
        for list in ["Every lens", "Recorded inputs"] {
            h.get_by_label(list).click();
            h.run();
        }
        let mut missing = Vec::new();
        for l in &table {
            h.state_mut().map.lens = l.key.clone();
            h.run();
            missing.extend(common::mapcheck::missing_glyphs(&h.ctx, &h.output().shapes));
        }
        missing.sort();
        missing.dedup();
        assert!(missing.is_empty(), "{tape}: {missing:#?}");
    }
}

/// A Runner stepped by hand into a store, ingesting as it goes: the second pass to 1901.
fn record_long(tape: &Tape, ticks: u64) -> Store {
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
        loop {
            let busy = runner.advance(97).busy;
            let obs: Vec<Obs> = std::mem::take(&mut *seen.lock().unwrap());
            for o in obs {
                store.ingest(o).expect("the record takes it");
            }
            if !busy {
                break;
            }
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
    take(&mut runner, &mut store, Cmd::Step(ticks));
    assert_eq!(store.tick(), ticks);
    store
}

#[test]
#[ignore = "the engine's run to 1901, every tick: 2 to 5 minutes in release; scripts/gui.sh runs it by name"]
fn lens_v2_domains_hold_the_engines_long_run() {
    // The map review's second major: v1's domains and notes were kept for the second pass's
    // table because the oracle's range carried over, but the run departs further from its
    // oracle than v1's (D̂'s county medians' median 117 against 11), so rationing, the
    // horse-day's price, the good's price and the relief burden ran past their domains, and two
    // notes were false. From D2.5 the domains of the dynamic lenses hold the engine's own run to
    // 1901 with a margin (decision 332, amended). Here every lens of the table, for every county
    // at every report tick through the first of 1901, read from the GUI's record of the run as
    // the map reads it, lies inside its domain; the gap lens alone runs below its domain's low
    // end, 1 (the tolerance), where a county is within the tolerance, which its legend names.
    let t0 = Instant::now();
    let store = record_long(v2_tape(), 7852);
    let recorded = t0.elapsed().as_secs_f64();
    let w = store.world().expect("a world").clone();
    let measures: Vec<wl::Measure> = lenses()
        .iter()
        .map(|l| wl::Measure::of(&l.key).expect("a measure"))
        .collect();
    let nodes: Vec<String> = w.nodes.iter().map(|n| n.key.as_str().to_string()).collect();
    let firsts: Vec<wl::Readings> = nodes
        .iter()
        .map(|k| vm_map::readings(&store, k, 0))
        .collect();
    let n = lenses().len();
    let mut lo = vec![(f64::INFINITY, String::new()); n];
    let mut hi = vec![(f64::NEG_INFINITY, String::new()); n];
    let mut none = vec![0usize; n];
    for t in 0..7852_u64 {
        for (c, k) in nodes.iter().enumerate() {
            let x = vm_map::readings(&store, k, t);
            for (i, m) in measures.iter().enumerate() {
                match wl::value(*m, &x, Some(&firsts[c])) {
                    Ok(v) => {
                        if v < lo[i].0 {
                            lo[i] = (v, format!("{k} at {t}"));
                        }
                        if v > hi[i].0 {
                            hi[i] = (v, format!("{k} at {t}"));
                        }
                    }
                    Err(_) => none[i] += 1,
                }
            }
        }
    }
    let ln = rustyecon_engine::num::ln;
    let mut out = Vec::new();
    for (i, l) in lenses().iter().enumerate() {
        let (a, b) = l.domain;
        let span = |x: f64, y: f64| match l.scale {
            rustyecon_worldgen::tables::Scale::SequentialLog => (ln(y) - ln(x)) / (ln(b) - ln(a)),
            _ => (y - x) / (b - a),
        };
        let below = if l.key == "gap.oracle" {
            f64::NAN
        } else {
            span(a, lo[i].0)
        };
        let line = format!(
            "{:<22} domain [{a}, {b}]  run [{:.6} ({}), {:.6} ({})]  margins {below:.3} below, \
             {:.3} above; no value {}",
            l.key,
            lo[i].0,
            lo[i].1,
            hi[i].0,
            hi[i].1,
            span(hi[i].0, b),
            none[i]
        );
        println!("{line}");
        assert_eq!(none[i], 0, "{line}");
        assert!(hi[i].0 <= b, "{line}");
        if l.key != "gap.oracle" {
            assert!(a <= lo[i].0, "{line}");
        }
        out.push(line);
    }
    assert_eq!(out.len(), 36);
    println!(
        "recorded 7,852 ticks in {recorded:.1} s; read every lens for 93 counties at every tick \
         in {:.1} s",
        t0.elapsed().as_secs_f64() - recorded
    );
}
