//! Diagnostic: locate which tick phase unbalances a good.
//!
//! When the conservation ledger panics, it reports *that* a good drifted. This
//! reports *where*: it runs a scenario to the tick before the breach, then steps
//! that tick phase by phase (bypassing the tick-close assert) and prints, per
//! phase, every add and remove of the target good, which inventories they
//! touched, the cleared orders by owner, and each market's buy/sell/price.
//!
//! It found the real cause of the lr_16 breach — a market carrying 1.28e-5 of
//! value in 5.7e-24 of quantity, where an absolute quantity epsilon in the
//! seller settlement skipped the payee entirely.
//!
//! Usage: cargo run --example conservation_probe -- <scenario> <ticks> <good_id>

use rustyecon::{
    scenario::loader,
    state::{apply_state_deltas, GameData, SimState},
    systems::{clearing, decisions, pop_update, price_update, production, transactions, run_tick},
    types::{delta::StateDelta, ids::InventoryId},
};
use std::path::PathBuf;

fn summarise(phase: &str, deltas: &[StateDelta], target: u32) {
    let mut adds = 0.0;
    let mut removes = 0.0;
    let mut detail: Vec<String> = Vec::new();
    for d in deltas {
        match d {
            StateDelta::AddToInventory { inv, good, qty, prov, .. } if good.0 == target => {
                adds += qty;
                detail.push(format!("    +{qty:.6e} -> inv {} ({prov:?})", inv.0));
            }
            StateDelta::RemoveFromInventory { inv, good, qty, prov } if good.0 == target => {
                removes += qty;
                detail.push(format!("    -{qty:.6e} <- inv {} ({prov:?})", inv.0));
            }
            _ => {}
        }
    }
    let net = adds - removes;
    if adds != 0.0 || removes != 0.0 {
        println!("  {phase:<14} adds {adds:.6e}  removes {removes:.6e}  net {net:+.6e}");
        for line in detail.iter().take(12) {
            println!("{line}");
        }
        if detail.len() > 12 {
            println!("    … {} more", detail.len() - 12);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scenario = &args[1];
    let ticks: u64 = args[2].parse().unwrap();
    let target: u32 = args[3].parse().unwrap();

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(scenario);
    let s = loader::load(&dir).expect("scenario loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);

    for _ in 0..ticks {
        run_tick(&mut state, &gd, &events, rustyecon::kernel::AgentArm::Legacy, rustyecon::systems::clearing::PriceRule::Imbalance);
    }
    println!("stepped to tick {} — now probing one tick, good {target}", state.tick);

    println!("-- inventory ownership --");
    for p in &state.pop_groups {
        println!(
            "  inv {:>3}  pop {} (region {}, size {:.4}, employed {}) holds {:.6e}",
            p.inventory.0,
            p.id.0,
            p.region.0,
            p.size,
            p.is_employed,
            state.inventory(p.inventory).get(rustyecon::types::ids::GoodId(target)),
        );
    }
    for b in &state.buildings {
        println!("  inv {:>3}  building {} (region {})", b.inventory.0, b.id.0, b.region.0);
    }

    step_and_report(&mut state, &gd, &events, target);
}

/// Replicates run_tick's phase order without the tick-close assert.
fn step_and_report(
    state: &mut SimState,
    gd: &GameData,
    events: &rustyecon::scenario::EventSchedule,
    target: u32,
) {
    let d = events.deltas_for_tick(state.tick);
    summarise("events", &d, target);
    apply_state_deltas(state, &d);

    let (d, orders) = decisions::run(state, gd, rustyecon::kernel::AgentArm::Legacy);
    summarise("decisions", &d, target);
    apply_state_deltas(state, &d);

    let (d, fills) = clearing::run(state, gd, &orders);
    summarise("clearing", &d, target);
    apply_state_deltas(state, &d);

    // Per-market view of what buyers were charged vs what sellers can be paid.
    use rustyecon::types::order::OrderSide;
    let mut markets: std::collections::BTreeMap<(u32, u32), (f64, f64)> = Default::default();
    for o in &orders {
        let rate = match o.side {
            OrderSide::Buy => fills.buyer_fill(o.node, o.good),
            OrderSide::Sell => fills.seller_fill(o.node, o.good),
        };
        let cleared = o.qty * rate;
        if cleared <= 0.0 {
            continue;
        }
        let e = markets.entry((o.node.0, o.good.0)).or_insert((0.0, 0.0));
        match o.side {
            OrderSide::Buy => e.0 += cleared,
            OrderSide::Sell => e.1 += cleared,
        }
    }
    println!("-- orders by owner (cleared > 0) --");
    for o in &orders {
        let rate = match o.side {
            OrderSide::Buy => fills.buyer_fill(o.node, o.good),
            OrderSide::Sell => fills.seller_fill(o.node, o.good),
        };
        let cleared = o.qty * rate;
        if cleared <= 0.0 {
            continue;
        }
        let price = state.price(o.node, o.good);
        println!(
            "  node {} good {} {:?} owner {:?} cleared {:.6e} value {:.6e}",
            o.node.0,
            o.good.0,
            o.side,
            o.owner,
            cleared,
            cleared * price
        );
    }

    println!("-- markets (node, good): buy_cleared vs sell_cleared, price --");
    for ((n, g), (b, s)) in &markets {
        let price = state.price(
            rustyecon::types::ids::MarketNodeId(*n),
            rustyecon::types::ids::GoodId(*g),
        );
        let flag = if *b > *s + 1e-18 { "  <== buys exceed sells" } else { "" };
        println!("  node {n} good {g}: buy {b:.6e}  sell {s:.6e}  price {price:.6e}{flag}");
    }

    let d = transactions::run(state, gd, &orders, &fills);
    summarise("transactions", &d, target);
    apply_state_deltas(state, &d);

    let d = pop_update::run(state, gd);
    summarise("pop_update", &d, target);
    apply_state_deltas(state, &d);

    let d = production::run(state, gd);
    summarise("production", &d, target);
    apply_state_deltas(state, &d);

    let spoil: Vec<StateDelta> = (0..state.inventories.len())
        .map(|i| StateDelta::SpoilInventory { inv: InventoryId(i as u32) })
        .collect();
    // Spoilage carries no explicit good; report the measured effect instead.
    let before: f64 = state.inventories.iter().map(|i| i.get(rustyecon::types::ids::GoodId(target))).sum();
    apply_state_deltas(state, &spoil);
    let after: f64 = state.inventories.iter().map(|i| i.get(rustyecon::types::ids::GoodId(target))).sum();
    if (after - before).abs() > 0.0 {
        println!("  {:<14} net {:+.6e} (Spoilage)", "spoilage", after - before);
    }

    let d = price_update::run(state, gd, rustyecon::systems::clearing::PriceRule::Imbalance);
    summarise("price_update", &d, target);
    apply_state_deltas(state, &d);
}
