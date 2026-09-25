//! The behaviour seam (docs/ENGINE.md §4): what an actor sees, what it may return, and the error
//! a hook reports.
//!
//! R13 holds by construction. A [`View`] carries the actor's own holding and own state, the
//! posted prices and EMAs, the current params and the clock, and nothing else: no other actor's
//! holdings, orders or state, no cleared volumes, and never the oracle. The engine checks what a
//! hook returns against its whitelist (E7) before anything is applied.

use crate::ext::Agents;
use rustyecon_core::num::NumError;
use rustyecon_core::{
    ActorId, ClassId, Clock, CoreError, GoodId, Inventory, MarketBook, NodeId, Params, StateDelta,
    TakeError,
};
use rustyecon_markets::Order;
use std::fmt;

/// The posted side of the market book: price and EMA per (node, good), read-only. Cleared
/// volumes are aggregates of other actors' orders and are not in it (R13).
#[derive(Debug, Clone, Copy)]
pub struct Posted<'a> {
    book: &'a MarketBook,
}

impl<'a> Posted<'a> {
    /// The posted side of `book`.
    pub fn new(book: &'a MarketBook) -> Posted<'a> {
        Posted { book }
    }
    /// The posted price of (node, good); `None` outside the book.
    pub fn price(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.book.quote(n, g).map(|q| q.price)
    }
    /// The price EMA of (node, good); `None` outside the book.
    pub fn ema(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.book.quote(n, g).map(|q| q.ema)
    }
}

/// What one actor sees when a hook runs: the state as it stood when the phase began (E6).
#[derive(Debug, Clone, Copy)]
pub struct View<'a, S> {
    /// The tick being run.
    pub tick: u64,
    /// The run's clock, for converting registered rates and flows to per-tick values (A13).
    pub clock: &'a Clock,
    /// The actor.
    pub me: ActorId,
    /// Its registered class.
    pub class: ClassId,
    /// Its home node.
    pub home: NodeId,
    /// The home node's currency, in which it budgets and pays.
    pub currency: GoodId,
    /// Its own holding.
    pub own: &'a Inventory,
    /// Its own state.
    pub own_state: &'a S,
    /// Posted prices and EMAs.
    pub posted: Posted<'a>,
    /// The current params, typed by unit.
    pub params: Params<'a>,
}

/// Which hook ran.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Hook {
    /// Phase 1: orders, payouts and Instant endowments.
    Decide,
    /// Phase 4: recipes.
    Produce,
    /// Phase 5b: upkeep.
    Upkeep,
}

impl fmt::Display for Hook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Hook::Decide => "decide",
            Hook::Produce => "produce",
            Hook::Upkeep => "upkeep",
        })
    }
}

/// What `decide` returns: this tick's orders and the deltas it applies in phase 1.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Decision {
    /// Orders, each with `actor == me` and the actor's class.
    pub orders: Vec<Order>,
    /// Deltas: transfers from the actor, Instant endowments to it, its own extension deltas.
    pub deltas: Vec<StateDelta<Agents>>,
}

/// A hook that cannot run. It stops the tick.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentError {
    /// A param or holding read that does not fit the world.
    Core(CoreError),
    /// A budget or scale helper refused its inputs.
    Num(NumError),
    /// A dry-run take refused its request.
    Take(TakeError),
    /// The actor is not declared, or its state is not of its spec's kind.
    Mismatch(ActorId),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::Core(e) => write!(f, "{e}"),
            AgentError::Num(e) => write!(f, "{e}"),
            AgentError::Take(e) => write!(f, "{e}"),
            AgentError::Mismatch(a) => {
                write!(
                    f,
                    "{a} is not declared, or its state does not match its spec"
                )
            }
        }
    }
}

impl std::error::Error for AgentError {}

impl From<CoreError> for AgentError {
    fn from(e: CoreError) -> AgentError {
        AgentError::Core(e)
    }
}

impl From<NumError> for AgentError {
    fn from(e: NumError) -> AgentError {
        AgentError::Num(e)
    }
}

impl From<TakeError> for AgentError {
    fn from(e: TakeError) -> AgentError {
        AgentError::Take(e)
    }
}

/// A behaviour kind. Phase 2 implements it for new kinds; each hook reads only its [`View`].
///
/// Hooks return `Result` (docs/ENGINE.md §4 amended at P0.5): a param read or a helper can
/// refuse, and nothing in the engine panics.
pub trait Behaviour: Send + Sync {
    /// The actor's own extension state.
    type Own;
    /// Phase 1: this tick's orders and deltas.
    fn decide(&self, v: &View<'_, Self::Own>) -> Result<Decision, AgentError>;
    /// Phase 4: recipe burns and mints.
    fn produce(&self, v: &View<'_, Self::Own>) -> Result<Vec<StateDelta<Agents>>, AgentError>;
    /// Phase 5b: upkeep burns.
    fn upkeep(&self, v: &View<'_, Self::Own>) -> Result<Vec<StateDelta<Agents>>, AgentError>;
}
