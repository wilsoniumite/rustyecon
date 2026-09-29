//! CAPACITY's plant on a desk (P2.2b; docs/probe/LOOPS-RULES.md §3–§4; LOOP-SPEC §2.2–§2.5): the
//! type desk, the maker and the capacity desk each take an optional plant, y = P^(1−θ)·z^θ over
//! the desk's own Leontief bundle z, built from s bundles a unit, worn at u a tick and ordered by
//! M3's rule. A desk without a plant runs its kind's code unchanged; at θ = 1 a planted desk makes
//! its kind's orders, budgets and produce bit for bit (LOOPS-RULES §3.9).

pub mod rules;
pub mod spec;
