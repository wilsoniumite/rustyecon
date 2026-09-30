# PHASE2-S1: Phase 2 proper's first session, the wall and the open commons (P2.3)

Dated 2026-09-30. Steps P2.3.0–P2.3.16 on branch `phase2-proper`, from `reboot` at `f7d1eae`.
Rulings: decisions 360–393 (P2.3.0). Registered before any code: the wall frame
[wall/SPEC.md](wall/SPEC.md) (sha256 `cc2b6d1c…9be0`, P2.3.1) and the commons frame
[commons/SPEC.md](commons/SPEC.md) (`60f21f56…b40d`, P2.3.5, with Tier 3S). Dated amendments: the
wall's A1 (E0, P2.3.3) and A2 (E6, P2.3.16); the commons' A1, A2 (E0, P2.3.7–8) and A3 (two bands,
P2.3.16). As built: [WALL-RULES.md](WALL-RULES.md), [COMMONS-RULES.md](COMMONS-RULES.md). Results:
[results/wall/](results/wall/README.md), [results/commons/](results/commons/README.md), the
wave's machinery [results/p23-wave/](results/p23-wave/README.md).

**Conditions of every number below, unless its line says otherwise.** The many-market roles
(basket provider, basket workers, category desk, type desk) with the wall's three optional fields
or the workers' optional exit; C2m (every labour market at 5.2 a year), 52 ticks a year, ρ 0, the
fixed basket, `Saturate`, planned assignment, no government, no loop. Tolerance 1e-3 in log on 18
observables (wall) or 22 (commons); L from the engine's elasticity probe, 22,000 ticks at IW1,
141,000 at C1 and 142,000 at C2; Tiers 3 and 3S again at 10·L. The binaries `markets` from
`198554a` (sha256 `3f957fb0…2371`) and, for 54 reruns, `c4adbf8` (`be3266ab…42e9`), WSL release.
"The mirror" is the registered `wm.py` (wall) or `cm.py` (commons).

## 0. The verdict

| id | what it is | mode A | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | kicks | verdict |
|---|---|---|---|---|---|---|---|---|
| I0 | Appendix B's county (P2.1) | PASS | | | | | | GO (P2.1) |
| IW1 | 1d's B economy, three worker types, a solved wall at x\* = 1 | 3.3e-16 | 26/26 | 38/38 | 39/39 (39/39) | 20/20 (20/20) | 13/13 | **GO**, reserved-only types |
| IC1 | IW1 with the entrant's χ_max 0.25, on the line | 3.3e-16 | 103/103 in all | | | 20/20 | | control, as registered |
| C1 | I1 with one priced type (exit in food), the commons full | 2.0e-15 | 30/30 | 42/42 | 43/43 (43/43) | 24/24 (24/24) | 13/13 | **GO at C2m, a point result** |
| C2 | as C1, the commons with room | 5.6e-16 | 30/30 | 38/38 + 4 V | 41/41 + 2 V (the same) | 24/24 (24/24) | 13/13 | **GO** |
| C1N | C1 at χ_max 0.25 (negative control) | | 100 CONVERGED, 9 DIVERGED, 6 VACUOUS in all | | | | | as registered |

**GO for IW1, C1 and C2, as registered, and narrower than the run first read it.** Every verdict
line holds, and in 3,633 compared runs (1,208 at the wall, 2,425 at the commons) every class is
the mirror's and every tick to tolerance of the 3,397 CONVERGED ones is the mirror's to the tick.
Every CONVERGED run ends within 1.04e-13 in log of its oracle point; every kick set decays. The
new finding, the subsistence trap, falls in exactly the 203 runs the mirror put it in, and in no
verdict run. 124 of 59,788 scored lines failed, none a class, a tick or a verdict: 65 runaway
ticks held to another price reference than the mirror's, 58 shadow rents read against the
mirror's early stop, and one wall line carrying the 52-a-year stall at 365 ticks a year. Dated
amendments written after the wave re-read all 124 (§5). The reviews narrow the GO three ways:
- **C1 has no margin in the dials.** Two of its registered Tier-3 runs fall into the trap with
  every price rate × 0.9, and one with every buffer × 1.1. So PLAN's "green with margin" is not
  met at C1; C2 and IW1 keep their Tier 3 at the same moves, and I1 those two runs at ±25%.
- **The wall's GO does not test the type switch.** The trained and master sell only reserved
  hours (ε 0), so no agent chooses between the pool and a reserved market (O97).
- **The zero-rent answer is the commons' idle part, for one plot-taking type.** No land market
  at zero rent runs; idle enclosed land at r = 0 is in no instance (§3).

## 1. What was built

- **Roles, no new kind, state or market rule.** At the wall (decision 395): the category desk's
  `tail` (hours at tasks closed to machines) and `reserved` (a type's hours bought on its own
  labour market), and the provider's `more` (further transfers). At the commons (398): the
  workers' `exit`, hours = min(max(n(0), N − T_o/h), n(r̂)), which rations the commons from the
  pop's own params and posted prices, and a land buy for plots that spill onto enclosed land. Each
  field is skipped when absent; every committed tape keeps its text, ids and hash streams (R1).
- **Instances, tapes, harness.** IW1 and IC1 from unit 1d's `WorkerEconomy`, C1, C2 and C1N from
  1e's `ParcelEconomy`, each within 1e-13 of the frames' 50-digit solves; the tapes
  `markets-iw1.ron`, `markets-c1.ron`, `markets-c2.ron`. The wall reads each desk's threshold
  x = 1 − s in log; the commons reads the plots' regime and shadow rent through the rule.
- **Tests and gates.** 30 new tests and 2 fix tests, each failing without its change; 50 mutants,
  each killed. `gate.sh` and `gui.sh` green on WSL and Windows; the pins unmoved (gate
  `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`).
- **The wave.** 3,721 jobs and 24 R1 checks on WSL, the scorer committed before it (311). Four
  machinery faults, each fixed in its own commit and disclosed: the wall's basket count (P2.3.10,
  before the wave), dated commons shocks (P2.3.12), the runner's quoting (P2.3.13), a column.

## 2. The prediction and the result

**The wall, IW1** (SPEC §6–§7, registered at P2.3.1):

| registered | result |
|---|---|
| E0: the mirror within 1e-12 in log | 1.31e-14 over 2,000 ticks; four tick-1 residues of 3.4e-17 absolute, allowed by A1 (written after a development diff, before E0's run) |
| E2: mode A below 1e-9; L 22,000, 20,000, 164,000 | 3.3e-16, 4.4e-16, 4.4e-16; L equal; goods' τ 106.4 ticks |
| E3: GO; medians (slowest) 340 (452), 416 (575), 561 (705); 3S 404 (550) | **Held**, every run's ticks the mirror's |
| E4: base decay 0.534 a year; the technique share at e^(−0.05) a tick | 0.5338; within 2.2e-16 |
| E5: land.mach × 2 bottoms at 0.165 of Y\* (93 dead ticks); tail × 2 at 0.516, reserved × 2 at 0.500 | 0.1648 (93), 0.5157, 0.5000, while Y\* does not move (O98) |
| E6: only JB(0.5) breaches (11 ticks, depth −0.444); displaced shares stall at 5e-323 | Held; **4 runs at 365 a year stall at 3.46e-322**, the stall of that tick length (A2) |
| E7: stocks 31/31, joint2 60/60, joint4 40/40, basin 516/516; 12, 365 a year, Hold, tilt 1 64/64 | **Held**; history's 81 windows the mirror's |
| E8, E9: IC1 103/103 and 20/20; every edge within 1e-12, none below 0.2 | Held; edges within 5.6e-16, the least 0.21636 |

**The commons, C1 and C2** (SPEC §5.5–§6.9, registered at P2.3.5):

| registered | result |
|---|---|
| E0: the mirror within 1e-12 | 2.6e-14 at C1 and C2; C1N's `p[mach]*0.5` parts from tick 41 (the trap amplifies rounding), under A1 and A2 (A2 written after E0's official run) |
| E2: mode A; L 141,000, 142,000 | 2.0e-15, 5.6e-16; L equal |
| E3: GO; medians C1 319/372/508, C2 315/360/514; every run ends in its regime, r_o within 1e-9 | **Held**; 224/224 in regime; r_o within 3.0e-13 of the oracle's |
| E4: stocks 41/41 each | Held (medians 419, 444 ticks) |
| E5: the trap in C1 4/60 joint2, 12/40 joint4, 89/430 basin; C2 8/40, 77/430; runaway ticks within 5% | Every class, seed and run name; **64 runaway ticks beyond 5%**: the reference (A3) |
| E6, E7: C1's land.mach history runs away at tick 7,788; tilt 1 C1 3 DIVERGED | Held, tick for tick |
| E8: 12 and 365 a year; r_o at the end within 1e-9 of the mirror's | Classes and ticks held; **58 r_o lines fail** against the mirror's early stop (A3) |
| E9: enclose 4/4; C1N 100 + 9 DIVERGED + 6 VACUOUS | Held; one runaway tick 293 against 279 (A3) |
| Refutations (both frames) | None, on A2's reading of the wall's end line |

## 3. The zero-rent remedy

The hard point was a land market at zero rent: under `Saturate` its unsold price falls to the
runaway bound, and R3 forbids a clamp. The commons frame scanned six answers in its mirror before
any code (SPEC §1, 714 runs), as L0 chose the maker's reservation for the idle horse market
([IDLE.md](IDLE.md)):
- **The commons as a market**, under `Saturate` or `Hold`: exact while crowded, but where it has
  room (r_o\* = 0; 8 of C1's 115 battery runs, and C2) its price falls by a constant factor a tick.
  The real observables converge (within 1.1e-7 to 3.0e-7 in log); the price fails the log bound
  after 1,126–11,590 ticks, then the positive range. A limit of what a market can post.
- **The commoners' reservation**, L0's device at ψ·r, chatters at ψ·r where the oracle's rent is
  0 (12 and 8 ORBITING); **a floor order** rests at any rent above r_o\* (72 of 115 STUCK).
- **Chosen (398): the commons is no market.** The workers hold it and the participation rule
  gives out its plots; the shadow rent is a readout. 115/115, exact in all three regimes, half-life
  49 ticks against 876, no price floored. The engine ran it exactly as registered.

What it does not answer: **idle enclosed land at r = 0** is unfunded under the probe's transfer
(the provider's only income is rent, so its baskets are −ν·N there), and a positive-price market
cannot rest at 0. Decisions 153, 160, 161, 376 and 381 are untested. **A commons shared by several
types** needs each other's plot demand, which R13 forbids (O101). Both need one thing the engine
lacks: a zero price markets can hold, a free-good state (O96 with O101).

## 4. The subsistence trap and C1's margin

With the exit valued at food's posted price, nobody working is a second absorbing state: workers
leave, food is not supplied, and every price inflates together. The frame found it in the mirror
and registered it in the families; the engine put it in exactly the same 203 runs. I1, with no
exit, never enters it (0 of 530). The fidelity review traced three: the exit's value over the wage
reaches 1.0–1.1 by tick 20, participation falls to 0.002–0.04 and then 0, food supply to 0, and
prices cross the bound at ticks 287–369.

How close C1 sits to it: on the basin grid the machine price's edge lies between × 0.481 and
× 0.458, 0.04–0.06 in log from the verdict run at × 0.5; every other basin direction at C1 and
C2 has 0.23 or more. At rest the exit is worth 0.529 of the wage at C1 and 0.354 at C2. The
review's hypothesis, recorded untested: the trap sits closer where the exit is worth more, and a
1750-like exit (a cottager's plot) may be worth more than C1's.

## 5. What the reviews found, and the corrected verdict

Two reviews ran after P2.3.15. **Both say the verdicts hold**, and neither changes a class.

**Measurement.** Fresh builds of `198554a` and `c4adbf8` give the wave's binaries byte for byte;
110 stratified jobs rerun byte for byte (526 files), 14 on a Windows build; `gather.py` and
`score.py` reproduce `runs.jsonl` and `lines.csv`; an independent rescore from the raw summaries
gives every class and tick of the 3,633 runs; its own 50-digit solves match both instances.
Three minors, each answered at P2.3.16:
1. **The wall's refutation line.** The committed scorer prints "refutations: ['CONVERGED runs
   ending at the wall (every share 0 or at most 5e-323, x = 1)']"; "none met" was a reading after
   the result. The READMEs quote it; amendment A2 states the stall at each tick length.
2. **"Nothing changed after a result was read"** is scoped to the scored wave; the READMEs list
   the E0 amendments made after an E0 result (the wall's A1, the commons' A1 and A2).
3. **A shared target directory can give a stale binary.** The rule, one `CARGO_TARGET_DIR` per
   checkout and commit, is in `p23-wave/FIXES.md` and STATE's repro notes.

**Fidelity.** The roles match units 1d and 1e; the commons rule is not a clamp; the scored runs'
last rows equal the frames' 50-digit points. Two majors and three minors:
- **C1's dial margin** (major; §4). Answered: the commons README and STATE call C1's GO a point
  result at C2m with its neighbourhood; O100's remedy becomes a prerequisite of PLAN's gate and
  of the phase diagram for every priced-exit instance; the next wave registers a
  dial-neighbourhood family (±10% and ±25% of rate, buffer and adjust; tilt) on C1, C2 and IW1;
  the exit-value hypothesis is recorded (decision 399, amended). A spot check reran eleven of the
  review's runs on the wave's binary: each gave the review's class.
- **The zero-rent answer is narrower than the headlines** (major; §3). Answered: decision 398's
  summary and next step 7 say it covers one plot-taking pop and runs no land market at zero rent;
  the 1750-like instance's prerequisites now list O95, O96, O97, O100 and O101; O96 and O101 are
  one problem, a free-good state.
- Minors: the wall's verdict carries its qualifier (O97); "runs away" for the market form reads
  as a representation limit (§3; 398, O96, O101); O100 gains a candidate aimed at the cause. The
  exit is valued at a posted price for food the exiters never sell, so nothing refills an empty
  food market; selling home output is 1e's open question 6, an oracle addendum that changes
  goods clearing (home food joins food's supply, rent is paid in money) and so the point.

**The amendments' effect.** `score.py --amended` reads A2 and A3; without the flag every line is
as committed. With it 124 lines go from fail to pass, 1,059 others change their reference and
still pass, none goes from pass to fail, and no refutation is listed. O107 is ruled (the
harness's reference); O108's readings stay open to veto.

**The corrected verdict.** IW1 GO (52 ticks a year, C2m, ρ 0), for reserved-only types beside one
pooled type; the switch between pooled and walled is untested. C2 GO. C1 GO at C2m as
registered, a point result: at ±10% of the rate and buffer dials it would be LOCAL, through the
subsistence trap.

## 6. What this means

**For PLAN's Phase 2 gate.** Of its four stationary configurations, the Appendix B instance (I0),
a wall-regime instance (IW1) and an open-commons instance (C2; C1 without margin) are now green at
C2m; the 1750-like instance is not built. "Green with margin" and "a stable region documented and
referenced by the default dials" are not met: C1 sits at the corner of its stable region, and no
region is mapped. A11's kill condition is not triggered (no registered mode B failed); its session
budget is still yours.

**For the 1750-like instance** (goods chain or flow; decisions 284, 285, 308). It combines what
this session split apart: a priced exit in food (the trap, O100), several types on one commons
(O101), a trained type that moves between the pool and its reserved tasks (O97), a walled type
under s(q) (O95, 1e's addendum, decision 377), and perhaps idle land (O96). As a goods chain it
adds a loop by its own registration at C2g (308, 391), storable running goods (O51) and a funded
steam county (O62). My recommendation: register it first as a flow instance at C2m on these
roles, loop-free as I1–I3, C1 and IW1 are, so that its failures are the new margins' and not the
chain's; then the goods chain's version by its own registration. Neither can start before O100's
remedy, O97's rule and O101's free-good state are scanned in the mirror.

**For the phase diagram.** C1 shows its boundary is near the default dials, and that it is the
trap, not a slow mode. The dial-neighbourhood family is its first slice, on the roles as they are;
the map proper waits for O100's remedy. Four dials at five values over three Tier 3s is about
2,500 runs, one wave of this size.

**For G2** (lenses, after this gate). Nothing here moves it forward. The wall's depth and the
commons' regime, shadow rent and participation are in `stats.tsv`, ready for its first lenses.

**Recommendation.** Take IW1, C1 and C2 as GO, read as §5 narrows them, with amendments A2 and A3
and decisions 360–399 as amended. Next, in order: one registered wave of O22's families on I1–I3
(O109) and the dial-neighbourhood family on C1, C2 and IW1; O100's remedy scan with the
home-output candidate; O97's rule and the free-good state (O96, O101), each chosen by a mirror
scan; then the 1750-like instance as a flow instance.

## 7. Open questions

1. **The trap's remedy** (O100). Which of participation at a rate, entry for food at zero output,
   a storable exit good or home output sold restores C1's margin, and at what cost to the point?
2. **A zero price markets can hold** (O96, O101). Can a free-good state be added to `markets`
   without moving a pinned hash, and does it close idle enclosed land and a shared commons?
3. **The type switch** (O97). What rule splits a pop's hours between the pool and its reserved
   market, and does it keep the wall's GO at the four targets where the trained would pool?
4. **Paths at the wall** (O98, O99, O14). Labour-demand shocks halve output on the way while Y\*
   does not move; ex-post assignment is untested here.
5. **Not tested:** ρ > 0, a government (1f), several plot-taking types, a CES basket, 12 ticks a
   year as a default, and the 1750-like instance's dials.
