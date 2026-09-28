//! The Appendix B roles' rules (docs/probe/RULES.md), each decision one of the pinning paper's
//! margins, read from posted prices at the actor's home node and its own holding, state and the
//! current params (R13). Nothing here reads a volume, a fill, another actor or the oracle.
//!
//! Like the scripted actor, no rule produces an order admission refuses or a burn that falls
//! short, whatever the prices:
//!
//! - **Budgets.** A transfer or payout comes first, capped by the coin, and is taken from a copy
//!   of the holding. Every buy's budget is then capped by what the copy still holds and taken
//!   from it, and the second budget of a pair is at most `max_remainder(outlay, first)`, so the
//!   pair never sums past the outlay (the reservation design's hygiene, a judges' graft).
//! - **Sells** offer at most what the copy still holds of the good, taken as admission takes it.
//! - **Recipes** run at `min_k max_scale(held_k, a_k)` over the inputs whose coefficient is not
//!   zero: an input with a zero coefficient does not bind, so at x = 1 or x = 0 exactly the desk
//!   still makes its output from the other input.

use crate::behaviour::{AgentError, Behaviour, Decision, View};
use crate::ext::{
    ActorState, AgentDelta, Agents, GoodDeskState, MachDeskState, ProviderState, WorkersState,
};
use crate::roles::spec::{Assign, Basket, GoodDesk, MachDesk, Provider, Scale, Schedule, Workers};
use rustyecon_core::num;
use rustyecon_core::{
    ActorId, Amount, CoreError, GoodId, Holder, Inventory, Provenance, Site, StateDelta,
};
use rustyecon_markets::{Order, Side};

pub(crate) type Delta = StateDelta<Agents>;

/// The posted price of `g` at the actor's home node.
pub(crate) fn price<S>(v: &View<'_, S>, g: GoodId) -> Result<f64, AgentError> {
    v.posted.price(v.home, g).ok_or_else(|| {
        AgentError::Core(CoreError::Shape(format!(
            "no posted price for {g} at {}",
            v.home
        )))
    })
}

/// A param's per-tick value at one of its uses, converted by the method its site was resolved
/// with (amended at S2.2): a `Dimensionless` value as it is, a flow by `Clock::flow`, a rate as
/// a `Clock::share` of a stock, or, for the step rule's `up` and `down`, as a `Clock::log_step`
/// (docs/probe/RULES.md §2). The site names the method, so the rule cannot convert another way.
pub(crate) fn param<S>(v: &View<'_, S>, site: Site) -> Result<f64, AgentError> {
    Ok(site.per_tick(&v.params, v.clock)?)
}

pub(crate) fn buy<S>(v: &View<'_, S>, good: GoodId, qty: f64, budget: f64) -> Order {
    Order {
        actor: v.me,
        class: v.class,
        node: v.home,
        good,
        qty,
        side: Side::Buy { budget },
    }
}

pub(crate) fn sell<S>(v: &View<'_, S>, good: GoodId, qty: f64) -> Order {
    Order {
        actor: v.me,
        class: v.class,
        node: v.home,
        good,
        qty,
        side: Side::Sell,
    }
}

pub(crate) fn set<S>(v: &View<'_, S>, state: ActorState) -> Delta {
    StateDelta::Actor(AgentDelta::SetState { actor: v.me, state })
}

/// Take `q` of `g` from the copy of the holding, when there is anything to take.
pub(crate) fn take(dry: &mut Inventory, g: GoodId, q: f64) -> Result<(), AgentError> {
    if q > 0.0 {
        dry.take(g, Amount::Qty(q))?;
    }
    Ok(())
}

/// What a copy of the holding still holds of `good`, up to `want`, taken from the copy.
pub(crate) fn offer(dry: &mut Inventory, good: GoodId, want: f64) -> Result<f64, AgentError> {
    let q = want.min(dry.get(good));
    take(dry, good, q)?;
    Ok(q)
}

/// Two budgets out of one outlay: the first at most `first` and the outlay, the second at most
/// `second` and what the outlay has left, `max_remainder(total, b1)`. Each is also capped by
/// the currency the copy still holds, and taken from it, as admission will take it.
fn two_budgets(
    total: f64,
    first: f64,
    second: f64,
    dry: &mut Inventory,
    currency: GoodId,
) -> Result<(f64, f64), AgentError> {
    let b1 = first.min(total).min(dry.get(currency));
    take(dry, currency, b1)?;
    let b2 = second
        .min(num::max_remainder(total, b1)?)
        .min(dry.get(currency));
    take(dry, currency, b2)?;
    Ok((b1, b2))
}

/// The Leontief scale over the inputs, `(held, coefficient)`, whose coefficient is not zero: the
/// most every burn `coefficient·y` fits its holding. 0 when no input binds.
pub(crate) fn leontief(inputs: &[(f64, f64)]) -> Result<f64, AgentError> {
    let mut y: Option<f64> = None;
    for &(held, coef) in inputs {
        if coef > 0.0 {
            let x = num::max_scale(held, coef)?;
            y = Some(y.map_or(x, |z| z.min(x)));
        }
    }
    Ok(y.unwrap_or(0.0))
}

pub(crate) fn burn(me: Holder, good: GoodId, qty: f64, prov: Provenance, out: &mut Vec<Delta>) {
    if qty > 0.0 {
        out.push(StateDelta::Burn {
            from: me,
            good,
            amount: Amount::Qty(qty),
            prov,
        });
    }
}

/// Pay `share(rate)` of the coin the copy holds above `floor` to `to`, as a transfer from the
/// actor, and take it from the copy.
fn pay_above<S>(
    v: &View<'_, S>,
    to: ActorId,
    rate: Site,
    floor: f64,
    dry: &mut Inventory,
    out: &mut Decision,
) -> Result<(), AgentError> {
    let coin = dry.get(v.currency);
    if coin > floor {
        let pay = (param(v, rate)? * (coin - floor)).min(coin);
        if pay > 0.0 {
            take(dry, v.currency, pay)?;
            out.deltas.push(StateDelta::Transfer {
                from: Holder::Actor(v.me),
                to: Holder::Actor(to),
                good: v.currency,
                amount: Amount::Qty(pay),
            });
        }
    }
    Ok(())
}

/// A household's baskets: `budget` at P_s = p + h·r buys n = budget/P_s baskets, posted as a buy
/// of n of the good and h·n of the space from one budget.
fn basket_orders<S>(
    v: &View<'_, S>,
    b: &Basket,
    budget: f64,
    dry: &mut Inventory,
    out: &mut Decision,
) -> Result<(), AgentError> {
    let p = price(v, b.good)?;
    let r = price(v, b.space)?;
    let h = param(v, b.per_basket)?;
    let n = budget / (p + h * r);
    let (bg, bs) = two_budgets(budget, p * n, r * (h * n), dry, v.currency)?;
    out.orders.push(buy(v, b.good, n, bg));
    out.orders.push(buy(v, b.space, h * n, bs));
    Ok(())
}

/// A household eats min(good, space/h) baskets of what it holds, as `Consumption`. What is not
/// eaten dies at 5a: the good bought this tick and the space are one-tick goods.
fn eat<S>(v: &View<'_, S>, b: &Basket) -> Result<Vec<Delta>, AgentError> {
    let h = param(v, b.per_basket)?;
    let n = leontief(&[(v.own.get(b.good), 1.0), (v.own.get(b.space), h)])?;
    let me = Holder::Actor(v.me);
    let mut out = Vec::new();
    burn(me, b.good, n, Provenance::Consumption, &mut out);
    burn(me, b.space, h * n, Provenance::Consumption, &mut out);
    Ok(out)
}

impl Behaviour for Provider {
    type Own = ProviderState;

    fn decide(&self, v: &View<'_, ProviderState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let me = Holder::Actor(v.me);
        let mut dry = v.own.clone();
        // T of land services, endowed and all offered: it buys its own space on the market.
        let t = param(v, self.endowment)?;
        if t > 0.0 {
            out.deltas.push(StateDelta::Mint {
                to: me,
                good: self.land,
                qty: t,
                prov: Provenance::Endowment,
            });
        }
        out.orders.push(sell(v, self.land, t));
        // One basket per head at posted prices, N·P_s, paid from coin held; a shortfall is
        // recorded in its state (R12).
        let p = price(v, self.basket.good)?;
        let r = price(v, self.basket.space)?;
        let h = param(v, self.basket.per_basket)?;
        let due = param(v, self.heads)? * (p + h * r);
        let paid = due.min(dry.get(v.currency));
        if paid > 0.0 {
            take(&mut dry, v.currency, paid)?;
            out.deltas.push(StateDelta::Transfer {
                from: me,
                to: Holder::Actor(self.transfer_to),
                good: v.currency,
                amount: Amount::Qty(paid),
            });
        }
        // Its own baskets, from a share of the coin the transfer left.
        let budget = param(v, self.spend)? * dry.get(v.currency);
        basket_orders(v, &self.basket, budget, &mut dry, &mut out)?;
        out.deltas
            .push(set(v, ActorState::Provider(ProviderState { due, paid })));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, ProviderState>) -> Result<Vec<Delta>, AgentError> {
        eat(v, &self.basket)
    }

    fn upkeep(&self, _: &View<'_, ProviderState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}

impl Behaviour for Workers {
    type Own = WorkersState;

    fn decide(&self, v: &View<'_, WorkersState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        // The hours whose work cost χ ~ U[0, χ_max] is at most ln(1 + w/P_s): a share
        // F = ln1p(w/P_s)/χ_max of N, at most all of them (the support of χ, not a clamp).
        let w = price(v, self.labour)?;
        let p = price(v, self.basket.good)?;
        let r = price(v, self.basket.space)?;
        let h = param(v, self.basket.per_basket)?;
        let s = (num::ln1p(w / (p + h * r)) / param(v, self.chi_max)?).min(1.0);
        // Only the offer is minted, and all of it is offered.
        let hours = param(v, self.heads)? * s;
        if hours > 0.0 {
            out.deltas.push(StateDelta::Mint {
                to: Holder::Actor(v.me),
                good: self.labour,
                qty: hours,
                prov: Provenance::Endowment,
            });
        }
        out.orders.push(sell(v, self.labour, hours));
        let budget = param(v, self.spend)? * dry.get(v.currency);
        basket_orders(v, &self.basket, budget, &mut dry, &mut out)?;
        out.deltas
            .push(set(v, ActorState::Workers(WorkersState { share: s })));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, WorkersState>) -> Result<Vec<Delta>, AgentError> {
        eat(v, &self.basket)
    }

    fn upkeep(&self, _: &View<'_, WorkersState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}

/// The schedule's values, read at use time.
pub(crate) struct Tasks {
    eta: f64,
    g0: f64,
    g1: f64,
    k: f64,
}

impl Tasks {
    pub(crate) fn read<S>(v: &View<'_, S>, s: &Schedule) -> Result<Tasks, AgentError> {
        Ok(Tasks {
            eta: param(v, s.eta)?,
            g0: param(v, s.g0)?,
            g1: param(v, s.g1)?,
            k: param(v, s.k)?,
        })
    }

    /// J(x) = η(g0·x + g1·x^(k+1)/(k+1)): the machine services per unit of output on [0, x).
    pub(crate) fn j(&self, x: f64) -> f64 {
        let k1 = self.k + 1.0;
        self.eta * (self.g0 * x + self.g1 * num::pow(x, k1) / k1)
    }

    /// The measure of the tasks i in [0, 1] on which a machine is cheaper at posted prices,
    /// γ(i)·p_m < w, γ(i) = η(g0 + g1·i^k): i^k < z with z = (w/(η·p_m) − g0)/g1. It is a
    /// measure of a subset of [0, 1], so it is 0 when z ≤ 0 and 1 when z ≥ 1; these are the
    /// corners x = 0 (w/p_m ≤ γ(0)) and x = 1 (w/p_m ≥ γ(1)), reached without a clamp.
    pub(crate) fn measure(&self, ratio: f64) -> f64 {
        let z = (ratio / self.eta - self.g0) / self.g1;
        if z.is_nan() || z <= 0.0 {
            0.0
        } else if z >= 1.0 {
            1.0
        } else {
            num::pow(z, 1.0 / self.k)
        }
    }
}

/// The largest double t in [0, 1] for which `holds` is true, given that it holds at 0, fails at
/// 1 and is monotone between: a bisection over the bit patterns of the non-negative doubles,
/// which order like their values. At most 64 halvings, and no tolerance.
fn largest_where(holds: impl Fn(f64) -> bool) -> f64 {
    let one: f64 = 1.0;
    let (mut lo, mut hi) = (0u64, one.to_bits());
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if holds(f64::from_bits(mid)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::from_bits(lo)
}

/// The ex-post assignment: the cutoff x_u that uses up `l` hours and `m` machine services,
/// l·J(x_u) = m·(1 − x_u), as (1 − x_u, J(x_u)). With no hours every task goes to machines
/// (x_u = 1), and with no machine services to people (x_u = 0). Otherwise x_u is the largest x
/// with fl(l·J(x)) ≤ fl(m·(1 − x)). When that root lies nearer x = 1, the human share
/// s = 1 − x_u is resolved in its own right, as the largest s with fl(m·s) < fl(l·J(1 − s)),
/// so that a small s keeps its relative precision instead of inheriting the spacing of the
/// doubles near 1.
pub(crate) fn assign_ex_post(t: &Tasks, l: f64, m: f64, planned: f64) -> (f64, f64) {
    let one: f64 = 1.0;
    if l == 0.0 && m == 0.0 {
        return (planned, t.j(one - planned));
    }
    if l == 0.0 || l * t.j(one) <= 0.0 {
        return (0.0, t.j(one));
    }
    if m == 0.0 {
        return (one, t.j(0.0));
    }
    let x = largest_where(|x| l * t.j(x) <= m * (one - x));
    if one - x < x {
        let s = largest_where(|s| m * s < l * t.j(one - s));
        return (s, t.j(one - s));
    }
    (one - x, t.j(x))
}

/// A step rule's next scale: q·exp(step·(m − d)) above the band, q·exp(step·(m + d)) below it,
/// q inside it.
fn stepped<S>(
    v: &View<'_, S>,
    q: f64,
    m: f64,
    up: Site,
    down: Site,
    dead: Site,
) -> Result<f64, AgentError> {
    let d = param(v, dead)?;
    let g = if m > d {
        param(v, up)? * (m - d)
    } else if m < -d {
        param(v, down)? * (m + d)
    } else {
        0.0
    };
    Ok(q * num::exp(g))
}

/// What a desk sees of its margin at posted prices.
pub(crate) struct Margin {
    /// The cash cost of one unit made.
    pub(crate) cost: f64,
    /// The price over that cost; for the machine desk, net of its own input, p_m(1 − a)/c.
    pub(crate) markup: f64,
    /// The value at posted prices of the output it holds to sell, net of its own input: the
    /// cash rule's ceiling reads its stationary coin from it.
    pub(crate) worth: f64,
}

/// What a desk's scale rule decides: the coin it spends on inputs this tick, and its scale
/// state. Any payout is made first, from the copy of the holding.
pub(crate) fn scale_outlay<S>(
    v: &View<'_, S>,
    scale: &Scale,
    state: f64,
    m: &Margin,
    dry: &mut Inventory,
    out: &mut Decision,
) -> Result<(f64, f64), AgentError> {
    let (c, mu, worth) = (m.cost, m.markup, m.worth);
    match *scale {
        Scale::Cash {
            turnover,
            tilt,
            payout,
        } => {
            if let Some(ceiling) = payout {
                let target = worth / param(v, turnover)?;
                let floor = num::exp(param(v, ceiling.ceiling)?) * target;
                pay_above(v, ceiling.to, ceiling.rate, floor, dry, out)?;
            }
            // share(v·μ^κ): the annual turnover is tilted, then converted by the turnover's
            // own site, a `Share`.
            let rate = turnover.value(&v.params)? * num::pow(mu, param(v, tilt)?);
            let outlay = turnover.convert(v.clock, rate)? * dry.get(v.currency);
            Ok((outlay, outlay / c))
        }
        Scale::Step {
            up,
            down,
            dead,
            buffer,
            to,
            rate,
            ..
        } => {
            let q = stepped(v, state, num::ln(mu), up, down, dead)?;
            let want = c * q;
            pay_above(v, to, rate, want / param(v, buffer)?, dry, out)?;
            Ok((want.min(dry.get(v.currency)), q))
        }
    }
}

impl Behaviour for GoodDesk {
    type Own = GoodDeskState;

    fn decide(&self, v: &View<'_, GoodDeskState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, self.labour)?;
        let pm = price(v, self.mach)?;
        let p = price(v, self.output)?;
        let t = Tasks::read(v, &self.schedule)?;
        // The technique: the human share moves a share of its gap to 1 − (the measure of the
        // tasks a machine does more cheaply) each tick. That is M3 at rest.
        let target = 1.0 - t.measure(w / pm);
        let s0 = v.own_state.share;
        let s = s0 + param(v, self.adjust)? * (target - s0);
        let j = t.j(1.0 - s);
        // Unit cost at posted prices; the markup p/c is 1 at rest (M2).
        let c = s * w + j * pm;
        let margin = Margin {
            cost: c,
            markup: p / c,
            worth: p * v.own.get(self.output),
        };
        let (outlay, scale) = scale_outlay(
            v,
            &self.scale,
            v.own_state.scale,
            &margin,
            &mut dry,
            &mut out,
        )?;
        let q = outlay / c;
        let offered = offer(&mut dry, self.output, v.own.get(self.output))?;
        out.orders.push(sell(v, self.output, offered));
        let (bl, bm) = two_budgets(outlay, w * (s * q), pm * (j * q), &mut dry, v.currency)?;
        out.orders.push(buy(v, self.labour, s * q, bl));
        out.orders.push(buy(v, self.mach, j * q, bm));
        out.deltas.push(set(
            v,
            ActorState::GoodDesk(GoodDeskState {
                share: s,
                scale,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, GoodDeskState>) -> Result<Vec<Delta>, AgentError> {
        let t = Tasks::read(v, &self.schedule)?;
        let planned = v.own_state.share;
        let l = v.own.get(self.labour);
        let m = v.own.get(self.mach);
        let (used, j) = match self.assign {
            Assign::Planned => (planned, t.j(1.0 - planned)),
            Assign::ExPost => assign_ex_post(&t, l, m, planned),
        };
        let y = leontief(&[(l, used), (m, j)])?;
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if y > 0.0 {
            burn(me, self.labour, used * y, Provenance::Production, &mut out);
            burn(me, self.mach, j * y, Provenance::Production, &mut out);
            out.push(StateDelta::Mint {
                to: me,
                good: self.output,
                qty: y,
                prov: Provenance::Production,
            });
        }
        out.push(set(
            v,
            ActorState::GoodDesk(GoodDeskState {
                used,
                output: y,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn upkeep(&self, _: &View<'_, GoodDeskState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}

impl Behaviour for MachDesk {
    type Own = MachDeskState;

    fn decide(&self, v: &View<'_, MachDeskState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, self.labour)?;
        let r = price(v, self.land)?;
        let pm = price(v, self.output)?;
        let a = param(v, self.recipe.own)?;
        let lam = param(v, self.recipe.labour)?;
        let b = param(v, self.recipe.land)?;
        // The cash cost of one unit made, and its markup in the net form: what one unit earns
        // once its own input is kept, p_m(1 − a), over that cost. 1 at rest (M1).
        let c = lam * w + b * r;
        let held = v.own.get(self.output);
        let net = pm * (1.0 - a);
        let margin = Margin {
            cost: c,
            markup: net / c,
            worth: net * held,
        };
        let (outlay, scale) = scale_outlay(
            v,
            &self.scale,
            v.own_state.scale,
            &margin,
            &mut dry,
            &mut out,
        )?;
        let q = outlay / c;
        // It keeps a·q of what it made last tick for its own use and offers the rest; the
        // inputs it buys are ordered on the whole q.
        let keep = (a * q).min(held);
        let offered = offer(&mut dry, self.output, held - keep)?;
        out.orders.push(sell(v, self.output, offered));
        let (bl, br) = two_budgets(outlay, w * (lam * q), r * (b * q), &mut dry, v.currency)?;
        out.orders.push(buy(v, self.labour, lam * q, bl));
        out.orders.push(buy(v, self.land, b * q, br));
        out.deltas.push(set(
            v,
            ActorState::MachDesk(MachDeskState {
                scale,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, MachDeskState>) -> Result<Vec<Delta>, AgentError> {
        let a = param(v, self.recipe.own)?;
        let lam = param(v, self.recipe.labour)?;
        let b = param(v, self.recipe.land)?;
        let y = leontief(&[
            (v.own.get(self.output), a),
            (v.own.get(self.labour), lam),
            (v.own.get(self.land), b),
        ])?;
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if y > 0.0 {
            // The burn comes first, so it takes the lots made last tick; the mint makes a lot
            // that sells next tick.
            burn(me, self.output, a * y, Provenance::Production, &mut out);
            burn(me, self.labour, lam * y, Provenance::Production, &mut out);
            burn(me, self.land, b * y, Provenance::Production, &mut out);
            out.push(StateDelta::Mint {
                to: me,
                good: self.output,
                qty: y,
                prov: Provenance::Production,
            });
        }
        out.push(set(
            v,
            ActorState::MachDesk(MachDeskState {
                output: y,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn upkeep(&self, _: &View<'_, MachDeskState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}
