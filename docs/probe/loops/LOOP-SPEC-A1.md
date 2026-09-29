# LOOP-SPEC-A1: the loop step registered again, with the engine's genesis carry

Dated 2026-09-29. Label `fix-report`, step L0.8 on branch `phase2-loops`. It amends
`LOOP-SPEC.md` (sha256 `e16e0be2ab521ed28c5d79811fabbb250ca5bbb107bd00f656d45d0993fb89ad`;
registered in `D:/rustyecon-p2l/loop-mirror/`, in the repository as
`docs/probe/loops/LOOP-SPEC.md`), which stays as registered. This file's sha256 is in
`SHA256SUMS` beside it and in `docs/probe/results/loops/registration.md`. **§A1.5 replaces
LOOP-SPEC §8 for P2.2b's scoring.** The rest of LOOP-SPEC stands unless a section here says
otherwise. No P2.2b engine code or run exists: `crates/agents`, `crates/probe` and
`crates/engine` hold no plant code, and `tapes/` holds no rule-B tape.

**Why.** The mirror review (2026-09-29) found LOOP-SPEC's mirror without the engine's genesis
carry, which STATE's O57 asks of a mirror that registers for the engine. With the carry no class
moves at any verdict, family or flow-control instance, but registered values move past §8's own
tolerances (§A1.3), so a faithful engine would have failed the registration. The review's minor
findings are answered here too (§A1.2 item 5, §A1.5, §A1.6).

**Conditions of every number below, unless its line says otherwise:** LOOP-SPEC's header: chain8
under rule B, C2g, 52 ticks a year, ρ 0, J_b 1, ex post, ψ 0.25, plants on the fodder desk, the
capacity desk and the maker at θ 0.8 and plant δ 10% a year, s1, s_Kp = 2δ_p; L 202,000–232,000
ticks, Tiers 3 and 3S again at 10·L; tol 1e-3 in log; HORSES-RULES §5's classes. The map is
`lm_carry.py` (§A1.1); "registered" means LOOP-SPEC as first registered.

## A1.1 The carry, as the engine must build it

The engine gives a one-tick good held at genesis one more tick of life than a unit made in a
tick (HORSES-RULES §6.4; P2.2a's trace diff). So:

- **Sellers.** A genesis lot that does not sell at tick 0 is offered again at tick 1, beside tick
  0's output: the good desk's good, the fodder desk's fodder, and the capacity desk's horse-days
  (in the flow control, the horse-day desk's). Storable fodder (LF4, LN5) keeps what it does not
  sell anyway, so it carries nothing.
- **Buyers.** A genesis lot bought at tick 0 and not used is used at tick 1: the good desk's
  horse-days; the capacity desk's fodder, its running and plant lines together; the maker's
  fodder, its build and plant lines together; the fodder desk's horse-days, running and plant
  lines together; in the flow control, the horse-day desk's fodder. Labour and land are `Instant`
  and carry nothing.
- **A planted desk** (STATE decision 273) takes carried goods into its bundles as it takes any it
  holds (LOOP-SPEC §2.2): B = max_scale over the bundle's goods held, B·z/(z + s·I) to running
  and B·I/(z + s·I) to the build. Since labour does not carry, a carried good adds bundles only
  as far as the labour bought that tick allows.

The first two are what P2.2a's engine already does; the third is the one new convention. The
engine's plant build must follow it, or E0 parts at tick 1 (STATE O68). At rest the carry is
rounding, at most 1.1e-15 of a flow. Displaced, it is not small: after the genesis tick it reaches
0.60 of the capacity desk's and the maker's fodder flow (w × 0.5, r × 0.5), 0.96 of the fodder
desk's horse-day flow (r × 0.5), and 9 times a flow for a × 10 stock of a one-tick good.

`make_lm_carry.py` writes `lm_carry.py` from the registered `lm_mirror.py`, which it reads and
never edits. A desk with no carry runs `lm_mirror`'s lines unchanged.

## A1.2 The checks of the carry map, and of the plant's dynamic lines

Before any run was scored (`lmc_nest.py` → `lmc_nest.out`; `lmd_check.py` → `lmd_check.out`):

1. **With every new layer off** (the loop cut, no plant, no store, on P2.2a's instances) the carry
   map is the trace diff's `i_carry.py` bit for bit over 6,000 ticks in 22 runs: H1 and H2
   heads × 10, w × 2, pK × 0.5, a 1e-9 kick, P8 heads × 2, H2 r × 2, IG × 2, Zs × 2 and If × 0.5,
   each at ψ 0 and 0.25, and M1's P7 on hold and w × 2. `i_carry.py` met the engine within
   1.6e-12 on H2 r × 2 (L0.6).
2. **At θ 1 the plant layer is the plant-free run bit for bit**, with the carry, in 94 runs over
   6,000 ticks: rule B and B-cut, plants on every combination of desks, 11 displacements with the
   one-tick-good stocks among them, and the flow control's six.
3. **From tick 2 the carry map is `lm_mirror.py` bit for bit** over 2,000 ticks, at LB1, LB3, LW1,
   LF4, LN2 and LC1 on w × 2, r × 2, Zs × 2, If × 10 and heads × 10 (29 runs).
4. **Mode A.** Against `lm_mirror.py` the largest gap in log over 2,000 ticks is at most 1.05e-14
   at every instance but LN6, whose mode A dies either way.
5. **The plant's dynamic lines, a second time** (STATE decision 280; the review found that only
   the rest point checked them). `lm_drs2.py` is a second rule-B map with the plant live, written
   from §2.2–§2.5's text on the independent `lm_drs.py`, not from `lm_mirror`'s plant lines: its
   order, wear, cost ratios, targets, coin caps and bundle split are the spec's words. Against
   `lm_mirror.py` (no carry) from the same displaced genesis, at LB1, LB3, LF1, LF2, LC1, LN2, LN4
   and LF3 on 14 displacements each (110 runs), the largest gap in log is 4.8e-12 over 2,000
   ticks and 1.4e-11 over 6,000, where the map against itself one ulp apart parts by up to 6.4e-11.
   No run passes 1e-9.

## A1.3 What the carry moved

The whole battery ran again on `lm_carry.py`: 3,794 runs at 25 instances, the kick sets, the
P8-like and tick-length runs, and the tables (`run_all.sh`, about 70 minutes on 47 cores).
`lmc_compare.py` sets each run beside its registered twin.

- **No class moves at any verdict, family or flow-control instance.** Every run of LB1–LB3,
  LF1, LF2, LF4, LF5, LF7, LF8, LC1 and LW1–LW3 has its registered class. Each tier's median and
  largest years move by at most 0.1 year (§7.6's pooled Tier 1–2 median by at most 0.5), the
  kick sets' g by at most 0.0009 a year, §7.2's cost shocks by at most 0.010 in any value, and
  §7.3's glut not at all.
- **Registered values move past §8's tolerances in 9–12 runs per verdict instance at L** (and
  their twins at 10·L). Six are the one-tick goods' stock displacements (the good's, fodder's and
  the horse-days' stocks × 2 and × 10), whose unsold half now carries: their ticks to tolerance
  rise 11–38% and their lowest horse price falls 6–46% (LB3 Zs × 10: 0.684 → 0.369). The others
  are r × 2 (the fodder low 0.376 → 0.409 at LB1), JB(0.5) and N(0.5) (lows up 5–10%), and at LB3
  JA(0.5), pK × 1.05 (ticks −17%) and s × 1.2 (−11%).
- **The negative controls flip where they are unstable.** 29 runs change class, all among the
  controls: LN7 19 (between DIVERGED and ORBITING), LN1 5, LN9 3, LN5 1, LF6 1. Every control keeps
  its verdict.

## A1.4 §7 restated with the carry

### The verdict table (LOOP-SPEC §0, restated)

| id | mode A (largest gap) | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | stocks family | verdict |
|---|---|---|---|---|---|---|---|
| **LB1** | PASS (6.7e-16) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| **LB2** | PASS (1.0e-15) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| **LB3** | PASS (4.5e-14) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | **GO** |
| LF1, LF2, LF4, LF5, LF7, LF8, LC1 | PASS (≤ 5.7e-14) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | GO, each |
| LW1, LW2, LW3 | PASS (≤ 6.7e-16) | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | GO, each |
| LF3 (ψ 0) | PASS | 20/20 | 24/24 | 24/25 (24/25) | 27/28 (27/28) | 25/28 | NO-GO |
| LF6 (capacity plant at use) | PASS | 11/20 | 4/24 | 3/25 (3/25) | 5/28 (5/28) | 5/28 | NO-GO |
| LW0 (flow, no plants) | PASS | 18/18 | 22/22 | 15/23 (15/23) | 14/16 (14/16) | 12/16 | NO-GO |
| LN1 (no plant) | PASS | 0/20 | 0/24 | 0/25 | 0/22 | 0/22 | NO-GO |
| LN2 (no maker plant) | PASS | 20/20 | 22/24 | 9/25 | 21/26 | 14/26 | NO-GO |
| LN3 (fodder plant alone) | PASS | 0/20 | 0/24 | 0/25 | 0/24 | 0/24 | NO-GO |
| LN4 (no fodder plant) | PASS | 20/20 | 24/24 | 25/25 | 26/26 | 25/26 | GO in the tiers |
| LN5 (4-week store) | PASS | 0/20 | 0/24 | 0/25 | 0/28 | 0/28 | NO-GO |
| LN6 (whole-gap orders) | DEAD | 0/20 | 0/24 | 0/25 | 0/28 | 0/28 | NO-GO |
| LN7 (B-cut, no plant) | PASS | 0/20 | 0/24 | 0/25 | 0/22 | 1/22 | NO-GO |
| LN8 (B-cut, no plant, fodder 1.3) | PASS | 20/20 | 18/24 | 14/25 | 16/22 | 17/22 | NO-GO |
| LN9 (no plant, fodder 1.3) | PASS | 16/20 | 8/24 | 6/25 | 10/22 | 7/22 | NO-GO |

Registered values that differ: LB3's mode-A gap (3.7e-14), LF6 Tier 1 (10/20), LN1's stocks
family (1/22), LN7 Tier 3 (1/25), LN9 Tier 2 (9/24) and stocks family (8/22).

### §7.1 restated: speeds, troughs, dead ticks by market and the horse price, per tier

"Years" is the median and the largest over converged runs. Dead ticks are the largest count in
the tier for each market, fodder, horse-days, the good and land, which are scored (STATE
decision 274); labour's is reported, not scored. The union of all markets is reported in the
tables' source (`lmc_tables.out`). "Horse price" is the horse's lowest over its target.

| id | tier | years | lowest baskets | dead: fodder, horse-days, good, land | labour (reported) | horse price |
|---|---|---|---|---|---|---|
| LB1 | 1 | 22.2 / 35.1 | 0.895 | 0, 0, 0, 0 | 0 | 0.726 |
| | 2 | 30.1 / 43.0 | 0.656 | 5, 0, 0, 0 | 0 | 0.219 |
| | 3 | 37.0 / 64.6 | 0.255 | 175, 50, 18, 18 | 167 | 0.149 |
| | 3S | 32.7 / 46.6 | 0.500 | 9, 37, 0, 0 | 0 | 0.218 |
| LB2 | 1 | 18.0 / 33.5 | 0.897 | 0, 0, 0, 0 | 0 | 0.767 |
| | 2 | 27.8 / 38.0 | 0.659 | 5, 0, 0, 0 | 0 | 0.224 |
| | 3 | 32.1 / 54.0 | 0.260 | 263, 50, 18, 18 | 175 | 0.150 |
| | 3S | 30.2 / 43.4 | 0.500 | 9, 33, 0, 0 | 0 | 0.214 |
| LB3 | 1 | 34.7 / 58.3 | 0.891 | 0, 0, 0, 0 | 0 | 0.600 |
| | 2 | 48.8 / 73.2 | 0.650 | 5, 0, 0, 0 | 0 | 0.202 |
| | 3 | 60.8 / 135.4 | 0.245 | 173, 51, 21, 19 | 148 | 0.149 |
| | 3S | 57.6 / 78.6 | 0.500 | 9, 55, 0, 0 | 0 | 0.180 |
| LF1 | 1 | 20.8 / 44.4 | 0.923 | 0, 0, 0, 0 | 0 | 0.757 |
| | 2 | 34.0 / 54.5 | 0.733 | 0, 0, 0, 0 | 0 | 0.225 |
| | 3 | 41.3 / 90.5 | 0.373 | 215, 25, 19, 18 | 237 | 0.129 |
| | 3S | 37.5 / 60.3 | 0.500 | 3, 31, 0, 0 | 0 | 0.215 |
| LF2 | 1 | 19.2 / 50.0 | 0.895 | 0, 0, 0, 0 | 0 | 0.733 |
| | 2 | 36.4 / 63.0 | 0.657 | 5, 0, 0, 0 | 0 | 0.222 |
| | 3 | 46.8 / 79.7 | 0.256 | 158, 50, 18, 18 | 207 | 0.149 |
| | 3S | 44.5 / 71.3 | 0.500 | 9, 34, 0, 0 | 0 | 0.216 |
| LF4 | 1 | 19.1 / 36.9 | 0.937 | 0, 0, 0, 0 | 0 | 0.774 |
| | 2 | 29.8 / 44.2 | 0.763 | 0, 0, 0, 0 | 0 | 0.288 |
| | 3 | 35.3 / 74.8 | 0.411 | 156, 24, 18, 18 | 179 | 0.129 |
| | 3S | 32.0 / 47.9 | 0.500 | 0, 23, 0, 0 | 0 | 0.171 |
| LF5 | 1 | 22.5 / 34.3 | 0.870 | 0, 0, 0, 0 | 0 | 0.675 |
| | 2 | 31.7 / 44.3 | 0.611 | 12, 8, 0, 0 | 0 | 0.245 |
| | 3 | 36.3 / 51.6 | 0.222 | 249, 79, 22, 18 | 170 | 0.165 |
| | 3S | 32.7 / 48.6 | 0.500 | 13, 78, 0, 0 | 0 | 0.188 |
| LF7 | 1 | 19.2 / 38.9 | 0.925 | 0, 0, 0, 0 | 0 | 0.789 |
| | 2 | 29.5 / 50.0 | 0.735 | 0, 0, 0, 0 | 0 | 0.286 |
| | 3 | 37.2 / 68.0 | 0.375 | 196, 24, 19, 18 | 231 | 0.130 |
| | 3S | 35.2 / 55.0 | 0.500 | 3, 30, 0, 0 | 0 | 0.208 |
| LF8 | 1 | 38.8 / 70.3 | 0.920 | 0, 0, 0, 0 | 0 | 0.661 |
| | 2 | 58.8 / 88.5 | 0.730 | 0, 0, 0, 0 | 0 | 0.215 |
| | 3 | 72.5 / 204.5 | 0.369 | 196, 26, 19, 19 | 230 | 0.132 |
| | 3S | 64.7 / 102.3 | 0.500 | 3, 50, 0, 0 | 0 | 0.168 |
| LC1 | 1 | 19.4 / 34.0 | 0.936 | 0, 0, 0, 0 | 0 | 0.722 |
| | 2 | 28.8 / 41.5 | 0.760 | 0, 0, 0, 0 | 0 | 0.215 |
| | 3 | 34.7 / 61.4 | 0.411 | 169, 27, 19, 18 | 163 | 0.153 |
| | 3S | 29.6 / 45.0 | 0.500 | 0, 26, 0, 0 | 0 | 0.186 |
| LW1 | 1 | 14.5 / 33.8 | 0.897 | 0, 0, 0, 0 | 0 | — |
| | 2 | 21.4 / 41.3 | 0.662 | 6, 0, 0, 0 | 0 | — |
| | 3 | 28.0 / 47.9 | 0.262 | 208, 54, 21, 20 | 116 | — |
| | 3S | 28.9 / 45.9 | 0.500 | 10, 5, 0, 0 | 0 | — |
| LW2 | 1 | 14.6 / 34.2 | 0.899 | 0, 0, 0, 0 | 0 | — |
| | 2 | 21.7 / 41.7 | 0.665 | 6, 0, 0, 0 | 0 | — |
| | 3 | 28.5 / 48.4 | 0.268 | 209, 55, 21, 20 | 114 | — |
| | 3S | 29.3 / 46.4 | 0.500 | 10, 5, 0, 0 | 0 | — |
| LW3 | 1 | 14.2 / 33.1 | 0.893 | 0, 0, 0, 0 | 0 | — |
| | 2 | 20.7 / 40.4 | 0.657 | 4, 0, 0, 0 | 0 | — |
| | 3 | 27.0 / 47.1 | 0.253 | 209, 63, 22, 20 | 120 | — |
| | 3S | 28.0 / 45.0 | 0.500 | 10, 5, 0, 0 | 0 | — |

The same numbers hold at 10·L. LOOP-SPEC §7.1's notes on N(2), Tier 3S's baskets and the stocks
family stand (at LB1 the workers' coin × 0.02 still gives the most dead ticks, and heads × 10 the
largest peak D̂).

### §7.2 restated: the cost shocks at genesis

"Trough" is the goods' lowest over the new Y\*; the 5% columns are the years from which the
installed heads, the capacity plant and the fodder plant stay within 5% of their targets.

| id | b × 2: trough; years to tol; heads, capacity plant, fodder plant to 5%; horse price low | b × 0.5: the same |
|---|---|---|
| LB1 | 0.704; 46.3 y; 10.1, 9.4, 8.6 y; 0.194 | 0.859; 64.6 y; 14.4, 14.2, 13.7 y; 0.704 |
| LB2 | 0.716; 43.4 y; 10.6, 11.2, 9.1 y; 0.194 | 0.863; 54.0 y; 14.1, 14.1, 13.8 y; 0.791 |
| LB3 | 0.678; 66.6 y; 12.5, 24.3, 8.3 y; 0.196 | 0.850; 135.4 y; 24.0, 34.8, 31.7 y; 0.390 |
| LF1 | 0.825; 52.5 y; 13.0, 12.0, 9.1 y | 0.853; 90.5 y; 17.6, 17.3, 23.4 y |
| LF2 | 0.706; 76.3 y; 26.5, 34.4, 24.2 y | 0.857; 48.2 y; 5.7, 9.8, 18.4 y |
| LF4 | 0.964; 46.8 y; 9.4, 8.6, 8.0 y | 0.859; 74.8 y; 14.6, 14.3, 18.9 y |
| LF5 | 0.658; 47.0 y; 11.0, 9.9, 9.4 y | 0.859; 49.0 y; 15.0, 14.8, 14.7 y |
| LF7 | 0.835; 48.3 y; 13.5, 13.7, 9.6 y | 0.857; 68.0 y; 17.2, 17.2, 16.1 y |
| LF8 | 0.804; 72.5 y; 23.3, 24.1, 8.0 y | 0.845; 204.5 y; 42.6, 54.3, 40.6 y |
| LC1 | 0.809; 45.3 y; 9.8, 9.0, 8.1 y | 0.867; 61.4 y; 13.7, 13.5, 12.9 y |
| LW1 | 0.679; 38.8 y; —, 12.4, 8.7 y | 0.865; 31.5 y; —, 4.2, 12.6 y |
| LW2 | 0.688; 39.2 y; —, 12.8, 9.1 y | 0.871; 31.9 y; —, 4.4, 12.8 y |
| LW3 | 0.659; 37.8 y; —, 11.5, 8.0 y | 0.854; 31.0 y; —, 3.6, 12.1 y |

### §7.3 and §7.4: unchanged

The carry moves none of §7.3's glut numbers (heads × 2 and × 10 at LB1–LB3, ψ 0.25 and 0) and
none of the tick-length results. heads × 0.1 at LB1 converges in 46.1 years with 194 dead ticks
(193 on horse-days, 89 on fodder, 30 on the good), the tasks' horse-days falling to 0.032 of
target (registered 0.033); at LB3, 260 dead ticks in 78.3 years.

### §7.5 restated: the pass-through (r × 2 at genesis)

| id | fodder | horse-days | good | land | labour (reported) | fodder low | tasks' horse-days low | class |
|---|---|---|---|---|---|---|---|---|
| **LB1** | 175 | **0** | 11 | 12 | 167 | 0.409 | 0.509 | CONVERGED, 50.3 y |
| LB2 | 263 | 8 | 10 | 11 | 175 | 0.413 | 0.487 | CONVERGED, 47.8 y |
| LB3 | 173 | 0 | 12 | 12 | 148 | 0.402 | 0.527 | CONVERGED, 78.9 y |
| LF4 (13-week store) | 156 | 0 | 11 | 11 | 171 | 0.373 | 0.577 | CONVERGED |
| LC1 (loop cut, plants) | 169 | 0 | 11 | 12 | 163 | 0.420 | 0.526 | CONVERGED |
| LN4 (no fodder plant) | 301 | 74 | 39 | 11 | 0 | 0.173 | 0.310 | CONVERGED |
| LN7 (loop cut, no plant) | 162 | **146** | 93 | 12 | 0 | 0.234 | 0.270 | ORBITING |
| LN8 (loop cut, no plant, fodder 1.3) | 197 | **148** | 80 | 12 | 0 | 0.220 | 0.185 | CONVERGED |
| LW1 (flow control) | 208 | 0 | 11 | 12 | 116 | 0.402 | 0.516 | CONVERGED |
| LN1 (no plant) | 398 | 2,700 | 82 | 10 | 0 | 0.002 | 0.002 | DEAD (registered: DIVERGED at 287) |

LOOP-SPEC §7.5's reading stands: the capacity plant takes the pass-through into horse-days from
146–148 dead ticks to 0–8, and fodder's 156–263 dead ticks are the county's (the flow control has
208).

### §7.6 restated: O14 like for like

| set | b × 2 trough, ln (of old Y) | T2 dead (union) | T3 baskets / Y\* (median / lowest) | T3 dead (union) | T3 peak D̂ | T1–2 years (median / worst) |
|---|---|---|---|---|---|---|
| flow control, no plants (LW0) | −2.358 (0.09), DIVERGED | 215 | 0.435 / 0.061 | 214 | 3,379 | 20.3 / 39.0 |
| **flow control, plants (LW1)** | **−0.389 (0.68)** | 6 | 0.675 / 0.262 | 212 | 2,590 | **19.5 / 41.3** |
| LW2 / LW3 | −0.375 / −0.418 | 6 / 4 | 0.684 / 0.268; 0.659 / 0.253 | 210 / 217 | 2,607 / 2,555 | 19.5 / 41.7; 19.3 / 40.4 |
| **LB1** (stocks, plants) | **−0.353 (0.70)** | 5 | 0.702 / 0.255 | 244 | 12,029 | **26.5 / 43.0** |
| LB2 / LB3 | −0.336 / −0.390 | 5 / 5 | 0.714 / 0.260; 0.678 / 0.245 | 322 / 232 | 9,440 / 20,677 | 24.1 / 38.0; 42.8 / 73.2 |
| LF1 (θ 0.7) | −0.194 (0.82) | 0 | 0.737 / 0.373 | 315 | 9,746 | 29.5 / 54.5 |
| LF4 (store) | −0.039 (0.96) | 0 | 0.781 / 0.411 | 266 | 11,760 | 23.4 / 44.2 |
| LC1 (loop cut) | −0.214 (0.81) | 0 | 0.757 / 0.411 | 237 | 11,761 | 25.0 / 41.5 |

Like for like, stocks lift the b × 2 trough by 0.036 in log at δ 8% (0.039 at 10%, 0.028 at 4%)
and take 1.36 times the flow control's median years at δ 8% (1.24 at 10%, 2.22 at 4%).

### §7.7 restated: the negative controls, class by class

At L, from the carry battery (the tiers' converged counts are the verdict table's):

| id | Tier 1 | Tier 2 | Tier 3 | Tier 3S | stocks family |
|---|---|---|---|---|---|
| LN1 | DEAD 20 | DEAD 24 | DEAD 14, DIVERGED 11 | DEAD 18, DIVERGED 4 | DEAD 15, DIVERGED 7 |
| LN2 | CONVERGED 20 | CONVERGED 22, ORBITING 2 | CONVERGED 9, ORBITING 16 | CONVERGED 21, ORBITING 5 | CONVERGED 14, ORBITING 12 |
| LN3 | DEAD 20 | DEAD 24 | DEAD 23, ORBITING 2 (b × 0.5 at genesis and dated) | DEAD 24 | DEAD 24 |
| LN4 | CONVERGED 20 | CONVERGED 24 | CONVERGED 25 | CONVERGED 26 | CONVERGED 25, DIVERGED 1 (the fodder desk's coin × 0.02) |
| LN5 | ORBITING 20 | ORBITING 24 | ORBITING 25 | ORBITING 28 | ORBITING 28 |
| LN6 | DEAD 20 | DEAD 24 | DEAD 25 | DEAD 28 | DEAD 28 (mode A DEAD) |
| LN7 | ORBITING 15, DIVERGED 5 | ORBITING 16, DIVERGED 8 | ORBITING 21, DIVERGED 4 | ORBITING 16, DIVERGED 6 | ORBITING 20, DIVERGED 1, CONVERGED 1 |
| LN8 | CONVERGED 20 | CONVERGED 18, ORBITING 6 | CONVERGED 14, ORBITING 11 | CONVERGED 16, ORBITING 6 | CONVERGED 17, ORBITING 5 |
| LN9 | CONVERGED 16, DEAD 4 | CONVERGED 8, DEAD 15, DIVERGED 1 | CONVERGED 6, DEAD 9, DIVERGED 10 | CONVERGED 10, DEAD 9, DIVERGED 3 | CONVERGED 7, DEAD 8, DIVERGED 7 |
| LF3 (ψ 0) | CONVERGED 20 | CONVERGED 24 | CONVERGED 24, DIVERGED 1 | CONVERGED 27, DIVERGED 1 | CONVERGED 25, DIVERGED 3 |
| LF6 | CONVERGED 11, ORBITING 9 | CONVERGED 4, ORBITING 20 | CONVERGED 3, ORBITING 22 | CONVERGED 5, ORBITING 23 | CONVERGED 5, ORBITING 23 |
| LW0 | CONVERGED 18 | CONVERGED 22 | CONVERGED 15, DIVERGED 8 | CONVERGED 14, DIVERGED 2 | CONVERGED 12, DIVERGED 4 |

LF3's five divergences are r × 2, heads × 2, heads × 10 and the workers' coin × 0.02 and × 0.1,
the horse price crossing 1e-6 of genesis at ticks 156–173, as registered. LOOP-SPEC §7.7's "LN3:
every run DEAD" is corrected here: two Tier-3 runs orbit.

### Labour supply's bound (new; STATE decision 277)

The runs at L in which a worker's unclamped offer ln(1 + w/P_s)/χ_max reaches 1 (every head at
work), each with its ticks at the bound and its peak:

| id | runs at the bound: ticks (peak) |
|---|---|
| LB1 | r × 0.5 262 (2.05); N(0.5) 238 (1.86); the workers' coin × 2 82 (1.19); heads × 0.1 45 (1.05); JB(2) 22 (1.18) |
| LB2 | r × 0.5 273 (2.13); N(0.5) 250 (1.95); workers' coin × 2 98 (1.25); heads × 0.1 48 (1.05); JB(2) 24 (1.22) |
| LB3 | r × 0.5 247 (1.89); N(0.5) 219 (1.68); heads × 0.1 48 (1.07); workers' coin × 2 28 (1.03); JB(2) 18 (1.11) |
| LW1 | r × 0.5 264 (2.28); N(0.5) 242 (2.10); workers' coin × 2 106 (1.35); JB(2) 26 (1.23) |
| LW2 | r × 0.5 279 (2.34); N(0.5) 256 (2.15); workers' coin × 2 120 (1.40); JB(2) 29 (1.28); the good desk's coin × 0.02 8 (1.02) |
| LW3 | r × 0.5 238 (2.15); N(0.5) 214 (2.00); workers' coin × 2 81 (1.26); JB(2) 21 (1.14) |

The families' and controls' runs at the bound are in `lmc_tables.out`, section H. No run starts
at the bound; every run of LB1–LB3 and LW1–LW3 at the bound converges.

## A1.5 Registered predictions for P2.2b's engine run (replaces §8)

Registered before any P2.2b engine code or run exists. The conditions are this file's header and
LOOP-SPEC §2, with §A1.1's carry. The engine is expected to match the mirror as P2.2a's did:
- every class exactly at LB1–LB3, the families and the flow controls;
- ticks and years within 10%, with three runs per instance allowed within 25%;
- lowest prices and troughs within 5% of the value given;
- dead ticks within 10% or 5 ticks, whichever is larger, market by market for fodder, horse-days,
  the good and land; labour's and the union's are reported, not scored (decision 274);
- ticks at labour's bound within 10% or 5 ticks; a run whose peak is within 5% of the bound may
  cross it or not.

**E0. Before any mode-B run: a trace diff.** `lm_carry.tick` against the engine for 2,000 ticks at
LB1 on hold, w × 2, w × 0.5, r × 2, r × 0.5, b × 2 at genesis, heads × 2, heads × 10 and Zs × 2 (a
one-tick stock), and at LW1 on hold, r × 0.5 and b × 2, each from the engine's own genesis. They
must agree within 1e-12 in log, the allowed partings being the order's cancellation
(HORSES-RULES §6.4) and, after a glut, the reservation's chatter within the mirror's own one-ulp
spread (O55). A parting at tick 1 names the carry (§A1.1): the registration is amended before any
scored run, not after.

**E1. Nesting (R1).**
- With every plant and the loop absent, every committed tape keeps its text, `tape_hash`,
  `world_id` and per-tick hash stream. The gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
  demo-gb `0xfad880fe08d06645` and the probe, markets and horses pins do not move.
- A planted tape at θ 1 equals the same tape without plants, value for value, for 2,000 ticks at
  LB1 on hold, w × 2 and r × 2.
- A fixed plant (no wear, no order) equals fixed-Q drs, with Q the plant held.

**E2. The rest point and mode A.** At LB1–LB3 and every cost target, genesis from 1g's
`ChainEconomy` with the plants at κ_p·X is a fixed point within 1e-12. Mode A passes at L, with
largest gaps of 6.7e-16, 1.0e-15 and 4.5e-14 in the mirror and below 1e-9 in the engine.

**E3. The verdict instances.**

| id | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | stocks family | L (the mirror's rule) |
|---|---|---|---|---|---|---|
| LB1 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 211,000 |
| LB2 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 202,000 |
| LB3 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 232,000 |

Every run is CONVERGED, and so every instance is GO. The years, lowest baskets, dead ticks by
market and lowest horse prices per tier are §A1.4's §7.1 table. The engine's own elasticity probe
sets its L; the mirror's τ are LOOP-SPEC §6's.

**E4. The kick sets and the slowest mode.** Every kick set decays. The engine's fitted g at the
base is within 0.03 a year of the mirror's: 0.847 (LB1), 0.830 (LB2), 0.901 (LB3), 0.836 (LW1).
The mirror's largest root per tick over the targets is 0.997950, 0.997501 and 0.999068.

**E5. The cost shocks.** §A1.4's §7.2 rows: troughs, years to tolerance, and the years for the
heads and each plant to come within 5%, at LB1–LB3 and LW1–LW3.

**E6. The glut.** heads × 10 and heads × 2 converge at LB1–LB3 with no dead tick. The lowest horse
price is 0.214–0.221 of target, the highest 2.02–3.76, and the withheld ticks are LOOP-SPEC §7.3's
(303, 1,315; 221, 1,034; 662, 2,694).

**E7. The pass-through.** After r × 2 at LB1: horse-days at most 5 dead ticks (mirror 0), fodder
175, the good 11, land 12; the fodder low 0.409 and the tasks' horse-days low 0.509. B-cut without
plants (LN7) has at least 100 dead horse-day ticks (mirror 146) and does not converge.

**E8. The flow control.** LW1–LW3 are GO, with §A1.4's §7.1, §7.2 and §7.6 numbers. The O14
comparison is read as §7.6 reads it: a b × 2 trough 0.036 higher in log with stocks at δ 8%, and
1.36 times the median years.

**E9. Negative controls** (each at L). Each keeps its verdict: NO-GO for LN1, LN2, LN3, LN5,
LN6, LN7, LN8, LN9, LF3, LF6 and LW0; LN4 GO in Tiers 1–3S with its stocks family 25/26. Each
tier's converged count is §A1.4's verdict table's within 2 runs; a run that does not converge may
be DEAD, DIVERGED or ORBITING where §7.7's table has another of the three (the carry alone moved
19 such runs at LN7). Beside that:
- LN1 and LN7 converge in no tier (the roots 1.0147 and 1.0104 a tick);
- LN2 orbits in Tier 3 (at least 10 of 25);
- LN3 is DEAD in every tier but two Tier-3 runs (b × 0.5 at genesis and dated), which orbit;
- LN5 orbits in every tier; LN6 is DEAD in every tier and in mode A;
- LF3 diverges on r × 2, heads × 2, heads × 10 and the workers' coin × 0.02 and × 0.1, the horse
  price crossing 1e-6 at ticks 156–173; at ψ 0, heads × 2 and × 10 cross it at ticks 155–158 at
  LB1–LB3;
- LF6 converges in 11, 4, 3, 5 and 5 runs (Tiers 1, 2, 3, 3S and the stocks family);
- LW0 fails Tier 3 (15 of 23 converge);
- LB1 at 12 ticks a year fails Tiers 1–2 (0 of 40).

**E10. Families, if built.** LF1, LF2, LF5, LF7, LF8 and LC1 are GO with §A1.4's numbers. LF4
needs STATE O51's roles and is GO in the mirror.

**E11. Labour supply's bound.** The runs of LB1–LB3 and LW1–LW3 that reach it are §A1.4's, with
their ticks at the bound; none starts there, and each converges.

**What would refute the design and send it back to the mirror:**
- a class change at LB1–LB3;
- a runaway of the horse price at LB1–LB3 with ψ 0.25;
- a converged run ending more than 1e-12 off its target's oracle point;
- a θ = 1 tape differing from the plant-free tape;
- the engine's slowest mode above 1 at any target.

## A1.6 Numbering, cross-references and S1

LOOP-SPEC numbered its decisions L1–L12 and its open items O-L1–O-L7, and cited STATE's and
FUNDED's items by their numbers of the day. In STATE.md from L0.9:

| LOOP-SPEC | STATE.md |
|---|---|
| L1–L12 | decisions 260–271 |
| O-L1–O-L7 | O60–O66 |
| "FUNDED decision 242" (§2.1), "decision 243" (§2.7) | FUNDED's 242 and 243, now 256 and 257 (STATE's own 243 is M5's order) |
| "STATE's O51" (§2.6), "O51 in STATE" (§7.5), "O51's roles" (L5, E10) | STATE's O51, storable running goods |
| "IDLE-SPEC's O51" (§0 item 5, §7.3) | STATE's O52, the maker's collapse through a long glut |
| "FUNDED's O51" (§6) | STATE's O58, the space-heavy county |

**S1 with its pumping loop** (GOODS-CHAIN §6 step 5 lists it with rule B's horse) is out of P2.2b
(STATE decision 276): M3's county is unfunded at S1's point, as for S1 without pumping (L10,
O-L3). Both wait for a funded steam county.

**O-L7 is answered by decision 274**: labour's dead ticks are reported, not scored, at the
harness's bar.

## A1.7 Files

Under `D:/rustyecon-p2l/fix-report/`:
- `loop-carry/make_lm_carry.py`: writes `lm_carry.py` from `lm_mirror.py`.
- `loop-carry/lmc.py`: runs a registered script (`lm_battery.py`, `lm_extra.py`, `lm_kick.py`,
  `lm_summary.py`, unedited) on `lm_carry` in place of `lm_mirror`, and records each run's ticks at
  labour supply's bound.
- `loop-carry/run_all.sh`: the battery, the extra runs, the kick sets and the tables, in order.
- `loop-carry/lmc_nest.py`, `lmd_check.py`, `lm_drs2.py`, `lmc_supply.py`, `lmc_compare.py`,
  `lmc_tables.py`: §A1.2's checks, the second plant map, labour's bound for FUNDED-A1, the
  run-by-run comparison with the registered battery (§A1.3), and §A1.4's tables.
- `loop-carry/model/`: the model with every output (`lm_battery_<id>.json`, `lm_battery.out`,
  `lm_extra.*`, `lm_kick.*`, `lm_summary.*`, `lmc_*.out`, `lmd_check.out`), and the registered
  outputs in `registered/`.
- `amendments/SHA256SUMS`: this file's sha256, FUNDED-A1's and IDLE-SPEC-A1's.

To rerun in WSL: copy `D:/rustyecon-p2l/loop-mirror/{ag,capacity,model}` to a scratch folder, put
`make_lm_carry.py` and `run_all.sh` beside `model/` and run `python3 make_lm_carry.py model`; copy
the `lmc_*`, `lmd_check.py` and `lm_drs2.py` scripts into `model/`, and move the registered
outputs into `model/registered/`; then `bash run_all.sh` (fix its `cd` to the folder),
`python3 lmc_nest.py`, `python3 lmd_check.py`, `python3 lmc_compare.py` and
`python3 lmc_tables.py` in `model/`.
