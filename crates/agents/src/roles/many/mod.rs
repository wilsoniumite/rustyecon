//! The many-market roles (P2.1; docs/probe/MARKETS-RULES.md; MARKETS-SPEC §2): four behaviour
//! kinds that carry the Appendix B roles to economies with many final categories and many
//! machine types, whose known answers are oracle units 1b and 1c.
//!
//! - **BasketProvider** (a Pop): the provider, with a basket of many items. P_s is the basket's
//!   price at posted prices; the land it sells may be an item (space, bought on the market).
//! - **BasketWorkers** (a Pop): the workers, with a basket of many items.
//! - **CategoryDesk** (a Desk): the good desk on its segments of the shared task line (decision
//!   60), buying the task type's services at efficiency θ and direct land.
//! - **TypeDesk** (a Desk): the machine desk, buying other types' services beside hours and land.
//!
//! [`switch`] (P2.4; the type switch at the wall, O97) runs a `BasketWorkers` with a `pool` as a
//! switch pop, whose state is the appended `SwitchWorkers`.
//!
//! They are new kinds, not changed ones: the Appendix B kinds are untouched, so `tapes/appb.ron`
//! keeps its hash, and each new kind reuses an Appendix B kind's state (`Provider`, `Workers`,
//! `GoodDesk`, `MachDesk`), so every reader of those states reads these. On Appendix B each rule
//! makes its Appendix B role's floating-point operations exactly (the nesting check,
//! MARKETS-SPEC §2.7; `markets_i0_nests_appb` in crates/probe).

pub mod rules;
pub mod spec;
pub mod switch;
