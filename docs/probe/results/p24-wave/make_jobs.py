"""P2.4 (label run, 2026-09-30): the job list of the scored waves of the three rules built in
Phase 2 proper's second session, each as its registration names it:

- trap: the subsistence trap's remedy, participation at a rate (docs/probe/trap/SPEC.md §8,
  E2-E10; registration docs/probe/results/trap/registration.md; E0-E2 passed at P2.4.6);
- switch: the type switch at the wall, the migration rule (docs/probe/switch/SPEC.md §7, E1-E9;
  registration docs/probe/results/switch/registration.md; E0-E2 passed at P2.4.9);
- free: a zero price markets can hold, the free step (docs/probe/free/SPEC.md §9.1, E2-E5;
  registration docs/probe/results/free/registration.md with A1; E0-E2 passed at P2.4.13).

One `markets` process per job (job.sh), each in its own directory under /root/scratch/p24-bcd-runs
(archived, CSVs gzipped, to D:/rustyecon-p24/runs/bcd/ when gathered). Run names come from the
harness's own lists (`markets list`), which name every registered run (README, "Names").

The trap (L: C1P and C1PN 141,000 at 52 a year, 26,000 at 12, 1,113,000 at 365; C2P 142,000,
26,000, 1,116,000):
  trap/<i>/info/            elasticity at 52, 12 and 365 a year; the oracle's point (C1P, C2P, C1PN)
  trap/<i>/modea/<tpy>/     mode A (hold) at L at 52, 12 and 365 a year (E2)
  trap/<i>/kick/<target>/   the kick set at the base and at each of the 12 dated cost targets (C1P,
                            C2P; E2-E3), at the base at 12 and 365 a year (reported), and C1PN's base
  trap/<i>/L/<run>/         the battery, 115 runs (E3)
  trap/<i>/10L/<run>/       Tier 3 (43) at 10 L (E3)
  trap/<i>/tier3s/, tier3s10L/   Tier 3S (24) at L and 10 L, --first-year (E3)
  trap/<i>/stocks/, pace/   stocks (41) and pace (5), --first-year (E4; registration §3)
  trap/<i>/joint2/, joint4/, basin/   (E5)
  trap/<i>/history/         cycle(land.mach,1500,80) and cycle(commons,1500,80), 81 x 1,500 ticks,
                            CSV every tick (E6)
  trap/<i>/hold/, tilt1/    the battery with one-sided Hold, and with every tilt 1 (E7)
  trap/<i>/tpy12/, tpy365/  Tiers 1-2 at 12 and 365 a year at their L (E8)
  trap/<i>/enclose/         enclosure, 4 (E9)
  trap/c1pn/L/<run>/        C1PN's battery at C1's L (E9)
  trap/ctl/c1/<setting>/<run>/     the controls: C1 as registered, Tier 3 at the 17 dial settings (E9)
  trap/ctl/<i>/tilt2/<run>/        the controls: C1P and C2P, Tier 3 at every tilt 2 (E9)
  trap/<i>/dial/<setting>/<run>/   Tier 3 of C1P and C2P at the 17 dial settings (E10)
The switch (L: IS1 22,000 at 52 a year, 20,000 at 12, 164,000 at 365; IS2 24,000, 20,000,
183,000):
  switch/<i>/info/          elasticity at 52, 12, 365; the oracle's point at 52, 12, 365 (E2, E9)
  switch/<i>/modea/<tpy>/   mode A at L at 52, 12 and 365 a year (E2)
  switch/<i>/kick/<target>/ the kick set at the base and at each of the 12 dated cost targets (E4)
  switch/<i>/L/<run>/       the battery, 109 runs (E3, E5, E6)
  switch/<i>/tier3s/<run>/  Tier 3S (20), --first-year (E3)
  switch/<i>/10L/<run>/     Tier 3 (41) and Tier 3S (20, --first-year) at 10 L (E3)
  switch/is1/stocks/, joint2/, joint4/, basin/   (E7)
  switch/<i>/tpy12/, tpy365/, hold/   Tiers 1-2 at 12 and 365 a year, and under one-sided Hold (E7)
  switch/<i>/nbhd/<setting>/<run>/    Tier 3 at the 17 dial settings (E7)
  switch/is1/swrate/<f>/<run>/        Tier 3 at rate.switch.* x 0.75, 0.9, 1.1, 1.25 (E7)
  switch/is1/corner/<run>/  sw[trained]=0.5, sw[master]=0.2, RW(2) and x*/2 at L, CSV every tick:
                            a walled pop's share moves by exactly e^(k g) a tick (E4)
  switch/is1/env/<run>/     hold and each posted price x (1 +- 1e-9) at genesis, 22,000 ticks, CSV
                            every tick: the kick envelope's decay at the base (E4); and the same
                            with tail.services=0.11@genesis in every run (E4, reported; README)
The free step (L: IL1 56,000 at 52 a year, 13,000 at 12, 390,000 at 365; CT2 141,000, 26,000,
1,114,000):
  free/<i>/info/            elasticity at 52, 12, 365; the oracle's point
  free/<i>/modea/<tpy>/     mode A at L at 52, 12 and 365 a year (E2)
  free/<i>/kick/<target>/   the kick set at the base and at each of the 12 dated cost targets (E3)
  free/<i>/L/<run>/         the battery (IL1 100, CT2 115) (E3)
  free/<i>/10L/<run>/       Tier 3 (IL1 38, CT2 43) at 10 L (E3)
  free/<i>/tier3s/, tier3s10L/   Tier 3S (22, 24) at L and 10 L, --first-year (E3)
  free/<i>/dial/<setting>/<run>/ Tier 3 at the 17 dial settings (E4)
  free/<i>/stocks/, joint2/, joint4/   (E5)
  free/<i>/tpy12/, tpy365/  Tiers 1-2 at 12 and 365 a year at their L (E5)

CSV rows: every tick in the histories and the envelope runs; elsewhere about a hundred a run (every
L/100; the last row is always written).
The list is sorted by cost, largest first. Usage (WSL): python3 make_jobs.py > jobs.txt"""
import os
import shlex
import subprocess
import sys

B = os.environ.get("MARKETS", "/root/scratch/p24-bcd/markets")
JOB = "/mnt/d/rustyecon-wt/p24/docs/probe/results/p24-wave/job.sh"
OUT = "/root/scratch/p24-bcd-runs"
# the engine's own file_name (crates/probe/src/bin/markets.rs)
TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}

SETTINGS = ([f"rate.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"buffer.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"adjust.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"tilt.*={v}" for v in ("0.05", "0.1", "0.25", "0.5", "1")])
TRAP_L = {"c1p": {52: 141000, 12: 26000, 365: 1113000},
          "c2p": {52: 142000, 12: 26000, 365: 1116000},
          "c1pn": {52: 141000, 12: 26000, 365: 1113000}}
SWITCH_L = {"is1": {52: 22000, 12: 20000, 365: 164000},
            "is2": {52: 24000, 12: 20000, 365: 183000}}
FREE_L = {"il1": {52: 56000, 12: 13000, 365: 390000},
          "ct2": {52: 141000, 12: 26000, 365: 1114000}}


def targets(pairs):
    return [f"{c}={v}@dated" for c, vs in pairs for v in vs]


TRAP_TARGETS = {
    "c1p": targets((("land.mach", ("0.44", "0.36", "0.8", "0.2")), ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                    ("commons", ("26.73", "21.87", "48.6", "12.15")))),
    "c2p": targets((("land.mach", ("0.44", "0.36", "0.8", "0.2")), ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                    ("commons", ("34.32", "28.08", "62.4", "15.6"))))}
SWITCH_TARGETS = targets((("land.mach", ("0.44", "0.36", "0.8", "0.2")),
                          ("tail.services", ("0.11", "0.09", "0.2", "0.05")),
                          ("res.services.trained", ("0.044", "0.036", "0.08", "0.02"))))
FREE_TARGETS = {
    "il1": targets((("land.mach", ("0.44", "0.36", "0.8", "0.2")), ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                    ("inst.land", ("572", "468", "1040", "260")))),
    "ct2": targets((("land.mach", ("0.44", "0.36", "0.8", "0.2")), ("b.food", ("0.66", "0.54", "1.2", "0.3")),
                    ("exit.To", ("21.45", "17.55", "39", "9.75"))))}
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


def kick(d, inst, target, L, *opts):
    job(16 * 1.25 * L, d, "kick", target, "--inst", inst, *opts, "--ticks", str(L), "--horizon",
        str(L), "--csv", d)


def info(base, inst, Ls, points_at=(52,)):
    for tpy in (52, 12, 365):
        job(1000, f"{base}/info/el{tpy}", "elasticity", "--inst", inst, "--tpy", str(tpy))
        run(f"{base}/modea/{tpy}", inst, "hold", Ls[tpy], max(1, Ls[tpy] // 100), "--tpy", str(tpy))
    for tpy in points_at:
        job(1000, f"{base}/info/point{tpy}", "point", "--inst", inst, "--tpy", str(tpy))


def need(n, rows, what):
    assert len(rows) == n, (what, len(rows))
    return rows


# ---------------------------------------------------------------- the trap
for i in ("c1p", "c2p", "c1pn"):
    info(f"{OUT}/trap/{i}", i, TRAP_L[i])
for i in ("c1p", "c2p"):
    base = f"{OUT}/trap/{i}"
    L = TRAP_L[i][52]
    ev, ev10 = L // 100, L // 10
    for t in ["hold"] + TRAP_TARGETS[i]:
        kick(f"{base}/kick/{fname(t)}", i, t, L)
    for tpy in (12, 365):
        kick(f"{base}/kick/hold-{tpy}", i, "hold", TRAP_L[i][tpy], "--tpy", str(tpy))
    bat = need(115, listing(i, "battery"), (i, "battery"))
    for name, tier, _ in bat:
        run(f"{base}/L/{fname(name)}", i, name, L, ev)
        run(f"{base}/hold/{fname(name)}", i, name, L, ev, "--one-sided", "hold")
        run(f"{base}/tilt1/{fname(name)}", i, name, L, ev, "--set", "tilt.*=1")
        if tier == "3":
            run(f"{base}/10L/{fname(name)}", i, name, 10 * L, ev10)
            run(f"{OUT}/trap/ctl/{i}/tilt2/{fname(name)}", i, name, L, ev, "--set", "tilt.*=2")
            for s in SETTINGS:
                run(f"{base}/dial/{fname(s)}/{fname(name)}", i, name, L, ev, "--set", s)
    for name, _, _ in need(24, listing(i, "tier3s"), (i, "tier3s")):
        run(f"{base}/tier3s/{fname(name)}", i, name, L, ev, "--first-year")
        run(f"{base}/tier3s10L/{fname(name)}", i, name, 10 * L, ev10, "--first-year")
    for fam, n, fy in (("stocks", 41, True), ("pace", 5, True), ("joint2", 60, False),
                       ("joint4", 40, False), ("basin", 430, False), ("enclose", 4, False)):
        for name, _, _ in need(n, listing(i, fam), (i, fam)):
            run(f"{base}/{fam}/{fname(name)}", i, name, L, ev, *(("--first-year",) if fy else ()))
    hist = listing(i, "history")
    assert [h[0] for h in hist] == ["cycle(land.mach,1500,80)", "cycle(commons,1500,80)"], hist
    for name, _, _ in hist:
        run(f"{base}/history/{fname(name)}", i, name, 81 * 1500, 1)
    for tpy in (12, 365):
        Lt = TRAP_L[i][tpy]
        tl = [(n, t) for n, t, _ in listing(i, "battery", "--tpy", str(tpy)) if t in ("1", "2")]
        need(72, tl, (i, tpy))
        for name, _ in tl:
            run(f"{base}/tpy{tpy}/{fname(name)}", i, name, Lt, Lt // 100, "--tpy", str(tpy))
kick(f"{OUT}/trap/c1pn/kick/hold", "c1pn", "hold", TRAP_L["c1pn"][52])
for name, _, _ in need(115, listing("c1pn", "battery"), "c1pn"):
    run(f"{OUT}/trap/c1pn/L/{fname(name)}", "c1pn", name, TRAP_L["c1pn"][52], 1410)
for name, tier, _ in need(115, listing("c1", "battery"), "c1"):
    if tier == "3":
        for s in SETTINGS:
            run(f"{OUT}/trap/ctl/c1/{fname(s)}/{fname(name)}", "c1", name, 141000, 1410, "--set", s)

# ---------------------------------------------------------------- the switch
for i in ("is1", "is2"):
    base = f"{OUT}/switch/{i}"
    Ls = SWITCH_L[i]
    L = Ls[52]
    info(base, i, Ls, points_at=(52, 12, 365))
    for t in ["hold"] + SWITCH_TARGETS:
        kick(f"{base}/kick/{fname(t)}", i, t, L)
    bat = need(109, listing(i, "battery"), (i, "battery"))
    for name, tier, _ in bat:
        run(f"{base}/L/{fname(name)}", i, name, L, L // 100)
        if tier == "3":
            run(f"{base}/10L/{fname(name)}", i, name, 10 * L, L // 10)
            for s in SETTINGS:
                run(f"{base}/nbhd/{fname(s)}/{fname(name)}", i, name, L, L // 100, "--set", s)
            if i == "is1":
                for f in ("0.75", "0.9", "1.1", "1.25"):
                    s = f"rate.switch.*={f}"
                    run(f"{base}/swrate/{fname(s)}/{fname(name)}", i, name, L, L // 100, "--set", s)
        if tier in ("1", "2"):
            run(f"{base}/hold/{fname(name)}", i, name, L, L // 100, "--one-sided", "hold")
    for name, _, _ in need(20, listing(i, "tier3s"), (i, "tier3s")):
        run(f"{base}/tier3s/{fname(name)}", i, name, L, L // 100, "--first-year")
        run(f"{base}/10L/{fname(name)}", i, name, 10 * L, L // 10, "--first-year")
    for tpy in (12, 365):
        tl = [(n, t) for n, t, _ in listing(i, "battery", "--tpy", str(tpy)) if t in ("1", "2")]
        need(68, tl, (i, tpy))
        for name, _ in tl:
            run(f"{base}/tpy{tpy}/{fname(name)}", i, name, Ls[tpy], Ls[tpy] // 100, "--tpy", str(tpy))
base = f"{OUT}/switch/is1"
for fam, n in (("stocks", 31), ("joint2", 60), ("joint4", 40), ("basin", 516)):
    for name, _, _ in need(n, listing("is1", fam), ("is1", fam)):
        run(f"{base}/{fam}/{fname(name)}", "is1", name, 22000, 220)
for name in ("sw[trained]=0.5", "sw[master]=0.2", "RW(2)", "x*/2"):
    run(f"{base}/corner/{fname(name)}", "is1", name, 22000, 1)
for pre, d in (("", "env"), ("tail.services=0.11@genesis+", "env-ts011")):
    run(f"{base}/{d}/hold", "is1", pre + "hold" if pre == "" else pre.rstrip("+"), 22000, 1)
    for m in WALL_MARKETS:
        for f in ("1.000000001", "0.999999999"):
            name = f"{pre}p[{m}]*{f}"
            run(f"{base}/{d}/{fname(name)}", "is1", name, 22000, 1)

# ---------------------------------------------------------------- the free step
for i, nbat, n3, n3s, nst in (("il1", 100, 38, 22, 39), ("ct2", 115, 43, 24, 41)):
    base = f"{OUT}/free/{i}"
    Ls = FREE_L[i]
    L = Ls[52]
    ev, ev10 = L // 100, L // 10
    info(base, i, Ls)
    for t in ["hold"] + FREE_TARGETS[i]:
        kick(f"{base}/kick/{fname(t)}", i, t, L)
    bat = need(nbat, listing(i, "battery"), (i, "battery"))
    assert sum(1 for _, t, _ in bat if t == "3") == n3
    for name, tier, _ in bat:
        run(f"{base}/L/{fname(name)}", i, name, L, ev)
        if tier == "3":
            run(f"{base}/10L/{fname(name)}", i, name, 10 * L, ev10)
            for s in SETTINGS:
                run(f"{base}/dial/{fname(s)}/{fname(name)}", i, name, L, ev, "--set", s)
    for name, _, _ in need(n3s, listing(i, "tier3s"), (i, "tier3s")):
        run(f"{base}/tier3s/{fname(name)}", i, name, L, ev, "--first-year")
        run(f"{base}/tier3s10L/{fname(name)}", i, name, 10 * L, ev10, "--first-year")
    for fam, n in (("stocks", nst), ("joint2", 60), ("joint4", 40)):
        for name, _, _ in need(n, listing(i, fam), (i, fam)):
            run(f"{base}/{fam}/{fname(name)}", i, name, L, ev)
    for tpy in (12, 365):
        tl = [(n, t) for n, t, _ in listing(i, "battery", "--tpy", str(tpy)) if t in ("1", "2")]
        need(nbat - n3, tl, (i, tpy))
        for name, _ in tl:
            run(f"{base}/tpy{tpy}/{fname(name)}", i, name, Ls[tpy], max(1, Ls[tpy] // 100), "--tpy",
                str(tpy))

jobs.sort(key=lambda j: -j[0])
for _, cmd in jobs:
    print(cmd)
print(f"{len(jobs)} jobs, {sum(c for c, _ in jobs) / 1e6:.1f} million ticks", file=sys.stderr)
