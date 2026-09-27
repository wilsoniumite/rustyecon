//! The cast: one behaviour per declared actor, dispatched by kind (docs/ENGINE.md §4).
//!
//! `Cast` is built once from the world's declarations and never changes; an actor's mutable
//! state lives in `SimState` and reaches its hooks through a [`View`]. The cast also makes the
//! load checks that need the whole world rather than one spec: a scripted actor budgets in its
//! home currency, so it buys only at nodes that quote in it, and it never pays itself. An
//! Appendix B role (P2.0), or a many-market role (P2.1), trades at its home node only; a desk role must be a Desk and a
//! household role a Pop; what it is endowed with must be `Instant`, since `decide` may mint only
//! that; and it never transfers to or pays itself.

use crate::behaviour::{AgentError, Behaviour, Decision, Posted, View};
use crate::ext::{
    ActorState, Agents, GoodDeskState, MachDeskState, ProviderState, ScriptState, WorkersState,
};
use crate::roles::many::spec::{BasketProvider, BasketWorkers, CategoryDesk, TypeDesk};
use crate::roles::spec::{GoodDesk, MachDesk, Provider, Workers};
use crate::spec::{Script, Spec};
use rustyecon_core::{
    ActorDecl, ActorId, ActorKind, GoodId, Holder, Key, Life, LoadError, LoadErrorKind, SimState,
    StateDelta, World,
};

/// One actor's behaviour.
#[derive(Debug, Clone)]
enum Member {
    Scripted(Script),
    Provider(Provider),
    Workers(Workers),
    GoodDesk(GoodDesk),
    MachDesk(MachDesk),
    BasketProvider(BasketProvider),
    BasketWorkers(BasketWorkers),
    CategoryDesk(CategoryDesk),
    TypeDesk(TypeDesk),
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

/// The checks on an Appendix B role that need the whole world: its kind, what it is endowed
/// with, and whom it pays.
fn check_role(
    w: &World<Agents>,
    decl: &ActorDecl<Spec>,
    kind: ActorKind,
    endowed: Option<(GoodId, &str)>,
    pays: Option<(ActorId, &str)>,
) -> Result<(), LoadError> {
    let at = |field: &str, why: &str| {
        let path = if field.is_empty() {
            format!("actors[{}].spec", decl.key)
        } else {
            format!("actors[{}].spec.{field}", decl.key)
        };
        LoadError::new(path, LoadErrorKind::Invalid(why.to_string()))
    };
    if decl.id.kind() != kind {
        return Err(at(
            "",
            match kind {
                ActorKind::Desk => "a desk role must be declared a Desk",
                ActorKind::Pop => "a household role must be declared a Pop",
            },
        ));
    }
    if let Some((g, field)) = endowed {
        if w.good(g).map(|d| d.life) != Some(Life::Instant) {
            return Err(at(
                field,
                "an endowment minted in decide must be an Instant good",
            ));
        }
    }
    if let Some((to, field)) = pays {
        if to == decl.id {
            return Err(at(field, "an actor does not pay itself"));
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
                Spec::Provider(p) => {
                    let pays = Some((p.transfer_to, "transfer.to"));
                    check_role(w, decl, ActorKind::Pop, Some((p.land, "land")), pays)?;
                    Member::Provider(p.clone())
                }
                Spec::Workers(p) => {
                    check_role(w, decl, ActorKind::Pop, Some((p.labour, "labour")), None)?;
                    Member::Workers(p.clone())
                }
                Spec::GoodDesk(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    Member::GoodDesk(d.clone())
                }
                Spec::MachDesk(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    Member::MachDesk(d.clone())
                }
                Spec::BasketProvider(p) => {
                    let pays = Some((p.transfer_to, "transfer.to"));
                    check_role(w, decl, ActorKind::Pop, Some((p.land, "land")), pays)?;
                    Member::BasketProvider(p.clone())
                }
                Spec::BasketWorkers(p) => {
                    check_role(w, decl, ActorKind::Pop, Some((p.labour, "labour")), None)?;
                    Member::BasketWorkers(p.clone())
                }
                Spec::CategoryDesk(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    Member::CategoryDesk(d.clone())
                }
                Spec::TypeDesk(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    Member::TypeDesk(d.clone())
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
            Member::Scripted(b) => b.decide(&role_view(a, s, w, scripted)?),
            Member::Provider(b) => b.decide(&role_view(a, s, w, provider)?),
            Member::Workers(b) => b.decide(&role_view(a, s, w, workers)?),
            Member::GoodDesk(b) => b.decide(&role_view(a, s, w, good_desk)?),
            Member::MachDesk(b) => b.decide(&role_view(a, s, w, mach_desk)?),
            Member::BasketProvider(b) => b.decide(&role_view(a, s, w, provider)?),
            Member::BasketWorkers(b) => b.decide(&role_view(a, s, w, workers)?),
            Member::CategoryDesk(b) => b.decide(&role_view(a, s, w, good_desk)?),
            Member::TypeDesk(b) => b.decide(&role_view(a, s, w, mach_desk)?),
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
            Member::Scripted(b) => b.produce(&role_view(a, s, w, scripted)?),
            Member::Provider(b) => b.produce(&role_view(a, s, w, provider)?),
            Member::Workers(b) => b.produce(&role_view(a, s, w, workers)?),
            Member::GoodDesk(b) => b.produce(&role_view(a, s, w, good_desk)?),
            Member::MachDesk(b) => b.produce(&role_view(a, s, w, mach_desk)?),
            Member::BasketProvider(b) => b.produce(&role_view(a, s, w, provider)?),
            Member::BasketWorkers(b) => b.produce(&role_view(a, s, w, workers)?),
            Member::CategoryDesk(b) => b.produce(&role_view(a, s, w, good_desk)?),
            Member::TypeDesk(b) => b.produce(&role_view(a, s, w, mach_desk)?),
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
            Member::Scripted(b) => b.upkeep(&role_view(a, s, w, scripted)?),
            Member::Provider(b) => b.upkeep(&role_view(a, s, w, provider)?),
            Member::Workers(b) => b.upkeep(&role_view(a, s, w, workers)?),
            Member::GoodDesk(b) => b.upkeep(&role_view(a, s, w, good_desk)?),
            Member::MachDesk(b) => b.upkeep(&role_view(a, s, w, mach_desk)?),
            Member::BasketProvider(b) => b.upkeep(&role_view(a, s, w, provider)?),
            Member::BasketWorkers(b) => b.upkeep(&role_view(a, s, w, workers)?),
            Member::CategoryDesk(b) => b.upkeep(&role_view(a, s, w, good_desk)?),
            Member::TypeDesk(b) => b.upkeep(&role_view(a, s, w, mach_desk)?),
        }
    }
}

fn scripted(s: &ActorState) -> Option<&ScriptState> {
    match s {
        ActorState::Scripted(st) => Some(st),
        _ => None,
    }
}

fn provider(s: &ActorState) -> Option<&ProviderState> {
    match s {
        ActorState::Provider(st) => Some(st),
        _ => None,
    }
}

fn workers(s: &ActorState) -> Option<&WorkersState> {
    match s {
        ActorState::Workers(st) => Some(st),
        _ => None,
    }
}

fn good_desk(s: &ActorState) -> Option<&GoodDeskState> {
    match s {
        ActorState::GoodDesk(st) => Some(st),
        _ => None,
    }
}

fn mach_desk(s: &ActorState) -> Option<&MachDeskState> {
    match s {
        ActorState::MachDesk(st) => Some(st),
        _ => None,
    }
}

/// The view of an actor whose state `pick` takes out of its `ActorState`: its own holding and
/// state, the posted book, the params. A state of another kind is `Mismatch`.
fn role_view<'a, S>(
    a: ActorId,
    s: &'a SimState<Agents>,
    w: &'a World<Agents>,
    pick: fn(&ActorState) -> Option<&S>,
) -> Result<View<'a, S>, AgentError> {
    let decl = w.actor(a).ok_or(AgentError::Mismatch(a))?;
    let own_state = s
        .ext()
        .get(&a)
        .and_then(pick)
        .ok_or(AgentError::Mismatch(a))?;
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
