#!/usr/bin/env bash
# D2.4 (label run, 2026-09-30): one job of the wave (make_jobs.py). usage: job.sh DIR CMD ARGS...
# Runs the frozen harness `horses CMD ARGS...` with its stdout and stderr in DIR, and writes
# DIR/exit with the exit code and the wall time. The binary is /root/scratch/d2-runs/bin/horses,
# built in release on WSL from the D2.4 commit; its sha256 is in the wave's BIN.sha256.
B=/root/scratch/d2-runs/bin/horses
dir=$1
shift
mkdir -p "$dir"
t0=$(date +%s.%N)
"$B" "$@" >"$dir/out.txt" 2>"$dir/err.txt"
rc=$?
t1=$(date +%s.%N)
echo "$rc $(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.3f", b-a}')" >"$dir/exit"
