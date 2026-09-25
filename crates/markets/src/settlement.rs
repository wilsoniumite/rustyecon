//! Settlement (docs/ENGINE.md §3.2): one filled quantity per order, both sides through an
//! escrow, and rationing recorded per class (R12).
//!
//! About half of `v2p3: systems/transactions/mod.rs` is rewritten. July settled buyers first,
//! capped them by cash there (`:185-223`), and paid sellers pro rata to what buyers took; it
//! kept per-kind currency maps keyed by bare `(u32, u32)`, so a desk and a pop sharing a number
//! were paid into one entry (defect 10, `:200-219, 390-395`); it forgave negative balances
//! (`:396-406`, N8); and it dropped flows under absolute epsilons (`:125, 190, 357-433`, N6).
//! None of that moves. Here every holder is a typed [`Holder`], budgets were bound at admission,
//! and each market settles in four steps through its escrow, `Holder::Escrow(node, good)`:
//!
//! 1. each seller, in actor order, ships `ship_i = qty_i·seller_fill` of the good in;
//! 2. each buyer, in actor order, pays `p·r_j` of the currency in, `r_j = feasible_j·buyer_fill`;
//! 3. the escrow delivers `r_j` to each buyer, the largest last with `All`;
//! 4. the escrow pays `p·ship_i` to each seller, the largest shipper last with `All`.
//!
//! Every fill is at most 1 and rounding is monotone, so `ship_i <= qty_i` and `p·r_j <=
//! budget_j`, which admission took from the same holdings in the same order; receipts only add.
//! The largest taker goes last, so no earlier take exceeds the escrow, and every escrow ends
//! empty. Payments are nominal. A quantity can differ from nominal by rounding in two places:
//! the last taker's `All`, and any take from a holding of several lots, which moves the sum of
//! the lots it takes (a seller's bread, or a bread escrow). So `shipped <= offered` and
//! `filled <= feasible` hold only up to that rounding, and the lines are built from what
//! `apply` actually moved.

use crate::clearing::{canonical, Fills, MarketFill};
use crate::order::{Line, OrderError, Side, SideTag};
use rustyecon_core::{
    ActorId, Amount, ClassId, Ext, GoodId, Holder, NodeId, SimState, StateDelta, World,
};
use serde::{Deserialize, Serialize};

/// What one admitted order did, from the quantities `apply` moved. One per admitted order.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SettleLine {
    /// The actor.
    pub actor: ActorId,
    /// Its class.
    pub class: ClassId,
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// The side.
    pub side: SideTag,
    /// A buyer's good received, or a seller's good shipped.
    pub qty: f64,
    /// A buyer's currency paid, or a seller's currency received.
    pub value: f64,
}

/// Rationing in one market for one class and side (R12). One per (node, good, class, side) that
/// had an order.
///
/// For a buyer class the cash shortfall is `requested − feasible` and the market shortfall
/// `feasible − filled`. For a seller class `requested = feasible` = offered, and `filled` is what
/// it shipped. `filled` is what `apply` moved, so when a take spans several lots or is a last
/// taker's `All` it can exceed `feasible` (or what was offered) by rounding, a few ulps: the
/// market shortfall is then a few ulps below zero, not an invariant to test exactly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RationLine {
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// The class.
    pub class: ClassId,
    /// The side.
    pub side: SideTag,
    /// What the class's orders asked for or offered.
    pub requested: f64,
    /// What their budgets could pay for (buyers), or what they offered (sellers).
    pub feasible: f64,
    /// What they received (buyers) or shipped (sellers).
    pub filled: f64,
}

/// What a planned delta does, for [`SettlePlan::realize`]. Each holds its line's index.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Role {
    /// Step 1: a seller ships into the escrow.
    Ship(usize),
    /// Step 2: a buyer pays into the escrow.
    Pay(usize),
    /// Step 3: the escrow delivers to a buyer.
    Receive(usize),
    /// Step 4: the escrow pays a seller.
    Paid(usize),
}

/// The settlement of one tick: its deltas, to be applied in phase 3, and what each one is for.
#[derive(Debug, Clone, PartialEq)]
pub struct SettlePlan<E: Ext> {
    lines: Vec<Line>,
    deltas: Vec<StateDelta<E>>,
    roles: Vec<Role>,
}

/// A taker from an escrow: its line, its actor, the quantity that ranks it and the amount it is
/// due.
#[derive(Debug, Clone, Copy)]
struct Taker {
    line: usize,
    actor: ActorId,
    rank: f64,
    due: f64,
}

/// Plan the settlement of admitted and cleared lines, reading each market's posted price from
/// the phase-start state. Markets settle in (node, good) order; a market trades when
/// [`MarketFill::trades`]. Lines and fills must come from this world, or the result is
/// [`OrderError::UnknownMarket`].
pub fn settle<E: Ext>(
    lines: &[Line],
    f: &Fills,
    s: &SimState<E>,
    w: &World<E>,
) -> Result<SettlePlan<E>, OrderError> {
    let lines: Vec<Line> = canonical(lines).into_iter().copied().collect();
    // Line indices by market, and by actor then side within one.
    let mut by_market: Vec<usize> = (0..lines.len()).collect();
    by_market.sort_by_key(|&i| {
        let o = &lines[i].order;
        (o.node, o.good, o.actor, o.side.tag())
    });
    let mut groups: Vec<(NodeId, GoodId, Vec<usize>)> = Vec::new();
    for i in by_market {
        let o = &lines[i].order;
        match groups.last_mut() {
            Some((n, g, idx)) if (*n, *g) == (o.node, o.good) => idx.push(i),
            _ => groups.push((o.node, o.good, vec![i])),
        }
    }
    let mut plan = SettlePlan {
        lines,
        deltas: Vec::new(),
        roles: Vec::new(),
    };
    for (node, good, idx) in groups {
        let unknown = || OrderError::UnknownMarket { node, good };
        let fill = *f.get(node, good).ok_or_else(unknown)?;
        let currency = w.node(node).ok_or_else(unknown)?.currency;
        let price = s.price(node, good).ok_or_else(unknown)?;
        if fill.trades() {
            plan.market(&idx, &fill, price, currency);
        }
    }
    Ok(plan)
}

impl<E: Ext> SettlePlan<E> {
    /// The deltas, in order: markets by (node, good), each in steps 1 to 4.
    pub fn deltas(&self) -> &[StateDelta<E>] {
        &self.deltas
    }

    /// Turn the plan into one [`SettleLine`] per admitted order, in (actor, node, good, side)
    /// order, and the [`RationLine`]s, in (node, good, class, side) order, from the quantity
    /// `apply` returned for each delta.
    pub fn realize(self, moved: &[f64]) -> Result<(Vec<SettleLine>, Vec<RationLine>), OrderError> {
        if moved.len() != self.deltas.len() {
            return Err(OrderError::Moved {
                deltas: self.deltas.len(),
                moved: moved.len(),
            });
        }
        let mut settled: Vec<SettleLine> = self
            .lines
            .iter()
            .map(|l| SettleLine {
                actor: l.order.actor,
                class: l.order.class,
                node: l.order.node,
                good: l.order.good,
                side: l.order.side.tag(),
                qty: 0.0,
                value: 0.0,
            })
            .collect();
        for (role, &m) in self.roles.iter().zip(moved) {
            match *role {
                Role::Ship(i) | Role::Receive(i) => settled[i].qty = m,
                Role::Pay(i) | Role::Paid(i) => settled[i].value = m,
            }
        }
        let rationing = ration(&self.lines, &settled);
        Ok((settled, rationing))
    }

    fn push(&mut self, d: StateDelta<E>, role: Role) {
        self.deltas.push(d);
        self.roles.push(role);
    }

    /// Steps 1 to 4 for one trading market; `idx` holds its lines in actor order.
    fn market(&mut self, idx: &[usize], fill: &MarketFill, price: f64, currency: GoodId) {
        let (node, good) = (fill.node, fill.good);
        let escrow = Holder::Escrow(node, good);
        let mut sellers = Vec::new();
        let mut buyers = Vec::new();
        for &i in idx {
            let l = &self.lines[i];
            let actor = l.order.actor;
            match l.order.side {
                Side::Sell => {
                    let ship = l.order.qty * fill.seller_fill;
                    sellers.push(Taker {
                        line: i,
                        actor,
                        rank: ship,
                        due: price * ship,
                    });
                }
                Side::Buy { .. } => {
                    let r = l.feasible * fill.buyer_fill;
                    buyers.push(Taker {
                        line: i,
                        actor,
                        rank: r,
                        due: r,
                    });
                }
            }
        }
        // 1. Sellers ship their filled quantity in.
        for t in &sellers {
            if t.rank > 0.0 {
                self.push(
                    transfer(Holder::Actor(t.actor), escrow, good, Amount::Qty(t.rank)),
                    Role::Ship(t.line),
                );
            }
        }
        // 2. Buyers pay for theirs, in the order admission took their budgets.
        for t in &buyers {
            let pay = price * t.rank;
            if pay > 0.0 {
                self.push(
                    transfer(Holder::Actor(t.actor), escrow, currency, Amount::Qty(pay)),
                    Role::Pay(t.line),
                );
            }
        }
        // 3. The escrow delivers the good; 4. it pays the sellers.
        self.empty(escrow, good, &buyers, Role::Receive);
        self.empty(escrow, currency, &sellers, Role::Paid);
    }

    /// Pay out one good of the escrow: every taker but the last its due, in actor order, then
    /// the last `All`. The last is the largest by `rank` (`total_cmp`), ties to the highest id,
    /// so no earlier take can exceed what the escrow holds.
    fn empty(&mut self, escrow: Holder, good: GoodId, takers: &[Taker], role: fn(usize) -> Role) {
        let Some(last) = takers
            .iter()
            .max_by(|a, b| a.rank.total_cmp(&b.rank).then(a.actor.cmp(&b.actor)))
            .copied()
        else {
            return;
        };
        for t in takers {
            if t.line != last.line && t.due > 0.0 {
                self.push(
                    transfer(escrow, Holder::Actor(t.actor), good, Amount::Qty(t.due)),
                    role(t.line),
                );
            }
        }
        self.push(
            transfer(escrow, Holder::Actor(last.actor), good, Amount::All),
            role(last.line),
        );
    }
}

fn transfer<E: Ext>(from: Holder, to: Holder, good: GoodId, amount: Amount) -> StateDelta<E> {
    StateDelta::Transfer {
        from,
        to,
        good,
        amount,
    }
}

/// One line per (node, good, class, side) with an order, each a left fold in actor order.
fn ration(lines: &[Line], settled: &[SettleLine]) -> Vec<RationLine> {
    let mut idx: Vec<usize> = (0..lines.len()).collect();
    idx.sort_by_key(|&i| {
        let o = &lines[i].order;
        (o.node, o.good, o.class, o.side.tag(), o.actor)
    });
    let mut out: Vec<RationLine> = Vec::new();
    for i in idx {
        let (o, feasible, filled) = (&lines[i].order, lines[i].feasible, settled[i].qty);
        let key = (o.node, o.good, o.class, o.side.tag());
        match out.last_mut() {
            Some(r) if (r.node, r.good, r.class, r.side) == key => {
                r.requested += o.qty;
                r.feasible += feasible;
                r.filled += filled;
            }
            _ => out.push(RationLine {
                node: o.node,
                good: o.good,
                class: o.class,
                side: o.side.tag(),
                requested: o.qty,
                feasible,
                filled,
            }),
        }
    }
    out
}
