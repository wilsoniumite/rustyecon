//! Clearing (docs/ENGINE.md §3.1), salvaged from `v2p3: systems/clearing/mod.rs:15-56`.
//!
//! Per market, in (node, good) order, `S` is the sum of the sells and `D` the sum of the
//! feasible buys, both left folds in canonical line order. July summed the posted quantities,
//! so `D` counted demand that could not pay (N7); here it is admitted demand only. The fills are
//! July's: `buyer_fill = min(S/D, 1)` and `seller_fill = min(D/S, 1)`, 0 where the side is
//! empty. Every market gets its volumes recorded, 0 and 0 where nothing was posted.

use crate::order::{Line, OrderError, Side};
use rustyecon_core::{Ext, GoodId, NodeId, StateDelta, World};
use serde::{Deserialize, Serialize};

/// One market's clearing: its volumes and both fills.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarketFill {
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// `S`: the sum of the sells.
    pub supply: f64,
    /// `D`: the sum of the feasible buys (N7).
    pub demand: f64,
    /// The share of each feasible buy that is filled: `min(S/D, 1)`, 0 when `D = 0`.
    pub buyer_fill: f64,
    /// The share of each sell that is filled: `min(D/S, 1)`, 0 when `S = 0`.
    pub seller_fill: f64,
}

impl MarketFill {
    /// Whether the market trades: both sides are posted and both fills are positive. A fill is
    /// 0 with both sides posted only when `S/D` or `D/S` underflows (below 2⁻¹⁰⁷⁴), and then
    /// nothing trades rather than one side alone.
    pub fn trades(&self) -> bool {
        self.supply > 0.0 && self.demand > 0.0 && self.buyer_fill > 0.0 && self.seller_fill > 0.0
    }
}

/// Every market's clearing, in (node, good) order. Made by [`clear`].
#[derive(Debug, Clone, PartialEq)]
pub struct Fills {
    markets: Vec<MarketFill>,
}

impl Fills {
    /// Every market of the world, in (node, good) order.
    pub fn markets(&self) -> &[MarketFill] {
        &self.markets
    }

    /// One market's clearing.
    pub fn get(&self, node: NodeId, good: GoodId) -> Option<&MarketFill> {
        self.markets
            .binary_search_by_key(&(node, good), |m| (m.node, m.good))
            .ok()
            .map(|i| &self.markets[i])
    }
}

/// Clear admitted lines: `SetVolumes` for every market of the world, in (node, good) order, and
/// the fills. A line for a market the world does not have is [`OrderError::UnknownMarket`].
pub fn clear<E: Ext>(
    lines: &[Line],
    w: &World<E>,
) -> Result<(Vec<StateDelta<E>>, Fills), OrderError> {
    let n_goods = w.n_goods();
    let mut supply = vec![0.0; w.n_nodes() * n_goods];
    let mut demand = supply.clone();
    for line in canonical(lines) {
        let o = &line.order;
        let i = slot(w, o.node, o.good)?;
        match o.side {
            Side::Sell => supply[i] += o.qty,
            Side::Buy { .. } => demand[i] += line.feasible,
        }
    }
    let mut deltas = Vec::new();
    let mut markets = Vec::new();
    for (node, good) in w.markets() {
        let i = slot(w, node, good)?;
        let (s, d) = (supply[i], demand[i]);
        let buyer_fill = if d > 0.0 { (s / d).min(1.0) } else { 0.0 };
        let seller_fill = if s > 0.0 { (d / s).min(1.0) } else { 0.0 };
        deltas.push(StateDelta::SetVolumes {
            node,
            good,
            supply: s,
            demand: d,
        });
        markets.push(MarketFill {
            node,
            good,
            supply: s,
            demand: d,
            buyer_fill,
            seller_fill,
        });
    }
    Ok((deltas, Fills { markets }))
}

/// The lines in canonical order, (actor, node, good, side). [`crate::admit`] returns them so;
/// sorting again keeps every fold canonical whatever order a caller keeps them in.
pub(crate) fn canonical(lines: &[Line]) -> Vec<&Line> {
    let mut sorted: Vec<&Line> = lines.iter().collect();
    sorted.sort_by_key(|l| l.order.key());
    sorted
}

/// The flat book index of a market of `w`.
fn slot<E: Ext>(w: &World<E>, node: NodeId, good: GoodId) -> Result<usize, OrderError> {
    if w.node(node).is_none() || !w.has_market(good) {
        return Err(OrderError::UnknownMarket { node, good });
    }
    Ok(node.idx() * w.n_goods() + good.idx())
}
