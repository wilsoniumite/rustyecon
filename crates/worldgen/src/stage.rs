//! The demo's second pass: stage v2a.1 of the goods chain on every county (docs/demo/WORLD-V2.md;
//! D2.2, 2026-09-30; decisions 320–330, 339).
//!
//! `rustyecon worldgen worlds/demo-gb --stage v2a1` reads v1's county tables and the stage's
//! three files ([`StageTables`]): `machine_types.csv` (the horse: δ, κ, ω and J_b), the stage's
//! settings `stage-v2a1.csv` in `world.csv`'s format (the tape's name, its basis marker, C2g's
//! dials and the stock dials, the task assignment, the lens table), and `lenses-v2a1.csv`.
//! [`compile_stage`] then compiles each county under GOODS-CHAIN's rule A:
//!
//! - **The roles** (§2.2): P2.0's good desk with ex-post assignment, the capacity desk (M3,
//!   wet), the maker (M2) with its reservation, P2.1's type desk for fodder and its basket
//!   households; six actors a county, keyed as P2.2a's tapes are.
//! - **The history** (§3; decision 323): v1's steps, stepped exactly as v1's, each mapped to the
//!   coefficients whose fold is its param: a b step sets fodder's land ω·b and the pasture
//!   (1 − ω)·b·κ/δ together, λ the head's labour λ·κ/δ, a its own horse-days a·κ/δ; every
//!   other param as v1. v1's a, λ and b are not registered (decision 328).
//! - **The checks** (§9.2): v1's, and at genesis and after every step date unit 1g's chain
//!   ([`crate::chain`]), `Interior` and funded, meeting v1's 1a point within [`COLLAPSE`] (R1b),
//!   its own p_K/r, p_f/r, heads and fodder moving at most `max_step` at a date and
//!   [`MAX_YEAR`] over a trailing year (decision 330). An unfunded date does not compile: the
//!   error names the county and the date (decision 326).
//! - **Genesis** (§5; decision 325): each county at 1g's point under the horses genesis rule
//!   (HORSES-SPEC §5.3), every price and coin divided by p/r so the good costs 1 coin.
//!
//! The oracle is solved here, outside any `Sim`; no agent reads it (R13).

use crate::atlas::Atlas;
use crate::chain::{self, ChainParams, ChainPoint, MachineType};
use crate::compile::{clock, plan_county, Compiled, Plan, Summary, MAX_YEAR};
use crate::csv::Table;
use crate::history::{self, Month, Step};
use crate::tables::{self, Dial, Instance, Lens, Param, Parsed, Ramp, Tables, World};
use crate::CompileError;
use rustyecon_core::{num, Clock, FlowPerYear, RatePerYear, Years};
use std::collections::BTreeMap;

/// The texts of a stage's three files, as the cli reads them from the world's directory with
/// `--stage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageTables {
    /// The stage's key, as `--stage` names it (`v2a1`).
    pub key: String,
    /// `machine_types.csv`.
    pub machine_types: String,
    /// `stage-<key>.csv`.
    pub stage: String,
    /// The stage's lens table, `lenses-<key>.csv` (D2.3).
    pub lenses: String,
}

impl StageTables {
    /// The file names of stage `key`, in the order of the fields.
    pub fn files(key: &str) -> [String; 3] {
        [
            "machine_types.csv".to_string(),
            format!("stage-{key}.csv"),
            StageTables::lens_file(key),
        ]
    }

    /// The name of stage `key`'s lens table.
    pub fn lens_file(key: &str) -> String {
        format!("lenses-{key}.csv")
    }
}

/// The stages this compiler knows: v2a.1, GOODS-CHAIN's first (decision 320).
pub const STAGES: [&str; 1] = ["v2a1"];

/// The stage's dials, with their units, in the order the tape lists them: P2.2a's C2g for the
/// six markets and four desks, and the stock rules' (HORSES-SPEC §5, §5.2; IDLE-SPEC).
pub const STAGE_DIALS: [(&str, &str); 24] = [
    ("ledger.rel_flow", "Dimensionless"),
    ("ledger.rel_stock", "Dimensionless"),
    ("rate.labour", "RatePerYear"),
    ("rate.land", "RatePerYear"),
    ("rate.fodder", "RatePerYear"),
    ("rate.horse", "RatePerYear"),
    ("rate.traction", "RatePerYear"),
    ("rate.good", "RatePerYear"),
    ("adjust.technique.good", "RatePerYear"),
    ("buffer.desk.good.cash", "RatePerYear"),
    ("buffer.desk.capacity.cash", "RatePerYear"),
    ("buffer.desk.maker.cash", "RatePerYear"),
    ("buffer.desk.fodder.cash", "RatePerYear"),
    ("spend.workers", "RatePerYear"),
    ("spend.provider", "RatePerYear"),
    ("tilt.desk.good", "Dimensionless"),
    ("tilt.desk.capacity", "Dimensionless"),
    ("tilt.desk.maker", "Dimensionless"),
    ("tilt.desk.fodder", "Dimensionless"),
    ("price.ema_tc", "Years"),
    ("adjust.invest.capacity", "RatePerYear"),
    ("adjust.invest.maker", "RatePerYear"),
    ("cover.maker", "Years"),
    ("reserve.maker", "Dimensionless"),
];

/// C2g as P2.2a registered it (D-G13; HORSES-SPEC §5; `probe::horses::setup::dials` on F5, held
/// by `c2g_is_the_probes`), with the stock dials and the maker's reservation ψ 0.25 (IDLE-SPEC;
/// decision 253). The capacity desk's s_K is 2δ of the machine type and is checked against it.
/// A stage at other dials needs its own probe run first (decision 324).
pub const C2G: [(&str, f64); 21] = [
    ("rate.labour", 5.2),
    ("rate.land", 0.1625),
    ("rate.fodder", 5.2),
    ("rate.horse", 5.2),
    ("rate.traction", 5.2),
    ("rate.good", 2.6),
    ("adjust.technique.good", 2.6),
    ("buffer.desk.good.cash", 5.2),
    ("buffer.desk.capacity.cash", 5.2),
    ("buffer.desk.maker.cash", 5.2),
    ("buffer.desk.fodder.cash", 5.2),
    ("spend.workers", 13.0),
    ("spend.provider", 13.0),
    ("tilt.desk.good", 0.0),
    ("tilt.desk.capacity", 0.0),
    ("tilt.desk.maker", 0.0),
    ("tilt.desk.fodder", 0.0),
    ("price.ema_tc", 0.5),
    ("adjust.invest.maker", 0.0),
    ("cover.maker", 4.0 / 52.0),
    ("reserve.maker", 0.25),
];

/// The largest relative difference 1g's point may have from v1's 1a point at a county date on
/// x\*, v, P_s, Y, N_a and p (R1b): the collapse holds to 8e-16 on the history (WORLD-V2 §4).
pub const COLLAPSE: f64 = 1e-13;

/// A stage, read and checked.
#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    /// Its key (`v2a1`).
    pub key: String,
    /// The tape's name, with the illustrative marker.
    pub name: String,
    /// The basis marker.
    pub basis: String,
    /// The machine type every county's `machine_types` names.
    pub machine: MachineType,
    /// The good desk's task assignment: `ExPost` (D-G6) or `Planned`.
    pub assign: String,
    /// The lens table's file name.
    pub lenses: String,
    /// The lens table, read and checked (D2.3).
    pub lens_rows: Vec<Lens>,
    /// The dials, in [`STAGE_DIALS`] order.
    pub dials: Vec<Dial>,
}

impl Stage {
    /// The value of a dial by key.
    pub fn dial(&self, key: &str) -> Option<f64> {
        self.dials.iter().find(|d| d.key == key).map(|d| d.value)
    }
}

/// A county param as the stage registers it: v1's households and task line, and rule A's
/// coefficients in place of v1's (a, λ, b) (decision 328).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StageParam {
    /// N, a year.
    Workers,
    /// T, a year.
    Land,
    /// h.
    Space,
    /// η.
    Eta,
    /// g0.
    G0,
    /// g1.
    G1,
    /// k.
    K,
    /// χ_max.
    ChiMax,
    /// ω·b, land per unit of fodder.
    FodderLand,
    /// a·κ/δ, a head's own horse-days.
    OwnHours,
    /// λ·κ/δ, a head's labour.
    HorseLabour,
    /// (1 − ω)·b·κ/δ, a head's pasture.
    HorseLand,
}

impl StageParam {
    /// Every stage param, in the order the tape lists a county's.
    pub const ALL: [StageParam; 12] = [
        StageParam::Workers,
        StageParam::Land,
        StageParam::Space,
        StageParam::Eta,
        StageParam::G0,
        StageParam::G1,
        StageParam::K,
        StageParam::ChiMax,
        StageParam::FodderLand,
        StageParam::OwnHours,
        StageParam::HorseLabour,
        StageParam::HorseLand,
    ];

    /// The last part of its tape key, `county.<c>.<column>`.
    pub fn column(self) -> &'static str {
        match self {
            StageParam::Workers => "workers",
            StageParam::Land => "land",
            StageParam::Space => "space",
            StageParam::Eta => "eta",
            StageParam::G0 => "g0",
            StageParam::G1 => "g1",
            StageParam::K => "k",
            StageParam::ChiMax => "chi_max",
            StageParam::FodderLand => "fodder.land",
            StageParam::OwnHours => "horse.own_hours",
            StageParam::HorseLabour => "horse.labour",
            StageParam::HorseLand => "horse.land",
        }
    }

    /// Its tape key after `county.<c>.`, with the machine type's key `h` (`horse`; `mach` on the
    /// flow path): the recipe's coefficients are the machine's.
    pub fn key(self, h: &str) -> String {
        match self {
            StageParam::OwnHours => format!("{h}.own_hours"),
            StageParam::HorseLabour => format!("{h}.labour"),
            StageParam::HorseLand => format!("{h}.land"),
            q => q.column().to_string(),
        }
    }

    /// Its unit: N and T are flows a year, the rest dimensionless (the recipe's coefficients are
    /// per tick at the tape's clock, as P2.2a's tapes register them).
    pub fn unit(self) -> &'static str {
        match self {
            StageParam::Workers | StageParam::Land => "FlowPerYear",
            _ => "Dimensionless",
        }
    }

    /// Its index in [`StageParam::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }

    /// v1's param it carries unchanged, if it is one of them.
    pub fn v1(self) -> Option<Param> {
        Some(match self {
            StageParam::Workers => Param::Workers,
            StageParam::Land => Param::Land,
            StageParam::Space => Param::Space,
            StageParam::Eta => Param::Eta,
            StageParam::G0 => Param::G0,
            StageParam::G1 => Param::G1,
            StageParam::K => Param::K,
            StageParam::ChiMax => Param::ChiMax,
            _ => return None,
        })
    }
}

/// A county's stage params: the twelve values, in [`StageParam::ALL`] order.
pub type StageInstance = [f64; 12];

/// The stage params of a v1 instance under machine type `m` at the clock `c`.
pub fn stage_instance(inst: &Instance, m: &MachineType, c: &Clock) -> StageInstance {
    let v = |p: Param| inst[p.index()];
    let r = chain::rule_a(m, c, v(Param::A), v(Param::Lam), v(Param::B));
    let mut out = [0.0; 12];
    for p in StageParam::ALL {
        out[p.index()] = match p {
            StageParam::FodderLand => r.fodder_land,
            StageParam::OwnHours => r.own_hours,
            StageParam::HorseLabour => r.labour,
            StageParam::HorseLand => r.pasture,
            q => q.v1().map_or(0.0, v),
        };
    }
    out
}

/// What unit 1g's chain is built from at a v1 instance.
pub fn chain_params(inst: &Instance, m: &MachineType, c: &Clock) -> ChainParams {
    let v = |p: Param| inst[p.index()];
    let (kappa, delta) = m.per_tick(c);
    ChainParams {
        workers: v(Param::Workers),
        land: v(Param::Land),
        space: v(Param::Space),
        eta: v(Param::Eta),
        g0: v(Param::G0),
        g1: v(Param::G1),
        k: v(Param::K),
        chi_max: v(Param::ChiMax),
        rule: chain::rule_a(m, c, v(Param::A), v(Param::Lam), v(Param::B)),
        kappa,
        delta,
        fodder: m.has_fodder(),
    }
}

/// One step of a stage param: v1's step, mapped (decision 323).
#[derive(Debug, Clone, PartialEq)]
pub struct StageStep {
    /// The month it falls on.
    pub month: Month,
    /// The param it sets.
    pub param: StageParam,
    /// The value it sets.
    pub value: f64,
    /// v1's step it maps.
    pub from: Param,
    /// v1's value, six significant figures.
    pub v1_value: f64,
    /// The ramps that moved v1's param since its last step, by index into the table.
    pub ramps: Vec<usize>,
}

/// v1's steps of a county mapped to the stage's (decision 323): a b step becomes fodder's land
/// and the pasture together, λ the head's labour, a its own horse-days, the rest themselves. In
/// (month, param) order.
pub fn stage_steps(steps: &[Step], m: &MachineType, c: &Clock) -> Vec<StageStep> {
    let mut out = Vec::with_capacity(steps.len() + steps.len() / 8);
    for s in steps {
        let r = |a: f64, lam: f64, b: f64| chain::rule_a(m, c, a, lam, b);
        let mapped: Vec<(StageParam, f64)> = match s.param {
            Param::B => {
                let x = r(0.0, 0.0, s.value);
                vec![
                    (StageParam::FodderLand, x.fodder_land),
                    (StageParam::HorseLand, x.pasture),
                ]
            }
            Param::Lam => vec![(StageParam::HorseLabour, r(0.0, s.value, 0.0).labour)],
            Param::A => vec![(StageParam::OwnHours, r(s.value, 0.0, 0.0).own_hours)],
            p => {
                let q = StageParam::ALL
                    .into_iter()
                    .find(|q| q.v1() == Some(p))
                    .unwrap_or(StageParam::Workers);
                vec![(q, s.value)]
            }
        };
        for (param, value) in mapped {
            out.push(StageStep {
                month: s.month,
                param,
                value,
                from: s.param,
                v1_value: s.value,
                ramps: s.ramps.clone(),
            });
        }
    }
    out.sort_by_key(|s| (s.month, s.param));
    out
}

fn read_machine_types(text: &str) -> Result<Vec<MachineType>, CompileError> {
    let t = Table::parse(
        "machine_types.csv",
        text,
        &[
            "key",
            "rule",
            "delta",
            "kappa",
            "omega",
            "build_lag",
            "note",
        ],
    )?;
    let mut out: Vec<MachineType> = Vec::new();
    for r in &t.rows {
        let w = format!("machine_types.csv:{}", r.line);
        let key = r.get(0).to_string();
        if key.is_empty() || !key.bytes().all(|b| b.is_ascii_lowercase()) {
            return Err(CompileError::new(&w, format!("`{key}` is not a key (a-z)")));
        }
        if out.iter().any(|m| m.key == key) {
            return Err(CompileError::new(&w, format!("`{key}` appears twice")));
        }
        if r.get(1) != "A" {
            return Err(CompileError::new(
                &w,
                format!(
                    "{key}'s rule is `{}`: this compiler writes GOODS-CHAIN's rule A only; rule B's \
                     loop enters a county only through its own registration (decision 308, O83)",
                    r.get(1)
                ),
            ));
        }
        let num_ = |i: usize, what: &str| -> Result<f64, CompileError> {
            match r.get(i).parse::<f64>() {
                Ok(v) if v.is_finite() => Ok(v),
                _ => Err(CompileError::new(
                    format!("{w} {what}"),
                    format!("`{}` is not a finite number", r.get(i)),
                )),
            }
        };
        let delta = num_(2, "delta")?;
        let kappa = num_(3, "kappa")?;
        let omega = num_(4, "omega")?;
        let build_lag = r.get(5).parse::<u32>().map_err(|_| {
            CompileError::new(format!("{w} build_lag"), "not a whole number of ticks")
        })?;
        if !(delta > 0.0 && delta < 1.0) {
            return Err(CompileError::new(
                format!("{w} delta"),
                format!("δ {delta} a year is not in (0, 1)"),
            ));
        }
        if kappa <= 0.0 {
            return Err(CompileError::new(format!("{w} kappa"), "κ is positive"));
        }
        if !(0.0..1.0).contains(&omega) {
            return Err(CompileError::new(
                format!("{w} omega"),
                format!("ω {omega} is not in [0, 1)"),
            ));
        }
        if build_lag != 1 {
            return Err(CompileError::new(
                format!("{w} build_lag"),
                "J_b is 1 tick: the probe's stocks were run at J_b 1 (HORSES-SPEC §1.2)",
            ));
        }
        let note = r.get(6).to_string();
        if note.chars().any(char::is_control) {
            return Err(CompileError::new(&w, "the note holds a control character"));
        }
        out.push(MachineType {
            key,
            delta,
            kappa,
            omega,
            build_lag,
            note,
        });
    }
    Ok(out)
}

/// Read and check a stage's files: its settings and dials (C2g exactly, s_K = 2δ), its machine
/// type (rule A, δ in (0, 1), κ positive, ω in [0, 1), J_b 1) and its lens table.
pub fn parse_stage(st: &StageTables) -> Result<Stage, CompileError> {
    if !STAGES.contains(&st.key.as_str()) {
        return Err(CompileError::new(
            "--stage",
            format!("no stage `{}`: this compiler knows {STAGES:?}", st.key),
        ));
    }
    let file = format!("stage-{}.csv", st.key);
    let t = Table::parse(&file, &st.stage, &["key", "value", "unit", "note"])?;
    let mut rows: BTreeMap<String, (String, String, String, String)> = BTreeMap::new();
    for r in &t.rows {
        let w = format!("{file}:{}", r.line);
        if r.get(3).chars().any(char::is_control) || r.get(1).chars().any(char::is_control) {
            return Err(CompileError::new(&w, "a control character"));
        }
        if rows
            .insert(
                r.get(0).to_string(),
                (
                    r.get(1).to_string(),
                    r.get(2).to_string(),
                    r.get(3).to_string(),
                    w.clone(),
                ),
            )
            .is_some()
        {
            return Err(CompileError::new(
                w,
                format!("`{}` appears twice", r.get(0)),
            ));
        }
    }
    let mut take = |key: &str, unit: &str| -> Result<(String, String, String), CompileError> {
        let Some((v, u, n, w)) = rows.remove(key) else {
            return Err(CompileError::new(&file, format!("no row `{key}`")));
        };
        if u != unit {
            return Err(CompileError::new(
                format!("{w} {key}"),
                format!("the unit is `{u}`, not `{unit}`"),
            ));
        }
        Ok((v, n, w))
    };
    let (name, _, wn) = take("name", "")?;
    let (basis, _, wb) = take("basis", "")?;
    if !basis.starts_with(tables::ILLUSTRATIVE_BASIS) {
        return Err(CompileError::new(
            wb,
            format!(
                "the basis marker starts with `{}`: this compiler writes illustrative worlds \
                 only (R4, R5)",
                tables::ILLUSTRATIVE_BASIS
            ),
        ));
    }
    if !name.contains(tables::ILLUSTRATIVE_NAME) {
        return Err(CompileError::new(
            wn,
            format!(
                "an illustrative world's name carries the marker {} (R4, R5)",
                tables::ILLUSTRATIVE_NAME
            ),
        ));
    }
    let (machine_key, _, wm) = take("machine_type", "")?;
    let (assign, _, wa) = take("assign", "")?;
    if assign != "ExPost" {
        return Err(CompileError::new(
            wa,
            format!(
                "assign is `{assign}`: the stage's good desk assigns tasks after the fact, ExPost \
                 (D-G6), as P2.2a ran it"
            ),
        ));
    }
    let (lenses, _, wl) = take("lenses", "")?;
    if lenses != StageTables::lens_file(&st.key) {
        return Err(CompileError::new(
            wl,
            format!("the lens table is `{}`", StageTables::lens_file(&st.key)),
        ));
    }
    let types = read_machine_types(&st.machine_types)?;
    let Some(machine) = types.into_iter().find(|m| m.key == machine_key) else {
        return Err(CompileError::new(
            wm,
            format!("machine_types.csv has no `{machine_key}`"),
        ));
    };
    let mut dials = Vec::new();
    for (key, unit) in STAGE_DIALS {
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
        let want = if key == "adjust.invest.capacity" {
            Some(2.0 * machine.delta)
        } else {
            C2G.iter().find(|(k, _)| *k == key).map(|&(_, v)| v)
        };
        if let Some(want) = want {
            if value != want {
                return Err(CompileError::new(
                    w,
                    format!(
                        "{key} is {value}, not {want}: the stage runs at P2.2a's registered C2g \
                         with s_K = 2δ and the maker's reservation ψ 0.25, the one point at \
                         which the probe's battery is evidence (decision 324)"
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
    let lens_rows = tables::parse_lens_table(&lenses, &st.lenses)?;
    Ok(Stage {
        key: st.key.clone(),
        name,
        basis,
        machine,
        assign,
        lenses,
        lens_rows,
        dials,
    })
}

/// The largest |Δ ln| between two chain points over the chain's own prices (p_K/r, p_f/r) and
/// over its own quantities (heads installed, heads made, fodder), which v1's list does not
/// cover (decision 330).
fn chain_distance(a: &ChainPoint, b: &ChainPoint) -> (f64, f64) {
    let l = |x: f64, y: f64| {
        if x.is_finite() && y.is_finite() {
            num::ln(x / y).abs()
        } else {
            0.0
        }
    };
    let prices = [l(a.pk, b.pk), l(a.pf, b.pf)]
        .into_iter()
        .fold(0.0, f64::max);
    let quantities = [l(a.heads(), b.heads()), l(a.made, b.made), l(a.qf, b.qf)]
        .into_iter()
        .fold(0.0, f64::max);
    (prices, quantities)
}

fn keep(slot: &mut (f64, String), v: f64, at: &str) {
    if v > slot.0 {
        *slot = (v, at.to_string());
    }
}

/// Solve the chain at a county date and check it: `Interior`, funded, and meeting v1's 1a
/// point within [`COLLAPSE`] (R1b).
fn checked_point(
    at: &str,
    inst: &Instance,
    stage: &Stage,
    c: &Clock,
    v1: &oracle::Eq1a,
    collapse: &mut (f64, String),
) -> Result<ChainPoint, CompileError> {
    let e = chain::point(&chain_params(inst, &stage.machine, c), c)
        .map_err(|e| CompileError::new(at, e))?;
    if !e.funded || e.provider_baskets <= 0.0 {
        return Err(CompileError::new(
            at,
            "the provider cannot fund one basket per potential worker (T·r ≤ N·P_s) in the \
             chain (unit 1g)",
        ));
    }
    let rel = |a: f64, b: f64| ((a - b) / b).abs();
    let d = [
        rel(e.x_star, v1.x_star),
        rel(e.v, v1.v),
        rel(e.p_s, v1.p_s),
        rel(e.y, v1.y),
        rel(e.n_a, v1.n_a),
        rel(e.p, v1.p),
    ]
    .into_iter()
    .fold(0.0, f64::max);
    if d.is_nan() || d > COLLAPSE {
        return Err(CompileError::new(
            at,
            format!(
                "the chain's point parts from v1's 1a point by {d:e} relative, above {COLLAPSE:e}: \
                 rule A's fold is not the county's (R1b)"
            ),
        ));
    }
    keep(collapse, d, at);
    Ok(e)
}

/// Compile a world's tables and a stage's files against an atlas (decision 327).
pub fn compile_stage(
    tables: &Tables,
    st: &StageTables,
    atlas: &Atlas,
) -> Result<Compiled, CompileError> {
    let stage = parse_stage(st)?;
    compile_stage_parsed(tables, &stage, atlas)
}

/// [`compile_stage`] with the stage already read.
pub fn compile_stage_parsed(
    tables: &Tables,
    stage: &Stage,
    atlas: &Atlas,
) -> Result<Compiled, CompileError> {
    let Parsed {
        world,
        counties,
        ramps,
        ..
    } = tables::parse(tables, atlas)?;
    let lenses = stage.lens_rows.clone();
    let c = clock(&world);
    for county in &counties {
        if county.machine_types != [stage.machine.key.as_str()] {
            return Err(CompileError::new(
                format!("regions.csv:{} {}", county.line, county.key),
                format!(
                    "machine_types is {:?}, not the stage's `{}`",
                    county.machine_types, stage.machine.key
                ),
            ));
        }
    }
    let mut summary = Summary::new(counties.len());
    let mut plans = Vec::with_capacity(counties.len());
    for county in counties {
        let mut plan = plan_county(&world, county, &ramps, &c, &mut summary)?;
        // On the flow path (R1a) the machine lives one tick and the county is v1's: its point is
        // 1a's itself, and there is no chain to solve.
        if stage.machine.is_flow() {
            plans.push(plan);
            continue;
        }
        let key = plan.county.key.clone();
        let at0 = format!("{key} at {}", world.start);
        let p0 = checked_point(
            &at0,
            &plan.county.genesis,
            stage,
            &c,
            &plan.point,
            &mut summary.max_collapse,
        )?;
        let mut chain = vec![p0];
        let mut inst = plan.county.genesis;
        let mut i = 0;
        for (k, (m, e1a)) in plan.points.iter().enumerate() {
            while i < plan.steps.len() && plan.steps[i].month <= *m {
                inst[plan.steps[i].param.index()] = plan.steps[i].value;
                i += 1;
            }
            let at = format!("{key} at {}", history::date(&world, *m));
            let e = checked_point(&at, &inst, stage, &c, e1a, &mut summary.max_collapse)?;
            let (dp, dq) = chain_distance(&e, &chain[k]);
            for (what, d) in [
                ("prices (p_K/r, p_f/r)", dp),
                ("quantities (heads, fodder)", dq),
            ] {
                if d > world.max_step {
                    return Err(CompileError::new(
                        &at,
                        format!(
                            "one date moves the chain's own {what} by {d:.4} in log, above \
                             max_step {}: make the history more gradual (O14; decision 330)",
                            world.max_step
                        ),
                    ));
                }
            }
            // The chain's point in force twelve months before: index 0 is genesis.
            let year_ago = plan.points[..k]
                .iter()
                .rposition(|(pm, _)| *pm <= m - 12)
                .map_or(&chain[0], |j| &chain[j + 1]);
            let (yp, yq) = chain_distance(&e, year_ago);
            for (what, d) in [("prices", yp), ("quantities", yq)] {
                if d > MAX_YEAR {
                    return Err(CompileError::new(
                        &at,
                        format!(
                            "the year to this date moves the chain's own {what} by {d:.4} in log, \
                             above {MAX_YEAR} a year: spread the change over more years (O14; \
                             decision 330)"
                        ),
                    ));
                }
            }
            keep(&mut summary.max_step_chain, dp.max(dq), &at);
            keep(&mut summary.max_year_chain, yp.max(yq), &at);
            chain.push(e);
        }
        plan.chain = chain;
        plans.push(plan);
    }
    let w = World {
        name: stage.name.clone(),
        basis: stage.basis.clone(),
        dials: stage.dials.clone(),
        ..world
    };
    let (tape, events) = if stage.machine.is_flow() {
        write_flow(&w, stage, &plans, &ramps, atlas.digest)
    } else {
        let mut gens = Vec::with_capacity(plans.len());
        for p in &plans {
            gens.push(stage_genesis(&w, stage, &p.county.genesis, &p.chain[0])?);
        }
        write_stage(&w, stage, &plans, &gens, &ramps, atlas.digest)
    };
    summary.events = events;
    Ok(Compiled {
        tape,
        world: w,
        counties: plans,
        ramps,
        lenses,
        summary,
        atlas_digest: atlas.digest,
    })
}

/// The stage's markets at every node, in the order its genesis lists their prices: labour, land,
/// fodder, the horse, its hours (horse-days) and the good (HORSES-SPEC §7.1's order).
pub const MARKETS: [&str; 6] = ["labour", "land", "fodder", "horse", "traction", "good"];

/// The six roles' actor suffixes, in the order genesis lists their coin (P2.2a's desks, then
/// the households).
pub const ROLES: [&str; 6] = [
    "desk.good",
    "desk.capacity",
    "desk.maker",
    "desk.fodder",
    "provider",
    "workers",
];

/// A county's genesis under the stage (WORLD-V2 §5): the horses genesis rule (HORSES-SPEC
/// §5.3) at 1g's point, every price and coin divided by p/r so the good costs 1 coin.
#[derive(Debug, Clone, PartialEq)]
pub struct StageGenesis {
    /// The prices in [`MARKETS`] order, the good's 1.
    pub prices: [f64; 6],
    /// The good desk's human share 1 − x\*.
    pub share: f64,
    /// Each actor's stationary coin, in [`ROLES`] order.
    pub coin: [f64; 6],
    /// The good desk's good, one tick's output.
    pub good: f64,
    /// The capacity desk's heads, the tasks' Y·J(x\*)/κ.
    pub heads: f64,
    /// Their horse-days, κ times the heads.
    pub hours: f64,
    /// The maker's serving stock after wear, its record `own`.
    pub own: f64,
    /// The maker's finished heads: q_b and its cover of sales.
    pub finished: f64,
    /// The fodder desk's fodder, one tick's output.
    pub fodder: f64,
    /// The maker's cost of a head at the point, as its rule sums it, at r = 1.
    pub maker_cost: f64,
    /// The maker's cover of finished heads at the point.
    pub band: f64,
}

/// A county's genesis at its chain point `e` under instance `inst`, as
/// `probe::horses::setup::genesis` makes a wet instance's at r = 1, then every price and coin
/// divided by p/r (the roles are homogeneous of degree zero in prices and coin).
pub fn stage_genesis(
    w: &World,
    stage: &Stage,
    inst: &Instance,
    e: &ChainPoint,
) -> Result<StageGenesis, CompileError> {
    let c = clock(w);
    let share = |v: f64| c.share(RatePerYear(v));
    let dial = |k: &str| stage.dial(k).unwrap_or(0.0);
    let v = |p: Param| inst[p.index()];
    let m = &stage.machine;
    let (kappa, delta) = m.per_tick(&c);
    let rule = chain::rule_a(m, &c, v(Param::A), v(Param::Lam), v(Param::B));
    let n = c.flow(FlowPerYear(v(Param::Workers)));
    let t = c.flow(FlowPerYear(v(Param::Land)));
    let (wage, r) = (e.v, 1.0);
    // P_s as the households sum it: (0.0 + 1·p) + h·r.
    let mut ps = 0.0;
    ps += 1.0 * e.p;
    ps += v(Param::Space) * r;
    let tau = n * ps;
    let hours = n * (num::ln1p(wage / ps) / v(Param::ChiMax)).min(1.0);
    // The maker's cost of a head, as its rule sums it: its goods (a·run_g + build_g) first,
    // then labour (a·run_lab + build_lab), then land.
    let a = rule.own_hours;
    let mut cm = 0.0;
    if m.has_fodder() {
        cm += (a * 1.0 + 0.0) * e.pf;
    }
    cm += (a * 0.0 + rule.labour) * wage;
    cm += rule.pasture * r;
    let cover = c
        .ticks(Years(dial("cover.maker")))
        .map_err(|x| CompileError::new("cover.maker", x.to_string()))?;
    let band = f64::from(cover) * e.made * cm / e.pk;
    let own = (1.0 - delta) * e.serving;
    let finished = e.made + band;
    let buffer = |d: &str| share(dial(&format!("buffer.desk.{d}.cash")));
    // The fodder desk's cost, as the type desk sums it: ((0.0 + λ_f·w) + b_f·r).
    let mut cf = 0.0;
    cf += 0.0 * wage;
    cf += rule.fodder_land * r;
    let stationary = [
        e.p * e.good / buffer("good"),
        e.ph * kappa * e.capacity / buffer("capacity"),
        cm * e.made / buffer("maker"),
        cf * e.qf / buffer("fodder"),
        tau + (r * t - tau) / share(dial("spend.provider")),
        (tau + wage * hours) / share(dial("spend.workers")),
    ];
    let at_r = [wage, r, e.pf, e.pk, e.ph, e.p];
    Ok(StageGenesis {
        prices: at_r.map(|x| x / e.p),
        share: e.one_minus_x,
        coin: stationary.map(|x| x / e.p),
        good: e.good,
        heads: e.capacity,
        hours: kappa * e.capacity,
        own,
        finished,
        fodder: e.qf,
        maker_cost: cm,
        band,
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

/// The tape of a compiled stage. Every county's genesis was computed at the check, so it cannot
/// fail here.
fn write_stage(
    w: &World,
    stage: &Stage,
    plans: &[Plan],
    gens: &[StageGenesis],
    ramps: &[Ramp],
    atlas: u64,
) -> (String, usize) {
    let c = clock(w);
    let m = &stage.machine;
    let h = m.good();
    let mark = &w.basis;
    let assumed = |why: &str| format!("Assumed({})", quote(&format!("{mark}: {why}")));
    let steps: Vec<Vec<StageStep>> = plans.iter().map(|p| stage_steps(&p.steps, m, &c)).collect();
    let events: usize = steps.iter().map(Vec::len).sum();
    let mut dates: Vec<Month> = steps.iter().flatten().map(|s| s.month).collect();
    dates.sort_unstable();
    dates.dedup();
    let (kappa, delta) = m.per_tick(&c);
    let mut o = String::new();
    let mut line = |s: &str| {
        o.push_str(s);
        o.push('\n');
    };
    for l in [
        format!(
            "// The demo world's second pass, `{}` (docs/demo/WORLD-V2.md): {} historic counties",
            w.name,
            plans.len()
        ),
        "// of the United Kingdom, each its own node running GOODS-CHAIN's stage v2a.1 under rule A:".into(),
        "// land grows fodder, a maker breeds horses, a capacity desk holds the herd and hires out".into(),
        "// horse-days, and the good desk buys horse-days and labour (P2.2a's roles, HORSES-RULES).".into(),
        "//".into(),
        format!(
            "// Generated: do not edit. `rustyecon worldgen worlds/demo-gb --stage {} --out",
            stage.key
        ),
        format!(
            "// tapes/demo-gb-v2.ron` writes it from worlds/demo-gb/*.csv (stage-{}.csv and",
            stage.key
        ),
        format!(
            "// machine_types.csv among them) and the atlas data/atlas/gb.atlas.ron ({atlas:016x}),"
        ),
        "// and the test `demo_v2_tape_is_its_compilers_output` checks this file against the compiler.".into(),
        "//".into(),
        format!(
            "// Illustrative: every entry's basis is Assumed(\"{mark}: ...\") and the name"
        ),
        "// carries [illustrative], so nothing from this tape may be scored or cited (R4, R5).".into(),
        "//".into(),
        format!(
            "// The horse (machine_types.csv): delta = {} a year ({} a tick), kappa = {} horse-days a",
            f(m.delta),
            f(delta),
            f(m.kappa)
        ),
        format!(
            "// head a year ({} a tick), omega = {}, J_b = {} tick. Each county's flow machine (a, lam, b)",
            f(kappa),
            f(m.omega),
            m.build_lag
        ),
        "// becomes fodder's land omega*b and a head's own horse-days a*kappa/delta, labour".into(),
        "// lam*kappa/delta and pasture (1 - omega)*b*kappa/delta, per tick (crates/worldgen/src/chain.rs).".into(),
        "//".into(),
        "// Genesis puts every county at its own oracle point (WORLD-V2 §5): unit 1g's ChainEconomy".into(),
        "// (crates/oracle; the horse's hours at J = 2, decision 232) at the county's per-tick instance,".into(),
        "// under the horses genesis rule (HORSES-SPEC §5.3), every price and coin divided by p/r so".into(),
        "// that the good costs 1 coin. No agent reads the oracle at run time (R13).".into(),
        "//".into(),
        format!(
            "// The history is v1's, mapped (WORLD-V2 §3): {events} dated SetParam steps on {} dates. A b",
            dates.len()
        ),
        "// step sets fodder.land and horse.land together, lam horse.labour, a horse.own_hours, and the".into(),
        "// rest themselves; each copies a schedule param county.<c>.<param>.<YYYY-MM> whose basis names".into(),
        "// the ramps of worlds/demo-gb/history.csv that moved it.".into(),
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
        format!(
            "        // The ledger's tolerances and the dials, shared by every county (stage-{}.csv).",
            stage.key
        ),
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
    line("        // Structure: fodder, horse-days and the good last one tick; the horse's hours, wear");
    line("        // and running recipe, fodder's recipe but its land, and the basket's good.");
    let rule = format!("GOODS-CHAIN §5 rule A (machine_types.csv: {})", m.note);
    for (key, value, unit, why) in [
        (
            "life.one_tick".to_string(),
            1.0 / f64::from(w.ticks_per_year),
            "Years",
            "one tick: J = 1 (PROBE-SPEC §1.2), as P2.2a's tapes".to_string(),
        ),
        (format!("{h}.kappa"), m.kappa, "FlowPerYear", rule.clone()),
        (
            format!("{h}.delta"),
            m.delta,
            "FractionPerYear",
            rule.clone(),
        ),
        (
            format!("{h}.run.fodder"),
            1.0,
            "Dimensionless",
            rule.clone(),
        ),
        (
            format!("{h}.run.labour"),
            0.0,
            "Dimensionless",
            rule.clone(),
        ),
        ("fodder.own".to_string(), 0.0, "Dimensionless", rule.clone()),
        (
            "fodder.labour".to_string(),
            0.0,
            "Dimensionless",
            rule.clone(),
        ),
        (
            "good.weight".to_string(),
            1.0,
            "Dimensionless",
            "the basket's good, one unit (HORSES-SPEC §2.9)".to_string(),
        ),
    ] {
        line(&param(&key, value, unit, &assumed(&why)));
    }
    line("        // Each county's instance at the start: v1's households and task line (regions.csv),");
    line("        // and rule A's coefficients from its (a, lam, b), per tick.");
    for p in plans {
        let cty = &p.county;
        let s = stage_instance(&cty.genesis, m, &c);
        let v = |q: Param| cty.genesis[q.index()];
        for q in StageParam::ALL {
            let why = match q.v1() {
                Some(_) => format!("{}: {}", cty.name, cty.why),
                None => format!(
                    "{}: rule A on (a, lam, b) = ({}, {}, {})",
                    cty.name,
                    f(v(Param::A)),
                    f(v(Param::Lam)),
                    f(v(Param::B))
                ),
            };
            line(&param(
                &format!("{}.{}", cty.key, q.key(h)),
                s[q.index()],
                q.unit(),
                &assumed(&why),
            ));
        }
    }
    line("        // The history's steps: schedule params, each copied by the event of its key.");
    for (p, st) in plans.iter().zip(&steps) {
        let cty = &p.county;
        let mut sorted: Vec<&StageStep> = st.iter().collect();
        sorted.sort_by_key(|s| (s.param, s.month));
        for s in sorted {
            let names: Vec<&str> = s.ramps.iter().map(|&i| ramps[i].key.as_str()).collect();
            let stamp = history::stamp(w, s.month);
            let mapped = if s.param.v1().is_some() {
                String::new()
            } else {
                format!(" (v1's {} {}, by rule A)", s.from.column(), f(s.v1_value))
            };
            let why = format!(
                "{}'s {} from {stamp}{mapped}: {}",
                cty.name,
                s.param.key(h),
                names.join("; ")
            );
            line(&param(
                &format!("{}.{}.{stamp}", cty.key, s.param.key(h)),
                s.value,
                s.param.unit(),
                &assumed(&why),
            ));
        }
    }
    line("    ],");
    line("    goods: [");
    line("        (key: \"coin\", life: Indefinite, price_rate: None),");
    line("        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),");
    line("        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),");
    line("        (key: \"fodder\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.fodder\")),");
    line(&format!(
        "        (key: \"{h}\", life: Indefinite, price_rate: Some(\"rate.{h}\")),"
    ));
    line("        (key: \"traction\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.traction\")),");
    line(
        "        (key: \"good\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.good\")),",
    );
    line("    ],");
    line("    nodes: [");
    for p in plans {
        line(&format!(
            "        (key: {}, currency: \"coin\"),",
            quote(&p.county.key)
        ));
    }
    line("    ],");
    line("    channels: [],");
    line("    classes: [\"good_desks\", \"capacity_desks\", \"maker_desks\", \"fodder_desks\", \"owners\", \"workers\"],");
    line("    actors: [");
    let role = assumed(
        "P2.2a's roles: the probe's rules with the horse held as a stock (docs/probe/HORSES-RULES.md), the maker's reservation (IDLE-SPEC)",
    );
    let scale = |d: &str| {
        format!(
            "scale: Cash((turnover: \"buffer.desk.{d}.cash\", tilt: \"tilt.desk.{d}\", payout: None))"
        )
    };
    let running = format!(
        "running: (goods: [(good: \"fodder\", coef: \"{h}.run.fodder\")], labour: \"{h}.run.labour\")"
    );
    for (p, g) in plans.iter().zip(gens) {
        let cty = &p.county;
        let e = &p.chain[0];
        let k = &cty.key;
        line(&format!(
            "        // {} ({k}): x* = {}, w/r = {}, p_f/r = {}, p_K/r = {}, p_h/r = {}, p/r = {},",
            cty.name,
            f(e.x_star),
            f(e.v),
            f(e.pf),
            f(e.pk),
            f(e.ph),
            f(e.p)
        ));
        line(&format!(
            "        //   Y = {}, N_a = {}, heads {} (the tasks') and {} (the maker's), q_b = {}, fodder {}.",
            f(e.y),
            f(e.n_a),
            f(e.capacity),
            f(e.serving),
            f(e.made),
            f(e.qf)
        ));
        line(&format!(
            "        (key: \"{k}.desk.good\", kind: Desk, class: \"good_desks\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: GoodDesk((output: \"good\", labour: \"labour\", mach: \"traction\", \
             schedule: (eta: \"{k}.eta\", g0: \"{k}.g0\", g1: \"{k}.g1\", k: \"{k}.k\"), \
             technique: (adjust: \"adjust.technique.good\", share: {}), assign: {}, {}))),",
            f(g.share),
            stage.assign,
            scale("good")
        ));
        line(&format!(
            "        (key: \"{k}.desk.capacity\", kind: Desk, class: \"capacity_desks\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: CapacityDesk((stock: \"{h}\", hours: \"traction\", labour: \"labour\", \
             kappa: \"{h}.kappa\", {running}, delta: \"{h}.delta\", \
             adjust: \"adjust.invest.capacity\", order: Target, {}))),",
            scale("capacity")
        ));
        line(&format!(
            "        (key: \"{k}.desk.maker\", kind: Desk, class: \"maker_desks\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: Maker((output: \"{h}\", labour: \"labour\", land: \"land\", \
             own_hours: \"{k}.{h}.own_hours\", kappa: \"{h}.kappa\", {running}, \
             build: (goods: [], labour: \"{k}.{h}.labour\", land: \"{k}.{h}.land\"), \
             delta: \"{h}.delta\", adjust: \"adjust.invest.maker\", cover: Some(\"cover.maker\"), \
             reserve: Some(\"reserve.maker\"), own: {}, {}))),",
            f(g.own),
            scale("maker")
        ));
        line(&format!(
            "        (key: \"{k}.desk.fodder\", kind: Desk, class: \"fodder_desks\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: TypeDesk((output: \"fodder\", labour: \"labour\", land: \"land\", \
             recipe: (own: \"fodder.own\", inputs: [], labour: \"fodder.labour\", land: \"{k}.fodder.land\"), \
             {}))),",
            scale("fodder")
        ));
        let basket = format!(
            "basket: [(good: \"good\", weight: \"good.weight\"), (good: \"land\", weight: \"{k}.space\")]"
        );
        line(&format!(
            "        (key: \"{k}.provider\", kind: Pop, class: \"owners\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: BasketProvider((land: \"land\", endowment: \"{k}.land\", \
             transfer: (to: \"{k}.workers\", heads: \"{k}.workers\"), {basket}, \
             spend: \"spend.provider\"))),"
        ));
        line(&format!(
            "        (key: \"{k}.workers\", kind: Pop, class: \"workers\", home: \"{k}\", basis: {role},"
        ));
        line(&format!(
            "         spec: BasketWorkers((labour: \"labour\", heads: \"{k}.workers\", \
             chi_max: \"{k}.chi_max\", {basket}, spend: \"spend.workers\"))),"
        ));
    }
    line("    ],");
    line("    genesis: (");
    line(&format!(
        "        basis: {},",
        assumed(
            "each county at its own oracle point, unit 1g's ChainEconomy at rho = 0 (crates/oracle; \
             docs/demo/WORLD-V2.md §5), under the horses genesis rule (HORSES-SPEC §5.3), prices \
             and coins divided by p/r so that the good costs 1 coin; written by rustyecon worldgen"
        )
    ));
    line("        prices: [");
    for (p, g) in plans.iter().zip(gens) {
        let k = &p.county.key;
        let parts: Vec<String> = MARKETS
            .iter()
            .zip(g.prices)
            .map(|(mk, x)| {
                let good = if *mk == "horse" { h } else { mk };
                format!("(node: \"{k}\", good: \"{good}\", price: {})", f(x))
            })
            .collect();
        line(&format!("            {},", parts.join(", ")));
    }
    line("        ],");
    line("        holdings: [");
    for (p, g) in plans.iter().zip(gens) {
        let k = &p.county.key;
        line(&format!(
            "            (holder: \"{k}.desk.good\", goods: [(\"coin\", {}), (\"good\", {})]), \
             (holder: \"{k}.desk.capacity\", goods: [(\"coin\", {}), (\"{h}\", {}), (\"traction\", {})]),",
            f(g.coin[0]),
            f(g.good),
            f(g.coin[1]),
            f(g.heads),
            f(g.hours)
        ));
        line(&format!(
            "            (holder: \"{k}.desk.maker\", goods: [(\"coin\", {}), (\"{h}\", {})]), \
             (holder: \"{k}.desk.fodder\", goods: [(\"coin\", {}), (\"fodder\", {})]),",
            f(g.coin[2]),
            f(g.own + g.finished),
            f(g.coin[3]),
            f(g.fodder)
        ));
        line(&format!(
            "            (holder: \"{k}.provider\", goods: [(\"coin\", {})]), \
             (holder: \"{k}.workers\", goods: [(\"coin\", {})]),",
            f(g.coin[4]),
            f(g.coin[5])
        ));
    }
    line("        ],");
    line("    ),");
    line("    events: [");
    let event_basis = assumed("history step");
    let mut all: Vec<(Month, &str, StageParam)> = plans
        .iter()
        .zip(&steps)
        .flat_map(|(p, st)| {
            st.iter()
                .map(move |s| (s.month, p.county.key.as_str(), s.param))
        })
        .collect();
    all.sort_unstable();
    let mut last = None;
    for (mo, k, q) in all {
        if last != Some(mo) {
            line(&format!("        // {}", history::date(w, mo)));
            last = Some(mo);
        }
        let key = format!("{k}.{}.{}", q.key(h), history::stamp(w, mo));
        line(&format!(
            "        (key: \"{key}\", at: \"{}\", basis: {event_basis}, act: SetParam(param: \"{k}.{}\", to: \"{key}\")),",
            history::date(w, mo),
            q.key(h)
        ));
    }
    line("    ],");
    line("    recurring: [],");
    line(")");
    (o, events)
}

/// The tape of a stage on its flow path (R1a; WORLD-V2 §9.3): the machine lives one tick (δ =
/// 1), no fodder (ω 0), so the owner desk runs the good desk's code and the maker the type
/// desk's, on the probe's C2 with planned assignment; genesis is v1's, the probe's rule at 1a.
/// Only `v2_flow_path_is_v1` reaches it, through [`compile_stage_parsed`] with a stage built
/// in code: `parse_stage` refuses δ = 1.
fn write_flow(
    w: &World,
    stage: &Stage,
    plans: &[Plan],
    ramps: &[Ramp],
    atlas: u64,
) -> (String, usize) {
    let c = clock(w);
    let m = &stage.machine;
    let h = m.good();
    let mark = &w.basis;
    let assumed = |why: &str| format!("Assumed({})", quote(&format!("{mark}: {why}")));
    let steps: Vec<Vec<StageStep>> = plans.iter().map(|p| stage_steps(&p.steps, m, &c)).collect();
    // No fodder on the flow path: a b step sets the machine's land alone, one event.
    let events = steps
        .iter()
        .flatten()
        .filter(|s| s.param != StageParam::FodderLand)
        .count();
    let mut o = String::new();
    let mut line = |s: &str| {
        o.push_str(s);
        o.push('\n');
    };
    for l in [
        format!(
            "// The demo world's second pass on its flow path, `{}`: {} counties, the machine one tick",
            w.name,
            plans.len()
        ),
        format!(
            "// (R1a; docs/demo/WORLD-V2.md §9.3), on the atlas {atlas:016x}. Written only by the \
             test v2_flow_path_is_v1."
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
    let rule = assumed("the flow path (R1a)");
    for (key, value, unit) in [
        (
            "life.one_tick".to_string(),
            1.0 / f64::from(w.ticks_per_year),
            "Years",
        ),
        (format!("{h}.kappa"), m.kappa, "FlowPerYear"),
        (format!("{h}.delta"), m.delta, "FractionPerYear"),
        (format!("{h}.run.labour"), 0.0, "Dimensionless"),
        ("good.weight".to_string(), 1.0, "Dimensionless"),
    ] {
        line(&param(&key, value, unit, &rule));
    }
    for p in plans {
        let cty = &p.county;
        let s = stage_instance(&cty.genesis, m, &c);
        for q in StageParam::ALL {
            if q == StageParam::FodderLand {
                continue;
            }
            line(&param(
                &format!("{}.{}", cty.key, q.key(h)),
                s[q.index()],
                q.unit(),
                &rule,
            ));
        }
    }
    for (p, st) in plans.iter().zip(&steps) {
        for s in st.iter().filter(|s| s.param != StageParam::FodderLand) {
            let names: Vec<&str> = s.ramps.iter().map(|&i| ramps[i].key.as_str()).collect();
            line(&param(
                &format!(
                    "{}.{}.{}",
                    p.county.key,
                    s.param.key(h),
                    history::stamp(w, s.month)
                ),
                s.value,
                s.param.unit(),
                &assumed(&names.join("; ")),
            ));
        }
    }
    line("    ],");
    line("    goods: [");
    line("        (key: \"coin\", life: Indefinite, price_rate: None),");
    line("        (key: \"labour\", life: Instant, price_rate: Some(\"rate.labour\")),");
    line("        (key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),");
    line(&format!(
        "        (key: \"{h}\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.{h}\")),"
    ));
    line(
        "        (key: \"good\", life: Years(\"life.one_tick\"), price_rate: Some(\"rate.good\")),",
    );
    line("    ],");
    line("    nodes: [");
    for p in plans {
        line(&format!(
            "        (key: {}, currency: \"coin\"),",
            quote(&p.county.key)
        ));
    }
    line("    ],");
    line("    channels: [],");
    line(&format!(
        "    classes: [\"good_desks\", \"{h}_desks\", \"owners\", \"workers\"],"
    ));
    line("    actors: [");
    let scale = |d: &str| {
        format!(
            "scale: Cash((turnover: \"buffer.desk.{d}.cash\", tilt: \"tilt.desk.{d}\", payout: None))"
        )
    };
    let mut gens = Vec::with_capacity(plans.len());
    for p in plans {
        let k = &p.county.key;
        let g = crate::compile::genesis(w, &p.county.genesis, &p.point);
        line(&format!(
            "        (key: \"{k}.desk.good\", kind: Desk, class: \"good_desks\", home: \"{k}\", basis: {rule},"
        ));
        line(&format!(
            "         spec: OwnerDesk((output: \"good\", labour: \"labour\", stock: \"{h}\", \
             schedule: (eta: \"{k}.eta\", g0: \"{k}.g0\", g1: \"{k}.g1\", k: \"{k}.k\"), \
             technique: (adjust: \"adjust.technique.good\", share: {}), assign: {}, \
             kappa: \"{h}.kappa\", running: [], delta: \"{h}.delta\", \
             adjust: \"adjust.invest.good\", {}))),",
            f(p.point.one_minus_x_star),
            stage.assign,
            scale("good")
        ));
        line(&format!(
            "        (key: \"{k}.desk.{h}\", kind: Desk, class: \"{h}_desks\", home: \"{k}\", basis: {rule},"
        ));
        line(&format!(
            "         spec: Maker((output: \"{h}\", labour: \"labour\", land: \"land\", \
             own_hours: \"{k}.{h}.own_hours\", kappa: \"{h}.kappa\", \
             running: (goods: [], labour: \"{h}.run.labour\"), \
             build: (goods: [], labour: \"{k}.{h}.labour\", land: \"{k}.{h}.land\"), \
             delta: \"{h}.delta\", adjust: \"adjust.invest.{h}\", cover: None, own: 0.0, {}))),",
            scale(h)
        ));
        let basket = format!(
            "basket: [(good: \"good\", weight: \"good.weight\"), (good: \"land\", weight: \"{k}.space\")]"
        );
        line(&format!(
            "        (key: \"{k}.provider\", kind: Pop, class: \"owners\", home: \"{k}\", basis: {rule},"
        ));
        line(&format!(
            "         spec: BasketProvider((land: \"land\", endowment: \"{k}.land\", \
             transfer: (to: \"{k}.workers\", heads: \"{k}.workers\"), {basket}, \
             spend: \"spend.provider\"))),"
        ));
        line(&format!(
            "        (key: \"{k}.workers\", kind: Pop, class: \"workers\", home: \"{k}\", basis: {rule},"
        ));
        line(&format!(
            "         spec: BasketWorkers((labour: \"labour\", heads: \"{k}.workers\", \
             chi_max: \"{k}.chi_max\", {basket}, spend: \"spend.workers\"))),"
        ));
        gens.push(g);
    }
    line("    ],");
    line("    genesis: (");
    line(&format!("        basis: {rule},"));
    line("        prices: [");
    for (p, g) in plans.iter().zip(&gens) {
        let k = &p.county.key;
        let [pw, pr, ppm, pp] = g.prices;
        line(&format!(
            "            (node: \"{k}\", good: \"good\", price: {}), (node: \"{k}\", good: \"labour\", price: {}), \
             (node: \"{k}\", good: \"land\", price: {}), (node: \"{k}\", good: \"{h}\", price: {}),",
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
             (holder: \"{k}.desk.{h}\", goods: [(\"coin\", {}), (\"{h}\", {})]),",
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
    let mut all: Vec<(Month, &str, StageParam)> = plans
        .iter()
        .zip(&steps)
        .flat_map(|(p, st)| {
            st.iter()
                .filter(|s| s.param != StageParam::FodderLand)
                .map(move |s| (s.month, p.county.key.as_str(), s.param))
        })
        .collect();
    all.sort_unstable();
    for (mo, k, q) in all {
        let key = format!("{k}.{}.{}", q.key(h), history::stamp(w, mo));
        line(&format!(
            "        (key: \"{key}\", at: \"{}\", basis: {rule}, act: SetParam(param: \"{k}.{}\", to: \"{key}\")),",
            history::date(w, mo),
            q.key(h)
        ));
    }
    line("    ],");
    line("    recurring: [],");
    line(")");
    (o, events)
}
