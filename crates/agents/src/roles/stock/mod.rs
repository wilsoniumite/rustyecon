//! The stock roles (P2.2; docs/probe/HORSES-RULES.md; HORSES-SPEC §2): three behaviour kinds
//! for machines held as stocks of a durable good (the horse), whose known answers are oracle
//! unit 1g's.
//!
//! - **Maker** (a Desk, M2): builds the durable good from the hours of its own serving stock,
//!   bought goods, labour and land; keeps what its plan needs as its serving stock, and offers
//!   the rest under a cover of a few ticks of sales (the reservation band, D-G5).
//! - **CapacityDesk** (a Desk, M3 wet): holds the durable good, buys its running inputs and
//!   sells its hours (horse-days), never using them itself; runs its stock while the cash rule
//!   at the running cost covers it, and orders toward the cash rule's stock at the full cost.
//! - **OwnerDesk** (a Desk, M1): the good desk holding its own machines, registered for R1 and
//!   the restricted alternative (D-G4).
//!
//! They are new kinds, not changed ones: every older kind is untouched, so every committed tape
//! keeps its hash. Each has a state of its own (`Maker`, `Capacity`, `Owner`), appended after
//! `MachDesk`, because a stock needs a record a holding cannot give: which units serve and which
//! are finished (the maker), and what was held when decide ran (the capacity desk). A good that
//! lives one tick takes the maker and the owner desk down the flow path, where they run the type
//! desk's and the good desk's code (R1).

pub mod rules;
pub mod spec;
