//! Tick length (A13): the EMA's step response and the `Imbalance` path under a constant
//! imbalance are the same after one year at 12, 52 and 365 ticks a year, and equal their
//! continuous-time forms. July's `EMA_ALPHA = 2/53` and per-tick α were tied to weekly ticks.

mod common;

use common::*;
use rustyecon_core::{num, FlowPerYear};

/// Relative bar on a quantity composed tick by tick for one year. Each tick rounds a handful of
/// times at 1.1e-16, so 365 ticks accumulate a few parts in 1e-13 at most; 1e-12 leaves room,
/// and a per-tick rule that ignored the tick length would miss by order Δ, 3e-3 or more.
const COMPOSE_BAR: f64 = 1e-12;

/// The tick lengths compared.
const TICKS: [u32; 3] = [12, 52, 365];

fn world_at(tpy: u32) -> (W, S) {
    load_text(&edit(
        "ticks_per_year: 52",
        &format!("ticks_per_year: {tpy}"),
    ))
}

fn rel(a: f64, b: f64) -> f64 {
    ((a - b) / b).abs()
}

#[test]
fn ema_is_tick_length_invariant() {
    // Town grain's price steps from 1 to 3 at t = 0 and holds (the market is idle). After one
    // year the EMA's remaining gap has decayed by exp(−1/τ), τ = 0.5 years, at every tick
    // length.
    let decay = num::exp(-1.0 / 0.5);
    let mut gaps = Vec::new();
    for tpy in TICKS {
        let (w, mut s) = world_at(tpy);
        let (town, grain) = (node(&w, "town"), good(&w, "grain"));
        set_price(&mut s, &w, town, grain, 3.0);
        for _ in 0..tpy {
            price_step(&mut s, &w).unwrap();
        }
        assert_eq!(s.price(town, grain), Some(3.0), "an idle market holds");
        let gap = (s.ema(town, grain).unwrap() - 3.0) / (1.0 - 3.0);
        assert!(
            rel(gap, decay) <= COMPOSE_BAR,
            "{tpy} ticks a year: gap {gap} against {decay}"
        );
        gaps.push(gap);
    }
    for g in &gaps[1..] {
        assert!(rel(*g, gaps[0]) <= COMPOSE_BAR, "{gaps:?}");
    }
}

#[test]
fn imbalance_path_is_tick_length_invariant() {
    // The farm sells 1 grain a year in town and the workers buy 2 a year with all their cash,
    // which never binds: x = 0.5 every tick. After one year the price is exp(r·x) at rate
    // r = 5.2 a year, at every tick length, through the market phases.
    let target = num::exp(5.2 * 0.5);
    let mut ends = Vec::new();
    for tpy in TICKS {
        let (w, mut s) = world_at(tpy);
        let (town, grain, coin) = (node(&w, "town"), good(&w, "grain"), good(&w, "coin"));
        let flow = |v| w.clock.flow(FlowPerYear(v));
        for _ in 0..tpy {
            let cash = held(&s, holder(&w, "workers"), coin);
            let orders = vec![
                sell(&w, "farm", "town", "grain", flow(1.0)),
                buy(&w, "workers", "town", "grain", flow(2.0), cash),
            ];
            let t = market_tick(&mut s, &w, orders).unwrap();
            let f = t.fills.get(town, grain).unwrap();
            assert_eq!(
                (f.supply, f.demand),
                (flow(1.0), flow(2.0)),
                "cash never binds"
            );
        }
        let p = s.price(town, grain).unwrap();
        assert!(
            rel(p, target) <= COMPOSE_BAR,
            "{tpy} ticks a year: {p} against {target}"
        );
        ends.push(p);
    }
    for p in &ends[1..] {
        assert!(rel(*p, ends[0]) <= COMPOSE_BAR, "{ends:?}");
    }
}
