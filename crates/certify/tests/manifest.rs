//! The manifest (docs/CERTIFY.md §9; N10, D4, R16): it records a run's hashes by digest and
//! each checkpoint it wrote, only a checkpoint of the run itself, at relative paths; and a
//! resume verifies against it.

mod common;

use certify::{tape_hash, Hex, Manifest, ManifestError, RunKey, VerifyError};
use common::*;
use rustyecon_core::fnv1a_64;
use rustyecon_engine::prelude::{Checkpoint, Sim, Tape};

fn key(t: &Tape, sim: &Sim) -> RunKey {
    RunKey {
        build: build(),
        tape_hash: Hex(tape_hash(t)),
        world_id: Hex(sim.world().world_id),
    }
}

/// A manifest of `t` run from genesis to `until`, and the `Sim` there.
fn ran(t: &Tape, until: u64) -> (Manifest, Sim) {
    let mut s = sim(t);
    let mut m = Manifest::begin(key(t, &s), s.hash(), &s, None);
    s.run_until(until, &mut |r| m.tick(r.tick + 1, r.hash))
        .unwrap();
    (m, s)
}

/// The gate with a mint dated at tick 50: the same world, another state from tick 50 on.
fn edited_gate() -> Tape {
    let gate = tape(GATE);
    let mut t = gate.clone();
    t.events.push(
        ron::from_str(&format!(
            "(key: \"windfall\", at: \"{}\", basis: Assumed(\"test\"), act: Mint(holder: \
             \"workers\", good: \"coin\", qty: 7.0))",
            date_of(&gate, 50)
        ))
        .unwrap(),
    );
    t
}

fn checkpoint_at(t: &Tape, tick: u64) -> Checkpoint {
    let mut s = sim(t);
    s.run_until(tick, &mut |_| {}).unwrap();
    s.checkpoint().unwrap()
}

#[test]
fn manifest_refuses_a_checkpoint_off_the_run() {
    // §9: a record is the run's own hash for its tick, never a checkpoint compared with itself.
    // A checkpoint of the run at the tick it reached records; one whose state is not what the
    // run hashed there (another run of the same world), one at another tick, and one of another
    // world are OffRun.
    let gate = tape(GATE);
    let (mut m, s) = ran(&gate, 100);
    let own = s.checkpoint().unwrap();
    m.checkpoint(&own, "tick_00000100.bin")
        .expect("the run's own checkpoint");
    let rec = &m.checkpoints[0];
    assert_eq!(
        (rec.tick, rec.digest, rec.state_hash),
        (100, Hex(own.digest()), Hex(s.hash()))
    );
    let edited = edited_gate();
    assert_eq!(sim(&edited).world().world_id, s.world().world_id);
    let other_run = checkpoint_at(&edited, 100);
    let e = m.checkpoint(&other_run, "x.bin").unwrap_err();
    assert!(
        matches!(&e, ManifestError::OffRun(w) if w.contains("hashes to")),
        "{e}"
    );
    let earlier = checkpoint_at(&gate, 60);
    let e = m.checkpoint(&earlier, "x.bin").unwrap_err();
    assert!(
        matches!(&e, ManifestError::OffRun(w) if w.contains("its tick is 60")),
        "{e}"
    );
    let appb = checkpoint_at(&tape(APPB), 100);
    let e = m.checkpoint(&appb, "x.bin").unwrap_err();
    assert!(
        matches!(&e, ManifestError::OffRun(w) if w.contains("world_id")),
        "{e}"
    );
    assert_eq!(m.checkpoints.len(), 1);
    // Before any tick, the starting state: genesis.
    let fresh = sim(&gate);
    let mut m0 = Manifest::begin(key(&gate, &fresh), fresh.hash(), &fresh, None);
    m0.checkpoint(&fresh.checkpoint().unwrap(), "tick_00000000.bin")
        .expect("the starting state");
    let mut m0 = Manifest::begin(key(&gate, &fresh), fresh.hash() ^ 1, &fresh, None);
    assert!(m0
        .checkpoint(&fresh.checkpoint().unwrap(), "t.bin")
        .is_err());
}

#[test]
fn manifest_paths_are_relative() {
    // §9: files are recorded relative to the manifest's directory, with `/`, so a manifest made
    // on Windows and one made in WSL agree; an absolute path, a drive or a `..` is refused.
    let gate = tape(GATE);
    let (m, s) = ran(&gate, 10);
    let cp = s.checkpoint().unwrap();
    for (given, recorded) in [
        ("tick_00000010.bin", "tick_00000010.bin"),
        ("sub\\tick_00000010.ron", "sub/tick_00000010.ron"),
        ("sub/tick_00000010.ron", "sub/tick_00000010.ron"),
    ] {
        let mut m = m.clone();
        m.checkpoint(&cp, given).unwrap();
        assert_eq!(m.checkpoints[0].file, recorded);
        m.telemetry(given, 3, 4).unwrap();
        assert_eq!(m.telemetry.as_ref().unwrap().file, recorded);
    }
    for bad in [
        "/abs/tick.bin",
        "C:/abs/tick.bin",
        "C:\\abs\\tick.bin",
        "../up.bin",
        "sub/../../up.bin",
        "",
        "sub//tick.bin",
    ] {
        let mut m = m.clone();
        assert!(
            matches!(m.checkpoint(&cp, bad), Err(ManifestError::Path(_))),
            "{bad:?}"
        );
        assert!(m.telemetry(bad, 1, 1).is_err(), "{bad:?}");
    }
}

#[test]
fn manifest_records_the_hash_stream_and_reads_back() {
    // §9, §10: the digest is FNV-1a 64 over the hash file's body, so `grep -v '^#' | fnv`
    // checks it; the header names the build, the tape, the world and the start; and the RON
    // reads back equal. A manifest of another format is refused as such.
    let gate = tape(GATE);
    let (m, _) = ran(&gate, 50);
    let mut s = sim(&gate);
    let mut body = String::new();
    s.run_until(50, &mut |r| {
        body.push_str(&Manifest::line(r.tick + 1, r.hash))
    })
    .unwrap();
    assert_eq!(m.hashes.digest, Hex(fnv1a_64(body.as_bytes())));
    assert_eq!(m.hashes.count, 50);
    assert_eq!(m.hashes.last, Some((50, Hex(s.hash()))));
    assert_eq!((m.from, m.until), (0, 50));
    let header = m.hashes_header();
    assert_eq!(
        header,
        format!(
            "# rustyecon hashes 1\n# build 0123456789abcdef0123456789abcdef01234567 clean \
             test-target\n# tape gate tape_hash {} world_id {}\n# from 0\n",
            m.run.tape_hash, m.run.world_id
        )
    );
    let text = m.to_ron();
    assert!(text.starts_with("Manifest(\n    format: 1,"));
    assert_eq!(Manifest::from_ron(&text), Ok(m.clone()));
    assert_eq!(
        Manifest::from_ron(&text.replacen("format: 1", "format: 2", 1)),
        Err(ManifestError::Format { found: 2 })
    );
    assert!(Manifest::from_ron(&text.replacen("telemetry: None", "", 1)).is_err());
}

#[test]
fn a_resume_verifies_against_its_manifest() {
    // D4, R16: a resume needs a manifest of the same tape (`tape_hash`), a checkpoint of its
    // world, and a record at the checkpoint's tick with the same digest and state hash. It
    // catches a checkpoint of another tape, world, run or tick, and a file replaced since.
    let gate = tape(GATE);
    let (mut m, s) = ran(&gate, 100);
    let cp = s.checkpoint().unwrap();
    m.checkpoint(&cp, "tick_00000100.bin").unwrap();
    let th = tape_hash(&gate);
    let r = m.verify(&cp, th).expect("its own checkpoint verifies");
    assert_eq!(
        (r.tick, r.digest, r.state_hash, &r.parent),
        (100, Hex(cp.digest()), Hex(s.hash()), &m.run)
    );
    let edited = edited_gate();
    assert_eq!(
        m.verify(&cp, tape_hash(&edited)),
        Err(VerifyError::OtherTape {
            manifest: Hex(th),
            tape: Hex(tape_hash(&edited))
        })
    );
    assert!(matches!(
        m.verify(&checkpoint_at(&tape(APPB), 100), th),
        Err(VerifyError::OtherWorld { .. })
    ));
    assert_eq!(
        m.verify(&checkpoint_at(&gate, 60), th),
        Err(VerifyError::NotRecorded { tick: 60 })
    );
    assert_eq!(
        m.verify(&checkpoint_at(&edited, 100), th),
        Err(VerifyError::Differs {
            tick: 100,
            what: "digest"
        })
    );
    // The resumed run records its parent and names it in its hash header, and a checkpoint of
    // its starting state records against the parent's state hash.
    let resumed = Sim::resume(&gate, &cp).unwrap();
    let mut child = Manifest::begin(key(&gate, &resumed), m.genesis_hash.0, &resumed, Some(r));
    assert_eq!(child.from, 100);
    assert!(child
        .hashes_header()
        .ends_with(&format!("# from 100 resumed {}\n", Hex(cp.digest()))));
    child
        .checkpoint(&cp, "again.bin")
        .expect("the resumed start");
}
