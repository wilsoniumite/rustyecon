//! The Appendix B tape and its generator (docs/probe/RULES.md §4).
//!
//! A [`Setup`] names everything a probe run is made from: the tick length, the instance, the
//! dials, the rule variants, the displacement of genesis and any dated shocks. [`tape_ron`]
//! writes it as a tape. The oracle's equilibrium seeds genesis (PROBE-SPEC §4.6): its prices
//! relative to r = 1 coin, 1 − x\*, one tick's output held by each desk, and each actor's
//! stationary coin under the dials. The oracle is solved here, outside any `Sim`; no agent reads
//! it (R13). `tapes/appb.ron` is `tape_ron(&Setup::registered(52))`, checked by a test.

use oracle::{Economy, Eq1a, Params, PowerSchedule, Regime, UniformWorkCost};
use rustyecon_core::{num, Clock, Date, FlowPerYear, RatePerYear};

/// The date of tick 0.
pub const START: &str = "1750-01-01";

/// The instance's coefficients as the tape registers them: N and T per year, the rest
/// dimensionless (PROBE-SPEC §1.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instance {
    /// N, potential hours per year.
    pub workers: f64,
    /// T, land services per year.
    pub land: f64,
    /// h, the space in one basket.
    pub space: f64,
    /// a, the machine desk's own input.
    pub a: f64,
    /// λ, its hours.
    pub lam: f64,
    /// b, its space: the cost-shock target.
    pub b: f64,
    /// η.
    pub eta: f64,
    /// g0.
    pub g0: f64,
    /// g1.
    pub g1: f64,
    /// k.
    pub k: f64,
    /// χ_max.
    pub chi_max: f64,
}

impl Instance {
    /// The SSRN Appendix B instance, with N = 4 and T = 10 per tick at 52 ticks a year.
    pub const fn appendix_b() -> Instance {
        Instance {
            workers: 208.0,
            land: 520.0,
            space: 1.0,
            a: 0.3,
            lam: 0.05,
            b: 0.4,
            eta: 1.0,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
            chi_max: 1.0,
        }
    }
}

/// The clock of a probe tape.
pub fn clock(tpy: u32) -> Result<Clock, String> {
    Ok(Clock {
        start: Date::parse(START).map_err(|e| e.to_string())?,
        ticks_per_year: tpy,
    })
}

/// The oracle's equilibrium of `inst` with b set to `b`, per tick at `tpy` ticks a year, every
/// price relative to r = 1 (unit 1a at (ρ, δ, J_b) = (0, 1, 1)).
pub fn equilibrium(inst: &Instance, b: f64, tpy: u32) -> Result<Eq1a, String> {
    let c = clock(tpy)?;
    let params = Params {
        workers: c.flow(FlowPerYear(inst.workers)),
        land: c.flow(FlowPerYear(inst.land)),
        space: inst.space,
        a: inst.a,
        lam: inst.lam,
        b,
        schedule: PowerSchedule {
            eta: inst.eta,
            g0: inst.g0,
            g1: inst.g1,
            k: inst.k,
        },
        work_cost: UniformWorkCost {
            chi_max: inst.chi_max,
        },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    };
    let economy = Economy::new(params).map_err(|e| e.to_string())?;
    match economy.solve().map_err(|e| e.to_string())? {
        Regime::Interior(eq) => Ok(*eq),
        other => Err(format!(
            "b = {b}: no interior equilibrium ({})",
            other.name()
        )),
    }
}

/// The design's dials, per year as the tape registers them (docs/probe/RULES.md §3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dials {
    /// `rate.labour`, RatePerYear.
    pub rate_labour: f64,
    /// `rate.land`, RatePerYear.
    pub rate_land: f64,
    /// `rate.mach`, RatePerYear.
    pub rate_mach: f64,
    /// `rate.good`, RatePerYear.
    pub rate_good: f64,
    /// `price.ema_tc`, Years.
    pub ema_tc: f64,
    /// `adjust.technique`, RatePerYear.
    pub adjust_technique: f64,
    /// `spend.workers`, RatePerYear.
    pub spend_workers: f64,
    /// `spend.provider`, RatePerYear.
    pub spend_provider: f64,
    /// `buffer.desk.good.cash`, RatePerYear (turnover).
    pub turnover_good: f64,
    /// `buffer.desk.mach.cash`, RatePerYear (turnover).
    pub turnover_mach: f64,
    /// `tilt.desk.good`, Dimensionless.
    pub tilt_good: f64,
    /// `tilt.desk.mach`, Dimensionless.
    pub tilt_mach: f64,
}

impl Dials {
    /// The registered dials: design-analytic-first's C2, with the markup tilt registered at 0.
    pub const fn registered() -> Dials {
        Dials {
            rate_labour: 5.2,
            rate_land: 1.3,
            rate_mach: 1.3,
            rate_good: 2.6,
            ema_tc: 0.5,
            adjust_technique: 2.6,
            spend_workers: 13.0,
            spend_provider: 13.0,
            turnover_good: 5.2,
            turnover_mach: 5.2,
            tilt_good: 0.0,
            tilt_mach: 0.0,
        }
    }

    /// The dials by tape key.
    fn fields(&mut self) -> [(&'static str, &mut f64); 12] {
        [
            ("rate.labour", &mut self.rate_labour),
            ("rate.land", &mut self.rate_land),
            ("rate.mach", &mut self.rate_mach),
            ("rate.good", &mut self.rate_good),
            ("price.ema_tc", &mut self.ema_tc),
            ("adjust.technique", &mut self.adjust_technique),
            ("spend.workers", &mut self.spend_workers),
            ("spend.provider", &mut self.spend_provider),
            ("buffer.desk.good.cash", &mut self.turnover_good),
            ("buffer.desk.mach.cash", &mut self.turnover_mach),
            ("tilt.desk.good", &mut self.tilt_good),
            ("tilt.desk.mach", &mut self.tilt_mach),
        ]
    }

    /// Set a dial by its tape key, or scale a family: `rate.*` scales the four price rates,
    /// `buffer.*` both desks' turnovers and `tilt.*` sets both tilts.
    pub fn set(&mut self, key: &str, value: f64) -> Result<(), String> {
        match key {
            "rate.*" => {
                self.rate_labour *= value;
                self.rate_land *= value;
                self.rate_mach *= value;
                self.rate_good *= value;
                return Ok(());
            }
            "buffer.*" => {
                self.turnover_good *= value;
                self.turnover_mach *= value;
                return Ok(());
            }
            "tilt.*" => {
                self.tilt_good = value;
                self.tilt_mach = value;
                return Ok(());
            }
            _ => {}
        }
        for (k, v) in self.fields() {
            if k == key {
                *v = value;
                return Ok(());
            }
        }
        Err(format!("no dial {key}"))
    }
}

/// How the good desk assigns tasks in production.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assign {
    /// Leontief at the planned technique (the registered default).
    Planned,
    /// The ex-post cutoff that uses up both inputs (the registered alternative).
    ExPost,
}

/// The desks' scale rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleRule {
    /// The cash rule (the registered default).
    Cash,
    /// The cash rule with a payout above a ceiling (a registered variant): `payout.desk.*`,
    /// RatePerYear, and `ceiling.desk.*`, the log ceiling.
    Ceiling {
        /// The payout rate per year.
        rate: f64,
        /// The log ceiling over the stationary coin.
        ceiling: f64,
    },
    /// July's margin-step rule (the registered negative control).
    Step {
        /// `step.desk.*.up`, RatePerYear: log step per unit log margin.
        up: f64,
        /// `step.desk.*.down`, RatePerYear.
        down: f64,
        /// `dead.desk.*`, Dimensionless: the band on the log margin.
        dead: f64,
        /// `payout.desk.*`, RatePerYear: the payout of coin above the buffer.
        payout: f64,
    },
}

impl ScaleRule {
    /// design-adaptive's ceiling payout, as the variant registers it: half the excess a tick
    /// (36.04/yr at 52) above twice the stationary coin.
    pub fn ceiling() -> ScaleRule {
        ScaleRule::Ceiling {
            rate: 26.0,
            ceiling: std::f64::consts::LN_2,
        }
    }

    /// The negative control as design-analytic-first's ablation ran it: symmetric steps of
    /// 2.6 a year, no band, the payout at 26 a year.
    pub fn july_step() -> ScaleRule {
        ScaleRule::Step {
            up: 2.6,
            down: 2.6,
            dead: 0.0,
            payout: 26.0,
        }
    }
}

/// The one-sided rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneSided {
    /// A one-sided market's price steps by e^(±k).
    Saturate,
    /// A one-sided market's price holds.
    Hold,
}

/// The genesis human share: the oracle's times a factor, or set outright.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Share {
    /// (1 − x\*)·f.
    Times(f64),
    /// 1 − x for this x.
    TechniqueAt(f64),
}

/// A displacement of genesis from the oracle's point: factors on the prices, the human share,
/// each actor's coin (desk.good, desk.mach, provider, workers) and the desks' stocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Displacement {
    /// w.
    pub w: f64,
    /// r.
    pub r: f64,
    /// p_m.
    pub pm: f64,
    /// p.
    pub p: f64,
    /// 1 − x.
    pub share: Share,
    /// Coin, in actor order: desk.good, desk.mach, provider, workers.
    pub coin: [f64; 4],
    /// The good desk's stock of the good.
    pub good: f64,
    /// The machine desk's stock of machine services.
    pub mach: f64,
}

impl Displacement {
    /// No displacement: mode A.
    pub const fn none() -> Displacement {
        Displacement {
            w: 1.0,
            r: 1.0,
            pm: 1.0,
            p: 1.0,
            share: Share::Times(1.0),
            coin: [1.0; 4],
            good: 1.0,
            mach: 1.0,
        }
    }
}

/// A dated change of b, by a `SetParam` at the start of `tick`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shock {
    /// The tick it fires in.
    pub tick: u64,
    /// b from then on.
    pub b: f64,
}

/// Everything a probe tape is made from.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    /// Ticks per year.
    pub tpy: u32,
    /// The instance the tape registers (its b is the b in force from tick 0).
    pub instance: Instance,
    /// The b whose oracle point seeds genesis.
    pub genesis_b: f64,
    /// The dials.
    pub dials: Dials,
    /// The good desk's task assignment.
    pub assign: Assign,
    /// The desks' scale rule.
    pub scale: ScaleRule,
    /// The one-sided rule.
    pub one_sided: OneSided,
    /// The displacement of genesis.
    pub displace: Displacement,
    /// Dated changes of b, in tick order.
    pub shocks: Vec<Shock>,
}

impl Setup {
    /// The registered setup at `tpy` ticks a year: the Appendix B instance, the registered
    /// dials and variants, genesis at the oracle's point (mode A).
    pub fn registered(tpy: u32) -> Setup {
        let instance = Instance::appendix_b();
        Setup {
            tpy,
            instance,
            genesis_b: instance.b,
            dials: Dials::registered(),
            assign: Assign::Planned,
            scale: ScaleRule::Cash,
            one_sided: OneSided::Saturate,
            displace: Displacement::none(),
            shocks: Vec::new(),
        }
    }

    /// The b in force at `tick`.
    pub fn b_at(&self, tick: u64) -> f64 {
        self.shocks
            .iter()
            .rfind(|s| s.tick <= tick)
            .map_or(self.instance.b, |s| s.b)
    }
}

/// The genesis a setup writes: the undisplaced oracle point and the displaced values.
#[derive(Debug, Clone, PartialEq)]
pub struct Genesis {
    /// The oracle's point at `genesis_b`.
    pub point: Eq1a,
    /// The genesis prices: w, r, p_m, p.
    pub prices: [f64; 4],
    /// The good desk's genesis human share.
    pub share: f64,
    /// The stationary coins at the point, before displacement, in actor order.
    pub stationary: [f64; 4],
    /// The genesis coins, in actor order: desk.good, desk.mach, provider, workers.
    pub coin: [f64; 4],
    /// The good desk's stock of the good.
    pub good: f64,
    /// The machine desk's stock of machine services.
    pub mach: f64,
}

/// The actors, in the order the probe lists them.
pub const ACTORS: [&str; 4] = ["desk.good", "desk.mach", "provider", "workers"];

/// Genesis for a setup.
pub fn genesis(s: &Setup) -> Result<Genesis, String> {
    let e = equilibrium(&s.instance, s.genesis_b, s.tpy)?;
    let c = clock(s.tpy)?;
    let share = |v: f64| c.share(RatePerYear(v));
    let inst = &s.instance;
    let n = c.flow(FlowPerYear(inst.workers));
    let t = c.flow(FlowPerYear(inst.land));
    let (w, r, pm, p) = (e.v, 1.0, e.p_m, e.p);
    // At the point, the transfer is N·P_s and the workers offer N·F(ln(1 + w/P_s)) hours, as
    // their rule computes them; each actor spends its share of its coin, and at rest that is
    // exactly what it pays out each tick (docs/probe/RULES.md §4).
    let ps = p + inst.space * r;
    let tau = n * ps;
    let hours = n * (num::ln1p(w / ps) / inst.chi_max).min(1.0);
    let d = &s.dials;
    let stationary = [
        p * e.y / share(d.turnover_good),
        (inst.lam * w + s.genesis_b * r) * e.k / share(d.turnover_mach),
        tau + (r * t - tau) / share(d.spend_provider),
        (tau + w * hours) / share(d.spend_workers),
    ];
    let x = &s.displace;
    let one_minus_x = match x.share {
        Share::Times(f) => e.one_minus_x_star * f,
        Share::TechniqueAt(xx) => 1.0 - xx,
    };
    if !(0.0..=1.0).contains(&one_minus_x) {
        return Err(format!(
            "the genesis human share {one_minus_x} is outside [0, 1]"
        ));
    }
    let mut coin = stationary;
    for (c, f) in coin.iter_mut().zip(x.coin) {
        *c *= f;
    }
    Ok(Genesis {
        prices: [w * x.w, r * x.r, pm * x.pm, p * x.p],
        share: one_minus_x,
        stationary,
        coin,
        good: e.y * x.good,
        mach: e.k * x.mach,
        point: e,
    })
}

fn f(x: f64) -> String {
    format!("{x:?}")
}

const INST: &str =
    "Literature(\"SSRN 7226858 App. B via laborformal 31b3482 paths/code/macro.py\")";
const DESIGN: &str =
    "Assumed(\"design-analytic-first C2, registered 2026-09-26 (docs/probe/RULES.md)\")";
const ROLE: &str =
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

/// The tape of a setup, as RON text with its derivation in the header.
pub fn tape_ron(s: &Setup) -> Result<String, String> {
    let g = genesis(s)?;
    let e = &g.point;
    let c = clock(s.tpy)?;
    let d = &s.dials;
    let i = &s.instance;
    let n = c.flow(FlowPerYear(i.workers));
    let t = c.flow(FlowPerYear(i.land));
    let mut out = Lines(Vec::new());
    let o = &mut out;
    for line in [
        "// The Appendix B world (docs/probe/RULES.md): the SSRN Appendix B flow economy on the"
            .to_string(),
        "// engine, run by the four Appendix B roles, for the Phase 2 probe (PROBE-SPEC,".into(),
        "// 2026-09-26).".into(),
        "//".into(),
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin appb-tape -- <path>` writes"
            .into(),
        "// it, and the test `appb_tape_is_its_generators_output` checks this file against the"
            .into(),
        "// generator.".into(),
        "//".into(),
        "// Genesis is the oracle's equilibrium (PROBE-SPEC §4.6, mode A): crates/oracle (unit 1a,"
            .into(),
        format!(
            "// P1.1) solved at the instance's per-tick values, N = {} and T = {} at {} ticks a",
            f(n),
            f(t),
            s.tpy
        ),
        format!("// year, and b = {}, gives", f(s.genesis_b)),
        format!("//   x* = {}, 1 - x* = {},", f(e.x_star), f(e.one_minus_x_star)),
        format!(
            "//   v = w/r = {}, p_m/r = {}, p/r = {},",
            f(e.v),
            f(e.p_m),
            f(e.p)
        ),
        format!(
            "//   P_s/r = {}, Y = {}, K = {}, N_a = {}.",
            f(e.p_s),
            f(e.y),
            f(e.k),
            f(e.n_a)
        ),
        "// The genesis prices are those ratios with r = 1 coin. The good desk holds Y of the good"
            .into(),
        "// and the machine desk K of machine services, one tick's output each, and the good desk's"
            .into(),
        "// human share 1 - x is 1 - x*. Each actor's coin is its stationary balance under the"
            .into(),
        format!(
            "// dials, with share(v) = 1 - exp(-v/{}) (Clock::share; docs/probe/RULES.md §4):",
            s.tpy
        ),
        format!(
            "//   desk.good  p*Y/share(buffer.desk.good.cash)               = {}",
            f(g.stationary[0])
        ),
        format!(
            "//   desk.mach  (lam*w + b*r)*K/share(buffer.desk.mach.cash)   = {}",
            f(g.stationary[1])
        ),
        format!(
            "//   provider   N*P_s + (r*T - N*P_s)/share(spend.provider)    = {}",
            f(g.stationary[2])
        ),
        format!(
            "//   workers    (N*P_s + w*N*F)/share(spend.workers)           = {}",
            f(g.stationary[3])
        ),
        "//              with F = ln1p(w/P_s)/chi_max, the share of hours offered.".into(),
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in"
            .into(),
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
        "        name: \"appb\",".into(),
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
    o.line("        // The instance (PROBE-SPEC §1.5).");
    for (key, value, unit) in [
        ("inst.workers", i.workers, "FlowPerYear"),
        ("inst.land", i.land, "FlowPerYear"),
        ("inst.space", i.space, "Dimensionless"),
        ("inst.a", i.a, "Dimensionless"),
        ("inst.lam", i.lam, "Dimensionless"),
        ("inst.b", i.b, "Dimensionless"),
        ("inst.eta", i.eta, "Dimensionless"),
        ("inst.g0", i.g0, "Dimensionless"),
        ("inst.g1", i.g1, "Dimensionless"),
        ("inst.k", i.k, "Dimensionless"),
        ("inst.chi_max", i.chi_max, "Dimensionless"),
    ] {
        o.param(key, value, unit, INST);
    }
    o.line("        // Structure: machine services and the good last one tick (PROBE-SPEC §1.2).");
    o.param(
        "life.one_tick",
        1.0 / f64::from(s.tpy),
        "Years",
        "Assumed(\"one tick: J_b = 1 with delta = 1 (PROBE-SPEC §1.2)\")",
    );
    o.line(
        "        // The design's dials (docs/probe/RULES.md §3). The wage is the fastest price.",
    );
    o.param(
        "price.ema_tc",
        d.ema_tc,
        "Years",
        "Approximate(\"the gate world's value; no rule reads the EMA\")",
    );
    for (key, value) in [
        ("rate.labour", d.rate_labour),
        ("rate.land", d.rate_land),
        ("rate.mach", d.rate_mach),
        ("rate.good", d.rate_good),
        ("adjust.technique", d.adjust_technique),
        ("spend.workers", d.spend_workers),
        ("spend.provider", d.spend_provider),
        ("buffer.desk.good.cash", d.turnover_good),
        ("buffer.desk.mach.cash", d.turnover_mach),
    ] {
        o.param(key, value, "RatePerYear", DESIGN);
    }
    match s.scale {
        ScaleRule::Cash | ScaleRule::Ceiling { .. } => {
            let tilt = "Assumed(\"judges' graft from design-reservation: the markup tilt, \
                        registered at 0 (docs/probe/RULES.md §5)\")";
            o.param("tilt.desk.good", d.tilt_good, "Dimensionless", tilt);
            o.param("tilt.desk.mach", d.tilt_mach, "Dimensionless", tilt);
        }
        ScaleRule::Step { .. } => {}
    }
    match s.scale {
        ScaleRule::Cash => {}
        ScaleRule::Ceiling { rate, ceiling } => {
            let basis = "Assumed(\"variant: design-adaptive's ceiling payout \
                         (docs/probe/RULES.md §5)\")";
            for desk in ["good", "mach"] {
                o.param(&format!("payout.desk.{desk}"), rate, "RatePerYear", basis);
                o.param(
                    &format!("ceiling.desk.{desk}"),
                    ceiling,
                    "Dimensionless",
                    basis,
                );
            }
        }
        ScaleRule::Step {
            up,
            down,
            dead,
            payout,
        } => {
            let basis = "Assumed(\"negative control: July's margin-step rule as \
                         design-analytic-first's ablation ran it (docs/probe/RULES.md §5)\")";
            for desk in ["good", "mach"] {
                o.param(&format!("step.desk.{desk}.up"), up, "RatePerYear", basis);
                o.param(
                    &format!("step.desk.{desk}.down"),
                    down,
                    "RatePerYear",
                    basis,
                );
                o.param(&format!("dead.desk.{desk}"), dead, "Dimensionless", basis);
                o.param(&format!("payout.desk.{desk}"), payout, "RatePerYear", basis);
            }
        }
    }
    if !s.shocks.is_empty() {
        o.line("        // The b shocks' values: schedule params (PROBE-SPEC §4.7, G9).");
        for (k, shock) in s.shocks.iter().enumerate() {
            o.param(
                &format!("inst.b.shock.{}", k + 1),
                shock.b,
                "Dimensionless",
                "Assumed(\"PROBE-SPEC §4.7: a cost shock b'\")",
            );
        }
    }
    for line in [
        "    ],",
        "    goods: [",
        "        (key: \"coin\", life: Indefinite, price_rate: None),",
        "        (key: \"good\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.good\")),",
        "        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),",
        "        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),",
        "        (key: \"mach\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.mach\")),",
        "    ],",
        "    nodes: [(key: \"home\", currency: \"coin\")],",
        "    channels: [],",
        "    classes: [\"good_desks\", \"mach_desks\", \"owners\", \"workers\"],",
        "    actors: [",
    ] {
        o.line(line);
    }
    let assign = match s.assign {
        Assign::Planned => "Planned",
        Assign::ExPost => "ExPost",
    };
    let scale = |desk: &str, stock: f64| -> Vec<String> {
        match s.scale {
            ScaleRule::Cash => vec![format!(
                "            scale: Cash((turnover: \"buffer.desk.{desk}.cash\", tilt: \
                 \"tilt.desk.{desk}\", payout: None)),"
            )],
            ScaleRule::Ceiling { .. } => vec![
                format!(
                    "            scale: Cash((turnover: \"buffer.desk.{desk}.cash\", tilt: \
                     \"tilt.desk.{desk}\","
                ),
                format!(
                    "                payout: Some((to: \"provider\", rate: \"payout.desk.{desk}\", \
                     ceiling: \"ceiling.desk.{desk}\")))),"
                ),
            ],
            ScaleRule::Step { .. } => vec![
                format!(
                    "            scale: Step((up: \"step.desk.{desk}.up\", down: \
                     \"step.desk.{desk}.down\", dead: \"dead.desk.{desk}\","
                ),
                format!(
                    "                buffer: \"buffer.desk.{desk}.cash\", payout: (to: \
                     \"provider\", rate: \"payout.desk.{desk}\"), scale: {})),",
                    f(stock)
                ),
            ],
        }
    };
    o.line(format!(
        "        (key: \"desk.good\", kind: Desk, class: \"good_desks\", home: \"home\", basis: {ROLE},"
    ));
    o.line("         spec: GoodDesk((");
    o.line("            output: \"good\", labour: \"labour\", mach: \"mach\",");
    o.line(
        "            schedule: (eta: \"inst.eta\", g0: \"inst.g0\", g1: \"inst.g1\", k: \"inst.k\"),",
    );
    o.line(format!(
        "            technique: (adjust: \"adjust.technique\", share: {}),",
        f(g.share)
    ));
    o.line(format!("            assign: {assign},"));
    for l in scale("good", e.y) {
        o.line(l);
    }
    o.line("         ))),");
    o.line(format!(
        "        (key: \"desk.mach\", kind: Desk, class: \"mach_desks\", home: \"home\", basis: {ROLE},"
    ));
    o.line("         spec: MachDesk((");
    o.line("            output: \"mach\", labour: \"labour\", land: \"land\",");
    o.line("            recipe: (own: \"inst.a\", labour: \"inst.lam\", land: \"inst.b\"),");
    for l in scale("mach", e.k) {
        o.line(l);
    }
    o.line("         ))),");
    for line in [
        format!(
            "        (key: \"provider\", kind: Pop, class: \"owners\", home: \"home\", basis: {ROLE},"
        ),
        "         spec: Provider((".into(),
        "            land: \"land\", endowment: \"inst.land\",".into(),
        "            transfer: (to: \"workers\", heads: \"inst.workers\"),".into(),
        "            basket: (good: \"good\", space: \"land\", per_basket: \"inst.space\"),".into(),
        "            spend: \"spend.provider\",".into(),
        "         ))),".into(),
        format!(
            "        (key: \"workers\", kind: Pop, class: \"workers\", home: \"home\", basis: {ROLE},"
        ),
        "         spec: Workers((".into(),
        "            labour: \"labour\", heads: \"inst.workers\", chi_max: \"inst.chi_max\",".into(),
        "            basket: (good: \"good\", space: \"land\", per_basket: \"inst.space\"),".into(),
        "            spend: \"spend.workers\",".into(),
        "         ))),".into(),
        "    ],".into(),
    ] {
        o.line(line);
    }
    let [pw, pr, ppm, pp] = g.prices;
    for line in [
        "    genesis: (".to_string(),
        format!(
            "        basis: Approximate(\"oracle unit 1a (crates/oracle, P1.1) at the SSRN Appendix \
             B instance, N = {} and T = {} per tick, b = {}; stationary coins per \
             docs/probe/RULES.md §4; written by rustyecon-probe's appb-tape\"),",
            f(n),
            f(t),
            f(s.genesis_b)
        ),
        "        prices: [".into(),
        format!("            (node: \"home\", good: \"good\", price: {}),", f(pp)),
        format!("            (node: \"home\", good: \"labour\", price: {}),", f(pw)),
        format!("            (node: \"home\", good: \"land\", price: {}),", f(pr)),
        format!("            (node: \"home\", good: \"mach\", price: {}),", f(ppm)),
        "        ],".into(),
        "        holdings: [".into(),
        format!(
            "            (holder: \"desk.good\", goods: [(\"coin\", {}), (\"good\", {})]),",
            f(g.coin[0]),
            f(g.good)
        ),
        format!(
            "            (holder: \"desk.mach\", goods: [(\"coin\", {}), (\"mach\", {})]),",
            f(g.coin[1]),
            f(g.mach)
        ),
        format!(
            "            (holder: \"provider\", goods: [(\"coin\", {})]),",
            f(g.coin[2])
        ),
        format!(
            "            (holder: \"workers\", goods: [(\"coin\", {})]),",
            f(g.coin[3])
        ),
        "        ],".into(),
        "    ),".into(),
        "    events: [".into(),
    ] {
        o.line(line);
    }
    for (k, shock) in s.shocks.iter().enumerate() {
        let date = c
            .date_of(shock.tick)
            .ok_or_else(|| format!("tick {} has no date", shock.tick))?;
        o.line(format!(
            "        (key: \"b.shock.{}\", at: \"{date}\", basis: Assumed(\"PROBE-SPEC §4.7: a \
             dated cost shock at tick {}\"),",
            k + 1,
            shock.tick
        ));
        o.line(format!(
            "         act: SetParam(param: \"inst.b\", to: \"inst.b.shock.{}\")),",
            k + 1
        ));
    }
    o.line("    ],");
    o.line("    recurring: [],");
    o.line(")");
    let mut text = out.0.join("\n");
    text.push('\n');
    Ok(text)
}
