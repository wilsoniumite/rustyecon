//! The Phase 2 probe's harness (PROBE-SPEC; docs/probe/RULES.md).
//!
//! - [`setup`]: the Appendix B tape, generated from the oracle's equilibrium, and the setups
//!   that vary it (tick length, dials, rule variants, displaced genesis, dated shocks).
//! - [`perturb`]: the named runs: PROBE-SPEC §4.7's battery and the judges' added families.
//! - [`harness`]: runs a setup on the engine, reads the observables every tick, scores them
//!   against the oracle's values computed outside the Sim, and classifies the run (§4.5).
//! - [`protocol`]: the protocol's numbers.
//!
//! The binaries: `appb-tape` writes `tapes/appb.ron`; `probe` runs named perturbations and
//! writes per-tick CSV. Nothing on the engine path depends on this crate, so no agent can reach
//! the oracle through it (R13).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod harness;
pub mod perturb;
pub mod protocol;
pub mod setup;
