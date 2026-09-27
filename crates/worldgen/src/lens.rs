//! The demo world's lenses (docs/demo/WORLD.md §6; docs/GUI.md §4, "Lenses"), 2026-09-27, D.3.
//!
//! `worlds/demo-gb/lenses.csv` lists the map's lenses: each is a measure of one county at one
//! report tick, with its unit, scale, reference and fixed domain. This module holds the one
//! definition of every measure, so the GUI's map and table compute nothing themselves (U6):
//! they gather a county's recorded numbers into [`Readings`] and call [`value`]. The measures
//! are ratios, shares, counts and logs of what the engine reports: posted prices, cleared
//! volumes, whether a market traded (`MarketLine::trades`), rationing lines, params and the
//! roles' own states. None is fed back to a run (R13).
//!
//! The lenses belong in `crates/observe`'s `measure` when that crate exists (docs/GUI.md §7.3,
//! Phase 2). Until then they live here, beside the table that lists them and the compiler that
//! writes the keys they read ([`CountyKeys`]), in a crate with no engine, egui, file, thread or
//! clock. The two oracle lenses, `gap.oracle` and `gap.wage`, need that crate's `oracle_gap`,
//! and [`value`] says so rather than compute them.
//!
//! Transcendentals go through `core::num` (U9): the change lenses take a log.

use crate::compile::ROLES;
use crate::tables::{self, Lens, Param, Scale};
use crate::CompileError;
use rustyecon_core::num;

/// The bundled table, `worlds/demo-gb/lenses.csv`.
pub const DEMO_GB_LENSES: &str = include_str!("../../../worlds/demo-gb/lenses.csv");

/// The demo world's lenses, in table order, read and checked from the bundled table.
pub fn demo_gb() -> Result<Vec<Lens>, CompileError> {
    tables::parse_lenses(DEMO_GB_LENSES)
}

/// The trailing window the `no.trade` lens counts over: a year of the demo's 52 ticks.
pub const NO_TRADE_WINDOW: u64 = 52;

/// What a level lens measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// w / (p + h·r): the wage in baskets.
    WageBaskets,
    /// w / p.
    WageGoods,
    /// w / r.
    WageLand,
    /// r / p.
    RentGoods,
    /// r·T_c / (w·L_c + r·T_c).
    ShareLand,
    /// w·L_c / (w·L_c + r·T_c).
    ShareLabour,
    /// 1 − the good desk's planned human share.
    Frontier,
    /// L_c / N, N per tick.
    Participation,
    /// Y_c / N, N per tick.
    OutputPerHead,
    /// p / r.
    PriceGood,
    /// p_m / r.
    PriceMach,
    /// The provider's due / (r·T_c).
    ReliefBurden,
    /// (due − paid) / due.
    Shortfall,
    /// The largest 1 − filled/requested over the county's rationing lines.
    Rationing,
    /// Ticks of the trailing [`NO_TRADE_WINDOW`] in which one of the county's four markets did
    /// not trade (`MarketLine::trades`). Not the probe's dead tick, which also counts a market
    /// that clears below half its oracle volume and waits for `crates/observe` (O17); so it is
    /// not called `dead`.
    NoTrade,
    /// D̂ against the county's own oracle point: needs `crates/observe`.
    GapOracle,
    /// ln((w/r)/v*): needs `crates/observe`.
    GapWage,
    /// A county param as the run holds it, in its registered unit.
    Param(Param),
}

/// What a lens measures: a level, or a level's change since the record's first tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Measure {
    /// The level itself.
    Level(Level),
    /// ln(level ÷ the county's level at the record's first report tick).
    Since(Level),
}

impl Level {
    /// The level a lens key names: `wage.land`, `param.eta`, …
    pub fn of(key: &str) -> Option<Level> {
        if let Some(col) = key.strip_prefix("param.") {
            return Param::ALL
                .into_iter()
                .find(|p| p.column() == col)
                .map(Level::Param);
        }
        Some(match key {
            "wage.baskets" => Level::WageBaskets,
            "wage.goods" => Level::WageGoods,
            "wage.land" => Level::WageLand,
            "rent.goods" => Level::RentGoods,
            "share.land" => Level::ShareLand,
            "share.labour" => Level::ShareLabour,
            "frontier.x" => Level::Frontier,
            "participation" => Level::Participation,
            "output.per.head" => Level::OutputPerHead,
            "price.good" => Level::PriceGood,
            "price.mach" => Level::PriceMach,
            "relief.burden" => Level::ReliefBurden,
            "shortfall" => Level::Shortfall,
            "rationing" => Level::Rationing,
            "no.trade" => Level::NoTrade,
            "gap.oracle" => Level::GapOracle,
            "gap.wage" => Level::GapWage,
            _ => return None,
        })
    }

    /// Whether it needs the oracle, which waits for `crates/observe`.
    pub fn needs_observe(self) -> bool {
        matches!(self, Level::GapOracle | Level::GapWage)
    }
}

impl Measure {
    /// The measure a lens key names: a level's key, or `since.<level key>`.
    pub fn of(key: &str) -> Option<Measure> {
        match key.strip_prefix("since.") {
            Some(base) => Level::of(base).map(Measure::Since),
            None => Level::of(key).map(Measure::Level),
        }
    }

    /// The level it reads.
    pub fn level(self) -> Level {
        match self {
            Measure::Level(l) | Measure::Since(l) => l,
        }
    }
}

/// A lens's reference as a number: the value its `reference` column names, written
/// `<value> = <what>` or `<what> = <value>`. `None` when it has none.
pub fn reference_value(reference: &str) -> Option<f64> {
    let (a, b) = reference.split_once(" = ")?;
    let number = |s: &str| s.trim().parse::<f64>().ok().filter(|v| v.is_finite());
    number(a).or_else(|| number(b))
}

/// The centre of a lens's scale: its reference for a diverging scale; none for a sequential
/// one, which runs from its domain's low end to its high end.
pub fn centre(lens: &Lens) -> Option<f64> {
    match lens.scale {
        Scale::Diverging => reference_value(&lens.reference),
        Scale::Sequential | Scale::SequentialLog => None,
    }
}

/// The tape keys a county's readings come from, as the compiler writes them (`compile`): the
/// county's node, its four markets' goods, its params `<node>.<column>`, and its four roles'
/// actors `<node>.<role>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountyKeys {
    /// The node: `county.<chapman>`.
    pub node: String,
    /// The goods of its four markets: labour, land, machine services, the good.
    pub labour: &'static str,
    /// Land services.
    pub land: &'static str,
    /// Machine services.
    pub mach: &'static str,
    /// The good.
    pub good: &'static str,
    /// Each param, in [`Param::ALL`] order.
    pub params: [String; 11],
    /// The good desk.
    pub desk_good: String,
    /// The machine desk.
    pub desk_mach: String,
    /// The provider.
    pub provider: String,
    /// The workers.
    pub workers: String,
}

/// The goods of a county's four markets, in the order labour, land, machine services, the good.
pub const MARKETS: [&str; 4] = ["labour", "land", "mach", "good"];

impl CountyKeys {
    /// The keys of the county whose node is `node`.
    pub fn of(node: &str) -> CountyKeys {
        let role = |r: &str| format!("{node}.{r}");
        CountyKeys {
            node: node.to_string(),
            labour: MARKETS[0],
            land: MARKETS[1],
            mach: MARKETS[2],
            good: MARKETS[3],
            params: Param::ALL.map(|p| format!("{node}.{}", p.column())),
            desk_good: role(ROLES[0]),
            desk_mach: role(ROLES[1]),
            provider: role(ROLES[2]),
            workers: role(ROLES[3]),
        }
    }

    /// A param's key.
    pub fn param(&self, p: Param) -> &str {
        &self.params[p.index()]
    }
}

/// The params a county's lenses read: N (participation, output per head, and its lens), h
/// (the wage in baskets), and the history's levers T, η, λ, b and χ_max, each with its lens.
pub const PARAMS_READ: [Param; 7] = [
    Param::Workers,
    Param::Space,
    Param::Land,
    Param::Eta,
    Param::Lam,
    Param::B,
    Param::ChiMax,
];

/// One county's recorded numbers at one report tick, as the run reported them. A number the
/// record lacks is `None`, and a lens that needs it has no value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Readings {
    /// The posted prices of labour (w), land (r), machine services (p_m) and the good (p), in
    /// the node's currency, as the tick's settlement used them (`MarketLine::price`).
    pub prices: [Option<f64>; 4],
    /// The cleared volumes of the four markets, per tick (`MarketLine::cleared`).
    pub cleared: [Option<f64>; 4],
    /// Each param after the tick, in [`Param::ALL`] order and its registered unit (N and T a
    /// year).
    pub params: [Option<f64>; 11],
    /// N per tick: the workers param through the clock's conversion for its use (the engine's
    /// `ClockMethod::per_tick`), which the caller applies.
    pub n_tick: Option<f64>,
    /// The good desk's planned human share (`GoodDeskState::share`), after the tick.
    pub share: Option<f64>,
    /// The provider's last transfer: what it owed and what it paid (`ProviderState`).
    pub due: Option<f64>,
    /// See `due`.
    pub paid: Option<f64>,
    /// The tick's rationing lines at the county's node: (requested, filled), each class's.
    pub rationing: Vec<(f64, f64)>,
    /// For each report tick of the trailing window that the record holds, oldest first,
    /// whether all four of the county's markets traded (`MarketLine::trades`).
    pub traded: Vec<bool>,
}

/// Why a lens has no value for a county.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoValue {
    /// The lens needs what is not built: the oracle lenses wait for `crates/observe`.
    Unavailable(String),
    /// A number it reads is not in the record.
    Missing(&'static str),
    /// Its formula is undefined here, such as a ratio over zero.
    Undefined(&'static str),
}

impl std::fmt::Display for NoValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoValue::Unavailable(why) => write!(f, "unavailable: {why}"),
            NoValue::Missing(what) => write!(f, "not recorded: {what}"),
            NoValue::Undefined(why) => write!(f, "undefined: {why}"),
        }
    }
}

fn need(v: Option<f64>, what: &'static str) -> Result<f64, NoValue> {
    v.ok_or(NoValue::Missing(what))
}

fn ratio(a: f64, b: f64, what: &'static str) -> Result<f64, NoValue> {
    if b > 0.0 {
        Ok(a / b)
    } else {
        Err(NoValue::Undefined(what))
    }
}

/// A level's value from one county's readings.
pub fn level(l: Level, x: &Readings) -> Result<f64, NoValue> {
    let price = |i: usize, what| need(x.prices[i], what);
    let cleared = |i: usize, what| need(x.cleared[i], what);
    let w = || price(0, "the price of labour");
    let r = || price(1, "the price of land");
    let pm = || price(2, "the price of machine services");
    let p = || price(3, "the price of the good");
    let factor = || -> Result<(f64, f64), NoValue> {
        let wl = w()? * cleared(0, "labour cleared")?;
        let rt = r()? * cleared(1, "land cleared")?;
        Ok((wl, rt))
    };
    match l {
        Level::WageBaskets => {
            let h = need(x.params[Param::Space.index()], "the space in a basket (h)")?;
            ratio(w()?, p()? + h * r()?, "the basket's price is 0")
        }
        Level::WageGoods => ratio(w()?, p()?, "the good's price is 0"),
        Level::WageLand => ratio(w()?, r()?, "the price of land is 0"),
        Level::RentGoods => ratio(r()?, p()?, "the good's price is 0"),
        Level::ShareLand => {
            let (wl, rt) = factor()?;
            ratio(rt, wl + rt, "no factor income")
        }
        Level::ShareLabour => {
            let (wl, rt) = factor()?;
            ratio(wl, wl + rt, "no factor income")
        }
        Level::Frontier => Ok(1.0 - need(x.share, "the good desk's technique")?),
        Level::Participation => ratio(
            cleared(0, "labour cleared")?,
            need(x.n_tick, "N")?,
            "N is 0",
        ),
        Level::OutputPerHead => ratio(
            cleared(3, "the good cleared")?,
            need(x.n_tick, "N")?,
            "N is 0",
        ),
        Level::PriceGood => ratio(p()?, r()?, "the price of land is 0"),
        Level::PriceMach => ratio(pm()?, r()?, "the price of land is 0"),
        Level::ReliefBurden => {
            let rent = r()? * cleared(1, "land cleared")?;
            ratio(need(x.due, "the provider's due")?, rent, "no rent was paid")
        }
        Level::Shortfall => {
            let due = need(x.due, "the provider's due")?;
            let paid = need(x.paid, "the provider's payment")?;
            ratio(due - paid, due, "nothing was due")
        }
        Level::Rationing => x
            .rationing
            .iter()
            .filter(|(req, _)| *req > 0.0)
            .map(|(req, fill)| 1.0 - fill / req)
            .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))))
            .ok_or(NoValue::Missing("a rationing line with a request")),
        Level::NoTrade => {
            if x.traded.is_empty() {
                return Err(NoValue::Missing("whether the markets traded"));
            }
            Ok(x.traded.iter().filter(|t| !**t).count() as f64)
        }
        Level::GapOracle | Level::GapWage => Err(NoValue::Unavailable(
            "the oracle lenses need crates/observe's oracle_gap (docs/GUI.md §7.3)".to_string(),
        )),
        Level::Param(q) => need(x.params[q.index()], "the param"),
    }
}

/// A lens's value from one county's readings at a report tick, and, for a change lens, the
/// county's readings at the record's first report tick.
pub fn value(m: Measure, now: &Readings, first: Option<&Readings>) -> Result<f64, NoValue> {
    match m {
        Measure::Level(l) => level(l, now),
        Measure::Since(l) => {
            let v = level(l, now)?;
            let first = first.ok_or(NoValue::Missing("the record's first tick"))?;
            let v0 = level(l, first)?;
            if v > 0.0 && v0 > 0.0 {
                Ok(num::ln(v / v0))
            } else {
                Err(NoValue::Undefined("a change in log needs positive levels"))
            }
        }
    }
}
