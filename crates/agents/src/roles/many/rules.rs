//! The many-market roles' rules (docs/probe/MARKETS-RULES.md; MARKETS-SPEC §2). Each is an
//! Appendix B role generalised: the households buy a basket of many items, a category desk works
//! its segments of the shared task line (decision 60) with the task type's services and direct
//! land, and a type desk buys other types' services beside its hours and land. As for the
//! Appendix B roles, each decision reads posted prices at the actor's home node, its own holding
//! and state, and the current params (R13), and nothing here reads a volume, a fill, another
//! actor or the oracle.
//!
//! **Nesting.** On Appendix B, with the basket [(good, 1), (land, h)], one segment of density 1,
//! θ = 1, no direct land and no bought service, each rule makes the Appendix B role's
//! floating-point operations exactly (MARKETS-SPEC §2.7): P_s is summed from 0.0 in item order,
//! so (0.0 + 1.0·p) + h·r = p + h·r; the top segment's hours are μ·s from the carried s, never
//! μ·(1.0 − x); task services use J(x) − J(e) with J(0) = 0.0 exactly; costs sum as
//! ((w·H) + (p_τ·M/θ)) + r·b and ((0.0 + Σ a·p) + λ·w) + b·r; budgets are p·(coef·q), cut from
//! one outlay by [`budget_chain`] in admission's (good) order, which for Appendix B's two inputs
//! is their list order, and so is `two_budgets`.
//!
//! **The wall's additions** (P2.3; docs/probe/WALL-RULES.md): a category desk's optional `tail`
//! L^H adds to its hours per unit, h = H + L^H, one addition and none without it; its optional
//! `reserved` inputs, each a reserved type's hours at R_ji a unit, add p_i·R_ji to its cost in
//! list order, are ordered at R_ji·q when R_ji is not zero, and enter its Leontief; the
//! provider's optional `more` pays further transfers N_i·P_s in list order from the coin left,
//! its state holding the sums. With all three absent every rule is P2.1's bit for bit.
//!
//! **The commons' addition** (P2.3; docs/probe/COMMONS-RULES.md): the workers' optional `exit`,
//! the priced exit s(q) in an exit good of their basket with a commons they hold and never trade.
//! Their hours become min(max(n(0), N − T_o/h), n(r̂)) ([`workers_participation`]), and where the
//! plots spill onto enclosed land they buy it at r in the chain of their baskets and burn it in
//! `produce`. Without it the workers take P2.1's path, and with it switched off (s₀ = s̲ = 0) they
//! make P2.1's hours, orders and share bit for bit.
//!
//! **The trap's remedy** (P2.4; docs/probe/TRAP-RULES.md): the exit's optional `pace`,
//! participation at a rate. The workers' share, their own state, moves a share a of its gap to
//! the rule's hours over N each tick, and they offer N times it; the plots stay the rule's.
//! Without it the rule's hours and share are offered and kept, P2.3's bit for bit.
//!
//! The arithmetic cannot fail, as RULES §2 says of the Appendix B roles: every budget is capped
//! by the copy of the holding and taken from it, each after the first is at most
//! `max_remainder(outlay, spent so far)`, sells take from the same copy, and recipes run at
//! `max_scale` over the inputs whose coefficient is not zero.

use crate::behaviour::{AgentError, Behaviour, Decision, View};
use crate::ext::{
    ActorState, GoodDeskState, MachDeskState, PlantState, ProviderState, WorkersState,
};
use crate::roles::many::spec::{BasketProvider, BasketWorkers, CategoryDesk, Item, TypeDesk};
use crate::roles::plant::rules::{built, burned, mint_plant, sign, split, Plan, PlantNow};
use crate::roles::rules::{
    burn, buy, leontief, offer, param, price, scale_outlay, sell, set, take, Delta, Margin, Tasks,
};
use rustyecon_core::num;
use rustyecon_core::{Amount, GoodId, Holder, Inventory, Provenance, Site, StateDelta};

/// Budgets out of one outlay, a chain over the wanted budgets (MARKETS-SPEC §2.6): the first at
/// most its want, the outlay and the coin the copy holds; each later one at most its want,
/// `max_remainder(total, the budgets before it)` and the coin the copy still holds. Each is
/// taken from the copy, as admission will take it, and in admission's order: by good, since
/// admission walks an actor's orders by (node, good) and every role trades at its home node. So
/// the copy's coin falls through exactly the subtractions admission's does, and no budget is
/// refused, whatever the order the inputs are listed in (in another order, the pair's rounded
/// sum can pass the coin by half an ulp of the outlay). `wants` pairs each good with its want;
/// the budgets come back in the order given. For the Appendix B roles' two inputs, listed in
/// good order, this is `two_budgets` exactly.
pub(crate) fn budget_chain(
    total: f64,
    wants: &[(GoodId, f64)],
    dry: &mut Inventory,
    currency: GoodId,
) -> Result<Vec<f64>, AgentError> {
    let mut order: Vec<usize> = (0..wants.len()).collect();
    order.sort_by_key(|&i| wants[i].0);
    let mut out = vec![0.0; wants.len()];
    let mut spent = 0.0;
    for (n, &i) in order.iter().enumerate() {
        let room = if n == 0 {
            total
        } else {
            num::max_remainder(total, spent)?
        };
        let b = wants[i].1.min(room).min(dry.get(currency));
        take(dry, currency, b)?;
        spent += b;
        out[i] = b;
    }
    Ok(out)
}

/// P_s = Σ_j z_j·p_j at posted prices, summed from 0.0 in item order.
fn basket_price<S>(v: &View<'_, S>, items: &[Item]) -> Result<f64, AgentError> {
    basket_price_at(items, &|s| param(v, s), &|g| price(v, g))
}

/// A param's per-tick value at one of its sites, as a rule reads it.
pub type ParamAt<'a> = dyn Fn(Site) -> Result<f64, AgentError> + 'a;
/// A good's posted price, as a rule reads it.
pub type PriceOf<'a> = dyn Fn(GoodId) -> Result<f64, AgentError> + 'a;

/// P_s from `par` and `pr`, summed from 0.0 in item order: [`basket_price`]'s operations.
fn basket_price_at(items: &[Item], par: &ParamAt<'_>, pr: &PriceOf<'_>) -> Result<f64, AgentError> {
    let mut ps = 0.0;
    for it in items {
        ps += par(it.weight)? * pr(it.good)?;
    }
    Ok(ps)
}

/// Where the workers' exit plots stand at posted prices (the commons frame's §3.1; unit 1e's
/// `ExitLand` for one type, docs/unit-1e.md §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotRegime {
    /// No plot pays at any rent (h = 0 or s₀ ≤ s̲): the supply is the floor's, with no plots.
    Unused,
    /// The plots fit the commons at no rent: hours n(0), and the rest of the commons idles.
    Commons,
    /// The commons is full: hours N − T_o/h, rationed by a shadow rent that nobody receives.
    Crowded,
    /// A plot pays at the market rent r and the plots spill onto enclosed land: hours n(r), and
    /// T_p = h·(N − n(r)) − T_o is rented on the land market.
    Enclosed,
    /// The commons fills at the rent where a plot stops paying, below r: hours are the floor's
    /// supply and no plot is rented (unit-1e.md §4.4).
    Split,
}

/// The workers' participation under the priced exit with the commons they hold, at posted prices
/// (the commons frame's §3.1): the rule's hours and every quantity it forms on the way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Participation {
    /// Where the plots stand.
    pub regime: PlotRegime,
    /// The hours offered, min(max(n(0), N − T_o/h), n(r̂)).
    pub hours: f64,
    /// The share kept in `WorkersState`: the regime's F (F(e₀) in Commons, (N − T_o/h)/N when
    /// Crowded, F(e_r̂) when Enclosed or split, F(p_g·s̲) when unused), hours/N to one rounding.
    pub share: f64,
    /// T_p, the enclosed land the plots rent; 0 but in the Enclosed regime.
    pub plots: f64,
    /// n(0), the supply with every plot free.
    pub free: f64,
    /// n(r̂), the supply at the rent a plot pays on enclosed land, or the floor's.
    pub paid: f64,
    /// N − T_o/h, the supply that fills the commons; 0 when unused.
    pub crowded: f64,
    /// P_s, the basket's price.
    pub basket_price: f64,
    /// N, the heads' hours a tick.
    pub heads: f64,
}

/// F(e) = min(max(ln1p((w − e)/(P_s + e))/χ_max, 0), 1): the share of the heads whose work cost
/// χ ~ U[0, χ_max] is below the exit's, e the exit life's money value. The bounds 0 and 1 are the
/// support of χ, the shares F can take, not a clamp on a price (R3).
fn worth_working(chi: f64, w: f64, ps: f64, e: f64) -> f64 {
    (num::ln1p((w - e) / (ps + e)) / chi).clamp(0.0, 1.0)
}

/// The workers' participation from their spec, `par` (a param's per-tick value at one of its
/// sites) and `pr` (a good's posted price), as their rule forms it; `None` without an exit. The
/// harness reads its readouts (the regime, T_p) through this function, so the two cannot part.
///
/// Its evaluation order is the commons frame's §3.1: P_s summed as the basket's price; then
/// e₀ = p_g·s₀; the branch test r·h < p_g·Δ with both products rounded once; e_r by `num::fma`;
/// each n by `num::ln1p` of (w − e)/(P_s + e); then N − T_o/h. With s₀ = s̲ = 0 every e is 0.0,
/// so F is ln1p(w/P_s)/χ_max within [0, 1], P2.1's share bit for bit whenever w > 0.
pub fn workers_participation(
    workers: &BasketWorkers,
    par: &ParamAt<'_>,
    pr: &PriceOf<'_>,
) -> Result<Option<Participation>, AgentError> {
    let Some(x) = &workers.exit else {
        return Ok(None);
    };
    let w = pr(workers.labour)?;
    let ps = basket_price_at(&workers.basket, par, pr)?;
    let n = par(workers.heads)?;
    let chi = par(workers.chi_max)?;
    let pg = pr(x.good)?;
    let r = pr(x.land)?;
    let (s0, sf, h, to) = (par(x.gross)?, par(x.floor)?, par(x.plot)?, par(x.commons)?);
    Ok(Some(participation(n, chi, w, ps, pg, r, (s0, sf, h, to))))
}

/// The rule itself (the commons frame's §3.1), on numbers: heads N, χ_max, the wage w, P_s, the
/// exit good's price p_g, the land rent r, and the exit (s₀, s̲, h, T_o), N and T_o per tick.
fn participation(
    n: f64,
    chi: f64,
    w: f64,
    ps: f64,
    pg: f64,
    r: f64,
    (s0, sf, h, to): (f64, f64, f64, f64),
) -> Participation {
    let delta = s0 - sf;
    let at = |regime, hours, share, plots, free, paid, crowded| Participation {
        regime,
        hours,
        share,
        plots,
        free,
        paid,
        crowded,
        basket_price: ps,
        heads: n,
    };
    // A pop that takes no plot supplies the floor's hours.
    if !(h > 0.0 && delta > 0.0) {
        let f = worth_working(chi, w, ps, pg * sf);
        let hours = n * f;
        return at(PlotRegime::Unused, hours, f, 0.0, hours, hours, 0.0);
    }
    let e0 = pg * s0;
    let f0 = worth_working(chi, w, ps, e0);
    let free = n * f0;
    // A plot pays at the market rent where r·h < p_g·Δ; the exit is then worth p_g·s₀ − r·h,
    // rounded once. Where it does not pay, the floor's.
    let plot_pays = r * h < pg * delta;
    let fr = if plot_pays {
        worth_working(chi, w, ps, num::fma(-r, h, e0))
    } else {
        worth_working(chi, w, ps, pg * sf)
    };
    let paid = n * fr;
    let crowded = n - to / h;
    if free >= crowded {
        at(PlotRegime::Commons, free, f0, 0.0, free, paid, crowded)
    } else if paid > crowded {
        at(
            PlotRegime::Crowded,
            crowded,
            crowded / n,
            0.0,
            free,
            paid,
            crowded,
        )
    } else if plot_pays {
        let plots = h * (n - paid) - to;
        at(PlotRegime::Enclosed, paid, fr, plots, free, paid, crowded)
    } else {
        at(PlotRegime::Split, paid, fr, 0.0, free, paid, crowded)
    }
}

/// A household's baskets: `budget` at P_s buys n = budget/P_s baskets, posted as a buy of z_j·n
/// of each item with budget p_j·(z_j·n), cut from one budget by [`budget_chain`].
fn basket_orders<S>(
    v: &View<'_, S>,
    items: &[Item],
    ps: f64,
    budget: f64,
    dry: &mut Inventory,
    out: &mut Decision,
) -> Result<(), AgentError> {
    let n = budget / ps;
    let mut qty = Vec::with_capacity(items.len());
    let mut wants = Vec::with_capacity(items.len());
    for it in items {
        let q = param(v, it.weight)? * n;
        qty.push(q);
        wants.push((it.good, price(v, it.good)? * q));
    }
    let budgets = budget_chain(budget, &wants, dry, v.currency)?;
    for ((it, q), b) in items.iter().zip(qty).zip(budgets) {
        out.orders.push(buy(v, it.good, q, b));
    }
    Ok(())
}

/// A household eats min_j(held_j/z_j) baskets of what it holds, as `Consumption`, burning z_j·n
/// of each item. What is not eaten dies at 5a: every item is a one-tick good.
fn eat<S>(v: &View<'_, S>, items: &[Item]) -> Result<Vec<Delta>, AgentError> {
    let mut held = Vec::with_capacity(items.len());
    for it in items {
        held.push((v.own.get(it.good), param(v, it.weight)?));
    }
    let n = leontief(&held)?;
    let me = Holder::Actor(v.me);
    let mut out = Vec::new();
    for (it, (_, z)) in items.iter().zip(held) {
        burn(me, it.good, z * n, Provenance::Consumption, &mut out);
    }
    Ok(out)
}

impl Behaviour for BasketProvider {
    type Own = ProviderState;

    fn decide(&self, v: &View<'_, ProviderState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let me = Holder::Actor(v.me);
        let mut dry = v.own.clone();
        // T of land services, endowed and all offered; any space it eats it buys on the market.
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
        let ps = basket_price(v, &self.basket)?;
        let mut due = param(v, self.heads)? * ps;
        let mut paid = due.min(dry.get(v.currency));
        if paid > 0.0 {
            take(&mut dry, v.currency, paid)?;
            out.deltas.push(StateDelta::Transfer {
                from: me,
                to: Holder::Actor(self.transfer_to),
                good: v.currency,
                amount: Amount::Qty(paid),
            });
        }
        // Each further transfer in list order, N_i·P_s from the coin the copy still holds
        // (P2.3); the state holds the sums, ((N_0·P_s) + N_1·P_s) + …, so certify's shortfall
        // reads them all. With none, due and paid are the old numbers bit for bit.
        for t in &self.more {
            let due_i = param(v, t.heads)? * ps;
            let paid_i = due_i.min(dry.get(v.currency));
            if paid_i > 0.0 {
                take(&mut dry, v.currency, paid_i)?;
                out.deltas.push(StateDelta::Transfer {
                    from: me,
                    to: Holder::Actor(t.to),
                    good: v.currency,
                    amount: Amount::Qty(paid_i),
                });
            }
            due += due_i;
            paid += paid_i;
        }
        let budget = param(v, self.spend)? * dry.get(v.currency);
        basket_orders(v, &self.basket, ps, budget, &mut dry, &mut out)?;
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

impl Behaviour for BasketWorkers {
    type Own = WorkersState;

    fn decide(&self, v: &View<'_, WorkersState>) -> Result<Decision, AgentError> {
        if self.exit.is_some() {
            return self.decide_with_exit(v);
        }
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        // The hours whose work cost χ ~ U[0, χ_max] is at most ln(1 + w/P_s), with P_s the
        // basket's price: a share F of N, at most all of them (the support of χ, not a clamp).
        let w = price(v, self.labour)?;
        let ps = basket_price(v, &self.basket)?;
        let s = (num::ln1p(w / ps) / param(v, self.chi_max)?).min(1.0);
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
        basket_orders(v, &self.basket, ps, budget, &mut dry, &mut out)?;
        out.deltas
            .push(set(v, ActorState::Workers(WorkersState { share: s })));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, WorkersState>) -> Result<Vec<Delta>, AgentError> {
        let mut out = eat(v, &self.basket)?;
        // The plots use the enclosed land the workers rented for them (the commons frame's
        // §3.2): all of it is burned as `Consumption`, so none of it spoils.
        if let Some(x) = &self.exit {
            let me = Holder::Actor(v.me);
            burn(
                me,
                x.land,
                v.own.get(x.land),
                Provenance::Consumption,
                &mut out,
            );
        }
        Ok(out)
    }

    fn upkeep(&self, _: &View<'_, WorkersState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}

impl BasketWorkers {
    /// Their decision with the priced exit and the commons they hold (the commons frame's §3.1,
    /// §3.2; decision 398): the hours [`workers_participation`] gives, offered as P2.1's are;
    /// and, where the plots spill onto enclosed land, one buy of T_p land at r in the same budget
    /// chain as the baskets. The chain's total is P + share(spend)·(C − P), with C the coin as the
    /// phase began and P = min(r·T_p, C) the plots' rent the coin covers, so the rent comes first
    /// and the baskets from the rest. Where T_p = 0 the baskets' budget is share(spend)·C and the
    /// orders are P2.1's. The state holds the regime's F; with the exit's `pace` (P2.4) it holds
    /// the paced share, and the hours are N times it.
    fn decide_with_exit(&self, v: &View<'_, WorkersState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let p = workers_participation(self, &|s| param(v, s), &|g| price(v, g))?.ok_or(
            AgentError::Core(rustyecon_core::CoreError::Shape(
                "the workers' exit rule without an exit".into(),
            )),
        )?;
        // Participation at a rate (P2.4; the trap scan's §5.2; decision 404): with a pace the
        // share of the heads offering hours moves a share a of its gap to the rule's share
        // F* = hours/N, s = s₀ + a·(F* − s₀), the technique's form, and they offer N·s; only the
        // hours lag, the plots below are the rule's. A convex combination of two shares in
        // [0, 1], so nothing is clamped (R3). Without one they offer the rule's hours and keep
        // its share, P2.3's bit for bit.
        let (hours, share) = match self.exit.as_ref().and_then(|x| x.pace) {
            Some(pace) => {
                let a = param(v, pace.adjust)?;
                let target = p.hours / p.heads;
                let s0 = v.own_state.share;
                let s = s0 + a * (target - s0);
                (p.heads * s, s)
            }
            None => (p.hours, p.share),
        };
        if hours > 0.0 {
            out.deltas.push(StateDelta::Mint {
                to: Holder::Actor(v.me),
                good: self.labour,
                qty: hours,
                prov: Provenance::Endowment,
            });
        }
        out.orders.push(sell(v, self.labour, hours));
        let spend = param(v, self.spend)?;
        let coin = dry.get(v.currency);
        match &self.exit {
            Some(x) if p.plots > 0.0 => {
                let r = price(v, x.land)?;
                let rent = r * p.plots;
                let paid = rent.min(coin);
                let budget = spend * (coin - paid);
                let n = budget / p.basket_price;
                let mut lines: Vec<(GoodId, f64)> = Vec::with_capacity(self.basket.len() + 1);
                let mut wants = Vec::with_capacity(self.basket.len() + 1);
                for it in &self.basket {
                    let q = param(v, it.weight)? * n;
                    lines.push((it.good, q));
                    wants.push((it.good, price(v, it.good)? * q));
                }
                lines.push((x.land, p.plots));
                wants.push((x.land, rent));
                let budgets = budget_chain(paid + budget, &wants, &mut dry, v.currency)?;
                for ((g, q), b) in lines.into_iter().zip(budgets) {
                    out.orders.push(buy(v, g, q, b));
                }
            }
            _ => {
                let budget = spend * coin;
                basket_orders(v, &self.basket, p.basket_price, budget, &mut dry, &mut out)?;
            }
        }
        out.deltas
            .push(set(v, ActorState::Workers(WorkersState { share })));
        Ok(out)
    }
}

/// A category's segments of the line, read at use time: the edges 0, e_1, …, e_{S−1}, 1 and
/// the density on each segment.
struct Line {
    edges: Vec<f64>,
    density: Vec<f64>,
}

impl Line {
    fn read<S>(v: &View<'_, S>, d: &CategoryDesk) -> Result<Line, AgentError> {
        let mut edges = Vec::with_capacity(d.edges.len() + 2);
        edges.push(0.0);
        for e in &d.edges {
            edges.push(param(v, *e)?);
        }
        edges.push(1.0);
        let mut density = Vec::with_capacity(d.density.len());
        for m in &d.density {
            density.push(param(v, *m)?);
        }
        Ok(Line { edges, density })
    }

    /// L̄_j = Σ_s μ_js(e_s − e_{s−1}), the hours of the all-human method.
    fn all_human(&self) -> f64 {
        let mut l = 0.0;
        for (i, mu) in self.density.iter().enumerate() {
            l += mu * (self.edges[i + 1] - self.edges[i]);
        }
        l
    }

    /// Whether the category has tasks anywhere on the line.
    fn has_tasks(&self) -> bool {
        self.density.iter().any(|&mu| mu > 0.0)
    }

    /// H_j and M_j, hours and task services (at efficiency 1) per unit, at the human share s
    /// and threshold x = 1 − s (unit-1b.md §4.1, in its evaluation order, §5.1): machines on
    /// [e_{s−1}, e_s] ∩ [0, x), people on the rest. The top segment's hours are μ·s from the
    /// carried s, never μ·(1.0 − x), so that s near 0 keeps its relative precision; J(0) is
    /// 0.0 exactly.
    fn tasks(&self, t: &Tasks, s: f64) -> (f64, f64) {
        let x = 1.0 - s;
        let top = self.density.len();
        let (mut h, mut m) = (0.0, 0.0);
        for (i, &mu) in self.density.iter().enumerate() {
            let (lo, hi) = (self.edges[i], self.edges[i + 1]);
            let j_lo = if lo == 0.0 { 0.0 } else { t.j(lo) };
            if i + 1 == top && x > lo {
                h += mu * s;
                m += mu * (t.j(x) - j_lo);
            } else if x >= hi {
                m += mu * (t.j(hi) - j_lo);
            } else if x <= lo {
                h += mu * (hi - lo);
            } else {
                h += mu * (hi - x);
                m += mu * (t.j(x) - j_lo);
            }
        }
        (h, m)
    }
}

impl CategoryDesk {
    /// The pool's hours per unit, h = H + L^H with the tail if the desk has one (P2.3): one
    /// addition, and none without a tail, so the old path is untouched. The tail, if any, comes
    /// back with it.
    fn with_tail<S>(&self, v: &View<'_, S>, h: f64) -> Result<(f64, Option<f64>), AgentError> {
        match self.tail {
            Some(site) => {
                let l = param(v, site)?;
                Ok((h + l, Some(l)))
            }
            None => Ok((h, None)),
        }
    }

    /// Each reserved input, in list order (P2.3): its good and its coefficient R_ji.
    fn reserved_now<S>(&self, v: &View<'_, S>) -> Result<Vec<(GoodId, f64)>, AgentError> {
        let mut out = Vec::with_capacity(self.reserved.len());
        for i in &self.reserved {
            out.push((i.good, param(v, i.coef)?));
        }
        Ok(out)
    }
}

impl Behaviour for CategoryDesk {
    type Own = GoodDeskState;

    fn decide(&self, v: &View<'_, GoodDeskState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, self.labour)?;
        let pt = price(v, self.service)?;
        let p = price(v, self.output)?;
        let r = price(v, self.land)?;
        let t = Tasks::read(v, &self.schedule)?;
        let theta = param(v, self.theta)?;
        // The technique: the human share moves a share of its gap to 1 − X, X the measure of
        // the line on which a machine of the task type is cheaper at posted prices,
        // γ(X) = θ·w/p_τ. It reads prices only, so every desk's threshold rests at one x*
        // (decision 60), whatever its own densities.
        let target = 1.0 - t.measure((theta * w) / pt);
        let s0 = v.own_state.share;
        let s = s0 + param(v, self.adjust)? * (target - s0);
        let line = Line::read(v, self)?;
        let (h, m) = line.tasks(&t, s);
        let (h, tail) = self.with_tail(v, h)?;
        let ms = m / theta;
        let b = param(v, self.direct_land)?;
        // Unit cost at posted prices, then each reserved type's hours at its own wage, in list
        // order (P2.3); the markup p/c is 1 at rest (M2j).
        let mut reserved = Vec::with_capacity(self.reserved.len());
        for (g, coef) in self.reserved_now(v)? {
            reserved.push((g, coef, price(v, g)?));
        }
        let mut c = ((w * h) + (pt * ms)) + (r * b);
        for &(_, coef, pr) in &reserved {
            c += pr * coef;
        }
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
        // An order for every input the category can use, at quantity 0 where the planned x
        // makes its coefficient 0: hours (with a positive tail, whatever x), the task services,
        // land, and each reserved input whose coefficient is not zero. Each line carries its
        // price, and its budget is p·(coef·q).
        let mut lines: Vec<(GoodId, f64, f64)> = Vec::with_capacity(3 + reserved.len());
        if line.all_human() > 0.0 || tail.is_some_and(|l| l > 0.0) {
            lines.push((self.labour, h * q, w));
        }
        if line.has_tasks() {
            lines.push((self.service, ms * q, pt));
        }
        if b > 0.0 {
            lines.push((self.land, b * q, r));
        }
        for &(g, coef, pr) in &reserved {
            if coef > 0.0 {
                lines.push((g, coef * q, pr));
            }
        }
        let wants: Vec<(GoodId, f64)> = lines.iter().map(|&(g, qty, pg)| (g, pg * qty)).collect();
        let budgets = budget_chain(outlay, &wants, &mut dry, v.currency)?;
        for ((g, qty, _), bud) in lines.into_iter().zip(budgets) {
            out.orders.push(buy(v, g, qty, bud));
        }
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
        // Leontief at the planned technique over the inputs whose coefficient is not zero.
        let t = Tasks::read(v, &self.schedule)?;
        let planned = v.own_state.share;
        let line = Line::read(v, self)?;
        let (h, m) = line.tasks(&t, planned);
        let (h, _) = self.with_tail(v, h)?;
        let ms = m / param(v, self.theta)?;
        let b = param(v, self.direct_land)?;
        let reserved = self.reserved_now(v)?;
        let mut inputs = Vec::with_capacity(3 + reserved.len());
        inputs.push((v.own.get(self.labour), h));
        inputs.push((v.own.get(self.service), ms));
        inputs.push((v.own.get(self.land), b));
        for &(g, coef) in &reserved {
            inputs.push((v.own.get(g), coef));
        }
        let y = leontief(&inputs)?;
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if y > 0.0 {
            burn(me, self.labour, h * y, Provenance::Production, &mut out);
            burn(me, self.service, ms * y, Provenance::Production, &mut out);
            burn(me, self.land, b * y, Provenance::Production, &mut out);
            for (g, coef) in reserved {
                burn(me, g, coef * y, Provenance::Production, &mut out);
            }
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
                used: planned,
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

impl TypeDesk {
    /// Its decision, with its plant if it has one (P2.2b; LOOPS-RULES §4.1): the orders and the
    /// deltas but its own state, which comes back apart with the plant's plan. Without a plant it
    /// is P2.1's type desk to the bit.
    pub(crate) fn plan<S>(
        &self,
        v: &View<'_, S>,
        st: &MachDeskState,
        plant: Option<&PlantNow>,
    ) -> Result<(Decision, MachDeskState, Option<Plan>), AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, self.labour)?;
        let r = price(v, self.land)?;
        let pk = price(v, self.output)?;
        let a = param(v, self.own)?;
        let lam = param(v, self.labour_coef)?;
        let b = param(v, self.land_coef)?;
        // The cash cost of one unit made, its bought services first, in list order, then
        // hours and land; and its markup in the net form, p_k(1 − a_kk)/c_k, 1 at rest (M1k).
        // With a plant, c is the bundle's cost, and the margin reads it (decision 295).
        let mut bought = Vec::with_capacity(self.inputs.len());
        let mut c = 0.0;
        for i in &self.inputs {
            let (coef, pl) = (param(v, i.coef)?, price(v, i.good)?);
            c += coef * pl;
            bought.push((i.good, coef, pl));
        }
        c += lam * w;
        c += b * r;
        let held = v.own.get(self.output);
        let net = pk * (1.0 - a);
        let margin = Margin {
            cost: c,
            markup: net / c,
            worth: net * held,
        };
        let (outlay, scale) = scale_outlay(v, &self.scale, st.scale, &margin, &mut dry, &mut out)?;
        // The plan: the cash rule at the full cost c_full = c + (u·P_K)·kr, which is c without
        // a plant, at θ = 1 and at u = 0.
        let ratios = plant.map(|p| p.ratios(c));
        let q = match &ratios {
            Some(x) => outlay / x.full,
            None => outlay / c,
        };
        // It keeps a_kk·q of what it made last tick for its own use and offers the rest; every
        // input it buys is ordered on the whole q (RULES' graft 4).
        let keep = (a * q).min(held);
        let offered = offer(&mut dry, self.output, held - keep)?;
        out.orders.push(sell(v, self.output, offered));
        // The plant: K*_p = kr·q, ordered by M3's rule, at most the coin left after the running
        // bundle buys, sign(C − c·q)/P_K. Its bundles are bought on the same lines, a_g·(q + s·I),
        // from a total of outlay + P_K·I, which is the outlay at I = 0.
        let (n, total, rec) = match (plant, ratios) {
            (Some(p), Some(x)) => {
                let held_p = v.own.get(p.good);
                let target = x.kr * q;
                let coin = v.own.get(v.currency);
                let order = p.order_of(target, held_p).min(sign(coin - c * q) / x.pk);
                let rec = Plan {
                    held: held_p,
                    target,
                    order,
                    run: q,
                };
                (q + p.s * order, outlay + x.pk * order, Some(rec))
            }
            _ => (q, outlay, None),
        };
        let mut lines: Vec<(GoodId, f64)> = Vec::with_capacity(bought.len() + 2);
        let mut wants = Vec::with_capacity(bought.len() + 2);
        for (g, coef, pl) in bought {
            let qty = coef * n;
            lines.push((g, qty));
            wants.push((g, pl * qty));
        }
        lines.push((self.labour, lam * n));
        wants.push((self.labour, w * (lam * n)));
        lines.push((self.land, b * n));
        wants.push((self.land, r * (b * n)));
        let budgets = budget_chain(total, &wants, &mut dry, v.currency)?;
        for ((g, qty), bud) in lines.into_iter().zip(budgets) {
            out.orders.push(buy(v, g, qty, bud));
        }
        Ok((out, MachDeskState { scale, ..*st }, rec))
    }

    /// Its production, with its plant if it has one (LOOPS-RULES §3.6): the burns and mints
    /// but its own state, which comes back apart with the plant units built.
    pub(crate) fn run<S>(
        &self,
        v: &View<'_, S>,
        st: &MachDeskState,
        plant: Option<(&PlantNow, &PlantState)>,
    ) -> Result<(Vec<Delta>, MachDeskState, f64), AgentError> {
        let a = param(v, self.own)?;
        let lam = param(v, self.labour_coef)?;
        let b = param(v, self.land_coef)?;
        let mut inputs = Vec::with_capacity(self.inputs.len() + 3);
        inputs.push((v.own.get(self.output), a));
        let mut coefs = Vec::with_capacity(self.inputs.len());
        for i in &self.inputs {
            let coef = param(v, i.coef)?;
            inputs.push((v.own.get(i.good), coef));
            coefs.push((i.good, coef));
        }
        inputs.push((v.own.get(self.labour), lam));
        inputs.push((v.own.get(self.land), b));
        // B, the bundles held, read over every unit held (carried units included, decision
        // 273). Without a plant it is the output.
        let bundles = leontief(&inputs)?;
        let (y, total, made) = match plant {
            None => (bundles, bundles, 0.0),
            Some((p, rec)) => {
                let (z, i) = split(bundles, rec.run, rec.order, p.s);
                let y = p.make(rec.held, z);
                let made = built(bundles, z, i, p.s)?;
                (y, burned(z, made, p.s), made)
            }
        };
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if total > 0.0 {
            // The own-input burn comes first, so it takes the lots made last tick; the mint
            // makes a lot that sells next tick.
            burn(me, self.output, a * total, Provenance::Production, &mut out);
            for (g, coef) in coefs {
                burn(me, g, coef * total, Provenance::Production, &mut out);
            }
            burn(
                me,
                self.labour,
                lam * total,
                Provenance::Production,
                &mut out,
            );
            burn(me, self.land, b * total, Provenance::Production, &mut out);
        }
        if y > 0.0 {
            out.push(StateDelta::Mint {
                to: me,
                good: self.output,
                qty: y,
                prov: Provenance::Production,
            });
        }
        if let Some((p, _)) = plant {
            mint_plant(me, p, made, &mut out);
        }
        Ok((out, MachDeskState { output: y, ..*st }, made))
    }
}

impl Behaviour for TypeDesk {
    type Own = MachDeskState;

    fn decide(&self, v: &View<'_, MachDeskState>) -> Result<Decision, AgentError> {
        let (mut out, state, _) = self.plan(v, v.own_state, None)?;
        out.deltas.push(set(v, ActorState::MachDesk(state)));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, MachDeskState>) -> Result<Vec<Delta>, AgentError> {
        let (mut out, state, _) = self.run(v, v.own_state, None)?;
        out.push(set(v, ActorState::MachDesk(state)));
        Ok(out)
    }

    fn upkeep(&self, _: &View<'_, MachDeskState>) -> Result<Vec<Delta>, AgentError> {
        Ok(Vec::new())
    }
}
