//! Decompose the `velocity` metric into the things it multiplies together.
//!
//! `velocity = Σ(cleared × price) / regional currency`. Both DRIFTING metrics
//! are scored on a percentile band, and velocity's band stays enormous at every
//! price-adjustment speed under both agent arms — so the question is which
//! factor is moving: the price level, the traded quantity, or the money.
//!
//!     cargo run --release --example velocity_probe -- <scenario> <ticks> <arm>

use rustyecon::{kernel::AgentArm, scenario::loader, systems::run_tick, types::ids::*};

fn band(v: &[f64]) -> f64 {
    let mut s: Vec<f64> = v.iter().copied().filter(|x| x.is_finite() && *x > 0.0).collect();
    if s.len() < 3 {
        return f64::NAN;
    }
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |p: f64| {
        let pos = p / 100.0 * (s.len() - 1) as f64;
        let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
        s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
    };
    at(95.0) / at(5.0)
}

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    let ticks: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(1000);
    let arm = match a.next().as_deref() {
        Some("kernel") => AgentArm::Kernel,
        _ => AgentArm::Legacy,
    };

    let s = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);
    let region = RegionId(0);
    let node = gd.region(region).market_node;
    let currency = gd.market_node(node).currency_good;

    let (mut vel, mut level, mut money, mut turnover, mut qty) =
        (vec![], vec![], vec![], vec![], vec![]);
    let mut dead_ticks = 0usize;

    for t in 1..=ticks {
        run_tick(&mut state, &gd, &events, arm);
        if t < 150 {
            continue;
        }
        // Cleared volume and its value, as certify::metrics computes them.
        let (mut txn, mut units, mut lnsum, mut n) = (0.0, 0.0, 0.0, 0);
        for g in 0..gd.num_goods() {
            let good = GoodId(g as u32);
            if Some(good) == currency {
                continue;
            }
            let cleared = state.supply(node, good).min(state.demand(node, good));
            let p = state.price(node, good);
            txn += cleared * p;
            units += cleared;
            if p > 0.0 {
                lnsum += p.ln();
                n += 1;
            }
        }
        // Regional currency: every inventory belonging to this region's pops and
        // to its recipe instances.
        let mut gbp = 0.0;
        if let Some(c) = currency {
            for p in state.pop_groups.iter().filter(|p| p.region == region) {
                gbp += state.inventory(p.inventory).get(c);
            }
            let mut seen = Vec::new();
            for ri in state.recipe_instances.iter().filter(|r| r.region == region) {
                if !seen.contains(&ri.input_inv) {
                    seen.push(ri.input_inv);
                    gbp += state.inventory(ri.input_inv).get(c);
                }
            }
        }
        let v = if gbp > 0.0 { txn / gbp } else { f64::NAN };
        if v < 0.003 {
            dead_ticks += 1;
        }
        vel.push(v);
        turnover.push(txn);
        qty.push(units);
        money.push(gbp);
        level.push(if n > 0 { (lnsum / n as f64).exp() } else { f64::NAN });
    }

    println!("{dir}  arm={arm:?}  ticks 150..{ticks}");
    println!("  velocity        band {:>12.4e}", band(&vel));
    println!("  price level     band {:>12.4e}   (geometric mean of prices)", band(&level));
    println!("  cleared units   band {:>12.4e}   (real trade)", band(&qty));
    println!("  nominal turnover band {:>11.4e}", band(&turnover));
    println!("  regional money  band {:>12.4e}", band(&money));
    println!(
        "  ticks below the DEAD threshold: {:.1}%   cleared units min {:.3e} max {:.3e}",
        100.0 * dead_ticks as f64 / vel.len() as f64,
        qty.iter().copied().fold(f64::INFINITY, f64::min),
        qty.iter().copied().fold(0.0, f64::max),
    );
}
