"""P2.4 (label run, 2026-09-30): what the three scorers of the B-D waves share: the line table,
the bands, the formats. As families-wave/score.py and p23-wave/score.py (P2.4.2, P2.3.11)."""
import csv
import json
import math
import os
import statistics
from collections import OrderedDict, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
INF = math.inf
TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}
SETTINGS = ([f"rate.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"buffer.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"adjust.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"tilt.*={v}" for v in ("0.05", "0.1", "0.25", "0.5", "1")])


def fname(run):
    return "".join(TR.get(c, c) for c in run)


class Lines:
    """Every line the scorer reads: step, instance, set, group (the 25% allowance's unit), run,
    what, the registered value, the engine's, the band and the status (pass, fail, charged,
    reported, missing)."""

    def __init__(self):
        self.rows = []

    def add(self, step, inst, set_, group, run, what, reg, eng, band, status):
        self.rows.append(OrderedDict(step=step, inst=inst, set=set_, group=group, run=run,
                                     what=what, registered=fmt(reg), engine=fmt(eng), band=band,
                                     status=status))
        return status

    def charge(self, limit=3):
        """The 25% allowance: at most `limit` runs a group may be charged; beyond it, a charged
        run's lines fail."""
        runs = defaultdict(list)
        for r in self.rows:
            if r["status"] == "charged":
                runs[(r["inst"], r["group"])].append(r)
        for key, rs in runs.items():
            seen = []
            for r in rs:
                if r["run"] not in seen:
                    seen.append(r["run"])
                if len(seen) > limit:
                    r["status"] = "fail"
                    r["band"] += f"; beyond the {limit} runs the allowance takes"

    def tally(self):
        st = defaultdict(int)
        for r in self.rows:
            st[r["status"]] += 1
        return st


def fmt(x):
    if x is None:
        return "-"
    if isinstance(x, bool):
        return str(x)
    if isinstance(x, float):
        if math.isnan(x):
            return "nan"
        if math.isinf(x):
            return "inf" if x > 0 else "-inf"
        return repr(x)
    if isinstance(x, (list, dict)):
        return json.dumps(x, sort_keys=True)
    return str(x)


def st_ticks(m, e):
    """Ticks: within 10%, or charged within 25%; a registered None must be None."""
    if m is None or e is None:
        return "pass" if m is None and e is None else "fail"
    d = abs(e - m)
    if d <= 0.1 * m:
        return "pass"
    if d <= 0.25 * m:
        return "charged"
    return "fail"


def st_rel(m, e, frac):
    if m is None or e is None:
        return "pass" if m is None and e is None else "fail"
    if isinstance(m, float) and not math.isfinite(m):
        return "pass" if m == e else "fail"
    return "pass" if abs(e - m) <= frac * abs(m) else "fail"


def st_count(m, e, frac=0.1, floor=5):
    if m is None or e is None:
        return "pass" if m is None and e is None else "fail"
    return "pass" if abs(e - m) <= max(frac * m, floor) else "fail"


def st_abs(m, e, tol=0.05, above=0.1):
    if m is None or (isinstance(m, float) and not math.isfinite(m)) or m <= above:
        return "reported"
    if e is None or (isinstance(e, float) and math.isnan(e)):
        return "fail"
    return "pass" if abs(e - m) <= tol else "fail"


def st_exact(m, e):
    return "pass" if m == e else "fail"


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


def stat(rec, k, default=None):
    return (rec or {}).get("stats", {}).get(k, default)


def ok_record(e):
    return e is not None and e.get("rc") == 0 and "missing" not in e and "gather_error" not in e


def why_missing(e):
    if e is None:
        return "no record"
    return e.get("missing") or e.get("gather_error") or f"exit {e.get('rc')}"


def kick_bar(pl, L):
    """The mirror's kick prediction (MARKETS-SPEC 7.5 on the mirror): PL^(0.9 L) <= 1e-3."""
    return pl is not None and pl < 1.0 and (0.9 * L) * math.log(pl) <= math.log(1e-3)


def write_rows(path, rows):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    keys = []
    for r in rows:
        for k in r:
            if k not in keys:
                keys.append(k)
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=keys or ["none"])
        w.writeheader()
        for r in rows:
            w.writerow({k: fmt(r.get(k)) for k in keys})


def write_lines(out_dir, lines):
    os.makedirs(out_dir, exist_ok=True)
    head = list(lines.rows[0]) if lines.rows else ["step"]
    with open(os.path.join(out_dir, "lines.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=head)
        w.writeheader()
        w.writerows(lines.rows)
    with open(os.path.join(out_dir, "fails.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=head)
        w.writeheader()
        w.writerows([r for r in lines.rows if r["status"] in ("fail", "missing", "charged")])


def read_recs(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def class_counts(names, cls_of):
    c = defaultdict(int)
    for n in names:
        c[cls_of(n) or "missing"] += 1
    return dict(sorted(c.items()))


def printout(lines, title):
    st = lines.tally()
    print(f"{title}: {len(lines.rows)} lines: " + ", ".join(f"{k} {v}" for k, v in sorted(st.items())))
    by = defaultdict(lambda: defaultdict(int))
    for r in lines.rows:
        by[(r["step"], r["inst"], r["set"])][r["status"]] += 1
    for k in sorted(by):
        print(f"  {k[0]} {k[1]} {k[2]}: " + ", ".join(f"{s} {n}" for s, n in sorted(by[k].items())))
    return st
