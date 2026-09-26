//! The agents (docs/ENGINE.md §4): the behaviour seam, the scripted actor, and the four
//! Appendix B roles of the Phase 2 probe.
//!
//! PLAN §3.2's agent rules, where every decision is one of the pinning paper's margins, fill in
//! during Phase 2 as new behaviour kinds. Phase 0 has the scripted actor, whose lines and recipe
//! are registered params read at use time; the probe (P2.0) adds the provider, the workers, the
//! good desk and the machine desk of the SSRN Appendix B economy ([`roles`]). The crate implements core's extension seam
//! ([`Agents`]): the actors' specs on the tape, their own state (hashed and checkpointed), their
//! own deltas and the tape actions on them.
//!
//! Agents read posted prices, their own holding and state, and the current params, through a
//! [`View`] that reaches nothing else, so no agent reads another's orders or state (R13); this
//! crate never depends on the oracle. The engine applies what hooks return, after checking it
//! against a whitelist (E7). Nothing here holds global state or does I/O (E2), and no behavioural
//! literal lives in the code (R4): every number comes from the tape.

pub mod behaviour;
pub mod cast;
pub mod ext;
pub mod roles;
pub mod script;
pub mod spec;

pub use behaviour::{AgentError, Behaviour, Decision, Hook, Posted, View};
pub use cast::Cast;
pub use ext::{
    ActorState, AgentDelta, Agents, GoodDeskState, MachDeskState, ProviderState, RawAgentAction,
    ScriptState, WorkersState,
};
pub use roles::spec::{
    Assign, Basket, Ceiling, GoodDesk, MachDesk, MachRecipe, Provider, RawBasket, RawCash,
    RawCeiling, RawGoodDesk, RawMachDesk, RawMachRecipe, RawPayoutTo, RawProvider, RawScale,
    RawSchedule, RawStep, RawTechnique, RawTransfer, RawWorkers, Scale, Schedule, Workers,
};
pub use spec::{
    BuyLine, Payout, RawBuy, RawPayout, RawRecipe, RawScript, RawSell, RawSellQty, RawSpec, Recipe,
    Script, SellLine, SellQty, Spec,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_types_cross_threads() {
        // E3: the engine keeps these in a Sim that runs on a worker thread.
        fn ok<T: Send + Sync + 'static>() {}
        ok::<Cast>();
        ok::<Spec>();
        ok::<ActorState>();
        ok::<AgentDelta>();
        ok::<AgentError>();
        ok::<Decision>();
    }
}
