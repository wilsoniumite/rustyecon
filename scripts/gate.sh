#!/usr/bin/env bash
# The Phase 0 gate (docs/PLAN.md Phase 0; docs/ENGINE.md §11), as run on WSL Ubuntu, the
# primary machine, and by .github/workflows/ci.yml.
#
# From a Windows shell:
#   wsl -d ubuntu --exec bash -lc '/mnt/c/<path to the repo>/scripts/gate.sh'
# Use --exec: with a plain -- the exit code is lost.
#
# It fails on the first step that fails:
#   1. cargo fmt --all --check;
#   2. cargo clippy --workspace --all-targets, with every warning an error;
#   3. cargo test --workspace --release, with every warning an error;
#   4. the repeat-hash test by name, which must run and pass (a renamed test cannot drop out);
#   5. the gate tape run twice through the binary to tick 2,080: the two per-tick hash files
#      must be identical, and the final hash is printed.
# Then, recorded and never gated: cargo check of rustyecon-engine for wasm32-unknown-unknown,
# when that target is installed.
#
# The build goes to CARGO_TARGET_DIR, outside the tree; the default is
# $HOME/scratch/target-rustyecon-gate. On a machine with no network and a warm cargo cache, set
# CARGO_NET_OFFLINE=true.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/scratch/target-rustyecon-gate}"
case "$CARGO_TARGET_DIR" in
    "$root" | "$root"/*)
        echo "gate: CARGO_TARGET_DIR ($CARGO_TARGET_DIR) is inside the tree; put it outside" >&2
        exit 1
        ;;
esac
export RUSTFLAGS="${RUSTFLAGS:-} -D warnings"
export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings"

step() { printf '\n== gate: %s\n' "$*"; }

step "toolchain (rust-toolchain.toml)"
rustc -V
cargo -V

step "fmt"
cargo fmt --all --check

step "clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

step "test"
cargo test --locked --workspace --release

step "repeat-hash test"
out="$(cargo test --locked --release -p rustyecon-engine --test determinism \
    -- --exact gate_repeat_identical_hashes 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out"
if ! grep -q '^test gate_repeat_identical_hashes \.\.\. ok$' <<<"$out"; then
    echo "gate: gate_repeat_identical_hashes did not run" >&2
    exit 1
fi

step "gate tape, two runs through the binary"
cargo build --locked --release -p rustyecon-cli
bin="$CARGO_TARGET_DIR/release/rustyecon"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
"$bin" run tapes/gate.ron --until 2080 --hashes "$work/a.txt" >"$work/a.out"
"$bin" run tapes/gate.ron --until 2080 --hashes "$work/b.txt" >"$work/b.out"
if ! cmp -s "$work/a.txt" "$work/b.txt"; then
    echo "gate: two runs of tapes/gate.ron gave different hash streams" >&2
    exit 1
fi
echo "ticks hashed: $(wc -l <"$work/a.txt")"
echo "gate hash: $(tail -n 1 "$work/a.out")"

step "wasm32 check (recorded, not gated)"
if rustup target list --installed 2>/dev/null | grep -qx 'wasm32-unknown-unknown'; then
    if cargo check --locked --target wasm32-unknown-unknown -p rustyecon-engine; then
        echo "wasm32-unknown-unknown: rustyecon-engine checks"
    else
        echo "wasm32-unknown-unknown: rustyecon-engine does not check (recorded, not gated)"
    fi
else
    echo "wasm32-unknown-unknown is not installed; skipped"
fi

step "green"
