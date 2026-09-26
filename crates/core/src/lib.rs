//! The substrate every other crate builds on (docs/ENGINE.md §2): ids and keys, goods, the
//! clock and time units, the param registry, inventories of lots with lives, the state and the
//! extension seam, the deltas and the single pass that applies them, the conservation ledger,
//! the state hash, checkpoints, the tape's schema and its resolver, and the one module for
//! transcendentals. It knows nothing about agent rules, so the oracle, the markets and the
//! agents share its types without sharing logic, and it depends on no workspace crate (N14).
//!
//! Salvaged from the July engine (tag `july-v2-phase-3`) with its defects fixed as it moved:
//! typed holders for defect 10, coalescing lots for N9, an atomic transfer and shortfalls that
//! stop the tick for defect 9 and N3, checks in every profile for N2, sorted events for N1,
//! checkpoints that name their world for N11, and registered tolerances with no absolute
//! term for A12. It holds no global state and does no I/O (E2).

pub mod apply;
pub mod checkpoint;
pub mod clock;
pub mod delta;
pub mod error;
pub mod ext;
pub mod hash;
pub mod ids;
pub mod inventory;
pub mod ledger;
pub mod num;
pub mod registry;
pub mod state;
pub mod tape;
pub mod units;
pub mod world;

pub use apply::apply;
pub use checkpoint::{Checkpoint, CHECKPOINT_FORMAT};
pub use clock::{Clock, ClockError, Date, DateError};
pub use delta::{Phase, Provenance, StateDelta};
pub use error::{CheckpointError, CoreError, LoadError, LoadErrorKind};
pub use ext::{Ext, Never, NoExt};
pub use hash::{fnv1a_64, state_hash};
pub use ids::{
    ActorId, ActorKind, ChannelId, ClassId, DeskId, EventId, GoodId, Holder, InvalidKey, Key,
    NodeId, ParamId, PopId,
};
pub use inventory::{Amount, Inventory, Lot, Shortfall, TakeError, Taken};
pub use ledger::{Breach, Ledger, RunAudit, RunLedger, ShortfallLine, TickAudit};
pub use num::NumError;
pub use registry::{Basis, ParamDef, Params, Registry};
pub use state::{MarketBook, Quote, SimState};
pub use tape::{resolve, ParamUse, Resolver, Tape, SCHEMA};
pub use units::{
    CompoundPerYear, Dimensionless, FlowPerYear, FractionPerYear, RatePerYear, Unit, UnitKind,
    Years,
};
pub use world::{
    ActorDecl, ChannelDef, Firing, GoodDef, KeyIndex, Keyed, Life, MarketConfig, NodeDef, OneSided,
    PriceRule, Recurring, Schedule, ScheduleParam, Tolerances, World,
};

#[cfg(test)]
pub(crate) mod testkit {
    //! Fixtures for core's own tests: one small tape, read with the empty extension and with a
    //! counting extension that exercises the seam.

    use crate::delta::{Phase, StateDelta};
    use crate::error::{CoreError, LoadError, LoadErrorKind};
    use crate::ext::Ext;
    use crate::ids::{ActorId, GoodId, Holder, Key};
    use crate::ledger::{Ledger, TickAudit};
    use crate::state::SimState;
    use crate::tape::{resolve, Resolver, Tape};
    use crate::world::{ActorDecl, World};
    use crate::NoExt;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    /// The fixture tape. Its actors have the spec `()`.
    pub const FIXTURE: &str = include_str!("../testdata/core.ron");

    pub fn tape() -> Tape<NoExt> {
        Tape::from_ron(FIXTURE).expect("the fixture parses")
    }

    pub fn load() -> (World<NoExt>, SimState<NoExt>) {
        resolve(&tape()).expect("the fixture resolves")
    }

    /// The fixture for the counting extension: each spec is `(start: n)`.
    pub fn ext_text() -> String {
        FIXTURE.replace("spec: ()", "spec: (start: 3)")
    }

    pub fn load_ext() -> (World<TestExt>, SimState<TestExt>) {
        let t: Tape<TestExt> = Tape::from_ron(&ext_text()).expect("the ext fixture parses");
        resolve(&t).expect("the ext fixture resolves")
    }

    /// Resolve a variant of the fixture made by editing its text.
    pub fn load_text(text: &str) -> Result<(World<NoExt>, SimState<NoExt>), LoadError> {
        resolve(&Tape::<NoExt>::from_ron(text)?)
    }

    /// The error a text variant of the fixture fails with.
    pub fn load_err(text: &str) -> LoadError {
        match load_text(text) {
            Ok(_) => panic!("the variant was expected to fail to load"),
            Err(e) => e,
        }
    }

    /// Replace exactly one occurrence of `from` in the fixture.
    pub fn edit(from: &str, to: &str) -> String {
        assert_eq!(FIXTURE.matches(from).count(), 1, "{from:?} must occur once");
        FIXTURE.replacen(from, to, 1)
    }

    pub fn holder<E: Ext>(w: &World<E>, key: &str) -> Holder {
        Holder::Actor(w.id_of::<ActorId>(key).expect("a fixture actor"))
    }

    pub fn good<E: Ext>(w: &World<E>, key: &str) -> GoodId {
        w.id_of::<GoodId>(key).expect("a fixture good")
    }

    pub fn held<E: Ext>(s: &SimState<E>, h: Holder, g: GoodId) -> f64 {
        s.holding(h).map_or(0.0, |inv| inv.get(g))
    }

    /// Apply deltas in a phase under a throwaway ledger, expecting success.
    pub fn apply_ok<E: Ext>(s: &mut SimState<E>, w: &World<E>, phase: Phase, ds: &[StateDelta<E>]) {
        let mut l = Ledger::open(s, w).expect("the ledger opens");
        crate::apply(s, w, phase, ds, &mut l).expect("the deltas apply");
    }

    /// Open a ledger, run `body` on the state, and close it.
    pub fn tick<E: Ext>(
        s: &mut SimState<E>,
        w: &World<E>,
        body: impl FnOnce(&mut SimState<E>, &mut Ledger) -> Result<(), CoreError>,
    ) -> Result<TickAudit, CoreError> {
        let mut l = Ledger::open(s, w)?;
        body(s, &mut l)?;
        l.close(s, w)
    }

    /// Whether the exact sum of `terms` is zero. The terms are grown into a Shewchuk expansion
    /// by TwoSum, with zeros dropped: its components do not overlap and sum exactly to the
    /// terms' sum, so that sum is zero exactly when no component is left. No float fold rounds.
    pub fn exactly_zero(terms: impl IntoIterator<Item = f64>) -> bool {
        let mut expansion: Vec<f64> = Vec::new();
        for t in terms {
            let mut q = t;
            let mut grown = Vec::with_capacity(expansion.len() + 1);
            for &c in &expansion {
                let (s, e) = crate::num::two_sum(q, c);
                if e != 0.0 {
                    grown.push(e);
                }
                q = s;
            }
            if q != 0.0 {
                grown.push(q);
            }
            expansion = grown;
        }
        expansion.is_empty()
    }

    /// Write a quantity straight into a holding, bypassing `apply` and the ledger.
    pub fn write_direct<E: Ext>(s: &mut SimState<E>, h: Holder, g: GoodId, delta: f64) {
        let inv = s.holdings.entry(h).or_default();
        let raw = inv.raw_mut();
        match raw.iter_mut().find(|(good, _)| *good == g) {
            Some((_, lots)) => lots.last_mut().expect("a lot").qty += delta,
            None => {
                raw.push((
                    g,
                    vec![crate::inventory::Lot {
                        qty: delta,
                        life: None,
                    }],
                ));
                raw.sort_by_key(|(good, _)| *good);
            }
        }
    }

    /// A counting extension: each actor has a counter, a tape action or an actor bumps it.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TestExt;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct RawCounter {
        pub start: u64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct RawBump {
        pub actor: Key,
        pub by: u64,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Bump {
        pub actor: ActorId,
        pub by: u64,
    }

    impl Ext for TestExt {
        type State = BTreeMap<ActorId, u64>;
        type Delta = Bump;
        type RawActor = RawCounter;
        type Actor = u64;
        type RawAction = RawBump;

        fn resolve_actor(raw: &RawCounter, _: &mut Resolver<'_>) -> Result<u64, LoadError> {
            Ok(raw.start)
        }
        fn resolve_action(raw: &RawBump, r: &mut Resolver<'_>) -> Result<Bump, LoadError> {
            if raw.by == 0 {
                return Err(r.error("by", LoadErrorKind::Invalid("a bump of zero".into())));
            }
            Ok(Bump {
                actor: r.actor(&raw.actor, "actor")?,
                by: raw.by,
            })
        }
        fn genesis(actors: &[ActorDecl<u64>]) -> Result<Self::State, LoadError> {
            Ok(actors.iter().map(|a| (a.id, a.spec)).collect())
        }
        fn apply(s: &mut Self::State, d: &Bump) -> Result<(), CoreError> {
            let c = s
                .get_mut(&d.actor)
                .ok_or(CoreError::UnknownActor(d.actor))?;
            *c = c
                .checked_add(d.by)
                .ok_or_else(|| CoreError::Ext("overflow".into()))?;
            Ok(())
        }
        fn owner(d: &Bump) -> Option<ActorId> {
            Some(d.actor)
        }
        fn validate(s: &Self::State, actors: &[ActorDecl<u64>]) -> Result<(), CoreError> {
            let ok = s.len() == actors.len() && actors.iter().all(|a| s.contains_key(&a.id));
            if ok {
                Ok(())
            } else {
                Err(CoreError::Shape("counters do not match the actors".into()))
            }
        }
        fn canonical_actor(_: &mut RawCounter) {}
        fn canonical_action(_: &mut RawBump) {}
    }
}

#[cfg(test)]
mod tests {
    use crate::testkit;
    use crate::*;

    #[test]
    fn core_has_no_workspace_dependencies() {
        // N14: July's core types needed its agents. Core's manifest names no workspace crate.
        for line in include_str!("../Cargo.toml").lines().map(str::trim) {
            if line.starts_with('#') || line == r#"name = "rustyecon-core""# {
                continue;
            }
            assert!(
                !line.contains("rustyecon-"),
                "core depends on a workspace crate: {line}"
            );
        }
        // A tape with no behaviour resolves, fires its events through apply and closes its
        // ledger every tick.
        let (w, mut s) = testkit::load();
        let everyone: Vec<StateDelta<NoExt>> = w
            .actors
            .iter()
            .map(|a| StateDelta::Age {
                holder: Holder::Actor(a.id),
            })
            .collect();
        let (mut fired, mut spoiled) = (Vec::new(), 0.0);
        for t in 0..160 {
            let mut l = Ledger::open(&s, &w).unwrap();
            for f in w.schedule.fire(t) {
                fired.push((t, w.key_of(f.event).unwrap().to_string()));
                apply(&mut s, &w, Phase::Events, &[f.action], &mut l).unwrap();
            }
            let moved = apply(&mut s, &w, Phase::Upkeep, &everyone, &mut l).unwrap();
            spoiled += moved.iter().sum::<f64>();
            apply(
                &mut s,
                &w,
                Phase::Prices,
                &[StateDelta::AdvanceTick],
                &mut l,
            )
            .unwrap();
            let audit = l.close(&s, &w).unwrap();
            assert!(audit.max_margin <= 1.0);
        }
        assert_eq!(s.tick(), 160);
        let fired: Vec<(u64, &str)> = fired.iter().map(|(t, k)| (*t, k.as_str())).collect();
        assert_eq!(
            fired,
            vec![
                (51, "pension"),
                (73, "grain.gift"),
                (103, "pension"),
                (103, "rate.up"),
                (155, "pension")
            ]
        );
        let (pensioners, coin, grain) = (
            testkit::holder(&w, "pensioners"),
            testkit::good(&w, "coin"),
            testkit::good(&w, "grain"),
        );
        assert_eq!(testkit::held(&s, pensioners, coin), 15.0);
        assert_eq!(testkit::held(&s, pensioners, grain), 2.0);
        assert_eq!(s.param(w.id_of("rate.grain").unwrap()), Some(10.4));
        assert_eq!(spoiled, 7.0, "the genesis bread spoiled at its life's end");
    }

    #[test]
    fn core_types_cross_threads() {
        // E3: an engine built on these can run on a worker thread.
        fn ok<T: Send + Sync + 'static>() {}
        ok::<World<NoExt>>();
        ok::<SimState<NoExt>>();
        ok::<Tape<NoExt>>();
        ok::<Checkpoint<NoExt>>();
        ok::<Ledger>();
        ok::<TickAudit>();
        ok::<CoreError>();
        ok::<LoadError>();
        ok::<CheckpointError>();
        ok::<NumError>();
    }
}
