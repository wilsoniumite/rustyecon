//! `certify`: one run from genesis, scored (docs/CERTIFY.md §6–§8). It gathers what each
//! battery reads (the base run's observations and hashes, a second run, the replay audit, the
//! resumes, the kicks) and hands each to its pure function; then the reports, the seal and the
//! manifest. It returns an error only for a base tape that does not load, a reserved key,
//! criteria that do not fit the tape, or an illustrative tape that has lost its name's marker;
//! every failure after that is inside the certificate.

use crate::battery::{self, BalanceBars, DeterminismInputs, KickBars, ResumeRun, SettlesBars};
use crate::certificate::{
    seal, BatteryId, BatteryResult, Certificate, CriteriaRef, Parts, ILLUSTRATIVE,
    ILLUSTRATIVE_BASIS,
};
use crate::criteria::{BatterySpec, Criteria, CriteriaError, Fit};
use crate::kick::{kick_segment, reserved_keys};
use crate::manifest::{tape_hash, Build, Hex, Manifest, RunKey};
use crate::obs::{kick_ticks, segment_before, segments, Names, Obs, Segment};
use rustyecon_core::Basis;
use rustyecon_engine::prelude::{audit_replay, Checkpoint, LoadError, Sim, Tape, TickReport};
use std::fmt;

/// How a run is scored.
#[derive(Debug, Clone, Copy)]
pub enum Scoring<'a> {
    /// Against registered criteria, read from `file` (its base name is recorded). The run goes
    /// to their `until`.
    Criteria {
        /// The criteria.
        criteria: &'a Criteria,
        /// Their file.
        file: &'a str,
    },
    /// With no criteria, to `until`: the certificate is UNSCORED at best (N4).
    Unscored {
        /// The state tick to run to.
        until: u64,
    },
}

/// What `certify` makes: the certificate, the manifest, and the hash file's text.
#[derive(Debug, Clone, PartialEq)]
pub struct Certified {
    /// The certificate, sealed.
    pub certificate: Certificate,
    /// The run's manifest.
    pub manifest: Manifest,
    /// The hash file: the manifest's `#` header, then one `{t} 0x{hash:016x}` line per tick.
    pub hashes: String,
}

/// Why `certify` did not run.
#[derive(Debug, Clone, PartialEq)]
pub enum CertifyError {
    /// The tape does not load.
    Load(LoadError),
    /// The tape uses a key under `certify.`, which the kick reserves.
    Reserved(Vec<String>),
    /// The criteria do not fit the tape.
    Criteria(CriteriaError),
    /// A basis says the tape is illustrative, and its name lacks the marker that keeps it
    /// unscored (R5): the entry, by path.
    Illustrative(String),
}

impl fmt::Display for CertifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertifyError::Load(e) => write!(f, "the tape does not load: {e}"),
            CertifyError::Reserved(k) => write!(
                f,
                "the tape uses keys under \"certify.\", which the kick reserves: {}",
                k.join(", ")
            ),
            CertifyError::Criteria(e) => write!(f, "the criteria do not fit the tape: {e}"),
            CertifyError::Illustrative(at) => write!(
                f,
                "{at} has an illustrative basis, and the tape's name lacks {ILLUSTRATIVE}: an \
                 illustrative tape is never scored (R5), so its name keeps the marker"
            ),
        }
    }
}

impl std::error::Error for CertifyError {}

/// The first entry of `tape` whose basis says it is illustrative (its text starts with
/// [`ILLUSTRATIVE_BASIS`]), by path: the params, the actors, genesis, the events and the
/// recurring entries, in that order.
pub fn illustrative_basis(tape: &Tape) -> Option<String> {
    let says = |b: &Basis| {
        let text = match b {
            Basis::Measured { source, .. } => source,
            Basis::Literature(s) | Basis::Approximate(s) | Basis::Assumed(s) => s,
            Basis::Fitted { fit } => fit,
        };
        text.starts_with(ILLUSTRATIVE_BASIS)
    };
    let params = tape
        .params
        .iter()
        .map(|p| (format!("params[{}]", p.key), &p.basis));
    let actors = tape
        .actors
        .iter()
        .map(|a| (format!("actors[{}]", a.key), &a.basis));
    let genesis = std::iter::once(("genesis".to_string(), &tape.genesis.basis));
    let events = tape
        .events
        .iter()
        .map(|e| (format!("events[{}]", e.key), &e.basis));
    let recurring = tape
        .recurring
        .iter()
        .map(|r| (format!("recurring[{}]", r.key), &r.basis));
    params
        .chain(actors)
        .chain(genesis)
        .chain(events)
        .chain(recurring)
        .find(|(_, b)| says(b))
        .map(|(path, _)| path)
}

/// The base run: its observations, hashes and last report, the checkpoints it kept, how far it
/// got and why it stopped.
struct Base {
    obs: Vec<Obs>,
    hashes: Vec<u64>,
    body: String,
    last: Option<TickReport>,
    kept: Vec<Checkpoint>,
    reached: u64,
    stopped: Option<String>,
}

/// Run `sim` from genesis to `until`, keeping a checkpoint at each tick of `keep` it reaches.
fn base_run(
    sim: &mut Sim,
    until: u64,
    keep: &[u64],
    manifest: &mut Manifest,
    on_tick: &mut dyn FnMut(&TickReport),
) -> Base {
    let mut b = Base {
        obs: Vec::new(),
        hashes: Vec::new(),
        body: String::new(),
        last: None,
        kept: Vec::new(),
        reached: sim.tick(),
        stopped: None,
    };
    loop {
        let t = sim.tick();
        if keep.contains(&t) {
            match sim.checkpoint() {
                Ok(cp) => b.kept.push(cp),
                Err(e) => b.stopped = Some(e.to_string()),
            }
        }
        if t >= until || b.stopped.is_some() {
            break;
        }
        match sim.step() {
            Ok(r) => {
                on_tick(&r);
                b.obs.push(Obs::of(&r, sim));
                b.hashes.push(r.hash);
                b.body.push_str(&Manifest::line(r.tick + 1, r.hash));
                manifest.tick(r.tick + 1, r.hash);
                b.reached = r.tick + 1;
                b.last = Some(r);
            }
            Err(e) => {
                b.stopped = Some(e.to_string());
                break;
            }
        }
    }
    b
}

/// A kept checkpoint at `tick`.
fn kept(b: &Base, tick: u64) -> Option<&Checkpoint> {
    b.kept.iter().find(|c| c.state().tick() == tick)
}

/// Determinism's inputs (§6): a second run, the replay audit, and each resume through the
/// checkpoint's bytes.
fn determinism_inputs(tape: &Tape, b: &Base, until: u64, resume_at: &[u64]) -> DeterminismInputs {
    let second = Sim::new(tape).map_err(|e| e.to_string()).and_then(|mut s| {
        let mut out = Vec::new();
        s.run_until(until, &mut |r| out.push(r.hash))
            .map(|()| out)
            .map_err(|e| e.to_string())
    });
    let replay = audit_replay(tape, until).map_err(|e| e.to_string());
    let resumes = resume_at
        .iter()
        .map(|&at| {
            let outcome = kept(b, at)
                .ok_or_else(|| format!("the base run kept no checkpoint at tick {at}"))
                .and_then(|cp| Checkpoint::from_bytes(&cp.to_bytes()).map_err(|e| e.to_string()))
                .and_then(|cp| Sim::resume(tape, &cp).map_err(|e| e.to_string()))
                .and_then(|mut s| {
                    let mut hashes = Vec::new();
                    let mut last = None;
                    s.run_until(until, &mut |r| {
                        hashes.push(r.hash);
                        last = Some(r.clone());
                    })
                    .map_err(|e| e.to_string())?;
                    last.map(|l| (hashes, l))
                        .ok_or_else(|| "the resumed run ran no tick".to_string())
                });
            ResumeRun { at, outcome }
        })
        .collect();
    DeterminismInputs {
        base: b.hashes.clone(),
        base_last: b.last.clone(),
        second,
        replay,
        resumes,
    }
}

/// Certify a run of `tape` from genesis (§6–§9): run it, score it, seal the certificate, and
/// record the manifest and the hash file. `on_tick` sees every report of the base run (the cli
/// feeds telemetry from it).
pub fn certify(
    tape: &Tape,
    scoring: Scoring<'_>,
    build: &Build,
    on_tick: &mut dyn FnMut(&TickReport),
) -> Result<Certified, CertifyError> {
    let reserved = reserved_keys(tape);
    if !reserved.is_empty() {
        return Err(CertifyError::Reserved(reserved));
    }
    // R5: an illustrative tape's name carries the marker the seal reads, so a tape whose bases
    // say illustrative and whose name has lost it is refused before it runs (D.4).
    if !tape.header.name.contains(ILLUSTRATIVE) {
        if let Some(at) = illustrative_basis(tape) {
            return Err(CertifyError::Illustrative(at));
        }
    }
    let mut sim = Sim::new(tape).map_err(CertifyError::Load)?;
    let world = sim.world().clone();
    let names = Names::of(&world);
    let run = RunKey {
        build: build.clone(),
        tape_hash: Hex(tape_hash(tape)),
        world_id: Hex(world.world_id),
    };
    let genesis_hash = sim.hash();
    let genesis_prices: Vec<f64> = world
        .markets()
        .map(|(n, g)| sim.price(n, g).unwrap_or(0.0))
        .collect();
    let (criteria, file, fit): (Option<&Criteria>, &str, Option<Fit>) = match scoring {
        Scoring::Criteria { criteria, file } => {
            let fit = criteria.fit(&world.clock).map_err(CertifyError::Criteria)?;
            (Some(criteria), file, Some(fit))
        }
        Scoring::Unscored { .. } => (None, "", None),
    };
    let until = match (&fit, scoring) {
        (Some(f), _) => f.until,
        (None, Scoring::Unscored { until }) => until,
        (None, Scoring::Criteria { .. }) => 0,
    };
    let segs: Vec<Segment> = match &fit {
        Some(f) => segments(&world, 0, until, f.min_segment),
        None => Vec::new(),
    };
    // The kick fires at the run's end and at every dated shock, merged or not (§7, amended at
    // S2.5), so a regime shorter than min_segment is kicked too.
    let kick_at: Vec<u64> = match criteria {
        Some(c) if c.battery(BatteryId::Kick).is_some() => kick_ticks(&world, 0, until),
        _ => Vec::new(),
    };
    let mut keep: Vec<u64> = fit.as_ref().map_or_else(Vec::new, |f| f.resume_at.clone());
    keep.extend(&kick_at);
    keep.sort_unstable();
    keep.dedup();

    let mut manifest = Manifest::begin(run.clone(), genesis_hash, &sim, None);
    let b = base_run(&mut sim, until, &keep, &mut manifest, on_tick);
    let complete = b.reached >= until && b.stopped.is_none();

    let mut batteries: Vec<BatteryResult> = vec![battery::conservation(&b.obs, b.reached, until)];
    let mut reports = Vec::new();
    if let (Some(c), Some(fit)) = (criteria, &fit) {
        for spec in &c.batteries {
            let result =
                match spec {
                    BatterySpec::Conservation => None,
                    BatterySpec::Runaway { bound } => Some(battery::runaway(
                        &b.obs,
                        &genesis_prices,
                        bound.value,
                        &names,
                    )),
                    // The rest read a whole run; one that stopped early did not run them, and the
                    // seal fails each as "did not run".
                    _ if !complete => None,
                    BatterySpec::Determinism { .. } => Some(battery::determinism(
                        &determinism_inputs(tape, &b, until, &fit.resume_at),
                    )),
                    BatterySpec::Trades { .. } => Some(battery::trades(
                        &b.obs,
                        &battery::trade_windows(until, fit.trades_every.unwrap_or(0)),
                        &names,
                    )),
                    BatterySpec::Balance {
                        level,
                        spread,
                        min_samples,
                        run_share,
                    } => Some(battery::balance(
                        &b.obs,
                        &segs,
                        &BalanceBars {
                            level: level.value,
                            spread: spread.value,
                            min_samples: min_samples.value,
                            run_share: run_share.value,
                        },
                        &names,
                    )),
                    BatterySpec::Settles {
                        w_from,
                        f_from,
                        dead_share,
                        band,
                    } => Some(battery::settles(
                        &b.obs,
                        &segs,
                        &SettlesBars {
                            w_from: w_from.value,
                            f_from: f_from.value,
                            dead_share: dead_share.value,
                            band: band.value,
                        },
                        &names,
                    )),
                    BatterySpec::Kick {
                        size,
                        max_gain,
                        max_peak,
                        ..
                    } => {
                        let bars = KickBars {
                            size: size.value,
                            horizon: fit.kick_horizon.unwrap_or(0),
                            tail: fit.kick_tail.unwrap_or(0),
                            max_gain: max_gain.value,
                            max_peak: max_peak.value,
                        };
                        let base_file = file.rsplit(['/', '\\']).next().unwrap_or(file);
                        let kicks: Vec<_> = kick_at
                            .iter()
                            .map(|&t| {
                                let k = segment_before(&segs, t).unwrap_or(usize::MAX);
                                match kept(&b, t) {
                                    Some(cp) => kick_segment(tape, &world, cp, k, &bars, base_file),
                                    None => battery::KickSegment {
                                        segment: k,
                                        at: t,
                                        runs: Err(format!(
                                            "the base run kept no checkpoint at tick {t}"
                                        )),
                                    },
                                }
                            })
                            .collect();
                        Some(battery::kick(&kicks, &bars, &names))
                    }
                };
            batteries.extend(result);
        }
        reports = battery::reports(&b.obs, &segs, c.reports.rationed_below.value, &names);
    }
    let criteria_ref = criteria.map(|c| CriteriaRef {
        file: file.rsplit(['/', '\\']).next().unwrap_or(file).to_string(),
        date: c.date,
        hash: Hex(c.hash()),
        tape_hash: c.tape.tape_hash,
        listed: c.listed(),
        price_shocks: c.allowed_price_shocks(),
    });
    let certificate = seal(Parts {
        run,
        tape: world.name.clone(),
        criteria: criteria_ref,
        until,
        reached: b.reached,
        stopped: b.stopped.clone(),
        genesis_hash: Hex(genesis_hash),
        final_hash: Hex(b.hashes.last().copied().unwrap_or(genesis_hash)),
        price_shocks: battery::price_shocks(&b.obs),
        segments: segs,
        batteries,
        reports,
    });
    let hashes = format!("{}{}", manifest.hashes_header(), b.body);
    Ok(Certified {
        certificate,
        manifest,
        hashes,
    })
}
