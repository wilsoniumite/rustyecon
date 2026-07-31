//! Is `supply > 0 || demand > 0` ever true while the traded quantity
//! `min(supply, demand)` is ZERO, on a real run of the shipped corpus?
//!
//! That is the exact condition under which B8's numeraire gate admits a tick
//! whose wage nobody paid. Review artefact.
//!
//!     cargo run --release --example b8_gate_census -- <scenario-dir> ...

use rustyecon::kernel::AgentArm;
use rustyecon::scenario::loader;
use rustyecon::systems::{clearing::PriceRule, run_tick};
use std::path::Path;

fn main() {
    let dirs: Vec<String> = std::env::args().skip(1).collect();
    for dir in &dirs {
        for arm in [AgentArm::Legacy, AgentArm::Kernel] {
            let mut s = loader::load(Path::new(dir)).expect(dir);
            let Some(cr) = s.criteria.clone() else { continue };
            let gd = s.game_data.clone();
            let Some(labour) = gd.goods.iter().find(|g| g.name == cr.labour_good).map(|g| g.id)
            else {
                println!("{dir} [{arm:?}]: no labour good");
                continue;
            };

            let n = gd.regions.len();
            let mut gate_pass = vec![0u64; n];
            let mut gate_pass_no_trade = vec![0u64; n];
            let mut in_window = 0u64;
            for _ in 0..1000 {
                run_tick(&mut s.state, &gd, &s.events, arm, PriceRule::Imbalance);
                if !cr.in_analysis(s.state.tick) {
                    continue;
                }
                in_window += 1;
                for (ri, reg) in gd.regions.iter().enumerate() {
                    let sup = s.state.supply(reg.market_node, labour);
                    let dem = s.state.demand(reg.market_node, labour);
                    if sup > 0.0 || dem > 0.0 {
                        gate_pass[ri] += 1;
                        if sup.min(dem) <= 0.0 {
                            gate_pass_no_trade[ri] += 1;
                        }
                    }
                }
            }
            println!("{dir} [{arm:?}] window = {in_window} ticks");
            for (ri, reg) in gd.regions.iter().enumerate() {
                println!(
                    "   {:<12} gate ADMITTED {:>5} ticks; of those {:>5} traded ZERO labour \
                     ({:.1}%)",
                    reg.name,
                    gate_pass[ri],
                    gate_pass_no_trade[ri],
                    100.0 * gate_pass_no_trade[ri] as f64 / gate_pass[ri].max(1) as f64
                );
            }
        }
    }
}
