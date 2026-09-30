# The wall instance on the engine: the scored runs (E2–E9)

Dated 2026-09-30. Step P2.3.15 on branch `phase2-proper`, label `run`. Scratch
`D:/rustyecon-p23/run/`, raw runs `D:/rustyecon-p23/runs/wall/` (CSVs gzipped). It runs the wall
frame's §7 ([../../wall/SPEC.md](../../wall/SPEC.md), registered at P2.3.1,
[registration.md](registration.md)) on the engine, E2–E9, as the wave's README names each job
([../p23-wave/README.md](../p23-wave/README.md)), and scores each against the registration line
by line with the scorer committed before the wave (P2.3.11). E0–E2 came before, at P2.3.4
([e0.md](e0.md)).

Conditions of every number here, unless its line says otherwise: IW1 (unit 1d's B economy with
three worker types, the solved wall), the many-market roles with the wall's three fields, C2m, 52
ticks a year, ρ 0, `Saturate`, planned assignment, L 22,000 (the engine's own elasticity probe),
tolerance 1e-3 in log on the 18 observables; Tiers 3 and 3S again at 10·L.

## Verdict

**IW1 is GO, as registered.** Every line of the verdict holds: mode A, Tiers 1–3 (26/26, 38/38,
39/39), Tier 3S (20/20), Tiers 3 and 3S again at 10·L (39/39, 20/20), and all 13 kick sets. The
scorer read 15,304 lines at IW1 and IC1: 8,682 pass, 6,621 are reported and not scored, and **one
fails**: E6's "every CONVERGED run ends at the wall", at four runs at 365 ticks a year, whose
displaced shares stall at 70 subnormal ulps (3.46e-322) where the registered text gives the
52-a-year stall, 10 ulps (5e-323). The cause is arithmetic, below; I read it as the wall, not as a
refutation, and flag that reading for review (O108).

The engine is the mirror to a degree the bands do not need. In all 1,208 compared runs (IW1's
1,085 and IC1's 123):
- every class is the mirror's;
- every tick to tolerance is the mirror's, to the tick (fig1);
- the lowest baskets and the lowest depths are the mirror's to the seven digits the harness
  prints (within 5.0e-8 and 4.5e-7);
- at IW1, every run's dead ticks, dead ticks by market and breach ticks equal the mirror's.

| E | registered (SPEC §7) | engine | |
|---|---|---|---|
| E2 mode A and L | PASS at L below 1e-9: 3.3e-16 (52 a year), 1.6e-15 (12), 4.4e-16 (365); L 22,000, 20,000, 164,000 from the engine's probe; goods' τ 106.4 | PASS: 3.3e-16, 4.4e-16, 4.4e-16; L 22,000, 20,000, 164,000; τ 106.4 | holds |
| E3 the verdict | GO: Tier 1 26/26, Tier 2 38/38, Tier 3 39/39, Tier 3S 20/20; Tiers 3 and 3S at 10·L the same classes and ticks | GO: 26/26, 38/38, 39/39, 20/20; at 10·L 39/39 and 20/20, every tick to tolerance the L run's | holds |
| E3 §6.2's table | ticks 340 (452), 416 (575), 561 (705), 3S 404 (550); troughs, dead ticks by market, depths, breaches per tier | every entry equal to its printed digits (the table below) | holds |
| E4 kick sets | every set decays at every target, gain_tail ≤ 1e-3 (mirror ≤ 1.5e-5) | 13/13 PASS, largest gain_tail 1.07e-5, largest peak 2.77 | holds |
| E4 the base's decay | within 0.03 a year of 0.534 (0.988002 a tick) | 0.5338 a year (0.9880017 a tick), the slowest of 14 kicks | holds |
| E4 the technique's rate | exactly 0.951229 | every s_t/s_(t−1) while the wall binds within 2.2e-16 of e^(−0.05) | holds |
| E5 the cost shocks | land.mach × 2 bottoms at 0.165 of the new Y\* with 93 dead ticks; tail.services × 2 at 0.516 and res.services.trained × 2 at 0.500 with none | 0.1648 (93 dead; dated 0.1642, 94); 0.5157 (0); 0.5000 (0) | holds |
| E6 breaches | only JB(0.5): 11 ticks from tick 0, lowest depth −0.444, largest share 0.101; no other run below 0.128 | only JB(0.5): 11 ticks from tick 0, −0.4437, 0.1007; the next lowest 0.12803 (land.mach 0.8, dated) | holds |
| E6 the shares | fall by 0.951229 exactly while the wall binds; thresholds back at 78, 105, 124 (124 in x\*/2, 102 in JB(0.5)); a share never displaced stays 0.0; a displaced one ends at 5e-323; 5e-323 after 30,000 ticks from 0.5 | all as registered, tick for tick | holds |
| E6 no saturation | no pop saturates in the battery | 0 saturated ticks | holds |
| E6 at the wall at the end | every CONVERGED run ends at the wall, every labour market trading | 1,082 of 1,086 end with every share 0 or at most 5e-323; the other 4 (365 a year) at 3.46e-322; every labour market trades at the end of all 1,086 | **one line fails** (below) |
| E7 the families | stocks 31/31, joint2 60/60, joint4 40/40, basin 516/516; history 79/79 windows and the last from tick 686, each × 0.5 window's basket-less tick; 12 and 365 a year 64/64 each; Hold identical to Saturate; tilt 1 64/64; §6.3's numbers | every count, every run's class and ticks, and §6.3's table to its printed digits; history 405 of 405 window lines; Hold equal to Saturate in every statistic, 64/64 | holds |
| E8 the line control | IC1 103/103 and Tier 3S 20/20 at 25,000; §6.4's numbers reported | 103/103 and 20/20; every tick to tolerance the mirror's | holds (numbers reported) |
| E9 the edges | §2.4's gaps within 1e-12 at genesis and every target; none below 0.2 | all 117 within 5.6e-16; least 0.21636 (land.mach 0.8) | holds |

**Refutation criteria** (SPEC §7), none met on my reading:
- a class change in the registered battery: none, in 103 runs (nor in the other 1,105);
- a CONVERGED run ending more than 1e-12 in log off its target's oracle point: none; the largest
  end gap is 1.0e-13 (`tpy365/s[goods]=0.05`), in every other set below it;
- a converged run ending off the wall (a desk's share not back to 0), or a labour market dead at
  the end: every labour market trades at the end of every CONVERGED run. Four runs at 365 a year
  end with a share at 3.46e-322, which the scorer's reading (at most 5e-323) counts off the wall;
  I read them as at the wall (below; O108). **This is a reading taken after the result, and it is
  yours to veto.**
- the rest point differing from unit 1d's by more than 1e-12: none (E2 at P2.3.4);
- a kick set failing, or the slowest mode at or above 1: none; every set passes;
- any committed pin moving: none (the gates at P2.3.10 and P2.3.12, both machines).

## The failed line: where a share stalls at 365 ticks a year

The frame registers that a displaced share "decays until it stalls at 5e-323, ten subnormal ulps,
where a·s rounds to 0" (SPEC §6.2), and E6 that every CONVERGED run ends at the wall. The scorer,
committed before the wave, read "at the wall" as each share on the last row 0.0 or at most 5e-323,
in every IW1 set. Four runs fail it: `s[services]=0.05`, `s[services]=0.2`, `s[goods]=0.05` and
`s[goods]=0.2` at 365 ticks a year, whose displaced share ends at 3.46e-322, with its threshold at
1.0 exactly and every labour market trading.

The stall is where a·s rounds to 0, so it depends on the tick length: with a = share(adjust) =
1 − e^(−2.6/tpy), a share of k subnormal ulps stops moving once k·a < 1/2. At 52 a year a is
0.0488 and k is 10 (4.9e-323); at 365 a year a is 0.00710 and k is 70 (3.46e-322); at 12 a year a
is 0.195 and k is 2. The registered 5e-323 is the 52-a-year stall, the only one the frame names.
The engine's shares equal the mirror's to the bit (E0), so the mirror stalls there too. The line
fails as the scorer read it; the refutation criterion ("a desk's share not back to 0") is, on the
frame's own account of the stall, not met.

## How it ran

- **The binary** is `markets` from P2.3.11 (`198554a`), built in WSL release, sha256
  `3f957fb0d95b1522eb69214b0fec458eedd7aa1699069e735fe775c335b62371`. It carries P2.3.10's fix:
  the harness's baskets eaten over Y\* summed only the provider's and the workers' baskets, P2.1's
  two households, and read 0.725 at IW1's rest; found by reading the code while the scorer was
  written, fixed with a test and the gates before the wave (WALL-RULES §4, amended).
- **The jobs:** 1,248 at the wall of the wave's 3,721 (`../p23-wave/jobs.txt`), one `markets`
  process each, 46 at a time on WSL's 48 threads, on 2026-09-30 from 09:29. The wave's runner
  stripped each job line's quotes, so 251 wall jobs whose run name holds a parenthesis (JA, JB, N,
  RC, RW, `joint`, `cycle`) never started; they ran from 10:17 on the same binary with the quotes
  kept (P2.3.13; `../p23-wave/rerun/README.md`). Every job exited 0, and each printed its job's
  run name. The dated-commons fix (P2.3.12) touched no wall job.
- **The raw runs** are gzipped in `D:/rustyecon-p23/runs/wall/`, and `gather.py` regenerates the
  wave's `runs.jsonl` from them byte for byte (sha256 `f9fc7416…ea79`, decision 311).
- **The scorer** (`../p23-wave/score.py`, committed before the wave) and its readings
  (`../p23-wave/README.md`) are unchanged but for one table column at the commons (P2.3.14,
  `../p23-wave/FIXES.md`). Nothing in the harness, the rules or the protocol changed after a
  result was read.

## The details

### E3: the battery, Tier 3S and 10·L (§6.2's table)

[verdicts.csv](verdicts.csv), [tiers.csv](tiers.csv), [battery.csv](battery.csv) (every run of
every set, engine beside mirror). Engine / mirror where they are two numbers:

| tier | CONVERGED | ticks to tol, median (slowest) | lowest baskets | lowest depth | dead ticks, worst | breach ticks, most | peak D̂ median / worst | worst buyer fill | worst transfer shortfall |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 26/26 | 340 (452), the mirror's | 0.694 / 0.694 | 0.840 / 0.840 | 0 / 0 | 0 / 0 | 56 / 370 | 0.676 | 0 |
| 2 | 38/38 | 416 (575), the mirror's | 0.330 / 0.330 | 0.496 / 0.496 | 7 / 7 | 0 / 0 | 215 / 1,113 | 0.314 | 22.25 |
| 3 | 39/39 | 561 (705), the mirror's | 0.000 / 0.000 | −0.444 / −0.444 | 117 / 117 | 11 / 11 | 853 / ∞ | 0.000 | 706.14 |
| 3S | 20/20 | 404 (550), the mirror's | 0.286 / 0.286 | 0.643 / 0.643 | 8 / 8 | 0 / 0 | 506 / 1,253 | 0.286 | 3.60 |

Dead ticks by market, the worst over the tier, engine and mirror equal: Tier 2 goods 7, mach 4,
services 2; Tier 3 services 115, goods 115, mach 84, labour.master 56, labour.trained 47, land
30, labour 0; Tier 3S goods 8, mach 1, services 1. Highest participation (entrant, trained,
master): 0.457, 0.320, 0.483 (Tier 1) to 0.748, 0.618, 0.697 (Tier 3); no saturated tick. At 10·L
every Tier 3 and 3S run keeps its class and its ticks to tolerance.

### E4: the kick sets and the slowest mode

[kicks.csv](kicks.csv), [envelope.csv](envelope.csv). Each target's kick set (`markets kick`, the
dated run, H = 22,000): 13/13 PASS, gain in the tail 4.9e-6 to 1.07e-5, peak at most 2.77. The
base's envelope, the frame's own measure on the engine's runs (each price × (1 ± 1e-9) at genesis,
22,000 ticks): the slowest decay between 1e-1 and 1e-3 is 0.9880017 a tick (the trained's wage
× (1 − 1e-9)), 0.5338 a year, against the mirror's 0.988002 and 0.534; the 14 kicks run from
0.98611 to 0.98800. The mirror's largest root over the targets, 0.991911, is its own (the
harness has no PL); every kick set's decay is the reading of "below 1".

### E5 and E6: the cost shocks and the wall's own readouts

[readouts.csv](readouts.csv); fig2, fig3. The six cost-shock runs bottom and die as registered:
land.mach × 2 at 0.16484 of the new Y\* with 93 dead ticks (dated 0.16424, 94), tail.services × 2
at 0.51573 (dated 0.51276), res.services.trained × 2 at 0.50000, neither with a dead tick, while
Y\* does not move (OW2). JB(0.5) is the only breach: 11 ticks from tick 0, the depth from −0.4437
up through 0 at tick 11, each desk's share peaking at 0.1007 and back within tolerance at tick 102
(fig2). The thresholds of `s[D]=V` come back at ticks 78, 105 and 124, and x\*/2's at 124, each
the mirror's. While the wall binds every displaced share falls by e^(−0.05) = 0.951229 a tick
to 2.2e-16, and at L it is 5e-323 (ten subnormal ulps) in every battery run that displaced it;
`s[goods]=0.5` is at 5e-323 after 30,000 ticks. No share never displaced leaves 0.0.

### E7: the families

[families.csv](families.csv), [history.csv](history.csv). Engine / mirror:

| family | CONVERGED | ticks to tol, median (slowest) | dead ticks, worst | lowest baskets | ticks with no baskets, most | runs with a breach, most breach ticks | saturated ticks, most (E / T / M) |
|---|---|---|---|---|---|---|---|
| stocks | 31/31 | 457 (626), the mirror's | 77 / 77 | 0.000 / 0.000 | 2 / 2 | 0, 0 | 0 / 0 / 0 |
| joint2 | 60/60 | 572 (729), the mirror's | 140 / 140 | 0.032 / 0.032 | 0 / 0 | 3, 10 | 1 / 0 / 0 |
| joint4 | 40/40 | 666 (834), the mirror's | 203 / 203 | 0.000 / 0.000 | 13 / 13 | 16, 49 | 13 / 4 / 6 |
| basin | 516/516 | 550 (1,025), the mirror's | 250 / 250 | 0.000 / 0.000 | 18 / 18 | 86, 48 | 158 / 1 / 8 |
| 12 a year, Tiers 1–2 | 64/64 | 118 (187), the mirror's | 1 / 1 | 0.401 / 0.401 | 0 / 0 | 0, 0 | 0 |
| 365 a year, Tiers 1–2 | 64/64 | 2,510 (3,943), the mirror's | 56 / 56 | 0.281 / 0.281 | 0 / 0 | 0, 0 | 0 |
| Hold, Tiers 1–2 | 64/64 | 360 (575), the mirror's | 7 / 7 | 0.330 / 0.330 | 0 / 0 | 0, 0 | 0 |
| tilt 1, Tiers 1–2 | 64/64 | 299 (488), the mirror's | 9 / 9 | 0.306 / 0.306 | 0 / 0 | 0, 0 | 0 |

(The medians over both tiers together. SPEC §6.3 prints them tier by tier: 106 (149) and 134
(187) at 12 a year, 2,319 (3,095) and 2,852 (3,943) at 365, 267 (392) and 346 (488) at tilt 1;
the engine's are 106 (149) and 133.5 (187), 2,319 (3,095) and 2,852.5 (3,943), 267 (392) and
346.5 (488), each the mirror's; SPEC rounds a half.) Hold equals Saturate
in every statistic of every run. The history, land.mach cycled 80 times 1,500 ticks apart: every
window in tolerance at its end, with the mirror's ticks, dead ticks, lowest baskets and depth in
all 81; the last window (22,000 ticks at the base) in tolerance from tick 686; a tick with no
baskets in each of the 16 × 0.5 windows (OW3).

### E8 and E9: the line control and the edges

IC1 at 25,000 ticks: 103/103 and Tier 3S 20/20 CONVERGED, each run's ticks the mirror's. Its
numbers are reported, as registered; they differ from the mirror's in two ways, neither a class
or a tick: at IC1 the task margin is active and the depth is below 0 at rest, so every tick is a
"breach" and the mirror's count stops at its early stop (about 4,000) where the engine's runs to
25,000; and two Tier 3S runs (`coin.desk.services*0.5`, `coin.desk.goods*0.5`) have 0 dead ticks
against the mirror's 2, and N(2) one fewer dead tick than the mirror's in each of three markets. E9 ([edges.csv](edges.csv)): the harness's
point at the base and every target gives every gap of SPEC §2.4 within 5.6e-16 of the 50-digit
values; the least over the targets is 0.21636 (land.mach 0.8's depth), above 0.2.

## Files

Here: [verdicts.csv](verdicts.csv), [tiers.csv](tiers.csv), [battery.csv](battery.csv) (every
compared run: class, ticks, troughs, depths, dead and breach ticks by market, largest shares,
participation, engine beside mirror), [families.csv](families.csv), [kicks.csv](kicks.csv),
[envelope.csv](envelope.csv), [readouts.csv](readouts.csv), [history.csv](history.csv),
[edges.csv](edges.csv), [fails.csv](fails.csv) (the one failed line). Every scored line of both
instances is in `../p23-wave/lines.csv.gz`, and the scorer's printout in `../p23-wave/score.out`.
Plots in `../../figs/wall/`: fig1 ticks to tolerance, engine against mirror, every CONVERGED run;
fig2 JB(0.5)'s breach; fig3 the three cost shocks' baskets.
