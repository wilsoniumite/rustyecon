"""build-commons (2026-09-30): Tier 3S on the registered commons mirror, for the registration.

Decision 368 puts a stocks tier in the new instances' verdicts: every desk's stock and coin, and
each household's coin, at x0.5 and x2 (P2.2a's Tier 3S, decision 229; the wall's `tier3s`),
with the start distance the largest D-hat of the first scored year. The commons frame
(D:/rustyecon-p23/frame-commons/SPEC.md, sha256 60f21f56...b40d) reports a stocks family and no
Tier 3S. This script runs Tier 3S on the frame's own mirror, unedited, before any engine code
for the commons exists, so the verdict's stocks tier has registered predictions:

  - each run through `battery_c.run` (the frame's runner: class, ticks to tolerance, the regime
    readouts), at the frame's L and early stop;
  - the first year's largest D-hat (ticks 0-51, the engine's `--first-year` start) by the same
    tick map and observables (`cm.tick`, `battery_c.observe`, `battery_c.targets_of`);
  - the class the engine's classifier gives with that start: VACUOUS where the first year's D-hat
    is at most 1, else the runner's.

Names are the engine's (`coin.desk.<d>*F`); the mirror's are `coin.<d>*F`. Output: one JSON
line per run, in the engine's `tier3s` order, to tier3s.jsonl beside this file.

usage (WSL): cd /mnt/d/rustyecon-p23/frame-commons/model && python3 /mnt/d/rustyecon-p23/build-commons/tier3s_c.py
"""
import json
import math
import sys
from multiprocessing import Pool

sys.path.insert(0, "/mnt/d/rustyecon-p23/frame-commons/model")
import instances  # noqa: E402  (sets the commons T_o)
import cm  # noqa: E402
import battery_c as bc  # noqa: E402

OUT = "/mnt/d/rustyecon-p23/build-commons/tier3s.jsonl"
DESKS = ["manufactures", "food", "care", "shelter", "mach"]
ELAST = json.load(open("/mnt/d/rustyecon-p23/frame-commons/model/elasticity.json"))


def names():
    """The engine's Tier 3S (`probe::markets::perturb::tier3s`) with the mirror's name of each."""
    out = []
    for d in DESKS:
        for f in ("0.5", "2"):
            out.append((f"stock.{d}*{f}", f"stock.{d}*{f}"))
    for d in DESKS:
        for f in ("0.5", "2"):
            out.append((f"coin.desk.{d}*{f}", f"coin.{d}*{f}"))
    for a in ("workers", "provider"):
        for f in ("0.5", "2"):
            out.append((f"coin.{a}*{f}", f"coin.{a}*{f}"))
    return out


def first_year(name, run):
    """The largest D-hat over ticks 0-51 from the displaced genesis, as `battery_c.run` scores."""
    E, O, st0 = cm.build(name)
    st = bc.displace(E, st0, run)
    st["genesis"] = True
    tgt, _ = bc.targets_of(E, O)
    worst = 0.0
    for _ in range(52):
        new, info = cm.tick(E, st, diag=True)
        o = bc.observe(E, st, info, new)
        dh = max(abs(math.log(o[k] / tgt[k])) if o[k] > 0 and math.isfinite(o[k]) else math.inf
                 for k in tgt) / bc.TOL
        worst = max(worst, dh)
        st = new
    return worst


def job(a):
    name, engine_name, mirror_name, mult = a
    L = ELAST[f"{name} 52"]["L"] * mult
    r = bc.run(name, mirror_name, L=L, stable_at=True)
    d0 = first_year(name, mirror_name)
    cls = "VACUOUS" if d0 <= 1.0 else r["cls"]
    keep = {k: r.get(k) for k in ("cls", "err", "peak", "peak_t", "ttol", "dead", "baskets_trough",
                                  "baskets_none", "worst_fill", "transfer_ticks", "provider_coin_low",
                                  "regime_ticks", "regime_switches", "regime_end", "regime_star",
                                  "ro_low", "ro_high", "ro_end", "ro_star", "tp_high", "ticks_run",
                                  "stopped_early", "final_Dh")}
    return dict(name=name, run=engine_name, mirror_run=mirror_name, L=L,
                set="tier3s" if mult == 1 else "tier3sx10", d0_first_year=d0,
                cls_engine_rule=cls, runner=keep)


if __name__ == "__main__":
    jobs = [(n, e, m, mult) for mult in (1, 10) for n in ("C1", "C2") for e, m in names()]
    with Pool(24) as p:
        rows = p.map(job, jobs, chunksize=1)
    with open(OUT, "w") as f:
        for r in rows:
            f.write(json.dumps(r, default=str) + "\n")
    for r in rows:
        x = r["runner"]
        print(f"{r['name']} {r['set']:9s} {r['run']:28s} {r['cls_engine_rule']:9s} (runner {x['cls']}) "
              f"d0 {r['d0_first_year']:.1f} ttol {x['ttol']} dead {x['dead']} peak {x['peak']:.0f}")
