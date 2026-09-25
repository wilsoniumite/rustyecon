//! The agents' side of core's extension seam (docs/ENGINE.md §2.3 and §4): the actors' specs,
//! their own state, their own deltas and the tape actions on them.
//!
//! Core knows goods, holdings, the book and params and nothing about behaviour (N14). [`Agents`]
//! supplies the rest. Its state is one [`ActorState`] per declared actor, hashed and checkpointed
//! with the rest of `SimState`. Phases 2 and 3 grow it by adding variants; core does not reopen.

use crate::spec::{self, RawSpec, Spec};
use rustyecon_core::{ActorDecl, ActorId, CoreError, Ext, Key, LoadError, Resolver};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The extension marker for rustyecon's agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Agents;

/// One actor's own state. One variant per behaviour kind; it matches the actor's [`Spec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActorState {
    /// A scripted actor's state.
    Scripted(ScriptState),
}

/// A scripted actor's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptState {
    /// Whether it acts. A dormant actor emits nothing and posts nothing.
    pub active: bool,
}

/// The agents' own deltas, applied through core's `Actor` arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentDelta {
    /// Wake or put to sleep a scripted actor. Owned by that actor; the tape emits it (the entrant
    /// pattern: a dormant desk and a dated event).
    SetActive {
        /// The actor.
        actor: ActorId,
        /// Its new state.
        active: bool,
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
            .map(|a| {
                let state = match &a.spec {
                    Spec::Scripted(s) => ActorState::Scripted(ScriptState { active: s.active }),
                };
                (a.id, state)
            })
            .collect())
    }

    fn apply(s: &mut Self::State, d: &AgentDelta) -> Result<(), CoreError> {
        match *d {
            AgentDelta::SetActive { actor, active } => match s.get_mut(&actor) {
                Some(ActorState::Scripted(st)) => {
                    st.active = active;
                    Ok(())
                }
                None => Err(CoreError::UnknownActor(actor)),
            },
        }
    }

    fn owner(d: &AgentDelta) -> Option<ActorId> {
        match *d {
            AgentDelta::SetActive { actor, .. } => Some(actor),
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
            match (s.get(&a.id), &a.spec) {
                (Some(ActorState::Scripted(_)), Spec::Scripted(_)) => {}
                (None, _) => {
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
