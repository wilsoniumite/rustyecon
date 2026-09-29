//! The stock roles' rules (P2.2; docs/probe/HORSES-RULES.md; HORSES-SPEC §2.6–§2.8): the maker
//! (M2), the capacity desk (M3, wet) and the owner desk (M1). A durable good is held as a stock
//! that serves κ hours a unit a tick, wears by δ a tick at upkeep (a `Depreciation` burn of δ
//! times the stock the rule recorded as serving), and is ordered by a rule that closes a share
//! of its gap to the cash rule's target beyond replacing wear. As for the other roles, each
//! decision reads posted prices at the actor's home node, its own holding and state, and the
//! current params (R13); nothing here reads a volume, a fill, another actor or the oracle.
//!
//! **Two paths, chosen by the durable good's life** at load ([`crate::Cast::new`]): an
//! `Indefinite` good is a stock (the stock path); a good that lives one tick is the probe's flow
//! service, and the maker and the owner desk then run the type desk's and the good desk's code
//! themselves (the flow path), so that with the stocks layer off they are those kinds exactly
//! (R1, HORSES-SPEC §1.6 and §2.6, §2.8). The capacity desk has no flow path: at δ = 1 its
//! phase-start holding is always zero (GOODS-CHAIN §3.1).
//!
//! The arithmetic cannot fail, as for the other roles: every budget is cut from one total by
//! the budget chain in admission's order and capped by the copy of the holding, sells take from
//! the same copy, recipes run at `max_scale` over the inputs whose coefficient is not zero, and
//! a wear burn is at most what is held. `max(·, 0)` on an order or a transfer into service is a
//! sign (R3): the rules never order a negative quantity; it caps nothing.

use crate::behaviour::{AgentError, Behaviour, Decision, View};
use crate::ext::{
    ActorState, AgentDelta, CapacityState, GoodDeskState, MachDeskState, MakerState, OwnerState,
};
use crate::roles::many::rules::budget_chain;
use crate::roles::many::spec::TypeDesk;
use crate::roles::rules::{
    assign_ex_post, burn, buy, leontief, offer, param, price, scale_outlay, sell, set, Delta,
    Margin, Tasks,
};
use crate::roles::spec::{Assign, GoodDesk};
use crate::roles::stock::spec::{CapacityDesk, Maker, OrderRule, OwnerDesk};
use rustyecon_core::num;
use rustyecon_core::{GoodId, Holder, Provenance, Site, StateDelta};

/// `x` when it is positive, else 0: the sign of an order or a transfer into service, which is
/// never negative (R3: a sign, not a cap). A NaN is 0.
fn sign(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

/// The same view of the actor with another state, for running an older kind's code on it.
fn with<'a, S, T>(v: &View<'a, S>, own_state: &'a T) -> View<'a, T> {
    View {
        tick: v.tick,
        clock: v.clock,
        me: v.me,
        class: v.class,
        home: v.home,
        currency: v.currency,
        own: v.own,
        own_state,
        posted: v.posted,
        params: v.params,
    }
}

/// Every `SetState` among `deltas` restated by `f`; the rest as they are.
fn restate(deltas: Vec<Delta>, f: impl Fn(ActorState) -> ActorState) -> Vec<Delta> {
    deltas
        .into_iter()
        .map(|d| match d {
            StateDelta::Actor(AgentDelta::SetState { actor, state }) => {
                StateDelta::Actor(AgentDelta::SetState {
                    actor,
                    state: f(state),
                })
            }
            other => other,
        })
        .collect()
}

/// A wear burn of `worn` of `good`, as `Depreciation`.
fn wear(me: Holder, good: GoodId, worn: f64, out: &mut Vec<Delta>) {
    burn(me, good, worn, Provenance::Depreciation, out);
}

/// A param's per-tick value at one of its sites, as a rule reads it.
type ParamAt<'a> = dyn Fn(Site) -> Result<f64, AgentError> + 'a;
/// A good's posted price at the actor's home node.
type PriceOf<'a> = dyn Fn(GoodId) -> Result<f64, AgentError> + 'a;

/// The goods a maker buys per unit built, in list order: the running recipe's goods, then the
/// build's not already listed, each with a·run_g + build_g.
fn bought_per_unit(m: &Maker, a: f64, par: &ParamAt<'_>) -> Result<Vec<(GoodId, f64)>, AgentError> {
    let mut out: Vec<(GoodId, f64, f64)> = Vec::with_capacity(m.running.goods.len());
    for i in &m.running.goods {
        out.push((i.good, par(i.coef)?, 0.0));
    }
    for i in &m.build.goods {
        let b = par(i.coef)?;
        match out.iter_mut().find(|x| x.0 == i.good) {
            Some(x) => x.2 = b,
            None => out.push((i.good, 0.0, b)),
        }
    }
    Ok(out
        .into_iter()
        .map(|(g, run, b)| (g, a * run + b))
        .collect())
}

/// A maker's labour per unit built, a·run_lab + build_lab, and land per unit.
fn labour_land_per_unit(m: &Maker, a: f64, par: &ParamAt<'_>) -> Result<(f64, f64), AgentError> {
    let lam = a * par(m.running.labour)? + par(m.build.labour)?;
    Ok((lam, par(m.build.land)?))
}

/// The cost of a unit built at posted prices, bought goods first in list order, then labour and
/// land, with the bought goods' prices: its own machines' running inputs included, their hours
/// not.
fn unit_cost(
    goods: &[(GoodId, f64)],
    lam: f64,
    b: f64,
    w: f64,
    r: f64,
    pr: &PriceOf<'_>,
) -> Result<(Vec<f64>, f64), AgentError> {
    let mut prices = Vec::with_capacity(goods.len());
    let mut c = 0.0;
    for &(g, coef) in goods {
        let pg = pr(g)?;
        c += coef * pg;
        prices.push(pg);
    }
    c += lam * w;
    c += b * r;
    Ok((prices, c))
}

/// The net price of a unit built: what it earns once the wear of the machines that built it is
/// kept, p_K·(1 − δ·a/κ). The rule and [`maker_reservation`] both form it here.
fn net_of_wear(pk: f64, d: f64, a: f64, kappa: f64) -> f64 {
    pk * (1.0 - d * a / kappa)
}

/// The maker's reservation ψ at its param's site, 0 without one. The rule and
/// [`maker_reservation`] both read it here.
fn reserve_of(m: &Maker, par: &ParamAt<'_>) -> Result<f64, AgentError> {
    match m.reserve {
        Some(site) => par(site),
        None => Ok(0.0),
    }
}

/// Whether a maker with reservation ψ withholds its finished stock at net markup `markup`: only
/// a reservation above 0 acts (L0.7), so a maker without one offers what P2.2a's offered at any
/// markup, a negative one included (δ·a/κ > 1, which no load check rules out).
pub fn withholds(psi: f64, markup: f64) -> bool {
    psi > 0.0 && markup < psi
}

/// A maker's reservation as its rule reads it on the stock path (IDLE-SPEC §6): its net markup
/// p_K·(1 − δ·a/κ)/c at posted prices, its reservation ψ (0 without one), and whether it
/// withholds. The rule forms these with the same code, from `par`, a param's per-tick value at
/// one of its sites, and `pr`, a good's posted price; an observer outside the Sim (the probe's
/// harness) reads them through [`maker_reservation`], so the two cannot part (L0.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reservation {
    /// The net markup.
    pub markup: f64,
    /// ψ, 0 without a reservation.
    pub psi: f64,
    /// Whether it offers none of its finished stock.
    pub withholds: bool,
}

/// [`Reservation`] for the maker `m`, from `par` and `pr` as its rule reads them.
pub fn maker_reservation(
    m: &Maker,
    par: &ParamAt<'_>,
    pr: &PriceOf<'_>,
) -> Result<Reservation, AgentError> {
    let w = pr(m.labour)?;
    let r = pr(m.land)?;
    let pk = pr(m.output)?;
    let a = par(m.own_hours)?;
    let kappa = par(m.kappa)?;
    let d = par(m.delta)?;
    let goods = bought_per_unit(m, a, par)?;
    let (lam, b) = labour_land_per_unit(m, a, par)?;
    let (_, c) = unit_cost(&goods, lam, b, w, r, pr)?;
    let net = net_of_wear(pk, d, a, kappa);
    let markup = net / c;
    let psi = reserve_of(m, par)?;
    Ok(Reservation {
        markup,
        psi,
        withholds: withholds(psi, markup),
    })
}

/// The maker (M2) as the cast runs it: its spec, and the type desk it is on the flow path.
#[derive(Debug, Clone)]
pub struct MakerRole {
    spec: Maker,
    flow: Option<TypeDesk>,
}

impl MakerRole {
    /// The maker of `spec`; on the flow path (`flow`) it runs the type desk's code on its own
    /// params: `own_hours` as the kept own input, the build as the recipe.
    pub fn new(spec: &Maker, flow: bool) -> MakerRole {
        let flow = flow.then(|| TypeDesk {
            output: spec.output,
            labour: spec.labour,
            land: spec.land,
            own: spec.own_hours,
            inputs: spec.build.goods.clone(),
            labour_coef: spec.build.labour,
            land_coef: spec.build.land,
            scale: spec.scale,
        });
        MakerRole {
            spec: spec.clone(),
            flow,
        }
    }

    /// The goods it buys per unit built, in list order: the running recipe's goods, then the
    /// build's not already listed, each with a·run_g + build_g.
    fn bought<S>(&self, v: &View<'_, S>, a: f64) -> Result<Vec<(GoodId, f64)>, AgentError> {
        bought_per_unit(&self.spec, a, &|s| param(v, s))
    }

    /// Labour per unit built, a·run_lab + build_lab, and land per unit.
    fn labour_land<S>(&self, v: &View<'_, S>, a: f64) -> Result<(f64, f64), AgentError> {
        labour_land_per_unit(&self.spec, a, &|s| param(v, s))
    }

    fn mach_state(s: &MakerState) -> MachDeskState {
        MachDeskState {
            scale: s.scale,
            output: s.output,
        }
    }

    fn from_mach(s: &MakerState) -> impl Fn(ActorState) -> ActorState + '_ {
        move |state| match state {
            ActorState::MachDesk(m) => ActorState::Maker(MakerState {
                scale: m.scale,
                output: m.output,
                ..*s
            }),
            other => other,
        }
    }
}

impl Behaviour for MakerRole {
    type Own = MakerState;

    fn decide(&self, v: &View<'_, MakerState>) -> Result<Decision, AgentError> {
        if let Some(t) = &self.flow {
            let st = MakerRole::mach_state(v.own_state);
            let mut d = t.decide(&with(v, &st))?;
            d.deltas = restate(d.deltas, MakerRole::from_mach(v.own_state));
            return Ok(d);
        }
        let m = &self.spec;
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, m.labour)?;
        let r = price(v, m.land)?;
        let pk = price(v, m.output)?;
        let a = param(v, m.own_hours)?;
        let kappa = param(v, m.kappa)?;
        let d = param(v, m.delta)?;
        let goods = self.bought(v, a)?;
        let (lam, b) = self.labour_land(v, a)?;
        // The cost of a unit built at posted prices, bought goods first in list order, then
        // labour and land: its own machines' running inputs included, their hours not.
        let (prices, c) = unit_cost(&goods, lam, b, w, r, &|g| price(v, g))?;
        // Its finished stock is what it holds beyond its serving stock; 0 where rounding puts
        // the record an ulp above the holding (a sign).
        let held = v.own.get(m.output);
        let own = v.own_state.own;
        let finished = sign(held - own);
        // The net markup: what a unit earns once the wear of the machines that built it is
        // kept, p_K·(1 − δ·a/κ), over its cost. 1 at rest.
        let net = net_of_wear(pk, d, a, kappa);
        let margin = Margin {
            cost: c,
            markup: net / c,
            worth: net * finished,
        };
        let (outlay, scale) =
            scale_outlay(v, &m.scale, v.own_state.scale, &margin, &mut dry, &mut out)?;
        let q = outlay / c;
        // Its own stock: K*_m = a·q/κ serves the plan; it moves δ·K*_m of its finished units
        // into service, and s_Km of its gap beyond, at most what is finished.
        let target = a * q / kappa;
        let gap = (1.0 - d) * target - own;
        let keep = sign(d * target + param(v, m.adjust)? * gap).min(finished);
        let serving = own + keep;
        // The offer: what is finished and not moved into service, less a cover of b_K ticks of
        // sales at cost over price (the reservation band, D-G5); finished units do not wear.
        let cover = match m.cover {
            Some(site) => param(v, site)?,
            None => 0.0,
        };
        let psi = reserve_of(m, &|s| param(v, s))?;
        // The reservation (IDLE-SPEC, L0.4): below ψ it offers none of its finished stock; it
        // holds it, unworn, and still buys and breeds on the whole q by its cash rule. Off (ψ 0
        // or absent) is structural (L0.7): the markup's sign also rests on 1 − δ·a/κ, which no
        // load check bounds, so `markup < 0.0` alone would withhold where δ·a/κ > 1. Off, the
        // offer is P2.2a's bit for bit at any markup.
        let want = if withholds(psi, margin.markup) {
            0.0
        } else {
            sign(finished - keep - cover * q * c / pk)
        };
        let offered = offer(&mut dry, m.output, want)?;
        out.orders.push(sell(v, m.output, offered));
        // Every input it buys, on the whole q.
        let mut lines: Vec<(GoodId, f64)> = Vec::with_capacity(goods.len() + 2);
        let mut wants = Vec::with_capacity(goods.len() + 2);
        for (&(g, coef), &pg) in goods.iter().zip(&prices) {
            let qty = coef * q;
            lines.push((g, qty));
            wants.push((g, pg * qty));
        }
        lines.push((m.labour, lam * q));
        wants.push((m.labour, w * (lam * q)));
        lines.push((m.land, b * q));
        wants.push((m.land, r * (b * q)));
        let budgets = budget_chain(outlay, &wants, &mut dry, v.currency)?;
        for ((g, qty), bud) in lines.into_iter().zip(budgets) {
            out.orders.push(buy(v, g, qty, bud));
        }
        out.deltas.push(set(
            v,
            ActorState::Maker(MakerState {
                scale,
                serving,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, MakerState>) -> Result<Vec<Delta>, AgentError> {
        if let Some(t) = &self.flow {
            let st = MakerRole::mach_state(v.own_state);
            let out = t.produce(&with(v, &st))?;
            return Ok(restate(out, MakerRole::from_mach(v.own_state)));
        }
        let m = &self.spec;
        let a = param(v, m.own_hours)?;
        let kappa = param(v, m.kappa)?;
        let goods = self.bought(v, a)?;
        let (lam, b) = self.labour_land(v, a)?;
        // Leontief over its serving stock's hours and what it bought; its machines are not
        // burned, and the units it makes join its finished stock, sold from the next tick.
        let serving = v.own_state.serving;
        let mut inputs = Vec::with_capacity(goods.len() + 3);
        inputs.push((kappa * serving, a));
        for &(g, coef) in &goods {
            inputs.push((v.own.get(g), coef));
        }
        inputs.push((v.own.get(m.labour), lam));
        inputs.push((v.own.get(m.land), b));
        let y = leontief(&inputs)?;
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if y > 0.0 {
            for &(g, coef) in &goods {
                burn(me, g, coef * y, Provenance::Production, &mut out);
            }
            burn(me, m.labour, lam * y, Provenance::Production, &mut out);
            burn(me, m.land, b * y, Provenance::Production, &mut out);
            out.push(StateDelta::Mint {
                to: me,
                good: m.output,
                qty: y,
                prov: Provenance::Production,
            });
        }
        out.push(set(
            v,
            ActorState::Maker(MakerState {
                output: y,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn upkeep(&self, v: &View<'_, MakerState>) -> Result<Vec<Delta>, AgentError> {
        if self.flow.is_some() {
            return Ok(Vec::new());
        }
        let m = &self.spec;
        // Its serving stock wears by δ; what is left serves again from the next decide.
        let serving = v.own_state.serving;
        let worn = (param(v, m.delta)? * serving).min(v.own.get(m.output));
        let mut out = Vec::new();
        wear(Holder::Actor(v.me), m.output, worn, &mut out);
        out.push(set(
            v,
            ActorState::Maker(MakerState {
                own: serving - worn,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }
}

/// Each running good with its coefficient and posted price, in list order.
type Run = Vec<(GoodId, f64, f64)>;

/// The running inputs of an hour at posted prices, and the running cost of their goods,
/// O = (0.0 + Σ run_g·p_g), summed in list order (labour is added after, where it has labour).
fn running<S>(
    v: &View<'_, S>,
    goods: &[crate::roles::many::spec::Input],
) -> Result<(Run, f64), AgentError> {
    let mut run = Vec::with_capacity(goods.len());
    let mut o = 0.0;
    for i in goods {
        let (coef, pg) = (param(v, i.coef)?, price(v, i.good)?);
        o += coef * pg;
        run.push((i.good, coef, pg));
    }
    Ok((run, o))
}

/// The hours a stock gives with the running inputs held: κ·stock, and at most what each held
/// input with a positive coefficient runs (`max_scale`).
fn hours_run(cap: f64, inputs: &[(f64, f64)]) -> Result<f64, AgentError> {
    let mut z = cap;
    for &(held, coef) in inputs {
        if coef > 0.0 {
            z = z.min(num::max_scale(held, coef)?);
        }
    }
    Ok(z)
}

impl Behaviour for CapacityDesk {
    type Own = CapacityState;

    fn decide(&self, v: &View<'_, CapacityState>) -> Result<Decision, AgentError> {
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, self.labour)?;
        let ph = price(v, self.hours)?;
        let pk = price(v, self.stock)?;
        let kappa = param(v, self.kappa)?;
        let d = param(v, self.delta)?;
        // The running cost of an hour at posted prices, O, and its full cost O + δ·p_K/κ.
        let (run, o0) = running(v, &self.running.goods)?;
        let run_lab = param(v, self.running.labour)?;
        let o = o0 + run_lab * w;
        let full = o + d * pk / kappa;
        let held = v.own.get(self.stock);
        let coin = v.own.get(v.currency);
        let margin = Margin {
            cost: full,
            markup: ph / full,
            worth: ph * v.own.get(self.hours),
        };
        let (outlay, scale) = scale_outlay(
            v,
            &self.scale,
            v.own_state.scale,
            &margin,
            &mut dry,
            &mut out,
        )?;
        // Use: every installed unit's hours while the outlay covers their running cost, the
        // cash rule at the running cost, so sunk capital competes at its operating cost.
        let cap = kappa * held;
        let z = if o > 0.0 { cap.min(outlay / o) } else { cap };
        // Investment: K* = B/(κ·O + δ·p_K), the cash rule at the full cost; the order replaces
        // wear and closes s_K of the gap, at most what the coin left after the running inputs
        // buys. In a glut the order stops (a sign).
        let target = outlay / (kappa * o + d * pk);
        let s = param(v, self.adjust)?;
        let base = match self.order {
            OrderRule::Target => d * target,
            OrderRule::Held => d * held,
        };
        let want = sign(base + s * (target - held));
        let order = want.min(sign(coin - o * z) / pk);
        // Every hour it holds is offered: they were made last tick.
        let offered = offer(&mut dry, self.hours, v.own.get(self.hours))?;
        out.orders.push(sell(v, self.hours, offered));
        let mut lines: Vec<(GoodId, f64)> = Vec::with_capacity(run.len() + 2);
        let mut wants = Vec::with_capacity(run.len() + 2);
        for &(g, coef, pg) in &run {
            let qty = coef * z;
            lines.push((g, qty));
            wants.push((g, pg * qty));
        }
        lines.push((self.labour, run_lab * z));
        wants.push((self.labour, w * (run_lab * z)));
        lines.push((self.stock, order));
        wants.push((self.stock, pk * order));
        // Their wants sum to at most the coin: the running inputs from the outlay, the order
        // from what is left.
        let budgets = budget_chain(coin, &wants, &mut dry, v.currency)?;
        for ((g, qty), bud) in lines.into_iter().zip(budgets) {
            out.orders.push(buy(v, g, qty, bud));
        }
        out.deltas.push(set(
            v,
            ActorState::Capacity(CapacityState {
                scale,
                held,
                target,
                order,
                run: z,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, CapacityState>) -> Result<Vec<Delta>, AgentError> {
        // Hours from the stock held when decide ran (units bought this tick serve from the
        // next), with the running inputs held.
        let kappa = param(v, self.kappa)?;
        let (run, _) = running(v, &self.running.goods)?;
        let run_lab = param(v, self.running.labour)?;
        let mut held_inputs: Vec<(f64, f64)> = run
            .iter()
            .map(|&(g, coef, _)| (v.own.get(g), coef))
            .collect();
        held_inputs.push((v.own.get(self.labour), run_lab));
        let z = hours_run(kappa * v.own_state.held, &held_inputs)?;
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if z > 0.0 {
            for &(g, coef, _) in &run {
                burn(me, g, coef * z, Provenance::Production, &mut out);
            }
            burn(
                me,
                self.labour,
                run_lab * z,
                Provenance::Production,
                &mut out,
            );
            out.push(StateDelta::Mint {
                to: me,
                good: self.hours,
                qty: z,
                prov: Provenance::Production,
            });
        }
        out.push(set(
            v,
            ActorState::Capacity(CapacityState {
                output: z,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn upkeep(&self, v: &View<'_, CapacityState>) -> Result<Vec<Delta>, AgentError> {
        // The stock held when decide ran wears by δ; units bought this tick do not.
        let worn = (param(v, self.delta)? * v.own_state.held).min(v.own.get(self.stock));
        let mut out = Vec::new();
        wear(Holder::Actor(v.me), self.stock, worn, &mut out);
        Ok(out)
    }
}

/// The owner desk (M1) as the cast runs it: its spec, and the good desk it is on the flow path.
#[derive(Debug, Clone)]
pub struct OwnerRole {
    spec: OwnerDesk,
    flow: Option<GoodDesk>,
}

impl OwnerRole {
    /// The owner desk of `spec`; on the flow path (`flow`) it runs the good desk's code with
    /// its durable good as the machine services it buys.
    pub fn new(spec: &OwnerDesk, flow: bool) -> OwnerRole {
        let flow = flow.then_some(GoodDesk {
            output: spec.output,
            labour: spec.labour,
            mach: spec.stock,
            schedule: spec.schedule,
            adjust: spec.technique,
            share: spec.share,
            assign: spec.assign,
            scale: spec.scale,
        });
        OwnerRole {
            spec: spec.clone(),
            flow,
        }
    }

    fn good_state(s: &OwnerState) -> GoodDeskState {
        GoodDeskState {
            share: s.share,
            used: s.used,
            scale: s.scale,
            output: s.output,
        }
    }

    fn from_good(s: &OwnerState) -> impl Fn(ActorState) -> ActorState + '_ {
        move |state| match state {
            ActorState::GoodDesk(g) => ActorState::Owner(OwnerState {
                share: g.share,
                used: g.used,
                scale: g.scale,
                output: g.output,
                ..*s
            }),
            other => other,
        }
    }
}

impl Behaviour for OwnerRole {
    type Own = OwnerState;

    fn decide(&self, v: &View<'_, OwnerState>) -> Result<Decision, AgentError> {
        if let Some(g) = &self.flow {
            let st = OwnerRole::good_state(v.own_state);
            let mut d = g.decide(&with(v, &st))?;
            d.deltas = restate(d.deltas, OwnerRole::from_good(v.own_state));
            return Ok(d);
        }
        let m = &self.spec;
        let mut out = Decision::default();
        let mut dry = v.own.clone();
        let w = price(v, m.labour)?;
        let pk = price(v, m.stock)?;
        let p = price(v, m.output)?;
        let kappa = param(v, m.kappa)?;
        let d = param(v, m.delta)?;
        let t = Tasks::read(v, &m.schedule)?;
        // The full cost of an owned hour at posted prices, δ·p_K/κ + O (ρ = 0): the price the
        // good desk's rules read where the wet desk reads the hour's market price.
        let (run, o) = running(v, &m.running)?;
        let ph = d * pk / kappa + o;
        let target = 1.0 - t.measure(w / ph);
        let s0 = v.own_state.share;
        let s = s0 + param(v, m.technique)? * (target - s0);
        let j = t.j(1.0 - s);
        let c = s * w + j * ph;
        let margin = Margin {
            cost: c,
            markup: p / c,
            worth: p * v.own.get(m.output),
        };
        let (outlay, scale) =
            scale_outlay(v, &m.scale, v.own_state.scale, &margin, &mut dry, &mut out)?;
        let q = outlay / c;
        let offered = offer(&mut dry, m.output, v.own.get(m.output))?;
        out.orders.push(sell(v, m.output, offered));
        // Its stock: K* = J·q/κ serves the plan; the order replaces the target's wear and
        // closes s_K of the gap after this tick's wear, at most what the coin left after hours
        // and running inputs buys.
        let hours = j * q;
        let k_star = hours / kappa;
        let held = v.own.get(m.stock);
        let gap = (1.0 - d) * k_star - held;
        let want = sign(d * k_star + param(v, m.adjust)? * gap);
        let coin = v.own.get(v.currency);
        let mut spent = w * (s * q);
        for &(_, coef, pg) in &run {
            spent += pg * (coef * hours);
        }
        let order = want.min(sign(coin - spent) / pk);
        let mut lines: Vec<(GoodId, f64)> = Vec::with_capacity(run.len() + 2);
        let mut wants = Vec::with_capacity(run.len() + 2);
        lines.push((m.labour, s * q));
        wants.push((m.labour, w * (s * q)));
        for &(g, coef, pg) in &run {
            let qty = coef * hours;
            lines.push((g, qty));
            wants.push((g, pg * qty));
        }
        lines.push((m.stock, order));
        wants.push((m.stock, pk * order));
        let budgets = budget_chain(coin, &wants, &mut dry, v.currency)?;
        for ((g, qty), bud) in lines.into_iter().zip(budgets) {
            out.orders.push(buy(v, g, qty, bud));
        }
        out.deltas.push(set(
            v,
            ActorState::Owner(OwnerState {
                share: s,
                scale,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn produce(&self, v: &View<'_, OwnerState>) -> Result<Vec<Delta>, AgentError> {
        if let Some(g) = &self.flow {
            let st = OwnerRole::good_state(v.own_state);
            let out = g.produce(&with(v, &st))?;
            return Ok(restate(out, OwnerRole::from_good(v.own_state)));
        }
        let m = &self.spec;
        let t = Tasks::read(v, &m.schedule)?;
        let planned = v.own_state.share;
        // Every unit it holds now serves, those bought this tick included (M1's convention);
        // its hours run while the running inputs held last.
        let serving = v.own.get(m.stock);
        let kappa = param(v, m.kappa)?;
        let (run, _) = running(v, &m.running)?;
        let held_inputs: Vec<(f64, f64)> = run
            .iter()
            .map(|&(g, coef, _)| (v.own.get(g), coef))
            .collect();
        let hours = hours_run(kappa * serving, &held_inputs)?;
        let l = v.own.get(m.labour);
        let (used, j) = match m.assign {
            Assign::Planned => (planned, t.j(1.0 - planned)),
            Assign::ExPost => assign_ex_post(&t, l, hours, planned),
        };
        let y = leontief(&[(l, used), (hours, j)])?;
        let me = Holder::Actor(v.me);
        let mut out = Vec::new();
        if y > 0.0 {
            burn(me, m.labour, used * y, Provenance::Production, &mut out);
            for &(g, coef, _) in &run {
                burn(me, g, coef * (j * y), Provenance::Production, &mut out);
            }
            out.push(StateDelta::Mint {
                to: me,
                good: m.output,
                qty: y,
                prov: Provenance::Production,
            });
        }
        out.push(set(
            v,
            ActorState::Owner(OwnerState {
                used,
                output: y,
                serving,
                ..*v.own_state
            }),
        ));
        Ok(out)
    }

    fn upkeep(&self, v: &View<'_, OwnerState>) -> Result<Vec<Delta>, AgentError> {
        if self.flow.is_some() {
            return Ok(Vec::new());
        }
        let m = &self.spec;
        let worn = (param(v, m.delta)? * v.own_state.serving).min(v.own.get(m.stock));
        let mut out = Vec::new();
        wear(Holder::Actor(v.me), m.stock, worn, &mut out);
        Ok(out)
    }
}
