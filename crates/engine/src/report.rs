//! The observation (docs/ENGINE.md §7.4, E4): what one tick reports, and the owned copies a
//! frontend asks for.
//!
//! A report costs O(markets + orders + events) and never copies holdings; `Sim::observe_holdings`
//! does that when a frontend asks. Ids in a report are dense and are read against
//! `Sim::world()`. Every type here is owned, `Clone`, `Send` and serialisable, so a frontend can
//! ship it across a channel or a wire.

use rustyecon_agents::Agents;
use rustyecon_core::{Date, GoodId, Holder, Key, NodeId, Phase, RunAudit, StateDelta, TickAudit};
use rustyecon_markets::{RationLine, SettleLine};
use serde::{Deserialize, Serialize};

/// What one tick did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickReport {
    /// The tick that ran: the report of tick `t` is made on the state whose tick was `t`.
    pub tick: u64,
    /// The date of tick `t`, `clock.date_of(t)`.
    pub date: Date,
    /// The hash of the state after the tick, whose tick is `t + 1`.
    pub hash: u64,
    /// Every market, (node, non-currency good), in (node, good) order.
    pub markets: Vec<MarketLine>,
    /// One per admitted order, with the quantities `apply` moved (§3.2), in (actor, node, good,
    /// side) order.
    pub settlements: Vec<SettleLine>,
    /// One per (node, good, class, side) that had an order (R12).
    pub rationing: Vec<RationLine>,
    /// The tick's ledger: its (good, provenance) lines, `Rounding` included, and margins (R2).
    pub audit: TickAudit,
    /// The run's ledger so far, since the `Sim`'s starting tick: its lines summed, each good's
    /// drift over the run, and the run's margin (R2).
    pub run: RunAudit,
    /// The tape events that fired, in firing order.
    pub events: Vec<FiredEvent>,
}

/// One market's tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarketLine {
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// The posted price this tick's settlement used.
    pub price: f64,
    /// The price posted for the next tick, after the update.
    pub next_price: f64,
    /// The EMA after the update.
    pub ema: f64,
    /// `S`, the sells.
    pub supply: f64,
    /// `D`, the feasible buys (N7).
    pub demand: f64,
    /// The good that left the escrow for the buyers.
    pub cleared: f64,
    /// The share of each feasible buy that was filled.
    pub buyer_fill: f64,
    /// The share of each sell that was filled.
    pub seller_fill: f64,
}

/// A tape event that fired.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiredEvent {
    /// The event's key.
    pub key: Key,
    /// 0 for a dated event; k for the k-th firing of a recurring one.
    pub occurrence: u32,
    /// The resolved action.
    pub action: StateDelta<Agents>,
    /// For a `SetParam`, the key of the param its value was copied from, which carries the new
    /// value's basis (`Firing::source`); `None` for any other action (amended at S2.2, D10
    /// item 2).
    pub source: Option<Key>,
}

/// Every holder's total of every good it holds, in (holder, good) order: an owned copy for
/// another thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoldingTotals(pub Vec<(Holder, GoodId, f64)>);

/// One applied delta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEntry {
    /// The phase it was applied in.
    pub phase: Phase,
    /// The delta.
    pub delta: StateDelta<Agents>,
    /// What `apply` returned for it: the quantity it moved.
    pub moved: f64,
}

/// Every delta one tick applied, in order. Replaying it through `core::apply` from the state the
/// tick began on reproduces the state it ended on (§7.6).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Trace(pub Vec<TraceEntry>);
