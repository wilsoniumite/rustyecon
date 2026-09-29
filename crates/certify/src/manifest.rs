//! A run's identity and its record (docs/CERTIFY.md §3 and §9; N10, D4, R16): the tape hash, the
//! build, the run key the GUI reuses, and the manifest, which records the run's inputs (the
//! build, the tape by its hash, the world, and any checkpoint it resumed from) and every hash and
//! checkpoint it made, so that a resume can be verified against the run that made its
//! checkpoint. The criteria a certified run was scored against are the certificate's to record,
//! by file, date and hash, not the manifest's.
//!
//! No Parquet and no I/O: paths are strings the caller chose, recorded relative with `/`, and the
//! web build can use every type here (GUI.md §7.2 c).

use rustyecon_core::{fnv1a_64, state_hash, Date, Fnv};
use rustyecon_engine::prelude::{Sim, Tape};
use rustyecon_engine::Checkpoint;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A 64-bit hash or digest, written `"0x%016x"` so that it reads the same in every RON file and
/// every line of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hex(pub u64);

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl Hex {
    /// Parse `0x` and exactly 16 lower-case hex digits.
    pub fn parse(s: &str) -> Option<Hex> {
        let digits = s.strip_prefix("0x")?;
        let ok = digits.len() == 16
            && digits
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !ok {
            return None;
        }
        u64::from_str_radix(digits, 16).ok().map(Hex)
    }
}

impl Serialize for Hex {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Hex, D::Error> {
        let s = String::deserialize(d)?;
        Hex::parse(&s).ok_or_else(|| {
            serde::de::Error::custom(format!("{s:?} is not 0x and 16 lower-case hex digits"))
        })
    }
}

/// The tape hash (N10, C2): FNV-1a 64 over the canonical `to_ron` text. The parsed tape is
/// everything the loader reads (unknown fields are refused, and the few fields that may be
/// absent, the maker's `reserve`, a good's `untraded` and a desk's `plant`, mean off when absent
/// and are written when on), and
/// `to_ron` writes all of it in canonical order, so this covers everything `world_id` and every
/// `prefix_id` cover (the name, every basis, the schedule and the inline numbers too) and ignores
/// only what the loader ignores: comments, whitespace, line endings and list order.
pub fn tape_hash(t: &Tape) -> u64 {
    fnv1a_64(t.to_ron().as_bytes())
}

/// The build that ran: its commit, whether the tree was clean, and the target and compiler.
/// The cli stamps it (§3); a run that cannot read its git state records the commit `unknown`
/// and `dirty: true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    /// The commit, 40 hex digits, or `unknown`.
    pub commit: String,
    /// Whether the build's sources differed from the commit.
    pub dirty: bool,
    /// Cargo's target triple.
    pub target: String,
    /// `rustc -V`.
    pub rustc: String,
}

impl Build {
    /// `clean` or `dirty`.
    pub fn state(&self) -> &'static str {
        if self.dirty {
            "dirty"
        } else {
            "clean"
        }
    }
}

/// What names a run: the build, the tape hash and the world. The GUI keys its runs by it (GUI.md
/// §3.3); it leaves out `prefix_id`, which changes every tick, since the tape hash fixes the
/// schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunKey {
    /// The build.
    pub build: Build,
    /// [`tape_hash`] of the tape.
    pub tape_hash: Hex,
    /// The world's `world_id`.
    pub world_id: Hex,
}

/// The manifest's format.
pub const MANIFEST_FORMAT: u32 = 1;

/// The checkpoint a run resumed from, and the run that made it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumedFrom {
    /// The checkpoint's state tick.
    pub tick: u64,
    /// Its digest, over `(world_id, prefix_id, state, run)` (ENGINE §2.5).
    pub digest: Hex,
    /// Its state's hash.
    pub state_hash: Hex,
    /// The run that made it.
    pub parent: RunKey,
}

/// The hash file a run wrote: how many lines, the last, and FNV-1a 64 over its body, the lines
/// `{t} 0x{hash:016x}\n` without the `#` header, so `grep -v '^#' | fnv` checks it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashRecord {
    /// The lines.
    pub count: u64,
    /// The last line's tick and hash.
    #[serde(deserialize_with = "rustyecon_core::tape::raw::required")]
    pub last: Option<(u64, Hex)>,
    /// FNV-1a 64 over the body.
    pub digest: Hex,
}

/// A checkpoint this run wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointRecord {
    /// Its state tick.
    pub tick: u64,
    /// Its state's hash, which is the run's hash for that tick.
    pub state_hash: Hex,
    /// Its digest.
    pub digest: Hex,
    /// Its file, relative to the manifest's directory, with `/`.
    pub file: String,
}

/// The telemetry file a run wrote: `certify --telemetry` records it (§10, §12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryRecord {
    /// Its file, relative to the manifest's directory, with `/`.
    pub file: String,
    /// Its rows.
    pub rows: u64,
    /// FNV-1a 64 over its bytes.
    pub digest: Hex,
}

/// A run's record (§9): what it read (the build, the tape by hash, the world, any checkpoint it
/// resumed from) and what it made (its hash stream by digest, its checkpoints, its telemetry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "Manifest")]
pub struct Manifest {
    /// [`MANIFEST_FORMAT`].
    pub format: u32,
    /// The run's key.
    pub run: RunKey,
    /// The tape's name.
    pub tape: String,
    /// The genesis state's hash.
    pub genesis_hash: Hex,
    /// The clock's start.
    pub start: Date,
    /// The clock's ticks a year.
    pub ticks_per_year: u32,
    /// The state tick the run started from: 0, or a resumed checkpoint's.
    pub from: u64,
    /// The state tick it reached: the last tick folded, or `from`.
    pub until: u64,
    /// The checkpoint it resumed from.
    #[serde(deserialize_with = "rustyecon_core::tape::raw::required")]
    pub resumed_from: Option<ResumedFrom>,
    /// Its hash stream.
    pub hashes: HashRecord,
    /// The checkpoints it wrote, in order.
    pub checkpoints: Vec<CheckpointRecord>,
    /// Its telemetry.
    #[serde(deserialize_with = "rustyecon_core::tape::raw::required")]
    pub telemetry: Option<TelemetryRecord>,
}

/// Why a manifest does not read, or refuses a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The RON does not parse as a manifest.
    Parse(String),
    /// A manifest of another format.
    Format {
        /// Its format.
        found: u32,
    },
    /// A checkpoint that is not this run's state at its tick (§9): another world, another tick,
    /// or a state whose hash is not the one the run folded.
    OffRun(String),
    /// A file that is not a relative path.
    Path(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::Parse(m) => write!(f, "the manifest does not parse: {m}"),
            ManifestError::Format { found } => write!(
                f,
                "manifest format {found}, but this build reads format {MANIFEST_FORMAT} only"
            ),
            ManifestError::OffRun(m) => write!(f, "not a checkpoint of this run: {m}"),
            ManifestError::Path(p) => {
                write!(
                    f,
                    "{p:?} is not a relative path under the manifest's directory"
                )
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// Why a checkpoint does not verify against a manifest (D4, R16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    /// The manifest's run read another tape.
    OtherTape {
        /// The manifest's tape hash.
        manifest: Hex,
        /// The resuming tape's.
        tape: Hex,
    },
    /// The checkpoint belongs to another world than the manifest's run.
    OtherWorld {
        /// The manifest's world.
        manifest: Hex,
        /// The checkpoint's.
        checkpoint: Hex,
    },
    /// The manifest records no checkpoint at the checkpoint's tick.
    NotRecorded {
        /// The checkpoint's tick.
        tick: u64,
    },
    /// The record at its tick has another digest or state hash: a file replaced since.
    Differs {
        /// The tick.
        tick: u64,
        /// What differs.
        what: &'static str,
    },
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerifyError::OtherTape { manifest, tape } => write!(
                f,
                "the manifest's run read tape_hash {manifest}, not this tape's {tape}"
            ),
            VerifyError::OtherWorld {
                manifest,
                checkpoint,
            } => write!(
                f,
                "the checkpoint's world_id is {checkpoint}, not the manifest's {manifest}"
            ),
            VerifyError::NotRecorded { tick } => {
                write!(f, "the manifest records no checkpoint at tick {tick}")
            }
            VerifyError::Differs { tick, what } => write!(
                f,
                "the checkpoint at tick {tick} is not the one recorded: its {what} differs"
            ),
        }
    }
}

impl std::error::Error for VerifyError {}

/// A file as the manifest records it: relative, with `/`. An absolute path, a drive, or a `..`
/// segment is refused, so a manifest made on Windows and one made in WSL agree (§9).
fn relative(file: &str) -> Result<String, ManifestError> {
    let f = file.replace('\\', "/");
    let drive = f.as_bytes().get(1) == Some(&b':');
    let bad = f.is_empty()
        || f.starts_with('/')
        || drive
        || f.split('/').any(|seg| seg == ".." || seg.is_empty());
    if bad {
        Err(ManifestError::Path(file.to_string()))
    } else {
        Ok(f)
    }
}

fn pretty() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::new().new_line("\n".to_string())
}

impl Manifest {
    /// The record of a run that starts at `sim`'s state: genesis (with `genesis_hash` its hash),
    /// or a resumed checkpoint (`resumed_from`, which [`Manifest::verify`] returned).
    pub fn begin(
        run: RunKey,
        genesis_hash: u64,
        sim: &Sim,
        resumed_from: Option<ResumedFrom>,
    ) -> Manifest {
        let w = sim.world();
        Manifest {
            format: MANIFEST_FORMAT,
            run,
            tape: w.name.clone(),
            genesis_hash: Hex(genesis_hash),
            start: w.clock.start,
            ticks_per_year: w.clock.ticks_per_year,
            from: sim.tick(),
            until: sim.tick(),
            resumed_from,
            hashes: HashRecord {
                count: 0,
                last: None,
                digest: Hex(Fnv::new().finish()),
            },
            checkpoints: Vec::new(),
            telemetry: None,
        }
    }

    /// The hash of the state the run started from: genesis, or the checkpoint it resumed.
    fn start_hash(&self) -> u64 {
        match &self.resumed_from {
            Some(r) => r.state_hash.0,
            None => self.genesis_hash.0,
        }
    }

    /// The hash file's line for a state tick and its hash.
    pub fn line(tick: u64, hash: u64) -> String {
        format!("{tick} 0x{hash:016x}\n")
    }

    /// Fold one hash line, `{tick} 0x{hash:016x}\n`, into the record: `tick` is the state's tick
    /// after a step and `hash` that state's hash (`TickReport.hash`).
    pub fn tick(&mut self, tick: u64, hash: u64) {
        // FNV-1a's running state is the digest of what it has read, so the fold continues from
        // the recorded digest.
        let mut body = Fnv::resume(self.hashes.digest.0);
        body.write(Manifest::line(tick, hash).as_bytes());
        self.hashes.count += 1;
        self.hashes.last = Some((tick, Hex(hash)));
        self.hashes.digest = Hex(body.finish());
        self.until = tick;
    }

    /// Record a checkpoint this run wrote to `file`. It must be of this run's world, at the tick
    /// the run has reached, with the state the run last hashed there: the `TickReport.hash` of
    /// the tick before, or the starting state's hash when no tick has run. Otherwise it is
    /// [`ManifestError::OffRun`], so a record is the run's hash for its tick and never a
    /// checkpoint compared with itself.
    pub fn checkpoint(&mut self, cp: &Checkpoint, file: &str) -> Result<(), ManifestError> {
        if cp.world_id() != self.run.world_id.0 {
            return Err(ManifestError::OffRun(format!(
                "its world_id is {}, the run's {}",
                Hex(cp.world_id()),
                self.run.world_id
            )));
        }
        let tick = cp.state().tick();
        if tick != self.until {
            return Err(ManifestError::OffRun(format!(
                "its tick is {tick}, and the run is at {}",
                self.until
            )));
        }
        let expected = match self.hashes.last {
            Some((t, h)) if t == tick => h.0,
            _ => self.start_hash(),
        };
        let hash = state_hash(cp.state());
        if hash != expected {
            return Err(ManifestError::OffRun(format!(
                "its state hashes to {}, and the run hashed {} at tick {tick}",
                Hex(hash),
                Hex(expected)
            )));
        }
        self.checkpoints.push(CheckpointRecord {
            tick,
            state_hash: Hex(hash),
            digest: Hex(cp.digest()),
            file: relative(file)?,
        });
        Ok(())
    }

    /// Record the run's telemetry file.
    pub fn telemetry(&mut self, file: &str, rows: u64, digest: u64) -> Result<(), ManifestError> {
        self.telemetry = Some(TelemetryRecord {
            file: relative(file)?,
            rows,
            digest: Hex(digest),
        });
        Ok(())
    }

    /// The `#` lines a hash file opens with (§10): the format, the build, the tape and world,
    /// and the starting tick with the digest of any checkpoint resumed from.
    pub fn hashes_header(&self) -> String {
        let b = &self.run.build;
        let mut out = format!(
            "# rustyecon hashes 1\n# build {} {} {}\n# tape {} tape_hash {} world_id {}\n# from {}",
            b.commit,
            b.state(),
            b.target,
            self.tape,
            self.run.tape_hash,
            self.run.world_id,
            self.from
        );
        if let Some(r) = &self.resumed_from {
            out.push_str(&format!(" resumed {}", r.digest));
        }
        out.push('\n');
        out
    }

    /// The line the cli prints before a run's final hash (§10, amended at S2.4): the header's
    /// fields on one line, `run build <commit> <clean|dirty> <target> tape <name> tape_hash 0x…
    /// world_id 0x… from <tick>`, with ` resumed <digest>` on a resume, so a hash on stdout
    /// names its run as the hash file does (R16).
    pub fn run_line(&self) -> String {
        let b = &self.run.build;
        let mut out = format!(
            "run build {} {} {} tape {} tape_hash {} world_id {} from {}",
            b.commit,
            b.state(),
            b.target,
            self.tape,
            self.run.tape_hash,
            self.run.world_id,
            self.from
        );
        if let Some(r) = &self.resumed_from {
            out.push_str(&format!(" resumed {}", r.digest));
        }
        out
    }

    /// Verify a checkpoint against this manifest before a resume (D4, R16): the manifest's run
    /// read the resuming tape (`tape_hash`), the checkpoint is of the run's world, and the run
    /// recorded a checkpoint at its tick with the same digest and state hash. It catches mix-ups,
    /// not forgery (§9). Returns what the resumed run's manifest records.
    pub fn verify(&self, cp: &Checkpoint, tape_hash: u64) -> Result<ResumedFrom, VerifyError> {
        if self.run.tape_hash.0 != tape_hash {
            return Err(VerifyError::OtherTape {
                manifest: self.run.tape_hash,
                tape: Hex(tape_hash),
            });
        }
        if self.run.world_id.0 != cp.world_id() {
            return Err(VerifyError::OtherWorld {
                manifest: self.run.world_id,
                checkpoint: Hex(cp.world_id()),
            });
        }
        let tick = cp.state().tick();
        let Some(rec) = self.checkpoints.iter().rev().find(|c| c.tick == tick) else {
            return Err(VerifyError::NotRecorded { tick });
        };
        if rec.digest.0 != cp.digest() {
            return Err(VerifyError::Differs {
                tick,
                what: "digest",
            });
        }
        let hash = state_hash(cp.state());
        if rec.state_hash.0 != hash {
            return Err(VerifyError::Differs {
                tick,
                what: "state hash",
            });
        }
        Ok(ResumedFrom {
            tick,
            digest: rec.digest,
            state_hash: rec.state_hash,
            parent: self.run.clone(),
        })
    }

    /// The manifest as RON, opening `Manifest(`.
    pub fn to_ron(&self) -> String {
        match ron::ser::to_string_pretty(self, pretty()) {
            Ok(s) => format!("Manifest{s}\n"),
            Err(e) => format!("/* the manifest does not serialise: {e} */\n"),
        }
    }

    /// Read a manifest. A format other than [`MANIFEST_FORMAT`] is refused as such.
    pub fn from_ron(s: &str) -> Result<Manifest, ManifestError> {
        #[derive(Deserialize)]
        #[serde(rename = "Manifest")]
        struct Probe {
            format: u32,
        }
        if let Ok(p) = ron::from_str::<Probe>(s) {
            if p.format != MANIFEST_FORMAT {
                return Err(ManifestError::Format { found: p.format });
            }
        }
        let m: Manifest = ron::from_str(s).map_err(|e| ManifestError::Parse(e.to_string()))?;
        if m.format != MANIFEST_FORMAT {
            return Err(ManifestError::Format { found: m.format });
        }
        Ok(m)
    }
}
