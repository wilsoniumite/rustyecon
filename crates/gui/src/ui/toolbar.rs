//! The toolbar (docs/GUI.md §4): Open; run and pause (Space); step one tick (`.`); step a
//! year; run until a tick or a date; the speed cap; the date, tick and ticks per year; the
//! identity chip (commit and dirty flag, `world_id`, `tape_hash`, origin); the health chip
//! (status, the tick's largest margin, the hash, "ledger changed" or "ledger unchecked" with
//! why, and a failed run's ledger line and last good tick).

use super::fmt;
use crate::model::{Intent, Model};
use crate::run::{LedgerCheck, RunId};
use crate::vm::toolbar::{self, Status, ToolbarVm};
use egui::{Button, RichText};

/// The toolbar's own text: the "run until" field and why it did not read.
#[derive(Debug, Default)]
pub struct ToolbarState {
    until: String,
    error: Option<String>,
}

/// The speed caps offered, in model years a second; `None` runs as fast as the machine does.
const YEARS_A_SECOND: [Option<u32>; 5] = [None, Some(1), Some(10), Some(100), Some(1000)];

fn speed_name(tps: Option<u32>, tpy: u32) -> String {
    match tps {
        None => "no cap".to_string(),
        Some(t) if tpy > 0 && t % tpy == 0 => {
            let y = t / tpy;
            if y == 1 {
                "1 year/s".to_string()
            } else {
                format!("{y} years/s")
            }
        }
        Some(t) => format!("{t} ticks/s"),
    }
}

/// Draw the toolbar of the focused run.
pub fn show(ui: &mut egui::Ui, m: &Model, st: &mut ToolbarState, out: &mut Vec<Intent>) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Open…")
            .on_hover_text("open a tape file")
            .clicked()
        {
            out.push(Intent::PickTape);
        }
        let runs: Vec<(RunId, String)> = m
            .runs()
            .map(|r| {
                let name = r
                    .store
                    .world()
                    .map_or_else(|| r.path.clone().unwrap_or_default(), |w| w.name.clone());
                (r.id, format!("{}: {name}", r.id))
            })
            .collect();
        let Some(run) = m.focused() else {
            ui.weak("no tape open");
            return;
        };
        if runs.len() > 1 {
            let current = runs
                .iter()
                .find(|(id, _)| *id == run.id)
                .map_or_else(String::new, |r| r.1.clone());
            egui::ComboBox::from_id_salt("focus")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for (id, name) in &runs {
                        if ui.selectable_label(*id == run.id, name).clicked() {
                            out.push(Intent::Focus(*id));
                        }
                    }
                });
        }
        let vm = toolbar::build(&run.store, run.origin, m.ledger(run.id));
        let can = vm.controls.can_run;
        let running = vm.controls.running;
        let label = if running { "Pause" } else { "Run" };
        if ui
            .add_enabled(can || running, Button::new(label))
            .on_hover_text("Space")
            .clicked()
        {
            out.push(Intent::RunPause);
        }
        if ui
            .add_enabled(can && !running, Button::new("Step"))
            .on_hover_text("one tick (.)")
            .clicked()
        {
            out.push(Intent::Step(1));
        }
        if ui
            .add_enabled(can && !running, Button::new("Step a year"))
            .clicked()
        {
            out.push(Intent::StepYear);
        }
        let label = ui.label("until");
        let edit = ui
            .add(
                egui::TextEdit::singleline(&mut st.until)
                    .desired_width(96.0)
                    .hint_text("tick or date"),
            )
            .labelled_by(label.id)
            .on_hover_text(
                "a tick pauses before it runs, as the cli's --until; a date YYYY-MM-DD runs \
                 through the tick it falls in",
            );
        let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let go = ui.add_enabled(can, Button::new("Run until")).clicked();
        if go || (entered && can) {
            match run
                .store
                .world()
                .map(|w| toolbar::parse_until(&st.until, &w.clock))
            {
                Some(Ok(t)) => {
                    st.error = None;
                    out.push(Intent::Run { until: Some(t) });
                }
                Some(Err(e)) => st.error = Some(e),
                None => {}
            }
        }
        if let Some(e) = &st.error {
            ui.colored_label(ui.visuals().error_fg_color, e);
        }
        let tpy = vm.clock.as_ref().map_or(1, |c| c.ticks_per_year);
        egui::ComboBox::new("speed", "speed cap")
            .selected_text(speed_name(m.session.speed, tpy))
            .show_ui(ui, |ui| {
                for y in YEARS_A_SECOND {
                    let tps = y.map(|y| y.saturating_mul(tpy));
                    if ui
                        .selectable_label(m.session.speed == tps, speed_name(tps, tpy))
                        .clicked()
                    {
                        out.push(Intent::Speed(tps));
                    }
                }
            });
        ui.separator();
        chips(ui, &vm);
    });
}

fn chips(ui: &mut egui::Ui, vm: &ToolbarVm) {
    if let Some(c) = &vm.clock {
        ui.label(format!(
            "{} · tick {} · {} ticks/year",
            c.date, c.tick, c.ticks_per_year
        ));
    }
    if let Some(id) = &vm.identity {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            let commit: String = id.commit.chars().take(10).collect();
            let state = if id.dirty { "dirty" } else { "clean" };
            ui.label(format!(
                "{} · {} · {commit} {state} · world {} · tape {}",
                id.name, id.origin, id.world_id, id.tape_hash
            ))
            .on_hover_text(format!(
                "build {} ({}), world_id {}, tape_hash {}, origin {}",
                id.commit, state, id.world_id, id.tape_hash, id.origin
            ));
        });
    }
    let h = &vm.health;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        let status = match h.status {
            Status::Empty => "Empty",
            Status::Paused => "Paused",
            Status::Running => "Running",
            Status::Poisoned => "Poisoned",
            Status::Stopped => "Stopped",
            Status::Ended => "Ended: the run's worker stopped unasked",
        };
        let mut text = status.to_string();
        if let Some(p) = &h.paused {
            if h.status == Status::Paused {
                text.push_str(&format!(" ({p})"));
            }
        }
        if let Some(mm) = h.max_margin {
            text.push_str(&format!(" · max_margin {}", fmt(mm)));
        }
        if let Some(hash) = &h.hash {
            text.push_str(&format!(" · hash {hash}"));
        }
        match &h.ledger {
            LedgerCheck::Changed => text.push_str(" · ledger changed"),
            LedgerCheck::Unknown(_) => text.push_str(" · ledger unchecked"),
            LedgerCheck::NoParent | LedgerCheck::Same => {}
        }
        let alarm = matches!(h.status, Status::Poisoned | Status::Stopped | Status::Ended);
        let rich = if alarm {
            RichText::new(text).color(ui.visuals().error_fg_color)
        } else {
            RichText::new(text)
        };
        let mut hover = String::new();
        if let Some(r) = h.run_margin {
            hover.push_str(&format!("the run's largest margin {r}\n"));
        }
        let resp = ui.label(rich);
        if !hover.is_empty() {
            resp.on_hover_text(hover);
        }
        if let Some(line) = &h.ledger_line {
            ui.colored_label(ui.visuals().error_fg_color, line);
        }
        for line in &h.ledger_keys {
            ui.label(line);
        }
        if let Some(t) = h.last_good_tick {
            ui.label(format!("last good tick {t}"));
        }
        if let Some(s) = &h.stopped {
            ui.colored_label(ui.visuals().error_fg_color, s);
        }
        if let LedgerCheck::Unknown(why) = &h.ledger {
            ui.label(format!("ledger unchecked: {why}"));
        }
    });
}
