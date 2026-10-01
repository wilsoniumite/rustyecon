"""P2.4 (label families, 2026-09-30): the scorer of wave A, the families wave, committed before the
wave's first job (decision 311). It scores runs.jsonl (gather.py) against the registered
predictions in docs/probe/families/registered/ (SPEC.md §5, registration.md), line by line, and
writes lines.csv (every line), the tables, and a printout with the tallies and the refutation
criteria. The bands (SPEC §5, as P2.3's):

- every run's class exactly, the mirror's, in every set (VACUOUS included);
- ticks to tolerance within 10% where both runs are CONVERGED, with up to three runs a set
  (an instance's set, one I3 map cell, or one instance's dial setting) charged within 25%;
- a DIVERGED run's runaway tick on the scored clock within 5%, on the harness's reference (the
  displaced genesis prices; O107);
- dead ticks within 10% or 5, whichever is larger;
- the lowest baskets eaten over Y* within 0.05 absolute where the mirror's is above 0.1
  (reported below it);
- the history, window by window: in tolerance at the window's end as the mirror's, its ticks to
  tolerance (10%, three windows an instance charged within 25%) and dead ticks (10% or 5), and
  the runaway's window and tick (5%) if the mirror's runs away;
- I3's map cells: mode A PASS where registered; each kick set's pass as registered (the mirror's
  largest root PL: PASS iff PL^(0.9 L) <= 1e-3); the cell's verdict (MARKETS-SPEC 7.9 with the
  kick) as registered;
- the dial family's base kick sets: reported beside the mirror's PL (SPEC §5.4), not scored.

A line's status is pass, fail, charged (between 10% and 25%, within the allowance), reported, or
missing (no engine number, which fails a scored line). The refutation criteria (SPEC §5.5): a
class that is not the mirror's; a CONVERGED run whose last tick's D-hat exceeds 1e-9 (a gap of
1e-12 in log: it rests off its oracle point); a scored kick set or a map cell's verdict that is
not the registered one.

Usage (WSL): python3 score.py RUNS.jsonl OUT_DIR [--self-test]"""
import csv
import json
import math
import os
import statistics
import sys
from collections import OrderedDict, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
REG = os.path.join(REPO, "docs", "probe", "families", "registered")
L_I = {"i1": 144000, "i2": 52000, "i3": 484000}
MAP_L = {"r0.5b0.5": 968000, "r0.5b1": 968000, "r0.5b2": 968000, "r1b0.5": 484000,
         "r1b2": 484000}
L_D = {"iw1": 22000, "c1": 141000, "c2": 142000}


# ---------------------------------------------------------------- lines (p23-wave/score.py's)
class Lines:
    def __init__(self):
        self.rows = []

    def add(self, part, inst, set_, group, run, what, reg, eng, band, status):
        self.rows.append(OrderedDict(part=part, inst=inst, set=set_, group=group, run=run,
                                     what=what, registered=fmt(reg), engine=fmt(eng), band=band,
                                     status=status))
        return status

    def charge(self, limit=3):
        """The 25% allowance: at most `limit` runs a group may be charged; beyond it, a charged
        run's lines fail."""
        runs = defaultdict(list)
        for r in self.rows:
            if r["status"] == "charged":
                runs[(r["part"], r["inst"], r["set"], r["group"])].append(r)
        for key, rs in runs.items():
            seen = []
            for r in rs:
                if r["run"] not in seen:
                    seen.append(r["run"])
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
    return "pass" if abs(e - m) <= frac * abs(m) else "fail"


def st_count(m, e, frac=0.1, floor=5):
    if m is None or e is None:
        return "pass" if m is None and e is None else "fail"
    return "pass" if abs(e - m) <= max(frac * m, floor) else "fail"


def st_abs(m, e, tol=0.05, above=0.1):
    if m is None or m <= above or (isinstance(m, float) and not math.isfinite(m)):
        return "reported"
    if e is None or (isinstance(e, float) and math.isnan(e)):
        return "fail"
    return "pass" if abs(e - m) <= tol else "fail"


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


def kick_bar(pl, L):
    """The mirror's kick prediction (MARKETS-SPEC 7.5 on the mirror): PL^(0.9 L) <= 1e-3."""
    return pl is not None and pl < 1.0 and (0.9 * L) * math.log(pl) <= math.log(1e-3)


# ---------------------------------------------------------------- the registered files
def jsonl(name):
    with open(os.path.join(REG, name)) as f:
        return [json.loads(line) for line in f if line.strip()]


def registered():
    R = {"i": {}, "map": {}, "map_pl": {}, "dial": {}, "dial_pl": {}, "history": {}}
    for s in ("i_stocks", "i_joint2", "i_joint4", "i_basin", "i_hold", "i_tilt1"):
        for r in jsonl(f"{s}.jsonl"):
            R["i"][(s[2:], r["inst"].lower(), r["run"])] = r
    for r in jsonl("i3_map.jsonl"):
        R["map"][(r["cell"], r["run"])] = r
    for r in jsonl("i3_map_pl.jsonl"):
        R["map_pl"][(r["cell"], r["target"])] = r
    for r in jsonl("dial.jsonl"):
        R["dial"][(r["inst"].lower(), r["dials"], r["run"])] = r
    for r in jsonl("dial_pl.jsonl"):
        R["dial_pl"][(r["inst"].lower(), r["dials"])] = r
    with open(os.path.join(REG, "history.json")) as f:
        for h in json.load(f):
            R["history"][h["inst"].lower()] = h
    return R


# ---------------------------------------------------------------- one run's lines
def run_lines(out, part, inst, set_, group, m, e, self_test):
    """The lines of one run: m the registered row, e the engine's record (or None)."""
    run = m["run"]
    if e is None or e.get("rc") != 0 or "missing" in e or "gather_error" in e:
        why = "no record" if e is None else (e.get("missing") or e.get("gather_error")
                                             or f"exit {e.get('rc')}")
        out.add(part, inst, set_, group, run, "class", m["cls"], why, "exactly", "missing")
        return
    ec = e.get("class")
    out.add(part, inst, set_, group, run, "class", m["cls"], ec, "exactly",
            "pass" if ec == m["cls"] else "fail")
    if m["cls"] == "CONVERGED" and ec == "CONVERGED":
        out.add(part, inst, set_, group, run, "ticks to tolerance", m.get("ttol"),
                e.get("in_tol_from"), "10%, three runs a set within 25%",
                st_ticks(m.get("ttol"), e.get("in_tol_from")))
    if m["cls"] == "DIVERGED" and ec == "DIVERGED":
        out.add(part, inst, set_, group, run, "runaway tick", m.get("runaway_t"),
                e.get("runaway_tick"), "5% (the harness's reference)",
                st_rel(m.get("runaway_t"), e.get("runaway_tick"), 0.05))
    if m.get("dead") is not None and ec not in ("DIVERGED", "ERROR") and m["cls"] == ec:
        out.add(part, inst, set_, group, run, "dead ticks", m.get("dead"), e.get("dead"),
                "10% or 5", st_count(m.get("dead"), e.get("dead")))
    if m["cls"] == ec and ec not in ("DIVERGED", "ERROR"):
        out.add(part, inst, set_, group, run, "lowest baskets over Y*", m.get("baskets_trough"),
                e.get("low_baskets"), "0.05 where the mirror's is above 0.1",
                st_abs(m.get("baskets_trough"), e.get("low_baskets")))
    if ec == "CONVERGED":
        last = e.get("last")
        if self_test:
            out.add(part, inst, set_, group, run, "end D-hat", None, last,
                    "at most 1e-9 (a gap of 1e-12 in log)", "missing")
        else:
            out.add(part, inst, set_, group, run, "end D-hat", 1e-9, last,
                    "at most 1e-9 (a gap of 1e-12 in log)",
                    "pass" if last is not None and last <= 1e-9 else "fail")


# ---------------------------------------------------------------- the sets
def score(recs, R, self_test):
    out = Lines()
    E = defaultdict(dict)
    for e in recs:
        if e.get("cmd") == "run" and e["set"] in ("stocks", "joint2", "joint4", "basin", "hold",
                                                  "tilt1", "history"):
            E[("i", e["set"], e["inst"])][e["names"][0]] = e
        elif e["set"] == "map":
            E[("map", e["cell"])][e["names"][0]] = e
        elif e["set"] == "map_modea":
            E[("map", e["cell"])]["hold"] = e
        elif e["set"] == "map_kick":
            E[("map_kick", e["cell"])][e["names"][0]] = e
        elif e["set"] == "dial":
            E[("dial", e["inst"], e["setting"])][e["names"][0]] = e
        elif e["set"] == "dial_kick":
            E[("dial_kick", e["inst"])][e["setting"]] = e
    # A1: O22's families on I1-I3
    for (s, inst, run), m in R["i"].items():
        e = E[("i", s, inst)].get(run)
        run_lines(out, "A1", inst, s, s, m, e, self_test)
    for inst, h in R["history"].items():
        e = E[("i", "history", inst)].get(h["run"])
        wins = (e or {}).get("windows") or []
        for w in h["windows"]:
            ew = next((x for x in wins if x["w"] == w["w"]), None)
            reg_end = w["end_Dh"] is not None and w["end_Dh"] <= 1
            if ew is None:
                out.add("A1", inst, "history", "history", f"window {w['w']}", "in tolerance at its end",
                        reg_end, None, "exactly", "missing")
                continue
            out.add("A1", inst, "history", "history", f"window {w['w']}", "in tolerance at its end",
                    reg_end, ew["in_tol_at_end"], "exactly",
                    "pass" if reg_end == ew["in_tol_at_end"] else "fail")
            out.add("A1", inst, "history", "history", f"window {w['w']}", "ticks to tolerance",
                    w["ttol"], ew["ttol"], "10%, three windows within 25%", st_ticks(w["ttol"], ew["ttol"]))
            out.add("A1", inst, "history", "history", f"window {w['w']}", "dead ticks", w["dead"],
                    ew["dead"], "10% or 5", st_count(w["dead"], ew["dead"]))
        if h.get("err"):
            # "runaway at window W, t=T" (fam_i.run_cycle): the window exactly, the tick 5%
            ww, tt = None, None
            if h["err"].startswith("runaway at window"):
                a, b = h["err"][len("runaway at window "):].split(", t=")
                ww, tt = int(a), int(b)
            ew = (e or {}).get("runaway_window")
            ok = ew is not None and ww is not None and ew[0] == ww and abs(ew[1] - tt) <= 0.05 * tt
            out.add("A1", inst, "history", "history", h["run"], "runaway", [ww, tt], ew,
                    "window exactly, tick 5%", "missing" if e is None else ("pass" if ok else "fail"))
        elif e is not None:
            out.add("A1", inst, "history", "history", h["run"], "runaway", None,
                    e.get("runaway_window"), "none, as registered",
                    "pass" if e.get("runaway_window") is None else "fail")
    # I3's map cells
    cells = sorted({c for c, _ in R["map"]})
    verdicts = []
    for cell in cells:
        L = MAP_L[cell]
        Em = E[("map", cell)]
        Ek = E[("map_kick", cell)]
        for (c, run), m in R["map"].items():
            if c != cell:
                continue
            e = Em.get(run)
            if run == "hold":
                reg_a = "PASS" if m.get("peak", math.inf) <= 1e-6 else "FAIL"
                eng_a = None if e is None else ("PASS" if (e.get("mode_a") or "").startswith("PASS")
                                                else e.get("mode_a"))
                out.add("A1", "i3", "map", cell, "hold", "mode A", reg_a, eng_a,
                        "PASS: every gap at most 1e-9, every market live, fills at least 1 - 1e-9",
                        "missing" if e is None else ("pass" if eng_a == reg_a else "fail"))
                continue
            run_lines(out, "A1", "i3", "map", cell, m, e, self_test)
        kicks_reg, kicks_eng = {}, {}
        for (c, target), p in R["map_pl"].items():
            if c != cell:
                continue
            name = "hold" if target == "base" else f"{target}@dated"
            reg = kick_bar(p["PL"], L)
            kicks_reg[name] = reg
            ek = Ek.get(name)
            eng = None if ek is None or "passed" not in ek else ek["passed"]
            kicks_eng[name] = eng
            out.add("A1", "i3", "map_kick", cell, name, "kick set passes", reg, eng,
                    f"as the mirror's PL {p['PL']:.6f} predicts (PL^(0.9 L) <= 1e-3)",
                    "missing" if eng is None else ("pass" if eng == reg else "fail"))
        verdicts.append((cell, cell_verdict(R, cell, lambda r: r.get("cls"), kicks_reg,
                                            R["map"][(cell, "hold")].get("peak", math.inf) <= 1e-6),
                         cell_verdict(R, cell, lambda r: (Em.get(r["run"]) or {}).get("class"),
                                      kicks_eng, (Em.get("hold") or {}).get("mode_a", "").startswith("PASS"))))
    for cell, vr, ve in verdicts:
        out.add("A1", "i3", "map", cell, "verdict", "the cell's verdict (MARKETS-SPEC 7.9)", vr, ve,
                "exactly", "pass" if vr == ve else ("missing" if ve is None else "fail"))
    # A2: the dial neighbourhood
    for (inst, dials, run), m in R["dial"].items():
        e = E[("dial", inst, dials)].get(run)
        run_lines(out, "A2", inst, "dial", dials, m, e, self_test)
    for (inst, dials), p in R["dial_pl"].items():
        ek = E[("dial_kick", inst)].get(dials)
        reg = kick_bar(p["PL"], L_D[inst])
        eng = None if ek is None or "passed" not in ek else ek["passed"]
        out.add("A2", inst, "dial_kick", dials, "hold", "base kick set passes", reg, eng,
                f"reported beside the mirror's PL {p['PL']:.6f}", "reported")
    out.charge()
    return out, E, verdicts


def cell_verdict(R, cell, cls_of, kicks, mode_a):
    """MARKETS-SPEC 7.9 with the kick: GO if mode A passes (with its base kick), and every
    non-vacuous run of Tiers 1-3 is CONVERGED with its target's kick passing; LOCAL if Tiers 1-2
    are; NO-GO otherwise. A run's target is its cost shock's (dated), or the base."""
    if not mode_a or not kicks.get("hold"):
        return "NO-GO"
    ok = {1: True, 2: True, 3: True}
    for (c, run), m in R["map"].items():
        if c != cell or run == "hold":
            continue
        cls = cls_of(m)
        if cls is None:
            return None
        if cls == "VACUOUS":
            continue
        tgt = "hold"
        if "=" in run and "@" in run:
            tgt = run.split("@")[0] + "@dated"
        conv = cls == "CONVERGED" and kicks.get(tgt, False)
        if not conv:
            ok[int(m["tier"])] = False
    if all(ok.values()):
        return "GO"
    if ok[1] and ok[2]:
        return "LOCAL"
    return "NO-GO"


# ---------------------------------------------------------------- tables
def tables(out_dir, lines, R, E, verdicts):
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "lines.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(lines.rows[0]))
        w.writeheader()
        w.writerows(lines.rows)
    fails = [r for r in lines.rows if r["status"] in ("fail", "missing")]
    with open(os.path.join(out_dir, "fails.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(lines.rows[0]))
        w.writeheader()
        w.writerows(fails)
    # A1 tallies: per set and instance
    rows = []
    for s in ("stocks", "joint2", "joint4", "basin", "hold", "tilt1"):
        for inst in ("i1", "i2", "i3"):
            ms = [m for (ss, ii, _), m in R["i"].items() if ss == s and ii == inst]
            es = [E[("i", s, inst)].get(m["run"]) or {} for m in ms]
            rows.append(tally_row(f"{s}", inst, "-", ms, es))
    for cell in sorted({c for c, _ in R["map"]}):
        ms = [m for (c, run), m in R["map"].items() if c == cell and run != "hold"]
        es = [E[("map", cell)].get(m["run"]) or {} for m in ms]
        rows.append(tally_row("map", "i3", cell, ms, es))
    write_rows(os.path.join(out_dir, "families.csv"), rows)
    # A2 tallies: per instance, setting and tier
    rows = []
    for inst in ("iw1", "c1", "c2"):
        settings = []
        for (i, d, _) in R["dial"]:
            if i == inst and d not in settings:
                settings.append(d)
        for d in settings:
            for tier in ("3", "3S"):
                ms = [m for (i, dd, _), m in R["dial"].items() if i == inst and dd == d and m["tier"] == tier]
                es = [E[("dial", inst, d)].get(m["run"]) or {} for m in ms]
                r = tally_row(f"tier {tier}", inst, d, ms, es)
                p = R["dial_pl"].get((inst, d))
                ek = E[("dial_kick", inst)].get(d) or {}
                r["pl"] = None if p is None else round(p["PL"], 6)
                r["kick_engine"] = ek.get("passed")
                rows.append(r)
    write_rows(os.path.join(out_dir, "dial.csv"), rows)
    rows = [dict(cell=c, registered=vr, engine=ve) for c, vr, ve in verdicts]
    write_rows(os.path.join(out_dir, "map_verdicts.csv"), rows)


def tally_row(set_, inst, group, ms, es):
    cm = defaultdict(int)
    ce = defaultdict(int)
    for m, e in zip(ms, es):
        cm[m["cls"]] += 1
        ce[e.get("class", "missing")] += 1
    tm = [m.get("ttol") for m in ms if m["cls"] == "CONVERGED"]
    te = [e.get("in_tol_from") for e in es if e.get("class") == "CONVERGED"]
    fmtc = lambda c: "; ".join(f"{k} {v}" for k, v in sorted(c.items()))
    return OrderedDict(set=set_, inst=inst, group=group, runs=len(ms), mirror=fmtc(cm),
                       engine=fmtc(ce), ttol_median_mirror=med(tm), ttol_median_engine=med(te),
                       ttol_max_mirror=max([x for x in tm if x is not None], default=None),
                       ttol_max_engine=max([x for x in te if x is not None], default=None))


def write_rows(path, rows):
    if not rows:
        return
    keys = []
    for r in rows:
        for k in r:
            if k not in keys:
                keys.append(k)
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=keys)
        w.writeheader()
        for r in rows:
            w.writerow({k: fmt(r.get(k)) for k in keys})


# ---------------------------------------------------------------- main
def refutations(lines, E, self_test):
    out = []
    bad_cls = [r for r in lines.rows if r["what"] == "class" and r["status"] == "fail"]
    if bad_cls:
        out.append(f"{len(bad_cls)} runs whose class is not the mirror's")
    if not self_test:
        off = [r for r in lines.rows if r["what"] == "end D-hat" and r["status"] == "fail"]
        if off:
            out.append(f"{len(off)} CONVERGED runs ending off their oracle point (gap above 1e-12)")
    kicks = [r for r in lines.rows if r["what"] == "kick set passes" and r["status"] == "fail"]
    if kicks:
        out.append(f"{len(kicks)} scored kick sets not as registered")
    ver = [r for r in lines.rows if r["run"] == "verdict" and r["status"] == "fail"]
    if ver:
        out.append(f"{len(ver)} map cells whose verdict is not the registered one")
    return out


def main(runs_path, out_dir, self_test=False):
    with open(runs_path) as f:
        recs = [json.loads(line) for line in f if line.strip()]
    R = registered()
    lines, E, verdicts = score(recs, R, self_test)
    tables(out_dir, lines, R, E, verdicts)
    st = defaultdict(int)
    for r in lines.rows:
        st[r["status"]] += 1
    by_part = defaultdict(lambda: defaultdict(int))
    for r in lines.rows:
        by_part[(r["part"], r["set"])][r["status"]] += 1
    print(f"{len(recs)} records; {len(lines.rows)} lines: " +
          ", ".join(f"{k} {v}" for k, v in sorted(st.items())))
    for k in sorted(by_part):
        print(f"  {k[0]} {k[1]}: " + ", ".join(f"{s} {n}" for s, n in sorted(by_part[k].items())))
    for cell, vr, ve in verdicts:
        print(f"  I3 map cell {cell}: registered {vr}, engine {ve}")
    ref = refutations(lines, E, self_test)
    print("refutations: " + (str(ref) if ref else "none"))
    miss = [r for r in lines.rows if r["status"] == "missing"]
    if self_test:
        other = [r for r in miss if r["what"] != "end D-hat"]
        print(f"self-test: {st['fail']} lines fail; {len(miss)} missing, of which "
              f"{len(miss) - len(other)} are end-of-run lines only the engine's run to L gives "
              f"and {len(other)} others")
        return st["fail"] == 0 and not other and not ref
    return st["fail"] == 0 and st["missing"] == 0 and not ref


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    ok = main(args[0], args[1], "--self-test" in sys.argv)
    sys.exit(0 if ok else 1)
