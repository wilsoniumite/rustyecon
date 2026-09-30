"""P2.3.11 (label run, 2026-09-30): the scorer of Phase 2 proper's first wave, committed before it
(decision 311). It scores the engine's runs (runs.jsonl, from gather.py) against the two
registrations line by line:
- the wall, IW1 and IC1: docs/probe/wall/SPEC.md §7 (E2-E9), registered at P2.3.1, with A1;
- the open commons, C1, C2 and C1N: docs/probe/commons/SPEC.md §5.5-§6.9 (E2-E9), registered at
  P2.3.5 with Tier 3S (registration §3), A1 and A2.
Every reading of a registered tolerance that the text leaves open is README.md's, beside this file,
fixed before the wave. Usage (WSL): python3 score.py RUNS.jsonl OUTDIR [--self-test]
Writes OUTDIR/lines.tsv (every line: step, instance, set, run, what, registered, engine, band,
status), OUTDIR/wall/*.csv and OUTDIR/commons/*.csv (the tables), and prints the tallies, the
verdicts and the refutation criteria. A line's status is pass, fail, charged (beyond 10% and
within 25%: the allowance of three runs a set at the wall, three an instance and step at the
commons), reported (compared, never scored), or missing (no engine number: scored as a fail
unless the line is reported)."""
import csv
import json
import math
import os
import statistics
import sys
from collections import defaultdict, OrderedDict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
REG_W = os.path.join(REPO, "docs/probe/wall/registered")
REG_C = os.path.join(REPO, "docs/probe/commons/registered")
T3S_C = os.path.join(REPO, "docs/probe/results/commons/tier3s/tier3s.jsonl")
INF = math.inf
TINY = 4.9406564584124654e-323      # 10 subnormal ulps, the wall's stalled share (SPEC §6.2)
A_TECH = math.exp(-2.6 / 52.0)      # 1 - share(adjust), 0.951229...

WALL_CATS = ["services", "goods"]
WALL_MARKETS = ["labour", "land", "mach", "services", "goods", "labour.trained", "labour.master"]
WALL_POPS = {"entrant": "workers", "trained": "workers.trained", "master": "workers.master"}
C_MARKETS = ["labour", "land", "manufactures", "food", "care", "shelter", "mach"]
C_DESKS = ["manufactures", "food", "care", "shelter", "mach"]
C_L = {"c1": 141000, "c2": 142000, "c1n": 141000}
C_NAME = {"c1": "C1", "c2": "C2", "c1n": "C1/chi025"}


# ---------------------------------------------------------------- lines

class Lines:
    def __init__(self):
        self.rows = []

    def add(self, step, inst, set_, run, what, reg, eng, band, status):
        self.rows.append(OrderedDict(step=step, inst=inst, set=set_, run=run, what=what,
                                     registered=fmt(reg), engine=fmt(eng), band=band,
                                     status=status))
        return status

    def charge(self, key_of, limit=3):
        """The 25% allowance: at most `limit` runs a group may be charged; beyond it, a charged
        run's lines fail. A run is charged once, however many of its tick lines are charged."""
        runs = defaultdict(list)
        for r in self.rows:
            if r["status"] == "charged":
                runs[key_of(r)].append(r)
        for key, rs in runs.items():
            seen = []
            for r in rs:
                rid = (r["inst"], r["set"], r["run"])
                if rid not in seen:
                    seen.append(rid)
                if len(seen) > limit:
                    r["status"] = "fail"
                    r["band"] += f"; beyond the {limit} runs the allowance takes"


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
    """Ticks: within 10%, or charged within 25%; a registered None (not in tolerance at the end)
    must be None."""
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
    if isinstance(m, float) and math.isinf(m):
        return "pass" if e == m else "fail"
    if e is None or (isinstance(e, float) and math.isnan(e)):
        return "fail"
    return "pass" if abs(e - m) <= frac * abs(m) else "fail"


def st_count(m, e, frac=0.1, floor=5):
    if m is None or e is None:
        return "pass" if m is None and e is None else "fail"
    return "pass" if abs(e - m) <= max(frac * m, floor) else "fail"


def st_abs(m, e, tol=0.05, above=0.1):
    """Within `tol` absolute where the registered value is above `above`; reported below it."""
    if m is None:
        return "reported"
    if m <= above:
        return "reported"
    if e is None or (isinstance(e, float) and math.isnan(e)):
        return "fail"
    return "pass" if abs(e - m) <= tol else "fail"


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


# ---------------------------------------------------------------- engine views

def stat(rec, k, default=None):
    return rec.get("stats", {}).get(k, default)


def wall_view(rec):
    """The engine's run in the wall mirror's columns (registered/runs_*.tsv)."""
    cs = rec.get("clock_start", 0)
    fb = stat(rec, "wall.first_breach|-")
    v = dict(
        cls=rec.get("class"), ttol=rec.get("in_tol_from"), peak=rec.get("peak_dhat"),
        dead=rec.get("dead"), baskets_trough=rec.get("low_baskets"),
        baskets_none=rec.get("no_basket_ticks"), worst_fill=stat(rec, "wall.worst_buyer_fill|-"),
        transfer_short=rec.get("transfer_short"), depth_min=stat(rec, "wall.depth_min|-"),
        breach=_int(stat(rec, "wall.breach_ticks|-")),
        first_breach=None if fb is None else int(fb) - cs,
        smax={c: stat(rec, f"wall.share_max|{c}") for c in WALL_CATS},
        F_hi={p: stat(rec, f"wall.participation_hi|{a}") for p, a in WALL_POPS.items()},
        sat={p: _int(stat(rec, f"wall.saturated_ticks|{a}")) for p, a in WALL_POPS.items()},
        dead_m={m: _int(stat(rec, f"dead.below_floor|{m}")) for m in WALL_MARKETS},
        runaway=rec.get("runaway_tick"),
        end_gap=None if rec.get("last") is None else rec["last"] * 1e-3,
        end=rec.get("end"), thr_return=rec.get("thr_return"), sdecay=rec.get("sdecay"))
    return v


def _int(x):
    return None if x is None else int(x)


def wall_mirror(row):
    f = lambda k: None if row[k] == "" else float(row[k])
    i = lambda k: None if row[k] == "" else int(row[k])
    return dict(
        cls=row["cls"], ttol=i("ticks_to_tol"), peak=f("peak_Dhat"), dead=i("dead"),
        baskets_trough=f("baskets_trough"), baskets_none=i("baskets_none"),
        worst_fill=f("worst_fill"), transfer_short=f("transfer_short"), depth_min=f("depth_min"),
        breach=i("breach_ticks"), first_breach=i("first_breach"),
        smax={c: f(f"smax_{c}") for c in WALL_CATS},
        F_hi={p: f(f"F_hi_{p}") for p in WALL_POPS}, sat={p: i(f"sat_{p}") for p in WALL_POPS},
        dead_m={m: i(f"dead_{m}") for m in WALL_MARKETS}, tier=row["tier"], tpy=float(row["tpy"]))


def read_tsv(path):
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))
    return OrderedDict((r["run"], r) for r in rows)


def commons_view(rec):
    cs = rec.get("clock_start", 0)
    end = rec.get("end") or {}
    rt = {}
    for k in ("Unused", "Commons", "Crowded", "Enclosed", "Split"):
        rt[k] = _int(stat(rec, f"commons.regime_ticks|{k}"))
    regimes = {"Commons": rt["Commons"], "Crowded": _add(rt["Crowded"], rt["Split"]),
               "Enclosed": rt["Enclosed"]}
    return dict(
        cls=rec.get("class"), ttol=rec.get("in_tol_from"), peak=rec.get("peak_dhat"),
        dead=rec.get("dead"), baskets_trough=rec.get("low_baskets"),
        baskets_none=rec.get("no_basket_ticks"), worst_fill=rec.get("worst_fill"),
        transfer_short=rec.get("transfer_short"), transfer_ticks=rec.get("transfer_short_ticks"),
        trough={m: stat(rec, f"trough.cleared|{m}") for m in C_MARKETS},
        regime_ticks=regimes, unused=rt["Unused"], split=rt["Split"],
        switches=_int(stat(rec, "commons.switches|-")), regime_end=stat(rec, "commons.regime_end|-"),
        regime_star=stat(rec, "commons.regime_star|-"), ro_end=end.get("ro_over_r"),
        ro_low=stat(rec, "commons.ro_low|-"), ro_high=stat(rec, "commons.ro_high|-"),
        tp_high=stat(rec, "commons.tp_max|-"),
        provider_coin_low=stat(rec, "commons.provider_coin_low|-"), runaway=rec.get("runaway_tick"),
        end_gap=None if rec.get("last") is None else rec["last"] * 1e-3, clock_start=cs)


def _add(a, b):
    return None if a is None or b is None else a + b


def commons_mirror(r):
    rr = r.get("runner", r)
    err = rr.get("err") or ""
    runaway = None
    if "runaway at t=" in err:
        runaway = int(err.split("runaway at t=")[1].split(",")[0].split()[0])
    return dict(cls=(r.get("cls_engine_rule") or rr["cls"]) if "runner" in r else rr["cls"],
                ttol=rr.get("ttol"), peak=rr.get("peak"), dead=rr.get("dead"),
                baskets_trough=rr.get("baskets_trough"), baskets_none=rr.get("baskets_none"),
                worst_fill=rr.get("worst_fill"), transfer_short=rr.get("transfer_short"),
                transfer_ticks=rr.get("transfer_ticks"), trough=rr.get("trough"),
                regime_ticks=rr.get("regime_ticks", {}), switches=rr.get("regime_switches"),
                regime_end=rr.get("regime_end"), regime_star=rr.get("regime_star"),
                ro_end=rr.get("ro_end"), ro_star=rr.get("ro_star"), ro_low=rr.get("ro_low"),
                ro_high=rr.get("ro_high"), tp_high=rr.get("tp_high"),
                provider_coin_low=rr.get("provider_coin_low"), runaway=runaway,
                ticks_run=rr.get("ticks_run"), stopped_early=rr.get("stopped_early"),
                L=r.get("L"), tier=r.get("tier"), slack=r.get("slack", False),
                D0=rr.get("D0", r.get("d0_first_year")), err=err)


def c_engine_name(mirror_run):
    """The mirror's run name as the engine names it (registration §3): exit.To is `commons`, a
    desk's coin is coin.desk.<d>."""
    n = mirror_run.replace("exit.To=", "commons=")
    for d in C_DESKS:
        n = n.replace(f"coin.{d}*", f"coin.desk.{d}*")
    return n


# ---------------------------------------------------------------- the wall

def score_wall(recs, L_, out):
    W = defaultdict(dict)   # set -> run -> engine record
    for r in recs:
        if r.get("family") == "wall" or r.get("rel", "").startswith("wall/"):
            W[(r.get("inst"), r.get("set"))][_name(r)] = r
    reg = {s: read_tsv(os.path.join(REG_W, f"runs_{s}.tsv"))
           for s in ("battery", "tier3s", "stocks", "joint2", "joint4", "basin", "tpy12",
                     "tpy365", "hold", "tilt1", "line", "tier3s_line")}
    per_run = []   # the table battery.csv etc.

    def compare(step, set_label, reg_rows, eng_rows, scored=True, only=None, ticks_only=False):
        """Per run: the class exactly, ticks within 10% (25% for three runs a set), the baskets'
        trough and the lowest depth within 5%, dead and breach ticks within 10% or 5, each
        market's dead ticks within 10% or 5; the rest reported."""
        n_conv = 0
        for run, row in reg_rows.items():
            if only and not only(run, row):
                continue
            m = wall_mirror(row)
            e = eng_rows.get(run)
            if e is None or "class" not in e:
                L_.add(step, "iw1" if "line" not in set_label else "ic1", set_label, run, "class",
                       m["cls"], None, "exact", "missing" if scored else "reported")
                continue
            v = wall_view(e)
            inst = e.get("inst")
            rep = lambda s: s if scored else "reported"
            L_.add(step, inst, set_label, run, "class", m["cls"], v["cls"], "exact",
                   rep("pass" if m["cls"] == v["cls"] else "fail"))
            n_conv += v["cls"] == "CONVERGED"
            row_out = OrderedDict(set=set_label, run=run, tier=m["tier"], cls_mirror=m["cls"],
                                  cls_engine=v["cls"], ttol_mirror=m["ttol"], ttol_engine=v["ttol"])
            if m["cls"] == v["cls"] == "CONVERGED":
                L_.add(step, inst, set_label, run, "ticks to tolerance", m["ttol"], v["ttol"],
                       "10% (25% for three runs a set)", rep(st_ticks(m["ttol"], v["ttol"])))
                if not ticks_only:
                    L_.add(step, inst, set_label, run, "lowest baskets over Y*",
                           m["baskets_trough"], v["baskets_trough"], "5%",
                           rep(st_rel(m["baskets_trough"], v["baskets_trough"], 0.05)))
                    L_.add(step, inst, set_label, run, "lowest depth", m["depth_min"],
                           v["depth_min"], "5%", rep(st_rel(m["depth_min"], v["depth_min"], 0.05)))
                    L_.add(step, inst, set_label, run, "dead ticks", m["dead"], v["dead"],
                           "10% or 5 ticks", rep(st_count(m["dead"], v["dead"])))
                    L_.add(step, inst, set_label, run, "breach ticks", m["breach"], v["breach"],
                           "10% or 5 ticks", rep(st_count(m["breach"], v["breach"])))
                    for mk in WALL_MARKETS:
                        if m["dead_m"][mk] or v["dead_m"][mk]:
                            L_.add(step, inst, set_label, run, f"dead ticks: {mk}", m["dead_m"][mk],
                                   v["dead_m"][mk], "10% or 5 ticks",
                                   rep(st_count(m["dead_m"][mk], v["dead_m"][mk])))
                    for k, what in (("peak", "peak D-hat"), ("worst_fill", "worst buyer fill"),
                                    ("transfer_short", "transfer shortfall"),
                                    ("baskets_none", "ticks with no baskets"),
                                    ("first_breach", "first breach tick")):
                        L_.add(step, inst, set_label, run, what, m[k], v[k], "reported", "reported")
            for k in ("baskets_trough", "depth_min", "dead", "breach", "first_breach", "peak",
                      "worst_fill", "transfer_short", "baskets_none"):
                row_out[f"{k}_mirror"] = m[k]
                row_out[f"{k}_engine"] = v[k]
            for mk in WALL_MARKETS:
                row_out[f"dead_{mk}_mirror"] = m["dead_m"][mk]
                row_out[f"dead_{mk}_engine"] = v["dead_m"][mk]
            for c in WALL_CATS:
                row_out[f"smax_{c}_mirror"] = m["smax"][c]
                row_out[f"smax_{c}_engine"] = v["smax"][c]
            for p in WALL_POPS:
                row_out[f"F_hi_{p}_mirror"] = m["F_hi"][p]
                row_out[f"F_hi_{p}_engine"] = v["F_hi"][p]
                row_out[f"sat_{p}_mirror"] = m["sat"][p]
                row_out[f"sat_{p}_engine"] = v["sat"][p]
            row_out["end_gap_engine"] = v["end_gap"]
            per_run.append(row_out)
        return n_conv

    tallies = OrderedDict()
    E = lambda s: W.get(("iw1", s), {})
    # E2: mode A and L
    for tpy, Lr, gap in ((52, 22000, 3.3e-16), (12, 20000, 1.6e-15), (365, 164000, 4.4e-16)):
        r = W.get(("iw1", "modea"), {})
        rec = next((x for x in r.values() if x.get("tpy") == tpy), None)
        ok = rec is not None and rec.get("mode_a") == "PASS" and rec["peak_dhat"] * 1e-3 < 1e-9
        L_.add("E2", "iw1", "modea", f"hold at {tpy} a year", "mode A at L, largest gap below 1e-9",
               gap, None if rec is None else rec["peak_dhat"] * 1e-3, "PASS, below 1e-9",
               "pass" if ok else ("missing" if rec is None else "fail"))
        el = next((x for x in W.get(("iw1", "info"), {}).values()
                   if x.get("cmd") == "elasticity" and x.get("tpy") == tpy), None)
        L_.add("E2", "iw1", "info", f"elasticity at {tpy} a year", "L from the engine's elasticity",
               Lr, None if el is None else el.get("L"), "equal",
               "pass" if el is not None and el.get("L") == Lr else ("missing" if el is None else "fail"))
        if tpy == 52 and el is not None:
            L_.add("E3", "iw1", "info", "elasticity", "goods' tau (ticks)", 106.4,
                   el["markets"].get("goods", {}).get("tau"), "printed digits",
                   "pass" if el["markets"].get("goods", {}).get("tau") == 106.4 else "fail")
    r = W.get(("ic1", "modea"), {})
    rec = next(iter(r.values()), None)
    L_.add("E8", "ic1", "modea", "hold at 52 a year", "mode A at L", 2.2e-16,
           None if rec is None else rec["peak_dhat"] * 1e-3, "reported",
           "reported")
    el = next((x for x in W.get(("ic1", "info"), {}).values() if x.get("cmd") == "elasticity"), None)
    L_.add("E8", "ic1", "info", "elasticity", "L", 25000, None if el is None else el.get("L"),
           "reported", "reported")

    # E3: the battery at L, Tier 3S at L, Tiers 3 and 3S at 10 L
    compare("E3", "battery", reg["battery"], E("L"))
    compare("E3", "tier3s", reg["tier3s"], E("tier3s"))
    t3 = OrderedDict((k, v) for k, v in reg["battery"].items() if v["tier"] == "3")
    compare("E3", "battery at 10L", t3, E("10L"), ticks_only=True)
    compare("E3", "tier3s at 10L", reg["tier3s"], E("10L"), ticks_only=True)
    for label, rows, eng in (("Tier 1", {k: v for k, v in reg["battery"].items() if v["tier"] == "1"}, E("L")),
                             ("Tier 2", {k: v for k, v in reg["battery"].items() if v["tier"] == "2"}, E("L")),
                             ("Tier 3", t3, E("L")), ("Tier 3S", reg["tier3s"], E("tier3s")),
                             ("Tier 3 at 10L", t3, E("10L")), ("Tier 3S at 10L", reg["tier3s"], E("10L"))):
        n = sum(1 for k in rows if eng.get(k, {}).get("class") == "CONVERGED")
        tallies[label] = (n, len(rows))
        L_.add("E3", "iw1", label, "-", "runs CONVERGED", f"{len(rows)}/{len(rows)}",
               f"{n}/{len(rows)}", "all", "pass" if n == len(rows) else "fail")
    # §6.2's table, tier by tier
    tier_rows = []
    for label, rows, eng in (("1", {k: v for k, v in reg["battery"].items() if v["tier"] == "1"}, E("L")),
                             ("2", {k: v for k, v in reg["battery"].items() if v["tier"] == "2"}, E("L")),
                             ("3", t3, E("L")), ("3S", reg["tier3s"], E("tier3s"))):
        tier_rows.append(wall_tier(L_, "E3", label, rows, eng))
    # E4: kick sets, the base's envelope decay, the technique's rate
    kicks = W.get(("iw1", "kick"), {})
    kick_rows = []
    for tgt in ["hold"] + [f"{c}={v}@dated" for c, vs in (("land.mach", ("0.44", "0.36", "0.8", "0.2")),
                                                           ("tail.services", ("0.11", "0.09", "0.2", "0.05")),
                                                           ("res.services.trained", ("0.044", "0.036", "0.08", "0.02")))
                           for v in vs]:
        k = kicks.get(fname(tgt))
        ok = k is not None and k.get("passed") and k.get("gain_tail", INF) <= 1e-3
        L_.add("E4", "iw1", "kick", tgt, "kick set decays: gain_tail at most 1e-3", "PASS (mirror at most 1.5e-5)",
               None if k is None else f"{'PASS' if k.get('passed') else 'FAIL'} {fmt(k.get('gain_tail'))}",
               "PASS and gain_tail <= 1e-3", "pass" if ok else ("missing" if k is None else "fail"))
        kick_rows.append(OrderedDict(target=tgt, passed=None if k is None else k.get("passed"),
                                     gain_tail=None if k is None else k.get("gain_tail"),
                                     gain_peak=None if k is None else k.get("gain_peak"),
                                     kicks=None if k is None else k.get("kicks")))
    env = next((r for r in recs if r.get("cmd") == "envelope"), None)
    rates = [k["rate"] for k in env["kicks"] if k.get("rate")] if env else []
    slow = max(rates) if rates else None
    L_.add("E4", "iw1", "env", "base", "the kick envelope's decay at the base, a year", 0.534,
           None if slow is None else slow ** 52, "within 0.03 a year",
           "missing" if slow is None else ("pass" if abs(slow ** 52 - 0.534) <= 0.03 else "fail"))
    L_.add("E4", "iw1", "env", "base", "the kick envelope's decay at the base, a tick", 0.988002,
           slow, "reported", "reported")
    env_rows = [OrderedDict(k) for k in (env["kicks"] if env else [])]
    # the technique's rate, exactly, from the displaced shares while the wall binds (E4, E6)
    sd = []
    for run in [k for k in reg["battery"] if k.startswith("s[") or k in ("x*/2", "JB(0.5)")]:
        e = E("L").get(run, {})
        s = e.get("sdecay")
        if s:
            sd.append((run, s["max_rel_dev"], s["ticks"]))
    worst = max((x[1] for x in sd), default=None)
    L_.add("E4", "iw1", "L", "s[D]=V, x*/2, JB(0.5)",
           "each displaced share falls by exactly 1 - share(adjust) = 0.951229 a tick while the wall binds",
           "exact", worst, "to rounding: every s_t/s_(t-1) within 1e-12 of e^(-0.05)",
           "missing" if worst is None else ("pass" if worst <= 1e-12 and len(sd) == 8 else "fail"))
    # E5: the cost shocks
    for run, what in (("land.mach=0.8@genesis", "land.mach x 2"), ("land.mach=0.8@dated", "land.mach x 2, dated"),
                      ("tail.services=0.2@genesis", "tail.services x 2"), ("tail.services=0.2@dated", "tail.services x 2, dated"),
                      ("res.services.trained=0.08@genesis", "res.services.trained x 2"),
                      ("res.services.trained=0.08@dated", "res.services.trained x 2, dated")):
        m = wall_mirror(reg["battery"][run])
        e = E("L").get(run)
        v = wall_view(e) if e else None
        L_.add("E5", "iw1", "battery", run, f"{what}: lowest baskets over the new Y*", m["baskets_trough"],
               None if v is None else v["baskets_trough"], "5%",
               "missing" if v is None else st_rel(m["baskets_trough"], v["baskets_trough"], 0.05))
        L_.add("E5", "iw1", "battery", run, f"{what}: dead ticks", m["dead"], None if v is None else v["dead"],
               "10% or 5 ticks", "missing" if v is None else st_count(m["dead"], v["dead"]))
    # E6: the wall's readouts
    readout_rows = wall_readouts(L_, reg, E)
    # E7: the families
    fam_rows = []
    for s, n in (("stocks", 31), ("joint2", 60), ("joint4", 40), ("basin", 516), ("tpy12", 64),
                 ("tpy365", 64), ("hold", 64), ("tilt1", 64)):
        nc = compare("E7", s, reg[s], E(s))
        L_.add("E7", "iw1", s, "-", "runs CONVERGED", f"{n}/{n}", f"{nc}/{len(E(s))}", "all",
               "pass" if nc == n and len(E(s)) == n else "fail")
        fam_rows.append(wall_family(L_, s, reg[s], E(s)))
    # Hold identical to Saturate (Tiers 1-2)
    same, total = 0, 0
    for run, row in reg["hold"].items():
        a, b = E("hold").get(run), E("L").get(run)
        if a is None or b is None:
            continue
        total += 1
        keys = ("class", "in_tol_from", "peak_dhat", "dead", "low_baskets", "no_basket_ticks",
                "worst_fill", "transfer_short")
        if all(a.get(k) == b.get(k) for k in keys) and a.get("stats") == b.get("stats"):
            same += 1
    L_.add("E7", "iw1", "hold", "-", "Hold identical to Saturate in every statistic", "64/64",
           f"{same}/{total}", "all", "pass" if same == total == 64 else "fail")
    hist_rows = wall_history(L_, recs)
    # E8: the line control
    nl = compare("E8", "line", reg["line"], W.get(("ic1", "L"), {}), scored=False)
    n3 = compare("E8", "tier3s_line", reg["tier3s_line"], W.get(("ic1", "tier3s"), {}), scored=False)
    for label, n, tot in (("line", nl, 103), ("tier3s_line", n3, 20)):
        L_.add("E8", "ic1", label, "-", "runs CONVERGED", f"{tot}/{tot}", f"{n}/{tot}", "all",
               "pass" if n == tot else "fail")
    # E9: the edges
    edge_rows = wall_edges(L_, recs)
    # the verdict
    ma = [x for x in L_.rows if x["step"] == "E2" and x["inst"] == "iw1" and "mode A" in x["what"]
          and x["run"].startswith("hold at 52")]
    kicks_ok = all(x["status"] == "pass" for x in L_.rows if x["step"] == "E4" and x["set"] == "kick")
    conv_ok = all(n == t for n, t in tallies.values())
    verdict = "GO" if ma and ma[0]["status"] == "pass" and conv_ok and kicks_ok else (
        "LOCAL" if tallies.get("Tier 1", (0, 1))[0] == tallies.get("Tier 1", (0, 1))[1]
        and tallies.get("Tier 2", (0, 1))[0] == tallies.get("Tier 2", (0, 1))[1] else "NO-GO")
    L_.add("E3", "iw1", "verdict", "-", "IW1's verdict", "GO", verdict,
           "mode A, Tiers 1-3 and 3S CONVERGED at L and 3, 3S at 10 L, every kick set",
           "pass" if verdict == "GO" else "fail")
    refute = wall_refutations(L_, recs, reg, E)
    write_csv(out, "wall/battery.csv", per_run)
    write_csv(out, "wall/tiers.csv", tier_rows)
    write_csv(out, "wall/families.csv", fam_rows)
    write_csv(out, "wall/kicks.csv", kick_rows)
    write_csv(out, "wall/envelope.csv", env_rows)
    write_csv(out, "wall/readouts.csv", readout_rows)
    write_csv(out, "wall/history.csv", hist_rows)
    write_csv(out, "wall/edges.csv", edge_rows)
    write_csv(out, "wall/verdicts.csv", [OrderedDict(inst="iw1", verdict=verdict, prediction="GO",
                                                     **{k: f"{n}/{t}" for k, (n, t) in tallies.items()},
                                                     kicks="13/13 PASS" if kicks_ok else "not all PASS",
                                                     refutations="; ".join(refute) or "none")])
    return verdict, refute


def _name(r):
    """A record's key within its (instance, set): a run job's run name; any other job's (a kick
    set, mode A, the elasticity probe, the point) directory, which is the engine's file name of
    its target."""
    if r.get("cmd") == "run" and r.get("set") != "modea":
        return r.get("run") or (r.get("names") or ["?"])[0]
    return r.get("rel", "?").split("/")[-1]


TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}


def fname(run):
    return "".join(TR.get(c, c) for c in run)


def wall_tier(L_, step, label, rows, eng):
    ms = [wall_mirror(v) | {"run": k} for k, v in rows.items()]
    es = [(k, wall_view(eng[k])) for k in rows if k in eng and "class" in eng[k]]
    out = OrderedDict(tier=label, runs=len(ms), conv_mirror=sum(m["cls"] == "CONVERGED" for m in ms),
                      conv_engine=sum(v["cls"] == "CONVERGED" for _, v in es))

    def both(name, fm, fe, band, stf):
        mv, ev = fm(ms), (fe([v for _, v in es]) if es else None)
        out[f"{name}_mirror"], out[f"{name}_engine"] = mv, ev
        L_.add(step, "iw1", f"Tier {label}", "-", name, mv, ev, band,
               "missing" if ev is None else stf(mv, ev))

    tt = lambda xs: [x["ttol"] for x in xs if x["cls"] == "CONVERGED" and x["ttol"] is not None]
    both("ticks to tolerance, median", lambda xs: med(tt(xs)), lambda xs: med(tt(xs)), "10%",
         lambda m, e: "pass" if st_ticks(m, e) == "pass" else "fail")
    both("ticks to tolerance, slowest", lambda xs: max(tt(xs), default=None),
         lambda xs: max(tt(xs), default=None), "10%",
         lambda m, e: "pass" if st_ticks(m, e) == "pass" else "fail")
    both("lowest baskets over Y*", lambda xs: min(x["baskets_trough"] for x in xs),
         lambda xs: min(x["baskets_trough"] for x in xs), "5%", lambda m, e: st_rel(m, e, 0.05))
    both("lowest depth", lambda xs: min(x["depth_min"] for x in xs),
         lambda xs: min(x["depth_min"] for x in xs), "5%", lambda m, e: st_rel(m, e, 0.05))
    both("dead ticks, worst", lambda xs: max(x["dead"] for x in xs),
         lambda xs: max(x["dead"] for x in xs), "10% or 5 ticks", st_count)
    for mk in WALL_MARKETS:
        both(f"dead ticks, worst: {mk}", lambda xs, mk=mk: max(x["dead_m"][mk] for x in xs),
             lambda xs, mk=mk: max(x["dead_m"][mk] for x in xs), "10% or 5 ticks", st_count)
    both("runs with a breach", lambda xs: sum(1 for x in xs if x["breach"]),
         lambda xs: sum(1 for x in xs if x["breach"]), "exact", lambda m, e: "pass" if m == e else "fail")
    both("most breach ticks", lambda xs: max(x["breach"] for x in xs),
         lambda xs: max(x["breach"] for x in xs), "10% or 5 ticks", st_count)
    both("saturated ticks, most", lambda xs: max(max(x["sat"].values()) for x in xs),
         lambda xs: max(max(x["sat"].values()) for x in xs), "exact", lambda m, e: "pass" if m == e else "fail")
    for name, f in (("peak D-hat, median", lambda xs: med([x["peak"] for x in xs if math.isfinite(x["peak"])])),
                    ("peak D-hat, worst", lambda xs: max(x["peak"] for x in xs)),
                    ("worst buyer fill", lambda xs: min(x["worst_fill"] for x in xs)),
                    ("worst transfer shortfall", lambda xs: max(x["transfer_short"] for x in xs)),
                    ("ticks with no baskets, most", lambda xs: max(x["baskets_none"] for x in xs))):
        both(name, f, f, "reported", lambda m, e: "reported")
    for p in WALL_POPS:
        f = lambda xs, p=p: max(x["F_hi"][p] for x in xs)
        both(f"highest participation: {p}", f, f, "reported", lambda m, e: "reported")
    return out


def wall_family(L_, s, reg_rows, eng):
    ms = [wall_mirror(v) for v in reg_rows.values()]
    es = [wall_view(eng[k]) for k in reg_rows if k in eng and "class" in eng[k]]
    tpy = 12 if s == "tpy12" else (365 if s == "tpy365" else 52)
    tt = lambda xs: [x["ttol"] for x in xs if x["cls"] == "CONVERGED" and x["ttol"] is not None]
    out = OrderedDict(set=s, runs=len(ms), conv_mirror=sum(m["cls"] == "CONVERGED" for m in ms),
                      conv_engine=sum(v["cls"] == "CONVERGED" for v in es))
    for name, f, band, stf in (
            ("ticks median", lambda xs: med(tt(xs)), "10%", lambda m, e: "pass" if st_ticks(m, e) == "pass" else "fail"),
            ("ticks slowest", lambda xs: max(tt(xs), default=None), "10%", lambda m, e: "pass" if st_ticks(m, e) == "pass" else "fail"),
            ("dead worst", lambda xs: max(x["dead"] for x in xs), "10% or 5 ticks", st_count),
            ("lowest baskets", lambda xs: min(x["baskets_trough"] for x in xs), "5%", lambda m, e: st_rel(m, e, 0.05)),
            ("most ticks with no baskets", lambda xs: max(x["baskets_none"] for x in xs), "10% or 5 ticks", st_count),
            ("runs with a breach", lambda xs: sum(1 for x in xs if x["breach"]), "reported", None),
            ("most breach ticks", lambda xs: max(x["breach"] for x in xs), "10% or 5 ticks", st_count),
            ("saturated ticks, most", lambda xs: [max(x["sat"][p] for x in xs) for p in WALL_POPS], "reported", None)):
        mv = f(ms)
        ev = f(es) if es else None
        out[f"{name}_mirror"], out[f"{name}_engine"] = mv, ev
        L_.add("E7", "iw1", s, "-", name, mv, ev, band,
               "reported" if stf is None else ("missing" if ev is None else stf(mv, ev)))
    out["years_median_mirror"] = None if out["ticks median_mirror"] is None else out["ticks median_mirror"] / tpy
    out["years_median_engine"] = None if out["ticks median_engine"] is None else out["ticks median_engine"] / tpy
    return out


def wall_readouts(L_, reg, E):
    rows = []
    bat = E("L")
    # only JB(0.5) breaches: 11 ticks from tick 0, lowest depth -0.444, largest share 0.101
    breached = sorted(k for k, e in bat.items() if (wall_view(e)["breach"] or 0) > 0)
    L_.add("E6", "iw1", "battery", "-", "runs that breach the wall", ["JB(0.5)"], breached, "exact",
           "pass" if breached == ["JB(0.5)"] else ("missing" if not bat else "fail"))
    jb = bat.get("JB(0.5)")
    m = wall_mirror(reg["battery"]["JB(0.5)"])
    if jb:
        v = wall_view(jb)
        L_.add("E6", "iw1", "battery", "JB(0.5)", "breach ticks", m["breach"], v["breach"], "10% or 5 ticks",
               st_count(m["breach"], v["breach"]))
        L_.add("E6", "iw1", "battery", "JB(0.5)", "first breach tick", m["first_breach"], v["first_breach"],
               "exact", "pass" if m["first_breach"] == v["first_breach"] else "fail")
        L_.add("E6", "iw1", "battery", "JB(0.5)", "lowest depth", m["depth_min"], v["depth_min"], "5%",
               st_rel(m["depth_min"], v["depth_min"], 0.05))
        sm = max(m["smax"].values())
        se = max(v["smax"].values())
        L_.add("E6", "iw1", "battery", "JB(0.5)", "largest human share", sm, se, "5%", st_rel(sm, se, 0.05))
    # no other run's depth below 0.128, land.mach 0.8's
    others = [(k, wall_view(e)["depth_min"]) for k, e in bat.items() if k != "JB(0.5)"]
    if others:
        k, lo = min(others, key=lambda x: x[1])
        mk, mlo = min(((k2, wall_mirror(r)["depth_min"]) for k2, r in reg["battery"].items() if k2 != "JB(0.5)"),
                      key=lambda x: x[1])
        L_.add("E6", "iw1", "battery", k, "the lowest depth of every other run", f"{mlo!r} ({mk})",
               f"{lo!r} ({k})", "5%, at land.mach=0.8", "pass" if st_rel(mlo, lo, 0.05) == "pass"
               and k.startswith("land.mach=0.8") else "fail")
    # the thresholds' return: 78, 105 and 124 of s[D]=0.05, 0.2, 0.5; 124 in x*/2; 102 in JB(0.5)
    for run, reg_t, cats in (("s[services]=0.05", 78, ["services"]), ("s[goods]=0.05", 78, ["goods"]),
                             ("s[services]=0.2", 105, ["services"]), ("s[goods]=0.2", 105, ["goods"]),
                             ("s[services]=0.5", 124, ["services"]), ("s[goods]=0.5", 124, ["goods"]),
                             ("x*/2", 124, WALL_CATS), ("JB(0.5)", 102, WALL_CATS)):
        e = bat.get(run, {})
        tr = e.get("thr_return")
        et = None if tr is None else max(tr[c] for c in cats)
        L_.add("E6", "iw1", "battery", run, "thresholds back within tolerance (tick)", reg_t, et,
               "10%", "missing" if et is None else ("pass" if st_ticks(reg_t, et) == "pass" else "fail"))
        rows.append(OrderedDict(run=run, what="thresholds back (tick)", registered=reg_t, engine=et))
    # no pop saturates in the registered battery
    sat = sum(sum(v for v in wall_view(e)["sat"].values() if v) for e in bat.values())
    L_.add("E6", "iw1", "battery", "-", "saturated ticks over the battery", 0, sat if bat else None,
           "exact", "missing" if not bat else ("pass" if sat == 0 else "fail"))
    # every CONVERGED run ends at the wall, every labour market trading
    off, dead_end, never, decayed, n_end = [], [], [], [], 0
    for s in ("L", "tier3s", "10L", "stocks", "joint2", "joint4", "basin", "tpy12", "tpy365",
              "hold", "tilt1", "history", "s30k"):
        for run, e in E(s).items():
            if e.get("class") != "CONVERGED" or not e.get("end"):
                continue
            n_end += 1
            end = e["end"]
            for c in WALL_CATS:
                sv = end.get(f"s_planned_{c}")
                if not (sv == 0.0 or (sv is not None and 0 < sv <= TINY)) or end.get(f"x.{c}") != 1.0:
                    off.append(f"{s}/{run}:{c}={sv!r}")
            for mk in ("labour", "labour.trained", "labour.master"):
                if not (end.get(f"vol.{mk}") or 0) > 0:
                    dead_end.append(f"{s}/{run}:{mk}")
    L_.add("E6", "iw1", "all", "-", "CONVERGED runs ending at the wall (every share 0 or at most 5e-323, x = 1)",
           "all", f"{n_end - len(set(o.split(':')[0] for o in off))}/{n_end}", "all",
           "pass" if not off and n_end else ("missing" if not n_end else "fail"))
    L_.add("E6", "iw1", "all", "-", "CONVERGED runs with every labour market trading at the end", "all",
           f"{n_end - len(set(d.split(':')[0] for d in dead_end))}/{n_end}", "all",
           "pass" if not dead_end and n_end else ("missing" if not n_end else "fail"))
    rows.append(OrderedDict(run="-", what="runs off the wall at the end", registered=0, engine=len(off),
                            detail="; ".join(off[:20])))
    rows.append(OrderedDict(run="-", what="runs with a labour market dead at the end", registered=0,
                            engine=len(dead_end), detail="; ".join(dead_end[:20])))
    # a share never displaced stays 0.0 exactly; a displaced one decays to 5e-323
    n_dec = 0
    for run, e in bat.items():
        v = wall_view(e)
        if v["breach"]:
            continue
        n_dec += bool(e.get("end"))
        displaced = set()
        if run.startswith("s["):
            displaced.add(run[2:run.index("]")])
        if run == "x*/2":
            displaced = set(WALL_CATS)
        for c in WALL_CATS:
            if c not in displaced and v["smax"][c] != 0.0:
                never.append(f"{run}:{c}={v['smax'][c]!r}")
            if c in displaced and e.get("end") and e["end"].get(f"s_planned_{c}") != TINY:
                decayed.append(f"{run}:{c}={(e.get('end') or {}).get(f's_planned_{c}')!r}")
    jbend = (bat.get("JB(0.5)", {}).get("end") or {})
    for c in WALL_CATS:
        if jbend and jbend.get(f"s_planned_{c}") != TINY:
            decayed.append(f"JB(0.5):{c}={jbend.get(f's_planned_{c}')!r}")
    L_.add("E6", "iw1", "battery", "-", "a share never displaced stays 0.0 exactly (runs without a breach)",
           0, len(never) if bat else None, "none departs", "missing" if not bat else ("pass" if not never else "fail"))
    L_.add("E6", "iw1", "battery", "-", "a displaced share ends at 5e-323 (ten subnormal ulps) at L", 0,
           len(decayed) if n_dec else None, "none departs",
           "missing" if not n_dec else ("pass" if not decayed else "fail"))
    rows.append(OrderedDict(run="-", what="never-displaced shares not 0.0", registered=0, engine=len(never),
                            detail="; ".join(never[:20])))
    rows.append(OrderedDict(run="-", what="displaced shares not at 5e-323 at the end", registered=0,
                            engine=len(decayed), detail="; ".join(decayed[:20])))
    s30 = next(iter(E("s30k").values()), None)
    sv = None if s30 is None else (s30.get("end") or {}).get("s_planned_goods")
    L_.add("E6", "iw1", "s30k", "s[goods]=0.5", "the share after 30,000 ticks from 0.5", TINY, sv, "exact",
           "missing" if sv is None else ("pass" if sv == TINY else "fail"))
    return rows


def wall_history(L_, recs):
    reg = json.load(open(os.path.join(REG_W, "history_land.json")))
    rec = next((r for r in recs if r.get("rel", "").startswith("wall/iw1/history/")), None)
    rows = []
    wins = {w["k"]: w for w in (rec or {}).get("windows", [])}
    for m in reg:
        k = m["k"]
        w = wins.get(k)
        row = OrderedDict(k=k, factor=m["factor"])
        for key in ("in_tol_at_end", "ttol", "peak", "dead", "baskets", "depth_min", "end_Dh"):
            row[f"{key}_mirror"] = m[key]
            row[f"{key}_engine"] = None if w is None else w.get(key)
        rows.append(row)
        if w is None:
            L_.add("E7", "iw1", "history", f"window {k}", "window", "run", None, "-", "missing")
            continue
        if 1 <= k <= 79:
            L_.add("E7", "iw1", "history", f"window {k}", "in tolerance at its end", True, w["in_tol_at_end"],
                   "exact", "pass" if w["in_tol_at_end"] else "fail")
        if k == 80:
            L_.add("E7", "iw1", "history", "window 80", "in tolerance at its end", True, w["in_tol_at_end"],
                   "exact", "pass" if w["in_tol_at_end"] else "fail")
        L_.add("E7", "iw1", "history", f"window {k}", "ticks to tolerance", m["ttol"], w["ttol"],
               "10% (25% for three windows)", st_ticks(m["ttol"], w["ttol"]))
        L_.add("E7", "iw1", "history", f"window {k}", "dead ticks", m["dead"], w["dead"], "10% or 5 ticks",
               st_count(m["dead"], w["dead"]))
        L_.add("E7", "iw1", "history", f"window {k}", "lowest baskets over Y*", m["baskets"], w["baskets"],
               "5% (0 exactly where 0)", st_rel(m["baskets"], w["baskets"], 0.05))
        L_.add("E7", "iw1", "history", f"window {k}", "lowest depth", m["depth_min"], w["depth_min"], "5%",
               st_rel(m["depth_min"], w["depth_min"], 0.05))
    half = [w for k, w in wins.items() if 1 <= k <= 79 and reg[k]["factor"] == 0.5]
    nomach = sum(1 for w in half if w["baskets"] == 0.0)
    L_.add("E7", "iw1", "history", "x 0.5 windows", "a tick with no baskets in each x 0.5 window",
           "16/16", f"{nomach}/{len(half)}", "all", "pass" if nomach == len(half) == 16 else
           ("missing" if not half else "fail"))
    return rows


def wall_edges(L_, recs):
    reg = {e["tag"]: e for e in json.load(open(os.path.join(REG_W, "edges.json")))}
    pt = next((r for r in recs if r.get("rel") == "wall/iw1/info/point"), None)
    rows = []
    NE, NT, NM, chiE = 130 / 52, 52 / 52, 26 / 52, 1.0
    g1, lam, own = 0.5, 0.05, 0.3
    least = INF
    for p in (pt or {}).get("points", []):
        tag = p["label"]
        b, rT, rM = 0.4, 0.04, 0.03
        if tag.startswith("land.mach="):
            b = float(tag.split("=")[1])
        if tag.startswith("res.services.trained="):
            rT = float(tag.split("=")[1])
        v, Ps, Y, nD = p["v"], p["P_s"], p["Y"], p["n_D"]
        wT, wM = p["reserved wages"]
        DT, DM = p["hours"][1], p["hours"][2]
        pm = p["type prices"][0]
        C = (wT / Ps) * rT + (wM / Ps) * rM
        om_max = (1 - C) / (nD / Y)
        S_max = NE * min(math.log1p(om_max) / chiE, 1.0)
        e = dict(junction=math.log(v / (g1 * b / (1 - own - g1 * lam))), depth=math.log(v / (g1 * pm)),
                 ceiling=math.log(om_max / (v / Ps)), laborshort=math.log(S_max / nD),
                 saturation=math.log(NE / nD), trained=math.log(NT / DT), master=math.log(NM / DM),
                 eps_trained=math.log(wT / (1.5 * v)), eps_master=math.log(wM / (1.8 * v)))
        m = reg.get(tag)
        if m is None:
            continue
        for k, x in e.items():
            L_.add("E9", "iw1", "edges", tag, f"gap {k}", m[k], x, "1e-12",
                   "pass" if abs(x - m[k]) <= 1e-12 else "fail")
            rows.append(OrderedDict(target=tag, gap=k, registered=m[k], engine=x, diff=x - m[k]))
            if k in ("junction", "depth", "ceiling", "laborshort", "trained", "master") and tag != "base":
                least = min(least, x)
    L_.add("E9", "iw1", "edges", "-", "the least gap over the cost targets", 0.21636355006779342,
           None if least == INF else least, "at least 0.2", "missing" if least == INF else
           ("pass" if least >= 0.2 else "fail"))
    if len(rows) != 13 * 9:
        L_.add("E9", "iw1", "edges", "-", "edges computed", 117, len(rows), "all", "missing")
    return rows


def wall_refutations(L_, recs, reg, E):
    hits = []
    changed = [x for x in L_.rows if x["step"] == "E3" and x["set"] == "battery" and x["what"] == "class"
               and x["status"] != "pass"]
    if changed:
        hits.append(f"class change in the registered battery: {len(changed)}")
    worst, where = 0.0, None
    for r in recs:
        if r.get("rel", "").startswith("wall/iw1/") and r.get("class") == "CONVERGED" and r.get("set") != "env":
            g = (r.get("last") or 0.0) * 1e-3
            if g > worst:
                worst, where = g, r["rel"]
    L_.add("refute", "iw1", "all", where or "-", "the largest end gap of a CONVERGED run (log)", "1e-12",
           worst, "at most 1e-12", "pass" if worst <= 1e-12 else "fail")
    if worst > 1e-12:
        hits.append(f"a CONVERGED run ends {worst:.2e} off its point ({where})")
    for x in L_.rows:
        if x["step"] == "E6" and x["status"] == "fail" and (
                "ending at the wall" in x["what"] or "labour market trading" in x["what"]):
            hits.append(x["what"])
        if x["step"] == "E4" and x["set"] == "kick" and x["status"] == "fail":
            hits.append(f"kick set {x['run']}: {x['engine']}")
    return hits


# ---------------------------------------------------------------- the commons

def score_commons(recs, L_, out):
    C = defaultdict(dict)
    for r in recs:
        if r.get("rel", "").startswith("commons/"):
            C[(r.get("inst"), r.get("set"))][_name(r)] = r
    bat = [json.loads(l) for l in open(os.path.join(REG_C, "battery_v3.jsonl"))]
    fam = [json.loads(l) for l in open(os.path.join(REG_C, "families_v2.jsonl"))]
    tpy = [json.loads(l) for l in open(os.path.join(REG_C, "tpy_v2.jsonl"))]
    neg = [json.loads(l) for l in open(os.path.join(REG_C, "negctl.jsonl"))]
    t3s = [json.loads(l) for l in open(T3S_C)]
    hist = json.load(open(os.path.join(REG_C, "history.json")))
    lin = json.load(open(os.path.join(REG_C, "lin.json")))
    elas = json.load(open(os.path.join(REG_C, "elasticity.json")))
    pred = json.load(open(os.path.join(REG_C, "predictions.json")))
    reg = defaultdict(OrderedDict)   # (inst, set) -> engine run name -> mirror record
    inv = {v: k for k, v in C_NAME.items()}
    for r in bat + [x for x in fam if x["set"] != "tpy12"] + tpy + neg:
        # families_v2's tpy12 runs equal tpy_v2's, which are the tick-length set's
        inst = inv[r["name"]]
        s = {"battery": "L", "tier3x10": "10L", "negctl": "L"}.get(r["set"], r["set"])
        reg[(inst, s)][c_engine_name(r["run"])] = r
    for r in t3s:
        inst = inv[r["name"]]
        s = "tier3s" if r["set"] == "tier3s" else "tier3s10L"
        reg[(inst, s)][r["run"]] = r
    per_run = []
    step_of = {"L": "E3", "10L": "E3", "tier3s": "E3", "tier3s10L": "E3", "stocks": "E4",
               "joint2": "E5", "joint4": "E5", "basin": "E5", "hold": "E7", "tilt1": "E7",
               "tpy12": "E8", "tpy365": "E8", "enclose": "E9"}

    def compare(inst, s):
        step = "E9" if inst == "c1n" else step_of[s]
        eng = C.get((inst, s), {})
        for run, r in reg[(inst, s)].items():
            m = commons_mirror(r)
            e = eng.get(run)
            if e is None or "class" not in e:
                L_.add(step, inst, s, run, "class", m["cls"], None, "exact", "missing")
                continue
            v = commons_view(e)
            L_.add(step, inst, s, run, "class", m["cls"], v["cls"], "exact",
                   "pass" if m["cls"] == v["cls"] else "fail")
            row = OrderedDict(inst=inst, set=s, run=run, tier=m["tier"], slack=m["slack"],
                              cls_mirror=m["cls"], cls_engine=v["cls"], ttol_mirror=m["ttol"],
                              ttol_engine=v["ttol"], runaway_mirror=m["runaway"],
                              runaway_engine=v["runaway"])
            if m["cls"] == v["cls"] == "DIVERGED" and m["runaway"] is not None:
                L_.add(step, inst, s, run, "runaway tick", m["runaway"], v["runaway"], "5%",
                       st_rel(m["runaway"], v["runaway"], 0.05))
            if m["cls"] == v["cls"] == "CONVERGED":
                L_.add(step, inst, s, run, "ticks to tolerance", m["ttol"], v["ttol"],
                       "10% (25% for three runs an instance and step)", st_ticks(m["ttol"], v["ttol"]))
                L_.add(step, inst, s, run, "lowest baskets over Y*", m["baskets_trough"], v["baskets_trough"],
                       "0.05 where the mirror's is above 0.1", st_abs(m["baskets_trough"], v["baskets_trough"]))
                for mk in C_MARKETS:
                    if m["trough"] is None:
                        break
                    L_.add(step, inst, s, run, f"lowest cleared over the oracle's: {mk}", m["trough"][mk],
                           v["trough"][mk], "0.05 where the mirror's is above 0.1",
                           st_abs(m["trough"][mk], v["trough"][mk]))
                L_.add(step, inst, s, run, "dead ticks", m["dead"], v["dead"], "10% or 5 ticks",
                       st_count(m["dead"], v["dead"]))
                # regime ticks: the mirror stopped early; its end regime holds for the rest of L
                Le = e.get("ticks")
                for lab in ("Commons", "Crowded", "Enclosed"):
                    mc = m["regime_ticks"].get(lab, 0)
                    if m["stopped_early"] and lab == m["regime_end"]:
                        mc += Le - m["ticks_run"]
                    ec = v["regime_ticks"][lab]
                    if mc or ec:
                        L_.add(step, inst, s, run, f"ticks in {lab}", mc, ec,
                               "10% or 2 (the mirror's end regime carried to L)", st_count(mc, ec, 0.1, 2))
                L_.add(step, inst, s, run, "regime switches", m["switches"], v["switches"], "10% or 2",
                       st_count(m["switches"], v["switches"], 0.1, 2))
                if m["regime_star"] == "Crowded":
                    L_.add(step, inst, s, run, "r_o/r at the end (Crowded target)", m["ro_end"], v["ro_end"],
                           "1e-9 relative", st_rel(m["ro_end"], v["ro_end"], 1e-9))
                for k, what in (("peak", "peak D-hat"), ("worst_fill", "worst fill (mirror buyers; engine both sides)"),
                                ("transfer_ticks", "ticks the transfer fell short"),
                                ("provider_coin_low", "provider's lowest coin over genesis"),
                                ("tp_high", "largest T_p"), ("baskets_none", "ticks with no baskets")):
                    L_.add(step, inst, s, run, what, m[k], v[k], "reported", "reported")
            for k in ("baskets_trough", "dead", "switches", "regime_end", "regime_star", "ro_end",
                      "peak", "worst_fill", "transfer_ticks", "provider_coin_low", "tp_high",
                      "end_gap"):
                row[f"{k}_mirror"] = m.get(k)
                row[f"{k}_engine"] = v.get(k)
            for lab in ("Commons", "Crowded", "Enclosed"):
                row[f"ticks_{lab}_mirror"] = m["regime_ticks"].get(lab, 0)
                row[f"ticks_{lab}_engine"] = v["regime_ticks"][lab]
            row["ticks_run_mirror"] = m["ticks_run"]
            per_run.append(row)

    for inst in ("c1", "c2"):
        for s in ("L", "10L", "tier3s", "tier3s10L", "stocks", "enclose", "joint2", "joint4", "basin",
                  "hold", "tilt1", "tpy12", "tpy365"):
            compare(inst, s)
    compare("c1n", "L")

    verdicts, tier_rows, fam_rows = [], [], []
    for inst in ("c1", "c2"):
        N = C_NAME[inst]
        # E2: mode A, L, the base kick set
        for tpy_, key in ((52, 52), (12, 12), (365, 365)):
            rec = next((x for x in C.get((inst, "modea"), {}).values() if x.get("tpy") == tpy_), None)
            ma = next(x for x in pred["mode_a"] if x["name"] == N and x["tpy"] == float(tpy_))
            ok = rec is not None and rec.get("mode_a") == "PASS" and rec["peak_dhat"] * 1e-3 < 1e-9
            L_.add("E2", inst, "modea", f"hold at {tpy_} a year", "mode A at L", ma["worst"],
                   None if rec is None else rec["peak_dhat"] * 1e-3, "PASS, below 1e-9",
                   ("pass" if ok else ("missing" if rec is None else "fail")) if tpy_ == 52 else "reported")
            el = next((x for x in C.get((inst, "info"), {}).values()
                       if x.get("cmd") == "elasticity" and x.get("tpy") == tpy_), None)
            Lr = elas[f"{N} {tpy_}"]["L"]
            L_.add("E2", inst, "info", f"elasticity at {tpy_} a year", "L from the engine's elasticity", Lr,
                   None if el is None else el.get("L"), "equal",
                   "pass" if el is not None and el.get("L") == Lr else ("missing" if el is None else "fail"))
        # E3: the counts, the kick sets, the tiers
        kicks = C.get((inst, "kick"), {})
        kicks_ok = True
        targets = ["hold"] + [t for t in COMMONS_TARGETS[inst]]
        for t in targets:
            k = kicks.get(fname(t))
            ok = k is not None and k.get("passed") and k.get("gain_tail", INF) <= 1e-3
            kicks_ok &= bool(ok)
            L_.add("E3" if t != "hold" else "E2", inst, "kick", t, "kick set decays: gain_tail at most 1e-3",
                   "PASS", None if k is None else f"{'PASS' if k.get('passed') else 'FAIL'} {fmt(k.get('gain_tail'))}",
                   "PASS and gain_tail <= 1e-3", "pass" if ok else ("missing" if k is None else "fail"))
        for tpy_ in (12, 365):
            k = kicks.get(f"hold-{tpy_}")
            L_.add("E2", inst, "kick", f"hold at {tpy_} a year", "kick set decays", "reported",
                   None if k is None else f"{'PASS' if k.get('passed') else 'FAIL'} {fmt(k.get('gain_tail'))}",
                   "reported", "reported")
        counts = OrderedDict()
        for label, s, tiers in (("Tier 1", "L", ("1",)), ("Tier 2", "L", ("2",)), ("Tier 3", "L", ("3",)),
                                ("Tier 3 at 10L", "10L", ("3",)), ("Tier 3S", "tier3s", ("3S", None)),
                                ("Tier 3S at 10L", "tier3s10L", ("3S", None))):
            regs = {k: commons_mirror(v) for k, v in reg[(inst, s)].items()
                    if str(commons_mirror(v)["tier"]) in tiers or (s.startswith("tier3s"))}
            eng = C.get((inst, s), {})
            mc = defaultdict(int)
            ec = defaultdict(int)
            for k, m in regs.items():
                mc[m["cls"]] += 1
                ec[eng.get(k, {}).get("class", "missing")] += 1
            counts[label] = (dict(ec), dict(mc))
            L_.add("E3", inst, label, "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())),
                   "equal", "pass" if dict(mc) == dict(ec) else "fail")
            if s in ("L", "10L"):
                tier_rows.append(commons_tier(L_, inst, label, regs, eng))
        # the verdict: mode A PASS; every non-vacuous run of Tiers 1-3 and 3S CONVERGED, and Tier 3
        # and 3S at 10 L; every kick set
        ma_ok = any(x["status"] == "pass" for x in L_.rows if x["step"] == "E2" and x["inst"] == inst
                    and x["run"] == "hold at 52 a year" and x["what"] == "mode A at L")
        conv_ok = all(set(e) <= {"CONVERGED", "VACUOUS"} and e.get("VACUOUS", 0) == m.get("VACUOUS", 0)
                      for e, m in counts.values())
        t12_ok = all(set(counts[t][0]) <= {"CONVERGED", "VACUOUS"} for t in ("Tier 1", "Tier 2"))
        verdict = "GO" if ma_ok and conv_ok and kicks_ok else ("LOCAL" if ma_ok and t12_ok else "NO-GO")
        L_.add("E3", inst, "verdict", "-", f"{N}'s verdict", "GO", verdict,
               "SPEC §5.5 with Tier 3S", "pass" if verdict == "GO" else "fail")
        verdicts.append(OrderedDict(inst=inst, verdict=verdict, prediction="GO",
                                    **{k: json.dumps(v[0], sort_keys=True) for k, v in counts.items()},
                                    kicks=f"{sum(1 for t in targets if kicks.get(fname(t), {}).get('passed'))}/{len(targets)} PASS"))
        # E3 §6.7: every CONVERGED battery run ends in its target's regime, with its shadow rent
        bad_regime, bad_rent, n = [], [], 0
        for run, e in C.get((inst, "L"), {}).items():
            if e.get("class") != "CONVERGED":
                continue
            v = commons_view(e)
            n += 1
            star = {"Split": "Crowded"}.get(v["regime_star"], v["regime_star"])
            if v["regime_end"] != star:
                bad_regime.append(f"{run}: {v['regime_end']} at a {v['regime_star']} target")
            ro = v["ro_end"]
            ok = (star == "Commons" and ro == 0.0) or (star == "Enclosed" and ro == 1.0) or \
                 (star == "Crowded" and ro is not None)
            if not ok:
                bad_rent.append(f"{run}: r_o/r {ro!r} at a {star} target")
        L_.add("E3", inst, "L", "-", "CONVERGED runs ending in their target's regime", f"{n}/{n}",
               f"{n - len(bad_regime)}/{n}", "all", "pass" if not bad_regime and n else "fail")
        L_.add("E3", inst, "L", "-", "shadow rent at the end: 0 exactly at Commons, r exactly at Enclosed",
               f"{n}/{n}", f"{n - len(bad_rent)}/{n}", "all", "pass" if not bad_rent and n else "fail")
        # the families' counts (§6.5) and the trap's seeds
        for s in ("stocks", "joint2", "joint4", "basin", "hold", "tilt1", "tpy12", "tpy365", "enclose"):
            regs = {k: commons_mirror(v) for k, v in reg[(inst, s)].items()}
            eng = C.get((inst, s), {})
            mc, ec = defaultdict(int), defaultdict(int)
            for k, m in regs.items():
                mc[m["cls"]] += 1
                ec[eng.get(k, {}).get("class", "missing")] += 1
            step = step_of[s]
            L_.add(step, inst, s, "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())), "equal",
                   "pass" if dict(mc) == dict(ec) else "fail")
            mdiv = sorted(k for k, m in regs.items() if m["cls"] == "DIVERGED")
            ediv = sorted(k for k in regs if eng.get(k, {}).get("class") == "DIVERGED")
            if mdiv or ediv:
                L_.add(step, inst, s, "-", "the runs DIVERGED (the subsistence trap)", mdiv, ediv, "equal",
                       "pass" if mdiv == ediv else "fail")
            fam_rows.append(commons_family(L_, inst, s, regs, eng, step))
        # E6: history
        hist_rows = commons_history(L_, inst, hist, recs)
        write_csv(out, f"commons/history_{inst}.csv", hist_rows)
    # E9: the negative control
    regs = {k: commons_mirror(v) for k, v in reg[("c1n", "L")].items()}
    eng = C.get(("c1n", "L"), {})
    mc, ec = defaultdict(int), defaultdict(int)
    for k, m in regs.items():
        mc[m["cls"]] += 1
        ec[eng.get(k, {}).get("class", "missing")] += 1
    L_.add("E9", "c1n", "L", "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())), "equal",
           "pass" if dict(mc) == dict(ec) else "fail")
    mdiv = sorted(k for k, m in regs.items() if m["cls"] == "DIVERGED")
    ediv = sorted(k for k in regs if eng.get(k, {}).get("class") == "DIVERGED")
    L_.add("E9", "c1n", "L", "-", "the runs DIVERGED (the subsistence trap)", mdiv, ediv, "equal",
           "pass" if mdiv == ediv else "fail")
    fam_rows.append(commons_family(L_, "c1n", "L", regs, eng, "E9"))
    refute = commons_refutations(L_, recs, C, reg)
    write_csv(out, "commons/runs.csv", per_run)
    write_csv(out, "commons/tiers.csv", tier_rows)
    write_csv(out, "commons/families.csv", fam_rows)
    write_csv(out, "commons/verdicts.csv", verdicts)
    kick_rows = [OrderedDict(inst=r["inst"], target=_name(r), tpy=r.get("tpy"), passed=r.get("passed"),
                             gain_tail=r.get("gain_tail"), gain_peak=r.get("gain_peak"), kicks=r.get("kicks"))
                 for r in recs if r.get("cmd") == "kick" and r.get("rel", "").startswith("commons/")]
    write_csv(out, "commons/kicks.csv", kick_rows)
    return verdicts, refute


COMMONS_TARGETS = {
    c: [f"{k}={v}@dated" for k, vs in (("land.mach", ("0.44", "0.36", "0.8", "0.2")),
                                       ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                                       ("commons", cv)) for v in vs]
    for c, cv in (("c1", ("26.73", "21.87", "48.6", "12.15")),
                  ("c2", ("34.32", "28.08", "62.4", "15.6")))}


def commons_stats(rs, tpy=52.0):
    """predict_summary.tier_stats's numbers, over the runs given as views."""
    ns = [r for r in rs if not r.get("slack")]
    conv = [r for r in ns if r["cls"] == "CONVERGED" and r["ttol"] is not None]
    live = [r for r in rs if r["cls"] not in ("DIVERGED", "missing")]
    tt = [r["ttol"] for r in conv]
    q = lambda xs, f: f(xs) if xs else None
    return OrderedDict(
        ttol_median=q(tt, statistics.median), ttol_max=q(tt, max),
        peak_median=q([r["peak"] for r in live], statistics.median), peak_max=q([r["peak"] for r in live], max),
        dead_median=q([r["dead"] for r in live], statistics.median), dead_max=q([r["dead"] for r in live], max),
        worst_fill=q([r["worst_fill"] for r in live], min),
        baskets_low=q([r["baskets_trough"] for r in live], min),
        baskets_none_max=q([r["baskets_none"] for r in live], max),
        transfer_short_runs=sum(1 for r in live if (r.get("transfer_ticks") or 0) > 0),
        transfer_ticks_max=q([r.get("transfer_ticks") or 0 for r in live], max),
        provider_coin_low=q([r["provider_coin_low"] for r in live if r.get("provider_coin_low") is not None], min),
        switches_max=q([r["switches"] for r in live if r.get("switches") is not None], max),
        trough=OrderedDict((m, q([r["trough"][m] for r in live if r.get("trough")], min)) for m in C_MARKETS))


def commons_tier(L_, inst, label, regs, eng):
    ms = [dict(m, slack=m["slack"]) for m in regs.values()]
    es = []
    for k, m in regs.items():
        if k in eng and "class" in eng[k]:
            es.append(dict(commons_view(eng[k]), slack=m["slack"]))
    a, b = commons_stats(ms), commons_stats(es) if es else None
    out = OrderedDict(inst=inst, tier=label)
    for k in a:
        if k == "trough":
            for mk in C_MARKETS:
                out[f"trough_{mk}_mirror"] = a["trough"][mk]
                out[f"trough_{mk}_engine"] = None if b is None else b["trough"][mk]
                L_.add("E3", inst, label, "-", f"lowest cleared over the oracle's: {mk}", a["trough"][mk],
                       None if b is None else b["trough"][mk], "0.05 where above 0.1",
                       "missing" if b is None else st_abs(a["trough"][mk], b["trough"][mk]))
            continue
        out[f"{k}_mirror"], out[f"{k}_engine"] = a[k], None if b is None else b[k]
        ev = None if b is None else b[k]
        if k in ("ttol_median", "ttol_max"):
            stt = "missing" if b is None else ("pass" if st_ticks(a[k], ev) == "pass" else "fail")
            band = "10%"
        elif k == "dead_max":
            stt, band = ("missing" if b is None else st_count(a[k], ev)), "10% or 5 ticks"
        elif k == "baskets_low":
            stt, band = ("missing" if b is None else st_abs(a[k], ev)), "0.05 where above 0.1"
        else:
            stt, band = "reported", "reported"
        L_.add("E3", inst, label, "-", k, a[k], ev, band, stt)
    return out


def commons_family(L_, inst, s, regs, eng, step):
    ms = list(regs.values())
    es = [dict(commons_view(eng[k]), slack=m["slack"]) for k, m in regs.items() if k in eng and "class" in eng[k]]
    tpy = 12.0 if s == "tpy12" else (365.0 if s == "tpy365" else 52.0)
    a, b = commons_stats(ms, tpy), commons_stats(es, tpy) if es else None
    out = OrderedDict(inst=inst, set=s, runs=len(ms))
    for k in ("ttol_median", "ttol_max", "dead_max", "baskets_low"):
        out[f"{k}_mirror"], out[f"{k}_engine"] = a[k], None if b is None else b[k]
        ev = None if b is None else b[k]
        if k.startswith("ttol"):
            stt = "missing" if b is None else ("pass" if st_ticks(a[k], ev) == "pass" else "fail")
        elif k == "dead_max":
            stt = "missing" if b is None else st_count(a[k], ev)
        else:
            stt = "missing" if b is None else st_abs(a[k], ev)
        L_.add(step, inst, s, "-", k, a[k], ev, "the bands of SPEC §5.6", stt)
    out["years_median_mirror"] = None if a["ttol_median"] is None else a["ttol_median"] / tpy
    out["years_median_engine"] = None if not b or b["ttol_median"] is None else b["ttol_median"] / tpy
    return out


def commons_history(L_, inst, hist, recs):
    N = C_NAME[inst]
    rows = []
    for coef, run in (("land.mach", "cycle(land.mach,1500,80)"), ("exit.To", "cycle(commons,1500,80)")):
        m = next(h for h in hist if h["name"] == N and h["coef"] == coef)
        rec = next((r for r in recs if r.get("rel", "") == f"commons/{inst}/history/" + fname(run)), None)
        wins = {w["k"]: w for w in (rec or {}).get("windows", [])}
        label = f"history {coef if coef != 'exit.To' else 'commons'}"
        # the runaway: the mirror's window and tick
        merr = m["err"]
        eng_run = None if rec is None else rec.get("runaway_tick")
        if merr:
            w5, t5 = int(merr.split("window ")[1].split(",")[0]), int(merr.split("t=")[1])
            reg_tick = w5 * 1500 + t5
            L_.add("E6", inst, label, run, "runaway tick of the run", reg_tick, eng_run, "5%",
                   "missing" if rec is None else st_rel(reg_tick, eng_run, 0.05))
        else:
            L_.add("E6", inst, label, run, "no runaway", None, eng_run, "none",
                   "missing" if rec is None else ("pass" if eng_run is None else "fail"))
        for w in m["windows"]:
            k = w["w"]
            e = wins.get(k)
            row = OrderedDict(inst=inst, coef=coef, w=k, regime=w["regime"], end_Dh_mirror=w["end_Dh"],
                              end_Dh_engine=None if e is None else e["end_Dh"], ttol_mirror=w["ttol"],
                              ttol_engine=None if e is None else e["ttol"], dead_mirror=w["dead"],
                              dead_engine=None if e is None else e["dead"])
            rows.append(row)
            if e is None:
                L_.add("E6", inst, label, f"window {k}", "window", "run", None, "-", "missing")
                continue
            if merr and k == int(merr.split("window ")[1].split(",")[0]):
                continue   # the runaway window: its tick is the line above
            inside = w["end_Dh"] is not None and w["end_Dh"] <= 1
            L_.add("E6", inst, label, f"window {k}", "in tolerance at its end", inside, e["in_tol_at_end"],
                   "exact", "pass" if inside == e["in_tol_at_end"] else "fail")
            L_.add("E6", inst, label, f"window {k}", "ticks to tolerance", w["ttol"], e["ttol"],
                   "10% (25% for three windows an instance)", st_ticks(w["ttol"], e["ttol"]))
            L_.add("E6", inst, label, f"window {k}", "dead ticks", w["dead"], e["dead"], "10% or 5 ticks",
                   st_count(w["dead"], e["dead"]))
    return rows


def commons_refutations(L_, recs, C, reg):
    hits = []
    for inst in ("c1", "c2"):
        e3 = [x for x in L_.rows if x["step"] == "E3" and x["inst"] == inst and x["what"] == "class"]
        bad = [x for x in e3 if x["status"] != "pass"]
        if bad:
            hits.append(f"{inst}: class change in E3 ({len(bad)} runs)")
        div = [x["run"] for x in e3 if x["engine"] == "DIVERGED"]
        if div:
            hits.append(f"{inst}: the trap in E3 runs: {div}")
    worst, where = 0.0, None
    for r in recs:
        if r.get("rel", "").startswith("commons/") and r.get("class") == "CONVERGED":
            g = (r.get("last") or 0.0) * 1e-3
            if g > worst:
                worst, where = g, r["rel"]
    L_.add("refute", "commons", "all", where or "-", "the largest end gap of a CONVERGED run (log)", "1e-12",
           worst, "at most 1e-12", "pass" if worst <= 1e-12 else "fail")
    if worst > 1e-12:
        hits.append(f"a CONVERGED run rests {worst:.2e} off its point ({where})")
    trap = [x for x in L_.rows if x["what"] == "the runs DIVERGED (the subsistence trap)"
            and x["set"] in ("joint2", "joint4", "basin", "tilt1", "L") and x["registered"] not in ("[]", "-")]
    fell = [x for x in trap if x["engine"] not in ("[]", "-")]
    hist5 = [x for x in L_.rows if x["what"] == "runaway tick of the run" and x["engine"] not in ("-",)]
    L_.add("refute", "commons", "families", "-", "some predicted trap run falls into it (E5, E6, E9)",
           "yes", "yes" if fell or hist5 else "no", "at least one", "pass" if fell or hist5 else "fail")
    if not (fell or hist5):
        hits.append("none of the trap's predicted runs falls into it")
    return hits


# ---------------------------------------------------------------- output

def write_csv(out, rel, rows):
    path = os.path.join(out, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    keys = []
    for r in rows:
        for k in r:
            if k not in keys:
                keys.append(k)
    with open(path, "w", newline="\n") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(keys)
        for r in rows:
            w.writerow([fmt(r.get(k)) for k in keys])


def main(runs_path, out, self_test=False):
    recs = [json.loads(l) for l in open(runs_path)]
    L_ = Lines()
    wv, wref = score_wall(recs, L_, out)
    cv, cref = score_commons(recs, L_, out)
    # the 25% allowances: three runs a set at the wall; three an instance and step at the commons
    L_.charge(lambda r: ("wall", r["inst"], r["set"]) if r["inst"] in ("iw1", "ic1")
              else ("commons", r["inst"], r["step"]))
    for r in L_.rows:   # a missing engine number is a fail unless the line is reported
        if r["status"] == "missing" and not self_test:
            r["band"] += "; missing"
    write_csv(out, "lines.tsv".replace(".tsv", ".csv"), L_.rows)
    tally = defaultdict(lambda: defaultdict(int))
    for r in L_.rows:
        tally[(r["inst"], r["step"])][r["status"]] += 1
    print("lines by instance and step:")
    for k in sorted(tally):
        print(f"  {k[0]:8s} {k[1]:7s} " + ", ".join(f"{s} {n}" for s, n in sorted(tally[k].items())))
    tot = defaultdict(int)
    for r in L_.rows:
        tot[r["status"]] += 1
    print("all lines:", dict(tot))
    print("the wall: IW1", wv, "; refutations:", wref or "none")
    for v in cv:
        print(f"the commons: {v['inst']} {v['verdict']}")
    print("the commons' refutations:", cref or "none")
    bad = [r for r in L_.rows if r["status"] in ("fail",) or (r["status"] == "missing" and not self_test)]
    print(f"{len(bad)} lines fail or are missing")
    for r in bad[:80]:
        print("  ", "\t".join(str(r[k]) for k in ("step", "inst", "set", "run", "what", "registered", "engine", "band")))
    return 0


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    sys.exit(main(args[0], args[1], "--self-test" in sys.argv))
