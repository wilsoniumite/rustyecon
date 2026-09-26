//! The `Sim` (docs/ENGINE.md §7.1, §7.5, §7.6): one run, stepped a tick at a time and read
//! through accessors that cannot change it.
//!
//! Every field is private, and no method hands out `&mut` to the state or the world (E4): the
//! only way anything enters a run is the tape (E1), and the only writer of the state is
//! `core::apply`, which the replay audit proves every tick. A failed step poisons the `Sim`
//! (E5): it refuses to step or checkpoint until it is rebuilt from a tape or a checkpoint.

use crate::error::{ResumeError, RunError, RunErrorKind};
use crate::report::{HoldingTotals, TickReport, Trace};
use crate::tick::run_tick;
use crate::{Checkpoint, Tape, World};
use rustyecon_agents::{ActorState, Agents, Cast};
use rustyecon_core::{
    resolve, state_hash, ActorId, CoreError, GoodId, Holder, Inventory, LoadError, LoadErrorKind,
    NodeId, ParamId, Phase, RunLedger, SimState,
};

/// Whether a `Sim` can step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// It can step.
    Ready,
    /// A step failed in this tick and phase; it refuses to step or checkpoint (E5).
    Poisoned {
        /// The tick that failed.
        tick: u64,
        /// The phase that failed.
        phase: Phase,
    },
}

/// One run.
#[derive(Debug, Clone)]
pub struct Sim {
    world: World,
    state: SimState<Agents>,
    cast: Cast,
    ledger: RunLedger,
    status: Status,
    hash: u64,
    last: Option<TickReport>,
}

impl Sim {
    /// A `Sim` at `state`: a new run's ledger opens on it, or a resumed run's continues.
    fn from_parts(
        world: World,
        state: SimState<Agents>,
        cast: Cast,
        run: Option<RunLedger>,
    ) -> Result<Sim, CoreError> {
        let hash = state_hash(&state);
        let ledger = match run {
            Some(run) => run,
            None => RunLedger::open(&state, &world)?,
        };
        Ok(Sim {
            world,
            state,
            cast,
            ledger,
            status: Status::Ready,
            hash,
            last: None,
        })
    }

    /// A run of `tape` from its genesis.
    pub fn new(tape: &Tape) -> Result<Sim, LoadError> {
        let (world, state) = resolve(tape)?;
        let cast = Cast::new(&world)?;
        // A resolved genesis fits its world, so the run's ledger opens on it; should it not,
        // that is the genesis's fault.
        Sim::from_parts(world, state, cast, None)
            .map_err(|e| LoadError::new("genesis", LoadErrorKind::Invalid(e.to_string())))
    }

    /// A run of `tape` from a checkpoint (§7.6, N11). The checkpoint must belong to the tape's
    /// world (`world_id`), agree with everything the tape fires before its tick (`prefix_id`),
    /// and fit the world's shape. So a checkpoint survives an edit dated at or after its tick,
    /// a dated `SetParam` to a new value included, and is refused after any edit to the world or
    /// to the past. Its format and digest, which covers its identity, its state and its run's
    /// ledger, were checked when it was decoded, and its fields cannot change since. The run
    /// continues the checkpoint's ledger, so its conservation audit is the uninterrupted run's
    /// (R2; amended at P0.9, O8).
    pub fn resume(tape: &Tape, cp: &Checkpoint) -> Result<Sim, ResumeError> {
        let (world, _) = resolve(tape).map_err(ResumeError::Load)?;
        if cp.world_id() != world.world_id {
            return Err(ResumeError::WrongWorld {
                tape: world.world_id,
                checkpoint: cp.world_id(),
            });
        }
        let tick = cp.state().tick();
        let prefix = world.prefix_id(tick);
        if cp.prefix_id() != prefix {
            return Err(ResumeError::WrongPrefix {
                tick,
                tape: prefix,
                checkpoint: cp.prefix_id(),
            });
        }
        cp.validate(&world).map_err(ResumeError::Invalid)?;
        let cast = Cast::new(&world).map_err(ResumeError::Load)?;
        Sim::from_parts(world, cp.state().clone(), cast, Some(cp.run().clone()))
            .map_err(ResumeError::Invalid)
    }

    fn poisoned(&self) -> Result<(), RunError> {
        match self.status {
            Status::Ready => Ok(()),
            Status::Poisoned { tick, phase } => Err(RunError {
                tick,
                phase,
                kind: RunErrorKind::Poisoned,
            }),
        }
    }

    fn advance(&mut self, trace: Option<&mut Trace>) -> Result<TickReport, RunError> {
        self.poisoned()?;
        let tick = self.state.tick();
        match run_tick(
            &self.world,
            &mut self.state,
            &self.cast,
            &mut self.ledger,
            trace.map(|t| &mut t.0),
        ) {
            Ok(report) => {
                self.hash = report.hash;
                self.last = Some(report.clone());
                Ok(report)
            }
            Err(f) => {
                self.status = Status::Poisoned {
                    tick,
                    phase: f.phase,
                };
                self.hash = state_hash(&self.state);
                Err(RunError {
                    tick,
                    phase: f.phase,
                    kind: f.kind,
                })
            }
        }
    }

    /// Run one tick (§7.2) and report it.
    pub fn step(&mut self) -> Result<TickReport, RunError> {
        self.advance(None)
    }

    /// Run one tick and return, with its report, every delta it applied, with its phase and the
    /// quantity it moved.
    pub fn step_traced(&mut self) -> Result<(TickReport, Trace), RunError> {
        let mut trace = Trace::default();
        let report = self.advance(Some(&mut trace))?;
        Ok((report, trace))
    }

    /// Step until the state's tick is `until`, calling `on_tick` after each step. Runs nothing
    /// when the tick is already at or past `until`.
    pub fn run_until(
        &mut self,
        until: u64,
        on_tick: &mut dyn FnMut(&TickReport),
    ) -> Result<(), RunError> {
        self.poisoned()?;
        while self.state.tick() < until {
            let report = self.step()?;
            on_tick(&report);
        }
        Ok(())
    }

    /// A checkpoint of the current state: its `world_id`, its `prefix_id`, the state and the
    /// run's ledger.
    pub fn checkpoint(&self) -> Result<Checkpoint, RunError> {
        self.poisoned()?;
        Checkpoint::of(&self.world, self.state.clone(), self.ledger.clone()).map_err(|e| RunError {
            tick: self.state.tick(),
            phase: Phase::Measure,
            kind: RunErrorKind::Core(e),
        })
    }

    /// Whether the `Sim` can step.
    pub fn status(&self) -> Status {
        self.status
    }

    /// The report of the last tick that succeeded, if any ran since the `Sim` was built.
    pub fn last_report(&self) -> Option<&TickReport> {
        self.last.as_ref()
    }

    /// The world, for key lookups and definitions.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// The state's tick: the next tick to run.
    pub fn tick(&self) -> u64 {
        self.state.tick()
    }

    /// The state's hash (cached; after a failed step, of the state as the failure left it).
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// The posted price of (node, good); `None` outside the book.
    pub fn price(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.state.price(n, g)
    }

    /// The price EMA of (node, good).
    pub fn ema(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.state.ema(n, g)
    }

    /// The last clearing's supply of (node, good).
    pub fn supply(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.state.supply(n, g)
    }

    /// The last clearing's feasible demand of (node, good).
    pub fn demand(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.state.demand(n, g)
    }

    /// A param's current value.
    pub fn param(&self, p: ParamId) -> Option<f64> {
        self.state.param(p)
    }

    /// One holder's inventory.
    pub fn holding(&self, h: Holder) -> Option<&Inventory> {
        self.state.holding(h)
    }

    /// Every holder that holds some of `g`, with its total, in `Holder` order.
    pub fn holdings_of(&self, g: GoodId) -> impl Iterator<Item = (Holder, f64)> + '_ {
        self.state
            .holdings()
            .iter()
            .filter(move |(_, inv)| !inv.lots(g).is_empty())
            .map(move |(h, inv)| (*h, inv.get(g)))
    }

    /// One actor's own state.
    pub fn actor_state(&self, a: ActorId) -> Option<&ActorState> {
        self.state.ext().get(&a)
    }

    /// An owned copy of every holder's totals, for another thread.
    pub fn observe_holdings(&self) -> HoldingTotals {
        let mut out = Vec::new();
        for (h, inv) in self.state.holdings() {
            for (g, q) in inv.goods() {
                out.push((*h, g, q));
            }
        }
        HoldingTotals(out)
    }
}

#[cfg(test)]
mod tests {
    //! The run's ledger across a resume (R2; docs/ENGINE.md §7.4 and §7.6; O8). These need the
    //! `Sim`'s own state, to leak into it the way a writer bug would, so they live here.

    use super::*;
    use rustyecon_core::{apply, Breach, Ledger, Provenance, StateDelta};

    const GATE: &str = include_str!("../../../tapes/gate.ron");

    /// Leak `q` coin to the first actor between two ticks, past every tick's ledger: the kind of
    /// writer bug the run's ledger exists for. No tick's audit sees it; the run's does.
    fn leak(sim: &mut Sim, q: f64) {
        let coin = sim.world.id_of::<GoodId>("coin").expect("the gate's coin");
        let to = Holder::Actor(sim.world.actors[0].id);
        let mut aside = Ledger::open(&sim.state, &sim.world).expect("a ledger opens");
        let mint = StateDelta::Mint {
            to,
            good: coin,
            qty: q,
            prov: Provenance::Event,
        };
        apply(
            &mut sim.state,
            &sim.world,
            Phase::Events,
            &[mint],
            &mut aside,
        )
        .expect("the leak applies");
        sim.hash = state_hash(&sim.state);
    }

    /// Run `t` with a leak of `q` before every tick until it stops or reaches `until`,
    /// resuming from a round-tripped checkpoint every `every` ticks. Returns where it ended and
    /// the error that stopped it.
    fn leaky(t: &Tape, q: f64, every: Option<u64>, until: u64) -> (u64, Option<RunError>) {
        let mut sim = Sim::new(t).expect("the gate loads");
        while sim.tick() < until {
            if let Some(k) = every.filter(|&k| sim.tick() > 0 && sim.tick().is_multiple_of(k)) {
                let cp = sim.checkpoint().expect("a checkpoint");
                let cp = Checkpoint::from_bytes(&cp.to_bytes()).expect("the bytes decode");
                sim = Sim::resume(t, &cp).expect("the checkpoint resumes");
                assert_eq!(sim.tick() % k, 0);
            }
            leak(&mut sim, q);
            if let Err(e) = sim.step() {
                return (sim.tick(), Some(e));
            }
        }
        (sim.tick(), None)
    }

    #[test]
    fn resumed_run_stops_where_the_uninterrupted_run_does() {
        // O8 (P0.9): the run's ledger lived in the Sim and not in the checkpoint, so a resumed
        // run audited from its checkpoint. A leak of 1e-9 coin a tick stopped an uninterrupted
        // run within a few ticks and passed all 2,080 when resumed every 2 ticks. Now the
        // checkpoint carries the run's ledger, and a run resumed every k ticks, for any k,
        // stops at the same tick with the same breach.
        let t = Tape::from_ron(GATE).expect("the gate parses");
        let leak = 1e-9;
        let (at, stopped) = leaky(&t, leak, None, 520);
        let e = stopped.expect("the uninterrupted run stops");
        let RunErrorKind::Core(CoreError::Conservation(b)) = &e.kind else {
            panic!("expected a conservation breach, got {e}");
        };
        let b: &Breach = b;
        let coin = Sim::new(&t).unwrap().world.id_of::<GoodId>("coin").unwrap();
        // The run's breach, over the run from genesis; no tick saw the leak.
        assert_eq!(
            (e.phase, b.good, b.since, b.tick),
            (Phase::Measure, coin, 0, e.tick)
        );
        assert!(
            b.drift > b.tol && b.drift <= leak * (e.tick + 1) as f64 * (1.0 + 1e-2),
            "{b}"
        );
        assert!(
            at > 1 && at < 100,
            "the leak stops the run early: tick {at}"
        );
        for every in [1, 2, 3, 7] {
            let (end, resumed) = leaky(&t, leak, Some(every), 520);
            assert_eq!(end, at, "resumed every {every} ticks");
            assert_eq!(resumed.as_ref(), Some(&e), "resumed every {every} ticks");
        }
        // With no leak, runs resumed at any cadence reach 520 unstopped.
        assert_eq!(leaky(&t, 0.0, Some(2), 520), (520, None));
    }
}
