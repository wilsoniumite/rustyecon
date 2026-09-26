//! The Appendix B roles (P2.0): four behaviour kinds for the Phase 2 probe, whose question is
//! whether agents deciding at the pinning paper's margins reach the oracle's equilibrium of the
//! SSRN Appendix B economy (docs/probe/RULES.md; PROBE-SPEC).
//!
//! - **Provider** (a Pop): owns the land, sells all of its services, transfers N·P_s to the
//!   workers at posted prices, and spends a share of the rest of its coin on baskets.
//! - **Workers** (a Pop): offer N·F(ln(1 + w/P_s)) hours, F the uniform work cost's cdf, and
//!   spend a share of their coin on baskets.
//! - **GoodDesk** (a Desk): moves its technique toward the task measure at posted prices, spends
//!   a share of its coin on hours and machine services, and makes the good by Leontief at its
//!   technique.
//! - **MachDesk** (a Desk): spends a share of its coin on hours and space, keeps a·q of last
//!   tick's machine services for its own use, sells the rest, and makes machine services.
//!
//! The rules are design-analytic-first's, with the judges' grafts (docs/probe/RULES.md §5):
//! the markup tilt and a ceiling payout as registered dials of the cash rule, the technique
//! carried as 1 − x, Leontief at the planned technique as the default with ex-post assignment as
//! the alternative, and July's margin-step rule as the registered negative control. Every dial
//! is a tape param (R4); nothing here reads the oracle (R13).

pub mod rules;
pub mod spec;
