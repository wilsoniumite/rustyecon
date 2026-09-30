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

## The jobs the wave's runner never started (P2.3.13)

The wave ended at 10:08 (38 minutes) with xargs' exit code 123, and gather found 703 jobs with no
exit file, 701 of the wave's and 2 of the rerun's (and 4 of the R1 check's). The runner,
`xargs -P 46 -I{} bash -c '{}' < jobs.txt`, as README.md names it, strips the quotes of each input
line even with `-I`. So every job whose run name holds a parenthesis (JA, JB, N, RC, RW, `joint`,
`cycle`) reached bash unquoted and was a syntax error before `job.sh` started: no output, no exit
file, nothing seen. Names with brackets and stars reached bash unquoted too and ran, since bash
leaves a glob that matches nothing as it is; `check_names.py` confirms that each of the 2,961
runs that ran printed its job's name. So the R1 check above compared the 20 of its 24 jobs that
ran (76 files, none differing), and it is run again in full.

- `make_missing.py` → `jobs-missing.txt`: every job, from `../jobs.txt`, `jobs-rerun.txt` and
  `jobs-r1.txt`, whose directory has no exit file, 707 in all, each as its list wrote it: the 701
  of the wave on the wave's binary (P2.3.11), the rerun's two (`cycle(commons,1500,80)` at C1 and
  C2) and the R1 check's four on P2.3.12's.
- `run_missing.sh` runs them with `xargs -d '\n'`, which takes each line as it is.
- `check_names.py` checks, on the final `runs.jsonl`, that every run and kick set ran and printed
  its job's name.

The job list, the scorer and the gather script are unchanged; the 707 jobs ran after the wave's
other jobs had finished, and no result of the wave had been scored.
