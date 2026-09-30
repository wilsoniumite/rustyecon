#!/bin/bash
# P2.3.13 (label run, 2026-09-30): run the jobs the wave's runner never started (make_missing.py),
# 46 at a time, each line as it is: -d '\n' turns off xargs' quote processing. Usage (WSL):
# bash run_missing.sh (in this directory)
xargs -d '\n' -P 46 -I{} bash -c '{}' < jobs-missing.txt
