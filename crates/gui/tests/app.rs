//! The app headless (docs/GUI.md §8.1, egui_kittest with no GPU): `rustyecon-gui
//! tapes/gate.ron` opens the tape paused at tick 0 with every price plotted, Space runs it and
//! Space pauses it, and `.` steps one tick. The worker wakes the frame loop through the app's
//! `request_repaint` callback.

use egui_kittest::Harness;
use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::model::Model;
use rustyecon_gui::run::{PauseReason, RunStatus};
use std::time::{Duration, Instant};

fn step_until(h: &mut Harness<'_, GuiApp>, done: impl Fn(&Model) -> bool) {
    let t0 = Instant::now();
    while !done(h.state().model()) {
        h.step();
        assert!(
            t0.elapsed() < Duration::from_secs(300),
            "the app did not get there"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn status(m: &Model) -> Option<RunStatus> {
    m.focused().map(|r| r.store.status())
}

#[test]
fn the_app_opens_a_tape_paused_and_space_runs_it() {
    let tape = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes/gate.ron").to_string();
    let mut h = Harness::new_eframe(|cc| {
        GuiApp::new(
            &cc.egui_ctx,
            Launch {
                tape: Some(tape),
                files: None,
            },
        )
    });
    step_until(&mut h, |m| {
        status(m) == Some(RunStatus::Paused { tick: 0, why: None })
    });
    assert_eq!(h.state().model().session.plots.len(), 6, "every price");
    h.key_press(egui::Key::Space);
    step_until(&mut h, |m| m.focused().is_some_and(|r| r.store.tick() > 10));
    h.key_press(egui::Key::Space);
    step_until(&mut h, |m| {
        matches!(
            status(m),
            Some(RunStatus::Paused {
                why: Some(PauseReason::Asked),
                ..
            })
        )
    });
    let at = h.state().model().focused().unwrap().store.tick();
    h.key_press(egui::Key::Period);
    step_until(&mut h, |m| {
        matches!(
            status(m),
            Some(RunStatus::Paused {
                why: Some(PauseReason::Stepped),
                ..
            })
        )
    });
    assert_eq!(h.state().model().focused().unwrap().store.tick(), at + 1);
}
