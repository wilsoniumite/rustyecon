//! The cast: one behaviour per declared actor, dispatched by kind (docs/ENGINE.md §4).
//!
//! `Cast` is built once from the world's declarations and never changes; an actor's mutable
//! state lives in `SimState` and reaches its hooks through a [`View`]. The cast also makes the
//! load checks that need the whole world rather than one spec: a scripted actor budgets in its
//! home currency, so it buys only at nodes that quote in it, and it never pays itself.

use crate::behaviour::{AgentError, Behaviour, Decision, Posted, View};
use crate::ext::{ActorState, Agents};
use crate::spec::{Script, Spec};
use rustyecon_core::{
    ActorDecl, ActorId, Holder, Key, LoadError, LoadErrorKind, SimState, StateDelta, World,
};

/// One actor's behaviour.
#[derive(Debug, Clone)]
enum Member {
    Scripted(Script),
}

/// Every declared actor's behaviour, in `ActorId` order.
#[derive(Debug, Clone)]
pub struct Cast {
    members: Vec<(ActorId, Member)>,
}

fn key_of(w: &World<Agents>, k: impl rustyecon_core::Keyed) -> String {
    w.key_of(k).map_or_else(|| "?".to_string(), Key::to_string)
}

fn check_scripted(w: &World<Agents>, decl: &ActorDecl<Spec>, s: &Script) -> Result<(), LoadError> {
    let at = |field: String, why: &str| {
        LoadError::new(
            format!("actors[{}].spec.{field}", decl.key),
            LoadErrorKind::Invalid(why.to_string()),
        )
    };
    let home = w
        .node(decl.home)
        .ok_or_else(|| at("home".into(), "the home is not a node"))?
        .currency;
    for b in &s.buy {
        let currency = w.node(b.node).map(|n| n.currency);
        if currency != Some(home) {
            return Err(at(
                format!("buy[{}/{}].node", key_of(w, b.node), key_of(w, b.good)),
                "a scripted actor buys only at nodes that quote in its home currency",
            ));
        }
    }
    if let Some(p) = &s.payout {
        if p.to.iter().any(|(a, _)| *a == decl.id) {
            return Err(at(
                format!("payout.to[{}]", decl.key),
                "an actor does not pay itself",
            ));
        }
    }
    Ok(())
}

impl Cast {
    /// The cast of a world, after the checks that need the whole world.
    pub fn new(w: &World<Agents>) -> Result<Cast, LoadError> {
        let mut members = Vec::with_capacity(w.actors.len());
        for decl in &w.actors {
            let member = match &decl.spec {
                Spec::Scripted(s) => {
                    check_scripted(w, decl, s)?;
                    Member::Scripted(s.clone())
                }
            };
            members.push((decl.id, member));
        }
        members.sort_by_key(|(a, _)| *a);
        Ok(Cast { members })
    }

    /// Every actor, in `ActorId` order.
    pub fn actors(&self) -> impl Iterator<Item = ActorId> + '_ {
        self.members.iter().map(|(a, _)| *a)
    }

    fn member(&self, a: ActorId) -> Result<&Member, AgentError> {
        self.members
            .binary_search_by_key(&a, |(id, _)| *id)
            .map(|i| &self.members[i].1)
            .map_err(|_| AgentError::Mismatch(a))
    }

    /// Run `a`'s `decide` on `s`.
    pub fn decide(
        &self,
        a: ActorId,
        s: &SimState<Agents>,
        w: &World<Agents>,
    ) -> Result<Decision, AgentError> {
        match self.member(a)? {
            Member::Scripted(script) => script.decide(&scripted_view(a, s, w)?),
        }
    }

    /// Run `a`'s `produce` on `s`.
    pub fn produce(
        &self,
        a: ActorId,
        s: &SimState<Agents>,
        w: &World<Agents>,
    ) -> Result<Vec<StateDelta<Agents>>, AgentError> {
        match self.member(a)? {
            Member::Scripted(script) => script.produce(&scripted_view(a, s, w)?),
        }
    }

    /// Run `a`'s `upkeep` on `s`.
    pub fn upkeep(
        &self,
        a: ActorId,
        s: &SimState<Agents>,
        w: &World<Agents>,
    ) -> Result<Vec<StateDelta<Agents>>, AgentError> {
        match self.member(a)? {
            Member::Scripted(script) => script.upkeep(&scripted_view(a, s, w)?),
        }
    }
}

/// The view of a scripted actor: its own holding and state, the posted book, the params.
fn scripted_view<'a>(
    a: ActorId,
    s: &'a SimState<Agents>,
    w: &'a World<Agents>,
) -> Result<View<'a, crate::ext::ScriptState>, AgentError> {
    let decl = w.actor(a).ok_or(AgentError::Mismatch(a))?;
    let own_state = match s.ext().get(&a) {
        Some(ActorState::Scripted(st)) => st,
        None => return Err(AgentError::Mismatch(a)),
    };
    view(decl, s, w, own_state)
}

/// Build the view of the actor `decl` over `s`.
pub fn view<'a, S>(
    decl: &ActorDecl<Spec>,
    s: &'a SimState<Agents>,
    w: &'a World<Agents>,
    own_state: &'a S,
) -> Result<View<'a, S>, AgentError> {
    let own = s.holding(Holder::Actor(decl.id)).ok_or(AgentError::Core(
        rustyecon_core::CoreError::UnknownHolder(Holder::Actor(decl.id)),
    ))?;
    let currency = w
        .node(decl.home)
        .ok_or(AgentError::Core(rustyecon_core::CoreError::UnknownNode(
            decl.home,
        )))?
        .currency;
    Ok(View {
        tick: s.tick(),
        clock: &w.clock,
        me: decl.id,
        class: decl.class,
        home: decl.home,
        currency,
        own,
        own_state,
        posted: Posted::new(s.book()),
        params: s.params(&w.registry),
    })
}
