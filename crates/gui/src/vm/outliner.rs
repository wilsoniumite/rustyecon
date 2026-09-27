//! The outliner's view-model (docs/GUI.md §4): the tape's entities by key (U7), each with
//! whether it is selected and pinned, and the series its "plot" button plots.
//!
//! The pinned entities come first, in the order they were pinned; then markets, actors, goods,
//! nodes, classes, params and events, each in key order. An entity's own series is its price
//! for a market, its holding of its home currency for an actor, its value for a registered
//! param, and its drift over the run for a good. Nodes, classes, events and schedule params
//! have none.

use super::{firing_date, param_unit};
use crate::run::{At, Entity, HolderKey, Measure, SeriesKey, Store};
use rustyecon_engine::prelude::*;
use serde::Serialize;

/// One entity's row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RowVm {
    /// The entity, by key.
    pub entity: Entity,
    /// Its key, as the row shows it: `town/bread` for a market.
    pub label: String,
    /// What it is: a unit, a kind, a date.
    pub detail: String,
    /// Whether it is the selection.
    pub selected: bool,
    /// Whether it is pinned.
    pub pinned: bool,
    /// Its own series, if it has one.
    pub series: Option<SeriesKey>,
    /// Whether that series is plotted.
    pub plotted: bool,
}

/// One kind of entity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SectionVm {
    /// The kind: "Markets", "Actors", …
    pub title: String,
    /// Its entities, in key order.
    pub rows: Vec<RowVm>,
}

/// The outliner.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutlinerVm {
    /// The pinned entities, in the order they were pinned.
    pub pinned: Vec<RowVm>,
    /// Every entity, by kind.
    pub sections: Vec<SectionVm>,
}

/// An entity's own series, from the world alone, so it can be plotted before any tick ran.
pub fn own_series(w: &World, e: &Entity) -> Option<SeriesKey> {
    let (measure, at) = match e {
        Entity::Market { node, good } => (
            Measure::Price,
            At::Market {
                node: node.clone(),
                good: good.clone(),
            },
        ),
        Entity::Actor(a) => {
            let decl = w.actor(w.id_of::<ActorId>(a.as_str())?)?;
            let cur = w.node(decl.home)?.currency;
            (
                Measure::Held,
                At::Holding {
                    holder: HolderKey::Actor(a.clone()),
                    good: w.key_of(cur)?.clone(),
                },
            )
        }
        Entity::Param(p) => {
            w.id_of::<ParamId>(p.as_str())?;
            (Measure::Param, At::Param(p.clone()))
        }
        Entity::Good(g) => (Measure::RunDrift, At::Good(g.clone())),
        Entity::Node(_) | Entity::Class(_) | Entity::Event(_) => return None,
    };
    Some(SeriesKey { measure, at })
}

/// An entity with its label and detail.
type Named = (Entity, String, String);

/// Every entity of the world, by kind, each with its label and detail.
fn entities(w: &World) -> Vec<(&'static str, Vec<Named>)> {
    let key = |k: Option<&Key>| k.cloned();
    let mut markets = Vec::new();
    for (n, g) in w.markets() {
        if let (Some(node), Some(good)) = (key(w.key_of(n)), key(w.key_of(g))) {
            let cur = w.node(n).and_then(|d| w.key_of(d.currency));
            let detail = cur.map_or_else(String::new, |c| format!("{c} per {good}"));
            markets.push((
                Entity::Market {
                    node: node.clone(),
                    good: good.clone(),
                },
                format!("{node}/{good}"),
                detail,
            ));
        }
    }
    markets.sort_by(|a, b| a.1.cmp(&b.1));
    let mut actors: Vec<(Entity, String, String)> = w
        .actors
        .iter()
        .map(|a| {
            let class = w.key_of(a.class).map_or("", Key::as_str);
            let home = w.key_of(a.home).map_or("", Key::as_str);
            (
                Entity::Actor(a.key.clone()),
                a.key.to_string(),
                format!("{:?}, {class}, at {home}", a.id.kind()),
            )
        })
        .collect();
    actors.sort_by(|a, b| a.1.cmp(&b.1));
    let goods = w
        .goods
        .iter()
        .map(|g| {
            let detail = if w.is_currency(g.id) {
                "currency".to_string()
            } else {
                format!("{:?}", g.life)
            };
            (Entity::Good(g.key.clone()), g.key.to_string(), detail)
        })
        .collect();
    let nodes = w
        .nodes
        .iter()
        .map(|n| {
            let cur = w.key_of(n.currency).map_or("", Key::as_str);
            (
                Entity::Node(n.key.clone()),
                n.key.to_string(),
                format!("currency {cur}"),
            )
        })
        .collect();
    let classes = w
        .classes
        .iter()
        .map(|c| (Entity::Class(c.clone()), c.to_string(), String::new()))
        .collect();
    let mut params: Vec<(Entity, String, String)> = w
        .registry
        .params()
        .iter()
        .map(|p| &p.key)
        .chain(w.schedule.params().iter().map(|p| &p.key))
        .map(|k| (Entity::Param(k.clone()), k.to_string(), param_unit(w, k)))
        .collect();
    params.sort_by(|a, b| a.1.cmp(&b.1));
    let mut events: Vec<(Entity, String, String)> = w
        .schedule
        .once()
        .iter()
        .filter_map(|f| {
            let k = w.key_of(f.event)?;
            Some((
                Entity::Event(k.clone()),
                k.to_string(),
                firing_date(w, f.tick, k),
            ))
        })
        .chain(w.schedule.every().iter().filter_map(|r| {
            let k = w.key_of(r.event)?;
            Some((
                Entity::Event(k.clone()),
                k.to_string(),
                format!(
                    "from {}, every {} ticks",
                    firing_date(w, r.first, k),
                    r.period
                ),
            ))
        }))
        .collect();
    events.sort_by(|a, b| a.1.cmp(&b.1));
    vec![
        ("Markets", markets),
        ("Actors", actors),
        ("Goods", goods),
        ("Nodes", nodes),
        ("Classes", classes),
        ("Params", params),
        ("Events", events),
    ]
}

/// The outliner of a run, given the selection, the pins and the plotted series.
pub fn build(
    store: &Store,
    selection: Option<&Entity>,
    pins: &[Entity],
    plots: &[SeriesKey],
) -> Option<OutlinerVm> {
    let w = store.world()?;
    let row = |(entity, label, detail): (Entity, String, String)| {
        let series = own_series(w, &entity);
        RowVm {
            selected: selection == Some(&entity),
            pinned: pins.contains(&entity),
            plotted: series.as_ref().is_some_and(|s| plots.contains(s)),
            series,
            entity,
            label,
            detail,
        }
    };
    let sections: Vec<SectionVm> = entities(w)
        .into_iter()
        .map(|(title, rows)| SectionVm {
            title: title.to_string(),
            rows: rows.into_iter().map(row).collect(),
        })
        .collect();
    let pinned = pins
        .iter()
        .filter_map(|p| {
            sections
                .iter()
                .flat_map(|s| &s.rows)
                .find(|r| &r.entity == p)
                .cloned()
        })
        .collect();
    Some(OutlinerVm { pinned, sections })
}
