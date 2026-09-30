#!/usr/bin/env bash
# D2.4 (label run, 2026-09-30): run the wave, jobs.txt's lines, 46 at a time on WSL's 48 threads
# (every core but two). usage: wave.sh JOBS LOG
# Each line is one job (job.sh or longjob.sh); each job writes its own exit file, so the scorer
# sees a job that failed. The log gets the start and end times.
JOBS=$1
LOG=$2
echo "start $(date -u +%Y-%m-%dT%H:%M:%SZ) $(wc -l <"$JOBS") jobs" >>"$LOG"
xargs -P 46 -d '\n' -I{} bash -c '{}' <"$JOBS"
echo "end $(date -u +%Y-%m-%dT%H:%M:%SZ) exit $?" >>"$LOG"
