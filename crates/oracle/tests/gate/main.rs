//! The oracle's gate: every golden of docs/unit-1a.md §6, docs/unit-1b.md §7,
//! docs/unit-1c.md §7 and docs/unit-1d.md §7, and the dump interface.
//!
//! One test crate, so the goldens and fixtures compile once and every golden constant
//! must be used by some test.

mod goldens;
mod goldens_1b;
mod goldens_1c;
mod goldens_1d;
mod support;
mod support_1b;
mod support_1c;
mod support_1d;

mod dump_line;
mod g1_appendix_b;
mod g2_figure_3;
mod g3_automation_path;
mod g4_durability;
mod g5_general_instances;
mod g5_random_economies;
mod g6_closure;
mod g7_three_taxes;
mod g8_regimes;
mod goldens_file;

mod c1_nesting;
mod c2_ces_share;
mod c3_fork_economy;
mod c4_paths;
mod c5_random_categories;
mod c6_price_block;
mod c7_gaps_and_regimes;
mod c8_goldens_file;

mod m1_nesting;
mod m2_machine_block;
mod m3_two_recipe;
mod m4_many_types;
mod m5_interest;
mod m6_random_leontief;
mod m7_regimes_and_validation;
mod m8_goldens_file;

mod d1_nesting;
mod d2_one_type_corners;
mod d3_human_required;
mod d4_worker_types;
mod d5_full_economy;
mod d6_wall_switches;
mod d7_random_workers;
mod d8_regimes_and_validation;
mod d9_goldens_file;
