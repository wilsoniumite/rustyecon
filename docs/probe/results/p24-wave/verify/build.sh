#!/bin/bash
# run (2026-10-01): a fresh WSL release build of the B-D wave's commit 2b68736, from a git archive
# export, in a target directory of its own; compares its markets with the wave's binary.
set -e
rm -rf /root/scratch/p24-exportC /root/scratch/p24-exportC-target
mkdir -p /root/scratch/p24-exportC
tar -xf /mnt/d/rustyecon-p24/run/exportC-2b68736.tar -C /root/scratch/p24-exportC
cd /root/scratch/p24-exportC
date -u +%FT%TZ
CARGO_TARGET_DIR=/root/scratch/p24-exportC-target cargo build --release -p rustyecon-probe 2>&1 | tail -3
date -u +%FT%TZ
sha256sum /root/scratch/p24-exportC-target/release/markets /root/scratch/p24-bcd/markets
cat /root/scratch/p24-bcd/bin.sha256
