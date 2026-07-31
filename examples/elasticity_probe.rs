//! Measure the price elasticity of what agents actually post.
//!
//! The price rule is a feedback loop: `p` moves on the gap between posted
//! supply and posted demand, and next tick the agents re-post. Two things decide
//! whether that loop settles:
//!
//!   1. **Does an equilibrium price exist at all** — do `s(p)` and `d(p)` cross?
//!      If both sides are perfectly price-inelastic there is no `p` that clears
//!      the market, and the price does not converge to anything because there is
//!      nothing to converge to.
//!   2. **Is the loop gain below 1** — near a crossing the gain is
//!      `|1 + α(ε_d − ε_s)|`, so damping buys stability *given* a crossing.
//!
//! Everything so far has been about (2). This measures (1), by the same
//! differential trick `certify::invariants` uses for B6: perturb the posted
//! prices on a clone, re-run the decision phase, and see what moves.
//!
//!     cargo run --release --example elasticity_probe -- <scenario> <ticks> <arm>

use rustyecon::{
    kernel::AgentArm,
    scenario::loader,
    systems::{clearing::PriceRule, decisions, run_tick},
    types::{ids::*, order::OrderSide},
};

/// Posted supply and demand per (node, good).
fn posted(state: &rustyecon::state::SimState, gd: &rustyecon::state::GameData) -> Vec<(f64, f64)> {
    let mut v = vec![(0.0, 0.0); gd.num_nodes() * gd.num_goods()];
    let (_, orders) = decisions::run(state, gd, AgentArm::Legacy);
    for o in &orders {
        let i = o.node.idx() * gd.num_goods() + o.good.idx();
        match o.side {
            OrderSide::Sell => v[i].0 += o.qty,
            OrderSide::Buy => v[i].1 += o.qty,
        }
    }
    v
}

fn posted_with(
    state: &rustyecon::state::SimState,
    gd: &rustyecon::state::GameData,
    arm: AgentArm,
) -> Vec<(f64, f64)> {
    let mut v = vec![(0.0, 0.0); gd.num_nodes() * gd.num_goods()];
    let (_, orders) = decisions::run(state, gd, arm);
    for o in &orders {
        let i = o.node.idx() * gd.num_goods() + o.good.idx();
        match o.side {
            OrderSide::Sell => v[i].0 += o.qty,
            OrderSide::Buy => v[i].1 += o.qty,
        }
    }
    let _ = posted;
    v
}

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    let ticks: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(200);
    let arm = match a.next().as_deref() {
        Some("kernel") => AgentArm::Kernel,
        _ => AgentArm::Legacy,
    };

    let s = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);
    for _ in 0..ticks {
        run_tick(&mut state, &gd, &events, arm, PriceRule::Imbalance);
    }

    // A 1% price bump, applied to every good at once, then the decision phase
    // re-run against an otherwise identical state.
    const BUMP: f64 = 1.01;
    let base = posted_with(&state, &gd, arm);
    let mut probe = state.clone();
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            probe.set_price(node, good, state.price(node, good) * BUMP);
        }
    }
    let bumped = posted_with(&probe, &gd, arm);

    let dlnp = BUMP.ln();
    println!("{dir}  arm={arm:?}  after {ticks} ticks — elasticity of POSTED quantity to a +1% price move");
    println!("  {:<10} {:>12} {:>12} {:>10} {:>10}   crossing?", "good", "supply", "demand", "eps_s", "eps_d");
    for n in 0..gd.num_nodes().min(1) {
        for g in 0..gd.num_goods() {
            let i = n * gd.num_goods() + g;
            let (s0, d0) = base[i];
            let (s1, d1) = bumped[i];
            if s0 <= 0.0 && d0 <= 0.0 {
                continue;
            }
            let el = |a: f64, b: f64| {
                if a > 0.0 && b > 0.0 { (b / a).ln() / dlnp } else { f64::NAN }
            };
            let (es, ed) = (el(s0, s1), el(d0, d1));
            // A crossing needs the two curves to move toward each other: supply
            // weakly up in price, demand strictly down, and not both flat.
            let cross = match (es.is_finite(), ed.is_finite()) {
                (true, true) if (es - ed) > 1e-6 => "yes",
                _ => "NO — no price clears this market",
            };
            println!(
                "  {:<10} {s0:>12.4} {d0:>12.4} {es:>10.3} {ed:>10.3}   {cross}",
                gd.good(GoodId(g as u32)).name
            );
        }
    }
}
