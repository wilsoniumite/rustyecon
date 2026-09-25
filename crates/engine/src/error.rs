//! The engine's errors (docs/ENGINE.md §7.5). Written by hand, each with a `Display` that gives
//! the ledger line for a shortfall or a conservation breach; every one is `Send + Sync +
//! 'static` (E3).

use rustyecon_agents::{AgentError, Agents, Hook};
use rustyecon_core::{ActorId, CoreError, LoadError, Phase, StateDelta};
use rustyecon_markets::{Order, OrderError, PriceError};
use std::fmt;

/// A step that failed. The `Sim` is then poisoned (E5).
#[derive(Debug, Clone, PartialEq)]
pub struct RunError {
    /// The tick that was running (the state's tick when it began).
    pub tick: u64,
    /// The phase that failed.
    pub phase: Phase,
    /// What failed.
    pub kind: RunErrorKind,
}

/// What made a step fail.
#[derive(Debug, Clone, PartialEq)]
pub enum RunErrorKind {
    /// A delta or the ledger: a shortfall (with its ledger line), a conservation breach, a value
    /// or id that does not fit.
    Core(CoreError),
    /// Admission, clearing or settlement.
    Order(OrderError),
    /// The price update.
    Price(PriceError),
    /// A hook that could not run.
    Agent {
        /// The actor.
        actor: ActorId,
        /// The hook.
        hook: Hook,
        /// Why.
        error: AgentError,
    },
    /// A hook returned a delta outside its whitelist (E7, R13).
    ForeignWrite {
        /// The actor.
        actor: ActorId,
        /// The hook.
        hook: Hook,
        /// The delta.
        delta: StateDelta<Agents>,
    },
    /// `decide` returned an order for another actor or another class (E7, R13).
    ForeignOrder {
        /// The actor.
        actor: ActorId,
        /// The order.
        order: Order,
    },
    /// The `Sim` was poisoned by an earlier error; rebuild it with `Sim::new` or `Sim::resume`.
    Poisoned,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tick {}, phase {}: {}", self.tick, self.phase, self.kind)
    }
}

impl fmt::Display for RunErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunErrorKind::Core(e) => write!(f, "{e}"),
            RunErrorKind::Order(e) => write!(f, "{e}"),
            RunErrorKind::Price(e) => write!(f, "{e}"),
            RunErrorKind::Agent { actor, hook, error } => {
                write!(f, "{actor}'s {hook} hook failed: {error}")
            }
            RunErrorKind::ForeignWrite { actor, hook, delta } => write!(
                f,
                "{actor}'s {hook} hook returned a {} outside its whitelist: {delta:?}",
                delta.name()
            ),
            RunErrorKind::ForeignOrder { actor, order } => write!(
                f,
                "{actor}'s decide hook returned an order for {} in {}: {order:?}",
                order.actor, order.class
            ),
            RunErrorKind::Poisoned => write!(
                f,
                "the sim is poisoned by an earlier error; rebuild it from a tape or a checkpoint"
            ),
        }
    }
}

impl std::error::Error for RunError {}

/// A resume that is refused (§7.6, N11). A checkpoint of another format, or one whose state
/// does not match its digest, is refused earlier, when it is decoded
/// ([`CheckpointError`](rustyecon_core::CheckpointError)).
#[derive(Debug, Clone, PartialEq)]
pub enum ResumeError {
    /// The tape does not load.
    Load(LoadError),
    /// The checkpoint belongs to another world: the tape's world content or genesis differs.
    WrongWorld {
        /// The tape's `world_id`.
        tape: u64,
        /// The checkpoint's.
        checkpoint: u64,
    },
    /// Something that fired before the checkpoint's tick differs between the tape and the
    /// checkpoint.
    WrongPrefix {
        /// The checkpoint's tick.
        tick: u64,
        /// The tape's `prefix_id` at that tick.
        tape: u64,
        /// The checkpoint's.
        checkpoint: u64,
    },
    /// The state does not fit the world.
    Invalid(CoreError),
}

impl fmt::Display for ResumeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResumeError::Load(e) => write!(f, "the tape does not load: {e}"),
            ResumeError::WrongWorld { tape, checkpoint } => write!(
                f,
                "the checkpoint belongs to world 0x{checkpoint:016x}, but the tape is world \
                 0x{tape:016x}"
            ),
            ResumeError::WrongPrefix {
                tick,
                tape,
                checkpoint,
            } => write!(
                f,
                "the tape's events before tick {tick} differ from the checkpoint's (prefix \
                 0x{tape:016x}, checkpoint 0x{checkpoint:016x})"
            ),
            ResumeError::Invalid(e) => write!(f, "the checkpoint does not fit the tape: {e}"),
        }
    }
}

impl std::error::Error for ResumeError {}

/// A replay audit that failed (§7.6).
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayError {
    /// The tape does not load.
    Load(LoadError),
    /// The live run failed.
    Run(RunError),
    /// The shadow state's hash differs from the live one after a tick.
    Mismatch {
        /// The tick that ran.
        tick: u64,
        /// The live hash.
        live: u64,
        /// The shadow's.
        shadow: u64,
    },
    /// A traced delta did not apply to the shadow state.
    Shadow {
        /// The tick that ran.
        tick: u64,
        /// The phase.
        phase: Phase,
        /// Why.
        error: CoreError,
    },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::Load(e) => write!(f, "the tape does not load: {e}"),
            ReplayError::Run(e) => write!(f, "{e}"),
            ReplayError::Mismatch { tick, live, shadow } => write!(
                f,
                "replay mismatch after tick {tick}: live 0x{live:016x}, shadow 0x{shadow:016x}"
            ),
            ReplayError::Shadow { tick, phase, error } => write!(
                f,
                "replay mismatch at tick {tick}, phase {phase}: a traced delta does not apply \
                 to the shadow state: {error}"
            ),
        }
    }
}

impl std::error::Error for ReplayError {}
