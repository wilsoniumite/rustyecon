//! The loop step's harness (P2.2b; docs/probe/LOOPS-RULES.md §8): runs a loop setup on the engine,
//! reads §8.1's observables every tick from the `TickReport` and the actors' own state, scores
//! them against the oracle's values for the same tick (FUNDED's chain8, solved here, outside the
//! Sim, at the coefficients in force), and classifies the run with the markets probe's streaming
//! classifier, as P2.2a's harness does (HORSES-RULES §5). The rows, the transient and stock
//! statistics and the reports are P2.2a's ([`crate::horses::harness`], [`crate::horses::report`]),
//! read on a loop instance; §8.3's new readouts are [`Readouts`].
//!
//! The observables (§8.1): v; fodder's, the horse's (wet), the horse-day's and the good's prices
//! over r; the good desk's human share; every market's cleared volume; each desk's output; the
//! capacity desk's heads (wet); and each plant held, the planted state's `plant.held` (the plant
//! at decide), where θ < 1. The maker's own heads are dropped: rule B's maker holds no serving
//! stock. No dial is a band, so tol_o is the floor, 1e-3, on every one. A tick is dead when
//! labour, land, fodder, the horse-days or the good does not trade or clears below half its
//! target; the horse market is idle, not dead.

use super::perturb::Perturbation;
use super::{genesis, tape_ron, Config, Genesis, Instance, Point, Setup};
use crate::harness::{Stop, Summary};
use crate::horses::harness::{
    hold_check, items, maker_readout, running_cost, shock_distance, the_maker, Row, Stats,
    StockTrack, Stocks, Target as Base,
};
use crate::horses::instance::{Point as BasePoint, HOURS};
use crate::horses::report::{g, opt, stats_fields, summary_fields};
use crate::markets::harness::Classifier;
use crate::perturb::Start;
use crate::protocol::{LIVE_FLOOR, RUNAWAY, TOL_FLOOR};
use certify::battery::within_bound;
use certify::fold::min2;
use certify::Obs;
use rustyecon_core::num;
use rustyecon_core::num::ln;
use rustyecon_core::{FlowPerYear, Site};
use rustyecon_engine::prelude::{
    ActorId, ClassId, GoodId, Holder, PriceError, RunErrorKind, SideTag, Sim, Tape, TickReport,
};
use rustyecon_engine::rustyecon_agents::{ActorState, CapacityState, MakerState, PlantState, Spec};

/// The observables of a setup, in order (LOOPS-RULES §8.1): `v`, `pi.<market>` for every market
/// but labour and land, `s.good`, `vol.<market>` in market order, `y.<output>` for each desk in
/// desk order, `heads.capacity` (wet), and `plant.<d>` for each planted desk where θ < 1.
pub fn observables(s: &Setup) -> Vec<String> {
    let inst = &s.instance;
    let markets = inst.markets();
    let mut o = vec!["v".to_string()];
    o.extend(markets[2..].iter().map(|m| format!("pi.{m}")));
    o.push("s.good".into());
    o.extend(markets.iter().map(|m| format!("vol.{m}")));
    o.extend(outputs(inst).iter().map(|g| format!("y.{g}")));
    if !inst.is_flow() {
        o.push("heads.capacity".into());
    }
    if s.theta < 1.0 {
        o.extend(inst.planted().iter().map(|d| format!("plant.{d}")));
    }
    o
}

/// Each desk's output good, in desk order: the good, the horse-days, the horse (wet), fodder.
pub fn outputs(inst: &Instance) -> Vec<String> {
    let mut v = vec!["good".to_string(), HOURS.to_string()];
    if !inst.is_flow() {
        v.push("horse".into());
    }
    v.push("fodder".into());
    v
}

/// GOODS-CHAIN's nine's observables (HORSES-SPEC §7.11), as indices into [`observables`]; the
/// total stock, the ninth, is summed by the stock statistics (wet).
fn nine(s: &Setup) -> Vec<usize> {
    let o = observables(s);
    let at = |k: &str| o.iter().position(|x| x == k);
    let hours = format!("pi.{HOURS}");
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

/// A loop point as P2.2a's statistics read one: heads made are heads sold (a = 0), the capacity
/// desk's heads X/κ, no serving stock; the flow control has no horse.
fn base_point(inst: &Instance, e: &Point, kappa: f64) -> BasePoint {
    let wet = !inst.is_flow();
    BasePoint {
        x_star: e.x_star,
        one_minus_x: e.one_minus_x,
        v: e.v,
        p_s: e.p_s,
        y: e.y,
        n_a: e.n_a,
        p: e.p,
        good: e.good,
        pf: e.pf,
        pk: if wet { e.pk } else { f64::NAN },
        ph: e.ph,
        o: e.o,
        qf: e.qf,
        made: if wet { e.made } else { 0.0 },
        sold: if wet { e.made } else { 0.0 },
        hours: e.hours,
        task_hours: e.task_hours,
        capacity: if wet { e.hours / kappa } else { 0.0 },
        serving: 0.0,
        provider_baskets: e.provider_baskets,
        worker_baskets: e.worker_baskets,
        funded: e.funded,
        flow: None,
    }
}

/// The oracle's values at one set of coefficients, per tick, relative to r = 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    /// P2.2a's target, which its statistics read: the observables in [`observables`] order,
    /// the volumes, baskets, items, outputs, the capacity desk's heads X/κ and the maker's
    /// finished stock at rest.
    pub base: Base,
    /// Each planted desk's plant at rest, κ_p·X_d, in [`Instance::planted`] order.
    pub plants: Vec<f64>,
    /// The horse-days the tasks use, Y·J(x\*), and the fodder desk's, 2·q_f (0 with the loop cut).
    pub hours: [f64; 2],
    /// The point.
    pub point: Point,
}

impl Target {
    /// The target of `s` at the point `e` of the instance in force `inst`, with land services
    /// `land` per tick (LOOPS-RULES §8.1).
    pub fn of(s: &Setup, inst: &Instance, e: &Point, land: f64) -> Result<Target, String> {
        let wet = !inst.is_flow();
        let (kappa, _) = inst.per_tick(s.tpy)?;
        let kap = s.plant_params()?.rest_kappa();
        let heads = e.hours / kappa;
        let mut prices = vec![e.v, e.pf];
        let mut volume = vec![e.n_a, land, e.qf];
        if wet {
            prices.push(e.pk);
            volume.push(e.made);
        }
        prices.push(e.ph);
        volume.push(e.hours);
        prices.push(e.p);
        volume.push(e.good);
        let mut output = vec![e.good, e.hours];
        if wet {
            output.push(e.made);
        }
        output.push(e.qf);
        // Each plant as genesis writes it (LOOPS-RULES §7.4).
        let plants: Vec<f64> = inst
            .planted()
            .iter()
            .map(|d| match d.as_str() {
                "capacity" => kap * (kappa * heads),
                "maker" => kap * e.made,
                "fodder" => kap * e.qf,
                _ => kap * e.hours,
            })
            .collect();
        let mut obs = prices;
        obs.push(e.one_minus_x);
        obs.extend(volume.iter().copied());
        obs.extend(output.iter().copied());
        if wet {
            obs.push(heads);
        }
        if s.theta < 1.0 {
            obs.extend(plants.iter().copied());
        }
        // The maker's finished stock at rest: q_b and its cover of b_K ticks of sales at cost
        // over price, its cost summed as its rule sums it (LOOPS-RULES §7.4).
        let finished = if wet {
            let r_ = inst.rule;
            let mut cm = 0.0;
            cm += r_.horse_fodder * e.pf;
            cm += r_.horse_labour * e.v;
            cm += r_.horse_land * inst.b;
            e.made + s.cover_ticks()? * e.made * cm / e.pk
        } else {
            f64::NAN
        };
        Ok(Target {
            base: Base {
                obs,
                volume,
                baskets: e.y,
                households: [e.provider_baskets, e.worker_baskets],
                items: vec![e.good, inst.county.space * e.y],
                output,
                heads: if wet { heads } else { f64::NAN },
                finished,
                point: base_point(inst, e, kappa),
            },
            plants,
            hours: [e.task_hours, e.fodder_hours],
            point: e.clone(),
        })
    }
}

/// One tick's row: P2.2a's row on the loop instance, and the loop step's readouts (LOOPS-RULES
/// §8.3, §8.5).
#[derive(Debug, Clone, PartialEq)]
pub struct LoopRow {
    /// P2.2a's row: prices, observables, targets, gaps, the markets' lines, coins, outputs, the
    /// stock records, the dead and idle flags and the reservation's readouts.
    pub row: Row,
    /// Each planted desk's plant record after the tick, in [`Instance::planted`] order.
    pub plants: Vec<PlantState>,
    /// Whether each market was dead this tick: it did not trade or cleared below half its
    /// target (the horse market's is always false: it is idle, not dead).
    pub dead_by: Vec<bool>,
    /// Labour supply's reach, z = ln1p(w/P_s)/χ_max at the tick's posted prices (decision 277);
    /// every head works at z ≥ 1.
    pub supply_ratio: f64,
    /// The horse-days the good desk (the tasks) and the fodder desk bought this tick: their
    /// classes' filled buys of the horse-days.
    pub hours: [f64; 2],
}

impl LoopRow {
    /// The row as a CSV line: P2.2a's columns, then each plant's held, target, order and built,
    /// then `supply_ratio`, `hours_tasks` and `hours_fodder`.
    pub fn csv(&self) -> String {
        let mut v = vec![self.row.csv()];
        for p in &self.plants {
            for x in [p.held, p.target, p.order, p.built] {
                v.push(format!("{x:?}"));
            }
        }
        v.push(format!("{:?}", self.supply_ratio));
        v.push(format!("{:?}", self.hours[0]));
        v.push(format!("{:?}", self.hours[1]));
        v.join(",")
    }
}

/// The CSV header of [`LoopRow::csv`], for a setup: P2.2a's columns (HORSES-RULES §5), then
/// `plant.<d>.held`, `.target`, `.order`, `.built` for each planted desk, `supply_ratio`,
/// `hours_tasks` and `hours_fodder` (LOOPS-RULES §8.5).
pub fn csv_header(s: &Setup) -> String {
    let inst = &s.instance;
    let obs = observables(s);
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
    for d in inst.planted() {
        for f in ["held", "target", "order", "built"] {
            h.push(format!("plant.{d}.{f}"));
        }
    }
    h.push("supply_ratio".into());
    h.push("hours_tasks".into());
    h.push("hours_fodder".into());
    h.join(",")
}

/// The ids a run reads.
struct Ids {
    /// Each market's good, in market order.
    goods: Vec<GoodId>,
    /// Each basket item's good.
    items: Vec<GoodId>,
    coin: GoodId,
    hours: GoodId,
    /// Each actor, in actor order.
    actors: Vec<ActorId>,
    /// The households' classes: owners (the provider's) and workers.
    classes: [ClassId; 2],
    /// The horse-days' buyers' classes: the good desks and the fodder desks.
    buyers: [ClassId; 2],
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
            hours: good(HOURS)?,
            actors: inst
                .actors()
                .iter()
                .map(|a| actor(a))
                .collect::<Result<_, _>>()?,
            classes: [class("owners")?, class("workers")?],
            buyers: [class("good_desks")?, class("fodder_desks")?],
        })
    }
}

/// Labour supply's reach at posted prices (decision 277): the workers' own rule's
/// z = ln1p(w/P_s)/χ_max, before its `min(1)`, P_s = (0.0 + Σ weight·p) over the basket in its
/// order, from the resolved spec and the params in force. Read here, outside the Sim.
fn supply_ratio(sim: &Sim, price_of: &dyn Fn(GoodId) -> Option<f64>) -> Result<f64, String> {
    let clock = &sim.world().clock;
    let par = |s: Site| -> Result<f64, String> {
        let v = sim
            .param(s.param)
            .ok_or_else(|| format!("no param {}", s.param))?;
        s.convert(clock, v).map_err(|e| e.to_string())
    };
    let pr = |g: GoodId| price_of(g).ok_or_else(|| format!("no posted price for {g}"));
    for a in &sim.world().actors {
        if let Spec::BasketWorkers(bw) = &a.spec {
            let mut ps = 0.0;
            for it in &bw.basket {
                ps += par(it.weight)? * pr(it.good)?;
            }
            return Ok(num::ln1p(pr(bw.labour)? / ps) / par(bw.chi_max)?);
        }
    }
    Err("the world has no basket workers".into())
}

/// The capacity desk's records: its output, and the stock records P2.2a's statistics read.
fn read_capacity(c: &CapacityState, output: &mut Vec<f64>, st: &mut Stocks) {
    output.push(c.output);
    st.tasks = c.held;
    st.target = c.target;
    st.order = c.order;
    st.run = c.run;
}

/// The maker's records, with the heads it holds: its output, its serving stock (0 under rule B)
/// and its finished heads, its holding less its record.
fn read_maker(m: &MakerState, horses: f64, output: &mut Vec<f64>, st: &mut Stocks) {
    output.push(m.output);
    st.maker = m.serving;
    st.own = m.own;
    let h = horses - m.own;
    st.finished = if h > 0.0 { h } else { 0.0 };
}

#[allow(clippy::too_many_arguments)]
fn row(
    sim: &Sim,
    s: &Setup,
    ids: &Ids,
    r: &TickReport,
    o: &Obs,
    target: &Target,
    kappa: f64,
    delta: f64,
    horse_market: Option<usize>,
) -> Result<LoopRow, String> {
    let inst = &s.instance;
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
    let held = |a: ActorId, g: GoodId| sim.holding(Holder::Actor(a)).map_or(0.0, |inv| inv.get(g));
    let horse = horse_market.map(|m| ids.goods[m]);
    let mut planned = Vec::with_capacity(1);
    let mut used = Vec::with_capacity(1);
    let mut output = Vec::with_capacity(desks.len());
    let mut plants = Vec::new();
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
        let horses = horse.map_or(0.0, |g| held(a, g));
        match (desks[d].as_str(), sim.actor_state(a)) {
            ("good", Some(ActorState::GoodDesk(s))) => {
                planned.push(s.share);
                used.push(s.used);
                output.push(s.output);
            }
            ("capacity", Some(ActorState::Capacity(c))) => read_capacity(c, &mut output, &mut st),
            ("capacity", Some(ActorState::PlantedCapacity(c))) => {
                read_capacity(&c.desk, &mut output, &mut st);
                plants.push(c.plant);
            }
            ("maker", Some(ActorState::Maker(m))) => read_maker(m, horses, &mut output, &mut st),
            ("maker", Some(ActorState::PlantedMaker(m))) => {
                read_maker(&m.desk, horses, &mut output, &mut st);
                plants.push(m.plant);
            }
            ("fodder" | HOURS, Some(ActorState::MachDesk(m))) => output.push(m.output),
            ("fodder" | HOURS, Some(ActorState::PlantedType(m))) => {
                output.push(m.desk.output);
                plants.push(m.plant);
            }
            _ => return Err(format!("actor {a} is not the desk its instance says")),
        }
    }
    if plants.len() != inst.planted().len() {
        return Err(format!(
            "{} planted states where the instance plants {}",
            plants.len(),
            inst.planted().len()
        ));
    }
    let provider = ids.actors[ids.actors.len() - 2];
    let transfer = match o.transfers.iter().find(|(a, _, _)| *a == provider) {
        Some(&(_, due, paid)) => [due, paid],
        None => return Err("provider is not a provider".into()),
    };
    let coin: Vec<f64> = ids.actors.iter().map(|&a| held(a, ids.coin)).collect();
    // Each household's baskets, as P2.2a's harness reads them.
    let weights = [1.0, inst.county.space];
    let mut baskets = [0.0; 2];
    let mut binding = [None; 2];
    let bought = |g: GoodId, class: ClassId| {
        r.rationing
            .iter()
            .filter(|l| l.good == g && l.class == class && l.side == SideTag::Buy)
            .map(|l| l.filled)
            .fold(0.0, |a, b| a + b)
    };
    for (h, &class) in ids.classes.iter().enumerate() {
        let mut best: Option<(f64, usize)> = None;
        for (i, &g) in ids.items.iter().enumerate() {
            if weights[i] <= 0.0 {
                continue;
            }
            let n_i = num::max_scale(bought(g, class), weights[i]).map_err(|e| e.to_string())?;
            if best.is_none_or(|(b, _)| n_i < b) {
                best = Some((n_i, i));
            }
        }
        if let Some((b, i)) = best {
            baskets[h] = b;
            binding[h] = Some(i);
        }
    }
    let hours = [
        bought(ids.hours, ids.buyers[0]),
        bought(ids.hours, ids.buyers[1]),
    ];
    let price_of = |g: GoodId| ids.goods.iter().position(|x| *x == g).map(|i| price[i]);
    let [w, rr] = [price[0], price[1]];
    // A horse-day's running cost and full cost at posted prices, from the tape's running recipe
    // (LOOPS-RULES §8.3).
    if let Some(m) = horse_market {
        let o_run = running_cost(sim, &price_of)?.unwrap_or(f64::NAN);
        st.running = o_run;
        st.full = o_run + delta * price[m] / kappa;
    }
    let supply_ratio = supply_ratio(sim, &price_of)?;
    let mut obs = vec![w / rr];
    obs.extend(price[2..].iter().map(|p| p / rr));
    obs.extend(used.iter().copied());
    obs.extend(cleared.iter().copied());
    obs.extend(output.iter().copied());
    if !inst.is_flow() {
        obs.push(st.tasks);
    }
    if s.theta < 1.0 {
        obs.extend(plants.iter().map(|p| p.held));
    }
    let base = &target.base;
    let gap: Vec<f64> = obs
        .iter()
        .zip(&base.obs)
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
    let dead_by: Vec<bool> = (0..n)
        .map(|m| {
            Some(m) != horse_market && (!trades[m] || cleared[m] < LIVE_FLOOR * base.volume[m])
        })
        .collect();
    let dead = dead_by.iter().any(|&d| d);
    let (idle, no_order) = match horse_market {
        Some(m) => (cleared[m] < LIVE_FLOOR * base.volume[m], demand[m] == 0.0),
        None => (false, false),
    };
    Ok(LoopRow {
        row: Row {
            tick: r.tick,
            price,
            obs,
            target: base.obs.clone(),
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
        },
        plants,
        dead_by,
        supply_ratio,
        hours,
    })
}

/// One plant's readouts over the scored ticks (LOOPS-RULES §8.3).
#[derive(Debug, Clone, PartialEq)]
pub struct PlantStat {
    /// The desk, `d` of `plant.<d>`.
    pub desk: String,
    /// The scored tick from which |P/P\* − 1| ≤ 0.05 holds to the end, if it does.
    pub in_five: Option<u64>,
    /// P/P\*'s lowest.
    pub low: f64,
    /// P/P\*'s highest.
    pub high: f64,
}

/// The loop step's readouts of a run (LOOPS-RULES §8.3), beside P2.2a's statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct Readouts {
    /// Scored ticks on which each market did not trade or cleared below half its target, in
    /// market order (decision 274); the horse market's is 0 (it is idle, not dead).
    pub dead_by: Vec<u64>,
    /// Ticks of the whole run, those before a dated shock included, at labour supply's bound
    /// (z ≥ 1; decision 277).
    pub bound_ticks: u64,
    /// z's largest over the whole run (0 before the first tick).
    pub bound_peak: f64,
    /// The lowest over the scored ticks of the horse-days the tasks bought over Y·J(x\*), and of
    /// the fodder desk's over 2·q_f (NaN where that target is 0).
    pub hours_low: [f64; 2],
    /// The horse price's highest over its target (NaN in the flow control).
    pub pk_high: f64,
    /// Each plant's readouts, in [`Instance::planted`] order; empty at θ = 1.
    pub plants: Vec<PlantStat>,
}

/// The streaming state of [`Readouts`].
struct Track {
    out: Readouts,
    last_out: Vec<Option<u64>>,
    seen: u64,
    horse: Option<usize>,
}

impl Track {
    fn new(s: &Setup, markets: usize, horse: Option<usize>) -> Track {
        let planted = if s.theta < 1.0 {
            s.instance.planted()
        } else {
            Vec::new()
        };
        Track {
            last_out: vec![None; planted.len()],
            out: Readouts {
                dead_by: vec![0; markets],
                bound_ticks: 0,
                bound_peak: 0.0,
                hours_low: [f64::INFINITY; 2],
                pk_high: if horse.is_some() {
                    f64::NEG_INFINITY
                } else {
                    f64::NAN
                },
                plants: planted
                    .into_iter()
                    .map(|desk| PlantStat {
                        desk,
                        in_five: None,
                        low: f64::INFINITY,
                        high: f64::NEG_INFINITY,
                    })
                    .collect(),
            },
            seen: 0,
            horse,
        }
    }

    /// Every tick of the run: labour supply's bound.
    fn every(&mut self, row: &LoopRow) {
        let z = row.supply_ratio;
        if z.is_nan() || z > self.out.bound_peak {
            self.out.bound_peak = z;
        }
        self.out.bound_ticks += u64::from(z >= 1.0);
    }

    /// Each scored tick.
    fn scored(&mut self, row: &LoopRow, t: &Target, scored: u64) {
        self.seen += 1;
        for (m, &d) in row.dead_by.iter().enumerate() {
            self.out.dead_by[m] += u64::from(d);
        }
        for (i, &target) in t.hours.iter().enumerate() {
            if target > 0.0 {
                self.out.hours_low[i] = min2(self.out.hours_low[i], row.hours[i] / target);
            }
        }
        if let Some(m) = self.horse {
            let rel = (row.row.price[m] / row.row.price[1]) / t.point.pk;
            let hi = &mut self.out.pk_high;
            if !hi.is_nan() && (rel.is_nan() || rel > *hi) {
                *hi = rel;
            }
        }
        for (i, p) in self.out.plants.iter_mut().enumerate() {
            let rel = row.plants[i].held / t.plants[i];
            p.low = min2(p.low, rel);
            if !p.high.is_nan() && (rel.is_nan() || rel > p.high) {
                p.high = rel;
            }
            if rel.is_nan() || (rel - 1.0).abs() > 0.05 {
                self.last_out[i] = Some(scored);
            }
        }
    }

    fn finish(mut self) -> Readouts {
        for (i, p) in self.out.plants.iter_mut().enumerate() {
            p.in_five = match self.last_out[i] {
                None if self.seen > 0 => Some(0),
                None => None,
                Some(k) if k + 1 < self.seen => Some(k + 1),
                Some(_) => None,
            };
        }
        for x in self.out.hours_low.iter_mut() {
            if x.is_infinite() {
                *x = f64::NAN;
            }
        }
        self.out
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
    /// P2.2a's §7.11 statistics.
    pub stats: Stats,
    /// The loop step's readouts.
    pub readouts: Readouts,
    /// The last row.
    pub last: Option<LoopRow>,
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
    each: &mut dyn FnMut(&LoopRow),
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
    let names = observables(&setup);
    let markets = inst.markets();
    let (kappa, delta) = inst.per_tick(setup.tpy)?;
    let horse_market = markets.iter().position(|m| m == "horse");
    let class_names: Vec<String> = sim.world().classes.iter().map(|k| k.to_string()).collect();
    let engine_markets: Vec<String> = {
        let w = sim.world();
        w.markets()
            .map(|(_, g)| w.key_of(g).map_or_else(|| g.to_string(), |k| k.to_string()))
            .collect()
    };
    // The target for each land factor in force.
    let mut targets: Vec<(u64, Target)> = Vec::new();
    let mut target_at = |tick: u64| -> Result<Target, String> {
        let b = setup.b_at(tick);
        if let Some((_, t)) = targets.iter().find(|(k, _)| *k == b.to_bits()) {
            return Ok(t.clone());
        }
        let i = setup.instance_at(tick);
        let t = Target::of(&setup, &i, &i.point(setup.tpy)?, land)?;
        targets.push((b.to_bits(), t.clone()));
        Ok(t)
    };
    let clock_start = pert.clock_start(ticks);
    let start = pert.start();
    let t0 = target_at(clock_start)?;
    let n_prices = markets.len() - 1;
    // D̂_0 (PROBE-SPEC §4.5; HORSES-SPEC §7.5): a stock or coin displacement reads its first
    // year's largest D̂, set below.
    let first_year = matches!(start, Start::Stocks);
    let mut d0 = match start {
        Start::Hold | Start::Nominal | Start::Stocks => 0.0,
        Start::Prices => {
            let mut genesis_obs = vec![g.prices[0] / g.prices[1]];
            genesis_obs.extend(g.prices[2..].iter().map(|p| p / g.prices[1]));
            genesis_obs.push(g.share);
            genesis_obs
                .iter()
                .zip(&t0.base.obs)
                .map(|(o, t)| ln(o / t).abs())
                .fold(0.0, f64::max)
                / TOL_FLOOR
        }
        Start::Shock => shock_distance(&t0.base.point, &base_point(&inst, &g.point, kappa)),
    };
    let year = u64::from(setup.tpy);
    let total = clock_start + ticks;
    let mut classifier = Classifier::new(ticks, names.len(), n_prices);
    let mut stats = Stats::sized(markets.len(), inst.desks().len(), items().len());
    let mut track = StockTrack::new(horse_market, !inst.is_flow(), kappa, delta, nine(&setup));
    let mut readouts = Track::new(&setup, markets.len(), horse_market);
    let maker = if inst.is_flow() {
        None
    } else {
        the_maker(&sim)
    };
    if !inst.is_flow() {
        let (k0, k1) = (g.point.hours / kappa, t0.base.heads);
        stats.stock.paper = if k1 >= k0 {
            1.0
        } else {
            ln(k1 / k0) / num::ln1p(-delta)
        };
    }
    let mut stop = Stop::Ran;
    let mut last: Option<LoopRow> = None;
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
        let mut lr = row(
            &sim,
            &setup,
            &ids,
            &report,
            &o,
            &target,
            kappa,
            delta,
            horse_market,
        )?;
        if let Some(m) = &maker {
            let price = &lr.row.price;
            let price_of = |g: GoodId| ids.goods.iter().position(|x| *x == g).map(|i| price[i]);
            let res = maker_readout(&sim, m, &price_of)?;
            lr.row.markup = res.markup;
            lr.row.withheld = res.withholds;
        }
        each(&lr);
        readouts.every(&lr);
        let row = &lr.row;
        if hold_failure.is_none() {
            hold_failure = hold_check(row, &names, &markets);
        }
        if row.tick >= clock_start {
            let scored = row.tick - clock_start;
            if first_year && scored < year {
                d0 = d0.max(row.dhat);
            }
            let logs: Vec<f64> = row.obs.iter().map(|&x| ln(x)).collect();
            classifier.push(row.dhat, &logs, row.dead);
            stats.push(row, &o, &target.base, &class_names);
            track.push(&mut stats.stock, row, &target.base, scored);
            readouts.scored(&lr, &target, scored);
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
        last = Some(lr);
        if let Some(why) = away {
            stop = Stop::Runaway(why);
            break;
        }
    }
    track.finish(&mut stats.stock);
    if matches!(start, Start::Shock) {
        let y1 = target_at(total.saturating_sub(1))?.base.baskets;
        let y0 = g.point.y;
        let low = stats.baskets.0 * target_at(clock_start)?.base.baskets;
        stats.depth = Some((ln(y1 / y0), ln(low / y0), ln(low / y1)));
    }
    let r_end = last.as_ref().map_or(f64::NAN, |r| r.row.price[1]) / g.prices[1];
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
        readouts: readouts.finish(),
        last,
        hold_failure,
        engine_markets,
    })
}

fn joined(parts: Vec<String>) -> String {
    if parts.is_empty() {
        "-".into()
    } else {
        parts.join(";")
    }
}

/// The nine columns the loop step appends to `summary.tsv` (LOOPS-RULES §8.5): `dead_by`
/// (`market:count` for each market but the horse's), `bound_ticks`, `bound_peak`,
/// `hours_tasks_low`, `hours_fodder_low`, `pk_high`, `plants_in_5pct_from`, `plants_low` and
/// `plants_high` (`desk:value` for each plant, or `-`).
pub fn readout_fields(rec: &Record) -> Vec<String> {
    let r = &rec.readouts;
    let markets = rec.setup.instance.markets();
    let dead_by = markets
        .iter()
        .zip(&r.dead_by)
        .filter(|(m, _)| *m != "horse")
        .map(|(m, n)| format!("{m}:{n}"))
        .collect();
    let nan_dash = |x: f64| if x.is_nan() { "-".to_string() } else { g(x) };
    vec![
        joined(dead_by),
        r.bound_ticks.to_string(),
        g(r.bound_peak),
        nan_dash(r.hours_low[0]),
        nan_dash(r.hours_low[1]),
        nan_dash(r.pk_high),
        joined(
            r.plants
                .iter()
                .map(|p| format!("{}:{}", p.desk, opt(p.in_five)))
                .collect(),
        ),
        joined(
            r.plants
                .iter()
                .map(|p| format!("{}:{}", p.desk, g(p.low)))
                .collect(),
        ),
        joined(
            r.plants
                .iter()
                .map(|p| format!("{}:{}", p.desk, g(p.high)))
                .collect(),
        ),
    ]
}

/// The run's `summary.tsv` line: P2.2a's 50 columns and the loop step's nine.
pub fn summary_line(rec: &Record) -> String {
    let mut v = summary_fields(
        &rec.name,
        &rec.summary,
        &rec.stats,
        &rec.setup.instance.markets(),
        &rec.hold_failure,
        &rec.stop,
    );
    v.extend(readout_fields(rec));
    v.join("\t")
}

/// The run's `stats.tsv` lines: P2.2a's statistics in long form, then the loop step's
/// (`dead.by_market`, `bound.*`, `hours.*`, `glut.pk_high`, `plant.<d>.*`).
pub fn stats_lines(rec: &Record) -> Vec<String> {
    let inst = &rec.setup.instance;
    let markets = inst.markets();
    let mut out = stats_fields(
        &rec.stats,
        &markets,
        &outputs(inst),
        &items(),
        &rec.engine_markets,
    );
    let r = &rec.readouts;
    let mut put = |stat: &str, at: &str, value: String| {
        out.push((stat.to_string(), at.to_string(), value));
    };
    for (m, n) in markets.iter().zip(&r.dead_by) {
        if m != "horse" {
            put("dead.by_market", m, n.to_string());
        }
    }
    put("bound.ticks", "-", r.bound_ticks.to_string());
    put("bound.peak", "-", g(r.bound_peak));
    put("hours.tasks_low", "-", g(r.hours_low[0]));
    put("hours.fodder_low", "-", g(r.hours_low[1]));
    put("glut.pk_high", "-", g(r.pk_high));
    for p in &r.plants {
        put(
            &format!("plant.{}.in_5pct_from", p.desk),
            "-",
            opt(p.in_five),
        );
        put(&format!("plant.{}.low", p.desk), "-", g(p.low));
        put(&format!("plant.{}.high", p.desk), "-", g(p.high));
    }
    out.into_iter()
        .map(|(stat, at, value)| format!("{}\t{stat}\t{at}\t{value}", rec.name))
        .collect()
}

/// The config's name, for the reports.
pub fn config_name(inst: &Instance) -> &'static str {
    match inst.config {
        Config::Wet => "wet",
        Config::Flow => "flow",
    }
}
