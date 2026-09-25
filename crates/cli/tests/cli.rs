//! The product path (docs/ENGINE.md §8 and §11, cli): the `rustyecon` binary on the gate world,
//! its checkpoints and hashes files, and its exit codes. Hashes are compared exactly.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const GATE: &str = include_str!("../../../tapes/gate.ron");
const TICKS: u64 = 2080;

fn rustyecon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rustyecon"))
        .args(args)
        .output()
        .expect("the binary runs")
}

fn code(out: &Output) -> i32 {
    out.status.code().expect("an exit code")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Write a tape text into `dir` and return its path.
fn write_tape(dir: &Path, name: &str, text: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, text).expect("the tape is written");
    p
}

/// Replace exactly one occurrence of `from` in the gate tape.
fn edit(from: &str, to: &str) -> String {
    assert_eq!(GATE.matches(from).count(), 1, "{from:?} must occur once");
    GATE.replacen(from, to, 1)
}

fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("the hashes file")
        .lines()
        .map(str::to_string)
        .collect()
}

fn s(p: &Path) -> &str {
    p.to_str().expect("a UTF-8 path")
}

#[test]
fn gate_resume_through_product_path() {
    // N11: run the gate world with checkpoints in both formats, then resume from ticks 1, 520,
    // 1,040 and 2,079; each resumed run's --hashes lines equal the uninterrupted run's tail.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let full = dir.path().join("full.txt");
    let out = rustyecon(&["run", s(&tape), "--until", "2080", "--hashes", s(&full)]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let reference = lines(&full);
    assert_eq!(reference.len(), TICKS as usize);
    assert!(reference[0].starts_with("1 0x") && reference[2079].starts_with("2080 0x"));
    assert_eq!(stdout(&out).lines().last(), Some(reference[2079].as_str()));
    for (format, ext) in [("bin", "bin"), ("ron", "ron")] {
        let cps = dir.path().join(format!("cp-{format}"));
        // Every 520th state, and the final state of runs that stop at 1 and 2,079.
        for (until, every) in [("2080", Some("520")), ("1", None), ("2079", None)] {
            let mut args = vec![
                "run",
                s(&tape),
                "--until",
                until,
                "--out",
                s(&cps),
                "--format",
                format,
            ];
            if let Some(n) = every {
                args.extend(["--checkpoint-every", n]);
            }
            let out = rustyecon(&args);
            assert_eq!(code(&out), 0, "{}", stderr(&out));
        }
        for name in [1, 520, 1040, 1560, 2079, 2080] {
            assert!(
                cps.join(format!("tick_{name:08}.{ext}")).is_file(),
                "{name}"
            );
        }
        for at in [1usize, 520, 1040, 2079] {
            let cp = cps.join(format!("tick_{at:08}.{ext}"));
            let tail = dir.path().join(format!("tail-{format}-{at}.txt"));
            let out = rustyecon(&[
                "resume",
                s(&cp),
                "--tape",
                s(&tape),
                "--until",
                "2080",
                "--hashes",
                s(&tail),
            ]);
            assert_eq!(code(&out), 0, "{}", stderr(&out));
            assert_eq!(
                lines(&tail),
                reference[at..],
                "resumed at {at} from {format}"
            );
            assert_eq!(stdout(&out).lines().last(), Some(reference[2079].as_str()));
        }
    }
}

#[test]
fn replay_command_passes_on_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let run = rustyecon(&["run", s(&tape), "--until", "2080"]);
    let out = rustyecon(&["replay", s(&tape), "--until", "2080"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().last(), stdout(&run).lines().last());
}

#[test]
fn shortfall_stops_the_run() {
    // N3: a burn of more than is held stops the run with exit 2, the ledger line and the last
    // good tick on stderr; the hashes file ends at the last good tick.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(
        dir.path(),
        "theft.ron",
        &edit(
            "    events: [\n",
            "    events: [\n        (key: \"theft\", at: \"1751-06-01\", basis: Assumed(\"test\"), \
             act: Burn(holder: \"workers\", good: \"coin\", amount: Qty(1e9))),\n",
        ),
    );
    let hashes = dir.path().join("h.txt");
    let out = rustyecon(&["run", s(&tape), "--until", "2080", "--hashes", s(&hashes)]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    let err = stderr(&out);
    // 1751-06-01 falls in tick 73.
    assert!(err.contains("tick 73, phase 0 (events)"), "{err}");
    assert!(
        err.contains("shortfall at tick 73") && err.contains("a Event burn"),
        "{err}"
    );
    assert!(err.contains("asked pop#1 for 1e9 of good#1"), "{err}");
    assert!(err.contains("the last good tick is 72"), "{err}");
    let h = lines(&hashes);
    assert_eq!(h.len(), 73);
    assert!(h[72].starts_with("73 0x"));
}

#[test]
fn undefined_good_fails_to_load() {
    // N2: an undefined good is a load error in every profile, exit 1, naming its tape path.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(
        dir.path(),
        "salt.ron",
        &edit(
            r#"(node: "village", good: "grain", qty: "mill.buy.grain.village", weight: 0.375)"#,
            r#"(node: "village", good: "salt", qty: "mill.buy.grain.village", weight: 0.375)"#,
        ),
    );
    for cmd in ["run", "replay"] {
        let out = rustyecon(&[cmd, s(&tape), "--until", "10"]);
        assert_eq!(code(&out), 1, "{}", stderr(&out));
        assert!(
            stderr(&out).contains("actors[mill].spec.buy[village/salt].good"),
            "{}",
            stderr(&out)
        );
    }
    let out = rustyecon(&["registry", s(&tape)]);
    assert_eq!(code(&out), 1);
    // A missing tape file is a load error too; a bad argument is 1, not clap's 2.
    let out = rustyecon(&["run", s(&dir.path().join("none.ron")), "--until", "10"]);
    assert_eq!(code(&out), 1);
    let out = rustyecon(&["run", s(&tape), "--until", "soon"]);
    assert_eq!(code(&out), 1);
    let out = rustyecon(&["run", s(&tape), "--until", "10", "--checkpoint-every", "5"]);
    assert_eq!(code(&out), 1, "--checkpoint-every needs --out");
}

#[test]
fn resume_refuses_another_tape() {
    // N11: a checkpoint of the gate world is refused under a tape whose world differs (exit 3),
    // and under one whose past differs.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let cps = dir.path().join("cps");
    let out = rustyecon(&["run", s(&tape), "--until", "600", "--out", s(&cps)]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let cp = cps.join("tick_00000600.bin");
    let other_world = write_tape(
        dir.path(),
        "richer.ron",
        &edit(
            r#"(holder: "workers", goods: [("coin", 60.0)])"#,
            r#"(holder: "workers", goods: [("coin", 61.0)])"#,
        ),
    );
    let other_past = write_tape(
        dir.path(),
        "early.ron",
        &edit(
            r#"(key: "mine.cut", at: "1760-03-01""#,
            r#"(key: "mine.cut", at: "1755-03-01""#,
        ),
    );
    for (t, why) in [
        (&other_world, "belongs to world"),
        (&other_past, "before tick 600"),
    ] {
        let out = rustyecon(&["resume", s(&cp), "--tape", s(t), "--until", "700"]);
        assert_eq!(code(&out), 3, "{}", stderr(&out));
        assert!(stderr(&out).contains(why), "{}", stderr(&out));
    }
    // A corrupt checkpoint is a checkpoint error too.
    let bad = dir.path().join("bad.bin");
    fs::write(&bad, b"RUSTYECK\x01\x00\x00\x00garbage").unwrap();
    let out = rustyecon(&["resume", s(&bad), "--tape", s(&tape), "--until", "700"]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    // The right tape resumes.
    let out = rustyecon(&["resume", s(&cp), "--tape", s(&tape), "--until", "700"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    // --until before the checkpoint's tick is an argument error.
    let out = rustyecon(&["resume", s(&cp), "--tape", s(&tape), "--until", "500"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
}

#[test]
fn unknown_extension_errors() {
    // A checkpoint is .bin or .ron; anything else is refused before any file is read.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    for name in ["cp.chk", "cp", "cp.BIN"] {
        let out = rustyecon(&[
            "resume",
            s(&dir.path().join(name)),
            "--tape",
            s(&tape),
            "--until",
            "10",
        ]);
        assert_eq!(code(&out), 1, "{name}: {}", stderr(&out));
        assert!(stderr(&out).contains(".bin or .ron"), "{}", stderr(&out));
    }
}

#[test]
fn failed_checkpoint_save_stops_the_run() {
    // July printed a failed save and ran on (`v2p3: runner.rs:488-505`). Here --out naming a
    // regular file stops the run at the first checkpoint with exit 3, and no further tick runs:
    // the hashes file ends at the failed checkpoint's tick.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let file = dir.path().join("not-a-dir");
    fs::write(&file, "a regular file").unwrap();
    let hashes = dir.path().join("h.txt");
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "100",
        "--out",
        s(&file),
        "--checkpoint-every",
        "10",
        "--hashes",
        s(&hashes),
    ]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    let h = lines(&hashes);
    assert_eq!(h.len(), 10);
    assert!(h[9].starts_with("10 0x"), "{h:?}");
    assert!(stdout(&out).is_empty(), "no final hash after a failed run");
}

#[test]
fn registry_lists_every_number() {
    // R4: every param with its unit, per-tick value and basis, and the inline numbers.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let out = rustyecon(&["registry", s(&tape)]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    let tape = rustyecon_engine::Tape::from_ron(GATE).expect("the gate tape parses");
    let listed = text.lines().filter(|l| l.starts_with("params[")).count();
    assert_eq!(listed, tape.params.len());
    for p in &tape.params {
        let prefix = format!("params[{}]\t", p.key);
        assert!(text.lines().any(|l| l.starts_with(&prefix)), "{prefix}");
    }
    for expected in [
        "params[mine.capacity]\t5.2e1\tFlowPerYear live\tflow 1e0 per tick\tAssumed(gate world)",
        "params[life.bread]\t5.77e-2\tYears fixed\tticks 3e0 per tick\tAssumed(three weeks)",
        "params[rate.grain]\t5.2e0\tRatePerYear live\tlog_step 1e-1 per tick",
        "actors[mill].spec.recipe.inputs[grain]\t2e0\tDimensionless inline",
        "genesis.prices[town/bread].price\t2e0\tDimensionless inline",
        "recurring[pension].act.qty\t5e0\tDimensionless inline",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in\n{text}");
    }
}

#[test]
fn no_behaviour_switches() {
    // E1, N13: behaviour comes from the tape. July's --agents, --price-rule and --supply-rule
    // do not return, and no override of any kind exists; each is an argument error.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    for (flag, value) in [
        ("--price-rule", "ratio"),
        ("--agents", "kernel"),
        ("--supply-rule", "inelastic"),
        ("--set", "mine.capacity=1"),
    ] {
        let out = rustyecon(&["run", s(&tape), "--until", "10", flag, value]);
        assert_eq!(code(&out), 1, "{flag}: {}", stderr(&out));
    }
}
