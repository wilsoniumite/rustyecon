//! `session.ron` (U8): what a session keeps between launches, outside every tape. The tapes it
//! opened, by path with their `tape_hash`; the plotted series; the pinned entities; the
//! breakpoints; the speed cap; the session's serial, which minted keys carry
//! (`gui.<serial>.<n>`, docs/GUI.md §5.1 item 2); and, from G1, the watchlist and the plot
//! panels shown on a log scale. A branch lives in memory until "Save tape as" writes it: the
//! session does not re-make branches at launch (G0.2). Text in, text out: `platform` reads and
//! writes the file.

use crate::run::{Breakpoint, Entity, SeriesKey};
use certify::Hex;
use serde::{Deserialize, Deserializer, Serialize};

/// The format this build writes. Format 2 (G0.2) adds the serial; format 3 (G1) the watchlist,
/// the log-scale panels, and event and date breakpoints. This build reads format 2 too, with
/// no series watched and every panel linear, and writes it back as 3; a format-1 file does not
/// read, and is set aside. A G0 build sets a format-3 file aside.
pub const SESSION_FORMAT: u32 = 3;

/// The oldest format this build reads.
pub const SESSION_FORMAT_READ: u32 = 2;

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
    /// The watched series, by key, in the order they were watched (G1). Absent from a
    /// format-2 file, where it reads as none.
    #[serde(default)]
    pub watch: Vec<SeriesKey>,
    /// The units whose plot panel is drawn on a log scale (G1). Absent from a format-2 file,
    /// where it reads as none.
    #[serde(default)]
    pub log_axes: Vec<String>,
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
            watch: Vec::new(),
            log_axes: Vec::new(),
        }
    }
}

impl Session {
    /// The session as RON.
    pub fn to_ron(&self) -> String {
        let pretty = ron::ser::PrettyConfig::new().struct_names(true);
        ron::ser::to_string_pretty(self, pretty).expect("a session serialises")
    }

    /// A session from RON, if it is one this build reads: every field its format has present,
    /// none unknown, none its format lacks, and a format from [`SESSION_FORMAT_READ`] to
    /// [`SESSION_FORMAT`]. A format-3 file has `watch` and `log_axes`; a format-2 file, as G0
    /// wrote it, has neither and no event or date breakpoint, and reads as format 3 with nothing
    /// watched and no log scale.
    pub fn from_ron(text: &str) -> Result<Session, String> {
        #[derive(Deserialize)]
        #[serde(rename = "Session")]
        struct Probe {
            format: u32,
        }
        /// Which of format 3's fields a file has written.
        #[derive(Deserialize)]
        #[serde(rename = "Session")]
        struct Written {
            #[serde(default, deserialize_with = "written")]
            watch: bool,
            #[serde(default, deserialize_with = "written")]
            log_axes: bool,
        }
        let readable = |f: u32| (SESSION_FORMAT_READ..=SESSION_FORMAT).contains(&f);
        let format = |f: u32| {
            format!(
                "session format {f}; this build reads {SESSION_FORMAT_READ} to {SESSION_FORMAT}"
            )
        };
        if let Ok(p) = ron::from_str::<Probe>(text) {
            if !readable(p.format) {
                return Err(format(p.format));
            }
        }
        let mut s: Session = ron::from_str(text).map_err(|e| e.to_string())?;
        if !readable(s.format) {
            return Err(format(s.format));
        }
        let w: Written = ron::from_str(text).map_err(|e| e.to_string())?;
        if s.format == 3 {
            for (field, has) in [("watch", w.watch), ("log_axes", w.log_axes)] {
                if !has {
                    return Err(format!("session format 3 lacks the field {field}"));
                }
            }
        }
        if s.format < 3 {
            if w.watch || w.log_axes {
                return Err(format!(
                    "session format {} does not have the watchlist or log scales",
                    s.format
                ));
            }
            let later = s
                .breakpoints
                .iter()
                .find(|b| matches!(b, Breakpoint::OnEvent(_) | Breakpoint::OnDate(_)));
            if let Some(b) = later {
                return Err(format!(
                    "session format {} does not have event or date breakpoints: {b:?}",
                    s.format
                ));
            }
        }
        s.format = SESSION_FORMAT;
        Ok(s)
    }
}

/// Read a field only to say that it was written: `#[serde(default)]` gives `false` where it
/// was not.
fn written<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    serde::de::IgnoredAny::deserialize(d).map(|_| true)
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
