//! The replay audit (docs/ENGINE.md §7.6): test_03's design, and July's shadow replay
//! (`v2p3: runner.rs:240-255`), every tick.
//!
//! A live `Sim` runs with `step_traced`. Beside it a shadow `SimState` from the same genesis
//! applies each tick's trace through `core::apply` alone, with no behaviour, clearing or
//! settlement code, under a ledger of its own. The two hashes are compared after every tick. A
//! match proves that `apply` is the only writer: nothing the hooks or the markets do escapes the
//! delta stream. July compared every 64th tick; this compares every one.

use crate::error::ReplayError;
use crate::sim::Sim;
use crate::Tape;
use rustyecon_core::{apply, resolve, state_hash, Ledger, Phase};

/// Run `tape` until the state's tick is `until` beside a shadow that only applies the trace,
/// comparing hashes every tick. Returns the final hash.
pub fn audit_replay(tape: &Tape, until: u64) -> Result<u64, ReplayError> {
    let (world, mut shadow) = resolve(tape).map_err(ReplayError::Load)?;
    let mut live = Sim::new(tape).map_err(ReplayError::Load)?;
    while live.tick() < until {
        let tick = live.tick();
        let (report, trace) = live.step_traced().map_err(ReplayError::Run)?;
        let err = |phase: Phase| move |error| ReplayError::Shadow { tick, phase, error };
        let mut l = Ledger::open(&shadow, &world).map_err(err(Phase::Events))?;
        // Apply each run of same-phase entries as one batch, in trace order.
        let entries = &trace.0;
        let mut i = 0;
        while i < entries.len() {
            let phase = entries[i].phase;
            let n = entries[i..].iter().take_while(|e| e.phase == phase).count();
            let batch: Vec<_> = entries[i..i + n].iter().map(|e| e.delta.clone()).collect();
            apply(&mut shadow, &world, phase, &batch, &mut l).map_err(err(phase))?;
            i += n;
        }
        l.close(&shadow, &world).map_err(err(Phase::Measure))?;
        let shadow_hash = state_hash(&shadow);
        if shadow_hash != report.hash {
            return Err(ReplayError::Mismatch {
                tick,
                live: report.hash,
                shadow: shadow_hash,
            });
        }
    }
    Ok(live.hash())
}
