//! The inspector (docs/GUI.md §4): the selection in detail, at the cursor. Every number shows
//! its unit, and each one the run records has a "plot" toggle, so any series can be plotted
//! from here.

use super::{fmt, value_label};
use crate::model::Intent;
use crate::run::SeriesKey;
use crate::vm::inspector::{
    ActorVm, EventVm, InspectorVm, MarketVm, OtherVm, ParamRefVm, ParamVm, ValueVm,
};
use crate::vm::registry::CopiedVm;

/// Draw the inspector.
pub fn show(ui: &mut egui::Ui, vm: &InspectorVm, plots: &[SeriesKey], out: &mut Vec<Intent>) {
    egui::ScrollArea::both()
        .id_salt("inspector")
        .show(ui, |ui| match vm {
            InspectorVm::Market(m) => market(ui, m, plots, out),
            InspectorVm::Actor(a) => actor(ui, a, plots, out),
            InspectorVm::Param(p) => param(ui, p, plots, out),
            InspectorVm::Event(e) => event(ui, e),
            InspectorVm::Other(o) => other(ui, o, plots, out),
            InspectorVm::Unknown(e) => {
                ui.weak(format!("{e:?} is not in this run's world"));
            }
        });
}

fn at(ui: &mut egui::Ui, tick: Option<u64>, date: &str) {
    match tick {
        Some(t) => ui.label(format!("tick {t} ({date})")),
        None => ui.weak("no tick has run"),
    };
}

/// Rows of numbers: label, value and unit, and a plot toggle for a recorded series.
fn values(
    ui: &mut egui::Ui,
    id: &str,
    rows: &[ValueVm],
    plots: &[SeriesKey],
    out: &mut Vec<Intent>,
) {
    egui::Grid::new(("values", id))
        .num_columns(3)
        .show(ui, |ui| {
            for r in rows {
                ui.label(&r.label);
                value_label(ui, r.value, &r.unit);
                plot_toggle(ui, r.series.as_ref(), plots, out);
                ui.end_row();
            }
        });
}

fn plot_toggle(
    ui: &mut egui::Ui,
    s: Option<&SeriesKey>,
    plots: &[SeriesKey],
    out: &mut Vec<Intent>,
) {
    let Some(s) = s else {
        ui.label("");
        return;
    };
    super::plot_button(ui, s, plots.contains(s), out);
}

fn copied(ui: &mut egui::Ui, c: &Option<CopiedVm>) {
    if let Some(c) = c {
        ui.label(format!(
            "copied from {} by {} on {} (tick {}): {}",
            c.source, c.event, c.date, c.tick, c.basis
        ));
    }
}

fn param_refs(ui: &mut egui::Ui, id: &str, refs: &[ParamRefVm]) {
    egui::Grid::new(("params", id))
        .num_columns(5)
        .striped(true)
        .show(ui, |ui| {
            for r in refs {
                ui.label(&r.path);
                ui.label(r.key.to_string());
                value_label(ui, r.value, &r.unit);
                match r.per_tick {
                    Some(k) => ui
                        .label(format!("{} per tick ({})", fmt(k), r.method))
                        .on_hover_text(format!("{k}")),
                    None => ui.weak(&r.method),
                };
                ui.weak(&r.basis);
                ui.end_row();
                if r.copied.is_some() {
                    ui.label("");
                    ui.label("");
                    copied(ui, &r.copied);
                    ui.end_row();
                }
            }
        });
}

fn market(ui: &mut egui::Ui, m: &MarketVm, plots: &[SeriesKey], out: &mut Vec<Intent>) {
    ui.heading(format!("Market {}/{}", m.node, m.good));
    at(ui, m.tick, &m.date);
    values(ui, "market", &m.values, plots, out);
    ui.horizontal(|ui| {
        ui.label("ln(p′/p)");
        value_label(ui, m.log_step, "per tick");
    });
    ui.separator();
    ui.label(format!("rule {}, one-sided {}", m.rule, m.one_sided));
    let refs: Vec<ParamRefVm> = m
        .rate
        .iter()
        .chain(m.ema_time_constant.iter())
        .cloned()
        .collect();
    param_refs(ui, "market", &refs);
    ui.separator();
    ui.strong("Rationing by class");
    if m.rationing.is_empty() {
        ui.weak("no class had an order in this market this tick");
    }
    for r in &m.rationing {
        ui.label(format!("{} ({})", r.class, r.side));
        values(
            ui,
            &format!("ration {} {}", r.class, r.side),
            &r.values,
            plots,
            out,
        );
    }
}

fn actor(ui: &mut egui::Ui, a: &ActorVm, plots: &[SeriesKey], out: &mut Vec<Intent>) {
    ui.heading(format!("Actor {}", a.key));
    ui.label(format!(
        "{}, class {}, at {}, spec {}; {}",
        a.kind, a.class, a.home, a.spec, a.basis
    ));
    at(ui, a.tick, &a.date);
    match (&a.state, a.snapshot) {
        (Some(s), _) => ui.label(format!(
            "state {s} (snapshot of state tick {})",
            a.state_tick
        )),
        (None, true) => ui.weak(format!("no own state at state tick {}", a.state_tick)),
        (None, false) => ui.weak(format!(
            "no snapshot of state tick {} yet: pause the run to take one",
            a.state_tick
        )),
    };
    ui.separator();
    ui.strong("Spec: its params");
    param_refs(ui, &format!("actor {}", a.key), &a.params);
    if !a.inline.is_empty() {
        egui::Grid::new(("inline", a.key.as_str()))
            .num_columns(3)
            .show(ui, |ui| {
                for n in &a.inline {
                    ui.label(&n.path);
                    ui.label(fmt(n.value)).on_hover_text(format!("{}", n.value));
                    ui.weak(&n.basis);
                    ui.end_row();
                }
            });
    }
    ui.separator();
    ui.strong("Holdings after the tick");
    egui::Grid::new(("holdings", a.key.as_str()))
        .num_columns(3)
        .show(ui, |ui| {
            for h in &a.holdings {
                ui.label(h.good.to_string());
                value_label(ui, h.held.value, &h.held.unit);
                plot_toggle(ui, h.held.series.as_ref(), plots, out);
                ui.end_row();
                if let Some(lots) = &h.lots {
                    ui.label("");
                    let text: Vec<String> = lots
                        .iter()
                        .map(|l| match l.life {
                            Some(life) => format!("{} ({life} ticks left)", fmt(l.qty)),
                            None => fmt(l.qty),
                        })
                        .collect();
                    ui.weak(format!("lots: {}", text.join(", ")));
                    ui.end_row();
                }
            }
        });
    ui.separator();
    ui.strong("Settlements in the tick");
    if a.settlements.is_empty() {
        ui.weak("no order settled");
    }
    for s in &a.settlements {
        ui.label(format!("{} {}/{}", s.side, s.node, s.good));
        values(
            ui,
            &format!("settle {} {} {}", s.side, s.node, s.good),
            &s.values,
            plots,
            out,
        );
    }
}

fn param(ui: &mut egui::Ui, p: &ParamVm, plots: &[SeriesKey], out: &mut Vec<Intent>) {
    let r = &p.row;
    ui.heading(format!("Param {}", r.key));
    ui.label(format!(
        "{}, {}, genesis {} {}",
        r.unit,
        r.use_,
        fmt(r.genesis),
        r.unit
    ));
    ui.weak(&r.basis);
    values(ui, "param", std::slice::from_ref(&p.value), plots, out);
    copied(ui, &r.copied);
    ui.separator();
    ui.strong("Uses");
    egui::Grid::new("param-sites")
        .num_columns(3)
        .show(ui, |ui| {
            for s in &r.sites {
                ui.label(&s.path);
                ui.label(&s.method);
                let now = s
                    .per_tick_now
                    .map_or_else(String::new, |v| format!(", now {}", fmt(v)));
                ui.label(format!("{} per tick at genesis{now}", fmt(s.per_tick)));
                ui.end_row();
            }
        });
    for (title, refs) in [("Set by", &p.set_by), ("Copied by", &p.copied_by)] {
        if refs.is_empty() {
            continue;
        }
        ui.separator();
        ui.strong(title);
        for e in refs {
            let state = if e.fired { "fired" } else { "scheduled" };
            ui.label(format!(
                "{} {} (tick {}): {} — {state}",
                e.key, e.date, e.tick, e.what
            ));
        }
    }
}

fn event(ui: &mut egui::Ui, e: &EventVm) {
    ui.heading(format!("Event {}", e.key));
    ui.label(format!("{}, {} (tick {})", e.kind, e.date, e.tick));
    if let Some(every) = &e.every {
        ui.label(format!("every {every}"));
    }
    if let Some(last) = &e.last {
        ui.label(format!("last {last}"));
    }
    ui.label(&e.what);
    if let Some((k, b)) = &e.source {
        ui.label(format!("copies {k}: {b}"));
    }
    ui.weak(&e.basis);
    ui.separator();
    if e.fired.is_empty() {
        ui.weak("not fired by the cursor");
    } else {
        let ticks: Vec<String> = e.fired.iter().map(|(t, _)| t.to_string()).collect();
        ui.label(format!("fired in ticks {}", ticks.join(", ")));
    }
}

fn other(ui: &mut egui::Ui, o: &OtherVm, plots: &[SeriesKey], out: &mut Vec<Intent>) {
    ui.heading(&o.title);
    egui::Grid::new("other-facts")
        .num_columns(2)
        .show(ui, |ui| {
            for (k, v) in &o.facts {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            }
        });
    values(ui, "other", &o.values, plots, out);
}
