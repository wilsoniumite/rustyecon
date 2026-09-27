//! The markets probe (P2.1; MARKETS-SPEC; docs/probe/MARKETS-RULES.md): the Phase 2 probe's
//! agents on economies with many final categories and many machine types, whose known answers
//! are oracle units 1b and 1c.
//!
//! - [`instance`]: the instances I0–I3, L2, L3 and G1 as the tape registers them, and the
//!   oracle's equilibrium of each, solved outside any `Sim`.
//! - [`setup`]: the dials (C2m, C2L), the setups that vary them, genesis from the oracle, and the
//!   tape of the four many-market kinds.
//! - [`perturb`]: the run grammar of §7.7, the registered batteries with their tiers and slack
//!   runs, and the families of §7.10.
//! - [`harness`]: runs a setup, reads §7.1's observables every tick, scores them against the
//!   oracle, classifies the run (PROBE-SPEC §4.5), and keeps §7.11's transient statistics.
//! - [`kick`]: the kick set at a run's end (§7.5), through certify.
//! - [`cli`]: the options the two binaries share.
//! - [`probes`]: §7.8's one-tick elasticity probe, which gives L, and the open-loop probe.
//!
//! P2.0's harness (`crate::harness`, `crate::setup`, `crate::perturb`) is unchanged and still
//! makes docs/probe/results/ byte for byte; on I0 this harness nests it (`markets_i0_nests_appb`).

pub mod cli;
pub mod harness;
pub mod instance;
pub mod kick;
pub mod perturb;
pub mod probes;
pub mod setup;
