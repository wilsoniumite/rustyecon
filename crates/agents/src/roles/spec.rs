//! The specs of the four Appendix B roles (docs/probe/RULES.md; PROBE-SPEC §1.3): the provider,
//! the workers, the good desk and the machine desk. Each is a variant of the agents' spec
//! (`Provider((..))`, `Workers((..))`, `GoodDesk((..))`, `MachDesk((..))`), added at P2.0.
//!
//! Every rate, flow, coefficient and dial is a registered param referenced by key and read at
//! use time (R4, E1), so a dated `SetParam` retargets the actor: the instance's coefficients
//! (`inst.*`) as much as the design's dials. The only inline numbers are genesis state, which is
//! dimensionless structural data under the actor's `basis`: the good desk's genesis human share
//! 1 − x, and a step rule's genesis scale. Every field is required and none has a default, as in
//! core's raw types (`rustyecon_core::tape::raw`).
#![deny(missing_docs)]

use rustyecon_core::tape::raw::required;
use rustyecon_core::{
    ActorId, ClockMethod, GoodId, Key, LoadError, LoadErrorKind, ParamUse, Resolver, Site,
};
use serde::{Deserialize, Serialize};

/// A household's basket, as the tape writes it: one unit of `good` and `per_basket` units of
/// `space`, bought in that ratio and eaten together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBasket {
    /// Good key, not a currency: the final good. Required, no default.
    pub good: Key,
    /// Good key, not a currency and not `good`: the space (land services). Required, no
    /// default.
    pub space: Key,
    /// Param key, unit `Dimensionless`, live: h, the space in one basket. Required, no default.
    pub per_basket: Key,
}

/// The provider's transfer, as the tape writes it: N·P_s of the home currency each tick, P_s =
/// p + h·r at posted prices, to the actor that holds the workers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTransfer {
    /// Actor key, not the provider itself. Required, no default.
    pub to: Key,
    /// Param key, unit `FlowPerYear`, live: N, the heads the transfer feeds, per year. Required,
    /// no default.
    pub heads: Key,
}

/// The provider (a Pop): it owns the land, sells its services, funds one basket per head and
/// eats the rest of what it earns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProvider {
    /// Good key, an `Instant` good: the land services it is endowed with and sells. Required,
    /// no default.
    pub land: Key,
    /// Param key, unit `FlowPerYear`, live: T, the land services per year. Required, no default.
    pub endowment: Key,
    /// The transfer. Required, no default.
    pub transfer: RawTransfer,
    /// Its basket. Required, no default.
    pub basket: RawBasket,
    /// Param key, unit `RatePerYear`, live: the share of its coin, after the transfer, that it
    /// spends on baskets each tick. Required, no default.
    pub spend: Key,
}

/// The workers (a Pop): N potential hours, each worked when its work cost χ ~ U[0, χ_max] is at
/// most ln(1 + w/P_s).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawWorkers {
    /// Good key, an `Instant` good: the hours. Required, no default.
    pub labour: Key,
    /// Param key, unit `FlowPerYear`, live: N, the potential hours per year. Required, no
    /// default.
    pub heads: Key,
    /// Param key, unit `Dimensionless`, live: χ_max, the top of the work cost's support.
    /// Required, no default.
    pub chi_max: Key,
    /// Their basket. Required, no default.
    pub basket: RawBasket,
    /// Param key, unit `RatePerYear`, live: the share of their coin spent on baskets each tick.
    /// Required, no default.
    pub spend: Key,
}

/// The task schedule γ(i) = η(g0 + g1·i^k), as the tape writes it: each key a `Dimensionless`
/// param, live.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSchedule {
    /// Param key: η. Required, no default.
    pub eta: Key,
    /// Param key: g0. Required, no default.
    pub g0: Key,
    /// Param key: g1. Required, no default.
    pub g1: Key,
    /// Param key: k, positive. Required, no default.
    pub k: Key,
}

/// The technique rule, as the tape writes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTechnique {
    /// Param key, unit `RatePerYear`, live: the share of the gap to the target human share
    /// closed each tick (`Clock::share`). Required, no default.
    pub adjust: Key,
    /// `f64`, dimensionless, in [0, 1]: the genesis human share 1 − x, genesis state. Required,
    /// no default.
    pub share: f64,
}

/// How the good desk assigns tasks in production.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Assign {
    /// Leontief at the planned technique: y = min(L/(1 − x), M/J(x)); an input with a zero
    /// coefficient does not bind. The registered default.
    Planned,
    /// The cutoff that uses up both inputs held, L·J(x_u) = M·(1 − x_u), then Leontief at x_u
    /// (design-analytic-first's ex-post assignment; a registered alternative).
    ExPost,
}

/// A payout of coin above a ceiling, as the tape writes it (a registered variant of the cash
/// rule). Each tick the desk pays `share(rate)·(coin − e^ceiling·target)` to `to` when its coin
/// is above `e^ceiling·target`, where `target` is the value at posted prices of the output it
/// holds to sell, net of its own input, over `share(turnover)`: its stationary coin at rest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCeiling {
    /// Actor key, not the payer: the owner of the desk's profits. Required, no default.
    pub to: Key,
    /// Param key, unit `RatePerYear`, live: the share of the excess paid each tick. Required, no
    /// default.
    pub rate: Key,
    /// Param key, unit `Dimensionless`, live: the ceiling as a log multiple of the target.
    /// Required, no default.
    pub ceiling: Key,
}

/// The cash scale rule, as the tape writes it: each tick the desk spends
/// `share(turnover·μ^tilt)` of its coin on inputs at posted prices, μ its markup at posted
/// prices. Its coin is its scale, and it is constant only at zero margin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCash {
    /// Param key, unit `RatePerYear`, live: the turnover v of the desk's coin. Required, no
    /// default.
    pub turnover: Key,
    /// Param key, unit `Dimensionless`, live: κ, the outlay's tilt toward its markup (0: none).
    /// Required, no default.
    pub tilt: Key,
    /// `Option<RawCeiling>`: a payout above a ceiling. Required (write `None` or `Some(..)`), no
    /// default.
    #[serde(deserialize_with = "required")]
    pub payout: Option<RawCeiling>,
}

/// A payout, as the tape writes it: `share(rate)` of the coin above the buffer, to `to`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPayoutTo {
    /// Actor key, not the payer. Required, no default.
    pub to: Key,
    /// Param key, unit `RatePerYear`, live. Required, no default.
    pub rate: Key,
}

/// The margin-step scale rule, as the tape writes it (July's rule, the registered negative
/// control): the scale q steps by `exp(step·(m ∓ dead))` while the log margin m at posted
/// prices is outside ±dead; the outlay c·q is capped by the coin less the payout, which pays
/// out the coin above `c·q/share(buffer)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStep {
    /// Param key, unit `RatePerYear`, live: the log step per unit log margin above the band
    /// (`Clock::log_step`). Required, no default.
    pub up: Key,
    /// Param key, unit `RatePerYear`, live: the log step per unit log margin below the band.
    /// Required, no default.
    pub down: Key,
    /// Param key, unit `Dimensionless`, live: the band's half-width on the log margin.
    /// Required, no default.
    pub dead: Key,
    /// Param key, unit `RatePerYear`, live: the buffer's turnover; the coin target is the
    /// outlay over its share. Required, no default.
    pub buffer: Key,
    /// The payout of the coin above the target. Required, no default.
    pub payout: RawPayoutTo,
    /// `f64`, finite and non-negative: the genesis scale, output per tick, genesis state.
    /// Required, no default.
    pub scale: f64,
}

/// A desk's scale rule, as the tape writes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawScale {
    /// The cash rule (the registered default).
    Cash(RawCash),
    /// The margin-step rule (the registered negative control).
    Step(RawStep),
}

/// The good desk (a Desk): it makes the final good from hours and machine services over the
/// task line, machines on [0, x) and people on (x, 1].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGoodDesk {
    /// Good key, not a currency: the final good, sold. Required, no default.
    pub output: Key,
    /// Good key, not a currency: hours, bought. Required, no default.
    pub labour: Key,
    /// Good key, not a currency: machine services, bought. Required, no default.
    pub mach: Key,
    /// The task schedule. Required, no default.
    pub schedule: RawSchedule,
    /// The technique rule. Required, no default.
    pub technique: RawTechnique,
    /// How production assigns tasks. Required, no default.
    pub assign: Assign,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
}

/// The machine desk's recipe, as the tape writes it: one machine service takes `own` machine
/// services, `labour` hours and `land` space, each key a `Dimensionless` param, live.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMachRecipe {
    /// Param key: a, its own output as input, kept and not bought. Required, no default.
    pub own: Key,
    /// Param key: λ, hours. Required, no default.
    pub labour: Key,
    /// Param key: b, space. Required, no default.
    pub land: Key,
}

/// The machine desk (a Desk): it makes machine services from its own output, hours and space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMachDesk {
    /// Good key, not a currency: machine services, made, kept in part and sold. Required, no
    /// default.
    pub output: Key,
    /// Good key, not a currency: hours, bought. Required, no default.
    pub labour: Key,
    /// Good key, not a currency: space, bought. Required, no default.
    pub land: Key,
    /// The recipe. Required, no default.
    pub recipe: RawMachRecipe,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
}

/// A resolved basket.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Basket {
    /// The final good.
    pub good: GoodId,
    /// The space.
    pub space: GoodId,
    /// h, a live `Dimensionless` param.
    pub per_basket: Site,
}

/// The resolved provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    /// The land services it is endowed with.
    pub land: GoodId,
    /// T, a live `FlowPerYear` param.
    pub endowment: Site,
    /// The transfer's recipient.
    pub transfer_to: ActorId,
    /// N, a live `FlowPerYear` param.
    pub heads: Site,
    /// Its basket.
    pub basket: Basket,
    /// Its spending rate, a live `RatePerYear` param read as a `Share`.
    pub spend: Site,
}

/// The resolved workers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workers {
    /// The hours.
    pub labour: GoodId,
    /// N, a live `FlowPerYear` param.
    pub heads: Site,
    /// χ_max, a live `Dimensionless` param.
    pub chi_max: Site,
    /// Their basket.
    pub basket: Basket,
    /// Their spending rate, a live `RatePerYear` param read as a `Share`.
    pub spend: Site,
}

/// The resolved schedule: live `Dimensionless` params.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    /// η.
    pub eta: Site,
    /// g0.
    pub g0: Site,
    /// g1.
    pub g1: Site,
    /// k.
    pub k: Site,
}

/// A resolved ceiling payout.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ceiling {
    /// The recipient.
    pub to: ActorId,
    /// The rate, a live `RatePerYear` param read as a `Share`.
    pub rate: Site,
    /// The log ceiling, a live `Dimensionless` param.
    pub ceiling: Site,
}

/// A resolved scale rule.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Scale {
    /// The cash rule.
    Cash {
        /// The turnover, a live `RatePerYear` param read as a `Share`.
        turnover: Site,
        /// The tilt, a live `Dimensionless` param.
        tilt: Site,
        /// The payout above a ceiling, if any.
        payout: Option<Ceiling>,
    },
    /// The margin-step rule.
    Step {
        /// The step above the band, a live `RatePerYear` param read as a `LogStep`.
        up: Site,
        /// The step below the band, a live `RatePerYear` param read as a `LogStep`.
        down: Site,
        /// The band, a live `Dimensionless` param.
        dead: Site,
        /// The buffer's turnover, a live `RatePerYear` param read as a `Share`.
        buffer: Site,
        /// The payout's recipient.
        to: ActorId,
        /// The payout's rate, a live `RatePerYear` param read as a `Share`.
        rate: Site,
        /// The genesis scale.
        scale: f64,
    },
}

impl Scale {
    /// The genesis scale state: a step rule's registered genesis scale, and 0 for the cash
    /// rule, whose scale is its coin.
    pub fn genesis(&self) -> f64 {
        match self {
            Scale::Cash { .. } => 0.0,
            Scale::Step { scale, .. } => *scale,
        }
    }

    /// The actor the rule pays, if it pays anyone.
    pub fn pays(&self) -> Option<ActorId> {
        match self {
            Scale::Cash { payout, .. } => payout.map(|c| c.to),
            Scale::Step { to, .. } => Some(*to),
        }
    }

    /// The rule's sites, each with its path under the spec (`scale.…`).
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        let mut at = |field: &str, site: Site| out.push((format!("scale.{field}"), site));
        match *self {
            Scale::Cash {
                turnover,
                tilt,
                payout,
            } => {
                at("turnover", turnover);
                at("tilt", tilt);
                if let Some(c) = payout {
                    at("payout.rate", c.rate);
                    at("payout.ceiling", c.ceiling);
                }
            }
            Scale::Step {
                up,
                down,
                dead,
                buffer,
                rate,
                ..
            } => {
                at("up", up);
                at("down", down);
                at("dead", dead);
                at("buffer", buffer);
                at("payout.rate", rate);
            }
        }
    }
}

fn site(out: &mut Vec<(String, Site)>, path: &str, site: Site) {
    out.push((path.to_string(), site));
}

impl Provider {
    /// Its sites, each with its path under the spec; [`crate::Spec::sites`] lists them.
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        site(out, "endowment", self.endowment);
        site(out, "transfer.heads", self.heads);
        site(out, "basket.per_basket", self.basket.per_basket);
        site(out, "spend", self.spend);
    }
}

impl Workers {
    /// Their sites, each with its path under the spec.
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        site(out, "heads", self.heads);
        site(out, "chi_max", self.chi_max);
        site(out, "basket.per_basket", self.basket.per_basket);
        site(out, "spend", self.spend);
    }
}

impl GoodDesk {
    /// Its sites, each with its path under the spec.
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        site(out, "schedule.eta", self.schedule.eta);
        site(out, "schedule.g0", self.schedule.g0);
        site(out, "schedule.g1", self.schedule.g1);
        site(out, "schedule.k", self.schedule.k);
        site(out, "technique.adjust", self.adjust);
        self.scale.sites(out);
    }
}

impl MachDesk {
    /// Its sites, each with its path under the spec.
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        site(out, "recipe.own", self.recipe.own);
        site(out, "recipe.labour", self.recipe.labour);
        site(out, "recipe.land", self.recipe.land);
        self.scale.sites(out);
    }
}

/// The resolved good desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoodDesk {
    /// The final good.
    pub output: GoodId,
    /// Hours.
    pub labour: GoodId,
    /// Machine services.
    pub mach: GoodId,
    /// The task schedule.
    pub schedule: Schedule,
    /// The technique's adjustment rate, a live `RatePerYear` param read as a `Share`.
    pub adjust: Site,
    /// The genesis human share 1 − x.
    pub share: f64,
    /// How production assigns tasks.
    pub assign: Assign,
    /// The scale rule.
    pub scale: Scale,
}

/// The resolved machine desk's recipe: live `Dimensionless` params.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MachRecipe {
    /// a.
    pub own: Site,
    /// λ.
    pub labour: Site,
    /// b.
    pub land: Site,
}

/// The resolved machine desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachDesk {
    /// Machine services.
    pub output: GoodId,
    /// Hours.
    pub labour: GoodId,
    /// Space.
    pub land: GoodId,
    /// The recipe.
    pub recipe: MachRecipe,
    /// The scale rule.
    pub scale: Scale,
}

pub(crate) fn live(
    r: &mut Resolver<'_>,
    key: &Key,
    method: ClockMethod,
    field: &str,
) -> Result<Site, LoadError> {
    r.param(key, method, ParamUse::Live, field)
}

/// A traded good: not a currency, since a currency has no market, and not an untraded good (a
/// plant, P2.2b.1), which has none either.
pub(crate) fn traded(r: &mut Resolver<'_>, key: &Key, field: &str) -> Result<GoodId, LoadError> {
    let g = r.good(key, field)?;
    if r.is_currency(g) {
        return Err(r.error(field, LoadErrorKind::CurrencyOrder));
    }
    if r.is_untraded(g) {
        return Err(r.error(field, LoadErrorKind::NoMarket));
    }
    Ok(g)
}

/// Goods a role names must be distinct: one good in two roles would net its orders.
pub(crate) fn distinct(r: &Resolver<'_>, goods: &[(GoodId, &str)]) -> Result<(), LoadError> {
    for (i, (g, field)) in goods.iter().enumerate() {
        if goods[..i].iter().any(|(h, _)| h == g) {
            return Err(r.error(
                field,
                LoadErrorKind::Invalid("a good named twice in one role".into()),
            ));
        }
    }
    Ok(())
}

fn basket(r: &mut Resolver<'_>, raw: &RawBasket) -> Result<Basket, LoadError> {
    r.enter("basket");
    let good = traded(r, &raw.good, "good")?;
    let space = traded(r, &raw.space, "space")?;
    distinct(r, &[(good, "good"), (space, "space")])?;
    let per_basket = live(r, &raw.per_basket, ClockMethod::Value, "per_basket")?;
    r.leave();
    Ok(Basket {
        good,
        space,
        per_basket,
    })
}

/// A genesis share: finite, sign bit clear, at most 1.
pub(crate) fn share(r: &Resolver<'_>, v: f64, field: &str) -> Result<f64, LoadError> {
    let v = r.quantity(v, field)?;
    if v <= 1.0 {
        Ok(v)
    } else {
        Err(r.error(field, LoadErrorKind::BadValue(v)))
    }
}

pub(crate) fn scale(r: &mut Resolver<'_>, raw: &RawScale) -> Result<Scale, LoadError> {
    r.enter("scale");
    let out = match raw {
        RawScale::Cash(c) => {
            let turnover = live(r, &c.turnover, ClockMethod::Share, "turnover")?;
            let tilt = live(r, &c.tilt, ClockMethod::Value, "tilt")?;
            let payout = match &c.payout {
                Some(p) => {
                    r.enter("payout");
                    let ceiling = Ceiling {
                        to: r.actor(&p.to, "to")?,
                        rate: live(r, &p.rate, ClockMethod::Share, "rate")?,
                        ceiling: live(r, &p.ceiling, ClockMethod::Value, "ceiling")?,
                    };
                    r.leave();
                    Some(ceiling)
                }
                None => None,
            };
            Scale::Cash {
                turnover,
                tilt,
                payout,
            }
        }
        RawScale::Step(s) => {
            let up = live(r, &s.up, ClockMethod::LogStep, "up")?;
            let down = live(r, &s.down, ClockMethod::LogStep, "down")?;
            let dead = live(r, &s.dead, ClockMethod::Value, "dead")?;
            let buffer = live(r, &s.buffer, ClockMethod::Share, "buffer")?;
            r.enter("payout");
            let to = r.actor(&s.payout.to, "to")?;
            let rate = live(r, &s.payout.rate, ClockMethod::Share, "rate")?;
            r.leave();
            let scale = r.quantity(s.scale, "scale")?;
            Scale::Step {
                up,
                down,
                dead,
                buffer,
                to,
                rate,
                scale,
            }
        }
    };
    r.leave();
    Ok(out)
}

/// Resolve a provider's spec. The resolver's path already names `actors[key].spec`.
pub fn resolve_provider(raw: &RawProvider, r: &mut Resolver<'_>) -> Result<Provider, LoadError> {
    let land = traded(r, &raw.land, "land")?;
    let endowment = live(r, &raw.endowment, ClockMethod::Flow, "endowment")?;
    r.enter("transfer");
    let transfer_to = r.actor(&raw.transfer.to, "to")?;
    let heads = live(r, &raw.transfer.heads, ClockMethod::Flow, "heads")?;
    r.leave();
    let basket = basket(r, &raw.basket)?;
    distinct(r, &[(land, "land"), (basket.good, "basket.good")])?;
    let spend = live(r, &raw.spend, ClockMethod::Share, "spend")?;
    Ok(Provider {
        land,
        endowment,
        transfer_to,
        heads,
        basket,
        spend,
    })
}

/// Resolve the workers' spec.
pub fn resolve_workers(raw: &RawWorkers, r: &mut Resolver<'_>) -> Result<Workers, LoadError> {
    let labour = traded(r, &raw.labour, "labour")?;
    let heads = live(r, &raw.heads, ClockMethod::Flow, "heads")?;
    let chi_max = live(r, &raw.chi_max, ClockMethod::Value, "chi_max")?;
    let basket = basket(r, &raw.basket)?;
    distinct(
        r,
        &[
            (labour, "labour"),
            (basket.good, "basket.good"),
            (basket.space, "basket.space"),
        ],
    )?;
    let spend = live(r, &raw.spend, ClockMethod::Share, "spend")?;
    Ok(Workers {
        labour,
        heads,
        chi_max,
        basket,
        spend,
    })
}

/// Resolve a good desk's spec.
pub fn resolve_good_desk(raw: &RawGoodDesk, r: &mut Resolver<'_>) -> Result<GoodDesk, LoadError> {
    let output = traded(r, &raw.output, "output")?;
    let labour = traded(r, &raw.labour, "labour")?;
    let mach = traded(r, &raw.mach, "mach")?;
    distinct(r, &[(output, "output"), (labour, "labour"), (mach, "mach")])?;
    r.enter("schedule");
    let schedule = Schedule {
        eta: live(r, &raw.schedule.eta, ClockMethod::Value, "eta")?,
        g0: live(r, &raw.schedule.g0, ClockMethod::Value, "g0")?,
        g1: live(r, &raw.schedule.g1, ClockMethod::Value, "g1")?,
        k: live(r, &raw.schedule.k, ClockMethod::Value, "k")?,
    };
    r.leave();
    r.enter("technique");
    let adjust = live(r, &raw.technique.adjust, ClockMethod::Share, "adjust")?;
    let genesis = share(r, raw.technique.share, "share")?;
    r.leave();
    let scale = scale(r, &raw.scale)?;
    Ok(GoodDesk {
        output,
        labour,
        mach,
        schedule,
        adjust,
        share: genesis,
        assign: raw.assign,
        scale,
    })
}

/// Resolve a machine desk's spec.
pub fn resolve_mach_desk(raw: &RawMachDesk, r: &mut Resolver<'_>) -> Result<MachDesk, LoadError> {
    let output = traded(r, &raw.output, "output")?;
    let labour = traded(r, &raw.labour, "labour")?;
    let land = traded(r, &raw.land, "land")?;
    distinct(r, &[(output, "output"), (labour, "labour"), (land, "land")])?;
    r.enter("recipe");
    let recipe = MachRecipe {
        own: live(r, &raw.recipe.own, ClockMethod::Value, "own")?,
        labour: live(r, &raw.recipe.labour, ClockMethod::Value, "labour")?,
        land: live(r, &raw.recipe.land, ClockMethod::Value, "land")?,
    };
    r.leave();
    let scale = scale(r, &raw.scale)?;
    Ok(MachDesk {
        output,
        labour,
        land,
        recipe,
        scale,
    })
}

/// A scale rule's inline numbers, for the registry listing.
pub fn scale_numbers(raw: &RawScale, out: &mut Vec<(String, f64)>) {
    if let RawScale::Step(s) = raw {
        out.push(("scale.scale".to_string(), s.scale));
    }
}
