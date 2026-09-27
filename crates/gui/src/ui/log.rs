//! The log (docs/GUI.md §4): loads, runs, pauses, fired events, refusals and errors, oldest
//! first, and the breakpoint on error. A line with a tick moves the cursor there.

use crate::model::{Cursor, Intent};
use crate::vm::log::LogVm;
use egui::{RichText, Sense};

/// Draw the log.
pub fn show(ui: &mut egui::Ui, vm: &LogVm, break_on_error: bool, out: &mut Vec<Intent>) {
    ui.horizontal(|ui| {
        let mut on = break_on_error;
        if ui
            .checkbox(&mut on, "Break on error")
            .on_hover_text("pause the run on a run error, and say it was this breakpoint")
            .changed()
        {
            out.push(Intent::BreakOnError(on));
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
