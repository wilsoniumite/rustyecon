//! The markets probe's tapes and their generator (MARKETS-SPEC §2, §5; docs/probe/MARKETS-RULES.md).
//!
//! A [`Setup`] names what a run is made from: the instance, the tick length, the dials, the
//! one-sided rule, the displacement of genesis, any coefficients changed from tick 0, and any
//! dated shocks. [`tape_ron`] writes it as a tape of the four many-market kinds. The oracle's
//! equilibrium (unit 1c, solved here, outside any `Sim`) seeds genesis: its prices relative to
//! r = 1 coin, 1 − x\* for every category desk, one tick's output held by each desk, and each
//! actor's stationary coin under the dials (MARKETS-SPEC §5.4). No agent reads the oracle
//! (R13). `tapes/markets-<id>.ron` is `tape_ron(&Setup::registered(id, 52))`, checked by a test.

use super::instance::{Instance, Point, COMMONS_BASIS, FREE_BASIS, SWITCH_BASIS, WALL_BASIS};
use crate::setup::{clock, OneSided, START};
use rustyecon_core::{num, FlowPerYear, RatePerYear};
use rustyecon_engine::rustyecon_agents::{pop_market, PopMarket};

/// One dial: a tape param with its unit and basis (R4).
#[derive(Debug, Clone, PartialEq)]
pub struct Dial {
    /// The param's key.
    pub key: String,
    /// Its value, per year where its unit is per year.
    pub value: f64,
    /// Its unit, as the tape writes it.
    pub unit: &'static str,
    /// Its basis, as the tape writes it.
    pub basis: String,
}

/// The dials of a run, in the tape's order, and the name of the registered set they start from.
#[derive(Debug, Clone, PartialEq)]
pub struct Dials {
    /// `c2m` or `c2l`.
    pub set: String,
    /// The dials.
    pub values: Vec<Dial>,
}

const C2M: &str = "Assumed(\"MARKETS-SPEC C2m, design-analytic-first C2 per role\")";
/// The pace's dial's basis (the trap scan's §5.4; decision 406).
const PACE_DIAL: &str =
    "Assumed(\"the trap scan's §4: participation at land's and the types' rate\")";
/// The pace's dial: participation's rate, a year (P2.4; decision 406).
pub const PACE_KEY: &str = "adjust.participation.workers";
/// The switch's dials' basis (the switch scan's §3.7; decision 410).
const SWITCH_DIAL: &str = "Assumed(\"scan-switch SPEC §6.3\")";
/// The switch's rate a year (the switch scan's §6.3; decision 410).
pub const SWITCH_RATE: f64 = 26.0;
/// The free step's dial's basis (the free scan's §8, R14; decision 417).
const FREE_DIAL: &str = "Assumed(\"FREE-SPEC: the scan's window 0.3–1 at C2m, c 0.5\")";
/// The commons market's rate's basis (the free scan's mirror, `fm.dials`: land's rate).
const COMMONS_RATE: &str =
    "Assumed(\"FREE-SPEC §6.4: the pops' commons market at land's rate (fm.dials)\")";
const C2L: &str = "Assumed(\"MARKETS-SPEC C2L: type rates at labour's, markup tilt 1 (§5.2)\")";

impl Dials {
    /// C2m (MARKETS-SPEC §5.1): design-analytic-first's C2 carried one for one to every market
    /// and desk of its role. Labour 5.2 a year, each category's good 2.6, land and each type
    /// 1.3; each technique 2.6; each desk's turnover 5.2 and tilt 0; household spending 13.
    pub fn c2m(inst: &Instance) -> Dials {
        let mut v = Vec::new();
        let mut rate = |key: String, value: f64| {
            v.push(Dial {
                key,
                value,
                unit: "RatePerYear",
                basis: C2M.into(),
            })
        };
        rate("rate.labour".into(), 5.2);
        // Each reserved type's labour market at labour's rate (decision 364: C2 per role).
        for t in &inst.wtypes {
            rate(format!("rate.{}", t.market()), 5.2);
        }
        // Each switch pop's rate at a switch instance only (P2.4; the switch scan's §3.7; decision
        // 410): 26 a year, a `rate.*` dial, so `rate.*` scales it with the markets' rates.
        if inst.switch {
            for t in &inst.wtypes {
                v.push(Dial {
                    key: format!("rate.switch.{}", t.key),
                    value: SWITCH_RATE,
                    unit: "RatePerYear",
                    basis: SWITCH_DIAL.into(),
                });
            }
        }
        let mut rate = |key: String, value: f64| {
            v.push(Dial {
                key,
                value,
                unit: "RatePerYear",
                basis: C2M.into(),
            })
        };
        rate("rate.land".into(), 1.3);
        // The commons' market at land's rate where pops trade it (P2.4; the free scan's mirror,
        // `fm.dials`), a `rate.*` dial.
        if !inst.commoners.is_empty() {
            v.push(Dial {
                key: "rate.commons".into(),
                value: 1.3,
                unit: "RatePerYear",
                basis: COMMONS_RATE.into(),
            });
        }
        let mut rate = |key: String, value: f64| {
            v.push(Dial {
                key,
                value,
                unit: "RatePerYear",
                basis: C2M.into(),
            })
        };
        for t in &inst.types {
            rate(format!("rate.{}", t.key), 1.3);
        }
        for c in &inst.categories {
            rate(format!("rate.{}", c.key), 2.6);
        }
        for c in &inst.categories {
            rate(format!("adjust.technique.{}", c.key), 2.6);
        }
        // Participation's rate at a paced instance only (P2.4; the trap scan's §5.4; decision
        // 406): 1.3 a year, land's and the types' rate, beside the techniques' so `adjust.*`
        // scales it with them.
        if inst.paced_exit() {
            v.push(Dial {
                key: PACE_KEY.into(),
                value: 1.3,
                unit: "RatePerYear",
                basis: PACE_DIAL.into(),
            });
        }
        let mut rate = |key: String, value: f64| {
            v.push(Dial {
                key,
                value,
                unit: "RatePerYear",
                basis: C2M.into(),
            })
        };
        for d in inst.desks() {
            rate(format!("buffer.desk.{d}.cash"), 5.2);
        }
        rate("spend.workers".into(), 13.0);
        rate("spend.provider".into(), 13.0);
        for d in inst.desks() {
            v.push(Dial {
                key: format!("tilt.desk.{d}"),
                value: 0.0,
                unit: "Dimensionless",
                basis: C2M.into(),
            });
        }
        // The free step's c at a free instance only (P2.4; the free scan's §5; decision 417): no
        // dial family scales it.
        if let Some(f) = &inst.free {
            v.push(Dial {
                key: format!("free.{}", f.good),
                value: f.scale,
                unit: "Dimensionless",
                basis: FREE_DIAL.into(),
            });
        }
        v.push(Dial {
            key: "price.ema_tc".into(),
            value: 0.5,
            unit: "Years",
            basis: "Approximate(\"the gate world's value; no rule reads the EMA\")".into(),
        });
        Dials {
            set: "c2m".into(),
            values: v,
        }
    }

    /// C2L (MARKETS-SPEC §5.2), for the loop: C2m with each type's rate at labour's, 5.2 a
    /// year, and the markup tilt 1 on every desk.
    pub fn c2l(inst: &Instance) -> Dials {
        let mut d = Dials::c2m(inst);
        d.set = "c2l".into();
        for t in &inst.types {
            let key = format!("rate.{}", t.key);
            for x in d.values.iter_mut().filter(|x| x.key == key) {
                x.value = 5.2;
                x.basis = C2L.into();
            }
        }
        for x in d.values.iter_mut().filter(|x| x.key.starts_with("tilt.")) {
            x.value = 1.0;
            x.basis = C2L.into();
        }
        d
    }

    /// A registered set by name.
    pub fn named(name: &str, inst: &Instance) -> Result<Dials, String> {
        match name {
            "c2m" => Ok(Dials::c2m(inst)),
            "c2l" => Ok(Dials::c2l(inst)),
            _ => Err(format!("no dial set {name}: c2m or c2l")),
        }
    }

    /// A dial's value by key.
    pub fn get(&self, key: &str) -> Result<f64, String> {
        self.values
            .iter()
            .find(|d| d.key == key)
            .map(|d| d.value)
            .ok_or_else(|| format!("no dial {key}"))
    }

    /// Set a dial by key, or a family: `rate.*` scales every price rate, `buffer.*` every
    /// desk's turnover and `adjust.*` every technique rate by the value; `tilt.*` sets every
    /// tilt to it (as P2.0's `--set` does). At a switch instance `rate.*` scales the switch's
    /// rates too, and `rate.switch.*` scales them alone (P2.4; the switch scan's `switch` dial).
    pub fn set(&mut self, key: &str, value: f64) -> Result<(), String> {
        let family = |prefix: &str, d: &Dial| d.key.starts_with(prefix);
        match key {
            "rate.*" | "buffer.*" | "adjust.*" | "rate.switch.*" => {
                let prefix = &key[..key.len() - 1];
                for d in self.values.iter_mut().filter(|d| family(prefix, d)) {
                    d.value *= value;
                }
                return Ok(());
            }
            "tilt.*" => {
                for d in self.values.iter_mut().filter(|d| family("tilt.", d)) {
                    d.value = value;
                }
                return Ok(());
            }
            _ => {}
        }
        let d = self
            .values
            .iter_mut()
            .find(|d| d.key == key)
            .ok_or_else(|| format!("no dial {key}"))?;
        d.value = value;
        Ok(())
    }
}

/// A genesis human share: the oracle's times a factor, or set outright.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShareAt {
    /// (1 − x\*)·f.
    Times(f64),
    /// 1 − x for this x.
    At(f64),
    /// This share exactly (the wall's `s[D]=V`, decision 396).
    Is(f64),
}

/// The paced workers' genesis share (P2.4): the point's S/N times a factor, at most 1 (the trap
/// scan's `displace`, `part.workers*F`), or set outright (`part.workers=V`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaceAt {
    /// min((S/N)·f, 1).
    Times(f64),
    /// This share exactly.
    Is(f64),
}

/// A displacement of genesis from the oracle's point, by factors, each in its list's order.
#[derive(Debug, Clone, PartialEq)]
pub struct Displacement {
    /// Each posted price, in [`Instance::markets`] order.
    pub price: Vec<f64>,
    /// Each category desk's human share.
    pub share: Vec<ShareAt>,
    /// Each actor's coin, in [`Instance::actors`] order.
    pub coin: Vec<f64>,
    /// Each desk's stock of its output, in [`Instance::desks`] order.
    pub stock: Vec<f64>,
    /// The paced workers' genesis share (P2.4); read at a paced instance only.
    pub pace: PaceAt,
    /// Each reserved pop's genesis pool share, in [`Instance::wtypes`] order (P2.4): `None` for
    /// the point's a\*, `Some(V)` for V exactly (`sw[T]=V`). Read at a switch instance only.
    pub switch: Vec<Option<f64>>,
    /// Each posted price set to V times labour's genesis price, after the factors (`p[M]=V`;
    /// P2.4, the free scan's grammar), in [`Instance::markets`] order.
    pub set: Vec<Option<f64>>,
}

impl Displacement {
    /// No displacement: mode A.
    pub fn none(inst: &Instance) -> Displacement {
        Displacement {
            price: vec![1.0; inst.markets().len()],
            share: vec![ShareAt::Times(1.0); inst.categories.len()],
            coin: vec![1.0; inst.actors().len()],
            stock: vec![1.0; inst.desks().len()],
            pace: PaceAt::Times(1.0),
            switch: vec![None; inst.wtypes.len()],
            set: vec![None; inst.markets().len()],
        }
    }
}

/// A dated change of a coefficient, by a `SetParam` at the start of `tick`.
#[derive(Debug, Clone, PartialEq)]
pub struct Shock {
    /// The tick it fires in.
    pub tick: u64,
    /// The param it sets.
    pub param: String,
    /// Its value from then on.
    pub value: f64,
}

/// Everything a markets tape is made from.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    /// Ticks per year.
    pub tpy: u32,
    /// The registered instance, whose oracle point seeds genesis.
    pub instance: Instance,
    /// The dials.
    pub dials: Dials,
    /// The one-sided rule.
    pub one_sided: OneSided,
    /// The displacement of genesis.
    pub displace: Displacement,
    /// Coefficients the tape registers at another value from tick 0 (a cost shock at genesis);
    /// genesis stays at the registered point.
    pub at_genesis: Vec<(String, f64)>,
    /// Dated coefficient changes, in tick order.
    pub shocks: Vec<Shock>,
    /// Whether a stock or coin displacement's start distance is the largest D̂ of its first
    /// scored year (Tier 3S, decision 229 as 368 and 397 carry it to the wall), not |ln F| over
    /// the tolerance (P2.1's stocks family). No tape reads it.
    pub first_year_d0: bool,
}

impl Setup {
    /// The registered setup of an instance at `tpy` ticks a year: C2m, Saturate, genesis at the
    /// oracle's point (mode A).
    pub fn registered(id: &str, tpy: u32) -> Result<Setup, String> {
        let instance = Instance::named(id)?;
        Ok(Setup {
            tpy,
            dials: Dials::c2m(&instance),
            one_sided: OneSided::Saturate,
            displace: Displacement::none(&instance),
            at_genesis: Vec::new(),
            shocks: Vec::new(),
            first_year_d0: false,
            instance,
        })
    }

    /// The coefficients changed from the registered instance and in force at `tick`, in the
    /// order they were set: those set from tick 0, then each dated shock that has fired, the
    /// latest value of each param only.
    pub fn changes_at(&self, tick: u64) -> Vec<(String, f64)> {
        let mut out: Vec<(String, f64)> = Vec::new();
        let fired = self
            .shocks
            .iter()
            .filter(|s| s.tick <= tick)
            .map(|s| (s.param.clone(), s.value));
        for (p, v) in self.at_genesis.iter().cloned().chain(fired) {
            out.retain(|(q, _)| *q != p);
            out.push((p, v));
        }
        out
    }

    /// The instance in force at `tick`.
    pub fn instance_at(&self, tick: u64) -> Result<Instance, String> {
        let mut i = self.instance.clone();
        for (p, v) in self.changes_at(tick) {
            i.set(&p, v)?;
        }
        Ok(i)
    }

    /// The instance the tape registers from tick 0.
    pub fn tape_instance(&self) -> Result<Instance, String> {
        let mut i = self.instance.clone();
        for (p, v) in &self.at_genesis {
            i.set(p, *v)?;
        }
        Ok(i)
    }
}

/// The genesis a setup writes: the undisplaced oracle point and the displaced values.
#[derive(Debug, Clone, PartialEq)]
pub struct Genesis {
    /// The oracle's point at the registered instance.
    pub point: Point,
    /// The genesis prices, in market order.
    pub prices: Vec<f64>,
    /// Each category desk's genesis human share.
    pub shares: Vec<f64>,
    /// The stationary coins at the point, before displacement, in actor order.
    pub stationary: Vec<f64>,
    /// The genesis coins, in actor order.
    pub coin: Vec<f64>,
    /// Each desk's genesis stock of its output, in desk order.
    pub stock: Vec<f64>,
    /// At a paced instance (P2.4): the workers' share at the point, S/N, and their genesis
    /// share, the tape's `pace.share`.
    pub pace: Option<(f64, f64)>,
    /// At a switch instance (P2.4): each reserved pop's genesis pool share, the tape's
    /// `pool.share`, in [`Instance::wtypes`] order; empty elsewhere.
    pub switch: Vec<f64>,
    /// At a commons market (P2.4): each commoner's decision by its rule at the point's prices,
    /// which its genesis coin reads; empty elsewhere.
    pub pops: Vec<PopMarket>,
}

/// The oracle point's P_s as the households' rule sums it: Σ z_j·p_j from 0.0 in item order,
/// space at r last.
pub fn basket_price(inst: &Instance, cat_price: &[f64], r: f64) -> f64 {
    let mut ps = 0.0;
    for (c, p) in inst.categories.iter().zip(cat_price) {
        ps += c.weight * p;
    }
    if let Some(h) = inst.space {
        ps += h * r;
    }
    ps
}

/// Genesis for a setup (MARKETS-SPEC §5.4): the oracle's point with each actor's stationary
/// coin, in the operation order that makes I0's genesis appb's bit for bit. A worker-form
/// instance (the wall frame's §3.6) takes each pop's hours from unit 1d's `WorkerEq::hours`,
/// and each reserved wage as its labour market's price; the provider's transfer is
/// τ = ((N·P_s) + N_1·P_s) + …, summed in pop order as its rule pays it. An open-commons instance
/// (the commons frame's §3.8) takes the workers' hours as unit 1e's supply S at the point, and
/// their coin as r·T_p + (N·P_s + w·S − r·T_p)/share(spend), T_p the plots' rented land, which is
/// P2.1's formula where T_p = 0.
///
/// At a free instance (P2.4; the free scan's `fm.genesis`): on the idle stretch (IL1) the prices
/// are the point's in wage units, w 1 and land 0, the provider, whose only income is rent, holds
/// no coin, and the workers' coin is w·S/share(spend); at a commons market (CT2) the commons is at
/// the point's r_o, and each commoner's coin is r·T_p,i + (N_i·P_s + w·h_i + r_o·(T_o,i − bid_i)
/// − r·T_p,i)/share(spend), its hours, bid and plots by its own rule at those prices.
pub fn genesis(s: &Setup) -> Result<Genesis, String> {
    let inst = &s.instance;
    let e = inst.point(s.tpy)?;
    let c = clock(s.tpy)?;
    let share = |v: f64| c.share(RatePerYear(v));
    let d = &s.dials;
    let n = c.flow(FlowPerYear(inst.workers));
    let t = c.flow(FlowPerYear(inst.land));
    // r is 1 in every point's units but unit 1e's idle stretch's, where it is 0 and w is 1.
    let (w, r) = (e.v, e.rent);
    let ps = basket_price(inst, &e.cat_price, r);
    let tau_w = n * ps;
    let hours = if inst.worker_form || inst.parcel() {
        e.hours[0]
    } else {
        n * (num::ln1p(w / ps) / inst.chi_max).min(1.0)
    };
    // Each reserved pop's transfer N_i·P_s, and the provider's whole transfer.
    let mut tau = tau_w;
    let mut reserved_pops = Vec::with_capacity(inst.wtypes.len());
    for (i, wt) in inst.wtypes.iter().enumerate() {
        let tq = c.flow(FlowPerYear(wt.workers)) * ps;
        tau += tq;
        let spend = d.get("spend.workers")?;
        reserved_pops.push((tq + e.wage[i] * e.hours[i + 1]) / share(spend));
    }
    let mut stationary = Vec::new();
    for (j, cat) in inst.categories.iter().enumerate() {
        let v = d.get(&format!("buffer.desk.{}.cash", cat.key))?;
        stationary.push(e.cat_price[j] * e.cat_output[j] / share(v));
    }
    for (k, ty) in inst.types.iter().enumerate() {
        // The purchased-input cost, as the desk sums it: bought services, hours, land.
        let mut cost = 0.0;
        for (l, a) in ty.row.iter().enumerate() {
            if l != k && *a > 0.0 {
                cost += a * e.type_price[l];
            }
        }
        cost += ty.labour * w;
        cost += ty.land * r;
        let v = d.get(&format!("buffer.desk.{}.cash", ty.key))?;
        stationary.push(cost * e.type_services[k] / share(v));
    }
    // Each commoner's decision at the point's prices, the commons at r_o (P2.4).
    let r_o = e.commons.as_ref().map_or(0.0, |k| k.plot_rent);
    let mut pops = Vec::with_capacity(inst.commoners.len());
    if !inst.commoners.is_empty() {
        let first = &inst.commoners[0];
        let g = inst
            .categories
            .iter()
            .position(|j| j.key == first.good)
            .ok_or("the commoners' exit good is no category")?;
        tau = 0.0;
        for k in &inst.commoners {
            let nk = c.flow(FlowPerYear(k.workers));
            tau += nk * ps;
            let to = c.flow(FlowPerYear(k.share));
            pops.push(pop_market(
                (nk, k.chi_max),
                (w, ps, e.cat_price[g], r, r_o),
                (k.gross, k.floor, k.plot, to),
            ));
        }
    }
    if e.rent == 0.0 {
        // The idle stretch (P2.4): the provider's only income is rent, 0, so it holds no coin.
        stationary.push(0.0);
    } else {
        stationary.push(tau + (r * t - tau) / share(d.get("spend.provider")?));
    }
    if inst.commoners.is_empty() {
        match e.commons.as_ref().map(|c| c.rented) {
            // On the idle stretch the provider pays no support: the workers' coin is their
            // wage bill's, w·S/share(spend) (P2.4; `fm.genesis`).
            _ if e.rent == 0.0 => stationary.push(w * hours / share(d.get("spend.workers")?)),
            // The plots' rent first, from the coin; the baskets from the rest.
            Some(tp) if tp > 0.0 => {
                let rent = r * tp;
                stationary.push(rent + (tau_w + w * hours - rent) / share(d.get("spend.workers")?));
            }
            _ => stationary.push((tau_w + w * hours) / share(d.get("spend.workers")?)),
        }
    }
    stationary.extend(reserved_pops);
    for (k, pm) in inst.commoners.iter().zip(&pops) {
        let tk = c.flow(FlowPerYear(k.workers)) * ps;
        let inc = ((tk + w * pm.hours) + r_o * (pm.offer - pm.bid)) - r * pm.plots;
        stationary.push(r * pm.plots + inc / share(d.get("spend.workers")?));
    }
    let x = &s.displace;
    let mut prices = vec![w, r];
    prices.extend(e.type_price.iter().copied());
    prices.extend(e.cat_price.iter().copied());
    prices.extend(e.wage.iter().copied());
    if !inst.commoners.is_empty() {
        prices.push(r_o);
    }
    if x.price.len() != prices.len()
        || x.set.len() != prices.len()
        || x.share.len() != inst.categories.len()
        || x.coin.len() != stationary.len()
        || x.stock.len() != inst.desks().len()
    {
        return Err("the displacement does not fit the instance".into());
    }
    for (p, f) in prices.iter_mut().zip(&x.price) {
        *p *= f;
    }
    // `p[M]=V` (P2.4): V times labour's genesis price, after the factors.
    let wage = prices[0];
    for (p, v) in prices.iter_mut().zip(&x.set) {
        if let Some(v) = v {
            *p = v * wage;
        }
    }
    let mut shares = Vec::new();
    for sh in &x.share {
        let v = match *sh {
            // At the commons a share scaled past 1 is 1, as the frame's mirror displaces it
            // (`battery_c.displace`; the registration's §3): only the basin's s[food] runs reach
            // it. Elsewhere a share above 1 is refused below.
            ShareAt::Times(f) if inst.parcel() => (e.one_minus_x * f).min(1.0),
            ShareAt::Times(f) => e.one_minus_x * f,
            ShareAt::At(xx) => 1.0 - xx,
            ShareAt::Is(v) => v,
        };
        if !(0.0..=1.0).contains(&v) {
            return Err(format!("the genesis human share {v} is outside [0, 1]"));
        }
        shares.push(v);
    }
    let coin = stationary.iter().zip(&x.coin).map(|(c, f)| c * f).collect();
    let mut stock: Vec<f64> = e.cat_output.clone();
    stock.extend(e.type_services.iter().copied());
    for (q, f) in stock.iter_mut().zip(&x.stock) {
        *q *= f;
    }
    // The paced workers' genesis share (the trap scan's §5.1, §5.4): the point's S/N, the rule's
    // share at the oracle's prices, displaced as the mirror's `displace` does.
    let pace = if inst.paced_exit() {
        let at = hours / n;
        let share = match x.pace {
            PaceAt::Times(f) => (at * f).min(1.0),
            PaceAt::Is(v) => v,
        };
        if !(0.0..=1.0).contains(&share) {
            return Err(format!(
                "the workers' genesis share {share} is outside [0, 1]"
            ));
        }
        Some((at, share))
    } else {
        None
    };
    // Each switch pop's genesis pool share (P2.4; the switch scan's §3.8): the point's a*, or V
    // exactly (`sw[T]=V`), in [0, 1].
    let mut switch = Vec::new();
    if inst.switch {
        if x.switch.len() != inst.wtypes.len() || e.pool_share.len() != inst.wtypes.len() {
            return Err("the switch's displacement does not fit the instance".into());
        }
        for (at, set) in e.pool_share.iter().zip(&x.switch) {
            let a = set.unwrap_or(*at);
            if !(0.0..=1.0).contains(&a) {
                return Err(format!("the genesis pool share {a} is outside [0, 1]"));
            }
            switch.push(a);
        }
    }
    Ok(Genesis {
        point: e,
        prices,
        shares,
        stationary,
        coin,
        stock,
        pace,
        switch,
        pops,
    })
}

fn f(x: f64) -> String {
    format!("{x:?}")
}

const ROLE: &str =
    "Assumed(\"MARKETS-SPEC §2: the probe's rules carried to many markets (docs/probe/MARKETS-RULES.md)\")";
const WALL_ROLE: &str = "Assumed(\"the wall frame's §3: the many-market roles with the tail, the \
     reserved hours and further transfers (docs/probe/WALL-RULES.md)\")";
const COMMONS_ROLE: &str =
    "Assumed(\"the commons frame's §3: the many-market roles, the workers with \
     the priced exit and their commons (docs/probe/COMMONS-RULES.md)\")";
const SWITCH_ROLE: &str = "Assumed(\"the switch scan's §3: the wall's roles, each reserved pop \
     switching between its own market and the pool (docs/probe/SWITCH-RULES.md)\")";
const FREE_ROLE: &str = "Assumed(\"the free scan's §6: the commons' roles, a market whose price \
     may be 0 and pops on a commons market (docs/probe/FREE-RULES.md)\")";

/// The tape's text, one line at a time.
struct Lines(Vec<String>);

impl Lines {
    fn line(&mut self, text: impl Into<String>) {
        self.0.push(text.into());
    }

    fn param(&mut self, key: &str, value: f64, unit: &str, basis: &str) {
        self.line(format!(
            "        (key: \"{key}\", value: {}, unit: {unit}, basis: {basis}),",
            f(value)
        ));
    }
}

fn list(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| f(*x)).collect();
    format!("[{}]", parts.join(", "))
}

/// The tape of a setup, as RON text with its derivation in the header.
pub fn tape_ron(s: &Setup) -> Result<String, String> {
    let g = genesis(s)?;
    let inst = s.tape_instance()?;
    let mut out = Lines(Vec::new());
    let o = &mut out;
    let name = format!("markets-{}", inst.id);
    let header = if inst.worker_form {
        wall_header(s, &g, &inst)?
    } else if inst.free.is_some() {
        free_header(s, &g, &inst)?
    } else if inst.exit.is_some() {
        commons_header(s, &g, &inst)?
    } else {
        p21_header(s, &g, &inst)?
    };
    for line in header {
        o.line(line);
    }
    let one_sided = match s.one_sided {
        OneSided::Saturate => "Saturate",
        OneSided::Hold => "Hold",
    };
    for line in [
        "Tape(".to_string(),
        "    schema: 1,".into(),
        "    header: (".into(),
        format!("        name: \"{name}\","),
        format!("        start: \"{START}\","),
        format!("        ticks_per_year: {},", s.tpy),
        format!(
            "        market: (rule: Imbalance, one_sided: {one_sided}, ema_time_constant: \
             \"price.ema_tc\"),"
        ),
        "        ledger: (rel_flow: \"ledger.rel_flow\", rel_stock: \"ledger.rel_stock\"),".into(),
        "    ),".into(),
        "    params: [".into(),
        "        // The ledger's tolerances, as in the gate world (A12).".into(),
    ] {
        o.line(line);
    }
    body(s, &g, &inst, o)?;
    let mut text = out.0.join("\n");
    text.push('\n');
    Ok(text)
}

/// The header of a P2.1 tape: its derivation from unit 1c.
fn p21_header(s: &Setup, g: &Genesis, inst: &Instance) -> Result<Vec<String>, String> {
    let e = &g.point;
    let c = clock(s.tpy)?;
    let base = &s.instance;
    Ok(vec![
        format!(
            "// The markets probe's {} world (MARKETS-SPEC §1; docs/probe/MARKETS-RULES.md): {},",
            inst.id.to_uppercase(),
            inst.title
        ),
        "// run by the four many-market roles, for P2.1 (2026-09-27).".into(),
        "//".into(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin markets-tape -- --inst"
            .into(),
        format!(
            "// {} <path>` writes it, and the test `markets_tapes_are_their_generators_output` checks",
            inst.id
        ),
        "// this file against the generator.".into(),
        "//".into(),
        "// Genesis is the oracle's equilibrium (MARKETS-SPEC §1.4, mode A): crates/oracle (unit 1c,"
            .into(),
        format!(
            "// P1.5) solved at the instance's per-tick values, N = {} and T = {} at {} ticks a year,",
            f(c.flow(FlowPerYear(base.workers))),
            f(c.flow(FlowPerYear(base.land))),
            s.tpy
        ),
        "// every type's flow recipe as its operating recipe (δ = J = 1, ρ = 0), gives".into(),
        format!("//   x* = {}, 1 - x* = {},", f(e.x_star), f(e.one_minus_x)),
        format!(
            "//   v = w/r = {}, P_s/r = {}, Y = {}, N_a = {},",
            f(e.v),
            f(e.p_s),
            f(e.y),
            f(e.n_a)
        ),
        format!(
            "//   type prices {} and services {},",
            list(&e.type_price),
            list(&e.type_services)
        ),
        format!(
            "//   category prices {} and outputs {}.",
            list(&e.cat_price),
            list(&e.cat_output)
        ),
        "// The genesis prices are those ratios with r = 1 coin. Each desk holds one tick's output,"
            .into(),
        "// each category desk's human share is 1 - x*, and each actor's coin is its stationary"
            .into(),
        format!(
            "// balance under the dials, share(v) = 1 - exp(-v/{}) (MARKETS-SPEC §5.4):",
            s.tpy
        ),
        "//   category desk  p_j*y_j/share(turnover)".into(),
        "//   type desk      (sum_l a_kl*p_l + lam_k*w + b_k*r)*X_k/share(turnover)".into(),
        "//   provider       N*P_s + (r*T - N*P_s)/share(spend.provider)".into(),
        "//   workers        (N*P_s + w*N*F)/share(spend.workers), F = ln1p(w/P_s)/chi_max".into(),
        format!("// giving {} in actor order", list(&g.stationary)),
        format!("// ({}).", inst.actors().join(", ")),
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .into(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ])
}

/// The header of a wall tape (P2.3; docs/probe/WALL-RULES.md §4): its derivation from unit 1d.
fn wall_header(s: &Setup, g: &Genesis, inst: &Instance) -> Result<Vec<String>, String> {
    let e = &g.point;
    let c = clock(s.tpy)?;
    let base = &s.instance;
    let mut heads = vec![f(c.flow(FlowPerYear(base.workers)))];
    heads.extend(
        base.wtypes
            .iter()
            .map(|t| f(c.flow(FlowPerYear(t.workers)))),
    );
    // A switch instance (P2.4; docs/probe/SWITCH-RULES.md) names its frame, its step and its
    // test; IW1's lines are as they were.
    let (frame, intro, test) = if inst.switch {
        (
            "docs/probe/switch/SPEC.md; docs/probe/SWITCH-RULES.md",
            vec![
                "// run by the four many-market roles with the wall's optional fields, each reserved pop"
                    .to_string(),
                "// switching between its own market and the pool (its optional `pool`: the type switch"
                    .into(),
                "// at the wall, the migration rule), for P2.4 (2026-09-30).".into(),
            ],
            "markets_is1_tape_is_its_generators_output",
        )
    } else {
        (
            "docs/probe/wall/SPEC.md; docs/probe/WALL-RULES.md",
            vec![
                "// run by the four many-market roles with the wall's optional fields (a category desk's"
                    .to_string(),
                "// tail and reserved hours, the provider's further transfers), for P2.3 (2026-09-30)."
                    .into(),
            ],
            "markets_iw1_tape_is_its_generators_output",
        )
    };
    let mut out = vec![
        format!("// The wall world {} ({frame}):", inst.id.to_uppercase(),),
        format!("// {},", inst.title),
    ];
    out.extend(intro);
    out.extend([
        "//".to_string(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin markets-tape -- --inst"
            .into(),
        format!("// {} <path>` writes it, and the test `{test}` checks", inst.id),
        "// this file against the generator.".into(),
        "//".into(),
        "// Genesis is the oracle's equilibrium (mode A): crates/oracle unit 1d,".into(),
        format!(
            "// WorkerEconomy::solve, at the instance's per-tick values, N = {} by worker type and",
            list_s(&heads)
        ),
        format!(
            "// T = {} at {} ticks a year, the machine's flow recipe as its operating recipe (δ = J = 1,",
            f(c.flow(FlowPerYear(base.land))),
            s.tpy
        ),
        "// ρ = 0), gives".into(),
        format!(
            "//   margin {}, x* = {}, 1 - x* = {},",
            e.margin,
            f(e.x_star),
            f(e.one_minus_x)
        ),
        format!("//   v = w/r = {}, P_s/r = {}, Y = {},", f(e.v), f(e.p_s), f(e.y)),
        format!("//   the pool's hours n_D = {},", f(e.pool)),
        format!("//   reserved wages {},", list(&e.wage)),
        format!("//   hours by worker type {},", list(&e.hours)),
        format!(
            "//   type prices {} and services {},",
            list(&e.type_price),
            list(&e.type_services)
        ),
        format!(
            "//   category prices {} and outputs {}.",
            list(&e.cat_price),
            list(&e.cat_output)
        ),
        "// The genesis prices are those ratios with r = 1 coin, each reserved wage its labour"
            .into(),
        "// market's price. Each desk holds one tick's output, each category desk's human share is"
            .into(),
        "// 1 - x*, and each actor's coin is its stationary balance under the dials,".into(),
        format!(
            "// share(v) = 1 - exp(-v/{}) (MARKETS-SPEC §5.4; the wall frame's §3.6):",
            s.tpy
        ),
        "//   category desk  p_j*y_j/share(turnover)".into(),
        "//   type desk      (sum_l a_kl*p_l + lam_k*w + b_k*r)*X_k/share(turnover)".into(),
        "//   provider       tau + (r*T - tau)/share(spend.provider), tau = ((N*P_s) + N_1*P_s) + ..."
            .into(),
        "//   each pop       (N_i*P_s + w_i*hours_i)/share(spend.workers), hours_i unit 1d's".into(),
        format!("// giving {} in actor order", list(&g.stationary)),
        format!("// ({}).", inst.actors().join(", ")),
    ]);
    if inst.switch {
        let eff: Vec<f64> = base.wtypes.iter().map(|t| t.efficiency).collect();
        out.extend([
            format!(
                "// The reserved types' efficiencies are {}; each reserved pop's pool share moves toward",
                list(&eff)
            ),
            "// the market that pays more at the rate `rate.switch.<type>` a year (the switch scan's §3.3);"
                .into(),
            format!(
                "// the point's shares a* (pool hours over hours) are {}, pooled {:?},",
                list(&e.pool_share),
                e.pooled
            ),
            format!(
                "// switch distances {}, and the genesis shares written are {}.",
                list(&e.switch_distance),
                list(&g.switch)
            ),
        ]);
    }
    out.extend([
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .to_string(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ]);
    Ok(out)
}

/// The header of an open-commons tape (P2.3; docs/probe/COMMONS-RULES.md §4): its derivation
/// from unit 1e.
fn commons_header(s: &Setup, g: &Genesis, inst: &Instance) -> Result<Vec<String>, String> {
    let e = &g.point;
    let c = clock(s.tpy)?;
    let base = &s.instance;
    let x = base
        .exit
        .as_ref()
        .ok_or("an open-commons tape without an exit")?;
    let k = e
        .commons
        .as_ref()
        .ok_or("an open-commons point without unit 1e's readouts")?;
    // A paced instance (P2.4; docs/probe/TRAP-RULES.md) names its step, its test and its pace.
    let (intro, test) = match g.pace {
        Some(_) => (
            vec![
                "// run by the four many-market roles, the workers with the priced exit in food, the"
                    .to_string(),
                "// commons they hold and their participation at a rate (their optional `exit` with"
                    .into(),
                "// its `pace`: the trap's remedy, docs/probe/trap/SPEC.md), for P2.4 (2026-09-30)."
                    .into(),
            ],
            "commons_paced_tapes_are_their_generators_output",
        ),
        None => (
            vec![
                "// run by the four many-market roles, the workers with the priced exit in food and the"
                    .to_string(),
                "// commons they hold (their optional `exit`), for P2.3 (2026-09-30).".into(),
            ],
            "commons_tapes_are_their_generators_output",
        ),
    };
    let mut out = vec![
        format!(
            "// The open-commons world {} (docs/probe/commons/SPEC.md; docs/probe/COMMONS-RULES.md):",
            inst.id.to_uppercase(),
        ),
        format!("// {},", inst.title),
    ];
    out.extend(intro);
    out.extend([
        "//".to_string(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin markets-tape -- --inst"
            .into(),
        format!("// {} <path>` writes it, and the test `{test}` checks", inst.id),
        "// this file against the generator.".into(),
        "//".into(),
        "// Genesis is the oracle's equilibrium (mode A): crates/oracle unit 1e, ParcelEconomy::solve,"
            .into(),
        format!(
            "// at the instance's per-tick values, N = {}, the enclosed land T = {} and the commons",
            f(c.flow(FlowPerYear(base.workers))),
            f(c.flow(FlowPerYear(base.land)))
        ),
        format!(
            "// T_o = {} at {} ticks a year, the exit (s0 {}, floor {}, h {}) in {}, the machine's flow",
            f(c.flow(FlowPerYear(x.commons))),
            s.tpy,
            f(x.gross),
            f(x.floor),
            f(x.plot),
            x.good
        ),
        "// recipe as its operating recipe (δ = J = 1, ρ = 0), gives".into(),
        format!(
            "//   the plots {}, plot rent r_o = {}, rented plots T_p = {}, commons used {},",
            k.regime,
            f(k.plot_rent),
            f(k.rented),
            f(k.occupied)
        ),
        format!("//   x* = {}, 1 - x* = {},", f(e.x_star), f(e.one_minus_x)),
        format!(
            "//   v = w/r = {}, P_s/r = {}, Y = {}, the supply S = {},",
            f(e.v),
            f(e.p_s),
            f(e.y),
            f(e.pool)
        ),
        format!(
            "//   type prices {} and services {},",
            list(&e.type_price),
            list(&e.type_services)
        ),
        format!(
            "//   category prices {} and outputs {}.",
            list(&e.cat_price),
            list(&e.cat_output)
        ),
        "// The genesis prices are those ratios with r = 1 coin. Each desk holds one tick's output,"
            .into(),
        "// each category desk's human share is 1 - x*, and each actor's coin is its stationary"
            .into(),
        format!(
            "// balance under the dials, share(v) = 1 - exp(-v/{}) (MARKETS-SPEC §5.4; the commons frame's §3.8):",
            s.tpy
        ),
        "//   category desk  p_j*y_j/share(turnover)".into(),
        "//   type desk      (sum_l a_kl*p_l + lam_k*w + b_k*r)*X_k/share(turnover)".into(),
        "//   provider       N*P_s + (r*T - N*P_s)/share(spend.provider)".into(),
        "//   workers        r*T_p + (N*P_s + w*S - r*T_p)/share(spend.workers)".into(),
        format!("// giving {} in actor order", list(&g.stationary)),
        format!("// ({}).", inst.actors().join(", ")),
    ]);
    if let Some((at, share)) = g.pace {
        out.extend([
            "// The workers' share moves 1 - exp(-rate/tpy) of its gap to the rule's hours over N each"
                .to_string(),
            format!("// tick, the rate `{PACE_KEY}` a year. The point's S/N is {},", f(at)),
            format!(
                "// and the genesis share written is {} (the trap scan's §5.1, §5.4).",
                f(share)
            ),
        ]);
    }
    out.extend([
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .to_string(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ]);
    Ok(out)
}

fn list_s(v: &[String]) -> String {
    format!("[{}]", v.join(", "))
}

/// The header of a free instance's tape (P2.4; docs/probe/FREE-RULES.md): its derivation from
/// unit 1e, its free-able market and, at a commons market, each commoner's genesis decision.
fn free_header(s: &Setup, g: &Genesis, inst: &Instance) -> Result<Vec<String>, String> {
    let e = &g.point;
    let c = clock(s.tpy)?;
    let base = &s.instance;
    let k = e
        .commons
        .as_ref()
        .ok_or("a free instance's point without unit 1e's readouts")?;
    let fr = inst
        .free
        .as_ref()
        .ok_or("a free tape without a free market")?;
    let units = if e.rent == 0.0 {
        "per unit of the pool's wage, w = 1, r = 0 (the idle stretch)"
    } else {
        "per unit of rent, r = 1"
    };
    let mut out = vec![
        format!(
            "// The free world {} (docs/probe/free/SPEC.md; docs/probe/FREE-RULES.md):",
            inst.id.to_uppercase()
        ),
        format!("// {},", inst.title),
        format!(
            "// run by the four many-market roles, the good `{}` with a free step (its price may be 0),",
            fr.good
        ),
        "// for P2.4 (2026-09-30).".to_string(),
        "//".into(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin markets-tape -- --inst"
            .into(),
        format!(
            "// {} <path>` writes it, and the test `il1_ct2_tapes_are_their_generators_output` checks",
            inst.id
        ),
        "// this file against the generator.".into(),
        "//".into(),
        "// Genesis is the oracle's equilibrium (mode A): crates/oracle unit 1e, ParcelEconomy::solve,"
            .into(),
        format!(
            "// at the instance's per-tick values, the enclosed land T = {} at {} ticks a year, the",
            f(c.flow(FlowPerYear(base.land))),
            s.tpy
        ),
        "// machine's flow recipe as its operating recipe (δ = J = 1, ρ = 0), gives".into(),
        format!(
            "//   land {}, the plots {}, plot rent r_o = {}, rented plots T_p = {}, commons used {},",
            k.land_market,
            k.regime,
            f(k.plot_rent),
            f(k.rented),
            f(k.occupied)
        ),
        format!(
            "//   the market's land T_m = {}, idle T_idle = {}, provider baskets {}, funded {},",
            f(k.market_land),
            f(k.idle),
            f(e.provider_baskets),
            k.funded
        ),
        format!("//   x* = {}, 1 - x* = {},", f(e.x_star), f(e.one_minus_x)),
        format!(
            "//   v = {}, P_s = {}, Y = {}, the supply S = {}, hours by pop {},",
            f(e.v),
            f(e.p_s),
            f(e.y),
            f(e.pool),
            list(&e.hours)
        ),
        format!(
            "//   type prices {} and services {},",
            list(&e.type_price),
            list(&e.type_services)
        ),
        format!(
            "//   category prices {} and outputs {}, {units}.",
            list(&e.cat_price),
            list(&e.cat_output)
        ),
        "// Each desk holds one tick's output, each category desk's human share is 1 - x*, and each"
            .into(),
        format!(
            "// actor's coin is its stationary balance, share(v) = 1 - exp(-v/{}) (the free scan's",
            s.tpy
        ),
        "// fm.genesis):".into(),
        "//   category desk  p_j*y_j/share(turnover)".into(),
        "//   type desk      (sum_l a_kl*p_l + lam_k*w + b_k*r)*X_k/share(turnover)".into(),
    ];
    if inst.commoners.is_empty() {
        out.extend([
            "//   provider       0 on the idle stretch: its only income is rent, 0 at r = 0"
                .to_string(),
            "//   workers        w*S/share(spend.workers): the provider pays min(N*P_s, coin) = 0"
                .into(),
        ]);
    } else {
        out.extend([
            "//   provider       tau + (r*T - tau)/share(spend.provider), tau = sum_i N_i*P_s".to_string(),
            "//   each pop       r*T_p,i + (N_i*P_s + w*h_i + r_o*(T_o,i - bid_i) - r*T_p,i)/share(spend.workers)"
                .into(),
        ]);
        for (k, p) in inst.commoners.iter().zip(&g.pops) {
            out.push(format!(
                "//                  {}: hours {}, commons bid {}, offer {}, T_p {}",
                k.pop(),
                f(p.hours),
                f(p.bid),
                f(p.offer),
                f(p.plots)
            ));
        }
    }
    out.extend([
        format!("// giving {} in actor order", list(&g.stationary)),
        format!("// ({}).", inst.actors().join(", ")),
        "// The free step (FREE-SPEC §6.1): p' = p*e^(kx) + (c*p_ref)*expm1(kx), 0 where that is not"
            .to_string(),
        format!(
            "// positive, p_ref the price of `{}`, c the dial `free.{}` = {}.",
            fr.reference,
            fr.good,
            f(fr.scale)
        ),
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .into(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ]);
    Ok(out)
}

/// A tape's params, goods, nodes, classes, actors, genesis and events.
fn body(s: &Setup, g: &Genesis, inst: &Instance, o: &mut Lines) -> Result<(), String> {
    let c = clock(s.tpy)?;
    let tau = inst.task_type()?;
    let n = c.flow(FlowPerYear(inst.workers));
    let t = c.flow(FlowPerYear(inst.land));
    let markets = inst.markets();
    let role = if inst.switch {
        SWITCH_ROLE
    } else if inst.worker_form {
        WALL_ROLE
    } else if inst.free.is_some() {
        FREE_ROLE
    } else if inst.exit.is_some() {
        COMMONS_ROLE
    } else {
        ROLE
    };
    o.param(
        "ledger.rel_flow",
        1e-12,
        "Dimensionless",
        "Assumed(\"July REL_FLOW, as the gate world\")",
    );
    o.param(
        "ledger.rel_stock",
        1e-11,
        "Dimensionless",
        "Assumed(\"July REL_STOCK, as the gate world\")",
    );
    if inst.worker_form {
        o.line("        // The instance (the wall frame's §2.1, §3.5).");
    } else if inst.free.is_some() {
        o.line("        // The instance (MARKETS-SPEC §1.2, §2.9; the free scan's §3.1).");
    } else if inst.exit.is_some() {
        o.line("        // The instance (MARKETS-SPEC §1.2, §2.9; the commons frame's §2.1).");
    } else {
        o.line("        // The instance (MARKETS-SPEC §1.2, §2.9).");
    }
    for (key, value, unit, basis) in inst.params()? {
        // The wall frame's own numbers are its calibration, assumed; the rest are literature.
        let kind = if basis == WALL_BASIS
            || basis == COMMONS_BASIS
            || basis == SWITCH_BASIS
            || basis == FREE_BASIS
        {
            "Assumed"
        } else {
            "Literature"
        };
        o.param(&key, value, unit, &format!("{kind}(\"{basis}\")"));
    }
    o.line("        // Structure: every produced good lasts one tick (MARKETS-SPEC §2.1).");
    o.param(
        "life.one_tick",
        1.0 / f64::from(s.tpy),
        "Years",
        "Assumed(\"one tick: J = 1 with delta = 1 (PROBE-SPEC §1.2)\")",
    );
    o.line(format!(
        "        // The dials ({}; MARKETS-SPEC §5). The wage is the fastest price.",
        s.dials.set
    ));
    for d in &s.dials.values {
        o.param(&d.key, d.value, d.unit, &d.basis);
    }
    if !s.shocks.is_empty() {
        o.line("        // The dated shocks' values: schedule params (MARKETS-SPEC §7.7).");
        // Each carries the unit of the param it sets (P2.3.12): the commons' `inst.commons` and
        // `inst.land` are `FlowPerYear`, which `SetParam` refuses to set from a `Dimensionless`
        // param. Every other coefficient is `Dimensionless`, as before.
        let units: Vec<(String, &'static str)> = inst
            .params()?
            .into_iter()
            .map(|(key, _, unit, _)| (key, unit))
            .collect();
        for (k, sh) in s.shocks.iter().enumerate() {
            let unit = units
                .iter()
                .find(|(key, _)| *key == sh.param)
                .map_or("Dimensionless", |(_, unit)| *unit);
            o.param(
                &format!("{}.shock.{}", sh.param, k + 1),
                sh.value,
                unit,
                "Assumed(\"MARKETS-SPEC §1.5, §7.7: a dated cost shock\")",
            );
        }
    }
    o.line("    ],");
    o.line("    goods: [");
    o.line("        (key: \"coin\", life: Indefinite, price_rate: None),");
    o.line("        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),");
    // A free-able good's free step (P2.4; the free scan's §6.1), written only where it has one.
    let free = |good: &str| match &inst.free {
        Some(f) if f.good == good => format!(
            ", free: Some((reference: \"{}\", scale: \"free.{}\"))",
            f.reference, f.good
        ),
        _ => String::new(),
    };
    o.line(format!(
        "        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\"){}),",
        free("land")
    ));
    let produced = 2 + inst.types.len() + inst.categories.len();
    for m in &markets[2..produced] {
        o.line(format!(
            "        (key: \"{m}\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.{m}\")),"
        ));
    }
    // Each reserved type's hours, Instant as the pool's are (the wall frame's §3.5), and the
    // commons where pops trade it (P2.4), Instant as land is.
    for m in &markets[produced..] {
        o.line(format!(
            "        (key: \"{m}\", life: Instant, price_rate: Some(\"rate.{m}\"){}),",
            free(m)
        ));
    }
    o.line("    ],");
    o.line("    nodes: [(key: \"home\", currency: \"coin\")],");
    o.line("    channels: [],");
    let mut classes: Vec<String> = inst
        .desks()
        .iter()
        .map(|d| format!("\"{d}_desks\""))
        .collect();
    classes.extend(inst.household_classes().iter().map(|k| format!("\"{k}\"")));
    o.line(format!("    classes: [{}],", classes.join(", ")));
    o.line("    actors: [");
    let scale = |desk: &str| {
        format!(
            "            scale: Cash((turnover: \"buffer.desk.{desk}.cash\", tilt: \
             \"tilt.desk.{desk}\", payout: None)),"
        )
    };
    let task = &inst.types[tau].key;
    let mut edges: Vec<String> = Vec::new();
    for s_ in 0..inst.edges.len() {
        edges.push(format!("\"inst.edge.{}\"", s_ + 1));
    }
    for (j, cat) in inst.categories.iter().enumerate() {
        let density: Vec<String> = (0..inst.segments())
            .map(|s_| format!("\"inst.{}.mu.{}\"", cat.key, s_ + 1))
            .collect();
        o.line(format!(
            "        (key: \"desk.{0}\", kind: Desk, class: \"{0}_desks\", home: \"home\", basis: {role},",
            cat.key
        ));
        o.line("         spec: CategoryDesk((");
        o.line(format!(
            "            output: \"{}\", labour: \"labour\", service: \"{task}\", land: \"land\",",
            cat.key
        ));
        o.line(format!(
            "            theta: \"inst.{task}.theta\", direct_land: \"inst.{}.land\",",
            cat.key
        ));
        o.line(
            "            schedule: (eta: \"inst.eta\", g0: \"inst.g0\", g1: \"inst.g1\", k: \"inst.k\"),",
        );
        o.line(format!(
            "            line: (edges: [{}], density: [{}]),",
            edges.join(", "),
            density.join(", ")
        ));
        o.line(format!(
            "            technique: (adjust: \"adjust.technique.{}\", share: {}),",
            cat.key,
            f(g.shares[j])
        ));
        o.line(scale(&cat.key));
        // The wall's optional fields (P2.3), written only where the category has them.
        if cat.tail > 0.0 {
            o.line(format!(
                "            tail: Some(\"inst.{}.tail\"),",
                cat.key
            ));
        }
        let reserved: Vec<String> = inst
            .wtypes
            .iter()
            .zip(&cat.reserved)
            .filter(|(_, r)| **r > 0.0)
            .map(|(w, _)| {
                format!(
                    "(good: \"{}\", coef: \"inst.{}.reserved.{}\")",
                    w.market(),
                    cat.key,
                    w.key
                )
            })
            .collect();
        if !reserved.is_empty() {
            o.line(format!("            reserved: [{}],", reserved.join(", ")));
        }
        o.line("         ))),");
    }
    for (k, ty) in inst.types.iter().enumerate() {
        let inputs: Vec<String> = inst
            .types
            .iter()
            .enumerate()
            .filter(|(l, _)| *l != k && ty.row[*l] > 0.0)
            .map(|(_, other)| {
                format!(
                    "(good: \"{0}\", coef: \"inst.{1}.in.{0}\")",
                    other.key, ty.key
                )
            })
            .collect();
        o.line(format!(
            "        (key: \"desk.{0}\", kind: Desk, class: \"{0}_desks\", home: \"home\", basis: {role},",
            ty.key
        ));
        o.line("         spec: TypeDesk((");
        o.line(format!(
            "            output: \"{}\", labour: \"labour\", land: \"land\",",
            ty.key
        ));
        o.line(format!(
            "            recipe: (own: \"inst.{0}.own\", inputs: [{1}], labour: \"inst.{0}.labour\", \
             land: \"inst.{0}.land\"),",
            ty.key,
            inputs.join(", ")
        ));
        o.line(scale(&ty.key));
        o.line("         ))),");
    }
    let mut items: Vec<String> = inst
        .categories
        .iter()
        .map(|c| format!("(good: \"{0}\", weight: \"inst.{0}.weight\")", c.key))
        .collect();
    if inst.space.is_some() {
        items.push("(good: \"land\", weight: \"inst.space.weight\")".into());
    }
    let basket = format!("            basket: [{}],", items.join(", "));
    // The provider's first transfer: to the workers, or at a commons market (P2.4) to the first
    // commoner; the rest, one per reserved pop or further commoner in order, as `more`.
    let first = match inst.commoners.first() {
        Some(k) => format!("(to: \"{}\", heads: \"inst.{}.workers\")", k.pop(), k.key),
        None => "(to: \"workers\", heads: \"inst.workers\")".to_string(),
    };
    for line in [
        format!(
            "        (key: \"provider\", kind: Pop, class: \"owners\", home: \"home\", basis: {role},"
        ),
        "         spec: BasketProvider((".into(),
        "            land: \"land\", endowment: \"inst.land\",".into(),
        format!("            transfer: {first},"),
        basket.clone(),
        "            spend: \"spend.provider\",".into(),
    ] {
        o.line(line);
    }
    // The further transfers, one per reserved pop in order (P2.3), or per commoner after the
    // first (P2.4), written only if any.
    let mut more: Vec<String> = inst
        .wtypes
        .iter()
        .map(|w| format!("(to: \"{}\", heads: \"inst.{}.workers\")", w.pop(), w.key))
        .collect();
    more.extend(
        inst.commoners
            .iter()
            .skip(1)
            .map(|k| format!("(to: \"{}\", heads: \"inst.{}.workers\")", k.pop(), k.key)),
    );
    if !more.is_empty() {
        o.line(format!("            more: [{}],", more.join(", ")));
    }
    o.line("         ))),");
    let mut pops = Vec::new();
    if inst.commoners.is_empty() {
        pops.push((
            "workers".to_string(),
            "workers".to_string(),
            "labour".to_string(),
            "inst.workers".to_string(),
            "inst.chi_max".to_string(),
        ));
    }
    for w in &inst.wtypes {
        pops.push((
            w.pop(),
            format!("{}_workers", w.key),
            w.market(),
            format!("inst.{}.workers", w.key),
            format!("inst.{}.chi_max", w.key),
        ));
    }
    for (i, (key, class, labour, heads, chi)) in pops.into_iter().enumerate() {
        for line in [
            format!(
                "        (key: \"{key}\", kind: Pop, class: \"{class}\", home: \"home\", basis: {role},"
            ),
            "         spec: BasketWorkers((".into(),
            format!("            labour: \"{labour}\", heads: \"{heads}\", chi_max: \"{chi}\","),
            basket.clone(),
            "            spend: \"spend.workers\",".into(),
        ] {
            o.line(line);
        }
        // The pool's workers' priced exit and commons (P2.3), written only where the instance
        // has one.
        if let (0, Some(x)) = (i, &inst.exit) {
            // With the pace (P2.4; the trap scan's §5.1), written last in the block, only where
            // the instance is paced.
            let pace = match g.pace {
                Some((_, share)) => format!(
                    ", pace: Some((adjust: \"{PACE_KEY}\", share: {}))",
                    f(share)
                ),
                None => String::new(),
            };
            o.line(format!(
                "            exit: Some((good: \"{}\", gross: \"inst.exit.gross\", floor: \
                 \"inst.exit.floor\", plot: \"inst.exit.plot\", commons: \"inst.commons\", land: \
                 \"land\"{pace})),",
                x.good
            ));
        }
        // Each reserved pop's pool at a switch instance (P2.4; the switch scan's §3.7), written
        // last in its block.
        if let (true, Some(k)) = (inst.switch, i.checked_sub(1)) {
            let t = &inst.wtypes[k];
            o.line(format!(
                "            pool: Some((good: \"labour\", efficiency: \"inst.{0}.efficiency\", rate: \
                 \"rate.switch.{0}\", share: {1})),",
                t.key,
                f(g.switch[k])
            ));
        }
        o.line("         ))),");
    }
    // Each commoner's pop on the commons' market (P2.4; the free scan's §6.3), its exit with
    // `market`, last in its block.
    for k in &inst.commoners {
        for line in [
            format!(
                "        (key: \"{}\", kind: Pop, class: \"{}_workers\", home: \"home\", basis: {role},",
                k.pop(),
                k.key
            ),
            "         spec: BasketWorkers((".into(),
            format!(
                "            labour: \"labour\", heads: \"inst.{0}.workers\", chi_max: \"inst.{0}.chi_max\",",
                k.key
            ),
            basket.clone(),
            "            spend: \"spend.workers\",".into(),
            format!(
                "            exit: Some((good: \"{1}\", gross: \"inst.{0}.exit.gross\", floor: \
                 \"inst.{0}.exit.floor\", plot: \"inst.{0}.exit.plot\", commons: \
                 \"inst.{0}.commons\", land: \"land\", market: Some(\"commons\"))),",
                k.key, k.good
            ),
            "         ))),".into(),
        ] {
            o.line(line);
        }
    }
    let genesis_basis = if inst.free.is_some() {
        format!(
            "        basis: Approximate(\"oracle unit 1e (crates/oracle, ParcelEconomy) at the free \
             scan's {}, T = {} per tick; stationary coins per its fm.genesis; written by \
             rustyecon-probe's markets-tape\"),",
            inst.id.to_uppercase(),
            f(t)
        )
    } else if inst.exit.is_some() {
        format!(
            "        basis: Approximate(\"oracle unit 1e (crates/oracle, ParcelEconomy) at the commons \
             frame's {}, N = {} and T = {} per tick; stationary coins per the commons frame's \
             §3.8; written by rustyecon-probe's markets-tape\"),",
            inst.id.to_uppercase(),
            f(n),
            f(t)
        )
    } else if inst.switch {
        format!(
            "        basis: Approximate(\"oracle unit 1d (crates/oracle, WorkerEconomy) at the switch \
             scan's {}, N = {} for the pool and T = {} per tick; stationary coins and pool shares \
             per the switch scan's §3.8; written by rustyecon-probe's markets-tape\"),",
            inst.id.to_uppercase(),
            f(n),
            f(t)
        )
    } else if inst.worker_form {
        format!(
            "        basis: Approximate(\"oracle unit 1d (crates/oracle, WorkerEconomy) at the wall \
             frame's {}, N = {} for the pool and T = {} per tick; stationary coins per the wall \
             frame's §3.6; written by rustyecon-probe's markets-tape\"),",
            inst.id.to_uppercase(),
            f(n),
            f(t)
        )
    } else {
        format!(
            "        basis: Approximate(\"oracle unit 1c (crates/oracle, P1.5) at MARKETS-SPEC's {}, \
             N = {} and T = {} per tick; stationary coins per MARKETS-SPEC §5.4; written by \
             rustyecon-probe's markets-tape\"),",
            inst.id.to_uppercase(),
            f(n),
            f(t)
        )
    };
    for line in [
        "    ],".to_string(),
        "    genesis: (".into(),
        genesis_basis,
        "        prices: [".into(),
    ] {
        o.line(line);
    }
    for (m, p) in markets.iter().zip(&g.prices) {
        o.line(format!(
            "            (node: \"home\", good: \"{m}\", price: {}),",
            f(*p)
        ));
    }
    o.line("        ],");
    o.line("        holdings: [");
    for (i, d) in inst.desks().iter().enumerate() {
        o.line(format!(
            "            (holder: \"desk.{d}\", goods: [(\"coin\", {}), (\"{d}\", {})]),",
            f(g.coin[i]),
            f(g.stock[i])
        ));
    }
    let nd = inst.desks().len();
    for (i, h) in inst.households().iter().enumerate() {
        o.line(format!(
            "            (holder: \"{h}\", goods: [(\"coin\", {})]),",
            f(g.coin[nd + i])
        ));
    }
    o.line("        ],");
    o.line("    ),");
    o.line("    events: [");
    for (k, sh) in s.shocks.iter().enumerate() {
        let date = c
            .date_of(sh.tick)
            .ok_or_else(|| format!("tick {} has no date", sh.tick))?;
        o.line(format!(
            "        (key: \"shock.{}\", at: \"{date}\", basis: Assumed(\"MARKETS-SPEC §7.7: a \
             dated cost shock at tick {}\"),",
            k + 1,
            sh.tick
        ));
        o.line(format!(
            "         act: SetParam(param: \"{}\", to: \"{}.shock.{}\")),",
            sh.param,
            sh.param,
            k + 1
        ));
    }
    o.line("    ],");
    o.line("    recurring: [],");
    o.line(")");
    Ok(())
}
