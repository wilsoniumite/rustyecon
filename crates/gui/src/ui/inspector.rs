//! The inspector (docs/GUI.md §4): the selection in detail, at the cursor. Every number shows
//! its unit, and each one the run records has a "plot" toggle, so any series can be plotted
//! from here.
//!
//! A market also shows why its price is what it is (G1): the tick's step recomputed by
//! markets' own `imbalance` and `next_price` from the tick's recorded inputs, beside the next
//! price the run recorded, and the log waterfall, ln(p_t/p_0) as Σ k·x a year at a time with
//! the residual and the events fired.

use super::charts::{self, Charts};
use super::{fmt, value_label};
use crate::model::Intent;
use crate::run::{Breakpoint, SeriesKey};
use crate::vm::inspector::{
    ActorVm, EventVm, InspectorVm, MarketVm, OtherVm, ParamRefVm, ParamVm, ValueVm,
};
use crate::vm::pricestep::{ExplainerVm, WaterfallVm};
use crate::vm::registry::CopiedVm;
use egui::Color32;
use egui_plot::{Bar, Legend, Plot};

/// What the inspector reads besides its view-model.
pub struct Ctx<'a> {
    /// The plotted series.
    pub plots: &'a [SeriesKey],
    /// The watched series.
    pub watch: &'a [SeriesKey],
    /// The session's breakpoints.
    pub breakpoints: &'a [Breakpoint],
    /// A market's log waterfall up to the cursor, or why there is none.
    pub waterfall: Option<&'a Result<WaterfallVm, String>>,
}

/// Draw the inspector.
pub fn show(
    ui: &mut egui::Ui,
    vm: &InspectorVm,
    ctx: &Ctx<'_>,
    charts: &mut Charts,
    out: &mut Vec<Intent>,
) {
    egui::ScrollArea::both()
        .id_salt("inspector")
        .show(ui, |ui| match vm {
            InspectorVm::Market(m) => market(ui, m, ctx, charts, out),
            InspectorVm::Actor(a) => actor(ui, a, ctx, out),
            InspectorVm::Param(p) => param(ui, p, ctx, out),
            InspectorVm::Event(e) => event(ui, e, ctx.breakpoints, out),
            InspectorVm::Other(o) => other(ui, o, ctx, out),
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

/// Rows of numbers: label, value and unit, and plot and watch toggles for a recorded series.
fn values(ui: &mut egui::Ui, id: &str, rows: &[ValueVm], ctx: &Ctx<'_>, out: &mut Vec<Intent>) {
    egui::Grid::new(("values", id))
        .num_columns(4)
        .show(ui, |ui| {
            for r in rows {
                ui.label(&r.label);
                value_label(ui, r.value, &r.unit);
                plot_toggle(ui, r.series.as_ref(), ctx, out);
                ui.end_row();
            }
        });
}

/// A series' plot and watch buttons, or two empty cells.
fn plot_toggle(ui: &mut egui::Ui, s: Option<&SeriesKey>, ctx: &Ctx<'_>, out: &mut Vec<Intent>) {
    let Some(s) = s else {
        ui.label("");
        ui.label("");
        return;
    };
    super::plot_button(ui, s, ctx.plots.contains(s), out);
    super::watch_button(ui, s, ctx.watch.contains(s), out);
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

fn market(
    ui: &mut egui::Ui,
    m: &MarketVm,
    ctx: &Ctx<'_>,
    charts: &mut Charts,
    out: &mut Vec<Intent>,
) {
    ui.heading(format!("Market {}/{}", m.node, m.good));
    at(ui, m.tick, &m.date);
    values(ui, "market", &m.values, ctx, out);
    ui.horizontal(|ui| {
        ui.label("ln(p′/p)");
        value_label(ui, m.log_step, "per tick");
    });
    ui.separator();
    ui.strong("Why this price: the tick's step, recomputed");
    match (&m.explainer, &m.unexplained) {
        (Some(e), _) => explainer(ui, e),
        (None, Some(why)) => {
            ui.weak(why);
        }
        (None, None) => {}
    }
    if let Some(w) = ctx.waterfall {
        ui.separator();
        let term = w.as_ref().map_or("k·x", |w| w.term.as_str());
        ui.strong(format!("ln(p/p₀) as Σ {term}: the log waterfall"));
        match w {
            Ok(w) => waterfall(ui, w, charts),
            Err(why) => {
                ui.weak(why);
            }
        }
    }
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
            ctx,
            out,
        );
    }
}

/// The explainer: every input, and markets' own result beside the run's, bit for bit.
pub fn explainer(ui: &mut egui::Ui, e: &ExplainerVm) {
    egui::Grid::new("explainer")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let row = |ui: &mut egui::Ui, k: &str, v: String| {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            };
            row(ui, "p, posted", format!("{:?}", e.price));
            row(ui, "S", format!("{:?}", e.supply));
            row(ui, "D", format!("{:?}", e.demand));
            let side = if e.one_side { " (one side posted)" } else { "" };
            row(
                ui,
                "x = imbalance(S, D)",
                format!("{:?}{side}", e.imbalance),
            );
            row(
                ui,
                &format!("k = {}({})", e.method, e.rate),
                format!(
                    "{:?} per tick, of {} {}",
                    e.k,
                    fmt(e.rate_value),
                    e.rate_unit
                ),
            );
            row(ui, "k·x", format!("{:?}", e.kx));
            // A free step (P2.4.11; FREE-SPEC §6.1): its shift c·p_ref, and markets' own step.
            match &e.free {
                Some(f) => {
                    row(
                        ui,
                        &format!("c·p_ref = {}·p[{}]", f.scale, f.reference),
                        format!(
                            "{:?}·{:?} = {:?}",
                            f.scale_value, f.reference_price, f.shift
                        ),
                    );
                    row(
                        ui,
                        &format!("next_price_free({}, p, k, S, D, c·p_ref)", e.one_sided),
                        format!("{:?}", e.next),
                    );
                }
                None => row(
                    ui,
                    &format!("next_price({}, {}, p, k, S, D)", e.rule, e.one_sided),
                    format!("{:?}", e.next),
                ),
            }
            match (e.recorded, e.equal) {
                (Some(r), Some(eq)) => {
                    let said = if eq {
                        "equal bit for bit"
                    } else {
                        "DIFFERS from the recomputed step"
                    };
                    row(ui, "the run's next price", format!("{r:?}: {said}"));
                }
                _ => row(
                    ui,
                    "the run's next price",
                    "not recorded (the lean catalogue)".to_string(),
                ),
            }
            if let Some(l) = e.log_step {
                row(ui, "ln(next/p)", format!("{l:?}"));
            }
        });
    if e.rule == "Ratio" {
        ui.weak("Ratio ignores k: the next price is p·D/S where both sides posted, else p");
    }
}

/// The waterfall: a bar a year of Σ k·x (Σ ln(D/S) under `Ratio`), stacked from ln(p/p₀) at the
/// year's start; the line of ln(p/p₀) itself; and each event fired, a line at its tick, orange
/// where it acts on this market's price or its rate. What the plot is lent is recorded in
/// `charts`. Public so a test can draw it alone.
pub fn waterfall(ui: &mut egui::Ui, w: &WaterfallVm, charts: &mut Charts) {
    let term = format!("Σ {}", w.term);
    ui.label(format!(
        "at tick {}: ln(p/p₀) {} = {term} {} + residual {}",
        w.tick,
        fmt(w.level),
        fmt(w.explained),
        fmt(w.residual)
    ))
    .on_hover_text(format!(
        "p₀ {:?} at tick {}, p {:?}; ln(p/p₀) {:?}, {term} {:?}, residual {:?}",
        w.p0, w.start, w.p, w.level, w.explained, w.residual
    ));
    let moving = w.events.iter().filter(|e| e.moves).count();
    ui.weak(format!(
        "{} ticks one-sided, {} without every input recorded; {} events fired, {moving} on \
         this price or its rate",
        w.one_sided,
        w.missing,
        w.events.len()
    ));
    let mut before = 0.0;
    let bars: Vec<Bar> = w
        .bins
        .iter()
        .map(|b| {
            let mid = (b.from + b.to) as f64 / 2.0;
            let width = (b.to - b.from) as f64 * 0.9;
            let bar = Bar::new(mid, b.kx)
                .base_offset(before)
                .width(width)
                .name(format!("{}: {term} {}", b.label, fmt(b.kx)));
            before = b.level;
            bar
        })
        .collect();
    let mut level: Vec<[f64; 2]> = vec![[w.start as f64, 0.0]];
    level.extend(w.bins.iter().map(|b| [b.to as f64, b.level]));
    let (orange, blue) = (super::plots::OTHER[1], super::plots::OTHER[0]);
    let plot = Plot::new("waterfall")
        .height(200.0)
        .legend(Legend::default());
    charts::show(charts, "waterfall", plot, ui, move |pui, lend| {
        lend.bars(pui, &term, bars, blue);
        lend.line(pui, "ln(p/p₀)", level, orange);
        for e in &w.events {
            // Not the plots' grey, which marks their cursor.
            let colour = if e.moves {
                orange
            } else {
                Color32::from_gray(128)
            };
            let name = format!("{} ({})", e.key, e.what);
            lend.vline(pui, &name, e.tick as f64, colour);
        }
    });
}

fn actor(ui: &mut egui::Ui, a: &ActorVm, ctx: &Ctx<'_>, out: &mut Vec<Intent>) {
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
                plot_toggle(ui, h.held.series.as_ref(), ctx, out);
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
            ctx,
            out,
        );
    }
}

fn param(ui: &mut egui::Ui, p: &ParamVm, ctx: &Ctx<'_>, out: &mut Vec<Intent>) {
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
    values(ui, "param", std::slice::from_ref(&p.value), ctx, out);
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

fn event(ui: &mut egui::Ui, e: &EventVm, breakpoints: &[Breakpoint], out: &mut Vec<Intent>) {
    ui.heading(format!("Event {}", e.key));
    // G1: a breakpoint on this event pauses a run after each tick it fires in.
    let at = Breakpoint::OnEvent(e.key.clone());
    let mut on = breakpoints.contains(&at);
    if ui
        .checkbox(&mut on, format!("break on event {}", e.key))
        .on_hover_text("pause a run after each tick this event fires in")
        .changed()
    {
        out.push(Intent::Breakpoint { at, on });
    }
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

fn other(ui: &mut egui::Ui, o: &OtherVm, ctx: &Ctx<'_>, out: &mut Vec<Intent>) {
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
    values(ui, "other", &o.values, ctx, out);
}
