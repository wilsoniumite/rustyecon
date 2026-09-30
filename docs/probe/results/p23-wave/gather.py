"""P2.3.11 (label run, 2026-09-30): gather the scored wave's jobs into runs.jsonl, one JSON object
a job, sorted by directory, for score.py. It reads each job's directory as job.sh left it (or
the archive's gzipped copy: a `<run>.csv` may be `<run>.csv.gz`), and the job list for each job's
arguments. Usage (WSL): python3 gather.py JOBS ROOT OUT, ROOT the directory the jobs wrote to
(/root/scratch/p23-runs) or its archive (/mnt/d/rustyecon-p23/runs).

Each record has the job's `dir` (relative to ROOT), `cmd`, `inst`, `set`, `tpy`, `ticks` (the scored
length L), `rc` and `secs`, and by command:
- run: the summary line's fields (`markets`' SUMMARY), `stats` ({"stat|where": value}), the
  scored clock's start (`clock_start`, L/4 for a dated shock), the runaway tick relative to it
  where the run left the runaway bound, and from its CSV: the last row (`end`: every column of it)
  and, for the wall's every-tick runs, `thr_return` (each desk's threshold back within tolerance:
  1 + the last scored tick its gap exceeds 1e-3, 0 if never) and `sdecay` (while the wall binds,
  depth >= 0, the largest |s_t/s_(t-1) - e^(-2.6/52)| over normal s_(t-1), relative); for a
  history run, its windows (`windows`), as the mirrors read them (the wall's wb.history, the
  commons' battery_c.run_cycle).
- kick: the summary line (`pass`, `kicks`, `gain_tail`, `gain_peak`, `notes`) and each kick.
- elasticity: `tau_max`, `L`, and each market's eps_d, eps_s and tau.
- point: each target's line as a dict.
And one record `envelope` (set `env`): the kick envelope's decay at the wall's base from the
env runs, as the frame's wstab.kick measures it: for each market and sign, g(t) the largest
|ln(p_kick/p_base)| over the posted prices, the gains g(t)/g(0), their suffix maximum, and the
decay a tick between its first falls below 1e-1 and 1e-3."""
import csv
import gzip
import io
import json
import math
import os
import re
import shlex
import sys
from multiprocessing import Pool

A_TECH = math.exp(-2.6 / 52.0)   # 1 - share(adjust) at 2.6 a year, 52 ticks a year
SUMMARY = ["run", "class", "d0", "E1", "E2", "E3", "E4", "kappa", "max_W", "last", "in_tol_from",
           "dead", "dead_W", "dead_F", "band_v", "r_end", "peak_dhat", "peak_tick", "worst_fill",
           "low_baskets", "low_baskets_tick", "no_basket_ticks", "transfer_short",
           "transfer_short_ticks", "depth_eq", "depth_trough_y0", "depth_trough_y1",
           "lowest_market", "mode_a", "why"]
INTS = {"in_tol_from", "dead", "dead_W", "dead_F", "peak_tick", "low_baskets_tick",
        "no_basket_ticks", "transfer_short_ticks"}
WALL_CATS = ["services", "goods"]
WALL_LABOUR = ["labour", "labour.trained", "labour.master"]


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


def parse_args(cmdline):
    a = shlex.split(cmdline)
    d, cmd, rest = a[1], a[2], a[3:]
    o = {"dir": d, "cmd": cmd, "names": [], "tpy": 52, "flags": []}
    it = iter(rest)
    for x in it:
        if x in ("--inst", "--ticks", "--tpy", "--csv", "--every", "--horizon", "--set",
                 "--one-sided"):
            v = next(it)
            if x == "--inst":
                o["inst"] = v
            elif x == "--ticks":
                o["ticks"] = int(v)
            elif x == "--tpy":
                o["tpy"] = int(v)
            elif x == "--horizon":
                o["horizon"] = int(v)
            elif x in ("--set", "--one-sided"):
                o["flags"] += [x, v]
        elif x.startswith("--"):
            o["flags"].append(x)
        else:
            o["names"].append(x)
    return o


def file_name(run):
    tr = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_",
          "[": "_", "]": "_"}
    return "".join(tr.get(c, c) for c in run)


def read_csv_rows(path):
    f = opener(path)
    if f is None:
        return None, None
    with f:
        r = csv.reader(f)
        head = next(r)
        rows = list(r)
    return head, rows


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
    stats = {}
    f = opener(os.path.join(d, "stats.tsv"))
    if f is not None:
        with f:
            for line in f.read().splitlines()[1:]:
                p = line.split("\t")
                if len(p) == 4:
                    stats[f"{p[1]}|{p[2]}"] = num(p[3])
    rec["stats"] = stats
    name = rec["run"]
    dated = "@dated" in name
    rec["clock_start"] = o["ticks"] // 4 if dated else 0
    m = re.search(r"left the runaway bound at tick (\d+)", rec.get("why") or "")
    rec["runaway_tick"] = int(m.group(1)) - rec["clock_start"] if m else None
    head, rows = read_csv_rows(os.path.join(d, file_name(name) + ".csv"))
    if head is None:
        rec["missing"] = "csv"
        return rec
    col = {h: i for i, h in enumerate(head)}
    last = rows[-1]
    rec["end"] = {h: num(last[i]) for h, i in col.items()}
    cs = rec["clock_start"]
    if o["every"] == 1 and o["inst"] in ("iw1", "ic1") and o["set"] in ("L", "tier3s", "env"):
        thr = {}
        for c in WALL_CATS:
            g = col[f"gap_x.{c}"]
            last_out = None
            for r in rows:
                t = int(r[0])
                if t >= cs and float(r[g]) > 1e-3:
                    last_out = t - cs
            thr[c] = 0 if last_out is None else last_out + 1
        rec["thr_return"] = thr
        dev, n = 0.0, 0
        dep = col["depth"]
        for c in WALL_CATS:
            s = col[f"s_planned_{c}"]
            prev = None
            for r in rows:
                t = int(r[0])
                x = float(r[s])
                if prev is not None and t > cs and float(r[dep]) >= 0.0 and prev >= 2.2250738585072014e-308:
                    dev = max(dev, abs(x / prev / A_TECH - 1.0))
                    n += 1
                prev = x
        rec["sdecay"] = {"max_rel_dev": dev, "ticks": n}
    if o["set"] == "history":
        rec["windows"] = history_windows(o, head, rows)
    return rec


def history_windows(o, head, rows):
    """Per window of a cycle run (P ticks, the last to the run's end at the wall), as the
    mirrors read them: the D-hat at its last tick, 1 + its last tick out of tolerance (0 if none;
    None at the commons if its last tick is out, as run_cycle), its peak, its dead ticks, and at
    the wall its lowest baskets over Y* (every household over the output target y.services, z 1)
    and its lowest depth."""
    col = {h: i for i, h in enumerate(head)}
    P = 1500
    wall = o["inst"] in ("iw1", "ic1")
    wins = {}
    for r in rows:
        t = int(r[0])
        k = min(t // P, 80)
        w = wins.setdefault(k, {"k": k, "start": k * P, "end_Dh": None, "last_out": None,
                                "peak": 0.0, "dead": 0, "baskets": math.inf,
                                "depth_min": math.inf, "ticks": 0, "last_t": None,
                                "regime_ticks": {}})
        dh = float(r[col["dhat"]])
        w["end_Dh"] = dh
        w["last_t"] = t - k * P
        w["ticks"] += 1
        if dh > 1 or not math.isfinite(dh):
            w["last_out"] = t - k * P
        w["peak"] = max(w["peak"], dh) if math.isfinite(dh) else math.inf
        if r[col["dead"]] in ("true", "1", "True"):
            w["dead"] += 1
        if wall:
            b = sum(float(r[col[h]]) for h in head if h.startswith("baskets_"))
            w["baskets"] = min(w["baskets"], b / float(r[col["y.services_star"]]))
            w["depth_min"] = min(w["depth_min"], float(r[col["depth"]]))
        else:
            g = r[col["regime"]]
            w["regime_ticks"][g] = w["regime_ticks"].get(g, 0) + 1
    out = []
    for k in sorted(wins):
        w = wins[k]
        lo = w.pop("last_out")
        if wall:
            w["ttol"] = 0 if lo is None else lo + 1
        else:
            w["ttol"] = None if lo is not None and lo == w["last_t"] else (0 if lo is None else lo + 1)
        w["in_tol_at_end"] = w["end_Dh"] is not None and w["end_Dh"] <= 1
        out.append(w)
    return out


def kick_digest(o, root):
    d = os.path.join(root, o["rel"])
    rec = {}
    f = opener(os.path.join(d, "out.txt"))
    with f:
        lines = f.read().splitlines()
    if len(lines) < 2:
        rec["missing"] = "kick line"
        return rec
    p = lines[1].split("\t")
    rec.update(run=p[0], at=int(p[1]), horizon=int(p[2]), passed=p[3] == "PASS", kicks=int(p[4]),
               gain_tail=float(p[5]), gain_peak=float(p[6]), notes=p[7] if len(p) > 7 else "")
    f = opener(os.path.join(d, f"kick-{file_name(p[0])}.tsv"))
    if f is not None:
        with f:
            ks = f.read().splitlines()[1:]
        rec["each"] = [dict(zip(["market", "sign", "size", "gain_tail", "gain_peak", "error"],
                                [num(x) for x in k.split("\t")])) for k in ks if k]
    return rec


def elasticity_digest(o, root):
    d = os.path.join(root, o["rel"])
    with opener(os.path.join(d, "out.txt")) as f:
        lines = f.read().splitlines()
    rec = {"markets": {}}
    for line in lines:
        m = re.match(r"tau_max ([0-9.]+) ticks; L = (\d+)", line)
        if m:
            rec["tau_max"], rec["L"] = float(m.group(1)), int(m.group(2))
        p = line.split("\t")
        if len(p) == 6 and p[0] != "market" and not line.startswith(" "):
            rec["markets"][p[0]] = dict(k=float(p[1]), eps_d=float(p[2]), eps_s=float(p[3]),
                                        mult=float(p[4]), tau=float(p[5]))
    return rec


def point_digest(o, root):
    d = os.path.join(root, o["rel"])
    with opener(os.path.join(d, "out.txt")) as f:
        lines = f.read().splitlines()
    pts = []
    for line in lines:
        p = line.split("\t")
        e = {"inst": p[0], "label": p[1]}
        for field in p[2:]:
            if "[" in field:
                k = field[:field.index("[")].strip()
                e[k] = json.loads(field[field.index("["):])
            else:
                k, v = field.rsplit(" ", 1)
                e[k] = num(v)
        pts.append(e)
    return {"points": pts}


def one(o_root):
    o, root = o_root
    base = {k: o.get(k) for k in ("rel", "cmd", "inst", "set", "tpy", "ticks", "names", "flags",
                                  "every", "horizon")}
    d = os.path.join(root, o["rel"])
    ex = opener(os.path.join(d, "exit"))
    if ex is None:
        base["rc"] = None
        return base
    with ex:
        rc, secs = ex.read().split()
    base["rc"], base["secs"] = int(rc), float(secs)
    fn = {"run": run_digest, "kick": kick_digest, "elasticity": elasticity_digest,
          "point": point_digest}[o["cmd"]]
    try:
        base.update(fn(o, root))
    except Exception as e:  # recorded, never hidden: score.py fails such a line
        base["gather_error"] = repr(e)
    return base


def envelope(recs, root):
    """The kick envelope's decay at the wall's base (the frame's wstab.kick) from the env runs."""
    env = [r for r in recs if r.get("set") == "env" and r.get("cmd") == "run"]
    base = next((r for r in env if r["names"] == ["hold"]), None)
    if base is None:
        return None

    def prices(r):
        head, rows = read_csv_rows(os.path.join(root, r["rel"], file_name(r["names"][0]) + ".csv"))
        idx = [i for i, h in enumerate(head) if h.startswith("p.")]
        return [h for h in head if h.startswith("p.")], [[float(x[i]) for i in idx] for x in rows]

    names, pb = prices(base)
    out = []
    for r in env:
        if r is base:
            continue
        _, pk = prices(r)
        H = min(len(pk), len(pb))
        gs = []
        g0 = None
        for t in range(H):
            g = max(abs(math.log(pk[t][i] / pb[t][i])) for i in range(len(names)))
            if g0 is None:
                g0 = g
            gs.append(g / g0 if g0 > 0 else math.nan)
        n = len(gs)
        tail = gs[max(0, n - math.ceil(0.1 * H)):]
        suf = [0.0] * n
        mx = 0.0
        for i in range(n - 1, -1, -1):
            mx = max(mx, gs[i])
            suf[i] = mx
        t1 = next((i for i in range(n) if suf[i] < 1e-1), None)
        t2 = next((i for i in range(n) if suf[i] < 1e-3), None)
        rate = None
        if t1 is not None and t2 is not None and t2 > t1:
            rate = math.exp(math.log(suf[t2] / suf[t1]) / (t2 - t1))
        out.append(dict(run=r["names"][0], g0=g0, gain_tail=max(tail), gain_peak=max(gs), t1=t1,
                        t2=t2, rate=rate))
    return {"rel": "wall/iw1/env/@envelope", "cmd": "envelope", "inst": "iw1", "set": "env",
            "kicks": out}


def main(jobs_path, root, out_path):
    todo = []
    for line in open(jobs_path):
        line = line.strip()
        if not line:
            continue
        o = parse_args(line)
        rel = os.path.relpath(o["dir"], os.environ.get("JOBROOT", "/root/scratch/p23-runs"))
        o["rel"] = rel
        parts = rel.split("/")
        o["family"] = parts[0]
        o["set"] = parts[2] if len(parts) > 2 else None
        if o["cmd"] == "run":
            o["every"] = next((int(x) for x in re.findall(r"--every (\d+)", line)), 1)
        todo.append(o)
    todo.sort(key=lambda o: o["rel"])
    with Pool(46) as p:
        recs = p.map(one, [(o, root) for o in todo], chunksize=4)
    env = envelope(recs, root)
    if env:
        recs.append(env)
    with open(out_path, "w", newline="\n") as f:
        for r in recs:
            f.write(json.dumps(r, sort_keys=True, allow_nan=True) + "\n")
    bad = [r["rel"] for r in recs if r.get("rc") not in (0,) and r.get("cmd") != "envelope"]
    print(f"{len(recs)} records; {len(bad)} jobs without exit 0: {bad[:10]}")


if __name__ == "__main__":
    main(*sys.argv[1:4])
