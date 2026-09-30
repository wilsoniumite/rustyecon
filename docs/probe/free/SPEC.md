# FREE-SPEC: a zero price markets can hold (O96, O101), the free step

Dated 2026-09-30. Work label `scan-free`, Phase 2 proper's second session (P2.4, PHASE2-S1 §6
item D), on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24` at `a483ed0`; nothing in the
worktree was changed). Scratch: `D:/rustyecon-p24/scan-free/`. Every number below is the free
mirror's (`model/fm.py`) at C2m, 52 ticks a year, ρ 0, tolerance 1e-3 in log, unless its line
says otherwise, or the oracle's where the line says so. The evidence is in the files of §13,
whose sha256 are in `SHA256SUMS` beside this file.

It does five things, in this order:

1. names the problem and the candidates for a zero price (§1–§2);
2. builds two instances that need one, checked by three solves and a scan 16 times finer (§3);
3. scans every candidate on both instances' full battery, the dial-neighbourhood family and
   their local roots (§4), and chooses (§5);
4. states the rule as the engine should build it, and proves its rest point (§6–§8);
5. registers predictions for the engine's battery (§9–§10).

## 0. The verdict in brief

**GO for the free step at c = 0.5, on both instances.** A market may carry an optional free
step. Its price moves as p' = p·e^(kx) + (c·p_ref)·(e^(kx) − 1), with p_ref the posted price of a
reference good (labour) at the same node. A step that would take the price to 0 or below posts 0:
the good is free. At 0 the price stays while supply is at least demand, and leaves 0 by itself
when demand exceeds supply. No state is added. Absent, or at c = 0, the step is Saturate's, bit
for bit. It moves no equilibrium, so no oracle addendum is needed.

| candidate (§2) | IL1, idle land at r = 0 (Tiers 1–3) | CT2, two types on one commons (Tiers 1–3) | the dial family (17 × Tier 3 on each) |
|---|---|---|---|
| msat: Saturate (today) | 1 of 100 CONVERGED; the rest run away at ticks 801–3,189 | 2 of 115; the rest run away at 1,034–9,853 or orbit | 34 of 1,377 CONVERGED |
| zo: free only when one-sided (the brief's example, read literally) | msat, run for run | msat, run for run | msat's, run for run |
| zs3, zs2: the snap at φ = 0.001, 0.01 of the wage | 94/94; 92/94 | 109/111 (2 STUCK); 109/111 | 36 and 68 not CONVERGED |
| zp001, zp01: the free step at c = 0.01, 0.1 | 94/94 each | 109/111 (2 ORBITING); 111/111, slowest 57,264 ticks | 34 and 2 not CONVERGED |
| zp03: c = 0.3 | 94/94 | 111/111, slowest 15,545 | 1,309/1,309, slowest 30,140 |
| **zp05: c = 0.5** | **94/94**, slowest 678 | **111/111**, slowest 5,080 | **1,309/1,309** (+68 VACUOUS), slowest 18,365 |
| zp1: c = 1 | 94/94 | 111/111 | 1,309/1,309, slowest 9,275 |
| zp15, zp2: c = 1.5, 2 | 94/94 | 111/111 | 2 and 9 not CONVERGED (IL1's scarce target: at tilt 1 for c 1.5; at rate × 1.25 or tilt ≥ 0.25 for c 2) |
| zp3, zp4: c = 3, 4 | 92/94 (ORBITING at c 3, DEAD at c 4) | 111/111 | not run |

Counts are non-vacuous runs (VACUOUS by construction: §3.4). Every CONVERGED run of a free step
ends with its market at the oracle's price: 0 where the oracle's is 0, positive where it is
positive (§4.3).

- **Idle enclosed land at r = 0 (O96).** IL1 is I1's economy on unit 1e's idle stretch: few
  workers, one priced type in food on plots taken free on idle enclosed land (`ExitLand::Idle`,
  decisions 161 and 381), 37% of the land idle. It is funded under a stated transfer (§3.2):
  the provider pays what rent brings in, which is 0 at r = 0, and the workers' support is their
  own household's. Mode A is exact (D̂ 2.2e-13, the land free on every tick).
- **A commons shared by two types (O101).** CT2 is I1's economy with two plot-taking types that
  share one commons with room. The commons is traded on a market that both pops offer and bid on,
  each reading only posted prices and its own params (R13). With the free step it rests at 0
  where the commons has room, at the shadow rent where it is crowded, and at or above r where the
  plots spill onto enclosed land.
- **The window for c.** On both instances' batteries every run converges for 0.3 ≤ c ≤ 2, and on
  both dial families for 0.3 ≤ c ≤ 1. Below, the commons' mode at a small positive rent is too
  slow (at c 0.1 the kick bar fails at CT2's b.food × 2, and one dial orbits). Above, reopening
  from 0 overshoots IL1's scarce rent and the land market settles on a 0 ↔ positive cycle. c 0.5
  sits inside by a factor of 1.7 below and 2 above.
- **What it does not do.** It does not make the one-type commons faster than decision 398's
  rule (a commons market is 2.5–3.6 times slower at C1, §4.4), so the rule stays there. It does
  not test decision 160 (a land-alone exit good, FG8 in §11). The Enclosed regime with several pops is
  exact only where every pop spills past its share (§6.3).

## 1. The problem

A market whose equilibrium price is exactly 0 cannot post it today. Prices move by
p' = p·e^(kx) (`markets::next_price`), so with supply above demand a price falls by a constant
factor a tick and never reaches 0. The harness's runaway bound (1e-6 of genesis) fails first,
then the engine's positive-range check (`update_prices`, `PriceError::NonFinite`). The real side
converges meanwhile. This is decision 398's amended reading: a limit of what a market can post.

Two places in unit 1e have a zero price with the good in use:
- **idle enclosed land** on the idle stretch (unit-1e.md §2.8, §4.6): r = 0, T_idle > 0, every
  contestable task automated (x\* = 1), prices per unit of the pool's wage;
- **a commons with room** (§4.4, `Commons`): the plot rent r_o = 0. For one plot-taking type
  decision 398's rule gives out the plots with no price; several types would need each other's
  plot demand, which R13 forbids (O101). So several types need the commons as a market, and a
  market at r_o = 0 needs a zero price.

Both are one problem (decision 398 as amended; O96 amended at P2.3.16): a zero price markets can
hold.

## 2. The candidates

Each reads posted prices, its own state and params only (R13), and floors, caps or smooths no
positive price (R3). A step on an agent's offer would be allowed (R3), but none rests at 0
(the commons frame's §1.5 and scan; §2.2 below).

### 2.1 Scanned

- **msat**, Saturate: no rule, the engine today. The control.
- **zo, the one-sided free state** (the brief's example, read literally): a market with supply and
  no bid posts 0; a free market reopens at φ·p_ref when a bid arrives. Where the good is in use
  (idle land, a commons with room) there is always a bid, so it never goes free: it is msat.
- **zs, the snap**: p' = p·e^(kx); a falling p' below φ·p_ref posts 0; at 0 the market stays free
  while x ≤ 0 and reopens at φ·p_ref when x > 0. φ at 0.001 and 0.01 of the wage.
- **zp, the free step**: p' = p·e^(kx) + (c·p_ref)·expm1(kx), and a p' at or below 0 posts 0. At 0,
  x ≤ 0 keeps it at 0 and x > 0 lifts it to c·p_ref·expm1(kx). In u = p + c·p_ref it is Saturate's
  step while p_ref stands still. c at 0.01, 0.1, 0.3, 0.5, 1, 1.5, 2, 3 and 4.

The reference is the labour market's posted price at the same node: the pool's wage, the
numeraire of the idle stretch (decisions 153, 376) and positive in every regime. A reference that
can itself be free (land, for the commons) would leave both at geometric decay when both are 0.

### 2.2 Argued, not run

- **Hold**: both markets always have bids, so it never acts (the commons frame's mhold was msat
  run for run).
- **The owner's reservation** (a step on the provider's offer, as L0's maker's): on a withheld
  tick no desk gets land, production stops, and the rent orbits a positive level (the commons
  frame §1.5; its mres at the commons orbited at ψ·r).
- **A floor order** (the provider bids for its own idle land): it needs the desks' demand to
  size, which R13 forbids, and at the commons it left a continuum of rest points (mfloor, 72 of
  115 STUCK).

## 3. The instances

### 3.1 IL1 and CT2

Both are the markets probe's I1 (MARKETS-SPEC §1.2: C3's four categories, 1a's machine a 0.3,
λ 0.05, b 0.4, θ 1; γ = 0.2 + 0.8x; ρ 0; flow; T 520 a year) with priced worker types in food
(decision 151), support one basket (ν 1) and I1's scalars unless listed:

| param | IL1 | CT2 |
|---|---|---|
| worker types | one: N 10.4 a year (0.2 a tick), χ_max 2 | two, `wa` and `wb`: N 104 a year each (2 a tick), χ_max 1 |
| exit (s₀, s̲, h) in food | (6, 0, 2.7); h/s₀ 0.45 | wa (0.2, 0, 0.09), C2's; wb (0.25, 0, 0.1125); h/s₀ 0.45 |
| commons T_o | none | 19.5 a year (0.375 a tick), each pop holding half |
| free-able market | land, reference labour, c = 0.5 | the commons, reference labour, c = 0.5 |

**IL1** sits on the idle stretch. At r = 0 every machine cost is labour alone, so x\* = 1
(unit-1e.md §2.8), and the land in use is set by labour: T_m = S·B^q/L^q. Few workers make the
land idle. χ_max 2 and s₀ 6 put participation inside F's support (0.770) with the exit worth 0.18
of the wage at rest (0.53 at C1, 0.35 at C2: IL1 sits further from the subsistence trap, by 399's
hypothesis). Its exiters take plots free on idle enclosed land.

**CT2** splits C2's economy into two plot-taking types with different plots. The commons is sized
so the base has room (the plots use 0.350 of 0.375, 6.7% idle), the commons × 0.9 is crowded, and
the commons × 0.5 spills onto enclosed land with both pops beyond their shares.

**How they were chosen** (a forking path, disclosed). `model/search_il.py` looked at N 10.4 and
15.6 a year, χ_max 1, 2, 3 and s₀ 3, 6, 10 (h/s₀ 0.45) on the f64 oracle; at χ_max 1 participation
saturates (F = 1) and no plot is taken, so χ_max 2 and s₀ 6 were taken, the first pair with
interior participation, plots on idle land and every target but inst.land × 0.5 idle.
`search_ct.py` tried wb at C1's exit (0.3, 0.135) with the commons at 24–36 a year (the commons
× 0.5 went straight to Enclosed); `search_ct2.py` tried wb at (0.3, 0.135), (0.25, 0.1125) and
(0.22, 0.099) with the commons at 0.95, 1.1 and 1.25 of the room edge G(0). wb (0.25, 0.1125) was
taken, and the commons set at 19.5 a year, a little below 1.1·G(0) (20.02), so that the commons
× 0.9 lies inside the Crowded band (17.09–18.20) and the commons × 0.5 spills with both pops beyond
their shares. No displaced-start run was used to choose any number of either instance. c was chosen
after the scan (§5): its first pick was c 1, and the dial families at c 1.5 and 2, run to map the
window's upper edge, moved it to c 0.5.

### 3.2 IL1's funding, a stated transfer

Under the probe's transfer the provider pays N·P_s from rent, and at r = 0 it has none: the
oracle's `funded` is false and its provider baskets are −ν·N = −0.2 (the commons frame's §1.1).
IL1 is registered under the transfer as the roles already build it: the provider pays
min(N·P_s, coin), so it pays what rent brings in, which is 0 at r = 0. Its coin is 0 at genesis
and at rest. The workers' participation reads the support ν·P_s as before: support "through
someone else's household" (unit-1e.md §2.4, main.tex:369), here the pop's own pooled wage. The
market allocation is the oracle's: total spending is w·S = Y·P_s, SSRN App. C's income identity
with r·T_m = 0, so every market clears at the oracle's quantities (§7). Only the split of baskets
between the two households differs from the oracle's accounts, as O102's does at C1's Enclosed
targets. At IL1's one scarce target (inst.land × 0.5, r/w 0.0254) the provider is funded
(baskets 1.154) and pays in full. So the brief's "impossible at ρ 0" is met by naming the
transfer, not by changing it; a 1f transfer would move the support's payer, not the point.

### 3.3 The oracle's answer, three ways

Per tick at 52 a year. `solve/fsolve.py` is written from unit-1e.md §4 (several types) and reads
no code of `crates/oracle` and no mirror (50 digits, `points_*_coarse.json`). `check/` solves
every target with the worktree's `ParcelEconomy` (`check.out`). The mirror's f64 oracle is
`model/ofree.py`. `check/compare.out`: the crate and the mirror each agree with the 50 digits
within 6.5e-16 relative on x\*, r/w, P_s/w, Y, S, T_m, T_p and p_food/w at all 26 targets, and on
r_o within 4.8e-13 (CT2's b.food × 2, where r_o/w is 0.0218: the cancellation p_g·s₀ − e over h).
The regimes agree at all 26.

| | IL1 | CT2 |
|---|---|---|
| where, land, exit land | idle stretch, `Idle`, `Idle` | the line, `Scarce`, `Commons` |
| x\* | 1 | 0.7474168388728093814 |
| r/w | 0 | 2.0681652993467759682 (v 0.48352034545587) |
| P_s/w | 0.044558571428571428571 | 3.7121874592087070297 |
| p_food/w, p_mach/w | 0.0298928571428571, 0.0714285714285714 | 1.765370, 1.253238 |
| hours S (participation) | 0.15400799499335622941 (0.770040) | 0.54773479377871321783 (wa 0.147674, wb 0.126193) |
| Y | 3.4563045909188342978 | 5.7188350589848032297 |
| T_m, T_p, T_idle | 6.14001647905, 0.124178413518, 3.73580510743 | 10, 0, 0 |
| commons used / T_o | none | 0.350025 / 0.375 (6.7% idle) |
| provider baskets, `funded` | −0.2, false (§3.2) | 1.571, true |

Every target (`solve/points_fine.json`, `check/check.out`):

| target | IL1 | CT2 |
|---|---|---|
| land.mach × 1.1, × 0.9, × 2, × 0.5 | Idle; only T_m moves (6.263, 6.017, 7.372, 5.524) | Commons; x\* 0.74080, 0.77445, 0.69105, 0.98099 |
| b.food × 1.1, × 0.9, × 2, × 0.5 | Idle; T_m 6.347, 5.933, 8.214, 5.103 | Commons, Commons, **Crowded** (r_o/r 0.011781), Commons |
| inst.land × 1.1, × 0.9, × 2 (IL1); the commons × 1.1, × 2 (CT2) | Idle, the base's point (VACUOUS by construction) | Commons, the base's point (VACUOUS by construction) |
| inst.land × 0.5 (IL1); the commons × 0.9 (CT2) | **the wall, Scarce**, r/w 0.0253659093919833, S 0.119995 | **Crowded**, r_o/r 0.641199 |
| the commons × 0.5 (CT2) | | **Enclosed**, T_p 0.14287; each pop's plots 1.51 and 1.92 of its share |

**Uniqueness (decision 70, 362).** Neither instance is certified by 1e's Proposition 5 (IL1's
point is not a crowded commons; CT2 has two plot-taking types), and the crate reports
`certified` false at all 26. So decision 378 applies: each target was scanned 16 times finer than
`EXIT_SCAN`. `solve/fine.out`: at every one of the 26 targets the sign sequence over the corner
(64 points), the line (65,537), the wall (161, to 2^40 of v(1)) and the idle stretch (65) changes
once, the line is monotone, and the root equals the coarse scan's to 25 digits.

### 3.4 The cost targets

| coefficient | base | × 1.1, × 0.9, × 2, × 0.5 |
|---|---|---|
| `land.mach` | 0.4 | 0.44, 0.36, 0.8, 0.2 |
| `b.food` | 0.6 | 0.66, 0.54, 1.2, 0.3 |
| IL1: `inst.land` (T a year) | 520 | 572, 468, 1040, 260 |
| CT2: the commons (a year, split equally) | 19.5 | 21.45, 17.55, 39, 9.75 |

Each at genesis and dated at L/4. At IL1 land.mach and b.food move only the land in use (at r = 0
land has no price to move), and inst.land × 0.5 makes land scarce: the land market must reopen
from 0 to r/w 0.0254. At CT2 the base is free and the commons × 0.9 and b.food × 2 make it crowded:
the commons must reopen to 0.641·r and to 0.0118·r.

## 4. The scan

`model/run_free.py` on `model/fm.py`, runs in `runs/scan{A..G}.jsonl` and `runs/reg{Z,H}.jsonl`,
tables in `runs/table_all.out`, `table_E.out`, `table_FG.out`; local roots `model/lin_free.py` →
`runs/lin_c2m.out`, `lin_all.out`, `lin_all05.out`, `lin_dial05.out`, and
`model/lin_enclosed.py` → `runs/lin_enclosed.out`. 21,693 runs in all.

### 4.1 The mirror, and what must not move

`fm.py` is the commons frame's `cm.py` (sha256 `4d28d20b…dd86`, copied unedited beside it) with the
scan's additions, each marked. `nest_free.out`: with no free-able market and one pop, `fm.tick` is
`cm.tick` in every state value on every tick, 3,000 ticks from genesis, w × 2 and w × 0.5, at I0,
I1, C1, C2 and the commons frame's msat at C1 and C2 (18 checks). The free step with c = 0 is
Saturate bit for bit on I1's land and C1's and C2's commons markets (9 checks). Several pops on a
commons market, with one pop, are the commons frame's msat bit for bit (6 checks).

### 4.2 The battery, the dial family and mode A

Tiers 1–3 as MARKETS-SPEC §7.7, IL1 with the wage-unit grammar of §6.4 (100 runs), CT2 with the
commons frame's (115). The dial family is Tier 3 at every price rate (the commons' included),
every desk's buffer (cash turnover) and every adjust at × 0.75, × 0.9, × 1.1, × 1.25, and every
tilt at 0.05, 0.1, 0.25, 0.5 and 1: 17 settings, 646 runs at IL1 and 731 at CT2.

| | mode A | Tier 1 | Tier 2 | Tier 3 | dial family |
|---|---|---|---|---|---|
| IL1 msat (from r 0.005·w) | DIVERGED at 1,463 | 0/25 | 0/37 | 1/38 | 17/646 |
| IL1 zs3 | exact | 25/25 | 33/33 (+4 V) | 36/36 (+2 V), slowest 5,573 | 610/612 (+34 V): inst.land × 0.5 ORBITING at tilt 1 |
| IL1 zs2 | exact | 25/25 | 33/33 | 34/36: 2 ORBITING | 578/612: 30 ORBITING, 4 DEAD |
| IL1 zp001 | exact | 25/25 | 33/33 | 36/36, slowest 3,379 | 612/612, slowest 4,499 |
| IL1 zp01 | exact | 25/25 | 33/33 | 36/36, slowest 1,198 | 612/612, slowest 1,617 |
| IL1 zp03 | exact | 25/25 | 33/33 | 36/36, slowest 771 | 612/612, slowest 1,051 |
| **IL1 zp05** | **exact, D̂ 2.2e-13** | **25/25, 281 (474)** | **33/33, 334 (570)** | **36/36, 516 (678)** | **612/612, 507 (917)** |
| IL1 zp1 | exact | 25/25 | 33/33 | 36/36, 516 (678) | 612/612, 502 (915) |
| IL1 zp15 | | | | | 610/612: inst.land × 0.5 DEAD at tilt 1 |
| IL1 zp2 | exact | 25/25 | 33/33 | 36/36 | 603/612: inst.land × 0.5 DEAD or ORBITING at rate × 1.25 and tilt ≥ 0.25; JB(2) DIVERGED at tilt 1 |
| IL1 zp3, zp4 | exact | 25/25 | 33/33 | 34/36: inst.land × 0.5 ORBITING (c 3), DEAD (c 4) | not run |
| CT2 msat (from r_o 0.5·r) | DIVERGED at 8,103 | 0/30 | 1/42 | 1/43 | 17/731 |
| CT2 zs3 | exact | 30/30 | 40/40 (+2 V), slowest 13,538 | 39/41: b.food × 2 STUCK | 663/697: 32 STUCK, 2 ORBITING |
| CT2 zs2 | exact | 30/30 | 40/40 | 39/41: b.food × 2 ORBITING | 663/697: 34 ORBITING |
| CT2 zp001 | exact | 30/30 | 40/40 | 39/41: b.food × 2 ORBITING | 663/697: 34 ORBITING |
| CT2 zp01 | exact | 30/30 | 40/40, slowest 7,894 | 41/41, slowest 57,264 | 695/697: b.food × 2 ORBITING at rate × 0.75 |
| CT2 zp03 | exact | 30/30 | 40/40, slowest 6,056 | 41/41, slowest 15,545 | 697/697, slowest 30,140 |
| **CT2 zp05** | **exact, D̂ 8.9e-13** | **30/30, 306 (498)** | **40/40, 354 (5,080)** | **41/41, 512 (3,357)** | **697/697 (+34 V), 483 (18,365)** |
| CT2 zp1 | exact | 30/30 | 40/40, 354 (3,708) | 41/41, 512 (5,405) | 697/697, 481 (9,275) |
| CT2 zp15, zp2 | exact | 30/30 | 40/40 | 41/41 | 697/697 each |
| CT2 zp3, zp4 | exact | 30/30 | 40/40 | 41/41, slowest 3,907, 3,248 | not run |

Ticks to tolerance: median (slowest) of the CONVERGED runs. V: VACUOUS by construction (IL1's
inst.land × 1.1, × 0.9, × 2 and CT2's commons × 1.1, × 2, which do not move the point; §3.4). A
blank cell was not run. zo is msat run for run on both instances (`scanB`, `scanD`: every class
and tick the same), once its positive price is held to Saturate's bound as msat's is. `scanA` ran
zo with a free-able market's bound [0, 1e6·scale] and so read its runs as converging while its
price fell for ever; those rows are superseded and left out of every table.

What each failure is:
- **msat.** The free-able market's supply exceeds its demand at every positive price, so its price
  falls by a constant factor a tick: at IL1 it crosses the bound at ticks 801–3,189, at CT2 at
  1,034–9,853. The real side is within 1.3e-5 (IL1) and 2.4e-7 (CT2) in log by then. Its 17
  converging dial runs are IL1's scarce and CT2's crowded targets, where the price is positive.
- **zs.** Exact where the price is 0 or above φ·w. For an equilibrium price in (0, φ·w) it has no
  rest point (§7's step 1 does not hold for it). At IL1's scarce target it reopens at φ·w, far
  below r/w 0.0254, and at tilt 1 the snap chatters (55,933 switches in 56,000 ticks). At CT2's
  b.food × 2 (r_o 0.0218·w) the commons' step is Saturate's and so slow that the run is STUCK at
  0.0023·w after 141,000 ticks (§4.4's roots).
- **zp at small c.** The same slowness at CT2's b.food × 2: the market's log-step speed is about
  k·(1 + c·p_ref/p)·|ε|, and the plots' elasticity ε is small at a small rent. ORBITING at c 0.01;
  at c 0.1 converged in the battery (57,264 ticks) but not at rate × 0.75.
- **zp at large c.** Reopening from 0 at IL1's scarce target lifts land to about c·p_ref·(e^(kx) − 1)
  in one tick. Where that overshoots r\* = 0.0254·w, the next tick's excess supply takes the price
  back below 0, and the market settles on a 0 ↔ positive two-cycle: at c 1.5 with tilt 1, at c 2
  with rate × 1.25 or tilt ≥ 0.25, at c 3 and 4 at C2m (DEAD at c 4: the land market alternates
  every tick). Its local root at the point is fine (0.98774 at c 4): the cycle lives on the kink.

### 4.3 The free-able market's end state

Every CONVERGED run of a free step ends with its market at the oracle's price, in every set:
0 where the target's is 0 (IL1's idle targets, CT2's Commons targets) and positive where it is
positive (IL1's inst.land × 0.5, CT2's Crowded and Enclosed targets). At zp05: 706 of 706 at IL1
and 808 of 808 at CT2 (`table_FG.out`). From a positive land price IL1's market is free within
1–4 ticks at c 0.5 (1–2 at c 1, 2–16 at c 0.1). The most free/priced switches in a zp05 battery run
are 6 (IL1's JB(2)) and 2 (CT2). At CT2's Enclosed target the commons ends 0.13–0.81% above r in
the battery and the dial family: inside the continuum of rest points r_o ≥ r (§6.3).

### 4.4 Local roots (PL)

P2.1's measure on the free mirror (`lin_free.py`): 8 directions, 40,000 ticks after 10,000. A
free-able market at 0 is held at 0: a small perturbation leaves its supply above demand, so its
price stays 0 and its root is 0. At a free point every candidate has the economy's roots with the
good free.

| point | msat | zs3 | zp01 | **zp05** | zp1 | zp4 |
|---|---|---|---|---|---|---|
| IL1 base (and every idle target) | no rest point | 0.988687 | 0.988687 | **0.988687** (half-life 61) | 0.988687 | 0.988687 |
| IL1 inst.land × 0.5 (the wall, land priced) | 0.998138 | 0.998138 | 0.994122 | **0.989993** (69) | 0.988833 | 0.987735 |
| CT2 base (and every Commons target) | no rest point | 0.985886 | 0.985886 | **0.985886** (49) | 0.985886 | 0.985886 |
| CT2 commons × 0.9 (Crowded, r_o 0.64·r) | 0.999051 | 0.999051 | 0.998981 | **0.998697** (532) | 0.998342 | 0.996196 |
| CT2 b.food × 2 (Crowded, r_o 0.0118·r) | 0.999994 | 0.999994 | 0.999964 | **0.999846** (4,493) | 0.999698 | 0.998801 |
| CT2 commons × 0.5 (Enclosed) | 0.987107 | 0.987107 | 0.987107 | **0.987107** (53) | 0.987107 | 0.987107 |

**The Enclosed point has a switch.** There every pop fills its own share at r_o ≥ r, and the
commons' price above r moves nothing real: a continuum of rest points, one neutral direction,
held in the measure. At r_o = r exactly a perturbation that lifts land's price above the
commons' switches every pop to bidding all its plots on the commons. `lin_free` holds the commons
at r exactly, so at tilt 0.5 and 1 its estimate crosses the switch and reads 1.057 and 1.072.
Held at 1.05·r, inside the continuum, the point's roots are 0.9767–0.9903 at all 17 dials
(`lin_enclosed.out`), and every Enclosed run of the dial family converges.

The kick bar (MARKETS-SPEC §7.5: PL^(0.9L) ≤ 1e-3) needs PL ≤ 0.9999456 at CT2 (L 141,000) and
≤ 0.999863 at IL1 (L 56,000). zp05 passes at all 26 targets (`lin_all05.out`; the tightest
e^-19.6, CT2's b.food × 2), and across the 17 dials at both bases and the four priced targets (`lin_dial05.out`);
zp01, zs3 and msat fail at CT2's b.food × 2. 1 − PL there is proportional to r\*_o + c·w within 1%,
so the bar needs c ≥ 0.17 at C2m.

**A commons market for one type (the commons frame's O101 note).** With the free step (c 0.1 and
c 1), C1's and C2's commons as a market converge 115/115 and 109/109 (`scanB`): the 8 of C1's runs
that msat lost to the runaway bound now post 0 within 50–464 ticks and converge. But at C1 it is
slower than decision 398's rule: Tier 2 and 3 medians 1,347 and 1,267 ticks at c 1 against the
rule's 372 and 508. So one type keeps the rule; the market serves several.

### 4.5 The window for c

| c | 0.01 | 0.1 | 0.3 | 0.5 | 1 | 1.5 | 2 | 3 | 4 |
|---|---|---|---|---|---|---|---|---|---|
| batteries, runs not CONVERGED (of 205) | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 2 |
| dial families, runs not CONVERGED (of 1,309) | 34 | 2 | 0 | 0 | 0 | 2 | 9 | | |
| CT2 b.food × 2 kick bar | fails | fails (PL 0.999964) | passes | passes (0.999846) | passes (0.999698) | | | | passes (0.998801) |

## 5. The choice

**The free step at c = 0.5 (zp05), with labour's price as the reference.** It is exact at every
rest point (§7), adds no state, and is Saturate's step bit for bit when absent or at c 0. It is one
of the three scanned scales (0.3, 0.5, 1) whose every run of both batteries and both dial families
converges with its market at the oracle's price, and whose every kick bar passes. It makes the
commons' slow mode at a small rent 24 times faster than Saturate's (half-life 4,493 ticks against
108,510), and it leaves idle land's price at exactly 0, a root of 0.

**Why 0.5 in the window [0.3, 1].** The window's edges lie in (0.1, 0.3] below (the slow mode at a
small rent) and in (1, 1.5] above (the reopening overshoot). 0.5 is inside by at least 1.7 below
and 2 above, and the failure above is the worse one (a dead market, not a slow one), so the
choice leans low. It passes the kick bar with a factor of 3 on c. c 1, the scan's first pick, is
also inside but at most 1.5 from the upper edge. c is a tape param, so a later instance registers
its own after its own scan: the upper edge moves with the smallest positive rent the market must
reopen to (here r\*/w 0.0254 against a reopening step of c·(e^k − 1) ≈ 0.0126 at c 0.5).

**The named alternative (R6): the snap, zs at φ 0.001 of the wage.** It is also exact at 0 and
nests the same way, and it keeps Saturate's step at every price above φ·w, closer to today's
engine. It is not chosen: it has no rest point for an equilibrium price in (0, φ·w), it reopens a
scarce market at φ·w and climbs from there at Saturate's pace, it is STUCK at CT2's b.food × 2 in 32
of 34 dial runs, and it chatters at IL1's scarce target at tilt 1.

## 6. The rule as the engine builds it

### 6.1 In `markets` (the free step)

A good may carry an optional free step on the tape:

```rust
/// The free step (FREE-SPEC §6.1): the good's price may be 0. Absent: Saturate's or Hold's step,
/// bit for bit, and a positive price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFreeStep {
    /// Good key: the reference, a traded good with a market at every node this good trades at.
    pub reference: Key,
    /// Param key, unit `Dimensionless`, live: c, the step's scale in units of the reference.
    pub scale: Key,
}
// in RawGood, the one optional field (as `untraded`):
#[serde(default, skip_serializing_if = "Option::is_none")]
pub free: Option<RawFreeStep>,
```

It resolves to `GoodDef::free: Option<FreeStep { reference: GoodId, scale: Site }>`, the site a
`value` read, **left out of the resolved world's serialization when absent** (as the maker's
`reserve` since L0.5), so every committed tape keeps its canonical text, `tape_hash` and
`world_id`.

**The step** (`markets::prices`), for a good with `free`, from the posted price p, the per-tick
log step k, the volumes S and D, the reference's posted price p_ref at the same node (read from
the book at phase start, like p) and c:

```
one-sided (exactly one of S, D is 0) and Hold:  p' = p                    (0 stays 0)
else:  x   = imbalance(S, D)                                              (as now)
       e   = num::exp(k·x);  em1 = num::expm1(k·x)
       q   = p·e + (c·p_ref)·em1        each product rounded once, then the sum
       p'  = q if q > 0.0, else +0.0    (the good is free)
```

- `update_prices`: a free-able market may post +0.0; every other keeps today's positive-range
  check. The EMA of a free-able market may reach +0.0 likewise; every other keeps its check.
  `SetPrice` is emitted when the bits change, as now.
- **Admission at a price of 0** (`order::admit_actor`): a buy at a posted price of 0 is feasible
  in full, whatever its budget, 0 included: a free good is taken for nothing. Written as a branch
  before `num::max_qty` (whose zero budget gives 0, "nothing is bought with nothing"). It is
  reachable only at a free-able market, since every other price is positive.
- Settlement pays price·qty = 0.0 exactly; nothing divides by a price.
- **Genesis**: a free-able good may take a genesis price of 0; every other keeps `price > 0`.
- **Load checks** (each a `LoadError` with its path): the reference is a traded good with a
  market, not the good itself, not a currency, not untraded; the scale is a `Dimensionless`
  param, finite and > 0; `free` only under `PriceRule::Imbalance`.
- The GUI's market inspector explains a free step with the same function (`next_price`'s free
  variant), so the waterfall shows the shift c·p_ref·expm1(kx).

### 6.2 In `agents`: nothing for idle land

Every role already prices its inputs at posted prices and budgets p·q, so at a free market its
budget is 0 and admission fills it (§6.1). The workers' plot buy on idle land posts T_p with
budget r·T_p = 0 (decision 398's `exit`, unchanged; at r = 0 its regime is `Enclosed` in the
rule's code and `Idle` in the harness, decision 381). No role divides by the land price
(audited: `many/rules.rs` divides by the basket's price, a cost and a machine price, each
positive at every tick of both instances). The provider's transfer is P2.1's, min(N·P_s, coin).

### 6.3 In `agents`: several pops on a commons market (O101)

The workers' `exit` block gains one optional field, `market: Key`, a traded `Instant` good (the
commons), free-able, not in the pop's basket. With it:

- the pop holds T_o,i (its `commons` param, its share) as an endowment each tick and offers it
  on that market, as the provider offers land;
- it reads the posted plot rent r_o and decides by `fm.pop_decide` (the commons frame's
  `commons_market_decide` for one pop): r̂ = r where a plot pays at r (r·h < p_g·Δ), else p_g·Δ/h.
  While r_o < r̂ its plots pay r_o: hours n(p_g·s₀ − r_o·h) by `num::fma`, and it bids
  h·(N − hours) for the commons. At r_o ≥ r̂ its plots pay r̂: it bids min(G, T_o,i) for the
  commons (its own share first) and, where a plot pays at r and G > T_o,i, rents G − T_o,i of
  enclosed land;
- the commons bid and the land buy go first in its budget chain, as decision 398's land buy does.

Without `market` the pop is decision 398's rule, bit for bit. With one pop the market form is the
commons frame's msat bit for bit (§4.1). The provider pays each pop N_i·P_s through its `more`
transfers (decision 395), in list order. R13: each pop reads posted prices and its own params
(N_i, χ_i, s₀, s̲, h, T_o,i), never another pop's demand.

**Its limit in the Enclosed regime.** At r_o ≥ r each pop fills its own share first. That is
exact where every pop's plots exceed its share at r (CT2's commons × 0.5: 1.51 and 1.92 of each
share). Where a pop's plots fit its share at r while another's spill, the commons has room at
r_o ≥ r and excess demand at r_o < r, so r_o orbits r (argued; no CT2 target is there). A load
check cannot see it; the frame registers it as an open item (§11).

### 6.4 In the harness (`probe::markets`)

- **Instances** `IL1` and `CT2` (§3.1), with genesis from unit 1e's point: IL1 in wage units
  (w 1, land 0, the provider's coin 0, the workers' coin w·S/share(spend)); CT2 in rent units
  with the commons at 0 and each pop's coin from its own income at the point.
- **Wage units at IL1** (decision 153): the observables are every price over w (not v = w/r),
  each desk's threshold x_j = 1 − s_j (the wall's, decision 396), each market's cleared volume
  (land's target the land in use, T_m + T_p) and each desk's output: 20 observables. r/w is a
  readout.
- **Grammar at IL1**: p[land]=V sets land's price to V·w (V 0.00125, 0.005, 0.025 for Tiers 1–3:
  the rents that raise the basket's cost by about 5%, 20% and 100%, since P_s/B^q is 0.02508);
  p[land]\*F is a no-op there. s[D]=V at 0.05, 0.2, 0.5 and x\*/2 as the wall's. JA is a nominal
  move in wage units (land's price 0 stays 0), so it is never VACUOUS at IL1, as N is not. The
  targets `inst.land=V` (T a year) and, at CT2, `exit.To=V` (the commons a year, split equally).
- **The runaway bound** of a free-able market is [0, 1e6 × max(its genesis price, its
  reference's)]; every other price keeps [1e-6, 1e6] × genesis.
- **Readouts**, from posted prices through the rules' own code: ticks each free-able market posts
  0, its first free tick, its switches between 0 and a positive price, its price over its
  reference at the end against the oracle's; the provider's lowest coin (IL1: 0); each pop's
  commons bid, offer and net rent; the regime (at IL1, `Idle` where the rule says `Enclosed` at
  r = 0; at CT2 from the commons' posted price: 0 `Commons`, below r `Crowded`, at or above r
  `Enclosed`, never from bids against offers, which differ by rounding at rest).
- **Tier 3S at CT2** scales both pops' coins together as `coin.workers`, as the mirror does.

### 6.5 Tests

Each must fail with the change it guards undone:
1. `free_step_absent_is_saturate`: `next_price` with `free` absent against today's over 3,000
   random (p, k, S, D), bit for bit; and with c = 0, bit for bit.
2. `free_step_posts_zero_and_reopens`: a market with S > D steps to exactly 0.0 in finite ticks
   and stays; at 0 with D > S it posts c·p_ref·expm1(kx) > 0; at 0 under Hold one-sided it stays.
3. `free_market_takes_buys_for_nothing`: at a posted price 0 a buy with budget 0 is feasible in
   full and settles with no currency moved; at a positive price admission is today's.
4. `only_free_goods_may_be_zero`: a genesis price 0 or a step to 0 on a good without `free` is
   refused (`BadValue`, `NonFinite`), with it accepted; each load check of §6.1.
5. `free_field_moves_no_world_id`: every committed tape's text, `tape_hash`, `world_id` and
   2,000-tick hash stream unchanged (gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
   demo-gb `0xfad880fe08d06645`, and every probe, markets, horses, loops, wall and commons pin).
6. `pops_on_a_commons_market_rule`: rule-level, with `market` present a pop's hours, commons bid
   and land buy equal §6.3's formulas over 3,000 random states at CT2 (prices over twelve
   decades, r_o on both sides of r̂); with `market` absent, decision 398's rule bit for bit.
7. `il1_ct2_rest_at_the_oracle`: every registered target, three ticks from the oracle's f64 point
   leave every observable within 1e-12 in log, every market trading, the free-able market at the
   oracle's price (0 or positive).
8. `harness_reads_free_state`: the free readouts and IL1's `Idle` label from the rules' code.
9. `il1_ct2_tapes_are_their_generators_output`.

## 7. The rest point is the oracle's: the proof

Take a live rest point: every market clears and trades, every actor spends its income, nothing
spoils (MARKETS-SPEC §4.1).

1. **The free step's rest set is complementary slackness.** For a free-able market at rest,
   p' = p. If p > 0 then (p + c·p_ref)·e^(kx) − c·p_ref = p, so e^(kx) = 1 and x = 0: S = D, as
   Saturate's rest. If p = 0 then max(c·p_ref·expm1(kx), 0) = 0 iff x ≤ 0: D ≤ S at price 0. So the
   rest points are exactly p ≥ 0, S ≥ D, p·(S − D) = 0: SSRN A.1's "unused fixed inputs have zero
   rent" and unit 1e's r_o = 0 on a commons with room. No state, so no other rest point.
2. **IL1 with land free.** Desks' costs have no land term, so each price is its unit cost in
   labour and machine services; the machine's is λ·w/(1 − a); every desk's target share is 0
   (θ·w/p_mach = 14 > γ(1) = 1) and s' = s(1 − adjust) rests only at s = 0: x = 1. These are unit
   1e §4.6's idle-stretch prices in wage units. The workers' rule at r = 0 takes the plot branch
   (0·h < p_g·Δ), supplies n(p_g·s₀), the oracle's S, and buys T_p = h·(N − S) at 0, filled in
   full (§6.1). Labour clears: n_D = S. Every household spends its income on the common basket:
   w·S + 0, the oracle's I, so Y is the oracle's and T_m = Y·B^q = S·B^q/L^q. Land's rest (x ≤ 0)
   is T_m + T_p ≤ T: the idle stretch's condition. So a live rest point with land free is IL1's
   point, and T_idle = T − T_m − T_p.
3. **IL1 with land priced.** Then S = D on land: all land is in use, and prices relative to w
   satisfy the wall's (or the line's) equations with r > 0: an equilibrium with scarce land.
   The scan of §3.3 finds one sign change at every target, on the idle stretch at 12 of 13 and on
   the wall at inst.land × 0.5. So at an idle target there is no rest point with land priced, and
   at inst.land × 0.5 the rest point is the wall's, with the free step at x = 0.
4. **CT2.** Each pop's hours and plot bid at posted prices are unit-1e.md §4.3's for its type at
   plot rent min(r_o, r̂). With the commons' rest set of step 1: r_o = 0 with Σ G_i(0) ≤ T_o is
   `Commons`; 0 < r_o < r with Σ G_i(r_o) = T_o is `Crowded` (the oracle's shadow rent, G
   nonincreasing); r_o ≥ r with every pop filling its own share and renting G_i(r) − T_o,i is
   `Enclosed` (§6.3's condition). Land clears T = T_m + T_p. Total spending on baskets is
   w·S + r·T_m: the commons rents are paid between pops and the plot rents on enclosed land to the
   provider, as §3.2 of the commons frame, so goods clear at the oracle's Y. By §3.3's uniqueness
   the live rest point is the oracle's point, with the commons' price its shadow rent.
5. **The mirror confirms it.** Three ticks from each instance's point leave every price bit for
   bit at IL1 (drift 0.0) and within 4.4e-16 at CT2; the free-able market stays at 0.0. Mode A at
   L: D̂ 2.2e-13 (IL1) and 8.9e-13 (CT2), the free market at 0 on every tick. `lin_free`'s rest
   states at all 26 targets drift 0.0 over a tick.

## 8. R1, R3, R13, R14

- **R1.** `free` absent: `next_price`, admission, genesis and the range checks are today's, and
  the field is not serialized, so every committed tape keeps its text, `tape_hash`, `world_id` and
  stream. c = 0 is Saturate bit for bit (§4.1). `market` absent: decision 398's rule.
- **R3.** The step moves the price only by the market's own imbalance. Its one nonlinearity is the
  price's domain, p ≥ 0: a step that would go below 0 posts 0. It never holds a price at any value
  but 0, and 0 is the oracle's price wherever it binds at rest (step 1 of §7). It adds no
  smoothing layer and no cap. It does change the step's size at a positive price, to
  k·(1 + c·p_ref/p), as a rate dial does. A reviewer may read "posts 0" as a floor at 0; the frame
  records that reading as its main risk (§10), with the evidence that without it no price rule of
  the scan reaches 0 in finite time.
- **R13.** The markets read posted prices (the reference's). The pops read posted prices and
  their own params. Nobody reads a fill, another actor or the oracle.
- **R14.** c is a registered `Dimensionless` param (hours of the reference good per unit of the
  free good, a quantity ratio), live, with its basis `Assumed("FREE-SPEC: the scan's window
  0.3–1 at C2m, c 0.5")`. No absolute epsilon: the free state is decided by the step's sign.

## 9. The protocol and REGISTERED PREDICTIONS

Registered 2026-09-30, before any engine code for the free step, the pops' commons market or
the two instances exists. The mirror is `model/fm.py` with c 0.5 (`CANDS["zp05"]`); every number
is from `runs/scanF.jsonl` (mode A at 52, the battery, the dial family), `runs/regH.jsonl` (the
rest), `runs/lin_all05.out`, `runs/lin_dial05.out` and `runs/lin_enclosed.out`, summarised in
`runs/pred05.out`.

### 9.1 The engine run, in order

- **E0: the build's trace diff.** The engine against the mirror's traces in `e0/` (2,000 ticks,
  genesis carry, every double written exactly; `model/trace_free.py`): IL1 `hold`,
  `p[land]=0.025` (free again at tick 4), `inst.land=260@genesis` (reopens to the wall),
  `p[labour]*2`, `JB(2)` (30 priced ticks); CT2 `hold`, `exit.To=17.55@genesis` (reopens to
  Crowded), `b.food=1.2@genesis` (Crowded at r_o 0.0118·r, 22 free ticks first),
  `exit.To=9.75@genesis` (Enclosed), `p[labour]*2`, `p[land]*2` (94 priced ticks). Every positive
  price and every other state value within 1e-12 in log of the mirror's, and each free-able
  market's price exactly 0.0 on exactly the mirror's ticks. A parting blocks scoring until a dated
  amendment explains it, as the frames' E0s did. E0 also compares the harness's first joint draw
  with the mirror's at CT2, whose market order includes the commons (O106's lesson).
- **E1: nesting (R1).** Every pin of §6.5 test 5; tests 1, 5 and 6.
- **E2: mode A and L.** Mode A at 12, 52 and 365 a year; L from the engine's elasticity probe,
  which leaves out a free-able market posting 0 at genesis (its price has no log to perturb).
- **E3: the verdict battery.** Tiers 1–3 at L, Tier 3 again at 10·L, Tier 3S (decision 368) at L
  and at 10·L, with a kick set at every target.
- **E4: the dial family** (17 settings × Tier 3), reported beside the verdict (decision 399's
  amendment, PHASE2-S1 §6).
- **E5: the families**, reported: stocks, joint2, joint4, and Tiers 1–2 at 12 and 365 a year.

**The verdict, per instance** (MARKETS-SPEC §7.9 with Tier 3S): GO where mode A passes, every
non-vacuous run of Tiers 1–3 and Tier 3S is CONVERGED, Tiers 3 and 3S are CONVERGED again at 10·L,
every target's kick set decays, and every CONVERGED run ends with its free-able market at the
oracle's price (0 where the oracle's is 0, positive where it is positive). LOCAL: Tiers 1–2
CONVERGED, some Tier 3 or 3S run not. NO-GO otherwise.

### 9.2 The prediction: IL1 and CT2 GO

**L** (the elasticity probe's rule, `model/elasticity_free.out`): IL1 56,000 at 52 a year (τ_max
care's price, 277 ticks), 13,000 at 12, 390,000 at 365; CT2 141,000, 26,000 and 1,114,000 (τ_max
care's, 704 ticks).

**Mode A**: IL1 D̂ 2.2e-13 at 52, 2.2e-13 at 12, 1.1e-13 at 365, land free on every tick; CT2
8.9e-13, 9.0e-12 and 2.6e-12, the commons free on every tick.

| | Tier 1 | Tier 2 | Tier 3 | Tier 3 at 10·L | Tier 3S | Tier 3S at 10·L |
|---|---|---|---|---|---|---|
| IL1 | 25/25 | 33/33 (+4 V) | 36/36 (+2 V) | 36/36 (+2 V) | 22/22 | 22/22 |
| ticks to tol, median (slowest) | 281 (474) | 334 (570) | 516 (678) | the same | 356 (547) | the same |
| peak D̂; most dead ticks; lowest baskets/Y\* | 1,573; 34; 0.207 | 3,118; 48; 0.044 | 5,774; 119; 0.003 | | 1,253; 2; 0.286 | |
| CT2 | 30/30 | 40/40 (+2 V) | 41/41 (+2 V) | 41/41 (+2 V) | 24/24 | 24/24 |
| ticks to tol, median (slowest) | 306 (498) | 354 (5,080) | 512 (3,357) | the same | 342 (543) | the same |
| peak D̂; most dead ticks; lowest baskets/Y\* | 439; 0; 0.675 | 1,840; 82; 0.185 | 7,593; 151; 0.000 | | 1,253; 46; 0.286 | |

V: VACUOUS by construction (§3.4). Tier 3S is each desk's stock and coin and each household's coin
at × 0.5 and × 2 (IL1's provider holds no coin, so it has 22 runs, CT2 24, with each pop's coin
scaled together as `coin.workers`); its start distance is the first year's largest D̂, least 32.0
(IL1) and 65.1 (CT2), so none is VACUOUS. The slowest CT2 runs are its Crowded targets: the commons
× 0.9 (5,080 ticks) and b.food × 2 (3,357 at genesis, 2,946 dated).

**The kick sets** (the largest root per tick, PL, and the bar PL^(0.9L) ≤ 1e-3):

| target | IL1 | CT2 |
|---|---|---|
| the base and every free target | 0.988687 (half-life 61): 12 of 13 targets | 0.985484–0.987912 (47–57): 10 of 13 |
| the scarce or crowded targets | inst.land × 0.5: 0.989993 (69) | commons × 0.9: 0.998697 (532); b.food × 2: 0.999846 (4,493), the bar e^-19.6 |
| the Enclosed target | | commons × 0.5: 0.987107 (53), with the commons' price inside its continuum (§4.4) |

Every bar passes. Over the 17 dials the largest roots are 0.991467 (IL1 base, rate × 0.75),
0.992783 (IL1 inst.land × 0.5, rate × 0.75), 0.999024 (CT2 commons × 0.9, rate × 0.75) and 0.9903
(CT2 Enclosed, buffer × 0.75); at CT2's b.food × 2 every dial's root is at most 0.999884
(rate × 0.75, the bar e^-14.7), and all pass their bars. At CT2's Enclosed target a kick that
lifts land's price by more than the commons' gap above r (0.13–0.81% at the runs' ends) crosses
the switch of §6.3: such a kick is reported, not scored (§11, OF1).

**The free-able market** (every CONVERGED run, both instances, all sets): it ends at 0 at every
free target and positive at every priced one, at the oracle's price there (r/w 0.025366 at IL1's
inst.land × 0.5; r_o/r 0.641199 and 0.011781 at CT2's Crowded targets; at or above r at CT2's
Enclosed target). From a positive land price IL1's market is free within 1–4 ticks; the most
free/priced switches in a battery run are 6 (IL1, JB(2)) and 2 (CT2). The regime at the end is the
oracle's in every CONVERGED run (IL1: `Idle` 670, the wall's `Enclosed` 36; CT2: `Commons` 734,
`Crowded` 38, `Enclosed` 36, read from the commons' posted price, §6.4).

**The dial family** (Tier 3 at 17 settings): IL1 612/612 (+34 V), median 507, slowest 917 (rate
× 0.75); CT2 697/697 (+34 V), median 483, slowest 18,365 (rate × 0.75, b.food × 2). Per setting the
Tier-3 medians lie in 414–673 (IL1) and 309–607 (CT2).

**The families** (reported, not in the verdict):

| | stocks | joint2 | joint4 | 12 a year, Tiers 1–2 | 365 a year, Tiers 1–2 |
|---|---|---|---|---|---|
| IL1 | 39/39, 392 (662) | 60/60, 540 (658) | 39/40, 645 (834); 1 in the subsistence trap | 58/58 (+4 V), T1 111 (341), T2 127 (450) | 58/58 (+4 V), T1 1,799 (3,116), T2 2,052 (3,792) |
| CT2 | 41/41, 420 (561) | 60/60, 546 (661) | 33/40, 594 (890); 7 in the subsistence trap | 70/70 (+2 V), T1 860 (1,310), T2 1,052 (1,398) | 70/70 (+2 V), T1 1,968 (2,404), T2 2,294 (35,674) |

The trap runs are joint(4, s) for s 10 (both), 12, 18, 21, 22, 34 and 39 (CT2), running away at ticks
254–276: nobody works, food is not supplied, and every price inflates together (O100). The free
step does not cause it: joint(4, 10) under Saturate runs away at the same tick 254 at both
instances (`runs/trap_check.out`). C2 put 8 of 40 joint4 runs in the trap; CT2 puts 7.

### 9.3 How close the engine must be to the mirror

As the frames': every class exactly; ticks to tolerance within 10%, with up to three runs an
instance within 25%; lowest baskets and cleared volumes within 0.05 absolute where the mirror's is
above 0.1; dead ticks within 10% or 5 ticks; the free-able market's end state exactly (0 or
positive) and its first free tick within 2 ticks; its free/priced switches within 10% or 2;
runaway ticks of the trap within 5% against the harness's reference (decision 399 as amended);
PL within 1e-4 where it is below 0.999, within 2e-5 above.

## 10. What would refute the frame

- A registered free-able market that ends priced at a free target, or free at a priced one, in a
  CONVERGED run; or one whose price leaves [0, 1e6 × its scale].
- Any IL1 or CT2 target where the engine's live rest point differs from the oracle's point beyond
  the tolerance (§7's proof would be wrong).
- A pin that moves (R1), or any committed tape's `world_id` that moves when `free` is absent.
- An engine Tier 1–3 or 3S run of either instance that is not CONVERGED, or a kick set that fails.
- The main risk, a reading rather than a run: that "a step below 0 posts 0" is a price floor under
  R3. The frame's answer is §8; the ruling is yours (decision FG1 below).

## 11. Decisions proposed and open items

Claude's, on your delegation, each open to veto, to be numbered in the session's range (400–449)
at registration; the alternative named is the registered one (R6).

- **FG1. The free step is the zero price markets hold (O96, O101).** An optional per-good field:
  p' = p·e^(kx) + (c·p_ref)·expm1(kx), 0 where that is not positive; a buy at a price of 0 is
  feasible in full; off when absent, bit for bit. Alternative: the snap (zs at φ 0.001 of the
  wage), exact at 0 and above φ·w, slow at small rents and without a rest point below φ·w.
- **FG2. c 0.5, with labour's price as the reference, a live tape param.** The window at C2m on
  both instances' batteries and dial families is [0.3, 1]. Alternative: c 1, the scan's first
  pick, inside the window but at most 1.5 from its upper edge.
- **FG3. IL1 is Phase 2's idle-land instance, under the stated transfer** (§3.2): the provider pays
  its rent income, 0 at r = 0, and the workers' support is their own household's; the oracle's
  `funded` false is disclosed. This tests decisions 153, 161, 376 and 381, and 398's deferral ends.
  Alternative: 376's "no idle-land instance" until a 1f transfer funds the support.
- **FG4. CT2 is Phase 2's several-types commons (O101).** The commons is traded on a market the
  pops hold in equal shares and offer, each filling its own share first at r_o ≥ r̂ (§6.3), with
  the free step. Alternative: a commons actor, a new role that holds and offers the commons and
  spends its rent.
- **FG5. One plot-taking type keeps decision 398's rule.** Alternative: the market form everywhere,
  2.5–3.6 times slower at C1.
- **FG6. IL1's harness in wage units** (§6.4): 20 observables with prices over w and thresholds
  x_j; p[land]=V at 0.00125, 0.005, 0.025 of w; the targets inst.land and the commons; a free-able
  market's runaway bound [0, 1e6 × its scale]. Alternative: rent units with r/w an observable,
  which has no log at 0.
- **FG7. The verdict for IL1 and CT2** is §9.1's: P2.1's with Tier 3S, the kick sets and the
  free-able market's end state; the dial family and the families reported. Alternative: the dial
  family in the verdict (PLAN's "green with margin").
- **FG8. Decision 160 stays untested.** A land-alone exit good is free at r = 0 too, and the
  workers' branch test r·h < p_g·Δ reads 0 < 0 there, so they would take the floor where 160
  decides at the wall's end (q = 1/b̃_g); the rule would need the desk's recipe (R13), and the
  roles have no land-alone category. Alternative: a land-only category and a limit rule now.

Open items (to be numbered in O110–O129):

- **OF1. The Enclosed regime with several pops.** Filling one's own share first is exact only
  where every pop's plots exceed its share at r; where one pop's fit, r_o has no rest and orbits r
  (argued; no CT2 target is there). And the Enclosed point has a switch at r_o = r and a continuum
  of rest points above it (§4.4): a land kick larger than the commons' gap crosses the switch.
- **OF2. c per instance.** The window's upper edge is set by the smallest positive price the
  market must reopen to (c·p_ref·(e^k − 1) against r\*), the lower by the slowest positive rent
  (the kick bar). The 1750-like instance registers its own c after its own scan; the free step
  also changes a positive price's step size to k·(1 + c·p_ref/p), a dial interaction.
- **OF3. IL1's accounts.** The provider's baskets are 0 in the agents and −0.2 in the oracle's
  books; only the split between households differs, as O102's does at C1's Enclosed targets.
- **OF4. The commons' slow mode at a small rent.** CT2's b.food × 2 (r_o 0.0118·r) has a half-life
  of 4,493 ticks at c 0.5; CT2's Crowded band is narrow (the commons 17.09–18.20 a year).
- **OF5. The one-sided free state (zo)** never acts where a good has buyers. A market with none at
  all (O54's switched-off technique) is where it would, and it is unscanned.
- **OF6. The regime readout of a commons market** must come from its posted price (0, below r, at
  or above r), not from bids against offers, which differ by rounding at rest (8 of the mirror's
  Crowded runs read `Commons` that way).

## 12. What this frame did not do

- It changed nothing in the worktree and built no engine code.
- It did not test decision 160 (FG8), ρ > 0, a government, 12 ticks a year as a default, or a
  free step on any good but land and the commons.
- It did not scan the snap or the free step with a reference other than labour.
- It did not run the dial family's local roots at every target, only at both bases and the four
  priced targets.
- It did not re-run the commons frame's C1 and C2 with the free step beyond their batteries.

## 13. Files

In `D:/rustyecon-p24/scan-free/`, sha256 in `SHA256SUMS`:

- `SPEC.md`, this file.
- `model/`: `fm.py` (the free mirror), `cm.py` and `p21/` (the commons frame's mirror, unedited),
  `ofree.py` (the f64 oracle in wage units), `fb.py` (the battery), `run_free.py` (the jobs),
  `lin_free.py`, `lin_enclosed.py` (local roots), `nest_free.py` → `nest_free.out`,
  `elasticity_free.py` → `elasticity_free.out/.json`, `table_free.py`, `pred_free.py`,
  `summ_free.py`, `smoke_free.py`, `trace_free.py`, `targets_free.py`, `search_il.py`, `search_ct.py`,
  `search_ct2.py` (how the instances were chosen), and the frame's `instances.py`,
  `battery_c.py`, `run_battery_c.py`, `elasticity.py`, `elasticity.json`, `lin_c.py` (unedited).
- `solve/`: `fsolve.py` (the independent 50-digit solve), `points_*_coarse.json/.log`,
  `points_fine.json`, `fine.out`, `collect_fine.py`.
- `check/`: the scratch crate on the worktree's oracle (`Cargo.toml`, `src/main.rs`, `run.sh`),
  `check.out`, `compare.py`, `compare.out`.
- `runs/`: every run (`scan{A..G}.jsonl`, `reg{Z,H}.jsonl`, `try1.jsonl`), the tables and the local
  roots (`*.out`, `*.json`), `pred05.out`.
- `e0/`: the mirror's traces for E0.
- `base-commons/`, `base-wall/`: copies of the frames' scratch, unedited.
