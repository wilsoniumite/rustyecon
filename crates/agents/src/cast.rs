//! The cast: one behaviour per declared actor, dispatched by kind (docs/ENGINE.md §4).
//!
//! `Cast` is built once from the world's declarations and never changes; an actor's mutable
//! state lives in `SimState` and reaches its hooks through a [`View`]. The cast also makes the
//! load checks that need the whole world rather than one spec: a scripted actor budgets in its
//! home currency, so it buys only at nodes that quote in it, and it never pays itself. An
//! Appendix B role (P2.0), or a many-market role (P2.1), trades at its home node only; a desk role must be a Desk and a
//! household role a Pop; what it is endowed with must be `Instant`, since `decide` may mint only
//! that; and it never transfers to or pays itself. A stock role (P2.2) holds a durable good that
//! is `Indefinite` and wears by less than all of it a tick, or, for the maker and the owner desk,
//! one that lives one tick at δ = 1, the flow path, with no cover and no running recipe. A good
//! that lives more than one tick is bought only as the durable good a capacity or owner desk
//! holds, which it nets (M6), and no role offers one in full: the maker sells its durable good
//! under its cover (M5).

use crate::behaviour::{AgentError, Behaviour, Decision, Posted, View};
use crate::ext::{
    ActorState, Agents, CapacityState, GoodDeskState, MachDeskState, MakerState, OwnerState,
    ProviderState, ScriptState, WorkersState,
};
use crate::roles::many::spec::{BasketProvider, BasketWorkers, CategoryDesk, TypeDesk};
use crate::roles::spec::{GoodDesk, MachDesk, Provider, Workers};
use crate::roles::stock::rules::{MakerRole, OwnerRole};
use crate::roles::stock::spec::CapacityDesk;
use crate::spec::{Script, Spec};
use rustyecon_core::{
    ActorDecl, ActorId, ActorKind, GoodId, Holder, Key, Life, LoadError, LoadErrorKind, SimState,
    Site, StateDelta, World,
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
    Maker(MakerRole),
    CapacityDesk(CapacityDesk),
    OwnerDesk(OwnerRole),
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

fn invalid(decl: &ActorDecl<Spec>, field: &str, why: &str) -> LoadError {
    LoadError::new(
        format!("actors[{}].spec.{field}", decl.key),
        LoadErrorKind::Invalid(why.to_string()),
    )
}

/// A live param's per-tick value at genesis, converted by its site's method.
fn at_genesis(
    w: &World<Agents>,
    decl: &ActorDecl<Spec>,
    site: Site,
    field: &str,
) -> Result<f64, LoadError> {
    let v = w
        .registry
        .get(site.param)
        .map(|p| p.genesis)
        .ok_or_else(|| invalid(decl, field, "an unregistered param"))?;
    site.convert(&w.clock, v)
        .map_err(|e| invalid(decl, field, &e.to_string()))
}

/// Whether a stock role's durable good takes the flow path (HORSES-SPEC §3, SG6 and SG7): an
/// `Indefinite` good is a stock worn by δ < 1 a tick; a good that lives one tick is the probe's
/// flow service, at δ = 1 a tick exactly; any other life is refused.
fn flow_path(
    w: &World<Agents>,
    decl: &ActorDecl<Spec>,
    good: GoodId,
    field: &str,
    delta: Site,
) -> Result<bool, LoadError> {
    let d = at_genesis(w, decl, delta, "delta")?;
    match w.good(good).map(|g| g.life) {
        Some(Life::Indefinite) if d < 1.0 => Ok(false),
        Some(Life::Indefinite) => Err(invalid(
            decl,
            "delta",
            "a durable good held as a stock wears by less than all of it a tick",
        )),
        Some(Life::Ticks(1)) if d == 1.0 => Ok(true),
        Some(Life::Ticks(1)) => Err(invalid(
            decl,
            "delta",
            "a good that lives one tick is the flow path, which wears all of it a tick",
        )),
        _ => Err(invalid(
            decl,
            field,
            "a stock role's durable good is Indefinite (a stock) or lives one tick (the flow \
             path)",
        )),
    }
}

/// The goods a kind buys and does not net against a holding, each with its field: the goods
/// whose leftovers must die at the tick's ageing (M6). The capacity and owner desks net their
/// holding of the durable good they hold and of nothing else, so their running goods and labour
/// are listed (O48: they were once exempt for every good they buy). A scripted actor's lines are
/// its own affair.
fn bought(s: &Spec) -> Vec<(GoodId, &'static str)> {
    match s {
        Spec::Scripted(_) => Vec::new(),
        Spec::Provider(p) => vec![
            (p.basket.good, "basket.good"),
            (p.basket.space, "basket.space"),
        ],
        Spec::Workers(p) => vec![
            (p.basket.good, "basket.good"),
            (p.basket.space, "basket.space"),
        ],
        Spec::GoodDesk(d) => vec![(d.labour, "labour"), (d.mach, "mach")],
        Spec::MachDesk(d) => vec![(d.labour, "labour"), (d.land, "land")],
        Spec::BasketProvider(p) => p.basket.iter().map(|i| (i.good, "basket")).collect(),
        Spec::BasketWorkers(p) => p.basket.iter().map(|i| (i.good, "basket")).collect(),
        Spec::CategoryDesk(d) => vec![
            (d.labour, "labour"),
            (d.service, "service"),
            (d.land, "land"),
        ],
        Spec::TypeDesk(d) => {
            let mut v: Vec<(GoodId, &'static str)> =
                d.inputs.iter().map(|i| (i.good, "recipe.inputs")).collect();
            v.extend([(d.labour, "labour"), (d.land, "land")]);
            v
        }
        Spec::Maker(d) => {
            let mut v: Vec<(GoodId, &'static str)> = d
                .running
                .goods
                .iter()
                .map(|i| (i.good, "running.goods"))
                .collect();
            v.extend(d.build.goods.iter().map(|i| (i.good, "build.goods")));
            v.extend([(d.labour, "labour"), (d.land, "land")]);
            v
        }
        Spec::CapacityDesk(d) => {
            let mut v: Vec<(GoodId, &'static str)> = d
                .running
                .goods
                .iter()
                .map(|i| (i.good, "running.goods"))
                .collect();
            v.push((d.labour, "labour"));
            v
        }
        Spec::OwnerDesk(d) => {
            let mut v: Vec<(GoodId, &'static str)> =
                d.running.iter().map(|i| (i.good, "running")).collect();
            v.push((d.labour, "labour"));
            v
        }
    }
}

/// The goods a kind offers in full, every unit it holds beyond its own use, each with its field:
/// the goods whose unsold units must die at the tick's ageing (M5). The maker offers its durable
/// good under its cover (M2's band, D-G5; `None` is P3 (v)'s control), and on the flow path that
/// good lives one tick (`flow_path`), so it is not listed. A scripted actor's lines are its own
/// affair.
fn sold_in_full(s: &Spec) -> Vec<(GoodId, &'static str)> {
    match s {
        Spec::Scripted(_) | Spec::Maker(_) => Vec::new(),
        Spec::Provider(p) => vec![(p.land, "land")],
        Spec::Workers(p) => vec![(p.labour, "labour")],
        Spec::BasketProvider(p) => vec![(p.land, "land")],
        Spec::BasketWorkers(p) => vec![(p.labour, "labour")],
        Spec::GoodDesk(d) => vec![(d.output, "output")],
        Spec::MachDesk(d) => vec![(d.output, "output")],
        Spec::CategoryDesk(d) => vec![(d.output, "output")],
        Spec::TypeDesk(d) => vec![(d.output, "output")],
        Spec::CapacityDesk(d) => vec![(d.hours, "hours")],
        Spec::OwnerDesk(d) => vec![(d.output, "output")],
    }
}

/// Whether a good lives at most one tick: `Instant` or one tick. Any other life stores it.
fn dies_within_a_tick(w: &World<Agents>, g: GoodId) -> bool {
    matches!(
        w.good(g).map(|d| d.life),
        Some(Life::Instant | Life::Ticks(1))
    )
}

/// M6 (HORSES-SPEC §3, SG7): a good that lives more than one tick is bought only by a kind that
/// nets its holding of it. Any other kind would carry what it did not use into the next tick as
/// if it were new.
fn check_bought(w: &World<Agents>, decl: &ActorDecl<Spec>) -> Result<(), LoadError> {
    for (g, field) in bought(&decl.spec) {
        if !dies_within_a_tick(w, g) {
            return Err(invalid(
                decl,
                field,
                "a good that lives more than one tick is bought only as the durable good a \
                 capacity or owner desk holds, which it nets",
            ));
        }
    }
    Ok(())
}

/// M5 (GOODS-CHAIN §3.1, §3.2 fact 1): a good that lives more than one tick is not offered in
/// full. A seller that offers every unit it holds keeps what did not sell and offers it again,
/// a unit root in its stock; the share rule, I/(1 + b_G), or the maker's cover, damps it.
fn check_sold(w: &World<Agents>, decl: &ActorDecl<Spec>) -> Result<(), LoadError> {
    for (g, field) in sold_in_full(&decl.spec) {
        if !dies_within_a_tick(w, g) {
            return Err(invalid(
                decl,
                field,
                "a good that lives more than one tick is not offered in full: a stock offered \
                 in full is a unit root (M5)",
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
                Spec::Maker(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    let flow = flow_path(w, decl, d.output, "output", d.delta)?;
                    // The flow path is the type desk's: no cover and no running recipe.
                    if flow && d.cover.is_some() {
                        return Err(invalid(decl, "cover", "the flow path holds no cover"));
                    }
                    if flow
                        && (!d.running.goods.is_empty()
                            || at_genesis(w, decl, d.running.labour, "running.labour")? != 0.0)
                    {
                        return Err(invalid(
                            decl,
                            "running",
                            "the flow path has no running recipe: its own input is its kept \
                             output",
                        ));
                    }
                    Member::Maker(MakerRole::new(d, flow))
                }
                Spec::CapacityDesk(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    if flow_path(w, decl, d.stock, "stock", d.delta)? {
                        return Err(invalid(
                            decl,
                            "stock",
                            "a capacity desk holds a durable good: at δ = 1 it would hold none \
                             when it decides",
                        ));
                    }
                    Member::CapacityDesk(d.clone())
                }
                Spec::OwnerDesk(d) => {
                    let pays = d.scale.pays().map(|a| (a, "scale.payout.to"));
                    check_role(w, decl, ActorKind::Desk, None, pays)?;
                    let flow = flow_path(w, decl, d.stock, "stock", d.delta)?;
                    if flow && !d.running.is_empty() {
                        return Err(invalid(
                            decl,
                            "running",
                            "the flow path has no running recipe: it is the good desk's",
                        ));
                    }
                    Member::OwnerDesk(OwnerRole::new(d, flow))
                }
            };
            members.push((decl.id, member));
        }
        // M6, once every actor's own checks have passed; then M5, so that a tape an older check
        // refuses reports the older error.
        for decl in &w.actors {
            check_bought(w, decl)?;
        }
        for decl in &w.actors {
            check_sold(w, decl)?;
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
            Member::Maker(b) => b.decide(&role_view(a, s, w, maker)?),
            Member::CapacityDesk(b) => b.decide(&role_view(a, s, w, capacity)?),
            Member::OwnerDesk(b) => b.decide(&role_view(a, s, w, owner)?),
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
            Member::Maker(b) => b.produce(&role_view(a, s, w, maker)?),
            Member::CapacityDesk(b) => b.produce(&role_view(a, s, w, capacity)?),
            Member::OwnerDesk(b) => b.produce(&role_view(a, s, w, owner)?),
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
            Member::Maker(b) => b.upkeep(&role_view(a, s, w, maker)?),
            Member::CapacityDesk(b) => b.upkeep(&role_view(a, s, w, capacity)?),
            Member::OwnerDesk(b) => b.upkeep(&role_view(a, s, w, owner)?),
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

fn maker(s: &ActorState) -> Option<&MakerState> {
    match s {
        ActorState::Maker(st) => Some(st),
        _ => None,
    }
}

fn capacity(s: &ActorState) -> Option<&CapacityState> {
    match s {
        ActorState::Capacity(st) => Some(st),
        _ => None,
    }
}

fn owner(s: &ActorState) -> Option<&OwnerState> {
    match s {
        ActorState::Owner(st) => Some(st),
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
