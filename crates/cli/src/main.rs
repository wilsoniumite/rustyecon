//! The `rustyecon` binary: a thin frontend over `rustyecon-engine` (docs/ENGINE.md §8) and
//! `rustyecon-certify` (docs/CERTIFY.md §10). It parses arguments, reads and writes files, maps
//! file extensions to formats, and turns results into exit codes. The tick loop, resume and the
//! shadow-replay audit live in the engine; the batteries, the certificate and the manifest in
//! certify. Behaviour comes from the tape, never from a command-line switch (R4, E1): July's
//! `--agents`, `--price-rule` and `--supply-rule` do not return (N13).
//!
//! Every hash it prints or writes names its run (R16): a `--hashes` file opens with `#` lines
//! naming the build, the tape by `tape_hash`, the `world_id` and the starting tick, and stdout
//! prints a `run …` line with the same fields before the final hash. With `--out`, a run writes
//! `manifest.ron` beside its checkpoints, and a resume verifies its checkpoint against the
//! manifest of the run that made it.
//!
//! `worldgen` compiles a world's tables into a tape through `rustyecon-worldgen`, which reads no
//! file: the cli reads the tables and writes the tape. `licences` prints the licence and
//! attribution of the county atlas the binary bundles (data/atlas/, ODbL 1.0).
//!
//! Exit codes: 0 ok (for `certify`, PASS); 1 a load or argument error (an unknown checkpoint
//! extension, criteria that do not load or fit, and an `--out` holding the manifest a resume
//! verifies against included); 2 a run error, with the error or ledger line and the last good
//! tick on stderr; 3 an I/O or checkpoint error, a refused or unverified resume included; 4 a
//! replay mismatch; 5 a certificate whose verdict is FAIL or UNSCORED, written before the exit.

use certify::telemetry::TelemetryWriter;
use certify::{tape_hash, Build, Criteria, Hex, Manifest, RunKey, Scoring, Verdict};
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use rustyecon_engine::prelude::*;
use rustyecon_engine::registry;
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::{compile, compile_stage, StageTables, Tables};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Load or argument error.
const LOAD: u8 = 1;
/// Run error.
const RUN: u8 = 2;
/// I/O or checkpoint error, a refused or unverified resume included.
const IO: u8 = 3;
/// Replay mismatch.
const MISMATCH: u8 = 4;
/// A certificate whose verdict is FAIL or UNSCORED (N4, C4).
const VERDICT: u8 = 5;

/// The manifest's file name in an `--out` directory.
const MANIFEST: &str = "manifest.ron";
/// `certify`'s outputs in its `--out` directory (docs/CERTIFY.md §10).
const CERTIFICATE: &str = "certificate.ron";
const HASHES: &str = "hashes.txt";
const TELEMETRY: &str = "telemetry.parquet";

#[derive(Parser)]
#[command(
    name = "rustyecon",
    version,
    about = "Run, resume, replay, list and certify rustyecon tapes"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a tape from its genesis until the state's tick is --until.
    Run {
        /// The tape (RON).
        tape: PathBuf,
        /// Run until the state's tick is this.
        #[arg(long)]
        until: u64,
        #[command(flatten)]
        out: Output,
    },
    /// Resume a checkpoint (.bin or .ron) under a tape until the state's tick is --until. The
    /// checkpoint is verified against the manifest of the run that made it.
    Resume {
        /// The checkpoint: `.bin` (binary) or `.ron` (human-readable).
        checkpoint: PathBuf,
        /// The tape of the run that made the checkpoint: its tape_hash must be the one that
        /// run's manifest records.
        #[arg(long)]
        tape: PathBuf,
        /// Run until the state's tick is this.
        #[arg(long)]
        until: u64,
        /// The manifest of the run that made the checkpoint; by default `manifest.ron` beside
        /// the checkpoint.
        #[arg(long, value_name = "FILE")]
        manifest: Option<PathBuf>,
        #[command(flatten)]
        out: Output,
    },
    /// Run a tape beside a shadow that only applies each tick's deltas, comparing hashes every
    /// tick.
    Replay {
        /// The tape (RON).
        tape: PathBuf,
        /// Run until the state's tick is this.
        #[arg(long)]
        until: u64,
    },
    /// List every number a tape feeds a run: each param with its unit, its use, each place it
    /// is read with that read's conversion and per-tick value, and its basis; then the inline
    /// numbers under their entries' bases.
    Registry {
        /// The tape (RON).
        tape: PathBuf,
    },
    /// Certify a run of a tape from genesis: score it against dated criteria, or run it unscored
    /// to --until, and write DIR/certificate.ron, DIR/manifest.ron and DIR/hashes.txt, with
    /// --telemetry DIR/telemetry.parquet too. Exit 0 is PASS; 5 is FAIL or UNSCORED, written
    /// before the exit.
    #[command(group(ArgGroup::new("scoring").required(true).args(["criteria", "until"])))]
    Certify {
        /// The tape (RON).
        tape: PathBuf,
        /// The dated criteria file, `<tape>-<YYYY-MM-DD>.ron`; the run goes to their `until`.
        #[arg(long, value_name = "FILE")]
        criteria: Option<PathBuf>,
        /// With no criteria, run to this state tick; the verdict is UNSCORED at best (N4).
        #[arg(long)]
        until: Option<u64>,
        /// The directory the certificate, manifest and hash file go to.
        #[arg(long, value_name = "DIR")]
        out: PathBuf,
        /// Also write the base run's telemetry as tidy long Parquet.
        #[arg(long)]
        telemetry: bool,
    },
    /// Compile a world's tables and the bundled county atlas into a tape (crates/worldgen;
    /// docs/demo/WORLD.md): read DIR/world.csv, counties.csv, regions.csv, history.csv and
    /// lenses.csv, check them, solve every county's oracle at genesis and after every step,
    /// and write the tape to --out. Without --out it checks and reports only. With --stage it
    /// also reads the stage's machine_types.csv and stage-<STAGE>.csv and compiles the goods
    /// chain's stage (docs/demo/WORLD-V2.md).
    Worldgen {
        /// The world's directory, such as worlds/demo-gb.
        dir: PathBuf,
        /// Write the tape here.
        #[arg(short, long, value_name = "FILE")]
        out: Option<PathBuf>,
        /// Compile the goods chain's stage, such as v2a1 (the demo's second pass). Without it
        /// the compiler writes v1's tape.
        #[arg(long, value_name = "STAGE")]
        stage: Option<String>,
    },
    /// Print the licence and attribution of the data this binary bundles: the county atlas,
    /// data/atlas/gb.atlas.ron, under the ODbL 1.0 (data/atlas/LICENSE and ATTRIBUTION).
    Licences,
}

#[derive(Args)]
struct Output {
    /// Write the final state, and any --checkpoint-every states, to DIR/tick_{:08}.{bin|ron},
    /// and the run's manifest to DIR/manifest.ron.
    #[arg(long, value_name = "DIR")]
    out: Option<PathBuf>,
    /// Also checkpoint every state the run reaches whose tick is a multiple of N.
    #[arg(long, value_name = "N", requires = "out", value_parser = clap::value_parser!(u64).range(1..))]
    checkpoint_every: Option<u64>,
    /// The checkpoint format.
    #[arg(long, value_enum, default_value_t = Format::Bin)]
    format: Format,
    /// Write the run's `#` header, then one `{tick} 0x{hash:016x}` line per tick run, the
    /// state's tick after the step.
    #[arg(long, value_name = "FILE")]
    hashes: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    /// Binary: `RUSTYECK`, the format, then bincode 1.
    Bin,
    /// Human-readable RON.
    Ron,
}

/// A failure: its exit code and what goes to stderr.
struct Exit {
    code: u8,
    message: String,
}

impl Exit {
    fn new(code: u8, message: impl Into<String>) -> Exit {
        Exit {
            code,
            message: message.into(),
        }
    }
}

fn io_error(what: &str, path: &Path, e: std::io::Error) -> Exit {
    Exit::new(IO, format!("{what} {}: {e}", path.display()))
}

/// The build this binary was made from (docs/CERTIFY.md §3), stamped by `build.rs`. Anything
/// but an explicit clean stamp reads as dirty (fail closed).
fn build() -> Build {
    Build {
        commit: env!("RUSTYECON_COMMIT").to_string(),
        dirty: env!("RUSTYECON_DIRTY") != "false",
        target: env!("RUSTYECON_TARGET").to_string(),
        rustc: env!("RUSTYECON_RUSTC").to_string(),
    }
}

/// What names a run of `t` in `w` by this build.
fn run_key(t: &Tape, w: &World) -> RunKey {
    RunKey {
        build: build(),
        tape_hash: Hex(tape_hash(t)),
        world_id: Hex(w.world_id),
    }
}

fn load_tape(path: &Path) -> Result<Tape, Exit> {
    let text = fs::read_to_string(path).map_err(|e| {
        Exit::new(
            LOAD,
            format!("cannot read the tape {}: {e}", path.display()),
        )
    })?;
    Tape::from_ron(&text).map_err(|e| {
        Exit::new(
            LOAD,
            format!("the tape {} does not load: {e}", path.display()),
        )
    })
}

fn load_error(path: &Path, e: LoadError) -> Exit {
    Exit::new(
        LOAD,
        format!("the tape {} does not load: {e}", path.display()),
    )
}

/// Write `bytes` to `path` through `path.tmp` and a rename, so a reader never sees half a file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Exit> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("out");
    let tmp = path.with_file_name(format!("{name}.tmp"));
    fs::write(&tmp, bytes).map_err(|e| io_error("cannot write", &tmp, e))?;
    fs::rename(&tmp, path).map_err(|e| io_error("cannot write", path, e))
}

/// Whether two paths name one existing file.
fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// Write the current state to `dir/tick_{:08}.{bin|ron}`, record it in the manifest, and write
/// the manifest beside it.
fn save(sim: &Sim, dir: &Path, format: Format, manifest: &mut Manifest) -> Result<(), Exit> {
    let cp = sim
        .checkpoint()
        .map_err(|e| Exit::new(IO, format!("cannot checkpoint: {e}")))?;
    let (ext, bytes) = match format {
        Format::Bin => ("bin", cp.to_bytes()),
        Format::Ron => ("ron", cp.to_ron().into_bytes()),
    };
    fs::create_dir_all(dir).map_err(|e| io_error("cannot create the directory", dir, e))?;
    let name = format!("tick_{:08}.{ext}", cp.state().tick());
    let path = dir.join(&name);
    fs::write(&path, bytes).map_err(|e| io_error("cannot write the checkpoint", &path, e))?;
    manifest
        .checkpoint(&cp, &name)
        .map_err(|e| Exit::new(IO, format!("cannot record {}: {e}", path.display())))?;
    write_atomic(&dir.join(MANIFEST), manifest.to_ron().as_bytes())
}

/// The hashes file, flushed before any exit.
struct Hashes(Option<(PathBuf, BufWriter<File>)>);

impl Hashes {
    /// Create the file and write the run's `#` header.
    fn open(path: Option<&PathBuf>, header: &str) -> Result<Hashes, Exit> {
        match path {
            None => Ok(Hashes(None)),
            Some(p) => {
                let f = File::create(p).map_err(|e| io_error("cannot create", p, e))?;
                let mut w = BufWriter::new(f);
                w.write_all(header.as_bytes())
                    .map_err(|e| io_error("cannot write", p, e))?;
                Ok(Hashes(Some((p.clone(), w))))
            }
        }
    }
    fn line(&mut self, tick: u64, hash: u64) -> Result<(), Exit> {
        if let Some((p, w)) = &mut self.0 {
            w.write_all(Manifest::line(tick, hash).as_bytes())
                .map_err(|e| io_error("cannot write", p, e))?;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Exit> {
        if let Some((p, w)) = &mut self.0 {
            w.flush().map_err(|e| io_error("cannot write", p, e))?;
        }
        Ok(())
    }
}

/// Step `sim` until the state's tick is `until`, writing hashes, checkpoints and the manifest as
/// asked. A failed step is exit 2; a failed write is exit 3, and no further tick runs.
fn drive(mut sim: Sim, mut manifest: Manifest, until: u64, out: &Output) -> Result<(), Exit> {
    if until < sim.tick() {
        return Err(Exit::new(
            LOAD,
            format!("--until {until} is before the starting tick {}", sim.tick()),
        ));
    }
    let mut hashes = Hashes::open(out.hashes.as_ref(), &manifest.hashes_header())?;
    let mut saved = None;
    while sim.tick() < until {
        if let Err(e) = sim.step() {
            hashes.flush()?;
            let last = match sim.last_report() {
                Some(r) => format!(
                    "the last good tick is {} (state tick {})",
                    r.tick,
                    r.tick + 1
                ),
                None => format!("no tick completed; the run started at tick {}", e.tick),
            };
            return Err(Exit::new(RUN, format!("run error: {e}\n{last}")));
        }
        let (t, h) = (sim.tick(), sim.hash());
        hashes.line(t, h)?;
        manifest.tick(t, h);
        if let (Some(dir), Some(n)) = (&out.out, out.checkpoint_every) {
            if t.is_multiple_of(n) {
                hashes.flush()?;
                save(&sim, dir, out.format, &mut manifest)?;
                saved = Some(t);
            }
        }
    }
    hashes.flush()?;
    if let Some(dir) = &out.out {
        if saved != Some(sim.tick()) {
            save(&sim, dir, out.format, &mut manifest)?;
        }
    }
    println!("{}", manifest.run_line());
    println!("{} 0x{:016x}", sim.tick(), sim.hash());
    Ok(())
}

/// Resume a checkpoint (docs/CERTIFY.md §10, C9): decode it, resume it under the tape (whose
/// refusals stand), then verify it against the manifest of the run that made it: the same
/// `tape_hash`, the same world, and a record at its tick with its digest and state hash.
fn resume(
    checkpoint: &Path,
    tape: &Path,
    until: u64,
    manifest: Option<&Path>,
    out: &Output,
) -> Result<(), Exit> {
    let binary = match checkpoint.extension().and_then(|e| e.to_str()) {
        Some("bin") => true,
        Some("ron") => false,
        _ => {
            return Err(Exit::new(
                LOAD,
                format!("{}: a checkpoint is .bin or .ron", checkpoint.display()),
            ))
        }
    };
    let t = load_tape(tape)?;
    let cp = if binary {
        let b = fs::read(checkpoint)
            .map_err(|e| io_error("cannot read the checkpoint", checkpoint, e))?;
        Checkpoint::from_bytes(&b)
    } else {
        let s = fs::read_to_string(checkpoint)
            .map_err(|e| io_error("cannot read the checkpoint", checkpoint, e))?;
        Checkpoint::from_ron(&s)
    }
    .map_err(|e| Exit::new(IO, format!("{}: {e}", checkpoint.display())))?;
    let sim = Sim::resume(&t, &cp).map_err(|e| {
        let code = match e {
            ResumeError::Load(_) => LOAD,
            _ => IO,
        };
        Exit::new(code, format!("cannot resume {}: {e}", checkpoint.display()))
    })?;
    let mpath = manifest.map_or_else(
        || {
            checkpoint
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(MANIFEST)
        },
        Path::to_path_buf,
    );
    let unverified = |why: String| Exit::new(IO, format!("unverified checkpoint: {why}"));
    let text = fs::read_to_string(&mpath)
        .map_err(|e| unverified(format!("cannot read the manifest {}: {e}", mpath.display())))?;
    let parent = Manifest::from_ron(&text)
        .map_err(|e| unverified(format!("the manifest {}: {e}", mpath.display())))?;
    if let Some(dir) = &out.out {
        if same_file(&dir.join(MANIFEST), &mpath) {
            return Err(Exit::new(
                LOAD,
                format!(
                    "--out {} holds the manifest this resume verifies against, which the new \
                     run's manifest would replace; write to another directory",
                    dir.display()
                ),
            ));
        }
    }
    let key = run_key(&t, sim.world());
    let from = parent
        .verify(&cp, key.tape_hash.0)
        .map_err(|e| unverified(format!("{e} (manifest {})", mpath.display())))?;
    let genesis = Sim::new(&t).map_err(|e| load_error(tape, e))?.hash();
    let record = Manifest::begin(key, genesis, &sim, Some(from));
    drive(sim, record, until, out)
}

/// Certify a run of a tape (docs/CERTIFY.md §10): the certificate, the manifest and the hash
/// file, with `telemetry` the base run's Parquet, then the rendering. Once the run has started,
/// the certificate is written before any exit.
fn certify_run(
    tape: &Path,
    criteria: Option<&Path>,
    until: Option<u64>,
    out: &Path,
    telemetry: bool,
) -> Result<(), Exit> {
    let t = load_tape(tape)?;
    let loaded = match criteria {
        Some(p) => {
            let text = fs::read_to_string(p).map_err(|e| {
                Exit::new(
                    LOAD,
                    format!("cannot read the criteria {}: {e}", p.display()),
                )
            })?;
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let c = Criteria::from_ron(&text, &name).map_err(|e| {
                Exit::new(
                    LOAD,
                    format!("the criteria {} do not load: {e}", p.display()),
                )
            })?;
            Some((c, name))
        }
        None => None,
    };
    let scoring = match (&loaded, until) {
        (Some((c, name)), _) => Scoring::Criteria {
            criteria: c,
            file: name,
        },
        (None, Some(u)) => Scoring::Unscored { until: u },
        (None, None) => {
            return Err(Exit::new(
                LOAD,
                "certify needs --criteria FILE or --until T",
            ))
        }
    };
    fs::create_dir_all(out).map_err(|e| io_error("cannot create the directory", out, e))?;
    let b = build();
    let tele_path = out.join(TELEMETRY);
    let mut writer = if telemetry {
        let sim = Sim::new(&t).map_err(|e| load_error(tape, e))?;
        let key = run_key(&t, sim.world());
        let f = File::create(&tele_path).map_err(|e| io_error("cannot create", &tele_path, e))?;
        let w = TelemetryWriter::new(BufWriter::new(f), sim.world(), &key)
            .map_err(|e| Exit::new(IO, format!("{}: {e}", tele_path.display())))?;
        Some(w)
    } else {
        None
    };
    let mut failed: Option<String> = None;
    let result = certify::certify(&t, scoring, &b, &mut |r| {
        if failed.is_none() {
            if let Some(w) = writer.as_mut() {
                if let Err(e) = w.push(r) {
                    failed = Some(e.to_string());
                }
            }
        }
    });
    let certified = match result {
        Ok(c) => c,
        Err(e) => {
            if telemetry {
                // Nothing ran, so the file holds no run; it is this command's own output.
                let _ = fs::remove_file(&tele_path);
            }
            return Err(Exit::new(LOAD, format!("certify: {e}")));
        }
    };
    let mut manifest = certified.manifest;
    if let Some(w) = writer {
        if failed.is_none() {
            match w.finish() {
                Ok(fin) => {
                    if let Err(e) = manifest.telemetry(TELEMETRY, fin.rows, fin.digest) {
                        failed = Some(e.to_string());
                    }
                }
                Err(e) => failed = Some(e.to_string()),
            }
        }
        if failed.is_some() {
            // A partial file is not the run's telemetry, and the manifest does not name it.
            let _ = fs::remove_file(&tele_path);
        }
    }
    let cert = &certified.certificate;
    write_atomic(&out.join(HASHES), certified.hashes.as_bytes())?;
    write_atomic(&out.join(CERTIFICATE), cert.to_ron().as_bytes())?;
    write_atomic(&out.join(MANIFEST), manifest.to_ron().as_bytes())?;
    print!("{}", cert.render());
    if let Some(e) = failed {
        return Err(Exit::new(
            IO,
            format!("the telemetry {} was not written: {e}", tele_path.display()),
        ));
    }
    match cert.verdict() {
        Verdict::Pass => Ok(()),
        v => Err(Exit::new(
            VERDICT,
            format!(
                "verdict {v}: the certificate is {}",
                out.join(CERTIFICATE).display()
            ),
        )),
    }
}

fn run(cmd: Cmd) -> Result<(), Exit> {
    match cmd {
        Cmd::Run { tape, until, out } => {
            let t = load_tape(&tape)?;
            let sim = Sim::new(&t).map_err(|e| load_error(&tape, e))?;
            let manifest = Manifest::begin(run_key(&t, sim.world()), sim.hash(), &sim, None);
            drive(sim, manifest, until, &out)
        }
        Cmd::Resume {
            checkpoint,
            tape,
            until,
            manifest,
            out,
        } => resume(&checkpoint, &tape, until, manifest.as_deref(), &out),
        Cmd::Replay { tape, until } => {
            let t = load_tape(&tape)?;
            let sim = Sim::new(&t).map_err(|e| load_error(&tape, e))?;
            let named = Manifest::begin(run_key(&t, sim.world()), sim.hash(), &sim, None);
            match audit_replay(&t, until) {
                Ok(hash) => {
                    println!("replay matched every tick");
                    println!("{}", named.run_line());
                    println!("{until} 0x{hash:016x}");
                    Ok(())
                }
                Err(e) => {
                    let code = match e {
                        ReplayError::Load(_) => LOAD,
                        ReplayError::Run(_) => RUN,
                        ReplayError::Mismatch { .. } | ReplayError::Shadow { .. } => MISMATCH,
                    };
                    Err(Exit::new(code, format!("replay: {e}")))
                }
            }
        }
        Cmd::Registry { tape } => {
            let t = load_tape(&tape)?;
            let lines = registry(&t).map_err(|e| load_error(&tape, e))?;
            println!("path\tvalue\tunit use\tmethod per tick at site; …\tbasis");
            for line in lines {
                println!("{line}");
            }
            Ok(())
        }
        Cmd::Certify {
            tape,
            criteria,
            until,
            out,
            telemetry,
        } => certify_run(&tape, criteria.as_deref(), until, &out, telemetry),
        Cmd::Worldgen { dir, out, stage } => worldgen(&dir, out.as_deref(), stage.as_deref()),
        Cmd::Licences => {
            print!("{}", rustyecon_worldgen::atlas::licences());
            Ok(())
        }
    }
}

/// Compile the world in `dir` (crates/worldgen): the cli reads the tables and writes the tape;
/// the compiler reads no file. The tape is loaded and resolved before it is written, and its
/// `tape_hash` printed.
fn worldgen(dir: &Path, out: Option<&Path>, stage: Option<&str>) -> Result<(), Exit> {
    let read = |name: &str| {
        let path = dir.join(name);
        fs::read_to_string(&path).map_err(|e| io_error("cannot read", &path, e))
    };
    let tables = Tables {
        world: read(Tables::FILES[0])?,
        counties: read(Tables::FILES[1])?,
        regions: read(Tables::FILES[2])?,
        history: read(Tables::FILES[3])?,
        lenses: read(Tables::FILES[4])?,
    };
    let atlas = Atlas::gb().map_err(|e| Exit::new(LOAD, format!("{e}")))?;
    let compiled = match stage {
        None => compile(&tables, &atlas),
        Some(key) => {
            let [types, settings] = StageTables::files(key);
            let st = StageTables {
                key: key.to_string(),
                machine_types: read(&types)?,
                stage: read(&settings)?,
            };
            compile_stage(&tables, &st, &atlas)
        }
    }
    .map_err(|e| Exit::new(LOAD, format!("{e}")))?;
    let tape = Tape::from_ron(&compiled.tape).map_err(|e| {
        Exit::new(
            LOAD,
            format!("worldgen: the compiled tape does not load: {e}"),
        )
    })?;
    let sim = Sim::new(&tape).map_err(|e| {
        Exit::new(
            LOAD,
            format!("worldgen: the compiled tape does not resolve: {e}"),
        )
    })?;
    let s = &compiled.summary;
    println!(
        "worldgen {}: {} counties, {} steps on {} county dates, every one solved Interior",
        dir.display(),
        s.counties,
        s.events,
        s.step_dates
    );
    println!(
        "  funding: the provider's own baskets per unit of N at least {:.3} ({})",
        s.min_funding.0, s.min_funding.1
    );
    println!(
        "  participation {:.3} ({}) to {:.3} ({}); x* {:.3} ({}) to {:.3} ({})",
        s.min_participation.0,
        s.min_participation.1,
        s.max_participation.0,
        s.max_participation.1,
        s.min_x.0,
        s.min_x.1,
        s.max_x.0,
        s.max_x.1
    );
    println!(
        "  largest move at one date: relative prices and technique {:.4} in log ({}), \
         quantities {:.4} ({})",
        s.max_step_prices.0, s.max_step_prices.1, s.max_step_quantities.0, s.max_step_quantities.1
    );
    println!(
        "  largest move over a trailing year: relative prices and technique {:.4} in log ({}), \
         quantities {:.4} ({})",
        s.max_year_prices.0, s.max_year_prices.1, s.max_year_quantities.0, s.max_year_quantities.1
    );
    if stage.is_some() {
        println!(
            "  the chain (unit 1g): meets v1's 1a point within {:.1e} relative ({}); its own \
             prices and quantities move at most {:.4} in log at one date ({}), {:.4} over a \
             trailing year ({})",
            s.max_collapse.0,
            s.max_collapse.1,
            s.max_step_chain.0,
            s.max_step_chain.1,
            s.max_year_chain.0,
            s.max_year_chain.1
        );
    }
    println!(
        "tape {:?}: {} bytes, tape_hash 0x{:016x}, world_id 0x{:016x}",
        tape.header.name,
        compiled.tape.len(),
        tape_hash(&tape),
        sim.world().world_id
    );
    if let Some(path) = out {
        write_atomic(path, compiled.tape.as_bytes())?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Help and version go to stdout with 0; every argument error is 1, not clap's 2,
            // which is the run-error code here.
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::from(LOAD)
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match run(cli.cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e.message);
            ExitCode::from(e.code)
        }
    }
}
