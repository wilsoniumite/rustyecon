#!/bin/bash
# fix-report (2026-10-01): the B-D wave's own start and end records, kept. archive.sh copied the
# raw runs to D: with `cp -r`, which drops mtimes, so the archive's file times are the copy's
# (2026-10-01 06:30 UTC on), not the run's, and the per-job timing check (jobcheck.out) can no
# longer be re-derived from it. The runner's four records survive only in /root/scratch/p24-bcd;
# here they are copied with `cp -a` (times kept) beside the archive and listed with full times.
set -eu
S=/root/scratch/p24-bcd
D=/mnt/d/rustyecon-p24/runs/bcd/wave-records
mkdir -p $D
cp -a $S/start.txt $S/end.txt $S/run.log $S/bin.sha256 $D/
for f in start.txt end.txt run.log bin.sha256; do
  printf '%s\t%s\t%s\n' "$f" "$(date -u -r $S/$f +%FT%T.%NZ)" "$(date -u -r $D/$f +%FT%T.%NZ)"
done
echo "--- contents"
for f in start.txt end.txt run.log bin.sha256; do echo "$f: $(cat $D/$f)"; cmp $S/$f $D/$f && echo "  same bytes as $S/$f"; done
sha256sum $S/markets
