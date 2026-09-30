//! Rule A's chain for one county (docs/demo/WORLD-V2.md §2, §9.2; GOODS-CHAIN §5; decisions 322,
//! 323 and 329), D2.2, 2026-09-30.
//!
//! A county's flow machine (a, λ, b) becomes the horse: fodder is made from ω·b land alone; a
//! head is bred from a·κ/δ of the maker's own horse-days, λ·κ/δ labour and (1 − ω)·b·κ/δ pasture,
//! κ and δ per tick; a horse-day runs on one unit of fodder. At ρ = 0 the chain's equilibrium is
//! the county's (unit 1g's collapse to 1a, R1b).
//!
//! This is the stage's one definition of the mapping (O32 for rule A): [`rule_a`] computes the
//! recipe's per-tick coefficients from the clock and the county's (a, λ, b) in the order
//! `probe::horses::instance::Instance::rule_a` does, and [`goods_chain`] builds unit 1g's
//! `GoodsChain` as `Instance::chain` does, so the tape's coefficients and the oracle's are the
//! probe harness's bit for bit (`rule_a_is_the_probes`). worldgen does not depend on the probe
//! crate (decision 329). The oracle is solved here, outside any `Sim`; no agent reads it (R13).

use oracle::{
    Category, ChainCategory, ChainEconomy, GoodsChain, GoodsRecipe, Machine, Material,
    PowerSchedule, Regime, UniformWorkCost,
};
use rustyecon_core::{Clock, FlowPerYear, FractionPerYear};

/// A machine type (`machine_types.csv`): its key, its rule, its wear and hours a year, the share
/// of the machine's land its horse eats as fodder while it works, and its build lag.
#[derive(Debug, Clone, PartialEq)]
pub struct MachineType {
    /// Its key, the durable good's (`horse`).
    pub key: String,
    /// δ, its wear a year (`FractionPerYear`); 1 is the flow path, all of it a tick.
    pub delta: f64,
    /// κ, its hours (horse-days) a unit a year (`FlowPerYear`).
    pub kappa: f64,
    /// ω, the share of a machine service's land that is its fodder.
    pub omega: f64,
    /// J_b, its build lag in ticks.
    pub build_lag: u32,
    /// A note.
    pub note: String,
}

impl MachineType {
    /// κ and δ per tick at the clock `c`: `Clock::flow` and `Clock::fraction`, as the engine
    /// converts `horse.kappa` and `horse.delta` for the roles.
    pub fn per_tick(&self, c: &Clock) -> (f64, f64) {
        (
            c.flow(FlowPerYear(self.kappa)),
            c.fraction(FractionPerYear(self.delta)),
        )
    }

    /// Whether the machine lives one tick: the flow path, the stocks layer off (R1a).
    pub fn is_flow(&self) -> bool {
        self.delta == 1.0
    }

    /// Whether fodder is a traded good (ω > 0).
    pub fn has_fodder(&self) -> bool {
        self.omega > 0.0
    }
}

/// Rule A's per-tick coefficients (HORSES-SPEC §1.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuleA {
    /// Land per unit of fodder, ω·b.
    pub fodder_land: f64,
    /// A head's own horse-days, a·κ/δ (a on the flow path).
    pub own_hours: f64,
    /// A head's labour, λ·κ/δ (λ on the flow path).
    pub labour: f64,
    /// A head's pasture, (1 − ω)·b·κ/δ (b on the flow path).
    pub pasture: f64,
}

/// Rule A's coefficients for a county's flow machine (a, λ, b) at the clock `c`, in the order
/// `probe::horses::instance::Instance::rule_a` computes them. On the flow path a head is the
/// probe's one-tick service, and its recipe is the flow machine's (a, λ, b).
pub fn rule_a(m: &MachineType, c: &Clock, a: f64, lam: f64, b: f64) -> RuleA {
    let (kappa, d) = m.per_tick(c);
    let w = m.omega;
    if m.is_flow() {
        return RuleA {
            fodder_land: w * b,
            own_hours: a,
            labour: lam,
            pasture: b,
        };
    }
    RuleA {
        fodder_land: w * b,
        own_hours: a * kappa / d,
        labour: lam * kappa / d,
        pasture: (1.0 - w) * b * kappa / d,
    }
}

/// What unit 1g's chain is built from at one date: the county's households and task line as the
/// tape registers them (N and T a year), rule A's coefficients per tick, and κ and δ per tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainParams {
    /// N, potential hours a year.
    pub workers: f64,
    /// T, land services a year.
    pub land: f64,
    /// h, the space in one basket.
    pub space: f64,
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
    /// Rule A's coefficients per tick.
    pub rule: RuleA,
    /// κ per tick.
    pub kappa: f64,
    /// δ per tick.
    pub delta: f64,
    /// Whether fodder is a traded good.
    pub fodder: bool,
}

/// Unit 1g's chain at `p`, per tick at the clock `c`, with the horse's hours built at J = `lag`
/// ticks (J = 2 for the capacity desk's hours, decision 232: at ρ = 0 every price and quantity is
/// J 1's), as `probe::horses::instance::Instance::chain` builds it.
pub fn goods_chain(p: &ChainParams, c: &Clock, lag: u32) -> GoodsChain {
    let r = p.rule;
    let goods = |inputs: Vec<(&str, f64)>, labor: f64, land: f64| GoodsRecipe {
        inputs: inputs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
        labor,
        land,
    };
    let (materials, operating) = if p.fodder {
        (
            vec![Material {
                key: "FODDER".into(),
                recipe: goods(vec![], 0.0, r.fodder_land),
            }],
            goods(vec![("FODDER", 1.0)], 0.0, 0.0),
        )
    } else {
        (Vec::new(), goods(vec![], 0.0, 0.0))
    };
    GoodsChain {
        workers: c.flow(FlowPerYear(p.workers)),
        land: c.flow(FlowPerYear(p.land)),
        schedule: PowerSchedule {
            eta: p.eta,
            g0: p.g0,
            g1: p.g1,
            k: p.k,
        },
        work_cost: UniformWorkCost { chi_max: p.chi_max },
        rho: 0.0,
        edges: vec![0.0, 1.0],
        categories: vec![
            ChainCategory {
                key: "GOOD".into(),
                category: Category {
                    weight: 1.0,
                    direct_land: 0.0,
                    density: vec![1.0],
                },
                inputs: vec![],
            },
            ChainCategory {
                key: "SPACE".into(),
                category: Category {
                    weight: p.space,
                    direct_land: 1.0,
                    density: vec![0.0],
                },
                inputs: vec![],
            },
        ],
        materials,
        machines: vec![Machine {
            key: "HORSE".into(),
            build: goods(vec![("HORSE_DAYS", r.own_hours)], r.labour, r.pasture),
            hours: "HORSE_DAYS".into(),
            hours_per_period: p.kappa,
            task_efficiency: 1.0,
            operating,
            delta: p.delta,
            build_lag: lag,
        }],
    }
}

/// The chain's equilibrium as the stage reads it: per tick, relative to r = 1, as
/// `probe::horses::instance::Point` reads it off the stocks path.
#[derive(Debug, Clone, PartialEq)]
pub struct ChainPoint {
    /// x\*.
    pub x_star: f64,
    /// 1 − x\*, carried.
    pub one_minus_x: f64,
    /// v = w/r.
    pub v: f64,
    /// P_s.
    pub p_s: f64,
    /// Y, baskets.
    pub y: f64,
    /// N_a, hours worked.
    pub n_a: f64,
    /// Participation N_a/N.
    pub participation: f64,
    /// p, the good's price.
    pub p: f64,
    /// The good's output.
    pub good: f64,
    /// Fodder's price (NaN where it is not traded).
    pub pf: f64,
    /// A head's price, p_K.
    pub pk: f64,
    /// A horse-day's price, p_h = O + δ·p_K/κ.
    pub ph: f64,
    /// A horse-day's running cost, O.
    pub o: f64,
    /// Fodder made a tick.
    pub qf: f64,
    /// Heads made a tick, q_b.
    pub made: f64,
    /// Heads sold a tick, q_b·(1 − δ·a/κ).
    pub sold: f64,
    /// Every horse-day a tick, the maker's own included.
    pub hours: f64,
    /// The horse-days the tasks use, Y·J(x\*).
    pub task_hours: f64,
    /// The heads that do the tasks, Y·J(x\*)/κ: the capacity desk's.
    pub capacity: f64,
    /// The maker's serving stock, a·q_b/κ (a its own horse-days per head).
    pub serving: f64,
    /// The provider's baskets, T/P_s − N.
    pub provider_baskets: f64,
    /// The workers' baskets.
    pub worker_baskets: f64,
    /// Whether the transfer is funded at the point.
    pub funded: bool,
}

impl ChainPoint {
    /// Every head installed: the tasks' and the maker's serving stock.
    pub fn heads(&self) -> f64 {
        self.capacity + self.serving
    }
}

/// Solve the chain at `p` (J = 2) and read its interior point, as
/// `probe::horses::instance::Instance::point` does off the flow path.
pub fn point(p: &ChainParams, c: &Clock) -> Result<ChainPoint, String> {
    let chain = goods_chain(p, c, 2);
    let e = ChainEconomy::new(chain).map_err(|e| format!("the chain does not build: {e}"))?;
    let q = match e
        .solve()
        .map_err(|e| format!("the chain's oracle fails: {e}"))?
    {
        Regime::Interior(q) => q,
        other => {
            return Err(format!(
                "the chain's oracle finds no interior equilibrium ({})",
                other.name()
            ))
        }
    };
    let good = |k: &str| {
        q.good(k)
            .map(|g| (g.price, g.output))
            .ok_or_else(|| format!("the chain has no {k}"))
    };
    let (price, y_good) = good("GOOD")?;
    let (pf, qf) = if p.fodder {
        good("FODDER")?
    } else {
        (f64::NAN, 0.0)
    };
    let m = q.machine("HORSE").ok_or("the chain has no horse")?;
    let eq = &q.eq;
    let hours_row = e.chain().materials.len() + e.chain().machines.len();
    let task_hours = eq
        .types
        .get(hours_row)
        .map(|t| t.task_services)
        .ok_or("the chain has no hours row")?;
    let (kappa, d) = (p.kappa, p.delta);
    let r = p.rule;
    Ok(ChainPoint {
        x_star: eq.x_star,
        one_minus_x: eq.one_minus_x_star,
        v: eq.v,
        p_s: eq.p_s,
        y: eq.y,
        n_a: eq.n_a,
        participation: eq.participation,
        p: price,
        good: y_good,
        pf,
        pk: m.price,
        ph: m.hour_price,
        o: m.operating_cost,
        qf,
        made: m.made,
        sold: m.made * (1.0 - r.own_hours * d / kappa),
        hours: m.hours,
        task_hours,
        capacity: task_hours / kappa,
        serving: r.own_hours * m.made / kappa,
        provider_baskets: eq.provider_baskets,
        worker_baskets: eq.worker_baskets,
        funded: eq.funded,
    })
}
