//! The editor's rules (docs/GUI.md §5.1; D3, U2, E8): what `materialise` stamps and refuses,
//! the offer of a `RemoveParam`, minted keys, the lineage a saved tape carries, and "ledger
//! changed". Branches run on this thread through real Runners (`common::sync`), so each step is
//! deterministic; `tests/branch.rs` drives the same through `ThreadDriver`.

mod common;

use common::sync::SyncHost;
use common::{key, tape_of, today, GATE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::edit::export::{GuiManifest, ANCESTOR, LINEAGE, MANIFEST, SERIES};
use rustyecon_gui::edit::{
    self, form, keys, lineage_path, materialise, stamp, Act, Basis, EditError, EditOp, Form,
    FormError, Lineage, OpKind, Replaced, TapeEdit, Unit,
};
use rustyecon_gui::model::{Intent, Model, RawError, Session};
use rustyecon_gui::run::{LedgerCheck, Origin, RunId, RunStatus, EXPERIMENT_MARKER};
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
    // The removals of the other kinds, through the model: RemoveRecurring(pension), and the
    // RemoveParam(pension.period) it offers. Neither stamps anything, the name carries the
    // marker, and the lineage keeps the entry and the param each removed.
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
            kind: OpKind::RemoveRecurring,
            key: "pension".to_string(),
            note: "no pension".to_string(),
            ..Form::default()
        }),
    );
    h.act(&mut m, Intent::Apply);
    assert_eq!(m.editor().offer, [key("pension.period")]);
    h.act(
        &mut m,
        Intent::RemoveOrphans {
            note: "its period goes with it".to_string(),
        },
    );
    let b = m.focused().unwrap();
    assert_eq!(b.parent, Some(RunId(0)));
    let mut kept = bases(&gate);
    kept.remove("recurring[pension]");
    assert_eq!(
        bases(&b.tape).len(),
        kept.len() - 1,
        "pension.period is gone"
    );
    kept.remove("params[pension.period]");
    assert_eq!(bases(&b.tape), kept, "nothing is stamped");
    assert_eq!(
        b.tape.header.name,
        format!("gate [GUI experiment {}]", today())
    );
    assert_eq!(b.tape.to_ron().matches(EXPERIMENT_MARKER).count(), 1);
    let l = b.lineage.as_ref().expect("a lineage");
    let pension = gate.recurring.iter().find(|r| r.key.as_str() == "pension");
    let period = gate
        .params
        .iter()
        .find(|p| p.key.as_str() == "pension.period");
    let replaced: Vec<_> = l.edits.iter().map(|a| a.replaced.clone()).collect();
    assert_eq!(
        replaced,
        [
            Some(Replaced::Recurring(pension.unwrap().clone())),
            Some(Replaced::Param(period.unwrap().clone())),
        ]
    );
    let notes: Vec<&str> = l.edits.iter().map(|a| a.edit.note.as_str()).collect();
    assert_eq!(notes, ["no pension", "its period goes with it"]);
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
    assert_eq!(m.ledger(branch), LedgerCheck::Same);
    assert_eq!(m.ledger(base_run), LedgerCheck::NoParent);
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
    let by_hand = dir.path().join("by-hand.ron").display().to_string();
    std::fs::write(&by_hand, t.to_ron()).unwrap();
    std::fs::copy(lineage_path(&saved), lineage_path(&by_hand)).unwrap();
    let hand_hash = certify::tape_hash(&t);
    h.act(&mut m, Intent::Open(by_hand.clone()));
    let hand = m.focus().unwrap();
    assert_eq!(m.focused().unwrap().origin, Origin::Experiment);
    assert_eq!(
        m.ledger(hand),
        LedgerCheck::Changed,
        "the hand-edited tolerance shows"
    );
    assert_eq!(m.ledger(branch), LedgerCheck::Same);
    assert_eq!(m.ledger(base_run), LedgerCheck::NoParent);
    assert!(
        m.log()
            .iter()
            .any(|l| l.text.contains("the tape was edited after it was saved")),
        "the lineage no longer describes the tape"
    );
    let r = m.focused().unwrap();
    let chip = rustyecon_gui::vm::toolbar::build(&r.store, r.origin, m.ledger(hand));
    assert_eq!(chip.health.ledger, LedgerCheck::Changed);
    let files: BTreeMap<String, String> = m.export_files(hand).unwrap().into_iter().collect();
    let manifest = GuiManifest::from_ron(&files[MANIFEST]).unwrap();
    assert_eq!(manifest.ledger_changed, Some(true));
    assert!(files[SERIES].contains("\n# ledger changed: yes\n"));
    // The lineage describes the tape as saved, not as edited by hand: the export says so.
    let l = Lineage::from_ron(&files[LINEAGE]).unwrap();
    assert_ne!(l.tape_hash.0, hand_hash);
    assert!(manifest.lineage_stale);
    assert!(files[SERIES].contains(&format!(
        "\n# lineage tape.lineage.ron (describes tape_hash {}, not this tape: the tape was \
         edited by hand after it was saved, and those edits are in no lineage)\n",
        l.tape_hash
    )));
    // Saved again, its lineage goes beside it (without it, a tape whose only mark was its
    // lineage would reopen as a run), and the log says it describes another tape.
    let again = dir.path().join("again.ron").display().to_string();
    h.act(&mut m, Intent::SaveTape(again.clone()));
    assert!(std::path::Path::new(&lineage_path(&again)).exists());
    assert!(m.log().iter().any(|e| e.text
        == format!(
            "the lineage saved beside {again} describes tape_hash {}, not this tape's {}: the \
             tape was edited by hand after it was saved, and those edits are in no lineage",
            l.tape_hash,
            certify::Hex(hand_hash)
        )));
    let files: BTreeMap<String, String> = m.export_files(branch).unwrap().into_iter().collect();
    assert!(files[SERIES].contains("\n# ledger changed: no\n"));
    assert!(
        !GuiManifest::from_ron(&files[MANIFEST])
            .unwrap()
            .lineage_stale
    );
    assert!(files[SERIES].contains("\n# lineage tape.lineage.ron\n"));
    let files: BTreeMap<String, String> = m.export_files(base_run).unwrap().into_iter().collect();
    assert!(files[SERIES].contains("\n# ledger changed: no (no parent: a tape no GUI edit made)\n"));
    assert_eq!(
        GuiManifest::from_ron(&files[MANIFEST])
            .unwrap()
            .ledger_changed,
        Some(false)
    );
    // A fresh session opens only the hand-edited tape. Its lineage's ancestor, the base file,
    // is read from its path, and the tolerances differ.
    let mut fresh = Model::default();
    let mut h2 = SyncHost::new(64);
    h2.act(&mut fresh, Intent::Open(by_hand.clone()));
    let only = fresh.focus().unwrap();
    assert_eq!(fresh.runs().count(), 1, "the ancestor is read, not opened");
    assert_eq!(fresh.ledger(only), LedgerCheck::Changed);
    assert!(fresh.run(only).unwrap().ancestor().is_some());
    // Without the base file, the check cannot be made: the chip and the export say so, and
    // never say "no".
    let moved = dir.path().join("moved.ron");
    std::fs::rename(&base, &moved).unwrap();
    let mut fresh = Model::default();
    let mut h2 = SyncHost::new(64);
    h2.act(&mut fresh, Intent::Open(by_hand.clone()));
    let only = fresh.focus().unwrap();
    let LedgerCheck::Unknown(why) = fresh.ledger(only) else {
        panic!("{:?}", fresh.ledger(only));
    };
    assert!(
        why.starts_with(&format!("its lineage's ancestor {base} cannot be read: ")),
        "{why}"
    );
    let r = fresh.focused().unwrap();
    let chip = rustyecon_gui::vm::toolbar::build(&r.store, r.origin, fresh.ledger(only));
    assert_eq!(chip.health.ledger, LedgerCheck::Unknown(why.clone()));
    let files: BTreeMap<String, String> = fresh.export_files(only).unwrap().into_iter().collect();
    assert_eq!(
        GuiManifest::from_ron(&files[MANIFEST])
            .unwrap()
            .ledger_changed,
        None
    );
    assert!(files[SERIES].contains(&format!("\n# ledger changed: unknown ({why})\n")));
    assert!(!files.contains_key(ANCESTOR));
    // A base file that changed on disk is not the ancestor the lineage names.
    std::fs::write(
        &base,
        GATE.replacen("name: \"gate\"", "name: \"gate, moved\"", 1),
    )
    .unwrap();
    let mut fresh = Model::default();
    let mut h2 = SyncHost::new(64);
    h2.act(&mut fresh, Intent::Open(by_hand.clone()));
    let only = fresh.focus().unwrap();
    assert!(
        matches!(fresh.ledger(only), LedgerCheck::Unknown(w) if w.ends_with("it changed on disk")),
        "{:?}",
        fresh.ledger(only)
    );
    // A tape that carries the marker and has no lineage has no parent this session can name.
    let lone = dir.path().join("lone.ron").display().to_string();
    std::fs::write(&lone, t.to_ron()).unwrap();
    h2.act(&mut fresh, Intent::Open(lone));
    let lone = fresh.focus().unwrap();
    assert_eq!(fresh.focused().unwrap().origin, Origin::Experiment);
    assert!(matches!(fresh.ledger(lone), LedgerCheck::Unknown(_)));
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
    // §5.1 item 2, E8: a minted key, gui.<serial>.<n>, is new to every tape of the run tree:
    // siblings and ancestors included, the staged edits too, and after a branch removes the
    // entry that held it. As fixed after G0.2's verification, it is new to every tape open in
    // the session, so to a saved branch reopened as its own root, to a second root of one
    // file and to the siblings of a closed parent, and to keys of every kind (params and
    // recurring entries too); its n never falls back within a session; and a set-aside
    // session's serial is carried forward. A typed key that a tape of the session has is
    // refused, by the form and again by Apply.
    let dir = tempfile::tempdir().unwrap();
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
    let taken = |k: &Key| {
        format!(
            "refused: the key {k} is taken by a tape of this session; a new entry needs a new \
             key (mint one)"
        )
    };
    // Every key of every open tape: no minted key may be among them.
    let all_keys = |m: &Model| -> BTreeSet<String> {
        m.runs().flat_map(|r| keys::tape_keys(&r.tape)).collect()
    };
    // A child.
    let k1 = m.mint_key().unwrap();
    assert_eq!(k1.as_str(), "gui.1.1");
    h.act(&mut m, Intent::Stage(add(&k1)));
    // Staged, its key is taken: the next mint differs, and typing it again is refused.
    assert_eq!(m.mint_key().unwrap().as_str(), "gui.1.2");
    h.act(&mut m, Intent::Stage(add(&k1)));
    assert_eq!(m.editor().error, Some(taken(&k1)));
    assert_eq!(m.editor().staged.len(), 1);
    h.act(&mut m, Intent::Apply);
    let child = m.focus().unwrap();
    assert_ne!(child, base);
    // A sibling, from the base: its key is new to the tree, the child's included.
    h.act(&mut m, Intent::Focus(base));
    let k2 = m.mint_key().unwrap();
    assert_ne!(k2, k1);
    h.act(&mut m, Intent::Stage(add(&k1)));
    assert_eq!(m.editor().error, Some(taken(&k1)));
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
    assert_eq!(m.editor().error, Some(taken(&k1)));
    let tree = m.tree(grandchild);
    assert_eq!(tree.len(), 4, "{tree:?}");
    for r in [base, child, sibling, grandchild] {
        assert!(tree.contains(&r));
    }
    // Keys of every kind: a param and a recurring entry, each with a minted key, in two more
    // siblings. The next mint skips both, and an event may not take either key.
    h.act(&mut m, Intent::Focus(base));
    let kp = m.mint_key().unwrap();
    h.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::AddParam,
            key: kp.to_string(),
            value: "40".to_string(),
            unit: "FlowPerYear".to_string(),
            note: "a new capacity".to_string(),
            ..Form::default()
        }),
    );
    let ke = m.mint_key().unwrap();
    h.act(
        &mut m,
        Intent::Stage(Form {
            act: format!("SetParam(param: \"mine.capacity\", to: \"{kp}\")"),
            ..add(&ke)
        }),
    );
    assert_eq!(m.editor().error, None);
    h.act(&mut m, Intent::Apply);
    let with_param = m.focus().unwrap();
    assert!(m
        .run(with_param)
        .unwrap()
        .tape
        .params
        .iter()
        .any(|p| p.key == kp));
    h.act(&mut m, Intent::Focus(base));
    let kr = m.mint_key().unwrap();
    h.act(
        &mut m,
        Intent::Stage(Form {
            kind: OpKind::AddRecurring,
            key: kr.to_string(),
            date: "1752-01-01".to_string(),
            every: "pension.period".to_string(),
            act: "Mint(holder: \"pensioners\", good: \"coin\", qty: 1.0)".to_string(),
            note: "a bonus".to_string(),
            ..Form::default()
        }),
    );
    assert_eq!(m.editor().error, None);
    h.act(&mut m, Intent::Apply);
    let with_recurring = m.focus().unwrap();
    assert!(m
        .run(with_recurring)
        .unwrap()
        .tape
        .recurring
        .iter()
        .any(|r| r.key == kr));
    h.act(&mut m, Intent::Focus(base));
    for k in [&kp, &kr] {
        assert!(m.taken_keys().contains(k.as_str()), "{k}");
        h.act(&mut m, Intent::Stage(add(k)));
        assert_eq!(m.editor().error, Some(taken(k)));
    }
    let next = m.mint_key().unwrap();
    assert!(!all_keys(&m).contains(next.as_str()), "{next}");
    // A fresh session whose tape holds a param and a recurring entry under its serial mints
    // past both, with nothing staged.
    let both = dir.path().join("both.ron").display().to_string();
    let mut t = m.run(with_param).unwrap().tape.clone();
    t.recurring = m.run(with_recurring).unwrap().tape.recurring.clone();
    std::fs::write(&both, t.to_ron()).unwrap();
    let mut fresh = Model::default();
    SyncHost::new(64).act(&mut fresh, Intent::Open(both));
    let fresh_key = fresh.mint_key().unwrap();
    for k in [&kp, &ke, &kr] {
        assert_ne!(&fresh_key, k);
    }
    assert!(
        !keys::tape_keys(&t).contains(fresh_key.as_str()),
        "{fresh_key}"
    );
    // A saved child reopened: it opens as its own root, and a branch of it mints a key new to
    // the sibling it was branched beside.
    h.act(&mut m, Intent::Focus(child));
    let c1 = dir.path().join("c1.ron").display().to_string();
    h.act(&mut m, Intent::SaveTape(c1.clone()));
    h.act(&mut m, Intent::Open(c1.clone()));
    let reopened = m.focus().unwrap();
    assert_eq!(m.run(reopened).unwrap().parent, None, "its own root");
    let kc = m.mint_key().unwrap();
    assert!(!all_keys(&m).contains(kc.as_str()), "{kc}");
    h.act(&mut m, Intent::Stage(add(&kc)));
    h.act(&mut m, Intent::Apply);
    let reopened_child = m.focus().unwrap();
    assert_eq!(m.run(reopened_child).unwrap().parent, Some(reopened));
    // The sibling's key may not be typed into a branch of the reopened child either.
    h.act(&mut m, Intent::Stage(add(&k2)));
    assert_eq!(m.editor().error, Some(taken(&k2)));
    // The same file opened twice: each root's branch gets its own key.
    h.act(
        &mut m,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    let second = m.focus().unwrap();
    h.act(&mut m, Intent::Run { until: Some(600) });
    let ks = m.mint_key().unwrap();
    assert!(!all_keys(&m).contains(ks.as_str()), "{ks}");
    h.act(&mut m, Intent::Stage(add(&k1)));
    assert_eq!(
        m.editor().error,
        Some(taken(&k1)),
        "the first root's child has it"
    );
    h.act(&mut m, Intent::Stage(add(&ks)));
    h.act(&mut m, Intent::Apply);
    assert_eq!(m.run(m.focus().unwrap()).unwrap().parent, Some(second));
    // A key typed on one run and staged, then found in a tape opened before Apply: Apply
    // refuses it.
    let kt = key("gui.9.9");
    h.act(&mut m, Intent::Focus(base));
    h.act(&mut m, Intent::Stage(add(&kt)));
    assert_eq!(m.editor().error, None);
    let holder = dir.path().join("holder.ron").display().to_string();
    let mut t = tape_of(GATE);
    let mut e = t.events[0].clone();
    e.key = kt.clone();
    t.events.push(e);
    std::fs::write(&holder, t.to_ron()).unwrap();
    h.act(&mut m, Intent::Open(holder));
    h.act(&mut m, Intent::Focus(base));
    let runs = m.runs().count();
    h.act(&mut m, Intent::Apply);
    assert_eq!(m.runs().count(), runs, "no branch");
    assert_eq!(
        m.editor().error.as_deref(),
        Some(
            "edit 1: the key gui.9.9 is taken; a new entry needs a key new to every tape of \
             this session"
        )
    );
    h.act(&mut m, Intent::Unstage(0));
    // Closed runs: a closed sibling's key is not minted or typed again, and closing a parent
    // leaves its children's siblings in the session.
    h.act(&mut m, Intent::Close(sibling));
    assert!(m.taken_keys().contains(k2.as_str()));
    h.act(&mut m, Intent::Stage(add(&k2)));
    assert_eq!(m.editor().error, Some(taken(&k2)));
    h.act(&mut m, Intent::Close(base));
    h.act(&mut m, Intent::Focus(with_param));
    let kx = m.mint_key().unwrap();
    assert!(!all_keys(&m).contains(kx.as_str()), "{kx}");
    h.act(&mut m, Intent::Stage(add(&k1)));
    assert_eq!(
        m.editor().error,
        Some(taken(&k1)),
        "the closed base's child has it"
    );
    // No n is minted twice in a session: a key staged and dropped is not minted again.
    h.act(&mut m, Intent::Stage(add(&kx)));
    assert_eq!(m.editor().staged.len(), 1);
    h.act(&mut m, Intent::Unstage(0));
    assert_ne!(m.mint_key().unwrap(), kx);
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
    assert_eq!(keys::mint(5, &BTreeSet::new(), 0).as_str(), "gui.5.1");
    // A session that does not read as a whole but whose serial does is set aside, and the
    // new session takes the serial after it.
    let mut later = Model::default();
    let old = Session {
        serial: 7,
        ..Session::default()
    }
    .to_ron()
    .replacen("format: 3,", "format: 4,", 1);
    h.act(&mut later, Intent::SessionRead(Ok(old)));
    assert_eq!(later.session.serial, 8);
    h.act(
        &mut later,
        Intent::TapeRead {
            path: "gate.ron".to_string(),
            text: Ok(GATE.to_string()),
            lineage: None,
        },
    );
    assert_eq!(later.mint_key().unwrap().as_str(), "gui.8.1");
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
    // Reopened, the saved grandchild is an experiment with its lineage, and an export of the
    // reopened run carries the lineage and the ancestor's tape: the base, which this session
    // holds.
    h.act(&mut m, Intent::Open(grand_path.clone()));
    let reopened = m.focused().unwrap();
    assert_eq!(reopened.parent, None, "its own root");
    assert_eq!(reopened.origin, Origin::Experiment);
    assert_eq!(reopened.lineage.as_ref().map(|l| l.edits.len()), Some(2));
    assert!(matches!(reopened.store.status(), RunStatus::Paused { .. }));
    let exported = |m: &mut Model, h: &mut SyncHost, name: &str| {
        let out = dir.path().join(name);
        h.act(m, Intent::Export(out.display().to_string()));
        let lineage = std::fs::read_to_string(out.join(LINEAGE)).expect("the lineage is exported");
        assert_eq!(Lineage::from_ron(&lineage).unwrap().edits.len(), 2);
        let ancestor = std::fs::read_to_string(out.join(ANCESTOR)).expect("the ancestor's tape");
        assert_eq!(
            certify::tape_hash(&Tape::from_ron(&ancestor).unwrap()),
            base_hash
        );
        let manifest =
            GuiManifest::from_ron(&std::fs::read_to_string(out.join(MANIFEST)).unwrap()).unwrap();
        assert_eq!(manifest.lineage.as_deref(), Some(LINEAGE));
        assert_eq!(manifest.origin, Origin::Experiment);
    };
    exported(&mut m, &mut h, "export");
    // So does the in-memory grandchild's.
    h.act(&mut m, Intent::Focus(grand));
    exported(&mut m, &mut h, "export-grand");
    // A fresh session that opens only the saved grandchild reads the base from the path its
    // lineage names, and exports it too.
    let mut fresh = Model::default();
    let mut h2 = SyncHost::new(64);
    h2.act(&mut fresh, Intent::Open(grand_path.clone()));
    assert_eq!(fresh.runs().count(), 1);
    exported(&mut fresh, &mut h2, "export-fresh");
    // U3's other clauses. The lineage alone marks a tape: a removal-only branch, whose only
    // marker is its name, saved, its name put back by hand and its lineage kept, reopens as
    // an experiment.
    h.act(&mut m, Intent::Focus(root));
    stage(
        &mut h,
        &mut m,
        Form {
            kind: OpKind::RemoveEvent,
            key: "oven.opens".to_string(),
            note: "only a removal".to_string(),
            ..Form::default()
        },
    );
    h.act(&mut m, Intent::Apply);
    let removal = m.focused().unwrap().tape.clone();
    assert_eq!(removal.to_ron().matches(EXPERIMENT_MARKER).count(), 1);
    let removal_path = dir.path().join("removal.ron").display().to_string();
    h.act(&mut m, Intent::SaveTape(removal_path.clone()));
    let mut unmarked = removal.clone();
    unmarked.header.name = "gate".to_string();
    assert_eq!(
        Origin::of(&unmarked),
        Origin::Run,
        "no marker is left in it"
    );
    let unmarked_path = dir.path().join("unmarked.ron").display().to_string();
    std::fs::write(&unmarked_path, unmarked.to_ron()).unwrap();
    std::fs::copy(lineage_path(&removal_path), lineage_path(&unmarked_path)).unwrap();
    h.act(&mut m, Intent::Open(unmarked_path.clone()));
    assert_eq!(m.focused().unwrap().origin, Origin::Experiment);
    // Without the lineage beside it, the same tape is a run.
    let bare_path = dir.path().join("bare.ron").display().to_string();
    std::fs::write(&bare_path, unmarked.to_ron()).unwrap();
    h.act(&mut m, Intent::Open(bare_path));
    assert_eq!(m.focused().unwrap().origin, Origin::Run);
    // A basis that carries the marker marks the tape, though its name does not.
    let mut by_basis = tape_of(GATE);
    by_basis.params[0].basis = stamp(today(), "by hand");
    assert_eq!(by_basis.header.name, "gate");
    assert_eq!(Origin::of(&by_basis), Origin::Experiment);
    let by_basis_path = dir.path().join("by-basis.ron").display().to_string();
    std::fs::write(&by_basis_path, by_basis.to_ron()).unwrap();
    h.act(&mut m, Intent::Open(by_basis_path));
    assert_eq!(m.focused().unwrap().origin, Origin::Experiment);
    // A tape with neither is a run.
    assert_eq!(Origin::of(&tape_of(GATE)), Origin::Run);
    assert_eq!(m.run(root).unwrap().origin, Origin::Run);
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
