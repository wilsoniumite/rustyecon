#!/bin/bash
# P2.3.12 (label run, 2026-09-30): R1 for the dated-commons fix. The 24 jobs of jobs-r1.txt, run by
# the new binary, must give the wave's files byte for byte: out.txt, summary.tsv, stats.tsv, the
# CSV and a kick set's kick-*.tsv. Usage (WSL): bash r1check.sh
N=/root/scratch/p23-r1check
W=/root/scratch/p23-runs
bad=0; n=0
for d in $(awk '{print $2}' jobs-r1.txt); do
  rel=${d#$N/}
  for f in $(cd $N/$rel && ls | grep -v -e '^exit$' -e '^err.txt$'); do
    n=$((n+1))
    if ! cmp -s "$N/$rel/$f" "$W/$rel/$f"; then echo "DIFFERS $rel/$f"; bad=$((bad+1)); fi
  done
done
echo "R1 check: $n files compared, $bad differ"
