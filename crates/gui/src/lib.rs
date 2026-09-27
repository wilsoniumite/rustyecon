//! rustyecon's interactive frontend (docs/GUI.md): it runs the engine live, records what each
//! tick reports, and draws it. It observes; the tape decides (PLAN R16).
//!
//! One seam runs through it, and it stays fixed as the GUI grows (GUI.md §3):
//!
//! ```text
//! Intent → reduce → Effect::Send(Cmd) → Driver → Runner (owns the Sim) → Obs → Store → view-model
//! ```
//!
//! - [`model`]: the model and [`model::reduce`], its only writer. An [`model::Intent`] goes in
//!   and [`model::Effect`]s come out: commands for a run, file reads and saves. No egui, no I/O.
//! - [`run`]: the run's side. [`run::Runner`] is the one struct that owns a `Sim` (U1). It
//!   takes [`run::Cmd`]s, steps, extracts each tick's rows ([`run::Extractor`]), keeps the ring
//!   of checkpoints, and hands out [`run::Obs`]. [`run::Store`] records them in memory. No egui,
//!   no clock, no thread, no I/O: it moves into `crates/observe` at G2 (D13).
//! - [`vm`]: one pure view-model builder per panel, reading run types only. No egui.
//! - [`edit`]: the editor's model: tape edits, `materialise`, the lineage, `plan` and export,
//!   as tapes and text. No egui, no file, no thread, no clock.
//! - [`drive`]: [`drive::ThreadDriver`], one worker thread per run with its clock and channels,
//!   and [`drive::Host`], which carries out the model's effects.
//! - [`platform`]: files: tapes, lineages, exports, `session.ron` and `layout.ron`.
//! - [`ui`] and [`app`]: egui. Panels draw view-models and return intents.
//!
//! The scans in `tests/scans.rs` hold the seams: no egui in model, run, edit, vm, drive or
//! platform; no trigonometry outside `ui/` (D12); no raw transcendental or hashed container in
//! the egui-free modules (U9).

pub mod app;
pub mod drive;
pub mod edit;
pub mod model;
pub mod platform;
pub mod run;
pub mod ui;
pub mod vm;

use certify::Build;

/// The build this binary was made from, stamped by the cli's `build.rs`, which this crate
/// shares (docs/CERTIFY.md §3): the commit, the dirty flag, the target and the compiler.
/// Anything but an explicit clean stamp reads as dirty (fail closed), as in the cli.
pub fn build() -> Build {
    Build {
        commit: env!("RUSTYECON_COMMIT").to_string(),
        dirty: env!("RUSTYECON_DIRTY") != "false",
        target: env!("RUSTYECON_TARGET").to_string(),
        rustc: env!("RUSTYECON_RUSTC").to_string(),
    }
}
