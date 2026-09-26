//! The product path (docs/ENGINE.md §8 and §11, cli): the `rustyecon` binary on the gate world,
//! its checkpoints and hashes files, and its exit codes. Hashes are compared exactly. From S2.4
//! (docs/CERTIFY.md §10): the hash output names its run, a run's manifest records its
//! checkpoints, a resume verifies against it, and `certify` writes its verdict and exits on it.
//!
//! Test bars for `certify`, named once and none of them a registered bar: runs of 520 ticks in
//! one-year segments, resumed at half, with a runaway bound of 1e4 (loose enough to pass) or of
//! 1 + 1e-6 (tight enough to fail).

use certify::{tape_hash, Certificate, Criteria, Hex, Manifest, ResumedFrom, Verdict};
use rustyecon_engine::prelude::{Clock, ClockMethod, Date, Sim};
use rustyecon_engine::{Checkpoint, Tape};
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

/// The body of a hashes file: its `{t} 0x{hash}` lines, without the `#` header that names the
/// run (docs/CERTIFY.md §10; `hash_output_names_its_run` checks the header).
fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("the hashes file")
        .lines()
        .filter(|l| !l.starts_with('#'))
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
    // Each run writes its own directory, since its manifest.ron, which a resume verifies
    // against, records the checkpoints that run made (from S2.4, docs/CERTIFY.md §10).
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
        let runs: [(&str, Option<&str>, &[usize]); 3] = [
            ("2080", Some("520"), &[520, 1040, 1560, 2080]),
            ("1", None, &[1]),
            ("2079", None, &[2079]),
        ];
        for (until, every, _) in runs {
            let run_dir = cps.join(format!("until-{until}"));
            let mut args = vec![
                "run",
                s(&tape),
                "--until",
                until,
                "--out",
                s(&run_dir),
                "--format",
                format,
            ];
            if let Some(n) = every {
                args.extend(["--checkpoint-every", n]);
            }
            let out = rustyecon(&args);
            assert_eq!(code(&out), 0, "{}", stderr(&out));
        }
        let holder = |at: usize| {
            let (until, _, _) = runs
                .iter()
                .find(|(_, _, made)| made.contains(&at))
                .expect("a run made it");
            cps.join(format!("until-{until}"))
        };
        for name in [1, 520, 1040, 1560, 2079, 2080] {
            assert!(
                holder(name).join(format!("tick_{name:08}.{ext}")).is_file(),
                "{name}"
            );
        }
        for at in [1usize, 520, 1040, 2079] {
            let cp = holder(at).join(format!("tick_{at:08}.{ext}"));
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
    // R16 and C9 (from S2.4): a dated edit after the checkpoint keeps the world and the past, so
    // the engine would resume it (E1), but it is another tape, and the manifest of the run that
    // made the checkpoint names this one's tape_hash: exit 3, unverified.
    let later = write_tape(
        dir.path(),
        "later.ron",
        &edit(
            r#"(key: "bread.line.up", at: "1765-09-01""#,
            r#"(key: "bread.line.up", at: "1766-09-01""#,
        ),
    );
    let out = rustyecon(&["resume", s(&cp), "--tape", s(&later), "--until", "700"]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains("unverified checkpoint") && err.contains("tape_hash"),
        "{err}"
    );
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
    // Each copy is written away from its run's directory, so it names that run's manifest (from
    // S2.4); the refusals below come before the manifest is read, and their messages stand.
    let manifest = cps.join("manifest.ron");
    let resume = |name: &str, text: &str| {
        let p = dir.path().join(name);
        fs::write(&p, text).unwrap();
        rustyecon(&[
            "resume",
            s(&p),
            "--tape",
            s(&tape),
            "--until",
            "200",
            "--manifest",
            s(&manifest),
        ])
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

/// The build the binary under test was stamped with (`build.rs`): commit, state and target.
fn stamped() -> String {
    let state = if env!("RUSTYECON_DIRTY") == "false" {
        "clean"
    } else {
        "dirty"
    };
    format!(
        "{} {state} {}",
        env!("RUSTYECON_COMMIT"),
        env!("RUSTYECON_TARGET")
    )
}

/// The gate's tape hash and world, as `0x…`, and its genesis hash.
fn gate_ids() -> (String, String, u64) {
    let t = Tape::from_ron(GATE).unwrap();
    let sim = Sim::new(&t).unwrap();
    (
        format!("0x{:016x}", tape_hash(&t)),
        format!("0x{:016x}", sim.world().world_id),
        sim.hash(),
    )
}

/// The `#` header of a hashes file.
fn header(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .take_while(|l| l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn read_manifest(path: &Path) -> Manifest {
    Manifest::from_ron(&fs::read_to_string(path).unwrap()).expect("the manifest reads")
}

fn digest_of(cp: &Path) -> u64 {
    Checkpoint::from_bytes(&fs::read(cp).unwrap())
        .unwrap()
        .digest()
}

/// FNV-1a 64, written here apart from core's.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[test]
fn hash_output_names_its_run() {
    // R16 (docs/CERTIFY.md §10): a hashes file opens with `#` lines naming the build, the tape
    // by tape_hash, the world and the starting tick, with the resumed checkpoint's digest on a
    // resume; stdout prints a `run …` line with the same fields before the final hash, and
    // replay names its run the same way. The body is unchanged.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let (th, wid, _) = gate_ids();
    let build = stamped();
    let named = format!("tape gate tape_hash {th} world_id {wid}");
    let cps = dir.path().join("cps");
    let h = dir.path().join("h.txt");
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "520",
        "--out",
        s(&cps),
        "--hashes",
        s(&h),
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(
        header(&h),
        [
            "# rustyecon hashes 1".to_string(),
            format!("# build {build}"),
            format!("# {named}"),
            "# from 0".to_string(),
        ]
    );
    let body = lines(&h);
    assert_eq!(body.len(), 520);
    assert!(body[0].starts_with("1 0x"));
    let printed: Vec<String> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(
        printed,
        [
            format!("run build {build} {named} from 0"),
            body[519].clone()
        ]
    );
    // A resume names the checkpoint it started from, by digest.
    let cp = cps.join("tick_00000520.bin");
    let digest = format!("0x{:016x}", digest_of(&cp));
    let h2 = dir.path().join("h2.txt");
    let out = rustyecon(&[
        "resume",
        s(&cp),
        "--tape",
        s(&tape),
        "--until",
        "600",
        "--hashes",
        s(&h2),
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let resumed = header(&h2);
    assert_eq!(
        resumed,
        [
            "# rustyecon hashes 1".to_string(),
            format!("# build {build}"),
            format!("# {named}"),
            format!("# from 520 resumed {digest}"),
        ]
    );
    assert_eq!(lines(&h2).len(), 80);
    let text = stdout(&out);
    let printed: Vec<&str> = text.lines().collect();
    assert_eq!(printed.len(), 2);
    assert_eq!(
        printed[0],
        format!("run build {build} {named} from 520 resumed {digest}")
    );
    // Replay names its run before its final hash.
    let out = rustyecon(&["replay", s(&tape), "--until", "520"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    let printed: Vec<&str> = text.lines().collect();
    assert_eq!(printed[1], format!("run build {build} {named} from 0"));
    assert_eq!(printed[2], body[519]);
}

#[test]
fn resume_requires_a_recorded_checkpoint() {
    // R16 and C9 (docs/CERTIFY.md §9, §10): a resume is verified against the manifest of the
    // run that made the checkpoint, and each way it can fail is exit 3, "unverified
    // checkpoint": no manifest beside it, a manifest that does not read, one of another world,
    // one with no record at its tick, and a record whose digest or state hash differs.
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
        "--checkpoint-every",
        "300",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let cp = cps.join("tick_00000300.bin");
    let good = read_manifest(&cps.join("manifest.ron"));
    assert_eq!(
        good.checkpoints.iter().map(|c| c.tick).collect::<Vec<_>>(),
        [300, 600]
    );
    let resume = |cp: &Path, manifest: Option<&Path>| {
        let mut args = vec!["resume", s(cp), "--tape", s(&tape), "--until", "700"];
        if let Some(m) = manifest {
            args.extend(["--manifest", s(m)]);
        }
        rustyecon(&args)
    };
    let refused = |out: &Output, why: &str| {
        assert_eq!(code(out), 3, "{why}: {}", stderr(out));
        let err = stderr(out);
        assert!(
            err.contains("unverified checkpoint") && err.contains(why),
            "{why}: {err}"
        );
        assert!(stdout(out).is_empty(), "{why}: no tick ran");
    };
    // No manifest beside a copy.
    let alone = dir.path().join("alone");
    fs::create_dir_all(&alone).unwrap();
    fs::copy(&cp, alone.join("tick_00000300.bin")).unwrap();
    refused(
        &resume(&alone.join("tick_00000300.bin"), None),
        "cannot read the manifest",
    );
    // A manifest that does not read.
    let junk = dir.path().join("junk.ron");
    fs::write(&junk, "Manifest(format: 1)").unwrap();
    refused(&resume(&cp, Some(&junk)), "does not parse");
    let write = |name: &str, m: &Manifest| {
        let p = dir.path().join(name);
        fs::write(&p, m.to_ron()).unwrap();
        p
    };
    // Another world.
    let mut m = good.clone();
    m.run.world_id = Hex(m.run.world_id.0 ^ 1);
    refused(&resume(&cp, Some(&write("world.ron", &m))), "world_id");
    // No record at the checkpoint's tick.
    let mut m = good.clone();
    m.checkpoints.retain(|c| c.tick != 300);
    refused(
        &resume(&cp, Some(&write("unrecorded.ron", &m))),
        "records no checkpoint at tick 300",
    );
    // A record whose digest, or whose state hash, differs: a file replaced since.
    for what in ["digest", "state hash"] {
        let mut m = good.clone();
        let rec = m.checkpoints.iter_mut().find(|c| c.tick == 300).unwrap();
        if what == "digest" {
            rec.digest = Hex(rec.digest.0 ^ 1);
        } else {
            rec.state_hash = Hex(rec.state_hash.0 ^ 1);
        }
        refused(
            &resume(&cp, Some(&write("differs.ron", &m))),
            &format!("its {what} differs"),
        );
    }
    // The run's own manifest verifies, beside the checkpoint or named.
    for m in [None, Some(cps.join("manifest.ron"))] {
        let out = resume(&cp, m.as_deref());
        assert_eq!(code(&out), 0, "{}", stderr(&out));
    }
}

#[test]
fn resume_records_its_parent() {
    // D4 and R16 (docs/CERTIFY.md §9, §10): a resumed run's manifest records the checkpoint it
    // started from (its tick, digest and state hash) and the run that made it; it starts at
    // that tick, records its own checkpoints, and a resume of one of them verifies against it.
    // An --out that holds the manifest a resume verifies against is exit 1, and that manifest
    // is left as it was.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let (a, b, c) = (
        dir.path().join("a"),
        dir.path().join("b"),
        dir.path().join("c"),
    );
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "600",
        "--out",
        s(&a),
        "--checkpoint-every",
        "300",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let cp = a.join("tick_00000300.bin");
    let out = rustyecon(&[
        "resume",
        s(&cp),
        "--tape",
        s(&tape),
        "--until",
        "900",
        "--out",
        s(&b),
        "--checkpoint-every",
        "300",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let (ma, mb) = (
        read_manifest(&a.join("manifest.ron")),
        read_manifest(&b.join("manifest.ron")),
    );
    let rec = ma.checkpoints.iter().find(|c| c.tick == 300).unwrap();
    assert_eq!(
        mb.resumed_from,
        Some(ResumedFrom {
            tick: 300,
            digest: Hex(digest_of(&cp)),
            state_hash: rec.state_hash,
            parent: ma.run.clone(),
        })
    );
    assert_eq!((mb.from, mb.until, mb.hashes.count), (300, 900, 600));
    assert_eq!(mb.genesis_hash, ma.genesis_hash);
    assert_eq!(mb.run, ma.run);
    assert_eq!(
        mb.checkpoints.iter().map(|c| c.tick).collect::<Vec<_>>(),
        [600, 900]
    );
    // The resumed run's tick-600 state is the parent run's: the same state hash.
    let parent_600 = ma.checkpoints.iter().find(|c| c.tick == 600).unwrap();
    assert_eq!(mb.checkpoints[0].state_hash, parent_600.state_hash);
    // A checkpoint of the resumed run verifies against its own manifest.
    let out = rustyecon(&[
        "resume",
        s(&b.join("tick_00000600.bin")),
        "--tape",
        s(&tape),
        "--until",
        "700",
        "--out",
        s(&c),
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let mc = read_manifest(&c.join("manifest.ron"));
    assert_eq!(mc.resumed_from.as_ref().map(|r| r.tick), Some(600));
    // Writing the new manifest over the one it verified against is refused.
    let before = fs::read(a.join("manifest.ron")).unwrap();
    let out = rustyecon(&[
        "resume",
        s(&cp),
        "--tape",
        s(&tape),
        "--until",
        "700",
        "--out",
        s(&a),
    ]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("holds the manifest"),
        "{}",
        stderr(&out)
    );
    assert_eq!(fs::read(a.join("manifest.ron")).unwrap(), before);
}

/// A criteria file for the gate, run to 1760-01-01 (tick 520) in one-year segments and resumed
/// at half, with the runaway bound and tape hash given, in a directory of its own.
fn gate_criteria(dir: &Path, bound: f64, hash: &str) -> (PathBuf, String) {
    let text = format!(
        "Criteria(\n    format: 1,\n    date: \"2026-09-26\",\n    reason: \"cli test\",\n    \
         supersedes: None,\n    tape: (name: \"gate\", tape_hash: \"{hash}\"),\n    until: \
         \"1760-01-01\",\n    until_basis: Assumed(\"test\"),\n    min_segment: (value: 1.0, \
         unit: Years, basis: Assumed(\"test\")),\n    batteries: [Conservation, \
         Determinism(resume_at: [(value: 0.5, unit: Share, basis: Assumed(\"test\"))]), \
         Runaway(bound: (value: {bound:?}, unit: Ratio, basis: Assumed(\"test\")))],\n    \
         reports: (rationed_below: (value: 1e-9, unit: Relative, basis: Assumed(\"test\"))),\n)\n"
    );
    let d = dir.join(format!("criteria-{bound}-{hash}"));
    fs::create_dir_all(&d).unwrap();
    let p = d.join("gate-2026-09-26.ron");
    fs::write(&p, &text).unwrap();
    (p, text)
}

fn read_certificate(dir: &Path) -> Certificate {
    Certificate::from_ron(&fs::read_to_string(dir.join("certificate.ron")).unwrap())
        .expect("the certificate reads back")
}

#[test]
fn certify_command_exits_on_its_verdict() {
    // N4, C4 (docs/CERTIFY.md §10): `certify` writes its certificate, manifest and hash file,
    // prints the rendering verdict first, and exits 0 on PASS and 5 on FAIL or UNSCORED, each
    // with its certificate written. With neither --criteria nor --until, or with both, it is
    // exit 1, as are criteria that do not load or do not fit, and then nothing is written.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let (th, _, _) = gate_ids();
    let certify = |out: &Path, how: &[&str]| {
        let mut args = vec!["certify", s(&tape), "--out", s(out)];
        args.extend(how);
        rustyecon(&args)
    };
    let (pass, _) = gate_criteria(dir.path(), 1e4, &th);
    let (fail, _) = gate_criteria(dir.path(), 1.000_001, &th);
    let (other, _) = gate_criteria(dir.path(), 1e4, "0x0000000000000001");
    for (name, how, exit, verdict) in [
        ("pass", vec!["--criteria", s(&pass)], 0, Verdict::Pass),
        ("fail", vec!["--criteria", s(&fail)], 5, Verdict::Fail),
        ("unscored", vec!["--until", "100"], 5, Verdict::Unscored),
        ("other", vec!["--criteria", s(&other)], 5, Verdict::Unscored),
    ] {
        let out_dir = dir.path().join(name);
        let out = certify(&out_dir, &how);
        assert_eq!(code(&out), exit, "{name}: {}", stderr(&out));
        let c = read_certificate(&out_dir);
        assert_eq!(c.verdict(), verdict, "{name}");
        assert!(
            stdout(&out).starts_with(&format!("VERDICT: {verdict}  gate  tape_hash {th}")),
            "{name}: {}",
            stdout(&out)
        );
        let m = read_manifest(&out_dir.join("manifest.ron"));
        assert_eq!(&m.run, c.run());
        assert_eq!(m.hashes.count, c.until());
        assert_eq!(lines(&out_dir.join("hashes.txt")).len() as u64, c.until());
        assert!(!out_dir.join("telemetry.parquet").exists());
    }
    // Arguments and criteria that stop certify before it runs: exit 1, nothing written.
    let bad_name = dir.path().join("gate-criteria.ron");
    fs::copy(&pass, &bad_name).unwrap();
    let short = dir.path().join("short");
    fs::create_dir_all(&short).unwrap();
    let short_file = short.join("gate-2026-09-26.ron");
    fs::write(
        &short_file,
        fs::read_to_string(&pass)
            .unwrap()
            .replace("\"1760-01-01\"", "\"1750-06-01\""),
    )
    .unwrap();
    for (name, how) in [
        ("neither", vec![]),
        ("both", vec!["--criteria", s(&pass), "--until", "100"]),
        ("named", vec!["--criteria", s(&bad_name)]),
        ("unfit", vec!["--criteria", s(&short_file)]),
        // The telemetry file is made before the run, and removed when the criteria do not fit.
        (
            "unfit-telemetry",
            vec!["--criteria", s(&short_file), "--telemetry"],
        ),
    ] {
        let out_dir = dir.path().join(format!("refused-{name}"));
        let out = certify(&out_dir, &how);
        assert_eq!(code(&out), 1, "{name}: {}", stderr(&out));
        for f in [
            "certificate.ron",
            "manifest.ron",
            "hashes.txt",
            "telemetry.parquet",
        ] {
            assert!(!out_dir.join(f).exists(), "{name}: {f}");
        }
    }
}

#[test]
fn manifest_names_every_input() {
    // N10 (docs/CERTIFY.md §3, §9): a run's manifest names the build with its target, the tape
    // by tape_hash, the world, the genesis state's hash and the clock; a resumed run's names
    // the checkpoint's digest; a certificate names its criteria by hash; and with --telemetry
    // the manifest pins the Parquet file by its rows and digest.
    let dir = tempfile::tempdir().unwrap();
    let tape = write_tape(dir.path(), "gate.ron", GATE);
    let (th, wid, genesis) = gate_ids();
    let a = dir.path().join("a");
    let out = rustyecon(&[
        "run",
        s(&tape),
        "--until",
        "100",
        "--out",
        s(&a),
        "--checkpoint-every",
        "50",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let m = read_manifest(&a.join("manifest.ron"));
    assert_eq!(m.run.build.commit, env!("RUSTYECON_COMMIT"));
    assert_eq!(m.run.build.dirty, env!("RUSTYECON_DIRTY") != "false");
    assert_eq!(m.run.build.target, env!("RUSTYECON_TARGET"));
    assert_eq!(m.run.build.rustc, env!("RUSTYECON_RUSTC"));
    assert!(m.run.build.rustc.starts_with("rustc "), "{:?}", m.run.build);
    let commit = &m.run.build.commit;
    let hex40 = commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit());
    assert!(commit == "unknown" || hex40, "{commit}");
    assert_eq!(m.run.tape_hash.to_string(), th);
    assert_eq!(m.run.world_id.to_string(), wid);
    assert_eq!(m.genesis_hash, Hex(genesis));
    assert_eq!((m.tape.as_str(), m.ticks_per_year), ("gate", 52));
    assert_eq!(m.start.to_string(), "1750-01-01");
    assert_eq!((m.from, m.until), (0, 100));
    assert_eq!(m.resumed_from, None);
    // A resume names its checkpoint by digest.
    let cp = a.join("tick_00000050.bin");
    let b = dir.path().join("b");
    let out = rustyecon(&[
        "resume",
        s(&cp),
        "--tape",
        s(&tape),
        "--until",
        "60",
        "--out",
        s(&b),
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let mb = read_manifest(&b.join("manifest.ron"));
    assert_eq!(mb.resumed_from.map(|r| r.digest), Some(Hex(digest_of(&cp))));
    // A certificate names its criteria by hash, and telemetry is pinned by digest.
    let (crit, text) = gate_criteria(dir.path(), 1e4, &th);
    let c_dir = dir.path().join("c");
    let out = rustyecon(&[
        "certify",
        s(&tape),
        "--criteria",
        s(&crit),
        "--out",
        s(&c_dir),
        "--telemetry",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let c = read_certificate(&c_dir);
    let registered = Criteria::from_ron(&text, "gate-2026-09-26.ron").unwrap();
    let cref = c.criteria().expect("criteria");
    assert_eq!(cref.hash, Hex(registered.hash()));
    assert_eq!(cref.file, "gate-2026-09-26.ron");
    let mc = read_manifest(&c_dir.join("manifest.ron"));
    let parquet = fs::read(c_dir.join("telemetry.parquet")).unwrap();
    let t = mc.telemetry.expect("the telemetry is recorded");
    assert_eq!(t.file, "telemetry.parquet");
    assert_eq!(t.digest, Hex(fnv1a(&parquet)));
    assert!(t.rows > 0);
    assert!(parquet.starts_with(b"PAR1") && parquet.ends_with(b"PAR1"));
    // A telemetry file that cannot be created is exit 3, before the run.
    let blocked = dir.path().join("blocked");
    fs::create_dir_all(blocked.join("telemetry.parquet")).unwrap();
    let out = rustyecon(&[
        "certify",
        s(&tape),
        "--until",
        "10",
        "--out",
        s(&blocked),
        "--telemetry",
    ]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(!blocked.join("certificate.ron").exists());
}
