//! Files (docs/GUI.md §3.2): tapes, their lineages, exports, `session.ron` and `layout.ron`
//! natively, through std::fs, and native file dialogs through rfd. The Parquet spill joins at
//! G3. The date an experiment is stamped with is read here too, from the system clock.
//!
//! The session's directory is `$RUSTYECON_GUI_DIR` when set, else the platform's configuration
//! directory: `%APPDATA%\rustyecon\gui` on Windows, `$XDG_CONFIG_HOME/rustyecon/gui` or
//! `~/.config/rustyecon/gui` elsewhere. Both files are small, and each is written through a
//! `.tmp` file and a rename, so a reader never sees half of one. A file that does not read is
//! set aside as `<name>.unreadable`, never overwritten. A saved tape, its lineage and an
//! export's files are written only where no file is: nothing the user has is written over.

use rustyecon_engine::prelude::Date;
use std::path::{Path, PathBuf};

/// The session file's name (U8).
pub const SESSION: &str = "session.ron";
/// The tile layout's file name, beside it (U8).
pub const LAYOUT: &str = "layout.ron";

/// A tape's text, or why it could not be read.
pub fn read_text(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Today's date in UTC, from the system clock: the date a GUI experiment is stamped with.
/// `None` if the clock reads before 1970.
pub fn today() -> Option<Date> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Date::from_days(i64::try_from(secs / 86_400).ok()?)
}

/// Write each `(path, text)`, creating the directories they need, but only where no file is:
/// if any path exists, nothing is written. Returns the paths written.
pub fn write_new(files: &[(String, String)]) -> Result<Vec<String>, String> {
    if let Some((p, _)) = files.iter().find(|(p, _)| Path::new(p).exists()) {
        return Err(format!("{p} exists, and no file is written over"));
    }
    let mut done = Vec::new();
    for (p, text) in files {
        let path = Path::new(p);
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("{p}: {e}"))?;
        std::io::Write::write_all(&mut f, text.as_bytes()).map_err(|e| format!("{p}: {e}"))?;
        done.push(p.clone());
    }
    Ok(done)
}

/// Ask the user where to save a tape: the native dialog, filtered to `.ron`.
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_save_tape() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Save tape as")
        .add_filter("tape", &["ron"])
        .save_file()
        .map(|p| p.display().to_string())
}

/// No dialog on the web until W1.
#[cfg(target_arch = "wasm32")]
pub fn pick_save_tape() -> Option<String> {
    None
}

/// Ask the user for a directory to export into: the native dialog.
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_dir() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Export into")
        .pick_folder()
        .map(|p| p.display().to_string())
}

/// No dialog on the web until W1.
#[cfg(target_arch = "wasm32")]
pub fn pick_dir() -> Option<String> {
    None
}

/// Ask the user for a tape file: the native dialog, filtered to `.ron`. `None` if the user
/// chose none. It blocks the frame while it is open, which a native G0 accepts; the web build's
/// `AsyncFileDialog` waits for W1.
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_tape() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Open a tape")
        .add_filter("tape", &["ron"])
        .pick_file()
        .map(|p| p.display().to_string())
}

/// No dialog on the web until W1.
#[cfg(target_arch = "wasm32")]
pub fn pick_tape() -> Option<String> {
    None
}

/// Where the session lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Files {
    dir: PathBuf,
}

impl Files {
    /// The session in `dir`.
    pub fn at(dir: impl Into<PathBuf>) -> Files {
        Files { dir: dir.into() }
    }

    /// The session's default directory, if the environment names one.
    pub fn default_dir() -> Option<PathBuf> {
        let var = |k: &str| {
            std::env::var_os(k)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        if let Some(d) = var("RUSTYECON_GUI_DIR") {
            return Some(d);
        }
        let base = if cfg!(windows) {
            var("APPDATA")?
        } else {
            var("XDG_CONFIG_HOME").or_else(|| var("HOME").map(|h| h.join(".config")))?
        };
        Some(base.join("rustyecon").join("gui"))
    }

    /// The directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// `session.ron`'s path.
    pub fn session_path(&self) -> PathBuf {
        self.dir.join(SESSION)
    }

    /// `layout.ron`'s path.
    pub fn layout_path(&self) -> PathBuf {
        self.dir.join(LAYOUT)
    }

    /// `session.ron`'s text: `None` when there is none yet.
    pub fn read_session(&self) -> Option<Result<String, String>> {
        read_if_there(self.session_path())
    }

    /// Write `session.ron`.
    pub fn write_session(&self, text: &str) -> Result<(), String> {
        self.write(SESSION, text)
    }

    /// `layout.ron`'s text: `None` when there is none yet.
    pub fn read_layout(&self) -> Option<Result<String, String>> {
        read_if_there(self.layout_path())
    }

    /// Write `layout.ron`.
    pub fn write_layout(&self, text: &str) -> Result<(), String> {
        self.write(LAYOUT, text)
    }

    /// Move a file that does not read out of the way, to `<name>.unreadable`, so a fresh one is
    /// written beside it and the old one is kept.
    pub fn set_aside(&self, name: &str) -> Result<PathBuf, String> {
        let from = self.dir.join(name);
        let to = self.dir.join(format!("{name}.unreadable"));
        std::fs::rename(&from, &to).map_err(|e| format!("{}: {e}", from.display()))?;
        Ok(to)
    }

    fn write(&self, name: &str, text: &str) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let path = self.dir.join(name);
        let tmp = self.dir.join(format!("{name}.tmp"));
        std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// A file's text: `None` when there is no file there.
pub fn read_if_there(path: impl AsRef<Path>) -> Option<Result<String, String>> {
    let path = path.as_ref();
    match std::fs::read_to_string(path) {
        Ok(t) => Some(Ok(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => Some(Err(format!("{}: {e}", path.display()))),
    }
}
