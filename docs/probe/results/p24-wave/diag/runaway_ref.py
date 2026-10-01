"""P2.4 (label run, 2026-10-01), after the free step's wave was scored, not a scorer: its eight
subsistence-trap runs (joint4 at IL1 and CT2, every one DIVERGED in mirror and engine) on the free
registration's own mirror, with the runaway bound held to the displaced genesis prices, as the
harness holds it (decision 399 as amended; the commons' A3, O107), instead of the undisplaced
genesis `fb.run` holds it to. The mirror is a copy of D:/rustyecon-p24/scan-free/model, every file
checked against that scan's SHA256SUMS, not edited: two module functions are wrapped from outside,
`fb.displace` (to keep the displaced state) and `fb.out_of_bounds` (to read it).
Usage (WSL): python3 runaway_ref.py MODEL_COPY OUT.json  (prints a table; writes one JSON line a run)"""
import json
import os
import sys

M, OUT = sys.argv[1], os.path.abspath(sys.argv[2])
sys.path.insert(0, M)
os.chdir(M)
import fb  # noqa: E402

KEEP = {}
_disp, _oob = fb.displace, fb.out_of_bounds


def displace(E, st, term):
    out = _disp(E, st, term)
    KEEP["p"] = list(out["p"])
    return out


def oob_displaced(E, pr, p_gen):
    return _oob(E, pr, KEEP["p"])


RUNS = [("ct2", 10), ("ct2", 12), ("ct2", 18), ("ct2", 21), ("ct2", 22), ("ct2", 34), ("ct2", 39), ("il1", 10)]
L = {"il1": 56000, "ct2": 141000}
rows = []
print("instance run | the mirror, bound at the undisplaced genesis (as registered) | at the displaced genesis (the harness's)")
for inst, s in RUNS:
    name = f"{inst.upper()}/zp05"
    run = f"joint(4,{s})"
    fb.displace, fb.out_of_bounds = displace, _oob
    a = fb.run(name, run, tpy=52.0, L=L[inst], stable_at=True)
    fb.out_of_bounds = oob_displaced
    b = fb.run(name, run, tpy=52.0, L=L[inst], stable_at=True)
    fb.displace, fb.out_of_bounds = _disp, _oob
    ta = int(a["err"].split("t=")[1]) if a.get("err") else None
    tb = int(b["err"].split("t=")[1]) if b.get("err") else None
    rows.append(dict(inst=inst, run=run, cls_undisplaced=a["cls"], tick_undisplaced=ta, cls_displaced=b["cls"],
                     tick_displaced=tb))
    print(f"{inst} {run} | {a['cls']} {ta} | {b['cls']} {tb}")
with open(OUT, "w", newline="\n") as f:
    for r in rows:
        f.write(json.dumps(r, sort_keys=True) + "\n")
