//! The project's first **correctness** criterion (v2 Phase 4; PLAN, "The
//! missing instrument").
//!
//! Every other criterion in this repository asks *did it stay put*. `DRIFTING`,
//! `SWINGING`, `UNSTABLE`, `DEAD` are all stability rules, and a world that
//! settles instantly on nonsense passes all of them. Nothing has ever asked *did
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
//! It is a *long-run, full-utilisation, zero-profit* benchmark. It is not a
//! prediction of the tick-by-tick path, and a run whose prices sit some distance
//! from it is not thereby broken — real economies carry margins, rents and
//! disequilibrium. What the distance buys is a *direction and a magnitude* for
//! each market: "flour 2.5× too dear" is a lead, where "the band was 11.5×" is
//! only a complaint.
//!
//! Three simplifications are load-bearing and are stated rather than buried:
//!
//! 1. **Capital is ignored.** `component_reqs` are a stock requirement, not a
//!    flow input, and amortising them needs a depreciation rate that no tape
//!    registers. The values here are therefore labour-*flow* values. No scenario
//!    in the corpus declares a component requirement, so nothing currently
//!    depends on the choice; a world that does will need this revisited, and
//!    that is a design event, not a tweak.
//! 2. **Input scaling is read at nameplate.** `Fixed` and `SemiVariable` inputs
//!    are per-unit only at full utilisation, which is the regime this benchmark
//!    is *about*. `tracer_2r`'s `grow_grain` is the corpus's one `Fixed` input,
//!    and below full size it really does draw more labour per grain than this
//!    says — so the benchmark flatters that scenario's producer, in the
//!    direction of making grain look *dearer* than the value implies. Recorded
//!    because it is an error with a known sign.
//! 3. **Joint products each carry the whole input bill.** A recipe with two
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
/// A cycle that lowers cost each time round (make A from B, B from A, cheaper
/// every lap) has no fixed point at all, and without a cap it spins forever.
/// Hitting the cap is reported as [`Value::NotConverged`] rather than returning
/// whatever the last sweep happened to hold: a value that is still moving is not
/// a value.
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

// ── 1. Labour values ──────────────────────────────────────────────────────────

/// What the recipe graph says one unit of a good is worth in labour, or why it
/// says nothing.
///
/// An enum rather than an `Option<f64>` because the three ways of having no
/// value are three different findings, and collapsing them would hide which one
/// a world has. METHODOLOGY R5's fail-closed rule applies to all three equally,
/// but the *diagnosis* differs: `Unreachable` is a modelling gap in the tape,
/// `Free` is a technology that makes something from nothing, and `NotConverged`
/// is a defect in the technology graph itself.
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
    /// The relaxation was still finding cheaper routes when [`MAX_SWEEPS`] ran
    /// out. Reported, never rounded off.
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
///   not a definition of anything.
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

    let mut sweeps = 0;
    let mut converged = false;
    for _ in 0..MAX_SWEEPS {
        sweeps += 1;
        let mut changed = false;
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
                }
            }
        }
        if !changed {
            converged = true;
            break;
        }
    }

    let values = (0..n)
        .map(|i| {
            if currency[i] {
                Value::Currency
            } else if !converged {
                // Conservative on purpose: a graph that never settled has no
                // trustworthy values anywhere in it, because a route that is
                // still getting cheaper feeds everything downstream of it.
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
    pub log_sd: f64,
    /// Ticks in the window where the ratio existed.
    pub samples: u64,
    /// Ticks in the window where the market traded but the ratio had no
    /// logarithm — a zero, negative or non-finite price on either leg.
    pub unpriced: u64,
}

impl GapReading {
    /// The gap as a multiple, with its direction named.
    ///
    /// A bare distance is useless for diagnosis: `0.917` says nothing a reader
    /// can act on, `flour 2.503x too dear` names the market and which way to
    /// look. This is the whole reason B8 is worth having over the existing
    /// `RealWage` band.
    pub fn phrase(&self) -> String {
        let x = self.log_gap.exp();
        if x >= 1.0 {
            format!("{} {:.3}x too dear", self.good, x)
        } else {
            format!("{} {:.3}x too cheap", self.good, 1.0 / x)
        }
    }
}

/// A (region, good) pair the criterion could not be applied to at all.
#[derive(Debug, Clone)]
pub struct GapFailure {
    pub region: String,
    pub good: String,
    pub reason: String,
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
    traded: Vec<bool>,
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
        let n = regions.len() * game_data.num_goods();
        Self {
            num_goods: game_data.num_goods(),
            regions,
            labour,
            values,
            count: vec![0; n],
            mean: vec![0.0; n],
            m2: vec![0.0; n],
            unpriced: vec![0; n],
            traded: vec![false; n],
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

    /// Take one tick's reading of every market that traded.
    ///
    /// The trading gate is [`crate::certify::invariants::BalanceWatch`]'s
    /// convention: a market with neither supply nor demand posts a stale price
    /// that no agent acted on, and counting it would let a world be scored on
    /// prices nobody paid.
    pub fn observe(&mut self, state: &SimState) {
        let Some(labour) = self.labour else { return };
        for (ri, (_, node)) in self.regions.iter().enumerate() {
            let p_labour = state.price(*node, labour);
            for good_idx in 0..self.num_goods {
                let good = GoodId(good_idx as u32);
                if !self.scored(good) {
                    continue;
                }
                if state.supply(*node, good) <= 0.0 && state.demand(*node, good) <= 0.0 {
                    continue;
                }
                let i = ri * self.num_goods + good_idx;
                self.traded[i] = true;
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
                let sd = if self.count[i] < 2 {
                    f64::NAN
                } else {
                    (self.m2[i] / (self.count[i] - 1) as f64).sqrt()
                };
                out.push(GapReading {
                    region: name.clone(),
                    good: game_data.goods[good_idx].name.clone(),
                    implied,
                    log_gap: self.mean[i],
                    log_sd: sd,
                    samples: self.count[i],
                    unpriced: self.unpriced[i],
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
                if !self.traded[i] {
                    continue;
                }
                let good = GoodId(good_idx as u32);
                let value = self.values.get(good);
                let reason = if value.relative_price().is_none() {
                    format!("implied value unavailable — {}", value.why())
                } else if self.count[i] == 0 {
                    format!(
                        "no computable relative price in {} traded tick(s)",
                        self.unpriced[i]
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

    /// B8's verdict and its measurement.
    ///
    /// # Why there is no distance threshold, and why that was the hard call
    ///
    /// A threshold on the gap would be a registered constant (R2) and would have
    /// to be picked *before* seeing results (R6). The numbers available when
    /// this was written were `legacy α×1` at ~300×, `kernel α×1` at ~4e9× and
    /// `legacy α×0.01` at ~2.5× — so any bar between about 3× and 200× would
    /// have separated the corpus into "the run I like" and "the runs I do not",
    /// and picking one would have been fitting a criterion to a result. That is
    /// precisely the R6 sin, and it is worse here than usual: this is the *first*
    /// correctness criterion, so a threshold chosen now would silently define
    /// what correctness means for everything after it.
    ///
    /// There is also no defensible bar available from outside the data. A
    /// competitive-equilibrium benchmark is a limit, not a tolerance; real
    /// economies carry margins, rents and transport wedges, and nothing in this
    /// project yet says how big those should be. A "2×" would be a number
    /// somebody made up.
    ///
    /// So B8 **reports always and fails only on the threshold-free case**: a
    /// market where the distance cannot be computed. That is the house
    /// fail-closed rule (engine.md, "NaN = FAIL") applied to a new metric, not a
    /// new bar — and it is falsifiable, which is the test that matters: a run
    /// where a traded good never has a relative price fails, and
    /// `tests/test_11_correctness.rs` shows it doing so.
    ///
    /// The consequence is stated plainly rather than hidden: **a run can be
    /// 4×10⁹ off and still see B8 PASS.** The number is in the certificate
    /// either way, which is the point — the instrument was missing, and an
    /// instrument that reports is worth more than a bar that flatters. When the
    /// corpus has a region that is both alive and in-band, a dated criteria file
    /// can register a bar against *that* evidence.
    pub fn report(&self, game_data: &GameData) -> (bool, String) {
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

        let worst = readings
            .iter()
            .max_by(|a, b| {
                a.log_gap
                    .abs()
                    .partial_cmp(&b.log_gap.abs())
                    .expect("readings are finite by construction")
            })
            .expect("non-empty");

        // Per good, the worst region — a distance per good, as the criterion
        // specifies, rather than one number that hides which market is wrong.
        let mut per_good: Vec<&GapReading> = Vec::new();
        for r in &readings {
            match per_good.iter_mut().find(|g| g.good == r.good) {
                Some(slot) => {
                    if r.log_gap.abs() > slot.log_gap.abs() {
                        *slot = r;
                    }
                }
                None => per_good.push(r),
            }
        }
        // Ordered worst-first and truncated, so the line stays readable in a
        // world with fifty goods. The full table is [`Self::readings`]; this is
        // the certificate's headline, not its dataset.
        per_good.sort_by(|a, b| {
            b.log_gap
                .abs()
                .partial_cmp(&a.log_gap.abs())
                .expect("readings are finite by construction")
        });
        let shown = per_good.len().min(LIST_GOODS);
        let mut list: Vec<String> = per_good[..shown].iter().map(|r| r.phrase()).collect();
        if per_good.len() > shown {
            list.push(format!("+{} more", per_good.len() - shown));
        }

        (
            true,
            format!(
                "worst {} at {} (ln {:+.3} sd {:.3}, n={}); {}",
                worst.phrase(),
                worst.region,
                worst.log_gap,
                worst.log_sd,
                worst.samples,
                list.join(", ")
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::good::{GoodDef, MovementType, ShelfLife};
    use crate::types::ids::{MarketNodeId, RegionId};
    use crate::types::market_node::{MarketNodeDef, MarketTier};
    use crate::types::recipe::{InputScaling, RecipeDef, RecipeInput, RecipeOutput, StrategyKind};
    use crate::state::game_data::{KernelParams, SupplyRule};

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
            Value::NotConverged,
            "and nothing in a graph that never settled is trustworthy, including \
             the goods that looked resolved"
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
            log_sd: 0.1,
            samples: 851,
            unpriced: 0,
        };
        assert_eq!(dear.phrase(), "flour 2.500x too dear");
        let cheap = GapReading { log_gap: -(4.0f64).ln(), ..dear.clone() };
        assert_eq!(cheap.phrase(), "flour 4.000x too cheap");
        // Exactly right reads as "1.000x too dear" rather than as an error; the
        // number, not the word, is the measurement.
        let right = GapReading { log_gap: 0.0, ..dear };
        assert_eq!(right.phrase(), "flour 1.000x too dear");
    }
}
