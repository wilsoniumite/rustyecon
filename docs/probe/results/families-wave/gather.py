"""P2.4 (label families, 2026-09-30): gather wave A's jobs into runs.jsonl, one JSON object a job,
in the job list's order, for score.py. As p23-wave/gather.py (P2.3.11): it reads each job's
directory as job.sh left it (or the archive's gzipped copy: a `<run>.csv` may be `<run>.csv.gz`)
and the job list for each job's arguments. Usage (WSL): python3 gather.py JOBS ROOT OUT, ROOT the
directory the jobs wrote to (/root/scratch/p24-families-runs) or its archive.

Each record has the job's `rel` (its directory relative to ROOT), `cmd`, `inst`, `set`, `cell`
(an I3 map cell, `r<fr>b<ft>`), `setting` (a dial setting, as --set gives it), `first_year`,
`ticks` (the scored length L), `rc` and `secs`, and by command:
- run: the summary line's fields (`markets`' SUMMARY), the scored clock's start (`clock_start`,
  L/4 for a dated shock), and the runaway tick relative to it where the run left the runaway
  bound; for a history run its windows, read from the CSV's every-tick rows as the mirror reads
  them (fam_i.run_cycle, as the commons' battery_c.run_cycle): per window of 1,500 ticks the D-hat
  at its last tick, 1 + its last tick out of tolerance (0 if none; None if its last tick is out),
  its dead ticks, and the run's runaway window and tick;
- kick: the summary line (`passed`, `kicks`, `gain_tail`, `gain_peak`, `notes`)."""
import csv
import gzip
import io
import json
import os
import re
import shlex
import sys
from multiprocessing import Pool

SUMMARY = ["run", "class", "d0", "E1", "E2", "E3", "E4", "kappa", "max_W", "last", "in_tol_from",
           "dead", "dead_W", "dead_F", "band_v", "r_end", "peak_dhat", "peak_tick", "worst_fill",
           "low_baskets", "low_baskets_tick", "no_basket_ticks", "transfer_short",
           "transfer_short_ticks", "depth_eq", "depth_trough_y0", "depth_trough_y1",
           "lowest_market", "mode_a", "why"]
INTS = {"in_tol_from", "dead", "dead_W", "dead_F", "peak_tick", "low_baskets_tick",
        "no_basket_ticks", "transfer_short_ticks"}
P = 1500


def num(s):
    if s in ("-", ""):
        return None
    try:
        return float(s)
    except ValueError:
        return s


def opener(path):
    if os.path.exists(path):
        return open(path, newline="")
    if os.path.exists(path + ".gz"):
        return io.TextIOWrapper(gzip.open(path + ".gz"), newline="")
    return None


def file_name(run):
    tr = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_",
          "[": "_", "]": "_"}
    return "".join(tr.get(c, c) for c in run)


def parse_job(line, root_out):
    a = shlex.split(line)
    d, cmd, rest = a[1], a[2], a[3:]
    o = {"dir": d, "cmd": cmd, "names": [], "sets": [], "one_sided": None, "first_year": False}
    it = iter(rest)
    for x in it:
        if x in ("--inst", "--ticks", "--csv", "--every", "--horizon", "--set", "--one-sided"):
            v = next(it)
            if x == "--inst":
                o["inst"] = v
            elif x == "--ticks":
                o["ticks"] = int(v)
            elif x == "--every":
                o["every"] = int(v)
            elif x == "--horizon":
                o["horizon"] = int(v)
            elif x == "--set":
                o["sets"].append(v)
            elif x == "--one-sided":
                o["one_sided"] = v
        elif x == "--first-year":
            o["first_year"] = True
        else:
            o["names"].append(x)
    rel = os.path.relpath(d, root_out)
    o["rel"] = rel
    parts = rel.split(os.sep)
    o["cell"] = o["setting"] = None
    if parts[0] == "i":
        if parts[2] == "map":
            o["set"], o["cell"] = "map", parts[3]
            if parts[4] == "kick":
                o["set"] = "map_kick"
            elif parts[4] == "modea":
                o["set"] = "map_modea"
        else:
            o["set"] = parts[2]
    else:
        o["set"] = "dial_kick" if parts[3] == "kick" else "dial"
        o["setting"] = o["sets"][0]
    return o


def read_csv_rows(path):
    f = opener(path)
    if f is None:
        return None, None
    with f:
        r = csv.reader(f)
        head = next(r)
        rows = list(r)
    return head, rows


def history_windows(head, rows):
    """Per window of 1,500 ticks, as fam_i.run_cycle reads it."""
    col = {h: i for i, h in enumerate(head)}
    wins = {}
    for r in rows:
        t = int(r[0])
        k = t // P
        w = wins.setdefault(k, {"w": k, "end_Dh": None, "last_out": None, "last_t": None,
                                "dead": 0, "ticks": 0})
        dh = float(r[col["dhat"]])
        w["end_Dh"] = dh
        w["last_t"] = t - k * P
        w["ticks"] += 1
        if not dh <= 1:
            w["last_out"] = t - k * P
        if r[col["dead"]] in ("true", "1", "True"):
            w["dead"] += 1
    out = []
    for k in sorted(wins):
        w = wins[k]
        lo = w.pop("last_out")
        w["ttol"] = None if (lo is not None and lo == P - 1) else (0 if lo is None else lo + 1)
        w["in_tol_at_end"] = w["end_Dh"] is not None and w["end_Dh"] <= 1 and w["last_t"] == P - 1
        out.append(w)
    return out


def run_digest(o, root):
    d = os.path.join(root, o["rel"])
    rec = {}
    f = opener(os.path.join(d, "summary.tsv"))
    if f is None:
        rec["missing"] = "summary.tsv"
        return rec
    with f:
        lines = f.read().splitlines()
    if len(lines) < 2:
        rec["missing"] = "summary row"
        return rec
    head = lines[0].split("\t")
    assert head == SUMMARY, head
    vals = lines[1].split("\t")
    vals += [""] * (len(SUMMARY) - len(vals))
    for k, v in zip(SUMMARY, vals):
        if k in ("run", "class", "lowest_market", "mode_a", "why"):
            rec[k] = v
        elif k in INTS:
            rec[k] = int(v) if v not in ("-", "") else None
        else:
            rec[k] = num(v)
    dated = "@dated" in rec["run"]
    rec["clock_start"] = o["ticks"] // 4 if dated else 0
    m = re.search(r"left the runaway bound at tick (\d+)", rec.get("why") or "")
    rec["runaway_tick"] = int(m.group(1)) - rec["clock_start"] if m else None
    if o["set"] == "history":
        hd, rows = read_csv_rows(os.path.join(d, file_name(rec["run"]) + ".csv"))
        if hd is None:
            rec["missing"] = "csv"
            return rec
        rec["windows"] = history_windows(hd, rows)
        if m:
            t = int(m.group(1))
            rec["runaway_window"] = [t // P, t % P]
    return rec


def kick_digest(o, root):
    d = os.path.join(root, o["rel"])
    rec = {}
    f = opener(os.path.join(d, "out.txt"))
    if f is None:
        rec["missing"] = "out.txt"
        return rec
    with f:
        lines = f.read().splitlines()
    if len(lines) < 2:
        rec["missing"] = "kick line"
        return rec
    p = lines[1].split("\t")
    rec.update(run=p[0], at=int(p[1]), horizon=int(p[2]), passed=p[3] == "PASS", kicks=int(p[4]),
               gain_tail=float(p[5]), gain_peak=float(p[6]), notes=p[7] if len(p) > 7 else "")
    return rec


def one(a):
    o, root = a
    base = {k: o.get(k) for k in ("rel", "cmd", "inst", "set", "cell", "setting", "first_year",
                                  "one_sided", "sets", "ticks", "names", "every", "horizon")}
    ex = opener(os.path.join(root, o["rel"], "exit"))
    if ex is None:
        base["rc"] = None
        return base
    with ex:
        rc, secs = ex.read().split()
    base["rc"], base["secs"] = int(rc), float(secs)
    fn = {"run": run_digest, "kick": kick_digest}[o["cmd"]]
    try:
        base.update(fn(o, root))
    except Exception as e:  # recorded, never hidden: score.py fails such a line
        base["gather_error"] = repr(e)
    return base


def main(jobs, root, out, root_out="/root/scratch/p24-families-runs"):
    js = [parse_job(line, root_out) for line in open(jobs) if line.strip()]
    with Pool(46) as p:
        recs = p.map(one, [(o, root) for o in js], chunksize=8)
    with open(out, "w") as f:
        for r in recs:
            f.write(json.dumps(r, sort_keys=True) + "\n")
    miss = sum(1 for r in recs if r.get("rc") != 0 or "missing" in r or "gather_error" in r)
    print(f"{len(recs)} records; {miss} with a nonzero exit, a missing file or a gather error")


if __name__ == "__main__":
    main(*sys.argv[1:4])
