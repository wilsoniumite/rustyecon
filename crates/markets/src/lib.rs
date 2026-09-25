//! Clearing, settlement and the price update (PLAN §3.3). Clearing is pro-rata per node and
//! good; both sides settle from one filled quantity, with cash bound at the order; rationing
//! is recorded per buyer class; and prices move by imbalance with no floors or clamps. It is
//! salvaged from the July engine in Phase 0, and Phase 2 builds the agents on it.
