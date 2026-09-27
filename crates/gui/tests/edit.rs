//! The editor's rules (docs/GUI.md §5.1; D3, U2, E8): what `materialise` stamps and refuses,
//! the offer of a `RemoveParam`, minted keys, the lineage a saved tape carries, and "ledger
//! changed". Branches run on this thread through real Runners (`common::sync`), so each step is
//! deterministic; `tests/branch.rs` drives the same through `ThreadDriver`.

mod common;

use common::sync::SyncHost;
use common::{key, tape_of, today, GATE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::edit::export::{GuiManifest, LINEAGE, MANIFEST, SERIES};
use rustyecon_gui::edit::{
    self, form, keys, lineage_path, materialise, stamp, Act, Basis, EditError, EditOp, Form,
    FormError, Lineage, OpKind, Replaced, TapeEdit, Unit,
};
use rustyecon_gui::model::{Intent, Model, RawError};
use rustyecon_gui::run::{Origin, RunId, RunStatus, EXPERIMENT_MARKER};
use std::collections::{BTreeMap, BTreeSet};

fn d(s: &str) -> Date {
    Date::parse(s).expect("a date")
}

fn te(op: EditOp, note: &str) -> TapeEdit {
    TapeEdit {
        op,
        note: note.to_string(),
    }
}

fn set_param(param: &str, to: &str) -> Act {
    Act::SetParam {
        param: key(param),
        to: key(to),
    }
}

/// Every basis a tape carries, by where it sits: `params[k]`, `events[k]`, `recurring[k]`,
/// `actors[k]` and `genesis`.
fn bases(t: &Tape) -> BTreeMap<String, Basis> {
    let mut out = BTreeMap::new();
    for p in &t.params {
        out.insert(format!("params[{}]", p.key), p.basis.clone());
    }
    for e in &t.events {
        out.insert(format!("events[{}]", e.key), e.basis.clone());
    }
    for r in &t.recurring {
        out.insert(format!("recurring[{}]", r.key), r.basis.clone());
    }
    for a in &t.actors {
        out.insert(format!("actors[{}]", a.key), a.basis.clone());
    }
    out.insert("genesis".to_string(), t.genesis.basis.clone());
    out
}

#[test]
fn gui_edits_are_always_assumed() {
    // D3, U2: every entry an edit adds or changes carries its own basis, stamped
    // Assumed("GUI experiment <date>: <note>") with that edit's note; every other basis is
    // left as it was; the name carries the marker; and no edit carries a basis of its own.
    let gate = tape_of(GATE);
    for (on, suffix) in [(today(), "a"), (d("2031-02-28"), "b")] {
        let edits = vec![
            te(
                EditOp::AddParam {
                    key: key("gui.1.1"),
                    value: 40.0,
                    unit: Unit::FlowPerYear,
                },
                &format!("a new capacity {suffix}"),
            ),
            te(
                EditOp::AddEvent {
                    key: key("gui.1.2"),
                    at: d("1765-06-01"),
                    act: set_param("mine.capacity", "gui.1.1"),
                },
                &format!("restore to it {suffix}"),
            ),
            te(
                EditOp::AddRecurring {
                    key: key("gui.1.3"),
                    first: d("1752-01-01"),
                    every: key("pension.period"),
                    last: Some(d("1760-01-01")),
                    act: Act::Mint {
                        holder: key("pensioners"),
                        good: key("coin"),
                        qty: 1.0,
                    },
                },
                &format!("a bonus {suffix}"),
            ),
            te(
                EditOp::SetGenesisParam {
                    key: key("mill.spend"),
                    value: 12.0,
                },
                &format!("spend more {suffix}"),
            ),
            te(
                EditOp::RemoveEvent(key("oven.opens")),
                &format!("no oven {suffix}"),
            ),
        ];
        let b = materialise(&gate, &edits, on).expect("the edits make a branch");
        let before = bases(&gate);
        let after = bases(&b.tape);
        // The stamped entries: exactly those added or changed, each with its own note.
        let stamped: BTreeMap<&str, &str> = [
            ("params[gui.1.1]", "a new capacity"),
            ("events[gui.1.2]", "restore to it"),
            ("recurring[gui.1.3]", "a bonus"),
            ("params[mill.spend]", "spend more"),
        ]
        .into_iter()
        .collect();
        for (at, basis) in &after {
            match stamped.get(at.as_str()) {
                Some(note) => assert_eq!(
                    *basis,
                    Basis::Assumed(format!("GUI experiment {on}: {note} {suffix}")),
                    "{at}"
                ),
                None => assert_eq!(Some(basis), before.get(at), "{at} keeps its basis"),
            }
        }
        assert!(!after.contains_key("events[oven.opens]"));
        assert_eq!(after.len(), before.len() + 2, "three added, one removed");
        // Every basis the GUI wrote carries the marker, and nothing else in the tape does.
        let marked: BTreeSet<&str> = after
            .iter()
            .filter(|(_, b)| format!("{b:?}").contains(EXPERIMENT_MARKER))
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(marked, stamped.keys().copied().collect());
        assert_eq!(b.tape.header.name, format!("gate [GUI experiment {on}]"));
        assert_eq!(Origin::of(&b.tape), Origin::Experiment);
        // The value it changed, and what it replaced and removed.
        let spend = b
            .tape
            .params
            .iter()
            .find(|p| p.key.as_str() == "mill.spend");
        assert_eq!(spend.map(|p| p.value), Some(12.0));
        assert_eq!(
            b.applied[3].replaced,
            Some(Replaced::Value {
                value: 10.4,
                basis: Basis::Assumed("gate world".to_string()),
            })
        );
        let oven = gate.events.iter().find(|e| e.key.as_str() == "oven.opens");
        assert_eq!(
            b.applied[4].replaced,
            Some(Replaced::Event(oven.unwrap().clone()))
        );
        assert!(b.applied.iter().all(|a| a.on == on));
        // An edit, written out, carries no basis: the GUI chooses none.
        for e in &edits {
            let text = ron::to_string(e).unwrap();
            assert!(
                !text.contains("basis") && !text.contains("Assumed"),
                "{text}"
            );
        }
        assert_eq!(
            stamp(on, "why"),
            Basis::Assumed(format!("GUI experiment {on}: why"))
        );
    }
    // No edit without a note: empty, or only spaces.
    for note in ["", "   "] {
        let e = materialise(
            &gate,
            &[te(EditOp::RemoveEvent(key("oven.opens")), note)],
            today(),
        );
        assert_eq!(e.err(), Some(EditError::EmptyNote { edit: 0 }));
    }
    // A branch made only of removals is marked too, by its name.
    let b = materialise(
        &gate,
        &[te(EditOp::RemoveEvent(key("oven.opens")), "no oven")],
        today(),
    )
    .unwrap();
    assert_eq!(b.tape.header.name, "gate [GUI experiment 2026-09-27]");
    assert_eq!(Origin::of(&b.tape), Origin::Experiment);
    // A branch of a branch carries one marker, its own day's.
    let bb = materialise(
        &b.tape,
        &[te(
            EditOp::AddEvent {
                key: key("gui.2.1"),
                at: d("1766-01-01"),
                act: set_param("mine.capacity", "mine.capacity.base"),
            },
            "restore",
        )],
        d("2026-10-01"),
    )
    .expect("a branch of the branch");
    assert_eq!(bb.tape.header.name, "gate [GUI experiment 2026-10-01]");
    assert_eq!(bb.tape.header.name.matches(EXPERIMENT_MARKER).count(), 1);
}

#[test]
fn ledger_tolerances_are_not_editable() {
    // §5.1 item 2: the params header.ledger names are not editable. materialise refuses a
    // change of their genesis value, their removal and an event that sets one; the form
    // refuses them before that; and a tape whose tolerances differ from its parent's shows
    // "ledger changed" in the health chip and in the export.
    let gate = tape_of(GATE);
    for k in ["ledger.rel_flow", "ledger.rel_stock"] {
        for op in [
            EditOp::SetGenesisParam {
                key: key(k),
                value: 1e-10,
            },
            EditOp::RemoveParam(key(k)),
            EditOp::AddEvent {
                key: key("gui.1.1"),
                at: d("1760-01-01"),
                act: set_param(k, "ledger.rel_stock"),
            },
        ] {
            let e = materialise(&gate, &[te(op.clone(), "looser")], today());
            assert_eq!(
                e.err(),
                Some(EditError::Ledger {
                    edit: 0,
                    key: key(k)
                }),
                "{op}"
            );
            assert_eq!(edit::touches_ledger(&gate, &op), Some(key(k)));
        }
        let f = Form {
            kind: OpKind::SetGenesisParam,
            key: k.to_string(),
            value: "1e-10".to_string(),
            note: "looser".to_string(),
            ..Form::default()
        };
        assert_eq!(
            form::parse(&f, &gate, &BTreeSet::new()),
            Err(FormError::Ledger(k.to_string()))
        );
    }
    // Reading a tolerance changes nothing: a SetParam that copies one touches no tolerance.
    let copy = EditOp::AddEvent {
        key: key("gui.1.1"),
        at: d("1760-01-01"),
        act: set_param("price.ema_tc", "ledger.rel_flow"),
    };
    assert_eq!(edit::touches_ledger(&gate, &copy), None);
    // Through the model: the form is refused, and the refusal is shown.
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.ron");
    std::fs::write(&base, GATE).unwrap();
    let base = base.display().to_string();
    let mut m = Model::default();
    let mut h = SyncHost::new(64);
    h.act(&mut m, Intent::Today(today()));
    h.act(&mut m, Intent::Open(base.clone()));
    h.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::RemoveParam,
            key: "ledger.rel_flow".to_string(),
            note: "looser".to_string(),
            ..Form::default()
        }),
    );
    assert!(m.editor().staged.is_empty());
    assert_eq!(
        m.editor().error.as_deref(),
        Some(
            "refused: ledger.rel_flow is a ledger tolerance (header.ledger), which is not editable"
        )
    );
    // "Ledger changed": a branch keeps its parent's tolerances, and its chip says so.
    let base_run = m.focus().unwrap();
    h.act(&mut m, Intent::Run { until: Some(600) });
    h.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::RemoveEvent,
            key: "oven.opens".to_string(),
            note: "no oven".to_string(),
            ..Form::default()
        }),
    );
    h.act(&mut m, Intent::Apply);
    let branch = m.focus().unwrap();
    assert_ne!(branch, base_run);
    assert!(!m.ledger_changed(branch));
    let saved = dir.path().join("no-oven.ron").display().to_string();
    h.act(&mut m, Intent::SaveTape(saved.clone()));
    assert!(std::path::Path::new(&saved).exists());
    // The saved tape, its tolerance edited by hand, reopened: its lineage names the base,
    // which this session holds, and the tolerances differ.
    let mut t = Tape::from_ron(&std::fs::read_to_string(&saved).unwrap()).unwrap();
    let flow = t
        .params
        .iter_mut()
        .find(|p| p.key.as_str() == "ledger.rel_flow")
        .unwrap();
    assert_eq!(flow.value, 1e-12);
    flow.value = 2e-12;
    let by_hand = dir.path().join("by-hand.ron");
    std::fs::write(&by_hand, t.to_ron()).unwrap();
    std::fs::copy(
        lineage_path(&saved),
        lineage_path(&by_hand.display().to_string()),
    )
    .unwrap();
    h.act(&mut m, Intent::Open(by_hand.display().to_string()));
    let hand = m.focus().unwrap();
    assert_eq!(m.focused().unwrap().origin, Origin::Experiment);
    assert!(m.ledger_changed(hand), "the hand-edited tolerance shows");
    assert!(!m.ledger_changed(branch) && !m.ledger_changed(base_run));
    assert!(
        m.log()
            .iter()
            .any(|l| l.text.contains("the tape was edited after it was saved")),
        "the lineage no longer describes the tape"
    );
    let r = m.focused().unwrap();
    let chip = rustyecon_gui::vm::toolbar::build(&r.store, r.origin, m.ledger_changed(hand));
    assert!(chip.health.ledger_changed);
    let files: BTreeMap<String, String> = m.export_files(hand).unwrap().into_iter().collect();
    let manifest = GuiManifest::from_ron(&files[MANIFEST]).unwrap();
    assert!(manifest.ledger_changed);
    assert!(files[SERIES].contains("\n# ledger changed: yes\n"));
    let files: BTreeMap<String, String> = m.export_files(branch).unwrap().into_iter().collect();
    assert!(files[SERIES].contains("\n# ledger changed: no\n"));
}

#[test]
fn removing_the_last_use_offers_remove_param() {
    // §5.1 item 4: a removal that leaves a param unreferenced does not load (ENGINE §2.6).
    // materialise names every such param, and the editor offers a RemoveParam for each, with a
    // note of its own; accepted, the branch is made with both edits.
    let gate = tape_of(GATE);
    let cut = te(EditOp::RemoveEvent(key("mine.cut")), "no cut");
    assert_eq!(
        materialise(&gate, std::slice::from_ref(&cut), today()).err(),
        Some(EditError::Orphans {
            params: vec![key("mine.capacity.cut")]
        })
    );
    // Two removals, two orphans.
    let both = [
        cut.clone(),
        te(EditOp::RemoveEvent(key("mine.restored")), "no restoring"),
    ];
    let Err(EditError::Orphans { params }) = materialise(&gate, &both, today()) else {
        panic!("two orphans");
    };
    let got: BTreeSet<Key> = params.into_iter().collect();
    assert_eq!(
        got,
        [key("mine.capacity.base"), key("mine.capacity.cut")]
            .into_iter()
            .collect()
    );
    // A removal that orphans nothing is offered nothing.
    assert!(materialise(
        &gate,
        &[te(EditOp::RemoveEvent(key("oven.opens")), "no oven")],
        today()
    )
    .is_ok());
    // A param still in use cannot go: the load error names it and the edit.
    let e = materialise(
        &gate,
        &[te(EditOp::RemoveParam(key("mill.spend")), "no spend")],
        today(),
    );
    let Some(EditError::Load { error, edit }) = e.err() else {
        panic!("a load error");
    };
    assert_eq!(edit, Some(0), "{error}");
    // Through the model: Apply offers the RemoveParam; a note is required; accepted, the
    // branch has both edits, each with its note, and the removed entries in its lineage.
    let mut m = Model::default();
    let mut h = SyncHost::new(64);
    h.act(&mut m, Intent::Today(today()));
    h.act(
        &mut m,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    h.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::RemoveEvent,
            key: "mine.cut".to_string(),
            note: "no cut".to_string(),
            ..Form::default()
        }),
    );
    h.act(&mut m, Intent::Apply);
    assert_eq!(m.runs().count(), 1, "no branch yet");
    assert_eq!(m.editor().offer, [key("mine.capacity.cut")]);
    assert_eq!(
        m.editor().error.as_deref(),
        Some(
            "the removal leaves mine.capacity.cut unreferenced, which does not load: remove \
             it too, with a note"
        )
    );
    h.act(
        &mut m,
        Intent::RemoveOrphans {
            note: " ".to_string(),
        },
    );
    assert_eq!(m.runs().count(), 1, "a RemoveParam needs its own note");
    assert_eq!(m.editor().offer, [key("mine.capacity.cut")]);
    h.act(
        &mut m,
        Intent::RemoveOrphans {
            note: "its value goes with it".to_string(),
        },
    );
    assert_eq!(m.runs().count(), 2);
    let b = m.focused().unwrap();
    assert_eq!(b.parent, Some(RunId(0)));
    let ops: Vec<(String, String)> = b
        .applied
        .iter()
        .map(|a| (a.edit.op.to_string(), a.edit.note.clone()))
        .collect();
    assert_eq!(
        ops,
        [
            ("RemoveEvent mine.cut".to_string(), "no cut".to_string()),
            (
                "RemoveParam mine.capacity.cut".to_string(),
                "its value goes with it".to_string()
            )
        ]
    );
    assert!(matches!(&b.applied[1].replaced, Some(Replaced::Param(p)) if p.value == 26.0));
    assert!(m.editor().staged.is_empty() && m.editor().offer.is_empty());
}

#[test]
fn minted_keys_never_collide() {
    // §5.1 item 2, E8: a minted key, gui.<serial>.<n>, is new to the whole run tree: siblings
    // and ancestors included, the staged edits too, and after a branch removes the entry that
    // held it. A typed key that the tree has is refused.
    let mut m = Model::default();
    let mut h = SyncHost::new(64);
    h.act(&mut m, Intent::Today(today()));
    h.act(
        &mut m,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    h.act(&mut m, Intent::Run { until: Some(600) });
    let base = m.focus().unwrap();
    let add = |k: &Key| Form {
        kind: OpKind::AddEvent,
        key: k.to_string(),
        date: "1765-06-01".to_string(),
        act: "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")".to_string(),
        note: "restore early".to_string(),
        ..Form::default()
    };
    // A child.
    let k1 = m.mint_key().unwrap();
    assert_eq!(k1.as_str(), "gui.1.1");
    h.act(&mut m, Intent::Stage(add(&k1)));
    // Staged, its key is taken: the next mint differs.
    assert_eq!(m.mint_key().unwrap().as_str(), "gui.1.2");
    h.act(&mut m, Intent::Apply);
    let child = m.focus().unwrap();
    assert_ne!(child, base);
    // A sibling, from the base: its key is new to the tree, the child's included.
    h.act(&mut m, Intent::Focus(base));
    let k2 = m.mint_key().unwrap();
    assert_ne!(k2, k1);
    h.act(&mut m, Intent::Stage(add(&k1)));
    assert_eq!(
        m.editor().error.as_deref(),
        Some("refused: the key gui.1.1 is taken in this run tree; a new entry needs a new key (mint one)")
    );
    h.act(&mut m, Intent::Stage(add(&k2)));
    h.act(&mut m, Intent::Apply);
    let sibling = m.focus().unwrap();
    // A grandchild that removes the child's event: the key it held is still the tree's.
    h.act(&mut m, Intent::Focus(child));
    h.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::RemoveEvent,
            key: k1.to_string(),
            note: "undo".to_string(),
            ..Form::default()
        }),
    );
    h.act(&mut m, Intent::Apply);
    let grandchild = m.focus().unwrap();
    assert!(!m
        .run(grandchild)
        .unwrap()
        .tape
        .events
        .iter()
        .any(|e| e.key == k1));
    let k3 = m.mint_key().unwrap();
    assert!(k3 != k1 && k3 != k2, "{k3}");
    h.act(&mut m, Intent::Stage(add(&k1)));
    assert!(m
        .editor()
        .error
        .as_deref()
        .is_some_and(|e| e.contains("gui.1.1 is taken")));
    // Every key minted is new to every tape of the tree.
    let tree = m.tree(grandchild);
    assert_eq!(tree.len(), 4, "{tree:?}");
    for r in [base, child, sibling, grandchild] {
        assert!(tree.contains(&r));
    }
    // A key of a closed run is not minted again.
    h.act(&mut m, Intent::Close(sibling));
    h.act(&mut m, Intent::Focus(base));
    assert!(m.taken_keys(base).contains(k2.as_str()));
    // Another session mints under its own serial.
    let mut s = m.session.clone();
    s.serial = 5;
    let mut other = Model::with_session(s);
    h.act(
        &mut other,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    assert_eq!(other.mint_key().unwrap().as_str(), "gui.5.1");
    assert_eq!(keys::mint(5, &BTreeSet::new()).as_str(), "gui.5.1");
}

#[test]
fn saved_tape_carries_its_lineage() {
    // §5.1 item 6: "Save tape as" writes the canonical tape, which the cli runs unchanged, and
    // <name>.lineage.ron beside it: the nearest ancestor on disk by tape_hash and path, and
    // every edit since it, in order, each with its note, its date and what it replaced. For a
    // child, and for a grandchild of an unsaved child. Once saved, a tape is its children's
    // nearest ancestor. The base file is never overwritten, and an export carries the lineage.
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.ron");
    std::fs::write(&base, GATE).unwrap();
    let base = base.display().to_string();
    let base_hash = certify::tape_hash(&tape_of(GATE));
    let mut m = Model::default();
    let mut h = SyncHost::new(64);
    h.act(&mut m, Intent::Today(today()));
    h.act(&mut m, Intent::Open(base.clone()));
    h.act(&mut m, Intent::Run { until: Some(900) });
    let root = m.focus().unwrap();
    let stage = |h: &mut SyncHost, m: &mut Model, f: Form| {
        h.act(m, Intent::Stage(f));
        assert_eq!(m.editor().error, None);
    };
    // A child, saved.
    stage(
        &mut h,
        &mut m,
        Form {
            kind: OpKind::AddEvent,
            key: "gui.1.1".to_string(),
            date: "1765-06-01".to_string(),
            act: "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")".to_string(),
            note: "restore early".to_string(),
            ..Form::default()
        },
    );
    h.act(&mut m, Intent::Apply);
    let child = m.focus().unwrap();
    let child_path = dir.path().join("child.ron").display().to_string();
    h.act(&mut m, Intent::SaveTape(child_path.clone()));
    let saved = std::fs::read_to_string(&child_path).expect("the tape is saved");
    let child_tape = m.run(child).unwrap().tape.clone();
    assert_eq!(saved, child_tape.to_ron(), "the canonical text");
    assert_eq!(Tape::from_ron(&saved).unwrap(), child_tape);
    assert!(
        Sim::new(&Tape::from_ron(&saved).unwrap()).is_ok(),
        "the cli runs it"
    );
    let l = Lineage::from_ron(&std::fs::read_to_string(lineage_path(&child_path)).unwrap())
        .expect("the lineage reads");
    assert_eq!(l.parent.path, base);
    assert_eq!(l.parent.tape_hash.0, base_hash);
    assert_eq!(l.tape_hash.0, certify::tape_hash(&child_tape));
    assert_eq!(l.edits.len(), 1);
    assert_eq!(l.edits[0].edit.note, "restore early");
    assert_eq!(l.edits[0].on, today());
    assert_eq!(l.edits[0].replaced, None);
    assert_eq!(
        m.run(child).unwrap().path.as_deref(),
        Some(child_path.as_str())
    );
    // A grandchild of an unsaved child: base → spend (unsaved) → no oven, saved. Its lineage
    // names the base file and lists both branches' edits in order, with what each replaced.
    h.act(&mut m, Intent::Focus(root));
    stage(
        &mut h,
        &mut m,
        Form {
            kind: OpKind::SetGenesisParam,
            key: "mill.spend".to_string(),
            value: "12".to_string(),
            note: "spend more".to_string(),
            ..Form::default()
        },
    );
    h.act(&mut m, Intent::Apply);
    let spend = m.focus().unwrap();
    assert!(m.run(spend).unwrap().rerun.is_some(), "a world edit reruns");
    h.act(&mut m, Intent::Run { until: Some(700) });
    stage(
        &mut h,
        &mut m,
        Form {
            kind: OpKind::RemoveEvent,
            key: "oven.opens".to_string(),
            note: "no oven".to_string(),
            ..Form::default()
        },
    );
    h.act(&mut m, Intent::Apply);
    let grand = m.focus().unwrap();
    assert_eq!(m.run(grand).unwrap().parent, Some(spend));
    let grand_path = dir.path().join("grand.ron").display().to_string();
    h.act(&mut m, Intent::SaveTape(grand_path.clone()));
    let l =
        Lineage::from_ron(&std::fs::read_to_string(lineage_path(&grand_path)).unwrap()).unwrap();
    assert_eq!(
        (l.parent.path.as_str(), l.parent.tape_hash.0),
        (base.as_str(), base_hash)
    );
    let notes: Vec<&str> = l.edits.iter().map(|a| a.edit.note.as_str()).collect();
    assert_eq!(notes, ["spend more", "no oven"]);
    assert_eq!(
        l.edits[0].replaced,
        Some(Replaced::Value {
            value: 10.4,
            basis: Basis::Assumed("gate world".to_string())
        })
    );
    let oven = tape_of(GATE)
        .events
        .into_iter()
        .find(|e| e.key.as_str() == "oven.opens")
        .unwrap();
    assert_eq!(l.edits[1].replaced, Some(Replaced::Event(oven)));
    assert_eq!(l.lines().len(), 3);
    // A child of the saved child: its nearest ancestor on disk is the saved child.
    h.act(&mut m, Intent::Focus(child));
    stage(
        &mut h,
        &mut m,
        Form {
            kind: OpKind::RemoveEvent,
            key: "oven.opens".to_string(),
            note: "and no oven".to_string(),
            ..Form::default()
        },
    );
    h.act(&mut m, Intent::Apply);
    let l = m.focused().unwrap().lineage.clone().unwrap();
    assert_eq!(l.parent.path, child_path);
    assert_eq!(l.parent.tape_hash.0, certify::tape_hash(&child_tape));
    assert_eq!(l.edits.len(), 1);
    // The base file is never overwritten, nor is any file.
    let before = std::fs::read(&base).unwrap();
    h.act(&mut m, Intent::SaveTape(base.clone()));
    assert!(m
        .editor()
        .error
        .as_deref()
        .is_some_and(|e| e.contains("never overwritten")));
    h.act(&mut m, Intent::Focus(grand));
    h.act(&mut m, Intent::SaveTape(child_path.clone()));
    assert!(m
        .editor()
        .error
        .as_deref()
        .is_some_and(|e| e.contains("never overwritten")));
    let elsewhere = dir.path().join("other.ron");
    std::fs::write(&elsewhere, "not mine").unwrap();
    h.act(&mut m, Intent::SaveTape(elsewhere.display().to_string()));
    assert!(m
        .editor()
        .error
        .as_deref()
        .is_some_and(|e| e.contains("no file is written over")));
    assert_eq!(std::fs::read(&base).unwrap(), before);
    assert_eq!(std::fs::read_to_string(&elsewhere).unwrap(), "not mine");
    // Reopened, the saved grandchild is an experiment with its lineage, and an export of it
    // carries the lineage and the ancestor's tape.
    h.act(&mut m, Intent::Open(grand_path.clone()));
    let reopened = m.focused().unwrap();
    assert_eq!(reopened.origin, Origin::Experiment);
    assert_eq!(reopened.lineage.as_ref().map(|l| l.edits.len()), Some(2));
    let out = dir.path().join("export");
    h.act(&mut m, Intent::Focus(grand));
    h.act(&mut m, Intent::Export(out.display().to_string()));
    let lineage = std::fs::read_to_string(out.join(LINEAGE)).expect("the lineage is exported");
    assert_eq!(Lineage::from_ron(&lineage).unwrap().edits.len(), 2);
    let ancestor = std::fs::read_to_string(out.join("ancestor.ron")).unwrap();
    assert_eq!(
        certify::tape_hash(&Tape::from_ron(&ancestor).unwrap()),
        base_hash
    );
    let manifest =
        GuiManifest::from_ron(&std::fs::read_to_string(out.join(MANIFEST)).unwrap()).unwrap();
    assert_eq!(manifest.lineage.as_deref(), Some(LINEAGE));
    assert!(matches!(
        m.focused().unwrap().store.status(),
        RunStatus::Paused { .. }
    ));
}

#[test]
fn the_form_checks_first() {
    // §5.1 item 2: an empty note, a malformed key, a malformed date, an act that does not read
    // (with the parser's line and column), a bad value and a bad unit are refused before
    // anything is applied.
    let gate = tape_of(GATE);
    let good = Form {
        kind: OpKind::AddEvent,
        key: "gui.1.1".to_string(),
        date: "1765-06-01".to_string(),
        act: "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")".to_string(),
        note: "restore early".to_string(),
        ..Form::default()
    };
    let none = BTreeSet::new();
    assert!(form::parse(&good, &gate, &none).is_ok());
    let with = |f: fn(&mut Form)| {
        let mut x = good.clone();
        f(&mut x);
        form::parse(&x, &gate, &none).err()
    };
    assert_eq!(with(|f| f.note = "  ".into()), Some(FormError::Note));
    assert_eq!(
        with(|f| f.key = "Mine Cut".into()),
        Some(FormError::Key(
            "the key \"Mine Cut\" must match [a-z0-9_.-]+".to_string()
        ))
    );
    assert!(matches!(
        with(|f| f.date = "1765-13-45".into()),
        Some(FormError::Date { field: "date", .. })
    ));
    let e =
        with(|f| f.act = "SetParam(param: \"mine.capacity\" to: \"mine.capacity.base\")".into());
    let Some(FormError::Act { line, col, .. }) = e else {
        panic!("{e:?}");
    };
    assert_eq!((line, col), (1, 33), "at `to`, where a comma should be");
    assert_eq!(
        with(|f| f.key = "mine.cut".into()).map(|_| ()),
        None,
        "taken is the model's to say"
    );
    let taken: BTreeSet<String> = ["mine.cut".to_string()].into_iter().collect();
    let mut x = good.clone();
    x.key = "mine.cut".into();
    assert_eq!(
        form::parse(&x, &gate, &taken),
        Err(FormError::Taken("mine.cut".to_string()))
    );
    let param = Form {
        kind: OpKind::AddParam,
        key: "gui.1.2".to_string(),
        value: "inf".to_string(),
        unit: "FlowPerYear".to_string(),
        note: "n".to_string(),
        ..Form::default()
    };
    assert!(matches!(
        form::parse(&param, &gate, &none),
        Err(FormError::Value(_))
    ));
    let mut p = param.clone();
    p.value = "40".into();
    p.unit = "Furlongs".into();
    assert_eq!(
        form::parse(&p, &gate, &none),
        Err(FormError::Unit("Furlongs".to_string()))
    );
    p.unit = "FlowPerYear".into();
    assert!(form::parse(&p, &gate, &none).is_ok());
    // Through the model, an act that does not read is kept for the raw pane.
    let mut m = Model::default();
    let mut h = SyncHost::new(64);
    h.act(
        &mut m,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    let mut bad = good.clone();
    bad.act = "SetParam(param: \"mine.capacity\" to: \"mine.capacity.base\")".into();
    h.act(&mut m, Intent::Stage(bad.clone()));
    assert_eq!(
        m.editor().raw,
        Some(RawError {
            text: bad.act.clone(),
            line: 1,
            col: 33,
            message: "Expected comma".to_string(),
        })
    );
    // Apply needs a date to stamp with.
    h.act(&mut m, Intent::Stage(good.clone()));
    h.act(&mut m, Intent::Apply);
    assert_eq!(m.runs().count(), 1);
    assert_eq!(
        m.editor().error.as_deref(),
        Some("no date to stamp the experiment with")
    );
}
