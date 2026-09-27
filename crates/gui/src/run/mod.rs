//! The run's side of the seam (docs/GUI.md §3.3, §3.4): commands in, observations out.
//!
//! - [`Runner`] owns the one `Sim` of a run (U1). It takes a [`Cmd`], steps when
//!   [`Runner::advance`] is called, extracts each tick, keeps the ring of checkpoints, and hands
//!   each [`Obs`] to its sink. It has no clock and no thread: a driver calls it (E2).
//! - [`Extractor`] turns a `Sim` and its `TickReport` into one [`Row`] of values named by tape
//!   key ([`SeriesKey`], U7), plus the tick's hash.
//! - [`Store`] records a run's observations in memory: f64 columns with explicit gaps, the hash
//!   stream, fired events, the ring and the run's status. A non-finite value stops it (U10).
//! - [`Decimator`] thins a series for drawing and keeps every column's true minimum and maximum.
//! - [`log`] turns observations into log lines.
//!
//! Nothing here uses egui, a clock, a thread or a file. Pause, speed and seek change when the
//! Runner is called, never what it computes: every path through it steps the `Sim` one tick at a
//! time, so a GUI run hashes as the cli's run of the same tape does (U4).

mod decimate;
mod extract;
pub mod log;
mod ring;
mod runner;
mod store;

pub use decimate::{decimate, Decimator};
pub use extract::{
    state_fields, At, Catalogue, Entity, Extractor, HolderKey, Measure, SeriesKey, StateField,
};
pub use ring::ring_tick_at_or_after;
pub use runner::{Progress, Runner};
pub use store::{Failure, IngestError, RunStatus, Series, Store};

use certify::RunKey;
use rustyecon_engine::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A run in this process: a number the model gives each run it opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RunId(pub u32);

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "run {}", self.0)
    }
}

/// The marker a GUI edit leaves in a tape's name and in every basis it stamps (D3, U2):
/// `[GUI experiment <date>]` and `Assumed("GUI experiment <date>: <note>")`.
pub const EXPERIMENT_MARKER: &str = "GUI experiment";

/// Where a number comes from (D5, U3): a run of a tape, an experiment, the record or the
/// oracle. The last two arrive with their phases (G5, G1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// A run of a tape that no GUI edit made.
    Run,
    /// A run of a tape a GUI edit made: its name or a basis carries the marker. From G0.2, a
    /// tape with a lineage is one too.
    Experiment,
}

impl Origin {
    /// A run's origin, from its tape: an experiment when its canonical text, which holds the
    /// name and every basis and nothing else in free text, carries [`EXPERIMENT_MARKER`].
    pub fn of(tape: &Tape) -> Origin {
        if tape.to_ron().contains(EXPERIMENT_MARKER) {
            Origin::Experiment
        } else {
            Origin::Run
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Origin::Run => write!(f, "run"),
            Origin::Experiment => write!(f, "experiment"),
        }
    }
}

/// Whether a run's ledger tolerances differ from its parent's (docs/GUI.md §5.1 item 2): the
/// params `header.ledger` names, by key, value and unit. The model compares them. A run whose
/// parent this session cannot read says so, and never says "no".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerCheck {
    /// A run with no parent to compare with: a tape no GUI edit made, with no lineage.
    NoParent,
    /// The tolerances equal the parent's: the parent run's, or those of the ancestor the
    /// lineage names.
    Same,
    /// They differ: the tape was edited by hand.
    Changed,
    /// Not checked, and why: no parent run is open and the ancestor the lineage names is not
    /// held, or the tape is an experiment and no lineage names its parent.
    Unknown(String),
}

impl LedgerCheck {
    /// Whether the tolerances changed, when the check can say: `Some(false)` for a run with
    /// no parent, and `None` when it is unchecked.
    pub fn changed(&self) -> Option<bool> {
        match self {
            LedgerCheck::NoParent | LedgerCheck::Same => Some(false),
            LedgerCheck::Changed => Some(true),
            LedgerCheck::Unknown(_) => None,
        }
    }
}

impl fmt::Display for LedgerCheck {
    /// As the export's `#` line writes it, after "ledger changed: ".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerCheck::NoParent => write!(f, "no (no parent: a tape no GUI edit made)"),
            LedgerCheck::Same => write!(f, "no"),
            LedgerCheck::Changed => write!(f, "yes"),
            LedgerCheck::Unknown(why) => write!(f, "unknown ({why})"),
        }
    }
}

/// What the model asks a Runner to do. None of these changes what a tick computes.
#[derive(Debug, Clone)]
pub enum Cmd {
    /// Load a tape: from its genesis, or resumed from a ring checkpoint a Runner of this process
    /// took (D4). A refused resume is logged and the tape reruns from genesis.
    Load {
        /// The tape.
        tape: Box<Tape>,
        /// Where to start, if not from genesis.
        from: Option<ResumeFrom>,
    },
    /// Run until the state's tick is `until`, or on without end. The driver applies the speed
    /// cap `max_tps`, in ticks per second; the Runner has no clock and ignores it.
    Run {
        /// Pause when the state's tick reaches this.
        until: Option<u64>,
        /// The speed cap, in ticks per second.
        max_tps: Option<u32>,
    },
    /// Pause at the end of the current slice.
    Pause,
    /// Run this many ticks, then pause.
    Step(u64),
    /// Replace the breakpoints.
    Breakpoints(Vec<Breakpoint>),
    /// What the next load records (D.3): G0's whole catalogue unless told otherwise. The
    /// model sends the lean one before it loads a world too large to record whole.
    Catalogue(Catalogue),
    /// Report the state at this tick: now, when the run reaches it, or from a scratch `Sim`
    /// resumed at the latest ring checkpoint at or before it (deep inspection).
    Snapshot(u64),
    /// Drop the run; the driver's worker ends.
    Stop,
}

/// Where a load starts, when not at genesis. At G0 only a ring checkpoint; files arrive at G3,
/// and only once verified (D4).
#[derive(Debug, Clone)]
pub enum ResumeFrom {
    /// A checkpoint a Runner of this process took.
    Ring(RingCheckpoint),
}

/// A checkpoint of the ring (D4): the run's key, the state's tick and `Checkpoint::to_bytes`.
/// Opaque. Only a Runner makes one, so a resume from it starts from a state a Runner of this
/// process reached, never from a number read from outside the tape (U1).
#[derive(Clone)]
pub struct RingCheckpoint {
    run: RunKey,
    tick: u64,
    prefix_id: u64,
    bytes: Vec<u8>,
}

impl RingCheckpoint {
    /// The run that took it.
    pub fn run(&self) -> &RunKey {
        &self.run
    }

    /// The state's tick.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// The `prefix_id` the checkpoint stores, the schedule's firings before its tick, read off
    /// the checkpoint when the Runner took it. `edit::plan` compares it with a branch's; the
    /// resume itself checks the bytes (`Sim::resume`).
    pub fn prefix_id(&self) -> u64 {
        self.prefix_id
    }

    /// The size of its bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether it holds no bytes (never, for one a Runner took).
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// The checkpoint it holds, decoded; its digest is checked on the way (ENGINE §7.6).
    pub fn checkpoint(&self) -> Result<Checkpoint, CheckpointError> {
        Checkpoint::from_bytes(&self.bytes)
    }
}

impl fmt::Debug for RingCheckpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RingCheckpoint")
            .field("run", &self.run)
            .field("tick", &self.tick)
            .field("prefix_id", &self.prefix_id)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// A condition that pauses a run. At G0 the only one is an error; event and date breakpoints
/// join at G1, conditions at G2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Breakpoint {
    /// Pause on a run error. A failed `Sim` is poisoned and stops either way (E5); with this
    /// set, the pause is reported as the breakpoint's.
    OnError,
}

/// Why a run paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PauseReason {
    /// A `Pause` command.
    Asked,
    /// A `Step` ran its ticks.
    Stepped,
    /// A `Run` reached its `until`.
    Reached(u64),
    /// A breakpoint fired.
    Breakpoint(Breakpoint),
    /// A step failed with no breakpoint set; the run is poisoned.
    Failed,
}

impl fmt::Display for PauseReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PauseReason::Asked => write!(f, "paused"),
            PauseReason::Stepped => write!(f, "stepped"),
            PauseReason::Reached(t) => write!(f, "reached tick {t}"),
            PauseReason::Breakpoint(Breakpoint::OnError) => write!(f, "breakpoint on error"),
            PauseReason::Failed => write!(f, "stopped by a run error"),
        }
    }
}

/// The state at one tick, read without changing it (E4): every holder's totals and lots, and
/// every actor's own state. Ids are dense, read against the run's `World`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The state's tick.
    pub tick: u64,
    /// The state's hash.
    pub hash: u64,
    /// Every holder's total of every good it holds.
    pub holdings: HoldingTotals,
    /// The lots behind each total, in the same (holder, good) order, as the inventory keeps
    /// them.
    pub lots: Vec<(Holder, GoodId, Vec<Lot>)>,
    /// Every actor's own state, in `ActorId` order.
    pub actors: Vec<(ActorId, ActorState)>,
}

/// Why a load did not start where it was asked to.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// The tape does not load; the run has no `Sim`.
    Load(LoadError),
    /// A ring checkpoint's bytes do not decode; the tape reruns from genesis.
    Decode(CheckpointError),
    /// `Sim::resume` refused the checkpoint under this tape; the tape reruns from genesis.
    Resume(ResumeError),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Load(e) => write!(f, "the tape does not load: {e}"),
            Refusal::Decode(e) => {
                write!(
                    f,
                    "the checkpoint does not decode ({e}); rerunning from genesis"
                )
            }
            Refusal::Resume(e) => write!(f, "the resume is refused ({e}); rerunning from genesis"),
        }
    }
}

/// One tick's extracted values: the report's tick, the state hash after it, a value per series
/// that exists this tick (by catalogue index), and the events that fired.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The tick that ran.
    pub tick: u64,
    /// The hash of the state after it, whose tick is `tick + 1` (`TickReport::hash`).
    pub hash: u64,
    /// `(catalogue index, value)`, one per series this tick has. A series a tick lacks is a gap.
    pub cells: Vec<(u32, f64)>,
    /// The tape events that fired, in firing order.
    pub events: Vec<FiredEvent>,
}

/// One slice's rows, in tick order, with the catalogue entries they introduce.
#[derive(Debug, Clone, PartialEq)]
pub struct ObsBatch {
    /// The catalogue's length before this batch: `new_series[i]` is entry `known + i`.
    pub known: u32,
    /// The series first seen in this batch.
    pub new_series: Vec<SeriesKey>,
    /// One row per tick, consecutive.
    pub rows: Vec<Row>,
    /// The report of the last tick in the batch.
    pub last: Box<TickReport>,
}

/// What a Runner reports.
#[derive(Debug, Clone)]
pub enum Obs {
    /// A tape loaded; the run is paused at `tick`.
    Loaded {
        /// The run's key: build, `tape_hash` and `world_id` (certify's `RunKey`).
        run: RunKey,
        /// The tape the run loaded, for the registry listing and the events' bases.
        tape: Box<Tape>,
        /// The run's world, for key lookups on this side of the channel.
        world: Box<World>,
        /// The state's tick: 0 from genesis, the checkpoint's on a resume.
        tick: u64,
        /// The state's hash.
        hash: u64,
    },
    /// The run started running from `tick`.
    Running {
        /// The state's tick.
        tick: u64,
    },
    /// One slice's ticks.
    Batch(ObsBatch),
    /// The run paused.
    Paused {
        /// The state's tick.
        tick: u64,
        /// Why.
        why: PauseReason,
    },
    /// A ring checkpoint was taken.
    Checkpointed(RingCheckpoint),
    /// A snapshot that was asked for.
    Snapshot(Box<Snapshot>),
    /// A step failed; the `Sim` is poisoned (E5).
    Failed {
        /// The error, whose `Display` is its ledger line.
        error: RunError,
        /// The report of the last good tick, if any tick ran.
        last: Option<Box<TickReport>>,
    },
    /// A load was refused, in whole or in part.
    Refused(Refusal),
    /// The run's worker ended without being told to stop: its Runner panicked, and a Runner
    /// that panicked says nothing more. A driver reports it once; a Runner never does. The run
    /// can go no further.
    Ended,
}
