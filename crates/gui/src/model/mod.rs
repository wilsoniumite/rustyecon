//! The model (docs/GUI.md §3.3): every run, the session, the focus, the cursor, the selection
//! and the log, written only by [`reduce`].
//!
//! `reduce` is a pure state machine. An [`Intent`] goes in: a user's act from a panel, a file's
//! text, or a run's observation. [`Effect`]s come out: a command for a run's driver, a file to
//! read, a session to save. It does no I/O and owns no thread, clock or `Sim`; the driver host
//! carries out the effects and feeds their results back as intents.

mod session;

pub use session::{Base, Session, SESSION_FORMAT};

use crate::run::log::{self, Entry, Level};
use crate::run::{At, RunStatus};
use crate::run::{Breakpoint, Cmd, Entity, Measure, Obs, Origin, RunId, SeriesKey, Store};
use certify::{tape_hash, Hex};
use rustyecon_engine::prelude::*;
use std::collections::BTreeMap;

/// Where the panels look in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cursor {
    /// The latest tick of the focused run, following it as it runs.
    #[default]
    Live,
    /// One tick.
    At(u64),
}

/// One run: its tape, where the tape came from, its identity and what it has reported.
#[derive(Debug, Clone)]
pub struct Run {
    /// Its id in this process.
    pub id: RunId,
    /// The file the tape was read from, if any.
    pub path: Option<String>,
    /// The tape.
    pub tape: Tape,
    /// `certify::tape_hash` of the tape.
    pub tape_hash: u64,
    /// Run or experiment (D5).
    pub origin: Origin,
    /// Everything its Runner reported.
    pub store: Store,
    /// The `until` of the last `Run` command, which a change of speed repeats.
    until: Option<u64>,
}

/// What the model is told.
#[derive(Debug, Clone)]
pub enum Intent {
    /// Open a tape file.
    Open(String),
    /// A tape file's text, or why it could not be read.
    TapeRead {
        /// The path.
        path: String,
        /// Its text, or the read error.
        text: Result<String, String>,
    },
    /// `session.ron`'s text at launch, or why it could not be read.
    SessionRead(Result<String, String>),
    /// A run's observation.
    Observed {
        /// The run.
        run: RunId,
        /// What it reported.
        obs: Obs,
    },
    /// Run the focused run on if it is paused, or pause it if it is running (Space).
    RunPause,
    /// Run the focused run until the state's tick is `until`, or on.
    Run {
        /// Where to pause.
        until: Option<u64>,
    },
    /// Pause the focused run.
    Pause,
    /// Step the focused run this many ticks (`.` steps one).
    Step(u64),
    /// Step the focused run one model year.
    StepYear,
    /// Set the speed cap, in ticks per second.
    Speed(Option<u32>),
    /// Set or clear the breakpoint on error, for every run.
    BreakOnError(bool),
    /// Ask the focused run for a snapshot of a tick.
    Snapshot(u64),
    /// Focus a run.
    Focus(RunId),
    /// Select an entity, or clear the selection.
    Select(Option<Entity>),
    /// Move the cursor.
    Cursor(Cursor),
    /// Plot a series.
    Plot(SeriesKey),
    /// Stop plotting a series.
    Unplot(SeriesKey),
    /// Pin an entity.
    Pin(Entity),
    /// Unpin an entity.
    Unpin(Entity),
    /// Close a run.
    Close(RunId),
    /// A line for the log from the host.
    Note(String),
    /// A file operation failed.
    FileFailed {
        /// What was being done.
        what: String,
        /// Why it failed.
        why: String,
    },
}

/// What the model asks for.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Read a tape file and answer with [`Intent::TapeRead`].
    ReadTape(String),
    /// Start a driver for a new run.
    Spawn(RunId),
    /// Send a run's driver a command.
    Send {
        /// The run.
        run: RunId,
        /// The command.
        cmd: Cmd,
    },
    /// Stop a run's driver and drop it.
    Close(RunId),
    /// Write `session.ron`.
    SaveSession,
    /// Move a `session.ron` that does not read aside, so that no save overwrites it.
    SetAsideSession,
}

/// The whole model.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// What `session.ron` keeps.
    pub session: Session,
    runs: BTreeMap<RunId, Run>,
    focus: Option<RunId>,
    cursor: Cursor,
    selection: Option<Entity>,
    log: Vec<Entry>,
    next: u32,
}

impl Model {
    /// A model with this session.
    pub fn with_session(session: Session) -> Model {
        Model {
            session,
            ..Model::default()
        }
    }

    /// Every run, by id.
    pub fn runs(&self) -> impl Iterator<Item = &Run> {
        self.runs.values()
    }

    /// One run.
    pub fn run(&self, id: RunId) -> Option<&Run> {
        self.runs.get(&id)
    }

    /// The focused run's id.
    pub fn focus(&self) -> Option<RunId> {
        self.focus
    }

    /// The focused run.
    pub fn focused(&self) -> Option<&Run> {
        self.focus.and_then(|id| self.runs.get(&id))
    }

    /// The cursor.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// The selection.
    pub fn selection(&self) -> Option<&Entity> {
        self.selection.as_ref()
    }

    /// The log, oldest first.
    pub fn log(&self) -> &[Entry] {
        &self.log
    }

    fn note(&mut self, level: Level, text: String) {
        self.log.push(Entry::note(level, text));
    }
}

/// The only writer of the model. Every intent is handled here, and every effect it needs comes
/// back to the caller; nothing else changes the model.
pub fn reduce(m: &mut Model, i: Intent) -> Vec<Effect> {
    match i {
        Intent::Open(path) => vec![Effect::ReadTape(path)],
        Intent::TapeRead { path, text } => open(m, path, text),
        Intent::SessionRead(Ok(text)) => match Session::from_ron(&text) {
            Ok(s) => {
                m.session = s;
                Vec::new()
            }
            Err(e) => {
                m.note(
                    Level::Error,
                    format!("session.ron does not read ({e}); starting a new session"),
                );
                vec![Effect::SetAsideSession]
            }
        },
        Intent::SessionRead(Err(e)) => {
            m.note(Level::Error, format!("session.ron cannot be read: {e}"));
            Vec::new()
        }
        Intent::Observed { run, obs } => observed(m, run, obs),
        Intent::RunPause => match m.focused().map(|r| r.store.status()) {
            Some(RunStatus::Running { .. }) => focused_cmd(m, Cmd::Pause),
            Some(RunStatus::Paused { .. }) => run_focused(m, None),
            _ => Vec::new(),
        },
        Intent::Run { until } => run_focused(m, until),
        Intent::Pause => match m.focused().map(|r| r.store.status()) {
            Some(RunStatus::Running { .. }) => focused_cmd(m, Cmd::Pause),
            _ => Vec::new(),
        },
        Intent::Step(n) => {
            if n > 0 && m.focused().is_some_and(|r| r.store.can_run()) {
                focused_cmd(m, Cmd::Step(n))
            } else {
                Vec::new()
            }
        }
        Intent::StepYear => {
            let year = m
                .focused()
                .and_then(|r| r.store.world())
                .map(|w| u64::from(w.clock.ticks_per_year));
            match year {
                Some(n) => reduce(m, Intent::Step(n)),
                None => Vec::new(),
            }
        }
        Intent::Speed(v) => {
            m.session.speed = v;
            let mut out = vec![Effect::SaveSession];
            if let Some(r) = m.focused() {
                if matches!(r.store.status(), RunStatus::Running { .. }) {
                    let until = r.until;
                    out.extend(focused_cmd(m, Cmd::Run { until, max_tps: v }));
                }
            }
            out
        }
        Intent::BreakOnError(on) => {
            let b = &mut m.session.breakpoints;
            b.retain(|x| *x != Breakpoint::OnError);
            if on {
                b.push(Breakpoint::OnError);
                b.sort();
            }
            let mut out = vec![Effect::SaveSession];
            for id in m.runs.keys() {
                out.push(Effect::Send {
                    run: *id,
                    cmd: Cmd::Breakpoints(m.session.breakpoints.clone()),
                });
            }
            out
        }
        Intent::Snapshot(t) => focused_cmd(m, Cmd::Snapshot(t)),
        Intent::Focus(id) => {
            if m.runs.contains_key(&id) {
                m.focus = Some(id);
            }
            Vec::new()
        }
        Intent::Select(e) => {
            m.selection = e;
            Vec::new()
        }
        Intent::Cursor(c) => {
            m.cursor = c;
            Vec::new()
        }
        Intent::Plot(k) => {
            if m.session.plots.contains(&k) {
                return Vec::new();
            }
            m.session.plots.push(k);
            vec![Effect::SaveSession]
        }
        Intent::Unplot(k) => {
            let before = m.session.plots.len();
            m.session.plots.retain(|p| *p != k);
            changed(before != m.session.plots.len())
        }
        Intent::Pin(e) => {
            if m.session.pins.contains(&e) {
                return Vec::new();
            }
            m.session.pins.push(e);
            vec![Effect::SaveSession]
        }
        Intent::Unpin(e) => {
            let before = m.session.pins.len();
            m.session.pins.retain(|p| *p != e);
            changed(before != m.session.pins.len())
        }
        Intent::Close(id) => close(m, id),
        Intent::Note(text) => {
            m.note(Level::Info, text);
            Vec::new()
        }
        Intent::FileFailed { what, why } => {
            m.note(Level::Error, format!("{what}: {why}"));
            Vec::new()
        }
    }
}

/// `SaveSession` if the session changed.
fn changed(yes: bool) -> Vec<Effect> {
    if yes {
        vec![Effect::SaveSession]
    } else {
        Vec::new()
    }
}

/// Send the focused run a command.
fn focused_cmd(m: &Model, cmd: Cmd) -> Vec<Effect> {
    match m.focus {
        Some(run) => vec![Effect::Send { run, cmd }],
        None => Vec::new(),
    }
}

/// Run the focused run on, until `until`, at the session's speed, if it can run.
fn run_focused(m: &mut Model, until: Option<u64>) -> Vec<Effect> {
    let max_tps = m.session.speed;
    let Some(r) = m.focus.and_then(|id| m.runs.get_mut(&id)) else {
        return Vec::new();
    };
    if !r.store.can_run() {
        return Vec::new();
    }
    r.until = until;
    vec![Effect::Send {
        run: r.id,
        cmd: Cmd::Run { until, max_tps },
    }]
}

/// A tape file's text: parse it, record it as a base, and start its run paused at genesis.
fn open(m: &mut Model, path: String, text: Result<String, String>) -> Vec<Effect> {
    let text = match text {
        Ok(t) => t,
        Err(e) => {
            m.note(Level::Error, format!("cannot read the tape {path}: {e}"));
            return Vec::new();
        }
    };
    let tape = match Tape::from_ron(&text) {
        Ok(t) => t,
        Err(e) => {
            m.note(Level::Error, format!("the tape {path} does not load: {e}"));
            return Vec::new();
        }
    };
    let hash = tape_hash(&tape);
    let mut out = Vec::new();
    match m.session.bases.iter_mut().find(|b| b.path == path) {
        Some(b) if b.tape_hash.0 != hash => {
            m.log.push(Entry::note(
                Level::Info,
                format!(
                    "the tape {path} changed on disk since the session recorded it ({} became \
                     {}); it opens as a new root run",
                    b.tape_hash,
                    Hex(hash)
                ),
            ));
            b.tape_hash = Hex(hash);
            out.push(Effect::SaveSession);
        }
        Some(_) => {}
        None => {
            m.session.bases.push(Base {
                path: path.clone(),
                tape_hash: Hex(hash),
            });
            out.push(Effect::SaveSession);
        }
    }
    let id = RunId(m.next);
    m.next += 1;
    let run = Run {
        id,
        path: Some(path),
        origin: Origin::of(&tape),
        tape_hash: hash,
        tape: tape.clone(),
        store: Store::default(),
        until: None,
    };
    m.runs.insert(id, run);
    m.focus = Some(id);
    m.cursor = Cursor::Live;
    out.extend([
        Effect::Spawn(id),
        Effect::Send {
            run: id,
            cmd: Cmd::Breakpoints(m.session.breakpoints.clone()),
        },
        Effect::Send {
            run: id,
            cmd: Cmd::Load {
                tape: Box::new(tape),
                from: None,
            },
        },
    ]);
    out
}

/// A run's observation: log it, record it, and react. A run that is closed is ignored.
fn observed(m: &mut Model, id: RunId, obs: Obs) -> Vec<Effect> {
    if !m.runs.contains_key(&id) {
        return Vec::new();
    }
    m.log.extend(log::entries(id, &obs));
    let mut out = Vec::new();
    if let Obs::Loaded { world, .. } = &obs {
        // A new session plots every price (docs/GUI.md §4).
        if m.session.plots.is_empty() {
            for (n, g) in world.markets() {
                if let (Some(node), Some(good)) = (world.key_of(n), world.key_of(g)) {
                    m.session.plots.push(SeriesKey {
                        measure: Measure::Price,
                        at: At::Market {
                            node: node.clone(),
                            good: good.clone(),
                        },
                    });
                }
            }
            out.push(Effect::SaveSession);
        }
    }
    let run = m.runs.get_mut(&id).expect("checked above");
    let was_stopped = run.store.stopped().is_some();
    if let Err(e) = run.store.ingest(obs) {
        if !was_stopped {
            m.log.push(Entry {
                run: Some(id),
                tick: None,
                level: Level::Error,
                text: e.to_string(),
            });
            // The record stopped, so the run stops too.
            out.push(Effect::Send {
                run: id,
                cmd: Cmd::Pause,
            });
        }
    }
    out
}

/// Close a run: drop its driver, and its base from the session unless another run reads it.
fn close(m: &mut Model, id: RunId) -> Vec<Effect> {
    let Some(run) = m.runs.remove(&id) else {
        return Vec::new();
    };
    let mut out = vec![Effect::Close(id)];
    if let Some(path) = &run.path {
        let shared = m.runs.values().any(|r| r.path.as_ref() == Some(path));
        if !shared {
            m.session.bases.retain(|b| &b.path != path);
            out.push(Effect::SaveSession);
        }
    }
    if m.focus == Some(id) {
        m.focus = m.runs.keys().next_back().copied();
    }
    out
}
