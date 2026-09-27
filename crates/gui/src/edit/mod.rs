//! The editor's model (docs/GUI.md §5.1; D3, U2, R16): tape edits, `materialise`, the
//! lineage, `plan` and export. No egui, no file, no thread and no clock: text and tapes in,
//! tapes and text out. `model/` calls it; `platform/` writes what it makes.
//!
//! - A [`TapeEdit`] is one [`EditOp`] and a required note. No arm carries a basis: the GUI
//!   chooses none (D3).
//! - [`materialise`] applies edits to a parent tape. It stamps every entry it adds or changes
//!   with `Assumed("GUI experiment <date>: <note>")`, marks the name `[GUI experiment <date>]`,
//!   refuses the ledger's tolerances, round-trips the text, and validates through `Sim::new`,
//!   which it builds, reads the world off and drops: only a Runner keeps a `Sim` (U1). A
//!   removal that leaves a param unreferenced comes back as [`EditError::Orphans`], so the
//!   editor can offer the `RemoveParam`.
//! - [`plan`] picks where a branch starts: the parent's latest ring checkpoint whose prefix the
//!   branch shares, or genesis, with the reason.
//! - [`lineage`] names the nearest ancestor on disk and every edit since it.
//! - [`export`] makes the CSV, the manifest envelope, the tapes and the lineage as text.
//! - [`form`] reads the editor's text fields into a `TapeEdit`; [`keys`] checks and mints keys.
//!
//! An entry is written in the tape's raw schema, with its `Basis` and `Unit`: plain data, none
//! of which writes a state. The engine re-exports them (`rustyecon_engine::raw`, and `Basis`
//! and `Unit` in its prelude; amended at G1.1), so the GUI has no edge to core. A scan holds
//! the raw schema out of every module but this one, and core out of the crate
//! (`the_gui_reaches_core_through_the_engine_alone`).

pub mod export;
pub mod form;
pub mod keys;
pub mod lineage;
pub mod plan;

pub use form::{Form, FormError, OpKind};
pub use lineage::{lineage_path, Ancestor, Lineage, LINEAGE_FORMAT};
pub use plan::{plan, Plan};
pub use rustyecon_engine::prelude::{Basis, Unit};
pub use rustyecon_engine::raw::{RawAct, RawEvent, RawParam, RawRecurring};

use crate::run::EXPERIMENT_MARKER;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::RawAgentAction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// A tape action of this engine's agents: what an added event or recurring entry does.
pub type Act = RawAct<RawAgentAction>;
/// A dated event of this engine's agents.
pub type Event = RawEvent<RawAgentAction>;
/// A recurring entry of this engine's agents.
pub type Recurrence = RawRecurring<RawAgentAction>;

/// One edit of a tape, and why it was made. The note is required and must not be empty; no
/// arm carries a basis, since `materialise` stamps every entry it adds or changes (D3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TapeEdit {
    /// What changes.
    pub op: EditOp,
    /// Why: required, not empty. It goes into the stamped basis.
    pub note: String,
}

/// What an edit changes. Each arm touches an entry that carries its own basis (R16): a param,
/// a dated event or a recurring entry. A good, node, channel or class, a genesis line and an
/// actor's inline number have none, and no arm edits them (§5.1 item 2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditOp {
    /// A new dated event: it fires in the tick `at` falls in.
    AddEvent {
        /// Its key, new to every tape of the session (E8).
        key: Key,
        /// Its date.
        at: Date,
        /// What it does, `SetParam(param: .., to: ..)` for a dial.
        act: Act,
    },
    /// Remove a dated event.
    RemoveEvent(Key),
    /// A new recurring entry.
    AddRecurring {
        /// Its key, new to every tape of the session.
        key: Key,
        /// Its first firing.
        first: Date,
        /// Its period: a param of unit `Years`.
        every: Key,
        /// No firing after this date.
        last: Option<Date>,
        /// What it does.
        act: Act,
    },
    /// Remove a recurring entry.
    RemoveRecurring(Key),
    /// A new param: a new value that a `SetParam` can copy, so it keeps a basis (§5.2).
    AddParam {
        /// Its key, new to every tape of the session.
        key: Key,
        /// Its value, in `unit`.
        value: f64,
        /// Its unit.
        unit: Unit,
    },
    /// Remove a param nothing references any more.
    RemoveParam(Key),
    /// Change a param's genesis value. A world edit when the world reads the param: the branch
    /// reruns from genesis.
    SetGenesisParam {
        /// The param.
        key: Key,
        /// Its new genesis value, in its unit.
        value: f64,
    },
}

impl EditOp {
    /// The key of the entry the edit adds, removes or changes.
    pub fn key(&self) -> &Key {
        match self {
            EditOp::AddEvent { key, .. }
            | EditOp::AddRecurring { key, .. }
            | EditOp::AddParam { key, .. }
            | EditOp::SetGenesisParam { key, .. }
            | EditOp::RemoveEvent(key)
            | EditOp::RemoveRecurring(key)
            | EditOp::RemoveParam(key) => key,
        }
    }

    /// The key the edit adds to the tape, if it adds an entry.
    pub fn added(&self) -> Option<&Key> {
        match self {
            EditOp::AddEvent { key, .. }
            | EditOp::AddRecurring { key, .. }
            | EditOp::AddParam { key, .. } => Some(key),
            _ => None,
        }
    }

    /// The edit's name, as the editor lists it.
    pub fn name(&self) -> &'static str {
        match self {
            EditOp::AddEvent { .. } => "AddEvent",
            EditOp::RemoveEvent(_) => "RemoveEvent",
            EditOp::AddRecurring { .. } => "AddRecurring",
            EditOp::RemoveRecurring(_) => "RemoveRecurring",
            EditOp::AddParam { .. } => "AddParam",
            EditOp::RemoveParam(_) => "RemoveParam",
            EditOp::SetGenesisParam { .. } => "SetGenesisParam",
        }
    }
}

impl fmt::Display for EditOp {
    /// The edit in one line, its act as the tape writes it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let act = |a: &Act| one_line(a);
        match self {
            EditOp::AddEvent { key, at, act: a } => {
                write!(f, "AddEvent {key} on {at}: {}", act(a))
            }
            EditOp::AddRecurring {
                key,
                first,
                every,
                last,
                act: a,
            } => {
                let last = last.map_or_else(|| "no end".to_string(), |d| format!("until {d}"));
                write!(
                    f,
                    "AddRecurring {key} from {first} every {every}, {last}: {}",
                    act(a)
                )
            }
            EditOp::AddParam { key, value, unit } => write!(f, "AddParam {key} = {value} {unit}"),
            EditOp::SetGenesisParam { key, value } => {
                write!(f, "SetGenesisParam {key} = {value}")
            }
            EditOp::RemoveEvent(k) | EditOp::RemoveRecurring(k) | EditOp::RemoveParam(k) => {
                write!(f, "{} {k}", self.name())
            }
        }
    }
}

/// What an edit replaced: the value and basis a changed param had, or the entry removed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Replaced {
    /// A param's genesis value and its basis, before `SetGenesisParam`.
    Value {
        /// The value.
        value: f64,
        /// Its basis.
        basis: Basis,
    },
    /// The event removed.
    Event(Event),
    /// The recurring entry removed.
    Recurring(Recurrence),
    /// The param removed.
    Param(RawParam),
}

impl fmt::Display for Replaced {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Replaced::Value { value, basis } => write!(f, "replaced {value}, {basis:?}"),
            Replaced::Event(e) => write!(f, "removed {}", one_line(e)),
            Replaced::Recurring(r) => write!(f, "removed {}", one_line(r)),
            Replaced::Param(p) => write!(f, "removed {}", one_line(p)),
        }
    }
}

/// A tape entry as RON on one line.
pub fn one_line<T: Serialize>(v: &T) -> String {
    ron::to_string(v).unwrap_or_else(|e| format!("({e})"))
}

/// One edit as `materialise` applied it: the edit, the date it was made on, and what it
/// replaced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applied {
    /// The edit, with its note.
    pub edit: TapeEdit,
    /// The date of the experiment: the stamp's date.
    pub on: Date,
    /// The value and basis it replaced, or the entry it removed.
    pub replaced: Option<Replaced>,
}

impl fmt::Display for Applied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} ({:?})", self.on, self.edit.op, self.edit.note)?;
        if let Some(r) = &self.replaced {
            write!(f, "; {r}")?;
        }
        Ok(())
    }
}

/// A materialised branch: its tape, the edits as applied, and the world `Sim::new` resolved.
#[derive(Debug, Clone)]
pub struct Branch {
    /// The branch's tape, as `from_ron` read its canonical text back.
    pub tape: Tape,
    /// The edits, in order, with what each replaced.
    pub applied: Vec<Applied>,
    /// Its world, for [`plan`].
    pub world: World,
}

/// Why edits do not make a branch. Where an error comes from one edit, it names it, by its
/// place in the list.
#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    /// No edit to apply.
    NoEdits,
    /// An edit without a note (D3).
    EmptyNote {
        /// The edit.
        edit: usize,
    },
    /// An edit of a ledger tolerance: the params `header.ledger` names are not editable
    /// (§5.1 item 2).
    Ledger {
        /// The edit.
        edit: usize,
        /// The tolerance.
        key: Key,
    },
    /// A new entry whose key the tape has already, or an earlier edit added (E8).
    Taken {
        /// The edit.
        edit: usize,
        /// The key.
        key: Key,
    },
    /// A removal or change of an entry the tape does not have.
    Missing {
        /// The edit.
        edit: usize,
        /// What kind of entry.
        kind: &'static str,
        /// The key.
        key: Key,
    },
    /// The removals leave these params of the parent unreferenced, which does not load
    /// (ENGINE §2.6): the editor offers a `RemoveParam` for each, with its own note.
    Orphans {
        /// The params, in the order the loader found them.
        params: Vec<Key>,
    },
    /// The tape's canonical text did not read back: the parser's line and column, and the
    /// edit whose entry holds that line, if one does.
    Parse {
        /// The line, from 1.
        line: usize,
        /// The column, from 1.
        col: usize,
        /// The parser's message.
        message: String,
        /// The edit.
        edit: Option<usize>,
    },
    /// The branch does not load: `Sim::new`'s error, with its tape path, and the edit it maps
    /// to, if one.
    Load {
        /// The error.
        error: LoadError,
        /// The edit.
        edit: Option<usize>,
    },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let which =
            |e: &Option<usize>| e.map_or_else(String::new, |i| format!(" (edit {})", i + 1));
        match self {
            EditError::NoEdits => write!(f, "no edit to apply"),
            EditError::EmptyNote { edit } => write!(f, "edit {}: a note is required", edit + 1),
            EditError::Ledger { edit, key } => write!(
                f,
                "edit {}: {key} is a ledger tolerance (header.ledger), which is not editable",
                edit + 1
            ),
            EditError::Taken { edit, key } => write!(
                f,
                "edit {}: the key {key} is taken; a new entry needs a key new to every tape of \
                 this session",
                edit + 1
            ),
            EditError::Missing { edit, kind, key } => {
                write!(f, "edit {}: the tape has no {kind} {key}", edit + 1)
            }
            EditError::Orphans { params } => {
                let keys: Vec<&str> = params.iter().map(Key::as_str).collect();
                write!(
                    f,
                    "the removal leaves {} unreferenced, which does not load: remove {} too, \
                     with a note",
                    keys.join(", "),
                    if keys.len() == 1 { "it" } else { "them" }
                )
            }
            EditError::Parse {
                line,
                col,
                message,
                edit,
            } => write!(
                f,
                "the branch's text does not read back at line {line}, column {col}: \
                 {message}{}",
                which(edit)
            ),
            EditError::Load { error, edit } => {
                write!(f, "the branch does not load: {error}{}", which(edit))
            }
        }
    }
}

impl std::error::Error for EditError {}

/// The marker a GUI edit leaves in a tape's name: `[GUI experiment <date>]` (D3, U2).
pub fn marker(on: Date) -> String {
    format!("[{EXPERIMENT_MARKER} {on}]")
}

/// The basis `materialise` stamps on every entry it adds or changes:
/// `Assumed("GUI experiment <date>: <note>")` (D3, U2).
pub fn stamp(on: Date, note: &str) -> Basis {
    Basis::Assumed(format!("{EXPERIMENT_MARKER} {on}: {note}"))
}

/// A branch's name: the parent's, with any marker it carries taken out, and this one's
/// appended. A branch made only of removals is marked too.
pub fn branch_name(parent: &str, on: Date) -> String {
    let open = format!("[{EXPERIMENT_MARKER} ");
    let mut name = parent.to_string();
    while let Some(i) = name.find(&open) {
        let Some(len) = name[i..].find(']') else {
            break;
        };
        let from = if name[..i].ends_with(' ') { i - 1 } else { i };
        name.replace_range(from..i + len + 1, "");
    }
    let name = name.trim_end();
    if name.is_empty() {
        marker(on)
    } else {
        format!("{name} {}", marker(on))
    }
}

/// The two params `header.ledger` names: the ledger's tolerances.
pub fn ledger_keys(t: &Tape) -> [&Key; 2] {
    [&t.header.ledger.rel_flow, &t.header.ledger.rel_stock]
}

/// Whether `child`'s ledger tolerances differ from `parent`'s: the params each tape's
/// `header.ledger` names, by key, value (bit for bit) and unit. A GUI edit cannot change them;
/// a tape edited by hand can (§5.1 item 2).
pub fn ledger_changed(child: &Tape, parent: &Tape) -> bool {
    let tolerances = |t: &Tape| {
        ledger_keys(t).map(|k| {
            let p = t.params.iter().find(|p| p.key == *k);
            (k.clone(), p.map(|p| (p.value.to_bits(), p.unit)))
        })
    };
    tolerances(child) != tolerances(parent)
}

/// The ledger tolerance an edit would change, if any: a `SetGenesisParam` or `RemoveParam` of
/// one, or an event that sets one. Reading one (a `SetParam`'s `to`) changes nothing.
pub fn touches_ledger(t: &Tape, op: &EditOp) -> Option<Key> {
    let ledger = ledger_keys(t);
    fn set_by(a: &Act) -> Option<&Key> {
        match a {
            RawAct::SetParam { param, .. } => Some(param),
            _ => None,
        }
    }
    let key = match op {
        EditOp::SetGenesisParam { key, .. } | EditOp::RemoveParam(key) => Some(key),
        EditOp::AddEvent { act, .. } | EditOp::AddRecurring { act, .. } => set_by(act),
        _ => None,
    }?;
    ledger.contains(&key).then(|| key.clone())
}

/// Apply one edit to `t`, stamping `basis` on what it adds or changes. `taken` holds every key
/// the parent tape has and every key an earlier edit added.
fn apply_one(
    t: &mut Tape,
    i: usize,
    op: &EditOp,
    basis: &Basis,
    taken: &mut BTreeSet<String>,
) -> Result<Option<Replaced>, EditError> {
    if let Some(key) = op.added() {
        if !taken.insert(key.to_string()) {
            return Err(EditError::Taken {
                edit: i,
                key: key.clone(),
            });
        }
    }
    let missing = |kind: &'static str, key: &Key| EditError::Missing {
        edit: i,
        kind,
        key: key.clone(),
    };
    Ok(match op {
        EditOp::AddEvent { key, at, act } => {
            t.events.push(RawEvent {
                key: key.clone(),
                at: *at,
                basis: basis.clone(),
                act: act.clone(),
            });
            None
        }
        EditOp::AddRecurring {
            key,
            first,
            every,
            last,
            act,
        } => {
            t.recurring.push(RawRecurring {
                key: key.clone(),
                first: *first,
                every: every.clone(),
                last: *last,
                basis: basis.clone(),
                act: act.clone(),
            });
            None
        }
        EditOp::AddParam { key, value, unit } => {
            t.params.push(RawParam {
                key: key.clone(),
                value: *value,
                unit: *unit,
                basis: basis.clone(),
            });
            None
        }
        EditOp::RemoveEvent(key) => {
            let at = t
                .events
                .iter()
                .position(|e| e.key == *key)
                .ok_or_else(|| missing("event", key))?;
            Some(Replaced::Event(t.events.remove(at)))
        }
        EditOp::RemoveRecurring(key) => {
            let at = t
                .recurring
                .iter()
                .position(|e| e.key == *key)
                .ok_or_else(|| missing("recurring entry", key))?;
            Some(Replaced::Recurring(t.recurring.remove(at)))
        }
        EditOp::RemoveParam(key) => {
            let at = t
                .params
                .iter()
                .position(|p| p.key == *key)
                .ok_or_else(|| missing("param", key))?;
            Some(Replaced::Param(t.params.remove(at)))
        }
        EditOp::SetGenesisParam { key, value } => {
            let p = t
                .params
                .iter_mut()
                .find(|p| p.key == *key)
                .ok_or_else(|| missing("param", key))?;
            let old = Replaced::Value {
                value: p.value,
                basis: p.basis.clone(),
            };
            p.value = *value;
            p.basis = basis.clone();
            Some(old)
        }
    })
}

/// Build a branch of `parent` from `edits`, made on `on` (docs/GUI.md §5.1 item 3).
///
/// Every edit needs a note, and none may touch a ledger tolerance. Each entry an edit adds or
/// changes gets the basis [`stamp`]`(on, note)`; the name becomes [`branch_name`]. The result
/// is written with `to_ron` and read back with `from_ron`, and `Sim::new` validates it: that is
/// `resolve` and `Cast::new`, which also makes the whole-world checks. A removal that leaves a
/// param of the parent unreferenced gives [`EditError::Orphans`], every such param listed.
pub fn materialise(parent: &Tape, edits: &[TapeEdit], on: Date) -> Result<Branch, EditError> {
    if edits.is_empty() {
        return Err(EditError::NoEdits);
    }
    let mut t = parent.clone();
    let mut taken = keys::tape_keys(parent);
    let mut applied = Vec::with_capacity(edits.len());
    for (i, e) in edits.iter().enumerate() {
        if e.note.trim().is_empty() {
            return Err(EditError::EmptyNote { edit: i });
        }
        if let Some(key) = touches_ledger(parent, &e.op) {
            return Err(EditError::Ledger { edit: i, key });
        }
        let replaced = apply_one(&mut t, i, &e.op, &stamp(on, &e.note), &mut taken)?;
        applied.push(Applied {
            edit: e.clone(),
            on,
            replaced,
        });
    }
    t.header.name = branch_name(&parent.header.name, on);
    let text = t.to_ron();
    let t = Tape::from_ron(&text).map_err(|e| read_back(&e, &text, edits))?;
    match Sim::new(&t) {
        Ok(sim) => Ok(Branch {
            world: sim.world().clone(),
            tape: t,
            applied,
        }),
        Err(e) => Err(not_loaded(parent, &t, e, edits)),
    }
}

/// A parse error of the round trip, with its line and column, mapped to the edit whose entry
/// holds that line of the canonical text.
fn read_back(e: &LoadError, text: &str, edits: &[TapeEdit]) -> EditError {
    let message = match &e.kind {
        LoadErrorKind::Parse(m) => m.clone(),
        other => other.to_string(),
    };
    let (line, col, message) = split_position(&message);
    // The entry an error line falls in: the last `(key: "…"` at or above it.
    let owner = text
        .lines()
        .take(line)
        .filter_map(|l| {
            let rest = l.trim_start().strip_prefix("(key: \"")?;
            Some(rest.split('"').next()?.to_string())
        })
        .last();
    let edit = owner.and_then(|k| edits.iter().position(|e| e.op.key().as_str() == k));
    EditError::Parse {
        line,
        col,
        message,
        edit,
    }
}

/// `line:col: message`, as ron writes a position, split; `(0, 0, text)` when there is none.
pub fn split_position(text: &str) -> (usize, usize, String) {
    let mut parts = text.splitn(3, ':');
    let (Some(l), Some(c), Some(rest)) = (parts.next(), parts.next(), parts.next()) else {
        return (0, 0, text.to_string());
    };
    match (l.trim().parse(), c.trim().parse()) {
        (Ok(line), Ok(col)) => (line, col, rest.trim().to_string()),
        _ => (0, 0, text.to_string()),
    }
}

/// A load error of the branch: the orphans a removal left, or the error mapped to its edit.
fn not_loaded(parent: &Tape, branch: &Tape, e: LoadError, edits: &[TapeEdit]) -> EditError {
    let added: BTreeSet<&Key> = edits.iter().filter_map(|e| e.op.added()).collect();
    let orphan = |e: &LoadError| -> Option<Key> {
        if e.kind != LoadErrorKind::UnusedParam {
            return None;
        }
        let key = e.path.strip_prefix("params[")?.strip_suffix(']')?;
        let key = Key::new(key).ok()?;
        let of_parent = parent.params.iter().any(|p| p.key == key);
        (of_parent && !added.contains(&key)).then_some(key)
    };
    let Some(first) = orphan(&e) else {
        return EditError::Load {
            edit: edit_of(&e, edits),
            error: e,
        };
    };
    // Every orphan: take each out of a scratch copy and load again, until the loader finds
    // something else, or nothing.
    let mut params = vec![first];
    let mut scratch = branch.clone();
    loop {
        let last = params.last().expect("one orphan at least");
        scratch.params.retain(|p| p.key != *last);
        match Sim::new(&scratch) {
            Err(e) => match orphan(&e) {
                Some(k) if !params.contains(&k) => params.push(k),
                _ => break,
            },
            Ok(_) => break,
        }
    }
    EditError::Orphans { params }
}

/// The edit a load error comes from: the one whose entry its path names
/// (`events[gui.1.1].act`), else the one whose key the error names.
fn edit_of(e: &LoadError, edits: &[TapeEdit]) -> Option<usize> {
    let bracketed = e
        .path
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(k, _)| k.to_string());
    let named = match &e.kind {
        LoadErrorKind::Unknown { key, .. }
        | LoadErrorKind::Duplicate { key, .. }
        | LoadErrorKind::SetParamOnFixed { key }
        | LoadErrorKind::UnitMismatch { key, .. } => Some(key.clone()),
        _ => None,
    };
    let refers = |op: &EditOp, k: &str| {
        if op.key().as_str() == k {
            return true;
        }
        let act = match op {
            EditOp::AddEvent { act, .. } | EditOp::AddRecurring { act, .. } => act,
            _ => return false,
        };
        match act {
            RawAct::SetParam { param, to } => param.as_str() == k || to.as_str() == k,
            RawAct::ScalePrice { by, .. } => by.as_str() == k,
            _ => false,
        }
    };
    for k in [bracketed, named].into_iter().flatten() {
        if let Some(i) = edits.iter().position(|e| e.op.key().as_str() == k) {
            return Some(i);
        }
        if let Some(i) = edits.iter().position(|e| refers(&e.op, &k)) {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    #[test]
    fn a_branch_name_carries_one_marker() {
        let on = d("2026-09-27");
        assert_eq!(branch_name("gate", on), "gate [GUI experiment 2026-09-27]");
        assert_eq!(
            branch_name("gate [GUI experiment 2026-09-20]", on),
            "gate [GUI experiment 2026-09-27]"
        );
        assert_eq!(
            branch_name(
                "gate [GUI experiment 2026-09-20] b [GUI experiment 2026-09-21]",
                on
            ),
            "gate b [GUI experiment 2026-09-27]"
        );
        assert_eq!(branch_name("", on), "[GUI experiment 2026-09-27]");
        assert_eq!(
            stamp(on, "why"),
            Basis::Assumed("GUI experiment 2026-09-27: why".to_string())
        );
    }

    #[test]
    fn positions_split_as_ron_writes_them() {
        assert_eq!(
            split_position("1:29: Expected comma"),
            (1, 29, "Expected comma".to_string())
        );
        assert_eq!(
            split_position("no position"),
            (0, 0, "no position".to_string())
        );
    }
}
