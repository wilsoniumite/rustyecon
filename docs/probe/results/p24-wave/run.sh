#!/bin/bash
# P2.4 (label run, 2026-09-30): runs the B-D waves' job list, 46 jobs at a time, each line as it is
# (-d '\n' turns off xargs' quote processing; p23-wave/rerun/README.md). Usage (WSL):
#   bash run.sh jobs.txt > run.log 2>&1
xargs -d '\n' -P 46 -I{} bash -c '{}' < "$1"
echo "xargs exit $?"
