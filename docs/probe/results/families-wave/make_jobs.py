"""P2.4 (label families, 2026-09-30): the job list of wave A, the families wave: O22's families on
I1-I3 and the dial-neighbourhood family on C1, C2 and IW1, as registered in
docs/probe/families/SPEC.md (docs/probe/results/families/registration.md). One `markets` process
per job (job.sh), each in its own directory under /root/scratch/p24-families-runs (archived,
gzipped, to D:/rustyecon-p24/families/archive/ when gathered). Run names come from the frozen
binary's own lists (`markets list`), which the registration saved and checked.

A1, O22's families on I1-I3 (C2m, 52 a year; L: I1 144,000, I2 52,000, I3 484,000), stocks first:
  i/<inst>/stocks/<run>/     the stocks family (I1 41, I2 29, I3 50)
  i/<inst>/joint2/, joint4/  60 and 40 seeds
  i/<inst>/basin/            I1 412, I2 516, I3 509
  i/<inst>/history/          cycle(<first coefficient>,1500,80) for 81 x 1,500 = 121,500 ticks,
                             CSV every tick (window by window)
  i/<inst>/hold/, tilt1/     the battery (Tiers 1-3) with --one-sided hold, and with
                             --set tilt.*=1
  i/i3/map/<cell>/<run>/     I3's five unrun map cells, --set rate.*=fr --set buffer.*=ft for
                             (fr, ft) = (0.5, 0.5), (0.5, 1), (0.5, 2), (1, 0.5), (1, 2): mode A
                             (hold) and the battery (Tiers 1-3) at L = 484,000/fr
  i/i3/map/<cell>/kick/<target>/   the kick set at the base and each of the eight cost targets
                             (dated), H = L
A2, the dial neighbourhood on C1, C2 and IW1 (each instance's L at every setting: IW1 22,000, C1
141,000, C2 142,000), for each of 17 settings, rate.*, buffer.* and adjust.* x 0.75, 0.9, 1.1,
1.25 and tilt.* = 0.05, 0.1, 0.25, 0.5, 1:
  dial/<inst>/<setting>/<run>/        Tier 3 (IW1 39, C1 and C2 43) and Tier 3S (20, 24, 24, with
                                      --first-year)
  dial/<inst>/<setting>/kick/hold/    the base kick set, H = L (reported)

CSV rows: every tick in the histories; elsewhere about a hundred a run (every L/100, and the last).
The list is in the registered priority (SPEC §4: stocks first), and by cost within a set.
Usage (WSL): python3 make_jobs.py > jobs.txt"""
import os
import shlex
import subprocess
import sys

B = os.environ.get("MARKETS", "/root/scratch/p24-families/markets")
JOB = "/mnt/d/rustyecon-wt/p24/docs/probe/results/families-wave/job.sh"
OUT = "/root/scratch/p24-families-runs"
# the engine's own file_name (crates/probe/src/bin/markets.rs)
TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}

L_I = {"i1": 144000, "i2": 52000, "i3": 484000}
EVERY_I = {"i1": 1440, "i2": 520, "i3": 4840}
L_D = {"iw1": 22000, "c1": 141000, "c2": 142000}
EVERY_D = {"iw1": 220, "c1": 1410, "c2": 1420}
MAP_CELLS = [("0.5", "0.5"), ("0.5", "1"), ("0.5", "2"), ("1", "0.5"), ("1", "2")]
I3_TARGETS = [f"{c}={v}@dated" for c, vs in (("land.power", ("0.55", "0.45", "1", "0.25")),
                                              ("b.food", ("0.66", "0.54", "1.2", "0.3")))
              for v in vs]
SETTINGS = ([f"rate.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"buffer.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"adjust.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"tilt.*={v}" for v in ("0.05", "0.1", "0.25", "0.5", "1")])
SIZES = {("stocks", "i1"): 41, ("stocks", "i2"): 29, ("stocks", "i3"): 50,
         ("joint2", "i1"): 60, ("joint2", "i2"): 60, ("joint2", "i3"): 60,
         ("joint4", "i1"): 40, ("joint4", "i2"): 40, ("joint4", "i3"): 40,
         ("basin", "i1"): 412, ("basin", "i2"): 516, ("basin", "i3"): 509,
         ("battery", "i1"): 107, ("battery", "i2"): 77, ("battery", "i3"): 119}


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


sets = []  # (priority, [(cost, command)])


def new_set():
    sets.append([])
    return sets[-1]


def job(into, cost, d, *args):
    cmd = [JOB, d, *args]
    into.append((cost, " ".join(shlex.quote(c) for c in cmd)))


def run(into, d, inst, name, ticks, every, *opts):
    cost = ticks * (1.25 if "@dated" in name else 1.0)
    job(into, cost, d, "run", name, "--inst", inst, *opts, "--ticks", str(ticks), "--csv", d,
        "--every", str(every))


# ---- A1: O22's families on I1-I3, stocks first
for fam in ("stocks", "joint2", "joint4", "basin"):
    s = new_set()
    for inst in ("i1", "i2", "i3"):
        names = listing(inst, fam)
        assert len(names) == SIZES[(fam, inst)], (fam, inst, len(names))
        for name, _, _ in names:
            run(s, f"{OUT}/i/{inst}/{fam}/{fname(name)}", inst, name, L_I[inst], EVERY_I[inst])
s = new_set()
for inst in ("i1", "i2", "i3"):
    hist = listing(inst, "history")
    assert len(hist) == 1, hist
    run(s, f"{OUT}/i/{inst}/history/{fname(hist[0][0])}", inst, hist[0][0], 81 * 1500, 1)
for variant, opts in (("hold", ("--one-sided", "hold")), ("tilt1", ("--set", "tilt.*=1"))):
    s = new_set()
    for inst in ("i1", "i2", "i3"):
        bat = listing(inst, "battery")
        assert len(bat) == SIZES[("battery", inst)], (inst, len(bat))
        for name, _, _ in bat:
            run(s, f"{OUT}/i/{inst}/{variant}/{fname(name)}", inst, name, L_I[inst],
                EVERY_I[inst], *opts)
s = new_set()
bat = listing("i3", "battery")
for fr, ft in MAP_CELLS:
    cell = f"r{fr}b{ft}"
    opts = ("--set", f"rate.*={fr}", "--set", f"buffer.*={ft}")
    L = int(L_I["i3"] / float(fr))
    every = L // 100
    base = f"{OUT}/i/i3/map/{cell}"
    run(s, f"{base}/modea", "i3", "hold", L, every, *opts)
    for name, _, _ in bat:
        run(s, f"{base}/{fname(name)}", "i3", name, L, every, *opts)
    for target in ["hold"] + I3_TARGETS:
        d = f"{base}/kick/{fname(target)}"
        job(s, 17 * 1.25 * L, d, "kick", target, "--inst", "i3", *opts, "--ticks", str(L),
            "--horizon", str(L), "--csv", d)

# ---- A2: the dial neighbourhood on C1, C2 and IW1
s = new_set()
for inst in ("iw1", "c1", "c2"):
    t3 = [n for n, tier, _ in listing(inst, "battery") if tier == "3"]
    t3s = [n for n, _, _ in listing(inst, "tier3s")]
    assert len(t3) == {"iw1": 39, "c1": 43, "c2": 43}[inst], (inst, len(t3))
    assert len(t3s) == {"iw1": 20, "c1": 24, "c2": 24}[inst], (inst, len(t3s))
    L = L_D[inst]
    for setting in SETTINGS:
        base = f"{OUT}/dial/{inst}/{fname(setting)}"
        opts = ("--set", setting)
        for name in t3:
            run(s, f"{base}/{fname(name)}", inst, name, L, EVERY_D[inst], *opts)
        for name in t3s:
            run(s, f"{base}/{fname(name)}", inst, name, L, EVERY_D[inst], *opts, "--first-year")
        d = f"{base}/kick/hold"
        job(s, 16 * 1.25 * L, d, "kick", "hold", "--inst", inst, *opts, "--ticks", str(L),
            "--horizon", str(L), "--csv", d)

n = 0
cost = 0.0
for s in sets:
    for c, cmd in sorted(s, key=lambda j: -j[0]):
        print(cmd)
        n += 1
        cost += c
print(f"{n} jobs, {cost / 1e6:.1f} million ticks", file=sys.stderr)
