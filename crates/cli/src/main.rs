//! The `rustyecon` binary: a thin frontend over `rustyecon-engine` (docs/ENGINE.md §8). It parses
//! arguments, reads and writes files, maps file extensions to formats, and turns results into
//! exit codes. The tick loop, resume and the shadow-replay audit live in the engine. Behaviour
//! comes from the tape, never from a command-line switch (R4, E1): July's `--agents`,
//! `--price-rule` and `--supply-rule` do not return (N13).
//!
//! Exit codes: 0 ok; 1 a load or argument error (an unknown checkpoint extension included);
//! 2 a run error, with the error or ledger line and the last good tick on stderr; 3 an I/O or
//! checkpoint error, a refused resume included; 4 a replay mismatch.

use clap::{Args, Parser, Subcommand, ValueEnum};
use rustyecon_engine::prelude::*;
use rustyecon_engine::registry;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Load or argument error.
const LOAD: u8 = 1;
/// Run error.
const RUN: u8 = 2;
/// I/O or checkpoint error, a refused resume included.
const IO: u8 = 3;
/// Replay mismatch.
const MISMATCH: u8 = 4;

#[derive(Parser)]
#[command(
    name = "rustyecon",
    version,
    about = "Run, resume, replay and list rustyecon tapes"
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
    /// Resume a checkpoint (.bin or .ron) under a tape until the state's tick is --until.
    Resume {
        /// The checkpoint: `.bin` (binary) or `.ron` (human-readable).
        checkpoint: PathBuf,
        /// The tape the checkpoint belongs to; a dated edit at or after its tick is allowed.
        #[arg(long)]
        tape: PathBuf,
        /// Run until the state's tick is this.
        #[arg(long)]
        until: u64,
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
    /// List every number a tape feeds a run: each param with its unit, per-tick value and
    /// basis, and the inline numbers under their entries' bases.
    Registry {
        /// The tape (RON).
        tape: PathBuf,
    },
}

#[derive(Args)]
struct Output {
    /// Write the final state, and any --checkpoint-every states, to DIR/tick_{:08}.{bin|ron}.
    #[arg(long, value_name = "DIR")]
    out: Option<PathBuf>,
    /// Also checkpoint every state the run reaches whose tick is a multiple of N.
    #[arg(long, value_name = "N", requires = "out", value_parser = clap::value_parser!(u64).range(1..))]
    checkpoint_every: Option<u64>,
    /// The checkpoint format.
    #[arg(long, value_enum, default_value_t = Format::Bin)]
    format: Format,
    /// Write one `{tick} 0x{hash:016x}` line per tick run, the state's tick after the step.
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

/// Write the current state to `dir/tick_{:08}.{bin|ron}`.
fn save(sim: &Sim, dir: &Path, format: Format) -> Result<(), Exit> {
    let cp = sim
        .checkpoint()
        .map_err(|e| Exit::new(IO, format!("cannot checkpoint: {e}")))?;
    let (ext, bytes) = match format {
        Format::Bin => ("bin", cp.to_bytes()),
        Format::Ron => ("ron", cp.to_ron().into_bytes()),
    };
    fs::create_dir_all(dir).map_err(|e| io_error("cannot create the directory", dir, e))?;
    let path = dir.join(format!("tick_{:08}.{ext}", cp.state().tick()));
    fs::write(&path, bytes).map_err(|e| io_error("cannot write the checkpoint", &path, e))
}

/// The hashes file, flushed before any exit.
struct Hashes(Option<(PathBuf, BufWriter<File>)>);

impl Hashes {
    fn open(path: Option<&PathBuf>) -> Result<Hashes, Exit> {
        match path {
            None => Ok(Hashes(None)),
            Some(p) => {
                let f = File::create(p).map_err(|e| io_error("cannot create", p, e))?;
                Ok(Hashes(Some((p.clone(), BufWriter::new(f)))))
            }
        }
    }
    fn line(&mut self, tick: u64, hash: u64) -> Result<(), Exit> {
        if let Some((p, w)) = &mut self.0 {
            writeln!(w, "{tick} 0x{hash:016x}").map_err(|e| io_error("cannot write", p, e))?;
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

/// Step `sim` until the state's tick is `until`, writing hashes and checkpoints as asked. A
/// failed step is exit 2; a failed write is exit 3, and no further tick runs.
fn drive(mut sim: Sim, until: u64, out: &Output) -> Result<(), Exit> {
    if until < sim.tick() {
        return Err(Exit::new(
            LOAD,
            format!("--until {until} is before the starting tick {}", sim.tick()),
        ));
    }
    let mut hashes = Hashes::open(out.hashes.as_ref())?;
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
        let t = sim.tick();
        hashes.line(t, sim.hash())?;
        if let (Some(dir), Some(n)) = (&out.out, out.checkpoint_every) {
            if t.is_multiple_of(n) {
                hashes.flush()?;
                save(&sim, dir, out.format)?;
                saved = Some(t);
            }
        }
    }
    hashes.flush()?;
    if let Some(dir) = &out.out {
        if saved != Some(sim.tick()) {
            save(&sim, dir, out.format)?;
        }
    }
    println!("{} 0x{:016x}", sim.tick(), sim.hash());
    Ok(())
}

fn run(cmd: Cmd) -> Result<(), Exit> {
    match cmd {
        Cmd::Run { tape, until, out } => {
            let t = load_tape(&tape)?;
            let sim = Sim::new(&t).map_err(|e| load_error(&tape, e))?;
            drive(sim, until, &out)
        }
        Cmd::Resume {
            checkpoint,
            tape,
            until,
            out,
        } => {
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
            let t = load_tape(&tape)?;
            let cp = if binary {
                let b = fs::read(&checkpoint)
                    .map_err(|e| io_error("cannot read the checkpoint", &checkpoint, e))?;
                Checkpoint::from_bytes(&b)
            } else {
                let s = fs::read_to_string(&checkpoint)
                    .map_err(|e| io_error("cannot read the checkpoint", &checkpoint, e))?;
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
            drive(sim, until, &out)
        }
        Cmd::Replay { tape, until } => {
            let t = load_tape(&tape)?;
            match audit_replay(&t, until) {
                Ok(hash) => {
                    println!("replay matched every tick");
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
            println!("path\tvalue\tunit\tper tick\tbasis");
            for line in lines {
                println!("{line}");
            }
            Ok(())
        }
    }
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
