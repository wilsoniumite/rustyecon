"""P2.3.11 (label run, 2026-09-30): the job list of the scored wave of Phase 2 proper's two
loop-free instances, the wall (docs/probe/wall/SPEC.md §7, E2-E9) and the open commons
(docs/probe/commons/SPEC.md §5.5, E2-E9, with registration §3's Tier 3S), as WALL-RULES §8 and
COMMONS-RULES §8 name the commands. One `markets` process per job (job.sh), each in its own
directory under /root/scratch/p23-runs (gzipped to D:/rustyecon-p23/runs/ when gathered).

The wall, IW1 (L 22,000 at 52 a year; 20,000 at 12; 164,000 at 365) and IC1 (L 25,000):
  wall/iw1/info/            elasticity at 52, 12 and 365 a year; the oracle's point (E9)
  wall/iw1/modea/<tpy>/     mode A (hold) at L at 52, 12 and 365 a year (E2)
  wall/iw1/L/<run>/         the registered battery, 103 runs, CSV every tick (E3, E5, E6)
  wall/iw1/tier3s/<run>/    Tier 3S, 20 runs, first-year D-hat, CSV every tick (E3)
  wall/iw1/10L/<run>/       Tier 3 (39) and Tier 3S (20) again at 10 L (E3)
  wall/iw1/<family>/<run>/  stocks (31), joint2 (60), joint4 (40), basin (516) (E7)
  wall/iw1/history/         cycle(land.mach,1500,80) for 80 x 1,500 + 22,000 ticks, CSV every tick
  wall/iw1/tpy12, tpy365/   Tiers 1-2 at 12 and 365 a year at their L (E7)
  wall/iw1/hold, tilt1/     Tiers 1-2 with one-sided Hold, and with every tilt 1 (E7)
  wall/iw1/kick/<target>/   the kick set at the base and at each of the 12 cost targets (dated) (E4)
  wall/iw1/env/<run>/       hold and each price x (1 +- 1e-9) at genesis, 22,000 ticks, CSV every
                            tick: the kick envelope's decay at the base, as the mirror measures it
                            (wstab.kick; E4)
  wall/iw1/s30k/            s[goods]=0.5 for 30,000 ticks (E6's 5e-323)
  wall/ic1/...              IC1: elasticity, point, mode A, the battery (103) and Tier 3S (20) at
                            25,000 (E8)
The commons, C1 (L 141,000; 26,000 at 12; 1,113,000 at 365) and C2 (142,000; 26,000; 1,116,000),
and the negative control C1N at C1's L:
  commons/<c>/info/         elasticity at 52, 12, 365; the oracle's point
  commons/<c>/modea/<tpy>/  mode A at L at 52, 12 and 365 a year (E2)
  commons/<c>/L/<run>/      the battery, 115 runs (E3)
  commons/<c>/10L/<run>/    Tier 3 (43) at 10 L (E3)
  commons/<c>/tier3s/, tier3s10L/   Tier 3S (24) at L and 10 L, first-year D-hat (E3)
  commons/<c>/kick/<target>/        the kick set at the base and each of the 12 dated targets (E3),
                                    and at the base at 12 and 365 a year (E2, reported)
  commons/<c>/stocks, enclose/      the stocks family (41) and enclosure (4) (E4, E9)
  commons/<c>/joint2, joint4, basin/        (E5)
  commons/<c>/history/      cycle(land.mach,1500,80) and cycle(commons,1500,80) for 81 x 1,500
                            ticks, CSV every tick (E6)
  commons/<c>/hold, tilt1/  the battery with one-sided Hold, and with every tilt 1 (E7)
  commons/<c>/tpy12, tpy365/        Tiers 1-2 at 12 and 365 a year at their L (E8)
  commons/c1n/L/<run>/      the negative control's battery at C1's L (E9)

CSV rows: every tick where a per-tick readout is scored (the wall's battery and Tier 3S at L, the
envelope, both histories); every 52 elsewhere at the wall; at the commons every 520 at L, 5,200 at
10 L, 120 at 12 a year and 3,650 at 365 (the last row is always written).
Usage (WSL): python3 make_jobs.py > jobs.txt"""
import os
import shlex
import subprocess
import sys

B = os.environ.get("MARKETS", "/root/scratch/p23-run/markets")
JOB = "/mnt/d/rustyecon-wt/p23/docs/probe/results/p23-wave/job.sh"
OUT = "/root/scratch/p23-runs"
# the engine's own file_name (crates/probe/src/bin/markets.rs)
TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}

WALL_L = {52: 22000, 12: 20000, 365: 164000}
IC1_L = 25000
COMMONS_L = {"c1": {52: 141000, 12: 26000, 365: 1113000},
             "c2": {52: 142000, 12: 26000, 365: 1116000}}
# the wall's cost targets (SPEC §2.3) and the commons' (SPEC §2.6), dated, for the kick sets
WALL_TARGETS = [f"{c}={v}@dated" for c, vs in (("land.mach", ("0.44", "0.36", "0.8", "0.2")),
                                               ("tail.services", ("0.11", "0.09", "0.2", "0.05")),
                                               ("res.services.trained",
                                                ("0.044", "0.036", "0.08", "0.02")))
                for v in vs]
COMMONS_TARGETS = {
    c: [f"{k}={v}@dated" for k, vs in (("land.mach", ("0.44", "0.36", "0.8", "0.2")),
                                       ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                                       ("commons", cv)) for v in vs]
    for c, cv in (("c1", ("26.73", "21.87", "48.6", "12.15")),
                  ("c2", ("34.32", "28.08", "62.4", "15.6")))}
WALL_MARKETS = ["labour", "land", "mach", "services", "goods", "labour.trained", "labour.master"]


def fname(run):
    return "".join(TR.get(c, c) for c in run)


def listing(inst, fam, *opts):
    out = subprocess.run([B, "list", fam, "--inst", inst, *opts], capture_output=True, text=True,
                         check=True).stdout
    rows = []
    for line in out.splitlines():
        p = line.split("\t")
        tier = p[1].replace("tier ", "") if len(p) > 1 else None
        rows.append((p[0], tier, len(p) > 2 and p[2] == "slack"))
    return rows


jobs = []  # (cost in ticks, command)


def job(cost, d, *args):
    cmd = [JOB, d, *args]
    jobs.append((cost, " ".join(shlex.quote(c) for c in cmd)))


def run(d, inst, name, ticks, every, *opts):
    cost = ticks * (1.25 if "@dated" in name else 1.0)
    job(cost, d, "run", name, "--inst", inst, *opts, "--ticks", str(ticks), "--csv", d, "--every",
        str(every))


# ---- the wall
for inst in ("iw1", "ic1"):
    base = f"{OUT}/wall/{inst}"
    tpys = (52, 12, 365) if inst == "iw1" else (52,)
    for tpy in tpys:
        job(1000, f"{base}/info/el{tpy}", "elasticity", "--inst", inst, "--tpy", str(tpy))
        Lt = WALL_L[tpy] if inst == "iw1" else IC1_L
        run(f"{base}/modea/{tpy}", inst, "hold", Lt, tpy, "--tpy", str(tpy))
    job(1000, f"{base}/info/point", "point", "--inst", inst)

L = WALL_L[52]
bat = listing("iw1", "battery")
assert len(bat) == 103, len(bat)
for name, tier, _ in bat:
    run(f"{OUT}/wall/iw1/L/{fname(name)}", "iw1", name, L, 1)
    if tier == "3":
        run(f"{OUT}/wall/iw1/10L/{fname(name)}", "iw1", name, 10 * L, 520)
t3s = listing("iw1", "tier3s")
assert len(t3s) == 20, len(t3s)
for name, _, _ in t3s:
    run(f"{OUT}/wall/iw1/tier3s/{fname(name)}", "iw1", name, L, 1, "--first-year")
    run(f"{OUT}/wall/iw1/10L/{fname(name)}", "iw1", name, 10 * L, 520, "--first-year")
for fam, n in (("stocks", 31), ("joint2", 60), ("joint4", 40), ("basin", 516)):
    names = listing("iw1", fam)
    assert len(names) == n, (fam, len(names))
    for name, _, _ in names:
        run(f"{OUT}/wall/iw1/{fam}/{fname(name)}", "iw1", name, L, 52)
hist = listing("iw1", "history")
assert [h[0] for h in hist] == ["cycle(land.mach,1500,80)"], hist
run(f"{OUT}/wall/iw1/history/{fname(hist[0][0])}", "iw1", hist[0][0], 80 * 1500 + L, 1)
for tpy in (12, 365):
    t12 = [(n, t) for n, t, _ in listing("iw1", "battery", "--tpy", str(tpy)) if t in ("1", "2")]
    assert len(t12) == 64
    for name, _ in t12:
        run(f"{OUT}/wall/iw1/tpy{tpy}/{fname(name)}", "iw1", name, WALL_L[tpy], tpy, "--tpy",
            str(tpy))
for name, tier, _ in bat:
    if tier in ("1", "2"):
        run(f"{OUT}/wall/iw1/hold/{fname(name)}", "iw1", name, L, 52, "--one-sided", "hold")
        run(f"{OUT}/wall/iw1/tilt1/{fname(name)}", "iw1", name, L, 52, "--set", "tilt.*=1")
for target in ["hold"] + WALL_TARGETS:
    d = f"{OUT}/wall/iw1/kick/{fname(target)}"
    job(16 * 1.25 * L, d, "kick", target, "--inst", "iw1", "--ticks", str(L), "--horizon",
        str(L), "--csv", d)
run(f"{OUT}/wall/iw1/env/hold", "iw1", "hold", L, 1)
for m in WALL_MARKETS:
    for f in ("1.000000001", "0.999999999"):
        name = f"p[{m}]*{f}"
        run(f"{OUT}/wall/iw1/env/{fname(name)}", "iw1", name, L, 1)
run(f"{OUT}/wall/iw1/s30k", "iw1", "s[goods]=0.5", 30000, 30000)
line = listing("ic1", "battery")
assert len(line) == 103
for name, _, _ in line:
    run(f"{OUT}/wall/ic1/L/{fname(name)}", "ic1", name, IC1_L, 52)
for name, _, _ in listing("ic1", "tier3s"):
    run(f"{OUT}/wall/ic1/tier3s/{fname(name)}", "ic1", name, IC1_L, 52, "--first-year")

# ---- the commons
for c in ("c1", "c2"):
    Lc = COMMONS_L[c]
    base = f"{OUT}/commons/{c}"
    for tpy in (52, 12, 365):
        job(1000, f"{base}/info/el{tpy}", "elasticity", "--inst", c, "--tpy", str(tpy))
        run(f"{base}/modea/{tpy}", c, "hold", Lc[tpy], Lc[tpy], "--tpy", str(tpy))
    job(1000, f"{base}/info/point", "point", "--inst", c)
    L = Lc[52]
    bat = listing(c, "battery")
    assert len(bat) == 115, len(bat)
    for name, tier, _ in bat:
        run(f"{base}/L/{fname(name)}", c, name, L, 520)
        if tier == "3":
            run(f"{base}/10L/{fname(name)}", c, name, 10 * L, 5200)
        run(f"{base}/hold/{fname(name)}", c, name, L, 520, "--one-sided", "hold")
        run(f"{base}/tilt1/{fname(name)}", c, name, L, 520, "--set", "tilt.*=1")
    t3s = listing(c, "tier3s")
    assert len(t3s) == 24, len(t3s)
    for name, _, _ in t3s:
        run(f"{base}/tier3s/{fname(name)}", c, name, L, 520, "--first-year")
        run(f"{base}/tier3s10L/{fname(name)}", c, name, 10 * L, 5200, "--first-year")
    for fam, n in (("stocks", 41), ("enclose", 4), ("joint2", 60), ("joint4", 40),
                   ("basin", 430)):
        names = listing(c, fam)
        assert len(names) == n, (fam, len(names))
        for name, _, _ in names:
            run(f"{base}/{fam}/{fname(name)}", c, name, L, 520)
    hist = listing(c, "history")
    assert [h[0] for h in hist] == ["cycle(land.mach,1500,80)", "cycle(commons,1500,80)"], hist
    for name, _, _ in hist:
        run(f"{base}/history/{fname(name)}", c, name, 81 * 1500, 1)
    for tpy, every in ((12, 120), (365, 3650)):
        tl = [(n, t) for n, t, _ in listing(c, "battery", "--tpy", str(tpy)) if t in ("1", "2")]
        assert len(tl) == 72
        for name, _ in tl:
            run(f"{base}/tpy{tpy}/{fname(name)}", c, name, Lc[tpy], every, "--tpy", str(tpy))
    for target in ["hold"] + COMMONS_TARGETS[c]:
        d = f"{base}/kick/{fname(target)}"
        job(16 * 1.25 * L, d, "kick", target, "--inst", c, "--ticks", str(L), "--horizon", str(L),
            "--csv", d)
    for tpy in (12, 365):
        d = f"{base}/kick/hold-{tpy}"
        job(16 * Lc[tpy], d, "kick", "hold", "--inst", c, "--tpy", str(tpy), "--ticks",
            str(Lc[tpy]), "--horizon", str(Lc[tpy]), "--csv", d)
neg = listing("c1n", "battery")
assert len(neg) == 115
for name, _, _ in neg:
    run(f"{OUT}/commons/c1n/L/{fname(name)}", "c1n", name, COMMONS_L["c1"][52], 520)

jobs.sort(key=lambda j: -j[0])
for _, cmd in jobs:
    print(cmd)
print(f"{len(jobs)} jobs, {sum(c for c, _ in jobs) / 1e6:.1f} million ticks", file=sys.stderr)
