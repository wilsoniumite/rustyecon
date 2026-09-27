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
#      gate names so far run by name and passed, so a renamed test cannot drop out;
#   4. the cli's hashes: gui_equals_cli writes the GUI path's hashes of tapes/gate.ron (2,080
#      ticks) and tapes/appb.ron (20,000), and the body of `rustyecon run <tape> --until <T>
#      --hashes` must equal them byte for byte, both binaries built --release on this machine.
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
# that holds the GUI's use of core to `num`.
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
    the_gui_names_core_for_num_alone
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

step "the cli's hashes, against the GUI's"
cargo build --locked --release -p rustyecon-cli
bin="$CARGO_TARGET_DIR/release/rustyecon"
for spec in gate:2080 appb:20000; do
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

step "green"
