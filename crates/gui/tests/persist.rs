//! `session.ron` and `layout.ron` (U8): what a session keeps round-trips, a file this build does
//! not read is refused and set aside rather than overwritten, and the host saves the session
//! whenever it changes.

mod common;

use common::build;
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Host;
use rustyecon_gui::model::{Base, Intent, Model, Session};
use rustyecon_gui::platform::{Files, LAYOUT, SESSION};
use rustyecon_gui::run::{At, Breakpoint, Entity, HolderKey, Measure, SeriesKey};
use rustyecon_gui::ui::layout::{self, Pane};
use std::sync::Arc;

fn key(s: &str) -> Key {
    Key::new(s).unwrap()
}

fn full_session() -> Session {
    Session {
        bases: vec![Base {
            path: "tapes/gate.ron".to_string(),
            tape_hash: certify::Hex(0x5406_6d05_3474_846b),
        }],
        plots: vec![
            SeriesKey {
                measure: Measure::Price,
                at: At::Market {
                    node: key("town"),
                    good: key("bread"),
                },
            },
            SeriesKey {
                measure: Measure::Filled,
                at: At::Class {
                    node: key("village"),
                    good: key("bread"),
                    class: key("pensioners"),
                    side: SideTag::Buy,
                },
            },
            SeriesKey {
                measure: Measure::Held,
                at: At::Holding {
                    holder: HolderKey::Escrow {
                        node: key("town"),
                        good: key("grain"),
                    },
                    good: key("coin"),
                },
            },
            SeriesKey {
                measure: Measure::Declared,
                at: At::Line {
                    good: key("bread"),
                    prov: Provenance::Spoilage,
                },
            },
            SeriesKey {
                measure: Measure::RunMargin,
                at: At::World,
            },
        ],
        pins: vec![
            Entity::Market {
                node: key("town"),
                good: key("bread"),
            },
            Entity::Param(key("mine.capacity")),
        ],
        breakpoints: vec![Breakpoint::OnError],
        speed: Some(520),
        ..Session::default()
    }
}

#[test]
fn a_session_round_trips_and_refuses_what_it_does_not_know() {
    let s = full_session();
    let text = s.to_ron();
    assert!(text.contains("tape_hash: \"0x54066d053474846b\""), "{text}");
    assert_eq!(Session::from_ron(&text), Ok(s.clone()));
    assert_eq!(
        Session::from_ron(&Session::default().to_ron()),
        Ok(Session::default())
    );
    // An unknown field, a missing one, and another format are refused.
    let unknown = text.replacen("speed:", "colour: 1, speed:", 1);
    assert!(Session::from_ron(&unknown).is_err());
    let missing = text.replacen("speed: Some(520),", "", 1);
    assert!(Session::from_ron(&missing).is_err());
    let serial = text.replacen("serial: 1,", "", 1);
    assert!(
        Session::from_ron(&serial).is_err(),
        "the serial is required"
    );
    let newer = text.replacen("format: 2,", "format: 3,", 1);
    assert_eq!(
        Session::from_ron(&newer),
        Err("session format 3; this build reads 2".to_string())
    );
    // G0.1's format, with no serial, is refused as a format.
    let older = serial.replacen("format: 2,", "format: 1,", 1);
    assert_eq!(
        Session::from_ron(&older),
        Err("session format 1; this build reads 2".to_string())
    );
}

#[test]
fn a_layout_round_trips_and_gains_a_pane_it_lacks() {
    let tree = layout::default_tree();
    for p in Pane::ALL {
        assert!(tree.tiles.find_pane(&p).is_some(), "{p:?}");
    }
    let text = layout::to_ron(&tree);
    assert!(layout::from_ron(&text).expect("it reads") == tree);
    // A layout saved before a pane existed (G0.1's had no editor and no compare) gains it.
    let mut short = tree.clone();
    for p in [Pane::Log, Pane::Editor, Pane::Compare] {
        let id = short.tiles.find_pane(&p).unwrap();
        short.remove_recursively(id);
    }
    let back = layout::from_ron(&layout::to_ron(&short)).expect("it reads");
    for p in Pane::ALL {
        assert!(back.tiles.find_pane(&p).is_some(), "{p:?}");
    }
    assert!(layout::from_ron("not a layout").is_err());
}

#[test]
fn files_write_whole_and_set_aside_what_does_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let files = Files::at(dir.path().join("gui"));
    assert!(files.read_session().is_none() && files.read_layout().is_none());
    files.write_session("one").unwrap();
    files.write_session("two").unwrap();
    assert_eq!(files.read_session(), Some(Ok("two".to_string())));
    assert!(!files.dir().join("session.ron.tmp").exists());
    files.write_layout("tiles").unwrap();
    assert_eq!(files.read_layout(), Some(Ok("tiles".to_string())));
    let kept = files.set_aside(SESSION).unwrap();
    assert_eq!(std::fs::read_to_string(kept).unwrap(), "two");
    assert!(files.read_session().is_none());
    assert!(
        files.set_aside(SESSION).is_err(),
        "nothing left to set aside"
    );
    assert!(files.layout_path().ends_with(LAYOUT));
}

#[test]
fn the_host_saves_the_session_when_it_changes_and_keeps_a_bad_one() {
    let dir = tempfile::tempdir().unwrap();
    let files = Files::at(dir.path());
    std::fs::write(files.session_path(), "Session(format: 1)").unwrap();
    let mut m = Model::default();
    let mut host = Host::new(build(), Arc::new(|| {}), Some(files.clone()));
    let text = files.read_session().unwrap();
    host.act(&mut m, Intent::SessionRead(text));
    // The unreadable file is kept aside, and nothing was written over it.
    let aside = dir.path().join("session.ron.unreadable");
    assert_eq!(
        std::fs::read_to_string(&aside).unwrap(),
        "Session(format: 1)"
    );
    assert!(files.read_session().is_none());
    assert!(m
        .log()
        .iter()
        .any(|l| l.text.contains("session.ron.unreadable")));
    // A change is saved at once, and reads back.
    host.act(&mut m, Intent::Speed(Some(52)));
    let saved = Session::from_ron(&files.read_session().unwrap().unwrap()).unwrap();
    assert_eq!(saved.speed, Some(52));
    assert_eq!(saved, m.session);
    // A host with no files saves nothing, and says nothing.
    let mut quiet = Host::new(build(), Arc::new(|| {}), None);
    let before = m.log().len();
    quiet.act(&mut m, Intent::Speed(None));
    assert_eq!(m.log().len(), before);
}
