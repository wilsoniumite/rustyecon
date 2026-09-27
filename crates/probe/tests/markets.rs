//! The markets probe's tapes, harness and nesting (MARKETS-SPEC §2.7, §7.6, §8; docs/probe/
//! MARKETS-RULES.md): each committed tape is its generator's output, loads and runs
//! deterministically, and conserves every tick; I0 in the many-market kinds nests P2.0's roles on
//! Appendix B bit for bit; and mode A holds at the oracle's point where the rest point is stable.
//!
//! One bar is relative and named here, as in `appb.rs`: `COIN`, 1e-12 of the money stock, for
//! the drift of the total coin over a run. Mode A's bars are PROBE-SPEC §4.6's, in
//! `probe::protocol`.

use probe::harness::{run as run_appb, Row as AppbRow};
use probe::markets::harness::{observables, run, Row};
use probe::markets::instance::Instance;
use probe::markets::kick::kick_set;
use probe::markets::perturb::{battery, family, slack, Perturbation};
use probe::markets::probes::run_length;
use probe::markets::setup::{tape_ron, Dials, Setup};
use probe::protocol::{RUN_TICKS, TOL_FLOOR};
use probe::setup::Setup as AppbSetup;
use rustyecon_engine::prelude::{GoodId, Holder, Provenance, Sim, Tape};

const APPB: &str = include_str!("../../../tapes/appb.ron");
const TAPES: [(&str, &str); 7] = [
    ("i0", include_str!("../../../tapes/markets-i0.ron")),
    ("i1", include_str!("../../../tapes/markets-i1.ron")),
    ("i2", include_str!("../../../tapes/markets-i2.ron")),
    ("i3", include_str!("../../../tapes/markets-i3.ron")),
    ("l2", include_str!("../../../tapes/markets-l2.ron")),
    ("l3", include_str!("../../../tapes/markets-l3.ron")),
    ("g1", include_str!("../../../tapes/markets-g1.ron")),
];
const COIN: f64 = 1e-12;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn registered(id: &str) -> Setup {
    Setup::registered(id, 52).expect("a registered instance")
}

#[test]
fn markets_tapes_are_their_generators_output() {
    // tapes/markets-<id>.ron is `markets-tape --inst <id>`'s output for the registered setup at
    // 52 ticks a year, byte for byte, so its genesis is the oracle's point and nobody edits it.
    for (id, text) in TAPES {
        let generated = tape_ron(&registered(id)).expect("the generator runs");
        assert!(
            generated == text,
            "tapes/markets-{id}.ron differs from its generator: run `cargo run -p \
             rustyecon-probe --bin markets-tape -- --inst {id} tapes/markets-{id}.ron`"
        );
    }
    assert_eq!(Instance::IDS.len(), TAPES.len());
}

#[test]
fn markets_i0_genesis_is_appb() {
    // Unit 1c's operating form of I0 is unit 1a's G1 bit for bit (MARKETS-SPEC §1.2), and the
    // generator sums the stationary coins in appb's order, so I0's genesis is appb's: every
    // genesis price and holding, by key, to the bit.
    let a = Tape::from_ron(APPB).unwrap().canonical();
    let b = Tape::from_ron(TAPES[0].1).unwrap().canonical();
    assert_eq!(a.genesis.prices, b.genesis.prices);
    assert_eq!(a.genesis.holdings, b.genesis.holdings);
    let share = |t: &Tape| -> Vec<u64> {
        t.to_ron()
            .lines()
            .filter(|l| l.contains("share:"))
            .map(|l| {
                let v: f64 = l
                    .split("share:")
                    .nth(1)
                    .unwrap()
                    .trim()
                    .trim_end_matches([',', ')'])
                    .trim()
                    .parse()
                    .unwrap();
                v.to_bits()
            })
            .collect()
    };
    assert_eq!(share(&a), share(&b));
}

/// Every number a harness reads of a tick, P2.0's row.
fn appb_numbers(r: &AppbRow) -> Vec<u64> {
    let mut v: Vec<f64> = Vec::new();
    v.extend(r.price);
    v.extend(r.obs);
    v.extend(r.target);
    v.extend(r.gap);
    v.push(r.dhat);
    v.extend(r.supply);
    v.extend(r.demand);
    v.extend(r.buyer_fill);
    v.extend(r.seller_fill);
    v.extend(r.spoiled);
    v.extend(r.coin);
    v.push(r.planned);
    v.extend(r.transfer);
    v.push(r.margin);
    v.iter().map(|x| x.to_bits()).collect()
}

/// The same numbers of the markets harness's row, in the same order.
fn markets_numbers(r: &Row) -> Vec<u64> {
    let mut v: Vec<f64> = Vec::new();
    v.extend(&r.price);
    v.extend(&r.obs);
    v.extend(&r.target);
    v.extend(&r.gap);
    v.push(r.dhat);
    v.extend(&r.supply);
    v.extend(&r.demand);
    v.extend(&r.buyer_fill);
    v.extend(&r.seller_fill);
    v.extend(&r.spoiled);
    v.extend(&r.coin);
    v.push(r.planned[0]);
    v.extend(r.transfer);
    v.push(r.margin);
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn markets_i0_nests_appb() {
    // MARKETS-SPEC §2.7: I0 in the four many-market kinds, with the basket [(good, 1), (land,
    // h)], gives the P2.0 kinds' per-tick prices, cleared volumes, fills, spoilage, outputs,
    // planned shares, coins and transfer bit for bit, and the harness reads the same
    // observables, targets, gaps and D̂: for 20,000 ticks from the registered genesis, and for
    // 3,000 ticks each from w×2, JB(0.5), b = 0.2 at genesis and machine services ×0.1. The
    // state hashes differ, because the spec variants differ; nothing a harness reads does.
    let old = AppbSetup::registered(52);
    let new = registered("i0");
    assert_eq!(
        observables(&new.instance),
        [
            "v",
            "pi.mach",
            "pi.good",
            "s.good",
            "vol.labour",
            "vol.land",
            "vol.mach",
            "vol.good",
            "y.good",
            "y.mach"
        ]
    );
    for (appb, markets, ticks) in [
        ("hold", "hold", RUN_TICKS),
        ("w*2", "w*2", 3000),
        ("JB(0.5)", "JB(0.5)", 3000),
        ("b=0.2@genesis", "land.mach=0.2@genesis", 3000),
        ("mach*0.1", "stock.mach*0.1", 3000),
    ] {
        let mut a = Vec::new();
        let ra = run_appb(&old, appb, ticks, &mut |r| a.push(appb_numbers(r))).unwrap();
        let mut b = Vec::new();
        let rb = run(&new, markets, ticks, &mut |r| {
            b.push(markets_numbers(r));
        })
        .unwrap();
        assert_eq!(a.len(), b.len(), "{appb}: ticks run");
        for (t, (x, y)) in a.iter().zip(&b).enumerate() {
            assert!(x == y, "{appb}: tick {t} differs");
        }
        let sa = probe::harness::classify(&ra);
        assert_eq!(sa.class, rb.summary.class, "{appb}");
        assert_eq!(sa.in_tol_from, rb.summary.in_tol_from, "{appb}");
        assert_eq!(sa.dead, rb.summary.dead, "{appb}");
        assert_eq!(sa.d0.to_bits(), rb.summary.d0.to_bits(), "{appb}");
        assert_eq!(sa.max_w.to_bits(), rb.summary.max_w.to_bits(), "{appb}");
        assert_eq!(
            sa.envelope.map(f64::to_bits),
            rb.summary.envelope.map(f64::to_bits),
            "{appb}"
        );
        for (m, (ta, tb)) in ra.trough.iter().zip(&rb.stats.trough).enumerate() {
            assert_eq!(
                (ta.0.to_bits(), ta.1),
                (tb.0.to_bits(), tb.1),
                "{appb}: trough {m}"
            );
        }
        assert_eq!(
            ra.transfer_short.to_bits(),
            rb.stats.transfer.0.to_bits(),
            "{appb}"
        );
    }
}

#[test]
fn markets_tapes_load_and_run_deterministically() {
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

#[test]
fn markets_conserve_every_tick() {
    // R2 on every instance, at rest and through a large transient (w ×2 and every desk's coin
    // ×0.1 at genesis, where markets ration and produced goods spoil): every tick's ledger and
    // the run's close within their registered tolerances, and the money stock stays put.
    for (id, text) in TAPES {
        let mut setup = registered(id);
        let mut name = "w*2".to_string();
        for d in setup.instance.desks() {
            name.push_str(&format!("+coin.desk.{d}*0.1"));
        }
        Perturbation::parse(&name)
            .unwrap()
            .apply(&mut setup, 2000)
            .unwrap();
        for text in [text.to_string(), tape_ron(&setup).unwrap()] {
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
                assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0, "{id}");
                assert!((money(&s) - m0).abs() <= COIN * m0, "{id}");
                if r.audit
                    .lines
                    .iter()
                    .any(|l| l.1 == Provenance::Spoilage && l.2 < 0.0)
                {
                    spoiled += 1;
                }
            }
            assert!(spoiled > 0, "{id}: the world spoils what it does not sell");
        }
    }
}

#[test]
fn markets_rest_point_is_the_oracles() {
    // MARKETS-SPEC §4.1: the oracle's point is a rest point of the roles' map, whatever its
    // stability. One tick from genesis at the oracle's f64 point leaves every observable within
    // 1e-12 of the oracle in log, every market trading with both fills 1 to rounding, at every
    // instance, at both dial sets and at every registered cost target.
    for id in Instance::IDS {
        let base = registered(id);
        let mut targets = vec![base.instance.clone()];
        for c in &base.instance.coefs {
            for v in &c.values {
                let mut i = base.instance.clone();
                i.set(&c.param, v.parse().unwrap()).unwrap();
                targets.push(i);
            }
        }
        for dials in ["c2m", "c2l"] {
            for inst in &targets {
                let mut s = base.clone();
                s.instance = inst.clone();
                s.dials = Dials::named(dials, inst).unwrap();
                let mut rows = Vec::new();
                let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
                assert_eq!(rec.hold_failure, None, "{id} {dials}");
                for r in &rows {
                    let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                    assert!(worst <= 1e-12, "{id} {dials}: {worst:e} at tick {}", r.tick);
                }
            }
        }
    }
}

#[test]
fn markets_hold_at_the_oracle_point() {
    // Mode A (PROBE-SPEC §4.6, MARKETS-SPEC §7.6): from the oracle's f64 point with each actor's
    // stationary coin, for L = 20,000 ticks, every observable stays within 1e-9 of the oracle in
    // log, every market trades with every fill at least 1 − 1e-9, no produced good spoils
    // beyond 1e-9 of its volume, and the ledger is clean: I1, I2 and I3 at C2m, and L2 and L3 at
    // C2L, where the frame predicts the rest point stable (§6.2). A loop with growth g per tick
    // would cross 1e-9 within ln(1e6)/ln(g) ticks. L2 and L3 at C2m are predicted unstable; the
    // harness reports them (P2.1.2), and this test does not pin a result.
    for (id, dials) in [
        ("i1", "c2m"),
        ("i2", "c2m"),
        ("i3", "c2m"),
        ("l2", "c2l"),
        ("l3", "c2l"),
    ] {
        let mut s = registered(id);
        s.dials = Dials::named(dials, &s.instance).unwrap();
        let rec = run(&s, "hold", RUN_TICKS, &mut |_| {}).unwrap();
        assert_eq!(rec.stop, probe::harness::Stop::Ran, "{id}");
        assert_eq!(rec.hold_failure, None, "{id} {dials}");
        let worst = rec.stats.peak.0 * TOL_FLOOR;
        assert!(worst <= 1e-12, "{id}: the largest gap was {worst:e}");
    }
}

#[test]
fn markets_batteries_are_registered() {
    // MARKETS-SPEC §7.7: every price and technique alone at six factors, the joint shapes, RC
    // with several categories and RT with two types, x*/2, and each registered cost coefficient
    // at four values at genesis and dated: 107, 77, 119, 77 and 119 runs for I1, I2, I3, L2 and
    // L3 by tier, and the slack runs are the frame's (slack.out).
    let counts = |id: &str| {
        let b = battery(&registered(id).instance, 52).unwrap();
        let t = |k: u8| b.iter().filter(|r| r.tier == k).count();
        (b.len(), t(1), t(2), t(3))
    };
    assert_eq!(counts("i1"), (107, 30, 38, 39));
    assert_eq!(counts("i2"), (77, 20, 28, 29));
    assert_eq!(counts("i3"), (119, 34, 42, 43));
    assert_eq!(counts("l2"), (77, 20, 28, 29));
    assert_eq!(counts("l3"), (119, 34, 42, 43));
    assert_eq!(counts("i0").0, 57);
    let slack_runs = |id: &str| -> Vec<String> {
        battery(&registered(id).instance, 52)
            .unwrap()
            .into_iter()
            .filter(|r| r.slack)
            .map(|r| r.name)
            .collect()
    };
    let every = |d: &str| {
        ["1.05", "0.95", "1.2", "0.8", "2", "0.5"]
            .iter()
            .map(|f| format!("s[{d}]*{f}"))
            .collect::<Vec<_>>()
    };
    let sorted = |mut v: Vec<String>| {
        v.sort();
        v
    };
    let mut i1 = every("manufactures");
    i1.extend(["s[care]*1.05", "s[care]*0.95", "s[care]*1.2", "s[care]*2"].map(String::from));
    assert_eq!(sorted(slack_runs("i1")), sorted(i1));
    let mut i3 = every("manufactures");
    for d in ["food", "shelter"] {
        i3.extend(every(d).into_iter().filter(|n| !n.ends_with("*2")));
    }
    assert_eq!(sorted(slack_runs("i3")), sorted(i3));
    let mut l3 = every("manufactures");
    l3.extend(["s[care]*1.05", "s[care]*1.2", "s[care]*2"].map(String::from));
    assert_eq!(sorted(slack_runs("l3")), sorted(l3));
    assert!(slack_runs("i2").is_empty() && slack_runs("l2").is_empty());
    // Every run of every battery and family parses and applies. In G1, whose 1 − x* is 0.545,
    // a technique ×2 (each s ×2, JA(2), JB(0.5)) would put 1 − x above 1, so those six runs
    // are not expressible, and the generator refuses them rather than clamp.
    for id in Instance::IDS {
        let s = registered(id);
        let mut refused = Vec::new();
        for f in ["battery", "stocks", "joint2", "joint4", "basin", "history"] {
            for name in family(&s.instance, 52, f).unwrap() {
                let mut t = s.clone();
                Perturbation::parse(&name)
                    .and_then(|p| p.apply(&mut t, RUN_TICKS))
                    .unwrap_or_else(|e| panic!("{id} {name}: {e}"));
                match tape_ron(&t) {
                    Ok(_) => {}
                    // G1's joint draws of the technique within ×/÷2 can pass 1 too.
                    Err(e) if e.contains("outside [0, 1]") && f != "battery" && id == "g1" => {}
                    Err(e) if e.contains("outside [0, 1]") && f == "battery" => refused.push(name),
                    Err(e) => panic!("{id} {name}: {e}"),
                }
            }
        }
        let want: Vec<String> = if id == "g1" {
            let mut v: Vec<String> = ["manufactures", "food", "care", "shelter"]
                .iter()
                .map(|d| format!("s[{d}]*2"))
                .collect();
            v.extend(["JA(2)".to_string(), "JB(0.5)".to_string()]);
            v
        } else {
            Vec::new()
        };
        assert_eq!(sorted(refused), sorted(want), "{id}");
    }
    // The slack rule on a hand case: food has no tasks above 0.75, so moving I1's threshold
    // from 0.8 to 0.9 changes nothing it makes.
    let i1 = registered("i1").instance;
    assert!(slack(&i1, 1, 0.8, 0.9));
    assert!(!slack(&i1, 1, 0.7, 0.9));
}

#[test]
fn markets_shocks_and_shapes_apply_as_named() {
    // The grammar of MARKETS-SPEC §7.7: a dated shock lands at L/4 through a SetParam of its
    // schedule param and restarts the clock; a shock at genesis changes the tape's coefficient
    // and not genesis; RC and RT move the relative prices they name.
    let s = registered("i3");
    let mut t = s.clone();
    let p = Perturbation::parse("b.food=1.2@dated").unwrap();
    p.apply(&mut t, 4000).unwrap();
    assert_eq!(p.clock_start(4000), 1000);
    assert_eq!(t.changes_at(999), vec![]);
    assert_eq!(
        t.changes_at(1000),
        vec![("inst.food.land".to_string(), 1.2)]
    );
    let mut sm = sim(&tape_ron(&t).unwrap());
    let food_land = sm
        .world()
        .id_of::<rustyecon_engine::prelude::ParamId>("inst.food.land")
        .unwrap();
    for _ in 0..1000 {
        sm.step().unwrap();
    }
    assert_eq!(sm.param(food_land), Some(0.6));
    sm.step().unwrap();
    assert_eq!(sm.param(food_land), Some(1.2));
    let mut g = s.clone();
    Perturbation::parse("land.power=1@genesis")
        .unwrap()
        .apply(&mut g, 4000)
        .unwrap();
    let a = probe::markets::setup::genesis(&s).unwrap();
    let b = probe::markets::setup::genesis(&g).unwrap();
    assert_eq!(a, b);
    assert!(tape_ron(&g)
        .unwrap()
        .contains("(key: \"inst.power.land\", value: 1.0,"));
    let mut r = s.clone();
    Perturbation::parse("RC(2)+RT(0.5)")
        .unwrap()
        .apply(&mut r, 4000)
        .unwrap();
    // Markets: labour, land, engine, power, manufactures, food, care, shelter.
    assert_eq!(
        r.displace.price,
        vec![1.0, 1.0, 0.5, 2.0, 2.0, 2.0, 0.5, 0.5]
    );
}

#[test]
fn markets_kick_set_decays_at_a_stable_point() {
    // MARKETS-SPEC §7.5 (MG7): the kick set at the end of I1's mode-A run, with a short horizon,
    // runs one base continuation and ± each market, every kick is realised, and a 1e-9 kick at
    // a stable point decays by certify's bars. At L2 under C2m, whose point the frame predicts
    // unstable, the same set fails.
    let k = kick_set(&registered("i1"), "hold", 200, 2000).unwrap();
    assert_eq!(k.kicks.len(), 2 * 7);
    assert!(k.kicks.iter().all(|x| x.error.is_none() && x.size > 0.0));
    assert!(k.pass, "{:?}", k.notes);
    let l2 = kick_set(&registered("l2"), "hold", 200, 2000).unwrap();
    assert!(!l2.pass);
}

#[test]
fn run_length_follows_the_rule() {
    // PROBE-SPEC §4.4: L = 200·τ_max rounded up to a thousand ticks, and at least 20,000.
    assert_eq!(run_length(67.7), 20_000);
    assert_eq!(run_length(519.4), 104_000);
    assert_eq!(run_length(2419.0), 484_000);
}
