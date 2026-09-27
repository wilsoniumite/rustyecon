//! `session.ron` (U8): what a session keeps between launches, outside every tape. The tapes it
//! opened, by path with their `tape_hash`; the plotted series; the pinned entities; the
//! breakpoints; the speed cap. G0.2 adds each branch's edits. Text in, text out: `platform`
//! reads and writes the file.

use crate::run::{Breakpoint, Entity, SeriesKey};
use certify::Hex;
use serde::{Deserialize, Deserializer, Serialize};

/// The format this build reads and writes.
pub const SESSION_FORMAT: u32 = 1;

/// A tape opened from a file: its path and its `tape_hash` when last read. A base whose file
/// changed on disk opens as a new root run (docs/GUI.md §5.1 item 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Base {
    /// The path it was opened from.
    pub path: String,
    /// `certify::tape_hash` of the tape read there.
    pub tape_hash: Hex,
}

/// Everything a session keeps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// [`SESSION_FORMAT`].
    pub format: u32,
    /// The tapes opened, in the order they were.
    pub bases: Vec<Base>,
    /// The plotted series, by key.
    pub plots: Vec<SeriesKey>,
    /// The pinned entities, by key.
    pub pins: Vec<Entity>,
    /// The breakpoints every run gets.
    pub breakpoints: Vec<Breakpoint>,
    /// The speed cap, in ticks per second; `None` runs as fast as the machine does. Written
    /// always, as `None` or `Some(..)`.
    #[serde(deserialize_with = "required")]
    pub speed: Option<u32>,
}

/// Read an `Option` field that must be written, as core reads a tape's: serde would otherwise
/// read a missing `Option` as `None`.
fn required<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

impl Default for Session {
    /// No tape, nothing plotted or pinned, a breakpoint on error, no speed cap.
    fn default() -> Session {
        Session {
            format: SESSION_FORMAT,
            bases: Vec::new(),
            plots: Vec::new(),
            pins: Vec::new(),
            breakpoints: vec![Breakpoint::OnError],
            speed: None,
        }
    }
}

impl Session {
    /// The session as RON.
    pub fn to_ron(&self) -> String {
        let pretty = ron::ser::PrettyConfig::new().struct_names(true);
        ron::ser::to_string_pretty(self, pretty).expect("a session serialises")
    }

    /// A session from RON, if it is one this build reads: every field present, none unknown,
    /// and the format this build writes.
    pub fn from_ron(text: &str) -> Result<Session, String> {
        let s: Session = ron::from_str(text).map_err(|e| e.to_string())?;
        if s.format != SESSION_FORMAT {
            return Err(format!(
                "session format {}; this build reads {SESSION_FORMAT}",
                s.format
            ));
        }
        Ok(s)
    }
}
