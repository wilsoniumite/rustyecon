# The wave's rerun (P2.3.12), after a harness bug the wave found

Dated 2026-09-30, label `run`. Twelve minutes into the wave, the first jobs to finish
included twelve errors, all the same: a dated shock of the commons could not load ("cannot set a
FlowPerYear param from a Dimensionless param"). The tape writer wrote each dated shock's schedule
param `Dimensionless`; `inst.commons` and `inst.land` are `FlowPerYear`. So every job with
`commons=V@dated`, `enclose=F@dated` or `cycle(commons,P,N)` fails before its first tick: 54 jobs
of the 3,721, all at C1, C2 and C1N (the battery's four dated commons shocks, Tier 3's two at 10·L,
Hold's and tilt 1's four, 12 and 365 a year's two each, the kick sets at the four dated commons
targets, enclosure's two dated runs and the commons history, at each instance; C1N's four). No wall
job is touched: its coefficients are `Dimensionless`.

The fix (P2.3.12, COMMONS-RULES §4, amended) gives a schedule param the unit of the param it sets;
every other tape is the same text. It is committed with its test, the gates green on both
machines, before the rerun. The wave itself ran to its end on the P2.3.11 binary, and nothing
else in it was changed. Then, in this directory:
- `make_rerun.py` → `jobs-rerun.txt`: the 54 jobs, the same lines but for the job script
  (`rerun/job.sh`, the binary built from P2.3.12's commit, `/root/scratch/p23-run2/markets`). Their
  failed outputs are kept aside (`/root/scratch/p23-runs-failed/`, archived as
  `D:/rustyecon-p23/runs-failed/`), and the reruns write the same directories, which `gather.py`
  reads as any other.
- `jobs-r1.txt` and `r1check.sh`: R1 for the fix. 24 jobs the bug did not touch (dated shocks of
  `Dimensionless` coefficients, genesis commons shocks, enclosure at genesis, the land.mach
  history, joint, basin, Tier 3S, C1N and the wall's, and two kick sets), run again on the new
  binary; every file they write must equal the wave's byte for byte.
- The scorer, the gather script and the job list are unchanged.
