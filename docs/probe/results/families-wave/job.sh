#!/bin/bash
# P2.4 (label families, 2026-09-30): one job of wave A, the families wave. usage: job.sh DIR CMD ARGS...
# Runs the frozen harness `markets CMD ARGS...` with its stdout and stderr in DIR, and writes
# DIR/exit with the exit code and the wall time. The binary is /root/scratch/p24-families/markets
# (sha256 in BIN.sha256 beside this file when the wave starts), built from the commit that adds
# this file. As p23-wave/job.sh (P2.3.11), with the binary's path changed.
B=/root/scratch/p24-families/markets
dir=$1; shift
mkdir -p "$dir"
t0=$(date +%s.%N)
"$B" "$@" > "$dir/out.txt" 2> "$dir/err.txt"
rc=$?
t1=$(date +%s.%N)
echo "$rc $(awk -v a="$t0" -v b="$t1" "BEGIN{printf \"%.3f\", b-a}")" > "$dir/exit"
