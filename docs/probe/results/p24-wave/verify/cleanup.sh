#!/bin/bash
# run (2026-10-01): the B-D wave's raw runs are archived on D: and regenerate runs.jsonl byte for
# byte (archive.log); delete the WSL copies, the staging copy, the preflight's runs and this run's
# own export build.
set -e
echo "source files $(find /root/scratch/p24-bcd-runs -type f | wc -l); archive files $(find /mnt/d/rustyecon-p24/runs/bcd -type f -not -name runs.jsonl.gz | wc -l)"
du -sh /root/scratch/p24-bcd-runs /root/scratch/p24-bcd-arch /root/scratch/p24-bcd-pf /root/scratch/p24-exportC /root/scratch/p24-exportC-target
rm -rf /root/scratch/p24-bcd-runs /root/scratch/p24-bcd-arch /root/scratch/p24-bcd-pf /root/scratch/p24-exportC /root/scratch/p24-exportC-target
ls -d /root/scratch/p24-* 
df -h /root/scratch | tail -1
