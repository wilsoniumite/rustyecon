//! Admission (docs/ENGINE.md §3.1): cash binds at the order (N7), sells are checked
//! cumulatively across nodes, and every order names a declared actor, its registered class and a
//! market. Every check here is exact; no test in this file needs a tolerance.

mod common;

use common::*;
use rustyecon_core::num;
use rustyecon_core::{ActorId, Amount, DeskId, GoodId, NodeId, Phase, Provenance, StateDelta};
use rustyecon_markets::{admit, Order, OrderError, Side, SideTag};

#[test]
fn over_budget_is_an_order_error() {
    let (w, s) = load();
    // The pensioners hold 6 coin. A budget of 6 fits whole; 6.5 does not.
    let ok = admit(
        vec![buy(&w, "pensioners", "village", "grain", 10.0, 6.0)],
        &s,
        &w,
    )
    .unwrap();
    assert_eq!(ok.len(), 1);
    let err = admit(
        vec![buy(&w, "pensioners", "village", "grain", 10.0, 6.5)],
        &s,
        &w,
    )
    .unwrap_err();
    let OrderError::OverBudget {
        order,
        currency,
        remaining,
    } = err
    else {
        panic!("expected OverBudget, got {err:?}");
    };
    assert_eq!(
        (order.qty, currency, remaining),
        (10.0, good(&w, "coin"), 6.0)
    );
    // Budgets are taken in (node, good) order and each binds whole, whatever it buys: town
    // (node 1) before village (node 2), so 4 at town leaves 2 for the village.
    let err = admit(
        vec![
            buy(&w, "pensioners", "village", "grain", 1.0, 4.0),
            buy(&w, "pensioners", "town", "bread", 1.0, 4.0),
        ],
        &s,
        &w,
    )
    .unwrap_err();
    let OrderError::OverBudget {
        order, remaining, ..
    } = err
    else {
        panic!("expected OverBudget, got {err:?}");
    };
    assert_eq!((order.node, remaining), (node(&w, "village"), 2.0));
    // Two budgets that exhaust the cash exactly are admitted.
    let lines = admit(
        vec![
            buy(&w, "pensioners", "village", "grain", 1.0, 2.5),
            buy(&w, "pensioners", "town", "bread", 1.0, 3.5),
        ],
        &s,
        &w,
    )
    .unwrap();
    assert_eq!(lines.len(), 2);
}

#[test]
fn over_posting_seller_is_an_order_error() {
    let (w, s) = load();
    // The farm holds 16 grain: offering 16 fits, 16.5 does not, and the mill has none to offer.
    assert!(admit(vec![sell(&w, "farm", "village", "grain", 16.0)], &s, &w).is_ok());
    let err = admit(vec![sell(&w, "farm", "village", "grain", 16.5)], &s, &w).unwrap_err();
    assert!(
        matches!(err, OrderError::OverPosted { order, remaining } if order.qty == 16.5 && remaining == 16.0),
        "{err:?}"
    );
    let err = admit(vec![sell(&w, "mill", "town", "grain", 1.0)], &s, &w).unwrap_err();
    assert!(
        matches!(err, OrderError::OverPosted { remaining, .. } if remaining == 0.0),
        "{err:?}"
    );
}

#[test]
fn sells_at_two_nodes_are_checked_cumulatively() {
    let (w, s) = load();
    // 10 at town and 6 at the village use the farm's 16 grain exactly.
    let lines = admit(
        vec![
            sell(&w, "farm", "village", "grain", 6.0),
            sell(&w, "farm", "town", "grain", 10.0),
        ],
        &s,
        &w,
    )
    .unwrap();
    assert_eq!(lines.len(), 2);
    // 10 and 7 do not: each fits alone, and the second in node order (the village) is refused
    // with what the first left.
    let err = admit(
        vec![
            sell(&w, "farm", "town", "grain", 10.0),
            sell(&w, "farm", "village", "grain", 7.0),
        ],
        &s,
        &w,
    )
    .unwrap_err();
    let OrderError::OverPosted { order, remaining } = err else {
        panic!("expected OverPosted, got {err:?}");
    };
    assert_eq!((order.node, remaining), (node(&w, "village"), 6.0));

    // A perishable good held in two lots is checked by the takes settlement will make, not by
    // subtracting from the lot sum. The mill holds 0.1 bread with two ticks left and 0.2 with
    // three, 0.30000000000000004 in all. Subtraction would leave 0.30000000000000004 − 0.1 =
    // 0.20000000000000004 after a sell of 0.1, but taking 0.1 removes the first lot and leaves
    // 0.2, so settlement would fall short on the second sell.
    let text = edit(r#"("bread", 9.0)"#, r#"("bread", 0.1)"#);
    let (w, mut s) = load_text(&text);
    let (mill, bread) = (holder(&w, "mill"), good(&w, "bread"));
    apply_in(
        &mut s,
        &w,
        Phase::Upkeep,
        &[StateDelta::Age { holder: mill }],
    )
    .unwrap();
    let mint = StateDelta::Mint {
        to: mill,
        good: bread,
        qty: 0.2,
        prov: Provenance::Production,
    };
    apply_in(&mut s, &w, Phase::Production, &[mint]).unwrap();
    let lots: Vec<(f64, Option<u32>)> = s
        .holding(mill)
        .unwrap()
        .lots(bread)
        .iter()
        .map(|l| (l.qty, l.life))
        .collect();
    assert_eq!(lots, vec![(0.1, Some(2)), (0.2, Some(3))]);
    let rem = held(&s, mill, bread) - 0.1;
    assert!(rem > 0.2, "subtraction leaves {rem:e}");
    let mut after = s.holding(mill).unwrap().clone();
    after.take(bread, Amount::Qty(0.1)).unwrap();
    assert!(
        after.take(bread, Amount::Qty(rem)).is_err(),
        "settlement would fall short"
    );
    let err = admit(
        vec![
            sell(&w, "mill", "port", "bread", 0.1),
            sell(&w, "mill", "town", "bread", rem),
        ],
        &s,
        &w,
    )
    .unwrap_err();
    assert!(
        matches!(err, OrderError::OverPosted { order, remaining } if order.node == node(&w, "town") && remaining == 0.2),
        "{err:?}"
    );
    // What the lots hold is admitted, and settles with no shortfall: the workers buy at the
    // port in florin and the pensioners in town in coin, each everything offered.
    let orders = vec![
        sell(&w, "mill", "port", "bread", 0.1),
        sell(&w, "mill", "town", "bread", 0.2),
        buy(&w, "workers", "port", "bread", 0.1, 10.0),
        buy(&w, "pensioners", "town", "bread", 0.2, 6.0),
    ];
    let t = market_tick(&mut s, &w, orders).expect("the tick settles");
    assert_eq!(held(&s, mill, bread), 0.0);
    assert_eq!(held(&s, holder(&w, "workers"), bread), 0.1);
    assert_eq!(held(&s, holder(&w, "pensioners"), bread), 0.2);
    assert!(t.audit.max_margin <= 1.0);
}

#[test]
fn duplicate_order_is_rejected() {
    let (w, s) = load();
    let err = admit(
        vec![
            buy(&w, "workers", "town", "bread", 1.0, 2.0),
            buy(&w, "workers", "town", "bread", 2.0, 4.0),
        ],
        &s,
        &w,
    )
    .unwrap_err();
    assert_eq!(
        err,
        OrderError::Duplicate {
            actor: actor(&w, "workers"),
            node: node(&w, "town"),
            good: good(&w, "bread"),
            side: SideTag::Buy,
        }
    );
    // A zero-quantity duplicate is still a second post.
    assert!(matches!(
        admit(
            vec![
                sell(&w, "farm", "town", "grain", 1.0),
                sell(&w, "farm", "town", "grain", 0.0),
            ],
            &s,
            &w
        ),
        Err(OrderError::Duplicate { .. })
    ));
    // A buy and a sell in one market are two keys.
    let lines = admit(
        vec![
            sell(&w, "mill", "town", "bread", 1.0),
            buy(&w, "mill", "town", "bread", 1.0, 2.0),
        ],
        &s,
        &w,
    )
    .unwrap();
    assert_eq!(lines.len(), 2);
}

#[test]
fn orders_are_checked_against_the_world() {
    let (w, s) = load();
    let base = buy(&w, "workers", "town", "bread", 1.0, 2.0);
    let refused = |o: Order| admit(vec![o], &s, &w).unwrap_err();
    // The class is the actor's registered class.
    let wrong = Order {
        class: class(&w, "producers"),
        ..base
    };
    assert!(matches!(refused(wrong), OrderError::WrongClass { .. }));
    // A currency has no market, at its own node or any other.
    for at in ["town", "port"] {
        let o = Order {
            node: node(&w, at),
            good: good(&w, "coin"),
            ..base
        };
        assert!(matches!(refused(o), OrderError::NoMarket { .. }));
    }
    // Ids outside the world, in every build profile (N2).
    let ghost = ActorId::Desk(DeskId(9));
    assert_eq!(
        refused(Order {
            actor: ghost,
            ..base
        }),
        OrderError::UnknownActor(ghost)
    );
    assert_eq!(
        refused(Order {
            node: NodeId(7),
            ..base
        }),
        OrderError::UnknownNode(NodeId(7))
    );
    assert_eq!(
        refused(Order {
            good: GoodId(7),
            ..base
        }),
        OrderError::UnknownGood(GoodId(7))
    );
    // Values are finite with a clear sign bit.
    for bad in [f64::NAN, f64::INFINITY, -1.0, -0.0] {
        let q = Order { qty: bad, ..base };
        assert!(matches!(
            refused(q),
            OrderError::BadValue { what: "qty", .. }
        ));
        let b = Order {
            side: Side::Buy { budget: bad },
            ..base
        };
        assert!(matches!(
            refused(b),
            OrderError::BadValue { what: "budget", .. }
        ));
    }
}

#[test]
fn budgets_bind_per_currency() {
    let (w, s) = load();
    // The workers hold 20 coin and 10 florin: 20 coin at town and 10 florin at the port fit.
    let lines = admit(
        vec![
            buy(&w, "workers", "town", "bread", 1.0, 20.0),
            buy(&w, "workers", "port", "bread", 1.0, 10.0),
        ],
        &s,
        &w,
    )
    .unwrap();
    assert_eq!(lines.len(), 2);
    // 11 florin does not, whatever coin is left.
    let err = admit(
        vec![
            buy(&w, "workers", "town", "bread", 1.0, 1.0),
            buy(&w, "workers", "port", "bread", 1.0, 11.0),
        ],
        &s,
        &w,
    )
    .unwrap_err();
    assert!(
        matches!(err, OrderError::OverBudget { currency, remaining, .. } if currency == good(&w, "florin") && remaining == 10.0),
        "{err:?}"
    );
}

#[test]
fn feasible_quantity_is_what_the_budget_buys() {
    let (w, mut s) = load();
    let (town, bread) = (node(&w, "town"), good(&w, "bread"));
    // At price 2, a budget of 6 buys 3 of the 10 asked; a budget of 0 buys nothing, and its
    // line stays, since what it asked for is rationing's to record.
    let lines = admit(
        vec![
            buy(&w, "pensioners", "town", "bread", 10.0, 6.0),
            buy(&w, "workers", "town", "bread", 4.0, 0.0),
        ],
        &s,
        &w,
    )
    .unwrap();
    let feasible: Vec<f64> = lines.iter().map(|l| l.feasible).collect();
    assert_eq!(feasible, vec![3.0, 0.0]);
    // No price guard (N6): when budget/price is infinite, the quantity binds.
    set_price(&mut s, &w, town, bread, 1e-310);
    let lines = admit(vec![buy(&w, "workers", "town", "bread", 4.0, 20.0)], &s, &w).unwrap();
    assert_eq!(num::max_qty(20.0, 1e-310), Ok(f64::INFINITY));
    assert_eq!(lines[0].feasible, 4.0);
    // Zero-quantity orders are dropped and bind no cash: the 6 coin stay for the second buy.
    let lines = admit(
        vec![
            buy(&w, "pensioners", "town", "bread", 0.0, 6.0),
            buy(&w, "pensioners", "village", "grain", 1.0, 6.0),
        ],
        &s,
        &w,
    )
    .unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].order.node, node(&w, "village"));
}
