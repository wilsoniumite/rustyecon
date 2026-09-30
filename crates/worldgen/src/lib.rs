//! The tape compiler (PLAN §3.7). Human-editable tables of regions, parcels, deposits,
//! population, channels and dated timelines (technology, enclosure, laws and taxes, monetary
//! regimes, wars, world prices) compile into one validated tape with a signed genesis; the
//! compiler generates the ids and checks references and units. It fills in during Phase 4,
//! starting with England by historic county.
//!
//! [`atlas`] loads the county atlas (docs/GUI.md §6; data/atlas/README.md): the regions, their
//! shared borders and their measures. It lives here, with no egui, so that the GUI's map and the
//! compiler read one definition of the regions.
//!
//! [`compile()`] is the first compiler (D.2, 2026-09-27), for the illustrative demo world
//! `worlds/demo-gb` (docs/demo/WORLD.md): its tables ([`Tables`]) are read and checked against
//! the atlas ([`tables`]), each county's history is composed and stepped ([`history`]), every
//! county is solved by the oracle at genesis and after every step, and the tape is written.
//! The compiler reads no file: the cli reads the tables and writes the tape. The oracle is
//! solved here, outside any `Sim`, to seed genesis and check the tables; no agent reads it
//! (R13).
//!
//! [`lens`] (D.3, 2026-09-27) defines the demo world's lenses, the measures its map colours by,
//! over a county's recorded numbers, beside `lenses.csv` and the keys the compiler writes. The
//! GUI calls it and defines none (U6), until `crates/observe` takes them (docs/GUI.md §7.3).
//!
//! [`stage`] (D2.2, 2026-09-30) compiles the demo's second pass (docs/demo/WORLD-V2.md): the same
//! county tables under GOODS-CHAIN's stage v2a.1, rule A's horse and fodder on every county, with
//! [`chain`] the one definition of rule A's recipe and unit 1g's chain per county. Without a
//! stage the compiler is v1's and writes v1's tape bit for bit (R1).

pub mod atlas;
pub mod chain;
pub mod compile;
mod csv;
pub mod history;
pub mod lens;
pub mod stage;
pub mod tables;

pub use compile::{compile, Compiled, Plan, Summary};
pub use stage::{compile_stage, StageTables};
pub use tables::Tables;

use std::fmt;

/// Why a world did not compile: where, and what is wrong there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    /// A file and line, a table row's key and column, or a county and date.
    pub at: String,
    /// What is wrong.
    pub why: String,
}

impl CompileError {
    /// An error at `at`.
    pub fn new(at: impl Into<String>, why: impl Into<String>) -> CompileError {
        CompileError {
            at: at.into(),
            why: why.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "worldgen: {}: {}", self.at, self.why)
    }
}

impl std::error::Error for CompileError {}
