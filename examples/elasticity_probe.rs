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
//! ## An elasticity with no stated state is not a measurement
//!
//! **[CORRECTION 2026-07-31] The first version of this probe printed a table
//! that could not be reproduced from what it printed**, and PLAN Phase 4 and
//! `src/kernel/mod.rs` duly quoted a headline range (`ε_s` "0.216 to 45.2") that
//! nothing recorded the conditions of. Re-running found the individual figures
//! scattered across *three different tick counts and two different evolution
//! rules*: 1.111 exists only at tick 20 with the state evolved under the shipped
//! rule; 16.7/45.2/0.216/6.10 only at tick 20 with the state evolved under
//! `reservation_goods`; 43.5/6.73/17.7 only at tick 200 under the same. That is
//! not a property of the rule, it is a tour of a trajectory.
//!
//! `ε_s = I/q − 1` is a function of the *state*, not of the rule: the same rule
//! reads 0.000 on a desk posting its whole stock and 45 on a desk posting a
//! fortyfifth of it. So the probe now prints, above every table, the full set of
//! conditions needed to reproduce the row — scenario, tape sha, arm, the rule the
//! trajectory was evolved under, the rule the posting was measured under, the
//! tick count, and the `state_hash` of the state measured on — and it takes a
//! *list* of tick counts, because a single tick chosen after the fact is a
//! sample of one from a series nobody was shown.
//!
//!     cargo run --release --example elasticity_probe -- <scenario> <ticks|t1,t2,…> <arm> [run_rule] [post_rule]

use rustyecon::{
    certify::{certificate::tape_sha, state_hash},
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

/// What one tick count's measurement is worth quoting on its own.
///
/// Kept so the sweep can print the *series* a single quoted figure was drawn
/// from. The Phase 4 headline range was assembled from three tick counts and two
/// evolution rules without saying so; a summary line per tick makes that visible
/// the next time someone reaches for a number.
struct Sweep {
    ticks: u64,
    state_hash: u64,
    markets: usize,
    elastic: usize,
    max_eps_s: f64,
    max_gain: f64,
    unstable: usize,
    no_supply: usize,
}

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "data/scenarios/lr_00".into());
    // A LIST of tick counts, not one: `ε_s = I/q − 1` is a function of the state,
    // so a single tick chosen after the numbers were seen is a sample of one.
    let ticks_arg = a.next().unwrap_or_else(|| "200".into());
    let mut ticks: Vec<u64> =
        ticks_arg.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    ticks.sort_unstable();
    ticks.dedup();
    if ticks.is_empty() {
        ticks.push(200);
    }
    let arm = match a.next().as_deref() {
        Some("kernel") => AgentArm::Kernel,
        _ => AgentArm::Legacy,
    };
    let run_rule: Option<SupplyRule> = a.next().and_then(|s| s.parse().ok());
    let post_rule: Option<SupplyRule> = a.next().and_then(|s| s.parse().ok());

    let path = std::path::Path::new(&dir);
    let s = loader::load(path).expect("scenario loads");
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    if let Some(r) = run_rule {
        gd.kernel.supply_rule = r;
    }
    let evolved_under = gd.kernel.supply_rule;
    let measured_under = post_rule.unwrap_or(evolved_under);
    let tape = tape_sha(path);

    const BUMP: f64 = 1.01;
    let dlnp = BUMP.ln();

    let mut done = 0u64;
    let mut sweep: Vec<Sweep> = Vec::new();

    for &t in &ticks {
        while done < t {
            // Evolved under `evolved_under` — the posting rule is swapped in only
            // for the measurement below, and swapped back before the next tick.
            gd.kernel.supply_rule = evolved_under;
            run_tick(&mut state, &gd, &events, arm, PriceRule::Imbalance);
            done += 1;
        }
        gd.kernel.supply_rule = measured_under;

        let base = posted(&state, &gd, arm);

        // ── Control: the old all-goods bump ──────────────────────────────────
        let mut uniform = state.clone();
        for n in 0..gd.num_nodes() {
            for g in 0..gd.num_goods() {
                let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
                uniform.set_price(node, good, state.price(node, good) * BUMP);
            }
        }
        let uniform = posted(&uniform, &gd, arm);

        // ── The conditions, printed with the table and not left to a caption ──
        //
        // Everything needed to reproduce a row: which tape, which arm, which rule
        // the trajectory ran under, which rule the posting was evaluated under,
        // how many ticks, and the hash of the exact state measured on. R14: a
        // number quoted without these is not reproducible and should not be
        // quoted.
        println!(
            "\nCONDITIONS  scenario={dir}  tape_sha={tape}  arm={arm:?}  \
             evolved_under={}  posted_under={}  ticks={t}  state_hash={:016x}  b_out={}",
            evolved_under.name(),
            measured_under.name(),
            state_hash(&state),
            gd.kernel.b_out,
        );
        println!(
            "  OWN-PRICE: one (node, good) at a time, central ±1% in log price. \
             `eps_s+` is the one-sided +1% the Phase 4 table was read off. \
             UNIFORM: every price bumped at once (degree-0 control). \
             gain = |1 + alpha(eps_d - eps_s)|, alpha per good as registered; the loop is \
             locally stable only where gain < 1."
        );
        println!(
            "  {:<4} {:<12} {:>11} {:>11} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6} {:>7}   crossing?",
            "node",
            "good",
            "supply",
            "demand",
            "eps_s",
            "eps_s+",
            "eps_d",
            "u_eps_s",
            "u_eps_d",
            "alpha",
            "gain"
        );

        let (mut markets, mut elastic, mut unstable, mut no_supply) = (0, 0, 0, 0);
        let (mut max_eps_s, mut max_gain) = (f64::NEG_INFINITY, f64::NEG_INFINITY);

        for n in 0..gd.num_nodes() {
            for g in 0..gd.num_goods() {
                let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
                let i = n * gd.num_goods() + g;
                let (s0, d0) = base[i];
                if s0 <= 0.0 && d0 <= 0.0 {
                    continue;
                }
                markets += 1;

                // One price moved, everything else held. This is the partial the
                // crossing question is about. Central rather than one-sided: the
                // gain below is a claim about a *derivative* at the operating
                // point, and a one-sided secant over a 1% step is not one.
                let p0 = state.price(node, good);
                let mut up = state.clone();
                up.set_price(node, good, p0 * BUMP);
                let up = posted(&up, &gd, arm)[i];
                let mut dn = state.clone();
                dn.set_price(node, good, p0 / BUMP);
                let dn = posted(&dn, &gd, arm)[i];

                let es = elasticity(dn.0, up.0, 2.0 * dlnp);
                let ed = elasticity(dn.1, up.1, 2.0 * dlnp);
                let es_one = elasticity(s0, up.0, dlnp);
                let (us, ud) = (
                    elasticity(s0, uniform[i].0, dlnp),
                    elasticity(d0, uniform[i].1, dlnp),
                );

                let alpha = gd.good(good).alpha;
                // The gain is only defined where both curves are on the curve at
                // all. A market with zero posted supply is not near a crossing —
                // its imbalance is pinned at +1 and the price rule steps by the
                // full alpha regardless of any elasticity — so it is counted
                // separately rather than folded in as "gain = 1".
                let gain = if es.is_finite() && ed.is_finite() {
                    (1.0 + alpha * (ed - es)).abs()
                } else {
                    f64::NAN
                };
                if s0 <= 0.0 {
                    no_supply += 1;
                }
                if es.is_finite() {
                    if es > 1e-6 {
                        elastic += 1;
                    }
                    max_eps_s = max_eps_s.max(es);
                }
                if gain.is_finite() {
                    max_gain = max_gain.max(gain);
                    if gain > 1.0 {
                        unstable += 1;
                    }
                }

                let cross = match (es.is_finite(), ed.is_finite()) {
                    (true, true) if (es - ed) > 1e-6 => "yes",
                    (true, false) if es > 1e-6 => "yes (supply only)",
                    (false, true) if ed < -1e-6 => "yes (demand only)",
                    _ => "NO - no price clears this market",
                };
                println!(
                    "  {n:<4} {:<12} {s0:>11.4} {d0:>11.4} {es:>8.3} {es_one:>8.3} {ed:>8.3} \
                     {us:>8.3} {ud:>8.3} {alpha:>6.3} {gain:>7.3}   {cross}",
                    gd.good(good).name
                );
            }
        }
        sweep.push(Sweep {
            ticks: t,
            state_hash: state_hash(&state),
            markets,
            elastic,
            max_eps_s,
            max_gain,
            unstable,
            no_supply,
        });
    }

    // ── The series, so no single tick can be quoted as "the" elasticity ───────
    println!(
        "\nSWEEP  scenario={dir}  arm={arm:?}  evolved_under={}  posted_under={}",
        evolved_under.name(),
        measured_under.name()
    );
    println!(
        "  {:>6} {:>18} {:>8} {:>8} {:>10} {:>9} {:>9} {:>10}",
        "ticks", "state_hash", "markets", "elastic", "max eps_s", "max gain", "gain>1", "no supply"
    );
    for r in &sweep {
        // "no market had a finite reading" is not "the maximum was zero", and
        // printing −inf for it would invite exactly the kind of quotation this
        // sweep exists to stop. R5's convention: a value that does not exist is
        // reported as absent, never as a number.
        let fmt = |v: f64| {
            if v.is_finite() {
                format!("{v:.3}")
            } else {
                "none".to_string()
            }
        };
        println!(
            "  {:>6} {:>18} {:>8} {:>8} {:>10} {:>9} {:>9} {:>10}",
            r.ticks,
            format!("{:016x}", r.state_hash),
            r.markets,
            r.elastic,
            fmt(r.max_eps_s),
            fmt(r.max_gain),
            r.unstable,
            r.no_supply
        );
    }
}
