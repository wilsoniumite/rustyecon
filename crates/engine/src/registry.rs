//! The registry listing (docs/ENGINE.md §5, R4): every number a tape feeds a run, in one place.
//!
//! Two kinds of number: registered params, each with its unit, whether it is live or fixed, its
//! per-tick value (§6) and its basis; and the dimensionless structural data that sits inline
//! under an entry's basis (recipe coefficients, weights, genesis prices and stocks, event
//! quantities). `rustyecon registry <tape>` prints this list.

use crate::{Tape, World};
use rustyecon_core::tape::raw::RawAct;
use rustyecon_core::{
    resolve, Amount, Basis, CompoundPerYear, FlowPerYear, FractionPerYear, LoadError, RatePerYear,
    Unit, Years,
};
use std::fmt;

/// What kind of number a line lists.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// A registered param.
    Param {
        /// Its unit.
        unit: Unit,
        /// Whether the loader turned it into structure, so no `SetParam` may change it.
        fixed: bool,
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
                fixed,
                per_tick,
            } => {
                let use_ = if *fixed { "fixed" } else { "live" };
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

/// The per-tick value of a param by its unit and use (§6's table).
fn per_tick(w: &World, p: &rustyecon_core::ParamDef) -> Option<(&'static str, f64)> {
    let c = &w.clock;
    let v = p.genesis;
    let price_rate = w.goods.iter().any(|g| g.price_rate == Some(p.id));
    Some(match p.unit {
        Unit::Dimensionless => ("value", v),
        Unit::FlowPerYear => ("flow", c.flow(FlowPerYear(v))),
        Unit::RatePerYear if price_rate => ("log_step", c.log_step(RatePerYear(v))),
        Unit::RatePerYear => ("share", c.share(RatePerYear(v))),
        Unit::CompoundPerYear => ("compound", c.compound(CompoundPerYear(v))),
        Unit::FractionPerYear => ("fraction", c.fraction(FractionPerYear(v))),
        Unit::Years if p.fixed => ("ticks", f64::from(c.ticks(Years(v)).ok()?)),
        Unit::Years => ("weight", c.weight(Years(v))),
    })
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

/// Every number `tape` feeds a run: the registered params in key order, then the inline
/// numbers in the tape's canonical order.
pub fn registry(tape: &Tape) -> Result<Vec<RegistryLine>, LoadError> {
    let (w, _) = resolve(tape)?;
    let mut out = Vec::new();
    for p in w.registry.params() {
        out.push(RegistryLine {
            path: format!("params[{}]", p.key),
            value: p.genesis,
            entry: Entry::Param {
                unit: p.unit,
                fixed: p.fixed,
                per_tick: per_tick(&w, p),
            },
            basis: p.basis.clone(),
        });
    }
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
