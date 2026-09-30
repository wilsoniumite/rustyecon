#!/usr/bin/env bash
# D2.4 (label run, 2026-09-30): the long run's job (WORLD-V2 §11.4). usage: longjob.sh DIR
# Runs the frozen test binary's `demo_v2_runs_to_1901` (crates/worldgen/tests/demo_v2.rs), which
# runs tapes/demo-gb-v2.ron through the first tick of 1901, scores every county every tick
# against its moving oracle point, and writes DIR/long-run.csv and DIR/series.csv. Its output is
# in DIR/out.txt, and DIR/exit has the exit code and the wall time. The binary is
# /root/scratch/d2-runs/bin/demo_v2, built with `cargo test --release --no-run` on WSL from the
# D2.4 commit; its sha256 is in the wave's BIN.sha256. It reads the tables and the tape from the
# worktree, whose files the commit holds.
T=/root/scratch/d2-runs/bin/demo_v2
dir=$1
mkdir -p "$dir"
cd /mnt/d/rustyecon-wt/d2/crates/worldgen || exit 3
t0=$(date +%s.%N)
DEMO_V2_LONG_OUT="$dir" "$T" --ignored --exact demo_v2_runs_to_1901 --nocapture >"$dir/out.txt" 2>"$dir/err.txt"
rc=$?
t1=$(date +%s.%N)
echo "$rc $(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.3f", b-a}')" >"$dir/exit"
