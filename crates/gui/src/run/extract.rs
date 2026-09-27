//! The Extractor (docs/GUI.md §3.4): on the worker, it turns a `Sim` and the tick's
//! `TickReport` into one [`Row`]: a value per series, named by tape key, plus the tick's hash.
//!
//! G0's catalogue is what the report and the read-only accessors give, and nothing computed
//! (U6): every `MarketLine` field; each class line's requested, feasible and filled; each
//! order's settled quantity and value; every holding; every registered param's current value;
//! the tick's audit lines, its largest margin and drift; the run's drift per good and its margin.
//! Fired events travel in the row. A value is stamped with the tick that ran: report fields
//! describe that tick, and holdings and params are read from the state it left.

use super::Row;
use rustyecon_engine::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// What a series measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Measure {
    /// `MarketLine::price`: posted, used by the tick's settlement.
    Price,
    /// `MarketLine::next_price`: posted for the next tick.
    NextPrice,
    /// `MarketLine::ema`, after the update.
    Ema,
    /// `MarketLine::supply`, `S`.
    Supply,
    /// `MarketLine::demand`, feasible `D`.
    Demand,
    /// `MarketLine::cleared`: the good that left the escrow.
    Cleared,
    /// `MarketLine::buyer_fill`.
    BuyerFill,
    /// `MarketLine::seller_fill`.
    SellerFill,
    /// `RationLine::requested`: what a class asked for or offered.
    Requested,
    /// `RationLine::feasible`: what its budgets could pay for, or what it offered.
    Feasible,
    /// `RationLine::filled`: what it received or shipped.
    Filled,
    /// `SettleLine::qty`: an order's good received or shipped.
    SettledQty,
    /// `SettleLine::value`: an order's currency paid or received.
    SettledValue,
    /// A holder's total of a good, after the tick.
    Held,
    /// A registered param's value, after the tick.
    Param,
    /// The tick's net declared quantity of a good by provenance (`TickAudit::lines`).
    Declared,
    /// The tick's largest margin, `|drift|/tol` over goods (`TickAudit::max_margin`).
    TickMargin,
    /// The tick's largest `|drift|` over goods (`TickAudit::max_drift`).
    TickDrift,
    /// The run's drift of a good since its first tick (`RunAudit::drift`).
    RunDrift,
    /// The run's largest margin (`RunAudit::max_margin`).
    RunMargin,
}

impl Measure {
    /// The measure's name, as the report's field is called.
    pub fn name(self) -> &'static str {
        match self {
            Measure::Price => "price",
            Measure::NextPrice => "next_price",
            Measure::Ema => "ema",
            Measure::Supply => "supply",
            Measure::Demand => "demand",
            Measure::Cleared => "cleared",
            Measure::BuyerFill => "buyer_fill",
            Measure::SellerFill => "seller_fill",
            Measure::Requested => "requested",
            Measure::Feasible => "feasible",
            Measure::Filled => "filled",
            Measure::SettledQty => "settled_qty",
            Measure::SettledValue => "settled_value",
            Measure::Held => "held",
            Measure::Param => "param",
            Measure::Declared => "declared",
            Measure::TickMargin => "tick_margin",
            Measure::TickDrift => "tick_drift",
            Measure::RunDrift => "run_drift",
            Measure::RunMargin => "run_margin",
        }
    }
}

/// A holder by key: an actor, or the escrow of a market.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HolderKey {
    /// An actor.
    Actor(Key),
    /// The settlement escrow of (node, good).
    Escrow {
        /// The node.
        node: Key,
        /// The good.
        good: Key,
    },
}

/// Where a series is measured, by tape key (U7).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum At {
    /// The whole run.
    World,
    /// A market, (node, good).
    Market {
        /// The node.
        node: Key,
        /// The good.
        good: Key,
    },
    /// A class's line in a market (R12).
    Class {
        /// The node.
        node: Key,
        /// The good.
        good: Key,
        /// The class.
        class: Key,
        /// The side.
        side: SideTag,
    },
    /// An actor's order in a market.
    Order {
        /// The actor.
        actor: Key,
        /// The node.
        node: Key,
        /// The good.
        good: Key,
        /// The side.
        side: SideTag,
    },
    /// A holder's stock of a good.
    Holding {
        /// The holder.
        holder: HolderKey,
        /// The good.
        good: Key,
    },
    /// A registered param.
    Param(Key),
    /// A good.
    Good(Key),
    /// A good's ledger line by provenance.
    Line {
        /// The good.
        good: Key,
        /// The provenance.
        prov: Provenance,
    },
}

/// A tape entity by key (U7): what a selection or a pin names. Dense ids are resolved against
/// one run's `World` when they are used.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Entity {
    /// A good.
    Good(Key),
    /// A node.
    Node(Key),
    /// A market, (node, good).
    Market {
        /// The node.
        node: Key,
        /// The good.
        good: Key,
    },
    /// A class.
    Class(Key),
    /// An actor, desk or pop.
    Actor(Key),
    /// A registered param.
    Param(Key),
    /// A dated or recurring event.
    Event(Key),
}

/// A series' name: what it measures and where, by tape key (docs/GUI.md §3.4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SeriesKey {
    /// What it measures.
    pub measure: Measure,
    /// Where.
    pub at: At,
}

impl fmt::Display for SeriesKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let side = |s: &SideTag| match s {
            SideTag::Buy => "buy",
            SideTag::Sell => "sell",
        };
        let m = self.measure.name();
        match &self.at {
            At::World => write!(f, "{m}"),
            At::Market { node, good } => write!(f, "{m} at {node}/{good}"),
            At::Class {
                node,
                good,
                class,
                side: s,
            } => write!(f, "{m} of {class} ({}) at {node}/{good}", side(s)),
            At::Order {
                actor,
                node,
                good,
                side: s,
            } => write!(f, "{m} of {actor} ({}) at {node}/{good}", side(s)),
            At::Holding {
                holder: HolderKey::Actor(a),
                good,
            } => write!(f, "{m} of {good} by {a}"),
            At::Holding {
                holder: HolderKey::Escrow { node, good: g },
                good,
            } => write!(f, "{m} of {good} in the escrow of {node}/{g}"),
            At::Param(p) => write!(f, "{m} {p}"),
            At::Good(g) => write!(f, "{m} of {g}"),
            At::Line { good, prov } => write!(f, "{m} of {good} by {prov:?}"),
        }
    }
}

/// Where a value sits, by dense id: the Extractor's own index, cheap to compare every tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    World,
    Market(NodeId, GoodId),
    Class(NodeId, GoodId, ClassId, SideTag),
    Order(ActorId, NodeId, GoodId, SideTag),
    Holding(Holder, GoodId),
    Param(ParamId),
    Good(GoodId),
    Line(GoodId, Provenance),
}

/// Turns each tick into a [`Row`], numbering series as they first appear.
#[derive(Debug, Clone)]
pub struct Extractor {
    world: World,
    params: Vec<ParamId>,
    index: BTreeMap<(Measure, Slot), u32>,
    catalogue: Vec<SeriesKey>,
}

impl Extractor {
    /// An Extractor for runs of `world`, with an empty catalogue.
    pub fn new(world: &World) -> Extractor {
        Extractor {
            world: world.clone(),
            params: world.registry.params().iter().map(|p| p.id).collect(),
            index: BTreeMap::new(),
            catalogue: Vec::new(),
        }
    }

    /// Every series seen so far, by catalogue index.
    pub fn catalogue(&self) -> &[SeriesKey] {
        &self.catalogue
    }

    /// The row of the tick `r` reports, read from `r` and from `sim` after it. Series seen for
    /// the first time are appended to `new`, in catalogue order.
    pub fn row(&mut self, sim: &Sim, r: &TickReport, new: &mut Vec<SeriesKey>) -> Row {
        let mut cells = Vec::new();
        for m in &r.markets {
            let at = Slot::Market(m.node, m.good);
            for (measure, v) in [
                (Measure::Price, m.price),
                (Measure::NextPrice, m.next_price),
                (Measure::Ema, m.ema),
                (Measure::Supply, m.supply),
                (Measure::Demand, m.demand),
                (Measure::Cleared, m.cleared),
                (Measure::BuyerFill, m.buyer_fill),
                (Measure::SellerFill, m.seller_fill),
            ] {
                cells.push((self.slot(measure, at, new), v));
            }
        }
        for l in &r.rationing {
            let at = Slot::Class(l.node, l.good, l.class, l.side);
            for (measure, v) in [
                (Measure::Requested, l.requested),
                (Measure::Feasible, l.feasible),
                (Measure::Filled, l.filled),
            ] {
                cells.push((self.slot(measure, at, new), v));
            }
        }
        for s in &r.settlements {
            let at = Slot::Order(s.actor, s.node, s.good, s.side);
            cells.push((self.slot(Measure::SettledQty, at, new), s.qty));
            cells.push((self.slot(Measure::SettledValue, at, new), s.value));
        }
        for (holder, good, q) in sim.observe_holdings().0 {
            cells.push((
                self.slot(Measure::Held, Slot::Holding(holder, good), new),
                q,
            ));
        }
        let params = std::mem::take(&mut self.params);
        for &p in &params {
            if let Some(v) = sim.param(p) {
                cells.push((self.slot(Measure::Param, Slot::Param(p), new), v));
            }
        }
        self.params = params;
        for (good, prov, v) in &r.audit.lines {
            cells.push((
                self.slot(Measure::Declared, Slot::Line(*good, *prov), new),
                *v,
            ));
        }
        for (good, d) in &r.run.drift {
            cells.push((self.slot(Measure::RunDrift, Slot::Good(*good), new), *d));
        }
        for (measure, v) in [
            (Measure::TickMargin, r.audit.max_margin),
            (Measure::TickDrift, r.audit.max_drift),
            (Measure::RunMargin, r.run.max_margin),
        ] {
            cells.push((self.slot(measure, Slot::World, new), v));
        }
        Row {
            tick: r.tick,
            hash: r.hash,
            cells,
            events: r.events.clone(),
        }
    }

    /// The catalogue index of `(measure, slot)`, numbering it if it is new.
    fn slot(&mut self, measure: Measure, slot: Slot, new: &mut Vec<SeriesKey>) -> u32 {
        if let Some(&i) = self.index.get(&(measure, slot)) {
            return i;
        }
        let key = SeriesKey {
            measure,
            at: self.at(slot),
        };
        let i = u32::try_from(self.catalogue.len()).expect("fewer than 2^32 series");
        self.catalogue.push(key.clone());
        new.push(key);
        self.index.insert((measure, slot), i);
        i
    }

    /// A slot by tape key. Every id in a report is the world's, so each has a key; one that did
    /// not would name the id itself rather than fail.
    fn at(&self, slot: Slot) -> At {
        let w = &self.world;
        let key = |k: Option<&Key>, id: String| {
            k.cloned()
                .unwrap_or_else(|| Key::new(id.replace('#', ".")).expect("a valid key"))
        };
        let good = |g: GoodId| key(w.key_of(g), g.to_string());
        let node = |n: NodeId| key(w.key_of(n), n.to_string());
        let actor = |a: ActorId| key(w.key_of(a), a.to_string());
        match slot {
            Slot::World => At::World,
            Slot::Market(n, g) => At::Market {
                node: node(n),
                good: good(g),
            },
            Slot::Class(n, g, c, side) => At::Class {
                node: node(n),
                good: good(g),
                class: key(w.key_of(c), c.to_string()),
                side,
            },
            Slot::Order(a, n, g, side) => At::Order {
                actor: actor(a),
                node: node(n),
                good: good(g),
                side,
            },
            Slot::Holding(h, g) => At::Holding {
                holder: match h {
                    Holder::Actor(a) => HolderKey::Actor(actor(a)),
                    Holder::Escrow(n, eg) => HolderKey::Escrow {
                        node: node(n),
                        good: good(eg),
                    },
                },
                good: good(g),
            },
            Slot::Param(p) => At::Param(key(w.key_of(p), p.to_string())),
            Slot::Good(g) => At::Good(good(g)),
            Slot::Line(g, prov) => At::Line {
                good: good(g),
                prov,
            },
        }
    }
}
