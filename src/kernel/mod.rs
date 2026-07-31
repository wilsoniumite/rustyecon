//! The desk kernel ([kernel.md](../../docs/architecture/kernel.md)).
//!
//! One shape for every actor: a single scale variable, nudged by small
//! asymmetric multiplicative steps, gated by a dead-band on one local pressure
//! signal σ, evaluated only on the desk's stagger phase. What varies by desk
//! kind is which σ it reads and where its overflow routes.
//!
//! This runs *beside* the legacy agent layer, selected by [`AgentArm`], because
//! the two have to be A/B'd against each other under identical scenarios before
//! either is deleted (METHODOLOGY R7 and R10). PLAN Phase 4 removes the loser.
//!
//! # Readings taken where the spec was ambiguous
//!
//! Three, each recorded here and in kernel.md rather than chosen silently.
//!
//! **The cash reserve** is `b_cash · outlay_per_tick / S`, the formula as
//! kernel.md literally writes it.
//!
//! It was implemented as `b_cash · S · outlay_per_tick` first, on the argument
//! that the parameter table calls `b_cash` a band "in activations of outlay" and
//! that `b_cash · S = 6.5 · 4 = 26` matches the legacy dividend desk's
//! registered `reserve_multiple` of 26.0. That argument was wrong, and it is
//! worth keeping the reason: legacy's 26 is a *dividend retention* threshold —
//! pay out cash above 26× smoothed input cost — while Rule 3's reserve is a
//! floor on *spending*. Two different quantities that happen to be denominated
//! the same way.
//!
//! The consequence was decisive rather than subtle. A building in the corpus
//! opens with 50 currency against a per-tick input cost of 8.8; the wrong
//! reading put its reserve at 228.8, so `budget = max(cash − reserve, 0)` was
//! exactly zero, and it bought nothing, produced nothing, earned nothing, and
//! could never climb out. Under the literal formula the reserve is 14.3 and the
//! budget 35.7, against 35.2 needed to fund one activation's inputs — which is
//! close enough to suggest the corpus's genesis cash was sized against this
//! reading.
//!
//! The absorbing state is still real in the general case: any desk that starts
//! below its cash band buys nothing and can never earn its way back. It is not
//! reached at these numbers, and it is recorded rather than guarded, because a
//! guard would hide it from the phase map (PLAN Phase 5) that should find it.
//!
//! **σ, which is not the spec's σ — a defect found by implementing it.** The
//! table gives the producer's pressure signal as
//! `(band_out − inventory_out)/band_out`. That signal cannot expand a desk, and
//! the reason is Rule 1 in the same document.
//!
//! Rule 1 posts *everything* above the band, so the buffer is swept flat every
//! tick and inventory at the next decision is exactly `band + last tick's
//! production`. Substituting, σ rests at `−1/b_out` — with the registered
//! `b_out = 2.0`, a desk selling every unit it offers reads **−0.5**, sails past
//! the −0.05 dead-band, and shrinks. Every activation. Forever. Nothing about
//! the desk being healthy enters the arithmetic; `desk_buffer_sigma_is_pinned`
//! is the proof and [`buffer_sigma`] is kept solely to carry it.
//!
//! Measured before it was diagnosed: under the spec's σ, every desk in `lr_00`
//! falls to `epsilon · size` within ~80 ticks and sits there for the remaining
//! 920 while flour goes 0.6 → 7,898 and wheat goes to 0. Quantities frozen,
//! prices doing all the adjusting.
//!
//! The deeper problem is informational, and no rearrangement of the formula
//! fixes it: once Rule 1 has swept the surplus onto the market, output inventory
//! says nothing about *excess* demand. A desk that sells everything it makes
//! reads identically whether demand is 1× or 100× its output — the quantity
//! signal saturates at "sold out". So [`sigma`] takes each side from the signal
//! that has information: **unsold offers** contract, **margin** expands. Both
//! are own-state, and the fill steers scale rather than the posted quantity, so
//! Rule 1 still names only stock and scale.
//!
//! **Rule 1 was price-inelastic, and that is why no equilibrium price existed.**
//! Recorded at length because it is the largest correction the kernel has taken
//! and because the spec that produced it — `docs/design/price-responsive-supply.md`
//! — is a prediction, not a result, and parts of it are wrong (below).
//!
//! `max(inventory − b_out·flow, 0)` mentions no price, so `ε_s = 0` exactly, for
//! every good, measured. With `ε_d = 0` on labour and services the price loop's
//! gain is `|1 + α(ε_d − ε_s)| = 1` — a unit root. The corpus was not failing a
//! stability *tuning*; it had no fixed point to fail to reach.
//!
//! [`SupplyRule::Reservation`] divides the band by the desk's own markup `R`, so
//! it withholds a fixed *value* of stock and the price decides how many units
//! that value is. Three properties are load-bearing and each has a test:
//! monotone increasing in price, never more than held, and **identical to the
//! shipped rule at `R = 1`** — which is why every zero-profit resting-state
//! result in the repo (all five `solv_*` tapes, mode A) is unchanged bit for bit.
//!
//! `b_out` stops being an inert threshold and becomes the supply curve's slope:
//! at a desk's resting point `ε_s = b_out/R`, so the stability condition
//! `α(ε_s − ε_d) < 2` collapses to `α·b_out < 2R`. Registered: `0.1 × 2 = 0.2`.
//!
//! **The dimensional pick, made deliberately rather than inherited.** kernel.md
//! calls `b_out` a band "in activations of throughput" while `rule_1_sell`
//! computes `flow` per *tick*, so the band is 2 ticks of cover, not 8. Under the
//! old rule that only moved a threshold; under the new one it *is* the
//! elasticity — `ε_s* = 2` or `8`. **Per-tick is kept**, unchanged from the
//! shipped code, for one reason that outranks the others: it is what makes the
//! new rule reduce to the old one exactly at `R = 1`. Changing the functional
//! form and the band width in the same commit would leave no way to attribute
//! either measurement. `α·b_out = 0.2` is a factor of ten inside the boundary;
//! the activation reading (0.8) is also stable and is a Phase 5 sweep point.
//!
//! **WHAT IT MEASURED, 2026-07-31, and the headline is not the good news.**
//!
//! The elasticity is real: on a live state, storable goods read `ε_s` of 0.216
//! to 45.2 (own-price, one `(node, good)` bumped at a time), labour reads 1.67
//! to 3.19 under the full rule, and the uniform all-goods bump reads **exactly
//! 0.000**, which is the degree-0 homogeneity the design requires and the reason
//! the old probe could not have seen any of this.
//!
//! **And on the lr corpus the full rule is a catastrophe: 72 live regions'
//! worth of economy goes to ZERO.** Legacy 33 live, kernel/inelastic 8,
//! kernel/reservation **0**, with `DEAD` tripping in all 72 regions and
//! `POP_DESTITUTION` in all 72. It loses the pre-registered A/B against both the
//! legacy arm and the shipped kernel, on G1.
//!
//! **The decomposition says which half.** [`SupplyRule::ReservationGoods`] — the
//! reservation band on storable outputs, the shipped `π·H` on labour — reads
//! 5 live regions against the shipped kernel's 8, but its median `LevelRange`
//! falls from **3.27e15× to 6.53e8×** and its median B8 price gap from
//! **1.78e11× to 1.61e8×**. On the solvable worlds it does something nothing in
//! this repository had done: displaced by 2× on one price, `solv_1g` under the
//! imbalance rule keeps its market (volume trend **1.134×**, against 0.068
//! under the shipped rule), and `solv_chain`'s median price gap goes from
//! **1.5e11× to 2.12×** against a registered bar of 1.10×.
//!
//! **The mechanism of the collapse, traced rather than guessed.** In `lr_00` the
//! flour market has zero posted supply from tick 0 under *both* rules — that
//! part is pre-existing — so its imbalance is pinned at +1 and the imbalance
//! rule's saturating normaliser marks it up by the full `α` every tick, for
//! ever. `P_basket` is mostly flour, so the pop's reservation wage
//! `w_res = parity · P_basket` inflates at 10%/tick while `p_labour` does not
//! (labour's market is often two-sided-empty, where the price rule freezes).
//! The real wage falls through the shutdown wage `(2/3)·parity` at tick 7 and
//! never returns: labour supply is identically zero from then on, so nothing is
//! produced, and the economy is gone by tick 9.
//!
//! Two things follow, and both are findings rather than excuses. (1) The
//! design's "structural price floor" argument is only half of one: withholding
//! does stop a price falling to zero, but against a *starved* market it converts
//! the failure into an unbounded mark-UP — B7's pins flip from `-1.000000` to
//! `+1.000000` across the corpus. (2) A reservation price indexed to a basket
//! whose own market has collapsed is indexed to a runaway; the rule is degree-0
//! in prices by design, and that is exactly why a uniform runaway in everything
//! *except* the wage cannot be escaped by inflation.
//!
//! **What the spec got wrong, found by implementing it.** §2.2's labour rule
//! `posted = min(H, π·H·max(1 + b_out(1 − w_res/w), 0))` is *not* the
//! generalisation it claims to be for a pop at the π corner. At `w = w_res` it
//! posts `π·H`, as advertised — but π is clamped to 1 in the lr corpus, so the
//! capacity `(1+b_out)·π·H` is cut to `H` by the physical cap and the *upside*
//! is gone: above the reservation wage the pop can post nothing extra, while
//! below it the supply still falls away. The rule is one-sided wherever π = 1,
//! which is where the corpus lives. That is recorded, not patched: the fix is
//! π's, not Rule 1's, and inventing a second capacity term to restore symmetry
//! would be a constant nobody registered.
//!
//! **A second thing the spec got wrong, and this one is structural.** §2's law
//! treats `R = 1` as the desk's indifference point. That is the right
//! reservation price only where the *equilibrium* markup is one. `solv_labour`
//! is the corpus's one world with a scarce second factor: `recipe_size` binds,
//! the firm earns a capacity rent, and its equilibrium markup is **R = 2**. The
//! band is then halved, the desk posts more than it produces, and a
//! hand-computed fixed point that the shipped rule holds to 0 ulp stops being a
//! fixed point at all. The same tape is already the registered counterexample to
//! reading B8 as a correctness verdict; it is now also the counterexample to
//! reading `R = 1` as break-even. A Leontief reservation price cannot see a
//! scarce second factor either. `mode_a_breaks_on_solv_labour_…` asserts it, as
//! a failure, rather than patching it: a rent-aware reservation price is a new
//! mechanism with its own derivation and its own A/B.
//!
//! **What was deliberately NOT implemented.** The spec's §5 mirrors the change
//! onto Rule 3's buying (`target = S·desired·R`). It is left out, and the reason
//! is R10 rather than time: it moves `ε_d`, and this change exists to measure
//! what moving `ε_s` does. Two mechanisms landing together have one receipt
//! between them. §5 is a separate change with its own A/B.
//!
//! **What the tapes register, and why it is the OLD rule.** All 33
//! `game_data.ron` blocks carry `supply_rule: inelastic`. R10 says a change
//! ships when the certified suite says it is no worse, and on the pre-registered
//! gates this one is worse; flipping the tapes is a decision taken on the
//! receipt, not on the way to producing it. `--supply-rule` runs the others, and
//! the effective value is folded into the run identity so no two certificates
//! can be confused.
//!
//! **Fill-keyed posting, which the invariant forbids.** kernel.md's price
//! formation section states that no desk may compute what it offers "from
//! another agent's demand, or from the fill it most recently received" — and
//! then Rule 1 has services post `scale · last_fill`, which is the second of
//! those. The contradiction is real and is not resolved here: Rule 1 is
//! implemented as written, because Phase 4's job is to implement the spec and
//! measure it, not to quietly patch it. What the spec's own analysis predicts
//! is a fixed point at `imbalance = √(d/scale) − 1`, strictly negative below
//! capacity, corrected only by scale moving. Whether it is corrected fast enough
//! is a measurement, and the A/B is the measurement.

use crate::state::game_data::SupplyRule;
use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
    provenance::Provenance,
    ids::{GoodId, InventoryId, MarketNodeId, OwnerId},
    order::{Order, OrderSide},
    recipe::{RecipeDef, StrategyKind},
    recipe_instance::RecipeInstance,
};
use serde::{Deserialize, Serialize};

/// Which agent layer a run uses.
///
/// Recorded in the run certificate, so two certificates in `results/` cannot be
/// mistaken for each other, and folded into the run id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentArm {
    /// The three `StrategyState` strategies plus the pop PD controller.
    #[default]
    Legacy,
    /// The desk kernel: Rules 1–3 over one scale variable.
    Kernel,
}

impl AgentArm {
    pub fn name(&self) -> &'static str {
        match self {
            AgentArm::Legacy => "legacy",
            AgentArm::Kernel => "kernel",
        }
    }
}

impl std::str::FromStr for AgentArm {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "legacy" => Ok(AgentArm::Legacy),
            "kernel" => Ok(AgentArm::Kernel),
            other => Err(format!("unknown agent arm {other:?} (legacy | kernel)")),
        }
    }
}

/// The fractional part of `x`, for `x ≥ 0`.
fn frac(x: f64) -> f64 {
    x - x.floor()
}

/// The deterministic Weyl fraction that gives each desk its own step size.
///
/// `frac(phi · (desk_id + tick/S))` with `phi` irrational is equidistributed in
/// [0,1) and never repeats, so the population gets heterogeneous steps with no
/// RNG and no per-desk state. Integer division on `tick/S` is deliberate: the
/// argument advances once per activation, not once per tick.
pub fn weyl(phi: f64, desk_id: u64, tick: u64, s: u64) -> f64 {
    frac(phi * (desk_id as f64 + (tick / s) as f64))
}

/// Whether this desk acts on this tick. Staggering divides the effective step
/// size by S and desynchronises cobweb spirals deterministically.
pub fn on_phase(tick: u64, id: u64, s: u64) -> bool {
    (tick + id) % s == 0
}

/// Where a desk buys and where it sells. A channel operator spans two nodes;
/// everything else trades at its own region's node.
fn nodes(ri: &RecipeInstance, game_data: &GameData) -> (MarketNodeId, MarketNodeId) {
    match ri.channel {
        Some(ch) => {
            let c = game_data.channel(ch);
            (c.from, c.to)
        }
        None => {
            let home = game_data.region(ri.region).market_node;
            (home, home)
        }
    }
}

/// Whether the kernel governs this instance's scale.
///
/// `DividendPayout` is not a desk — Rule 3's overflow routing replaces it
/// wholesale (PLAN Phase 4, P4.4), and until then the legacy handler keeps
/// paying it so cash still leaves buildings and the arm is comparable.
fn is_desk(recipe: &RecipeDef) -> bool {
    !matches!(recipe.strategy, StrategyKind::DividendPayout { .. })
}

/// A frozen desk posts and buys but never adjusts scale — kernel.md's parcel
/// and government kinds. No scenario in the corpus registers one, so this branch
/// is exercised only by unit tests, and says so rather than implying coverage.
fn is_frozen(recipe: &RecipeDef) -> bool {
    matches!(recipe.strategy, StrategyKind::AlwaysRun)
}

/// Post orders and emit scale decisions for one tick.
///
/// Signature mirrors the legacy `decisions::run` so the two arms are swapped at
/// one call site rather than threaded through the engine.
pub fn run(
    state: &SimState,
    game_data: &GameData,
    deltas: &mut Vec<StateDelta>,
    orders: &mut Vec<Order>,
) {
    let k = &game_data.kernel;

    for ri in &state.recipe_instances {
        let recipe = game_data.recipe(ri.recipe);
        if !is_desk(recipe) {
            // Rule 3's overflow routing has replaced the dividend desk, so its
            // instance must not also run: a GBP-in/GBP-out recipe left with a
            // nonzero scale would move the same cash a second time in the
            // production phase.
            if ri.chosen_size != 0.0 {
                deltas.push(StateDelta::SetChosenSize { instance: ri.id, size: 0.0 });
            }
            continue;
        }
        let (buy_node, sell_node) = nodes(ri, game_data);
        // Computed once and handed to both rules. Rule 1 and Rule 2 must read
        // the *same* R in the same tick, or a desk posts as if profitable while
        // shrinking as if not — the spec's risk 7, and the cheapest possible
        // way to avoid it is to not compute it twice.
        let (revenue, cost) = unit_margin(state, recipe, buy_node, sell_node);

        rule_1_sell(state, game_data, ri, recipe, sell_node, (revenue, cost), orders, deltas);

        if !is_frozen(recipe) && on_phase(state.tick, ri.id.0 as u64, k.s) {
            rule_2_nudge(state, game_data, ri, recipe, sell_node, (revenue, cost), deltas);
        }
    }

    rule_3_buy_and_route(state, game_data, deltas, orders);
    pop_desks(state, game_data, deltas, orders);
}

// ── Pop desks ─────────────────────────────────────────────────────────────────

/// Within-category shares from a multinomial logit on price.
///
/// `share_i ∝ weight_i · exp(−beta · p_i / p̄)`, so a category's spending tilts
/// toward whichever member got cheaper, at a sensitivity the tape registers.
/// Prices are divided by their own mean before the exponential, which is what
/// makes `beta` a pure number instead of something with units of 1/currency —
/// and therefore what keeps it meaningful under any redenomination (R12).
///
/// Replaces the legacy `sub_state`, a per-pop stored allocation that crept
/// toward fixed weights at a capped rate and never looked at a price at all.
/// Every need category in the lr corpus has exactly one entry, so on that
/// corpus this returns `[1.0]` and `beta` is inert; it is written and tested
/// here rather than implied to be exercised.
pub fn logit_shares(weights: &[f64], prices: &[f64], beta: f64) -> Vec<f64> {
    let n = weights.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![1.0];
    }
    let mean = prices.iter().sum::<f64>() / n as f64;
    if !(mean > 0.0) {
        return vec![1.0 / n as f64; n];
    }
    let raw: Vec<f64> = weights
        .iter()
        .zip(prices)
        .map(|(w, p)| w.max(0.0) * (-beta * (p / mean)).exp())
        .collect();
    let total: f64 = raw.iter().sum();
    if total > 0.0 {
        raw.into_iter().map(|r| r / total).collect()
    } else {
        vec![1.0 / n as f64; n]
    }
}

/// Quantity and cost of one tick's basket at a given wealth tier.
fn basket(
    state: &SimState,
    game_data: &GameData,
    node: MarketNodeId,
    wealth: f64,
    size: f64,
) -> (Vec<Vec<f64>>, f64) {
    let per_pop = game_data.interpolated_qty(wealth);
    let mut cost = 0.0;
    let mut per_entry = Vec::with_capacity(game_data.need_categories.len());
    for (ci, cat) in game_data.need_categories.iter().enumerate() {
        let total = per_pop.get(ci).copied().unwrap_or(0.0) * size;
        let weights: Vec<f64> = cat.entries.iter().map(|e| e.weight).collect();
        let prices: Vec<f64> = cat
            .entries
            .iter()
            .map(|e| state.price(node, e.good).max(0.0))
            .collect();
        let shares = logit_shares(&weights, &prices, game_data.kernel.beta);
        let qtys: Vec<f64> = shares.iter().map(|s| s * total).collect();
        for (q, p) in qtys.iter().zip(&prices) {
            cost += q * p;
        }
        per_entry.push(qtys);
    }
    (per_entry, cost)
}

/// The subsistence basket's price at this node — the tier-0 row of the
/// registered table, which is what `parity` is denominated in.
fn subsistence_price(state: &SimState, game_data: &GameData, node: MarketNodeId) -> f64 {
    basket(state, game_data, node, 0.0, 1.0).1
}

/// Pops: the labour pair scaled by participation, and the consumption desk.
///
/// **Buying is per tick, not per activation**, unlike a producer desk. That is
/// forced by the engine rather than chosen: `pop_update` consumes every
/// non-currency good a pop holds at the end of every tick — pops do not
/// stockpile — so a pop that bought four ticks' worth on its activation would
/// have three of them destroyed. Only the two *scales*, π and the wealth tier,
/// are staggered.
fn pop_desks(
    state: &SimState,
    game_data: &GameData,
    deltas: &mut Vec<StateDelta>,
    orders: &mut Vec<Order>,
) {
    let k = &game_data.kernel;
    let max_tier = game_data.max_wealth_tier();

    for pop in &state.pop_groups {
        let node = game_data.region(pop.region).market_node;
        let currency = game_data.market_node(node).currency_good;
        let acting = on_phase(state.tick, pop.id.0 as u64, k.s);

        // ── Rule 1 for labour: the pair rule, scaled by participation ──
        //
        // The employed half posts its full hours, the unemployed half posts
        // fill-scaled hours — that pair *is* Rule 1's fill-stickiness at pop
        // grain — and π scales both. π is the state variable that lets the
        // labour market integrate its flow error, the role inventory plays for
        // a storable.
        // Read once and shared by Rule 1's reservation wage and σ_π's target, so
        // a pop cannot post against one basket price and re-aim π at another.
        // Computed only for the pops that have both a labour good and a
        // registered parity — it walks the whole need table, and before this
        // change it ran on activation ticks only.
        let p_basket = match (pop.labour_good, pop.parity) {
            (Some(_), Some(_)) => subsistence_price(state, game_data, node),
            _ => 0.0,
        };

        if let Some(labour) = pop.labour_good {
            let hours = if pop.is_employed {
                pop.size
            } else {
                pop.size * pop.last_labour_fill_rate
            };
            let qty = match k.supply_rule {
                // `ReservationGoods` keeps the shipped labour posting on
                // purpose: it is the decomposition arm, and its whole job is to
                // hold this branch fixed while the storable branch moves, so the
                // A/B can say which half of the change did what.
                SupplyRule::Inelastic | SupplyRule::ReservationGoods => {
                    hours * pop.participation
                }
                // `parity` is already registered per pop, in baskets per hour and
                // derived from technology, so the reservation wage needs no new
                // constant: `w_res = parity · P_basket`. Both sides are real, so
                // R12 holds under redenomination — a nominal reservation would
                // pin a wage from outside the price system.
                SupplyRule::Reservation => match pop.parity {
                    Some(parity) => labour_posted(
                        hours,
                        pop.participation,
                        k.b_out,
                        state.price(node, labour),
                        parity * p_basket,
                    ),
                    // A tape that registers no parity has no reservation wage to
                    // post against. Falling back to the inelastic rule is the
                    // honest reading and is what the lr corpus would have done
                    // before `parity` landed; it is not a default constant.
                    None => hours * pop.participation,
                },
            };
            if qty > 0.0 {
                orders.push(Order {
                    node,
                    good: labour,
                    side: OrderSide::Sell,
                    owner: OwnerId::PopGroup(pop.id),
                    qty,
                });
            }
        }

        // ── Rule 3 for the consumption desk ──
        let wealth = pop.wealth.clamp(k.epsilon * max_tier, max_tier);
        let (qtys, cost) = basket(state, game_data, node, wealth, pop.size);
        let cash = currency
            .map(|c| state.inventory(pop.inventory).get(c).max(0.0))
            .unwrap_or(f64::INFINITY);
        // The same reading as a producer desk's, so the money-demand anchor
        // that pins the price level is one rule and not two.
        let reserve = k.b_cash * cost / k.s as f64;
        let budget = (cash - reserve).max(0.0);
        // Below the band the pop still eats: subsistence is not discretionary,
        // and a reserve that suppressed buying entirely would starve a pop
        // holding exactly its own savings target.
        let affordable = cash.min(f64::INFINITY);
        let spend = if cost <= budget { cost } else { cost.min(affordable) };
        let ration = if cost > 0.0 { (spend / cost).clamp(0.0, 1.0) } else { 0.0 };

        for (ci, cat) in game_data.need_categories.iter().enumerate() {
            for (ei, entry) in cat.entries.iter().enumerate() {
                let qty = qtys[ci][ei] * ration;
                if qty > 0.0 {
                    orders.push(Order {
                        node,
                        good: entry.good,
                        side: OrderSide::Buy,
                        owner: OwnerId::PopGroup(pop.id),
                        qty,
                    });
                }
            }
        }

        if !acting {
            continue;
        }
        let u = weyl(k.phi, pop.id.0 as u64, state.tick, k.s);

        // ── Rule 2 for the consumption desk ──
        //
        // σ = (cash − reserve)/reserve: sustained income pushes the wealth tier
        // up, sustained shortfall pushes it down. `size` is the basket table's
        // top tier, which is the absorption cap.
        if currency.is_some() && reserve > 0.0 && max_tier > 0.0 {
            let sigma_c = (cash - reserve) / reserve;
            let mut w = wealth;
            if sigma_c > k.dead {
                w *= 1.0 + k.eta_up * u;
            } else if sigma_c < -k.dead {
                w *= 1.0 - k.eta_dn * u;
            }
            let w = w.clamp(k.epsilon * max_tier, max_tier);
            if (w - pop.wealth).abs() > 1e-12 {
                deltas.push(StateDelta::SetPopWealth { pop: pop.id, wealth: w });
            }
        }

        // ── Rule 2 for the labour margin ──
        //
        // σ_π = (w_posted/P_basket − parity)/parity, both sides real. A nominal
        // parity would pin a price from outside the price system (R12), and
        // under uniform deflation the ratio would not move at all.
        if let (Some(labour), Some(parity)) = (pop.labour_good, pop.parity) {
            if p_basket > 0.0 && parity > 0.0 {
                let real_wage = state.price(node, labour) / p_basket;
                let sigma_pi = (real_wage - parity) / parity;
                let mut pi = pop.participation;
                if sigma_pi > k.dead {
                    pi *= 1.0 + k.eta_up * u;
                } else if sigma_pi < -k.dead {
                    pi *= 1.0 - k.eta_dn * u;
                }
                // The labour margin's size is 1 by definition, so π ∈ [ε, 1].
                let pi = pi.clamp(k.epsilon, 1.0);
                if (pi - pop.participation).abs() > 1e-12 {
                    deltas.push(StateDelta::SetPopParticipation {
                        pop: pop.id,
                        participation: pi,
                    });
                }
            }
        }
    }
}

// ── Rule 1 — SELL above the band ──────────────────────────────────────────────

/// The value of stock a desk withholds, expressed in units of the good.
///
/// `band = b_out · flow / R` with `R = revenue/cost` the desk's markup at posted
/// prices, written as `b_out · flow · cost / revenue` so no price is ever a
/// divisor. Equivalently, and this is the sentence that explains it: **the desk
/// withholds stock worth `b_out` ticks of its own input outlay, valued at the
/// posted price.** Dear ticks make that value cheap in units, so the desk
/// releases; cheap ticks make it expensive, so the desk holds.
///
/// Multi-output recipes split the withheld value by *revenue* share, which is
/// what using one recipe-level `R` for every output is. A registered choice, not
/// a discovery: a physical-share split gives different per-good elasticities.
///
/// The two degenerate branches are reachable and both are the economically right
/// answer, so neither is a guard bolted on to avoid a NaN:
///
/// * **`revenue ≤ 0`** — the posted price is zero, so selling gains nothing:
///   band `∞`, post nothing. This is the structural price floor. As `p → 0`
///   supply → 0, imbalance → +1 and the price rule marks *up*; the "price marks
///   itself down forever" mode B7 catches becomes unreachable through this
///   channel without a clamp being added anywhere (R1).
/// * **`cost ≤ 0`** — a desk with no purchased inputs has no reservation price
///   at all: band 0, post everything, `ε_s = 0`, and that market keeps its unit
///   root. Recorded rather than guarded, because a guard would hide it from the
///   Phase 5 phase map that ought to find it. No such desk is registered in the
///   corpus; frozen sources and enclosed parcels are where it will first appear.
///
/// Order matters: `revenue ≤ 0` is tested first, so a desk with neither revenue
/// nor cost withholds rather than dumping. Selling into a zero price is the
/// worse of the two mistakes.
pub fn reservation_band(b_out: f64, flow: f64, revenue: f64, cost: f64) -> f64 {
    if !(revenue > 0.0) {
        return f64::INFINITY;
    }
    if !(cost > 0.0) {
        return 0.0;
    }
    b_out * flow * cost / revenue
}

/// Stock above the band, with an infinite band meaning "post nothing".
///
/// Spelled out rather than left to `(stock − INFINITY).max(0.0)` — which does
/// give 0 — because the day `stock` is itself non-finite that expression is a
/// NaN posted quantity, and R5 says a metric that cannot be computed FAILS
/// rather than propagates.
pub fn posted_above_band(stock: f64, band: f64) -> f64 {
    if !band.is_finite() || !stock.is_finite() {
        return 0.0;
    }
    (stock - band).max(0.0)
}

/// A pop's posted hours under the reservation rule.
///
/// `posted = min(H, π·H · max(1 + b_out·(1 − w_res/w), 0))`, the same shape as
/// the storable band term for term: capacity `(1+b_out)·π·H` capped at the pop's
/// physical hours, reserve `b_out·π·H`, so at the reservation wage the pop posts
/// one unit of throughput out of `1 + b_out` units of capacity and the
/// elasticity is `b_out` wherever π sits.
///
/// The naive `H·max(1 − w_res/w, 0)` was written first and is recorded as
/// rejected: it posts **zero** at `w = w_res`, collapsing the labour market at
/// exactly the wage it should clear at, and at the corpus's π floor of 0.01 its
/// elasticity is `(1−π)/π = 99`, loop gain `|1 − 0.1·99| = 8.9`.
///
/// **The cap bites in the corpus and the rule is one-sided there.** With π = 1
/// the capacity `(1+b_out)·π·H` is cut to `H`, so a pop already offering all its
/// hours cannot offer more when the wage rises — the response exists only below
/// `w_res`. That is a real limitation of the design as specified, not of this
/// implementation, and it is π's job to fix, not Rule 1's.
///
/// `w_res ≤ 0` (a free subsistence basket, or an unregistered `parity`) means no
/// reservation wage exists, so the pop posts its physical hours: the labour
/// analogue of the `cost ≤ 0` desk, and elastic at zero like it.
pub fn labour_posted(hours: f64, pi: f64, b_out: f64, wage: f64, w_res: f64) -> f64 {
    let base = pi * hours;
    if !(w_res > 0.0) {
        return (base * (1.0 + b_out)).min(hours).max(0.0);
    }
    if !(wage > 0.0) {
        // No wage posted at all: the reservation binds absolutely.
        return 0.0;
    }
    let release = (1.0 + b_out * (1.0 - w_res / wage)).max(0.0);
    (base * release).min(hours).max(0.0)
}

/// Post every marketable output above the buffer band.
///
/// The posted quantity names only the desk's own stock and scale. That is what
/// makes `imbalance = (demand − supply)/max(demand, supply)` a *measurement*
/// rather than a restatement of the seller's own decision, which is the defect
/// Phase 3 found in the legacy layer: posting `min(stock, 1.2 × demand)` pinned
/// the imbalance at −1/6 whenever the cap bound, independent of price, and the
/// price marked itself down forever.
fn rule_1_sell(
    state: &SimState,
    game_data: &GameData,
    ri: &RecipeInstance,
    recipe: &RecipeDef,
    sell_node: MarketNodeId,
    (revenue, cost): (f64, f64),
    orders: &mut Vec<Order>,
    deltas: &mut Vec<StateDelta>,
) {
    let k = &game_data.kernel;
    let currency = game_data.market_node(sell_node).currency_good;
    // Scale can sit at zero on a freshly loaded tape; bands measured off it
    // would all be zero and every σ would be 0/0. The floor is the same one
    // Rule 2 clamps to, so a desk at rest is measured against the smallest
    // scale it is allowed to hold rather than against nothing.
    let s_eff = ri.chosen_size.max(k.epsilon * ri.recipe_size);
    let mut worst_fill: Option<f64> = None;

    for output in &recipe.outputs {
        // Currency is not a marketable output: a recipe with money on both
        // sides is a transfer wearing a recipe's clothes, and posting it for
        // sale would be selling pounds for pounds.
        if Some(output.good) == currency {
            continue;
        }
        let stock = state.inventory(ri.output_inv).get(output.good);
        let flow = s_eff * output.qty_per_unit;

        let qty = if k.storable(&game_data.good(output.good).shelf_life) {
            // The band is the low-pass filter. Stock above it is surplus and is
            // offered; stock below it is the buffer that absorbs a demand shock
            // before the shock can reach scale. Because inventory is a *stock*
            // it integrates the flow error, so a persistent `d < s` accumulates
            // somewhere σ can see it.
            //
            // Under `Reservation` the band is additionally divided by the
            // desk's own markup, which is the whole change: the band is then a
            // fixed *value* rather than a fixed quantity, `∂posted/∂ln p = band`
            // and `ε_s = inventory/posted − 1`. At `R = 1` the two branches are
            // the same number, which is why this is a generalisation and not a
            // replacement.
            let band = match k.supply_rule {
                SupplyRule::Inelastic => k.b_out * flow,
                SupplyRule::Reservation | SupplyRule::ReservationGoods => {
                    reservation_band(k.b_out, flow, revenue, cost)
                }
            };
            posted_above_band(stock, band)
        } else {
            // No stock to integrate anything, so the flow itself is the offer:
            // one activation's production, capped by what the desk actually has.
            // Both terms name only this desk's own scale and its own stock.
            //
            // kernel.md's Rule 1 says `scale · last_fill` here. That is dropped,
            // because the same document's price-formation section forbids a desk
            // computing what it offers "from the fill it most recently
            // received", and B6 now enforces that. `last_fill` is still read —
            // by σ, to steer *scale* — which is the placement kernel.md's own
            // analysis argues for: a flow case is corrected by its state
            // variable moving, not by its offer being re-derived every tick.
            //
            // `Reservation` does NOT touch this branch, and that is a positive
            // claim rather than an omission: once a perishable has been made its
            // input cost is *sunk*, an unsold unit is worth nothing at end of
            // tick, and selling at any positive price beats holding what cannot
            // be held. The profit-maximising supply of an already-produced
            // perishable is vertical. A withholding rule here would model a desk
            // destroying its own output. `local_services` therefore keeps a unit
            // root on the supply side, and the pre-registered prediction is that
            // it stays the corpus's worst market and that B7 can still pin it.
            flow.min(stock).max(0.0)
        };

        // The desk's own realized rate on what it posted. Under pro-rata
        // clearing every seller at a node gets the same rate, so the market
        // seller fill *is* this desk's fill; it is carried as own state so the
        // rule reads its own memory rather than the order book.
        let supply = state.supply(sell_node, output.good);
        let demand = state.demand(sell_node, output.good);
        let realized = if supply > 0.0 { (demand / supply).min(1.0) } else { 1.0 };
        worst_fill = Some(worst_fill.map_or(realized, |w: f64| w.min(realized)));

        if qty > 0.0 {
            orders.push(Order {
                node: sell_node,
                good: output.good,
                side: OrderSide::Sell,
                owner: OwnerId::RecipeInstance(ri.id),
                qty,
            });
        }
    }

    if let Some(f) = worst_fill {
        let next = k.fill_alpha * f + (1.0 - k.fill_alpha) * ri.last_fill;
        if (next - ri.last_fill).abs() > 1e-12 {
            deltas.push(StateDelta::SetDeskFill { instance: ri.id, fill: next });
        }
    }
}

// ── Rule 2 — NUDGE scale ──────────────────────────────────────────────────────

/// kernel.md's producer signal: how far the output buffer sits below its band.
///
/// **Kept only to demonstrate why it is not used.** See `sigma` below and the
/// finding recorded in kernel.md: under Rule 1 this quantity is pinned at
/// `−1/b_out` for any desk that sells what it posts, so it never signals
/// expansion and a healthy desk shrinks to the floor. `desk_buffer_sigma_is_pinned`
/// is the standing proof.
pub fn buffer_sigma(band: f64, inventory: f64) -> f64 {
    if band <= 0.0 {
        return 0.0;
    }
    (band - inventory) / band
}

/// The pressure signal a desk actually reads.
///
/// Two one-sided signals, each used where it carries information:
///
/// * **Contraction — unsold offers.** `fill` is the desk's own EMA of the rate
///   it realized on what it posted. Below 1 it offered more than the market
///   took, which is the only own-state evidence of a glut once Rule 1 has
///   already swept everything above the band onto the market.
/// * **Expansion — margin.** A desk that sells out learns only `d ≥ s`; the
///   quantity signal saturates and cannot distinguish demand at 1× its output
///   from demand at 100×. What does distinguish them is the price, through the
///   margin — which kernel.md already makes the producer's expansion gate.
///
/// Both are own-state, so the invariant holds: `fill` steers *scale*, never the
/// posted quantity. Rule 1 still names only stock and scale.
pub fn sigma(fill: f64, relative_margin: f64, dead: f64) -> f64 {
    if fill < 1.0 - dead {
        // Offers came back unsold. Report the shortfall, so a market taking
        // half of what was offered contracts harder than one taking 95%.
        fill - 1.0
    } else {
        relative_margin
    }
}

/// Revenue and input cost per unit of recipe size, at posted prices.
fn unit_margin(
    state: &SimState,
    recipe: &RecipeDef,
    buy_node: MarketNodeId,
    sell_node: MarketNodeId,
) -> (f64, f64) {
    let revenue: f64 = recipe
        .outputs
        .iter()
        .map(|o| state.price(sell_node, o.good) * o.qty_per_unit)
        .sum();
    let cost: f64 = recipe
        .inputs
        .iter()
        .map(|i| state.price(buy_node, i.good) * i.qty_per_unit)
        .sum();
    (revenue, cost)
}

/// One activation's scale step.
fn rule_2_nudge(
    state: &SimState,
    game_data: &GameData,
    ri: &RecipeInstance,
    recipe: &RecipeDef,
    sell_node: MarketNodeId,
    (revenue, cost): (f64, f64),
    deltas: &mut Vec<StateDelta>,
) {
    let k = &game_data.kernel;
    let margin = revenue - cost;
    let reference = ((revenue + cost) / 2.0).max(1e-12);

    // Nothing marketable to be under pressure about.
    let currency = game_data.market_node(sell_node).currency_good;
    if !recipe.outputs.iter().any(|o| Some(o.good) != currency) {
        return;
    }
    let sigma = sigma(ri.last_fill, margin / reference, k.dead);

    let u = weyl(k.phi, ri.id.0 as u64, state.tick, k.s);
    let mut scale = ri.chosen_size;
    if sigma > k.dead && margin > 0.0 {
        // Expansion is gated on margin; contraction is not. A desk losing money
        // must always be able to shrink, or the gate becomes a trap.
        scale *= 1.0 + k.eta_up * u;
    } else if sigma < -k.dead {
        scale *= 1.0 - k.eta_dn * u;
    }
    // Clamped every activation, not only when a step fired: the bound is on the
    // state variable, and it is what lifts a desk sitting at exactly zero back
    // to a scale from which it can observe a recovering market.
    let scale = scale.clamp(k.epsilon * ri.recipe_size, ri.recipe_size);

    if (scale - ri.chosen_size).abs() > 1e-12 {
        deltas.push(StateDelta::SetChosenSize { instance: ri.id, size: scale });
    }
}

// ── Rule 3 — BUY below the band ───────────────────────────────────────────────

/// Where an owner's overflow goes: the inventories of its claim holders.
///
/// Read from the legacy `DividendPayout` wiring — `input_inv` is the firm,
/// `output_inv` is the holder — because that wiring already encodes exactly the
/// pointer Rule 3 needs, and Phase 6's `OwnershipRegister` is what replaces it
/// with real fractional claims. Until then a firm's holders are whoever its
/// dividend instances pointed at, and the split is even rather than pro-rata by
/// share, because no share sizes are registered anywhere yet.
fn claim_holders(state: &SimState, game_data: &GameData, inv: InventoryId) -> Vec<InventoryId> {
    state
        .recipe_instances
        .iter()
        .filter(|ri| ri.input_inv == inv && !is_desk(game_data.recipe(ri.recipe)))
        .map(|ri| ri.output_inv)
        .collect()
}

/// Buy inputs for the owners whose activation this is.
///
/// Evaluated once per inventory *owner*, not once per desk: desks sharing an
/// inventory share one cash position, and letting each of them size a purchase
/// against the same un-drained balance is how two of them come to spend the
/// same pound. The pass is keyed off the inventory id so the phase is a
/// property of the owner, and iteration is over the inventory index so the
/// order of the emitted deltas never depends on a hash.
fn rule_3_buy_and_route(
    state: &SimState,
    game_data: &GameData,
    deltas: &mut Vec<StateDelta>,
    orders: &mut Vec<Order>,
) {
    let k = &game_data.kernel;

    for inv_idx in 0..state.inventories.len() {
        let inv = InventoryId(inv_idx as u32);
        if !on_phase(state.tick, inv_idx as u64, k.s) {
            continue;
        }
        let residents: Vec<&RecipeInstance> = state
            .recipe_instances
            .iter()
            .filter(|ri| ri.input_inv == inv && is_desk(game_data.recipe(ri.recipe)))
            .collect();
        if residents.is_empty() {
            continue;
        }

        // Every resident buys at its own node; they agree except for channel
        // operators, and the currency is a property of the node.
        let buy_node = nodes(residents[0], game_data).0;
        let Some(currency) = game_data.market_node(buy_node).currency_good else {
            // No currency system: nothing rations purchases, so each desk asks
            // for what it needs and clearing rations in kind.
            for ri in &residents {
                post_input_orders(state, game_data, ri, 1.0, orders, None);
            }
            continue;
        };

        let cash = state.inventory(inv).get(currency).max(0.0);
        let outlay: f64 = residents
            .iter()
            .map(|ri| outlay_per_tick(state, game_data, ri, currency))
            .sum();

        // kernel.md's formula, literally. See the module header for the reading
        // that was tried first, why it was wrong, and what it did.
        let reserve = k.b_cash * outlay / k.s as f64;
        let budget = (cash - reserve).max(0.0);

        // What an activation's production actually needs, net of stock already
        // held — S ticks' worth, because production runs every tick and this
        // desk will not be back until its next phase.
        let want: f64 = residents
            .iter()
            .map(|ri| planned_spend(state, game_data, ri, currency))
            .sum();

        // Pro-rata across inputs when cash is short, so scarcity is shared
        // rather than resolved by whichever desk the loop reached first.
        let ration = if want > budget && want > 0.0 { budget / want } else { 1.0 };
        for ri in &residents {
            post_input_orders(state, game_data, ri, ration, orders, Some(currency));
        }

        // Rule 3, second half: cash above the band leaves the firm.
        //
        // This IS the dividend system, and it is the reason cash cannot pool in
        // a firm by construction rather than by a separate desk deciding to pay
        // out. It is taken here, in the decisions phase, from cash the desk has
        // already reserved and already committed — `overflow` is what is left
        // over after both — so it can never strand a purchase this desk just
        // posted.
        let planned = want * ration;
        let overflow = (cash - reserve - planned).max(0.0);
        if overflow > 1e-12 {
            let holders = claim_holders(state, game_data, inv);
            if !holders.is_empty() {
                let share = overflow / holders.len() as f64;
                let life = game_data.good(currency).shelf_life.initial_life();
                for h in holders {
                    deltas.push(StateDelta::RemoveFromInventory {
                        inv,
                        good: currency,
                        qty: share,
                        prov: Provenance::Transfer,
                    });
                    deltas.push(StateDelta::AddToInventory {
                        inv: h,
                        good: currency,
                        qty: share,
                        life,
                        prov: Provenance::Transfer,
                    });
                }
            }
        }
    }
}

/// Input cost of one tick of production at the desk's current scale.
fn outlay_per_tick(
    state: &SimState,
    game_data: &GameData,
    ri: &RecipeInstance,
    currency: GoodId,
) -> f64 {
    let (buy_node, _) = nodes(ri, game_data);
    game_data
        .recipe(ri.recipe)
        .inputs
        .iter()
        .filter(|i| i.good != currency)
        .map(|i| i.desired(ri.chosen_size, ri.recipe_size) * state.price(buy_node, i.good))
        .sum()
}

/// What this desk intends to spend this activation: `S` ticks of inputs, less
/// what it already holds.
fn planned_spend(
    state: &SimState,
    game_data: &GameData,
    ri: &RecipeInstance,
    currency: GoodId,
) -> f64 {
    let k = &game_data.kernel;
    let (buy_node, _) = nodes(ri, game_data);
    game_data
        .recipe(ri.recipe)
        .inputs
        .iter()
        .filter(|i| i.good != currency)
        .map(|i| {
            let need = i.desired(ri.chosen_size, ri.recipe_size) * k.s as f64;
            let held = state.inventory(ri.input_inv).get(i.good);
            (need - held).max(0.0) * state.price(buy_node, i.good)
        })
        .sum()
}

/// Emit this desk's buy orders, scaled by the owner's cash ration.
fn post_input_orders(
    state: &SimState,
    game_data: &GameData,
    ri: &RecipeInstance,
    ration: f64,
    orders: &mut Vec<Order>,
    currency: Option<GoodId>,
) {
    let k = &game_data.kernel;
    let (buy_node, _) = nodes(ri, game_data);
    for input in &game_data.recipe(ri.recipe).inputs {
        if Some(input.good) == currency {
            continue;
        }
        let need = input.desired(ri.chosen_size, ri.recipe_size) * k.s as f64;
        let held = state.inventory(ri.input_inv).get(input.good);
        let qty = (need - held).max(0.0) * ration;
        if qty > 0.0 {
            orders.push(Order {
                node: buy_node,
                good: input.good,
                side: OrderSide::Buy,
                owner: OwnerId::RecipeInstance(ri.id),
                qty,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHI: f64 = 0.6180339887498949;

    #[test]
    fn every_desk_acts_exactly_once_per_stagger_period() {
        let s = 4;
        for id in 0..12u64 {
            let hits: Vec<u64> = (100..100 + s).filter(|t| on_phase(*t, id, s)).collect();
            assert_eq!(hits.len(), 1, "desk {id} acted {hits:?} times in one period");
        }
    }

    #[test]
    fn the_stagger_spreads_desks_across_the_phase() {
        // The point of staggering is that desks do not all move on the same
        // tick. With S=4 and 12 desks, each tick should carry a quarter of them.
        let s = 4;
        for tick in 0..8u64 {
            let acting = (0..12u64).filter(|id| on_phase(tick, *id, s)).count();
            assert_eq!(acting, 3, "tick {tick}");
        }
    }

    #[test]
    fn the_weyl_step_is_heterogeneous_and_never_repeats() {
        let s = 4;
        // Different desks on the same activation get different step sizes.
        let steps: Vec<f64> = (0..8u64).map(|id| weyl(PHI, id, 0, s)).collect();
        for i in 0..steps.len() {
            for j in i + 1..steps.len() {
                assert!((steps[i] - steps[j]).abs() > 1e-6, "desks {i} and {j} share a step");
            }
        }
        // And one desk's steps do not settle into a cycle.
        let own: Vec<f64> = (0..40u64).map(|a| weyl(PHI, 3, a * s, s)).collect();
        for i in 0..own.len() {
            for j in i + 1..own.len() {
                assert!((own[i] - own[j]).abs() > 1e-9, "activations {i} and {j} repeat");
            }
        }
        assert!(own.iter().all(|u| (0.0..1.0).contains(u)));
    }

    #[test]
    fn a_rational_phi_would_have_repeated_which_is_why_it_is_rejected() {
        // The negative control for the test above: with phi = 1/2 the sequence
        // is period 2, so stagger would be synchronised updating in disguise.
        let own: Vec<f64> = (0..8u64).map(|a| weyl(0.5, 3, a * 4, 4)).collect();
        assert!((own[0] - own[2]).abs() < 1e-12, "expected a period-2 cycle: {own:?}");
    }

    /// The standing proof that kernel.md's producer σ cannot expand a desk.
    ///
    /// One desk, storable output, healthy: it sells every unit it offers and
    /// produces at capacity every tick. Rule 1 posts `inventory − band`, so the
    /// recursion is
    ///
    ///     post   = max(I − band, 0)      (all of it sells)
    ///     I_next = I − post + production
    ///
    /// which settles at `I = band + production` — and the spec's σ then reads
    /// `−production/band = −1/b_out`, however well the desk is doing.
    #[test]
    fn desk_buffer_sigma_is_pinned_below_the_dead_band_for_a_healthy_desk() {
        let (b_out, dead) = (2.0, 0.05);
        let (scale, qty_out) = (10.0, 1.0);
        let band = b_out * scale * qty_out;
        let production = scale * qty_out;

        let mut inventory = 0.0_f64;
        let mut sigma = 0.0;
        for _ in 0..200 {
            sigma = buffer_sigma(band, inventory);
            let posted = (inventory - band).max(0.0);
            inventory = inventory - posted + production; // every unit posted sells
        }

        assert!(
            (sigma - (-1.0 / b_out)).abs() < 1e-12,
            "a healthy desk should rest at -1/b_out, got {sigma}"
        );
        assert!(
            sigma < -dead,
            "and that is below the dead-band, so it contracts: {sigma} < {}",
            -dead
        );
        // Not a scale artifact: the band and production both scale with `scale`,
        // so shrinking cannot escape it. This is why the desks hit the floor.
        for smaller in [1.0, 0.1, 0.01] {
            let band = b_out * smaller * qty_out;
            let inv = band + smaller * qty_out;
            assert!((buffer_sigma(band, inv) - (-1.0 / b_out)).abs() < 1e-12);
        }
    }

    #[test]
    fn the_replacement_sigma_is_two_sided() {
        let dead = 0.05;
        // Sold out and profitable: expand.
        assert!(sigma(1.0, 0.30, dead) > dead);
        // Sold out but margins competed away: hold.
        assert!(sigma(1.0, 0.01, dead).abs() <= dead);
        // Sold out at a loss: contract.
        assert!(sigma(1.0, -0.30, dead) < -dead);
        // Half the offers came back: contract, and harder than a near miss.
        assert!(sigma(0.5, 5.0, dead) < -dead);
        assert!(sigma(0.5, 5.0, dead) < sigma(0.8, 5.0, dead));
        // A glut overrides a fat paper margin — the goods did not sell.
        assert!(sigma(0.2, 100.0, dead) < 0.0);
        // Inside the fill dead-band, the margin decides again.
        assert!(sigma(0.99, 0.30, dead) > dead);
    }

    #[test]
    fn the_arm_flag_round_trips_and_rejects_nonsense() {
        use std::str::FromStr;
        assert_eq!(AgentArm::from_str("legacy").unwrap(), AgentArm::Legacy);
        assert_eq!(AgentArm::from_str("kernel").unwrap(), AgentArm::Kernel);
        assert_eq!(AgentArm::default(), AgentArm::Legacy);
        assert!(AgentArm::from_str("desk").is_err());
    }
}
