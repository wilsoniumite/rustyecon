//! ADVERSARIAL PROBE (review session, 2026-07-31). MEASURE the price loop's
//! gain instead of arguing it from `alpha * b_out`.
//!
//! The claim under test, from `src/kernel/mod.rs`'s header: at a desk's resting
//! point `eps_s = b_out/R`, so `alpha(eps_s - eps_d) < 2` collapses to
//! `alpha*b_out < 2R`, registered at `0.1 * 2.0 = 0.2` — "a factor of ten inside
//! the boundary". That is an argument about a resting point. The gain the run
//! actually experiences is
//!
//!     G(p) = d ln p_{t+1} / d ln p_t = 1 + alpha * d/dlnp clamp(I(p), +-1)
//!
//! with `I = (d - s)/max(d, s)` recomputed from the posting functions at price
//! `p`, everything else in the state held fixed. That is what is measured here,
//! by central difference, at two places that matter:
//!
//!   * `p0`, the price the run is actually at;
//!   * `p*`, the CROSSING (`z = d - s` changes sign), found by bisection —
//!     the only point at which the linearisation the claim rests on is valid.
//!
//! A market with no crossing gets no gain reading, and says so: the loop-gain
//! question does not arise where there is no fixed point to be stable about.
//!
//! Usage:
//!     cargo run --release --example adv_loop_gain -- \
//!         <scenario> <ticks> <arm> <run_rule> <post_rule>

use rustyecon::{
    kernel::AgentArm,
    scenario::loader,
    state::game_data::SupplyRule,
    systems::{clearing::PriceRule, decisions, run_tick},
    types::{ids::*, order::OrderSide},
};

/// Posted (supply, demand) at one (node, good) with its price set to `p`.
fn sd(
    state: &rustyecon::state::SimState,
    gd: &rustyecon::state::GameData,
    arm: AgentArm,
    node: MarketNodeId,
    good: GoodId,
    p: f64,
) -> (f64, f64) {
    let mut probe = state.clone();
    probe.set_price(node, good, p);
    let (_, orders) = decisions::run(&probe, gd, arm);
    let mut s = 0.0;
    let mut d = 0.0;
    for o in &orders {
        if o.node != node || o.good != good {
            continue;
        }
        match o.side {
            OrderSide::Sell => s += o.qty,
            OrderSide::Buy => d += o.qty,
        }
    }
    (s, d)
}

fn imbalance(s: f64, d: f64) -> f64 {
    if s <= 0.0 && d <= 0.0 {
        return 0.0;
    }
    (d - s) / s.max(d)
}

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

    let s0 = loader::load(std::path::Path::new(&dir)).expect("scenario loads");
    let (mut state, mut gd, events) = (s0.state, s0.game_data, s0.events);
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

    // Central difference in log price. 0.5% each way: small enough that the
    // band is still the binding term, large enough not to be float noise.
    const H: f64 = 0.005;

    println!(
        "{dir}  arm={arm:?}  ran under {}  posting under {}  after {ticks} ticks  (b_out={}, alpha per good)",
        ran_under.name(),
        gd.kernel.supply_rule.name(),
        gd.kernel.b_out
    );
    println!(
        "  {:<5} {:<10} {:>6} {:>9} {:>9} {:>9} {:>10} {:>11} {:>9} {:>9}",
        "node", "good", "alpha", "eps_s@p0", "eps_d@p0", "|G|@p0", "crossing", "eps_s@p*", "eps_d@p*", "|G|@p*"
    );

    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            let p0 = state.price(node, good);
            let alpha = gd.good(good).alpha;
            if !(p0 > 0.0) || alpha <= 0.0 {
                continue;
            }
            let (s_b, d_b) = sd(&state, &gd, arm, node, good, p0);
            if s_b <= 0.0 && d_b <= 0.0 {
                continue;
            }

            let gain_at = |p: f64| -> (f64, f64, f64) {
                let (sp, dp) = sd(&state, &gd, arm, node, good, p * (1.0 + H));
                let (sm, dm) = sd(&state, &gd, arm, node, good, p * (1.0 - H));
                let dlnp = ((1.0 + H) / (1.0 - H)).ln();
                let es = if sp > 0.0 && sm > 0.0 { (sp / sm).ln() / dlnp } else { f64::NAN };
                let ed = if dp > 0.0 && dm > 0.0 { (dp / dm).ln() / dlnp } else { f64::NAN };
                let di = (imbalance(sp, dp).clamp(-1.0, 1.0)
                    - imbalance(sm, dm).clamp(-1.0, 1.0))
                    / dlnp;
                (es, ed, (1.0 + alpha * di).abs())
            };

            let (es0, ed0, g0) = gain_at(p0);

            // Bisect for the crossing z = d - s over 16 decades around p0.
            let z = |p: f64| {
                let (s, d) = sd(&state, &gd, arm, node, good, p);
                d - s
            };
            let (mut lo, mut hi) = (p0 * 1e-8, p0 * 1e8);
            let (zl, zh) = (z(lo), z(hi));
            let crossing = if zl.signum() != zh.signum() && zl != 0.0 && zh != 0.0 {
                for _ in 0..80 {
                    // Geometric bisection: the price axis is a log axis here.
                    let m = (lo * hi).sqrt();
                    if z(m).signum() == zl.signum() {
                        lo = m;
                    } else {
                        hi = m;
                    }
                }
                Some((lo * hi).sqrt())
            } else {
                None
            };

            match crossing {
                Some(pstar) => {
                    let (es, ed, gs) = gain_at(pstar);
                    let (ss, ds) = sd(&state, &gd, arm, node, good, pstar);
                    // A "crossing" where both sides are ~0 is two empty curves
                    // meeting, not a market clearing. Labelled, never counted.
                    let degenerate = ss <= 1e-9 || ds <= 1e-9 || !es.is_finite() || !ed.is_finite();
                    println!(
                        "  {n:<5} {:<10} {alpha:>6.3} {es0:>9.3} {ed0:>9.3} {g0:>9.4} {:>10.3e} {es:>11.3} {ed:>9.3} {gs:>9.4}  q*={:.4}{}",
                        gd.good(good).name,
                        pstar / p0,
                        ss.min(ds),
                        if degenerate {
                            "  DEGENERATE"
                        } else if gs > 1.0 {
                            "  REAL-CROSSING GAIN>1"
                        } else {
                            "  REAL-CROSSING gain<1"
                        }
                    );
                }
                None => {
                    println!(
                        "  {n:<5} {:<10} {alpha:>6.3} {es0:>9.3} {ed0:>9.3} {g0:>9.4} {:>10} {:>11} {:>9} {:>9}",
                        gd.good(good).name, "NONE", "-", "-", "-"
                    );
                }
            }
        }
    }
}
