# The demo world's second pass: horses and fodder (stage v2a.1)

Dated 2026-09-30. Written at step D2.0 (label `design`) on branch `demo-v2`, from `reboot` at
`f7d1eae`. Docs only: no code, table or tape changes at D2.0. WORLD.md §11 points here; this file is
separate because WORLD.md would pass 800 lines with it (decision 342).

**The request (2026-09-27).** A nice map of the United Kingdom with Victoria-style lenses over a
fairly complex world of regions, goods and history, "not necessarily perfectly accurate but enough
to get a feel for it", with colours that change as the simulation runs; and an intuitive production
chain in which goods use other goods, with inventories. Demo v1 (WORLD.md) gave the map, 25 lenses
and 150 years of history over a flow machine. This pass gives each county a concrete chain: land
grows fodder, fodder feeds horses, a maker breeds horses, a capacity desk holds the herd and hires
out horse-days, and the good desk buys horse-days and labour.

What this file decides, each numbered and open to veto (§12, decisions 320–342):

1. the county economy under GOODS-CHAIN's rule A, and what each of v1's ramps moves in it (§2, §3);
2. funding, checked on all 93 counties with oracle unit 1g (§4);
3. genesis (§5);
4. what the map will show over the history, from the mirror (§6);
5. the lenses (§7), and how they make capital's lag legible (§8);
6. the compiler and the tape (§9);
7. what is left of O26 and O39 (§10);
8. the battery, county by county, and the long run's checks, with predictions registered before any
   engine run (§11; [v2/registration.md](v2/registration.md)).

Nothing here is research. Every number in the world stays `Assumed("illustrative demo …")`, and the
v2 tape's name carries `[illustrative]`, so nothing it produces is scored or cited (R4, R5, decision
131). The battery and the long run check the agents against their own oracle; they do not score
history.

## 0. The pass in one page

- **Stage v2a.1 on every county** (D-G12, decision 320). Each of the 93 counties keeps v1's row and
  v1's history. Its flow machine becomes rule A's horse: fodder from land alone, a head bred from
  the maker's own horse-days, labour and pasture, worn at 8% a year, hired out wet by the horse-day.
  At ρ = 0 the chain's equilibrium is v1's county exactly (the collapse, R1b), at every one of the
  25,480 step dates: 1g and 1a agree within 8e-16 on every price and quantity and 5.3e-15 on the
  provider's baskets (§4).
- **Funding.** Every county is funded at genesis and at every step date, least at Surrey in April
  1834 (0.119 of a basket per unit of N, v1's figure). Of the battery's cost targets at the six
  dates, b × 1.1, × 0.9 and × 0.5 are funded at all 558 county-dates; b × 2 at 241 of 558 (12 of 93
  counties in 1750, 72 of 93 in 1875 and 1900). The compiler refuses an unfunded date with its path;
  the battery drops an unfunded target and lists it (§4).
- **What the map will show.** The mirror runs each county's history on the stage's roles (§6). No
  market goes dead, the horse market never idles, and the maker never withholds. But capital lags.
  The median county's median gap to its moving equilibrium is D̂ 117 (0.117 in log), and the coal
  and cotton counties' 0.33–0.38 (Lanarkshire 383, Lancashire 377, Glamorgan 362, Durham 331),
  against v1's 11 and 34. A like-for-like flow county at the same dials runs at D̂
  28. Horses fall to 0.72–0.73 of their equilibrium in Glamorgan, Durham and Lanarkshire in the
  1850s and 1860s. Six thinly funded counties (Surrey, Armagh, Caernarfonshire, Sussex, Down,
  Tyrone) have ticks with a transfer shortfall, 14,502 county-ticks in all, where v1 had none. The
  cause is D-G14: a capacity desk's target comes from its coin, so its herd grows only as its
  horse-days' scarcity rent (up to 23% over full cost) accumulates as coin, while v1's history grows
  output by up to 3.3% a year over a decade in the coal and cotton counties (§6).
- **The lenses** (§7). v1's 25 stay, six with a changed reading, and eleven show the chain: horses
  per head, horses against their equilibrium and against the desk's own target, the horse-day's
  price and markup, the horse's price over its replacement cost (never a valuation, and no value on
  a tick when no horse traded), fodder's price, land's shares to fodder and to the working stock,
  the reservation's ticks and the idle horse market's ticks. The gap lens shows capital's lag with
  its cause beside it (§8).
- **The tape** (§9). A new tape, `tapes/demo-gb-v2.ron`, compiled from the same county tables by
  `rustyecon worldgen worlds/demo-gb --stage v2a1`. Without `--stage` the compiler writes v1's tape
  bit for bit; its pin `0xfad880fe08d06645` stays. About 13.6 MB and 33,332 events. The GUI should
  run it to 1901 in about 25 s (v1: 16.1 s, measured today).
- **The battery** (§11). P2.2a's 93-run battery at every county's instance on 1 January 1750, 1800,
  1825, 1850, 1875 and 1900, funded targets only, L 84,000: 51,260 scored runs, and 634 unfunded b ×
  2 runs dropped and named. The mirror's predictions are registered in
  [v2/registration.md](v2/registration.md) before any engine code of this pass exists.
- **Next pass.** Stage v2a.1b, rule B's loop with plants (LOOPS.md), joins no county here: decision
  308 needs a funded county and its own registration for each (O83).

## 1. What changes, and what does not

- **v1 stays.** `worlds/demo-gb/`'s counties, regions (but one reserved column, §9), history and
  lenses, `tapes/demo-gb.ron` and its pin `0xfad880fe08d06645` (hash stream `0xdb63cc96f769fb3e`)
  are unchanged. The compiler without a stage is v1's code path and writes v1's tape bit for bit
  (R1; decision 327).
- **One county table.** v2 reads v1's `counties.csv`, `regions.csv` and `history.csv`. It adds a
  machine type table, a stage table and a lens table (§9). D-G12's four new county columns (`farm`,
  `wood`, `ore`, `water`) wait for the stages that use them (v2a.3, v2a.4).
- **The roles** are P2.2a's, unchanged: P2.0's `GoodDesk` with `assign: ExPost`, the stock kinds
  `CapacityDesk` and `Maker` (with the reservation, L0.4), P2.1's `TypeDesk` for fodder and its
  `BasketProvider` and `BasketWorkers` (HORSES-RULES §2–§3). No role, market or engine code changes
  for the stage.
- **The dials** are C2g, held exactly, and the clock is held to 52 ticks a year (O39; decisions 121,
  237).
- **No trade.** Each county is still its own node, with no channels. The gap lens still compares
  each county with its own oracle.

## 2. The county economy under rule A

### 2.1 Each county's chain

GOODS-CHAIN §5's rule A, applied to each county's v1 row (N, T, h, η, g0, g1, k, a, λ, b, χ_max) at
the params in force (decision 322):

| | rule | at v1's base county (a 0.3, λ 0.03, b 0.8) | unit |
|---|---|---|---|
| fodder | ω·b land per unit, from land alone (no labour, no horse-days: no loop) | 0.68 | land per unit |
| a horse-day | one unit of fodder; no labour | 1 | fodder per horse-day |
| a head (the build) | a·κ/δ of the maker's own horse-days, λ·κ/δ labour, (1 − ω)·b·κ/δ pasture | 187.2, 18.72, 74.90 | per head |
| wear | δ = 8% a year, `Clock::fraction`: 1 − 0.92^(1/52) = 0.0016022 a tick | | FractionPerYear |
| hours | κ = 52 horse-days a head a year, 1 a tick | | FlowPerYear |
| ω | 0.85, the chain's (GOODS-CHAIN §0), P2.2a's F5 and F6 | | |
| build lag, interest | J_b 1 tick; ρ = 0 | | |

At ρ = 0 the fold of this chain per horse-day is (a, λ, b) again: a·κ/δ own horse-days worn at δ/κ a
horse-day give a; λ·κ/δ labour gives λ; fodder's ω·b and the pasture's (1 − ω)·b·κ/δ worn give b. So
the chain's equilibrium is v1's county (unit 1g; HORSES-RULES §6.2,
`horses_oracle_is_the_flow_county_at_rho_zero`). P2.2a made this economy GO on v1's base county at δ
8% and 10%, ω 0.85, on its funded targets (F5, F6; HORSES.md §0), and IDLE made the reservation GO
on both (IDLE.md §0).

**Why δ is 8% everywhere.** It is F5's, the verdict δ of CHAIN's horse (decision 255), and a horse's
working life of about twelve years. No county's history says otherwise in this stage: the one
durable good stands for the county's whole working stock (decision 321), and engines at 5% come with
their own good at v2a.4. δ 10% (F6) is the named alternative. A county whose δ moved over its
history would need the recipe's κ/δ terms stepped with it; no county needs it.

**Why ω is 0.85 everywhere.** It is the chain's share of a horse's land that is in its fodder
(GOODS-CHAIN §0: about 0.84 for CHAIN's horse), and F5's. At ρ = 0 ω moves no equilibrium, only
paths, and ω ½, 0.85 and 1 are all GO in P2.2a.

### 2.2 The roles per county

Every county is a node `county.<c>`, quoting in `coin`, with six actors (v1 had four). Keys follow
P2.2a's tapes (decision 321), so that `traction` sorts after `labour` (HORSES-RULES §8 item 1):

| actor | kind | class | buys | sells |
|---|---|---|---|---|
| `county.<c>.desk.good` | `GoodDesk`, `assign: ExPost` (D-G6) | `good_desks` | labour, traction | good |
| `county.<c>.desk.capacity` | `CapacityDesk` (M3 wet), order `Target`, s_K = 2δ | `capacity_desks` | fodder, horses | traction |
| `county.<c>.desk.maker` | `Maker` (M2), cover 4 weeks, reservation ψ 0.25 | `maker_desks` | fodder (its herd's), labour, land | horses |
| `county.<c>.desk.fodder` | `TypeDesk` (P2.1), own input 0 | `fodder_desks` | land | fodder |
| `county.<c>.provider` | `BasketProvider` (P2.1), basket [(good, 1), (land, h)] | `owners` | good, land | land (its endowment T) |
| `county.<c>.workers` | `BasketWorkers` (P2.1), the same basket | `workers` | good, land | labour |

Goods (seven): `coin`; `labour` and `land`, `Instant`; `fodder`, `traction` and `good`, one tick;
`horse`, `Indefinite`, worn by `Depreciation` burns (D-G2). Classes are shared across nodes, as v1's
are; a rationing line names its node.

**Params.** Shared once: C2g's dials and the stock dials (§2.3), `life.one_tick`, the horse's
`horse.kappa` (52, FlowPerYear), `horse.delta` (0.08, FractionPerYear), `horse.run.fodder` (1),
`horse.run.labour` (0), `fodder.own` (0), `fodder.labour` (0) and `good.weight` (1). Per county
(twelve, 1,116 in all): v1's `workers`, `land`, `space`, `eta`, `g0`, `g1`, `k`, `chi_max`, and rule
A's `fodder.land`, `horse.own_hours`, `horse.labour` and `horse.land`, each keyed
`county.<c>.<name>`. v1's `a`, `lam` and `b` are not registered: no rule reads them (decision 328).
The recipe's coefficients are computed per tick at the tape's tick length, in the order
`probe::horses::instance::Instance::rule_a` computes them, so they are the harness's bit for bit
(decision 329; this closes O32 for rule A).

### 2.3 The dials

C2g (D-G13) as P2.2a registered it, held exactly by the compiler as v1 holds C2 (decision 324):
labour's, fodder's, the horse's and the horse-day's price rates 5.2 a year, land's 0.1625, the
good's 2.6; the technique 2.6; every desk's turnover 5.2 and tilt 0; household spending 13;
`price.ema_tc` 0.5 years. The stock dials: the capacity desk's s_K = 2δ = 0.16 a year, the maker's
s_Km 0, its cover 4/52 years (4 ticks), its reservation ψ = 0.25 (decision 253). The one-sided rule
is `Saturate`. The clock is 52 ticks a year from 1750-01-01; the history runs to 1901-01-01, 7,852
ticks.

## 3. The history on the chain

v1's 605 ramps compose and step exactly as WORLD.md §4.3 says, on v1's params, so every step date
and value is v1's. The compiler then maps each step to the coefficients whose fold is that param
(decision 323):

| v1 param | its ramps | what it moves in v2a.1 | how it reads now |
|---|---|---|---|
| N (`workers`) | `population.*` | N, as v1 | people: more hands, and more baskets to fund |
| T (`land`) | `sites.*`, enclosure, improvement, potato ground, fen drainage, clearances, kelp, mines, slate, coal's mineral rent | T, as v1 | the land every use draws on: homes, fodder fields and horse pasture |
| h (`space`) | `improvement.rotation`, `improvement.high-farming` | h, as v1 | a basket's own land |
| η (`eta`) | `textile.*`, `threshing`, `mechanisation.general` | η, as v1: the horse-days a task needs, γ(x) = η(g0 + g1·x) | machines take over tasks |
| χ_max | `poor-law.*` | χ_max, as v1 | the work cost against dependence |
| b | `steam.coalfields`, `canals`, `railways.gb`, `railways.ireland` | `fodder.land` = ω·b and `horse.land` = (1 − ω)·b·κ/δ, together | coal and carriage stand in for the land that feeds and breeds the working stock |
| λ (`lam`) | `machine-tools`, `engineering.spread` | `horse.labour` = λ·κ/δ | tools cut the labour it takes to raise and break a head |
| a | `railways.own-input` | `horse.own_hours` = a·κ/δ | cheaper carriage: fewer of the maker's own horse-days per head |

A b step becomes two `SetParam`s on the same date; every other step one. v1's 30,078 steps (workers
10,589, land 8,766, b 3,254, λ 2,614, η 1,868, space 1,074, a 1,023, χ_max 890) become 33,332 events
on the same 25,480 county dates.

**Why this mapping.** It keeps ω fixed, so every step leaves the county near the economy P2.2a
tested (ω 0.85), and at ρ = 0 each date's oracle is v1's, so v1's funding, its bounds and its lens
ranges carry over unchanged (§4). The history still reads as v1's: the steam and railway ramps still
cheapen the machine side through its land, and the textile ramps still move the task line. What it
cannot say yet is that steam replaced horses: one durable good stands for the whole working stock
until v2a.4 gives engines their own good and recipes (G3). The named alternative moves fodder's land
alone, so coal replaces oats while pasture stays; ω then drifts to about 0.72 in the coal counties,
and fodder's factor must be rescaled at every date to keep v1's point.

**The bounds hold.** The compiler bounds each date's move of the oracle's relative prices, technique
and quantities at `max_step` 0.03 in log, and each trailing year's at 0.1 (WORLD.md §4.4). v1's list
is unchanged by the collapse. The chain's own readings, which v1's list does not cover, move at most
0.021 at one date (the horse's price p_K/r, East Lothian, May 1873), 0.022 (heads and fodder made,
Gloucestershire, May 1848) and 0.010 (fodder's price, Monmouthshire, April 1770), and 0.048 over a
trailing year (heads, Lanarkshire, to June 1845). The v2 compiler bounds them too (decision 330).

## 4. Funding (O43)

**The check.** The provider must fund one basket for every potential worker out of rent: its own
baskets T/P_s − N must be positive (the oracle's `funded`). v1 checked it at genesis and every step
date (WORLD.md §3.3), but its battery ran b × 2 without checking it, and v1's base county is
unfunded at b × 2 (O43). The stage checks it with unit 1g's `ChainEconomy` (the horse's hours at J =
2, decision 232) at three kinds of point (decision 326):

1. every county at genesis and after every step date: the compiler's check, as v1's;
2. every battery cost target: b × 1.1, × 0.9, × 2 and × 0.5 at each county's instance on 1 January
   of 1750, 1800, 1825, 1850, 1875 and 1900;
3. the long run's own points are those of item 1.

**What the compiler does.** An unfunded genesis or step date does not compile. The error names the
county and date, `county.<c> at <YYYY-MM-01>: the provider cannot fund one basket per potential
worker (T·r ≤ N·P_s)`, as v1's does. There is no named adjustment: an unfunded date is a table to
fix. The battery is not the compiler's. Its harness drops an unfunded cost target from a
county-date's run list and names it (county, date, target), as P2.2a dropped F5's and F6's b × 2
(HORSES-RULES §5). It never substitutes another target.

**Run on all 93 counties** (2026-09-30; scratch `D:/rustyecon-d2/design/fund/`, `d2-fund`, a
standalone crate on this worktree's oracle, probe and worldgen at `f7d1eae`; 25,573 solves in 1.4 s,
and 2,232 target solves):

| points | solves | interior | funded | least baskets per unit of N (where) |
|---|---|---|---|---|
| genesis | 93 | 93 | 93 | 0.1785 (median 0.557, most 1.334) |
| every step date | 25,480 | 25,480 | 25,480 | 0.1190 (Surrey, 1834-04), v1's to the digit |
| b × 1.1, six dates | 558 | 558 | 558 | 0.0498 (Surrey, 1825) |
| b × 0.9 | 558 | 558 | 558 | 0.229 (Surrey, 1825) |
| b × 0.5 | 558 | 558 | 558 | 0.927 (Surrey, 1825) |
| b × 2 | 558 | 558 | **241** | −0.348 (Armagh, 1825); most 2.958 (Lancashire, 1900) |

b × 2 by date: unfunded at 81 counties in 1750, 71 in 1800, 68 in 1825, 55 in 1850, 21 in 1875 and
21 in 1900. Output per head rises over the history, so rent covers more of the transfer, and the
target funds in more counties. Of the 241 funded b × 2 targets, 39 have a margin below 0.05 baskets
per unit of N and 16 below 0.02 (least 0.0017, Clackmannanshire 1825); they stay in the battery, and
their transfer shortfalls are reported (O84). The lowest history margins are Surrey 0.119 (1834-04),
Armagh 0.123 (1825-12), Down 0.159 (1825-07), Caernarfonshire 0.165 (1834-01) and Anglesey 0.176
(1835-01): London's ring, pre-Famine Ulster and north-west Wales, v1's heavy relief burdens
(WORLD.md §5.1).

**The collapse, checked at every point** (R1b): 1g against v1's 1a point at genesis and every step
date, largest relative difference: x\* 1.7e-16, v 7.6e-16, P_s 6.7e-16, Y 8.0e-16, N_a 7.2e-16, p
7.7e-16, the horse-day's price against p_m 4.4e-16, the tasks' horse-days against Y·J(x\*) 6.1e-16,
the provider's baskets 5.3e-15. Every target solve is interior.

The per-county table (margin in 1750, the least over the history and where, in 1901, the least at b
× 1.1, and the dates at which b × 2 is funded) is `v2/funding.csv`, and every target's margin
`v2/targets.csv`, both registered with the predictions.

## 5. Genesis

Every county starts at its own oracle point, 1g's `ChainEconomy` at its genesis instance (J = 2; at
ρ = 0 every price and quantity is J = 1's), under the horses harness's genesis rule (HORSES-RULES
§4, "Genesis"; decision 325):

- prices relative to r = 1: w, p_f, p_K, p_h and p; the good desk's human share 1 − x\*;
- the good desk holds one tick's good Y; the capacity desk its heads H = Y·J(x\*)/κ and κ·H
  horse-days; the maker its record own = (1 − δ)·a_I·q_b/κ and a holding of own + q_b +
  b_K·q_b·c_m/p_K (the cover); the fodder desk one tick's fodder q_f;
- each actor's stationary coin under the dials, summed in the order its rule sums it.

Then every price and coin is divided by p/r, so the good costs 1 coin in every county, as in v1
(WORLD.md §1). The roles are homogeneous of degree zero in prices and coin, so this changes no real
quantity; the mirror, which works at r = 1, agrees with the engine to rounding, and the trace diff
seeds the mirror from the tape's own genesis (§11). Each county's actors carry a comment with its
point. The test `v2_genesis_is_the_horses_rule_at_each_county` holds each county's genesis to
`probe::horses::setup::genesis` on the same instance, divided by p/r, to 1e-14.

The mirror checks the rest point (§11): three ticks from each county's genesis move no coordinate
more than 9.9e-16, and the mirror's oracle (unit 1a on rule A's fold) meets 1g's point within
1.3e-15 at every county's genesis.

## 6. What the history will look like: the mirror's long run

The mirror (§11.1) runs each county's history, 1750-01-01 to the first tick of 1901 (7,852 ticks),
on the stage's roles, and scores it every tick against its own oracle point at the params in force,
as the demo's long run is scored (WORLD.md §8), with P2.2a's observables. The counties do not trade,
so the 93-node run is 93 single-county runs. D̂ is the largest |ln(o/o\*)| over the observables
without the horse market's volume (which idles by design, O80), in units of 1e-3.

**First, the mirror reproduces v1.** With the stocks layer off (the horse lives one tick, δ = 1 a
tick, ω 0, the flow path, which is P2.1's I0 and so P2.0's roles bit for bit), under C2 with planned
assignment, the mirror gives v1's committed long run: the county medians' median 10.8 (v1 10.8), the
highest county median Glamorgan's 34.1 (v1 34.1), the largest D̂ 56.3 at Glamorgan (v1 56.3), the
lowest cleared volume 0.945 at Glamorgan (v1 0.945), and no dead tick or shortfall. So the history's
composition, its timing and the scoring are the engine's.

**Like for like** (decision 235; conditions: 93 counties, 7,852 ticks, D̂ as above; "every 13th"
reads each county's D̂ every 13th tick):

| run | county-ticks median D̂ (every 13th) | county medians: median | highest county median | largest D̂ | dead ticks | shortfall ticks | lowest cleared / target |
|---|---|---|---|---|---|---|---|
| v1: flow, C2, planned | 11.7 | 10.8 | 34.1 (Glamorgan) | 56.3 | 0 | 0 | 0.945 (Glamorgan, good) |
| the flow county at C2g, ex post | 30.6 | 28.4 | 123.7 (Glamorgan) | 207.8 | 0 | 0 | 0.812 (Glamorgan, machine services) |
| **v2a.1** (rule A, C2g, ψ 0.25) | **126.7** | **116.7** | **382.7 (Lanarkshire)** | **543.5 (Glamorgan)** | **0** | **14,502** | **0.606 (Glamorgan, horse-days, 1863)** |

C2g alone makes the flow county's history about 2.6 times as far from its moving equilibrium as
v1's, and stocks another 4.1 times. That is the direction HORSES.md §4 found for a single shock (C2g
about 3 times slower, stocks 2.3–2.8 times more), now over a history.

**v2a.1, county by county** (the per-county table is `v2/long-run.csv`, registered):

| county | v1 D̂ median | flow at C2g | v2a.1 D̂ median | v2a.1 largest | heads / equilibrium, lowest (year) | in 1901 | shortfall ticks |
|---|---|---|---|---|---|---|---|
| Middlesex | 20.3 | 75.4 | 179.2 | 434 | 0.764 (1827) | 0.870 | 0 |
| Lancashire | 28.8 | 106.3 | 376.9 | 531 | 0.743 (1851) | 0.839 | 0 |
| West Riding | 22.0 | 60.5 | 279.6 | 417 | 0.804 (1858) | 0.869 | 0 |
| Durham | 29.4 | 99.1 | 330.8 | 512 | 0.728 (1863) | 0.791 | 0 |
| Glamorgan | 34.1 | 123.7 | 361.9 | 544 | 0.717 (1863) | 0.774 | 0 |
| Lanarkshire | 30.7 | 115.9 | 382.7 | 542 | 0.725 (1851) | 0.819 | 0 |
| Norfolk | 9.3 | 24.0 | 101.2 | 301 | 0.867 (1851) | 0.997 | 0 |
| Wiltshire | 6.5 | 12.0 | 72.3 | 219 | 0.921 (1849) | 0.999 | 0 |
| Cornwall | 12.0 | 44.5 | 184.4 | 326 | 0.847 (1851) | 1.033 | 0 |
| Sutherland | 4.7 | 10.5 | 62.7 | 231 | 0.927 (1848) | 1.016 | 0 |
| Antrim | 16.2 | 60.9 | 200.4 | 337 | 0.819 (1830) | 0.945 | 0 |
| Armagh | 13.1 | 54.5 | 143.0 | 259 | 0.850 (1777) | 1.041 | 3,619 |
| Surrey | 25.2 | 114.2 | 281.6 | 444 | 0.765 (1856) | 0.807 | 5,593 |

"Heads" are the capacity desk's and the maker's together, over the oracle's at the params in force.

**What holds.** No market goes dead in any county (labour, land, fodder, horse-days and the good
each clear at least half their oracle volume every tick); the lowest is 0.606, Glamorgan's
horse-days in 1863. The horse market never idles and never goes without an order; the maker's markup
never falls below 0.981, so the reservation never acts (its floor is 0.25). A horse-day always sells
above its running cost (p_h/O at least 1.62), and the herd is used at 98% of its hours or more.

**What lags, and why.** In the counties that grow fastest, the herd runs 25–30% short of its
equilibrium for decades, and prices follow. In Lancashire, at the median tick and in log against the
equilibrium, the wage in land is +0.28, the horse-day's price +0.31 and the good's +0.32; the good
desk's human share is +0.22 (it does more of its tasks by hand); the land cleared is −0.10 (some
land goes unrented) and the good −0.25. Across all counties the gap in the wage in land runs from
−0.03 (Cornwall) to +0.42 (Glamorgan) in log. The mechanism is D-G14's, capital's inertia under the
cash rule:

- the capacity desk's target is K\* = share(v)·C/(κ·O + δ·p_K), so it scales with its coin, and its
  coin scales with its own revenue, which is its current herd's hours; near a steady coin its target
  is about its herd times one plus the horse-day's markup over full cost;
- the herd grows only as that markup, the scarcity rent, is kept as coin. It reaches +23%
  (Glamorgan), and the desk still holds 0.87 of its own target or more;
- v1's history grows the oracle's output 2.2–2.4% a year on average in Glamorgan, Lanarkshire and
  Lancashire (36-fold in Lancashire between 1750 and 1901), and up to 3.3% a year over a decade; the
  median county 0.7% a year, and 1.7% in its fastest decade. That is faster than the rent can build
  the herd in the industrial counties. Where output falls or its mix moves (Sutherland grows 0.3% a
  year), the herd lags less but still lags: D̂ 63 at Sutherland's median.

The agents have no credit (Phase 8), and entry, the candidate fix, is Phase 3's (D-G14, decision
285). So the demo will show the departure, not hide it (§8; O81).

**Families that do not fix it** (named for the record, not dial sets of this stage): land's price
rate at C2's 1.3 a year instead of 0.1625 leaves the county medians' median at 113.9 (the largest
634) and cuts the shortfall ticks to 4,347; s_K at 4δ leaves it at 117.3 (the herd holds 0.92 of its
own target or more, but the target itself lags). The lag is in the target, not the order.

**The shortfalls.** Six counties have ticks in which the provider cannot pay the whole transfer:
Surrey 5,593 ticks (10.4% of the transfer unpaid on those ticks, on average), Armagh 3,619 (2.4%),
Caernarfonshire 2,320 (4.8%), Sussex 1,479 (2.3%), Down 1,115 (1.2%) and Tyrone 376 (0.4%). They are
the thin-funded counties of §4. With prices off their equilibrium the basket costs more against the
rent roll than the oracle says, and the thinnest margins go below zero. v1 had none. The `shortfall`
lens will show them (O82).

**At the end.** By the first tick of 1901 the median county's D̂ is 60 (0.06 in log from its
equilibrium), and Glamorgan's 362. Counties whose ramps ended early settle: Sutherland's herd ends
at 1.016 of its equilibrium and Wiltshire's at 0.999.

## 7. The lenses

A lens is `{ measure, unit, scale, domain }` with a reference for diverging scales, one row of a
lens table in `lenses.csv`'s format (WORLD.md §6). The v2 tape takes its own table,
`worlds/demo-gb/lenses-v2a1.csv` (decision 332). Palettes stay neutral (viridis; purple to orange
through white), and domains stay fixed for the run and chosen without it (decision 129): from the
oracle's range over every county at genesis and after every step, with a margin, where the oracle
sets one; from the mirror's long run and battery, with a margin, where the rest value is fixed (a
markup is 1 at rest, a ratio to target is 1).

### 7.1 v1's 25

**Kept, with v1's rows unchanged (19):** `wage.baskets`, `wage.goods`, `wage.land`, `rent.goods`,
`share.land`, `share.labour`, `frontier.x`, `participation`, `output.per.head`, `price.good`,
`relief.burden`, `shortfall`, `rationing`, `since.wage.baskets`, `since.output.per.head`,
`param.workers`, `param.land`, `param.eta` and `param.chi_max`. Their measures read the same markets
and states, and their oracle ranges are v1's to the digit (the collapse; the mirror recomputed them:
`wage.land` 0.4787 to 2.0419, `wage.baskets` 0.9468 to 1.2677, `output.per.head` 1.8544 to 7.2062).
`frontier.x` is the good desk's planned share, as in v1, although production now assigns tasks after
the fact.

**Changed (6):**

| lens | v1 | v2a.1 | domain |
|---|---|---|---|
| `price.hday` (was `price.mach`) | p_m/r | p_h/r, the horse-day's price; at rest O + δ·p_K/κ, which is v1's p_m | sequential log, [0.4, 1.4] (oracle 0.432 to 1.230, v1's) |
| `no.trade` | a tick in which one of four markets did not trade | one of the five markets that must trade (labour, land, fodder, horse-days, the good); the horse market is counted apart (`idle.horse`), since its orders stop by design | sequential, [0, 52] |
| `param.b` | b | the fold: `fodder.land` + δ·`horse.land`/κ, land in a horse-day | sequential, [0.3, 0.8] (0.309 to 0.800) |
| `param.lam` | λ | the fold: δ·`horse.labour`/κ | sequential, [0.013, 0.030] (0.0136 to 0.0300) |
| `gap.oracle` | D̂ over v1's ten observables, [0.1, 100] | D̂ over the stage's 17 (P2.2a's 18 without the horse market's volume), unit 1g at the params in force | sequential log, [1, 1000] (the long run reaches 544) |
| `gap.wage` | ln((w/r)/v\*), [−0.06, 0.06] | the same measure | diverging, [−0.5, 0.5] (the long run: −0.031 to +0.416) |

**The oracle lenses come forward** (decision 331). `gap.oracle`, `gap.wage` and `horses.vs.oracle`
need unit 1g at each county's params in force. worldgen gains `oracle_gap`, beside the lens measures
(decision 128): it solves 1g's `ChainEconomy` outside any `Sim` when a county's params change, and
the GUI and the long-run test both call it. Nothing it returns reaches an agent (R13); the GUI
observes (R16). It is `crates/observe`'s `oracle_gap` in waiting, as the lens measures are its
`measure`.

### 7.2 The chain's lenses (11 new)

In `lenses.csv`'s format
(`key,name,measure,unit,scale,reference,domain_lo,domain_hi,inputs,source,note`):

```
horses.per.head,Horses per head,(CapacityState.held + MakerState.serving) / N_tick,heads per unit of N,sequential-log,,1.0,10.0,Sim::actor_state(county.C.desk.capacity; county.C.desk.maker); Sim::param(county.C.workers) per tick,extractor+param,the working stock the county holds; oracle 1.31 (Dunbartonshire 1900) to 8.25 (Northumberland 1900)
since.horses.per.head,Horses per head since 1750,ln(horses.per.head / its genesis value),log difference,diverging,0 = the county at genesis,-1.5,1.5,as horses.per.head,extractor+param,oracle -0.68 to 0.83
horses.vs.oracle,Horses against their equilibrium,ln(heads / heads* at the params in force),log difference,diverging,0 = the oracle,-0.5,0.5,as horses.per.head; oracle unit 1g at the county's params in force,observe,capital's lag (D-G14): the long run reaches -0.33 (Glamorgan 1863); never fed back (R13)
horses.vs.plan,Horses against the desk's own target,ln(CapacityState.held / CapacityState.target),log difference,diverging,0 = its target K*,-0.3,0.3,Sim::actor_state(county.C.desk.capacity),extractor,the desk's order closes 2 delta of this gap a year; the long run: -0.14 to +0.06
price.fodder,Fodder's price in rent,p_f / r,land-service units per unit of fodder,sequential-log,,0.2,0.8,MarketLine.price (fodder; land),report,at rest omega*b: oracle 0.2625 to 0.680
price.horse,Horse price over its replacement cost,p_K*(1 - delta*a_I/kappa) / c_m at posted prices (the maker's markup),multiples of the replacement cost,diverging,1 = the replacement cost,0.0,2.0,MarketLine.price (horse; fodder; labour; land) and a trade this tick; Sim::param(county.C.horse.*; horse.*),report+param,no value on a tick when no horse traded (an idle market's price is not a valuation: O47; O54); the maker withholds below psi 0.25; its lowest is 0.981 in the long run and 0.212 in the battery (the reservation's floor)
hday.markup,Horse-day markup over full cost,p_h / (O + delta*p_K/kappa) - 1,share,diverging,0 = the full cost,-0.3,0.3,MarketLine.price (traction; fodder; horse); Sim::param(horse.*),report+param,the scarcity rent that builds the herd (D-G14's quasi-rent); the long run: -0.076 to +0.233
land.to.fodder,Land to fodder,fodder.land * TypeDeskState.output / T_c,share of land cleared,sequential,,0.4,0.8,Sim::actor_state(county.C.desk.fodder); Sim::param(county.C.fodder.land); MarketLine.cleared (land),extractor+param,oracle 0.484 (West Riding 1901) to 0.728 (Norfolk 1801)
land.to.horses,Land to the working stock,(fodder.land * fodder made + horse.land * heads made) / T_c,share of land cleared,sequential,,0.5,0.9,as land.to.fodder; Sim::actor_state(county.C.desk.maker); Sim::param(county.C.horse.land),extractor+param,fodder and pasture together: oracle 0.570 to 0.857; the rest is homes (h per basket)
reserve.ticks,Ticks the maker withheld,ticks in the trailing 52 in which the maker's markup was below psi,ticks per year,sequential,,0,52,as price.horse; Sim::param(reserve.maker),report+param,the reservation (IDLE-SPEC; decision 253); 0 in every county on the history (section 6)
idle.horse,Ticks the horse market was idle,ticks in the trailing 52 in which no horse traded,ticks per year,sequential,,0,52,MarketLine::trades (horse),report,O80's idle ticks: orders stop in a glut by the order rule's design; 0 on the history
```

`extractor+param` is a new source (an actor's state and a param together), which the table's check
learns at D2.3. The v2 table is v1's 25 rows, six of them changed as §7.1 says, and these 11: 36
lenses. The opening lens stays `wage.land`, the paper's v; the selector lists the chain's lenses as
a group, "Horses and fodder", first.

**The horse's price** (decision 333) is shown only as a ratio to what it costs the maker to replace
a head at posted prices, never as p_K/r, and only on a tick when a horse traded. On a tick with no
trade the county has no value and is painted in the map's no-value grey, and its card says "idle: no
horse traded; the last posted price was X of replacement cost". ψ = 0.25 is marked on the legend. A
price that rests for years where orders stopped is an idle market's (O54), not a value of the herd.

**Where each comes from.** Every new lens reads what the engine reports or the actors' own states,
through the Extractor's lean catalogue, which records states already; the markups are computed from
posted prices and params by the rule's own formula (`maker_reservation`), as the harness reads it
(decision 279). `horses.vs.oracle`, `gap.oracle` and `gap.wage` need `oracle_gap` (§7.1). worldgen's
`Readings` holds four prices and four volumes today; for the stage it holds each market's by key,
and the six actors' states, and `CountyKeys` is the stage's. The GUI takes the lens table by the
tape's name (`lens::for_tape`), v1's for `demo-gb [illustrative]` and v2's for `demo-gb-v2
[illustrative]`; today it always takes v1's.

**Held by tests** (at D2.3): `lens_v2_domains_hold_the_oracle_range` (every level lens whose oracle
sets a range, at every county date); `demo_v2_lens_values_equal_the_engine` (every lens for every
county at report ticks 0, 51 and 259 against the engine's own numbers, the two markups against the
maker's and the capacity desk's decisions); `price_horse_has_no_value_on_an_idle_tick` (a county
whose horse market did not trade); the mirror's long-run ranges above as the domains' check on the
dynamic lenses.

## 8. Making capital's lag legible (D-G14)

Capital takes 30–80 years to settle after a shock (HORSES.md §4), and §6 shows a history that keeps
shocking it. So the gap lens will be orange over the industrial counties for most of the nineteenth
century. The design makes that read as "short of horses, catching up", not as "failing" (decision
334):

1. **The gap lens names its bands.** `gap.oracle` runs on a log scale from 1 to 1,000 with the
   legend's marks at 1 (the probe's tolerance), 10 (1% in log), 100 (10%) and 1,000, each named. In
   the mirror half the counties' medians lie between 90 and 177 (53 of 93 between 50 and 150, four
   above 300), and the legend says so: "most counties run 5–20% from equilibrium in log; capital
   settles over decades (D-G14)".
2. **The card names the cause.** A selected county's card lists the three observables that carry its
   gap, with their signs (in Lancashire around 1850, the horse's and the horse-day's prices 0.3–0.4
   above their equilibrium in log, the heads 0.3–0.4 below), beside the three stock readings: horses
   against their equilibrium, against the desk's own target, and the horse-day's markup. A county
   short of horses shows a positive markup: the rent that is buying more.
3. **The stock has its own lenses.** `horses.vs.oracle` (diverging, short in purple, over in orange)
   shows where the herd lags; `horses.vs.plan` shows that the desk is ordering toward its own
   target; `hday.markup` shows the rent; `since.horses.per.head` shows the herd growing. Seen
   together, the herd lags because the target lags, not because the market fails.
4. **The chart shows the catch-up.** The map's chart for a selected county draws the lens over the
   run. On `horses.vs.oracle` it shows the herd falling behind as each ramp runs and closing after
   it ends (Sutherland ends at 1.016, Wiltshire at 0.999).
5. **Neutral colours, no alarm.** No red, no green, as v1 (decision 129). `shortfall` stays its own
   lens; `no.trade` and `idle.horse` stay 0, and the map shows that too.
6. **What stays out.** No lens scores the lag against the paper's time (D-G14's readouts are the
   harness's, not the map's), and no lens is dropped because it is uncomfortable: the gap lens was
   disabled in v1 only because the oracle was not wired.

## 9. The compiler and the tape

### 9.1 Tables

| file | status | what |
|---|---|---|
| `counties.csv`, `history.csv`, `lenses.csv`, `world.csv` | unchanged | v1's |
| `regions.csv` | one column | `machine_types` holds `horse` where it held `mach` (`derive.py` writes it; `--check` compares) |
| `machine_types.csv` | new | one row: `horse`, rule A, δ 0.08 a year, κ 52 a year, ω 0.85, J_b 1 tick, and a note |
| `stage-v2a1.csv` | new | the stage's settings in `world.csv`'s format: the tape's name `demo-gb-v2 [illustrative]`, its basis marker, C2g's dials, the stock dials (s_K, s_Km, cover, ψ), `ExPost`, the lens table's name |
| `lenses-v2a1.csv` | new | §7's 36 rows |

**The reserved column** (WORLD.md §7; decision 327): `machine_types` names the county's machine
type, now `horse`, keyed into `machine_types.csv`. Without a stage the compiler compiles a `horse`
county as v1's flow machine, which is rule A's collapse at ρ = 0 exactly, and the tape it writes
does not name the column, so v1's tape is unchanged. With `--stage v2a1` it writes the horse as a
stock. `categories` stays `good` and `carriers` empty, and any other value is still refused.

### 9.2 The compiler

- **Entry.** `rustyecon worldgen worlds/demo-gb [--stage v2a1] [--out PATH]`. `compile(&Tables,
  &Atlas)` stays v1's; `compile_stage(&Tables, &Stage, &Atlas)` is new. The cli reads the stage's
  three files only with `--stage`.
- **Checks, before a line is written.** v1's (WORLD.md §8) and, for the stage: the clock is 52 ticks
  a year (O39, in v1 too); the dials are C2g exactly (`tables::C2G` held to
  `probe::horses::setup::dials` on F5 by test); δ in (0, 1), κ positive, ω in [0, 1), ψ in (0, 1);
  every county at genesis and after every step date solved by 1g, `Interior`, one crossing on the
  grid (at ρ = 0 1g's excess demand is 1a's on the fold), funded, participation short of saturation;
  the relative prices and technique, the quantities, and the chain's own p_K/r, p_f/r, heads and
  fodder each within `max_step` at a date and `MAX_YEAR` over a trailing year; the collapse (1g
  against 1a within 1e-13 relative on x\*, v, P_s, Y, N_a and p, at every point).
- **The mapping** (O32, for rule A). `worldgen::chain` builds rule A's `GoodsChain` per county per
  tick, from the clock (`Clock::flow`, `Clock::fraction`) and the county's params, and the recipe
  coefficients in the order the harness computes them. `rule_a_is_the_probes` holds it to
  `probe::horses::instance::Instance::{rule_a, chain}` bit for bit on F5, F6 and ten counties.
- **The writer** emits §2.2's goods, nodes, classes, actors, params and genesis, and each b step as
  two schedule params and two events (`county.<c>.fodder.land.<YYYY-MM>`,
  `county.<c>.horse.land.<YYYY-MM>`), each basis naming the ramps that moved it, as v1's do.

### 9.3 R1: v1 nests

- `demo_tape_is_its_compilers_output` holds v1's tape, and `demo_runs_to_1901` its final hash
  `0xfad880fe08d06645` and stream `0xdb63cc96f769fb3e`, as now.
- `v2_flow_path_is_v1` (new): the stage with the horse on its flow path (δ = 1 a tick, ω 0, planned
  assignment, C2) runs every county's prices, cleared volumes and coins equal to v1's bit for bit
  for 520 ticks. The actor kinds differ, so the hashes do; P2.2a's R1a (P2.1's I0 bit for bit) and
  `markets_i0_nests_appb` are the reason it should hold.
- Every committed tape's text and hashes stay: gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
  the probe's, the markets probe's, the horses and loops tapes' streams.

### 9.4 The tape, and the GUI's speed

**The tape**, `tapes/demo-gb-v2.ron`, named `demo-gb-v2 [illustrative]`, gets its own pin at D2.2
(decision 339). Estimates from v1's (12,174,574 bytes, 30,078 steps): 33,332 schedule params and as
many events, six actors and twelve params per county, so about 13.6 MB, 0.9 MB compressed.

**Speed, measured today** (Windows, release, this worktree at `f7d1eae`, `D:/rustyecon-targets/d2`;
`D:/rustyecon-d2/design/speed/`):

| | measured |
|---|---|
| v1 in the GUI, `rustyecon-gui --smoke 7852 tapes/demo-gb.ron`, twice | 16.2 and 16.1 s to 1901: 485–488 ticks a second, 9.3 model years a second; CPU per frame running p50 6.53–6.57 ms, p90 7.43–7.47 ms; paused p90 5.83–6.14 ms |
| v1 through the cli, `rustyecon run --until 7852` | 12.5 s wall, loading included; final hash `0xfad880fe08d06645` |
| one county for 52,000 ticks: appb (P2.0's four roles) | 0.70–0.72 s, 73,000 ticks a second |
| one county: `horses-h1` (the stage's six actors and seven goods) | 1.11–1.12 s, 47,000 ticks a second: 1.57 times appb's cost a tick |

**Estimated for v2**, scaling v1's per county-tick cost by 1.57: about 25 s to 1901 in the GUI, some
310 ticks or 6 model years a second, and 19 s through the cli. The lean catalogue grows from 5,970
series to about 9,200 (six markets, 19 state fields and more rationing lines a county), so a run to
1901 holds about 0.58 GB of series, against 0.37 GB. The frame's p90 should stay near G4's 8 ms bar,
since painting does not grow; `oracle_gap`'s 25,480 solves of 1g run as county params change, off
the frame (O85, O86).

## 10. What is left of O26 and O39

| item | status | fix | test |
|---|---|---|---|
| **the clock** held to 52 ticks a year | fixed at D2.1 | the compiler refuses `ticks_per_year` other than 52, in v1 and in the stage, as it refuses dials off their set (decisions 121, 237) | `the_compiler_refuses_bad_tables` gains 12 and 4 a year (at 4 a year v1's tape ran to 46,548 dead county-ticks) |
| V7, the card's value a tick early | closed at G1.6 (decision 215) | the card held to the engine beside the map | `lens_values_equal_the_engine`: every county's card at ticks 0, 51 and 259 |
| V8, the card's change lens against the cursor | closed at G1.6 | the same | the same |
| V9, the app handing the map no cursor | closed at G1.6 | the demo script's cursor behind live | `the_demo_script_switches_lenses_runs_hovers_and_selects` |
| C6, the legend painted reversed | closed at G1.6 | colours read from the painted shapes | `rebuilt_mesh_colours_are_the_lens_colours`: the 64 painted legend segments |
| C7, a fresh mesh's later vertices in a neighbour's colour | closed at G1.6 | every painted vertex read | the same: every vertex of the fill mesh, region by region |
| C8, the recolouring skipping each region's last vertex | closed at G1.6 | the same | the same |
| A2, the credit transparent | closed at G1.6 | the credit read from the painted text | `the_credit_is_painted_whole_on_a_narrow_window`: every line painted and seen |
| A3, the credit off the canvas | closed at G1.6 | the same | the same: inside its clip rect; `the_credit_is_painted_clear_of_the_legend` |
| **the credit's wrap** at 1,024 × 768 | closed at G1.6 | each line wraps to the canvas | `the_credit_is_painted_whole_on_a_narrow_window` at 1,280 × 800, 1,024 × 768, 700 × 768 and 480 × 600 (without the wrap it fails at 700 × 768) |
| **the legend and credit over Cornwall** at the fitted view | fixed at D2.1 (decision 343) | `View::fit` fits the atlas into the canvas less the legend's box and the credit's rect: the fit's rect is the canvas above the legend's top where the map's south-west corner would fall under it, and the credit moves above the legend on a narrow canvas as now | `the_fitted_view_leaves_every_county_clear`: at 1,600 × 900, 1,280 × 800, 1,024 × 768 and 700 × 768, every drawn region's painted rect is disjoint from the legend's and the credit's painted rects, Cornwall and Devon by name |
| **the ranked table's value column** cut under a long lens name | fixed at D2.1 (decision 344): the selector's button also cut its long name short, which had widened the sidebar | the county column is `Column::remainder().clip(true)`, the value column keeps its width, and the header line wraps; v2's longest names ("Horse price over its replacement cost", "Horses against the desk's own target") are the test's | `the_ranked_values_are_painted_whole`: under the longest lens name and unit, at the pane's narrowest width, every painted value text lies inside its clip rect, read from the shapes as the credit test reads them |

G1.6's tests run on v1's demo store. D2.3 adds the v2 store to each, and D2's bounded verification
reruns `verify-map-r2`'s `mutate.py` on the v2 map (decision 338).

## 11. The battery and the long run's checks

### 11.1 The mirror

`D:/rustyecon-d2/design/mirror/` (WSL `/mnt/d/...`), decision 336:

- `model/` and `ag/`: P2.2a's frame mirror (`D:/rustyecon-p2g/frame/model/`, `ag/`), copied
  unedited; `h_mirror.py` is sha256 `042f12b2…428e`, the registered one. The original is not edited.
- `model/i_mirror.py`: IDLE's reservation, copied unedited from `D:/rustyecon-p2l/idle-scan/`.
- `i_carry.py`: written by the registered `make_i_carry.py`, byte for byte
  `D:/rustyecon-p2l/idle-engine/tracediff/i_carry.py` (the reservation and the engine's genesis
  carry, O57).
- The extension, new files only: `d2_world.py` (the 93 counties under rule A at C2g with ψ 0.25,
  from `counties.json`, which `d2-fund` dumps from the compiler's own plans: each county's genesis
  row and every step with the tick it fires in), `d2_long.py` (the long run), `d2_modes.py` (the
  flow controls), `d2_lin.py` (local growth), `d2_battery.py` (the battery), `d2_lens_ranges.py`,
  `d2_steps.py`, `d2_vgap.py`.

**Checked before any prediction was taken:**
- the mirror's oracle against 1g's point at every county's genesis: within 1.3e-15;
- the rest point: three ticks from every genesis move no coordinate more than 9.9e-16;
- the battery's port (`d2_battery.run`, the map `i_carry.tick`) against P2.2a's frame on F5 with the
  reservation and the carry off: 91 of 91 runs equal in class, ticks to tolerance and dead ticks to
  `h_battery_F5.json`;
- the history's driver against v1: the flow path at C2 gives v1's committed long run (§6).

### 11.2 Local stability

At every county's instance on 1 January of the six dates, at its base point and each funded cost
target (2,473 points), the mirror's largest growth of a displacement per tick (P2.2a's PL, four
directions over 100 years after 20) is below 1 at every point. The largest is 0.998487 a tick (0.924
a year), at Brecknockshire's 1850 b × 0.5; at the base points 0.9197–0.9227 a year, the horse's
wear, 1 − δ = 0.92, as HORSES.md found. The rest residual is at most 3.4e-15. So 3·T6 is at most
27,400 ticks everywhere.

### 11.3 The battery (decision 335)

- **Instances:** every county's instance in force on 1 January of 1750, 1800, 1825, 1850, 1875 and
  1900, as v1's §3.3 took them: 558 county-dates.
- **Runs:** P2.2a's battery, HORSES-RULES §5: each price (w, r, p_f, p_K, p_h, p) and the technique
  alone at ±5% (Tier 1), ±20% (Tier 2), ×2 and ×0.5 (Tier 3); JA, JB and N at the same factors;
  x\*/2; b × 1.1 and × 0.9 (Tier 2), × 2 and × 0.5 (Tier 3), at genesis and dated; Tier 3S, each of
  the twelve stocks and coins at × 0.5 and × 2. 93 runs a county-date, less its unfunded targets:
  51,894 listed, of which 634 are unfunded b × 2 runs, dropped and named, and 51,260 are scored.
- **Dials and roles:** the stage's (§2), with the reservation on and the genesis carry.
- **L:** 84,000 ticks, F5's registered L (HORSES-RULES §6.7, set by land's time constant under C2g),
  above the mirror's 3·T6 everywhere (§11.2). Dated shocks at L/4. One length; P2.2a's 10·L reruns
  of Tier 3 and 3S are not repeated (decision 335). Before any scored run the engine's elasticity
  probe runs at each county-date, and if its 200·τ_max passes 84,000 anywhere, a dated amendment
  lengthens L there first (decision 282).
- **Kicks:** each county-date's base kick set (12 kicks of 1e-9, H = L), judged by
  `criteria/appb-2026-09-26.ron`'s bars.
- **Scoring:** HORSES-RULES §5's observables, targets (1g at the coefficients in force), D̂ at tol
  1e-3, classes, dead and idle ticks. A county-date is GO when every one of its runs CONVERGES and
  its kick set decays.
- **On the engine:** P2.2a's harness, `horses`, runs a county-date as an instance whose county is
  that date's v1 row. The compiler writes the 558 rows (`rustyecon worldgen worlds/demo-gb --stage
  v2a1 --instances PATH`), and `horses --counties PATH --inst demo:<key>@<year>` reads them, so the
  probe gains no dependency on worldgen. The job list and the scorer are committed before the first
  job (decision 311). At about 47,000 ticks a second a county, the 51,260 scored runs take about an
  hour on 46 threads.

**The mirror's prediction** (2026-09-30, `D:/rustyecon-d2/design/out-mirror/battery.jsonl`, 46
processes on WSL shared with another branch's scan, 92 minutes; registered as `v2/battery-runs.csv`
and `v2/battery-county-dates.csv`):

- **Every one of the 51,260 scored runs CONVERGES, at every one of the 558 county-dates.** Each ends
  within D̂ 2.4e-10 of its target (2.4e-13 in log). So every county-date is GO in the mirror. The
  634 unfunded b × 2 runs are listed as not a target.
- **Ticks to tolerance**: median 3,010 and 90th percentile 4,526; the largest 5,661 (109 years, the
  West Riding in 1850 after the capacity desk's heads × 2). By tier, median and largest: Tier 1
  1,770 and 3,798; Tier 2 2,852 and 4,629; Tier 3 3,793 and 5,519; Tier 3S 3,403 and 5,661.
- **Dead ticks** come only in Tier 3 and 3S, never in the second half of a run, in 6,479 runs: every
  county-date's r × 2, N × 2, JA × 2, JB × 2 and × 0.5, x\*/2, heads × 0.5 and the fodder desk's
  coin × 0.5; p_f × 0.5 at 436 of 558, b × 0.5 (at genesis and dated) at 408, p × 2 at 299, and five
  other runs at fewer. The most is 277 horse-day ticks (Suffolk 1825, b × 0.5): the herd cannot grow
  fast enough for the cheaper target, as at F5 (253 in P2.2a's frame). By market the most in one
  run: horse-days 277, fodder 46, the good 37, land 21, labour 9.
- **The idle horse market** (O80): idle in 19,589 runs, up to 418 ticks (Lancashire and the West
  Riding in 1900, heads × 2), with up to 88 ticks without an order.
- **The reservation acts** in 2,803 runs: every county-date's r × 2, N × 2, JB × 0.5 and heads × 2,
  all 482 funded b × 2 runs, 55 p × 0.5, 33 x\*/2 and one JA × 2. It withholds for up to 49 ticks
  (Lanarkshire 1900, heads × 2). The horse's lowest price over its target is 0.200 (the West Riding
  1900) and the maker's lowest markup 0.212 (Selkirkshire 1875): the reservation's floor, as IDLE
  found (0.187–0.251). The highest horse price is 7.65 times its target (Lancashire 1900, b × 0.5).
- **Transfer shortfalls** in 5,905 runs, up to 2,142 ticks (Clackmannanshire 1825, b × 2, the
  thinnest funded target, O84); 106 of the 482 b × 2 runs have one.
- **Troughs**: the lowest baskets 0.311 of Y\* (Norfolk 1800, r × 0.5); the heads 0.544 of target at
  their lowest (Norfolk 1800) and 1.90 at their highest (the East Riding 1800, b × 2).
- **Capital's time** (D-G14, unscored): after b × 2 the heads come within 5% of target in 8.0–25.8
  years (median 9.5; the paper's zero builds 7.1); after b × 0.5 in 34.7–40.9 years (the paper: one
  tick); after b × 1.1 in 0.8–2.4 years; after b × 0.9 in 5.0–9.5.
- **Reproducible**: two county-dates rerun (186 runs) give their lines byte for byte.

### 11.4 The long run

The 93-node run from genesis through the first tick of 1901 (7,852 ticks), run by name in
`scripts/gate.sh` as `demo_v2_runs_to_1901`, scored as §6 scores the mirror: every event fired
(33,332), the ledger closed every tick, and per county the dead, idle, no-order, withheld and
shortfall ticks, the lowest cleared volume over its target, D̂ (median, 90th percentile, largest, at
the end), the heads against the oracle (lowest, highest, at the end), the horse price and markup
extremes and the quasi-rent. §6's numbers are the predictions.

### 11.5 Registered, and how the engine is scored against it

[v2/registration.md](v2/registration.md) (sha256
`972c7d21805f5de06ae712b53d2072f278b134c949d2f3769e38dc5fb612a5b0`, its files in `v2/SHA256SUMS`)
registers, before any engine code of this pass, the mirror's sources by sha256, its inputs, the
predictions (the long run per county, the battery per run and per county-date, local growth per
point, funding per county and target) and these rules (decision 337):

- **E0, the trace diff**, before any scored run: five counties (Lancashire, Glamorgan, Surrey,
  Sutherland, Bedfordshire), each from the engine's own genesis on its tape, hold and w × 2 at 1850
  and b × 2 at genesis where funded, 2,000 ticks: every observable within 1e-9 in log, with decision
  304's reading of the horse volume's cancellation. A parting at tick 1 stops the run for an
  amendment.
- **E1, nesting**: §9.3.
- **E2, rest and mode A**: three ticks from each county's genesis within 1e-12 of the oracle; mode A
  at each county-date to L, every observable within 1e-9 in log.
- **E3, the battery**: every run's class the mirror's; ticks to tolerance within 10% of the mirror's
  run for run, but at most three runs a county-date within 25%; dead ticks within 5 or 10%,
  whichever is larger; the lowest baskets within 1e-3.
- **E4, the kicks**: every county-date's base kick set decays, its g below 1 a tick.
- **E5, the long run**: per county, dead and idle ticks the mirror's (0), withheld ticks within 5,
  shortfall ticks within 5 or 1%, whichever is larger; D̂ median and largest within 5%; the heads'
  lowest over the oracle within 1e-3 absolute.
- **Refutation**: a class other than the mirror's at any county-date; any dead tick in the long run;
  a runaway with ψ 0.25; a CONVERGED run ending more than 1e-12 off its oracle point; a kick set
  that grows.

A county-date the engine finds not GO leaves the map as it is (the demo is illustrative), and is
reported with its county, date and run, and a dated open item.

## 12. Decisions 320–342, open to veto

Taken by Claude on your word of 2026-09-26 and 2026-09-27 ("I leave all those calls up to you";
"keep going"). Each names its alternative.

320. **D-G12 taken: the demo's second pass is the goods chain from stage v2a.1, rule A, on every
     county.** Stage v2a.1b (rule B's loop with plants) joins no county in this pass: decision 308
     needs a funded county and its own registration for each, so it is the next pass (O83).
     *Alternative:* O27's first plan, unit 1b's categories and 1c's machine types on the many-market
     roles at C2m.
321. **One durable good, keyed `horse`, stands for the county's whole working stock** until v2a.4
     gives engines their own good; the tape's keys are P2.2a's (`horse`, `traction`, `fodder`,
     `maker`, `capacity`). The map's labels say "horses" and the doc says what they stand for (O87).
     *Alternative:* a neutral name (`stock`, `power`), which reads worse in 1750 and no better in
     1850.
322. **Rule A on each county's v1 row, δ 8% a year in every county, ω 0.85, κ 52 a year, J_b 1, ρ
     0.** *Alternative:* δ 10% (F6), or δ by county.
323. **v1's ramps keep their params, and the compiler maps each to the coefficients whose fold is
     that param** (b to fodder's land and the pasture together; λ to a head's labour; a to its own
     horse-days). At ρ = 0 every date's oracle is v1's. *Alternative:* b's ramps on fodder's land
     alone, ω drifting to about 0.72 and fodder's factor rescaled to keep v1's point.
324. **C2g held exactly and the clock held to 52, in v1 and v2**; s_K 2δ, s_Km 0, cover 4 weeks, ψ
     0.25 (decision 253), `ExPost` (D-G6), fodder's rate 5.2 (decision 226). *Alternative:* a dial
     family as default (land's rate 1.3, or s_K 4δ), which the mirror finds does not shrink the lag
     (§6) and has no probe behind it.
325. **Genesis at 1g's point per county under the horses genesis rule, divided by p/r so the good
     costs 1 coin**, as v1. *Alternative:* r = 1, as the horses tapes.
326. **Funding: the compiler refuses an unfunded date with its path; the battery drops an unfunded
     target and names it; no adjustment.** *Alternative:* the largest funded factor below 2 in place
     of an unfunded b × 2.
327. **One set of county tables; `machine_types` holds `horse`; the stage is three new files and
     `--stage v2a1`; without it the compiler writes v1's tape bit for bit.** *Alternative:* a
     separate `worlds/demo-gb-v2/` with its own copies of the county tables.
328. **Only params a rule reads are registered**: rule A's coefficients per county, not a, λ and b;
     the param lenses for b and λ show the fold. *Alternative:* register a, λ and b as unread params
     beside them.
329. **The rule-A mapping lives in worldgen** (O32 for rule A), held to the probe's bit for bit by
     test. *Alternative:* worldgen depends on the probe crate.
330. **The v2 compiler also bounds the chain's own oracle moves** (p_K/r, p_f/r, heads, fodder)
     under `max_step` and `MAX_YEAR`. *Alternative:* v1's list alone.
331. **The oracle lenses come forward in worldgen** (`oracle_gap`, 1g at the params in force,
     outside the Sim), called by the GUI and the long-run test alike, extending decision 128's
     departure (U6). *Alternative:* wait for `crates/observe`, with the lag shown only against the
     desk's own target.
332. **The lenses: v1's 25 (19 kept, six changed) and 11 of the chain, 36 in all**, domains from the
     oracle's range, or from the mirror's long run and battery where the rest value is fixed, with
     margins; neutral palettes; the opening lens stays `wage.land`. *Alternative:* the chain's
     lenses only for the v2 tape.
333. **The horse's price is shown as its markup over replacement cost, and has no value on a tick
     when no horse traded.** *Alternative:* p_K/r with an idle mark.
334. **Capital's lag is shown with its cause, not scored and not hidden** (§8). *Alternative:* hide
     `gap.oracle` on the v2 tape until D-G14's departure lifts.
335. **The battery: P2.2a's 93 runs at 558 county-dates, funded targets only, L 84,000, one length,
     each county-date's base kick set.** *Alternative:* P2.2a's 10·L reruns of Tier 3 and 3S too,
     about 2.3 times the compute.
336. **The mirror is P2.2a's frame, copied unedited, with IDLE's reservation and the registered
     genesis carry, driven by new files**, and checked against v1's long run and P2.2a's F5 before
     any prediction. *Alternative:* a new mirror.
337. **The engine is scored against the registration by §11.5's lines.** *Alternative:* the demo's
     v1 standard alone (no dead tick, no shortfall), which v2 would fail on shortfalls by design.
338. **O39's remainder is fixed at D2.1**: the clock, the fitted view clear of the legend and
     credit, the ranked table's value column; G1.6's eight mutants and the credit's wrap stay
     closed, their tests extended to the v2 store. *Alternative:* leave the layout notes to G4
     proper.
339. **The v2 tape is new, `tapes/demo-gb-v2.ron`, with its own pin**; v1's tape and pin are not
     touched. *Alternative:* v2 replaces v1's tape and pin.
340. **The long run's like-for-like reference is the flow county at C2g with ex-post assignment**
     (decision 235). *Alternative:* v1's run as the only reference.
341. **The build's steps are §13's, each with both gates green before a commit that touches code.**
342. **This design is `docs/demo/WORLD-V2.md`, with WORLD.md §11 pointing to it**, since WORLD.md
     would pass 800 lines. *Alternative:* WORLD.md §11 in full.

## 13. The build, D2.1–D2.6

1. **D2.1, O39's remainder on v1.** The clock refused off 52; the fitted view clear of the legend
   and credit; the ranked table's values whole. Tests as §10. v1's tape and every pin unchanged.
2. **D2.2, the stage in the compiler.** `machine_types.csv`, `stage-v2a1.csv`, `derive.py`'s column,
   `compile_stage`, `worldgen::chain`, the writer, the checks of §9.2, the tests of §9.2 and §9.3,
   and `tapes/demo-gb-v2.ron` with its pin. Then E0 (the trace diff) before any scored run.
3. **D2.3, the lenses and the map.** `lenses-v2a1.csv`, the measures and `oracle_gap` in worldgen,
   the GUI's lens table per tape, the idle horse price, the card's cause line, and §7's tests; the
   G1.6 tests on the v2 store; two screenshots of the v2 map, taken as v1's were.
4. **D2.4, the battery and the long run on the engine**, scored against the registration (§11.5):
   the compiler's `--instances` and the harness's `--counties` and `demo:` instances (§11.3), the
   engine's elasticity probe at every county-date, the kick sets, and `demo_v2_runs_to_1901` in
   `scripts/gate.sh` by name. The scorer, the gather script and the job list are committed before
   the first job (decision 311).
5. **D2.5, one bounded verification** (as D.2 and D.3 had), one fix round, one re-check of the fixed
   items.
6. **D2.6, the report and the close**: STATE, README, GUI.md's block, and your look at the map.

## 14. Open items O81–O88

- **O81. Capital's lag over the history.** Under rule A at C2g the median county's median gap is D̂
  117 (0.117 in log) and Lanarkshire's 383, against 28 for the flow county at the same dials and 11
  for v1. Neither land's rate at 1.3 nor s_K at 4δ shrinks it. D-G14's departure, seen over a
  history; entry (Phase 3) is the candidate fix.
- **O82. Transfer shortfalls in six thin-funded counties** (Surrey 5,593 ticks, Armagh 3,619,
  Caernarfonshire 2,320, Sussex 1,479, Down 1,115, Tyrone 376), where v1 had none.
- **O83. v2a.1b in the demo.** Each county needs a funded rule-B point and its own registration
  (decision 308); chain8 was funded only by moving N. The next pass searches the 93 counties for
  them.
- **O84. Thin b × 2 targets.** 39 funded b × 2 targets have margins under 0.05 baskets per unit of N
  (16 under 0.02). They run; their shortfalls are reported.
- **O85. The GUI at v2's size.** 25 s to 1901, about 9,200 series and 0.58 GB, and a frame p90 near
  8 ms, are estimates until D2.3 measures them.
- **O86. `oracle_gap`'s cost.** 25,480 solves of 1g over a run; its time and where it runs are
  measured at D2.3.
- **O87. The horse stands for engines.** In 1850 Lancashire the one durable good is labelled
  "horses"; v2a.4's engine good and G3's recipe choice replace it.
- **O88. The history's pace is v1's.** It was bounded for a flow machine (WORLD.md §4.4); nothing in
  it was re-dated for capital's time.

## 15. Where the evidence is

Scratch `D:/rustyecon-d2/design/` (WSL `/mnt/d/rustyecon-d2/design/`):

- `fund/`: `d2-fund`, the funding check, collapse, chain ranges and the mirror's county dump;
  outputs in `out-d08/` (`summary.txt`, `funding.tsv`, `targets.tsv`, `funding-table.tsv`,
  `counties.json`).
- `mirror/`: §11.1's mirror and its extension; outputs in `out-mirror/` (`long-c2g.json`,
  `long-flow-v1.json`, `long-flow-c2g.json`, `long-land13.json`, `long-sk4.json`, `lin.json`,
  `battery.jsonl`, `validate-f5.json`, `lens-ranges.txt`, `vgap.json`).
- `speed/`: the GUI's and the cli's timings.
- `tools/`: the summaries that made §4's and §6's tables.

## 16. As built

Dated entries, one per build step; the design above stands as registered.

**D2.1 (2026-09-30): O39's remainder on v1.** The compiler refuses a clock other than 52 ticks a
year. The fitted map leaves the legend and the credit clear (Cornwall sat under the legend at
1,280 × 800). The lens selector's button cuts a long name short, and the ranked table clips county
names before values (at a 480-point window the value column had run 33 points off). Each has a test
that fails with its fix undone (§10; decisions 343, 344).

**D2.2 (2026-09-30): the stage in the compiler.** `rustyecon worldgen worlds/demo-gb --stage v2a1`
compiles `tapes/demo-gb-v2.ron`; `crates/worldgen/src/chain.rs` holds rule A and 1g's chain,
`stage.rs` the stage's tables, checks, genesis and writer.
- **R1.** Without `--stage` the compiler writes v1's tape bit for bit (`regions.csv`'s
  `machine_types` now reads `horse`, and v1 compiles it as its flow machine). v1's pin
  `0xfad880fe08d06645` and every committed tape's hashes are unchanged.
- **The checks, on all 93 counties** (WSL, 3.4 s for the compile): 25,480 step dates and 93
  genesis points, every one Interior and funded under 1a and 1g; 1g meets 1a within 8.0e-16
  relative (Leicestershire, July 1853); the chain's own prices and quantities move at most 0.0222
  in log at a date (Gloucestershire, May 1848) and 0.0475 over a trailing year (Lanarkshire, June
  1845). These are §3's and §4's numbers.
- **The tape.** 14,828,343 bytes (988,553 gzipped), 71,355 lines, 33,332 events on 25,480 county
  dates, 558 actors, `tape_hash` `0xb06f07458015e332`, `world_id` `0x0b8d31536f99be41`. §9.4
  estimated 13.6 MB: the per-county comments and bases are longer than v1's.
- **Its pin** (decision 339; `demo_v2_pin`, run by name in `scripts/gate.sh`): through the first
  tick of 1901 every one of the 33,332 events fires and the ledger closes every tick (largest
  margin 5.9e-4 of the tolerance); final hash `0x45b7c1201f8ae633`, hash stream
  `0xacb3ca2bee63b3bf`, the same on WSL and Windows. The pin scores nothing: the long run's scoring
  against the registration is D2.4's.
- **At rest** (`v2_rests_at_every_county_oracle_point`): with the history removed, every county's
  18 observables stay within 5.6e-14 in log of 1g's point for three ticks and 6.8e-14 for two
  years (the horse market's volume), every market trading and the ledger closed every tick.
- **Held to the probe**: rule A, 1g's chain and its point are `probe::horses`'s bit for bit on F5,
  F6 and ten counties at genesis and in 1850 (`rule_a_is_the_probes`); each county's genesis is
  the horses rule divided by p/r to 1e-14 (`v2_genesis_is_the_horses_rule_at_each_county`); the
  dials are C2g key by key (`c2g_is_the_probes`).
- **Refused** (`the_stage_compiler_refuses_bad_tables`): dials off C2g, s_K not 2δ, no
  reservation, ex-ante assignment, an unknown machine type or stage, δ, ω or J_b out of range, rule
  B, a county's machine type not the horse, a clock off 52, and an unfunded date with its county
  and date (Surrey's population raised 30% over 1830–1834).

**D2.3 (2026-09-30): the lenses and the map.** `worlds/demo-gb/lenses-v2a1.csv` holds the 36
lenses of §7: the chain's 11 first, as the group "Horses and fodder", then v1's 25 with six changed.
`crates/worldgen/src/lens.rs` defines every measure; the GUI gathers a county's recorded numbers
and computes nothing (U6). The map takes a tape's lens table by the tape's name (`lens::for_tape`).
- **The oracle lenses come forward** (decision 331): `lens::oracle_gap` solves 1g from the params
  the run recorded, outside any `Sim`, and keeps each point by its params (decision 352). v1's two
  oracle lenses stay without a value on v1's tape (decision 348).
- **The horse's price** (decision 333) is the maker's net markup by its rule's own sum, with no value
  on a tick when no horse traded; the county's card says "idle: no horse traded this tick; the
  posted price is X of the replacement cost", and the legend marks ψ from the run's param. A
  diverging scale may now centre on 1 (decision 349).
- **Capital's lag, legible** (§8; decisions 350, 351): the selected county's card names the three
  observables furthest from their equilibrium, with signs, beside the herd against its equilibrium,
  against the desk's target, and the horse-day's markup. The gap lens's bands are named in its
  note, shown under the legend. The chart of a lens over the run (§8 item 4) is not built (O89).
- **Held by tests.** `demo_v2_lens_values_equal_the_engine`: every one of the 36 lenses for every
  county at report ticks 0, 51 and 259, 10,044 values, against a Sim's own report, params and
  states, the maker's markup against `maker_reservation` and the horse-day's against the capacity
  desk's recipe, the oracle lenses against 1g at the Sim's params, to 1e-12.
  `price_horse_has_no_value_on_an_idle_tick`: Bedfordshire's desk starts with three times its
  herd, orders none, and no horse trades. `lens_v2_domains_hold_the_oracle_range`: each domain
  holds the oracle's range at every county date (horses per head 1.31 to 8.25, fodder's price
  0.2625 to 0.680, land to the working stock 0.570 to 0.857, v1's to the digit).
  `v2_map_values_equal_table` and `the_v2_map_is_fitted_clear_and_its_values_whole`: G4's gate and
  D2.1's layout on the v2 store. `gui_equals_cli_demo_gb_v2`: the GUI path's hashes of the v2 tape
  to 1901 are the cli's (R16), in `scripts/gui.sh`.
- **The GUI's speed** (O85; Windows, release, the smoke mode `rustyecon-gui --smoke 7852`, on a
  machine shared with another branch's builds in WSL; `D:/rustyecon-d2/build/speed/`): v2 reaches
  1901 in 27.6 and 27.9 s, 281–284 ticks or 5.4 model years a second, frame p50 7.7–7.9 ms and
  p90 10.1–10.3 ms while running, p90 8.9–9.0 ms paused. v1 on the same build and load: 16.5 s
  twice, p90 9.8–9.9 ms running and 8.9–9.3 ms paused (the design's 7.4 ms and 5.8–6.1 ms were
  measured on a quieter machine). So v2 costs 1.68 times v1's time a tick in the GUI (§9.4
  estimated 1.57), and its frame about 0.3 ms more at p90. Through the cli: 27.0 s on Windows,
  and on WSL the pin's run takes 27–28 s, 280–290 ticks a second.

**D2.3b (2026-09-30): the flow path nests v1, `oracle_gap`'s cost, and the map's screenshots.**
- **R1, the flow path** (§9.3; `v2_flow_path_is_v1`): the stage compiled with its machine on the
  flow path (δ = 1 a tick, ω 0, planned assignment, the probe's C2; the owner desk and the maker
  run P2.0's good desk's and the type desk's code, and the durable good is keyed `mach`) runs v1's
  tables with v1's history for 520 ticks, 785 events firing, with every county's four market
  lines' prices and cleared volumes and its four actors' coins equal to v1's tape's, bit for bit
  (193,440 coins). Only the test reaches the flow path: `parse_stage` refuses δ = 1 (decision 354).
- **`oracle_gap`'s cost** (O86; WSL, release, `oracle_gap_cost_is_recorded`, run by name): the
  gap lens's first map at a report tick takes 19.2 ms for 93 counties, 1.2 ms from the memo; one
  1g solve takes 196 µs, so a run that visits every one of the 25,573 county dates solves for about
  5 s in all, spread over the run as its params change.
- **Screenshots** (`docs/demo/v2/map-<lens>-<year>.png`, 16 in all, about 65 KB each): the map pane
  of the GUI at the D2.3 commit, rendered headlessly through egui_kittest's wgpu renderer at
  1,600 × 1,000 from a scratch crate (`D:/rustyecon-d2/build/shot/`), cropped to the pane and
  quantized to 256 colours. Four chain lenses at the first report of 1750, 1800, 1850 and 1901.
  What they show, from the engine's run of the committed tape (not scored; the long run's scoring
  against the registration is D2.4's):

  | lens | 1750 | 1800 | 1850 | 1901 |
  |---|---|---|---|---|
  | horses per head, lowest to highest | 1.77 (Surrey) to 3.83 (Lanarkshire) | 1.55 to 4.45 (Northumberland) | 1.35 to 5.14 | 1.18 (Dunbartonshire) to 6.96 |
  | horses against equilibrium, lowest; median | 0 at every county | −0.18 (Renfrewshire); −0.05 | −0.32 (Lanarkshire); −0.13 | −0.26 (Glamorgan); −0.02 |
  | horse-day markup, median; highest | 0 at every county | +0.04; +0.16 (Lancashire) | +0.07; +0.19 (Lanarkshire) | +0.01; +0.13 (Glamorgan) |
  | land to the working stock, lowest to highest | 0.74 (West Riding) to 0.84 (Norfolk) | 0.71 to 0.85 | 0.61 to 0.82 | 0.58 to 0.82 (Huntingdonshire) |

  At genesis every county rests at its equilibrium (the herd's gap and the markup are 0 to
  2e-16). Through the century the fast-growing counties' herds fall behind, and their horse-days
  carry a markup, the rent that buys more (§6, §8); by 1901 the median county is 2% short.
