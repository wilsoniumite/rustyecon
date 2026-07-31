//! ADVERSARIAL, part 2: what B8's samples are made of on a REAL run.
//!
//! B8's `observe` gate is `supply > 0 || demand > 0` on the SCORED good. It
//! never asks (a) whether that market cleared anything, or (b) whether the
//! numeraire — the divisor in every single reading — traded at all. This runs a
//! scenario under the certificate's own window and counts.
//!
//!     cargo run --release --example b8_wild -- <dir> <ticks> <legacy|kernel> [imbalance|ratio]

use rustyecon::kernel::AgentArm;
use rustyecon::scenario::loader;
use rustyecon::systems::{clearing::PriceRule, run_tick};
use rustyecon::certify::technology::labour_values;
use rustyecon::types::ids::GoodId;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let dir = a[0].clone();
    let ticks: u64 = a[1].parse().unwrap();
    let arm = if a[2] == "kernel" { AgentArm::Kernel } else { AgentArm::Legacy };
    let rule = if a.get(3).map(|s| s.as_str()) == Some("ratio") {
        PriceRule::Ratio
    } else {
        PriceRule::Imbalance
    };

    let mut sc = loader::load(Path::new(&dir)).expect("load");
    let cr = sc.criteria.clone().expect("criteria");
    let gd = sc.game_data;
    let labour = gd.goods.iter().find(|g| g.name == cr.labour_good).map(|g| g.id).expect("labour");
    let lv = labour_values(&gd, labour);

    // Per (region, good): total B8 samples, samples with a DEAD numeraire,
    // samples where the scored market cleared exactly nothing.
    let ng = gd.num_goods();
    let nr = gd.regions.len();
    let mut total = vec![0u64; nr * ng];
    let mut dead_num = vec![0u64; nr * ng];
    let mut no_clear = vec![0u64; nr * ng];
    // Two log-space means: B8's, and one restricted to samples where the
    // numeraire market was live AND the scored market actually cleared.
    let mut sum_all = vec![0.0f64; nr * ng];
    let mut sq_all = vec![0.0f64; nr * ng];
    let mut sum_clean = vec![0.0f64; nr * ng];
    let mut n_clean = vec![0u64; nr * ng];

    for _ in 0..ticks {
        run_tick(&mut sc.state, &gd, &sc.events, arm, rule);
        if !cr.in_analysis(sc.state.tick) {
            continue;
        }
        for (ri, r) in gd.regions.iter().enumerate() {
            let node = r.market_node;
            let pl = sc.state.price(node, labour);
            let num_live =
                sc.state.supply(node, labour) > 0.0 && sc.state.demand(node, labour) > 0.0;
            for gi in 0..ng {
                let good = GoodId(gi as u32);
                if good == labour {
                    continue;
                }
                let Some(implied) = lv.get(good).relative_price() else { continue };
                let s = sc.state.supply(node, good);
                let d = sc.state.demand(node, good);
                if s <= 0.0 && d <= 0.0 {
                    continue;
                }
                let realised = sc.state.price(node, good) / pl;
                if !(realised.is_finite() && realised > 0.0) {
                    continue;
                }
                let x = (realised / implied).ln();
                if !x.is_finite() {
                    continue;
                }
                let i = ri * ng + gi;
                total[i] += 1;
                sum_all[i] += x;
                sq_all[i] += x * x;
                if !num_live {
                    dead_num[i] += 1;
                }
                let cleared = s.min(d);
                if cleared <= 0.0 {
                    no_clear[i] += 1;
                }
                if num_live && cleared > 0.0 {
                    n_clean[i] += 1;
                    sum_clean[i] += x;
                }
            }
        }
    }

    println!(
        "{dir} {:?} {:?}  window {}..{}",
        arm, rule, cr.transient, cr.analysis_end
    );
    println!(
        "{:<12} {:<10} {:>6} {:>10} {:>10} {:>12} {:>12} {:>7}",
        "region", "good", "n", "deadNumer", "nothingClrd", "B8 gap x", "clean gap x", "nClean"
    );
    for (ri, r) in gd.regions.iter().enumerate() {
        for gi in 0..ng {
            let i = ri * ng + gi;
            if total[i] == 0 {
                continue;
            }
            let b8 = (sum_all[i] / total[i] as f64).exp();
            let clean = if n_clean[i] > 0 {
                format!("{:.4e}", (sum_clean[i] / n_clean[i] as f64).exp())
            } else {
                "—".into()
            };
            println!(
                "{:<12} {:<10} {:>6} {:>10} {:>10} {:>12.4e} {:>12} {:>7}",
                r.name, gd.goods[gi].name, total[i], dead_num[i], no_clear[i], b8, clean, n_clean[i]
            );
        }
    }

    // Machine-readable: one line per (region, good) with the log-space sd, so a
    // "reads right on average, never right on any tick" market can be found.
    for (ri, r) in gd.regions.iter().enumerate() {
        for gi in 0..ng {
            let i = ri * ng + gi;
            if total[i] < 2 { continue; }
            let n = total[i] as f64;
            let m = sum_all[i] / n;
            let sd = ((sq_all[i] / n - m * m).max(0.0)).sqrt();
            println!("SDLINE	{dir}	{}	{}	{m:.6}	{sd:.6}	{}", r.name, gd.goods[gi].name, total[i]);
        }
    }

    // ── Is the per-good list N findings, or one shared divisor plus N-1? ──
    //
    // Every reading divides by the SAME p_labour. If the wage alone is wrong by
    // a factor K, all N goods read K times too dear and B8 prints N separate
    // "too dear" lines for one error. Split each region's readings into the
    // geometric-mean common factor (which is entirely a statement about the
    // numeraire) and the residual (which is the only part that says anything
    // about the relative prices BETWEEN goods).
    println!("\nDECOMPOSITION — common wage factor vs residual relative-price error");
    for (ri, r) in gd.regions.iter().enumerate() {
        let mut logs: Vec<(String, f64)> = Vec::new();
        for gi in 0..ng {
            let i = ri * ng + gi;
            if total[i] > 0 {
                logs.push((gd.goods[gi].name.clone(), sum_all[i] / total[i] as f64));
            }
        }
        if logs.is_empty() {
            continue;
        }
        let common = logs.iter().map(|(_, x)| x).sum::<f64>() / logs.len() as f64;
        print!("  {:<12} common {:.4e}x |", r.name, common.exp());
        for (n, x) in &logs {
            print!(" {n} {:.4}x", (x - common).exp());
        }
        println!();
    }
}
