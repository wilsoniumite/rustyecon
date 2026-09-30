"""run (P2.3.15, 2026-09-30), a diagnostic after scoring: the trap runs' runaway ticks on the
registered commons mirror (cm.py, battery_c.displace, unedited, read in place), measured against
two references: the undisplaced genesis prices (the mirror's battery_c.run, p_gen = st0's) and
the displaced genesis prices (the engine harness's, g.prices). Prints both beside the engine's
tick from runs.jsonl. Usage (WSL): python3 runaway_ref.py RUNS.jsonl"""
import json
import math
import sys
from multiprocessing import Pool

sys.path.insert(0, "/mnt/d/rustyecon-p23/frame-commons/model")
import instances  # noqa: F401,E402  (sets the commons T_o)
import cm  # noqa: E402
import battery_c as bc  # noqa: E402

cm.VARIANTS["chi025"] = dict(chimax=0.25)
NAME = {"c1": "C1", "c2": "C2", "c1n": "C1/chi025"}


def out_of(p, ref):
    return any((not math.isfinite(x)) or x <= 0 or x / r > 1e6 or x / r < 1e-6 for x, r in zip(p, ref))


def job(a):
    inst, run = a
    E, O, st0 = cm.build(NAME[inst], cm.C2M, 52.0)
    st = bc.displace(E, dict(st0), run)
    st["genesis"] = True
    und, dis = list(st0["p"]), list(st["p"])
    t_und = t_dis = None
    for t in range(600):
        st = cm.tick(E, st)
        if t_und is None and out_of(st["p"], und):
            t_und = t
        if t_dis is None and out_of(st["p"], dis):
            t_dis = t
        if t_und is not None and t_dis is not None:
            break
    return inst, run, t_und, t_dis


if __name__ == "__main__":
    eng = {}
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("class") == "DIVERGED" and r.get("inst") in NAME and r.get("set") in ("joint2", "joint4", "basin", "L", "tilt1") and "@" not in r.get("run", "@") and r.get("set") != "tilt1":
            eng[(r["inst"], r["run"])] = r.get("runaway_tick")
    with Pool(46) as p:
        res = p.map(job, sorted(eng))
    n = same_und = same_dis = 0
    worst = 0
    for inst, run, tu, td in res:
        e = eng[(inst, run)]
        n += 1
        same_dis += td == e
        worst = max(worst, abs(td - e) if td is not None and e is not None else 999)
        print(f"{inst}\t{run}\tmirror, undisplaced reference {tu}\tmirror, displaced reference {td}\tengine {e}")
    print(f"{n} trap runs at genesis: the mirror against the displaced reference equals the engine's tick in {same_dis}; largest difference {worst} ticks")
