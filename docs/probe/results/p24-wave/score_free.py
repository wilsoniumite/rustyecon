"""P2.4 (label run, 2026-09-30): the scorer of the free step's wave, committed before the wave's
first job (decision 311). It scores runs.jsonl (gather.py) against the registered predictions in
docs/probe/free/registered/ (SPEC.md §9; registration.md with its amendment A1, which bears on E0
only; decision 424), line by line, and writes lines.csv (every line), fails.csv, the tables and a
printout with the tallies, the verdicts and the refutation criteria. The bands are SPEC §9.3's:

- every run's class exactly;
- ticks to tolerance within 10%, with up to three runs an instance within 25% (a group: an
  instance's step; one dial setting of an instance in E4);
- the lowest baskets over Y* and each market's lowest cleared volume within 0.05 absolute where
  the mirror's is above 0.1 (reported below it);
- dead ticks within 10% or 5;
- the free-able market (IL1's land, CT2's commons): its end state exactly (0, or positive), its
  first free tick within 2 ticks, and its free/priced switches within 10% or 2; the regime at the
  end the mirror's, exactly;
- a DIVERGED run's runaway tick within 5%, against the harness's reference (decision 399 as
  amended), as the mirror's `err`;
- every CONVERGED engine run's last D-hat at most 1e-9 (a gap of 1e-12 in log);
- mode A at 12, 52 and 365 a year: PASS with its largest gap below 1e-9, the free-able market free
  on every tick; L from the engine's elasticity probe the registered L (IL1 at 12 a year reported:
  decision 424 keeps 13,000 where the probe prints its floor, 20,000);
- the kick sets at the base and the 12 cost targets passing as the mirror's largest root predicts
  (PL^(0.9 L) <= 1e-3: every one).

The verdict per instance (SPEC §9.1): GO where mode A passes, every non-vacuous run of Tiers 1-3
and 3S is CONVERGED, Tiers 3 and 3S CONVERGED again at 10 L, every target's kick set decays, and
every CONVERGED run of them ends with its free-able market at the oracle's price (0 where the
oracle's is 0, positive where it is positive); LOCAL where Tiers 1-2 are CONVERGED; NO-GO
otherwise. The dial family (E4) and the families (E5) are scored line by line and reported beside
the verdict. The refutation criteria are SPEC §10's (`refutations`).

Usage (WSL): python3 score_free.py RUNS.jsonl OUT_DIR [--self-test]"""
import json
import math
import os
import re
import sys
from collections import OrderedDict, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import (REPO, Lines, fname, kick_bar, med, ok_record, printout, read_recs, st_abs,  # noqa: E402
                    st_count, st_rel, st_ticks, stat, why_missing, write_lines, write_rows)

REG = os.path.join(REPO, "docs", "probe", "free", "registered")
MARKETS = ["labour", "land", "manufactures", "food", "care", "shelter", "mach"]
DESKS = ["manufactures", "food", "care", "shelter", "mach"]
FREE = {"il1": "land", "ct2": "commons"}
LT = {"il1": {52: 56000, 12: 13000, 365: 390000}, "ct2": {52: 141000, 12: 26000, 365: 1114000}}
NAME = {"il1": "IL1/zp05", "ct2": "CT2/zp05"}
COEF = {"il1": (("land.mach", ("0.44", "0.36", "0.8", "0.2")), ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                ("inst.land", ("572", "468", "1040", "260"))),
        "ct2": (("land.mach", ("0.44", "0.36", "0.8", "0.2")), ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                ("exit.To", ("21.45", "17.55", "39", "9.75")))}
FACTORS = ("x1.1", "x0.9", "x2", "x0.5")
SETS = {"battery": ("L", "E3"), "tier3x10": ("10L", "E3"), "tier3s": ("tier3s", "E3"),
        "tier3sx10": ("tier3s10L", "E3"), "stocks": ("stocks", "E5"), "joint2": ("joint2", "E5"),
        "joint4": ("joint4", "E5"), "tpy12": ("tpy12", "E5"), "tpy365": ("tpy365", "E5")}
END_LINES = ("end D-hat",)


def eng_name(run):
    for d in DESKS:
        run = run.replace(f"coin.{d}*", f"coin.desk.{d}*")
    return run


def runaway_of(err):
    m = re.search(r"runaway at t=(\d+)", err or "")
    return int(m.group(1)) if m else None


def registered():
    R = defaultdict(OrderedDict)
    modes = {}
    for fn in ("scanF.jsonl", "regH.jsonl"):
        for line in open(os.path.join(REG, fn)):
            r = json.loads(line)
            inst = r["name"].split("/")[0].lower()
            s = r["set"]
            if s.startswith("modeA"):
                modes[(inst, {"modeA": 52, "modeA12": 12, "modeA365": 365}[s])] = r
                continue
            setting = None
            if s.startswith("dial:"):
                k, f = s[len("dial:"):].split(" ")
                setting = f"{k}.*={f.lstrip('x')}"
                es, step = "dial", "E4"
            else:
                es, step = SETS[s]
            R[(inst, es, setting)][eng_name(r["run"])] = (step, r)
    return R, modes


def engine_index(recs):
    E = defaultdict(dict)
    for e in recs:
        if e.get("key") != "free":
            continue
        name = (e.get("names") or ["-"])[0]
        if e.get("cmd") == "kick":
            E[(e["inst"], "kick", None)][os.path.basename(e["rel"])] = e
        elif e.get("cmd") in ("elasticity", "point"):
            E[(e["inst"], "info", None)][os.path.basename(e["rel"])] = e
        elif e.get("set") == "modea":
            E[(e["inst"], "modea", None)][e.get("tpy")] = e
        else:
            setting = e.get("setting") if e.get("set") == "dial" else None
            E[(e["inst"], e["set"], setting)][name] = e
    return E


def run_lines(out, step, inst, set_, group, run, m, e, self_test):
    if not ok_record(e):
        out.add(step, inst, set_, group, run, "class", m["cls"], why_missing(e), "exactly", "missing")
        return
    ec, mc = e.get("class"), m["cls"]
    out.add(step, inst, set_, group, run, "class", mc, ec, "exactly", "pass" if ec == mc else "fail")
    if mc == ec == "DIVERGED":
        rw = runaway_of(m.get("err"))
        out.add(step, inst, set_, group, run, "runaway tick", rw, e.get("runaway_tick"),
                "5% (the harness's reference)", st_rel(rw, e.get("runaway_tick"), 0.05))
    if not (mc == ec == "CONVERGED"):
        return
    out.add(step, inst, set_, group, run, "ticks to tolerance", m.get("ttol"), e.get("in_tol_from"),
            "10%, three runs a group within 25%", st_ticks(m.get("ttol"), e.get("in_tol_from")))
    out.add(step, inst, set_, group, run, "lowest baskets over Y*", m.get("baskets_trough"), e.get("low_baskets"),
            "0.05 where the mirror's is above 0.1", st_abs(m.get("baskets_trough"), e.get("low_baskets")))
    for mk in MARKETS:
        mt = (m.get("trough") or {}).get(mk)
        et = stat(e, f"trough.cleared|{mk}")
        out.add(step, inst, set_, group, run, f"lowest cleared over the oracle's: {mk}", mt, et,
                "0.05 where the mirror's is above 0.1", st_abs(mt, et))
    out.add(step, inst, set_, group, run, "dead ticks", m.get("dead"), e.get("dead"), "10% or 5",
            st_count(m.get("dead"), e.get("dead")))
    g = FREE[inst]
    mend = (m.get("free_end") or {}).get(g)
    eend = stat(e, f"free.end_over_ref|{g}")
    ms = None if mend is None else ("0" if mend == 0.0 else "positive")
    es_ = None if eend is None else ("0" if eend == 0.0 else "positive")
    out.add(step, inst, set_, group, run, f"the free-able market's end state: {g}", ms, es_, "exactly",
            "pass" if ms == es_ else "fail")
    mf = (m.get("free_first") or {}).get(g)
    ef = stat(e, f"free.first_zero|{g}")
    ef = None if ef is None else int(ef)
    st = ("pass" if ef is None else "fail") if mf is None else (
        "fail" if ef is None else ("pass" if abs(ef - mf) <= 2 else "fail"))
    out.add(step, inst, set_, group, run, f"its first free tick: {g}", mf, ef, "within 2 ticks", st)
    msw = (m.get("free_switches") or {}).get(g)
    esw = stat(e, f"free.switches|{g}")
    out.add(step, inst, set_, group, run, f"its free/priced switches: {g}", msw, esw, "10% or 2",
            st_count(msw, esw, 0.1, 2))
    re_ = stat(e, "free.regime_end|-")
    out.add(step, inst, set_, group, run, "the regime at the end", m.get("regime_end"), re_, "exactly",
            "pass" if re_ == m.get("regime_end") else "fail")
    last = e.get("last")
    out.add(step, inst, set_, group, run, "end D-hat", 1e-9, last, "at most 1e-9 (a gap of 1e-12 in log)",
            "missing" if self_test else ("pass" if last is not None and last <= 1e-9 else "fail"))
    for k, ek, what in ((m.get("peak"), e.get("peak_dhat"), "peak D-hat"),
                        ((m.get("free_ticks") or {}).get(g), stat(e, f"free.zero_ticks|{g}"),
                         "ticks the free-able market posts 0 (the mirror's to its early stop)"),
                        (m.get("regime_switches"), stat(e, "free.regime_switches|-"), "regime switches")):
        out.add(step, inst, set_, group, run, what, k, ek, "reported", "reported")


def score(recs, self_test):
    out = Lines()
    R, modes = registered()
    E = engine_index(recs)
    per_run = []
    for (inst, es, setting), rows in R.items():
        eng = E.get((inst, es, setting), {})
        for run, (step, m) in rows.items():
            e = eng.get(run)
            group = f"{step} {setting}" if setting else step
            run_lines(out, step, inst, es, group, run, m, e, self_test)
            g = FREE[inst]
            per_run.append(OrderedDict(
                step=step, inst=inst, set=es, setting=setting, run=run, tier=m.get("tier"),
                cls_mirror=m["cls"], cls_engine=(e or {}).get("class"), ttol_mirror=m.get("ttol"),
                ttol_engine=(e or {}).get("in_tol_from"), runaway_mirror=runaway_of(m.get("err")),
                runaway_engine=(e or {}).get("runaway_tick"), dead_mirror=m.get("dead"),
                dead_engine=(e or {}).get("dead"), baskets_mirror=m.get("baskets_trough"),
                baskets_engine=(e or {}).get("low_baskets"),
                free_end_mirror=(m.get("free_end") or {}).get(g), free_end_engine=stat(e, f"free.end_over_ref|{g}"),
                free_first_mirror=(m.get("free_first") or {}).get(g),
                free_first_engine=stat(e, f"free.first_zero|{g}"),
                free_switches_mirror=(m.get("free_switches") or {}).get(g),
                free_switches_engine=stat(e, f"free.switches|{g}"),
                regime_end_mirror=m.get("regime_end"), regime_end_engine=stat(e, "free.regime_end|-"),
                end_dhat_engine=(e or {}).get("last")))
    # E2: mode A and L
    elas = json.load(open(os.path.join(REG, "elasticity_free.json")))
    mode_rows = []
    for inst in ("il1", "ct2"):
        g = FREE[inst]
        for tpy in (52, 12, 365):
            m = modes[(inst, tpy)]
            e = E.get((inst, "modea", None), {}).get(tpy)
            ok = ok_record(e) and e.get("mode_a") == "PASS" and e["peak_dhat"] * 1e-3 < 1e-9
            out.add("E2", inst, "modea", "E2", f"hold at {tpy} a year", "mode A at L, largest gap below 1e-9",
                    m["peak"] * 1e-3, None if not ok_record(e) else e["peak_dhat"] * 1e-3, "PASS, below 1e-9",
                    "pass" if ok else ("missing" if not ok_record(e) else "fail"))
            zt = stat(e, f"free.zero_ticks|{g}")
            out.add("E2", inst, "modea", "E2", f"hold at {tpy} a year", f"the free-able market free on every tick: {g}",
                    LT[inst][tpy], zt, "every tick",
                    "missing" if not ok_record(e) else ("pass" if zt == LT[inst][tpy] else "fail"))
            el = E.get((inst, "info", None), {}).get(f"el{tpy}")
            Lr = elas[f"{NAME[inst].split('/')[0]}/zp01 {tpy}"]["L"]
            st = "pass" if ok_record(el) and el.get("L") == Lr else ("missing" if not ok_record(el) else "fail")
            out.add("E2", inst, "info", "E2", f"elasticity at {tpy} a year", "L from the engine's elasticity", Lr,
                    None if not ok_record(el) else el.get("L"),
                    "equal" if (inst, tpy) != ("il1", 12) else "reported (decision 424: the probe prints its floor)",
                    st if (inst, tpy) != ("il1", 12) else "reported")
            mode_rows.append(OrderedDict(inst=inst, tpy=tpy, L=LT[inst][tpy], mirror_peak=m["peak"],
                                         engine_peak=(e or {}).get("peak_dhat"), engine_mode_a=(e or {}).get("mode_a"),
                                         engine_zero_ticks=zt, engine_L=(el or {}).get("L")))
    # E3: the kick sets
    lin = json.load(open(os.path.join(REG, "lin_all05.json")))
    kick_rows, kicks_ok = [], {}
    for inst in ("il1", "ct2"):
        pl = {x["target"]: x["PL"] for x in lin if x["name"] == NAME[inst] and x.get("dials", "C2m") == "C2m"}
        tg = [("hold", "base")] + [(f"{c}={v}@dated", f"{c} {f}") for c, vs in COEF[inst] for v, f in zip(vs, FACTORS)]
        allok = True
        for t, mt in tg:
            reg = kick_bar(pl[mt], LT[inst][52])
            k = E.get((inst, "kick", None), {}).get(fname(t))
            eng = None if not ok_record(k) else (bool(k.get("passed")) and k.get("gain_tail", math.inf) <= 1e-3)
            allok &= bool(eng)
            out.add("E3", inst, "kick", "kick", t, "kick set decays (gain_tail at most 1e-3)", reg, eng,
                    f"as the mirror's PL {pl[mt]:.6f} predicts (PL^(0.9 L) <= 1e-3)",
                    "missing" if eng is None else ("pass" if eng == reg else "fail"))
            kick_rows.append(OrderedDict(inst=inst, target=t, pl=pl[mt], predicted=reg, passed=(k or {}).get("passed"),
                                         gain_tail=(k or {}).get("gain_tail"), gain_peak=(k or {}).get("gain_peak"),
                                         kicks=(k or {}).get("kicks")))
        kicks_ok[inst] = allok
    # counts per set
    fam_rows, dial_rows = [], []
    for (inst, es, setting), rows in R.items():
        eng = E.get((inst, es, setting), {})
        step = next(iter(rows.values()))[0]
        mc, ec = defaultdict(int), defaultdict(int)
        for run, (_, m) in rows.items():
            mc[m["cls"]] += 1
            ec[(eng.get(run) or {}).get("class", "missing")] += 1
        group = f"{step} {setting}" if setting else step
        out.add(step, inst, es, group, "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())), "equal",
                "pass" if dict(mc) == dict(ec) else "fail")
        mdiv = sorted(r for r, (_, m) in rows.items() if m["cls"] == "DIVERGED")
        ediv = sorted(r for r in rows if (eng.get(r) or {}).get("class") == "DIVERGED")
        if mdiv or ediv:
            out.add(step, inst, es, group, "-", "the runs DIVERGED (the subsistence trap)", mdiv, ediv, "equal",
                    "pass" if mdiv == ediv else "fail")
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
                          baskets_low_mirror=min([m.get("baskets_trough") for m, _ in conv
                                                  if m.get("baskets_trough") is not None], default=None),
                          baskets_low_engine=min([(e or {}).get("low_baskets") for _, e in conv
                                                  if (e or {}).get("low_baskets") is not None], default=None),
                          diverged_mirror=" ".join(mdiv), diverged_engine=" ".join(ediv))
        (dial_rows if setting else fam_rows).append(row)
    # the verdicts
    verdicts = []
    for inst in ("il1", "ct2"):
        # MARKETS-SPEC 7.9's mode A: at 52 a year (12 and 365 are scored as lines, and a failure
        # there is a refutation, SPEC §10)
        ma = all(r["status"] == "pass" for r in out.rows if r["step"] == "E2" and r["inst"] == inst
                 and r["set"] == "modea" and r["run"] == "hold at 52 a year")
        c = OrderedDict()
        for label, es, pick in (("Tier 1", "L", lambda m: m.get("tier") == 1), ("Tier 2", "L", lambda m: m.get("tier") == 2),
                                ("Tier 3", "L", lambda m: m.get("tier") == 3), ("Tier 3 at 10L", "10L", lambda m: True),
                                ("Tier 3S", "tier3s", lambda m: True), ("Tier 3S at 10L", "tier3s10L", lambda m: True)):
            rows = R.get((inst, es, None), {})
            eng = E.get((inst, es, None), {})
            sel = [(r, m) for r, (_, m) in rows.items() if pick(m)]
            mc, ec = defaultdict(int), defaultdict(int)
            for r, m in sel:
                mc[m["cls"]] += 1
                ec[(eng.get(r) or {}).get("class", "missing")] += 1
            c[label] = (dict(ec), dict(mc))
            out.add("E3", inst, label, "E3", "-", "classes", dict(sorted(mc.items())), dict(sorted(ec.items())), "equal",
                    "pass" if dict(mc) == dict(ec) else "fail")
        conv_ok = all(set(e) <= {"CONVERGED", "VACUOUS"} and e.get("VACUOUS", 0) == m.get("VACUOUS", 0)
                      for e, m in c.values())
        t12 = all(set(c[t][0]) <= {"CONVERGED", "VACUOUS"} for t in ("Tier 1", "Tier 2"))
        free_ok = all(r["status"] == "pass" for r in out.rows if r["step"] == "E3" and r["inst"] == inst
                      and r["what"].startswith("the free-able market's end state"))
        verdict = "GO" if ma and conv_ok and kicks_ok[inst] and free_ok else ("LOCAL" if ma and t12 else "NO-GO")
        out.add("E3", inst, "verdict", "verdict", "-", f"{inst.upper()}'s verdict", "GO", verdict,
                "SPEC §9.1: mode A; Tiers 1-3 and 3S CONVERGED at L, 3 and 3S at 10 L; kick sets; the free-able end",
                "pass" if verdict == "GO" else "fail")
        verdicts.append(OrderedDict(inst=inst, verdict=verdict, prediction="GO",
                                    **{k: json.dumps(v[0], sort_keys=True) for k, v in c.items()},
                                    kicks="13/13 PASS" if kicks_ok[inst] else "not all PASS",
                                    free_end="every CONVERGED run as the oracle's" if free_ok else "not all"))
    out.charge()
    return out, dict(per_run=per_run, fam=fam_rows, dial=dial_rows, modea=mode_rows, kicks=kick_rows,
                     verdicts=verdicts)


def refutations(out, self_test):
    res = []
    fe = [r for r in out.rows if r["what"].startswith("the free-able market's end state") and r["status"] == "fail"]
    if fe:
        res.append(f"{len(fe)} CONVERGED runs whose free-able market ends priced at a free target or free at a priced one")
    if not self_test:
        off = [r for r in out.rows if r["what"] == "end D-hat" and r["status"] == "fail"]
        if off:
            res.append(f"{len(off)} CONVERGED runs resting off the oracle's point (gap above 1e-12)")
    ma = [r for r in out.rows if r["step"] == "E2" and r["set"] == "modea" and r["status"] == "fail"]
    if ma:
        res.append(f"{len(ma)} mode-A lines failing (a rest point off the oracle's)")
    bad = [r for r in out.rows if r["what"] == "class" and r["step"] == "E3" and r["status"] == "fail"
           and r["engine"] != "CONVERGED" and r["registered"] == "CONVERGED"]
    if bad:
        res.append(f"{len(bad)} Tier 1-3 or 3S runs not CONVERGED")
    ks = [r for r in out.rows if r["what"].startswith("kick set decays") and r["status"] == "fail"]
    if ks:
        res.append(f"{len(ks)} kick sets failing")
    return res


# ---------------------------------------------------------------- the self-test's records
def selftest_records():
    recs = []
    R, modes = registered()
    for (inst, es, setting), rows in R.items():
        g = FREE[inst]
        for run, (step, m) in rows.items():
            stats = {f"trough.cleared|{mk}": (m.get("trough") or {}).get(mk) for mk in MARKETS}
            stats.update({f"free.end_over_ref|{g}": (m.get("free_end") or {}).get(g),
                          f"free.first_zero|{g}": (m.get("free_first") or {}).get(g),
                          f"free.switches|{g}": (m.get("free_switches") or {}).get(g),
                          f"free.zero_ticks|{g}": (m.get("free_ticks") or {}).get(g),
                          "free.regime_end|-": m.get("regime_end"),
                          "free.regime_switches|-": m.get("regime_switches")})
            recs.append(dict(key="free", cmd="run", inst=inst, set=es, setting=setting, names=[run], rc=0,
                             ticks=m["L"], in_tol_from=m.get("ttol"), dead=m.get("dead"),
                             low_baskets=m.get("baskets_trough"), peak_dhat=m.get("peak"),
                             runaway_tick=runaway_of(m.get("err")), last=None, stats=stats, **{"class": m["cls"]}))
    for (inst, tpy), m in modes.items():
        recs.append(dict(key="free", cmd="run", inst=inst, set="modea", tpy=tpy, names=["hold"], rc=0,
                         mode_a="PASS", peak_dhat=m["peak"], ticks=m["L"],
                         stats={f"free.zero_ticks|{FREE[inst]}": (m.get("free_ticks") or {}).get(FREE[inst])},
                         **{"class": m["cls"]}))
    elas = json.load(open(os.path.join(REG, "elasticity_free.json")))
    for inst in ("il1", "ct2"):
        for tpy in (52, 12, 365):
            recs.append(dict(key="free", cmd="elasticity", inst=inst, set="info", tpy=tpy,
                             rel=f"free/{inst}/info/el{tpy}", rc=0,
                             L=elas[f"{NAME[inst].split('/')[0]}/zp01 {tpy}"]["L"]))
    lin = json.load(open(os.path.join(REG, "lin_all05.json")))
    for inst in ("il1", "ct2"):
        pl = {x["target"]: x["PL"] for x in lin if x["name"] == NAME[inst] and x.get("dials", "C2m") == "C2m"}
        tg = [("hold", "base")] + [(f"{c}={v}@dated", f"{c} {f}") for c, vs in COEF[inst] for v, f in zip(vs, FACTORS)]
        for t, mt in tg:
            ok = kick_bar(pl[mt], LT[inst][52])
            recs.append(dict(key="free", cmd="kick", inst=inst, set="kick", rel=f"free/{inst}/kick/{fname(t)}", rc=0,
                             passed=ok, gain_tail=1e-5 if ok else 1.0, gain_peak=1.0, kicks=12))
    return recs


def selftest_negative(recs):
    """Five records made wrong: a battery class, a free end state (priced at a free target), a
    first free tick 3 ticks late, a joint4 trap run's runaway tick 10% off, and a mode A that fails."""
    import copy
    bad = copy.deepcopy(recs)
    done = set()
    for r in bad:
        n = (r.get("names") or ["-"])[0]
        if r.get("cmd") == "run" and r["inst"] == "ct2" and r["set"] == "L" and n == "JB(2)" and "a" not in done:
            r["class"] = "STUCK"
            done.add("a")
        elif r.get("cmd") == "run" and r["inst"] == "il1" and r["set"] == "L" and n == "p[labour]*2" and "b" not in done:
            r["stats"]["free.end_over_ref|land"] = 0.01
            done.add("b")
        elif r.get("cmd") == "run" and r["inst"] == "il1" and r["set"] == "L" and n == "p[land]=0.025" and "c" not in done:
            r["stats"]["free.first_zero|land"] = (r["stats"]["free.first_zero|land"] or 0) + 3
            done.add("c")
        elif r.get("cmd") == "run" and r["inst"] == "ct2" and r["set"] == "joint4" and r["class"] == "DIVERGED" and "d" not in done:
            r["runaway_tick"] = int(r["runaway_tick"] * 1.1)
            done.add("d")
        elif r.get("cmd") == "run" and r["inst"] == "il1" and r["set"] == "modea" and r.get("tpy") == 365 and "e" not in done:
            r["mode_a"] = "FAIL: a market's gap 1e-6"
            done.add("e")
    assert done == set("abcde"), done
    return bad


def main(runs_path, out_dir, self_test=False):
    recs = read_recs(runs_path)
    out, T = score(recs, self_test)
    ref = refutations(out, self_test)
    write_lines(out_dir, out)
    for k, fn in (("per_run", "runs.csv"), ("fam", "families.csv"), ("dial", "dial.csv"), ("modea", "modea.csv"),
                  ("kicks", "kicks.csv"), ("verdicts", "verdicts.csv")):
        write_rows(os.path.join(out_dir, fn), T[k])
    st = printout(out, f"free: {len(recs)} records")
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
