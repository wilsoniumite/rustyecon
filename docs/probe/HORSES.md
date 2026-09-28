# HORSES: the stocks probe (P2.2a)

Dated 2026-09-28. Step P2.2 on branch `phase2-goods`, from `reboot`'s line at `92ba68e`. Frame:
`D:/rustyecon-p2g/frame/HORSES-SPEC.md` (HORSES-SPEC, sha256 `0e23e809…d5dd`). Build: P2.2.1
(`d636b76`), [HORSES-RULES.md](HORSES-RULES.md). Registration, before any mode-B run and naming
`d636b76`, clean: [results/horses/registration.md](results/horses/registration.md) (sha256
`2baaa34d…f4c6`). Design: `D:/rustyecon-goods/GOODS-CHAIN.md` (stage v2a.1, rule A; P1–P8).

**Conditions of every number below, unless its line says otherwise.** The harness `horses` from
`d636b76` (sources and tapes at `6b0ebcc` match the registration), release, WSL; the roles of §1;
dials C2g (labour, fodder, horse and horse-day rates 5.2 a year, land 0.1625, the good 2.6; s_K =
2δ, s_Km 0, band 4 weeks); ρ 0, J_b 1 tick, 52 ticks a year; L 75,000–258,000 ticks, Tier 3 and 3S
again at 10·L; tol 1e-3 in log on 18 observables; CONVERGED only if the target's 1e-9 kick set
decays too. Runs: `D:/rustyecon-p2g/runs/`; tables: [results/horses/](results/horses/)
(`D:/rustyecon-p2g/report/make_results.py`); plots: [figs/horses/](figs/horses/).

## 0. The verdict

| id | what it is | δ, ω | mode A | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | verdict |
|---|---|---|---|---|---|---|---|---|
| H1 | Appendix B's county under rule A | 10%, ½ | PASS | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| H2 | as H1 | 10%, 1 | PASS | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| H3 | as H1 | 8%, ½ | PASS | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| H4 | as H1 | 8%, 1 | PASS | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| F1–F4, F7–F10 | δ 4%; ω 0.85; fodder's rate 1.3 | | PASS | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | GO, each |
| F5, F6 | v1's base county (the demo's) | 8%, 10%; 0.85 | PASS | 20/20 | 24/24 | 23/23 (23/23) | 24/24 (24/24) | GO, funded targets |
| R1a | the stocks layer off (C2, planned) | δ = 1 a tick | PASS | 16/16 | 20/20 | 21/21 | 12/12 | P2.1's I0, bit for bit |

**GO for v2a.1, as registered, and narrower than the run first read it.** The probe's roles, with
the horse as a durable good that a maker breeds and a wet capacity desk holds, wears, feeds and
hires out by the day, find unit 1g's equilibrium of v2a.1's horse economy at all four verdict
instances. All 1,980 scored mode-B runs at 52 ticks a year converge (H1–H4 and ten families; 1,298
at L, 682 at 10·L), from displaced prices, technique, costs, horse stocks and coins; every one ends
within 2.4e-13 in log of the oracle, all 68 kick sets decay, and the class is the registered
mirror's in every run; with the stocks layer off it is P2.1's I0 bit for bit, by construction.
Beside it: weekly is the floor (12 a year fails everywhere, 24 at ω 1); a tenfold glut of horses
diverges everywhere through the horse price's unfloored fall, P8's idle runaway; P7 converges where
STUCK was registered. The reviews confirm the verdict and narrow its reading. Like for like, stocks
give about a quarter of the gain in a cost shock's trough that the run claimed, and cost 2.3–2.8
times the years, not 6–7. At ω 1 the GO passes through falls of 96–99.7% in the horse's price, the
mechanism that diverges at heads × 10. And P1's kick rates at ω 1 and P2's miss on the fast side:
the engine's slowest mode is exactly wear, 1 − δ a year, which price kicks do not measure.

## 1. What was built

- **Engine: nothing changed.** Wear is an upkeep burn tagged `Depreciation`, already allowed.
- **Roles** (`crates/agents/src/roles/stock/`; HORSES-RULES §3). The **maker** (M2) records its
  serving stock and breeds heads from its own horse-days, labour and pasture; it offers finished
  heads, which do not wear, under a 4-week band. The wet **capacity desk** (M3) buys fodder for each
  horse-day it runs and sells horse-days, whose price comes to rest at O + δ·p_K/κ; it targets K\* =
  B/(κO + δp_K), orders max(δK\* + s_K(K\* − H), 0) capped by its coin, and wears its phase-start
  holding. The **owner desk** (M1) serves R1a and P7. At δ = 1 a tick the maker and owner desk run
  `TypeDesk`'s and `GoodDesk`'s code; no pinned hash moves.
- **Harness, tapes, tests.** `probe::horses`, the binaries `horses` and `horses-tape`, the tapes
  `tapes/horses-{h1,h2,h3,h4,r1a,p7}.ron` from 1g's `ChainEconomy`, and 19 tests (865 in all).
- **Departures** (HORSES-RULES §8): horse-days keyed `traction` (`hday` sorts before `labour`); L
  longer than the frame quoted (land's τ under C2g is 374–389 ticks); the task's names, not
  STOCKS.md. The rest point holds within 1e-12; the trace diff agrees within 1e-12 on 8 of 12 runs,
  the others parting on the horse's volume, a small difference of large terms (explained).

## 2. The prediction and the result

| prediction (HORSES-SPEC §6, registered) | result |
|---|---|
| GO at H1–H4 and every family; no run ERROR, DIVERGED, DEAD, STUCK or ORBITING; speeds and transients (§6.3) | **Held**: 1,980/1,980, class the mirror's run for run ([fig1](figs/horses/fig1_ttol_engine_vs_mirror.png)); tier speeds within 0.7 y; 90 of 93 runs per H within 10% of the mirror's ticks (the other 3, one-tick stocks × 2, 16–21% slower: a genesis lot sells twice in the engine, a limit §6.1 states); Tier 2's worst peaks 1–2% high |
| P1 (H1, H2): mode A; 93/93; Tier-3 baskets ≥ 0.44; dead ≤ 45, ≤ 119; b′ 0.8 trough ≥ 0.6, ≤ 12 fodder dead; b′ 0.2 ≥ 0.7; kicks 0.89–0.92 a year | Held (0.444; 42, **120**; 0.645, 0.625 with 11, 7; 0.744, 0.726); kicks H1 0.891, **H2 0.845 out**, faster |
| P1 on H3, H4: kicks 0.91–0.94 | H3 0.912; **H4 0.877 out**, faster; troughs held |
| P2 (F1, F2): 93/93; no dead tick after b′ 0.2; kicks 0.975–0.995 | Held but the kicks: **0.945 and 0.919–0.932, out**, faster |
| P3 (ii) C2, s_K 0, ω 1 fails: 1.0014–1.0017 (10%), 1.0008–1.0010 (4%) a tick | **Fails**: 1.00133 (H2), 1.00118 (H4), 1.00077 (F2), just below; w × 1.05 ORBITING |
| P3 (iii) 12 a year fails, a 1e-9 kick ends DEAD or DIVERGED; 24 a year fails at H2 | **Fails** (1.0195, 1.0637); H2, H4 DEAD, H1 DIVERGED, **H3 VACUOUS** (grows to D̂ 927, orbits); **H4 at 24 a year fails too** (1.00038; predicted 0.99983) |
| P3 (iv) the order on the stock held: runaway at ticks 1,823–3,456 | **Fails**, 1.0126–1.0193; runaways at 1,979–3,180 after b′ 0.8, 2,872–**3,551** after a kick |
| P3 (v) finished heads offered in full: stable | **Held**, faster (g 0.822–0.945 a year against 0.894–0.992) |
| P5 and §6.2: stable at 52 and 365, unstable at 12; 52 and 365 within 2% | Held but P5 on the engine's g: H3 1.4%, H4 1.7%, **H1 2.6%, H2 2.4%**; g is 0.02–0.07 a year faster than PL everywhere |
| P7 (M1, C2, δ 4%, ω 1, b′ 0.2): trough ≈ 0.43; ≥ 250 dead, on the good; p × 100; STUCK | 0.4252; 322 on the good; p × 140.5, p_K × 433.9; **CONVERGED at tick 12,467**, the mirror's path |
| P8 (a 0.005, heads × 2): p_K < 1e-10 of target in 6 y; DIVERGED at 138; the rest within 7% | Tick 231 (4.44 y); DIVERGED at 138; goods 3.5%, land 0.03%, hours 5.9%, p 6.1%, **wage 7.2%** |
| Paths (§6.4) | **Held** to 2–3 digits ([fig2](figs/horses/fig2_cost_rise_b2.png), [fig3](figs/horses/fig3_expansion_b05.png), [fig4](figs/horses/fig4_glut_heads_x2.png)) |
| Not predicted | heads.capacity × 10 DIVERGES at all 14 instances (§3) |

## 3. Mode A and mode B

**Mode A** (hold at L: the largest gap in log, the base kick set's largest tail) and **tick length**
(Tiers 1–2: converged, median / slowest years; [ticklength.csv](results/horses/ticklength.csv)):

| set | 12 a year | 24 a year | 52 a year | 365 a year |
|---|---|---|---|---|
| H1 | **FAIL** at tick 709; 0/44 | 1.2e-14, 2.3e-5; 44/44, 48 / 83 | 2.4e-15, 4.0e-5; 44/44, 45 / 83 | 7.5e-14, 2.1e-4; 44/44, 45 / 83 |
| H2 | **FAIL** at 222; 0/44 | 1.9e-14, **kicks grow** (3.6e6); **0/44** | 3.7e-14, 6.8e-5; 44/44, 46 / 76 | 7.2e-14, 2.9e-4; 44/44, 43 / 75 |
| H3 | **FAIL** at 746; 0/44 | 1.7e-15, 2.5e-5; 44/44, 55 / 99 | 4.9e-15, 4.5e-5; 44/44, 51 / 100 | 6.2e-15, 3.3e-4; 44/44, 51 / 100 |
| H4 | **FAIL** at 213; 0/44 | 2.1e-15, **kicks grow** (3.4e5); **0/44** | 1.8e-14, 4.6e-5; 44/44, 56 / 95 | 4.1e-15, 3.3e-4; 44/44, 53 / 94 |

At 12 a year H1 and H3 end 8 DIVERGED and 36 ORBITING, H2 and H4 DEAD. At 365 a year the kick sits
at its rounding floor: tails of 3.0e-4 to 5.6e-4 pass, but say little beyond "did not grow".

**Mode B at 52 a year** ([verdicts.csv](results/horses/verdicts.csv),
[battery.csv](results/horses/battery.csv)): all 68 kick sets pass, the largest `gain_tail` 1.72e-4
against 1e-3 ([fig6](figs/horses/fig6_kick_envelopes.png)); no run is ERROR or VACUOUS; 10·L gives
the same ticks to tolerance. Eight Tier-3S runs at H1 have D̂₀ = ∞ (no horse volume while orders
stop), so the vacuity test cannot fire, but each peaks at D̂ ≥ 63 without that volume. **Controls,
P7, P8** ([fig7](figs/horses/fig7_negative_controls.png), [fig8](figs/horses/fig8_p7_p8.png);
[negative_controls.csv](results/horses/negative_controls.csv),
[m1_p7.csv](results/horses/m1_p7.csv), [p8.csv](results/horses/p8.csv)) are as §2; P7 under C2g
bottoms at 0.358 with 250 dead ticks, and P8's base kick set fails its tail bar at 40,000 ticks
(1.4e-3, g 0.983 a year), slow but stable.

**The stocks family** (every coin × 0.02 and × 0.1, every stock × 0.1 and × 10, at all 14 instances;
[stocks_family.csv](results/horses/stocks_family.csv)): 322 of 336 CONVERGE. The good desk's coin ×
0.02 drops baskets to 0.02 of Y\* for a tick; `own.maker` × 0.1 is the slowest (441–1,324 dead and
idle ticks, 89–253 years). **heads.capacity × 10 DIVERGES at every instance**: orders stop and the
horse's price falls at its full rate until it crosses the runaway bound at ticks 138–167 (at ω ½
after 111–135 ticks of dead labour).

## 4. Stocks and transients against P2.0 and P2.1 (O14)

The run set H1–H4 beside I0 alone ([transients.csv](results/horses/transients.csv)), which moves
stocks, C2g, ex-post assignment and the fodder market at once. The fidelity review ran the flow
county (R1a) in the same harness under each dial set and assignment (C2, planned, is I0 exactly).
Like for like ([o14_like_for_like.csv](results/horses/o14_like_for_like.csv);
[fig5](figs/horses/fig5_o14_like_for_like.png); worst, or median / worst; D̂ without horse volume):

| set | b × 2 trough, ln (of old Y) | T2 dead | T3 baskets / Y\* | T3 dead | T3 peak D̂ | T3 shortfall | T1–2 years |
|---|---|---|---|---|---|---|---|
| I0 (C2, planned) | −2.004 (0.135) | 12 | 0.40 / 0.135 | 111 | 2,321 | 770 | 7.9 / 9.8 |
| flow, C2, ex post | −1.143 (0.32) | 7 | 0.44 / 0.215 | 89 | 2,385 | 247 | 6.1 / 8.4 |
| **flow, C2g, ex post** | **−0.940 (0.39)** | **0** | **0.55 / 0.230** | **123** | **2,366** | **286** | **19.8 / 41.2** |
| H1 | −0.565 (0.57) | 0 | 0.669 / 0.444 | 42 | 3,700 | 20 | 45 / 83 |
| H2 | −0.598 (0.55) | 0 | 0.678 / 0.444 | 120 | 7,868 | 28 | 46 / 76 |
| H3 | −0.564 (0.57) | 0 | 0.669 / 0.444 | 42 | 3,745 | 20 | 51 / 100 |
| H4 | −0.597 (0.55) | 0 | 0.678 / 0.444 | 86 | 8,318 | 28 | 56 / 95 |

Of the 1.44 in log between I0's trough and H1's, ex-post assignment gives 0.86, C2g 0.20 and stocks
about 0.38, a quarter. Stocks lift Tier 3's floor from 0.230 to 0.444, which is N(2)'s in every flow
control and every H. They cut the worst dead ticks from 123 to 42 at ω ½, not at ω 1 (120 and 86,
fodder ticks after r × 2); on the good alone the control's r × 2 has 122 against H's 20–21. The
transfer shortfall falls tenfold; the worst peak D̂ rises 1.6 to 3.5 times. C2g alone makes the flow
county about three times slower (6.1 to 19.8 years, ex post), and stocks add 2.3–2.8 times more.

**Capital's time scale (D-G14; [paths.csv](results/horses/paths.csv)).** After b × 2, a 38% glut,
the installed stock comes within 5% of its target in 29.1, 39.8, 33.0 and 47.2 years (H1–H4) and 40
and 78 at δ 4%, where the paper's zero builds take 3.1, 3.9 and 8.0; it falls to 0.82–0.90 of target
and overshoots to 1.03–1.11 by years 30–40. After b × 0.5, which the paper fills in one tick, it
takes 34–44 years and overshoots by 39% (ω ½) and 80–83% (ω 1); the quasi-rent is +0.4% (ω ½) and
+6.2% (ω 1) three ticks on, +42% to +58% at 1/δ. Neither lifting criterion is met.

**The glut and the idle horse market.** After heads × 2 the hour price falls to 0.98–1.08 of the
running cost O, orders stop for 0.06–3.2 years (8.8 at F2), finished heads pile up to 1.25–2.30
times rest, and at F1 labour clears below half its volume for 550 ticks (the registered
technological unemployment) and converges. Whenever orders stop the horse's price falls without a
floor: at ω 1 to 0.3–1.7% of target after r × 2, b × 2 and N(2), 3.6–4.1% after heads × 2, 0.1% at
F2; at ω 0.85 (F3–F6) to 1.4–3.6%; at ω ½ it stays above 10%. The maker, at tilt 0, keeps breeding
at a cost far above that price. These runs converge because the fall stops three decades or more
above the runaway bound; at heads × 10 it does not.

## 5. What the reviews found, and the corrected verdict

Two reviews ran after the run. **Both say the verdict holds**, and neither changes a class.
- **Measurement.** A Windows rebuild reran 58 runs and two kick sets byte for byte. A 60-digit solve
  reading no oracle code matches the 1,190 targets within 4.4e-16 in log (R1b); its own rescorer
  reproduces 30 runs tick by tick; wear conserves within 4e-16 a tick, money drifts at most 9.5e-15,
  and R13 holds. Minors: the count is **1,980, not the run's 2,548** (H1–H4 counted twice); R1a
  holds **by construction**, so its tests check plumbing and the capacity desk has no R1, its
  evidence being mode A, the rest point, the trace diff and the rescoring; the 365-a-year kicks sit
  at their floor; D̂₀ is infinite for some 3S runs (§3).
- **Fidelity.** The roles match GOODS-CHAIN §3.1 line by line and the rest-point proof matches the
  code. The verdict does not lean on D-G6 or C2g: under planned assignment, and under C2, H1–H4
  converge 93/93 (40,000 ticks, no kicks); s_Km 0 is right (at 0.2 and 0.4 H2's b × 0.5 orbits).
  **Majors:** the O14 comparison is confounded (§4); the ω 1 GO rests on the unfloored fall that
  ends heads × 10 and P8, which a remedy for GOODS-CHAIN's open question 4 would change, perhaps
  with the classes; v2a.1 is a partial base for v2a.1b (§6). **Minors:** P1's and P2's rate
  intervals went unscored, and the slowest mode is exactly 1 − δ a year (1e-4 kicks decay at 0.9600
  a year at F1 and F2 over years 100–500; the maker's own stock at 0.9000 at H1, H2), with slower
  readings (0.989 at F1, years 20–100) transients; M6's check exempts the capacity and owner desks
  from every good they buy (`cast.rs:191`), so a tape with storable fodder only the capacity desk
  buys loads, and its fodder desk piles up 50 times its stock in 520 ticks.

One fidelity reading goes too far, "no effect on the worst baskets": like for like the floor rises
from 0.230 to 0.444, where N(2), a nominal shock, stops it. **The corrected verdict** is §0's: GO
for v2a.1 at H1–H4 (C2g, 52 a year; rule A's maker, ρ 0, J_b 1), with its narrowings.

## 6. What this means

**For v2a.1b, loops with plants.** The horse held as a stock carries over: its wear, M3's order rule
at 2δ, the maker's band, the fodder market, and the harness with Tier 3S, the idle horse market and
windows from the slowest mode. What v2a.1b needs is not here: **the plant** (CAPACITY's y =
K^(1−θ)·z^θ, a plant each loop desk holds and never trades, its nesting at θ 1 and at a fixed plant,
and how it composes with an M3 holding, which CAPACITY lists as untested); a damper on **the
pass-through**, since the capacity desk passes a fodder shortfall one for one into horse-days (H2
after r × 2: 120 fodder ticks, 38 on horse-days, 21 on the good), the amplifier LOOPS.md names; **a
funded rule-B county** (O41); **the chain's maker**, one from bought inputs, run only as P8; **a
remedy for the idle horse market** (O47) before a loop adds its absorbing zero; and **M6 narrowed**
before fodder may be stored (O48).

**For the demo's second pass (O27, D-G12).** v2a.1 is GO on the demo's county (F5, F6, ω 0.85) on
its funded targets: it can enter the demo on these roles at C2g, the clock held at 52 (O39), once
D-G12 is ruled. With it go: v1's b × 2 is unfunded on the base county (O43), so the demo's
county-by-county battery must check funding; a cost shock takes 30–100 years to settle, unscored
(D-G14); and the horse's price can fall to 3% of its target, so a lens on it shows an idle market's
fall, not a valuation.

**For Phase 2 proper.** The loop-free wall and commons instances are unaffected and still open
first. For the 1750-like instance as a goods chain at ρ 0 (D-G1, D-G11), P2.2a is the first engine
evidence that machine stocks are GO at weekly ticks, at three costs: L of 75,000–258,000 ticks, 4 to
13 times I0's, with every kick horizon; weekly as the floor (O45; decision 121); and a
horse-to-steam switch idles the old machine's market by construction, which is P8's runaway, so the
idle-market remedy (O47) is on that instance's critical path.

**Recommendation.** Close P2.2a as GO for v2a.1 at C2g and 52 ticks a year, read as §0 narrows it,
and take the frame's decisions 220–233 with the report's 234–239.
- **Before P2.2b's frame:** GOODS-CHAIN §6's mirror step, with CAPACITY's plant on rule B's loop
  desks composed with M3's horse holding and its predictions registered; a funded rule-B county
  (O41); a remedy for the idle machine market, chosen by a mirror scan and run in the engine on
  heads × 10, P8 and the ω 1 battery (O47; the candidate is the maker's reservation at replacement
  cost, since P2.1's one-sided fix does not exist); M6 narrowed to the durable good (O48); and a
  like-for-like flow control in the frame.
- **The demo's second pass** may start stage v2a.1 on funded counties after your ruling on D-G12.
- **Phase 2 proper** opens as planned on the loop-free instances; the goods chain's 1750-like
  instance waits for O47.

## 7. Open questions

1. **The idle machine market** (O47). Which remedy stops the horse price short of the bound without
   a clamp (R3), and does it change ω 1's classes?
2. **Loops** (v2a.1b). Does CAPACITY's plant damp rule B's loop through a Leontief capacity desk?
3. **Makers from bought inputs.** CHAIN's horse and S1 without its pumping loop (GOODS-CHAIN §6 step
   4, not framed here) are untested beyond P8. Do they need C2g, which rule A does not?
4. **Measures.** Should kick sets carry stock and coin kicks, horizons from 1 − δ, and avoid the
   365-a-year floor? **Capital's time** (D-G14): can the demo show 30–80 years to settle?
5. **Not tested:** ρ > 0, J_b > 1, storable running goods, several machine goods, categories.
