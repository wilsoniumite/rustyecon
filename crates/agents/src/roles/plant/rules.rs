//! The plant's rules (P2.2b; docs/probe/LOOPS-RULES.md §3; LOOP-SPEC §2.2; `lm_mirror:117–186`),
//! and the three planted desks as the cast runs them.
//!
//! Notation: c the cost of one bundle at posted prices, summed in the kind's own order; C the
//! coin the desk held when decide ran; P the plant it held then; z the bundles it plans to run; I
//! the plant units it orders; u, s and s_Kp the plant's wear, size and adjustment a tick. Every
//! power is `num::pow` (libm), so both machines make the same bits. `sign(x)` is x where positive,
//! else 0: the sign of an order, never a cap (R3).
//!
//! **What a planted desk reads** (R13): its own coin, holdings, plant and state; the posted
//! prices; θ, δ_p, s, s_Kp and its kind's params. No volume, fill, other actor or oracle.
//!
//! **Its arithmetic cannot fail**, as the other roles': the plant's order is capped by the coin
//! left after the running bundle; the type desk's and the maker's budgets are cut from outlay +
//! P_K·I and the capacity desk's from its coin, each by the budget chain; produce reads the
//! bundles B over every unit held, splits them by the plan, and burns one line per good of
//! a_g·(used + s·I′), with the build I′ at most what B leaves after the running bundles, so no
//! burn overdraws; the wear is at most what is held.
//!
//! The kinds' own rules take the plant as an option (`TypeDesk::plan`, `CapacityDesk::plan`,
//! `MakerRole::plan_stock` and their produce and upkeep halves), so a desk without a plant runs
//! exactly its kind's code, and at θ = 1 (or u = 0) every plant branch reduces to it: kr = 0,
//! c_full = c, κ_p = 0, no order, z_got = B and y = z_got (LOOPS-RULES §3.9).

use crate::behaviour::{AgentError, Behaviour, Decision, View};
use crate::ext::{
    ActorState, PlantState, PlantedCapacityState, PlantedMakerState, PlantedTypeState,
};
use crate::roles::many::spec::TypeDesk;
use crate::roles::plant::spec::{Plant, PlantOrder, PlantTarget};
use crate::roles::rules::{burn, param, set, Delta};
use crate::roles::stock::rules::MakerRole;
use crate::roles::stock::spec::{CapacityDesk, Maker};
use rustyecon_core::num;
use rustyecon_core::{GoodId, Holder, Provenance, StateDelta};

/// `x` when it is positive, else 0: the sign of an order, never a cap (R3). A NaN is 0.
pub(crate) fn sign(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

/// A plant's dials at use, per tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlantNow {
    /// The plant good.
    pub(crate) good: GoodId,
    /// θ.
    pub(crate) theta: f64,
    /// u, the wear a tick.
    pub(crate) u: f64,
    /// s, bundles a plant unit.
    pub(crate) s: f64,
    /// s_p, the share of the gap closed a tick.
    pub(crate) sp: f64,
    /// How it is ordered.
    pub(crate) order: PlantOrder,
    /// Its target.
    pub(crate) target: PlantTarget,
}

/// The cost ratios at a bundle cost c (`ratios`, `lm_mirror:134–147`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Ratios {
    /// P_K = s·c, a plant unit's cost.
    pub(crate) pk: f64,
    /// kr, plant units per bundle at the cheapest plant; 0 where the formulas do not run.
    pub(crate) kr: f64,
    /// c_full = c + (u·P_K)·kr, a unit's cost with the plant's user cost; c where they do not.
    pub(crate) full: f64,
    /// ζ = kr^(−(1−θ)), bundles a unit made at that ratio; 1 where they do not.
    pub(crate) zeta: f64,
    /// κ_p = kr^θ, plant units a unit made at that ratio; 0 where they do not.
    pub(crate) kap: f64,
}

/// A desk's plan for its plant at decide: its record, less what produce builds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Plan {
    /// P, the plant held at decide.
    pub(crate) held: f64,
    /// K\*_p.
    pub(crate) target: f64,
    /// I.
    pub(crate) order: f64,
    /// z, the bundles planned.
    pub(crate) run: f64,
}

impl Plan {
    /// The plant's record after decide, keeping what the last produce built.
    pub(crate) fn record(self, last: &PlantState) -> PlantState {
        PlantState {
            held: self.held,
            target: self.target,
            order: self.order,
            run: self.run,
            built: last.built,
        }
    }
}

impl PlantNow {
    /// The plant's dials at their sites, as the rule reads them now.
    pub(crate) fn read<S>(v: &View<'_, S>, p: &Plant) -> Result<PlantNow, AgentError> {
        Ok(PlantNow {
            good: p.good,
            theta: param(v, p.theta)?,
            u: param(v, p.delta)?,
            s: param(v, p.size)?,
            sp: param(v, p.adjust)?,
            order: p.order,
            target: p.target,
        })
    }

    /// Whether the plant's cost formulas run: θ < 1 and u > 0. At θ = 1 (constant returns) or
    /// u = 0 (a fixed plant, fixed-Q drs) they are not evaluated (`lm_mirror:138–140`).
    pub(crate) fn costs(&self) -> bool {
        self.theta < 1.0 && self.u > 0.0
    }

    /// The cost ratios at the bundle cost `c`.
    pub(crate) fn ratios(&self, c: f64) -> Ratios {
        let pk = self.s * c;
        if !self.costs() {
            return Ratios {
                pk,
                kr: 0.0,
                full: c,
                zeta: 1.0,
                kap: 0.0,
            };
        }
        let th = self.theta;
        let kr = ((1.0 - th) * c) / ((th * self.u) * pk);
        Ratios {
            pk,
            kr,
            full: c + (self.u * pk) * kr,
            zeta: num::pow(kr, -(1.0 - th)),
            kap: num::pow(kr, th),
        }
    }

    /// The order at `target` with `held` plant units (`p_order`, `lm_mirror:159–166`): `Target`
    /// replaces the target's wear and closes s_Kp of the gap; `Gap` replaces the plant's own
    /// wear and closes the whole gap. Never negative (a sign).
    pub(crate) fn order_of(&self, target: f64, held: f64) -> f64 {
        match self.order {
            PlantOrder::Target => sign((self.u * target) + (self.sp * (target - held))),
            PlantOrder::Gap => sign((self.u * held) + (target - held)),
        }
    }

    /// What `z` bundles make on the plant `held`: P^(1−θ)·z^θ, 0 at z = 0 (`p_make`,
    /// `lm_mirror:169–171`); z itself at θ = 1, with no power.
    pub(crate) fn make(&self, held: f64, z: f64) -> f64 {
        if self.theta < 1.0 {
            if z > 0.0 {
                num::pow(held, 1.0 - self.theta) * num::pow(z, self.theta)
            } else {
                0.0
            }
        } else {
            z
        }
    }

    /// The bundles that make `y` on the plant `held` (`p_inverse`, `lm_mirror:174–181`): 0 at
    /// y = 0, (y/P^(1−θ))^(1/θ) otherwise, and `None` (no number of bundles, the mirror's +∞)
    /// for y > 0 on no plant; y itself at θ = 1.
    pub(crate) fn inverse(&self, held: f64, y: f64) -> Option<f64> {
        if self.theta >= 1.0 {
            return Some(y);
        }
        if y <= 0.0 {
            return Some(0.0);
        }
        if held <= 0.0 {
            return None;
        }
        Some(num::pow(
            y / num::pow(held, 1.0 - self.theta),
            1.0 / self.theta,
        ))
    }
}

/// The bundles held: min over the inputs `(held, coefficient)` whose coefficient is not zero of
/// `max_scale(held, coefficient)`, the kind's own Leontief read over every unit held; `None` when
/// no coefficient binds.
pub(crate) fn bundles_held(inputs: &[(f64, f64)]) -> Result<Option<f64>, AgentError> {
    let mut b: Option<f64> = None;
    for &(held, coef) in inputs {
        if coef > 0.0 {
            let x = num::max_scale(held, coef)?;
            b = Some(b.map_or(x, |z| z.min(x)));
        }
    }
    Ok(b)
}

/// The bundles held, `b`, split by the plan (z, I) recorded at decide: z_got =
/// min((B·z)/(z + s·I), B) run and I_got = (B·I)/(z + s·I) built; with no order all of B runs
/// and nothing is divided (LOOPS-RULES §3.6 step 2).
pub(crate) fn split(b: f64, z: f64, i: f64, s: f64) -> (f64, f64) {
    if i > 0.0 {
        let tot = z + s * i;
        (((b * z) / tot).min(b), (b * i) / tot)
    } else {
        (b, 0.0)
    }
}

/// The build I′ = min(I_got, max_scale(max_remainder(B, used), s)): so fl(used + fl(s·I′)) ≤ B,
/// and the running and the build bundles fit what is held (§3.6 step 5). 0 with no order.
pub(crate) fn built(b: f64, used: f64, i_got: f64, s: f64) -> Result<f64, AgentError> {
    if i_got > 0.0 {
        Ok(i_got.min(num::max_scale(num::max_remainder(b, used)?, s)?))
    } else {
        Ok(0.0)
    }
}

/// T = used + s·I′, the bundles one burn per good takes; `used` itself with no build, the
/// plant-free burn.
pub(crate) fn burned(used: f64, built: f64, s: f64) -> f64 {
    if built > 0.0 {
        used + s * built
    } else {
        used
    }
}

/// The mint of the plant units built, after the output, so they serve from the next tick.
pub(crate) fn mint_plant(me: Holder, p: &PlantNow, built: f64, out: &mut Vec<Delta>) {
    if built > 0.0 {
        out.push(StateDelta::Mint {
            to: me,
            good: p.good,
            qty: built,
            prov: Provenance::Production,
        });
    }
}

/// The plant's wear at upkeep: min(u·P, held), P the plant recorded at decide, as a
/// `Depreciation` burn; units built this tick do not wear this tick (§3.7).
pub(crate) fn wear_plant<S>(
    v: &View<'_, S>,
    p: &PlantNow,
    record: &PlantState,
    out: &mut Vec<Delta>,
) {
    let worn = (p.u * record.held).min(v.own.get(p.good));
    burn(
        Holder::Actor(v.me),
        p.good,
        worn,
        Provenance::Depreciation,
        out,
    );
}

/// A type desk with a plant (LOOPS-RULES §4.1): the fodder desk, and the flow control's
/// horse-day desk.
#[derive(Debug, Clone)]
pub struct PlantedType {
    desk: TypeDesk,
    plant: Plant,
}

impl PlantedType {
    /// The planted type desk of `desk`, whose plant is `plant`.
    pub fn new(desk: &TypeDesk, plant: &Plant) -> PlantedType {
        PlantedType {
            desk: desk.clone(),
            plant: *plant,
        }
    }
}

impl Behaviour for PlantedType {
    type Own = PlantedTypeState;

    fn decide(&self, v: &View<'_, PlantedTypeState>) -> Result<Decision, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut d, desk, plan) = self.desk.plan(v, &st.desk, Some(&p))?;
        let plant = plan.map_or(st.plant, |x| x.record(&st.plant));
        d.deltas.push(set(
            v,
            ActorState::PlantedType(PlantedTypeState { desk, plant }),
        ));
        Ok(d)
    }

    fn produce(&self, v: &View<'_, PlantedTypeState>) -> Result<Vec<Delta>, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut out, desk, built) = self.desk.run(v, &st.desk, Some((&p, &st.plant)))?;
        let plant = PlantState { built, ..st.plant };
        out.push(set(
            v,
            ActorState::PlantedType(PlantedTypeState { desk, plant }),
        ));
        Ok(out)
    }

    fn upkeep(&self, v: &View<'_, PlantedTypeState>) -> Result<Vec<Delta>, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let mut out = Vec::new();
        wear_plant(v, &p, &v.own_state.plant, &mut out);
        Ok(out)
    }
}

/// A capacity desk with a plant beside its stock (LOOPS-RULES §4.2): M3 wet, the plant's bundle
/// the running recipe (decision 261).
#[derive(Debug, Clone)]
pub struct PlantedCapacity {
    desk: CapacityDesk,
    plant: Plant,
}

impl PlantedCapacity {
    /// The planted capacity desk of `desk`, whose plant is `plant`.
    pub fn new(desk: &CapacityDesk, plant: &Plant) -> PlantedCapacity {
        PlantedCapacity {
            desk: desk.clone(),
            plant: *plant,
        }
    }
}

impl Behaviour for PlantedCapacity {
    type Own = PlantedCapacityState;

    fn decide(&self, v: &View<'_, PlantedCapacityState>) -> Result<Decision, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut d, desk, plan) = self.desk.plan(v, &st.desk, Some(&p))?;
        let plant = plan.map_or(st.plant, |x| x.record(&st.plant));
        d.deltas.push(set(
            v,
            ActorState::PlantedCapacity(PlantedCapacityState { desk, plant }),
        ));
        Ok(d)
    }

    fn produce(&self, v: &View<'_, PlantedCapacityState>) -> Result<Vec<Delta>, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut out, desk, built) = self.desk.run(v, &st.desk, Some((&p, &st.plant)))?;
        let plant = PlantState { built, ..st.plant };
        out.push(set(
            v,
            ActorState::PlantedCapacity(PlantedCapacityState { desk, plant }),
        ));
        Ok(out)
    }

    fn upkeep(&self, v: &View<'_, PlantedCapacityState>) -> Result<Vec<Delta>, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let mut out = self.desk.wear(v, &v.own_state.desk)?;
        wear_plant(v, &p, &v.own_state.plant, &mut out);
        Ok(out)
    }
}

/// A maker with a plant (LOOPS-RULES §4.3): M2 on the stock path, from bought inputs.
#[derive(Debug, Clone)]
pub struct PlantedMaker {
    role: MakerRole,
    plant: Plant,
}

impl PlantedMaker {
    /// The planted maker of `spec` (on the stock path), whose plant is `plant`.
    pub fn new(spec: &Maker, plant: &Plant) -> PlantedMaker {
        PlantedMaker {
            role: MakerRole::new(spec, false),
            plant: *plant,
        }
    }
}

impl Behaviour for PlantedMaker {
    type Own = PlantedMakerState;

    fn decide(&self, v: &View<'_, PlantedMakerState>) -> Result<Decision, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut d, desk, plan) = self.role.plan_stock(v, &st.desk, Some(&p))?;
        let plant = plan.map_or(st.plant, |x| x.record(&st.plant));
        d.deltas.push(set(
            v,
            ActorState::PlantedMaker(PlantedMakerState { desk, plant }),
        ));
        Ok(d)
    }

    fn produce(&self, v: &View<'_, PlantedMakerState>) -> Result<Vec<Delta>, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut out, desk, built) = self.role.run_stock(v, &st.desk, Some((&p, &st.plant)))?;
        let plant = PlantState { built, ..st.plant };
        out.push(set(
            v,
            ActorState::PlantedMaker(PlantedMakerState { desk, plant }),
        ));
        Ok(out)
    }

    fn upkeep(&self, v: &View<'_, PlantedMakerState>) -> Result<Vec<Delta>, AgentError> {
        let p = PlantNow::read(v, &self.plant)?;
        let st = v.own_state;
        let (mut out, desk) = self.role.wear_stock(v, &st.desk)?;
        wear_plant(v, &p, &st.plant, &mut out);
        out.push(set(
            v,
            ActorState::PlantedMaker(PlantedMakerState {
                desk,
                plant: st.plant,
            }),
        ));
        Ok(out)
    }
}
