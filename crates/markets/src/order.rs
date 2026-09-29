//! Orders and admission (docs/ENGINE.md §3.1): cash binds at the order (N7).
//!
//! Salvaged from `v2p3: types/order.rs`, which had neither a budget nor a class. An order now
//! carries both: a buy's budget is the most it may pay, in the node's currency, and the class is
//! the unit rationing is recorded by (R12). Admission checks every order against the phase-start
//! state and turns it into a [`Line`] with its feasible quantity. Budgets bind here and nowhere
//! else: settlement never cuts a buyer, so the demand the price reads is demand that can pay.
//! July capped cash at settlement (`v2p3: systems/transactions/mod.rs:185-223`), so its volumes
//! counted demand that could not pay and its rationing went unrecorded.

use rustyecon_core::num::{self, is_clean, NumError};
use rustyecon_core::{
    ActorId, Amount, ClassId, CoreError, Ext, GoodId, Holder, Inventory, NodeId, SimState,
    TakeError, World,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Which side of its market an order is on. A buy carries its budget.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    /// A buy. `budget` is the most it may pay, in the node's currency; it binds at admission.
    Buy {
        /// The budget, in the node's currency.
        budget: f64,
    },
    /// A sell, from the actor's own stock.
    Sell,
}

impl Side {
    /// The side without its budget.
    pub fn tag(self) -> SideTag {
        match self {
            Side::Buy { .. } => SideTag::Buy,
            Side::Sell => SideTag::Sell,
        }
    }
}

/// A side, without a buy's budget. Buys order before sells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SideTag {
    /// The buying side.
    Buy,
    /// The selling side.
    Sell,
}

/// One actor's order in one market, (node, good), for this tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Order {
    /// Who posts it.
    pub actor: ActorId,
    /// The actor's registered class, which rationing is recorded by.
    pub class: ClassId,
    /// The node. In Phase 0 an actor may post at any node (docs/ENGINE.md §3.1).
    pub node: NodeId,
    /// The good; never a currency or an untraded good.
    pub good: GoodId,
    /// The quantity asked for or offered this tick.
    pub qty: f64,
    /// The side, with a buy's budget.
    pub side: Side,
}

impl Order {
    /// The canonical key: orders are admitted in (actor, node, good, side) order, and an actor
    /// posts at most one order per key.
    pub fn key(&self) -> (ActorId, NodeId, GoodId, SideTag) {
        (self.actor, self.node, self.good, self.side.tag())
    }
}

/// An admitted order with its feasible quantity: for a buy, `min(qty, max_qty(budget, price))`;
/// for a sell, its quantity. Lines come from [`admit`] only.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Line {
    /// The order.
    pub order: Order,
    /// What it can take or give at the posted price.
    pub feasible: f64,
}

/// An order that admission refuses, or settlement plumbing that does not fit its world. Every
/// one stops the tick; none is clamped or forgiven.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderError {
    /// The actor is not declared in the world.
    UnknownActor(ActorId),
    /// The node is not in the world.
    UnknownNode(NodeId),
    /// The good is not in the world.
    UnknownGood(GoodId),
    /// The good has no market: a currency, or an untraded good (amended at P2.2b.1).
    NoMarket {
        /// The node.
        node: NodeId,
        /// The currency or untraded good.
        good: GoodId,
    },
    /// The order's class is not the actor's registered class.
    WrongClass {
        /// The actor.
        actor: ActorId,
        /// The class on the order.
        class: ClassId,
        /// The actor's registered class.
        registered: ClassId,
    },
    /// A quantity or budget that is not finite or has its sign bit set (`-0.0` included).
    BadValue {
        /// The order.
        order: Order,
        /// Which value.
        what: &'static str,
        /// The value.
        value: f64,
    },
    /// Two orders with one key (actor, node, good, side).
    Duplicate {
        /// The actor.
        actor: ActorId,
        /// The node.
        node: NodeId,
        /// The good.
        good: GoodId,
        /// The side.
        side: SideTag,
    },
    /// A budget larger than what remains of the actor's currency after the budgets admitted
    /// before it.
    OverBudget {
        /// The order.
        order: Order,
        /// The node's currency.
        currency: GoodId,
        /// What remained of it.
        remaining: f64,
    },
    /// A sell larger than what remains of the good after the actor's sells admitted before it,
    /// at any node.
    OverPosted {
        /// The order.
        order: Order,
        /// What remained of the good.
        remaining: f64,
    },
    /// A line or fill for a market this world does not have: lines and fills are cleared and
    /// settled against the world they were admitted in.
    UnknownMarket {
        /// The node.
        node: NodeId,
        /// The good.
        good: GoodId,
    },
    /// [`crate::SettlePlan::realize`] was given a moved list whose length is not the plan's.
    Moved {
        /// The plan's deltas.
        deltas: usize,
        /// The quantities given.
        moved: usize,
    },
    /// A state that does not fit its world: a declared actor without a holding, or a market
    /// without a book slot.
    Core(CoreError),
    /// A budget helper refused its inputs.
    Num(NumError),
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::UnknownActor(a) => write!(f, "order from undefined actor {a}"),
            OrderError::UnknownNode(n) => write!(f, "order at undefined {n}"),
            OrderError::UnknownGood(g) => write!(f, "order for undefined {g}"),
            OrderError::NoMarket { node, good } => {
                write!(
                    f,
                    "order for {good} at {node}: a currency or an untraded good has no market"
                )
            }
            OrderError::WrongClass {
                actor,
                class,
                registered,
            } => write!(
                f,
                "order from {actor} names {class}, but its registered class is {registered}"
            ),
            OrderError::BadValue { order, what, value } => write!(
                f,
                "order from {} for {} at {}: {what} is {value:e}; values are finite with a clear \
                 sign bit",
                order.actor, order.good, order.node
            ),
            OrderError::Duplicate {
                actor,
                node,
                good,
                side,
            } => write!(
                f,
                "{actor} posted two {side:?} orders for {good} at {node}; one per market and side"
            ),
            OrderError::OverBudget {
                order,
                currency,
                remaining,
            } => write!(
                f,
                "over budget: {} bids {:e} of {currency} for {} at {}, with {remaining:e} left \
                 after its earlier budgets",
                order.actor,
                match order.side {
                    Side::Buy { budget } => budget,
                    Side::Sell => 0.0,
                },
                order.good,
                order.node
            ),
            OrderError::OverPosted { order, remaining } => write!(
                f,
                "over posted: {} offers {:e} of {} at {}, with {remaining:e} left after its \
                 earlier sells",
                order.actor, order.qty, order.good, order.node
            ),
            OrderError::UnknownMarket { node, good } => {
                write!(f, "no market ({node}, {good}) in this world")
            }
            OrderError::Moved { deltas, moved } => write!(
                f,
                "a settlement plan of {deltas} deltas was realized with {moved} moved quantities"
            ),
            OrderError::Core(e) => write!(f, "{e}"),
            OrderError::Num(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for OrderError {}

/// Admit this tick's orders against the phase-start state.
///
/// Orders are sorted by (actor, node, good, side) first, so their input order is irrelevant.
/// Then every key must be unique, every order must name a declared actor with its registered
/// class, a node, and a non-currency good, and every quantity and budget must be finite with a
/// clear sign bit. Zero-quantity orders are dropped. Last, per actor and in that order, each
/// budget is taken from a copy of the actor's holding of the node's currency (`OverBudget` if it
/// does not fit), and each sell from its holding of the good, cumulatively across nodes
/// (`OverPosted`). These are the takes settlement makes, in the order it makes them, so a
/// payment or shipment that fits here fits there. For a currency, which is one lot, the take is
/// the subtraction `rem − budget` (docs/ENGINE.md §3.1).
pub fn admit<E: Ext>(
    mut orders: Vec<Order>,
    s: &SimState<E>,
    w: &World<E>,
) -> Result<Vec<Line>, OrderError> {
    orders.sort_by_key(Order::key);
    if let Some(pair) = orders.windows(2).find(|p| p[0].key() == p[1].key()) {
        let (actor, node, good, side) = pair[0].key();
        return Err(OrderError::Duplicate {
            actor,
            node,
            good,
            side,
        });
    }
    for o in &orders {
        check(o, w)?;
    }
    let mut lines = Vec::with_capacity(orders.len());
    let mut rest: &[Order] = &orders;
    while let Some(first) = rest.first() {
        let n = rest.partition_point(|o| o.actor <= first.actor);
        let (mine, tail) = rest.split_at(n);
        admit_actor(mine, s, w, &mut lines)?;
        rest = tail;
    }
    Ok(lines)
}

/// The checks that need no holding: ids, class, market and values.
fn check<E: Ext>(o: &Order, w: &World<E>) -> Result<(), OrderError> {
    let decl = w.actor(o.actor).ok_or(OrderError::UnknownActor(o.actor))?;
    if o.class != decl.class {
        return Err(OrderError::WrongClass {
            actor: o.actor,
            class: o.class,
            registered: decl.class,
        });
    }
    w.node(o.node).ok_or(OrderError::UnknownNode(o.node))?;
    w.good(o.good).ok_or(OrderError::UnknownGood(o.good))?;
    if !w.has_market(o.good) {
        return Err(OrderError::NoMarket {
            node: o.node,
            good: o.good,
        });
    }
    let bad = |what: &'static str, value: f64| OrderError::BadValue {
        order: *o,
        what,
        value,
    };
    if !is_clean(o.qty) {
        return Err(bad("qty", o.qty));
    }
    if let Side::Buy { budget } = o.side {
        if !is_clean(budget) {
            return Err(bad("budget", budget));
        }
    }
    Ok(())
}

/// Admit one actor's orders, already sorted and checked, walking a copy of its holding.
fn admit_actor<E: Ext>(
    orders: &[Order],
    s: &SimState<E>,
    w: &World<E>,
    lines: &mut Vec<Line>,
) -> Result<(), OrderError> {
    let Some(first) = orders.first() else {
        return Ok(());
    };
    let h = Holder::Actor(first.actor);
    let mut dry = s
        .holding(h)
        .cloned()
        .ok_or(OrderError::Core(CoreError::UnknownHolder(h)))?;
    for o in orders {
        if o.qty == 0.0 {
            continue;
        }
        let feasible = match o.side {
            Side::Buy { budget } => {
                let currency = w
                    .node(o.node)
                    .ok_or(OrderError::UnknownNode(o.node))?
                    .currency;
                take(&mut dry, currency, budget).map_err(|remaining| OrderError::OverBudget {
                    order: *o,
                    currency,
                    remaining,
                })?;
                let price = s.price(o.node, o.good).ok_or_else(|| {
                    OrderError::Core(CoreError::Shape(format!(
                        "no book slot for ({}, {})",
                        o.node, o.good
                    )))
                })?;
                // No price guard (N6): when budget/price is infinite, qty binds.
                o.qty
                    .min(num::max_qty(budget, price).map_err(OrderError::Num)?)
            }
            Side::Sell => {
                take(&mut dry, o.good, o.qty).map_err(|remaining| OrderError::OverPosted {
                    order: *o,
                    remaining,
                })?;
                o.qty
            }
        };
        lines.push(Line {
            order: *o,
            feasible,
        });
    }
    Ok(())
}

/// Take `q` of `g` from the dry-run holding; on a shortfall, return what was held.
fn take(dry: &mut Inventory, g: GoodId, q: f64) -> Result<(), f64> {
    match dry.take(g, Amount::Qty(q)) {
        Ok(_) => Ok(()),
        Err(TakeError::Shortfall(sf)) => Err(sf.held),
        // Unreachable: `check` refused every unclean value first. Refused, not a panic.
        Err(TakeError::Invalid(_)) => Err(dry.get(g)),
    }
}
