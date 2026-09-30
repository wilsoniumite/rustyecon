//! The free step (amended at P2.4.11; FREE-SPEC §6.1, docs/probe/free/SPEC.md; decision 416): a
//! good with a free step moves by p′ = p·e^(kx) + (c·p_ref)·expm1(kx) and posts 0 where that is
//! not positive; at 0 a buy is feasible in full whatever its budget; only such a good may post 0.
//! Absent, or at c = 0, the step is `Imbalance`'s bit for bit.

mod common;

use common::*;
use rustyecon_core::{num, LoadErrorKind, OneSided, Phase, PriceRule, RatePerYear, StateDelta};
use rustyecon_markets::{next_price, next_price_free, step, update_prices, PriceError};

/// The fixture with a free step on grain, its reference bread, c the param `free.grain` at
/// `c` (grain is `Indefinite` and traded at every node).
fn free_grain(c: &str) -> String {
    let t = edit(
        r#"(key: "grain", life: Indefinite, price_rate: Some("rate.grain")),"#,
        r#"(key: "grain", life: Indefinite, price_rate: Some("rate.grain"), free: Some((reference: "bread", scale: "free.grain"))),"#,
    );
    edit_text(
        &t,
        r#"(key: "life.bread", value: 0.0577, unit: Years, basis: Assumed("three weeks")),"#,
        &format!(
            r#"(key: "life.bread", value: 0.0577, unit: Years, basis: Assumed("three weeks")),
        (key: "free.grain", value: {c}, unit: Dimensionless, basis: Assumed("the free step's c")),"#
        ),
    )
}

/// The fixture's per-tick log step for grain and bread: 5.2 a year at 52 ticks a year.
fn k(w: &W) -> f64 {
    w.clock.log_step(RatePerYear(5.2))
}

/// A small deterministic generator (SplitMix64) in [0, 1).
struct Draw(u64);

impl Draw {
    fn unit(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[test]
fn free_step_absent_is_saturate() {
    // FREE-SPEC §6.5 test 1: with the shift c·p_ref 0 the free step is `Imbalance`'s, bit for bit,
    // under both one-sided rules, over 3,000 random prices, rates and volumes, one side empty in
    // some; and a world whose good has no free step moves its price by `next_price`.
    let mut d = Draw(0x2026_0930);
    let mut one_sided_seen = 0;
    for i in 0..3000 {
        let p = num::exp(60.0 * d.unit() - 30.0);
        let k = 0.5 * d.unit();
        let (s, dm) = match i % 10 {
            0 => (0.0, 1.0 + d.unit()),
            1 => (1.0 + d.unit(), 0.0),
            2 => (0.0, 0.0),
            _ => (10.0 * d.unit(), 10.0 * d.unit()),
        };
        one_sided_seen += usize::from((s > 0.0) != (dm > 0.0));
        for os in [OneSided::Saturate, OneSided::Hold] {
            let free = next_price_free(os, p, k, s, dm, 0.0);
            let old = next_price(PriceRule::Imbalance, os, p, k, s, dm);
            assert_eq!(free.to_bits(), old.to_bits(), "{p:e} {k} {s} {dm} {os:?}");
        }
    }
    assert!(one_sided_seen > 500);
    // The fixture has no free step: its grain steps by `step`, as before.
    let (w, mut s) = load();
    assert!(w.free_step(good(&w, "grain")).is_none());
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    set_volumes(&mut s, &w, town, grain, 100.0, 50.0);
    price_step(&mut s, &w).unwrap();
    assert_eq!(s.price(town, grain).unwrap(), step(1.0, k(&w), -0.5));
    // With c set to 0 by a dated `SetParam`, a free-able good's step is `step`'s, bit for bit.
    let (w, mut s) = load_text(&free_grain("0.5"));
    let c = w.id_of("free.grain").unwrap();
    apply_in(
        &mut s,
        &w,
        Phase::Events,
        &[StateDelta::SetParam {
            param: c,
            value: 0.0,
        }],
    )
    .unwrap();
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    for (sup, dem) in [(100.0, 50.0), (50.0, 100.0), (3.0, 3.0)] {
        let before = s.price(town, grain).unwrap();
        set_volumes(&mut s, &w, town, grain, sup, dem);
        price_step(&mut s, &w).unwrap();
        let x = rustyecon_markets::imbalance(sup, dem);
        assert_eq!(
            s.price(town, grain).unwrap().to_bits(),
            step(before, k(&w), x).to_bits()
        );
    }
}

#[test]
fn free_step_posts_zero_and_reopens() {
    // FREE-SPEC §6.5 test 2: a free-able market with S > D steps to exactly +0.0 in finite ticks
    // and stays there while S ≥ D; at 0 with D > S it posts c·p_ref·expm1(kx) > 0; at 0, one-sided
    // under `Hold` (the fixture's rule), it stays 0. Its EMA may reach 0 too.
    let (w, mut s) = load_text(&free_grain("0.5"));
    let (town, grain, bread) = (node(&w, "town"), good(&w, "grain"), good(&w, "bread"));
    let mut ticks = 0;
    while s.price(town, grain).unwrap() > 0.0 {
        let p = s.price(town, grain).unwrap();
        let p_ref = s.price(town, bread).unwrap();
        set_volumes(&mut s, &w, town, grain, 100.0, 50.0);
        price_step(&mut s, &w).unwrap();
        let q = p * num::exp(k(&w) * -0.5) + (0.5 * p_ref) * num::expm1(k(&w) * -0.5);
        let want = if q > 0.0 { q } else { 0.0 };
        assert_eq!(s.price(town, grain).unwrap().to_bits(), want.to_bits());
        ticks += 1;
        assert!(ticks < 100, "the price never reached 0");
    }
    assert_eq!(s.price(town, grain).unwrap().to_bits(), 0.0_f64.to_bits());
    // It stays at 0 while S ≥ D, balanced included.
    for (sup, dem) in [(100.0, 50.0), (7.0, 7.0), (0.0, 0.0)] {
        set_volumes(&mut s, &w, town, grain, sup, dem);
        price_step(&mut s, &w).unwrap();
        assert_eq!(s.price(town, grain).unwrap().to_bits(), 0.0_f64.to_bits());
    }
    // One-sided under `Hold`: a buyer alone leaves it at 0.
    set_volumes(&mut s, &w, town, grain, 0.0, 5.0);
    price_step(&mut s, &w).unwrap();
    assert_eq!(s.price(town, grain).unwrap(), 0.0);
    // The EMA follows it down into the subnormals, where (1 − weight)·ema rounds back to the
    // smallest one; a genesis price of 0 gives an EMA of 0 (`only_free_goods_may_be_zero`).
    for _ in 0..20_000 {
        set_volumes(&mut s, &w, town, grain, 100.0, 50.0);
        price_step(&mut s, &w).unwrap();
    }
    let ema = s.ema(town, grain).unwrap();
    assert!((0.0..1e-300).contains(&ema), "{ema:e}");
    // D > S reopens it by itself, to c·p_ref·expm1(kx).
    let p_ref = s.price(town, bread).unwrap();
    set_volumes(&mut s, &w, town, grain, 50.0, 100.0);
    price_step(&mut s, &w).unwrap();
    let want = 0.0 * num::exp(k(&w) * 0.5) + (0.5 * p_ref) * num::expm1(k(&w) * 0.5);
    assert!(want > 0.0);
    assert_eq!(s.price(town, grain).unwrap().to_bits(), want.to_bits());
    // A step that is not a number is posted as it is, so the run stops, never as a free 0.
    let nan = next_price_free(
        OneSided::Saturate,
        f64::INFINITY,
        0.1,
        100.0,
        50.0,
        f64::INFINITY,
    );
    assert!(nan.is_nan());
    // Under `Saturate` a one-sided buyer reopens it too.
    let t = edit_text(&free_grain("0.5"), "one_sided: Hold", "one_sided: Saturate");
    let (w, mut s) = load_text(&t);
    set_price(&mut s, &w, town, grain, 0.0);
    set_volumes(&mut s, &w, town, grain, 0.0, 5.0);
    price_step(&mut s, &w).unwrap();
    assert!(s.price(town, grain).unwrap() > 0.0);
}

#[test]
fn free_market_takes_buys_for_nothing() {
    // FREE-SPEC §6.5 test 3: at a posted price of 0 a buy with budget 0 is feasible in full and
    // settles with no currency moved; at a positive price admission is `max_qty`'s, as before.
    let (w, mut s) = load_text(&free_grain("0.5"));
    let (town, grain, coin) = (node(&w, "town"), good(&w, "grain"), good(&w, "coin"));
    set_price(&mut s, &w, town, grain, 0.0);
    let text = with_holdings(
        &free_grain("0.5"),
        r#"
            (holder: "farm", goods: [("coin", 100.0), ("grain", 16.0)]),
            (holder: "mill", goods: [("bread", 9.0), ("coin", 50.0), ("florin", 30.0), ("grain", 16.0)]),
            (holder: "pensioners", goods: [("coin", 6.0)]),
            (holder: "workers", goods: [("coin", 20.0), ("florin", 10.0)]),"#,
    );
    let (w2, mut s2) = load_text(&text);
    set_price(&mut s2, &w2, town, grain, 0.0);
    let coins = |s: &S| {
        (
            held(s, holder(&w2, "workers"), coin),
            held(s, holder(&w2, "mill"), coin),
        )
    };
    let before = coins(&s2);
    let orders = vec![
        buy(&w2, "workers", "town", "grain", 5.0, 0.0),
        sell(&w2, "mill", "town", "grain", 8.0),
    ];
    let t = market_tick(&mut s2, &w2, orders).expect("the tick runs");
    let line = t
        .lines
        .iter()
        .find(|l| {
            l.order.good == grain && matches!(l.order.side, rustyecon_markets::Side::Buy { .. })
        })
        .expect("the buy is admitted");
    assert_eq!(line.feasible, 5.0);
    assert_eq!(held(&s2, holder(&w2, "workers"), grain), 5.0);
    assert_eq!(coins(&s2), before, "no currency moves at a price of 0");
    // A positive budget at 0 is taken at admission and nothing is paid.
    let (w3, mut s3) = load_text(&text);
    set_price(&mut s3, &w3, town, grain, 0.0);
    let before = held(&s3, holder(&w3, "workers"), coin);
    let orders = vec![
        buy(&w3, "workers", "town", "grain", 5.0, 3.0),
        sell(&w3, "mill", "town", "grain", 8.0),
    ];
    let t = market_tick(&mut s3, &w3, orders).expect("the tick runs");
    let bought = t
        .lines
        .iter()
        .find(|l| {
            l.order.good == grain && matches!(l.order.side, rustyecon_markets::Side::Buy { .. })
        })
        .expect("the buy is admitted");
    assert_eq!(bought.feasible, 5.0);
    assert_eq!(held(&s3, holder(&w3, "workers"), coin), before);
    // At a positive price the budget binds as before: 3 coin at 2 buys 1.5.
    set_price(&mut s, &w, town, grain, 2.0);
    let lines =
        rustyecon_markets::admit(vec![buy(&w, "workers", "town", "grain", 5.0, 3.0)], &s, &w)
            .expect("admitted");
    assert_eq!(lines[0].feasible, num::max_qty(3.0, 2.0).unwrap());
    assert_eq!(lines[0].feasible, 1.5);
    // Without a free step no price is 0, so no buy reaches the branch.
    let (w0, _) = load();
    assert!(w0.free_step(good(&w0, "grain")).is_none());
}

#[test]
fn only_free_goods_may_be_zero() {
    // FREE-SPEC §6.5 test 4: a genesis price of 0 on a good without a free step is refused and
    // accepted with one; a step to 0 on a good without one is `NonFinite`, with one it posts 0; a
    // `SetPrice` or `SetEma` of 0 is refused without one; and each load check of §6.1.
    let zero = |t: &str| {
        edit_text(
            t,
            r#"(node: "town", good: "grain", price: 1.0)"#,
            r#"(node: "town", good: "grain", price: 0.0)"#,
        )
    };
    let e = try_load(&zero(common::FIXTURE)).unwrap_err();
    assert_eq!(e.path, "genesis.prices[town/grain].price");
    assert!(matches!(e.kind, LoadErrorKind::BadValue(v) if v == 0.0));
    let (w, s) = load_text(&zero(&free_grain("0.5")));
    let (town, grain) = (node(&w, "town"), good(&w, "grain"));
    assert_eq!(s.price(town, grain), Some(0.0));
    assert_eq!(s.ema(town, grain), Some(0.0));
    // −0.0 is never a price.
    let neg = edit_text(
        &free_grain("0.5"),
        r#"(node: "town", good: "grain", price: 1.0)"#,
        r#"(node: "town", good: "grain", price: -0.0)"#,
    );
    assert!(try_load(&neg).is_err());
    // A step to 0 without a free step stops the run; with one it posts 0. At the smallest
    // subnormal and a rate of 5 a tick, a step of e^(−4.95) rounds to 0.
    let tiny = |t: &str| {
        let t = edit_text(
            t,
            r#"(node: "town", good: "grain", price: 1.0)"#,
            r#"(node: "town", good: "grain", price: 5e-324)"#,
        );
        edit_text(
            &t,
            r#"(key: "rate.grain", value: 5.2,"#,
            r#"(key: "rate.grain", value: 260.0,"#,
        )
    };
    let (w, mut s) = load_text(&tiny(common::FIXTURE));
    set_volumes(&mut s, &w, town, grain, 100.0, 1.0);
    assert!(matches!(
        update_prices(&s, &w),
        Err(PriceError::NonFinite { to, .. }) if to == 0.0
    ));
    let (w, mut s) = load_text(&tiny(&free_grain("0.5")));
    set_volumes(&mut s, &w, town, grain, 100.0, 1.0);
    price_step(&mut s, &w).unwrap();
    assert_eq!(s.price(town, grain), Some(0.0));
    // `SetPrice` and `SetEma` of 0: refused without a free step.
    let (w, mut s) = load();
    for d in [
        StateDelta::SetPrice {
            node: town,
            good: grain,
            price: 0.0,
        },
        StateDelta::SetEma {
            node: town,
            good: grain,
            ema: 0.0,
        },
    ] {
        assert!(apply_in(&mut s, &w, Phase::Prices, &[d]).is_err());
    }
    // A `ScalePrice` of a free price posts 0·f = 0; of any other price 0 stays refused.
    let (w, mut s) = load_text(&free_grain("0.5"));
    set_price(&mut s, &w, town, grain, 0.0);
    apply_in(
        &mut s,
        &w,
        Phase::Events,
        &[StateDelta::ScalePrice {
            node: town,
            good: grain,
            factor: 2.0,
        }],
    )
    .unwrap();
    assert_eq!(s.price(town, grain), Some(0.0));
    // The load checks, each at its path.
    let base = free_grain("0.5");
    let refuse = |text: String, path: &str| {
        let e = try_load(&text).unwrap_err();
        assert_eq!(e.path, path, "{e}");
    };
    // The reference is the good itself, a currency, or unknown.
    for (to, path) in [
        ("grain", "goods[grain].free.reference"),
        ("coin", "goods[grain].free.reference"),
        ("rye", "goods[grain].free.reference"),
    ] {
        refuse(
            edit_text(
                &base,
                r#"reference: "bread""#,
                &format!(r#"reference: "{to}""#),
            ),
            path,
        );
    }
    // The scale is not Dimensionless, or 0 at genesis.
    refuse(
        edit_text(
            &base,
            r#"(key: "free.grain", value: 0.5, unit: Dimensionless"#,
            r#"(key: "free.grain", value: 0.5, unit: RatePerYear"#,
        ),
        "goods[grain].free.scale",
    );
    refuse(free_grain("0.0"), "goods[grain].free.scale");
    // On a currency (no market), or under `Ratio`.
    let on_coin = edit_text(
        &edit_text(
            &base,
            r#"(key: "grain", life: Indefinite, price_rate: Some("rate.grain"), free: Some((reference: "bread", scale: "free.grain"))),"#,
            r#"(key: "grain", life: Indefinite, price_rate: Some("rate.grain")),"#,
        ),
        r#"(key: "coin", life: Indefinite, price_rate: None),"#,
        r#"(key: "coin", life: Indefinite, price_rate: None, free: Some((reference: "bread", scale: "free.grain"))),"#,
    );
    refuse(on_coin, "goods[coin].free");
    let ratio = edit_text(
        &base,
        "market: (rule: Imbalance, one_sided: Hold,",
        "market: (rule: Ratio, one_sided: Hold,",
    );
    refuse(ratio, "goods[grain].free");
}
