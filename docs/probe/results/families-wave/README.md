# Wave A of Phase 2 proper's second session: the families wave, its jobs and its scorer

Dated 2026-09-30. Step P2.4.2 on branch `phase2-s2`, label `families`, scratch
`D:/rustyecon-p24/families/`. This directory is committed before the wave's first job (decision
311): the job list, the job script, the runner, the gather script, the scorer, its self-test and
the self-test's output. It scores one registration, line by line:
[../families/registration.md](../families/registration.md) (P2.4.1), which fixes the frame
[../../families/SPEC.md](../../families/SPEC.md) and its registered predictions in
[../../families/registered/](../../families/registered/), made on the mirrors before any run of
this wave. It is the pattern of [../p23-wave/](../p23-wave/README.md), with that wave's fixes
taken in from the start: the runner reads each job line as it is (`xargs -d '\n'`), and one
target directory serves this checkout and commit.

The binary is the harness `markets` as of P2.4.2. No engine or harness code changed in P2.4.1 or
P2.4.2, so it is the P2.3.12 binary (`c4adbf8`) rebuilt: on 2026-09-30 a WSL release build of
`a483ed0` in `/root/scratch/target-p24` gave sha256 `be3266ab…42e9`, the P2.3 rerun's
(`p23-wave/BIN.sha256`). The wave builds it again from the commit that adds this directory and
copies it to `/root/scratch/p24-families/markets`; its sha256 goes in `BIN.sha256` when the wave
starts.

## Files

| file | what |
|---|---|
| `make_jobs.py` → `jobs.txt` | the job list, one `markets` process a job, in the registered priority (stocks first) and by cost within a set; its docstring lists every set |
| `job.sh` | one job: the frozen binary with its output in its own directory, and the exit code and wall time |
| `run.sh` | the runner: 46 jobs at a time, each line as it is |
| `gather.py` | reads every job's directory (or the gzipped archive) into `runs.jsonl`, one record a job |
| `score.py` | scores `runs.jsonl` against `docs/probe/families/registered/`; writes `lines.csv`, `fails.csv`, `families.csv`, `map_verdicts.csv`, `dial.csv` and a printout |
| `selftest.py` | writes the registered mirror records as `gather.py` writes the engine's and scores them, then a negative control |
| `selftest.out` | the self-test's printout at this commit |
| `SHA256SUMS` | the sha256 of each of these files |

How to run the wave (WSL), from this directory:

```
MARKETS=/root/scratch/p24-families/markets python3 make_jobs.py > jobs.txt   # regenerates the committed list
bash run.sh jobs.txt > /root/scratch/p24-families/run.log 2>&1
python3 gather.py jobs.txt /root/scratch/p24-families-runs /root/scratch/p24-families/runs.jsonl
python3 score.py /root/scratch/p24-families/runs.jsonl ../families/
```

## What is run

SPEC §3 in full; in brief:

- **A1, O22's families on I1–I3** (C2m, 52 ticks a year; L 144,000, 52,000 and 484,000, P2.1's),
  stocks first: stocks (41, 29, 50 runs), joint2 (60 each), joint4 (40 each), basin (412, 516,
  509), the history `cycle(<first coefficient>,1500,80)` for 81 windows of 1,500 ticks (every
  tick in the CSV), and the battery (107, 77, 119) under one-sided `Hold` and with every tilt 1;
  and I3's five unrun map cells, `--set rate.*=fr --set buffer.*=ft` for (fr, ft) = (0.5, 0.5),
  (0.5, 1), (0.5, 2), (1, 0.5), (1, 2): mode A, the battery at L = 484,000/fr, and the kick set at
  the base and each of the eight cost targets (dated), H = L.
- **A2, the dial neighbourhood on C1, C2 and IW1** (each instance's registered L at every setting:
  141,000, 142,000, 22,000): Tier 3 (43, 43, 39) and Tier 3S (24, 24, 20; `--first-year`) at each
  of 17 settings, `rate.*`, `buffer.*` and `adjust.*` × 0.75, 0.9, 1.1, 1.25 and `tilt.*` = 0.05,
  0.1, 0.25, 0.5, 1; and the base kick set at each setting, H = L (reported).

6,347 runs and 96 kick sets, 6,443 jobs; about 2,260 million ticks (the job list's count, a kick
set counted as its runs). The committed `jobs.txt` was written on 2026-09-30 from the lists of the
`a483ed0` build in `/root/scratch/target-p24` (`MARKETS=` that binary), whose sha256 is the one
above; the job lines name `/root/scratch/p24-families/markets`, which `job.sh` runs.

## How the scorer reads each registered band

Every number of the engine is `markets`' own: the summary line and, for the history, the CSV's
every-tick rows. The registered values are the mirror records in `registered/` (SPEC §4).

- *Every run's class exactly*, the mirror's, in every set, VACUOUS included.
- *Ticks to tolerance within 10%*, where both runs are CONVERGED (`in_tol_from` against the
  mirror's `ttol`); up to three runs a set within 25% (charged), beyond them a charged run's line
  fails. A set is an instance's family, one I3 map cell, or one instance's dial setting.
- *A DIVERGED run's runaway tick within 5%*, on the scored clock (a dated run's clock starts at
  L/4), against the mirror's on the harness's reference, the displaced genesis prices (O107).
- *Dead ticks within 10% or 5*, whichever is larger, where the classes agree and the run did not
  diverge.
- *The lowest baskets eaten over Y\* within 0.05 absolute* where the mirror's is above 0.1;
  reported below it.
- *The end*: every CONVERGED engine run's last D̂ at most 1e-9 (a gap of 1e-12 in log), since
  no dial of the family moves the oracle's point.
- *The history, window by window* (`fam_i.run_cycle`'s reading, which is the commons'
  `run_cycle`'s): in tolerance at the window's end as the mirror's; ticks to tolerance at 10%
  (three windows an instance within 25%); dead ticks at 10% or 5; and no runaway, as registered.
- *I3's map cells*: mode A PASS; each of the 45 kick sets passing as the mirror's PL predicts
  (PASS iff PL^(0.9·L) ≤ 1e-3); the cell's verdict, MARKETS-SPEC §7.9 with the kick, as
  registered.
- *The dial family's 51 base kick sets*: reported beside the mirror's PL, not scored.
- *The refutation criteria* (SPEC §5.5): a class that is not the mirror's; a CONVERGED run ending
  off its oracle point; a scored kick set or a map cell's verdict that is not the registered one.

A line's status is pass, fail, charged, reported, or missing (no engine number: a scored line
that is missing fails the wave).

## The self-test

`selftest.py` writes the registered mirror records as runs.jsonl records and `score.py
--self-test` scores them (`selftest.out`): 6,443 records, 32,411 lines; 24,903 pass, 1,221 are
reported, none fails, and 6,287 are missing, every one an end-of-run line, which only an engine
run to L gives (the mirror stops a run once it has stayed within 1e-6 of its target for 2,000
ticks). The five map cells read GO and no refutation is listed.

Then a negative control, so the scorer is seen to fail: the same records with nine made wrong on
purpose (a class, ticks to tolerance 30% and 15% off, a runaway tick 10% off, dead ticks off by
20, the lowest baskets off by 0.2, a history window's end, a map kick set, and an end D̂ of 1e-3),
scored as the engine's. Exactly those ten lines fail or are charged (the kick set also changes its
cell's verdict, GO to LOCAL; the 15% tick is charged), and the refutations name the class, the
end, the kick set and the verdict. Both pass at this commit.

## Runs made before this commit

None of this wave's runs. The registration's checks (SPEC §2) ran the mirrors only, and read the
frozen binary's lists and tapes (`markets list`, `markets tape`), which run no tick.

## Files added after the wave (P2.4.16)

- `BIN.sha256`: the wave's binary's sha256 (the P2.4.2 build from a `git archive` export of
  `66ae453`, `be3266ab…42e9`, the registration's) and the wave's `runs.jsonl`'s.
- `score.out`: `score.py`'s printout on that `runs.jsonl`. The scored record is
  [../families/README.md](../families/README.md).

No file listed in `SHA256SUMS` changed; `sha256sum -c SHA256SUMS` passes but for this README,
which gained this section.
