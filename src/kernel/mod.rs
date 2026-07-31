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

use crate::state::{GameData, SimState};
use crate::types::{
    delta::StateDelta,
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
            continue;
        }
        let (buy_node, sell_node) = nodes(ri, game_data);

        rule_1_sell(state, game_data, ri, recipe, sell_node, orders, deltas);

        if !is_frozen(recipe) && on_phase(state.tick, ri.id.0 as u64, k.s) {
            rule_2_nudge(state, game_data, ri, recipe, buy_node, sell_node, deltas);
        }
    }

    rule_3_buy_and_route(state, game_data, orders);
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
        if let Some(labour) = pop.labour_good {
            let hours = if pop.is_employed {
                pop.size
            } else {
                pop.size * pop.last_labour_fill_rate
            };
            let qty = hours * pop.participation;
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
            let p_basket = subsistence_price(state, game_data, node);
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
            (stock - k.b_out * flow).max(0.0)
        } else {
            // No stock to integrate anything. See the module header: this is
            // kernel.md's rule, and it is the one that keys a posted quantity to
            // a fill.
            (flow * ri.last_fill).max(k.epsilon * flow).min(stock)
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
    buy_node: MarketNodeId,
    sell_node: MarketNodeId,
    deltas: &mut Vec<StateDelta>,
) {
    let k = &game_data.kernel;
    let (revenue, cost) = unit_margin(state, recipe, buy_node, sell_node);
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

/// Buy inputs for the owners whose activation this is.
///
/// Evaluated once per inventory *owner*, not once per desk: desks sharing an
/// inventory share one cash position, and letting each of them size a purchase
/// against the same un-drained balance is how two of them come to spend the
/// same pound. The pass is keyed off the inventory id so the phase is a
/// property of the owner, and iteration is over the inventory index so the
/// order of the emitted deltas never depends on a hash.
fn rule_3_buy_and_route(state: &SimState, game_data: &GameData, orders: &mut Vec<Order>) {
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
