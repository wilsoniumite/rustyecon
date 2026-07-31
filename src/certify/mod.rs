//! The certification stack (v2 Phase 1, spec docs/architecture/engine.md).
//!
//! A run that is not conserved, deterministic, and scored against pre-registered
//! criteria is an anecdote. These guarantees live in the engine, not in notebooks:
//!
//! - [`hash`] — a canonical state fingerprint, the basis of the golden-hash tests.
//! - [`ledger`] — per-tick conservation accounting; a breach panics the run.
//! - run certificate and criteria land here as the phase progresses.

pub mod certificate;
pub mod criteria;
pub mod invariants;
pub mod hash;
pub mod ledger;
pub mod metrics;
pub mod nan;
pub mod verdict;

pub use certificate::{Battery, Certificate, RunIdentity};
pub use criteria::{Criteria, FailureClass, Metric, NanPolicy, Rule, Window};
pub use hash::state_hash;
pub use ledger::{ConservationLedger, Shortfall, TickAudit};
pub use metrics::{MetricsCollector, RegionSeries};
pub use verdict::{RegionVerdict, StabilityReport};
