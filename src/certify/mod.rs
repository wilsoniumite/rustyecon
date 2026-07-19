//! The certification stack (v2 Phase 1, spec docs/architecture/engine.md).
//!
//! A run that is not conserved, deterministic, and scored against pre-registered
//! criteria is an anecdote. These guarantees live in the engine, not in notebooks:
//!
//! - [`hash`] — a canonical state fingerprint, the basis of the golden-hash tests.
//! - conservation ledger, run certificate, and criteria land here as the phase
//!   progresses.

pub mod hash;

pub use hash::state_hash;
