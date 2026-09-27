//! Files (docs/GUI.md §3.2): tapes, `session.ron` and `layout.ron` natively, through std::fs.
//! File dialogs (rfd) join with the toolbar's Open, and the Parquet spill at G3.
//!
//! The session's directory is `$RUSTYECON_GUI_DIR` when set, else the platform's configuration
//! directory: `%APPDATA%\rustyecon\gui` on Windows, `$XDG_CONFIG_HOME/rustyecon/gui` or
//! `~/.config/rustyecon/gui` elsewhere. Both files are small, and each is written through a
//! `.tmp` file and a rename, so a reader never sees half of one. A file that does not read is
//! set aside as `<name>.unreadable`, never overwritten.

use std::path::{Path, PathBuf};

/// The session file's name (U8).
pub const SESSION: &str = "session.ron";
/// The tile layout's file name, beside it (U8).
pub const LAYOUT: &str = "layout.ron";

/// A tape's text, or why it could not be read.
pub fn read_text(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
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
        read_if_there(&self.session_path())
    }

    /// Write `session.ron`.
    pub fn write_session(&self, text: &str) -> Result<(), String> {
        self.write(SESSION, text)
    }

    /// `layout.ron`'s text: `None` when there is none yet.
    pub fn read_layout(&self) -> Option<Result<String, String>> {
        read_if_there(&self.layout_path())
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

fn read_if_there(path: &Path) -> Option<Result<String, String>> {
    match std::fs::read_to_string(path) {
        Ok(t) => Some(Ok(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => Some(Err(format!("{}: {e}", path.display()))),
    }
}
