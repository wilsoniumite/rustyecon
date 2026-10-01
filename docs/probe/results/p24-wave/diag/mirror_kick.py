"""P2.4 (label run, 2026-10-01), after the free step's wave was scored, not a scorer: kicks at three
of CT2's dated targets on the free registration's own mirror (a copy of
D:/rustyecon-p24/scan-free/model, every file checked against that scan's SHA256SUMS, not edited),
the harness's way (probe::markets::kick): the base economy for L/4 ticks and the shocked one for
L, then one market's posted price times (1 +- 1e-9) against the unkicked continuation for H
ticks; g(t) the largest |ln(p_kick/p_base)| over the markets (a free market's equal prices read 0),
the gain g(t)/g(0), its tail the last tenth of H. The registration predicted each target's kick
set from the mirror's largest root (PL^(0.9 L) <= 1e-3); this runs the kicks themselves.
Usage (WSL): python3 mirror_kick.py MODEL_COPY [H]"""
import copy
import math
import os
import sys

M = sys.argv[1]
H = int(sys.argv[2]) if len(sys.argv) > 2 else 70500
sys.path.insert(0, M)
os.chdir(M)
import fm  # noqa: E402

L = 141000
KICKS = {"b.food=1.2": [("care", -1), ("mach", -1), ("care", 1), ("shelter", 1), ("commons", 1), ("labour", -1)],
         "exit.To=17.55": [("care", -1), ("manufactures", 1), ("mach", 1)],
         "exit.To=9.75": [("commons", 1), ("commons", -1), ("care", -1)]}
for target, kicks in KICKS.items():
    coef, v = target.split("=")
    E, O, st = fm.build("CT2/zp05")
    I2 = fm.shocked(E.I, coef, float(v))
    E2 = fm.Econ(I2, fm.dials(I2, dict(fm.cm.C2M), 52.0, None))
    st["genesis"] = True
    for t in range(L // 4):
        st = fm.tick(E, st)
    for t in range(L):
        st, _ = fm.tick(E2, st, diag=True)
    mi = E2.markets
    free = set(E2.free)
    print(f"CT2 {target}@dated, kicked at tick {L // 4 + L}, H {H}; posted prices "
          + " ".join(f"{m} {x:.6g}" for m, x in zip(mi, st["p"])))

    def cont(s):
        out = []
        for t in range(H):
            s, _ = fm.tick(E2, s, diag=True)
            out.append(list(s["p"]))
        return out

    base = cont(copy.deepcopy(st))
    tail = math.ceil(0.1 * H)
    for m, sign in kicks:
        k = copy.deepcopy(st)
        i = mi.index(m)
        k["p"][i] *= (1.0 + sign * 1e-9)
        kp = cont(k)
        g0 = abs(math.log(k["p"][i] / st["p"][i]))
        gains = [max(0.0 if (j in free and a == b) else abs(math.log(b / a))
                     for j, (a, b) in enumerate(zip(pb, pk))) / g0 for pb, pk in zip(base, kp)]
        print(f"  {m:13} {'+' if sign > 0 else '-'}  tail {max(gains[-tail:]):.4e}  peak {max(gains):.4e}  "
              f"at H/2 {gains[H // 2]:.4e}  at H {gains[-1]:.4e}")
