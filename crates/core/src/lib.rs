//! The substrate every other crate builds on: ids, goods, inventories, the state deltas and
//! the single pass that applies them, the conservation ledger, the state hash, checkpoints,
//! and the tape's runtime form. It knows nothing about agent rules, so the oracle, the
//! markets and the agents can share its types without sharing logic. It is salvaged from the
//! July engine in Phase 0 and grows with the ontology of PLAN §3.1 in Phases 2 and 3.
