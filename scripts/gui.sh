#!/usr/bin/env bash
# The GUI's gate (docs/GUI.md §8.2, D1), which GUI.md calls ci/gui.sh: it lives beside
# scripts/gate.sh, where the repository's gate is. It gates G-stages only, never an engine step.
# WSL Ubuntu is the primary machine; Windows runs the same script under Git Bash.
#
# From a Windows shell:
#   wsl -d ubuntu --exec bash -lc '/mnt/c/<path to the repo>/scripts/gui.sh'
#
# It fails on the first step that fails:
#   1. cargo fmt --check of rustyecon-gui;
#   2. cargo clippy -p rustyecon-gui --all-targets, with every warning an error, under the
#      workspace lints (the crate sets `[lints] workspace = true`);
#   3. cargo test -p rustyecon-gui --release, with every warning an error, and each test G0's
#      and G1's gates name run by name and passed, so a renamed test cannot drop out;
#   3b. on Linux (WSL), G1's measurement by name: a_200_point_sweep_builds_in_under_16_ms, an
#      ignored test, must run and pass (docs/GUI.md §9, G1's gate: median of 20, under 16 ms);
#   4. the cli's hashes: gui_equals_cli writes the GUI path's hashes of tapes/gate.ron (2,080
#      ticks) and tapes/appb.ron (20,000), and gui_equals_cli_demo_gb those of tapes/demo-gb.ron
#      (7,852, through the first tick of 1901, with the lean catalogue), and the body of
#      `rustyecon run <tape> --until <T> --hashes` must equal them byte for byte, both binaries
#      built --release on this machine;
#      and the same for the two branches the editor's tests materialise and run (G0.2):
#      branch_resume_equals_rerun writes branch.ron with its hashes, a branch resumed from its
#      parent's ring, and removal_only_branch_is_an_experiment writes removal.ron with its
#      hashes, each 2,080 ticks, the parent's record before the resume and the branch's after;
#      each also writes its run key's tape_hash, which must be the one the cli's hash file
#      names.
#
# The build goes to CARGO_TARGET_DIR, outside the tree; the default is
# $HOME/scratch/target-rustyecon-gui. On a machine with no network and a warm cargo cache, set
# CARGO_NET_OFFLINE=true.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/scratch/target-rustyecon-gui}"
case "$CARGO_TARGET_DIR" in
    "$root" | "$root"/*)
        echo "gui: CARGO_TARGET_DIR ($CARGO_TARGET_DIR) is inside the tree; put it outside" >&2
        exit 1
        ;;
esac
export RUSTFLAGS="${RUSTFLAGS:-} -D warnings"
export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings"

step() { printf '\n== gui: %s\n' "$*"; }

# The tests G0's gate names (docs/GUI.md §9), as each part of G0 lands. G0.1's second part
# adds every_drawn_vertex_is_recorded, the view-model goldens, G0's kittest scripts, and the scan
# that holds the GUI's use of core to `num`. The fixes after its verification add the theft
# script and the script whose session another tape wrote. G0.2 adds the editor's: the two
# branch tests, the five rules of §8.1's editing row, the editor's two kittest scripts (the
# form's refusals; apply, compare, export and save), and the scan that keeps edit/ pure. G0's
# close (G0.3) adds the reducer's state-machine tests (tests/model.rs), which the gate names as a
# group, so every test it names is checked by name. D.3 (the map, brought forward on branch
# demo-world) adds the map's six (tests/map.rs, docs/GUI.md §8.1's map row and G4's gate). D.4,
# after D.3's verification, adds six more of the map's (the lens values against the engine, the
# palettes, the mesh built afresh and the legend, every triangle's hit, a hover and a click on
# every part, the atlas's credit) and the demo tape's hashes against the cli's. G1 (2026-09-27)
# renames the core scan (the GUI's edge to core went at G1.1) and adds the oracle lab's, the
# price-step explainer's and waterfall's, the breakpoints', the watchlist's and log axes', the
# snapshots', the lab's scan, and the credit on a narrow window (O26).
named=(
    gui_equals_cli
    failed_run_shows_its_ledger_line
    decimation_keeps_extremes
    nonfinite_ingest_stops_with_the_series_named
    model_run_edit_vm_import_no_egui
    no_trig_outside_ui
    no_raw_transcendentals
    no_hashed_collections
    every_drawn_vertex_is_recorded
    gate_view_models_equal_their_goldens
    appb_view_models_equal_their_goldens
    one_key_press_gives_a_live_price_plot
    the_gate_script_runs_pauses_steps_and_inspects
    the_appb_script_runs_pauses_steps_and_inspects
    the_gui_reaches_core_through_the_engine_alone
    the_theft_script_shows_a_failed_run
    a_second_tapes_session_still_plots_every_price
    branch_resume_equals_rerun
    removal_only_branch_is_an_experiment
    gui_edits_are_always_assumed
    saved_tape_carries_its_lineage
    removing_the_last_use_offers_remove_param
    ledger_tolerances_are_not_editable
    minted_keys_never_collide
    the_editor_refuses_an_empty_note_a_malformed_key_and_a_malformed_date
    the_branch_script_applies_compares_exports_and_saves
    edit_reaches_no_model_file_thread_or_clock
    opening_a_tape_records_its_base_and_loads_it_paused
    a_tape_that_does_not_parse_or_read_is_logged_and_not_run
    space_runs_and_pauses_and_steps_follow_the_status
    a_poisoned_run_takes_no_more_commands
    a_stopped_record_pauses_its_run_once
    speed_breakpoints_plots_and_pins_live_in_the_session
    closing_a_run_stops_its_driver_and_ignores_its_late_observations
    a_changed_base_is_logged_and_an_edited_tape_is_an_experiment
    a_session_that_does_not_read_is_set_aside
    a_selected_actor_asks_for_the_snapshot_its_cursor_reads
    rationing_onsets_are_logged_once_a_class_line
    a_log_line_moves_the_cursor_to_the_tick_it_names
    a_run_whose_worker_ended_says_so_and_stops
    a_session_of_another_tape_still_plots_every_price
    apply_branches_from_the_parents_ring_and_files_are_effects
    the_atlas_triangulates_and_labels_hit_their_regions
    every_region_drawn_once
    map_values_equal_table
    lens_domains_are_fixed_for_the_run
    demo_lens_view_models_equal_their_goldens
    the_demo_script_switches_lenses_runs_hovers_and_selects
    lens_values_equal_the_engine
    the_scales_are_the_neutral_palettes
    rebuilt_mesh_colours_are_the_lens_colours
    every_triangle_hits_its_region
    hover_and_click_name_every_part
    the_credit_is_painted_clear_of_the_legend
    gui_equals_cli_demo_gb
    lab_presets_solve_to_their_goldens
    the_lab_shows_appendix_b_bit_for_bit
    a_points_fields_are_the_oracles_doubles
    f_over_x_shows_its_root_and_bracket
    knobs_list_set_and_sweep
    an_edited_instance_is_no_longer_its_goldens
    the_lab_script_shows_appendix_b_and_its_goldens
    lab_reaches_no_run_model_file_thread_or_clock
    the_explainer_equals_next_price_on_every_gate_tick
    the_explainer_equals_next_price_on_every_appb_tick
    the_waterfall_adds_up_to_the_log_price
    the_explainer_script_paints_the_step_and_the_waterfall
    event_and_date_breakpoints_pause_after_their_tick
    breakpoints_change_no_hash
    breakpoints_watch_and_log_scales_live_in_the_session
    the_watch_and_breakpoint_script
    png_snapshots_say_never_citable
    the_snapshot_script_marks_the_picture_never_citable
    the_credit_is_painted_whole_on_a_narrow_window
)

step "toolchain (rust-toolchain.toml)"
rustc -V
cargo -V

step "fmt"
cargo fmt -p rustyecon-gui --check

step "clippy"
cargo clippy --locked -p rustyecon-gui --all-targets -- -D warnings

step "test"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
out="$(RUSTYECON_GUI_HASHES="$work" cargo test --locked --release -p rustyecon-gui 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out"
for t in "${named[@]}"; do
    if ! grep -q "^test $t \.\.\. ok$" <<<"$out"; then
        echo "gui: $t did not run and pass" >&2
        exit 1
    fi
done
echo "the ${#named[@]} named tests ran and passed"

# G1's gate: a 200-point sweep's view-model builds in under 16 ms, on WSL, median of 20. The test
# is ignored by default (a measurement) and run here by name; elsewhere it is not gated.
if [ "$(uname -s)" = "Linux" ]; then
    step "G1's sweep measurement, by name"
    sweep="$(cargo test --locked --release -p rustyecon-gui --test lab -- --ignored --exact \
        a_200_point_sweep_builds_in_under_16_ms --nocapture 2>&1)" || {
        printf '%s\n' "$sweep"
        exit 1
    }
    if ! grep -q "^test a_200_point_sweep_builds_in_under_16_ms \.\.\. ok$" <<<"$sweep"; then
        printf '%s\n' "$sweep"
        echo "gui: a_200_point_sweep_builds_in_under_16_ms did not run and pass" >&2
        exit 1
    fi
    grep '^a 200-point sweep' <<<"$sweep"
else
    step "G1's sweep measurement: gated on Linux only, skipped here"
fi

step "the cli's hashes, against the GUI's"
cargo build --locked --release -p rustyecon-cli
bin="$CARGO_TARGET_DIR/release/rustyecon"
for spec in gate:2080 appb:20000 demo-gb:7852; do
    name="${spec%%:*}"
    until="${spec##*:}"
    "$bin" run "tapes/$name.ron" --until "$until" --hashes "$work/$name.cli" >"$work/$name.out"
    grep -v '^#' "$work/$name.cli" >"$work/$name.body"
    if [ ! -s "$work/$name.hashes" ]; then
        echo "gui: gui_equals_cli wrote no $name.hashes" >&2
        exit 1
    fi
    if ! cmp -s "$work/$name.body" "$work/$name.hashes"; then
        echo "gui: the GUI's hashes of tapes/$name.ron differ from the cli's" >&2
        diff "$work/$name.body" "$work/$name.hashes" | head -5 >&2
        exit 1
    fi
    echo "tapes/$name.ron: $(wc -l <"$work/$name.hashes") ticks equal, final $(tail -n 1 "$work/$name.hashes")"
done
grep '^run build ' "$work/gate.out"
# The branches the editor made: each tape as materialise wrote it, run by the cli from genesis.
# The cli's tape_hash of it must be the GUI run key's, and its hashes the GUI's.
for name in branch removal; do
    if [ ! -s "$work/$name.ron" ] || [ ! -s "$work/$name.hashes" ] ||
        [ ! -s "$work/$name.tape_hash" ]; then
        echo "gui: the branch tests wrote no $name.ron, $name.hashes or $name.tape_hash" >&2
        exit 1
    fi
    "$bin" run "$work/$name.ron" --until 2080 --hashes "$work/$name.cli" >"$work/$name.out"
    cli_hash="$(sed -n 's/^# tape .* tape_hash \(0x[0-9a-f]\{16\}\) world_id .*/\1/p' "$work/$name.cli")"
    gui_hash="$(cat "$work/$name.tape_hash")"
    if [ -z "$cli_hash" ] || [ "$cli_hash" != "$gui_hash" ]; then
        echo "gui: the cli's tape_hash of the $name branch (${cli_hash:-none}) is not" \
            "the GUI run key's ($gui_hash)" >&2
        exit 1
    fi
    grep -v '^#' "$work/$name.cli" >"$work/$name.body"
    if ! cmp -s "$work/$name.body" "$work/$name.hashes"; then
        echo "gui: the GUI's hashes of the $name branch differ from the cli's" >&2
        diff "$work/$name.body" "$work/$name.hashes" | head -5 >&2
        exit 1
    fi
    echo "$name branch ($(sed -n 's/^# tape \(.*\) tape_hash.*/\1/p' "$work/$name.cli")," \
        "tape_hash $cli_hash, the GUI's):" \
        "$(wc -l <"$work/$name.hashes") ticks equal, final $(tail -n 1 "$work/$name.hashes")"
done

step "green"
