//! Rule 1's price responsiveness, tested falsification-first.
//!
//! The defect this exists for is measured and specific: posted supply had price
//! elasticity **exactly 0.000** for every good under the kernel, so the price
//! loop's gain was `|1 + α(ε_d − ε_s)| = 1` — a unit root, and no equilibrium
//! price existed for the rule to find. [`SupplyRule::Reservation`] divides the
//! buffer band by the desk's own markup, so what is withheld is a fixed *value*
//! and the price decides how many units that is.
//!
//! Every check below is paired with the thing that would have caught a lie:
//!
//! * the elasticity tests run under **both** rules, and the shipped rule must
//!   read a bit-identical quantity — a test that only ran the new rule could not
//!   tell "the fix works" from "the harness moves everything";
//! * the **uniform** bump is asserted to read 0.000 under the *new* rule, which
//!   is what proves the old all-goods probe blind. Someone reading 0.000 off
//!   that probe would have concluded the fix had failed;
//! * the loop-gain bound is asserted to be **violated** at `α·b_out = 2.5`. A
//!   bound nothing crosses is decoration;
//! * mode A on all five `solv_*` tapes must be **unchanged**, because the new
//!   rule is a strict generalisation that reduces to the old one at `R = 1`. If
//!   the hand-computed fixed points moved, the reduction is not exact and the
//!   whole "generalisation" claim is false.
//!
//! # [CORRECTION 2026-07-31] One of those pairs was not a pair
//!
//! The loop-gain bullet above described a test named
//! `the_loop_gain_bound_is_real_in_both_directions`, and that test never touched
//! the engine: `loop_residual` takes `eps_s` as an argument, is handed the
//! constant `B_OUT`, and iterates against analytic constant-elasticity curves.
//! It checks the arithmetic of `|1 + α(ε_d − ε_s)| < 1`. It cannot check that
//! the engine's markets sit where the arithmetic was evaluated, and the claim
//! that shipped beside it — "`α·b_out = 0.2`, a factor of ten inside the
//! boundary" — was therefore argued, not measured.
//!
//! The test keeps its body and is **renamed** to say what it does. The
//! conformance question it was mistaken for now has its own measurement,
//! `the_engines_measured_loop_gain_is_not_the_argued_0_8`, with
//! `the_gain_instrument_reads_the_derived_number_where_a_desk_actually_rests`
//! as its positive control. Measured answer: the derived 0.8 holds where a desk
//! rests (`solv_1g` reads `ε_s = 2.000` exactly) and nowhere else in `lr_00`,
//! where the worst reading in 20 ticks is **g = 2.28** — outside the boundary,
//! not a factor of ten inside it.
//!
//! Design and derivations: `docs/design/price-responsive-supply.md`.

use std::path::PathBuf;

use rustyecon::{
    certify::{state_hash, Certificate},
    kernel::{labour_posted, posted_above_band, reservation_band, AgentArm},
    runner::{RunConfig, SimRunner},
    scenario::loader,
    state::game_data::SupplyRule,
    state::{GameData, SimState},
    systems::{
        clearing::{imbalance, price_next},
        decisions, run_tick,
    },
    types::{
        ids::{GoodId, MarketNodeId},
        order::OrderSide,
    },
};

const B_OUT: f64 = 2.0; // the registered value, in every tape
const ALPHA: f64 = 0.1; // the registered per-good price step in the lr corpus
const BUMP: f64 = 1.01;

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios").join(name)
}

/// Total posted supply and demand per (node, good) from one decision phase.
fn posted(state: &SimState, gd: &GameData, arm: AgentArm) -> Vec<(f64, f64)> {
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

/// Run a scenario forward under one rule and hand back the state it reached.
///
/// The two rules are then evaluated **against the same state**, which is the
/// only way to isolate the posting function: letting each rule run its own
/// trajectory and comparing the endpoints would compare two different worlds.
fn advance(name: &str, ticks: u64, arm: AgentArm, rule: SupplyRule) -> (SimState, GameData) {
    let s = loader::load(&scenario_dir(name)).expect("scenario loads");
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    gd.kernel.supply_rule = rule;
    for _ in 0..ticks {
        run_tick(&mut state, &gd, &events, arm, rustyecon::systems::clearing::PriceRule::Imbalance);
    }
    (state, gd)
}

// ── 1. The defect, and that the fix removes it ────────────────────────────────

#[test]
fn the_shipped_rule_posts_a_bit_identical_quantity_when_one_price_moves() {
    // The defect control. `max(inventory − b_out·flow, 0)` names no price, so a
    // 1% move in any single price must leave every posted supply *bit* identical
    // — not "close". If this ever fails the shipped rule has acquired a price
    // term from somewhere and the measurement below is not measuring what it
    // says. It is asserted on the corpus scenario the finding was measured on.
    let (state, gd) = advance("lr_00", 200, AgentArm::Kernel, SupplyRule::Inelastic);
    let base = posted(&state, &gd, AgentArm::Kernel);

    let mut moved = 0;
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            let mut probe = state.clone();
            probe.set_price(node, good, state.price(node, good) * BUMP);
            let after = posted(&probe, &gd, AgentArm::Kernel);
            for i in 0..base.len() {
                if base[i].0 != after[i].0 {
                    moved += 1;
                }
            }
        }
    }
    assert_eq!(moved, 0, "the shipped Rule 1 moved {moved} posted supplies on a price alone");
}

#[test]
fn the_reservation_rule_makes_posted_supply_move_on_its_own_price() {
    // The same probe against the same state — only the posting rule differs —
    // so anything that moves here is the rule and not the trajectory.
    let (state, mut gd) = advance("lr_00", 200, AgentArm::Kernel, SupplyRule::Inelastic);
    let base = posted(&state, &gd, AgentArm::Kernel);
    gd.kernel.supply_rule = SupplyRule::Reservation;
    let base_new = posted(&state, &gd, AgentArm::Kernel);

    let mut elastic_markets = 0;
    let mut worst = 0.0f64;
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            let i = n * gd.num_goods() + g;
            if base_new[i].0 <= 0.0 {
                continue;
            }
            let mut probe = state.clone();
            probe.set_price(node, good, state.price(node, good) * BUMP);
            let after = posted(&probe, &gd, AgentArm::Kernel)[i].0;
            if after <= 0.0 {
                continue;
            }
            let eps = (after / base_new[i].0).ln() / BUMP.ln();
            if eps > 1e-6 {
                elastic_markets += 1;
            }
            worst = worst.max(eps);
        }
    }
    assert!(
        elastic_markets > 0,
        "no market gained a positive supply elasticity; worst reading {worst:.6}"
    );
    // And the elasticity must be positive, not merely nonzero: a negative one
    // would be a supply curve sloping the wrong way, which has no crossing
    // either. `base` is unused except to prove the two rules were compared on
    // one state.
    assert_eq!(base.len(), base_new.len());
    assert!(worst > 0.0, "supply elasticity must be positive, got {worst:.6}");
}

#[test]
fn the_all_goods_probe_cannot_see_the_fix() {
    // The test that would have caught someone declaring the change a failure off
    // the old probe — or, worse, declaring the OLD rule fine off it. The
    // reservation band depends on prices only through `cost/revenue`, which is
    // homogeneous of degree 0, so a uniform inflation must leave every posted
    // supply where it was. That is a requirement, not a defect: a supply that
    // responded to uniform inflation would be money illusion and would break
    // R12. It also means the probe as originally written was blind.
    let (state, mut gd) = advance("lr_00", 200, AgentArm::Kernel, SupplyRule::Inelastic);
    gd.kernel.supply_rule = SupplyRule::Reservation;
    let base = posted(&state, &gd, AgentArm::Kernel);

    let mut probe = state.clone();
    for n in 0..gd.num_nodes() {
        for g in 0..gd.num_goods() {
            let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
            probe.set_price(node, good, state.price(node, good) * BUMP);
        }
    }
    let after = posted(&probe, &gd, AgentArm::Kernel);

    for i in 0..base.len() {
        if base[i].0 <= 0.0 || after[i].0 <= 0.0 {
            continue;
        }
        let eps = (after[i].0 / base[i].0).ln() / BUMP.ln();
        // Not bit-identical: `Σ (1.01·p)·q` and `1.01·Σ p·q` differ in the last
        // ulp, so the claim is that the probe reads 0.000, which is the claim
        // that matters and is the one that makes it blind.
        assert!(eps.abs() < 1e-6, "uniform bump moved supply at slot {i}: eps_s = {eps:e}");
    }
}

// ── 2. The posting function's four required properties ────────────────────────

/// Posted quantity of one storable output as a function of its own price, for a
/// single-output desk: `revenue = p·qty_out`, `cost` fixed in `p`.
fn post_at(stock: f64, flow: f64, qty_out: f64, cost: f64, p: f64) -> f64 {
    posted_above_band(stock, reservation_band(B_OUT, flow, p * qty_out, cost))
}

#[test]
fn posted_supply_is_monotone_increasing_in_price_and_never_exceeds_stock() {
    // The two properties the brief names, over a grid rather than at a point:
    // a rule can be monotone at one operating point and fold back at another.
    for &stock in &[0.0, 0.5, 3.0, 40.0, 1e6] {
        for &flow in &[0.25, 1.0, 7.5] {
            for &qty_out in &[0.5, 1.0, 3.0] {
                for &cost in &[0.01, 1.0, 25.0] {
                    // Seeded at 0, not −∞: the first sample would otherwise
                    // count as "a strict rise" and the stock = 0 row — the row
                    // that proves the grid can distinguish a flat rule from a
                    // responsive one — would pass for free.
                    let mut prev = 0.0f64;
                    let mut strictly_rose = false;
                    // Geometric grid: the interesting structure (the shutdown
                    // price) sits at a ratio, not at an offset.
                    for k in 0..400 {
                        let p = 1e-4 * 1.05f64.powi(k);
                        let q = post_at(stock, flow, qty_out, cost, p);
                        assert!(q.is_finite(), "posted quantity must be a number: {q}");
                        assert!(
                            q <= stock + 1e-12,
                            "posted {q} exceeds stock {stock} at p={p}"
                        );
                        assert!(q >= 0.0, "posted {q} is negative at p={p}");
                        assert!(
                            q >= prev - 1e-12,
                            "posted fell from {prev} to {q} as price rose to {p}"
                        );
                        if q > prev + 1e-12 {
                            strictly_rose = true;
                        }
                        prev = q;
                    }
                    // A rule that is "monotone" because it never moves is the
                    // defect, so the grid must contain a real rise wherever
                    // there is stock to release.
                    assert_eq!(
                        strictly_rose,
                        stock > 0.0,
                        "stock {stock}: expected a strict rise iff there is stock to release"
                    );
                }
            }
        }
    }
}

#[test]
fn at_break_even_the_new_rule_is_exactly_the_shipped_rule() {
    // The claim that makes this a generalisation rather than a replacement, and
    // the reason every zero-profit resting-state result in the repo survives it.
    for &stock in &[0.0, 1.0, 17.5, 1e5] {
        for &flow in &[0.125, 1.0, 9.0] {
            let cost = 3.0;
            let qty_out = 1.5;
            let p = cost / qty_out; // R = 1 exactly
            let new = post_at(stock, flow, qty_out, cost, p);
            let old = (stock - B_OUT * flow).max(0.0);
            assert_eq!(new, old, "stock {stock} flow {flow}: {new} != {old}");
        }
    }
}

#[test]
fn elasticity_equals_capacity_over_posted_minus_one() {
    // The closed form `ε_s = I/q − 1` against a numerically differentiated
    // posting function. This is what makes `b_out` "the slope of the supply
    // curve" a checkable statement rather than a slogan.
    let h = 1e-6;
    let (mut checked, mut worst) = (0, 0.0f64);
    for &stock in &[1.0, 12.0, 500.0] {
        for &flow in &[0.5, 2.0] {
            for &cost in &[0.5, 4.0, 30.0] {
                for k in 0..60 {
                    let p = 0.05 * 1.2f64.powi(k);
                    let q = post_at(stock, flow, 1.0, cost, p);
                    if q <= 1e-9 || q >= stock - 1e-9 {
                        continue; // at a corner the derivative is one-sided
                    }
                    let up = post_at(stock, flow, 1.0, cost, p * (1.0 + h));
                    let dn = post_at(stock, flow, 1.0, cost, p * (1.0 - h));
                    let numeric = (up - dn) / (2.0 * h) / q;
                    let closed = stock / q - 1.0;
                    worst = worst.max((numeric - closed).abs() / closed.max(1.0));
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 100, "the grid must actually contain interior points, got {checked}");
    assert!(worst < 1e-6, "closed form and numeric derivative disagree by {worst:e}");
}

#[test]
fn at_a_desks_resting_point_the_elasticity_is_b_out_over_the_markup() {
    // `I = band + flow` is where a desk that sells everything it posts settles
    // (the same recursion that pins the superseded σ at −1/b_out), so `q = flow`
    // and `ε_s = band/flow = b_out/R`. This is the row of the stability table
    // the whole design rests on, so it is asserted rather than believed.
    for &r in &[0.5, 1.0, 2.0] {
        let (flow, qty_out) = (3.0, 1.0);
        let cost = 6.0;
        let p = r * cost / qty_out;
        let band = reservation_band(B_OUT, flow, p * qty_out, cost);
        let stock = band + flow;
        let q = post_at(stock, flow, qty_out, cost, p);
        assert!((q - flow).abs() < 1e-12, "resting desk should post its flow, got {q}");
        let eps = stock / q - 1.0;
        assert!((eps - B_OUT / r).abs() < 1e-12, "R={r}: eps_s {eps} != b_out/R {}", B_OUT / r);
    }
}

#[test]
fn the_two_degenerate_branches_are_reachable_and_neither_produces_a_nan() {
    // Both are economics, not guards, and both are recorded in the module doc.
    // A desk with no purchased inputs has no reservation price and posts
    // everything; a desk facing a zero price withholds everything.
    assert_eq!(reservation_band(B_OUT, 5.0, 10.0, 0.0), 0.0, "no cost, no reservation");
    assert_eq!(post_at(40.0, 5.0, 1.0, 0.0, 2.0), 40.0, "cost-free desk posts its stock");
    assert!(reservation_band(B_OUT, 5.0, 0.0, 3.0).is_infinite(), "zero price, infinite band");
    assert_eq!(post_at(40.0, 5.0, 1.0, 3.0, 0.0), 0.0, "a desk does not sell into a zero price");
    // The structural price floor: as p → 0 supply → 0, so imbalance → +1 and the
    // price rule marks UP. No clamp; a supply response.
    assert_eq!(post_at(40.0, 5.0, 1.0, 3.0, 1e-9), 0.0);
    assert!(post_at(40.0, 5.0, 1.0, 3.0, 1e9) > 0.0);
    // And no branch may produce a NaN, because R5 says an uncomputable metric
    // FAILS and a NaN order quantity would poison every aggregate downstream.
    for (rev, cost) in [(0.0, 0.0), (f64::NAN, 1.0), (1.0, f64::NAN), (-1.0, -1.0)] {
        let q = post_at(1.0, 1.0, 1.0, cost, rev);
        assert!(q.is_finite(), "rev {rev} cost {cost} gave {q}");
    }
}

#[test]
fn the_labour_rule_is_monotone_capped_at_hours_and_reduces_to_pi_h_at_the_reservation_wage() {
    let (hours, w_res) = (30.0, 4.0);
    for &pi in &[0.01, 0.25, 0.5, 1.0] {
        // Reduces to today's rule at the reservation wage.
        let at_res = labour_posted(hours, pi, B_OUT, w_res, w_res);
        assert!(
            (at_res - (pi * hours).min(hours)).abs() < 1e-12,
            "pi={pi}: {at_res} != pi*H"
        );
        // Monotone in the wage, never above physical hours, never negative.
        let mut prev = f64::NEG_INFINITY;
        for k in 0..300 {
            let w = 0.01 * 1.05f64.powi(k);
            let q = labour_posted(hours, pi, B_OUT, w, w_res);
            assert!(q.is_finite() && q >= 0.0 && q <= hours, "w={w} pi={pi} gave {q}");
            assert!(q >= prev - 1e-12, "posted hours fell from {prev} to {q} at w={w}");
            prev = q;
        }
        // The shutdown wage falls out: below w_res·b_out/(1+b_out) the pop posts
        // nothing at all.
        let shut = w_res * B_OUT / (1.0 + B_OUT);
        assert!(labour_posted(hours, pi, B_OUT, shut * 0.999, w_res) == 0.0);
        assert!(labour_posted(hours, pi, B_OUT, shut * 1.001, w_res) > 0.0);
    }
}

#[test]
fn the_rejected_naive_labour_rule_is_shown_to_be_the_disaster_it_was_rejected_for() {
    // Recorded as a control because it was the first form written down and
    // because "we considered it" is worth nothing without the number. The naive
    // `H·max(1 − w_res/w, 0)` posts ZERO at exactly the wage the market should
    // clear at, and its elasticity at the corpus's π floor is (1−π)/π = 99,
    // giving a loop gain of |1 − 0.1·99| = 8.9 — violently unstable.
    let naive = |hours: f64, w: f64, w_res: f64| hours * (1.0 - w_res / w).max(0.0);
    assert_eq!(naive(30.0, 4.0, 4.0), 0.0, "the naive rule collapses the market at w_res");
    assert!(labour_posted(30.0, 1.0, B_OUT, 4.0, 4.0) > 0.0, "the shipped shape does not");

    // The elasticity claim, measured rather than asserted from algebra: at the
    // operating point where the naive rule posts what the π-relative one does,
    // its elasticity is ~1/π.
    let (pi, hours, w_res) = (0.01, 30.0, 4.0);
    let w = w_res / (1.0 - pi); // naive posts exactly pi*H here
    let h = 1e-7;
    let q = naive(hours, w, w_res);
    let eps_naive = (naive(hours, w * (1.0 + h), w_res) - naive(hours, w * (1.0 - h), w_res))
        / (2.0 * h)
        / q;
    assert!(eps_naive > 90.0, "expected ~99, got {eps_naive}");
    assert!(
        (ALPHA * eps_naive - 1.0).abs() > 2.0,
        "and that is outside the stable band |1 - alpha*eps| < 1: gain {}",
        (1.0 - ALPHA * eps_naive).abs()
    );

    // The π-relative construction keeps it at b_out wherever π sits.
    let eps_ours = {
        let q = labour_posted(hours, pi, B_OUT, w_res, w_res);
        (labour_posted(hours, pi, B_OUT, w_res * (1.0 + h), w_res)
            - labour_posted(hours, pi, B_OUT, w_res * (1.0 - h), w_res))
            / (2.0 * h)
            / q
    };
    assert!((eps_ours - B_OUT).abs() < 1e-5, "expected b_out = {B_OUT}, got {eps_ours}");
}

// ── 3. The stability bound: the algebra, and then the engine ──────────────────

/// Iterate `p ← p·(1 + α·imbalance)` against analytic curves, and report the
/// worst |ln(p/p*)| over the last tenth of the run.
///
/// **Analytic curves.** `eps_s` is a parameter of this function, not a reading
/// off the engine, and nothing here calls the kernel. That is deliberate — it
/// isolates the scalar map — but it is also the whole limit of what the test
/// below can say. See the correction on that test.
fn loop_residual(alpha: f64, eps_s: f64, eps_d: f64, start: f64, steps: usize) -> f64 {
    let (p_star, q_star) = (1.0, 10.0);
    let mut p = start;
    let mut worst_late = 0.0f64;
    for t in 0..steps {
        let s = q_star * (p / p_star).powf(eps_s);
        let d = q_star * (p / p_star).powf(eps_d);
        p = price_next(p, alpha, imbalance(s, d));
        if !p.is_finite() || p <= 0.0 {
            return f64::INFINITY;
        }
        if t >= steps - steps / 10 {
            worst_late = worst_late.max((p / p_star).ln().abs());
        }
    }
    worst_late
}

#[test]
fn the_loop_gain_bound_is_arithmetic_and_this_test_checks_only_the_arithmetic() {
    // **RENAMED 2026-07-31 (R14). Was
    // `the_loop_gain_bound_is_real_in_both_directions`, and that name claimed
    // more than the body checks.** The old name, and the comment that said the
    // first line demonstrated "the registered product α·b_out = 0.2", read as a
    // statement about the engine. It is not one. `loop_residual` takes `eps_s`
    // as an ARGUMENT — it is passed `B_OUT` here, i.e. the resting-point value
    // `b_out/R` at `R = 1` — and iterates the scalar map against analytic
    // constant-elasticity curves. No kernel code runs in this test. It shows
    // that `|1 + α(ε_d − ε_s)| < 1` is the right condition for THAT map, which
    // is arithmetic, and it says nothing about whether the engine's markets sit
    // at `ε_s = 2`.
    //
    // They do not: `the_engines_measured_loop_gain_is_not_the_argued_0_8` below
    // measures `ε_s` and `ε_d` off a live state and finds gains above 1 at the
    // registered dials. The two tests are kept apart on purpose — this one is
    // the closed form, that one is the conformance — because the session that
    // wrote them conflated exactly these two things.
    //
    // Convergence of the scalar map at `α·ε_s = 0.1 × 2 = 0.2`, ε_s SUPPLIED...
    let ok = loop_residual(ALPHA, B_OUT, 0.0, 1.5, 4000);
    assert!(ok < 1e-6, "registered dials must converge, residual {ok:e}");
    // ...and the negative control, which is the point of the test: past
    // `α·ε_s = 2` the same map does not settle. A bound nothing violates is
    // decoration. Note it does not explode — the imbalance normaliser saturates
    // at ±1 — so the honest assertion is "does not converge", not "diverges to
    // infinity", and that distinction is itself a finding about the price rule.
    let bad = loop_residual(ALPHA, 25.0, 0.0, 1.5, 4000);
    assert!(bad > 1e-3, "eps_s = 25 (alpha*eps_s = 2.5) must not settle, residual {bad:e}");
    assert!(bad.is_finite(), "and it is a bounded limit cycle, not a blow-up: {bad:e}");

    // The unit root the shipped rule sat on: ε_s = ε_d = 0 leaves the price
    // wherever it started, forever. This is the row the whole change is aimed at.
    let unit_root = loop_residual(ALPHA, 0.0, 0.0, 1.5, 4000);
    assert!(
        (unit_root - 1.5f64.ln()).abs() < 1e-9,
        "with both elasticities zero the price cannot move at all: {unit_root}"
    );
}

/// `ε_s` and `ε_d` for one `(node, good)` on a live state, by central difference
/// in the log price, and the loop gain that follows from them.
///
/// Central rather than one-sided: the gain is a claim about a derivative at the
/// operating point, and a secant from `p` to `1.01·p` is not one. Returns `None`
/// where either side has no reading — a market with zero posted supply is not
/// near a crossing at all, and folding it in as "gain = 1" would be inventing a
/// measurement.
fn measured_gain(
    state: &SimState,
    gd: &GameData,
    arm: AgentArm,
    node: MarketNodeId,
    good: GoodId,
) -> Option<(f64, f64, f64)> {
    let i = node.idx() * gd.num_goods() + good.idx();
    let p0 = state.price(node, good);
    let mut up = state.clone();
    up.set_price(node, good, p0 * BUMP);
    let up = posted(&up, gd, arm)[i];
    let mut dn = state.clone();
    dn.set_price(node, good, p0 / BUMP);
    let dn = posted(&dn, gd, arm)[i];
    if up.0 <= 0.0 || dn.0 <= 0.0 || up.1 <= 0.0 || dn.1 <= 0.0 {
        return None;
    }
    let dln = 2.0 * BUMP.ln();
    let (eps_s, eps_d) = ((up.0 / dn.0).ln() / dln, (up.1 / dn.1).ln() / dln);
    let alpha = gd.good(good).alpha;
    Some((eps_s, eps_d, (1.0 + alpha * (eps_d - eps_s)).abs()))
}

#[test]
fn the_engines_measured_loop_gain_is_not_the_argued_0_8() {
    // **THE CORRECTION, 2026-07-31 (R14).** The kernel module header, PLAN
    // Phase 4 and the design note all state that at the registered dials
    // `α·b_out = 0.1 × 2 = 0.2` puts the loop "a factor of ten inside the
    // boundary", i.e. `g = 0.8`. That was ARGUED from `ε_s* = b_out/R` at a
    // desk's RESTING POINT, and then quoted as if it described the engine. This
    // test measures it instead, and the argued number is wrong in both
    // directions:
    //
    //   * most live markets read `g` between 0.9 and 1.0, not 0.8, because
    //     `ε_s` is nowhere near 2 (`ε_s = I/q − 1` — a desk posting its whole
    //     stock reads 0, and a market with `ε_s = ε_d = 0` reads `g = 1.000`
    //     EXACTLY, the unit root, under the price-responsive rule too);
    //   * and at least one market reads `g > 1` — outside the boundary the
    //     design says the registered dials sit a factor of ten inside.
    //
    // Measured on `lr_00`, kernel arm, state evolved under the SHIPPED rule
    // (`inelastic`, which is what all 33 tapes register), posting evaluated
    // under `reservation`. The worst reading in the first 20 ticks is at
    // **tick 7, node 1, wheat: ε_s = 31.68, ε_d = −1.16, α = 0.100, g = 2.28** —
    // a desk posting 0.47 units against 30.9 demanded, i.e. holding some forty
    // times what it offers, which is precisely `ε_s = I/q − 1` far from rest.
    //
    // Asserted as a FAILURE of the claim rather than as a defect to fix, in the
    // house pattern: if this ever stops firing, either the corpus reached its
    // resting points (the good outcome) or the elasticity stopped depending on
    // the state, and either way the claim above needs re-measuring, not this
    // test relaxing.
    let s = loader::load(&scenario_dir("lr_00")).expect("scenario loads");
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    gd.kernel.supply_rule = SupplyRule::Inelastic;

    let (mut worst, mut worst_at) = (0.0f64, String::new());
    let mut unit_roots = 0;
    let mut readings = 0;
    for t in 1..=20u64 {
        gd.kernel.supply_rule = SupplyRule::Inelastic;
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, rustyecon::systems::clearing::PriceRule::Imbalance);
        gd.kernel.supply_rule = SupplyRule::Reservation;
        for n in 0..gd.num_nodes() {
            for g in 0..gd.num_goods() {
                let (node, good) = (MarketNodeId(n as u32), GoodId(g as u32));
                let Some((eps_s, eps_d, gain)) = measured_gain(&state, &gd, AgentArm::Kernel, node, good)
                else {
                    continue;
                };
                readings += 1;
                if (gain - 1.0).abs() < 1e-12 {
                    unit_roots += 1;
                }
                if gain > worst {
                    worst = gain;
                    worst_at = format!(
                        "tick {t}, node {n}, {}: eps_s = {eps_s:.3}, eps_d = {eps_d:.3}, \
                         alpha = {:.3}, g = {gain:.3}",
                        gd.good(good).name,
                        gd.good(good).alpha,
                    );
                }
            }
        }
    }

    assert!(readings > 10, "the scan must find markets to read, got {readings}");
    assert!(
        worst > 1.0,
        "the engine's own loop gain is claimed to sit at 0.8; the worst measured \
         reading over 20 ticks of lr_00 was {worst:.3} ({worst_at}). If this is now \
         below 1 the claim may be defensible again — RE-MEASURE and mark the docs, \
         do not delete the test."
    );
    // And the unit root survives the fix in every market where the desk posts
    // its whole stock, which is the other half of why 0.8 is not the engine's
    // number. Reported, not asserted at a count: the count moves with the
    // trajectory and pinning it would be pinning a tape.
    println!("engine loop gain: worst {worst:.3} at {worst_at}; {unit_roots}/{readings} readings sat at g = 1.000 exactly");
}

#[test]
fn the_gain_instrument_reads_the_derived_number_where_a_desk_actually_rests() {
    // The positive control for the test above, and the reason its 2.28 is a fact
    // about the STATE rather than about the instrument. A guard that only ever
    // reports "worse than derived" could be a broken thermometer; this one is
    // shown reading the derived value exactly where the derivation applies.
    //
    // `solv_1g` under the reservation rule sits at its hand-computed fixed point:
    // one desk, `R = 1`, selling everything it posts, so `I = band + flow` and
    // `ε_s* = b_out/R = 2` is not an approximation. The engine measures
    // **ε_s = 2.000** there, at tick 50 and unchanged at tick 200.
    //
    // And one detail that the "α·b_out = 0.2" slogan hides, worth its own line:
    // **α is per good, not a kernel dial.** `grain` in this tape registers
    // α = 0.05, so the gain at this resting point is |1 + 0.05·(0 − 2)| = 0.900,
    // not the 0.800 the design's table quotes for α = 0.1. The product α·b_out
    // is 0.2 for `lr_00`'s wheat and flour and 0.1 for its labour. There is no
    // single registered α to be a factor of ten inside anything.
    for ticks in [50u64, 200] {
        let (state, gd) = advance("solv_1g", ticks, AgentArm::Kernel, SupplyRule::Reservation);
        let grain = gd.goods.iter().position(|g| g.name == "grain").expect("grain");
        let (node, good) = (MarketNodeId(0), GoodId(grain as u32));
        let (eps_s, eps_d, gain) =
            measured_gain(&state, &gd, AgentArm::Kernel, node, good).expect("grain trades here");
        assert!(
            (eps_s - B_OUT).abs() < 2e-3,
            "at a resting desk eps_s must be b_out/R = {B_OUT}, measured {eps_s} after {ticks} ticks"
        );
        let alpha = gd.good(good).alpha;
        let expected = (1.0 + alpha * (eps_d - B_OUT)).abs();
        assert!(
            (gain - expected).abs() < 1e-3,
            "gain {gain} should be |1 + {alpha}(eps_d - b_out)| = {expected} after {ticks} ticks"
        );
        assert!(gain < 1.0, "and the resting point is inside the boundary: {gain}");
    }
}

// ── 4. Nothing that already worked may break ──────────────────────────────────

#[test]
fn mode_a_is_bit_identical_under_both_rules_on_the_zero_profit_tapes() {
    // Four of the five `solv_*` tapes start AT a hand-computed equilibrium where
    // every desk is at zero profit and `R = 1` *exactly* — each tape's numbers
    // are dyadic rationals, so that is exact in f64, not nearly. The new rule
    // must therefore be the old rule there, tick for tick, to the last bit.
    //
    // This is the strongest available check that the change is a
    // generalisation: if the reduction at R = 1 were even slightly inexact,
    // 1,000 ticks of a multiplicative loop would show it.
    for name in ["solv_1g", "solv_1g_money", "solv_1g_money_2x", "solv_chain"] {
        let (a, _) = advance(name, 1000, AgentArm::Kernel, SupplyRule::Inelastic);
        let (b, _) = advance(name, 1000, AgentArm::Kernel, SupplyRule::Reservation);
        assert_eq!(
            state_hash(&a),
            state_hash(&b),
            "{name}: the reservation rule moved a fixed point it should reduce to"
        );
    }
}

#[test]
fn mode_a_breaks_on_solv_labour_because_its_equilibrium_markup_is_two_not_one() {
    // **The limitation of this design, found by running it, asserted so it
    // cannot quietly go away.**
    //
    // The reservation rule treats `R = 1` as the price at which a desk is
    // indifferent. That is the right reservation price only where the
    // equilibrium markup IS one. `solv_labour` is the corpus's one world with a
    // scarce second factor: `recipe_size` binds, the firm earns a capacity rent,
    // and its equilibrium markup is revenue 20·p_grain over outlay 10·p_labour
    // with `p_labour = p_grain` — i.e. **R = 2, at the equilibrium**.
    //
    // The band is then `b_out·flow/2`, the desk posts more than it produces,
    // supply exceeds demand, and the hand-computed fixed point stops being one.
    // The same tape is already the registered counterexample to reading B8 as a
    // correctness verdict; it is now also the registered counterexample to
    // reading `R = 1` as "break-even". A Leontief reservation price cannot see a
    // scarce second factor either.
    //
    // This is asserted as a FAILURE rather than fixed, per the house rule: the
    // fix is a rent-aware reservation price, which is a new mechanism with its
    // own derivation and its own A/B, not a patch smuggled in beside this one.
    let (a, _) = advance("solv_labour", 1000, AgentArm::Kernel, SupplyRule::Inelastic);
    let (b, _) = advance("solv_labour", 1000, AgentArm::Kernel, SupplyRule::Reservation);
    assert_ne!(
        state_hash(&a),
        state_hash(&b),
        "if this ever passes, either solv_labour's rent vanished or the rule stopped keying on R"
    );

    // And the direction is the predicted one: a desk whose markup is above
    // break-even releases MORE stock, so grain supply is higher on tick 1.
    let (s0, gd0) = advance("solv_labour", 1, AgentArm::Kernel, SupplyRule::Inelastic);
    let old = posted(&s0, &gd0, AgentArm::Kernel);
    let (s1, gd1) = advance("solv_labour", 1, AgentArm::Kernel, SupplyRule::Reservation);
    let new = posted(&s1, &gd1, AgentArm::Kernel);
    let grain = gd0.goods.iter().position(|g| g.name == "grain").expect("grain");
    let i = grain; // one node
    assert!(
        new[i].0 > old[i].0,
        "a desk above break-even must release more, not less: {} vs {}",
        new[i].0,
        old[i].0
    );
}

fn certify(scenario: &str, ticks: u64, agents: AgentArm, rule: SupplyRule) -> Certificate {
    let dir = scenario_dir(scenario);
    let s = loader::load(&dir).expect("scenario loads");
    let out = tempfile::tempdir().unwrap();
    let config = RunConfig {
        agents,
        price_rule: rustyecon::systems::clearing::PriceRule::Imbalance,
        ticks,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: out.path().to_path_buf(),
        record: false,
        telemetry_every: 1,
        certify: true,
        results_dir: out.path().join("results"),
        supply_rule: Some(rule),
    };
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&dir)
        .with_criteria(s.criteria);
    runner.run().expect("certify mode yields a certificate")
}

#[test]
fn b6_still_passes_under_the_kernel_with_the_price_responsive_rule() {
    // "It only reads a posted price" is exactly the kind of claim that turns out
    // to have a demand term hiding in it, so it is re-run rather than argued.
    // B6 perturbs last tick's aggregated supply and demand on a clone and
    // re-runs the decision phase; the reservation band reads `state.price`,
    // which B6 does not touch.
    //
    // The falsification half lives in `test_10_invariants.rs`, where the same
    // battery is shown FIRING on the legacy layer's `1.2 × demand` sell cap.
    let cert = certify("lr_00", 200, AgentArm::Kernel, SupplyRule::Reservation);
    let b6 = cert.batteries.iter().find(|b| b.id == "B6").expect("B6");
    assert!(b6.pass, "B6 must still pass under the reservation rule: {}", b6.detail);
}

#[test]
fn the_supply_rule_is_recorded_in_the_run_identity() {
    // `--supply-rule` changes the mechanism without changing the tape sha, so
    // without this two certificates for one tape would be indistinguishable and
    // the A/B would have no receipt.
    let old = certify("lr_00", 30, AgentArm::Kernel, SupplyRule::Inelastic);
    let new = certify("lr_00", 30, AgentArm::Kernel, SupplyRule::Reservation);
    assert_eq!(old.identity.agents, "kernel+imbalance+inelastic");
    assert_eq!(new.identity.agents, "kernel+imbalance+reservation");
    assert_eq!(old.identity.tape_sha, new.identity.tape_sha, "same tape");
    assert_ne!(old.identity.run, new.identity.run, "different runs");
}
