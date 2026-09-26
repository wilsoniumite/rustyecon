//! The actors' specs: their tape form (schema 1) and their resolved form (docs/ENGINE.md §4
//! and §5).
//!
//! A spec is an enum, so Phase 2's behaviour kinds are new variants and not a new schema shape:
//! `Scripted(..)` since P0.5, and the four Appendix B roles since P2.0 (`Provider`, `Workers`,
//! `GoodDesk`, `MachDesk`; [`crate::roles::spec`]). This module holds the scripted actor's.
//!
//! Every rate, flow and capacity is a registered param referenced by key and read at use time,
//! so a dated `SetParam` retargets the actor (R4, E1); recipe coefficients and weights are
//! dimensionless structural data and sit inline under the actor's `basis`. Every field is
//! required and none has a default, as in core's raw types (`rustyecon_core::tape::raw`).
#![deny(missing_docs)]

use crate::roles::spec::{
    resolve_good_desk, resolve_mach_desk, resolve_provider, resolve_workers, scale_numbers,
    GoodDesk, MachDesk, Provider, RawGoodDesk, RawMachDesk, RawProvider, RawWorkers, Workers,
};
use rustyecon_core::tape::raw::required;
use rustyecon_core::{
    ActorId, GoodId, Key, LoadError, LoadErrorKind, NodeId, ParamId, ParamUse, Resolver, Unit,
};
use serde::{Deserialize, Serialize};

/// An actor's spec as the tape writes it: `spec: Scripted((..))`, `spec: GoodDesk((..))`, ...
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawSpec {
    /// A scripted actor: fixed lines and a Leontief recipe, each quantity a registered param.
    Scripted(RawScript),
    /// The Appendix B provider (a Pop; P2.0).
    Provider(RawProvider),
    /// The Appendix B workers (a Pop; P2.0).
    Workers(RawWorkers),
    /// The Appendix B good desk (a Desk; P2.0).
    GoodDesk(RawGoodDesk),
    /// The Appendix B machine desk (a Desk; P2.0).
    MachDesk(RawMachDesk),
}

/// A scripted actor, as the tape writes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawScript {
    /// `bool`. Whether the actor acts from genesis; a dormant actor (`false`) does nothing until
    /// a dated `SetActive` wakes it. Required, no default.
    pub active: bool,
    /// `Option<RawRecipe>`. The actor's Leontief recipe, run in phase 4. Required (write `None`
    /// or `Some(..)`), no default.
    #[serde(deserialize_with = "required")]
    pub recipe: Option<RawRecipe>,
    /// Buy lines, one per (node, good); any order. Required, no default. Every node named must
    /// quote in the actor's home currency, which is what budgets are drawn from.
    pub buy: Vec<RawBuy>,
    /// Sell lines, one per (node, good); any order. Required, no default.
    pub sell: Vec<RawSell>,
    /// `Option<Key>`, a param of unit `RatePerYear`, live. The continuous rate at which the
    /// actor's cash, after payouts, is budgeted across its buy lines each tick. `Some` exactly
    /// when there are buy lines. Required (write `None` or `Some(..)`), no default.
    #[serde(deserialize_with = "required")]
    pub spend: Option<Key>,
    /// `Option<RawPayout>`. Transfers of the actor's cash to other actors each tick. Required
    /// (write `None` or `Some(..)`), no default.
    #[serde(deserialize_with = "required")]
    pub payout: Option<RawPayout>,
}

/// A Leontief recipe: `x = min(flow(capacity), min_k held_k / a_k)` per tick, burning `a_k·x` of
/// each input and minting `o_l·x` of each output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRecipe {
    /// `(good key, coefficient)` pairs; each coefficient is dimensionless, finite and positive.
    /// A recipe with no inputs is an endowment. Required, no default.
    pub inputs: Vec<(Key, f64)>,
    /// `(good key, coefficient)` pairs; each coefficient is dimensionless, finite and positive.
    /// A recipe with no outputs is consumption. No good may be both an input and an output.
    /// Required, no default.
    pub outputs: Vec<(Key, f64)>,
    /// Param key, unit `FlowPerYear`, live. The most the recipe runs per year. Required, no
    /// default.
    pub capacity: Key,
}

/// A buy line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBuy {
    /// Node key. Required, no default.
    pub node: Key,
    /// Good key, not a currency. Required, no default.
    pub good: Key,
    /// Param key, unit `FlowPerYear`, live. The quantity asked for per year. Required, no
    /// default.
    pub qty: Key,
    /// `f64`, dimensionless, finite and positive. The line's share of the spending total; the
    /// weights of all buy lines, added left to right in (node, good) order, must be exactly 1.
    /// Required, no default.
    pub weight: f64,
}

/// A sell line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSell {
    /// Node key. Required, no default.
    pub node: Key,
    /// Good key, not a currency. Required, no default.
    pub good: Key,
    /// How much is offered. Required, no default.
    pub qty: RawSellQty,
}

/// How much a sell line offers. Lines are posted in (node, good) order, each at most what the
/// actor still holds after the lines before it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawSellQty {
    /// Param key, unit `FlowPerYear`, live: this much per year, or what is left if less.
    Flow(Key),
    /// Everything still held.
    AllHeld,
}

/// A payout: `P = share(rate)·cash` each tick, split by weight across recipients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPayout {
    /// Param key, unit `RatePerYear`, live. Required, no default.
    pub rate: Key,
    /// `(actor key, weight)` pairs, never the payer itself; each weight is dimensionless, finite
    /// and positive, and the weights, added left to right in actor order (desks, then pops,
    /// each by key), must be exactly 1. Required, no default.
    pub to: Vec<(Key, f64)>,
}

/// An actor's spec resolved to ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Spec {
    /// A scripted actor.
    Scripted(Script),
    /// The provider.
    Provider(Provider),
    /// The workers.
    Workers(Workers),
    /// A good desk.
    GoodDesk(GoodDesk),
    /// A machine desk.
    MachDesk(MachDesk),
}

/// A scripted actor, resolved. Lists are in canonical order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    /// Whether it acts from genesis.
    pub active: bool,
    /// Its recipe.
    pub recipe: Option<Recipe>,
    /// Its buy lines, in (node, good) order.
    pub buy: Vec<BuyLine>,
    /// Its sell lines, in (node, good) order.
    pub sell: Vec<SellLine>,
    /// Its spending rate, a live `RatePerYear` param; `Some` exactly when it has buy lines.
    pub spend: Option<ParamId>,
    /// Its payout.
    pub payout: Option<Payout>,
}

/// A resolved recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    /// Inputs and coefficients, by good.
    pub inputs: Vec<(GoodId, f64)>,
    /// Outputs and coefficients, by good.
    pub outputs: Vec<(GoodId, f64)>,
    /// The capacity, a live `FlowPerYear` param.
    pub capacity: ParamId,
}

/// A resolved buy line.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BuyLine {
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// The quantity per year, a live `FlowPerYear` param.
    pub qty: ParamId,
    /// The share of the spending total.
    pub weight: f64,
}

/// A resolved sell line.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SellLine {
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// How much is offered.
    pub qty: SellQty,
}

/// How much a resolved sell line offers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SellQty {
    /// A live `FlowPerYear` param, capped by what is still held.
    Flow(ParamId),
    /// Everything still held.
    AllHeld,
}

/// A resolved payout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payout {
    /// The rate, a live `RatePerYear` param.
    pub rate: ParamId,
    /// Recipients and weights, in `ActorId` order.
    pub to: Vec<(ActorId, f64)>,
}

fn duplicate(r: &Resolver<'_>, field: &str, kind: &'static str, key: String) -> LoadError {
    r.error(field, LoadErrorKind::Duplicate { kind, key })
}

/// A dimensionless coefficient or weight: finite and positive.
fn positive(r: &Resolver<'_>, v: f64, field: &str) -> Result<f64, LoadError> {
    let v = r.quantity(v, field)?;
    if v > 0.0 {
        Ok(v)
    } else {
        Err(r.error(field, LoadErrorKind::BadValue(v)))
    }
}

/// Weights must add, left to right in canonical order, to exactly 1.
fn weights_are_one(r: &Resolver<'_>, weights: &[f64], field: &str) -> Result<(), LoadError> {
    let sum = weights.iter().fold(0.0, |acc, w| acc + w);
    if sum == 1.0 {
        Ok(())
    } else {
        Err(r.error(field, LoadErrorKind::WeightsNotOne { sum }))
    }
}

fn goods(
    r: &mut Resolver<'_>,
    list: &[(Key, f64)],
    side: &str,
) -> Result<Vec<(GoodId, f64)>, LoadError> {
    let mut out: Vec<(GoodId, f64)> = Vec::with_capacity(list.len());
    for (k, a) in list {
        let field = format!("{side}[{k}]");
        let g = r.good(k, &field)?;
        // A recipe that burned or minted money would make a currency-denominated cost, or
        // money with a production provenance (R14): costs are goods.
        if r.is_currency(g) {
            return Err(r.error(&field, LoadErrorKind::CurrencyInRecipe));
        }
        if out.iter().any(|(h, _)| *h == g) {
            return Err(duplicate(r, &field, "recipe good", k.to_string()));
        }
        out.push((g, positive(r, *a, &field)?));
    }
    out.sort_by_key(|(g, _)| *g);
    Ok(out)
}

fn recipe(r: &mut Resolver<'_>, raw: &RawRecipe) -> Result<Recipe, LoadError> {
    r.enter("recipe");
    let inputs = goods(r, &raw.inputs, "inputs")?;
    let outputs = goods(r, &raw.outputs, "outputs")?;
    for (k, _) in &raw.outputs {
        let field = format!("outputs[{k}]");
        let g = r.good(k, &field)?;
        if inputs.iter().any(|(h, _)| *h == g) {
            return Err(r.error(
                &field,
                LoadErrorKind::GoodOnBothSides {
                    good: k.to_string(),
                },
            ));
        }
    }
    if inputs.is_empty() && outputs.is_empty() {
        return Err(r.error(
            "",
            LoadErrorKind::Invalid("a recipe needs an input or an output".into()),
        ));
    }
    let capacity = r.param(&raw.capacity, Unit::FlowPerYear, ParamUse::Live, "capacity")?;
    r.leave();
    Ok(Recipe {
        inputs,
        outputs,
        capacity,
    })
}

/// The (node, good) of an order line, which must not be a currency.
fn market(r: &mut Resolver<'_>, node: &Key, good: &Key) -> Result<(NodeId, GoodId), LoadError> {
    let n = r.node(node, "node")?;
    let g = r.good(good, "good")?;
    if r.is_currency(g) {
        return Err(r.error("good", LoadErrorKind::CurrencyOrder));
    }
    Ok((n, g))
}

fn buys(r: &mut Resolver<'_>, raw: &[RawBuy]) -> Result<Vec<BuyLine>, LoadError> {
    let mut out: Vec<BuyLine> = Vec::with_capacity(raw.len());
    for b in raw {
        r.enter(format!("buy[{}/{}]", b.node, b.good));
        let (node, good) = market(r, &b.node, &b.good)?;
        if out.iter().any(|l| (l.node, l.good) == (node, good)) {
            return Err(duplicate(
                r,
                "",
                "buy line",
                format!("{}/{}", b.node, b.good),
            ));
        }
        let qty = r.param(&b.qty, Unit::FlowPerYear, ParamUse::Live, "qty")?;
        let weight = positive(r, b.weight, "weight")?;
        r.leave();
        out.push(BuyLine {
            node,
            good,
            qty,
            weight,
        });
    }
    out.sort_by_key(|l| (l.node, l.good));
    let weights: Vec<f64> = out.iter().map(|l| l.weight).collect();
    if !out.is_empty() {
        weights_are_one(r, &weights, "buy")?;
    }
    Ok(out)
}

fn sells(r: &mut Resolver<'_>, raw: &[RawSell]) -> Result<Vec<SellLine>, LoadError> {
    let mut out: Vec<SellLine> = Vec::with_capacity(raw.len());
    for s in raw {
        r.enter(format!("sell[{}/{}]", s.node, s.good));
        let (node, good) = market(r, &s.node, &s.good)?;
        if out.iter().any(|l| (l.node, l.good) == (node, good)) {
            return Err(duplicate(
                r,
                "",
                "sell line",
                format!("{}/{}", s.node, s.good),
            ));
        }
        let qty = match &s.qty {
            RawSellQty::Flow(k) => {
                SellQty::Flow(r.param(k, Unit::FlowPerYear, ParamUse::Live, "qty")?)
            }
            RawSellQty::AllHeld => SellQty::AllHeld,
        };
        r.leave();
        out.push(SellLine { node, good, qty });
    }
    out.sort_by_key(|l| (l.node, l.good));
    Ok(out)
}

fn payout(r: &mut Resolver<'_>, raw: &RawPayout) -> Result<Payout, LoadError> {
    r.enter("payout");
    let rate = r.param(&raw.rate, Unit::RatePerYear, ParamUse::Live, "rate")?;
    let mut to: Vec<(ActorId, f64)> = Vec::with_capacity(raw.to.len());
    for (k, w) in &raw.to {
        let field = format!("to[{k}]");
        let a = r.actor(k, &field)?;
        if to.iter().any(|(b, _)| *b == a) {
            return Err(duplicate(r, &field, "payout recipient", k.to_string()));
        }
        to.push((a, positive(r, *w, &field)?));
    }
    if to.is_empty() {
        return Err(r.error(
            "to",
            LoadErrorKind::Invalid("a payout needs a recipient".into()),
        ));
    }
    to.sort_by_key(|(a, _)| *a);
    let weights: Vec<f64> = to.iter().map(|(_, w)| *w).collect();
    weights_are_one(r, &weights, "to")?;
    r.leave();
    Ok(Payout { rate, to })
}

/// Resolve a spec. The resolver's path already names `actors[key].spec`.
pub fn resolve(raw: &RawSpec, r: &mut Resolver<'_>) -> Result<Spec, LoadError> {
    match raw {
        RawSpec::Scripted(s) => {
            let recipe = match &s.recipe {
                Some(rec) => Some(recipe(r, rec)?),
                None => None,
            };
            let buy = buys(r, &s.buy)?;
            let sell = sells(r, &s.sell)?;
            let spend = match (&s.spend, buy.is_empty()) {
                (Some(k), false) => Some(r.param(k, Unit::RatePerYear, ParamUse::Live, "spend")?),
                (None, true) => None,
                (Some(_), true) => {
                    return Err(r.error(
                        "spend",
                        LoadErrorKind::Invalid("a spend rate without buy lines".into()),
                    ))
                }
                (None, false) => {
                    return Err(r.error(
                        "spend",
                        LoadErrorKind::Invalid("buy lines need a spend rate".into()),
                    ))
                }
            };
            let payout = match &s.payout {
                Some(p) => Some(payout(r, p)?),
                None => None,
            };
            Ok(Spec::Scripted(Script {
                active: s.active,
                recipe,
                buy,
                sell,
                spend,
                payout,
            }))
        }
        RawSpec::Provider(p) => resolve_provider(p, r).map(Spec::Provider),
        RawSpec::Workers(p) => resolve_workers(p, r).map(Spec::Workers),
        RawSpec::GoodDesk(d) => resolve_good_desk(d, r).map(Spec::GoodDesk),
        RawSpec::MachDesk(d) => resolve_mach_desk(d, r).map(Spec::MachDesk),
    }
}

/// Put a raw spec's own lists in canonical order, for `Tape::to_ron`: recipe goods by key,
/// lines by (node, good) key, payout recipients by key. The Appendix B roles hold no lists.
pub fn canonical(raw: &mut RawSpec) {
    match raw {
        RawSpec::Provider(_)
        | RawSpec::Workers(_)
        | RawSpec::GoodDesk(_)
        | RawSpec::MachDesk(_) => {}
        RawSpec::Scripted(s) => {
            if let Some(rec) = &mut s.recipe {
                rec.inputs.sort_by(|a, b| a.0.cmp(&b.0));
                rec.outputs.sort_by(|a, b| a.0.cmp(&b.0));
            }
            s.buy
                .sort_by(|a, b| (&a.node, &a.good).cmp(&(&b.node, &b.good)));
            s.sell
                .sort_by(|a, b| (&a.node, &a.good).cmp(&(&b.node, &b.good)));
            if let Some(p) = &mut s.payout {
                p.to.sort_by(|a, b| a.0.cmp(&b.0));
            }
        }
    }
}

/// The spec's inline numbers, each with its tape path under the actor, for the registry listing
/// (`rustyecon registry`): recipe coefficients and line and payout weights; for the Appendix B
/// desks, their genesis state (the good desk's human share, a step rule's scale).
pub fn inline_numbers(raw: &RawSpec) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    match raw {
        RawSpec::Provider(_) | RawSpec::Workers(_) => {}
        RawSpec::GoodDesk(d) => {
            out.push(("technique.share".to_string(), d.technique.share));
            scale_numbers(&d.scale, &mut out);
        }
        RawSpec::MachDesk(d) => scale_numbers(&d.scale, &mut out),
        RawSpec::Scripted(s) => {
            if let Some(rec) = &s.recipe {
                for (k, a) in &rec.inputs {
                    out.push((format!("recipe.inputs[{k}]"), *a));
                }
                for (k, o) in &rec.outputs {
                    out.push((format!("recipe.outputs[{k}]"), *o));
                }
            }
            for b in &s.buy {
                out.push((format!("buy[{}/{}].weight", b.node, b.good), b.weight));
            }
            if let Some(p) = &s.payout {
                for (k, w) in &p.to {
                    out.push((format!("payout.to[{k}]"), *w));
                }
            }
        }
    }
    out
}
