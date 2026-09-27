//! The oracle's gate: every golden of docs/unit-1a.md §6, docs/unit-1b.md §7,
//! docs/unit-1c.md §7, docs/unit-1d.md §7, docs/unit-1e.md §7 and docs/unit-1f.md §7, the dump
//! interface, and Phase 1's gate item by item (`p1_gate`).
//!
//! One test crate, so the goldens and fixtures compile once and every golden constant
//! must be used by some test.

mod goldens;
mod goldens_1b;
mod goldens_1c;
mod goldens_1d;
mod goldens_1e;
mod goldens_1f;
mod support;
mod support_1b;
mod support_1c;
mod support_1d;
mod support_1e;
mod support_1f;

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

mod e10_goldens_file;
mod e1_nesting;
mod e2_exit_value;
mod e3_race;
mod e4_commons;
mod e5_idle_land;
mod e6_types_and_fork;
mod e7_multiplicity;
mod e8_random_parcels;
mod e9_regimes_and_validation;

mod f10_regimes_and_validation;
mod f11_goldens_file;
mod f1_nesting;
mod f2_three_taxes;
mod f3_coverage;
mod f4_transfers;
mod f5_payroll;
mod f6_consumption_tax;
mod f7_conditionality;
mod f8_ces;
mod f9_random_households;

mod p1_gate;
