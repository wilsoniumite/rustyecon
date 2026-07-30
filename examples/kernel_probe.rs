//! Trace one scenario under the desk kernel, printing what each rule saw.
//!
//! A diagnostic, not a test: when a desk sits at its floor for a thousand ticks
//! the certificate says only that it did, and the question is which of scale,
//! stock, margin or cash was the binding one.
//!
//!     cargo run --release --example kernel_probe -- data/scenarios/lr_00 400 20

use rustyecon::{
    kernel::AgentArm,
    scenario::loader,
    systems::run_tick,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    let ticks: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(400);
    let every: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(20);

    let s = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);

    let names: Vec<String> = gd.goods.iter().map(|g| g.name.clone()).collect();
    let node0 = rustyecon::types::ids::MarketNodeId(0);

    println!("tick | {:<38} | {}", "chosen/size per instance @region0", "price @node0");
    for t in 0..=ticks {
        if t % every == 0 {
            let scales: Vec<String> = state
                .recipe_instances
                .iter()
                .filter(|ri| ri.region.0 == 0)
                .map(|ri| {
                    format!(
                        "{}={:.3}/{:.0}",
                        &gd.recipe(ri.recipe).name[..4.min(gd.recipe(ri.recipe).name.len())],
                        ri.chosen_size,
                        ri.recipe_size
                    )
                })
                .collect();
            let prices: Vec<String> = names
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    format!(
                        "{}={:.4}",
                        &n[..3.min(n.len())],
                        state.price(node0, rustyecon::types::ids::GoodId(i as u32))
                    )
                })
                .collect();
            let stock: Vec<String> = state
                .recipe_instances
                .iter()
                .filter(|ri| ri.region.0 == 0)
                .take(2)
                .map(|ri| {
                    let outs: Vec<String> = gd
                        .recipe(ri.recipe)
                        .outputs
                        .iter()
                        .map(|o| format!("{:.2}", state.inventory(ri.output_inv).get(o.good)))
                        .collect();
                    outs.join("/")
                })
                .collect();
            println!(
                "{t:5} | {:<38} | {} | stock {}",
                scales.join(" "),
                prices.join(" "),
                stock.join(",")
            );
        }
        run_tick(&mut state, &gd, &events, AgentArm::Kernel);
    }
}
