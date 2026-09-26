//! The build stamp (docs/CERTIFY.md §3): the commit the binary was built from, whether its
//! sources differed from it, the target and the compiler, so that every hash, certificate and
//! manifest the binary writes names its build (R16).
//!
//! - `RUSTYECON_COMMIT`: `git rev-parse HEAD`, 40 hex digits, or `unknown`.
//! - `RUSTYECON_DIRTY`: `false` only when `git status --porcelain` over the build's sources
//!   (`crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `clippy.toml`, untracked
//!   files included) is empty; `true` otherwise, and whenever git cannot be read (fail closed).
//! - `RUSTYECON_TARGET`: cargo's `TARGET`.
//! - `RUSTYECON_RUSTC`: `$RUSTC -V`.
//!
//! The script reruns when any of those sources changes, and when the worktree's `HEAD`, index or
//! `logs/HEAD`, the branch `HEAD` names, `packed-refs`, or anything under the common directory's
//! `refs/heads` does, so a commit, a checkout, a staged change and an edit all restamp. The
//! reflog and the `refs/heads` tree are watched because a branch whose ref is packed has no loose
//! file to watch: its first commit writes a new loose ref, which only a watched directory sees,
//! and git appends to the worktree's `logs/HEAD` on every commit, amend, reset and checkout
//! (amended at S2.5). In WSL a worktree made from Windows names its git directory by a Windows path
//! (`gitdir: C:/…`), which Linux git cannot read; the script maps `X:/` to `/mnt/x/` and retries.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The build's sources, relative to the workspace root.
const SOURCES: [&str; 5] = [
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "clippy.toml",
];

/// How to run git on the workspace: as is, or with the git directory mapped for WSL.
struct Git {
    root: PathBuf,
    mapped: Option<PathBuf>,
}

impl Git {
    fn command(&self) -> Command {
        let mut c = Command::new("git");
        c.arg("-C").arg(&self.root).arg("--no-optional-locks");
        if let Some(dir) = &self.mapped {
            c.env("GIT_DIR", dir)
                .env("GIT_WORK_TREE", &self.root)
                .args(["-c", "safe.directory=*"]);
        }
        c
    }

    /// git's stdout, trimmed, when it exits 0.
    fn run(&self, args: &[&str]) -> Option<String> {
        let out = self.command().args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
    }

    /// A path git prints, made absolute against the root.
    fn path(&self, args: &[&str]) -> Option<PathBuf> {
        let p = PathBuf::from(self.run(args)?);
        Some(if p.is_absolute() {
            p
        } else {
            self.root.join(p)
        })
    }
}

/// `C:/x` or `C:\x` as `/mnt/c/x`.
fn wsl_path(p: &str) -> Option<PathBuf> {
    let b = p.as_bytes();
    if b.len() < 3
        || !b[0].is_ascii_alphabetic()
        || b[1] != b':'
        || !(b[2] == b'/' || b[2] == b'\\')
    {
        return None;
    }
    let drive = char::from(b[0]).to_ascii_lowercase();
    Some(PathBuf::from(format!(
        "/mnt/{drive}/{}",
        p[3..].replace('\\', "/")
    )))
}

/// The git to use: the root as is when git reads it, else a `.git` file's Windows path mapped.
fn git(root: &Path) -> Option<Git> {
    let plain = Git {
        root: root.to_path_buf(),
        mapped: None,
    };
    if plain.run(&["rev-parse", "--git-dir"]).is_some() {
        return Some(plain);
    }
    let text = std::fs::read_to_string(root.join(".git")).ok()?;
    let dir = text.trim().strip_prefix("gitdir:")?.trim();
    let mapped = Git {
        root: root.to_path_buf(),
        mapped: Some(wsl_path(dir)?),
    };
    mapped.run(&["rev-parse", "--git-dir"]).map(|_| mapped)
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| manifest_dir.clone(), Path::to_path_buf);
    for s in SOURCES {
        println!("cargo:rerun-if-changed={}", root.join(s).display());
    }
    println!("cargo:rerun-if-changed={}", root.join(".git").display());

    let mut commit = "unknown".to_string();
    let mut dirty = true;
    if let Some(g) = git(&root) {
        if let Some(c) = g.run(&["rev-parse", "HEAD"]) {
            if c.len() == 40 && c.bytes().all(|b| b.is_ascii_hexdigit()) {
                commit = c;
            }
        }
        let mut status = vec!["status", "--porcelain", "--untracked-files=all", "--"];
        status.extend(SOURCES);
        if commit != "unknown" {
            if let Some(s) = g.run(&status) {
                dirty = !s.is_empty();
            }
        }
        let mut watched = Vec::new();
        if let Some(dir) = g.path(&["rev-parse", "--git-dir"]) {
            watched.push(dir.join("HEAD"));
            watched.push(dir.join("index"));
            watched.push(dir.join("logs").join("HEAD"));
        }
        if let Some(common) = g.path(&["rev-parse", "--git-common-dir"]) {
            if let Some(r) = g.run(&["symbolic-ref", "-q", "HEAD"]) {
                watched.push(common.join(r));
            }
            watched.push(common.join("packed-refs"));
            // A directory: cargo reruns when anything under it changes, a new loose ref too.
            watched.push(common.join("refs").join("heads"));
        }
        for w in watched.iter().filter(|w| w.exists()) {
            println!("cargo:rerun-if-changed={}", w.display());
        }
    }
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(&rustc)
        .arg("-V")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());
    println!("cargo:rustc-env=RUSTYECON_COMMIT={commit}");
    println!("cargo:rustc-env=RUSTYECON_DIRTY={dirty}");
    println!("cargo:rustc-env=RUSTYECON_TARGET={target}");
    println!("cargo:rustc-env=RUSTYECON_RUSTC={version}");
}
