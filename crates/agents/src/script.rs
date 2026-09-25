//! The scripted actor (docs/ENGINE.md §4): fixed lines and a Leontief recipe, every quantity and
//! rate a registered param read at use time, so a dated `SetParam` retargets it.
//!
//! It never produces an order admission refuses or a burn that falls short, whatever the rates:
//!
//! - **Budgets.** Payouts come first, each capped by what is left of the actor's cash, walked by
//!   the same single-lot subtraction the inventory and admission perform. The spending total is
//!   `B = share(spend)·cash_after`; each buy line but the last gets `min(w·B, rem)` and the last
//!   `min(max_remainder(B, Σ earlier), rem)`, walking `rem` down the same way. Other actors'
//!   phase-1 transfers only add to the cash admission sees, and rounding is monotone, so every
//!   budget fits there, even when `share` rounds to exactly 1.
//! - **Sells.** Lines post in (node, good) order, each at most what a copy of the holding still
//!   holds after the lines before it, taken the way admission takes it.
//! - **Recipes.** `x = min(flow(capacity), min_k max_scale(held_k, a_k))`, so each burn `a_k·x`
//!   fits its holding; July needed a shared-input reservation for this
//!   (`v2p3: systems/production/mod.rs:34-77`).

use crate::behaviour::{AgentError, Behaviour, Decision, View};
use crate::ext::{Agents, ScriptState};
use crate::spec::{Script, SellQty};
use rustyecon_core::num;
use rustyecon_core::{
    ActorId, Amount, FlowPerYear, Holder, Inventory, Provenance, RatePerYear, StateDelta,
};
use rustyecon_markets::{Order, Side};

/// Split `total` by `weights` in order: each part but the last is `fl(w·total)`, the last is
/// the largest remainder that keeps the parts' left fold within `total`. Each part is then capped
/// by what `dry` still holds of `currency` and taken from it. Returns the parts.
fn split(
    total: f64,
    weights: impl ExactSizeIterator<Item = f64>,
    dry: &mut Inventory,
    currency: rustyecon_core::GoodId,
) -> Result<Vec<f64>, AgentError> {
    let n = weights.len();
    let mut parts = Vec::with_capacity(n);
    let mut spent = 0.0;
    for (i, w) in weights.enumerate() {
        let nominal = if i + 1 < n {
            w * total
        } else if spent < total {
            num::max_remainder(total, spent)?
        } else {
            0.0
        };
        let part = nominal.min(dry.get(currency));
        if part > 0.0 {
            dry.take(currency, Amount::Qty(part))?;
        }
        spent += part;
        parts.push(part);
    }
    Ok(parts)
}

impl Behaviour for Script {
    type Own = ScriptState;

    fn decide(&self, v: &View<'_, ScriptState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        if !v.own_state.active {
            return Ok(out);
        }
        let me = Holder::Actor(v.me);
        let mut dry = v.own.clone();
        // Payouts first, from the cash held when the phase began.
        if let Some(p) = &self.payout {
            let share = v.clock.share(v.params.get::<RatePerYear>(p.rate)?);
            let total = share * dry.get(v.currency);
            let parts = split(total, p.to.iter().map(|(_, w)| *w), &mut dry, v.currency)?;
            for ((to, _), part) in p.to.iter().zip(parts) {
                if part > 0.0 {
                    out.deltas.push(StateDelta::Transfer {
                        from: me,
                        to: Holder::Actor(*to),
                        good: v.currency,
                        amount: Amount::Qty(part),
                    });
                }
            }
        }
        // Buy lines, in (node, good) order, budgeting what the payouts left.
        if let Some(spend) = self.spend {
            let share = v.clock.share(v.params.get::<RatePerYear>(spend)?);
            let total = share * dry.get(v.currency);
            let budgets = split(
                total,
                self.buy.iter().map(|l| l.weight),
                &mut dry,
                v.currency,
            )?;
            for (line, budget) in self.buy.iter().zip(budgets) {
                out.orders.push(Order {
                    actor: v.me,
                    class: v.class,
                    node: line.node,
                    good: line.good,
                    qty: v.clock.flow(v.params.get::<FlowPerYear>(line.qty)?),
                    side: Side::Buy { budget },
                });
            }
        }
        // Sell lines, in (node, good) order, each at most what is still held.
        for line in &self.sell {
            let left = dry.get(line.good);
            let want = match line.qty {
                SellQty::Flow(p) => v.clock.flow(v.params.get::<FlowPerYear>(p)?),
                SellQty::AllHeld => left,
            };
            let qty = want.min(left);
            if qty > 0.0 {
                dry.take(line.good, Amount::Qty(qty))?;
            }
            out.orders.push(Order {
                actor: v.me,
                class: v.class,
                node: line.node,
                good: line.good,
                qty,
                side: Side::Sell,
            });
        }
        Ok(out)
    }

    fn produce(&self, v: &View<'_, ScriptState>) -> Result<Vec<StateDelta<Agents>>, AgentError> {
        let mut out = Vec::new();
        let Some(recipe) = self.recipe.as_ref().filter(|_| v.own_state.active) else {
            return Ok(out);
        };
        let mut x = v.clock.flow(v.params.get::<FlowPerYear>(recipe.capacity)?);
        for &(g, a) in &recipe.inputs {
            x = x.min(num::max_scale(v.own.get(g), a)?);
        }
        if x <= 0.0 {
            return Ok(out);
        }
        let me = Holder::Actor(v.me);
        let burn = match v.me {
            ActorId::Desk(_) => Provenance::Production,
            ActorId::Pop(_) => Provenance::Consumption,
        };
        for &(good, a) in &recipe.inputs {
            let q = a * x;
            if q > 0.0 {
                out.push(StateDelta::Burn {
                    from: me,
                    good,
                    amount: Amount::Qty(q),
                    prov: burn,
                });
            }
        }
        let mint = if recipe.inputs.is_empty() {
            Provenance::Endowment
        } else {
            Provenance::Production
        };
        for &(good, o) in &recipe.outputs {
            let q = o * x;
            if q > 0.0 {
                out.push(StateDelta::Mint {
                    to: me,
                    good,
                    qty: q,
                    prov: mint,
                });
            }
        }
        Ok(out)
    }

    fn upkeep(&self, _: &View<'_, ScriptState>) -> Result<Vec<StateDelta<Agents>>, AgentError> {
        // Nothing in Phase 0: a scripted actor holds no capital.
        Ok(Vec::new())
    }
}
