#!/bin/bash
# run (2026-10-01): archive the B-D wave's raw runs to D: with every CSV gzipped (gzip -n: no name
# or time), via a staging copy on WSL's disk, and check that the committed gather regenerates the
# wave's runs.jsonl byte for byte from the staging copy and again from D:.
set -euo pipefail
SRC=/root/scratch/p24-bcd-runs
STG=/root/scratch/p24-bcd-arch
DST=/mnt/d/rustyecon-p24/runs/bcd
G=/mnt/d/rustyecon-wt/p24/docs/probe/results/p24-wave
A=/root/scratch/p24-bcd/runs.jsonl
V=/root/scratch/p24-run-v
date -u +%FT%TZ
rm -rf $STG
cp -a $SRC $STG
find $STG -name '*.csv' -print0 | xargs -0 -P 16 -n 64 gzip -n
echo "staged: $(find $STG -type f | wc -l) files, $(du -sh $STG | cut -f1); csv left $(find $STG -name '*.csv' | wc -l)"
date -u +%FT%TZ
cd $G
python3 gather.py jobs.txt $STG $V/runs-stage.jsonl
if cmp $V/runs-stage.jsonl $A; then echo "STAGING REGENERATES runs.jsonl BYTE FOR BYTE"; else echo "STAGING DIFFERS"; exit 1; fi
date -u +%FT%TZ
if [ -e $DST ]; then echo "$DST exists"; exit 1; fi
mkdir -p $DST
cp -r $STG/. $DST/
echo "copied: $(find $DST -type f | wc -l) files"
date -u +%FT%TZ
python3 gather.py jobs.txt $DST $V/runs-arch.jsonl
if cmp $V/runs-arch.jsonl $A; then echo "ARCHIVE ON D: REGENERATES runs.jsonl BYTE FOR BYTE"; else echo "ARCHIVE DIFFERS"; exit 1; fi
gzip -n -c $A > $DST/runs.jsonl.gz
zcat $DST/runs.jsonl.gz | sha256sum
sha256sum $A $V/runs-arch.jsonl
du -sh $DST
date -u +%FT%TZ
