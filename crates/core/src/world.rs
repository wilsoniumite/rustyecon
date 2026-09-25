//! The resolved world: everything a run reads and never writes (docs/ENGINE.md §2.1 and §2.6).
//!
//! A `World` comes only from [`crate::resolve`]. Its dense ids are numbered per kind in key byte
//! order, and it maps both ways between ids and keys ([`World::id_of`], [`World::key_of`]).

use crate::clock::Clock;
use crate::delta::StateDelta;
use crate::ext::Ext;
use crate::hash::Fnv;
use crate::ids::{
    ActorId, ChannelId, ClassId, DeskId, EventId, GoodId, Holder, Key, NodeId, ParamId, PopId,
};
use crate::registry::Registry;
use serde::{Deserialize, Serialize};

/// A good's shelf life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Life {
    /// Never spoils. Every currency is indefinite.
    Indefinite,
    /// Dies at the ageing of the tick it was minted in; minted only in phases 0 and 1.
    Instant,
    /// Lasts this many ticks (at least 1), from a fixed `Years` param.
    Ticks(u32),
}

impl Life {
    /// The life a newly minted lot starts with: `Some(0)` for Instant, `Some(L)` for `Ticks(L)`,
    /// `None` for indefinite.
    pub fn initial(self) -> Option<u32> {
        match self {
            Life::Indefinite => None,
            Life::Instant => Some(0),
            Life::Ticks(l) => Some(l),
        }
    }
}

/// A good.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoodDef {
    /// Its dense id.
    pub id: GoodId,
    /// Its key.
    pub key: Key,
    /// Its shelf life.
    pub life: Life,
    /// The registered `RatePerYear` its price moves at; `None` exactly for a currency.
    pub price_rate: Option<ParamId>,
}

/// A market node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDef {
    /// Its dense id.
    pub id: NodeId,
    /// Its key.
    pub key: Key,
    /// The good its prices are quoted in.
    pub currency: GoodId,
}

/// A static channel between two nodes. Nothing crosses one in Phase 0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelDef {
    /// Its dense id.
    pub id: ChannelId,
    /// Its key.
    pub key: Key,
    /// The node it leaves.
    pub from: NodeId,
    /// The node it reaches.
    pub to: NodeId,
}

/// A declared actor. The actor set is fixed at load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorDecl<A> {
    /// Its id.
    pub id: ActorId,
    /// Its key; desks and pops share one namespace.
    pub key: Key,
    /// Its registered class.
    pub class: ClassId,
    /// Its home node.
    pub home: NodeId,
    /// Its resolved spec, the extension's.
    pub spec: A,
}

/// The ledger's registered tolerances (A12): both `Dimensionless` and fixed, with no absolute
/// term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tolerances {
    /// Slack per unit of gross flow.
    pub rel_flow: ParamId,
    /// Slack per unit of stock held.
    pub rel_stock: ParamId,
}

/// How prices move (N13). Required on the tape; there is no default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceRule {
    /// `p' = p·exp(k·x)` on the imbalance `x`.
    Imbalance,
    /// `p' = p·D/S` when both sides trade.
    Ratio,
}

/// What a one-sided market does to its price (F8). Required on the tape; there is no default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneSided {
    /// The imbalance saturates at ±1, as in July.
    Saturate,
    /// The price holds: a one-sided market carries no price evidence.
    Hold,
}

/// The market settings of a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketConfig {
    /// The price rule.
    pub rule: PriceRule,
    /// The one-sided rule.
    pub one_sided: OneSided,
    /// The EMA's time constant, a live `Years` param.
    pub ema_time_constant: ParamId,
}

/// One firing of a tape event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Firing<E: Ext> {
    /// The tick it fires in.
    pub tick: u64,
    /// The event.
    pub event: EventId,
    /// 0 for a dated event; k for the k-th firing of a recurring one.
    pub occurrence: u32,
    /// The resolved action.
    pub action: StateDelta<E>,
    /// For a `SetParam`, the param its value was copied from, which carries the value's basis.
    pub source: Option<ParamId>,
}

/// A recurring tape entry: it fires at `first + k·period` for k = 0, 1, … up to `last`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Recurring<E: Ext> {
    /// The event.
    pub event: EventId,
    /// The tick of its first firing.
    pub first: u64,
    /// Its period in ticks, at least 1.
    pub period: u64,
    /// The last tick it may fire in.
    pub last: Option<u64>,
    /// The resolved action.
    pub action: StateDelta<E>,
    /// For a `SetParam`, the param its value was copied from.
    pub source: Option<ParamId>,
}

impl<E: Ext> Recurring<E> {
    /// The occurrence due at `tick`, if one is.
    pub fn occurrence_at(&self, tick: u64) -> Option<u32> {
        if tick < self.first || self.last.is_some_and(|l| tick > l) {
            return None;
        }
        let since = tick - self.first;
        if !since.is_multiple_of(self.period) {
            return None;
        }
        Some(u32::try_from(since / self.period).unwrap_or(u32::MAX))
    }

    fn firing(&self, tick: u64, occurrence: u32) -> Firing<E> {
        Firing {
            tick,
            event: self.event,
            occurrence,
            action: self.action.clone(),
            source: self.source,
        }
    }
}

/// The tape's events, resolved and sorted (N1).
#[derive(Debug, Clone, PartialEq)]
pub struct Schedule<E: Ext> {
    /// Dated events, sorted by (tick, key).
    once: Vec<Firing<E>>,
    /// Recurring entries, by key.
    every: Vec<Recurring<E>>,
}

impl<E: Ext> Schedule<E> {
    pub(crate) fn new(mut once: Vec<Firing<E>>, mut every: Vec<Recurring<E>>) -> Schedule<E> {
        once.sort_by_key(|f| (f.tick, f.event));
        every.sort_by_key(|r| r.event);
        Schedule { once, every }
    }

    /// Everything that fires at `tick`, in key order: the dated events found by
    /// `partition_point` (the list is sorted, unlike July's, `v2p3: scenario/mod.rs:25-42`),
    /// merged with the recurring entries due.
    pub fn fire(&self, tick: u64) -> Vec<Firing<E>> {
        let lo = self.once.partition_point(|f| f.tick < tick);
        let hi = self.once.partition_point(|f| f.tick <= tick);
        let mut out: Vec<Firing<E>> = self.once[lo..hi].to_vec();
        for r in &self.every {
            if let Some(k) = r.occurrence_at(tick) {
                out.push(r.firing(tick, k));
            }
        }
        out.sort_by_key(|f| f.event);
        out
    }

    /// Every firing with `tick < until`, in firing order: by tick, then key.
    pub fn firings_before(&self, until: u64) -> Vec<Firing<E>> {
        let hi = self.once.partition_point(|f| f.tick < until);
        let mut out: Vec<Firing<E>> = self.once[..hi].to_vec();
        for r in &self.every {
            let mut tick = r.first;
            let mut k: u32 = 0;
            while tick < until && r.last.is_none_or(|l| tick <= l) {
                out.push(r.firing(tick, k));
                k = k.saturating_add(1);
                match tick.checked_add(r.period) {
                    Some(t) => tick = t,
                    None => break,
                }
            }
        }
        out.sort_by_key(|f| (f.tick, f.event));
        out
    }

    /// The dated events, sorted by (tick, key).
    pub fn once(&self) -> &[Firing<E>] {
        &self.once
    }

    /// The recurring entries, by key.
    pub fn every(&self) -> &[Recurring<E>] {
        &self.every
    }
}

/// The key lists of a world, each sorted, so position is id and lookup is a binary search.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyIndex {
    pub(crate) goods: Vec<Key>,
    pub(crate) nodes: Vec<Key>,
    pub(crate) channels: Vec<Key>,
    pub(crate) classes: Vec<Key>,
    pub(crate) params: Vec<Key>,
    pub(crate) desks: Vec<Key>,
    pub(crate) pops: Vec<Key>,
    pub(crate) events: Vec<Key>,
}

fn find(keys: &[Key], key: &str) -> Option<u32> {
    keys.binary_search_by(|k| k.as_str().cmp(key))
        .ok()
        .and_then(|i| u32::try_from(i).ok())
}

impl KeyIndex {
    pub(crate) fn actor(&self, key: &str) -> Option<ActorId> {
        find(&self.desks, key)
            .map(|i| ActorId::Desk(DeskId(i)))
            .or_else(|| find(&self.pops, key).map(|i| ActorId::Pop(PopId(i))))
    }

    pub(crate) fn actor_key(&self, a: ActorId) -> Option<&Key> {
        match a {
            ActorId::Desk(d) => self.desks.get(d.idx()),
            ActorId::Pop(p) => self.pops.get(p.idx()),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// An id kind with keys: goods, nodes, channels, classes, params, actors and events.
pub trait Keyed: Copy + sealed::Sealed {
    #[doc(hidden)]
    fn lookup(index: &KeyIndex, key: &str) -> Option<Self>;
    #[doc(hidden)]
    fn name(index: &KeyIndex, id: Self) -> Option<&Key>;
}

macro_rules! keyed {
    ($id:ident, $field:ident) => {
        impl sealed::Sealed for $id {}
        impl Keyed for $id {
            fn lookup(index: &KeyIndex, key: &str) -> Option<Self> {
                find(&index.$field, key).map($id)
            }
            fn name(index: &KeyIndex, id: Self) -> Option<&Key> {
                index.$field.get(id.idx())
            }
        }
    };
}

keyed!(GoodId, goods);
keyed!(NodeId, nodes);
keyed!(ChannelId, channels);
keyed!(ClassId, classes);
keyed!(ParamId, params);
keyed!(EventId, events);

impl sealed::Sealed for ActorId {}
impl Keyed for ActorId {
    fn lookup(index: &KeyIndex, key: &str) -> Option<Self> {
        index.actor(key)
    }
    fn name(index: &KeyIndex, id: Self) -> Option<&Key> {
        index.actor_key(id)
    }
}

/// The resolved world of one tape.
#[derive(Debug, Clone)]
pub struct World<E: Ext> {
    /// The tape's name. Not part of the world's identity.
    pub name: String,
    /// FNV-1a 64 over the run content and the genesis state (docs/ENGINE.md §2.6).
    pub world_id: u64,
    /// The calendar.
    pub clock: Clock,
    /// The registered params.
    pub registry: Registry,
    /// The ledger's tolerances.
    pub tol: Tolerances,
    /// The market settings.
    pub market: MarketConfig,
    /// The goods, by id.
    pub goods: Vec<GoodDef>,
    /// The nodes, by id.
    pub nodes: Vec<NodeDef>,
    /// The channels, by id.
    pub channels: Vec<ChannelDef>,
    /// The classes, by id.
    pub classes: Vec<Key>,
    /// The actors in `ActorId` order: every desk, then every pop.
    pub actors: Vec<ActorDecl<E::Actor>>,
    /// The resolved events.
    pub schedule: Schedule<E>,
    pub(crate) keys: KeyIndex,
    pub(crate) currency: Vec<bool>,
}

impl<E: Ext> World<E> {
    /// The id of the entity of kind `I` with this key.
    pub fn id_of<I: Keyed>(&self, key: &str) -> Option<I> {
        I::lookup(&self.keys, key)
    }

    /// The key of this id.
    pub fn key_of<I: Keyed>(&self, id: I) -> Option<&Key> {
        I::name(&self.keys, id)
    }

    /// The number of goods.
    pub fn n_goods(&self) -> usize {
        self.goods.len()
    }

    /// The number of nodes.
    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// A good's definition.
    pub fn good(&self, g: GoodId) -> Option<&GoodDef> {
        self.goods.get(g.idx())
    }

    /// A node's definition.
    pub fn node(&self, n: NodeId) -> Option<&NodeDef> {
        self.nodes.get(n.idx())
    }

    /// An actor's declaration.
    pub fn actor(&self, a: ActorId) -> Option<&ActorDecl<E::Actor>> {
        let i = match a {
            ActorId::Desk(d) if d.idx() < self.keys.desks.len() => d.idx(),
            ActorId::Pop(p) if p.idx() < self.keys.pops.len() => self.keys.desks.len() + p.idx(),
            _ => return None,
        };
        self.actors.get(i)
    }

    /// Whether some node quotes its prices in `g`. A currency has no market anywhere.
    pub fn is_currency(&self, g: GoodId) -> bool {
        self.currency.get(g.idx()).copied().unwrap_or(false)
    }

    /// Every market, (node, non-currency good), in (node, good) order.
    pub fn markets(&self) -> impl Iterator<Item = (NodeId, GoodId)> + '_ {
        self.nodes.iter().flat_map(move |n| {
            self.goods
                .iter()
                .filter(move |g| !self.is_currency(g.id))
                .map(move |g| (n.id, g.id))
        })
    }

    /// Whether `h` can hold goods in this world: a declared actor, or the escrow of a market.
    pub fn is_holder(&self, h: Holder) -> bool {
        match h {
            Holder::Actor(a) => self.actor(a).is_some(),
            Holder::Escrow(n, g) => {
                n.idx() < self.nodes.len() && g.idx() < self.goods.len() && !self.is_currency(g)
            }
        }
    }

    /// FNV-1a 64 over bincode 1 of every firing with `tick < t`, in firing order, each as (tick,
    /// event key, occurrence, resolved action). A checkpoint of the state whose tick is `t`
    /// stores it, and a resume refuses a tape whose past differs.
    pub fn prefix_id(&self, tick: u64) -> u64 {
        let mut h = Fnv::new();
        for f in self.schedule.firings_before(tick) {
            let key = self.key_of(f.event).map_or("", Key::as_str);
            h.write_serialized(&(f.tick, key, f.occurrence, &f.action));
        }
        h.finish()
    }
}
