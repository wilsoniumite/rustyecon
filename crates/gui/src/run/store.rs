//! The in-memory store (docs/GUI.md §3.4): one per run, it records everything the run's
//! Runner reports. Series are f64 columns with explicit gaps: a tick a series lacks has no
//! entry, never a filler value. G3 adds chunks and the spill.
//!
//! A non-finite value stops ingestion (U10). The row that holds it is dropped whole, so every
//! series ends at the same last good tick, and the error names the series and the tick; every
//! later observation is refused with the same error.
//!
//! A load also keeps the tape and `engine::registry`'s listing of it, made once, so the
//! registry panel and the inspector read every param's uses and bases as the cli prints them.

use super::{Obs, PauseReason, Refusal, RingCheckpoint, SeriesKey, Snapshot};
use certify::RunKey;
use rustyecon_engine::prelude::*;
use rustyecon_engine::RegistryLine;
use std::collections::BTreeMap;
use std::fmt;

/// One series: the ticks it has a value at, in increasing order, and the values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Series {
    ticks: Vec<u64>,
    values: Vec<f64>,
}

impl Series {
    /// A series of these points, which must be in increasing tick order and finite.
    pub fn from_points(points: &[(u64, f64)]) -> Series {
        let mut s = Series::default();
        for &(t, v) in points {
            assert!(s.ticks.last().is_none_or(|&l| t > l), "ticks must increase");
            assert!(v.is_finite(), "values must be finite");
            s.push(t, v);
        }
        s
    }

    fn push(&mut self, tick: u64, value: f64) {
        self.ticks.push(tick);
        self.values.push(value);
    }

    /// The ticks with a value, increasing.
    pub fn ticks(&self) -> &[u64] {
        &self.ticks
    }

    /// The values, one per tick.
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// The number of points.
    pub fn len(&self) -> usize {
        self.ticks.len()
    }

    /// Whether it has no point.
    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }

    /// The last point.
    pub fn last(&self) -> Option<(u64, f64)> {
        Some((*self.ticks.last()?, *self.values.last()?))
    }

    /// The value at `tick`, if the series has one there.
    pub fn at(&self, tick: u64) -> Option<f64> {
        let i = self.ticks.binary_search(&tick).ok()?;
        Some(self.values[i])
    }

    /// The index range of the points with ticks in `[lo, hi)`.
    pub fn range(&self, lo: u64, hi: u64) -> (usize, usize) {
        let from = self.ticks.partition_point(|&t| t < lo);
        let to = self.ticks.partition_point(|&t| t < hi).max(from);
        (from, to)
    }
}

/// Why the store refused an observation. Every one stops ingestion.
#[derive(Debug, Clone, PartialEq)]
pub enum IngestError {
    /// A value that is not finite (U10): the series and the tick.
    NonFinite {
        /// The series.
        series: SeriesKey,
        /// The tick of the row that held it.
        tick: u64,
        /// The value.
        value: f64,
    },
    /// A batch that does not continue the run's ticks.
    OutOfOrder {
        /// The tick the store expected next.
        expected: u64,
        /// The tick the batch began with.
        got: u64,
    },
    /// A batch whose catalogue does not continue the store's, or a cell outside it.
    Catalogue {
        /// The store's catalogue length.
        expected: u32,
        /// The batch's.
        got: u32,
    },
    /// A batch before any load.
    NotLoaded,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestError::NonFinite {
                series,
                tick,
                value,
            } => write!(
                f,
                "a non-finite value ({value}) in {series} at tick {tick}; ingestion stopped"
            ),
            IngestError::OutOfOrder { expected, got } => write!(
                f,
                "a batch from tick {got} where tick {expected} was next; ingestion stopped"
            ),
            IngestError::Catalogue { expected, got } => write!(
                f,
                "a batch whose catalogue starts at {got} where the store's has {expected}; \
                 ingestion stopped"
            ),
            IngestError::NotLoaded => write!(f, "a batch before any load; ingestion stopped"),
        }
    }
}

impl std::error::Error for IngestError {}

/// A failed run: the error, whose `Display` is its ledger line, and the last good report.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    /// The error.
    pub error: RunError,
    /// The report of the last tick that succeeded, if any ran.
    pub last: Option<Box<TickReport>>,
}

impl Failure {
    /// The last tick that ran without error.
    pub fn last_good_tick(&self) -> Option<u64> {
        self.last.as_ref().map(|r| r.tick)
    }
}

/// Where a run stands, as its observations say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// No tape is loaded.
    Empty,
    /// Paused at `tick`: since the load (`why` is `None`), or for the reason given.
    Paused {
        /// The state's tick.
        tick: u64,
        /// Why, if not since the load.
        why: Option<PauseReason>,
    },
    /// Running; `tick` is the latest state tick recorded.
    Running {
        /// The state's tick.
        tick: u64,
    },
    /// A step failed in this tick and phase (E5).
    Poisoned {
        /// The tick that failed.
        tick: u64,
        /// The phase that failed.
        phase: Phase,
    },
    /// Ingestion stopped (U10).
    Stopped,
    /// The run's worker ended without being told to (a panic): nothing more will come.
    Ended,
}

/// Everything one run has reported.
#[derive(Debug, Clone)]
pub struct Store {
    generation: u64,
    run: Option<RunKey>,
    tape: Option<Box<Tape>>,
    world: Option<Box<World>>,
    registry: Result<Vec<RegistryLine>, String>,
    start: u64,
    tick: u64,
    hash: u64,
    status: RunStatus,
    paused: Option<(u64, PauseReason)>,
    catalogue: Vec<SeriesKey>,
    index: BTreeMap<SeriesKey, usize>,
    series: Vec<Series>,
    hashes: Vec<u64>,
    events: Vec<(u64, FiredEvent)>,
    ring: Vec<RingCheckpoint>,
    snapshots: BTreeMap<u64, Snapshot>,
    failure: Option<Failure>,
    refusals: Vec<Refusal>,
    last: Option<Box<TickReport>>,
    stopped: Option<IngestError>,
    ended: bool,
}

impl Default for Store {
    fn default() -> Store {
        Store {
            generation: 0,
            run: None,
            tape: None,
            world: None,
            registry: Ok(Vec::new()),
            start: 0,
            tick: 0,
            hash: 0,
            status: RunStatus::Empty,
            paused: None,
            catalogue: Vec::new(),
            index: BTreeMap::new(),
            series: Vec::new(),
            hashes: Vec::new(),
            events: Vec::new(),
            ring: Vec::new(),
            snapshots: BTreeMap::new(),
            failure: None,
            refusals: Vec::new(),
            last: None,
            stopped: None,
            ended: false,
        }
    }
}

impl Store {
    /// Record one observation. Once an error stops ingestion, every later observation is
    /// refused with it.
    pub fn ingest(&mut self, obs: Obs) -> Result<(), IngestError> {
        if let Some(e) = &self.stopped {
            return Err(e.clone());
        }
        let result = self.take(obs);
        if let Err(e) = &result {
            self.stopped = Some(e.clone());
            self.status = RunStatus::Stopped;
        }
        result
    }

    fn take(&mut self, obs: Obs) -> Result<(), IngestError> {
        match obs {
            Obs::Loaded {
                run,
                tape,
                world,
                tick,
                hash,
            } => {
                // A load starts the record afresh; refusals that led to it are kept. The
                // registry listing is the cli's, made once from the tape the run loaded.
                let refusals = std::mem::take(&mut self.refusals);
                let registry = rustyecon_engine::registry(&tape).map_err(|e| e.to_string());
                *self = Store {
                    generation: self.generation + 1,
                    run: Some(run),
                    tape: Some(tape),
                    world: Some(world),
                    registry,
                    start: tick,
                    tick,
                    hash,
                    status: RunStatus::Paused { tick, why: None },
                    refusals,
                    ..Store::default()
                };
            }
            Obs::Running { tick } => {
                if !self.poisoned() {
                    self.status = RunStatus::Running { tick };
                }
            }
            Obs::Batch(b) => {
                if self.run.is_none() {
                    return Err(IngestError::NotLoaded);
                }
                let known = u32::try_from(self.catalogue.len()).unwrap_or(u32::MAX);
                if b.known != known {
                    return Err(IngestError::Catalogue {
                        expected: known,
                        got: b.known,
                    });
                }
                for key in b.new_series {
                    self.index.insert(key.clone(), self.catalogue.len());
                    self.catalogue.push(key);
                    self.series.push(Series::default());
                }
                for row in b.rows {
                    let expected = self.start + self.hashes.len() as u64;
                    if row.tick != expected {
                        return Err(IngestError::OutOfOrder {
                            expected,
                            got: row.tick,
                        });
                    }
                    // Check the whole row before recording any of it (U10).
                    for &(i, v) in &row.cells {
                        let Some(key) = self.catalogue.get(i as usize) else {
                            return Err(IngestError::Catalogue {
                                expected: u32::try_from(self.catalogue.len()).unwrap_or(u32::MAX),
                                got: i,
                            });
                        };
                        if !v.is_finite() {
                            return Err(IngestError::NonFinite {
                                series: key.clone(),
                                tick: row.tick,
                                value: v,
                            });
                        }
                    }
                    for (i, v) in row.cells {
                        self.series[i as usize].push(row.tick, v);
                    }
                    self.hashes.push(row.hash);
                    self.events
                        .extend(row.events.into_iter().map(|e| (row.tick, e)));
                    self.tick = row.tick + 1;
                    self.hash = row.hash;
                }
                self.last = Some(b.last);
                if !self.poisoned() {
                    self.status = RunStatus::Running { tick: self.tick };
                }
            }
            Obs::Paused { tick, why } => {
                self.paused = Some((tick, why));
                if !self.poisoned() {
                    self.status = RunStatus::Paused {
                        tick,
                        why: Some(why),
                    };
                }
            }
            Obs::Checkpointed(cp) => self.ring.push(cp),
            Obs::Snapshot(s) => {
                self.snapshots.insert(s.tick, *s);
            }
            Obs::Failed { error, last } => {
                self.status = RunStatus::Poisoned {
                    tick: error.tick,
                    phase: error.phase,
                };
                self.failure = Some(Failure { error, last });
            }
            Obs::Ended => {
                self.ended = true;
                self.status = RunStatus::Ended;
            }
            Obs::Refused(r) => {
                if matches!(r, Refusal::Load(_)) {
                    self.status = RunStatus::Empty;
                    self.run = None;
                    self.tape = None;
                    self.world = None;
                    self.registry = Ok(Vec::new());
                }
                self.refusals.push(r);
            }
        }
        Ok(())
    }

    fn poisoned(&self) -> bool {
        self.failure.is_some()
    }

    /// Whether the run can be stepped: loaded, not poisoned, still recorded, and its worker
    /// alive.
    pub fn can_run(&self) -> bool {
        self.run.is_some() && self.failure.is_none() && self.stopped.is_none() && !self.ended
    }

    /// The run's key, once loaded.
    pub fn run(&self) -> Option<&RunKey> {
        self.run.as_ref()
    }

    /// The run's world, once loaded.
    pub fn world(&self) -> Option<&World> {
        self.world.as_deref()
    }

    /// The tape the run loaded.
    pub fn tape(&self) -> Option<&Tape> {
        self.tape.as_deref()
    }

    /// `engine::registry`'s listing of the tape, the cli's `rustyecon registry`: every param
    /// with each of its uses, then the inline numbers. Empty before a load.
    pub fn registry(&self) -> Result<&[RegistryLine], &str> {
        match &self.registry {
            Ok(lines) => Ok(lines),
            Err(e) => Err(e),
        }
    }

    /// How many loads this store has recorded. A load starts the record afresh, so a reader
    /// that keeps something derived from a series (the plot cache) starts again when it
    /// changes.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The report ticks recorded: `start..tick`. Empty before any tick ran.
    pub fn reports(&self) -> std::ops::Range<u64> {
        self.start..self.tick
    }

    /// The report tick a cursor reads: its own, brought inside the record, or with no cursor
    /// (live) the latest tick that ran. `None` before any tick ran. Report tick `t` is the tick
    /// that ran: its report's values, and the state it left, whose tick is `t + 1`.
    pub fn report_at(&self, cursor: Option<u64>) -> Option<u64> {
        let r = self.reports();
        if r.is_empty() {
            return None;
        }
        let last = r.end - 1;
        Some(cursor.map_or(last, |t| t.clamp(r.start, last)))
    }

    /// The state tick a cursor reads, for a snapshot: the state its report tick left, or the
    /// state at the load before any tick ran.
    pub fn state_at(&self, cursor: Option<u64>) -> u64 {
        self.report_at(cursor).map_or(self.start, |t| t + 1)
    }

    /// The state's tick at the load: 0 from genesis.
    pub fn start(&self) -> u64 {
        self.start
    }

    /// The latest state tick recorded.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// The hash of that state.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Where the run stands.
    pub fn status(&self) -> RunStatus {
        self.status
    }

    /// The last pause and its reason, even when the run is poisoned.
    pub fn paused(&self) -> Option<(u64, PauseReason)> {
        self.paused
    }

    /// Every series seen, by catalogue index.
    pub fn catalogue(&self) -> &[SeriesKey] {
        &self.catalogue
    }

    /// A series by key.
    pub fn series(&self, key: &SeriesKey) -> Option<&Series> {
        self.index.get(key).map(|&i| &self.series[i])
    }

    /// `TickReport::hash` of every tick recorded, from tick [`Store::start`]: entry `i` is the
    /// hash of the state whose tick is `start + i + 1`.
    pub fn hashes(&self) -> &[u64] {
        &self.hashes
    }

    /// Every fired event, with the tick it fired in.
    pub fn events(&self) -> &[(u64, FiredEvent)] {
        &self.events
    }

    /// The ring checkpoints, in tick order.
    pub fn ring(&self) -> &[RingCheckpoint] {
        &self.ring
    }

    /// The snapshot of a tick, if one was taken.
    pub fn snapshot(&self, tick: u64) -> Option<&Snapshot> {
        self.snapshots.get(&tick)
    }

    /// The run's failure, if a step failed.
    pub fn failure(&self) -> Option<&Failure> {
        self.failure.as_ref()
    }

    /// Every refusal the run's loads met.
    pub fn refusals(&self) -> &[Refusal] {
        &self.refusals
    }

    /// The report of the latest tick recorded.
    pub fn last_report(&self) -> Option<&TickReport> {
        self.last.as_deref()
    }

    /// The error that stopped ingestion, if one did.
    pub fn stopped(&self) -> Option<&IngestError> {
        self.stopped.as_ref()
    }
}
