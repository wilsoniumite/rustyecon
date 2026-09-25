//! The registry listing (docs/ENGINE.md §5, R4): every number a tape feeds a run, in one place.
//!
//! Two kinds of number: params, each with its unit, how the run uses it (live, fixed, or read
//! by the schedule alone), its per-tick value (§6) and its basis; and the dimensionless
//! structural data that sits inline under an entry's basis (recipe coefficients, weights,
//! genesis prices and stocks, event quantities). `rustyecon registry <tape>` prints this list.

use crate::{Tape, World};
use rustyecon_core::tape::raw::RawAct;
use rustyecon_core::{
    resolve, Amount, Basis, CompoundPerYear, FlowPerYear, FractionPerYear, Key, LoadError,
    RatePerYear, StateDelta, Unit, Years,
};
use std::fmt;

/// How a run uses a param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Use {
    /// Read at use time; a dated `SetParam` may change it.
    Live,
    /// Turned into structure at load, or fixing what a past tick meant; no `SetParam` may
    /// change it.
    Fixed,
    /// Read by the schedule alone: a value a `SetParam` copies, or a recurring period. It is
    /// not in the state or in `world_id` (§2.6).
    Schedule,
}

impl fmt::Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Use::Live => "live",
            Use::Fixed => "fixed",
            Use::Schedule => "schedule",
        })
    }
}

/// What kind of number a line lists.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// A param from the tape's `params` list.
    Param {
        /// Its unit.
        unit: Unit,
        /// How the run uses it.
        use_: Use,
        /// The per-tick conversion its use takes (§6) and the result at genesis.
        per_tick: Option<(&'static str, f64)>,
    },
    /// A dimensionless number inline under its entry's basis.
    Inline,
}

/// One number.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryLine {
    /// Its tape path.
    pub path: String,
    /// Its value at genesis.
    pub value: f64,
    /// What kind of number it is.
    pub entry: Entry,
    /// Where it comes from.
    pub basis: Basis,
}

impl fmt::Display for RegistryLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let basis = match &self.basis {
            Basis::Measured { source, vintage } => format!("Measured({source}, {vintage})"),
            Basis::Literature(s) => format!("Literature({s})"),
            Basis::Approximate(s) => format!("Approximate({s})"),
            Basis::Fitted { fit } => format!("Fitted({fit})"),
            Basis::Assumed(s) => format!("Assumed({s})"),
        };
        match &self.entry {
            Entry::Param {
                unit,
                use_,
                per_tick,
            } => {
                let tick = match per_tick {
                    Some((how, v)) => format!("{how} {v:e} per tick"),
                    None => "-".to_string(),
                };
                write!(
                    f,
                    "{}\t{:e}\t{unit} {use_}\t{tick}\t{basis}",
                    self.path, self.value
                )
            }
            Entry::Inline => write!(
                f,
                "{}\t{:e}\tDimensionless inline\t-\t{basis}",
                self.path, self.value
            ),
        }
    }
}

/// The per-tick value of `v` in `unit` (§6's table): `ticks` for a period or shelf life,
/// `log_step` for a price rate, and otherwise the unit's own conversion.
fn per_tick(
    w: &World,
    unit: Unit,
    v: f64,
    whole_ticks: bool,
    price_rate: bool,
) -> Option<(&'static str, f64)> {
    let c = &w.clock;
    Some(match unit {
        Unit::Dimensionless => ("value", v),
        Unit::FlowPerYear => ("flow", c.flow(FlowPerYear(v))),
        Unit::RatePerYear if price_rate => ("log_step", c.log_step(RatePerYear(v))),
        Unit::RatePerYear => ("share", c.share(RatePerYear(v))),
        Unit::CompoundPerYear => ("compound", c.compound(CompoundPerYear(v))),
        Unit::FractionPerYear => ("fraction", c.fraction(FractionPerYear(v))),
        Unit::Years if whole_ticks => ("ticks", f64::from(c.ticks(Years(v)).ok()?)),
        Unit::Years => ("weight", c.weight(Years(v))),
    })
}

/// Whether a registered param is some good's price rate.
fn is_price_rate(w: &World, p: rustyecon_core::ParamId) -> bool {
    w.goods.iter().any(|g| g.price_rate == Some(p))
}

/// The per-tick value of a schedule param: a `SetParam`'s source converts as the param it sets,
/// and a recurring period is whole ticks.
fn schedule_per_tick(w: &World, key: &Key, unit: Unit, v: f64) -> Option<(&'static str, f64)> {
    let s = &w.schedule;
    let firings = s.once().iter().map(|f| (&f.source, &f.action));
    let recurring = s.every().iter().map(|r| (&r.source, &r.action));
    let target = firings
        .chain(recurring)
        .find_map(|(source, action)| match action {
            StateDelta::SetParam { param, .. } if source.as_ref() == Some(key) => Some(*param),
            _ => None,
        });
    match target {
        Some(p) => per_tick(w, unit, v, false, is_price_rate(w, p)),
        None => per_tick(w, unit, v, true, false),
    }
}

fn act_numbers<A>(act: &RawAct<A>) -> Vec<(&'static str, f64)> {
    match act {
        RawAct::Mint { qty, .. } => vec![("act.qty", *qty)],
        RawAct::Burn {
            amount: Amount::Qty(q),
            ..
        }
        | RawAct::Transfer {
            amount: Amount::Qty(q),
            ..
        } => vec![("act.amount", *q)],
        _ => Vec::new(),
    }
}

/// Every number `tape` feeds a run: the params in key order, registered and the schedule's
/// alike, then the inline numbers in the tape's canonical order.
pub fn registry(tape: &Tape) -> Result<Vec<RegistryLine>, LoadError> {
    let (w, _) = resolve(tape)?;
    let mut params: Vec<(&Key, RegistryLine)> = Vec::new();
    for p in w.registry.params() {
        let (use_, whole_ticks) = if p.fixed {
            (Use::Fixed, true)
        } else {
            (Use::Live, false)
        };
        let line = RegistryLine {
            path: format!("params[{}]", p.key),
            value: p.genesis,
            entry: Entry::Param {
                unit: p.unit,
                use_,
                per_tick: per_tick(&w, p.unit, p.genesis, whole_ticks, is_price_rate(&w, p.id)),
            },
            basis: p.basis.clone(),
        };
        params.push((&p.key, line));
    }
    for p in w.schedule.params() {
        let line = RegistryLine {
            path: format!("params[{}]", p.key),
            value: p.value,
            entry: Entry::Param {
                unit: p.unit,
                use_: Use::Schedule,
                per_tick: schedule_per_tick(&w, &p.key, p.unit, p.value),
            },
            basis: p.basis.clone(),
        };
        params.push((&p.key, line));
    }
    params.sort_by(|a, b| a.0.cmp(b.0));
    let mut out: Vec<RegistryLine> = params.into_iter().map(|(_, l)| l).collect();
    let t = tape.canonical();
    let inline = |path: String, value: f64, basis: &Basis| RegistryLine {
        path,
        value,
        entry: Entry::Inline,
        basis: basis.clone(),
    };
    for a in &t.actors {
        for (path, v) in rustyecon_agents::spec::inline_numbers(&a.spec) {
            out.push(inline(
                format!("actors[{}].spec.{path}", a.key),
                v,
                &a.basis,
            ));
        }
    }
    let g = &t.genesis;
    for p in &g.prices {
        let path = format!("genesis.prices[{}/{}].price", p.node, p.good);
        out.push(inline(path, p.price, &g.basis));
    }
    for h in &g.holdings {
        for (good, q) in &h.goods {
            let path = format!("genesis.holdings[{}].goods[{good}]", h.holder);
            out.push(inline(path, *q, &g.basis));
        }
    }
    for e in &t.events {
        for (field, v) in act_numbers(&e.act) {
            out.push(inline(format!("events[{}].{field}", e.key), v, &e.basis));
        }
    }
    for e in &t.recurring {
        for (field, v) in act_numbers(&e.act) {
            out.push(inline(format!("recurring[{}].{field}", e.key), v, &e.basis));
        }
    }
    Ok(out)
}
