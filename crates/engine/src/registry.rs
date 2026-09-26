//! The registry listing (docs/ENGINE.md §5, R4): every number a tape feeds a run, in one place.
//!
//! Two kinds of number: params, each with its unit, how the run uses it (live, fixed, or read
//! by the schedule alone), each of its uses with the conversion that use takes and its per-tick
//! value (§6), and its basis; and the dimensionless structural data that sits inline under an
//! entry's basis (recipe coefficients, weights, genesis prices and stocks, event quantities).
//! `rustyecon registry <tape>` prints this list.
//!
//! Each use is a site the resolver recorded, with the method the run reads it by (amended at
//! S2.2, D10 item 4). Before, the listing guessed one method per param from its unit and from
//! whether some good's price moved at it, so a `RatePerYear` both a price rate and a spending
//! rate showed as `log_step` alone.

use crate::{Tape, World};
use rustyecon_core::tape::raw::RawAct;
use rustyecon_core::{
    resolve, Amount, Basis, ClockMethod, LoadError, LoadErrorKind, ParamSite, Unit,
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
    /// Read by the schedule alone: a value a `SetParam` copies, a recurring period, or a
    /// `ScalePrice`'s factor (amended at S2.3). It is not in the state or in `world_id` (§2.6).
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

/// One use of a param: where the tape references it, the conversion that use takes, and the
/// per-tick value at the param's genesis (or schedule) value.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteLine {
    /// The tape path of the reference, such as `actors[mill].spec.spend`.
    pub path: String,
    /// The conversion the run applies there.
    pub method: ClockMethod,
    /// `method.per_tick` of the listed value on the tape's clock.
    pub per_tick: f64,
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
        /// Each use, in (path, method) order. Empty for a param only a `SetParam` targets.
        sites: Vec<SiteLine>,
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
            Entry::Param { unit, use_, sites } => {
                let uses = if sites.is_empty() {
                    "-".to_string()
                } else {
                    let each: Vec<String> = sites
                        .iter()
                        .map(|s| format!("{} {:e} per tick at {}", s.method, s.per_tick, s.path))
                        .collect();
                    each.join("; ")
                };
                write!(
                    f,
                    "{}\t{:e}\t{unit} {use_}\t{uses}\t{basis}",
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

/// Each recorded use of a param with value `v`, converted on the world's clock.
fn site_lines(w: &World, sites: &[ParamSite], v: f64) -> Result<Vec<SiteLine>, LoadError> {
    sites
        .iter()
        .map(|s| {
            let per_tick = s.method.per_tick(&w.clock, v).map_err(|e| {
                LoadError::new(s.path.clone(), LoadErrorKind::Invalid(e.to_string()))
            })?;
            Ok(SiteLine {
                path: s.path.clone(),
                method: s.method,
                per_tick,
            })
        })
        .collect()
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
/// alike, each with its uses, then the inline numbers in the tape's canonical order.
pub fn registry(tape: &Tape) -> Result<Vec<RegistryLine>, LoadError> {
    let (w, _) = resolve(tape)?;
    let mut params: Vec<(&rustyecon_core::Key, RegistryLine)> = Vec::new();
    for p in w.registry.params() {
        let use_ = if p.fixed { Use::Fixed } else { Use::Live };
        let line = RegistryLine {
            path: format!("params[{}]", p.key),
            value: p.genesis,
            entry: Entry::Param {
                unit: p.unit,
                use_,
                sites: site_lines(&w, &p.sites, p.genesis)?,
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
                sites: site_lines(&w, &p.sites, p.value)?,
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
