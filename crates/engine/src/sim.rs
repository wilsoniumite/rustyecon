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
    resolve, state_hash, ActorId, CheckpointError, GoodId, Holder, Inventory, LoadError, NodeId,
    ParamId, Phase, SimState, CHECKPOINT_FORMAT,
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
    status: Status,
    hash: u64,
    last: Option<TickReport>,
}

impl Sim {
    fn from_parts(world: World, state: SimState<Agents>, cast: Cast) -> Sim {
        let hash = state_hash(&state);
        Sim {
            world,
            state,
            cast,
            status: Status::Ready,
            hash,
            last: None,
        }
    }

    /// A run of `tape` from its genesis.
    pub fn new(tape: &Tape) -> Result<Sim, LoadError> {
        let (world, state) = resolve(tape)?;
        let cast = Cast::new(&world)?;
        Ok(Sim::from_parts(world, state, cast))
    }

    /// A run of `tape` from a checkpoint (§7.6, N11). The checkpoint must be of this format,
    /// belong to the tape's world (`world_id`), agree with everything the tape fires before its
    /// tick (`prefix_id`), and fit the world's shape. So a checkpoint survives an edit dated at
    /// or after its tick, and is refused after any edit to the world or to the past.
    pub fn resume(tape: &Tape, cp: &Checkpoint) -> Result<Sim, ResumeError> {
        let (world, _) = resolve(tape).map_err(ResumeError::Load)?;
        if cp.format != CHECKPOINT_FORMAT {
            return Err(ResumeError::Checkpoint(CheckpointError::Format {
                found: cp.format,
                expected: CHECKPOINT_FORMAT,
            }));
        }
        if cp.world_id != world.world_id {
            return Err(ResumeError::WrongWorld {
                tape: world.world_id,
                checkpoint: cp.world_id,
            });
        }
        let tick = cp.state.tick();
        let prefix = world.prefix_id(tick);
        if cp.prefix_id != prefix {
            return Err(ResumeError::WrongPrefix {
                tick,
                tape: prefix,
                checkpoint: cp.prefix_id,
            });
        }
        cp.validate(&world).map_err(ResumeError::Invalid)?;
        let cast = Cast::new(&world).map_err(ResumeError::Load)?;
        Ok(Sim::from_parts(world, cp.state.clone(), cast))
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

    /// A checkpoint of the current state: its `world_id`, its `prefix_id` and the state.
    pub fn checkpoint(&self) -> Result<Checkpoint, RunError> {
        self.poisoned()?;
        Ok(Checkpoint::of(&self.world, self.state.clone()))
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
