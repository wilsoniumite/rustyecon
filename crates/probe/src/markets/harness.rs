//! The markets harness (MARKETS-SPEC §7): runs a setup on the engine, reads §7.1's observables
//! every tick from the `TickReport` and the actors' own state, scores them against the oracle's
//! values for the same tick (solved here, outside the Sim, at the coefficients in force), and
//! classifies the run as PROBE-SPEC §4.5 does. It keeps §7.11's transient statistics (O14) as it
//! goes, so a run of any length holds a fixed amount of memory.
//!
//! The observables are v = w/r, each type's and each category's price over r, each category
//! desk's human share, the cleared volume of every market, and each desk's output (§7.1). No
//! dial is a band, so tol_o is the floor, 1e-3, on every one (§7.3). The oracle-free measures
//! are certify's, as P2.0's harness reads them (C11): `Obs` for the tick, the runaway bound,
//! rationed fills, the NaN-keeping folds and troughs, and the dead-share rule.
//!
//! On I0 the observables, their order, the targets, D̂ and the classes are P2.0's
//! (`crate::harness`), which stays as it was and pinned to docs/probe/results/.
//!
//! **At the wall** (a worker-form instance, P2.3; docs/probe/WALL-RULES.md §5): the observables
//! add each reserved wage over r after the prices, and read each desk's threshold x_j = 1 − s_j
//! in place of s_j, whose target 0 has no log (decision 396); the reserved labour markets' volumes
//! are among the volumes. Every household (the provider, the workers and each reserved pop) has
//! its baskets. The wall's own readouts (the depth, breach ticks, each desk's largest share, each
//! pop's participation range and saturated ticks) are kept beside §7.11's statistics, reported and
//! never scored.
//!
//! **At the commons** (an open-commons instance, P2.3; docs/probe/COMMONS-RULES.md §5): the
//! targets are unit 1e's, labour's volume the supply S and land's the enclosed land T of the
//! instance in force; a cost shock's start distance is read over every observable, as the wall's.
//! Each tick the regime of the plots, the shadow rent over r and the plots' rented land are read
//! through the workers' own rule (`workers_participation`) at the tick's posted prices and the
//! params in force, so the readouts cannot part from what the workers did; they are kept beside
//! §7.11's statistics, reported and never scored.
//!
//! **At a paced commons instance** (P2.4; docs/probe/TRAP-RULES.md §4): the workers' state share
//! is the paced share, the hours offered over N; the rule's share F\* at the tick's posted prices
//! is read beside it (`part_target`), and the lag and the ticks without hours are kept as the
//! pace's readouts (`pace.*`), reported and never scored.
//!
//! **At a switch instance** (P2.4; docs/probe/SWITCH-RULES.md §4): the observables add each switch
//! pop's reserved share 1 − a after the tick (`rs.<type>`), whose target is 1 − a\* from unit 1d;
//! each reserved market's volume target is its reserved hours D_i. Each switch pop's pool share
//! and its gap g = ln(ε·w/w_i) at the tick's posted prices, through the rule's own function, are
//! read each tick and kept as the switch's readouts (`switch.*`), reported and never scored.

use super::instance::{Instance, Point};
use super::perturb::Perturbation;
use super::setup::{genesis, tape_ron, Genesis, Setup};
use crate::harness::{Class, Stop, Summary};
use crate::perturb::Start;
use crate::protocol::{DEAD_SHARE, HOLD_TOL, LIVE_FLOOR, RUNAWAY, TOL_FLOOR};
use certify::battery::{dead_share_ok, rationed, within_bound};
use certify::fold::{max2, min2};
use certify::Obs;
use rustyecon_core::num;
use rustyecon_core::num::ln;
use rustyecon_core::{CoreError, FlowPerYear, Site};
use rustyecon_engine::prelude::{
    ActorId, ClassId, GoodId, Holder, PriceError, RunErrorKind, SideTag, Sim, Tape, TickReport,
};
use rustyecon_engine::rustyecon_agents::{
    switch_gap, workers_participation, ActorState, AgentError, BasketWorkers, Participation,
    PlotRegime, Pool, Spec,
};
use std::collections::BTreeMap;

/// The observables of an instance, in order (MARKETS-SPEC §7.1): `v`, `pi.<type>`,
/// `pi.<category>`, `s.<category>`, `vol.<market>` in market order, `y.<category>`, `y.<type>`.
/// On I0 this is P2.0's set O in P2.0's order. At the wall each reserved wage `w.<type>` follows
/// the prices, and `x.<category>` replaces `s.<category>` (the wall frame's §5.1). At a switch
/// instance each switch pop's reserved share `rs.<type>` comes last (P2.4; the switch scan's §3.9).
pub fn observables(inst: &Instance) -> Vec<String> {
    let mut o = vec!["v".to_string()];
    o.extend(inst.types.iter().map(|t| format!("pi.{}", t.key)));
    o.extend(inst.categories.iter().map(|c| format!("pi.{}", c.key)));
    o.extend(inst.wtypes.iter().map(|t| format!("w.{}", t.key)));
    let technique = if inst.worker_form { "x" } else { "s" };
    o.extend(
        inst.categories
            .iter()
            .map(|c| format!("{technique}.{}", c.key)),
    );
    o.extend(inst.markets().iter().map(|m| format!("vol.{m}")));
    o.extend(inst.categories.iter().map(|c| format!("y.{}", c.key)));
    o.extend(inst.types.iter().map(|t| format!("y.{}", t.key)));
    if inst.switch {
        o.extend(inst.wtypes.iter().map(|t| format!("rs.{}", t.key)));
    }
    o
}

/// The number of prices among the observables: v, each type's, each category's and each reserved
/// wage (the classifier's band reads them).
pub fn price_count(inst: &Instance) -> usize {
    1 + inst.types.len() + inst.categories.len() + inst.wtypes.len()
}

/// γ(1) = η·(g0 + g1), the schedule at the top task (k's power of 1 is 1).
pub fn gamma_top(inst: &Instance) -> f64 {
    inst.eta * (inst.g0 + inst.g1)
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
    /// Each household's baskets, in [`Instance::households`] order: the provider's, the
    /// workers', then each reserved pop's.
    pub households: Vec<f64>,
    /// Each basket item's quantity eaten, z_j·Y, in item order (space last).
    pub items: Vec<f64>,
    /// Each desk's output, in desk order.
    pub output: Vec<f64>,
    /// The point.
    pub point: Point,
}

impl Target {
    /// The target of an equilibrium with land services `land` per tick: land clears T, each
    /// category's good z_j·Y, and each type's market what the category desks and the other
    /// types buy of it (a type's own input is kept, not traded; MARKETS-SPEC §1.4).
    pub fn of(inst: &Instance, e: &Point, land: f64) -> Target {
        // The pool's hours clear `labour` (N_a in unit 1c, n_D in 1d); each reserved type's
        // market clears its reserved hours D_i (the wall frame's §5.1), 1d's `reserved_hours`,
        // which is its `hours` at a wall bit for bit and less where it pools (P2.4).
        let mut volume = vec![e.pool, land];
        volume.extend(e.type_traded.iter().copied());
        volume.extend(e.cat_output.iter().copied());
        if inst.worker_form {
            volume.extend(e.reserved.iter().copied());
        }
        let mut output = e.cat_output.clone();
        output.extend(e.type_services.iter().copied());
        let mut obs = vec![e.v];
        obs.extend(e.type_price.iter().copied());
        obs.extend(e.cat_price.iter().copied());
        obs.extend(e.wage.iter().copied());
        // The technique: 1 − x* as P2.1 reads it, or the threshold x* = 1 − (1 − x*) at the wall.
        let technique = if inst.worker_form {
            1.0 - e.one_minus_x
        } else {
            e.one_minus_x
        };
        obs.extend(inst.categories.iter().map(|_| technique));
        obs.extend(volume.iter().copied());
        obs.extend(output.iter().copied());
        // Each switch pop's reserved share at the point, 1 − a* (1 at its wall; P2.4).
        if inst.switch {
            obs.extend(e.pool_share.iter().map(|a| 1.0 - a));
        }
        let mut items = e.cat_output.clone();
        if let Some(h) = inst.space {
            items.push(h * e.y);
        }
        let mut households = vec![e.provider_baskets];
        households.extend(e.pop_baskets.iter().copied());
        Target {
            obs,
            volume,
            baskets: e.y,
            households,
            items,
            output,
            point: e.clone(),
        }
    }
}

/// The distance between two oracle points (MARKETS-SPEC §1.5): the largest |ln| over 1 − x,
/// v, every price, Y, every type's services and N_a, over the tolerance.
pub fn shock_distance(a: &Point, b: &Point) -> f64 {
    let mut pairs = vec![
        (a.one_minus_x, b.one_minus_x),
        (a.v, b.v),
        (a.y, b.y),
        (a.n_a, b.n_a),
    ];
    pairs.extend(
        a.type_price
            .iter()
            .copied()
            .zip(b.type_price.iter().copied()),
    );
    pairs.extend(a.cat_price.iter().copied().zip(b.cat_price.iter().copied()));
    pairs.extend(
        a.type_services
            .iter()
            .copied()
            .zip(b.type_services.iter().copied()),
    );
    pairs
        .iter()
        .map(|(x, y)| ln(x / y).abs())
        .fold(0.0, f64::max)
        / TOL_FLOOR
}

/// The distance between two targets at the wall: the largest |ln| over every observable, over
/// the tolerance, as the wall frame's mirror measures a cost shock's start (`wb.run`).
pub fn target_distance(a: &Target, b: &Target) -> f64 {
    a.obs
        .iter()
        .zip(&b.obs)
        .map(|(x, y)| ln(x / y).abs())
        .fold(0.0, f64::max)
        / TOL_FLOOR
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
    /// Each category desk's planned human share after the tick.
    pub planned: Vec<f64>,
    /// Each desk's output this tick, in desk order.
    pub output: Vec<f64>,
    /// The provider's transfer this tick: due and paid (the sums over its transfers).
    pub transfer: [f64; 2],
    /// The baskets each household ate, in [`Instance::households`] order: min_j of what it
    /// bought of each item over z_j, as its rule eats them.
    pub baskets: Vec<f64>,
    /// The item that bound each household's baskets, if it bought any item.
    pub binding: Vec<Option<usize>>,
    /// Each worker pop's participation this tick, its hours' share of its heads, the pool's
    /// workers first (its own state; read at the wall).
    pub participation: Vec<f64>,
    /// The wall's depth at the tick's posted prices, ln(θ·w/(p_τ·γ(1))) (the wall frame's
    /// §5.3): below 0 the machine comparison pins the wage again.
    pub depth: f64,
    /// At the commons, the plots as the workers' rule read them this tick.
    pub plots: Option<Plots>,
    /// At a switch instance, each switch pop's pool share a after the tick, in pop order (P2.4).
    pub pool: Vec<f64>,
    /// At a switch instance, each switch pop's gap g = ln(ε·w/w_i) at the prices this tick
    /// settled at, through the rule's own `switch_gap` (P2.4).
    pub switch_gap: Vec<f64>,
    /// The tick's ledger margin.
    pub margin: f64,
    /// Whether the tick is dead: some market did not trade, or cleared less than the live
    /// floor of its oracle volume.
    pub dead: bool,
}

/// The workers' plots on one tick (the commons frame's §3.8): the regime, the shadow rent over r
/// and the enclosed land rented, as their rule formed them at the tick's posted prices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plots {
    /// Where the plots stand.
    pub regime: PlotRegime,
    /// r_o/r: 0 on a commons with room, the shadow rent when crowded (or at the split, the rent
    /// at which a plot stops paying), 1 when the plots spill onto enclosed land.
    pub rent: f64,
    /// T_p, the enclosed land the plots rent.
    pub rented: f64,
    /// The rule's share F\* = hours/N at the tick's posted prices, the share paced workers move
    /// toward (P2.4; the trap scan's §5.4 `part_target`); the rule's own share without a pace.
    pub target: f64,
}

/// The shadow rent r_o of the workers' plots, a readout that nobody receives (the commons
/// frame's §3.8, `cm.shadow_rent`): 0 with room or no plot, r when the plots spill onto enclosed
/// land, and when the commons is full (or at the split) the plot rent at which the rule's supply
/// is its hours: F(e) = hours/N at e = (w − k·P_s)/(1 + k), k = expm1(χ_max·hours/N), and
/// r_o = (p_g·s₀ − e)/h, at most r.
pub fn shadow_rent(p: &Participation, chi: f64, w: f64, pg: f64, s0: f64, h: f64, r: f64) -> f64 {
    match p.regime {
        PlotRegime::Unused | PlotRegime::Commons => 0.0,
        PlotRegime::Enclosed => r,
        PlotRegime::Crowded | PlotRegime::Split => {
            let k = num::expm1(p.hours / p.heads * chi);
            let e = (w - k * p.basket_price) / (1.0 + k);
            ((pg * s0 - e) / h).min(r)
        }
    }
}

/// The workers' plots as their rule forms them from `sim`'s params in force and the posted
/// prices `price_of` (by good), with the shadow rent; `None` without an exit.
pub fn plots_now(
    sim: &Sim,
    workers: &BasketWorkers,
    price_of: &dyn Fn(GoodId) -> Option<f64>,
) -> Result<Option<Plots>, String> {
    let Some(x) = &workers.exit else {
        return Ok(None);
    };
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
    let Some(p) = workers_participation(workers, &par, &pr).map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let f = |e: AgentError| e.to_string();
    let (w, pg, r) = (
        pr(workers.labour).map_err(f)?,
        pr(x.good).map_err(f)?,
        pr(x.land).map_err(f)?,
    );
    let chi = par(workers.chi_max).map_err(f)?;
    let (s0, h) = (par(x.gross).map_err(f)?, par(x.plot).map_err(f)?);
    Ok(Some(Plots {
        regime: p.regime,
        rent: shadow_rent(&p, chi, w, pg, s0, h, r) / r,
        rented: p.plots,
        target: p.hours / p.heads,
    }))
}

/// The regime's name as the frame's mirror labels it (`cm.exit_rule`): the split is `Crowded`.
pub fn mirror_regime(r: PlotRegime) -> &'static str {
    match r {
        PlotRegime::Unused => "Unused",
        PlotRegime::Commons => "Commons",
        PlotRegime::Crowded | PlotRegime::Split => "Crowded",
        PlotRegime::Enclosed => "Enclosed",
    }
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
    for m in &markets {
        for f in ["S", "D", "bfill", "sfill", "spoiled"] {
            h.push(format!("{m}_{f}"));
        }
    }
    for it in items(inst) {
        h.push(format!("eaten_{it}"));
    }
    for a in inst.actors() {
        h.push(format!("coin_{a}"));
    }
    for c in &inst.categories {
        h.push(format!("s_planned_{}", c.key));
    }
    h.push("transfer_due".into());
    h.push("transfer_paid".into());
    let households = inst.households();
    for k in &households {
        h.push(format!("baskets_{k}"));
    }
    for k in &households {
        h.push(format!("bound_{k}"));
    }
    h.push("ledger_margin".into());
    h.push("dead".into());
    if inst.worker_form {
        for k in &households[1..] {
            h.push(format!("part_{k}"));
        }
        h.push("depth".into());
    }
    if inst.exit.is_some() {
        for k in ["part_workers", "regime", "ro_over_r", "plots_rented"] {
            h.push(k.into());
        }
    }
    // The rule's share beside the paced share (P2.4), at a paced instance only.
    if inst.paced_exit() {
        h.push("part_target".into());
    }
    // Each switch pop's pool share and gap (P2.4), at a switch instance only.
    if inst.switch {
        for k in &households[2..] {
            h.push(format!("pool_{k}"));
        }
        for k in &households[2..] {
            h.push(format!("swgap_{k}"));
        }
    }
    h.join(",")
}

/// The basket's items by good, in item order: each category's good, then land if households
/// buy space.
pub fn items(inst: &Instance) -> Vec<String> {
    let mut v: Vec<String> = inst.categories.iter().map(|c| c.key.clone()).collect();
    if inst.space.is_some() {
        v.push("land".into());
    }
    v
}

impl Row {
    /// The row as a CSV line, every float in Rust's shortest round-trip form.
    pub fn csv(&self, inst: &Instance) -> String {
        let mut v = vec![self.tick.to_string()];
        let mut push = |x: f64| v.push(format!("{x:?}"));
        self.price.iter().for_each(|&x| push(x));
        self.obs.iter().for_each(|&x| push(x));
        self.target.iter().for_each(|&x| push(x));
        self.gap.iter().for_each(|&x| push(x));
        push(self.dhat);
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
        self.baskets.iter().for_each(|&x| push(x));
        let names = items(inst);
        for b in &self.binding {
            v.push(b.map_or("-".to_string(), |i| names[i].clone()));
        }
        v.push(format!("{:?}", self.margin));
        v.push(u8::from(self.dead).to_string());
        if inst.worker_form {
            for x in &self.participation {
                v.push(format!("{x:?}"));
            }
            v.push(format!("{:?}", self.depth));
        }
        if inst.exit.is_some() {
            v.push(format!(
                "{:?}",
                self.participation.first().copied().unwrap_or(f64::NAN)
            ));
            match &self.plots {
                Some(p) => {
                    v.push(format!("{:?}", p.regime));
                    v.push(format!("{:?}", p.rent));
                    v.push(format!("{:?}", p.rented));
                }
                None => v.extend(["-".to_string(), "-".into(), "-".into()]),
            }
        }
        if inst.paced_exit() {
            v.push(
                self.plots
                    .as_ref()
                    .map_or("-".to_string(), |p| format!("{:?}", p.target)),
            );
        }
        if inst.switch {
            for x in self.pool.iter().chain(&self.switch_gap) {
                v.push(format!("{x:?}"));
            }
        }
        v.join(",")
    }
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
    /// The households' classes, in [`Instance::households`] order: owners (the provider's),
    /// workers, then each reserved pop's.
    classes: Vec<ClassId>,
    /// The number of desks: the actors before the households.
    desks: usize,
    /// The pool's workers' resolved spec, where they hold a priced exit (the commons).
    exit_workers: Option<BasketWorkers>,
    /// Each reserved pop's pool, where it switches (P2.4), in pop order.
    pools: Vec<Option<Pool>>,
}

impl Ids {
    fn of(sim: &Sim, inst: &Instance) -> Result<Ids, String> {
        let w = sim.world();
        let good = |k: &str| w.id_of::<GoodId>(k).ok_or(format!("no good {k}"));
        let actor = |k: &str| w.id_of::<ActorId>(k).ok_or(format!("no actor {k}"));
        let class = |k: &str| w.id_of::<ClassId>(k).ok_or(format!("no class {k}"));
        let exit_workers = w
            .actors
            .iter()
            .find(|a| a.key.as_str() == "workers")
            .and_then(|a| match &a.spec {
                Spec::BasketWorkers(p) if p.exit.is_some() => Some(p.clone()),
                _ => None,
            });
        let pools = inst
            .wtypes
            .iter()
            .map(|t| {
                w.actors
                    .iter()
                    .find(|a| a.key.as_str() == t.pop())
                    .and_then(|a| match &a.spec {
                        Spec::BasketWorkers(p) => p.pool,
                        _ => None,
                    })
            })
            .collect();
        Ok(Ids {
            exit_workers,
            pools,
            desks: inst.desks().len(),
            goods: inst
                .markets()
                .iter()
                .map(|m| good(m))
                .collect::<Result<_, _>>()?,
            items: items(inst)
                .iter()
                .map(|m| good(m))
                .collect::<Result<_, _>>()?,
            coin: good("coin")?,
            actors: inst
                .actors()
                .iter()
                .map(|a| actor(a))
                .collect::<Result<_, _>>()?,
            classes: inst
                .household_classes()
                .iter()
                .map(|k| class(k))
                .collect::<Result<_, _>>()?,
        })
    }
}

fn row(
    sim: &Sim,
    inst: &Instance,
    ids: &Ids,
    r: &TickReport,
    o: &Obs,
    target: &Target,
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
    let nc = inst.categories.len();
    let mut planned = Vec::with_capacity(nc);
    let mut used = Vec::with_capacity(nc);
    let mut output = Vec::with_capacity(ids.desks);
    for (d, &a) in ids.actors.iter().take(ids.desks).enumerate() {
        match sim.actor_state(a) {
            Some(ActorState::GoodDesk(s)) if d < nc => {
                planned.push(s.share);
                used.push(s.used);
                output.push(s.output);
            }
            Some(ActorState::MachDesk(s)) if d >= nc => output.push(s.output),
            _ => return Err(format!("actor {a} is not the desk its instance says")),
        }
    }
    let mut participation = Vec::with_capacity(ids.actors.len() - ids.desks - 1);
    let mut pool = Vec::new();
    for &a in &ids.actors[ids.desks + 1..] {
        match sim.actor_state(a) {
            Some(ActorState::Workers(s)) => participation.push(s.share),
            // A switch pop (P2.4): its participation F, and its pool share a.
            Some(ActorState::SwitchWorkers(s)) => {
                participation.push(s.share);
                pool.push(s.pool);
            }
            _ => return Err(format!("actor {a} is not the pop its instance says")),
        }
    }
    let provider = ids.actors[ids.desks];
    let transfer = match o.transfers.iter().find(|(a, _, _)| *a == provider) {
        Some(&(_, due, paid)) => [due, paid],
        None => return Err("provider is not a provider".into()),
    };
    let coin: Vec<f64> = ids
        .actors
        .iter()
        .map(|&a| {
            sim.holding(Holder::Actor(a))
                .map_or(0.0, |inv| inv.get(ids.coin))
        })
        .collect();
    // Each household's baskets, from what its class bought of each item this tick: it holds
    // nothing else of them when it eats (every item lives one tick), and its rule eats
    // min_j max_scale(held_j, z_j).
    let weights: Vec<f64> = {
        let mut z: Vec<f64> = inst.categories.iter().map(|c| c.weight).collect();
        if let Some(h) = inst.space {
            z.push(h);
        }
        z
    };
    let mut baskets = vec![0.0; ids.classes.len()];
    let mut binding = vec![None; ids.classes.len()];
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
    let mut obs = vec![w / rr];
    // Every other price over r, in market order: the types, the categories, then the reserved
    // wages (the observables' order).
    obs.extend(price[2..].iter().map(|p| p / rr));
    if inst.worker_form {
        obs.extend(used.iter().map(|s| 1.0 - s));
    } else {
        obs.extend(used.iter().copied());
    }
    obs.extend(cleared.iter().copied());
    obs.extend(output.iter().copied());
    // Each switch pop's reserved share after the tick, 1 − a (P2.4), and its gap at the posted
    // prices this tick settled at, with ε in force: the pool's wage is `labour`'s, its own the
    // reserved market's (the markets' order: labour first, the reserved labour markets last).
    let mut gaps = Vec::new();
    if inst.switch {
        obs.extend(pool.iter().map(|a| 1.0 - a));
        let clock = &sim.world().clock;
        let first = n - inst.wtypes.len();
        for (i, p) in ids.pools.iter().enumerate() {
            if let Some(p) = p {
                let v = sim
                    .param(p.efficiency.param)
                    .ok_or("the pool's efficiency is not a param")?;
                let eps = p.efficiency.convert(clock, v).map_err(|e| e.to_string())?;
                gaps.push(switch_gap(eps, price[0], price[first + i]));
            }
        }
    }
    // The depth at the posted prices this tick settled at (the wall frame's §5.3).
    let tau = inst.task_type()?;
    let depth = ln((inst.types[tau].theta * w) / (price[2 + tau] * gamma_top(inst)));
    // The plots, through the workers' own rule at the prices this tick settled at.
    let plots = match &ids.exit_workers {
        Some(p) => {
            let price_of = |g: GoodId| o.markets.iter().find(|l| l.good == g).map(|l| l.price);
            plots_now(sim, p, &price_of)?
        }
        None => None,
    };
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
    let dead = (0..n).any(|m| !trades[m] || cleared[m] < LIVE_FLOOR * target.volume[m]);
    Ok(Row {
        tick: r.tick,
        price,
        obs,
        target: target.obs.clone(),
        gap,
        dhat,
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
        participation,
        depth,
        plots,
        pool,
        switch_gap: gaps,
        margin: o.margin,
        dead,
    })
}

/// Mode A's per-tick check (PROBE-SPEC §4.6, MARKETS-SPEC §7.6), with fills and spoilage read
/// relative to volume, as P2.0's: every gap at most 1e-9, every market trading with both fills
/// at least 1 − 1e-9, and no produced good spoiling beyond 1e-9 of its volume.
fn hold_check(row: &Row, names: &[String], markets: &[String], produced: usize) -> Option<String> {
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
        // Produced goods only: the types' and the categories' (a reserved type's hours, like
        // the pool's, are not produced).
        if (2..produced).contains(&m) && row.spoiled[m] > HOLD_TOL * row.cleared[m] {
            return Some(format!(
                "tick {}: {:e} of {market} spoiled",
                row.tick, row.spoiled[m]
            ));
        }
    }
    None
}

/// The streaming classifier: what PROBE-SPEC §4.5's classes read, kept tick by tick over the
/// scored run, with P2.0's definitions (`crate::harness::classify`). The stocks probe's harness
/// (`crate::horses::harness`) classifies with it too.
#[derive(Debug, Clone)]
pub(crate) struct Classifier {
    l: usize,
    seen: usize,
    envelope: [f64; 4],
    max_w: f64,
    max_f: f64,
    dead: u64,
    dead_w: u64,
    dead_f: u64,
    /// Per observable, the NaN-keeping (min, max) of ln o over F.
    rest: Vec<Option<(f64, f64)>>,
    /// For v and each price over r, the (min, max) of ln o over W.
    band: Vec<(f64, f64)>,
    last_out: Option<usize>,
    first_out_in_w: Option<usize>,
    last: f64,
}

impl Classifier {
    pub(crate) fn new(l: u64, n_obs: usize, n_prices: usize) -> Classifier {
        Classifier {
            l: l as usize,
            seen: 0,
            envelope: [0.0; 4],
            max_w: 0.0,
            max_f: 0.0,
            dead: 0,
            dead_w: 0,
            dead_f: 0,
            rest: vec![None; n_obs],
            band: vec![(f64::INFINITY, f64::NEG_INFINITY); n_prices],
            last_out: None,
            first_out_in_w: None,
            last: f64::NAN,
        }
    }

    pub(crate) fn push(&mut self, dhat: f64, ln_obs: &[f64], dead: bool) {
        let i = self.seen;
        self.seen += 1;
        let l = self.l;
        for (j, e) in self.envelope.iter_mut().enumerate() {
            if j * l / 4 <= i && i < (j + 1) * l / 4 {
                *e = e.max(dhat);
            }
        }
        let (w0, f0) = (l / 2, l * 9 / 10);
        if i < l {
            if i >= w0 {
                self.max_w = self.max_w.max(dhat);
                if dead {
                    self.dead_w += 1;
                }
                if dhat > 1.0 && self.first_out_in_w.is_none() {
                    self.first_out_in_w = Some(i);
                }
                for (b, &x) in self.band.iter_mut().zip(ln_obs) {
                    *b = (b.0.min(x), b.1.max(x));
                }
            }
            if i >= f0 {
                self.max_f = self.max_f.max(dhat);
                if dead {
                    self.dead_f += 1;
                }
                for (r, &x) in self.rest.iter_mut().zip(ln_obs) {
                    *r = Some(match *r {
                        None => (x, x),
                        Some((lo, hi)) => (min2(lo, x), max2(hi, x)),
                    });
                }
            }
        }
        if dead {
            self.dead += 1;
        }
        if dhat > 1.0 {
            self.last_out = Some(i);
        }
        self.last = dhat;
    }

    pub(crate) fn summary(&self, stop: &Stop, d0: f64, start: Start, r_end: f64) -> Summary {
        let l = self.l;
        let complete = self.seen >= l;
        let quarter = |j: usize| {
            let (a, b) = (j * l / 4, ((j + 1) * l / 4).min(self.seen));
            if a < b {
                self.envelope[j]
            } else {
                f64::NAN
            }
        };
        let envelope = [quarter(0), quarter(1), quarter(2), quarter(3)];
        let (w0, _) = (l / 2, l * 9 / 10);
        let max_w = if complete { self.max_w } else { f64::NAN };
        let max_f = if complete { self.max_f } else { f64::NAN };
        let kappa = if d0 > 0.0 { max_f / d0 } else { f64::NAN };
        let (dead_w, dead_f) = if complete {
            (self.dead_w, self.dead_f)
        } else {
            (0, 0)
        };
        let at_rest = complete
            && self
                .rest
                .iter()
                .all(|r| r.is_some_and(|(lo, hi)| hi - lo <= TOL_FLOOR / 10.0));
        let band = if complete {
            let mut b = [f64::NAN; 3];
            for (i, bi) in b.iter_mut().enumerate() {
                if let Some(&(lo, hi)) = self.band.get(i) {
                    *bi = num::exp(hi - lo);
                }
            }
            b
        } else {
            [f64::NAN; 3]
        };
        let in_tol_from = match self.last_out {
            None if self.seen > 0 => Some(0),
            None => None,
            Some(k) if k + 1 < self.seen => Some((k + 1) as u64),
            Some(_) => None,
        };
        let nominal = matches!(start, Start::Nominal | Start::Hold);
        let (class, why) = match stop {
            Stop::Error(e) => (Class::Error, e.clone()),
            Stop::Runaway(e) => (Class::Diverged, e.clone()),
            Stop::Ran => {
                let [_, e2, e3, e4] = envelope;
                if e4 > e3 && e3 > e2 && e4 > d0.max(1.0) {
                    (
                        Class::Diverged,
                        "the envelope grows: E4 > E3 > E2".to_string(),
                    )
                } else if !dead_share_ok(dead_w, (l - w0) as u64, dead_f, DEAD_SHARE) {
                    (Class::Dead, String::new())
                } else if !nominal && d0 <= 1.0 {
                    (Class::Vacuous, String::new())
                } else if max_w <= 1.0 && at_rest && (nominal || kappa < 1.0) {
                    (Class::Converged, String::new())
                } else if at_rest {
                    (Class::Stuck, String::new())
                } else {
                    (Class::Orbiting, String::new())
                }
            }
        };
        Summary {
            class,
            why,
            d0,
            envelope,
            kappa,
            max_w,
            last: self.last,
            in_tol_from,
            dead: [self.dead, dead_w, dead_f],
            band,
            r_end,
            first_out_in_w: if complete {
                self.first_out_in_w.map(|i| i as u64)
            } else {
                None
            },
        }
    }
}

/// A per-(market, class, side) rationing record (MARKETS-SPEC §7.11; CERTIFY §6's reports).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ration {
    /// The worst filled/feasible over lines with something feasible.
    pub worst: f64,
    /// Ticks with that fill below 1 − 1e-9.
    pub ticks: u64,
    /// Σ(requested − feasible): budget rationing.
    pub budget: f64,
    /// Σ(feasible − filled): market rationing.
    pub market: f64,
}

/// The transient statistics of MARKETS-SPEC §7.11 (O14), over the scored run (from the shock
/// for a dated shock), against the oracle's values at the coefficients in force. Reported, never
/// scored.
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
    /// Buyer fills: the worst buyer and seller fill over every market (sides that exist).
    pub worst_fill: f64,
    /// For a cost shock: ln(Y′/Y) of the equilibrium change, and the trough of baskets eaten
    /// in log against Y and against Y′.
    pub depth: Option<(f64, f64, f64)>,
    /// The wall's own readouts, at a worker-form instance (the wall frame's §5.3).
    pub wall: Option<Wall>,
    /// The commons' readouts, at an open-commons instance (the commons frame's §3.8).
    pub commons: Option<CommonsStats>,
    /// The pace's readouts, at a paced instance (P2.4; the trap scan's §5.4).
    pub pace: Option<PaceStats>,
    /// Each switch pop's readouts, at a switch instance (P2.4; the switch scan's §3.9), in pop
    /// order.
    pub switch: Option<Vec<SwitchStats>>,
}

/// A switch pop's readouts over the scored run (P2.4; the switch scan's §3.9, the mirror's
/// `swb.run`), reported and never scored.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchStats {
    /// The scored ticks with its pool share above 1e-9.
    pub live: u64,
    /// Its largest pool share (0 if it never pools).
    pub max: f64,
    /// The sign changes of its gap g between scored ticks with |g| above 1e-6 (the band keeps
    /// rounding-level flips near a pooled rest out of the count).
    pub band: u64,
    last: i8,
    /// Its pool share on the last scored tick.
    pub end: f64,
    /// Its gap on the last scored tick.
    pub gap_end: f64,
}

impl SwitchStats {
    fn new() -> SwitchStats {
        SwitchStats {
            live: 0,
            max: 0.0,
            band: 0,
            last: 0,
            end: f64::NAN,
            gap_end: f64::NAN,
        }
    }

    fn push(&mut self, a: f64, g: f64) {
        self.live += u64::from(a > 1e-9);
        self.max = max2(self.max, a);
        if g.abs() > 1e-6 {
            let sign = if g > 0.0 { 1 } else { -1 };
            if self.last != 0 && sign != self.last {
                self.band += 1;
            }
            self.last = sign;
        }
        self.end = a;
        self.gap_end = g;
    }
}

/// The pace's readouts over the scored run (P2.4; the trap scan's §5.4), reported and never
/// scored: how far the paced share F (the workers' state, the hours offered over N) lags the
/// rule's share F\* at the tick's posted prices, and the ticks with no hours offered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaceStats {
    /// The largest |ln(F/F\*)| (0 where both are 0, infinite where one alone is).
    pub gap_max: f64,
    /// The least F/F\* (1 where both are 0, infinite where F\* alone is).
    pub low: f64,
    /// The ticks with labour's supply 0: no hours offered.
    pub zero_hours: u64,
}

impl PaceStats {
    fn new() -> PaceStats {
        PaceStats {
            gap_max: 0.0,
            low: f64::INFINITY,
            zero_hours: 0,
        }
    }

    fn push(&mut self, row: &Row) {
        if let (Some(&f), Some(p)) = (row.participation.first(), &row.plots) {
            let ratio = if p.target > 0.0 {
                f / p.target
            } else if f == 0.0 {
                1.0
            } else {
                f64::INFINITY
            };
            let gap = if ratio > 0.0 {
                ln(ratio).abs()
            } else {
                f64::INFINITY
            };
            self.gap_max = max2(self.gap_max, gap);
            self.low = min2(self.low, ratio);
        }
        if let Some(&s) = row.supply.first() {
            self.zero_hours += u64::from(s <= 0.0);
        }
    }
}

/// The commons' readouts over the scored run (the commons frame's §3.8; `battery_c.run`'s),
/// reported and never scored: the plots' regime tick by tick, the shadow rent over r, the plots'
/// rented land, and the provider's coin, each read through the workers' own rule.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonsStats {
    /// Ticks in each regime: unused, Commons, Crowded, Enclosed, and the split, apart.
    pub ticks: [u64; 5],
    /// Switches between the mirror's labels (`mirror_regime`: the split counted as Crowded).
    pub switches: u64,
    last: Option<&'static str>,
    /// The regime on the last tick, the mirror's label.
    pub end: String,
    /// The target's regime on the last tick, unit 1e's `ExitLand` by name.
    pub star: String,
    /// r_o/r: the lowest, the highest and the last.
    pub rent: (f64, f64, f64),
    /// The target's r_o over r = 1 on the last tick.
    pub rent_star: f64,
    /// The largest T_p.
    pub rented_max: f64,
    /// The provider's lowest coin after a tick over its genesis coin (1 if it never falls).
    pub provider_coin_low: f64,
    provider_genesis: f64,
}

impl CommonsStats {
    fn new(provider_genesis: f64) -> CommonsStats {
        CommonsStats {
            ticks: [0; 5],
            switches: 0,
            last: None,
            end: String::new(),
            star: String::new(),
            rent: (f64::INFINITY, f64::NEG_INFINITY, f64::NAN),
            rent_star: f64::NAN,
            rented_max: 0.0,
            provider_coin_low: 1.0,
            provider_genesis,
        }
    }

    fn push(&mut self, row: &Row, t: &Target, provider: usize) {
        if let Some(p) = &row.plots {
            let k = match p.regime {
                PlotRegime::Unused => 0,
                PlotRegime::Commons => 1,
                PlotRegime::Crowded => 2,
                PlotRegime::Enclosed => 3,
                PlotRegime::Split => 4,
            };
            self.ticks[k] += 1;
            let label = mirror_regime(p.regime);
            if self.last.is_some_and(|l| l != label) {
                self.switches += 1;
            }
            self.last = Some(label);
            self.end = label.to_string();
            self.rent = (min2(self.rent.0, p.rent), max2(self.rent.1, p.rent), p.rent);
            self.rented_max = max2(self.rented_max, p.rented);
        }
        if let Some(c) = &t.point.commons {
            self.star = c.regime.clone();
            self.rent_star = c.plot_rent;
        }
        if let Some(&c) = row.coin.get(provider) {
            self.provider_coin_low = min2(self.provider_coin_low, c / self.provider_genesis);
        }
    }
}

/// The wall's own readouts over the scored run (the wall frame's §5.3; decision 396), reported
/// and never scored.
#[derive(Debug, Clone, PartialEq)]
pub struct Wall {
    /// The lowest depth ln(θ·w/(p_τ·γ(1))) at posted prices, and its tick.
    pub depth_min: (f64, u64),
    /// Breach ticks: the depth below 0.
    pub breach: u64,
    /// The first breach tick.
    pub first_breach: Option<u64>,
    /// The longest run of consecutive breach ticks.
    pub breach_run: u64,
    run: u64,
    /// Each category desk's largest human share used.
    pub share_max: Vec<f64>,
    /// Each worker pop's participation range (lowest, highest), the pool's workers first.
    pub participation: Vec<(f64, f64)>,
    /// Each worker pop's saturated ticks, all its heads at work (F = 1: the support of χ).
    pub saturated: Vec<u64>,
    /// The worst buyer fill over every market, as the wall frame's mirror reads it.
    pub worst_buyer_fill: f64,
}

impl Wall {
    fn new(inst: &Instance) -> Wall {
        Wall {
            depth_min: (f64::INFINITY, 0),
            breach: 0,
            first_breach: None,
            breach_run: 0,
            run: 0,
            share_max: vec![0.0; inst.categories.len()],
            participation: vec![(f64::INFINITY, f64::NEG_INFINITY); inst.wtypes.len() + 1],
            saturated: vec![0; inst.wtypes.len() + 1],
            worst_buyer_fill: 1.0,
        }
    }

    fn push(&mut self, row: &Row) {
        Stats::trough(&mut self.depth_min, row.tick, row.depth);
        if row.depth < 0.0 {
            self.breach += 1;
            self.run += 1;
            self.breach_run = self.breach_run.max(self.run);
            if self.first_breach.is_none() {
                self.first_breach = Some(row.tick);
            }
        } else {
            self.run = 0;
        }
        // The share each desk decided this tick and produced at (its state's `share`, which is
        // its `used` after production).
        for (m, &s) in self.share_max.iter_mut().zip(&row.planned) {
            *m = max2(*m, s);
        }
        for (i, &f) in row.participation.iter().enumerate() {
            let r = &mut self.participation[i];
            *r = (min2(r.0, f), max2(r.1, f));
            self.saturated[i] += u64::from(f >= 1.0);
        }
        for m in 0..row.buyer_fill.len() {
            self.worst_buyer_fill = min2(self.worst_buyer_fill, row.buyer_fill[m]);
        }
    }
}

impl Stats {
    fn new(inst: &Instance) -> Stats {
        let (nm, nd, ni) = (inst.markets().len(), inst.desks().len(), items(inst).len());
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
            wall: inst.worker_form.then(|| Wall::new(inst)),
            commons: None,
            pace: inst.paced_exit().then(PaceStats::new),
            switch: inst
                .switch
                .then(|| vec![SwitchStats::new(); inst.wtypes.len()]),
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

    fn push(&mut self, row: &Row, o: &Obs, t: &Target, class_names: &[String]) {
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
        // Every household's baskets: the provider's and the workers', then each reserved pop's
        // at the wall (P2.3.10), in `Instance::households` order, so Y\* is what they all eat.
        // With two households this is the old sum, bit for bit.
        let total = row.baskets[2..]
            .iter()
            .fold(row.baskets[0] + row.baskets[1], |a, &b| a + b);
        let mut slot = (self.baskets.0, self.baskets.1);
        Stats::trough(&mut slot, row.tick, total / t.baskets);
        self.baskets = (slot.0, slot.1, self.baskets.2 + u64::from(total == 0.0));
        for (i, (&e, &z)) in row.eaten.iter().zip(&t.items).enumerate() {
            let mut slot = (self.item[i].0, 0);
            Stats::trough(&mut slot, row.tick, e / z);
            self.item[i] = (slot.0, self.item[i].1 + u64::from(e == 0.0));
        }
        for h in 0..row.baskets.len() {
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
        if let Some(w) = self.wall.as_mut() {
            w.push(row);
        }
        if let Some(c) = self.commons.as_mut() {
            // The provider's coin: the first household, after the desks.
            c.push(row, t, t.output.len());
        }
        if let Some(p) = self.pace.as_mut() {
            p.push(row);
        }
        if let Some(s) = self.switch.as_mut() {
            for ((st, &a), &g) in s.iter_mut().zip(&row.pool).zip(&row.switch_gap) {
                st.push(a, g);
            }
        }
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
    let c = crate::setup::clock(setup.tpy)?;
    // Land clears the enclosed land T of the instance in force (it moves under `enclose`, P2.3).
    let land = c.flow(FlowPerYear(setup.instance.land));
    let names = observables(&inst);
    let markets = inst.markets();
    let class_names: Vec<String> = sim.world().classes.iter().map(|k| k.to_string()).collect();
    let engine_markets: Vec<String> = {
        let w = sim.world();
        w.markets()
            .map(|(_, g)| w.key_of(g).map_or_else(|| g.to_string(), |k| k.to_string()))
            .collect()
    };
    // The target for each set of coefficients in force.
    let mut targets: Vec<(Vec<(String, f64)>, Target)> = Vec::new();
    let mut target_at = |tick: u64| -> Result<Target, String> {
        let key = setup.changes_at(tick);
        if let Some((_, t)) = targets.iter().find(|(k, _)| *k == key) {
            return Ok(t.clone());
        }
        let i = setup.instance_at(tick)?;
        let t = Target::of(&i, &i.point(setup.tpy)?, c.flow(FlowPerYear(i.land)));
        targets.push((key, t.clone()));
        Ok(t)
    };
    let clock_start = pert.clock_start(ticks);
    let start = pert.start();
    let t0 = target_at(clock_start)?;
    let n_prices = price_count(&inst);
    let produced = 2 + inst.types.len() + inst.categories.len();
    // Tier 3S (decision 229; the wall frame's §5.2): a stock or coin start reads the largest D̂
    // of its first scored year, set below.
    let first_year = setup.first_year_d0 && matches!(start, Start::Stocks);
    let mut d0 = match start {
        Start::Hold | Start::Nominal => 0.0,
        Start::Prices => {
            let mut genesis_obs = vec![g.prices[0] / g.prices[1]];
            genesis_obs.extend(g.prices[2..].iter().map(|p| p / g.prices[1]));
            if inst.worker_form {
                genesis_obs.extend(g.shares.iter().map(|s| 1.0 - s));
            } else {
                genesis_obs.extend(g.shares.iter().copied());
            }
            let prices = genesis_obs
                .iter()
                .zip(&t0.obs)
                .map(|(o, t)| ln(o / t).abs())
                .fold(0.0, f64::max);
            // Each switch pop's reserved share at genesis against its target's, 1 − a against
            // 1 − a* (P2.4; the switch scan's `swb.d0`); none elsewhere.
            g.switch
                .iter()
                .zip(&t0.point.pool_share)
                .map(|(a, at)| ln((1.0 - a) / (1.0 - at)).abs())
                .fold(prices, f64::max)
                / TOL_FLOOR
        }
        Start::Stocks if first_year => 0.0,
        Start::Stocks => {
            // A paced share's displacement (P2.4) counts as a stock's factor, its genesis share
            // over the point's S/N; elsewhere there is none.
            let pace = g.pace.map_or(0.0, |(at, share)| ln(share / at).abs());
            pert.stock_factors()
                .iter()
                .map(|f| ln(*f).abs())
                .fold(pace, f64::max)
                / TOL_FLOOR
        }
        Start::Shock if inst.worker_form || inst.exit.is_some() => {
            target_distance(&t0, &Target::of(&inst, &g.point, land))
        }
        Start::Shock => shock_distance(&t0.point, &g.point),
    };
    let year = u64::from(setup.tpy);
    let total = clock_start + ticks;
    let mut classifier = Classifier::new(ticks, names.len(), n_prices);
    let mut stats = Stats::new(&inst);
    if inst.exit.is_some() {
        stats.commons = Some(CommonsStats::new(g.coin[ids.desks]));
    }
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
        let row = row(&sim, &inst, &ids, &report, &o, &target)?;
        each(&row);
        if hold_failure.is_none() {
            hold_failure = hold_check(&row, &names, &markets, produced);
        }
        if row.tick >= clock_start {
            if first_year && row.tick - clock_start < year {
                d0 = d0.max(row.dhat);
            }
            let logs: Vec<f64> = row.obs.iter().map(|&x| ln(x)).collect();
            classifier.push(row.dhat, &logs, row.dead);
            stats.push(&row, &o, &target, &class_names);
        }
        // The runaway bound (PROBE-SPEC §4.5): every posted price within [1e-6, 1e6] times its
        // genesis value, by certify's relative bound (A12).
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
