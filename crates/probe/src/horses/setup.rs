//! The stocks probe's tapes and their generator (HORSES-SPEC §2, §5; docs/probe/HORSES-RULES.md).
//!
//! A [`Setup`] names what a run is made from: the instance, the tick length, the dials, the
//! rule variants (the good desk's task assignment, the capacity desk's order rule, the maker's
//! cover and its reservation, L0.4), the displacement of genesis, the county's machine land from tick 0, and any dated
//! shocks of it. [`tape_ron`] writes it as a tape of P2.1's households and fodder desk, P2.0's
//! good desk and the three stock kinds. The oracle's equilibrium (unit 1g, solved here, outside
//! any `Sim`) seeds genesis: its prices relative to r = 1 coin, 1 − x\* for the good desk, each
//! desk's stock at its rest value, and each actor's stationary coin under the dials (HORSES-SPEC
//! §5.3). No agent reads the oracle (R13). `tapes/horses-<id>.ron` is
//! `tape_ron(&Setup::registered(id, 52))`, checked by a test.

use super::instance::{Config, Instance, Point, HOURS, RULE_A};
use crate::markets::setup::{Dial, Dials, ShareAt};
use crate::setup::{clock, OneSided, START};
use rustyecon_core::{num, FlowPerYear, RatePerYear, Years};
use rustyecon_engine::rustyecon_agents::{Assign, OrderRule};

const C2G: &str = "Assumed(\"GOODS-CHAIN C2g (D-G13); HORSES-SPEC §5\")";
const C2G13: &str =
    "Assumed(\"HORSES-SPEC §5.1: C2g with fodder's price at C2m's type rate (F7-F10)\")";
const C2: &str = "Assumed(\"MARKETS-SPEC C2m, design-analytic-first C2 per role\")";
const STOCK: &str = "Assumed(\"GOODS-CHAIN D-G3, D-G5 (C2g); HORSES-SPEC §5.2\")";
const RESERVE: &str = "Assumed(\"IDLE-SPEC 2026-09-29, mirror scan\")";

fn rate(key: String, value: f64, basis: &str) -> Dial {
    Dial {
        key,
        value,
        unit: "RatePerYear",
        basis: basis.into(),
    }
}

/// The dials of an instance: a registered set by name (HORSES-SPEC §5).
///
/// - `c2g` (D-G13, the registered set): labour's price rate 5.2 a year; land's 0.1625, C2's ÷
///   8; the machine and hour markets' and fodder's 5.2 (C2L's type rate, §5.1); the good's 2.6;
///   each desk's turnover 5.2 and tilt 0; the technique 2.6; household spending 13.
/// - `c2g13`: `c2g` with fodder's rate at 1.3 (F7–F10).
/// - `c2`: P2.1's C2m per role (labour 5.2, land and every machine-side market 1.3, the good
///   2.6), for R1a (the stocks layer off) and P7.
///
/// And the stock rules' dials (§5.2): the capacity desk's s_K at 2δ a year, the maker's s_Km
/// and the owner desk's s_K at 0, and the maker's cover at 4 weeks.
pub fn dials(inst: &Instance, name: &str) -> Result<Dials, String> {
    let (land, machine, fodder, basis) = match name {
        "c2g" => (0.1625, 5.2, 5.2, C2G),
        "c2g13" => (0.1625, 5.2, 1.3, C2G13),
        "c2" => (1.3, 1.3, 1.3, C2),
        _ => return Err(format!("no dial set {name}: c2g, c2g13 or c2")),
    };
    let mut v = Vec::new();
    v.push(rate("rate.labour".into(), 5.2, basis));
    v.push(rate("rate.land".into(), land, basis));
    for m in inst.markets() {
        match m.as_str() {
            "labour" | "land" => {}
            "good" => v.push(rate("rate.good".into(), 2.6, basis)),
            "fodder" => v.push(rate("rate.fodder".into(), fodder, basis)),
            _ => v.push(rate(format!("rate.{m}"), machine, basis)),
        }
    }
    v.push(rate("adjust.technique.good".into(), 2.6, basis));
    for d in inst.desks() {
        v.push(rate(format!("buffer.desk.{d}.cash"), 5.2, basis));
    }
    v.push(rate("spend.workers".into(), 13.0, basis));
    v.push(rate("spend.provider".into(), 13.0, basis));
    for d in inst.desks() {
        v.push(Dial {
            key: format!("tilt.desk.{d}"),
            value: 0.0,
            unit: "Dimensionless",
            basis: basis.into(),
        });
    }
    v.push(Dial {
        key: "price.ema_tc".into(),
        value: 0.5,
        unit: "Years",
        basis: "Approximate(\"the gate world's value; no rule reads the EMA\")".into(),
    });
    // The stock rules' dials.
    let maker = &inst.keys.maker;
    match inst.config {
        Config::Wet => v.push(rate(
            "adjust.invest.capacity".into(),
            2.0 * inst.delta,
            STOCK,
        )),
        Config::Owner => v.push(rate("adjust.invest.good".into(), 0.0, STOCK)),
    }
    v.push(rate(format!("adjust.invest.{maker}"), 0.0, STOCK));
    if !inst.is_flow() {
        v.push(Dial {
            key: format!("cover.{maker}"),
            value: 4.0 / 52.0,
            unit: "Years",
            basis: STOCK.into(),
        });
    }
    Ok(Dials {
        set: name.into(),
        values: v,
    })
}

/// A displacement of genesis from the oracle's point, by factors, each in its list's order.
#[derive(Debug, Clone, PartialEq)]
pub struct Displacement {
    /// Each posted price, in [`Instance::markets`] order.
    pub price: Vec<f64>,
    /// The good desk's human share.
    pub share: ShareAt,
    /// Each actor's coin, in [`Instance::actors`] order.
    pub coin: Vec<f64>,
    /// Each stock, in [`stocks`] order.
    pub stock: Vec<f64>,
}

/// The stocks a run can displace, by the name the run grammar gives them (HORSES-SPEC §7.7):
///
/// - `stock.good`: the good desk's good, one tick's output;
/// - `heads.capacity`, `hours.capacity`: the capacity desk's heads, and the horse-days it holds
///   to sell (wet);
/// - `heads.good`: the owner desk's heads (M1);
/// - `own.<maker>`, `finished.<maker>`: the maker's serving stock after wear (its record and its
///   holding together) and its finished heads;
/// - `stock.<maker>`: on the flow path, the maker's stock of its output (I0's `stock.mach`);
/// - `stock.fodder`: the fodder desk's fodder, one tick's output.
pub fn stocks(inst: &Instance) -> Vec<String> {
    let maker = &inst.keys.maker;
    let mut v = vec!["stock.good".to_string()];
    if inst.is_flow() {
        v.push(format!("stock.{maker}"));
        return v;
    }
    match inst.config {
        Config::Wet => {
            v.push("heads.capacity".into());
            v.push("hours.capacity".into());
        }
        Config::Owner => v.push("heads.good".into()),
    }
    v.push(format!("own.{maker}"));
    v.push(format!("finished.{maker}"));
    if inst.has_fodder() {
        v.push("stock.fodder".into());
    }
    v
}

impl Displacement {
    /// No displacement: mode A.
    pub fn none(inst: &Instance) -> Displacement {
        Displacement {
            price: vec![1.0; inst.markets().len()],
            share: ShareAt::Times(1.0),
            coin: vec![1.0; inst.actors().len()],
            stock: vec![1.0; stocks(inst).len()],
        }
    }
}

/// A dated change of the county's machine land b, by `SetParam`s at the start of `tick`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shock {
    /// The tick it fires in.
    pub tick: u64,
    /// b from then on.
    pub b: f64,
}

/// Everything a stocks-probe tape is made from.
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
    /// The good desk's task assignment (HORSES-SPEC §2.4: `ExPost`, D-G6; R1a registers
    /// `Planned`, as P2.1's I0 is, with `ExPost` as appb's registered alternative).
    pub assign: Assign,
    /// The capacity desk's order rule (`Target`; `Held`, the negative control).
    pub order: OrderRule,
    /// Whether the maker holds its cover (`true`, registered; `false`, the control that offers
    /// every finished head).
    pub cover: bool,
    /// The maker's reservation ψ (IDLE-SPEC, L0.4): `Some` writes the param `reserve.<maker>`
    /// and the maker's `reserve` field; `None`, registered, writes neither, so the tape is
    /// P2.2a's byte for byte.
    pub reserve: Option<f64>,
    /// The displacement of genesis.
    pub displace: Displacement,
    /// The county's machine land the tape registers from tick 0 (a cost shock at genesis);
    /// genesis stays at the registered point.
    pub b_genesis: Option<f64>,
    /// Dated changes of b, in tick order.
    pub shocks: Vec<Shock>,
}

impl Setup {
    /// The registered setup of an instance at `tpy` ticks a year: its dials, Saturate, the
    /// registered variants, genesis at the oracle's point (mode A).
    pub fn registered(id: &str, tpy: u32) -> Result<Setup, String> {
        Setup::of(Instance::named(id)?, tpy)
    }

    /// The setup of `instance` at `tpy` ticks a year as [`Setup::registered`] makes it: its
    /// dials, Saturate, the registered variants, mode A. A demo instance (D2.4) comes from its
    /// county table, not from a registered id.
    pub fn of(instance: Instance, tpy: u32) -> Result<Setup, String> {
        let dials = dials(&instance, &instance.dials)?;
        Ok(Setup {
            tpy,
            dials,
            one_sided: OneSided::Saturate,
            assign: if instance.is_flow() {
                Assign::Planned
            } else {
                Assign::ExPost
            },
            order: OrderRule::Target,
            cover: true,
            reserve: None,
            displace: Displacement::none(&instance),
            b_genesis: None,
            shocks: Vec::new(),
            instance,
        })
    }

    /// The county's machine land in force at `tick`.
    pub fn b_at(&self, tick: u64) -> f64 {
        self.shocks
            .iter()
            .rfind(|s| s.tick <= tick)
            .map(|s| s.b)
            .or(self.b_genesis)
            .unwrap_or(self.instance.county.b)
    }

    /// The instance in force at `tick`.
    pub fn instance_at(&self, tick: u64) -> Instance {
        self.instance.with_b(self.b_at(tick))
    }

    /// The instance the tape registers from tick 0.
    pub fn tape_instance(&self) -> Instance {
        self.instance
            .with_b(self.b_genesis.unwrap_or(self.instance.county.b))
    }

    /// The cover in ticks the maker reads at genesis: the registered dial through the clock,
    /// or 0 where it holds none.
    pub fn cover_ticks(&self) -> Result<f64, String> {
        if !self.cover || self.instance.is_flow() {
            return Ok(0.0);
        }
        let years = self
            .dials
            .get(&format!("cover.{}", self.instance.keys.maker))?;
        let t = clock(self.tpy)?
            .ticks(Years(years))
            .map_err(|e| e.to_string())?;
        Ok(f64::from(t))
    }

    /// The reservation ψ the maker reads: its param, or 0 (off) where it has none.
    pub fn psi(&self) -> f64 {
        self.reserve.unwrap_or(0.0)
    }
}

/// The genesis a setup writes: the undisplaced oracle point and the displaced values.
#[derive(Debug, Clone, PartialEq)]
pub struct Genesis {
    /// The oracle's point at the registered instance.
    pub point: Point,
    /// The genesis prices, in market order.
    pub prices: Vec<f64>,
    /// The good desk's genesis human share.
    pub share: f64,
    /// The stationary coins at the point, before displacement, in actor order.
    pub stationary: Vec<f64>,
    /// The genesis coins, in actor order.
    pub coin: Vec<f64>,
    /// Each stock at the point, before displacement, in [`stocks`] order.
    pub rest: Vec<f64>,
    /// Each stock at genesis, in [`stocks`] order.
    pub stock: Vec<f64>,
    /// The maker's cost of a head at the point, as its rule sums it.
    pub maker_cost: f64,
    /// The maker's cover of finished heads at the point.
    pub band: f64,
}

impl Genesis {
    /// A stock's genesis value by its name.
    pub fn stock_of(&self, inst: &Instance, name: &str) -> Result<f64, String> {
        stocks(inst)
            .iter()
            .position(|s| s == name)
            .map(|i| self.stock[i])
            .ok_or_else(|| format!("{}: no stock {name}", inst.id))
    }
}

/// The flow path's genesis: the markets probe's I0 genesis on the instance's county, bit for bit
/// P2.1's (and so appb's) for Appendix B.
fn flow_genesis(s: &Setup) -> Result<Genesis, String> {
    let inst = &s.instance;
    let m = crate::markets::setup::Setup {
        tpy: s.tpy,
        instance: inst.flow()?,
        dials: s.dials.clone(),
        one_sided: s.one_sided,
        displace: crate::markets::setup::Displacement {
            price: s.displace.price.clone(),
            share: vec![s.displace.share],
            coin: s.displace.coin.clone(),
            stock: s.displace.stock.clone(),
        },
        at_genesis: Vec::new(),
        shocks: Vec::new(),
    };
    let g = crate::markets::setup::genesis(&m)?;
    let point = inst.point(s.tpy)?;
    let rest = vec![g.point.cat_output[0], g.point.type_services[0]];
    Ok(Genesis {
        point,
        prices: g.prices,
        share: g.shares[0],
        stationary: g.stationary,
        coin: g.coin,
        rest,
        stock: g.stock,
        maker_cost: f64::NAN,
        band: 0.0,
    })
}

/// Genesis for a setup (HORSES-SPEC §5.3): the oracle's point with each desk's stock at its rest
/// value and each actor's stationary coin, summed in the order its rule sums.
pub fn genesis(s: &Setup) -> Result<Genesis, String> {
    if s.instance.is_flow() {
        return flow_genesis(s);
    }
    let inst = &s.instance;
    let e = inst.point(s.tpy)?;
    let c = clock(s.tpy)?;
    let share = |v: f64| c.share(RatePerYear(v));
    let d = &s.dials;
    let (kappa, delta) = inst.per_tick(s.tpy)?;
    let rule = inst.rule_a(s.tpy)?;
    let n = c.flow(FlowPerYear(inst.county.workers));
    let t = c.flow(FlowPerYear(inst.county.land));
    let (w, r) = (e.v, 1.0);
    // P_s as the households sum it: (0.0 + 1·p) + h·r.
    let mut ps = 0.0;
    ps += 1.0 * e.p;
    ps += inst.county.space * r;
    let tau = n * ps;
    let hours = n * (num::ln1p(w / ps) / inst.county.chi_max).min(1.0);
    let buffer = |desk: &str| d.get(&format!("buffer.desk.{desk}.cash"));
    let maker = &inst.keys.maker;
    // The maker's cost of a head, as its rule sums it: its goods (a·run_g + build_g) first,
    // then labour (a·run_lab + build_lab), then land.
    let a = rule.own_hours;
    let mut cm = 0.0;
    if inst.has_fodder() {
        cm += (a * 1.0 + 0.0) * e.pf;
    }
    cm += (a * 0.0 + rule.labour) * w;
    cm += rule.pasture * r;
    let band = s.cover_ticks()? * e.made * cm / e.pk;
    let own = (1.0 - delta) * e.serving;
    let finished = e.made + band;
    let mut stationary = Vec::new();
    let mut rest = vec![e.good];
    // The good desk: p·Y/share(turnover).
    stationary.push(e.p * e.good / share(buffer("good")?));
    match inst.config {
        Config::Wet => {
            // The capacity desk: p_h·κ·H/share(turnover), holding H heads and κ·H horse-days.
            stationary.push(e.ph * kappa * e.capacity / share(buffer("capacity")?));
            rest.push(e.capacity);
            rest.push(kappa * e.capacity);
        }
        // The owner desk holds its heads after this tick's wear.
        Config::Owner => rest.push((1.0 - delta) * e.capacity),
    }
    stationary.push(cm * e.made / share(buffer(maker)?));
    rest.push(own);
    rest.push(finished);
    if inst.has_fodder() {
        // The fodder desk: its cost as the type desk sums it, ((0.0 + λ_f·w) + b_f·r).
        let mut cf = 0.0;
        cf += 0.0 * w;
        cf += rule.fodder_land * r;
        stationary.push(cf * e.qf / share(buffer("fodder")?));
        rest.push(e.qf);
    }
    stationary.push(tau + (r * t - tau) / share(d.get("spend.provider")?));
    stationary.push((tau + w * hours) / share(d.get("spend.workers")?));
    let x = &s.displace;
    let mut prices = vec![w, r];
    if inst.has_fodder() {
        prices.push(e.pf);
    }
    prices.push(e.pk);
    if inst.config == Config::Wet {
        prices.push(e.ph);
    }
    prices.push(e.p);
    if x.price.len() != prices.len()
        || x.coin.len() != stationary.len()
        || x.stock.len() != rest.len()
    {
        return Err("the displacement does not fit the instance".into());
    }
    for (p, f) in prices.iter_mut().zip(&x.price) {
        *p *= f;
    }
    let one_minus_x = match x.share {
        ShareAt::Times(f) => e.one_minus_x * f,
        ShareAt::At(xx) => 1.0 - xx,
    };
    if !(0.0..=1.0).contains(&one_minus_x) {
        return Err(format!(
            "the genesis human share {one_minus_x} is outside [0, 1]"
        ));
    }
    let coin = stationary.iter().zip(&x.coin).map(|(c, f)| c * f).collect();
    let stock = rest.iter().zip(&x.stock).map(|(q, f)| q * f).collect();
    Ok(Genesis {
        point: e,
        prices,
        share: one_minus_x,
        stationary,
        coin,
        rest,
        stock,
        maker_cost: cm,
        band,
    })
}

fn f(x: f64) -> String {
    format!("{x:?}")
}

const ROLE: &str =
    "Assumed(\"HORSES-SPEC §2: the probe's rules with the horse held as a stock (docs/probe/HORSES-RULES.md)\")";
const ROLE_P21: &str =
    "Assumed(\"MARKETS-SPEC §2: the probe's rules carried to many markets (docs/probe/MARKETS-RULES.md)\")";
const ROLE_P20: &str =
    "Assumed(\"design-analytic-first with the judges' grafts (docs/probe/RULES.md)\")";

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

/// The params an instance registers at `tpy`, with the county's machine land at `b`: (key,
/// value, unit, basis), in the tape's order (HORSES-SPEC §2.10). Only params some rule reads are
/// listed.
pub fn params(
    inst: &Instance,
    tpy: u32,
) -> Result<Vec<(String, f64, &'static str, String)>, String> {
    let n = inst.county;
    let county = format!("Literature(\"{}\")", inst.county_basis);
    let rule = format!("Assumed(\"{RULE_A}\")");
    let rule_a = inst.rule_a(tpy)?;
    let h = &inst.keys.horse;
    let mut out = vec![
        (
            "inst.workers".to_string(),
            n.workers,
            "FlowPerYear",
            county.clone(),
        ),
        (
            "inst.land".to_string(),
            n.land,
            "FlowPerYear",
            county.clone(),
        ),
        (
            "inst.chi_max".to_string(),
            n.chi_max,
            "Dimensionless",
            county.clone(),
        ),
        (
            "inst.eta".to_string(),
            n.eta,
            "Dimensionless",
            county.clone(),
        ),
        ("inst.g0".to_string(), n.g0, "Dimensionless", county.clone()),
        ("inst.g1".to_string(), n.g1, "Dimensionless", county.clone()),
        ("inst.k".to_string(), n.k, "Dimensionless", county.clone()),
        (
            "inst.good.weight".to_string(),
            1.0,
            "Dimensionless",
            county.clone(),
        ),
        (
            "inst.space.weight".to_string(),
            n.space,
            "Dimensionless",
            county.clone(),
        ),
    ];
    if inst.has_fodder() {
        out.push(("inst.fodder.own".into(), 0.0, "Dimensionless", rule.clone()));
        out.push((
            "inst.fodder.labour".into(),
            0.0,
            "Dimensionless",
            rule.clone(),
        ));
        out.push((
            "inst.fodder.land".into(),
            rule_a.fodder_land,
            "Dimensionless",
            rule.clone(),
        ));
        out.push((
            format!("inst.{h}.run.fodder"),
            1.0,
            "Dimensionless",
            rule.clone(),
        ));
    }
    out.push((
        format!("inst.{h}.run.labour"),
        0.0,
        "Dimensionless",
        rule.clone(),
    ));
    // The horse: its hours a year and wear a year, and a head's build per tick at this tick
    // length. On the flow path the build is the flow machine's (a, λ, b), I0's.
    let flow_basis = if inst.is_flow() {
        county.clone()
    } else {
        rule.clone()
    };
    out.push((
        format!("inst.{h}.kappa"),
        inst.kappa,
        "FlowPerYear",
        rule.clone(),
    ));
    out.push((
        format!("inst.{h}.delta"),
        inst.delta,
        "FractionPerYear",
        rule.clone(),
    ));
    out.push((
        format!("inst.{h}.own_hours"),
        rule_a.own_hours,
        "Dimensionless",
        flow_basis.clone(),
    ));
    out.push((
        format!("inst.{h}.labour"),
        rule_a.labour,
        "Dimensionless",
        flow_basis.clone(),
    ));
    out.push((
        format!("inst.{h}.land"),
        rule_a.pasture,
        "Dimensionless",
        flow_basis,
    ));
    Ok(out)
}

/// The params a change of b moves: fodder's land (where it is traded) and the horse's build
/// land, each with its value at `b`.
pub fn b_params(inst: &Instance, b: f64, tpy: u32) -> Result<Vec<(String, f64)>, String> {
    let r = inst.with_b(b).rule_a(tpy)?;
    let mut v = Vec::new();
    if inst.has_fodder() {
        v.push(("inst.fodder.land".to_string(), r.fodder_land));
    }
    v.push((format!("inst.{}.land", inst.keys.horse), r.pasture));
    Ok(v)
}

fn list(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| f(*x)).collect();
    format!("[{}]", parts.join(", "))
}

/// The tape of a setup, as RON text with its derivation in the header.
pub fn tape_ron(s: &Setup) -> Result<String, String> {
    let g = genesis(s)?;
    let e = &g.point;
    let c = clock(s.tpy)?;
    let base = &s.instance;
    let inst = s.tape_instance();
    let h = inst.keys.horse.clone();
    let maker = inst.keys.maker.clone();
    let (kappa, delta) = inst.per_tick(s.tpy)?;
    let n = c.flow(FlowPerYear(inst.county.workers));
    let t = c.flow(FlowPerYear(inst.county.land));
    let markets = inst.markets();
    let mut out = Lines(Vec::new());
    let o = &mut out;
    let name = format!("horses-{}", inst.id);
    for line in [
        format!(
            "// The stocks probe's {} world (HORSES-SPEC §1; docs/probe/HORSES-RULES.md): {},",
            inst.id.to_uppercase(),
            inst.title
        ),
        "// for P2.2a (2026-09-28).".into(),
        "//".into(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin horses-tape -- --inst".into(),
        format!(
            "// {} <path>` writes it, and the test `horses_tapes_are_their_generators_output` checks",
            inst.id
        ),
        "// this file against the generator.".into(),
        "//".into(),
    ] {
        o.line(line);
    }
    if inst.is_flow() {
        for line in [
            "// The stocks layer off (R1a, HORSES-SPEC §1.6): the horse lives one tick and wears all of"
                .to_string(),
            "// it (δ = 1 a tick), so the owner desk runs P2.0's good desk and the maker P2.1's type desk;"
                .into(),
            "// genesis is P2.1's I0's, the oracle's point of unit 1c in operating form (unit 1a's G1),".into(),
            format!(
                "//   x* = {}, v = {}, P_s = {}, Y = {}, N_a = {}, p_m = {}, K = {},",
                f(e.x_star),
                f(e.v),
                f(e.p_s),
                f(e.y),
                f(e.n_a),
                f(e.pk),
                f(e.made)
            ),
            format!(
                "// with each actor's stationary coin {} in actor order ({}).",
                list(&g.stationary),
                inst.actors().join(", ")
            ),
        ] {
            o.line(line);
        }
    } else {
        for line in [
            "// Genesis is the oracle's equilibrium (HORSES-SPEC §1.4, §5.3, mode A): crates/oracle unit 1g's"
                .to_string(),
            format!(
                "// ChainEconomy (the horse's hours at J = 2, decision 232) at N = {} and T = {} a tick, {} ticks",
                f(c.flow(FlowPerYear(base.county.workers))),
                f(c.flow(FlowPerYear(base.county.land))),
                s.tpy
            ),
            format!(
                "// a year, δ = {} and κ = {} a tick, gives x* = {}, v = w/r = {},",
                f(delta),
                f(kappa),
                f(e.x_star),
                f(e.v)
            ),
            format!(
                "//   P_s/r = {}, Y = {}, N_a = {}, the good {}, fodder {}, a head {}, a horse-day {}",
                f(e.p_s),
                f(e.y),
                f(e.n_a),
                f(e.p),
                f(e.pf),
                f(e.pk),
                f(e.ph)
            ),
            format!(
                "//   (running cost {}); heads made {} a tick, the tasks' heads {}, the maker's {}; fodder {}.",
                f(e.o),
                f(e.made),
                f(e.capacity),
                f(e.serving),
                f(e.qf)
            ),
            "// The genesis prices are those ratios with r = 1 coin; the good desk's human share is 1 - x*;".into(),
            "// each desk holds its stock at rest: one tick's output of the good and of fodder, the tasks'".into(),
            "// heads and their horse-days (wet) or those heads after wear (M1), and the maker its serving".into(),
            format!(
                "// stock after wear (its state) and q_b plus its cover of {} ticks of sales, {} heads. Each coin",
                f(s.cover_ticks()?),
                f(g.band)
            ),
            format!(
                "// is the actor's stationary balance under the dials, share(v) = 1 - exp(-v/{}), HORSES-SPEC",
                s.tpy
            ),
            format!(
                "// §5.3, giving {} in actor order ({}).",
                list(&g.stationary),
                inst.actors().join(", ")
            ),
        ] {
            o.line(line);
        }
    }
    for line in [
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .to_string(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ] {
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
    o.line(
        "        // The instance (HORSES-SPEC §1.2, §2.10): the county, rule A's fodder and horse.",
    );
    for (key, value, unit, basis) in params(&inst, s.tpy)? {
        o.param(&key, value, unit, &basis);
    }
    o.line(
        "        // Structure: fodder, horse-days and the good last one tick (HORSES-SPEC §2.1).",
    );
    o.param(
        "life.one_tick",
        1.0 / f64::from(s.tpy),
        "Years",
        "Assumed(\"one tick: J = 1 (PROBE-SPEC §1.2)\")",
    );
    o.line(format!(
        "        // The dials ({}; HORSES-SPEC §5) and the stock rules' (§5.2).",
        s.dials.set
    ));
    for d in &s.dials.values {
        if !s.cover && d.key == format!("cover.{maker}") {
            continue;
        }
        o.param(&d.key, d.value, d.unit, &d.basis);
    }
    if let Some(psi) = s.reserve {
        if inst.is_flow() {
            return Err("the flow path holds no reservation (--reserve)".into());
        }
        o.param(&format!("reserve.{maker}"), psi, "Dimensionless", RESERVE);
    }
    if !s.shocks.is_empty() {
        o.line("        // The dated shocks' values: schedule params (HORSES-SPEC §7.7).");
        for (k, sh) in s.shocks.iter().enumerate() {
            for (p, v) in b_params(base, sh.b, s.tpy)? {
                o.param(
                    &format!("{p}.shock.{}", k + 1),
                    v,
                    "Dimensionless",
                    "Assumed(\"HORSES-SPEC §1.5, §7.7: a dated cost shock of b\")",
                );
            }
        }
    }
    o.line("    ],");
    o.line("    goods: [");
    o.line("        (key: \"coin\", life: Indefinite, price_rate: None),");
    o.line("        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),");
    o.line("        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),");
    for m in &markets[2..] {
        let life = if *m == h && !inst.is_flow() {
            "Indefinite".to_string()
        } else {
            "Years(\"life.one_tick\")".to_string()
        };
        o.line(format!(
            "        (key: \"{m}\", life: {life}, price_rate: Some(\"rate.{m}\")),"
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
    classes.push("\"owners\"".into());
    classes.push("\"workers\"".into());
    o.line(format!("    classes: [{}],", classes.join(", ")));
    o.line("    actors: [");
    let scale = |desk: &str| {
        format!(
            "            scale: Cash((turnover: \"buffer.desk.{desk}.cash\", tilt: \
             \"tilt.desk.{desk}\", payout: None)),"
        )
    };
    let assign = match s.assign {
        Assign::Planned => "Planned",
        Assign::ExPost => "ExPost",
    };
    let run_goods = if inst.has_fodder() {
        format!("[(good: \"fodder\", coef: \"inst.{h}.run.fodder\")]")
    } else {
        "[]".to_string()
    };
    let schedule =
        "            schedule: (eta: \"inst.eta\", g0: \"inst.g0\", g1: \"inst.g1\", k: \"inst.k\"),";
    // The good desk.
    match inst.config {
        Config::Wet => {
            o.line(format!(
                "        (key: \"desk.good\", kind: Desk, class: \"good_desks\", home: \"home\", basis: {ROLE_P20},"
            ));
            o.line("         spec: GoodDesk((");
            o.line(format!(
                "            output: \"good\", labour: \"labour\", mach: \"{HOURS}\","
            ));
            o.line(schedule);
            o.line(format!(
                "            technique: (adjust: \"adjust.technique.good\", share: {}),",
                f(g.share)
            ));
            o.line(format!("            assign: {assign},"));
        }
        Config::Owner => {
            o.line(format!(
                "        (key: \"desk.good\", kind: Desk, class: \"good_desks\", home: \"home\", basis: {ROLE},"
            ));
            o.line("         spec: OwnerDesk((");
            o.line(format!(
                "            output: \"good\", labour: \"labour\", stock: \"{h}\","
            ));
            o.line(schedule);
            o.line(format!(
                "            technique: (adjust: \"adjust.technique.good\", share: {}),",
                f(g.share)
            ));
            o.line(format!("            assign: {assign},"));
            o.line(format!(
                "            kappa: \"inst.{h}.kappa\", running: {run_goods}, delta: \"inst.{h}.delta\","
            ));
            o.line("            adjust: \"adjust.invest.good\",");
        }
    }
    o.line(scale("good"));
    o.line("         ))),");
    let running = format!("(goods: {run_goods}, labour: \"inst.{h}.run.labour\")");
    // The capacity desk.
    if inst.config == Config::Wet {
        let order = match s.order {
            OrderRule::Target => "Target",
            OrderRule::Held => "Held",
        };
        o.line(format!(
            "        (key: \"desk.capacity\", kind: Desk, class: \"capacity_desks\", home: \"home\", basis: {ROLE},"
        ));
        o.line("         spec: CapacityDesk((");
        o.line(format!(
            "            stock: \"{h}\", hours: \"{HOURS}\", labour: \"labour\", kappa: \"inst.{h}.kappa\","
        ));
        o.line(format!("            running: {running},"));
        o.line(format!(
            "            delta: \"inst.{h}.delta\", adjust: \"adjust.invest.capacity\", order: {order},"
        ));
        o.line(scale("capacity"));
        o.line("         ))),");
    }
    // The maker.
    let cover = if s.cover && !inst.is_flow() {
        format!("Some(\"cover.{maker}\")")
    } else {
        "None".to_string()
    };
    let own = stocks(&inst)
        .iter()
        .position(|x| *x == format!("own.{maker}"))
        .map_or(0.0, |i| g.stock[i]);
    o.line(format!(
        "        (key: \"desk.{maker}\", kind: Desk, class: \"{maker}_desks\", home: \"home\", basis: {ROLE},"
    ));
    o.line("         spec: Maker((");
    o.line(format!(
        "            output: \"{h}\", labour: \"labour\", land: \"land\", own_hours: \"inst.{h}.own_hours\","
    ));
    o.line(format!(
        "            kappa: \"inst.{h}.kappa\", running: {running},"
    ));
    o.line(format!(
        "            build: (goods: [], labour: \"inst.{h}.labour\", land: \"inst.{h}.land\"),"
    ));
    let reserve = match s.reserve {
        Some(_) => format!(" reserve: Some(\"reserve.{maker}\"),"),
        None => String::new(),
    };
    o.line(format!(
        "            delta: \"inst.{h}.delta\", adjust: \"adjust.invest.{maker}\", cover: {cover},{reserve} own: {},",
        f(own)
    ));
    o.line(scale(&maker));
    o.line("         ))),");
    // The fodder desk.
    if inst.has_fodder() {
        o.line(format!(
            "        (key: \"desk.fodder\", kind: Desk, class: \"fodder_desks\", home: \"home\", basis: {ROLE_P21},"
        ));
        o.line("         spec: TypeDesk((");
        o.line("            output: \"fodder\", labour: \"labour\", land: \"land\",");
        o.line(
            "            recipe: (own: \"inst.fodder.own\", inputs: [], labour: \"inst.fodder.labour\", \
             land: \"inst.fodder.land\"),",
        );
        o.line(scale("fodder"));
        o.line("         ))),");
    }
    let basket = "            basket: [(good: \"good\", weight: \"inst.good.weight\"), (good: \"land\", weight: \"inst.space.weight\")],";
    for line in [
        format!(
            "        (key: \"provider\", kind: Pop, class: \"owners\", home: \"home\", basis: {ROLE_P21},"
        ),
        "         spec: BasketProvider((".into(),
        "            land: \"land\", endowment: \"inst.land\",".into(),
        "            transfer: (to: \"workers\", heads: \"inst.workers\"),".into(),
        basket.into(),
        "            spend: \"spend.provider\",".into(),
        "         ))),".into(),
        format!(
            "        (key: \"workers\", kind: Pop, class: \"workers\", home: \"home\", basis: {ROLE_P21},"
        ),
        "         spec: BasketWorkers((".into(),
        "            labour: \"labour\", heads: \"inst.workers\", chi_max: \"inst.chi_max\",".into(),
        basket.into(),
        "            spend: \"spend.workers\",".into(),
        "         ))),".into(),
        "    ],".into(),
        "    genesis: (".into(),
        format!(
            "        basis: Approximate(\"oracle unit 1g (crates/oracle) at HORSES-SPEC's {}, N = {} and \
             T = {} per tick; stationary coins and stocks per HORSES-SPEC §5.3; written by \
             rustyecon-probe's horses-tape\"),",
            inst.id.to_uppercase(),
            f(n),
            f(t)
        ),
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
    let actors = inst.actors();
    let stock = |name: &str| g.stock_of(&inst, name);
    for (i, a) in actors.iter().enumerate() {
        let mut goods = vec![format!("(\"coin\", {})", f(g.coin[i]))];
        match a.as_str() {
            "desk.good" => {
                goods.push(format!("(\"good\", {})", f(stock("stock.good")?)));
                if inst.config == Config::Owner && !inst.is_flow() {
                    goods.push(format!("(\"{h}\", {})", f(stock("heads.good")?)));
                }
            }
            "desk.capacity" => {
                goods.push(format!("(\"{h}\", {})", f(stock("heads.capacity")?)));
                goods.push(format!("(\"{HOURS}\", {})", f(stock("hours.capacity")?)));
            }
            "desk.fodder" => goods.push(format!("(\"fodder\", {})", f(stock("stock.fodder")?))),
            "provider" | "workers" => {}
            _ => {
                let held = if inst.is_flow() {
                    stock(&format!("stock.{maker}"))?
                } else {
                    stock(&format!("own.{maker}"))? + stock(&format!("finished.{maker}"))?
                };
                goods.push(format!("(\"{h}\", {})", f(held)));
            }
        }
        o.line(format!(
            "            (holder: \"{a}\", goods: [{}]),",
            goods.join(", ")
        ));
    }
    o.line("        ],");
    o.line("    ),");
    o.line("    events: [");
    for (k, sh) in s.shocks.iter().enumerate() {
        let date = c
            .date_of(sh.tick)
            .ok_or_else(|| format!("tick {} has no date", sh.tick))?;
        for (j, (p, _)) in b_params(base, sh.b, s.tpy)?.iter().enumerate() {
            o.line(format!(
                "        (key: \"shock.{}.{}\", at: \"{date}\", basis: Assumed(\"HORSES-SPEC §7.7: a \
                 dated cost shock of b at tick {}\"),",
                k + 1,
                j + 1,
                sh.tick
            ));
            o.line(format!(
                "         act: SetParam(param: \"{p}\", to: \"{p}.shock.{}\")),",
                k + 1
            ));
        }
    }
    o.line("    ],");
    o.line("    recurring: [],");
    o.line(")");
    let mut text = out.0.join("\n");
    text.push('\n');
    Ok(text)
}
