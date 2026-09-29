//! The loop step's tapes and plants on the engine (P2.2b; docs/probe/LOOPS-RULES.md §7, §10):
//! each committed tape is its generator's output, loads and runs deterministically, and conserves
//! every tick, the plants' wear included; the oracle's point is a fixed point of the planted
//! roles' map at every built instance and funded target; with every plant at θ = 1 a planted
//! tape is the plant-free tape value for value; and a fixed plant (δ_p 0) is fixed-Q drs.
//!
//! Two bars are relative and named here: `COIN`, 1e-12 of the money stock, for the drift of the
//! total coin over a run, as in `horses.rs`; and `REST`, 1e-12 in log, for three ticks from the
//! oracle's f64 point (LOOPS-RULES §10).

use probe::horses::loops::{genesis, params, stocks, tape_ron, Config, Instance, Setup};
use rustyecon_core::{num, StateDelta};
use rustyecon_engine::prelude::{
    ActorId, Amount, GoodId, Holder, NodeId, Phase, Provenance, Sim, Tape, TickReport,
};
use rustyecon_engine::rustyecon_agents::{ActorState, PlantState};

const TAPES: [(&str, &str); 6] = [
    ("lb1", include_str!("../../../tapes/loops-lb1.ron")),
    ("lb2", include_str!("../../../tapes/loops-lb2.ron")),
    ("lb3", include_str!("../../../tapes/loops-lb3.ron")),
    ("lw1", include_str!("../../../tapes/loops-lw1.ron")),
    ("lw2", include_str!("../../../tapes/loops-lw2.ron")),
    ("lw3", include_str!("../../../tapes/loops-lw3.ron")),
];
const COIN: f64 = 1e-12;
const REST: f64 = 1e-12;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn registered(id: &str) -> Setup {
    Setup::registered(id, 52).expect("a registered instance")
}

fn key_of<I: rustyecon_core::Keyed>(s: &Sim, id: I) -> String {
    s.world().key_of(id).expect("a key").to_string()
}

fn actor(s: &Sim, key: &str) -> ActorId {
    s.world().id_of::<ActorId>(key).expect("an actor")
}

fn good(s: &Sim, key: &str) -> GoodId {
    s.world().id_of::<GoodId>(key).expect("a good")
}

fn held(s: &Sim, a: &str, g: &str) -> f64 {
    s.holding(Holder::Actor(actor(s, a)))
        .map_or(0.0, |i| i.get(good(s, g)))
}

fn plant_of(s: &Sim, a: &str) -> Option<PlantState> {
    match s.actor_state(actor(s, a)) {
        Some(ActorState::PlantedType(p)) => Some(p.plant),
        Some(ActorState::PlantedMaker(p)) => Some(p.plant),
        Some(ActorState::PlantedCapacity(p)) => Some(p.plant),
        _ => None,
    }
}

fn gap(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        num::ln(a / b).abs()
    }
}

#[test]
fn loops_tapes_are_their_generators_output() {
    // tapes/loops-<id>.ron is `horses-tape --inst <id>`'s output for the registered setup at 52
    // ticks a year, byte for byte, so its genesis is the oracle's point and nobody edits it.
    for (id, text) in TAPES {
        let generated = tape_ron(&registered(id)).expect("the generator runs");
        assert!(
            generated == text,
            "tapes/loops-{id}.ron differs from its generator: run `cargo run -p \
             rustyecon-probe --bin horses-tape -- --inst {id} tapes/loops-{id}.ron`"
        );
    }
    // Every built instance writes a tape that loads and builds its cast; LF4 and LN5 wait for
    // storable fodder (LOOP-SPEC-A2).
    for id in Instance::IDS {
        let text = tape_ron(&registered(id)).unwrap_or_else(|e| panic!("{id}: {e}"));
        sim(&text);
    }
    for id in ["lf4", "ln5"] {
        assert!(Instance::named(id).is_err(), "{id}");
        assert!(Instance::is_loop(id), "{id}");
    }
}

#[test]
fn loops_tapes_load_and_run_deterministically() {
    // R8: two runs of each tape give identical reports and hash streams, and the state moves.
    for (id, text) in TAPES {
        let (mut a, mut b) = (sim(text), sim(text));
        let genesis = a.hash();
        for _ in 0..500 {
            let ra = a.step().expect("the world runs");
            let rb = b.step().expect("the world runs");
            assert_eq!(ra, rb, "{id}");
        }
        assert_eq!(a.hash(), b.hash(), "{id}");
        assert_ne!(a.hash(), genesis, "{id}");
    }
}

/// One tick at the rest point: every market must trade, and no market line may name a plant;
/// the largest gap, with what it is, over every price (posted and next) in log, both fills of
/// every market from 1, every coin, stock and plant from genesis in log, each plant's target
/// from its genesis plant in log, and each plant's order from u·K*_p (in log under M3's rule; as
/// a share of K*_p under `Gap`, which takes the whole gap K*_p − P each tick and so carries the
/// rounding of the plant itself).
fn rest_gap(s: &Sim, r: &TickReport, setup: &Setup, what: &str) -> (f64, String) {
    let inst = &setup.instance;
    let g = genesis(setup).expect("genesis");
    let mut worst = (0.0, String::new());
    let mut note = |d: f64, name: String| {
        if d > worst.0 || d.is_nan() {
            worst = (d, name);
        }
    };
    for (m, p0) in inst.markets().iter().zip(&g.prices) {
        let line = r
            .markets
            .iter()
            .find(|l| key_of(s, l.good) == *m)
            .expect("a market line");
        assert!(
            line.trades(),
            "{what}: {m} does not trade at tick {}",
            r.tick
        );
        note(gap(line.price, *p0), format!("{m}'s price"));
        note(gap(line.next_price, *p0), format!("{m}'s next price"));
        note((1.0 - line.buyer_fill).abs(), format!("{m}'s buyer fill"));
        note((1.0 - line.seller_fill).abs(), format!("{m}'s seller fill"));
    }
    // A plant is not a market (§1).
    assert!(
        r.markets
            .iter()
            .all(|l| !key_of(s, l.good).starts_with("plant.")),
        "{what}: a plant has a market line"
    );
    for (a, c0) in inst.actors().iter().zip(&g.coin) {
        note(gap(held(s, a, "coin"), *c0), format!("{a}'s coin"));
    }
    let u = setup.plant_params().expect("plant params").u;
    for (name, q0) in stocks(inst).iter().zip(&g.stock) {
        let (holder, good_key) = match name.as_str() {
            "stock.good" => ("desk.good".to_string(), "good".to_string()),
            "heads.capacity" => ("desk.capacity".into(), "horse".into()),
            "hours.capacity" => ("desk.capacity".into(), "traction".into()),
            "finished.maker" => ("desk.maker".into(), "horse".into()),
            "stock.fodder" => ("desk.fodder".into(), "fodder".into()),
            "stock.traction" => ("desk.traction".into(), "traction".into()),
            p => {
                let d = p.strip_prefix("plant.").expect("a plant");
                let desk = format!("desk.{d}");
                let rec = plant_of(s, &desk).expect("a planted state");
                note(gap(rec.target, *q0), format!("{p}'s target"));
                let want = u * rec.target;
                let og = if inst.design.order == "Gap" {
                    (rec.order - want).abs() / rec.target
                } else {
                    gap(rec.order, want)
                };
                note(og, format!("{p}'s order"));
                (desk, p.to_string())
            }
        };
        // What the desk holds after the tick: its output, its horses after the tick's wear and
        // build, the maker's finished heads, the plant P + I′ − u·P.
        note(gap(held(s, &holder, &good_key), *q0), name.clone());
    }
    worst
}

#[test]
fn loops_rest_point_is_the_oracles() {
    // LOOPS-RULES §10 (E2): the oracle's point is a fixed point of the planted roles' map. Three
    // ticks from genesis at the oracle's f64 point leave every price, coin, stock and plant
    // within 1e-12 of genesis in log, every market trading (the horse market too) with both fills
    // 1, and each plant's order u·K*_p with K*_p its genesis plant, at every built instance and
    // funded target (b ×1, ×1.1, ×0.9, ×2, ×0.5); and LB1 at 12, 24 and 365 ticks a year.
    //
    // LN6, CAPACITY's negative control, orders the whole gap each tick: it passes its plant's
    // rounding to its order at full weight, 1/u (about 500) times the order's size, and s (about
    // 40) times that into its bundles' demand, and so into every market. Its bar is `GAP_REST`,
    // and its worst gap is printed (3.0e-12 at b x0.5 on 2026-09-30, fodder's buyer fill).
    const GAP_REST: f64 = 1e-11;
    let mut checked = 0;
    for id in Instance::IDS {
        let base = registered(id);
        let bar = if base.instance.design.order == "Gap" {
            GAP_REST
        } else {
            REST
        };
        for f in [1.0, 1.1, 0.9, 2.0, 0.5] {
            let inst = base.instance.with_b(base.instance.b * f);
            if !inst.point(52).unwrap().funded {
                continue;
            }
            let mut setup = base.clone();
            setup.instance = inst;
            let mut s = sim(&tape_ron(&setup).unwrap());
            let mut worst = (0.0, String::new());
            for _ in 0..3 {
                let r = s.step().expect("the world runs");
                let w = rest_gap(&s, &r, &setup, &format!("{id} b x{f}"));
                if w.0 > worst.0 {
                    worst = (w.0, format!("{} at tick {}", w.1, r.tick));
                }
            }
            if bar != REST {
                println!("{id} b x{f}: worst {:e}, {}", worst.0, worst.1);
            }
            assert!(worst.0 <= bar, "{id} b x{f}: {} {:e} off", worst.1, worst.0);
            checked += 1;
        }
    }
    assert_eq!(
        checked,
        Instance::IDS.len() * 5,
        "every target is funded (FUNDED §5)"
    );
    for tpy in [12, 24, 365] {
        let setup = Setup::registered("lb1", tpy).unwrap();
        let mut s = sim(&tape_ron(&setup).unwrap());
        for _ in 0..3 {
            let r = s.step().expect("the world runs");
            let (d, what) = rest_gap(&s, &r, &setup, &format!("lb1 at {tpy} a year"));
            assert!(
                d <= REST,
                "lb1 at {tpy} a year: {what} {d:e} off at tick {}",
                r.tick
            );
        }
    }
}

/// Every market line's numbers, by the good's key, bit for bit.
fn market_bits(s: &Sim, r: &TickReport) -> Vec<(String, [u64; 8])> {
    r.markets
        .iter()
        .map(|l| {
            (
                key_of(s, l.good),
                [
                    l.price.to_bits(),
                    l.next_price.to_bits(),
                    l.ema.to_bits(),
                    l.supply.to_bits(),
                    l.demand.to_bits(),
                    l.cleared.to_bits(),
                    l.buyer_fill.to_bits(),
                    l.seller_fill.to_bits(),
                ],
            )
        })
        .collect()
}

/// Every actor's holding of every good but the plants, by keys, bit for bit.
fn holding_bits(s: &Sim) -> Vec<(String, String, u64)> {
    let w = s.world();
    let mut out = Vec::new();
    for a in &w.actors {
        let inv = s.holding(Holder::Actor(a.id)).expect("a holding");
        for g in &w.goods {
            if g.key.as_str().starts_with("plant.") {
                continue;
            }
            out.push((
                a.key.to_string(),
                g.key.to_string(),
                inv.get(g.id).to_bits(),
            ));
        }
    }
    out
}

/// Every actor's own state read as its kind's, the plant record aside.
fn kind_states(s: &Sim) -> Vec<(String, ActorState)> {
    let w = s.world();
    w.actors
        .iter()
        .map(|a| {
            let st = *s.actor_state(a.id).expect("a state");
            let kind = match st {
                ActorState::PlantedType(p) => ActorState::MachDesk(p.desk),
                ActorState::PlantedMaker(p) => ActorState::Maker(p.desk),
                ActorState::PlantedCapacity(p) => ActorState::Capacity(p.desk),
                other => other,
            };
            (a.key.to_string(), kind)
        })
        .collect()
}

#[test]
fn loops_theta_one_is_the_plain_tape() {
    // LOOPS-RULES §3.9 (E1): with every plant at θ = 1, LB1 and LW1 are their plant-free tapes
    // (LN1, LW0) value for value: at hold, from w ×2 and from r ×2, every market line, every
    // holding but the plant goods, and every actor's state read as its kind's, bit for bit over
    // 2,000 ticks. The plants are 0 at genesis and never ordered.
    for (planted, plain) in [("lb1", "ln1"), ("lw1", "lw0")] {
        for (market, f) in [("labour", 1.0), ("labour", 2.0), ("land", 2.0)] {
            let mut a = registered(planted).with_theta(1.0);
            let mut b = registered(plain);
            for s in [&mut a, &mut b] {
                let inst = s.instance.clone();
                s.displace.price_of(&inst, market, f).unwrap();
            }
            let (mut sa, mut sb) = (sim(&tape_ron(&a).unwrap()), sim(&tape_ron(&b).unwrap()));
            let what = format!("{planted} at θ 1 against {plain}, {market} x{f}");
            assert_eq!(holding_bits(&sa), holding_bits(&sb), "{what}: genesis");
            for t in 0..2000 {
                let ra = sa.step().expect("the planted tape runs");
                let rb = sb.step().expect("the plain tape runs");
                assert_eq!(
                    market_bits(&sa, &ra),
                    market_bits(&sb, &rb),
                    "{what}: tick {t}"
                );
                assert_eq!(holding_bits(&sa), holding_bits(&sb), "{what}: tick {t}");
                assert_eq!(kind_states(&sa), kind_states(&sb), "{what}: tick {t}");
                for d in a.instance.planted() {
                    let p = plant_of(&sa, &format!("desk.{d}")).expect("a planted state");
                    assert_eq!((p.order, p.built, p.target), (0.0, 0.0, 0.0), "{what}");
                    assert_eq!(held(&sa, &format!("desk.{d}"), &format!("plant.{d}")), 0.0);
                }
            }
        }
    }
}

#[test]
fn loops_fixed_plant_is_drs() {
    // LOOPS-RULES §3.9, §8.4 (E1): LB1 with `--fixed-plants` (δ_p 0, genesis at the design's
    // plants Q = κ_p·X) over 2,000 ticks: every plant holding stays at Q bit for bit, no plant
    // good is minted or burned, and no plant order is placed.
    let setup = registered("lb1").with_fixed_plants().unwrap();
    let inst = setup.instance.clone();
    let g = genesis(&setup).unwrap();
    let mut s = sim(&tape_ron(&setup).unwrap());
    let plants: Vec<(String, f64)> = inst
        .planted()
        .iter()
        .map(|d| {
            let q = g.stock_of(&inst, &format!("plant.{d}")).unwrap();
            assert!(q > 0.0, "{d}: the design's plant");
            (d.clone(), q)
        })
        .collect();
    let plant_goods: Vec<GoodId> = plants
        .iter()
        .map(|(d, _)| good(&s, &format!("plant.{d}")))
        .collect();
    for t in 0..2000 {
        let (_, trace) = s.step_traced().expect("the world runs");
        for (d, q) in &plants {
            let desk = format!("desk.{d}");
            let held_now = held(&s, &desk, &format!("plant.{d}"));
            assert_eq!(held_now.to_bits(), q.to_bits(), "{d} at tick {t}");
            let rec = plant_of(&s, &desk).expect("a planted state");
            assert_eq!((rec.order, rec.built), (0.0, 0.0), "{d} at tick {t}");
            assert_eq!(rec.held.to_bits(), q.to_bits(), "{d} at tick {t}");
        }
        for e in &trace.0 {
            let moved = match &e.delta {
                StateDelta::Mint { good, .. } | StateDelta::Burn { good, .. } => {
                    plant_goods.contains(good)
                }
                _ => false,
            };
            assert!(!moved, "a plant good moved at tick {t}: {:?}", e.delta);
        }
    }
}

#[test]
fn loops_conserve_every_tick() {
    // R2 on every committed loops tape, at rest and through a large transient (w ×2, every desk's
    // coin ×0.1 and every plant ×2 at genesis): every tick's ledger and the run's close within
    // their registered tolerances, and the money stock stays put. Every `Depreciation` burn is in
    // upkeep and exactly δ·H of heads (the capacity desk's holding when it decided) or u·P of a
    // plant (the plant its desk recorded at decide), and moved what it asked for; every plant
    // mint is in produce.
    for (id, text) in TAPES {
        let mut setup = registered(id);
        let inst = setup.instance.clone();
        setup.displace.price_of(&inst, "labour", 2.0).unwrap();
        for d in inst.desks() {
            setup
                .displace
                .coin_of(&inst, &format!("desk.{d}"), 0.1)
                .unwrap();
        }
        for d in inst.planted() {
            setup
                .displace
                .stock_of(&inst, &format!("plant.{d}"), 2.0)
                .unwrap();
        }
        let (_, delta) = inst.per_tick(52).unwrap();
        let u = setup.plant_params().unwrap().u;
        for text in [text.to_string(), tape_ron(&setup).unwrap()] {
            let mut s = sim(&text);
            let coin = good(&s, "coin");
            let plant_goods: Vec<GoodId> = inst
                .planted()
                .iter()
                .map(|d| good(&s, &format!("plant.{d}")))
                .collect();
            let money = |s: &Sim| {
                s.world()
                    .actors
                    .iter()
                    .map(|a| s.holding(Holder::Actor(a.id)).map_or(0.0, |i| i.get(coin)))
                    .fold(0.0, |x, y| x + y)
            };
            let m0 = money(&s);
            let (mut worn_heads, mut worn_plants, mut built) = (0, 0, 0);
            for _ in 0..2000 {
                let (r, trace) = s.step_traced().expect("no breach stops the run");
                assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0, "{id}");
                assert!((money(&s) - m0).abs() <= COIN * m0, "{id}");
                for e in &trace.0 {
                    match &e.delta {
                        StateDelta::Mint { good, .. } if plant_goods.contains(good) => {
                            assert_eq!(e.phase, Phase::Production, "{id}: a plant minted");
                            built += 1;
                        }
                        StateDelta::Burn {
                            from: Holder::Actor(a),
                            good,
                            amount,
                            prov: Provenance::Depreciation,
                        } => {
                            assert_eq!(e.phase, Phase::Upkeep, "{id}");
                            let Amount::Qty(q) = amount else {
                                panic!("{id}: a wear burn of everything");
                            };
                            assert_eq!(e.moved.to_bits(), q.to_bits(), "{id}");
                            let want = if plant_goods.contains(good) {
                                worn_plants += 1;
                                let rec = match s.actor_state(*a) {
                                    Some(ActorState::PlantedType(p)) => p.plant,
                                    Some(ActorState::PlantedMaker(p)) => p.plant,
                                    Some(ActorState::PlantedCapacity(p)) => p.plant,
                                    other => panic!("{id}: {other:?} wears a plant"),
                                };
                                u * rec.held
                            } else {
                                worn_heads += 1;
                                match s.actor_state(*a) {
                                    Some(ActorState::PlantedCapacity(c)) => delta * c.desk.held,
                                    Some(ActorState::Capacity(c)) => delta * c.held,
                                    other => panic!("{id}: {other:?} wears heads"),
                                }
                            };
                            assert_eq!(q.to_bits(), want.to_bits(), "{id}");
                        }
                        StateDelta::Burn { good, prov, .. } if plant_goods.contains(good) => {
                            panic!("{id}: a plant burned as {prov:?}");
                        }
                        _ => {}
                    }
                }
            }
            assert!(
                worn_plants > 0 && built > 0,
                "{id}: plants wear and are built"
            );
            assert_eq!(
                worn_heads > 0,
                inst.config == Config::Wet,
                "{id}: heads wear"
            );
        }
    }
}

#[test]
fn loops_tapes_name_their_plants_untraded() {
    // LOOPS-RULES §1, §3.1: each plant is an untraded good, held by its desk alone, with no
    // market, no genesis price and no book slot that is ever written.
    for (id, text) in TAPES {
        let mut s = sim(text);
        let inst = registered(id).instance;
        let w = s.world().clone();
        for d in inst.planted() {
            let g = w
                .id_of::<GoodId>(&format!("plant.{d}"))
                .expect("a plant good");
            assert!(w.is_untraded(g) && !w.has_market(g), "{id}: plant.{d}");
            assert!(
                w.markets().all(|(_, m)| m != g),
                "{id}: plant.{d} has a market"
            );
            let holders: Vec<Holder> = s.holdings_of(g).map(|(h, _)| h).collect();
            assert_eq!(
                holders,
                vec![Holder::Actor(actor(&s, &format!("desk.{d}")))],
                "{id}: plant.{d}"
            );
        }
        let home = w.id_of::<NodeId>("home").unwrap();
        for _ in 0..50 {
            s.step().unwrap();
        }
        for d in inst.planted() {
            let g = w.id_of::<GoodId>(&format!("plant.{d}")).unwrap();
            assert_eq!(s.price(home, g), Some(1.0), "{id}: plant.{d}'s slot");
            assert_eq!(s.supply(home, g), Some(0.0), "{id}: plant.{d}'s slot");
        }
    }
}

/// A JSON value, as `instances.json` holds them: enough of JSON for this file, with no new
/// dependency (LOOPS-RULES §10).
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn parse(text: &str) -> Json {
        let b: Vec<char> = text.chars().collect();
        let mut i = 0;
        let v = Json::value(&b, &mut i);
        Json::ws(&b, &mut i);
        assert_eq!(i, b.len(), "trailing text");
        v
    }

    fn ws(b: &[char], i: &mut usize) {
        while *i < b.len() && b[*i].is_whitespace() {
            *i += 1;
        }
    }

    fn value(b: &[char], i: &mut usize) -> Json {
        Json::ws(b, i);
        match b[*i] {
            '{' => {
                *i += 1;
                let mut out = Vec::new();
                loop {
                    Json::ws(b, i);
                    if b[*i] == '}' {
                        *i += 1;
                        return Json::Obj(out);
                    }
                    let Json::Str(k) = Json::value(b, i) else {
                        panic!("a key at {i}")
                    };
                    Json::ws(b, i);
                    assert_eq!(b[*i], ':');
                    *i += 1;
                    out.push((k, Json::value(b, i)));
                    Json::ws(b, i);
                    if b[*i] == ',' {
                        *i += 1;
                    }
                }
            }
            '[' => {
                *i += 1;
                let mut out = Vec::new();
                loop {
                    Json::ws(b, i);
                    if b[*i] == ']' {
                        *i += 1;
                        return Json::Arr(out);
                    }
                    out.push(Json::value(b, i));
                    Json::ws(b, i);
                    if b[*i] == ',' {
                        *i += 1;
                    }
                }
            }
            '"' => {
                *i += 1;
                let mut s = String::new();
                while b[*i] != '"' {
                    if b[*i] == '\\' {
                        *i += 1;
                        match b[*i] {
                            'u' => {
                                let hex: String = b[*i + 1..*i + 5].iter().collect();
                                let c = u32::from_str_radix(&hex, 16).expect("a \\u escape");
                                s.push(char::from_u32(c).unwrap_or('?'));
                                *i += 4;
                            }
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            c => s.push(c),
                        }
                    } else {
                        s.push(b[*i]);
                    }
                    *i += 1;
                }
                *i += 1;
                Json::Str(s)
            }
            't' => {
                *i += 4;
                Json::Bool(true)
            }
            'f' => {
                *i += 5;
                Json::Bool(false)
            }
            'n' => {
                *i += 4;
                Json::Null
            }
            _ => {
                let start = *i;
                while *i < b.len() && "+-0123456789.eE".contains(b[*i]) {
                    *i += 1;
                }
                let t: String = b[start..*i].iter().collect();
                Json::Num(t.parse().expect("a number"))
            }
        }
    }

    fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(v) => v
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, x)| x)
                .unwrap_or_else(|| panic!("no {key}")),
            other => panic!("{other:?} is not an object"),
        }
    }

    fn num(&self) -> f64 {
        match self {
            Json::Num(x) => *x,
            other => panic!("{other:?} is not a number"),
        }
    }

    fn str(&self) -> &str {
        match self {
            Json::Str(s) => s,
            other => panic!("{other:?} is not a string"),
        }
    }

    fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(v) => v,
            other => panic!("{other:?} is not an array"),
        }
    }
}

fn rel(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        ((a - b) / b).abs()
    }
}

#[test]
fn loops_instances_are_fundeds() {
    // LOOPS-RULES §10: the generator's params are `instances.json`'s `tape_params` bit for bit;
    // its oracle points (rule B, the loop cut and the flow control, at three δ and five land
    // factors) are the file's `points` within 1e-15 relative; s1 is 40.47205917167808; and each
    // genesis plant κ_p·X is the file's `plants` K* within 1e-12, for the fodder desk, the maker,
    // the capacity desk's running bundle and the flow control's horse-day desk, at θ 0.8 and 0.7
    // and δ_p 10% and 4% a year.
    let json = Json::parse(include_str!("../../../docs/probe/loops/instances.json"));
    let tp = json.get("tape_params");
    let mine = |id: &str| params(&Instance::named(id).unwrap(), 52).unwrap();
    for (list, id) in [("county", "lb1"), ("B", "lb1")] {
        for row in tp.get(list).arr() {
            let r = row.arr();
            let (key, value, unit) = (r[0].str(), r[1].num(), r[2].str());
            let p = mine(id)
                .into_iter()
                .find(|p| p.0 == key)
                .unwrap_or_else(|| panic!("{key}"));
            assert_eq!((p.1.to_bits(), p.2), (value.to_bits(), unit), "{key}");
        }
    }
    let cut = mine("lc1");
    assert_eq!(
        cut.iter()
            .find(|p| p.0 == "inst.fodder.traction")
            .unwrap()
            .1,
        0.0
    );
    // The flow control's horse-day recipe (FUNDED §4.2).
    let flow = &json.get("instances").arr()[1];
    assert_eq!(flow.get("id").str(), "B-flow");
    let (f, l, b) = Instance::named("lw1").unwrap().flow_recipe(52).unwrap();
    let hd = flow.get("horse_day");
    for (x, k) in [(f, "fodder"), (l, "labour"), (b, "land")] {
        assert!(rel(x, hd.get(k).num()) <= 1e-15, "{k}: {x:e}");
    }
    // The points.
    let points = json.get("points");
    let mut n = 0;
    for delta in [0.08, 0.1, 0.04] {
        for fac in [1.0, 1.1, 0.9, 2.0, 0.5] {
            let key = format!("delta {delta}, b x{fac}");
            let at = points.get(&key);
            for (name, id) in [("B", "lb1"), ("B-cut", "lc1"), ("B-flow", "lw1")] {
                let mut inst = Instance::named(id).unwrap().with_b(fac);
                inst.delta = delta;
                let e = inst.point(52).unwrap();
                let o = at.get(name);
                let prices = o.get("prices");
                let volumes = o.get("volumes");
                let mut pairs = vec![
                    (e.x_star, o.get("x_star").num(), "x*"),
                    (e.v, o.get("v").num(), "v"),
                    (e.p_s, o.get("P_s").num(), "P_s"),
                    (e.y, o.get("Y").num(), "Y"),
                    (e.n_a, o.get("N_a").num(), "N_a"),
                    (e.pf, prices.get("fodder").num(), "p_f"),
                    (e.ph, prices.get("horse_day").num(), "p_h"),
                    (e.qf, volumes.get("fodder").num(), "q_f"),
                    (e.hours, volumes.get("horse_days").num(), "X"),
                ];
                if name != "B-flow" {
                    pairs.push((e.pk, prices.get("horse").num(), "p_K"));
                    pairs.push((e.made, volumes.get("horses_made").num(), "q_b"));
                    pairs.push((e.capacity, volumes.get("horses_installed").num(), "H"));
                }
                let by = volumes.get("horse_days_by");
                pairs.push((e.task_hours, by.get("good_desk_tasks").num(), "the tasks'"));
                if name != "B-cut" {
                    pairs.push((e.fodder_hours, by.get("fodder_desk").num(), "fodder's"));
                }
                for (x, want, what) in pairs {
                    assert!(rel(x, want) <= 1e-15, "{key} {name} {what}: {x:e} {want:e}");
                }
                // The file writes the good's price as P_s − h, which cancels: it holds to the
                // rounding of P_s (FUNDED's `aggregates`), not to 1e-15 of p.
                let p = prices.get("good").num();
                assert!(
                    (e.p - p).abs() <= 4.0 * f64::EPSILON * e.p_s,
                    "{key} {name} p: {:e} {p:e}",
                    e.p
                );
                n += 1;
            }
        }
    }
    assert_eq!(n, 45);
    // s1 and the plants.
    let setup = Setup::registered("lb1", 52).unwrap();
    assert_eq!(setup.plant_params().unwrap().size, 40.47205917167808);
    let mut m = 0;
    for row in json.get("plants").arr() {
        let target = row.get("target").str();
        let fac: f64 = target
            .strip_prefix("land x")
            .and_then(|t| t.split(' ').next())
            .unwrap()
            .parse()
            .unwrap();
        let (theta, dp) = (
            row.get("theta").num(),
            row.get("plant_delta_per_year").num(),
        );
        let (id, stock) = match row.get("desk").str() {
            "fodder" => ("lb1", "plant.fodder"),
            "maker" => ("lb1", "plant.maker"),
            "horse-days, running bundle" => ("lb1", "plant.capacity"),
            "horse-days" => ("lw1", "plant.traction"),
            other => panic!("{other}"),
        };
        let mut s = Setup::registered(id, 52)
            .unwrap()
            .with_theta(theta)
            .with_plant_delta(dp)
            .unwrap();
        s.instance = s.instance.with_b(fac);
        let g = genesis(&s).unwrap();
        let k = g.stock_of(&s.instance, stock).unwrap();
        let want = row.get("K_star").num();
        assert!(
            rel(k, want) <= 1e-12,
            "{target} {stock} θ {theta} δp {dp}: {k:e} {want:e}"
        );
        m += 1;
    }
    assert_eq!(m, 80);
}
