"""P2.4 (label run, 2026-10-01), after the free step's wave was scored, not a scorer: CT2's dial run
JB(2) at tilt.*=1, the one free/priced-switch line still failing after A2 (the mirror's 2
switches to its early stop, the engine's 8 over L). The engine's first 4,001 ticks every tick (a
diagnostic run of the wave's binary, `markets run JB(2) --inst ct2 --set tilt.*=1 --ticks 4001
--every 1`) against the free registration's mirror (a copy of D:/rustyecon-p24/scan-free/model,
checked against that scan's SHA256SUMS, not edited) recording every tick: where each posts the
commons at 0 or a positive price, and how far apart the two paths are before they part.
Then, at the ticks around the parting, each pop's coin in the mirror and its commons spend r_o*bid,
and the engine's commons demand against the sum of the pops' bids (its budget chain caps the
commons' budget at what the baskets leave, FREE-RULES: "the orders are the mirror's while the coin
covers it").
Usage (WSL): python3 ct2_switches.py MODEL_COPY ENGINE_CSV"""
import csv
import math
import os
import sys

M, ECSV = sys.argv[1], os.path.abspath(sys.argv[2])
sys.path.insert(0, M)
os.chdir(M)
import cm  # noqa: E402
import fb  # noqa: E402
import fm  # noqa: E402

over = {f"tilt.{d}": 1.0 for d in ("manufactures", "food", "care", "shelter", "mach")}
rec = []
r = fb.run("CT2/zp05", "JB(2)", tpy=52.0, L=141000, stable_at=True, over=over, record=(rec, 1))
mp = [x["p"] for x in rec]
rows = list(csv.reader(open(ECSV)))
h = rows[0]
cols = [h.index(f"p.{m}") for m in ("labour", "land", "manufactures", "food", "care", "shelter", "mach", "commons")]
ep = [[float(x[i]) for i in cols] for x in rows[1:]]
# align: the mirror records the state a tick starts from; find the engine row offset that matches
best = min((sum(abs(math.log(ep[t + o][0] / mp[t][0])) for t in range(1, 50)), o) for o in (-1, 0, 1))
o = best[1]
print(f"mirror: {r['cls']}, ticks to tolerance {r['ttol']}, free/priced switches {r['free_switches']}, "
      f"ticks run {r['ticks_run']}; engine rows {len(ep)}; row offset {o}")


def episodes(ps):
    out, start = [], None
    for t, p in enumerate(ps):
        if p > 0 and start is None:
            start = t
        if p == 0 and start is not None:
            out.append((start, t - 1))
            start = None
    if start is not None:
        out.append((start, len(ps) - 1))
    return out


n = min(len(mp), len(ep) - max(o, 0))
mc = [mp[t][7] for t in range(n)]
ec = [ep[t + o][7] for t in range(n)]
print("mirror's priced spans (ticks):", episodes(mc))
print("engine's priced spans (ticks):", episodes(ec))
gap, part = 0.0, None
for t in range(n):
    a, b = mc[t], ec[t]
    if (a == 0.0) != (b == 0.0):
        print(f"first tick where one posts the commons at 0 and the other not: {t}; mirror {a!r}, engine {b!r}")
        break
    g = max(abs(math.log(ep[t + o][j] / mp[t][j])) for j in range(7))
    if a > 0 and b > 0:
        g = max(g, abs(math.log(b / a)))
    if part is None and g > 1e-12:
        part = t
        print(f"the paths part at tick {t}: the largest gap in log over the posted prices {g:.3e} "
              f"(before it at most {gap:.3e}); the commons: mirror {a!r}, engine {b!r}")
    gap = max(gap, g)

E, O, st0 = fm.build("CT2/zp05", dict(cm.C2M), 52.0, over)
st = fb.displace(E, dict(st0), "JB(2)")
st["genesis"] = True
d = {k: i for i, k in enumerate(h)}
for t in range(97):
    new, info = fm.tick(E, st, diag=True)
    if t >= 92:
        pops = info["pops"]
        x = rows[t + 1 + o]
        print(f"tick {t}: mirror coin wa {st['Mw'][0]:.6g}, wb {st['Mw'][1]:.6g}; r_o*bid wa "
              f"{st['p'][7] * pops[0]['bid']:.6g}, wb {st['p'][7] * pops[1]['bid']:.6g}; mirror's bids "
              f"{pops[0]['bid'] + pops[1]['bid']:.6g} | engine's commons demand {float(x[d['commons_D']]):.6g}, "
              f"its pops' bids {float(x[d['commons_bid_workers.wa']]) + float(x[d['commons_bid_workers.wb']]):.6g}")
    st = new
