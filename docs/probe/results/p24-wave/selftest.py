"""P2.4 (label run, 2026-09-30): the scorers' self-test. For each of the three waves it writes the
registered mirror records as gather.py writes the engine's (runs.jsonl records), scores them with
`--self-test`, and then scores a negative control: the same records with a few made wrong on
purpose, which must fail exactly those lines and name the refutations. Usage (WSL):
python3 selftest.py OUT_DIR  (writes OUT_DIR/<key>/ and OUT_DIR/<key>-neg/)."""
import copy
import csv
import json
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import score_free  # noqa: E402
import score_switch  # noqa: E402
import score_trap  # noqa: E402
from common import REPO, fname, kick_bar  # noqa: E402

MARKETS_C = ["labour", "land", "manufactures", "food", "care", "shelter", "mach"]


# ---------------------------------------------------------------- the trap
def trap_records():
    T = score_trap
    recs = []
    for (inst, es, setting), rows in T.registered().items():
        for run, (step, m) in rows.items():
            L = {"10L": 10, "tier3s10L": 10}.get(es, 1) * T.L52[inst]
            if es in ("tpy12", "tpy365"):
                L = int(m["L"])
            if es == "history":
                wins = [dict(w=w, in_tol_at_end=m["ttols"][w] is not None, ttol=m["ttols"][w],
                             dead=m["deads"][w], regime=m["regimes"][w]) for w in range(m["windows"])]
                recs.append(dict(key="trap", cmd="run", inst=inst, set=es, setting=setting, names=[run],
                                 rc=0, ticks=81 * 1500, windows=wins, runaway_window=None,
                                 **{"class": m["cls"]}))
                continue
            rt = dict(m.get("regime_ticks") or {})
            if m.get("stopped_early") and m.get("regime_end"):
                rt[m["regime_end"]] = rt.get(m["regime_end"], 0) + L - m["ticks_run"]
            stats = {f"trough.cleared|{mk}": (m.get("trough") or {}).get(mk) for mk in MARKETS_C}
            stats.update({f"commons.regime_ticks|{k}": rt.get(k, 0) for k in ("Commons", "Crowded", "Enclosed")})
            stats.update({"commons.regime_ticks|Split": 0, "commons.switches|-": m.get("regime_switches"),
                          "commons.regime_end|-": m.get("regime_end"),
                          "commons.provider_coin_low|-": m.get("provider_coin_low"),
                          "commons.tp_max|-": m.get("tp_high")})
            if inst in T.PACED:
                stats["pace.zero_hours|-"] = m.get("hours_zero")
            recs.append(dict(key="trap", cmd="run", inst=inst, set=es, setting=setting, names=[run], rc=0,
                             ticks=L, in_tol_from=m.get("ttol"), dead=m.get("dead"),
                             low_baskets=m.get("baskets_trough"), no_basket_ticks=m.get("baskets_none"),
                             peak_dhat=m.get("peak"), transfer_short_ticks=m.get("transfer_ticks"),
                             runaway_tick=T.runaway_of(m.get("err_harness_ref") or m.get("err")),
                             last=m.get("final_Dh"), stats=stats, end={}, **{"class": m["cls"]}))
    for ma in json.load(open(os.path.join(T.REG, "modea.json"))):
        recs.append(dict(key="trap", cmd="run", inst=ma["inst"].lower(), set="modea", tpy=int(ma["tpy"]),
                         names=["hold"], rc=0, mode_a="PASS", peak_dhat=ma["worst"] * 1e3,
                         **{"class": "CONVERGED"}))
    elas = json.load(open(os.path.join(T.REG, "elasticity-P1.3r.json")))
    for inst in ("c1p", "c2p", "c1pn"):
        for tpy in (52, 12, 365):
            Lr = elas[f"{T.MIRROR[inst]} {tpy}"]["L"] if inst in T.MIRROR else {52: 141000, 12: 26000, 365: 1113000}[tpy]
            recs.append(dict(key="trap", cmd="elasticity", inst=inst, set="info", tpy=tpy,
                             rel=f"trap/{inst}/info/el{tpy}", rc=0, L=Lr))
    lin = json.load(open(os.path.join(T.REG, "lin-P1.3r.json")))
    for inst in ("c1p", "c2p"):
        pl = {x["target"]: x["PL"] for x in lin if x["name"] == T.MIRROR[inst]}
        tgts = [("hold", "base")]
        for c, _ in T.BASE[inst]:
            for v, f in zip(T.TARGET_VALUES[inst][c], T.FACTORS):
                tgts.append((f"{c}={v}@dated", f"{'exit.To' if c == 'commons' else c} {f}"))
        for t, mt in tgts:
            ok = kick_bar(pl[mt], T.L52[inst])
            recs.append(dict(key="trap", cmd="kick", inst=inst, set="kick", rel=f"trap/{inst}/kick/{fname(t)}",
                             rc=0, passed=ok, gain_tail=pl[mt] ** (0.9 * T.L52[inst]) if ok else 1.0,
                             gain_peak=1.0, kicks=14))
    return recs


def trap_negative(recs):
    """Five records made wrong: a class, ticks to tolerance 30% off, a runaway tick 10% off, a
    tick with no hours in a C1P battery run, and a kick set that fails."""
    bad = copy.deepcopy(recs)
    done = set()
    for r in bad:
        n = (r.get("names") or ["-"])[0]
        if r.get("cmd") == "run" and r["inst"] == "c1p" and r["set"] == "L" and n == "p[labour]*2" and "a" not in done:
            r["in_tol_from"] = int(r["in_tol_from"] * 1.3)
            done.add("a")
        elif r.get("cmd") == "run" and r["inst"] == "c2p" and r["set"] == "stocks" and "b" not in done:
            r["class"] = "DIVERGED"
            done.add("b")
        elif r.get("cmd") == "run" and r["inst"] == "c1" and r["set"] == "ctl-dial" and r["class"] == "DIVERGED" and "c" not in done:
            r["runaway_tick"] = int(r["runaway_tick"] * 1.1)
            done.add("c")
        elif r.get("cmd") == "run" and r["inst"] == "c1p" and r["set"] == "L" and n == "p[food]*2" and "d" not in done:
            r["stats"]["pace.zero_hours|-"] = 3
            done.add("d")
        elif r.get("cmd") == "kick" and r["inst"] == "c2p" and r["rel"].endswith("b.food_1.2_dated") and "e" not in done:
            r["passed"] = False
            done.add("e")
    assert done == set("abcde"), done
    return bad


# ---------------------------------------------------------------- the switch
def switch_records():
    S = score_switch
    return S.selftest_records()


def switch_negative(recs):
    return score_switch.selftest_negative(recs)


# ---------------------------------------------------------------- the free step
def free_records():
    return score_free.selftest_records()


def free_negative(recs):
    return score_free.selftest_negative(recs)


def run(key, recs, out_dir, neg):
    path = os.path.join(out_dir, f"{key}{'-neg' if neg else ''}.jsonl")
    with open(path, "w") as f:
        for r in recs:
            f.write(json.dumps(r, sort_keys=True) + "\n")
    mod = {"trap": score_trap, "switch": score_switch, "free": score_free}[key]
    print(f"==== {key}{' (negative control)' if neg else ''}")
    sys.stdout.flush()
    ok = mod.main(path, os.path.join(out_dir, f"{key}{'-neg' if neg else ''}"), True)
    sys.stdout.flush()
    return ok


def main(out_dir):
    os.makedirs(out_dir, exist_ok=True)
    good = True
    for key, mk, neg in (("trap", trap_records, trap_negative), ("switch", switch_records, switch_negative),
                         ("free", free_records, free_negative)):
        recs = mk()
        ok = run(key, recs, out_dir, False)
        print(f"self-test {key}: {'PASS' if ok else 'FAIL'}")
        good &= ok
        nok = run(key, neg(recs), out_dir, True)
        with open(os.path.join(out_dir, f"{key}-neg", "fails.csv")) as f:
            fails = list(csv.DictReader(f))
        shown = [r for r in fails if r["status"] != "missing"]
        print(f"negative control {key}: the scorer {'fails it' if not nok else 'passes it (WRONG)'}; "
              f"{len(shown)} lines fail or are charged (the missing end-of-run lines left out):")
        for r in shown:
            print(f"   {r['step']} {r['inst']} {r['set']} {r['run']}: {r['what']}: {r['registered']} vs "
                  f"{r['engine']} -> {r['status']}")
        good &= not nok
    print("SELF-TEST " + ("PASS" if good else "FAIL"))
    return good


if __name__ == "__main__":
    sys.exit(0 if main(sys.argv[1]) else 1)
