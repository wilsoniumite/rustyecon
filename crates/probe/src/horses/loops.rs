//! The loop step's instances and tapes (P2.2b; docs/probe/LOOPS-RULES.md §2, §7; LOOP-SPEC §4;
//! FUNDED's chain8): GOODS-CHAIN's rule B on the funded county, fodder raised with horse-days
//! (the loop), the horse held by M3's wet capacity desk and bred by M2 from bought fodder, with
//! CAPACITY's plant on the fodder desk, the capacity desk and the maker; and the flow control,
//! the stocks layer off, with a horse-day desk in place of the horse.
//!
//! An [`Instance`] is a row of LOOPS-RULES §7.1. Its oracle point is FUNDED's: unit 1g's
//! `ChainEconomy` on the chain the tape registers, solved here, outside any `Sim` (R13); at ρ 0
//! the plants at s1 leave it unchanged (decision 184), and the flow control's point is B's
//! (FUNDED §4.2). [`tape_ron`] writes a [`Setup`] as a tape whose genesis is that point, with
//! each desk's stock at rest, each plant at κ_p·X from the price-free rest ratio, and each actor's
//! stationary coin summed in its rule's order (LOOPS-RULES §7.4). `tapes/loops-<id>.ron` is
//! `tape_ron(&Setup::registered(id, 52))` for LB1–LB3 and LW1–LW3.
//!
//! The harness's observables and readouts for these instances (LOOPS-RULES §8) are not here: the
//! harness still reads rule A's instances only.

use crate::horses::instance::HOURS;
use crate::markets::setup::{Dial, Dials};
use crate::setup::{clock, OneSided, START};
use oracle::{
    s1_size, Category, ChainCategory, ChainEconomy, GoodsChain, GoodsRecipe, Machine, Material,
    PowerSchedule, Regime, UniformWorkCost,
};
use rustyecon_core::{num, Clock, FlowPerYear, FractionPerYear, RatePerYear, Years};

/// chain8's basis (FUNDED §3; `instances.json`).
pub const COUNTY: &str = "funded 2026-09-29: 1g's HORSE county at N 8 (O41)";
/// Rule B's basis (FUNDED §4.1).
pub const RULE_B: &str = "CHAIN.md §1.3-1.4 via unit-1g.md §3.3; funded 2026-09-29";
/// The plants' basis (LOOPS-RULES §7.2).
pub const PLANTS: &str = "CAPACITY 2026-09-27; LOOP-SPEC 2026-09-29, mirror scan";
/// The flow control's horse-day recipe's basis (FUNDED §4.2).
pub const FLOW: &str = "FUNDED §4.2: the horse-day's running recipe with delta/kappa of a head's \
                        folded in; funded 2026-09-29";

const C2G: &str = "Assumed(\"GOODS-CHAIN C2g (D-G13); HORSES-SPEC §5\")";
const C2G13: &str =
    "Assumed(\"HORSES-SPEC §5.1: C2g with fodder's price at C2m's type rate (F7-F10)\")";
const STOCK: &str = "Assumed(\"GOODS-CHAIN D-G3, D-G5 (C2g); HORSES-SPEC §5.2\")";
const RESERVE: &str = "Assumed(\"IDLE-SPEC 2026-09-29, mirror scan\")";

/// chain8, per year where per year (FUNDED §3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct County {
    /// N, potential hours a year.
    pub workers: f64,
    /// T, land services a year.
    pub land: f64,
    /// h, the space in a basket.
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
}

/// chain8: 1g's HORSE county at N 8.
pub const CHAIN8: County = County {
    workers: 416.0,
    land: 520.0,
    space: 1.0,
    chi_max: 0.05,
    eta: 1.0,
    g0: 0.2,
    g1: 0.8,
    k: 1.0,
};

/// Rule B's recipes, per unit made at every tick length (decision 259), before the land factor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuleB {
    /// Horse-days a ton of fodder (0 with the loop cut).
    pub fodder_traction: f64,
    /// Labour a ton of fodder.
    pub fodder_labour: f64,
    /// Land a ton of fodder, at land factor 1.
    pub fodder_land: f64,
    /// Fodder a horse-day.
    pub run_fodder: f64,
    /// Labour a horse-day.
    pub run_labour: f64,
    /// Fodder a head.
    pub horse_fodder: f64,
    /// Labour a head.
    pub horse_labour: f64,
    /// Pasture a head, at land factor 1.
    pub horse_land: f64,
    /// κ, horse-days a head a year.
    pub kappa: f64,
}

/// Rule B's numbers (FUNDED §4.1).
pub const HORSE_B: RuleB = RuleB {
    fodder_traction: 2.0,
    fodder_labour: 8.0,
    fodder_land: 1.0,
    run_fodder: 0.0176,
    run_labour: 0.1,
    horse_fodder: 8.0,
    horse_labour: 20.0,
    horse_land: 3.0,
    kappa: 250.0,
};

/// Which roles the instance runs (LOOPS-RULES §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    /// The horse held by the capacity desk, bred by the maker (rule B).
    Wet,
    /// The flow control: a horse-day desk in place of the horse (LOOPS-RULES §2.2).
    Flow,
}

/// Which desks carry a plant: the capacity desk (the flow control's horse-day desk), the maker,
/// the fodder desk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plants {
    /// The capacity desk's plant, or the flow control's horse-day desk's.
    pub capacity: bool,
    /// The maker's.
    pub maker: bool,
    /// The fodder desk's.
    pub fodder: bool,
}

/// The plants' design and dials (LOOPS-RULES §7.1): θ, δ_p a year, the order and the capacity
/// desk's target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Design {
    /// θ.
    pub theta: f64,
    /// δ_p, the plants' wear a year.
    pub delta: f64,
    /// `Target` (M3's rule) or `Gap` (LN6).
    pub order: &'static str,
    /// The capacity desk's target: `Herd` or `Bundles` (LF6).
    pub target: &'static str,
}

/// The design: θ 0.8, δ_p 10% a year, M3's rule, the herd's plant.
pub const DESIGN: Design = Design {
    theta: 0.8,
    delta: 0.1,
    order: "Target",
    target: "Herd",
};

/// One of LOOPS-RULES §7.1's instances.
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    /// Its id: `lb1`–`lb3`, `lf1`–`lf8` but `lf4`, `lc1`, `lw0`–`lw3`, `ln1`–`ln9` but `ln5`.
    pub id: String,
    /// What it is, in a line.
    pub title: String,
    /// The county.
    pub county: County,
    /// The recipes, before the land factor.
    pub rule: RuleB,
    /// Whether the fodder desk buys horse-days (the loop); false is B-cut.
    pub looped: bool,
    /// δ, the horse's wear a year.
    pub delta: f64,
    /// b, the land factor on every land coefficient (1 at the registered point).
    pub b: f64,
    /// Which roles hold the horse.
    pub config: Config,
    /// Which desks are planted.
    pub plants: Plants,
    /// The plants' design.
    pub design: Design,
    /// The maker's reservation ψ; `None` is off (LF3).
    pub reserve: Option<f64>,
    /// Its registered dial set: `c2g` or `c2g13`.
    pub dials: String,
}

const ALL: Plants = Plants {
    capacity: true,
    maker: true,
    fodder: true,
};
const NONE: Plants = Plants {
    capacity: false,
    maker: false,
    fodder: false,
};

impl Instance {
    #[allow(clippy::too_many_arguments)]
    fn make(
        id: &str,
        title: &str,
        delta: f64,
        config: Config,
        looped: bool,
        plants: Plants,
        design: Design,
        dials: &str,
    ) -> Instance {
        Instance {
            id: id.into(),
            title: title.into(),
            county: CHAIN8,
            rule: HORSE_B,
            looped,
            delta,
            b: 1.0,
            config,
            plants,
            design,
            reserve: (config == Config::Wet).then_some(0.25),
            dials: dials.into(),
        }
    }

    /// The registered instance with this id (LOOPS-RULES §7.1). LF4 and LN5 are not built
    /// (LOOP-SPEC-A2).
    pub fn named(id: &str) -> Result<Instance, String> {
        use Config::{Flow, Wet};
        let d = DESIGN;
        let wet = |id: &str, title: &str, delta: f64| {
            Instance::make(id, title, delta, Wet, true, ALL, d, "c2g")
        };
        let th = |theta: f64| Design { theta, ..d };
        let flow_plants = Plants {
            capacity: true,
            maker: false,
            fodder: true,
        };
        Ok(match id {
            "lb1" => wet(
                "lb1",
                "rule B at chain8, δ 8%, three plants (verdict)",
                0.08,
            ),
            "lb2" => wet(
                "lb2",
                "rule B at chain8, δ 10%, three plants (verdict)",
                0.1,
            ),
            "lb3" => wet(
                "lb3",
                "rule B at chain8, δ 4%, three plants (verdict)",
                0.04,
            ),
            "lf1" => Instance::make(
                "lf1",
                "LB1 at θ 0.7 (family)",
                0.08,
                Wet,
                true,
                ALL,
                th(0.7),
                "c2g",
            ),
            "lf7" => Instance::make(
                "lf7",
                "LB2 at θ 0.7 (family)",
                0.1,
                Wet,
                true,
                ALL,
                th(0.7),
                "c2g",
            ),
            "lf8" => Instance::make(
                "lf8",
                "LB3 at θ 0.7 (family)",
                0.04,
                Wet,
                true,
                ALL,
                th(0.7),
                "c2g",
            ),
            "lf2" => Instance::make(
                "lf2",
                "LB1 with the plants' wear at 4% a year (family)",
                0.08,
                Wet,
                true,
                ALL,
                Design { delta: 0.04, ..d },
                "c2g",
            ),
            "lf3" => {
                let mut i = wet("lf3", "LB1 with no reservation (sensitivity)", 0.08);
                i.reserve = None;
                i
            }
            "lf5" => Instance::make(
                "lf5",
                "LB1 with fodder's price at 1.3 a year (family)",
                0.08,
                Wet,
                true,
                ALL,
                d,
                "c2g13",
            ),
            "lf6" => Instance::make(
                "lf6",
                "LB1 with the capacity desk's plant targeting its bundles (negative control)",
                0.08,
                Wet,
                true,
                ALL,
                Design {
                    target: "Bundles",
                    ..d
                },
                "c2g",
            ),
            "lc1" => Instance::make(
                "lc1",
                "LB1 with the loop cut (B-cut)",
                0.08,
                Wet,
                false,
                ALL,
                d,
                "c2g",
            ),
            "lw1" => Instance::make(
                "lw1",
                "the flow control at δ 8%, plants on the fodder and horse-day desks",
                0.08,
                Flow,
                true,
                flow_plants,
                d,
                "c2g",
            ),
            "lw2" => Instance::make(
                "lw2",
                "the flow control at δ 10%, plants on the fodder and horse-day desks",
                0.1,
                Flow,
                true,
                flow_plants,
                d,
                "c2g",
            ),
            "lw3" => Instance::make(
                "lw3",
                "the flow control at δ 4%, plants on the fodder and horse-day desks",
                0.04,
                Flow,
                true,
                flow_plants,
                d,
                "c2g",
            ),
            "lw0" => Instance::make(
                "lw0",
                "the flow control at δ 8% with no plants (negative control)",
                0.08,
                Flow,
                true,
                NONE,
                d,
                "c2g",
            ),
            "ln1" => Instance::make(
                "ln1",
                "LB1 with no plants (negative control)",
                0.08,
                Wet,
                true,
                NONE,
                d,
                "c2g",
            ),
            "ln2" => Instance::make(
                "ln2",
                "LB1 with plants on the fodder and capacity desks (placement)",
                0.08,
                Wet,
                true,
                Plants {
                    capacity: true,
                    maker: false,
                    fodder: true,
                },
                d,
                "c2g",
            ),
            "ln3" => Instance::make(
                "ln3",
                "LB1 with a plant on the fodder desk alone (placement)",
                0.08,
                Wet,
                true,
                Plants {
                    capacity: false,
                    maker: false,
                    fodder: true,
                },
                d,
                "c2g",
            ),
            "ln4" => Instance::make(
                "ln4",
                "LB1 with plants on the capacity desk and the maker (placement)",
                0.08,
                Wet,
                true,
                Plants {
                    capacity: true,
                    maker: true,
                    fodder: false,
                },
                d,
                "c2g",
            ),
            "ln6" => Instance::make(
                "ln6",
                "LB1 with every plant ordering its whole gap (negative control)",
                0.08,
                Wet,
                true,
                ALL,
                Design { order: "Gap", ..d },
                "c2g",
            ),
            "ln7" => Instance::make(
                "ln7",
                "the loop cut with no plants (negative control)",
                0.08,
                Wet,
                false,
                NONE,
                d,
                "c2g",
            ),
            "ln8" => Instance::make(
                "ln8",
                "the loop cut with no plants, fodder's price at 1.3 a year (negative control)",
                0.08,
                Wet,
                false,
                NONE,
                d,
                "c2g13",
            ),
            "ln9" => Instance::make(
                "ln9",
                "LB1 with no plants, fodder's price at 1.3 a year (negative control)",
                0.08,
                Wet,
                true,
                NONE,
                d,
                "c2g13",
            ),
            "lf4" | "ln5" => {
                return Err(format!(
                    "{id} needs storable fodder (STATE O51), which P2.2b does not build \
                     (LOOP-SPEC-A2)"
                ))
            }
            _ => {
                return Err(format!(
                    "no loop instance {id}: lb1-lb3, lf1-lf8 but lf4, lc1, lw0-lw3, ln1-ln9 but \
                     ln5"
                ))
            }
        })
    }

    /// Every built instance's id, in LOOPS-RULES §7.1's order.
    pub const IDS: [&'static str; 23] = [
        "lb1", "lb2", "lb3", "lf1", "lf7", "lf8", "lf2", "lf3", "lf5", "lf6", "lc1", "lw1", "lw2",
        "lw3", "lw0", "ln1", "ln2", "ln3", "ln4", "ln6", "ln7", "ln8", "ln9",
    ];

    /// The instances whose tapes are committed (LOOPS-RULES §7.6).
    pub const TAPES: [&'static str; 6] = ["lb1", "lb2", "lb3", "lw1", "lw2", "lw3"];

    /// Whether `id` names a loop instance, built or not.
    pub fn is_loop(id: &str) -> bool {
        Instance::IDS.contains(&id) || id == "lf4" || id == "ln5"
    }

    /// Whether it is the flow control.
    pub fn is_flow(&self) -> bool {
        self.config == Config::Flow
    }

    /// The markets, in the harness's order: labour, land, fodder, the horse (wet), the
    /// horse-days, the good.
    pub fn markets(&self) -> Vec<String> {
        let mut m = vec!["labour".to_string(), "land".into(), "fodder".into()];
        if !self.is_flow() {
            m.push("horse".into());
        }
        m.push(HOURS.into());
        m.push("good".into());
        m
    }

    /// The desks, in the harness's order: the good desk, the capacity desk and the maker (wet)
    /// or the horse-day desk (flow), the fodder desk.
    pub fn desks(&self) -> Vec<String> {
        let mut d = vec!["good".to_string()];
        if self.is_flow() {
            d.push(HOURS.into());
        } else {
            d.push("capacity".into());
            d.push("maker".into());
        }
        d.push("fodder".into());
        d
    }

    /// The actors, in the harness's order: each desk, then the provider and the workers.
    pub fn actors(&self) -> Vec<String> {
        let mut a: Vec<String> = self.desks().iter().map(|d| format!("desk.{d}")).collect();
        a.push("provider".into());
        a.push("workers".into());
        a
    }

    /// The planted desks, in desk order (capacity or traction, maker, fodder): each is the key
    /// `d` of its plant good `plant.<d>` and its params.
    pub fn planted(&self) -> Vec<String> {
        let mut v = Vec::new();
        if self.plants.capacity {
            v.push(if self.is_flow() { HOURS } else { "capacity" }.to_string());
        }
        if self.plants.maker && !self.is_flow() {
            v.push("maker".into());
        }
        if self.plants.fodder {
            v.push("fodder".into());
        }
        v
    }

    /// The instance with the land factor at `b`.
    pub fn with_b(&self, b: f64) -> Instance {
        let mut i = self.clone();
        i.b = b;
        i
    }

    /// κ and δ a tick at `tpy`.
    pub fn per_tick(&self, tpy: u32) -> Result<(f64, f64), String> {
        let c = clock(tpy)?;
        Ok((
            c.flow(FlowPerYear(self.rule.kappa)),
            c.fraction(FractionPerYear(self.delta)),
        ))
    }

    /// Horse-days a ton of fodder: 2, or 0 with the loop cut.
    pub fn fodder_traction(&self) -> f64 {
        if self.looped {
            self.rule.fodder_traction
        } else {
            0.0
        }
    }

    /// The flow control's horse-day recipe at `tpy` (LOOPS-RULES §2.2; `flow_recipe`,
    /// `lm_mirror:68–71`): fo + fI·(δ/κ), lo + lI·(δ/κ) and (bI·b)·(δ/κ), δ/κ formed first.
    pub fn flow_recipe(&self, tpy: u32) -> Result<(f64, f64, f64), String> {
        let (kappa, d) = self.per_tick(tpy)?;
        let r = self.rule;
        let dk = d / kappa;
        Ok((
            r.run_fodder + r.horse_fodder * dk,
            r.run_labour + r.horse_labour * dk,
            (r.horse_land * self.b) * dk,
        ))
    }

    /// Unit 1g's chain at `tpy` (FUNDED §4.1; LOOPS-RULES §7.5), per tick, with the horse's
    /// hours built at J = `lag` ticks (J 2 is J 1 bit for bit at ρ 0).
    pub fn chain(&self, tpy: u32, lag: u32) -> Result<GoodsChain, String> {
        let c = clock(tpy)?;
        let (kappa, d) = self.per_tick(tpy)?;
        let r = self.rule;
        let n = self.county;
        let goods = |inputs: Vec<(&str, f64)>, labor: f64, land: f64| GoodsRecipe {
            inputs: inputs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            labor,
            land,
        };
        let fodder_in = if self.looped {
            vec![("HORSE_DAYS", r.fodder_traction)]
        } else {
            vec![]
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
            materials: vec![Material {
                key: "FODDER".into(),
                recipe: goods(fodder_in, r.fodder_labour, r.fodder_land * self.b),
            }],
            machines: vec![Machine {
                key: "HORSE".into(),
                build: goods(
                    vec![("FODDER", r.horse_fodder)],
                    r.horse_labour,
                    r.horse_land * self.b,
                ),
                hours: "HORSE_DAYS".into(),
                hours_per_period: kappa,
                task_efficiency: 1.0,
                operating: goods(vec![("FODDER", r.run_fodder)], r.run_labour, 0.0),
                delta: d,
                build_lag: lag,
            }],
        })
    }

    /// The oracle's interior equilibrium at `tpy`, per tick, relative to r = 1 (LOOPS-RULES
    /// §7.5). The flow control's is B's (FUNDED §4.2).
    pub fn point(&self, tpy: u32) -> Result<Point, String> {
        let (kappa, _) = self.per_tick(tpy)?;
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
        let (pf, qf) = good("FODDER")?;
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
            hours: m.hours,
            task_hours,
            fodder_hours: self.fodder_traction() * qf,
            capacity: m.hours / kappa,
            provider_baskets: eq.provider_baskets,
            worker_baskets: eq.worker_baskets,
            funded: eq.funded,
        })
    }
}

/// The oracle's equilibrium, as the tapes and the tests read it: per tick, relative to r = 1.
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
    /// Fodder's price.
    pub pf: f64,
    /// A head's price, p_K.
    pub pk: f64,
    /// A horse-day's price, p_h = O + δ·p_K/κ.
    pub ph: f64,
    /// A horse-day's running cost, O.
    pub o: f64,
    /// Fodder made a tick, q_f.
    pub qf: f64,
    /// Heads made a tick, q_b.
    pub made: f64,
    /// Every horse-day a tick, X.
    pub hours: f64,
    /// The horse-days the tasks use, Y·J(x\*).
    pub task_hours: f64,
    /// The horse-days the fodder desk uses, 2·q_f (0 with the loop cut).
    pub fodder_hours: f64,
    /// The heads that make every horse-day, X/κ.
    pub capacity: f64,
    /// The provider's baskets.
    pub provider_baskets: f64,
    /// The workers' baskets.
    pub worker_baskets: f64,
    /// Whether the transfer is funded at the point.
    pub funded: bool,
}

/// The dials of an instance (LOOPS-RULES §7.3): C2g as P2.2a registered it (`c2g`, or `c2g13`
/// with fodder's rate at 1.3), each desk's turnover and tilt, household spending, the capacity
/// desk's s_K at 2δ and the maker's s_Km at 0 and cover at 4 weeks (wet), and each plant's s_Kp
/// at 2·δ_p (`plant_delta`, a year).
pub fn dials(inst: &Instance, name: &str, plant_delta: f64) -> Result<Dials, String> {
    let (fodder, basis) = match name {
        "c2g" => (5.2, C2G),
        "c2g13" => (1.3, C2G13),
        _ => {
            return Err(format!(
                "no dial set {name} for a loop instance: c2g or c2g13"
            ))
        }
    };
    let rate = |key: String, value: f64, basis: &str| Dial {
        key,
        value,
        unit: "RatePerYear",
        basis: basis.into(),
    };
    let mut v = vec![
        rate("rate.labour".into(), 5.2, basis),
        rate("rate.land".into(), 0.1625, basis),
    ];
    for m in &inst.markets()[2..] {
        v.push(match m.as_str() {
            "good" => rate("rate.good".into(), 2.6, basis),
            "fodder" => rate("rate.fodder".into(), fodder, basis),
            _ => rate(format!("rate.{m}"), 5.2, basis),
        });
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
    if !inst.is_flow() {
        v.push(rate(
            "adjust.invest.capacity".into(),
            2.0 * inst.delta,
            STOCK,
        ));
        v.push(rate("adjust.invest.maker".into(), 0.0, STOCK));
        v.push(Dial {
            key: "cover.maker".into(),
            value: 4.0 / 52.0,
            unit: "Years",
            basis: STOCK.into(),
        });
    }
    let plants = format!("Assumed(\"{PLANTS}\")");
    for d in inst.planted() {
        v.push(rate(
            format!("adjust.plant.{d}"),
            2.0 * plant_delta,
            &plants,
        ));
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
    pub share: f64,
    /// Each actor's coin, in [`Instance::actors`] order.
    pub coin: Vec<f64>,
    /// Each stock, in [`stocks`] order.
    pub stock: Vec<f64>,
}

impl Displacement {
    /// No displacement.
    pub fn none(inst: &Instance) -> Displacement {
        Displacement {
            price: vec![1.0; inst.markets().len()],
            share: 1.0,
            coin: vec![1.0; inst.actors().len()],
            stock: vec![1.0; stocks(inst).len()],
        }
    }

    /// Multiply the genesis price of `market` by `f`.
    pub fn price_of(&mut self, inst: &Instance, market: &str, f: f64) -> Result<(), String> {
        let i = position(&inst.markets(), market)?;
        self.price[i] *= f;
        Ok(())
    }

    /// Multiply the genesis coin of `actor` by `f`.
    pub fn coin_of(&mut self, inst: &Instance, actor: &str, f: f64) -> Result<(), String> {
        let i = position(&inst.actors(), actor)?;
        self.coin[i] *= f;
        Ok(())
    }

    /// Multiply the genesis stock `name` ([`stocks`]) by `f`.
    pub fn stock_of(&mut self, inst: &Instance, name: &str, f: f64) -> Result<(), String> {
        let i = position(&stocks(inst), name)?;
        self.stock[i] *= f;
        Ok(())
    }
}

fn position(list: &[String], name: &str) -> Result<usize, String> {
    list.iter()
        .position(|x| x == name)
        .ok_or_else(|| format!("no {name} in {}", list.join(", ")))
}

/// The stocks a run can displace, by the name the run grammar gives them (LOOPS-RULES §8.4):
/// `stock.good`, `heads.capacity`, `hours.capacity`, `finished.maker`, `stock.fodder` (wet), or
/// `stock.good`, `stock.traction`, `stock.fodder` (flow); then `plant.<d>` for each planted desk.
pub fn stocks(inst: &Instance) -> Vec<String> {
    let mut v = vec!["stock.good".to_string()];
    if inst.is_flow() {
        v.push(format!("stock.{HOURS}"));
    } else {
        v.push("heads.capacity".into());
        v.push("hours.capacity".into());
        v.push("finished.maker".into());
    }
    v.push("stock.fodder".into());
    v.extend(inst.planted().iter().map(|d| format!("plant.{d}")));
    v
}

/// Everything a loop tape is made from.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    /// Ticks per year.
    pub tpy: u32,
    /// The instance, whose oracle point seeds genesis.
    pub instance: Instance,
    /// The dials.
    pub dials: Dials,
    /// The one-sided rule (`Saturate`: the reservation needs it).
    pub one_sided: OneSided,
    /// The plants' θ on the tape (`--theta`); the design's by default.
    pub theta: f64,
    /// The plants' δ_p a year on the tape (`--plant-delta`); the design's by default.
    pub plant_delta: f64,
    /// `--fixed-plants`: δ_p 0 on every plant, genesis at the registered design's plants and
    /// sizes (LOOPS-RULES §3.9, §8.4).
    pub fixed_plants: bool,
    /// The displacement of genesis.
    pub displace: Displacement,
    /// The land factor the tape registers from tick 0 (a cost shock at genesis); genesis stays
    /// at the registered point.
    pub b_genesis: Option<f64>,
}

impl Setup {
    /// The registered setup of an instance at `tpy` ticks a year: its dials, Saturate, the
    /// design's plants, genesis at the oracle's point.
    pub fn registered(id: &str, tpy: u32) -> Result<Setup, String> {
        let instance = Instance::named(id)?;
        let dials = dials(&instance, &instance.dials, instance.design.delta)?;
        Ok(Setup {
            tpy,
            dials,
            one_sided: OneSided::Saturate,
            theta: instance.design.theta,
            plant_delta: instance.design.delta,
            fixed_plants: false,
            displace: Displacement::none(&instance),
            b_genesis: None,
            instance,
        })
    }

    /// Every plant at θ = `theta`: its size, s_Kp and genesis follow.
    pub fn with_theta(mut self, theta: f64) -> Setup {
        self.theta = theta;
        self
    }

    /// Every plant at δ_p = `delta` a year: its size, s_Kp and genesis follow.
    pub fn with_plant_delta(mut self, delta: f64) -> Result<Setup, String> {
        self.plant_delta = delta;
        self.dials = dials(&self.instance, &self.dials.set, delta)?;
        Ok(self)
    }

    /// `--fixed-plants`: δ_p 0 and s_Kp 0 on every plant, genesis at the design's plants.
    pub fn with_fixed_plants(mut self) -> Result<Setup, String> {
        self.fixed_plants = true;
        self.dials = dials(&self.instance, &self.dials.set, 0.0)?;
        Ok(self)
    }

    /// The instance the tape registers from tick 0.
    pub fn tape_instance(&self) -> Instance {
        self.instance
            .with_b(self.b_genesis.unwrap_or(self.instance.b))
    }

    /// The cover in ticks the maker reads: `cover.maker` through the clock (0 in the flow
    /// control).
    pub fn cover_ticks(&self) -> Result<f64, String> {
        if self.instance.is_flow() {
            return Ok(0.0);
        }
        let years = self.dials.get("cover.maker")?;
        let t = clock(self.tpy)?
            .ticks(Years(years))
            .map_err(|e| e.to_string())?;
        Ok(f64::from(t))
    }

    /// The plants' size, wear a tick and adjustment a year on the tape: s1 at (θ, u) with
    /// u = `Clock::fraction`(δ_p), 1 at θ = 1 (`lm_mirror:127–128`); with fixed plants the
    /// design's s1, u 0 and s_Kp 0.
    pub fn plant_params(&self) -> Result<PlantParams, String> {
        let c = clock(self.tpy)?;
        let u_design = c.fraction(FractionPerYear(self.plant_delta));
        let size = if self.theta < 1.0 {
            s1_size(self.theta, u_design)
        } else {
            1.0
        };
        Ok(if self.fixed_plants {
            PlantParams {
                theta: self.theta,
                delta: 0.0,
                u: 0.0,
                size,
                u_design,
            }
        } else {
            PlantParams {
                theta: self.theta,
                delta: self.plant_delta,
                u: u_design,
                size,
                u_design,
            }
        })
    }
}

/// The plants' params on a tape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlantParams {
    /// θ.
    pub theta: f64,
    /// δ_p a year, as the tape writes it.
    pub delta: f64,
    /// u, δ_p a tick, as the rule reads it.
    pub u: f64,
    /// s.
    pub size: f64,
    /// u at the design's δ_p, which sets the genesis plants.
    pub u_design: f64,
}

impl PlantParams {
    /// κ_p = kr^θ at the price-free rest ratio kr = (1 − θ)/((θ·u)·s) at the design's u
    /// (`rest_ratios`, `lm_mirror:150–156`): the plant a unit made holds at rest. 0 at θ = 1.
    pub fn rest_kappa(&self) -> f64 {
        if self.theta >= 1.0 || self.u_design <= 0.0 {
            return 0.0;
        }
        let kr = (1.0 - self.theta) / ((self.theta * self.u_design) * self.size);
        num::pow(kr, self.theta)
    }

    /// (c_full, ζ) at the bundle cost `c` with the tape's params, as the rule forms them
    /// (`ratios`, `lm_mirror:134–147`): (c, 1) at θ = 1 or u = 0.
    pub fn full_and_zeta(&self, c: f64) -> (f64, f64) {
        if !(self.theta < 1.0 && self.u > 0.0) {
            return (c, 1.0);
        }
        let th = self.theta;
        let pk = self.size * c;
        let kr = ((1.0 - th) * c) / ((th * self.u) * pk);
        (c + (self.u * pk) * kr, num::pow(kr, -(1.0 - th)))
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
    /// Each desk's bundle cost at the point, as its rule sums it, in desk order (the good
    /// desk's is its unit cost).
    pub cost: Vec<f64>,
    /// The maker's cover of finished heads at the point (0 in the flow control).
    pub band: f64,
}

impl Genesis {
    /// A stock's genesis value by its name.
    pub fn stock_of(&self, inst: &Instance, name: &str) -> Result<f64, String> {
        Ok(self.stock[position(&stocks(inst), name)?])
    }
}

/// Genesis for a setup (LOOPS-RULES §7.4): the oracle's point, each desk's stock at rest, each
/// plant at κ_p·X_d, and each actor's stationary coin, summed in the order its rule sums.
pub fn genesis(s: &Setup) -> Result<Genesis, String> {
    let inst = &s.instance;
    let e = inst.point(s.tpy)?;
    let c = clock(s.tpy)?;
    let share = |v: f64| c.share(RatePerYear(v));
    let d = &s.dials;
    let buffer = |desk: &str| d.get(&format!("buffer.desk.{desk}.cash"));
    let (kappa, _) = inst.per_tick(s.tpy)?;
    let pl = s.plant_params()?;
    let kap = pl.rest_kappa();
    let r_ = inst.rule;
    let n = c.flow(FlowPerYear(inst.county.workers));
    let t = c.flow(FlowPerYear(inst.county.land));
    let (w, r) = (e.v, 1.0);
    let (b_f, b_h) = (r_.fodder_land * inst.b, r_.horse_land * inst.b);
    // P_s as the households sum it: (0.0 + 1·p) + h·r.
    let mut ps = 0.0;
    ps += 1.0 * e.p;
    ps += inst.county.space * r;
    let tau = n * ps;
    let hours = n * (num::ln1p(w / ps) / inst.county.chi_max).min(1.0);
    // The fodder desk's bundle cost, as the type desk sums it: ((0.0 + a·p_h) + λ·w) + b·r.
    let mut cf = 0.0;
    cf += inst.fodder_traction() * e.ph;
    cf += r_.fodder_labour * w;
    cf += b_f * r;
    let planted = |desk: &str| inst.planted().iter().any(|x| x == desk);
    // A desk's stationary coin: (c_full·ζ)·X/share planted, c·X/share not.
    let coin_of = |desk: &str, cost: f64, x: f64| -> Result<f64, String> {
        let turnover = share(buffer(desk)?);
        Ok(if planted(desk) {
            let (full, zeta) = pl.full_and_zeta(cost);
            ((full * zeta) * x) / turnover
        } else {
            (cost * x) / turnover
        })
    };
    let x = e.hours;
    let heads = x / kappa;
    let mut stationary = vec![e.p * e.good / share(buffer("good")?)];
    let mut rest = vec![e.good];
    let mut cost = vec![f64::NAN];
    let mut plants = Vec::new();
    let mut band = 0.0;
    match inst.config {
        Config::Wet => {
            // The capacity desk: p_h·κ·H/share, holding H = X/κ heads and κ·H horse-days.
            stationary.push(e.ph * kappa * heads / share(buffer("capacity")?));
            rest.push(heads);
            rest.push(kappa * heads);
            let mut o = 0.0;
            o += r_.run_fodder * e.pf;
            o += r_.run_labour * w;
            cost.push(o);
            if inst.plants.capacity {
                plants.push(kap * (kappa * heads));
            }
            // The maker: its cost of a head as its rule sums it, ((0.0 + 8·p_f) + 20·w) +
            // 3b·r (a = 0), and q_b plus its cover of b_K ticks of sales at cost over price.
            let a = 0.0;
            let mut cm = 0.0;
            cm += (a * r_.run_fodder + r_.horse_fodder) * e.pf;
            cm += (a * r_.run_labour + r_.horse_labour) * w;
            cm += b_h * r;
            band = s.cover_ticks()? * e.made * cm / e.pk;
            stationary.push(coin_of("maker", cm, e.made)?);
            rest.push(e.made + band);
            cost.push(cm);
            if inst.plants.maker {
                plants.push(kap * e.made);
            }
        }
        Config::Flow => {
            // The horse-day desk: its bundle cost ((0.0 + f·p_f) + l·w) + b·r; it holds X.
            let (f_hd, l_hd, b_hd) = inst.flow_recipe(s.tpy)?;
            let mut chd = 0.0;
            chd += f_hd * e.pf;
            chd += l_hd * w;
            chd += b_hd * r;
            stationary.push(coin_of(HOURS, chd, x)?);
            rest.push(x);
            cost.push(chd);
            if inst.plants.capacity {
                plants.push(kap * x);
            }
        }
    }
    stationary.push(coin_of("fodder", cf, e.qf)?);
    rest.push(e.qf);
    cost.push(cf);
    if inst.plants.fodder {
        plants.push(kap * e.qf);
    }
    rest.extend(plants);
    stationary.push(tau + (r * t - tau) / share(d.get("spend.provider")?));
    stationary.push((tau + w * hours) / share(d.get("spend.workers")?));
    let mut prices = vec![w, r, e.pf];
    if !inst.is_flow() {
        prices.push(e.pk);
    }
    prices.push(e.ph);
    prices.push(e.p);
    let xd = &s.displace;
    if xd.price.len() != prices.len()
        || xd.coin.len() != stationary.len()
        || xd.stock.len() != rest.len()
    {
        return Err("the displacement does not fit the instance".into());
    }
    for (p, f) in prices.iter_mut().zip(&xd.price) {
        *p *= f;
    }
    let one_minus_x = e.one_minus_x * xd.share;
    if !(0.0..=1.0).contains(&one_minus_x) {
        return Err(format!(
            "the genesis human share {one_minus_x} is outside [0, 1]"
        ));
    }
    let coin = stationary
        .iter()
        .zip(&xd.coin)
        .map(|(c, f)| c * f)
        .collect();
    let stock = rest.iter().zip(&xd.stock).map(|(q, f)| q * f).collect();
    Ok(Genesis {
        point: e,
        prices,
        share: one_minus_x,
        stationary,
        coin,
        rest,
        stock,
        cost,
        band,
    })
}

/// The params an instance registers at `tpy` (LOOPS-RULES §7.2; `instances.json`'s
/// `tape_params`): (key, value, unit, basis), in the tape's order.
pub fn params(
    inst: &Instance,
    tpy: u32,
) -> Result<Vec<(String, f64, &'static str, String)>, String> {
    let n = inst.county;
    let county = format!("Assumed(\"{COUNTY}\")");
    let rule = format!("Assumed(\"{RULE_B}\")");
    let r = inst.rule;
    let mut out: Vec<(String, f64, &'static str, String)> = Vec::new();
    let mut push = |k: &str, v: f64, unit: &'static str, basis: &str| {
        out.push((k.to_string(), v, unit, basis.to_string()));
    };
    push("inst.workers", n.workers, "FlowPerYear", &county);
    push("inst.land", n.land, "FlowPerYear", &county);
    push("inst.chi_max", n.chi_max, "Dimensionless", &county);
    push("inst.eta", n.eta, "Dimensionless", &county);
    push("inst.g0", n.g0, "Dimensionless", &county);
    push("inst.g1", n.g1, "Dimensionless", &county);
    push("inst.k", n.k, "Dimensionless", &county);
    push("inst.good.weight", 1.0, "Dimensionless", &county);
    push("inst.space.weight", n.space, "Dimensionless", &county);
    push("inst.fodder.own", 0.0, "Dimensionless", &rule);
    push(
        "inst.fodder.traction",
        inst.fodder_traction(),
        "Dimensionless",
        &rule,
    );
    push(
        "inst.fodder.labour",
        r.fodder_labour,
        "Dimensionless",
        &rule,
    );
    push(
        "inst.fodder.land",
        r.fodder_land * inst.b,
        "Dimensionless",
        &rule,
    );
    match inst.config {
        Config::Wet => {
            push(
                "inst.horse.run.fodder",
                r.run_fodder,
                "Dimensionless",
                &rule,
            );
            push(
                "inst.horse.run.labour",
                r.run_labour,
                "Dimensionless",
                &rule,
            );
            push("inst.horse.kappa", r.kappa, "FlowPerYear", &rule);
            push("inst.horse.delta", inst.delta, "FractionPerYear", &rule);
            push("inst.horse.own_hours", 0.0, "Dimensionless", &rule);
            push("inst.horse.fodder", r.horse_fodder, "Dimensionless", &rule);
            push("inst.horse.labour", r.horse_labour, "Dimensionless", &rule);
            push(
                "inst.horse.land",
                r.horse_land * inst.b,
                "Dimensionless",
                &rule,
            );
        }
        Config::Flow => {
            let flow = format!("Assumed(\"{FLOW}\")");
            let (f_hd, l_hd, b_hd) = inst.flow_recipe(tpy)?;
            push("inst.traction.own", 0.0, "Dimensionless", &flow);
            push("inst.traction.fodder", f_hd, "Dimensionless", &flow);
            push("inst.traction.labour", l_hd, "Dimensionless", &flow);
            push("inst.traction.land", b_hd, "Dimensionless", &flow);
        }
    }
    Ok(out)
}

fn f(x: f64) -> String {
    format!("{x:?}")
}

fn list(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| f(*x)).collect();
    format!("[{}]", parts.join(", "))
}

const ROLE: &str =
    "Assumed(\"HORSES-SPEC §2: the probe's rules with the horse held as a stock (docs/probe/HORSES-RULES.md)\")";
const ROLE_P21: &str =
    "Assumed(\"MARKETS-SPEC §2: the probe's rules carried to many markets (docs/probe/MARKETS-RULES.md)\")";
const ROLE_P20: &str =
    "Assumed(\"design-analytic-first with the judges' grafts (docs/probe/RULES.md)\")";
const ROLE_PLANT: &str =
    "Assumed(\"LOOP-SPEC §2.2: CAPACITY's plant on a loop desk (docs/probe/LOOPS-RULES.md)\")";

/// The tape of a setup, as RON text with its derivation in the header.
pub fn tape_ron(s: &Setup) -> Result<String, String> {
    let g = genesis(s)?;
    let e = &g.point;
    let c: Clock = clock(s.tpy)?;
    let base = &s.instance;
    let inst = s.tape_instance();
    let (kappa, delta) = inst.per_tick(s.tpy)?;
    let pl = s.plant_params()?;
    let markets = inst.markets();
    let planted = inst.planted();
    let mut o: Vec<String> = Vec::new();
    let name = format!("loops-{}", inst.id);
    let flow = inst.is_flow();
    o.push(format!(
        "// The loop step's {} world (LOOP-SPEC §4; docs/probe/LOOPS-RULES.md): {},",
        inst.id.to_uppercase(),
        inst.title
    ));
    o.push("// for P2.2b (2026-09-30).".into());
    o.push("//".into());
    o.push(
        "// Generated: do not edit. `cargo run -p rustyecon-probe --bin horses-tape -- --inst"
            .into(),
    );
    o.push(format!(
        "// {} <path>` writes it, and the test `loops_tapes_are_their_generators_output` checks",
        inst.id
    ));
    o.push("// this file against the generator.".into());
    o.push("//".into());
    o.push(format!(
        "// Genesis is the oracle's equilibrium (LOOPS-RULES §7.4, §7.5): crates/oracle unit 1g's ChainEconomy on \
         FUNDED's chain8 under rule B{} (the horse's hours at J = 2), at N = {} and T = {} a tick, {} ticks a year,",
        if base.looped { "" } else { " with the loop cut" },
        f(c.flow(FlowPerYear(base.county.workers))),
        f(c.flow(FlowPerYear(base.county.land))),
        s.tpy
    ));
    o.push(format!(
        "// δ = {} and κ = {} a tick, land factor {}: x* = {}, v = w/r = {}, P_s/r = {}, Y = {}, N_a = {},",
        f(delta),
        f(kappa),
        f(base.b),
        f(e.x_star),
        f(e.v),
        f(e.p_s),
        f(e.y),
        f(e.n_a)
    ));
    o.push(format!(
        "//   the good {}, fodder {}, a head {}, a horse-day {} (running cost {}); heads made {} a tick,",
        f(e.p),
        f(e.pf),
        f(e.pk),
        f(e.ph),
        f(e.o),
        f(e.made)
    ));
    o.push(format!(
        "//   every horse-day {} ({} the tasks', {} the fodder desk's), fodder {}.",
        f(e.hours),
        f(e.task_hours),
        f(e.fodder_hours),
        f(e.qf)
    ));
    if flow {
        o.push(
            "// The flow control (LOOPS-RULES §2.2): no horse; a horse-day desk makes horse-days from its running"
                .into(),
        );
        o.push(
            "// recipe with δ/κ of a head's folded in, and the point is rule B's (FUNDED §4.2)."
                .into(),
        );
    }
    o.push(
        "// The genesis prices are those ratios with r = 1 coin; the good desk's human share is 1 - x*; each desk"
            .into(),
    );
    o.push(format!(
        "// holds its stock at rest{}; each plant is κ_p·X of its desk, κ_p = {} from the price-free rest ratio",
        if flow {
            ""
        } else {
            ", the maker q_b and its cover of 4 weeks"
        },
        f(pl.rest_kappa())
    ));
    o.push(format!(
        "// (θ {}, s {}). Each coin is the actor's stationary balance, giving {} in actor order",
        f(pl.theta),
        f(pl.size),
        list(&g.stationary)
    ));
    o.push(format!("// ({}).", inst.actors().join(", ")));
    o.push(
        "// No agent reads the oracle at run time (R13): it seeds genesis here and scores runs in the harness"
            .into(),
    );
    o.push("// (crates/probe), outside the Sim.".into());
    let one_sided = match s.one_sided {
        OneSided::Saturate => "Saturate",
        OneSided::Hold => "Hold",
    };
    o.push("Tape(".into());
    o.push("    schema: 1,".into());
    o.push("    header: (".into());
    o.push(format!("        name: \"{name}\","));
    o.push(format!("        start: \"{START}\","));
    o.push(format!("        ticks_per_year: {},", s.tpy));
    o.push(format!(
        "        market: (rule: Imbalance, one_sided: {one_sided}, ema_time_constant: \
         \"price.ema_tc\"),"
    ));
    o.push(
        "        ledger: (rel_flow: \"ledger.rel_flow\", rel_stock: \"ledger.rel_stock\"),".into(),
    );
    o.push("    ),".into());
    o.push("    params: [".into());
    let param = |o: &mut Vec<String>, key: &str, value: f64, unit: &str, basis: &str| {
        o.push(format!(
            "        (key: \"{key}\", value: {}, unit: {unit}, basis: {basis}),",
            f(value)
        ));
    };
    o.push("        // The ledger's tolerances, as in the gate world (A12).".into());
    param(
        &mut o,
        "ledger.rel_flow",
        1e-12,
        "Dimensionless",
        "Assumed(\"July REL_FLOW, as the gate world\")",
    );
    param(
        &mut o,
        "ledger.rel_stock",
        1e-11,
        "Dimensionless",
        "Assumed(\"July REL_STOCK, as the gate world\")",
    );
    o.push("        // The instance (LOOPS-RULES §7.2; FUNDED §4.1): chain8 and rule B.".into());
    for (key, value, unit, basis) in params(&inst, s.tpy)? {
        param(&mut o, &key, value, unit, &basis);
    }
    o.push("        // Structure: fodder, horse-days and the good last one tick.".into());
    param(
        &mut o,
        "life.one_tick",
        1.0 / f64::from(s.tpy),
        "Years",
        "Assumed(\"one tick: J = 1 (PROBE-SPEC §1.2)\")",
    );
    o.push(format!(
        "        // The dials ({}; LOOPS-RULES §7.3) and the stock rules' and plants' (§7.2).",
        s.dials.set
    ));
    for d in &s.dials.values {
        param(&mut o, &d.key, d.value, d.unit, &d.basis);
    }
    if let Some(psi) = inst.reserve {
        param(&mut o, "reserve.maker", psi, "Dimensionless", RESERVE);
    }
    if !planted.is_empty() {
        let basis = format!("Assumed(\"{PLANTS}\")");
        o.push(format!(
            "        // The plants (LOOPS-RULES §3.2): θ, δ_p a year, and s1 at (θ, u = {}).",
            f(pl.u_design)
        ));
        for d in &planted {
            param(
                &mut o,
                &format!("plant.{d}.theta"),
                pl.theta,
                "Dimensionless",
                &basis,
            );
            param(
                &mut o,
                &format!("plant.{d}.delta"),
                pl.delta,
                "FractionPerYear",
                &basis,
            );
            param(
                &mut o,
                &format!("plant.{d}.size"),
                pl.size,
                "Dimensionless",
                &basis,
            );
        }
    }
    o.push("    ],".into());
    o.push("    goods: [".into());
    o.push("        (key: \"coin\", life: Indefinite, price_rate: None),".into());
    o.push("        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),".into());
    o.push("        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),".into());
    for m in &markets[2..] {
        let life = if m == "horse" {
            "Indefinite"
        } else {
            "Years(\"life.one_tick\")"
        };
        o.push(format!(
            "        (key: \"{m}\", life: {life}, price_rate: Some(\"rate.{m}\")),"
        ));
    }
    for d in &planted {
        o.push(format!(
            "        (key: \"plant.{d}\", life: Indefinite, price_rate: None, untraded: true),"
        ));
    }
    o.push("    ],".into());
    o.push("    nodes: [(key: \"home\", currency: \"coin\")],".into());
    o.push("    channels: [],".into());
    let mut classes: Vec<String> = inst
        .desks()
        .iter()
        .map(|d| format!("\"{d}_desks\""))
        .collect();
    classes.push("\"owners\"".into());
    classes.push("\"workers\"".into());
    o.push(format!("    classes: [{}],", classes.join(", ")));
    o.push("    actors: [".into());
    let scale = |desk: &str| {
        format!(
            "            scale: Cash((turnover: \"buffer.desk.{desk}.cash\", tilt: \
             \"tilt.desk.{desk}\", payout: None)),"
        )
    };
    let plant = |desk: &str, target: &str| {
        format!(
            "            plant: Some((good: \"plant.{desk}\", theta: \"plant.{desk}.theta\", delta: \
             \"plant.{desk}.delta\", size: \"plant.{desk}.size\", adjust: \"adjust.plant.{desk}\", \
             order: {}, target: {target})),",
            base.design.order
        )
    };
    let is_planted = |desk: &str| planted.iter().any(|x| x == desk);
    // The good desk.
    o.push(format!(
        "        (key: \"desk.good\", kind: Desk, class: \"good_desks\", home: \"home\", basis: {ROLE_P20},"
    ));
    o.push("         spec: GoodDesk((".into());
    o.push(format!(
        "            output: \"good\", labour: \"labour\", mach: \"{HOURS}\","
    ));
    o.push(
        "            schedule: (eta: \"inst.eta\", g0: \"inst.g0\", g1: \"inst.g1\", k: \"inst.k\"),"
            .into(),
    );
    o.push(format!(
        "            technique: (adjust: \"adjust.technique.good\", share: {}),",
        f(g.share)
    ));
    o.push("            assign: ExPost,".into());
    o.push(scale("good"));
    o.push("         ))),".into());
    let running = "(goods: [(good: \"fodder\", coef: \"inst.horse.run.fodder\")], labour: \
                   \"inst.horse.run.labour\")";
    let basis_of = |desk: &str, role: &'static str| {
        if is_planted(desk) {
            ROLE_PLANT
        } else {
            role
        }
    };
    if flow {
        // The flow control's horse-day desk.
        o.push(format!(
            "        (key: \"desk.{HOURS}\", kind: Desk, class: \"{HOURS}_desks\", home: \"home\", basis: {},",
            basis_of(HOURS, ROLE_P21)
        ));
        o.push("         spec: TypeDesk((".into());
        o.push(format!(
            "            output: \"{HOURS}\", labour: \"labour\", land: \"land\","
        ));
        o.push(
            "            recipe: (own: \"inst.traction.own\", inputs: [(good: \"fodder\", coef: \
             \"inst.traction.fodder\")], labour: \"inst.traction.labour\", land: \
             \"inst.traction.land\"),"
                .into(),
        );
        o.push(scale(HOURS));
        if is_planted(HOURS) {
            o.push(plant(HOURS, "Bundles"));
        }
        o.push("         ))),".into());
    } else {
        // The capacity desk.
        o.push(format!(
            "        (key: \"desk.capacity\", kind: Desk, class: \"capacity_desks\", home: \"home\", basis: {},",
            basis_of("capacity", ROLE)
        ));
        o.push("         spec: CapacityDesk((".into());
        o.push(format!(
            "            stock: \"horse\", hours: \"{HOURS}\", labour: \"labour\", kappa: \"inst.horse.kappa\","
        ));
        o.push(format!("            running: {running},"));
        o.push(
            "            delta: \"inst.horse.delta\", adjust: \"adjust.invest.capacity\", order: Target,"
                .into(),
        );
        o.push(scale("capacity"));
        if is_planted("capacity") {
            o.push(plant("capacity", base.design.target));
        }
        o.push("         ))),".into());
        // The maker.
        o.push(format!(
            "        (key: \"desk.maker\", kind: Desk, class: \"maker_desks\", home: \"home\", basis: {},",
            basis_of("maker", ROLE)
        ));
        o.push("         spec: Maker((".into());
        o.push(
            "            output: \"horse\", labour: \"labour\", land: \"land\", own_hours: \"inst.horse.own_hours\","
                .into(),
        );
        o.push(format!(
            "            kappa: \"inst.horse.kappa\", running: {running},"
        ));
        o.push(
            "            build: (goods: [(good: \"fodder\", coef: \"inst.horse.fodder\")], labour: \
             \"inst.horse.labour\", land: \"inst.horse.land\"),"
                .into(),
        );
        let reserve = if inst.reserve.is_some() {
            " reserve: Some(\"reserve.maker\"),"
        } else {
            ""
        };
        o.push(format!(
            "            delta: \"inst.horse.delta\", adjust: \"adjust.invest.maker\", cover: \
             Some(\"cover.maker\"),{reserve} own: 0.0,"
        ));
        o.push(scale("maker"));
        if is_planted("maker") {
            o.push(plant("maker", "Bundles"));
        }
        o.push("         ))),".into());
    }
    // The fodder desk.
    o.push(format!(
        "        (key: \"desk.fodder\", kind: Desk, class: \"fodder_desks\", home: \"home\", basis: {},",
        basis_of("fodder", ROLE_P21)
    ));
    o.push("         spec: TypeDesk((".into());
    o.push("            output: \"fodder\", labour: \"labour\", land: \"land\",".into());
    o.push(format!(
        "            recipe: (own: \"inst.fodder.own\", inputs: [(good: \"{HOURS}\", coef: \
         \"inst.fodder.traction\")], labour: \"inst.fodder.labour\", land: \"inst.fodder.land\"),"
    ));
    o.push(scale("fodder"));
    if is_planted("fodder") {
        o.push(plant("fodder", "Bundles"));
    }
    o.push("         ))),".into());
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
            "        basis: Approximate(\"oracle unit 1g (crates/oracle) at LOOPS-RULES' {}, N = {} and \
             T = {} per tick; stationary coins, stocks and plants per LOOPS-RULES §7.4; written by \
             rustyecon-probe's horses-tape\"),",
            inst.id.to_uppercase(),
            f(c.flow(FlowPerYear(inst.county.workers))),
            f(c.flow(FlowPerYear(inst.county.land)))
        ),
        "        prices: [".into(),
    ] {
        o.push(line);
    }
    for (m, p) in markets.iter().zip(&g.prices) {
        o.push(format!(
            "            (node: \"home\", good: \"{m}\", price: {}),",
            f(*p)
        ));
    }
    o.push("        ],".into());
    o.push("        holdings: [".into());
    let stock = |name: &str| g.stock_of(&inst, name);
    for (i, a) in inst.actors().iter().enumerate() {
        let mut goods = vec![format!("(\"coin\", {})", f(g.coin[i]))];
        let desk = a.strip_prefix("desk.").unwrap_or("");
        match a.as_str() {
            "desk.good" => goods.push(format!("(\"good\", {})", f(stock("stock.good")?))),
            "desk.capacity" => {
                goods.push(format!("(\"horse\", {})", f(stock("heads.capacity")?)));
                goods.push(format!("(\"{HOURS}\", {})", f(stock("hours.capacity")?)));
            }
            "desk.maker" => goods.push(format!("(\"horse\", {})", f(stock("finished.maker")?))),
            "desk.fodder" => goods.push(format!("(\"fodder\", {})", f(stock("stock.fodder")?))),
            "provider" | "workers" => {}
            _ => goods.push(format!(
                "(\"{HOURS}\", {})",
                f(stock(&format!("stock.{HOURS}"))?)
            )),
        }
        if is_planted(desk) {
            goods.push(format!(
                "(\"plant.{desk}\", {})",
                f(stock(&format!("plant.{desk}"))?)
            ));
        }
        o.push(format!(
            "            (holder: \"{a}\", goods: [{}]),",
            goods.join(", ")
        ));
    }
    o.push("        ],".into());
    o.push("    ),".into());
    o.push("    events: [],".into());
    o.push("    recurring: [],".into());
    o.push(")".into());
    let mut text = o.join("\n");
    text.push('\n');
    Ok(text)
}
