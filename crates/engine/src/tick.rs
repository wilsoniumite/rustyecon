//! One tick (docs/ENGINE.md §7.2 and §7.3): eight phases under one ledger, every change a core
//! delta through `apply`, and the hooks' output checked against its whitelist before any of it
//! is applied.
//!
//! Salvaged in design from `v2p3: systems/mod.rs`, with the new phase order, the audit always
//! returned (July's `run_tick` discarded it, `v2p3: systems/mod.rs:37`, N3), phase-start reads
//! for every hook (E6) and a whitelist (E7).

use crate::error::RunErrorKind;
use crate::report::{FiredEvent, MarketLine, TickReport, TraceEntry};
use rustyecon_agents::{Agents, Cast, Hook};
use rustyecon_core::{
    apply, state_hash, ActorId, CoreError, Ext, Holder, Ledger, Life, Phase, Provenance, RunLedger,
    SimState, StateDelta, World,
};
use rustyecon_markets::{admit, clear, settle, update_prices, Order, SideTag};

pub(crate) type Delta = StateDelta<Agents>;

/// Where and why a tick stopped.
#[derive(Debug)]
pub(crate) struct Failure {
    pub(crate) phase: Phase,
    pub(crate) kind: RunErrorKind,
}

fn fail(phase: Phase) -> impl Fn(RunErrorKind) -> Failure {
    move |kind| Failure { phase, kind }
}

fn core(phase: Phase) -> impl Fn(CoreError) -> Failure {
    move |e| Failure {
        phase,
        kind: RunErrorKind::Core(e),
    }
}

/// One hook's output for one actor.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HookOutput {
    pub(crate) actor: ActorId,
    pub(crate) deltas: Vec<Delta>,
    pub(crate) orders: Vec<Order>,
}

/// Run `hook` for every actor in `visit`, each on the same state (E6), and return the outputs
/// in `ActorId` order whatever the visiting order.
pub(crate) fn collect(
    hook: Hook,
    cast: &Cast,
    s: &SimState<Agents>,
    w: &World<Agents>,
    visit: impl Iterator<Item = ActorId>,
) -> Result<Vec<HookOutput>, RunErrorKind> {
    let mut out = Vec::new();
    for actor in visit {
        let agent = |error| RunErrorKind::Agent { actor, hook, error };
        let (deltas, orders) = match hook {
            Hook::Decide => {
                let d = cast.decide(actor, s, w).map_err(agent)?;
                (d.deltas, d.orders)
            }
            Hook::Produce => (cast.produce(actor, s, w).map_err(agent)?, Vec::new()),
            Hook::Upkeep => (cast.upkeep(actor, s, w).map_err(agent)?, Vec::new()),
        };
        out.push(HookOutput {
            actor,
            deltas,
            orders,
        });
    }
    out.sort_by_key(|o| o.actor);
    Ok(out)
}

/// Whether `hook` may emit `d` for `me` (§7.3's table). Core's phase rules still apply
/// underneath.
pub(crate) fn allowed(hook: Hook, me: ActorId, d: &Delta, w: &World<Agents>) -> bool {
    let mine = Holder::Actor(me);
    match (hook, d) {
        (_, StateDelta::Actor(x)) => Agents::owner(x) == Some(me),
        (Hook::Decide, StateDelta::Transfer { from, to, .. }) => {
            *from == mine && matches!(to, Holder::Actor(_))
        }
        (Hook::Decide, StateDelta::Mint { to, good, prov, .. }) => {
            *to == mine
                && *prov == Provenance::Endowment
                && w.good(*good).is_some_and(|g| g.life == Life::Instant)
        }
        (Hook::Produce, StateDelta::Burn { from, prov, .. }) => {
            let own = match me {
                ActorId::Desk(_) => Provenance::Production,
                ActorId::Pop(_) => Provenance::Consumption,
            };
            *from == mine && *prov == own
        }
        (Hook::Produce, StateDelta::Mint { to, prov, .. }) => {
            *to == mine && matches!(prov, Provenance::Production | Provenance::Endowment)
        }
        (Hook::Upkeep, StateDelta::Burn { from, prov, .. }) => {
            *from == mine && *prov == Provenance::Depreciation
        }
        _ => false,
    }
}

/// Check one actor's output against the whitelist: every delta allowed, every order its own
/// and in its class, and orders from `decide` only.
pub(crate) fn check(hook: Hook, out: &HookOutput, w: &World<Agents>) -> Result<(), RunErrorKind> {
    let actor = out.actor;
    if let Some(delta) = out.deltas.iter().find(|d| !allowed(hook, actor, d, w)) {
        return Err(RunErrorKind::ForeignWrite {
            actor,
            hook,
            delta: delta.clone(),
        });
    }
    let class = w.actor(actor).map(|a| a.class);
    if let Some(order) = out
        .orders
        .iter()
        .find(|o| hook != Hook::Decide || o.actor != actor || Some(o.class) != class)
    {
        return Err(RunErrorKind::ForeignOrder {
            actor,
            order: *order,
        });
    }
    Ok(())
}

/// Collects the applied deltas of a traced step.
pub(crate) struct Tracer<'t>(pub(crate) Option<&'t mut Vec<TraceEntry>>);

impl Tracer<'_> {
    fn record(&mut self, phase: Phase, ds: &[Delta], moved: &[f64]) {
        if let Some(out) = self.0.as_deref_mut() {
            out.extend(ds.iter().zip(moved).map(|(d, &m)| TraceEntry {
                phase,
                delta: d.clone(),
                moved: m,
            }));
        }
    }
}

/// The state, world and ledger of a running tick.
pub(crate) struct Run<'a, 't> {
    pub(crate) w: &'a World<Agents>,
    pub(crate) s: &'a mut SimState<Agents>,
    pub(crate) l: Ledger,
    pub(crate) trace: Tracer<'t>,
}

impl Run<'_, '_> {
    /// Apply `ds` in `phase`, recording them in the trace.
    pub(crate) fn apply(&mut self, phase: Phase, ds: &[Delta]) -> Result<Vec<f64>, Failure> {
        let moved = apply(self.s, self.w, phase, ds, &mut self.l).map_err(core(phase))?;
        self.trace.record(phase, ds, &moved);
        Ok(moved)
    }

    /// Check every actor's output, then apply the deltas in `ActorId` order. Returns the orders,
    /// in the same order. Nothing is applied unless every output passes.
    pub(crate) fn apply_hooks(
        &mut self,
        hook: Hook,
        phase: Phase,
        outputs: Vec<HookOutput>,
    ) -> Result<Vec<Order>, Failure> {
        for out in &outputs {
            check(hook, out, self.w).map_err(fail(phase))?;
        }
        let mut deltas = Vec::new();
        let mut orders = Vec::new();
        for out in outputs {
            deltas.extend(out.deltas);
            orders.extend(out.orders);
        }
        self.apply(phase, &deltas)?;
        Ok(orders)
    }

    fn hooks(&mut self, cast: &Cast, hook: Hook, phase: Phase) -> Result<Vec<Order>, Failure> {
        let outputs = collect(hook, cast, self.s, self.w, cast.actors()).map_err(fail(phase))?;
        self.apply_hooks(hook, phase, outputs)
    }
}

/// Run one tick on `s`: phases 0 to 7 (§7.2). Its ledger closes through the run's ledger, which
/// checks the tick and then the run so far (R2). On an error the state is left partway through
/// the tick, and the caller poisons the `Sim`.
pub(crate) fn run_tick(
    w: &World<Agents>,
    s: &mut SimState<Agents>,
    cast: &Cast,
    ledger: &mut RunLedger,
    trace: Option<&mut Vec<TraceEntry>>,
) -> Result<TickReport, Failure> {
    let t = s.tick();
    let l = Ledger::open(s, w).map_err(core(Phase::Events))?;
    let mut run = Run {
        w,
        s,
        l,
        trace: Tracer(trace),
    };

    // 0. Events, in firing order.
    let firings = w.schedule.fire(t);
    let actions: Vec<Delta> = firings.iter().map(|f| f.action.clone()).collect();
    run.apply(Phase::Events, &actions)?;
    let mut events = Vec::with_capacity(firings.len());
    for f in firings {
        let key = w
            .key_of(f.event)
            .ok_or_else(|| CoreError::Shape(format!("{} has no key", f.event)))
            .map_err(core(Phase::Events))?;
        events.push(FiredEvent {
            key: key.clone(),
            occurrence: f.occurrence,
            action: f.action,
        });
    }

    // 1. Decisions, on the phase-start state.
    let orders = run.hooks(cast, Hook::Decide, Phase::Decisions)?;

    // 2. Admission and clearing.
    let order = |phase| {
        move |e| Failure {
            phase,
            kind: RunErrorKind::Order(e),
        }
    };
    let lines = admit(orders, run.s, w).map_err(order(Phase::Clearing))?;
    let (volumes, fills) = clear(&lines, w).map_err(order(Phase::Clearing))?;
    run.apply(Phase::Clearing, &volumes)?;

    // 3. Settlement through the escrows; none may remain.
    let plan = settle(&lines, &fills, run.s, w).map_err(order(Phase::Settlement))?;
    let moved = run.apply(Phase::Settlement, plan.deltas())?;
    let (settlements, rationing) = plan.realize(&moved).map_err(order(Phase::Settlement))?;
    if let Some(Holder::Escrow(node, good)) = run
        .s
        .holdings()
        .keys()
        .find(|h| matches!(h, Holder::Escrow(..)))
    {
        return Err(Failure {
            phase: Phase::Settlement,
            kind: RunErrorKind::Core(CoreError::EscrowLeft {
                node: *node,
                good: *good,
            }),
        });
    }
    // The prices settlement used, for the report.
    let mut posted = Vec::new();
    for (n, g) in w.markets() {
        let p = run
            .s
            .price(n, g)
            .ok_or_else(|| CoreError::Shape(format!("no book slot for ({n}, {g})")))
            .map_err(core(Phase::Settlement))?;
        posted.push(p);
    }

    // 4. Production, on the phase-start state.
    run.hooks(cast, Hook::Produce, Phase::Production)?;

    // 5a. Core ageing, per holder in Holder order; 5b. the upkeep hook.
    let ages: Vec<Delta> = run
        .s
        .holdings()
        .keys()
        .map(|&holder| StateDelta::Age { holder })
        .collect();
    run.apply(Phase::Upkeep, &ages)?;
    run.hooks(cast, Hook::Upkeep, Phase::Upkeep)?;

    // 6. Prices, then the tick advances.
    let mut prices = update_prices(run.s, w).map_err(|e| Failure {
        phase: Phase::Prices,
        kind: RunErrorKind::Price(e),
    })?;
    prices.push(StateDelta::AdvanceTick);
    run.apply(Phase::Prices, &prices)?;

    // 7. Measure: the tick's audit and the run's, the hash, the report. A breach of either
    // stops the run here (R2); nothing discards it (N3).
    let Run { s, l, .. } = run;
    let (audit, run_audit) = ledger.close_tick(l, s, w).map_err(core(Phase::Measure))?;
    let hash = state_hash(s);
    let date = w
        .clock
        .date_of(t)
        .ok_or_else(|| CoreError::Shape(format!("tick {t} has no date")))
        .map_err(core(Phase::Measure))?;
    let mut markets = Vec::with_capacity(posted.len());
    for ((node, good), price) in w.markets().zip(posted) {
        let fill = fills
            .get(node, good)
            .ok_or_else(|| CoreError::Shape(format!("no fill for ({node}, {good})")))
            .map_err(core(Phase::Measure))?;
        let quote = s
            .book()
            .quote(node, good)
            .ok_or_else(|| CoreError::Shape(format!("no book slot for ({node}, {good})")))
            .map_err(core(Phase::Measure))?;
        let cleared = settlements
            .iter()
            .filter(|l| l.node == node && l.good == good && l.side == SideTag::Buy)
            .fold(0.0, |acc, l| acc + l.qty);
        markets.push(MarketLine {
            node,
            good,
            price,
            next_price: quote.price,
            ema: quote.ema,
            supply: fill.supply,
            demand: fill.demand,
            cleared,
            buyer_fill: fill.buyer_fill,
            seller_fill: fill.seller_fill,
        });
    }
    Ok(TickReport {
        tick: t,
        date,
        hash,
        markets,
        settlements,
        rationing,
        audit,
        run: run_audit,
        events,
    })
}

#[cfg(test)]
mod tests {
    //! The hooks (docs/ENGINE.md §7.3 and §11, engine; E6, E7, R13, R2), on the gate world.

    use super::*;
    use crate::Tape;
    use rustyecon_agents::AgentDelta;
    use rustyecon_core::{resolve, Amount, DeskId, GoodId, NodeId, ParamId, PopId};
    use rustyecon_markets::Side;

    const GATE: &str = include_str!("../../../tapes/gate.ron");

    fn gate(text: &str) -> (World<Agents>, SimState<Agents>, Cast) {
        let t = Tape::from_ron(text).expect("the tape parses");
        let (w, s) = resolve(&t).expect("the tape resolves");
        let cast = Cast::new(&w).expect("the cast builds");
        (w, s, cast)
    }

    #[test]
    fn decisions_read_phase_start_state() {
        // E6: from a fixed state, visiting the actors in id order and in reverse gives identical
        // outputs, for every hook.
        let (w, mut s, cast) = gate(GATE);
        let mut ledger = RunLedger::open(&s, &w).unwrap();
        for _ in 0..100 {
            run_tick(&w, &mut s, &cast, &mut ledger, None).expect("the gate world runs");
        }
        let ids: Vec<ActorId> = cast.actors().collect();
        for hook in [Hook::Decide, Hook::Produce, Hook::Upkeep] {
            let forward = collect(hook, &cast, &s, &w, ids.iter().copied()).unwrap();
            let reverse = collect(hook, &cast, &s, &w, ids.iter().rev().copied()).unwrap();
            assert_eq!(forward, reverse, "{hook}");
            if hook != Hook::Upkeep {
                assert!(forward.iter().any(|o| !o.deltas.is_empty()), "{hook} emits");
            }
        }
        // Renaming keys so that the id order reverses leaves every actor's tick-0 orders and
        // deltas unchanged, up to the renaming (ENGINE §14: at tick 0 only, since later sums
        // fold in the new id order).
        let renames = [
            ("farm", "desk.d"),
            ("mill", "desk.c"),
            ("mine", "desk.b"),
            ("oven", "desk.a"),
            ("pensioners", "pop.b"),
            ("workers", "pop.a"),
        ];
        let mut text = GATE.to_string();
        for (from, to) in renames {
            text = text.replace(&format!("\"{from}\""), &format!("\"{to}\""));
        }
        let (w2, s2, cast2) = gate(&text);
        let map = |a: ActorId| -> ActorId {
            let key = w.key_of(a).unwrap().as_str();
            let to = renames.iter().find(|(f, _)| *f == key).unwrap().1;
            w2.id_of::<ActorId>(to).unwrap()
        };
        assert_eq!(map(ActorId::Desk(DeskId(0))), ActorId::Desk(DeskId(3)));
        assert_eq!(map(ActorId::Pop(PopId(0))), ActorId::Pop(PopId(1)));
        let (_, s0, _) = gate(GATE);
        let before = collect(Hook::Decide, &cast, &s0, &w, cast.actors()).unwrap();
        let after = collect(Hook::Decide, &cast2, &s2, &w2, cast2.actors()).unwrap();
        let holder = |h: Holder| match h {
            Holder::Actor(a) => Holder::Actor(map(a)),
            e => e,
        };
        let mut renamed: Vec<HookOutput> = before
            .into_iter()
            .map(|o| HookOutput {
                actor: map(o.actor),
                deltas: o
                    .deltas
                    .into_iter()
                    .map(|d| match d {
                        StateDelta::Transfer {
                            from,
                            to,
                            good,
                            amount,
                        } => StateDelta::Transfer {
                            from: holder(from),
                            to: holder(to),
                            good,
                            amount,
                        },
                        other => other,
                    })
                    .collect(),
                orders: o
                    .orders
                    .into_iter()
                    .map(|mut ord| {
                        ord.actor = map(ord.actor);
                        ord
                    })
                    .collect(),
            })
            .collect();
        renamed.sort_by_key(|o| o.actor);
        assert_eq!(renamed.len(), after.len());
        for (r, a) in renamed.iter().zip(&after) {
            assert_eq!(r.actor, a.actor);
            assert_eq!(r.orders, a.orders, "{}'s orders moved with its id", r.actor);
            // A payout split follows ActorId order (the last recipient takes the remainder), so
            // its rounding may move with the renaming; who is paid does not.
            let paid = |o: &HookOutput| {
                let mut to: Vec<Holder> = o
                    .deltas
                    .iter()
                    .map(|d| match d {
                        StateDelta::Transfer { to, .. } => *to,
                        other => panic!("decide emitted {other:?}"),
                    })
                    .collect();
                to.sort();
                to
            };
            assert_eq!(paid(r), paid(a));
        }
        assert!(after
            .iter()
            .any(|o| !o.orders.is_empty() && !o.deltas.is_empty()));
    }

    #[test]
    fn behaviour_output_is_whitelisted() {
        // E7: each hook may emit only its own list (§7.3); anything else is ForeignWrite or
        // ForeignOrder, checked before any of the phase's deltas applies.
        let (w, s, _) = gate(GATE);
        let (farm, mill, workers) = (
            ActorId::Desk(DeskId(0)),
            ActorId::Desk(DeskId(1)),
            ActorId::Pop(PopId(1)),
        );
        let (bread, coin, town) = (GoodId(0), GoodId(1), NodeId(0));
        assert_eq!(w.key_of(bread).unwrap().as_str(), "bread");
        assert_eq!(w.key_of(coin).unwrap().as_str(), "coin");
        let me = Holder::Actor(mill);
        let other = Holder::Actor(farm);
        let escrow = Holder::Escrow(town, bread);
        let mint = |to, prov| StateDelta::Mint {
            to,
            good: bread,
            qty: 1.0,
            prov,
        };
        let burn = |from, prov| StateDelta::Burn {
            from,
            good: bread,
            amount: Amount::Qty(1.0),
            prov,
        };
        let transfer = |from, to| StateDelta::Transfer {
            from,
            to,
            good: coin,
            amount: Amount::Qty(1.0),
        };
        let set_active = |actor| {
            StateDelta::Actor(AgentDelta::SetActive {
                actor,
                active: true,
            })
        };
        let everywhere: Vec<Delta> = vec![
            mint(other, Provenance::Production),
            burn(other, Provenance::Production),
            transfer(escrow, me),
            transfer(me, escrow),
            transfer(other, me),
            StateDelta::SetPrice {
                node: town,
                good: bread,
                price: 1.0,
            },
            StateDelta::SetEma {
                node: town,
                good: bread,
                ema: 1.0,
            },
            StateDelta::SetVolumes {
                node: town,
                good: bread,
                supply: 1.0,
                demand: 1.0,
            },
            StateDelta::SetParam {
                param: ParamId(0),
                value: 1.0,
            },
            StateDelta::Age { holder: me },
            StateDelta::AdvanceTick,
            set_active(farm),
        ];
        let forbidden: [(Hook, Vec<Delta>); 3] = [
            (
                Hook::Decide,
                vec![
                    burn(me, Provenance::Production),
                    // Bread is not Instant: only an Instant endowment may be minted in decide.
                    mint(me, Provenance::Endowment),
                ],
            ),
            (
                Hook::Produce,
                vec![
                    burn(me, Provenance::Consumption),
                    burn(me, Provenance::Depreciation),
                    mint(me, Provenance::Event),
                    transfer(me, other),
                ],
            ),
            (
                Hook::Upkeep,
                vec![
                    burn(me, Provenance::Production),
                    mint(me, Provenance::Production),
                    transfer(me, other),
                ],
            ),
        ];
        let out = |deltas: Vec<Delta>, orders: Vec<rustyecon_markets::Order>| HookOutput {
            actor: mill,
            deltas,
            orders,
        };
        for (hook, own) in forbidden {
            for d in everywhere.iter().cloned().chain(own) {
                match check(hook, &out(vec![d.clone()], Vec::new()), &w) {
                    Err(RunErrorKind::ForeignWrite { actor, delta, .. }) => {
                        assert_eq!((actor, delta), (mill, d));
                    }
                    other => panic!("{hook}: {d:?} was {other:?}"),
                }
            }
        }
        let allowed: [(Hook, Vec<Delta>); 3] = [
            (Hook::Decide, vec![transfer(me, other), set_active(mill)]),
            (
                Hook::Produce,
                vec![
                    burn(me, Provenance::Production),
                    mint(me, Provenance::Production),
                    mint(me, Provenance::Endowment),
                    set_active(mill),
                ],
            ),
            (
                Hook::Upkeep,
                vec![burn(me, Provenance::Depreciation), set_active(mill)],
            ),
        ];
        for (hook, ds) in allowed {
            check(hook, &out(ds, Vec::new()), &w).expect("allowed");
        }
        // A pop burns with Consumption, not Production.
        let pop = |prov| HookOutput {
            actor: workers,
            deltas: vec![burn(Holder::Actor(workers), prov)],
            orders: Vec::new(),
        };
        check(Hook::Produce, &pop(Provenance::Consumption), &w).expect("a pop consumes");
        assert!(check(Hook::Produce, &pop(Provenance::Production), &w).is_err());
        // Orders: only decide's, only the actor's own, only in its class.
        let class = w.actor(mill).unwrap().class;
        let order = rustyecon_markets::Order {
            actor: mill,
            class,
            node: town,
            good: bread,
            qty: 1.0,
            side: Side::Sell,
        };
        check(Hook::Decide, &out(Vec::new(), vec![order]), &w).expect("its own order");
        let foreign = [
            (
                Hook::Decide,
                rustyecon_markets::Order {
                    actor: farm,
                    ..order
                },
            ),
            (
                Hook::Decide,
                rustyecon_markets::Order {
                    class: w.actor(workers).unwrap().class,
                    ..order
                },
            ),
            (Hook::Produce, order),
            (Hook::Upkeep, order),
        ];
        for (hook, o) in foreign {
            match check(hook, &out(Vec::new(), vec![o]), &w) {
                Err(RunErrorKind::ForeignOrder { actor, order }) => {
                    assert_eq!((actor, order), (mill, o));
                }
                other => panic!("{hook}: {o:?} was {other:?}"),
            }
        }
        // Nothing of a phase applies when one actor's output is foreign, even after a valid one.
        let mut s2 = s.clone();
        let hash = rustyecon_core::state_hash(&s2);
        let l = Ledger::open(&s2, &w).unwrap();
        let mut run = Run {
            w: &w,
            s: &mut s2,
            l,
            trace: Tracer(None),
        };
        let outputs = vec![
            out(vec![transfer(me, other)], Vec::new()),
            HookOutput {
                actor: workers,
                deltas: vec![StateDelta::AdvanceTick],
                orders: Vec::new(),
            },
        ];
        let f = run
            .apply_hooks(Hook::Decide, Phase::Decisions, outputs)
            .unwrap_err();
        assert!(matches!(f.kind, RunErrorKind::ForeignWrite { actor, .. } if actor == workers));
        assert_eq!(f.phase, Phase::Decisions);
        assert_eq!(rustyecon_core::state_hash(&s2), hash);
    }
}
