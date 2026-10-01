# SPEC: O97, the type switch at the wall, scanned in the mirror and registered (scan-switch)

Dated 2026-09-30. Work label `scan-switch`, Phase 2 proper's second session (PHASE2-S1 §6, item C),
branch `phase2-s2`. The worktree `D:/rustyecon-wt/p24` was read and not changed. It was at
`a483ed0` when this work began; P2.4.1–P2.4.3 (`45fb30a`–`ec66cb7`, wave A's registration and
machinery, docs only) landed during it and touch no crate, so the oracle is the same code.
Scratch: `D:/rustyecon-p24/scan-switch/`. This file chooses O97's rule by a mirror scan and
registers the mirror's predictions for its engine run, before any engine code for it exists (R5).
Its proposals are SW1–SW7 and its open items OS1–OS6 (§8), for STATE.md to number in its ranges
(decisions 400–449, open items O110–O129).

**Evidence.**
- The oracle: the worktree's unit 1d (`WorkerEconomy::solve`), unchanged, called from this label's
  copy of the wall frame's scratch crate (`oracle/`, a `sw` mode added; the frame's own copy is
  untouched). Its doubles are `registered/points_is1.jsonl` and `points_is2.jsonl`.
- An independent 50-digit solve, `hp/solve_sw_mp.py` (mpmath 1.3.0, laborformal's venv). It is
  written from unit-1d.md §4's equations, extends the wall frame's `hp/solve_mp.py` to types with
  pool efficiency ε > 0, and reads no oracle code.
- The switch mirror `model/wms.py`, written by `model/make_wms.py` from the wall frame's registered
  mirror `wm.py` (copied, sha256 `3f93de64…c4bc` checked) by 8 exact replacements. The wall
  frame's `wall.py`, `wb.py`, `wstab.py`, `wel.py`, `run_battery.py` and the rest are copied
  unedited; `sw.py`, `swb.py`, `swlocal.py`, `swel.py` and `swrun.py` carry them to the switch.
- No engine run was made. Every agent number below is the mirror's.

**Conditions of every number, unless its line says otherwise.**
- The economy is IS1 (§2): IW1 with unit 1d's E efficiencies on its reserved types.
- The rule is the chosen one (§3): the migration rule, participation at the split's own wage,
  `rate.switch` 26 a year.
- The dials are C2m at 52 ticks a year, with `Saturate` and planned assignment; ρ 0, the fixed
  basket, no government, no loop.
- L is 22,000 ticks at IS1 (24,000 at the control IS2), by the elasticity probe's rule (§6.5).
  The tolerance is 1e-3 in log on each of 20 observables (§3.9).
- The classes and the early stop are the wall frame's (PROBE-SPEC §4.5; a run stops early once
  it has stayed within 1e-6 in log of its target for 2,000 ticks, from tick 4,000 on).
- Python 3.10.12 with numpy 2.2.6 in WSL, 12 to 24 processes.

## 0. The answer

**GO for the migration rule, on IS1.** A type with pool efficiency ε > 0 and reserved tasks keeps
one state, its pool share a: the share of its hours it sells to the pool. Each tick, before it
offers, it reads two posted wages, the pool's wage for an hour of its own, e = ε·w, and its
reserved wage w_i, and moves a toward the market that pays more:

    g = ln(e / w_i)
    a' = a + share(k·g)·(1 − a)    if g > 0      (a share of the reserved hours moves to the pool)
    a' = a·exp(k·g)                if g < 0      (a share of the pool hours moves back)
    a' = a                         if g = 0

with share(x) = 1 − e^(−x) and k = rate.switch/tpy. Its participation reads the wage its offered
hours earn, v = w_i + a'·(e − w_i); its hours are n = N_i·min(ln1p(v/P_s)/χ_i, 1); it sells
(1 − a')·n hours on its reserved market and ε·(a'·n) efficiency hours to the pool. A share of the
worse market's hours, share(k·|g|), migrates each tick. At rate 26 a year a 10% wage gap moves
4.9% of them a week.

It rests exactly at unit 1d's switch, v_i = max(ε_i·v, ζ_i·ν_i·P_s): a type is at its wall where
a = 0 and w_i ≥ ε·w, and pooled where w_i = ε·w, with a = its pool hours over its hours (§4).

**The scan** (§6). Candidates on IS1's battery (109 runs) and Tier 3S (20), and on the control
IS2's; the rest point at 39 points each; the dial neighbourhood for the viable ones.

| candidate | the oracle point a rest point | IS1: battery and 3S (129) | IS2: battery and 3S (129) | verdict |
|---|---|---|---|---|
| **step**: every hour to the better market, a kept at a tie | only on a bitwise tie: at 3 of 39 points (pooled, 12 a year) one ulp moves every hour and the economy dies within 3 ticks | 121 CONVERGED, **8 DEAD** (every run to a pooled target) | 5 CONVERGED, **100 DEAD, 4 DIVERGED**; 3S 20 DEAD | rejected |
| **replicator**: logit(a') = logit(a) + k·g (rate 5.2 at IS1, 26 at IS2) | yes, exactly | 121 CONVERGED, **8 STUCK** (every run to a pooled target from IS1's walled genesis: a = 0 absorbs) | 129/129 | rejected: a spurious rest point |
| **static split**: a = 1/(1 + e^(−β·g)), β 50 | no: at a tie it puts half the type's hours in the pool; 7.2 in log off after 3 ticks | — | — | rejected: moves the point (a new oracle, not 1d's) |
| **migration, participation at the split's wage** (chosen), rate 1.3 to 520 a year | yes, exactly (6.7e-16 after 3 ticks) | 129/129 at each of 1.3, 2.6, 5.2, 13, 26, 52, 104, 260, 520 | 129/129 (26) | **GO at 26** |
| migration at rate 1,040 | yes | 127 CONVERGED, 2 ORBITING (res.services.trained=0.02, the largest pooled share) | — | the edge at 52 a year |
| **migration, participation at max(ε·w, w_i)** (1d's v_i read literally), rate 2.6 to 26 | yes, exactly | 129/129 at each | 129/129 (26) | the named alternative |

**At the chosen rule** (§6, §7):
- **IS1 is GO, as IW1 is**: Tiers 1–3 28/28, 40/40, 41/41, Tier 3S 20/20, Tiers 3 and 3S again at
  10·L 61/61; every kick set at the 13 targets passes. Medians (slowest) 338 (452), 446 (575),
  561 (734) ticks; Tier 3S 404 (558). IW1's are 340 (452), 416 (575), 561 (705).
- **The types cross, both ways.** In 61 of the 129 runs a type pools at some tick. From IS1's
  walled genesis the trained reaches its pooled share at all four pooled targets, and the master
  at tail.services 0.2; displaced into the pool (sw[T]=V, RW, JB, x\*/2), each returns to its wall.
- **The other 68 runs are IW1's, run for run.** Where no gap ever turns positive the switch never
  acts, and class, ticks to tolerance and peak D̂ are the wall frame's registered ones exactly.
- **Its margin.** The dial neighbourhood (rate, buffer and adjust × 0.75, 0.9, 1.1, 1.25; tilt
  0.05 to 1) keeps IS1's Tier 3 41/41 at all 17 settings, and so does IS2's; the switch's rate
  alone × 0.75–1.25 41/41 at each. All 129 converge at every rate from 1.3 to 520 a year; at
  1,040 two orbit. At 12 ticks a year the edge is lower: IS2's Tiers 1–2 converge at 52 a year
  and 10 of 68 orbit at 104 (OS4).
- **The largest root per tick** over IS1's 13 targets is 0.992722 (0.684 a year), at
  tail.services 0.11, where the trained sits 0.0095 in log inside its switch. IW1's largest is
  0.991911. At every walled target IS1's roots are IW1's to the printed digit.
- **The cost.** Where the pool pays more, the trained leave the reserved market services need:
  RW(2) bottoms at 0.026 of Y\* with 53 dead ticks, against IW1's 0.576 and none (OS2).

**The instance** (§2). IS1 is IW1 with 1d's E efficiencies: the trained ε 1.5, the master ε 1.8,
support 1 each. At IW1's base both stay at their walls (premia 1.0764 and 1.6003), so the base
point is IW1's double for double. The trained pools at four of the 12 cost targets
(tail.services 0.11 and 0.2, res.services.trained 0.036 and 0.02), the master at one
(tail.services 0.2). The control IS2 is IS1 with the trained's reserved hours 0.02: the trained
pools at its base and returns to its wall at res.services.trained × 2.

**The roles' addition** (§3). One optional field on the workers, `pool` (the pool's labour good,
ε, the rate, the genesis share), and one `ActorState` variant appended at the end,
`SwitchWorkers { share, pool }`. No new kind, market rule or other state. Off when absent (P2.3's
role bit for bit) and structurally off at ε = 0.

## 1. The problem

Unit 1d (unit-1d.md §4.3) prices a type's hours at

    v_i = max(ε_i·v, ζ_i·ν_i·P_s),    ζ_i = expm1(χ_i·D_i/N_i)

A type sells at the pool's wage if that covers its reserved work: it is **pooled**, does its
reserved hours D_i and offers the rest, ε_i·(n_i − D_i) efficiency hours, to the pool. Otherwise
its wage rises until its supply equals its reserved demand: it is **at its wall**, with a premium
v_i/(ε_i·v) above 1. The two meet where the type's pool hours are 0, so the switch is continuous.

IW1's trained and master have ε 0 (decision 394), so each sells on exactly one market and no
agent ever chose between two. The fidelity review of P2.3 named the gap (O97, decision 397 as
amended): the wall's GO does not test the switch.

An agent rule needs, from posted prices alone (R13):
- to rest exactly at 1d's point in both regimes, with no band and no other rest point;
- to leave a pooled type's reserved wage pinned at ε·w by moving hours, since nothing else pins
  it: the type is indifferent there, so its split is set by the reserved market's clearing, not by
  prices;
- no clamp on a price or a share (R3; a step on its own offer is allowed);
- to nest to the current roles when off, bit for bit.

The indifference is the whole difficulty. A static split (a function of the price ratio) must sit
off ε·w to split the hours at all, so it moves the point. A step moves every hour at the smallest
gap, so it chatters. An integrator that moves the split while the gap is open rests only where the gap is
closed; the question is which one keeps both corners open and is stable.

## 2. The instances

### 2.1 IS1: parameters

IW1 as registered (the wall frame's SPEC §2.1: services z 1, μ 0.75, tail 0.1, the trained's R
0.04; goods z 1, μ 1, the master's R 0.03; space 1; Appendix B's machine θ 1, own 0.3, λ 0.05,
b 0.4; η 0.5; T 520 a year; the entrant 130 a year at χ_max 1; the trained 52 and the master 26
at χ_max 2), with two changes:

| item | IW1 | IS1 | basis |
|---|---|---|---|
| trained ε | 0 | **1.5** | unit-1d.md §3.3 E's trained |
| master ε | 0 | **1.8** | unit-1d.md §3.3 E7's master |
| support ν | 1, 1 | 1, 1 | the probe's transfer |
| the switch | none | the rule of §3 at rate.switch 26 a year, both types | SW1, SW2 |

The oracle takes these through `WorkerType { efficiency }`; nothing else in `worker_params` moves.

### 2.2 IS1: the oracle's points at 52 a year

`registered/points_is1.jsonl` (39 points: the base and 12 targets at 52, 12 and 365 a year).
a\* is a type's pool hours over its hours; d is its switch distance ln(ζ_i·ν_i·P_s/(ε_i·v)) at
50 digits, above 0 at its wall and below 0 pooled.

| target | v | P_s | w_T | w_M | trained, a\* | master, a\* | d_T, d_M | depth | coverage | participation E / T / M |
|---|---|---|---|---|---|---|---|---|---|---|
| base | 0.807310 | 1.532910 | 1.303538 | 2.325461 | wall, 0 | wall, 0 | +0.0737, +0.4702 | 0.9426 | 1.6309 | 0.4231 / 0.3077 / 0.4615 |
| land.mach=0.44 | 0.798896 | 1.559880 | 1.286684 | 2.285469 | wall, 0 | wall, 0 | +0.0711, +0.4633 | 0.8460 | 1.6027 | 0.4135 / 0.3008 / 0.4511 |
| land.mach=0.36 | 0.816318 | 1.506161 | 1.321600 | 2.368453 | wall, 0 | wall, 0 | +0.0763, +0.4774 | 1.0478 | 1.6598 | 0.4331 / 0.3150 / 0.4724 |
| land.mach=0.8 | 0.742377 | 1.809678 | 1.173977 | 2.021411 | wall, 0 | wall, 0 | +0.0528, +0.4139 | 0.2164 | 1.3815 | 0.3438 / 0.2500 / 0.3750 |
| land.mach=0.2 | 0.859751 | 1.401921 | 1.408953 | 2.578238 | wall, 0 | wall, 0 | +0.0885, +0.5104 | 1.6001 | 1.7833 | 0.4783 / 0.3478 / 0.5217 |
| tail.services=0.11 | 0.890219 | 1.555513 | 1.335328 | 2.359749 | **pooled, 0.007031** | wall, 0 | **−0.0095**, +0.3871 | 1.0310 | 1.6072 | 0.4525 / 0.3099 / 0.4615 |
| tail.services=0.09 | 0.726831 | 1.512992 | 1.286600 | 2.295244 | wall, 0 | wall, 0 | +0.1656, +0.5621 | 0.8467 | 1.6524 | 0.3923 / 0.3077 / 0.4615 |
| tail.services=0.2 | 1.744737 | 1.913275 | 2.617105 | 3.140526 | **pooled, 0.286088** | **pooled, 0.049672** | −0.4753, −0.0788 | 1.6121 | 1.3067 | 0.6481 / 0.4310 / 0.4857 |
| tail.services=0.05 | 0.449548 | 1.455049 | 1.237327 | 2.207343 | wall, 0 | wall, 0 | +0.6070, +1.0035 | 0.3986 | 1.7182 | 0.2692 / 0.3077 / 0.4615 |
| res.services.trained=0.044 | 0.815551 | 1.548558 | 1.498715 | 2.349198 | wall, 0 | wall, 0 | +0.2030, +0.4702 | 0.9518 | 1.6144 | 0.4231 / 0.3385 / 0.4615 |
| res.services.trained=0.036 | 0.786142 | 1.519709 | 1.179213 | 2.305434 | **pooled, 0.035671** | wall, 0 | −0.0475, +0.4881 | 0.9184 | 1.6451 | 0.4169 / 0.2872 / 0.4615 |
| res.services.trained=0.08 | 0.994887 | 1.889080 | 4.578869 | 2.865778 | wall, 0 | wall, 0 | +1.1211, +0.4702 | 1.1304 | 1.3234 | 0.4231 / 0.6154 / 0.4615 |
| res.services.trained=0.02 | 0.647259 | 1.475570 | 0.970889 | 2.238474 | **pooled, 0.391428** | wall, 0 | −0.6023, +0.6530 | 0.7400 | 1.6943 | 0.3637 / 0.2528 / 0.4615 |

- **Every walled target is IW1's point double for double**: the 27 of IS1's 39 points where both
  types are at their walls equal the wall frame's `points.jsonl` in every value genesis and
  scoring read (`model/walled_same.out`). So are their edges (the wall frame's §2.4): the least
  depth 0.2164 at land.mach 0.8, coverage at least 1.3067.
- **At the pooled targets** the wall's depth is 0.74–1.61 and coverage 1.31–1.69, so every
  point is a funded wall; no type's participation passes 0.65, and no reserved market is short
  (D_i/N_i at most 0.615, at res.services.trained 0.08, as IW1's).
- **The switch distance is registered, not bounded** (SW7): tail.services 0.11 sits 0.0095 in log
  inside the trained's switch, by design, so that a type near its switch is in the battery.
- At 12 and 365 a year every target has the same prices, regime and pool shares; quantities scale
  by 52/tpy.

### 2.3 Uniqueness and the independent solve

- The oracle returns `Interior` with margin `Wall` at all 39 points of IS1 and all 39 of IS2.
- `hp/solve_sw_mp.py` (`out/solve_sw_mp.out`, `solve_sw_mp_is2.out`) solves every point at 50
  digits from the equations: the least fixed point of the walk over the four walled sets, the
  pool's clearing at the wall by bisection, and the sign sequence of the excess demand on 1,001
  thresholds of the line and 800 wages of the wall. **Exactly one sign change at all 78 points.**
  Every pooled flag is the oracle's; every value within 1.13e-15 relative of the oracle's doubles
  (v, P_s, Y, n_D, p_m, X, both prices, both wages, both reserved demands, both types' hours, the
  entrant's hours, the provider's baskets); pool hours within 2.9e-16 absolute.
- At ρ 0 uniqueness is also proved (unit-1d.md §5.4: f nonincreasing along the whole path).

### 2.4 IS2, the reported control

IS2 is IS1 with the trained's reserved hours 0.02 a unit of services (`registered/points_is2.jsonl`).
The trained is pooled at its base (a\* 0.391428, d −0.6023) and at every target but
res.services.trained × 2 (0.04, IS1's base point, where it walls). The master is walled throughout
(its least distance +0.0542, at tail.services 0.2). Its targets are its own coefficients' ×1.1,
×0.9, ×2 and ×0.5: land.mach and tail.services as IS1's, res.services.trained 0.022, 0.018, 0.04
and 0.01. It tests a pooled base, which IS1's battery reaches only at its cost targets. It is a
control and not a verdict instance (SW5): its land.mach 0.8 target sits 0.016 in log above the
wall's junction, below decision 370's 0.2.

## 3. The rule, as the engine should build it

### 3.1 What changes, and what does not

- **Core, markets and engine: nothing.**
- **Agents:** one optional field on `BasketWorkers` (the many-market workers), `pool`, and one
  `ActorState` variant appended after `PlantedCapacity`, `SwitchWorkers(SwitchWorkersState)`, as
  P2.2b appended its planted states (decision 287). A workers actor without `pool` keeps
  `Workers(WorkersState)` and P2.3's rule bit for bit. The field is
  `#[serde(default, skip_serializing_if = "Option::is_none")]` on the raw and the resolved spec, so
  every committed tape keeps its canonical text, `tape_hash` and `world_id` (L0.5's lesson).
- **Probe:** `probe::markets` gains IS1 and IS2, the switch's observables, readouts and grammar
  term (§3.9), and one tape, `tapes/markets-is1.ron` (IS2 by `--inst is2`, no committed tape).

### 3.2 The field

    RawBasketWorkers.pool: Option<RawPool>
    RawPool {
        good: Key,        // the pool's labour good (`labour`): an `Instant` good, not the pop's own `labour`
        efficiency: Key,  // param, `Dimensionless`, live: ε, efficiency hours per hour in the pool
        rate: Key,        // param, `RatePerYear`, live, read as `LogStep` (rate/tpy): k
        share: f64,       // the pool share at genesis, in [0, 1] (as the technique's `share`)
    }

The pop's own `labour` good stays its reserved market. Resolved: `Pool { good: GoodId,
efficiency: Site, rate: Site }`, the genesis share seeding the state.

### 3.3 Decide, in this evaluation order (normative)

For a workers actor with `pool`, at its home node, from the phase-start state:

1. w = price(pool.good); w_i = price(labour); P_s = the basket's price, summed from 0.0 in item
   order (as now); ε = param(efficiency); a = its state's `pool`.
2. **If ε == 0.0, the rule is off:** decide exactly as P2.3's workers (hours N·min(ln1p(w_i/P_s)/χ,
   1), mint and sell them, basket orders), and the state is `SwitchWorkers { share: F, pool: a }`
   with a unchanged. No pool order is placed.
3. e = ε·w; g = ln(e/w_i); k = param(rate) as `LogStep`.
   - if g > 0: a' = a + (−expm1(−(k·g)))·(1 − a);
   - if g < 0: a' = a·exp(k·g);
   - otherwise a' = a.
4. v = w_i + a'·(e − w_i).
5. F = min(ln1p(v/P_s)/χ_max, 1.0) (the support of χ, not a clamp); n = N·F, N = param(heads).
6. r = (1 − a')·n, the hours offered on its own market; p = ε·(a'·n), the efficiency hours offered
   to the pool.
7. Mint r of `labour` (when r > 0) and sell r, as now. When p > 0, mint p of `pool.good`
   (`Endowment`) and sell p. At a' = 0 no pool order is placed, so a walled type's orders are
   P2.3's.
8. The basket budget is share(spend)·coin, and the basket orders are P2.3's.
9. The state is `SwitchWorkers { share: F, pool: a' }`.

`model/sw_vectors.py` writes this order again from this text and matches the mirror's tick bit for
bit on 40,000 random type-states (every price × e^(±1), pool shares uniform, 0, 1 and at genesis,
and exact ties); its first 24 are `registered/rule_vectors.json`, inputs and outputs as Python
`repr` doubles, for the engine's rule test.

**Produce and upkeep** are P2.3's: the pop eats its baskets. Unsold hours on either market die
(both goods are `Instant`). Its coin receives w_i·(sold own hours) + w·(sold pool hours) at
settlement; nothing in the rule reads either.

**Why no clamp (R3).** At g > 0, a' is a convex combination of a and 1 with weight share(k·g) in
[0, 1); at g < 0, a' = a·e^(k·g) with the factor in (0, 1). So a' stays in [0, 1] by
construction. v is a convex combination of the two posted wages. The only min is F's, the
support of χ, as in every workers rule.

**Why R13 holds.** The rule reads two posted prices, the basket's prices, its own params (ε, k, N,
χ_max) and its own state a. No volume, fill, other actor or oracle.

**Its corners are open.** At a = 0 and g > 0, a' = share(k·g) > 0; at a = 1 and g < 0,
a' = e^(k·g) < 1. A type at its wall enters the pool as soon as the pool pays more, and leaves it
as soon as it pays less. This is what the replicator lacks (§6.2).

### 3.4 State

`SwitchWorkersState { share: f64, pool: f64 }`: `share` is F, as `WorkersState::share` is;
`pool` is a, in [0, 1]. The checks `apply` and `validate` make read both. At genesis `pool` is the
spec's `share` and `share` 0.0, as `WorkersState` starts.

### 3.5 Off

- **Absent:** P2.3's rule and state, bit for bit; every committed tape's text, ids and streams.
- **ε = 0:** the rule is skipped (step 2); the orders and deltas are P2.3's; the state holds a.
- The wall frame's IW1 and IC1 and the commons' C1, C2 and C1N carry no `pool` and are unchanged.

### 3.6 Load checks

Each a `LoadError` at its path:
- `pool.good` not declared, not `Instant`, or equal to the pop's own `labour`;
- `efficiency` not `Dimensionless`; `rate` not `RatePerYear`;
- `share` outside [0, 1] or not finite;
- `pool` together with `exit` (the priced exit's commons rule is not scanned with the switch).

A pop whose own good no desk buys would post a reserved market with no demand, which `Saturate`
drives down without end; the instance must give every switch pop reserved hours (IS1 and IS2 do:
R_T 0.04 or 0.02 on services, R_M 0.03 on goods).

### 3.7 The tape

As `tapes/markets-iw1.ron` with:
- params `inst.trained.efficiency` 1.5 and `inst.master.efficiency` 1.8 (`Dimensionless`, basis
  `Assumed("scan-switch SPEC §2.1: unit-1d.md §3.3 E's efficiencies")`), and
  `rate.switch.trained` and `rate.switch.master` 26 (`RatePerYear`, basis `Assumed("scan-switch
  SPEC §6.3")`);
- each reserved pop's spec with
  `pool: (good: "labour", efficiency: "inst.<type>.efficiency", rate: "rate.switch.<type>", share: a*)`,
  a\* 0.0 at IS1's base.

`rate.switch.*` is a `rate.*` dial, so the harness's `rate.*` scaling moves it with the markets'
rates.

### 3.8 Genesis

The wall frame's §3.6, unchanged, on `worker_params` with the reserved types' efficiencies:
prices v, 1, p_m, p_j and each type's wage v_i (ε_i·v where pooled); shares 1 − x\* = 0.0;
stocks z·Y and X; coins as registered for IW1, each pop (N_i·P_s + v_i·hours_i)/share(spend),
hours_i unit 1d's `WorkerEq::hours` (reserved plus pool hours). Each switch pop's `pool` is
a\* = `WorkerEq::pool_hours / WorkerEq::hours` where pooled and pool hours are positive, else 0.0.
At IS1's base every double is IW1's.

### 3.9 The harness

- **Observables: 20.** IW1's 18 (v; π of the type and each category; each reserved wage; each
  desk's threshold x = 1 − s; the seven markets' cleared volumes; each desk's output) and each
  switch pop's reserved share `rs.<type>` = 1 − a, in log. Targets from 1d: the pool's n_D on
  `labour`, each reserved market's D_i (`WorkerEq::reserved_hours`), each reserved wage v_i, and
  1 − a\* (1 at a wall). Every tolerance is the 1e-3 floor.
- **The run grammar** adds `sw[T]=V`: the switch pop T's pool share set to V exactly at genesis
  (Tiers 1–3 take V 0.05, 0.2 and 0.5). Every other term is the wall's (WALL-RULES §4; RW(F) is
  the pool's wage × F and each reserved wage × 1/F).
- **The battery** (`swb.run_list`): IW1's 103 runs in their order and names, then `sw[trained]=V`
  and `sw[master]=V` at the three values: 109 runs, 28, 40 and 41 by tier, none slack. Tier 3S is
  IW1's 20. A cost term moves the coefficient as at IW1; IS2's res.services.trained values are its
  own (§2.4).
- **Families** (reported): the dial neighbourhood on Tier 3 (41 runs: every `rate.*` including
  `rate.switch.*`, every `buffer.*` and every `adjust.*` × 0.75, 0.9, 1.1, 1.25; every desk's tilt
  0.05, 0.1, 0.25, 0.5, 1); `rate.switch.*` alone × 0.75, 0.9, 1.1, 1.25 on Tier 3; Tiers 1–2 at
  12 and 365 a year and under `Hold`; the wall's stocks (31), joint2 (60), joint4 (40) and basin
  (516, `pow_whole`); Tiers 3 and 3S at 10·L.
- **The switch's readouts**, reported and never scored, per switch pop: its ticks with a above
  1e-9; its largest a; the sign changes of its gap g between ticks where |g| > 1e-6 (the band
  keeps rounding-level flips near a pooled rest out of the count); a and g at the end. The wall's
  readouts (depth, breaches, largest shares, participation, dead ticks by market) are kept.
- **Start distances** are the wall frame's, with the reserved share 1 − a in D̂₀.

### 3.10 Tests the build adds, each failing without its change

- `switch_rule_matches_the_registered_vectors`: `registered/rule_vectors.json`, bit for bit.
- `switch_off_is_the_reserved_pop`: with ε 0 the actor's orders and deltas equal P2.3's reserved
  pop's from 500 random states; with `pool` absent the state variant is `Workers`.
- `switch_moves_toward_the_better_market`: from a = 0 at g > 0, a' = share(k·g); from a = 1 at
  g < 0, a' = e^(k·g); at g = 0 exactly, a' = a; a' in [0, 1] at extreme gaps.
- `switch_pays_both_markets`: settlement pays w_i on the reserved hours sold and w on the pool
  hours sold; money is conserved (R8) through RW(2).
- `markets_is1_genesis_is_unit_1d` and `markets_is1_rest_point_is_the_oracles` (39 points, 3
  ticks, 1e-12), as IW1's; `markets_is1_never_crossing_is_iw1`: IS1 from hold for 2,000 ticks
  gives IW1's tape's prices, S, D, fills and coins bit for bit.
- The load checks of §3.6.

## 4. Why the rest point is the oracle's

**The oracle's point is a fixed point, exactly.** Take genesis at 1d's point with a = a\*.
- A walled type: a\* = 0 and w_i > ε·w (its switch distance is at least +0.0528 at IS1's walled
  targets). Step 3 gives a' = 0·e^(k·g) = 0.0; step 4 gives v = w_i + 0·(e − w_i) = w_i; its
  hours are N·F(ln1p(w_i/P_s)) = D_i at ζ_i, all on its own market, none offered to the pool.
  These are P2.3's reserved pop's operations, so IW1's argument (the wall frame's §3.7) applies.
- A pooled type: the oracle posts its wage as `efficiency * v`, the double the rule computes as
  ε·w with w = v. So e == w_i bit for bit, g = ln(1) = 0.0, and a' = a\* exactly; v = w_i exactly;
  its hours n = N·F(ln1p(ε·v/P_s)) are 1d's supply at its wage; (1 − a\*)·n = D_i and
  ε·a\*·n = ε·(n − D_i) to rounding.
- Every other role is the wall frame's, and the pool's clearing now counts each pooled type's
  ε·(n_i − D_i), unit-1d.md §4.4's S.

In the mirror, genesis leaves every observable within 6.7e-16 in log after 3 ticks and within
4.4e-15 over 2,000 ticks at all 39 points of IS1 (within 1.1e-15 and 2.4e-15 at IS2's 39), every
state value within 3.7e-15, money within 1.2e-15 (`model/sw_rest.out`, `sw_rest_is2.out`).

**Every live rest point is the oracle's.** Take a live rest point of the agents: every market
trades and clears, and every state is constant. For a switch type, a' = a requires one of:
1. g = 0: w_i = ε·w. Its reserved market clears, (1 − a)·n = D_i, with n = N·F(ln1p(ε·w/P_s)),
   and the pool receives ε·(n − D_i) ≥ 0, so ε·w/P_s ≥ ζ_i. This is 1d's pooled type.
2. a = 0 and g < 0: all its hours are reserved, its market clears at w_i/P_s = ζ_i, and
   w_i > ε·w. This is 1d's type at its wall.
3. a = 1 and g > 0: it offers no reserved hours, while its reserved demand D_i = Y·R_ŷi > 0, so
   its market does not trade. Not live.

(The corner a = 0 with g = 0 is 1d's switch point, pool hours 0, which 1d counts pooled; the
allocation is the same.) So each type's wage is v_i = max(ε_i·v, ζ_i·ν_i·P_s) with ν_i 1, and its
pool supply is 1d's. With the wall frame's §3.7 items 1–6 unchanged, the rest point's equations
are unit 1d's at IS1's instance. They have exactly one solution (§2.3). So the agents' only live
rest point is the oracle's, and the rule has no band.

**R3 and R13** hold as §3.3 says. **Nesting** is §3.5 and §5.1.

## 5. Checks on the mirror

### 5.1 The switch off is the wall mirror, bit for bit (`model/sw_nest.out`)

- **IW1, no switch:** `wms.tick` is `wm.tick` bit for bit (every price, share, coin, stock equal as
  doubles, every tick, 3,000 ticks) from the wall frame's E0 starts (hold, p[labour]\*2, JB(0.5),
  x\*/2, s[goods]=0.5, RW(2), p[labour.trained]\*0.5, coin.workers.master\*0.1, stock.mach\*0.01)
  and the three coefficients' ×2 or ×0.5 at genesis: 12 runs.
- **The rule present with ε 0:** the same 12, bit for bit (structurally off).
- **I0–I3** (no reserved type), from hold, w\*2 and JB(0.5): 12 runs, bit for bit.
- **IS1 with the rule on, where no gap turns positive:** hold, p[labour]\*0.95 and \*0.8,
  p[labour.trained]\*1.2, p[labour.master]\*2, coin.workers.trained\*0.5, p[goods]\*1.05: every
  state value equals IW1's `wm` run bit for bit for 3,000 ticks, the pool shares 0.0 throughout.

### 5.2 The rule and the money (`sw_vectors.out`, `sw_money.out`)

- The rule written again from §3.3 matches the mirror's tick on 40,000 type-states bit for bit.
- Through six runs where types cross (RW(2), x\*/2, sw[trained]=0.5, JB(0.5), and hold at the two
  pooled targets), money is conserved within 2.9e-15 over 3,000 ticks, and the pool's supply is
  the entrant's hours plus each switch type's ε·a·n within 2.2e-16.

### 5.3 The full L (`sw_fullL.out`)

Fifteen runs to 22,000 ticks without the early stop (IS1: the four pooled targets, RW(2), x\*/2,
sw[trained]=0.5, JB(0.5), p[labour.trained]\*0.5, land.mach=0.8; IS2: hold,
res.services.trained=0.04, RW(2), x\*/2, tail.services=0.2@dated). Each keeps its class and ticks
to tolerance and ends within 1.55e-14 in log of its oracle point. Each pooled type ends at a\*
within 1.6e-15 absolute (7e-14 relative, at a\* 0.007). A type back at its wall ends in the subnormals: its share decays by e^(k·g)
a tick until a·e^(k·g) rounds back to a, at 13 ulps (6.4e-323) for the trained and 2 (1e-323) for
the master at 52 a year, as the technique's share stalls (A2; OS5).

## 6. The scan

### 6.1 The candidates

Every candidate reads the same posted wages, e = ε·w and w_i, keeps one state a (the step and
the static split need none), shares §3.3's steps 4–9, and is off at ε = 0.

- **Step:** a' = 1 if e > w_i, 0 if e < w_i, a at a tie. A step on the type's own offer (R3
  allows it).
- **Replicator:** logit(a') = logit(a) + k·g, written a' = (a + a·x)/(1 + a·x), x = expm1(k·g).
  Smooth; rests at g = 0 or at a corner.
- **Static split:** a' = 1/(1 + e^(−β·g)), β 50. Smooth; rests off the tie.
- **Migration** (chosen): §0's rule. Its gain is k·(1 − a) for g > 0 and k·a for g < 0, a kink at
  the rest.
- **Participation** for the integrators: at the split's wage (v = w_i + a'·(e − w_i)) or at the
  better posted wage (v = max(e, w_i), 1d's v_i read literally). They agree at every rest point.

A reservation on the reserved offer at ε·w (L0's device) is the step with a limit price; it would
chatter as the commoners' reservation did (the commons frame's §1.5), and a split that reads its
own last fills would break R13. Neither was run.

### 6.2 What each did (`out/scan_summary.out`; runs in `runs/`, not registered)

- **Step.** Exact only on a bitwise tie. At three of IS1's 39 points (three of the four pooled
  targets at 12 a year) the genesis wage is an ulp off ε·w; every hour jumps to one market, the other empties, and
  the economy breaks within 3 ticks (`sw_rest.out`). In the battery all 8 runs to a pooled target
  are DEAD at IS1; at IS2, whose base is pooled, 100 of 109 are DEAD and 4 DIVERGED, and all 20 of
  Tier 3S are DEAD.
- **Replicator.** The oracle's point is a fixed point, but a = 0 is absorbing: 0·x = 0. From IS1's
  walled genesis the trained can never enter the pool, so all 8 runs to a pooled target end STUCK
  at a rest point that is not the oracle's (the trained forced to its wall). At IS2, where no type
  that starts at a = 0 is pooled at its target, it converges 129/129. It fails exactly where a
  type must leave its wall, which is O97's case.
- **Static split.** At a tie it puts half the type's hours in the pool; at IS1's base it puts
  2.4% of the trained's hours there, where 1d puts none. Genesis is 7.2 in log off after 3 ticks.
  It is a different model (a preference over markets), with its own oracle; not 1d's.
- The step and the replicator fail the verdict battery and the static split fails at genesis, so
  none of the three was taken to the neighbourhood or the local roots.
- **Migration.** Exact (§4), and every class CONVERGED: at the split's wage at every rate from
  1.3 to 520 a year, at max(ε·w, w_i) at 2.6, 5.2, 13 and 26 (the rates run). Its edge at 52 a year is between 520 (129/129) and 1,040 (2 ORBITING, both
  at res.services.trained 0.02, where the trained's pooled share is largest).

### 6.3 The rate

| rate.switch a year | IS1 battery, slowest run (ticks) | largest root over IS1's targets (at tail.services 0.11) | the trained's corner at the base, e^(−k·d) | IS1 Tier 3 in the neighbourhood | IS2 at 12 a year |
|---|---|---|---|---|---|
| 1.3 | 5,279 (tail.services=0.11@dated) | — | — | — | — |
| 2.6 | 2,797 | 0.999334 | 0.996324 | — | — |
| 5.2 | 1,443 | 0.998657 | 0.992661 | — | — |
| 13 | 737 | 0.996553 | 0.981752 | 697/697 | 68/68 |
| **26** | **734** (tail.services=0.2@genesis) | **0.992722** | **0.963838** | **697/697** | **68/68** |
| 52 | 706 (JA(0.5), IW1's own slowest) | 0.987316 | 0.928983 | 697/697 | 68/68 |
| 104 | 706 | — | — | — | 58 CONVERGED, 10 ORBITING |
| 208 | — | — | — | — | 55 DEAD, 13 DIVERGED |
| 520 | 706 | — | — | — | — |
| 1,040 | 127 CONVERGED, 2 ORBITING | — | — | — | — |

The rate trades speed near the switch against the distance to the step it tends to. At 13 a year
and below, the switch is the slowest mode at tail.services 0.11 by a wide margin (0.9966 to 0.9993
a tick: years to settle), and from 5.2 down its corner at the base (0.9927 and slower) is slower
than the rest of the map there (0.9871). From 52 up it is never the slowest, but at monthly ticks
IS2's edge lies between 52 and 104. **26 a year**
(5 times labour's price rate) is chosen: its largest root, 0.992722 at the one target 0.0095 in
log from the switch, is within 0.001 of IW1's largest; every corner rate is at most 0.973931; its
edge is a factor 20–40 away at weekly ticks and 2–4 at monthly ones. The named dial alternative is
13 a year, twice the monthly margin and slower near the switch (SW2).

### 6.4 The participation form

At rate 26 both forms give every class the same on IS1's battery and 3S, IS1's neighbourhood and
IS2's battery and 3S; ticks differ by a few (IS1's slowest 734 at the split's wage, 738 at max).
At rate 5.2 the split's wage is a little faster (slowest 1,443 against 1,515). Their largest roots
at rate 26 agree within 2e-4 at every target (0.992722 and 0.992783 at tail.services 0.11;
`out/local_max26.out`). The split's wage is
chosen: participation reads what the offered hours earn at posted prices, and the rule stays a
convex combination with no kink at the tie. max(ε·w, w_i) is the named alternative (SW1).

### 6.5 The chosen rule on IS1 and IS2 (`out/tables_is1.out`, `tables_is2.out`; `registered/runs_*.tsv`)

**IS1**, 52 a year, L 22,000:

| set | runs | classes | ticks to tol, median (slowest) | worst dead (run) | lowest baskets over Y\* (run) | lowest depth; runs breaching | runs a type pools | largest pool share T / M |
|---|---|---|---|---|---|---|---|---|
| Tier 1 | 28 | 28 CONVERGED | 338 (452, JA(0.95)) | 0 | 0.694 (s[goods]=0.05) | +0.840; 0 | 5 | 0.141 / 0.040 |
| Tier 2 | 40 | 40 CONVERGED | 446 (575, JA(0.8)) | 8 (s[services]=0.2) | 0.330 (s[goods]=0.2) | +0.496; 0 | 16 | 0.508 / 0.178 |
| Tier 3 | 41 | 41 CONVERGED | 561 (734, tail.services=0.2@genesis) | 117 (JA(0.5)) | 0.002 (x\*/2) | −0.444 (JB(0.5)); 1 | 34 | 0.973 / 0.843 |
| Tier 3S | 20 | 20 CONVERGED | 404 (558, coin.desk.mach\*2) | 8 | 0.286 (stock.mach\*0.5) | +0.643; 0 | 6 | 0.100 / 0 |
| Tiers 3, 3S at 10·L | 61 | 61 CONVERGED | the same ticks | 117 | 0.002 | −0.444; 1 | 40 | 0.973 / 0.843 |
| stocks | 31 | 31 CONVERGED | 457 (635) | 84 | 0.000 (stock.mach\*0.01) | +0.090; 0 | 13 | 0.459 / 0.831 |
| joint2 | 60 | 60 CONVERGED | 581.5 (734) | 145 | 0.032 | −0.275; 3 | 57 | 0.955 / 0.805 |
| joint4 | 40 | 40 CONVERGED | 665.5 (853) | 204 | 0.000 | −1.339; 16 | 40 | 0.999 / 0.984 |
| basin | 516 | 516 CONVERGED | 572 (1,012, p[mach]\*0.1227) | 250 | 0.000 | −1.189; 90 | 405 | 1.000 / 0.999 |
| 12 a year, Tiers 1–2 | 68 | 68 CONVERGED | 118.5 (187) | 1 | 0.401 | +0.496; 0 | 21 | 0.487 / 0.137 |
| 365 a year, Tiers 1–2 | 68 | 68 CONVERGED | 2,566.5 (3,949) | 56 | 0.281 | +0.496; 0 | 19 | 0.513 / 0.193 |
| Hold, Tiers 1–2 | 68 | 68 CONVERGED | identical to Saturate, run for run | | | | | |
| neighbourhood, 17 settings × Tier 3 | 697 | 697 CONVERGED | 446 (566) at rate ×1.25 to 748 (986) at rate ×0.75 | 170 (JA(0.5), rate ×0.75) | 0.000 (x\*/2) | −0.444; 1 each | 34–35 each | |
| rate.switch alone ×0.75–1.25 | 164 | 164 CONVERGED | 561–562 (726–736) | 118 | 0.000 | −0.444; 1 each | 34 each | |

- **The 68 runs where no type ever pools are IW1's**, class, ticks and peak D̂ each the wall
  frame's registered value (`registered/never_pooled_is1.tsv`).
- **The four pooled targets** (genesis and dated alike; `runs_is1_battery.tsv`):
  tail.services 0.11 452 ticks (IW1 383), tail.services 0.2 734 (683), res.services.trained 0.036
  310 (319), res.services.trained 0.02 415 (423). Each switch type ends at its a\*.
- **Kick sets** (`registered/local.json`): all 13 targets pass, 14 kicks each; gain_tail at most
  1.39e-5, gain_peak at most 3.00. The base envelope decays at 0.988002 a tick (0.5338 a year,
  IW1's), the slowest at tail.services 0.11 (0.992768, 0.686 a year).
- **Largest root per tick** (PL): IW1's at every walled target (0.983590–0.991911); 0.992722,
  0.991067, 0.986969, 0.986832 at the four pooled ones. At a walled target a switch type's share
  moves by exactly e^(k·g) a tick while its gap stays below 0 (its corner rate): at most 0.973931
  (the trained at land.mach 0.8), decoupled, as the technique's share is.
- **The path cost of the switch** (OS2): RW(2) 0.026 of Y\* and 53 dead ticks (IW1 0.576 and 0);
  p[labour.trained]\*0.5 0.203 and 31 (0.576 and 0); p[labour]\*2 0.307 and 8 (0.773 and 0);
  s[services]=0.5 0.204 and 53 (0.222 and 21); tail.services 0.2 0.449 and 16 (0.516 and 0).

**IS2** (control), L 24,000 at 52 a year (20,000 at 12, 183,000 at 365): battery 109/109 (Tiers
346 (460), 440.5 (574), 556 (677)), Tier 3S 20/20 (406.5 (557)), 10·L 61/61, neighbourhood
697/697, 12 and 365 a year 68/68 each, Hold 68/68. res.services.trained=0.04, where the trained
returns to its wall, 509 ticks. Kick sets 13/13, gain_tail at most 1.43e-5; PL 0.983819–0.989622.
At 12 a year the trained's reserved market's one-tick multiplier is −0.010 (`swel.out`), near
the monthly edge (OS4).

## 7. Registered predictions for the engine run

Registered before any engine code for the switch exists. The conditions are the header's and
§2–§3's. The bands are the wall frame's (its §7):
- **classes** exactly;
- **ticks and years** within 10%, three runs a set allowed within 25%;
- **troughs and lowest depths** within 5%;
- **dead-tick and breach-tick counts** within 10% or 5 ticks, whichever is larger;
- **the switch's readouts**: ticks with a > 1e-9 and the largest a within 10% or 5 ticks;
  gap sign changes reported only.

Every run-by-run number is in `registered/runs_is1_<set>.tsv` and `runs_is2_<set>.tsv` (the
mirror stops a run early; the engine runs L ticks, so end values are read against the oracle,
not against the mirror's early stop, O108's lesson).

**E0. Before any mode-B run: a trace diff.** `wms.tick` against the engine for 2,000 ticks.
- IS1: hold, p[labour]\*2, RW(2), RW(0.5), JB(0.5), x\*/2, s[goods]=0.5, sw[trained]=0.5,
  sw[master]=0.2, p[labour.trained]\*0.5, stock.mach\*0.01, tail.services=0.2@genesis,
  tail.services=0.11@genesis, res.services.trained=0.02@genesis. IS2: hold,
  res.services.trained=0.04@genesis, RW(2).
- Every price, share, coin, stock, S and D within 1e-12 in log; each pool share within 1e-12
  absolute, and within 1e-12 in log where both are above 1e-6. The wall's A1 residue
  (stock.mach\*0.01 at tick 1) and the budget chain's ulp are allowed.
- `registered/rule_vectors.json` bit for bit (§3.10).

**E1. Nesting (R1).**
- With `pool` absent: every committed tape's text, `tape_hash`, `world_id` and 2,000-tick hash
  stream, `markets-iw1.ron`, `markets-c1.ron` and `markets-c2.ron` included; gate
  `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645` and the probe,
  markets, horses and loops pins do not move; P2.3's nesting tests pass unchanged.
- With ε 0: IS1's tape gives IW1's prices, S, D, fills and coins bit for bit (§5.1).
- **The 68 never-pooled runs of IS1's battery and Tier 3S are the engine's IW1 runs of the P2.3
  wave**, class, ticks to tolerance and peak D̂ (within 1e-9 relative), run for run
  (`registered/never_pooled_is1.tsv`). The closest any of them comes to its switch is a gap of
  −0.013 in log (coin.desk.goods\*0.5; `model/never_margin.out`), far above rounding.

**E2. The rest point and mode A.**
- Genesis from 1d's `WorkerEconomy` is a fixed point within 1e-12 after 3 ticks at IS1's and
  IS2's base and 12 targets, at 12, 52 and 365 a year (78 points; the mirror 1.1e-15).
- Mode A passes at L (the engine's gap below 1e-9).
- The engine's elasticity probe sets L: at IS1's base the switch does not act on the probe's
  e^(±0.01) kicks (the trained's gap stays below −0.063), so its τ are IW1's (goods 106.4 ticks)
  and L is 22,000, 20,000 and 164,000 at 52, 12 and 365 a year; at IS2 it acts, and L is 24,000,
  20,000 and 183,000 (`swel.out`).

**E3. The verdict.** IS1 is GO: Tier 1 28/28, Tier 2 40/40, Tier 3 41/41, Tier 3S 20/20
CONVERGED, and Tiers 3 and 3S again at 10·L with the same classes and ticks. Medians (slowest)
338 (452), 446 (575), 561 (734); Tier 3S 404 (558). Every other statistic per run as the TSVs.

**E4. The kick sets and the roots.** Every kick set decays at all 13 targets of IS1 and of IS2
(gain_tail at most 1e-3; the mirror's at most 1.4e-5). The engine's fitted decay at IS1's base is
within 0.03 a year of 0.534, and at tail.services 0.11 of 0.686. The mirror's largest root over
IS1's targets is 0.992722. At a walled target a switch pop's share moves by exactly e^(k·g) a tick
while its gap is below 0.

**E5. The switch.**
- In 61 of IS1's 129 battery and 3S runs a type's share passes 1e-9 at some tick; in the other
  68 it never leaves 0.0 (E1).
- At the pooled targets each switch pop ends at its a\* within 1e-10 relative (the oracle's
  value, §2.2; the mirror's full-L runs within 7e-14); the trained reaches it from its wall in all eight runs (genesis and dated), the
  master in the two at tail.services 0.2.
- At every other target each switch pop ends at its wall: its share 0.0 if it never pooled,
  else at most 1e-300 (the subnormal stall: at 52 a year 13 ulps for the trained and 2 for the
  master in the mirror).
- The largest pool shares and pooled ticks per run as the TSVs.

**E6. The path cost** (scored in the trough band): RW(2) bottoms at 0.026 of Y\* with 53 dead
ticks; p[labour.trained]\*0.5 at 0.203 with 31; p[labour]\*2 at 0.307 with 8; s[services]=0.5 at
0.204 with 53; tail.services=0.2@genesis at 0.449 with 16. Only JB(0.5) breaches the wall in the
battery (lowest depth −0.444), as at IW1.

**E7. The families**, reported with §6.5's numbers in the bands: the neighbourhood 697/697 and
the switch's rate alone 164/164; 12 and 365 a year 68/68 each; Hold identical to Saturate run for
run; stocks 31/31, joint2 60/60, joint4 40/40, basin 516/516.

**E8. The control IS2**, reported: battery 109/109, Tier 3S 20/20, 10·L 61/61, neighbourhood
697/697, 12 and 365 a year and Hold 68/68 each, kick sets 13/13; its TSVs.

**E9. The switch distances.** The generator's own solve reproduces §2.2's pooled flags, a\* and
switch distances within 1e-12 at every point of IS1 and IS2, and every walled target's point is
IW1's double for double.

**What would refute the choice and send it back to the mirror:**
- a class change in IS1's registered battery or Tier 3S, at L or 10·L;
- a CONVERGED run ending more than 1e-12 in log off its target's oracle point (the mirror's
  full-L runs end within 1.6e-14);
- a switch pop ending on the wrong side of its switch, or a pooled pop's share more than 1e-9
  relative from a\*;
- the engine's rest point differing from unit 1d's by more than 1e-12;
- a never-pooled run differing from IW1's;
- a kick set failing, or the slowest mode at or above 1 at any target;
- any committed pin moving.

**The scorer** is written and committed before the wave (decision 311). It reads the TSVs above,
`registered/local.json`, `never_pooled_is1.tsv`, `points_is1.jsonl`, `points_is2.jsonl` and
`switch_mp.json`, `switch_mp_is2.json`.

## 8. Decisions proposed and open items

Each is Claude's, on the user's delegation ("I leave all those calls up to you"; "keep going"),
and open to veto. The alternative named is the registered one (R6).

- **SW1. O97's rule is the migration rule, with participation at the split's own wage** (§0,
  §3.3): a switch pop's pool share moves toward the market that pays more by share(k·|g|) of the
  worse market's hours a tick, g = ln(ε·w/w_i); it offers (1 − a)·n on its own market and ε·a·n to
  the pool; n reads v = w_i + a·(ε·w − w_i). It rests exactly at 1d's switch, has open corners and
  no clamp. Alternative: the same rule with participation at max(ε·w, w_i), 1d's v_i read
  literally, which scanned alike. Rejected by the scan: the step (dead at every pooled rest), the
  replicator (a spurious rest where a type must leave its wall), a static split (moves the point).
- **SW2. `rate.switch` is 26 a year** (`RatePerYear`, read as `LogStep`), per switch pop, a
  `rate.*` dial (§6.3). Alternative: 13 a year, twice the margin at monthly ticks and slower near
  the switch.
- **SW3. The roles' addition is one optional field and one state variant** (§3.1–§3.6):
  `BasketWorkers.pool` (the pool's good, ε, the rate, the genesis share) and
  `ActorState::SwitchWorkers { share, pool }` appended after `PlantedCapacity`; off when absent and
  structurally off at ε 0; no new kind or market rule; `pool` with `exit` refused. Alternative:
  a new actor kind for a type on two markets.
- **SW4. IS1 is O97's instance** (§2): IW1 with unit 1d's E efficiencies (trained 1.5, master
  1.8), support 1, both switch pops at rate 26. Its base point is IW1's double for double; the
  trained pools at four of the 12 targets and the master at one. Its verdict is IW1's rule
  (decision 397): mode A, Tiers 1–3 and 3S CONVERGED, Tiers 3 and 3S again at 10·L, every kick set,
  L from the engine's probe, the scorer first; on 109 battery runs (IW1's 103 and sw[T]=V).
  Alternative: IS2, a pooled base, as the verdict instance.
- **SW5. IS2 is a reported control** (§2.4): IS1 with the trained's reserved hours 0.02, pooled at
  its base, walled at ×2. Not a verdict instance: its land.mach 0.8 target sits 0.016 in log above
  the wall's junction (decision 370 asks 0.2). Alternative: no control.
- **SW6. The harness at the switch** (§3.9): 20 observables (IW1's 18 and each switch pop's
  reserved share 1 − a in log), `sw[T]=V`, the switch's readouts reported and not scored, and the
  families (the neighbourhood with `rate.*` including `rate.switch.*`; the switch's rate alone; 12
  and 365 a year; Hold; stocks, joint2, joint4, basin; Tiers 3 and 3S at 10·L). Alternative: the
  pool shares read only through the volumes.
- **SW7. Each switch pop's distance from its switch is registered at every target and not
  bounded** (§2.2), beside decision 370's edges, which IS1 keeps (its walled targets are IW1's).
  Alternative: a 0.2 bound, which would drop tail.services 0.11 and res.services.trained 0.036,
  the targets nearest the switch.

Open items:

- **OS1. O97's other two cases are not built:** a pooled type with ε ≠ 1 and no reserved tasks
  (it needs only its hours scaled into the pool, and its own market would have no demand), and
  support ν ≠ 1 (the provider's transfer would be ν_i·N_i·P_s and participation ln1p(v/(ν_i·P_s))).
  Each needs its own nesting and scan when an instance uses it.
- **OS2. The switch's path cost.** When the pool's wage jumps above a type's reserved wage, the
  type leaves the reserved market its desks need, and output falls hard before the reserved wage
  catches up: RW(2) bottoms at 0.026 of Y\* with 53 dead ticks, where IW1 keeps 0.576 and none.
  O98's cousin; O14 and O24 carry it. A slower rate softens it and costs speed near the switch.
- **OS3. A type near its switch is slow.** At tail.services 0.11 (0.0095 in log inside the switch,
  a\* 0.007) the largest root is 0.992722 a tick at rate 26 and 0.998657 at 5.2. Whatever the rule,
  a pooled share near 0 moves little on the side that shrinks it.
- **OS4. The margin at monthly ticks.** At 12 a year IS2's Tiers 1–2 converge at rate 52 and 10
  of 68 orbit at 104; at 52 a year the edge is between 520 and 1,040. At rate 26 and 12 a year the
  pooled trained's reserved market has a one-tick multiplier of −0.010. An instance at monthly
  ticks may need `rate.switch` scaled.
- **OS5. The walled pool share stalls in the subnormals** (13 ulps for the trained at 52 a year),
  as the technique's share does (the wall's A2). A registration that reads its end value reads it
  as 0 or at most 1e-300.
- **OS6. The switch under the priced exit.** The rule compares wages only, so it is form-free;
  but a walled type under s(q) needs 1e's addendum first (O95, decision 377), and `pool` with
  `exit` is refused until a scan runs them together. The 1750-like trained type needs both.

## 9. Files, and how to rerun

Everything is under `D:/rustyecon-p24/scan-switch/`; `run.sh` reruns it in WSL, the 50-digit
solves on Windows.

| file | what |
|---|---|
| `SPEC.md` | this file |
| `SHA256SUMS` | sha256 of this file and of every registered output and script |
| `oracle/` | the wall frame's scratch crate, copied, pointed at the p24 worktree, with a `sw` mode |
| `hp/solve_sw_mp.py` | the independent 50-digit solve with pooled types |
| `hp/solve_mp.py`, `hp/edges_mp.py` | the wall frame's, copied unedited |
| `out/points_iw1.jsonl` (and `points.jsonl`, the same) | the wall frame's IW1 points, copied |
| `out/points_is1.jsonl`, `points_is2.jsonl`, `switch_mp*.json`, `solve_sw_mp*.out` | the oracle's points and the 50-digit solve |
| `model/wm.py`, `mm_carry.py`, `wall.py`, `wb.py`, `wstab.py`, `wel.py`, `run_battery.py` and the rest of the wall frame's model | copied unedited (sha256 as registered there) |
| `model/make_wms.py` → `model/wms.py` | the switch mirror, 8 checked replacements of `wm.py` |
| `model/sw.py` | IS1 and IS2, genesis, observables |
| `model/swb.py`, `swrun.py`, `swsum.py`, `swtab.py`, `swtsv.py`, `swcmp.py` | the battery, the runner and the tables |
| `model/sw_nest.py`, `sw_rest.py`, `sw_vectors.py`, `sw_money.py`, `sw_fullL.py`, `swel.py`, `swlocal.py`, `never_pooled.py`, `walled_same.py`, `is2_points.py`, `is_table.py` | the checks of §2 and §4–§6 and their `.out` files |
| `out/scan_summary.out`, `local_pass1.out`, `local_26.out`, `tables_is1.out`, `tables_is2.out`, `is1_vs_iw1.out`, `reg_is1_vs_iw1.out`, `is1_table.out`, `is2_table.out` | the scan's and the registration's summaries |
| `registered/` | `runs_is1_<set>.tsv` (12 sets), `runs_is2_<set>.tsv` (7), `local.json`, `never_pooled_is1.tsv`, `rule_vectors.json`, `points_is1.jsonl`, `points_is2.jsonl`, `switch_mp.json`, `switch_mp_is2.json` |
| `runs/` | every run's summary (jsonl): the scan's passes 1–4 and the registration runs; hashed for the record, not registered (the TSVs are), regenerated by `run.sh` |
| `explore/` | the oracle patch and a first look at a few runs; not registered |
| `run.sh` | rebuilds it all |
