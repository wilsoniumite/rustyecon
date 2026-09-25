//! Settlement (docs/ENGINE.md §3.2): one filled quantity per order serves both sides, through
//! the market's escrow, with cash bound at the order (N7), no forgiveness (N8), typed holders
//! (defect 10) and rationing recorded per class (R12). Every check here is exact; no test in
//! this file needs a tolerance.

mod common;

use common::*;
use rustyecon_core::{
    apply, ActorId, Amount, DeskId, GoodId, Holder, Ledger, NodeId, Phase, PopId, RatePerYear,
    StateDelta,
};
use rustyecon_markets::{
    admit, clear, imbalance, settle, step, OrderError, RationLine, SettleLine, SideTag,
};
use std::collections::BTreeSet;

/// The transfers out of an escrow of one good, in plan order: (to, amount).
fn outs(plan: &[Delta], n: NodeId, g: GoodId, of: GoodId) -> Vec<(Holder, Amount)> {
    plan.iter()
        .filter_map(|d| match d {
            StateDelta::Transfer {
                from: Holder::Escrow(dn, dg),
                to,
                good,
                amount,
            } if (*dn, *dg, *good) == (n, g, of) => Some((*to, *amount)),
            _ => None,
        })
        .collect()
}

/// The quantity `apply` moved for the transfer of `good` from `from` to `to`.
fn moved_by(t: &Tick, from: Holder, to: Holder, good: GoodId) -> f64 {
    let i = t
        .plan
        .iter()
        .position(|d| {
            matches!(d, StateDelta::Transfer { from: f, to: o, good: g, .. }
                if (*f, *o, *g) == (from, to, good))
        })
        .expect("the transfer is planned");
    t.moved[i]
}

#[test]
fn cash_short_buyer_settles_both_sides_from_one_fill() {
    let (w, mut s) = load();
    let (town, bread, coin) = (node(&w, "town"), good(&w, "bread"), good(&w, "coin"));
    let (mill, pensioners) = (holder(&w, "mill"), holder(&w, "pensioners"));
    // The pensioners ask 10 bread at price 2 on a budget of 6, all their coin, so 3 is
    // feasible. The mill offers 5.
    let orders = vec![
        buy(&w, "pensioners", "town", "bread", 10.0, 6.0),
        sell(&w, "mill", "town", "bread", 5.0),
    ];
    let t = market_tick(&mut s, &w, orders).unwrap();
    let volumes = StateDelta::SetVolumes {
        node: town,
        good: bread,
        supply: 5.0,
        demand: 3.0,
    };
    assert!(t.volumes.contains(&volumes));
    let f = t.fills.get(town, bread).unwrap();
    assert_eq!((f.buyer_fill, f.seller_fill), (1.0, 0.6));
    // Both sides settle from the one filled quantity: the buyer gets 3 and pays 6, and the
    // seller ships 3 and gets 6.
    let b = line_of(&t, actor(&w, "pensioners"), town, bread, true);
    assert_eq!((b.qty, b.value), (3.0, 6.0));
    let sl = line_of(&t, actor(&w, "mill"), town, bread, false);
    assert_eq!((sl.qty, sl.value), (3.0, 6.0));
    assert_eq!(held(&s, pensioners, bread), 3.0);
    assert_eq!(held(&s, pensioners, coin), 0.0);
    assert_eq!((held(&s, mill, bread), held(&s, mill, coin)), (6.0, 56.0));
    // The class line says who went short and why: 10 asked, 3 affordable, 3 received.
    let line = |c: &str, side| {
        *t.rationing
            .iter()
            .find(|r| r.class == class(&w, c) && r.side == side)
            .unwrap()
    };
    let r = line("pensioners", SideTag::Buy);
    assert_eq!((r.requested, r.feasible, r.filled), (10.0, 3.0, 3.0));
    let r = line("producers", SideTag::Sell);
    assert_eq!((r.requested, r.feasible, r.filled), (5.0, 5.0, 3.0));
    // The price reads demand that can pay: x = (3 − 5)/5 = −0.4, so it falls. July's
    // settlement would have read D = 10, x = +0.5, and raised it.
    let x = imbalance(5.0, 3.0);
    assert_eq!(x, -0.4);
    let k = w.clock.log_step(RatePerYear(5.2));
    let p = s.price(town, bread).unwrap();
    assert_eq!(p, step(2.0, k, x));
    assert!(p < 2.0);
    assert!(step(2.0, k, imbalance(5.0, 10.0)) > 2.0);
    assert!(!t.escrow_left);
    assert!(t.audit.max_margin <= 1.0);
}

#[test]
fn two_kinds_same_number_settle_apart() {
    let (w, mut s) = load();
    // Desk(0) is the farm and Pop(0) the pensioners; Desk(1) is the mill and Pop(1) the
    // workers. July keyed currency flows by bare (u32, u32) and paid the wrong kind.
    assert_eq!(actor(&w, "farm"), ActorId::Desk(DeskId(0)));
    assert_eq!(actor(&w, "pensioners"), ActorId::Pop(PopId(0)));
    assert_eq!(actor(&w, "mill"), ActorId::Desk(DeskId(1)));
    assert_eq!(actor(&w, "workers"), ActorId::Pop(PopId(1)));
    let (bread, coin) = (good(&w, "bread"), good(&w, "coin"));
    let before = s.clone();
    let orders = vec![
        sell(&w, "mill", "town", "bread", 4.0),
        buy(&w, "farm", "town", "bread", 1.0, 2.0),
        buy(&w, "pensioners", "town", "bread", 1.0, 2.0),
        buy(&w, "workers", "town", "bread", 1.0, 2.0),
    ];
    let t = market_tick(&mut s, &w, orders).unwrap();
    // Each holder moved one quantity of coin and one of bread, exactly its own settle line.
    for key in ["farm", "mill", "pensioners", "workers"] {
        let a = actor(&w, key);
        let h = Holder::Actor(a);
        let mine: Vec<&SettleLine> = t.settled.iter().filter(|l| l.actor == a).collect();
        assert_eq!(mine.len(), 1, "{key}");
        let l = mine[0];
        let (coin0, bread0) = (held(&before, h, coin), held(&before, h, bread));
        match l.side {
            SideTag::Buy => {
                assert_eq!(held(&s, h, coin), coin0 - l.value, "{key}");
                assert_eq!(held(&s, h, bread), bread0 + l.qty, "{key}");
            }
            SideTag::Sell => {
                assert_eq!(held(&s, h, coin), coin0 + l.value, "{key}");
                assert_eq!(held(&s, h, bread), bread0 - l.qty, "{key}");
            }
        }
    }
    let got = |key: &str| {
        (
            held(&s, holder(&w, key), coin),
            held(&s, holder(&w, key), bread),
        )
    };
    assert_eq!(got("farm"), (98.0, 1.0));
    assert_eq!(got("pensioners"), (4.0, 1.0));
    assert_eq!(got("mill"), (56.0, 6.0));
    assert_eq!(got("workers"), (18.0, 1.0));
    // Every payment names a typed holder: Desk(0) and Pop(0) pay from two holdings.
    let payers: Vec<Holder> = t
        .plan
        .iter()
        .filter_map(|d| match d {
            StateDelta::Transfer {
                from: from @ Holder::Actor(_),
                good,
                ..
            } if *good == coin => Some(*from),
            _ => None,
        })
        .collect();
    let desk = |n| Holder::Actor(ActorId::Desk(DeskId(n)));
    let pop = |n| Holder::Actor(ActorId::Pop(PopId(n)));
    assert_eq!(payers, vec![desk(0), pop(0), pop(1)]);
}

#[test]
fn no_forgiveness_currency_moves_only_by_transfers() {
    let (w, mut s) = load();
    let before = s.clone();
    let orders = vec![
        // Town bread: the pensioners spend all 6 coin on 3 of the 10 they ask.
        sell(&w, "mill", "town", "bread", 5.0),
        buy(&w, "pensioners", "town", "bread", 10.0, 6.0),
        // Village grain, over-supplied: every buyer is filled.
        sell(&w, "farm", "village", "grain", 10.0),
        buy(&w, "mill", "village", "grain", 2.0, 5.0),
        buy(&w, "workers", "village", "grain", 3.0, 20.0),
        // Port bread, in florin, under-supplied.
        sell(&w, "mill", "port", "bread", 2.0),
        buy(&w, "workers", "port", "bread", 3.0, 10.0),
    ];
    let t = market_tick(&mut s, &w, orders).unwrap();
    // Settlement mints and burns nothing: every delta is a transfer, and the ledger has no
    // line at all.
    assert!(t
        .plan
        .iter()
        .all(|d| matches!(d, StateDelta::Transfer { .. })));
    assert!(t.audit.lines.is_empty());
    // Each holding of each currency is exactly its opening balance with the transfers that name
    // it applied in plan order: one lot, so a payment subtracts and a receipt adds. Nothing
    // else touches it; July clipped a desk's balance with `(old + revenue).min(0.0)`.
    for c in [good(&w, "coin"), good(&w, "florin")] {
        for key in ["farm", "mill", "pensioners", "workers"] {
            let h = holder(&w, key);
            let mut bal = held(&before, h, c);
            for (d, &m) in t.plan.iter().zip(&t.moved) {
                if let StateDelta::Transfer { from, to, good, .. } = d {
                    if *good == c && *from == h {
                        bal -= m;
                    }
                    if *good == c && *to == h {
                        bal += m;
                    }
                }
            }
            assert_eq!(held(&s, h, c), bal, "{key}");
        }
    }
    // A buyer that spent everything holds exactly 0, never less, and is advanced nothing: next
    // tick its budget is 0, it buys nothing, and its request is recorded.
    let pensioners = holder(&w, "pensioners");
    assert_eq!(held(&s, pensioners, good(&w, "coin")), 0.0);
    let t = market_tick(
        &mut s,
        &w,
        vec![
            sell(&w, "mill", "town", "bread", 1.0),
            buy(&w, "pensioners", "town", "bread", 1.0, 0.0),
        ],
    )
    .unwrap();
    assert_eq!(held(&s, pensioners, good(&w, "coin")), 0.0);
    let r = t
        .rationing
        .iter()
        .find(|r| r.class == class(&w, "pensioners"))
        .unwrap();
    assert_eq!((r.requested, r.feasible, r.filled), (1.0, 0.0, 0.0));
    assert!(
        t.plan.is_empty(),
        "a market with no feasible demand does not trade"
    );
}

#[test]
fn escrow_is_empty_after_settlement() {
    let (w, mut s) = load();
    let orders = vec![
        sell(&w, "mill", "town", "bread", 5.0),
        buy(&w, "pensioners", "town", "bread", 10.0, 6.0),
        buy(&w, "workers", "town", "bread", 1.0, 3.0),
        sell(&w, "farm", "village", "grain", 10.0),
        buy(&w, "mill", "village", "grain", 2.0, 5.0),
        // One-sided: the port's grain market records its supply and does not trade.
        sell(&w, "farm", "port", "grain", 3.0),
    ];
    let mut l = Ledger::open(&s, &w).unwrap();
    let lines = admit(orders, &s, &w).unwrap();
    let (volumes, fills) = clear(&lines, &w).unwrap();
    apply(&mut s, &w, Phase::Clearing, &volumes, &mut l).unwrap();
    let plan = settle(&lines, &fills, &s, &w).unwrap();
    // One delta at a time: each trading market's escrow appears in steps 1 and 2 and is gone
    // once step 4 has paid its last seller.
    let mut seen = BTreeSet::new();
    for d in plan.deltas() {
        apply(
            &mut s,
            &w,
            Phase::Settlement,
            std::slice::from_ref(d),
            &mut l,
        )
        .unwrap();
        for h in s.holdings().keys() {
            if let Holder::Escrow(n, g) = h {
                seen.insert((*n, *g));
            }
        }
    }
    let trading: BTreeSet<(NodeId, GoodId)> = [("town", "bread"), ("village", "grain")]
        .iter()
        .map(|(n, g)| (node(&w, n), good(&w, g)))
        .collect();
    assert_eq!(seen, trading);
    assert!(!s.holdings().keys().any(|h| matches!(h, Holder::Escrow(..))));
    let audit = l
        .close(&s, &w)
        .expect("no escrow is left and every good conserves");
    assert!(audit.max_margin <= 1.0);
}

#[test]
fn settlement_is_invariant_to_order_input_order() {
    let (w, s0) = load();
    let orders = vec![
        sell(&w, "mill", "town", "bread", 5.0),
        buy(&w, "pensioners", "town", "bread", 10.0, 3.0),
        buy(&w, "workers", "town", "bread", 2.5, 7.0),
        buy(&w, "farm", "town", "bread", 1.5, 4.0),
        sell(&w, "farm", "village", "grain", 7.0),
        sell(&w, "farm", "town", "grain", 2.0),
        buy(&w, "mill", "village", "grain", 3.0, 5.0),
        buy(&w, "workers", "village", "grain", 4.0, 6.0),
        buy(&w, "workers", "town", "grain", 1.0, 1.0),
        sell(&w, "mill", "port", "bread", 1.0),
        buy(&w, "workers", "port", "bread", 2.0, 9.0),
    ];
    let reference = market_tick(&mut s0.clone(), &w, orders.clone()).unwrap();
    let n = orders.len();
    let mut inputs = vec![orders.iter().rev().copied().collect::<Vec<_>>()];
    for r in 1..n {
        let mut o = orders.clone();
        o.rotate_left(r);
        inputs.push(o);
    }
    // A stride coprime with the length visits every order once, in a scattered order.
    inputs.push((0..n).map(|i| orders[(i * 4) % n]).collect());
    for input in inputs {
        let t = market_tick(&mut s0.clone(), &w, input).unwrap();
        assert_eq!(t.lines, reference.lines);
        assert_eq!(t.plan, reference.plan);
        assert_eq!(t.settled, reference.settled);
        assert_eq!(t.rationing, reference.rationing);
        assert_eq!(t.prices, reference.prices);
        assert_eq!(t.hash, reference.hash);
    }
}

#[test]
fn all_taker_is_largest_then_highest_id() {
    let text = edit(
        r#"(holder: "farm", goods: [("coin", 100.0), ("grain", 16.0)])"#,
        r#"(holder: "farm", goods: [("bread", 5.0), ("coin", 100.0), ("grain", 16.0)])"#,
    );
    let (w, s0) = load_text(&text);
    let (town, bread, coin) = (node(&w, "town"), good(&w, "bread"), good(&w, "coin"));
    let (farm, mill) = (holder(&w, "farm"), holder(&w, "mill"));
    let (pensioners, workers) = (holder(&w, "pensioners"), holder(&w, "workers"));
    let plan = |orders| market_tick(&mut s0.clone(), &w, orders).unwrap().plan;

    // Equal buyers: every one but the highest id takes its quantity, in actor order, and the
    // highest takes All.
    let p = plan(vec![
        sell(&w, "mill", "town", "bread", 3.0),
        buy(&w, "workers", "town", "bread", 1.0, 2.0),
        buy(&w, "farm", "town", "bread", 1.0, 2.0),
        buy(&w, "pensioners", "town", "bread", 1.0, 2.0),
    ]);
    assert_eq!(
        outs(&p, town, bread, bread),
        vec![
            (farm, Amount::Qty(1.0)),
            (pensioners, Amount::Qty(1.0)),
            (workers, Amount::All)
        ]
    );
    // A unique largest buyer goes last whatever its id: here the lowest.
    let p = plan(vec![
        sell(&w, "mill", "town", "bread", 4.0),
        buy(&w, "workers", "town", "bread", 1.0, 2.0),
        buy(&w, "farm", "town", "bread", 2.0, 4.0),
        buy(&w, "pensioners", "town", "bread", 1.0, 2.0),
    ]);
    assert_eq!(
        outs(&p, town, bread, bread),
        vec![
            (pensioners, Amount::Qty(1.0)),
            (workers, Amount::Qty(1.0)),
            (farm, Amount::All)
        ]
    );
    // Sellers are paid the same way, ranked by what they shipped.
    let p = plan(vec![
        sell(&w, "farm", "town", "bread", 2.0),
        sell(&w, "mill", "town", "bread", 2.0),
        buy(&w, "workers", "town", "bread", 4.0, 8.0),
    ]);
    assert_eq!(
        outs(&p, town, bread, coin),
        vec![(farm, Amount::Qty(4.0)), (mill, Amount::All)]
    );
    let p = plan(vec![
        sell(&w, "farm", "town", "bread", 3.0),
        sell(&w, "mill", "town", "bread", 1.0),
        buy(&w, "workers", "town", "bread", 4.0, 8.0),
    ]);
    assert_eq!(
        outs(&p, town, bread, coin),
        vec![(mill, Amount::Qty(2.0)), (farm, Amount::All)]
    );
}

#[test]
fn settle_lines_carry_moved_quantities() {
    // Two sellers offer 0.1 and 0.2 bread, 0.30000000000000004 in all, against a buyer of 0.3:
    // the seller fill is 0.3/0.30000000000000004 < 1, and the quantities the last takers get
    // differ from nominal by rounding.
    let text = edit(
        r#"(holder: "farm", goods: [("coin", 100.0), ("grain", 16.0)])"#,
        r#"(holder: "farm", goods: [("bread", 0.1), ("coin", 100.0), ("grain", 16.0)])"#,
    );
    let text = edit_text(&text, r#"("bread", 9.0)"#, r#"("bread", 0.2)"#);
    let (w, mut s) = load_text(&text);
    let (town, bread, coin) = (node(&w, "town"), good(&w, "bread"), good(&w, "coin"));
    let (farm, mill, workers) = (
        holder(&w, "farm"),
        holder(&w, "mill"),
        holder(&w, "workers"),
    );
    let escrow = Holder::Escrow(town, bread);
    let t = market_tick(
        &mut s,
        &w,
        vec![
            sell(&w, "farm", "town", "bread", 0.1),
            sell(&w, "mill", "town", "bread", 0.2),
            buy(&w, "workers", "town", "bread", 0.3, 20.0),
        ],
    )
    .unwrap();
    let f = *t.fills.get(town, bread).unwrap();
    assert!(f.seller_fill < 1.0 && f.buyer_fill == 1.0);
    let farm_l = *line_of(&t, actor(&w, "farm"), town, bread, false);
    let mill_l = *line_of(&t, actor(&w, "mill"), town, bread, false);
    let buyer = *line_of(&t, actor(&w, "workers"), town, bread, true);
    // Each line holds what apply moved for it.
    assert_eq!(farm_l.qty, moved_by(&t, farm, escrow, bread));
    assert_eq!(mill_l.qty, moved_by(&t, mill, escrow, bread));
    assert_eq!(buyer.value, moved_by(&t, workers, escrow, coin));
    assert_eq!(buyer.qty, moved_by(&t, escrow, workers, bread));
    assert_eq!(farm_l.value, moved_by(&t, escrow, farm, coin));
    assert_eq!(mill_l.value, moved_by(&t, escrow, mill, coin));
    // The last takers' quantities are not the nominal ones...
    let price = 2.0;
    assert_ne!(buyer.qty, 0.3 * f.buyer_fill);
    assert_ne!(mill_l.value, price * (0.2 * f.seller_fill));
    // ...and the good the buyer received is exactly what the sellers shipped, folded in actor
    // order, since the escrow holds it as one lot.
    assert_eq!(buyer.qty, farm_l.qty + mill_l.qty);
    assert_eq!(held(&s, workers, bread), buyer.qty);
    assert!(!t.escrow_left);
    assert!(t.audit.max_margin <= 1.0);
    // A moved list of the wrong length is refused.
    let (w, s) = load();
    let lines = admit(
        vec![
            sell(&w, "mill", "town", "bread", 1.0),
            buy(&w, "workers", "town", "bread", 1.0, 2.0),
        ],
        &s,
        &w,
    )
    .unwrap();
    let (_, fills) = clear(&lines, &w).unwrap();
    let plan = settle(&lines, &fills, &s, &w).unwrap();
    let n = plan.deltas().len();
    assert_eq!(
        plan.realize(&[]),
        Err(OrderError::Moved {
            deltas: n,
            moved: 0
        })
    );
}

#[test]
fn rationing_is_recorded_per_class() {
    // The pensioners also hold grain, to sell in the village.
    let text = edit(
        r#"(holder: "pensioners", goods: [("coin", 6.0)])"#,
        r#"(holder: "pensioners", goods: [("coin", 6.0), ("grain", 3.0)])"#,
    );
    let (w, mut s) = load_text(&text);
    let (town, village) = (node(&w, "town"), node(&w, "village"));
    let (bread, grain) = (good(&w, "bread"), good(&w, "grain"));
    // Town bread: supply 3 against feasible demand 6, so every buyer gets half of what it can
    // pay for. Village grain: the two producers buy, and the mill is cash-short (1.8 buys 2 of
    // the 5 it asks at 0.9); supply 3 against feasible demand 6 again. So the producers' buy
    // line of village grain sums two members: 4 + 5 asked, 4 + 2 affordable, 2 + 1 received.
    let t = market_tick(
        &mut s,
        &w,
        vec![
            sell(&w, "mill", "town", "bread", 3.0),
            buy(&w, "farm", "town", "bread", 2.0, 4.0),
            buy(&w, "pensioners", "town", "bread", 4.0, 2.0),
            buy(&w, "workers", "town", "bread", 3.0, 6.0),
            sell(&w, "pensioners", "village", "grain", 3.0),
            buy(&w, "farm", "village", "grain", 4.0, 10.0),
            buy(&w, "mill", "village", "grain", 5.0, 1.8),
        ],
    )
    .unwrap();
    let line = |n, g, c: &str, side, requested, feasible, filled| RationLine {
        node: n,
        good: g,
        class: class(&w, c),
        side,
        requested,
        feasible,
        filled,
    };
    // One line per (node, good, class, side) with an order, in that order, each the sum of its
    // members' orders in actor order.
    assert_eq!(
        t.rationing,
        vec![
            line(town, bread, "households", SideTag::Buy, 3.0, 3.0, 1.5),
            line(town, bread, "pensioners", SideTag::Buy, 4.0, 1.0, 0.5),
            line(town, bread, "producers", SideTag::Buy, 2.0, 2.0, 1.0),
            line(town, bread, "producers", SideTag::Sell, 3.0, 3.0, 3.0),
            line(village, grain, "pensioners", SideTag::Sell, 3.0, 3.0, 3.0),
            line(village, grain, "producers", SideTag::Buy, 9.0, 6.0, 3.0),
        ]
    );
    // Each member's own share is on its settle line.
    let farm = line_of(&t, actor(&w, "farm"), village, grain, true);
    let mill = line_of(&t, actor(&w, "mill"), village, grain, true);
    assert_eq!((farm.qty, mill.qty), (2.0, 1.0));
    // The recorded demand is the feasible sum, below what was asked.
    assert_eq!(s.demand(town, bread), Some(6.0));
    assert_eq!(s.supply(town, bread), Some(3.0));
    assert_eq!(s.demand(village, grain), Some(6.0));
}

#[test]
fn one_filled_quantity_serves_many_takers() {
    // §3.2 with several takers on a rationed side. Town bread is short: two sellers offer 4
    // against 4.5 feasible, so two buyers are filled at 8/9. Village grain is long: two sellers
    // offer 17 against 5 feasible, so two sellers ship at 5/17. On every line: a buyer pays
    // exactly p·(feasible·buyer_fill), and every buyer but the last receives exactly
    // feasible·buyer_fill; a seller ships exactly qty·seller_fill, and every seller but the
    // last is paid exactly p·(qty·seller_fill). Each escrow holds one lot here (the bread lots
    // share a life), so no take splits across lots and every nominal quantity is exact.
    let text = edit(
        r#"(holder: "farm", goods: [("coin", 100.0), ("grain", 16.0)])"#,
        r#"(holder: "farm", goods: [("bread", 5.0), ("coin", 100.0), ("grain", 16.0)])"#,
    );
    let text = edit_text(
        &text,
        r#"("bread", 9.0), ("coin", 50.0), ("florin", 30.0)"#,
        r#"("bread", 9.0), ("coin", 50.0), ("florin", 30.0), ("grain", 7.0)"#,
    );
    let (w, mut s) = load_text(&text);
    let (town, village) = (node(&w, "town"), node(&w, "village"));
    let (bread, grain) = (good(&w, "bread"), good(&w, "grain"));
    let orders = vec![
        sell(&w, "farm", "town", "bread", 1.5),
        sell(&w, "mill", "town", "bread", 2.5),
        buy(&w, "pensioners", "town", "bread", 10.0, 3.0),
        buy(&w, "workers", "town", "bread", 3.0, 6.0),
        sell(&w, "farm", "village", "grain", 10.0),
        sell(&w, "mill", "village", "grain", 7.0),
        buy(&w, "pensioners", "village", "grain", 2.0, 3.0),
        buy(&w, "workers", "village", "grain", 3.0, 5.0),
    ];
    let prices = [
        (town, bread, s.price(town, bread).unwrap()),
        (village, grain, s.price(village, grain).unwrap()),
    ];
    let t = market_tick(&mut s, &w, orders).unwrap();
    let mut rationed = [0, 0];
    for (n, g, p) in prices {
        let f = *t.fills.get(n, g).unwrap();
        assert!(f.trades());
        let lines: Vec<&rustyecon_markets::Line> = t
            .lines
            .iter()
            .filter(|l| (l.order.node, l.order.good) == (n, g))
            .collect();
        for (side, fill, i) in [
            (SideTag::Buy, f.buyer_fill, 0),
            (SideTag::Sell, f.seller_fill, 1),
        ] {
            let nominal = |l: &rustyecon_markets::Line| match side {
                SideTag::Buy => l.feasible * fill,
                SideTag::Sell => l.order.qty * fill,
            };
            let takers: Vec<&&rustyecon_markets::Line> = lines
                .iter()
                .filter(|l| l.order.side.tag() == side)
                .collect();
            let last = takers
                .iter()
                .max_by(|a, b| {
                    nominal(a)
                        .total_cmp(&nominal(b))
                        .then(a.order.actor.cmp(&b.order.actor))
                })
                .unwrap()
                .order
                .actor;
            if fill < 1.0 && takers.len() > 1 {
                rationed[i] += 1;
            }
            for l in takers {
                let q = nominal(l);
                let sl = line_of(&t, l.order.actor, n, g, side == SideTag::Buy);
                match side {
                    SideTag::Buy => {
                        assert_eq!(sl.value, p * q, "{sl:?}");
                        if l.order.actor != last {
                            assert_eq!(sl.qty, q, "{sl:?}");
                        }
                    }
                    SideTag::Sell => {
                        assert_eq!(sl.qty, q, "{sl:?}");
                        if l.order.actor != last {
                            assert_eq!(sl.value, p * q, "{sl:?}");
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        rationed,
        [1, 1],
        "two rationed buyers in one market, two sellers in another"
    );
    // The fills are the ones described: 8/9 of the bread buys and 5/17 of the grain sells.
    let bread_fill = t.fills.get(town, bread).unwrap();
    assert_eq!(bread_fill.buyer_fill, 4.0 / 4.5);
    assert_eq!(t.fills.get(village, grain).unwrap().seller_fill, 5.0 / 17.0);
    assert!(!t.escrow_left);
    assert!(t.audit.max_margin <= 1.0);
}

#[test]
fn a_market_whose_fill_underflows_does_not_trade() {
    // Amendment 5: with both sides posted, a fill is 0 only when S/D underflows below 2⁻¹⁰⁷⁴;
    // then nothing trades, rather than the sellers shipping into the escrow unpaid and the last
    // buyer taking the goods for nothing. Here the mill offers 1e-24 bread and the workers can
    // pay for 1e300 at a price of 1e-300.
    let (w, mut s) = load();
    let (town, bread, coin) = (node(&w, "town"), good(&w, "bread"), good(&w, "coin"));
    set_price(&mut s, &w, town, bread, 1e-300);
    let before = s.clone();
    let t = market_tick(
        &mut s,
        &w,
        vec![
            sell(&w, "mill", "town", "bread", 1e-24),
            buy(&w, "workers", "town", "bread", 1e300, 20.0),
        ],
    )
    .unwrap();
    let f = *t.fills.get(town, bread).unwrap();
    assert_eq!((f.supply, f.demand), (1e-24, 1e300));
    assert_eq!((f.buyer_fill, f.seller_fill), (0.0, 1.0));
    assert!(!f.trades());
    assert!(t.plan.is_empty(), "{:?}", t.plan);
    for key in ["mill", "workers"] {
        for g in [bread, coin] {
            assert_eq!(
                held(&s, holder(&w, key), g),
                held(&before, holder(&w, key), g)
            );
        }
    }
    let settled: Vec<(f64, f64)> = t.settled.iter().map(|l| (l.qty, l.value)).collect();
    assert_eq!(settled, [(0.0, 0.0), (0.0, 0.0)]);
}
