"""D2.4 (label run, 2026-09-30): capital's lag over the history, county by county, the engine's
beside the mirror's (WORLD-V2 §6, §8; D-G14). Reported, not scored.

From the long run's every 13th tick (the engine's series.csv; the mirror's long-c2g-final.json,
whose series are sampled at the same ticks) of the installed heads over their equilibrium at the
params in force: the lowest and its year; the years spent below 0.95 and below 0.90; the year from
which the herd stays within 5% to the first tick of 1901 ("-" if it is not within 5% then); the
herd in 1901; and the largest difference between the engine's and the mirror's ratio over the
run. The county's kind is report.py's.

usage: python3 lag.py WORLD SERIES_CSV MIRROR_LONG_JSON > capital-lag.csv
"""
import csv
import json
import sys

WORLD, SERIES, MLONG = sys.argv[1:4]
sys.path.insert(0, __file__.rsplit("/", 1)[0])
tags = {r["key"]: r for r in csv.DictReader(open(f"{WORLD}/regions.csv"))}


def kind(k):
    t = tags[k]
    if float(t["coal"]) >= 0.5 or float(t["textile"]) >= 0.5:
        return "coal and textile"
    if float(t["metro"]) >= 0.1:
        return "London's ring"
    if float(t["highland"]) >= 0.5:
        return "Highland"
    return "rural"


eng = {}
for r in csv.DictReader(open(SERIES)):
    eng.setdefault(r["county"], []).append((int(r["tick"]), float(r["heads_over_oracle"])))
mir = {r["key"]: [(s[0], s[5]) for s in r["series"]] for r in json.load(open(MLONG))}


def stats(xs):
    lo_t, lo = min(xs, key=lambda p: p[1])
    step = 13 / 52
    below95 = sum(step for _, v in xs if v < 0.95)
    below90 = sum(step for _, v in xs if v < 0.90)
    out = [t for t, v in xs if abs(v - 1.0) > 0.05]
    within = "-" if xs[-1][0] in out else (
        f"{1750 + (out[-1] + 13) / 52:.1f}" if out else "1750.0")
    return lo, 1750 + lo_t / 52, below95, below90, within, xs[-1][1]


w = csv.writer(sys.stdout, lineterminator="\n")
w.writerow(["county", "name", "kind", "heads_low_e", "year_low_e", "heads_low_m", "year_low_m",
            "years_below_0.95_e", "years_below_0.95_m", "years_below_0.90_e",
            "years_below_0.90_m", "within_5pct_from_e", "within_5pct_from_m", "heads_1901_e",
            "heads_1901_m", "largest_difference"])
for k in sorted(eng):
    e, m = eng[k], mir[k]
    assert [t for t, _ in e] == [t for t, _ in m], k
    a, b = stats(e), stats(m)
    diff = max(abs(x[1] - y[1]) for x, y in zip(e, m))
    w.writerow([k, tags[k]["name"], kind(k), f"{a[0]:.4f}", f"{a[1]:.1f}", f"{b[0]:.4f}",
                f"{b[1]:.1f}", f"{a[2]:.1f}", f"{b[2]:.1f}", f"{a[3]:.1f}", f"{b[3]:.1f}", a[4],
                b[4], f"{a[5]:.4f}", f"{b[5]:.4f}", f"{diff:.2e}"])
