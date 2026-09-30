//! The world tables, read and checked row by row (docs/demo/WORLD.md §3, §4, §6, §8).
//!
//! - `world.csv`: the tape's name, the basis marker, the clock, the stepping rule and the dials
//!   every county shares.
//! - `counties.csv`: the rough facts `derive.py` reads; the compiler checks their keys, codes,
//!   names and areas against the atlas.
//! - `regions.csv`: each county's instance at the start date, with the tags the history's
//!   weights read, the reserved columns for categories, machine types and carriers, and a basis.
//! - `history.csv`: the dated ramps.
//! - `lenses.csv`: the map's lenses, checked here so that the GUI reads a well-formed table.

use crate::atlas::{Atlas, Country};
use crate::csv::{Row, Table};
use crate::CompileError;
use rustyecon_core::Date;
use std::collections::{BTreeMap, BTreeSet};

/// The texts of a world's tables, as the cli reads them from `worlds/<world>/`. The compiler
/// does no file I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tables {
    /// `world.csv`.
    pub world: String,
    /// `counties.csv`.
    pub counties: String,
    /// `regions.csv`.
    pub regions: String,
    /// `history.csv`.
    pub history: String,
    /// `lenses.csv`.
    pub lenses: String,
}

impl Tables {
    /// The file names, in the order of the fields.
    pub const FILES: [&'static str; 5] = [
        "world.csv",
        "counties.csv",
        "regions.csv",
        "history.csv",
        "lenses.csv",
    ];
}

/// One of a county's instance params: the columns of `regions.csv` a tape registers, and the
/// params a history row may move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Param {
    /// N, potential hours a year.
    Workers,
    /// T, land services a year.
    Land,
    /// h, the space in one basket.
    Space,
    /// η, the schedule's scale.
    Eta,
    /// g0.
    G0,
    /// g1.
    G1,
    /// k.
    K,
    /// a, the machine desk's own input.
    A,
    /// λ, its hours.
    Lam,
    /// b, its land.
    B,
    /// χ_max, the top of the work cost's support.
    ChiMax,
}

impl Param {
    /// Every param, in the order the tape lists them.
    pub const ALL: [Param; 11] = [
        Param::Workers,
        Param::Land,
        Param::Space,
        Param::Eta,
        Param::G0,
        Param::G1,
        Param::K,
        Param::A,
        Param::Lam,
        Param::B,
        Param::ChiMax,
    ];

    /// Its column in `regions.csv` and the last part of its tape key.
    pub fn column(self) -> &'static str {
        match self {
            Param::Workers => "workers",
            Param::Land => "land",
            Param::Space => "space",
            Param::Eta => "eta",
            Param::G0 => "g0",
            Param::G1 => "g1",
            Param::K => "k",
            Param::A => "a",
            Param::Lam => "lam",
            Param::B => "b",
            Param::ChiMax => "chi_max",
        }
    }

    /// The unit the tape registers it in: N and T are flows a year, the rest dimensionless.
    pub fn unit(self) -> &'static str {
        match self {
            Param::Workers | Param::Land => "FlowPerYear",
            _ => "Dimensionless",
        }
    }

    /// Its index in [`Param::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }

    fn of(column: &str) -> Option<Param> {
        Param::ALL.into_iter().find(|p| p.column() == column)
    }
}

/// A county's instance: the eleven params, in [`Param::ALL`] order, as the tape registers
/// them (N and T a year).
pub type Instance = [f64; 11];

/// The tags a history weight or selector may read, each in [0, 1] (WORLD.md §3.1).
pub const TAGS: [&str; 15] = [
    "upland",
    "coal",
    "textile",
    "eng",
    "metro",
    "port",
    "canal",
    "openfield",
    "improver",
    "fen",
    "mining",
    "slate",
    "highland",
    "speen",
    "kelp",
];

/// The kinds of factory textile (and the other trades `textile.other` covers).
pub const KINDS: [&str; 9] = [
    "cotton", "wool", "linen", "hosiery", "silk", "pottery", "ribbons", "carpets", "boots",
];

/// A nation, as the tables name it. `ireland` is the six counties of Northern Ireland, the only
/// part of Ireland the atlas holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nation {
    /// `england`.
    England,
    /// `wales`, with Monmouthshire.
    Wales,
    /// `scotland`.
    Scotland,
    /// `ireland`: the six counties.
    Ireland,
}

impl Nation {
    fn of(s: &str) -> Option<Nation> {
        match s {
            "england" => Some(Nation::England),
            "wales" => Some(Nation::Wales),
            "scotland" => Some(Nation::Scotland),
            "ireland" => Some(Nation::Ireland),
            _ => None,
        }
    }

    fn country(self) -> Country {
        match self {
            Nation::England => Country::England,
            Nation::Wales => Country::Wales,
            Nation::Scotland => Country::Scotland,
            Nation::Ireland => Country::NorthernIreland,
        }
    }
}

/// A dial every county shares, registered once.
#[derive(Debug, Clone, PartialEq)]
pub struct Dial {
    /// Its tape key.
    pub key: String,
    /// Its value.
    pub value: f64,
    /// Its unit, as the tape writes it.
    pub unit: &'static str,
    /// Its basis text, after the world's marker.
    pub note: String,
}

/// The dials the four roles and the ledger read, with their units, in the order the tape
/// lists them. `life.one_tick` is not here: it is one tick at the world's clock.
pub const DIALS: [(&str, &str); 14] = [
    ("ledger.rel_flow", "Dimensionless"),
    ("ledger.rel_stock", "Dimensionless"),
    ("price.ema_tc", "Years"),
    ("rate.labour", "RatePerYear"),
    ("rate.land", "RatePerYear"),
    ("rate.mach", "RatePerYear"),
    ("rate.good", "RatePerYear"),
    ("adjust.technique", "RatePerYear"),
    ("spend.workers", "RatePerYear"),
    ("spend.provider", "RatePerYear"),
    ("buffer.desk.good.cash", "RatePerYear"),
    ("buffer.desk.mach.cash", "RatePerYear"),
    ("tilt.desk.good", "Dimensionless"),
    ("tilt.desk.mach", "Dimensionless"),
];

/// The probe's registered dials, C2 (docs/probe/RULES.md §3, §5; REPORT §4), in the units of
/// [`DIALS`]. A world's dials must be these: C2 is the one point at which the probe's battery,
/// its kicks and this history's run are evidence (O14), so a world at other dials needs its own
/// probe run first. `dials_are_the_probes_registered_c2` holds this list to
/// `probe::setup::Dials::registered()`. The ledger's tolerances and `price.ema_tc` are not the
/// probe's dials and are not listed.
pub const C2: [(&str, f64); 11] = [
    ("rate.labour", 5.2),
    ("rate.land", 1.3),
    ("rate.mach", 1.3),
    ("rate.good", 2.6),
    ("adjust.technique", 2.6),
    ("spend.workers", 13.0),
    ("spend.provider", 13.0),
    ("buffer.desk.good.cash", 5.2),
    ("buffer.desk.mach.cash", 5.2),
    ("tilt.desk.good", 0.0),
    ("tilt.desk.mach", 0.0),
];

/// The one machine type a county may name in `regions.csv` (WORLD-V2 §9.1; decision 327): the
/// horse, which v1 compiles as its flow machine (rule A's collapse at rho = 0) and stage v2a.1 as a
/// durable good.
pub const MACHINE_TYPE: &str = "horse";

/// The ticks a year every world runs at: the probe's registered tick (decisions 121, 237), at
/// which its battery, its kicks and the demo's long run are evidence. The compiler refuses any
/// other clock, as it refuses dials off their set (O39, D2.1).
pub const TICKS_PER_YEAR: u32 = 52;

/// The most `max_step` may be, in log: the design's gradual history, whose largest move at one
/// date is 0.026, ran with no dead tick and no shortfall (docs/demo/WORLD.md §4.4; O14). A
/// larger `max_step` would let an abrupt history compile.
pub const MAX_STEP_CEILING: f64 = 0.03;

/// The most `step_log` may be, in log: the coarsest stepping that has been run, §4.4's 2%
/// fallback.
pub const STEP_LOG_CEILING: f64 = 0.02;

/// What an illustrative world's tape name carries (R4, R5), and what its basis marker starts
/// with. Until Phase 4's research tables, the compiler writes illustrative worlds only.
pub const ILLUSTRATIVE_NAME: &str = "[illustrative]";

/// See [`ILLUSTRATIVE_NAME`].
pub const ILLUSTRATIVE_BASIS: &str = "illustrative";

/// `world.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct World {
    /// The tape's name. It carries [`ILLUSTRATIVE_NAME`].
    pub name: String,
    /// The basis marker: every entry's basis is `Assumed("<marker>: <why>")`. It starts with
    /// [`ILLUSTRATIVE_BASIS`].
    pub basis: String,
    /// Tick 0's date, the first of a year.
    pub start: Date,
    /// The history's horizon, the first of a year: no ramp ends after it.
    pub end: Date,
    /// Ticks a year.
    pub ticks_per_year: u32,
    /// `Saturate` or `Hold`.
    pub one_sided: String,
    /// A step is emitted when a param has moved this much in log since the last.
    pub step_log: f64,
    /// The most the oracle's relative prices and technique may move at one date, in log.
    pub max_step: f64,
    /// The dials, in [`DIALS`] order.
    pub dials: Vec<Dial>,
}

impl World {
    /// The first year.
    pub fn start_year(&self) -> i32 {
        self.start.y
    }

    /// The horizon's year.
    pub fn end_year(&self) -> i32 {
        self.end.y
    }

    /// The value of a dial by key.
    pub fn dial(&self, key: &str) -> Option<f64> {
        self.dials.iter().find(|d| d.key == key).map(|d| d.value)
    }
}

/// A row of `counties.csv`, as far as the compiler reads it.
#[derive(Debug, Clone, PartialEq)]
pub struct CountyFacts {
    /// The region key.
    pub key: String,
    /// Its area, square kilometres, as derive.py read it.
    pub area_km2: f64,
}

/// A row of `regions.csv`: one county.
#[derive(Debug, Clone, PartialEq)]
pub struct County {
    /// `county.<chapman>`, the atlas's key and the node's.
    pub key: String,
    /// The Chapman code.
    pub chapman: String,
    /// The Historic Counties Standard code.
    pub hcs: String,
    /// The atlas's name.
    pub name: String,
    /// The nation.
    pub nation: Nation,
    /// The instance at the start date.
    pub genesis: Instance,
    /// The tags, in [`TAGS`] order.
    pub tags: [f64; 15],
    /// The textile kind, when `textile` is above 0.
    pub textile_kind: Option<String>,
    /// The year factory textiles start, when `textile` is above 0.
    pub textile_from: Option<i32>,
    /// Reserved: the county's categories (now `good`).
    pub categories: Vec<String>,
    /// Its machine types: the one durable good its flow machine stands for, `horse` (decision
    /// 327). v1 compiles it as its flow machine; a stage as the durable good.
    pub machine_types: Vec<String>,
    /// Reserved: its carriers (now none).
    pub carriers: Vec<String>,
    /// How its instance was derived: the basis text of its params.
    pub why: String,
    /// Its line in `regions.csv`.
    pub line: usize,
}

impl County {
    /// A tag's value.
    pub fn tag(&self, name: &str) -> f64 {
        TAGS.iter()
            .position(|t| *t == name)
            .map_or(0.0, |i| self.tags[i])
    }
}

/// When a ramp starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// The first of January of this year.
    Year(i32),
    /// The county's own `textile_from`.
    TextileFrom,
}

/// Which counties a ramp moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// Every county.
    All,
    /// The counties of these nations.
    Nations(Vec<Nation>),
    /// The counties whose tag is above 0.
    Tag(String),
    /// The textile counties of these kinds.
    Kinds(Vec<String>),
    /// These counties, by key.
    Counties(Vec<String>),
}

impl Selector {
    /// Whether it selects `c`.
    pub fn selects(&self, c: &County) -> bool {
        match self {
            Selector::All => true,
            Selector::Nations(n) => n.contains(&c.nation),
            Selector::Tag(t) => c.tag(t) > 0.0,
            Selector::Kinds(k) => {
                c.tag("textile") > 0.0 && c.textile_kind.as_ref().is_some_and(|x| k.contains(x))
            }
            Selector::Counties(keys) => keys.contains(&c.key),
        }
    }
}

/// How much a ramp moves a county: its exponent on the ramp's factor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Weight {
    /// 1.
    One,
    /// A tag.
    Tag(String),
    /// Upland outside the Highlands: upland·(1 − highland).
    Waste,
    /// 1 − upland.
    Lowland,
    /// 1 − coal/2.
    Offcoal,
}

impl Weight {
    /// Its value for `c`.
    pub fn of(&self, c: &County) -> f64 {
        match self {
            Weight::One => 1.0,
            Weight::Tag(t) => c.tag(t),
            Weight::Waste => c.tag("upland") * (1.0 - c.tag("highland")),
            Weight::Lowland => 1.0 - c.tag("upland"),
            Weight::Offcoal => 1.0 - 0.5 * c.tag("coal"),
        }
    }
}

/// How a ramp moves its param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Multiplied by a factor moving geometrically from 1 to `to^w`.
    Scale,
    /// Moved geometrically from `from` to `to`, absolute.
    Path,
}

/// A row of `history.csv`: one ramp.
#[derive(Debug, Clone, PartialEq)]
pub struct Ramp {
    /// Its key; a step's basis names the ramps that moved it.
    pub key: String,
    /// When it starts.
    pub start: Start,
    /// The year it ends, on the first of January.
    pub end: i32,
    /// The counties it moves.
    pub regions: Selector,
    /// The param it moves.
    pub param: Param,
    /// How.
    pub mode: Mode,
    /// For `scale` 1; for `path` the value at `start`.
    pub from: f64,
    /// For `scale` the factor at `end` (weight 1); for `path` the value at `end`.
    pub to: f64,
    /// The grid on which the compiler looks for a step, a divisor of 12.
    pub steps_per_year: u32,
    /// The weight.
    pub weight: Weight,
    /// The paper's lever.
    pub lever: String,
    /// A note.
    pub note: String,
    /// Its line in `history.csv`.
    pub line: usize,
}

/// A lens's colour scale (docs/GUI.md §4; U-rules: neutral scales).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Sequential, linear.
    Sequential,
    /// Sequential on a log axis.
    SequentialLog,
    /// Diverging around a reference.
    Diverging,
}

/// Where a lens reads its inputs (docs/demo/WORLD.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensSource {
    /// The tick report alone.
    Report,
    /// The report and the params.
    ReportParam,
    /// The params alone.
    Param,
    /// The actors' states, which the GUI's Extractor records.
    Extractor,
    /// The actors' states and the params together (the second pass, D2.3).
    ExtractorParam,
    /// The oracle, through `crates/observe` (not built yet): v1's oracle lenses, which have no
    /// value until it is.
    Observe,
    /// The oracle at the county's params in force through worldgen's `lens::oracle_gap`,
    /// `crates/observe`'s in waiting (the second pass, D2.3; decision 331).
    Oracle,
}

/// A row of `lenses.csv`: one of the map's lenses.
#[derive(Debug, Clone, PartialEq)]
pub struct Lens {
    /// Its key.
    pub key: String,
    /// Its name, which says what it measures.
    pub name: String,
    /// The measure, as a formula over the county's prices, volumes, params and states.
    pub measure: String,
    /// The unit of its values.
    pub unit: String,
    /// Its colour scale.
    pub scale: Scale,
    /// What its reference value means, if it has one.
    pub reference: String,
    /// The fixed domain the scale spans, for the whole run.
    pub domain: (f64, f64),
    /// Its inputs.
    pub inputs: String,
    /// Where they come from.
    pub source: LensSource,
    /// A note.
    pub note: String,
}

/// The tables, read and checked.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    /// `world.csv`.
    pub world: World,
    /// `regions.csv`, in key order.
    pub counties: Vec<County>,
    /// `history.csv`, in table order.
    pub ramps: Vec<Ramp>,
    /// `lenses.csv`, in table order.
    pub lenses: Vec<Lens>,
}

fn at(t: &Table, r: &Row) -> String {
    format!("{}:{}", t.file, r.line)
}

fn number(t: &Table, r: &Row, col: &str) -> Result<f64, CompileError> {
    let s = r.get(t.col(col));
    match s.parse::<f64>() {
        Ok(v) if v.is_finite() && !(v == 0.0 && v.is_sign_negative()) => Ok(v),
        _ => Err(CompileError::new(
            format!("{} {col}", at(t, r)),
            format!("`{s}` is not a finite number"),
        )),
    }
}

fn year(t: &Table, r: &Row, col: &str) -> Result<i32, CompileError> {
    let s = r.get(t.col(col));
    s.parse::<i32>().map_err(|_| {
        CompileError::new(
            format!("{} {col}", at(t, r)),
            format!("`{s}` is not a year"),
        )
    })
}

fn check_key(where_: &str, key: &str) -> Result<(), CompileError> {
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
    {
        return Err(CompileError::new(
            where_,
            format!("`{key}` is not a key (a-z, 0-9, `_`, `.` and `-`)"),
        ));
    }
    Ok(())
}

fn text(where_: &str, what: &str, s: &str) -> Result<(), CompileError> {
    if s.chars().any(char::is_control) {
        return Err(CompileError::new(
            where_,
            format!("{what} holds a control character"),
        ));
    }
    Ok(())
}

fn list(s: &str) -> Vec<String> {
    s.split(' ')
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect()
}

/// Read `world.csv`.
fn world(text_: &str) -> Result<World, CompileError> {
    let t = Table::parse("world.csv", text_, &["key", "value", "unit", "note"])?;
    let mut rows: BTreeMap<String, (String, String, String, String)> = BTreeMap::new();
    for r in &t.rows {
        let key = r.get(0).to_string();
        let w = at(&t, r);
        check_key(&w, &key)?;
        text(&w, "the note", r.get(3))?;
        if rows
            .insert(
                key.clone(),
                (
                    r.get(1).to_string(),
                    r.get(2).to_string(),
                    r.get(3).to_string(),
                    w.clone(),
                ),
            )
            .is_some()
        {
            return Err(CompileError::new(w, format!("`{key}` appears twice")));
        }
    }
    let mut take = |key: &str, unit: &str| -> Result<(String, String, String), CompileError> {
        let Some((v, u, n, w)) = rows.remove(key) else {
            return Err(CompileError::new("world.csv", format!("no row `{key}`")));
        };
        if u != unit {
            return Err(CompileError::new(
                format!("{w} {key}"),
                format!("the unit is `{u}`, not `{unit}`"),
            ));
        }
        Ok((v, n, w))
    };
    let date = |(v, _, w): (String, String, String)| -> Result<Date, CompileError> {
        let d = Date::parse(&v).map_err(|e| CompileError::new(&w, e.to_string()))?;
        if d.m != 1 || d.d != 1 {
            return Err(CompileError::new(&w, "the date is the first of January"));
        }
        Ok(d)
    };
    let positive = |(v, _, w): (String, String, String)| -> Result<f64, CompileError> {
        match v.parse::<f64>() {
            Ok(x) if x.is_finite() && x > 0.0 => Ok(x),
            _ => Err(CompileError::new(
                &w,
                format!("`{v}` is not a finite positive number"),
            )),
        }
    };
    let (name, _, w) = take("name", "")?;
    let (basis, _, wb) = take("basis", "")?;
    if name.is_empty() || basis.is_empty() {
        return Err(CompileError::new(
            w,
            "the name and the basis marker are text",
        ));
    }
    text(&wb, "the basis marker", &basis)?;
    text(&w, "the name", &name)?;
    // Until Phase 4's research tables, every world this compiler writes is illustrative, so the
    // marker cannot be dropped from both cells to make an unmarked tape of the same tables.
    if !basis.starts_with(ILLUSTRATIVE_BASIS) {
        return Err(CompileError::new(
            wb,
            format!(
                "the basis marker starts with `{ILLUSTRATIVE_BASIS}`: until Phase 4's research \
                 tables, this compiler writes illustrative worlds only (R4, R5)"
            ),
        ));
    }
    if !name.contains(ILLUSTRATIVE_NAME) {
        return Err(CompileError::new(
            w,
            format!("an illustrative world's name carries the marker {ILLUSTRATIVE_NAME} (R4, R5)"),
        ));
    }
    let start = date(take("start", "")?)?;
    let end = date(take("end", "")?)?;
    if end.y <= start.y {
        return Err(CompileError::new(
            "world.csv end",
            "the end is after the start",
        ));
    }
    let (tpy, _, wt) = take("ticks_per_year", "")?;
    let ticks_per_year = match tpy.parse::<u32>() {
        Ok(n) if n > 0 => n,
        _ => {
            return Err(CompileError::new(
                wt,
                format!("`{tpy}` is not a positive whole number"),
            ))
        }
    };
    // O39 (D2.1, 2026-09-30): the clock is held to the probe's registered tick, as the dials are
    // held to their set. At 4 ticks a year v1's tables compiled, and the run had 46,548 dead
    // county-ticks (docs/demo/WORLD-V2.md §10; decisions 121, 237).
    if ticks_per_year != TICKS_PER_YEAR {
        return Err(CompileError::new(
            wt,
            format!(
                "ticks_per_year is {ticks_per_year}, not {TICKS_PER_YEAR}: the clock is held to \
                 the probe's registered tick, as the dials are to their set (O39; decisions 121, \
                 237)"
            ),
        ));
    }
    let (one_sided, _, wo) = take("one_sided", "")?;
    if one_sided != "Saturate" && one_sided != "Hold" {
        return Err(CompileError::new(wo, "one_sided is Saturate or Hold"));
    }
    let mut ceiling = |key: &str, most: f64, why: &str| -> Result<f64, CompileError> {
        let row = take(key, "Dimensionless")?;
        let at = row.2.clone();
        let v = positive(row)?;
        if v > most {
            return Err(CompileError::new(
                at,
                format!("{key} {v} is above its ceiling {most}: {why} (O14)"),
            ));
        }
        Ok(v)
    };
    let step_log = ceiling(
        "step_log",
        STEP_LOG_CEILING,
        "the coarsest stepping that has been run (docs/demo/WORLD.md §4.4)",
    )?;
    let max_step = ceiling(
        "max_step",
        MAX_STEP_CEILING,
        "a larger bound would let an abrupt history compile",
    )?;
    let mut dials = Vec::new();
    for (key, unit) in DIALS {
        let (v, note, w) = take(key, unit)?;
        let value = match v.parse::<f64>() {
            Ok(x) if x.is_finite() && x >= 0.0 && !(x == 0.0 && x.is_sign_negative()) => x,
            _ => {
                return Err(CompileError::new(
                    w,
                    format!("`{v}` is not a finite number at least 0"),
                ))
            }
        };
        if key.starts_with("ledger.") && !(value > 0.0 && value < 1.0) {
            return Err(CompileError::new(w, "a ledger tolerance is in (0, 1)"));
        }
        if let Some(&(_, c2)) = C2.iter().find(|(k, _)| *k == key) {
            if value != c2 {
                return Err(CompileError::new(
                    w,
                    format!(
                        "{key} is {value}, not the probe's registered C2 value {c2}: C2 is the \
                         one point at which the probe's battery and this history's run are \
                         evidence (docs/probe/RULES.md §3, REPORT §4; O14)"
                    ),
                ));
            }
        }
        dials.push(Dial {
            key: key.to_string(),
            value,
            unit,
            note,
        });
    }
    if let Some((key, (_, _, _, w))) = rows.into_iter().next() {
        return Err(CompileError::new(w, format!("unknown row `{key}`")));
    }
    Ok(World {
        name,
        basis,
        start,
        end,
        ticks_per_year,
        one_sided,
        step_log,
        max_step,
        dials,
    })
}

const COUNTY_COLUMNS: [&str; 28] = [
    "key",
    "chapman",
    "hcs",
    "name",
    "nation",
    "area_km2",
    "pop1750_k",
    "pop1801_k",
    "pop1841_k",
    "pop1851_k",
    "pop1901_k",
    "upland",
    "coal",
    "textile",
    "textile_kind",
    "textile_from",
    "eng",
    "metro",
    "port",
    "canal",
    "openfield",
    "improver",
    "fen",
    "mining",
    "slate",
    "highland",
    "speen",
    "kelp",
];

/// The columns of `regions.csv`, in order.
pub const REGION_COLUMNS: [&str; 37] = [
    "key",
    "chapman",
    "hcs",
    "name",
    "nation",
    "workers",
    "land",
    "space",
    "eta",
    "g0",
    "g1",
    "k",
    "a",
    "lam",
    "b",
    "chi_max",
    "upland",
    "coal",
    "textile",
    "textile_kind",
    "textile_from",
    "eng",
    "metro",
    "port",
    "canal",
    "openfield",
    "improver",
    "fen",
    "mining",
    "slate",
    "highland",
    "speen",
    "kelp",
    "categories",
    "machine_types",
    "carriers",
    "why",
];

/// Check a region's key, codes, name and nation against the atlas.
fn against_atlas(
    w: &str,
    atlas: &Atlas,
    key: &str,
    chapman: &str,
    hcs: &str,
    name: &str,
    nation: &str,
) -> Result<Nation, CompileError> {
    let Some(region) = atlas.region(key) else {
        return Err(CompileError::new(
            w,
            format!("`{key}` is not a region of the atlas"),
        ));
    };
    if region.chapman != chapman {
        return Err(CompileError::new(
            w,
            format!("{key}'s Chapman code is {}, not {chapman}", region.chapman),
        ));
    }
    if !region.hcs.iter().any(|h| h == hcs) {
        return Err(CompileError::new(
            w,
            format!("{key}'s HCS codes are {:?}, without {hcs}", region.hcs),
        ));
    }
    if region.name != name {
        return Err(CompileError::new(
            w,
            format!(
                "{key}'s name is `{}` in the atlas, not `{name}`",
                region.name
            ),
        ));
    }
    let Some(n) = Nation::of(nation) else {
        return Err(CompileError::new(
            w,
            format!("`{nation}` is not england, wales, scotland or ireland"),
        ));
    };
    if n.country() != region.country {
        return Err(CompileError::new(
            w,
            format!(
                "{key} lies in {:?} in the atlas, not {nation}",
                region.country
            ),
        ));
    }
    Ok(n)
}

/// Every region of the atlas has a row, and no other key does.
fn same_keys<'a>(
    file: &str,
    atlas: &Atlas,
    keys: impl Iterator<Item = &'a str>,
) -> Result<(), CompileError> {
    let mut seen = BTreeSet::new();
    for k in keys {
        if !seen.insert(k) {
            return Err(CompileError::new(file, format!("`{k}` has two rows")));
        }
    }
    for r in &atlas.regions {
        if !seen.contains(r.key.as_str()) {
            return Err(CompileError::new(
                file,
                format!("the atlas's region {} ({}) has no row", r.key, r.name),
            ));
        }
    }
    Ok(())
}

fn counties(text_: &str, atlas: &Atlas) -> Result<Vec<CountyFacts>, CompileError> {
    let t = Table::parse("counties.csv", text_, &COUNTY_COLUMNS)?;
    let mut out = Vec::new();
    for r in &t.rows {
        let w = at(&t, r);
        let key = r.get(0);
        against_atlas(&w, atlas, key, r.get(1), r.get(2), r.get(3), r.get(4))?;
        let area = number(&t, r, "area_km2")?;
        let atlas_area = atlas.region(key).map_or(0.0, |x| x.area_km2);
        // counties.csv writes the atlas's area to 0.1 km2.
        if (area - atlas_area).abs() > 0.05 + 1e-9 {
            return Err(CompileError::new(
                format!("{w} area_km2"),
                format!("{area} km2, where the atlas measures {atlas_area}"),
            ));
        }
        out.push(CountyFacts {
            key: key.to_string(),
            area_km2: area,
        });
    }
    same_keys("counties.csv", atlas, out.iter().map(|c| c.key.as_str()))?;
    Ok(out)
}

fn regions(text_: &str, atlas: &Atlas, w: &World) -> Result<Vec<County>, CompileError> {
    let t = Table::parse("regions.csv", text_, &REGION_COLUMNS)?;
    let mut out = Vec::new();
    for r in &t.rows {
        let wr = at(&t, r);
        let key = r.get(0).to_string();
        let nation = against_atlas(&wr, atlas, &key, r.get(1), r.get(2), r.get(3), r.get(4))?;
        let mut genesis = [0.0; 11];
        for p in Param::ALL {
            let v = number(&t, r, p.column())?;
            let ok = match p {
                Param::G0 | Param::G1 | Param::Lam => v >= 0.0,
                Param::A => (0.0..1.0).contains(&v),
                _ => v > 0.0,
            };
            if !ok {
                return Err(CompileError::new(
                    format!("{wr} {} {}", key, p.column()),
                    format!("{v} is out of range"),
                ));
            }
            genesis[p.index()] = v;
        }
        let mut tags = [0.0; 15];
        for (i, tag) in TAGS.iter().enumerate() {
            let v = number(&t, r, tag)?;
            if !(0.0..=1.0).contains(&v) {
                return Err(CompileError::new(
                    format!("{wr} {key} {tag}"),
                    format!("{v} is not in [0, 1]"),
                ));
            }
            tags[i] = v;
        }
        let kind = r.get(t.col("textile_kind"));
        let from = r.get(t.col("textile_from"));
        let (textile_kind, textile_from) = if tags[2] > 0.0 {
            if !KINDS.contains(&kind) {
                return Err(CompileError::new(
                    format!("{wr} {key} textile_kind"),
                    format!("`{kind}` is not one of {KINDS:?}"),
                ));
            }
            let y = year(&t, r, "textile_from")?;
            if y < w.start_year() || y >= w.end_year() {
                return Err(CompileError::new(
                    format!("{wr} {key} textile_from"),
                    format!("{y} is outside the world's years"),
                ));
            }
            (Some(kind.to_string()), Some(y))
        } else {
            if !kind.is_empty() || !from.is_empty() {
                return Err(CompileError::new(
                    format!("{wr} {key}"),
                    "a county with textile 0 has no textile_kind or textile_from",
                ));
            }
            (None, None)
        };
        let categories = list(r.get(t.col("categories")));
        let machine_types = list(r.get(t.col("machine_types")));
        let carriers = list(r.get(t.col("carriers")));
        // Reserved (WORLD.md §7): more categories wait for the many-market roles, and carriers
        // for transport desks and an equilibrium with trade. The one machine type is the horse
        // (WORLD-V2 §9.1; decision 327): v1 compiles it as its flow machine, rule A's collapse
        // at rho = 0, and a stage (`--stage v2a1`) as the durable good machine_types.csv defines.
        if categories != ["good"] || machine_types != [MACHINE_TYPE] || !carriers.is_empty() {
            return Err(CompileError::new(
                format!("{wr} {key}"),
                "categories `good`, machine_types `horse` and no carriers: more wait for the \
                 many-market roles, the goods chain's later stages and transport desks \
                 (docs/demo/WORLD.md §7, WORLD-V2.md §9.1)",
            ));
        }
        let why = r.get(t.col("why")).to_string();
        text(&wr, "why", &why)?;
        if why.is_empty() {
            return Err(CompileError::new(
                format!("{wr} {key} why"),
                "every county says how its instance was derived",
            ));
        }
        out.push(County {
            key,
            chapman: r.get(1).to_string(),
            hcs: r.get(2).to_string(),
            name: r.get(3).to_string(),
            nation,
            genesis,
            tags,
            textile_kind,
            textile_from,
            categories,
            machine_types,
            carriers,
            why,
            line: r.line,
        });
    }
    same_keys("regions.csv", atlas, out.iter().map(|c| c.key.as_str()))?;
    out.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(out)
}

fn selector(w: &str, s: &str, counties: &[County]) -> Result<Selector, CompileError> {
    if s == "all" {
        return Ok(Selector::All);
    }
    if s.starts_with("county.") {
        let keys = list(s);
        for k in &keys {
            if counties.binary_search_by(|c| c.key.cmp(k)).is_err() {
                return Err(CompileError::new(w, format!("no county `{k}`")));
            }
        }
        return Ok(Selector::Counties(keys));
    }
    let (kind, arg) = s.split_once(':').unwrap_or((s, ""));
    match kind {
        "nation" => arg
            .split('+')
            .map(|n| {
                Nation::of(n).ok_or_else(|| CompileError::new(w, format!("`{n}` is not a nation")))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Selector::Nations),
        "tag" if TAGS.contains(&arg) => Ok(Selector::Tag(arg.to_string())),
        "kind" => {
            let kinds: Vec<String> = arg.split('+').map(str::to_string).collect();
            if let Some(k) = kinds.iter().find(|k| !KINDS.contains(&k.as_str())) {
                return Err(CompileError::new(w, format!("`{k}` is not a textile kind")));
            }
            Ok(Selector::Kinds(kinds))
        }
        _ => Err(CompileError::new(
            w,
            format!("`{s}` is not `all`, `nation:a+b`, `tag:t`, `kind:k1+k2` or county keys"),
        )),
    }
}

fn weight(w: &str, s: &str) -> Result<Weight, CompileError> {
    match s {
        "" => Ok(Weight::One),
        "waste" => Ok(Weight::Waste),
        "lowland" => Ok(Weight::Lowland),
        "offcoal" => Ok(Weight::Offcoal),
        t if TAGS.contains(&t) => Ok(Weight::Tag(t.to_string())),
        _ => Err(CompileError::new(
            w,
            format!("`{s}` is not a tag, `waste`, `lowland`, `offcoal` or empty"),
        )),
    }
}

const HISTORY_COLUMNS: [&str; 12] = [
    "key",
    "start",
    "end",
    "regions",
    "param",
    "mode",
    "from",
    "to",
    "steps_per_year",
    "weight",
    "lever",
    "note",
];

fn history(text_: &str, w: &World, counties: &[County]) -> Result<Vec<Ramp>, CompileError> {
    let t = Table::parse("history.csv", text_, &HISTORY_COLUMNS)?;
    let mut keys = BTreeSet::new();
    let mut out = Vec::new();
    for r in &t.rows {
        let key = r.get(0).to_string();
        let wr = format!("{} {key}", at(&t, r));
        check_key(&wr, &key)?;
        if !keys.insert(key.clone()) {
            return Err(CompileError::new(wr, "the key appears twice"));
        }
        let start = match r.get(1) {
            "textile_from" => Start::TextileFrom,
            _ => Start::Year(year(&t, r, "start")?),
        };
        let end = year(&t, r, "end")?;
        if end > w.end_year() {
            return Err(CompileError::new(
                &wr,
                format!(
                    "it ends in {end}, after the world's horizon {}",
                    w.end_year()
                ),
            ));
        }
        if let Start::Year(s) = start {
            if s < w.start_year() || s >= end {
                return Err(CompileError::new(
                    &wr,
                    format!("{s}–{end} is not a span within the world's years"),
                ));
            }
        }
        let regions = selector(&wr, r.get(3), counties)?;
        if start == Start::TextileFrom && !matches!(regions, Selector::Kinds(_)) {
            return Err(CompileError::new(
                &wr,
                "a ramp that starts at textile_from selects by kind",
            ));
        }
        let param = Param::of(r.get(4)).ok_or_else(|| {
            CompileError::new(&wr, format!("`{}` is not a param column", r.get(4)))
        })?;
        let mode = match r.get(5) {
            "scale" => Mode::Scale,
            "path" => Mode::Path,
            m => {
                return Err(CompileError::new(
                    &wr,
                    format!("mode `{m}` is not scale or path"),
                ))
            }
        };
        let from = number(&t, r, "from")?;
        let to = number(&t, r, "to")?;
        if !(from > 0.0 && to > 0.0) {
            return Err(CompileError::new(&wr, "from and to are positive"));
        }
        if mode == Mode::Scale && from != 1.0 {
            return Err(CompileError::new(&wr, "a scale ramp's from is 1"));
        }
        let steps_per_year = match r.get(8).parse::<u32>() {
            Ok(n) if (1..=12).contains(&n) && 12 % n == 0 => n,
            _ => {
                return Err(CompileError::new(
                    &wr,
                    "steps_per_year divides 12: a step falls on the first of a month",
                ))
            }
        };
        let weight = weight(&wr, r.get(9))?;
        for (what, s) in [("lever", r.get(10)), ("note", r.get(11))] {
            text(&wr, what, s)?;
        }
        out.push(Ramp {
            key,
            start,
            end,
            regions,
            param,
            mode,
            from,
            to,
            steps_per_year,
            weight,
            lever: r.get(10).to_string(),
            note: r.get(11).to_string(),
            line: r.line,
        });
    }
    Ok(out)
}

const LENS_COLUMNS: [&str; 11] = [
    "key",
    "name",
    "measure",
    "unit",
    "scale",
    "reference",
    "domain_lo",
    "domain_hi",
    "inputs",
    "source",
    "note",
];

fn lenses(file: &str, text_: &str) -> Result<Vec<Lens>, CompileError> {
    let t = Table::parse(file, text_, &LENS_COLUMNS)?;
    let mut keys = BTreeSet::new();
    let mut out = Vec::new();
    for r in &t.rows {
        let key = r.get(0).to_string();
        let wr = format!("{} {key}", at(&t, r));
        check_key(&wr, &key)?;
        if !keys.insert(key.clone()) {
            return Err(CompileError::new(wr, "the key appears twice"));
        }
        for (i, col) in LENS_COLUMNS.iter().enumerate() {
            text(&wr, col, r.get(i))?;
        }
        if r.get(1).is_empty() || r.get(2).is_empty() || r.get(3).is_empty() {
            return Err(CompileError::new(
                &wr,
                "a lens has a name, a measure and a unit",
            ));
        }
        if let Some(p) = key.strip_prefix("param.") {
            if Param::of(p).is_none() {
                return Err(CompileError::new(
                    &wr,
                    format!("`{p}` is not a param column"),
                ));
            }
        }
        // D.3: every lens is a measure that `lens::value` defines, so the GUI computes none.
        let Some(measure) = crate::lens::Measure::of(&key) else {
            return Err(CompileError::new(
                &wr,
                "the key names no measure that crates/worldgen/src/lens.rs defines",
            ));
        };
        if let crate::lens::Measure::Since(_) = measure {
            let base = key.trim_start_matches("since.");
            if !keys.contains(base) {
                return Err(CompileError::new(
                    &wr,
                    format!("a change lens comes after its level's row, `{base}`"),
                ));
            }
        }
        let scale = match r.get(4) {
            "sequential" => Scale::Sequential,
            "sequential-log" => Scale::SequentialLog,
            "diverging" => Scale::Diverging,
            s => {
                return Err(CompileError::new(
                    &wr,
                    format!("scale `{s}` is not sequential, sequential-log or diverging"),
                ))
            }
        };
        let lo = number(&t, r, "domain_lo")?;
        let hi = number(&t, r, "domain_hi")?;
        let ok = lo < hi
            && match scale {
                Scale::Sequential => true,
                Scale::SequentialLog => lo > 0.0,
                // A diverging scale may centre on any reference inside its domain, checked
                // below: 0 for a log difference, 1 for a ratio such as the horse's price over
                // its replacement cost (D2.3).
                Scale::Diverging => true,
            };
        if !ok {
            return Err(CompileError::new(
                &wr,
                format!("the domain [{lo}, {hi}] does not fit a {} scale", r.get(4)),
            ));
        }
        // A reference, when there is one, names its value (`<value> = <what>`), and a
        // diverging scale centres on it (D.3).
        let reference = r.get(5);
        let centre = crate::lens::reference_value(reference);
        if !reference.is_empty() && centre.is_none() {
            return Err(CompileError::new(
                &wr,
                format!("the reference `{reference}` names no value (`<value> = <what>`)"),
            ));
        }
        if scale == Scale::Diverging && !centre.is_some_and(|c| lo < c && c < hi) {
            return Err(CompileError::new(
                &wr,
                "a diverging scale centres on a reference inside its domain",
            ));
        }
        let source = match r.get(9) {
            "report" => LensSource::Report,
            "report+param" => LensSource::ReportParam,
            "param" => LensSource::Param,
            "extractor" => LensSource::Extractor,
            "extractor+param" => LensSource::ExtractorParam,
            "observe" => LensSource::Observe,
            "oracle" => LensSource::Oracle,
            s => {
                return Err(CompileError::new(
                    &wr,
                    format!(
                        "source `{s}` is not report, report+param, param, extractor, \
                         extractor+param, observe or oracle"
                    ),
                ))
            }
        };
        out.push(Lens {
            key,
            name: r.get(1).to_string(),
            measure: r.get(2).to_string(),
            unit: r.get(3).to_string(),
            scale,
            reference: r.get(5).to_string(),
            domain: (lo, hi),
            inputs: r.get(8).to_string(),
            source,
            note: r.get(10).to_string(),
        });
    }
    Ok(out)
}

/// Read and check `lenses.csv` alone. The GUI's map reads the bundled table through
/// [`crate::lens::demo_gb`].
pub fn parse_lenses(text: &str) -> Result<Vec<Lens>, CompileError> {
    lenses("lenses.csv", text)
}

/// Read and check a lens table named `file` (`lenses-v2a1.csv`, D2.3), as [`parse_lenses`].
pub fn parse_lens_table(file: &str, text: &str) -> Result<Vec<Lens>, CompileError> {
    lenses(file, text)
}

/// Read and check every table against the atlas.
pub fn parse(tables: &Tables, atlas: &Atlas) -> Result<Parsed, CompileError> {
    let world = world(&tables.world)?;
    counties(&tables.counties, atlas)?;
    let counties = regions(&tables.regions, atlas, &world)?;
    let ramps = history(&tables.history, &world, &counties)?;
    let lenses = lenses("lenses.csv", &tables.lenses)?;
    Ok(Parsed {
        world,
        counties,
        ramps,
        lenses,
    })
}
