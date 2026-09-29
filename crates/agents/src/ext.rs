//! The agents' side of core's extension seam (docs/ENGINE.md §2.3 and §4): the actors' specs,
//! their own state, their own deltas and the tape actions on them.
//!
//! Core knows goods, holdings, the book and params and nothing about behaviour (N14). [`Agents`]
//! supplies the rest. Its state is one [`ActorState`] per declared actor, hashed and checkpointed
//! with the rest of `SimState`. Phases 2 and 3 grow it by adding variants; core does not reopen.
//! The four Appendix B roles added theirs at P2.0, after `Scripted`, so the scripted state keeps
//! its encoding and every existing hash (docs/ENGINE.md, amended at P2.0). The four many-market
//! roles of P2.1 add none: each keeps its Appendix B role's state. The three stock roles of P2.2
//! add three, `Maker`, `Capacity` and `Owner`, appended after `MachDesk` for the same reason. The
//! three planted desks of P2.2b add three more, `PlantedType`, `PlantedMaker` and
//! `PlantedCapacity`, appended after `Owner`: each nests its kind's state beside one
//! [`PlantState`] (decision 287), and a desk without a plant keeps its old variant.

use crate::spec::{self, RawSpec, Spec};
use rustyecon_core::num::is_clean;
use rustyecon_core::{ActorDecl, ActorId, CoreError, Ext, Key, LoadError, Resolver};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The extension marker for rustyecon's agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Agents;

/// One actor's own state. One variant per behaviour kind; it matches the actor's [`Spec`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActorState {
    /// A scripted actor's state.
    Scripted(ScriptState),
    /// The provider's state (P2.0).
    Provider(ProviderState),
    /// The workers' state (P2.0).
    Workers(WorkersState),
    /// A good desk's state (P2.0).
    GoodDesk(GoodDeskState),
    /// A machine desk's state (P2.0).
    MachDesk(MachDeskState),
    /// A maker's state (P2.2).
    Maker(MakerState),
    /// A capacity desk's state (P2.2).
    Capacity(CapacityState),
    /// An owner desk's state (P2.2).
    Owner(OwnerState),
    /// A planted type desk's state (P2.2b).
    PlantedType(PlantedTypeState),
    /// A planted maker's state (P2.2b).
    PlantedMaker(PlantedMakerState),
    /// A planted capacity desk's state (P2.2b).
    PlantedCapacity(PlantedCapacityState),
}

/// A scripted actor's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptState {
    /// Whether it acts. A dormant actor emits nothing and posts nothing.
    pub active: bool,
}

/// The provider's record of its last transfer (R12: a transfer it could not pay in full rations
/// the workers, and no market line records that). Both 0 before the first tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProviderState {
    /// What the transfer owed: N·P_s at the posted prices.
    pub due: f64,
    /// What it paid: the due, or all its coin if that was less.
    pub paid: f64,
}

/// The workers' record of their last offer: the participation share s, so N·s hours were
/// offered. 0 before the first tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkersState {
    /// The share of potential hours offered, in [0, 1].
    pub share: f64,
}

/// A good desk's state. `share` is its technique; the rest are records, except a step rule's
/// `scale`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GoodDeskState {
    /// The planned human share 1 − x, in [0, 1]: the technique, carried as 1 − x so that the
    /// approach to the x = 1 corner never cancels (a judges' graft, docs/probe/RULES.md).
    pub share: f64,
    /// The human share its last production used, in [0, 1]: `share` under `Assign::Planned`,
    /// the cutoff that used up both inputs under `Assign::ExPost`. The genesis share before the
    /// first tick.
    pub used: f64,
    /// The scale: a step rule's state (the output per tick it plans for), or the cash rule's
    /// last planned output, a record (0 before the first tick).
    pub scale: f64,
    /// What its last production made. 0 before the first tick.
    pub output: f64,
}

/// A machine desk's state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MachDeskState {
    /// The scale, as for the good desk.
    pub scale: f64,
    /// What its last production made. 0 before the first tick.
    pub output: f64,
}

/// A maker's state (P2.2, HORSES-SPEC §2.6): the cash rule's record, its last output, and its
/// serving stock, which is part of what it holds of its output; the rest is its finished stock.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MakerState {
    /// The scale, as for the machine desk.
    pub scale: f64,
    /// What its last production made. 0 before the first tick.
    pub output: f64,
    /// Its serving stock after the last tick's wear, which serves again from the next decide:
    /// its record, since a holding does not say which units serve. The genesis value is the
    /// tape's.
    pub own: f64,
    /// Its serving stock this tick, `own` and the finished units it moved into service, whose
    /// hours it builds with and which wear at upkeep. 0 before the first tick.
    pub serving: f64,
}

/// A capacity desk's state (P2.2, HORSES-SPEC §2.7): the cash rule's record, its last output,
/// and its records of this tick's decision.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CapacityState {
    /// The scale: the cash rule's outlay over the full cost of an hour, a record.
    pub scale: f64,
    /// The hours its last production made. 0 before the first tick.
    pub output: f64,
    /// The stock it held when this tick's decide ran: the machines that make hours and wear
    /// this tick. 0 before the first tick.
    pub held: f64,
    /// K\*, the stock the cash rule at the full cost would hold. 0 before the first tick.
    pub target: f64,
    /// The units of stock it ordered. 0 before the first tick.
    pub order: f64,
    /// The hours it planned to run, the cash rule at the running cost. 0 before the first tick.
    pub run: f64,
}

/// An owner desk's state (P2.2, HORSES-SPEC §2.8): the good desk's, and its serving stock.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OwnerState {
    /// The planned human share 1 − x, in [0, 1], as the good desk's.
    pub share: f64,
    /// The human share its last production used, in [0, 1], as the good desk's.
    pub used: f64,
    /// The scale, as the good desk's.
    pub scale: f64,
    /// What its last production made. 0 before the first tick.
    pub output: f64,
    /// The stock that served in its last production, and wears at upkeep. 0 before the first
    /// tick, and always under the flow path.
    pub serving: f64,
}

/// A plant's record (P2.2b; LOOPS-RULES §3.8): what its desk decided about it at decide, and what
/// it built at produce. All 0 before the first tick.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct PlantState {
    /// P, the plant its desk held when this tick's decide ran: the units that serve this tick and
    /// wear at upkeep.
    pub held: f64,
    /// K\*_p, the plant its rule targets.
    pub target: f64,
    /// I, the plant units it ordered as bundles of its own recipe.
    pub order: f64,
    /// z, the bundles it planned to run: the type desk's and the maker's plan q, the capacity
    /// desk's z (its desk record's `run`).
    pub run: f64,
    /// I′, the plant units its last produce built, which serve from the next tick.
    pub built: f64,
}

impl PlantState {
    fn values(&self) -> [(&'static str, f64); 5] {
        [
            ("plant.held", self.held),
            ("plant.target", self.target),
            ("plant.order", self.order),
            ("plant.run", self.run),
            ("plant.built", self.built),
        ]
    }
}

/// A planted type desk's state (P2.2b): the type desk's, and its plant's.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlantedTypeState {
    /// The type desk's state.
    pub desk: MachDeskState,
    /// The plant's record.
    pub plant: PlantState,
}

/// A planted maker's state (P2.2b): the maker's, and its plant's.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlantedMakerState {
    /// The maker's state.
    pub desk: MakerState,
    /// The plant's record.
    pub plant: PlantState,
}

/// A planted capacity desk's state (P2.2b): the capacity desk's, and its plant's.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlantedCapacityState {
    /// The capacity desk's state.
    pub desk: CapacityState,
    /// The plant's record.
    pub plant: PlantState,
}

impl ActorState {
    /// The state's values, each with its name, for the checks `apply` and `validate` make.
    fn values(&self) -> Vec<(&'static str, f64)> {
        let planted = |desk: ActorState, plant: &PlantState| {
            let mut v = desk.values();
            v.extend(plant.values());
            v
        };
        match self {
            ActorState::Scripted(_) => Vec::new(),
            ActorState::Provider(s) => vec![("due", s.due), ("paid", s.paid)],
            ActorState::Workers(s) => vec![("share", s.share)],
            ActorState::GoodDesk(s) => vec![
                ("share", s.share),
                ("used", s.used),
                ("scale", s.scale),
                ("output", s.output),
            ],
            ActorState::MachDesk(s) => vec![("scale", s.scale), ("output", s.output)],
            ActorState::Maker(s) => vec![
                ("scale", s.scale),
                ("output", s.output),
                ("own", s.own),
                ("serving", s.serving),
            ],
            ActorState::Capacity(s) => vec![
                ("scale", s.scale),
                ("output", s.output),
                ("held", s.held),
                ("target", s.target),
                ("order", s.order),
                ("run", s.run),
            ],
            ActorState::Owner(s) => vec![
                ("share", s.share),
                ("used", s.used),
                ("scale", s.scale),
                ("output", s.output),
                ("serving", s.serving),
            ],
            ActorState::PlantedType(s) => planted(ActorState::MachDesk(s.desk), &s.plant),
            ActorState::PlantedMaker(s) => planted(ActorState::Maker(s.desk), &s.plant),
            ActorState::PlantedCapacity(s) => planted(ActorState::Capacity(s.desk), &s.plant),
        }
    }

    /// The values that are shares, which lie in [0, 1].
    fn shares(&self) -> Vec<(&'static str, f64)> {
        match self {
            ActorState::Workers(s) => vec![("share", s.share)],
            ActorState::GoodDesk(s) => vec![("share", s.share), ("used", s.used)],
            ActorState::Owner(s) => vec![("share", s.share), ("used", s.used)],
            _ => Vec::new(),
        }
    }

    /// Whether two states are of one kind.
    fn same_kind(&self, other: &ActorState) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Every value finite with a clear sign bit (so not `-0.0`: the state is hashed, R8), and
    /// every share at most 1.
    fn check(&self) -> Result<(), CoreError> {
        for (what, value) in self.values() {
            if !is_clean(value) {
                return Err(CoreError::BadValue { what, value });
            }
        }
        for (what, value) in self.shares() {
            if value > 1.0 {
                return Err(CoreError::BadValue { what, value });
            }
        }
        Ok(())
    }
}

/// The agents' own deltas, applied through core's `Actor` arm.
///
/// `Eq` was dropped at P2.0, when `SetState` brought `f64` fields (PROBE-SPEC G10).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentDelta {
    /// Wake or put to sleep a scripted actor. Owned by that actor; the tape emits it (the entrant
    /// pattern: a dormant desk and a dated event).
    SetActive {
        /// The actor.
        actor: ActorId,
        /// Its new state.
        active: bool,
    },
    /// Replace an actor's own state with one of the same kind (P2.0). Owned by that actor. Every
    /// value must be finite with a clear sign bit and every share at most 1, or `apply` refuses
    /// it and nothing changes.
    SetState {
        /// The actor.
        actor: ActorId,
        /// Its new state.
        state: ActorState,
    },
}

/// The agents' tape actions, as the tape writes them: `Actor(SetActive(actor: "oven", active:
/// true))`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawAgentAction {
    /// Wake or put to sleep a scripted actor.
    SetActive {
        /// Actor key, a scripted actor. Required, no default.
        actor: Key,
        /// `bool`. Required, no default.
        active: bool,
    },
}

/// An actor's genesis state, from its spec.
pub(crate) fn genesis_state(spec: &Spec) -> ActorState {
    match spec {
        Spec::Scripted(s) => ActorState::Scripted(ScriptState { active: s.active }),
        Spec::Provider(_) => ActorState::Provider(ProviderState {
            due: 0.0,
            paid: 0.0,
        }),
        Spec::Workers(_) => ActorState::Workers(WorkersState { share: 0.0 }),
        Spec::GoodDesk(d) => ActorState::GoodDesk(GoodDeskState {
            share: d.share,
            used: d.share,
            scale: d.scale.genesis(),
            output: 0.0,
        }),
        Spec::MachDesk(d) => ActorState::MachDesk(MachDeskState {
            scale: d.scale.genesis(),
            output: 0.0,
        }),
        // The many-market roles (P2.1) keep their Appendix B role's state.
        Spec::BasketProvider(_) => ActorState::Provider(ProviderState {
            due: 0.0,
            paid: 0.0,
        }),
        Spec::BasketWorkers(_) => ActorState::Workers(WorkersState { share: 0.0 }),
        Spec::CategoryDesk(d) => ActorState::GoodDesk(GoodDeskState {
            share: d.share,
            used: d.share,
            scale: d.scale.genesis(),
            output: 0.0,
        }),
        // A planted desk (P2.2b) has the planted variant exactly when its spec has a plant.
        Spec::TypeDesk(d) => {
            let desk = MachDeskState {
                scale: d.scale.genesis(),
                output: 0.0,
            };
            match d.plant {
                None => ActorState::MachDesk(desk),
                Some(_) => ActorState::PlantedType(PlantedTypeState {
                    desk,
                    plant: PlantState::default(),
                }),
            }
        }
        // The stock roles (P2.2) have states of their own, appended after `MachDesk`, so every
        // existing encoding and hash stays.
        Spec::Maker(d) => {
            let desk = MakerState {
                scale: d.scale.genesis(),
                output: 0.0,
                own: d.own,
                serving: 0.0,
            };
            match d.plant {
                None => ActorState::Maker(desk),
                Some(_) => ActorState::PlantedMaker(PlantedMakerState {
                    desk,
                    plant: PlantState::default(),
                }),
            }
        }
        Spec::CapacityDesk(d) => {
            let desk = CapacityState {
                scale: d.scale.genesis(),
                output: 0.0,
                held: 0.0,
                target: 0.0,
                order: 0.0,
                run: 0.0,
            };
            match d.plant {
                None => ActorState::Capacity(desk),
                Some(_) => ActorState::PlantedCapacity(PlantedCapacityState {
                    desk,
                    plant: PlantState::default(),
                }),
            }
        }
        Spec::OwnerDesk(d) => ActorState::Owner(OwnerState {
            share: d.share,
            used: d.share,
            scale: d.scale.genesis(),
            output: 0.0,
            serving: 0.0,
        }),
    }
}

impl Ext for Agents {
    type State = BTreeMap<ActorId, ActorState>;
    type Delta = AgentDelta;
    type RawActor = RawSpec;
    type Actor = Spec;
    type RawAction = RawAgentAction;

    fn resolve_actor(raw: &RawSpec, r: &mut Resolver<'_>) -> Result<Spec, LoadError> {
        spec::resolve(raw, r)
    }

    fn resolve_action(raw: &RawAgentAction, r: &mut Resolver<'_>) -> Result<AgentDelta, LoadError> {
        match raw {
            RawAgentAction::SetActive { actor, active } => Ok(AgentDelta::SetActive {
                actor: r.actor(actor, "actor")?,
                active: *active,
            }),
        }
    }

    fn genesis(actors: &[ActorDecl<Spec>]) -> Result<Self::State, LoadError> {
        Ok(actors
            .iter()
            .map(|a| (a.id, genesis_state(&a.spec)))
            .collect())
    }

    fn apply(s: &mut Self::State, d: &AgentDelta) -> Result<(), CoreError> {
        match *d {
            AgentDelta::SetActive { actor, active } => match s.get_mut(&actor) {
                Some(ActorState::Scripted(st)) => {
                    st.active = active;
                    Ok(())
                }
                Some(_) => Err(CoreError::Ext(format!(
                    "{actor} is not a scripted actor, and SetActive wakes only those"
                ))),
                None => Err(CoreError::UnknownActor(actor)),
            },
            AgentDelta::SetState { actor, state } => match s.get_mut(&actor) {
                Some(st) if st.same_kind(&state) => {
                    state.check()?;
                    *st = state;
                    Ok(())
                }
                Some(_) => Err(CoreError::Ext(format!(
                    "a SetState for {actor} is not of its kind"
                ))),
                None => Err(CoreError::UnknownActor(actor)),
            },
        }
    }

    fn owner(d: &AgentDelta) -> Option<ActorId> {
        match *d {
            AgentDelta::SetActive { actor, .. } | AgentDelta::SetState { actor, .. } => Some(actor),
        }
    }

    fn validate(s: &Self::State, actors: &[ActorDecl<Spec>]) -> Result<(), CoreError> {
        if s.len() != actors.len() {
            return Err(CoreError::Shape(format!(
                "{} actor states for {} declared actors",
                s.len(),
                actors.len()
            )));
        }
        for a in actors {
            match s.get(&a.id) {
                Some(st) if st.same_kind(&genesis_state(&a.spec)) => st.check()?,
                Some(_) => {
                    return Err(CoreError::Shape(format!(
                        "declared actor {}'s state is not of its spec's kind",
                        a.key
                    )))
                }
                None => {
                    return Err(CoreError::Shape(format!(
                        "declared actor {} has no state",
                        a.key
                    )))
                }
            }
        }
        Ok(())
    }

    fn canonical_actor(raw: &mut RawSpec) {
        spec::canonical(raw);
    }

    fn canonical_action(_: &mut RawAgentAction) {}
}
