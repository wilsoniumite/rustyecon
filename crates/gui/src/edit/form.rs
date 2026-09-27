//! The editor's form (docs/GUI.md §5.1 item 2): its text fields, read into a [`TapeEdit`]
//! before anything is applied. The form checks first: the note is not empty, a key matches
//! `[a-z0-9_.-]+` and a new one is new to every tape of the session, a date parses, an act
//! reads as the tape writes one, and no edit touches a ledger tolerance. An act that does not
//! read is refused with the parser's line and column, which the editor's raw pane shows beside
//! the text it read.

use super::keys;
use super::{touches_ledger, Act, EditOp, TapeEdit, Unit};
use rustyecon_engine::prelude::*;
use std::collections::BTreeSet;
use std::fmt;

/// What kind of edit the form makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpKind {
    /// A new dated event.
    #[default]
    AddEvent,
    /// Remove a dated event.
    RemoveEvent,
    /// A new recurring entry.
    AddRecurring,
    /// Remove a recurring entry.
    RemoveRecurring,
    /// A new param.
    AddParam,
    /// Remove a param.
    RemoveParam,
    /// A param's new genesis value.
    SetGenesisParam,
}

/// Which of the form's fields a kind reads, besides the key and the note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Fields {
    /// The date: `at`, or a recurring entry's `first`.
    pub date: bool,
    /// The act.
    pub act: bool,
    /// A recurring entry's period and last date.
    pub recurring: bool,
    /// A value.
    pub value: bool,
    /// A unit.
    pub unit: bool,
}

impl OpKind {
    /// Every kind, in the order the editor offers them.
    pub const ALL: [OpKind; 7] = [
        OpKind::AddEvent,
        OpKind::RemoveEvent,
        OpKind::AddRecurring,
        OpKind::RemoveRecurring,
        OpKind::AddParam,
        OpKind::RemoveParam,
        OpKind::SetGenesisParam,
    ];

    /// Its name, as `EditOp` names it.
    pub fn name(self) -> &'static str {
        match self {
            OpKind::AddEvent => "AddEvent",
            OpKind::RemoveEvent => "RemoveEvent",
            OpKind::AddRecurring => "AddRecurring",
            OpKind::RemoveRecurring => "RemoveRecurring",
            OpKind::AddParam => "AddParam",
            OpKind::RemoveParam => "RemoveParam",
            OpKind::SetGenesisParam => "SetGenesisParam",
        }
    }

    /// The fields it reads.
    pub fn fields(self) -> Fields {
        let mut f = Fields::default();
        match self {
            OpKind::AddEvent => {
                f.date = true;
                f.act = true;
            }
            OpKind::AddRecurring => {
                f.date = true;
                f.act = true;
                f.recurring = true;
            }
            OpKind::AddParam => {
                f.value = true;
                f.unit = true;
            }
            OpKind::SetGenesisParam => f.value = true,
            OpKind::RemoveEvent | OpKind::RemoveRecurring | OpKind::RemoveParam => {}
        }
        f
    }

    /// Whether it adds an entry, whose key must be new.
    pub fn adds(self) -> bool {
        matches!(
            self,
            OpKind::AddEvent | OpKind::AddRecurring | OpKind::AddParam
        )
    }
}

/// The form's text, as typed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Form {
    /// The kind of edit.
    pub kind: OpKind,
    /// The entry's key.
    pub key: String,
    /// The date, `YYYY-MM-DD`: an event's `at`, a recurring entry's `first`.
    pub date: String,
    /// The act, as the tape writes it: `SetParam(param: "mine.capacity", to: "mine.capacity.base")`.
    pub act: String,
    /// A recurring entry's period: a param key of unit `Years`.
    pub every: String,
    /// A recurring entry's last date, or empty for none.
    pub last: String,
    /// A value.
    pub value: String,
    /// A unit: `FlowPerYear`, `Years`, …
    pub unit: String,
    /// Why: required.
    pub note: String,
}

/// Why the form was refused.
#[derive(Debug, Clone, PartialEq)]
pub enum FormError {
    /// The note is empty (D3).
    Note,
    /// The key is malformed.
    Key(String),
    /// The key is not new to the tapes of this session (E8).
    Taken(String),
    /// A date does not parse.
    Date {
        /// Which field.
        field: &'static str,
        /// Why.
        why: String,
    },
    /// The act does not read: the parser's line and column in the act's text.
    Act {
        /// The line, from 1.
        line: usize,
        /// The column, from 1.
        col: usize,
        /// The parser's message.
        message: String,
    },
    /// The period's key is malformed.
    Every(String),
    /// The value does not read as a finite number.
    Value(String),
    /// The unit is not one the tape knows.
    Unit(String),
    /// The edit touches a ledger tolerance, which is not editable.
    Ledger(String),
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "refused: ")?;
        match self {
            FormError::Note => write!(f, "a note is required"),
            FormError::Key(why) => write!(f, "{why}"),
            FormError::Taken(k) => write!(
                f,
                "the key {k} is taken by a tape of this session; a new entry needs a new key \
                 (mint one)"
            ),
            FormError::Date { field, why } => write!(f, "the {field} does not parse: {why}"),
            FormError::Act { line, col, message } => write!(
                f,
                "the act does not read at line {line}, column {col}: {message}"
            ),
            FormError::Every(why) => write!(f, "the period: {why}"),
            FormError::Value(why) => write!(f, "the value: {why}"),
            FormError::Unit(u) => write!(
                f,
                "the unit {u:?} is not one of Dimensionless, Years, FlowPerYear, RatePerYear, \
                 CompoundPerYear, FractionPerYear"
            ),
            FormError::Ledger(k) => write!(
                f,
                "{k} is a ledger tolerance (header.ledger), which is not editable"
            ),
        }
    }
}

fn date(text: &str, field: &'static str) -> Result<Date, FormError> {
    Date::parse(text.trim()).map_err(|e| FormError::Date {
        field,
        why: e.to_string(),
    })
}

/// The act's text, read as the tape reads one.
pub fn act(text: &str) -> Result<Act, FormError> {
    ron::from_str::<Act>(text).map_err(|e| FormError::Act {
        line: e.position.line,
        col: e.position.col,
        message: e.code.to_string(),
    })
}

fn value(text: &str) -> Result<f64, FormError> {
    let text = text.trim();
    match text.parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(v),
        Ok(v) => Err(FormError::Value(format!("{v} is not finite"))),
        Err(e) => Err(FormError::Value(format!("{text:?}: {e}"))),
    }
}

/// Read the form into an edit of `tape`. A key it adds must not be in `taken`: every key of
/// every tape open in this session, of every run closed in it, and of the edits waiting to be
/// applied.
pub fn parse(form: &Form, tape: &Tape, taken: &BTreeSet<String>) -> Result<TapeEdit, FormError> {
    let note = form.note.trim();
    if note.is_empty() {
        return Err(FormError::Note);
    }
    let key = keys::valid(&form.key).map_err(FormError::Key)?;
    let op = match form.kind {
        OpKind::AddEvent => EditOp::AddEvent {
            key,
            at: date(&form.date, "date")?,
            act: act(&form.act)?,
        },
        OpKind::AddRecurring => {
            let first = date(&form.date, "date")?;
            let every = keys::valid(&form.every).map_err(FormError::Every)?;
            let last = match form.last.trim() {
                "" => None,
                t => Some(date(t, "last date")?),
            };
            EditOp::AddRecurring {
                key,
                first,
                every,
                last,
                act: act(&form.act)?,
            }
        }
        OpKind::AddParam => {
            let v = value(&form.value)?;
            let u = form.unit.trim();
            let unit = ron::from_str::<Unit>(u).map_err(|_| FormError::Unit(u.to_string()))?;
            EditOp::AddParam {
                key,
                value: v,
                unit,
            }
        }
        OpKind::SetGenesisParam => EditOp::SetGenesisParam {
            key,
            value: value(&form.value)?,
        },
        OpKind::RemoveEvent => EditOp::RemoveEvent(key),
        OpKind::RemoveRecurring => EditOp::RemoveRecurring(key),
        OpKind::RemoveParam => EditOp::RemoveParam(key),
    };
    if let Some(k) = op.added() {
        if taken.contains(k.as_str()) {
            return Err(FormError::Taken(k.to_string()));
        }
    }
    if let Some(k) = touches_ledger(tape, &op) {
        return Err(FormError::Ledger(k.to_string()));
    }
    Ok(TapeEdit {
        op,
        note: note.to_string(),
    })
}
