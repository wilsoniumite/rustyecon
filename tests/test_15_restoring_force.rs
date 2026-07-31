//! **The one result the 2026-07-31 session rests on, re-established and cut down
//! to what it actually says.**
//!
//! The claim under audit, as it was written into `src/kernel/mod.rs` and PLAN
//! Phase 4:
//!
//! > a displaced `solv_1g` grain price returns to 1.000 under `reservation`
//! > while sitting at 1.010 with standard deviation exactly zero under
//! > `inelastic`.
//!
//! Re-measured independently by `examples/restoring_force.rs` (the full sweep,
//! both price rules, all three supply rules, three tapes) and by the tests
//! below. **Two of the three halves survive and one does not:**
//!
//! * **SURVIVES, and is exact.** Under `inelastic` a +1% displacement does not
//!   merely have a small spread — the price series is *bit-constant* for all
//!   1000 ticks. Every kernel response to the displacement is identically zero.
//! * **SURVIVES, with a boundary nobody had stated.** The freeze holds exactly
//!   as far as the tape's own `dead = 0.05` predicts and not one grid point
//!   further: frozen at 1.0512×, moving at 1.0513×, against the dial-derived
//!   `(1 + d/2)/(1 − d/2) = 1.051282`.
//! * **DOES NOT SURVIVE.** "Returns to 1.000" is false at the tape's own
//!   registered exactness. It returns to `1.0000000000186`, and that residual is
//!   set by the **absolute** `1e-12` price-delta guard in
//!   `src/systems/price_update/mod.rs:28`, not by the economics — demonstrated
//!   by doubling the price level and watching the residual halve.
//!
//! And one thing the original statistic could not have seen, because the
//! displacement was only ever applied upward:
//!
//! * **The freeze is ONE-SIDED.** Displace *down* by 1% and the price does not
//!   sit there: it walks up through the target and rests at `+0.87%` on the
//!   other side, with `sd` over the scored window still exactly 0. So `sd = 0`
//!   does not mean "no restoring force" — it means "no motion in the window",
//!   and those are different claims. `max − min` over the *whole* series is the
//!   statistic that separates them, and it is what these tests use.
//!
//! Every test here asserts what was MEASURED, including where the measurement
//! contradicts the recorded claim. None of them is a bar to be widened.

use std::path::PathBuf;

use rustyecon::{
    kernel::AgentArm,
    scenario::{equilibrium::Equilibrium, loader},
    state::game_data::SupplyRule,
    systems::{clearing::PriceRule, run_tick},
    types::ids::{GoodId, MarketNodeId},
};

const TICKS: u64 = 1000;
const NODE: MarketNodeId = MarketNodeId(0);

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios").join(name)
}

/// `ln(p_displaced / p_numeraire)` on every tick, index 0 = the displaced
/// genesis before any tick has run.
///
/// Index 0 is included deliberately: without it, a run that moved during the
/// first tick and then froze is indistinguishable from one that never moved,
/// and that distinction is the whole content of the second half of this file.
fn series(
    name: &str,
    price_rule: PriceRule,
    supply: SupplyRule,
    displaced: &str,
    numeraire: &str,
    factor: f64,
) -> Vec<f64> {
    let s = loader::load(&scenario_dir(name)).unwrap_or_else(|e| panic!("{name} loads: {e}"));
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    gd.kernel.supply_rule = supply;
    let id = |g: &str| {
        GoodId(gd.goods.iter().position(|x| x.name == g).unwrap_or_else(|| panic!("{g}")) as u32)
    };
    let (d, n) = (id(displaced), id(numeraire));

    let p = state.price(NODE, d) * factor;
    state.set_price(NODE, d, p);
    state.set_price_ema(NODE, d, p);

    let mut out = Vec::with_capacity(TICKS as usize + 1);
    out.push((state.price(NODE, d) / state.price(NODE, n)).ln());
    for _ in 0..TICKS {
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, price_rule);
        out.push((state.price(NODE, d) / state.price(NODE, n)).ln());
    }
    out
}

/// `max − min`. Exact under any summation, so `0.0` means bit-constant and means
/// it without reference to an estimator — unlike a two-pass standard deviation,
/// which reads ~1e-15 on constant data and would make "exactly zero" a claim
/// about the arithmetic rather than about the run.
fn spread(v: &[f64]) -> f64 {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for x in v {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    hi - lo
}

/// Welford, the estimator `certify::invariants::std_dev` and B8 both use. On
/// bit-constant data every delta is identically 0.0 so this returns exactly 0 —
/// which is what makes the original "standard deviation exactly zero" a true
/// sentence about a real property rather than about a rounding accident.
fn welford_sd(v: &[f64]) -> f64 {
    let (mut mean, mut m2) = (0.0f64, 0.0f64);
    for (i, x) in v.iter().enumerate() {
        let d = x - mean;
        mean += d / (i + 1) as f64;
        m2 += d * (x - mean);
    }
    (m2 / (v.len() - 1) as f64).sqrt()
}

fn load_eq(name: &str) -> Equilibrium {
    Equilibrium::load(&scenario_dir(name))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
        .unwrap_or_else(|| panic!("{name} registers no equilibrium.ron"))
}

/// `ln` of the registered target ratio for a tape's displaced good.
fn target_log(eq: &Equilibrium) -> f64 {
    let d = eq.displaced.as_ref().expect("registers a displacement");
    (eq.prices[&d.good] / eq.prices[&eq.numeraire]).ln()
}

// ── 1. The unit root, asserted as an exact property ───────────────────────────

/// **THE RESULT, RE-ESTABLISHED, AND IT IS STRONGER THAN "SD IS ZERO".**
///
/// Under `inelastic` the shipped Rule 1 posts `max(stock − b_out·flow, 0)`,
/// which names no price. A +1% displacement therefore leaves posted supply
/// bit-identical, both markets balanced, and every σ inside its dead band. The
/// prediction is not "the spread is small" but "no state variable moves at all",
/// so that is what is asserted: the price series is bit-constant over all 1000
/// ticks, on three tapes, under both price rules.
///
/// Measured 2026-07-31 (second pass): `max − min` is `0.0`, exactly, in all 12
/// configurations below, and Welford's sd is `0.0` exactly with it.
#[test]
fn under_the_inelastic_rule_a_small_upward_displacement_never_moves_at_all() {
    for (tape, good) in [("solv_1g", "grain"), ("solv_chain", "flour"), ("solv_1g_money", "grain")]
    {
        for rule in [PriceRule::Imbalance, PriceRule::Ratio] {
            for f in [1.001, 1.01] {
                let v = series(tape, rule, SupplyRule::Inelastic, good, "labour", f);
                assert_eq!(
                    spread(&v),
                    0.0,
                    "{tape}/{}/inelastic displaced {f}x: the price series must be BIT-CONSTANT \
                     — the shipped Rule 1 names no price, so nothing can respond. max-min was \
                     {:e}. If this moved, Rule 1 has acquired a price term.",
                    rule.name(),
                    spread(&v)
                );
                assert_eq!(welford_sd(&v), 0.0, "{tape}/{}: sd must be exactly 0", rule.name());
                // And it sits where it was put, to the last bit: the displaced
                // ratio, not the target.
                assert_eq!(
                    v[v.len() - 1],
                    v[0],
                    "{tape}/{}: the final price must be the displaced one, bit for bit",
                    rule.name()
                );
            }
        }
    }
}

/// **The falsification control for the test above**, and it fires on a defect
/// that provably exists rather than an invented one.
///
/// A test asserting "nothing moved" passes trivially on an engine that cannot
/// move anything. So the same code is run one grid point past the boundary the
/// tape's own dial predicts. Rule 2 gates scale on the desk's relative margin
/// `2(R − 1)/(R + 1)` against `dead = 0.05`, so the freeze must end at
/// `(1 + d/2)/(1 − d/2) = 1.051282…`.
///
/// Measured: frozen at `1.0512×` (max−min exactly 0), moving at `1.0513×`
/// (max−min 1.027). The prediction comes from a registered dial and the
/// measurement lands between two adjacent four-decimal probes of it; nothing
/// here was fitted.
#[test]
fn the_freeze_ends_exactly_where_the_registered_dead_band_says_it_does() {
    let s = loader::load(&scenario_dir("solv_1g")).expect("loads");
    let dead = s.game_data.kernel.dead;
    let edge = (1.0 + dead / 2.0) / (1.0 - dead / 2.0);
    assert!(
        (edge - 1.051282).abs() < 1e-5,
        "the dial-derived edge moved: dead = {dead} gives {edge}"
    );

    let inside = series("solv_1g", PriceRule::Imbalance, SupplyRule::Inelastic, "grain", "labour", 1.0512);
    let outside = series("solv_1g", PriceRule::Imbalance, SupplyRule::Inelastic, "grain", "labour", 1.0513);
    assert_eq!(
        spread(&inside),
        0.0,
        "just inside the dead band the price must not move at all, got {:e}",
        spread(&inside)
    );
    assert!(
        spread(&outside) > 1.0,
        "just OUTSIDE the dead band the price must move, and move far — otherwise the test \
         above is measuring an engine that cannot move anything. Got max-min {:e}",
        spread(&outside)
    );
    // The same boundary on the low side, which is where the two-sidedness of the
    // gate is checked rather than assumed.
    let lo_in = series("solv_1g", PriceRule::Imbalance, SupplyRule::Inelastic, "grain", "labour", 0.9513);
    let lo_out = series("solv_1g", PriceRule::Imbalance, SupplyRule::Inelastic, "grain", "labour", 0.9512);
    assert!(
        (1.0 / edge - 0.951220).abs() < 1e-5,
        "the low edge moved: {}",
        1.0 / edge
    );
    assert!(
        spread(&lo_out) > 1.0,
        "outside the low edge the price must move: {:e}",
        spread(&lo_out)
    );
    // NOT asserted as bit-constant: see the next test — inside the low edge the
    // price DOES move, for a reason that has nothing to do with Rule 2.
    assert!(spread(&lo_in) < spread(&lo_out) / 10.0);
}

/// **A FINDING THE ORIGINAL MEASUREMENT COULD NOT HAVE SEEN, because it only
/// ever displaced upward.** The freeze is one-sided.
///
/// Displace `solv_1g`'s grain price DOWN by 1% under the same `inelastic` rule
/// and the price does not sit there. It walks *up*, crosses the target, and
/// rests at **+0.87%** on the other side — while the standard deviation over the
/// scored window is still exactly 0.0, because all of the motion happens in the
/// transient.
///
/// The mechanism is not Rule 1 and not Rule 2. Below the zero-profit price the
/// desk's revenue is under its outlay, Rule 3's cash band bites, it rations its
/// own input purchases, labour demand falls, `p_labour` falls with it, and the
/// RATIO rises. Nothing price-elastic is involved: the numeraire moved.
///
/// The consequence for the recorded claim is the important part. **`sd = 0` over
/// a post-transient window is not evidence of "no restoring force"** — it is
/// evidence of "no motion in the window", and this configuration satisfies the
/// second while violating the first. Asserted so that a summary can never again
/// quote the sd alone.
#[test]
fn the_inelastic_freeze_is_one_sided_and_sd_zero_does_not_mean_the_price_stayed() {
    let eq = load_eq("solv_1g");
    let want = target_log(&eq);
    let v = series("solv_1g", PriceRule::Imbalance, SupplyRule::Inelastic, "grain", "labour", 0.99);

    // The window `certify::metrics` and test_12 score, index i = tick i.
    let win = &v[150..=1000];
    assert_eq!(
        welford_sd(win),
        0.0,
        "over the scored window the spread is exactly zero, as the recorded claim says"
    );
    // And yet the price is nowhere near where it was put.
    assert!(
        spread(&v) > 1e-2,
        "the whole series must show real motion; max-min was {:e}",
        spread(&v)
    );
    let ends = v[v.len() - 1] - want;
    assert!(
        ends > 0.0,
        "displaced DOWN 1%, the price must end ABOVE the target — it crossed and rested on the \
         far side. It ended at {:.6}x of target.",
        ends.exp()
    );
    assert!(
        (ends.exp() - 1.008834).abs() < 1e-5,
        "measured 2026-07-31 (third pass): ends at 1.008834x of target, got {:.6}x. A change \
         here is a real change in the cash channel; re-measure and mark the docs.",
        ends.exp()
    );
}

/// **And the drift is mostly the NUMERAIRE, which a ratio alone cannot say.**
///
/// The test above shows the *relative* price ending 0.88% above target after a
/// 1% downward displacement. This one splits it, because "the grain price came
/// back" and "the wage fell" are different economies and the ratio reports them
/// identically.
///
/// Measured at tick 1000, `solv_1g`, `imbalance`, `inelastic`, displaced ×0.99:
/// `p_grain` **0.4950 → 0.499753**, closing **95.1%** of its own 0.005
/// displacement, while `p_labour` **1.0 → 0.990754** — a fall of 0.0092, nearly
/// twice the grain price's remaining error. So the displaced good recovered
/// almost completely and the numeraire went further the other way, under a rule
/// with supply elasticity exactly zero that has no price-response to do it with.
///
/// The channel is Rule 3's cash band, not Rule 1: below zero profit the desk's
/// revenue (0.495 × 20 = 9.9) is under its outlay (10), its balance drains, it
/// rations its own labour purchases, labour demand falls under the 10 hours
/// posted, and the wage follows. Grain then rises because the rationed desk
/// produces less. The pop's cash rises to 30.61 from 30.00 — the mirror image of
/// the drain, and the cheapest available confirmation that this is a cash-flow
/// story and not a price-formation one.
#[test]
fn the_downward_drift_is_the_wage_falling_not_the_displaced_good_failing_to_return() {
    let s = loader::load(&scenario_dir("solv_1g")).expect("loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);
    let id = |g: &str| GoodId(gd.goods.iter().position(|x| x.name == g).expect("good") as u32);
    let (grain, labour) = (id("grain"), id("labour"));

    let p0 = state.price(NODE, grain) * 0.99;
    state.set_price(NODE, grain, p0);
    state.set_price_ema(NODE, grain, p0);
    for _ in 0..TICKS {
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, PriceRule::Imbalance);
    }
    let (pg, pl) = (state.price(NODE, grain), state.price(NODE, labour));

    // The displaced good came back almost all the way on its own.
    assert!(
        (pg - 0.5).abs() < 0.01 * (0.5 - p0).abs() * 10.0 && (pg - 0.5).abs() < 5e-4,
        "p_grain should recover to ~0.499753 from a displaced 0.4950, got {pg}"
    );
    // The numeraire is what did not stay put, and it moved further than the
    // residual in the ratio.
    assert!(
        pl < 0.995,
        "p_labour should have fallen to ~0.990754; got {pl}. If the wage now holds, the cash \
         channel has changed and the one-sided freeze needs re-deriving."
    );
    assert!(
        (0.5 - pg).abs() < (1.0 - pl).abs(),
        "the ratio's residual must be mostly the numeraire: p_grain moved {:.6} from target, \
         p_labour moved {:.6}",
        (0.5 - pg).abs(),
        (1.0 - pl).abs()
    );
}

// ── 2. The return, and what actually stops it ─────────────────────────────────

/// **"Returns to 1.000 exactly" is FALSE at the tape's own registered
/// exactness, and this asserts the failure.**
///
/// `equilibrium.ron` registers `fixed_point_tol_log = 1e-12` and calls it float
/// slack — every number in these tapes is a dyadic rational, so a genuine return
/// to the fixed point is bit-exact. The displaced run does not get there. It
/// stops at `|ln dev| = 1.86e-11`, nineteen times the registered gate, on all
/// three tapes and at every displacement that returns at all.
///
/// That the residual is *identical* across three different worlds — `solv_1g`,
/// `solv_chain`, `solv_1g_money` all read 1.862e-11 at 1.01× — is the tell: it
/// is not a property of any of those economies. The next test says what it is.
#[test]
fn the_reservation_rule_does_not_return_to_the_registered_exactness_and_all_three_tapes_miss_by_the_same_amount() {
    let mut readings = Vec::new();
    for (tape, good) in [("solv_1g", "grain"), ("solv_chain", "flour"), ("solv_1g_money", "grain")]
    {
        let eq = load_eq(tape);
        let want = target_log(&eq);
        let v = series(tape, PriceRule::Imbalance, SupplyRule::Reservation, good, "labour", 1.01);
        let dev = (v[v.len() - 1] - want).abs();
        assert!(
            dev > eq.fixed_point_tol_log,
            "{tape}: the displaced price now returns to within the registered \
             fixed_point_tol_log of {:.1e} (measured {dev:.4e}). That would mean the 1e-12 \
             absolute guard in src/systems/price_update/mod.rs no longer binds — a REAL \
             improvement. Write it up in docs/PLAN.md; do not delete this assertion.",
            eq.fixed_point_tol_log
        );
        // It does come back a very long way, and that half of the claim stands.
        assert!(dev < 1e-9, "{tape}: it should still return to ~1e-11, got {dev:.4e}");
        readings.push(dev);
    }
    let (lo, hi) = (
        readings.iter().cloned().fold(f64::INFINITY, f64::min),
        readings.iter().cloned().fold(0.0f64, f64::max),
    );
    assert!(
        hi / lo < 1.01,
        "three different economies must all stall at the same residual if the stall is \
         numerical rather than economic: got {readings:?}"
    );
}

/// **WHAT STOPS THE RETURN IS A CONSTANT IN THE SOURCE, AND IT BREAKS MONEY
/// NEUTRALITY UNDER DISPLACEMENT.**
///
/// `src/systems/price_update/mod.rs:28` emits a price delta only when
/// `|p_next − p| > 1e-12` — an **absolute** threshold, in currency units, with
/// behavioural meaning, registered nowhere (R2). Near the fixed point the
/// per-tick step is `p·α·|imbalance|` and `imbalance ≈ −2e` in the log price
/// error, so the run stalls at `e ≈ 1e-12/(2·α·p)`. At `solv_1g_money`'s
/// `p_grain = 0.5` and `α = 0.05` that predicts `2.0e-11`; measured 1.86e-11.
///
/// The prediction that makes it a mechanism rather than a coincidence:
/// `solv_1g_money_2x` is the SAME real economy at exactly twice the price level.
/// Double `p` and the same algebra halves `e`. Measured: **1.8617e-11 at 1x and
/// 9.8936e-12 at 2x** — a ratio of 1.882, against a predicted 2.
///
/// So a redenomination changed a real outcome. `money_is_exactly_neutral_…` in
/// `test_12_solvable.rs` cannot see this and is not wrong: it runs both tapes AT
/// the fixed point, where no price ever drifts and the guard never binds. It is
/// only off the fixed point that the constant becomes visible, and being off the
/// fixed point is the entire subject of this file.
///
/// Asserted as a FAILURE. Removing the guard, or making it relative, is a
/// mechanism change with its own A/B — not a patch to smuggle in beside a
/// measurement.
#[test]
fn the_return_stalls_on_an_absolute_constant_so_the_same_economy_at_2x_converges_twice_as_close() {
    let want = target_log(&load_eq("solv_1g_money"));
    let one =
        series("solv_1g_money", PriceRule::Imbalance, SupplyRule::Reservation, "grain", "labour", 1.01);
    let two = series(
        "solv_1g_money_2x",
        PriceRule::Imbalance,
        SupplyRule::Reservation,
        "grain",
        "labour",
        1.01,
    );
    let (d1, d2) = ((one[one.len() - 1] - want).abs(), (two[two.len() - 1] - want).abs());
    assert!(
        d1 > 0.0 && d2 > 0.0,
        "both must stall short of the target for the comparison to mean anything: {d1:e}, {d2:e}"
    );
    let ratio = d1 / d2;
    assert!(
        ratio > 1.5,
        "a redenomination must change the residual — that is the R2/R12 violation this test \
         exists to record. 1x stalled at {d1:.4e}, 2x at {d2:.4e}, ratio {ratio:.3} (predicted \
         2.0 from the absolute 1e-12 guard). If this ratio is now ~1.0 the guard has been made \
         relative or removed, which is a real fix — record it in docs/PLAN.md and re-derive \
         this test rather than deleting it."
    );
    // The predicted stall itself, from the guard and the dials, not from a fit.
    let predicted = 1e-12 / (2.0 * 0.05 * 0.5);
    assert!(
        d1 < predicted && d1 > predicted / 2.0,
        "the 1x residual {d1:.4e} should sit just under the derived stall {predicted:.4e}"
    );
}

// ── 3. The basin ──────────────────────────────────────────────────────────────

/// **The basin is not an interval, so "the basin is X%" is not a sentence this
/// engine supports.**
///
/// Under `imbalance/reservation` on `solv_1g` the connected set of
/// displacements around the fixed point whose deviation *shrinks* runs from
/// about `0.936×` to about `1.046×` — measured by
/// `examples/restoring_force.rs`, which bisects the first crossing outward from
/// `f = 1` in log-factor space.
///
/// But the predicate is **not monotone**: displacements well beyond that
/// boundary also shrink — `1.2311×`, `1.4389×`, `2.5491×` among them — while
/// `1.05×` does not. Those are separate attractors, not a wider basin, and
/// averaging them into one number would hide the structure entirely.
///
/// This test asserts the non-monotonicity, because it is the finding: a single
/// "basin width" reported for this mechanism would be wrong in kind, not just
/// in value.
#[test]
fn the_set_of_displacements_that_come_back_is_not_an_interval() {
    let eq = load_eq("solv_1g");
    let want = target_log(&eq);
    let shrank = |f: f64| -> bool {
        let v = series("solv_1g", PriceRule::Imbalance, SupplyRule::Reservation, "grain", "labour", f);
        let d0 = (v[0] - want).abs();
        let d1 = (v[v.len() - 1] - want).abs();
        d1.is_finite() && d0 > 0.0 && d1 < d0
    };
    // Inside the connected component.
    assert!(shrank(1.02), "1.02x must contract — it is inside the component");
    // The first failure outward.
    assert!(!shrank(1.05), "1.05x must NOT contract; that is where the component ends");
    // And an island beyond it. If this ever contracts monotonically the map has
    // changed character and the basin can be reported as an interval again —
    // which would be a real result, and must be written up rather than absorbed.
    assert!(
        shrank(1.4389),
        "1.4389x contracted in the 2026-07-31 scan while 1.05x did not — that non-monotonicity \
         is why no single basin width is reported. If it has gone, re-measure the scan in \
         examples/restoring_force.rs and mark docs/PLAN.md."
    );
}

/// **`solv_1g` is not knife-edge, and neither is it representative.**
///
/// The same +1% displacement behaves identically on all three tapes: bit-frozen
/// under `inelastic`, back to 1.86e-11 under `reservation`. So the headline is
/// not an artefact of the one-good world.
///
/// What is NOT shared is everything else. On the downward side and at larger
/// displacements the three worlds diverge completely — `solv_chain` displaced
/// to `0.9524×` under `inelastic` runs its relative price to **1.6e20** and
/// trades on 6% of the window, where `solv_1g` under the same displacement sits
/// quietly at `1.038×` with its markets alive throughout. Recorded here so the
/// shared headline is never read as "the family agrees".
#[test]
fn the_headline_transfers_to_the_other_tapes_and_nothing_else_does() {
    let eq1 = load_eq("solv_1g");
    let eqc = load_eq("solv_chain");
    let a = series("solv_1g", PriceRule::Imbalance, SupplyRule::Inelastic, "grain", "labour", 0.9524);
    let b = series("solv_chain", PriceRule::Imbalance, SupplyRule::Inelastic, "flour", "labour", 0.9524);
    let da = (a[a.len() - 1] - target_log(&eq1)).abs();
    let db = (b[b.len() - 1] - target_log(&eqc)).abs();
    assert!(
        da < 0.1,
        "solv_1g at 0.9524x should rest near its target (measured 1.038x): {:.4}x",
        da.exp()
    );
    assert!(
        db > 10.0,
        "solv_chain at 0.9524x should run away (measured 1.6e20x): {:.4e}x. The two worlds do \
         NOT behave alike off the +1% row, and a summary that says they do is wrong.",
        db.exp()
    );
}
