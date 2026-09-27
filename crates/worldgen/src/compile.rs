//! The compiler: the checked tables become one tape (docs/demo/WORLD.md §8; PLAN §3.7).
//!
//! Each county is a node, `county.<chapman>`, quoting in one coin, with the four markets of the
//! Appendix B economy and the probe's four roles at their C2 dials (docs/probe/RULES.md). Its
//! eleven instance params are registered as `county.<c>.<column>`; its history is a dated
//! `SetParam` for every step, copying a schedule param `county.<c>.<column>.<YYYY-MM>` whose
//! basis names the ramps that moved it. Genesis puts every county at its own oracle point, as
//! `tapes/appb.ron` is made (RULES §4): unit 1a at (ρ, δ, J_b) = (0, 1, 1), its prices scaled
//! so that the good costs 1 coin, 1 − x\*, one tick's output held by each desk, and each
//! actor's stationary coin under the dials. The oracle is solved here, outside any `Sim`; no
//! agent reads it (R13).
//!
//! Before it writes a line, the compiler solves every county at genesis and after every step:
//! each solve must be `Interior` with one crossing of n_D − n_S on a 401-point grid, funded,
//! and short of saturated participation. No step date may move the oracle's relative prices
//! and technique, nor its quantities, by more than `max_step` in log, and no trailing year may
//! move either by more than [`MAX_YEAR`] (O14: the probe's roles take gradual change well and
//! abrupt change badly). `max_step` itself may not pass [`tables::MAX_STEP_CEILING`].

use crate::atlas::Atlas;
use crate::history::{self, Month, Step};
use crate::tables::{self, County, Instance, Lens, Param, Parsed, Ramp, Tables, World};
use crate::CompileError;
use oracle::{Economy, Eq1a, Params, PowerSchedule, Regime, UniformWorkCost};
use rustyecon_core::{num, Clock, FlowPerYear, RatePerYear};

/// A compiled world.
#[derive(Debug, Clone, PartialEq)]
pub struct Compiled {
    /// The tape, as RON text with its derivation in comments.
    pub tape: String,
    /// The world's settings and dials.
    pub world: World,
    /// Each county's plan, in key order.
    pub counties: Vec<Plan>,
    /// The history's ramps, which each step's `ramps` index.
    pub ramps: Vec<Ramp>,
    /// The map's lenses.
    pub lenses: Vec<Lens>,
    /// What the checks found.
    pub summary: Summary,
    /// The atlas's digest (FNV-1a 64 of its text).
    pub atlas_digest: u64,
}

/// One county, as compiled.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// Its row.
    pub county: County,
    /// Its oracle point at genesis, per tick, every price relative to r = 1.
    pub point: Eq1a,
    /// Its steps, in (month, param) order.
    pub steps: Vec<Step>,
    /// Its oracle point after each step date, in month order: the point of the instance in
    /// force from the first of that month on.
    pub points: Vec<(Month, Eq1a)>,
}

impl Plan {
    /// The instance in force from the first of month `m` on (after that month's steps).
    pub fn instance_at(&self, m: Month) -> Instance {
        let mut inst = self.county.genesis;
        for s in self.steps.iter().take_while(|s| s.month <= m) {
            inst[s.param.index()] = s.value;
        }
        inst
    }
}

/// The extremes the checks met, each with where.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// Counties.
    pub counties: usize,
    /// `SetParam` steps, one event each.
    pub events: usize,
    /// (county, date) pairs at which some param steps: the oracle solves after genesis.
    pub step_dates: usize,
    /// The least of the provider's own baskets per unit of N.
    pub min_funding: (f64, String),
    /// The least participation N_a/N.
    pub min_participation: (f64, String),
    /// The most participation N_a/N.
    pub max_participation: (f64, String),
    /// The least x\*.
    pub min_x: (f64, String),
    /// The most x\*.
    pub max_x: (f64, String),
    /// The largest move at one date of the oracle's relative prices and technique
    /// (1 − x\*, w/r, p_m/r, p/r), in log.
    pub max_step_prices: (f64, String),
    /// The largest move at one date of its quantities (Y, K, N_a), in log.
    pub max_step_quantities: (f64, String),
    /// The largest move over a trailing year of the relative prices and technique, in log:
    /// from the point in force twelve months before a step date to the point after it.
    pub max_year_prices: (f64, String),
    /// The same, of the quantities.
    pub max_year_quantities: (f64, String),
}

/// The clock the world runs on.
pub fn clock(w: &World) -> Clock {
    Clock {
        start: w.start,
        ticks_per_year: w.ticks_per_year,
    }
}

/// Unit 1a's params for an instance, per tick.
pub fn params(inst: &Instance, c: &Clock) -> Params {
    let v = |p: Param| inst[p.index()];
    Params {
        workers: c.flow(FlowPerYear(v(Param::Workers))),
        land: c.flow(FlowPerYear(v(Param::Land))),
        space: v(Param::Space),
        a: v(Param::A),
        lam: v(Param::Lam),
        b: v(Param::B),
        schedule: PowerSchedule {
            eta: v(Param::Eta),
            g0: v(Param::G0),
            g1: v(Param::G1),
            k: v(Param::K),
        },
        work_cost: UniformWorkCost {
            chi_max: v(Param::ChiMax),
        },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    }
}

/// The grid on which the single crossing is counted.
const GRID: u32 = 400;

/// Participation at or above this is saturated (the workers offer every hour).
const SATURATED: f64 = 0.999;

/// Solve an instance and check it: `Interior`, one crossing of n_D − n_S on the grid,
/// funded, participation short of saturation.
pub fn solve(at: &str, inst: &Instance, c: &Clock) -> Result<Eq1a, CompileError> {
    let economy =
        Economy::new(params(inst, c)).map_err(|e| CompileError::new(at, format!("{e:?}")))?;
    let mut crossings = 0;
    let mut last: Option<bool> = None;
    for j in 0..=GRID {
        let x = 1e-9 + (1.0 - 2e-9) * f64::from(j) / f64::from(GRID);
        let f = economy.at(x).excess_demand();
        if f.is_finite() && f != 0.0 {
            let pos = f > 0.0;
            if last.is_some_and(|l| l != pos) {
                crossings += 1;
            }
            last = Some(pos);
        }
    }
    let e = match economy.solve() {
        Ok(Regime::Interior(e)) => *e,
        Ok(other) => {
            return Err(CompileError::new(
                at,
                format!(
                    "the oracle finds no interior equilibrium ({})",
                    other.name()
                ),
            ))
        }
        Err(e) => return Err(CompileError::new(at, format!("the oracle fails: {e:?}"))),
    };
    if crossings != 1 {
        return Err(CompileError::new(
            at,
            format!("n_D − n_S crosses zero {crossings} times on the grid, not once"),
        ));
    }
    if !e.funded {
        return Err(CompileError::new(
            at,
            "the provider cannot fund one basket per potential worker (T·r ≤ N·P_s)",
        ));
    }
    if e.participation >= SATURATED {
        return Err(CompileError::new(
            at,
            format!("participation {} is saturated", e.participation),
        ));
    }
    Ok(e)
}

/// The most a county's oracle point may move over any trailing year, in log, over its
/// relative prices and technique and over its quantities alike (O14): each step date's point
/// is compared with the point in force twelve months before it. A run of steps each under
/// `max_step` can still be abrupt over a year. The committed history's largest are 0.039 in
/// prices (Lanarkshire, 1848) and 0.051 in quantities (Monmouthshire, 1840), and it runs with
/// no dead tick, a lowest cleared volume of 0.945 of its target and a largest D̂ of 56. Middlesex's
/// land ×1.3 within 1800, every date under 0.03, moves it 0.19 and 0.29 in a year, and its run
/// falls to 0.78 of its target with D̂ 248 (verify-world-r1, 2026-09-27). The bound sits at
/// twice the history's largest; between the two, nothing has been run.
pub const MAX_YEAR: f64 = 0.1;

/// The largest |Δ ln| between two points over the relative prices and technique, and over
/// the quantities.
fn distance(a: &Eq1a, b: &Eq1a) -> (f64, f64) {
    let l = |x: f64, y: f64| num::ln(x / y).abs();
    let prices = [
        l(a.one_minus_x_star, b.one_minus_x_star),
        l(a.v, b.v),
        l(a.p_m, b.p_m),
        l(a.p, b.p),
    ]
    .into_iter()
    .fold(0.0, f64::max);
    let quantities = [l(a.y, b.y), l(a.k, b.k), l(a.n_a, b.n_a)]
        .into_iter()
        .fold(0.0, f64::max);
    (prices, quantities)
}

fn keep(slot: &mut (f64, String), v: f64, at: &str, more: bool) {
    if (more && v > slot.0) || (!more && v < slot.0) {
        *slot = (v, at.to_string());
    }
}

/// Compile a world's tables against an atlas.
pub fn compile(tables: &Tables, atlas: &Atlas) -> Result<Compiled, CompileError> {
    let Parsed {
        world,
        counties,
        ramps,
        lenses,
    } = tables::parse(tables, atlas)?;
    let c = clock(&world);
    let mut summary = Summary {
        counties: counties.len(),
        events: 0,
        step_dates: 0,
        min_funding: (f64::INFINITY, String::new()),
        min_participation: (f64::INFINITY, String::new()),
        max_participation: (f64::NEG_INFINITY, String::new()),
        min_x: (f64::INFINITY, String::new()),
        max_x: (f64::NEG_INFINITY, String::new()),
        max_step_prices: (0.0, String::new()),
        max_step_quantities: (0.0, String::new()),
        max_year_prices: (0.0, String::new()),
        max_year_quantities: (0.0, String::new()),
    };
    let mut plans = Vec::new();
    for county in counties {
        let steps = history::steps(&world, &county, &ramps)?;
        let mut inst = county.genesis;
        let at0 = format!("{} at {}", county.key, world.start);
        let point = solve(&at0, &inst, &c)?;
        let mut prev = point.clone();
        let mut note = |e: &Eq1a, inst: &Instance, at: &str| {
            let n = c.flow(FlowPerYear(inst[Param::Workers.index()]));
            keep(&mut summary.min_funding, e.provider_baskets / n, at, false);
            keep(&mut summary.min_participation, e.participation, at, false);
            keep(&mut summary.max_participation, e.participation, at, true);
            keep(&mut summary.min_x, e.x_star, at, false);
            keep(&mut summary.max_x, e.x_star, at, true);
        };
        note(&point, &inst, &at0);
        let mut points: Vec<(Month, Eq1a)> = Vec::new();
        let mut i = 0;
        while i < steps.len() {
            let m = steps[i].month;
            while i < steps.len() && steps[i].month == m {
                inst[steps[i].param.index()] = steps[i].value;
                i += 1;
            }
            let at = format!("{} at {}", county.key, history::date(&world, m));
            let e = solve(&at, &inst, &c)?;
            note(&e, &inst, &at);
            let (dp, dq) = distance(&e, &prev);
            for (what, d) in [
                (
                    "relative prices and technique (1 − x*, w/r, p_m/r, p/r)",
                    dp,
                ),
                ("quantities (Y, K, N_a)", dq),
            ] {
                if d > world.max_step {
                    return Err(CompileError::new(
                        &at,
                        format!(
                            "one date moves the oracle's {what} by {d:.4} in log, above \
                             max_step {}: make the history more gradual (O14)",
                            world.max_step
                        ),
                    ));
                }
            }
            // The point in force twelve months before: the last dated at or before then.
            let year_ago = points
                .iter()
                .rev()
                .find(|(pm, _)| *pm <= m - 12)
                .map_or(&point, |(_, e)| e);
            let (yp, yq) = distance(&e, year_ago);
            for (what, d) in [("relative prices and technique", yp), ("quantities", yq)] {
                if d > MAX_YEAR {
                    return Err(CompileError::new(
                        &at,
                        format!(
                            "the year to this date moves the oracle's {what} by {d:.4} in log, \
                             above {MAX_YEAR} a year: spread the change over more years (O14)"
                        ),
                    ));
                }
            }
            keep(&mut summary.max_step_prices, dp, &at, true);
            keep(&mut summary.max_step_quantities, dq, &at, true);
            keep(&mut summary.max_year_prices, yp, &at, true);
            keep(&mut summary.max_year_quantities, yq, &at, true);
            summary.step_dates += 1;
            points.push((m, e.clone()));
            prev = e;
        }
        summary.events += steps.len();
        plans.push(Plan {
            county,
            point,
            steps,
            points,
        });
    }
    let tape = write(&world, &plans, &ramps, atlas.digest);
    Ok(Compiled {
        tape,
        world,
        counties: plans,
        ramps,
        lenses,
        summary,
        atlas_digest: atlas.digest,
    })
}

/// A double as the tape writes it: the shortest text that reads back to it.
fn f(x: f64) -> String {
    format!("{x:?}")
}

/// A RON string literal. The tables' texts hold no control character (checked on reading).
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

/// The genesis of one county: its prices (w, r, p_m, p) with p = 1 coin, and each actor's
/// stationary coin, in the order desk.good, desk.mach, provider, workers.
pub struct Genesis {
    /// w, r, p_m, p.
    pub prices: [f64; 4],
    /// The stationary coins.
    pub coin: [f64; 4],
}

/// A county's genesis at its oracle point `e`, as `probe::setup::genesis` makes appb's, with
/// every price and coin divided by p/r so that the good costs 1 coin (the roles are
/// homogeneous of degree zero in prices and coin).
pub fn genesis(w: &World, inst: &Instance, e: &Eq1a) -> Genesis {
    let c = clock(w);
    let dial = |k: &str| w.dial(k).unwrap_or(0.0);
    let share = |k: &str| c.share(RatePerYear(dial(k)));
    let v = |p: Param| inst[p.index()];
    let n = c.flow(FlowPerYear(v(Param::Workers)));
    let t = c.flow(FlowPerYear(v(Param::Land)));
    let (w_, r, pm, p) = (e.v / e.p, 1.0 / e.p, e.p_m / e.p, 1.0);
    let ps = p + v(Param::Space) * r;
    let tau = n * ps;
    let hours = n * (num::ln1p(w_ / ps) / v(Param::ChiMax)).min(1.0);
    Genesis {
        prices: [w_, r, pm, p],
        coin: [
            p * e.y / share("buffer.desk.good.cash"),
            (v(Param::Lam) * w_ + v(Param::B) * r) * e.k / share("buffer.desk.mach.cash"),
            tau + (r * t - tau) / share("spend.provider"),
            (tau + w_ * hours) / share("spend.workers"),
        ],
    }
}

/// The four roles' actor suffixes, in the order genesis lists their coin.
pub const ROLES: [&str; 4] = ["desk.good", "desk.mach", "provider", "workers"];

fn write(w: &World, plans: &[Plan], ramps: &[Ramp], atlas: u64) -> String {
    let mark = &w.basis;
    let assumed = |why: &str| format!("Assumed({})", quote(&format!("{mark}: {why}")));
    let events: usize = plans.iter().map(|p| p.steps.len()).sum();
    let mut dates: Vec<Month> = plans
        .iter()
        .flat_map(|p| p.steps.iter().map(|s| s.month))
        .collect();
    dates.sort_unstable();
    dates.dedup();
    let mut o = String::new();
    let mut line = |s: &str| {
        o.push_str(s);
        o.push('\n');
    };
    for l in [
        format!("// The demo world, `{}` (docs/demo/WORLD.md): {} historic counties of the", w.name, plans.len()),
        "// United Kingdom, each its own node running the probe's four Appendix B roles".into(),
        "// (docs/probe/RULES.md) at its own instance, with no channels between them.".into(),
        "//".into(),
        "// Generated: do not edit. `rustyecon worldgen worlds/demo-gb --out tapes/demo-gb.ron`".into(),
        format!("// writes it from worlds/demo-gb/*.csv and the atlas data/atlas/gb.atlas.ron ({atlas:016x}),"),
        "// and the test `demo_tape_is_its_compilers_output` checks this file against the compiler.".into(),
        "//".into(),
        "// Illustrative: every entry's basis is Assumed(\"illustrative demo, ...\") and the name".into(),
        "// carries [illustrative], so nothing from this tape may be scored or cited (R4, R5).".into(),
        "//".into(),
        "// Genesis puts every county at its own oracle point, as tapes/appb.ron is made (RULES §4):".into(),
        "// unit 1a (crates/oracle) at (rho, delta, J_b) = (0, 1, 1), solved at the county's".into(),
        "// per-tick instance, with every price divided by p/r so that the good costs 1 coin; the".into(),
        "// good desk holds one tick's Y of the good and the machine desk one tick's K of machine".into(),
        "// services, the good desk's human share is 1 - x*, and each actor's coin is its".into(),
        "// stationary balance under the dials. No agent reads the oracle at run time (R13).".into(),
        "//".into(),
        format!(
            "// The history is {events} dated SetParam steps on {} dates: each copies a schedule param",
            dates.len()
        ),
        "// county.<c>.<param>.<YYYY-MM> whose basis names the ramps of worlds/demo-gb/history.csv".into(),
        format!(
            "// that moved it, stepped when the composed value has moved {} in log (WORLD.md §4.3).",
            w.step_log
        ),
        "Tape(".into(),
        "    schema: 1,".into(),
        "    header: (".into(),
        format!("        name: {},", quote(&w.name)),
        format!("        start: \"{}\",", w.start),
        format!("        ticks_per_year: {},", w.ticks_per_year),
        format!(
            "        market: (rule: Imbalance, one_sided: {}, ema_time_constant: \"price.ema_tc\"),",
            w.one_sided
        ),
        "        ledger: (rel_flow: \"ledger.rel_flow\", rel_stock: \"ledger.rel_stock\"),".into(),
        "    ),".into(),
        "    params: [".into(),
        "        // The ledger's tolerances and the dials, shared by every county (world.csv).".into(),
    ] {
        line(&l);
    }
    let param = |key: &str, value: f64, unit: &str, basis: &str| {
        format!(
            "        (key: {}, value: {}, unit: {unit}, basis: {basis}),",
            quote(key),
            f(value)
        )
    };
    for d in &w.dials {
        line(&param(&d.key, d.value, d.unit, &assumed(&d.note)));
    }
    line("        // Structure: machine services and the good last one tick (PROBE-SPEC §1.2).");
    line(&param(
        "life.one_tick",
        1.0 / f64::from(w.ticks_per_year),
        "Years",
        &assumed("one tick: J_b = 1 with delta = 1 (PROBE-SPEC §1.2), as tapes/appb.ron"),
    ));
    line("        // Each county's instance at the start (regions.csv): N and T a year, the rest dimensionless.");
    for p in plans {
        let c = &p.county;
        let basis = assumed(&format!("{}: {}", c.name, c.why));
        for q in Param::ALL {
            line(&param(
                &format!("{}.{}", c.key, q.column()),
                c.genesis[q.index()],
                q.unit(),
                &basis,
            ));
        }
    }
    line("        // The history's steps: schedule params, each copied by the event of its key.");
    for p in plans {
        let c = &p.county;
        let mut sorted: Vec<&Step> = p.steps.iter().collect();
        sorted.sort_by_key(|s| (s.param, s.month));
        for s in sorted {
            let names: Vec<&str> = s.ramps.iter().map(|&i| ramps[i].key.as_str()).collect();
            let why = format!(
                "{}'s {} from {}: {}",
                c.name,
                s.param.column(),
                history::stamp(w, s.month),
                names.join("; ")
            );
            line(&param(
                &format!(
                    "{}.{}.{}",
                    c.key,
                    s.param.column(),
                    history::stamp(w, s.month)
                ),
                s.value,
                s.param.unit(),
                &assumed(&why),
            ));
        }
    }
    for l in [
        "    ],",
        "    goods: [",
        "        (key: \"coin\", life: Indefinite, price_rate: None),",
        "        (key: \"good\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.good\")),",
        "        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),",
        "        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),",
        "        (key: \"mach\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.mach\")),",
        "    ],",
        "    nodes: [",
    ] {
        line(l);
    }
    for p in plans {
        line(&format!(
            "        (key: {}, currency: \"coin\"),",
            quote(&p.county.key)
        ));
    }
    for l in [
        "    ],",
        "    channels: [],",
        "    classes: [\"good_desks\", \"mach_desks\", \"owners\", \"workers\"],",
        "    actors: [",
    ] {
        line(l);
    }
    let role = assumed(
        "the probe's four roles, design-analytic-first with the judges' grafts (docs/probe/RULES.md)",
    );
    let mut gens = Vec::new();
    for p in plans {
        let c = &p.county;
        let e = &p.point;
        let g = genesis(w, &c.genesis, e);
        let k = &c.key;
        line(&format!(
            "        // {} ({k}): x* = {}, w/r = {}, p_m/r = {}, p/r = {}, Y = {}, K = {}, N_a = {}.",
            c.name,
            f(e.x_star),
            f(e.v),
            f(e.p_m),
            f(e.p),
            f(e.y),
            f(e.k),
            f(e.n_a)
        ));
        line(&format!(
            "        (key: \"{k}.desk.good\", kind: Desk, class: \"good_desks\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: GoodDesk((output: \"good\", labour: \"labour\", mach: \"mach\", \
             schedule: (eta: \"{k}.eta\", g0: \"{k}.g0\", g1: \"{k}.g1\", k: \"{k}.k\"), \
             technique: (adjust: \"adjust.technique\", share: {}), assign: Planned, \
             scale: Cash((turnover: \"buffer.desk.good.cash\", tilt: \"tilt.desk.good\", payout: None))))),",
            f(e.one_minus_x_star)
        ));
        line(&format!(
            "        (key: \"{k}.desk.mach\", kind: Desk, class: \"mach_desks\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: MachDesk((output: \"mach\", labour: \"labour\", land: \"land\", \
             recipe: (own: \"{k}.a\", labour: \"{k}.lam\", land: \"{k}.b\"), \
             scale: Cash((turnover: \"buffer.desk.mach.cash\", tilt: \"tilt.desk.mach\", payout: None))))),"
        ));
        line(&format!(
            "        (key: \"{k}.provider\", kind: Pop, class: \"owners\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: Provider((land: \"land\", endowment: \"{k}.land\", \
             transfer: (to: \"{k}.workers\", heads: \"{k}.workers\"), \
             basket: (good: \"good\", space: \"land\", per_basket: \"{k}.space\"), \
             spend: \"spend.provider\"))),"
        ));
        line(&format!(
            "        (key: \"{k}.workers\", kind: Pop, class: \"workers\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: Workers((labour: \"labour\", heads: \"{k}.workers\", chi_max: \"{k}.chi_max\", \
             basket: (good: \"good\", space: \"land\", per_basket: \"{k}.space\"), \
             spend: \"spend.workers\"))),"
        ));
        gens.push(g);
    }
    line("    ],");
    line("    genesis: (");
    line(&format!(
        "        basis: {},",
        assumed(
            "each county at its own oracle point, unit 1a at (rho, delta, J_b) = (0, 1, 1) \
             (crates/oracle; docs/probe/RULES.md §4), prices divided by p/r so that the good \
             costs 1 coin, stationary coins under the dials; written by rustyecon worldgen"
        )
    ));
    line("        prices: [");
    for (p, g) in plans.iter().zip(&gens) {
        let k = &p.county.key;
        let [pw, pr, ppm, pp] = g.prices;
        line(&format!(
            "            (node: \"{k}\", good: \"good\", price: {}), (node: \"{k}\", good: \"labour\", price: {}), \
             (node: \"{k}\", good: \"land\", price: {}), (node: \"{k}\", good: \"mach\", price: {}),",
            f(pp),
            f(pw),
            f(pr),
            f(ppm)
        ));
    }
    line("        ],");
    line("        holdings: [");
    for (p, g) in plans.iter().zip(&gens) {
        let k = &p.county.key;
        let e = &p.point;
        line(&format!(
            "            (holder: \"{k}.desk.good\", goods: [(\"coin\", {}), (\"good\", {})]), \
             (holder: \"{k}.desk.mach\", goods: [(\"coin\", {}), (\"mach\", {})]),",
            f(g.coin[0]),
            f(e.y),
            f(g.coin[1]),
            f(e.k)
        ));
        line(&format!(
            "            (holder: \"{k}.provider\", goods: [(\"coin\", {})]), \
             (holder: \"{k}.workers\", goods: [(\"coin\", {})]),",
            f(g.coin[2]),
            f(g.coin[3])
        ));
    }
    line("        ],");
    line("    ),");
    line("    events: [");
    let event_basis = assumed("history step");
    let mut all: Vec<(Month, &str, Param)> = plans
        .iter()
        .flat_map(|p| {
            p.steps
                .iter()
                .map(move |s| (s.month, p.county.key.as_str(), s.param))
        })
        .collect();
    all.sort_unstable();
    let mut last = None;
    for (m, k, q) in all {
        if last != Some(m) {
            line(&format!("        // {}", history::date(w, m)));
            last = Some(m);
        }
        let key = format!("{k}.{}.{}", q.column(), history::stamp(w, m));
        line(&format!(
            "        (key: \"{key}\", at: \"{}\", basis: {event_basis}, act: SetParam(param: \"{k}.{}\", to: \"{key}\")),",
            history::date(w, m),
            q.column()
        ));
    }
    line("    ],");
    line("    recurring: [],");
    line(")");
    o
}
