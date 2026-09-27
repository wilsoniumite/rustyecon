//! Compare (docs/GUI.md §4): a branch against its parent. The two identities, the first
//! differing hash, the tape diff by key, the lineage, and the difference of the plotted
//! series at the cursor and at its largest.

use super::fmt;
use crate::vm::compare::{Change, CompareVm, IdentityVm};
use egui::RichText;

fn identity(ui: &mut egui::Ui, who: &str, id: &IdentityVm) {
    let commit: String = id.commit.chars().take(10).collect();
    let state = if id.dirty { "dirty" } else { "clean" };
    ui.label(format!(
        "{who}: {} · {commit} {state} · world {} · tape {} · state ticks {} to {}",
        id.name, id.world_id, id.tape_hash, id.start, id.tick
    ));
}

/// Draw the compare panel.
pub fn show(ui: &mut egui::Ui, vm: &CompareVm) {
    egui::ScrollArea::vertical()
        .id_salt("compare-scroll")
        .show(ui, |ui| {
            identity(ui, "parent", &vm.parent);
            identity(ui, "branch", &vm.child);
            let (lo, hi) = vm.compared;
            match &vm.first_difference {
                Some(d) => {
                    let report = d.report_tick.map_or_else(String::new, |t| {
                        format!(", left by report tick {t} ({})", d.date)
                    });
                    ui.label(format!(
                        "first differing hash: state tick {}{report}: parent {}, branch {}",
                        d.state_tick, d.parent, d.child
                    ));
                }
                None if lo <= hi => {
                    ui.label(format!("no hash differs in state ticks {lo} to {hi}"));
                }
                None => {
                    ui.weak("no tick recorded by both yet");
                }
            }
            if vm.before_resume {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!(
                        "the hashes differ at or before state tick {}, where the branch \
                         started: its start is not the parent's state",
                        vm.child.start
                    ),
                );
            }
            ui.separator();
            ui.label(RichText::new("Lineage").strong());
            if vm.lineage.is_empty() {
                ui.weak("no lineage");
            }
            for l in &vm.lineage {
                ui.label(l);
            }
            ui.separator();
            ui.label(RichText::new(format!("Tape diff ({})", vm.tape.len())).strong());
            for l in &vm.tape {
                let change = match l.change {
                    Change::Added => "added",
                    Change::Removed => "removed",
                    Change::Changed => "changed",
                };
                ui.label(format!("{} {} {change}", l.section, l.key));
                if let Some(b) = &l.before {
                    ui.weak(RichText::new(format!("- {b}")).monospace());
                }
                if let Some(a) = &l.after {
                    ui.label(RichText::new(format!("+ {a}")).monospace());
                }
            }
            ui.separator();
            let at = vm
                .cursor
                .map_or_else(|| "no tick yet".to_string(), |t| format!("tick {t}"));
            ui.label(
                RichText::new(format!("Plotted series, branch minus parent, at {at}")).strong(),
            );
            let v = |x: Option<f64>| x.map_or_else(|| "–".to_string(), fmt);
            egui::Grid::new("compare-series")
                .striped(true)
                .num_columns(6)
                .show(ui, |ui| {
                    for h in [
                        "series",
                        "parent",
                        "branch",
                        "difference",
                        "largest",
                        "first",
                    ] {
                        ui.strong(h);
                    }
                    ui.end_row();
                    for s in &vm.series {
                        let (p, c, d) = s.at_cursor;
                        ui.label(format!("{} ({})", s.label, s.unit));
                        ui.label(v(p));
                        ui.label(v(c));
                        ui.label(v(d));
                        ui.label(s.largest.map_or_else(
                            || "none".to_string(),
                            |(t, d)| format!("{} at tick {t}", fmt(d)),
                        ));
                        ui.label(
                            s.first_differs
                                .map_or_else(|| "–".to_string(), |t| format!("tick {t}")),
                        );
                        ui.end_row();
                    }
                });
        });
}
