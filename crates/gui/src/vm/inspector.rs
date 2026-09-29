//! The inspector's view-model (docs/GUI.md §4): the selection in detail, at the cursor.
//!
//! - **Market:** p, next p, ema, S, D, cleared, both fills, rationing by class, ln(p′/p) (the
//!   last step, "why is this price 12.3", through the engine's `num`), and the rule with its
//!   rate param, unit, basis and per-tick step; and the price-step explainer (G1), the tick's
//!   step recomputed by markets' own `imbalance` and `next_price` ([`super::pricestep`]).
//! - **Actor:** its spec's uses of params, each with key, unit, current value and basis; its
//!   inline numbers; its holdings after the tick; the lots behind them and its own state, from
//!   a snapshot of that state (the model asks the Runner for one); and its settle lines.
//! - **Param:** its registry row, the events that set it and the events that copy from it.
//! - **Event:** its date, what it does, the param it copies with that param's basis, its own
//!   basis on the tape, and its firings so far.
//! - Goods, nodes and classes list what the world says of them.
//!
//! Each number carries its unit and, where the run records it, its series, so the panel can
//! plot it.

use super::registry::{copied_into, param_row, CopiedVm, ParamRowVm};
use super::{
    basis, date, describe, firing_date, param_unit, registry_line, report_tick, set_param_target,
    unit_of,
};
use crate::run::{At, Entity, HolderKey, Measure, SeriesKey, Store};
use rustyecon_engine::num;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::Spec;
use rustyecon_engine::Entry;
use serde::Serialize;

/// One number: what it is, its value, its unit, and its series if the run records one.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueVm {
    /// What it is.
    pub label: String,
    /// Its value at the cursor, if there is one.
    pub value: Option<f64>,
    /// Its unit.
    pub unit: String,
    /// Its series, if the run records it.
    pub series: Option<SeriesKey>,
}

/// A param as a use reads it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParamRefVm {
    /// Where the tape references it, under its entry.
    pub path: String,
    /// The param.
    pub key: Key,
    /// Its registered unit.
    pub unit: String,
    /// The conversion this use takes.
    pub method: String,
    /// Its value after the cursor's tick.
    pub value: Option<f64>,
    /// That value per tick, by this use's conversion.
    pub per_tick: Option<f64>,
    /// Its basis.
    pub basis: String,
    /// The last `SetParam` into it, if one fired.
    pub copied: Option<CopiedVm>,
}

/// One class's rationing in a market.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RationVm {
    /// The class.
    pub class: Key,
    /// `buy` or `sell`.
    pub side: String,
    /// Requested, feasible and filled.
    pub values: Vec<ValueVm>,
}

/// A market at the cursor.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarketVm {
    /// The node.
    pub node: Key,
    /// The good.
    pub good: Key,
    /// The report tick.
    pub tick: Option<u64>,
    /// Its date.
    pub date: String,
    /// p, next p, ema, S, D, cleared, buyer and seller fill.
    pub values: Vec<ValueVm>,
    /// ln(p′/p): the tick's step in the log price.
    pub log_step: Option<f64>,
    /// The tick's rationing lines, by class and side.
    pub rationing: Vec<RationVm>,
    /// The price rule.
    pub rule: String,
    /// What a one-sided market does.
    pub one_sided: String,
    /// The good's price rate, read as a log step per tick.
    pub rate: Option<ParamRefVm>,
    /// The EMA's time constant.
    pub ema_time_constant: Option<ParamRefVm>,
    /// The tick's step, recomputed by markets' own functions from its recorded inputs (G1).
    pub explainer: Option<super::pricestep::ExplainerVm>,
    /// Why there is no explainer, when there is none.
    pub unexplained: Option<String>,
}

/// One lot.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LotVm {
    /// Its quantity.
    pub qty: f64,
    /// Its remaining life in ticks, `None` for a good that does not age.
    pub life: Option<u32>,
}

/// One holding of an actor.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HoldingVm {
    /// The good.
    pub good: Key,
    /// The quantity held after the cursor's tick, from the record.
    pub held: ValueVm,
    /// Its lots, from the snapshot of that state, if one was taken.
    pub lots: Option<Vec<LotVm>>,
}

/// One order's settlement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SettleVm {
    /// The node.
    pub node: Key,
    /// The good.
    pub good: Key,
    /// `buy` or `sell`.
    pub side: String,
    /// The good received or shipped, and the currency paid or received.
    pub values: Vec<ValueVm>,
}

/// An inline number.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InlineVm {
    /// Its tape path.
    pub path: String,
    /// Its value.
    pub value: f64,
    /// Its entry's basis.
    pub basis: String,
}

/// An actor at the cursor.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActorVm {
    /// Its key.
    pub key: Key,
    /// Desk or pop.
    pub kind: String,
    /// Its class.
    pub class: String,
    /// Its home node.
    pub home: String,
    /// Its spec's kind: `Scripted`, `GoodDesk`, …
    pub spec: String,
    /// Its tape basis.
    pub basis: String,
    /// The report tick.
    pub tick: Option<u64>,
    /// Its date.
    pub date: String,
    /// The state tick the snapshot is of: the state the cursor's tick left.
    pub state_tick: u64,
    /// Whether a snapshot of that state has arrived.
    pub snapshot: bool,
    /// Its own state, from the snapshot.
    pub state: Option<String>,
    /// Each use of a param in its spec.
    pub params: Vec<ParamRefVm>,
    /// Its spec's inline numbers.
    pub inline: Vec<InlineVm>,
    /// What it holds after the tick.
    pub holdings: Vec<HoldingVm>,
    /// Its orders' settlements in the tick.
    pub settlements: Vec<SettleVm>,
}

/// A tape event that fired or will.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventRefVm {
    /// The event.
    pub key: Key,
    /// Its date.
    pub date: String,
    /// The tick it fires in.
    pub tick: u64,
    /// What it does.
    pub what: String,
    /// Whether it has fired by the cursor.
    pub fired: bool,
}

/// A param.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParamVm {
    /// Its registry row at the cursor.
    pub row: ParamRowVm,
    /// Its value, with its series if the run records one.
    pub value: ValueVm,
    /// The events that set it.
    pub set_by: Vec<EventRefVm>,
    /// The events that copy it into another param.
    pub copied_by: Vec<EventRefVm>,
}

/// A tape event.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EventVm {
    /// Its key.
    pub key: Key,
    /// `dated` or `recurring`.
    pub kind: String,
    /// A dated event's date, or a recurring entry's first.
    pub date: String,
    /// A recurring entry's period: its param and the whole ticks it rounds to.
    pub every: Option<String>,
    /// A recurring entry's last date, if it has one.
    pub last: Option<String>,
    /// The tick it first fires in.
    pub tick: u64,
    /// What it does.
    pub what: String,
    /// The param it copies from, and that param's basis.
    pub source: Option<(Key, String)>,
    /// Its own basis on the tape.
    pub basis: String,
    /// Its firings at or before the cursor: (tick, occurrence).
    pub fired: Vec<(u64, u32)>,
}

/// A good, a node or a class.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OtherVm {
    /// What it is: `good bread`.
    pub title: String,
    /// What the world says of it.
    pub facts: Vec<(String, String)>,
    /// Its numbers at the cursor.
    pub values: Vec<ValueVm>,
}

/// The inspector.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum InspectorVm {
    /// A market.
    Market(Box<MarketVm>),
    /// An actor.
    Actor(Box<ActorVm>),
    /// A param.
    Param(Box<ParamVm>),
    /// An event.
    Event(Box<EventVm>),
    /// A good, node or class.
    Other(Box<OtherVm>),
    /// A key this run's world does not have.
    Unknown(Entity),
}

fn value(store: &Store, w: &World, label: &str, key: SeriesKey, tick: Option<u64>) -> ValueVm {
    ValueVm {
        label: label.to_string(),
        value: tick.and_then(|t| store.series(&key)?.at(t)),
        unit: unit_of(w, &key),
        series: Some(key),
    }
}

fn side(s: SideTag) -> &'static str {
    match s {
        SideTag::Buy => "buy",
        SideTag::Sell => "sell",
    }
}

/// A use of a param, at report tick `tick`.
fn param_ref(
    store: &Store,
    w: &World,
    path: String,
    site: Site,
    tick: Option<u64>,
) -> Option<ParamRefVm> {
    let key = w.key_of(site.param)?.clone();
    let def = w.registry.get(site.param)?;
    let value = super::registry::current_value(store, &key, def.genesis, false, tick);
    let lines = store.registry().unwrap_or(&[]);
    Some(ParamRefVm {
        path,
        unit: def.unit.to_string(),
        method: site.method.name().to_string(),
        per_tick: value.and_then(|v| site.method.per_tick(&w.clock, v).ok()),
        value,
        basis: basis(&def.basis),
        copied: tick.and_then(|t| copied_into(store, lines, key.as_str(), t)),
        key,
    })
}

fn market(store: &Store, w: &World, node: &Key, good: &Key, tick: Option<u64>) -> Option<MarketVm> {
    w.id_of::<NodeId>(node.as_str())?;
    let g = w.id_of::<GoodId>(good.as_str())?;
    let at = At::Market {
        node: node.clone(),
        good: good.clone(),
    };
    let values: Vec<ValueVm> = [
        ("p", Measure::Price),
        ("next p", Measure::NextPrice),
        ("ema", Measure::Ema),
        ("S", Measure::Supply),
        ("D", Measure::Demand),
        ("cleared", Measure::Cleared),
        ("buyer fill", Measure::BuyerFill),
        ("seller fill", Measure::SellerFill),
    ]
    .into_iter()
    .map(|(label, measure)| {
        let key = SeriesKey {
            measure,
            at: at.clone(),
        };
        value(store, w, label, key, tick)
    })
    .collect();
    // ln(p′/p): a display transform of two recorded prices (U6), through core's libm.
    let log_step = match (values[0].value, values[1].value) {
        (Some(p), Some(q)) if p > 0.0 && q > 0.0 => Some(num::ln(q / p)),
        _ => None,
    };
    let mut classes: Vec<(Key, SideTag)> = store
        .catalogue()
        .iter()
        .filter_map(|k| match (&k.measure, &k.at) {
            (
                Measure::Requested,
                At::Class {
                    node: kn,
                    good: kg,
                    class,
                    side,
                },
            ) if kn == node && kg == good => Some((class.clone(), *side)),
            _ => None,
        })
        .collect();
    classes.sort();
    let rationing = classes
        .into_iter()
        .filter_map(|(class, s)| {
            let at = At::Class {
                node: node.clone(),
                good: good.clone(),
                class: class.clone(),
                side: s,
            };
            let values: Vec<ValueVm> = [
                ("requested", Measure::Requested),
                ("feasible", Measure::Feasible),
                ("filled", Measure::Filled),
            ]
            .into_iter()
            .map(|(label, measure)| {
                let key = SeriesKey {
                    measure,
                    at: at.clone(),
                };
                value(store, w, label, key, tick)
            })
            .collect();
            // A class that had no order this tick has no line.
            values[0].value.is_some().then(|| RationVm {
                class,
                side: side(s).to_string(),
                values,
            })
        })
        .collect();
    let rate = w
        .good(g)
        .and_then(|d| d.price_rate)
        .and_then(|site| param_ref(store, w, format!("goods[{good}].price_rate"), site, tick));
    let ema_time_constant = param_ref(
        store,
        w,
        "header.market.ema_time_constant".to_string(),
        w.market.ema_time_constant,
        tick,
    );
    let (explainer, unexplained) =
        match tick.map(|t| super::pricestep::explain(store, node, good, t)) {
            Some(Ok(e)) => (Some(e), None),
            Some(Err(why)) => (None, Some(why)),
            None => (None, Some("no tick has run".to_string())),
        };
    Some(MarketVm {
        node: node.clone(),
        good: good.clone(),
        tick,
        date: tick.map_or_else(String::new, |t| date(w, t)),
        values,
        log_step,
        rationing,
        rule: format!("{:?}", w.market.rule),
        one_sided: format!("{:?}", w.market.one_sided),
        rate,
        ema_time_constant,
        explainer,
        unexplained,
    })
}

fn spec_kind(s: &Spec) -> &'static str {
    match s {
        Spec::Scripted(_) => "Scripted",
        Spec::Provider(_) => "Provider",
        Spec::Workers(_) => "Workers",
        Spec::GoodDesk(_) => "GoodDesk",
        Spec::MachDesk(_) => "MachDesk",
        Spec::BasketProvider(_) => "BasketProvider",
        Spec::BasketWorkers(_) => "BasketWorkers",
        Spec::CategoryDesk(_) => "CategoryDesk",
        Spec::TypeDesk(d) if d.plant.is_some() => "TypeDesk, planted",
        Spec::TypeDesk(_) => "TypeDesk",
        Spec::Maker(d) if d.plant.is_some() => "Maker, planted",
        Spec::Maker(_) => "Maker",
        Spec::CapacityDesk(d) if d.plant.is_some() => "CapacityDesk, planted",
        Spec::CapacityDesk(_) => "CapacityDesk",
        Spec::OwnerDesk(_) => "OwnerDesk",
    }
}

fn actor(store: &Store, w: &World, key: &Key, cursor: Option<u64>) -> Option<ActorVm> {
    let id = w.id_of::<ActorId>(key.as_str())?;
    let decl = w.actor(id)?;
    let tick = report_tick(store, cursor);
    let state_tick = store.state_at(cursor);
    let snap = store.snapshot(state_tick);
    let prefix = format!("actors[{key}].spec.");
    let params = decl
        .spec
        .sites(w)
        .into_iter()
        .filter_map(|(path, site)| param_ref(store, w, path, site, tick))
        .collect();
    let lines = store.registry().unwrap_or(&[]);
    let inline = lines
        .iter()
        .filter(|l| matches!(l.entry, Entry::Inline) && l.path.starts_with(&prefix))
        .map(|l| InlineVm {
            path: l.path[prefix.len()..].to_string(),
            value: l.value,
            basis: basis(&l.basis),
        })
        .collect();
    let mut goods: Vec<Key> = store
        .catalogue()
        .iter()
        .filter_map(|k| match (&k.measure, &k.at) {
            (
                Measure::Held,
                At::Holding {
                    holder: HolderKey::Actor(a),
                    good,
                },
            ) if a == key => Some(good.clone()),
            _ => None,
        })
        .collect();
    goods.sort();
    goods.dedup();
    let holdings = goods
        .into_iter()
        .filter_map(|good| {
            let g = w.id_of::<GoodId>(good.as_str())?;
            let series = SeriesKey {
                measure: Measure::Held,
                at: At::Holding {
                    holder: HolderKey::Actor(key.clone()),
                    good: good.clone(),
                },
            };
            let held = value(store, w, "held", series, tick);
            let lots = snap.map(|s| {
                s.lots
                    .iter()
                    .find(|(h, lg, _)| *h == Holder::Actor(id) && *lg == g)
                    .map(|(_, _, lots)| {
                        lots.iter()
                            .map(|l| LotVm {
                                qty: l.qty,
                                life: l.life,
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            });
            // A good held neither after the tick nor in the snapshot is left out.
            (held.value.is_some() || lots.as_ref().is_some_and(|l: &Vec<LotVm>| !l.is_empty()))
                .then_some(HoldingVm { good, held, lots })
        })
        .collect();
    let mut orders: Vec<(Key, Key, SideTag)> = store
        .catalogue()
        .iter()
        .filter_map(|k| match (&k.measure, &k.at) {
            (
                Measure::SettledQty,
                At::Order {
                    actor: a,
                    node,
                    good,
                    side,
                },
            ) if a == key => Some((node.clone(), good.clone(), *side)),
            _ => None,
        })
        .collect();
    orders.sort();
    let settlements = orders
        .into_iter()
        .filter_map(|(node, good, s)| {
            let at = At::Order {
                actor: key.clone(),
                node: node.clone(),
                good: good.clone(),
                side: s,
            };
            let values: Vec<ValueVm> = [
                ("qty", Measure::SettledQty),
                ("value", Measure::SettledValue),
            ]
            .into_iter()
            .map(|(label, measure)| {
                let k = SeriesKey {
                    measure,
                    at: at.clone(),
                };
                value(store, w, label, k, tick)
            })
            .collect();
            values[0].value.is_some().then(|| SettleVm {
                node,
                good,
                side: side(s).to_string(),
                values,
            })
        })
        .collect();
    let basis_on_tape = store
        .tape()
        .and_then(|t| t.actors.iter().find(|a| &a.key == key))
        .map_or_else(String::new, |a| basis(&a.basis));
    Some(ActorVm {
        key: key.clone(),
        kind: format!("{:?}", id.kind()),
        class: w
            .key_of(decl.class)
            .map_or_else(String::new, Key::to_string),
        home: w.key_of(decl.home).map_or_else(String::new, Key::to_string),
        spec: spec_kind(&decl.spec).to_string(),
        basis: basis_on_tape,
        tick,
        date: tick.map_or_else(String::new, |t| date(w, t)),
        state_tick,
        snapshot: snap.is_some(),
        state: snap.and_then(|s| {
            s.actors
                .iter()
                .find(|(a, _)| *a == id)
                .map(|(_, st)| format!("{st:?}"))
        }),
        params,
        inline,
        holdings,
        settlements,
    })
}

/// Every firing before the end of the record or of the schedule, whichever is later.
fn all_firings(
    store: &Store,
    w: &World,
) -> Vec<(
    u64,
    Key,
    StateDelta<rustyecon_engine::rustyecon_agents::Agents>,
    Option<Key>,
)> {
    let end = w
        .schedule
        .once()
        .last()
        .map_or(0, |f| f.tick + 1)
        .max(store.tick());
    w.schedule
        .firings_before(end)
        .into_iter()
        .filter_map(|f| Some((f.tick, w.key_of(f.event)?.clone(), f.action, f.source)))
        .collect()
}

fn param(store: &Store, w: &World, key: &Key, cursor: Option<u64>) -> Option<ParamVm> {
    let tick = report_tick(store, cursor);
    let lines = store.registry().ok()?;
    let l = registry_line(lines, key.as_str())?;
    let row = param_row(store, w, lines, l, tick)?;
    let registered = w.id_of::<ParamId>(key.as_str()).is_some();
    let value = ValueVm {
        label: "value".to_string(),
        value: row.current,
        unit: param_unit(w, key),
        series: registered.then(|| SeriesKey {
            measure: Measure::Param,
            at: At::Param(key.clone()),
        }),
    };
    let fired = |t: u64, k: &Key| {
        tick.is_some_and(|c| t <= c) && store.events().iter().any(|(ft, e)| *ft == t && &e.key == k)
    };
    let mut set_by = Vec::new();
    let mut copied_by = Vec::new();
    for (t, k, action, source) in all_firings(store, w) {
        let r = EventRefVm {
            date: firing_date(w, t, &k),
            tick: t,
            what: describe(w, &action),
            fired: fired(t, &k),
            key: k,
        };
        if set_param_target(w, &action).as_ref() == Some(key) {
            set_by.push(r.clone());
        }
        if source.as_ref() == Some(key) {
            copied_by.push(r);
        }
    }
    Some(ParamVm {
        row,
        value,
        set_by,
        copied_by,
    })
}

fn event(store: &Store, w: &World, key: &Key, cursor: Option<u64>) -> Option<EventVm> {
    let tick = report_tick(store, cursor);
    let id = w.id_of::<EventId>(key.as_str())?;
    let tape = store.tape();
    let lines = store.registry().unwrap_or(&[]);
    let source_of = |s: &Option<Key>| {
        s.as_ref().map(|k| {
            let b = registry_line(lines, k.as_str()).map_or_else(String::new, |l| basis(&l.basis));
            (k.clone(), b)
        })
    };
    let fired = store
        .events()
        .iter()
        .filter(|(t, e)| &e.key == key && tick.is_some_and(|c| *t <= c))
        .map(|(t, e)| (*t, e.occurrence))
        .collect();
    if let Some(f) = w.schedule.once().iter().find(|f| f.event == id) {
        let basis_on_tape = tape
            .and_then(|t| t.events.iter().find(|e| &e.key == key))
            .map_or_else(String::new, |e| basis(&e.basis));
        return Some(EventVm {
            key: key.clone(),
            kind: "dated".to_string(),
            date: firing_date(w, f.tick, key),
            every: None,
            last: None,
            tick: f.tick,
            what: describe(w, &f.action),
            source: source_of(&f.source),
            basis: basis_on_tape,
            fired,
        });
    }
    let r = w.schedule.every().iter().find(|r| r.event == id)?;
    let raw = tape.and_then(|t| t.recurring.iter().find(|e| &e.key == key));
    Some(EventVm {
        key: key.clone(),
        kind: "recurring".to_string(),
        date: raw.map_or_else(|| date(w, r.first), |e| e.first.to_string()),
        every: Some(match raw {
            Some(e) => format!("{}, {} ticks", e.every, r.period),
            None => format!("{} ticks", r.period),
        }),
        last: raw.and_then(|e| e.last).map(|d| d.to_string()),
        tick: r.first,
        what: describe(w, &r.action),
        source: source_of(&r.source),
        basis: raw.map_or_else(String::new, |e| basis(&e.basis)),
        fired,
    })
}

fn other(store: &Store, w: &World, e: &Entity, cursor: Option<u64>) -> Option<OtherVm> {
    let tick = report_tick(store, cursor);
    let fact = |k: &str, v: String| (k.to_string(), v);
    match e {
        Entity::Good(key) => {
            let g = w.good(w.id_of::<GoodId>(key.as_str())?)?;
            let mut facts = vec![fact("life", format!("{:?}", g.life))];
            if w.is_currency(g.id) {
                facts.push(fact("currency", "yes".to_string()));
            }
            if w.is_untraded(g.id) {
                facts.push(fact("untraded", "yes: held, never traded".to_string()));
            }
            if let Some(site) = g.price_rate {
                facts.push(fact(
                    "price rate",
                    w.key_of(site.param)
                        .map_or_else(String::new, Key::to_string),
                ));
            }
            let markets: Vec<String> = w
                .markets()
                .filter(|(_, mg)| *mg == g.id)
                .filter_map(|(n, _)| Some(format!("{}/{key}", w.key_of(n)?)))
                .collect();
            facts.push(fact("markets", markets.join(", ")));
            let mut values = vec![value(
                store,
                w,
                "drift over the run",
                SeriesKey {
                    measure: Measure::RunDrift,
                    at: At::Good(key.clone()),
                },
                tick,
            )];
            let mut provs: Vec<Provenance> = store
                .catalogue()
                .iter()
                .filter_map(|k| match &k.at {
                    At::Line { good, prov } if good == key => Some(*prov),
                    _ => None,
                })
                .collect();
            provs.sort();
            provs.dedup();
            for prov in provs {
                values.push(value(
                    store,
                    w,
                    &format!("declared by {prov:?}"),
                    SeriesKey {
                        measure: Measure::Declared,
                        at: At::Line {
                            good: key.clone(),
                            prov,
                        },
                    },
                    tick,
                ));
            }
            Some(OtherVm {
                title: format!("good {key}"),
                facts,
                values,
            })
        }
        Entity::Node(key) => {
            let n = w.node(w.id_of::<NodeId>(key.as_str())?)?;
            let markets: Vec<String> = w
                .markets()
                .filter(|(mn, _)| *mn == n.id)
                .filter_map(|(_, g)| Some(format!("{key}/{}", w.key_of(g)?)))
                .collect();
            let actors: Vec<String> = w
                .actors
                .iter()
                .filter(|a| a.home == n.id)
                .map(|a| a.key.to_string())
                .collect();
            Some(OtherVm {
                title: format!("node {key}"),
                facts: vec![
                    fact(
                        "currency",
                        w.key_of(n.currency)
                            .map_or_else(String::new, Key::to_string),
                    ),
                    fact("markets", markets.join(", ")),
                    fact("home of", actors.join(", ")),
                ],
                values: Vec::new(),
            })
        }
        Entity::Class(key) => {
            let c = w.id_of::<ClassId>(key.as_str())?;
            let actors: Vec<String> = w
                .actors
                .iter()
                .filter(|a| a.class == c)
                .map(|a| a.key.to_string())
                .collect();
            let mut values = Vec::new();
            for k in store.catalogue() {
                if let (Measure::Filled, At::Class { class, .. }) = (&k.measure, &k.at) {
                    if class == key {
                        values.push(value(store, w, &k.to_string(), k.clone(), tick));
                    }
                }
            }
            Some(OtherVm {
                title: format!("class {key}"),
                facts: vec![fact("actors", actors.join(", "))],
                values,
            })
        }
        _ => None,
    }
}

/// The inspector of `selection`, at report tick `cursor` (`None`: live).
pub fn build(store: &Store, selection: &Entity, cursor: Option<u64>) -> Option<InspectorVm> {
    let w = store.world()?;
    let tick = report_tick(store, cursor);
    let vm = match selection {
        Entity::Market { node, good } => {
            market(store, w, node, good, tick).map(|v| InspectorVm::Market(Box::new(v)))
        }
        Entity::Actor(a) => actor(store, w, a, cursor).map(|v| InspectorVm::Actor(Box::new(v))),
        Entity::Param(p) => param(store, w, p, cursor).map(|v| InspectorVm::Param(Box::new(v))),
        Entity::Event(e) => event(store, w, e, cursor).map(|v| InspectorVm::Event(Box::new(v))),
        other_entity => {
            other(store, w, other_entity, cursor).map(|v| InspectorVm::Other(Box::new(v)))
        }
    };
    Some(vm.unwrap_or_else(|| InspectorVm::Unknown(selection.clone())))
}

#[cfg(test)]
mod tests {
    use super::spec_kind;
    use crate::run::{state_fields, StateField};
    use rustyecon_engine::prelude::*;

    #[test]
    fn inspector_reads_planted_states() {
        // LOOPS-RULES §10 (P2.2b.1): the inspector names the three planted kinds, and
        // `state_fields` lists each planted state's desk fields, as its kind's, then its plant
        // record.
        let tape = Tape::from_ron(include_str!("../../../../tapes/loops-lb1.ron")).unwrap();
        let mut sim = Sim::new(&tape).unwrap();
        sim.step().unwrap();
        let w = sim.world().clone();
        let plant = [
            StateField::PlantHeld,
            StateField::PlantTarget,
            StateField::PlantOrder,
            StateField::PlantRun,
            StateField::PlantBuilt,
        ];
        for (key, kind, desk) in [
            (
                "desk.fodder",
                "TypeDesk, planted",
                vec![StateField::Scale, StateField::Output],
            ),
            (
                "desk.maker",
                "Maker, planted",
                vec![
                    StateField::Scale,
                    StateField::Output,
                    StateField::Own,
                    StateField::Serving,
                ],
            ),
            (
                "desk.capacity",
                "CapacityDesk, planted",
                vec![
                    StateField::Scale,
                    StateField::Output,
                    StateField::Held,
                    StateField::Target,
                    StateField::Order,
                    StateField::Run,
                ],
            ),
        ] {
            let id = w.id_of::<ActorId>(key).unwrap();
            assert_eq!(spec_kind(&w.actor(id).unwrap().spec), kind);
            let fields: Vec<StateField> = state_fields(sim.actor_state(id).unwrap())
                .into_iter()
                .map(|(f, _)| f)
                .collect();
            let mut want = desk.clone();
            want.extend(plant);
            assert_eq!(fields, want, "{key}");
            let values = state_fields(sim.actor_state(id).unwrap());
            assert!(values[desk.len()].1 > 0.0, "{key}: the plant held");
        }
        let good = w.id_of::<ActorId>("desk.good").unwrap();
        assert_eq!(spec_kind(&w.actor(good).unwrap().spec), "GoodDesk");
        assert_eq!(StateField::PlantBuilt.name(), "plant.built");
    }
}
