# LOOPS: the loop step on the engine (P2.2b)

Dated 2026-09-30. Steps P2.2b.0–P2.2b.4 on branch `phase2-plants`, from `reboot` at `8b07c8a`.
Registered at L0.8 (`452c0e7`), before any plant code: [LOOP-SPEC](loops/LOOP-SPEC.md) as
[LOOP-SPEC-A1](loops/LOOP-SPEC-A1.md) amends it (§A1.5 holds E0–E11), [FUNDED](loops/FUNDED.md)
with FUNDED-A1, and [registration.md](results/loops/registration.md). One amendment came before
any code, [LOOP-SPEC-A2](loops/LOOP-SPEC-A2.md): LN5 waits for O51. Spec
[LOOPS-RULES.md](LOOPS-RULES.md); results [results/loops/](results/loops/README.md); plots
[figs/loops/](figs/loops/).

**Conditions of every number below, unless its line says otherwise.** chain8 (1g's horse county
at N 8) under rule B: fodder raised with horse-days, the horse held by M3's wet capacity desk and
bred by M2's maker from bought fodder. CAPACITY's plant y = K^(1−θ)·z^θ on the fodder desk, the
capacity desk (beside the horses, its target the herd's plant) and the maker: θ 0.8, plant δ 10%
a year, s1, s_Kp = 2δ_p. The maker's reservation at ψ 0.25. C2g, 52 ticks a year, ρ 0, J_b 1,
ex post. L 202,000–232,000 ticks; Tier 3 and 3S again at 10·L. The frozen `horses` of `3205025`
(sha256 `e0e8f2a6…893a`), release, WSL. "The mirror" is the registered `lm_carry.py`.

## 0. The verdict

| id | what it is | mode A | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | stocks | verdict |
|---|---|---|---|---|---|---|---|---|
| LB1 | rule B, horse δ 8% | 8.9e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LB2 | horse δ 10% | 2.4e-15 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LB3 | horse δ 4% | 5.6e-16 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LF1, LF7, LF8; LF2; LF5; LC1 | θ 0.7; δ_p 4%; fodder rate 1.3; the loop cut | ≤ 5.0e-15 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO, each |
| LW1–LW3 | the flow control, with plants | ≤ 2.0e-15 | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | GO (O14's reference) |
| LF3 | LB1 at ψ 0 | PASS | 20/20 | 24/24 | 24/25 (24/25) | 27/28 (27/28) | 25/28 | NO-GO |
| LN1, LN7 | no plants; the loop cut without plants | PASS | 0/20 | 0/24 | 0/25 | 0/22 | 0/22 | NO-GO |
| LN2–LN4, LN6, LN8, LN9, LF6, LW0 | the other controls (§3) | | | | | | | as registered |
| LN5, LF4 | storable fodder | not built (A2; O51) | | | | | | |

**GO for rule B's horse loop at chain8, as registered, and narrower than the run first read it.**
Every registered prediction held: 1,276 scored lines pass, 100 are reported, LN5 is not run. At
the 12 GO instances, over 2,052 runs, the engine is the mirror in every class, tick to tolerance,
withheld tick and tick at labour's bound. Every converged run ends within 1.9e-13 in log of its
oracle point, every kick set decays, and no refutation criterion was hit. The two reviews confirm
the verdict and narrow how it reads, in three ways.
- **The plants do not damp the loop alone.** The maker's reservation acts in 117 of LB1–LB3's 375
  runs at L; without it (LF3) LB1 loses five, the pass-through's r × 2 among them. Through the
  pass-through and the gluts the maker breeds and sells nothing: its coin falls to 1e-5 of
  genesis or below (4e-90 at worst), and the horse price sits frozen at 0.15–0.22 of target for
  up to 48 years, an idle market's price, not a valuation.
- **The time to tolerance is mostly the plants'.** The maker's plant is the last observable into
  tolerance in 59 of LB1's 97 converged battery runs and 82 of LB3's. After b × 2 the economy takes 43–67
  years; the heads come within 5% in 1.1–2.1 times the paper's time.
- **The GO covers one small loop in one funded county**: under 1% of the land, fodder that cannot
  be stored, C2g. The design was chosen in the mirror on this same battery, so the GO shows that
  the engine is faithful, not that the design works elsewhere.

Like for like, stocks lift the b × 2 trough by 0.036 in log and take 1.36 times the flow
control's median years, 1.17 times on the observables both share; at horse δ 4%, 2.2 and 1.9.

## 1. What was built

- **Core: an untraded good** (decision 286): `Indefinite`, with no price rate, market or genesis
  price. `World::markets()` lists traded goods only, so clearing, the price rule, the report,
  certify's kick set and the GUI never see a plant. The engine's one change boxes a delta (297).
- **Agents: the plant** (`roles/plant`), an optional last field on the TypeDesk, the maker and the
  capacity desk, with three `ActorState` variants after `Owner`. It is minted in produce and
  burns u·P at upkeep; at θ 1 a desk is its plain kind bit for bit.
- **Probe:** `probe::horses::loops`: the 23 built instances, six tapes `tapes/loops-*.ron`, and
  the rule-B readouts (the plants and their 5% times, dead ticks by market, labour's bound, the
  horse-days by buyer, the reservation's, the running cost read from the tape).
- **R1 held**: the 13 horses and markets tapes' 2,000-tick streams and the pins, on both machines.
  905 workspace tests (28 new); 44 mutants, each killed by a named test. Commits `6142c9a`,
  `e58e07e`, `3205025`, `6ae6674`, and P2.2b.4 (this report and the fix round).

## 2. The prediction and the result

| registered (A1 §A1.5) | result |
|---|---|
| E0: trace diff within 1e-12, nothing at tick 1 | **Held.** 8 of 12 runs agree within 8.7e-13. Four part on the horse volume's cancellation, by 3.9e-12 to 2.9e-11. Nothing parts at tick 1. How it was read: §5 |
| E1: pins unchanged; θ 1 is the plain tape; a fixed plant is drs | **Held.** The fixed plant agrees within 7.4e-14, but the horse volume (1.7e-12, 2.4e-12), by the same cancellation |
| E2: rest point within 1e-12; mode A below 1e-9 | **Held**: 8.9e-16 to 2.4e-15 (the mirror's 6.7e-16 to 4.5e-14) |
| E3: LB1–LB3 GO with A1 §7.1's numbers | **Held**: 537 of 537 CONVERGED; §7.1 to printed precision, but four one-tick ties (O77) |
| E4: kick sets decay; g 0.847, 0.830, 0.901, 0.836 a year ± 0.03 | **Held**: 24 of 24; 0.8478, 0.8310, 0.9008, 0.8463; largest g 0.998971 a tick |
| E5: A1 §7.2's cost shocks | **Held** to printed precision; LB1 b × 2: trough 0.704, 46.3 years, horse low 0.194 |
| E6: heads × 2, × 10 converge, no dead tick; withheld 303, 1,315; 221, 1,034; 662, 2,694 | **Held**, tick for tick. The horse market is idle 353–3,047 ticks (§4) |
| E7: LB1 r × 2 ≤ 5 horse-day dead ticks, fodder 175; LN7 ≥ 100, not CONVERGED | **Held**: 0 and 175, CONVERGED in 50.3 years; LN7 146, ORBITING |
| E8: LW1–LW3 GO; trough 0.036 higher; 1.36 times the years | **Held**: 0.0362; 1.361 (26.5 against 19.5 years) |
| E9: the controls keep their verdicts, counts within 2 | **Held**; LN7's stocks family 0/22 against 1/22 |
| E10: the families GO; E11: A1's runs at labour's bound | **Held**; LF4 not built. 28 runs at the bound, A1's ticks and peaks, each CONVERGED |
| Refutations | **None**: no class change, runaway at ψ 0.25, end gap above 1e-12, θ 1 parting or g above 1 |

## 3. Mode A and mode B

Mode A passes at L at every GO instance (largest gaps 2.2e-16 to 5.0e-15). The kick sets (12 of
1e-9, horizon L) decay at every target; the engine's g is the mirror's within 0.005 a year at
LB1–LB3 and 0.016 at LW1 (O78). At 12 ticks a year LB1 converges in 0 of 40 and mode A orbits:
weekly stays the floor (decision 268). At 10·L, Tiers 3 and 3S give the same class and ticks to
tolerance in every run. The controls keep A1's counts in every tier but LN7's stocks family.
Without plants the loop does not converge (LN1, LN7). The maker needs its plant (LN2 orbits in 16
of 25 Tier-3 runs), and so does the fodder desk (LN4 loses a stocks run). The capacity plant's
target must be the herd's (LF6), and orders must be partial (LN6 is DEAD). A slower fodder price
does not replace the plants (LN8, LN9). The flow control needs them too (LW0 runs away at 141).

## 4. The pass-through, O14 like for like, and capital's time

**The pass-through and the reservation** ([relay.csv](results/loops/relay.csv),
[relay_counts.csv](results/loops/relay_counts.csv); 10,400 ticks rerun with a row every tick).
After r × 2 the capacity plant keeps LB1's horse-days live (0 dead ticks, LN7 146), but the loop
converges through the reservation too ([fig2](figs/loops/fig2_passthrough_r2.png),
[fig4](figs/loops/fig4_glut_heads_x10.png)). Coin and output: the maker's lowest, of genesis and
of target.

| run | years | withheld | horse idle | price frozen: ticks, of target | coin | output | maker's plant low | at ψ 0 |
|---|---|---|---|---|---|---|---|---|
| LB1 r × 2 | 50.3 | 251 | 412 | 145, 0.149 | 4.0e-7 | 6.0e-6 | 0.45 | runaway, tick 170 (LF3) |
| LB3 r × 2 | 78.9 | 404 | 587 | 282, 0.150 | 9.2e-12 | 1.0e-9 | 0.32 | – |
| LB1 heads × 2 | 45.4 | 303 | 477 | 139, 0.223 | 2.2e-8 | 6.6e-7 | 0.40 | runaway, tick 156 |
| LB1 heads × 10 | 65.3 | 1,315 | 1,524 | 1,161, 0.223 | 1.3e-43 | 2.9e-35 | 0.063 | runaway, tick 156 |
| LB3 heads × 10 | 118.2 | 2,694 | 3,047 | 2,510, 0.223 | 4.3e-90 | 1.1e-72 | 0.0049 | runaway, tick 155 |

Over LB1–LB3's battery and stocks family at L the reservation acts in 117 of 375 runs, and in 19
the maker's output falls below 1% of target (52-tick rows; 21 on every tick's troughs). The
harness counts the horse market idle, not dead, so E6's "no dead tick" stands beside 353–3,047
idle ticks. With no offer and no order the price rests at 0.22 of target after a glut, and after
r × 2 it holds from 0.15 of target, the ratio drifting to 0.18 as v moves (decision 245; O54).
This is O52's collapse of the maker's coin, under rule B, but with no deep shortage after it: the
heads stay at 0.89 of target or above and the price peaks at 3.3–3.8 times target after heads ×
10, where rule A's heads fell to 0.40–0.53 and its price rose 8–120 times.

**O14 like for like** ([o14.csv](results/loops/o14.csv), [o14_years.csv](results/loops/o14_years.csv),
[o14_trough.csv](results/loops/o14_trough.csv)). Tier 1–2 median years over converged runs; the
last three columns on 52-tick rows; "shared" is the observables both have, less the plants.

| stocks / flow | trough, ln of old Y | years (harness) | all observables | without plants | shared |
|---|---|---|---|---|---|
| LB1 / LW1 (δ 8%) | −0.353 / −0.389 | 26.5 / 19.5 (1.36) | 27 / 20 | 25 / 18 | 21 / 18 (1.17) |
| LB2 / LW2 (δ 10%) | −0.336 / −0.375 | 24.1 / 19.5 (1.24) | 25 / 20 | 23 / 18 | 21 / 18 (1.17) |
| LB3 / LW3 (δ 4%) | −0.390 / −0.418 | 42.8 / 19.3 (2.21) | 43 / 20 | 42 / 18 | 35 / 18 (1.94) |

Half of the registered 1.36 is what the flow control does not have: the maker's plant and the
horse market. And the trough is a path, not an equilibrium. b × 2 moves the good's equilibrium by
−0.16% in log (Y\* 9.918 to 9.902), yet the good falls to 0.704 of the new Y\* at LB1 and 0.679
at LW1, at tick 9, and stays below 0.9 for 27 and 32 ticks: 217 and 239 times the move, in log.

**Capital's time (D-G14)** ([capital_time.csv](results/loops/capital_time.csv),
[last_in.csv](results/loops/last_in.csv), [fig5](figs/loops/fig5_plants_b2.png)). Years to
tolerance, then to within 5% of target:

| run | to tolerance | heads | the paper's heads | capacity plant | maker's plant | fodder plant |
|---|---|---|---|---|---|---|
| LB1 b × 2 | 46.3 | 10.1 | 6.2 | 9.4 | 19.5 | 8.6 |
| LB2 b × 2 | 43.4 | 10.6 | 5.0 | 11.2 | 18.4 | 9.1 |
| LB3 b × 2 | 66.6 | 12.5 | 11.8 | 24.3 | 36.7 | 8.3 |
| LF2 b × 2 (δ_p 4%) | 76.3 | 26.5 | 6.2 | 34.4 | 18.6 | 24.2 |
| LB1 b × 0.5 | 64.6 | 14.4 | 1 tick | 14.2 | 28.3 | 13.7 |
| LB3 b × 0.5 | 135.4 | 24.0 | 1 tick | 34.8 | 55.3 | 31.7 |

The heads take 1.06–2.1 times the paper's zero-build time after b × 2 (4.3 at δ_p 4%; P2.2a's
rule-A heads 5–12 times), and 14–24 years after b × 0.5, which the paper fills in a tick. The
quasi-rent three ticks on is +6.1% to +7.0% after b × 0.5 and −3.4% to −3.7% after b × 2. The
last observable into tolerance (ties count for both) is the maker's plant in 59 of LB1's 97
converged runs (the horse market in 50), 82 of LB3's, and a plant in 77 of LW1's 83. So the
years of E3, E5 and O14 mostly time an assumed device, and scale with δ_p (LF2's b × 2: 76 years
against 46). Nothing scores this (decision 309).

## 5. What the reviews found, and the corrected verdict

Two reviews ran after P2.2b.3. **Both say the verdict holds**, and neither changes a class.

**Measurement.** A fresh build of `6ae6674` gives the frozen binaries byte for byte; 147 wave jobs
over all 23 instances rerun byte for byte on WSL, and 8 on Windows; E0's trace diff reruns
identically. An independent rescore from the raw summaries gives every count, all 72 §7.1 rows,
the 48 §7.2 rows and every named outcome. Plant wear is conserved bit for bit, money drifts at
most 3.5e-14, and R13 holds. Three minors, each answered at P2.2b.4:
1. **Decision 304 came after E0's first output**, as a decision, not a dated amendment. Under the
   registered reading (the demand side) LB1 b × 2 in E0 (3.9e-12) and E1's two fixed-plant runs
   (1.7e-12, 2.4e-12) are partings, written up as the same rounding: the maker's offer cancels
   19–92 times. None is at tick 1, so no amendment follows ([e0.md](results/loops/e0.md) §9;
   decision 310).
2. **The scorer's stamp is a scratch file's time**, and `fix1.py`–`fix4.py` edited the scorer
   while the wave ran (02:26:53–02:28:46, 89 jobs done), for what the self-test on the mirror's
   outputs showed. From now on the scorer and job list are committed before a wave (311).
3. **The archive did not regenerate `runs.tsv`** (gzipped CSVs), and E11's aggregate failed open.
   `gather_gz.py` and a fail-closed `score_fc.py` now reproduce `runs.tsv` byte for byte from the
   archive, and every scored table (`D:/rustyecon-p2b/fix-report/`).

**Fidelity.** The plant layer matches `lm_carry.py` line by line; there is no hidden clamp; no
coin cap cut an order in 16 stress runs; the rest-point proof checks against the code. Three
majors and two minors:
- **The pass-through GO is the plant and the reservation together** (§4). Decision 264 is
  reworded, and O52, O54 and the E6 and E7 tables are amended (decision 307).
- **Capital's time was not read.** §4 reads it, the maker's plant included (decision 309; O46).
- **Decision 284 reaches too far.** Decision 308 narrows it (§6).
- Minors: the O14 ratio's makeup and the trough's transient (now in §4); o14.csv gave LW0's
  runaway peak, and gains `T3_peak_ex_runaway` (3,379).

**The corrected verdict.** GO for rule B's horse loop at chain8 (C2g, 52 ticks a year, ρ 0, J_b
1, fodder that cannot be stored), with CAPACITY's plant on every loop desk and the maker's
reservation at ψ 0.25 together. Capital's time is reported, not scored.

## 6. What this means

**For Phase 2 proper and its 1750-like instance.** The loop-free wall and commons instances are
unaffected and open first. A goods chain with a loop now has engine evidence, but narrow, that
plants and the reservation reach unit 1g's point at weekly ticks. So decision 308 narrows 284: a
loop enters Phase 2 only through its own mirror registration, at its dials, in its own funded
county, and then its engine battery. That instance must settle its dials: at Phase 2 proper's
C2m (decision 119) CAPACITY's plant at θ 0.8 is not GO in the mirror (74/77, 113/119), so θ 0.7
(GO here as LF1, LF7, LF8) is the candidate, or the goods chain runs at C2g. Its horse-to-steam
switch idles a machine market whose price will rest near 0.22 of target for decades (O54). S1
needs a funded steam county (O62), grain that keeps needs O51, and shocks will take 29–135 years.

**For the demo's second pass** (v2a.1b on funded counties). v2a.1 (rule A) is ready on funded
counties after your ruling on D-G12 (HORSES §6). v2a.1b needs, per county, chain8's funding check
(chain8 was funded only by moving N from 12 to 8) and a mirror registration at the demo's dials.
Its horse price can sit at 0.15–0.22 of target for years; a lens must show an idle market's.

**For GOODS-CHAIN's next stages.** v2a.1b is done at chain8. v2a.2 adds no loop and needs the
ex-post category desk (O42). v2a.4 and v2a.6 need storable running goods (O51): a seller with a
cover and buyer netting, each with a mirror scan, which also runs LF4 and LN5 (registered; O71).

**Recommendation.** Close P2.2b as GO for v2a.1b's rule-B horse at chain8, read as §5 narrows it,
and take decisions 286–306 with this step's 307–311 (308 narrows 284, yours to veto). Next on
this line: O51's roles, after their mirror scan, with LF4 and LN5 their first runs. Before the
1750-like goods chain carries a loop: its mirror registration at its dials (C2m at θ 0.7, or your
ruling for C2g) in a funded county, with the idle market's run (O54) and each stock's 5% time
registered, the maker's plant included (309). The demo's second pass starts at v2a.1, as P2.2a
said; v2a.1b joins only on counties funded and registered as chain8 was.

## 7. Open questions

1. **The maker's collapse** (O52). Can a floor order or a tilt take the reservation's part without
   moving the local roots, or does it wait for Phase 3's entry and exit?
2. **How far the GO reaches** (O79): C2m, storable fodder, a loop that weighs on land, other
   funded counties and 12 ticks a year are untested.
3. **Capital's time** (D-G14, O46). Can a demo or a scored path window show 29–135 years to
   settle, when an assumed plant sets that time?
4. **Why the maker's plant removes the relay cycle** (O63). **Not tested:** ρ > 0, J_b > 1, S1's
   pumping loop (O62), several machine goods, categories.
