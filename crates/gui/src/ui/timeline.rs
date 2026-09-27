//! The timeline (docs/GUI.md §4): a scrubber over the run's span with the cursor, each year's
//! first tick, the fired and the scheduled events, and the ring checkpoints. A click or a drag
//! on the strip moves the cursor; "Live" makes it follow the run again. Below the strip, the
//! events are listed with their dates.

use crate::model::{Cursor, Intent};
use crate::vm::timeline::TimelineVm;
use egui::{pos2, vec2, Align2, Color32, FontId, Sense, Stroke};

/// Draw the timeline.
pub fn show(ui: &mut egui::Ui, vm: &TimelineVm, out: &mut Vec<Intent>) {
    let fired = vm.events.iter().filter(|e| e.fired).count();
    ui.horizontal(|ui| {
        match &vm.cursor {
            Some(c) => ui.label(format!(
                "cursor {} (tick {}){}",
                c.date,
                c.tick,
                if c.live { ", live" } else { "" }
            )),
            None => ui.weak("no tick has run"),
        };
        let live = vm.cursor.as_ref().is_none_or(|c| c.live);
        if ui.add_enabled(!live, egui::Button::new("Live")).clicked() {
            out.push(Intent::Cursor(Cursor::Live));
        }
        ui.weak(format!(
            "{fired} fired · {} scheduled · {} ring checkpoints",
            vm.events.len() - fired,
            vm.ring.len()
        ));
    });
    let width = ui.available_width().max(100.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 72.0), Sense::click_and_drag());
    let span = (vm.end - vm.start).max(1) as f32;
    let x_of = |t: u64| rect.left() + (t.saturating_sub(vm.start)) as f32 / span * rect.width();
    let p = ui.painter_at(rect);
    let v = ui.visuals();
    let fg = v.text_color();
    let weak = v.weak_text_color();
    let mid = rect.top() + 30.0;
    // The span recorded, and the span to come.
    p.rect_filled(
        egui::Rect::from_min_max(
            pos2(x_of(vm.start), mid - 3.0),
            pos2(x_of(vm.now), mid + 3.0),
        ),
        1.0,
        weak,
    );
    p.line_segment(
        [pos2(rect.left(), mid), pos2(rect.right(), mid)],
        Stroke::new(1.0, weak),
    );
    // Years: a tick mark each, a label where there is room.
    let mut last_label = f32::NEG_INFINITY;
    for y in &vm.years {
        let x = x_of(y.tick);
        p.line_segment(
            [pos2(x, mid - 6.0), pos2(x, mid + 6.0)],
            Stroke::new(1.0, weak),
        );
        if x - last_label > 44.0 {
            p.text(
                pos2(x, rect.top() + 2.0),
                Align2::CENTER_TOP,
                y.year.to_string(),
                FontId::proportional(11.0),
                weak,
            );
            last_label = x;
        }
    }
    // Ring checkpoints below the line.
    for &t in &vm.ring {
        let x = x_of(t);
        p.rect_filled(
            egui::Rect::from_center_size(pos2(x, mid + 14.0), vec2(4.0, 4.0)),
            0.0,
            fg,
        );
    }
    // Events above it: filled once fired, hollow while scheduled.
    let mut hovered = None;
    let pointer = resp.hover_pos();
    for e in &vm.events {
        let c = pos2(x_of(e.tick), mid - 12.0);
        if e.fired {
            p.circle_filled(c, 4.0, fg);
        } else {
            p.circle_stroke(c, 4.0, Stroke::new(1.5, fg));
        }
        if pointer.is_some_and(|q| (q.x - c.x).abs() < 6.0 && (q.y - c.y).abs() < 8.0) {
            hovered = Some(e);
        }
    }
    // The cursor.
    if let Some(c) = &vm.cursor {
        let x = x_of(c.tick);
        p.line_segment(
            [pos2(x, rect.top() + 14.0), pos2(x, rect.bottom())],
            Stroke::new(2.0, Color32::from_rgb(0x4c, 0x78, 0xa8)),
        );
    }
    if let Some(e) = hovered {
        resp.clone().on_hover_text(format!(
            "{} {} (tick {}{}): {}{}",
            e.key,
            e.date,
            e.tick,
            if e.occurrence > 0 {
                format!(", occurrence {}", e.occurrence)
            } else {
                String::new()
            },
            e.what,
            if e.fired { "" } else { " — scheduled" }
        ));
    }
    if resp.clicked() || resp.dragged() {
        if let Some(q) = resp.interact_pointer_pos() {
            let f = ((q.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let t = vm.start + (f * span).round() as u64;
            if vm.now > vm.start {
                out.push(Intent::Cursor(Cursor::At(t.min(vm.now - 1))));
            }
        }
    }
    egui::ScrollArea::vertical()
        .id_salt("timeline-events")
        .show(ui, |ui| {
            egui::Grid::new("timeline-events-grid")
                .striped(true)
                .show(ui, |ui| {
                    for e in &vm.events {
                        let label = if e.fired { "fired" } else { "scheduled" };
                        if ui.link(&e.date).clicked() && e.tick < vm.now {
                            out.push(Intent::Cursor(Cursor::At(e.tick)));
                        }
                        ui.label(format!("tick {}", e.tick));
                        ui.label(e.key.to_string());
                        ui.label(&e.what);
                        ui.weak(label);
                        ui.end_row();
                    }
                });
        });
}
