# The first scored wave of Phase 2 proper: its jobs and its scorer

Dated 2026-09-30. Step P2.3.11 on branch `phase2-proper`, label `run`, scratch
`D:/rustyecon-p23/run/`. This directory is committed before the wave's first job (decision 311):
the job list, the job script, the gather script, the scorer, its self-test, and the readings the
scorer takes of the registered tolerances where the registrations leave a choice. It scores two
registrations, each line by line:

- **the wall**, IW1 and the line control IC1: [../../wall/SPEC.md](../../wall/SPEC.md) §7
  (E2–E9), registered at P2.3.1 ([../wall/registration.md](../wall/registration.md)), with its
  amendment A1 (E0 only), and E0–E2 passed at P2.3.4 ([../wall/e0.md](../wall/e0.md));
- **the open commons**, C1, C2 and the negative control C1N: [../../commons/SPEC.md](../../commons/SPEC.md)
  §5.5–§6.9 (E2–E9), registered at P2.3.5 with Tier 3S ([../commons/registration.md](../commons/registration.md)
  §3), with A1 and A2 (E0 only), and E0–E2 passed at P2.3.9 ([../commons/e0.md](../commons/e0.md)).

The binary is the harness as of P2.3.10, which fixed the wall's basket count before any scored run
(WALL-RULES §4, amended). It is built in WSL release from the commit that adds this directory and
copied to `/root/scratch/p23-run/markets`; its sha256 is recorded in `BIN.sha256` when the wave
starts (the binary cannot exist before the commit it is built from).

## Files

| file | what |
|---|---|
| `make_jobs.py` → `jobs.txt` | the job list: 3,721 jobs, one `markets` process each, sorted by cost; its docstring lists every set and its length |
| `job.sh` | one job: runs the frozen binary with its output in its own directory, and writes the exit code and wall time |
| `gather.py` | reads every job's directory (or the gzipped archive) into `runs.jsonl`, one record a job, and the kick envelope at the wall's base |
| `score.py` | scores `runs.jsonl` against the registered files; writes `lines.csv` (every line) and the tables |
| `selftest.py` | writes the registered mirrors' own outputs as gather writes the engine's, for `score.py` to score |
| `SHA256SUMS` | the sha256 of each of these files |

The wave runs in WSL, 46 jobs at a time on 48 threads: `xargs -P 46 -I{} bash -c '{}' < jobs.txt`.
The raw runs go to `/root/scratch/p23-runs/` and are archived, CSVs gzipped, to
`D:/rustyecon-p23/runs/`; `gather.py` reads either and gives the same `runs.jsonl`.

## What is run

At the wall (L 22,000 at 52 a year, 20,000 at 12, 164,000 at 365; IC1 25,000):
- E2: the elasticity probe at 52, 12 and 365 a year, and mode A (hold) at L at each;
- E3: the registered battery (103) at L; Tier 3S (20) at L with the first year's D̂ as its
  start; Tier 3 (39) and Tier 3S again at 10·L;
- E4: the kick set at the base and at each of the 12 cost targets (the dated run, H = L); the
  envelope runs (below);
- E6: `s[goods]=0.5` for 30,000 ticks;
- E7: stocks (31), joint2 (60), joint4 (40), basin (516); Tiers 1–2 at 12 and 365 a year; Tiers
  1–2 under one-sided `Hold` and with every tilt 1; `cycle(land.mach,1500,80)` for
  80 × 1,500 + 22,000 ticks, the frame's history (`wb.history`);
- E8: IC1's battery (103) and Tier 3S (20) at 25,000;
- E9: the oracle's point at the base and every target (`markets point`).

At the commons (L 141,000 at C1 and C1N, 142,000 at C2; 26,000 at 12 a year; 1,113,000 and
1,116,000 at 365):
- E2: the elasticity probe and mode A at L at 52, 12 and 365 a year; the base kick sets at 52,
  and at 12 and 365 (reported);
- E3: the battery (115) at L; Tier 3 (43) at 10·L; Tier 3S (24) at L and 10·L; the kick set at
  each of the 12 cost targets' dated runs;
- E4: stocks (41); E5: joint2 (60), joint4 (40), basin (430); E6: `cycle(land.mach,1500,80)` and
  `cycle(commons,1500,80)` for 81 × 1,500 ticks, the mirror's `run_cycle`; E7: the battery under
  `Hold` and with every tilt 1; E8: Tiers 1–2 at 12 and 365 a year; E9: `enclose` (4) and C1N's
  battery (115) at C1's L.

**Not in this wave:** O22's families on I1–I3, which decision 368 orders first. This wave is the
two frames' registered protocols; those families have no registered predictions and are
reported only, so their order changes no line here. They are their own wave (open item, next
step 7).

## How the scorer reads each registered tolerance

Every number of the engine is `markets`' own: the summary line, `stats.tsv`, and the CSV's rows.
Nothing in the engine or the harness is changed for the wave.

**The wall** (SPEC §7's bands).
- *Classes exactly:* each run's class against the mirror's run of the same name
  (`registered/runs_<set>.tsv`), in every set.
- *Ticks and years within 10%, three runs a set within 25%:* ticks to tolerance (`in_tol_from`
  against `ticks_to_tol`), run by run, where both are CONVERGED. A run between 10% and 25% is
  charged; at most three a set may be, and beyond them its lines fail. Years are ticks over the
  tick length, the same line. A tier's or family's median and slowest are scored at 10% and
  charge no run.
- *Troughs and lowest depths within 5%:* each run's lowest baskets eaten over Y\* (every
  household; P2.3.10) and lowest depth (`wall.depth_min`), relative; a registered 0 must be 0.
- *Dead-tick and breach-tick counts within 10% or 5 ticks:* each run's dead ticks, its dead ticks
  in each market (`dead.below_floor`, a tick when the market clears less than half its oracle
  volume, which includes no trade; a market is compared where either count is above 0), and its
  breach ticks.
- Reported beside them, not scored: peak D̂, the worst buyer fill (`wall.worst_buyer_fill`), the
  transfer shortfall, the ticks with no baskets, the first breach tick, each desk's largest share,
  each pop's highest participation and saturated ticks, except where E6 states them.
- *Tiers 3 and 3S at 10·L, "the same classes and the same ticks to tolerance":* class exactly and
  ticks within the band above, against the run's registered row at L.
- *E4, the kick sets:* `markets kick` at the base (hold) and at each target's dated run, H = L:
  certify's PASS and the largest gain in the tail at most 1e-3.
- *E4, the fitted decay at the base:* the frame's own measure (`model/wstab.kick`) on the engine's
  runs: `hold` and each posted price × (1 ± 1e-9) at genesis, 22,000 ticks each, every tick in
  the CSV; g(t) the largest |ln(p_kick/p_base)| over the posted prices, the gains g(t)/g(0), their
  suffix maximum, and the decay a tick between its first falls below 1e-1 and 1e-3; the slowest
  of the 14. Its 52nd power must be within 0.03 of 0.534. The harness's kick reports only its
  tail and peak gains, so the envelope is read from `markets run`'s CSV; no code is added.
- *E4 and E6, "exactly 0.951229" and "follow s_t = s_(t−1)·(1 − share(adjust)) exactly":* to
  rounding: in the eight battery runs that displace a share (`s[D]=V` six times, `x*/2`,
  `JB(0.5)`), every s_t/s_(t−1) on a tick whose depth at posted prices is at least 0 and whose
  s_(t−1) is a normal double is within 1e-12 of e^(−0.05), relative.
- *E4, "the mirror's largest root per tick over the targets is 0.991911":* the mirror's; the
  harness has no PL. The refutation's "slowest mode at or above 1 at any target" is read through
  the kick sets, which pass only if every kick decays below 1e-3 of its size over H.
- *E5:* the six runs `land.mach=0.8`, `tail.services=0.2` and `res.services.trained=0.08`, at
  genesis and dated: their troughs and dead ticks against their registered rows.
- *E6:* only `JB(0.5)` breaches in the battery (the set of breaching runs exactly), its breach
  ticks (10% or 5), first breach tick (exactly 0), lowest depth and largest share (5%); every
  other run's lowest depth within 5% of the mirror's least, 0.128, and at a `land.mach=0.8` run;
  the thresholds back within tolerance (1 + the last tick either displaced desk's threshold gap
  exceeds 1e-3) at 78, 105 and 124 in `s[D]=0.05`, `0.2`, `0.5`, 124 in `x*/2` and 102 in
  `JB(0.5)`, within 10%; no saturated tick in the battery; every CONVERGED run of every IW1 set
  ends at the wall (each desk's share on the last row 0.0 or at most 5e-323, each threshold 1.0)
  with every labour market clearing above 0; in the battery's runs without a breach a share not
  displaced at genesis never leaves 0.0 (`wall.share_max`), and a displaced one, and both of
  `JB(0.5)`'s, ends at 5e-323 exactly; and `s[goods]=0.5` is at 5e-323 after 30,000 ticks.
- *E7:* every family run in the bands above, and each set's count of CONVERGED runs. *Hold
  identical to Saturate:* each Tiers 1–2 run under `Hold` equals the same run under `Saturate` in
  class, every summary number and every statistic. *History:* per window as `wb.history` reads it
  (1,500 ticks, the last 22,000): windows 1–80 in tolerance at their end; ticks to tolerance
  (charged per set), dead ticks, the lowest baskets over Y\* (every household over the target's
  output of services, z 1) and the lowest depth in the bands; and a tick with no baskets in each
  of the 16 × 0.5 windows.
- *E8:* IC1's classes are scored (103/103 and 20/20 CONVERGED); its numbers are compared and
  reported, "reported, not scored".
- *E9:* each gap of SPEC §2.4 from `markets point`'s doubles at the base and each target:
  ln(v/v(1)) with v(1) = γ(1)·b/(1 − a − γ(1)·λ); ln(v/(γ(1)·p_mach)); ln(ω_max/ω) with
  ω = v/P_s, C = (w_T/P_s)·R_T + (w_M/P_s)·R_M and ω_max = (1 − C)/(n_D/Y); ln(S(ω_max)/n_D) with
  S = N_E·min(ln1p(ω_max)/χ_E, 1); ln(N_E/n_D); ln(N_i/D_i); ln(w_i/(ε_i·v)) with E's ε 1.5 and
  1.8. Each within 1e-12 of `registered/edges.json`, and the least over the targets at least 0.2.
  The self-test builds the point from the frame's registered oracle points and passes all 117.
- *E2 (again, in the wave):* mode A PASS at L at 52, 12 and 365 a year with its largest gap below
  1e-9, and L from the engine's elasticity probe equal to the registered one.
- *The refutations* (SPEC §7): a class change in the registered battery; a CONVERGED run, in any
  set but the envelope's, whose last tick's largest gap exceeds 1e-12 in log; a converged run
  ending off the wall, or with a labour market not trading at the end; a kick set failing; and
  (from P2.3.4 and the gates) the rest point and the pins.
- *The verdict* (FW9, decision 397): GO when mode A passes at 52 a year, every run of Tiers 1–3 and
  3S is CONVERGED at L, Tiers 3 and 3S again at 10·L, and every kick set passes.

**The commons** (SPEC §5.6's bands).
- *Every class exactly*, VACUOUS included, in every set; the names are the engine's
  (registration §3: `exit.To` is `commons`, a desk's coin `coin.desk.<d>`). Tier 3S is
  `tier3s/tier3s.jsonl`'s runner, with its `cls_engine_rule` as its class.
- *Ticks to tolerance within 10%, up to three runs an instance within 25%:* run by run where both
  are CONVERGED. The allowance is counted per instance and step (E3 to E9), since each step is
  scored as its own line; E3 is the battery, Tier 3 at 10·L and Tier 3S at L and 10·L. A tier's or
  family's median and slowest (non-slack runs, `predict_summary.tier_stats`) at 10%, charging none.
- *Lowest baskets and cleared volumes within 0.05 absolute where the mirror's is above 0.1:* each
  run's lowest baskets over Y\* and each market's lowest cleared volume over its oracle volume;
  below 0.1, reported.
- *Dead ticks within 10% or 5.*
- *Regime ticks and switches within 10% or 2:* the ticks in each of the mirror's regimes
  (Commons, Crowded with the split, Enclosed) and the switches between them. The mirror stops a
  run once it has stayed within 1e-6 of its target for 2,000 ticks; its counts end there, while
  the engine runs to L. The regime the mirror ended in is carried to L: the engine's ticks in it
  are compared with the mirror's plus L less the mirror's ticks run.
- *r_o at the end within 1e-9 relative (Crowded targets):* where the mirror's target regime is
  Crowded and both runs are CONVERGED, the engine's r_o/r on its last tick (the CSV's
  `ro_over_r`, full precision) against the mirror's `ro_end`.
- *Runaway ticks of the trap within 5%:* a DIVERGED run's tick leaving the runaway bound, less
  the scored clock's start, against the mirror's `runaway at t`; for the history, the tick of the
  run (window × 1,500 + t).
- *§6.1 and §6.3:* each tier's classes equal the mirror's; each tier's medians and slowest at 10%,
  worst dead ticks at 10% or 5, lowest baskets and each market's lowest cleared volume at 0.05;
  the rest reported. The worst fill is reported: the engine's summary reads buyers and sellers,
  the mirror's buyers only.
- *§6.2:* every kick set at every target passes (13 an instance). The largest root is the
  mirror's (no PL in the harness).
- *§6.5 and §6.8:* each family's classes, and the names of its DIVERGED runs, equal the mirror's
  (O106: E0 found the joint draws equal, so the per-seed predictions hold).
- *§6.6:* per window as `run_cycle` reads it (1,500 ticks): in tolerance at its end as the
  mirror's, ticks to tolerance (10%) and dead ticks (10% or 5), and the land.mach history's runaway
  at C1 within 5% of tick 7,788 of the run.
- *§6.7:* every CONVERGED battery run ends in its target's regime (the split counted as
  Crowded), with r_o/r 0.0 exactly at a Commons target and 1.0 exactly at an Enclosed one.
- *E2 (again):* mode A at 52 a year PASS below 1e-9 (12 and 365 reported, as registered); L from
  the engine's probe equal to the registered; the base kick set at 52 a year passes (12 and 365
  reported).
- *The refutations* (§6.9): a class change in E3; a CONVERGED run whose last tick's largest gap
  exceeds 1e-12 in log (read as "rests off the oracle's point", the wall's number); the trap in any
  E3 run; none of the trap's predicted runs (E5, E6, E9) falling into it.
- *The verdict* (§5.5 with Tier 3S): GO when mode A passes, every non-vacuous run of Tiers 1–3 and
  Tier 3S is CONVERGED at L, Tier 3 and 3S again at 10·L, the VACUOUS runs are the registered
  ones, and every kick set passes; LOCAL when mode A and Tiers 1–2 hold and a Tier-3 run does not.

A line's status is pass, fail, charged (between 10% and 25%, within the allowance), reported, or
missing (no engine number, which fails a scored line).

## The self-test, and the runs made before the wave

`selftest.py` writes the registered files as runs.jsonl records and `score.py` scores them:
37,184 lines pass, 22,591 are reported, and 13 are missing, the lines that need a per-tick CSV
the registered files do not hold (the thresholds' return, the shares' decay and end values, the
end-of-run checks, the 30,000-tick share) or the engine's own measure (the envelope's rate is
there; the technique's rate is not). Its E9 builds the point from the frame's registered oracle
points and reproduces all 117 of `edges.json`'s gaps within 1e-12.

Before this commit, to test `job.sh` and `gather.py`, ten jobs were run that are not scored in
the wave or were run at E0 or E2 already: mode A at IW1 and C1 at 52 a year; the point and the
elasticity probe at IW1 and the point at C1; C1's base kick set at 52 a year (E2, P2.3.9); E0's
`s[goods]=0.5` and `land.mach=0.8@genesis` at IW1 for 2,000 ticks; and `hold` and
`p[labour]*1.000000001` at IW1 for 300 ticks, too short for the envelope's decay to form a rate
(none was). Their records were checked for their fields, and `score.py` was run on them to see
that it runs on the engine's records; on ten records nearly every line is missing, and none of
its numbers is a result of the wave (`D:/rustyecon-p23/run/gtest/`, `/root/scratch/p23-gtest/`).
The E9 lines it printed are the oracle's point, which the build already checks against the
frame's registered points bit for bit (`markets_iw1_genesis_is_unit_1d`).
