//! The toolbar's view-model (docs/GUI.md §4): the identity chip, the clock and the health chip.
//!
//! The identity names the run (U3): the build's commit and dirty flag, the `world_id`, the
//! `tape_hash` and the origin. The health gives the status, the tick's largest ledger margin and
//! the state hash; a failed run gives `Poisoned`, its ledger line and its last good tick (E5).

use crate::run::{Origin, PauseReason, RunStatus, Store};
use serde::Serialize;

/// Who the numbers belong to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IdentityVm {
    /// The tape's name.
    pub name: String,
    /// The build's commit.
    pub commit: String,
    /// Whether the build's sources differed from it.
    pub dirty: bool,
    /// `world_id`, as `0x%016x`.
    pub world_id: String,
    /// `tape_hash`, as `0x%016x`.
    pub tape_hash: String,
    /// Run or experiment.
    pub origin: Origin,
}

/// Where the run is in time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClockVm {
    /// The state's tick.
    pub tick: u64,
    /// Its date, `YYYY-MM-DD`.
    pub date: String,
    /// Ticks per year.
    pub ticks_per_year: u32,
}

/// The status a run shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Status {
    /// No tape is loaded.
    Empty,
    /// Paused.
    Paused,
    /// Running.
    Running,
    /// A step failed; the run cannot go on (E5).
    Poisoned,
    /// Ingestion stopped on a non-finite value (U10).
    Stopped,
}

/// How the run is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HealthVm {
    /// The status.
    pub status: Status,
    /// Why it paused, if it did.
    pub paused: Option<String>,
    /// The latest tick's largest ledger margin (`TickAudit::max_margin`), at most 1.
    pub max_margin: Option<f64>,
    /// The run's largest ledger margin so far (`RunAudit::max_margin`).
    pub run_margin: Option<f64>,
    /// The state's hash, as `0x%016x`.
    pub hash: Option<String>,
    /// A failed run's ledger line: the error as the engine prints it.
    pub ledger_line: Option<String>,
    /// A failed run's last good tick.
    pub last_good_tick: Option<u64>,
    /// Why ingestion stopped, if it did.
    pub stopped: Option<String>,
}

/// The toolbar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolbarVm {
    /// The identity chip, once a tape is loaded.
    pub identity: Option<IdentityVm>,
    /// The clock, once a tape is loaded.
    pub clock: Option<ClockVm>,
    /// The health chip.
    pub health: HealthVm,
}

/// The toolbar of a run: its record and its origin.
pub fn build(store: &Store, origin: Origin) -> ToolbarVm {
    let identity = store.run().zip(store.world()).map(|(k, w)| IdentityVm {
        name: w.name.clone(),
        commit: k.build.commit.clone(),
        dirty: k.build.dirty,
        world_id: k.world_id.to_string(),
        tape_hash: k.tape_hash.to_string(),
        origin,
    });
    let clock = store.world().map(|w| ClockVm {
        tick: store.tick(),
        date: w
            .clock
            .date_of(store.tick())
            .map_or_else(String::new, |d| d.to_string()),
        ticks_per_year: w.clock.ticks_per_year,
    });
    let status = match store.status() {
        RunStatus::Empty => Status::Empty,
        RunStatus::Paused { .. } => Status::Paused,
        RunStatus::Running { .. } => Status::Running,
        RunStatus::Poisoned { .. } => Status::Poisoned,
        RunStatus::Stopped => Status::Stopped,
    };
    let paused = store.paused().map(|(t, why)| match why {
        PauseReason::Reached(_) => format!("{why}"),
        _ => format!("{why} at tick {t}"),
    });
    let last = store.last_report();
    let failure = store.failure();
    let health = HealthVm {
        status,
        paused,
        max_margin: last.map(|r| r.audit.max_margin),
        run_margin: last.map(|r| r.run.max_margin),
        hash: store.run().map(|_| certify::Hex(store.hash()).to_string()),
        ledger_line: failure.map(|f| f.error.to_string()),
        last_good_tick: failure.and_then(|f| f.last_good_tick()),
        stopped: store.stopped().map(|e| e.to_string()),
    };
    ToolbarVm {
        identity,
        clock,
        health,
    }
}
