//! Branches through the whole seam (docs/GUI.md §5.1, §8.1; E1, N11, D3, U3): the editor's
//! Apply, `plan`, a new Runner on its own `ThreadDriver` worker that receives the parent's ring
//! checkpoint in `Cmd::Load`, and compare and export over what the two runs recorded.
//!
//! When `RUSTYECON_GUI_HASHES` names a directory, `branch_resume_equals_rerun` writes the
//! branch's tape there as `branch.ron` and its hashes as `branch.hashes`, and
//! `removal_only_branch_is_an_experiment` writes `removal.ron` and `removal.hashes`: one
//! `{t} 0x{hash:016x}` line per tick from 1 to 2,080, the parent's record before the resume
//! and the branch's own after it. `scripts/gui.sh` compares each with the cli binary's run of
//! that tape.

mod common;

use certify::Manifest;
use common::{
    build, driver, gate_tick, key, pump_until, reference_hashes, tape_of, today, wait_paused, GATE,
    GATE_TICKS,
};
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Driver;
use rustyecon_gui::drive::Host;
use rustyecon_gui::edit::export::{GuiManifest, LINEAGE, MANIFEST, SERIES, TAPE};
use rustyecon_gui::edit::{materialise, EditOp, Form, Lineage, OpKind, TapeEdit};
use rustyecon_gui::model::{Intent, Model, Run};
use rustyecon_gui::run::{
    Cmd, Obs, Origin, PauseReason, RunId, RunStatus, Store, EXPERIMENT_MARKER,
};
use rustyecon_gui::vm;
use std::sync::Arc;
use std::time::Instant;

/// A model and a host whose runs are `ThreadDriver` workers, with the gate open, run to its
/// end and paused.
fn gate_run_to_the_end() -> (Model, Host) {
    let mut m = Model::default();
    let mut host = Host::new(build(), Arc::new(|| {}), None);
    host.act(&mut m, Intent::Today(today()));
    host.act(
        &mut m,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    pump_until(&mut host, &mut m, "the gate loaded", |m| {
        m.focused().is_some_and(|r| r.store.run().is_some())
    });
    run_to_the_end(&mut host, &mut m);
    (m, host)
}

/// Run the focused run to state tick 2,080 and wait for its pause.
fn run_to_the_end(host: &mut Host, m: &mut Model) {
    host.act(
        m,
        Intent::Run {
            until: Some(GATE_TICKS),
        },
    );
    pump_until(host, m, "the run's end", |m| {
        m.focused().is_some_and(|r| {
            r.store.status()
                == RunStatus::Paused {
                    tick: GATE_TICKS,
                    why: Some(PauseReason::Reached(GATE_TICKS)),
                }
        })
    });
}

/// Wait for the focused run, a new branch, to load.
fn loaded(host: &mut Host, m: &mut Model) {
    pump_until(host, m, "the branch loaded", |m| {
        m.focused().is_some_and(|r| r.store.run().is_some())
    });
}

/// The branch's hash lines, 1 to 2,080: the parent's record before the branch's start, which
/// the branch shares, and the branch's own from there.
fn hash_lines(parent: &Run, branch: &Run) -> String {
    let start = branch.store.start();
    let mut body = String::new();
    for t in 1..=GATE_TICKS {
        let h = if t <= start {
            parent.store.state_hash_at(t)
        } else {
            branch.store.state_hash_at(t)
        };
        body.push_str(&Manifest::line(t, h.expect("a recorded hash")));
    }
    body
}

/// Write a branch's tape and hash lines where `RUSTYECON_GUI_HASHES` names, for gui.sh.
fn write_for_the_cli(name: &str, parent: &Run, branch: &Run) {
    if let Some(dir) = std::env::var_os("RUSTYECON_GUI_HASHES") {
        let dir = std::path::Path::new(&dir);
        std::fs::write(dir.join(format!("{name}.ron")), branch.tape.to_ron())
            .expect("the tape is written");
        std::fs::write(
            dir.join(format!("{name}.hashes")),
            hash_lines(parent, branch),
        )
        .expect("the hashes are written");
    }
}

#[test]
fn branch_resume_equals_rerun() {
    // §8.1: a SetParam of mine.capacity to mine.capacity.base, dated 1765-06-01, restores the
    // capacity early. mine.capacity.base is a source already (mine.restored), so world_id is
    // kept. The branch resumes from the largest ring tick at or before tick_of(1765-06-01),
    // tick_of(1765-01-01); its hashes equal a rerun of its tape from genesis; and the first
    // differing report hash is at tick_of(1765-06-01).
    let (mut m, mut host) = gate_run_to_the_end();
    let parent = m.focus().unwrap();
    let k = m.mint_key().unwrap();
    host.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::AddEvent,
            key: k.to_string(),
            date: "1765-06-01".to_string(),
            act: "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")".to_string(),
            note: "restore the capacity early".to_string(),
            ..Form::default()
        }),
    );
    assert_eq!(m.editor().error, None);
    host.act(&mut m, Intent::Apply);
    let branch = m.focus().unwrap();
    assert_ne!(branch, parent);
    let (edit, jan) = (gate_tick("1765-06-01"), gate_tick("1765-01-01"));
    assert!(jan < edit);
    let b = m.run(branch).unwrap();
    assert_eq!(b.parent, Some(parent));
    assert_eq!(
        b.resume.as_ref().map(|cp| cp.tick()),
        Some(jan),
        "the largest ring tick at or before the edit"
    );
    // The ring tick is the tick 1765-01-01 falls in, which may begin in the old year.
    let begins = m
        .run(parent)
        .unwrap()
        .store
        .world()
        .unwrap()
        .clock
        .date_of(jan);
    let begins = begins.unwrap();
    assert!(begins <= Date::parse("1765-01-01").unwrap());
    assert!(m.log().iter().any(|l| l.text.ends_with(&format!(
        "resumes from the parent's ring checkpoint at state tick {jan} ({begins})"
    ))));
    loaded(&mut host, &mut m);
    let b = m.run(branch).unwrap();
    assert_eq!(b.store.start(), jan, "the Runner resumed there");
    assert!(b.store.refusals().is_empty());
    let p = m.run(parent).unwrap();
    assert_eq!(
        b.store.run().unwrap().world_id,
        p.store.run().unwrap().world_id,
        "world_id is kept"
    );
    assert_eq!(b.store.start_hash(), p.store.state_hash_at(jan).unwrap());
    run_to_the_end(&mut host, &mut m);
    let (p, b) = (m.run(parent).unwrap(), m.run(branch).unwrap());
    // The resumed run equals the rerun.
    let rerun = reference_hashes(&b.tape, GATE_TICKS);
    assert_eq!(b.store.hashes(), &rerun[jan as usize..]);
    // The parent's hashes agree with the rerun's until the edit's tick and not after.
    let first = rerun
        .iter()
        .zip(p.store.hashes())
        .position(|(a, b)| a != b)
        .expect("the edit changes the run") as u64;
    assert_eq!(first, edit, "the first differing report hash");
    // Compare says the same, and nothing differs before the resume.
    let lines = b.lineage.as_ref().map(|l| l.lines()).unwrap();
    let c = vm::compare::build(&p.store, &b.store, &lines, &m.session.plots, None).unwrap();
    let d = c.first_difference.expect("a difference");
    assert_eq!((d.report_tick, d.state_tick), (Some(edit), edit + 1));
    assert!(!c.before_resume);
    assert_eq!(c.compared, (jan, GATE_TICKS));
    assert_eq!(c.tape.len(), 2, "the name and the event: {:?}", c.tape);
    assert_eq!(c.tape[1].key, k.to_string());
    let price = c
        .series
        .iter()
        .find(|s| s.label == "price at town/fuel")
        .unwrap();
    assert!(price.first_differs.is_some_and(|t| t > edit));
    assert!(lines[0].starts_with("from gate.ron (tape_hash 0x54066d053474846b)"));
    write_for_the_cli("branch", p, b);
    // An export of the branch records the resume in its manifest.
    let files = m.export_files(branch).unwrap();
    let manifest = files.iter().find(|(n, _)| n == MANIFEST).unwrap();
    let g = GuiManifest::from_ron(&manifest.1).unwrap();
    let r = g.manifest.resumed_from.as_ref().expect("resumed");
    assert_eq!((r.tick, &r.parent), (jan, p.store.run().unwrap()));
    assert_eq!(r.state_hash.0, p.store.state_hash_at(jan).unwrap());
    assert_eq!(g.manifest.from, jan);
    assert_eq!(g.manifest.hashes.count, GATE_TICKS - jan);
    assert_eq!(
        g.manifest.hashes.last.map(|(t, h)| (t, h.0)),
        Some((GATE_TICKS, rerun[rerun.len() - 1]))
    );
    // A resumed record whose start is not the parent's state there is flagged by compare.
    let mut forged = Store::default();
    forged
        .ingest(Obs::Loaded {
            run: b.store.run().unwrap().clone(),
            tape: Box::new(b.tape.clone()),
            world: Box::new(b.store.world().unwrap().clone()),
            tick: jan,
            hash: p.store.state_hash_at(jan).unwrap() ^ 1,
        })
        .unwrap();
    let c = vm::compare::build(&p.store, &forged, &[], &[], None).unwrap();
    assert!(c.before_resume);
    assert_eq!(c.first_difference.map(|d| d.state_tick), Some(jan));
}

#[test]
fn removal_only_branch_is_an_experiment() {
    // §8.1 and D3: RemoveEvent(mine.cut), then the offered RemoveParam(mine.capacity.cut). No
    // entry is stamped, and the branch's name carries the marker, so it is an experiment, and
    // its export says so. mine.capacity.cut is a param only the schedule reads, outside
    // world_id (ENGINE §2.6), so the removals keep the world: the branch resumes from the ring
    // tick before the cut, 1760-01-01's, and its hashes equal a rerun's. A world edit reruns
    // from genesis and the log says why.
    let (mut m, mut host) = gate_run_to_the_end();
    let parent = m.focus().unwrap();
    host.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::RemoveEvent,
            key: "mine.cut".to_string(),
            note: "no cut".to_string(),
            ..Form::default()
        }),
    );
    host.act(&mut m, Intent::Apply);
    assert_eq!(m.editor().offer, [key("mine.capacity.cut")]);
    host.act(
        &mut m,
        Intent::RemoveOrphans {
            note: "the cut's value goes with it".to_string(),
        },
    );
    let branch = m.focus().unwrap();
    assert_ne!(branch, parent);
    let b = m.run(branch).unwrap();
    let name = format!("gate [{EXPERIMENT_MARKER} {}]", today());
    assert_eq!(b.tape.header.name, name);
    assert_eq!(b.origin, Origin::Experiment);
    assert_eq!(Origin::of(&b.tape), Origin::Experiment);
    // Removals stamp nothing: the marker is in the name alone.
    let text = b.tape.to_ron();
    assert_eq!(text.matches(EXPERIMENT_MARKER).count(), 1, "{text}");
    let jan = gate_tick("1760-01-01");
    assert!(jan < gate_tick("1760-03-01"));
    assert_eq!(b.resume.as_ref().map(|cp| cp.tick()), Some(jan));
    loaded(&mut host, &mut m);
    run_to_the_end(&mut host, &mut m);
    let (p, b) = (m.run(parent).unwrap(), m.run(branch).unwrap());
    let rerun = reference_hashes(&b.tape, GATE_TICKS);
    assert_eq!(b.store.start(), jan);
    assert_eq!(b.store.hashes(), &rerun[jan as usize..]);
    write_for_the_cli("removal", p, b);
    // The toolbar's identity chip says "experiment".
    let t = vm::toolbar::build(&b.store, b.origin, m.ledger_changed(branch));
    assert_eq!(
        t.identity.as_ref().map(|i| i.origin),
        Some(Origin::Experiment)
    );
    assert_eq!(t.identity.map(|i| i.name), Some(name.clone()));
    let (run_key, tape_hash) = (b.store.run().unwrap().clone(), b.tape_hash);
    // Its export says "experiment" in the CSV's # lines and in the manifest, and carries the
    // lineage with both edits and what they removed.
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("export");
    host.act(&mut m, Intent::Export(out.display().to_string()));
    let csv = std::fs::read_to_string(out.join(SERIES)).expect("the CSV is written");
    let head: Vec<&str> = csv.lines().take_while(|l| l.starts_with('#')).collect();
    assert_eq!(head[0], "# rustyecon-gui export: experiment, draft");
    assert!(head[1].starts_with(&format!(
        "# run build {} {} ",
        build().commit,
        build().state()
    )));
    assert!(
        head[1].contains(&format!("tape {name} tape_hash ")),
        "{}",
        head[1]
    );
    assert!(head.contains(&"# origin experiment"));
    assert!(head.contains(&"# lineage tape.lineage.ron"));
    let columns = csv.lines().find(|l| !l.starts_with('#')).unwrap();
    assert!(
        columns.starts_with("tick,date,price at town/bread,"),
        "{columns}"
    );
    assert_eq!(
        csv.lines().filter(|l| !l.starts_with('#')).count() as u64,
        1 + GATE_TICKS - jan,
        "a header and one row per report tick the branch ran"
    );
    let g = GuiManifest::from_ron(&std::fs::read_to_string(out.join(MANIFEST)).unwrap())
        .expect("the envelope reads");
    assert_eq!(g.origin, Origin::Experiment);
    assert!(g.draft);
    assert_eq!(g.stamp(), "experiment, draft");
    assert_eq!(g.lineage.as_deref(), Some(LINEAGE));
    assert_eq!(g.manifest.run, run_key);
    let tape = Tape::from_ron(&std::fs::read_to_string(out.join(TAPE)).unwrap()).unwrap();
    assert_eq!(certify::tape_hash(&tape), tape_hash);
    let l = Lineage::from_ron(&std::fs::read_to_string(out.join(LINEAGE)).unwrap()).unwrap();
    let notes: Vec<&str> = l.edits.iter().map(|a| a.edit.note.as_str()).collect();
    assert_eq!(notes, ["no cut", "the cut's value goes with it"]);
    assert!(l.edits.iter().all(|a| a.replaced.is_some()));
    // The base's export says "run".
    let files = m.export_files(parent).unwrap();
    let csv = &files.iter().find(|(n, _)| n == SERIES).unwrap().1;
    assert!(csv.starts_with("# rustyecon-gui export: run, draft\n"));
    assert!(!files.iter().any(|(n, _)| n == LINEAGE));
    // A world edit reruns from genesis, and says why.
    host.act(&mut m, Intent::Focus(parent));
    host.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::SetGenesisParam,
            key: "mill.spend".to_string(),
            value: "12".to_string(),
            note: "spend more".to_string(),
            ..Form::default()
        }),
    );
    host.act(&mut m, Intent::Apply);
    let world = m.focus().unwrap();
    let w = m.run(world).unwrap();
    assert!(w.resume.is_none());
    let why = w.rerun.clone().expect("a rerun");
    assert!(why.starts_with("the edit changes the world: world_id 0x43628a8e0fd5f695 became "));
    assert!(m
        .log()
        .iter()
        .any(|l| l.text.ends_with(&format!("reruns from genesis: {why}"))));
    loaded(&mut host, &mut m);
    assert_eq!(m.run(world).unwrap().store.start(), 0);
    assert_eq!(m.run(RunId(0)).map(|r| r.id), Some(parent));
    // Its genesis is another world's, which compare reports as the first difference, at state
    // tick 0, and does not flag: a rerun does not share its parent's start.
    let (p, w) = (m.run(parent).unwrap(), m.run(world).unwrap());
    let c = vm::compare::build(&p.store, &w.store, &[], &m.session.plots, None).unwrap();
    let d = c.first_difference.expect("another genesis");
    assert_eq!((d.state_tick, d.report_tick), (0, None));
    assert!(!c.before_resume);
}

#[test]
#[ignore = "a measurement for STATE.md, not a gate: --release -- --ignored --nocapture"]
fn gate_rerun_time_is_recorded() {
    // docs/GUI.md §9, G0's gate: the gate world's rerun time, with no threshold. A world edit
    // (mill.spend's genesis value) through materialise, then Sim::new and a run to 2,080
    // through ThreadDriver, whose Runner extracts every tick with the G0 Extractor; the
    // median of 5.
    let gate = tape_of(GATE);
    let edit = [TapeEdit {
        op: EditOp::SetGenesisParam {
            key: key("mill.spend"),
            value: 12.0,
        },
        note: "spend more".to_string(),
    }];
    let mut times = Vec::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        let b = materialise(&gate, &edit, today()).expect("a branch");
        let mut d = driver();
        let mut log = Vec::new();
        d.send(Cmd::Load {
            tape: Box::new(b.tape),
            from: None,
        });
        d.send(Cmd::Run {
            until: Some(GATE_TICKS),
            max_tps: None,
        });
        assert_eq!(
            wait_paused(&mut d, &mut log),
            (GATE_TICKS, PauseReason::Reached(GATE_TICKS))
        );
        times.push(t0.elapsed());
    }
    times.sort();
    println!(
        "gate rerun (materialise, Sim::new, run_until 2080 through ThreadDriver): median {:.1} ms \
         of {:?}",
        times[2].as_secs_f64() * 1e3,
        times
    );
}
