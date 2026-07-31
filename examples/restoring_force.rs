//! **Is there a restoring force?** Displace one registered price and watch.
//!
//! An independent re-measurement of the one result the 2026-07-31 session rests
//! on. Everything else that session produced was either refuted (the elasticity
//! table, the loop-gain claim) or downgraded (B8, which turned out to be a
//! reporter wearing a battery's label). The claim left standing was:
//!
//! > displace `solv_1g`'s grain price by 1%; under `supply_rule = inelastic` it
//! > sits at 1.010 for ever with standard deviation **exactly zero**, and under
//! > `supply_rule = reservation` it returns to 1.000.
//!
//! That was measured at ONE displacement, in ONE direction, on ONE tape, under
//! ONE price rule, and then quoted as a property of the mechanism. This probe
//! asks the questions that would turn it into one:
//!
//!   1. **the sweep** — the whole registered grid and its reciprocals, under
//!      every supply rule and both price rules;
//!   2. **the BASIN** — the connected set of displacements around the fixed
//!      point that comes back, measured rather than assumed to exist;
//!   3. **is `solv_1g` knife-edge** — the same sweep on `solv_chain` (two
//!      production stages) and `solv_1g_money` (the same real economy with the
//!      price level pinned).
//!
//! # Why the spread is measured three ways, and why that is not belt-and-braces
//!
//! "Standard deviation exactly zero" is a claim about an *estimator* as much as
//! about the data, and the difference decides whether the original sentence is
//! true. A two-pass sd of a bit-constant series is **not** zero: summing 851
//! copies of `x` and dividing by 851 does not return `x`, so the residuals are
//! ~1e-16 and the sd reads ~1e-15. Welford's update — which is what
//! [`certify::invariants`] and B8 both use — adds `(x − mean)/n` to the mean, so
//! on constant data every delta is *identically* 0.0 and `m2` never leaves 0.0.
//!
//! So this probe reports:
//!
//! * `sd` — Welford, the repo's own estimator, for comparability with the
//!   certificate. Exactly `0` on a frozen series.
//! * `range(win)` and `range(all)` — `max − min` of `ln(p/p_num)`, which is
//!   exact under any summation and needs no estimator at all. `range(all) == 0`
//!   is the true unit-root signature: the price *never moved*, as against
//!   `range(win) == 0`, which only says it stopped moving before the window
//!   opened. The original claim needs the first and the sweep below shows cases
//!   that satisfy only the second.
//!
//! # What is registered here and what is invented (nothing)
//!
//! No behavioural constant is introduced (R2) and no bar is chosen (R6). The
//! table columns are threshold-free: `ends` and `|ln dev|` are numbers,
//! `contraction` is a direction, `range` is exact, `alive` is a fraction of
//! window ticks on which every good with a registered `cleared` target actually
//! changed hands.
//!
//! The basin scan needs the word "returned" to mean something, so it reports
//! three verdicts side by side rather than picking one:
//!
//! * `exact` — `|ln(final/target)| ≤ fixed_point_tol_log`, the tape's OWN
//!   registered 1e-12, used for exactly the statistic it was registered for;
//! * `registered` — `≤ displaced.tol_log = ln(1.10)`, also the tape's own, and
//!   flagged as what it is: that bar was registered against a MEDIAN over the
//!   window, and applying it to an endpoint is a different statistic. It is
//!   printed for orientation, never as a pass;
//! * `contracted` — `|ln dev| final < |ln dev| at genesis`. Threshold-free, and
//!   the only one of the three that invents nothing at all.
//!
//! # The basin is defined as a connected component, and the reason matters
//!
//! A bisection reports a boundary only if the predicate is monotone in the
//! displacement, and **it is not** (see the results). So "the basin" here means
//! the connected run of grid points containing `f = 1`: the scan walks outward
//! from the fixed point until the predicate first fails, bisects THAT crossing,
//! and then reports every point beyond it that still returns as an ISLAND
//! rather than folding it into the interval. An island is a real finding — it
//! says the map has more than one attractor — and averaging it into a basin
//! width would hide it.
//!
//!     cargo run --release --example restoring_force [scenario ...]

use std::path::PathBuf;

use rustyecon::{
    certify::{certificate::tape_sha, Criteria},
    kernel::AgentArm,
    scenario::{equilibrium::Equilibrium, loader},
    state::game_data::SupplyRule,
    state::{GameData, SimState},
    systems::{clearing::PriceRule, run_tick},
    types::ids::{GoodId, MarketNodeId},
};

/// The horizon every solvable tape registers as `analysis_end`.
const TICKS: u64 = 1000;

/// These worlds have exactly one region, so exactly one market node.
const NODE: MarketNodeId = MarketNodeId(0);

/// The displacement grid and its reciprocals. Named in the task brief rather
/// than chosen here, so it is not a grid picked after seeing which points
/// behaved well.
const GRID: [f64; 8] = [1.001, 1.01, 1.02, 1.05, 1.1, 1.2, 1.5, 2.0];

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios").join(name)
}

/// One displaced run, recorded per tick rather than summarised in the loop.
///
/// Index 0 is the *genesis* state after the displacement and before any tick, so
/// `log_ratio[0]` is the pure perturbation and `contraction` has a denominator
/// that is a displacement rather than a displacement plus one tick of response.
struct Run {
    log_ratio: Vec<f64>,
    /// `min(supply, demand)` per scored good, `[tick][good]`; index 0 is genesis
    /// and holds nothing.
    cleared: Vec<Vec<f64>>,
    /// Set the moment any statistic this probe reads stops being computable.
    /// R5: a metric that cannot be computed FAILS; it is never rendered as a
    /// number that happens to be finite by accident.
    broken: Option<String>,
    /// One line describing the final state, for the mechanism diagnostic.
    final_state: String,
}

/// A compact description of where a run ended, so a claim about *why* a price
/// rested can be checked against the desk that rested it.
fn describe(state: &SimState, gd: &GameData, scored: &[GoodId]) -> String {
    let mut parts = Vec::new();
    for g in scored {
        // The price is printed per good, not only the ratio: the one-sided
        // freeze below turns out to be the NUMERAIRE moving, and a ratio alone
        // cannot say which side of it moved.
        parts.push(format!(
            "{} p {:.6} s/d {:.4}/{:.4}",
            gd.good(*g).name,
            state.price(NODE, *g),
            state.supply(NODE, *g),
            state.demand(NODE, *g)
        ));
    }
    for ri in &state.recipe_instances {
        let out: Vec<String> = gd
            .recipe(ri.recipe)
            .outputs
            .iter()
            .map(|o| {
                format!(
                    "{}={:.4}",
                    gd.good(o.good).name,
                    state.inventory(ri.output_inv).get(o.good)
                )
            })
            .collect();
        if !out.is_empty() {
            parts.push(format!("scale {:.4} stock[{}]", ri.chosen_size, out.join(",")));
        }
    }
    parts.join(" | ")
}

/// Run one tape to `TICKS` with one genesis price multiplied by `factor`.
///
/// The displacement is applied to the loaded state, not to a second tape on
/// disk, for the same reason `tests/test_12_solvable.rs` does it: the good and
/// the target live in `equilibrium.ron`, and a second directory would be a
/// second place for them to drift out of step.
fn simulate(
    name: &str,
    price_rule: PriceRule,
    supply: SupplyRule,
    displaced: GoodId,
    numeraire: GoodId,
    factor: f64,
    scored: &[GoodId],
) -> Run {
    let s = loader::load(&scenario_dir(name)).unwrap_or_else(|e| panic!("{name} loads: {e}"));
    let (mut state, mut gd, events) = (s.state, s.game_data, s.events);
    gd.kernel.supply_rule = supply;

    let p = state.price(NODE, displaced) * factor;
    state.set_price(NODE, displaced, p);
    // The EMA is a smoothed copy nothing behavioural reads, but leaving it at
    // the undisplaced value seeds a phantom transient into a plotted series.
    state.set_price_ema(NODE, displaced, p);

    let mut log_ratio = Vec::with_capacity(TICKS as usize + 1);
    let mut cleared = Vec::with_capacity(TICKS as usize + 1);
    let mut broken: Option<String> = None;

    // The ratio itself is checked, not just its two prices: a numeraire that has
    // decayed to a denormal is finite and positive while `p/p_num` is `inf`, and
    // that case occurs in this corpus under the `ratio` price rule.
    let record = |state: &SimState, log_ratio: &mut Vec<f64>, broken: &mut Option<String>| {
        let (a, b) = (state.price(NODE, displaced), state.price(NODE, numeraire));
        let r = (a / b).ln();
        if !r.is_finite() && broken.is_none() {
            *broken = Some(format!("ln(p/p_num) not finite at tick {} (p={a:e}, num={b:e})", log_ratio.len()));
        }
        log_ratio.push(r);
    };

    record(&state, &mut log_ratio, &mut broken);
    cleared.push(vec![f64::NAN; scored.len()]);

    for _ in 0..TICKS {
        run_tick(&mut state, &gd, &events, AgentArm::Kernel, price_rule);
        record(&state, &mut log_ratio, &mut broken);
        cleared.push(
            scored.iter().map(|g| state.supply(NODE, *g).min(state.demand(NODE, *g))).collect(),
        );
    }
    let final_state = describe(&state, &gd, scored);
    Run { log_ratio, cleared, broken, final_state }
}

/// Welford's standard deviation — the estimator `certify::invariants::std_dev`
/// and B8 both use, reproduced here so a number in this table and a number on a
/// certificate line are the same statistic.
///
/// On a bit-constant series every `x − mean` is identically 0.0, so `m2` never
/// leaves 0.0 and this returns **exactly** 0. A two-pass formula on the same
/// data returns ~1e-15. That difference is the whole content of the phrase
/// "standard deviation exactly zero", which is why the estimator is named.
///
/// `None` below two samples rather than 0.0 or a NaN: an undefined spread is a
/// declared absence under a registered convention, which R5 explicitly does not
/// treat as a failure, while a NaN rendered as a number is.
fn welford_sd(v: &[f64]) -> Option<f64> {
    if v.len() < 2 {
        return None;
    }
    let (mut mean, mut m2) = (0.0f64, 0.0f64);
    for (i, x) in v.iter().enumerate() {
        let n = (i + 1) as f64;
        let d = x - mean;
        mean += d / n;
        m2 += d * (x - mean);
    }
    Some((m2 / (v.len() - 1) as f64).sqrt())
}

/// `max − min`. Exact under any summation, so `0.0` here means bit-constant and
/// means it without reference to an estimator.
fn range(v: &[f64]) -> f64 {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for x in v {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    hi - lo
}

struct Reading {
    ends: f64,
    dev_end: f64,
    contraction: f64,
    sd_win: Option<f64>,
    range_win: f64,
    range_all: f64,
    alive: f64,
    first_dead: Option<usize>,
    broken: Option<String>,
    final_state: String,
}

fn read(run: &Run, target_log: f64, window: std::ops::Range<usize>) -> Reading {
    let win: Vec<f64> = run.log_ratio[window.clone()].to_vec();
    let last = run.log_ratio[run.log_ratio.len() - 1];
    let dev_end = (last - target_log).abs();
    let dev_0 = (run.log_ratio[0] - target_log).abs();

    let mut first_dead = None;
    let mut live = 0usize;
    for t in window.clone() {
        if run.cleared[t].iter().all(|q| *q > 0.0) {
            live += 1;
        } else if first_dead.is_none() {
            first_dead = Some(t);
        }
    }

    let mut broken = run.broken.clone();
    if broken.is_none() && !dev_end.is_finite() {
        broken = Some("final deviation is not finite".into());
    }

    Reading {
        ends: (last - target_log).exp(),
        dev_end,
        contraction: if dev_0 > 0.0 { dev_end / dev_0 } else { f64::NAN },
        sd_win: welford_sd(&win),
        range_win: range(&win),
        range_all: range(&run.log_ratio),
        alive: live as f64 / (window.end - window.start) as f64,
        first_dead,
        broken,
        final_state: run.final_state.clone(),
    }
}

fn zero_or(v: f64) -> String {
    if v == 0.0 {
        "0 (exact)".to_string()
    } else {
        format!("{v:.3e}")
    }
}

fn render(r: &Reading) -> String {
    if let Some(why) = &r.broken {
        return format!("UNCOMPUTABLE — {why}");
    }
    format!(
        "{:>14.6} {:>10.3e} {:>10.3e} {:>11} {:>11} {:>11} {:>6.3}{}",
        r.ends,
        r.dev_end,
        r.contraction,
        match r.sd_win {
            Some(v) => zero_or(v),
            None => "n/a".to_string(),
        },
        zero_or(r.range_win),
        zero_or(r.range_all),
        r.alive,
        match r.first_dead {
            Some(t) => format!("  (first dead tick {t})"),
            None => String::new(),
        }
    )
}

const HEAD: &str = "  factor             ends   |ln dev|   contract   sd(win)  range(win)  range(all)  alive";

/// The scored window, matching `tests/test_12_solvable.rs` and
/// `certify::metrics` so a number here and a number in a certificate are taken
/// over the same ticks. Index `i` is tick `i`, because index 0 is genesis.
fn window(c: &Criteria) -> std::ops::Range<usize> {
    let lo = c.transient.max(1) as usize;
    let hi = (c.analysis_end as usize + 1).min(TICKS as usize + 1);
    lo..hi
}

struct Tape {
    name: String,
    eq: Equilibrium,
    criteria: Criteria,
    displaced: GoodId,
    numeraire: GoodId,
    scored: Vec<GoodId>,
    target_log: f64,
    tape_sha: String,
    dead: f64,
}

fn load_tape(name: &str) -> Tape {
    let dir = scenario_dir(name);
    let s = loader::load(&dir).unwrap_or_else(|e| panic!("{name} loads: {e}"));
    let eq = Equilibrium::load(&dir)
        .unwrap_or_else(|e| panic!("{name}: {e}"))
        .unwrap_or_else(|| panic!("{name} registers no equilibrium.ron"));
    let criteria = s.criteria.unwrap_or_else(|| panic!("{name} registers criteria.ron"));
    let id = |g: &str| {
        GoodId(
            s.game_data
                .goods
                .iter()
                .position(|x| x.name == g)
                .unwrap_or_else(|| panic!("{name}: no good {g:?}")) as u32,
        )
    };
    let d = eq.displaced.clone().unwrap_or_else(|| panic!("{name} registers no displacement"));
    let displaced = id(&d.good);
    let numeraire = id(&eq.numeraire);
    let mut names: Vec<&String> = eq.cleared.keys().collect();
    names.sort();
    let scored = names.into_iter().map(|g| id(g)).collect();
    let target_log = (eq.prices[&d.good] / eq.prices[&eq.numeraire]).ln();
    Tape {
        name: name.to_string(),
        eq,
        criteria,
        displaced,
        numeraire,
        scored,
        target_log,
        tape_sha: tape_sha(&dir),
        // Read from the tape, never assumed: the freeze prediction below is a
        // statement about THIS tape's registered dead-band.
        dead: s.game_data.kernel.dead,
    }
}

impl Tape {
    fn case(&self, price: PriceRule, supply: SupplyRule, factor: f64) -> Reading {
        let run = simulate(
            &self.name,
            price,
            supply,
            self.displaced,
            self.numeraire,
            factor,
            &self.scored,
        );
        read(&run, self.target_log, window(&self.criteria))
    }
}

/// The three verdicts the basin scan reports side by side. Named so a reader can
/// see which bar produced which boundary rather than being handed one number.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// `|ln dev| ≤ fixed_point_tol_log` — the tape's own 1e-12.
    Exact,
    /// `|ln dev| ≤ displaced.tol_log` — the tape's own ln(1.10). REGISTERED FOR
    /// A MEDIAN, applied here to an endpoint; printed, never treated as a pass.
    Registered,
    /// `|ln dev| final < |ln dev| at genesis`. Invents nothing.
    Contracted,
}

impl Verdict {
    fn name(&self) -> &'static str {
        match self {
            Verdict::Exact => "exact(<=1e-12)",
            Verdict::Registered => "registered(<=ln1.10)",
            Verdict::Contracted => "contracted",
        }
    }
    fn holds(&self, t: &Tape, r: &Reading) -> bool {
        if r.broken.is_some() {
            return false; // fail-closed
        }
        match self {
            Verdict::Exact => r.dev_end <= t.eq.fixed_point_tol_log,
            Verdict::Registered => {
                r.dev_end <= t.eq.displaced.as_ref().map_or(0.0, |d| d.tol_log)
            }
            Verdict::Contracted => r.contraction < 1.0,
        }
    }
}

// ── 1. The sweep ──────────────────────────────────────────────────────────────

fn sweep(tape: &Tape) {
    let d = tape.eq.displaced.clone().expect("checked at load");
    println!(
        "\n══ SWEEP — {} (tape_sha {}), displacing {} against numeraire {}",
        tape.name, tape.tape_sha, d.good, tape.eq.numeraire
    );
    println!(
        "   arm kernel | target p/p_num = {:.6} | window {:?} of {TICKS} ticks | \
         ends = final (p/p_num)/target",
        tape.target_log.exp(),
        window(&tape.criteria)
    );
    println!(
        "   contract = |ln dev| final / at genesis (1.000 = never moved, <1 = shrank) | \
         range(all) = 0 is the unit root"
    );

    let mut factors: Vec<f64> = GRID.iter().map(|f| 1.0 / f).collect();
    factors.reverse();
    factors.extend_from_slice(&GRID);

    for price in [PriceRule::Imbalance, PriceRule::Ratio] {
        for supply in
            [SupplyRule::Inelastic, SupplyRule::Reservation, SupplyRule::ReservationGoods]
        {
            println!("\n  {}/{}\n{HEAD}", price.name(), supply.name());
            for f in &factors {
                println!("  {:>6.4} {}", f, render(&tape.case(price, supply, *f)));
            }
        }
    }
}

// ── 2. The basin, as a connected component ────────────────────────────────────

fn basin(tape: &Tape, price: PriceRule, supply: SupplyRule, verdict: Verdict) {
    // 40 points per side, geometric out to 8x and 1/8x. A grid, not a set of
    // interesting points: the purpose is to see the predicate's shape before
    // assuming it has one.
    const N: i32 = 40;
    let side = |up: bool| -> Vec<f64> {
        (1..=N)
            .map(|k| {
                let e = k as f64 / N as f64;
                if up {
                    8f64.powf(e)
                } else {
                    8f64.powf(-e)
                }
            })
            .collect()
    };

    for (label, up) in [("UP  ", true), ("DOWN", false)] {
        let grid = side(up);
        let flags: Vec<(f64, bool)> = grid
            .iter()
            .map(|f| (*f, verdict.holds(tape, &tape.case(price, supply, *f))))
            .collect();

        // The connected component containing f = 1: walk outward until the
        // first failure. Everything past that which still holds is an island.
        let first_bad = flags.iter().position(|(_, ok)| !*ok);
        let (edge_in, edge_out) = match first_bad {
            Some(0) => (1.0, flags[0].0),
            Some(i) => (flags[i - 1].0, flags[i].0),
            None => {
                println!(
                    "  {label} {:<10}/{:<18} {:<22}: every grid point out to {:.4}x holds — \
                     no boundary inside the scan",
                    price.name(),
                    supply.name(),
                    verdict.name(),
                    grid[grid.len() - 1]
                );
                continue;
            }
        };
        let islands: Vec<f64> = flags
            .iter()
            .skip(first_bad.unwrap() + 1)
            .filter(|(_, ok)| *ok)
            .map(|(f, _)| *f)
            .collect();

        // Bisect the FIRST crossing only, in log-factor space: the grid is
        // geometric and the perturbation multiplicative.
        let (mut lo, mut hi) = (edge_in.ln(), edge_out.ln());
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if verdict.holds(tape, &tape.case(price, supply, mid.exp())) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        println!(
            "  {label} {:<10}/{:<18} {:<22}: holds out to {:.6}x, fails from {:.6}x{}",
            price.name(),
            supply.name(),
            verdict.name(),
            lo.exp(),
            hi.exp(),
            if islands.is_empty() {
                String::new()
            } else {
                format!(
                    "  ** {} ISLAND(S) beyond the boundary — the predicate is NOT monotone, so \
                     this is a component and not an interval: {:?}",
                    islands.len(),
                    islands.iter().take(5).map(|f| format!("{f:.4}")).collect::<Vec<_>>()
                )
            }
        );
    }
}

// ── 3. The unit root, checked for exactness rather than smallness ─────────────

/// Under `inelastic` the claim is not that the spread is small — it is that the
/// price never moves at all. So this checks `max − min` of the whole series,
/// which is exact, and reports the dial-derived displacement at which the freeze
/// should stop, computed from the tape's own `dead` rather than fitted.
///
/// Rule 2 gates on the desk's relative margin `2(R − 1)/(R + 1)`. A desk whose
/// only displaced price is its output rests iff `|2(f − 1)/(f + 1)| ≤ dead`,
/// i.e. `f ∈ [(1 − d/2)/(1 + d/2), (1 + d/2)/(1 − d/2)]`. At the registered
/// `d = 0.05` that is `[0.951220, 1.051282]`.
fn unit_root(tape: &Tape, price: PriceRule) {
    let d = tape.dead;
    let hi = (1.0 + d / 2.0) / (1.0 - d / 2.0);
    println!(
        "\n══ UNIT ROOT — {} under {}/inelastic. Is the price frozen EXACTLY, and out to where?",
        tape.name,
        price.name()
    );
    println!(
        "   prediction from the tape's registered dead = {d}: frozen iff \
         |2(f-1)/(f+1)| <= dead, i.e. f in [{:.6}, {:.6}]",
        1.0 / hi,
        hi
    );
    println!("{HEAD}");
    let mut probes: Vec<f64> = vec![1.0];
    for f in [1.001, 1.01, 1.02, 1.05, 1.0512, 1.0513, 1.06, 1.1] {
        probes.push(f);
        probes.push(1.0 / f);
    }
    probes.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    for f in probes {
        println!("  {:>6.4} {}", f, render(&tape.case(price, SupplyRule::Inelastic, f)));
    }
}

// ── 4. Where a rested price actually rested, and on what stock ────────────────

/// The mechanism check. A price that stops moving has stopped because some
/// market balanced; this prints WHICH quantities balanced it, so "it returned"
/// and "it rested somewhere else" can be told apart by more than the price.
fn mechanism(tape: &Tape, price: PriceRule, supply: SupplyRule, factors: &[f64]) {
    println!("\n══ FINAL STATE — {} under {}/{}", tape.name, price.name(), supply.name());
    for f in factors {
        let r = tape.case(price, supply, *f);
        println!("  f={f:<8.4} ends {:>12.9}x  |  {}", r.ends, r.final_state);
    }
}

// ── 5. Why the return stops where it stops, and what that costs ──────────────

/// **The return has a floor, and the floor is a constant in the source.**
///
/// `src/systems/price_update/mod.rs:28` only emits a price delta when
/// `|p_next − p| > 1e-12`. That is an **absolute** threshold in currency units
/// with behavioural meaning, so a run stops converging the moment its per-tick
/// step falls under it. For a displaced `solv_1g` the step is
/// `p·α·|imbalance|`, and near the fixed point `imbalance ≈ −2·e` where `e` is
/// the log price error (the desk's band is `b_out·flow/R`, so a markup of
/// `1 + e` releases `2·b_out·flow·e ≈ 40e` extra against a demand of 20). The
/// convergence therefore stalls at
///
/// ```text
///     p·α·2e ≤ 1e-12   ⇒   e ≤ 1e-12/(2·α·p) = 1e-12/(2·0.05·0.5) = 2.0e-11
/// ```
///
/// and the measured residual is **1.86e-11**. That is not economics. It is a
/// numerical guard, and it is the reason "returns to 1.000 exactly" is false at
/// the tape's own registered `fixed_point_tol_log = 1e-12`.
///
/// **It is also level-dependent, which is testable.** `solv_1g_money_2x` is the
/// same real economy at twice the price level (`tests/test_12_solvable.rs`
/// asserts bit-identical real series *undisplaced*). Double `p` and the same
/// algebra halves `e`. So this runs both tapes with the same *relative*
/// displacement and prints both residuals: if the guard is behavioural, the 2x
/// tape must converge twice as close, and a redenomination will have changed a
/// real outcome — the R2/R12 violation the neutrality test cannot see because
/// it only ever runs the tapes at their exact fixed point, where nothing drifts.
fn price_guard(factor: f64) {
    println!(
        "\n══ THE STALL — is the residual a fact about the economy or about the 1e-12 \
         guard in src/systems/price_update/mod.rs:28?"
    );
    println!(
        "   both tapes displaced by the SAME relative factor {factor}; \
         solv_1g_money_2x is solv_1g_money at exactly twice the price level"
    );
    for name in ["solv_1g_money", "solv_1g_money_2x"] {
        let dir = scenario_dir(name);
        let s = loader::load(&dir).unwrap_or_else(|e| panic!("{name} loads: {e}"));
        let id = |g: &str| {
            GoodId(s.game_data.goods.iter().position(|x| x.name == g).expect("good") as u32)
        };
        let (grain, labour) = (id("grain"), id("labour"));
        let run = simulate(
            name,
            PriceRule::Imbalance,
            SupplyRule::Reservation,
            grain,
            labour,
            factor,
            &[grain, labour],
        );
        let last = run.log_ratio[run.log_ratio.len() - 1];
        let genesis_level = {
            let s2 = loader::load(&dir).expect("loads");
            s2.state.price(NODE, grain)
        };
        println!(
            "   {name:<18} p_grain at genesis {genesis_level:.4} | \
             final ln(p_grain/p_labour) − ln(0.5) = {:>10.4e} | predicted stall \
             1e-12/(2·alpha·p) = {:.4e}",
            last - 0.5f64.ln(),
            1e-12 / (2.0 * s.game_data.good(grain).alpha * genesis_level),
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let names: Vec<String> = if args.is_empty() {
        ["solv_1g", "solv_chain", "solv_1g_money"].iter().map(|s| s.to_string()).collect()
    } else {
        args
    };

    price_guard(1.01);

    for name in &names {
        let tape = load_tape(name);
        sweep(&tape);
        unit_root(&tape, PriceRule::Imbalance);
        // Both rules: the `inelastic` rows are what show the downward freeze to
        // be the numeraire drifting rather than the displaced good returning.
        mechanism(
            &tape,
            PriceRule::Imbalance,
            SupplyRule::Inelastic,
            &[1.0, 1.01, 0.99, 0.98],
        );
        mechanism(
            &tape,
            PriceRule::Imbalance,
            SupplyRule::Reservation,
            &[1.0, 1.001, 1.01, 1.02, 1.05, 0.999, 0.99, 0.98],
        );
        println!(
            "\n══ BASIN — {}: the connected set of displacements that comes back.\n   \
             three verdicts, side by side; the tape's own bars are {:.1e} (exact) and \
             {:.6} (ln 1.10, REGISTERED FOR A MEDIAN — shown for orientation only)",
            tape.name,
            tape.eq.fixed_point_tol_log,
            tape.eq.displaced.as_ref().map_or(f64::NAN, |d| d.tol_log),
        );
        for price in [PriceRule::Imbalance, PriceRule::Ratio] {
            for supply in
                [SupplyRule::Inelastic, SupplyRule::Reservation, SupplyRule::ReservationGoods]
            {
                for v in [Verdict::Exact, Verdict::Registered, Verdict::Contracted] {
                    basin(&tape, price, supply, v);
                }
            }
        }
    }
}
