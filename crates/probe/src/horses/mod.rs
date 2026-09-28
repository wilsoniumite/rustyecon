//! The stocks probe (P2.2a; HORSES-SPEC; docs/probe/HORSES-RULES.md): the probe's agents with
//! the horse held as a durable good, a stock that a maker builds, a capacity desk holds, wears,
//! feeds and hires out by the horse-day, whose known answers are oracle unit 1g's.
//!
//! - [`instance`]: the instances H1–H4, the families F1–F10, R1a (the stocks layer off), P7 and
//!   P8, as GOODS-CHAIN §5's rule A on a county, and the oracle's equilibrium of each, solved
//!   outside any `Sim`.
//! - [`setup`]: the dials (C2g, and C2 for R1a and P7), the setups that vary them, genesis from
//!   the oracle, and the tape.
//! - [`perturb`]: the run grammar of §7.7, the registered battery with Tier 3S, and the families
//!   of §7.10.
//! - [`harness`]: runs a setup, reads §7.1's observables every tick, scores them against the
//!   oracle, classifies the run, and keeps §7.11's transient and stock statistics.
//! - [`kick`]: the kick set at a run's end (§7.5), and the engine's slowest mode for L (§7.4).
//! - [`cli`]: the options the two binaries share.
//! - [`probes`]: §7.8's one-tick elasticity probe and the open-loop probe.
//!
//! The markets probe's harness (`crate::markets`) is unchanged but for its classifier, which
//! this harness shares; on R1a this harness nests it (`horses_r1a_nests_i0`).

pub mod cli;
pub mod harness;
pub mod instance;
pub mod kick;
pub mod perturb;
pub mod probes;
pub mod setup;
