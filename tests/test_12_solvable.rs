//! **The economics unit test this project never had.**
//!
//! Every other test in this suite asks whether the engine held together —
//! conserved, deterministic, replayable, inside a band. None of them asks
//! whether the numbers it produced are *right*, because until 2026-07-31 no
//! scenario in the corpus had a right answer: `tracer_2r`'s own header says it
//! is "not to be economics", and `lr_*` is a factorial search grid built to look
//! *for* a stable region, not 24 worlds each expected to work.
//!
//! `data/scenarios/solv_*` are five worlds whose competitive equilibrium — price
//! vector, quantity vector, money stock and metric series — is written down in
//! closed form, by hand, from the tape, in `docs/design/solvable-scenarios.md`
//! and in each tape's own header. `equilibrium.ron` beside each tape is the
//! machine-readable copy. This file runs them and compares.
//!
//! # The two modes
//!
//! **Mode A — stationarity.** Genesis *is* the equilibrium, so the claim is that
//! the price vector never moves. The tolerance is `1e-12` in logs and it is
//! FLOAT slack, not economic slack: every number in these tapes is a dyadic
//! rational, so the fixed point is bit-exact in f64 and every rule's response at
//! it is *identically* zero. Any motion at all is a defect — in the engine or in
//! the derivation, and either is worth knowing. Mode A is invariant to `alpha`
//! and to the price rule by construction (a market with `d == s` is stationary
//! under both), so running it under both rules is also a check on the harness.
//!
//! **Mode B — attraction.** Genesis displaced by doubling one registered price.
//! The tolerance is `ln(1.10)`, derived from dials and not from a result:
//! `dead = 0.05` makes the rest *set* `revenue/cost ∈ [0.951, 1.051]`, and the
//! state variable moves multiplicatively once per activation so it can sit one
//! `eta_dn = 0.05` step outside; `1.05 × 1.05 = 1.1025`.
//!
//! # What was found, 2026-07-31
//!
//! **Mode A passes exactly, everywhere, under the desk kernel.** All five tapes,
//! both price rules, worst deviation `0.000e0` — not "small", zero. The kernel
//! reproduces a hand-computed competitive equilibrium bit-for-bit, including
//! `solv_chain`'s two-stage price vector and `solv_labour`'s interior labour
//! market. That is the first time anything in this repository has been shown to
//! put a price in the right place.
//!
//! **Mode B fails on every configuration**, and not by a near miss: displaced by
//! 2× on one good, every world's traded volume decays monotonically across the
//! scored window while its relative price wanders by factors of 10 to 1e20. The
//! fixed point exists and is exact; nothing attracts to it. Those numbers are
//! asserted below with the measurements in the panic messages, because **a
//! recorded finding is worth more than a green tick** — and because widening a
//! registered bar to make a test pass is the one thing this project treats as
//! unforgivable. Five tests below assert FAILURES or falsification controls and
//! say so in their names.
//!
//! # The rule this file is written under
//!
//! **There is not one tolerance in this file.** Every bar is read from the
//! scenario's `equilibrium.ron`, registered before the engine was ever pointed
//! at these tapes. The only comparisons this file makes on its own authority are
//! threshold-free ones — "did the traded volume shrink across the window", which
//! is a direction and not a level.

use std::collections::HashMap;
use std::path::PathBuf;

use rustyecon::{
    certify::{verdict::level_range, Criteria},
    kernel::AgentArm,
    state::game_data::SupplyRule,
    scenario::{equilibrium::Equilibrium, loader},
    systems::{clearing::PriceRule, run_tick},
    types::ids::{GoodId, MarketNodeId},
};

/// The horizon every solvable tape's `criteria.ron` registers as `analysis_end`.
const TICKS: u64 = 1000;

/// These worlds have exactly one region, so exactly one market node.
const NODE: MarketNodeId = MarketNodeId(0);

/// Every tape in the family, in the order the report should read.
const FAMILY: [&str; 5] =
    ["solv_1g", "solv_1g_money", "solv_1g_money_2x", "solv_chain", "solv_labour"];

/// The four that register a mode-B displacement. `solv_1g_money_2x` is a control
/// whose only job is to be compared against `solv_1g_money` from the same fixed
/// point, so displacing it would introduce a second difference and destroy that.
const DISPLACEABLE: [&str; 4] = ["solv_1g", "solv_1g_money", "solv_chain", "solv_labour"];

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios").join(name)
}

fn load_eq(name: &str) -> Equilibrium {
    Equilibrium::load(&scenario_dir(name))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
        .unwrap_or_else(|| panic!("{name} registers no equilibrium.ron"))
}

// ── running a tape and recording what it did ─────────────────────────────────

/// One tick's worth of everything the comparisons below need.
///
/// Recorded per tick rather than summarised inside the loop, because a summary
/// cannot be re-asked a different question when a result turns out to be
/// surprising — and it turned out to be surprising twice while this was written.
struct Frame {
    /// Posted price of every good at node 0, indexed by `GoodId`.
    price: Vec<f64>,
    /// `min(supply, demand)` per good — what actually changed hands.
    cleared: Vec<f64>,
    /// Every inventory's holding of every good, `[inventory][good]`.
    inventory: Vec<Vec<f64>>,
    /// `chosen_size` of every recipe instance.
    scale: Vec<f64>,
    wealth: Vec<f64>,
    participation: Vec<f64>,
}

struct Sim {
    goods: Vec<String>,
    currency: Option<GoodId>,
    frames: Vec<Frame>,
    criteria: Criteria,
}

impl Sim {
    fn good_id(&self, name: &str) -> GoodId {
        GoodId(
            self.goods
                .iter()
                .position(|g| g == name)
                .unwrap_or_else(|| panic!("no good named {name:?} in {:?}", self.goods))
                as u32,
        )
    }

    /// Indices into `frames` covering the criteria's analysis window. Frame `i`
    /// is tick `i + 1`, matching how `certify::metrics` indexes its series, so a
    /// number here and a number in a certificate are taken over the same ticks.
    fn window(&self) -> std::ops::Range<usize> {
        let lo = self.criteria.transient.saturating_sub(1) as usize;
        let hi = (self.criteria.analysis_end as usize).min(self.frames.len());
        lo..hi
    }

    /// `ln(p_good / p_numeraire)` on every tick.
    fn log_ratio(&self, good: GoodId, numeraire: GoodId) -> Vec<f64> {
        self.frames
            .iter()
            .map(|f| (f.price[good.idx()] / f.price[numeraire.idx()]).ln())
            .collect()
    }
}

/// Run one tape to `TICKS`, optionally displacing one genesis price first.
///
/// The displacement is applied to the loaded state rather than to a second tape
/// on disk, and the reason is R6 rather than convenience: `equilibrium.ron`
/// registers WHICH good and BY WHAT FACTOR, so the perturbation is data written
/// down before the run, and a second directory would have been a second place
/// for it to drift out of step with its own target.
fn simulate(name: &str, arm: AgentArm, rule: PriceRule, displace: Option<(&str, f64)>) -> Sim {
    // No supply rule named: run whatever the tape registered. Every existing
    // result in this file was measured that way and stays comparable.
    simulate_with(name, arm, rule, displace, None)
}

/// The same run with `kernel.supply_rule` overridden.
///
/// Split out rather than threaded through every call site so that the readings
/// taken before the switch existed are demonstrably unchanged: `simulate` still
/// means "as the tape says".
fn simulate_with(
    name: &str,
    arm: AgentArm,
    rule: PriceRule,
    displace: Option<(&str, f64)>,
    supply: Option<SupplyRule>,
) -> Sim {
    let dir = scenario_dir(name);
    let s = loader::load(&dir).unwrap_or_else(|e| panic!("{name} loads: {e}"));
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    if let Some(sr) = supply {
        gd.kernel.supply_rule = sr;
    }
    let criteria = s.criteria.unwrap_or_else(|| panic!("{name} registers criteria.ron"));
    let goods: Vec<String> = gd.goods.iter().map(|g| g.name.clone()).collect();
    let currency = gd.market_node(NODE).currency_good;

    if let Some((good, factor)) = displace {
        let id = GoodId(
            goods
                .iter()
                .position(|g| g == good)
                .unwrap_or_else(|| panic!("{name}: no good {good:?}")) as u32,
        );
        for n in 0..gd.num_nodes() {
            let node = MarketNodeId(n as u32);
            let p = state.price(node, id) * factor;
            state.set_price(node, id, p);
            // The EMA is a smoothed copy of the price and nothing behavioural
            // reads it (only telemetry and the NaN scan do), but leaving it at
            // the undisplaced value would seed a phantom transient into a series
            // somebody later plots. Moved with the price it summarises.
            state.set_price_ema(node, id, p);
        }
    }

    let mut frames = Vec::with_capacity(TICKS as usize);
    for _ in 0..TICKS {
        run_tick(&mut state, &gd, &events, arm, rule);
        frames.push(Frame {
            price: (0..gd.num_goods()).map(|g| state.price(NODE, GoodId(g as u32))).collect(),
            cleared: (0..gd.num_goods())
                .map(|g| {
                    let good = GoodId(g as u32);
                    state.supply(NODE, good).min(state.demand(NODE, good))
                })
                .collect(),
            inventory: state
                .inventories
                .iter()
                .map(|inv| (0..gd.num_goods()).map(|g| inv.get(GoodId(g as u32))).collect())
                .collect(),
            scale: state.recipe_instances.iter().map(|ri| ri.chosen_size).collect(),
            wealth: state.pop_groups.iter().map(|p| p.wealth).collect(),
            participation: state.pop_groups.iter().map(|p| p.participation).collect(),
        });
    }

    Sim { goods, currency, frames, criteria }
}

// ── comparing a run against its registered target ────────────────────────────

/// Deterministic iteration order over a target vector, with the numeraire
/// dropped.
///
/// Hash order would make "which good was worst" depend on the allocator's mood.
/// The numeraire is dropped because `ln(p_num/p_num) = 0` identically — it is
/// not a measurement, and leaving it in would pad the worst case with a zero and
/// add a column to the per-good list that can never say anything. B8 excludes
/// labour from its own summary for the same reason.
fn targets(prices: &HashMap<String, f64>, numeraire: &str) -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = prices
        .iter()
        .filter(|(k, _)| k.as_str() != numeraire)
        .map(|(k, p)| (k.clone(), *p))
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

/// What a comparison found, per good and overall.
struct Gap {
    worst_good: String,
    worst: f64,
    per_good: Vec<(String, f64)>,
}

impl Gap {
    fn of(per_good: Vec<(String, f64)>) -> Self {
        let (worst_good, worst) = per_good
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).expect("finite"))
            .cloned()
            .expect("at least one non-numeraire good is registered");
        Gap { worst_good, worst, per_good }
    }

    fn render(&self) -> String {
        let list: Vec<String> =
            self.per_good.iter().map(|(g, v)| format!("{g} {:.4}x", v.exp())).collect();
        format!(
            "worst {} |ln|={:.3e} ({:.4}x) [{}]",
            self.worst_good,
            self.worst,
            self.worst.exp(),
            list.join(", ")
        )
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    if v.is_empty() {
        return f64::NAN;
    }
    v[v.len() / 2]
}

/// Mode A: the largest deviation from the target vector at ANY tick.
///
/// Max, not median, and that is the whole difference between the modes: the
/// claim here is that nothing moved, and a median would let a run wander off and
/// come back without being noticed.
fn mode_a_gap(sim: &Sim, eq: &Equilibrium) -> Gap {
    let num = sim.good_id(&eq.numeraire);
    Gap::of(
        targets(&eq.prices, &eq.numeraire)
            .into_iter()
            .map(|(good, target)| {
                let want = target.ln();
                let worst = sim
                    .log_ratio(sim.good_id(&good), num)
                    .into_iter()
                    .map(|got| (got - want).abs())
                    .fold(0.0f64, f64::max);
                (good, worst)
            })
            .collect(),
    )
}

/// Mode B: the MEDIAN deviation over the scored window, per good.
///
/// Median of the SIGNED deviation, then its magnitude — not the median of the
/// magnitude. A price orbiting symmetrically about the right answer *is* right
/// on average, and taking `|·|` first would score that orbit as a persistent
/// error equal to its own amplitude.
///
/// **This statistic is generous by construction, and one result below proves
/// it:** `solv_labour` under the imbalance rule reads a median gap of 1.006×
/// while its real wage swings across a band of 8.1× and its traded volume falls
/// to a seventh of target. A median alone cannot tell a converged run from a symmetric
/// orbit, which is why every reading is reported beside the band and the volume.
fn mode_b_gap(sim: &Sim, eq: &Equilibrium) -> Gap {
    let num = sim.good_id(&eq.numeraire);
    let w = sim.window();
    Gap::of(
        targets(&eq.prices, &eq.numeraire)
            .into_iter()
            .map(|(good, target)| {
                let want = target.ln();
                let devs: Vec<f64> = sim.log_ratio(sim.good_id(&good), num)[w.clone()]
                    .iter()
                    .map(|g| g - want)
                    .collect();
                (good, median(devs).abs())
            })
            .collect(),
    )
}

/// The DRIFTING statistic (95th/5th-percentile band) on the real wage over the
/// scored window — the same number `criteria.ron` scores, reported here beside
/// the correctness reading so the two can be read together.
fn real_wage_band(sim: &Sim) -> f64 {
    let labour = sim.good_id(&sim.criteria.labour_good);
    let staple = sim.good_id(&sim.criteria.staple_good);
    let w = sim.window();
    let series: Vec<f64> =
        sim.frames[w].iter().map(|f| f.price[labour.idx()] / f.price[staple.idx()]).collect();
    level_range(&series)
}

/// **Did the market stay alive?** Mean cleared volume over the last tenth of the
/// scored window divided by the mean over the first tenth, minimised over the
/// goods the tape registers a cleared target for.
///
/// This is the one comparison in this file not read from a registered file, and
/// it is deliberately THRESHOLD-FREE: it asks for a *direction*, not a level.
/// Below 1 the world is trading less at the end of the window than at the start;
/// at exactly 1 nothing changed. No constant is invented, so there is nothing
/// here to tune.
///
/// It exists because of what mode B turned up: a price can sit on its target
/// while the economy underneath it evaporates, and a correctness test reporting
/// only the price would have called that a pass.
fn volume_trend(sim: &Sim, eq: &Equilibrium) -> (String, f64) {
    let w = sim.window();
    let n = (w.end - w.start) / 10;
    let mean = |lo: usize, hi: usize, g: GoodId| -> f64 {
        sim.frames[lo..hi].iter().map(|f| f.cleared[g.idx()]).sum::<f64>() / (hi - lo) as f64
    };
    let mut goods: Vec<&String> = eq.cleared.keys().collect();
    goods.sort();
    let mut worst = (String::new(), f64::INFINITY);
    for good in goods {
        let id = sim.good_id(good);
        let first = mean(w.start, w.start + n, id);
        let last = mean(w.end - n, w.end, id);
        // A market already dead at the start of the window has no trend; report
        // it as fully collapsed rather than as 0/0.
        let ratio = if first > 0.0 { last / first } else { 0.0 };
        if ratio < worst.1 {
            worst = (good.clone(), ratio);
        }
    }
    worst
}

/// How far the run's volume sits from the registered target, for the report.
fn volume_level(sim: &Sim, eq: &Equilibrium) -> String {
    let w = sim.window();
    let half = w.start + (w.end - w.start) / 2;
    let mut goods: Vec<(&String, &f64)> = eq.cleared.iter().collect();
    goods.sort_by_key(|(g, _)| (*g).clone());
    goods
        .into_iter()
        .map(|(good, target)| {
            let id = sim.good_id(good);
            let mean: f64 = sim.frames[half..w.end].iter().map(|f| f.cleared[id.idx()]).sum::<f64>()
                / (w.end - half) as f64;
            format!("{good} {:.3}", mean / target)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// One line per configuration, in the same shape for every scenario, so the
/// table in a run log can be read across rows.
fn report(tag: &str, gap: &Gap, sim: &Sim, eq: &Equilibrium) {
    let (good, trend) = volume_trend(sim, eq);
    eprintln!(
        "  {tag:<32} {}\n      realwage band {:.4}x | cleared/target {} | volume trend {good} {:.4}x",
        gap.render(),
        real_wage_band(sim),
        volume_level(sim, eq),
        trend
    );
}

// ── MODE A: does the engine sit still at a hand-computed fixed point? ─────────

/// **The headline claim of the whole family.** Every solvable tape, under the
/// desk kernel, under both price rules, holds its hand-computed equilibrium.
///
/// Stated as one test rather than five so a partial pass cannot be quoted as a
/// pass, and asserted on the *whole price vector* rather than on `RealWage`,
/// because `solv_chain` can reach the right real wage with the wrong
/// intermediate price and that is exactly what a one-good world cannot see.
///
/// Measured 2026-07-31: worst deviation `0.000e0` on all ten configurations.
/// Not "within tolerance" — identically zero, which is what a dyadic-rational
/// fixed point should give and what nothing in this repo had ever demonstrated.
#[test]
fn the_kernel_holds_every_hand_computed_fixed_point_under_both_price_rules() {
    eprintln!("\nMODE A — genesis IS the equilibrium; the claim is that nothing moves.");
    for name in FAMILY {
        let eq = load_eq(name);
        for rule in [PriceRule::Imbalance, PriceRule::Ratio] {
            let sim = simulate(name, AgentArm::Kernel, rule, None);
            let gap = mode_a_gap(&sim, &eq);
            report(&format!("{name}/kernel/{}", rule.name()), &gap, &sim, &eq);
            assert!(
                gap.worst <= eq.fixed_point_tol_log,
                "{name} under kernel/{}: the price vector MOVED off a fixed point that is \
                 exact in f64. Worst |ln(realised/target)| = {:.6e} on {}, against the \
                 registered {:.1e}. Either a rule responded to a zero signal or the \
                 derivation in {} is wrong — do not widen this bar to find out which.",
                rule.name(),
                gap.worst,
                gap.worst_good,
                eq.fixed_point_tol_log,
                eq.derivation,
            );
            // The independent half. Prices sitting still is also what a world
            // that had quietly stopped trading would report, and mode B below
            // shows that failure mode is real rather than hypothetical.
            let (good, trend) = volume_trend(&sim, &eq);
            assert!(
                trend >= 1.0,
                "{name} under kernel/{}: prices held but {good}'s traded volume fell to \
                 {trend:.4}x across the scored window. A correct price on a dying market is \
                 not a correct economy.",
                rule.name(),
            );
        }
    }
}

/// **The falsification control for mode A.** A gate nobody has watched fire is
/// not a gate.
///
/// The test above passes with a deviation of exactly zero, which is suspicious
/// on its face — so here is the same code FAILING on a defect deliberately
/// introduced into a world that is otherwise identical. The defect is the
/// smallest one that is still economically meaningful: the producer's genesis
/// stock is moved by one part in a million, so Rule 1 posts marginally more than
/// the pop demands, the imbalance is no longer exactly zero, and the price rule
/// starts to walk.
///
/// If this test ever passes — the perturbed world also sitting still — then the
/// mode-A test above is measuring nothing and both must be re-derived.
#[test]
fn mode_a_fires_on_a_one_part_in_a_million_perturbation() {
    let eq = load_eq("solv_1g");
    let s = loader::load(&scenario_dir("solv_1g")).expect("solv_1g loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);
    let (grain, labour) = (GoodId(0), GoodId(2));

    // 60.0 -> 60.00006. Rule 1 posts `stock - b_out*flow`, so the surplus rises
    // by exactly 6e-5 against a demand of 20 — an imbalance of 3e-6, thirteen
    // orders of magnitude above the gate and still invisible to every stability
    // criterion in the repo.
    let inv = state.buildings[0].inventory;
    state.inventory_mut(inv).add(grain, 6e-5, None);

    let want: f64 = eq.prices["grain"].ln();
    let mut worst = 0.0f64;
    for _ in 0..TICKS {
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, PriceRule::Imbalance);
        worst =
            worst.max(((state.price(NODE, grain) / state.price(NODE, labour)).ln() - want).abs());
    }
    eprintln!(
        "\nFALSIFICATION (mode A) — solv_1g with 6e-5 extra genesis grain: worst |ln| = {worst:.3e}"
    );
    assert!(
        worst > eq.fixed_point_tol_log,
        "the mode-A gate did not fire on a genuinely perturbed world (worst {worst:.3e} vs \
         tolerance {:.1e}). The clean pass above is therefore not evidence of anything.",
        eq.fixed_point_tol_log
    );
}

/// **A RECORDED FINDING, NOT A BUG TO BE SILENCED.**
///
/// Under `--agents legacy` mode A cannot be exact, and the reason is structural
/// rather than incidental: `building_agent.rs` caps posted supply at
/// `1.2 × demand` read from *last tick*, which is 0 at genesis, so the legacy
/// arm posts nothing at all on tick 0 and every market opens at
/// `imbalance = +1`. The world then has to find its way back, and where it lands
/// is what this records. Measured 2026-07-31, worst `|ln(realised/target)|` over
/// 1000 ticks under the imbalance rule:
///
/// ```text
///   solv_1g        1.48  (4.4x)        solv_chain    3.02  (20.5x)
///   solv_1g_money  1.51  (4.5x)        solv_labour  20.62  (9.1e8x)
/// ```
///
/// **And a contrast worth more than either column on its own:** the legacy arm
/// keeps its markets ALIVE while getting the prices wrong — cleared volume runs
/// at 0.98–1.00 of target on every tape — whereas the displaced kernel (mode B
/// below) gets far closer on price and lets the market die. Neither arm is
/// simply "better"; they fail differently, and no single-number A/B could have
/// said so.
///
/// `solv_labour` is the interesting row: the legacy layer has no participation
/// margin at all, so it posts every hour unconditionally against a
/// capacity-capped demand, the labour imbalance pins negative forever, and the
/// wage decays geometrically. That was PRE-REGISTERED in that tape's header
/// before the run, and B7 catches the pin independently — `node0/labour pinned
/// at -0.403032 for all 851 ticks`. The header predicted the pin at -0.5 from a
/// hand calculation; the measured -0.403 is the same phenomenon with the legacy
/// layer's actual posting, and the geometric decay it implies matches the
/// measured price to within the window's own spread.
///
/// This test asserts the FAILURE. If the legacy arm ever holds a fixed point it
/// breaks, and that is correct — it would be a real change in the legacy layer
/// and must not be absorbed silently by a test that only ever demanded "legacy
/// is bad".
#[test]
fn the_legacy_arm_cannot_hold_a_fixed_point_and_here_is_how_far_it_moves() {
    eprintln!("\nMODE A under --agents legacy — asserted to FAIL, measured rather than assumed.");
    for name in DISPLACEABLE {
        let eq = load_eq(name);
        let sim = simulate(name, AgentArm::Legacy, PriceRule::Imbalance, None);
        let gap = mode_a_gap(&sim, &eq);
        report(&format!("{name}/legacy/imbalance"), &gap, &sim, &eq);
        assert!(
            gap.worst > eq.fixed_point_tol_log,
            "{name} under legacy now HOLDS the fixed point to {:.3e}. That would be a real \
             improvement in the legacy agent layer and it must be written up, not absorbed \
             by flipping this assertion.",
            gap.worst
        );
    }
}

// ── MODE B: does a displaced world come back? It does not. ───────────────────

/// **THE CENTRAL NEGATIVE RESULT, ASSERTED RATHER THAN NARRATED.**
///
/// Displace one registered price by 2× and every configuration's traded volume
/// decays across the scored window. Measured 2026-07-31 as last-tenth over
/// first-tenth of the window, where 1.0 would mean no decay:
///
/// ```text
///                        imbalance   ratio
///   solv_1g                 0.068    0.000
///   solv_1g_money           0.356    0.000
///   solv_chain              0.000    0.000
///   solv_labour             0.204    0.000
/// ```
///
/// Every one of the eight is below 1, and all four `ratio` columns are markets
/// that traded exactly nothing for the last 800 ticks.
///
/// The fixed point is exact (mode A) and nothing attracts to it. **That is a
/// sharper statement of PLAN Phase 4's central finding than the elasticity probe
/// could make**: it is not only that the price rule chases a crossing that does
/// not exist, it is that even where a crossing provably *does* exist and is
/// sitting one factor of two away, the mechanism cannot walk to it.
///
/// The `ratio` rule is the more decisive of the two and its mechanism is
/// visible in the trace: `p ← p·d/s` against supply posted from stock has no
/// damping term at all, and Rule 1's buffer makes the feedback *positive* — a
/// desk that expands by `g` posts `b_out·g` LESS this tick while it refills its
/// band, so supply falls, `d/s` rises, the price rises, the margin widens and it
/// expands again. This is design-note prediction 7.1
/// (`docs/design/solvable-scenarios.md`) arriving as a measurement rather than
/// as a linearisation.
///
/// This test asserts the collapse. If a configuration ever holds its volume it
/// fails here, which is the intended behaviour: the improvement gets written up
/// rather than silently absorbed.
///
/// **PARTLY SUPERSEDED 2026-07-31 (R14), and by exactly the route this comment
/// asked for.** "No configuration recovers" is still true of everything this
/// test runs, because `simulate` runs the tape's registered
/// `kernel.supply_rule`, which is `inelastic`. It is NOT true in general: with
/// the reservation band on storable outputs, `solv_1g`/`imbalance` runs its
/// volume at 1.134x across the window. See
/// `mode_b_under_the_price_responsive_rule_beside_the_shipped_one` below, which
/// is where that reading is asserted. Left standing here as the baseline it is.
#[test]
fn every_displaced_world_loses_its_market_and_here_is_how_fast() {
    eprintln!("\nMODE B — genesis displaced 2x; RECORDED FINDING: no configuration recovers.");
    for name in DISPLACEABLE {
        let eq = load_eq(name);
        let d = eq.displaced.clone().expect("these four register a displacement");
        for rule in [PriceRule::Imbalance, PriceRule::Ratio] {
            let sim = simulate(name, AgentArm::Kernel, rule, Some((&d.good, d.factor)));
            let gap = mode_b_gap(&sim, &eq);
            report(&format!("{name}/kernel/{}", rule.name()), &gap, &sim, &eq);
            let (good, trend) = volume_trend(&sim, &eq);
            assert!(
                trend < 1.0,
                "{name} under kernel/{}: displaced by {}x on {} and the market SURVIVED — \
                 {good}'s volume ran at {trend:.4}x across the window instead of decaying. \
                 That contradicts the finding recorded here on 2026-07-31 and is a real \
                 result; write it up rather than deleting this assertion.",
                rule.name(),
                d.factor,
                d.good,
            );
        }
    }
}

/// **The price half of the same failure**, kept separate because the two say
/// different things and one of them is a trap.
///
/// Seven of the eight mode-B configurations miss the registered `ln(1.10)`, by
/// factors from 1.9× to 1.5e11×. The eighth — `solv_labour` under the imbalance
/// rule — reads a *median* gap of 1.006× and would have been reported as a clean
/// convergence by a test that stopped there. It is not converged: its real wage
/// swings across a band of 8.1× and its volume falls to a seventh of target. The median is
/// small because the orbit happens to be symmetric about the right answer, which
/// is exactly the artefact `mode_b_gap`'s doc comment warns about.
///
/// So this asserts what was measured, and names the one exception rather than
/// hiding it inside a loop that passes.
#[test]
fn seven_of_eight_displaced_configurations_miss_the_registered_price_target() {
    let mut inside: Vec<String> = Vec::new();
    let mut missed = 0usize;
    for name in DISPLACEABLE {
        let eq = load_eq(name);
        let d = eq.displaced.clone().expect("registered");
        for rule in [PriceRule::Imbalance, PriceRule::Ratio] {
            let sim = simulate(name, AgentArm::Kernel, rule, Some((&d.good, d.factor)));
            let gap = mode_b_gap(&sim, &eq);
            if gap.worst <= d.tol_log {
                inside.push(format!(
                    "{name}/{} (median {:.4}x, realwage band {:.2}x)",
                    rule.name(),
                    gap.worst.exp(),
                    real_wage_band(&sim)
                ));
            } else {
                missed += 1;
            }
        }
    }
    eprintln!("\nMODE B price target: {missed}/8 miss ln(1.10); inside on median: {inside:?}");
    assert_eq!(
        missed, 7,
        "the mode-B price outcome changed. Recorded 2026-07-31: 7 of 8 configurations miss \
         the registered ln(1.10), and the one that does not — solv_labour under the imbalance \
         rule — is inside on the MEDIAN only, while its real-wage band is 8.1x and its volume \
         decays. Configurations now inside: {inside:?}. If this improved, say so in \
         docs/PLAN.md and re-register; if it worsened, the same. DO NOT change the tolerance: \
         it is registered in equilibrium.ron and derived from dead=0.05 and eta_dn=0.05, not \
         from any measurement."
    );
    assert!(
        inside.iter().any(|c| c.starts_with("solv_labour/imbalance")),
        "the one configuration inside the median tolerance used to be solv_labour/imbalance; \
         it is now {inside:?}. Which one it is matters — that is the scenario with the \
         interior labour-market crossing — so this is not interchangeable bookkeeping."
    );
}

// ── The money-neutrality control (R2's sharpest audit) ───────────────────────

/// `solv_1g_money` and `solv_1g_money_2x` are the same world at two price
/// levels. Every kernel rule is a function of ratios — dead-bands on `sigma` and
/// `sigma_c`, clamps on `pi` and `scale/recipe_size`, `cash/reserve`, the ration
/// `budget/want` — so the real economy must be bit-identical and every price
/// exactly `2x`.
///
/// A failure means an absolute constant with behavioural meaning has leaked into
/// the source: a number that means one thing when a loaf costs 0.5 and another
/// when it costs 1.0. That is the R2 violation the rule exists to prevent, and
/// the assertion names the good and the tick.
///
/// Measured 2026-07-31: exact across all 1000 ticks, to 0 ulp. The one absolute
/// currency constant known to exist — `overflow > 1e-12` in
/// `rule_3_buy_and_route` — cannot bind here because the overflow is identically
/// 0.0 in both tapes; that is recorded in the 2x tape's header so a future
/// failure is diagnosed rather than re-derived.
#[test]
fn money_is_exactly_neutral_between_the_1x_and_2x_tapes() {
    let one = simulate("solv_1g_money", AgentArm::Kernel, PriceRule::Imbalance, None);
    let two = simulate("solv_1g_money_2x", AgentArm::Kernel, PriceRule::Imbalance, None);
    assert_eq!(one.goods, two.goods, "the two tapes must declare the same goods");
    let currency = one.currency.expect("the tape registers a currency");

    for (t, (a, b)) in one.frames.iter().zip(&two.frames).enumerate() {
        let tick = t + 1;
        for g in 0..one.goods.len() {
            let name = &one.goods[g];
            // Prices double, except the currency's own, which is a unit label
            // rather than a traded price and reads 1.0 in both tapes.
            let want = if GoodId(g as u32) == currency { a.price[g] } else { 2.0 * a.price[g] };
            assert_eq!(
                b.price[g], want,
                "tick {tick}: price of {name} is {} at 2x, expected exactly {want} (1x reads \
                 {}). Money is not neutral, so some rule read an absolute currency amount.",
                b.price[g], a.price[g]
            );
            assert_eq!(
                a.cleared[g], b.cleared[g],
                "tick {tick}: {name} cleared {} at 1x and {} at 2x — a redenomination changed \
                 the real economy.",
                a.cleared[g], b.cleared[g]
            );
        }
        for (i, (ia, ib)) in a.inventory.iter().zip(&b.inventory).enumerate() {
            for g in 0..one.goods.len() {
                let want = if GoodId(g as u32) == currency { 2.0 * ia[g] } else { ia[g] };
                assert_eq!(
                    ib[g], want,
                    "tick {tick}: inventory {i} holds {} of {} at 2x, expected {want}",
                    ib[g], one.goods[g]
                );
            }
        }
        assert_eq!(a.scale, b.scale, "tick {tick}: desk scales diverged");
        assert_eq!(a.wealth, b.wealth, "tick {tick}: pop wealth tiers diverged");
        assert_eq!(a.participation, b.participation, "tick {tick}: labour margins diverged");
    }
    eprintln!(
        "\nNEUTRALITY — 1x vs 2x: {} ticks, every real series bit-identical.",
        one.frames.len()
    );
}

/// **The falsification control for the neutrality test**, and it took two
/// attempts, which is itself a finding.
///
/// A comparison written so that it cannot fail — a run against itself, or over
/// quantities that happen to be zero — would report "neutral" forever. So the 2x
/// tape is re-run with its pop's cash left short of a true doubling, which is a
/// genuinely different world, and the same comparison must reject it.
///
/// **The first attempt used a shortfall of 1 currency unit and the comparison
/// did NOT fire.** That is not a broken test, it is a measurement: the pop's
/// reserve at 2x is 130, so 1 unit is `sigma_c = -0.0077`, well inside the
/// `dead = 0.05` band — the wealth rule never moves, the ration stays 1, the
/// orders are identical, and the balanced cash flow holds the shortfall at
/// exactly 1 forever. **The dead band absorbs a 0.77% cash shock completely and
/// permanently.** The shortfall used here is 10, which is `sigma_c = -0.077` and
/// outside the band. Both numbers are stated so nobody re-derives this.
#[test]
fn the_neutrality_comparison_rejects_a_world_that_is_not_a_redenomination() {
    let one = simulate("solv_1g_money", AgentArm::Kernel, PriceRule::Imbalance, None);

    let s = loader::load(&scenario_dir("solv_1g_money_2x")).expect("2x tape loads");
    let (mut state, gd, events) = (s.state, s.game_data, s.events);
    let gbp = GoodId(1);
    let pop_inv = state.pop_groups[0].inventory;
    state.inventory_mut(pop_inv).remove(gbp, 10.0);

    let mut differed = None;
    'outer: for t in 0..TICKS as usize {
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, PriceRule::Imbalance);
        for g in 0..gd.num_goods() {
            let good = GoodId(g as u32);
            if good == gbp {
                continue;
            }
            if state.price(NODE, good) != 2.0 * one.frames[t].price[g] {
                differed = Some((t + 1, gd.good(good).name.clone()));
                break 'outer;
            }
        }
    }
    let (tick, good) = differed.expect(
        "a world that is NOT a redenomination passed the neutrality comparison, so the \
         comparison proves nothing. Fix it before trusting the test above.",
    );
    eprintln!(
        "\nFALSIFICATION (neutrality) — 2x tape, pop 10 currency short: {good} diverged at tick {tick}"
    );
}

// ── The price-responsive Rule 1, measured on the worlds with a known answer ───

/// **Does the price-responsive Rule 1 make anything attract to the fixed point?**
///
/// This is the question the whole change exists to answer, asked on the only
/// worlds where the right answer is known before the run. `RESULT 2` above is
/// the baseline: under the shipped rule, displacing one genesis price by 2x
/// loses the market in every one of the eight kernel configurations, with the
/// traded volume decaying to 0.000-0.356 of its own opening level.
///
/// Nothing here invents a bar: `d.tol_log` is `equilibrium.ron`'s registered
/// `ln(1.10)`, and the volume trend is a ratio against the run's own first
/// tenth — a direction, not a level.
///
/// **MEASURED 2026-07-31, and it changes RESULT 2 above for one arm.**
/// `solv_1g` under `imbalance` + `reservation_goods` runs its volume at
/// **1.134x** across the window: displaced by 2x, it does not lose its market.
/// That is the first configuration in this repository where a displaced world
/// recovers, and it is asserted below so a regression is loud. `solv_chain`
/// under the same pair moves from a median price gap of **1.5e11x to 2.12x**
/// against a registered bar of 1.10x — still a miss, by a factor the earlier
/// runs could not have distinguished from infinity.
///
/// The **full** `reservation` rule does the opposite on every one of the eight:
/// volume trend 0.0000 everywhere. The labour half of the change kills the
/// market, which is why the decomposition arm exists.
#[test]
fn mode_b_under_the_price_responsive_rule_beside_the_shipped_one() {
    eprintln!("
MODE B — shipped (inelastic) vs price-responsive (reservation) Rule 1");
    let mut improved = Vec::new();
    let mut worsened = Vec::new();
    let mut survived: Vec<String> = Vec::new();
    for name in DISPLACEABLE {
        let eq = load_eq(name);
        let d = eq.displaced.clone().expect("these four register a displacement");
        for rule in [PriceRule::Imbalance, PriceRule::Ratio] {
            let mut row = Vec::new();
            for supply in
                [SupplyRule::Inelastic, SupplyRule::ReservationGoods, SupplyRule::Reservation]
            {
                let sim = simulate_with(
                    name,
                    AgentArm::Kernel,
                    rule,
                    Some((&d.good, d.factor)),
                    Some(supply),
                );
                let gap = mode_b_gap(&sim, &eq);
                let (good, trend) = volume_trend(&sim, &eq);
                report(
                    &format!("{name}/{}/{}", rule.name(), supply.name()),
                    &gap,
                    &sim,
                    &eq,
                );
                let _ = &good;
                row.push((gap.worst, trend, format!("{name}/{}/{}", rule.name(), supply.name())));
            }
            for (_, trend, tag) in &row {
                if *trend >= 1.0 {
                    survived.push(format!("{tag} (volume trend {trend:.4}x)"));
                }
            }
            let (old_gap, old_trend, _) = row[0].clone();
            let tag = format!("{name}/{}", rule.name());
            for (k, label) in [(1usize, "goods"), (2usize, "full")] {
                let (new_gap, new_trend, _) = row[k].clone();
                if new_gap < old_gap {
                    improved.push(format!("{tag}/{label} price {:.3e}->{:.3e}", old_gap, new_gap));
                } else if new_gap > old_gap {
                    worsened.push(format!("{tag}/{label} price {:.3e}->{:.3e}", old_gap, new_gap));
                }
                eprintln!(
                    "    => {tag}/{label}: median gap {:.4}x -> {:.4}x (registered bar {:.4}x),                      volume trend {:.4} -> {:.4}",
                    old_gap.exp(),
                    new_gap.exp(),
                    d.tol_log.exp(),
                    old_trend,
                    new_trend
                );
            }
        }
    }
    eprintln!("
  price gap improved: {improved:?}");
    eprintln!("  price gap worsened: {worsened:?}");
    eprintln!("  markets that did NOT decay: {survived:?}");
    // The recorded finding, asserted so a regression in either direction is
    // loud. It is not a tuned bar: "volume trend >= 1" means "traded no less at
    // the end of the window than at the start", which is a direction.
    assert_eq!(
        survived,
        vec!["solv_1g/imbalance/reservation_goods (volume trend 1.1338x)".to_string()],
        "the set of displaced configurations that keep their market changed. Recorded \
         2026-07-31: exactly one — solv_1g under imbalance with the reservation band on \
         storable outputs and the shipped labour rule — and it is the first in this repo. \
         If this grew, say so in docs/PLAN.md; if it shrank, the same. Do NOT delete the \
         assertion to make the test green."
    );
}
