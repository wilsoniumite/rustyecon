//! The demo world's lenses (docs/demo/WORLD.md §6; D.3, 2026-09-27): the one definition of
//! each measure the GUI's map and table show (`rustyecon_worldgen::lens`).
//!
//! - `every_lens_is_a_measure_lens_rs_defines`: the bundled `lenses.csv` reads, each key names
//!   a measure, each change lens follows its level, and each diverging scale centres on its
//!   reference; a table that breaks these is refused.
//! - `lens_keys_are_the_compiled_tapes`: every key a county's readings come from is in the
//!   committed tape, as the compiler writes it.
//! - `the_measures_follow_their_formulas`: each measure on hand-made readings, and why a
//!   measure has no value when an input is missing, a ratio is over zero, or the oracle waits
//!   for crates/observe.

use rustyecon_engine::prelude::{ActorId, GoodId, NodeId, ParamId, Sim, Tape};
use rustyecon_worldgen::lens::{
    centre, demo_gb, demo_gb_v2, for_tape, maker_markup, reference_value, value, ChainReadings,
    CountyKeys, Kind, Level, MarkupInputs, Measure, NoValue, Readings, WindowTick, MARKETS,
    PARAMS_READ, V1_TAPE, V2_TAPE,
};
use rustyecon_worldgen::tables::{parse_lenses, LensSource, Param, Scale};
use std::sync::OnceLock;

const TAPE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes/demo-gb.ron");
const LENSES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../worlds/demo-gb/lenses.csv"
);

fn tape() -> &'static Tape {
    static T: OnceLock<Tape> = OnceLock::new();
    T.get_or_init(|| {
        let text = std::fs::read_to_string(TAPE).unwrap_or_else(|e| panic!("{TAPE}: {e}"));
        Tape::from_ron(&text).unwrap_or_else(|e| panic!("{e}"))
    })
}

#[test]
fn every_lens_is_a_measure_lens_rs_defines() {
    let text = std::fs::read_to_string(LENSES).unwrap();
    assert_eq!(
        text,
        rustyecon_worldgen::lens::DEMO_GB_LENSES,
        "the bundle is the table"
    );
    let lenses = demo_gb().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(lenses.len(), 25);
    for (i, l) in lenses.iter().enumerate() {
        let m = Measure::of(&l.key).unwrap_or_else(|| panic!("{} names no measure", l.key));
        if let Measure::Since(level) = m {
            let base = l.key.trim_start_matches("since.");
            assert!(lenses[..i].iter().any(|b| b.key == base), "{}", l.key);
            assert_eq!(Measure::of(base), Some(Measure::Level(level)));
        }
        if l.scale == Scale::Diverging {
            let c = centre(l).expect("a diverging lens centres on its reference");
            assert!(l.domain.0 < c && c < l.domain.1, "{}", l.key);
        } else {
            assert_eq!(centre(l), None);
        }
        assert!(l.reference.is_empty() || reference_value(&l.reference).is_some());
    }
    assert_eq!(
        reference_value("schedule ceiling gamma(1)/J(1) = 1.667"),
        Some(1.667)
    );
    assert_eq!(reference_value("0 = the oracle"), Some(0.0));
    assert_eq!(reference_value("the oracle"), None);
    // A key no measure has, a change lens before its level, and a diverging scale with no
    // reference inside its domain are each refused.
    let head = text.lines().next().unwrap();
    let row = |key: &str, scale: &str, reference: &str, lo: &str, hi: &str| {
        format!("{head}\n{key},A lens,x,units,{scale},{reference},{lo},{hi},inputs,report,\n")
    };
    for (bad, why) in [
        (
            row("wage.imagined", "sequential", "", "0", "1"),
            "no measure",
        ),
        (
            row("since.wage.land", "diverging", "0 = genesis", "-1", "1"),
            "comes after its level",
        ),
        (
            row("wage.land", "diverging", "", "-1", "1"),
            "centres on a reference",
        ),
        (
            row("wage.land", "diverging", "2 = far", "-1", "1"),
            "centres on a reference",
        ),
        (
            row("wage.land", "sequential", "the top", "0", "1"),
            "names no value",
        ),
    ] {
        let e = parse_lenses(&bad).expect_err(why);
        assert!(e.why.contains(why), "{bad}: {e}");
    }
}

#[test]
fn lens_keys_are_the_compiled_tapes() {
    let sim = Sim::new(tape()).unwrap_or_else(|e| panic!("{e}"));
    let w = sim.world();
    assert_eq!(w.nodes.len(), 93);
    for n in &w.nodes {
        let k = CountyKeys::of(n.key.as_str());
        let node = w.id_of::<NodeId>(&k.node).expect("the node");
        for good in MARKETS {
            let g = w.id_of::<GoodId>(good).expect("the good");
            assert!(
                w.markets().any(|(mn, mg)| mn == node && mg == g),
                "{}/{good}",
                k.node
            );
        }
        for p in Param::ALL {
            assert!(w.id_of::<ParamId>(k.param(p)).is_some(), "{}", k.param(p));
        }
        for (actor, kind) in [
            (&k.desk_good, "GoodDesk"),
            (&k.desk_mach, "MachDesk"),
            (&k.provider, "Provider"),
            (&k.workers, "Workers"),
        ] {
            let a = w.id_of::<ActorId>(actor).expect("the actor");
            let state = sim.actor_state(a).expect("a state");
            assert!(format!("{state:?}").starts_with(kind), "{actor}: {state:?}");
        }
    }
    assert!(PARAMS_READ.contains(&Param::Workers) && PARAMS_READ.contains(&Param::Space));
}

fn at_rest() -> Readings {
    // w, r, p_m, p; cleared labour, land, machine services, the good.
    Readings {
        prices: [Some(2.0), Some(1.0), Some(1.2), Some(1.5)],
        cleared: [Some(3.0), Some(6.0), Some(4.0), Some(8.0)],
        params: {
            let mut p = [None; 11];
            p[Param::Workers.index()] = Some(208.0);
            p[Param::Space.index()] = Some(0.5);
            p[Param::Eta.index()] = Some(1.8);
            p
        },
        n_tick: Some(4.0),
        share: Some(0.25),
        due: Some(4.0),
        paid: Some(3.0),
        rationing: vec![(2.0, 2.0), (4.0, 3.0), (0.0, 0.0)],
        traded: vec![true, false, true, true, false],
        chain: None,
    }
}

#[test]
fn the_measures_follow_their_formulas() {
    let x = at_rest();
    let lv = |l: Level| value(Measure::Level(l), &x, None).unwrap();
    assert_eq!(lv(Level::WageBaskets), 2.0 / (1.5 + 0.5 * 1.0));
    assert_eq!(lv(Level::WageGoods), 2.0 / 1.5);
    assert_eq!(lv(Level::WageLand), 2.0);
    assert_eq!(lv(Level::RentGoods), 1.0 / 1.5);
    assert_eq!(lv(Level::ShareLand), 6.0 / (6.0 + 6.0));
    assert_eq!(lv(Level::ShareLabour), 6.0 / 12.0);
    assert_eq!(lv(Level::Frontier), 0.75);
    assert_eq!(lv(Level::Participation), 3.0 / 4.0);
    assert_eq!(lv(Level::OutputPerHead), 8.0 / 4.0);
    assert_eq!(lv(Level::PriceGood), 1.5);
    assert_eq!(lv(Level::PriceMach), 1.2);
    assert_eq!(lv(Level::ReliefBurden), 4.0 / 6.0);
    assert_eq!(lv(Level::Shortfall), 0.25);
    assert_eq!(
        lv(Level::Rationing),
        0.25,
        "the largest 1 − filled/requested"
    );
    assert_eq!(
        lv(Level::NoTrade),
        2.0,
        "the ticks in the window a market did not trade"
    );
    assert_eq!(lv(Level::Param(Param::Eta)), 1.8);
    assert_eq!(
        lv(Level::Param(Param::Workers)),
        208.0,
        "as registered, a year"
    );
    // A change since the first tick is a log ratio, 0 at the first tick itself.
    let mut later = x.clone();
    later.prices[0] = Some(4.0);
    let since = value(Measure::Since(Level::WageLand), &later, Some(&x)).unwrap();
    assert_eq!(since, rustyecon_core::num::ln(2.0));
    assert_eq!(
        value(Measure::Since(Level::WageLand), &x, Some(&x)),
        Ok(0.0)
    );
    assert_eq!(
        value(Measure::Since(Level::WageLand), &x, None),
        Err(NoValue::Missing("the record's first tick"))
    );
    // Why there is no value.
    let mut gap = x.clone();
    gap.prices[1] = None;
    assert_eq!(
        value(Measure::Level(Level::WageLand), &gap, None),
        Err(NoValue::Missing("the price of land"))
    );
    let mut zero = x.clone();
    zero.due = Some(0.0);
    zero.paid = Some(0.0);
    assert!(matches!(
        value(Measure::Level(Level::Shortfall), &zero, None),
        Err(NoValue::Undefined(_))
    ));
    let none = Readings::default();
    assert_eq!(
        value(Measure::Level(Level::Rationing), &none, None),
        Err(NoValue::Missing("a rationing line with a request"))
    );
    assert_eq!(
        value(Measure::Level(Level::NoTrade), &none, None),
        Err(NoValue::Missing("whether the markets traded"))
    );
    for l in [Level::GapOracle, Level::GapWage] {
        assert!(l.needs_observe());
        assert!(matches!(
            value(Measure::Level(l), &x, None),
            Err(NoValue::Unavailable(_))
        ));
    }
}

const TAPE_V2: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes/demo-gb-v2.ron");
const LENSES_V2: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../worlds/demo-gb/lenses-v2a1.csv"
);

fn tape_v2() -> &'static Tape {
    static T: OnceLock<Tape> = OnceLock::new();
    T.get_or_init(|| {
        let text = std::fs::read_to_string(TAPE_V2).unwrap_or_else(|e| panic!("{TAPE_V2}: {e}"));
        Tape::from_ron(&text).unwrap_or_else(|e| panic!("{e}"))
    })
}

/// The chain's eleven lenses (WORLD-V2 §7.2), in the table's order.
const CHAIN: [&str; 11] = [
    "horses.per.head",
    "since.horses.per.head",
    "horses.vs.oracle",
    "horses.vs.plan",
    "price.fodder",
    "price.horse",
    "hday.markup",
    "land.to.fodder",
    "land.to.horses",
    "reserve.ticks",
    "idle.horse",
];

#[test]
fn every_v2_lens_is_a_measure_lens_rs_defines() {
    // WORLD-V2 §7 (decision 332): the second pass's table is v1's 25 lenses, six of them
    // changed, and the chain's 11, 36 in all, each a measure lens.rs defines; v1's table is
    // unchanged; the GUI takes a tape's table by the tape's name.
    let text = std::fs::read_to_string(LENSES_V2).unwrap();
    assert_eq!(text, rustyecon_worldgen::lens::DEMO_GB_V2_LENSES);
    let v2 = demo_gb_v2().unwrap_or_else(|e| panic!("{e}"));
    let v1 = demo_gb().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(v2.len(), 36);
    let keys: Vec<&str> = v2.iter().map(|l| l.key.as_str()).collect();
    assert_eq!(
        &keys[..11],
        &CHAIN,
        "the chain's lenses come first, as a group"
    );
    // v1's 25 but price.mach, which the horse-day's price replaces.
    for l in &v1 {
        let want = if l.key == "price.mach" {
            "price.hday"
        } else {
            l.key.as_str()
        };
        assert!(keys.contains(&want), "{want}");
    }
    // Nineteen of v1's rows are kept whole; six change.
    let changed = [
        "price.hday",
        "no.trade",
        "param.b",
        "param.lam",
        "gap.oracle",
        "gap.wage",
    ];
    let mut kept = 0;
    for l in &v1 {
        if let Some(x) = v2.iter().find(|x| x.key == l.key) {
            if !changed.contains(&l.key.as_str()) {
                assert_eq!(x, l, "{} is v1's row", l.key);
                kept += 1;
            }
        }
    }
    assert_eq!(kept, 19);
    for l in &v2 {
        let m = Measure::of(&l.key).unwrap_or_else(|| panic!("{} names no measure", l.key));
        if l.scale == Scale::Diverging {
            let c = centre(l).expect("a diverging lens centres on its reference");
            assert!(l.domain.0 < c && c < l.domain.1, "{}", l.key);
        }
        // The three oracle lenses are oracle_gap's, and nothing else is.
        assert_eq!(
            m.level().needs_observe(),
            l.source == LensSource::Oracle,
            "{}",
            l.key
        );
    }
    let horse = v2
        .iter()
        .find(|l| l.key == "price.horse")
        .expect("price.horse");
    assert_eq!((horse.domain, centre(horse)), ((0.0, 2.0), Some(1.0)));
    // By the tape's name.
    assert_eq!(for_tape(V2_TAPE).unwrap(), v2);
    assert_eq!(for_tape(V1_TAPE).unwrap(), v1);
    assert_eq!(for_tape("gate").unwrap(), v1);
    assert_eq!(Kind::of_tape(&tape_v2().header.name), Kind::V2a1);
}

#[test]
fn v2_lens_keys_are_the_compiled_tapes() {
    // Every key a county's readings come from on the second pass is on its tape, as the stage's
    // compiler writes it; v1's a, λ and b are not (decision 328).
    let sim = Sim::new(tape_v2()).unwrap_or_else(|e| panic!("{e}"));
    let w = sim.world();
    assert_eq!(w.nodes.len(), 93);
    for n in &w.nodes {
        let k = CountyKeys::of_kind(n.key.as_str(), Kind::V2a1);
        let s = k.stage.as_ref().expect("the chain's keys");
        let node = w.id_of::<NodeId>(&k.node).expect("the node");
        assert_eq!(k.markets().len(), 6);
        assert_eq!(k.must_trade().len(), 5);
        assert!(!k.must_trade().contains(&"horse"));
        for good in k.markets() {
            let g = w.id_of::<GoodId>(good).expect("the good");
            assert!(
                w.markets().any(|(mn, mg)| mn == node && mg == g),
                "{}/{good}",
                k.node
            );
        }
        for p in Param::ALL {
            let on = w.id_of::<ParamId>(k.param(p)).is_some();
            assert_eq!(
                on,
                !matches!(p, Param::A | Param::Lam | Param::B),
                "{}",
                k.param(p)
            );
        }
        let shared = [&s.kappa, &s.delta, &s.run_fodder, &s.run_labour, &s.psi];
        for key in s.coef.iter().chain(shared) {
            assert!(w.id_of::<ParamId>(key).is_some(), "{key}");
        }
        for (actor, kind) in [
            (&k.desk_good, "GoodDesk"),
            (&s.desk_capacity, "Capacity"),
            (&s.desk_maker, "Maker"),
            (&s.desk_fodder, "MachDesk"),
            (&k.provider, "Provider"),
            (&k.workers, "Workers"),
        ] {
            let a = w.id_of::<ActorId>(actor).expect("the actor");
            let state = sim.actor_state(a).expect("a state");
            assert!(format!("{state:?}").starts_with(kind), "{actor}: {state:?}");
        }
    }
}

/// Hand-made second-pass readings: w 2, r 1, p_h 1.2, p 1.5; p_f 0.5, p_K 300 (a markup near 1).
fn chain_at_rest() -> Readings {
    let mut x = at_rest();
    x.params[Param::Lam.index()] = None;
    x.params[Param::B.index()] = None;
    x.chain = Some(ChainReadings {
        price_fodder: Some(0.5),
        price_horse: Some(300.0),
        cleared_fodder: Some(3.0),
        cleared_horse: Some(0.01),
        horse_traded: Some(true),
        coef: [Some(0.68), Some(187.2), Some(18.72), Some(74.9)],
        kappa: Some(1.0),
        delta: Some(0.0016),
        run_fodder: Some(1.0),
        run_labour: Some(0.0),
        psi: Some(0.25),
        used: Some(0.3),
        out_good: Some(8.0),
        out_hours: Some(4.0),
        out_horse: Some(0.02),
        out_fodder: Some(3.0),
        held: Some(4.0),
        target: Some(5.0),
        serving: Some(0.5),
        clock: None,
        window: Vec::new(),
    });
    x
}

#[test]
fn the_chain_measures_follow_their_formulas() {
    // WORLD-V2 §7.2: each of the chain's measures on hand-made readings, the two markups in
    // their rules' order of summation, rule A's folds for param.b and param.lam, the trailing
    // window's counts, and the horse's price with no value when no horse traded.
    let x = chain_at_rest();
    let lv = |l: Level| value(Measure::Level(l), &x, None).unwrap();
    assert_eq!(lv(Level::HorsesPerHead), (4.0 + 0.5) / 4.0);
    assert_eq!(lv(Level::HorsesVsPlan), rustyecon_core::num::ln(4.0 / 5.0));
    assert_eq!(lv(Level::PriceFodder), 0.5);
    assert_eq!(lv(Level::PriceHday), 1.2);
    let inputs = MarkupInputs {
        w: 2.0,
        r: 1.0,
        pf: 0.5,
        pk: 300.0,
        own_hours: 187.2,
        kappa: 1.0,
        delta: 0.0016,
        run_fodder: 1.0,
        run_labour: 0.0,
        labour: 18.72,
        land: 74.9,
    };
    let c_m = (187.2 * 1.0 + 0.0) * 0.5 + (187.2 * 0.0 + 18.72) * 2.0 + 74.9 * 1.0;
    let markup = 300.0 * (1.0 - 0.0016 * 187.2 / 1.0) / c_m;
    assert!(markup > 0.9 && markup < 1.1);
    assert_eq!(maker_markup(&inputs), markup);
    assert_eq!(lv(Level::PriceHorse), markup);
    let full = (0.0 + 1.0 * 0.5) + 0.0 * 2.0 + 0.0016 * 300.0 / 1.0;
    assert_eq!(lv(Level::HdayMarkup), 1.2 / full - 1.0);
    assert_eq!(lv(Level::LandToFodder), 0.68 * 3.0 / 6.0);
    assert_eq!(lv(Level::LandToHorses), (0.68 * 3.0 + 74.9 * 0.02) / 6.0);
    assert_eq!(lv(Level::Param(Param::B)), 0.68 + 0.0016 / 1.0 * 74.9);
    assert_eq!(lv(Level::Param(Param::Lam)), 0.0016 / 1.0 * 18.72);
    // No horse traded: no value, and the text names the posted price's markup (O47, O54).
    let mut idle = x.clone();
    if let Some(c) = idle.chain.as_mut() {
        c.horse_traded = Some(false);
    }
    match value(Measure::Level(Level::PriceHorse), &idle, None) {
        Err(NoValue::Idle(why)) => assert!(why.contains("no horse traded"), "{why}"),
        other => panic!("{other:?}"),
    }
    // The trailing window: two idle ticks, and one tick the maker's markup fell below ψ.
    let low = MarkupInputs { pk: 5.0, ..inputs };
    assert!(maker_markup(&low) < 0.25);
    let tick = |horse_traded: bool, m: MarkupInputs, psi: f64| WindowTick {
        horse_traded,
        markup: Some(m),
        psi: Some(psi),
    };
    let mut win = x.clone();
    if let Some(c) = win.chain.as_mut() {
        c.window = vec![
            tick(true, inputs, 0.25),
            tick(false, low, 0.25),
            tick(false, inputs, 0.25),
            tick(true, low, 0.0),
        ];
    }
    let wv = |l: Level| value(Measure::Level(l), &win, None).unwrap();
    assert_eq!(wv(Level::IdleHorse), 2.0);
    assert_eq!(wv(Level::ReserveTicks), 1.0, "ψ 0 withholds nothing (L0.7)");
    // v1's readings have no chain: the chain's lenses have none, and the oracle lenses wait.
    let v1 = at_rest();
    assert!(matches!(
        value(Measure::Level(Level::HorsesPerHead), &v1, None),
        Err(NoValue::Missing(_))
    ));
    assert!(matches!(
        value(Measure::Level(Level::HorsesVsOracle), &v1, None),
        Err(NoValue::Unavailable(_))
    ));
    // On the chain without the tape's clock the oracle cannot be solved, and says so.
    assert!(matches!(
        value(Measure::Level(Level::GapOracle), &x, None),
        Err(NoValue::Missing(_))
    ));
}
