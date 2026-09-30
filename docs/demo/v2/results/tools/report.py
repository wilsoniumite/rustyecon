"""D2.4 (label run, 2026-09-30): the results by county class, for the README. Reported, not
scored: it regroups score.py's tables.

The classes come from the world's own tags (worlds/demo-gb/regions.csv), fixed before the battery's
results were read, in this order of precedence:
- coal and textile: coal or textile at least 0.5 (26 counties);
- London's ring: metro at least 0.1 (7);
- Highland: highland at least 0.5 (5);
- rural: the rest (55).
And by nation (England 41, Wales 13, Scotland 33, Ireland's six Ulster counties).

usage: python3 report.py WORLD RESULTS > by-class.csv (and a text summary on stderr)
"""
import csv
import statistics
import sys

WORLD, RES = sys.argv[1:3]
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


cds = list(csv.DictReader(open(f"{RES}/county-dates.csv")))
lr = list(csv.DictReader(open(f"{RES}/long-run.csv")))
out = csv.writer(sys.stdout, lineterminator="\n")
out.writerow(["grouping", "class", "counties", "county_dates", "GO", "runs", "converged",
              "class_differs", "tin_over_10pct", "tin_over_25pct", "dead_misses", "baskets_misses",
              "kicks_pass", "E5_pass", "dhat_median_engine", "dhat_median_mirror",
              "dhat_max_engine", "dhat_max_mirror", "heads_low_engine", "heads_low_mirror",
              "shortfall_ticks_engine", "shortfall_ticks_mirror"])
for grouping, of in (("kind", kind), ("nation", lambda k: tags[k]["nation"])):
    classes = sorted({of(k) for k in tags}, key=lambda c: -sum(1 for k in tags if of(k) == c))
    for c in classes:
        ks = {k for k in tags if of(k) == c}
        cs = [r for r in cds if r["county"] in ks]
        ls = [r for r in lr if r["county"] in ks]
        s = lambda col: sum(int(r[col]) for r in cs)  # noqa: E731
        med = lambda col: statistics.median(float(r[col]) for r in ls)  # noqa: E731
        out.writerow([grouping, c, len(ks), len(cs), sum(1 for r in cs if r["GO"] == "GO"),
                      s("runs"), s("converged"), s("class_differs"), s("tin_over_10pct"),
                      s("tin_over_25pct"), s("dead_misses"), s("baskets_misses"),
                      sum(1 for r in cs if r["E4"] == "PASS"),
                      sum(1 for r in ls if r["E5"] == "PASS"),
                      f"{med('dhat_median_e'):.1f}", f"{med('dhat_median_m'):.1f}",
                      f"{max(float(r['dhat_max_e']) for r in ls):.1f}",
                      f"{max(float(r['dhat_max_m']) for r in ls):.1f}",
                      f"{min(float(r['heads_min_e']) for r in ls):.4f}",
                      f"{min(float(r['heads_min_m']) for r in ls):.4f}",
                      sum(int(r["shortfall_e"]) for r in ls),
                      sum(int(r["shortfall_m"]) for r in ls)])
