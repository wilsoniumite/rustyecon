"""P2.4 (label run, 2026-09-30): the scorer of the trap's remedy's wave, committed before the wave's
first job (decision 311). It scores runs.jsonl (gather.py) against the registered predictions in
docs/probe/trap/registered/ (SPEC.md §8-§9; registration.md, whose §3 readings it takes), line by
line, and writes lines.csv (every line), fails.csv, the tables and a printout with the tallies,
the verdicts and the refutation criteria. The bands are the commons frame's §5.6, with the runaway
ticks on the harness's reference (O107, the commons' A3) and every end value against the oracle's
(O108), as SPEC §8 says:

- every run's class exactly, the mirror's, in every set (VACUOUS included);
- ticks to tolerance within 10% where both runs are CONVERGED, with up to three runs a group
  charged within 25% (a group: an instance's step, E3 to E9; one dial setting of an instance, E10
  and the C1 controls; one history run's windows);
- the lowest baskets eaten over Y* and each market's lowest cleared volume over the oracle's
  within 0.05 absolute where the mirror's is above 0.1 (reported below it);
- dead ticks within 10% or 5;
- the ticks in each regime (Commons, Crowded with Split, Enclosed) and the regime switches within
  10% or 2, the mirror's end regime carried to L where it stopped early (p23-wave's reading);
- the regime at the end the mirror's, exactly; r_o/r at the end within 1e-9 relative of the
  oracle's at a Crowded target (O108);
- a DIVERGED run's runaway tick within 5%, on the harness's reference (the mirror's
  `err_harness_ref` where the registered record has it, else its `err`);
- every CONVERGED engine run's last D-hat at most 1e-9 (a gap of 1e-12 in log), and at a paced
  instance its paced share within 1e-12 of the rule's at the end (|ln(part_workers/part_target)|
  on the CSV's last row);
- at a paced instance, the ticks with no hours offered (`pace.zero_hours`) the mirror's
  `hours_zero` exactly;
- the history window by window: in tolerance at the window's end as the mirror's, ticks to
  tolerance (10%, three windows a run within 25%), dead ticks (10% or 5), no runaway;
- mode A at L PASS with its largest gap below 1e-9 at 52 a year (12 and 365 reported); L from the
  engine's elasticity probe the registered L at 52 a year; the kick sets at the base and at the 12
  cost targets passing as the mirror's largest root predicts (PL^(0.9 L) <= 1e-3; every one
  passes), C1PN's and the 12 and 365 a year base kicks reported.

The verdict per instance (SPEC §8): GO where mode A passes and every non-vacuous run of Tiers 1-3
and 3S is CONVERGED, with Tiers 3 and 3S CONVERGED again at 10 L (and every target's kick set
decays, the commons frame's §5.1); with margin where, in addition, every Tier-3 run of E10 is
CONVERGED or VACUOUS. The refutation criteria are SPEC §8's (`refutations`).

Usage (WSL): python3 score_trap.py RUNS.jsonl OUT_DIR [--self-test]"""
import json
import math
import os
import re
import sys
from collections import OrderedDict, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import (REPO, SETTINGS, Lines, fname, fmt, kick_bar, med, ok_record, printout,  # noqa: E402
                    read_recs, st_abs, st_count, st_rel, st_ticks, stat, why_missing, write_lines,
                    write_rows)

REG = os.path.join(REPO, "docs", "probe", "trap", "registered")
MARKETS = ["labour", "land", "manufactures", "food", "care", "shelter", "mach"]
DESKS = ["manufactures", "food", "care", "shelter", "mach"]
L52 = {"c1p": 141000, "c2p": 142000, "c1pn": 141000, "c1": 141000}
MIRROR = {"c1p": "C1/P1.3r", "c2p": "C2/P1.3r"}
BASE = {"c1p": (("land.mach", 0.4), ("b.food", 0.6), ("commons", 24.3)),
        "c2p": (("land.mach", 0.4), ("b.food", 0.6), ("commons", 31.2))}
TARGET_VALUES = {
    "c1p": {"land.mach": ("0.44", "0.36", "0.8", "0.2"), "b.food": ("0.66", "0.54", "1.2", "0.3"),
            "commons": ("26.73", "21.87", "48.6", "12.15")},
    "c2p": {"land.mach": ("0.44", "0.36", "0.8", "0.2"), "b.food": ("0.66", "0.54", "1.2", "0.3"),
            "commons": ("34.32", "28.08", "62.4", "15.6")}}
FACTORS = ("x1.1", "x0.9", "x2", "x0.5")
# the registered set -> (the engine's set directory, the step)
SETS = {"battery": ("L", "E3"), "tier3x10": ("10L", "E3"), "tier3s": ("tier3s", "E3"),
        "tier3s10": ("tier3s10L", "E3"), "stocks": ("stocks", "E4"), "pace": ("pace", "E4"),
        "joint2": ("joint2", "E5"), "joint4": ("joint4", "E5"), "basin": ("basin", "E5"),
        "history": ("history", "E6"), "hold": ("hold", "E7"), "tilt1": ("tilt1", "E7"),
        "tpy12": ("tpy12", "E8"), "tpy365": ("tpy365", "E8"), "enclose": ("enclose", "E9"),
        "negctl": ("L", "E9")}
PACED = {"c1p", "c2p", "c1pn"}
# the lines only an engine run to L gives (the mirror stops a run early): the self-test reads them
# as missing
END_LINES = ("end D-hat", "paced share at the end, |ln(F/F*)|", "r_o/r at the end (Crowded target)")


def eng_name(r):
    """The mirror's run name as the engine names it (registration §3)."""
    if r.get("engine_name"):
        return r["engine_name"]
    n = r["run"]
    if n == "cycle(land.mach)":
        return "cycle(land.mach,1500,80)"
    if n == "cycle(exit.To)":
        return "cycle(commons,1500,80)"
    n = n.replace("exit.To=", "commons=")
    for d in DESKS:
        n = n.replace(f"coin.{d}*", f"coin.desk.{d}*")
    return n


def runaway_of(err):
    m = re.search(r"runaway at t=(\d+)", err or "")
    return int(m.group(1)) if m else None


def registered():
    """(inst, engine set, setting) -> OrderedDict(engine name -> (step, mirror record))."""
    R = defaultdict(OrderedDict)
    for fn in ("runs.jsonl", "controls.jsonl"):
        for line in open(os.path.join(REG, fn)):
            r = json.loads(line)
            inst = r["inst"].lower()
            s = r["set"]
            setting = None
            if s.startswith("dials:"):
                k, v = s[len("dials:"):].split("=")
                es, step, setting = "dial", "E10", f"{k}.*={v}"
            elif s.startswith("ctl-dials:"):
                k, v = s[len("ctl-dials:"):].split("=")
                es, step, setting = "ctl-dial", "E9", f"{k}.*={v}"
            elif s == "ctl-tilt2":
                es, step, setting = "ctl-tilt2", "E9", "tilt.*=2"
            else:
                es, step = SETS[s]
            R[(inst, es, setting)][eng_name(r)] = (step, r)
    return R


def engine_index(recs):
    E = defaultdict(dict)
    for e in recs:
        if e.get("key") != "trap":
            continue
        setting = e.get("setting") if e.get("set") in ("dial", "ctl-dial", "ctl-tilt2") else None
        name = (e.get("names") or ["-"])[0]
        if e.get("cmd") == "kick":
            E[(e["inst"], "kick", None)][os.path.basename(e["rel"])] = e
        elif e.get("cmd") in ("elasticity", "point"):
            E[(e["inst"], "info", None)][os.path.basename(e["rel"])] = e
        elif e.get("set") == "modea":
            E[(e["inst"], "modea", None)][e.get("tpy")] = e
        else:
            E[(e["inst"], e["set"], setting)][name] = e
    return E


def regime_ticks(e):
    g = lambda k: stat(e, f"commons.regime_ticks|{k}")
    cr = g("Crowded")
    sp = g("Split")
    return {"Commons": g("Commons"), "Crowded": None if cr is None else cr + (sp or 0),
            "Enclosed": g("Enclosed")}


def run_lines(out, step, inst, set_, group, run, m, e, self_test):
    """The lines of one run: m the mirror's record, e the engine's (or None)."""
    if not ok_record(e):
        out.add(step, inst, set_, group, run, "class", m["cls"], why_missing(e), "exactly", "missing")
        return
    ec, mc = e.get("class"), m["cls"]
    out.add(step, inst, set_, group, run, "class", mc, ec, "exactly", "pass" if ec == mc else "fail")
    if inst in PACED:
        out.add(step, inst, set_, group, run, "ticks with no hours offered", m.get("hours_zero"),
                stat(e, "pace.zero_hours|-"), "exactly",
                "pass" if m.get("hours_zero") == stat(e, "pace.zero_hours|-") else "fail")
    if mc == ec == "DIVERGED":
        rw = runaway_of(m.get("err_harness_ref") or m.get("err"))
        out.add(step, inst, set_, group, run, "runaway tick", rw, e.get("runaway_tick"),
                "5% (the harness's reference)", st_rel(rw, e.get("runaway_tick"), 0.05))
    if mc == ec == "CONVERGED":
        out.add(step, inst, set_, group, run, "ticks to tolerance", m.get("ttol"),
                e.get("in_tol_from"), "10%, three runs a group within 25%",
                st_ticks(m.get("ttol"), e.get("in_tol_from")))
        out.add(step, inst, set_, group, run, "lowest baskets over Y*", m.get("baskets_trough"),
                e.get("low_baskets"), "0.05 where the mirror's is above 0.1",
                st_abs(m.get("baskets_trough"), e.get("low_baskets")))
        for mk in MARKETS:
            mt = (m.get("trough") or {}).get(mk)
            out.add(step, inst, set_, group, run, f"lowest cleared over the oracle's: {mk}", mt,
                    stat(e, f"trough.cleared|{mk}"), "0.05 where the mirror's is above 0.1",
                    st_abs(mt, stat(e, f"trough.cleared|{mk}")))
        out.add(step, inst, set_, group, run, "dead ticks", m.get("dead"), e.get("dead"), "10% or 5",
                st_count(m.get("dead"), e.get("dead")))
        rt = regime_ticks(e)
        Le = e.get("ticks")
        for lab in ("Commons", "Crowded", "Enclosed"):
            mt = (m.get("regime_ticks") or {}).get(lab, 0)
            if m.get("stopped_early") and lab == m.get("regime_end") and Le is not None:
                mt += Le - m["ticks_run"]
            if mt or rt[lab]:
                out.add(step, inst, set_, group, run, f"ticks in {lab}", mt, rt[lab],
                        "10% or 2 (the mirror's end regime carried to L)", st_count(mt, rt[lab], 0.1, 2))
        out.add(step, inst, set_, group, run, "regime switches", m.get("regime_switches"),
                stat(e, "commons.switches|-"), "10% or 2",
                st_count(m.get("regime_switches"), stat(e, "commons.switches|-"), 0.1, 2))
        re_ = stat(e, "commons.regime_end|-")
        out.add(step, inst, set_, group, run, "regime at the end", m.get("regime_end"), re_,
                "exactly", "pass" if re_ == m.get("regime_end") else "fail")
        if m.get("regime_star") == "Crowded":
            ro = (e.get("end") or {}).get("ro_over_r")
            out.add(step, inst, set_, group, run, "r_o/r at the end (Crowded target)", m.get("ro_star"),
                    ro, "1e-9 relative, against the oracle's (O108)",
                    "missing" if self_test else st_rel(m.get("ro_star"), ro, 1e-9))
        last = e.get("last")
        if self_test:
            out.add(step, inst, set_, group, run, "end D-hat", 1e-9, last,
                    "at most 1e-9 (a gap of 1e-12 in log)", "missing")
        else:
            out.add(step, inst, set_, group, run, "end D-hat", 1e-9, last,
                    "at most 1e-9 (a gap of 1e-12 in log)",
                    "pass" if last is not None and last <= 1e-9 else "fail")
        if inst in PACED:
            end = e.get("end") or {}
            f, fs = end.get("part_workers"), end.get("part_target")
            gap = abs(math.log(f / fs)) if f and fs else (0.0 if f == fs else math.inf)
            if self_test:
                out.add(step, inst, set_, group, run, "paced share at the end, |ln(F/F*)|", 1e-12,
                        None, "at most 1e-12", "missing")
            else:
                out.add(step, inst, set_, group, run, "paced share at the end, |ln(F/F*)|", 1e-12, gap,
                        "at most 1e-12", "pass" if gap <= 1e-12 else "fail")
        for k, ek, what in (("peak", e.get("peak_dhat"), "peak D-hat"),
                            ("transfer_ticks", e.get("transfer_short_ticks"), "ticks the transfer fell short"),
                            ("provider_coin_low", stat(e, "commons.provider_coin_low|-"),
                             "provider's lowest coin over genesis"),
                            ("tp_high", stat(e, "commons.tp_max|-"), "largest T_p"),
                            ("baskets_none", e.get("no_basket_ticks"), "ticks with no baskets")):
            out.add(step, inst, set_, group, run, what, m.get(k), ek, "reported", "reported")


def history_lines(out, inst, run, m, e, self_test):
    group = f"E6 {run}"
    wins = (e or {}).get("windows") or []
    ttols, deads = m.get("ttols") or [], m.get("deads") or []
    for w in range(m.get("windows") or 0):
        ew = next((x for x in wins if x["w"] == w), None)
        reg_in = ttols[w] is not None
        if ew is None:
            out.add("E6", inst, "history", group, f"{run} window {w}", "in tolerance at its end",
                    reg_in, None, "exactly", "missing")
            continue
        out.add("E6", inst, "history", group, f"{run} window {w}", "in tolerance at its end", reg_in,
                ew["in_tol_at_end"], "exactly", "pass" if reg_in == ew["in_tol_at_end"] else "fail")
        out.add("E6", inst, "history", group, f"{run} window {w}", "ticks to tolerance", ttols[w],
                ew["ttol"], "10%, three windows a run within 25%", st_ticks(ttols[w], ew["ttol"]))
        out.add("E6", inst, "history", group, f"{run} window {w}", "dead ticks", deads[w], ew["dead"],
                "10% or 5", st_count(deads[w], ew["dead"]))
        reg_regime = (m.get("regimes") or [None] * (w + 1))[w]
        out.add("E6", inst, "history", group, f"{run} window {w}", "regime at its end (the rule's)",
                reg_regime, ew.get("regime"), "reported", "reported")
    if e is not None:
        out.add("E6", inst, "history", group, run, "runaway", None, e.get("runaway_window"),
                "none, as registered", "pass" if e.get("runaway_window") is None else "fail")


def score(recs, self_test):
    out = Lines()
    R = registered()
    E = engine_index(recs)
    per_run = []
    for (inst, es, setting), rows in R.items():
        eng = E.get((inst, es, setting), {})
        for run, (step, m) in rows.items():
            e = eng.get(run)
            group = f"{step} {setting}" if setting else step
            if es == "history":
                history_lines(out, inst, run, m, e, self_test)
                continue
            run_lines(out, step, inst, es, group, run, m, e, self_test)
            per_run.append(OrderedDict(
                step=step, inst=inst, set=es, setting=setting, run=run, tier=m.get("tier"),
                slack=m.get("slack"), cls_mirror=m["cls"], cls_engine=(e or {}).get("class"),
                ttol_mirror=m.get("ttol"), ttol_engine=(e or {}).get("in_tol_from"),
                runaway_mirror=runaway_of(m.get("err_harness_ref") or m.get("err")),
                runaway_engine=(e or {}).get("runaway_tick"), dead_mirror=m.get("dead"),
                dead_engine=(e or {}).get("dead"), baskets_mirror=m.get("baskets_trough"),
                baskets_engine=(e or {}).get("low_baskets"), peak_mirror=m.get("peak"),
                peak_engine=(e or {}).get("peak_dhat"), hours_zero_mirror=m.get("hours_zero"),
                hours_zero_engine=stat(e, "pace.zero_hours|-"), pace_low_engine=stat(e, "pace.low|-"),
                regime_end_mirror=m.get("regime_end"), regime_end_engine=stat(e, "commons.regime_end|-"),
                end_dhat_engine=(e or {}).get("last")))
    # E2: mode A, L, the kick sets
    modea = json.load(open(os.path.join(REG, "modea.json")))
    elas = json.load(open(os.path.join(REG, "elasticity-P1.3r.json")))
    lin = json.load(open(os.path.join(REG, "lin-P1.3r.json")))
    mode_rows, kick_rows = [], []
    for ma in modea:
        inst, tpy = ma["inst"].lower(), int(ma["tpy"])
        e = E.get((inst, "modea", None), {}).get(tpy)
        ok = ok_record(e) and e.get("mode_a") == "PASS" and e["peak_dhat"] * 1e-3 < 1e-9
        scored = tpy == 52
        out.add("E2", inst, "modea", "E2", f"hold at {tpy} a year", "mode A at L, largest gap below 1e-9",
                ma["worst"], None if not ok_record(e) else e["peak_dhat"] * 1e-3, "PASS, below 1e-9",
                ("pass" if ok else ("missing" if not ok_record(e) else "fail")) if scored else "reported")
        mode_rows.append(OrderedDict(inst=inst, tpy=tpy, L=ma["L"], mirror_gap=ma["worst"],
                                     engine_gap=None if not ok_record(e) else e["peak_dhat"] * 1e-3,
                                     engine_mode_a=(e or {}).get("mode_a"),
                                     engine_pace_gap_max=stat(e, "pace.gap_max|-"),
                                     engine_pace_zero_hours=stat(e, "pace.zero_hours|-"),
                                     engine_regime_end=stat(e, "commons.regime_end|-")))
    for inst in ("c1p", "c2p", "c1pn"):
        for tpy in (52, 12, 365):
            key = f"{MIRROR.get(inst, 'C1/P1.3r')} {tpy}"
            Lr = elas[key]["L"] if inst in MIRROR else {52: 141000, 12: 26000, 365: 1113000}[tpy]
            el = E.get((inst, "info", None), {}).get(f"el{tpy}")
            scored = tpy == 52 and inst in MIRROR
            st = "pass" if ok_record(el) and el.get("L") == Lr else ("missing" if not ok_record(el) else "fail")
            out.add("E2", inst, "info", "E2", f"elasticity at {tpy} a year", "L from the engine's elasticity",
                    Lr, None if not ok_record(el) else el.get("L"), "equal", st if scored else "reported")
    kicks_ok = {}
    for inst in ("c1p", "c2p"):
        pl = {x["target"]: x["PL"] for x in lin if x["name"] == MIRROR[inst]}
        ks = E.get((inst, "kick", None), {})
        tgts = [("hold", "base")]
        for c, _ in BASE[inst]:
            for v, f in zip(TARGET_VALUES[inst][c], FACTORS):
                tgts.append((f"{c}={v}@dated", f"{'exit.To' if c == 'commons' else c} {f}"))
        allok = True
        for t, mt in tgts:
            reg = kick_bar(pl[mt], L52[inst])
            k = ks.get(fname(t))
            eng = None if not ok_record(k) else (bool(k.get("passed")) and k.get("gain_tail", math.inf) <= 1e-3)
            allok &= bool(eng)
            out.add("E3" if t != "hold" else "E2", inst, "kick", "kick", t, "kick set decays (gain_tail at most 1e-3)",
                    reg, eng, f"as the mirror's PL {pl[mt]:.6f} predicts (PL^(0.9 L) <= 1e-3)",
                    "missing" if eng is None else ("pass" if eng == reg else "fail"))
            kick_rows.append(OrderedDict(inst=inst, target=t, pl=pl[mt], predicted=reg,
                                         passed=(k or {}).get("passed"), gain_tail=(k or {}).get("gain_tail"),
                                         gain_peak=(k or {}).get("gain_peak"), kicks=(k or {}).get("kicks")))
        kicks_ok[inst] = allok
        for tpy in (12, 365):
            k = ks.get(f"hold-{tpy}")
            out.add("E2", inst, "kick", "kick", f"hold at {tpy} a year", "kick set decays", "reported",
                    None if not ok_record(k) else f"{'PASS' if k.get('passed') else 'FAIL'} {fmt(k.get('gain_tail'))}",
                    "reported", "reported")
            kick_rows.append(OrderedDict(inst=inst, target=f"hold at {tpy} a year", passed=(k or {}).get("passed"),
                                         gain_tail=(k or {}).get("gain_tail"), gain_peak=(k or {}).get("gain_peak"),
                                         kicks=(k or {}).get("kicks")))
    k = E.get(("c1pn", "kick", None), {}).get("hold")
    out.add("E2", "c1pn", "kick", "kick", "hold", "kick set decays", "reported",
            None if not ok_record(k) else f"{'PASS' if k.get('passed') else 'FAIL'} {fmt(k.get('gain_tail'))}",
            "reported", "reported")
    kick_rows.append(OrderedDict(inst="c1pn", target="hold", passed=(k or {}).get("passed"),
                                 gain_tail=(k or {}).get("gain_tail"), gain_peak=(k or {}).get("gain_peak"),
                                 kicks=(k or {}).get("kicks")))
    # counts: each set's classes, and the runs DIVERGED
    fam_rows, dial_rows = [], []
    for (inst, es, setting), rows in R.items():
        if es == "history":
            continue
        eng = E.get((inst, es, setting), {})
        step = next(iter(rows.values()))[0]
        mc, ec = defaultdict(int), defaultdict(int)
        for run, (_, m) in rows.items():
            mc[m["cls"]] += 1
            ec[(eng.get(run) or {}).get("class", "missing")] += 1
        group = f"{step} {setting}" if setting else step
        out.add(step, inst, es, group, "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())),
                "equal", "pass" if dict(mc) == dict(ec) else "fail")
        mdiv = sorted(r for r, (_, m) in rows.items() if m["cls"] == "DIVERGED")
        ediv = sorted(r for r in rows if (eng.get(r) or {}).get("class") == "DIVERGED")
        if mdiv or ediv:
            out.add(step, inst, es, group, "-", "the runs DIVERGED (the subsistence trap)", mdiv, ediv,
                    "equal", "pass" if mdiv == ediv else "fail")
        conv = [(m, eng.get(r)) for r, (_, m) in rows.items() if m["cls"] == "CONVERGED"]
        row = OrderedDict(step=step, inst=inst, set=es, setting=setting, runs=len(rows),
                          mirror=json.dumps(dict(sorted(mc.items()))), engine=json.dumps(dict(sorted(ec.items()))),
                          ttol_median_mirror=med([m.get("ttol") for m, _ in conv]),
                          ttol_median_engine=med([(e or {}).get("in_tol_from") for _, e in conv
                                                  if (e or {}).get("class") == "CONVERGED"]),
                          ttol_max_mirror=max([m.get("ttol") or 0 for m, _ in conv], default=None),
                          ttol_max_engine=max([(e or {}).get("in_tol_from") or 0 for _, e in conv
                                               if (e or {}).get("class") == "CONVERGED"], default=None),
                          dead_max_mirror=max([m.get("dead") or 0 for m, _ in conv], default=None),
                          dead_max_engine=max([(e or {}).get("dead") or 0 for _, e in conv], default=None),
                          zero_hours_engine=sum(int(stat(e, "pace.zero_hours|-") or 0) for e in
                                                [eng.get(r) for r in rows] if e) if inst in PACED else None,
                          diverged_mirror=" ".join(mdiv), diverged_engine=" ".join(ediv))
        (dial_rows if setting else fam_rows).append(row)
    # tiers, as SPEC §9.3 tabulates them
    tier_rows, counts = [], {}
    for inst in ("c1p", "c2p"):
        c = OrderedDict()
        for label, es, pick in (("Tier 1", "L", lambda m: m.get("tier") == 1),
                                ("Tier 2", "L", lambda m: m.get("tier") == 2),
                                ("Tier 3", "L", lambda m: m.get("tier") == 3),
                                ("Tier 3 at 10L", "10L", lambda m: True),
                                ("Tier 3S", "tier3s", lambda m: True),
                                ("Tier 3S at 10L", "tier3s10L", lambda m: True)):
            rows = R.get((inst, es, None), {})
            eng = E.get((inst, es, None), {})
            sel = [(r, m) for r, (_, m) in rows.items() if pick(m)]
            mc, ec = defaultdict(int), defaultdict(int)
            for r, m in sel:
                mc[m["cls"]] += 1
                ec[(eng.get(r) or {}).get("class", "missing")] += 1
            c[label] = (dict(ec), dict(mc))
            out.add("E3", inst, label, "E3", "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())),
                    "equal", "pass" if dict(mc) == dict(ec) else "fail")
            nons = [(m, eng.get(r)) for r, m in sel if not m.get("slack") and m["cls"] == "CONVERGED"]
            tier_rows.append(OrderedDict(
                inst=inst, tier=label, runs=len(sel), mirror=json.dumps(dict(sorted(mc.items()))),
                engine=json.dumps(dict(sorted(ec.items()))),
                ttol_median_mirror=med([m.get("ttol") for m, _ in nons]),
                ttol_median_engine=med([(e or {}).get("in_tol_from") for _, e in nons]),
                ttol_max_mirror=max([m.get("ttol") or 0 for m, _ in nons], default=None),
                ttol_max_engine=max([(e or {}).get("in_tol_from") or 0 for _, e in nons], default=None),
                peak_median_mirror=med([m.get("peak") for m, _ in nons]),
                peak_median_engine=med([(e or {}).get("peak_dhat") for _, e in nons]),
                dead_max_mirror=max([m.get("dead") or 0 for m, _ in nons], default=None),
                dead_max_engine=max([(e or {}).get("dead") or 0 for _, e in nons], default=None),
                baskets_low_mirror=min([m.get("baskets_trough") for m, _ in nons
                                        if m.get("baskets_trough") is not None], default=None),
                baskets_low_engine=min([(e or {}).get("low_baskets") for _, e in nons
                                        if (e or {}).get("low_baskets") is not None], default=None),
                no_basket_ticks_mirror=max([m.get("baskets_none") or 0 for m, _ in nons], default=None),
                no_basket_ticks_engine=max([(e or {}).get("no_basket_ticks") or 0 for _, e in nons], default=None)))
        counts[inst] = c
    # the verdicts
    verdicts = []
    for inst in ("c1p", "c2p"):
        ma = any(r["status"] == "pass" for r in out.rows if r["step"] == "E2" and r["inst"] == inst
                 and r["run"] == "hold at 52 a year" and r["what"].startswith("mode A"))
        c = counts[inst]
        conv_ok = all(set(e) <= {"CONVERGED", "VACUOUS"} and e.get("VACUOUS", 0) == m.get("VACUOUS", 0)
                      for e, m in c.values())
        t12 = all(set(c[t][0]) <= {"CONVERGED", "VACUOUS"} for t in ("Tier 1", "Tier 2"))
        verdict = "GO" if ma and conv_ok and kicks_ok[inst] else ("LOCAL" if ma and t12 else "NO-GO")
        margin = True
        for s in SETTINGS:
            eng = E.get((inst, "dial", s), {})
            rows = R.get((inst, "dial", s), {})
            if not rows or any((eng.get(r) or {}).get("class") not in ("CONVERGED", "VACUOUS") for r in rows):
                margin = False
        full = verdict + (" with margin" if verdict == "GO" and margin else "")
        out.add("E3", inst, "verdict", "verdict", "-", f"{inst.upper()}'s verdict", "GO with margin", full,
                "SPEC §8: mode A; Tiers 1-3 and 3S CONVERGED at L, 3 and 3S at 10 L; kick sets; margin: E10",
                "pass" if full == "GO with margin" else "fail")
        verdicts.append(OrderedDict(inst=inst, verdict=full, prediction="GO with margin",
                                    **{k: json.dumps(v[0], sort_keys=True) for k, v in c.items()},
                                    kicks="13/13 PASS" if kicks_ok[inst] else "not all PASS",
                                    e10="every Tier-3 run CONVERGED or VACUOUS" if margin else "not all"))
    out.charge()
    return out, R, E, dict(per_run=per_run, fam=fam_rows, dial=dial_rows, tiers=tier_rows,
                           verdicts=verdicts, modea=mode_rows, kicks=kick_rows)


def refutations(out, R, E, self_test):
    """SPEC §8's criteria, each sending the remedy back to the scan with home output sold."""
    res = []
    bad = [r for r in out.rows if r["what"] == "class" and r["status"] == "fail"
           and r["inst"] in ("c1p", "c2p") and r["step"] in ("E3", "E4", "E10")]
    if bad:
        res.append(f"{len(bad)} class changes in E3, E4 or E10 at C1P or C2P")
    if not self_test:
        off = [r for r in out.rows if r["what"] in ("end D-hat", "paced share at the end, |ln(F/F*)|")
               and r["status"] == "fail"]
        if off:
            res.append(f"{len(off)} CONVERGED runs resting off the oracle's point or with the paced share "
                       f"more than 1e-12 from the rule's")
    trap = []
    for (inst, es, setting), rows in R.items():
        if inst not in ("c1p", "c2p") or es in ("ctl-tilt2",) or rows and next(iter(rows.values()))[0] == "E9":
            continue
        eng = E.get((inst, es, setting), {})
        for r in rows:
            e = eng.get(r) or {}
            if e.get("class") == "DIVERGED" or (stat(e, "pace.zero_hours|-") or 0) > 0:
                trap.append(f"{inst} {es} {setting or ''} {r}")
    if trap:
        res.append(f"{len(trap)} C1P or C2P runs of E3-E8 or E10 in the trap or with a tick of no hours")
    pred, fell = 0, 0
    for (inst, es, setting), rows in R.items():
        if es not in ("ctl-dial", "ctl-tilt2"):
            continue
        eng = E.get((inst, es, setting), {})
        for r, (_, m) in rows.items():
            if m["cls"] == "DIVERGED":
                pred += 1
                fell += (eng.get(r) or {}).get("class") == "DIVERGED"
    out.add("E9", "controls", "controls", "E9", "-", "the controls' predicted trap runs falling into it",
            pred, fell, "at least one (SPEC §8: none falling would refute)", "pass" if fell > 0 else "fail")
    if pred and fell == 0:
        res.append("none of the controls' predicted trap runs fell into it")
    return res


def main(runs_path, out_dir, self_test=False):
    recs = read_recs(runs_path)
    out, R, E, T = score(recs, self_test)
    ref = refutations(out, R, E, self_test)
    write_lines(out_dir, out)
    for k, fn in (("per_run", "runs.csv"), ("fam", "families.csv"), ("dial", "dial.csv"),
                  ("tiers", "tiers.csv"), ("verdicts", "verdicts.csv"), ("modea", "modea.csv"),
                  ("kicks", "kicks.csv")):
        write_rows(os.path.join(out_dir, fn), T[k])
    st = printout(out, f"trap: {len(recs)} records")
    for v in T["verdicts"]:
        print(f"  verdict {v['inst']}: {v['verdict']} (registered {v['prediction']})")
    print("refutations: " + (str(ref) if ref else "none"))
    miss = [r for r in out.rows if r["status"] == "missing"]
    if self_test:
        other = [r for r in miss if r["what"] not in END_LINES]
        print(f"self-test: {st['fail']} lines fail; {len(miss)} missing, of which {len(miss) - len(other)} "
              f"are end-of-run lines only the engine's run to L gives and {len(other)} others")
        return st["fail"] == 0 and not other and not ref
    return st["fail"] == 0 and st["missing"] == 0 and not ref


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    sys.exit(0 if main(args[0], args[1], "--self-test" in sys.argv) else 1)
