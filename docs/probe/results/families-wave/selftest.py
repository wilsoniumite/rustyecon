"""P2.4 (label families, 2026-09-30): the scorer's self-test, before the wave (decision 311). It
writes the registered mirror records (docs/probe/families/registered/) as gather.py writes the
engine's (runs.jsonl's records: the summary line's fields, the runaway tick, the history's
windows, the kick sets' pass), then score.py scores them with --self-test. Every scored line the
mirror's records feed must pass. The only lines left missing are the end-of-run lines ("end
D-hat", a gap of at most 1e-12 at tick L), which only an engine run to L gives: the mirror stops
a run once it has stayed within 1e-6 of its target for 2,000 ticks.

A kick set's record is the mirror's prediction from its PL (PASS iff PL^(0.9 L) <= 1e-3); a map
cell's mode A record is PASS where the mirror's `hold` stays within a gap of 1e-9.

Then a negative control: the same records with nine made wrong on purpose (a class, ticks to
tolerance 30% and 15% off, a runaway tick 10% off, dead ticks off by 20, the lowest baskets off
by 0.2, a history window's end, a map kick set, and one end D-hat of 1e-3, every other
CONVERGED record given an end D-hat of 0), scored as the engine's. Exactly those lines must fail
(the 15% one charged; the kick set also moves its cell's verdict), the refutations must name the
class, the kick set, the cell's verdict and the end D-hat, and nothing else may change.
Usage (WSL): python3 selftest.py OUT_DIR"""
import copy
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import score as S  # noqa: E402

TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}


def fname(run):
    return "".join(TR.get(c, c) for c in run)


def rec_of(m, set_, inst, cell=None, setting=None):
    r = {"rel": f"selftest/{inst}/{set_}/{cell or setting or '-'}/{fname(m['run'])}", "cmd": "run",
         "inst": inst, "set": set_, "cell": cell, "setting": setting, "names": [m["run"]],
         "rc": 0, "run": m["run"], "class": m["cls"], "in_tol_from": m.get("ttol"),
         "dead": m.get("dead"), "low_baskets": m.get("baskets_trough"),
         "runaway_tick": m.get("runaway_t"), "last": None}
    if m["run"] == "hold" and set_ == "map_modea":
        r["mode_a"] = "PASS" if m.get("peak", math.inf) <= 1e-6 else "FAIL"
    return r


def records(R):
    recs = []
    for (s, inst, run), m in R["i"].items():
        recs.append(rec_of(m, s, inst))
    for inst, h in R["history"].items():
        wins = []
        for w in h["windows"]:
            wins.append(dict(w=w["w"], end_Dh=w["end_Dh"], ttol=w["ttol"], dead=w["dead"],
                             in_tol_at_end=w["end_Dh"] is not None and w["end_Dh"] <= 1))
        recs.append({"rel": f"selftest/{inst}/history", "cmd": "run", "inst": inst,
                     "set": "history", "names": [h["run"]], "rc": 0, "run": h["run"],
                     "class": "-", "windows": wins, "runaway_window": None})
    for (cell, run), m in R["map"].items():
        recs.append(rec_of(m, "map_modea" if run == "hold" else "map", "i3", cell=cell))
    for (cell, target), p in R["map_pl"].items():
        name = "hold" if target == "base" else f"{target}@dated"
        recs.append({"rel": f"selftest/i3/map_kick/{cell}/{fname(name)}", "cmd": "kick",
                     "inst": "i3", "set": "map_kick", "cell": cell, "names": [name], "rc": 0,
                     "passed": S.kick_bar(p["PL"], S.MAP_L[cell])})
    for (inst, dials, run), m in R["dial"].items():
        recs.append(rec_of(m, "dial", inst, setting=dials))
    for (inst, dials), p in R["dial_pl"].items():
        recs.append({"rel": f"selftest/{inst}/dial_kick/{fname(dials)}", "cmd": "kick",
                     "inst": inst, "set": "dial_kick", "setting": dials, "names": ["hold"],
                     "rc": 0, "passed": S.kick_bar(p["PL"], S.L_D[inst])})
    return recs


def find(recs, **kw):
    return next(r for r in recs if all(r.get(k) == v for k, v in kw.items()))


def negative(recs):
    """Nine records made wrong on purpose; returns them and the (what, run) of each line that
    must fail (or be charged)."""
    bad = copy.deepcopy(recs)
    want = []
    r = find(bad, set="basin", inst="i1", run=next(x["run"] for x in bad if x.get("set") == "basin"
                                                  and x["inst"] == "i1"))
    r["class"] = "DIVERGED"
    want.append(("class", r["run"], "fail"))
    r = find(bad, set="stocks", inst="i2", run="coin.workers*2")
    r["in_tol_from"] = int(round(r["in_tol_from"] * 1.3))
    want.append(("ticks to tolerance", r["run"], "fail"))
    r = find(bad, set="joint2", inst="i3", run="joint(2,7)")
    r["in_tol_from"] = int(round(r["in_tol_from"] * 1.15))
    want.append(("ticks to tolerance", r["run"], "charged"))
    r = find(bad, set="dial", inst="c1", setting="rate.*=0.9", run="JB(2)")
    r["runaway_tick"] = int(round(r["runaway_tick"] * 1.1))
    want.append(("runaway tick", r["run"], "fail"))
    r = find(bad, set="hold", inst="i1", run="JB(0.5)")
    r["dead"] = r["dead"] + 20
    want.append(("dead ticks", r["run"], "fail"))
    r = find(bad, set="tilt1", inst="i3", run="w*0.5")
    r["low_baskets"] = r["low_baskets"] - 0.2
    want.append(("lowest baskets over Y*", r["run"], "fail"))
    r = find(bad, set="history", inst="i1")
    r["windows"][40]["in_tol_at_end"] = False
    want.append(("in tolerance at its end", "window 40", "fail"))
    r = find(bad, set="map_kick", cell="r1b0.5", names=["b.food=1.2@dated"])
    r["passed"] = not r["passed"]
    want.append(("kick set passes", "b.food=1.2@dated", "fail"))
    want.append(("the cell's verdict (MARKETS-SPEC 7.9)", "verdict", "fail"))
    r = find(bad, set="dial", inst="iw1", setting="tilt.*=1", run="x*/2")
    r["last"] = 1e-3
    want.append(("end D-hat", r["run"], "fail"))
    return bad, want


def write(path, recs):
    with open(path, "w") as f:
        for r in recs:
            f.write(json.dumps(r, sort_keys=True) + "\n")


def main(out_dir):
    R = S.registered()
    recs = records(R)
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, "selftest.jsonl")
    write(path, recs)
    ok = S.main(path, out_dir, self_test=True)
    print("self-test " + ("PASSES" if ok else "FAILS"))
    # the negative control: the same records, nine made wrong, scored as the engine's (not
    # --self-test); every other CONVERGED record gets an end D-hat of 0, so its end line passes
    bad, want = negative(recs)
    for r in bad:
        if r.get("cmd") == "run" and r.get("class") == "CONVERGED" and r.get("last") is None:
            r["last"] = 0.0
    neg_dir = os.path.join(out_dir, "negative")
    os.makedirs(neg_dir, exist_ok=True)
    npath = os.path.join(neg_dir, "negative.jsonl")
    write(npath, bad)
    print("the negative control:")
    S.main(npath, neg_dir, self_test=False)
    import csv
    rows = list(csv.DictReader(open(os.path.join(neg_dir, "lines.csv"))))
    got = sorted((r["what"], r["run"], r["status"]) for r in rows
                 if r["status"] in ("fail", "charged", "missing"))
    wanted = sorted(want)
    neg_ok = got == wanted
    print(f"negative control: {len(got)} lines fail or are charged, {len(wanted)} made wrong; "
          + ("exactly those" if neg_ok else f"MISMATCH: got {got}, wanted {wanted}"))
    print("self-test and negative control " + ("PASS" if ok and neg_ok else "FAIL"))
    return ok and neg_ok


if __name__ == "__main__":
    sys.exit(0 if main(sys.argv[1]) else 1)
