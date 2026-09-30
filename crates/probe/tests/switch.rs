//! The type switch at the wall on the engine (P2.4; docs/probe/SWITCH-RULES.md; the switch scan,
//! docs/probe/switch/SPEC.md §3.10): the tape is its generator's output, genesis and every point
//! are unit 1d's with E's efficiencies (the registered doubles and the 50-digit switch table),
//! the point is a rest point of the roles at every target and tick length, a switch pop is paid
//! on both markets and money is conserved, a run that never crosses is IW1's bit for bit and so
//! is IS1 with ε 0, the harness reads the pool shares and gaps as the pops hold them, and the
//! battery, families, grammar and dials are the registered ones.
//!
//! Bars named here: `COIN`, 1e-12 of the money stock, for the drift of the total coin over a run
//! (as the wall's); `REL`, 1e-15 relative, for a sum the harness and the engine form in another
//! order. Mode A's bars are PROBE-SPEC §4.6's, in `probe::protocol`.

use certify::{tape_hash, Obs};
use probe::markets::harness::{csv_header, observables, run, Row};
use probe::markets::instance::Instance;
use probe::markets::perturb::{battery, family, Perturbation};
use probe::markets::setup::{genesis, tape_ron, Dials, Setup};
use probe::protocol::{RUN_TICKS, TOL_FLOOR};
use rustyecon_engine::prelude::{ActorId, GoodId, Holder, SideTag, Sim, Tape};
use rustyecon_engine::rustyecon_agents::ActorState;

const IS1: &str = include_str!("../../../tapes/markets-is1.ron");
const IW1: &str = include_str!("../../../tapes/markets-iw1.ron");
const REG: &str = "../../../docs/probe/switch/registered/";
const POINTS_IS1: &str = include_str!("../../../docs/probe/switch/registered/points_is1.jsonl");
const POINTS_IS2: &str = include_str!("../../../docs/probe/switch/registered/points_is2.jsonl");
const MP_IS1: &str = include_str!("../../../docs/probe/switch/registered/switch_mp.json");
const MP_IS2: &str = include_str!("../../../docs/probe/switch/registered/switch_mp_is2.json");
const RUNS_BATTERY: &str =
    include_str!("../../../docs/probe/switch/registered/runs_is1_battery.tsv");
const RUNS_BATTERY_IS2: &str =
    include_str!("../../../docs/probe/switch/registered/runs_is2_battery.tsv");
const RUNS_TIER3S: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_tier3s.tsv");
const RUNS_STOCKS: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_stocks.tsv");
const RUNS_BASIN: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_basin.tsv");
const RUNS_JOINT2: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_joint2.tsv");
const RUNS_JOINT4: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_joint4.tsv");
const RUNS_TPY12: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_tpy12.tsv");
const RUNS_NBHD: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_nbhd.tsv");
const RUNS_SWITCH: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_switch.tsv");
const RUNS_L10: &str = include_str!("../../../docs/probe/switch/registered/runs_is1_tier3L10.tsv");
const COIN: f64 = 1e-12;
const REL: f64 = 1e-15;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn setup(id: &str, tpy: u32) -> Setup {
    Setup::registered(id, tpy).expect("a switch instance")
}

/// A JSON value, enough of it for the scan's registered files (Python's `json.dumps`).
#[derive(Debug, Clone, PartialEq)]
enum J {
    N(f64),
    S(String),
    B(bool),
    Null,
    A(Vec<J>),
    O(Vec<(String, J)>),
}

impl J {
    fn parse(text: &str) -> J {
        let b = text.as_bytes();
        let mut i = 0;
        J::value(b, &mut i)
    }

    fn ws(b: &[u8], i: &mut usize) {
        while *i < b.len() && (b[*i] as char).is_whitespace() {
            *i += 1;
        }
    }

    fn value(b: &[u8], i: &mut usize) -> J {
        J::ws(b, i);
        match b[*i] {
            b'{' => {
                *i += 1;
                let mut o = Vec::new();
                loop {
                    J::ws(b, i);
                    if b[*i] == b'}' {
                        *i += 1;
                        break;
                    }
                    let J::S(k) = J::value(b, i) else {
                        panic!("a key")
                    };
                    J::ws(b, i);
                    assert_eq!(b[*i], b':');
                    *i += 1;
                    o.push((k, J::value(b, i)));
                    J::ws(b, i);
                    if b[*i] == b',' {
                        *i += 1;
                    }
                }
                J::O(o)
            }
            b'[' => {
                *i += 1;
                let mut a = Vec::new();
                loop {
                    J::ws(b, i);
                    if b[*i] == b']' {
                        *i += 1;
                        break;
                    }
                    a.push(J::value(b, i));
                    J::ws(b, i);
                    if b[*i] == b',' {
                        *i += 1;
                    }
                }
                J::A(a)
            }
            b'"' => {
                *i += 1;
                let from = *i;
                while b[*i] != b'"' {
                    *i += if b[*i] == b'\\' { 2 } else { 1 };
                }
                let s = String::from_utf8_lossy(&b[from..*i]).to_string();
                *i += 1;
                J::S(s)
            }
            _ => {
                let from = *i;
                while *i < b.len()
                    && !matches!(b[*i], b',' | b']' | b'}')
                    && !(b[*i] as char).is_whitespace()
                {
                    *i += 1;
                }
                let t = std::str::from_utf8(&b[from..*i]).unwrap();
                match t {
                    "true" => J::B(true),
                    "false" => J::B(false),
                    "null" => J::Null,
                    "NaN" => J::N(f64::NAN),
                    "Infinity" => J::N(f64::INFINITY),
                    "-Infinity" => J::N(f64::NEG_INFINITY),
                    _ => J::N(t.parse().unwrap_or_else(|_| panic!("{t}"))),
                }
            }
        }
    }

    fn get(&self, k: &str) -> &J {
        match self {
            J::O(o) => {
                &o.iter()
                    .find(|(x, _)| x == k)
                    .unwrap_or_else(|| panic!("{k}"))
                    .1
            }
            _ => panic!("not an object: {k}"),
        }
    }

    fn n(&self) -> f64 {
        match self {
            J::N(x) => *x,
            other => panic!("not a number: {other:?}"),
        }
    }

    fn b(&self) -> bool {
        match self {
            J::B(x) => *x,
            other => panic!("not a bool: {other:?}"),
        }
    }

    fn a(&self) -> &[J] {
        match self {
            J::A(a) => a,
            other => panic!("not an array: {other:?}"),
        }
    }

    fn s(&self) -> &str {
        match self {
            J::S(s) => s,
            other => panic!("not a string: {other:?}"),
        }
    }
}

/// An instance at a registered tag: `base` or `<coefficient>=<value>` as the battery names it.
fn at_tag(id: &str, tag: &str) -> Instance {
    let mut i = Instance::named(id).unwrap();
    if tag != "base" {
        let (c, v) = tag.split_once('=').unwrap();
        let param = i.coef(c).unwrap().param.clone();
        i.set(&param, v.parse().unwrap()).unwrap();
    }
    i
}

/// Every registered target's tag, the base first, as the scan wrote them.
fn tags(id: &str) -> Vec<String> {
    let i = Instance::named(id).unwrap();
    let mut t = vec!["base".to_string()];
    for c in &i.coefs {
        for v in &c.values {
            t.push(format!("{}={v}", c.name));
        }
    }
    t
}

#[test]
fn markets_is1_tape_is_its_generators_output() {
    // tapes/markets-is1.ron is `markets-tape --inst is1`'s output for the registered setup at 52
    // ticks a year, byte for byte, so its genesis is unit 1d's point and nobody edits it. Against
    // IW1's tape it adds the efficiencies, the switch's dials and each reserved pop's pool, and
    // changes the header, the name, the roles' basis and the genesis basis; the genesis prices
    // and holdings are IW1's, line for line (both types are at their walls at the base).
    let generated = tape_ron(&setup("is1", 52)).expect("the generator runs");
    assert!(
        generated == IS1,
        "tapes/markets-is1.ron differs from its generator: run `cargo run -p rustyecon-probe \
         --bin markets-tape -- --inst is1 tapes/markets-is1.ron`"
    );
    let genesis_lines = |t: &str| -> Vec<String> {
        t.lines()
            .skip_while(|l| !l.starts_with("        prices: ["))
            .take_while(|l| !l.starts_with("    events:"))
            .map(str::to_string)
            .collect()
    };
    assert!(!genesis_lines(IS1).is_empty());
    assert_eq!(genesis_lines(IS1), genesis_lines(IW1));
    for k in [
        "(key: \"inst.trained.efficiency\", value: 1.5, unit: Dimensionless,",
        "(key: \"inst.master.efficiency\", value: 1.8, unit: Dimensionless,",
        "(key: \"rate.switch.trained\", value: 26.0, unit: RatePerYear,",
        "(key: \"rate.switch.master\", value: 26.0, unit: RatePerYear,",
        "pool: Some((good: \"labour\", efficiency: \"inst.trained.efficiency\", rate: \"rate.switch.trained\", share: 0.0)),",
        "pool: Some((good: \"labour\", efficiency: \"inst.master.efficiency\", rate: \"rate.switch.master\", share: 0.0)),",
    ] {
        assert_eq!(IS1.matches(k).count(), 1, "{k}");
        assert!(!IW1.contains(k), "{k}");
    }
    // IW1's tape keeps its text, tape_hash and world_id (build-switch's base streams, b7b9242).
    let t = Tape::from_ron(IW1).unwrap();
    assert_eq!(tape_hash(&t), 0xec2c_4d1c_8f62_ea28);
    assert_eq!(sim(IW1).world().world_id, 0x7698_c09c_b7c5_4791);
    assert_eq!(tape_ron(&setup("iw1", 52)).unwrap(), IW1);
}

#[test]
fn markets_is1_points_are_the_registered_ones() {
    // The scan's §2.2–§2.4 and E9: at IS1's and IS2's base and 12 targets, at 12, 52 and 365
    // ticks a year (78 points), the harness's unit 1d point is the scan's oracle's registered
    // doubles (`points_is1.jsonl`, `points_is2.jsonl`: v, P_s, Y, n_D, 1 − x*, every price and
    // output, and each worker type's wage, hours, reserved and pool hours and pooled flag) bit
    // for bit; its pool shares a* and switch distances are the 50-digit solve's
    // (`switch_mp*.json`) within 1e-12, its pooled flags the solve's; and every walled target of
    // IS1 is IW1's point double for double.
    let _ = REG;
    for (id, points, mp) in [("is1", POINTS_IS1, MP_IS1), ("is2", POINTS_IS2, MP_IS2)] {
        let mp = J::parse(mp);
        let mut n = 0;
        for line in points.lines() {
            let p = J::parse(line);
            let tag = p.get("tag").s().to_string();
            let tpy = p.get("tpy").n() as u32;
            assert_eq!(p.get("regime").s(), "Interior");
            let inst = at_tag(id, &tag);
            let e = inst.point(tpy).unwrap();
            let pairs = [
                (e.v, p.get("v").n(), "v"),
                (e.p_s, p.get("p_s").n(), "p_s"),
                (e.y, p.get("y").n(), "y"),
                (e.pool, p.get("n_pool").n(), "n_pool"),
                (e.one_minus_x, p.get("one_minus_x").n(), "1 - x*"),
            ];
            for (x, y, what) in pairs {
                assert_eq!(x.to_bits(), y.to_bits(), "{id} {tag} @{tpy}: {what}");
            }
            for (j, c) in p.get("categories").a().iter().take(2).enumerate() {
                assert_eq!(e.cat_price[j].to_bits(), c.get("price").n().to_bits());
                assert_eq!(e.cat_output[j].to_bits(), c.get("output").n().to_bits());
            }
            let ty = &p.get("types").a()[0];
            assert_eq!(e.type_price[0].to_bits(), ty.get("price").n().to_bits());
            assert_eq!(
                e.type_services[0].to_bits(),
                ty.get("services").n().to_bits()
            );
            let workers = p.get("workers").a();
            for (q, wk) in workers.iter().enumerate() {
                assert_eq!(e.hours[q].to_bits(), wk.get("hours").n().to_bits(), "{tag}");
                if q == 0 {
                    continue;
                }
                let i = q - 1;
                assert_eq!(e.wage[i].to_bits(), wk.get("wage").n().to_bits(), "{tag}");
                assert_eq!(
                    e.reserved[i].to_bits(),
                    wk.get("reserved_hours").n().to_bits()
                );
                assert_eq!(e.pooled[i], wk.get("pooled").b(), "{id} {tag}");
                let a = if wk.get("pooled").b() && wk.get("pool_hours").n() > 0.0 {
                    wk.get("pool_hours").n() / wk.get("hours").n()
                } else {
                    0.0
                };
                assert_eq!(e.pool_share[i].to_bits(), a.to_bits(), "{id} {tag}");
            }
            let key = format!("{tag}@{tpy}");
            let m = mp.get(&key);
            for i in 0..2 {
                assert_eq!(e.pooled[i], m.get("pooled").a()[i].b(), "{key}");
                let d = m.get("switch").a()[i].n();
                assert!(
                    (e.switch_distance[i] - d).abs() <= 1e-12,
                    "{key}: switch {} against {d}",
                    e.switch_distance[i]
                );
                let a = m.get("a").a()[i].n();
                assert!(
                    (e.pool_share[i] - a).abs() <= 1e-12,
                    "{key}: a* {}",
                    e.pool_share[i]
                );
            }
            // A walled point of IS1 is IW1's.
            if id == "is1" && !e.pooled[0] && !e.pooled[1] {
                let w = at_tag("iw1", &tag).point(tpy).unwrap();
                assert_eq!(
                    (e.v, e.p_s, e.y, e.pool, &e.wage, &e.hours),
                    (w.v, w.p_s, w.y, w.pool, &w.wage, &w.hours),
                    "{tag}"
                );
                assert_eq!((&e.cat_price, &e.type_price), (&w.cat_price, &w.type_price));
                assert_eq!(e.reserved, w.hours[1..]);
            }
            n += 1;
        }
        assert_eq!(n, 39, "{id}");
    }
    // Where each type stands, the scan's §2.2 and §2.4: IS1's trained pools at four targets and
    // its master at one; IS2's trained pools at its base.
    let pooled = |id: &str, tag: &str| at_tag(id, tag).point(52).unwrap().pooled;
    assert_eq!(pooled("is1", "base"), [false, false]);
    assert_eq!(pooled("is1", "tail.services=0.2"), [true, true]);
    assert_eq!(pooled("is1", "tail.services=0.11"), [true, false]);
    assert_eq!(pooled("is2", "base"), [true, false]);
    assert_eq!(pooled("is2", "res.services.trained=0.04"), [false, false]);
    assert!(!Instance::named("iw1").unwrap().switch);
}

#[test]
fn markets_is1_rest_point_is_the_oracles() {
    // The scan's §4 and E2: unit 1d's point with E's efficiencies is a rest point of the roles'
    // map. Three ticks from genesis at the oracle's f64 point, each switch pop's pool share at its
    // a*, leave every observable (the reserved shares 1 − a among them) within 1e-12 of the
    // oracle in log, every market trading with both fills 1 to rounding, at IS1's and IS2's base
    // and 12 cost targets, at 12, 52 and 365 ticks a year: 78 points. A pooled pop's gap is within
    // an ulp of 0 (the oracle posts its wage as ε·v; at a few points of 12 a year the posted wage
    // is an ulp off, the scan's §6.2) and its share within 1e-12 of a* relative; a walled pop's
    // share stays 0.0 with its gap below 0.
    let mut pooled_rests = 0;
    for id in Instance::SWITCH_IDS {
        for tpy in [12, 52, 365] {
            let base = setup(id, tpy);
            for tag in tags(id) {
                let mut s = base.clone();
                s.instance = at_tag(id, &tag);
                let mut rows: Vec<Row> = Vec::new();
                let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
                assert_eq!(rec.hold_failure, None, "{id} {tag} @{tpy}");
                assert_eq!(rec.genesis.point.margin, "Wall");
                for r in &rows {
                    let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                    assert!(
                        worst <= 1e-12,
                        "{id} {tag} @{tpy}: {worst:e} at tick {}",
                        r.tick
                    );
                    for (i, &a) in r.pool.iter().enumerate() {
                        let at = rec.genesis.point.pool_share[i];
                        if rec.genesis.point.pooled[i] {
                            assert!((a - at).abs() <= 1e-12 * at, "{id} {tag} @{tpy}: {a} {at}");
                            assert!(r.switch_gap[i].abs() <= 1e-15, "{id} {tag} @{tpy}");
                        } else {
                            assert_eq!(a, 0.0, "{id} {tag} @{tpy}: a walled share moved");
                            assert!(r.switch_gap[i] < 0.0);
                        }
                    }
                }
                pooled_rests += rec.genesis.point.pooled.iter().filter(|&&p| p).count();
            }
        }
    }
    // IS1's five pooled types at 4 targets and IS2's trained at 12 of its 13 points, at three
    // tick lengths.
    assert_eq!(pooled_rests, 3 * (5 + 12));
}

#[test]
fn markets_is1_holds_at_the_oracle_point() {
    // Mode A (PROBE-SPEC §4.6; the scan's E2) at 52 ticks a year for 20,000 ticks, at IS1's base
    // (both types walled) and IS2's (the trained pooled at a* 0.391): every observable stays
    // within 1e-12 of the oracle in log, every market trades with every fill at least 1 − 1e-9, no
    // produced good spoils beyond 1e-9 of its volume, and the thresholds stay at the wall.
    for id in Instance::SWITCH_IDS {
        let rec = run(&setup(id, 52), "hold", RUN_TICKS, &mut |_| {}).unwrap();
        assert_eq!(rec.stop, probe::harness::Stop::Ran);
        assert_eq!(rec.hold_failure, None, "{id}");
        let worst = rec.stats.peak.0 * TOL_FLOOR;
        assert!(worst <= 1e-12, "{id}: the largest gap was {worst:e}");
        assert_eq!(rec.stats.wall.expect("the wall's readouts").breach, 0);
    }
}

/// Every number of one tick, by key: each market's posted price, next price, S, D, cleared
/// volume and fills, and each actor's coin.
fn numbers(sim: &Sim, obs: &Obs) -> Vec<u64> {
    let w = sim.world();
    let coin = w.id_of::<GoodId>("coin").unwrap();
    let mut v = Vec::new();
    for l in &obs.markets {
        v.extend([
            l.price,
            l.next_price,
            l.supply,
            l.demand,
            l.cleared,
            l.buyer_fill,
            l.seller_fill,
        ]);
    }
    for a in &w.actors {
        v.push(
            sim.holding(Holder::Actor(a.id))
                .map_or(0.0, |i| i.get(coin)),
        );
    }
    v.iter().map(|x| x.to_bits()).collect()
}

fn text_of(id: &str, name: &str) -> String {
    let mut s = setup(id, 52);
    Perturbation::parse(name)
        .unwrap()
        .apply(&mut s, 2000)
        .unwrap();
    tape_ron(&s).unwrap()
}

#[test]
fn markets_is1_never_crossing_is_iw1() {
    // The scan's §3.10 and §5.1 (E1): where no gap ever turns positive the switch never acts, and
    // IS1 is IW1 bit for bit: every market's prices, S, D, cleared volume and fills and every
    // actor's coin, for 2,000 ticks, from hold and from starts that keep both types inside their
    // walls, the pool shares 0.0 throughout. And with ε 0 the switch is structurally off (the
    // scan's §3.5): IS1 with both efficiencies 0 from tick 0 is IW1 bit for bit from starts that
    // would cross with it on (RW(2), p[labour]*2, x*/2, JB(0.5)).
    let quiet = [
        "hold",
        "p[labour]*0.95",
        "p[labour.trained]*1.2",
        "coin.workers.trained*0.5",
        "p[goods]*1.05",
    ];
    let off = "+inst.trained.efficiency=0@genesis+inst.master.efficiency=0@genesis";
    let crossing = ["RW(2)", "p[labour]*2", "x*/2", "JB(0.5)"];
    let mut cases: Vec<(String, String)> = quiet
        .iter()
        .map(|r| (text_of("is1", r), text_of("iw1", r)))
        .collect();
    for r in crossing {
        cases.push((text_of("is1", &format!("{r}{off}")), text_of("iw1", r)));
    }
    for (k, (a_text, b_text)) in cases.iter().enumerate() {
        let (mut a, mut b) = (sim(a_text), sim(b_text));
        let pops: Vec<ActorId> = ["workers.trained", "workers.master"]
            .iter()
            .map(|p| a.world().id_of::<ActorId>(p).unwrap())
            .collect();
        for t in 0..2000 {
            let ra = a.step().unwrap();
            let rb = b.step().unwrap();
            let (oa, ob) = (Obs::of(&ra, &a), Obs::of(&rb, &b));
            assert!(
                numbers(&a, &oa) == numbers(&b, &ob),
                "case {k}: tick {t} differs"
            );
            for &p in &pops {
                match a.actor_state(p) {
                    Some(ActorState::SwitchWorkers(s)) => assert_eq!(s.pool, 0.0, "case {k}"),
                    other => panic!("{other:?}"),
                }
            }
        }
    }
    // With the switch on, the crossing starts part from IW1 (the switch acts).
    let (mut a, mut b) = (sim(&text_of("is1", "RW(2)")), sim(&text_of("iw1", "RW(2)")));
    let mut parted = false;
    for _ in 0..50 {
        let (ra, rb) = (a.step().unwrap(), b.step().unwrap());
        parted |= numbers(&a, &Obs::of(&ra, &a)) != numbers(&b, &Obs::of(&rb, &b));
    }
    assert!(parted, "RW(2) with the switch on is not IW1's");
}

#[test]
fn switch_pays_both_markets() {
    // The scan's §3.3 and §3.10 test 4: a switch pop sells its own hours at its own wage and its
    // efficiency hours at the pool's, and is paid for both at settlement. In every tick of RW(2)
    // (which sends the trained into the pool and back) and of tail.services=0.2 (both types
    // pooled), each switch pop's coin moves by its transfer N_i·P_s, less what its baskets cost,
    // plus price times quantity on each market it sold on; the total coin stays put (R8) and
    // every tick's ledger closes (R2).
    let mut both = 0;
    for name in ["RW(2)", "tail.services=0.2@genesis"] {
        let mut s = sim(&text_of("is1", name));
        let w = s.world().clone();
        let good = |k: &str| w.id_of::<GoodId>(k).unwrap();
        let (coin, labour) = (good("coin"), good("labour"));
        let money = |s: &Sim| {
            w.actors
                .iter()
                .map(|a| s.holding(Holder::Actor(a.id)).map_or(0.0, |i| i.get(coin)))
                .fold(0.0, |x, y| x + y)
        };
        let m0 = money(&s);
        let provider = w.id_of::<ActorId>("provider").unwrap();
        let entrant = 2.5; // the pool's workers' heads a tick, 130 a year
                           // Each pop, its own market and its heads a tick (52 and 26 a year at 52 ticks a year).
        let pops = [
            (
                w.id_of::<ActorId>("workers.trained").unwrap(),
                good("labour.trained"),
                1.0,
            ),
            (
                w.id_of::<ActorId>("workers.master").unwrap(),
                good("labour.master"),
                0.5,
            ),
        ];
        for _ in 0..2000 {
            let before: Vec<f64> = pops
                .iter()
                .map(|(a, _, _)| s.holding(Holder::Actor(*a)).unwrap().get(coin))
                .collect();
            let chest = s.holding(Holder::Actor(provider)).unwrap().get(coin);
            let r = s.step().unwrap();
            assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0);
            assert!((money(&s) - m0).abs() <= COIN * m0);
            let price = |g: GoodId| r.markets.iter().find(|l| l.good == g).unwrap().price;
            // P_s as the provider sums it: services, goods, then space, weights 1; and its
            // transfers in list order, each at most the coin it has left (the workers first).
            let ps = ((0.0 + 1.0 * price(good("services"))) + 1.0 * price(good("goods")))
                + 1.0 * price(good("land"));
            let mut left = chest - (entrant * ps).min(chest);
            let transfers: Vec<f64> = pops
                .iter()
                .map(|(_, _, heads)| {
                    let t = (heads * ps).min(left);
                    left -= t;
                    t
                })
                .collect();
            let o = Obs::of(&r, &s);
            let (_, _, paid) = *o.transfers.iter().find(|x| x.0 == provider).unwrap();
            let all = (entrant * ps).min(chest) + transfers[0] + transfers[1];
            assert!(
                (paid - all).abs() <= COIN * paid,
                "{name}: {paid} against {all}"
            );
            for (k, (a, own, _)) in pops.iter().enumerate() {
                let mine: Vec<_> = r.settlements.iter().filter(|l| l.actor == *a).collect();
                let sold = |g: GoodId| {
                    mine.iter()
                        .find(|l| l.good == g && l.side == SideTag::Sell)
                        .map(|l| (l.qty, l.value))
                };
                let (q_own, v_own) = sold(*own).expect("its own market");
                assert!((v_own - price(*own) * q_own).abs() <= REL * v_own);
                let (q_pool, v_pool) = sold(labour).unwrap_or((0.0, 0.0));
                assert!((v_pool - price(labour) * q_pool).abs() <= REL * v_pool);
                both += usize::from(q_pool > 0.0 && q_own > 0.0);
                let spent: f64 = mine
                    .iter()
                    .filter(|l| l.side == SideTag::Buy)
                    .map(|l| l.value)
                    .sum();
                let after = s.holding(Holder::Actor(*a)).unwrap().get(coin);
                let want = before[k] + transfers[k] + v_own + v_pool - spent;
                assert!(
                    (after - want).abs() <= COIN * after.max(before[k]),
                    "{name}: pop {k}: {after} against {want}"
                );
            }
        }
    }
    assert!(both > 1000, "{both} pop-ticks sold on both markets");
}

#[test]
fn harness_reads_the_switch() {
    // The scan's §3.9 (the harness at the switch): at IS1's tail.services=0.2 (both types pooled
    // at the target, walled at genesis), tick by tick against the pops' own states in a run of
    // the same tape: the row's pool shares are the states', its reserved shares 1 − a, its gaps
    // ln(ε·w/w_i) at the prices the tick settled at; each reserved market's supply is the pop's
    // (1 − a)·N·F; the switch's readouts are the rows' ticks with a above 1e-9, the largest a,
    // the sign changes of g where |g| > 1e-6, and the last a and g. And at sw[trained]=0.5, where
    // the trained's share decays from 0.5 toward the subnormals for 2,000 ticks, most of them
    // with a in (0, 1e-9], which the live count leaves out. The CSV names the columns.
    let s = setup("is1", 52);
    let inst = &s.instance;
    let names = observables(inst);
    assert_eq!(&names[18..], ["rs.trained", "rs.master"]);
    for (name, ticks) in [
        ("tail.services=0.2@genesis", 1500),
        ("sw[trained]=0.5", 2000),
    ] {
        let (rows, rec) = read_switch(&s, name, ticks);
        let sw = rec.stats.switch.expect("the switch's readouts");
        if name.starts_with("tail") {
            for x in &sw {
                assert!(x.live > 0 && x.band > 0, "{name}: the type crosses");
            }
        } else {
            let tiny = rows
                .iter()
                .filter(|r| r.pool[0] > 0.0 && r.pool[0] <= 1e-9)
                .count();
            assert!(
                tiny > 1000 && sw[0].live < 1000,
                "{name}: {tiny}, {}",
                sw[0].live
            );
        }
    }
    let rows = read_switch(&s, "hold", 2).0;
    let h = csv_header(inst);
    for c in [
        "rs.trained",
        "pool_workers.trained",
        "pool_workers.master",
        "swgap_workers.trained",
        "swgap_workers.master",
    ] {
        assert!(h.split(',').any(|x| x == c), "{c}");
    }
    assert_eq!(h.split(',').count(), rows[0].csv(inst).split(',').count());
    // IW1's header has none of them.
    let iw1 = csv_header(&Instance::named("iw1").unwrap());
    assert!(!iw1
        .split(',')
        .any(|x| x.starts_with("rs.") || x.starts_with("pool_") || x.starts_with("swgap_")));
}

/// A run of IS1's harness with its rows, each row checked against the pops' own states in an
/// independent run of the same tape, and the switch's readouts against the rows.
fn read_switch(s: &Setup, name: &str, ticks: u64) -> (Vec<Row>, probe::markets::harness::Record) {
    let mut rows: Vec<Row> = Vec::new();
    let rec = run(s, name, ticks, &mut |r| rows.push(r.clone())).unwrap();
    let mut t = sim(&text_of("is1", name));
    let w = t.world().clone();
    let pops: Vec<ActorId> = ["workers.trained", "workers.master"]
        .iter()
        .map(|p| w.id_of::<ActorId>(p).unwrap())
        .collect();
    // Each pop's heads a tick: 52 and 26 a year at 52 ticks a year.
    let heads = [1.0, 0.5];
    let eps = [1.5, 1.8];
    let (mut live, mut max, mut band, mut last) = ([0u64; 2], [0.0f64; 2], [0u64; 2], [0i8; 2]);
    for r in &rows {
        t.step().unwrap();
        for i in 0..2 {
            let st = match t.actor_state(pops[i]) {
                Some(ActorState::SwitchWorkers(st)) => *st,
                other => panic!("{other:?}"),
            };
            assert_eq!(r.pool[i], st.pool, "tick {}", r.tick);
            assert_eq!(r.obs[18 + i], 1.0 - st.pool);
            assert_eq!(r.participation[1 + i], st.share);
            let g = rustyecon_core::num::ln((eps[i] * r.price[0]) / r.price[5 + i]);
            assert_eq!(r.switch_gap[i], g, "tick {}", r.tick);
            let own = (1.0 - st.pool) * (heads[i] * st.share);
            assert!(
                (r.supply[5 + i] - own).abs() <= REL * own,
                "tick {}",
                r.tick
            );
            live[i] += u64::from(st.pool > 1e-9);
            max[i] = max[i].max(st.pool);
            if g.abs() > 1e-6 {
                let sign = if g > 0.0 { 1 } else { -1 };
                band[i] += u64::from(last[i] != 0 && sign != last[i]);
                last[i] = sign;
            }
        }
    }
    let sw = rec.stats.switch.as_ref().expect("the switch's readouts");
    for i in 0..2 {
        assert_eq!(sw[i].live, live[i], "{name}");
        assert_eq!(sw[i].max, max[i], "{name}");
        assert_eq!(sw[i].band, band[i], "{name}");
        assert_eq!(sw[i].end, rows.last().unwrap().pool[i]);
        assert_eq!(sw[i].gap_end, rows.last().unwrap().switch_gap[i]);
    }
    (rows, rec)
}

/// A registered TSV's names, sorted as the scan wrote them.
fn names_of(tsv: &str) -> Vec<String> {
    let mut v: Vec<String> = tsv
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap().to_string())
        .collect();
    v.sort();
    v
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

/// A registered TSV's (run, tier) pairs, sorted by run.
fn tiers_of(tsv: &str) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = tsv
        .lines()
        .skip(1)
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f[0].to_string(), f[1].to_string())
        })
        .collect();
    v.sort();
    v
}

#[test]
fn switch_battery_and_families_are_the_registered_ones() {
    // The scan's §3.9 (SW6): the battery (109 runs: IW1's 103 in the wall's order, then sw[T]=V
    // for each switch pop at 0.05, 0.2 and 0.5; 28, 40 and 41 by tier), Tier 3S, stocks, joint2,
    // joint4 and basin at IS1, and IS2's battery with its own coefficient values, are the
    // registered TSVs' runs name for name (the TSVs are sorted), with the battery's tiers; the
    // tick-length and Hold sets are Tiers 1–2, the neighbourhood and the switch's rate family
    // Tier 3 (41 runs) at each setting, 10·L Tier 3 and Tier 3S. Every run parses, applies and
    // writes its tape. The observables are IW1's 18 and the two reserved shares.
    let s = setup("is1", 52);
    let inst = &s.instance;
    let b = battery(inst, 52).unwrap();
    let got: Vec<(String, String)> = b
        .iter()
        .map(|r| (r.name.clone(), r.tier.to_string()))
        .collect();
    let mut sorted_got = got.clone();
    sorted_got.sort();
    assert_eq!(sorted_got, tiers_of(RUNS_BATTERY));
    let t = |k: u8| b.iter().filter(|r| r.tier == k).count();
    assert_eq!((b.len(), t(1), t(2), t(3)), (109, 28, 40, 41));
    let iw1 = battery(&Instance::named("iw1").unwrap(), 52).unwrap();
    assert_eq!(b[..103], iw1[..]);
    assert_eq!(
        b[103..].iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
        [
            "sw[trained]=0.05",
            "sw[trained]=0.2",
            "sw[trained]=0.5",
            "sw[master]=0.05",
            "sw[master]=0.2",
            "sw[master]=0.5"
        ]
    );
    let is2 = battery(&Instance::named("is2").unwrap(), 52).unwrap();
    assert_eq!(
        sorted(is2.iter().map(|r| r.name.clone()).collect()),
        names_of(RUNS_BATTERY_IS2)
    );
    assert_eq!(
        sorted(family(inst, 52, "tier3s").unwrap()),
        names_of(RUNS_TIER3S)
    );
    assert_eq!(
        sorted(family(inst, 52, "stocks").unwrap()),
        names_of(RUNS_STOCKS)
    );
    assert_eq!(
        sorted(family(inst, 52, "joint2").unwrap()),
        names_of(RUNS_JOINT2)
    );
    assert_eq!(
        sorted(family(inst, 52, "joint4").unwrap()),
        names_of(RUNS_JOINT4)
    );
    assert_eq!(
        sorted(family(inst, 52, "basin").unwrap()),
        names_of(RUNS_BASIN)
    );
    let tier = |k: &[u8]| -> Vec<String> {
        sorted(
            b.iter()
                .filter(|r| k.contains(&r.tier))
                .map(|r| r.name.clone())
                .collect(),
        )
    };
    assert_eq!(tier(&[1, 2]), names_of(RUNS_TPY12));
    let three = tier(&[3]);
    let dedup = |tsv: &str| {
        let mut v = names_of(tsv);
        v.dedup();
        v
    };
    assert_eq!(three, dedup(RUNS_NBHD));
    assert_eq!(three, dedup(RUNS_SWITCH));
    let mut l10 = three.clone();
    l10.extend(family(inst, 52, "tier3s").unwrap());
    assert_eq!(sorted(l10), names_of(RUNS_L10));
    assert_eq!(names_of(RUNS_NBHD).len(), 17 * 41);
    assert_eq!(names_of(RUNS_SWITCH).len(), 4 * 41);
    for f in [
        "battery", "tier3s", "stocks", "joint2", "joint4", "basin", "history",
    ] {
        for name in family(inst, 52, f).unwrap() {
            let mut u = s.clone();
            Perturbation::parse(&name)
                .and_then(|p| p.apply(&mut u, RUN_TICKS))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            tape_ron(&u).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
    let mut want: Vec<String> = observables(&Instance::named("iw1").unwrap());
    want.extend(["rs.trained".to_string(), "rs.master".into()]);
    assert_eq!(observables(inst), want);
}

#[test]
fn switch_grammar_and_dials_apply_as_named() {
    // The registration's §3: sw[T]=V sets the switch pop's genesis pool share to V exactly, and
    // is refused where no pop switches or V is outside [0, 1]; it is a price start whose
    // distance is |ln((1 − V)/(1 − a*))| over the tolerance (the scan's `swb.d0`: 693.147 for
    // V = 0.5 at IS1's base); JA, JB and RW leave the pool shares at a*. The dials: 26 a year
    // for each switch pop, scaled by `rate.*` with every price rate, and alone by
    // `rate.switch.*`; IW1's dials have none.
    let s = setup("is1", 52);
    let applied = |name: &str| {
        let mut u = s.clone();
        Perturbation::parse(name).unwrap().apply(&mut u, 4000)?;
        genesis(&u)
    };
    assert_eq!(applied("sw[trained]=0.5").unwrap().switch, [0.5, 0.0]);
    assert_eq!(applied("sw[master]=0.2").unwrap().switch, [0.0, 0.2]);
    assert_eq!(applied("JB(0.5)+RW(2)").unwrap().switch, [0.0, 0.0]);
    assert!(applied("sw[trained]=1.5").is_err());
    assert!(applied("sw[nobody]=0.5").is_err());
    let mut iw1 = setup("iw1", 52);
    assert!(Perturbation::parse("sw[trained]=0.5")
        .unwrap()
        .apply(&mut iw1, 4000)
        .is_err());
    let is2 = genesis(&setup("is2", 52)).unwrap();
    assert!(
        is2.switch[0] > 0.39 && is2.switch[0] < 0.392,
        "{:?}",
        is2.switch
    );
    let rec = run(&s, "sw[trained]=0.5", 2, &mut |_| {}).unwrap();
    assert_eq!(rec.d0, rustyecon_core::num::ln(2.0) / TOL_FLOOR);
    assert!(matches!(rec.start, probe::perturb::Start::Prices));
    // The dials.
    let mut d = Dials::c2m(&s.instance);
    assert_eq!(d.get("rate.switch.trained").unwrap(), 26.0);
    assert_eq!(d.get("rate.switch.master").unwrap(), 26.0);
    d.set("rate.*", 0.75).unwrap();
    assert_eq!(d.get("rate.switch.trained").unwrap(), 19.5);
    assert_eq!(d.get("rate.labour").unwrap(), 5.2 * 0.75);
    let mut e = Dials::c2m(&s.instance);
    e.set("rate.switch.*", 1.25).unwrap();
    assert_eq!(e.get("rate.switch.master").unwrap(), 32.5);
    assert_eq!(e.get("rate.labour").unwrap(), 5.2);
    assert!(Dials::c2m(&Instance::named("iw1").unwrap())
        .get("rate.switch.trained")
        .is_err());
}

#[test]
fn markets_reports_the_switch() {
    // The command line (P2.4; the switch registration's §3): `markets run` at a switch instance
    // writes each switch pop's readouts to `stats.tsv` (`switch.live_ticks`, `switch.max`,
    // `switch.band`, `switch.end`, `switch.gap_end`, the pop as `where`), the harness's numbers,
    // and none at IW1; `--set rate.switch.*=F` scales the switch's rates alone; `markets point`
    // prints each type's reserved hours, pooled flag, a* and switch distance at a switch
    // instance.
    let dir = std::env::temp_dir().join(format!("switch-bin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let markets = env!("CARGO_BIN_EXE_markets");
    let names = ["sw[trained]=0.5", "tail.services=0.2@genesis"];
    let out = std::process::Command::new(markets)
        .args(["run", names[0], names[1], "--inst", "is1", "--ticks", "60"])
        .args(["--set", "rate.switch.*=0.5", "--csv"])
        .arg(dir.join("is1"))
        .output()
        .expect("markets runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stats = std::fs::read_to_string(dir.join("is1/stats.tsv")).unwrap();
    let mut s = setup("is1", 52);
    s.dials.set("rate.switch.*", 0.5).unwrap();
    for name in names {
        let rec = run(&s, name, 60, &mut |_| {}).unwrap();
        let sw = rec.stats.switch.unwrap();
        for (pop, x) in ["workers.trained", "workers.master"].iter().zip(&sw) {
            for (stat, value) in [
                ("switch.live_ticks", x.live.to_string()),
                ("switch.max", format!("{:.6e}", x.max)),
                ("switch.band", x.band.to_string()),
                ("switch.end", format!("{:.6e}", x.end)),
                ("switch.gap_end", format!("{:.6e}", x.gap_end)),
            ] {
                let line = format!("{name}\t{stat}\t{pop}\t{value}");
                assert!(stats.lines().any(|l| l == line), "{line}");
            }
        }
    }
    assert_eq!(
        stats.lines().filter(|l| l.contains("\tswitch.")).count(),
        2 * 2 * 5
    );
    let out = std::process::Command::new(markets)
        .args(["run", "hold", "--inst", "iw1", "--ticks", "2", "--csv"])
        .arg(dir.join("iw1"))
        .output()
        .expect("markets runs");
    assert!(out.status.success());
    let stats = std::fs::read_to_string(dir.join("iw1/stats.tsv")).unwrap();
    assert!(stats.contains("wall.breach_ticks") && !stats.contains("\tswitch."));
    let out = std::process::Command::new(markets)
        .args(["point", "--inst", "is1"])
        .output()
        .expect("markets runs");
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text
        .lines()
        .find(|l| l.starts_with("is1\ttail.services=0.2\t"))
        .expect("the target's line");
    assert!(line.contains("\tpooled [true, true]\t"), "{line}");
    assert!(line.contains("\tpool shares [0.2860879"), "{line}");
    assert!(line.contains("\tswitch distances [-0.4753"), "{line}");
    let _ = std::fs::remove_dir_all(&dir);
}
