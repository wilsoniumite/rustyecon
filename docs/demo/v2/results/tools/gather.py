"""D2.4 (label run, 2026-09-30): gather the wave (make_jobs.py) into four tables in DEST:

- runs.tsv: one line per battery run: its county-date, L, the harness's summary columns the scorer
  reads (probe::horses::report::SUMMARY), each market's dead ticks below the floor and troughs from
  stats.tsv, and the job's exit code and wall time;
- modea.tsv: one line per county-date: mode A's verdict and largest gap;
- kicks.tsv: one line per county-date: the base kick set's verdict and slowest mode g;
- long-run.csv and series.csv: the long run's tables as the test wrote them, and long.tsv with
  its exit code.

A job that did not run, or exited other than 0, is kept with its code, so the scorer sees it.
usage: python3 gather.py ROOT DEST
"""
import csv
import os
import re
import shutil
import sys

ROOT, DEST = sys.argv[1], sys.argv[2]
os.makedirs(DEST, exist_ok=True)
MARKETS = ("labour", "land", "fodder", "horse", "traction", "good")
COLS = ["class", "d0", "max_W", "last", "in_tol_from", "dead", "dead_W", "dead_F", "peak_dhat",
        "peak_tick", "low_baskets", "transfer_short_ticks", "heads_low", "heads_high",
        "in_5pct_from", "paper_ticks", "no_order_ticks", "idle_ticks", "pk_low",
        "peak_dhat_ex_horse", "withheld", "switches", "markup_low", "stop", "why"]
EXTRA = ([f"dead_{m}" for m in MARKETS] + [f"notrade_{m}" for m in MARKETS]
         + [f"trough_{m}" for m in MARKETS])


def exit_of(d):
    p = os.path.join(d, "exit")
    if not os.path.exists(p):
        return "missing", ""
    rc, wall = open(p).read().split()
    return rc, wall


def stats_of(d):
    out = {}
    p = os.path.join(d, "stats.tsv")
    if not os.path.exists(p):
        return out
    with open(p) as f:
        next(f)
        for line in f:
            run, stat, where, value = line.rstrip("\n").split("\t")
            out[(run, stat, where)] = value
    return out


ids = sorted(os.listdir(os.path.join(ROOT, "battery")))
with open(os.path.join(DEST, "runs.tsv"), "w") as o:
    o.write("\t".join(["county", "year", "run"] + COLS + EXTRA + ["rc", "wall"]) + "\n")
    for i in ids:
        d = os.path.join(ROOT, "battery", i)
        key, year = i.split("@")
        rc, wall = exit_of(d)
        p = os.path.join(d, "summary.tsv")
        rows = list(csv.DictReader(open(p), delimiter="\t")) if os.path.exists(p) else []
        st = stats_of(d)
        if not rows:
            o.write("\t".join([key, year, "-"] + ["-"] * (len(COLS) + len(EXTRA)) + [rc, wall])
                    + "\n")
        for r in rows:
            n = r["run"]
            ex = ([st.get((n, "dead.below_floor", m), "-") for m in MARKETS]
                  + [st.get((n, "dead.no_trade", m), "-") for m in MARKETS]
                  + [st.get((n, "trough.cleared", m), "-") for m in MARKETS])
            o.write("\t".join([key, year, n] + [r[c] for c in COLS] + ex + [rc, wall]) + "\n")

with open(os.path.join(DEST, "modea.tsv"), "w") as o:
    o.write("county\tyear\tmode_a\tclass\tpeak_dhat\tlast\trc\twall\n")
    for i in sorted(os.listdir(os.path.join(ROOT, "modea"))):
        d = os.path.join(ROOT, "modea", i)
        key, year = i.split("@")
        rc, wall = exit_of(d)
        p = os.path.join(d, "summary.tsv")
        rows = list(csv.DictReader(open(p), delimiter="\t")) if os.path.exists(p) else []
        r = rows[0] if rows else {}
        o.write("\t".join([key, year, r.get("mode_a", "-"), r.get("class", "-"),
                           r.get("peak_dhat", "-"), r.get("last", "-"), rc, wall]) + "\n")

PAT = re.compile(r"kick set (PASS|FAIL)\tg ([0-9.eE+-]+) a tick, ([0-9.eE+-]+) a year\t3\*T6 ([0-9]+)")
with open(os.path.join(DEST, "kicks.tsv"), "w") as o:
    o.write("county\tyear\tkick\tg_tick\tg_year\tthree_t6\tline\trc\twall\n")
    for i in sorted(os.listdir(os.path.join(ROOT, "kicks"))):
        d = os.path.join(ROOT, "kicks", i)
        key, year = i.split("@")
        rc, wall = exit_of(d)
        line = open(os.path.join(d, "out.txt")).read().strip() if os.path.exists(
            os.path.join(d, "out.txt")) else ""
        m = PAT.search(line)
        if m:
            kick, gt, gy, t6 = m.groups()
        else:
            kick = "PASS" if "kick set PASS" in line else ("FAIL" if "kick set FAIL" in line else "-")
            gt = gy = t6 = "-"
        o.write("\t".join([key, year, kick, gt, gy, t6, line.replace("\t", " | "), rc, wall])
                + "\n")

d = os.path.join(ROOT, "long")
rc, wall = exit_of(d)
for f in ("long-run.csv", "series.csv"):
    if os.path.exists(os.path.join(d, f)):
        shutil.copy(os.path.join(d, f), os.path.join(DEST, f))
tail = ""
if os.path.exists(os.path.join(d, "out.txt")):
    tail = [l for l in open(os.path.join(d, "out.txt")) if "counties," in l or l.startswith("test ")]
    tail = " | ".join(x.strip() for x in tail)
with open(os.path.join(DEST, "long.tsv"), "w") as o:
    o.write("rc\twall\tout\n")
    o.write(f"{rc}\t{wall}\t{tail}\n")
print(f"gathered {len(ids)} county-dates into {DEST}")
