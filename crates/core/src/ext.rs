//! The extension seam (docs/ENGINE.md §2.3, N14).
//!
//! Core knows goods, holdings, the book, params and the tape, and nothing about behaviour. An
//! [`Ext`] supplies the rest: the actors' specs on the tape and resolved, their own state (which
//! is hashed and checkpointed with the rest), their own deltas, and tape actions on them. The
//! agents crate implements it; Phases 2 and 3 grow it without reopening core.

use crate::error::{CoreError, LoadError};
use crate::ids::ActorId;
use crate::tape::Resolver;
use crate::world::ActorDecl;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An extension: the behaviour side of a world.
///
/// Implementors are unit marker types; the supertraits let the standard derives work on the
/// generic types that carry one (`StateDelta<E>`, `SimState<E>`, `Tape<E>` and the rest).
pub trait Ext: Clone + Debug + PartialEq + Send + Sync + 'static {
    /// The extension's state. Hashed and checkpointed with the rest of `SimState`.
    type State: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync + 'static;
    /// The extension's deltas, applied by [`Ext::apply`] through the `Actor` arm.
    type Delta: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync + 'static;
    /// An actor's spec as the tape writes it.
    type RawActor: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync + 'static;
    /// An actor's spec resolved to ids. Serialised into the world's identity.
    type Actor: Clone + Debug + Serialize + Send + Sync + 'static;
    /// A tape action on actors, as the tape writes it.
    type RawAction: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Resolve an actor's spec. Keys go through `r`, which records every param reference with
    /// its unit and use and names the tape path in errors.
    fn resolve_actor(raw: &Self::RawActor, r: &mut Resolver<'_>) -> Result<Self::Actor, LoadError>;
    /// Resolve a tape action into one extension delta.
    fn resolve_action(
        raw: &Self::RawAction,
        r: &mut Resolver<'_>,
    ) -> Result<Self::Delta, LoadError>;
    /// The extension state at genesis.
    fn genesis(actors: &[ActorDecl<Self::Actor>]) -> Result<Self::State, LoadError>;
    /// Apply one delta, all or nothing: an error leaves `s` unchanged.
    fn apply(s: &mut Self::State, d: &Self::Delta) -> Result<(), CoreError>;
    /// The actor a delta belongs to. `None` marks a world-level delta, legal only from the tape.
    fn owner(d: &Self::Delta) -> Option<ActorId>;
    /// Check a loaded state (from a checkpoint) against the declared actors.
    fn validate(s: &Self::State, actors: &[ActorDecl<Self::Actor>]) -> Result<(), CoreError>;
    /// Put a raw spec's own lists into canonical order, for `Tape::to_ron`.
    fn canonical_actor(raw: &mut Self::RawActor);
    /// Put a raw action's own lists into canonical order, for `Tape::to_ron`.
    fn canonical_action(raw: &mut Self::RawAction);
}

/// The empty extension, for core's own tests and for worlds with no behaviour: actors hold
/// goods and have the spec `()`, and no extension delta or tape action exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoExt;

/// A type with no values: `NoExt`'s deltas and tape actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Never {}

impl Ext for NoExt {
    type State = ();
    type Delta = Never;
    type RawActor = ();
    type Actor = ();
    type RawAction = Never;

    fn resolve_actor(_: &(), _: &mut Resolver<'_>) -> Result<(), LoadError> {
        Ok(())
    }
    fn resolve_action(raw: &Never, _: &mut Resolver<'_>) -> Result<Never, LoadError> {
        match *raw {}
    }
    fn genesis(_: &[ActorDecl<()>]) -> Result<(), LoadError> {
        Ok(())
    }
    fn apply(_: &mut (), d: &Never) -> Result<(), CoreError> {
        match *d {}
    }
    fn owner(d: &Never) -> Option<ActorId> {
        match *d {}
    }
    fn validate(_: &(), _: &[ActorDecl<()>]) -> Result<(), CoreError> {
        Ok(())
    }
    fn canonical_actor(_: &mut ()) {}
    fn canonical_action(raw: &mut Never) {
        match *raw {}
    }
}
