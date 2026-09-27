# MARKETS: the many-markets probe

Dated 2026-09-27. Step P2.1 on branch `phase2-markets`, from `reboot` at `708167f`. Frame:
`D:/rustyecon-p2m/frame/MARKETS-SPEC.md` (MARKETS-SPEC, sha256 `9d0859a6…c874`). Prediction:
`D:/rustyecon-p2m/predict/PREDICTION.md` (`fde5b794…a261`). Build: P2.1.1 (`d0ceaa6`),
[MARKETS-RULES.md](MARKETS-RULES.md). Registration: `D:/rustyecon-p2m/run/registration.md`
(sha256 `a92d9a9c…4a9c`), naming `d0ceaa6`, clean.

**Conditions of every number below, unless its line says otherwise.** The harness `markets` from
`d0ceaa6`, release, WSL. The one-market probe's roles ([REPORT.md](REPORT.md)) as four new kinds
(§1). Dials C2m, which is C2 copied to every market of its role; for the loop also C2L, which is
C2m with type rates 5.2 a year and tilt 1 on every desk. `Imbalance`, `Saturate`, planned
assignment, 52 ticks a year. L is set per instance by PROBE-SPEC §4.4 (20,000–484,000 ticks);
tol = 1e-3 in log on every observable. A run is CONVERGED only if its class is and the 1e-9 kick
set at its target decays (MARKETS-SPEC §7.5). Raw runs: `D:/rustyecon-p2m/runs/`; tables:
[results/markets/](results/markets/) (`D:/rustyecon-p2m/report/make_results.py`); plots:
[figs/markets/](figs/markets/).

## 0. The verdict

| id | what it is | markets | dials | mode A | Tier 1 | Tier 2 | Tier 3 (= at 10·L) | verdict |
|---|---|---|---|---|---|---|---|---|
| I0 | Appendix B in the new roles, the control | 4 | C2m | PASS | 16/16 | 20/20 | 21/21 | P2.0's 57 runs, bit for bit |
| I1 | 1b's four-category fork economy; desks buy land | 7 | C2m | PASS | 30/30 | 38/38 | 39/39 | **GO** |
| I2 | M4's engine and power, operating recipes: a chain | 5 | C2m | PASS | 20/20 | 28/28 | 29/29 | **GO**, slow |
| I3 | I1's categories on I2's types | 8 | C2m | PASS | 34/34 | 42/42 | 43/43 | **GO** |
| L2 | engine and power with build recipes: each buys the other's service | 5 | C2m | kick FAIL | 0/20 | 0/28 | 0/29 | **NO-GO** |
| L3 | I1's categories on L2's types | 8 | C2m | FAIL | 0/34 | 0/42 | 0/43 | **NO-GO** |
| L2 | as above | 5 | C2L | PASS | 20/20 | 26/28 | 18/29 | **NO-GO** |
| L3 | as above | 8 | C2L | PASS | 34/34 | 41/42 | 31/43 | **NO-GO** |
| G1 | 1b's gap economy, optional | 7 | C2m | PASS | 30/30 | 34/34 | 29/29, 6 not expressible | **GO** |

**Many markets: GO at the weekly tick, for economies with no loop of produced inputs. A loop is
NO-GO.** The probe's agents find the oracle's equilibrium in I1 (four categories, three desks
buying land), I2 (a chain of two machine types) and I3 (both), with C2's dials copied to every
market of their role and nothing re-tuned. All 303 registered runs converge. They start from every
price ×/÷2, from the technique at x\*/2 and from cost shocks ×2 and ×0.5, and each ends within
1.03e-14 in log of the oracle; every 1e-9 kick at every target decays. Median ticks to tolerance
in I1 and I3 are near I0's (324–630 against 320–523); I2 is four to five times slower. Where two
machine types each buy the other's non-storable service (L2, L3), every run diverges at C2m. At
C2L the point is locally stable, but ±20% displacements reach an absorbing zero from which neither
desk restarts. Every verdict is the one the predictor registered, run for run; the frame's
prediction that C2L converges was wrong. Both reviews confirm the verdicts and narrow them (§5):
every start had coins and stocks at their stationary values; the GO holds at 52 ticks a year, and
at 12 a year I2's point is unstable; it tests one task margin (decision 60); and I1–I3 are free of
loops only because decision 67 keeps goods out of machine recipes. A11 is not met.

## 1. What was built

- **Roles.** Four new kinds in `crates/agents/src/roles/many/`, with the old kinds, core, markets
  and engine unchanged (MARKETS-RULES §1–§3). `BasketProvider` and `BasketWorkers` buy a fixed
  basket, with space as land. A `CategoryDesk` per category carries its own continuous threshold on
  the shared task line, moving toward 1 − measure(θ·w/p_τ), with unit-1b §4.1's recipe. A
  `TypeDesk` per machine type is P2.0's machine desk plus bought services, with cost
  Σ a_kl·p_l + λw + br, a net markup, and min(a_kk·q, held) kept first. All run the cash rule.
- **Nesting.** On I0 the new kinds reproduce appb's per-tick prices, volumes, fills, coins and D̂
  bit for bit for 20,000 ticks, from four displaced starts. The gate world's and appb's hashes
  are unchanged.
- **Harness.** `crates/probe/src/markets/` (`markets`, `markets-tape`): instances from the
  oracle, batteries, families, O14 statistics, kick sets through certify, the probes; 20 new
  tests, 569 in all. The trace diff against the predictor's mirror agreed within 3.6e-14 (I1, I2).
- **Departures.** The budget chain runs in admission's order (in list order a budget could
  overshoot the coin by half an ulp); six of G1's 99 runs cannot be expressed; this report takes
  its task's names, not MARKETS-SPEC §8's (MARKETS-REPORT.md, results/markets-*.csv).

## 2. The prediction and the result

| prediction | source | result |
|---|---|---|
| GO for I1, I2, I3, G1; NO-GO for L2, L3 at C2m | PREDICTION §0; frame §6.2 | As both predicted |
| L2, L3 at C2L NO-GO through an absorbing zero: Tiers 1–3 20/20, 26/28, 18/29 and 34/34, 41/42, 31/43 | PREDICTION §0, §9 | Those tallies, from the same 13 failing runs in each; runaway at ticks 138–181 (predicted 134–175) |
| L2, L3 at C2L converge | frame §6.2 | **Wrong**: it used PL at the point and ran no displaced start |
| Ticks to tolerance, Tier 1/2/3 median (slowest): I1 324/381/525 (654), I2 1,594/1,902/2,390 (2,910), I3 396/574/630 (856) | PREDICTION §4 | Identical. The class agrees in all 845 runs, ticks to tolerance in 571 of 623 CONVERGED runs (fig2) |
| A quantity loop (1.19 a tick with prices frozen). The frame: the rationed machine desk keeps its coin and orders more; PREDICTION §3.5: power's coin falls while the goods desk's rises | frame §6.3; PREDICTION §3.5 | In the open-loop probe the loop grows about 1.2 a tick. The predictor's reading holds: in L2 C2L `p[good]*2` power's coin falls 9.87 → 3.64 by tick 11, and the goods desk's rises 24.9 → 29.6 |
| Mode A fails for I2 at 12/yr (mirror tick 2,558) and for L2, L3 at C2m | PREDICTION §4 | I2 at 2,427, L3 at 52/yr at 112 (mirror 115). L2 passes at 52 and 365/yr only because its genesis is a bit-exact fixed point, and its kick fails, as §4 said |
| 62 rate-scan and 40 map cells | PREDICTION §5.1–5.2 | 62/62, and 34 of the 35 map cells run. I2 (rates ×½, turnover ×1), PL 0.99967, is GO where LOCAL was called on a 60,000-tick cap. Its slowest run takes 34,139 ticks |
| Tick length, Tiers 1–2; O14 | PREDICTION §5.3, §7 | All but two cells: L3 C2L at 365/yr is 0/34, 2/42 (predicted 0/0), and I3 at 12/yr (below). O14 within a few per cent |

**I3 at 12 a year is where mirror and engine part.** PL from random directions (0.99132 at worst)
misses slow cones. PREDICTION §2.3 found one at the base (0.99992 a tick) and called the kick bar
borderline; the engine's base kick ends at 3.8e-3 against 1e-3. That cone is stable (at H = 5L the
kick passes), but at `land.power`=0.45 and `b.food`=0.54 kicks grow ×1.8e3 and ×1.1e8, unpredicted.

## 3. Mode A and mode B

**Mode A** (`markets run hold` at L; the largest gap in log where it passes):

| set | 12/yr | 52/yr | 365/yr |
|---|---|---|---|
| I0, I1, I3, G1 (C2m) | PASS, ≤ 3.0e-13 | PASS, ≤ 1.7e-15 | PASS, ≤ 3.6e-15 |
| I2 (C2m) | **FAIL** at tick 2,427; DIVERGED | PASS, 7.8e-16 | PASS, 1.2e-15 |
| L2 (C2m) | **FAIL** at 251 | PASS (genesis bit-exact); kick FAIL | PASS (bit-exact) |
| L3 (C2m) | **FAIL** at 252 | **FAIL** at 112; runaway at 559 | PASS (a fixed point to rounding) |
| L2, L3 (C2L) | PASS, ≤ 8.1e-14 | PASS, ≤ 2.2e-14 | PASS, ≤ 1.1e-13 |

**Mode B** at 52/yr ([verdicts.csv](results/markets/verdicts.csv),
[battery.csv](results/markets/battery.csv); fig1). Kick sets (1 ± 1e-9 on each price at every
target, H = L) all pass for I0–I3 and G1 (470 kicks, gain_tail ≤ 1.7e-5), and 8 of 8 for L2 and L3
at C2L; the ninth cannot be made, since its dated run dies. No run is ERROR or VACUOUS. Every
CONVERGED run ends within 1.03e-14 in log; Tier 3 at 10·L gives the same ticks to tolerance;
slack runs take 78–138 ticks. At C2m every L2 and L3 run DIVERGED. At C2L the same shapes fail
in both: JB(1.2) in Tier 2, and in Tier 3 r\*2, JB(2), RT(2), JA(0.5), JB(0.5), RT(0.5),
`p[engine]*0.5` and `p[power]*0.5`; L2 adds `p[engine]*0.8`, `p[good]*2` and `land.power`=1.2
at both times, L3 adds RC(2), `p[food]*2` and `b.food`=0.3 at both times. **How L2's
`p[good]*2` dies** (fig4): at tick 0 the goods desk doubles its bids and labour fills 0.54; at
tick 6 the engine makes 0.094 against the oracle's 2.51, and at tick 7 it keeps everything; power
makes 0 at tick 7 and the engine 0 at tick 8. Neither restarts (132 no-trade ticks on the
engine's market, 131 on power's), and both prices rise e^0.1 a tick to the runaway bound at
tick 139.

**Families** ([families.csv](results/markets/families.csv); fig5). Tick length, Tiers 1–2: runs
converged; median / slowest years to tolerance over the non-slack runs.

| set | I0 | I1 | I2 | I3 | G1 | L2 C2L | L3 C2L |
|---|---|---|---|---|---|---|---|
| 12/yr | 36/36; 22 / 28 | 68/68; **90 / 116** | **0/48**, mode A fails | **4/76**, base kick | 64/64; 69 / 100 | 48/48; 14 / 18 | 76/76; 77 / 125 |
| 52/yr | 36/36; 7.9 / 9.8 | 68/68; 6.8 / 10.8 | 48/48; 35 / 46 | 76/76; 10 / 12.5 | 64/64; 6.6 / 11.3 | 46/48; 5.3 / 7.5 | 75/76; 19 / 31 |
| 365/yr | 36/36; 6.4 / 8.6 | 68/68; 6.0 / 9.7 | 48/48; 22 / 32 | 76/76; 7.3 / 11.3 | 64/64; 6.1 / 10.8 | **33/48** | **2/76** |

The map (price rates × desk turnover) is GO in all of I1's cells; I2 is NO-GO at (1, ½), (2, ½)
and (2, 1); L2 and L3 are GO nowhere, LOCAL in the rates ×2 row; I3 is NO-GO at (2, ½), five cells
stopped by the time box, which also left families 5–9 unrun (stocks, joint, basin, history,
variants). Hash streams are identical on WSL and Windows.

## 4. Transients against the one-market probe (O14)

Reported, never scored: median / worst over the non-slack CONVERGED runs at 52/yr
([o14.csv](results/markets/o14.csv); fig3). I0 is REPORT §3's battery.

| set | T2 peak D̂ | T2 dead ticks | T2 lowest baskets/Y\* | T3 peak D̂ | T3 dead ticks | T3 lowest baskets/Y\* | T3 worst transfer shortfall |
|---|---|---|---|---|---|---|---|
| I0 | 346 / 857 | 0 / 12 | 0.77 / 0.54 | 1,290 / 2,321 | 35 / 111 | 0.40 / 0.135 | 770 |
| I1 | 310 / 1,288 | 0 / 84 | 0.74 / 0.28 | 1,142 / ∞ | 32 / 145 | 0.32 / **0** | 2,720 |
| I2 | 598 / 1,623 | 0 / 56 | 0.71 / 0.34 | 1,829 / 3,607 | 40 / 154 | 0.25 / 0.077 | 291 |
| I3 | 345 / 1,359 | 0 / 99 | 0.72 / 0.30 | 1,316 / 3,695 | 42 / 168 | 0.27 / 0.025 | 4,007 |
| G1 | 310 / 1,385 | 0 / 78 | 0.75 / 0.32 | 998 / 4,005 | 29 / 154 | 0.37 / 0.018 | 5,229 |
| L3 C2L | 539 / 2,720 | 0 / 157 | 0.59 / 0.086 | 1,181 / 4,062 | 46 / 1,670 | 0.31 / 0.017 | 20,002 |

Dated cost shocks ([depth.csv](results/markets/depth.csv); fig6): the equilibrium change ln(Y′/Y)
→ the trough of baskets in log (share of Y). The machine type's land ×2: I0 −0.128 → −2.00
(13.5%), I1 −0.113 → −3.15 (4.3%), G1 −0.031 → −3.16 (4.2%), I2 −0.055 → −2.09 (12%), I3
−0.044 → −2.27 (10%). `b.food` ×2: I1 −0.298 → −1.44 (24%), I3 −0.338 → −1.63 (20%), L3 C2L
−0.309 → −4.36 (1.3%).

**Does O14 get worse with more markets? Yes, in its tails; the medians stay at I0's** (Tier 3's
median peak D̂ 998–1,829 against 1,290, its median trough of baskets 0.25–0.37 against 0.40).
- **The worst runs are deeper.** Tier 2's worst trough of baskets halves (0.54 of Y\* to
  0.28–0.34), its worst dead ticks go from 12 to 56–99, and Tier 3's worst transfer shortfall is
  3.5–7 times I0's (not in I2). A machine-land shock of I0's size bottoms at 4% of Y in I1.
- **Ticks with no baskets at all**, in two I1 Tier-3 runs (`JB(0.5)` 3, `x*/2` 1): the machine desk
  keeps its whole stock, so manufactures and food make nothing, the loop's zero in miniature. Part
  of this is the rule: output is Leontief at the *planned* technique, so a missing machine service
  stops production even where labour clears (§5).
- **One small category caps consumption.** Fully automated manufactures (0.7% of P_s) binds 50–58%
  of short household ticks in I3 and 74–77% in L3. In I1, fully human care binds 29–33%, beside
  shelter (33–37%) and manufactures (24–32%): short of §7.11's bar, care binding most.

## 5. What the reviews found

Two reviews ran after the run. **Both say every verdict holds**; neither raised a blocker, and no
tally changes.
- **Measurement.** `d0ceaa6` rebuilt from `git archive` gives the run's binary byte for byte; the 27
  mode-A cells and 64 hard-case reruns, scored independently, match byte for byte. A 60-digit solve
  that reads no oracle code agrees with every target within 1.0e-15 in log and finds one sign
  change at each (decision 70). Every CSV rescored gives gaps within 1.8e-15. Kicks at the end
  states of all 623 CONVERGED runs, not only one per target, pass (gain_tail ≤ 1.8e-5). R13 holds.
  Minors: I3's 12/yr failure is a slow stable mode, and its instability is at two cost targets
  (§2); the engine's no-trade count is 132, not 131; the dead floor is knife-edge by 1 ulp at I2
  `N(2)`, tick 0, outside W.
- **Fidelity.** The rules match the frame's argument and L2's point checks by hand; the engine's
  class equals the mirror's in every run; no anchor or clamp; failing C2L runs reach exactly zero
  engine and power output. **Major:** the dependence on decision 67 is stated backwards. The
  loop's NO-GO does not depend on 67; the headline GO does (§6). Minors: the zero does not need
  keep-first (with a shared keep, all 17 failing runs still die); "no baskets from tick 8" is
  partly planned assignment (labour still clears; ex-post assignment would do the tasks by hand),
  triggered by labour rationing at tick 0; "from far away" needs scope (stationary coins and
  stocks, one task margin); I2's GO is weekly only; Q2's "must" rests on I2 alone.

**The corrected verdict** keeps every verdict and narrows the reading: many markets are GO at 52
ticks a year for loop-free flow economies, from displaced prices, technique and costs, with
stationary coins and stocks and one shared task margin. The loop is NO-GO for structural reasons
that no dial set tried removes.

## 6. What this means for Phase 2 proper

- **A11 is not met.** Mode B passes on all three verdict instances at the first dials tried. The
  loop fails mode B, but on a structure none of Phase 2's configurations needs (Appendix B, the
  wall, the open commons, 1750-like). So it constrains the instances rather than calling the
  fallback: no Phase 2 instance should carry a loop of non-storable produced inputs, and one that
  must brings A11 back.
- **The wall (after unit 1d) and the commons (after 1e)** start from these roles, this harness and
  C2m at 52/yr, and each adds an untested margin. The wall adds several labour markets and x\* = 1,
  where machines stop pinning the wage (G1's gap, where labour clearing sets w/p_m, is the nearest
  evidence, and GO). The commons adds land at zero rent: under `Saturate` an unsold price falls to
  the runaway bound (MARKETS-SPEC §1.1), which needs an answer without a clamp (R3).
- **Q2: the wage-first ordering carries, per market** (fig5; 62 cells as predicted). Labour ×½ is
  GO in I1 and I3 but NO-GO in the automated chain I2. A category's price must not outrun the wage
  it mostly pays: care ×4 is LOCAL in I1 and NO-GO in I3, where care buys 79% of the hours;
  manufactures ×4, with no hours, is GO. I2's margin is thin: good ×2 LOCAL, good ×4 and power ×2
  and ×4 NO-GO.
- **Decisions.** *60:* the GO tests many markets but one task margin; the desks' thresholds
  converge by exactly 0.951229 a tick, decoupled from prices. *61:* every stability number assumes
  continuous thresholds; cells need their own probe. *67: the headline GO depends on it.* Goods
  stay out of machine recipes, so I1–I3 have no loop; under PLAN §3.1's build bundle every
  instance with machines would carry a goods-and-machines loop, the structure that fails here.
  The loop's own NO-GO does not depend on 67: it comes from the frame's restrictions, M4's build
  recipes in the flow case (frame decision M2), non-storable services (δ = J = 1) and no entry.
  *70:* it holds everywhere; the zero is a dead state, not an equilibrium.
- **Q1: the loop is a structural gap, not a question of dials.** C2L makes the point locally
  stable, but no dial set tried near it is GO; the best is LOCAL (PREDICTION §5.4; the map). The
  fix is a stock the loop can draw on (Phase 3's durable machines, or a storable service) or entry
  for a desk at zero coin. Sharing the keep does not help.
- **Tick length.** At 12 a year I2 is unstable and I1 needs 90 years against 7 at 52: on
  Appendix B the monthly tick cost a factor of 3 in years, here 13.

**Recommendation.** Close P2.1 as it stands: many markets GO for loop-free economies at 52 a year,
the loop NO-GO, no fallback. Open Phase 2 proper after units 1d and 1e on loop-free wall and
commons instances, with these roles, this harness and C2m at 52 a year; its battery first runs the
families this probe left on I1–I3, stocks first. Carry the loop to Phase 3 as a structural gap,
and do not adopt C2L. Keep decision 67 for Phase 2 proper, since a veto would put every
multi-category instance behind Phase 3; keep 60, 61 and 70; keep the weekly tick.

## 7. Open questions

1. **The loop's fix.** Does a storable service or entry at zero coin close the absorbing zero?
2. **The unrun families**: stocks (a type's stock ×0.01 and ×10 above all), joint, basin, history,
   `Hold`, tilt 1, and five I3 map cells.
3. **Tick length.** Monthly ticks lose I2 and slow I1 thirteen-fold in years. Should the historical
   runs be weekly, or the dials be restated per tick length?
4. **Paths.** Does ex-post assignment remove the no-basket ticks? May one small category cap
   everyone's consumption?
5. **Measures.** H = L fails a slow stable mode (I3 at 12/yr), CERTIFY §15.1 (6) from the other
   side; PL from random directions misses slow cones.
6. **Not tested:** switching machine types (Q3), category inputs (Q4), durability and interest.
