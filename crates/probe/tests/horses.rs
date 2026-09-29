//! The stocks probe's tapes, harness and nesting (HORSES-SPEC §1.6, §4, §7.6, §8; docs/probe/
//! HORSES-RULES.md): each committed tape is its generator's output, loads and runs
//! deterministically, and conserves every tick, its wear included; the stocks layer off (R1a)
//! reproduces P2.1's I0 and appb's ex-post variant bit for bit; the oracle's point is a rest
//! point of the roles' map at every instance and target; and mode A holds at the verdict
//! instances.
//!
//! Two bars are relative and named here: `COIN`, 1e-12 of the money stock, for the drift of the
//! total coin over a run, as in `markets.rs`; and `REST`, 1e-12 in log, for three ticks from
//! the oracle's f64 point (HORSES-SPEC §8). Mode A's bars are PROBE-SPEC §4.6's, in
//! `probe::protocol`.

use probe::harness::{run as run_appb, Row as AppbRow, Stop};
use probe::horses::harness::{observables, run, Record, Row};
use probe::horses::instance::{Instance, HOURS};
use probe::horses::kick::{kick_set, slowest_mode, three_t6};
use probe::horses::perturb::{battery, family, tier3s, Perturbation};
use probe::horses::setup::{stocks, tape_ron, Setup};
use probe::markets::harness::{run as run_markets, Row as MarketsRow};
use probe::markets::setup::Setup as MarketsSetup;
use probe::protocol::TOL_FLOOR;
use probe::setup::{Assign as AppbAssign, Setup as AppbSetup};
use rustyecon_core::{FractionPerYear, StateDelta};
use rustyecon_engine::prelude::{ActorId, GoodId, Holder, Phase, Provenance, Sim, Tape};
use rustyecon_engine::rustyecon_agents::ActorState;

const TAPES: [(&str, &str); 5] = [
    ("h1", include_str!("../../../tapes/horses-h1.ron")),
    ("h2", include_str!("../../../tapes/horses-h2.ron")),
    ("h3", include_str!("../../../tapes/horses-h3.ron")),
    ("h4", include_str!("../../../tapes/horses-h4.ron")),
    ("r1a", include_str!("../../../tapes/horses-r1a.ron")),
];
const COIN: f64 = 1e-12;
const REST: f64 = 1e-12;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn registered(id: &str) -> Setup {
    Setup::registered(id, 52).expect("a registered instance")
}

#[test]
fn horses_tapes_are_their_generators_output() {
    // tapes/horses-<id>.ron is `horses-tape --inst <id>`'s output for the registered setup at 52
    // ticks a year, byte for byte, so its genesis is the oracle's point and nobody edits it.
    for (id, text) in TAPES {
        let generated = tape_ron(&registered(id)).expect("the generator runs");
        assert!(
            generated == text,
            "tapes/horses-{id}.ron differs from its generator: run `cargo run -p \
             rustyecon-probe --bin horses-tape -- --inst {id} tapes/horses-{id}.ron`"
        );
    }
    // Every registered instance writes a tape that loads.
    for id in Instance::IDS {
        let text = tape_ron(&registered(id)).unwrap_or_else(|e| panic!("{id}: {e}"));
        sim(&text);
    }
}

#[test]
fn horses_r1a_genesis_is_i0s() {
    // The stocks layer off writes P2.1's I0 genesis: every genesis price and holding by key,
    // and the good desk's human share, to the bit.
    let i0 = Tape::from_ron(include_str!("../../../tapes/markets-i0.ron"))
        .unwrap()
        .canonical();
    let r1a = Tape::from_ron(TAPES[4].1).unwrap().canonical();
    assert_eq!(i0.genesis.prices, r1a.genesis.prices);
    assert_eq!(i0.genesis.holdings, r1a.genesis.holdings);
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
fn markets_numbers(r: &MarketsRow) -> Vec<u64> {
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

/// The same numbers of the stocks harness's row, in the same order.
fn horses_numbers(r: &Row) -> Vec<u64> {
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

/// The runs R1a is checked on (HORSES-SPEC §1.6): the registered genesis for 20,000 ticks, and
/// four displaced starts for 3,000 each, named in each harness's grammar.
const R1A_RUNS: [(&str, &str, &str, u64); 5] = [
    ("hold", "hold", "hold", 20_000),
    ("w*2", "w*2", "w*2", 3000),
    ("JB(0.5)", "JB(0.5)", "JB(0.5)", 3000),
    (
        "b=0.2@genesis",
        "land.mach=0.2@genesis",
        "b=0.2@genesis",
        3000,
    ),
    ("mach*0.1", "stock.mach*0.1", "stock.mach*0.1", 3000),
];

#[test]
fn horses_r1a_nests_i0() {
    // R1 (HORSES-SPEC §1.6, §4.4): the owner desk and the maker at δ = 1 a tick, ω 0, with
    // P2.1's households and dials, give P2.1's I0 per-tick prices, observables, targets, gaps,
    // D̂, supply, demand, fills, spoilage, coins, planned share, transfer and ledger margin bit
    // for bit, and the classes, D̂_0, envelopes, W's largest D̂, dead ticks, troughs and
    // transfer shortfalls agree. The state hashes differ, because the spec variants differ.
    let i0 = MarketsSetup::registered("i0", 52).unwrap();
    let r1a = registered("r1a");
    assert_eq!(
        observables(&r1a.instance),
        probe::markets::harness::observables(&i0.instance)
    );
    for (_, markets, horses, ticks) in R1A_RUNS {
        let mut a = Vec::new();
        let ra = run_markets(&i0, markets, ticks, &mut |r| a.push(markets_numbers(r))).unwrap();
        let mut b = Vec::new();
        let rb = run(&r1a, horses, ticks, &mut |r| b.push(horses_numbers(r))).unwrap();
        assert_eq!(a.len(), b.len(), "{markets}: ticks run");
        for (t, (x, y)) in a.iter().zip(&b).enumerate() {
            assert!(x == y, "{markets}: tick {t} differs");
        }
        let (sa, sb) = (&ra.summary, &rb.summary);
        assert_eq!(sa.class, sb.class, "{markets}");
        assert_eq!(sa.in_tol_from, sb.in_tol_from, "{markets}");
        assert_eq!(sa.dead, sb.dead, "{markets}");
        assert_eq!(sa.d0.to_bits(), sb.d0.to_bits(), "{markets}");
        assert_eq!(sa.max_w.to_bits(), sb.max_w.to_bits(), "{markets}");
        assert_eq!(
            sa.envelope.map(f64::to_bits),
            sb.envelope.map(f64::to_bits),
            "{markets}"
        );
        for (m, (ta, tb)) in ra.stats.trough.iter().zip(&rb.stats.trough).enumerate() {
            assert_eq!(
                (ta.0.to_bits(), ta.1),
                (tb.0.to_bits(), tb.1),
                "{markets}: trough {m}"
            );
        }
        assert_eq!(
            ra.stats.transfer.0.to_bits(),
            rb.stats.transfer.0.to_bits(),
            "{markets}"
        );
    }
}

#[test]
fn horses_r1a_expost_nests_appb_expost() {
    // R1 with the good desk's registered alternative (HORSES-SPEC §1.6): R1a with `assign:
    // ExPost` gives appb's ex-post variant, P2.0's roles through P2.0's harness, bit for bit.
    let mut old = AppbSetup::registered(52);
    old.assign = AppbAssign::ExPost;
    let mut r1a = registered("r1a");
    r1a.assign = rustyecon_engine::rustyecon_agents::Assign::ExPost;
    for (appb, _, horses, ticks) in R1A_RUNS {
        let mut a = Vec::new();
        let ra = run_appb(&old, appb, ticks, &mut |r| a.push(appb_numbers(r))).unwrap();
        let mut b = Vec::new();
        let rb = run(&r1a, horses, ticks, &mut |r| b.push(horses_numbers(r))).unwrap();
        assert_eq!(a.len(), b.len(), "{appb}: ticks run");
        for (t, (x, y)) in a.iter().zip(&b).enumerate() {
            assert!(x == y, "{appb}: tick {t} differs");
        }
        let sa = probe::harness::classify(&ra);
        assert_eq!(sa.class, rb.summary.class, "{appb}");
        assert_eq!(sa.dead, rb.summary.dead, "{appb}");
        assert_eq!(sa.d0.to_bits(), rb.summary.d0.to_bits(), "{appb}");
    }
}

#[test]
fn horses_tapes_load_and_run_deterministically() {
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
fn horses_conserve_every_tick() {
    // R2 on every tape, at rest and through a large transient (w ×2, every desk's coin ×0.1 and
    // the capacity desk's heads ×2 at genesis, where markets ration, produced goods spoil and
    // horse orders stop): every tick's ledger and the run's close within their registered
    // tolerances, and the money stock stays put. Every wear burn is a `Depreciation` burn in
    // upkeep of exactly δ times the stock its desk recorded as serving (the capacity desk's
    // holding when it decided, the maker's serving stock), and moved what it asked for, so
    // never more than was held.
    for (id, text) in TAPES {
        let mut setup = registered(id);
        let mut name = "w*2".to_string();
        for d in setup.instance.desks() {
            name.push_str(&format!("+coin.desk.{d}*0.1"));
        }
        if stocks(&setup.instance)
            .iter()
            .any(|s| s == "heads.capacity")
        {
            name.push_str("+heads.capacity*2");
        }
        Perturbation::parse(&name)
            .unwrap()
            .apply(&mut setup, 2000)
            .unwrap();
        let (kappa_delta, flow) = (
            setup.instance.per_tick(52).unwrap(),
            setup.instance.is_flow(),
        );
        let delta = kappa_delta.1;
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
            let mut worn = 0;
            for _ in 0..2000 {
                let (r, trace) = s.step_traced().expect("no breach stops the run");
                assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0, "{id}");
                assert!((money(&s) - m0).abs() <= COIN * m0, "{id}");
                if r.audit
                    .lines
                    .iter()
                    .any(|l| l.1 == Provenance::Spoilage && l.2 < 0.0)
                {
                    spoiled += 1;
                }
                for e in &trace.0 {
                    let StateDelta::Burn {
                        from: Holder::Actor(a),
                        amount,
                        prov: Provenance::Depreciation,
                        ..
                    } = &e.delta
                    else {
                        continue;
                    };
                    assert_eq!(e.phase, Phase::Upkeep, "{id}");
                    let recorded = match s.actor_state(*a) {
                        Some(ActorState::Capacity(c)) => c.held,
                        Some(ActorState::Maker(m)) => m.serving,
                        Some(ActorState::Owner(o)) => o.serving,
                        other => panic!("{id}: {other:?} wears"),
                    };
                    let rustyecon_engine::prelude::Amount::Qty(q) = amount else {
                        panic!("{id}: a wear burn of everything");
                    };
                    assert_eq!(q.to_bits(), (delta * recorded).to_bits(), "{id}");
                    assert_eq!(e.moved.to_bits(), q.to_bits(), "{id}");
                    worn += 1;
                }
            }
            assert!(spoiled > 0, "{id}: the world spoils what it does not sell");
            assert_eq!(worn > 0, !flow, "{id}: stocks wear, flows do not");
        }
    }
}

#[test]
fn horses_rest_point_is_the_oracles() {
    // HORSES-SPEC §4: the oracle's point is a rest point of the roles' map, whatever its
    // stability. Three ticks from genesis at the oracle's f64 point leave every observable
    // within 1e-12 of the oracle in log, every market trading (the horse market too) with both
    // fills 1 to rounding, at every instance and at every funded cost target (b ×1.1, ×0.9, ×2,
    // ×0.5), and at 12, 24 and 365 ticks a year for the verdict instances.
    for id in Instance::IDS {
        let base = registered(id);
        let b = base.instance.county.b;
        for f in [1.0, 1.1, 0.9, 2.0, 0.5] {
            let inst = base.instance.with_b(b * f);
            if !inst.point(52).unwrap().funded {
                continue;
            }
            let mut s = base.clone();
            s.instance = inst;
            let mut rows = Vec::new();
            let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
            assert_eq!(rec.hold_failure, None, "{id} b x{f}");
            for r in &rows {
                let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                assert!(worst <= REST, "{id} b x{f}: {worst:e} at tick {}", r.tick);
            }
        }
    }
    for id in Instance::VERDICT {
        for tpy in [12, 24, 365] {
            let s = Setup::registered(id, tpy).unwrap();
            let rec = run(&s, "hold", 3, &mut |_| {}).unwrap();
            assert_eq!(rec.hold_failure, None, "{id} at {tpy} a year");
        }
    }
}

#[test]
fn horses_oracle_is_the_flow_county_at_rho_zero() {
    // R1b (HORSES-SPEC §1.6) and decision 232: at ρ = 0 the rule-A chain's equilibrium is the
    // county's flow economy, within 1e-15 relative on x*, v, P_s, Y, N_a, the horse-day's price
    // against p_m and the horse-days against K; and the horse's hours built at J = 2 give every
    // price and quantity J = 1 gives, bit for bit.
    for id in [
        "h1", "h2", "h3", "h4", "f1", "f2", "f3", "f4", "f5", "f6", "p8",
    ] {
        let inst = registered(id).instance;
        for f in [1.0, 1.1, 0.9, 2.0, 0.5] {
            let i = inst.with_b(inst.county.b * f);
            let e = i.point(52).unwrap();
            let flow = i.flow().unwrap().point(52).unwrap();
            let rel = |a: f64, b: f64| ((a - b) / b).abs();
            for (what, a, b) in [
                ("x*", e.x_star, flow.x_star),
                ("v", e.v, flow.v),
                ("P_s", e.p_s, flow.p_s),
                ("Y", e.y, flow.y),
                ("N_a", e.n_a, flow.n_a),
                ("p_h", e.ph, flow.type_price[0]),
                ("X", e.hours, flow.type_services[0]),
            ] {
                assert!(rel(a, b) <= 1e-15, "{id} b x{f}: {what} {a:e} {b:e}");
            }
            let solve = |lag: u32| {
                let c = oracle::ChainEconomy::new(i.chain(52, lag).unwrap()).unwrap();
                match c.solve().unwrap() {
                    oracle::Regime::Interior(q) => q,
                    _ => panic!("{id}: not interior"),
                }
            };
            let (j1, j2) = (solve(1), solve(2));
            assert_eq!(j1.eq.x_star.to_bits(), j2.eq.x_star.to_bits(), "{id}");
            assert_eq!(j1.eq.v.to_bits(), j2.eq.v.to_bits(), "{id}");
            for (a, b) in j1.goods.iter().zip(&j2.goods) {
                assert_eq!(a.price.to_bits(), b.price.to_bits(), "{id} {}", a.key);
                assert_eq!(a.output.to_bits(), b.output.to_bits(), "{id} {}", a.key);
            }
        }
    }
}

#[test]
fn horses_hold_at_the_oracle_point() {
    // Mode A (PROBE-SPEC §4.6, HORSES-SPEC §7.6): from the oracle's f64 point with each actor's
    // stationary coin and each desk's rest stock, for the frame's L (H1 24,000, H2 20,000, H3
    // 30,000, H4 25,000 ticks), every observable stays within 1e-9 of the oracle in log, every
    // market trades (the horse market too) with every fill at least 1 − 1e-9, no produced good
    // spoils beyond 1e-9 of its volume, and the ledger is clean.
    for (id, l) in [
        ("h1", 24_000),
        ("h2", 20_000),
        ("h3", 30_000),
        ("h4", 25_000),
    ] {
        let rec = run(&registered(id), "hold", l, &mut |_| {}).unwrap();
        assert_eq!(rec.stop, Stop::Ran, "{id}");
        assert_eq!(rec.hold_failure, None, "{id}");
        let worst = rec.stats.peak.0 * TOL_FLOOR;
        assert!(worst <= 1e-12, "{id}: the largest gap was {worst:e}");
    }
}

#[test]
fn horses_batteries_are_registered() {
    // HORSES-SPEC §7.7: every price and the technique alone at six factors, JA, JB and N,
    // x*/2, b at four factors at genesis and dated where the target is funded, and Tier 3S:
    // 93 runs on A0 (20, 24, 25 and 24 by tier) and 91 on v1's base county, whose b ×2 is not
    // a target (§1.5).
    let counts = |id: &str| {
        let b = battery(&registered(id).instance, 52).unwrap();
        let t = |k: u8| b.iter().filter(|r| r.tier == k).count();
        (b.len(), t(1), t(2), t(3), t(4))
    };
    for id in [
        "h1", "h2", "h3", "h4", "f1", "f2", "f3", "f4", "f7", "f8", "f9", "f10",
    ] {
        assert_eq!(counts(id), (93, 20, 24, 25, 24), "{id}");
    }
    for id in ["f5", "f6"] {
        assert_eq!(counts(id), (91, 20, 24, 23, 24), "{id}");
    }
    assert_eq!(tier3s(&registered("h1").instance).len(), 24);
    // Every run of every battery and family parses, applies and writes a tape that loads.
    for id in ["h1", "f5", "p7", "r1a"] {
        let s = registered(id);
        for f in ["battery", "stocks"] {
            for name in family(&s.instance, 52, f).unwrap() {
                let mut t = s.clone();
                Perturbation::parse(&name)
                    .and_then(|p| p.apply(&mut t, 20_000))
                    .unwrap_or_else(|e| panic!("{id} {name}: {e}"));
                sim(&tape_ron(&t).unwrap_or_else(|e| panic!("{id} {name}: {e}")));
            }
        }
    }
}

#[test]
fn horses_shocks_and_stocks_apply_as_named() {
    // The grammar of HORSES-SPEC §7.7: a dated shock of b lands at L/4 through a SetParam of
    // fodder's land and of the build's pasture, and restarts the clock; a shock at genesis
    // changes the tape's coefficients and not genesis; a stock displacement moves the holding
    // it names, and the maker's own stock moves its record with it.
    let s = registered("h1");
    let mut t = s.clone();
    let p = Perturbation::parse("b*2@dated").unwrap();
    p.apply(&mut t, 4000).unwrap();
    assert_eq!(p.clock_start(4000), 1000);
    assert_eq!(t.b_at(999), 0.4);
    assert_eq!(t.b_at(1000), 0.8);
    let mut sm = sim(&tape_ron(&t).unwrap());
    let w = sm.world();
    let fodder = w
        .id_of::<rustyecon_engine::prelude::ParamId>("inst.fodder.land")
        .unwrap();
    let pasture = w
        .id_of::<rustyecon_engine::prelude::ParamId>("inst.horse.land")
        .unwrap();
    let before = sm.param(pasture).unwrap();
    for _ in 0..1000 {
        sm.step().unwrap();
    }
    assert_eq!(sm.param(fodder), Some(0.2));
    sm.step().unwrap();
    assert_eq!(sm.param(fodder), Some(0.4));
    assert_eq!(sm.param(pasture), Some(2.0 * before));
    let mut g = s.clone();
    Perturbation::parse("b*0.5@genesis")
        .unwrap()
        .apply(&mut g, 4000)
        .unwrap();
    let a = probe::horses::setup::genesis(&s).unwrap();
    let b = probe::horses::setup::genesis(&g).unwrap();
    assert_eq!(a, b);
    assert!(tape_ron(&g)
        .unwrap()
        .contains("(key: \"inst.fodder.land\", value: 0.1,"));
    // The maker's own stock ×2: its holding and its record move together.
    let mut o = s.clone();
    Perturbation::parse("own.maker*2")
        .unwrap()
        .apply(&mut o, 4000)
        .unwrap();
    let text = tape_ron(&o).unwrap();
    let own = a.stock_of(&s.instance, "own.maker").unwrap();
    let fin = a.stock_of(&s.instance, "finished.maker").unwrap();
    assert!(text.contains(&format!("own: {:?},", 2.0 * own)));
    assert!(text.contains(&format!("(\"horse\", {:?})", 2.0 * own + fin)));
    // JB moves the machine side against labour and the good.
    let mut j = s.clone();
    Perturbation::parse("JB(2)")
        .unwrap()
        .apply(&mut j, 4000)
        .unwrap();
    // Markets: labour, land, fodder, horse, traction, good.
    assert_eq!(j.displace.price, vec![2.0, 1.0, 0.5, 0.5, 0.5, 2.0]);
    assert_eq!(s.instance.markets()[4], HOURS);
}

#[test]
fn horses_kick_set_decays_at_a_stable_point() {
    // HORSES-SPEC §7.5: the kick set at the end of H1's mode-A run, with a short horizon, runs
    // one base continuation and ± each of its six markets, every kick is realised, and the
    // envelope's slowest mode is about 1 − δ a year (§6.2's mirror has 0.910 at H1; the
    // engine's kick envelope reads 0.891).
    let (k, env) = kick_set(&registered("h1"), "hold", 200, 12_000).unwrap();
    assert_eq!(k.kicks.len(), 2 * 6);
    assert!(k.kicks.iter().all(|x| x.error.is_none() && x.size > 0.0));
    let g = slowest_mode(&env).expect("a fit window");
    let a_year = rustyecon_core::num::pow(g, 52.0);
    assert!(
        (0.85..0.95).contains(&a_year),
        "the slowest mode is {a_year} a year"
    );
    // Its 3·T6 is at most the frame's L at H1 (24,000 ticks, from the mirror's slower PL), so
    // the mirror's L stands (§7.4: the engine's g registers only a longer L).
    assert!((15_000..=24_000).contains(&three_t6(g)), "{}", three_t6(g));
    // δ a tick is the clock's.
    let (_, d) = registered("h1").instance.per_tick(52).unwrap();
    let c = probe::setup::clock(52).unwrap();
    assert_eq!(d, c.fraction(FractionPerYear(0.1)));
}

/// The registered setup of an instance with the maker's reservation at `psi` (L0.4).
fn reserved(id: &str, psi: f64) -> Setup {
    let mut s = registered(id);
    s.reserve = Some(psi);
    s
}

/// Every number of a row that the reservation could move: [`horses_numbers`], the stock
/// records, the cleared volumes and the outputs.
fn reserve_numbers(r: &Row) -> Vec<u64> {
    let mut v = horses_numbers(r);
    let s = &r.stocks;
    for x in [
        s.tasks, s.maker, s.own, s.finished, s.target, s.order, s.run, s.running, s.full,
    ] {
        v.push(x.to_bits());
    }
    v.extend(r.cleared.iter().map(|x| x.to_bits()));
    v.extend(r.output.iter().map(|x| x.to_bits()));
    v
}

/// A run's rows as [`reserve_numbers`], its record, and whether the maker withheld each tick.
fn numbers_of(setup: &Setup, name: &str, ticks: u64) -> (Vec<Vec<u64>>, Record, Vec<bool>) {
    let (mut rows, mut withheld) = (Vec::new(), Vec::new());
    let rec = run(setup, name, ticks, &mut |r| {
        rows.push(reserve_numbers(r));
        withheld.push(r.withheld);
    })
    .unwrap();
    (rows, rec, withheld)
}

#[test]
fn reserve_holds_the_idle_horse_price() {
    // IDLE-SPEC §6, test 2: at H2 with the capacity desk's heads ×10, P2.2a's glut, the maker's
    // reservation at ψ 0.25 keeps the horse price within [0.1, 1] of genesis over 400 ticks. On
    // every tick with neither an offer nor a bid the price is held bit for bit, and on every tick
    // the harness reads as withheld no head is offered. Without the flag the run leaves the
    // runaway bound at tick 138, as P2.2a's did.
    let on = reserved("h2", 0.25);
    let horse = on
        .instance
        .markets()
        .iter()
        .position(|m| m == "horse")
        .unwrap();
    let mut rows: Vec<Row> = Vec::new();
    let rec = run(&on, "heads.capacity*10", 400, &mut |r| rows.push(r.clone())).unwrap();
    assert_eq!(rec.stop, Stop::Ran);
    let g = rows[0].price[horse];
    for r in &rows {
        let rel = r.price[horse] / g;
        assert!((0.1..=1.0).contains(&rel), "tick {}: {rel}", r.tick);
        if r.withheld {
            assert_eq!(r.supply[horse], 0.0, "tick {}", r.tick);
        }
    }
    let mut quiet = 0;
    for p in rows.windows(2) {
        if p[0].supply[horse] == 0.0 && p[0].demand[horse] == 0.0 {
            quiet += 1;
            assert_eq!(p[1].price[horse].to_bits(), p[0].price[horse].to_bits());
        }
    }
    assert!(quiet > 0, "the market idles");
    assert!(rows.iter().any(|r| r.withheld), "the maker withholds");
    let off = run(&registered("h2"), "heads.capacity*10", 400, &mut |_| {}).unwrap();
    match off.stop {
        Stop::Runaway(why) => assert!(
            why.contains("price of horse") && why.contains("at tick 138"),
            "{why}"
        ),
        other => panic!("P2.2a's glut runs away: {other:?}"),
    }
}

#[test]
fn reserve_absent_or_zero_is_p22a() {
    // IDLE-SPEC §6, test 3 (R1): at H1-H4, and at H2 with the heads ×10, a tape with the
    // reservation at ψ 0 has every price, observable, target, volume, fill, coin, stock record
    // and output equal, bit for bit, to the tape without it, for 2,000 ticks or until P2.2a's
    // run stops. The tapes differ by the param and the field.
    for (id, name) in [
        ("h1", "hold"),
        ("h2", "hold"),
        ("h3", "hold"),
        ("h4", "hold"),
        ("h2", "heads.capacity*10"),
    ] {
        let zero = reserved(id, 0.0);
        assert_ne!(tape_ron(&zero).unwrap(), tape_ron(&registered(id)).unwrap());
        let (a, ra, _) = numbers_of(&registered(id), name, 2000);
        let (b, rb, wb) = numbers_of(&zero, name, 2000);
        assert_eq!(a, b, "{id} {name}");
        assert_eq!(ra.stop, rb.stop, "{id} {name}");
        assert!(wb.iter().all(|w| !w), "{id} {name}: ψ 0 never withholds");
        // Both are P2.2a's: at rest they run, and the glut leaves the runaway bound at tick 138,
        // its maker's markup down to 1e-6, so a reservation that acted at ψ 0 would show.
        match (name, &rb.stop) {
            ("hold", Stop::Ran) => {}
            ("heads.capacity*10", Stop::Runaway(why)) if why.contains("at tick 138") => {
                assert!(rb.stats.stock.markup_low < 1e-5, "{id} {name}");
            }
            (_, other) => panic!("{id} {name}: {other:?}"),
        }
    }
}

#[test]
fn reserve_leaves_the_rest_point() {
    // IDLE-SPEC §6, test 4, and §7: mode A at H1-H4 with ψ 0.25 is mode A without it, bit for
    // bit, for 2,000 ticks, and never withholds: at rest the maker's markup is 1. The comparison
    // can see a reservation: at ψ 1.01, above the rest's markup, the maker withholds from tick 0
    // and the run parts from mode A.
    for id in ["h1", "h2", "h3", "h4"] {
        let (a, _, _) = numbers_of(&registered(id), "hold", 2000);
        let (b, _, wb) = numbers_of(&reserved(id, 0.25), "hold", 2000);
        assert_eq!(a, b, "{id}");
        assert!(wb.iter().all(|w| !w), "{id}");
        let (c, _, wc) = numbers_of(&reserved(id, 1.01), "hold", 2000);
        assert!(wc[0], "{id}: at ψ 1.01 it withholds at genesis");
        assert_ne!(a, c, "{id}");
    }
}

#[test]
fn reserve_conserves_every_tick() {
    // R2 with the reservation acting: H2's glut (the capacity desk's heads ×10) at ψ 0.25 for
    // 2,000 ticks. Every tick's ledger and the run's close are within their tolerances, the money
    // stock stays put, every wear burn is δ times the recorded stock, and on each tick the maker
    // offers nothing while it holds finished heads, it keeps them: its heads move only by what it
    // made and what wore.
    let setup = {
        let mut s = reserved("h2", 0.25);
        Perturbation::parse("heads.capacity*10")
            .unwrap()
            .apply(&mut s, 2000)
            .unwrap();
        s
    };
    let delta = setup.instance.per_tick(52).unwrap().1;
    let mut s = sim(&tape_ron(&setup).unwrap());
    let coin = s.world().id_of::<GoodId>("coin").unwrap();
    let horse = s.world().id_of::<GoodId>("horse").unwrap();
    let maker = s.world().id_of::<ActorId>("desk.maker").unwrap();
    let money = |s: &Sim| {
        s.world()
            .actors
            .iter()
            .map(|a| s.holding(Holder::Actor(a.id)).map_or(0.0, |i| i.get(coin)))
            .fold(0.0, |x, y| x + y)
    };
    let held = |s: &Sim| {
        s.holding(Holder::Actor(maker))
            .map_or(0.0, |i| i.get(horse))
    };
    let m0 = money(&s);
    let mut withheld = 0;
    for _ in 0..2000 {
        let before = held(&s);
        let finished = match s.actor_state(maker) {
            Some(ActorState::Maker(m)) => before - m.own,
            other => panic!("{other:?}"),
        };
        let (r, trace) = s.step_traced().expect("no breach stops the run");
        assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0);
        assert!((money(&s) - m0).abs() <= COIN * m0);
        let line = r.markets.iter().find(|l| l.good == horse).unwrap();
        let mut worn = 0.0;
        for e in &trace.0 {
            let StateDelta::Burn {
                from: Holder::Actor(a),
                amount,
                prov: Provenance::Depreciation,
                ..
            } = &e.delta
            else {
                continue;
            };
            let recorded = match s.actor_state(*a) {
                Some(ActorState::Capacity(c)) => c.held,
                Some(ActorState::Maker(m)) => m.serving,
                other => panic!("{other:?} wears"),
            };
            let rustyecon_engine::prelude::Amount::Qty(q) = amount else {
                panic!("a wear burn of everything");
            };
            assert_eq!(q.to_bits(), (delta * recorded).to_bits());
            if *a == maker {
                worn += q;
            }
        }
        if line.supply == 0.0 && finished > 0.0 {
            withheld += 1;
            let made = match s.actor_state(maker) {
                Some(ActorState::Maker(m)) => m.output,
                other => panic!("{other:?}"),
            };
            let after = held(&s);
            assert!(
                ((before + made - worn) - after).abs() <= 1e-12 * before,
                "{before} {made} {worn} {after}"
            );
        }
    }
    assert!(withheld > 100, "the maker withheld on {withheld} ticks");
}

#[test]
fn horses_tapes_keep_their_world_ids() {
    // R1 for L0.4's optional field: `world_id` hashes the resolved actors, so a field added to a
    // kind moves it unless it is left out when absent. Each committed horses tape keeps the
    // world_id P2.2 recorded (D:/rustyecon-p2g/report/hashes/wsl-horses-<id>.txt). L0.4 as first
    // committed moved all six; L0.5 restored them. With the reservation the world is another.
    //
    // R1 for P2.2b.1's optional fields (a good's `untraded` flag, a desk's `plant`): the seven
    // markets tapes too, and each of the thirteen keeps its canonical text's `tape_hash` as well,
    // the pair P2.2 recorded (D:/rustyecon-p2g/report/hashes/wsl-<tape>.txt). A planted desk's
    // world is another: LB1 at θ = 1 runs as LN1 bit for bit (`loops.rs`), but its resolved
    // specs carry their plants.
    let p7 = include_str!("../../../tapes/horses-p7.ron");
    for (id, text, want, hash) in [
        (
            "h1",
            TAPES[0].1,
            0xc92f_78b1_7dd4_9bb6_u64,
            0x4976_7952_83cc_52ed_u64,
        ),
        (
            "h2",
            TAPES[1].1,
            0x386a_e50b_a941_4be3,
            0x7104_4f8a_b931_81a8,
        ),
        (
            "h3",
            TAPES[2].1,
            0x1562_9782_d6e4_b359,
            0x7892_9ea1_9c81_fb81,
        ),
        (
            "h4",
            TAPES[3].1,
            0x5059_ba9b_2ff1_b015,
            0x72ee_0740_db8e_0fa2,
        ),
        (
            "r1a",
            TAPES[4].1,
            0xddc8_6dbc_11b7_24f3,
            0xf31c_56f9_d322_26d1,
        ),
        ("p7", p7, 0xc9e4_2f5f_b078_f10f, 0x70ca_bdbe_0e0f_a986),
        (
            "markets-g1",
            include_str!("../../../tapes/markets-g1.ron"),
            0x4b0b_1301_cb7a_a351,
            0xff3c_23ea_26a6_464e,
        ),
        (
            "markets-i0",
            include_str!("../../../tapes/markets-i0.ron"),
            0x5068_f2ff_9dbb_fa00,
            0x9338_5d64_d42f_858e,
        ),
        (
            "markets-i1",
            include_str!("../../../tapes/markets-i1.ron"),
            0x965e_dbef_2664_b850,
            0x4dbe_0ba9_88f9_4b02,
        ),
        (
            "markets-i2",
            include_str!("../../../tapes/markets-i2.ron"),
            0x6c3e_2217_b9ee_fef9,
            0x8917_3d95_2d0b_5e65,
        ),
        (
            "markets-i3",
            include_str!("../../../tapes/markets-i3.ron"),
            0xb631_f8a0_f7d9_ab95,
            0xdea2_2786_21a5_2649,
        ),
        (
            "markets-l2",
            include_str!("../../../tapes/markets-l2.ron"),
            0xb47c_5301_5f7f_1637,
            0x8aa5_0fca_e92a_6260,
        ),
        (
            "markets-l3",
            include_str!("../../../tapes/markets-l3.ron"),
            0xd525_2228_af8a_95a6,
            0xde5e_9b54_8d5d_e83e,
        ),
    ] {
        assert_eq!(sim(text).world().world_id, want, "{id}");
        let tape = Tape::from_ron(text).expect("the tape parses");
        assert_eq!(certify::tape_hash(&tape), hash, "{id}");
    }
    let with = sim(&tape_ron(&reserved("h1", 0.25)).unwrap());
    assert_ne!(with.world().world_id, 0xc92f_78b1_7dd4_9bb6);
    let planted = probe::horses::loops::Setup::registered("lb1", 52)
        .unwrap()
        .with_theta(1.0);
    let plain = probe::horses::loops::Setup::registered("ln1", 52).unwrap();
    let id = |s: &probe::horses::loops::Setup| {
        sim(&probe::horses::loops::tape_ron(s).unwrap())
            .world()
            .world_id
    };
    assert_ne!(id(&planted), id(&plain));
}

#[test]
fn harness_reads_the_reservation_the_maker_acts_on() {
    // The engine review of L0.4 (2026-09-29): the harness once formed the maker's markup from
    // rule A's running recipe written into its code (fodder a·1 + 0, labour a·0 + λ), not from
    // the tape's params, so a frame with another running recipe would have read wrong markups
    // and withheld ticks, and nothing would have flagged it. Now it reads the tape's own recipes
    // through the rule's own code (`maker_readout`). H1 with the horses' running labour raised
    // from 0 to X, so that a unit costs twice what it does at rest and the markup at genesis
    // prices is about 0.5: at ψ 0.75 the engine's maker offers no head at tick 0 and the harness
    // reads it as withheld; at ψ 0.25 it offers and the harness reads it as offering. Both read
    // the markup a hand computation from the tape's numbers gives, with X in it. With the old
    // readout the markup read 1 and the first case read as offering.
    let base = registered("h1");
    let rule = base.instance.rule_a(52).unwrap();
    let (kappa, delta) = base.instance.per_tick(52).unwrap();
    let g = probe::horses::setup::genesis(&base).unwrap();
    let markets = base.instance.markets();
    let at = |m: &str| markets.iter().position(|x| x == m).unwrap();
    let (w, r, pf, pk) = (
        g.prices[at("labour")],
        g.prices[at("land")],
        g.prices[at("fodder")],
        g.prices[at("horse")],
    );
    let a = rule.own_hours;
    let c0 = (a * 1.0 + 0.0) * pf + (a * 0.0 + rule.labour) * w + rule.pasture * r;
    let x = c0 / (a * w);
    let hand = pk * (1.0 - delta * a / kappa)
        / ((a * 1.0 + 0.0) * pf + (a * x + rule.labour) * w + rule.pasture * r);
    assert!((hand - 0.5).abs() < 1e-9, "{hand}");
    let line = "(key: \"inst.horse.run.labour\", value: 0.0,";
    let mut read = Vec::new();
    for (psi, withholds) in [(0.75, true), (0.25, false)] {
        let text = tape_ron(&reserved("h1", psi)).unwrap();
        assert!(text.matches(line).count() == 1);
        let text = text.replace(
            line,
            &format!("(key: \"inst.horse.run.labour\", value: {x:?},"),
        );
        let mut s = sim(&text);
        let maker = probe::horses::harness::the_maker(&s).expect("a maker");
        let horse = s.world().id_of::<GoodId>("horse").unwrap();
        let report = s.step().unwrap();
        let price_of = |g: GoodId| report.markets.iter().find(|l| l.good == g).map(|l| l.price);
        let res = probe::horses::harness::maker_readout(&s, &maker, &price_of).unwrap();
        let offered = report
            .markets
            .iter()
            .find(|l| l.good == horse)
            .unwrap()
            .supply;
        assert_eq!(res.psi, psi);
        assert!(
            (res.markup / hand - 1.0).abs() < 1e-12,
            "{} {hand}",
            res.markup
        );
        assert_eq!(res.withholds, withholds, "ψ {psi}: markup {}", res.markup);
        assert_eq!(
            offered == 0.0,
            withholds,
            "ψ {psi}: the engine offered {offered}"
        );
        read.push(res.markup.to_bits());
    }
    assert_eq!(read[0], read[1]);
}
