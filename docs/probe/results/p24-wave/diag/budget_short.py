"""P2.4 (label run, 2026-10-01), after the free step's wave was scored, not a scorer: CT2's runs in
which the engine's budget chain cut a pop's commons bid (FREE-RULES: the commons' budget is
min(r_o*bid, what the baskets leave), so the orders are the mirror's only while the coin covers
the bid), read from the harness's `ration.budget_short` of the commons' buys in each run's
stats, with each run's class and, where the free scorer has them, its scored lines.
Usage (WSL): python3 budget_short.py RUNS.jsonl FREE_LINES.csv.gz"""
import csv
import gzip
import json
import sys
from collections import Counter

runs, lines = sys.argv[1], sys.argv[2]
hit, per_set = [], Counter()
for l in open(runs):
    r = json.loads(l)
    if r.get("key") != "free" or r.get("inst") != "ct2" or r.get("cmd") != "run":
        continue
    st = r.get("stats", {})
    short = sum((v or 0) for k, v in st.items() if k.startswith("ration.budget_short|commons/") and k.endswith("/buy"))
    per_set[(r["set"], short > 0)] += 1
    if short > 0:
        hit.append((r["set"], r.get("setting"), r["names"][0], r.get("class"), short))
print("CT2 runs by set, and whether a commons buy was cut by its budget:")
for k in sorted(per_set):
    print(f"  {k[0]:10} {'cut' if k[1] else 'never cut'}: {per_set[k]}")
st = Counter()
with gzip.open(lines, "rt", newline="") as f:
    rows = [x for x in csv.DictReader(f) if x["inst"] == "ct2"]
for s, setting, name, cls, short in hit:
    grp = [x for x in rows if x["set"] == s and x["run"] == name
           and (setting is None or x["group"].endswith(setting))]
    c = Counter(x["status"] for x in grp)
    st.update(c)
    print(f"{s} {setting or '-'} {name}: {cls}; budget short {short:.6g}; its lines: {dict(sorted(c.items()))}")
print(f"their lines in all: {dict(sorted(st.items()))}")
