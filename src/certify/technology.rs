//! B8 — the realised-versus-implied price gap. A **reporter**, not a battery
//! (v2 Phase 4; PLAN, "The missing instrument"; renamed 2026-07-31, §"The
//! decision" on [`PriceGapWatch::report`]).
//!
//! Every other criterion in this repository asks *did it stay put*. `DRIFTING`,
//! `SWINGING`, `UNSTABLE`, `DEAD` are all stability rules, and a world that
//! settles instantly on nonsense passes all of them. Nothing had ever asked *did
//! the prices go to the right place* — which is why a failure to settle could
//! never be told apart from a missing feature: there was no case where the right
//! answer was known.
//!
//! **The tape already contains the right answer.** Technology fixes relative
//! prices: at zero profit a good's price is the price of the inputs that went
//! into it, recursively, so `p_g / p_labour` is the labour embodied in one unit
//! of `g`. In `lr_00`, `wheat_farm` spends 0.3 labour per wheat and `grain_mill`
//! adds 0.2 on top of one wheat, so flour embodies 0.5 and the competitive
//! `RealWage = p_labour / p_flour` is exactly **2.0**. `RealWage` has been a
//! scored metric since Phase 1 and nothing had ever compared it to that number.
//!
//! # What this module is, and what it is not
//!
//! It is a *long-run, full-utilisation, zero-profit, single-node* benchmark. It
//! is not a prediction of the tick-by-tick path, and a run whose prices sit some
//! distance from it is not thereby broken — real economies carry margins, rents
//! and transport wedges. What the distance buys is a *direction and a magnitude*
//! for each market: "flour 2.5× too dear" is a lead, where "the band was 11.5×"
//! is only a complaint.
//!
//! **Every one of those words is load-bearing, and each is printed on the
//! certificate line** (`report`), because the corpus already contains a case
//! where a reading was mistaken for an error:
//!
//! 1. **Zero rent.** The benchmark prices labour and nothing else scarce.
//!    `solv_labour` is the corpus's one world with a binding second factor — the
//!    firm's `recipe_size` caps output, so it earns a capacity rent and its
//!    registered equilibrium is `p_grain = p_labour`, not the labour-value
//!    answer of 0.5. B8 read that world, which is *exactly right*, as
//!    "grain 2.000x too dear". Modelling the rent is not attempted here: it
//!    needs the scarce factor's shadow price, which is a solve this module does
//!    not do and a new registered object if it did. What is done instead is to
//!    say the assumption out loud on every line.
//! 2. **Full utilisation.** `Fixed` and `SemiVariable` inputs are per-unit only
//!    at nameplate, which is the regime this benchmark is *about*. `tracer_2r`'s
//!    `grow_grain` is the corpus's one `Fixed` input, and below full size it
//!    really does draw more labour per grain than this says — so the benchmark
//!    flatters that scenario's producer, in the direction of making grain look
//!    *dearer* than the value implies. Recorded because it is an error with a
//!    known sign.
//! 3. **Single node.** See [`LabourValues::pass_through`]. A good is worth the
//!    same everywhere here, so transport adds no value at the destination and a
//!    competitive importer that merely recovers its haulage cost scores as an
//!    error. The bias is computed from the tape rather than asserted, and
//!    printed.
//! 4. **Capital is ignored.** `component_reqs` are a stock requirement, not a
//!    flow input, and amortising them needs a depreciation rate that no tape
//!    registers. The values here are therefore labour-*flow* values. No scenario
//!    in the corpus declares a component requirement, so nothing currently
//!    depends on the choice; a world that does will need this revisited, and
//!    that is a design event, not a tweak.
//! 5. **Joint products each carry the whole input bill.** A recipe with two
//!    outputs charges each of them the full cost of the inputs. Splitting it
//!    would need a rule for apportioning, every such rule is a registered
//!    constant, and no corpus recipe has more than one output. Ported from
//!    `tools/derive_parity.py`, which made the same choice for the same reason.

use crate::certify::criteria::Criteria;
use crate::state::{GameData, SimState};
use crate::types::ids::{GoodId, MarketNodeId};

/// Cap on relaxation sweeps before the solver gives up and *says so*.
///
/// An acyclic recipe graph is resolved in at most one sweep per good, but the
/// graph is not acyclic — `flour_transport` takes flour and makes flour — so a
/// topological pass would not terminate and the solve is a relaxation instead.
///
/// **Superseded wording, marked in place (R14, 2026-07-31).** This comment used
/// to read "A cycle that lowers cost each time round … has no fixed point at
/// all, and without a cap it spins forever", and [`Value::NotConverged`] was
/// documented as "a defect in the technology graph itself". That is true of the
/// *constructed* runaway in `a_cost_reducing_cycle_is_reported_as_not_converged`
/// and false as a general reading of the cap. A cycle whose round-trip gain `f`
/// is just under one has a *unique positive* fixed point and merely approaches
/// it geometrically, needing about `ln(IMPROVE_REL)/ln(f)` laps to get inside
/// the improvement floor. Measured, on the `iron = 1 + f·tools`,
/// `tools = iron + 1` family in `examples/b8_adversary.rs`: `f = 0.89` lands in
/// **255** sweeps and `f = 0.90` **trips this cap** — at a graph whose exact
/// answer is `iron = 19.0`. So hitting the cap means *this relaxation did not
/// settle within this cap*, which is a statement about the solver, and only a
/// statement about the graph once somebody has looked. Reported as
/// [`Value::NotConverged`] rather than returning whatever the last sweep
/// happened to hold: a value that is still moving is not a value.
///
/// Raising the cap is not the fix and is deliberately not done here: it moves
/// the boundary without removing it, and 256 is what every certificate in the
/// corpus was produced under. The fix is a solver that reports its own residual
/// instead of a boolean, which is a change with its own A/B.
const MAX_SWEEPS: usize = 256;

/// A route is only "cheaper" if it is cheaper by more than floating-point dust.
///
/// Relative, not absolute, because labour contents in one world span whatever
/// range the technology spans. Without it two arithmetically-equal routes can
/// alternate forever at the last bit and the solver reports non-convergence for
/// a graph that converged.
const IMPROVE_REL: f64 = 1e-12;

/// How many goods the certificate's one-line summary names before it stops.
///
/// Presentation only — every reading is available from
/// [`PriceGapWatch::readings`], and the verdict does not depend on this. It
/// exists because a compiled world (PLAN Phase 8) has tens of goods and a
/// battery line nobody reads is a battery nobody checks.
const LIST_GOODS: usize = 6;

/// Render a log distance as a multiple with its direction named.
///
/// A bare distance is useless for diagnosis: `0.917` says nothing a reader can
/// act on, `flour 2.503x too dear` names the market and which way to look. This
/// is the whole reason B8 is worth having over the existing `RealWage` band.
fn phrase_log(subject: &str, log_gap: f64) -> String {
    let x = log_gap.exp();
    if x >= 1.0 {
        format!("{subject} {x:.3}x too dear")
    } else {
        format!("{subject} {:.3}x too cheap", 1.0 / x)
    }
}

// ── 1. Labour values ──────────────────────────────────────────────────────────

/// What the recipe graph says one unit of a good is worth in labour, or why it
/// says nothing.
///
/// An enum rather than an `Option<f64>` because the three ways of having no
/// value are three different findings, and collapsing them would hide which one
/// a world has. METHODOLOGY R5's fail-closed rule applies to all three equally,
/// but the *diagnosis* differs: `Unreachable` is a modelling gap in the tape,
/// `Free` is a technology that makes something from nothing, and `NotConverged`
/// is a good the relaxation had not finished with — see [`MAX_SWEEPS`] for why
/// that is not the same as a defect in the graph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    /// Labour embodied in one unit, direct plus indirect. Strictly positive.
    /// The labour good itself is `Labour(1.0)` by definition — it is the unit.
    Labour(f64),
    /// A currency good. Deliberately outside the labour accounting: currency is
    /// a claim, not a produced thing, and the `dividend` recipe that "makes" GBP
    /// from GBP would otherwise hand it a labour content of zero and make every
    /// price in the world look infinitely dear against it.
    Currency,
    /// No recipe route reaches labour — an endowment, an import, or a good that
    /// nothing in this world makes.
    Unreachable,
    /// A route exists and embodies no labour at all: a free good. Its
    /// competitive relative price is zero, which no positive posted price can
    /// ever match, so it is excluded from the distance rather than scored
    /// against an unreachable target.
    Free,
    /// This good's own value was still moving, or something upstream of it was,
    /// when [`MAX_SWEEPS`] ran out. Reported, never rounded off.
    ///
    /// **Scope corrected 2026-07-31 (R14).** This used to be applied to *every*
    /// good in the tape the moment any cycle failed to settle — including
    /// labour, which is `Labour(1.0)` by definition and which the relaxation
    /// never writes to, so the numeraire itself came back unpriced because two
    /// unrelated goods in another corner of the graph were chasing each other.
    /// The old comment justifying that ("a graph that never settled has no
    /// trustworthy values anywhere in it") is wrong in one specific way: a good
    /// that is neither still moving nor downstream of anything still moving is
    /// not fed by the runaway and its value is final. Only those two sets are
    /// marked now; `a_stalled_cycle_marks_only_the_goods_that_are_still_moving`
    /// is the guard.
    NotConverged,
}

impl Value {
    /// The implied competitive price of this good **relative to labour**.
    ///
    /// Zero profit gives `p_g = Σ a_i · p_i`, and unrolling the recursion to its
    /// only ungrounded input leaves `p_g = λ_g · p_labour` with `λ_g` the
    /// embodied labour. So the implied relative price *is* the labour value, and
    /// normalising against the labour good is a no-op — which is exactly why
    /// labour is the right numeraire to compare a posted price vector in.
    pub fn relative_price(&self) -> Option<f64> {
        match self {
            Value::Labour(v) if *v > 0.0 => Some(*v),
            _ => None,
        }
    }

    /// Why this good has no implied relative price, for the certificate line.
    pub fn why(&self) -> &'static str {
        match self {
            Value::Labour(_) => "resolved",
            Value::Currency => "currency good, outside the labour accounting",
            Value::Unreachable => "no recipe route reaches labour",
            Value::Free => "produced from nothing that embodies labour",
            Value::NotConverged => "labour value still moving after the sweep cap",
        }
    }
}

/// A recipe whose output is also one of its inputs, and the labour it adds that
/// lands in no good's value anywhere.
///
/// **The single-node assumption, made visible.** `flour_transport` takes one
/// flour and 0.1 labour and yields one flour. Treating that as production would
/// define flour in terms of itself, so it is skipped — and the consequence is
/// that the 0.1 labour it spends is in *no* good's labour content. A benchmark
/// that values a good identically at every node is the same benchmark that
/// cannot see haulage, and the two are one choice, not two.
///
/// **The choice, and why it went this way.** The alternative is to value goods
/// *per node*, so that flour at Leeds embodies flour at Manchester plus the
/// transport labour. It was rejected for this repair, on grounds that are
/// recorded rather than assumed:
///
/// - It needs a node-to-node route graph the solver does not have. `recipes` are
///   not sited: nothing in `RecipeDef` says which node a `flour_transport`
///   instance runs between, and the siting lives in `starting_state.ron`'s
///   building placements and in `channels`. Reading it would make the benchmark
///   depend on where buildings happen to have been *placed*, which is a genesis
///   choice, not a technology.
/// - It turns one value per good into one value per (node, good) and makes the
///   cheapest-route relaxation a shortest-path solve over a graph with cycles
///   *and* an economically meaningful direction. That is a new algorithm with
///   its own falsification burden, and it would no longer be the same algorithm
///   as `tools/derive_parity.py`, which is what keeps the `parity` the agents
///   aim at and the benchmark they are scored against from disagreeing.
///
/// So the benchmark stays single-node, and the bias is **computed from the tape
/// and printed** instead of being left as a surprise. On `lr_00` the whole of it
/// is one line: `flour_transport` adds 0.100 labour to a flour worth 0.500, so
/// an importing region's competitive flour price is 1.200× the benchmark and a
/// perfectly competitive importer scores as 1.200× too dear. That is the size of
/// the known bias, with its sign, and it bounds it: no route in the corpus
/// stacks two hauls.
#[derive(Debug, Clone)]
pub struct PassThrough {
    pub recipe: String,
    pub good: String,
    /// Labour per unit this recipe adds on top of the good's own value.
    pub added: f64,
    /// The good's single-node labour value, for scale.
    pub base: f64,
}

impl PassThrough {
    /// How much dearer than the benchmark a competitive unit is at the far end.
    pub fn bias(&self) -> f64 {
        (self.base + self.added) / self.base
    }

    pub fn phrase(&self) -> String {
        format!(
            "{} adds {:.3} labour to {} that is in no good's value ({:.3}x at the far end)",
            self.recipe,
            self.added,
            self.good,
            self.bias()
        )
    }
}

/// The solved labour content of every good in a tape.
#[derive(Debug, Clone)]
pub struct LabourValues {
    values: Vec<Value>,
    labour: GoodId,
    /// How many sweeps the relaxation actually took. Reported so a tape that
    /// only just converged is visible before it stops converging.
    sweeps: usize,
}

impl LabourValues {
    pub fn get(&self, good: GoodId) -> Value {
        self.values[good.idx()]
    }

    pub fn labour_good(&self) -> GoodId {
        self.labour
    }

    pub fn sweeps(&self) -> usize {
        self.sweeps
    }

    /// Every good the solver could not price, with the reason, currency
    /// excluded — currency is *designed* to be outside this and reporting it
    /// would bury the goods that are accidentally outside it.
    pub fn unresolved(&self, game_data: &GameData) -> Vec<String> {
        self.values
            .iter()
            .enumerate()
            .filter(|(_, v)| !matches!(v, Value::Labour(_) | Value::Currency))
            .map(|(i, v)| format!("{}: {}", game_data.goods[i].name, v.why()))
            .collect()
    }

    /// Every pass-through recipe and the labour it spends that no good's value
    /// carries — the size of the single-node bias. See [`PassThrough`].
    pub fn pass_through(&self, game_data: &GameData) -> Vec<PassThrough> {
        let mut currency = vec![false; game_data.num_goods()];
        for node in &game_data.market_nodes {
            if let Some(c) = node.currency_good {
                currency[c.idx()] = true;
            }
        }
        let mut out = Vec::new();
        for recipe in &game_data.recipes {
            for output in &recipe.outputs {
                let g = output.good;
                if output.qty_per_unit <= 0.0 || g == self.labour || currency[g.idx()] {
                    continue;
                }
                if !recipe.inputs.iter().any(|i| i.good == g) {
                    continue;
                }
                let Some(base) = self.get(g).relative_price() else { continue };
                // Total input bill on the same terms the solver uses: currency
                // legs skipped, everything else at its solved value.
                let mut total = 0.0;
                let mut ok = true;
                for input in &recipe.inputs {
                    if currency[input.good.idx()] {
                        continue;
                    }
                    match self.get(input.good).relative_price() {
                        Some(v) => total += v * input.qty_per_unit,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                let added = total / output.qty_per_unit - base;
                // A pass-through that adds nothing (or, through a joint output,
                // appears to remove value) is not a wedge and reporting it would
                // bury the ones that are.
                if !ok || !added.is_finite() || added <= 0.0 {
                    continue;
                }
                out.push(PassThrough {
                    recipe: recipe.name.clone(),
                    good: game_data.goods[g.idx()].name.clone(),
                    added,
                    base,
                });
            }
        }
        out
    }
}

/// Solve the labour embodied in one unit of every good, direct plus indirect.
///
/// Ported from the working Python in `tools/derive_parity.py`, which derives the
/// pop participation `parity` from the same graph. Kept as one algorithm in two
/// languages rather than two algorithms: if the labour values that set `parity`
/// and the labour values that score B8 could disagree, the run would be marked
/// against a benchmark its own agents were not aiming at.
///
/// Four rules carried over from that implementation, each because it was needed:
///
/// - **Currency goods are skipped**, on both sides. See [`Value::Currency`].
/// - **A good appearing on both sides of a recipe is passing through, not being
///   made.** `flour_transport` takes flour and labour and yields flour; treating
///   that as production would say flour costs flour plus 0.1 labour, which is
///   not a definition of anything. What that costs the benchmark is measured by
///   [`LabourValues::pass_through`] rather than left implicit.
/// - **The graph has cycles, so it is solved by relaxation.** A topological pass
///   has no order to run in.
/// - **Where several recipes make a good, the cheapest route wins.** Competitive
///   price is the cost of the best available technology, not the average or the
///   worst — a producer using the dear route is driven out, not averaged in.
pub fn labour_values(game_data: &GameData, labour: GoodId) -> LabourValues {
    let n = game_data.num_goods();
    let mut currency = vec![false; n];
    for node in &game_data.market_nodes {
        if let Some(c) = node.currency_good {
            currency[c.idx()] = true;
        }
    }

    // `None` = not yet reached. Distinguished from `Some(0.0)`, which is a good
    // this technology really does make out of nothing.
    let mut content: Vec<Option<f64>> = vec![None; n];
    content[labour.idx()] = Some(1.0);

    // Which goods the *final* sweep moved. Only meaningful when that sweep is
    // the cap: it is the seed set for "what was still moving when we stopped".
    let mut moved = vec![false; n];

    let mut sweeps = 0;
    let mut converged = false;
    for _ in 0..MAX_SWEEPS {
        sweeps += 1;
        let mut changed = false;
        moved.iter_mut().for_each(|m| *m = false);
        for recipe in &game_data.recipes {
            for out in &recipe.outputs {
                if out.qty_per_unit <= 0.0 || out.good == labour || currency[out.good.idx()] {
                    continue;
                }
                // Pass-through, not production.
                if recipe.inputs.iter().any(|i| i.good == out.good) {
                    continue;
                }
                let mut total = 0.0;
                let mut ok = true;
                for input in &recipe.inputs {
                    if currency[input.good.idx()] {
                        continue;
                    }
                    match content[input.good.idx()] {
                        Some(v) => total += v * input.qty_per_unit,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    continue;
                }
                let value = total / out.qty_per_unit;
                if !value.is_finite() || value < 0.0 {
                    continue;
                }
                let slot = &mut content[out.good.idx()];
                let better = match slot {
                    None => true,
                    Some(cur) => value < *cur * (1.0 - IMPROVE_REL),
                };
                if better {
                    *slot = Some(value);
                    changed = true;
                    moved[out.good.idx()] = true;
                }
            }
        }
        if !changed {
            converged = true;
            break;
        }
    }

    // Everything the runaway feeds, and nothing else. A good whose own value
    // stopped moving is still untrustworthy if one of its inputs has not — the
    // cheaper route upstream will re-price it on some later sweep — so the seed
    // set is closed downstream through the recipe graph before it is used.
    // Labour can never enter it: the sweep above never writes to labour, so it
    // is never in `moved`, and no recipe output may be labour either.
    if !converged {
        loop {
            let mut grew = false;
            for recipe in &game_data.recipes {
                let fed_by_a_runaway = recipe
                    .inputs
                    .iter()
                    .any(|i| !currency[i.good.idx()] && moved[i.good.idx()]);
                if !fed_by_a_runaway {
                    continue;
                }
                for out in &recipe.outputs {
                    if out.good == labour || currency[out.good.idx()] {
                        continue;
                    }
                    // A pass-through never set this good's value, so it cannot
                    // be the channel that moves it either.
                    if recipe.inputs.iter().any(|i| i.good == out.good) {
                        continue;
                    }
                    if !moved[out.good.idx()] {
                        moved[out.good.idx()] = true;
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
    }

    let values = (0..n)
        .map(|i| {
            if currency[i] {
                Value::Currency
            } else if !converged && moved[i] && content[i].is_some() {
                // `content[i].is_some()` matters: a good the closure reached but
                // that never got a value at all was never reached by a route,
                // and "unreachable" is the finding, not "still moving".
                Value::NotConverged
            } else {
                match content[i] {
                    None => Value::Unreachable,
                    Some(v) if v > 0.0 => Value::Labour(v),
                    Some(_) => Value::Free,
                }
            }
        })
        .collect();

    LabourValues { values, labour, sweeps }
}

// ── 2. The realised-vs-implied gap ────────────────────────────────────────────

/// One (region, good) reading: where the market put the price, against where
/// technology says it belongs.
#[derive(Debug, Clone)]
pub struct GapReading {
    pub region: String,
    pub good: String,
    /// Implied relative price `p_g / p_labour` — the good's labour content.
    pub implied: f64,
    /// Mean of `ln(realised / implied)` over the scored window. Zero is right,
    /// positive is too dear, negative is too cheap.
    pub log_gap: f64,
    /// Standard deviation of the same log ratio. A small `log_gap` with a large
    /// `log_sd` is a price that averaged out right while never *being* right,
    /// and the two must not be reported as the same thing.
    ///
    /// **`None`, never `NaN` (R5, fixed 2026-07-31).** This was `f64::NAN`
    /// whenever `samples < 2`, and `report` interpolated it straight onto the
    /// certificate line: a run with one priced tick printed `sd NaN` next to
    /// `pass = true`, and `certify::nan::scan` could not see it because that
    /// scan walks `SimState` and this number lives in the accumulator. R5 says a
    /// metric that cannot be computed FAILS. The dispersion of one sample is a
    /// value *declared missing* under a registered convention — the same class
    /// as μ/r\* on a quiet tick — so it is represented as an absence and
    /// rendered as an absence; what is banned is the NaN, and
    /// [`PriceGapWatch::uncomputable`] fails the report closed if one ever
    /// appears anyway.
    pub log_sd: Option<f64>,
    /// Ticks in the window where the ratio existed and was used.
    pub samples: u64,
    /// Ticks in the window where this market traded at all — `samples`'
    /// denominator, without which `n=1` cannot be told apart from a market that
    /// genuinely traded once.
    pub traded_ticks: u64,
    /// Traded ticks where the ratio had no logarithm — a zero, negative or
    /// non-finite price on either leg.
    pub unpriced: u64,
    /// Traded ticks dropped because the **numeraire** did not trade, so every
    /// price on that tick was measured against a stale wage.
    pub numeraire_idle: u64,
}

impl GapReading {
    /// The gap as a multiple, with its direction named.
    pub fn phrase(&self) -> String {
        phrase_log(&self.good, self.log_gap)
    }

    /// Traded ticks that produced no reading.
    pub fn discarded(&self) -> u64 {
        self.traded_ticks.saturating_sub(self.samples)
    }

    /// The dispersion, or the fact that there is not enough to compute one.
    pub fn sd_phrase(&self) -> String {
        match self.log_sd {
            Some(sd) => format!("sd {sd:.3}"),
            None => "sd n/a (one sample)".into(),
        }
    }

    /// Sample count with its denominator and where the rest of it went.
    pub fn census(&self) -> String {
        format!(
            "n={} of {} traded tick(s), {} discarded ({} unpriced, {} numeraire idle)",
            self.samples,
            self.traded_ticks,
            self.discarded(),
            self.unpriced,
            self.numeraire_idle
        )
    }
}

/// A (region, good) pair the criterion could not be applied to at all.
#[derive(Debug, Clone)]
pub struct GapFailure {
    pub region: String,
    pub good: String,
    pub reason: String,
}

/// One region's readings split into the part that is a statement about the
/// **wage** and the part that is a statement about each **good**.
///
/// # Why this exists, and why it is the most useful thing on the line
///
/// Every reading in a region is `ln(p_g/p_labour) − ln(λ_g)`. A wage that is
/// wrong by a factor `k` therefore shifts *every* good's reading by the same
/// `ln k`, and the old report printed the result as N independent findings.
/// `lr_00`/Manchester under the legacy arm read wheat 312.3×, flour 208.8× and
/// services 43,415× "too dear" — three numbers spanning two orders of magnitude,
/// which look like three problems and are one: a common factor of ~1,414×, which
/// is a statement about `p_labour`, plus residuals of 4.5× *too cheap*, 6.8× too
/// cheap and 30.7× too dear around it. The residuals are the only part of the
/// reading that is about wheat, flour or services at all, and two of the three
/// point the *opposite way* to the raw number.
///
/// **The common factor is an unweighted geometric mean over the region's scored
/// goods** — one vote per good, not per sample. Weighting by sample count would
/// let whichever market happened to trade most decide what the wage error is,
/// and the quantity being estimated is shared by all of them equally. The
/// residuals then sum to zero across the goods by construction, which is what
/// makes "common factor plus residual" a decomposition rather than two
/// statistics printed next to each other.
///
/// **What it is not.** It cannot tell a wrong wage from a world where every good
/// is dear for its own reason and the wage is right; no statistic reading only
/// this vector can. It says "here is the part of these N numbers that is one
/// number", which is exactly what a reader needs before deciding which market to
/// open.
#[derive(Debug, Clone)]
pub struct WageDecomposition {
    pub region: String,
    /// Unweighted mean of the region's `log_gap`s: the shared factor, in logs.
    pub common_log: f64,
    /// `(good, log_gap − common_log)`, worst first.
    pub residuals: Vec<(String, f64)>,
}

impl WageDecomposition {
    pub fn common_phrase(&self) -> String {
        phrase_log("common factor", self.common_log)
    }

    /// The residual list, worst first, truncated for the certificate line.
    pub fn residual_phrases(&self, limit: usize) -> Vec<String> {
        let shown = self.residuals.len().min(limit);
        let mut out: Vec<String> = self.residuals[..shown]
            .iter()
            .map(|(g, r)| phrase_log(g, *r))
            .collect();
        if self.residuals.len() > shown {
            out.push(format!("+{} more", self.residuals.len() - shown));
        }
        out
    }
}

/// Accumulates the realised-vs-implied price gap over the scored window.
///
/// Welford in log space, O(1) per market, for the reason `BalanceWatch` gives
/// in [`crate::certify::invariants`]: at the 673-region target the per-tick
/// series would be a larger artifact than the run.
///
/// **Why the mean of logs and not the median.** The median needs the series, and
/// the memory argument above forbids holding it. The mean of logs is the
/// geometric mean, which is the natural centre of a multiplicative quantity —
/// "2× too dear" and "2× too cheap" cancel, as they should, where an arithmetic
/// mean of ratios would score that pair at 1.25 and call the world dear. The
/// PLAN's tables quote medians of `RealWage`; the two statistics will not print
/// the same digits and are not meant to.
#[derive(Debug, Clone)]
pub struct PriceGapWatch {
    num_goods: usize,
    /// `(region name, its market node)`, in `GameData::regions` order.
    regions: Vec<(String, MarketNodeId)>,
    /// `None` when the scenario registers no labour good, e.g. `big_region`.
    labour: Option<GoodId>,
    values: LabourValues,
    /// Per `region_idx * num_goods + good_idx`.
    count: Vec<u64>,
    mean: Vec<f64>,
    m2: Vec<f64>,
    unpriced: Vec<u64>,
    /// Traded ticks dropped because the numeraire was idle, per (region, good).
    idle_drops: Vec<u64>,
    /// In-window ticks on which this market traded, per (region, good).
    traded_ticks: Vec<u64>,
    /// In-window ticks observed, and of those how many had an idle numeraire —
    /// per region, so the drop can be reported even when no good survived.
    ticks_seen: Vec<u64>,
    numeraire_idle_ticks: Vec<u64>,
    /// Set when `criteria.labour_good` names a good the tape does not contain.
    ///
    /// Distinct from "no labour good", which the corpus declares deliberately by
    /// registering the empty name — `big_region` and `supply_chain` are worlds
    /// with no labour market and are legitimately unscored. A NON-empty name
    /// that resolves to nothing is a different thing entirely: a typo in a
    /// registered criteria file, which previously produced the same silent
    /// "unscored" pass as the deliberate case. One is a world without wages; the
    /// other is a broken tape claiming to be one.
    numeraire_missing: Option<String>,
}

impl PriceGapWatch {
    /// Build from the tape and the scenario's registered criteria.
    ///
    /// The labour good comes from `criteria.labour_good` — the same field the
    /// `RealWage` metric already resolves its numerator from, so B8 and the
    /// stability suite cannot disagree about which good is the numeraire.
    pub fn new(game_data: &GameData, criteria: &Criteria) -> Self {
        let labour = game_data
            .goods
            .iter()
            .find(|g| g.name == criteria.labour_good)
            .map(|g| g.id);
        // Empty name = declared absent; non-empty and unresolved = a tape error.
        let numeraire_missing = (labour.is_none() && !criteria.labour_good.is_empty())
            .then(|| criteria.labour_good.clone());
        // A world with no labour good has no numeraire to normalise against, so
        // there is nothing to solve. Reported as unscored, never as clean.
        let values = labour
            .map(|l| labour_values(game_data, l))
            .unwrap_or_else(|| LabourValues {
                values: vec![Value::Unreachable; game_data.num_goods()],
                labour: GoodId(0),
                sweeps: 0,
            });
        let regions = game_data
            .regions
            .iter()
            .map(|r| (r.name.clone(), r.market_node))
            .collect::<Vec<_>>();
        let r = regions.len();
        let n = r * game_data.num_goods();
        Self {
            num_goods: game_data.num_goods(),
            regions,
            labour,
            values,
            count: vec![0; n],
            mean: vec![0.0; n],
            m2: vec![0.0; n],
            unpriced: vec![0; n],
            idle_drops: vec![0; n],
            traded_ticks: vec![0; n],
            ticks_seen: vec![0; r],
            numeraire_idle_ticks: vec![0; r],
            numeraire_missing,
        }
    }

    pub fn labour_values(&self) -> &LabourValues {
        &self.values
    }

    /// Whether this good is in scope for the comparison.
    ///
    /// Labour itself is excluded: its relative price against itself is 1 by
    /// construction and always exactly right, so scoring it would dilute the
    /// worst-good summary with a guaranteed pass. Currency is excluded for the
    /// reason [`Value::Currency`] gives.
    fn scored(&self, good: GoodId) -> bool {
        Some(good) != self.labour && !matches!(self.values.get(good), Value::Currency)
    }

    /// Take one tick's reading of every market that traded, **against a
    /// numeraire that also traded**.
    ///
    /// The per-good trading gate is
    /// [`crate::certify::invariants::BalanceWatch`]'s convention: a market with
    /// neither supply nor demand posts a stale price that no agent acted on, and
    /// counting it would let a world be scored on prices nobody paid.
    ///
    /// **The numeraire gate is the same convention applied to the denominator,
    /// and it was missing (fixed 2026-07-31).** Every reading here is
    /// `p_g / p_labour`, so a labour market with zero supply and zero demand
    /// sitting at a stale price makes *every* per-good reading in that region
    /// wrong by one shared factor — and, because it is shared, wrong in a way
    /// the per-good numbers cannot reveal. Gating only the numerator meant B8
    /// would report 851 confident readings on a run whose wage nobody had paid
    /// since tick 149. Ticks dropped for this reason are counted, per region and
    /// per market, and printed: a silent drop is the same defect one layer down.
    pub fn observe(&mut self, state: &SimState) {
        let Some(labour) = self.labour else { return };
        for (ri, (_, node)) in self.regions.iter().enumerate() {
            let p_labour = state.price(*node, labour);
            // CLEARED volume, not posted orders. `supply` and `demand` are the
            // summed sell and buy *order* quantities (`systems::clearing::run`);
            // what changes hands is `min` of the two. The first version of this
            // gate read `supply > 0 || demand > 0`, which admits every tick on
            // which labour was OFFERED and nobody hired — exactly the state a
            // dead labour market is in, and exactly the state the gate exists to
            // exclude. Measured on `tracer_2r` under the kernel arm, that
            // version admitted the great majority of the window on a wage
            // nobody had paid. A price nobody transacted at is not a numeraire.
            let numeraire_traded =
                state.supply(*node, labour).min(state.demand(*node, labour)) > 0.0;
            self.ticks_seen[ri] += 1;
            if !numeraire_traded {
                self.numeraire_idle_ticks[ri] += 1;
            }
            for good_idx in 0..self.num_goods {
                let good = GoodId(good_idx as u32);
                if !self.scored(good) {
                    continue;
                }
                if state.supply(*node, good) <= 0.0 && state.demand(*node, good) <= 0.0 {
                    continue;
                }
                let i = ri * self.num_goods + good_idx;
                // Counted before the numeraire gate: the market DID trade, and
                // `failures` needs to know that in order to say that a live
                // market went unmeasured rather than that nothing happened.
                self.traded_ticks[i] += 1;
                if !numeraire_traded {
                    self.idle_drops[i] += 1;
                    continue;
                }
                let Some(implied) = self.values.get(good).relative_price() else {
                    continue;
                };
                let realised = state.price(*node, good) / p_labour;
                if !(realised.is_finite() && realised > 0.0) {
                    self.unpriced[i] += 1;
                    continue;
                }
                let x = (realised / implied).ln();
                if !x.is_finite() {
                    self.unpriced[i] += 1;
                    continue;
                }
                self.count[i] += 1;
                let delta = x - self.mean[i];
                self.mean[i] += delta / self.count[i] as f64;
                self.m2[i] += delta * (x - self.mean[i]);
            }
        }
    }

    /// Every (region, good) that traded and produced a usable distance.
    pub fn readings(&self, game_data: &GameData) -> Vec<GapReading> {
        let mut out = Vec::new();
        for (ri, (name, _)) in self.regions.iter().enumerate() {
            for good_idx in 0..self.num_goods {
                let i = ri * self.num_goods + good_idx;
                if self.count[i] == 0 {
                    continue;
                }
                let good = GoodId(good_idx as u32);
                let Some(implied) = self.values.get(good).relative_price() else {
                    continue;
                };
                let sd = (self.count[i] >= 2)
                    .then(|| (self.m2[i] / (self.count[i] - 1) as f64).sqrt());
                out.push(GapReading {
                    region: name.clone(),
                    good: game_data.goods[good_idx].name.clone(),
                    implied,
                    log_gap: self.mean[i],
                    log_sd: sd,
                    samples: self.count[i],
                    traded_ticks: self.traded_ticks[i],
                    unpriced: self.unpriced[i],
                    numeraire_idle: self.idle_drops[i],
                });
            }
        }
        out
    }

    /// Markets the criterion could not be applied to at all.
    ///
    /// **This, and only this, is what B8 fails on — see [`Self::report`].** A
    /// good that traded through the whole scored window and never once had a
    /// relative price is not a world at an unusual price, it is a world where
    /// the question has no answer.
    pub fn failures(&self, game_data: &GameData) -> Vec<GapFailure> {
        let mut out = Vec::new();
        for (ri, (name, _)) in self.regions.iter().enumerate() {
            for good_idx in 0..self.num_goods {
                let i = ri * self.num_goods + good_idx;
                if self.traded_ticks[i] == 0 {
                    continue;
                }
                let good = GoodId(good_idx as u32);
                let value = self.values.get(good);
                let reason = if value.relative_price().is_none() {
                    format!("implied value unavailable — {}", value.why())
                } else if self.count[i] == 0 {
                    format!(
                        "no computable relative price in {} traded tick(s) \
                         ({} unpriced, {} with an idle numeraire)",
                        self.traded_ticks[i], self.unpriced[i], self.idle_drops[i]
                    )
                } else {
                    continue;
                };
                out.push(GapFailure {
                    region: name.clone(),
                    good: game_data.goods[good_idx].name.clone(),
                    reason,
                });
            }
        }
        out
    }

    /// Split each region's readings into a shared wage factor and per-good
    /// residuals. See [`WageDecomposition`].
    pub fn decompositions(&self, game_data: &GameData) -> Vec<WageDecomposition> {
        let readings = self.readings(game_data);
        let mut out: Vec<WageDecomposition> = Vec::new();
        for (name, _) in &self.regions {
            let here: Vec<&GapReading> = readings.iter().filter(|r| &r.region == name).collect();
            if here.is_empty() {
                continue;
            }
            let common_log = here.iter().map(|r| r.log_gap).sum::<f64>() / here.len() as f64;
            let mut residuals: Vec<(String, f64)> = here
                .iter()
                .map(|r| (r.good.clone(), r.log_gap - common_log))
                .collect();
            residuals.sort_by(|a, b| {
                b.1.abs()
                    .partial_cmp(&a.1.abs())
                    .expect("readings are checked finite before this runs")
            });
            out.push(WageDecomposition { region: name.clone(), common_log, residuals });
        }
        out
    }

    /// Every statistic in a reading set that is not a number (R5, fail-closed).
    ///
    /// A free function over readings rather than a peek at the accumulator, so a
    /// test can hand it a hand-built [`GapReading`] and watch it fire — the
    /// house rule is that a guard is not trusted until it has been seen firing
    /// on a defect known to exist. `log_sd: None` is *not* a hit: an absence is
    /// a declared-missing value under a registered convention, which R5's own
    /// parenthetical exempts. A `Some(NaN)` is a hit, and so is a non-finite
    /// `log_gap` or `implied`.
    pub fn uncomputable(readings: &[GapReading]) -> Vec<String> {
        let mut out = Vec::new();
        for r in readings {
            for (what, v) in [
                ("log_gap", Some(r.log_gap)),
                ("implied", Some(r.implied)),
                ("log_sd", r.log_sd),
            ] {
                if let Some(v) = v {
                    if !v.is_finite() {
                        out.push(format!("{}/{} {what} = {v}", r.region, r.good));
                    }
                }
            }
        }
        out
    }

    /// B8's line, and the only verdicts it is entitled to.
    ///
    /// # THE DECISION: this is a REPORTER, and it is now named like one
    ///
    /// Adversarial review, 2026-07-31, found that B8 was a reporter mislabelled
    /// as a battery: it carried the label `prices match technology`, printed
    /// PASS beside gaps of 4×10⁹, and every certificate in the corpus therefore
    /// asserted something no code in it had checked. The two ways out were to
    /// give it a threshold or to rename it. **It is renamed**, and the argument
    /// is not "there is no evidence for a bar yet" — that was the old argument
    /// and it is the weaker one. It is that *this benchmark cannot carry a bar
    /// at any threshold*, for two reasons that are properties of the benchmark
    /// rather than of the corpus:
    ///
    /// 1. **Its one hand-checked case is a known false positive.** `solv_labour`
    ///    has a closed-form competitive equilibrium, derived on paper and
    ///    registered in `equilibrium.ron` before any run: `p_grain = p_labour`.
    ///    The world is *right* when it sits there, and B8 reads it as
    ///    "2.000x too dear", because the firm earns a capacity rent and a
    ///    zero-rent benchmark cannot see one. A bar at 2× fails a correct world;
    ///    a bar above 2× is a bar chosen to let a known false positive through.
    ///    Neither is a criterion. The defect is in what the statistic measures,
    ///    and no choice of threshold repairs a specification error.
    /// 2. **It carries a second bias of known sign and unknown total.** The
    ///    benchmark is single-node, so haulage is in no good's value
    ///    ([`LabourValues::pass_through`]). On `lr_00` that is one recipe and
    ///    1.200×; on a compiled world (PLAN Phase 8) with stacked routes it is a
    ///    product over the route, and nothing in this module knows the route.
    ///    A bar would be applied to a quantity whose bias the code can bound for
    ///    today's corpus and not for tomorrow's.
    ///
    /// So the label is now `price/technology gap (report, no bar)`, and a PASS
    /// on it means **"the distance was computable"** and nothing else. That is a
    /// real claim with a real failure mode, and the failure mode fires: see
    /// [`Self::failures`] and the R5 sweep below.
    ///
    /// **What would make it a battery**, recorded so the next person does not
    /// have to re-derive it: a rent-aware implied vector (the scarce factor's
    /// shadow price, solved, not assumed) and a per-node valuation. With both,
    /// the implied vector equals the hand-derived `equilibrium.ron` vector on
    /// every `solv_*` tape — it already does on `solv_1g` and `solv_chain`,
    /// which have neither rent nor transport — and a per-scenario bar can then
    /// be registered in that scenario's dated criteria file, against a
    /// derivation rather than against a measurement. Registering one *today*, on
    /// a statistic that is knowingly 2× wrong on the one world whose answer is
    /// known, would be the R6 sin with extra steps.
    ///
    /// **The consequence, stated plainly rather than hidden: a run can be
    /// 4×10⁹ off and this line still says PASS.** It also says 4×10⁹, in words,
    /// with the market named and the direction named, which is what the
    /// instrument is for.
    ///
    /// # What it does fail on
    ///
    /// Three threshold-free cases, all of them R5's fail-closed rule rather than
    /// a bar:
    ///
    /// - a market that traded and never once had a computable relative price
    ///   (including because its numeraire never traded) — [`Self::failures`];
    /// - a tape that does not say how a traded good is made — same;
    /// - any statistic that came out non-finite — [`Self::uncomputable`].
    pub fn report(&self, game_data: &GameData) -> (bool, String) {
        // Fail-closed, and before the deliberate-absence branch: a criteria file
        // naming a good that does not exist scored nothing while reporting a
        // pass, which is R5's failure mode dressed as a convention.
        if let Some(name) = &self.numeraire_missing {
            return (
                false,
                format!(
                    "criteria registers labour_good {name:?}, which this tape does not                      define — nothing was scored (register \"\" for a world with no                      labour market, as big_region and supply_chain do)"
                ),
            );
        }
        if self.labour.is_none() {
            return (
                true,
                "unscored: no labour good registered, so there is no numeraire".into(),
            );
        }
        let failures = self.failures(game_data);
        if !failures.is_empty() {
            let f = &failures[0];
            return (
                false,
                format!(
                    "{} market(s) with no computable distance: {}/{} {}",
                    failures.len(),
                    f.region,
                    f.good,
                    f.reason
                ),
            );
        }

        let readings = self.readings(game_data);
        if readings.is_empty() {
            // Not a pass and not a failure of the economy: nothing traded that
            // this criterion applies to. Said out loud, because an empty scored
            // set reported as "PASS, 0 markets" is how a criterion stops
            // measuring without anyone noticing.
            return (
                true,
                format!(
                    "unscored: no traded good has an implied value ({})",
                    self.values.unresolved(game_data).join("; ")
                ),
            );
        }

        // R5, before anything is phrased: a NaN must never reach a passing line.
        let bad = Self::uncomputable(&readings);
        if !bad.is_empty() {
            return (
                false,
                format!(
                    "{} reading(s) whose statistic is not a number: {}",
                    bad.len(),
                    bad.join(", ")
                ),
            );
        }

        let worst = readings
            .iter()
            .max_by(|a, b| {
                a.log_gap
                    .abs()
                    .partial_cmp(&b.log_gap.abs())
                    .expect("checked finite by the sweep above")
            })
            .expect("non-empty");

        // The headline region is the one whose *shared* factor is largest —
        // i.e. where the numeraire is most suspect — because that is the finding
        // a reader can act on before opening any single market.
        let decomps = self.decompositions(game_data);
        let head = decomps
            .iter()
            .max_by(|a, b| {
                a.common_log
                    .abs()
                    .partial_cmp(&b.common_log.abs())
                    .expect("checked finite by the sweep above")
            })
            .expect("readings are non-empty, so at least one region decomposes");

        // The benchmark's assumptions, on every line. `solv_labour` is the
        // standing reason: its price is CORRECT and this instrument calls it
        // 2.000x too dear, and a reader who does not know what was assumed will
        // read that as a defect — one already did.
        let mut line = String::from(
            "REPORT, no bar (a PASS means the distance was computable). \
             Benchmark: long-run zero-profit, ZERO-RENT, FULL-UTILISATION, \
             SINGLE-NODE — a world with a scarce second factor is read as \
             'too dear' by exactly its rent",
        );
        let hauls = self.values.pass_through(game_data);
        if hauls.is_empty() {
            line.push_str(", and this tape has no pass-through recipe. ");
        } else {
            let shown: Vec<String> = hauls.iter().take(LIST_GOODS).map(|h| h.phrase()).collect();
            line.push_str(&format!("; single-node bias: {}. ", shown.join(", ")));
        }

        line.push_str(&format!(
            "{} {} — a statement about the WAGE, not about a good; residuals {}",
            head.region,
            head.common_phrase(),
            head.residual_phrases(LIST_GOODS).join(", ")
        ));
        if decomps.len() > 1 {
            line.push_str(&format!(" (+{} more region(s))", decomps.len() - 1));
        }

        line.push_str(&format!(
            ". Worst single reading {} at {} (ln {:+.3}, {}, {})",
            worst.phrase(),
            worst.region,
            worst.log_gap,
            worst.sd_phrase(),
            worst.census()
        ));

        // Region-level drops, which no single reading can show: a region where
        // the numeraire went idle on ticks when NO good traded leaves no market
        // to carry the count.
        let idle: u64 = self.numeraire_idle_ticks.iter().sum();
        if idle > 0 {
            let seen: u64 = self.ticks_seen.iter().sum();
            line.push_str(&format!(
                ". Numeraire idle on {idle} of {seen} observed region-tick(s)"
            ));
        }
        line.push('.');

        (true, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::game_data::{KernelParams, SupplyRule};
    use crate::types::good::{GoodDef, MovementType, ShelfLife};
    use crate::types::ids::{MarketNodeId, RegionId};
    use crate::types::market_node::{MarketNodeDef, MarketTier};
    use crate::types::recipe::{InputScaling, RecipeDef, RecipeInput, RecipeOutput, StrategyKind};

    fn good(i: u32, name: &str) -> GoodDef {
        GoodDef {
            id: GoodId(i),
            name: name.into(),
            alpha: 0.1,
            shelf_life: ShelfLife::Indefinite,
            movement_type: MovementType::Physical,
            divisible: true,
            storage_cost_per_tick: 0.0,
        }
    }

    fn recipe(i: u32, name: &str, ins: &[(u32, f64)], outs: &[(u32, f64)]) -> RecipeDef {
        RecipeDef {
            id: crate::types::ids::RecipeId(i),
            name: name.into(),
            inputs: ins
                .iter()
                .map(|(g, q)| RecipeInput {
                    good: GoodId(*g),
                    qty_per_unit: *q,
                    scaling: InputScaling::Variable,
                })
                .collect(),
            outputs: outs
                .iter()
                .map(|(g, q)| RecipeOutput { good: GoodId(*g), qty_per_unit: *q })
                .collect(),
            component_reqs: Vec::new(),
            reversible: false,
            strategy: StrategyKind::CapacityControl,
        }
    }

    fn kernel_params() -> KernelParams {
        KernelParams {
            s: 4,
            eta_up: 0.04,
            eta_dn: 0.05,
            dead: 0.05,
            b_out: 2.0,
            b_cash: 6.5,
            beta: 1.0,
            epsilon: 0.01,
            phi: 0.6180339887498949,
            fill_alpha: 0.25,
            supply_rule: SupplyRule::Inelastic,
        }
    }

    /// A tape with one currency, one labour good and whatever recipes the caller
    /// hands it. Built here rather than loaded, so the arithmetic under test is
    /// the arithmetic the assertions can be computed by hand from.
    fn tape(goods: Vec<GoodDef>, recipes: Vec<RecipeDef>, currency: Option<u32>) -> GameData {
        GameData {
            goods,
            recipes,
            kernel: kernel_params(),
            need_categories: Vec::new(),
            wealth_levels: Vec::new(),
            market_nodes: vec![MarketNodeDef {
                id: MarketNodeId(0),
                tier: MarketTier::Regional,
                region: Some(RegionId(0)),
                currency_good: currency.map(GoodId),
            }],
            channels: Vec::new(),
            regions: vec![crate::state::game_data::RegionDef {
                id: RegionId(0),
                name: "Only".into(),
                market_node: MarketNodeId(0),
                position: None,
            }],
        }
    }

    /// `lr_00`'s chain, by hand: 0.3 labour makes a wheat, one wheat plus 0.2
    /// labour makes a flour, so flour embodies 0.5 and the competitive RealWage
    /// is 1/0.5 = 2.0. This is the number PLAN quotes and the anchor for
    /// everything else in this module.
    #[test]
    fn the_lr_00_chain_comes_out_at_the_hand_computed_numbers() {
        let gd = tape(
            vec![
                good(0, "labour"),
                good(1, "wheat"),
                good(2, "flour"),
                good(3, "GBP"),
            ],
            vec![
                recipe(0, "wheat_farm", &[(0, 0.3)], &[(1, 1.0)]),
                recipe(1, "grain_mill", &[(1, 1.0), (0, 0.2)], &[(2, 1.0)]),
            ],
            Some(3),
        );
        let v = labour_values(&gd, GoodId(0));
        assert_eq!(v.get(GoodId(0)), Value::Labour(1.0), "labour is the unit");
        assert_eq!(v.get(GoodId(1)), Value::Labour(0.3));
        assert_eq!(v.get(GoodId(2)), Value::Labour(0.5));
        assert_eq!(v.get(GoodId(3)), Value::Currency, "GBP is not produced, it is money");
        // The implied RealWage, which is what the criterion is ultimately about.
        assert_eq!(1.0 / v.get(GoodId(2)).relative_price().unwrap(), 2.0);
    }

    /// The transport case, which is why the solver cannot be a topological pass.
    /// `flour_transport` takes flour and 0.1 labour and yields flour: a
    /// pass-through, and treating it as production would define flour in terms
    /// of itself. Flour must still come out at 0.5.
    #[test]
    fn a_good_on_both_sides_is_passing_through_not_being_made() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "wheat"), good(2, "flour"), good(3, "GBP")],
            vec![
                recipe(0, "wheat_farm", &[(0, 0.3)], &[(1, 1.0)]),
                recipe(1, "grain_mill", &[(1, 1.0), (0, 0.2)], &[(2, 1.0)]),
                recipe(2, "flour_transport", &[(2, 1.0), (0, 0.1)], &[(2, 1.0)]),
            ],
            Some(3),
        );
        let v = labour_values(&gd, GoodId(0));
        assert_eq!(v.get(GoodId(2)), Value::Labour(0.5), "transport is not a mill");
    }

    /// …and the 0.1 labour it spends has to be *accounted for* rather than
    /// silently dropped. It is in no good's value, so a competitive importer who
    /// recovers exactly his haulage cost prices flour at 0.6 and this benchmark
    /// calls him 1.2x too dear. That is the size of the single-node bias, with
    /// its sign, computed from the tape.
    #[test]
    fn the_labour_a_pass_through_spends_is_reported_as_the_single_node_bias() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "wheat"), good(2, "flour"), good(3, "GBP")],
            vec![
                recipe(0, "wheat_farm", &[(0, 0.3)], &[(1, 1.0)]),
                recipe(1, "grain_mill", &[(1, 1.0), (0, 0.2)], &[(2, 1.0)]),
                recipe(2, "flour_transport", &[(2, 1.0), (0, 0.1)], &[(2, 1.0)]),
                // Currency in and out: the `dividend` shape. It is a
                // pass-through too, and it must NOT be reported — currency is
                // outside the labour accounting by construction.
                recipe(3, "dividend", &[(3, 1.0)], &[(3, 1.0)]),
            ],
            Some(3),
        );
        let v = labour_values(&gd, GoodId(0));
        let hauls = v.pass_through(&gd);
        assert_eq!(hauls.len(), 1, "{hauls:?}");
        assert_eq!(hauls[0].recipe, "flour_transport");
        assert_eq!(hauls[0].good, "flour");
        assert!((hauls[0].added - 0.1).abs() < 1e-12, "added {}", hauls[0].added);
        assert!((hauls[0].bias() - 1.2).abs() < 1e-12, "bias {}", hauls[0].bias());
    }

    /// Where two recipes make a good, the cheap one sets the price. A producer
    /// on the dear route is competed out; it is not averaged in.
    #[test]
    fn the_cheapest_route_wins_whatever_order_the_recipes_are_listed_in() {
        for dear_first in [true, false] {
            let cheap = recipe(0, "efficient_mill", &[(0, 0.4)], &[(1, 1.0)]);
            let dear = recipe(1, "wasteful_mill", &[(0, 2.0)], &[(1, 1.0)]);
            let recipes = if dear_first { vec![dear, cheap] } else { vec![cheap, dear] };
            let gd = tape(vec![good(0, "labour"), good(1, "flour")], recipes, None);
            let v = labour_values(&gd, GoodId(0));
            assert_eq!(
                v.get(GoodId(1)),
                Value::Labour(0.4),
                "dear_first={dear_first}: the order recipes happen to be listed in \
                 must not decide a price"
            );
        }
    }

    /// Multi-unit outputs divide, so a recipe yielding two units of a good from
    /// one unit of labour halves its content.
    #[test]
    fn output_quantity_divides_the_input_bill() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "bread")],
            vec![recipe(0, "bakery", &[(0, 1.0)], &[(1, 4.0)])],
            None,
        );
        assert_eq!(labour_values(&gd, GoodId(0)).get(GoodId(1)), Value::Labour(0.25));
    }

    /// A good nothing makes has no labour value, and must be *said* to have
    /// none. Handing it a zero, or an average of the goods around it, would put
    /// a fabricated benchmark into a correctness criterion — which is the one
    /// place a fabricated number does the most damage.
    #[test]
    fn an_unmade_good_is_reported_unresolved_never_given_a_value() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "flour"), good(2, "iron")],
            vec![recipe(0, "mill", &[(0, 0.5)], &[(1, 1.0)])],
            None,
        );
        let v = labour_values(&gd, GoodId(0));
        assert_eq!(v.get(GoodId(2)), Value::Unreachable);
        assert_eq!(v.get(GoodId(2)).relative_price(), None);
        let un = v.unresolved(&gd);
        assert_eq!(un.len(), 1, "{un:?}");
        assert!(un[0].starts_with("iron:"), "{un:?}");
    }

    /// A good downstream of an unmade one is unresolved too — the ignorance
    /// propagates, it does not get papered over at the next step.
    #[test]
    fn ignorance_propagates_downstream() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "iron"), good(2, "tools")],
            vec![recipe(0, "smithy", &[(1, 2.0), (0, 1.0)], &[(2, 1.0)])],
            None,
        );
        let v = labour_values(&gd, GoodId(0));
        assert_eq!(v.get(GoodId(1)), Value::Unreachable);
        assert_eq!(v.get(GoodId(2)), Value::Unreachable, "tools cannot be priced off iron");
    }

    /// Currency in, real good out: the currency leg is skipped, so what is left
    /// is a good made from nothing. That is a `Free` good, not a zero-value one
    /// — its competitive relative price is zero and no positive posted price can
    /// match it, so it is excluded rather than scored against an impossible
    /// target.
    #[test]
    fn a_good_made_only_from_currency_is_free_not_cheap() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "GBP"), good(2, "manna")],
            vec![recipe(0, "alchemy", &[(1, 5.0)], &[(2, 1.0)])],
            Some(1),
        );
        let v = labour_values(&gd, GoodId(0));
        assert_eq!(v.get(GoodId(2)), Value::Free);
        assert_eq!(v.get(GoodId(2)).relative_price(), None);
    }

    /// The reason there is a sweep cap at all, shown firing. Two goods each made
    /// from the other at a loss-making ratio have no fixed point: every lap round
    /// the cycle is cheaper than the last, forever. The solver must say it did
    /// not converge rather than hand back the value the last sweep happened to
    /// hold — which would be an artifact of [`MAX_SWEEPS`], i.e. of nothing.
    ///
    /// **The third assertion was reversed on 2026-07-31 (R14, marked in place).**
    /// It used to read:
    ///
    /// ```text
    ///     assert_eq!(v.get(GoodId(0)), Value::NotConverged,
    ///         "and nothing in a graph that never settled is trustworthy, \
    ///          including the goods that looked resolved");
    /// ```
    ///
    /// That was wrong, and it was wrong about the *numeraire*: labour is
    /// `Labour(1.0)` by definition, the relaxation never writes to it, and
    /// marking it unpriced because two other goods were chasing each other made
    /// `PriceGapWatch` unable to score a single market in the whole tape. The
    /// claim it was reaching for — "a value fed by a runaway is not a value" —
    /// is kept, and is now enforced by closing the moving set *downstream*
    /// instead of over the whole tape. See [`Value::NotConverged`].
    #[test]
    fn a_cost_reducing_cycle_is_reported_as_not_converged() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "a"), good(2, "b")],
            vec![
                recipe(0, "seed_a", &[(0, 1.0)], &[(1, 1.0)]),
                // 1 b -> 1 a and 1 a -> 1 b, each yielding double, so a lap
                // round the cycle quarters the cost and never settles.
                recipe(1, "a_from_b", &[(2, 1.0)], &[(1, 2.0)]),
                recipe(2, "b_from_a", &[(1, 1.0)], &[(2, 2.0)]),
            ],
            None,
        );
        let v = labour_values(&gd, GoodId(0));
        assert_eq!(v.sweeps(), MAX_SWEEPS, "the cap must be what stopped it");
        assert_eq!(v.get(GoodId(1)), Value::NotConverged);
        assert_eq!(v.get(GoodId(2)), Value::NotConverged);
        assert_eq!(
            v.get(GoodId(0)),
            Value::Labour(1.0),
            "labour is the unit; a runaway elsewhere in the graph cannot move it"
        );
    }

    /// A well-formed cyclic graph — the one the relaxation exists for — still
    /// converges, and does so in a bounded number of sweeps.
    #[test]
    fn an_ordinary_cycle_converges_well_inside_the_cap() {
        let gd = tape(
            vec![good(0, "labour"), good(1, "iron"), good(2, "tools")],
            vec![
                // Tools need iron, iron needs tools: a genuine cycle with a
                // fixed point, because each lap adds labour rather than removing
                // it. Bootstrapped by a labour-only iron route.
                recipe(0, "dig_iron", &[(0, 2.0)], &[(1, 1.0)]),
                recipe(1, "smithy", &[(1, 1.0), (0, 1.0)], &[(2, 1.0)]),
                recipe(2, "mine_iron", &[(2, 0.1), (0, 1.0)], &[(1, 1.0)]),
            ],
            None,
        );
        let v = labour_values(&gd, GoodId(0));
        assert!(v.sweeps() < MAX_SWEEPS, "took {} sweeps", v.sweeps());
        // dig_iron gives iron 2.0; smithy then gives tools 3.0; mine_iron then
        // gives iron 1 + 0.1*3.0 = 1.3, which re-prices tools at 2.3, which
        // re-prices iron at 1.23, ... converging to iron = 1.1/0.9 and
        // tools = iron + 1.
        let iron = v.get(GoodId(1)).relative_price().unwrap();
        let tools = v.get(GoodId(2)).relative_price().unwrap();
        assert!((iron - 1.1 / 0.9).abs() < 1e-9, "iron {iron}");
        assert!((tools - (1.1 / 0.9 + 1.0)).abs() < 1e-9, "tools {tools}");
    }

    /// The phrase is the product. A distance of 0.917 is not actionable; "flour
    /// 2.503x too dear" names a market and a direction.
    #[test]
    fn the_phrase_names_the_good_and_the_direction() {
        let dear = GapReading {
            region: "Manchester".into(),
            good: "flour".into(),
            implied: 0.5,
            log_gap: (2.5f64).ln(),
            log_sd: Some(0.1),
            samples: 851,
            traded_ticks: 851,
            unpriced: 0,
            numeraire_idle: 0,
        };
        assert_eq!(dear.phrase(), "flour 2.500x too dear");
        let cheap = GapReading { log_gap: -(4.0f64).ln(), ..dear.clone() };
        assert_eq!(cheap.phrase(), "flour 4.000x too cheap");
        // Exactly right reads as "1.000x too dear" rather than as an error; the
        // number, not the word, is the measurement.
        let right = GapReading { log_gap: 0.0, ..dear };
        assert_eq!(right.phrase(), "flour 1.000x too dear");
    }

    /// The R5 sweep, shown firing on each defect it claims to catch — and shown
    /// *not* firing on `log_sd: None`, which is the declared-missing convention
    /// and not a NaN. A guard that fired on both would make every one-sample
    /// market a failure and the distinction meaningless.
    #[test]
    fn the_fail_closed_sweep_fires_on_a_nan_and_not_on_a_declared_absence() {
        let ok = GapReading {
            region: "Manchester".into(),
            good: "flour".into(),
            implied: 0.5,
            log_gap: 0.25,
            log_sd: None,
            samples: 1,
            traded_ticks: 851,
            unpriced: 0,
            numeraire_idle: 850,
        };
        assert!(
            PriceGapWatch::uncomputable(std::slice::from_ref(&ok)).is_empty(),
            "a dispersion that does not exist is an absence, not a NaN"
        );
        assert_eq!(ok.sd_phrase(), "sd n/a (one sample)");
        assert_eq!(
            ok.census(),
            "n=1 of 851 traded tick(s), 850 discarded (0 unpriced, 850 numeraire idle)"
        );

        for bad in [
            GapReading { log_sd: Some(f64::NAN), ..ok.clone() },
            GapReading { log_gap: f64::NAN, ..ok.clone() },
            GapReading { log_gap: f64::INFINITY, ..ok.clone() },
            GapReading { implied: f64::NAN, ..ok.clone() },
        ] {
            let hits = PriceGapWatch::uncomputable(std::slice::from_ref(&bad));
            assert_eq!(hits.len(), 1, "{bad:?} -> {hits:?}");
            assert!(hits[0].starts_with("Manchester/flour "), "{hits:?}");
        }
    }

    /// The decomposition, on the arithmetic it was designed from. Three goods
    /// whose gaps are 100x, 10x and 1000x share a common factor of exactly
    /// (100·10·1000)^(1/3) = 100x, leaving residuals of 1x, 0.1x and 10x — so
    /// the middle good, which reads "10x too dear", is really 10x too *cheap*
    /// once the wage is accounted for. That sign flip is the whole point.
    #[test]
    fn the_decomposition_separates_the_shared_factor_from_the_residuals() {
        let base = GapReading {
            region: "Manchester".into(),
            good: String::new(),
            implied: 1.0,
            log_gap: 0.0,
            log_sd: Some(0.0),
            samples: 10,
            traded_ticks: 10,
            unpriced: 0,
            numeraire_idle: 0,
        };
        let readings: Vec<GapReading> = [("wheat", 100.0), ("flour", 10.0), ("services", 1000.0)]
            .iter()
            .map(|(g, x)| GapReading {
                good: (*g).into(),
                log_gap: f64::ln(*x),
                ..base.clone()
            })
            .collect();
        let common = readings.iter().map(|r| r.log_gap).sum::<f64>() / 3.0;
        assert!((common.exp() - 100.0).abs() < 1e-9, "common {}", common.exp());
        let residual = |g: &str| {
            readings.iter().find(|r| r.good == g).unwrap().log_gap - common
        };
        assert!((residual("wheat").exp() - 1.0).abs() < 1e-9);
        assert!((residual("flour").exp() - 0.1).abs() < 1e-9);
        assert!((residual("services").exp() - 10.0).abs() < 1e-9);
        assert_eq!(phrase_log("flour", residual("flour")), "flour 10.000x too cheap");
        // ...and the residuals sum to zero, which is what makes this a
        // decomposition rather than two unrelated statistics.
        let sum: f64 = ["wheat", "flour", "services"].iter().map(|g| residual(g)).sum();
        assert!(sum.abs() < 1e-12, "residuals must sum to zero, got {sum}");
    }
}
