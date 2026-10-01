#!/bin/bash
# run (2026-10-01): the committed gather and scorers regenerate the wave's runs.jsonl and score
# outputs byte for byte.
set -u
V=/root/scratch/p24-run-v
W=/root/scratch/p24-bcd
rm -rf $V; mkdir -p $V
cd /mnt/d/rustyecon-wt/p24/docs/probe/results/p24-wave
sha256sum -c SHA256SUMS | grep -v ': OK$'
echo "p23 runs: $(zcat /mnt/d/rustyecon-p23/runs/runs.jsonl.gz | sha256sum)"
date -u +%FT%TZ
python3 gather.py jobs.txt /root/scratch/p24-bcd-runs $V/runs.jsonl
date -u +%FT%TZ
if cmp $V/runs.jsonl $W/runs.jsonl; then echo "GATHER REGENERATES runs.jsonl BYTE FOR BYTE"; fi
sha256sum $V/runs.jsonl $W/runs.jsonl
for k in trap switch free; do
  python3 score_$k.py $W/runs.jsonl $V/score-$k > $V/score-$k.out 2> $V/score-$k.err; echo "score_$k exit $?"
  cmp $V/score-$k.out $W/score-$k.out && echo "score-$k.out same"
  diff -r $V/score-$k $W/score-$k && echo "score-$k/ same ($(ls $V/score-$k | wc -l) files)"
  cat $V/score-$k.err
done
date -u +%FT%TZ
