"""P2.4 (label run, 2026-09-30): gather the B-D waves' jobs (the trap, the switch, the free step)
into runs.jsonl, one JSON object a job, in the job list's order, for the scorers. As
families-wave/gather.py and p23-wave/gather.py: it reads each job's directory as job.sh left it (or
the archive's gzipped copy: a `<run>.csv` may be `<run>.csv.gz`) and the job list for each job's
arguments. Usage (WSL): python3 gather.py JOBS ROOT OUT, ROOT the directory the jobs wrote to
(/root/scratch/p24-bcd-runs) or its archive.

Each record has the job's `rel` (its directory relative to the jobs' root), `key` (trap, switch or
free), `cmd`, `inst`, `set`, `setting` (a dial setting as --set gives it, or None), `first_year`,
`one_sided`, `tpy`, `ticks` (the scored length L), `names`, `rc` and `secs`, and by command:
- run: the summary line's fields (`markets`' SUMMARY), `stats` ({"stat|where": value}), the scored
  clock's start (`clock_start`, L/4 for a dated shock), the runaway tick relative to it where the
  run left the runaway bound, and from its CSV the last row (`end`, every column); for a history
  run its windows as the commons mirror's run_cycle reads them (per window of 1,500 ticks: the
  D-hat at its last tick, 1 + its last tick out of tolerance (0 if none; None if its last tick is
  out), its dead ticks, its regime at its last tick), and the runaway's window and tick;
- kick: the summary line (`passed`, `kicks`, `gain_tail`, `gain_peak`, `notes`) and each kick;
- elasticity: `tau_max`, `L`, and each market's eps_d, eps_s and tau;
- point: each target's line as a dict.
And one record a set of envelope runs (cmd `envelope`, set `env` or `env-ts011`): for each market
and sign, g(t) the largest |ln(p_kick/p_base)| over the posted prices, the gains g(t)/g(0), their
suffix maximum, and the decay a tick between its first falls below 1e-1 and below 1e-3, as the
wall frame's wstab.kick measures it (p23-wave/gather.py's `envelope`)."""
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

SUMMARY = ["run", "class", "d0", "E1", "E2", "E3", "E4", "kappa", "max_W", "last", "in_tol_from",
           "dead", "dead_W", "dead_F", "band_v", "r_end", "peak_dhat", "peak_tick", "worst_fill",
           "low_baskets", "low_baskets_tick", "no_basket_ticks", "transfer_short",
           "transfer_short_ticks", "depth_eq", "depth_trough_y0", "depth_trough_y1",
           "lowest_market", "mode_a", "why"]
INTS = {"in_tol_from", "dead", "dead_W", "dead_F", "peak_tick", "low_baskets_tick",
        "no_basket_ticks", "transfer_short_ticks"}
P = 1500
ROOT_OUT = "/root/scratch/p24-bcd-runs"


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


def parse_job(line):
    a = shlex.split(line)
    d, cmd, rest = a[1], a[2], a[3:]
    o = {"dir": d, "cmd": cmd, "names": [], "sets": [], "one_sided": None, "first_year": False,
         "tpy": 52}
    it = iter(rest)
    for x in it:
        if x in ("--inst", "--ticks", "--csv", "--every", "--horizon", "--set", "--one-sided",
                 "--tpy"):
            v = next(it)
            if x == "--inst":
                o["inst"] = v
            elif x == "--ticks":
                o["ticks"] = int(v)
            elif x == "--every":
                o["every"] = int(v)
            elif x == "--horizon":
                o["horizon"] = int(v)
            elif x == "--tpy":
                o["tpy"] = int(v)
            elif x == "--set":
                o["sets"].append(v)
            elif x == "--one-sided":
                o["one_sided"] = v
        elif x == "--first-year":
            o["first_year"] = True
        else:
            o["names"].append(x)
    rel = os.path.relpath(d, ROOT_OUT)
    o["rel"] = rel
    parts = rel.split("/")
    o["key"] = parts[0]
    o["setting"] = o["sets"][0] if o["sets"] else None
    if parts[1] == "ctl":
        o["set"] = "ctl-tilt2" if parts[3] == "tilt2" else "ctl-dial"
    else:
        o["set"] = parts[2]
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
    """Per window of 1,500 ticks, as the commons mirror's run_cycle reads it."""
    col = {h: i for i, h in enumerate(head)}
    wins = {}
    for r in rows:
        t = int(r[0])
        k = t // P
        w = wins.setdefault(k, {"w": k, "end_Dh": None, "last_out": None, "last_t": None,
                                "dead": 0, "ticks": 0, "regime": None})
        dh = float(r[col["dhat"]])
        w["end_Dh"] = dh
        w["last_t"] = t - k * P
        w["ticks"] += 1
        if not dh <= 1:
            w["last_out"] = t - k * P
        if r[col["dead"]] in ("true", "1", "True"):
            w["dead"] += 1
        if "regime" in col:
            w["regime"] = r[col["regime"]]
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
    stats = {}
    f = opener(os.path.join(d, "stats.tsv"))
    if f is not None:
        with f:
            for line in f.read().splitlines()[1:]:
                p = line.split("\t")
                if len(p) == 4:
                    stats[f"{p[1]}|{p[2]}"] = num(p[3])
    rec["stats"] = stats
    dated = "@dated" in rec["run"]
    rec["clock_start"] = o["ticks"] // 4 if dated else 0
    m = re.search(r"left the runaway bound at tick (\d+)", rec.get("why") or "")
    rec["runaway_tick"] = int(m.group(1)) - rec["clock_start"] if m else None
    hd, rows = read_csv_rows(os.path.join(d, file_name(rec["run"]) + ".csv"))
    if hd is None:
        rec["missing"] = "csv"
        return rec
    rec["end"] = {h: num(rows[-1][i]) for i, h in enumerate(hd)} if rows else {}
    if o["set"] == "history":
        rec["windows"] = history_windows(hd, rows)
        if m:
            t = int(m.group(1))
            rec["runaway_window"] = [t // P, t % P]
    if o["set"] == "corner":
        rec["corner"] = corner(hd, rows, o["tpy"])
    return rec


def corner(head, rows, tpy, rate=26.0):
    """While a switch pop's gap is below 0 its share moves by exactly e^(k g) a tick, k the switch's
    rate a tick: over every row t whose gap is negative and whose previous share is a normal double,
    the largest |a_t / a_(t-1) / e^(k g_t) - 1| (the CSV's row t: the gap read in tick t, the share
    after it)."""
    col = {h: i for i, h in enumerate(head)}
    k = rate / tpy
    dev, n = 0.0, 0
    for pop in ("workers.trained", "workers.master"):
        a, g = col[f"pool_{pop}"], col[f"swgap_{pop}"]
        for i in range(1, len(rows)):
            a0, a1, gt = float(rows[i - 1][a]), float(rows[i][a]), float(rows[i][g])
            if gt < 0 and a0 >= 2.2250738585072014e-308:
                dev = max(dev, abs(a1 / a0 / math.exp(k * gt) - 1.0))
                n += 1
    return {"max_rel_dev": dev, "ticks": n}


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
    f = opener(os.path.join(d, f"kick-{file_name(p[0])}.tsv"))
    if f is not None:
        with f:
            ks = f.read().splitlines()[1:]
        rec["each"] = [dict(zip(["market", "sign", "size", "gain_tail", "gain_peak", "error"],
                                [num(x) for x in k.split("\t")])) for k in ks if k]
    return rec


def elasticity_digest(o, root):
    d = os.path.join(root, o["rel"])
    f = opener(os.path.join(d, "out.txt"))
    if f is None:
        return {"missing": "out.txt"}
    with f:
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
    if "L" not in rec:
        rec["missing"] = "L"
    return rec


def point_digest(o, root):
    d = os.path.join(root, o["rel"])
    f = opener(os.path.join(d, "out.txt"))
    if f is None:
        return {"missing": "out.txt"}
    with f:
        lines = f.read().splitlines()
    pts = []
    for line in lines:
        p = line.split("\t")
        e = {"inst": p[0], "label": p[1]}
        for field in p[2:]:
            if "[" in field:
                k = field[:field.index("[")].strip()
                e[k] = json.loads(field[field.index("["):].replace("inf", "Infinity")
                                  .replace("NaN", "NaN"))
            elif " " in field:
                k, v = field.rsplit(" ", 1)
                e[k] = num(v)
        pts.append(e)
    return {"points": pts}


def one(a):
    o, root = a
    base = {k: o.get(k) for k in ("rel", "key", "cmd", "inst", "set", "setting", "first_year",
                                  "one_sided", "sets", "tpy", "ticks", "names", "every", "horizon")}
    ex = opener(os.path.join(root, o["rel"], "exit"))
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
    except Exception as e:  # recorded, never hidden: a scorer fails such a line
        base["gather_error"] = repr(e)
    return base


def envelope(recs, root, set_):
    """The kick envelope's decay (the wall frame's wstab.kick) from one set of env runs."""
    env = [r for r in recs if r.get("set") == set_ and r.get("cmd") == "run" and r.get("rc") == 0]
    base = next((r for r in env if "p[" not in r["names"][0]), None)
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
    return {"rel": f"switch/is1/{set_}/@envelope", "key": "switch", "cmd": "envelope",
            "inst": "is1", "set": set_, "rc": 0, "kicks": out}


def main(jobs, root, out):
    js = [parse_job(line) for line in open(jobs) if line.strip()]
    with Pool(46) as p:
        recs = p.map(one, [(o, root) for o in js], chunksize=8)
    for s in ("env", "env-ts011"):
        e = envelope(recs, root, s)
        if e:
            recs.append(e)
    with open(out, "w", newline="\n") as f:
        for r in recs:
            f.write(json.dumps(r, sort_keys=True) + "\n")
    miss = sum(1 for r in recs if r.get("rc") != 0 or "missing" in r or "gather_error" in r)
    print(f"{len(recs)} records; {miss} with a nonzero exit, a missing file or a gather error")


if __name__ == "__main__":
    main(*sys.argv[1:4])
