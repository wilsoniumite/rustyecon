//! The oracle lab's domain (docs/GUI.md §9, G1): an instance of any of the oracle's units 1a–1f,
//! solved by the oracle itself, and what the lab reads of it. No egui, no model, no file,
//! thread or clock: parameters in, the oracle's own numbers out.
//!
//! - [`Instance`] holds one unit's parameters, as the oracle's own types. [`presets`] builds
//!   the instances the goldens were computed on, and [`knobs`] lists and sets every number of
//!   an instance by its path (`schedule.eta`, `categories[1].weight`, …).
//! - [`Instance::solve`] validates and solves through the unit's own `solve`, and reports the
//!   regime and every output the unit's `outputs()` lists, under the key the oracle prints it
//!   with. Nothing is recomputed: each number is the oracle's double (U3, U6).
//! - [`Instance::at`] evaluates the unit's price and quantity block at a threshold x; [`fields`]
//!   reads every number of that point from the oracle's own `Debug`, so the lab keeps no copy of
//!   a point's fields, and adds f(x) = n_D − n_S through the point's own `excess_demand`.
//! - [`goldens`] reads the six committed goldens files, which the generators wrote at 70 digits
//!   from the equations, for the lab to show beside the oracle's outputs.
//!
//! The lab computes no measure of a run and feeds nothing into one (U5, R13): the oracle is
//! solved here, outside any `Sim`, and its numbers are shown as the oracle's, origin "oracle".

pub mod fields;
pub mod goldens;
pub mod knobs;
pub mod presets;

use oracle::{
    CategoryEconomy, CategoryParams, Economy, HouseholdEconomy, HouseholdParams, MachineEconomy,
    MachineParams, Output, Output1b, Params, ParcelEconomy, ParcelParams, Regime, SolveError,
    WorkerEconomy, WorkerParams,
};
use serde::Serialize;
use std::fmt;

/// One of the oracle's units (Phase 1): 1a one category, 1b many categories, 1c many machine
/// types, 1d worker types and the wall, 1e parcels and the priced exit, 1f households and a
/// government.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum OracleUnit {
    /// Unit 1a.
    U1a,
    /// Unit 1b.
    U1b,
    /// Unit 1c.
    U1c,
    /// Unit 1d.
    U1d,
    /// Unit 1e.
    U1e,
    /// Unit 1f.
    U1f,
}

impl OracleUnit {
    /// Every unit, in order.
    pub const ALL: [OracleUnit; 6] = [
        OracleUnit::U1a,
        OracleUnit::U1b,
        OracleUnit::U1c,
        OracleUnit::U1d,
        OracleUnit::U1e,
        OracleUnit::U1f,
    ];

    /// What the unit adds, in a few words.
    pub fn title(self) -> &'static str {
        match self {
            OracleUnit::U1a => "one category, durability and interest",
            OracleUnit::U1b => "many categories and the fork",
            OracleUnit::U1c => "many machine types and the Leontief inverse",
            OracleUnit::U1d => "worker types and the wall",
            OracleUnit::U1e => "parcels, the idle margin and the priced exit",
            OracleUnit::U1f => "households and government",
        }
    }
}

impl fmt::Display for OracleUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OracleUnit::U1a => "1a",
            OracleUnit::U1b => "1b",
            OracleUnit::U1c => "1c",
            OracleUnit::U1d => "1d",
            OracleUnit::U1e => "1e",
            OracleUnit::U1f => "1f",
        };
        f.write_str(s)
    }
}

/// An instance of one unit: its parameters, as the oracle's own types.
#[derive(Debug, Clone, PartialEq)]
pub enum Instance {
    /// Unit 1a.
    A(Params),
    /// Unit 1b.
    B(CategoryParams),
    /// Unit 1c.
    C(MachineParams),
    /// Unit 1d.
    D(WorkerParams),
    /// Unit 1e.
    E(ParcelParams),
    /// Unit 1f.
    F(HouseholdParams),
}

/// A validated economy of one unit.
#[derive(Debug, Clone)]
pub enum Validated {
    /// Unit 1a.
    A(Box<Economy>),
    /// Unit 1b.
    B(Box<CategoryEconomy>),
    /// Unit 1c.
    C(Box<MachineEconomy>),
    /// Unit 1d.
    D(Box<WorkerEconomy>),
    /// Unit 1e.
    E(Box<ParcelEconomy>),
    /// Unit 1f.
    F(Box<HouseholdEconomy>),
}

/// One reported output, as the unit's `outputs()` gives it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Value {
    /// A number.
    Float(f64),
    /// A number the unit reports only in some cases (φ when u = 1, …), `None` otherwise.
    Optional(Option<f64>),
    /// A flag.
    Flag(bool),
    /// A count.
    Count(u32),
}

impl Value {
    /// The number, where there is one: a float, a present optional, or a count.
    pub fn number(self) -> Option<f64> {
        match self {
            Value::Float(v) | Value::Optional(Some(v)) => Some(v),
            Value::Count(n) => Some(f64::from(n)),
            Value::Optional(None) | Value::Flag(_) => None,
        }
    }
}

impl fmt::Display for Value {
    /// As the oracle's dump prints it: a float by `{:?}`, the shortest digits that parse back
    /// to the same double, so a painted value is the oracle's double bit for bit.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(v) | Value::Optional(Some(v)) => write!(f, "{v:?}"),
            Value::Optional(None) => f.write_str("none"),
            Value::Flag(b) => write!(f, "{b}"),
            Value::Count(n) => write!(f, "{n}"),
        }
    }
}

impl From<Output> for Value {
    fn from(o: Output) -> Value {
        match o {
            Output::Float(v) => Value::Float(v),
            Output::FlowOnly(v) => Value::Optional(v),
            Output::Flag(b) => Value::Flag(b),
            Output::Count(n) => Value::Count(n),
        }
    }
}

impl From<Output1b> for Value {
    fn from(o: Output1b) -> Value {
        match o {
            Output1b::Float(v) => Value::Float(v),
            Output1b::Optional(v) => Value::Optional(v),
            Output1b::Flag(b) => Value::Flag(b),
            Output1b::Count(n) => Value::Count(n),
        }
    }
}

/// What a solve gave.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Solved {
    /// The regime's name as the oracle's `Regime::name` gives it, or "invalid" when the
    /// parameters do not validate, or "refused" when the solve returns an error.
    pub regime: String,
    /// What else the oracle says of it: the margin, the land market and the exit land at an
    /// equilibrium of 1d–1f; a boundary regime's diagnostic; the error.
    pub detail: Vec<(String, String)>,
    /// x*, at an equilibrium.
    pub x_star: Option<f64>,
    /// Every output of an equilibrium, by the key the oracle prints it under, in its order.
    pub outputs: Vec<(String, Value)>,
}

impl Solved {
    fn failed(regime: &str, why: String) -> Solved {
        Solved {
            regime: regime.to_string(),
            detail: vec![("why".to_string(), why)],
            x_star: None,
            outputs: Vec::new(),
        }
    }

    /// An output by its key.
    pub fn output(&self, key: &str) -> Option<Value> {
        self.outputs.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
    }
}

/// A boundary regime's name and diagnostic; an equilibrium is handled by the caller.
fn boundary<E>(r: &Regime<E>) -> Solved {
    let diagnostic = match r {
        Regime::BoundaryNoMargin { f_at_1 } => Some(("f(1)", *f_at_1)),
        Regime::NotViable { d_at_1 } => Some(("D(1)", *d_at_1)),
        Regime::NoInteriorAtZero { f_at_0 } => Some(("f(lo)", *f_at_0)),
        Regime::Interior(_) => None,
    };
    Solved {
        regime: r.name().to_string(),
        detail: diagnostic
            .map(|(k, v)| vec![(k.to_string(), format!("{v:?}"))])
            .unwrap_or_default(),
        x_star: None,
        outputs: Vec::new(),
    }
}

/// A solve's result as the lab reports it: the regime, and at an equilibrium x*, the detail
/// and the outputs.
fn report<E>(
    r: Result<Regime<E>, SolveError>,
    of: impl Fn(&E) -> (f64, Vec<(String, String)>, Vec<(String, Value)>),
) -> Solved {
    match r {
        Err(e) => Solved::failed("refused", e.to_string()),
        Ok(Regime::Interior(eq)) => {
            let (x_star, detail, outputs) = of(&eq);
            Solved {
                regime: "Interior".to_string(),
                detail,
                x_star: Some(x_star),
                outputs,
            }
        }
        Ok(other) => boundary(&other),
    }
}

fn keyed<K: fmt::Display, O: Into<Value>>(outputs: Vec<(K, O)>) -> Vec<(String, Value)> {
    outputs
        .into_iter()
        .map(|(k, o)| (k.to_string(), o.into()))
        .collect()
}

impl Instance {
    /// Its unit.
    pub fn unit(&self) -> OracleUnit {
        match self {
            Instance::A(_) => OracleUnit::U1a,
            Instance::B(_) => OracleUnit::U1b,
            Instance::C(_) => OracleUnit::U1c,
            Instance::D(_) => OracleUnit::U1d,
            Instance::E(_) => OracleUnit::U1e,
            Instance::F(_) => OracleUnit::U1f,
        }
    }

    /// The economy, validated by the unit's own `new`, or why not.
    pub fn validate(&self) -> Result<Validated, String> {
        let why = |e: oracle::ParamError| format!("invalid parameter: {e}");
        Ok(match self {
            Instance::A(p) => Validated::A(Box::new(Economy::new(p.clone()).map_err(why)?)),
            Instance::B(p) => Validated::B(Box::new(CategoryEconomy::new(p.clone()).map_err(why)?)),
            Instance::C(p) => Validated::C(Box::new(MachineEconomy::new(p.clone()).map_err(why)?)),
            Instance::D(p) => Validated::D(Box::new(WorkerEconomy::new(p.clone()).map_err(why)?)),
            Instance::E(p) => Validated::E(Box::new(ParcelEconomy::new(p.clone()).map_err(why)?)),
            Instance::F(p) => {
                Validated::F(Box::new(HouseholdEconomy::new(p.clone()).map_err(why)?))
            }
        })
    }

    /// Validate and solve.
    pub fn solve(&self) -> Solved {
        match self.validate() {
            Ok(e) => e.solve(),
            Err(why) => Solved::failed("invalid", why),
        }
    }
}

/// A point of the price and quantity block at a threshold x, of any unit.
#[derive(Debug, Clone, PartialEq)]
pub enum Point {
    /// Unit 1a.
    A(oracle::Point),
    /// Unit 1b.
    B(oracle::CategoryPoint),
    /// Unit 1c.
    C(oracle::MachinePoint),
    /// Unit 1d.
    D(oracle::WorkerPoint),
    /// Unit 1e.
    E(oracle::ParcelPoint),
    /// Unit 1f.
    F(oracle::HouseholdPoint),
}

impl Point {
    /// f(x) = n_D − n_S, the point's own `excess_demand`.
    pub fn excess_demand(&self) -> f64 {
        match self {
            Point::A(p) => p.excess_demand(),
            Point::B(p) => p.excess_demand(),
            Point::C(p) => p.excess_demand(),
            Point::D(p) => p.excess_demand(),
            Point::E(p) => p.excess_demand(),
            Point::F(p) => p.excess_demand(),
        }
    }

    /// The oracle's `Debug` of the point, from which [`fields::read`] takes its numbers.
    pub fn debug(&self) -> String {
        match self {
            Point::A(p) => format!("{p:?}"),
            Point::B(p) => format!("{p:?}"),
            Point::C(p) => format!("{p:?}"),
            Point::D(p) => format!("{p:?}"),
            Point::E(p) => format!("{p:?}"),
            Point::F(p) => format!("{p:?}"),
        }
    }

    /// Every number of the point by its path, f(x) first under the name `f`.
    pub fn fields(&self) -> Vec<(String, f64)> {
        let mut out = vec![(fields::F.to_string(), self.excess_demand())];
        out.extend(fields::read(&self.debug()));
        out
    }
}

impl Validated {
    /// Solve by the unit's own `solve`.
    pub fn solve(&self) -> Solved {
        match self {
            Validated::A(e) => report(e.solve(), |eq| (eq.x_star, Vec::new(), keyed(eq.outputs()))),
            Validated::B(e) => report(e.solve(), |eq| (eq.x_star, Vec::new(), keyed(eq.outputs()))),
            Validated::C(e) => report(e.solve(), |eq| (eq.x_star, Vec::new(), keyed(eq.outputs()))),
            Validated::D(e) => report(e.solve(), |eq| {
                let detail = vec![("margin".to_string(), format!("{:?}", eq.margin))];
                (eq.x_star, detail, keyed(eq.outputs()))
            }),
            Validated::E(e) => report(e.solve(), |eq| {
                let detail = vec![
                    ("margin".to_string(), format!("{:?}", eq.base.margin)),
                    ("land market".to_string(), format!("{:?}", eq.land_market)),
                    ("exit land".to_string(), format!("{:?}", eq.exit_land)),
                ];
                (eq.base.x_star, detail, keyed(eq.outputs()))
            }),
            Validated::F(e) => report(e.solve(), |eq| {
                let b = &eq.base;
                let detail = vec![
                    ("margin".to_string(), format!("{:?}", b.base.margin)),
                    ("land market".to_string(), format!("{:?}", b.land_market)),
                    ("exit land".to_string(), format!("{:?}", b.exit_land)),
                ];
                (b.base.x_star, detail, keyed(eq.outputs()))
            }),
        }
    }

    /// The price and quantity block at x, by the unit's own `at`.
    pub fn at(&self, x: f64) -> Point {
        match self {
            Validated::A(e) => Point::A(e.at(x)),
            Validated::B(e) => Point::B(e.at(x)),
            Validated::C(e) => Point::C(e.at(x)),
            Validated::D(e) => Point::D(e.at(x)),
            Validated::E(e) => Point::E(e.at(x)),
            Validated::F(e) => Point::F(e.at(x)),
        }
    }
}
