//! The stocks probe's harness (HORSES-SPEC §7): runs a setup on the engine, reads §7.1's
//! observables every tick from the `TickReport` and the actors' own state, scores them against
//! the oracle's values for the same tick (unit 1g, solved here, outside the Sim, at the
//! coefficients in force), and classifies the run as PROBE-SPEC §4.5 does, with the markets
//! probe's streaming classifier. It keeps §7.11's transient and stock statistics as it goes, so
//! a run of any length holds a fixed amount of memory.
//!
//! The observables are v = w/r; fodder's, the horse's, the horse-day's and the good's prices
//! over r; the good desk's human share; the cleared volume of every market; each desk's output;
//! and the two horse stocks, the heads that do the tasks (the capacity desk's at the tick's
//! start, or the owner desk's serving) and the maker's serving stock (§7.1). No dial is a band,
//! so tol_o is the floor, 1e-3, on every one (§7.3). The horse market is judged idle, not dead,
//! when it clears less than half its oracle volume (§7.2). On the flow path (R1a) the
//! observables, targets, dead ticks and start distances are P2.1's I0's, which are P2.0's.

use super::instance::{Config, Instance, Point, HOURS};
use super::perturb::Perturbation;
use super::setup::{genesis, stocks, tape_ron, Genesis, Setup};
use crate::harness::{Stop, Summary};
use crate::markets::harness::{Classifier, Ration};
use crate::perturb::Start;
use crate::protocol::{HOLD_TOL, LIVE_FLOOR, RUNAWAY, TOL_FLOOR};
use certify::battery::{rationed, within_bound};
use certify::fold::min2;
use certify::Obs;
use rustyecon_core::num;
use rustyecon_core::num::ln;
use rustyecon_core::{CoreError, FlowPerYear, Site};
use rustyecon_engine::prelude::{
    ActorId, ClassId, GoodId, Holder, PriceError, RunErrorKind, SideTag, Sim, Tape, TickReport,
};
use rustyecon_engine::rustyecon_agents::{
    maker_reservation, ActorState, AgentError, Input, Maker, Reservation, Spec,
};
use std::collections::BTreeMap;

/// The observables of an instance, in order (HORSES-SPEC §7.1): `v`, `pi.<market>` for every
/// market but labour and land, `s.good`, `vol.<market>` in market order, `y.<output>` for each
/// desk in desk order, and, off the flow path, `heads.capacity` (or `heads.good` under M1) and
/// `heads.<maker>`. On R1a this is P2.1's I0's set of ten, in its order.
pub fn observables(inst: &Instance) -> Vec<String> {
    let markets = inst.markets();
    let mut o = vec!["v".to_string()];
    o.extend(markets[2..].iter().map(|m| format!("pi.{m}")));
    o.push("s.good".into());
    o.extend(markets.iter().map(|m| format!("vol.{m}")));
    o.extend(outputs(inst).iter().map(|g| format!("y.{g}")));
    if !inst.is_flow() {
        o.push(match inst.config {
            Config::Wet => "heads.capacity".into(),
            Config::Owner => "heads.good".into(),
        });
        o.push(format!("heads.{}", inst.keys.maker));
    }
    o
}

/// Each desk's output good, in desk order.
pub fn outputs(inst: &Instance) -> Vec<String> {
    let mut v = vec!["good".to_string()];
    if inst.config == Config::Wet {
        v.push(HOURS.into());
    }
    v.push(inst.keys.horse.clone());
    if inst.has_fodder() {
        v.push("fodder".into());
    }
    v
}

/// The basket's items by good: the good, then land (space).
pub fn items() -> Vec<String> {
    vec!["good".into(), "land".into()]
}

/// GOODS-CHAIN's nine observables (HORSES-SPEC §7.11): v, the horse-day's price (the machine
/// services' on the flow path), the good's, 1 − x, cleared labour, land and goods, the good's
/// output and the total stock, as indices into [`observables`], the total stock last (its
/// index is `None`: it is summed).
fn nine(inst: &Instance) -> Vec<usize> {
    let o = observables(inst);
    let at = |k: &str| o.iter().position(|x| x == k);
    let hours = if inst.config == Config::Wet {
        format!("pi.{HOURS}")
    } else {
        format!("pi.{}", inst.keys.horse)
    };
    [
        "v",
        hours.as_str(),
        "pi.good",
        "s.good",
        "vol.labour",
        "vol.land",
        "vol.good",
        "y.good",
    ]
    .iter()
    .filter_map(|k| at(k))
    .collect()
}

/// The oracle's values at one set of coefficients, per tick, relative to r = 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    /// The observables' values, in [`observables`] order.
    pub obs: Vec<f64>,
    /// Each market's cleared volume, in market order.
    pub volume: Vec<f64>,
    /// Y, baskets eaten.
    pub baskets: f64,
    /// The provider's and the workers' baskets.
    pub households: [f64; 2],
    /// Each basket item's quantity eaten, in item order.
    pub items: Vec<f64>,
    /// Each desk's output, in desk order.
    pub output: Vec<f64>,
    /// Every installed head: the tasks' and the maker's serving stock.
    pub heads: f64,
    /// The maker's finished stock at rest: q_b and its cover.
    pub finished: f64,
    /// The point.
    pub point: Point,
}

impl Target {
    /// The target of an equilibrium with land services `land` per tick and a cover of `cover`
    /// ticks of sales.
    pub fn of(inst: &Instance, e: &Point, land: f64, cover: f64) -> Target {
        if let Some(f) = &e.flow {
            // P2.1's I0, as its harness builds the target (crate::markets::harness::Target).
            let mut volume = vec![f.n_a, land];
            volume.extend(f.type_traded.iter().copied());
            volume.extend(f.cat_output.iter().copied());
            let mut output = f.cat_output.clone();
            output.extend(f.type_services.iter().copied());
            let mut obs = vec![f.v];
            obs.extend(f.type_price.iter().copied());
            obs.extend(f.cat_price.iter().copied());
            obs.push(f.one_minus_x);
            obs.extend(volume.iter().copied());
            obs.extend(output.iter().copied());
            let mut items = f.cat_output.clone();
            items.push(inst.county.space * f.y);
            return Target {
                obs,
                volume,
                baskets: f.y,
                households: [f.provider_baskets, f.worker_baskets],
                items,
                output,
                heads: f64::NAN,
                finished: f64::NAN,
                point: e.clone(),
            };
        }
        let wet = inst.config == Config::Wet;
        let mut prices = vec![e.v];
        let mut volume = vec![e.n_a, land];
        if inst.has_fodder() {
            prices.push(e.pf);
            volume.push(e.qf);
        }
        prices.push(e.pk);
        volume.push(e.sold);
        if wet {
            prices.push(e.ph);
            volume.push(e.task_hours);
        }
        prices.push(e.p);
        volume.push(e.good);
        let mut output = vec![e.good];
        if wet {
            output.push(e.task_hours);
        }
        output.push(e.made);
        if inst.has_fodder() {
            output.push(e.qf);
        }
        let mut obs = prices;
        obs.push(e.one_minus_x);
        obs.extend(volume.iter().copied());
        obs.extend(output.iter().copied());
        obs.push(e.capacity);
        obs.push(e.serving);
        Target {
            obs,
            volume,
            baskets: e.y,
            households: [e.provider_baskets, e.worker_baskets],
            items: vec![e.good, inst.county.space * e.y],
            output,
            heads: e.capacity + e.serving,
            finished: e.made + cover * e.sold,
            point: e.clone(),
        }
    }
}

/// The distance between two oracle points (HORSES-SPEC §1.5): the largest |ln| over 1 − x, v,
/// Y, N_a, every price and the heads made, over the tolerance.
pub fn shock_distance(a: &Point, b: &Point) -> f64 {
    let pairs = [
        (a.one_minus_x, b.one_minus_x),
        (a.v, b.v),
        (a.y, b.y),
        (a.n_a, b.n_a),
        (a.p, b.p),
        (a.pf, b.pf),
        (a.pk, b.pk),
        (a.ph, b.ph),
        (a.made, b.made),
        (a.hours, b.hours),
    ];
    pairs
        .iter()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(x, y)| ln(x / y).abs())
        .fold(0.0, f64::max)
        / TOL_FLOOR
}

/// The stock records of a tick (HORSES-SPEC §7.1, §7.11): read from the actors' own state and
/// holdings after the tick, and the costs at the tick's posted prices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stocks {
    /// The heads that did the tasks: the capacity desk's holding when it decided, or the owner
    /// desk's serving stock.
    pub tasks: f64,
    /// The maker's serving stock.
    pub maker: f64,
    /// The maker's serving stock after wear, its record.
    pub own: f64,
    /// The maker's finished heads after the tick: its holding less its record.
    pub finished: f64,
    /// The capacity desk's target stock K\* (NaN under M1).
    pub target: f64,
    /// The capacity desk's order (NaN under M1).
    pub order: f64,
    /// The hours the capacity desk planned to run (NaN under M1).
    pub run: f64,
    /// A horse-day's running cost at posted prices, O.
    pub running: f64,
    /// Its full cost at posted prices, O + δ·p_K/κ.
    pub full: f64,
}

/// One tick's row: what the engine reported, the agents' own records, and the target.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The tick.
    pub tick: u64,
    /// The posted prices this tick settled at, in market order.
    pub price: Vec<f64>,
    /// The observables, in [`observables`] order.
    pub obs: Vec<f64>,
    /// The target, in the same order.
    pub target: Vec<f64>,
    /// Each observable's gap |ln(o/o\*)|; infinite when o is not positive.
    pub gap: Vec<f64>,
    /// D̂: the largest gap over its tolerance.
    pub dhat: f64,
    /// D̂ over every observable but the horse market's cleared volume, which is zero whenever
    /// its orders stop (§7.11).
    pub dhat_ex: f64,
    /// Supply per market.
    pub supply: Vec<f64>,
    /// Feasible demand per market.
    pub demand: Vec<f64>,
    /// Cleared volume per market.
    pub cleared: Vec<f64>,
    /// The buyers' fill per market.
    pub buyer_fill: Vec<f64>,
    /// The sellers' fill per market.
    pub seller_fill: Vec<f64>,
    /// Whether each market traded.
    pub trades: Vec<bool>,
    /// Spoilage of each market's good this tick.
    pub spoiled: Vec<f64>,
    /// What `Consumption` burned of each basket item this tick, in item order.
    pub eaten: Vec<f64>,
    /// Each actor's coin after the tick, in actor order.
    pub coin: Vec<f64>,
    /// The good desk's planned human share after the tick.
    pub planned: Vec<f64>,
    /// Each desk's output this tick, in desk order.
    pub output: Vec<f64>,
    /// The provider's transfer this tick: due and paid.
    pub transfer: [f64; 2],
    /// The baskets each household ate (provider, workers).
    pub baskets: [f64; 2],
    /// The item that bound each household's baskets, if it bought any item.
    pub binding: [Option<usize>; 2],
    /// The tick's ledger margin.
    pub margin: f64,
    /// Whether the tick is dead: a market but the horse's did not trade, or cleared less than
    /// the live floor of its oracle volume (on the flow path, any market).
    pub dead: bool,
    /// Whether the horse market is idle: it cleared less than the live floor of its oracle
    /// volume (off the flow path).
    pub idle: bool,
    /// Whether no one ordered horses: the horse market's demand is 0.
    pub no_order: bool,
    /// The stock records.
    pub stocks: Stocks,
    /// The maker's net markup p_K·(1 − δ·a/κ)/c_m at the tick's posted prices and the params
    /// in force, formed by its rule's own code from the tape's recipes ([`maker_readout`];
    /// IDLE-SPEC §6; NaN on the flow path). Read here, outside the Sim; no agent sees it.
    pub markup: f64,
    /// Whether the maker withheld its finished heads: a reservation ψ above 0 and its markup
    /// below it, as its rule decides.
    pub withheld: bool,
}

/// The CSV header of [`Row::csv`], for an instance.
pub fn csv_header(inst: &Instance) -> String {
    let obs = observables(inst);
    let markets = inst.markets();
    let mut h = vec!["tick".to_string()];
    h.extend(markets.iter().map(|m| format!("p.{m}")));
    h.extend(obs.iter().cloned());
    h.extend(obs.iter().map(|o| format!("{o}_star")));
    h.extend(obs.iter().map(|o| format!("gap_{o}")));
    h.push("dhat".into());
    h.push("dhat_ex".into());
    for m in &markets {
        for f in ["S", "D", "bfill", "sfill", "spoiled"] {
            h.push(format!("{m}_{f}"));
        }
    }
    for it in items() {
        h.push(format!("eaten_{it}"));
    }
    for a in inst.actors() {
        h.push(format!("coin_{a}"));
    }
    for f in [
        "s_planned_good",
        "transfer_due",
        "transfer_paid",
        "baskets_provider",
        "baskets_workers",
        "bound_provider",
        "bound_workers",
        "ledger_margin",
        "dead",
        "idle",
        "no_order",
        "stock_tasks",
        "stock_maker",
        "maker_own",
        "maker_finished",
        "capacity_target",
        "capacity_order",
        "capacity_run",
        "running_cost",
        "full_cost",
        "markup",
        "withheld",
    ] {
        h.push(f.to_string());
    }
    h.join(",")
}

impl Row {
    /// The row as a CSV line, every float in Rust's shortest round-trip form.
    pub fn csv(&self) -> String {
        let mut v = vec![self.tick.to_string()];
        let mut push = |x: f64| v.push(format!("{x:?}"));
        self.price.iter().for_each(|&x| push(x));
        self.obs.iter().for_each(|&x| push(x));
        self.target.iter().for_each(|&x| push(x));
        self.gap.iter().for_each(|&x| push(x));
        push(self.dhat);
        push(self.dhat_ex);
        for m in 0..self.supply.len() {
            push(self.supply[m]);
            push(self.demand[m]);
            push(self.buyer_fill[m]);
            push(self.seller_fill[m]);
            push(self.spoiled[m]);
        }
        self.eaten.iter().for_each(|&x| push(x));
        self.coin.iter().for_each(|&x| push(x));
        self.planned.iter().for_each(|&x| push(x));
        push(self.transfer[0]);
        push(self.transfer[1]);
        push(self.baskets[0]);
        push(self.baskets[1]);
        let names = items();
        for b in self.binding {
            v.push(b.map_or("-".to_string(), |i| names[i].clone()));
        }
        v.push(format!("{:?}", self.margin));
        v.push(u8::from(self.dead).to_string());
        v.push(u8::from(self.idle).to_string());
        v.push(u8::from(self.no_order).to_string());
        let s = &self.stocks;
        for x in [
            s.tasks, s.maker, s.own, s.finished, s.target, s.order, s.run, s.running, s.full,
        ] {
            v.push(format!("{x:?}"));
        }
        v.push(format!("{:?}", self.markup));
        v.push(u8::from(self.withheld).to_string());
        v.join(",")
    }
}

/// The first maker among a world's actors, as the tape resolved it; `None` if it has none. On
/// the flow path (R1a) it holds no reservation, and the harness reads none.
pub fn the_maker(sim: &Sim) -> Option<Maker> {
    sim.world().actors.iter().find_map(|a| match &a.spec {
        Spec::Maker(m) => Some(m.clone()),
        _ => None,
    })
}

/// The maker's reservation as its rule read it on a tick (IDLE-SPEC §6; L0.7): its net markup
/// p_K·(1 − δ·a/κ)/c_m, its ψ and whether it withheld, formed by the agents crate's own
/// [`maker_reservation`] from the resolved spec, the params in force after the tick's step
/// (the tick's events apply before its decisions) and the tick's posted prices, `price_of` by
/// good. So the readout reads the tape's own recipes, whatever they are, and cannot part from
/// the rule. Read here, outside the Sim; no agent sees it.
pub fn maker_readout(
    sim: &Sim,
    maker: &Maker,
    price_of: &dyn Fn(GoodId) -> Option<f64>,
) -> Result<Reservation, String> {
    let clock = &sim.world().clock;
    let par = |s: Site| -> Result<f64, AgentError> {
        let v = sim
            .param(s.param)
            .ok_or(AgentError::Core(CoreError::UnknownParam(s.param)))?;
        s.convert(clock, v).map_err(AgentError::Core)
    };
    let pr = |g: GoodId| -> Result<f64, AgentError> {
        price_of(g)
            .ok_or_else(|| AgentError::Core(CoreError::Shape(format!("no posted price for {g}"))))
    };
    maker_reservation(maker, &par, &pr).map_err(|e| e.to_string())
}

/// A horse-day's running cost O at posted prices, read from the tape's own running recipe (P2.2b;
/// LOOPS-RULES §8.3): the first capacity desk's (O = (0.0 + Σ run_g·p_g) + run_lab·w), or under
/// M1 the owner desk's (0.0 + Σ run_g·p_g), with the params in force after the tick's step, summed
/// in the rule's order. At every P2.2a instance that is (0.0 + 1·p_f) + 0·w, which is p_f bit for
/// bit; under rule B it is (0.0 + 0.0176·p_f) + 0.1·w. `None` where the world has neither desk.
/// Read here, outside the Sim; no agent sees it.
pub fn running_cost(
    sim: &Sim,
    price_of: &dyn Fn(GoodId) -> Option<f64>,
) -> Result<Option<f64>, String> {
    let clock = &sim.world().clock;
    let par = |s: Site| -> Result<f64, String> {
        let v = sim
            .param(s.param)
            .ok_or_else(|| format!("no param {}", s.param))?;
        s.convert(clock, v).map_err(|e| e.to_string())
    };
    let pr = |g: GoodId| price_of(g).ok_or_else(|| format!("no posted price for {g}"));
    let goods = |inputs: &[Input]| -> Result<f64, String> {
        let mut o = 0.0;
        for i in inputs {
            o += par(i.coef)? * pr(i.good)?;
        }
        Ok(o)
    };
    for a in &sim.world().actors {
        match &a.spec {
            Spec::CapacityDesk(c) => {
                let o = goods(&c.running.goods)?;
                return Ok(Some(o + par(c.running.labour)? * pr(c.labour)?));
            }
            Spec::OwnerDesk(d) => return Ok(Some(goods(&d.running)?)),
            _ => {}
        }
    }
    Ok(None)
}

/// The ids a run reads.
struct Ids {
    /// Each market's good, in market order.
    goods: Vec<GoodId>,
    /// Each basket item's good.
    items: Vec<GoodId>,
    coin: GoodId,
    /// Each actor, in actor order.
    actors: Vec<ActorId>,
    /// The households' classes: owners (the provider's) and workers.
    classes: [ClassId; 2],
}

impl Ids {
    fn of(sim: &Sim, inst: &Instance) -> Result<Ids, String> {
        let w = sim.world();
        let good = |k: &str| w.id_of::<GoodId>(k).ok_or(format!("no good {k}"));
        let actor = |k: &str| w.id_of::<ActorId>(k).ok_or(format!("no actor {k}"));
        let class = |k: &str| w.id_of::<ClassId>(k).ok_or(format!("no class {k}"));
        Ok(Ids {
            goods: inst
                .markets()
                .iter()
                .map(|m| good(m))
                .collect::<Result<_, _>>()?,
            items: items().iter().map(|m| good(m)).collect::<Result<_, _>>()?,
            coin: good("coin")?,
            actors: inst
                .actors()
                .iter()
                .map(|a| actor(a))
                .collect::<Result<_, _>>()?,
            classes: [class("owners")?, class("workers")?],
        })
    }
}

/// The per-tick coefficients the harness reads of an instance, for the costs it reports.
#[derive(Debug, Clone, Copy)]
struct Coefs {
    kappa: f64,
    delta: f64,
}

#[allow(clippy::too_many_arguments)]
fn row(
    sim: &Sim,
    inst: &Instance,
    ids: &Ids,
    r: &TickReport,
    o: &Obs,
    target: &Target,
    k: Coefs,
    horse_market: Option<usize>,
) -> Result<Row, String> {
    let n = ids.goods.len();
    let line = |g: GoodId| {
        o.markets
            .iter()
            .position(|l| l.good == g)
            .ok_or(format!("no market line for {g}"))
    };
    let mut at = Vec::with_capacity(n);
    for &g in &ids.goods {
        at.push(line(g)?);
    }
    let price: Vec<f64> = at.iter().map(|&i| o.markets[i].price).collect();
    let supply: Vec<f64> = at.iter().map(|&i| o.markets[i].supply).collect();
    let demand: Vec<f64> = at.iter().map(|&i| o.markets[i].demand).collect();
    let cleared: Vec<f64> = at.iter().map(|&i| o.markets[i].cleared).collect();
    let buyer_fill: Vec<f64> = at.iter().map(|&i| o.markets[i].buyer_fill).collect();
    let seller_fill: Vec<f64> = at.iter().map(|&i| o.markets[i].seller_fill).collect();
    let trades: Vec<bool> = at.iter().map(|&i| o.markets[i].trades()).collect();
    let spoiled = ids
        .goods
        .iter()
        .map(|g| {
            o.spoiled
                .get(g.idx())
                .copied()
                .ok_or(format!("no spoilage for {g}"))
        })
        .collect::<Result<Vec<f64>, _>>()?;
    let eaten = ids
        .items
        .iter()
        .map(|g| o.consumed.get(g.idx()).copied().unwrap_or(f64::NAN))
        .collect();
    let desks = inst.desks();
    let horse = ids.goods[inst
        .markets()
        .iter()
        .position(|m| *m == inst.keys.horse)
        .ok_or("no horse market")?];
    let held = |a: ActorId, g: GoodId| sim.holding(Holder::Actor(a)).map_or(0.0, |inv| inv.get(g));
    let mut planned = Vec::with_capacity(1);
    let mut used = Vec::with_capacity(1);
    let mut output = Vec::with_capacity(desks.len());
    let mut st = Stocks {
        tasks: 0.0,
        maker: 0.0,
        own: 0.0,
        finished: 0.0,
        target: f64::NAN,
        order: f64::NAN,
        run: f64::NAN,
        running: f64::NAN,
        full: f64::NAN,
    };
    for (d, &a) in ids.actors.iter().take(desks.len()).enumerate() {
        match (desks[d].as_str(), sim.actor_state(a)) {
            ("good", Some(ActorState::GoodDesk(s))) => {
                planned.push(s.share);
                used.push(s.used);
                output.push(s.output);
            }
            ("good", Some(ActorState::Owner(s))) => {
                planned.push(s.share);
                used.push(s.used);
                output.push(s.output);
                st.tasks = s.serving;
            }
            ("capacity", Some(ActorState::Capacity(s))) => {
                output.push(s.output);
                st.tasks = s.held;
                st.target = s.target;
                st.order = s.order;
                st.run = s.run;
            }
            ("fodder", Some(ActorState::MachDesk(s))) => output.push(s.output),
            (_, Some(ActorState::Maker(s))) => {
                output.push(s.output);
                st.maker = s.serving;
                st.own = s.own;
                let h = held(a, horse) - s.own;
                st.finished = if h > 0.0 { h } else { 0.0 };
            }
            _ => return Err(format!("actor {a} is not the desk its instance says")),
        }
    }
    let provider = ids.actors[ids.actors.len() - 2];
    let transfer = match o.transfers.iter().find(|(a, _, _)| *a == provider) {
        Some(&(_, due, paid)) => [due, paid],
        None => return Err("provider is not a provider".into()),
    };
    let coin: Vec<f64> = ids.actors.iter().map(|&a| held(a, ids.coin)).collect();
    // Each household's baskets, from what its class bought of each item this tick, as P2.1's
    // harness reads them.
    let weights = [1.0, inst.county.space];
    let mut baskets = [0.0; 2];
    let mut binding = [None; 2];
    for (h, &class) in ids.classes.iter().enumerate() {
        let mut best: Option<(f64, usize)> = None;
        for (i, &g) in ids.items.iter().enumerate() {
            if weights[i] <= 0.0 {
                continue;
            }
            let got = r
                .rationing
                .iter()
                .filter(|l| l.good == g && l.class == class && l.side == SideTag::Buy)
                .map(|l| l.filled)
                .fold(0.0, |a, b| a + b);
            let n_i = num::max_scale(got, weights[i]).map_err(|e| e.to_string())?;
            if best.is_none_or(|(b, _)| n_i < b) {
                best = Some((n_i, i));
            }
        }
        if let Some((b, i)) = best {
            baskets[h] = b;
            binding[h] = Some(i);
        }
    }
    let [w, rr] = [price[0], price[1]];
    // A horse-day's running cost and full cost at posted prices, from the tape's running recipe
    // (P2.2b.2; LOOPS-RULES §8.3): at every P2.2a instance, p_f bit for bit.
    if !inst.is_flow() {
        let markets = inst.markets();
        let pk = price[markets
            .iter()
            .position(|m| *m == inst.keys.horse)
            .unwrap_or(0)];
        let price_of = |g: GoodId| ids.goods.iter().position(|x| *x == g).map(|i| price[i]);
        let o_run = running_cost(sim, &price_of)?.unwrap_or(f64::NAN);
        st.running = o_run;
        st.full = o_run + k.delta * pk / k.kappa;
    }
    let mut obs = vec![w / rr];
    obs.extend(price[2..].iter().map(|p| p / rr));
    obs.extend(used.iter().copied());
    obs.extend(cleared.iter().copied());
    obs.extend(output.iter().copied());
    if !inst.is_flow() {
        obs.push(st.tasks);
        obs.push(st.maker);
    }
    let gap: Vec<f64> = obs
        .iter()
        .zip(&target.obs)
        .map(|(&x, &t)| {
            if x > 0.0 && x.is_finite() {
                ln(x / t).abs()
            } else {
                f64::INFINITY
            }
        })
        .collect();
    let dhat = gap.iter().fold(0.0, |a: f64, &g| a.max(g)) / TOL_FLOOR;
    // vol.<horse>'s index: v, the n − 2 prices, s.good, then the volumes in market order.
    let vol_horse = horse_market.map(|m| n + m);
    let dhat_ex = gap
        .iter()
        .enumerate()
        .filter(|(i, _)| Some(*i) != vol_horse)
        .fold(0.0, |a: f64, (_, &g)| a.max(g))
        / TOL_FLOOR;
    let dead = (0..n).any(|m| {
        Some(m) != horse_market && (!trades[m] || cleared[m] < LIVE_FLOOR * target.volume[m])
    });
    let (idle, no_order) = match horse_market {
        Some(m) => (cleared[m] < LIVE_FLOOR * target.volume[m], demand[m] == 0.0),
        None => (false, false),
    };
    Ok(Row {
        tick: r.tick,
        price,
        obs,
        target: target.obs.clone(),
        gap,
        dhat,
        dhat_ex,
        supply,
        demand,
        cleared,
        buyer_fill,
        seller_fill,
        trades,
        spoiled,
        eaten,
        coin,
        planned,
        output,
        transfer,
        baskets,
        binding,
        margin: o.margin,
        dead,
        idle,
        no_order,
        stocks: st,
        markup: f64::NAN,
        withheld: false,
    })
}

/// Mode A's per-tick check (PROBE-SPEC §4.6, HORSES-SPEC §7.6): every gap at most 1e-9, every
/// market trading (the horse market too) with both fills at least 1 − 1e-9, and no produced good
/// spoiling beyond 1e-9 of its volume.
pub(crate) fn hold_check(row: &Row, names: &[String], markets: &[String]) -> Option<String> {
    if let Some(i) = (0..row.gap.len()).find(|&i| row.gap[i].is_nan() || row.gap[i] > HOLD_TOL) {
        return Some(format!(
            "tick {}: {} is {:e} from the oracle in log",
            row.tick, names[i], row.gap[i]
        ));
    }
    for (m, market) in markets.iter().enumerate() {
        if !row.trades[m] {
            return Some(format!("tick {}: {market} did not trade", row.tick));
        }
        if rationed(row.buyer_fill[m], HOLD_TOL) || rationed(row.seller_fill[m], HOLD_TOL) {
            return Some(format!(
                "tick {}: {market} filled {:e} (buyers) and {:e} (sellers)",
                row.tick, row.buyer_fill[m], row.seller_fill[m]
            ));
        }
        if m >= 2 && row.spoiled[m] > HOLD_TOL * row.cleared[m] {
            return Some(format!(
                "tick {}: {:e} of {market} spoiled",
                row.tick, row.spoiled[m]
            ));
        }
    }
    None
}

/// The stock statistics of HORSES-SPEC §7.11 (D-G14, the glut), over the scored run against the
/// oracle's values at the coefficients in force. Reported, never scored.
#[derive(Debug, Clone, PartialEq)]
pub struct StockStats {
    /// The largest D̂ without the horse market's volume, and its tick.
    pub peak_ex: (f64, u64),
    /// The installed heads' lowest and highest over their target.
    pub heads_range: (f64, f64),
    /// The scored tick from which the installed heads stay within 5% of their target, if they
    /// do.
    pub in_five: Option<u64>,
    /// The paper's time: 1 tick for an expansion, ln(K′/K)/ln(1 − δ) ticks of zero builds for a
    /// contraction, K and K′ the installed heads at the genesis point and at the target.
    pub paper: f64,
    /// The quasi-rent p_h/(O + δ·p_K/κ) − 1 at posted prices, 3 ticks and 1/δ ticks after the
    /// scored clock starts.
    pub quasi: (f64, f64),
    /// Scored ticks on which no one ordered horses, and on which the horse market was idle.
    pub no_order: u64,
    /// Idle ticks of the horse market.
    pub idle: u64,
    /// The horse's lowest price over its target.
    pub pk_low: f64,
    /// The maker's highest finished stock over its rest value.
    pub finished_high: f64,
    /// The capacity desk's lowest utilisation, z/(κ·held).
    pub utilisation_low: f64,
    /// The hour price's lowest over its running cost at posted prices.
    pub hour_over_o_low: f64,
    /// Horses bought, trough and peak over their oracle volume.
    pub investment: (f64, f64),
    /// The scored tick from which GOODS-CHAIN's nine stay within tolerance, if they do.
    pub nine_in_tol: Option<u64>,
    /// Scored ticks on which the maker withheld its finished heads (its markup below ψ).
    pub withheld: u64,
    /// Changes between withholding and offering over the scored ticks.
    pub switches: u64,
    /// The maker's lowest net markup over the scored ticks.
    pub markup_low: f64,
}

/// The transient statistics of HORSES-SPEC §7.11 (O14), with P2.1's set (MARKETS-SPEC §7.11)
/// and the stocks'. Reported, never scored.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// The largest D̂ and its tick.
    pub peak: (f64, u64),
    /// Dead ticks per market: no trade, below the live floor, no supply, no demand.
    pub dead_market: Vec<[u64; 4]>,
    /// Each market's trough of cleared volume over its oracle volume, its tick, and its value on
    /// the last tick.
    pub trough: Vec<(f64, u64, f64)>,
    /// Each desk's trough of output over its oracle output, and its tick.
    pub output_trough: Vec<(f64, u64)>,
    /// The trough of baskets eaten over Y\*, its tick, and the ticks with none eaten.
    pub baskets: (f64, u64, u64),
    /// Each item's trough of what was eaten over z_j·Y\*, and the ticks with none eaten.
    pub item: Vec<(f64, u64)>,
    /// For each item, the household ticks with baskets below 0.99 of the household's oracle
    /// baskets in which it bound; and the number of such household ticks.
    pub binding: (Vec<u64>, u64),
    /// Rationing per (market index, class key, side).
    pub rationing: BTreeMap<(usize, String, &'static str), Ration>,
    /// Per market good: Σ spoiled and Σ supplied.
    pub spoilage: Vec<(f64, f64)>,
    /// The provider's Σ(due − paid) and its ticks short.
    pub transfer: (f64, u64),
    /// The worst buyer and seller fill over every market (sides that exist).
    pub worst_fill: f64,
    /// For a cost shock: ln(Y′/Y) of the equilibrium change, and the trough of baskets eaten
    /// in log against Y and against Y′.
    pub depth: Option<(f64, f64, f64)>,
    /// The stocks' statistics (off the flow path).
    pub stock: StockStats,
}

impl Stats {
    fn new(inst: &Instance) -> Stats {
        Stats::sized(inst.markets().len(), inst.desks().len(), items().len())
    }

    /// Empty statistics for `nm` markets, `nd` desks and `ni` basket items.
    pub(crate) fn sized(nm: usize, nd: usize, ni: usize) -> Stats {
        Stats {
            peak: (f64::NEG_INFINITY, 0),
            dead_market: vec![[0; 4]; nm],
            trough: vec![(f64::INFINITY, 0, f64::NAN); nm],
            output_trough: vec![(f64::INFINITY, 0); nd],
            baskets: (f64::INFINITY, 0, 0),
            item: vec![(f64::INFINITY, 0); ni],
            binding: (vec![0; ni], 0),
            rationing: BTreeMap::new(),
            spoilage: vec![(0.0, 0.0); nm],
            transfer: (0.0, 0),
            worst_fill: 1.0,
            depth: None,
            stock: StockStats {
                peak_ex: (f64::NEG_INFINITY, 0),
                heads_range: (f64::INFINITY, f64::NEG_INFINITY),
                in_five: None,
                paper: f64::NAN,
                quasi: (f64::NAN, f64::NAN),
                no_order: 0,
                idle: 0,
                pk_low: f64::INFINITY,
                finished_high: f64::NEG_INFINITY,
                utilisation_low: f64::INFINITY,
                hour_over_o_low: f64::INFINITY,
                investment: (f64::INFINITY, f64::NEG_INFINITY),
                nine_in_tol: None,
                withheld: 0,
                switches: 0,
                markup_low: f64::INFINITY,
            },
        }
    }

    fn trough(slot: &mut (f64, u64), tick: u64, x: f64) {
        if slot.0.is_nan() {
            return;
        }
        if x.is_nan() || x < slot.0 {
            *slot = (x, tick);
        }
    }

    pub(crate) fn push(&mut self, row: &Row, o: &Obs, t: &Target, class_names: &[String]) {
        if !self.peak.0.is_nan() && (row.dhat.is_nan() || row.dhat > self.peak.0) {
            self.peak = (row.dhat, row.tick);
        }
        for m in 0..row.supply.len() {
            let d = &mut self.dead_market[m];
            let no_trade = !row.trades[m];
            d[0] += u64::from(no_trade);
            d[1] += u64::from(row.cleared[m] < LIVE_FLOOR * t.volume[m]);
            d[2] += u64::from(no_trade && row.supply[m] == 0.0);
            d[3] += u64::from(no_trade && row.demand[m] == 0.0);
            let rel = row.cleared[m] / t.volume[m];
            let mut slot = (self.trough[m].0, self.trough[m].1);
            Stats::trough(&mut slot, row.tick, rel);
            self.trough[m] = (slot.0, slot.1, rel);
            self.spoilage[m].0 += row.spoiled[m];
            self.spoilage[m].1 += row.supply[m];
            if row.demand[m] > 0.0 {
                self.worst_fill = min2(self.worst_fill, row.buyer_fill[m]);
            }
            if row.supply[m] > 0.0 {
                self.worst_fill = min2(self.worst_fill, row.seller_fill[m]);
            }
        }
        for (d, (&y, &ys)) in row.output.iter().zip(&t.output).enumerate() {
            Stats::trough(&mut self.output_trough[d], row.tick, y / ys);
        }
        let total = row.baskets[0] + row.baskets[1];
        let mut slot = (self.baskets.0, self.baskets.1);
        Stats::trough(&mut slot, row.tick, total / t.baskets);
        self.baskets = (slot.0, slot.1, self.baskets.2 + u64::from(total == 0.0));
        for (i, (&e, &z)) in row.eaten.iter().zip(&t.items).enumerate() {
            let mut slot = (self.item[i].0, 0);
            Stats::trough(&mut slot, row.tick, e / z);
            self.item[i] = (slot.0, self.item[i].1 + u64::from(e == 0.0));
        }
        for h in 0..2 {
            if row.baskets[h] < 0.99 * t.households[h] {
                self.binding.1 += 1;
                if let Some(i) = row.binding[h] {
                    self.binding.0[i] += 1;
                }
            }
        }
        for l in &o.rationing {
            let class = class_names
                .get(l.class.idx())
                .cloned()
                .unwrap_or_else(|| l.class.to_string());
            let side = match l.side {
                SideTag::Buy => "buy",
                SideTag::Sell => "sell",
            };
            let r = self
                .rationing
                .entry((l.market as usize, class, side))
                .or_insert(Ration {
                    worst: f64::INFINITY,
                    ticks: 0,
                    budget: 0.0,
                    market: 0.0,
                });
            r.budget += l.requested - l.feasible;
            r.market += l.feasible - l.filled;
            if l.feasible > 0.0 {
                let f = l.filled / l.feasible;
                r.worst = min2(r.worst, f);
                r.ticks += u64::from(rationed(f, HOLD_TOL));
            }
        }
        self.transfer.0 += row.transfer[0] - row.transfer[1];
        self.transfer.1 += u64::from(row.transfer[1] < row.transfer[0]);
    }
}

/// The streaming stock statistics' state, beside [`StockStats`].
pub(crate) struct StockTrack {
    last_out_five: Option<u64>,
    last_out_nine: Option<u64>,
    seen: u64,
    one_over_delta: u64,
    horse: Option<usize>,
    wet: bool,
    kappa: f64,
    nine: Vec<usize>,
    /// Whether the maker withheld on the last scored tick.
    last_withheld: Option<bool>,
}

impl StockTrack {
    /// The state for a run: the horse market's index (none on a flow path), whether the capacity
    /// desk hires out the hours, κ and δ a tick, and GOODS-CHAIN's nine's indices.
    pub(crate) fn new(
        horse: Option<usize>,
        wet: bool,
        kappa: f64,
        delta: f64,
        nine: Vec<usize>,
    ) -> StockTrack {
        StockTrack {
            last_out_five: None,
            last_out_nine: None,
            seen: 0,
            one_over_delta: (1.0 / delta).round() as u64,
            horse,
            wet,
            kappa,
            nine,
            last_withheld: None,
        }
    }

    pub(crate) fn push(&mut self, st: &mut StockStats, row: &Row, t: &Target, scored: u64) {
        self.seen += 1;
        if !st.peak_ex.0.is_nan() && (row.dhat_ex.is_nan() || row.dhat_ex > st.peak_ex.0) {
            st.peak_ex = (row.dhat_ex, row.tick);
        }
        // GOODS-CHAIN's nine: its eight observables and the total stock.
        let heads = row.stocks.tasks + row.stocks.maker;
        let mut nine_out = self
            .nine
            .iter()
            .any(|&i| row.gap[i].is_nan() || row.gap[i] > TOL_FLOOR);
        let Some(m) = self.horse else {
            if nine_out {
                self.last_out_nine = Some(scored);
            }
            return;
        };
        let gap_heads = if heads > 0.0 {
            ln(heads / t.heads).abs()
        } else {
            f64::INFINITY
        };
        nine_out |= gap_heads.is_nan() || gap_heads > TOL_FLOOR;
        if nine_out {
            self.last_out_nine = Some(scored);
        }
        let rel = heads / t.heads;
        st.heads_range = (min2(st.heads_range.0, rel), st.heads_range.1.max(rel));
        let within = (rel - 1.0).abs() <= 0.05;
        if !within {
            self.last_out_five = Some(scored);
        }
        let pk = row.price[m] / row.price[1];
        let q = if self.wet {
            let ph = row.price[m + 1];
            ph / row.stocks.full - 1.0
        } else {
            f64::NAN
        };
        if scored == 3 {
            st.quasi.0 = q;
        }
        if scored == self.one_over_delta {
            st.quasi.1 = q;
        }
        st.no_order += u64::from(row.no_order);
        st.idle += u64::from(row.idle);
        st.pk_low = min2(st.pk_low, pk / t.point.pk);
        st.finished_high = st.finished_high.max(row.stocks.finished / t.finished);
        if self.wet {
            let cap = self.kappa * row.stocks.tasks;
            let u = if cap > 0.0 { row.output[1] / cap } else { 1.0 };
            st.utilisation_low = min2(st.utilisation_low, u);
            let ph = row.price[m + 1];
            if row.stocks.running > 0.0 {
                st.hour_over_o_low = min2(st.hour_over_o_low, ph / row.stocks.running);
            }
        }
        let inv = row.cleared[m] / t.volume[m];
        st.investment = (min2(st.investment.0, inv), st.investment.1.max(inv));
        // The idle market's readouts (IDLE-SPEC §6, "The harness").
        st.withheld += u64::from(row.withheld);
        if self.last_withheld.is_some_and(|w| w != row.withheld) {
            st.switches += 1;
        }
        self.last_withheld = Some(row.withheld);
        st.markup_low = min2(st.markup_low, row.markup);
    }

    pub(crate) fn finish(&self, st: &mut StockStats) {
        let settle = |last: Option<u64>| match last {
            None if self.seen > 0 => Some(0),
            None => None,
            Some(k) if k + 1 < self.seen => Some(k + 1),
            Some(_) => None,
        };
        st.in_five = settle(self.last_out_five);
        st.nine_in_tol = settle(self.last_out_nine);
    }
}

/// A run's record: what the classifier and the reports read.
#[derive(Debug, Clone)]
pub struct Record {
    /// The run's name.
    pub name: String,
    /// The setup it ran.
    pub setup: Setup,
    /// Its genesis.
    pub genesis: Genesis,
    /// How its start distance is measured.
    pub start: Start,
    /// D̂_0.
    pub d0: f64,
    /// The tick the scored clock starts at.
    pub clock_start: u64,
    /// The scored length L.
    pub ticks: u64,
    /// How it ended.
    pub stop: Stop,
    /// The summary: PROBE-SPEC §4.5's class and numbers.
    pub summary: Summary,
    /// §7.11's statistics.
    pub stats: Stats,
    /// The last row.
    pub last: Option<Row>,
    /// Mode A's check: the first tick and observable or market that failed, if any.
    pub hold_failure: Option<String>,
    /// The market names, in order, as the engine's market lines index them.
    pub engine_markets: Vec<String>,
}

/// Run a setup built from `name` for `ticks` scored ticks (and the ticks before a dated shock),
/// calling `each` with every row.
pub fn run(
    base: &Setup,
    name: &str,
    ticks: u64,
    each: &mut dyn FnMut(&Row),
) -> Result<Record, String> {
    let pert = Perturbation::parse(name)?;
    let mut setup = base.clone();
    pert.apply(&mut setup, ticks)?;
    let inst = setup.instance.clone();
    let text = tape_ron(&setup)?;
    let tape = Tape::from_ron(&text).map_err(|e| format!("the tape does not load: {e}"))?;
    let mut sim = Sim::new(&tape).map_err(|e| format!("the tape does not load: {e}"))?;
    let ids = Ids::of(&sim, &inst)?;
    let g = genesis(&setup)?;
    let land = sim
        .world()
        .clock
        .flow(FlowPerYear(setup.instance.county.land));
    let names = observables(&inst);
    let markets = inst.markets();
    let (kappa, delta) = inst.per_tick(setup.tpy)?;
    let k = Coefs { kappa, delta };
    let horse_market = if inst.is_flow() {
        None
    } else {
        markets.iter().position(|m| *m == inst.keys.horse)
    };
    let cover = setup.cover_ticks()?;
    let class_names: Vec<String> = sim.world().classes.iter().map(|k| k.to_string()).collect();
    let engine_markets: Vec<String> = {
        let w = sim.world();
        w.markets()
            .map(|(_, g)| w.key_of(g).map_or_else(|| g.to_string(), |k| k.to_string()))
            .collect()
    };
    // The target for each b in force.
    let mut targets: Vec<(u64, Target)> = Vec::new();
    let mut target_at = |tick: u64| -> Result<Target, String> {
        let b = setup.b_at(tick);
        if let Some((_, t)) = targets.iter().find(|(k, _)| *k == b.to_bits()) {
            return Ok(t.clone());
        }
        let i = setup.instance_at(tick);
        let t = Target::of(&i, &i.point(setup.tpy)?, land, cover);
        targets.push((b.to_bits(), t.clone()));
        Ok(t)
    };
    let clock_start = pert.clock_start(ticks);
    let start = pert.start();
    let t0 = target_at(clock_start)?;
    let n_prices = markets.len() - 1;
    // D̂_0 (PROBE-SPEC §4.5; HORSES-SPEC §7.5): a stock or coin displacement off the flow path
    // reads its first year's largest D̂, set below.
    let first_year = !inst.is_flow() && matches!(start, Start::Stocks);
    let mut d0 = match start {
        Start::Hold | Start::Nominal => 0.0,
        Start::Prices => {
            let mut genesis_obs = vec![g.prices[0] / g.prices[1]];
            genesis_obs.extend(g.prices[2..].iter().map(|p| p / g.prices[1]));
            genesis_obs.push(g.share);
            genesis_obs
                .iter()
                .zip(&t0.obs)
                .map(|(o, t)| ln(o / t).abs())
                .fold(0.0, f64::max)
                / TOL_FLOOR
        }
        Start::Stocks if first_year => 0.0,
        Start::Stocks => {
            pert.stock_factors()
                .iter()
                .map(|f| ln(*f).abs())
                .fold(0.0, f64::max)
                / TOL_FLOOR
        }
        Start::Shock => shock_distance(&t0.point, &g.point),
    };
    let year = u64::from(setup.tpy);
    let total = clock_start + ticks;
    let mut classifier = Classifier::new(ticks, names.len(), n_prices);
    let mut stats = Stats::new(&inst);
    let mut track = StockTrack::new(
        horse_market,
        inst.config == Config::Wet,
        kappa,
        delta,
        nine(&inst),
    );
    // The maker whose reservation the rows read (none on the flow path).
    let maker = if inst.is_flow() {
        None
    } else {
        the_maker(&sim)
    };
    let (k0, k1) = (g.point.capacity + g.point.serving, t0.heads);
    stats.stock.paper = if k1 >= k0 {
        1.0
    } else {
        ln(k1 / k0) / num::ln1p(-delta)
    };
    let mut stop = Stop::Ran;
    let mut last: Option<Row> = None;
    let mut hold_failure = None;
    for _ in 0..total {
        let tick = sim.tick();
        let target = target_at(tick)?;
        let report = match sim.step() {
            Ok(r) => r,
            Err(e) => {
                stop = match &e.kind {
                    RunErrorKind::Price(PriceError::NonFinite { .. }) => {
                        Stop::Runaway(e.to_string())
                    }
                    _ => Stop::Error(e.to_string()),
                };
                break;
            }
        };
        let o = Obs::of(&report, &sim);
        let mut row = row(&sim, &inst, &ids, &report, &o, &target, k, horse_market)?;
        if let Some(m) = &maker {
            let price_of = |g: GoodId| ids.goods.iter().position(|x| *x == g).map(|i| row.price[i]);
            let res = maker_readout(&sim, m, &price_of)?;
            row.markup = res.markup;
            row.withheld = res.withholds;
        }
        each(&row);
        if hold_failure.is_none() {
            hold_failure = hold_check(&row, &names, &markets);
        }
        if row.tick >= clock_start {
            let scored = row.tick - clock_start;
            if first_year && scored < year {
                d0 = d0.max(row.dhat);
            }
            let logs: Vec<f64> = row.obs.iter().map(|&x| ln(x)).collect();
            classifier.push(row.dhat, &logs, row.dead);
            stats.push(&row, &o, &target, &class_names);
            track.push(&mut stats.stock, &row, &target, scored);
        }
        // The runaway bound (PROBE-SPEC §4.5): every posted price within [1e-6, 1e6] times its
        // genesis value.
        let away = ids.goods.iter().enumerate().find_map(|(m, &gid)| {
            let l = report.markets.iter().find(|l| l.good == gid)?;
            let rel = l.next_price / g.prices[m];
            (!within_bound(rel, RUNAWAY)).then(|| {
                format!(
                    "the price of {} left the runaway bound at tick {}: {rel:e} of genesis",
                    markets[m], report.tick
                )
            })
        });
        last = Some(row);
        if let Some(why) = away {
            stop = Stop::Runaway(why);
            break;
        }
    }
    track.finish(&mut stats.stock);
    if matches!(start, Start::Shock) {
        let y1 = target_at(total.saturating_sub(1))?.baskets;
        let y0 = g.point.y;
        let low = stats.baskets.0 * target_at(clock_start)?.baskets;
        stats.depth = Some((ln(y1 / y0), ln(low / y0), ln(low / y1)));
    }
    let r_end = last.as_ref().map_or(f64::NAN, |r| r.price[1]) / g.prices[1];
    let summary = classifier.summary(&stop, d0, start, r_end);
    Ok(Record {
        name: name.to_string(),
        setup,
        genesis: g,
        start,
        d0,
        clock_start,
        ticks,
        stop,
        summary,
        stats,
        last,
        hold_failure,
        engine_markets,
    })
}

/// The stocks a setup's genesis holds, by name, for the reports.
pub fn genesis_stocks(s: &Setup) -> Result<Vec<(String, f64)>, String> {
    let g = genesis(s)?;
    Ok(stocks(&s.instance).into_iter().zip(g.stock).collect())
}
