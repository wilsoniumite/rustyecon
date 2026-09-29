//! The specs of the many-market roles (P2.1; docs/probe/MARKETS-RULES.md; MARKETS-SPEC §2): the
//! basket provider, the basket workers, the category desk and the type desk. Each is a new
//! variant of the agents' spec (`BasketProvider((..))`, `BasketWorkers((..))`,
//! `CategoryDesk((..))`, `TypeDesk((..))`), added beside the four Appendix B kinds, which stay
//! as they are, so `tapes/appb.ron` keeps its canonical form, hash and `world_id` (MARKETS-SPEC
//! §2.7).
//!
//! Every coefficient and dial is a registered param referenced by key and read at use time (R4,
//! E1), so a dated `SetParam` retargets the actor. The only inline number is genesis state: the
//! category desk's genesis human share 1 − x, and a step rule's genesis scale. Every field is
//! required and none has a default but the type desk's `plant` (P2.2b), whose absence is off.
//!
//! **Lists keep the order written.** A basket's items, a category's segments and a type's bought
//! services are evaluated in list order: P_s is summed from 0.0 in item order, a desk's budgets
//! are a chain in input order, and its hours and task services are summed over the segments in
//! line order. The order is part of the rule (as the oracle's category order is part of its
//! evaluation, unit-1b.md §5.1), so the canonical form does not sort these lists.
#![deny(missing_docs)]

use crate::ext::Agents;
use crate::roles::plant::spec::{resolve_plant, Plant, RawPlant};
use crate::roles::spec::{
    distinct, live, scale, share, traded, RawScale, RawSchedule, RawTransfer,
};
use crate::roles::spec::{Scale, Schedule};
use rustyecon_core::{
    ActorId, ClockMethod, GoodId, Key, LoadError, LoadErrorKind, Resolver, Site, World,
};
use serde::{Deserialize, Serialize};

/// One item of a basket, as the tape writes it: `weight` units of `good` in each basket.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawItem {
    /// Good key, not a currency: a category's good, or the land a household buys directly as
    /// space. Required, no default.
    pub good: Key,
    /// Param key, unit `Dimensionless`, live: z_j, the units of the good in one basket.
    /// Required, no default.
    pub weight: Key,
}

/// The basket provider (a Pop), as the tape writes it: the Appendix B provider with a basket
/// of many items. It owns the land, sells all its services, funds one basket per head at
/// posted prices and spends a share of the rest on baskets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBasketProvider {
    /// Good key, an `Instant` good: the land services it is endowed with and sells. Required,
    /// no default.
    pub land: Key,
    /// Param key, unit `FlowPerYear`, live: T, the land services per year. Required, no default.
    pub endowment: Key,
    /// The transfer: N·P_s to `to`. Required, no default.
    pub transfer: RawTransfer,
    /// Its basket's items, in evaluation order; at least one, each good once. Required, no
    /// default.
    pub basket: Vec<RawItem>,
    /// Param key, unit `RatePerYear`, live: the share of its coin, after the transfer, spent on
    /// baskets each tick. Required, no default.
    pub spend: Key,
}

/// The basket workers (a Pop), as the tape writes it: the Appendix B workers with a basket of
/// many items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBasketWorkers {
    /// Good key, an `Instant` good: the hours. Required, no default.
    pub labour: Key,
    /// Param key, unit `FlowPerYear`, live: N, the potential hours per year. Required, no
    /// default.
    pub heads: Key,
    /// Param key, unit `Dimensionless`, live: χ_max. Required, no default.
    pub chi_max: Key,
    /// Their basket's items, in evaluation order; at least one, each good once. Required, no
    /// default.
    pub basket: Vec<RawItem>,
    /// Param key, unit `RatePerYear`, live: the share of their coin spent on baskets each tick.
    /// Required, no default.
    pub spend: Key,
}

/// A category's place on the shared task line (decision 60), as the tape writes it: the
/// interior edges e_1 < … < e_{S−1} of the line's segments (0 and 1 are structural), and the
/// category's density of human hours on each segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLine {
    /// Param keys, unit `Dimensionless`, live: the interior edges, in line order. Empty for a
    /// line of one segment. Required, no default.
    pub edges: Vec<Key>,
    /// Param keys, unit `Dimensionless`, live: μ_js, hours by hand per unit of the category per
    /// unit length of line on each segment, one more than `edges`. Required, no default.
    pub density: Vec<Key>,
}

/// The category desk (a Desk), as the tape writes it: it makes one category's good from hours,
/// the task type's machine services and direct land, over its segments of the shared line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCategoryDesk {
    /// Good key, not a currency: the category's good, sold. Required, no default.
    pub output: Key,
    /// Good key, not a currency: hours, bought. Required, no default.
    pub labour: Key,
    /// Good key, not a currency: the task type's machine services, bought. Required, no
    /// default.
    pub service: Key,
    /// Good key, not a currency: land, bought for the category's direct use. Required, no
    /// default.
    pub land: Key,
    /// Param key, unit `Dimensionless`, live: θ_τ, the task type's efficiency, task units per
    /// unit of its service. Required, no default.
    pub theta: Key,
    /// Param key, unit `Dimensionless`, live: b_j, land per unit of the category. Required, no
    /// default.
    pub direct_land: Key,
    /// The task schedule γ, shared by every category (decision 60). Required, no default.
    pub schedule: RawSchedule,
    /// The category's segments of the line. Required, no default.
    pub line: RawLine,
    /// The technique rule: `adjust` and the genesis human share 1 − x. Required, no default.
    pub technique: crate::roles::spec::RawTechnique,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
}

/// One bought service of a type desk's recipe, as the tape writes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawInput {
    /// Good key, not a currency and not the desk's output: another type's service. Required,
    /// no default.
    pub good: Key,
    /// Param key, unit `Dimensionless`, live: a_kl, units of it per unit made. Required, no
    /// default.
    pub coef: Key,
}

/// The type desk's recipe, as the tape writes it: one unit of its service takes `own` of its
/// own service (kept, not bought), each bought service, `labour` hours and `land` space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTypeRecipe {
    /// Param key, unit `Dimensionless`, live: a_kk. Required, no default.
    pub own: Key,
    /// The services of other types it buys, in evaluation order (decision 67: machine
    /// services, labour and land only). Required, no default.
    pub inputs: Vec<RawInput>,
    /// Param key, unit `Dimensionless`, live: λ_k. Required, no default.
    pub labour: Key,
    /// Param key, unit `Dimensionless`, live: b_k. Required, no default.
    pub land: Key,
}

/// The type desk (a Desk), as the tape writes it: the Appendix B machine desk with bought
/// machine services.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTypeDesk {
    /// Good key, not a currency: the type's service, made, kept in part and sold. Required, no
    /// default.
    pub output: Key,
    /// Good key, not a currency: hours, bought. Required, no default.
    pub labour: Key,
    /// Good key, not a currency: space, bought. Required, no default.
    pub land: Key,
    /// The recipe. Required, no default.
    pub recipe: RawTypeRecipe,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
    /// `Option<RawPlant>`: CAPACITY's plant over its recipe's bundle (P2.2b; LOOPS-RULES §3,
    /// §4.1). `None` or absent is off; absent is not written, so every tape written before the
    /// field keeps its canonical text, `tape_hash` and `world_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plant: Option<RawPlant>,
}

/// A resolved basket item.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// The good.
    pub good: GoodId,
    /// z_j, a live `Dimensionless` param.
    pub weight: Site,
}

/// The resolved basket provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasketProvider {
    /// The land services it is endowed with.
    pub land: GoodId,
    /// T, a live `FlowPerYear` param.
    pub endowment: Site,
    /// The transfer's recipient.
    pub transfer_to: ActorId,
    /// N, a live `FlowPerYear` param.
    pub heads: Site,
    /// Its basket, in evaluation order.
    pub basket: Vec<Item>,
    /// Its spending rate, a live `RatePerYear` param read as a `Share`.
    pub spend: Site,
}

/// The resolved basket workers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasketWorkers {
    /// The hours.
    pub labour: GoodId,
    /// N, a live `FlowPerYear` param.
    pub heads: Site,
    /// χ_max, a live `Dimensionless` param.
    pub chi_max: Site,
    /// Their basket, in evaluation order.
    pub basket: Vec<Item>,
    /// Their spending rate, a live `RatePerYear` param read as a `Share`.
    pub spend: Site,
}

/// The resolved category desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryDesk {
    /// The category's good.
    pub output: GoodId,
    /// Hours.
    pub labour: GoodId,
    /// The task type's services.
    pub service: GoodId,
    /// Land.
    pub land: GoodId,
    /// θ_τ, a live `Dimensionless` param.
    pub theta: Site,
    /// b_j, a live `Dimensionless` param.
    pub direct_land: Site,
    /// The task schedule.
    pub schedule: Schedule,
    /// The interior edges, live `Dimensionless` params, in line order.
    pub edges: Vec<Site>,
    /// The densities, live `Dimensionless` params, one per segment.
    pub density: Vec<Site>,
    /// The technique's adjustment rate, a live `RatePerYear` param read as a `Share`.
    pub adjust: Site,
    /// The genesis human share 1 − x.
    pub share: f64,
    /// The scale rule.
    pub scale: Scale,
}

/// A resolved bought service.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Input {
    /// The service.
    pub good: GoodId,
    /// a_kl, a live `Dimensionless` param.
    pub coef: Site,
}

/// The resolved type desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDesk {
    /// The type's service.
    pub output: GoodId,
    /// Hours.
    pub labour: GoodId,
    /// Space.
    pub land: GoodId,
    /// a_kk, a live `Dimensionless` param.
    pub own: Site,
    /// The bought services, in evaluation order.
    pub inputs: Vec<Input>,
    /// λ_k, a live `Dimensionless` param.
    pub labour_coef: Site,
    /// b_k, a live `Dimensionless` param.
    pub land_coef: Site,
    /// The scale rule.
    pub scale: Scale,
    /// Its plant, if any (P2.2b). `world_id` hashes the resolved actors by bincode, so the field
    /// is left out when it is `None`, and a world without it keeps its `world_id` (L0.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plant: Option<Plant>,
}

fn site(out: &mut Vec<(String, Site)>, path: String, site: Site) {
    out.push((path, site));
}

/// A good's key in `w`, or its id when `w` does not name it.
fn key_of(w: &World<Agents>, g: GoodId) -> String {
    w.key_of(g).map_or_else(|| g.to_string(), Key::to_string)
}

fn basket_sites(w: &World<Agents>, basket: &[Item], out: &mut Vec<(String, Site)>) {
    for it in basket {
        site(
            out,
            format!("basket[{}].weight", key_of(w, it.good)),
            it.weight,
        );
    }
}

impl BasketProvider {
    /// Its sites, each with its path under the spec; [`crate::Spec::sites`] lists them.
    pub(crate) fn sites(&self, w: &World<Agents>, out: &mut Vec<(String, Site)>) {
        site(out, "endowment".into(), self.endowment);
        site(out, "transfer.heads".into(), self.heads);
        basket_sites(w, &self.basket, out);
        site(out, "spend".into(), self.spend);
    }
}

impl BasketWorkers {
    /// Their sites, each with its path under the spec.
    pub(crate) fn sites(&self, w: &World<Agents>, out: &mut Vec<(String, Site)>) {
        site(out, "heads".into(), self.heads);
        site(out, "chi_max".into(), self.chi_max);
        basket_sites(w, &self.basket, out);
        site(out, "spend".into(), self.spend);
    }
}

impl CategoryDesk {
    /// Its sites, each with its path under the spec.
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        site(out, "theta".into(), self.theta);
        site(out, "direct_land".into(), self.direct_land);
        site(out, "schedule.eta".into(), self.schedule.eta);
        site(out, "schedule.g0".into(), self.schedule.g0);
        site(out, "schedule.g1".into(), self.schedule.g1);
        site(out, "schedule.k".into(), self.schedule.k);
        for (i, e) in self.edges.iter().enumerate() {
            site(out, format!("line.edges[{i}]"), *e);
        }
        for (i, m) in self.density.iter().enumerate() {
            site(out, format!("line.density[{i}]"), *m);
        }
        site(out, "technique.adjust".into(), self.adjust);
        self.scale.sites(out);
    }
}

impl TypeDesk {
    /// Its sites, each with its path under the spec.
    pub(crate) fn sites(&self, w: &World<Agents>, out: &mut Vec<(String, Site)>) {
        site(out, "recipe.own".into(), self.own);
        for i in &self.inputs {
            site(
                out,
                format!("recipe.inputs[{}].coef", key_of(w, i.good)),
                i.coef,
            );
        }
        site(out, "recipe.labour".into(), self.labour_coef);
        site(out, "recipe.land".into(), self.land_coef);
        self.scale.sites(out);
        if let Some(p) = &self.plant {
            p.sites(out);
        }
    }
}

fn value(r: &mut Resolver<'_>, key: &Key, field: &str) -> Result<Site, LoadError> {
    live(r, key, ClockMethod::Value, field)
}

fn basket(r: &mut Resolver<'_>, raw: &[RawItem]) -> Result<Vec<Item>, LoadError> {
    if raw.is_empty() {
        return Err(r.error(
            "basket",
            LoadErrorKind::Invalid("a basket needs an item".into()),
        ));
    }
    let mut out: Vec<Item> = Vec::with_capacity(raw.len());
    for it in raw {
        r.enter(format!("basket[{}]", it.good));
        let good = traded(r, &it.good, "good")?;
        if out.iter().any(|o| o.good == good) {
            return Err(r.error(
                "good",
                LoadErrorKind::Duplicate {
                    kind: "basket item",
                    key: it.good.to_string(),
                },
            ));
        }
        let weight = value(r, &it.weight, "weight")?;
        r.leave();
        out.push(Item { good, weight });
    }
    Ok(out)
}

/// Resolve a basket provider's spec. The resolver's path already names `actors[key].spec`.
pub fn resolve_basket_provider(
    raw: &RawBasketProvider,
    r: &mut Resolver<'_>,
) -> Result<BasketProvider, LoadError> {
    let land = traded(r, &raw.land, "land")?;
    let endowment = live(r, &raw.endowment, ClockMethod::Flow, "endowment")?;
    r.enter("transfer");
    let transfer_to = r.actor(&raw.transfer.to, "to")?;
    let heads = live(r, &raw.transfer.heads, ClockMethod::Flow, "heads")?;
    r.leave();
    // The land it sells may be an item of its basket: space, bought on the market.
    let basket = basket(r, &raw.basket)?;
    let spend = live(r, &raw.spend, ClockMethod::Share, "spend")?;
    Ok(BasketProvider {
        land,
        endowment,
        transfer_to,
        heads,
        basket,
        spend,
    })
}

/// Resolve the basket workers' spec.
pub fn resolve_basket_workers(
    raw: &RawBasketWorkers,
    r: &mut Resolver<'_>,
) -> Result<BasketWorkers, LoadError> {
    let labour = traded(r, &raw.labour, "labour")?;
    let heads = live(r, &raw.heads, ClockMethod::Flow, "heads")?;
    let chi_max = value(r, &raw.chi_max, "chi_max")?;
    let basket = basket(r, &raw.basket)?;
    if basket.iter().any(|it| it.good == labour) {
        return Err(r.error(
            "labour",
            LoadErrorKind::Invalid("a good named twice in one role".into()),
        ));
    }
    let spend = live(r, &raw.spend, ClockMethod::Share, "spend")?;
    Ok(BasketWorkers {
        labour,
        heads,
        chi_max,
        basket,
        spend,
    })
}

/// Resolve a category desk's spec.
pub fn resolve_category_desk(
    raw: &RawCategoryDesk,
    r: &mut Resolver<'_>,
) -> Result<CategoryDesk, LoadError> {
    let output = traded(r, &raw.output, "output")?;
    let labour = traded(r, &raw.labour, "labour")?;
    let service = traded(r, &raw.service, "service")?;
    let land = traded(r, &raw.land, "land")?;
    distinct(
        r,
        &[
            (output, "output"),
            (labour, "labour"),
            (service, "service"),
            (land, "land"),
        ],
    )?;
    let theta = value(r, &raw.theta, "theta")?;
    let direct_land = value(r, &raw.direct_land, "direct_land")?;
    r.enter("schedule");
    let schedule = Schedule {
        eta: value(r, &raw.schedule.eta, "eta")?,
        g0: value(r, &raw.schedule.g0, "g0")?,
        g1: value(r, &raw.schedule.g1, "g1")?,
        k: value(r, &raw.schedule.k, "k")?,
    };
    r.leave();
    r.enter("line");
    if raw.line.density.len() != raw.line.edges.len() + 1 {
        return Err(r.error(
            "density",
            LoadErrorKind::Invalid(format!(
                "{} interior edges make {} segments, and {} densities are given",
                raw.line.edges.len(),
                raw.line.edges.len() + 1,
                raw.line.density.len()
            )),
        ));
    }
    let mut edges = Vec::with_capacity(raw.line.edges.len());
    for (i, e) in raw.line.edges.iter().enumerate() {
        edges.push(value(r, e, &format!("edges[{i}]"))?);
    }
    let mut density = Vec::with_capacity(raw.line.density.len());
    for (i, m) in raw.line.density.iter().enumerate() {
        density.push(value(r, m, &format!("density[{i}]"))?);
    }
    r.leave();
    r.enter("technique");
    let adjust = live(r, &raw.technique.adjust, ClockMethod::Share, "adjust")?;
    let genesis = share(r, raw.technique.share, "share")?;
    r.leave();
    let scale = scale(r, &raw.scale)?;
    Ok(CategoryDesk {
        output,
        labour,
        service,
        land,
        theta,
        direct_land,
        schedule,
        edges,
        density,
        adjust,
        share: genesis,
        scale,
    })
}

/// Resolve a type desk's spec.
pub fn resolve_type_desk(raw: &RawTypeDesk, r: &mut Resolver<'_>) -> Result<TypeDesk, LoadError> {
    let output = traded(r, &raw.output, "output")?;
    let labour = traded(r, &raw.labour, "labour")?;
    let land = traded(r, &raw.land, "land")?;
    distinct(r, &[(output, "output"), (labour, "labour"), (land, "land")])?;
    r.enter("recipe");
    let own = value(r, &raw.recipe.own, "own")?;
    let mut inputs: Vec<Input> = Vec::with_capacity(raw.recipe.inputs.len());
    for i in &raw.recipe.inputs {
        r.enter(format!("inputs[{}]", i.good));
        let good = traded(r, &i.good, "good")?;
        if good == output || good == labour || good == land || inputs.iter().any(|o| o.good == good)
        {
            return Err(r.error(
                "good",
                LoadErrorKind::Invalid("a good named twice in one role".into()),
            ));
        }
        let coef = value(r, &i.coef, "coef")?;
        r.leave();
        inputs.push(Input { good, coef });
    }
    let labour_coef = value(r, &raw.recipe.labour, "labour")?;
    let land_coef = value(r, &raw.recipe.land, "land")?;
    r.leave();
    let scale = scale(r, &raw.scale)?;
    let plant = resolve_plant(&raw.plant, r)?;
    Ok(TypeDesk {
        output,
        labour,
        land,
        own,
        inputs,
        labour_coef,
        land_coef,
        scale,
        plant,
    })
}
