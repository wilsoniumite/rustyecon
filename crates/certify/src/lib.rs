//! Certification and telemetry (PLAN §3.8): verdict-first, fail-closed certificates persisted
//! to `results/`, dated criteria files, the detectors and batteries that score a run, and tidy
//! long Parquet telemetry. The conservation ledger and the state hash are not here but in
//! `rustyecon-core`, because every tick asserts them. The July stack moves here in Phase 0's
//! second session (ADDENDUM A4); the oracle convergence battery and the oracle gap join in
//! Phase 2, and the scorecard against the record in Phases 6 and 7.
