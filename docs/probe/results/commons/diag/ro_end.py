"""run (P2.3.15, 2026-09-30), a diagnostic after scoring: the shadow rent at the end of the runs at
Crowded targets, against the oracle's. The scorer compares the engine's r_o/r on its last tick
(at L) with the mirror's `ro_end`, taken where the mirror stopped early (2,000 ticks within 1e-6 of
its target, from tick 4,000). This prints, for every such line, both against the oracle's r_o
(`ro_star`, the registered files'), and the mirror's stopping tick. Usage (WSL): python3 ro_end.py
LINES.csv (the scorer's lines.csv)"""
import csv
import json
import sys

REG = "/mnt/d/rustyecon-wt/p23/docs/probe/commons/registered/"
T3S = "/mnt/d/rustyecon-wt/p23/docs/probe/results/commons/tier3s/tier3s.jsonl"
INV = {"C1": "c1", "C2": "c2", "C1/chi025": "c1n"}
SETS = {"battery": "L", "tier3x10": "10L", "negctl": "L"}
DESKS = ["manufactures", "food", "care", "shelter", "mach"]


def name(run):
    n = run.replace("exit.To=", "commons=")
    for d in DESKS:
        n = n.replace(f"coin.{d}*", f"coin.desk.{d}*")
    return n


star, stop = {}, {}
for f in ("battery_v3.jsonl", "families_v2.jsonl", "tpy_v2.jsonl", "negctl.jsonl"):
    for line in open(REG + f):
        r = json.loads(line)
        k = (INV[r["name"]], SETS.get(r["set"], r["set"]), name(r["run"]))
        star[k], stop[k] = r.get("ro_star"), r.get("ticks_run")
for line in open(T3S):
    r = json.loads(line)
    k = (INV[r["name"]], "tier3s" if r["set"] == "tier3s" else "tier3s10L", r["run"])
    star[k], stop[k] = r["runner"]["ro_star"], r["runner"]["ticks_run"]
rows = [r for r in csv.DictReader(open(sys.argv[1])) if r["what"].startswith("r_o/r at the end")]
we = wm = 0.0
fails = []
for r in rows:
    k = (r["inst"], r["set"], r["run"])
    e, m, s = float(r["engine"]), float(r["registered"]), star[k]
    we, wm = max(we, abs(e / s - 1)), max(wm, abs(m / s - 1))
    if r["status"] == "fail":
        fails.append((k, abs(e / s - 1), abs(m / s - 1), stop[k]))
print(f"{len(rows)} lines at Crowded targets: the engine's r_o/r at L within {we:.2e} of the oracle's "
      f"(relative), the mirror's at its stop within {wm:.2e}")
print(f"{len(fails)} fail the scorer's 1e-9 against the mirror's:")
for k, e, m, s in fails:
    print(f"  {k[0]} {k[1]} {k[2]}: engine {e:.1e} from the oracle, mirror {m:.1e} (stopped at tick {s})")
