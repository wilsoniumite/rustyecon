//! The engine: one `Sim` owns a run. It loads a tape, steps one tick at a time in PLAN §3's
//! phase order, runs to a given tick, checkpoints, resumes, and audits a replay, returning each
//! tick's `TickReport`. Frontends, the `rustyecon` binary among them, drive it and read it
//! through read-only accessors. None of them mutates its state: every intervention is a dated
//! tape event, or a tape edit followed by a rerun or a resume (invariant E1). The engine holds no
//! global state and does no I/O (E2), its types cross threads (E3), and a failed step poisons the
//! `Sim` (E5). The contract is `docs/ENGINE.md`; this crate fills in at step P0.6.
