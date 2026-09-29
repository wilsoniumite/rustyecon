//! The specs of the stock roles (P2.2; docs/probe/HORSES-RULES.md; HORSES-SPEC §2): the maker
//! (M2), the capacity desk (M3, wet) and the owner desk (M1). Each is a new variant of the
//! agents' spec (`Maker((..))`, `CapacityDesk((..))`, `OwnerDesk((..))`), added after the
//! many-market kinds, which stay as they are, so every committed tape keeps its canonical form,
//! hash and `world_id`.
//!
//! Every coefficient and dial is a registered param referenced by key and read at use time (R4,
//! E1), so a dated `SetParam` retargets the actor: κ is a `FlowPerYear` (hours a unit of stock a
//! year), δ a `FractionPerYear`, a stock rule's adjustment a `RatePerYear` read as a share, and
//! the maker's cover a `Years` span read as whole ticks. The only inline numbers are genesis
//! state: the owner desk's genesis human share 1 − x, the maker's genesis serving stock after
//! wear, and a step rule's genesis scale. Every field is required and none has a default but
//! the maker's `reserve` (L0.4), whose absence is off.
//!
//! **Lists keep the order written**, as the many-market roles' do: a running recipe's goods and
//! a build's goods are evaluation order (costs are summed in list order), so the canonical form
//! does not sort them.
#![deny(missing_docs)]

use crate::roles::many::spec::{Input, RawInput};
use crate::roles::spec::{
    distinct, live, scale, share, traded, Assign, RawScale, RawSchedule, RawTechnique, Scale,
    Schedule,
};
use rustyecon_core::tape::raw::required;
use rustyecon_core::{ClockMethod, GoodId, Key, LoadError, LoadErrorKind, Resolver, Site};
use serde::{Deserialize, Serialize};

/// A machine's running recipe per hour of its use, as the tape writes it: the goods an hour
/// burns (fodder for a horse-day) and the labour it takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRunning {
    /// The goods bought per hour, in evaluation order, each named once. Required, no default.
    pub goods: Vec<RawInput>,
    /// Param key, unit `Dimensionless`, live: labour per hour. Required, no default.
    pub labour: Key,
}

/// A durable good's build recipe per unit made, as the tape writes it: the goods it buys, labour
/// and land. The maker's own machines' hours are apart (`own_hours`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawBuild {
    /// The goods bought per unit, in evaluation order, each named once. Required, no default.
    pub goods: Vec<RawInput>,
    /// Param key, unit `Dimensionless`, live: labour per unit. Required, no default.
    pub labour: Key,
    /// Param key, unit `Dimensionless`, live: land per unit. Required, no default.
    pub land: Key,
}

/// The maker (a Desk, M2), as the tape writes it: it builds a durable good from the hours of
/// its own serving stock of that good, bought goods, labour and land; keeps part of what it
/// built as its serving stock; and offers the rest under a cover of `cover` ticks of sales.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMaker {
    /// Good key, not a currency: the durable good, made, kept in part and sold. Required, no
    /// default.
    pub output: Key,
    /// Good key, not a currency: labour, bought. Required, no default.
    pub labour: Key,
    /// Good key, not a currency: land, bought. Required, no default.
    pub land: Key,
    /// Param key, unit `Dimensionless`, live: a, hours of its own serving stock per unit built.
    /// Required, no default.
    pub own_hours: Key,
    /// Param key, unit `FlowPerYear`, live: κ, hours a unit of stock gives a year. Required, no
    /// default.
    pub kappa: Key,
    /// The running recipe of its own machines' hours. Required, no default.
    pub running: RawRunning,
    /// The build recipe. Required, no default.
    pub build: RawBuild,
    /// Param key, unit `FractionPerYear`, live: δ, the stock's wear. Required, no default.
    pub delta: Key,
    /// Param key, unit `RatePerYear`, live: s_Km, the share of its serving stock's gap it
    /// closes each tick beyond wear. Required, no default.
    pub adjust: Key,
    /// `Option<Key>`: b_K, the cover of finished stock it holds back, a param of unit `Years`,
    /// live, read as whole ticks of sales (at least one); `None` is no cover, and it offers all
    /// its finished stock. Required (write `None` or `Some(..)`), no default.
    #[serde(deserialize_with = "required")]
    pub cover: Option<Key>,
    /// `Option<Key>`: ψ, the reservation: a param of unit `Dimensionless`, live (so finite and
    /// not negative, as every param is). While its net markup p_K·(1 − δ·a/κ)/c at posted
    /// prices is below ψ, the maker offers none of its finished stock (IDLE-SPEC, L0.4). `None`
    /// or absent is off. The one field of a stock kind that may be absent, so every tape
    /// written before it keeps its canonical text, `tape_hash` and `world_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve: Option<Key>,
    /// `f64`, finite and non-negative: its genesis serving stock after wear, genesis state.
    /// Required, no default.
    pub own: f64,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
}

/// The capacity desk's investment order, as the tape writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OrderRule {
    /// max(δ·K\* + s_K·(K\* − H), 0): replacement of the target's wear and a share of the gap.
    /// The registered rule (HORSES-SPEC §2.7).
    Target,
    /// max(δ·H + s_K·(K\* − H), 0): replacement of the held stock's wear (CHAIN's A3 as written,
    /// the registered negative control).
    Held,
}

/// The capacity desk (a Desk, M3 wet), as the tape writes it: it holds a durable good, never
/// uses its hours itself, buys their running inputs, makes the hours and sells them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCapacityDesk {
    /// Good key, not a currency: the durable good, bought and held. Required, no default.
    pub stock: Key,
    /// Good key, not a currency: its hours, made and sold. Required, no default.
    pub hours: Key,
    /// Good key, not a currency: labour, bought for the running recipe. Required, no default.
    pub labour: Key,
    /// Param key, unit `FlowPerYear`, live: κ. Required, no default.
    pub kappa: Key,
    /// The hours' running recipe. Required, no default.
    pub running: RawRunning,
    /// Param key, unit `FractionPerYear`, live: δ. Required, no default.
    pub delta: Key,
    /// Param key, unit `RatePerYear`, live: s_K, the share of the stock's gap to its target
    /// closed each tick beyond wear. Required, no default.
    pub adjust: Key,
    /// The investment order. Required, no default.
    pub order: OrderRule,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
}

/// The owner desk (a Desk, M1), as the tape writes it: the good desk holding its own machines,
/// which it buys, feeds and wears.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOwnerDesk {
    /// Good key, not a currency: the final good, sold. Required, no default.
    pub output: Key,
    /// Good key, not a currency: labour, bought. Required, no default.
    pub labour: Key,
    /// Good key, not a currency: the durable good, bought and held. Required, no default.
    pub stock: Key,
    /// The task schedule. Required, no default.
    pub schedule: RawSchedule,
    /// The technique rule. Required, no default.
    pub technique: RawTechnique,
    /// How production assigns tasks. Required, no default.
    pub assign: Assign,
    /// Param key, unit `FlowPerYear`, live: κ. Required, no default.
    pub kappa: Key,
    /// The goods an hour of its machines burns, in evaluation order, each named once; goods
    /// only, since ex-post assignment could not split held labour between tasks and machines.
    /// Required, no default.
    pub running: Vec<RawInput>,
    /// Param key, unit `FractionPerYear`, live: δ. Required, no default.
    pub delta: Key,
    /// Param key, unit `RatePerYear`, live: s_K. Required, no default.
    pub adjust: Key,
    /// The scale rule. Required, no default.
    pub scale: RawScale,
}

/// A resolved running recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Running {
    /// The goods, in evaluation order.
    pub goods: Vec<Input>,
    /// Labour per hour, a live `Dimensionless` param.
    pub labour: Site,
}

/// A resolved build recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Build {
    /// The goods, in evaluation order.
    pub goods: Vec<Input>,
    /// Labour per unit, a live `Dimensionless` param.
    pub labour: Site,
    /// Land per unit, a live `Dimensionless` param.
    pub land: Site,
}

/// The resolved maker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Maker {
    /// The durable good.
    pub output: GoodId,
    /// Labour.
    pub labour: GoodId,
    /// Land.
    pub land: GoodId,
    /// a, a live `Dimensionless` param.
    pub own_hours: Site,
    /// κ, a live `FlowPerYear` param read as a `Flow`.
    pub kappa: Site,
    /// The running recipe of its own machines' hours.
    pub running: Running,
    /// The build recipe.
    pub build: Build,
    /// δ, a live `FractionPerYear` param read as a `Fraction`.
    pub delta: Site,
    /// s_Km, a live `RatePerYear` param read as a `Share`.
    pub adjust: Site,
    /// b_K, a live `Years` param read as whole `Ticks`; `None` is no cover.
    pub cover: Option<Site>,
    /// ψ, the reservation, a live `Dimensionless` param; `None` is off. `world_id` hashes the
    /// resolved actors by bincode, so the field is left out when it is `None`, and a world
    /// without it keeps the `world_id` it had before the field existed (L0.5). The resolved
    /// kinds are hashed, never read back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve: Option<Site>,
    /// The genesis serving stock after wear.
    pub own: f64,
    /// The scale rule.
    pub scale: Scale,
}

/// The resolved capacity desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapacityDesk {
    /// The durable good.
    pub stock: GoodId,
    /// Its hours.
    pub hours: GoodId,
    /// Labour.
    pub labour: GoodId,
    /// κ, a live `FlowPerYear` param read as a `Flow`.
    pub kappa: Site,
    /// The hours' running recipe.
    pub running: Running,
    /// δ, a live `FractionPerYear` param read as a `Fraction`.
    pub delta: Site,
    /// s_K, a live `RatePerYear` param read as a `Share`.
    pub adjust: Site,
    /// The investment order.
    pub order: OrderRule,
    /// The scale rule.
    pub scale: Scale,
}

/// The resolved owner desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnerDesk {
    /// The final good.
    pub output: GoodId,
    /// Labour.
    pub labour: GoodId,
    /// The durable good.
    pub stock: GoodId,
    /// The task schedule.
    pub schedule: Schedule,
    /// The technique's adjustment rate, a live `RatePerYear` param read as a `Share`.
    pub technique: Site,
    /// The genesis human share 1 − x.
    pub share: f64,
    /// How production assigns tasks.
    pub assign: Assign,
    /// κ, a live `FlowPerYear` param read as a `Flow`.
    pub kappa: Site,
    /// The goods an hour burns, in evaluation order.
    pub running: Vec<Input>,
    /// δ, a live `FractionPerYear` param read as a `Fraction`.
    pub delta: Site,
    /// s_K, a live `RatePerYear` param read as a `Share`.
    pub adjust: Site,
    /// The scale rule.
    pub scale: Scale,
}

fn site(out: &mut Vec<(String, Site)>, path: String, site: Site) {
    out.push((path, site));
}

fn inputs_sites(
    w: &rustyecon_core::World<crate::ext::Agents>,
    at: &str,
    inputs: &[Input],
    out: &mut Vec<(String, Site)>,
) {
    for i in inputs {
        let key = w
            .key_of(i.good)
            .map_or_else(|| i.good.to_string(), Key::to_string);
        site(out, format!("{at}[{key}].coef"), i.coef);
    }
}

impl Maker {
    /// Its sites, each with its path under the spec; [`crate::Spec::sites`] lists them.
    pub(crate) fn sites(
        &self,
        w: &rustyecon_core::World<crate::ext::Agents>,
        out: &mut Vec<(String, Site)>,
    ) {
        site(out, "own_hours".into(), self.own_hours);
        site(out, "kappa".into(), self.kappa);
        inputs_sites(w, "running.goods", &self.running.goods, out);
        site(out, "running.labour".into(), self.running.labour);
        inputs_sites(w, "build.goods", &self.build.goods, out);
        site(out, "build.labour".into(), self.build.labour);
        site(out, "build.land".into(), self.build.land);
        site(out, "delta".into(), self.delta);
        site(out, "adjust".into(), self.adjust);
        if let Some(c) = self.cover {
            site(out, "cover".into(), c);
        }
        if let Some(p) = self.reserve {
            site(out, "reserve".into(), p);
        }
        self.scale.sites(out);
    }
}

impl CapacityDesk {
    /// Its sites, each with its path under the spec.
    pub(crate) fn sites(
        &self,
        w: &rustyecon_core::World<crate::ext::Agents>,
        out: &mut Vec<(String, Site)>,
    ) {
        site(out, "kappa".into(), self.kappa);
        inputs_sites(w, "running.goods", &self.running.goods, out);
        site(out, "running.labour".into(), self.running.labour);
        site(out, "delta".into(), self.delta);
        site(out, "adjust".into(), self.adjust);
        self.scale.sites(out);
    }
}

impl OwnerDesk {
    /// Its sites, each with its path under the spec.
    pub(crate) fn sites(
        &self,
        w: &rustyecon_core::World<crate::ext::Agents>,
        out: &mut Vec<(String, Site)>,
    ) {
        site(out, "schedule.eta".into(), self.schedule.eta);
        site(out, "schedule.g0".into(), self.schedule.g0);
        site(out, "schedule.g1".into(), self.schedule.g1);
        site(out, "schedule.k".into(), self.schedule.k);
        site(out, "technique.adjust".into(), self.technique);
        site(out, "kappa".into(), self.kappa);
        inputs_sites(w, "running", &self.running, out);
        site(out, "delta".into(), self.delta);
        site(out, "adjust".into(), self.adjust);
        self.scale.sites(out);
    }
}

fn value(r: &mut Resolver<'_>, key: &Key, field: &str) -> Result<Site, LoadError> {
    live(r, key, ClockMethod::Value, field)
}

fn twice(r: &Resolver<'_>, field: &str) -> LoadError {
    r.error(
        field,
        LoadErrorKind::Invalid("a good named twice in one role".into()),
    )
}

/// A list of bought goods, each a traded good named once and none of `taken`.
fn inputs(
    r: &mut Resolver<'_>,
    raw: &[RawInput],
    at: &str,
    taken: &[GoodId],
) -> Result<Vec<Input>, LoadError> {
    let mut out: Vec<Input> = Vec::with_capacity(raw.len());
    for i in raw {
        r.enter(format!("{at}[{}]", i.good));
        let good = traded(r, &i.good, "good")?;
        if taken.contains(&good) || out.iter().any(|o| o.good == good) {
            return Err(twice(r, "good"));
        }
        let coef = value(r, &i.coef, "coef")?;
        r.leave();
        out.push(Input { good, coef });
    }
    Ok(out)
}

fn running(r: &mut Resolver<'_>, raw: &RawRunning, taken: &[GoodId]) -> Result<Running, LoadError> {
    r.enter("running");
    let goods = inputs(r, &raw.goods, "goods", taken)?;
    let labour = value(r, &raw.labour, "labour")?;
    r.leave();
    Ok(Running { goods, labour })
}

/// Resolve a maker's spec. The resolver's path already names `actors[key].spec`.
pub fn resolve_maker(raw: &RawMaker, r: &mut Resolver<'_>) -> Result<Maker, LoadError> {
    let output = traded(r, &raw.output, "output")?;
    let labour = traded(r, &raw.labour, "labour")?;
    let land = traded(r, &raw.land, "land")?;
    distinct(r, &[(output, "output"), (labour, "labour"), (land, "land")])?;
    let taken = [output, labour, land];
    let own_hours = value(r, &raw.own_hours, "own_hours")?;
    let kappa = live(r, &raw.kappa, ClockMethod::Flow, "kappa")?;
    let running = running(r, &raw.running, &taken)?;
    r.enter("build");
    let build = Build {
        goods: inputs(r, &raw.build.goods, "goods", &taken)?,
        labour: value(r, &raw.build.labour, "labour")?,
        land: value(r, &raw.build.land, "land")?,
    };
    r.leave();
    let delta = live(r, &raw.delta, ClockMethod::Fraction, "delta")?;
    let adjust = live(r, &raw.adjust, ClockMethod::Share, "adjust")?;
    let cover = match &raw.cover {
        Some(k) => Some(live(r, k, ClockMethod::Ticks, "cover")?),
        None => None,
    };
    let reserve = match &raw.reserve {
        Some(k) => Some(value(r, k, "reserve")?),
        None => None,
    };
    let own = r.quantity(raw.own, "own")?;
    let scale = scale(r, &raw.scale)?;
    Ok(Maker {
        output,
        labour,
        land,
        own_hours,
        kappa,
        running,
        build,
        delta,
        adjust,
        cover,
        reserve,
        own,
        scale,
    })
}

/// Resolve a capacity desk's spec.
pub fn resolve_capacity_desk(
    raw: &RawCapacityDesk,
    r: &mut Resolver<'_>,
) -> Result<CapacityDesk, LoadError> {
    let stock = traded(r, &raw.stock, "stock")?;
    let hours = traded(r, &raw.hours, "hours")?;
    let labour = traded(r, &raw.labour, "labour")?;
    distinct(r, &[(stock, "stock"), (hours, "hours"), (labour, "labour")])?;
    let kappa = live(r, &raw.kappa, ClockMethod::Flow, "kappa")?;
    let running = running(r, &raw.running, &[stock, hours, labour])?;
    let delta = live(r, &raw.delta, ClockMethod::Fraction, "delta")?;
    let adjust = live(r, &raw.adjust, ClockMethod::Share, "adjust")?;
    let scale = scale(r, &raw.scale)?;
    Ok(CapacityDesk {
        stock,
        hours,
        labour,
        kappa,
        running,
        delta,
        adjust,
        order: raw.order,
        scale,
    })
}

/// Resolve an owner desk's spec.
pub fn resolve_owner_desk(
    raw: &RawOwnerDesk,
    r: &mut Resolver<'_>,
) -> Result<OwnerDesk, LoadError> {
    let output = traded(r, &raw.output, "output")?;
    let labour = traded(r, &raw.labour, "labour")?;
    let stock = traded(r, &raw.stock, "stock")?;
    distinct(
        r,
        &[(output, "output"), (labour, "labour"), (stock, "stock")],
    )?;
    r.enter("schedule");
    let schedule = Schedule {
        eta: value(r, &raw.schedule.eta, "eta")?,
        g0: value(r, &raw.schedule.g0, "g0")?,
        g1: value(r, &raw.schedule.g1, "g1")?,
        k: value(r, &raw.schedule.k, "k")?,
    };
    r.leave();
    r.enter("technique");
    let technique = live(r, &raw.technique.adjust, ClockMethod::Share, "adjust")?;
    let genesis = share(r, raw.technique.share, "share")?;
    r.leave();
    let kappa = live(r, &raw.kappa, ClockMethod::Flow, "kappa")?;
    let running = inputs(r, &raw.running, "running", &[output, labour, stock])?;
    let delta = live(r, &raw.delta, ClockMethod::Fraction, "delta")?;
    let adjust = live(r, &raw.adjust, ClockMethod::Share, "adjust")?;
    let scale = scale(r, &raw.scale)?;
    Ok(OwnerDesk {
        output,
        labour,
        stock,
        schedule,
        technique,
        share: genesis,
        assign: raw.assign,
        kappa,
        running,
        delta,
        adjust,
        scale,
    })
}
