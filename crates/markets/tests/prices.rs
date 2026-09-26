//! The price update (docs/ENGINE.md §3.3): no absolute gate or guard (N5, N6), the rule and the
//! one-sided behaviour from the tape (N13, F8), no silent fallback, and every rate read at use
//! time. Every check here is exact: each step is compared bit for bit with the rule it follows.

mod common;

use common::*;
use rustyecon_core::{num, state_hash, LoadErrorKind, Phase, RatePerYear, StateDelta, Years};
use rustyecon_markets::{imbalance, step, update_prices, PriceError};

/// The fixture's per-tick log step for grain and bread: 5.2 a year at 52 ticks a year.
fn k(w: &W) -> f64 {
    w.clock.log_step(RatePerYear(5.2))
}

#[test]
fn tiny_prices_move_by_rule() {
    // July's `|p' − p| > 1e-12` gate froze any price below about 1e-12/α (N5). At 1e-15 and
    // 1e-300 each step here is exactly p·exp(k·x), up under excess demand and down under
    // excess supply.
    let text = edit(
        r#"(node: "town", good: "grain", price: 1.0)"#,
        r#"(node: "town", good: "grain", price: 1e-15)"#,
    );
    let text = edit_text(
        &text,
        r#"(node: "village", good: "grain", price: 0.9)"#,
        r#"(node: "village", good: "grain", price: 1e-300)"#,
    );
    let (w, mut s) = load_text(&text);
    let (town, village, grain) = (node(&w, "town"), node(&w, "village"), good(&w, "grain"));
    for (round, (supply, demand)) in [(50.0, 100.0), (100.0, 50.0)].into_iter().enumerate() {
        let x = imbalance(supply, demand);
        for i in 0..20 {
            let before = (
                s.price(town, grain).unwrap(),
                s.price(village, grain).unwrap(),
            );
            let ema = s.ema(town, grain).unwrap();
            set_volumes(&mut s, &w, town, grain, supply, demand);
            set_volumes(&mut s, &w, village, grain, supply, demand);
            price_step(&mut s, &w).unwrap();
            let after = (
                s.price(town, grain).unwrap(),
                s.price(village, grain).unwrap(),
            );
            assert_eq!(after.0, step(before.0, k(&w), x));
            assert_eq!(after.1, step(before.1, k(&w), x));
            assert_ne!(after.0.to_bits(), before.0.to_bits());
            assert_ne!(after.1.to_bits(), before.1.to_bits());
            // The EMA moves too, once the price has left it; July gated it the same way.
            if round > 0 || i > 0 {
                assert_ne!(s.ema(town, grain).unwrap().to_bits(), ema.to_bits());
            }
        }
    }
}

#[test]
fn huge_price_overflow_is_a_run_error() {
    // Nothing caps a price: when the rule leaves the finite positive range, the update is an
    // error that stops the run, and nothing is emitted.
    let text = edit(
        r#"(node: "town", good: "grain", price: 1.0)"#,
        r#"(node: "town", good: "grain", price: 1.7e308)"#,
    );
    let (w, mut s) = load_text(&text);
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    set_volumes(&mut s, &w, town, grain, 1.0, 100.0);
    let err = update_prices(&s, &w).unwrap_err();
    assert_eq!(
        err,
        PriceError::NonFinite {
            node: town,
            good: grain,
            from: 1.7e308,
            to: f64::INFINITY,
        }
    );
    // Downward too: at the smallest subnormal a step of exp(−5) rounds to 0, which is not a
    // price either.
    let text = edit(
        r#"(node: "town", good: "grain", price: 1.0)"#,
        r#"(node: "town", good: "grain", price: 5e-324)"#,
    );
    let text = edit_text(
        &text,
        r#"(key: "rate.grain", value: 5.2,"#,
        r#"(key: "rate.grain", value: 520.0,"#,
    );
    let (w, mut s) = load_text(&text);
    set_volumes(&mut s, &w, town, grain, 2.0, 1.0);
    let err = update_prices(&s, &w).unwrap_err();
    assert_eq!(
        err,
        PriceError::NonFinite {
            node: town,
            good: grain,
            from: 5e-324,
            to: 0.0,
        }
    );
}

#[test]
fn tiny_and_huge_prices_settle_without_guards() {
    // July switched the buyer's cash cap off below price 1e-9 and dropped currency flows under
    // 1e-12 (N6). Here 1e-200 flows trade at prices 1e-15 and 1e200, and cash binds at both.
    let text = edit(
        r#"(node: "town", good: "grain", price: 1.0)"#,
        r#"(node: "town", good: "grain", price: 1e-15)"#,
    );
    let text = edit_text(
        &text,
        r#"(node: "village", good: "bread", price: 2.2)"#,
        r#"(node: "village", good: "bread", price: 1e200)"#,
    );
    let text = with_holdings(
        &text,
        r#"
            (holder: "farm", goods: [("bread", 1e-200), ("coin", 100.0)]),
            (holder: "mill", goods: [("grain", 1e-200)]),
            (holder: "pensioners", goods: [("coin", 6.0)]),
            (holder: "workers", goods: [("coin", 5e-216)]),"#,
    );
    let (w, mut s) = load_text(&text);
    let (town, village) = (node(&w, "town"), node(&w, "village"));
    let (bread, coin, grain) = (good(&w, "bread"), good(&w, "coin"), good(&w, "grain"));
    let h = |key| holder(&w, key);
    let t = market_tick(
        &mut s,
        &w,
        vec![
            // At 1e-15 the workers' 5e-216 coin binds: they can pay for about half the grain.
            sell(&w, "mill", "town", "grain", 1e-200),
            buy(&w, "workers", "town", "grain", 1e-200, 5e-216),
            // At 1e200, 1e-200 bread costs 1 coin.
            sell(&w, "farm", "village", "bread", 1e-200),
            buy(&w, "pensioners", "village", "bread", 1e-200, 1.0),
        ],
    )
    .unwrap();
    // Grain at 1e-15: cash binds, both sides settle from the one fill, and the coin flows of
    // about 5e-216 all move.
    let feasible = num::max_qty(5e-216, 1e-15).unwrap();
    assert!(feasible < 1e-200 && feasible > 0.0);
    let f = *t.fills.get(town, grain).unwrap();
    assert_eq!((f.supply, f.demand, f.buyer_fill), (1e-200, feasible, 1.0));
    let b = *line_of(&t, actor(&w, "workers"), town, grain, true);
    let sl = *line_of(&t, actor(&w, "mill"), town, grain, false);
    assert_eq!(sl.qty, 1e-200 * f.seller_fill);
    assert_eq!(b.qty, sl.qty, "the one buyer takes the whole shipment");
    assert_eq!(b.value, 1e-15 * feasible);
    assert!(b.value > 0.0 && b.value <= 5e-216);
    assert_eq!(sl.value, b.value);
    assert_eq!(held(&s, h("workers"), grain), b.qty);
    assert_eq!(held(&s, h("workers"), coin), 5e-216 - b.value);
    assert_eq!(held(&s, h("mill"), coin), b.value);
    assert_eq!(held(&s, h("mill"), grain), 1e-200 - sl.qty);
    // Bread at 1e200: 1e-200 of it for 1 coin.
    let b = *line_of(&t, actor(&w, "pensioners"), village, bread, true);
    assert_eq!((b.qty, b.value), (1e-200, 1.0));
    assert_eq!(held(&s, h("pensioners"), bread), 1e-200);
    assert_eq!(held(&s, h("pensioners"), coin), 5.0);
    assert_eq!(held(&s, h("farm"), coin), 101.0);
    assert_eq!(held(&s, h("farm"), bread), 0.0);
    assert!(!t.escrow_left);
    assert!(t.audit.max_margin <= 1.0);
}

#[test]
fn price_rule_comes_from_the_tape() {
    // No rule, no load (N13): the rule and the one-sided behaviour are required fields.
    for missing in ["rule: Imbalance, ", "one_sided: Hold, "] {
        let text = edit(missing, "");
        let err = try_load(&text).unwrap_err();
        assert!(matches!(err.kind, LoadErrorKind::Parse(_)), "{err}");
    }
    // Ratio and Imbalance, from one tape each, give their own first steps and hash apart.
    let orders = |w: &W| {
        vec![
            sell(w, "farm", "village", "grain", 4.0),
            buy(w, "mill", "village", "grain", 2.0, 5.0),
            buy(w, "workers", "village", "grain", 4.0, 5.0),
        ]
    };
    let (wi, mut si) = load();
    let (wr, mut sr) = load_text(&edit("rule: Imbalance", "rule: Ratio"));
    assert_ne!(wi.world_id, wr.world_id);
    let (village, grain) = (node(&wi, "village"), good(&wi, "grain"));
    market_tick(&mut si, &wi, orders(&wi)).unwrap();
    market_tick(&mut sr, &wr, orders(&wr)).unwrap();
    // S = 4 and D = 6 at price 0.9.
    assert_eq!(
        si.price(village, grain),
        Some(step(0.9, k(&wi), imbalance(4.0, 6.0)))
    );
    assert_eq!(sr.price(village, grain), Some(0.9 * (6.0 / 4.0)));
    assert_ne!(state_hash(&si), state_hash(&sr));
}

#[test]
fn one_sided_rule_comes_from_the_tape() {
    // F8: at S = 0 < D, Saturate multiplies the price by exp(k) every tick, and Hold keeps its
    // bits. At S > 0 = D, Saturate divides by it.
    let (wh, mut sh) = load();
    let (ws, mut ss) = load_text(&edit("one_sided: Hold", "one_sided: Saturate"));
    let (town, grain) = (node(&wh, "town"), good(&wh, "grain"));
    for (supply, demand, x) in [(0.0, 1.0, 1.0), (1.0, 0.0, -1.0)] {
        for _ in 0..10 {
            let (ph, ps) = (
                sh.price(town, grain).unwrap(),
                ss.price(town, grain).unwrap(),
            );
            set_volumes(&mut sh, &wh, town, grain, supply, demand);
            set_volumes(&mut ss, &ws, town, grain, supply, demand);
            price_step(&mut sh, &wh).unwrap();
            price_step(&mut ss, &ws).unwrap();
            assert_eq!(sh.price(town, grain).unwrap().to_bits(), ph.to_bits());
            assert_eq!(ss.price(town, grain), Some(ps * num::exp(k(&ws) * x)));
        }
    }
    // Through the market phases: a buyer with no seller moves the price only under Saturate.
    let (wh, mut sh) = load();
    let (ws, mut ss) = load_text(&edit("one_sided: Hold", "one_sided: Saturate"));
    market_tick(
        &mut sh,
        &wh,
        vec![buy(&wh, "workers", "town", "grain", 2.0, 5.0)],
    )
    .unwrap();
    market_tick(
        &mut ss,
        &ws,
        vec![buy(&ws, "workers", "town", "grain", 2.0, 5.0)],
    )
    .unwrap();
    assert_eq!(sh.price(town, grain), Some(1.0));
    assert_eq!(ss.price(town, grain), Some(num::exp(k(&ws))));
}

#[test]
fn ratio_nonfinite_is_an_error() {
    // July's ratio rule kept the old price when the new one was not finite (N13's fallback).
    // Here the run stops.
    let text = edit("rule: Imbalance", "rule: Ratio");
    let text = edit_text(
        &text,
        r#"(node: "town", good: "grain", price: 1.0)"#,
        r#"(node: "town", good: "grain", price: 1e300)"#,
    );
    let (w, mut s) = load_text(&text);
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    // A finite ratio of 1e20 overflows the price...
    set_volumes(&mut s, &w, town, grain, 1e-10, 1e10);
    assert_eq!(
        update_prices(&s, &w),
        Err(PriceError::NonFinite {
            node: town,
            good: grain,
            from: 1e300,
            to: f64::INFINITY,
        })
    );
    // ...and so does a ratio that overflows by itself.
    set_volumes(&mut s, &w, town, grain, 1e-300, 1e300);
    assert!(matches!(
        update_prices(&s, &w),
        Err(PriceError::NonFinite { to, .. }) if to == f64::INFINITY
    ));
    // A finite step is exactly p·(D/S), and a one-sided market holds.
    set_volumes(&mut s, &w, town, grain, 4.0, 3.0);
    let ds = update_prices(&s, &w).unwrap();
    assert!(ds.contains(&StateDelta::SetPrice {
        node: town,
        good: grain,
        price: 1e300 * (3.0 / 4.0),
    }));
    set_volumes(&mut s, &w, town, grain, 0.0, 3.0);
    let ds = update_prices(&s, &w).unwrap();
    assert!(!ds.iter().any(
        |d| matches!(d, StateDelta::SetPrice { node, good, .. } if (*node, *good) == (town, grain))
    ));
}

#[test]
fn genesis_ema_is_the_genesis_price() {
    // July started every price and EMA at 1.0 (`v2p3: state/sim_state.rs:56,59`). Each market's
    // genesis price comes from the tape, its EMA is that price, and its volumes are 0.
    let (w, s) = load();
    let mut n = 0;
    for (node, good) in w.markets() {
        let q = s.book().quote(node, good).unwrap();
        assert_eq!(q.ema.to_bits(), q.price.to_bits());
        assert_eq!((q.supply, q.demand), (0.0, 0.0));
        n += 1;
    }
    assert_eq!(n, 6);
    let (town, bread) = (node(&w, "town"), good(&w, "bread"));
    assert_eq!(s.price(town, bread), Some(2.0));
    // An idle market keeps its price and EMA bits: no delta is emitted.
    assert!(update_prices(&s, &w).unwrap().is_empty());
}

#[test]
fn params_are_read_at_use_time() {
    let (w, mut s) = load();
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    let rate = w.id_of("rate.grain").unwrap();
    let tau = w.id_of("price.ema_tc").unwrap();
    // A posted price away from its EMA, and excess demand.
    set_price(&mut s, &w, town, grain, 3.0);
    set_volumes(&mut s, &w, town, grain, 1.0, 2.0);
    let expect = |rate_v: f64, tau_v: f64| {
        let p = step(3.0, w.clock.log_step(RatePerYear(rate_v)), 0.5);
        let wt = w.clock.weight(Years(tau_v));
        let ema = wt * 3.0 + (1.0 - wt) * 1.0;
        (p, ema)
    };
    let read = |s: &S| {
        let ds = update_prices(s, &w).unwrap();
        let price = ds.iter().find_map(|d| match d {
            StateDelta::SetPrice { node, good, price } if (*node, *good) == (town, grain) => {
                Some(*price)
            }
            _ => None,
        });
        let ema = ds.iter().find_map(|d| match d {
            StateDelta::SetEma { node, good, ema } if (*node, *good) == (town, grain) => Some(*ema),
            _ => None,
        });
        (price.unwrap(), ema.unwrap())
    };
    assert_eq!(read(&s), expect(5.2, 0.5));
    // A dated SetParam changes the next read, of the same book, with nothing cached.
    let set = |s: &mut S, param, value| {
        apply_in(
            s,
            &w,
            Phase::Events,
            &[StateDelta::SetParam { param, value }],
        )
        .unwrap();
    };
    set(&mut s, rate, 10.4);
    assert_eq!(read(&s), expect(10.4, 0.5));
    set(&mut s, tau, 0.25);
    assert_eq!(read(&s), expect(10.4, 0.25));
    set(&mut s, rate, 5.2);
    set(&mut s, tau, 0.5);
    assert_eq!(read(&s), expect(5.2, 0.5));
}

#[test]
fn tiny_imbalances_move_the_price() {
    // O13 (P0.9), N5, A12: no dead band sits on the imbalance. However small x = (D − S)/
    // max(S, D) is, the price steps by exactly p·exp(k·x), and moves wherever exp(k·x) is not 1
    // in f64. A hidden absolute threshold on x (|x| below 1e-9 read as 0, say), however it is
    // spelled, fails here; the source scan (`no_behavioural_float_literals`) catches its
    // spelling as a float literal or as an integer made float.
    let (w, mut s) = load();
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    for gap in [1e-12, 1e-9, 1e-6] {
        let (supply, demand) = (1000.0, 1000.0 * (1.0 + gap));
        let x = imbalance(supply, demand);
        assert!(x > 0.0 && x < 2.0 * gap, "{gap}: x = {x}");
        let before = s.price(town, grain).unwrap();
        set_volumes(&mut s, &w, town, grain, supply, demand);
        price_step(&mut s, &w).unwrap();
        let after = s.price(town, grain).unwrap();
        assert_eq!(after, step(before, k(&w), x), "{gap}");
        assert_eq!(after, before * num::exp(k(&w) * x), "{gap}");
        assert!(after > before, "{gap}: the price moved");
    }
}
