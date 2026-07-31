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
//! Everything before Phase 4 was about (2). This measures (1), by the same
//! differential trick `certify::invariants` uses for B6: perturb the posted
//! prices on a clone, re-run the decision phase, and see what moves.
//!
//! ## The probe was blind, and this is the fix
//!
//! **The first version of this file bumped every good at every node by 1%
//! simultaneously**, and that reports `ε_s = 0.000` for a *correct*
//! price-responsive rule as loudly as for a broken one. The reservation rule's
//! posted quantity depends on prices only through the markup `R`, which is
//! homogeneous of degree 0 — a uniform inflation leaves every posted quantity
//! identically unchanged. That is a *requirement* (a supply that responded to
//! uniform inflation would be money illusion and would break R12), not a defect,
//! and it means the all-goods bump cannot distinguish the two rules at all.
//!
//! Someone could have implemented the fix, run the old probe, read 0.000, and
//! concluded it had failed — or, worse, run it against the old rule, read 0.000,
//! and concluded there was nothing to fix. So the probe now does both:
//!
//! * **own-price**: bump **one** `(node, good)` at a time and re-run the
//!   decision phase. This is the number the crossing question needs.
//! * **uniform**: the old all-goods bump, kept as a *control*. A degree-0
//!   posting rule must read exactly 0.000 here, and reading anything else is
//!   money illusion, not elasticity.
//!
//! ## Why the running rule and the posting rule are separate arguments
//!
//! An arm that kills its own markets posts nothing, and a probe that ran each
//! rule down its own trajectory would report "no elasticity" for a rule whose
//! elasticity is fine and whose *economy* is dead — two different findings
//! wearing one number. So the state may be evolved under one rule and the
//! posting function evaluated under another, which measures the posting function
//! on a common state. Run it both ways: they answer different questions.
//!
//!     cargo run --release --example elasticity_probe -- <scenario> <ticks> <arm> [run_rule] [post_rule]

use rustyecon::{
    kernel::AgentArm,
    scenario::loader,
    state::game_data::SupplyRule,
    systems::{clearing::PriceRule, decisions, run_tick},
    types::{ids::*, order::OrderSide},
};

/// Posted supply and demand per (node, good), from one decision phase.
fn posted(
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
    v
}

/// `d ln q / d ln p`, or NaN where the log does not exist.
///
/// NaN rather than 0 on a market that posted nothing: "did not move" and "was
/// never there" are different findings and the table must not merge them (R5).
fn elasticity(before: f64, after: f64, dlnp: f64) -> f64 {
    if before > 0.0 && after > 0.0 {
        (after / before).ln() / dlnp
    } else if before <= 0.0 && after <= 0.0 {
        f64::NAN
    } else {
        // Entered or left the market entirely under a 1% move — an infinite
        // local elasticity. Reported as such rather than as a large finite one.
        f64::INFINITY * if after > before { 1.0 } else { -1.0 }
    }
}

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    let ticks: u64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(200);
    let arm = match a.next().as_deref() {
        Some("kernel") => AgentArm::Kernel,
        _ => AgentArm::Legacy,
    };
    let run_rule: Option<SupplyRule> = a.next().and_then(|s| s.parse().ok());
    let post_rule: Option<SupplyRule> = a.next().and_then(|s| s.parse().ok());

    let s = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    if let Some(r) = run_rule {
        gd.kernel.supply_rule = r;
    }
    let evolved_under = gd.kernel.supply_rule;
    for _ in 0..ticks {
        run_tick(&mut state, &gd, &events, arm, PriceRule::Imbalance);
    }
    if let Some(r) = post_rule {
        gd.kernel.supply_rule = r;
    }

    const BUMP: f64 = 1.01;
    let dlnp = BUMP.ln();
    let base = posted(&state, &gd, arm);

    // ── Control: the old all-goods bump ──────────────────────────────────────
    let mut uniform = state.clone();
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            uniform.set_price(node, good, state.price(node, good) * BUMP);
        }
    }
    let uniform = posted(&uniform, &gd, arm);

    println!(
        "{dir}  arm={arm:?}  ran under {}, posting measured under {}  after {ticks} ticks",
        evolved_under.name(),
        gd.kernel.supply_rule.name()
    );
    println!("  OWN-PRICE: one (node, good) bumped +1% at a time. UNIFORM: every price bumped at once (degree-0 control).");
    println!(
        "  {:<6} {:<12} {:>12} {:>12} {:>9} {:>9} {:>9} {:>9}   crossing?",
        "node", "good", "supply", "demand", "eps_s", "eps_d", "u_eps_s", "u_eps_d"
    );

    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            let i = n * gd.num_goods() + g;
            let (s0, d0) = base[i];
            if s0 <= 0.0 && d0 <= 0.0 {
                continue;
            }

            // One price moved, everything else held. This is the partial the
            // crossing question is about.
            let mut probe = state.clone();
            probe.set_price(node, good, state.price(node, good) * BUMP);
            let own = posted(&probe, &gd, arm)[i];

            let es = elasticity(s0, own.0, dlnp);
            let ed = elasticity(d0, own.1, dlnp);
            let (us, ud) = (
                elasticity(s0, uniform[i].0, dlnp),
                elasticity(d0, uniform[i].1, dlnp),
            );

            // A crossing needs the two curves to move toward each other: supply
            // weakly up in price, demand weakly down, and not both flat.
            let cross = match (es.is_finite(), ed.is_finite()) {
                (true, true) if (es - ed) > 1e-6 => "yes",
                (true, false) if es > 1e-6 => "yes (supply only)",
                (false, true) if ed < -1e-6 => "yes (demand only)",
                _ => "NO - no price clears this market",
            };
            println!(
                "  {n:<6} {:<12} {s0:>12.4} {d0:>12.4} {es:>9.3} {ed:>9.3} {us:>9.3} {ud:>9.3}   {cross}",
                gd.good(good).name
            );
        }
    }
}
