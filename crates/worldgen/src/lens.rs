//! The demo world's lenses (docs/demo/WORLD.md §6; docs/GUI.md §4, "Lenses"), 2026-09-27, D.3;
//! the second pass's (docs/demo/WORLD-V2.md §7), D2.3, 2026-09-30.
//!
//! `worlds/demo-gb/lenses.csv` lists v1's map lenses and `lenses-v2a1.csv` the second pass's: each
//! is a measure of one county at one report tick, with its unit, scale, reference and fixed
//! domain. This module holds the one definition of every measure, so the GUI's map and table
//! compute nothing themselves (U6): they gather a county's recorded numbers into [`Readings`] and
//! call [`value`]. The measures are ratios, shares, counts and logs of what the engine reports:
//! posted prices, cleared volumes, whether a market traded (`MarketLine::trades`), rationing
//! lines, params and the roles' own states. None is fed back to a run (R13).
//!
//! The lenses belong in `crates/observe`'s `measure` when that crate exists (docs/GUI.md §7.3,
//! Phase 2). Until then they live here, beside the tables that list them and the compiler that
//! writes the keys they read ([`CountyKeys`]), in a crate with no engine, egui, file, thread or
//! clock. v1's two oracle lenses, `gap.oracle` and `gap.wage`, wait for that crate's
//! `oracle_gap`, and [`value`] says so. The second pass brings `oracle_gap` forward
//! ([`oracle_gap`]; decision 331): unit 1g at the county's params in force, solved outside any
//! `Sim` from the params the run recorded, so its three oracle lenses have values.
//!
//! The chain's two markups are the rules' own formulas, in the rules' order of summation: the
//! maker's net markup p_K·(1 − δ·a/κ)/c_m (`maker_reservation`, IDLE-SPEC §6) and the capacity
//! desk's p_h/(O + δ·p_K/κ); `demo_v2_lens_values_equal_the_engine` holds them to the engine's.
//!
//! Transcendentals go through `core::num` (U9): the change and gap lenses take a log.

use crate::compile::ROLES;
use crate::tables::{self, Lens, Param, Scale};
use crate::CompileError;
use rustyecon_core::{num, Clock};

/// v1's bundled table, `worlds/demo-gb/lenses.csv`.
pub const DEMO_GB_LENSES: &str = include_str!("../../../worlds/demo-gb/lenses.csv");

/// The second pass's bundled table, `worlds/demo-gb/lenses-v2a1.csv` (WORLD-V2 §7).
pub const DEMO_GB_V2_LENSES: &str = include_str!("../../../worlds/demo-gb/lenses-v2a1.csv");

/// v1's tape's name.
pub const V1_TAPE: &str = "demo-gb [illustrative]";

/// The second pass's tape's name (`stage-v2a1.csv`).
pub const V2_TAPE: &str = "demo-gb-v2 [illustrative]";

/// v1's lenses, in table order, read and checked from the bundled table.
pub fn demo_gb() -> Result<Vec<Lens>, CompileError> {
    tables::parse_lenses(DEMO_GB_LENSES)
}

/// The second pass's lenses, in table order, read and checked from the bundled table.
pub fn demo_gb_v2() -> Result<Vec<Lens>, CompileError> {
    tables::parse_lens_table("lenses-v2a1.csv", DEMO_GB_V2_LENSES)
}

/// The lens table of a tape by its name (WORLD-V2 §7.2): the second pass's for its tape, v1's
/// for any other.
pub fn for_tape(name: &str) -> Result<Vec<Lens>, CompileError> {
    match Kind::of_tape(name) {
        Kind::V1 => demo_gb(),
        Kind::V2a1 => demo_gb_v2(),
    }
}

/// Which demo world a tape is: its lens table, its markets and its keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// v1: the probe's four roles and four markets a county.
    V1,
    /// The second pass, stage v2a.1: six roles and six markets a county.
    V2a1,
}

impl Kind {
    /// The kind of the tape named `name`.
    pub fn of_tape(name: &str) -> Kind {
        if name == V2_TAPE {
            Kind::V2a1
        } else {
            Kind::V1
        }
    }
}

/// The trailing window the `no.trade`, `idle.horse` and `reserve.ticks` lenses count over: a
/// year of the demo's 52 ticks.
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
    /// p_m / r (v1).
    PriceMach,
    /// p_h / r, the horse-day's price (the second pass): at rest v1's p_m.
    PriceHday,
    /// The provider's due / (r·T_c).
    ReliefBurden,
    /// (due − paid) / due.
    Shortfall,
    /// The largest 1 − filled/requested over the county's rationing lines.
    Rationing,
    /// Ticks of the trailing [`NO_TRADE_WINDOW`] in which one of the county's markets that must
    /// trade did not (`MarketLine::trades`): v1's four; the second pass's five, all but the
    /// horse market, whose orders stop in a glut by design (O80). Not the probe's dead tick,
    /// which also counts a market that clears below half its oracle volume; so it is not called
    /// `dead`.
    NoTrade,
    /// D̂ against the county's own oracle point: v1's waits for `crates/observe`; the second
    /// pass's is [`oracle_gap`]'s, over P2.2a's observables but the horse market's volume.
    GapOracle,
    /// ln((w/r)/v*).
    GapWage,
    /// A county param as the run holds it, in its registered unit. On the second pass b and λ
    /// are not registered (decision 328), and their lenses show rule A's folds.
    Param(Param),
    /// (the capacity desk's herd + the maker's serving stock) / N, N per tick.
    HorsesPerHead,
    /// ln(heads / heads\*), heads\* at the params in force ([`oracle_gap`]).
    HorsesVsOracle,
    /// ln(the capacity desk's herd / its own target K\*).
    HorsesVsPlan,
    /// p_f / r.
    PriceFodder,
    /// The maker's net markup p_K·(1 − δ·a/κ)/c_m at posted prices: the horse's price over its
    /// replacement cost, with no value on a tick when no horse traded.
    PriceHorse,
    /// p_h / (O + δ·p_K/κ) − 1: the horse-day's markup over its full cost.
    HdayMarkup,
    /// Fodder's land over the land cleared: fodder.land · fodder made / T_c.
    LandToFodder,
    /// Fodder's land and the pasture over the land cleared.
    LandToHorses,
    /// Ticks of the trailing window in which the maker's markup was below its reservation ψ.
    ReserveTicks,
    /// Ticks of the trailing window in which no horse traded.
    IdleHorse,
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
            "price.hday" => Level::PriceHday,
            "relief.burden" => Level::ReliefBurden,
            "shortfall" => Level::Shortfall,
            "rationing" => Level::Rationing,
            "no.trade" => Level::NoTrade,
            "gap.oracle" => Level::GapOracle,
            "gap.wage" => Level::GapWage,
            "horses.per.head" => Level::HorsesPerHead,
            "horses.vs.oracle" => Level::HorsesVsOracle,
            "horses.vs.plan" => Level::HorsesVsPlan,
            "price.fodder" => Level::PriceFodder,
            "price.horse" => Level::PriceHorse,
            "hday.markup" => Level::HdayMarkup,
            "land.to.fodder" => Level::LandToFodder,
            "land.to.horses" => Level::LandToHorses,
            "reserve.ticks" => Level::ReserveTicks,
            "idle.horse" => Level::IdleHorse,
            _ => return None,
        })
    }

    /// Whether it needs the county's oracle point at its params in force: v1's wait for
    /// `crates/observe`; the second pass's are [`oracle_gap`]'s.
    pub fn needs_observe(self) -> bool {
        matches!(
            self,
            Level::GapOracle | Level::GapWage | Level::HorsesVsOracle
        )
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

/// The keys of the second pass's chain in a county (WORLD-V2 §2.2, §9): as `stage.rs` writes
/// them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageKeys {
    /// Fodder.
    pub fodder: &'static str,
    /// The horse, the durable good.
    pub horse: &'static str,
    /// Rule A's coefficients: `<node>.fodder.land`, `<node>.horse.own_hours`,
    /// `<node>.horse.labour`, `<node>.horse.land`.
    pub coef: [String; 4],
    /// κ, a year: `horse.kappa`.
    pub kappa: String,
    /// δ, a year: `horse.delta`.
    pub delta: String,
    /// A horse-day's fodder: `horse.run.fodder`.
    pub run_fodder: String,
    /// A horse-day's labour: `horse.run.labour`.
    pub run_labour: String,
    /// The maker's reservation ψ: `reserve.maker`.
    pub psi: String,
    /// The capacity desk.
    pub desk_capacity: String,
    /// The maker.
    pub desk_maker: String,
    /// The fodder desk.
    pub desk_fodder: String,
}

/// The tape keys a county's readings come from, as the compiler writes them: the county's
/// node, its markets' goods, its params `<node>.<column>`, its roles' actors `<node>.<role>`,
/// and on the second pass its chain's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountyKeys {
    /// The world the tape is.
    pub kind: Kind,
    /// The node: `county.<chapman>`.
    pub node: String,
    /// The goods of its four markets v1 has: labour, land, the machine-side hours (v1's
    /// machine services `mach`, the second pass's horse-days `traction`), the good.
    pub labour: &'static str,
    /// Land services.
    pub land: &'static str,
    /// Machine services (v1) or horse-days (the second pass).
    pub mach: &'static str,
    /// The good.
    pub good: &'static str,
    /// Each of v1's params, in [`Param::ALL`] order. On the second pass a, λ and b are not on
    /// the tape.
    pub params: [String; 11],
    /// The good desk.
    pub desk_good: String,
    /// v1's machine desk; on the second pass, the capacity desk, which sells the horse-days.
    pub desk_mach: String,
    /// The provider.
    pub provider: String,
    /// The workers.
    pub workers: String,
    /// The chain's keys, on the second pass.
    pub stage: Option<StageKeys>,
}

/// The goods of v1's four markets, in the order labour, land, machine services, the good.
pub const MARKETS: [&str; 4] = ["labour", "land", "mach", "good"];

/// The goods of the second pass's four markets in v1's slots: labour, land, horse-days, the good.
pub const MARKETS_V2: [&str; 4] = ["labour", "land", "traction", "good"];

impl CountyKeys {
    /// The keys of v1's county whose node is `node`.
    pub fn of(node: &str) -> CountyKeys {
        CountyKeys::of_kind(node, Kind::V1)
    }

    /// The keys of the county whose node is `node` on a tape of kind `kind`.
    pub fn of_kind(node: &str, kind: Kind) -> CountyKeys {
        let role = |r: &str| format!("{node}.{r}");
        let m = match kind {
            Kind::V1 => MARKETS,
            Kind::V2a1 => MARKETS_V2,
        };
        let stage = (kind == Kind::V2a1).then(|| StageKeys {
            fodder: "fodder",
            horse: "horse",
            coef: [
                role("fodder.land"),
                role("horse.own_hours"),
                role("horse.labour"),
                role("horse.land"),
            ],
            kappa: "horse.kappa".into(),
            delta: "horse.delta".into(),
            run_fodder: "horse.run.fodder".into(),
            run_labour: "horse.run.labour".into(),
            psi: "reserve.maker".into(),
            desk_capacity: role("desk.capacity"),
            desk_maker: role("desk.maker"),
            desk_fodder: role("desk.fodder"),
        });
        CountyKeys {
            kind,
            node: node.to_string(),
            labour: m[0],
            land: m[1],
            mach: m[2],
            good: m[3],
            params: Param::ALL.map(|p| format!("{node}.{}", p.column())),
            desk_good: role(ROLES[0]),
            desk_mach: match kind {
                Kind::V1 => role(ROLES[1]),
                Kind::V2a1 => role("desk.capacity"),
            },
            provider: role(ROLES[2]),
            workers: role(ROLES[3]),
            stage,
        }
    }

    /// A param's key.
    pub fn param(&self, p: Param) -> &str {
        &self.params[p.index()]
    }

    /// Every market of the county: v1's four, or the second pass's six (labour, land, fodder,
    /// the horse, horse-days, the good).
    pub fn markets(&self) -> Vec<&'static str> {
        match &self.stage {
            None => vec![self.labour, self.land, self.mach, self.good],
            Some(s) => vec![
                self.labour,
                self.land,
                s.fodder,
                s.horse,
                self.mach,
                self.good,
            ],
        }
    }

    /// The markets that must trade every tick, which `no.trade` counts: every market but the
    /// horse's, whose orders stop in a glut by design (O80).
    pub fn must_trade(&self) -> Vec<&'static str> {
        match &self.stage {
            None => self.markets(),
            Some(s) => vec![self.labour, self.land, s.fodder, self.mach, self.good],
        }
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

/// What the maker's net markup is formed from at one tick: posted prices and the params in
/// force per tick, as its rule reads them (`maker_reservation`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkupInputs {
    /// w.
    pub w: f64,
    /// r.
    pub r: f64,
    /// p_f.
    pub pf: f64,
    /// p_K.
    pub pk: f64,
    /// a, the head's own horse-days (`<node>.horse.own_hours`).
    pub own_hours: f64,
    /// κ per tick.
    pub kappa: f64,
    /// δ per tick.
    pub delta: f64,
    /// A horse-day's fodder.
    pub run_fodder: f64,
    /// A horse-day's labour.
    pub run_labour: f64,
    /// A head's labour (`<node>.horse.labour`).
    pub labour: f64,
    /// A head's pasture (`<node>.horse.land`).
    pub land: f64,
}

/// The maker's net markup p_K·(1 − δ·a/κ)/c_m at posted prices, summed as its rule sums it
/// (`maker_reservation`): the goods it buys per head (a·run_f + 0, fodder), then labour
/// (a·run_lab + build_lab), then land.
pub fn maker_markup(m: &MarkupInputs) -> f64 {
    let fodder = m.own_hours * m.run_fodder + 0.0;
    let lam = m.own_hours * m.run_labour + m.labour;
    let mut c = 0.0;
    c += fodder * m.pf;
    c += lam * m.w;
    c += m.land * m.r;
    let net = m.pk * (1.0 - m.delta * m.own_hours / m.kappa);
    net / c
}

/// One report tick of the trailing window, for the chain's counts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowTick {
    /// Whether a horse traded (`MarketLine::trades`).
    pub horse_traded: bool,
    /// The maker's markup inputs at that tick, where the record holds them all.
    pub markup: Option<MarkupInputs>,
    /// The maker's reservation ψ at that tick.
    pub psi: Option<f64>,
}

/// A county's chain readings on the second pass, at one report tick.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChainReadings {
    /// Fodder's posted price.
    pub price_fodder: Option<f64>,
    /// The horse's posted price.
    pub price_horse: Option<f64>,
    /// Fodder cleared.
    pub cleared_fodder: Option<f64>,
    /// Horses cleared.
    pub cleared_horse: Option<f64>,
    /// Whether a horse traded this tick.
    pub horse_traded: Option<bool>,
    /// Rule A's coefficients as registered, per tick: fodder.land, horse.own_hours,
    /// horse.labour, horse.land.
    pub coef: [Option<f64>; 4],
    /// κ per tick, through the clock's conversion for its use.
    pub kappa: Option<f64>,
    /// δ per tick, likewise.
    pub delta: Option<f64>,
    /// A horse-day's fodder.
    pub run_fodder: Option<f64>,
    /// A horse-day's labour.
    pub run_labour: Option<f64>,
    /// The maker's reservation ψ.
    pub psi: Option<f64>,
    /// The human share the good desk's last production used (`GoodDeskState::used`).
    pub used: Option<f64>,
    /// The good desk's output.
    pub out_good: Option<f64>,
    /// The capacity desk's output, horse-days.
    pub out_hours: Option<f64>,
    /// The maker's output, heads made.
    pub out_horse: Option<f64>,
    /// The fodder desk's output.
    pub out_fodder: Option<f64>,
    /// The capacity desk's herd when it decided (`CapacityState::held`).
    pub held: Option<f64>,
    /// Its target K\*.
    pub target: Option<f64>,
    /// The maker's serving stock (`MakerState::serving`).
    pub serving: Option<f64>,
    /// The tape's clock, for the oracle's per-tick N and T.
    pub clock: Option<Clock>,
    /// The trailing window, oldest first.
    pub window: Vec<WindowTick>,
}

impl ChainReadings {
    /// The maker's markup inputs at this tick, from the readings' prices.
    pub fn markup_inputs(&self, prices: &[Option<f64>; 4]) -> Option<MarkupInputs> {
        Some(MarkupInputs {
            w: prices[0]?,
            r: prices[1]?,
            pf: self.price_fodder?,
            pk: self.price_horse?,
            own_hours: self.coef[1]?,
            kappa: self.kappa?,
            delta: self.delta?,
            run_fodder: self.run_fodder?,
            run_labour: self.run_labour?,
            labour: self.coef[2]?,
            land: self.coef[3]?,
        })
    }
}

/// One county's recorded numbers at one report tick, as the run reported them. A number the
/// record lacks is `None`, and a lens that needs it has no value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Readings {
    /// The posted prices of labour (w), land (r), the machine-side hours (v1's p_m, the second
    /// pass's p_h) and the good (p), in the node's currency, as the tick's settlement used them
    /// (`MarketLine::price`).
    pub prices: [Option<f64>; 4],
    /// The cleared volumes of the four markets, per tick (`MarketLine::cleared`).
    pub cleared: [Option<f64>; 4],
    /// Each of v1's params after the tick, in [`Param::ALL`] order and its registered unit (N
    /// and T a year).
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
    /// whether all of the county's markets that must trade traded (`MarketLine::trades`).
    pub traded: Vec<bool>,
    /// The chain's readings, on the second pass.
    pub chain: Option<ChainReadings>,
}

/// Why a lens has no value for a county.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoValue {
    /// The lens needs what is not built: v1's oracle lenses wait for `crates/observe`.
    Unavailable(String),
    /// A number it reads is not in the record.
    Missing(&'static str),
    /// Its formula is undefined here, such as a ratio over zero.
    Undefined(&'static str),
    /// No horse traded this tick, so the horse has no price to show (O47, O54); the text names
    /// the posted price's markup.
    Idle(String),
}

impl std::fmt::Display for NoValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoValue::Unavailable(why) => write!(f, "unavailable: {why}"),
            NoValue::Missing(what) => write!(f, "not recorded: {what}"),
            NoValue::Undefined(why) => write!(f, "undefined: {why}"),
            NoValue::Idle(why) => write!(f, "idle: {why}"),
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

fn log_ratio(a: f64, b: f64, what: &'static str) -> Result<f64, NoValue> {
    if a > 0.0 && b > 0.0 && a.is_finite() && b.is_finite() {
        Ok(num::ln(a / b))
    } else {
        Err(NoValue::Undefined(what))
    }
}

fn chain_of(x: &Readings) -> Result<&ChainReadings, NoValue> {
    x.chain.as_ref().ok_or(NoValue::Missing(
        "the chain's readings (a lens of the second pass)",
    ))
}

/// A level's value from one county's readings.
pub fn level(l: Level, x: &Readings) -> Result<f64, NoValue> {
    // v1's oracle lenses wait for `crates/observe`: a county with no chain readings has none.
    if l.needs_observe() && x.chain.is_none() {
        return Err(NoValue::Unavailable(
            "the oracle lenses need crates/observe's oracle_gap (docs/GUI.md §7.3)".to_string(),
        ));
    }
    let price = |i: usize, what| need(x.prices[i], what);
    let cleared = |i: usize, what| need(x.cleared[i], what);
    let w = || price(0, "the price of labour");
    let r = || price(1, "the price of land");
    let pm = || price(2, "the price of the machine-side hours");
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
        Level::PriceMach | Level::PriceHday => ratio(pm()?, r()?, "the price of land is 0"),
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
        Level::GapOracle => {
            let g = oracle_gap::gaps(x)?;
            let d = g.iter().map(|(_, v)| v.abs()).fold(0.0, f64::max);
            Ok(d / oracle_gap::TOL)
        }
        Level::GapWage => {
            let e = oracle_gap::target(x)?;
            log_ratio(
                w()? / r()?,
                e.v,
                "the wage or the price of land is not positive",
            )
        }
        Level::Param(q) => match (x.params[q.index()], q, &x.chain) {
            (Some(v), _, _) => Ok(v),
            // Rule A's folds (decision 328): b = fodder.land + δ·horse.land/κ; λ =
            // δ·horse.labour/κ, κ and δ per tick.
            (None, Param::B, Some(c)) => {
                let kd = need(c.delta, "δ")? / need(c.kappa, "κ")?;
                Ok(need(c.coef[0], "fodder.land")? + kd * need(c.coef[3], "horse.land")?)
            }
            (None, Param::Lam, Some(c)) => {
                let kd = need(c.delta, "δ")? / need(c.kappa, "κ")?;
                Ok(kd * need(c.coef[2], "horse.labour")?)
            }
            _ => Err(NoValue::Missing("the param")),
        },
        Level::HorsesPerHead => {
            let c = chain_of(x)?;
            let heads = need(c.held, "the capacity desk's herd")?
                + need(c.serving, "the maker's serving stock")?;
            ratio(heads, need(x.n_tick, "N")?, "N is 0")
        }
        Level::HorsesVsOracle => {
            let c = chain_of(x)?;
            let e = oracle_gap::target(x)?;
            let heads = need(c.held, "the capacity desk's herd")?
                + need(c.serving, "the maker's serving stock")?;
            log_ratio(heads, e.heads(), "no heads")
        }
        Level::HorsesVsPlan => {
            let c = chain_of(x)?;
            log_ratio(
                need(c.held, "the capacity desk's herd")?,
                need(c.target, "the capacity desk's target")?,
                "no herd or no target",
            )
        }
        Level::PriceFodder => ratio(
            need(chain_of(x)?.price_fodder, "fodder's price")?,
            r()?,
            "the price of land is 0",
        ),
        Level::PriceHorse => {
            let c = chain_of(x)?;
            let m = c
                .markup_inputs(&x.prices)
                .ok_or(NoValue::Missing("the maker's markup inputs"))?;
            let markup = maker_markup(&m);
            match c.horse_traded {
                Some(true) => Ok(markup),
                Some(false) => Err(NoValue::Idle(format!(
                    "no horse traded this tick; the posted price is {} of the replacement cost",
                    three(markup)
                ))),
                None => Err(NoValue::Missing("whether a horse traded")),
            }
        }
        Level::HdayMarkup => {
            let c = chain_of(x)?;
            // The capacity desk's full cost at posted prices, as its rule sums it:
            // O = (0.0 + run_f·p_f) + run_lab·w, then O + δ·p_K/κ.
            let mut o = 0.0;
            o += need(c.run_fodder, "a horse-day's fodder")? * need(c.price_fodder, "p_f")?;
            let o = o + need(c.run_labour, "a horse-day's labour")? * w()?;
            let full = o + need(c.delta, "δ")? * need(c.price_horse, "p_K")? / need(c.kappa, "κ")?;
            Ok(ratio(pm()?, full, "the full cost is 0")? - 1.0)
        }
        Level::LandToFodder => {
            let c = chain_of(x)?;
            let land = need(c.coef[0], "fodder.land")? * need(c.out_fodder, "fodder made")?;
            ratio(land, cleared(1, "land cleared")?, "no land cleared")
        }
        Level::LandToHorses => {
            let c = chain_of(x)?;
            let fodder = need(c.coef[0], "fodder.land")? * need(c.out_fodder, "fodder made")?;
            let pasture = need(c.coef[3], "horse.land")? * need(c.out_horse, "heads made")?;
            ratio(
                fodder + pasture,
                cleared(1, "land cleared")?,
                "no land cleared",
            )
        }
        Level::ReserveTicks => {
            let c = chain_of(x)?;
            if c.window.is_empty() {
                return Err(NoValue::Missing("the trailing window"));
            }
            let mut n = 0.0;
            for t in &c.window {
                let (m, psi) = t
                    .markup
                    .zip(t.psi)
                    .ok_or(NoValue::Missing("the maker's markup inputs in the window"))?;
                // The rule's own test (`withholds`): only a reservation above 0 acts.
                if psi > 0.0 && maker_markup(&m) < psi {
                    n += 1.0;
                }
            }
            Ok(n)
        }
        Level::IdleHorse => {
            let c = chain_of(x)?;
            if c.window.is_empty() {
                return Err(NoValue::Missing("the trailing window"));
            }
            Ok(c.window.iter().filter(|t| !t.horse_traded).count() as f64)
        }
    }
}

/// A number to three significant figures, for a text.
fn three(v: f64) -> String {
    format!("{v:.3}")
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

/// The county's oracle point at its params in force, and its gaps (decision 331; WORLD-V2
/// §7.1): `crates/observe`'s `oracle_gap`, brought forward for the second pass. Unit 1g is
/// solved from the params the run recorded, outside any `Sim`, and each point is kept by its
/// params, so a run solves each county once a step date. Nothing here reaches an agent (R13).
pub mod oracle_gap {
    use super::{need, ChainReadings, NoValue, Readings};
    use crate::chain::{self, ChainParams, ChainPoint, RuleA};
    use crate::tables::Param;
    use rustyecon_core::num;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex, OnceLock};

    /// The probe's tolerance, 1e-3 in log (TOL_FLOOR): D̂ is in its multiples.
    pub const TOL: f64 = 1e-3;

    /// P2.2a's observables (HORSES-SPEC §7.1) but the horse market's volume, which idles by
    /// design (O80): the gap lens's seventeen.
    pub const OBSERVABLES: [&str; 17] = [
        "v",
        "pi.fodder",
        "pi.horse",
        "pi.traction",
        "pi.good",
        "s.good",
        "vol.labour",
        "vol.land",
        "vol.fodder",
        "vol.traction",
        "vol.good",
        "y.good",
        "y.traction",
        "y.horse",
        "y.fodder",
        "heads.capacity",
        "heads.maker",
    ];

    type Memo = Mutex<BTreeMap<Vec<u64>, Result<Arc<ChainPoint>, String>>>;

    fn memo() -> &'static Memo {
        static M: OnceLock<Memo> = OnceLock::new();
        M.get_or_init(|| Mutex::new(BTreeMap::new()))
    }

    /// The chain's params at the county's params in force, from its readings.
    pub fn params(x: &Readings) -> Result<(ChainParams, rustyecon_core::Clock), NoValue> {
        let c: &ChainReadings = x
            .chain
            .as_ref()
            .ok_or(NoValue::Missing("the chain's readings"))?;
        let v = |p: Param, what| need(x.params[p.index()], what);
        let clock = c.clock.ok_or(NoValue::Missing("the tape's clock"))?;
        Ok((
            ChainParams {
                workers: v(Param::Workers, "N")?,
                land: v(Param::Land, "T")?,
                space: v(Param::Space, "h")?,
                eta: v(Param::Eta, "η")?,
                g0: v(Param::G0, "g0")?,
                g1: v(Param::G1, "g1")?,
                k: v(Param::K, "k")?,
                chi_max: v(Param::ChiMax, "χ_max")?,
                rule: RuleA {
                    fodder_land: need(c.coef[0], "fodder.land")?,
                    own_hours: need(c.coef[1], "horse.own_hours")?,
                    labour: need(c.coef[2], "horse.labour")?,
                    pasture: need(c.coef[3], "horse.land")?,
                },
                kappa: need(c.kappa, "κ")?,
                delta: need(c.delta, "δ")?,
                fodder: true,
            },
            clock,
        ))
    }

    /// The county's oracle point (unit 1g, J = 2) at its params in force, from its readings;
    /// solved once for each set of params and kept.
    pub fn target(x: &Readings) -> Result<Arc<ChainPoint>, NoValue> {
        let (p, clock) = params(x)?;
        let mut key: Vec<u64> = [
            p.workers,
            p.land,
            p.space,
            p.eta,
            p.g0,
            p.g1,
            p.k,
            p.chi_max,
            p.rule.fodder_land,
            p.rule.own_hours,
            p.rule.labour,
            p.rule.pasture,
            p.kappa,
            p.delta,
        ]
        .iter()
        .map(|v| v.to_bits())
        .collect();
        key.push(u64::from(clock.ticks_per_year));
        let solved = {
            let m = memo()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            m.get(&key).cloned()
        };
        let got = match solved {
            Some(g) => g,
            None => {
                let g = chain::point(&p, &clock).map(Arc::new);
                memo()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(key, g.clone());
                g
            }
        };
        got.map_err(|_| NoValue::Undefined("the oracle has no interior point at these params"))
    }

    /// Each of [`OBSERVABLES`]' signed gap ln(o/o\*) at the report tick, in their order.
    pub fn gaps(x: &Readings) -> Result<Vec<(&'static str, f64)>, NoValue> {
        let c = x
            .chain
            .as_ref()
            .ok_or(NoValue::Missing("the chain's readings"))?;
        let e = target(x)?;
        let (_, clock) = params(x)?;
        let t = clock.flow(rustyecon_core::FlowPerYear(need(
            x.params[Param::Land.index()],
            "T",
        )?));
        let r = need(x.prices[1], "the price of land")?;
        let pr = |v: Option<f64>, what| -> Result<f64, NoValue> { Ok(need(v, what)? / r) };
        let obs = [
            pr(x.prices[0], "the wage")?,
            pr(c.price_fodder, "fodder's price")?,
            pr(c.price_horse, "the horse's price")?,
            pr(x.prices[2], "the horse-day's price")?,
            pr(x.prices[3], "the good's price")?,
            need(c.used, "the good desk's used share")?,
            need(x.cleared[0], "labour cleared")?,
            need(x.cleared[1], "land cleared")?,
            need(c.cleared_fodder, "fodder cleared")?,
            need(x.cleared[2], "horse-days cleared")?,
            need(x.cleared[3], "the good cleared")?,
            need(c.out_good, "the good made")?,
            need(c.out_hours, "horse-days made")?,
            need(c.out_horse, "heads made")?,
            need(c.out_fodder, "fodder made")?,
            need(c.held, "the capacity desk's herd")?,
            need(c.serving, "the maker's serving stock")?,
        ];
        let target = [
            e.v,
            e.pf,
            e.pk,
            e.ph,
            e.p,
            e.one_minus_x,
            e.n_a,
            t,
            e.qf,
            e.task_hours,
            e.good,
            e.good,
            e.task_hours,
            e.made,
            e.qf,
            e.capacity,
            e.serving,
        ];
        let mut out = Vec::with_capacity(OBSERVABLES.len());
        for ((name, o), t) in OBSERVABLES.iter().zip(obs).zip(target) {
            if !(o > 0.0 && o.is_finite()) {
                return Err(NoValue::Undefined("an observable is not positive"));
            }
            out.push((*name, num::ln(o / t)));
        }
        Ok(out)
    }
}
