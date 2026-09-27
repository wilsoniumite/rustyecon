//! The agents' side of core's extension seam (docs/ENGINE.md §2.3 and §4): the actors' specs,
//! their own state, their own deltas and the tape actions on them.
//!
//! Core knows goods, holdings, the book and params and nothing about behaviour (N14). [`Agents`]
//! supplies the rest. Its state is one [`ActorState`] per declared actor, hashed and checkpointed
//! with the rest of `SimState`. Phases 2 and 3 grow it by adding variants; core does not reopen.
//! The four Appendix B roles added theirs at P2.0, after `Scripted`, so the scripted state keeps
//! its encoding and every existing hash (docs/ENGINE.md, amended at P2.0). The four many-market
//! roles of P2.1 add none: each keeps its Appendix B role's state.

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

impl ActorState {
    /// The state's values, each with its name, for the checks `apply` and `validate` make.
    fn values(&self) -> Vec<(&'static str, f64)> {
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
        }
    }

    /// The values that are shares, which lie in [0, 1].
    fn shares(&self) -> Vec<(&'static str, f64)> {
        match self {
            ActorState::Workers(s) => vec![("share", s.share)],
            ActorState::GoodDesk(s) => vec![("share", s.share), ("used", s.used)],
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
        Spec::TypeDesk(d) => ActorState::MachDesk(MachDeskState {
            scale: d.scale.genesis(),
            output: 0.0,
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
