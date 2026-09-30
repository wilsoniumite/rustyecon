"""P2.3.12 (label run, 2026-09-30): the jobs the dated-commons bug stopped, and the R1 check.

The wave's binary (P2.3.11) wrote a dated shock's schedule param as Dimensionless, so every tape
with a dated shock of a FlowPerYear param failed to load: `commons=V@dated`, `enclose=F@dated`
and `cycle(commons,P,N)`, at C1, C2 and C1N (54 of the 3,721 jobs). P2.3.12 gives the schedule
param its target's unit. This script writes:
- jobs-rerun.txt: those 54 jobs, unchanged but for the job script (rerun/job.sh, the binary built
  from P2.3.12's commit), into the same directories (the failed outputs are moved aside first);
- jobs-r1.txt: 24 jobs of the wave the bug did not touch, run again by the new binary into
  /root/scratch/p23-r1check/, whose summary, statistics and CSV must equal the wave's byte for
  byte (every other tape is the same text).
Usage (WSL): python3 make_rerun.py (in this directory)"""
import re

AFFECTED = re.compile(r"commons=[0-9.]+@dated|enclose=[0-9.]+@dated|cycle\(commons")
OLD = "/mnt/d/rustyecon-wt/p23/docs/probe/results/p23-wave/job.sh"
NEW = "/mnt/d/rustyecon-wt/p23/docs/probe/results/p23-wave/rerun/job.sh"
jobs = [l.rstrip("\n") for l in open("../jobs.txt") if l.strip()]
rerun = [l.replace(OLD, NEW, 1) for l in jobs if AFFECTED.search(l)]
assert len(rerun) == 54, len(rerun)
open("jobs-rerun.txt", "w", newline="\n").write("\n".join(rerun) + "\n")
# the R1 check: runs with dated shocks of Dimensionless params and plain runs, at every
# instance, cheap ones (L at most 142,000)
picks = ["/wall/iw1/L/land.mach_0.8_dated ", "/wall/iw1/L/tail.services_0.2_dated ",
         "/wall/iw1/L/JB_0.5_ ", "/wall/iw1/stocks/coin.desk.goodsx0.02 ",
         "/wall/ic1/L/res.services.trained_0.08_dated ", "/wall/iw1/kick/land.mach_0.8_dated ",
         "/commons/c1/L/land.mach_0.8_dated ", "/commons/c1/L/b.food_1.2_dated ",
         "/commons/c1/L/commons_48.6_genesis ", "/commons/c1/L/p_mach_x0.5 ",
         "/commons/c1/enclose/enclose_1_genesis ", "/commons/c1/joint2/joint_2_3_ ",
         "/commons/c1/history/cycle_land.mach_1500_80_ ", "/commons/c2/L/b.food_0.3_dated ",
         "/commons/c2/L/commons_15.6_genesis ", "/commons/c2/L/JB_2_ ",
         "/commons/c2/enclose/enclose_0.5_genesis ", "/commons/c2/tier3s/coin.workersx0.5 ",
         "/commons/c1n/L/p_mach_x0.5 ", "/commons/c1n/L/land.mach_0.8_dated ",
         "/commons/c1/kick/land.mach_0.44_dated ", "/commons/c2/basin/p_labour_x0.12270440108 ",
         "/wall/iw1/basin/p_labour_x0.12270440108003683 ", "/wall/iw1/tier3s/stock.machx0.5 "]
r1 = []
for p in picks:
    hit = [l for l in jobs if p in l]
    assert len(hit) == 1, (p, len(hit))
    r1.append(hit[0].replace(OLD, NEW, 1).replace("/root/scratch/p23-runs/", "/root/scratch/p23-r1check/"))
open("jobs-r1.txt", "w", newline="\n").write("\n".join(r1) + "\n")
print(len(rerun), "rerun jobs;", len(r1), "R1 jobs")
