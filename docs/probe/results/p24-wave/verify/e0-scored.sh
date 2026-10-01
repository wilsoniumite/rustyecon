#!/bin/bash
# fix-report (2026-10-01): E0's three trace diffs, the build agents' scripts copied unedited, run
# on the binary that ran the B-D waves (/root/scratch/p24-bcd/markets, sha256 aa97c626...88d4),
# each compared byte for byte with the committed e0/tracediff.out. The trap's E0 ran on 9d805e69
# (7ddec38) and the switch's on 3477fe21 (1f38b94); the free step's on this binary already.
set -u
export PYTHONDONTWRITEBYTECODE=1
B=/root/scratch/p24-bcd/markets
E=/mnt/d/rustyecon-p24/fix-report/e0
R=/mnt/d/rustyecon-wt/p24/docs/probe/results
date -u +%FT%TZ
sha256sum $B
for s in p s f; do echo "tracediff_$s.py $(sha256sum $E/tracediff_$s.py | cut -c1-64)"; done
for d in trap switch free; do echo "scan-$d SHA256SUMS not OK: $(cd /mnt/d/rustyecon-p24/scan-$d && sha256sum -c SHA256SUMS 2>/dev/null | grep -vc ': OK$')"; done
cd $E
for k in trap:p switch:s free:f; do
  d=${k%%:*}; s=${k##*:}
  rm -rf /root/scratch/p24-fix-e0-$d
  python3 tracediff_$s.py $B /root/scratch/p24-fix-e0-$d > tracediff-$d.out 2> tracediff-$d.err
  echo "$d rc $?; stderr lines $(wc -l < tracediff-$d.err)"
  if cmp -s tracediff-$d.out $R/$d/e0/tracediff.out; then echo "$d: the same as the committed e0/tracediff.out byte for byte"; else echo "$d: DIFFERS from the committed e0/tracediff.out"; fi
done
rm -rf /root/scratch/p24-fix-e0-trap /root/scratch/p24-fix-e0-switch /root/scratch/p24-fix-e0-free
date -u +%FT%TZ
