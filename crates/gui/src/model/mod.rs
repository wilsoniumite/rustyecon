//! The model (docs/GUI.md §3.3): every run, the session, the focus, the cursor, the selection,
//! the editor and the log, written only by [`reduce`].
//!
//! `reduce` is a pure state machine. An [`Intent`] goes in: a user's act from a panel, a file's
//! text, or a run's observation. [`Effect`]s come out: a command for a run's driver, a file to
//! read or write, a session to save. It does no I/O and owns no thread, clock or `Sim`; the
//! driver host carries out the effects and feeds their results back as intents.
//!
//! Branches (§5.1, D3): the editor stages [`TapeEdit`]s read from its form, and Apply
//! materialises them on the focused run ([`edit::materialise`]). The branch is a new run with
//! that run as its parent. [`edit::plan`] picks its start, and its Runner receives the
//! parent's ring checkpoint in `Cmd::Load`, or reruns from genesis and the log says why. The
//! branch's lineage is made with it: the nearest ancestor on disk and every edit since.

mod session;

pub use session::{Base, Session, SESSION_FORMAT};

use crate::edit::export::{self, Source};
use crate::edit::{
    self, keys, lineage_path, Ancestor, Applied, EditError, EditOp, Form, FormError, Lineage, Plan,
    TapeEdit,
};
use crate::run::log::{self, Entry, Level, RationWatch};
use crate::run::{At, RunStatus};
use crate::run::{
    Breakpoint, Cmd, Entity, Measure, Obs, Origin, ResumeFrom, RingCheckpoint, RunId, SeriesKey,
    Store,
};
use certify::{tape_hash, Hex};
use rustyecon_engine::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

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
    /// The file the tape was read from or saved to, if any: a run with a path is on disk,
    /// and is its branches' nearest ancestor there.
    pub path: Option<String>,
    /// The tape.
    pub tape: Tape,
    /// `certify::tape_hash` of the tape.
    pub tape_hash: u64,
    /// Run or experiment (D5).
    pub origin: Origin,
    /// Everything its Runner reported.
    pub store: Store,
    /// The run it branched from, in this process.
    pub parent: Option<RunId>,
    /// The edits that made its tape from its parent's, as applied.
    pub applied: Vec<Applied>,
    /// Its lineage: made with the branch, or read beside the opened file.
    pub lineage: Option<Lineage>,
    /// The parent's ring checkpoint the branch was told to resume from, if `plan` found one.
    pub resume: Option<RingCheckpoint>,
    /// Why the branch reruns from genesis, if `plan` found no checkpoint.
    pub rerun: Option<String>,
    /// The tape of its nearest ancestor on disk, which an export carries.
    ancestor: Option<Box<Tape>>,
    /// The `until` of the last `Run` command, which a change of speed repeats.
    until: Option<u64>,
    /// The state ticks a snapshot was asked for since the last load, so each is asked once.
    asked: BTreeSet<u64>,
    /// The class lines being rationed, for the log's onsets.
    watch: RationWatch,
}

impl Run {
    fn new(id: RunId, path: Option<String>, tape: Tape, origin: Origin) -> Run {
        Run {
            id,
            path,
            tape_hash: tape_hash(&tape),
            tape,
            origin,
            store: Store::default(),
            parent: None,
            applied: Vec::new(),
            lineage: None,
            resume: None,
            rerun: None,
            ancestor: None,
            until: None,
            asked: BTreeSet::new(),
            watch: RationWatch::default(),
        }
    }

    /// The tape of its nearest ancestor on disk, when this session holds it.
    pub fn ancestor(&self) -> Option<&Tape> {
        self.ancestor.as_deref()
    }
}

/// An act the raw pane shows: the text the form read and where the parser stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawError {
    /// The act's text, as read.
    pub text: String,
    /// The line, from 1.
    pub line: usize,
    /// The column, from 1.
    pub col: usize,
    /// The parser's message.
    pub message: String,
}

/// The editor: the edits waiting for Apply, and what was refused.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Editor {
    /// The edits Apply will materialise, in order.
    pub staged: Vec<TapeEdit>,
    /// Why the last form or Apply was refused.
    pub error: Option<String>,
    /// The params the staged removals leave unreferenced: the editor offers a `RemoveParam`
    /// for each, with its own note.
    pub offer: Vec<Key>,
    /// An act that did not read, for the raw pane.
    pub raw: Option<RawError>,
}

/// A file job: what a write was for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    /// "Save tape as": a run's tape and its lineage.
    SaveTape {
        /// The run.
        run: RunId,
        /// The tape's path.
        path: String,
    },
    /// An export of a run into a directory.
    Export {
        /// The run.
        run: RunId,
        /// The directory.
        dir: String,
    },
}

/// What the model is told.
#[derive(Debug, Clone)]
pub enum Intent {
    /// Ask the user for a tape file to open (the toolbar's Open).
    PickTape,
    /// Open a tape file.
    Open(String),
    /// A tape file's text, or why it could not be read, and the text of the lineage file
    /// beside it, if there is one.
    TapeRead {
        /// The path.
        path: String,
        /// Its text, or the read error.
        text: Result<String, String>,
        /// `<name>.lineage.ron`'s text, or why it could not be read; `None` when there is none.
        lineage: Option<Result<String, String>>,
    },
    /// `session.ron`'s text at launch, or why it could not be read.
    SessionRead(Result<String, String>),
    /// Today's date: the date an experiment is stamped with. The host sends it at launch.
    Today(Date),
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
    /// Read the editor's form into an edit of the focused run and stage it.
    Stage(Form),
    /// Drop a staged edit.
    Unstage(usize),
    /// Materialise the staged edits on the focused run: a branch.
    Apply,
    /// Stage a `RemoveParam` with this note for each param the offer names, and apply.
    RemoveOrphans {
        /// The note: required.
        note: String,
    },
    /// Ask the user where to save the focused run's tape.
    PickSaveTape,
    /// Save the focused run's tape, and its lineage beside it. No file is written over.
    SaveTape(String),
    /// Ask the user for a directory to export the focused run into.
    PickExportDir,
    /// Export the focused run into a directory: the plotted series, the manifest, the tapes
    /// and the lineage. No file is written over.
    Export(String),
    /// A file job's files were written, or why not.
    Written {
        /// The job.
        job: Job,
        /// The paths written, or why they were not.
        result: Result<Vec<String>, String>,
    },
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
    /// Ask the user for a tape file, and answer with [`Intent::Open`] if one is chosen.
    PickTape,
    /// Read a tape file, and the lineage beside it, and answer with [`Intent::TapeRead`].
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
    /// Ask the user for a path to save a tape to, and answer with [`Intent::SaveTape`].
    PickSaveTape,
    /// Ask the user for a directory, and answer with [`Intent::Export`].
    PickExportDir,
    /// Write these files, `(path, text)`, none over an existing file, and answer with
    /// [`Intent::Written`].
    Write {
        /// What they are for.
        job: Job,
        /// The files.
        files: Vec<(String, String)>,
    },
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
    today: Option<Date>,
    editor: Editor,
    /// The keys of every run closed in this session, never minted again (E8).
    retired: BTreeSet<String>,
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

    /// The date experiments are stamped with, once the host has said.
    pub fn today(&self) -> Option<Date> {
        self.today
    }

    /// The editor.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// A run's parent, when it is a branch whose parent is still open.
    pub fn parent_of(&self, id: RunId) -> Option<&Run> {
        self.runs.get(&id)?.parent.and_then(|p| self.runs.get(&p))
    }

    /// The runs of `id`'s tree: its root (the first ancestor with no open parent) and every
    /// open run that descends from that root.
    pub fn tree(&self, id: RunId) -> Vec<RunId> {
        let root = |mut r: RunId| {
            while let Some(p) = self.runs.get(&r).and_then(|x| x.parent) {
                if !self.runs.contains_key(&p) {
                    break;
                }
                r = p;
            }
            r
        };
        let top = root(id);
        self.runs
            .keys()
            .copied()
            .filter(|r| root(*r) == top)
            .collect()
    }

    /// Every key a new entry of a branch of `id` may not take (E8): each key of every tape in
    /// its run tree, siblings and ancestors included, of the staged edits, and of every run
    /// closed in this session.
    pub fn taken_keys(&self, id: RunId) -> BTreeSet<String> {
        let mut out = self.retired.clone();
        for r in self.tree(id) {
            if let Some(run) = self.runs.get(&r) {
                out.extend(keys::tape_keys(&run.tape));
            }
        }
        out.extend(keys::added_keys(&self.editor.staged));
        out
    }

    /// A key for a new entry of a branch of the focused run: `gui.<serial>.<n>`, new to its
    /// run tree and to the staged edits.
    pub fn mint_key(&self) -> Option<Key> {
        let id = self.focus?;
        Some(keys::mint(self.session.serial, &self.taken_keys(id)))
    }

    /// Whether a run's ledger tolerances differ from its parent's: the branch's parent run, or
    /// the ancestor its lineage names when this session holds that tape (§5.1 item 2).
    pub fn ledger_changed(&self, id: RunId) -> bool {
        let Some(r) = self.runs.get(&id) else {
            return false;
        };
        if let Some(p) = self.parent_of(id) {
            return edit::ledger_changed(&r.tape, &p.tape);
        }
        let Some(l) = &r.lineage else {
            return false;
        };
        self.runs
            .values()
            .find(|a| {
                a.id != id
                    && a.path.as_deref() == Some(l.parent.path.as_str())
                    && a.tape_hash == l.parent.tape_hash.0
            })
            .is_some_and(|a| edit::ledger_changed(&r.tape, &a.tape))
    }

    /// The export of a run, as text: its files, by name in the export directory.
    pub fn export_files(&self, id: RunId) -> Result<Vec<(String, String)>, String> {
        let r = self.runs.get(&id).ok_or("no such run")?;
        let resumed = r
            .resume
            .as_ref()
            .and_then(|cp| export::resumed_from(cp, &r.store));
        let src = Source {
            store: &r.store,
            plots: &self.session.plots,
            origin: r.origin,
            resumed_from: resumed.as_ref(),
            lineage: r.lineage.as_ref(),
            ancestor: r.ancestor.as_deref(),
            ledger_changed: self.ledger_changed(id),
        };
        export::files(&src)
    }

    fn note(&mut self, level: Level, text: String) {
        self.log.push(Entry::note(level, text));
    }

    fn refuse(&mut self, why: String) -> Vec<Effect> {
        self.note(Level::Error, format!("editor: {why}"));
        self.editor.error = Some(why);
        Vec::new()
    }
}

/// The only writer of the model. Every intent is handled here, and every effect it needs comes
/// back to the caller; nothing else changes the model.
pub fn reduce(m: &mut Model, i: Intent) -> Vec<Effect> {
    match i {
        Intent::PickTape => vec![Effect::PickTape],
        Intent::Open(path) => vec![Effect::ReadTape(path)],
        Intent::TapeRead {
            path,
            text,
            lineage,
        } => open(m, path, text, lineage),
        Intent::SessionRead(Ok(text)) => match Session::from_ron(&text) {
            Ok(mut s) => {
                // A new launch: the next serial, so this session's keys are new.
                s.serial = s.serial.saturating_add(1);
                m.session = s;
                vec![Effect::SaveSession]
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
        Intent::Today(d) => {
            m.today = Some(d);
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
            snapshot_wanted(m)
        }
        Intent::Select(e) => {
            m.selection = e;
            snapshot_wanted(m)
        }
        Intent::Cursor(c) => {
            m.cursor = c;
            snapshot_wanted(m)
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
        Intent::Stage(form) => stage(m, &form),
        Intent::Unstage(i) => {
            if i < m.editor.staged.len() {
                m.editor.staged.remove(i);
                m.editor.offer.clear();
                m.editor.error = None;
            }
            Vec::new()
        }
        Intent::Apply => apply(m),
        Intent::RemoveOrphans { note } => {
            if m.editor.offer.is_empty() {
                return Vec::new();
            }
            let note = note.trim();
            if note.is_empty() {
                return m.refuse(FormError::Note.to_string());
            }
            let offer = std::mem::take(&mut m.editor.offer);
            for k in offer {
                m.editor.staged.push(TapeEdit {
                    op: EditOp::RemoveParam(k),
                    note: note.to_string(),
                });
            }
            apply(m)
        }
        Intent::PickSaveTape => match m.focus {
            Some(_) => vec![Effect::PickSaveTape],
            None => Vec::new(),
        },
        Intent::SaveTape(path) => save_tape(m, path),
        Intent::PickExportDir => match m.focus {
            Some(_) => vec![Effect::PickExportDir],
            None => Vec::new(),
        },
        Intent::Export(dir) => export_run(m, dir),
        Intent::Written { job, result } => written(m, job, result),
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

/// The actor inspector reads the lots and the actor's own state from a snapshot of the state at
/// the cursor (docs/GUI.md §4). Ask the focused run for one when an actor is selected, the run
/// is not running and its record has none of that state; each state tick is asked once per
/// load. A running run is not asked: its holdings come from the record every tick.
fn snapshot_wanted(m: &mut Model) -> Vec<Effect> {
    if !matches!(m.selection, Some(Entity::Actor(_))) {
        return Vec::new();
    }
    let cursor = match m.cursor {
        Cursor::Live => None,
        Cursor::At(t) => Some(t),
    };
    let Some(r) = m.focus.and_then(|id| m.runs.get_mut(&id)) else {
        return Vec::new();
    };
    let s = &r.store;
    let idle = !matches!(s.status(), RunStatus::Running { .. } | RunStatus::Empty);
    if s.run().is_none() || s.stopped().is_some() || !idle {
        return Vec::new();
    }
    let tick = s.state_at(cursor);
    if s.snapshot(tick).is_some() || !r.asked.insert(tick) {
        return Vec::new();
    }
    vec![Effect::Send {
        run: r.id,
        cmd: Cmd::Snapshot(tick),
    }]
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

/// Start a run's driver with the session's breakpoints and load its tape, from genesis or from
/// a ring checkpoint.
fn start(m: &Model, id: RunId, tape: Tape, from: Option<ResumeFrom>) -> [Effect; 3] {
    [
        Effect::Spawn(id),
        Effect::Send {
            run: id,
            cmd: Cmd::Breakpoints(m.session.breakpoints.clone()),
        },
        Effect::Send {
            run: id,
            cmd: Cmd::Load {
                tape: Box::new(tape),
                from,
            },
        },
    ]
}

/// A tape file's text: parse it, record it as a base, and start its run paused at genesis. A
/// lineage beside it makes the run an experiment (U3).
fn open(
    m: &mut Model,
    path: String,
    text: Result<String, String>,
    lineage: Option<Result<String, String>>,
) -> Vec<Effect> {
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
    let lineage = match lineage {
        None => None,
        Some(Err(e)) => {
            m.note(
                Level::Error,
                format!("the lineage beside {path} cannot be read: {e}"),
            );
            None
        }
        Some(Ok(t)) => match Lineage::from_ron(&t) {
            Ok(l) => {
                if l.tape_hash.0 != hash {
                    m.note(
                        Level::Error,
                        format!(
                            "the lineage beside {path} describes tape_hash {}, not this tape's \
                             {}: the tape was edited after it was saved",
                            l.tape_hash,
                            Hex(hash)
                        ),
                    );
                }
                Some(l)
            }
            Err(e) => {
                m.note(
                    Level::Error,
                    format!("the lineage beside {path} does not read: {e}"),
                );
                None
            }
        },
    };
    let id = RunId(m.next);
    m.next += 1;
    let origin = if lineage.is_some() {
        Origin::Experiment
    } else {
        Origin::of(&tape)
    };
    let mut run = Run::new(id, Some(path), tape.clone(), origin);
    run.lineage = lineage;
    m.runs.insert(id, run);
    m.focus = Some(id);
    m.cursor = Cursor::Live;
    out.extend(start(m, id, tape, None));
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
        // A run opens with every price plotted (docs/GUI.md §4): in a new session, and in a
        // session whose plots name nothing of this run's world, as another tape's do. Plots
        // are keys, kept across tapes, so a tape that shares keys keeps the user's choice.
        if !m.session.plots.iter().any(|k| k.in_world(world)) {
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
    // A pause or a load may leave the actor inspector without its snapshot.
    let settles = matches!(obs, Obs::Paused { .. } | Obs::Loaded { .. });
    if matches!(obs, Obs::Loaded { .. }) {
        run.asked.clear();
        run.watch = RationWatch::default();
    }
    // Rationing onsets by class, read before the store takes the batch and logged once it has.
    let onsets = match &obs {
        Obs::Batch(b) if !was_stopped => run.watch.onsets(id, b, run.store.catalogue()),
        _ => Vec::new(),
    };
    let ingested = run.store.ingest(obs);
    if ingested.is_ok() {
        m.log.extend(onsets);
    }
    if let Err(e) = ingested {
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
    if settles && m.focus == Some(id) {
        out.extend(snapshot_wanted(m));
    }
    out
}

/// Close a run: drop its driver, and its base from the session unless another run reads it.
/// Its keys are retired, so no later branch mints them (E8).
fn close(m: &mut Model, id: RunId) -> Vec<Effect> {
    let Some(run) = m.runs.remove(&id) else {
        return Vec::new();
    };
    m.retired.extend(keys::tape_keys(&run.tape));
    let mut out = vec![Effect::Close(id)];
    if let Some(path) = &run.path {
        let shared = m.runs.values().any(|r| r.path.as_ref() == Some(path));
        let base = m.session.bases.iter().any(|b| &b.path == path);
        if !shared && base {
            m.session.bases.retain(|b| &b.path != path);
            out.push(Effect::SaveSession);
        }
    }
    if m.focus == Some(id) {
        m.focus = m.runs.keys().next_back().copied();
    }
    out
}

/// Read the form into an edit of the focused run and stage it, or say why not.
fn stage(m: &mut Model, form: &Form) -> Vec<Effect> {
    let Some(id) = m.focus else {
        return m.refuse("no run is open to edit".to_string());
    };
    let taken = m.taken_keys(id);
    let tape = &m.runs[&id].tape;
    match edit::form::parse(form, tape, &taken) {
        Ok(e) => {
            m.editor.staged.push(e);
            m.editor.error = None;
            m.editor.raw = None;
            m.editor.offer.clear();
            Vec::new()
        }
        Err(e) => {
            m.editor.raw = match &e {
                FormError::Act { line, col, message } => Some(RawError {
                    text: form.act.clone(),
                    line: *line,
                    col: *col,
                    message: message.clone(),
                }),
                _ => None,
            };
            m.editor.error = Some(e.to_string());
            Vec::new()
        }
    }
}

/// Materialise the staged edits on the focused run, and start the branch.
fn apply(m: &mut Model) -> Vec<Effect> {
    let Some(on) = m.today else {
        return m.refuse("no date to stamp the experiment with".to_string());
    };
    let Some(pid) = m.focus else {
        return m.refuse("no run is open to edit".to_string());
    };
    if m.runs[&pid].store.world().is_none() {
        return m.refuse("the run has not loaded yet".to_string());
    }
    let edits = m.editor.staged.clone();
    // Keys new to the run tree, checked again: the tree may have grown since the edit was
    // staged.
    let mut tree = m.retired.clone();
    for r in m.tree(pid) {
        tree.extend(keys::tape_keys(&m.runs[&r].tape));
    }
    for (i, e) in edits.iter().enumerate() {
        if let Some(k) = e.op.added() {
            if tree.contains(k.as_str()) {
                return m.refuse(
                    EditError::Taken {
                        edit: i,
                        key: k.clone(),
                    }
                    .to_string(),
                );
            }
        }
    }
    match edit::materialise(&m.runs[&pid].tape, &edits, on) {
        Err(EditError::Orphans { params }) => {
            let why = EditError::Orphans {
                params: params.clone(),
            }
            .to_string();
            m.editor.offer = params;
            m.editor.error = Some(why);
            Vec::new()
        }
        Err(e) => m.refuse(e.to_string()),
        Ok(b) => branch(m, pid, b),
    }
}

/// Start a branch of `pid`: a new run whose Runner resumes from the parent's ring checkpoint
/// that `plan` picks, or reruns from genesis.
fn branch(m: &mut Model, pid: RunId, b: edit::Branch) -> Vec<Effect> {
    let parent = &m.runs[&pid];
    let plan = edit::plan(&parent.store, &b.world);
    let hash = tape_hash(&b.tape);
    let on_disk = parent.path.as_ref().map(|p| Ancestor {
        tape_hash: Hex(parent.tape_hash),
        path: p.clone(),
    });
    let ancestor = match on_disk {
        Some(_) => Some(Box::new(parent.tape.clone())),
        None => parent.ancestor.clone(),
    };
    let lineage = Lineage::of(hash, on_disk, parent.lineage.as_ref(), &b.applied);
    let id = RunId(m.next);
    m.next += 1;
    let mut run = Run::new(id, None, b.tape.clone(), Origin::Experiment);
    run.parent = Some(pid);
    run.applied = b.applied;
    run.lineage = lineage;
    run.ancestor = ancestor;
    let edits = run.applied.len();
    let (from, how) = match plan {
        Plan::Resume(cp) => {
            let date = b
                .world
                .clock
                .date_of(cp.tick())
                .map_or_else(String::new, |d| format!(" ({d})"));
            let how = format!(
                "resumes from the parent's ring checkpoint at state tick {}{date}",
                cp.tick()
            );
            run.resume = Some(cp.clone());
            (Some(ResumeFrom::Ring(cp)), how)
        }
        Plan::Rerun { reason } => {
            let how = format!("reruns from genesis: {reason}");
            run.rerun = Some(reason);
            (None, how)
        }
    };
    m.note(
        Level::Info,
        format!(
            "{id} branches from {pid} by {edits} edit{}: {} {how}",
            if edits == 1 { "" } else { "s" },
            b.tape.header.name
        ),
    );
    m.runs.insert(id, run);
    m.focus = Some(id);
    m.cursor = Cursor::Live;
    m.editor = Editor::default();
    start(m, id, b.tape, from).into()
}

/// "Save tape as": the focused run's canonical tape, and its lineage beside it. The base file
/// is never overwritten, nor is any other.
fn save_tape(m: &mut Model, path: String) -> Vec<Effect> {
    let path = path.trim().to_string();
    let Some(r) = m.focused() else {
        return Vec::new();
    };
    if path.is_empty() {
        return m.refuse("a path to save the tape to is required".to_string());
    }
    if m.runs
        .values()
        .any(|x| x.path.as_deref() == Some(path.as_str()))
    {
        return m.refuse(format!(
            "{path} is a tape this session opened or saved; a base file is never overwritten"
        ));
    }
    let mut files = vec![(path.clone(), r.tape.to_ron())];
    if let Some(l) = &r.lineage {
        files.push((lineage_path(&path), l.to_ron()));
    }
    vec![Effect::Write {
        job: Job::SaveTape { run: r.id, path },
        files,
    }]
}

/// Export the focused run into `dir`.
fn export_run(m: &mut Model, dir: String) -> Vec<Effect> {
    let dir = dir.trim().to_string();
    let Some(id) = m.focus else {
        return Vec::new();
    };
    if dir.is_empty() {
        return m.refuse("a directory to export into is required".to_string());
    }
    match m.export_files(id) {
        Ok(files) => {
            let files = files
                .into_iter()
                .map(|(name, text)| {
                    let p = std::path::Path::new(&dir).join(name);
                    (p.display().to_string(), text)
                })
                .collect();
            vec![Effect::Write {
                job: Job::Export { run: id, dir },
                files,
            }]
        }
        Err(e) => m.refuse(format!("cannot export {id}: {e}")),
    }
}

/// A file job done, or refused.
fn written(m: &mut Model, job: Job, result: Result<Vec<String>, String>) -> Vec<Effect> {
    match (job, result) {
        (Job::SaveTape { run, path }, Ok(files)) => {
            if let Some(r) = m.runs.get_mut(&run) {
                // Saved, the tape is its branches' nearest ancestor on disk.
                r.path = Some(path.clone());
            }
            m.note(
                Level::Info,
                format!("{run} saved as {path}: {}", files.join(", ")),
            );
        }
        (Job::Export { run, dir }, Ok(files)) => {
            let stamp = m
                .runs
                .get(&run)
                .map_or_else(String::new, |r| export::stamp(r.origin));
            m.note(
                Level::Info,
                format!("{run} exported to {dir} ({stamp}): {}", files.join(", ")),
            );
        }
        (Job::SaveTape { run, path }, Err(why)) => {
            m.refuse(format!("{run} not saved as {path}: {why}"));
        }
        (Job::Export { run, dir }, Err(why)) => {
            m.refuse(format!("{run} not exported to {dir}: {why}"));
        }
    }
    Vec::new()
}
