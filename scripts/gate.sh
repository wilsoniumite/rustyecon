#!/usr/bin/env bash
# The Phase 0 gate (docs/PLAN.md Phase 0; docs/ENGINE.md §11; docs/CERTIFY.md §14), as run on
# WSL Ubuntu, the primary machine, and by .github/workflows/ci.yml. Windows runs the same script
# under Git Bash.
#
# From a Windows shell:
#   wsl -d ubuntu --exec bash -lc '/mnt/c/<path to the repo>/scripts/gate.sh'
# Use --exec: with a plain -- the exit code is lost.
#
# It fails on the first step that fails:
#   1. cargo fmt --all --check, the GUI included;
#   2. cargo clippy --workspace --exclude rustyecon-gui --all-targets, with every warning an
#      error (D1: the GUI never gates engine work; scripts/gui.sh gates it at each G-stage);
#   3. cargo test --workspace --exclude rustyecon-gui --release, with every warning an error;
#   4. the repeat-hash test by name, which must run and pass (a renamed test cannot drop out);
#   5. certify without Parquet: check, clippy and test rustyecon-certify alone, with its
#      `parquet` feature off, and parquet absent from its tree (the GUI's web build);
#   6. the gate tape run twice through the binary to tick 2,080: the two hash files must be
#      identical, and the count of hashed ticks (the `#` header skipped) and the final hash are
#      printed;
#   7. the build stamp: the binary's `run …` line names `git rev-parse HEAD`, and its dirty flag
#      is whether `git status --porcelain` over the build's sources is non-empty;
#   8. the committed certificates: committed_certificates_recompute by name, which must run and
#      pass (C10), and each certificate's build commit an ancestor of HEAD;
#   9. the probe's report: probe_battery_csv_unchanged by name, which must run and pass (C11:
#      the probe reads certify's measures, and its 57 rows, its negative control and its shock
#      history must print as docs/probe/results/ does);
#  10. the demo world (D.2, docs/demo/WORLD.md): derive.py --check when a python 3 is found,
#      and demo_runs_to_1901 by name, which must run and pass (tapes/demo-gb.ron through the
#      first tick of 1901, every step fired, every county alive and near its moving oracle
#      point, its recorded hashes); and its second pass (D2.2, docs/demo/WORLD-V2.md):
#      demo_v2_pin by name (tapes/demo-gb-v2.ron through the first tick of 1901, every event
#      fired, the ledger closed every tick, its recorded hashes), and demo_v2_runs_to_1901 by
#      name (D2.4: the same run scored against each county's moving oracle point, no dead tick
#      in any county);
#  11. telemetry written twice through the binary, from two processes: the Parquet files, and
#      the manifests that pin them, must be byte-identical.
# Then, recorded and never gated: on Linux, one cargo check of rustyecon-gui in its own target
# directory, $CARGO_TARGET_DIR-gui (D1; docs/GUI.md §8.2: an engine-caused GUI break is fixed
# by the next G-stage at the latest), and cargo check of rustyecon-engine and of
# rustyecon-certify (Parquet-free) for wasm32-unknown-unknown, when that target is installed.
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

# git on this checkout. In WSL a worktree made from Windows names its git directory by a Windows
# path (`gitdir: C:/…`) that Linux git cannot read; map X:/ to /mnt/x/ and retry, as the cli's
# build.rs does (docs/CERTIFY.md §3). --no-optional-locks leaves the index as it is.
git_() {
    if git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
        git -C "$root" --no-optional-locks "$@"
        return
    fi
    local gd drive
    gd="$(sed -n 's/^gitdir: *//p' "$root/.git" 2>/dev/null | tr -d '\r')"
    case "$gd" in
        [A-Za-z]:/*)
            drive="$(printf '%s' "${gd:0:1}" | tr '[:upper:]' '[:lower:]')"
            gd="/mnt/$drive/${gd:3}"
            ;;
    esac
    GIT_DIR="$gd" GIT_WORK_TREE="$root" git -C "$root" --no-optional-locks \
        -c safe.directory='*' "$@"
}

# The build's sources, whose state the stamp's dirty flag reports (docs/CERTIFY.md §3).
sources=(crates Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml)

step "toolchain (rust-toolchain.toml)"
rustc -V
cargo -V

step "fmt"
cargo fmt --all --check

step "clippy"
cargo clippy --locked --workspace --exclude rustyecon-gui --all-targets -- -D warnings

step "test"
cargo test --locked --workspace --exclude rustyecon-gui --release

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

step "certify without Parquet (the feature off)"
# Selecting certify alone keeps the cli's feature out (resolver 2).
tree="$(cargo tree --locked -p rustyecon-certify -e normal --prefix none)"
if grep -q '^parquet ' <<<"$tree"; then
    echo "gate: parquet is in rustyecon-certify's tree with the feature off" >&2
    exit 1
fi
cargo check --locked -p rustyecon-certify
cargo clippy --locked -p rustyecon-certify --all-targets -- -D warnings
cargo test --locked --release -p rustyecon-certify

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
echo "ticks hashed: $(grep -vc '^#' "$work/a.txt")"
echo "gate hash: $(tail -n 1 "$work/a.out")"
grep '^#' "$work/a.txt"

step "build stamp"
run_line="$(grep '^run build ' "$work/a.out")"
echo "$run_line"
read -r _ _ stamp_commit stamp_state _ <<<"$run_line"
head_commit="$(git_ rev-parse HEAD)"
if [ -n "$(git_ status --porcelain --untracked-files=all -- "${sources[@]}")" ]; then
    tree_state=dirty
else
    tree_state=clean
fi
if [ "$stamp_commit" != "$head_commit" ] || [ "$stamp_state" != "$tree_state" ]; then
    echo "gate: the binary is stamped $stamp_commit $stamp_state, the checkout is" \
        "$head_commit $tree_state" >&2
    exit 1
fi
echo "stamp matches the checkout: $head_commit $tree_state"

step "committed certificates"
# C10: rerun both certified runs from their registered criteria; the verdict and each battery's
# pass flag are gated on every machine, the bytes on Linux only (Windows records them).
out="$(cargo test --locked --release -p rustyecon-certify --test results \
    -- --ignored --exact committed_certificates_recompute --show-output 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out"
if ! grep -q '^test committed_certificates_recompute \.\.\. ok$' <<<"$out"; then
    echo "gate: committed_certificates_recompute did not run" >&2
    exit 1
fi
for cert in results/*/certificate.ron; do
    c="$(sed -n '/^ *commit: "/{s/^ *commit: "\([0-9a-f]*\)",$/\1/p;q;}' "$cert")"
    if [ "${#c}" != 40 ] || ! git_ merge-base --is-ancestor "$c" HEAD; then
        echo "gate: $cert names build '$c', which is not an ancestor of HEAD" >&2
        exit 1
    fi
    echo "$cert: build $c, an ancestor of HEAD"
done

step "the probe's report, pinned"
# C11: the probe delegates its oracle-free measures to certify; its summaries must not move.
out="$(cargo test --locked --release -p rustyecon-probe --test pins \
    -- --ignored --exact probe_battery_csv_unchanged 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out"
if ! grep -q '^test probe_battery_csv_unchanged \.\.\. ok$' <<<"$out"; then
    echo "gate: probe_battery_csv_unchanged did not run" >&2
    exit 1
fi

step "the demo world"
# D.2 (docs/demo/WORLD.md): regions.csv and history.csv are derive.py's output from
# counties.csv (standard-library Python, run when a working python is found); the committed
# tapes/demo-gb.ron is the compiler's output (demo_tape_is_its_compilers_output, in the test
# step); and the long run by name, which must run and pass: the tape through the first tick of
# 1901, every step fired, every county alive and near its moving oracle point every tick, and
# the recorded hashes.
py=""
for c in python3 python; do
    if "$c" -c 'import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)' >/dev/null 2>&1; then
        py="$c"
        break
    fi
done
if [ -n "$py" ]; then
    "$py" worlds/demo-gb/derive.py --check
    echo "derive.py --check: regions.csv and history.csv are derive.py's output ($py)"
else
    echo "no python 3.8 or later: derive.py --check skipped (recorded, not gated)"
fi
out="$(cargo test --locked --release -p rustyecon-worldgen --test demo \
    -- --ignored --exact demo_runs_to_1901 --show-output 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out" | grep -v '^county\.'
if ! grep -q '^test demo_runs_to_1901 \.\.\. ok$' <<<"$out"; then
    echo "gate: demo_runs_to_1901 did not run" >&2
    exit 1
fi
# The second pass (D2.2, docs/demo/WORLD-V2.md): tapes/demo-gb-v2.ron is the compiler's output
# with --stage v2a1 (demo_v2_tape_is_its_compilers_output, in the test step), and its pin by
# name: through the first tick of 1901, every event fired, the ledger closed, its hashes.
out="$(cargo test --locked --release -p rustyecon-worldgen --test demo_v2 \
    -- --ignored --exact demo_v2_pin --show-output 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out"
if ! grep -q '^test demo_v2_pin \.\.\. ok$' <<<"$out"; then
    echo "gate: demo_v2_pin did not run" >&2
    exit 1
fi
# Its long run scored (D2.4, WORLD-V2 §11.4; decision 358): the same run against each county's
# moving oracle point, the registration's refutation (no dead tick in any county), the pin's
# hashes and the plans' params at the end; the table it writes is docs/demo/v2/results/'s.
out="$(cargo test --locked --release -p rustyecon-worldgen --test demo_v2 \
    -- --ignored --exact demo_v2_runs_to_1901 --show-output 2>&1)" || {
    printf '%s\n' "$out"
    exit 1
}
printf '%s\n' "$out" | grep -v '^county\.'
if ! grep -q '^test demo_v2_runs_to_1901 \.\.\. ok$' <<<"$out"; then
    echo "gate: demo_v2_runs_to_1901 did not run" >&2
    exit 1
fi

step "telemetry, written from two processes"
for k in 1 2; do
    rc=0
    "$bin" certify tapes/gate.ron --until 2080 --out "$work/t$k" --telemetry \
        >"$work/t$k.out" || rc=$?
    if [ "$rc" != 5 ] || ! grep -q '^VERDICT: UNSCORED ' "$work/t$k.out"; then
        echo "gate: an unscored certify exited $rc, not 5 with UNSCORED" >&2
        cat "$work/t$k.out" >&2
        exit 1
    fi
done
for f in telemetry.parquet manifest.ron certificate.ron hashes.txt; do
    if ! cmp -s "$work/t1/$f" "$work/t2/$f"; then
        echo "gate: two processes wrote different $f" >&2
        exit 1
    fi
done
echo "telemetry.parquet: $(wc -c <"$work/t1/telemetry.parquet") bytes, identical from two processes"
if ! grep -A 4 'telemetry: Some' "$work/t1/manifest.ron"; then
    echo "gate: the manifest does not record the telemetry" >&2
    exit 1
fi

step "the GUI, checked once (recorded, not gated; D1)"
# docs/GUI.md §8.2: every engine step checks the GUI once on WSL, in a target directory of its
# own, and records the result; it never fails the step. scripts/gui.sh gates the GUI at each
# G-stage.
if [ "$(uname -s)" != Linux ]; then
    echo "not on Linux: skipped (the check is WSL's)"
elif CARGO_TARGET_DIR="$CARGO_TARGET_DIR-gui" cargo check --locked -p rustyecon-gui; then
    echo "rustyecon-gui checks (recorded, not gated)"
else
    echo "rustyecon-gui does not check (recorded, not gated: the next G-stage fixes it)"
fi

step "wasm32 check (recorded, not gated)"
if rustup target list --installed 2>/dev/null | grep -qx 'wasm32-unknown-unknown'; then
    for p in rustyecon-engine rustyecon-certify; do
        if cargo check --locked --target wasm32-unknown-unknown -p "$p"; then
            echo "wasm32-unknown-unknown: $p checks"
        else
            echo "wasm32-unknown-unknown: $p does not check (recorded, not gated)"
        fi
    done
else
    echo "wasm32-unknown-unknown is not installed; skipped"
fi

step "green"
