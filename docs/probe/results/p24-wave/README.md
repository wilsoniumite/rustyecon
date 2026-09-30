# The scored waves of the three built rules: the trap's remedy, the type switch, the free step

Dated 2026-09-30. Step P2.4.14 on branch `phase2-s2`, label `run`, scratch
`D:/rustyecon-p24/run/`. This directory is committed before the first job of any of these waves
(decision 311): the job list, the job script, the runner, the gather script, the three scorers,
their self-test and its output. Each scorer scores one registration, line by line:

- **trap** (`score_trap.py`): the subsistence trap's remedy, participation at a rate.
  [../../trap/SPEC.md](../../trap/SPEC.md) §8–§9 (E2–E10), registered at P2.4.4
  ([../trap/registration.md](../trap/registration.md)); E0–E2 passed at P2.4.6
  ([../trap/e0.md](../trap/e0.md)).
- **switch** (`score_switch.py`): the type switch at the wall, the migration rule.
  [../../switch/SPEC.md](../../switch/SPEC.md) §7 (E1–E9), registered at P2.4.7
  ([../switch/registration.md](../switch/registration.md)); E0–E2 passed at P2.4.9
  ([../switch/e0.md](../switch/e0.md)).
- **free** (`score_free.py`): a zero price markets can hold, the free step.
  [../../free/SPEC.md](../../free/SPEC.md) §9 (E2–E5), registered at P2.4.10
  ([../free/registration.md](../free/registration.md)) with amendment A1 (E0 only, P2.4.12);
  E0–E2 passed at P2.4.13 ([../free/e0.md](../free/e0.md)); decision 424.

The binary is the harness `markets` as of this commit. No engine or harness code changed after
P2.4.11 (`733ee63`), so it is that build's code: the free E0's binary, built in
`/root/scratch/target-p24` at `f5fd1c5`, has sha256 `aa97c626…88d4`. The wave builds it again from
a `git archive` export of this commit, in its own target directory (FIXES.md's hazard), and copies
it to `/root/scratch/p24-bcd/markets`; its sha256 goes in `BIN.sha256` when the wave starts.
Wave A (the families, `../families-wave/`) runs on its own binary, the P2.4.2 build.

## Files

| file | what |
|---|---|
| `make_jobs.py` → `jobs.txt` | the job list, one `markets` process a job, sorted by cost; its docstring lists every set |
| `job.sh` | one job: the frozen binary with its output in its own directory, and the exit code and wall time |
| `run.sh` | the runner: 46 jobs at a time, each line as it is (`xargs -d '\n'`) |
| `gather.py` | reads every job's directory (or the gzipped archive) into `runs.jsonl`, one record a job, and the switch's envelope records |
| `common.py` | the line table, the bands and the formats the scorers share |
| `score_trap.py`, `score_switch.py`, `score_free.py` | the scorers; each writes `lines.csv`, `fails.csv`, its tables and a printout |
| `selftest.py` | writes each registration's mirror records as `gather.py` writes the engine's and scores them, then a negative control each |
| `selftest.out` | the self-test's printout at this commit |
| `SHA256SUMS` | the sha256 of each of these files |

How to run (WSL), from this directory:

```
MARKETS=/root/scratch/p24-bcd/markets python3 make_jobs.py > jobs.txt   # regenerates the committed list
bash run.sh jobs.txt > /root/scratch/p24-bcd/run.log 2>&1
python3 gather.py jobs.txt /root/scratch/p24-bcd-runs /root/scratch/p24-bcd/runs.jsonl
python3 score_trap.py /root/scratch/p24-bcd/runs.jsonl ../trap/
python3 score_switch.py /root/scratch/p24-bcd/runs.jsonl ../switch/
python3 score_free.py /root/scratch/p24-bcd/runs.jsonl ../free/
```

## What is run

`make_jobs.py`'s docstring has every set. In brief, each at its registered L (the elasticity
probe's, as E0–E2 confirmed; IL1 at 12 a year 13,000, decision 424):

- **trap** (4,770 jobs): C1P and C2P's mode A at 52, 12 and 365 a year, their kick sets at the
  base and the 12 dated cost targets (and the base at 12 and 365, reported), the battery (115),
  Tier 3 at 10·L, Tier 3S at L and 10·L, stocks (41) and pace (5), joint2, joint4, basin (430),
  both histories (81 × 1,500 ticks, every tick), Hold, tilt 1, Tiers 1–2 at 12 and 365 a year,
  enclose (4), and Tier 3 at the 17 dial settings (E10); C1PN's mode A, base kick set and battery;
  the controls: the registered C1's Tier 3 at the 17 settings and C1P and C2P's Tier 3 at every
  tilt 2.
- **switch** (3,071 jobs): IS1 and IS2's mode A and elasticity at 52, 12 and 365 a year and the
  oracle's point at each, kick sets at the base and the 12 targets, the battery (109), Tier 3S
  (20), Tiers 3 and 3S at 10·L, Tiers 1–2 at 12 and 365 a year and under Hold, Tier 3 at the 17
  dial settings; IS1's stocks (31), joint2, joint4, basin (516), Tier 3 at `rate.switch.*` × 0.75,
  0.9, 1.1, 1.25, four every-tick runs for the corner rate, and the kick envelope at the base (and
  with tail.services 0.11, reported).
- **free** (2,353 jobs): IL1 and CT2's mode A and elasticity at 52, 12 and 365 a year, the point,
  kick sets at the base and the 12 targets, the battery (100, 115), Tier 3 at 10·L, Tier 3S at L
  and 10·L, Tier 3 at the 17 dial settings, stocks, joint2, joint4, and Tiers 1–2 at 12 and 365 a
  year.

10,194 jobs in all (trap 4,770, switch 3,071, free 2,353), about 1,731 million ticks by the job
list's count (a kick set counted as 20·L). A kick set is one job.

## Readings the scorers take, declared before the waves

Where a registration leaves a choice, is inexact, or cannot be read as written, the scorers take
these readings. None changes a registered number.

1. **Names.** The mirrors' run names are the engine's, but for the trap's `exit.To=` (the engine's
   `commons=`), a desk's coin `coin.<d>*F` (`coin.desk.<d>*F`, trap and free), and the trap's
   histories `cycle(land.mach)` and `cycle(exit.To)` (`cycle(land.mach,1500,80)`,
   `cycle(commons,1500,80)`), as the registrations' §3 name them. Every registered run of every
   set is in the harness's own list (`markets list`), checked on the free E0's build before this
   commit (0 missing).
2. **The 25% allowance's group.** "Three runs a set" (the wall frame) or "three runs an instance"
   (the commons frame, the free step) is read as: at the trap and the free step, an instance's
   step (E3 … E9), and one dial setting of an instance for E10 (E4 at the free step) and the C1
   controls; at the switch, an instance's set, and one dial setting's Tier 3; a history run's
   windows are their own group.
3. **The mirror stops a run early; the engine runs L ticks.** Its end regime's ticks are carried
   to L (the trap's regime ticks, as `p23-wave/score.py` read the commons), and so is a switch
   pop's count of ticks with a above 1e-9 where the mirror stopped with a above 1e-9. The lines
   only an engine run to L gives (every end D̂, the paced share's end, r_o/r at a Crowded end, a
   switch pop's end) are "missing" in the self-test, as the families wave's were.
4. **The switch's walled end (E5).** The registration's "else at most 1e-300" is scored as written.
   Its own full-L run (`evidence/sw_fullL.out`) ends land.mach=0.8's trained at 1.47e-252: a
   pop's share decays by e^(k·g) a tick, and at 22,000 ticks a small gap (land.mach 0.8's
   −0.053) or a slower switch (`rate.*` or `rate.switch.*` × 0.75) leaves it above the stall.
   So that line is expected to fail there by the registration's own evidence. Beside it the
   scorer reads the refutation's criterion, "a switch pop ending on the wrong side of its
   switch": at a walled target the share below 1e-9 with its gap at the end below 0 (or exactly
   0.0).
5. **The switch's never-pooled runs (E1)** are compared with the P2.3 wave's engine IW1 runs
   (`D:/rustyecon-p23/runs/runs.jsonl.gz`, sha256 `f9fc7416…ea79` uncompressed): class and ticks
   to tolerance exactly, peak D̂ "within 1e-9 relative" read at the digits the harness prints
   (`%.6e`), the finest its summary gives.
6. **The switch's envelope (E4).** At IS1's base it is P2.3's measure: hold and each posted price ×
   (1 ± 1e-9) at genesis, every tick, the frame's `wstab.kick` rate, scored against 0.534 a year.
   At tail.services 0.11 the harness cannot start at the target's point without new code, so the
   decay there is measured with the shock and the kick both at genesis and is reported beside
   0.686, not scored. The kick set at that target (scored) is the check of its stability.
7. **The switch's corner rate (E4).** "A switch pop's share moves by exactly e^(k·g) a tick while
   its gap is below 0" is read on four every-tick runs (`sw[trained]=0.5`, `sw[master]=0.2`,
   `RW(2)`, `x*/2`): on every row with a negative gap and a normal previous share,
   |a_t/a_(t−1)/e^(k·g_t) − 1| at most 1e-12. The CSV's row t carries the gap read in tick t and
   the share after it. This alignment was fixed on a 1,000-tick every-tick run of
   `sw[trained]=0.5` on the free E0's build before this commit, which is also a sample of the
   line: read so, its 999 rows give 2.2e-16; read one row apart, 3.3e-2 (disclosed below).
8. **Mode A in the verdicts** is MARKETS-SPEC §7.9's, at 52 a year. At the switch and the free
   step mode A at 12 and 365 is scored as a line too (E2), and at the trap reported (its E2); the
   switch's IS2 is reported throughout (E8). L from the engine's elasticity probe is scored at
   every registered tick length but IL1's at 12 a year (decision 424) and the reported controls.
9. **Kick sets in the verdicts.** A CONVERGED run stands only if its target's kick set decays (the
   commons frame's §5.1; MARKETS-SPEC §7.5), so every verdict asks the base's and the 12 targets'
   kick sets to pass, as each registration predicts (the mirrors' PL: every bar passes).
10. **First-year starts.** Tier 3S and the trap's pace family start from the first year's largest
    D̂ (`--first-year`), and so do the trap's stocks (its registration §3). The switch's and the
    free step's stocks start from genesis, as their mirrors read them (the wall's stocks at P2.3;
    `regH.jsonl` has no first-year start for stocks).
11. **The trap's controls on C1** are the same runs as wave A's C1 Tier 3 at the 17 settings,
    which run on the P2.4.2 binary. The trap's E9 runs them again on this binary, as its
    registration lists them; the two are compared after both waves, reported.

## How each scorer reads its bands

Each scorer's docstring lists its bands, which are its registration's: every class exactly; ticks
to tolerance within 10% (three runs a group within 25%); the troughs (0.05 absolute where the
mirror's is above 0.1 at the commons frames; 5% relative at the wall); dead and breach ticks
within 10% or 5; the regime ticks and switches within 10% or 2; runaway ticks within 5% on the
harness's reference; every CONVERGED run's end D̂ at most 1e-9; and each rule's own readouts (the
trap's paced share and ticks without hours; the switch's live ticks, largest share, end state and
points; the free-able market's end state, first free tick and switches). Each lists its
refutation criteria (the trap's SPEC §8, the switch's §7, the free step's §10) and its verdict
rule. A line's status is pass, fail, charged (between 10% and 25%, within the allowance),
reported, or missing (no engine number: a scored line that is missing fails the wave).

## The self-test

`selftest.py` writes each registration's mirror records as `gather.py` writes the engine's and
scores them with `--self-test` (`selftest.out`). No line fails in any of the three; the only
missing lines are the end-of-run lines (and the switch's every-tick corner line) that only an
engine run to L gives, and no refutation is listed. Then a negative control each, with five
records made wrong on purpose; each scorer fails exactly those lines, the counts and verdicts they
feed, and names the refutations they meet.

## Runs made before this commit

None of these waves' runs. Before this commit, on the free E0's build (`aa97c626…88d4`): the
harness's lists (`markets list`, no tick) for every set; four 3,000-tick format samples
(`sw[trained]=0.5` at IS1, `p[mach]*0.5` at C1P, `p[labour]*2` at IL1 and CT2) and one
1,000-tick every-tick run of `sw[trained]=0.5` at IS1, to read the harness's summary, `stats.tsv`
and CSV columns for the new instances and the CSV's row alignment (reading 7). Their classes and
ticks were not compared with any registered value, and their directories are not kept.
