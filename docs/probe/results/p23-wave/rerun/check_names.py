"""P2.3.13 (label run, 2026-09-30): every run and kick set in runs.jsonl ran the job its list
named: the name `markets` printed equals the job's name, as parsed from the job list. (The wave's
runner stripped the jobs' quotes; a name whose brackets or star had matched a file would have run
as another name.) Usage (WSL): python3 check_names.py RUNS.jsonl"""
import json
import sys

n = bad = 0
for line in open(sys.argv[1]):
    r = json.loads(line)
    if r.get("cmd") in ("run", "kick") and r.get("run") is not None:
        n += 1
        if r["run"] != r["names"][0]:
            bad += 1
            print("DIFFERS", r["rel"], r["run"], r["names"][0])
    elif r.get("cmd") in ("run", "kick"):
        bad += 1
        print("NO RUN", r["rel"], r.get("rc"), r.get("missing"))
print(f"{n} runs and kick sets checked; {bad} differ or did not run")
