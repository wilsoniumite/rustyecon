//! ADVERSARIAL PROBE (review session, 2026-07-31). Sweep the posting function
//! over a wide price range instead of taking a single +1% derivative.
//!
//! `examples/elasticity_probe.rs` measures a *local* elasticity at one point. It
//! cannot answer three of the claims made for the reservation rule:
//!
//!   * monotone at ALL prices, including the extremes and the `revenue <= 0`
//!     branch — a local derivative at one price says nothing about a kink or a
//!     reversal three decades away;
//!   * never posts more than the desk holds — checked here per *order*, against
//!     the seller's own inventory, not argued from the formula;
//!   * a genuine CROSSING exists — a positive elasticity is not a crossing. If
//!     `sup_p s(p) < inf_p d(p)` there is still no price that clears, however
//!     elastic supply is.
//!
//! Usage:
//!     cargo run --release --example adv_supply_sweep -- \
//!         <scenario> <ticks> <arm> <run_rule> <post_rule> [decades] [pts_per_decade]

use rustyecon::{
    kernel::AgentArm,
    scenario::loader,
    state::game_data::SupplyRule,
    systems::{clearing::PriceRule, decisions, run_tick},
    types::{ids::*, order::OrderSide},
};

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    let ticks: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let arm = match a.next().as_deref() {
        Some("legacy") => AgentArm::Legacy,
        _ => AgentArm::Kernel,
    };
    let run_rule: Option<SupplyRule> = a.next().and_then(|s| s.parse().ok());
    let post_rule: Option<SupplyRule> = a.next().and_then(|s| s.parse().ok());
    let decades: f64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(8.0);
    let ppd: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(40);

    let s = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    if let Some(r) = run_rule {
        gd.kernel.supply_rule = r;
    }
    let ran_under = gd.kernel.supply_rule;
    for _ in 0..ticks {
        run_tick(&mut state, &gd, &events, arm, PriceRule::Imbalance);
    }
    if let Some(r) = post_rule {
        gd.kernel.supply_rule = r;
    }

    println!(
        "{dir}  arm={arm:?}  ran under {}  posting under {}  after {ticks} ticks",
        ran_under.name(),
        gd.kernel.supply_rule.name()
    );
    println!(
        "sweep: own price x10^[-{decades}, +{decades}], {ppd} pts/decade, plus p = 0 exactly"
    );
    println!(
        "  {:<5} {:<10} {:>12} {:>12} {:>12} {:>12} {:>10} {:>8} {:>10}",
        "node", "good", "s(p_min)", "s(p_0)", "s(p_max)", "sup_s", "inf_d", "cross?", "monotone?"
    );

    let n_pts = (2.0 * decades * ppd as f64) as usize + 1;
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            let p0 = state.price(node, good);
            if !(p0 > 0.0) {
                continue;
            }

            let mut xs: Vec<f64> = Vec::with_capacity(n_pts + 1);
            xs.push(0.0); // the revenue <= 0 branch, exactly
            for i in 0..n_pts {
                let e = -decades + 2.0 * decades * (i as f64) / ((n_pts - 1) as f64);
                xs.push(p0 * 10f64.powf(e));
            }

            let mut sup: Vec<f64> = Vec::with_capacity(xs.len());
            let mut dem: Vec<f64> = Vec::with_capacity(xs.len());
            let mut over: Vec<String> = Vec::new();
            let mut nonfinite = 0usize;

            for (k, &p) in xs.iter().enumerate() {
                let mut probe = state.clone();
                probe.set_price(node, good, p);
                let (_, orders) = decisions::run(&probe, &gd, arm);
                let mut s_tot = 0.0;
                let mut d_tot = 0.0;
                for o in &orders {
                    if o.node != node || o.good != good {
                        continue;
                    }
                    if !o.qty.is_finite() {
                        nonfinite += 1;
                    }
                    match o.side {
                        OrderSide::Sell => {
                            s_tot += o.qty;
                            // Never more than the seller holds — per order,
                            // against that seller's own stock.
                            let held = match o.owner {
                                OwnerId::RecipeInstance(id) => {
                                    let ri = probe.recipe_instance(id);
                                    probe.inventory(ri.output_inv).get(good)
                                }
                                OwnerId::PopGroup(id) => {
                                    let pop = &probe.pop_groups[id.0 as usize];
                                    // A pop's "stock" of labour is its hours.
                                    if pop.is_employed {
                                        pop.size
                                    } else {
                                        pop.size * pop.last_labour_fill_rate
                                    }
                                }
                                _ => f64::INFINITY,
                            };
                            if o.qty > held * (1.0 + 1e-12) + 1e-12 {
                                over.push(format!(
                                    "x{:.3e}: {:?} posts {:.6} holds {:.6}",
                                    p / p0,
                                    o.owner,
                                    o.qty,
                                    held
                                ));
                            }
                        }
                        OrderSide::Buy => d_tot += o.qty,
                    }
                }
                let _ = k;
                sup.push(s_tot);
                dem.push(d_tot);
            }

            // Skip the p = 0 point for the monotonicity scan (it is the left
            // endpoint of the sweep by construction, not part of the grid).
            let mut worst_drop = 0.0f64;
            let mut worst_at = (0.0, 0.0, 0.0);
            for i in 2..sup.len() {
                if sup[i] < sup[i - 1] - 1e-12 {
                    let rel = (sup[i - 1] - sup[i]) / sup[i - 1].max(1e-300);
                    if rel > worst_drop {
                        worst_drop = rel;
                        worst_at = (xs[i - 1] / p0, sup[i - 1], sup[i]);
                    }
                }
            }

            let sup_s = sup[1..].iter().cloned().fold(0.0f64, f64::max);
            let inf_d = dem[1..].iter().cloned().fold(f64::INFINITY, f64::min);
            let s_at_p0 = {
                let mut probe = state.clone();
                probe.set_price(node, good, p0);
                let (_, orders) = decisions::run(&probe, &gd, arm);
                orders
                    .iter()
                    .filter(|o| o.node == node && o.good == good && o.side == OrderSide::Sell)
                    .map(|o| o.qty)
                    .sum::<f64>()
            };
            if sup_s <= 0.0 && dem[1..].iter().all(|&d| d <= 0.0) {
                continue;
            }

            // A CROSSING is a sign change in excess demand z = d - s, not
            // "supply reached demand somewhere". At a high enough price demand
            // is zero and s >= d holds trivially; that is not a market clearing,
            // it is a market with nobody in it.
            let z: Vec<f64> = sup[1..]
                .iter()
                .zip(dem[1..].iter())
                .map(|(s, d)| d - s)
                .collect();
            let pos = z.iter().any(|v| *v > 1e-12);
            let neg = z.iter().any(|v| *v < -1e-12);
            let both_sided = z
                .iter()
                .zip(sup[1..].iter())
                .zip(dem[1..].iter())
                .any(|((v, s), d)| v.abs() <= 1e-9 * s.max(*d).max(1.0) && *s > 0.0 && *d > 0.0);
            let crosses = (pos && neg) || both_sided;

            println!(
                "  {n:<5} {:<10} {:>12.4} {:>12.4} {:>12.4} {:>12.4} {:>10.4} {:>8} {:>10}",
                gd.good(good).name,
                sup[1],
                s_at_p0,
                *sup.last().unwrap(),
                sup_s,
                inf_d,
                if crosses { "yes" } else { "NO" },
                if worst_drop == 0.0 { "yes" } else { "NO" },
            );
            println!(
                "        s(p=0) = {:.6}   z=d-s: min {:+.4e} max {:+.4e}  (z>0 somewhere: {pos}, z<0 somewhere: {neg})",
                sup[0],
                z.iter().cloned().fold(f64::INFINITY, f64::min),
                z.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            );
            if worst_drop > 0.0 {
                println!(
                    "        NON-MONOTONE: worst drop {:.4}% at x{:.3e}: {:.6} -> {:.6}",
                    worst_drop * 100.0,
                    worst_at.0,
                    worst_at.1,
                    worst_at.2
                );
            }
            if !over.is_empty() {
                println!("        OVER-POSTS ({} points):", over.len());
                for line in over.iter().take(5) {
                    println!("          {line}");
                }
            }
            if nonfinite > 0 {
                println!("        NON-FINITE posted quantities at {nonfinite} points");
            }
        }
    }
}
