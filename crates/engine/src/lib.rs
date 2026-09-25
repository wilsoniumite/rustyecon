//! The engine (docs/ENGINE.md §7): one [`Sim`] owns a run. It loads a tape, steps one tick at a
//! time in PLAN §3's phase order, runs to a given tick, checkpoints, resumes, and audits a
//! replay, returning each tick's [`TickReport`]. Frontends, the `rustyecon` binary among them,
//! drive it and read it through read-only accessors.
//!
//! The invariants of §0 hold here:
//!
//! - **E1.** No frontend mutates state: every intervention is a dated tape event, or a tape edit
//!   followed by a rerun ([`Sim::new`]) or a resume ([`Sim::resume`]). There is no setter, and
//!   this crate's API holds no writer of the state: it re-exports core's read-only types, never
//!   core itself, so `apply`, `resolve` and the ledger are out of a frontend's reach. A
//!   checkpoint's bytes are the one other input; its digest catches corruption, not forgery
//!   (docs/ENGINE.md §7.6).
//! - **E2.** No global state and no I/O: bytes in, bytes out. Paths live in the cli.
//! - **E3.** Every type a frontend holds is `Send + Sync + 'static`, so a `Sim` can run on a
//!   worker thread and send its reports over a channel.
//! - **E4.** No method hands out `&mut` to the state or the world; reading changes no hash.
//! - **E5.** A failed step poisons the `Sim` until it is rebuilt.
//! - **E6, E7.** Every hook reads the state as its phase began, and what it returns is checked
//!   against its whitelist before anything is applied (R13, R2).
//!
//! The engine re-exports markets and agents under their own names, and a [`prelude`] of the
//! types a frontend names, core's read-only ones among them, so a frontend depends on this crate
//! alone. Core itself is not re-exported. These do not compile:
//!
//! ```compile_fail
//! use rustyecon_engine::rustyecon_core::apply;
//! ```
//!
//! ```compile_fail
//! use rustyecon_engine::rustyecon_agents::rustyecon_core::apply;
//! ```
//!
//! ```compile_fail
//! use rustyecon_engine::rustyecon_markets::rustyecon_core::apply;
//! ```
//!
//! ```compile_fail
//! use rustyecon_engine::prelude::{apply, resolve, Ledger};
//! ```
//!
//! What a frontend does instead: step a `Sim` and read it.
//!
//! ```
//! use rustyecon_engine::prelude::*;
//!
//! fn run_a_year(sim: &mut Sim) -> Result<Vec<u64>, RunError> {
//!     let mut hashes = Vec::new();
//!     let until = sim.tick() + u64::from(sim.world().clock.ticks_per_year);
//!     sim.run_until(until, &mut |r: &TickReport| hashes.push(r.hash))?;
//!     Ok(hashes)
//! }
//! ```

mod error;
pub mod prelude;
pub mod registry;
mod replay;
mod report;
mod sim;
mod tick;

// Core is not re-exported: its writer (`apply`, `resolve`, the ledger) must stay out of a
// frontend's reach (E1). The prelude re-exports its read-only types.
pub use rustyecon_agents;
pub use rustyecon_markets;

pub use error::{ReplayError, ResumeError, RunError, RunErrorKind};
pub use registry::{registry, Entry, RegistryLine, Use};
pub use replay::audit_replay;
pub use report::{FiredEvent, HoldingTotals, MarketLine, TickReport, Trace, TraceEntry};
pub use sim::{Sim, Status};

/// A tape of this engine's agents.
pub type Tape = rustyecon_core::Tape<rustyecon_agents::Agents>;
/// A resolved world of this engine's agents.
pub type World = rustyecon_core::World<rustyecon_agents::Agents>;
/// A checkpoint of this engine's agents.
pub type Checkpoint = rustyecon_core::Checkpoint<rustyecon_agents::Agents>;
