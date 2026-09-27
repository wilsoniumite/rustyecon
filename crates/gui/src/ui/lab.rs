//! The oracle lab (docs/GUI.md §9, G1): pick a unit and a preset, edit the instance's numbers,
//! and see the oracle's regime and outputs beside their goldens; plot any field of the price and
//! quantity block over x in [0, 1], f(x) = n_D − n_S by default, with its root and bracket; and
//! sweep one knob.
//!
//! The lab needs no tape and drives no run: its form is the panels' own state, nothing of it
//! reaches the model, and every number it paints is the oracle's (origin "oracle"), a golden as
//! the generator wrote it, or their difference. An output is painted as the oracle's dump
//! prints it, the shortest digits that read back as its double, so what is on screen is the
//! oracle's number bit for bit.

use super::fmt;
use crate::lab::presets::{self, Preset};
use crate::lab::{fields, knobs, Instance, OracleUnit};
use crate::vm::lab::{self as vm, CurveVm, GoldenCellVm, LabVm, SweepVm};
use egui::{Color32, RichText};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use std::collections::BTreeMap;

/// The points of a field's curve over [0, 1].
pub const CURVE_POINTS: usize = 400;

/// The lab's state between frames.
pub struct LabState {
    unit: OracleUnit,
    preset: usize,
    inst: Instance,
    /// Each knob's text while it is being edited, and why a text did not read.
    texts: BTreeMap<String, String>,
    errors: BTreeMap<String, String>,
    filter: String,
    vm: Option<LabVm>,
    field: String,
    curve: Option<CurveVm>,
    sweep_knob: String,
    sweep_from: String,
    sweep_to: String,
    sweep_points: String,
    sweep_outputs: String,
    sweep: Option<Result<SweepVm, String>>,
}

impl Default for LabState {
    /// Unit 1a at its first preset, the SSRN Appendix B instance.
    fn default() -> LabState {
        let unit = OracleUnit::U1a;
        let inst = presets::of(unit)
            .first()
            .map_or_else(|| Instance::A(presets::appendix_b()), Preset::instance);
        let mut st = LabState {
            unit,
            preset: 0,
            inst,
            texts: BTreeMap::new(),
            errors: BTreeMap::new(),
            filter: String::new(),
            vm: None,
            field: fields::F.to_string(),
            curve: None,
            sweep_knob: String::new(),
            sweep_from: String::new(),
            sweep_to: String::new(),
            sweep_points: "200".to_string(),
            sweep_outputs: "x_star, v".to_string(),
            sweep: None,
        };
        st.sweep_defaults();
        st
    }
}

impl LabState {
    /// The preset shown, if the unit has one at that place.
    pub fn preset(&self) -> Option<Preset> {
        presets::of(self.unit).get(self.preset).copied()
    }

    /// The instance in the form.
    pub fn instance(&self) -> &Instance {
        &self.inst
    }

    /// The view-model shown, once made.
    pub fn view(&self) -> Option<&LabVm> {
        self.vm.as_ref()
    }

    /// The last sweep, or why it did not run.
    pub fn swept(&self) -> Option<&Result<SweepVm, String>> {
        self.sweep.as_ref()
    }

    /// The field plotted over x, once made.
    pub fn curve(&self) -> Option<&CurveVm> {
        self.curve.as_ref()
    }

    /// Show a preset of a unit: its instance, afresh.
    pub fn choose(&mut self, unit: OracleUnit, preset: usize) {
        self.unit = unit;
        self.preset = preset;
        if let Some(p) = self.preset() {
            self.inst = p.instance();
        }
        self.texts.clear();
        self.errors.clear();
        self.sweep = None;
        self.sweep_defaults();
        self.stale();
    }

    fn stale(&mut self) {
        self.vm = None;
        self.curve = None;
    }

    /// The sweep's defaults: the first knob, ±50% about its value.
    fn sweep_defaults(&mut self) {
        let ks = knobs::list(&self.inst);
        if let Some(k) = ks
            .iter()
            .find(|k| k.path == "workers")
            .or_else(|| ks.first())
        {
            self.sweep_knob = k.path.clone();
            self.sweep_from = format!("{:?}", k.value * 0.5);
            self.sweep_to = format!("{:?}", k.value * 1.5);
        }
    }
}

fn build_string() -> String {
    let b = crate::build();
    if b.dirty {
        format!("{} (dirty)", b.commit)
    } else {
        b.commit
    }
}

/// A one-line field labelled by `label`, which names it for a screen reader and a script.
fn field(ui: &mut egui::Ui, label: &str, text: &mut String, width: f32) -> egui::Response {
    let l = ui.label(label);
    ui.add(egui::TextEdit::singleline(text).desired_width(width))
        .labelled_by(l.id)
}

fn golden_cell(ui: &mut egui::Ui, g: Option<&GoldenCellVm>) {
    match g {
        None => {
            ui.label("");
            ui.label("");
        }
        Some(g) => {
            ui.label(&g.text)
                .on_hover_text(format!("{}: {}", g.key, g.note));
            let what = if g.absolute { "abs" } else { "rel" };
            let text = match g.diff {
                Some(d) => format!("{what} {d:.1e}"),
                None if g.agrees => "equal".to_string(),
                None => "differs".to_string(),
            };
            if g.agrees {
                ui.weak(text);
            } else {
                ui.colored_label(ui.visuals().error_fg_color, text);
            }
        }
    }
}

/// Draw the lab.
pub fn show(ui: &mut egui::Ui, st: &mut LabState) {
    egui::ScrollArea::vertical()
        .id_salt("lab")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            header(ui, st);
            ui.separator();
            egui::CollapsingHeader::new("Instance")
                .id_salt("lab-instance")
                .default_open(false)
                .show(ui, |ui| form(ui, st));
            if st.vm.is_none() {
                let preset = st.preset();
                st.vm = Some(vm::build(preset.as_ref(), &st.inst, &build_string()));
            }
            egui::CollapsingHeader::new("Solve")
                .id_salt("lab-solve")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(v) = &st.vm {
                        solved(ui, v);
                    }
                });
            egui::CollapsingHeader::new("A field over x")
                .id_salt("lab-curve")
                .default_open(true)
                .show(ui, |ui| curve(ui, st));
            egui::CollapsingHeader::new("Sweep")
                .id_salt("lab-sweep")
                .default_open(true)
                .show(ui, |ui| sweep(ui, st));
        });
}

fn header(ui: &mut egui::Ui, st: &mut LabState) {
    let mut chosen = None;
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_label("unit")
            .selected_text(format!("{} {}", st.unit, st.unit.title()))
            .show_ui(ui, |ui| {
                for u in OracleUnit::ALL {
                    if ui
                        .selectable_label(u == st.unit, format!("{u} {}", u.title()))
                        .clicked()
                    {
                        chosen = Some((u, 0));
                    }
                }
            });
        let list = presets::of(st.unit);
        let current = list
            .get(st.preset)
            .map_or_else(String::new, |p| p.id.to_string());
        egui::ComboBox::from_label("preset")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (i, p) in list.iter().enumerate() {
                    if ui
                        .selectable_label(i == st.preset, format!("{}: {}", p.id, p.title))
                        .clicked()
                    {
                        chosen = Some((st.unit, i));
                    }
                }
            });
        if ui
            .button("Reset to preset")
            .on_hover_text("the preset's own instance, every edit dropped")
            .clicked()
        {
            chosen = Some((st.unit, st.preset));
        }
    });
    if let Some((u, i)) = chosen {
        st.choose(u, i);
    }
}

fn form(ui: &mut egui::Ui, st: &mut LabState) {
    ui.weak(
        "every number of the instance, by its path; Enter or leaving a field sets it, and the \
         oracle validates it when it solves. The structure (categories, types, parcels, access, \
         exit forms, the budget's rule) is the preset's.",
    );
    ui.horizontal(|ui| {
        field(ui, "filter the knobs", &mut st.filter, 200.0);
    });
    let needle = st.filter.trim().to_lowercase();
    let mut set: Vec<(String, String)> = Vec::new();
    egui::Grid::new("lab-knobs")
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            for k in knobs::list(&st.inst) {
                if !needle.is_empty() && !k.path.to_lowercase().contains(&needle) {
                    continue;
                }
                let text = st.texts.entry(k.path.clone()).or_insert_with(|| {
                    if k.whole {
                        format!("{}", k.value)
                    } else {
                        format!("{:?}", k.value)
                    }
                });
                let r = field(ui, &k.path, text, 180.0);
                let entered = r.lost_focus()
                    || (r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if entered {
                    set.push((k.path.clone(), text.clone()));
                }
                match st.errors.get(&k.path) {
                    Some(e) => {
                        ui.colored_label(ui.visuals().error_fg_color, e);
                    }
                    None => {
                        ui.label("");
                    }
                }
                ui.end_row();
            }
        });
    for (path, text) in set {
        let before = st.inst.clone();
        match knobs::set(&mut st.inst, &path, &text) {
            Ok(()) => {
                st.errors.remove(&path);
                if st.inst != before {
                    st.sweep = None;
                    st.stale();
                }
            }
            Err(e) => {
                st.errors.insert(path, e);
            }
        }
    }
}

fn solved(ui: &mut egui::Ui, v: &LabVm) {
    let preset = v.preset.as_deref().unwrap_or("no preset");
    let edited = if v.edited { ", edited" } else { "" };
    ui.label(format!(
        "unit {} ({}), {preset}{edited}: {}",
        v.unit, v.unit_title, v.title
    ));
    if let Some(h) = &v.heading {
        ui.weak(format!("{}: {h}", v.file.as_deref().unwrap_or("")));
    }
    ui.label(format!("origin {}, build {}", v.origin, v.build));
    ui.horizontal_wrapped(|ui| {
        ui.strong(format!("regime {}", v.regime));
        for (k, d) in &v.detail {
            ui.label(format!("{k} {d}"));
        }
    });
    if v.edited {
        ui.weak("edited from its preset: its goldens are another economy's, and none is shown");
    } else if v.file.is_some() {
        ui.weak(format!(
            "{} outputs paired with a golden of the same name, {} within 1e-12 relative; \
             published values within 5e-6 absolute",
            v.paired.0, v.paired.1
        ));
    }
    egui::Grid::new("lab-outputs")
        .num_columns(6)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("output");
            ui.strong("oracle");
            ui.strong("golden");
            ui.strong("");
            ui.strong("published");
            ui.strong("");
            ui.end_row();
            for o in &v.outputs {
                ui.label(&o.key);
                ui.label(RichText::new(&o.value).monospace())
                    .on_hover_text(o.number.map_or_else(String::new, fmt));
                golden_cell(ui, o.golden.as_ref());
                golden_cell(ui, o.published.as_ref());
                ui.end_row();
            }
        });
    if !v.unpaired.is_empty() {
        egui::CollapsingHeader::new(format!(
            "{} goldens of this instance that no output of that name pairs with",
            v.unpaired.len()
        ))
        .id_salt("lab-unpaired")
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new("lab-unpaired-grid")
                .num_columns(2)
                .show(ui, |ui| {
                    for g in &v.unpaired {
                        ui.label(&g.key);
                        ui.label(&g.text).on_hover_text(&g.note);
                        ui.end_row();
                    }
                });
        });
    }
}

fn curve(ui: &mut egui::Ui, st: &mut LabState) {
    if st.curve.is_none() {
        st.curve = Some(vm::curve(&st.inst, &st.field, CURVE_POINTS));
    }
    let Some(c) = st.curve.clone() else {
        return;
    };
    let mut pick = None;
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_label("field")
            .selected_text(&c.field)
            .show_ui(ui, |ui| {
                for f in &c.fields {
                    if ui.selectable_label(*f == c.field, f).clicked() {
                        pick = Some(f.clone());
                    }
                }
            });
        let root = c.root.map_or_else(
            || "no root: not an equilibrium on the line".to_string(),
            |r| format!("x* {r:?}"),
        );
        ui.label(format!(
            "{root}; bracket [{:e}, {}]; regime {}",
            c.bracket.0, c.bracket.1, c.regime
        ));
    });
    if let Some(f) = pick {
        st.field = f;
        st.curve = None;
        return;
    }
    if let Some(e) = &c.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
        return;
    }
    ui.weak(format!(
        "{} at the bracket's ends: {} and {}{}",
        c.field,
        c.at_ends.0.map_or_else(|| "–".to_string(), fmt),
        c.at_ends.1.map_or_else(|| "–".to_string(), fmt),
        c.at_root
            .map_or_else(String::new, |v| format!("; at x* {}", fmt(v)))
    ));
    Plot::new("lab-curve-plot")
        .height(240.0)
        .legend(Legend::default())
        .x_axis_label("x")
        .y_axis_label(c.field.clone())
        .show(ui, |pui| {
            for s in &c.segments {
                pui.line(Line::new(c.field.clone(), PlotPoints::from(s.clone())));
            }
            if c.field == fields::F {
                pui.hline(HLine::new("0", 0.0).color(Color32::GRAY));
            }
            pui.vline(VLine::new("bracket", c.bracket.0).color(Color32::DARK_GRAY));
            pui.vline(VLine::new("bracket", c.bracket.1).color(Color32::DARK_GRAY));
            if let Some(r) = c.root {
                pui.vline(VLine::new("x*", r).color(Color32::from_rgb(0xf5, 0x85, 0x18)));
            }
        });
}

/// A sweep's line: the knob, its range, and how many points each regime took.
pub fn sweep_line(s: &SweepVm) -> String {
    let counts: Vec<String> = s.counts.iter().map(|(r, n)| format!("{r} {n}")).collect();
    format!(
        "{} over {} points, {} to {}: {}",
        s.knob,
        s.xs.len(),
        s.xs.first().map_or(0.0, |x| *x),
        s.xs.last().map_or(0.0, |x| *x),
        counts.join(", ")
    )
}

fn sweep(ui: &mut egui::Ui, st: &mut LabState) {
    let ks = knobs::list(&st.inst);
    let mut run = false;
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_label("knob to sweep")
            .selected_text(&st.sweep_knob)
            .show_ui(ui, |ui| {
                for k in &ks {
                    if ui
                        .selectable_label(k.path == st.sweep_knob, &k.path)
                        .clicked()
                    {
                        st.sweep_knob = k.path.clone();
                        st.sweep_from = format!("{:?}", k.value * 0.5);
                        st.sweep_to = format!("{:?}", k.value * 1.5);
                    }
                }
            });
        field(ui, "from", &mut st.sweep_from, 90.0);
        field(ui, "to", &mut st.sweep_to, 90.0);
        field(ui, "points", &mut st.sweep_points, 50.0);
    });
    ui.horizontal_wrapped(|ui| {
        field(ui, "outputs", &mut st.sweep_outputs, 260.0);
        run = ui.button("Run the sweep").clicked();
    });
    if run {
        let num = |t: &str, what: &str| {
            t.trim()
                .parse::<f64>()
                .map_err(|_| format!("{what} {t:?} is not a number"))
        };
        let outputs: Vec<String> = st
            .sweep_outputs
            .split([',', ' '])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        st.sweep = Some((|| {
            let from = num(&st.sweep_from, "from")?;
            let to = num(&st.sweep_to, "to")?;
            let points: usize = st
                .sweep_points
                .trim()
                .parse()
                .map_err(|_| format!("points {:?} is not a count", st.sweep_points))?;
            if !(2..=10_000).contains(&points) {
                return Err(format!("{points} points: from 2 to 10,000"));
            }
            vm::sweep(&st.inst, &st.sweep_knob, from, to, points, &outputs)
        })());
    }
    match &st.sweep {
        None => {
            ui.weak("choose a knob, its range and the outputs to read, then Run the sweep");
        }
        Some(Err(e)) => {
            ui.colored_label(ui.visuals().error_fg_color, e);
        }
        Some(Ok(s)) => {
            ui.label(sweep_line(s));
            Plot::new("lab-sweep-plot")
                .height(240.0)
                .legend(Legend::default())
                .x_axis_label(s.knob.clone())
                .show(ui, |pui| {
                    for (i, l) in s.lines.iter().enumerate() {
                        let colour = super::plots::PALETTE[i % super::plots::PALETTE.len()];
                        // A point with no equilibrium, or no number, breaks the line.
                        let mut segments: Vec<Vec<[f64; 2]>> = vec![Vec::new()];
                        for (x, v) in s.xs.iter().zip(&l.values) {
                            match v.filter(|v| v.is_finite()) {
                                Some(v) => {
                                    if let Some(seg) = segments.last_mut() {
                                        seg.push([*x, v]);
                                    }
                                }
                                None => segments.push(Vec::new()),
                            }
                        }
                        for seg in segments.into_iter().filter(|s| !s.is_empty()) {
                            pui.line(Line::new(l.key.clone(), PlotPoints::from(seg)).color(colour));
                        }
                    }
                });
        }
    }
}
