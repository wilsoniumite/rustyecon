//! Shared helpers for certify's tests: the tapes, a test build, criteria written as RON with
//! each test file's bars, and synthetic observations.

// Each test file is its own crate and uses part of this module.
#![allow(dead_code)]

use certify::{Build, Criteria, MarketObs, Names, Obs, Segment};
use rustyecon_engine::prelude::{GoodId, NodeId, Sim, Tape};

/// The gate world (docs/ENGINE.md §10).
pub const GATE: &str = include_str!("../../../../tapes/gate.ron");
/// The probe's Appendix B world.
pub const APPB: &str = include_str!("../../../../tapes/appb.ron");
/// appb under bcycle(1500,4): four dated changes of b, 1,500 ticks apart (docs/CERTIFY.md §13).
pub const BCYCLE: &str = include_str!("../../testdata/appb-bcycle.ron");
/// appb at desk turnover ×16 from w×1.05: converged, and frozen at an unstable point.
pub const FREEZE: &str = include_str!("../../testdata/appb-freeze.ron");
/// appb under July's step rule from w×2: the negative control, which runs away.
pub const JULY: &str = include_str!("../../testdata/appb-july.ron");
/// appb at desk turnover ×16 from its exact genesis: the probe's false-GO turnover cell, frozen
/// at an unstable point from the first tick (the verification's E4, S2.5).
pub const BUFFER16: &str = include_str!("../../testdata/appb-buffer16.ron");

/// The registered criteria of the gate world.
pub const GATE_CRITERIA: &str = include_str!("../../../../criteria/gate-2026-09-26.ron");
/// The registered criteria of appb.
pub const APPB_CRITERIA: &str = include_str!("../../../../criteria/appb-2026-09-26.ron");

/// A tape's registered criteria, `criteria/<name>-2026-09-26.ron`.
pub fn registered(name: &str) -> Criteria {
    let text = match name {
        "gate" => GATE_CRITERIA,
        "appb" => APPB_CRITERIA,
        _ => panic!("no registered criteria for {name}"),
    };
    Criteria::from_ron(text, &file(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// A battery spec as the RON a criteria file lists it by.
pub fn spec_ron(b: &certify::BatterySpec) -> String {
    ron::to_string(b).expect("a battery serialises")
}

/// Criteria text that registers `n` `ScalePrice` firings (amended at S2.5).
pub fn with_price_shocks(text: &str, n: u64) -> String {
    assert_eq!(text.matches("    batteries: [").count(), 1);
    text.replacen(
        "    batteries: [",
        &format!(
            "    price_shocks: Some((value: {n}, basis: Assumed(\"test\"))),\n    batteries: ["
        ),
        1,
    )
}

/// A tape.
pub fn tape(text: &str) -> Tape {
    Tape::from_ron(text).expect("the tape parses")
}

/// A test build: a clean 40-hex commit.
pub fn build() -> Build {
    Build {
        commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
        dirty: false,
        target: "test-target".to_string(),
        rustc: "rustc test".to_string(),
    }
}

/// The date of a tape's tick.
pub fn date_of(t: &Tape, tick: u64) -> String {
    rustyecon_engine::prelude::Clock {
        start: t.header.start,
        ticks_per_year: t.header.ticks_per_year,
    }
    .date_of(tick)
    .expect("a date")
    .to_string()
}

/// A bar in RON.
pub fn bar(value: f64, unit: &str) -> String {
    format!("(value: {value:?}, unit: {unit}, basis: Assumed(\"test\"))")
}

pub fn conservation() -> String {
    "Conservation".to_string()
}

pub fn determinism(shares: &[f64]) -> String {
    let bars: Vec<String> = shares.iter().map(|s| bar(*s, "Share")).collect();
    format!("Determinism(resume_at: [{}])", bars.join(", "))
}

pub fn runaway(bound: f64) -> String {
    format!("Runaway(bound: {})", bar(bound, "Ratio"))
}

pub fn trades(every: f64) -> String {
    format!("Trades(every: {})", bar(every, "Years"))
}

pub fn balance(level: f64, spread: f64, min_samples: u64, run_share: f64) -> String {
    format!(
        "Balance(level: {}, spread: {}, min_samples: (value: {min_samples}, basis: \
         Assumed(\"test\")), run_share: {})",
        bar(level, "Imbalance"),
        bar(spread, "Imbalance"),
        bar(run_share, "Share")
    )
}

pub fn settles(w_from: f64, f_from: f64, dead_share: f64, band: f64) -> String {
    format!(
        "Settles(w_from: {}, f_from: {}, dead_share: {}, band: {})",
        bar(w_from, "Share"),
        bar(f_from, "Share"),
        bar(dead_share, "Share"),
        bar(band, "LogWidth")
    )
}

pub fn kick(size: f64, horizon: f64, tail: f64, max_gain: f64, max_peak: f64) -> String {
    format!(
        "Kick(size: {}, horizon: {}, tail: {}, max_gain: {}, max_peak: {})",
        bar(size, "Relative"),
        bar(horizon, "Years"),
        bar(tail, "Share"),
        bar(max_gain, "Gain"),
        bar(max_peak, "Ratio")
    )
}

/// Criteria text for tape `name` with hash `hash`, run to `until`, with the batteries listed.
pub fn criteria_text(
    name: &str,
    hash: u64,
    until: &str,
    min_segment: f64,
    batteries: &[String],
    rationed_below: f64,
) -> String {
    format!(
        "Criteria(\n    format: 1,\n    date: \"2026-09-26\",\n    reason: \"test\",\n    \
         supersedes: None,\n    tape: (name: \"{name}\", tape_hash: \"0x{hash:016x}\"),\n    \
         until: \"{until}\",\n    until_basis: Assumed(\"test\"),\n    min_segment: {},\n    \
         batteries: [{}],\n    reports: (rationed_below: {}),\n)\n",
        bar(min_segment, "Years"),
        batteries.join(", "),
        bar(rationed_below, "Relative")
    )
}

/// The criteria file name for tape `name`.
pub fn file(name: &str) -> String {
    format!("{name}-2026-09-26.ron")
}

/// Load criteria text for tape `name`.
pub fn criteria(name: &str, text: &str) -> Criteria {
    Criteria::from_ron(text, &file(name)).unwrap_or_else(|e| panic!("the criteria load: {e}"))
}

/// A market's tick with the clearing's fills: `min(S/D, 1)` and `min(D/S, 1)`, 0 on an empty
/// side, and its price posted again.
pub fn market(price: f64, supply: f64, demand: f64, cleared: f64) -> MarketObs {
    MarketObs {
        node: NodeId(0),
        good: GoodId(0),
        price,
        next_price: price,
        supply,
        demand,
        cleared,
        buyer_fill: if demand > 0.0 {
            (supply / demand).min(1.0)
        } else {
            0.0
        },
        seller_fill: if supply > 0.0 {
            (demand / supply).min(1.0)
        } else {
            0.0
        },
    }
}

/// A tick of synthetic markets and nothing else.
pub fn obs(tick: u64, markets: Vec<MarketObs>) -> Obs {
    Obs {
        tick,
        markets,
        rationing: Vec::new(),
        consumed: Vec::new(),
        spoiled: Vec::new(),
        transfers: Vec::new(),
        margin: 0.0,
        run_margin: 0.0,
        price_shocks: 0,
    }
}

/// `n` ticks of one market at rest: price 1, supply = demand = cleared = 1, with `f(t)` applied.
pub fn at_rest(n: u64, f: impl Fn(u64, &mut MarketObs)) -> Vec<Obs> {
    (0..n)
        .map(|t| {
            let mut m = market(1.0, 1.0, 1.0, 1.0);
            f(t, &mut m);
            obs(t, vec![m])
        })
        .collect()
}

/// Names for `n` synthetic markets, `m/0` …
pub fn names(n: usize) -> Names {
    Names {
        markets: (0..n).map(|i| format!("m/{i}")).collect(),
        goods: vec!["g".to_string()],
        classes: vec!["c".to_string()],
        actors: Vec::new(),
    }
}

/// One segment `[from, to)`.
pub fn seg(from: u64, to: u64) -> Segment {
    Segment {
        from,
        to,
        opened_by: Vec::new(),
        merged: Vec::new(),
    }
}

/// A `Sim` of a tape.
pub fn sim(t: &Tape) -> Sim {
    Sim::new(t).expect("the tape loads")
}
