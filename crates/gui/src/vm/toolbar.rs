//! The toolbar's view-model (docs/GUI.md §4): the identity chip, the clock and the health chip.
//!
//! The identity names the run (U3): the build's commit and dirty flag, the `world_id`, the
//! `tape_hash` and the origin. The health gives the status, the tick's largest ledger margin and
//! the state hash; a failed run gives `Poisoned`, its ledger line and its last good tick (E5).
//! The controls say whether the run can take a tick and whether it is running; "run until"
//! reads a tick or a date ([`parse_until`]).

use crate::run::{LedgerCheck, Origin, PauseReason, RunStatus, Store};
use rustyecon_engine::prelude::{
    Clock, CoreError, Date, Key, Phase, RunError, RunErrorKind, World,
};
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
    /// The run's worker ended without being told to (a panic); nothing more will come.
    Ended,
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
    /// A failed run's ledger line: the error as the engine prints it, the cli's stderr, which
    /// names dense ids.
    pub ledger_line: Option<String>,
    /// The same failure by key (U7): the ids the line names, resolved against the run's world,
    /// and for a failure in the events phase the tape events due in that tick, in firing order.
    pub ledger_keys: Vec<String>,
    /// A failed run's last good tick.
    pub last_good_tick: Option<u64>,
    /// Why ingestion stopped, if it did.
    pub stopped: Option<String>,
    /// Whether the tape's ledger tolerances differ from its parent's (docs/GUI.md §5.1 item 2):
    /// a branch's parent run, or the ancestor a saved tape's lineage names. The caller knows
    /// the parent. A run with none says so, and a run whose parent is not held is unchecked,
    /// with the reason.
    pub ledger: LedgerCheck,
}

/// What the run controls may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ControlsVm {
    /// Whether the run can take a tick: loaded, not poisoned, still recorded.
    pub can_run: bool,
    /// Whether it is running now, so Space pauses it.
    pub running: bool,
}

/// The toolbar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolbarVm {
    /// The identity chip, once a tape is loaded.
    pub identity: Option<IdentityVm>,
    /// The clock, once a tape is loaded.
    pub clock: Option<ClockVm>,
    /// The run controls.
    pub controls: ControlsVm,
    /// The health chip.
    pub health: HealthVm,
}

/// What "run until" reads: a bare number is a state tick, as the cli's `--until` takes it, so
/// the run pauses before that tick runs; a date `YYYY-MM-DD` runs through the tick the date
/// falls in, so an event dated that day has fired.
pub fn parse_until(text: &str, clock: &Clock) -> Result<u64, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("a tick or a date YYYY-MM-DD".to_string());
    }
    if text.bytes().all(|b| b.is_ascii_digit()) {
        return text.parse::<u64>().map_err(|e| format!("tick {text}: {e}"));
    }
    let d = Date::parse(text).map_err(|e| format!("{text}: {e}"))?;
    let t = clock.tick_of(d).map_err(|e| format!("{text}: {e}"))?;
    t.checked_add(1)
        .ok_or_else(|| format!("{text}: past the calendar"))
}

/// A failed run's error by key (U7). The engine's line names dense ids (`pop#1`, `good#1`), as
/// the cli prints it; this reads the ids it names against the run's world: a shortfall's holder
/// and good, a hook's actor. A failure in the events phase also names the tape events due in
/// that tick, in the order they fire, one of which failed.
pub fn ledger_keys(w: &World, e: &RunError) -> Vec<String> {
    let key = |k: Option<&Key>, id: String| k.map_or(id, Key::to_string);
    let mut out = Vec::new();
    match &e.kind {
        RunErrorKind::Core(CoreError::Shortfall(l)) => {
            let what = match l.prov {
                Some(_) => "asked for",
                None => "asked to transfer",
            };
            out.push(format!(
                "by key: {} was {what} {:e} {} and held {:e}",
                super::holder(w, l.holder),
                l.requested,
                key(w.key_of(l.good), l.good.to_string()),
                l.held
            ));
        }
        RunErrorKind::Agent { actor, .. }
        | RunErrorKind::ForeignWrite { actor, .. }
        | RunErrorKind::ForeignOrder { actor, .. } => {
            out.push(format!(
                "by key: {actor} is {}",
                key(w.key_of(*actor), actor.to_string())
            ));
        }
        _ => {}
    }
    if e.phase == Phase::Events {
        let due: Vec<String> = w
            .schedule
            .fire(e.tick)
            .iter()
            .map(|f| {
                format!(
                    "{} ({})",
                    key(w.key_of(f.event), f.event.to_string()),
                    super::describe(w, &f.action)
                )
            })
            .collect();
        if !due.is_empty() {
            out.push(format!(
                "tick {}'s events, in firing order: {}",
                e.tick,
                due.join(", ")
            ));
        }
    }
    out
}

/// The toolbar of a run: its record, its origin, and whether its ledger's tolerances differ
/// from its parent's.
pub fn build(store: &Store, origin: Origin, ledger: LedgerCheck) -> ToolbarVm {
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
        RunStatus::Ended => Status::Ended,
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
        ledger_keys: failure
            .zip(store.world())
            .map_or_else(Vec::new, |(f, w)| ledger_keys(w, &f.error)),
        last_good_tick: failure.and_then(|f| f.last_good_tick()),
        stopped: store.stopped().map(|e| e.to_string()),
        ledger,
    };
    let controls = ControlsVm {
        can_run: store.can_run(),
        running: status == Status::Running,
    };
    ToolbarVm {
        identity,
        clock,
        controls,
        health,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn until_reads_a_tick_or_a_date() {
        let clock = Clock {
            start: Date::parse("1750-01-01").unwrap(),
            ticks_per_year: 52,
        };
        assert_eq!(parse_until("2080", &clock), Ok(2080));
        assert_eq!(parse_until(" 0 ", &clock), Ok(0));
        // 1760-03-01 falls in tick 528, so the run goes through it: state tick 529.
        let cut = clock.tick_of(Date::parse("1760-03-01").unwrap()).unwrap();
        assert_eq!(parse_until("1760-03-01", &clock), Ok(cut + 1));
        assert!(parse_until("", &clock).is_err());
        assert!(
            parse_until("1749-12-31", &clock).is_err(),
            "before the start"
        );
        assert!(parse_until("1760-13-01", &clock).is_err());
        assert!(parse_until("-3", &clock).is_err());
        assert!(parse_until("ten", &clock).is_err());
    }
}
