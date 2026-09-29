//! The Extractor (docs/GUI.md §3.4): on the worker, it turns a `Sim` and the tick's
//! `TickReport` into one [`Row`]: a value per series, named by tape key, plus the tick's hash.
//!
//! G0's catalogue is what the report and the read-only accessors give, and nothing computed
//! (U6): every `MarketLine` field; each class line's requested, feasible and filled; each
//! order's settled quantity and value; every holding; every registered param's current value;
//! the tick's audit lines, its largest margin and drift; the run's drift per good and its margin.
//! Fired events travel in the row. A value is stamped with the tick that ran: report fields
//! describe that tick, and holdings and params are read from the state it left.
//!
//! D.3 (2026-09-27) adds two measures the map's lenses read, each the engine's own: whether a
//! market traded (`MarketLine::trades`, 1 or 0) and each actor's own state (`Sim::actor_state`:
//! the provider's due and paid, the workers' share, a desk's share, used, scale and output). It
//! also adds the lean [`Catalogue`], for a world too large to record whole (the demo world's
//! 93 counties, docs/demo/WORLD.md §8): the model chooses it when it loads such a tape.

use super::Row;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_agents::PlantState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A number of an actor's own state (`ActorState`), by its field's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StateField {
    /// A desk's planned human share, or the workers' participation share.
    Share,
    /// The human share a good desk's last production used.
    Used,
    /// A desk's scale.
    Scale,
    /// What a desk's last production made.
    Output,
    /// What the provider's last transfer owed.
    Due,
    /// What it paid.
    Paid,
    /// A maker's serving stock after the last tick's wear (P2.2).
    Own,
    /// A maker's or an owner desk's serving stock this tick (P2.2).
    Serving,
    /// The stock a capacity desk held when this tick's decide ran (P2.2).
    Held,
    /// A capacity desk's target stock, K\* (P2.2).
    Target,
    /// The units of stock a capacity desk ordered (P2.2).
    Order,
    /// The hours a capacity desk planned to run (P2.2).
    Run,
    /// The plant a planted desk held when this tick's decide ran (P2.2b).
    PlantHeld,
    /// A planted desk's plant target, K\*_p (P2.2b).
    PlantTarget,
    /// The plant units a planted desk ordered (P2.2b).
    PlantOrder,
    /// The bundles a planted desk planned to run (P2.2b).
    PlantRun,
    /// The plant units a planted desk's last production built (P2.2b).
    PlantBuilt,
}

impl StateField {
    /// The field's name, as the state's type calls it.
    pub fn name(self) -> &'static str {
        match self {
            StateField::Share => "share",
            StateField::Used => "used",
            StateField::Scale => "scale",
            StateField::Output => "output",
            StateField::Due => "due",
            StateField::Paid => "paid",
            StateField::Own => "own",
            StateField::Serving => "serving",
            StateField::Held => "held",
            StateField::Target => "target",
            StateField::Order => "order",
            StateField::Run => "run",
            StateField::PlantHeld => "plant.held",
            StateField::PlantTarget => "plant.target",
            StateField::PlantOrder => "plant.order",
            StateField::PlantRun => "plant.run",
            StateField::PlantBuilt => "plant.built",
        }
    }
}

/// An actor state's numbers, by field, in the order the state's type lists them. A scripted
/// actor's state holds none. A planted desk's (P2.2b) are its kind's, then its plant's record.
pub fn state_fields(s: &ActorState) -> Vec<(StateField, f64)> {
    let planted = |desk: ActorState, p: &PlantState| {
        let mut v = state_fields(&desk);
        v.extend([
            (StateField::PlantHeld, p.held),
            (StateField::PlantTarget, p.target),
            (StateField::PlantOrder, p.order),
            (StateField::PlantRun, p.run),
            (StateField::PlantBuilt, p.built),
        ]);
        v
    };
    match s {
        ActorState::Scripted(_) => Vec::new(),
        ActorState::Provider(p) => vec![(StateField::Due, p.due), (StateField::Paid, p.paid)],
        ActorState::Workers(w) => vec![(StateField::Share, w.share)],
        ActorState::GoodDesk(d) => vec![
            (StateField::Share, d.share),
            (StateField::Used, d.used),
            (StateField::Scale, d.scale),
            (StateField::Output, d.output),
        ],
        ActorState::MachDesk(d) => {
            vec![(StateField::Scale, d.scale), (StateField::Output, d.output)]
        }
        ActorState::Maker(d) => vec![
            (StateField::Scale, d.scale),
            (StateField::Output, d.output),
            (StateField::Own, d.own),
            (StateField::Serving, d.serving),
        ],
        ActorState::Capacity(d) => vec![
            (StateField::Scale, d.scale),
            (StateField::Output, d.output),
            (StateField::Held, d.held),
            (StateField::Target, d.target),
            (StateField::Order, d.order),
            (StateField::Run, d.run),
        ],
        ActorState::Owner(d) => vec![
            (StateField::Share, d.share),
            (StateField::Used, d.used),
            (StateField::Scale, d.scale),
            (StateField::Output, d.output),
            (StateField::Serving, d.serving),
        ],
        ActorState::PlantedType(d) => planted(ActorState::MachDesk(d.desk), &d.plant),
        ActorState::PlantedMaker(d) => planted(ActorState::Maker(d.desk), &d.plant),
        ActorState::PlantedCapacity(d) => planted(ActorState::Capacity(d.desk), &d.plant),
    }
}

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
    /// `MarketLine::trades`: 1 when the market traded in the tick, 0 when it did not (D.3).
    Trades,
    /// A number of an actor's own state after the tick, `Sim::actor_state` (D.3).
    State(StateField),
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
            Measure::Trades => "trades",
            Measure::State(f) => f.name(),
        }
    }
}

/// What an Extractor records (docs/GUI.md §3.4, the catalogue filter of §10, brought forward
/// at D.3). A gap in a series is still a tick with no value; a series the catalogue leaves out
/// is simply not in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Catalogue {
    /// Everything: G0's catalogue with D.3's two measures.
    #[default]
    Full,
    /// For a world too large to record whole: each market's price, supply, demand, cleared
    /// volume and whether it traded; each class line's requested and filled; each actor's own
    /// state; every registered param; the run's margins and the tick's largest drift. No next
    /// price, EMA or fills, no feasible line, settle line, holding, audit line or drift by good.
    Lean,
}

impl Catalogue {
    /// A world of more nodes than this records the lean catalogue. The gate world has two and
    /// the Appendix B world one; the demo world has 93, whose whole catalogue would be 11,447
    /// series (1,116 settle lines each of quantity and value, 619 holdings, …).
    pub const LEAN_ABOVE: usize = 16;

    /// The catalogue for a world of `nodes` nodes.
    pub fn for_nodes(nodes: usize) -> Catalogue {
        if nodes > Catalogue::LEAN_ABOVE {
            Catalogue::Lean
        } else {
            Catalogue::Full
        }
    }

    /// Whether it records a measure at all.
    pub fn records(&self, m: Measure) -> bool {
        match self {
            Catalogue::Full => true,
            Catalogue::Lean => matches!(
                m,
                Measure::Price
                    | Measure::Supply
                    | Measure::Demand
                    | Measure::Cleared
                    | Measure::Trades
                    | Measure::Requested
                    | Measure::Filled
                    | Measure::State(_)
                    | Measure::Param
                    | Measure::TickMargin
                    | Measure::TickDrift
                    | Measure::RunMargin
            ),
        }
    }

    /// Whether it is the lean one.
    pub fn is_lean(&self) -> bool {
        matches!(self, Catalogue::Lean)
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
    /// An actor's own state (D.3).
    Actor(Key),
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

impl SeriesKey {
    /// Whether every entity the key names is in `w`, so that a run of `w` can record it. A
    /// session's plots name series by key, and a key of another tape's world names nothing in
    /// this one (U7).
    pub fn in_world(&self, w: &World) -> bool {
        let node = |k: &Key| w.id_of::<NodeId>(k.as_str()).is_some();
        let good = |k: &Key| w.id_of::<GoodId>(k.as_str()).is_some();
        let actor = |k: &Key| w.id_of::<ActorId>(k.as_str()).is_some();
        match &self.at {
            At::World => true,
            At::Market { node: n, good: g } => node(n) && good(g),
            At::Class {
                node: n,
                good: g,
                class,
                ..
            } => node(n) && good(g) && w.id_of::<ClassId>(class.as_str()).is_some(),
            At::Order {
                actor: a,
                node: n,
                good: g,
                ..
            } => actor(a) && node(n) && good(g),
            At::Holding { holder, good: g } => {
                good(g)
                    && match holder {
                        HolderKey::Actor(a) => actor(a),
                        HolderKey::Escrow { node: n, good: e } => node(n) && good(e),
                    }
            }
            At::Param(p) => {
                w.id_of::<ParamId>(p.as_str()).is_some() || w.schedule.param(p.as_str()).is_some()
            }
            At::Good(g) | At::Line { good: g, .. } => good(g),
            At::Actor(a) => actor(a),
        }
    }
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
            At::Actor(a) => write!(f, "{m} of {a}"),
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
    Actor(ActorId),
}

/// Turns each tick into a [`Row`], numbering series as they first appear.
#[derive(Debug, Clone)]
pub struct Extractor {
    world: World,
    catalogue: Catalogue,
    params: Vec<ParamId>,
    actors: Vec<ActorId>,
    index: BTreeMap<(Measure, Slot), u32>,
    series: Vec<SeriesKey>,
    /// The last tick's cells in the order they were made, so a tick of the same shape finds
    /// each index without a search.
    layout: Vec<((Measure, Slot), u32)>,
    next: Vec<((Measure, Slot), u32)>,
}

impl Extractor {
    /// An Extractor for runs of `world` that records G0's whole catalogue, with an empty
    /// catalogue so far.
    pub fn new(world: &World) -> Extractor {
        Extractor::with(world, Catalogue::Full)
    }

    /// An Extractor for runs of `world` that records `catalogue`.
    pub fn with(world: &World, catalogue: Catalogue) -> Extractor {
        let params = world.registry.params().iter().map(|p| p.id).collect();
        Extractor {
            world: world.clone(),
            catalogue,
            params,
            actors: world.actors.iter().map(|a| a.id).collect(),
            index: BTreeMap::new(),
            series: Vec::new(),
            layout: Vec::new(),
            next: Vec::new(),
        }
    }

    /// Every series seen so far, by catalogue index.
    pub fn catalogue(&self) -> &[SeriesKey] {
        &self.series
    }

    /// What it records.
    pub fn records(&self) -> &Catalogue {
        &self.catalogue
    }

    /// The row of the tick `r` reports, read from `r` and from `sim` after it. Series seen for
    /// the first time are appended to `new`, in catalogue order.
    pub fn row(&mut self, sim: &Sim, r: &TickReport, new: &mut Vec<SeriesKey>) -> Row {
        let mut cells = Vec::with_capacity(self.layout.len());
        self.next.clear();
        let full = !self.catalogue.is_lean();
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
                if full || self.catalogue.records(measure) {
                    cells.push((self.slot(measure, at, new), v));
                }
            }
            let traded = if m.trades() { 1.0 } else { 0.0 };
            cells.push((self.slot(Measure::Trades, at, new), traded));
        }
        for l in &r.rationing {
            let at = Slot::Class(l.node, l.good, l.class, l.side);
            for (measure, v) in [
                (Measure::Requested, l.requested),
                (Measure::Feasible, l.feasible),
                (Measure::Filled, l.filled),
            ] {
                if full || self.catalogue.records(measure) {
                    cells.push((self.slot(measure, at, new), v));
                }
            }
        }
        if full {
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
        }
        let params = std::mem::take(&mut self.params);
        for &p in &params {
            if let Some(v) = sim.param(p) {
                cells.push((self.slot(Measure::Param, Slot::Param(p), new), v));
            }
        }
        self.params = params;
        if full {
            for (good, prov, v) in &r.audit.lines {
                cells.push((
                    self.slot(Measure::Declared, Slot::Line(*good, *prov), new),
                    *v,
                ));
            }
            for (good, d) in &r.run.drift {
                cells.push((self.slot(Measure::RunDrift, Slot::Good(*good), new), *d));
            }
        }
        for (measure, v) in [
            (Measure::TickMargin, r.audit.max_margin),
            (Measure::TickDrift, r.audit.max_drift),
            (Measure::RunMargin, r.run.max_margin),
        ] {
            cells.push((self.slot(measure, Slot::World, new), v));
        }
        let actors = std::mem::take(&mut self.actors);
        for &a in &actors {
            if let Some(s) = sim.actor_state(a) {
                for (f, v) in state_fields(s) {
                    cells.push((self.slot(Measure::State(f), Slot::Actor(a), new), v));
                }
            }
        }
        self.actors = actors;
        std::mem::swap(&mut self.layout, &mut self.next);
        Row {
            tick: r.tick,
            hash: r.hash,
            cells,
            events: r.events.clone(),
        }
    }

    /// The catalogue index of `(measure, slot)`, numbering it if it is new. A tick shaped as
    /// the last one finds it at the same place in the last tick's layout.
    fn slot(&mut self, measure: Measure, slot: Slot, new: &mut Vec<SeriesKey>) -> u32 {
        let want = (measure, slot);
        let i = match self.layout.get(self.next.len()) {
            Some(&(k, i)) if k == want => i,
            _ => self.lookup(want, new),
        };
        self.next.push((want, i));
        i
    }

    fn lookup(&mut self, want: (Measure, Slot), new: &mut Vec<SeriesKey>) -> u32 {
        if let Some(&i) = self.index.get(&want) {
            return i;
        }
        let key = SeriesKey {
            measure: want.0,
            at: self.at(want.1),
        };
        let i = u32::try_from(self.series.len()).expect("fewer than 2^32 series");
        self.series.push(key.clone());
        new.push(key);
        self.index.insert(want, i);
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
            Slot::Actor(a) => At::Actor(actor(a)),
        }
    }
}
