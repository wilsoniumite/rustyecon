//! The gate world's run (docs/ENGINE.md §10 and §11, engine): conservation every tick, events in
//! date order, ids kept apart, rationing recorded by class, lots bounded, and the world's own bar.
//!
//! One bar here is relative, `REL` in `gate_ids_apart`: a holding's measured change
//! (`after − before`) and the signed sum of the quantities the trace moved round differently, each
//! operation within 2⁻⁵³ of the magnitudes involved, with a few dozen operations per holder and
//! tick. 1e-12 of those magnitudes is ample for that, and a payment into the wrong holder is off
//! by the whole payment. Everything else is compared exactly.

mod common;

use common::*;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_core::{apply, resolve, state_hash, Ledger, Life};
use std::collections::BTreeMap;

/// The relative bar of `gate_ids_apart` (see the module comment).
const REL: f64 = 1e-12;

#[test]
fn gate_conserves_every_tick() {
    // R2: every tick's ledger closes with every good's drift within its registered relative
    // tolerance, and no tick stops on a shortfall (any would be a RunError).
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let (grain, fuel, bread) = (good(&w, "grain"), good(&w, "fuel"), good(&w, "bread"));
    let mut spoiled = 0.0;
    for _ in 0..TICKS {
        let r = sim.step().expect("no tick stops");
        assert!(r.audit.max_margin <= 1.0, "tick {}: {:?}", r.tick, r.audit);
        assert!(r.audit.max_drift.is_finite());
        // The ledger sees the flows: the endowments, the mill's recipe and the pops' meals.
        assert!(audit_line(&r, grain, Provenance::Endowment) > 0.0);
        assert!(audit_line(&r, fuel, Provenance::Endowment) > 0.0);
        assert!(audit_line(&r, bread, Provenance::Production) > 0.0);
        assert!(audit_line(&r, bread, Provenance::Consumption) < 0.0);
        spoiled += audit_line(&r, bread, Provenance::Spoilage);
    }
    assert!(
        spoiled < 0.0,
        "unsold bread spoils, with its own ledger line"
    );
}

#[test]
fn gate_events_fire_in_date_order() {
    // N1: the four dated events are listed out of order on the tape; they fire once each, at
    // the tick their date falls in, in date order. The yearly pension fires every 52 ticks.
    let t = tape();
    let listed: Vec<&str> = t.events.iter().map(|e| e.key.as_str()).collect();
    let dated = [
        ("mine.cut", "1760-03-01"),
        ("bread.line.up", "1765-09-01"),
        ("oven.opens", "1768-04-01"),
        ("mine.restored", "1770-06-15"),
    ];
    let in_date_order: Vec<&str> = dated.iter().map(|(k, _)| *k).collect();
    assert_ne!(listed, in_date_order, "the tape lists them out of order");
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let mut fired: Vec<(u64, String, u32)> = Vec::new();
    sim.run_until(TICKS, &mut |r| {
        for e in &r.events {
            fired.push((r.tick, e.key.to_string(), e.occurrence));
        }
    })
    .unwrap();
    let once: Vec<(u64, &str)> = fired
        .iter()
        .filter(|(_, k, _)| k != "pension")
        .map(|(t, k, _)| (*t, k.as_str()))
        .collect();
    let expected: Vec<(u64, &str)> = dated.iter().map(|(k, d)| (tick_of(&w, d), *k)).collect();
    assert_eq!(once, expected);
    let pensions: Vec<(u64, u32)> = fired
        .iter()
        .filter(|(_, k, _)| k == "pension")
        .map(|(t, _, o)| (*t, *o))
        .collect();
    let first = tick_of(&w, "1751-01-01");
    let expected: Vec<(u64, u32)> = (0u32..)
        .map(|k| (first + 52 * u64::from(k), k))
        .take_while(|(t, _)| *t < TICKS)
        .collect();
    assert_eq!(pensions, expected);
    assert_eq!(pensions.len(), 40);
    assert!(
        fired.windows(2).all(|p| p[0].0 <= p[1].0),
        "firing order is tick order"
    );
}

#[test]
fn gate_ids_apart() {
    // Defect 10: Desk(0) (farm) and Pop(0) (pensioners) share a number, as do Desk(1) (mill)
    // and Pop(1) (workers). Every tick, each one's holding of every good changes by exactly the
    // signed sum of the traced deltas that name it, and every settlement transfer that names
    // an actor matches one of that actor's own settle lines.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let pairs = [
        (
            ActorId::Desk(DeskId(0)),
            "farm",
            ActorId::Pop(PopId(0)),
            "pensioners",
        ),
        (
            ActorId::Desk(DeskId(1)),
            "mill",
            ActorId::Pop(PopId(1)),
            "workers",
        ),
    ];
    let mut watched = Vec::new();
    for (desk, dk, pop, pk) in pairs {
        assert_eq!(actor(&w, dk), desk);
        assert_eq!(actor(&w, pk), pop);
        watched.push(Holder::Actor(desk));
        watched.push(Holder::Actor(pop));
    }
    let (_, mut shadow) = resolve(&t).unwrap();
    let goods: Vec<GoodId> = w.goods.iter().map(|g| g.id).collect();
    let held = |sim: &Sim, h: Holder, g: GoodId| sim.holding(h).map_or(0.0, |i| i.get(g));
    for _ in 0..TICKS {
        let before: Vec<f64> = watched
            .iter()
            .flat_map(|&h| goods.iter().map(move |&g| (h, g)))
            .map(|(h, g)| held(&sim, h, g))
            .collect();
        let (report, trace) = sim.step_traced().unwrap();
        // Signed sums and gross quantities per (holder, good), from the trace alone; ageing is
        // read off the shadow's lots before it applies.
        let mut net: BTreeMap<(Holder, GoodId), (f64, f64)> = BTreeMap::new();
        let mut post = |h: Holder, g: GoodId, q: f64| {
            let e = net.entry((h, g)).or_insert((0.0, 0.0));
            e.0 += q;
            e.1 += q.abs();
        };
        let mut l = Ledger::open(&shadow, &w).unwrap();
        for e in &trace.0 {
            match e.delta {
                StateDelta::Transfer { from, to, good, .. } => {
                    post(from, good, -e.moved);
                    post(to, good, e.moved);
                }
                StateDelta::Mint { to, good, .. } => post(to, good, e.moved),
                StateDelta::Burn { from, good, .. } => post(from, good, -e.moved),
                StateDelta::Age { holder } => {
                    if let Some(inv) = shadow.holding(holder) {
                        for &g in &goods {
                            let dying: f64 = inv
                                .lots(g)
                                .iter()
                                .filter(|lot| lot.life == Some(0))
                                .map(|lot| lot.qty)
                                .sum();
                            post(holder, g, -dying);
                        }
                    }
                }
                _ => {}
            }
            apply(
                &mut shadow,
                &w,
                e.phase,
                std::slice::from_ref(&e.delta),
                &mut l,
            )
            .unwrap();
        }
        assert_eq!(state_hash(&shadow), report.hash);
        let mut i = 0;
        for &h in &watched {
            for &g in &goods {
                let after = held(&sim, h, g);
                let (sum, gross) = net.get(&(h, g)).copied().unwrap_or((0.0, 0.0));
                let change = after - before[i];
                let bar = REL * (before[i].abs() + after.abs() + gross);
                assert!(
                    (change - sum).abs() <= bar,
                    "tick {}: {h} {g} changed by {change:e}, its deltas by {sum:e}",
                    report.tick
                );
                i += 1;
            }
        }
        // Every settlement transfer naming an actor is one of that actor's settle lines.
        for e in trace.0.iter().filter(|e| e.phase == Phase::Settlement) {
            let StateDelta::Transfer { from, to, good, .. } = e.delta else {
                panic!("settlement applies transfers only");
            };
            let (a, market, paying_in) = match (from, to) {
                (Holder::Actor(a), Holder::Escrow(n, g)) => (a, (n, g), true),
                (Holder::Escrow(n, g), Holder::Actor(a)) => (a, (n, g), false),
                _ => panic!("settlement moves between an actor and an escrow"),
            };
            let line = report
                .settlements
                .iter()
                .find(|l| {
                    l.actor == a
                        && (l.node, l.good) == market
                        && match (l.side, good == market.1, paying_in) {
                            // A seller ships the good in and is paid currency out.
                            (SideTag::Sell, true, true) => l.qty == e.moved,
                            (SideTag::Sell, false, false) => l.value == e.moved,
                            // A buyer pays currency in and receives the good out.
                            (SideTag::Buy, false, true) => l.value == e.moved,
                            (SideTag::Buy, true, false) => l.qty == e.moved,
                            _ => false,
                        }
                })
                .is_some();
            assert!(
                line,
                "tick {}: {e:?} matches no settle line of {a}",
                report.tick
            );
        }
    }
}

#[test]
fn gate_rations_and_records_by_class() {
    // R12 and N7: bread rations at tick 0 and again after the 1760 cut; the pensioners ask for
    // more bread than their budget pays for, every tick, at both nodes; and at every rationed
    // tick the recorded demand is the sum of the feasible quantities, below what was requested.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let bread = good(&w, "bread");
    let pensioners = class(&w, "pensioners");
    let cut = tick_of(&w, "1760-03-01");
    let all = reports(&mut sim, TICKS);
    let town_bread = |r: &TickReport| {
        *r.markets
            .iter()
            .find(|m| m.node == node(&w, "town") && m.good == bread)
            .unwrap()
    };
    // Rationing at tick 0: the genesis bread is far short of what the buyers can pay for.
    let first = town_bread(&all[0]);
    assert!(first.buyer_fill < 1.0 && first.demand > first.supply);
    // The cut halves the mill's fuel and so, once its fuel stock runs down, its bread: the
    // bread shortfall, summed over both nodes, of the year after the cut is more than twice
    // that of the year before it, and the worst fill after it is far below the worst before.
    let window = |range: std::ops::Range<u64>| -> (f64, f64) {
        let mut short = 0.0;
        let mut worst: f64 = 1.0;
        for tick in range {
            for m in all[tick as usize]
                .markets
                .iter()
                .filter(|m| m.good == bread)
            {
                short += (m.demand - m.supply).max(0.0);
                worst = worst.min(m.buyer_fill);
            }
        }
        (short, worst)
    };
    let (before, worst_before) = window(cut - 52..cut);
    let (after, worst_after) = window(cut..cut + 52);
    assert!(
        after > 2.0 * before,
        "bread shortfall {before} before the cut, {after} after"
    );
    assert!(
        worst_after < 0.5 * worst_before,
        "worst bread fill {worst_before} before the cut, {worst_after} after"
    );
    let mut rationed = 0;
    for r in &all {
        for m in r.markets.iter().filter(|m| m.good == bread) {
            let buyers: Vec<&RationLine> = r
                .rationing
                .iter()
                .filter(|l| (l.node, l.good, l.side) == (m.node, m.good, SideTag::Buy))
                .collect();
            let p = buyers
                .iter()
                .find(|l| l.class == pensioners)
                .expect("the pensioners buy bread at both nodes every tick");
            assert!(
                p.requested > p.feasible,
                "tick {}: the pensioners are cash-short",
                r.tick
            );
            if m.buyer_fill < 1.0 {
                rationed += 1;
                let feasible = buyers.iter().fold(0.0, |acc, l| acc + l.feasible);
                let requested = buyers.iter().fold(0.0, |acc, l| acc + l.requested);
                assert_eq!(
                    m.demand, feasible,
                    "tick {}: D is feasible demand (N7)",
                    r.tick
                );
                assert!(feasible < requested);
            }
        }
    }
    assert!(
        rationed > 100,
        "bread rations often: {rationed} market-ticks"
    );
}

#[test]
fn gate_lots_bounded() {
    // N9: lots of equal life coalesce, so over 20,000 ticks (385 years) a good holds at most
    // life + 1 lots in any holding, an indefinite good one, and a currency exactly one.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let bound: Vec<usize> = w
        .goods
        .iter()
        .map(|g| match g.life {
            Life::Indefinite => 1,
            Life::Instant => 1,
            Life::Ticks(l) => l as usize + 1,
        })
        .collect();
    let bread = good(&w, "bread");
    let mut most = 0;
    for _ in 0..20_000 {
        sim.step().expect("20,000 ticks run");
        for (h, inv) in w.actors.iter().map(|a| {
            let h = Holder::Actor(a.id);
            (h, sim.holding(h).expect("every actor holds an inventory"))
        }) {
            for g in &w.goods {
                let n = inv.lots(g.id).len();
                assert!(n <= bound[g.id.idx()], "{h} holds {n} lots of {}", g.key);
                if w.is_currency(g.id) {
                    assert_eq!(n, 1, "{h} holds its currency in one lot");
                }
                if g.id == bread {
                    most = most.max(n);
                }
            }
        }
    }
    assert!(most > 1, "bread is held in several lots at once");
}

#[test]
fn gate_world_meets_its_bar() {
    // §10's tuning bar: every market trades in every year, and no price leaves [1e-3, 1e3]
    // times its genesis value.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let genesis: Vec<f64> = w.markets().map(|(n, g)| sim.price(n, g).unwrap()).collect();
    let mut traded: BTreeMap<i32, Vec<bool>> = BTreeMap::new();
    for r in reports(&mut sim, TICKS) {
        let year = traded
            .entry(r.date.y)
            .or_insert_with(|| vec![false; genesis.len()]);
        for (i, m) in r.markets.iter().enumerate() {
            year[i] |= m.cleared > 0.0;
            for p in [m.price, m.next_price] {
                let ratio = p / genesis[i];
                assert!((1e-3..=1e3).contains(&ratio), "tick {}: {m:?}", r.tick);
            }
        }
    }
    assert_eq!(traded.len(), 40);
    for (year, markets) in traded {
        assert!(markets.iter().all(|&t| t), "{year}: a market did not trade");
    }
}
