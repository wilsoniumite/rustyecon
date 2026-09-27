//! The editor (docs/GUI.md §5.1): a form that reads into a tape edit, the edits staged for
//! the focused run, Apply, which makes a branch of it, and the run's files: "Save tape as…"
//! and export.
//!
//! The form checks first (`edit::form`): an empty note, a malformed key or date, a key the run
//! tree has, a ledger tolerance, and an act that does not read are refused, and the refusal is
//! painted. The raw pane beside the act is read-only: it shows the act's text as the form read
//! it, and where the parser stopped, by line and column. A removal that leaves a param
//! unreferenced offers the `RemoveParam`, with a note of its own. Nothing here changes the
//! model: every act is an [`Intent`].

use crate::edit::{Form, OpKind};
use crate::model::{Intent, Model};
use egui::{Button, RichText, TextEdit};

/// The editor's text between frames.
#[derive(Debug, Default)]
pub struct EditorState {
    /// The form.
    pub form: Form,
    /// The note for the `RemoveParam`s an orphan offer adds.
    orphan_note: String,
    /// Where "Save tape as" writes.
    save_path: String,
    /// Where an export goes.
    export_dir: String,
}

/// A labelled one-line field; the label names it for a screen reader and a script.
fn field(ui: &mut egui::Ui, label: &str, text: &mut String, hint: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let l = ui.label(label);
        ui.add(
            TextEdit::singleline(text)
                .desired_width(320.0)
                .hint_text(hint),
        )
        .labelled_by(l.id)
    })
    .inner
}

/// Draw the editor of the focused run.
pub fn show(ui: &mut egui::Ui, m: &Model, st: &mut EditorState, out: &mut Vec<Intent>) {
    let Some(run) = m.focused() else {
        return;
    };
    let ed = m.editor();
    egui::ScrollArea::vertical()
        .id_salt("editor-scroll")
        .show(ui, |ui| {
            let name = run
                .store
                .world()
                .map_or_else(|| run.tape.header.name.clone(), |w| w.name.clone());
            ui.strong(format!("Branch {} ({name})", run.id));
            ui.weak(
                "each edit carries a note; every entry it adds or changes is stamped \
                 Assumed(\"GUI experiment <date>: <note>\")",
            );
            egui::ComboBox::new("edit-kind", "edit kind")
                .selected_text(st.form.kind.name())
                .show_ui(ui, |ui| {
                    for k in OpKind::ALL {
                        if ui.selectable_label(st.form.kind == k, k.name()).clicked() {
                            st.form.kind = k;
                        }
                    }
                });
            let f = st.form.kind.fields();
            ui.horizontal(|ui| {
                field(ui, "edit key", &mut st.form.key, "gui.1.1, or mint one");
                if st.form.kind.adds() && ui.button("Mint key").clicked() {
                    if let Some(k) = m.mint_key() {
                        st.form.key = k.to_string();
                    }
                }
            });
            if f.date {
                ui.horizontal(|ui| {
                    field(ui, "edit date", &mut st.form.date, "YYYY-MM-DD");
                    // In a live run, "from now" is the next unrun tick's date.
                    let now = run
                        .store
                        .world()
                        .and_then(|w| w.clock.date_of(run.store.tick()));
                    if let Some(d) = now {
                        if ui
                            .button("now")
                            .on_hover_text("the first date of the next tick to run")
                            .clicked()
                        {
                            st.form.date = d.to_string();
                        }
                    }
                });
            }
            if f.recurring {
                field(
                    ui,
                    "edit every",
                    &mut st.form.every,
                    "a param of unit Years",
                );
                field(ui, "edit last", &mut st.form.last, "YYYY-MM-DD, or empty");
            }
            if f.value {
                field(
                    ui,
                    "edit value",
                    &mut st.form.value,
                    "a number, in its unit",
                );
            }
            if f.unit {
                field(ui, "edit unit", &mut st.form.unit, "FlowPerYear, Years, …");
            }
            if f.act {
                ui.horizontal(|ui| {
                    let l = ui.label("edit act");
                    ui.add(
                        TextEdit::multiline(&mut st.form.act)
                            .code_editor()
                            .desired_rows(2)
                            .desired_width(420.0)
                            .hint_text(
                                "SetParam(param: \"mine.capacity\", to: \"mine.capacity.base\")",
                            ),
                    )
                    .labelled_by(l.id);
                });
            }
            field(ui, "edit note", &mut st.form.note, "why: required");
            if ui.button("Add edit").clicked() {
                out.push(Intent::Stage(st.form.clone()));
            }
            if let Some(e) = &ed.error {
                ui.colored_label(ui.visuals().error_fg_color, e);
            }
            raw_pane(ui, m, st);
            ui.separator();
            staged(ui, m, st, out);
            ui.separator();
            files(ui, m, st, out);
        });
}

/// The raw pane: the act as the form read it, read-only, with the parser's line and column.
fn raw_pane(ui: &mut egui::Ui, m: &Model, st: &EditorState) {
    ui.label(RichText::new("Raw (read-only)").strong());
    let raw = m.editor().raw.as_ref();
    let text = raw.map_or(st.form.act.as_str(), |r| r.text.as_str());
    egui::Frame::group(ui.style()).show(ui, |ui| {
        for (i, line) in text.lines().enumerate() {
            ui.monospace(format!("{:>3} | {line}", i + 1));
            if let Some(r) = raw.filter(|r| r.line == i + 1) {
                let caret = " ".repeat(r.col.saturating_sub(1));
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    RichText::new(format!("    | {caret}^")).monospace(),
                );
            }
        }
        if let Some(r) = raw {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("line {}, column {}: {}", r.line, r.col, r.message),
            );
        }
    });
}

/// The staged edits, the orphan offer and Apply.
fn staged(ui: &mut egui::Ui, m: &Model, st: &mut EditorState, out: &mut Vec<Intent>) {
    let ed = m.editor();
    ui.label(RichText::new(format!("Staged edits ({})", ed.staged.len())).strong());
    for (i, e) in ed.staged.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}. {} ({:?})", i + 1, e.op, e.note));
            if ui.small_button("drop").clicked() {
                out.push(Intent::Unstage(i));
            }
        });
    }
    if !ed.offer.is_empty() {
        let keys: Vec<&str> = ed.offer.iter().map(|k| k.as_str()).collect();
        ui.label(format!(
            "The removal leaves {} unreferenced, which does not load. Remove {} too?",
            keys.join(", "),
            if keys.len() == 1 { "it" } else { "them" }
        ));
        field(ui, "orphan note", &mut st.orphan_note, "why: required");
        if ui.button("Remove it too").clicked() {
            out.push(Intent::RemoveOrphans {
                note: st.orphan_note.clone(),
            });
        }
    }
    let can = !ed.staged.is_empty() && m.today().is_some();
    if ui
        .add_enabled(can, Button::new("Apply"))
        .on_hover_text("materialise the staged edits: a branch of the focused run")
        .clicked()
    {
        out.push(Intent::Apply);
    }
}

/// A small "…" button that opens a native dialog; its hover text and its accessible name say
/// which.
fn dialog(ui: &mut egui::Ui, what: &str) -> egui::Response {
    let r = ui.button("…").on_hover_text(what);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, what));
    r
}

/// "Save tape as…" and export, of the focused run.
fn files(ui: &mut egui::Ui, m: &Model, st: &mut EditorState, out: &mut Vec<Intent>) {
    let Some(run) = m.focused() else {
        return;
    };
    ui.label(RichText::new(format!("Files of {}", run.id)).strong());
    if let Some(p) = &run.path {
        ui.weak(format!("on disk: {p}"));
    }
    ui.horizontal(|ui| {
        field(ui, "save path", &mut st.save_path, "a new file, <name>.ron");
        if ui.button("Save tape as").clicked() {
            out.push(Intent::SaveTape(st.save_path.clone()));
        }
        if dialog(ui, "choose where to save the tape").clicked() {
            out.push(Intent::PickSaveTape);
        }
    });
    ui.horizontal(|ui| {
        field(ui, "export directory", &mut st.export_dir, "a directory");
        if ui.button("Export").clicked() {
            out.push(Intent::Export(st.export_dir.clone()));
        }
        if dialog(ui, "choose a directory to export into").clicked() {
            out.push(Intent::PickExportDir);
        }
    });
    ui.weak(format!(
        "an export is stamped \"{}, draft\": the plotted series as CSV, manifest.ron, the tapes \
         and the lineage",
        run.origin
    ));
}
