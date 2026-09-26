//! The certificate (docs/CERTIFY.md §8; R9, N4, N12): verdict-first, fail-closed, persisted.
//!
//! One crate-private function, [`seal`], makes every certificate, and reading one back seals its
//! parts again, so a verdict cannot be edited into a file that disagrees with its numbers. The
//! seal reads every number the certificate serialises: a non-finite one anywhere fails the run,
//! with or without criteria (N12; July's scan read state fields, not what its certificate
//! rendered, `v2p3: certify/nan.rs:47-57`). A run with no criteria, or with criteria for another
//! tape, is UNSCORED and never PASS (N4; July certified such a run PASS, `v2p3:
//! runner.rs:473-478`, `certificate.rs:78`).

use crate::leaves::{leaves, Leaf};
use crate::manifest::{Hex, RunKey};
use crate::obs::Segment;
use rustyecon_core::tape::raw::required;
use rustyecon_core::Date;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A run's verdict. FAIL outranks UNSCORED, which outranks PASS (C4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Verdict {
    /// Every listed battery passed, every number is finite, the run reached its end, and the
    /// criteria were registered for this tape.
    Pass,
    /// Some battery failed, some number is not finite, the run stopped early, or the criteria
    /// lack a battery every scored run needs.
    Fail,
    /// Nothing failed, but there were no criteria for this tape (N4).
    Unscored,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Unscored => "UNSCORED",
        })
    }
}

/// A battery (§6), in the order a certificate lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BatteryId {
    /// R2: the ledger.
    Conservation,
    /// R8: repeat, replay, resume.
    Determinism,
    /// A12: the relative runaway bound.
    Runaway,
    /// Liveness: every market trades in every window.
    Trades,
    /// BalanceWatch: no pinned imbalance.
    Balance,
    /// REPORT §6: at rest, per segment.
    Settles,
    /// REPORT §5–§6: a kick decays.
    Kick,
}

impl fmt::Display for BatteryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// One number a battery read or a report computed: its name, what it is of (`town/bread`,
/// `bread`, `provider`, `run`), its segment, its value and the bar it was compared with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    /// Its name, such as `kick.gain_tail`.
    pub name: String,
    /// What it is of.
    pub at: String,
    /// The segment it belongs to, if one.
    #[serde(deserialize_with = "required")]
    pub segment: Option<u32>,
    /// The value.
    pub value: f64,
    /// The bar, if the verdict compares it with one.
    #[serde(deserialize_with = "required")]
    pub bar: Option<f64>,
}

/// One battery's outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatteryResult {
    /// The battery.
    pub id: BatteryId,
    /// Whether it passed.
    pub pass: bool,
    /// Every number it read, at least one for each thing it scores.
    pub readings: Vec<Reading>,
    /// What a reader should know, in words; no verdict reads them.
    pub notes: Vec<String>,
}

/// The criteria a run was scored against: enough to recompute every sealing rule on readback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriteriaRef {
    /// The file's base name.
    pub file: String,
    /// Its date.
    pub date: Date,
    /// `Criteria::hash`.
    pub hash: Hex,
    /// The tape hash it was registered for.
    pub tape_hash: Hex,
    /// The batteries it lists, in id order.
    pub listed: Vec<BatteryId>,
}

/// Everything a certificate holds but its verdict, its failures and its list of non-finite
/// numbers, which the seal computes from these.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Parts {
    pub(crate) run: RunKey,
    pub(crate) tape: String,
    pub(crate) criteria: Option<CriteriaRef>,
    pub(crate) until: u64,
    pub(crate) reached: u64,
    pub(crate) stopped: Option<String>,
    pub(crate) genesis_hash: Hex,
    pub(crate) final_hash: Hex,
    pub(crate) price_shocks: u64,
    pub(crate) segments: Vec<Segment>,
    pub(crate) batteries: Vec<BatteryResult>,
    pub(crate) reports: Vec<Reading>,
}

/// A run's certificate. Made only by the seal; read back only through it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawCertificate")]
pub struct Certificate {
    verdict: Verdict,
    failures: Vec<String>,
    run: RunKey,
    tape: String,
    criteria: Option<CriteriaRef>,
    until: u64,
    reached: u64,
    stopped: Option<String>,
    genesis_hash: Hex,
    final_hash: Hex,
    price_shocks: u64,
    segments: Vec<Segment>,
    batteries: Vec<BatteryResult>,
    reports: Vec<Reading>,
    nonfinite: Vec<String>,
}

/// A certificate as read, before the seal checks it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "Certificate")]
pub struct RawCertificate {
    /// The verdict it claims.
    pub verdict: Verdict,
    /// The failures it claims.
    pub failures: Vec<String>,
    /// The run.
    pub run: RunKey,
    /// The tape's name.
    pub tape: String,
    /// The criteria.
    #[serde(deserialize_with = "required")]
    pub criteria: Option<CriteriaRef>,
    /// The tick the run was to reach.
    pub until: u64,
    /// The tick it reached.
    pub reached: u64,
    /// Why it stopped early.
    #[serde(deserialize_with = "required")]
    pub stopped: Option<String>,
    /// The genesis state's hash.
    pub genesis_hash: Hex,
    /// The last state's hash.
    pub final_hash: Hex,
    /// Its `ScalePrice` firings.
    pub price_shocks: u64,
    /// Its segments.
    pub segments: Vec<Segment>,
    /// The batteries' outcomes.
    pub batteries: Vec<BatteryResult>,
    /// The reports.
    pub reports: Vec<Reading>,
    /// The non-finite numbers it claims.
    pub nonfinite: Vec<String>,
}

/// A certificate that does not read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateError {
    /// The RON does not parse as a certificate.
    Parse(String),
    /// A non-finite number the certificate does not list as one.
    NonFinite {
        /// Its path.
        path: String,
    },
    /// A verdict, failure list or non-finite list other than the seal gives for its numbers.
    EditedVerdict {
        /// What the file says.
        claimed: String,
        /// What the seal gives.
        sealed: String,
    },
}

impl fmt::Display for CertificateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertificateError::Parse(m) => write!(f, "the certificate does not parse: {m}"),
            CertificateError::NonFinite { path } => {
                write!(
                    f,
                    "a non-finite number at {path} that the certificate does not list"
                )
            }
            CertificateError::EditedVerdict { claimed, sealed } => write!(
                f,
                "the certificate says {claimed}, but its numbers seal to {sealed}"
            ),
        }
    }
}

impl std::error::Error for CertificateError {}

/// The path of every non-finite `f64` that `v` serialises, in serialisation order (N12): the
/// scan a certificate is sealed and read back with.
pub fn nonfinite_paths<T: Serialize + ?Sized>(v: &T) -> Vec<String> {
    leaves(v)
        .into_iter()
        .filter_map(|(p, l)| match l {
            Leaf::F64(x) if !x.is_finite() => Some(p),
            _ => None,
        })
        .collect()
}

/// The batteries every scored run lists (C12).
const REQUIRED: [BatteryId; 3] = [
    BatteryId::Conservation,
    BatteryId::Determinism,
    BatteryId::Runaway,
];

/// Seal a certificate (§8): every listed battery gets a result (one that did not run fails
/// with the note "did not run"), then the non-finite scan, the failures and the verdict. FAIL:
/// the run stopped before `until` (its error first), a non-finite number anywhere, criteria that
/// lack a battery C12 requires, or any battery that did not pass. Else UNSCORED: no criteria, or
/// criteria for another tape. Else PASS.
pub(crate) fn seal(mut parts: Parts) -> Certificate {
    if let Some(c) = &parts.criteria {
        for id in &c.listed {
            if !parts.batteries.iter().any(|b| b.id == *id) {
                parts.batteries.push(BatteryResult {
                    id: *id,
                    pass: false,
                    readings: Vec::new(),
                    notes: vec!["did not run".to_string()],
                });
            }
        }
    }
    parts.batteries.sort_by_key(|b| b.id);
    let nonfinite = nonfinite_paths(&parts);
    let mut failures = Vec::new();
    if parts.reached < parts.until {
        failures.push(format!(
            "the run stopped at tick {} of {}: {}",
            parts.reached,
            parts.until,
            parts.stopped.as_deref().unwrap_or("no error was recorded")
        ));
    }
    for p in &nonfinite {
        failures.push(format!("a non-finite number at {p}"));
    }
    let mut unscored = None;
    match &parts.criteria {
        None => unscored = Some("unscored: no criteria were registered for this tape".to_string()),
        Some(c) => {
            for need in REQUIRED {
                if !c.listed.contains(&need) {
                    failures.push(format!(
                        "the criteria do not list {need}, which every scored run needs (C12)"
                    ));
                }
            }
            let settles = c.listed.contains(&BatteryId::Settles);
            if settles != c.listed.contains(&BatteryId::Kick) {
                failures.push(
                    "the criteria list one of Settles and Kick without the other (C12)".to_string(),
                );
            }
            if c.tape_hash != parts.run.tape_hash {
                unscored = Some(format!(
                    "unscored: the criteria were registered for tape_hash {}, not this tape's {}",
                    c.tape_hash, parts.run.tape_hash
                ));
            }
        }
    }
    for b in &parts.batteries {
        if !b.pass {
            let why = b.notes.first().map_or("", String::as_str);
            failures.push(format!("{} failed: {why}", b.id));
        }
    }
    let verdict = if !failures.is_empty() {
        Verdict::Fail
    } else if unscored.is_some() {
        Verdict::Unscored
    } else {
        Verdict::Pass
    };
    failures.extend(unscored);
    let Parts {
        run,
        tape,
        criteria,
        until,
        reached,
        stopped,
        genesis_hash,
        final_hash,
        price_shocks,
        segments,
        batteries,
        reports,
    } = parts;
    Certificate {
        verdict,
        failures,
        run,
        tape,
        criteria,
        until,
        reached,
        stopped,
        genesis_hash,
        final_hash,
        price_shocks,
        segments,
        batteries,
        reports,
        nonfinite,
    }
}

impl TryFrom<RawCertificate> for Certificate {
    type Error = CertificateError;

    /// Read a certificate back: every non-finite number must be listed as one, and sealing its
    /// parts again must give the verdict, failures and list it claims (§8).
    fn try_from(raw: RawCertificate) -> Result<Certificate, CertificateError> {
        let parts = Parts {
            run: raw.run,
            tape: raw.tape,
            criteria: raw.criteria,
            until: raw.until,
            reached: raw.reached,
            stopped: raw.stopped,
            genesis_hash: raw.genesis_hash,
            final_hash: raw.final_hash,
            price_shocks: raw.price_shocks,
            segments: raw.segments,
            batteries: raw.batteries,
            reports: raw.reports,
        };
        if let Some(path) = nonfinite_paths(&parts)
            .into_iter()
            .find(|p| !raw.nonfinite.contains(p))
        {
            return Err(CertificateError::NonFinite { path });
        }
        let sealed = seal(parts);
        if sealed.verdict != raw.verdict
            || sealed.failures != raw.failures
            || sealed.nonfinite != raw.nonfinite
        {
            return Err(CertificateError::EditedVerdict {
                claimed: format!("{} with {} failure lines", raw.verdict, raw.failures.len()),
                sealed: format!(
                    "{} with {} failure lines",
                    sealed.verdict,
                    sealed.failures.len()
                ),
            });
        }
        Ok(sealed)
    }
}

fn pretty() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::new().new_line("\n".to_string())
}

impl Certificate {
    /// The verdict.
    pub fn verdict(&self) -> Verdict {
        self.verdict
    }

    /// Each reason the verdict is not PASS, the run's own error first.
    pub fn failures(&self) -> &[String] {
        &self.failures
    }

    /// The run's key.
    pub fn run(&self) -> &RunKey {
        &self.run
    }

    /// The tape's name.
    pub fn tape(&self) -> &str {
        &self.tape
    }

    /// The criteria it was scored against.
    pub fn criteria(&self) -> Option<&CriteriaRef> {
        self.criteria.as_ref()
    }

    /// The tick the run was to reach.
    pub fn until(&self) -> u64 {
        self.until
    }

    /// The tick it reached.
    pub fn reached(&self) -> u64 {
        self.reached
    }

    /// The error that stopped it early.
    pub fn stopped(&self) -> Option<&str> {
        self.stopped.as_deref()
    }

    /// The genesis state's hash.
    pub fn genesis_hash(&self) -> Hex {
        self.genesis_hash
    }

    /// The last state's hash.
    pub fn final_hash(&self) -> Hex {
        self.final_hash
    }

    /// The run's `ScalePrice` firings: a tape that sets its own prices says so.
    pub fn price_shocks(&self) -> u64 {
        self.price_shocks
    }

    /// The segments.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// The batteries' outcomes, in id order.
    pub fn batteries(&self) -> &[BatteryResult] {
        &self.batteries
    }

    /// One battery's outcome.
    pub fn battery(&self, id: BatteryId) -> Option<&BatteryResult> {
        self.batteries.iter().find(|b| b.id == id)
    }

    /// The reports: transient statistics, never scored.
    pub fn reports(&self) -> &[Reading] {
        &self.reports
    }

    /// The path of every non-finite number.
    pub fn nonfinite(&self) -> &[String] {
        &self.nonfinite
    }

    /// The certificate as text, verdict first (§8): the verdict line with the tape, its hash,
    /// the world and the build, then the failures, then each battery with its readings and
    /// notes, then the reports. It prints serialised fields only, each float in Rust's shortest
    /// round-trip form, and does no arithmetic.
    pub fn render(&self) -> String {
        let b = &self.run.build;
        let mut out = format!(
            "VERDICT: {}  {}  tape_hash {}  world_id {}  build {} {} {}\n",
            self.verdict,
            self.tape,
            self.run.tape_hash,
            self.run.world_id,
            b.commit,
            b.state(),
            b.target
        );
        for f in &self.failures {
            out.push_str(&format!("  - {f}\n"));
        }
        match &self.criteria {
            Some(c) => out.push_str(&format!(
                "criteria {} ({}, hash {}, for tape_hash {})\n",
                c.file, c.date, c.hash, c.tape_hash
            )),
            None => out.push_str("criteria none\n"),
        }
        out.push_str(&format!(
            "reached {} of {}; genesis {}; final {}; price shocks {}\n",
            self.reached, self.until, self.genesis_hash, self.final_hash, self.price_shocks
        ));
        for s in &self.segments {
            out.push_str(&format!("segment [{}, {})", s.from, s.to));
            if !s.opened_by.is_empty() {
                let keys: Vec<&str> = s.opened_by.iter().map(|k| k.as_str()).collect();
                out.push_str(&format!(" opened by {}", keys.join(", ")));
            }
            if !s.merged.is_empty() {
                let keys: Vec<&str> = s.merged.iter().map(|k| k.as_str()).collect();
                out.push_str(&format!(" merged {}", keys.join(", ")));
            }
            out.push('\n');
        }
        for bat in &self.batteries {
            let pass = if bat.pass { "PASS" } else { "FAIL" };
            out.push_str(&format!("{pass} {}\n", bat.id));
            for n in &bat.notes {
                out.push_str(&format!("    note: {n}\n"));
            }
            for r in &bat.readings {
                out.push_str(&format!("    {}\n", reading_line(r)));
            }
        }
        if !self.reports.is_empty() {
            out.push_str("reports\n");
            for r in &self.reports {
                out.push_str(&format!("    {}\n", reading_line(r)));
            }
        }
        out
    }

    /// The certificate as RON, opening `Certificate(verdict: ..`.
    pub fn to_ron(&self) -> String {
        match ron::ser::to_string_pretty(self, pretty()) {
            Ok(s) => format!("Certificate{s}\n"),
            Err(e) => format!("/* the certificate does not serialise: {e} */\n"),
        }
    }

    /// Read a certificate back through the seal (§8).
    pub fn from_ron(s: &str) -> Result<Certificate, CertificateError> {
        let raw: RawCertificate =
            ron::from_str(s).map_err(|e| CertificateError::Parse(e.to_string()))?;
        Certificate::try_from(raw)
    }
}

fn reading_line(r: &Reading) -> String {
    let mut s = format!("{} {}", r.name, r.at);
    if let Some(seg) = r.segment {
        s.push_str(&format!(" segment {seg}"));
    }
    s.push_str(&format!(" = {:?}", r.value));
    if let Some(bar) = r.bar {
        s.push_str(&format!(" (bar {bar:?})"));
    }
    s
}

#[cfg(test)]
mod tests {
    //! The seal on synthetic parts (docs/CERTIFY.md §8, §13): it needs no run, only numbers.

    use super::*;
    use crate::manifest::Build;
    use crate::obs::Segment;

    fn run_key(tape_hash: u64) -> RunKey {
        RunKey {
            build: Build {
                commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
                dirty: false,
                target: "test-target".to_string(),
                rustc: "rustc test".to_string(),
            },
            tape_hash: Hex(tape_hash),
            world_id: Hex(0xfeed),
        }
    }

    fn reading(name: &str, value: f64, bar: Option<f64>) -> Reading {
        Reading {
            name: name.to_string(),
            at: "run".to_string(),
            segment: Some(0),
            value,
            bar,
        }
    }

    fn battery(id: BatteryId, pass: bool) -> BatteryResult {
        BatteryResult {
            id,
            pass,
            readings: vec![
                reading("x.value", 0.5, Some(1.0)),
                reading("x.count", 3.0, None),
            ],
            notes: if pass {
                Vec::new()
            } else {
                vec!["it failed".to_string()]
            },
        }
    }

    fn criteria(listed: &[BatteryId], tape_hash: u64) -> CriteriaRef {
        CriteriaRef {
            file: "t-2026-09-26.ron".to_string(),
            date: Date::parse("2026-09-26").unwrap(),
            hash: Hex(1),
            tape_hash: Hex(tape_hash),
            listed: listed.to_vec(),
        }
    }

    const THREE: [BatteryId; 3] = [
        BatteryId::Conservation,
        BatteryId::Determinism,
        BatteryId::Runaway,
    ];

    /// Parts that seal PASS: criteria for this tape listing the three, each passing, and two
    /// reports.
    fn pass_parts() -> Parts {
        Parts {
            run: run_key(0xabc),
            tape: "t".to_string(),
            criteria: Some(criteria(&THREE, 0xabc)),
            until: 100,
            reached: 100,
            stopped: None,
            genesis_hash: Hex(2),
            final_hash: Hex(3),
            price_shocks: 0,
            segments: vec![Segment {
                from: 0,
                to: 100,
                opened_by: Vec::new(),
                merged: Vec::new(),
            }],
            batteries: THREE.iter().map(|id| battery(*id, true)).collect(),
            reports: vec![
                reading("trough.cleared", 0.25, None),
                reading("dead.ticks", 0.0, None),
            ],
        }
    }

    /// Set the k-th float of the parts' readings (each value, then its bar), batteries first;
    /// returns its path.
    fn set(p: &mut Parts, k: usize, x: f64) -> String {
        let mut i = 0;
        for (b, bat) in p.batteries.iter_mut().enumerate() {
            for (r, rd) in bat.readings.iter_mut().enumerate() {
                if i == k {
                    rd.value = x;
                    return format!("batteries[{b}].readings[{r}].value");
                }
                i += 1;
                if let Some(bar) = rd.bar.as_mut() {
                    if i == k {
                        *bar = x;
                        return format!("batteries[{b}].readings[{r}].bar");
                    }
                    i += 1;
                }
            }
        }
        for (r, rd) in p.reports.iter_mut().enumerate() {
            if i == k {
                rd.value = x;
                return format!("reports[{r}].value");
            }
            i += 1;
        }
        panic!("the parts hold {i} floats");
    }

    #[test]
    fn seal_fails_every_nonfinite_path() {
        // R9, N12: a non-finite number anywhere the certificate renders fails it, each path a
        // failure line, with criteria and without: the finite rule comes before UNSCORED.
        assert_eq!(seal(pass_parts()).verdict(), Verdict::Pass);
        let mut unscored = pass_parts();
        unscored.criteria = None;
        assert_eq!(seal(unscored).verdict(), Verdict::Unscored);
        let floats = crate::leaves::leaves(&pass_parts())
            .iter()
            .filter(|(_, l)| matches!(l, crate::leaves::Leaf::F64(_)))
            .count();
        assert_eq!(
            floats,
            3 * 3 + 2,
            "three batteries of three floats, and two reports"
        );
        for with_criteria in [true, false] {
            for k in 0..floats {
                for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                    let mut p = pass_parts();
                    if !with_criteria {
                        p.criteria = None;
                    }
                    let path = set(&mut p, k, x);
                    let c = seal(p);
                    assert_eq!(c.verdict(), Verdict::Fail, "{path} = {x}");
                    assert_eq!(c.nonfinite(), std::slice::from_ref(&path));
                    let line = format!("a non-finite number at {path}");
                    assert!(c.failures().contains(&line), "{:?}", c.failures());
                }
            }
        }
    }

    #[test]
    fn reports_carry_a_nonfinite_input_to_the_seal() {
        // Reports have no gate: a NaN cleared volume makes the segment's trough NaN, and the
        // seal fails the certificate for it. So does a non-finite value in any field a report
        // reads, a class line's and a provider's included: each reaches some report as NaN.
        let tick = |t: u64| crate::obs::Obs {
            tick: t,
            markets: vec![crate::obs::MarketObs {
                node: rustyecon_engine::prelude::NodeId(0),
                good: rustyecon_engine::prelude::GoodId(0),
                price: 1.0,
                next_price: 1.0,
                supply: 1.0,
                demand: 1.0,
                cleared: 1.0,
                buyer_fill: 1.0,
                seller_fill: 1.0,
            }],
            rationing: vec![crate::obs::RationObs {
                market: 0,
                class: rustyecon_engine::prelude::ClassId(0),
                side: rustyecon_engine::prelude::SideTag::Buy,
                requested: 2.0,
                feasible: 1.0,
                filled: 1.0,
            }],
            consumed: vec![1.0],
            spoiled: vec![0.0],
            transfers: vec![(
                rustyecon_engine::prelude::ActorId::Pop(rustyecon_engine::prelude::PopId(0)),
                1.0,
                1.0,
            )],
            margin: 0.0,
            run_margin: 0.0,
            price_shocks: 0,
        };
        let names = crate::obs::Names {
            markets: vec!["n/g".to_string()],
            goods: vec!["g".to_string()],
            classes: vec!["c".to_string()],
            actors: Vec::new(),
        };
        let segs = pass_parts().segments;
        let clean: Vec<crate::obs::Obs> = (0..10).map(tick).collect();
        let reports = crate::battery::reports(&clean, &segs, 1e-9, &names);
        assert!(nonfinite_paths(&reports).is_empty());
        type Set = fn(&mut crate::obs::Obs, f64);
        let fields: [(&str, Set); 11] = [
            ("supply", |o, x| o.markets[0].supply = x),
            ("demand", |o, x| o.markets[0].demand = x),
            ("cleared", |o, x| o.markets[0].cleared = x),
            ("buyer_fill", |o, x| o.markets[0].buyer_fill = x),
            ("seller_fill", |o, x| o.markets[0].seller_fill = x),
            ("requested", |o, x| o.rationing[0].requested = x),
            ("feasible", |o, x| o.rationing[0].feasible = x),
            ("filled", |o, x| o.rationing[0].filled = x),
            ("consumed", |o, x| o.consumed[0] = x),
            ("spoiled", |o, x| o.spoiled[0] = x),
            ("due", |o, x| o.transfers[0].1 = x),
        ];
        for (field, set) in fields {
            for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut obs = clean.clone();
                set(&mut obs[6], x);
                let reports = crate::battery::reports(&obs, &segs, 1e-9, &names);
                let bad = nonfinite_paths(&reports);
                assert!(!bad.is_empty(), "{field} = {x} reached no report");
                let mut p = pass_parts();
                p.reports = reports;
                assert_eq!(seal(p).verdict(), Verdict::Fail, "{field} = {x}");
            }
        }
        let mut obs = clean;
        obs[6].markets[0].cleared = f64::NAN;
        let reports = crate::battery::reports(&obs, &segs, 1e-9, &names);
        let trough = reports
            .iter()
            .position(|r| r.name == "trough.cleared")
            .unwrap();
        assert!(reports[trough].value.is_nan());
        let mut p = pass_parts();
        p.reports = reports;
        let c = seal(p);
        assert_eq!(c.verdict(), Verdict::Fail);
        assert!(c.nonfinite().contains(&format!("reports[{trough}].value")));
    }

    /// A certificate for each sealing rule, with the verdict it seals to.
    fn one_per_rule() -> Vec<(&'static str, Certificate, Verdict)> {
        let mut out = vec![("pass", seal(pass_parts()), Verdict::Pass)];
        let mut p = pass_parts();
        set(&mut p, 4, f64::NAN);
        out.push(("non-finite", seal(p), Verdict::Fail));
        let mut p = pass_parts();
        p.reached = 60;
        p.stopped = Some("tick 60, phase 7 (measure): a breach".to_string());
        out.push(("stopped", seal(p), Verdict::Fail));
        let mut p = pass_parts();
        p.criteria = Some(criteria(&THREE[..2], 0xabc));
        p.batteries.pop();
        out.push(("C12", seal(p), Verdict::Fail));
        let mut p = pass_parts();
        p.batteries[2].pass = false;
        p.batteries[2].notes = vec!["it failed".to_string()];
        out.push(("battery", seal(p), Verdict::Fail));
        let mut p = pass_parts();
        p.batteries.pop();
        out.push(("did not run", seal(p), Verdict::Fail));
        let mut p = pass_parts();
        p.criteria = None;
        out.push(("no criteria", seal(p), Verdict::Unscored));
        let mut p = pass_parts();
        p.criteria = Some(criteria(&THREE, 0xdef));
        out.push(("another tape", seal(p), Verdict::Unscored));
        out
    }

    #[test]
    fn a_listed_battery_with_no_result_fails() {
        // N4: a listed battery that did not run is not a pass. The seal gives it a result that
        // fails with "did not run", and the run fails.
        let (_, c, v) = one_per_rule().remove(5);
        assert_eq!(v, Verdict::Fail);
        let kick = c
            .battery(BatteryId::Runaway)
            .expect("the seal adds a result");
        assert!(!kick.pass);
        assert_eq!(kick.notes, ["did not run"]);
        assert_eq!(c.failures(), ["Runaway failed: did not run"]);
    }

    #[test]
    fn edited_verdict_is_refused() {
        // §8: every certificate is read back through the seal, so a verdict edited into a file
        // that disagrees with its numbers is refused, by from_ron and by ron::from_str alike, for
        // each sealing rule; and the file as written reads back equal.
        for (rule, c, v) in one_per_rule() {
            assert_eq!(c.verdict(), v, "{rule}: {:?}", c.failures());
            let text = c.to_ron();
            // Read back to the same text (a NaN is not equal to itself, its text is).
            let back = Certificate::from_ron(&text).map(|b| b.to_ron());
            assert_eq!(back.as_ref(), Ok(&text), "{rule}");
            let back = ron::from_str::<Certificate>(&text).map(|b| b.to_ron());
            assert_eq!(back.ok().as_ref(), Some(&text), "{rule}");
            let claimed = format!("verdict: {v:?},");
            assert!(text.contains(&claimed), "{rule}");
            for other in [Verdict::Pass, Verdict::Fail, Verdict::Unscored] {
                if other == v {
                    continue;
                }
                let edited = text.replacen(&claimed, &format!("verdict: {other:?},"), 1);
                assert!(
                    matches!(
                        Certificate::from_ron(&edited),
                        Err(CertificateError::EditedVerdict { .. })
                    ),
                    "{rule}: {v} edited to {other}"
                );
                assert!(ron::from_str::<Certificate>(&edited).is_err(), "{rule}");
                // Emptying the failures as well does not help.
                if !c.failures().is_empty() {
                    let start = edited.find("failures: [").unwrap();
                    let end = start + edited[start..].find("],").unwrap() + 2;
                    let blank = format!("{}failures: [],{}", &edited[..start], &edited[end..]);
                    assert!(Certificate::from_ron(&blank).is_err(), "{rule}: blanked");
                }
            }
        }
        // A certificate whose list of non-finite numbers is cleared is refused.
        let (_, c, _) = one_per_rule().remove(1);
        let text = c.to_ron();
        let path = c.nonfinite()[0].clone();
        let listed = format!("nonfinite: [\n        {path:?},\n    ],");
        assert!(text.contains(&listed), "{text}");
        let cleared = text.replacen(&listed, "nonfinite: [],", 1);
        assert_eq!(
            Certificate::from_ron(&cleared),
            Err(CertificateError::NonFinite { path })
        );
    }

    #[test]
    fn verdict_is_first_in_every_rendering() {
        // R9: verdict-first. The text opens with the verdict line, which names the tape, its
        // hash, the world and the build, then the failures; the RON opens with the verdict.
        for (rule, c, v) in one_per_rule() {
            let text = c.render();
            let first = text.lines().next().unwrap();
            assert!(
                first.starts_with(&format!("VERDICT: {v}  t  tape_hash ")),
                "{rule}: {first}"
            );
            assert!(first.contains("world_id 0x000000000000feed"), "{first}");
            assert!(
                first.ends_with("build 0123456789abcdef0123456789abcdef01234567 clean test-target"),
                "{first}"
            );
            let failures: Vec<&str> = text
                .lines()
                .skip(1)
                .take(c.failures().len())
                .map(|l| l.trim_start_matches("  - "))
                .collect();
            assert_eq!(failures, c.failures(), "{rule}");
            assert!(c
                .to_ron()
                .starts_with(&format!("Certificate(\n    verdict: {v:?},")));
        }
    }
}
