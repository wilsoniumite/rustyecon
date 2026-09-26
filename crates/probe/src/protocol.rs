//! The protocol's numbers (PROBE-SPEC §4), named once. They belong to the harness, which reads
//! runs, and never to a run: no agent sees them.

/// `probe.tol_floor`: the smallest gap, in log, the probe reads as economically meaningful.
/// tol_o = max(R_o, this); every R_o is 0 for the band-free rules registered here (§4.3).
pub const TOL_FLOOR: f64 = 1e-3;

/// `probe.hold_tol`: mode A's bar on every gap, fill shortfall and spoilage share (§4.6).
pub const HOLD_TOL: f64 = 1e-9;

/// `probe.live_floor`: a market that clears less than this share of its oracle volume makes a
/// dead tick (§4.5).
pub const LIVE_FLOOR: f64 = 0.5;

/// DEAD's share (§4.5): more dead ticks than this share of W, or any in F, is DEAD. The rule is
/// certify's `battery::dead_share_ok`, and certify reads its share from criteria (C11).
pub const DEAD_SHARE: f64 = 0.01;

/// `probe.runaway`: a posted price outside [1/this, this] times its genesis value is DIVERGED
/// (§4.5).
pub const RUNAWAY: f64 = 1e6;

/// L, the scored run length (§4.4), and mode A's registered length: 200·τ_max for the
/// registered dials is at most 20,000 (docs/probe/RULES.md §3).
pub const RUN_TICKS: u64 = 20_000;
