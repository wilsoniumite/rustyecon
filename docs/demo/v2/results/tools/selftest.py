"""D2.4 (label run, 2026-09-30): the scorer's self-test, before the wave. It writes the mirror's
own registered outputs in gather.py's tables, as if the engine had run exactly the mirror, and
score.py must then pass every line: each run's class, ticks to tolerance, dead ticks and baskets
the mirror's (the mirror's names mapped to the harness's), mode A PASS and every kick set PASS at
the mirror's base growth, and the long run the registered table. It checks the name map, the
tables' columns and the scoring code, not the engine.

usage: python3 selftest.py REG JSONL DEST
"""
import csv
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
REG, JSONL, DEST = sys.argv[1:4]
os.makedirs(DEST, exist_ok=True)
MARKETS = ("labour", "land", "fodder", "horse", "traction", "good")
DEADM = dict(L="labour", R="land", Fo="fodder", H="traction", G="good")
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


COLS = ["class", "d0", "max_W", "last", "in_tol_from", "dead", "dead_W", "dead_F", "peak_dhat",
        "peak_tick", "low_baskets", "transfer_short_ticks", "heads_low", "heads_high",
        "in_5pct_from", "paper_ticks", "no_order_ticks", "idle_ticks", "pk_low",
        "peak_dhat_ex_horse", "withheld", "switches", "markup_low", "stop", "why"]
EXTRA = ([f"dead_{m}" for m in MARKETS] + [f"notrade_{m}" for m in MARKETS]
         + [f"trough_{m}" for m in MARKETS])
cds = set()
with open(os.path.join(DEST, "runs.tsv"), "w") as o:
    o.write("\t".join(["county", "year", "run"] + COLS + EXTRA + ["rc", "wall"]) + "\n")
    for l in open(JSONL):
        r = json.loads(l)
        cds.add((r["key"], r["year"]))
        if r["cls"] == "NOT A TARGET":
            continue
        v = dict(r)
        row = {"class": r["cls"], "d0": r["d0"], "max_W": "-", "last": r["final"] * 1e-3 * 1e3,
               "in_tol_from": "-" if r.get("tin") is None else r["tin"], "dead": r["dead"],
               "dead_W": 0, "dead_F": 0, "peak_dhat": r["peak"], "peak_tick": r["peak_tick"],
               "low_baskets": r["minB"], "transfer_short_ticks": r["short_ticks"],
               "heads_low": r["hmin"], "heads_high": r["hmax"], "in_5pct_from": r["tK5"],
               "paper_ticks": r["paper"], "no_order_ticks": r["zero_orders"],
               "idle_ticks": r["idle"], "pk_low": r["pKmin"], "peak_dhat_ex_horse": r["peakx"],
               "withheld": r["withheld"], "switches": 0, "markup_low": r["mu_min"],
               "stop": "ran", "why": ""}
        ex = [str(r["dead_by"].get(x, 0)) if x else "0" for x in
              [next((k for k, m in DEADM.items() if m == mk), None) for mk in MARKETS]]
        ex += ["0"] * 6 + ["1"] * 6
        o.write("\t".join([r["key"], r["year"], engine_name(r["run"])]
                          + [str(row[c]) for c in COLS] + ex + ["0", "1"]) + "\n")
growth = {(r["county"], r["year"]): r for r in csv.DictReader(open(os.path.join(REG, "growth.csv")))
          if r["b_factor"] == "1.0"}
with open(os.path.join(DEST, "modea.tsv"), "w") as o:
    o.write("county\tyear\tmode_a\tclass\tpeak_dhat\tlast\trc\twall\n")
    for k, y in sorted(cds):
        o.write(f"{k}\t{y}\tPASS\tCONVERGED\t1e-11\t1e-11\t0\t1\n")
with open(os.path.join(DEST, "kicks.tsv"), "w") as o:
    o.write("county\tyear\tkick\tg_tick\tg_year\tthree_t6\tline\trc\twall\n")
    for k, y in sorted(cds):
        g = growth[(k, y)]
        o.write(f"{k}\t{y}\tPASS\t{g['growth_per_tick']}\t{g['growth_per_year']}\t0\tselftest\t0\t1\n")
with open(os.path.join(DEST, "long-run.csv"), "w") as o:
    o.write(open(os.path.join(REG, "long-run.csv")).read())
with open(os.path.join(DEST, "long.tsv"), "w") as o:
    o.write("rc\twall\tout\n0\t1\tselftest\n")
print(f"wrote the mirror's outputs as gathered tables in {DEST}: {len(cds)} county-dates")
