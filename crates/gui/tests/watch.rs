//! G1's panels (docs/GUI.md §9): event and date breakpoints, the watchlist and log axes. The
//! breakpoints through a Runner on this thread, and through the model and a host of Runners;
//! the watchlist and the log scales through the model and the session.

mod common;

use common::sync::SyncHost;
use common::{collecting, drain, key, reference_hashes, tape_of, GATE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::model::{Effect, Intent, Model, Session};
use rustyecon_gui::run::log::Level;
use rustyecon_gui::run::{At, Breakpoint, Cmd, Measure, Obs, PauseReason, Runner, SeriesKey};
use rustyecon_gui::vm;

fn gate_clock() -> Clock {
    let t = tape_of(GATE);
    Clock {
        start: t.header.start,
        ticks_per_year: t.header.ticks_per_year,
    }
}

fn tick_of(date: &str) -> u64 {
    gate_clock().tick_of(Date::parse(date).unwrap()).unwrap()
}

/// Every pause the Runner reported, in order.
fn pauses(obs: &[Obs]) -> Vec<(u64, PauseReason)> {
    obs.iter()
        .filter_map(|o| match o {
            Obs::Paused { tick, why } => Some((*tick, why.clone())),
            _ => None,
        })
        .collect()
}

/// Every tick's hash the Runner reported, in order.
fn hashes(obs: &[Obs]) -> Vec<u64> {
    obs.iter()
        .filter_map(|o| match o {
            Obs::Batch(b) => Some(b.rows.iter().map(|r| r.hash).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn settle(r: &mut Runner) {
    while r.advance(64).busy {}
}

#[test]
fn event_and_date_breakpoints_pause_after_their_tick() {
    // G1: an event breakpoint pauses the run after each tick its event fires in, every
    // occurrence of a recurring one; a date breakpoint after the tick its date falls in, once,
    // and not in a run already past it; a step stops at a breakpoint too. The hashes are the
    // run's without them (U4).
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    r.handle(Cmd::Breakpoints(vec![Breakpoint::OnEvent(key("pension"))]));
    r.handle(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
    let pension = tick_of("1751-01-01");
    for year in 0..3 {
        r.handle(Cmd::Run {
            until: Some(600),
            max_tps: None,
        });
        settle(&mut r);
        // The pension recurs every pension.period, a year: 52 whole ticks from its first.
        let want = pension + 52 * year + 1;
        assert_eq!(
            pauses(&drain(&seen)).last(),
            Some(&(
                want,
                PauseReason::Breakpoint(Breakpoint::OnEvent(key("pension")))
            )),
            "occurrence {year}"
        );
    }
    // A date breakpoint: once, after its tick; a step stops there too; one behind the state
    // never fires.
    let date = Date::parse("1754-06-01").unwrap();
    r.handle(Cmd::Breakpoints(vec![
        Breakpoint::OnDate(date),
        Breakpoint::OnDate(Date::parse("1751-06-01").unwrap()),
    ]));
    r.handle(Cmd::Step(1000));
    settle(&mut r);
    let d = tick_of("1754-06-01") + 1;
    assert_eq!(
        pauses(&drain(&seen)),
        [(d, PauseReason::Breakpoint(Breakpoint::OnDate(date)))]
    );
    r.handle(Cmd::Run {
        until: Some(900),
        max_tps: None,
    });
    settle(&mut r);
    assert_eq!(pauses(&drain(&seen)), [(900, PauseReason::Reached(900))]);
    // A key the world lacks never fires; the breakpoint on error does not pause a good run.
    r.handle(Cmd::Breakpoints(vec![
        Breakpoint::OnError,
        Breakpoint::OnEvent(key("no.such.event")),
    ]));
    r.handle(Cmd::Run {
        until: Some(2080),
        max_tps: None,
    });
    settle(&mut r);
    assert_eq!(pauses(&drain(&seen)), [(2080, PauseReason::Reached(2080))]);
}

#[test]
fn breakpoints_change_no_hash() {
    // U4 under G1's breakpoints: a run paused at every pension, at a date and at mine.cut
    // hashes as the run with none.
    let t = tape_of(GATE);
    let (mut r, seen) = collecting();
    r.handle(Cmd::Breakpoints(vec![
        Breakpoint::OnEvent(key("pension")),
        Breakpoint::OnEvent(key("mine.cut")),
        Breakpoint::OnDate(Date::parse("1765-06-01").unwrap()),
    ]));
    r.handle(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
    let mut all = Vec::new();
    let mut stops = 0;
    loop {
        r.handle(Cmd::Run {
            until: Some(2080),
            max_tps: None,
        });
        settle(&mut r);
        let obs = drain(&seen);
        all.extend(obs.iter().cloned());
        match pauses(&obs).last() {
            Some((_, PauseReason::Reached(2080))) => break,
            Some((_, PauseReason::Breakpoint(_))) => stops += 1,
            other => panic!("{other:?}"),
        }
    }
    // 40 pensions (1751 to 1790), the cut and the date.
    assert_eq!(stops, 42);
    assert_eq!(hashes(&all), reference_hashes(&t, 2080));
}

#[test]
fn breakpoints_watch_and_log_scales_live_in_the_session() {
    // The model: a breakpoint read from text, a date or an event's key, set for every run and
    // saved; text that is neither is logged and refused; a key the run's world lacks is set,
    // and the log says so. The watchlist and the log scales, each by key, each once.
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
    let e = h.act(&mut m, Intent::BreakAt(" 1765-06-01 ".to_string()));
    assert!(matches!(e[0], Effect::SaveSession));
    assert!(e[1..].iter().any(|x| matches!(
        x,
        Effect::Send { cmd: Cmd::Breakpoints(b), .. } if b.len() == 2
    )));
    h.act(&mut m, Intent::BreakAt("mine.cut".to_string()));
    let date = Date::parse("1765-06-01").unwrap();
    assert_eq!(
        m.session.breakpoints,
        [
            Breakpoint::OnError,
            Breakpoint::OnEvent(key("mine.cut")),
            Breakpoint::OnDate(date)
        ]
    );
    let lines = m.log().len();
    assert!(h
        .act(&mut m, Intent::BreakAt("Not a key!".to_string()))
        .is_empty());
    let last = m.log().last().unwrap();
    assert_eq!(last.level, Level::Error);
    assert!(last
        .text
        .contains("neither a date YYYY-MM-DD nor an event's key"));
    assert_eq!(m.log().len(), lines + 1);
    h.act(&mut m, Intent::BreakAt("elsewhere.event".to_string()));
    assert!(m
        .log()
        .iter()
        .any(|l| l.text.contains("no event elsewhere.event in gate's world")));
    h.act(
        &mut m,
        Intent::Breakpoint {
            at: Breakpoint::OnEvent(key("elsewhere.event")),
            on: false,
        },
    );
    h.act(
        &mut m,
        Intent::Breakpoint {
            at: Breakpoint::OnDate(date),
            on: false,
        },
    );
    assert_eq!(
        m.session.breakpoints,
        [Breakpoint::OnError, Breakpoint::OnEvent(key("mine.cut"))]
    );
    // The run pauses at the cut, and the log names the breakpoint.
    h.act(&mut m, Intent::Run { until: Some(2080) });
    let cut = tick_of("1760-03-01");
    let paused = m
        .log()
        .iter()
        .any(|l| l.text == format!("breakpoint on event mine.cut at state tick {}", cut + 1));
    assert!(paused, "{:?}", m.log().last());
    // The watchlist: by key, once each; the log scales, by unit.
    let bread = SeriesKey {
        measure: Measure::Price,
        at: At::Market {
            node: key("town"),
            good: key("bread"),
        },
    };
    assert_eq!(h.act(&mut m, Intent::Watch(bread.clone())).len(), 1);
    assert!(h.act(&mut m, Intent::Watch(bread.clone())).is_empty());
    assert_eq!(m.session.watch, std::slice::from_ref(&bread));
    let on = |on| Intent::LogAxis {
        unit: "coin per bread".to_string(),
        on,
    };
    assert_eq!(h.act(&mut m, on(true)).len(), 1);
    assert!(h.act(&mut m, on(true)).is_empty());
    assert_eq!(m.session.log_axes, ["coin per bread".to_string()]);
    // The session keeps all of it, format 3.
    let text = m.session.to_ron();
    assert!(text.contains("format: 3,"));
    assert_eq!(Session::from_ron(&text), Ok(m.session.clone()));
    // The watchlist's view: the price at the cursor and the change from the tick before.
    let store = &m.focused().unwrap().store;
    let w = vm::watch::build(store, &m.session.watch, &m.session.plots, Some(cut)).unwrap();
    let s = store.series(&bread).unwrap();
    assert_eq!(w.rows[0].value, s.at(cut));
    assert_eq!(w.rows[0].before, s.at(cut - 1));
    assert_eq!(
        w.rows[0].change.map(f64::to_bits),
        Some((s.at(cut).unwrap() - s.at(cut - 1).unwrap()).to_bits())
    );
    assert_eq!(w.rows[0].unit, "coin per bread");
    assert!(w.rows[0].plotted, "the gate opens with every price plotted");
    assert_eq!(h.act(&mut m, Intent::Unwatch(bread.clone())).len(), 1);
    assert!(h.act(&mut m, Intent::Unwatch(bread)).is_empty());
    assert!(m.session.watch.is_empty());
    assert_eq!(h.act(&mut m, on(false)).len(), 1);
    assert!(m.session.log_axes.is_empty());
}

#[test]
fn an_event_breakpoint_names_the_pause_before_a_date_in_one_tick() {
    // Decision 209, which G1's verification found no test of: when an event breakpoint and a
    // date breakpoint both fall in one tick, the run pauses once, after it, and says the
    // event's. mine.cut fires on 1760-03-01; the date breakpoint is that date. Listed either way
    // round, the reason is the event's; then the date's alone, once the event's is cleared.
    let cut = tick_of("1760-03-01");
    let date = Breakpoint::OnDate(Date::parse("1760-03-01").unwrap());
    let event = Breakpoint::OnEvent(key("mine.cut"));
    for list in [
        vec![date.clone(), event.clone()],
        vec![event.clone(), date.clone()],
    ] {
        let (mut r, seen) = collecting();
        r.handle(Cmd::Breakpoints(list.clone()));
        r.handle(Cmd::Load {
            tape: Box::new(tape_of(GATE)),
            from: None,
        });
        r.handle(Cmd::Run {
            until: Some(2080),
            max_tps: None,
        });
        settle(&mut r);
        assert_eq!(
            pauses(&drain(&seen)),
            [(cut + 1, PauseReason::Breakpoint(event.clone()))],
            "{list:?}"
        );
    }
    let (mut r, seen) = collecting();
    r.handle(Cmd::Breakpoints(vec![date.clone()]));
    r.handle(Cmd::Load {
        tape: Box::new(tape_of(GATE)),
        from: None,
    });
    r.handle(Cmd::Run {
        until: Some(2080),
        max_tps: None,
    });
    settle(&mut r);
    assert_eq!(
        pauses(&drain(&seen)),
        [(cut + 1, PauseReason::Breakpoint(date))]
    );
}
