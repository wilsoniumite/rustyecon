//! The timeline (docs/GUI.md §4): a scrubber over the run's span with the cursor, each year's
//! first tick, the fired and the scheduled events, and the ring checkpoints. A click or a drag
//! on the strip moves the cursor; "Live" makes it follow the run again. Below the strip, the
//! events are listed with their dates.
//!
//! As built at D.3: a schedule too long to list whole (the view-model's `density`) is drawn as a
//! bar a year on the strip, filled for the firings that ran and outlined for those to come, and
//! its list of the firings near the cursor lays out only the rows on screen.

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
        match vm.total {
            None => ui.weak(format!(
                "{fired} fired · {} scheduled · {} ring checkpoints",
                vm.events.len() - fired,
                vm.ring.len()
            )),
            Some(total) => {
                let done: u32 = vm.density.iter().map(|y| y.fired).sum();
                ui.weak(format!(
                    "{done} fired · {} scheduled · {} ring checkpoints; {total} firings, counted \
                     a year at a time, and the {} within a year of the cursor listed",
                    total.saturating_sub(done as usize),
                    vm.ring.len(),
                    vm.events.len()
                ))
            }
        };
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
    // A long schedule's firings, a bar a year: filled once fired, outlined while scheduled.
    let most = vm
        .density
        .iter()
        .map(|y| y.fired + y.scheduled)
        .max()
        .unwrap_or(0)
        .max(1);
    for (k, y) in vm.density.iter().enumerate() {
        let x0 = x_of(y.tick);
        let x1 = vm.density.get(k + 1).map_or(x_of(vm.end), |n| x_of(n.tick));
        let h = |n: u32| 16.0 * n as f32 / most as f32;
        let bottom = mid - 5.0;
        let fired = egui::Rect::from_min_max(
            pos2(x0, bottom - h(y.fired)),
            pos2(x1.max(x0 + 1.0), bottom),
        );
        p.rect_filled(fired, 0.0, weak);
        if y.scheduled > 0 {
            let top = bottom - h(y.fired + y.scheduled);
            let sched = egui::Rect::from_min_max(
                pos2(x0, top),
                pos2(x1.max(x0 + 1.0), bottom - h(y.fired)),
            );
            p.rect_stroke(sched, 0.0, Stroke::new(0.5, weak), egui::StrokeKind::Inside);
        }
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
    if !vm.density.is_empty() {
        // A long list: only the rows on screen are laid out.
        let row = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        egui::ScrollArea::vertical()
            .id_salt("timeline-events")
            .show_rows(ui, row, vm.events.len(), |ui, range| {
                for e in &vm.events[range] {
                    ui.horizontal(|ui| {
                        if ui.link(&e.date).clicked() && e.tick < vm.now {
                            out.push(Intent::Cursor(Cursor::At(e.tick)));
                        }
                        ui.label(format!("tick {}", e.tick));
                        ui.label(e.key.to_string());
                        ui.label(&e.what);
                        ui.weak(if e.fired { "fired" } else { "scheduled" });
                    });
                }
            });
        return;
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
