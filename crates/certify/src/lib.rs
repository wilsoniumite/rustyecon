//! Certification (PLAN §3.8; docs/CERTIFY.md, Phase 0 session 2): a run is scored against dated
//! criteria registered before it, and ends in a certificate that is verdict-first, fail-closed
//! and persisted, beside a manifest that records the run's inputs: the build, the tape by its
//! hash, the world and any checkpoint it resumed from. The certificate records the criteria by
//! file, date and hash (R9, N4, N10, N12, N15, A12).
//!
//! - [`criteria`]: every bar with its unit and basis, in a dated file per tape; nothing in code
//!   (R4). A retune is a new file.
//! - [`manifest`]: the tape hash, the build, the run key, and the manifest a resume verifies
//!   against (D4, R16). No Parquet and no I/O, so the GUI's web build uses it too.
//! - [`obs`]: what the batteries read, one `Obs` per tick, and the segments that restart the
//!   windows at each dated shock (N15).
//! - [`fold`]: maxima, minima, sums and log ranges that keep a NaN (N12).
//! - [`battery`]: Conservation (R2), Determinism (R8), Runaway (A12), Trades, Balance
//!   (BalanceWatch), Settles and Kick (REPORT §6), each pure over its inputs, and the reports
//!   (troughs, dead ticks, fills, rationing, consumption, spoilage, transfers).
//! - [`kick`]: the deferred and kicked tapes of the kick check.
//! - [`certificate`]: the verdict, the seal and the finite scan over every number rendered.
//! - [`run`]: [`certify`], one run from genesis, scored and sealed.
//! - `telemetry` (feature `parquet`, off by default): the run as tidy long Parquet, with
//!   `TickReport` as its input. It is the one module that names `parquet` or `std::io`, so the
//!   rest compiles without it, for the GUI's web build.
//!
//! The ledger and the state hash are not here but in core, since every tick asserts them.
//! Certify depends on core directly for its read-only helpers (`fnv1a_64`, `Fnv`, `num`,
//! `Basis`, `Date`) and calls none of core's writer; it re-exports nothing of core, and no
//! public function hands out a `Checkpoint`, `Sim` or `SimState` it built (checked by the
//! engine's source scans). It does no I/O: paths are the cli's. The oracle's convergence
//! battery and gap join in Phase 2 with `crates/observe`, and the scorecard against the record
//! in Phases 6 and 7.
//!
//! Salvaged from `july-v2-phase-3` where docs/CERTIFY.md says: BalanceWatch and its two tests
//! (`v2p3: certify/invariants.rs`), and the design of a verdict-first certificate persisted at a
//! stable path (`certificate.rs`). Fixed as they moved: N4 (an unscored run certified PASS), N10
//! (the tape hash missed inputs), N12 (statistics dropped NaN one sample at a time, and the NaN
//! scan read state fields, not the numbers rendered), N15 (windows were absolute ticks), and
//! every threshold moved from code into criteria.

pub mod battery;
pub mod certificate;
pub mod criteria;
pub mod fold;
pub mod kick;
mod leaves;
pub mod manifest;
pub mod obs;
pub mod run;
#[cfg(feature = "parquet")]
pub mod telemetry;

pub use certificate::{
    nonfinite_paths, BatteryId, BatteryResult, Certificate, CertificateError, CriteriaRef, Limit,
    RawCertificate, Reading, Verdict, ILLUSTRATIVE, ILLUSTRATIVE_BASIS,
};
pub use criteria::{
    Bar, BarUnit, BatterySpec, Count, Criteria, CriteriaError, Fit, ReportSpec, TapeRef,
};
pub use manifest::{
    tape_hash, Build, CheckpointRecord, HashRecord, Hex, Manifest, ManifestError, ResumedFrom,
    RunKey, TelemetryRecord, VerifyError,
};
pub use obs::{kick_ticks, segment_before, segments, MarketObs, Names, Obs, RationObs, Segment};
pub use run::{certify, illustrative_basis, Certified, CertifyError, Scoring};
