//! The registry's view-model (docs/GUI.md §4, §7.2 item 4): every param, as `engine::registry`
//! lists it for the cli's `rustyecon registry`, with its key, unit, use, genesis value, current
//! value and basis, and each of its uses with the conversion that use takes and its per-tick
//! value. Then the inline numbers, each under its entry's basis.
//!
//! Beside the current value: the basis of the param the last `SetParam` copied into it, with
//! that event's key and date, read from the fired event's `source` (D10 item 2), so a changed
//! value names where it came from.

use super::{basis, firing_date, param_key, registry_line, report_tick, set_param_target};
use crate::run::{At, Measure, SeriesKey, Store};
use rustyecon_engine::prelude::*;
use rustyecon_engine::{Entry, RegistryLine};
use serde::Serialize;

/// One use of a param.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SiteVm {
    /// The tape path of the reference.
    pub path: String,
    /// The conversion the run applies there: `flow`, `share`, `log_step`, …
    pub method: String,
    /// The per-tick value at the genesis value, as the listing gives it.
    pub per_tick: f64,
    /// The per-tick value at the current value, when that differs from genesis.
    pub per_tick_now: Option<f64>,
}

/// The value a `SetParam` copied into a param.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CopiedVm {
    /// The event.
    pub event: Key,
    /// Its date.
    pub date: String,
    /// The tick it fired in.
    pub tick: u64,
    /// The param it copied from.
    pub source: Key,
    /// That param's basis, which the new value carries.
    pub basis: String,
}

/// One param.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParamRowVm {
    /// Its key.
    pub key: String,
    /// Its registered unit.
    pub unit: String,
    /// How the run uses it: `live`, `fixed` or `schedule`.
    pub use_: String,
    /// Its genesis value.
    pub genesis: f64,
    /// Its value after the cursor's tick: the run's record for a registered param, the
    /// constant for a schedule param. Before any tick of a run from genesis, the genesis value.
    pub current: Option<f64>,
    /// Its basis.
    pub basis: String,
    /// Each use.
    pub sites: Vec<SiteVm>,
    /// The last `SetParam` into it at or before the cursor, if one fired.
    pub copied: Option<CopiedVm>,
}

/// One inline number.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InlineRowVm {
    /// Its tape path.
    pub path: String,
    /// Its value.
    pub value: f64,
    /// Its entry's basis.
    pub basis: String,
}

/// The registry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegistryVm {
    /// The cursor's report tick, once a tick has run.
    pub tick: Option<u64>,
    /// Every param, in key order.
    pub params: Vec<ParamRowVm>,
    /// Every inline number, in the tape's canonical order.
    pub inline: Vec<InlineRowVm>,
    /// Why the listing could not be made, if it could not.
    pub error: Option<String>,
}

/// The last `SetParam` into `key` fired at or before report tick `tick`.
pub fn copied_into(
    store: &Store,
    lines: &[RegistryLine],
    key: &str,
    tick: u64,
) -> Option<CopiedVm> {
    let w = store.world()?;
    let (t, e) = store.events().iter().rev().find(|(t, e)| {
        *t <= tick
            && e.source.is_some()
            && set_param_target(w, &e.action).is_some_and(|k| k.as_str() == key)
    })?;
    let source = e.source.clone()?;
    Some(CopiedVm {
        event: e.key.clone(),
        date: firing_date(w, *t, &e.key),
        tick: *t,
        basis: registry_line(lines, source.as_str()).map_or_else(String::new, |l| basis(&l.basis)),
        source,
    })
}

/// A param's value after report tick `tick`: the record's for a registered param, the
/// constant for a schedule param, the genesis value before any tick of a run from genesis.
pub fn current_value(
    store: &Store,
    key: &Key,
    genesis: f64,
    schedule: bool,
    tick: Option<u64>,
) -> Option<f64> {
    if schedule {
        return Some(genesis);
    }
    match tick {
        Some(t) => {
            let s = SeriesKey {
                measure: Measure::Param,
                at: At::Param(key.clone()),
            };
            store.series(&s)?.at(t)
        }
        None => (store.start() == 0).then_some(genesis),
    }
}

/// The registry of a run, at report tick `cursor` (`None`: live).
pub fn build(store: &Store, cursor: Option<u64>) -> Option<RegistryVm> {
    let w = store.world()?;
    let tick = report_tick(store, cursor);
    let lines = match store.registry() {
        Ok(l) => l,
        Err(e) => {
            return Some(RegistryVm {
                tick,
                params: Vec::new(),
                inline: Vec::new(),
                error: Some(e.to_string()),
            })
        }
    };
    let mut params = Vec::new();
    let mut inline = Vec::new();
    for l in lines {
        match param_row(store, w, lines, l, tick) {
            Some(row) => params.push(row),
            None => inline.push(InlineRowVm {
                path: l.path.clone(),
                value: l.value,
                basis: basis(&l.basis),
            }),
        }
    }
    Some(RegistryVm {
        tick,
        params,
        inline,
        error: None,
    })
}

/// A listing line's row, if it lists a param, at report tick `tick`.
pub fn param_row(
    store: &Store,
    w: &World,
    lines: &[RegistryLine],
    l: &RegistryLine,
    tick: Option<u64>,
) -> Option<ParamRowVm> {
    let (Entry::Param { unit, use_, sites }, Some(k)) = (&l.entry, param_key(l)) else {
        return None;
    };
    let key = Key::new(k).ok()?;
    let schedule = w.schedule.param(k).is_some();
    let current = current_value(store, &key, l.value, schedule, tick);
    let changed = current.filter(|c| c.to_bits() != l.value.to_bits());
    let sites = sites
        .iter()
        .map(|s| SiteVm {
            path: s.path.clone(),
            method: s.method.name().to_string(),
            per_tick: s.per_tick,
            per_tick_now: changed.and_then(|c| s.method.per_tick(&w.clock, c).ok()),
        })
        .collect();
    Some(ParamRowVm {
        key: k.to_string(),
        unit: unit.to_string(),
        use_: use_.to_string(),
        genesis: l.value,
        current,
        basis: basis(&l.basis),
        sites,
        copied: tick.and_then(|t| copied_into(store, lines, k, t)),
    })
}
