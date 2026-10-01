"""P2.4 fix round (label fix-report, 2026-10-01), after the result, not a scorer: CT2's commons
price r_o in every CT2 run of the free step's wave, read from the archived runs
(D:/rustyecon-p24/runs/bcd/free/ct2/, the wave's own CSVs and stats.tsv; no run made again).

CT2's class, its ticks to tolerance and its end D-hat read the real side only: the commons' price
and volume are not among its observables (FREE-RULES §5), and its end-state line reads the regime
from the price's sign (A2 (a)). This reads the price itself, against the oracle's.

- r_o* over w is the registration's 25-digit point (docs/probe/free/registered/points_fine.json),
  matched to the run by its star regime and `free.star_over_ref` (7 printed digits). In wage
  units, as the harness's reference (labour's posted price) has it.
- At a Crowded or Enclosed star: g = ln((p.commons/p.labour)/(r_o*/w)) at each CSV row (the CSV is
  written every L/100 ticks and at the last tick); "r_o in" is the first row from which |g| stays
  at most 1e-3 to the end, given as (the row before, that row]; the end gap is g at the last row.
  At the Enclosed star, where unit 1e sets r_o = r, also ln(p.commons/p.land) at the last row: the
  run's commons rent above its own land rent.
- At a Commons star (r_o* = 0): the first row from which p.commons is 0 to the end, and the end.
- Beside each: the run's own `in_tol_from` (the class's tolerance tick, the real side). A dated
  shock's run is 5L/4 ticks with the shock at L/4, where the scored clock restarts; every tick
  here is on the scored clock, as `in_tol_from` is (the CSV's own tick less the date).

Usage (WSL): python3 ct2_ro.py ARCHIVE_CT2_DIR POINTS_FINE_JSON OUT_CSV
"""
import csv
import gzip
import json
import math
import os
import sys
from collections import defaultdict

ROOT, POINTS, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
TOL = 1e-3

pts = json.load(open(POINTS))
stars = {}  # (regime, ro_over_w float) -> (key, float r/w)
for k, v in pts.items():
    if k.startswith("CT2 ") and v["regime"] in ("Crowded", "Enclosed"):
        stars[k] = (v["regime"], float(v["ro_over_w"]), float(v["r_over_w"]))


def star_for(regime, printed):
    m = [(k, s) for k, s in stars.items() if s[0] == regime and abs(s[1] / printed - 1) < 5e-7]
    if len(m) != 1:
        raise SystemExit(f"no unique point for {regime} {printed}: {m}")
    return m[0]


rows_out = []
for dp, dn, fn in sorted(os.walk(ROOT)):
    dn.sort()
    gz = [f for f in fn if f.endswith(".csv.gz")]
    if not gz or "out.txt" not in fn:
        continue
    rel = os.path.relpath(dp, ROOT)
    st = {}
    for line in open(os.path.join(dp, "stats.tsv")):
        p = line.rstrip("\n").split("\t")
        if len(p) == 4 and p[1].startswith("free."):
            st[(p[1], p[2])] = p[3]
    out = list(csv.reader(open(os.path.join(dp, "out.txt")), delimiter="\t"))
    o = dict(zip(out[0], out[1]))
    regime = st[("free.regime_star", "-")]
    with gzip.open(os.path.join(dp, gz[0]), "rt") as f:
        r = csv.reader(f)
        h = next(r)
        it, il, ild, ic = h.index("tick"), h.index("p.labour"), h.index("p.land"), h.index("p.commons")
        data = [(int(x[it]), float(x[il]), float(x[ild]), float(x[ic])) for x in r]
    # A dated shock restarts the scored clock at L/4 (the run is 5L/4 ticks; the CSV keeps the
    # absolute tick): every tick below is on the scored clock, as in_tol_from is.
    date = (data[-1][0] + 1) // 5 if o["run"].endswith("@dated") else 0
    if date and date not in [x[0] for x in data]:
        raise SystemExit(f"{rel}: the date {date} is not a CSV row")
    data = [(t - date, a, b, c) for t, a, b, c in data]
    rec = {"dir": rel, "run": o["run"], "class": o["class"], "in_tol_from": o["in_tol_from"],
           "regime_star": regime, "regime_end": st[("free.regime_end", "-")], "rows": len(data),
           "date": date, "last_tick": data[-1][0]}
    if regime == "Commons":
        k = len(data)
        while k > 0 and data[k - 1][3] == 0.0:
            k -= 1
        rec.update(point="ro*=0", ro_end_over_w=data[-1][3] / data[-1][1],
                   ro_in_after=(data[k - 1][0] if k > 0 else ""),
                   ro_in_at=(data[k][0] if k < len(data) else "never"))
    else:
        key, (reg, row, rw) = star_for(regime, float(st[("free.star_over_ref", "commons")]))
        g = [(t, math.log((c / w) / row) if c > 0 else -math.inf) for t, w, _, c in data]
        k = len(g)
        while k > 0 and abs(g[k - 1][1]) <= TOL:
            k -= 1
        rec.update(point=key, ro_star_over_w=repr(row), ro_end_over_w=repr(data[-1][3] / data[-1][1]),
                   end_gap=g[-1][1], ro_in_after=(g[k - 1][0] if k > 0 else ""),
                   ro_in_at=(g[k][0] if k < len(g) else "never"),
                   ro_over_r_end=(math.log(data[-1][3] / data[-1][2]) if regime == "Enclosed" else ""))
    rows_out.append(rec)

cols = ["dir", "run", "class", "in_tol_from", "regime_star", "regime_end", "point", "rows",
        "date", "last_tick", "ro_star_over_w", "ro_end_over_w", "end_gap", "ro_over_r_end", "ro_in_after",
        "ro_in_at"]
with open(OUT, "w", newline="") as f:
    w = csv.DictWriter(f, cols, lineterminator="\n")
    w.writeheader()
    for x in rows_out:
        w.writerow({c: x.get(c, "") for c in cols})

print(f"CT2 runs read: {len(rows_out)} (every archived CT2 run with a CSV)")
by = defaultdict(list)
for x in rows_out:
    by[(x["regime_star"], x["point"])].append(x)
for (reg, key), xs in sorted(by.items()):
    conv = [x for x in xs if x["class"] == "CONVERGED"]
    print(f"\n{reg} star, {key}: {len(xs)} runs, {len(conv)} CONVERGED")
    if reg == "Commons":
        end0 = sum(1 for x in conv if x["ro_end_over_w"] == 0.0)
        never = sum(1 for x in conv if x["ro_in_at"] == "never")
        late = [x for x in conv if x["ro_in_at"] not in ("never",) and int(x["ro_in_at"]) > int(x["in_tol_from"])]
        print(f"  CONVERGED runs ending with the commons free (r_o = 0): {end0} of {len(conv)}; "
              f"free from a row to the end: {len(conv) - never}")
        print(f"  runs whose commons is free to the end only from a row after in_tol_from: {len(late)}")
        continue
    gaps = [abs(x["end_gap"]) for x in conv]
    within = sum(1 for v in gaps if v <= TOL)
    print(f"  r_o*/w {xs[0]['ro_star_over_w']}; |end gap| in log: min {min(gaps):.3e}, "
          f"max {max(gaps):.3e}; within 1e-3 at the end: {within} of {len(conv)}")
    if reg == "Enclosed":
        orr = [x["ro_over_r_end"] for x in conv]
        print(f"  ln(r_o/r) at the end, the run's own land rent: min {min(orr):.3e}, max {max(orr):.3e}")
    late = []
    for x in conv:
        if x["ro_in_at"] == "never":
            continue
        lo = int(x["ro_in_after"]) if x["ro_in_after"] != "" else -1
        if lo >= int(x["in_tol_from"]):
            late.append((lo - int(x["in_tol_from"]), x))
    print(f"  CONVERGED runs whose r_o enters 1e-3 only after the class's tolerance tick "
          f"(its row before in_tol_from or later): {len(late)} of {len(conv) - sum(1 for x in conv if x['ro_in_at'] == 'never')}")
    rat = sorted(int(x["ro_in_after"]) / int(x["in_tol_from"]) for x in conv
                 if x["ro_in_at"] != "never" and x["ro_in_after"] != "")
    if rat:
        print(f"  r_o's entry over in_tol_from, at least (the row before / in_tol_from): "
              f"min {rat[0]:.2f}, median {rat[len(rat) // 2]:.2f}, max {rat[-1]:.2f}")
    for d, x in sorted(late, key=lambda t: -t[0])[:6]:
        print(f"    {x['dir']}: in_tol_from {x['in_tol_from']}, r_o in ({x['ro_in_after']}, {x['ro_in_at']}], "
              f"end gap {x['end_gap']:.3e}")
