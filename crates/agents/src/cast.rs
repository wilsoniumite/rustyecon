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
//! under its cover (M5). A maker's reservation (L0.4) is a stock's, never the flow path's, and
//! needs a world whose one-sided markets `Saturate`. A planted desk (P2.2b; LOOPS-RULES §6) has a
//! θ in (0, 1], a plant that wears by less than all of it a tick, a positive size, the cash rule,
//! a bundle that costs something, and a plant good of its own; the herd's target is a capacity
//! desk's only; a planted type desk has no own input, and a planted maker is on the stock path
//! and uses no hours of its own horses. The workers' priced exit (P2.3, the commons) rents an
//! `Instant` land, the land of the provider that pays their support, and has land for every plot.

use crate::behaviour::{AgentError, Behaviour, Decision, Posted, View};
use crate::ext::{
    ActorState, Agents, CapacityState, GoodDeskState, MachDeskState, MakerState, OwnerState,
    PlantedCapacityState, PlantedMakerState, PlantedTypeState, ProviderState, ScriptState,
    WorkersState,
};
use crate::roles::many::spec::{BasketProvider, BasketWorkers, CategoryDesk, PricedExit, TypeDesk};
use crate::roles::plant::rules::{PlantedCapacity, PlantedMaker, PlantedType};
use crate::roles::plant::spec::{Plant, PlantTarget};
use crate::roles::spec::{GoodDesk, MachDesk, Provider, Scale, Workers};
use crate::roles::stock::rules::{MakerRole, OwnerRole};
use crate::roles::stock::spec::CapacityDesk;
use crate::spec::{Script, Spec};
use rustyecon_core::{
    ActorDecl, ActorId, ActorKind, GoodId, Holder, Key, Life, LoadError, LoadErrorKind, OneSided,
    SimState, Site, StateDelta, World,
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
    PlantedType(PlantedType),
    PlantedMaker(PlantedMaker),
    PlantedCapacity(PlantedCapacity),
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

/// The checks on the workers' priced exit that need the whole world (the commons frame's §3.4,
/// checks 2 to 5): the plots' land is `Instant` and is the land of the basket provider whose
/// transfer pays the workers' support (ν = 1: a pop with an exit must be one); and there is land
/// for every plot, T + T_o > h·N at genesis, T that provider's endowment (unit-1e.md §3.2). Check
/// 3, s₀, s̲, h and T_o finite and not negative, is the registry's: every param's value is finite
/// with a clear sign bit.
fn check_exit(
    w: &World<Agents>,
    decl: &ActorDecl<Spec>,
    p: &BasketWorkers,
    x: &PricedExit,
) -> Result<(), LoadError> {
    if w.good(x.land).map(|d| d.life) != Some(Life::Instant) {
        return Err(invalid(
            decl,
            "exit.land",
            "the plots' land is an Instant good, rented and used within the tick",
        ));
    }
    let provider = w.actors.iter().find_map(|a| match &a.spec {
        Spec::BasketProvider(bp) if bp.transfer_to == decl.id => Some((a, bp)),
        _ => None,
    });
    let Some((pd, bp)) = provider else {
        return Err(invalid(
            decl,
            "exit",
            "a pop with an exit is paid its support: it must be a basket provider's transfer.to \
             (ν = 1)",
        ));
    };
    if bp.land != x.land {
        return Err(invalid(
            decl,
            "exit.land",
            "the plots rent the land that pays the workers' support: their provider's endowment",
        ));
    }
    let h = at_genesis(w, decl, x.plot, "exit.plot")?;
    let to = at_genesis(w, decl, x.commons, "exit.commons")?;
    let t = at_genesis(w, pd, bp.endowment, "endowment")?;
    let n = at_genesis(w, decl, p.heads, "heads")?;
    if t + to <= h * n {
        return Err(invalid(
            decl,
            "exit.plot",
            "land for every plot: T + T_o > h·N at genesis, T the provider's endowment \
             (unit-1e.md §3.2)",
        ));
    }
    Ok(())
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
        Spec::BasketWorkers(p) => {
            let mut v: Vec<(GoodId, &'static str)> =
                p.basket.iter().map(|i| (i.good, "basket")).collect();
            v.extend(p.exit.iter().map(|x| (x.land, "exit.land")));
            v
        }
        Spec::CategoryDesk(d) => {
            let mut v = vec![
                (d.labour, "labour"),
                (d.service, "service"),
                (d.land, "land"),
            ];
            v.extend(d.reserved.iter().map(|i| (i.good, "reserved")));
            v
        }
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

/// The checks on a planted desk that need the genesis values (LOOPS-RULES §6): θ in (0, 1], a wear
/// below all of the plant a tick (δ_p 0 allowed, the fixed plant), a positive size, the herd's
/// target on a capacity desk only (`herd`), the cash rule (decision 296), a bundle with a
/// coefficient above 0 (else its cost is 0 and kr is 0/0), and a plant good no other desk's
/// plant names (`seen`).
fn check_plant(
    w: &World<Agents>,
    decl: &ActorDecl<Spec>,
    p: &Plant,
    scale: &Scale,
    herd: bool,
    bundle: &[(Site, &str)],
    seen: &mut Vec<GoodId>,
) -> Result<(), LoadError> {
    let theta = at_genesis(w, decl, p.theta, "plant.theta")?;
    if !(theta > 0.0 && theta <= 1.0) {
        return Err(invalid(decl, "plant.theta", "a plant's θ is in (0, 1]"));
    }
    if at_genesis(w, decl, p.delta, "plant.delta")? >= 1.0 {
        return Err(invalid(
            decl,
            "plant.delta",
            "a plant wears by less than all of it a tick",
        ));
    }
    if at_genesis(w, decl, p.size, "plant.size")? <= 0.0 {
        return Err(invalid(
            decl,
            "plant.size",
            "a plant unit is built from a positive number of bundles",
        ));
    }
    if p.target == PlantTarget::Herd && !herd {
        return Err(invalid(
            decl,
            "plant.target",
            "the herd's plant is a capacity desk's target: a type desk's or a maker's plant \
             targets its bundles",
        ));
    }
    if !matches!(scale, Scale::Cash { .. }) {
        return Err(invalid(
            decl,
            "scale",
            "a planted desk takes the cash rule: the step rule sizes its own state, and the \
             mirror ran no plant on it (decision 296)",
        ));
    }
    let mut costs = false;
    for &(site, field) in bundle {
        if at_genesis(w, decl, site, field)? > 0.0 {
            costs = true;
        }
    }
    if !costs {
        return Err(invalid(
            decl,
            "plant",
            "a plant's bundle has a coefficient above 0: a bundle that costs nothing has no \
             cheapest plant",
        ));
    }
    if seen.contains(&p.good) {
        return Err(invalid(
            decl,
            "plant.good",
            "a plant good belongs to one desk: another desk's plant names it",
        ));
    }
    seen.push(p.good);
    Ok(())
}

impl Cast {
    /// The cast of a world, after the checks that need the whole world.
    pub fn new(w: &World<Agents>) -> Result<Cast, LoadError> {
        let mut members = Vec::with_capacity(w.actors.len());
        let mut plants: Vec<GoodId> = Vec::new();
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
                    // Its further transfers (P2.3): none to itself.
                    if let Some(t) = p.more.iter().find(|t| t.to == decl.id) {
                        let key = w
                            .key_of(t.to)
                            .map_or_else(|| t.to.to_string(), |k| k.to_string());
                        return Err(invalid(
                            decl,
                            &format!("more[{key}].to"),
                            "an actor does not pay itself",
                        ));
                    }
                    Member::BasketProvider(p.clone())
                }
                Spec::BasketWorkers(p) => {
                    check_role(w, decl, ActorKind::Pop, Some((p.labour, "labour")), None)?;
                    if let Some(x) = &p.exit {
                        check_exit(w, decl, p, x)?;
                    }
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
                    match &d.plant {
                        None => Member::TypeDesk(d.clone()),
                        Some(p) => {
                            // The bundle: the bought inputs, labour and land (LOOPS-RULES §4.1).
                            let mut bundle: Vec<(Site, &str)> =
                                d.inputs.iter().map(|i| (i.coef, "recipe.inputs")).collect();
                            bundle.push((d.labour_coef, "recipe.labour"));
                            bundle.push((d.land_coef, "recipe.land"));
                            check_plant(w, decl, p, &d.scale, false, &bundle, &mut plants)?;
                            if at_genesis(w, decl, d.own, "recipe.own")? != 0.0 {
                                return Err(invalid(
                                    decl,
                                    "recipe.own",
                                    "a planted type desk keeps no own input: LOOP-SPEC runs \
                                     none (decision 296)",
                                ));
                            }
                            Member::PlantedType(PlantedType::new(d, p))
                        }
                    }
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
                    // The reservation (IDLE-SPEC §6, L0.4): a stock's, and only where a market
                    // with a bid and no offer raises its price.
                    if d.reserve.is_some() {
                        if flow {
                            return Err(invalid(
                                decl,
                                "reserve",
                                "the flow path holds no reservation",
                            ));
                        }
                        if w.market.one_sided == OneSided::Hold {
                            return Err(invalid(
                                decl,
                                "reserve",
                                "a reservation needs `Saturate`: under `Hold` a market with a \
                                 bid and no offer keeps its price, so a maker that withholds \
                                 would never see it rise",
                            ));
                        }
                    }
                    match &d.plant {
                        None => Member::Maker(MakerRole::new(d, flow)),
                        Some(p) => {
                            // LOOP-SPEC §2.5: a maker from bought inputs, on the stock path, its
                            // bundle the build's goods, labour and land.
                            if flow {
                                return Err(invalid(
                                    decl,
                                    "plant",
                                    "a planted maker holds its durable good as a stock: the \
                                     flow path has no plant",
                                ));
                            }
                            if at_genesis(w, decl, d.own_hours, "own_hours")? != 0.0 {
                                return Err(invalid(
                                    decl,
                                    "own_hours",
                                    "a planted maker builds from bought inputs alone, with no \
                                     hours of its own stock (LOOP-SPEC §2.5; decision 296)",
                                ));
                            }
                            let mut bundle: Vec<(Site, &str)> = d
                                .build
                                .goods
                                .iter()
                                .map(|i| (i.coef, "build.goods"))
                                .collect();
                            bundle.push((d.build.labour, "build.labour"));
                            bundle.push((d.build.land, "build.land"));
                            check_plant(w, decl, p, &d.scale, false, &bundle, &mut plants)?;
                            Member::PlantedMaker(PlantedMaker::new(d, p))
                        }
                    }
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
                    match &d.plant {
                        None => Member::CapacityDesk(d.clone()),
                        Some(p) => {
                            // Its bundle is the running recipe (decision 261).
                            let mut bundle: Vec<(Site, &str)> = d
                                .running
                                .goods
                                .iter()
                                .map(|i| (i.coef, "running.goods"))
                                .collect();
                            bundle.push((d.running.labour, "running.labour"));
                            check_plant(w, decl, p, &d.scale, true, &bundle, &mut plants)?;
                            Member::PlantedCapacity(PlantedCapacity::new(d, p))
                        }
                    }
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
            Member::PlantedType(b) => b.decide(&role_view(a, s, w, planted_type)?),
            Member::PlantedMaker(b) => b.decide(&role_view(a, s, w, planted_maker)?),
            Member::PlantedCapacity(b) => b.decide(&role_view(a, s, w, planted_capacity)?),
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
            Member::PlantedType(b) => b.produce(&role_view(a, s, w, planted_type)?),
            Member::PlantedMaker(b) => b.produce(&role_view(a, s, w, planted_maker)?),
            Member::PlantedCapacity(b) => b.produce(&role_view(a, s, w, planted_capacity)?),
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
            Member::PlantedType(b) => b.upkeep(&role_view(a, s, w, planted_type)?),
            Member::PlantedMaker(b) => b.upkeep(&role_view(a, s, w, planted_maker)?),
            Member::PlantedCapacity(b) => b.upkeep(&role_view(a, s, w, planted_capacity)?),
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

fn planted_type(s: &ActorState) -> Option<&PlantedTypeState> {
    match s {
        ActorState::PlantedType(st) => Some(st),
        _ => None,
    }
}

fn planted_maker(s: &ActorState) -> Option<&PlantedMakerState> {
    match s {
        ActorState::PlantedMaker(st) => Some(st),
        _ => None,
    }
}

fn planted_capacity(s: &ActorState) -> Option<&PlantedCapacityState> {
    match s {
        ActorState::PlantedCapacity(st) => Some(st),
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
