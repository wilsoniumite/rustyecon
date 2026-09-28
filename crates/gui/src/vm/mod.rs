//! View-models (docs/GUI.md §3.2): one pure builder per panel. A builder reads run types (a
//! [`Store`], log lines, keys) and returns plain data, `Serialize + Debug`, which a panel in
//! `ui/` draws and a golden test pins (`tests/goldens.rs`). No egui, and nothing from `model/`,
//! so the builders move into `crates/observe` with the Runner at G2 (D13). The cursor comes in
//! as a report tick, `None` for live.
//!
//! Every number a builder gives is one the run recorded, or one of the engine's own functions
//! of recorded numbers (`ClockMethod::per_tick`, `engine::registry`), or a display transform
//! U6 allows: a log through `core::num`. Every number carries its unit.

pub mod compare;
pub mod inspector;
pub mod lab;
pub mod log;
pub mod map;
pub mod outliner;
pub mod plots;
pub mod pricestep;
pub mod registry;
pub mod timeline;
pub mod toolbar;
pub mod watch;

use crate::run::{At, HolderKey, Measure, SeriesKey, StateField, Store};
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::{AgentDelta, Agents};
use rustyecon_engine::RegistryLine;
use std::fmt::Debug;

/// The report tick the panels read: the cursor's, brought inside the record, or with no cursor
/// (live) the latest tick that ran. `None` before any tick ran. A report tick `t` is the tick
/// that ran: its report's values, and the state it left, whose tick is `t + 1`.
pub fn report_tick(store: &Store, cursor: Option<u64>) -> Option<u64> {
    store.report_at(cursor)
}

/// The first date of a tick, `YYYY-MM-DD`; empty past the calendar.
pub fn date(world: &World, tick: u64) -> String {
    world
        .clock
        .date_of(tick)
        .map_or_else(String::new, |d| d.to_string())
}

/// A basis as the tape writes it: `Assumed("gate world")`.
pub fn basis(b: &impl Debug) -> String {
    format!("{b:?}")
}

/// The key of a node's currency. A node this world lacks has none, and none is made up.
fn currency(w: &World, node: &Key) -> String {
    w.id_of::<NodeId>(node.as_str())
        .and_then(|n| w.node(n))
        .and_then(|n| w.key_of(n.currency))
        .map_or_else(|| format!("(no node {node} in this run)"), Key::to_string)
}

/// The unit of a series (docs/GUI.md §4: units always shown). Prices are in the node's
/// currency per unit of the good; flows are per tick, as the report counts them; fills are
/// shares; holdings and drifts are in the good; a param is in its registered unit; a margin is
/// a multiple of its tolerance.
pub fn unit_of(w: &World, key: &SeriesKey) -> String {
    match (key.measure, &key.at) {
        (Measure::Price | Measure::NextPrice | Measure::Ema, At::Market { node, good }) => {
            format!("{} per {good}", currency(w, node))
        }
        (Measure::Supply | Measure::Demand | Measure::Cleared, At::Market { good, .. })
        | (Measure::Requested | Measure::Feasible | Measure::Filled, At::Class { good, .. })
        | (Measure::SettledQty, At::Order { good, .. })
        | (Measure::Declared, At::Line { good, .. }) => format!("{good} per tick"),
        (Measure::SettledValue, At::Order { node, .. }) => {
            format!("{} per tick", currency(w, node))
        }
        (Measure::BuyerFill | Measure::SellerFill, _) => "share".to_string(),
        (Measure::Held, At::Holding { good, .. }) | (Measure::RunDrift, At::Good(good)) => {
            good.to_string()
        }
        (Measure::Param, At::Param(p)) => param_unit(w, p),
        (Measure::TickMargin | Measure::RunMargin, _) => "of its tolerance".to_string(),
        (Measure::TickDrift, _) => "largest over goods, each in its own unit".to_string(),
        (Measure::Trades, _) => "1 if it traded, else 0".to_string(),
        (Measure::State(f), At::Actor(a)) => match f {
            StateField::Share | StateField::Used => "share".to_string(),
            StateField::Scale | StateField::Output => "of its output per tick".to_string(),
            StateField::Own | StateField::Serving | StateField::Held | StateField::Target => {
                "units of the durable good".to_string()
            }
            StateField::Order => "units of the durable good per tick".to_string(),
            StateField::Run => "hours per tick".to_string(),
            StateField::Due | StateField::Paid => w
                .id_of::<ActorId>(a.as_str())
                .and_then(|id| w.actor(id))
                .and_then(|d| w.node(d.home))
                .and_then(|n| w.key_of(n.currency))
                .map_or_else(String::new, |c| format!("{c} per tick")),
        },
        _ => String::new(),
    }
}

/// A param's registered unit, a schedule param's included.
pub fn param_unit(w: &World, p: &Key) -> String {
    if let Some(d) = w
        .id_of::<ParamId>(p.as_str())
        .and_then(|id| w.registry.get(id))
    {
        return d.unit.to_string();
    }
    w.schedule
        .param(p.as_str())
        .map_or_else(String::new, |s| s.unit.to_string())
}

/// A holder by key: an actor's, or `escrow of town/bread`.
pub fn holder(w: &World, h: Holder) -> String {
    match h {
        Holder::Actor(a) => key_or_id(w.key_of(a), a),
        Holder::Escrow(n, g) => format!(
            "escrow of {}/{}",
            key_or_id(w.key_of(n), n),
            key_or_id(w.key_of(g), g)
        ),
    }
}

/// A holder's key, as a series names it.
pub fn holder_key(w: &World, h: Holder) -> Option<HolderKey> {
    Some(match h {
        Holder::Actor(a) => HolderKey::Actor(w.key_of(a)?.clone()),
        Holder::Escrow(n, g) => HolderKey::Escrow {
            node: w.key_of(n)?.clone(),
            good: w.key_of(g)?.clone(),
        },
    })
}

fn key_or_id(k: Option<&Key>, id: impl std::fmt::Display) -> String {
    k.map_or_else(|| id.to_string(), Key::to_string)
}

/// An amount as the tape writes it.
fn amount(a: &Amount) -> String {
    match a {
        Amount::Qty(q) => format!("{q}"),
        Amount::All => "all".to_string(),
    }
}

/// A resolved action, by key: what a tape event does.
pub fn describe(w: &World, delta: &StateDelta<Agents>) -> String {
    let good = |g: GoodId| key_or_id(w.key_of(g), g);
    let node = |n: NodeId| key_or_id(w.key_of(n), n);
    match delta {
        StateDelta::Mint {
            to, good: g, qty, ..
        } => {
            format!("Mint {qty} {} to {}", good(*g), holder(w, *to))
        }
        StateDelta::Burn {
            from,
            good: g,
            amount: a,
            ..
        } => format!("Burn {} {} from {}", amount(a), good(*g), holder(w, *from)),
        StateDelta::Transfer {
            from,
            to,
            good: g,
            amount: a,
        } => format!(
            "Transfer {} {} from {} to {}",
            amount(a),
            good(*g),
            holder(w, *from),
            holder(w, *to)
        ),
        StateDelta::SetParam { param, value } => {
            format!("SetParam {} to {value}", key_or_id(w.key_of(*param), param))
        }
        StateDelta::ScalePrice {
            node: n,
            good: g,
            factor,
        } => format!("ScalePrice {}/{} by {factor}", node(*n), good(*g)),
        StateDelta::Actor(AgentDelta::SetActive { actor, active }) => {
            format!("SetActive {} {active}", key_or_id(w.key_of(*actor), actor))
        }
        StateDelta::Actor(AgentDelta::SetState { actor, .. }) => {
            format!("SetState of {}", key_or_id(w.key_of(*actor), actor))
        }
        other => other.name().to_string(),
    }
}

/// The param a `SetParam` sets, by key.
pub fn set_param_target(w: &World, delta: &StateDelta<Agents>) -> Option<Key> {
    match delta {
        StateDelta::SetParam { param, .. } => w.key_of(*param).cloned(),
        _ => None,
    }
}

/// The registry listing's line for a param, `params[key]`.
pub fn registry_line<'a>(lines: &'a [RegistryLine], key: &str) -> Option<&'a RegistryLine> {
    lines.iter().find(|l| param_key(l) == Some(key))
}

/// The param key of a listing line, `params[key]`; `None` for an inline number.
pub fn param_key(l: &RegistryLine) -> Option<&str> {
    l.path.strip_prefix("params[")?.strip_suffix(']')
}

/// The date a firing is dated: a dated event's own date, and a recurring occurrence's tick's
/// first day.
pub fn firing_date(w: &World, tick: u64, key: &Key) -> String {
    // The dated events are sorted by (tick, day, key), so the tick's are found by bisection:
    // a schedule of 30,078 events (the demo world's) is read once a frame.
    let once = w.schedule.once();
    let lo = once.partition_point(|f| f.tick < tick);
    let hi = once.partition_point(|f| f.tick <= tick);
    let dated = once[lo..hi].iter().find(|f| w.key_of(f.event) == Some(key));
    match dated.and_then(|f| Date::from_days(f.day)) {
        Some(d) => d.to_string(),
        None => date(w, tick),
    }
}

/// A series' value at a report tick, if it has one there.
pub fn value_at(store: &Store, key: &SeriesKey, tick: Option<u64>) -> Option<f64> {
    store.series(key)?.at(tick?)
}
