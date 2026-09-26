//! Criteria: every bar a verdict compares against, registered before the run in a dated file,
//! one per tape, `criteria/<tape>-<date>.ron` (docs/CERTIFY.md §4; R4, R9, C3, C12).
//!
//! A bar carries its value, its unit and its basis (R4's five tags), and nothing has a default:
//! a bare number does not load. A retune is a new dated file with its reason and the file it
//! supersedes, never an edit. `from_ron` makes the load checks, which need only the file;
//! `fit` makes the checks that need the tape's clock, and turns spans into ticks.

use crate::certificate::BatteryId;
use crate::fold;
use crate::manifest::Hex;
use rustyecon_core::tape::raw::required;
use rustyecon_core::{fnv1a_64, Basis, Clock, Date, Years};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The criteria file format.
pub const CRITERIA_FORMAT: u32 = 1;

/// The unit of a bar (§4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarUnit {
    /// A ratio to a reference, above 1: the runaway bound.
    Ratio,
    /// A width in natural log: a band.
    LogWidth,
    /// A share of a whole, in [0, 1].
    Share,
    /// A relative size, in (0, 1): a kick, a fill's shortfall.
    Relative,
    /// A market imbalance `(D − S)/max(S, D)`, in (0, 1).
    Imbalance,
    /// A span in years, above 0.
    Years,
    /// A kick's gain, in (0, 1): its size at the end over its size at the start.
    Gain,
}

/// A threshold: its value, its unit and where it comes from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bar {
    /// The value, in `unit`.
    pub value: f64,
    /// The unit.
    pub unit: BarUnit,
    /// Where the value comes from (R4).
    pub basis: Basis,
}

/// A whole number of samples, with its basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Count {
    /// The count.
    pub value: u64,
    /// Where it comes from (R4).
    pub basis: Basis,
}

/// The tape the criteria were registered for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TapeRef {
    /// The tape's name.
    pub name: String,
    /// Its tape hash (§3).
    pub tape_hash: Hex,
}

/// One battery and its bars (§6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BatterySpec {
    /// Every tick's and the run's ledger margin at most 1, and the run reaches `until` (R2). No
    /// bar: a margin of 1 is the ledger's own definition.
    Conservation,
    /// Repeat, replay and resume give the same run (R8): resumed at each share of the run.
    Determinism {
        /// `Share`s of the run, each in the open (0, 1).
        resume_at: Vec<Bar>,
    },
    /// Every price stays within [1/bound, bound] of its genesis value (A12).
    Runaway {
        /// A `Ratio` above 1.
        bound: Bar,
    },
    /// Every market trades in every window of this span.
    Trades {
        /// `Years`, above 0.
        every: Bar,
    },
    /// No market's imbalance is pinned (BalanceWatch), in every segment.
    Balance {
        /// An `Imbalance`: a mean or run value above it in size can be a pin.
        level: Bar,
        /// An `Imbalance`: a standard deviation below it is no movement.
        spread: Bar,
        /// At least 2: the observations a verdict needs.
        min_samples: Count,
        /// A `Share` in (0, 1]: an unbroken run of one value this long is a pin.
        run_share: Bar,
    },
    /// Each segment settles: few dead ticks in W, none in F, and price and volume at rest in F
    /// (REPORT §6, criteria 1 and 3).
    Settles {
        /// A `Share`: W starts this far into a segment.
        w_from: Bar,
        /// A `Share`: F starts this far in, `w_from ≤ f_from < 1`.
        f_from: Bar,
        /// A `Share` below 1: the dead ticks W may hold, as a share of it.
        dead_share: Bar,
        /// A `LogWidth`: the widest band of `ln` price and `ln` cleared in F.
        band: Bar,
    },
    /// A 1e-9 kick in ± each price at each segment's end decays and never grows far (REPORT §5,
    /// §6 criterion 1).
    Kick {
        /// A `Relative` in (0, 1) that moves a float: the kick's size.
        size: Bar,
        /// `Years`: how long each kicked run goes on.
        horizon: Bar,
        /// A `Share` in (0, 1]: the last part of the horizon the gain is read over.
        tail: Bar,
        /// A `Gain` in (0, 1): the largest the kick may be, over its start, in the tail.
        max_gain: Bar,
        /// A `Ratio` above 1: the largest the kick may grow, over its start, anywhere in the
        /// horizon (amended at S2.3: the tail alone passed a kick that swung ×3e9 through dead
        /// markets and came back to the unstable point it left).
        max_peak: Bar,
    },
}

impl BatterySpec {
    /// The battery's id.
    pub fn id(&self) -> BatteryId {
        match self {
            BatterySpec::Conservation => BatteryId::Conservation,
            BatterySpec::Determinism { .. } => BatteryId::Determinism,
            BatterySpec::Runaway { .. } => BatteryId::Runaway,
            BatterySpec::Trades { .. } => BatteryId::Trades,
            BatterySpec::Balance { .. } => BatteryId::Balance,
            BatterySpec::Settles { .. } => BatteryId::Settles,
            BatterySpec::Kick { .. } => BatteryId::Kick,
        }
    }
}

/// The reports' one bar (§6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportSpec {
    /// A `Relative`: a fill below `1 − rationed_below` counts as rationed.
    pub rationed_below: Bar,
}

/// A tape's criteria, registered before its run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "Criteria")]
pub struct Criteria {
    /// [`CRITERIA_FORMAT`].
    pub format: u32,
    /// The registration date, which the file's name carries.
    pub date: Date,
    /// Why the file exists.
    pub reason: String,
    /// The file it replaces, an earlier one for the same tape.
    #[serde(deserialize_with = "required")]
    pub supersedes: Option<String>,
    /// The tape it was registered for.
    pub tape: TapeRef,
    /// The date the scored run ends at.
    pub until: Date,
    /// Why the run is that long.
    pub until_basis: Basis,
    /// `Years`: the shortest segment a dated shock may close (§5, C5).
    pub min_segment: Bar,
    /// The `ScalePrice` firings the run may carry, with their basis (amended at S2.5). A tape
    /// that sets its own prices certifies only as far as this says: dense dated shocks are a
    /// periodic nudge in all but name, which R3 bans. The one field that may be absent, since
    /// its absence is the strictest bar, none: a file written before it cannot loosen a verdict.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_shocks: Option<Count>,
    /// The batteries, each once.
    pub batteries: Vec<BatterySpec>,
    /// The reports' bar.
    pub reports: ReportSpec,
}

/// A criteria file that does not load, or does not fit its tape: where, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriteriaError {
    /// The path in the file, such as `batteries[Kick].size`.
    pub path: String,
    /// What is wrong.
    pub why: String,
}

impl fmt::Display for CriteriaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.why)
        } else {
            write!(f, "{}: {}", self.path, self.why)
        }
    }
}

impl std::error::Error for CriteriaError {}

fn err(path: impl Into<String>, why: impl Into<String>) -> CriteriaError {
    CriteriaError {
        path: path.into(),
        why: why.into(),
    }
}

/// The criteria in ticks, fitted to a tape's clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fit {
    /// n: the scored run's ticks, `tick_of(until)`.
    pub until: u64,
    /// m: the shortest segment.
    pub min_segment: u64,
    /// Each resume tick, `⌊share·n⌋`, in the file's order.
    pub resume_at: Vec<u64>,
    /// E: the Trades window.
    pub trades_every: Option<u64>,
    /// H: each kicked run's ticks.
    pub kick_horizon: Option<u64>,
    /// `⌈tail·H⌉`: the ticks the gain is read over.
    pub kick_tail: Option<u64>,
}

fn basis_text(b: &Basis) -> Vec<&str> {
    match b {
        Basis::Measured { source, vintage } => vec![source, vintage],
        Basis::Literature(s) | Basis::Approximate(s) | Basis::Assumed(s) => vec![s],
        Basis::Fitted { fit } => vec![fit],
    }
}

/// A bar of `unit` whose value passes `ok`, else an error at `path` saying it must be `what`.
fn bar(b: &Bar, path: &str, unit: BarUnit, what: &str, ok: bool) -> Result<f64, CriteriaError> {
    if b.unit != unit {
        return Err(err(
            path,
            format!("a {:?} bar, where this field is {unit:?}", b.unit),
        ));
    }
    if basis_text(&b.basis).iter().any(|t| t.trim().is_empty()) {
        return Err(err(
            path,
            "an empty basis: every bar says where it comes from",
        ));
    }
    if !(b.value.is_finite() && ok) {
        return Err(err(path, format!("{:e} is not {what}", b.value)));
    }
    Ok(b.value)
}

fn share(b: &Bar, path: &str) -> Result<f64, CriteriaError> {
    let v = b.value;
    bar(
        b,
        path,
        BarUnit::Share,
        "a share in [0, 1]",
        (0.0..=1.0).contains(&v),
    )
}

fn open_unit(b: &Bar, path: &str, unit: BarUnit) -> Result<f64, CriteriaError> {
    let v = b.value;
    bar(b, path, unit, "in the open (0, 1)", 0.0 < v && v < 1.0)
}

fn years(b: &Bar, path: &str) -> Result<f64, CriteriaError> {
    let v = b.value;
    bar(b, path, BarUnit::Years, "a span above 0 years", 0.0 < v)
}

/// `<tape>-<YYYY-MM-DD>.ron`: the tape name and date a criteria file's name carries.
fn name_and_date(file: &str) -> Option<(&str, Date)> {
    let base = file.rsplit(['/', '\\']).next()?;
    let stem = base.strip_suffix(".ron")?;
    let cut = stem.len().checked_sub(11)?;
    let (name, rest) = stem.split_at(cut);
    let date = Date::parse(rest.strip_prefix('-')?).ok()?;
    Some((name, date))
}

fn pretty() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::new().new_line("\n".to_string())
}

impl Criteria {
    /// Read and check a criteria file; `file` is its name, `<tape>-<date>.ron`, whose tape name
    /// and date must be the file's own (§4's load checks).
    pub fn from_ron(s: &str, file: &str) -> Result<Criteria, CriteriaError> {
        #[derive(Deserialize)]
        #[serde(rename = "Criteria")]
        struct Probe {
            format: u32,
        }
        if let Ok(p) = ron::from_str::<Probe>(s) {
            if p.format != CRITERIA_FORMAT {
                return Err(err(
                    "format",
                    format!(
                        "criteria format {}, but this build reads format {CRITERIA_FORMAT} only",
                        p.format
                    ),
                ));
            }
        }
        let c: Criteria = ron::from_str(s).map_err(|e| err("", e.to_string()))?;
        c.check(file)?;
        Ok(c)
    }

    fn check(&self, file: &str) -> Result<(), CriteriaError> {
        if self.format != CRITERIA_FORMAT {
            return Err(err("format", format!("format {}", self.format)));
        }
        let Some((name, date)) = name_and_date(file) else {
            return Err(err(
                "",
                format!("{file:?} is not named <tape>-<YYYY-MM-DD>.ron"),
            ));
        };
        if name != self.tape.name {
            return Err(err(
                "tape.name",
                format!("{:?}, but the file is named for {name:?}", self.tape.name),
            ));
        }
        if date != self.date {
            return Err(err(
                "date",
                format!("{}, but the file is dated {date}", self.date),
            ));
        }
        if let Some(prev) = &self.supersedes {
            match name_and_date(prev) {
                Some((n, d)) if n == self.tape.name && d < self.date => {}
                _ => {
                    return Err(err(
                        "supersedes",
                        format!(
                            "{prev:?} is not an earlier criteria file for {:?}",
                            self.tape.name
                        ),
                    ))
                }
            }
        }
        if basis_text(&self.until_basis)
            .iter()
            .any(|t| t.trim().is_empty())
        {
            return Err(err("until_basis", "an empty basis"));
        }
        if let Some(c) = &self.price_shocks {
            if basis_text(&c.basis).iter().any(|t| t.trim().is_empty()) {
                return Err(err("price_shocks", "an empty basis"));
            }
        }
        if self.reason.trim().is_empty() {
            return Err(err("reason", "the file says why it exists"));
        }
        years(&self.min_segment, "min_segment")?;
        open_unit(
            &self.reports.rationed_below,
            "reports.rationed_below",
            BarUnit::Relative,
        )?;
        let mut listed: Vec<BatteryId> = Vec::new();
        for b in &self.batteries {
            let id = b.id();
            let at = |f: &str| format!("batteries[{id}].{f}");
            if listed.contains(&id) {
                return Err(err(format!("batteries[{id}]"), "listed twice"));
            }
            listed.push(id);
            match b {
                BatterySpec::Conservation => {}
                BatterySpec::Determinism { resume_at } => {
                    if resume_at.is_empty() {
                        return Err(err(at("resume_at"), "no resume: the battery scores none"));
                    }
                    for (i, s) in resume_at.iter().enumerate() {
                        let v = s.value;
                        bar(
                            s,
                            &at(&format!("resume_at[{i}]")),
                            BarUnit::Share,
                            "a share in the open (0, 1)",
                            0.0 < v && v < 1.0,
                        )?;
                    }
                }
                BatterySpec::Runaway { bound } => {
                    let v = bound.value;
                    bar(
                        bound,
                        &at("bound"),
                        BarUnit::Ratio,
                        "a ratio above 1",
                        1.0 < v,
                    )?;
                }
                BatterySpec::Trades { every } => {
                    years(every, &at("every"))?;
                }
                BatterySpec::Balance {
                    level,
                    spread,
                    min_samples,
                    run_share,
                } => {
                    open_unit(level, &at("level"), BarUnit::Imbalance)?;
                    open_unit(spread, &at("spread"), BarUnit::Imbalance)?;
                    if min_samples.value < 2 {
                        return Err(err(at("min_samples"), "a spread needs at least 2 samples"));
                    }
                    if basis_text(&min_samples.basis)
                        .iter()
                        .any(|t| t.trim().is_empty())
                    {
                        return Err(err(at("min_samples"), "an empty basis"));
                    }
                    let v = run_share.value;
                    bar(
                        run_share,
                        &at("run_share"),
                        BarUnit::Share,
                        "a share in (0, 1]",
                        0.0 < v && v <= 1.0,
                    )?;
                }
                BatterySpec::Settles {
                    w_from,
                    f_from,
                    dead_share,
                    band,
                } => {
                    let w = share(w_from, &at("w_from"))?;
                    let f = share(f_from, &at("f_from"))?;
                    if !(w <= f && f < 1.0) {
                        return Err(err(at("f_from"), "w_from ≤ f_from < 1 is required"));
                    }
                    let d = dead_share.value;
                    bar(
                        dead_share,
                        &at("dead_share"),
                        BarUnit::Share,
                        "a share below 1",
                        (0.0..1.0).contains(&d),
                    )?;
                    let v = band.value;
                    bar(
                        band,
                        &at("band"),
                        BarUnit::LogWidth,
                        "a width above 0",
                        0.0 < v,
                    )?;
                }
                BatterySpec::Kick {
                    size,
                    horizon,
                    tail,
                    max_gain,
                    max_peak,
                } => {
                    let s = open_unit(size, &at("size"), BarUnit::Relative)?;
                    // The kick must move a float: fl(1 − size) < 1 < fl(1 + size).
                    if !(1.0 - s < 1.0 && 1.0 < 1.0 + s) {
                        return Err(err(at("size"), format!("{s:e} does not move 1 in f64")));
                    }
                    years(horizon, &at("horizon"))?;
                    let t = tail.value;
                    bar(
                        tail,
                        &at("tail"),
                        BarUnit::Share,
                        "a share in (0, 1]",
                        0.0 < t && t <= 1.0,
                    )?;
                    open_unit(max_gain, &at("max_gain"), BarUnit::Gain)?;
                    let v = max_peak.value;
                    bar(
                        max_peak,
                        &at("max_peak"),
                        BarUnit::Ratio,
                        "a ratio above 1",
                        1.0 < v,
                    )?;
                }
            }
        }
        // C12: the three every scored run needs, and Settles with Kick or neither.
        for need in [
            BatteryId::Conservation,
            BatteryId::Determinism,
            BatteryId::Runaway,
        ] {
            if !listed.contains(&need) {
                return Err(err(
                    "batteries",
                    format!("{need} is not listed: every scored run needs it (C12)"),
                ));
            }
        }
        let (settles, kick) = (
            listed.contains(&BatteryId::Settles),
            listed.contains(&BatteryId::Kick),
        );
        if settles != kick {
            return Err(err(
                "batteries",
                "Settles and Kick are listed together or not at all (C12): at rest is not \
                 settled until a kick decays",
            ));
        }
        Ok(())
    }

    /// The `ScalePrice` firings the criteria allow: the registered count, or 0 when none is.
    pub fn allowed_price_shocks(&self) -> u64 {
        self.price_shocks.as_ref().map_or(0, |c| c.value)
    }

    /// The batteries listed, in id order.
    pub fn listed(&self) -> Vec<BatteryId> {
        let mut v: Vec<BatteryId> = self.batteries.iter().map(BatterySpec::id).collect();
        v.sort();
        v
    }

    /// The spec of a listed battery.
    pub fn battery(&self, id: BatteryId) -> Option<&BatterySpec> {
        self.batteries.iter().find(|b| b.id() == id)
    }

    /// Fit the criteria to a tape's clock (§4's fit checks): the run holds a full segment and a
    /// full Trades window, each resume tick lies strictly inside the run and apart from the
    /// others, a kick runs at least two ticks and reads at least two, a segment can hold enough
    /// samples to find a pin, and F holds two ticks in the shortest segment.
    pub fn fit(&self, clock: &Clock) -> Result<Fit, CriteriaError> {
        let ticks = |b: &Bar, path: &str| -> Result<u64, CriteriaError> {
            clock
                .ticks(Years(b.value))
                .map(u64::from)
                .map_err(|e| err(path, e.to_string()))
        };
        let n = clock
            .tick_of(self.until)
            .map_err(|e| err("until", e.to_string()))?;
        let m = ticks(&self.min_segment, "min_segment")?;
        if n < m {
            return Err(err(
                "until",
                format!("the run's {n} ticks do not hold one segment of {m}"),
            ));
        }
        let mut fit = Fit {
            until: n,
            min_segment: m,
            resume_at: Vec::new(),
            trades_every: None,
            kick_horizon: None,
            kick_tail: None,
        };
        for b in &self.batteries {
            let id = b.id();
            let at = |f: &str| format!("batteries[{id}].{f}");
            match b {
                BatterySpec::Determinism { resume_at } => {
                    for (i, s) in resume_at.iter().enumerate() {
                        let t = fold::floor_share(s.value, n);
                        if t == 0 || t >= n || fit.resume_at.contains(&t) {
                            return Err(err(
                                at(&format!("resume_at[{i}]")),
                                format!("tick {t} is not a distinct tick strictly inside (0, {n})"),
                            ));
                        }
                        fit.resume_at.push(t);
                    }
                }
                BatterySpec::Trades { every } => {
                    let e = ticks(every, &at("every"))?;
                    if n < e {
                        return Err(err(
                            at("every"),
                            format!("the run's {n} ticks do not hold one window of {e}"),
                        ));
                    }
                    fit.trades_every = Some(e);
                }
                BatterySpec::Balance { min_samples, .. } => {
                    if m < min_samples.value {
                        return Err(err(
                            at("min_samples"),
                            format!(
                                "a segment of {m} ticks cannot hold {} samples",
                                min_samples.value
                            ),
                        ));
                    }
                }
                BatterySpec::Settles { f_from, .. } => {
                    let (from, to) = (fold::window_start(0, m, f_from.value), m);
                    if to.saturating_sub(from) < 2 {
                        return Err(err(
                            at("f_from"),
                            format!(
                                "F holds {} ticks of the shortest segment",
                                to - from.min(to)
                            ),
                        ));
                    }
                }
                BatterySpec::Kick { horizon, tail, .. } => {
                    let h = ticks(horizon, &at("horizon"))?;
                    if h < 2 {
                        return Err(err(at("horizon"), format!("{h} tick: a kick needs two")));
                    }
                    let k = fold::ceil_share(tail.value, h);
                    if k < 2 {
                        return Err(err(
                            at("tail"),
                            format!("the tail holds {k} tick of the horizon: it needs two"),
                        ));
                    }
                    fit.kick_horizon = Some(h);
                    fit.kick_tail = Some(k);
                }
                BatterySpec::Conservation | BatterySpec::Runaway { .. } => {}
            }
        }
        Ok(fit)
    }

    /// The criteria as RON, opening `Criteria(`: what [`Criteria::hash`] hashes.
    pub fn to_ron(&self) -> String {
        match ron::ser::to_string_pretty(self, pretty()) {
            Ok(s) => format!("Criteria{s}\n"),
            Err(e) => format!("/* the criteria do not serialise: {e} */\n"),
        }
    }

    /// FNV-1a 64 over [`Criteria::to_ron`]: the criteria's identity, which a certificate records
    /// with the file's name and date (`CriteriaRef`). The manifest records the run's inputs, not
    /// the criteria that scored it.
    pub fn hash(&self) -> u64 {
        fnv1a_64(self.to_ron().as_bytes())
    }
}
