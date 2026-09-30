"""P2.3.13 (label run, 2026-09-30): the jobs the wave's runner never started.

The wave ran `xargs -P 46 -I{} bash -c '{}' < jobs.txt` (README.md, "Files"). xargs strips the
quotes of its input lines even with -I, so every job whose run name holds a parenthesis (JA, JB, N,
RC, RW, joint, cycle) reached bash unquoted and was a syntax error: job.sh never ran, and the job's
directory has no exit file. Names with brackets and stars reached bash unquoted too; they ran, as
bash leaves a glob that matches nothing as it is, and gather.py checks every run's name against
its job. This script lists, from the job lists, every job whose directory has no exit file, and
writes jobs-missing.txt: each as its list wrote it (the wave's job script for the wave's jobs, the
rerun's for the rerun's and the R1 check's). They run with `xargs -d '\n'`, which takes each line
as it is, quotes and all. Usage (WSL): python3 make_missing.py (in this directory)"""
import os
import shlex

lines = []
for path in ("../jobs.txt", "jobs-rerun.txt", "jobs-r1.txt"):
    rerun_dirs = set()
    if path == "../jobs.txt":
        rerun_dirs = {shlex.split(l)[1] for l in open("jobs-rerun.txt") if l.strip()}
    for l in open(path):
        if not l.strip():
            continue
        d = shlex.split(l)[1]
        if d in rerun_dirs:
            continue   # the rerun's line is the one to run
        if not os.path.exists(os.path.join(d, "exit")):
            lines.append(l.rstrip("\n"))
open("jobs-missing.txt", "w", newline="\n").write("\n".join(lines) + "\n")
print(len(lines), "jobs never started")
