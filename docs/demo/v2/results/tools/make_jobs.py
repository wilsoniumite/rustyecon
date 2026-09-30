"""D2.4 (label run, 2026-09-30): the wave's job list, WORLD-V2 §11.5's E2–E5 as registration.md
and registration-A1.md set them, on the frozen harness (job.sh) and test binary (longjob.sh).
One process per job, each in its own directory under OUT (/root/scratch/d2-runs/wave).

- long/: the long run, `demo_v2_runs_to_1901` (E5; WORLD-V2 §11.4).
- battery/<key>@<year>/: `horses family battery` at each of the 558 county-dates at its L: P2.2a's
  93 runs less the unfunded b x 2 pair (E3), with summary.tsv, stats.tsv and each run's first and
  last rows (--every 1000000).
- kicks/<key>@<year>/: `horses slowest hold --ticks L --horizon L`, the county-date's base kick set
  and its slowest mode g (E4).
- modea/<key>@<year>/: `horses run hold` at L (E2's mode A).

L per county-date is registration-A1's (lengths.csv): the larger of 84,000 and the engine's
200·tau_max. The county table is the compiler's (`rustyecon worldgen worlds/demo-gb --stage v2a1
--instances`), at COUNTIES. The long job goes first, then the batteries (the longest), then the
kick sets and mode A, so the tail fills with short jobs.

usage: python3 make_jobs.py LENGTHS COUNTIES OUT TOOLS > jobs.txt
"""
import csv
import shlex
import sys

LENGTHS, COUNTIES, OUT, TOOLS = sys.argv[1:5]
L = {}
for r in csv.DictReader(open(LENGTHS)):
    L[f"{r['county']}@{r['year']}"] = int(r["L"])
ids = []
for line in open(COUNTIES):
    if line.startswith("#") or line.startswith("id\t"):
        continue
    ids.append(line.split("\t")[0])
assert len(ids) == 558 and set(ids) == set(L), "the county table and lengths.csv disagree"
job = f"{TOOLS}/job.sh"
q = shlex.quote
print(f"{TOOLS}/longjob.sh {OUT}/long")
for kind in ("battery", "kicks", "modea"):
    for i in ids:
        d = f"{OUT}/{kind}/{i}"
        common = ["--inst", f"demo:{i}", "--counties", COUNTIES, "--ticks", str(L[i])]
        if kind == "battery":
            args = ["family", "battery"] + common + ["--every", "1000000", "--csv", d]
        elif kind == "kicks":
            args = ["slowest", "hold"] + common + ["--horizon", str(L[i]), "--csv", d]
        else:
            args = ["run", "hold"] + common + ["--every", "1000000", "--csv", d]
        print(" ".join([job, d] + [q(a) for a in args]))
