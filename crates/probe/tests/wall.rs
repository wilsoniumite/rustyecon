//! The wall instance on the engine (P2.3; docs/probe/WALL-RULES.md; the wall frame,
//! docs/probe/wall/SPEC.md §3.8): the tape is its generator's output, genesis is unit 1d's
//! point, the point is a rest point of the roles at every target and tick length, the world
//! conserves and runs deterministically, the roles' three optional fields nest P2.1's path when
//! they are at zero, every P2.1 world keeps its ids, and the battery and families are the
//! registered ones, run name for run name.
//!
//! One bar is relative and named here, as in `markets.rs`: `COIN`, 1e-12 of the money stock,
//! for the drift of the total coin over a run. Mode A's bars are PROBE-SPEC §4.6's, in
//! `probe::protocol`.

use certify::{tape_hash, Obs};
use probe::markets::harness::{observables, run, Row};
use probe::markets::instance::Instance;
use probe::markets::perturb::{battery, family, Perturbation};
use probe::markets::setup::{genesis, tape_ron, Setup, ShareAt};
use probe::protocol::{RUN_TICKS, TOL_FLOOR};
use rustyecon_engine::prelude::{ActorId, GoodId, Holder, Provenance, Sim, Tape};

const IW1: &str = include_str!("../../../tapes/markets-iw1.ron");
const I1: &str = include_str!("../../../tapes/markets-i1.ron");
const RUNS_BATTERY: &str = include_str!("../../../docs/probe/wall/registered/runs_battery.tsv");
const RUNS_TIER3S: &str = include_str!("../../../docs/probe/wall/registered/runs_tier3s.tsv");
const RUNS_STOCKS: &str = include_str!("../../../docs/probe/wall/registered/runs_stocks.tsv");
const RUNS_BASIN: &str = include_str!("../../../docs/probe/wall/registered/runs_basin.tsv");
const RUNS_JOINT2: &str = include_str!("../../../docs/probe/wall/registered/runs_joint2.tsv");
const RUNS_JOINT4: &str = include_str!("../../../docs/probe/wall/registered/runs_joint4.tsv");
const RUNS_LINE: &str = include_str!("../../../docs/probe/wall/registered/runs_line.tsv");
const RUNS_TPY12: &str = include_str!("../../../docs/probe/wall/registered/runs_tpy12.tsv");
const COIN: f64 = 1e-12;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn iw1(tpy: u32) -> Setup {
    Setup::registered("iw1", tpy).expect("the wall instance")
}

/// A registered TSV's (run, tier) columns.
fn runs(tsv: &str) -> Vec<(String, String)> {
    let mut lines = tsv.lines();
    let head: Vec<&str> = lines.next().unwrap().split('\t').collect();
    assert_eq!(head[..2], ["run", "tier"]);
    lines
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f[0].to_string(), f[1].to_string())
        })
        .collect()
}

#[test]
fn markets_iw1_tape_is_its_generators_output() {
    // tapes/markets-iw1.ron is `markets-tape --inst iw1`'s output for the registered setup at
    // 52 ticks a year, byte for byte, so its genesis is unit 1d's point and nobody edits it.
    let generated = tape_ron(&iw1(52)).expect("the generator runs");
    assert!(
        generated == IW1,
        "tapes/markets-iw1.ron differs from its generator: run `cargo run -p rustyecon-probe \
         --bin markets-tape -- --inst iw1 tapes/markets-iw1.ron`"
    );
}

#[test]
fn markets_iw1_genesis_is_unit_1d() {
    // The wall frame's §2.2 and §3.6: the generator's oracle point is unit 1d's at the base,
    // double for double (the frame's registered points.jsonl), a solved wall (margin Wall, x* 1,
    // 1 − x* exactly 0); the genesis prices are its ratios with r = 1 in market order, each
    // reserved wage its market's; and the stationary coins, summed in the rules' order, are
    // the frame's mirror's (`wall.genesis`), bit for bit.
    let s = iw1(52);
    let g = genesis(&s).unwrap();
    let e = &g.point;
    assert_eq!(e.margin, "Wall");
    assert_eq!((e.x_star, e.one_minus_x), (1.0, 0.0));
    assert_eq!(e.v, 0.807_309_943_784_067_7);
    assert_eq!(e.p_s, 1.532_910_467_439_577_3);
    assert_eq!(e.y, 7.692_307_692_307_692);
    assert_eq!(e.pool, 1.057_692_307_692_307_7);
    assert_eq!(e.wage, [1.303_538_227_228_111_3, 2.325_460_702_671_447]);
    assert_eq!(
        e.hours,
        [
            1.057_692_307_692_307_7,
            0.307_692_307_692_307_65,
            0.230_769_230_769_230_73
        ]
    );
    assert_eq!(e.type_price, [0.629_093_567_413_147_8]);
    assert_eq!(e.type_services, [5.769_230_769_230_771]);
    assert_eq!(
        e.cat_price,
        [0.274_418_576_135_489_5, 0.258_491_891_304_087_8]
    );
    assert_eq!(e.provider_baskets, 2.523_538_205_530_696);
    assert_eq!(
        s.instance.markets(),
        [
            "labour",
            "land",
            "mach",
            "services",
            "goods",
            "labour.trained",
            "labour.master"
        ]
    );
    assert_eq!(
        g.prices,
        [
            e.v,
            1.0,
            e.type_price[0],
            e.cat_price[0],
            e.cat_price[1],
            e.wage[0],
            e.wage[1]
        ]
    );
    assert_eq!(g.shares, [0.0, 0.0]);
    assert_eq!(
        g.coin,
        [
            22.182_165_306_494_213,
            20.894_758_452_738_962,
            26.697_154_739_365_7,
            23.619_760_426_210_647,
            21.185_254_410_622_942,
            8.743_245_928_238_728,
            5.891_069_730_504_536
        ]
    );
    assert_eq!(
        s.instance.actors(),
        [
            "desk.services",
            "desk.goods",
            "desk.mach",
            "provider",
            "workers",
            "workers.trained",
            "workers.master"
        ]
    );
}

#[test]
fn markets_iw1_rest_point_is_the_oracles() {
    // The wall frame's §3.7 and §4.2: unit 1d's point is a rest point of the roles' map. Three
    // ticks from genesis at the oracle's f64 point leave every observable within 1e-12 of the
    // oracle in log, every market trading with both fills 1 to rounding, at the base and the 12
    // cost targets, at 12, 52 and 365 ticks a year. The depth at the base is the frame's
    // ln(g/γ(1)), 0.9426.
    for tpy in [12, 52, 365] {
        let base = iw1(tpy);
        let mut targets = vec![base.instance.clone()];
        for c in &base.instance.coefs {
            for v in &c.values {
                let mut i = base.instance.clone();
                i.set(&c.param, v.parse().unwrap()).unwrap();
                targets.push(i);
            }
        }
        assert_eq!(targets.len(), 13);
        for inst in targets {
            let mut s = base.clone();
            s.instance = inst;
            let mut rows: Vec<Row> = Vec::new();
            let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
            assert_eq!(rec.hold_failure, None, "tpy {tpy}");
            for r in &rows {
                let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                assert!(worst <= 1e-12, "tpy {tpy}: {worst:e} at tick {}", r.tick);
            }
            assert_eq!(rec.genesis.point.margin, "Wall");
        }
    }
    let mut rows: Vec<Row> = Vec::new();
    run(&iw1(52), "hold", 1, &mut |r| rows.push(r.clone())).unwrap();
    assert!((rows[0].depth - 0.942_574_842_758_311).abs() <= 1e-12);
    assert_eq!(rows[0].participation.len(), 3);
}

#[test]
fn markets_iw1_holds_at_the_oracle_point() {
    // Mode A (PROBE-SPEC §4.6; the wall frame's §7 E2) at 52 ticks a year for L = 20,000 ticks:
    // every observable stays within 1e-9 of the oracle in log, every market trades with every
    // fill at least 1 − 1e-9, no produced good spoils beyond 1e-9 of its volume, and the thresholds
    // stay at the wall, x = 1 exactly.
    let rec = run(&iw1(52), "hold", RUN_TICKS, &mut |r| {
        assert!(
            r.obs[6] == 1.0 && r.obs[7] == 1.0,
            "tick {}: the thresholds left the wall",
            r.tick
        );
    })
    .unwrap();
    assert_eq!(rec.stop, probe::harness::Stop::Ran);
    assert_eq!(rec.hold_failure, None);
    let worst = rec.stats.peak.0 * TOL_FLOOR;
    assert!(worst <= 1e-12, "the largest gap was {worst:e}");
    let wall = rec.stats.wall.expect("the wall's readouts");
    assert_eq!(wall.breach, 0);
    assert_eq!(wall.share_max, [0.0, 0.0]);
}

#[test]
fn markets_iw1_conserves_and_is_deterministic() {
    // R8: two runs of the tape give identical reports and hash streams, and the state moves.
    // R2: at rest and through a large transient (w ×2, every reserved wage ×0.5 and every
    // desk's coin ×0.1 at genesis), every tick's ledger and the run's close within their
    // registered tolerances, the money stock stays put, and the world spoils what it does not
    // sell.
    let (mut a, mut b) = (sim(IW1), sim(IW1));
    let genesis_hash = a.hash();
    for _ in 0..500 {
        let ra = a.step().expect("the world runs");
        let rb = b.step().expect("the world runs");
        assert_eq!(ra, rb);
    }
    assert_eq!(a.hash(), b.hash());
    assert_ne!(a.hash(), genesis_hash);
    let mut setup = iw1(52);
    let mut name = "w*2+p[labour.trained]*0.5+p[labour.master]*0.5".to_string();
    for d in setup.instance.desks() {
        name.push_str(&format!("+coin.desk.{d}*0.1"));
    }
    Perturbation::parse(&name)
        .unwrap()
        .apply(&mut setup, 2000)
        .unwrap();
    for text in [IW1.to_string(), tape_ron(&setup).unwrap()] {
        let mut s = sim(&text);
        let coin = s.world().id_of::<GoodId>("coin").unwrap();
        let money = |s: &Sim| {
            s.world()
                .actors
                .iter()
                .map(|a| s.holding(Holder::Actor(a.id)).map_or(0.0, |i| i.get(coin)))
                .fold(0.0, |x, y| x + y)
        };
        let m0 = money(&s);
        let mut spoiled = 0;
        for _ in 0..2000 {
            let r = s.step().expect("no breach stops the run");
            assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0);
            assert!((money(&s) - m0).abs() <= COIN * m0);
            if r.audit
                .lines
                .iter()
                .any(|l| l.1 == Provenance::Spoilage && l.2 < 0.0)
            {
                spoiled += 1;
            }
        }
        assert!(spoiled > 0, "the world spoils what it does not sell");
    }
}

/// I1's tape with the wall's fields at zero (the wall frame's §3.8): every category desk with a
/// tail of 0 and a reserved input at coefficient 0 on `zlabour`, and the provider with a further
/// transfer to `zworkers`, a pop of 0 heads that sells `zlabour`.
fn zeroed_i1() -> String {
    let mut t = I1.to_string();
    let mut rep = |from: &str, to: &str| {
        assert_eq!(t.matches(from).count(), 1, "{from:?}");
        t = t.replacen(from, to, 1);
    };
    rep(
        "    params: [\n",
        "    params: [\n        (key: \"zz.tail\", value: 0.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.reserved\", value: 0.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.heads\", value: 0.0, unit: FlowPerYear, basis: Assumed(\"test\")),\n        (key: \"zz.chi_max\", value: 1.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"rate.zlabour\", value: 5.2, unit: RatePerYear, basis: Assumed(\"test\")),\n",
    );
    rep(
        "    goods: [\n",
        "    goods: [\n        (key: \"zlabour\", life: Instant, price_rate: Some(\"rate.zlabour\")),\n",
    );
    rep(
        "\"owners\", \"workers\"],",
        "\"owners\", \"workers\", \"zworkers\"],",
    );
    for c in ["manufactures", "food", "care", "shelter"] {
        rep(
            &format!("tilt: \"tilt.desk.{c}\", payout: None)),\n"),
            &format!(
                "tilt: \"tilt.desk.{c}\", payout: None)),\n            tail: Some(\"zz.tail\"),\n            reserved: [(good: \"zlabour\", coef: \"zz.reserved\")],\n"
            ),
        );
    }
    rep(
        "            spend: \"spend.provider\",\n",
        "            spend: \"spend.provider\",\n            more: [(to: \"zworkers\", heads: \"zz.heads\")],\n",
    );
    let basket = I1
        .lines()
        .find(|l| l.trim_start().starts_with("basket:"))
        .expect("a basket");
    rep(
        "    ],\n    genesis: (\n",
        &format!(
            "        (key: \"zworkers\", kind: Pop, class: \"zworkers\", home: \"home\", basis: Assumed(\"test\"),\n         spec: BasketWorkers((\n            labour: \"zlabour\", heads: \"zz.heads\", chi_max: \"zz.chi_max\",\n{basket}\n            spend: \"spend.workers\",\n         ))),\n    ],\n    genesis: (\n"
        ),
    );
    rep(
        "        prices: [\n",
        "        prices: [\n            (node: \"home\", good: \"zlabour\", price: 1.0),\n",
    );
    t
}

/// Every number of one tick of the old markets and actors, by key: each market's posted price,
/// next price, S, D, cleared volume and fills, and each actor's coin.
fn numbers(sim: &Sim, obs: &Obs, markets: &[&str], actors: &[&str]) -> Vec<u64> {
    let w = sim.world();
    let coin = w.id_of::<GoodId>("coin").unwrap();
    let mut v = Vec::new();
    for m in markets {
        let g = w.id_of::<GoodId>(m).unwrap();
        let l = obs.markets.iter().find(|l| l.good == g).unwrap();
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
    for a in actors {
        let id = w.id_of::<ActorId>(a).unwrap();
        v.push(sim.holding(Holder::Actor(id)).map_or(0.0, |i| i.get(coin)));
    }
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn wall_roles_nest_the_many_roles() {
    // R1 at the run level (the wall frame's §3.8): I1's tape with a tail of 0, a reserved input
    // at coefficient 0 on every category desk and a further transfer to a pop of 0 heads gives
    // every old market's prices, S, D, cleared volume and fills, and every old actor's coin, bit
    // for bit as the field-free tape does, for 2,000 ticks, from genesis and through w ×2. An
    // addition that moved the old path by an ulp would part them.
    let markets = [
        "labour",
        "land",
        "mach",
        "manufactures",
        "food",
        "care",
        "shelter",
    ];
    let actors = [
        "desk.manufactures",
        "desk.food",
        "desk.care",
        "desk.shelter",
        "desk.mach",
        "provider",
        "workers",
    ];
    let zeroed = zeroed_i1();
    // w ×2 at genesis, written into both tapes' genesis prices alike.
    let bump = |t: &str| {
        let key = "(node: \"home\", good: \"labour\", price: ";
        let start = t.find(key).unwrap() + key.len();
        let end = start + t[start..].find(')').unwrap();
        let p: f64 = t[start..end].parse().unwrap();
        format!("{}{:?}{}", &t[..start], p * 2.0, &t[end..])
    };
    for (a_text, b_text) in [(I1.to_string(), zeroed.clone()), (bump(I1), bump(&zeroed))] {
        let (mut a, mut b) = (sim(&a_text), sim(&b_text));
        for t in 0..2000 {
            let ra = a.step().unwrap();
            let rb = b.step().unwrap();
            let (oa, ob) = (Obs::of(&ra, &a), Obs::of(&rb, &b));
            assert!(
                numbers(&a, &oa, &markets, &actors) == numbers(&b, &ob, &markets, &actors),
                "tick {t} differs"
            );
        }
    }
}

#[test]
fn markets_tapes_keep_their_world_ids() {
    // R1 for the three optional fields: `world_id` hashes the resolved actors, so a field left
    // in the resolved spec when absent would move every markets tape's `world_id`; the text and
    // `tape_hash` would stay. Each is the one the pre-build binary recorded (build-wall's
    // base streams, 2026-09-30, at f0c6667).
    for (text, tape, world) in [
        (
            include_str!("../../../tapes/markets-i0.ron"),
            0x9338_5d64_d42f_858e_u64,
            0x5068_f2ff_9dbb_fa00_u64,
        ),
        (I1, 0x4dbe_0ba9_88f9_4b02, 0x965e_dbef_2664_b850),
        (
            include_str!("../../../tapes/markets-i2.ron"),
            0x8917_3d95_2d0b_5e65,
            0x6c3e_2217_b9ee_fef9,
        ),
        (
            include_str!("../../../tapes/markets-i3.ron"),
            0xdea2_2786_21a5_2649,
            0xb631_f8a0_f7d9_ab95,
        ),
        (
            include_str!("../../../tapes/markets-l2.ron"),
            0x8aa5_0fca_e92a_6260,
            0xb47c_5301_5f7f_1637,
        ),
        (
            include_str!("../../../tapes/markets-l3.ron"),
            0xde5e_9b54_8d5d_e83e,
            0xd525_2228_af8a_95a6,
        ),
        (
            include_str!("../../../tapes/markets-g1.ron"),
            0xff3c_23ea_26a6_464e,
            0x4b0b_1301_cb7a_a351,
        ),
    ] {
        let t = Tape::from_ron(text).unwrap();
        assert_eq!(tape_hash(&t), tape, "{}", t.header.name);
        assert_eq!(sim(text).world().world_id, world, "{}", t.header.name);
    }
}

#[test]
fn wall_battery_and_families_are_the_registered_ones() {
    // The wall frame's §5.2 and §6: the battery (103 runs: 26, 38 and 39 by tier), Tier 3S, the
    // stocks, joint and basin families at IW1, and IC1's battery are the registered TSVs' runs,
    // name for name and in order, with the battery's tiers; the tick-length and one-sided sets
    // are its Tiers 1–2. Every run parses, applies and writes its tape.
    let s = iw1(52);
    let inst = &s.instance;
    let b = battery(inst, 52).unwrap();
    let got: Vec<(String, String)> = b
        .iter()
        .map(|r| (r.name.clone(), r.tier.to_string()))
        .collect();
    assert_eq!(got, runs(RUNS_BATTERY));
    let t = |k: u8| b.iter().filter(|r| r.tier == k).count();
    assert_eq!((b.len(), t(1), t(2), t(3)), (103, 26, 38, 39));
    assert!(b.iter().all(|r| !r.slack));
    let names = |tsv: &str| -> Vec<String> { runs(tsv).into_iter().map(|(n, _)| n).collect() };
    assert_eq!(family(inst, 52, "tier3s").unwrap(), names(RUNS_TIER3S));
    assert_eq!(family(inst, 52, "stocks").unwrap(), names(RUNS_STOCKS));
    assert_eq!(family(inst, 52, "joint2").unwrap(), names(RUNS_JOINT2));
    assert_eq!(family(inst, 52, "joint4").unwrap(), names(RUNS_JOINT4));
    assert_eq!(family(inst, 52, "basin").unwrap(), names(RUNS_BASIN));
    let low: Vec<String> = b
        .iter()
        .filter(|r| r.tier <= 2)
        .map(|r| r.name.clone())
        .collect();
    assert_eq!(low, names(RUNS_TPY12));
    let ic1 = Setup::registered("ic1", 52).unwrap();
    let line: Vec<String> = battery(&ic1.instance, 52)
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(line, names(RUNS_LINE));
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
    assert_eq!(
        observables(inst),
        [
            "v",
            "pi.mach",
            "pi.services",
            "pi.goods",
            "w.trained",
            "w.master",
            "x.services",
            "x.goods",
            "vol.labour",
            "vol.land",
            "vol.mach",
            "vol.services",
            "vol.goods",
            "vol.labour.trained",
            "vol.labour.master",
            "y.services",
            "y.goods",
            "y.mach"
        ]
    );
}

#[test]
fn wall_grammar_applies_as_named() {
    // The wall frame's §5.2: s[D]=V sets the share to V exactly; x*/2 puts every desk at x =
    // 1/2; JA and JB scale every labour market's price as w; RW moves the pool's wage against
    // the reserved wages; RC moves services against goods; joint draws in the mirror's market
    // order (labour, land, the categories, the type, the reserved labour markets).
    let s = iw1(52);
    let applied = |name: &str| {
        let mut u = s.clone();
        Perturbation::parse(name)
            .unwrap()
            .apply(&mut u, 4000)
            .unwrap();
        u
    };
    // Markets: labour, land, mach, services, goods, labour.trained, labour.master.
    assert_eq!(
        applied("RW(2)").displace.price,
        [2.0, 1.0, 1.0, 1.0, 1.0, 0.5, 0.5]
    );
    assert_eq!(
        applied("JA(2)").displace.price,
        [2.0, 1.0, 2.0, 2.0, 2.0, 2.0, 2.0]
    );
    assert_eq!(
        applied("JB(2)").displace.price,
        [2.0, 1.0, 0.5, 2.0, 2.0, 2.0, 2.0]
    );
    assert_eq!(
        applied("RC(2)").displace.price,
        [1.0, 1.0, 1.0, 2.0, 0.5, 1.0, 1.0]
    );
    let u = applied("s[goods]=0.05");
    assert_eq!(u.displace.share, [ShareAt::Times(1.0), ShareAt::Is(0.05)]);
    assert_eq!(genesis(&u).unwrap().shares, [0.0, 0.05]);
    assert_eq!(genesis(&applied("x*/2")).unwrap().shares, [0.5, 0.5]);
    // joint(F,SEED): SplitMix64 from SEED, u on [−1, 1], F^u for each price in the mirror's
    // order, then 2^u for each share.
    let mut state: u64 = 7;
    let mut signed = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        2.0 * ((z >> 11) as f64 / (1u64 << 53) as f64) - 1.0
    };
    let mut want = [1.0; 7];
    for m in [0, 1, 3, 4, 2, 5, 6] {
        want[m] *= rustyecon_core::num::pow(4.0, signed());
    }
    assert_eq!(applied("joint(4,7)").displace.price, want);
    // At P2.1's instances the order is the markets' own, as before.
    assert_eq!(Instance::named("i1").unwrap().wtypes.len(), 0);
}
