//! The product path (docs/ENGINE.md §8 and §11, cli): the `rustyecon` binary on the gate world,
//! its checkpoints and hashes files, and its exit codes. Hashes are compared exactly.

use rustyecon_engine::prelude::{Clock, ClockMethod, Date};
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
fn conservation_breach_stops_the_run() {
    // R2 and N3: a conservation breach, not only a shortfall, stops the run with exit 2, the
    // breach's ledger line and the last good tick on stderr; the hashes file ends at the last
    // good tick. Tolerances of 0 load (they are below 1), and the walk's rounding then breaches.
    let dir = tempfile::tempdir().unwrap();
    let text = edit(
        r#"(key: "ledger.rel_flow", value: 1e-12,"#,
        r#"(key: "ledger.rel_flow", value: 0.0,"#,
    );
    let from = r#"(key: "ledger.rel_stock", value: 1e-11,"#;
    assert_eq!(text.matches(from).count(), 1);
    let text = text.replacen(from, r#"(key: "ledger.rel_stock", value: 0.0,"#, 1);
    let tape = write_tape(dir.path(), "strict.ron", &text);
    let hashes = dir.path().join("h.txt");
    let out = rustyecon(&["run", s(&tape), "--until", "2080", "--hashes", s(&hashes)]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    let err = stderr(&out);
    let at: u64 = err
        .strip_prefix("run error: tick ")
        .and_then(|rest| rest.split(',').next())
        .and_then(|t| t.parse().ok())
        .unwrap_or_else(|| panic!("no failing tick in {err}"));
    assert!(
        err.contains(&format!(
            "tick {at}, phase 7 (measure): conservation failure at tick {at}: "
        )),
        "{err}"
    );
    assert!(err.contains("(tolerance 0e0)"), "{err}");
    let last = if at == 0 {
        "no tick completed; the run started at tick 0".to_string()
    } else {
        format!("the last good tick is {}", at - 1)
    };
    assert!(err.contains(&last), "{err}");
    let h = lines(&hashes);
    assert_eq!(h.len() as u64, at);
    assert!(stdout(&out).is_empty());
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
fn failed_final_checkpoint_save_stops_the_run() {
    // O12 (P0.9): with --out and no --checkpoint-every, the final state is written after the
    // last tick. When that write fails (--out names a regular file) the run fails with exit 3
    // and the error, not exit 0 with a final hash: July's N11 was a failed save that reported
    // success. Every tick ran, so the hashes file is complete.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let file = dir.path().join("not-a-dir");
    fs::write(&file, "a regular file").unwrap();
    let hashes = dir.path().join("h.txt");
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "10",
        "--out",
        s(&file),
        "--hashes",
        s(&hashes),
    ]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("cannot create the directory"),
        "{}",
        stderr(&out)
    );
    assert!(stdout(&out).is_empty(), "no final hash after a failed save");
    let h = lines(&hashes);
    assert_eq!(h.len(), 10);
    assert!(h[9].starts_with("10 0x"), "{h:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), "a regular file");
    // The same with --checkpoint-every when --until is not a multiple of it: the periodic
    // saves are skipped until tick 10 would be one, and the final save at 7 fails.
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "7",
        "--out",
        s(&file),
        "--checkpoint-every",
        "10",
    ]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(stdout(&out).is_empty());
}

#[test]
fn resume_refuses_an_edited_identity() {
    // O7 (P0.9), through the product path: a resume under a tape whose past differs is refused
    // (exit 3) and prints the tape's prefix_id. Writing that value into the RON checkpoint made
    // the same resume exit 0 under format 2, whose digest covered the state alone; now the
    // checkpoint is refused as edited, exit 3. The same for world_id under an edited world.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let cps = dir.path().join("cps");
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "600",
        "--out",
        s(&cps),
        "--format",
        "ron",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let cp = cps.join("tick_00000600.ron");
    let text = fs::read_to_string(&cp).unwrap();
    let past = write_tape(
        dir.path(),
        "cut61.ron",
        &edit(
            r#"(key: "mine.cut", at: "1760-03-01""#,
            r#"(key: "mine.cut", at: "1761-03-01""#,
        ),
    );
    let world = write_tape(
        dir.path(),
        "bread53.ron",
        &edit(
            r#"(key: "rate.bread", value: 5.2,"#,
            r#"(key: "rate.bread", value: 5.3,"#,
        ),
    );
    for (edited, field, printed) in [
        (&past, "prefix_id", "(prefix 0x"),
        (&world, "world_id", "the tape is world 0x"),
    ] {
        let out = rustyecon(&["resume", s(&cp), "--tape", s(edited), "--until", "700"]);
        assert_eq!(code(&out), 3, "{}", stderr(&out));
        // The refusal prints the tape's value, as 16 hex digits.
        let err = stderr(&out);
        let at = err
            .find(printed)
            .unwrap_or_else(|| panic!("{printed} in {err}"))
            + printed.len();
        let value = u64::from_str_radix(&err[at..at + 16], 16).unwrap();
        let line = text
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{field}: ")))
            .unwrap();
        let forged = text.replacen(line, &format!("    {field}: {value},"), 1);
        assert_ne!(forged, text);
        let p = dir.path().join(format!("{field}.ron"));
        fs::write(&p, &forged).unwrap();
        let out = rustyecon(&["resume", s(&p), "--tape", s(edited), "--until", "700"]);
        assert_eq!(code(&out), 3, "{field}: {}", stderr(&out));
        assert!(
            stderr(&out).contains("edited or corrupted"),
            "{field}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn registry_lists_every_number() {
    // R4: every param with its unit, its use, each place the run reads it with that read's
    // conversion and per-tick value (from S2.2, D10 item 4), and its basis; then the inline
    // numbers.
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
    let clock = Clock {
        start: Date::parse("1750-01-01").unwrap(),
        ticks_per_year: 52,
    };
    let spend = ClockMethod::Share.per_tick(&clock, 10.4).unwrap();
    let spend = format!(
        "params[mill.spend]\t1.04e1\tRatePerYear live\tshare {spend:e} per tick at \
         actors[mill].spec.spend\tAssumed(gate world)"
    );
    for expected in [
        "params[mine.capacity]\t5.2e1\tFlowPerYear live\tflow 1e0 per tick at \
         actors[mine].spec.recipe.capacity\tAssumed(gate world)",
        "params[life.bread]\t5.77e-2\tYears fixed\tticks 3e0 per tick at goods[bread].life\t\
         Assumed(three weeks)",
        "params[rate.grain]\t5.2e0\tRatePerYear live\tlog_step 1e-1 per tick at \
         goods[grain].price_rate\t",
        spend.as_str(),
        // Read by the schedule alone: a SetParam's source converts as the param it sets, at the
        // event's `to`, and a recurring period is whole ticks, at its `every`.
        "params[mine.capacity.cut]\t2.6e1\tFlowPerYear schedule\tflow 5e-1 per tick at \
         events[mine.cut].act.to\tAssumed(gate world: half)",
        "params[pension.period]\t1e0\tYears schedule\tticks 5.2e1 per tick at \
         recurring[pension].every\tAssumed(yearly)",
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

#[test]
fn resume_refuses_an_edited_checkpoint() {
    // R2, E1, N11: a RON checkpoint is readable, which invites editing it. An edit that is not
    // matched by the stored digest (a million coin for the workers) is refused before the run
    // resumes, exit 3; before format 2 it resumed with the coin in no ledger line. A forger who
    // recomputes the digest gets past that check, but a state that does not fit the world (a
    // bread lot with a life bread cannot have) is still refused by the resume's validation,
    // exit 3.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let cps = dir.path().join("cps");
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "100",
        "--out",
        s(&cps),
        "--format",
        "ron",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = fs::read_to_string(cps.join("tick_00000100.ron")).unwrap();
    let resume = |name: &str, text: &str| {
        let p = dir.path().join(name);
        fs::write(&p, text).unwrap();
        rustyecon(&["resume", s(&p), "--tape", s(&tape), "--until", "200"])
    };
    // The workers are Pop(1); their coin, good 1, is one lot: 1e6 is written in front of it.
    let at = text.find("Actor(Pop((1))): [").unwrap();
    let start =
        at + text[at..].find("                    (").unwrap() + "                    (".len();
    let richer = format!("{}1000000{}", &text[..start], &text[start..]);
    let out = resume("richer.ron", &richer);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("edited or corrupted"),
        "{}",
        stderr(&out)
    );
    // A bread lot with a life of 1000 ticks, its digest recomputed to match.
    let at = text.find("holdings: {").unwrap();
    let life = at + text[at..].find("Some(").unwrap() + "Some(".len();
    let close = life + text[life..].find(')').unwrap();
    let edited = format!("{}1000{}", &text[..life], &text[close..]);
    let forged = match rustyecon_engine::Checkpoint::from_ron(&edited) {
        Err(rustyecon_engine::prelude::CheckpointError::Digest { stored, computed }) => edited
            .replacen(
                &format!("digest: {stored},"),
                &format!("digest: {computed},"),
                1,
            ),
        other => panic!("expected a digest mismatch, got {other:?}"),
    };
    let out = resume("forged.ron", &forged);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("which bread cannot have"),
        "{}",
        stderr(&out)
    );
    // The untouched checkpoint resumes.
    let out = resume("same.ron", &text);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}
