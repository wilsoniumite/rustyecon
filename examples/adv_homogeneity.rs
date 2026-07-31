//! ADVERSARIAL PROBE (review session, 2026-07-31). How exact is "exactly 0.000"?
//!
//! `examples/elasticity_probe.rs` prints the uniform-bump control to three
//! decimal places and the header of `src/kernel/mod.rs` reports it as "exactly
//! 0.000". The band is `b_out·flow·cost/revenue`, and `(k·cost)/(k·revenue)` is
//! **not** bit-identical to `cost/revenue` in binary floating point. So the
//! honest claim is degree-0 to within rounding, and this measures which.
//!
//!     cargo run --release --example adv_homogeneity -- <scenario> <ticks> <rule> [k]

use rustyecon::{
    kernel::AgentArm,
    scenario::loader,
    state::game_data::SupplyRule,
    systems::{clearing::PriceRule, decisions, run_tick},
    types::{ids::*, order::OrderSide},
};

fn supplies(
    state: &rustyecon::state::SimState,
    gd: &rustyecon::state::GameData,
) -> Vec<f64> {
    let mut v = vec![0.0; gd.num_nodes() * gd.num_goods()];
    let (_, orders) = decisions::run(state, gd, AgentArm::Kernel);
    for o in &orders {
        if matches!(o.side, OrderSide::Sell) {
            v[o.node.idx() * gd.num_goods() + o.good.idx()] += o.qty;
        }
    }
    v
}

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    let ticks: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(20);
    let rule: SupplyRule = a
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(SupplyRule::Reservation);
    let k: f64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(1.01);

    let s0 = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, mut gd, events) = (s0.state, s0.game_data, s0.events);
    for _ in 0..ticks {
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, PriceRule::Imbalance);
    }
    gd.kernel.supply_rule = rule;

    let base = supplies(&state, &gd);
    let mut scaled = state.clone();
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            scaled.set_price(node, good, state.price(node, good) * k);
        }
    }
    let after = supplies(&scaled, &gd);

    let (mut exact, mut moved, mut worst_ulp, mut worst_eps) = (0, 0, 0i64, 0.0f64);
    let dlnk = k.ln();
    for i in 0..base.len() {
        if base[i] <= 0.0 && after[i] <= 0.0 {
            continue;
        }
        if base[i].to_bits() == after[i].to_bits() {
            exact += 1;
        } else {
            moved += 1;
            let ulps = (after[i].to_bits() as i64 - base[i].to_bits() as i64).abs();
            worst_ulp = worst_ulp.max(ulps);
            if base[i] > 0.0 && after[i] > 0.0 {
                worst_eps = worst_eps.max(((after[i] / base[i]).ln() / dlnk).abs());
            }
            println!(
                "  moved: node{} good{} {:.17e} -> {:.17e}  ({ulps} ulp)",
                i / gd.num_goods(),
                i % gd.num_goods(),
                base[i],
                after[i]
            );
        }
    }
    println!(
        "{dir} t={ticks} rule={} k={k}: {exact} markets bit-identical, {moved} moved, worst {worst_ulp} ulp, worst |uniform eps_s| = {worst_eps:.3e}",
        rule.name()
    );
}
