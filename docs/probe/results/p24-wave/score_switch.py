"""P2.4 (label run, 2026-09-30): the scorer of the type switch's wave, committed before the wave's
first job (decision 311). It scores runs.jsonl (gather.py) against the registered predictions in
docs/probe/switch/registered/ (SPEC.md §7; registration.md, whose §3 readings it takes), line by
line, and writes lines.csv (every line), fails.csv, the tables and a printout with the tallies,
the verdict and the refutation criteria. The bands are the wall frame's (SPEC §7):

- every run's class exactly;
- ticks to tolerance within 10%, three runs a set within 25% (a set: an instance's set, or one
  dial setting's Tier 3); at 10 L the class and the ticks only ("the same classes and ticks");
- the lowest baskets over Y* and the lowest depth within 5% relative (a registered 0 must be 0);
- dead ticks, breach ticks and each market's dead ticks within 10% or 5, whichever is larger;
- the switch's readouts per pop: its ticks with a above 1e-9 within 10% or 5 ticks (the mirror's
  count carried to L where it stopped early with a above 1e-9, as the regime ticks are carried;
  README), its largest a within 10% (a registered 0 must be 0.0 exactly); gap sign changes
  reported;
- every CONVERGED run's last D-hat at most 1e-9 (a gap of 1e-12 in log);
- E5: at a pooled target each switch pop ends at its a* within 1e-10 relative (switch_mp's a, the
  50-digit solve, at the run's tick length); at every other target its share ends 0.0 if it never
  pooled, else at most 1e-300, as registered; and, beside it, on its wall side (a below 1e-9 and
  its gap at the end below 0), which the refutation reads (README: the registration's own full-L
  run ends land.mach=0.8's trained at 1.47e-252);
- E1: the 68 never-pooled runs of IS1's battery and Tier 3S are the P2.3 wave's engine IW1 runs
  (class, ticks to tolerance exactly, peak D-hat within 1e-9 relative), and each pop's share never
  leaves 0.0 in them;
- E2: mode A at L PASS with its gap below 1e-9, and L from the engine's elasticity probe, at 52,
  12 and 365 a year (IS1 scored, IS2 reported);
- E4: every kick set at the base and the 12 cost targets passes (gain_tail at most 1e-3) at IS1
  and IS2; the kick envelope's decay at IS1's base within 0.03 a year of 0.534; at
  tail.services 0.11 reported (README); at a walled target a switch pop's share moves by exactly
  e^(k g) a tick while its gap is below 0 (to rounding, 1e-12 relative, on the every-tick runs);
- E6: only JB(0.5) breaches the wall in the battery;
- E7: the families' counts; Hold identical to Saturate run for run, in every statistic;
- E8: IS2, reported;
- E9: the engine's `point` gives switch_mp's pooled flags exactly and its a* and switch distances
  within 1e-12 at every point of IS1 and IS2, and every walled target's IS1 point is IW1's
  (P2.3's `point`) double for double.

Usage (WSL): python3 score_switch.py RUNS.jsonl OUT_DIR [--self-test] [--p23 P23_RUNS.jsonl.gz]"""
import csv
import gzip
import json
import math
import os
import sys
from collections import OrderedDict, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import (REPO, Lines, fname, med, ok_record, printout, read_recs, st_count, st_rel,  # noqa: E402
                    st_ticks, stat, why_missing, write_lines, write_rows)

REG = os.path.join(REPO, "docs", "probe", "switch", "registered")
P23 = "/mnt/d/rustyecon-p23/runs/runs.jsonl.gz"
POPS = {"trained": "workers.trained", "master": "workers.master"}
WALL_MARKETS = ["labour", "land", "mach", "services", "goods", "labour.trained", "labour.master"]
L52 = {"is1": 22000, "is2": 24000}
LT = {"is1": {52: 22000, 12: 20000, 365: 164000}, "is2": {52: 24000, 12: 20000, 365: 183000}}
K = 26.0  # rate.switch, a year
TARGETS = [f"{c}={v}@dated" for c, vs in (("land.mach", ("0.44", "0.36", "0.8", "0.2")),
                                          ("tail.services", ("0.11", "0.09", "0.2", "0.05")),
                                          ("res.services.trained", ("0.044", "0.036", "0.08", "0.02")))
           for v in vs]
# registered TSV set -> (engine set, step)
SETS = {"battery": ("L", "E3"), "tier3s": ("tier3s", "E3"), "tier3L10": ("10L", "E3"),
        "stocks": ("stocks", "E7"), "joint2": ("joint2", "E7"), "joint4": ("joint4", "E7"),
        "basin": ("basin", "E7"), "tpy12": ("tpy12", "E7"), "tpy365": ("tpy365", "E7"),
        "hold": ("hold", "E7"), "nbhd": ("nbhd", "E7"), "switch": ("swrate", "E7")}
END_LINES = ("end D-hat", "E5: a pooled pop ends at a*, within 1e-10 relative",
             "E5: a walled pop ends 0.0 if it never pooled, else at most 1e-300",
             "E5: on its wall side at the end (a below 1e-9, its gap below 0)",
             "E4: the share moves by exactly e^(k g) a tick while the gap is below 0")


def setting_of(dial):
    if not dial or dial == "hold*1":
        return None
    if dial.startswith("tilt="):
        return f"tilt.*={dial[5:]}"
    k, f = dial.split("*")
    return f"rate.switch.*={f}" if k == "switch" else f"{k}.*={f}"


def num(x):
    if x in ("", None):
        return None
    try:
        return float(x)
    except ValueError:
        return x


def read_tsv(path):
    with open(path, newline="") as f:
        return [{k: (v if k in ("run", "tier", "dial", "cls", "err") else num(v)) for k, v in r.items()}
                for r in csv.DictReader(f, delimiter="\t")]


def registered():
    """(inst, engine set, setting) -> OrderedDict(run -> (step, mirror row))."""
    R = defaultdict(OrderedDict)
    for fn in sorted(os.listdir(REG)):
        if not (fn.startswith("runs_") and fn.endswith(".tsv")):
            continue
        inst, s = fn[5:-4].split("_", 1)
        es, step = SETS[s]
        if inst == "is2":
            step = "E8"
        for r in read_tsv(os.path.join(REG, fn)):
            R[(inst, es, setting_of(r["dial"]))][r["run"]] = (step, r)
    return R


def mp():
    return {i: json.load(open(os.path.join(REG, f"switch_mp{'' if i == 'is1' else '_is2'}.json")))
            for i in ("is1", "is2")}


def target_of(run):
    """The run's target label as switch_mp keys it: a cost shock's `c=V`, else `base`."""
    for part in run.split("+"):
        if "=" in part and "@" in part and not part.startswith(("p[", "s[", "sw[")):
            return part.split("@")[0]
    return "base"


def engine_index(recs):
    E = defaultdict(dict)
    for e in recs:
        if e.get("key") != "switch":
            continue
        name = (e.get("names") or ["-"])[0]
        if e.get("cmd") == "kick":
            E[(e["inst"], "kick", None)][os.path.basename(e["rel"])] = e
        elif e.get("cmd") in ("elasticity", "point"):
            E[(e["inst"], "info", None)][os.path.basename(e["rel"])] = e
        elif e.get("cmd") == "envelope":
            E[(e["inst"], "envelope", None)][e["set"]] = e
        elif e.get("set") == "modea":
            E[(e["inst"], "modea", None)][e.get("tpy")] = e
        else:
            setting = e.get("setting") if e.get("set") in ("nbhd", "swrate") else None
            E[(e["inst"], e["set"], setting)][name] = e
    return E


def carried_live(m, t, L):
    """The mirror's ticks with a above 1e-9, carried to L where it stopped early above 1e-9."""
    live = m.get(f"live_{t}")
    end = m.get(f"end_{t}") or 0.0
    if live is None:
        return None
    if str(m.get("stopped_early")) == "True" and end > 1e-9 and L is not None:
        live += int(L - m["ticks_run"])
    return int(live)


def run_lines(out, step, inst, set_, group, run, m, e, mpd, self_test, ticks_only=False, scored=True):
    rep = (lambda s: s) if scored else (lambda s: "reported")
    if not ok_record(e):
        out.add(step, inst, set_, group, run, "class", m["cls"], why_missing(e), "exactly",
                "missing" if scored else "reported")
        return
    ec, mc = e.get("class"), m["cls"]
    out.add(step, inst, set_, group, run, "class", mc, ec, "exactly", rep("pass" if ec == mc else "fail"))
    if not (mc == ec == "CONVERGED"):
        return
    out.add(step, inst, set_, group, run, "ticks to tolerance", m.get("ticks_to_tol"), e.get("in_tol_from"),
            "10%, three runs a set within 25%", rep(st_ticks(m.get("ticks_to_tol"), e.get("in_tol_from"))))
    if ticks_only:
        return
    out.add(step, inst, set_, group, run, "lowest baskets over Y*", m.get("baskets_trough"),
            e.get("low_baskets"), "5%", rep(st_rel(m.get("baskets_trough"), e.get("low_baskets"), 0.05)))
    out.add(step, inst, set_, group, run, "lowest depth", m.get("depth_min"), stat(e, "wall.depth_min|-"),
            "5%", rep(st_rel(m.get("depth_min"), stat(e, "wall.depth_min|-"), 0.05)))
    out.add(step, inst, set_, group, run, "dead ticks", m.get("dead"), e.get("dead"), "10% or 5",
            rep(st_count(m.get("dead"), e.get("dead"))))
    bt = stat(e, "wall.breach_ticks|-")
    out.add(step, inst, set_, group, run, "breach ticks", m.get("breach_ticks"), bt, "10% or 5",
            rep(st_count(m.get("breach_ticks"), bt)))
    for mk in WALL_MARKETS:
        mv, ev = m.get(f"dead_{mk}"), stat(e, f"dead.below_floor|{mk}")
        if mv or ev:
            out.add(step, inst, set_, group, run, f"dead ticks: {mk}", mv, ev, "10% or 5",
                    rep(st_count(mv, ev)))
    L = e.get("ticks")
    for t, pop in POPS.items():
        ml = carried_live(m, t, L)
        el = stat(e, f"switch.live_ticks|{pop}")
        out.add(step, inst, set_, group, run, f"ticks with a above 1e-9: {t}", ml, el,
                "10% or 5 (the mirror's carried to L above 1e-9)", rep(st_count(ml, el)))
        mm, em = m.get(f"max_{t}"), stat(e, f"switch.max|{pop}")
        st = ("pass" if em == 0.0 else "fail") if mm == 0.0 else st_rel(mm, em, 0.1)
        out.add(step, inst, set_, group, run, f"largest a: {t}", mm, em, "10% (0 exactly where 0)", rep(st))
        out.add(step, inst, set_, group, run, f"gap sign changes: {t}", m.get(f"band_{t}"),
                stat(e, f"switch.band|{pop}"), "reported", "reported")
    last = e.get("last")
    out.add(step, inst, set_, group, run, "end D-hat", 1e-9, last, "at most 1e-9 (a gap of 1e-12 in log)",
            "missing" if self_test else rep("pass" if last is not None and last <= 1e-9 else "fail"))
    # E5: the end of each switch pop
    tpy = int(float(m.get("tpy") or 52))
    key = f"{target_of(run)}@{tpy}"
    pt = mpd.get(key)
    if pt is None:
        return
    for q, (t, pop) in enumerate(POPS.items()):
        end = stat(e, f"switch.end|{pop}")
        gap = stat(e, f"switch.gap_end|{pop}")
        if pt["pooled"][q]:
            a = pt["a"][q]
            ok = end is not None and a > 0 and abs(end - a) <= 1e-10 * a
            out.add("E5", inst, set_, group, run, END_LINES[1], a, end, f"{t}: within 1e-10 relative",
                    "missing" if self_test else rep("pass" if ok else "fail"))
        else:
            never = (m.get(f"live_{t}") or 0) == 0 and (m.get(f"max_{t}") or 0.0) == 0.0
            ok = end == 0.0 if never else (end is not None and end <= 1e-300)
            out.add("E5", inst, set_, group, run, END_LINES[2], "0.0" if never else "at most 1e-300", end,
                    f"{t}: as registered", "missing" if self_test else rep("pass" if ok else "fail"))
            side = end is not None and end < 1e-9 and (end == 0.0 or (gap is not None and gap < 0))
            out.add("E5", inst, set_, group, run, END_LINES[3], "walled", f"a {end!r}, gap {gap!r}",
                    f"{t}: the refutation's reading", "missing" if self_test else rep("pass" if side else "fail"))
    for k, ek, what in (("peak_Dhat", e.get("peak_dhat"), "peak D-hat"),
                        ("worst_fill", stat(e, "wall.worst_buyer_fill|-"), "worst buyer fill"),
                        ("transfer_short", e.get("transfer_short"), "transfer shortfall"),
                        ("baskets_none", e.get("no_basket_ticks"), "ticks with no baskets")):
        out.add(step, inst, set_, group, run, what, m.get(k), ek, "reported", "reported")


def printed(x):
    """A number as the harness's summary prints it (%.6e): E1's peak D-hat is read at these digits,
    the finest the summary gives; 1e-9 relative is below them (README)."""
    return None if x is None else f"{x:.6e}"


def load_p23(path):
    """The P2.3 wave's engine IW1 runs (battery and Tier 3S at L) and its IW1 point."""
    runs, point = {}, None
    op = gzip.open if path.endswith(".gz") else open
    with op(path, "rt") as f:
        for line in f:
            r = json.loads(line)
            rel = r.get("rel", "")
            if rel.startswith("wall/iw1/L/") or rel.startswith("wall/iw1/tier3s/"):
                runs[r.get("run")] = r
            elif rel == "wall/iw1/info/point":
                point = r
    return runs, point


def score(recs, self_test, p23_path):
    out = Lines()
    R = registered()
    E = engine_index(recs)
    MP = mp()
    per_run = []
    for (inst, es, setting), rows in R.items():
        eng = E.get((inst, es, setting), {})
        for run, (step, m) in rows.items():
            e = eng.get(run)
            group = f"{es} {setting}" if setting else es
            run_lines(out, step, inst, es, group, run, m, e, MP[inst], self_test,
                      ticks_only=(es == "10L"), scored=(inst == "is1"))
            per_run.append(OrderedDict(
                step=step, inst=inst, set=es, setting=setting, run=run, tier=m.get("tier"),
                cls_mirror=m["cls"], cls_engine=(e or {}).get("class"), ttol_mirror=m.get("ticks_to_tol"),
                ttol_engine=(e or {}).get("in_tol_from"), baskets_mirror=m.get("baskets_trough"),
                baskets_engine=(e or {}).get("low_baskets"), depth_mirror=m.get("depth_min"),
                depth_engine=stat(e, "wall.depth_min|-"), dead_mirror=m.get("dead"),
                dead_engine=(e or {}).get("dead"), breach_mirror=m.get("breach_ticks"),
                breach_engine=stat(e, "wall.breach_ticks|-"),
                live_trained_mirror=carried_live(m, "trained", (e or {}).get("ticks")),
                live_trained_engine=stat(e, "switch.live_ticks|workers.trained"),
                max_trained_mirror=m.get("max_trained"), max_trained_engine=stat(e, "switch.max|workers.trained"),
                end_trained_engine=stat(e, "switch.end|workers.trained"),
                live_master_mirror=carried_live(m, "master", (e or {}).get("ticks")),
                live_master_engine=stat(e, "switch.live_ticks|workers.master"),
                max_master_mirror=m.get("max_master"), max_master_engine=stat(e, "switch.max|workers.master"),
                end_master_engine=stat(e, "switch.end|workers.master"),
                end_dhat_engine=(e or {}).get("last")))
    # E1: the never-pooled runs against the P2.3 wave's IW1 runs
    p23, p23_point = load_p23(p23_path)
    never = read_tsv(os.path.join(REG, "never_pooled_is1.tsv"))
    n_same = 0
    for r in never:
        run = r["run"]
        es = "tier3s" if r["tier"] == "3S" else "L"
        e = E.get(("is1", es, None), {}).get(run)
        w = p23.get(run)
        if not ok_record(e) or w is None:
            out.add("E1", "is1", es, "E1", run, "the engine's IW1 run of the P2.3 wave", "IW1",
                    why_missing(e) if w is not None else "no P2.3 record", "class, ticks, peak D-hat", "missing")
            continue
        same = (e.get("class") == w.get("class") and e.get("in_tol_from") == w.get("in_tol_from")
                and printed(w.get("peak_dhat")) == printed(e.get("peak_dhat")))
        n_same += same
        out.add("E1", "is1", es, "E1", run, "the engine's IW1 run of the P2.3 wave: class, ticks, peak D-hat",
                [w.get("class"), w.get("in_tol_from"), w.get("peak_dhat")],
                [e.get("class"), e.get("in_tol_from"), e.get("peak_dhat")],
                "class and ticks exactly, peak D-hat as the harness prints it (7 digits; README)",
                "pass" if same else "fail")
        zero = all(stat(e, f"switch.max|{p}") == 0.0 for p in POPS.values())
        out.add("E1", "is1", es, "E1", run, "each pop's share never leaves 0.0", 0.0,
                [stat(e, f"switch.max|{p}") for p in POPS.values()], "exactly", "pass" if zero else "fail")
    out.add("E1", "is1", "never-pooled", "E1", "-", "never-pooled runs that are IW1's", f"{len(never)}/{len(never)}",
            f"{n_same}/{len(never)}", "all", "pass" if n_same == len(never) else "fail")
    # E5: the runs where a type pools
    pooled_reg = sorted(run for (inst, es, s), rows in R.items() if inst == "is1" and es in ("L", "tier3s")
                        for run, (_, m) in rows.items() if (m.get("max_trained") or 0) > 1e-9
                        or (m.get("max_master") or 0) > 1e-9)
    pooled_eng = sorted(run for es in ("L", "tier3s") for run, e in E.get(("is1", es, None), {}).items()
                        if (stat(e, "switch.max|workers.trained") or 0) > 1e-9
                        or (stat(e, "switch.max|workers.master") or 0) > 1e-9)
    out.add("E5", "is1", "L+tier3s", "E5", "-", "runs where a type's share passes 1e-9",
            f"{len(pooled_reg)} of 129", f"{len(pooled_eng)} of 129", "the same runs",
            "pass" if pooled_reg == pooled_eng else "fail")
    # E6: only JB(0.5) breaches the wall in the battery
    br_reg = sorted(run for run, (_, m) in R[("is1", "L", None)].items() if (m.get("breach_ticks") or 0) > 0)
    br_eng = sorted(run for run, e in E.get(("is1", "L", None), {}).items()
                    if (stat(e, "wall.breach_ticks|-") or 0) > 0)
    out.add("E6", "is1", "L", "E6", "-", "the battery's runs that breach the wall", br_reg, br_eng, "the same runs",
            "pass" if br_reg == br_eng else "fail")
    # E2: mode A and L
    modea_rows = []
    for inst in ("is1", "is2"):
        for tpy in (52, 12, 365):
            e = E.get((inst, "modea", None), {}).get(tpy)
            ok = ok_record(e) and e.get("mode_a") == "PASS" and e["peak_dhat"] * 1e-3 < 1e-9
            sc = inst == "is1"
            out.add("E2", inst, "modea", "E2", f"hold at {tpy} a year", "mode A at L, largest gap below 1e-9",
                    "PASS", None if not ok_record(e) else e["peak_dhat"] * 1e-3, "PASS, below 1e-9",
                    ("pass" if ok else ("missing" if not ok_record(e) else "fail")) if sc else "reported")
            el = E.get((inst, "info", None), {}).get(f"el{tpy}")
            Lr = LT[inst][tpy]
            st = "pass" if ok_record(el) and el.get("L") == Lr else ("missing" if not ok_record(el) else "fail")
            out.add("E2", inst, "info", "E2", f"elasticity at {tpy} a year", "L from the engine's elasticity", Lr,
                    None if not ok_record(el) else el.get("L"), "equal", st if sc else "reported")
            modea_rows.append(OrderedDict(inst=inst, tpy=tpy, L=Lr, engine_L=(el or {}).get("L"),
                                          engine_mode_a=(e or {}).get("mode_a"),
                                          engine_gap=None if not ok_record(e) else e["peak_dhat"] * 1e-3,
                                          end_trained=stat(e, "switch.end|workers.trained"),
                                          end_master=stat(e, "switch.end|workers.master")))
    # E4: kick sets, the envelope, the corner rate
    kick_rows, kicks_ok = [], {}
    for inst in ("is1", "is2"):
        allok = True
        for t in ["hold"] + TARGETS:
            k = E.get((inst, "kick", None), {}).get(fname(t))
            ok = ok_record(k) and bool(k.get("passed")) and k.get("gain_tail", math.inf) <= 1e-3
            allok &= ok
            st = "pass" if ok else ("missing" if not ok_record(k) else "fail")
            out.add("E4" if inst == "is1" else "E8", inst, "kick", "kick", t, "kick set decays (gain_tail at most 1e-3)",
                    "PASS", None if not ok_record(k) else [k.get("passed"), k.get("gain_tail")], "PASS, at most 1e-3",
                    st if inst == "is1" else "reported")
            kick_rows.append(OrderedDict(inst=inst, target=t, passed=(k or {}).get("passed"),
                                         gain_tail=(k or {}).get("gain_tail"), gain_peak=(k or {}).get("gain_peak"),
                                         kicks=(k or {}).get("kicks")))
        kicks_ok[inst] = allok
    env_rows = []
    for s, reg_year, scored in (("env", 0.534, True), ("env-ts011", 0.686, False)):
        env = E.get(("is1", "envelope", None), {}).get(s)
        rates = [k["rate"] for k in (env or {}).get("kicks", []) if k.get("rate")]
        slow = max(rates) if rates else None
        st = "missing" if slow is None else ("pass" if abs(slow ** 52 - reg_year) <= 0.03 else "fail")
        out.add("E4", "is1", s, "E4", "base" if s == "env" else "tail.services=0.11",
                "the kick envelope's decay, a year", reg_year, None if slow is None else slow ** 52,
                "within 0.03 a year" if scored else "reported (README: kicked at genesis with the shock)",
                st if scored else "reported")
        for k in (env or {}).get("kicks", []):
            env_rows.append(OrderedDict(set=s, **k))
    corner = [r for r in recs if r.get("key") == "switch" and r.get("set") == "corner"]
    worst, n = None, 0
    for r in corner:
        c = r.get("corner") or {}
        if c.get("ticks"):
            n += c["ticks"]
            worst = max(worst or 0.0, c["max_rel_dev"])
    out.add("E4", "is1", "corner", "E4", "sw[T]=V, RW(2), x*/2", END_LINES[4], "exact", worst,
            "to rounding: within 1e-12 relative, on ticks with a normal a and a negative gap",
            "missing" if (self_test or worst is None) else ("pass" if worst <= 1e-12 and n > 0 else "fail"))
    # E7: the families' counts; Hold as Saturate
    fam_rows = []
    for (inst, es, setting), rows in R.items():
        eng = E.get((inst, es, setting), {})
        step = next(iter(rows.values()))[0]
        mc, ec = defaultdict(int), defaultdict(int)
        for run, (_, m) in rows.items():
            mc[m["cls"]] += 1
            ec[(eng.get(run) or {}).get("class", "missing")] += 1
        out.add(step, inst, es, f"{es} {setting}" if setting else es, "-", "classes", dict(sorted(mc.items())),
                dict(sorted(ec.items())), "equal",
                ("pass" if dict(mc) == dict(ec) else "fail") if inst == "is1" else "reported")
        conv = [(m, eng.get(r)) for r, (_, m) in rows.items() if m["cls"] == "CONVERGED"]
        fam_rows.append(OrderedDict(
            step=step, inst=inst, set=es, setting=setting, runs=len(rows), mirror=json.dumps(dict(sorted(mc.items()))),
            engine=json.dumps(dict(sorted(ec.items()))),
            ttol_median_mirror=med([m.get("ticks_to_tol") for m, _ in conv]),
            ttol_median_engine=med([(e or {}).get("in_tol_from") for _, e in conv]),
            ttol_max_mirror=max([m.get("ticks_to_tol") or 0 for m, _ in conv], default=None),
            ttol_max_engine=max([(e or {}).get("in_tol_from") or 0 for _, e in conv], default=None),
            dead_max_mirror=max([m.get("dead") or 0 for m, _ in conv], default=None),
            dead_max_engine=max([(e or {}).get("dead") or 0 for _, e in conv], default=None),
            baskets_low_mirror=min([m.get("baskets_trough") for m, _ in conv if m.get("baskets_trough") is not None],
                                   default=None),
            baskets_low_engine=min([(e or {}).get("low_baskets") for _, e in conv
                                    if (e or {}).get("low_baskets") is not None], default=None),
            pooled_runs_mirror=sum(1 for m, _ in conv if (m.get("max_trained") or 0) > 1e-9
                                   or (m.get("max_master") or 0) > 1e-9),
            pooled_runs_engine=sum(1 for _, e in conv if (stat(e, "switch.max|workers.trained") or 0) > 1e-9
                                   or (stat(e, "switch.max|workers.master") or 0) > 1e-9)))
    for inst in ("is1", "is2"):
        same, total = 0, 0
        for run in R.get((inst, "hold", None), {}):
            a, b = E.get((inst, "hold", None), {}).get(run), E.get((inst, "L", None), {}).get(run)
            if not ok_record(a) or not ok_record(b):
                continue
            total += 1
            keys = ("class", "in_tol_from", "peak_dhat", "dead", "low_baskets", "no_basket_ticks", "worst_fill",
                    "transfer_short", "last")
            same += all(a.get(k) == b.get(k) for k in keys) and a.get("stats") == b.get("stats")
        n = len(R.get((inst, "hold", None), {}))
        out.add("E7" if inst == "is1" else "E8", inst, "hold", "hold", "-",
                "Hold identical to Saturate in every statistic", f"{n}/{n}", f"{same}/{total}", "all",
                ("pass" if same == total == n else "fail") if inst == "is1" else "reported")
    # E9: the points
    point_rows = []
    for inst in ("is1", "is2"):
        for tpy in (52, 12, 365):
            pe = E.get((inst, "info", None), {}).get(f"point{tpy}")
            pts = {p["label"]: p for p in (pe or {}).get("points", [])}
            for label in ["base"] + [t.split("@")[0] for t in TARGETS]:
                reg = MP[inst].get(f"{label}@{tpy}")
                p = pts.get(label)
                if reg is None:
                    continue
                if p is None:
                    out.add("E9", inst, "point", "E9", f"{label} at {tpy}", "pooled, a*, switch distances",
                            reg["pooled"], None, "exactly; 1e-12", "missing" if inst == "is1" else "reported")
                    continue
                da = max(abs(x - y) for x, y in zip(p.get("pool shares", []), reg["a"])) if p.get("pool shares") else math.inf
                dd = max(abs(x - y) for x, y in zip(p.get("switch distances", []), reg["switch"])) \
                    if p.get("switch distances") else math.inf
                ok = p.get("pooled") == reg["pooled"] and da <= 1e-12 and dd <= 1e-12
                out.add("E9", inst, "point", "E9", f"{label} at {tpy}", "pooled flags exactly; a* and switch distances",
                        [reg["pooled"], reg["a"], reg["switch"]], [p.get("pooled"), p.get("pool shares"),
                                                                    p.get("switch distances")],
                        "flags exactly; within 1e-12", ("pass" if ok else "fail") if inst == "is1" else "reported")
                point_rows.append(OrderedDict(inst=inst, tpy=tpy, label=label, pooled_reg=reg["pooled"],
                                              pooled_engine=p.get("pooled"), a_gap=da, switch_gap=dd))
    if p23_point is not None:
        iw = {p["label"]: p for p in p23_point.get("points", [])}
        pe = E.get(("is1", "info", None), {}).get("point52")
        pts = {p["label"]: p for p in (pe or {}).get("points", [])}
        for label in ["base"] + [t.split("@")[0] for t in TARGETS]:
            reg = MP["is1"].get(f"{label}@52")
            if reg is None or any(reg["pooled"]):
                continue
            a, b = pts.get(label), iw.get(label)
            if a is None or b is None:
                out.add("E9", "is1", "point", "E9", label, "a walled target's point is IW1's, double for double",
                        "IW1's", None, "exactly", "missing")
                continue
            keys = sorted(k for k in b if k not in ("inst", "label") and k in a)
            diff = [k for k in keys if a[k] != b[k]]
            out.add("E9", "is1", "point", "E9", label, "a walled target's point is IW1's, double for double",
                    f"{len(keys)} fields", f"{len(keys) - len(diff)} equal" + (f"; differ: {diff}" if diff else ""),
                    "exactly", "pass" if not diff else "fail")
    # the verdict
    t3 = {r for r, (_, m) in R[("is1", "L", None)].items() if str(m.get("tier")) == "3"}
    tallies = OrderedDict()
    for label, es, pick in (("Tier 1", "L", lambda r, m: str(m.get("tier")) == "1"),
                            ("Tier 2", "L", lambda r, m: str(m.get("tier")) == "2"),
                            ("Tier 3", "L", lambda r, m: str(m.get("tier")) == "3"),
                            ("Tier 3S", "tier3s", lambda r, m: True),
                            ("Tier 3 at 10L", "10L", lambda r, m: r in t3),
                            ("Tier 3S at 10L", "10L", lambda r, m: r not in t3)):
        rows = [(r, m) for r, (_, m) in R.get(("is1", es, None), {}).items() if pick(r, m)]
        eng = E.get(("is1", es, None), {})
        n = sum(1 for r, _ in rows if (eng.get(r) or {}).get("class") == "CONVERGED")
        tallies[label] = (n, len(rows))
        out.add("E3", "is1", label, "E3", "-", "runs CONVERGED", f"{len(rows)}/{len(rows)}", f"{n}/{len(rows)}",
                "all", "pass" if n == len(rows) else "fail")
    ma = any(r["status"] == "pass" for r in out.rows if r["step"] == "E2" and r["inst"] == "is1"
             and r["run"] == "hold at 52 a year" and r["what"].startswith("mode A"))
    conv = all(n == t for n, t in tallies.values())
    t12 = tallies["Tier 1"][0] == tallies["Tier 1"][1] and tallies["Tier 2"][0] == tallies["Tier 2"][1]
    verdict = "GO" if ma and conv and kicks_ok["is1"] else ("LOCAL" if ma and t12 else "NO-GO")
    out.add("E3", "is1", "verdict", "verdict", "-", "IS1's verdict", "GO", verdict,
            "mode A; Tiers 1-3 and 3S CONVERGED at L and 10 L; every kick set", "pass" if verdict == "GO" else "fail")
    verdicts = [OrderedDict(inst="is1", verdict=verdict, prediction="GO",
                            **{k: f"{n}/{t}" for k, (n, t) in tallies.items()},
                            kicks="13/13 PASS" if kicks_ok["is1"] else "not all PASS",
                            is2_kicks="13/13 PASS" if kicks_ok["is2"] else "not all PASS")]
    out.charge()
    return out, dict(per_run=per_run, fam=fam_rows, kicks=kick_rows, env=env_rows, modea=modea_rows,
                     points=point_rows, verdicts=verdicts)


def refutations(out, self_test):
    res = []
    bad = [r for r in out.rows if r["what"] == "class" and r["status"] == "fail" and r["inst"] == "is1"
           and r["set"] in ("L", "tier3s", "10L")]
    if bad:
        res.append(f"{len(bad)} class changes in IS1's battery or Tier 3S at L or 10 L")
    if not self_test:
        off = [r for r in out.rows if r["what"] == "end D-hat" and r["status"] == "fail"]
        if off:
            res.append(f"{len(off)} CONVERGED runs ending more than 1e-12 in log off their point")
        side = [r for r in out.rows if r["what"] == END_LINES[3] and r["status"] == "fail"]
        pooled = [r for r in out.rows if r["what"] == END_LINES[1] and r["status"] == "fail"
                  and not (r["engine"] not in ("-", "None") and abs(float(r["engine"]) - float(r["registered"]))
                           <= 1e-9 * float(r["registered"]))]
        if side or pooled:
            res.append(f"{len(side)} switch pops ending on the wrong side of their switch, {len(pooled)} pooled pops "
                       f"more than 1e-9 relative from a*")
    for what, label in (("mode A at L, largest gap below 1e-9", "a rest point off unit 1d's"),
                        ("pooled flags exactly; a* and switch distances", "a point off switch_mp's")):
        f = [r for r in out.rows if r["what"] == what and r["status"] == "fail" and r["inst"] == "is1"]
        if f:
            res.append(f"{len(f)} lines: {label}")
    nv = [r for r in out.rows if r["step"] == "E1" and r["run"] != "-" and r["status"] == "fail"]
    if nv:
        res.append(f"{len(nv)} never-pooled lines differing from IW1's")
    ks = [r for r in out.rows if r["what"].startswith("kick set decays") and r["status"] == "fail"]
    if ks:
        res.append(f"{len(ks)} kick sets failing")
    return res


# ---------------------------------------------------------------- the self-test's records
def selftest_records():
    """The registered mirror rows written as gather.py writes the engine's records."""
    recs = []
    R = registered()
    MP = mp()
    for (inst, es, setting), rows in R.items():
        for run, (step, m) in rows.items():
            L = int(m["L"])
            stats = {"wall.depth_min|-": m.get("depth_min"), "wall.breach_ticks|-": m.get("breach_ticks"),
                     "wall.worst_buyer_fill|-": m.get("worst_fill")}
            for mk in WALL_MARKETS:
                stats[f"dead.below_floor|{mk}"] = m.get(f"dead_{mk}")
            for t, pop in POPS.items():
                stats[f"switch.live_ticks|{pop}"] = carried_live(m, t, L)
                stats[f"switch.max|{pop}"] = m.get(f"max_{t}")
                stats[f"switch.band|{pop}"] = m.get(f"band_{t}")
                stats[f"switch.end|{pop}"] = m.get(f"end_{t}")
                stats[f"switch.gap_end|{pop}"] = m.get(f"gapend_{t}")
            recs.append(dict(key="switch", cmd="run", inst=inst, set=es, setting=setting, names=[run], rc=0,
                             ticks=L, in_tol_from=None if m.get("ticks_to_tol") is None else int(m["ticks_to_tol"]),
                             dead=None if m.get("dead") is None else int(m["dead"]), low_baskets=m.get("baskets_trough"),
                             no_basket_ticks=m.get("baskets_none"), peak_dhat=m.get("peak_Dhat"),
                             transfer_short=m.get("transfer_short"), last=None, stats=stats, **{"class": m["cls"]}))
    # hold = the battery's own numbers (Tiers 1-2), as the registration predicts
    bat = {(r["inst"], r["names"][0]): r for r in recs if r["set"] == "L"}
    for r in recs:
        if r["set"] == "hold":
            b = bat[(r["inst"], r["names"][0])]
            for k in ("in_tol_from", "peak_dhat", "dead", "low_baskets", "no_basket_ticks", "transfer_short", "last",
                      "stats", "class"):
                r[k] = b[k]
    for inst in ("is1", "is2"):
        for tpy in (52, 12, 365):
            recs.append(dict(key="switch", cmd="run", inst=inst, set="modea", tpy=tpy, names=["hold"], rc=0,
                             mode_a="PASS", peak_dhat=1e-12, **{"class": "CONVERGED"}))
            recs.append(dict(key="switch", cmd="elasticity", inst=inst, set="info", tpy=tpy,
                             rel=f"switch/{inst}/info/el{tpy}", rc=0, L=LT[inst][tpy]))
            pts = []
            for label in ["base"] + [t.split("@")[0] for t in TARGETS]:
                reg = MP[inst].get(f"{label}@{tpy}")
                if reg:
                    pts.append({"inst": inst, "label": label, "pooled": reg["pooled"], "pool shares": reg["a"],
                                "switch distances": reg["switch"]})
            recs.append(dict(key="switch", cmd="point", inst=inst, set="info", tpy=tpy,
                             rel=f"switch/{inst}/info/point{tpy}", rc=0, points=pts))
        for t in ["hold"] + TARGETS:
            recs.append(dict(key="switch", cmd="kick", inst=inst, set="kick", rel=f"switch/{inst}/kick/{fname(t)}",
                             rc=0, passed=True, gain_tail=1.4e-5, gain_peak=3.0, kicks=14))
    recs.append(dict(key="switch", cmd="envelope", inst="is1", set="env", rc=0,
                     kicks=[dict(run="p[labour]*1.000000001", rate=0.9880024693036012)]))
    recs.append(dict(key="switch", cmd="envelope", inst="is1", set="env-ts011", rc=0,
                     kicks=[dict(run="p[labour]*1.000000001", rate=0.9927682410409551)]))
    return recs


def selftest_negative(recs):
    """Five records made wrong: a class in the battery, a never-pooled run's ticks off by one, a
    pooled run's largest a 20% off, a failing kick set at a target, and a slower envelope."""
    import copy
    bad = copy.deepcopy(recs)
    done = set()
    for r in bad:
        n = (r.get("names") or ["-"])[0]
        if r.get("cmd") == "run" and r["inst"] == "is1" and r["set"] == "L" and n == "JA(2)" and "a" not in done:
            r["class"] = "ORBITING"
            done.add("a")
        elif r.get("cmd") == "run" and r["inst"] == "is1" and r["set"] == "L" and n == "JA(0.95)" and "b" not in done:
            r["in_tol_from"] += 1
            done.add("b")
        elif r.get("cmd") == "run" and r["inst"] == "is1" and r["set"] == "L" and n == "RW(2)" and "c" not in done:
            r["stats"]["switch.max|workers.trained"] *= 1.2
            done.add("c")
        elif r.get("cmd") == "kick" and r["inst"] == "is1" and r["rel"].endswith("land.mach_0.8_dated") and "d" not in done:
            r["passed"] = False
            done.add("d")
        elif r.get("cmd") == "envelope" and r["set"] == "env" and "e" not in done:
            r["kicks"][0]["rate"] = 0.9895
            done.add("e")
    assert done == set("abcde"), done
    return bad


def main(runs_path, out_dir, self_test=False, p23_path=P23):
    recs = read_recs(runs_path)
    out, T = score(recs, self_test, p23_path)
    ref = refutations(out, self_test)
    write_lines(out_dir, out)
    for k, fn in (("per_run", "runs.csv"), ("fam", "families.csv"), ("kicks", "kicks.csv"), ("env", "envelope.csv"),
                  ("modea", "modea.csv"), ("points", "points.csv"), ("verdicts", "verdicts.csv")):
        write_rows(os.path.join(out_dir, fn), T[k])
    st = printout(out, f"switch: {len(recs)} records")
    for v in T["verdicts"]:
        print(f"  verdict {v['inst']}: {v['verdict']} (registered {v['prediction']})")
    print("refutations: " + (str(ref) if ref else "none"))
    miss = [r for r in out.rows if r["status"] == "missing"]
    if self_test:
        other = [r for r in miss if r["what"] not in END_LINES]
        print(f"self-test: {st['fail']} lines fail; {len(miss)} missing, of which {len(miss) - len(other)} "
              f"are end-of-run or every-tick lines only the engine's runs give and {len(other)} others")
        return st["fail"] == 0 and not other and not ref
    return st["fail"] == 0 and st["missing"] == 0 and not ref


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    p23 = sys.argv[sys.argv.index("--p23") + 1] if "--p23" in sys.argv else P23
    if "--p23" in sys.argv:
        args.remove(p23)
    sys.exit(0 if main(args[0], args[1], "--self-test" in sys.argv, p23) else 1)
