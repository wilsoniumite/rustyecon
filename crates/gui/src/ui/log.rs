//! The log (docs/GUI.md §4): loads, runs, pauses, fired events, refusals and errors, oldest
//! first, and the breakpoints. A line with a tick moves the cursor there.
//!
//! The breakpoints (G1): on error, and on an event by its key or on a date, each for every run.
//! The field reads a date `YYYY-MM-DD` or else an event's key; the event inspector sets one on
//! its event too. A run pauses after the tick an event fires in or a date falls in, and the log
//! says which breakpoint paused it.

use crate::model::{Cursor, Intent};
use crate::run::Breakpoint;
use crate::vm::log::LogVm;
use egui::{RichText, Sense};

/// Draw the log, its breakpoints and the field that adds one.
pub fn show(
    ui: &mut egui::Ui,
    vm: &LogVm,
    breakpoints: &[Breakpoint],
    field: &mut String,
    out: &mut Vec<Intent>,
) {
    ui.horizontal_wrapped(|ui| {
        let mut on = breakpoints.contains(&Breakpoint::OnError);
        if ui
            .checkbox(&mut on, "Break on error")
            .on_hover_text("pause the run on a run error, and say it was this breakpoint")
            .changed()
        {
            out.push(Intent::BreakOnError(on));
        }
        for b in breakpoints.iter().filter(|b| **b != Breakpoint::OnError) {
            ui.label(b.to_string());
            let r = ui.small_button("✕").on_hover_text(format!("clear the {b}"));
            r.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("clear the {b}"))
            });
            if r.clicked() {
                out.push(Intent::Breakpoint {
                    at: b.clone(),
                    on: false,
                });
            }
        }
        let l = ui.label("break at");
        let r = ui
            .add(
                egui::TextEdit::singleline(field)
                    .desired_width(160.0)
                    .hint_text("YYYY-MM-DD or an event's key"),
            )
            .labelled_by(l.id);
        let entered = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (ui.button("Add breakpoint").clicked() || entered) && !field.trim().is_empty() {
            out.push(Intent::BreakAt(std::mem::take(field)));
        }
        let errors = vm.lines.iter().filter(|l| l.error).count();
        ui.weak(format!("{} lines, {errors} errors", vm.lines.len()));
    });
    let row = ui.text_style_height(&egui::TextStyle::Body);
    egui::ScrollArea::vertical()
        .id_salt("log")
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show_rows(ui, row, vm.lines.len(), |ui, range| {
            for l in &vm.lines[range] {
                let run = l.run.map_or_else(String::new, |r| format!("run {r} "));
                let tick = l.tick.map_or_else(String::new, |t| format!("[{t}] "));
                let text = format!("{run}{tick}{}", l.text);
                let rich = if l.error {
                    RichText::new(text).color(ui.visuals().error_fg_color)
                } else {
                    RichText::new(text)
                };
                let r = ui.add(egui::Label::new(rich).sense(Sense::click()).truncate());
                if r.clicked() {
                    if let Some(t) = l.tick {
                        out.push(Intent::Cursor(Cursor::At(t)));
                    }
                }
            }
        });
}
