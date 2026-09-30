"""D2.4 (label run, 2026-09-30): the engine's battery run by run, in the registered file's rows and
columns (docs/demo/v2/battery-runs.csv): the same county, year, run name (the mirror's) and tier,
with the engine's class, ticks to tolerance, dead, withheld and shortfall ticks, and troughs,
formatted as make_registration.py formatted the mirror's. A registered NOT A TARGET row stays one.
So the two files diff line by line. Reported; score.py scores.

usage: python3 engine_runs.py REG GATHERED > battery-runs.csv
"""
import csv
import math
import os
import sys

REG, G = sys.argv[1:3]
PRICE = dict(w="w", r="r", pf="p[fodder]", pK="p[horse]", ph="p[traction]", p="p[good]", s="s")
STOCK = dict(Hc="heads.capacity", Zs="hours.capacity", Hm="own.maker", F="finished.maker",
             If="stock.fodder", IG="stock.good", Mg="coin.desk.good", Mc="coin.desk.capacity",
             Mm="coin.desk.maker", Mf="coin.desk.fodder", Mw="coin.workers", Mp="coin.provider")


def fac(s):
    x = float(s)
    return str(int(x)) if x == int(x) else s


def engine_name(m):
    if m == "x*/2":
        return m
    if m.startswith("b x"):
        f, when = m[3:].split()
        return f"b*{fac(f)}@{when}"
    k, f = m.rsplit("x", 1)
    if k in ("JA", "JB", "N"):
        return f"{k}({fac(f)})"
    if k in PRICE:
        return f"{PRICE[k]}*{fac(f)}"
    return f"{STOCK[k]}*{fac(f)}"


def fmt(x, p=6):
    if x in (None, "", "-"):
        return ""
    v = float(x)
    if math.isinf(v) or math.isnan(v):
        return str(v)
    if v == int(v) and p == 6:
        return str(int(v))
    return f"{v:.{p}g}"


eng = {(r["county"], r["year"], r["run"]): r
       for r in csv.DictReader(open(os.path.join(G, "runs.tsv")), delimiter="\t")}
w = csv.writer(sys.stdout, lineterminator="\n")
reg = csv.DictReader(open(os.path.join(REG, "battery-runs.csv")))
w.writerow(reg.fieldnames)
for r in reg:
    if r["class"] == "NOT A TARGET":
        w.writerow([r[c] for c in reg.fieldnames])
        continue
    e = eng.get((r["county"], r["year"], engine_name(r["run"])))
    if e is None:
        w.writerow([r["county"], r["year"], r["run"], r["tier"], "MISSING"] + [""] * 7)
        continue
    w.writerow([r["county"], r["year"], r["run"], r["tier"], e["class"], fmt(e["in_tol_from"]),
                fmt(e["dead"]), fmt(e["withheld"]), fmt(e["transfer_short_ticks"]),
                fmt(e["low_baskets"], 4), fmt(e["heads_low"], 4), fmt(e["pk_low"], 4)])
