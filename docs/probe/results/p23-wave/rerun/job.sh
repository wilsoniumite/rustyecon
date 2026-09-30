#!/bin/bash
# P2.3.12 (label run, 2026-09-30): one job of the wave's rerun after the dated-commons fix. usage:
# job.sh DIR CMD ARGS... As ../job.sh, with the binary built from P2.3.12's commit,
# /root/scratch/p23-run2/markets (sha256 in ../BIN.sha256).
B=/root/scratch/p23-run2/markets
dir=$1; shift
mkdir -p "$dir"
t0=$(date +%s.%N)
"$B" "$@" > "$dir/out.txt" 2> "$dir/err.txt"
rc=$?
t1=$(date +%s.%N)
echo "$rc $(awk -v a="$t0" -v b="$t1" "BEGIN{printf \"%.3f\", b-a}")" > "$dir/exit"
