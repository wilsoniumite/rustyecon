//! `session.ron` (U8): what a session keeps between launches, outside every tape. The tapes it
//! opened, by path with their `tape_hash`; the plotted series; the pinned entities; the
//! breakpoints; the speed cap; and the session's serial, which minted keys carry
//! (`gui.<serial>.<n>`, docs/GUI.md §5.1 item 2). A branch lives in memory until "Save tape
//! as" writes it: the session does not re-make branches at launch (G0.2). Text in, text out:
//! `platform` reads and writes the file.

use crate::run::{Breakpoint, Entity, SeriesKey};
use certify::Hex;
use serde::{Deserialize, Deserializer, Serialize};

/// The format this build reads and writes. Format 2 (G0.2) adds the serial; a format-1 file
/// does not read, and is set aside.
pub const SESSION_FORMAT: u32 = 2;

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
    /// The session's serial: 1 for a new session, one more at each launch that reads it, and
    /// one more than a set-aside session's when its serial still reads. A minted key is
    /// `gui.<serial>.<n>`, so launches that read one `session.ron` in turn mint different
    /// keys. Two windows launched from one file, or a session whose file is lost, can share a
    /// serial; a key is then new only to the tapes each session opened.
    pub serial: u32,
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
    /// Serial 1, no tape, nothing plotted or pinned, a breakpoint on error, no speed cap.
    fn default() -> Session {
        Session {
            format: SESSION_FORMAT,
            serial: 1,
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
        #[derive(Deserialize)]
        #[serde(rename = "Session")]
        struct Probe {
            format: u32,
        }
        let format = |f: u32| format!("session format {f}; this build reads {SESSION_FORMAT}");
        if let Ok(p) = ron::from_str::<Probe>(text) {
            if p.format != SESSION_FORMAT {
                return Err(format(p.format));
            }
        }
        let s: Session = ron::from_str(text).map_err(|e| e.to_string())?;
        if s.format != SESSION_FORMAT {
            return Err(format(s.format));
        }
        Ok(s)
    }
}

/// The serial of a session file that does not read as a whole, if that field still reads: a
/// session of another format, or with a field this build does not know. A format-1 file has
/// none; no key was minted under one.
pub fn serial_of(text: &str) -> Option<u32> {
    #[derive(Deserialize)]
    #[serde(rename = "Session")]
    struct Probe {
        serial: u32,
    }
    ron::from_str::<Probe>(text).ok().map(|p| p.serial)
}
