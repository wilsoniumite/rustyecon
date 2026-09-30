"""D2.4 (label run, 2026-09-30): score the wave against the registration, line by line.

The registered predictions are docs/demo/v2/'s CSVs (sha256 in SHA256SUMS) and, for the battery's
other readouts, the mirror's battery.jsonl (sha256 fixed by registration.md §1); registration-A1.md
sets L per county-date (lengths.csv, sha256 in SHA256SUMS-A1). The engine's numbers are gather.py's
tables. WORLD-V2 §11.5's lines, as registration.md §4 quotes them:

- E2, mode A: at every county-date, hold at its L PASSES (every observable within 1e-9 in log
  every tick, every market trading, nothing rationed or spoiled).
- E3, the battery, run by run against the mirror's run of the same name: the class equal; ticks to
  tolerance within 10%, but at most three runs a county-date within 25% (a run beyond 10% is
  charged once; none may pass 25%); dead ticks within 5 or 10%, whichever is larger; the lowest
  baskets over Y* within 1e-3. The engine's runs must be the registered scored runs exactly.
- E4, the kicks: every county-date's base kick set PASSES and its slowest mode g is below 1 a tick.
- E5, the long run, per county: dead and idle ticks the mirror's (0); withheld ticks within 5;
  shortfall ticks within 5 or 1%, whichever is larger; D-hat's median and largest within 5%
  (relative); the heads' lowest over the oracle within 1e-3 absolute.
- Refutation: a class other than the mirror's at any county-date; any dead tick in the long run; a
  runaway; a CONVERGED run ending more than 1e-12 off its oracle point; a kick set that grows (it
  FAILS, or g is 1 or more a tick).

A county-date is GO when every one of its runs CONVERGES and its kick set decays. Everything else
the registration predicts (§3.3's and §3.4's aggregates) is computed the same way from both sides
and reported, not scored. E0 and E1 are not this script's (registration-A1.md; the gates).

usage: python3 score.py REG JSONL GATHERED OUT
  REG       docs/demo/v2 (the registration, its CSVs, registration-A1.md and lengths.csv)
  JSONL     the mirror's battery.jsonl (D:/rustyecon-d2/design/out-mirror/battery.jsonl)
  GATHERED  gather.py's DEST
  OUT       where the result tables go (docs/demo/v2/results)
"""
import collections
import csv
import hashlib
import json
import math
import os
import sys

REG, JSONL, G, OUT = sys.argv[1:5]
os.makedirs(OUT, exist_ok=True)
JSONL_SHA = "f1bb332c98f26ae9bc474f7cc67fd71d754792b502055a029603a69911d0dbef"
TPY = 52.0
LINES = []          # (id, what, registered, engine, tolerance, pass)


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def line(i, what, reg, eng, tol, ok):
    LINES.append((i, what, reg, eng, tol, "PASS" if ok else "FAIL"))
    return ok


def num(x):
    try:
        v = float(x)
    except (TypeError, ValueError):
        return None
    return v


# ---- the registration's files, checked
for sums in ("SHA256SUMS", "SHA256SUMS-A1"):
    for l in open(os.path.join(REG, sums)):
        want, name = l.split()
        got = sha(os.path.join(REG, name))
        assert got == want, f"{name}: sha256 {got}, registered {want}"
assert sha(JSONL) == JSONL_SHA, "battery.jsonl is not the registered file"

# ---- names: the mirror's run names to the harness's (P2.2a's analyze.py map)
PRICE = dict(w="w", r="r", pf="p[fodder]", pK="p[horse]", ph="p[traction]", p="p[good]", s="s")
STOCK = dict(Hc="heads.capacity", Zs="hours.capacity", Hm="own.maker", F="finished.maker",
             If="stock.fodder", IG="stock.good", Mg="coin.desk.good", Mc="coin.desk.capacity",
             Mm="coin.desk.maker", Mf="coin.desk.fodder", Mw="coin.workers", Mp="coin.provider")
DEADM = dict(L="labour", R="land", Fo="fodder", H="traction", G="good")


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


# ---- the mirror: every run, keyed (county, year, engine name)
MIR = {}
for l in open(JSONL):
    r = json.loads(l)
    r["tier"] = str(r["tier"])
    MIR[(r["key"], r["year"], engine_name(r["run"]))] = r
assert len(MIR) == 51894, len(MIR)
# the registered per-run CSV agrees with the JSONL it was written from
for r in csv.DictReader(open(os.path.join(REG, "battery-runs.csv"))):
    m = MIR[(r["county"], r["year"], engine_name(r["run"]))]
    assert r["class"] == m["cls"], r
    if m["cls"] != "NOT A TARGET":
        assert r["ticks_to_tol"] == ("" if m.get("tin") is None else str(m["tin"])), r
        assert int(r["dead"]) == m["dead"], r
NAMES = {r["county"]: r["name"] for r in csv.DictReader(
    open(os.path.join(REG, "battery-county-dates.csv")))}
LEN = {(r["county"], r["year"]): int(r["L"]) for r in csv.DictReader(
    open(os.path.join(REG, "lengths.csv")))}

# ---- the engine
ENG = {}
for r in csv.DictReader(open(os.path.join(G, "runs.tsv")), delimiter="\t"):
    ENG[(r["county"], r["year"], r["run"])] = r
MODEA = {(r["county"], r["year"]): r for r in csv.DictReader(
    open(os.path.join(G, "modea.tsv")), delimiter="\t")}
KICKS = {(r["county"], r["year"]): r for r in csv.DictReader(
    open(os.path.join(G, "kicks.tsv")), delimiter="\t")}
CDS = sorted({(k, y) for k, y, _ in MIR})
assert len(CDS) == 558

# ---- E2, mode A
passed = [cd for cd in CDS if MODEA.get(cd, {}).get("mode_a") == "PASS"
          and MODEA[cd]["rc"] == "0"]
worst = max((math.inf if num(MODEA[cd]["peak_dhat"]) is None else num(MODEA[cd]["peak_dhat"]), cd)
            for cd in CDS if cd in MODEA)
line("E2", "mode A PASSES at every county-date at its L", "558 of 558",
     f"{len(passed)} of 558; largest gap in log {worst[0] * 1e-3:.2e} ({worst[1][0]}@{worst[1][1]})",
     "every observable within 1e-9 in log", len(passed) == 558)

# ---- E3, the battery, run by run
rows = []
by_cd = collections.defaultdict(list)
refute = collections.defaultdict(list)
for key in sorted(MIR):
    m = MIR[key]
    if m["cls"] == "NOT A TARGET":
        if key in ENG:
            refute["extra"].append(key)
        continue
    e = ENG.get(key)
    k, y, n = key
    if e is None:
        rows.append(dict(county=k, year=y, run=n, tier=m["tier"], cls_e="MISSING", cls_m=m["cls"]))
        by_cd[(k, y)].append(rows[-1])
        continue
    tin_e = None if e["in_tol_from"] in ("-", "") else int(e["in_tol_from"])
    tin_m = m.get("tin")
    if tin_e is not None and tin_m is not None:
        rel = abs(tin_e - tin_m) / max(tin_m, 1)
    else:
        rel = 0.0 if tin_e == tin_m else math.inf
    dead_e, dead_m = int(e["dead"]), m["dead"]
    b_e, b_m = num(e["low_baskets"]), m["minB"]
    row = dict(county=k, year=y, run=n, tier=m["tier"], cls_e=e["class"], cls_m=m["cls"],
               tin_e=tin_e, tin_m=tin_m, tin_rel=rel, dead_e=dead_e, dead_m=dead_m,
               dead_ok=abs(dead_e - dead_m) <= max(5, 0.1 * dead_m),
               baskets_e=b_e, baskets_m=b_m,
               baskets_ok=b_e is not None and abs(b_e - b_m) <= 1e-3,
               withheld_e=int(e["withheld"]), withheld_m=m["withheld"],
               short_e=int(e["transfer_short_ticks"]), short_m=m["short_ticks"],
               idle_e=int(e["idle_ticks"]), idle_m=m["idle"],
               noorder_e=int(e["no_order_ticks"]), noorder_m=m["zero_orders"],
               heads_low_e=num(e["heads_low"]), heads_low_m=m["hmin"],
               heads_high_e=num(e["heads_high"]), heads_high_m=m["hmax"],
               pk_low_e=num(e["pk_low"]), pk_low_m=m["pKmin"],
               markup_low_e=num(e["markup_low"]), markup_low_m=m["mu_min"],
               peakx_e=num(e["peak_dhat_ex_horse"]), peakx_m=m["peakx"],
               k5_e=None if e["in_5pct_from"] in ("-", "") else int(e["in_5pct_from"]),
               k5_m=m.get("tK5"), last_e=num(e["last"]), last_m=m.get("final"),
               dead_W_e=int(e["dead_W"]), stop=e["stop"], rc=e["rc"],
               dead_by_e={x: int(e[f"dead_{DEADM[x]}"]) for x in DEADM},
               dead_by_m=m["dead_by"])
    rows.append(row)
    by_cd[(k, y)].append(row)
    if row["cls_e"] != row["cls_m"]:
        refute["class"].append(key)
    if e["stop"] == "runaway":
        refute["runaway"].append(key)
    if e["class"] == "CONVERGED" and (row["last_e"] is None or row["last_e"] * 1e-3 > 1e-12):
        refute["end"].append(key)
    if e["rc"] != "0":
        refute["job"].append(key)
for key in ENG:
    if key not in MIR:
        refute["extra"].append(key)
scored = [r for r in rows]
n_scored = len(scored)
line("E3", "the engine's runs are the registered scored runs", "51,260 runs; 634 not a target",
     f"{n_scored} registered runs, {sum(1 for r in rows if r['cls_e'] == 'MISSING')} missing, "
     f"{len(refute['extra'])} extra", "exactly",
     n_scored == 51260 and not refute["extra"]
     and not any(r["cls_e"] == "MISSING" for r in rows))

cd_rows = []
for cd in CDS:
    rs = by_cd[cd]
    over10 = [r for r in rs if r.get("tin_rel", math.inf) > 0.10]
    over25 = [r for r in rs if r.get("tin_rel", math.inf) > 0.25]
    cls_bad = [r for r in rs if r["cls_e"] != r["cls_m"]]
    dead_bad = [r for r in rs if not r.get("dead_ok", False)]
    bask_bad = [r for r in rs if not r.get("baskets_ok", False)]
    kick = KICKS.get(cd, {})
    g = num(kick.get("g_tick"))
    kick_ok = kick.get("kick") == "PASS" and g is not None and g < 1.0 and kick.get("rc") == "0"
    conv = all(r["cls_e"] == "CONVERGED" for r in rs)
    tin_ok = not over25 and len(over10) <= 3
    go = conv and kick_ok
    cd_rows.append(dict(county=cd[0], name=NAMES[cd[0]], year=cd[1], L=LEN[cd], runs=len(rs),
                        converged=sum(1 for r in rs if r["cls_e"] == "CONVERGED"),
                        class_differs=len(cls_bad), tin_over_10pct=len(over10),
                        tin_over_25pct=len(over25), dead_misses=len(dead_bad),
                        baskets_misses=len(bask_bad), kick=kick.get("kick", "-"),
                        g_year=kick.get("g_year", "-"), mode_a=MODEA.get(cd, {}).get("mode_a", "-"),
                        E3=("PASS" if not cls_bad and tin_ok and not dead_bad and not bask_bad
                            else "FAIL"),
                        E4="PASS" if kick_ok else "FAIL", GO="GO" if go else "NOT GO"))
    if not kick_ok:
        refute["kick"].append(cd)

n = len(cd_rows)
line("E3", "every run's class the mirror's", "51,260 of 51,260",
     f"{sum(1 for r in rows if r['cls_e'] == r['cls_m'])} of {n_scored}", "exactly",
     not refute["class"])
line("E3", "ticks to tolerance within 10% run for run, at most three runs a county-date within "
     "25%", "at all 558 county-dates",
     f"{sum(1 for c in cd_rows if c['tin_over_25pct'] == 0 and c['tin_over_10pct'] <= 3)} of 558; "
     f"{sum(c['tin_over_10pct'] for c in cd_rows)} runs beyond 10%, "
     f"{sum(c['tin_over_25pct'] for c in cd_rows)} beyond 25%", "10%; three a county-date at 25%",
     all(c["tin_over_25pct"] == 0 and c["tin_over_10pct"] <= 3 for c in cd_rows))
line("E3", "dead ticks within 5 or 10%, whichever is larger", "every run",
     f"{sum(c['dead_misses'] for c in cd_rows)} runs miss", "max(5, 10%)",
     all(c["dead_misses"] == 0 for c in cd_rows))
line("E3", "the lowest baskets over Y* within 1e-3", "every run",
     f"{sum(c['baskets_misses'] for c in cd_rows)} runs miss; largest difference "
     f"{max(abs(r['baskets_e'] - r['baskets_m']) for r in rows if r.get('baskets_e') is not None):.2e}",
     "1e-3", all(c["baskets_misses"] == 0 for c in cd_rows))
line("E4", "every county-date's base kick set decays, g below 1 a tick", "558 of 558",
     f"{sum(1 for c in cd_rows if c['E4'] == 'PASS')} of 558", "PASS and g < 1",
     all(c["E4"] == "PASS" for c in cd_rows))
line("GO", "every county-date GO (every run CONVERGED, its kick set decays)", "558 of 558",
     f"{sum(1 for c in cd_rows if c['GO'] == 'GO')} of 558", "all", all(c["GO"] == "GO" for c in cd_rows))

# ---- E5, the long run
LR_M = {r["county"]: r for r in csv.DictReader(open(os.path.join(REG, "long-run.csv")))}
LR_E = {r["county"]: r for r in csv.DictReader(open(os.path.join(G, "long-run.csv")))} if \
    os.path.exists(os.path.join(G, "long-run.csv")) else {}
long_rc = list(csv.DictReader(open(os.path.join(G, "long.tsv")), delimiter="\t"))[0]
line("E5", "the long run ran and its test passed", "exit 0", f"exit {long_rc['rc']}", "exit 0",
     long_rc["rc"] == "0" and len(LR_E) == 93)
lr_rows = []
E5_BAD = set()
for k in sorted(LR_M):
    m, e = LR_M[k], LR_E.get(k)
    if e is None:
        lr_rows.append(dict(county=k, name=m["name"], E5="FAIL (missing)"))
        continue
    f = lambda c: float(e[c])  # noqa: E731
    g = lambda c: float(m[c])  # noqa: E731
    checks = dict(
        dead=int(e["dead"]) == int(m["dead"]),
        idle=int(e["idle_horse"]) == int(m["idle_horse"]),
        withheld=abs(int(e["withheld"]) - int(m["withheld"])) <= 5,
        shortfall=abs(int(e["shortfall_ticks"]) - int(m["shortfall_ticks"]))
        <= max(5, 0.01 * int(m["shortfall_ticks"])),
        dhat_median=abs(f("dhat_median") - g("dhat_median")) <= 0.05 * g("dhat_median"),
        dhat_max=abs(f("dhat_max") - g("dhat_max")) <= 0.05 * g("dhat_max"),
        heads_min=abs(f("heads_over_oracle_min") - g("heads_over_oracle_min")) <= 1e-3)
    lr_rows.append(dict(
        county=k, name=m["name"], dead_e=e["dead"], dead_m=m["dead"], idle_e=e["idle_horse"],
        idle_m=m["idle_horse"], zero_orders_e=e["zero_orders"], zero_orders_m=m["zero_orders"],
        withheld_e=e["withheld"], withheld_m=m["withheld"], shortfall_e=e["shortfall_ticks"],
        shortfall_m=m["shortfall_ticks"],
        dhat_median_e=f"{f('dhat_median'):.2f}", dhat_median_m=m["dhat_median"],
        dhat_median_rel=f"{f('dhat_median') / g('dhat_median') - 1:+.4f}",
        dhat_max_e=f"{f('dhat_max'):.2f}", dhat_max_m=m["dhat_max"],
        dhat_max_rel=f"{f('dhat_max') / g('dhat_max') - 1:+.4f}",
        heads_min_e=f"{f('heads_over_oracle_min'):.4f}", heads_min_m=m["heads_over_oracle_min"],
        heads_min_diff=f"{f('heads_over_oracle_min') - g('heads_over_oracle_min'):+.5f}",
        dhat_p90_e=f"{f('dhat_p90'):.2f}", dhat_p90_m=m["dhat_p90"],
        dhat_1901_e=f"{f('dhat_1901'):.2f}", dhat_1901_m=m["dhat_1901"],
        lowest_cleared_e=f"{f('lowest_cleared_over_target'):.4f}",
        lowest_cleared_m=m["lowest_cleared_over_target"], lowest_market_e=e["lowest_market"],
        lowest_market_m=m["lowest_market"],
        heads_max_e=f"{f('heads_over_oracle_max'):.4f}", heads_max_m=m["heads_over_oracle_max"],
        heads_1901_e=f"{f('heads_over_oracle_1901'):.4f}", heads_1901_m=m["heads_over_oracle_1901"],
        desk_target_min_e=f"{f('capacity_over_desk_target_min'):.4f}",
        desk_target_min_m=m["capacity_over_desk_target_min"],
        horse_price_max_e=f"{f('horse_price_over_target_max'):.4f}",
        horse_price_max_m=m["horse_price_over_target_max"],
        markup_min_e=f"{f('maker_markup_min'):.4f}", markup_min_m=m["maker_markup_min"],
        quasi_rent_max_e=f"{f('quasi_rent_max'):.4f}", quasi_rent_max_m=m["quasi_rent_max"],
        prices_dhat_median_e=f"{f('prices_dhat_median'):.2f}",
        prices_dhat_median_m=m["prices_dhat_median"],
        heads_dhat_median_e=f"{f('heads_dhat_median'):.2f}",
        heads_dhat_median_m=m["heads_dhat_median"],
        E5="PASS" if all(checks.values()) else "FAIL: " + " ".join(c for c, ok in checks.items()
                                                                   if not ok)))
    E5_BAD.update((k, c) for c, ok in checks.items() if not ok)
    if int(e["dead"]) > 0:
        refute["long_dead"].append(k)
for what, col in (("dead ticks the mirror's (0)", "dead"), ("idle ticks the mirror's (0)", "idle"),
                  ("withheld ticks within 5", "withheld"),
                  ("shortfall ticks within 5 or 1%", "shortfall"),
                  ("D-hat median within 5%", "dhat_median"), ("D-hat largest within 5%", "dhat_max"),
                  ("the heads' lowest over the oracle within 1e-3", "heads_min")):
    bad = [r["county"] for r in lr_rows
           if (r["county"], col) in E5_BAD or r["E5"] == "FAIL (missing)"]
    line("E5", what, "93 of 93 counties", f"{93 - len(bad)} of 93"
         + (f" (misses: {', '.join(NAMES.get(b, b) for b in bad[:12])}"
            + (" …" if len(bad) > 12 else "") + ")" if bad else ""), "", not bad)

# ---- refutations
for what, key in (("a class other than the mirror's at any county-date", "class"),
                  ("any dead tick in the long run", "long_dead"),
                  ("a runaway with psi 0.25", "runaway"),
                  ("a CONVERGED run ending more than 1e-12 off its oracle point", "end"),
                  ("a kick set that grows (FAILS, or g of 1 or more a tick)", "kick")):
    hits = refute[key]
    line("refutation", what, "none", f"{len(hits)}" + (f": {hits[:5]}" if hits else ""), "none",
         not hits)
if refute["job"]:
    line("jobs", "every job exited 0", "all", f"{len(refute['job'])} runs from failed jobs", "",
         False)

# ---- the reported aggregates (registration §3.3, §3.4), both sides by the same code
AGG = []


def agg(what, reg, eng):
    AGG.append((what, reg, eng))


def where(r):
    return f"{NAMES[r['county']]} {r['year']}, {r['run']}"


def side(s):
    """The battery's aggregates for one side, 'e' or 'm'."""
    conv = [r for r in rows if r.get(f"cls_{s}") == "CONVERGED"]
    t = sorted(r[f"tin_{s}"] for r in conv if r.get(f"tin_{s}") is not None)
    out = {}
    out["classes"] = dict(collections.Counter(r.get(f"cls_{s}") for r in rows))
    out["tin"] = (t[len(t) // 2], t[int(0.9 * len(t))], t[-1],
                  where(max(conv, key=lambda r: r[f"tin_{s}"] or 0)))
    tiers = {}
    for tier in ("1", "2", "3", "3S"):
        tt = sorted(r[f"tin_{s}"] for r in conv if r["tier"] == tier and r.get(f"tin_{s}") is not None)
        tiers[tier] = (tt[len(tt) // 2], tt[-1])
    out["tiers"] = tiers
    lasts = [r[f"last_{s}"] for r in conv if r.get(f"last_{s}") is not None]
    out["end"] = max(lasts)
    dd = [r for r in rows if r.get(f"dead_{s}", 0) > 0]
    out["dead_runs"] = len(dd)
    out["dead_tiers"] = sorted({r["tier"] for r in dd})
    top = max(rows, key=lambda r: r.get(f"dead_{s}", 0))
    out["dead_max"] = (top.get(f"dead_{s}"), where(top))
    out["dead_by"] = {x: max(r[f"dead_by_{s}"][x] for r in rows if f"dead_by_{s}" in r)
                      for x in DEADM}
    idle = [r for r in rows if r.get(f"idle_{s}", 0) > 0]
    top = max(rows, key=lambda r: r.get(f"idle_{s}", 0))
    out["idle"] = (len(idle), top.get(f"idle_{s}"), where(top),
                   max(r.get(f"noorder_{s}", 0) for r in rows))
    wh = [r for r in rows if r.get(f"withheld_{s}", 0) > 0]
    top = max(rows, key=lambda r: r.get(f"withheld_{s}", 0))
    out["withheld"] = (len(wh), top.get(f"withheld_{s}"), where(top))
    top = min((r for r in rows if r.get(f"pk_low_{s}") is not None), key=lambda r: r[f"pk_low_{s}"])
    out["pk_low"] = (top[f"pk_low_{s}"], where(top))
    top = min((r for r in rows if r.get(f"markup_low_{s}") is not None),
              key=lambda r: r[f"markup_low_{s}"])
    out["markup_low"] = (top[f"markup_low_{s}"], where(top))
    sh = [r for r in rows if r.get(f"short_{s}", 0) > 0]
    top = max(rows, key=lambda r: r.get(f"short_{s}", 0))
    b2 = [r for r in rows if r["run"].startswith("b*2@")]
    out["short"] = (len(sh), top.get(f"short_{s}"), where(top),
                    sum(1 for r in b2 if r.get(f"short_{s}", 0) > 0), len(b2))
    top = min((r for r in rows if r.get(f"baskets_{s}") is not None), key=lambda r: r[f"baskets_{s}"])
    out["baskets"] = (top[f"baskets_{s}"], where(top))
    lo = min((r for r in rows if r.get(f"heads_low_{s}") is not None), key=lambda r: r[f"heads_low_{s}"])
    hi = max((r for r in rows if r.get(f"heads_high_{s}") is not None), key=lambda r: r[f"heads_high_{s}"])
    out["heads"] = (lo[f"heads_low_{s}"], where(lo), hi[f"heads_high_{s}"], where(hi))
    cap = {}
    for f in ("2", "0.5", "1.1", "0.9"):
        for when in ("genesis", "dated"):
            ys = sorted(r[f"k5_{s}"] / TPY for r in rows
                        if r["run"] == f"b*{f}@{when}" and r.get(f"k5_{s}") is not None)
            cap[(f, when)] = (ys[0], ys[len(ys) // 2], ys[-1]) if ys else None
    out["capital"] = cap
    return out


if rows and all("cls_e" in r for r in rows) and not any(r["cls_e"] == "MISSING" for r in rows):
    A, B = side("m"), side("e")
    agg("classes", A["classes"], B["classes"])
    agg("ticks to tolerance: median, 90th percentile, largest (where)", A["tin"], B["tin"])
    for tier in ("1", "2", "3", "3S"):
        agg(f"Tier {tier}: median and largest ticks to tolerance", A["tiers"][tier], B["tiers"][tier])
    agg("end gap: largest D-hat at the last tick of a CONVERGED run", f"{A['end']:.2e}", f"{B['end']:.2e}")
    agg("runs with a dead tick; their tiers", (A["dead_runs"], A["dead_tiers"]), (B["dead_runs"], B["dead_tiers"]))
    agg("most dead ticks in a run (where)", A["dead_max"], B["dead_max"])
    agg("most dead ticks in one run by market", A["dead_by"], B["dead_by"])
    agg("runs with an idle horse market; most idle ticks (where); most ticks without an order",
        A["idle"], B["idle"])
    agg("runs where the reservation acts; most withheld ticks (where)", A["withheld"], B["withheld"])
    agg("the horse's lowest price over its target (where)", A["pk_low"], B["pk_low"])
    agg("the maker's lowest markup (where)", A["markup_low"], B["markup_low"])
    agg("runs with a shortfall; most shortfall ticks (where); b x 2 runs with one, of all",
        A["short"], B["short"])
    agg("the lowest baskets over Y* (where)", A["baskets"], B["baskets"])
    agg("heads over target: lowest (where), highest (where)", A["heads"], B["heads"])
    for k in A["capital"]:
        agg(f"years to heads within 5% after b x {k[0]} at {k[1]}: least, median, most",
            A["capital"][k], B["capital"][k])
    gm = [float(r["growth_per_year"]) for r in csv.DictReader(open(os.path.join(REG, "growth.csv")))
          if r["b_factor"] == "1.0"]
    ge = [float(KICKS[cd]["g_year"]) for cd in CDS if num(KICKS.get(cd, {}).get("g_year")) is not None]
    agg("base local growth a year (the mirror's linearisation; the engine's kick-set g)",
        (min(gm), max(gm)), (min(ge), max(ge)) if ge else "-")
if LR_E:
    def lr(c, rs):
        return sorted(float(r[c]) for r in rs.values())
    for what, c in (("D-hat county medians: median", "dhat_median"), ("D-hat at 1901: median", "dhat_1901")):
        a, b = lr(c, LR_M), lr(c, LR_E)
        agg(f"long run: {what}", f"{a[len(a) // 2]:.2f}", f"{b[len(b) // 2]:.2f}")

    def top(rs, c, rev=True, k=4):
        p = 1 if c.startswith("dhat") else 3
        xs = sorted(rs.values(), key=lambda r: float(r[c]), reverse=rev)[:k]
        return "; ".join(f"{NAMES.get(r['county'], r['county'])} {float(r[c]):.{p}f}" for r in xs)
    agg("long run: highest county medians of D-hat", top(LR_M, "dhat_median"), top(LR_E, "dhat_median"))
    agg("long run: lowest county median of D-hat", top(LR_M, "dhat_median", False, 1),
        top(LR_E, "dhat_median", False, 1))
    agg("long run: largest D-hat", top(LR_M, "dhat_max", True, 1), top(LR_E, "dhat_max", True, 1))
    agg("long run: largest D-hat at 1901", top(LR_M, "dhat_1901", True, 1), top(LR_E, "dhat_1901", True, 1))

    def low(rs, c, k=3, fmt="{:.3f}"):
        xs = sorted(rs.values(), key=lambda r: float(r[c]))[:k]
        return "; ".join(f"{NAMES.get(r['county'], r['county'])} " + fmt.format(float(r[c]))
                         + (f" {r['lowest_market']}" if c == "lowest_cleared_over_target" else "")
                         for r in xs)
    agg("long run: lowest cleared volume over its oracle's", low(LR_M, "lowest_cleared_over_target"),
        low(LR_E, "lowest_cleared_over_target"))
    agg("long run: heads over their equilibrium, lowest", low(LR_M, "heads_over_oracle_min", 1),
        low(LR_E, "heads_over_oracle_min", 1))
    agg("long run: heads over their equilibrium, highest", top(LR_M, "heads_over_oracle_max", True, 1),
        top(LR_E, "heads_over_oracle_max", True, 1))
    agg("long run: the capacity desk's heads over its own target, lowest",
        low(LR_M, "capacity_over_desk_target_min", 1), low(LR_E, "capacity_over_desk_target_min", 1))
    agg("long run: the horse's price over its target, highest", top(LR_M, "horse_price_over_target_max", True, 1),
        top(LR_E, "horse_price_over_target_max", True, 1))
    agg("long run: the maker's markup, lowest", low(LR_M, "maker_markup_min", 1),
        low(LR_E, "maker_markup_min", 1))
    agg("long run: the horse-day's markup over full cost, highest", top(LR_M, "quasi_rent_max", True, 1),
        top(LR_E, "quasi_rent_max", True, 1))
    agg("long run: shortfall ticks, all counties",
        f"{sum(int(r['shortfall_ticks']) for r in LR_M.values())} in "
        f"{sum(1 for r in LR_M.values() if int(r['shortfall_ticks']))} counties",
        f"{sum(int(r['shortfall_ticks']) for r in LR_E.values())} in "
        f"{sum(1 for r in LR_E.values() if int(r['shortfall_ticks']))} counties")
    for c in ("dead", "idle_horse", "zero_orders", "withheld"):
        agg(f"long run: {c} ticks, all counties", sum(int(r[c]) for r in LR_M.values()),
            sum(int(r[c]) for r in LR_E.values()))

# ---- write
with open(os.path.join(OUT, "lines.csv"), "w", newline="") as f:
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["line", "what", "registered", "engine", "tolerance", "verdict"])
    w.writerows(LINES)
with open(os.path.join(OUT, "county-dates.csv"), "w", newline="") as f:
    w = csv.DictWriter(f, fieldnames=list(cd_rows[0].keys()), lineterminator="\n")
    w.writeheader()
    w.writerows(cd_rows)
RUNCOLS = ["county", "year", "run", "tier", "cls_e", "cls_m", "tin_e", "tin_m", "dead_e", "dead_m",
           "baskets_e", "baskets_m", "withheld_e", "withheld_m", "short_e", "short_m", "idle_e",
           "idle_m", "heads_low_e", "heads_low_m", "pk_low_e", "pk_low_m", "markup_low_e",
           "markup_low_m", "k5_e", "k5_m"]


def cell(v):
    if isinstance(v, float):
        return f"{v:.6g}"
    return "" if v is None else str(v)


with open(os.path.join(OUT, "battery-runs.csv"), "w", newline="") as f:
    w = csv.writer(f, lineterminator="\n")
    w.writerow(RUNCOLS)
    for r in rows:
        w.writerow([cell(r.get(c)) for c in RUNCOLS])
if lr_rows:
    cols = list(dict.fromkeys(c for r in lr_rows for c in r))
    with open(os.path.join(OUT, "long-run.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols, lineterminator="\n")
        w.writeheader()
        w.writerows(lr_rows)
with open(os.path.join(OUT, "aggregates.csv"), "w", newline="") as f:
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["what", "registered (the mirror)", "engine"])
    for a, b, c in AGG:
        w.writerow([a, b, c])
fails = [l for l in LINES if l[5] == "FAIL"]
print(f"{len(LINES)} scored lines: {len(LINES) - len(fails)} pass, {len(fails)} fail")
for l in LINES:
    print(f"  [{l[5]}] {l[0]}: {l[1]} | registered {l[2]} | engine {l[3]}")
print("reported, not scored:")
for a, b, c in AGG:
    print(f"  {a}: mirror {b} | engine {c}")
