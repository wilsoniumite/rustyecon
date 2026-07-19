//! Golden-hash determinism suite (v2 Phase 1, spec docs/architecture/engine.md).
//!
//! Determinism is a *tested* property (METHODOLOGY R4), not an intention. Three
//! equalities, run against a real scenario (`lr_00`) so they exercise the whole
//! tick loop including the now-active dividend desk:
//!
//! - **Repeat**: same (state, code) twice → identical per-tick hash sequence.
//! - **Resume**: checkpoint at T, reload, run K more → hashes equal the
//!   uninterrupted run (checkpoints are lossless, lot lives included).
//! - **Replay**: record the delta stream, apply it to a fresh genesis copy with
//!   no systems running → identical end hash. This also proves `apply_state_deltas`
//!   is the sole mutator: nothing the systems do escapes the delta stream.

use std::path::PathBuf;

use rustyecon::{
    certify::state_hash,
    output::checkpoint::{self, SaveFormat},
    scenario::{loader, EventSchedule},
    state::{apply_state_deltas, GameData, SimState},
    systems::{run_tick, run_tick_capture},
    types::delta::StateDelta,
};

const TICKS: u64 = 60;

fn load_lr00() -> (SimState, GameData, EventSchedule) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios/lr_00");
    let s = loader::load(&dir).expect("lr_00 scenario loads");
    (s.state, s.game_data, s.events)
}

/// Advance `state` by `ticks`, returning the state hash after each tick.
fn hash_seq(state: &mut SimState, gd: &GameData, events: &EventSchedule, ticks: u64) -> Vec<u64> {
    let mut hashes = Vec::with_capacity(ticks as usize);
    for _ in 0..ticks {
        run_tick(state, gd, events);
        hashes.push(state_hash(state));
    }
    hashes
}

#[test]
fn repeat_same_inputs_yield_identical_hash_sequence() {
    let (genesis, gd, events) = load_lr00();
    let a = hash_seq(&mut genesis.clone(), &gd, &events, TICKS);
    let b = hash_seq(&mut genesis.clone(), &gd, &events, TICKS);
    assert_eq!(a, b, "same (state, code, seed) must give identical per-tick hashes");
    // Guard against the degenerate pass where nothing changes tick-to-tick.
    assert!(a.windows(2).any(|w| w[0] != w[1]), "the run must actually evolve");
}

#[test]
fn resume_from_checkpoint_matches_uninterrupted_run() {
    const T: u64 = 25;
    const K: u64 = 25;
    let (genesis, gd, events) = load_lr00();

    // Uninterrupted reference: hash after each of ticks 1..=T+K.
    let reference = hash_seq(&mut genesis.clone(), &gd, &events, T + K);

    // Run to T, checkpoint (bincode), reload, continue K more.
    let mut state = genesis;
    let _ = hash_seq(&mut state, &gd, &events, T);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("resume.bin");
    checkpoint::save(&state, &SaveFormat::Binary, &path).unwrap();
    let mut resumed = checkpoint::load(&path).unwrap();
    assert_eq!(
        state_hash(&state),
        state_hash(&resumed),
        "checkpoint round-trip must be lossless (incl. lot lives, supply/demand, EMAs)"
    );

    let tail = hash_seq(&mut resumed, &gd, &events, K);
    assert_eq!(
        &reference[T as usize..],
        tail.as_slice(),
        "resumed run must track the uninterrupted run tick-for-tick"
    );
}

#[test]
fn replay_delta_stream_reproduces_end_state() {
    let (genesis, gd, events) = load_lr00();

    // Live run: capture the full ordered delta stream across all ticks.
    let mut live = genesis.clone();
    let mut stream: Vec<StateDelta> = Vec::new();
    for _ in 0..TICKS {
        stream.extend(run_tick_capture(&mut live, &gd, &events));
    }
    let live_hash = state_hash(&live);

    // Replay: apply the recorded stream to a fresh genesis with NO systems.
    let mut replayed = genesis;
    apply_state_deltas(&mut replayed, &stream);
    assert_eq!(
        state_hash(&replayed),
        live_hash,
        "replaying the delta stream onto genesis must reproduce the live end state"
    );
}
