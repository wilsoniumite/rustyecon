//! The registry listing (docs/ENGINE.md §5 and §7.1; D10 item 4, amended at S2.2): every param
//! with each use the run makes of it, at its tape path, with the conversion that use takes and
//! its per-tick value. Every comparison is exact: a per-tick value is one call of a `Clock`
//! method, computed here the same way.

mod common;

use common::*;
use rustyecon_core::{FlowPerYear, RatePerYear, Unit, Years};
use rustyecon_engine::prelude::*;
use rustyecon_engine::{registry, Entry, RegistryLine, SiteLine, Use};

/// The probe's world (docs/probe/RULES.md), for a second tape.
const APPB: &str = include_str!("../../../tapes/appb.ron");

/// The listing's row for the param `key`.
fn row<'a>(lines: &'a [RegistryLine], key: &str) -> &'a RegistryLine {
    let path = format!("params[{key}]");
    lines
        .iter()
        .find(|l| l.path == path)
        .unwrap_or_else(|| panic!("no row for {key}"))
}

/// A param row's unit, use and sites.
fn param_entry(line: &RegistryLine) -> (Unit, Use, &[SiteLine]) {
    match &line.entry {
        Entry::Param { unit, use_, sites } => (*unit, *use_, sites.as_slice()),
        Entry::Inline => panic!("{} is inline", line.path),
    }
}

fn site(path: &str, method: ClockMethod, per_tick: f64) -> SiteLine {
    SiteLine {
        path: path.to_string(),
        method,
        per_tick,
    }
}

#[test]
fn registry_names_each_use() {
    // A `RatePerYear` is a share of a stock where it draws on one and a log step where it moves
    // a price; only the use knows which. Before S2.2 the listing guessed one method per param,
    // from the unit and from whether some good's price moved at it. On a gate variant whose
    // mill spends at `rate.bread`, the bread's price rate, with `mill.spend` deleted, the row
    // lists both uses, each converted as the run converts it there.
    let text = edit(
        r#"spend: Some("mill.spend"),"#,
        r#"spend: Some("rate.bread"),"#,
    );
    let text = edit_text(
        &text,
        r#"(key: "mill.spend", value: 10.4, unit: RatePerYear, basis: Assumed("gate world")),"#,
        "",
    );
    let t = tape_of(&text);
    let clock = sim_of(&t).world().clock;
    let lines = registry(&t).expect("the variant lists");
    let bread = row(&lines, "rate.bread");
    let (unit, use_, sites) = param_entry(bread);
    assert_eq!((unit, use_), (Unit::RatePerYear, Use::Live));
    let share = clock.share(RatePerYear(5.2));
    let log_step = clock.log_step(RatePerYear(5.2));
    assert_ne!(share, log_step);
    assert_eq!(
        sites,
        [
            site("actors[mill].spec.spend", ClockMethod::Share, share),
            site("goods[bread].price_rate", ClockMethod::LogStep, log_step),
        ]
    );
    let printed = bread.to_string();
    for part in [
        format!("share {share:e} per tick at actors[mill].spec.spend"),
        format!("log_step {log_step:e} per tick at goods[bread].price_rate"),
    ] {
        assert!(printed.contains(&part), "{part:?} in {printed:?}");
    }
    assert!(lines.iter().all(|l| l.path != "params[mill.spend]"));

    // On the gate tape itself, one row of each kind of use: its unit, its use, and each site.
    let t = tape();
    let lines = registry(&t).expect("the gate lists");
    let flow = |v| clock.flow(FlowPerYear(v));
    let cases: [(&str, Unit, Use, Vec<SiteLine>); 10] = [
        (
            "ledger.rel_flow",
            Unit::Dimensionless,
            Use::Fixed,
            vec![site("header.ledger.rel_flow", ClockMethod::Value, 1e-12)],
        ),
        (
            "price.ema_tc",
            Unit::Years,
            Use::Live,
            vec![site(
                "header.market.ema_time_constant",
                ClockMethod::Weight,
                clock.weight(Years(0.5)),
            )],
        ),
        (
            "life.bread",
            Unit::Years,
            Use::Fixed,
            vec![site("goods[bread].life", ClockMethod::Ticks, 3.0)],
        ),
        (
            "rate.grain",
            Unit::RatePerYear,
            Use::Live,
            vec![site(
                "goods[grain].price_rate",
                ClockMethod::LogStep,
                clock.log_step(RatePerYear(5.2)),
            )],
        ),
        (
            "mill.spend",
            Unit::RatePerYear,
            Use::Live,
            vec![site(
                "actors[mill].spec.spend",
                ClockMethod::Share,
                clock.share(RatePerYear(10.4)),
            )],
        ),
        (
            "farm.payout",
            Unit::RatePerYear,
            Use::Live,
            vec![site(
                "actors[farm].spec.payout.rate",
                ClockMethod::Share,
                clock.share(RatePerYear(5.2)),
            )],
        ),
        // A SetParam's target is written there, not read: its uses are the world's.
        (
            "mine.capacity",
            Unit::FlowPerYear,
            Use::Live,
            vec![site(
                "actors[mine].spec.recipe.capacity",
                ClockMethod::Flow,
                flow(52.0),
            )],
        ),
        (
            "mill.buy.grain.village",
            Unit::FlowPerYear,
            Use::Live,
            vec![site(
                "actors[mill].spec.buy[village/grain].qty",
                ClockMethod::Flow,
                flow(156.0),
            )],
        ),
        // Read by the schedule alone: a source takes its target's method at the event's `to`,
        // and a period is whole ticks at its `every`.
        (
            "mine.capacity.cut",
            Unit::FlowPerYear,
            Use::Schedule,
            vec![site(
                "events[mine.cut].act.to",
                ClockMethod::Flow,
                flow(26.0),
            )],
        ),
        (
            "pension.period",
            Unit::Years,
            Use::Schedule,
            vec![site("recurring[pension].every", ClockMethod::Ticks, 52.0)],
        ),
    ];
    for (key, unit, use_, want) in cases {
        assert_eq!(
            param_entry(row(&lines, key)),
            (unit, use_, want.as_slice()),
            "{key}"
        );
    }

    // Every param of the gate and the probe's world is read somewhere, and every site's
    // per-tick value is its method's conversion of the listed value.
    for text in [GATE, APPB] {
        let t = tape_of(text);
        let clock = sim_of(&t).world().clock;
        let lines = registry(&t).expect("the tape lists");
        let params = lines.iter().filter(|l| l.path.starts_with("params["));
        assert_eq!(params.clone().count(), t.params.len());
        for line in params {
            let (unit, _, sites) = param_entry(line);
            assert!(!sites.is_empty(), "{} is read somewhere", line.path);
            for s in sites {
                assert_eq!(s.method.unit(), unit, "{} at {}", line.path, s.path);
                let want = s.method.per_tick(&clock, line.value).unwrap();
                assert_eq!(s.per_tick.to_bits(), want.to_bits(), "{}", s.path);
            }
        }
    }
}
