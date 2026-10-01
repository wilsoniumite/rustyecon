# TRAP-SPEC: the subsistence trap's remedy (O100), chosen by a mirror scan

Dated 2026-09-30. Work label `scan-trap`, for Phase 2 proper's second session (P2.4; PHASE2-S1
§6, item B) on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24` at `a483ed0`; nothing in the
worktree was changed). Scratch: `D:/rustyecon-p24/scan-trap/`. Every number below is the mirror's
(`model/tm.py`, a copy of the registered commons mirror `cm.py` with the candidates) at C2m, 52
ticks a year, ρ 0, tolerance 1e-3 in log, the commons frame's L (141,000 ticks at C1, 142,000 at
C2) and early stop, unless the line says otherwise. The evidence is in the files of §12, whose
sha256 are in `SHA256SUMS` beside this file.

## 0. The answer in brief

**GO: participation at a rate** (the workers' `exit.pace`). The share of heads offering hours
moves a share a = 1 − exp(−1.3/tpy) of its gap to the participation rule's share each tick
(0.02469 at 52 a year: half the gap closes in 27.7 ticks), instead of jumping to it. Only the
hours lag; the plots follow the rule at posted prices. The rest point is the oracle's, unchanged,
so no oracle addendum is needed. With the field absent the role is P2.3's bit for bit.

What it does, on the registered instances C1 and C2 (§3):

| | registered rule | paced, 1.3 a year |
|---|---|---|
| C1 Tier 3 at the 17 dial settings (rate, buffer, adjust × 0.75, 0.9, 1.1, 1.25; tilt 0.05–1) | 18 runs in the trap, at 7 settings | none |
| C1's edge in the dials (Tier 3, all 43 runs converge) | rate ≥ 0.95, buffer ≤ 1.05, tilt ≤ 0.1, adjust 0.25–1.5 | rate 0.5–3, buffer 0.25–2, tilt ≤ 1, adjust 0.25–4 (the whole grid but tilt 2 and adjust 0.1) |
| basin, C1 / C2 (430 each) | 89 / 77 in the trap | 0 / 0 |
| joint2, joint4, C1 / C2 | 4, 12 / 0, 8 | 0, 0 / 0, 0 |
| C1's land.mach history | runaway in window 5 | 81/81 windows |
| the negative control C1N (χ_max 0.25), Tier 3 | 9 in the trap | 0 |
| the point, every target | the oracle's | the oracle's (within 2.4e-15 in log) |
| largest root per tick, C1 / C2 base | 0.986104 / 0.985969 | 0.986127 / 0.986524 |
| Tier 1/2/3 median ticks to tolerance, C1 / C2 | 319/372/508 / 315/360/514 | 319/369/516 / 326/387/524 |

Every one of the registered rule's 218 trap runs in this scan converges under the pace, in 416–897
ticks (median 705), with participation never below 0.020 of its target. On a held-out instance
defined after the choice (C1 with a dearer exit, worth 0.696 of the wage at rest against C1's
0.529), the registered rule is LOCAL at C2m itself (4 Tier-3 runs trapped) and fails 16 of the 17
dial settings; under the pace every run of the same sets converges (§3.8).

**The named alternative (R6): home output sold** (candidate H, 1e's open question 6). It clears
C1's dial neighbourhood too and keeps the paths, but moves the point (x\* by up to 0.010, the
baskets by 8.6%), needs an oracle addendum, and leaves 38 of C1's and 40 of C2's basin runs and 5
of each's joint4 runs in the trap. **Rejected:** the food entrant (E) keeps the point and clears the
dial neighbourhood but leaves 42–44 basin runs in the trap and slows Tier 2 by 37–69%; the
storable exit good (S) makes the rest point locally unstable at C1 (largest root 1.0009 with a
cover of 13 ticks, 1.0068 with 4).

**What it does not do.** The trap's second attractor remains: beyond the stable region (every
tilt 2) three of C1P's Tier-3 runs still collapse, now slowly and without a tick of zero hours.
The rate was chosen on the families it is judged on (§4.3, disclosed); a held-out instance with a
dearer exit (§3.8) was run after the choice.

## 1. The trap in the mirror

### 1.1 What happens (C1, every price rate × 0.9, `p[mach]*0.5`; `scan/trace1.py`)

1. The machine's price is halved. At posted prices a machine does every task more cheaply (X = 1),
   so every desk moves its human share toward 0 at its adjust rate: 0.252 to 0.10 by tick 18.
2. Labour demand falls with it, from 0.51 to 0.22 hours a tick. The machine desk sells at half
   price, so its coin falls from 21 to 10 and its stock from 4.7 to 2.3 by tick 18.
3. The wage falls against food. The exit's money value over the wage, e₀/w = p_food·s₀/w, rises
   from 0.53 to 1.00 by tick 20. Participation falls from 0.54 hours a tick to 0.17 (tick 20) and
   0.064 (tick 30). Food's output falls from 5.2 to 0.56 a tick.
4. From tick 40 the exit ties the wage to food's price: e₀/w stays between 0.99 and 1.13. The
   wage inflates with food, faster than the machine's price (rate 1.3 a year), so the desks stay
   fully automated and hire no hours; only the machine desk does.
5. From tick 105 hours are exactly 0 on alternate ticks: F(e) is 0 wherever w ≤ e, the support
   of χ. With no hours the machine desk makes nothing, and it cannot restart from a zero stock,
   since its own output is an input (a_kk 0.3). Its stock is 0 from tick 110. Nothing is made
   after. Every price inflates, and the runaway bound is crossed at tick 325.

In this scan's baseline (the registered rule on every set of §3), every one of the 217 trap runs
that records it has 132–208 ticks with no hours offered; 11 of 2,536 converging runs have one,
each at tick 0, where the machine's genesis lot carries it (`scan/zero_tick.out`). The zero-hour
tick is how the trap becomes absorbing: with the machine's stock at 0, no price path restarts it
(the loop's absorbing zero, O21, with the exit in the chain).

### 1.2 How close C1 sits to it

The registered mirror on the 17 settings of the dial neighbourhood (`pred/controls.jsonl`, the
harness's runaway reference, O107):

| setting | C1 Tier-3 runs in the trap | runaway tick (harness's reference) |
|---|---|---|
| rate × 0.9 | `p[mach]*0.5`, `JB(2)` | 325, 335 |
| rate × 0.75 | `p[mach]*0.5`, `JA(0.5)`, `JB(2)` | 379, 380, 391 |
| buffer × 1.1 | `JB(2)` | 309 |
| buffer × 1.25 | `p[mach]*0.5`, `JA(0.5)`, `JB(2)` | 287, 287, 297 |
| tilt 0.25, 0.5, 1 | `p[mach]*0.5`, `JA(0.5)`, `JB(2)` each | 295, 288, 317; 293, 287, 305; 295, 287, 297 |
| the other 10 settings | none | |

The engine's recheck at P2.3.16 ran the first and third rows on the wave's binary: the same runs
at the same ticks (325, 335; 309). C2 has none in the neighbourhood.

## 2. The candidates, as scanned

Each is a copy-and-add in `model/tm.py`, marked `scan-trap`, read from the exit block and absent
by default. Each reads posted prices, its own state and its own params (R13), floors or caps no
price (R3), and rests at its oracle's point (§3.2).

- **P, participation at a rate** (`prate`, a year; variants 0.65, 1.3, 2.6, 5.2 and 13). The
  workers keep their share F in their state. Each tick F = F₀ + a·(F\* − F₀), with F\* = hours/N of
  the participation rule at posted prices and a = 1 − exp(−rate/tpy); they offer N·F hours. Two
  forms of the plots: at the hours offered, h·(N − N·F) − T_o where a plot pays at r (P), or the
  rule's T_p (Pr, only the hours lag). Genesis F is the point's S/N.
- **E, entry for food at zero output** (`entry`, its coin a multiple of the food desk's genesis
  coin; 0.1 and 1). A food entrant with the all-human recipe, L̄_food = 0.725 hours and b_food land
  a unit, no machine. It buys its inputs only while food's posted price exceeds its unit cost
  w·L̄ + r·b at posted prices (a step on its own order), at the desks' turnover and tilt, and sells
  all it makes. It starts with no food. At every interior point it is dormant: its cost exceeds
  food's price by ∫₀^x\* μ(t)(w − p_τ·γ(t)/θ)dt > 0, so its markup is at most 0.920 over the 26
  targets.
- **S, a storable exit good** (`store`, a cover of 4 or 13 ticks). The food desk offers
  held/(1 + cover) and carries the unsold rest, unspoilt (GOODS-CHAIN E1's seller). At rest it holds
  (1 + cover) ticks of output and sells one.
- **H, home output sold** (`home`). The exiters' food, s₀ a plot and s̲ for an exiter without one
  (plots = min(N − hours, (T_o + T_p)/h)), is offered in full on food's market each tick, a
  one-tick good, and its proceeds are the workers' coin. It changes the equilibrium: home food
  joins food's supply, so the desks make z_food·Y − H_f of it, land clears T − T_p = Y·ℓ − H_f·ℓ_g
  and labour Na = Y·a − H_f·a_g, with ℓ_g and a_g the land and hours embodied in a unit of food
  (`tm.oracle_at`, f64). At C1 home food is 16.7% of food's market; x\* moves 0.74834 → 0.74976 and
  the baskets 5.716 → 6.210, and at two C1 targets x\* crosses care's edge 0.75 (land.mach × 0.9:
  0.75600; b.food × 0.9: 0.76170; O105). Its 50-digit check was not written, since H is not chosen.

## 3. The scan

### 3.1 Nesting (`model/tm_nest.py` → `tm_nest.out`)

With every candidate absent, `tm` is the registered `cm` bit for bit: the oracle, genesis and every
state value on every tick of 316,948 ticks (C1, C2, C1N, I1, I0; the base and four cost targets;
six displacements; up to 3,000 ticks each), and every value of 20,000 random states one tick each.
Three branches were added to `tm.py` and `tb.py` after the first scans had run, each unused by
them: Pr's rule plots (`tprule`), the paced share's displacement (`part.workers`) and the runaway
bound on the harness's reference (`ref`). The nesting check ran again on the final files, and 775
scan runs (C1, C2, Pr, H and E 0.1; battery and joint4) rerun on them give the scan's class, ticks,
dead ticks, peak, baskets and error in every run.

### 3.2 The rest point (`model/restcheck.py` → `restcheck.out`)

At all 26 targets (C1 and C2, each base and its 12 cost targets), from the oracle's f64 point,
every candidate leaves every observable within 2.6e-15 in log over 3 ticks and 9.9e-15 over 300,
every fill at least 1 − 1e-14. H's point is its own oracle's. Mode A of the chosen rule at L passes
at 12, 52 and 365 a year, within 2.2e-14 (§9.1).

### 3.3 The families and the dial neighbourhood (`runs/*.jsonl` → `runs/scan.out`)

Each candidate ran the registered battery (Tiers 1–3 at L), the 17 dial settings on Tier 3,
joint2, joint4, basin and history on C1 and C2, and the negative control's Tier 3: 2,799 runs a
candidate. A failure is any class but CONVERGED or VACUOUS; in every set all failures are the
trap's runaway, except where the table says otherwise. The 17 settings are the session brief's:
every price rate, desk buffer (turnover) and adjust rate × 0.75, 0.9, 1.1 and 1.25, and every tilt
at 0.05, 0.1, 0.25, 0.5 and 1, as the engine's `--set` families move them (decision 399 (4) names
× 0.8 in place of × 0.75; the edge grid of §3.6 has it).

| candidate | C1 dial settings failing (runs) | basin C1 / C2 | joint2 C1 / C2 | joint4 C1 / C2 | C1 history | C1N Tier 3 | C1 Tier 1/2/3 median (slowest T3) | PL C1 / C2 base |
|---|---|---|---|---|---|---|---|---|
| none (registered) | 7/17 (18) | 89 / 77 | 4 / 0 | 12 / 8 | runaway, window 5 | 9 | 319/372/508 (828) | 0.986104 / 0.985969 |
| P 0.65 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 81/81 | 0 | 319/371/517 (713) | |
| P 1.3 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 81/81 | 0 | 319/369/515 (696) | 0.986127 / 0.986524 |
| **Pr 1.3 (chosen)** | **0** | **0 / 0** | **0 / 0** | **0 / 0** | **81/81** | **0** | **319/369/516 (698)** | **0.986127 / 0.986524** |
| P 2.6, Pr 2.6 | 1/17 (2: rate × 0.75) | 0 / 0 | 0 / 0 | 0 / 0 | 81/81 | 0 | 319/370/516 (854) | 0.986127 / 0.986281 |
| P 5.2 | 5/17 (8) | 43 / 24 | 2 / 0 | 6 / 4 | 81/81 | 0 | | |
| P 13 | 1/17 (2) | 38 / 18 | 0 / 0 | 6 / 4 | 81/81 | 0 | | |
| E 0.1 | 0 | 42 / 40 | 0 / 0 | 2 / 3 | 81/81 | 0 | 326/510/595 (997) | |
| E 1 | 0 | 44 / 42 | 0 / 0 | 2 / 3 | 81/81 | 0 | 326/630/669 (1097) | 0.986104 / 0.985969 |
| H | 0 | 38 / 40 | 0 / 0 | 5 / 5 | 81/81 | 0 | 288/357/480 (697) | 0.986444 / 0.986146 |
| S 13 (stopped) | C2: 13/17 (160, of them 150 ORBITING) | C1: all 246 runs made fail (191 ORBITING, 55 DIVERGED) / C2 51 | – / 0 | – / 6 | 81/81 | | C2 battery: 8 ORBITING; medians 17,000–22,000 ticks | 1.000888 (C1) / 0.999814 |
| S 4 (stopped) | C2: 2 of the 42 runs made fail | | | | | | | 1.006840 (C1) |

C2's dial neighbourhood has no failure under any candidate but S. Every C1 and C2 battery run
converges (or is VACUOUS by construction) under every candidate but S. Medians exclude P2.1's slack
runs, as registered. S's scans were stopped by hand once their local roots were known (§3.4); their
runs orbit to L.

### 3.4 The local roots (`lin/*.json`; the commons frame's method, 8 directions, ticks 10,000–50,000)

- **Pr 1.3**: 0.985261–0.990038 at C1's 13 targets (base 0.986127, half-life 50 ticks) and
  0.985569–0.988657 at C2's (base 0.986524, 51). The registered rule: 0.985366–0.991369 and
  0.985546–0.987860. The pace's own mode, 1 − a = 0.9753 a tick, is never the largest. Every kick
  bar PL^(0.9·L) ≤ 1e-3 passes by a wide margin.
- **E**: the registered rule's roots exactly (the entrant is dormant near the point, and its
  hoard is left out of the coordinates).
- **H**: 0.985530–0.990272 (C1), 0.985495–0.987639 (C2), at its own point.
- **S**: C1 1.000888 (cover 13) and 1.006840 (cover 4); C2 0.999814 (cover 13, half-life 3,726
  ticks). The stock and the price chase each other: an inventory cycle that does not damp at C1.
- L is unchanged by the pace: the elasticity probe on C1P and C2P gives τ_max 703.3 and 705.4 ticks
  (care's price), so L 141,000 and 142,000 (26,000 at 12 a year, 1,113,000 and 1,116,000 at 365)
  (`lin/elasticity-P1.3r.json`).

### 3.5 How the pace ends the trap

Under the pace F_t ≥ (1 − a)^t·F₀ > 0, so no tick is ever without hours (0 zero-hour ticks in
every Pr run of the scan and the predictions), and the machine's stock never reaches the absorbing
0. The wage is no longer tied to food by the exit within a tick: participation lags, so the wage
can fall below e₀ for a while, the desks de-automate, and labour demand returns. In the 218 runs
that are trapped under the registered rule, participation falls at worst to 0.020 of its target
and e₀/w rises at worst to 39.9, and every one recovers: median 705 ticks to tolerance, slowest
897, with up to 325 dead ticks and baskets at their lowest near 0 (`scan/former_trap.out`). The
paths are deep; they end.

### 3.6 The edges (`runs/edges.jsonl` → `runs/edges.out`)

Tier 3 of C1 and C2 on a finer grid (rate × 0.5–3, buffer × 0.25–2, adjust × 0.1–4, tilt 0.01–2;
30 settings beyond the 17). A cell is the number of Tier-3 runs of 43 that fail.

| setting | C1 | C1, Pr 1.3 (P 1.3 the same) | C1, P 2.6 | C1, E 1 | C1, H | C2 | C2, Pr 1.3 (P 1.3 the same) |
|---|---|---|---|---|---|---|---|
| rate × 0.5 / 0.6 / 0.7 / 0.8 | 6 / 4 / 3 / 3 | 0 | 6 / 3 / 2 / 0 | 0 | 0 | 1 / 1 / 0 / 0 | 0 |
| rate × 0.85 / 0.95 | 2 / 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| rate × 1.5 / 2 / 3 | 0 | 0 | 0 | 0 / 0 / 2 | 0 | 0 | 0 |
| buffer × 1.05 / 1.15 / 1.2 / 1.3 | 0 / 2 / 2 / 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| buffer × 1.4 / 1.5 / 1.75 / 2 | 3 / 4 / 5 / 6 | 0 | 0 / 0 / 0 / 1 | 0 | 0 | 0 | 0 |
| buffer × 0.25 / 0.5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| adjust × 0.1 | 23 ORBITING | 28 (DEAD, DIVERGED) | 30 | 14 | 24 | 0 | 15 |
| adjust × 0.25 / 0.5 / 1.5 | 0 | 0 | 0 / 0 / 1 | 0 | 0 | 0 | 0 |
| adjust × 2 / 4 | 2 / 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| tilt 0.01 / 0.02 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tilt 2 | 5 | 3 | 2 | 1 | 3 | 1 | 2 |

So under the pace at 1.3 a year C1's stable region (every Tier-3 run converging) covers every
price rate from × 0.5 to × 3, every buffer from × 0.25 to × 2, adjust from × 0.25 to × 4 (which
moves the pace with the techniques) and tilt up to 1; it ends at tilt 2 (the trap, slowly: 0
zero-hour ticks, runaway at 308–314 on the harness's reference) and at adjust × 0.1 (a different failure: techniques and
participation 10 times slower; the registered rule orbits there). The registered rule's region at
C1 ends at rate × 0.9–0.95, buffer × 1.05–1.1 and tilt 0.1–0.25.

### 3.7 Paths (non-slack CONVERGED runs of the battery; `scan/tiers.out`)

| | Tier 1: median (slowest); lowest baskets | Tier 2 | Tier 3: median (slowest); dead ticks worst |
|---|---|---|---|
| C1 registered | 319 (444); 0.632 | 372 (557); 0.131 | 508 (828); 183 |
| C1 Pr 1.3 | 319 (444); 0.632 | 369 (545); 0.131 | 516 (698); 150 |
| C2 registered | 315 (494); 0.688 | 360 (547); 0.205 | 514 (641); 153 |
| C2 Pr 1.3 | 326 (506); 0.621 | 387 (545); 0.161 | 524 (671); 150 |
| C1 E 0.1 | 326 (537); 0.245 | 510 (681); 0.037 | 595 (997); 192 |
| C1 H | 288 (465); 0.648 | 357 (489); 0.154 | 480 (697); 135 |

The pace costs C2 up to 7% in median ticks (Tier 2) and deepens its Tier-2 baskets from 0.205 to
0.161; at C1 it changes Tier 1 in no run and shortens Tier 3's slowest run. The entrant enters in
65 of C1's 115 battery runs, 6 of them in Tier 1, where it drives the lowest baskets from 0.632 to
0.245.

### 3.8 A held-out instance (`model/heldout.py` → `scan/heldout.out`; `runs/held.jsonl` → `runs/held.out`)

After the choice, C1 with a dearer exit: s₀ 0.4, h 0.18 (h/s₀ 0.45, C1's), its commons at the
log-middle of its Crowded band (32.67 a year; the band is 32.19–33.16). At rest e₀/w is 0.696
(C1: 0.529), the review's hypothesis for where the trap sits closer (decision 399 (5)).

Its point is Crowded, x\* 0.76099, participation 0.1274. The scan's sets, 1,378 runs a candidate:

| X4 | battery (115) | Tier 3 | dial settings failing (runs) | basin | joint2 | joint4 | land.mach history | Tier 1/2/3 median (slowest T3) |
|---|---|---|---|---|---|---|---|---|
| registered rule | 111 | **4 DIVERGED**: `JA(0.5)`, `JB(2)`, `p[land]*2`, `p[mach]*0.5` | **16/17 (82)** | 109 | 10 | 16 | runaway, window 3 | 354/395/568 (1,472) |
| **Pr 1.3** | **115** | 43/43 | **0** | **0** | **0** | **0** | 81/81 | 354/389/542 (812) |
| E 0.1 | 115 | 43/43 | 0 | 59 | 3 | 5 | 81/81 | 354/555/590 (1,702) |
| H | 115 | 43/43 | 0 | 54 | 2 | 8 | 81/81 | 355/415/553 (701) |

So the review's hypothesis holds in the mirror: with the exit worth 0.696 of the wage the
registered rule is LOCAL at C2m itself (four Tier-3 runs in the trap), and the trap is in 16 of the
17 dial settings. The pace clears all of it; the entrant and home output clear the verdict battery
and the neighbourhood but leave an eighth of the basin in the trap.

## 4. The choice

### 4.1 Participation at a rate, 1.3 a year, the plots the rule's (decision TR1)

- It is the only candidate that clears the trap from every scanned family: the dial neighbourhood,
  basin, joint2, joint4, history and the negative control, at C1 and C2 (§3.3). Its stable region at
  C1 covers the finer grid but tilt 2 and adjust × 0.1 (§3.6).
- It keeps the point, every target's, exactly, and the local roots (§3.2, §3.4). No addendum.
- It keeps the paths: Tier 1 as registered at C1, Tier 3's worst dead ticks lower at both (§3.7).
- It is the smallest change: one optional block on the workers' exit, one line of their rule,
  their existing state (§5). It is structural: households move between home and the market over
  time, as the desks move their technique at a rate (decision 60's partial adjustment).
- It reads posted prices and its own state (R13); F is a convex combination of two shares in
  [0, 1], so nothing is clamped (R3); absent, nothing changes (R1).
- **The plots follow the rule** (Pr): only the hours lag. It equals P in every class of the scan
  (2,799 runs at 1.3, 2,799 at 2.6) and in which runs fail on the edge grid (2,580 runs; at adjust
  × 0.1 five failing runs swap DEAD and DIVERGED), keeps T_p exact at rest, and needs no new plot
  formula.

### 4.2 The named alternative: home output sold (H)

It aims at the cause the fidelity review named: exiters value food at a price they never sell at,
so nothing refills an empty food market. Selling home output does refill it, clears C1's dial
neighbourhood and keeps the paths (Tier 3's median 480, the fastest). It is not chosen for three
reasons: it moves the point (a new instance, with an oracle addendum and its 50-digit check, 1e's
open question 6); it moves two of C1's targets across care's edge (O105); it leaves 38 and 40
basin runs and 5 joint4 runs each in the trap (54 and 8 at the held-out X4), because a trapped
economy's problem is the dead machine, not food alone. The extreme alternative is the registered
rule (none).

### 4.3 The rate: a forking path, disclosed

The rate was chosen from 0.65, 1.3, 2.6, 5.2 and 13 a year on the same families it is judged on.
Slower is better up to 1.3: 2.6 keeps one failing setting (rate × 0.75, two runs), 5.2 and 13 leave
18–43 basin runs trapped, and 0.65 is as clean as 1.3 but slower (C2 Tier 1 327 against 326; C1
Tier 3 517 against 515), and its own mode becomes the largest root (C1 0.987587, half-life 55
ticks; C2 0.987205; `lin/lin-P0.65-base.out`). 1.3 is also C2m's rate for land and the types'
prices, so the pace adds no new number to C2m's list of values (R7). What was not used to choose it: the finer edges (§3.6, run with the choice pending but not
read for it) and the held-out instance (§3.8, defined and run after it).

### 4.4 Why not the others

- **E, the entrant.** It keeps the point and its roots and clears the dial neighbourhood, but not
  the trap: 42–44 basin runs and 2–3 joint4 runs stay in (59 and 5 at X4). It enters in more than
  half the battery, Tier 1 included, and slows C1's Tier 2 by 37% (coin 0.1) to 69% (coin 1) and
  Tier 3 by 17–32%. Its coin at rest is a hoard, a neutral direction of the state.
- **S, the storable exit good.** Its rest point is the oracle's but unstable at C1 (PL 1.0009 and
  1.0068); at C2 its slowest mode has a half-life of 3,726 ticks and 150 of its 731 dial runs
  orbit. A storable running good needs a seller whose stock damps (O51's cover), not this one.

## 5. The rule as the engine should build it

### 5.1 The field (`crates/agents/src/roles/many/spec.rs`)

```rust
// in RawPricedExit, the one optional field (as the maker's `reserve`, L0.4, and the desk's `plant`):
/// `Option<RawPace>`: participation at a rate (O100's remedy; the trap scan's §5). `None` or
/// absent: the workers offer the rule's hours each tick, P2.3's role bit for bit; absent is not
/// written, so every tape written before the field keeps its canonical text, `tape_hash` and
/// `world_id`.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub pace: Option<RawPace>,

/// Participation at a rate, as the tape writes it: the technique's form on the workers' share.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPace {
    /// Param key, unit `RatePerYear`, live, read as a `Share` (`ClockMethod::Share`, as a
    /// technique's `adjust`): a = −expm1(−rate/tpy), the share of its gap to the rule's share the
    /// workers' share closes each tick. Required, no default.
    pub adjust: Key,
    /// The workers' share at genesis, F₀ in [0, 1]; the generator writes the point's S/N.
    /// Required, no default.
    pub share: f64,
}
```

It resolves to `Pace { adjust: Site, share: f64 }` in `PricedExit.pace`, the site
`live(r, &raw.adjust, ClockMethod::Share, "adjust")`; `BasketWorkers::sites` lists it as
`exit.pace.adjust`. `genesis_state` (`crates/agents/src/ext.rs`) gives the workers
`WorkersState { share }` with `share` = `pace.share` when the exit has a pace, and 0.0 otherwise
(unchanged). No state, kind, market or core change.

### 5.2 The rule (`rules.rs`, `BasketWorkers::decide_with_exit`)

After `let p = workers_participation(...)` (unchanged), with N = `p.heads` (the heads' hours a
tick) and s₀ = `v.own_state.share`:

```
without pace:  hours = p.hours;                        share = p.share        (P2.3's, bit for bit)
with pace:     a = param(pace.adjust)                  (a Share: −expm1(−rate/tpy))
               F* = p.hours / N                        (the rule's hours over the heads)
               s  = s₀ + a·(F* − s₀)                   (the technique's form, rounded as written)
               hours = N·s;                            share = s
```

The hours are minted and offered as now. The plots are the rule's, `p.plots`, and their land buy,
budget chain and burn are unchanged (the frame's §3.2). The state holds `share`. Nothing else in
the role changes. The harness's readouts of regime and shadow rent stay the rule's at posted
prices (`workers_participation`).

### 5.3 Load checks (each a `LoadError` at `actors[workers].spec.exit.pace…`)

1. `.share` finite and in [0, 1];
2. `.adjust` a `RatePerYear` param (the resolver's `UnitMismatch`), its value finite and not
   negative (the registry's).

### 5.4 The dial and the harness (`probe::markets`)

- **Instances** `C1P`, `C2P` and `C1PN`: C1, C2 and C1N (χ_max 0.25) with the pace, adjust key
  `adjust.participation.workers`, genesis share the 1e point's S/N. Oracle, targets, genesis prices
  and coins are C1's, C2's and C1N's.
- **The dial** `adjust.participation.workers`, 1.3 `RatePerYear`, joins C2m for an instance with a
  pace only (basis `Assumed("the trap scan's §4: participation at land's and the types' rate")`).
  `--set adjust.*=f` scales it with the techniques; `--set adjust.participation.workers=v` sets it.
- **Tapes** `tapes/markets-c1p.ron`, `tapes/markets-c2p.ron`: `markets-tape --inst c1p|c2p`'s
  output. C1PN has none committed.
- **Run grammar**: `part.workers*F` (the genesis share times F, at most 1) and `part.workers=V`.
- **Readouts**: the CSV's `part_workers` is the state share (hours offered over N), and a new
  `part_target` the rule's F\* at the tick's prices; `stats.tsv` adds `pace.gap_max` (largest
  |ln(F/F\*)|), `pace.low` (least F/F\*) and `pace.zero_hours` (ticks with no hours offered).

### 5.5 Tests, each failing with its change undone

1. `pace_absent_is_p23s`: with an exit and no pace, the workers decide and produce exactly as
   P2.3's over 3,000 random states; every committed tape's hash test stays green.
2. `paced_share_moves_at_its_rate`: from a state share s₀ and posted prices, the decision offers
   N·(s₀ + a·(F\* − s₀)) hours, sets the state to that share, and posts the rule's plots.
3. `paced_rest_is_the_oracles`: C1P, C2P and every target: three ticks from the oracle's f64
   point leave every observable within 1e-12 in log and the share within 1e-15 of the rule's.
4. `pace_is_checked_at_load`: a share outside [0, 1] or NaN; an adjust of the wrong unit.
5. `paced_genesis_state_is_the_tapes`: the genesis share is `pace.share`; without a pace 0.0.
6. `paced_workers_leave_the_trap`: C1P `p[mach]*0.5` at `rate.*=0.9` converges; C1's diverges.
7. `commons_paced_tapes_are_their_generators_output`.
8. `harness_reads_the_paced_share`: `part_workers` is the state share, `part_target` the rule's.

### 5.6 What must not move

The pins (gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`),
every committed tape's text, `tape_hash`, `world_id` and 2,000-tick hash stream (C1's and C2's
included), and TAPE.md's schema 1. `scripts/gate.sh` and `scripts/gui.sh` green on both machines
before the commit that adds the code.

## 6. The instances

| | C1P | C2P | C1PN |
|---|---|---|---|
| from | C1 (the commons full, Crowded) | C2 (the commons with room, Commons) | C1N (C1 at χ_max 0.25) |
| pace | 1.3 a year (a = 0.024690 a tick at 52; 0.1026 at 12; 0.003555 at 365) | the same | the same |
| genesis share S/N | 0.134615384615 (7/52) | 0.146811906829 | the point's |
| everything else | C1's (SPEC-commons §2.1) | C2's | C1N's |

## 7. The rest point is the oracle's: the proof

Take a live rest point of the paced map: every market clears exactly and trades, every actor
spends its income, nothing spoils, and every state is constant (MARKETS-SPEC §4.1).

1. **The share.** s = s + a·(F\* − s) with a = −expm1(−rate/tpy) ∈ (0, 1) for any finite rate > 0.
   So a·(F\* − s) = 0 and s = F\*: the workers offer N·F\* = the rule's hours at the rest prices.
2. **The plots** are the rule's T_p at the rest prices, in every regime.
3. So every actor's decision at the rest prices is P2.3's with the commons rule, and the commons
   frame's proof (its §3.5, steps 1–7) holds unchanged: the live rest point satisfies unit 1e's
   equations at the instance's parameters, which have one solution at every registered target.
   **The rest point is the oracle's point**: x\*, every price relative to r, every volume, the
   regime and the shadow rent, with the share at S/N.
4. **Conversely** the oracle's point with s = S/N is a rest point of the paced map: F\* there is
   S/N, so the share does not move.
5. **No new rest point.** The pace adds one state whose rest condition is s = F\*; it has no band,
   step or threshold. The corpse (no trade) is excluded by liveness, as before.
6. **In floating point** the share converges to within an ulp of the rule's: mode A ends with
   |s − F\*| ≤ 5.0e-16 at every tick length (§9.1).

R3: s is a convex combination of s₀ and F\*, both in [0, 1] (F\* by the support of χ, s₀ by
induction from the genesis share, checked at load), so no clamp is needed and no price is touched.
R13: the rule reads posted prices (through the participation rule), its own state and its params.
R1: with `pace` absent no new branch runs (§3.1 in the mirror; §5.5 test 1 in the engine).

## 8. The engine run, in order

Registered here before any code; the scorer is committed before its wave (decision 311). The
tolerances are the commons frame's §5.6, with the runaway ticks on the harness's reference (O107,
the commons' A3) and every end value against the oracle's (O108).

- **E0, the trace diff.** `tm.tick` (variant `P1.3r`, the genesis carry) against the engine for
  2,000 ticks, within 1e-12 in log: C1P `hold`, `p[labour]*2`, `JB(2)`, `p[mach]*0.5`,
  `commons=48.6@genesis`, `commons=12.15@genesis`, `part.workers*0.5`; C1P `p[mach]*0.5` at
  `rate.*=0.9` (a registered trap run of C1); C2P `hold`, `p[labour]*2`; C1PN `p[mach]*0.5`. A
  parting blocks scoring until a named rounding explains it.
- **E1, nesting.** Every pin of §5.6 and tests 1 and 5 of §5.5.
- **E2, mode A and the kick sets** at C1P, C2P, C1PN (52 a year; 12 and 365 reported).
- **E3, the verdict battery**: Tiers 1–3 at L, Tier 3 at 10·L, Tier 3S at L and 10·L (the commons
  registration's §3; the engine's names `coin.desk.<d>*F` are the mirror's `coin.<d>*F`, and
  `pred/runs.jsonl` carries both), C1P and C2P.
- **E4, stocks** (41) **and pace** (5: `part.workers*0.1`, `*0.5`, `*2`, `=0`, `=1`).
- **E5, joint2, joint4 and basin. E6, history** (`cycle(land.mach,1500,80)`, `cycle(commons,1500,80)`).
- **E7, Hold and tilt 1. E8, tick length** (Tiers 1–2 at 12 and 365 a year). **E9, enclose, C1PN's
  battery** (now a stress control), **and the controls**: C1 as registered at the 17 dial settings
  (the 18 trap runs of §1.2) and C1P and C2P at every tilt 2 (the trap beyond the edge).
- **E10, the dial neighbourhood**: Tier 3 of C1P and C2P at the 17 settings.

**The verdict** per instance is the commons frame's §5.5 with Tier 3S (decision 399): GO where mode
A passes and every non-vacuous run of Tiers 1–3 and 3S is CONVERGED, with Tiers 3 and 3S CONVERGED
again at 10·L. **With margin** where, in addition, every Tier-3 run of E10 is CONVERGED or VACUOUS
(decision TR4).

**What would refute this spec**, each sending the remedy back to the scan with H as the alternative:
- a class change in E3, E4 or E10 at C1P or C2P;
- any engine run at a registered target that rests off the oracle's point, or a paced share that
  ends more than 1e-12 from the rule's;
- an E0 parting not explained by a named rounding;
- the trap (a runaway) in any C1P or C2P run of E3–E8 or E10, or any tick with no hours offered in
  them;
- none of the controls' predicted trap runs (E9) falling into it.

## 9. Registered predictions

The conditions are §0's; the per-run predictions are `pred/runs.jsonl` (3,901 runs) and
`pred/controls.jsonl` (817), tabulated in `pred/predictions.out` and `.json`.

### 9.1 Mode A (from the oracle's point, L ticks; PASS: every observable within 1e-9 in log)

| | 52 a year | 12 | 365 |
|---|---|---|---|
| C1P | PASS, 2.4e-15 (L 141,000) | PASS, 2.1e-14 (26,000) | PASS, 8.9e-16 (1,113,000) |
| C2P | PASS, 4.4e-16 (142,000) | PASS, 1.9e-14 (26,000) | PASS, 1.6e-15 (1,116,000) |
| C1PN | PASS, 2.2e-16 | PASS, 2.2e-14 | PASS, 3.3e-16 |

Every fill at least 1 − 2e-14; the share within 5.0e-16 of the rule's at the end.

### 9.2 The verdicts

| | predicted | Tier 1 | Tier 2 | Tier 3 | Tier 3 at 10·L | Tier 3S (L; 10·L) | E10: 17 settings × 43 |
|---|---|---|---|---|---|---|---|
| **C1P** | **GO with margin** | 30/30 (27 + 3 slack) | 42/42 (39 + 3) | 43/43 (40 + 3) | 43/43 | 24/24; 24/24 | 731/731 |
| **C2P** | **GO with margin** | 30/30 (27 + 3) | 38/38, 4 VACUOUS | 41/41, 2 VACUOUS | 41/41, 2 VACUOUS | 24/24; 24/24 | 697/697, 34 VACUOUS |

VACUOUS by construction: C2P's commons × 1.1, × 0.9 and × 2, at genesis and dated (O103).

### 9.3 Speeds and paths, tier by tier (non-slack CONVERGED runs)

| | Tier | ticks to tol, median (slowest) | peak D̂ median / worst | dead ticks median / worst | lowest baskets (ticks with none) | least hours / S\* | e₀/w at most |
|---|---|---|---|---|---|---|---|
| C1P | 1 | 319 (444) | 71 / 504 | 0 / 0 | 0.632 (0) | 0.969 | 0.636 |
| C1P | 2 | 369 (545) | 269 / 2,184 | 0 / 83 | 0.131 (0) | 0.555 | 1.004 |
| C1P | 3 | 516 (698) | 1,089 / ∞ | 30 / 150 | 0 (10) | 0.166 | 2.750 |
| C2P | 1 | 326 (506) | 78 / 476 | 0 / 0 | 0.621 (0) | 0.861 | 0.436 |
| C2P | 2 | 387 (545) | 257 / 1,982 | 0 / 81 | 0.161 (0) | 0.570 | 0.678 |
| C2P | 3 | 524 (671) | 1,132 / ∞ | 28 / 150 | 0 (9) | 0.181 | 1.911 |

Every CONVERGED run ends in its target's regime; at a Crowded end the shadow rent is within 8.8e-15
of the oracle's. Tier 3 at 10·L equals Tier 3 run for run (early stop). No tick in any C1P or C2P
run has no hours offered. The provider's transfer falls short in 1/11/30 runs of C1P's Tiers
1/2/3 (at most 5/71/210 ticks; its coin at least 0.098 of genesis) and 1/12/28 of C2P's (13/66/166
ticks; 0.102).

### 9.4 The families

| family | C1P | C2P |
|---|---|---|
| stocks (41) | 41 CONVERGED; median 393 ticks (slowest 567) | 41; 392 (572) |
| pace (5) | 5; median 497 (527) | 5; 505 (558) |
| joint2 (60) | 60; 552 (710) | 60; 565 (666) |
| joint4 (40) | 40; 618 (810) | 40; 634 (870) |
| basin (430) | 430; 541 (856) | 430; 544 (897) |
| history, land.mach | 81/81 windows within tolerance; median 521 (734) | 81/81; 595 (693) |
| history, commons | 81/81; median 354 (491) | 81/81; 0 (303) |
| Hold (115) | 115, run for run the battery's | 109, 6 VACUOUS |
| tilt 1 (115) | 115; Tier 1/2/3 median 263/320/428 | 109, 6 VACUOUS |
| 12 a year, Tiers 1–2 (72) | 72; median 907 (T1) and 960 (T2) ticks, 75.6 and 80.0 years | 68, 4 VACUOUS; 93.1 and 101.3 years |
| 365 a year, Tiers 1–2 (72) | 72; 2,092 and 2,223 ticks, 5.7 and 6.1 years | 68, 4 VACUOUS; 5.8 and 6.6 years |
| enclose (4) | 4; 226 ticks | 4; 209 (222) |
| C1PN's battery (115) | 109 CONVERGED, 6 VACUOUS; Tier 1/2/3 median 411/520/575 | |

### 9.5 The dial neighbourhood (E10) and the controls (E9)

- C1P and C2P: every Tier-3 run CONVERGED at each of the 17 settings, but C2P's `commons=62.4`
  at genesis and dated, VACUOUS by construction at each (34); per-setting medians and slowest in
  `predictions.out` (C1P: medians 389–594 ticks, the slowest run 1,057 ticks at buffer × 0.75).
- **The registered C1 at the 17 settings** (no pace): the 18 trap runs of §1.2, every one DIVERGED
  at the tick given there (harness's reference, within 5%), and the other 713 CONVERGED.
- **C1P at every tilt 2**: `p[mach]*0.5`, `JB(2)` and `N(0.5)` DIVERGED at 308, 308 and 314 (harness's
  reference), with no tick of zero hours; 40 CONVERGED. **C2P at tilt 2**: `p[mach]*0.5` and `N(0.5)`
  DIVERGED at 314 and 305; 39 CONVERGED, 2 VACUOUS.

### 9.6 The largest root per tick (`lin/lin-P1.3r.json`)

| | base | range over the targets | slowest target |
|---|---|---|---|
| C1P | 0.986127 (half-life 50 ticks) | 0.985261–0.990038 | land.mach × 0.5 (Enclosed), 69 ticks |
| C2P | 0.986524 (51) | 0.985569–0.988657 | land.mach × 0.5, 61 ticks |

Every kick bar PL^(0.9·L) ≤ 1e-3 passes.

## 10. Decisions proposed and open items

Numbered here TR1–TR5 and O-TR1–O-TR5; the session numbers them in its ranges (decisions
400–449, open items O110–O129) at registration. Each is open to veto; the alternative named is the
registered one (R6).

- **TR1. O100's remedy is participation at a rate** (§4.1, §5): the workers' optional
  `exit.pace`, the share moving a = −expm1(−1.3/tpy) of its gap to the rule's each tick, only the
  hours lagging, the plots the rule's. *Alternative:* home output sold (H), with its 1e addendum and
  a new instance. *Extreme:* the registered rule.
- **TR2. The paced instances are new: C1P, C2P and C1PN**, with tapes `markets-c1p.ron` and
  `markets-c2p.ron`; C1 and C2 stay as registered, their tapes, wave and GO unchanged (R1).
  *Alternative:* C1 and C2 amended in place, which moves their tapes and ids.
- **TR3. The pace's dial is `adjust.participation.workers` at 1.3 a year in C2m**, for paced
  instances only, scaled by `adjust.*` (§5.4). *Alternative:* the technique's 2.6, which keeps one
  failing setting (rate × 0.75) at C1; or a family of its own outside `adjust.*`.
- **TR4. The engine run is §8's E0–E10, and "with margin" is a verdict line**: every Tier-3 run
  of the 17-setting neighbourhood CONVERGED or VACUOUS. *Alternative:* E10 reported beside the
  verdict, as decision 399 (4) registers the family for the roles as they are.
- **TR5. The registered C1's dial-neighbourhood trap runs are the controls** (E9), with C1P and
  C2P at tilt 2; C1PN is kept as a stress control. *Alternative:* C1PN as the negative control, now
  predicted to converge.

- **O-TR1. The trap's attractor remains.** Beyond the stable region (tilt 2; at 2.6 a year, rate
  × 0.75) runs still collapse, slowly: hours decay geometrically and never recover. The pace moves
  the boundary; it does not remove the second state.
- **O-TR2. The machine's own input makes a zero stock absorbing** whenever a tick passes with no
  labour for it (§1.1 step 5), independent of the exit. The pace removes the zero-hour ticks, not
  the absorbing zero; a machine instance with other ways to zero labour (a strike, a zero-wage
  shock) would meet it. O21's cousin.
- **O-TR3. Home output sold** (1e's open question 6) stays unanswered as an oracle addendum; the
  scan has its f64 equilibrium only (`tm.oracle_at`), unchecked at high precision, and on it two of
  C1's targets cross care's edge.
- **O-TR4. The pace's rate has no data behind it.** 1.3 a year (half the gap in 27.7 weeks) was
  chosen for margin in the mirror; the 1750-like instance's participation rate is a calibration
  question (R7).
- **O-TR5. A storable exit good as GOODS-CHAIN E1's seller is locally unstable at C1** (§3.4): O51
  needs a stock rule that damps before any storable running good is registered.

## 11. What this scan did not do

- No engine code, tape or test; no engine run. Every number is the mirror's or the oracle's.
- No oracle addendum: the chosen rule needs none. H's equilibrium was solved in f64 only.
- IW1 was not run (it has no exit). The 1750-like instance was not considered.
- ρ > 0, a government, several plot-taking types and the Split regime at a target were not
  exercised; none of the 26 targets is at the split.
- S was stopped once its roots were known; its scan tables are partial and labelled so.

## 12. Files

`D:/rustyecon-p24/scan-trap/`; sha256 of each in `SHA256SUMS`.
- `SPEC.md` (this file).
- `frame/`: the commons frame's `model/` and `solve/`, copied unedited (their sha256 match the
  frame's registered `SHA256SUMS`, also copied).
- `model/tm.py` (the candidates' mirror), `tb.py` (the runner), `ti.py`, `variants.py`;
  `tm_nest.py` → `tm_nest.out`; `restcheck.py` → `restcheck.out`; `scanrun.py`, `edges.py`,
  `tally.py`, `compare.py`, `edgetab.py`, `tiers.py`, `former_trap.py`, `zero_tick.py`,
  `trace_p.py`; `lin_t.py`, `lin_one.py`, `elasticity_t.py`; `heldout.py`; `predict.py`,
  `predict_ctl.py`, `count_jobs.py`, `modea.py`, `pred_summary.py`, `ctl_tab.py`; `p21/` (P2.1's
  scripts, as the frame copied them) and `elasticity.json` (the frame's).
- `scan/`: `trace0.py`, `trace1.py` → `trace1.out`, `dials.py` (the mechanism, §1.1);
  `former_trap.out`, `zero_tick.out`, `tiers.out`, `heldout.out`.
- `runs/`: `base.jsonl` (the registered rule), `cand-*.jsonl` (each candidate), `edges.jsonl`,
  `held.jsonl`; `scan.out`, `edges.out`, `held.out`; the drivers `go_*.sh` and their logs.
- `lin/`: `lin-*.json` and `.out` (the roots), `elasticity-P1.3r.json`.
- `pred/`: `runs.jsonl`, `controls.jsonl`, `modea.json`, `modea.out`, `predictions.out`,
  `predictions.json`, `predict.log`.
