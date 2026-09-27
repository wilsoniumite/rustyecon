//! The reducer as a state machine (docs/GUI.md §3.3, §8.1): what each intent does to the model
//! and which effects it asks for, driven by real Runners on this thread so every step is
//! deterministic.

mod common;

use common::{collecting, drain, edit, tape_of, GATE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::edit::{Form, OpKind};
use rustyecon_gui::model::{reduce, Cursor, Effect, Intent, Job, Model, Session};
use rustyecon_gui::run::{
    At, Breakpoint, Cmd, Entity, Measure, Obs, Origin, ResumeFrom, RunId, RunStatus, Runner,
    SeriesKey, EXPERIMENT_MARKER,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// A Runner and what its sink collected.
type Collected = (Runner, Arc<Mutex<Vec<Obs>>>);

/// A host on this thread: one Runner per run, advanced until idle after every command.
#[derive(Default)]
struct Sync {
    runners: BTreeMap<RunId, Collected>,
    sent: Vec<(RunId, String)>,
    saves: usize,
    slice: u32,
}

impl Sync {
    fn new(slice: u32) -> Sync {
        Sync {
            slice,
            ..Sync::default()
        }
    }

    fn act(&mut self, m: &mut Model, i: Intent) -> Vec<Effect> {
        let effects = reduce(m, i);
        self.apply(m, effects.clone());
        effects
    }

    fn apply(&mut self, m: &mut Model, effects: Vec<Effect>) {
        for e in effects {
            match e {
                Effect::Spawn(run) => {
                    self.runners.insert(run, collecting());
                }
                Effect::Send { run, cmd } => {
                    self.sent.push((run, format!("{cmd:?}")));
                    if let Some((r, _)) = self.runners.get_mut(&run) {
                        r.handle(cmd);
                    }
                }
                Effect::Close(run) => {
                    self.runners.remove(&run);
                }
                Effect::SaveSession => self.saves += 1,
                Effect::ReadTape(_)
                | Effect::SetAsideSession
                | Effect::PickTape
                | Effect::PickSaveTape
                | Effect::PickExportDir
                | Effect::Write { .. } => {}
            }
        }
        self.settle(m);
    }

    /// Advance every runner until idle, feeding each observation back.
    fn settle(&mut self, m: &mut Model) {
        loop {
            let mut arrived = Vec::new();
            for (id, (r, seen)) in &mut self.runners {
                while r.advance(self.slice).busy {}
                arrived.extend(drain(seen).into_iter().map(|o| (*id, o)));
            }
            if arrived.is_empty() {
                return;
            }
            for (run, obs) in arrived {
                let e = reduce(m, Intent::Observed { run, obs });
                self.apply(m, e);
            }
        }
    }

    /// The commands sent since the last call.
    fn sent(&mut self) -> Vec<(RunId, String)> {
        std::mem::take(&mut self.sent)
    }
}

fn open(h: &mut Sync, m: &mut Model, path: &str, text: &str) -> Vec<Effect> {
    h.act(
        m,
        Intent::TapeRead {
            path: path.to_string(),
            text: Ok(text.to_string()),
            lineage: None,
        },
    )
}

fn status(m: &Model) -> RunStatus {
    m.focused().expect("a focused run").store.status()
}

fn key(s: &str) -> Key {
    Key::new(s).unwrap()
}

#[test]
fn opening_a_tape_records_its_base_and_loads_it_paused() {
    let mut m = Model::default();
    assert!(matches!(
        reduce(&mut m, Intent::Open("tapes/gate.ron".to_string()))[..],
        [Effect::ReadTape(ref p)] if p == "tapes/gate.ron"
    ));
    let mut h = Sync::new(64);
    let e = open(&mut h, &mut m, "tapes/gate.ron", GATE);
    let kinds: Vec<String> = e
        .iter()
        .map(|x| {
            format!("{x:?}")
                .split([' ', '(', '{'])
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(kinds, ["SaveSession", "Spawn", "Send", "Send"]);
    assert!(
        matches!(&e[2], Effect::Send { cmd: Cmd::Breakpoints(b), .. } if b == &[Breakpoint::OnError])
    );
    assert!(matches!(
        &e[3],
        Effect::Send {
            cmd: Cmd::Load { from: None, .. },
            ..
        }
    ));
    let hash = certify::tape_hash(&tape_of(GATE));
    assert_eq!(m.session.bases.len(), 1);
    assert_eq!(m.session.bases[0].path, "tapes/gate.ron");
    assert_eq!(m.session.bases[0].tape_hash.0, hash);
    let run = m.focused().expect("the new run is focused");
    assert_eq!(
        (run.id, run.tape_hash, run.origin),
        (RunId(0), hash, Origin::Run)
    );
    assert_eq!(status(&m), RunStatus::Paused { tick: 0, why: None });
    assert_eq!(run.store.run().unwrap().tape_hash.0, hash);
    assert_eq!(m.cursor(), Cursor::Live);
    // A new session plots every price: six markets.
    assert_eq!(m.session.plots.len(), 6);
    assert!(m.session.plots.iter().all(|k| k.measure == Measure::Price));
    assert!(m.session.plots.contains(&SeriesKey {
        measure: Measure::Price,
        at: At::Market {
            node: key("village"),
            good: key("grain")
        }
    }));
}

#[test]
fn a_tape_that_does_not_parse_or_read_is_logged_and_not_run() {
    let mut m = Model::default();
    let mut h = Sync::new(64);
    assert!(open(&mut h, &mut m, "bad.ron", "Tape(schema: 1,").is_empty());
    let e = h.act(
        &mut m,
        Intent::TapeRead {
            path: "gone.ron".to_string(),
            text: Err("not found".to_string()),
            lineage: None,
        },
    );
    assert!(e.is_empty());
    assert!(m.focused().is_none() && m.session.bases.is_empty());
    let log: Vec<&str> = m.log().iter().map(|l| l.text.as_str()).collect();
    assert!(
        log[0].starts_with("the tape bad.ron does not load: "),
        "{log:?}"
    );
    assert_eq!(log[1], "cannot read the tape gone.ron: not found");
}

#[test]
fn space_runs_and_pauses_and_steps_follow_the_status() {
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    h.sent();
    // Space on a paused run runs it on, at the session's speed.
    m.session.speed = Some(1_000);
    let e = reduce(&mut m, Intent::RunPause);
    assert!(matches!(
        &e[..],
        [Effect::Send {
            cmd: Cmd::Run {
                until: None,
                max_tps: Some(1_000)
            },
            ..
        }]
    ));
    // While it runs (a run with an until, stopped by the host before it settles), Space pauses.
    h.act(&mut m, Intent::Run { until: Some(20) });
    assert_eq!(
        status(&m),
        RunStatus::Paused {
            tick: 20,
            why: Some(rustyecon_gui::run::PauseReason::Reached(20))
        }
    );
    let running = Obs::Running { tick: 20 };
    reduce(
        &mut m,
        Intent::Observed {
            run: RunId(0),
            obs: running,
        },
    );
    assert_eq!(status(&m), RunStatus::Running { tick: 20 });
    assert!(matches!(
        &reduce(&mut m, Intent::RunPause)[..],
        [Effect::Send {
            cmd: Cmd::Pause,
            ..
        }]
    ));
    assert!(matches!(
        &reduce(&mut m, Intent::Pause)[..],
        [Effect::Send {
            cmd: Cmd::Pause,
            ..
        }]
    ));
    // Steps: `.` one tick, a year 52; no step of zero.
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    h.act(&mut m, Intent::Step(1));
    assert_eq!(m.focused().unwrap().store.tick(), 1);
    h.act(&mut m, Intent::StepYear);
    assert_eq!(m.focused().unwrap().store.tick(), 53);
    assert!(reduce(&mut m, Intent::Step(0)).is_empty());
    // Nothing to run without a focus.
    let mut empty = Model::default();
    for i in [
        Intent::RunPause,
        Intent::Pause,
        Intent::Step(1),
        Intent::StepYear,
    ] {
        assert!(reduce(&mut empty, i).is_empty());
    }
}

#[test]
fn a_poisoned_run_takes_no_more_commands() {
    let mut m = Model::default();
    let mut h = Sync::new(16);
    open(&mut h, &mut m, "theft.ron", &common::theft());
    h.act(&mut m, Intent::Run { until: Some(200) });
    assert!(matches!(status(&m), RunStatus::Poisoned { tick: 73, .. }));
    h.sent();
    for i in [
        Intent::RunPause,
        Intent::Run { until: None },
        Intent::Step(3),
        Intent::StepYear,
        Intent::Pause,
    ] {
        assert!(reduce(&mut m, i).is_empty());
    }
}

#[test]
fn a_stopped_record_pauses_its_run_once() {
    // U10 at the model: a non-finite value stops the record, logs the series and tick, and
    // pauses the run, once.
    let mut m = Model::default();
    let mut h = Sync::new(4);
    open(&mut h, &mut m, "gate.ron", GATE);
    let (mut r, seen) = collecting();
    r.handle(Cmd::Load {
        tape: Box::new(tape_of(GATE)),
        from: None,
    });
    r.handle(Cmd::Step(4));
    while r.advance(4).busy {}
    let mut obs: Vec<Obs> = drain(&seen)
        .into_iter()
        .filter(|o| matches!(o, Obs::Batch(_)))
        .collect();
    let Obs::Batch(b) = &mut obs[0] else {
        unreachable!()
    };
    b.rows[2].cells[0].1 = f64::INFINITY;
    let mut effects = Vec::new();
    for _ in 0..2 {
        effects.extend(reduce(
            &mut m,
            Intent::Observed {
                run: RunId(0),
                obs: obs[0].clone(),
            },
        ));
    }
    assert!(matches!(
        &effects[..],
        [Effect::Send {
            cmd: Cmd::Pause,
            ..
        }]
    ));
    assert_eq!(status(&m), RunStatus::Stopped);
    let errors: Vec<&str> = m
        .log()
        .iter()
        .filter(|l| l.level == rustyecon_gui::run::log::Level::Error)
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("(inf) in price at town/bread at tick 2"),
        "{errors:?}"
    );
}

#[test]
fn speed_breakpoints_plots_and_pins_live_in_the_session() {
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    open(&mut h, &mut m, "gate-too.ron", GATE);
    h.sent();
    // The breakpoint reaches every run.
    let e = reduce(&mut m, Intent::BreakOnError(false));
    assert!(m.session.breakpoints.is_empty());
    assert_eq!(e.len(), 3);
    assert!(matches!(e[0], Effect::SaveSession));
    assert!(e[1..]
        .iter()
        .all(|x| matches!(x, Effect::Send { cmd: Cmd::Breakpoints(b), .. } if b.is_empty())));
    reduce(&mut m, Intent::BreakOnError(true));
    assert_eq!(m.session.breakpoints, [Breakpoint::OnError]);
    // A speed change saves, and a running run gets its run again at the new cap.
    reduce(
        &mut m,
        Intent::Observed {
            run: RunId(1),
            obs: Obs::Running { tick: 0 },
        },
    );
    let e = reduce(&mut m, Intent::Speed(Some(250)));
    assert_eq!(m.session.speed, Some(250));
    assert!(matches!(
        &e[..],
        [
            Effect::SaveSession,
            Effect::Send {
                run: RunId(1),
                cmd: Cmd::Run {
                    until: None,
                    max_tps: Some(250)
                }
            }
        ]
    ));
    // Plots and pins by key, each once.
    let k = SeriesKey {
        measure: Measure::Held,
        at: At::Holding {
            holder: rustyecon_gui::run::HolderKey::Actor(key("mill")),
            good: key("bread"),
        },
    };
    assert_eq!(reduce(&mut m, Intent::Plot(k.clone())).len(), 1);
    assert!(reduce(&mut m, Intent::Plot(k.clone())).is_empty());
    assert_eq!(reduce(&mut m, Intent::Unplot(k.clone())).len(), 1);
    assert!(reduce(&mut m, Intent::Unplot(k)).is_empty());
    let pin = Entity::Actor(key("mill"));
    assert_eq!(reduce(&mut m, Intent::Pin(pin.clone())).len(), 1);
    assert!(reduce(&mut m, Intent::Pin(pin.clone())).is_empty());
    assert_eq!(m.session.pins, std::slice::from_ref(&pin));
    // Selection, cursor and focus change the model and ask for nothing, but for an actor's
    // snapshot (a_selected_actor_asks_for_the_snapshot_its_cursor_reads).
    let market = Entity::Market {
        node: key("town"),
        good: key("bread"),
    };
    assert!(reduce(&mut m, Intent::Select(Some(market.clone()))).is_empty());
    assert_eq!(m.selection(), Some(&market));
    assert!(reduce(&mut m, Intent::Cursor(Cursor::At(9))).is_empty());
    assert_eq!(m.cursor(), Cursor::At(9));
    assert!(reduce(&mut m, Intent::Focus(RunId(0))).is_empty());
    assert_eq!(m.focus(), Some(RunId(0)));
    reduce(&mut m, Intent::Focus(RunId(7)));
    assert_eq!(m.focus(), Some(RunId(0)), "an unknown run is not focused");
    // The session round-trips.
    assert_eq!(
        Session::from_ron(&m.session.to_ron()),
        Ok(m.session.clone())
    );
}

#[test]
fn closing_a_run_stops_its_driver_and_ignores_its_late_observations() {
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "a.ron", GATE);
    open(&mut h, &mut m, "b.ron", GATE);
    open(&mut h, &mut m, "b.ron", GATE);
    assert_eq!(m.session.bases.len(), 2, "one base per path");
    let e = reduce(&mut m, Intent::Close(RunId(2)));
    assert!(
        matches!(&e[..], [Effect::Close(RunId(2))]),
        "b.ron is still open"
    );
    assert_eq!(m.focus(), Some(RunId(1)));
    let e = reduce(&mut m, Intent::Close(RunId(1)));
    assert!(matches!(
        &e[..],
        [Effect::Close(RunId(1)), Effect::SaveSession]
    ));
    assert_eq!(m.session.bases.len(), 1);
    assert!(reduce(&mut m, Intent::Close(RunId(1))).is_empty());
    let before = m.log().len();
    let late = reduce(
        &mut m,
        Intent::Observed {
            run: RunId(1),
            obs: Obs::Running { tick: 3 },
        },
    );
    assert!(late.is_empty() && m.log().len() == before);
    assert_eq!(m.runs().count(), 1);
}

#[test]
fn a_changed_base_is_logged_and_an_edited_tape_is_an_experiment() {
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    let changed = edit(
        GATE,
        r#"name: "gate","#,
        &format!(r#"name: "gate [{EXPERIMENT_MARKER} 2026-09-27]","#),
    );
    let e = open(&mut h, &mut m, "gate.ron", &changed);
    assert!(matches!(e[0], Effect::SaveSession));
    assert_eq!(
        m.session.bases[0].tape_hash.0,
        certify::tape_hash(&tape_of(&changed))
    );
    assert!(m
        .log()
        .iter()
        .any(|l| l.text.starts_with("the tape gate.ron changed on disk")));
    assert_eq!(m.focused().unwrap().origin, Origin::Experiment);
    assert_eq!(m.run(RunId(0)).unwrap().origin, Origin::Run);
}

#[test]
fn a_session_that_does_not_read_is_set_aside() {
    let mut m = Model::default();
    let e = reduce(
        &mut m,
        Intent::SessionRead(Ok("Session(format: 9)".to_string())),
    );
    assert!(matches!(&e[..], [Effect::SetAsideSession]));
    assert_eq!(m.session, Session::default());
    let s = Session {
        speed: Some(3),
        serial: 7,
        ..Session::default()
    };
    // A session that reads is this launch's with the next serial, saved at once, so the keys
    // this launch mints (gui.8.<n>) are new to every earlier launch's.
    let e = reduce(&mut m, Intent::SessionRead(Ok(s.to_ron())));
    assert!(matches!(&e[..], [Effect::SaveSession]));
    assert_eq!(
        m.session,
        Session {
            serial: 8,
            ..s.clone()
        }
    );
    assert_eq!(m.mint_key(), None, "no run is open to mint a key for");
    // A G0.1 session, format 1 with no serial, does not read and is set aside.
    let old = s.to_ron().replacen("format: 2,", "format: 1,", 1);
    let e = reduce(&mut m, Intent::SessionRead(Ok(old)));
    assert!(matches!(&e[..], [Effect::SetAsideSession]));
}

#[test]
fn a_selected_actor_asks_for_the_snapshot_its_cursor_reads() {
    // The actor inspector reads the lots and the actor's state from a snapshot of the state the
    // cursor's tick left (docs/GUI.md §4). The model asks the focused run for one when an actor
    // is selected and the run is not running, once for each state tick.
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    h.act(&mut m, Intent::Step(5));
    h.sent();
    let market = Entity::Market {
        node: key("town"),
        good: key("bread"),
    };
    assert!(reduce(&mut m, Intent::Select(Some(market))).is_empty());
    // Live, the cursor reads the state tick 5.
    let mill = Entity::Actor(key("mill"));
    let e = h.act(&mut m, Intent::Select(Some(mill.clone())));
    assert!(
        matches!(
            &e[..],
            [Effect::Send {
                cmd: Cmd::Snapshot(5),
                ..
            }]
        ),
        "{e:?}"
    );
    assert!(m.focused().unwrap().store.snapshot(5).is_some());
    assert!(reduce(&mut m, Intent::Select(Some(mill.clone()))).is_empty());
    // A cursor at report tick 2 reads the state tick 3, and one past the record the last.
    let e = h.act(&mut m, Intent::Cursor(Cursor::At(2)));
    assert!(matches!(
        &e[..],
        [Effect::Send {
            cmd: Cmd::Snapshot(3),
            ..
        }]
    ));
    assert!(reduce(&mut m, Intent::Cursor(Cursor::At(90))).is_empty());
    // A pause asks for the state it left.
    h.act(&mut m, Intent::Cursor(Cursor::Live));
    h.sent();
    h.act(&mut m, Intent::Step(2));
    let sent: Vec<String> = h.sent().into_iter().map(|x| x.1).collect();
    assert_eq!(sent, ["Step(2)", "Snapshot(7)"]);
    // A running run is not asked; the record holds its holdings every tick.
    reduce(
        &mut m,
        Intent::Observed {
            run: RunId(0),
            obs: Obs::Running { tick: 7 },
        },
    );
    reduce(&mut m, Intent::Select(None));
    assert!(reduce(&mut m, Intent::Select(Some(Entity::Actor(key("oven"))))).is_empty());
    // The toolbar's Open asks the host for a file dialog.
    assert!(matches!(
        &reduce(&mut m, Intent::PickTape)[..],
        [Effect::PickTape]
    ));
}

#[test]
fn rationing_onsets_are_logged_once_a_class_line() {
    // docs/GUI.md §4, the log: rationing onset by class. The first tick each class line is
    // filled below its request is one line, once a load; the reference is found here from the
    // recorded series alone. The gate's sellers are rationed on and off, so a line an episode
    // would log hundreds; this logs one each.
    let mut m = Model::default();
    let mut h = Sync::new(7);
    open(&mut h, &mut m, "gate.ron", GATE);
    h.act(&mut m, Intent::Step(700));
    let store = &m.focused().unwrap().store;
    let mut want: Vec<(u64, String)> = Vec::new();
    for k in store.catalogue() {
        let At::Class {
            node,
            good,
            class,
            side,
        } = &k.at
        else {
            continue;
        };
        if k.measure != Measure::Requested {
            continue;
        }
        let req = store.series(k).unwrap();
        let filled = store
            .series(&SeriesKey {
                measure: Measure::Filled,
                at: k.at.clone(),
            })
            .unwrap();
        let first = req
            .ticks()
            .iter()
            .zip(req.values())
            .find(|(&t, &r)| filled.at(t).expect("a line has both") < r);
        if let Some((&t, _)) = first {
            let side = if *side == SideTag::Buy { "buy" } else { "sell" };
            want.push((
                t,
                format!("rationing onset: {class} ({side}) at {node}/{good}, "),
            ));
        }
    }
    let got: Vec<(u64, String)> = m
        .log()
        .iter()
        .filter(|l| l.level == rustyecon_gui::run::log::Level::Rationing)
        .map(|l| (l.tick.unwrap(), l.text.clone()))
        .collect();
    assert!(want.len() >= 2, "the gate rations: {want:?}");
    assert_eq!(got.len(), want.len(), "{got:#?}\n{want:#?}");
    let mut got_sorted = got.clone();
    got_sorted.sort();
    let mut want_sorted = want.clone();
    want_sorted.sort();
    for ((gt, gtext), (wt, wtext)) in got_sorted.iter().zip(&want_sorted) {
        assert_eq!(gt, wt);
        assert!(gtext.starts_with(wtext.as_str()), "{gtext} / {wtext}");
    }
    // Bread rations from tick 0: the pensioners' coin buys less than they ask for.
    assert!(got
        .iter()
        .any(|(t, s)| *t == 0 && s.contains("pensioners (buy) at") && s.contains("/bread")));
}

#[test]
fn a_log_line_moves_the_cursor_to_the_tick_it_names() {
    // docs/GUI.md §4: a log line's tick is a report tick, the tick that ran, as the cursor
    // counts; a line about a state names the tick that left it and says "state tick". A click
    // on the snapshot line of state tick 5 puts the cursor on report tick 4, which reads that
    // same snapshot and asks for no other.
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    h.act(&mut m, Intent::Step(5));
    h.act(&mut m, Intent::Select(Some(Entity::Actor(key("mill")))));
    h.act(&mut m, Intent::Step(3));
    let store = &m.focused().unwrap().store;
    assert!(store.snapshot(5).is_some() && store.snapshot(8).is_some());
    let lines: Vec<(Option<u64>, &str)> =
        m.log().iter().map(|l| (l.tick, l.text.as_str())).collect();
    let find = |text: &str| {
        lines
            .iter()
            .find(|l| l.1 == text)
            .unwrap_or_else(|| panic!("no line {text:?} in {lines:#?}"))
            .0
    };
    assert_eq!(
        find("running from state tick 0"),
        None,
        "no tick left state 0"
    );
    assert_eq!(find("stepped at state tick 5"), Some(4));
    assert_eq!(find("snapshot of state tick 5"), Some(4));
    assert_eq!(find("running from state tick 5"), Some(4));
    assert_eq!(find("snapshot of state tick 8"), Some(7));
    // Every line's tick is a tick the record ran, which the cursor takes as it is.
    for l in m.log() {
        if let Some(t) = l.tick {
            assert_eq!(store.report_at(Some(t)), Some(t), "{l:?}");
        }
    }
    // The click: the cursor reads the state the line names, whose snapshot is there.
    let snap5 = find("snapshot of state tick 5").unwrap();
    h.sent();
    h.act(&mut m, Intent::Cursor(Cursor::At(snap5)));
    assert!(h.sent().is_empty(), "no other snapshot is asked for");
    assert_eq!(m.focused().unwrap().store.state_at(Some(snap5)), 5);
}

#[test]
fn a_run_whose_worker_ended_says_so_and_stops() {
    // A Runner that panics says nothing; its driver reports the worker's end once, as
    // Obs::Ended (drive/mod.rs). The run then shows it, logs it, and takes no command.
    let mut m = Model::default();
    let mut h = Sync::new(8);
    open(&mut h, &mut m, "gate.ron", GATE);
    h.act(&mut m, Intent::Step(3));
    h.sent();
    reduce(
        &mut m,
        Intent::Observed {
            run: RunId(0),
            obs: Obs::Ended,
        },
    );
    assert_eq!(status(&m), RunStatus::Ended);
    let run = m.focused().unwrap();
    assert!(!run.store.can_run());
    let t = rustyecon_gui::vm::toolbar::build(&run.store, run.origin, false);
    assert_eq!(t.health.status, rustyecon_gui::vm::toolbar::Status::Ended);
    assert!(!t.controls.can_run && !t.controls.running);
    let last = m.log().last().unwrap();
    assert_eq!(last.level, rustyecon_gui::run::log::Level::Error);
    assert!(last.text.contains("worker ended"), "{}", last.text);
    for i in [
        Intent::RunPause,
        Intent::Step(1),
        Intent::StepYear,
        Intent::Run { until: None },
    ] {
        assert!(h.act(&mut m, i).is_empty());
    }
    assert!(h.sent().is_empty());
}

#[test]
fn a_session_of_another_tape_still_plots_every_price() {
    // docs/GUI.md §4: a run opens with every price plotted. A session's plots name series by
    // key; the Appendix B world's name nothing in the gate's, so the gate's prices are added,
    // and the plots' view-model leaves the other world's keys out of the stack.
    let mut m = Model::default();
    let mut h = Sync::new(64);
    open(&mut h, &mut m, "appb.ron", common::APPB);
    let appb: Vec<SeriesKey> = m.session.plots.clone();
    assert_eq!(appb.len(), 4);
    open(&mut h, &mut m, "gate.ron", GATE);
    let gate: Vec<SeriesKey> = ["town", "village"]
        .iter()
        .flat_map(|n| {
            ["bread", "fuel", "grain"].map(|g| SeriesKey {
                measure: Measure::Price,
                at: At::Market {
                    node: key(n),
                    good: key(g),
                },
            })
        })
        .collect();
    let mut want = appb.clone();
    want.extend(gate.iter().cloned());
    assert_eq!(m.session.plots, want);
    h.act(&mut m, Intent::Step(3));
    let store = &m.focused().unwrap().store;
    let vm = rustyecon_gui::vm::plots::build(store, &m.session.plots, None).unwrap();
    let units: Vec<&str> = vm.panels.iter().map(|p| p.unit.as_str()).collect();
    assert_eq!(units, ["coin per bread", "coin per fuel", "coin per grain"]);
    let drawn: Vec<SeriesKey> = vm
        .panels
        .iter()
        .flat_map(|p| p.lines.iter().map(|l| l.key.clone()))
        .collect();
    let mut sorted = gate.clone();
    sorted.sort();
    let mut drawn_sorted = drawn.clone();
    drawn_sorted.sort();
    assert_eq!(drawn_sorted, sorted);
    assert!(vm
        .panels
        .iter()
        .flat_map(|p| &p.lines)
        .all(|l| l.points == 3));
    assert_eq!(vm.absent, appb);
    // A session that plots one of this world's series keeps its choice: nothing is added.
    let mut m2 = Model::with_session(Session {
        plots: vec![gate[0].clone()],
        ..Session::default()
    });
    let mut h2 = Sync::new(64);
    open(&mut h2, &mut m2, "gate.ron", GATE);
    assert_eq!(m2.session.plots, [gate[0].clone()]);
}

#[test]
fn apply_branches_from_the_parents_ring_and_files_are_effects() {
    // The editor as a state machine (docs/GUI.md §5.1): Apply needs a date and staged edits;
    // then it starts a new run whose Load carries the parent's ring checkpoint, focuses it and
    // clears the editor. "Save tape as" and export are Write effects, a write that lands puts
    // the saved tape on disk, and one that fails is logged and shown.
    let mut m = Model::default();
    let mut h = Sync::new(64);
    open(&mut h, &mut m, "gate.ron", GATE);
    assert!(h.act(&mut m, Intent::Apply).is_empty());
    assert_eq!(
        m.editor().error.as_deref(),
        Some("no date to stamp the experiment with")
    );
    h.act(&mut m, Intent::Today(Date::parse("2026-09-27").unwrap()));
    assert!(h.act(&mut m, Intent::Apply).is_empty());
    assert_eq!(m.editor().error.as_deref(), Some("no edit to apply"));
    h.act(&mut m, Intent::Run { until: Some(1000) });
    let form = Form {
        kind: OpKind::AddEvent,
        key: "gui.1.1".to_string(),
        date: "1765-06-01".to_string(),
        act: "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")".to_string(),
        note: "restore early".to_string(),
        ..Form::default()
    };
    assert!(h.act(&mut m, Intent::Stage(form.clone())).is_empty());
    assert_eq!(m.editor().staged.len(), 1);
    h.act(&mut m, Intent::Unstage(0));
    assert!(m.editor().staged.is_empty());
    h.act(&mut m, Intent::Stage(form));
    h.sent();
    let e = h.act(&mut m, Intent::Apply);
    let clock = m.focused().unwrap().store.world().unwrap().clock;
    let jan = clock.tick_of(Date::parse("1765-01-01").unwrap()).unwrap();
    match &e[..] {
        [Effect::Spawn(RunId(1)), Effect::Send {
            run: RunId(1),
            cmd: Cmd::Breakpoints(b),
        }, Effect::Send {
            run: RunId(1),
            cmd:
                Cmd::Load {
                    from: Some(ResumeFrom::Ring(cp)),
                    tape,
                },
        }] => {
            assert_eq!(b, &[Breakpoint::OnError]);
            assert_eq!(cp.tick(), jan);
            assert_eq!(cp.run(), m.run(RunId(0)).unwrap().store.run().unwrap());
            assert_eq!(tape.header.name, "gate [GUI experiment 2026-09-27]");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(m.focus(), Some(RunId(1)));
    assert_eq!(m.cursor(), Cursor::Live);
    assert!(m.editor().staged.is_empty() && m.editor().error.is_none());
    assert_eq!(
        m.focused().unwrap().store.start(),
        jan,
        "the runner resumed"
    );
    // Save: the tape and its lineage beside it.
    let e = h.act(&mut m, Intent::SaveTape(" out/b.ron ".to_string()));
    let [Effect::Write { job, files }] = &e[..] else {
        panic!("{e:?}");
    };
    assert_eq!(
        job,
        &Job::SaveTape {
            run: RunId(1),
            path: "out/b.ron".to_string()
        }
    );
    let paths: Vec<&str> = files.iter().map(|f| f.0.as_str()).collect();
    assert_eq!(paths, ["out/b.ron", "out/b.lineage.ron"]);
    assert_eq!(files[0].1, m.focused().unwrap().tape.to_ron());
    h.act(
        &mut m,
        Intent::Written {
            job: job.clone(),
            result: Ok(paths.iter().map(|p| p.to_string()).collect()),
        },
    );
    assert_eq!(m.focused().unwrap().path.as_deref(), Some("out/b.ron"));
    // Export: every file of it, under the directory.
    let e = h.act(&mut m, Intent::Export("out/x".to_string()));
    let [Effect::Write { job, files }] = &e[..] else {
        panic!("{e:?}");
    };
    let names: Vec<String> = files
        .iter()
        .map(|f| {
            let p = std::path::Path::new(&f.0);
            assert!(p.starts_with("out/x"), "{}", f.0);
            p.file_name().unwrap().to_string_lossy().to_string()
        })
        .collect();
    assert_eq!(
        names,
        [
            "series.csv",
            "manifest.ron",
            "tape.ron",
            "tape.lineage.ron",
            "ancestor.ron"
        ]
    );
    h.act(
        &mut m,
        Intent::Written {
            job: job.clone(),
            result: Err("disk full".to_string()),
        },
    );
    assert_eq!(
        m.editor().error.as_deref(),
        Some("run 1 not exported to out/x: disk full")
    );
    assert!(m
        .log()
        .iter()
        .any(|l| l.text == "editor: run 1 not exported to out/x: disk full"));
    // An empty path or directory is refused; the dialogs are effects.
    assert!(h.act(&mut m, Intent::SaveTape("  ".to_string())).is_empty());
    assert!(h.act(&mut m, Intent::Export(String::new())).is_empty());
    assert!(matches!(
        &h.act(&mut m, Intent::PickSaveTape)[..],
        [Effect::PickSaveTape]
    ));
    assert!(matches!(
        &h.act(&mut m, Intent::PickExportDir)[..],
        [Effect::PickExportDir]
    ));
}
