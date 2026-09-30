# COMMONS-SPEC: the open-commons instance of Phase 2 proper (frame)

Dated 2026-09-30. Work label `frame-commons`, for Phase 2 proper (STATE next step 7) on branch
`phase2-proper` (worktree `D:/rustyecon-wt/p23`, at `f7d1eae`; nothing in the worktree was changed).
Scratch: `D:/rustyecon-p23/frame-commons/`. Every number below is the mirror's (`model/cm.py`) at
C2m, 52 ticks a year, ρ 0, tolerance 1e-3 in log, or the oracle's where the line says so. The
evidence is in the files of §10, whose sha256 are in `SHA256SUMS` beside this file.

It frames unit 1e's open-commons instance under the default exit form s(q) = max(s₀ − q·h, s̲)
(ADDENDUM ruling 3): the commons with its shadow rent, idle land at zero rent, households on plots
and decision 149's participation rule, on the many-market roles (decisions 119–121). It does
four things, in this order:

1. a mirror scan for the idle land market at zero rent, and the choice it makes (§1);
2. the instances, checked by an independent 50-digit solve that reads no oracle code (§2);
3. the roles as engine Behaviours, the smallest additions, and the proof that they rest exactly
   at the oracle's point (§3);
4. the mirror, the protocol and the registered predictions (§4–§6).

## 0. The frame in brief

- **The instances.** C1 and C2 are the markets probe's I1 (four categories, 1a's machine, N 208
  and T 520 a year, χ ~ U[0, 1]) with one priced worker type, its exit good food, and a commons:
  - **C1**, exit (s₀ 0.3, s̲ 0, h 0.135), commons 24.3 a year: the commons is full and rationed by
    a shadow rent r_o = 0.43792 (rent = 1). Participation 0.13462, x\* 0.74834.
  - **C2**, exit (0.2, 0, 0.09), commons 31.2 a year: the commons has room; 48.8% of it idles at
    zero rent. Participation 0.14681, x\* 0.74350.

  Their 26 targets span the three regimes (Commons, Crowded, Enclosed). Each is interior on the
  line, with one equilibrium, funded, and certified by unit-1e.md §5.4 except b.food × 0.5. The
  oracle crate agrees with the 50-digit solve within 5.6e-15 at all 26.
- **The idle land market.** In 1e, land at zero rent idles in two places. The commons with room
  is one. Enclosed land on the idle stretch (r = 0) is the other.
  - The commons is no market. The commoners hold it, and the participation rule gives out its
    plots (§3.1). Its shadow rent is a readout that nobody receives.
  - The scan (§1) finds this the only candidate that rests exactly in all three regimes. Posting
    the commons as a market (Saturate or `Hold`) runs to the runaway bound whenever it has room:
    after 1,126–11,590 ticks, at all 4 of C1's Commons targets (8 runs). The commoners' reservation
    orbits at 0.1–0.25·r, off the point. A floor order gives a continuum of rest points.
  - Idle enclosed land at r = 0 cannot be funded under the probe's transfer. The provider's only
    income is rent, so its baskets are −ν·N there (checked in the oracle). No Phase 2 instance
    under that transfer holds it (decision 362; O96).
- **The additions.** One optional field on `BasketWorkers` (`exit`). The workers' participation
  becomes the regime-wise supply of unit-1e.md §4.3–4.4 for one type. Where plots spill onto
  enclosed land, they post one land buy. Nothing changes in core, markets, engine, the provider
  or the desks, and no state is added. With `exit` absent the role is P2.1's, bit for bit.
- **The prediction: C1 and C2 GO.**
  - Mode A passes at 12, 52 and 365 a year.
  - Every non-vacuous run in Tiers 1–3 and in Tier 3 at 10·L is CONVERGED: C1 115/115, C2
    109/109, and 6 of C2's commons shocks are VACUOUS by construction.
  - The largest root per tick is 0.98538–0.99139, and every kick bar passes.
  - Ticks to tolerance, median by Tier 1/2/3: C1 319/372/508, C2 315/360/514. The I1 control
    on the same mirror gives 324/381/525.
- **The subsistence trap (§6.8), a prediction of the families.** It is not in the verdict.
  - With exit valued at food's posted price and food made with labour, the agents have a second,
    absorbing state. Nobody works, food is not supplied, and every price inflates together.
  - C1 reaches it in 4/60 joint2 and 12/40 joint4 runs, in 89 of 430 basin runs, in 3 tilt-1
    Tier-3 runs, and in the land.mach history's fifth window at tick 288. C2 reaches it in 8/40
    joint4 runs and 77 of 430 basin runs.
  - I1 never does: 0 of 530 on the same families.
  - At χ_max 0.25, a negative control that changes one param, 9 of C1's Tier-3 runs fall in.

## 1. The idle land market at zero rent: the scan

### 1.1 Where it arises

Unit 1e has three idle margins (unit-1e.md §0.2). Two of them are land at zero rent:

| | the commons with room | idle enclosed land |
|---|---|---|
| regime | `Commons` (r = 1: enclosed land scarce) | the idle stretch, `LandMarket::Idle` (r = 0, the pool's wage numeraire) |
| what idles | open land plots do not fill: T_oi = T_o − G(0) > 0 | enclosed land production and plots do not use: T_idle = T − T_m − T_p > 0 |
| its price in the oracle | the plot rent r_o = 0 exactly | the rent r = 0 exactly |
| decisions | 148, 149, 156 | 153, 160, 161 |
| the provider's baskets (the probe's transfer) | r·T_m/P_s − ν·N ≥ 0.157 at every target here | −ν·N by construction: rent is its only income |

**Idle enclosed land cannot be funded under the probe's transfer.** The provider pays N·P_s to
the workers from rent. At r = 0 it has no income. The oracle reports it (`check/idle.out`, I1's
economy with few workers, dependence form):

| workers per tick | land market | T_idle of T 10 | participation | provider baskets | funded |
|---|---|---|---|---|---|
| 0.2 | Idle, r 0 | 2.026 | 1 | −0.200 = −ν·N | false |
| 0.1 | Idle, r 0 | 6.013 | 1 | −0.100 = −ν·N | false |

So O41/O43's lesson rules the idle stretch out of any Phase 2 instance that uses this transfer.
In an agent run the provider's coin reaches 0 at once, the transfer falls short every tick, and
the workers' baskets are not the oracle's. The hard point that remains for the commons is the
commons itself, **if it is posted as a market** (setting A). §1.5 records what each candidate
would do on the idle stretch (setting B).

### 1.2 The candidates (setting A: the commons)

In each market candidate the commons is a market of its own. Its price is the shadow rent r_o.
The workers, as commoners, offer T_o and bid for their plots at the posted r_o
(`cm.commons_market_decide`). Below r̂ (the market rent r where a plot pays there), the plots are
on the commons. At r_o ≥ r̂ they fill the commons first and rent the rest on enclosed land, as
the rule does. Each candidate reads only its own state, posted prices and params (R13). Each nests
(with `exit` absent the run is P2.1's), and none floors or caps a price (R3).

- **msat**: no rule, the engine's `Saturate`.
- **mhold**: the world's one-sided rule `Hold` (IDLE-SPEC's alternative).
- **mres25, mres10**: the commoners' reservation, a step on their offer, as L0's maker. They
  offer none of the commons while r_o < ψ·r, at ψ 0.25 and 0.1.
- **mfloor**: a floor order. The commoners also bid for any commons their plots leave: bid
  max(G, T_o).
- **rule**: the commons is no market, the reading of decision 153 for land in excess supply at
  zero rent. The commoners hold it, and the participation rule rations it (§3.1). The shadow rent
  is the rule's, and nobody receives it.

### 1.3 The scan

`model/scan.py` → `model/scan.out`, `model/runs/scan.jsonl`: 714 runs.
- **Mode A** is C1's genesis for L = 141,000 ticks.
- **Tiers 1–3** are C1's battery, 115 runs.
- **Room** is C1 with its commons made roomy by a dated shock at L/4 (× 1.1 and × 2: the
  equilibrium moves to `Commons`, r_o\* = 0).
- **C2** is C2 (room at genesis) from a positive commons price, 0.466·r: C2's own Crowded
  shadow rent at half its commons.
- **PL** is the largest root per tick at C1.

| candidate | PL at C1 | C1 mode A: largest D̂ | C1 Tiers 1–3 | room × 1.1 / × 2 | C2 |
|---|---|---|---|---|---|
| msat | 0.999209 | 2.4e-12 | 107 CONVERGED, **8 DIVERGED** | DIVERGED at tick 7,544 / 1,126 | DIVERGED at 1,130 |
| mhold | 0.999209 | 2.4e-12 | the same as msat, run for run | the same | the same |
| mres25 | 0.999209 | 2.4e-12 | 103 CONVERGED, **12 ORBITING** | ORBITING, D̂ 157 / 156 | off the point, D̂ 70 |
| mres10 | 0.999209 | 2.4e-12 | 107 CONVERGED, **8 ORBITING** | ORBITING, D̂ 65 / 65 | off the point, D̂ 28 |
| mfloor | 1.000000 | 2.4e-12 | 43 CONVERGED, **72 STUCK** | STUCK at the old r_o 0.438, D̂ 229 | STUCK at 0.466, D̂ 127 |
| **rule** | **0.986032** | 2.4e-12 | **115 CONVERGED** | **CONVERGED, D̂ 3.8e-12 at the end** | at the point, D̂ 3.3e-13 |

What each failure is:
- **msat and mhold.** The 8 DIVERGED are exactly C1's targets in the `Commons` regime: land.mach
  × 2, b.food × 0.5, and the commons × 1.1 and × 2, each at genesis and dated. There the commons'
  bid G(r_o) stays below its offer T_o at every r_o > 0. So r_o falls by a constant factor a tick
  until it crosses the runaway bound (1e-6 of genesis), after 1,126–11,590 ticks. The real
  observables are within 1.1e-7 to 3.0e-7 in log by then (final D̂ 1.1e-4 to 3.0e-4): the price
  is the only thing wrong, and it is wrong for ever. Left longer, the double underflows and the
  engine refuses the price. `Hold` never acts, because the commons always has a bid, so it is msat
  run for run.
- **mres.** At a `Commons` target the price falls to ψ·r and then chatters around it. The plots
  cost ψ·r instead of 0, so participation stays off the point (final D̂ 22–157). At ψ 0.25 the step
  also cuts off two Crowded targets whose shadow rent is below 0.25 (b.food × 0.9: r_o\* 0.238;
  land.mach × 1.1: 0.219).
- **mfloor.** Any r_o at or above r_o\* is a rest point, because the bid meets the offer. So the
  map has a unit root and a continuum of rest points: 72 of 115 runs stick where they were
  displaced to.
- **All market candidates are slow.** At C1 the commons' own mode has a root of 0.99921
  (half-life 876 ticks), against the rule's 0.98603 (49 ticks).

### 1.4 The choice

**The rule: the commons is no market** (decision 363).
- It is the only candidate that rests exactly in all three regimes. Every market candidate is
  exact only where the commons is crowded or spills (r_o\* > 0). None reaches r_o\* = 0.
- It needs no price for the idle part. The idle commons is simply not taken, and nothing is
  posted for it.
- It nests. With `exit` absent, and with the exit switched off (s₀ = s̲ = 0), the role is
  P2.1's `BasketWorkers` bit for bit (§4.2).
- It is structural. Its min and max are the edges of the supply map, not clamps on a price: the
  support of F, the commons' capacity, and the rent at which a plot stops paying. No price is
  floored, capped or smoothed (R3).
- It reads posted prices and its own params (R13).
- It is the fastest locally: half-life 49 ticks, against 876 for every market candidate.

**The named alternative (R6): msat, the commons as a market under `Saturate`.** It is exact
wherever the commons is crowded or spills, and needs a market and a good, not a new rule. It is
not chosen for three reasons. It runs away wherever the commons has room, which is C2's regime,
the 1750-like instance's, and 8 of C1's 115 battery runs. It is 18 times slower locally. And its
spill onto enclosed land needs the commons' size anyway, which is the rule's knowledge.

**The limit of the choice** (O97). The rule gives out one pop's commons from that pop's own
demand. Several types sharing one commons (1e's instance T) would need each other's plot demand,
which R13 forbids. For them the commons must be a market (exact only while crowded) or an actor
of its own.

### 1.5 Setting B, idle enclosed land at r = 0 (argued, not run)

The idle stretch is outside Phase 2 under the probe's transfer (§1.1). What each candidate would
do there, from the rules and setting A's runs:
- **Saturate.** The land market has S = T and D = T_m < T at a fixed technique, so
  x = −T_idle/T and the rent falls by exp(−k·T_idle/T) a tick. At N 0.2 (T_idle/T 0.2026, k 0.025)
  that is 0.99495 a tick: the runaway bound comes at tick 2,727 (52 years), then the double
  underflows. The real side converges as r/w → 0.
- **Hold.** The desks always bid for land, so it never acts.
- **The owner's reservation**, a step on the provider's offer. On a withheld tick no desk gets
  land. Food, care, shelter and the machine all use it, so production stops every other tick, and
  the rent orbits a positive level. That is not the oracle's x\* = 1.
- **A floor order.** The provider bids for its own idle land at the posted rent. The rent then
  holds at any level, which is the same continuum as mfloor, with r > 0.
- **NoMarket.** Land in excess supply is a free good. That needs the desks to price land at 0
  when its market has room: a posted price of 0, which the price step's range check refuses, or a
  "closed" market the desks can read. Either is a change to markets, and out of this frame.

So idle enclosed land needs both a funded support and a free-good rule in the engine. It gets
its own frame when an instance needs it (decision 362; O96).

## 2. The instances

### 2.1 C1 and C2

Both are the markets probe's I1 (MARKETS-SPEC §1.2) with its scalars (ρ 0, δ 1, J 1; γ = 0.2 + 0.8x;
C3's categories; 1a's machine a 0.3, λ 0.05, b 0.4, θ 1; tick 1/52 year), and:

| param (tape key) | unit | C1 | C2 | note |
|---|---|---|---|---|
| `inst.workers` N | FlowPerYear | 208 (4 a tick) | 208 | one type, efficiency 1 |
| `inst.chi_max` χ_max | Dimensionless | 1 | 1 | I1's (decision 361) |
| support ν | — | 1 basket | 1 basket | the probe's transfer, N·P_s (decision 368) |
| `inst.land` T (enclosed) | FlowPerYear | 520 (10 a tick) | 520 | the provider's endowment |
| `inst.commons` T_o | FlowPerYear | **24.3** (0.467308 a tick) | **31.2** (0.6 a tick) | open land, held by the workers, never traded |
| `inst.exit.gross` s₀ | Dimensionless | **0.3** | **0.2** | food a head a tick in exit |
| `inst.exit.floor` s̲ | Dimensionless | 0 | 0 | |
| `inst.exit.plot` h | Dimensionless | **0.135** | **0.09** | land a head a tick in exit; h/s₀ = 0.45 in both |
| exit good g | good | `food` | `food` | decision 151 |

q_enc = s₀/h = 2.22 in both, above q = r/p_food at every target (0.69–1.83; 1.17 at both bases).
So a plot pays at the market rent everywhere, and no enclosure point lies near an equilibrium.

### 2.2 How they were chosen (a forking path, disclosed)

1. `solve/explore*.py`: the Crowded band. With one priced type, the Crowded supply
   N − T_o/h must lie between n(0) and n(r) at the point's own prices. Its width in log is about
   h/(P_s + e) (§3.1). So the exit must be worth a good part of the wage, and h as large as
   certification allows. h ≤ s₀·b̄_food, with b̄_food = b_food, and b_food runs down to 0.3 at
   the b.food × 0.5 target. h/s₀ = 0.45 certifies every target but that one.
2. `model/search.py` (`search.out`, `search2.out`): χ_max ∈ {1, 0.75, 0.5, 0.35, 0.25} ×
   s₀ ∈ {0.2, …, 1} × h/s₀ ∈ {0.3, 0.45, 0.6}, each with C1's commons at the log-middle of its band
   and C2 at twice it. The criteria were every target on the line, Tier-2 targets at least 0.1
   in log from a regime boundary, and positive provider baskets. At χ_max 1 the commons doubled
   put C2's land.mach × 0.5 target on the wall for s₀ ≥ 0.25. So the first choice was χ_max 0.25,
   s₀ 0.3, h 0.135 (C1 15.6, C2 31.2 a year).
3. That choice's battery (`runs/battery_v1.out`) had 11 and 12 Tier-3 runs DIVERGED, all by the
   subsistence trap (§6.8). `model/diag_chi.py` (`diag_chi.out`) separated the causes on C1's
   Tier 3. The dependence form converges 39/39 at χ_max 0.25 and at 1. The priced exit converges
   43/43 at χ_max 1, 39/43 at 0.5 and 32/43 at 0.25. So the trap is the priced exit's, and its
   gain grows with the supply's elasticity 1/χ_max.
4. The final choice (`model/search3.py`) keeps I1's χ_max 1. C1 keeps s₀ 0.3 and h 0.135, with
   its commons at 24.3 a year (the band's middle, rounded). C2 takes s₀ 0.2 and h 0.09, so that its
   land.mach × 0.5 target stays on the line (x\* 0.957). C2's commons is 31.2 a year, twice its own
   band's middle, so that its commons × 0.5 target is Crowded.

No displaced-start run was used to choose any number but χ_max (step 3).

### 2.3 The oracle's answer

At rent 1, per tick at 52 a year. `solve/points.out` (50 digits), `check/compare.out` (the crate):

| | C1 | C2 |
|---|---|---|
| regime (`ExitLand`), land market | Crowded, Scarce | Commons, Scarce |
| x\* | 0.748337677699362711 | 0.743502494907990476 |
| v = w/r | 0.483993749575249315 | 0.481508566963529216 |
| P_s | 1.794963129973971726 | 1.794718722256366153 |
| p_food, p_mach | 0.853609808416, 0.605999553541 | 0.853521178147, 0.605822040497 |
| Y | 5.716335810836874224 | 5.729457566784460178 |
| hours S (participation) | 0.538461538461538462 = 7/13 (0.134615) | 0.587247627315616170 (0.146812) |
| n(0), n(r) at the point | 0.42147, 0.69382 | 0.58725, 0.77474 |
| plot rent r_o, q_o = r_o/p_food | 0.437920047266732310, 0.51302 | 0, 0 |
| exit value e = p_food·s(q_o) | 0.196964 | 0.170704 |
| commons used / T_o | 0.467308 / 0.467308 (full) | 0.307148 / 0.6 (51.2%; 48.8% idle at zero rent) |
| plots rented T_p | 0 | 0 |
| κ, provider baskets | 1.392786, 1.571145 | 1.392976, 1.571904 |
| f at x = 1e-12, at x = 1, f_∞ | 7.78, −0.390, −3.749 | 7.937, −0.560, −3.749 |

Every target (`solve/targets_table.md`):

| target | regime | x\* | v | P_s | Y | participation | r_o | T_p | commons used | provider baskets | certified |
|---|---|---|---|---|---|---|---|---|---|---|---|
| C1 base | Crowded | 0.74834 | 0.48399 | 1.79496 | 5.7163 | 0.1346 | 0.4379 | 0 | 0.4673 | 1.571 | yes |
| C1 land.mach × 1.1 | Crowded | 0.74733 | 0.53183 | 1.83241 | 5.6136 | 0.1346 | 0.2195 | 0 | 0.4673 | 1.457 | yes |
| C1 land.mach × 0.9 | Crowded | 0.74935 | 0.43606 | 1.75751 | 5.8235 | 0.1346 | 0.6570 | 0 | 0.4673 | 1.690 | yes |
| C1 land.mach × 2 | Commons | 0.70386 | 0.92238 | 2.16389 | 4.9838 | 0.2126 | 0 | 0 | 0.4252 | 0.621 | yes |
| C1 land.mach × 0.5 | Enclosed | 0.87344 | 0.27440 | 1.61007 | 6.2745 | 0.1069 | r | 0.0150 | 0.4673 | 2.202 | yes |
| C1 b.food × 1.1 | Crowded | 0.74650 | 0.48305 | 1.85487 | 5.5314 | 0.1346 | 0.6332 | 0 | 0.4673 | 1.391 | yes |
| C1 b.food × 0.9 | Crowded | 0.75157 | 0.48566 | 1.73512 | 5.9140 | 0.1346 | 0.2381 | 0 | 0.4673 | 1.763 | yes |
| C1 b.food × 2 | Enclosed | 0.83183 | 0.52714 | 2.39858 | 4.2275 | 0.0803 | r | 0.0293 | 0.4673 | 0.157 | yes |
| C1 b.food × 0.5 | Commons | 0.74452 | 0.48203 | 1.49477 | 6.9146 | 0.1742 | 0 | 0 | 0.4460 | 2.690 | no |
| C1 commons × 1.1 | Commons | 0.79986 | 0.51056 | 1.79727 | 5.6966 | 0.1167 | 0 | 0 | 0.4770 | 1.564 | yes |
| C1 commons × 0.9 | Enclosed | 0.73405 | 0.47666 | 1.79418 | 5.7393 | 0.1703 | r | 0.0274 | 0.4206 | 1.558 | yes |
| C1 commons × 2 | Commons | 0.79986 | 0.51056 | 1.79727 | 5.6966 | 0.1167 | 0 | 0 | 0.4770 | 1.564 | yes |
| C1 commons × 0.5 | Enclosed | 0.73288 | 0.47605 | 1.79411 | 5.6348 | 0.1701 | r | 0.2145 | 0.2337 | 1.454 | yes |
| C2 base | Commons | 0.74350 | 0.48151 | 1.79472 | 5.7295 | 0.1468 | 0 | 0 | 0.3071 | 1.572 | yes |
| C2 land.mach × 1.1 | Commons | 0.73683 | 0.52589 | 1.83178 | 5.6438 | 0.1608 | 0 | 0 | 0.3021 | 1.459 | yes |
| C2 land.mach × 0.9 | Commons | 0.75257 | 0.43755 | 1.75765 | 5.8211 | 0.1322 | 0 | 0 | 0.3124 | 1.689 | yes |
| C2 land.mach × 2 | Commons | 0.68677 | 0.90491 | 2.16065 | 5.0515 | 0.2527 | 0 | 0 | 0.2690 | 0.628 | yes |
| C2 land.mach × 0.5 | Commons | 0.95718 | 0.29637 | 1.61140 | 6.2663 | 0.0823 | 0 | 0 | 0.3304 | 2.206 | yes |
| C2 b.food × 1.1 | Commons | 0.74542 | 0.48249 | 1.85482 | 5.5342 | 0.1373 | 0 | 0 | 0.3106 | 1.391 | yes |
| C2 b.food × 0.9 | Commons | 0.74161 | 0.48054 | 1.73462 | 5.9389 | 0.1570 | 0 | 0 | 0.3035 | 1.765 | yes |
| C2 b.food × 2 | Commons | 0.82215 | 0.52211 | 2.39819 | 4.2416 | 0.0825 | 0 | 0 | 0.3303 | 0.170 | yes |
| C2 b.food × 0.5 | Commons | 0.73436 | 0.47681 | 1.49420 | 6.9549 | 0.2055 | 0 | 0 | 0.2860 | 2.693 | no |
| C2 commons × 1.1, × 0.9, × 2 | Commons | the base's | | | | | | | | | yes |
| C2 commons × 0.5 | Crowded | 0.73568 | 0.47749 | 1.79428 | 5.7507 | 0.1667 | 0.4660 | 0 | 0.3000 | 1.573 | yes |

C1's regimes at its 13 points: Crowded at 5 (the base, and land.mach and b.food at × 1.1 and
× 0.9), Commons at 4, Enclosed at 4. C2's: Commons at all but its commons × 0.5 (Crowded). In the Commons
regime the commons' size does not move the point, so C2's commons × 1.1, × 0.9 and × 2 are its
base (VACUOUS runs by construction; O99). Enclosure by law (a share of the commons moved to
enclosed land, `model/enclose_points.py`) raises participation and lowers the wage, main.tex:385's
sign. C1: 0.1346 → 0.1704, v 0.48399 → 0.47681, Enclosed at both shares. C2: 0.1468 → 0.1667
(Crowded at half) → 0.1904 (Enclosed, all).

### 2.4 Uniqueness, and the check that reads no oracle code

- **The independent solve.** `solve/csolve.py` is written from unit-1e.md §4 (4.1–4.6) and
  MARKETS-SPEC §1's statement of I1. It uses mpmath at 50 digits and reads no code of
  `crates/oracle` and no mirror.
  - It evaluates the sign sequence over 64 points of the all-human corner, 4,097 of the line
    (16 times `EXIT_SCAN`), 161 of the wall (v up to 2^40 of its start) and 65 of the idle stretch.
  - It bisects the root to its working precision and checks the Crowded shadow rent's closed
    form against a bisection (equal to every digit printed).
  - At all 26 points: one change of side; f monotone over the 4,097 line points; the corner
    positive (f ≥ 5.54); the wall (f ≤ −0.074) and f_∞ (≤ −3.70) negative. So the equilibrium
    is interior on the line and alone.
- **The oracle.** `check/` is a scratch crate on the worktree's `crates/oracle`, built on WSL.
  `ParcelEconomy::solve` returns `Interior` at all 26 points, in the same regime. It is `certified`
  at all but b.food × 0.5 (there its scan of 1,024 points is the count). It is `funded` at all 26.
- **Agreement.** The crate against the 50 digits: at most 5.6e-15 relative over x\*, v, P_s, Y,
  n_a, p_food, p_mach, T_oc, and r_o or T_p (`check/compare.out`; the largest on r_o, a
  cancellation p_food·s₀ − e). The mirror's f64 oracle against the 50 digits: at most 5.3e-15
  (`model/oracle_check.out`).
- **Decision 70 is met by proof and by count.** Proposition 5 (unit-1e.md §5.4) proves uniqueness
  at 24 of the 26 points: ρ 0, one plot-taking type, h ≤ s₀·b̄_food, h·ℓ₀ = 0.107 ≤ 1 for C1. At the
  two b.food × 0.5 points (h/s₀ 0.45 > b_food 0.3) the 4,097-point scan is monotone.

### 2.5 Funding

The probe's transfer is funded at rest and at every target: provider baskets 0.157 at least
(C1 b.food × 2), 0.170 (C2 b.food × 2), and 1.454–2.693 elsewhere. Those are the oracle's accounts,
r·T_m/P_s − N. In the agents the provider also receives the rent of plots on enclosed land (§3.2),
so its baskets are r·T/P_s − N, higher at the Enclosed targets (O98). The enclosure family's
targets are funded at 1.584–1.747.

### 2.6 The cost targets

| coefficient (tape param) | base | × 1.1, × 0.9, × 2, × 0.5 | why |
|---|---|---|---|
| `land.mach` (the machine's land b) | 0.4 | 0.44, 0.36, 0.8, 0.2 | I1's first |
| `b.food` (food's direct land) | 0.6 | 0.66, 0.54, 1.2, 0.3 | I1's second; it moves the exit good's price |
| `commons` (`inst.commons`, a year) | C1 24.3, C2 31.2 | C1 26.73, 21.87, 48.6, 12.15; C2 34.32, 28.08, 62.4, 15.6 | the commons: C1's move it across all three regimes |

Each is shocked at genesis and dated at L/4, as MARKETS-SPEC §7.7. The family `enclose`
(§5.4) moves a share (½, 1) of the commons to the enclosed land: two params in one dated event.

## 3. The roles as engine Behaviours

### 3.1 The workers' participation with the commons (decision 364)

`BasketWorkers` gains one optional block, `exit`. With it the pop is one worker type (N heads, χ
uniform on [0, χ_max], support one basket) with the priced exit in good g, and it holds a commons
T_o that it never trades. Each tick, at posted prices w (labour), r (land), p_g (the exit good)
and P_s (its basket, as now):

```
Δ    = s₀ − s̲
n(e) = N·min(max(ln1p((w − e)/(P_s + e))/χ_max, 0), 1)            the support of F, not a clamp
e(ρ) = fma(−ρ, h, p_g·s₀)   if ρ·h < p_g·Δ   (a plot pays at plot rent ρ)
     = p_g·s̲                otherwise        (the floor; no plot)
n0   = n(e(0));  plot_r = (r·h < p_g·Δ);  nr = n(e(r))                  (nr = n(p_g·s̲) if not plot_r)
Sc   = N − T_o/h
hours = min(max(n0, Sc), nr)
T_p   = h·(N − nr) − T_o   if plot_r and nr ≤ Sc,   else 0              (plots on enclosed land)
```

- **Commons** (n0 ≥ Sc): the plots fit the commons at no rent. hours = n0, and T_o − h·(N − n0)
  of the commons idles.
- **Crowded** (n0 < Sc < nr): the commons is full. hours = Sc, and the shadow rent r_o is the ρ
  with n(e(ρ)) = Sc. Nobody pays or receives it.
- **Enclosed** (nr ≤ Sc, a plot pays at r): the plots spill. hours = nr, and T_p is rented on the
  land market (§3.2).
- **The split** (nr ≤ Sc, no plot pays at r): the commons fills at the rent where a plot stops
  paying. hours = nr, the floor's supply; T_p = 0 (unit-1e.md §4.4).
- A pop that takes no plot (h = 0 or Δ ≤ 0) supplies n(p_g·s̲), with no plots.

This is unit-1e.md §4.3–4.4 for one type at the pop's own posted prices. It is decision 149's
F(ln((ν·P_s + w)/(ν·P_s + p_g·s(q)))), its s(q) at the rent its plot actually pays: 0, r_o or r.
It is homogeneous of degree zero in (w, r, p_g, P_s), so it reads relative prices. The share in
`WorkersState` is hours/N.

**Evaluation order** (for bit-for-bit nesting and the trace diff): P_s as now; then e0 = p_g·s₀;
the branch test r·h < p_g·Δ with both products rounded once; e_r by `num::fma`; each n by
`num::ln1p` of (w − e)/(P_s + e); then Sc = N − T_o/h. With s₀ = s̲ = 0, every e is 0.0, and
(w − 0.0)/(P_s + 0.0) is w/P_s. So hours are `N·min(ln1p(w/P_s)/χ_max, 1)` bit for bit whenever
w > 0 (the mirror checks it, §4.2).

### 3.2 Plots on enclosed land (decision 365)

In the Enclosed regime the workers post a buy of T_p land at r, with budget r·T_p. It goes in the
same `budget_chain` as the basket items, in admission's (good) order. The chain's total is
r·T_p + share(spend)·(C − r·T_p), with C the coin as the phase began. So the plot rent comes first
from the coin, the baskets from the rest, and admission refuses nothing (MARKETS-RULES §3). In
`produce` the workers burn the land they hold as `Consumption`. Plots use it, so nothing of it
spoils. Where T_p = 0 no land order is posted.

The oracle keeps the plots' rent in kind (decision 152: the home account). Here it is paid in
money, from the workers' coin to the provider. With the common basket, total spending on baskets
is w·S + r·T − r·T_p = w·S + r·T_m, the oracle's income identity. So every market clears at the
oracle's quantities. Only the split of baskets between the two households differs from the home
account's, by r·T_p/P_s (O98). At rest this matters only at C1's four Enclosed targets.

### 3.3 What does not change

Core, markets, engine: nothing. The provider (its transfer N·P_s and its endowment of T), the
category desks and the type desk are P2.1's roles. The commons is no good and no market. No
`ActorState` variant or field is added (decision 368): the regime, r_o and T_p are the harness's
readouts, recomputed from posted prices and params through the rule's own function.

### 3.4 The spec

In `crates/agents/src/roles/many/spec.rs`:

```rust
/// The priced exit with a commons (COMMONS-SPEC §3), as the tape writes it. Absent: the
/// dependence form, and the role is P2.1's bit for bit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPricedExit {
    /// Good key: g, the exit good, a traded good in the workers' basket (decision 151).
    pub good: Key,
    /// Param key, unit `Dimensionless`, live: s₀, the exit good a head yields at home.
    pub gross: Key,
    /// Param key, unit `Dimensionless`, live: s̲, the floor.
    pub floor: Key,
    /// Param key, unit `Dimensionless`, live: h, land service a plot takes.
    pub plot: Key,
    /// Param key, unit `FlowPerYear`, live: T_o, the commons the workers hold and never trade.
    pub commons: Key,
    /// Good key: land, an `Instant` traded good, for plots rented on enclosed land.
    pub land: Key,
}

// in RawBasketWorkers, the one optional field (as the maker's `reserve`, L0.4):
#[serde(default, skip_serializing_if = "Option::is_none")]
pub exit: Option<RawPricedExit>,
```

It resolves to `PricedExit { good, gross, floor, plot, commons, land }` with `Site`s:
`value` for the three `Dimensionless` params and `live(..., ClockMethod::Flow, "commons")` for
T_o, as `heads` resolves. `BasketWorkers::sites` lists them under `exit.`.

**Load checks** (`resolve_basket_workers` and `Cast::new`), each a `LoadError` with its path:
1. `exit.good` must be one of the workers' basket goods, and traded.
2. `exit.land` must be a traded `Instant` good, the one some role offers as an endowment. It must
   not be a basket item of the workers: "a basket with space and an exit plot" is refused for now
   (decision 366).
3. gross, floor and plot are finite and not negative; commons is finite and not negative.
4. Land for every plot: T + T_o > h·N at genesis, with T the provider's endowment (unit-1e.md
   §3.2).
5. The workers must be the provider's `transfer_to`, so that support is paid (ν = 1).

### 3.5 The rest point is the oracle's: the proof

Take a live rest point: every market clears exactly and trades, every actor spends its income,
nothing spoils (MARKETS-SPEC §4.1, R-a to R-c).
1. **Desks.** P2.1's argument holds unchanged (MARKETS-RULES §3). Each price is its unit cost at
   posted prices (M1k, M2j), and every desk's threshold is at the one x with
   γ(x) = θ·w/p_mach (M3j). These are unit 1c's price rows and task margin at r = 1 once prices
   are read relative to r.
2. **Supply.** At these prices the workers' hours are min(max(n0, Sc), nr). That is exactly unit
   1e's supply of one plot-taking type at the same prices (unit-1e.md §4.3–4.4):
   - G(0) ≤ T_o ⟺ n0 ≥ Sc (Commons, S = n0);
   - G(r) ≥ T_o on the plot branch ⟺ nr ≤ Sc (Enclosed, S = nr);
   - otherwise the Crowded supply is vertical at Sc, or the split at the rent where a plot stops
     paying (S = nr, the floor's).
   The rule is homogeneous of degree zero in the prices, so it gives the same S at the oracle's
   prices relative to r.
3. **Land.** The land market clears T = T_m + T_p, the desks' land plus the plots' rented land.
   T_p = G(r) − T_o is the oracle's rented plots, so the desks' land is the oracle's
   T_m = T − T_p, and Y = T_m/B^q (unit-1e.md §4.5).
4. **Labour.** It clears n_D = S: f = 0, unit-1e.md's equilibrium condition.
5. **Goods.** Every household buys the common basket. Total spending is w·S + r·T − r·T_p =
   w·S + r·T_m, the oracle's I (unit-1e.md §4.8). So baskets are Y, and each good's market clears
   z_j·Y. The provider's transfer N·P_s is paid in full, because its coin is constant only where
   rent income covers it, and every target is funded (§2.5).
6. So a live rest point satisfies unit 1e's equations at the instance's parameters. They have
   one solution at every registered target (§2.4). **The live rest point is the oracle's point**:
   x\*, every price relative to r, every volume, the regime, and the shadow rent as a readout.
7. **No new rest point.** The rule is a function of posted prices and params, with no state and
   no band. Any rest point must satisfy step 6's equations, so it is the one solution. The corpse
   (no trade) is excluded by liveness, as before. The commons is no market, so no idle price
   exists to drift.

The mirror confirms it (`model/nest.out`). At all 26 targets, three ticks from the oracle's f64
point leave every observable within 1.6e-15 in log, with every fill at least 1 − 1e-15. Mode A
at L holds within 2.4e-15 (52 a year), 2.8e-14 (12) and 1.5e-14 (365).

### 3.6 R1, R3, R13

- **R1.** With `exit` absent, the role takes no new branch. Every committed tape keeps its
  canonical text, `tape_hash`, `world_id` and per-tick hash stream: gate `0x61f9c8529131ff17`,
  appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`, and the probe, markets, horses and loops
  pins and streams. TAPE.md's schema stays 1.
- **R3.** No price is floored, capped or smoothed. The min and max bound a supply by its
  definition: F's support, the commons' capacity, the rent at which a plot stops paying. Prices
  move only by `next_price`.
- **R13.** The rule reads posted w, r, p_g and the basket's prices, and its own params (N, χ_max,
  s₀, s̲, h, T_o). It reads no fill, no volume, no other actor and no oracle. Genesis comes from
  unit 1e's point through the tape generator.

### 3.7 Tests

Each must fail with the change it guards undone:
1. `workers_without_exit_are_p21s`: rule-level, the new code path with `exit` absent against P2.1's
   over 3,000 random states. `markets_i0_nests_appb` and every tape's hash tests stay green.
2. `exit_switched_off_is_the_dependence_form`: I1's tape with an `exit` block of s₀ = s̲ = 0 (h 0
   and 0.12, T_o 0 and 30 a year) gives I1's run value for value for 2,000 ticks, from genesis and
   from w × 2.
3. `commons_rule_is_unit_1e_supply`: at points on the line of C1 and C2 in each regime (and at a
   split, with a constructed h), the rule's hours at the point's prices equal the oracle's
   `ParcelEconomy::at_with(x, τ)` supply, and its regime the oracle's `exit_land`.
4. `commons_rest_at_the_oracle`: C1, C2 and every registered target. Three ticks from the oracle's
   f64 point leave every observable within 1e-12 in log, every market trading and filled.
5. `plots_rent_enclosed_land`: at an Enclosed target the workers post one land buy of T_p with
   budget r·T_p, the chain's total as §3.2, and burn what they hold of it in `produce`.
   `many_roles_never_overbudget_or_overdraw` extends to the workers with `exit`, over 3,000 states
   at C1 with prices over twelve decades.
6. `exit_is_checked_at_load`: each refusal of §3.4, and a `commons` param of the wrong unit.
7. `harness_reads_the_rule_the_workers_act_on`: the harness's regime and r_o readouts come from the
   rule's own function (as L0.7's `maker_reservation`), tested on C1 at a displaced state.
8. `commons_tapes_are_their_generators_output`, with `tapes/markets-C1.ron` and `markets-C2.ron`.

### 3.8 The harness (`probe::markets`)

- **Instances.** `C1` and `C2` in `probe::markets::instance`, with the params of §2.1, keyed
  `inst.exit.gross`, `inst.exit.floor`, `inst.exit.plot` and `inst.commons`. The tape's exit good is
  `food`, and its land the provider's `land`.
- **Targets from unit 1e** (decision 369). The harness builds `ParcelParams` from the instance: the
  enclosed parcel T and the open parcel T_o, each quality 1; the one type with `ExitForm::Priced`;
  `exit_good` food; every type's flow recipe as its operating recipe (M9). It solves at genesis
  and at each dated shock. The land market's oracle volume is T (the plots' rented land included),
  and labour's is S.
- **Genesis** (`markets-tape`): from the 1e point. The workers' coin is
  r·T_p + (N·P_s + w·S − r·T_p)/share(spend), which is P2.1's formula where T_p = 0.
- **Run names.** P2.1's grammar, plus `commons=V@genesis|dated` (a cost coefficient) and
  `enclose=F@genesis|dated` (a share F of T_o moved to T in one event).
- **Readouts**, added to `summary.tsv` and `stats.tsv`, computed from posted prices and params
  through the rule's own code: ticks in each regime (Commons, Crowded, Enclosed, split), switches
  between regimes, r_o/r (low, high, end) against the oracle's, T_p's largest, and the provider's
  lowest coin over its genesis coin. Every P2.1 readout stays.
- **The kick** as P2.1's. The runaway bound, classes, tolerance and dead-tick rule are unchanged.

### 3.9 What must not move

The pins of §3.6. `scripts/gate.sh` and `scripts/gui.sh` must be green on both machines before the
commit that adds the code. The GUI names no new kind, since `BasketWorkers` keeps its kind.

## 4. The mirror

### 4.1 `model/cm.py`

- **Its base.** P2.1's registered mirror `mm_carry.py` (sha256 `dbd50ad9…538c`), copied unedited
  to `model/p21/` with the predictor's `mm.py` (`528756c1…b407`) and P2.1's `battery.py`, `mstab.py`
  and scripts.
- **What `cm.py` adds**, each marked "frame-commons":
  - `exit_rule` (§3.1, with an exact fma through `fractions`);
  - the plots' land buy (§3.2);
  - the workers' coin at genesis;
  - the f64 oracle of unit 1e at r = 1 (the regimes of §4.4 in `oracle_at`);
  - the enclosure shock;
  - for the scan only, the commons as a market (`commons_market_decide`).
- **The runner.** `battery_c.py` is P2.1's `battery.py` generalised. It has the same run
  grammar, tiers, observables, classes, early stop and statistics, plus the families and the
  regime readouts.

### 4.2 Its checks

- **Nesting** (`nest.out`). With `exit` absent, `cm.tick` equals `mm_carry.tick` in every state
  value on every tick. That holds for 3,000 ticks of I0 and I1 from genesis, w × 2 and w × 0.5. With
  the exit switched off (s₀ = s̲ = 0; h 0.12 or 0; T_o 0 or 30 a year), I1 runs bit for bit, genesis
  included.
- **The rest point**: §3.5's last paragraph.
- **The oracle**: §2.4.
- **The I1 control reproduces P2.1** (`runs/diag_dep1_battery.out`). I1 on this mirror and runner
  gives Tier 1/2/3 medians 324/381/525 ticks (slowest 654), Tier 2's worst peak D̂ 1,288, 84 dead
  ticks and lowest baskets 0.28. Those are PREDICTION §0's and the engine's numbers (MARKETS.md §1,
  §4).

## 5. The protocol (registered)

### 5.1 Observables, tolerance, classes

MARKETS-SPEC §7.1–§7.5, unchanged: v = w/r; π_j and π_mach; s_j; the cleared volume of labour,
land, each good and the machine's service; y_j and X_mach. That is 22 observables. Every tol_o is
1e-3. D̂ = max gap/1e-3. A tick is dead if a market does not trade or clears less than 0.5 of its
oracle volume. The runaway bound is [1e-6, 1e6] × genesis on every posted price. The classes are
ERROR, DIVERGED, DEAD, VACUOUS, CONVERGED, STUCK and ORBITING. CONVERGED stands only if its
target's kick set decays. The land volume's target is T. r_o and the regime are readouts, not
observables: r_o\* is 0 in Commons, and no log gap exists there.

### 5.2 L (the probe's rule, from the elasticity probe)

`model/elasticity.out`: τ_max is care's price at 703.3 ticks (C1) and 705.4 (C2), as in I1
(719.7). Labour's supply elasticity is 0 in Crowded (the vertical supply) and 1.44 in Commons.

| | 12 a year | 52 a year | 365 a year |
|---|---|---|---|
| C1 | 26,000 | 141,000 | 1,113,000 |
| C2 | 26,000 | 142,000 | 1,116,000 |

The engine's own probe sets L at registration, as P2.1's did.

### 5.3 The battery

MARKETS-SPEC §7.7's grammar at factors 1.05, 0.95, 1.2, 0.8, 2 and 0.5, over 7 prices, 4
techniques, JA, JB, N, RC, x\*/2, and §2.6's three coefficients at genesis and dated. That is 115
runs an instance: Tier 1 has 30, Tier 2 has 42, Tier 3 has 43. Tier 3 runs again at 10·L.
- **Slack runs**, tallied apart (the desk's recipe does not change): at C1 and C2 every
  s[manufactures] run, and s[care] × 1.05, × 1.2 and × 2. x\* 0.748 and 0.744 lie just below
  care's edge 0.75, so s[care] × 0.95, × 0.8 and × 0.5 are not slack (O101).
- **Vacuous by construction**: C2's commons × 1.1, × 0.9 and × 2, at genesis and dated.

### 5.4 The families (O22, stocks first), reported beside the verdict

1. **stocks**: every desk's coin × 0.02, 0.1, 0.5, 2; every desk's stock × 0.1, 0.5, 2; the
   machine's stock × 0.01 and × 10; the workers' coin × 0.1, 2; the provider's × 0.5, 2. That is
   41 an instance.
2. **joint2** (60) and **joint4** (40): every price × F^u, every s × 2^u, u uniform on [−1, 1]
   from SplitMix64 by seed. The draws follow MARKETS-RULES' wording (O102).
3. **basin**: 1.05^j, j = ±1 … ±43, for w, r, food (the largest share and the exit good), the
   machine, and food's technique. That is 430 an instance.
4. **history**: `cycle(land.mach,1500,80)` and `cycle(commons,1500,80)`, the commons cycling
   across the three regimes.
5. **variants**: the battery under `Hold`, and with every tilt at 1.
6. **tick length**: Tiers 1–2 at 12 and at 365 a year, at their L.
7. **enclose**: `enclose=0.5` and `enclose=1`, at genesis and dated.
8. **negative control**: C1's tape with `inst.chi_max` 0.25 and nothing else, the whole battery.

### 5.5 The engine run, in order (decision 371)

Each step is registered here before any code. The scorer is committed before its wave (decision
311).
- **E0: the build's trace diff.** `cm.tick`, with the genesis carry, against the engine for 2,000
  ticks: C1 `hold`, `p[labour]*2`, `JB(0.5)`, `commons=48.6@genesis` (to Commons) and
  `commons=12.15@genesis` (Enclosed, plots rented); C2 `hold` and `p[labour]*2`; the negative
  control's `p[mach]*0.5` up to its runaway. They must agree within 1e-12 in log. A parting blocks
  scoring until it is explained in the registration, as P2.1's budget-chain ulp was. E0 also
  compares the harness's first joint draw with the mirror's.
- **E1: nesting.** Every pin of §3.6, and tests 1–2 of §3.7.
- **E2: mode A and the base kick sets**, at C1 and C2, 52 a year (and 12 and 365, reported).
- **E3: the verdict battery.** Tiers 1–3 at L, and Tier 3 at 10·L, at C1 and C2, with a kick set
  at every target.
- **E4: stocks** (first, O22), then **E5: joint2, joint4 and basin**, **E6: history**,
  **E7: Hold and tilt 1**, **E8: tick length**, **E9: enclose and the negative control**.

**The verdict**, per instance, is MARKETS-SPEC §7.9's. GO: mode A PASS, and every non-vacuous run
in Tiers 1–3 CONVERGED, with Tier 3 CONVERGED again at 10·L. LOCAL: Tiers 1–2 CONVERGED, some Tier-3
run not. NO-GO: otherwise.

### 5.6 How close the engine must be to the mirror

As P2.1's and L0's engines matched their mirrors:
- every class exactly;
- ticks to tolerance within 10%, with up to three runs an instance within 25%;
- lowest baskets and cleared volumes within 0.05 absolute where the mirror's is above 0.1;
- dead ticks within 10% or 5 ticks;
- regime ticks and switches within 10% or 2;
- r_o at the end within 1e-9 relative (Crowded targets);
- runaway ticks of the trap within 5%.

## 6. Registered predictions

The conditions are §0's, the mirror's runs in `model/runs/`, and `model/predictions.out`.

### 6.1 The verdicts

| | predicted verdict | mode A (52/yr) | Tier 1 | Tier 2 | Tier 3 | Tier 3 at 10·L |
|---|---|---|---|---|---|---|
| **C1** | **GO** | PASS, 2.4e-15 | 30/30 (27 + 3 slack) | 42/42 (39 + 3) | 43/43 (40 + 3) | 43/43 |
| **C2** | **GO** | PASS, 4.4e-16 | 30/30 (27 + 3) | 38/38, 4 VACUOUS | 41/41, 2 VACUOUS | 41/41, 2 VACUOUS |

Mode A also passes at 12 a year (1.6e-14, 2.8e-14) and 365 (3.4e-15, 1.5e-14), with every fill at
least 1 − 3e-14.

### 6.2 The largest root per tick and the kick bars

`model/lin.out` (8 directions, ticks 10,000–40,000). Every kick bar PL^(0.9·L) ≤ 1e-3 passes.

| | base | range over the targets | slowest target |
|---|---|---|---|
| C1 | 0.986116 (half-life 50 ticks) | 0.985378–0.991388 | land.mach × 0.5 (Enclosed), 80 ticks |
| C2 | 0.985988 (49) | 0.985555–0.987888 | land.mach × 0.5, 57 ticks |

(P2.1's I1: 0.98661.)

### 6.3 Speeds and paths, tier by tier

Ticks to tolerance are for non-slack CONVERGED runs. "Lowest baskets" is baskets eaten over Y\*.
"Provider coin" is its lowest over its genesis coin.

| | Tier | ticks to tol, median (slowest) | years | peak D̂ median / worst | dead ticks median / worst | worst buyer fill | lowest baskets (ticks with none) | transfer short in (most ticks) | provider coin low |
|---|---|---|---|---|---|---|---|---|---|
| C1 | 1 | 319 (444) | 6.1 (8.5) | 71 / 504 | 0 / 0 | 0.524 | 0.632 (0) | 1 run (11) | 0.446 |
| C1 | 2 | 372 (557) | 7.2 (10.7) | 311 / 2,184 | 0 / 96 | 0.085 | 0.131 (0) | 10 (94) | 0.247 |
| C1 | 3 | 508 (828) | 9.8 (15.9) | 1,141 / ∞ | 30 / 183 | 0 | 0 (8) | 30 (293) | 0.047 |
| C2 | 1 | 315 (494) | 6.1 (9.5) | 72 / 419 | 0 / 0 | 0.597 | 0.688 (0) | 1 (2) | 0.470 |
| C2 | 2 | 360 (547) | 6.9 (10.5) | 232 / 1,739 | 0 / 79 | 0.134 | 0.205 (0) | 10 (73) | 0.278 |
| C2 | 3 | 514 (641) | 9.9 (12.3) | 1,068 / ∞ | 23 / 153 | 0 | 0 (8) | 27 (196) | 0.100 |

A peak D̂ of ∞ is a tick where some market does not trade (5 C1 and 4 C2 Tier-3 runs). The
slowest runs are C1 b.food = 1.2 (828 ticks) and p[mach] × 0.5 (739), and C2 p[land] × 0.5 (641).
The most dead ticks are C1 JB(2) 183 and p[mach] × 0.5 181, and C2 JB(2) 153.

**The lowest cleared volume over the oracle's, by market** (Tier 1 / Tier 2 / Tier 3):
- C1: labour 0.863 / 0.213 / 0.016; land 0.849 / 0.544 / 0.203; manufactures 0.632 / 0.131 / 0;
  food 0.639 / 0.150 / 0; care 0.713 / 0.143 / 0; shelter 0.640 / 0.156 / 0; mach 0.604 / 0.113 / 0.
- C2: labour 0.759 / 0.349 / 0.043; land 0.875 / 0.607 / 0.278; manufactures 0.688 / 0.205 / 0;
  food 0.697 / 0.236 / 0; care 0.724 / 0.249 / 0; shelter 0.703 / 0.244 / 0; mach 0.658 / 0.176 / 0.

**The regime's readouts** (the runs that visit each regime; switches median / most; r_o/r range;
T_p's largest):

| | Tier | Commons | Crowded | Enclosed | switches | r_o/r | T_p max |
|---|---|---|---|---|---|---|---|
| C1 | 1 | 0 | 30 | 4 | 0 / 2 | 0.168–1 | 0.0099 |
| C1 | 2 | 13 | 38 | 14 | 0 / 6 | 0–1 | 0.057 |
| C1 | 3 | 29 | 39 | 29 | 3 / 6 | 0–1 | 0.225 |
| C2 | 1 | 30 | 0 | 0 | 0 / 0 | 0 | 0 |
| C2 | 2 | 42 | 0 | 0 | 0 / 0 | 0 | 0 |
| C2 | 3 | 41 | 2 | 0 | 0 / 0 | 0–0.731 | 0 |

At every Crowded target the shadow rent ends at the oracle's r_o (C1's base 0.4379200472667323).
At every Commons target it ends at exactly 0, with the commons partly idle.

### 6.4 O14, like for like with I1 (the same mirror and runner)

| | Tier 1 / 2 / 3 median ticks | Tier 2 worst peak D̂ | Tier 2 lowest baskets | Tier 3 dead ticks, worst | largest root |
|---|---|---|---|---|---|
| I1 (dependence form) | 324 / 381 / 525 | 1,288 | 0.276 | 145 | 0.98661 |
| C1 (commons full) | 319 / 372 / 508 | 2,184 | 0.131 | 183 | 0.98612 |
| C2 (commons with room) | 315 / 360 / 514 | 1,739 | 0.205 | 153 | 0.98599 |

The commons converges at I1's speed. Its paths are deeper: Tier 2's worst baskets fall to 0.47
of I1's in C1 and 0.74 in C2, and Tier 3 has 26% (C1) and 6% (C2) more dead ticks at worst.

### 6.5 The families

| family | C1 | C2 | I1 (control) |
|---|---|---|---|
| stocks (41) | 41 CONVERGED; median 419 ticks | 41; 444 | not run |
| joint2 (60) | **56 CONVERGED, 4 DIVERGED** (seeds 3, 12, 34, 39) | 60 | 60 |
| joint4 (40) | **28, 12 DIVERGED** (seeds 3, 6, 10, 12, 17, 18, 22, 26, 28, 31, 34, 39) | **32, 8 DIVERGED** (3, 10, 12, 18, 21, 22, 34, 39) | 40 |
| basin (430) | **341, 89 DIVERGED** | **353, 77 DIVERGED** | 430 |
| Hold (115) | 115 CONVERGED | 109, 6 VACUOUS | — |
| tilt 1 (115) | **112, 3 DIVERGED** (Tier 3: p[mach] × 0.5, JA(0.5), JB(2)) | 109, 6 VACUOUS | — |
| 12 a year, Tiers 1–2 (72) | 72; median 908 ticks = 75.6 years | 68, 4 VACUOUS; 80.2 years | — |
| 365 a year, Tiers 1–2 (72) | 72; median 2,174 ticks = 6.0 years | 68, 4 VACUOUS; 5.9 years | — |
| enclose (4) | 4; 323 ticks | 4; 324 | — |
| negative control, χ_max 0.25 (115) | **100, 9 DIVERGED** (Tier 3), 6 VACUOUS | — | — |

**The basin**, converging from ×/÷ at j contiguous from 1 (`basin_table.py`):

| | w | r | food's price | machine's price | food's technique |
|---|---|---|---|---|---|
| C1 | × 0.231 to × 8.15 | ÷ 8.15 to × 2.407 | ÷ 8.15 to × 2.653 | × 0.481 to × 8.15 | the whole range |
| C2 | × 0.326 to × 8.15 | ÷ 8.15 to × 3.920 | ÷ 8.15 to × 3.072 | × 0.359 to × 8.15 | the whole range |

The Tier-3 factors (× 2, × 0.5) lie inside every basin. The boundary is always on the side that
lowers the wage against food, which raises the exit's value against work: labour cheaper, food
dearer, land dearer, machines cheaper (so the desks automate and hire fewer hours).

### 6.6 The history family

| | cycle(land.mach, 1500, 80) | cycle(commons, 1500, 80) |
|---|---|---|
| C1 | **runaway in window 5 at tick 288** (after 0.2 → 0.4, tick 7,788 of the run); windows 0–4 end within tolerance, median 366 ticks | 81/81 windows within tolerance; median 497 ticks; the targets are Crowded 17, Commons 32, Enclosed 32 |
| C2 | 81/81; median 551 ticks | 81/81; median 0 ticks (every commons shock but × 0.5 leaves the point where it is), at most 315 |

### 6.7 What the rule does through the regimes

C1's commons shocks move the equilibrium across all three regimes, and each converges:
- commons × 1.1 and × 2 → Commons, 438–443 ticks;
- commons × 0.9 → Enclosed with T_p up to 0.039, 324 ticks;
- commons × 0.5 → Enclosed with T_p up to 0.225, 448–449 ticks;
- C2's commons × 0.5 → Crowded, 315 ticks, r_o ending within 5e-15 of the oracle's 0.46595.

Every CONVERGED battery run ends in its target's regime. There, the shadow rent is within 1e-9
of the oracle's at a Crowded target, exactly 0 at a Commons target and exactly r at an Enclosed
one. The shadow rent needs no market: it is exactly 0 wherever the commons has room.

### 6.8 The subsistence trap

**The mechanism** (`model/trace_cycle.py` on C1's history, window 5; and `trace1.py` on the first
choice's p[food] × 0.5):
1. After a shock that leaves the wage low against food's price, the exit's money value
   e₀ = p_food·s₀ approaches the wage (e₀/w 0.80 at the window's first tick).
2. Where the supply is elastic (Commons or Enclosed), participation falls: hours go from 0.43 to
   0.10 in 18 ticks.
3. So food's output falls, to 0.10 of its target by tick 18 and 0.013 by tick 60, and its price
   rises.
4. That raises e₀ further. By tick 60, e₀/w ≈ 1: nobody works.
5. From then on no good is supplied. Every one-sided price steps up under `Saturate`, the wage
   and food's price together. e₀/w stays at 1.01–1.12, the provider's coin drains, and the posted
   prices cross the runaway bound after 254–321 ticks.

Food cannot be bought, so the exit is worth more than work, so nobody makes food. It is a
coordination failure with no way back under these rules. It mirrors the loop's absorbing zero
(O21), with the exit good in place of the machine service.

**Where it is predicted** (§6.5, §6.6):
- 4/60 joint2 and 12/40 joint4 at C1, and 8/40 joint4 at C2;
- 89 and 77 basin runs;
- C1's tilt-1 Tier 3 (3);
- C1's land.mach history, window 5;
- the negative control's Tier 3 (9: JA(0.5), JB(2), RC(0.5), p[land] × 2, p[mach] × 0.5,
  land.mach = 0.8 and b.food = 1.2 at genesis and dated), each a runaway at ticks 279–314.

**Never in** the verdict battery at χ_max 1, the stocks family, the commons history, 12 or 365 a
year, `Hold`, or enclosure. I1 never enters it: 0 of 530 on joint2, joint4 and basin. It is the
priced exit's, and its gain grows with the supply's elasticity. On C1's Tier 3 with the exit at
its band's middle it takes 0/43 at χ_max 1, 4/43 at 0.5 and 11/43 at 0.25 (`diag_chi.out`).

A clamp is no answer (R3). It is O95, for its own scan before the 1750-like instance.

### 6.9 What would refute the frame

Each of these would send the instance back to the frame, with the alternative of decision 363 or a
new instance:
- a class change in E3 (the verdict battery);
- any engine run at a registered target that rests off the oracle's point;
- a trace diff (E0) parting that is not explained by a named rounding;
- the trap in any E3 run, or none of the trap's predicted runs in E5–E6 and E9 falling into it
  (the mechanism would then be the mirror's alone).

## 7. Decisions proposed and open items

Each decision is open to veto; the alternative named is the registered one (R6). New decisions
are numbered from 360, open items from O95.

360. **The open-commons instances are C1 (the commons full, Crowded) and C2 (with room,
     Commons)**, on I1's economy with one priced type in food (§2.1). Their 26 targets span the
     three regimes, lie on the line, have one equilibrium each, and are funded. *Alternative:*
     unit 1e's K1 and K2 on Appendix B's economy, which is I0's roles, uncertified (the good has
     no land).
361. **χ_max 1, I1's.** *Alternative:* χ_max 0.25, with participation 35–44% and every target
     far from the wall, where the mirror predicts 9–12 Tier-3 runs in the subsistence trap.
362. **Phase 2's idle land at zero rent is the commons' idle part.** Idle enclosed land at r = 0
     (the idle stretch; decisions 153, 160, 161) enters no Phase 2 instance under the probe's
     transfer: the provider's baskets are −ν·N there by construction. *Alternative:* a funded
     support on the idle stretch (a 1f transfer, or the workers' own support) with a free-good
     rule for land in markets, in its own frame.
363. **The commons is no market** (§1.4). The commoners hold it and the participation rule gives
     out its plots. The shadow rent is a readout that nobody receives. *Alternative:* the commons
     as a market under `Saturate`: exact while crowded or spilling, running away wherever it has
     room, 18 times slower locally.
364. **The workers' rule is hours = min(max(n(0), N − T_o/h), n(r̂))** (§3.1): decision 149 for one
     priced pop holding its commons, with §3.1's evaluation order and fma. *Alternative:* the
     shadow rent solved by bisection, as the oracle does, with the same supply.
365. **Plots on enclosed land are rented in money** on the land market, first from the workers'
     coin, in one budget chain with their baskets (§3.2). *Alternative:* in kind, decision 152's
     letter, through an untraded home good the provider receives.
366. **The exit good must be in the workers' basket, and land must not be** (load checks 1 and 2).
     *Alternative:* a joint land order for space and plots.
367. **The cost coefficients are land.mach, b.food and the commons**; enclosure by law is a family.
     *Alternative:* the commons shocked only by enclosure (two params in one event).
368. **No new state and ν = 1.** `WorkersState` is unchanged. The regime, shadow rent and plots
     are harness readouts through the rule's own function. The support stays the probe's one
     basket. *Alternative:* ν as a param, and the regime in state.
369. **The harness's targets come from unit 1e's `ParcelEconomy`**, with the land volume T (plots
     included) and labour's S. *Alternative:* the dependence-form harness, with the commons
     instances scored on prices only.
370. **The subsistence trap is registered as the families' prediction, not the verdict's**
     (§6.8). *Alternative:* the families in the verdict, which would make C1 and C2 LOCAL at
     joint4.
371. **The engine run is §5.5's list**, E0–E9, the tolerances of §5.6, and the scorer committed
     before its wave. *Alternative:* the verdict alone (E0–E3).

- **O95. The subsistence trap.** Under the priced exit, with the exit good made with labour and
  valued at its posted price, the agents have a second absorbing state: nobody works, and every
  price inflates. §6.8 gives where and how often. It needs a structural answer, chosen by its own
  mirror scan, before the 1750-like instance, where food is the exit good by decision 151.
  Candidates, unscanned: participation adjusting toward its target (a rate, as the technique's),
  entry for food at zero output, or a storable exit good (O51).
- **O96. Idle enclosed land at r = 0.** It is unfunded under the probe's transfer (−ν·N), and a
  positive-price market cannot rest at 0 (§1.5). It needs a funded support and a free-good rule
  in markets, and waits for an instance that needs it.
- **O97. A commons shared by several types** (1e's T) needs each type's plot demand, which R13
  forbids a pop. It needs the commons as a market (exact while crowded only) or a commons actor.
  The 1750-like instance with several types needs it.
- **O98. The Enclosed regime's accounts.** Rent is paid in money, not in kind. The households'
  split of baskets differs from the home account's by r·T_p/P_s, and the provider's baskets from
  the oracle's. The market allocation is the oracle's.
- **O99. C2's commons shocks × 1.1, × 0.9 and × 2 are VACUOUS by construction.** In the Commons
  regime the commons' size does not move the point.
- **O100. 12 a year is slow here too:** 76–80 years median to tolerance (Tiers 1–2), as I1's 90
  (decision 121).
- **O101. C1's x\* lies 0.0017 below care's edge 0.75,** and C2's 0.0065. b.food × 0.9 crosses it.
  The slack runs depend on which side x\* is.
- **O102. The joint family's draws** follow MARKETS-RULES' wording in the mirror. Its per-seed
  predictions hold only if E0 finds the harness's first draw equal. Otherwise the counts are the
  prediction (C1 4/60 and 12/40, C2 0/60 and 8/40), within 25%.

## 8. What this frame did not do

- No engine code, tape or test. The build is the next step, after the veto window on
  decisions 360–371.
- No engine run. Every number is the mirror's or the oracle's.
- No scan of the trap's remedy (O95), and no run on idle enclosed land (O96).
- The wall-regime instance is framed elsewhere. No target here lies on the wall.

## 9. The headline, in one paragraph

C1 (the commons full, a shadow rent of 0.438 of the land rent) and C2 (the commons with room,
48.8% of it idle at zero rent) are I1 with one priced type in food. They are solved identically,
to 5.6e-15, by the oracle and by an independent 50-digit solve, with one equilibrium at each of 26
targets. The mirror predicts GO for both at C2m and 52 ticks a year: 115/115 and 109/109
non-vacuous runs CONVERGED, medians 319/372/508 and 315/360/514 ticks by tier, largest root
0.986. The prediction rests on one addition, the commons held by its commoners. The scan chose it
over a commons market, which runs away wherever the commons has room. Beside the verdict it
registers a new failure, the subsistence trap. It is absent from I1's 530 runs and present in 4–30%
of the joint and basin runs here.

## 10. Files

`D:/rustyecon-p23/frame-commons/`; sha256 of each in `SHA256SUMS`.
- `SPEC.md` (this file).
- `solve/csolve.py` (the 50-digit solve), `solve/points.py` → `points.out`, `points.json`,
  `targets_table.md`; `solve/explore*.py` → `explore4.out` (the forking path, §2.2).
- `check/Cargo.toml`, `check/src/main.rs` (the oracle crate's solve), `check/run.sh`,
  `check/run_idle.sh` → `check.out`, `idle.out`; `check/compare.py` → `compare.out`.
- `model/p21/`: P2.1's mirror and scripts, unedited (`mm.py`, `mm_carry.py`, `battery.py`,
  `run_battery.py`, `mstab.py`, `modes.py`, `kicks.py`, …).
- `model/cm.py` (the commons mirror), `battery_c.py`, `run_battery_c.py`, `instances.py`.
- `model/nest.py` → `nest.out`; `oracle_check.py` → `oracle_check.out`; `elasticity.py` →
  `elasticity.out`, `.json`; `lin_c.py` → `lin.out`, `.json`.
- `model/scan.py` → `scan.out`, `runs/scan.jsonl` (§1).
- `model/search.py`, `search2.py`, `search3.py` → `search.out`, `search2.out`, `search3.out`; `diag_chi.py` →
  `diag_chi.out`; `diag_families.py`; `negctl.py`; `enclose_points.py`; `basin_table.py`;
  `history.py`; `trace1.py`, `trace_cycle.py`; `smoke.py`, `speed.py`.
- `model/predict_summary.py` → `predictions.out`, `predictions.json`.
- `model/runs/`: `battery_v1.*` (the first choice, χ_max 0.25), `battery_v2.*`, `families_v2.*`,
  `tpy_v2.*`, `history.json`, `diag_dep1*.*`, `negctl.jsonl`, `scan.*`; `battery_v3.*` is
  `battery_v2` rerun on the final `cm.py` (after the scan's last edit, which touches only the
  market candidates): equal in class, ticks, dead ticks, peak, baskets and r_o in all 406 runs.
