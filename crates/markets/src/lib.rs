//! Orders, admission, clearing, settlement and the price update (docs/ENGINE.md §3; PLAN §3.3),
//! built on `rustyecon-core` alone.
//!
//! A tick's market work runs in three phases, and every change it makes is a core delta that the
//! engine applies:
//!
//! - **Phase 2.** [`admit`] checks the orders against the phase-start state and binds each buy's
//!   budget there (N7). [`clear`] folds the admitted lines into supply and feasible demand per
//!   market and returns the fills and a `SetVolumes` for every market.
//! - **Phase 3.** [`settle`] plans both sides of every trading market from one filled quantity
//!   per order, through the market's escrow; [`SettlePlan::realize`] turns the quantities `apply`
//!   moved into one [`SettleLine`] per order and one [`RationLine`] per (market, class, side)
//!   (R12).
//! - **Phase 6.** [`update_prices`] moves each price by the tape's rule and each EMA by its
//!   registered span in years (A13).
//!
//! Salvaged from the July engine (tag `july-v2-phase-3`), with its defects fixed as it moved: no
//! absolute epsilon on a price change, a currency flow or a quantity (N5, N6, A12); cash bound at
//! the order, not at settlement (N7); no balance forgiveness (N8); the price rule and the
//! one-sided rule from the tape, with no default and no silent fallback (N13, F8); and typed
//! holders, so a desk and a pop that share a number settle apart (defect 10). No literal but `0.0`
//! and `1.0` appears outside the tests (R4), no transcendental but through `core::num` (A5), no
//! hashed container (R8), and no I/O (E2). Nothing here panics on bad input.

pub mod clearing;
pub mod order;
pub mod prices;
pub mod settlement;

pub use clearing::{clear, Fills, MarketFill};
pub use order::{admit, Line, Order, OrderError, Side, SideTag};
pub use prices::{imbalance, next_price, step, update_prices, PriceError};
pub use settlement::{settle, RationLine, SettleLine, SettlePlan};

#[cfg(test)]
mod tests {
    use super::*;
    use rustyecon_core::NoExt;

    #[test]
    fn markets_types_cross_threads() {
        // E3: the engine builds reports and errors from these and sends them across threads.
        fn ok<T: Send + Sync + 'static>() {}
        ok::<Order>();
        ok::<Line>();
        ok::<OrderError>();
        ok::<Fills>();
        ok::<MarketFill>();
        ok::<SettlePlan<NoExt>>();
        ok::<SettleLine>();
        ok::<RationLine>();
        ok::<PriceError>();
    }
}
