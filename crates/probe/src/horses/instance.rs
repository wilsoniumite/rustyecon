//! The stocks probe's instances (HORSES-SPEC §1): GOODS-CHAIN §5's rule A on a county, the
//! horse as a durable good held as a stock, and their known answers from oracle unit 1g.
//!
//! An [`Instance`] is a county (its households, its task line and its flow machine (a, λ, b)
//! per tick at 52 a year, as P2.0 registered Appendix B's), the horse's wear δ and hours κ a
//! year, and ω, the share of the machine's land its horse eats as fodder while it works. Rule A
//! makes the flow machine concrete: fodder is made from ω·b land alone; a head is built from
//! a·κ/δ of the maker's own horse-days, λ·κ/δ labour and (1 − ω)·b·κ/δ pasture, κ and δ per
//! tick; a horse-day runs on one unit of fodder. So there is no loop (D-G1), and at ρ = 0 the
//! chain's equilibrium is the county's (the collapse, R1b).
//!
//! The oracle sees an instance through [`Instance::chain`]: unit 1g's `GoodsChain` on the same
//! per-tick numbers the tape registers, solved by `ChainEconomy` here, in the harness, outside
//! any `Sim`; no agent reads it (R13). An instance whose horse lives one tick (δ = 1 a tick, the
//! stocks layer off: R1a) is the flow economy, and its oracle is the markets probe's I0 on its
//! numbers ([`Instance::flow`]), so that its targets are P2.1's bit for bit.

use crate::markets::instance::Instance as Flow;
use crate::setup::clock;
use oracle::{
    Category, ChainCategory, ChainEconomy, GoodsChain, GoodsRecipe, Machine, Material,
    PowerSchedule, Regime, UniformWorkCost,
};
use rustyecon_core::{FlowPerYear, FractionPerYear};

/// A county: its households, its task line and its flow machine, as P2.0 registered Appendix B's
/// (N and T per year, the rest per tick at 52 a year and dimensionless).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct County {
    /// N, potential hours per year.
    pub workers: f64,
    /// T, land services per year.
    pub land: f64,
    /// h, the space in one basket.
    pub space: f64,
    /// χ_max.
    pub chi_max: f64,
    /// η.
    pub eta: f64,
    /// g0.
    pub g0: f64,
    /// g1.
    pub g1: f64,
    /// k.
    pub k: f64,
    /// a, the flow machine's own input per unit of its service.
    pub a: f64,
    /// λ, its hours.
    pub lam: f64,
    /// b, its land: the cost-shock target.
    pub b: f64,
}

impl County {
    /// SSRN Appendix B's county (A0): N 4 and T 10 a tick at 52 a year, h 1, χ ~ U[0, 1],
    /// γ = 0.2 + 0.8x, (a, λ, b) = (0.3, 0.05, 0.4).
    pub const fn appendix_b() -> County {
        County {
            workers: 208.0,
            land: 520.0,
            space: 1.0,
            chi_max: 1.0,
            eta: 1.0,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
            a: 0.3,
            lam: 0.05,
            b: 0.4,
        }
    }

    /// Demo v1's base county (docs/demo/WORLD.md §2): N 4.5 and T 10 a tick at 52 a year, h
    /// 0.15, η 2, (a, λ, b) = (0.3, 0.03, 0.8).
    pub const fn v1_base() -> County {
        County {
            workers: 234.0,
            land: 520.0,
            space: 0.15,
            chi_max: 1.0,
            eta: 2.0,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
            a: 0.3,
            lam: 0.03,
            b: 0.8,
        }
    }
}

/// Which roles hold the horse (HORSES-SPEC §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    /// The capacity desk (M3, wet) holds the horses and sells horse-days to the good desk (the
    /// P2.0 `GoodDesk`), which assigns tasks after the fact (D-G6); the maker (M2) builds them.
    Wet,
    /// The good desk holds its own horses (the owner desk, M1); the maker builds them. R1a and
    /// the P7 family.
    Owner,
}

/// The goods and desks an instance's tape names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    /// The durable good (`horse`; I0's `mach` for R1a).
    pub horse: String,
    /// The maker's desk key (`maker`; I0's `mach` for R1a).
    pub maker: String,
}

/// The horse-days' good. Its key sorts after `labour`, so that the good desk, which takes its
/// labour budget before its machine services' (P2.0's `two_budgets`), takes them in the order
/// admission does (MARKETS-RULES §3; HORSES-SPEC §2.1).
pub const HOURS: &str = "traction";

/// One of HORSES-SPEC's economies.
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    /// Its id: `h1`–`h4`, `f1`–`f10`, `r1a`, `p7`, `p8`.
    pub id: String,
    /// What it is, in a line.
    pub title: String,
    /// The county.
    pub county: County,
    /// The county's basis.
    pub county_basis: String,
    /// δ, the horse's wear a year (1 for the flow path: all of it a tick).
    pub delta: f64,
    /// ω, the share of the machine's land its horse eats as fodder.
    pub omega: f64,
    /// κ, horse-days a head a year.
    pub kappa: f64,
    /// Who holds the horses.
    pub config: Config,
    /// Its goods' and desks' keys.
    pub keys: Keys,
    /// Its registered dial set.
    pub dials: String,
}

/// The county's basis, and Appendix B's.
pub const SCALARS: &str = "SSRN 7226858 App. B via laborformal 31b3482";
/// Demo v1's base county's basis.
pub const V1_BASIS: &str = "docs/demo/WORLD.md §2: demo v1's base county";
/// Rule A's basis (HORSES-SPEC §2.10): never scored (R5, decision 187).
pub const RULE_A: &str = "GOODS-CHAIN §5 rule A, 2026-09-27; HORSES-SPEC 2026-09-28";

fn keys(horse: &str, maker: &str) -> Keys {
    Keys {
        horse: horse.into(),
        maker: maker.into(),
    }
}

impl Instance {
    #[allow(clippy::too_many_arguments)]
    fn make(
        id: &str,
        title: &str,
        county: County,
        basis: &str,
        delta: f64,
        omega: f64,
        config: Config,
        dials: &str,
    ) -> Instance {
        let (horse, maker) = if delta == 1.0 {
            ("mach", "mach")
        } else {
            ("horse", "maker")
        };
        Instance {
            id: id.into(),
            title: title.into(),
            county,
            county_basis: basis.into(),
            delta,
            omega,
            kappa: 52.0,
            config,
            keys: keys(horse, maker),
            dials: dials.into(),
        }
    }

    /// The registered instance with this id (HORSES-SPEC §1.1, §1.2).
    pub fn named(id: &str) -> Result<Instance, String> {
        let a0 = County::appendix_b();
        let v1 = County::v1_base();
        let wet = |id: &str, title: &str, d: f64, w: f64, dials: &str| {
            Instance::make(id, title, a0, SCALARS, d, w, Config::Wet, dials)
        };
        Ok(match id {
            "h1" => wet(
                "h1",
                "A0 under rule A, δ 10%, ω 1/2 (verdict)",
                0.1,
                0.5,
                "c2g",
            ),
            "h2" => wet(
                "h2",
                "A0 under rule A, δ 10%, ω 1 (verdict)",
                0.1,
                1.0,
                "c2g",
            ),
            "h3" => wet(
                "h3",
                "A0 under rule A, δ 8%, ω 1/2 (verdict)",
                0.08,
                0.5,
                "c2g",
            ),
            "h4" => wet(
                "h4",
                "A0 under rule A, δ 8%, ω 1 (verdict)",
                0.08,
                1.0,
                "c2g",
            ),
            "f1" => wet("f1", "A0, δ 4%, ω 1/2 (family, P2)", 0.04, 0.5, "c2g"),
            "f2" => wet("f2", "A0, δ 4%, ω 1 (family, P2)", 0.04, 1.0, "c2g"),
            "f3" => wet(
                "f3",
                "A0, δ 10%, the chain's ω 0.85 (family)",
                0.1,
                0.85,
                "c2g",
            ),
            "f4" => wet(
                "f4",
                "A0, δ 8%, the chain's ω 0.85 (family)",
                0.08,
                0.85,
                "c2g",
            ),
            "f5" => Instance::make(
                "f5",
                "v1's base county, δ 8%, ω 0.85 (family)",
                v1,
                V1_BASIS,
                0.08,
                0.85,
                Config::Wet,
                "c2g",
            ),
            "f6" => Instance::make(
                "f6",
                "v1's base county, δ 10%, ω 0.85 (family)",
                v1,
                V1_BASIS,
                0.1,
                0.85,
                Config::Wet,
                "c2g",
            ),
            "f7" => wet(
                "f7",
                "H1 with fodder's price at 1.3 a year",
                0.1,
                0.5,
                "c2g13",
            ),
            "f8" => wet(
                "f8",
                "H2 with fodder's price at 1.3 a year",
                0.1,
                1.0,
                "c2g13",
            ),
            "f9" => wet(
                "f9",
                "H3 with fodder's price at 1.3 a year",
                0.08,
                0.5,
                "c2g13",
            ),
            "f10" => wet(
                "f10",
                "H4 with fodder's price at 1.3 a year",
                0.08,
                1.0,
                "c2g13",
            ),
            "r1a" => Instance::make(
                "r1a",
                "the stocks layer off: M1 and M2 at δ = 1 a tick, ω 0, on I0 (R1a)",
                a0,
                SCALARS,
                1.0,
                0.0,
                Config::Owner,
                "c2",
            ),
            "p7" => Instance::make(
                "p7",
                "the first draft's default: M1 and M2 at δ 4%, ω 1, C2 (P7)",
                a0,
                SCALARS,
                0.04,
                1.0,
                Config::Owner,
                "c2",
            ),
            "p8" => {
                let mut c = a0;
                c.a = 0.005;
                Instance::make(
                    "p8",
                    "a maker from bought inputs: a 0.005, δ 4%, ω 1 (P8)",
                    c,
                    SCALARS,
                    0.04,
                    1.0,
                    Config::Wet,
                    "c2g",
                )
            }
            _ => return Err(format!("no instance {id}: h1-h4, f1-f10, r1a, p7 or p8")),
        })
    }

    /// Every registered instance's id, in HORSES-SPEC §1.1's order.
    pub const IDS: [&'static str; 17] = [
        "h1", "h2", "h3", "h4", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "r1a",
        "p7", "p8",
    ];

    /// The verdict instances (HORSES-SPEC §7.9).
    pub const VERDICT: [&'static str; 4] = ["h1", "h2", "h3", "h4"];

    /// Whether the horse lives one tick: the flow path, the stocks layer off (R1a).
    pub fn is_flow(&self) -> bool {
        self.delta == 1.0
    }

    /// Whether fodder is a traded good (ω > 0).
    pub fn has_fodder(&self) -> bool {
        self.omega > 0.0
    }

    /// The horse's hours per head and wear per tick at `tpy`.
    pub fn per_tick(&self, tpy: u32) -> Result<(f64, f64), String> {
        let c = clock(tpy)?;
        Ok((
            c.flow(FlowPerYear(self.kappa)),
            c.fraction(FractionPerYear(self.delta)),
        ))
    }

    /// Rule A's coefficients at `tpy` (HORSES-SPEC §1.2): fodder's land per unit, and a head's
    /// own horse-days, labour and pasture. On the flow path a head is the probe's one-tick
    /// service, and its recipe is the flow machine's (a, λ, b) (HORSES-SPEC §2.6).
    pub fn rule_a(&self, tpy: u32) -> Result<RuleA, String> {
        let (kappa, d) = self.per_tick(tpy)?;
        let (a, lam, b, w) = (self.county.a, self.county.lam, self.county.b, self.omega);
        if self.is_flow() {
            return Ok(RuleA {
                fodder_land: w * b,
                own_hours: a,
                labour: lam,
                pasture: b,
            });
        }
        Ok(RuleA {
            fodder_land: w * b,
            own_hours: a * kappa / d,
            labour: lam * kappa / d,
            pasture: (1.0 - w) * b * kappa / d,
        })
    }

    /// The markets, in the harness's order: labour, land, fodder (where traded), the horse, its
    /// hours (wet), the good. HORSES-SPEC §7.1's order; for R1a I0's, [labour, land, mach, good].
    pub fn markets(&self) -> Vec<String> {
        let mut m = vec!["labour".to_string(), "land".to_string()];
        if self.has_fodder() {
            m.push("fodder".into());
        }
        m.push(self.keys.horse.clone());
        if self.config == Config::Wet {
            m.push(HOURS.into());
        }
        m.push("good".into());
        m
    }

    /// The desks, in the harness's order: the good desk, the capacity desk (wet), the maker,
    /// the fodder desk (where traded). For R1a I0's, [good, mach].
    pub fn desks(&self) -> Vec<String> {
        let mut d = vec!["good".to_string()];
        if self.config == Config::Wet {
            d.push("capacity".into());
        }
        d.push(self.keys.maker.clone());
        if self.has_fodder() {
            d.push("fodder".into());
        }
        d
    }

    /// The actors, in the harness's order: each desk, then the provider and the workers.
    pub fn actors(&self) -> Vec<String> {
        let mut a: Vec<String> = self.desks().iter().map(|d| format!("desk.{d}")).collect();
        a.push("provider".into());
        a.push("workers".into());
        a
    }

    /// The instance with the county's machine land at `b` (a cost shock's target).
    pub fn with_b(&self, b: f64) -> Instance {
        let mut i = self.clone();
        i.county.b = b;
        i
    }

    /// Unit 1g's chain of the instance at `tpy` (HORSES-SPEC §1.2), per tick, with the horse's
    /// hours built at J = `lag` ticks (J = 2 for the capacity desk's hours, D-G8 item 4 and
    /// decision 232: at ρ = 0 every price and quantity is J 1's).
    pub fn chain(&self, tpy: u32, lag: u32) -> Result<GoodsChain, String> {
        let c = clock(tpy)?;
        let r = self.rule_a(tpy)?;
        let (kappa, d) = self.per_tick(tpy)?;
        let n = self.county;
        let goods = |inputs: Vec<(&str, f64)>, labor: f64, land: f64| GoodsRecipe {
            inputs: inputs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            labor,
            land,
        };
        let (materials, operating) = if self.has_fodder() {
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
        Ok(GoodsChain {
            workers: c.flow(FlowPerYear(n.workers)),
            land: c.flow(FlowPerYear(n.land)),
            schedule: PowerSchedule {
                eta: n.eta,
                g0: n.g0,
                g1: n.g1,
                k: n.k,
            },
            work_cost: UniformWorkCost { chi_max: n.chi_max },
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
                        weight: n.space,
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
                hours_per_period: kappa,
                task_efficiency: 1.0,
                operating,
                delta: d,
                build_lag: lag,
            }],
        })
    }

    /// The markets probe's I0 on this instance's county: the flow economy the stocks layer off
    /// must reproduce (R1a), whose oracle is unit 1c's operating form, unit 1a's G1 bit for bit.
    pub fn flow(&self) -> Result<Flow, String> {
        let n = self.county;
        let mut f = Flow::named("i0")?;
        f.workers = n.workers;
        f.land = n.land;
        f.chi_max = n.chi_max;
        f.eta = n.eta;
        f.g0 = n.g0;
        f.g1 = n.g1;
        f.k = n.k;
        f.space = Some(n.space);
        f.types[0].row[0] = n.a;
        f.types[0].labour = n.lam;
        f.types[0].land = n.b;
        Ok(f)
    }

    /// The oracle's interior equilibrium at `tpy` ticks a year, per tick, relative to r = 1.
    pub fn point(&self, tpy: u32) -> Result<Point, String> {
        if self.is_flow() {
            let e = self.flow()?.point(tpy)?;
            return Ok(Point {
                x_star: e.x_star,
                one_minus_x: e.one_minus_x,
                v: e.v,
                p_s: e.p_s,
                y: e.y,
                n_a: e.n_a,
                p: e.cat_price[0],
                good: e.cat_output[0],
                pf: f64::NAN,
                pk: e.type_price[0],
                ph: f64::NAN,
                o: f64::NAN,
                qf: 0.0,
                made: e.type_services[0],
                sold: e.type_traded[0],
                hours: e.type_services[0],
                task_hours: e.type_traded[0],
                capacity: 0.0,
                serving: 0.0,
                provider_baskets: e.provider_baskets,
                worker_baskets: e.worker_baskets,
                funded: e.provider_baskets > 0.0,
                flow: Some(e),
            });
        }
        let r = self.rule_a(tpy)?;
        let (kappa, d) = self.per_tick(tpy)?;
        let chain = self.chain(tpy, 2)?;
        let e = ChainEconomy::new(chain).map_err(|e| format!("{}: {e}", self.id))?;
        let q = match e.solve().map_err(|e| format!("{}: {e}", self.id))? {
            Regime::Interior(q) => q,
            other => {
                return Err(format!(
                    "{}: no interior equilibrium ({})",
                    self.id,
                    other.name()
                ))
            }
        };
        let good = |k: &str| {
            q.good(k)
                .map(|g| (g.price, g.output))
                .ok_or_else(|| format!("{}: the chain has no {k}", self.id))
        };
        let (p, y_good) = good("GOOD")?;
        let (pf, qf) = if self.has_fodder() {
            good("FODDER")?
        } else {
            (f64::NAN, 0.0)
        };
        let m = q
            .machine("HORSE")
            .ok_or_else(|| format!("{}: the chain has no horse", self.id))?;
        let eq = &q.eq;
        let hours_row = e.chain().materials.len() + e.chain().machines.len();
        let task_hours = eq
            .types
            .get(hours_row)
            .map(|t| t.task_services)
            .ok_or_else(|| format!("{}: no hours row", self.id))?;
        let serving = r.own_hours * m.made / kappa;
        Ok(Point {
            x_star: eq.x_star,
            one_minus_x: eq.one_minus_x_star,
            v: eq.v,
            p_s: eq.p_s,
            y: eq.y,
            n_a: eq.n_a,
            p,
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
            serving,
            provider_baskets: eq.provider_baskets,
            worker_baskets: eq.worker_baskets,
            funded: eq.funded,
            flow: None,
        })
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

/// The oracle's equilibrium, as the harness reads it: per tick, relative to r = 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
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
    /// p, the good's price.
    pub p: f64,
    /// The good's output.
    pub good: f64,
    /// Fodder's price (NaN where it is not traded).
    pub pf: f64,
    /// A head's price, p_K.
    pub pk: f64,
    /// A horse-day's price, p_h = O + δ·p_K/κ (NaN on the flow path).
    pub ph: f64,
    /// A horse-day's running cost, O (NaN on the flow path).
    pub o: f64,
    /// Fodder made a tick.
    pub qf: f64,
    /// Heads made a tick, q_b.
    pub made: f64,
    /// Heads sold a tick: q_b less those the maker keeps to replace its own, q_b·(1 − δ·a/κ).
    pub sold: f64,
    /// Every horse-day a tick, the maker's own included.
    pub hours: f64,
    /// The horse-days the tasks use, Y·J(x\*).
    pub task_hours: f64,
    /// The heads that do the tasks, Y·J(x\*)/κ: the capacity desk's, or the owner desk's.
    pub capacity: f64,
    /// The maker's serving stock, a·q_b/κ.
    pub serving: f64,
    /// The provider's baskets, T/P_s − N.
    pub provider_baskets: f64,
    /// The workers' baskets.
    pub worker_baskets: f64,
    /// Whether the transfer is funded at the point.
    pub funded: bool,
    /// On the flow path, the markets probe's point it is.
    pub flow: Option<crate::markets::instance::Point>,
}
