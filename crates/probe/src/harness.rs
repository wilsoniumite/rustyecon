//! The harness: runs a setup on the engine, reads PROBE-SPEC §4.1's observables every tick from
//! the `TickReport` and the actors' own state, scores them against the oracle's values for the
//! same tick (solved here, outside the Sim), and classifies the run (§4.5).
//!
//! Every tolerance is PROBE-SPEC's (see [`crate::protocol`]). The designs registered here have no
//! dead bands, so every R_o is 0 and tol_o is the floor, 1e-3, on every observable; a design
//! with bands would need the wedge solver of §3.3 (a), which is not built.

use crate::perturb::{Perturbation, Start};
use crate::protocol::{HOLD_TOL, LIVE_FLOOR, RUNAWAY, TOL_FLOOR};
use crate::setup::{equilibrium, genesis, tape_ron, Genesis, Setup, ACTORS};
use oracle::Eq1a;
use rustyecon_core::num;
use rustyecon_core::num::ln;
use rustyecon_engine::prelude::{
    ActorId, GoodId, Holder, PriceError, Provenance, RunErrorKind, Sim, Tape, TickReport,
};
use rustyecon_engine::rustyecon_agents::ActorState;

/// The observables, in order (PROBE-SPEC §4.3's set O).
pub const OBSERVABLES: [&str; 10] = [
    "v", "pi_m", "pi", "s", "labour", "land", "mach", "good", "q_good", "q_mach",
];

/// The markets, in order.
pub const MARKETS: [&str; 4] = ["labour", "land", "mach", "good"];

/// The oracle's values of the observables at one b, relative to r = 1, per tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// The observables' values, in [`OBSERVABLES`] order.
    pub obs: [f64; 10],
}

impl Target {
    /// The target of an equilibrium with land services `land` per tick: under the registered
    /// self-use and space choices, land clears T and machine services Y·J (PROBE-SPEC §1.6).
    pub fn of(e: &Eq1a, land: f64) -> Target {
        Target {
            obs: [
                e.v,
                e.p_m,
                e.p,
                e.one_minus_x_star,
                e.n_a,
                land,
                e.y * e.j_star,
                e.y,
                e.y,
                e.k,
            ],
        }
    }

    /// The cleared volume of market `m`, in [`MARKETS`] order.
    pub fn volume(&self, m: usize) -> f64 {
        self.obs[4 + m]
    }
}

/// The distance between two oracle points: the largest |ln| over 1 − x, v, p_m, p, Y, K and
/// N_a (PROBE-SPEC §4.7), over the tolerance.
pub fn shock_distance(a: &Eq1a, b: &Eq1a) -> f64 {
    [
        (a.one_minus_x_star, b.one_minus_x_star),
        (a.v, b.v),
        (a.p_m, b.p_m),
        (a.p, b.p),
        (a.y, b.y),
        (a.k, b.k),
        (a.n_a, b.n_a),
    ]
    .iter()
    .map(|(x, y)| ln(x / y).abs())
    .fold(0.0, f64::max)
        / TOL_FLOOR
}

/// One tick's row: what the engine reported, the agents' own records, and the target.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The tick.
    pub tick: u64,
    /// The posted prices this tick settled at: w, r, p_m, p.
    pub price: [f64; 4],
    /// The observables, in [`OBSERVABLES`] order.
    pub obs: [f64; 10],
    /// The target, in the same order.
    pub target: [f64; 10],
    /// Each observable's gap |ln(o/o\*)|; infinite when o is not positive.
    pub gap: [f64; 10],
    /// D̂: the largest gap over its tolerance.
    pub dhat: f64,
    /// Supply per market.
    pub supply: [f64; 4],
    /// Feasible demand per market.
    pub demand: [f64; 4],
    /// The buyers' fill per market.
    pub buyer_fill: [f64; 4],
    /// The sellers' fill per market.
    pub seller_fill: [f64; 4],
    /// Whether each market traded.
    pub trades: [bool; 4],
    /// Spoilage per market good this tick.
    pub spoiled: [f64; 4],
    /// Each actor's coin after the tick, in [`ACTORS`] order.
    pub coin: [f64; 4],
    /// The good desk's planned human share after the tick.
    pub planned: f64,
    /// The provider's transfer this tick: due and paid.
    pub transfer: [f64; 2],
    /// The tick's ledger margin.
    pub margin: f64,
    /// Whether the tick is dead: some market did not trade, or cleared less than the live floor
    /// of its oracle volume.
    pub dead: bool,
}

/// The CSV header of [`Row::csv`].
pub fn csv_header() -> String {
    let mut h = vec!["tick".to_string()];
    for p in ["w", "r", "pm", "p"] {
        h.push(p.to_string());
    }
    for o in OBSERVABLES {
        h.push(o.to_string());
    }
    for o in OBSERVABLES {
        h.push(format!("{o}_star"));
    }
    for o in OBSERVABLES {
        h.push(format!("gap_{o}"));
    }
    h.push("dhat".to_string());
    for m in MARKETS {
        for f in ["S", "D", "bfill", "sfill", "spoiled"] {
            h.push(format!("{m}_{f}"));
        }
    }
    for a in ACTORS {
        h.push(format!("coin_{a}"));
    }
    for f in [
        "s_planned",
        "transfer_due",
        "transfer_paid",
        "ledger_margin",
        "dead",
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
        for m in 0..4 {
            push(self.supply[m]);
            push(self.demand[m]);
            push(self.buyer_fill[m]);
            push(self.seller_fill[m]);
            push(self.spoiled[m]);
        }
        self.coin.iter().for_each(|&x| push(x));
        push(self.planned);
        push(self.transfer[0]);
        push(self.transfer[1]);
        push(self.margin);
        v.push(u8::from(self.dead).to_string());
        v.join(",")
    }
}

/// The ids a run reads.
struct Ids {
    goods: [GoodId; 4],
    coin: GoodId,
    actors: [ActorId; 4],
}

impl Ids {
    fn of(sim: &Sim) -> Result<Ids, String> {
        let w = sim.world();
        let good = |k: &str| w.id_of::<GoodId>(k).ok_or(format!("no good {k}"));
        let actor = |k: &str| w.id_of::<ActorId>(k).ok_or(format!("no actor {k}"));
        Ok(Ids {
            goods: [good("labour")?, good("land")?, good("mach")?, good("good")?],
            coin: good("coin")?,
            actors: [
                actor(ACTORS[0])?,
                actor(ACTORS[1])?,
                actor(ACTORS[2])?,
                actor(ACTORS[3])?,
            ],
        })
    }
}

fn row(sim: &Sim, ids: &Ids, r: &TickReport, target: &Target) -> Result<Row, String> {
    let line = |g: GoodId| {
        r.markets
            .iter()
            .find(|l| l.good == g)
            .ok_or(format!("no market line for {g}"))
    };
    let mut price = [0.0; 4];
    let mut supply = [0.0; 4];
    let mut demand = [0.0; 4];
    let mut cleared = [0.0; 4];
    let mut buyer_fill = [0.0; 4];
    let mut seller_fill = [0.0; 4];
    let mut trades = [false; 4];
    let mut spoiled = [0.0; 4];
    for (m, &g) in ids.goods.iter().enumerate() {
        let l = line(g)?;
        price[m] = l.price;
        supply[m] = l.supply;
        demand[m] = l.demand;
        cleared[m] = l.cleared;
        buyer_fill[m] = l.buyer_fill;
        seller_fill[m] = l.seller_fill;
        trades[m] = l.supply > 0.0 && l.demand > 0.0 && l.buyer_fill > 0.0 && l.seller_fill > 0.0;
        spoiled[m] = -r
            .audit
            .lines
            .iter()
            .filter(|(h, p, _)| *h == g && *p == Provenance::Spoilage)
            .fold(0.0, |acc, (_, _, q)| acc + q);
    }
    let (planned, used, q_good) = match sim.actor_state(ids.actors[0]) {
        Some(ActorState::GoodDesk(s)) => (s.share, s.used, s.output),
        _ => return Err("desk.good is not a good desk".into()),
    };
    let q_mach = match sim.actor_state(ids.actors[1]) {
        Some(ActorState::MachDesk(s)) => s.output,
        _ => return Err("desk.mach is not a machine desk".into()),
    };
    let transfer = match sim.actor_state(ids.actors[2]) {
        Some(ActorState::Provider(s)) => [s.due, s.paid],
        _ => return Err("provider is not a provider".into()),
    };
    let mut coin = [0.0; 4];
    for (c, &a) in coin.iter_mut().zip(&ids.actors) {
        *c = sim
            .holding(Holder::Actor(a))
            .map_or(0.0, |inv| inv.get(ids.coin));
    }
    let [w, rr, pm, p] = price;
    let obs = [
        w / rr,
        pm / rr,
        p / rr,
        used,
        cleared[0],
        cleared[1],
        cleared[2],
        cleared[3],
        q_good,
        q_mach,
    ];
    let mut gap = [0.0; 10];
    for (i, g) in gap.iter_mut().enumerate() {
        *g = if obs[i] > 0.0 && obs[i].is_finite() {
            ln(obs[i] / target.obs[i]).abs()
        } else {
            f64::INFINITY
        };
    }
    let dhat = gap.iter().fold(0.0, |a: f64, &g| a.max(g)) / TOL_FLOOR;
    let dead = (0..4).any(|m| !trades[m] || cleared[m] < LIVE_FLOOR * target.volume(m));
    Ok(Row {
        tick: r.tick,
        price,
        obs,
        target: target.obs,
        gap,
        dhat,
        supply,
        demand,
        buyer_fill,
        seller_fill,
        trades,
        spoiled,
        coin,
        planned,
        transfer,
        margin: r.audit.max_margin,
        dead,
    })
}

/// How a run ended before its last tick.
#[derive(Debug, Clone, PartialEq)]
pub enum Stop {
    /// It ran every tick.
    Ran,
    /// A posted price left [1/RUNAWAY, RUNAWAY] times its genesis value, or the price rule gave a
    /// non-finite price (DIVERGED).
    Runaway(String),
    /// Any other run error, a ledger breach included (ERROR: a bug, not an outcome).
    Error(String),
}

/// A run's record: what the classifier and the summary read.
#[derive(Debug, Clone)]
pub struct Record {
    /// The run's name.
    pub name: String,
    /// The setup it ran.
    pub setup: Setup,
    /// Its genesis.
    pub genesis: Genesis,
    /// How its start distance is measured, and the distance.
    pub start: Start,
    /// D̂_0.
    pub d0: f64,
    /// The tick the scored clock starts at.
    pub clock_start: u64,
    /// The scored length L.
    pub ticks: u64,
    /// How it ended.
    pub stop: Stop,
    /// D̂ per tick, from tick 0.
    pub dhat: Vec<f64>,
    /// ln of each observable per tick, from tick 0.
    pub ln_obs: Vec<[f64; 10]>,
    /// Dead ticks.
    pub dead: Vec<bool>,
    /// Ticks on which each market's buyers, then sellers, were rationed beyond the hold
    /// tolerance: [labour, land, mach, good] × [buyers, sellers].
    pub rationed: [[u64; 2]; 4],
    /// Spoilage per market good over the run.
    pub spoiled: [f64; 4],
    /// The smallest cleared volume of each market over its oracle volume, and the tick.
    pub trough: [(f64, u64); 4],
    /// The provider's transfer shortfall over the run, Σ (due − paid).
    pub transfer_short: f64,
    /// The last row.
    pub last: Option<Row>,
    /// Mode A's check (PROBE-SPEC §4.6): the first tick and observable that failed, if any.
    pub hold_failure: Option<String>,
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
    let text = tape_ron(&setup)?;
    let tape = Tape::from_ron(&text).map_err(|e| format!("the tape does not load: {e}"))?;
    let mut sim = Sim::new(&tape).map_err(|e| format!("the tape does not load: {e}"))?;
    let ids = Ids::of(&sim)?;
    let g = genesis(&setup)?;
    let land = sim
        .world()
        .clock
        .flow(rustyecon_core::FlowPerYear(setup.instance.land));
    // The target for each b in force.
    let mut targets: Vec<(f64, Target)> = Vec::new();
    let mut target_for = |b: f64| -> Result<Target, String> {
        if let Some((_, t)) = targets.iter().find(|(bb, _)| *bb == b) {
            return Ok(*t);
        }
        let t = Target::of(&equilibrium(&setup.instance, b, setup.tpy)?, land);
        targets.push((b, t));
        Ok(t)
    };
    let clock_start = pert.clock_start(ticks);
    let start = pert.start();
    let t0 = target_for(setup.b_at(clock_start))?;
    let d0 = match start {
        Start::Hold | Start::Nominal => 0.0,
        Start::Prices => {
            let [w, r, pm, p] = g.prices;
            let genesis_obs = [w / r, pm / r, p / r, g.share];
            (0..4)
                .map(|i| ln(genesis_obs[i] / t0.obs[i]).abs())
                .fold(0.0, f64::max)
                / TOL_FLOOR
        }
        Start::Stocks => {
            pert.stock_factors()
                .iter()
                .map(|f| ln(*f).abs())
                .fold(0.0, f64::max)
                / TOL_FLOOR
        }
        Start::Shock => shock_distance(
            &equilibrium(&setup.instance, setup.b_at(clock_start), setup.tpy)?,
            &g.point,
        ),
    };
    let total = clock_start + ticks;
    let mut rec = Record {
        name: name.to_string(),
        setup: setup.clone(),
        genesis: g.clone(),
        start,
        d0,
        clock_start,
        ticks,
        stop: Stop::Ran,
        dhat: Vec::with_capacity(total as usize),
        ln_obs: Vec::with_capacity(total as usize),
        dead: Vec::with_capacity(total as usize),
        rationed: [[0; 2]; 4],
        spoiled: [0.0; 4],
        trough: [(f64::INFINITY, 0); 4],
        transfer_short: 0.0,
        last: None,
        hold_failure: None,
    };
    for _ in 0..total {
        let tick = sim.tick();
        let target = target_for(setup.b_at(tick))?;
        let report = match sim.step() {
            Ok(r) => r,
            Err(e) => {
                rec.stop = match &e.kind {
                    RunErrorKind::Price(PriceError::NonFinite { .. }) => {
                        Stop::Runaway(e.to_string())
                    }
                    _ => Stop::Error(e.to_string()),
                };
                break;
            }
        };
        let row = row(&sim, &ids, &report, &target)?;
        each(&row);
        if rec.hold_failure.is_none() {
            rec.hold_failure = hold_check(&row);
        }
        for m in 0..4 {
            if row.buyer_fill[m] < 1.0 - HOLD_TOL {
                rec.rationed[m][0] += 1;
            }
            if row.seller_fill[m] < 1.0 - HOLD_TOL {
                rec.rationed[m][1] += 1;
            }
            rec.spoiled[m] += row.spoiled[m];
            let rel = row.obs[4 + m] / target.volume(m);
            if rel < rec.trough[m].0 {
                rec.trough[m] = (rel, row.tick);
            }
        }
        rec.transfer_short += row.transfer[0] - row.transfer[1];
        rec.dhat.push(row.dhat);
        let mut logs = [0.0; 10];
        for (l, o) in logs.iter_mut().zip(row.obs) {
            *l = ln(o);
        }
        rec.ln_obs.push(logs);
        rec.dead.push(row.dead);
        // The runaway bound (PROBE-SPEC §4.5): every posted price within [1e-6, 1e6] times its
        // genesis value.
        let away = ids.goods.iter().enumerate().find_map(|(m, &gid)| {
            let l = report.markets.iter().find(|l| l.good == gid)?;
            let rel = l.next_price / g.prices[m];
            (!(1.0 / RUNAWAY..=RUNAWAY).contains(&rel)).then(|| {
                format!(
                    "the price of {} left the runaway bound at tick {}: {rel:e} of genesis",
                    MARKETS[m], report.tick
                )
            })
        });
        rec.last = Some(row);
        if let Some(why) = away {
            rec.stop = Stop::Runaway(why);
            break;
        }
    }
    Ok(rec)
}

/// Mode A's per-tick check (PROBE-SPEC §4.6), with fills and spoilage read relative to volume:
/// at an f64 rest S and D differ by rounding, so about 1e-16 of the volume is rationed or
/// spoils each tick (a judges' graft from design-reservation).
fn hold_check(row: &Row) -> Option<String> {
    if let Some(i) = (0..10).find(|&i| row.gap[i].is_nan() || row.gap[i] > HOLD_TOL) {
        return Some(format!(
            "tick {}: {} is {:e} from the oracle in log",
            row.tick, OBSERVABLES[i], row.gap[i]
        ));
    }
    for (m, market) in MARKETS.iter().enumerate() {
        if !row.trades[m] {
            return Some(format!("tick {}: {} did not trade", row.tick, market));
        }
        if row.buyer_fill[m] < 1.0 - HOLD_TOL || row.seller_fill[m] < 1.0 - HOLD_TOL {
            return Some(format!(
                "tick {}: {} filled {:e} (buyers) and {:e} (sellers)",
                row.tick, market, row.buyer_fill[m], row.seller_fill[m]
            ));
        }
        if m >= 2 && row.spoiled[m] > HOLD_TOL * row.obs[4 + m] {
            return Some(format!(
                "tick {}: {:e} of {} spoiled",
                row.tick, row.spoiled[m], market
            ));
        }
    }
    None
}

/// The class of a run (PROBE-SPEC §4.5): the first that applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// A run error other than a non-finite price, or a ledger breach.
    Error,
    /// A non-finite price, the runaway bound, or a growing envelope.
    Diverged,
    /// More than 1% of W dead, or any tick of F.
    Dead,
    /// D̂_0 ≤ 1: the start is inside tolerance (mode B, not nominal).
    Vacuous,
    /// Within tolerance over W, at rest in F, and κ < 1.
    Converged,
    /// At rest in F, outside tolerance.
    Stuck,
    /// Live, bounded, not at rest.
    Orbiting,
}

impl Class {
    /// The class's name as PROBE-SPEC writes it.
    pub fn name(self) -> &'static str {
        match self {
            Class::Error => "ERROR",
            Class::Diverged => "DIVERGED",
            Class::Dead => "DEAD",
            Class::Vacuous => "VACUOUS",
            Class::Converged => "CONVERGED",
            Class::Stuck => "STUCK",
            Class::Orbiting => "ORBITING",
        }
    }
}

/// A run's summary: its class and PROBE-SPEC §4.5's reported numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// The class.
    pub class: Class,
    /// Why, for ERROR and DIVERGED.
    pub why: String,
    /// D̂_0.
    pub d0: f64,
    /// E_1 to E_4: the largest D̂ in each quarter of the scored run.
    pub envelope: [f64; 4],
    /// κ = max over F of D̂ over D̂_0 (NaN when D̂_0 is 0).
    pub kappa: f64,
    /// The largest D̂ over W.
    pub max_w: f64,
    /// The last D̂.
    pub last: f64,
    /// The scored tick after which D̂ stays at most 1, if it does.
    pub in_tol_from: Option<u64>,
    /// Dead ticks: in the whole scored run, in W, in F.
    pub dead: [u64; 3],
    /// July's band per relative price over W: exp(max ln − min ln) for v, π_m, π.
    pub band: [f64; 3],
    /// r at the end over r at genesis.
    pub r_end: f64,
    /// The first scored tick in W at which D̂ exceeded 1.
    pub first_out_in_w: Option<u64>,
}

/// Classify a run.
pub fn classify(rec: &Record) -> Summary {
    let from = rec.clock_start as usize;
    let dh = if rec.dhat.len() > from {
        &rec.dhat[from..]
    } else {
        &[][..]
    };
    let ln = if rec.ln_obs.len() > from {
        &rec.ln_obs[from..]
    } else {
        &[][..]
    };
    let dead = if rec.dead.len() > from {
        &rec.dead[from..]
    } else {
        &[][..]
    };
    let l = rec.ticks as usize;
    let quarter = |j: usize| {
        let (a, b) = (j * l / 4, ((j + 1) * l / 4).min(dh.len()));
        if a < b {
            dh[a..b].iter().fold(0.0, |m: f64, &x| m.max(x))
        } else {
            f64::NAN
        }
    };
    let envelope = [quarter(0), quarter(1), quarter(2), quarter(3)];
    let (w0, f0) = (l / 2, l * 9 / 10);
    let complete = dh.len() >= l;
    let max_of = |s: &[f64]| s.iter().fold(0.0, |m: f64, &x| m.max(x));
    let max_w = if complete {
        max_of(&dh[w0..])
    } else {
        f64::NAN
    };
    let max_f = if complete {
        max_of(&dh[f0..])
    } else {
        f64::NAN
    };
    let kappa = if rec.d0 > 0.0 {
        max_f / rec.d0
    } else {
        f64::NAN
    };
    let dead_w = if complete {
        dead[w0..].iter().filter(|&&d| d).count() as u64
    } else {
        0
    };
    let dead_f = if complete {
        dead[f0..].iter().filter(|&&d| d).count() as u64
    } else {
        0
    };
    // At rest in F: every observable's range of ln o at most tol/10.
    let at_rest = complete
        && (0..10).all(|i| {
            let (lo, hi) = ln[f0..]
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), o| {
                    (lo.min(o[i]), hi.max(o[i]))
                });
            hi - lo <= TOL_FLOOR / 10.0
        });
    let band = {
        let mut b = [f64::NAN; 3];
        if complete {
            for (i, bi) in b.iter_mut().enumerate() {
                let (lo, hi) = ln[w0..]
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), o| {
                        (lo.min(o[i]), hi.max(o[i]))
                    });
                *bi = num::exp(hi - lo);
            }
        }
        b
    };
    let in_tol_from = {
        let mut t = None;
        for (i, &x) in dh.iter().enumerate().rev() {
            if x > 1.0 {
                break;
            }
            t = Some(i as u64);
        }
        t
    };
    let first_out_in_w = if complete {
        dh[w0..]
            .iter()
            .position(|&x| x > 1.0)
            .map(|i| (w0 + i) as u64)
    } else {
        None
    };
    let r_end = rec.last.as_ref().map_or(f64::NAN, |r| r.price[1]) / rec.genesis.prices[1];
    let nominal = matches!(rec.start, Start::Nominal | Start::Hold);
    let (class, why) = match &rec.stop {
        Stop::Error(e) => (Class::Error, e.clone()),
        Stop::Runaway(e) => (Class::Diverged, e.clone()),
        Stop::Ran => {
            let [_, e2, e3, e4] = envelope;
            if e4 > e3 && e3 > e2 && e4 > rec.d0.max(1.0) {
                (
                    Class::Diverged,
                    "the envelope grows: E4 > E3 > E2".to_string(),
                )
            } else if dead_w as f64 > 0.01 * (l - w0) as f64 || dead_f > 0 {
                (Class::Dead, String::new())
            } else if !nominal && rec.d0 <= 1.0 {
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
        d0: rec.d0,
        envelope,
        kappa,
        max_w,
        last: dh.last().copied().unwrap_or(f64::NAN),
        in_tol_from,
        dead: [dead.iter().filter(|&&d| d).count() as u64, dead_w, dead_f],
        band,
        r_end,
        first_out_in_w,
    }
}
