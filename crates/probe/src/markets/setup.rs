//! The markets probe's tapes and their generator (MARKETS-SPEC §2, §5; docs/probe/MARKETS-RULES.md).
//!
//! A [`Setup`] names what a run is made from: the instance, the tick length, the dials, the
//! one-sided rule, the displacement of genesis, any coefficients changed from tick 0, and any
//! dated shocks. [`tape_ron`] writes it as a tape of the four many-market kinds. The oracle's
//! equilibrium (unit 1c, solved here, outside any `Sim`) seeds genesis: its prices relative to
//! r = 1 coin, 1 − x\* for every category desk, one tick's output held by each desk, and each
//! actor's stationary coin under the dials (MARKETS-SPEC §5.4). No agent reads the oracle
//! (R13). `tapes/markets-<id>.ron` is `tape_ron(&Setup::registered(id, 52))`, checked by a test.

use super::instance::{Instance, Point, COMMONS_BASIS, WALL_BASIS};
use crate::setup::{clock, OneSided, START};
use rustyecon_core::{num, FlowPerYear, RatePerYear};

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
        rate("rate.land".into(), 1.3);
        for t in &inst.types {
            rate(format!("rate.{}", t.key), 1.3);
        }
        for c in &inst.categories {
            rate(format!("rate.{}", c.key), 2.6);
        }
        for c in &inst.categories {
            rate(format!("adjust.technique.{}", c.key), 2.6);
        }
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
    /// tilt to it (as P2.0's `--set` does).
    pub fn set(&mut self, key: &str, value: f64) -> Result<(), String> {
        let family = |prefix: &str, d: &Dial| d.key.starts_with(prefix);
        match key {
            "rate.*" | "buffer.*" | "adjust.*" => {
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
}

impl Displacement {
    /// No displacement: mode A.
    pub fn none(inst: &Instance) -> Displacement {
        Displacement {
            price: vec![1.0; inst.markets().len()],
            share: vec![ShareAt::Times(1.0); inst.categories.len()],
            coin: vec![1.0; inst.actors().len()],
            stock: vec![1.0; inst.desks().len()],
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
pub fn genesis(s: &Setup) -> Result<Genesis, String> {
    let inst = &s.instance;
    let e = inst.point(s.tpy)?;
    let c = clock(s.tpy)?;
    let share = |v: f64| c.share(RatePerYear(v));
    let d = &s.dials;
    let n = c.flow(FlowPerYear(inst.workers));
    let t = c.flow(FlowPerYear(inst.land));
    let (w, r) = (e.v, 1.0);
    let ps = basket_price(inst, &e.cat_price, r);
    let tau_w = n * ps;
    let hours = if inst.worker_form || inst.exit.is_some() {
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
    stationary.push(tau + (r * t - tau) / share(d.get("spend.provider")?));
    match e.commons.as_ref().map(|c| c.rented) {
        // The plots' rent first, from the coin; the baskets from the rest.
        Some(tp) if tp > 0.0 => {
            let rent = r * tp;
            stationary.push(rent + (tau_w + w * hours - rent) / share(d.get("spend.workers")?));
        }
        _ => stationary.push((tau_w + w * hours) / share(d.get("spend.workers")?)),
    }
    stationary.extend(reserved_pops);
    let x = &s.displace;
    let mut prices = vec![w, r];
    prices.extend(e.type_price.iter().copied());
    prices.extend(e.cat_price.iter().copied());
    prices.extend(e.wage.iter().copied());
    if x.price.len() != prices.len()
        || x.share.len() != inst.categories.len()
        || x.coin.len() != stationary.len()
        || x.stock.len() != inst.desks().len()
    {
        return Err("the displacement does not fit the instance".into());
    }
    for (p, f) in prices.iter_mut().zip(&x.price) {
        *p *= f;
    }
    let mut shares = Vec::new();
    for sh in &x.share {
        let v = match *sh {
            // At the commons a share scaled past 1 is 1, as the frame's mirror displaces it
            // (`battery_c.displace`; the registration's §3): only the basin's s[food] runs reach
            // it. Elsewhere a share above 1 is refused below.
            ShareAt::Times(f) if inst.exit.is_some() => (e.one_minus_x * f).min(1.0),
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
    Ok(Genesis {
        point: e,
        prices,
        shares,
        stationary,
        coin,
        stock,
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
    Ok(vec![
        format!(
            "// The wall world {} (docs/probe/wall/SPEC.md; docs/probe/WALL-RULES.md):",
            inst.id.to_uppercase(),
        ),
        format!("// {},", inst.title),
        "// run by the four many-market roles with the wall's optional fields (a category desk's"
            .into(),
        "// tail and reserved hours, the provider's further transfers), for P2.3 (2026-09-30)."
            .into(),
        "//".into(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin markets-tape -- --inst"
            .into(),
        format!(
            "// {} <path>` writes it, and the test `markets_iw1_tape_is_its_generators_output` checks",
            inst.id
        ),
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
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .into(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ])
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
    Ok(vec![
        format!(
            "// The open-commons world {} (docs/probe/commons/SPEC.md; docs/probe/COMMONS-RULES.md):",
            inst.id.to_uppercase(),
        ),
        format!("// {},", inst.title),
        "// run by the four many-market roles, the workers with the priced exit in food and the"
            .into(),
        "// commons they hold (their optional `exit`), for P2.3 (2026-09-30).".into(),
        "//".into(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin markets-tape -- --inst"
            .into(),
        format!(
            "// {} <path>` writes it, and the test `commons_tapes_are_their_generators_output` checks",
            inst.id
        ),
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
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .into(),
        "// the harness (crates/probe), outside the Sim.".into(),
    ])
}

fn list_s(v: &[String]) -> String {
    format!("[{}]", v.join(", "))
}

/// A tape's params, goods, nodes, classes, actors, genesis and events.
fn body(s: &Setup, g: &Genesis, inst: &Instance, o: &mut Lines) -> Result<(), String> {
    let c = clock(s.tpy)?;
    let tau = inst.task_type()?;
    let n = c.flow(FlowPerYear(inst.workers));
    let t = c.flow(FlowPerYear(inst.land));
    let markets = inst.markets();
    let role = if inst.worker_form {
        WALL_ROLE
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
    } else if inst.exit.is_some() {
        o.line("        // The instance (MARKETS-SPEC §1.2, §2.9; the commons frame's §2.1).");
    } else {
        o.line("        // The instance (MARKETS-SPEC §1.2, §2.9).");
    }
    for (key, value, unit, basis) in inst.params()? {
        // The wall frame's own numbers are its calibration, assumed; the rest are literature.
        let kind = if basis == WALL_BASIS || basis == COMMONS_BASIS {
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
    o.line("        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),");
    let produced = 2 + inst.types.len() + inst.categories.len();
    for m in &markets[2..produced] {
        o.line(format!(
            "        (key: \"{m}\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.{m}\")),"
        ));
    }
    // Each reserved type's hours, Instant as the pool's are (the wall frame's §3.5).
    for m in &markets[produced..] {
        o.line(format!(
            "        (key: \"{m}\", life: Instant, price_rate: Some(\"rate.{m}\")),"
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
    for line in [
        format!(
            "        (key: \"provider\", kind: Pop, class: \"owners\", home: \"home\", basis: {role},"
        ),
        "         spec: BasketProvider((".into(),
        "            land: \"land\", endowment: \"inst.land\",".into(),
        "            transfer: (to: \"workers\", heads: \"inst.workers\"),".into(),
        basket.clone(),
        "            spend: \"spend.provider\",".into(),
    ] {
        o.line(line);
    }
    // The further transfers, one per reserved pop in order (P2.3), written only if any.
    let more: Vec<String> = inst
        .wtypes
        .iter()
        .map(|w| format!("(to: \"{}\", heads: \"inst.{}.workers\")", w.pop(), w.key))
        .collect();
    if !more.is_empty() {
        o.line(format!("            more: [{}],", more.join(", ")));
    }
    o.line("         ))),");
    let mut pops = vec![(
        "workers".to_string(),
        "workers".to_string(),
        "labour".to_string(),
        "inst.workers".to_string(),
        "inst.chi_max".to_string(),
    )];
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
            o.line(format!(
                "            exit: Some((good: \"{}\", gross: \"inst.exit.gross\", floor: \
                 \"inst.exit.floor\", plot: \"inst.exit.plot\", commons: \"inst.commons\", land: \
                 \"land\")),",
                x.good
            ));
        }
        o.line("         ))),");
    }
    let genesis_basis = if inst.exit.is_some() {
        format!(
            "        basis: Approximate(\"oracle unit 1e (crates/oracle, ParcelEconomy) at the commons \
             frame's {}, N = {} and T = {} per tick; stationary coins per the commons frame's \
             §3.8; written by rustyecon-probe's markets-tape\"),",
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
