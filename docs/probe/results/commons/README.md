# The open-commons instances on the engine: the scored runs (E2–E9)

Dated 2026-09-30. Step P2.3.15 on branch `phase2-proper`, label `run`. Scratch
`D:/rustyecon-p23/run/`, raw runs `D:/rustyecon-p23/runs/commons/` (CSVs gzipped). It runs the
commons frame's §5.5 ([../../commons/SPEC.md](../../commons/SPEC.md), registered at P2.3.5 with
Tier 3S, [registration.md](registration.md)) on the engine, E2–E9, as the wave's README names
each job ([../p23-wave/README.md](../p23-wave/README.md)), and scores each against the
registration line by line with the scorer committed before the wave (P2.3.11). E0–E2 came before,
at P2.3.9 ([e0.md](e0.md)), under amendments A1 and A2.

Conditions of every number here, unless its line says otherwise: C1 (the commons full, a shadow
rent) and C2 (the commons with room), I1's economy with one priced worker type in food, the
commons held by the workers and no market; C2m, 52 ticks a year, ρ 0, `Saturate`, planned
assignment; L 141,000 at C1 and C1N and 142,000 at C2 (the engine's elasticity probe);
tolerance 1e-3 in log on the 22 observables; Tier 3 and Tier 3S again at 10·L.

## Verdict

**C1 and C2 are GO, as registered.** Every line of the verdict holds at both: mode A, every
non-vacuous run of Tiers 1–3 and Tier 3S CONVERGED at L and again at 10·L, the six registered
VACUOUS runs at C2 VACUOUS, and all 26 kick sets. The subsistence trap appears exactly where the
mirror put it, run for run and seed for seed, and nowhere in the verdict battery. The scorer read
44,484 lines at C1, C2 and C1N: 28,391 pass, 15,970 are reported and not scored, and **123
fail**, of two kinds, neither a class and neither a tick to tolerance:
- **65 runaway ticks** of trap runs (64 at E5, at C1 and C2, and one at C1N, E9) differ from the
  mirror's by more than 5%, by up to 29 ticks (10.5%). The engine's runaway bound is relative to
  the run's displaced genesis prices, the mirror's to the undisplaced point; where a run displaced
  labour's or food's price the two bounds differ. The mirror against the engine's reference gives the engine's tick
  in 190 of the 195 trap runs displaced at genesis, and one tick apart in the other 5 (below).
- **58 shadow rents at the end**, at C1 at 12 and 365 ticks a year (E8), differ from the mirror's
  by more than 1e-9: the mirror's is read where it stopped early, 1.1e-9 to 7.0e-8 from the
  oracle's, and the engine's at L is within 3.0e-13 of the oracle's (below).

The committed scorer's printout (`../p23-wave/score.out`) reads "the commons: c1 GO", "the commons:
c2 GO" and "the commons' refutations: none".

*Amended at P2.3.16 (2026-09-30, label `fix-report`, after the two reviews):*
- **C1's GO is a point result at C2m.** The fidelity review ran C1's Tier 3 with the dials moved
  (6,000 ticks, on the P2.3.12 binary; a DIVERGED class is decisive by then, a CONVERGED one is
  the shorter horizon's). With every price rate × 0.9 it is 41/43: `p[mach]*0.5` and `JB(2)` fall
  into the subsistence trap. With every buffer × 1.1 it is 42/43, `JB(2)` in the trap.
  `p[mach]*0.5` also diverges at rate × 0.8, buffer × 1.25, adjust × 2, tilt 0.15–0.75 and 12
  ticks a year; it converges at rate × 1.25, buffer × 0.8, adjust × 0.5 and × 1.5, and tilt 0.05
  and 0.1. `JB(2)` diverges at each ±25% move tried. On the basin grid the machine price's edge
  is between × 0.481 (the last CONVERGED) and × 0.458, so the verdict run at × 0.5 is 0.04–0.09 in
  log from the trap (0.04–0.05 from the recheck's runs between the two); every other basin
  direction at C1 and C2 has 0.21 or more. So C1 would be
  LOCAL at those dials. C2 (41 + 2 VACUOUS) and IW1 keep their Tier 3 at the same moves, and
  I1's `p[mach]*0.5` and `JB(2)` converge at every ±25% move. A spot check at P2.3.16 reran nine
  of these moves (C1, C2, I1, IW1) and C1's two runs at C2m on the same binary, and got the
  review's class in all eleven (`D:/rustyecon-p23/fix-report/spot/`). The review's reading: at
  rest the exit is worth 0.529 of the wage at C1 and 0.354 at C2, and the trap sits closer where
  the exit is worth more.
- **What the commons answer covers.** The rule gives out one plot-taking pop's commons. It posts
  no price at zero rent, so no land market at zero rent runs here; idle enclosed land at r = 0
  (decisions 153, 160, 161, 376, 381) is in no instance and untested; several types sharing a
  commons need O101.
- **The failed lines, by amendment.** Dated amendment A3 ([registration-A3.md](registration-A3.md))
  holds the runaway ticks to the harness's reference (O107, ruled) and r_o at the end to the
  oracle's (O108). Written after the result and disclosed. Under it (`score.py --amended`,
  `../p23-wave/score-amended.out`) all 123 lines pass, no passing line fails, and the classes,
  ticks and verdicts are as scored.
- **The commons as a market** (SPEC §1.3–§1.4, the named alternative). Where the commons has room,
  `Saturate` takes its price down geometrically toward the oracle's r_o\* = 0 while the real
  observables converge (within 1.1e-7 to 3.0e-7 in log when it crosses the bound). "Runs away" in
  SPEC §1.4 means it fails the log-scale runaway bound and, later, the price step's positive range;
  it is a limit of what a market can post, not an economic divergence. What the alternative lacks
  is a zero price markets can hold (O96).

Apart from those two readings, the engine is the mirror in everything the bands read. In all 2,425
compared runs:
- every class is the mirror's, the 202 trap runs among them, name for name;
- every tick to tolerance of the 2,189 CONVERGED runs is the mirror's, to the tick (fig1);
- every run's regime ticks (with the mirror's end regime carried to L) and regime switches are
  the mirror's; the lowest baskets and every market's lowest cleared volume are the mirror's to
  2.6e-6; dead ticks are the mirror's but in three runs, by one tick.

| E | registered (SPEC §5.5–§6.9, registration §3) | engine | |
|---|---|---|---|
| E2 mode A, L, base kicks | PASS at L: 2.4e-15 (C1), 4.4e-16 (C2); at 12 and 365 reported; L 141,000, 142,000 (26,000; 1,113,000, 1,116,000); the base kick sets pass | PASS: 2.0e-15, 5.6e-16 (12 a year 3.1e-14, 2.2e-16; 365 5.1e-15, 6.7e-16); every L equal; base kick sets PASS (tail gains 1.3e-5, 5.7e-6; at 12 and 365 also PASS) | holds |
| E3 the verdict | GO: C1 30/30, 42/42, 43/43, 43/43 at 10·L; C2 30/30, 38/38 + 4 VACUOUS, 41/41 + 2, the same at 10·L; Tier 3S 24/24 at L and 10·L each | exactly those classes, every run; GO at both | holds |
| E3 kick sets | a kick set at every target | 26/26 PASS, largest tail gain 1.5e-5 | holds |
| E3 speeds and paths (§6.3) | medians C1 319/372/508, C2 315/360/514, and the tier table | every run's ticks equal, so every median, slowest and tier entry; dead ticks, lowest baskets and cleared volumes per tier equal | holds |
| E3 regimes (§6.7) | every CONVERGED battery run ends in its target's regime; r_o within 1e-9 at Crowded, 0 exactly at Commons, r exactly at Enclosed | 224/224 (115 + 109); r_o/r 0.0 and 1.0 exactly; at Crowded within 3.0e-13 of the oracle's | holds |
| E4 stocks | 41/41 each; median 419 and 444 ticks | 41/41 each; 419 and 444 | holds |
| E5 joint and basin | C1 joint2 56 + 4 DIVERGED (seeds 3, 12, 34, 39), joint4 28 + 12, basin 341 + 89; C2 joint2 60, joint4 32 + 8, basin 353 + 77; runaway ticks within 5% | every class and every DIVERGED run's name; 64 of the 190 runaway ticks beyond 5% | **64 lines fail** (runaway ticks) |
| E6 history | C1 land.mach: runaway in window 5 at tick 288 (7,788 of the run), windows 0–4 in tolerance; C1 commons, C2 both: 81/81 | the runaway at tick 7,788; windows 0–4 in tolerance; 81/81, 81/81, 81/81; every window's ticks and dead ticks the mirror's | holds |
| E7 Hold, tilt 1 | Hold C1 115, C2 109 + 6 VACUOUS; tilt 1 C1 112 + 3 DIVERGED (p[mach] × 0.5, JA(0.5), JB(2)), C2 109 + 6 VACUOUS | exactly those | holds |
| E8 tick length | 12 a year C1 72, C2 68 + 4 VACUOUS; 365 a year the same; 75.6 and 80.2 years, 6.0 and 5.9 years median; r_o at the end within 1e-9 | every class and tick; the medians equal; at C1, 58 end shadow rents beyond 1e-9 of the mirror's early-stopped ones | **58 lines fail** (r_o at the end) |
| E9 enclose, negative control | enclose 4/4 each, 323 and 324 ticks; C1N 100 + 9 DIVERGED + 6 VACUOUS, the 9 named, runaway at ticks 279–314 | 4/4, 323 and 324; C1N exactly those classes and names; runaway ticks 280–300, JB(2)'s 293 against 279 | **one line fails** (runaway tick) |

**Refutation criteria** (SPEC §6.9), none met:
- a class change in E3: none, in 412 runs;
- an engine run at a registered target resting off the oracle's point: none; the largest end gap
  of any CONVERGED run is 8.8e-14 in log (`c1/tpy365/b.food=0.54@genesis`), and the shadow rent
  at every Crowded end within 3.0e-13 of the oracle's;
- a trace-diff parting not explained by a named rounding: E0 (P2.3.9, A1 and A2);
- the trap in any E3 run: none; or none of its predicted runs falling in: all 203 predicted
  (joint, basin, tilt 1, C1's land.mach history, C1N) fall in, and no other run does.

## The failed lines

### 65 runaway ticks: the bound's reference

The runaway bound is "every posted price within [1e-6, 1e6] × genesis" (PROBE-SPEC §4.5). The
harness reads genesis as the run's own genesis, displaced (`harness.rs`, `g.prices`), as it has
since P2.1. The commons mirror's runner reads it as the undisplaced point (`battery_c.run`,
`p_gen = st0["p"]`, taken before `displace`). In the trap every price inflates together, so the
tick at which the first one leaves the bound depends on which reference it is held to only when
the displaced price is the one that leaves first: labour's price × F below 1 leaves an engine
bound F times lower, sooner (`p[labour]*0.1227`: 248 against 277), and food's × F above 1 an
engine bound F times higher, later (`p[food]*8.15`: 252 against 239 at C1). Where a run displaced
land or the machine, the ticks are the mirror's to the tick in 91 of 93 runs and one tick apart in
two.

`diag/runaway_ref.py` runs the registered mirror (`cm.py`, `battery_c.displace`, unedited) on
all 195 trap runs at genesis and measures each against both references
([diag/runaway_ref.out](diag/runaway_ref.out)): against the undisplaced point it gives the
registered ticks; against the displaced genesis it gives the engine's tick in 190 runs and one
tick apart in 5, which is the trap's own amplification of rounding (E0's A1). The class of every
trap run is the mirror's. So the 65 lines fail on the bound's reference, not on the path; the
registered band was set on the mirror's reference and the scorer read the engine's number as the
harness prints it. Neither reference is registered as the rule for this line; the harness's is
P2.1's, and I leave the choice between them to the reviews (open item O107).

### 58 shadow rents at the end: the mirror's early stop

The mirror stops a run once it has stayed within 1e-6 in log of its target for 2,000 ticks,
from tick 4,000 on; its `ro_end` is r_o/r on that tick. At 52 ticks a year that is within 1e-9
of the oracle's, and every such line passes. At 12 and 365 a year the mirror stopped at ticks
4,281–7,868, with r_o/r still 1.1e-9 to 7.0e-8 from the oracle's; the engine runs to L (26,000
and 1,113,000 ticks) and ends within 3.0e-13 of it (4.4e-16 at best). The scorer's reading, fixed before
the wave, compares the engine's r_o with the mirror's `ro_end`, so these 58 lines fail
([diag/ro_end.out](diag/ro_end.out)). Against the oracle's r_o, SPEC §6.7's statement, every one
of the 987 Crowded ends is within 3.0e-13.

## How it ran

- **The binaries.** `markets` from P2.3.11 (`198554a`), sha256
  `3f957fb0d95b1522eb69214b0fec458eedd7aa1699069e735fe775c335b62371`, for the wave; from P2.3.12
  (`c4adbf8`), sha256 `be3266abcf5e35f20b20158800a731ef948a518b42d9319d275e6332dd0542e9`, for the
  54 jobs the dated-commons bug had stopped.
- **The jobs:** 2,473 at the commons of the wave's 3,721, on 2026-09-30 from 09:29, 46 at a time.
  Two faults of the machinery, each fixed in its own commit and disclosed
  ([../p23-wave/rerun/README.md](../p23-wave/rerun/README.md)):
  - P2.3.12: every tape with a dated shock of a `FlowPerYear` param (`commons=V@dated`,
    `enclose=F@dated`, `cycle(commons,P,N)`) was refused at load, since the tape writer wrote its
    schedule param `Dimensionless`. 54 jobs failed before their first tick. Fixed with a test and
    the gates on both machines, and rerun; 24 untouched jobs rerun on the fixed binary gave the
    wave's files byte for byte (92 files).
  - P2.3.13: the runner stripped the jobs' quotes, so 452 commons jobs whose run name holds a
    parenthesis never started (two of them among the 54); they ran from 10:17 with the quotes
    kept, on the binary their list names.
  Every job exited 0 and printed its job's run name.
- **The raw runs** are gzipped in `D:/rustyecon-p23/runs/commons/` (the 54 failed outputs in
  `runs-failed/`, the R1 check's in `runs-r1check/`), and `gather.py` regenerates the wave's
  `runs.jsonl` from them byte for byte (sha256 `f9fc7416…ea79`).
- **The scorer** and its readings are unchanged but for one table column, the kick sets passed in
  `verdicts.csv`, which read 1/13 for 13/13 (P2.3.14, `../p23-wave/FIXES.md`); no line moved.
  Nothing in the harness, the rules or the protocol changed after a scored result was read.
  *Amended at P2.3.16:* that sentence is scoped to the scored wave. E0's amendment A1 was written
  after a development trace diff showed the negative control's trap run parting, and committed
  before E0's official run; A2 replaced A1's second condition after the official run showed it
  unmet ([registration-A2.md](registration-A2.md)); A3 came after the wave and the reviews.

## The details

### E3: the tiers (§6.3)

[verdicts.csv](verdicts.csv), [tiers.csv](tiers.csv), [runs.csv](runs.csv) (every run of every
set, engine beside mirror), [kicks.csv](kicks.csv). The engine's numbers, each the mirror's
(the lowest baskets to 2.6e-6):

| | tier | ticks to tol, median (slowest) | peak D̂ median / worst | dead ticks median / worst | worst fill | lowest baskets | transfer short in (most ticks) | provider coin low |
|---|---|---|---|---|---|---|---|---|
| C1 | 1 | 319 (444) | 71 / 504 | 0 / 0 | 0.524 | 0.632 | 1 run (11) | 0.446 |
| C1 | 2 | 372 (557) | 311 / 2,184 | 0 / 96 | 0.085 | 0.131 | 10 (94) | 0.247 |
| C1 | 3 | 508 (828) | 1,141 / ∞ | 30 / 183 | 0 | 0 | 30 (293) | 0.047 |
| C2 | 1 | 315 (494) | 72 / 419 | 0 / 0 | 0.597 | 0.688 | 1 (2) | 0.470 |
| C2 | 2 | 360 (547) | 232 / 1,739 | 0 / 79 | 0.134 | 0.205 | 10 (73) | 0.278 |
| C2 | 3 | 514 (641) | 1,068 / ∞ | 23 / 153 | 0 | 0 | 27 (196) | 0.100 |

Tier 3 at 10·L gives the same row as at L at both. Tier 3S: 24/24 at L and at 10·L at each, the
mirror's ticks (C1 median 379, slowest 534; C2 340, 543). Every CONVERGED battery run ends in its
target's regime, with r_o/r exactly 0 at a Commons target and exactly 1 at an Enclosed one.

### E4–E9: the families

[families.csv](families.csv), [history_c1.csv](history_c1.csv), [history_c2.csv](history_c2.csv).
Every run's class and ticks are the mirror's, so every count, median and slowest of SPEC §6.5 is
the engine's:

| family | C1 | C2 |
|---|---|---|
| stocks (41) | 41 CONVERGED; median 419 ticks | 41; 444 |
| joint2 (60) | 56, 4 DIVERGED (seeds 3, 12, 34, 39) | 60 |
| joint4 (40) | 28, 12 DIVERGED (the registered seeds) | 32, 8 DIVERGED (the registered seeds) |
| basin (430) | 341, 89 DIVERGED (the registered runs) | 353, 77 DIVERGED |
| Hold (115) | 115 | 109, 6 VACUOUS |
| tilt 1 (115) | 112, 3 DIVERGED (p[mach] × 0.5, JA(0.5), JB(2)) | 109, 6 VACUOUS |
| 12 a year, Tiers 1–2 (72) | 72; median 907.5 ticks = 75.6 years | 68, 4 VACUOUS; 963 = 80.2 years |
| 365 a year, Tiers 1–2 (72) | 72; median 2,174 ticks = 6.0 years | 68, 4 VACUOUS; 2,153 = 5.9 years |
| enclose (4) | 4; 323 ticks | 4; 324 |
| negative control C1N (115) | 100, 9 DIVERGED (the registered nine), 6 VACUOUS | — |

(The medians here are over every CONVERGED run, as SPEC §6.5; `families.csv` gives them over the
non-slack runs, 910.5 and 1,005 at 12 a year, and those are the mirror's too.) The history: at C1
the land.mach cycle runs away in window 5, 288 ticks after its step from × 0.5 to × 1, tick 7,788
of the run, the mirror's tick; windows 0–4 end in tolerance with the mirror's ticks and dead ticks
(fig2). C1's commons cycle (fig3) and both of C2's end all 81 windows in tolerance, each window's
ticks and dead ticks the mirror's (medians 497, 551 and 0).

## Files

Here: [verdicts.csv](verdicts.csv), [tiers.csv](tiers.csv), [runs.csv](runs.csv) (every
compared run: class, ticks, runaway tick, lowest baskets, dead ticks, regime ticks and switches,
r_o at the end, engine beside mirror), [families.csv](families.csv), [kicks.csv](kicks.csv),
[history_c1.csv](history_c1.csv), [history_c2.csv](history_c2.csv), [fails.csv](fails.csv) (the
123 failed lines), and `diag/` (`runaway_ref.py`, `ro_end.py` and their outputs). Every scored
line is in `../p23-wave/lines.csv.gz`. Plots in `../../figs/commons/`: fig1 ticks to tolerance,
engine against mirror; fig2 the trap in C1's land.mach history, window 5; fig3 the rule through
the three regimes in C1's commons history.
