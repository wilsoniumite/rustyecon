//! `apply`, the only writer of `SimState` (docs/ENGINE.md §2.4 and §2.5), salvaged from the core
//! arms of `v2p3: state/apply.rs`.
//!
//! Every good, node, holder and param is checked against the world before anything moves, in
//! every build profile (N2). Every value is finite with a clear sign bit. The phase rules of
//! §2.4 are enforced here; which actor may emit what is the engine's whitelist, not core's.
//! Each delta is atomic: on an error nothing of it has applied, though the deltas before it
//! have, so the caller stops the tick.

use crate::delta::{Phase, Provenance, StateDelta};
use crate::error::CoreError;
use crate::ext::Ext;
use crate::ids::{GoodId, Holder, NodeId};
use crate::inventory::{Amount, Inventory, Lot, TakeError};
use crate::ledger::{Ledger, ShortfallLine};
use crate::num::is_clean;
use crate::state::SimState;
use crate::world::{GoodDef, Life, World};

/// Apply `ds` in order, in `phase`, posting every mint, burn, spoilage and move to `l`. Returns
/// the quantity each delta moved: what a transfer delivered, a mint created, a burn destroyed,
/// or an ageing spoiled (summed over goods, ascending); 0 for the rest.
pub fn apply<E: Ext>(
    s: &mut SimState<E>,
    w: &World<E>,
    phase: Phase,
    ds: &[StateDelta<E>],
    l: &mut Ledger,
) -> Result<Vec<f64>, CoreError> {
    if l.width() != w.n_goods() {
        return Err(CoreError::Shape(format!(
            "a ledger over {} goods on a world of {}",
            l.width(),
            w.n_goods()
        )));
    }
    let mut moved = Vec::with_capacity(ds.len());
    for d in ds {
        moved.push(apply_one(s, w, phase, d, l)?);
    }
    Ok(moved)
}

fn apply_one<E: Ext>(
    s: &mut SimState<E>,
    w: &World<E>,
    phase: Phase,
    d: &StateDelta<E>,
    l: &mut Ledger,
) -> Result<f64, CoreError> {
    match d {
        StateDelta::SetPrice { node, good, price } => {
            let i = market(s, w, *node, *good)?;
            only(phase, Phase::Prices, "SetPrice")?;
            positive("price", *price)?;
            s.book.set_price(i, *price);
            Ok(0.0)
        }
        StateDelta::SetEma { node, good, ema } => {
            let i = market(s, w, *node, *good)?;
            only(phase, Phase::Prices, "SetEma")?;
            positive("ema", *ema)?;
            s.book.set_ema(i, *ema);
            Ok(0.0)
        }
        StateDelta::SetVolumes {
            node,
            good,
            supply,
            demand,
        } => {
            let i = market(s, w, *node, *good)?;
            only(phase, Phase::Clearing, "SetVolumes")?;
            clean("supply", *supply)?;
            clean("demand", *demand)?;
            s.book.set_volumes(i, *supply, *demand);
            Ok(0.0)
        }
        StateDelta::Transfer {
            from,
            to,
            good,
            amount,
        } => {
            good_def(w, *good)?;
            holder(s, w, *from)?;
            holder(s, w, *to)?;
            escrow_phase(phase, *from)?;
            escrow_phase(phase, *to)?;
            let lots = take(s, *from, *good, *amount, phase, None, l)?;
            let q = total(&lots);
            put(s, *to, *good, lots)?;
            drop_if_empty_escrow(s, *from);
            drop_if_empty_escrow(s, *to);
            l.moved(*good, q);
            Ok(q)
        }
        StateDelta::Mint {
            to,
            good,
            qty,
            prov,
        } => {
            let def = good_def(w, *good)?;
            holder(s, w, *to)?;
            escrow_phase(phase, *to)?;
            reserved(*prov)?;
            if def.life == Life::Instant && !matches!(phase, Phase::Events | Phase::Decisions) {
                return Err(CoreError::WrongPhase {
                    phase,
                    what: "a Mint of an Instant good",
                });
            }
            clean("mint quantity", *qty)?;
            let lot = Lot {
                qty: *qty,
                life: def.life.initial(),
            };
            put(s, *to, *good, vec![lot])?;
            drop_if_empty_escrow(s, *to);
            if *qty > 0.0 {
                l.declare(*good, *prov, *qty);
            }
            Ok(*qty)
        }
        StateDelta::Burn {
            from,
            good,
            amount,
            prov,
        } => {
            good_def(w, *good)?;
            holder(s, w, *from)?;
            escrow_phase(phase, *from)?;
            reserved(*prov)?;
            let lots = take(s, *from, *good, *amount, phase, Some(*prov), l)?;
            let q = total(&lots);
            drop_if_empty_escrow(s, *from);
            if q > 0.0 {
                l.declare(*good, *prov, -q);
            }
            Ok(q)
        }
        StateDelta::SetParam { param, value } => {
            let def = w
                .registry
                .get(*param)
                .ok_or(CoreError::UnknownParam(*param))?;
            only(phase, Phase::Events, "SetParam")?;
            if def.fixed {
                return Err(CoreError::FixedParam(*param));
            }
            clean("param value", *value)?;
            let slot = s
                .params
                .get_mut(param.idx())
                .ok_or(CoreError::UnknownParam(*param))?;
            *slot = *value;
            Ok(0.0)
        }
        StateDelta::Age { holder: h } => {
            holder(s, w, *h)?;
            escrow_phase(phase, *h)?;
            only(phase, Phase::Upkeep, "Age")?;
            let spoiled = match s.holdings.get_mut(h) {
                Some(inv) => inv.age(),
                None => Vec::new(),
            };
            let mut sum = 0.0;
            for (g, q) in spoiled {
                l.declare(g, Provenance::Spoilage, -q);
                sum += q;
            }
            Ok(sum)
        }
        StateDelta::Actor(d) => {
            E::apply(&mut s.ext, d)?;
            Ok(0.0)
        }
        StateDelta::AdvanceTick => {
            only(phase, Phase::Prices, "AdvanceTick")?;
            s.tick = s
                .tick
                .checked_add(1)
                .ok_or_else(|| CoreError::Shape("the tick counter overflowed".into()))?;
            Ok(0.0)
        }
    }
}

fn only(phase: Phase, allowed: Phase, what: &'static str) -> Result<(), CoreError> {
    if phase == allowed {
        Ok(())
    } else {
        Err(CoreError::WrongPhase { phase, what })
    }
}

fn escrow_phase(phase: Phase, h: Holder) -> Result<(), CoreError> {
    if matches!(h, Holder::Escrow(..)) && phase != Phase::Settlement {
        return Err(CoreError::WrongPhase {
            phase,
            what: "an escrow holder",
        });
    }
    Ok(())
}

fn reserved(prov: Provenance) -> Result<(), CoreError> {
    if prov == Provenance::Spoilage {
        return Err(CoreError::ReservedProvenance(prov));
    }
    Ok(())
}

fn clean(what: &'static str, v: f64) -> Result<(), CoreError> {
    if is_clean(v) {
        Ok(())
    } else {
        Err(CoreError::BadValue { what, value: v })
    }
}

fn positive(what: &'static str, v: f64) -> Result<(), CoreError> {
    if is_clean(v) && v > 0.0 {
        Ok(())
    } else {
        Err(CoreError::BadValue { what, value: v })
    }
}

fn good_def<E: Ext>(w: &World<E>, g: GoodId) -> Result<&GoodDef, CoreError> {
    w.good(g).ok_or(CoreError::UnknownGood(g))
}

/// The book index of a market: the node and good exist and the good is not a currency.
fn market<E: Ext>(
    s: &SimState<E>,
    w: &World<E>,
    node: NodeId,
    good: GoodId,
) -> Result<usize, CoreError> {
    w.node(node).ok_or(CoreError::UnknownNode(node))?;
    good_def(w, good)?;
    if w.is_currency(good) {
        return Err(CoreError::NoMarket { node, good });
    }
    s.book
        .index(node, good)
        .ok_or_else(|| CoreError::Shape("the book does not fit the world".into()))
}

/// A holder the world knows. A declared actor must also hold an inventory in the state.
fn holder<E: Ext>(s: &SimState<E>, w: &World<E>, h: Holder) -> Result<(), CoreError> {
    let known = w.is_holder(h)
        && match h {
            Holder::Actor(_) => s.holdings.contains_key(&h),
            Holder::Escrow(..) => true,
        };
    if known {
        Ok(())
    } else {
        Err(CoreError::UnknownHolder(h))
    }
}

fn take<E: Ext>(
    s: &mut SimState<E>,
    h: Holder,
    g: GoodId,
    a: Amount,
    phase: Phase,
    prov: Option<Provenance>,
    l: &mut Ledger,
) -> Result<Vec<Lot>, CoreError> {
    let tick = s.tick;
    let result = match s.holdings.get_mut(&h) {
        Some(inv) => inv.take(g, a),
        None => Inventory::new().take(g, a),
    };
    match result {
        Ok(lots) => Ok(lots),
        Err(TakeError::Shortfall(sf)) => {
            let line = ShortfallLine {
                tick,
                phase,
                holder: h,
                good: g,
                requested: sf.requested,
                held: sf.held,
                prov,
            };
            l.record_shortfall(line.clone());
            Err(CoreError::Shortfall(line))
        }
        Err(TakeError::Invalid(q)) => Err(CoreError::BadValue {
            what: "quantity taken",
            value: q,
        }),
    }
}

fn put<E: Ext>(s: &mut SimState<E>, h: Holder, g: GoodId, lots: Vec<Lot>) -> Result<(), CoreError> {
    s.holdings.entry(h).or_default().put(g, lots)
}

fn drop_if_empty_escrow<E: Ext>(s: &mut SimState<E>, h: Holder) {
    if matches!(h, Holder::Escrow(..)) && s.holdings.get(&h).is_some_and(Inventory::is_empty) {
        s.holdings.remove(&h);
    }
}

fn total(lots: &[Lot]) -> f64 {
    lots.iter().fold(0.0, |acc, lot| acc + lot.qty)
}

#[cfg(test)]
mod tests {
    //! The ledger through `apply` (salvaged from `v2p3: tests/test_04:42-121`), the phase rules
    //! of docs/ENGINE.md §2.4, and typed holders (defect 10).

    use super::*;
    use crate::ids::{ActorId, DeskId, PopId};
    use crate::testkit::{self, good, held, holder, tick, write_direct};
    use crate::NoExt;

    const ALL_PHASES: [Phase; 8] = [
        Phase::Events,
        Phase::Decisions,
        Phase::Clearing,
        Phase::Settlement,
        Phase::Production,
        Phase::Upkeep,
        Phase::Prices,
        Phase::Measure,
    ];

    fn one(
        s: &mut SimState<NoExt>,
        w: &World<NoExt>,
        phase: Phase,
        d: StateDelta<NoExt>,
    ) -> Result<f64, CoreError> {
        let mut l = Ledger::open(s, w)?;
        apply(s, w, phase, &[d], &mut l).map(|m| m[0])
    }

    #[test]
    fn untagged_creation_is_caught_through_apply() {
        let (w, mut s) = testkit::load();
        let (pensioners, grain) = (holder(&w, "pensioners"), good(&w, "grain"));
        // Units that appear with no provenance: every Mint needs one, so this is a direct write.
        let err = tick(&mut s.clone(), &w, |s, _| {
            write_direct(s, pensioners, grain, 5.0);
            Ok(())
        })
        .unwrap_err();
        assert!(matches!(err, CoreError::Conservation(_)));
        // The same units, declared as production, reconcile.
        tick(&mut s, &w, |s, l| {
            let mint = StateDelta::Mint {
                to: pensioners,
                good: grain,
                qty: 5.0,
                prov: Provenance::Production,
            };
            apply(s, &w, Phase::Production, &[mint], l).map(drop)
        })
        .expect("a declared mint conserves");
        assert_eq!(held(&s, pensioners, grain), 5.0);
    }

    #[test]
    fn shortfall_is_captured_and_not_silently_clamped() {
        let (w, mut s) = testkit::load();
        let (pensioners, grain) = (holder(&w, "pensioners"), good(&w, "grain"));
        write_direct(&mut s, pensioners, grain, 2.0);
        let mut l = Ledger::open(&s, &w).unwrap();
        let burn = StateDelta::Burn {
            from: pensioners,
            good: grain,
            amount: Amount::Qty(7.0),
            prov: Provenance::Consumption,
        };
        let err = apply(&mut s, &w, Phase::Production, &[burn], &mut l).unwrap_err();
        // The shortfall is a ledger line and an error that stops the tick; July clamped to
        // what was there and ran on.
        let CoreError::Shortfall(line) = err else {
            panic!("expected a shortfall");
        };
        assert_eq!((line.requested, line.held), (7.0, 2.0));
        assert_eq!(l.shortfall(), Some(&line));
        assert_eq!(held(&s, pensioners, grain), 2.0, "nothing was burned");
        assert!(l.lines().is_empty());
    }

    #[test]
    fn atomic_transfer_cannot_mint() {
        let (w, mut s) = testkit::load();
        let (farm, mill, pensioners, grain) = (
            holder(&w, "farm"),
            holder(&w, "mill"),
            holder(&w, "pensioners"),
            good(&w, "grain"),
        );
        // A short source moves nothing: no half of the transfer lands.
        let before = s.clone();
        let over = StateDelta::Transfer {
            from: farm,
            to: mill,
            good: grain,
            amount: Amount::Qty(16.5),
        };
        assert!(matches!(
            one(&mut s, &w, Phase::Decisions, over),
            Err(CoreError::Shortfall(_))
        ));
        assert_eq!(s, before);
        // A transfer moves exactly what it takes, and declares nothing.
        let audit = tick(&mut s, &w, |s, l| {
            let ds = [
                StateDelta::Transfer {
                    from: farm,
                    to: mill,
                    good: grain,
                    amount: Amount::Qty(6.25),
                },
                StateDelta::Transfer {
                    from: farm,
                    to: pensioners,
                    good: grain,
                    amount: Amount::All,
                },
            ];
            let moved = apply(s, &w, Phase::Decisions, &ds, l)?;
            assert_eq!(moved, vec![6.25, 9.75]);
            Ok(())
        })
        .unwrap();
        assert!(audit.lines.is_empty());
        assert_eq!(audit.max_drift, 0.0);
        assert_eq!(held(&s, farm, grain), 0.0);
        assert_eq!(held(&s, mill, grain) + held(&s, pensioners, grain), 16.0);
    }

    #[test]
    fn transfers_keep_lot_lives() {
        let (w, mut s) = testkit::load();
        let (farm, workers, bread) = (holder(&w, "farm"), holder(&w, "workers"), good(&w, "bread"));
        let t = StateDelta::Transfer {
            from: farm,
            to: workers,
            good: bread,
            amount: Amount::Qty(1.0),
        };
        one(&mut s, &w, Phase::Decisions, t).unwrap();
        let lots = s.holding(workers).unwrap().lots(bread);
        assert_eq!(lots.len(), 1);
        assert_eq!(
            lots[0].life,
            Some(3),
            "the genesis lot's life moved with it"
        );
    }

    #[test]
    fn desk_and_pop_sharing_a_number_are_distinct_holders() {
        // July keyed currency flows by a bare (u32, u32), so a building and a recipe instance
        // with one number shared a slot and the wrong one was paid (defect 10).
        let (w, mut s) = testkit::load();
        let desk = Holder::Actor(ActorId::Desk(DeskId(0)));
        let pop = Holder::Actor(ActorId::Pop(PopId(0)));
        assert_eq!(w.key_of(ActorId::Desk(DeskId(0))).unwrap().as_str(), "farm");
        assert_eq!(
            w.key_of(ActorId::Pop(PopId(0))).unwrap().as_str(),
            "pensioners"
        );
        let (mill, coin) = (holder(&w, "mill"), good(&w, "coin"));
        let pay = |to, q| StateDelta::Transfer {
            from: mill,
            to,
            good: coin,
            amount: Amount::Qty(q),
        };
        tick(&mut s, &w, |s, l| {
            apply(
                s,
                &w,
                Phase::Settlement,
                &[pay(desk, 3.0), pay(pop, 5.0)],
                l,
            )
            .map(drop)
        })
        .unwrap();
        assert_eq!(held(&s, desk, coin), 103.0);
        assert_eq!(held(&s, pop, coin), 5.0);
    }

    #[test]
    fn escrow_outside_settlement_is_an_error() {
        let (w, mut s) = testkit::load();
        let (farm, grain, coin) = (holder(&w, "farm"), good(&w, "grain"), good(&w, "coin"));
        let town = w.id_of("town").unwrap();
        let escrow = Holder::Escrow(town, grain);
        let into = StateDelta::Transfer {
            from: farm,
            to: escrow,
            good: grain,
            amount: Amount::Qty(4.0),
        };
        let out = StateDelta::Transfer {
            from: escrow,
            to: farm,
            good: grain,
            amount: Amount::All,
        };
        for phase in ALL_PHASES.into_iter().filter(|&p| p != Phase::Settlement) {
            assert_eq!(
                one(&mut s.clone(), &w, phase, into.clone()),
                Err(CoreError::WrongPhase {
                    phase,
                    what: "an escrow holder"
                })
            );
        }
        let mint = StateDelta::Mint {
            to: escrow,
            good: grain,
            qty: 1.0,
            prov: Provenance::Production,
        };
        assert!(matches!(
            one(&mut s.clone(), &w, Phase::Production, mint),
            Err(CoreError::WrongPhase { .. })
        ));
        // A currency has no market, so it has no escrow.
        let bad = StateDelta::Transfer {
            from: farm,
            to: Holder::Escrow(town, coin),
            good: coin,
            amount: Amount::Qty(1.0),
        };
        assert_eq!(
            one(&mut s.clone(), &w, Phase::Settlement, bad),
            Err(CoreError::UnknownHolder(Holder::Escrow(town, coin)))
        );
        // In settlement it works; an escrow left over fails the close, and an emptied one is
        // dropped at once.
        let mut left = s.clone();
        let err = tick(&mut left, &w, |s, l| {
            apply(s, &w, Phase::Settlement, std::slice::from_ref(&into), l).map(drop)
        })
        .unwrap_err();
        assert_eq!(
            err,
            CoreError::EscrowLeft {
                node: town,
                good: grain
            }
        );
        let audit = tick(&mut s, &w, |s, l| {
            apply(s, &w, Phase::Settlement, &[into.clone(), out.clone()], l).map(drop)
        })
        .unwrap();
        assert!(s.holding(escrow).is_none());
        assert_eq!(audit.max_drift, 0.0);
    }

    #[test]
    fn instant_mint_outside_phases_0_and_1_is_an_error() {
        let (w, mut s) = testkit::load();
        let (workers, labour) = (holder(&w, "workers"), good(&w, "labour"));
        let mint = StateDelta::Mint {
            to: workers,
            good: labour,
            qty: 8.0,
            prov: Provenance::Endowment,
        };
        for phase in ALL_PHASES {
            let r = one(&mut s.clone(), &w, phase, mint.clone());
            if matches!(phase, Phase::Events | Phase::Decisions) {
                assert_eq!(r, Ok(8.0));
            } else {
                assert_eq!(
                    r,
                    Err(CoreError::WrongPhase {
                        phase,
                        what: "a Mint of an Instant good"
                    })
                );
            }
        }
        // Minted in phase 1 with life Some(0), it is there for phases 2 to 4 and spoils at 5a.
        let audit = tick(&mut s, &w, |s, l| {
            apply(s, &w, Phase::Decisions, std::slice::from_ref(&mint), l)?;
            assert_eq!(s.holding(workers).unwrap().lots(labour)[0].life, Some(0));
            let spoiled = apply(
                s,
                &w,
                Phase::Upkeep,
                &[StateDelta::Age { holder: workers }],
                l,
            )?;
            assert_eq!(spoiled, vec![8.0]);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            audit.lines,
            vec![
                (labour, Provenance::Spoilage, -8.0),
                (labour, Provenance::Endowment, 8.0)
            ]
        );
        assert_eq!(held(&s, workers, labour), 0.0);
    }

    #[test]
    fn book_writes_outside_their_phase_are_errors() {
        let (w, s) = testkit::load();
        let town = w.id_of("town").unwrap();
        let (grain, coin) = (good(&w, "grain"), good(&w, "coin"));
        let rate = w.id_of("rate.grain").unwrap();
        let cases: Vec<(StateDelta<NoExt>, Phase)> = vec![
            (
                StateDelta::SetPrice {
                    node: town,
                    good: grain,
                    price: 1.5,
                },
                Phase::Prices,
            ),
            (
                StateDelta::SetEma {
                    node: town,
                    good: grain,
                    ema: 1.5,
                },
                Phase::Prices,
            ),
            (StateDelta::AdvanceTick, Phase::Prices),
            (
                StateDelta::SetVolumes {
                    node: town,
                    good: grain,
                    supply: 3.0,
                    demand: 2.0,
                },
                Phase::Clearing,
            ),
            (
                StateDelta::SetParam {
                    param: rate,
                    value: 1.0,
                },
                Phase::Events,
            ),
            (
                StateDelta::Age {
                    holder: holder(&w, "farm"),
                },
                Phase::Upkeep,
            ),
        ];
        for (d, allowed) in cases {
            for phase in ALL_PHASES {
                let r = one(&mut s.clone(), &w, phase, d.clone());
                if phase == allowed {
                    assert!(r.is_ok(), "{} in {phase}", d.name());
                } else {
                    assert!(
                        matches!(r, Err(CoreError::WrongPhase { .. })),
                        "{} in {phase}: {r:?}",
                        d.name()
                    );
                }
            }
        }
        // A currency has no market; prices and EMAs are positive; volumes are clean.
        let price = |node, good, price| StateDelta::SetPrice { node, good, price };
        assert_eq!(
            one(&mut s.clone(), &w, Phase::Prices, price(town, coin, 1.0)),
            Err(CoreError::NoMarket {
                node: town,
                good: coin
            })
        );
        for bad in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                one(&mut s.clone(), &w, Phase::Prices, price(town, grain, bad)),
                Err(CoreError::BadValue { .. })
            ));
        }
        let volumes = StateDelta::SetVolumes {
            node: town,
            good: grain,
            supply: -0.0,
            demand: 1.0,
        };
        assert!(matches!(
            one(&mut s.clone(), &w, Phase::Clearing, volumes),
            Err(CoreError::BadValue { .. })
        ));
        assert_eq!(
            one(
                &mut s.clone(),
                &w,
                Phase::Prices,
                price(NodeId(9), grain, 1.0)
            ),
            Err(CoreError::UnknownNode(NodeId(9)))
        );
    }

    #[test]
    fn set_param_respects_the_registry() {
        let (w, s) = testkit::load();
        let fixed = w.id_of("ledger.rel_flow").unwrap();
        assert!(w.registry.get(fixed).unwrap().fixed);
        let set = |param, value| StateDelta::SetParam { param, value };
        assert_eq!(
            one(&mut s.clone(), &w, Phase::Events, set(fixed, 1.0)),
            Err(CoreError::FixedParam(fixed))
        );
        let live = w.id_of("rate.grain").unwrap();
        for bad in [-0.0, f64::NAN, -2.0] {
            assert!(matches!(
                one(&mut s.clone(), &w, Phase::Events, set(live, bad)),
                Err(CoreError::BadValue { .. })
            ));
        }
        let ghost = crate::ids::ParamId(77);
        assert_eq!(
            one(&mut s.clone(), &w, Phase::Events, set(ghost, 1.0)),
            Err(CoreError::UnknownParam(ghost))
        );
        let mut t = s.clone();
        one(&mut t, &w, Phase::Events, set(live, 7.5)).unwrap();
        assert_eq!(t.param(live), Some(7.5));
    }

    #[test]
    fn unknown_holders_and_reserved_provenance_are_errors() {
        let (w, s) = testkit::load();
        let (farm, coin) = (holder(&w, "farm"), good(&w, "coin"));
        let ghost = Holder::Actor(ActorId::Pop(PopId(9)));
        let pay = StateDelta::Transfer {
            from: farm,
            to: ghost,
            good: coin,
            amount: Amount::Qty(1.0),
        };
        assert_eq!(
            one(&mut s.clone(), &w, Phase::Decisions, pay),
            Err(CoreError::UnknownHolder(ghost))
        );
        let rot = StateDelta::Burn {
            from: farm,
            good: coin,
            amount: Amount::Qty(1.0),
            prov: Provenance::Spoilage,
        };
        assert_eq!(
            one(&mut s.clone(), &w, Phase::Production, rot),
            Err(CoreError::ReservedProvenance(Provenance::Spoilage))
        );
    }
}
