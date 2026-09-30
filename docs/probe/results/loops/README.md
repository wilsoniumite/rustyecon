# The loop step on the engine: the scored runs (E3–E11)

Dated 2026-09-30. Step P2.2b.3 on branch `phase2-plants`, label `run`. Scratch
`D:/rustyecon-p2b/run/`, raw runs `D:/rustyecon-p2b/runs/`. It runs LOOP-SPEC-A1 §A1.5's E3–E11
as LOOPS-RULES §9 names them, and scores each against the registration line by line. E0–E2 came
before, at P2.2b.2 ([e0.md](e0.md)). The report is [LOOPS.md](../../LOOPS.md). **Amended at
P2.2b.4** (label `fix-report`, after the two reviews): the E1 row, "How it ran", the E6, E7 and
E8 sections and the files, each marked; the tables the reviews asked for are in the new section
"Added at P2.2b.4" at the end. No scored line or verdict moved.

Conditions of every number here, unless its line says otherwise: chain8 under rule B, C2g, 52
ticks a year, ρ 0, J_b 1, ex post, ψ 0.25, plants on the fodder desk, the capacity desk and the
maker at θ 0.8, δ_p 10% a year, s1, s_Kp = 2δ_p; L the engine's own (211,000 at horse δ 8%,
202,000 at 10%, 232,000 at 4%, 221,000 with the loop cut); Tier 3 and 3S again at 10·L.

## Verdict

**Every registered prediction holds, and no refutation criterion was hit.** LB1, LB2 and LB3
are GO. So are the flow controls LW1–LW3 and the families LF1, LF2, LF5, LF7, LF8 and LC1. Every
negative control keeps its verdict. The scorer read 1,377 lines: 1,276 pass, 100 are reported
and not scored, and one is not run (LN5, LOOP-SPEC-A2). None fails.

The engine is the mirror to a degree the tolerances do not need. At the 12 GO instances, in
2,052 runs:
- every class is the mirror's;
- every tick to tolerance is the mirror's, to the tick;
- the lowest baskets and troughs are within 2.4e-6 of the mirror's, and the horse price's lowest
  and highest within 1.0e-5;
- dead ticks are the mirror's on fodder, land and labour in every run. On the horse-days and the
  good, 26 runs differ by one tick. Each is a tie at the dead bar (below);
- the withheld ticks and the ticks at labour's bound are the mirror's in every run.

| E | registered | engine | |
|---|---|---|---|
| E1 nesting | tapes, pins and streams unchanged; θ 1 is the plain tape; the fixed plant is drs | P2.2b.2 ([e0.md](e0.md)); this step changes no code. The fixed plant is fixed-Q drs within 1e-12 on every observable but the horse market's volume, which parts by 1.7e-12 and 2.4e-12 (the offer's cancellation; decision 304's reading, taken after E0's first output: e0.md §9) | holds |
| E2 rest point, mode A | fixed point within 1e-12; mode A at L below 1e-9 | P2.2b.2; here mode A passes again at all 12 GO instances, largest gap 5.0e-15 | holds |
| E3 LB1–LB3 | 20/20, 24/24, 25/25 (25/25), 28/28 (28/28), 28/28 each; A1 §7.1 | the same counts; 537 of 537 runs CONVERGED; §7.1 to printed precision but four one-tick ties | holds |
| E4 kick sets | every set decays; base g 0.847, 0.830, 0.901, 0.836 a year ± 0.03; no g above 1 | 24 of 24 sets PASS; base g 0.8478, 0.8310, 0.9008, 0.8463; largest g 0.998971 a tick | holds |
| E5 cost shocks | A1 §7.2 at LB1–LB3, LW1–LW3 | every trough, year, 5% tick and horse price low equal to printed precision, at L and 10·L | holds |
| E6 the glut | heads × 2, × 10 CONVERGED, no dead tick; lows 0.214–0.221, highs 2.02–3.76; withheld 303, 1,315; 221, 1,034; 662, 2,694 | 12 of 12 CONVERGED, 0 dead ticks; lows 0.214–0.221, highs 2.02–3.76; withheld the same, tick for tick | holds |
| E7 pass-through | LB1 r × 2: horse-days ≤ 5 dead (mirror 0), fodder 175, good 11, land 12, lows 0.409 and 0.509; LN7 ≥ 100 horse-day dead ticks, not CONVERGED | LB1: 0, 175, 11, 12, 0.4093, 0.5091, CONVERGED in 50.3 y; LN7: 146, ORBITING | holds |
| E8 flow control | LW1–LW3 GO with A1 §7.1, §7.2, §7.6; b × 2 trough 0.036 higher in log with stocks; 1.36 times the median years | GO each; the numbers equal; 0.0362 higher; 1.361 times (26.5 against 19.5 years) | holds |
| E9 negative controls | each keeps its verdict; counts within 2 of A1; the named outcomes | every verdict kept; counts equal A1's in every tier but LN7's stocks family (0/22 against 1/22); every named outcome holds | holds |
| E10 families | LF1, LF2, LF5, LF7, LF8, LC1 GO with A1's numbers; LF4 needs O51 | GO each, every number equal to printed precision; LF4 not built | holds |
| E11 labour's bound | A1's runs and ticks at the bound; none starts there; each converges | 28 runs, each with A1's ticks and peak; z at tick 0 below 1 in every run; each CONVERGED | holds |

**Refutation criteria** (A1 §A1.5), none hit:
- a class change at LB1–LB3: none, in 537 runs;
- a runaway of the horse price at LB1–LB3 with ψ 0.25: none;
- a converged run ending more than 1e-12 off its oracle point: none; the largest end gap of any
  converged run is 1.9e-13 (LF8), 1.6e-13 at LB1–LB3;
- a θ = 1 tape differing from the plant-free tape: none (E1, P2.2b.2);
- the engine's slowest mode above 1 a tick at any target: none; the largest is 0.998971 (LB3,
  b × 0.5 at genesis), against the mirror's largest root 0.999068.

## How it ran

- **The harness** is P2.2b.2's, committed as `3205025`, built in release on WSL:
  `horses` sha256 `e0e8f2a6e6c293ec38dcb46b699ec94ba3b14043f669170aa04892c2518b893a`
  (`D:/rustyecon-p2b/run/BIN.sha256`). No rule, harness code or protocol changed after the first
  result was read. No engine bug was found, so there is no fix commit and no rerun.
- **The jobs** (`make_jobs.py`, `jobs.txt`): 3,549 runs and 24 kick-set jobs, one `horses`
  process each, 46 at a time on WSL's 48 threads. They took 74 minutes (02:19–03:33), some 2.3
  billion ticks. Every job exited 0, with an empty stderr. A run at 10·L took 220–400 s with 46
  running, against 65 s alone.
- **The runs**, at each instance: mode A (`hold`) at L; the battery (Tiers 1–3S) at L; Tier 3 and
  3S again at 10·L at every instance but the LN ones; the stocks family at L. Beside them:
  heads × 10 at 10·L at LB1–LB3 (E6); heads × 2 and × 10 with `--reserve none` at LB1–LB3, at L
  and 10·L (E9); LB1 at 12 ticks a year, mode A and Tiers 1–2 (E9); and E4's `horses slowest` at
  the base and b × 1.1, 0.9, 2, 0.5 at genesis, and `horses kick hold`, at LB1–LB3 and LW1, each
  with `--ticks L --horizon L`.
- **The run lengths** are the engine's `horses elasticity` (`D:/rustyecon-p2b/runs/lengths/`),
  equal to the mirror's at every instance. LN1, LN3, LN6 and LN7 run at LB1's 211,000, their
  mirror T6 being infinite. LB1 at 12 a year runs 49,000 ticks, LOOPS-RULES §8.6's L·tpy/52
  rounded up (the mirror's); the engine's elasticity term there is 38,000.
- **The scorer** (`score.py`) was written and stamped before any scored output was read
  (`SCORE.sha256`, 02:28:55, with 89 of 3,573 jobs done). It parses A1's tables from the
  registered file, and checks them against the mirror's run-by-run outputs
  (`lm_battery_<ID>.json`, `lm_extra.json`, `lm_kick.json`, copied with their sha256s): all 48
  rows of A1 §7.1 agree. Before it scored the engine it scored the mirror's own outputs, written
  in the engine's columns (`selftest.py`), and every line passed but the kick sets, which the
  self-test has no files for. One disclosure: a timing run of LB1 r × 2 at L was made at 02:17,
  before the scorer, to size the CSVs, and its summary was read. It was run again in the wave.
  *Amended at P2.2b.4 (the measurement review):* the stamp is a scratch file's time, and nothing
  outside the machine fixed the scorer or the job list before the wave began at 02:19:34. Four
  patches edited the scorer while the wave ran, each for what the self-test on the mirror's
  outputs showed (its files are dated 02:28:34–02:28:49):
  - `fix1.py` (02:26:53) keyed the tick lines per run and scored the tiers' medians and largest
    by `tick_agg`, charging no run (decision 305);
  - `fix2.py` (02:28:23) skipped non-finite peaks and baskets in the O14 row;
  - `fix3.py` (02:28:33) let the largest kick g read an empty set;
  - `fix4.py` (02:28:46) fixed `selftest.py`'s `@1L` tag and printed the refutation lines.

  `score.py` was stamped at 02:28:55, with 89 jobs done, and still has that sha256. That no
  engine output was read before the stamp cannot be checked; the timing run above is the one
  read. From P2.2b.4 on, a wave's scorer and job list are committed before it starts (decision
  311).
- **How the tolerances were read.**
  - "Every class exactly": each run's class against the mirror's run of the same name.
  - "Three runs per instance within 25%": a run is charged once when any of its tick or year
    readouts (ticks to tolerance, the heads' and plants' 5% ticks, the withheld ticks) is beyond
    10%. The tiers' medians and largest are scored at 10% but charge no run. None was charged.
  - Dead ticks: 10% or 5 ticks, market by market for fodder, horse-days, the good and land.
    Labour's and the union's are reported.
  - Labour's bound: 10% or 5 ticks; where either peak is within 5% of the bound, not scored.
- **The negative controls** are scored by their class counts per tier, within 2 runs of A1's,
  and the named outcomes of E9. Their run-by-run readouts are reported only
  ([agreement.csv](agreement.csv)).

## The details

### E3, E8 and E10: the GO instances

[verdicts.csv](verdicts.csv), [tiers.csv](tiers.csv), [agreement.csv](agreement.csv),
[battery.csv](battery.csv). Mode A's largest gap in log, at L:

| id | mode A | T1 | T2 | T3 (10·L) | T3S (10·L) | stocks | verdict |
|---|---|---|---|---|---|---|---|
| LB1 | PASS, 8.9e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LB2 | PASS, 2.4e-15 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LB3 | PASS, 5.6e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LW1 | PASS, 2.2e-16 | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | GO |
| LW2 | PASS, 2.0e-15 | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | GO |
| LW3 | PASS, 8.9e-16 | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | GO |
| LF1 | PASS, 8.9e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO |
| LF2 | PASS, 8.9e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO |
| LF5 | PASS, 8.9e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO |
| LF7 | PASS, 1.6e-15 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO |
| LF8 | PASS, 3.3e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO |
| LC1 | PASS, 5.0e-15 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO |

Per tier (A1 §7.1), the years' median and largest, the lowest baskets, each market's most dead
ticks and the horse price's lowest equal A1's to its printed precision, at L and at 10·L, at all
12 instances ([tiers.csv](tiers.csv)). The exceptions are one-tick differences in Tier 3S's dead
ticks:

| id | Tier 3S horse-days (A1) | Tier 3S good (A1) |
|---|---|---|
| LB1 | 38 (37) | 1 (0) |
| LB2 | 34 (33) | 1 (0) |
| LW2 | 5 (5) | 1 (0) |
| LF2, LF5, LF7 | 35 (34), 79 (78), 29 (30) | 1 (0), 1 (0), 1 (0) |
| LC1 | 27 (26) | 0 (0) |

**The one-tick ties.** They come from two runs, `heads.capacity*0.5` (horse-days) and
`coin.desk.good*0.5` (the good), and from `hours.capacity*0.5` at LC1. At tick 1 (tick 0 for
`hours.capacity*0.5`) each leaves its market at exactly half its target: at LB1 the good clears
0.49999999999999983 of Y\* and the horse-days 0.4999999999999999 of X; at LF7 the horse-days
clear 0.5 exactly, which is not dead, where the mirror counts one more. A tick is dead below half
the target, so the last ulp decides it, and the engine and the mirror round apart there. The
dead-tick tolerance is 10% or 5 ticks.

**The lowest baskets (O70).** The engine's reading (each household bound by its own scarcest item)
and the mirror's (the goods and the space eaten over both households) part by at most 2.4e-6 in
any run of the GO instances. No row parts beyond that.

### E4: the kick sets

[kicks.csv](kicks.csv); [fig6](../../figs/loops/fig6_kick_envelopes.png). Each set is 12 kicks of
1e-9 (10 at LW1), run L ticks after L ticks at the target.

| id | base g a year (registered) | b × 1.1 | b × 0.9 | b × 2 | b × 0.5 | the mirror's g at b × 0.5 |
|---|---|---|---|---|---|---|
| LB1 | 0.8478 (0.847) | 0.8495 | 0.8500 | 0.8686 | 0.8970 | 0.8969 |
| LB2 | 0.8310 (0.830) | 0.8349 | 0.8309 | 0.8650 | 0.8757 | 0.8755 |
| LB3 | 0.9008 (0.901) | 0.9002 | 0.9041 | 0.8958 | 0.9479 | 0.9479 |
| LW1 | 0.8463 (0.836) | 0.8546 | 0.8417 | 0.8718 | 0.8175 | 0.8173 |

Every set passes, the base kick sets included (`horses kick hold`: largest tail 4.9e-5, peak
6.3). The engine's g is P2.2a's reading (decision 293), fitted down to ten times the rounding
floor; the mirror's `lm_kick` fits to 1e-4 of the peak over 30,000 ticks. They agree within
0.005 a year at LB1–LB3 and within 0.016 at LW1.

### E5: the cost shocks at genesis

[cost_shocks.csv](cost_shocks.csv); [fig5](../../figs/loops/fig5_plants_b2.png). Every entry of
A1 §7.2 at LB1–LB3 and LW1–LW3 (and the families', E10) is the engine's to its printed precision,
at L and at 10·L: for example LB1 b × 2, trough 0.704, 46.3 years, heads, capacity plant and
fodder plant within 5% from 10.1, 9.4 and 8.6 years, the horse price's low 0.194; LB3 b × 0.5,
0.850, 135.4 years, 24.0, 34.8 and 31.7 years, 0.390.

### E6: the glut

[glut.csv](glut.csv); [fig4](../../figs/loops/fig4_glut_heads_x10.png).

| id | heads × 2: years; price low–high; withheld | heads × 10: years; price low–high; withheld |
|---|---|---|
| LB1 | 45.4; 0.218–2.16; 303 | 65.3; 0.219–3.76; 1,315 |
| LB2 | 39.8; 0.214–2.02; 221 | 58.0; 0.219–3.29; 1,034 |
| LB3 | 77.6; 0.221–2.24; 662 | 118.2; 0.221–3.56; 2,694 |

Each is CONVERGED with no dead tick, at L and at 10·L, and each number is the mirror's.
*Amended at P2.2b.4 (the fidelity review):* the harness counts the horse market as idle, not
dead. It is idle for 477, 1,524; 353, 1,217; 996 and 3,047 ticks. Through the glut the maker
withholds, keeps breeding and sells nothing: its coin falls to 5.2e-6 of genesis or below
(4.3e-90 at LB3 heads × 10), and the horse price rests unmoved at 0.223 of target for up to
2,510 ticks. With ψ 0 each of the six runs away at ticks 155–158. See "Added at P2.2b.4".

### E7: the pass-through (r × 2 at genesis)

[passthrough.csv](passthrough.csv); [fig2](../../figs/loops/fig2_passthrough_r2.png).

| id | fodder | horse-days | good | land | labour (reported) | fodder low | tasks' horse-days low | class |
|---|---|---|---|---|---|---|---|---|
| **LB1** | 175 | **0** | 11 | 12 | 167 | 0.409 | 0.509 | CONVERGED, 50.3 y |
| LB2 | 263 | 8 | 10 | 11 | 175 | 0.413 | 0.487 | CONVERGED |
| LB3 | 173 | 0 | 12 | 12 | 148 | 0.402 | 0.527 | CONVERGED |
| LC1 | 169 | 0 | 11 | 12 | 163 | 0.420 | 0.526 | CONVERGED |
| LN4 | 301 | 74 | 39 | 11 | 0 | 0.173 | 0.310 | CONVERGED |
| **LN7** | 162 | **146** | 93 | 12 | 0 | 0.234 | 0.270 | ORBITING |
| LN8 | 197 | 148 | 80 | 12 | 0 | 0.220 | 0.185 | CONVERGED |
| LW1 | 208 | 0 | 11 | 12 | 116 | 0.402 | 0.516 | CONVERGED |
| LN1 | 414 | 2,685 | 82 | 10 | 0 | 0.002 | 0.002 | DEAD |

Every row is A1 §7.5's but LN1's, a control reported and not scored, whose fodder and horse-day
counts part (398 and 2,700 in the mirror); it is DEAD either way. *Amended at P2.2b.4 (the
fidelity review):* LB1 converges after r × 2 through the capacity plant and the maker's
reservation together. The maker withholds for 251 ticks, the horse market is idle for 412, the
horse price rests at 0.149 of target for 145 ticks, and the maker's coin falls to 4.0e-7 of
genesis (every-tick rerun). At ψ 0 the same run (LF3) runs away at tick 170. See "Added at
P2.2b.4".

### E8: O14 like for like

[o14.csv](o14.csv); [fig3](../../figs/loops/fig3_o14_b2.png). A1 §7.6's columns, each equal to
its printed precision:

| set | b × 2 trough, ln (of old Y) | T2 dead | T3 baskets / Y\* | T3 dead | T3 peak D̂ | T1–2 years |
|---|---|---|---|---|---|---|
| LW0 (flow, no plants) | −2.358 (0.09), DIVERGED | 215 | 0.435 / 0.061 | 214 | 3,379 | 20.3 / 39.0 |
| **LW1 (flow, plants)** | **−0.389 (0.68)** | 6 | 0.675 / 0.262 | 212 | 2,590 | **19.5 / 41.3** |
| LW2 / LW3 | −0.375 / −0.418 | 6 / 4 | 0.684 / 0.268; 0.659 / 0.253 | 210 / 217 | 2,607 / 2,555 | 19.5 / 41.7; 19.3 / 40.4 |
| **LB1 (stocks, plants)** | **−0.353 (0.70)** | 5 | 0.702 / 0.255 | 244 | 12,029 | **26.5 / 43.0** |
| LB2 / LB3 | −0.336 / −0.390 | 5 / 5 | 0.714 / 0.260; 0.678 / 0.245 | 322 / 232 | 9,440 / 20,677 | 24.1 / 38.0; 42.8 / 73.2 |
| LF1 (θ 0.7) | −0.194 (0.82) | 0 | 0.737 / 0.373 | 315 | 9,746 | 29.5 / 54.5 |
| LC1 (loop cut) | −0.214 (0.81) | 0 | 0.757 / 0.411 | 237 | 11,761 | 25.0 / 41.5 |

Like for like, stocks lift the b × 2 trough by 0.0362 in log and take 1.361 times the flow
control's median years (registered 0.036 and 1.36). LW0's T3 peak D̂ is 3,379 over its runs that
do not run away, as the mirror records it; over all its Tier 3 runs it is 227,000 (JB(0.5), a
runaway). *Amended at P2.2b.4 (the fidelity review):* o14.csv's `T3_peak` is over every Tier-3
run, so it gains `T3_peak_ex_runaway`, the table's figure. The 1.36 is read on every observable,
the maker's plant and the horse market included, which the flow control lacks. On the
observables both share, less the plants, it is 1.17 at δ 8% and 1.94 at δ 4% (2.21 on every
observable). And b × 2 moves the good's equilibrium by −0.16% in log, while the good falls about
30% for half a year: the trough is a path, about 200 times the equilibrium's move in log. See
"Added at P2.2b.4".

### E9: the negative controls

[negative_controls.csv](negative_controls.csv), [glut_psi0.csv](glut_psi0.csv),
[ticklength12.csv](ticklength12.csv). Converged runs per tier, engine (A1):

| id | T1 | T2 | T3 | T3S | stocks | verdict |
|---|---|---|---|---|---|---|
| LN1 | 0/20 | 0/24 | 0/25 | 0/22 | 0/22 | NO-GO |
| LN2 | 20/20 | 22/24 | 9/25 | 21/26 | 14/26 | NO-GO |
| LN3 | 0/20 | 0/24 | 0/25 | 0/24 | 0/24 | NO-GO |
| LN4 | 20/20 | 24/24 | 25/25 | 26/26 | 25/26 | GO in the tiers |
| LN6 | 0/20 | 0/24 | 0/25 | 0/28 | 0/28 | NO-GO |
| LN7 | 0/20 | 0/24 | 0/25 | 0/22 | **0/22 (1/22)** | NO-GO |
| LN8 | 20/20 | 18/24 | 14/25 | 16/22 | 17/22 | NO-GO |
| LN9 | 16/20 | 8/24 | 6/25 | 10/22 | 7/22 | NO-GO |
| LF3 (ψ 0) | 20/20 | 24/24 | 24/25 (24/25) | 27/28 (27/28) | 25/28 | NO-GO |
| LF6 | 11/20 | 4/24 | 3/25 (3/25) | 5/28 (5/28) | 5/28 | NO-GO |
| LW0 | 18/18 | 22/22 | 15/23 (15/23) | 14/16 (14/16) | 12/16 | NO-GO |

Every count is A1's but LN7's stocks family. There `coin.workers*0.1` orbits where the mirror's
converges, within the 2 runs allowed. The class of every control run is the mirror's but five at
LN7, which move among DIVERGED, ORBITING and CONVERGED (JA(0.8), JB(2) and N(2) DIVERGED against
ORBITING; r × 1.2 ORBITING against DIVERGED), as the carry alone moved 19 there. The named
outcomes:
- LN1 and LN7 converge in no tier.
- LN2 orbits in 16 of 25 Tier 3 runs.
- LN3 is DEAD in every run but b × 0.5 at genesis and dated, which orbit.
- LN6 is DEAD in every run and in mode A.
- LF3 diverges on exactly the five named runs, the horse price crossing 1e-6 of genesis at ticks
  156 (heads × 2 and × 10), 169 (workers' coin × 0.02), 170 (r × 2) and 173 (× 0.1).
- At ψ 0, heads × 2 and × 10 run away at ticks 156, 156 (LB1), 158, 156 (LB2) and 155, 155
  (LB3), the mirror's ticks.
- LF6 converges in 11, 4, 3, 5 and 5 runs; LW0 in 15 of 23 Tier 3 runs.
- LB1 at 12 ticks a year converges in 0 of the mirror's 40 Tier 1–2 runs, and in none of the
  four cost targets beside them. Mode A orbits there with a largest gap of 0.30, where the
  mirror's diverges with 0.30. The map is unstable at 12 a year, and 12 of the 41 runs the two
  share are labelled apart, ORBITING against DIVERGED.
- LN5 is not run (LOOP-SPEC-A2; O51).

The controls' run-by-run readouts part more than the GO instances' (LN1 and LN6 above all, whose
markets die): LN6's lowest baskets by up to 15%, its horse price low by up to 29%. They are
reported, not scored ([agreement.csv](agreement.csv)).

### E11: labour supply's bound

[bound.csv](bound.csv). The runs at L that reach it, ticks at the bound (peak z):

| id | runs |
|---|---|
| LB1 | r × 0.5 262 (2.05); N(0.5) 238 (1.86); workers' coin × 2 82 (1.19); heads × 0.1 45 (1.05); JB(2) 22 (1.18) |
| LB2 | r × 0.5 273 (2.13); N(0.5) 250 (1.95); workers' coin × 2 98 (1.25); heads × 0.1 48 (1.05); JB(2) 24 (1.22) |
| LB3 | r × 0.5 247 (1.89); N(0.5) 219 (1.68); heads × 0.1 48 (1.07); workers' coin × 2 28 (1.03); JB(2) 18 (1.11) |
| LW1 | r × 0.5 264 (2.28); N(0.5) 242 (2.10); workers' coin × 2 106 (1.35); JB(2) 26 (1.23) |
| LW2 | r × 0.5 279 (2.34); N(0.5) 256 (2.15); workers' coin × 2 120 (1.40); JB(2) 29 (1.28); the good desk's coin × 0.02 8 (1.02) |
| LW3 | r × 0.5 238 (2.15); N(0.5) 214 (2.00); workers' coin × 2 81 (1.26); JB(2) 21 (1.14) |

Each is A1's, tick for tick. No run starts at the bound, and each of the 28 converges.

## Files

Written by `D:/rustyecon-p2b/run/score.py` (sha256 `0a98d77c…6dec`) and `extra_tables.py` from
`runs.tsv` (`gather.py`):
- [lines.csv](lines.csv): every scored line, with the registered value, the engine's, the
  tolerance and the verdict (1,377 lines);
- [verdicts.csv](verdicts.csv), [tiers.csv](tiers.csv), [cost_shocks.csv](cost_shocks.csv),
  [glut.csv](glut.csv), [passthrough.csv](passthrough.csv), [o14.csv](o14.csv),
  [negative_controls.csv](negative_controls.csv), [kicks.csv](kicks.csv), [bound.csv](bound.csv):
  E3–E11 as above;
- [battery.csv](battery.csv): every run beside the mirror's run of the same name (3,489 runs);
- [agreement.csv](agreement.csv), [glut_psi0.csv](glut_psi0.csv),
  [ticklength12.csv](ticklength12.csv): reported beside the scoring;
- [lists/](lists/): each instance's battery and stocks family, as `horses list` prints them;
- added at P2.2b.4, reported and not scored: [relay.csv](relay.csv),
  [relay_counts.csv](relay_counts.csv), [capital_time.csv](capital_time.csv),
  [last_in.csv](last_in.csv), [o14_years.csv](o14_years.csv), [o14_trough.csv](o14_trough.csv),
  and o14.csv's column `T3_peak_ex_runaway` (the last section).

The figures are in [docs/probe/figs/loops/](../../figs/loops/). Figures 2 and 3 plot the scored
runs again over their first years with a row every tick (the same binary; the runs are
deterministic); the others read the scored runs' rows, one every 52 ticks.

The raw runs, each with its CSV (gzipped), `summary.tsv`, `stats.tsv`, stdout, stderr and exit, are
in `D:/rustyecon-p2b/runs/` (`b52/<inst>/{modea,L,10L,stocks}/`, `extra/{glut10L,psi0,tpy12}/`,
`kicks/`, `lengths/`, `points/`).

*Amended at P2.2b.4 (the measurement review).* The archive regenerates the scoring. Its CSVs are
gzipped, which `gather.py` did not read, so `D:/rustyecon-p2b/fix-report/gather_gz.py` reads
them. From the archive it writes `runs.tsv` byte for byte as scored. `score_fc.py` is `score.py`
with E11's line "no run starts at the bound" failing closed on a run with no z at tick 0 (it
failed open); it fails on the review's regathered table, which had none. On the regenerated
table it reproduces `lines.csv` and every other scored table byte for byte, `battery.csv` as the
committed projection (four long columns of the scorer's, `m_final`, `why`, `m_why` and
`m_runaway`, were left out at P2.2b.3). So `/root/scratch/p2b-runs`, the plain duplicate, is no
longer needed to rescore.

## Added at P2.2b.4: what the reviews asked for

Reported, not scored. Written by `D:/rustyecon-p2b/fix-report/analysis.py`, `frozen.py` and
`merge_relay.py` from the scored runs, and from every-tick reruns of the E6 and E7 runs' first
10,400 ticks with the same frozen binary (`fine.sh`; the runs are deterministic).

**The reservation's part** ([relay.csv](relay.csv), [relay_counts.csv](relay_counts.csv)). Per
run: class and years; withheld ticks, switches, idle and no-order ticks of the horse market; the
horse price's low and high; the heads' low and high; the maker's plant low; and from the
every-tick rerun the maker's lowest coin (of genesis) and output (of target), ticks with its
output below 1% of target, the longest idle spell, and the longest stretch below 0.9 of target
over which the horse price does not move.

| run | withheld | idle | price unmoved: ticks, of target | maker's coin low | output low | heads low–high | ψ 0 |
|---|---|---|---|---|---|---|---|
| LB1 r × 2 | 251 | 412 | 145, 0.149 | 4.0e-7 | 6.0e-6 | 0.57–1.01 | DIVERGED, tick 170 (LF3) |
| LB2 r × 2 | 193 | 352 | 66, 0.150 | 9.5e-6 | 7.9e-5 | 0.56–1.00 | – |
| LB3 r × 2 | 404 | 587 | 282, 0.150 | 9.2e-12 | 1.0e-9 | 0.66–1.04 | – |
| LB1 heads × 2, × 10 | 303, 1,315 | 477, 1,524 | 139, 1,161 at 0.223 | 2.2e-8, 1.3e-43 | 6.6e-7, 2.9e-35 | 0.93, 0.90 low | DIVERGED, tick 156 |
| LB2 heads × 2, × 10 | 221, 1,034 | 353, 1,217 | 68, 891 at 0.223 | 5.2e-6, 3.4e-34 | 5.3e-5, 1.1e-27 | 0.93, 0.89 low | DIVERGED, 158, 156 |
| LB3 heads × 2, × 10 | 662, 2,694 | 996, 3,047 | 457, 2,510 at 0.223 | 1.7e-19, 4.3e-90 | 7.3e-16, 1.1e-72 | 0.94, 0.94 low | DIVERGED, tick 155 |

Over LB1–LB3's battery and stocks family at L (375 runs) the reservation acts in 36, 32 and 49,
117 in all. The maker's output falls below 1% of target in 19 of those, on 52-tick rows
(`below1.py`: N(2), heads × 2, r × 2, the workers' coin × 0.02 and × 0.1, heads × 10, and at LB3
r × 0.5 and plant.capacity × 10), and in none where it does not act. The switches reach 274, 262
and 450. LF3 (ψ 0) converges in 120 of LB1's 125.

**Capital's time (D-G14)** ([capital_time.csv](capital_time.csv), [last_in.csv](last_in.csv)).
The cost shocks at genesis at LB1–LB3, LF2 and LW1–LW3: years to tolerance; the heads', each
plant's and the paper's years to within 5% (the paper's is ln(K′/K)/ln(1 − δ) ticks of zero
builds for a glut, a tick for a shortage); the quasi-rent 3 ticks and 1/δ on. For which
observable comes into tolerance last, each converged battery run at L is read on its 52-tick
rows: the first row from which each gap stays within 1e-3, from the scored clock's start (L/4
for a dated shock). A tie counts every group that ties.

| instance | the maker's plant last | the horse market last | a plant last (flow) | runs |
|---|---|---|---|---|
| LB1 | 59 | 50 | – | 97 |
| LB2 | 63 | 32 | – | 97 |
| LB3 | 82 | 17 | – | 97 |
| LW1, LW2, LW3 | – | – | 77, 77, 76 | 83 each |

**O14's years and trough** ([o14_years.csv](o14_years.csv), [o14_trough.csv](o14_trough.csv)).
The Tier 1–2 upper medians over converged runs (as `score.py` reads them), on every observable
(the harness's ticks, and 52-tick rows), without the plants, and on the observables both the
stocks and the flow economies have, less the plants: LB1 26.5, 27, 25, 21 years against LW1's
19.5, 20, 18, 18; LB2 24.1, 25, 23, 21 against 19.5, 20, 18, 18; LB3 42.8, 43, 42, 35 against
19.3, 20, 18, 18. The trough, every tick (`/root/scratch/p2b-diag/fine`, P2.2b.3's reruns): b × 2
moves Y\* from 9.918 to 9.902 (−0.16% in log); the good falls to 0.704 (LB1) and 0.679 (LW1) of the
new Y\* at tick 9 and stays below 0.9 for 27 and 32 ticks, 217 and 239 times the move in log.
