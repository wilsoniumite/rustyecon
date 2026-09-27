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
    centre, demo_gb, reference_value, value, CountyKeys, Level, Measure, NoValue, Readings,
    MARKETS, PARAMS_READ,
};
use rustyecon_worldgen::tables::{parse_lenses, Param, Scale};
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
        lv(Level::Dead),
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
        value(Measure::Level(Level::Dead), &none, None),
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
