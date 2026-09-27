# MARKETS-RULES: the markets probe's agents, as built

Dated 2026-09-27. Steps P2.1.1 and P2.1.2 on branch `phase2-markets`, based on `708167f`.

The Phase 2 probe found that agents deciding at the pinning paper's margins reach Appendix B's
equilibrium from far away (docs/probe/REPORT.md; decisions 38–40). Its report names many markets
as Phase 2 proper's first untested risk. Oracle units 1b and 1c now give known answers for
economies with several final categories and several machine types. The markets probe (P2.1)
asks whether the same roles, with the probe's dials carried to every market, find those answers.
Its frame is `D:/rustyecon-p2m/frame/MARKETS-SPEC.md` (MARKETS-SPEC below, sha256
`9d0859a6…c874`), which fixes the instances, their mapping onto the engine, the known answers,
the protocol and the default dials. The prediction registered before any engine run is
`D:/rustyecon-p2m/predict/PREDICTION.md`.

This file records what was built, the rules as generalised from docs/probe/RULES.md (RULES
below), and how the build was checked before registration:

- what changed, and what did not (§1);
- the economies on the engine (§2) and the four new roles (§3);
- the dials and the genesis they imply (§4);
- the harness (§5);
- the checks of the build: nesting, the trace diff, the probes and mode A (§6);
- where a result depends on decisions 60, 61, 67 and 70 (§7);
- where the build departs from MARKETS-SPEC, and what stays open (§8);
- the registration (§9).

## 1. What changed, and what did not

- **Core, markets and engine: nothing.**
- **Agents** gained four behaviour kinds, `BasketProvider`, `BasketWorkers`, `CategoryDesk` and
  `TypeDesk`, in `crates/agents/src/roles/many/`. They are new variants of `RawSpec` and `Spec`,
  after `MachDesk`. The Appendix B kinds are untouched: the helpers they share with the new kinds
  became crate-visible, and nothing else in `roles/rules.rs` or `roles/spec.rs` changed.
- **No new state.** Each new kind keeps its Appendix B kind's `ActorState`: `Provider`,
  `Workers`, `GoodDesk`, `MachDesk`. Certify's `Obs::transfers` and the harness's state readers
  read them unchanged.
- **The probe** gained `probe::markets` (`crates/probe/src/markets/`) and two binaries,
  `markets-tape` and `markets`. P2.0's harness (`probe::harness`, `probe::setup`,
  `probe::perturb`) is unchanged and still makes docs/probe/results/ byte for byte.
- **The GUI** names the four kinds in its inspector (`crates/gui/src/vm/inspector.rs`).
- **Tapes:** `tapes/markets-<id>.ron` for I0, I1, I2, I3, L2, L3 and G1, each its generator's
  output.
- **The schema stays 1** (docs/TAPE.md, the row for P2.1.1).
- **Hashes.** `tapes/appb.ron` keeps its canonical form, `tape_hash` and `world_id`, and its
  20,000 per-tick hashes are unchanged (final `0xe1fa082b26995867`), as is the gate world's
  stream (final `0x61f9c8529131ff17`). The GUI's goldens and certify's committed certificates
  pin both, and `scripts/gate.sh` recomputes them.

## 2. The economies on the engine

One node, `home`, quotes in `coin`. The goods:

| good | life | who makes it | market rate |
|---|---|---|---|
| `labour` | Instant | `workers`, as an endowment in `decide` | `rate.labour` |
| `land` | Instant | `provider`, as an endowment in `decide` | `rate.land` |
| one per category with tasks (`good`; or `manufactures`, `food`, `care`, `shelter`) | Ticks(1), through `life.one_tick` | its category desk, in `produce` | `rate.<category>` |
| one per machine type (`mach`; or `engine`, `power`) | Ticks(1) | its type desk, in `produce` | `rate.<type>` |

The actors:

| actor | kind | class | spec |
|---|---|---|---|
| `desk.<category>` | Desk | `<category>_desks` | `CategoryDesk` |
| `desk.<type>` | Desk | `<type>_desks` | `TypeDesk` |
| `provider` | Pop | `owners` | `BasketProvider` |
| `workers` | Pop | `workers` | `BasketWorkers` |

One class per desk keeps rationing lines naming the buyer (R12). On I0 the keys are appb's:
`desk.good` in `good_desks`, `desk.mach` in `mach_desks`, the goods `good` and `mach`. So I0's
goods, classes and actors have appb's ids.

The tick is ENGINE §7.2's, as for P2.0 (RULES §1). A produced good made at t sells and is used
at t + 1 and dies at 5a of t + 1. A genesis lot of a Ticks(1) good sells twice, at tick 0 and
tick 1, and a buyer's unused purchase of it lives to tick 1. In I2 and I3 power made at t − 1 is
bought by the engine desk at t, whose service the category desks buy at t + 1. In L2 and L3 the
engine's service made at t − 1 is also power's input at t: a two-tick loop through two markets
whose supply is fixed within the tick.

## 3. The four roles

Notation, per tick at posted prices at the home node: w and r are labour's and land's prices,
p_j a category's good's, p_k a type's service's. share(v) is `Clock::share`, 1 − exp(−v/tpy),
and C is the actor's coin as the phase began. No rule reads a volume, a fill, another actor's
holding, orders or state, or the oracle (R13).

### Households: `BasketProvider` and `BasketWorkers`

They are RULES §2's provider and workers with the basket generalised. A basket is a list of
items (good, weight z_j). Space is an item whose good is `land`.

- **Basket price.** P_s = Σ_j z_j·p_j, summed from 0.0 in item order, with space at r.
- **Buying.** From a budget B they buy n = B/P_s baskets: z_j·n of each item, each with budget
  p_j·(z_j·n), cut from B by the budget chain (below).
- **Eating.** In produce they eat min_j(held_j/z_j) baskets, by `max_scale` over the items whose
  weight is not zero, and burn z_j·n of each as `Consumption`. What is not eaten dies at 5a.
- **Participation** (workers) is min(ln1p(w/P_s)/χ_max, 1) of N, with this P_s.
- **The transfer** (provider) is min(N·P_s, C), recorded as due and paid (R12). The provider then
  spends share(`spend.provider`) of the coin the transfer left; the workers spend
  share(`spend.workers`) of theirs.

### Category desk: `CategoryDesk`

It is RULES §2's good desk on its segments of the shared task line (decision 60), with direct
land, buying the task type's service at efficiency θ_τ.

- **Recipe.** At threshold x = 1 − s, per unit of output (unit-1b.md §4.1, in its evaluation
  order): for each segment [e_{i−1}, e_i] in line order with density μ,
  - if x ≥ e_i, task services M += μ·(J(e_i) − J(e_{i−1}));
  - if x ≤ e_{i−1}, hours H += μ·(e_i − e_{i−1});
  - otherwise H += μ·(e_i − x) and M += μ·(J(x) − J(e_{i−1})).

  On the top segment, whenever x > e_{S−1}, the hours are μ·s from the carried s, never
  μ·(1.0 − x), so a small s keeps its relative precision. J(0) is 0.0 exactly. The desk buys
  M/θ_τ of the task type's service and b_j land per unit.
- **Technique** (decision 61: a continuous threshold, not cells). The state is s = 1 − x. The
  target is 1 − X, X the measure of the line on which a machine of the task type is cheaper at
  posted prices, γ(X) = θ_τ·w/p_τ, computed as RULES' `measure((θ·w)/p_τ)`. Each tick s closes
  share(`adjust.technique.<j>`) of its gap. The rule reads prices only, not the desk's own
  densities, so every desk's threshold rests at one x\*.
- **Unit cost.** c_j = ((w·H) + (p_τ·M/θ)) + r·b_j at posted prices and the planned x; the
  markup is p_j/c_j.
- **Scale.** RULES' cash rule: it spends share(v·μ^κ)·C, v its turnover `buffer.desk.<j>.cash`
  and κ its tilt `tilt.desk.<j>`, and plans q = outlay/c_j. It offers all it holds of its good.
- **Orders.** Hours whenever the category has hours on the line (L̄_j > 0), and services
  whenever it has any density, each at quantity 0 where the planned x makes the coefficient 0
  (the good desk's behaviour at its corners); land only where b_j > 0. Quantities are H·q,
  (M/θ)·q and b_j·q, and budgets p·(coef·q).
- **Produce.** Leontief at the planned x over the inputs whose coefficient is not zero, as RULES
  says: y = min(L/H, S/(M/θ), R/b_j). It burns H·y hours, (M/θ)·y services and b_j·y land, and
  mints y of its good.
- **State** (the good desk's): `share` the plan, `used` the share production used (the plan),
  `scale` the last q, `output` the last y.

### Type desk: `TypeDesk`

It is RULES §2's machine desk with bought machine services (decision 67: machine recipes use
machine services, labour and land only).

- **Recipe.** Per unit of its service: a_kk of its own service (kept, not bought), a_kl of each
  listed type l (bought on l's market), λ_k hours and b_k land.
- **Cost and markup.** c_k = ((0.0 + Σ_l a_kl·p_l) + λ_k·w) + b_k·r, the bought services in
  list order; μ = p_k(1 − a_kk)/c_k, the net form.
- **Scale.** The cash rule. It keeps min(a_kk·q, K) of the K it made last tick and offers the
  rest. It orders every input on the whole q (RULES' graft 4, applied to bought services too):
  a_kl·q of each service, λ_k·q hours and b_k·q land, each with budget p·(coef·q). The hours and
  land orders are posted even where their coefficient is 0, at quantity 0, as the machine desk's
  are.
- **Produce.** y = min(held own/a_kk, held_l/a_kl, L/λ_k, R/b_k) over the nonzero coefficients.
  The own-input burn comes first, so it takes last tick's lots.

### Budgets: one outlay cut into many

MARKETS-SPEC §2.6 generalises RULES' `two_budgets` to a chain over the wanted budgets:

- b_1 = min(w_1, the outlay, the coin the copy holds);
- b_i = min(w_i, `max_remainder`(outlay, b_1 + … + b_{i−1}), the coin the copy still holds).

Each is taken from the copy of the holding. **The chain runs in the order admission takes the
budgets: by good** (admission walks an actor's orders by (node, good), and every role trades at
its home node). So the copy's coin falls through exactly the subtractions admission's does, and
no budget is refused whatever the prices. The budgets come back in the order the inputs are
listed. This is a departure from MARKETS-SPEC's wording, which chains in list order (§8 item 1).
In list order, a bought service listed before hours can take its budget first while admission
takes the hours first. Then the rounded sum b_1 + b_2 can pass the coin by half an ulp of the
outlay, and admission refuses the second. `many_roles_never_overbudget_or_overdraw` found it: the
engine desk of I2 with a share that rounds to 1.

### Nesting: the new kinds on Appendix B

On Appendix B (I0) the new kinds make the Appendix B kinds' floating-point operations exactly,
by MARKETS-SPEC §2.4–§2.6's evaluation orders:

- the basket [(good, 1), (land, h)] gives (0.0 + 1.0·p) + h·r = p + h·r, budgets p·(1.0·n) =
  p·n and r·(h·n), and burns 1.0·n = n and h·n;
- one segment of density 1 gives H = 1.0·s = s and M = 1.0·(J(x) − 0.0) = J(x);
- θ = 1 gives (1.0·w)/p_m = w/p_m and M/1.0 = M;
- the cost ((w·s) + (p_m·J)) + r·0.0 is s·w + J·p_m, the machine desk's ((0.0 + λ·w) + b·r) is
  λ·w + b·r;
- the budget chain over two inputs listed in good order is `two_budgets`.

Two tests check it (§6.1): the rule-level test over 3,000 random states, and the run-level test,
bit for bit over 20,000 ticks and four displaced starts.

### Why the rest point is the oracle's

MARKETS-SPEC §4.1 argues it: at a live rest point every market clears, every actor spends its
income and nothing spoils. With these rules, each type desk's coin is constant only where
p_k(1 − a_kk) = c_k (unit-1c.md §4.2 at u = 1), each category desk's only where p_j = c_j
(unit-1b.md §4.2 with π = p_τ/θ), and every threshold rests where γ(x) = θ·w/p_τ. Participation,
the transfer and the basket are unit 1b's closure. So the live rest point satisfies the oracle's
equations, which have one solution at ρ = 0 (decision 70), and no rule has a band. §6.2 checks it
on the engine at every instance, target and dial set.

## 4. The dials and genesis

### C2m and C2L (MARKETS-SPEC §5)

Every dial is a tape param with a unit and a basis (R4). **C2m** carries design-analytic-first's
C2 one for one to every market and desk of its role:

| key | unit | value | per tick at 52/yr |
|---|---|---|---|
| `rate.labour` | RatePerYear | 5.2 | k 0.1 |
| `rate.<category>` | RatePerYear | 2.6 | k 0.05 |
| `rate.land` | RatePerYear | 1.3 | k 0.025 |
| `rate.<type>` | RatePerYear | 1.3 | k 0.025 |
| `adjust.technique.<category>` | RatePerYear | 2.6 | share 0.048771 |
| `buffer.desk.<desk>.cash` (turnover) | RatePerYear | 5.2 | share 0.095163 |
| `tilt.desk.<desk>` | Dimensionless | 0 | — |
| `spend.workers`, `spend.provider` | RatePerYear | 13 | share 0.221199 |
| `price.ema_tc` | Years | 0.5 | read by no rule |

Basis: `Assumed("MARKETS-SPEC C2m, design-analytic-first C2 per role")`. **C2L**, for L2 and L3,
is C2m with every `rate.<type>` at 5.2 a year and every `tilt.desk.<desk>` at 1, with the basis
`Assumed("MARKETS-SPEC C2L: type rates at labour's, markup tilt 1 (§5.2)")`. The registered
tapes are at C2m; `--dials c2l` writes C2L. `--set KEY=VALUE` sets one dial, `rate.*`,
`buffer.*` and `adjust.*` scale a family, and `tilt.*` sets every tilt, as P2.0's `--set` does.

### The instances and their params

The instances are MARKETS-SPEC §1.2's, and `probe::markets::instance` holds them. Every
coefficient is a `Dimensionless` param, live, keyed as §2.9 suggests:

- the scalars: `inst.workers`, `inst.land` (FlowPerYear), `inst.chi_max`, `inst.eta`, `inst.g0`,
  `inst.g1`, `inst.k`;
- the line: `inst.edge.<s>` for the interior edges;
- each category j: `inst.<j>.weight`, `inst.<j>.land`, `inst.<j>.mu.<s>`, and `inst.space.weight`
  where households buy space;
- each type k: `inst.<k>.own`, `inst.<k>.in.<l>` for each type it buys, `inst.<k>.labour`,
  `inst.<k>.land`, and `inst.<k>.theta` for the task type.

A param no rule reads does not load, so only the task type has a θ, and a type lists only the
services it buys. The bases are §2.9's: C1's and C3's categories, the gap economy, M4's operating
recipes, M4's operating plus build recipes, and SSRN Appendix B for the scalars and 1a's
machine. L2's rows are written as the flow totals (engine: 0.1 own, 0.5 power, 0.12 hours, 0.4
land; power: 0.1 engine, 0.12 hours, 0.6 land), and the oracle sees the same numbers.

### Genesis (MARKETS-SPEC §5.4)

The generator (`markets-tape`) writes each tape from the oracle's point, unit 1c solved in the
harness with every type's flow recipe as its operating recipe (MARKETS-SPEC §1.2, M9):

- prices relative to r = 1 coin;
- each category desk's human share 1 − x\*;
- each desk's stock one tick's output: z_j·Y for a category desk, X_k for a type desk;
- each actor's stationary coin, summed in the order its rule sums:
  - a category desk, p_j·y_j/share(turnover);
  - a type desk, c_k·X_k/share(turnover), c_k its purchased-input cost;
  - the provider, N·P_s + (r·T − N·P_s)/share(spend.provider);
  - the workers, (N·P_s + w·N·F)/share(spend.workers).

They equal MARKETS-SPEC §5.4's table to every digit it prints (coin, r = 1):

| id | category desks | type desks | provider | workers |
|---|---|---|---|---|
| I0 | good 29.94282457255491 | mach 23.78417887464776 | 26.03270836105176 | 27.922094099033625 |
| I1 | manufactures 3.193704800042774, food 52.116851346228074, care 2.643468355164404, shelter 51.633485618082936 | mach 19.332568574007652 | 19.963368918929355 | 34.352655089299446 |
| I2 | good 7.464878882844522 | engine 7.007226056680158, power 6.939278290805243 | 30.054062732330266 | 19.69892610159803 |
| I3 | manufactures 0.7695982986754858, food 45.92741623439125, care 1.7876781245883648, shelter 56.92466673355374 | engine 5.279384267116784, power 5.233632437807863 | 24.101233330118085 | 27.24204097155671 |
| L2 | good 23.627560346528167 | engine 21.69099700625093, power 9.792244219584179 | 27.28328589309904 | 25.23706888467798 |
| L3 | manufactures 2.468825110586404, food 50.098204738226464, care 2.4042583242640276, shelter 52.96682384842411 | engine 17.12521157486544, power 7.753707769983283 | 21.295073036977016 | 31.933122117354404 |
| G1 | manufactures 3.57891680922062, food 46.59308038427203, care 1.952329570852621, shelter 56.40228858140203 | mach 6.5613474835711365 | 17.74561402849376 | 36.74389895692036 |

**I0's genesis is appb's, bit for bit.** Unit 1c's operating form of I0 gives unit 1a's G1 point
to the bit in every value genesis uses (x\*, 1 − x\*, v, p_m, p, P_s, Y, K, N_a, Y·J), at 12, 52
and 365 a year and at every cost target (`D:/rustyecon-p2m/build/ocheck/`). The generator sums
the coins in appb's order, so I0's genesis prices, holdings and share equal appb.ron's
(`markets_i0_genesis_is_appb`).

## 5. The harness: `probe::markets`

```sh
markets run NAME...     [options]  # named runs, one summary line each
markets family FAMILY   [options]  # battery, stocks, joint2, joint4, basin, history
markets list FAMILY     [options]  # names; the battery with its tier and slack runs
markets tape [NAME]     [options]  # the tape a run is made from
markets kick NAME...    [options]  # the kick set at the end of each named run (§7.5)
markets elasticity      [options]  # the one-tick elasticity probe and L (§7.8)
markets openloop        [options]  # the open-loop probe, prices frozen (§7.8)
markets point           [options]  # the oracle's point at the base and each cost target
options: --inst ID --tpy N --dials c2m|c2l --set KEY=VALUE --one-sided saturate|hold
         --ticks L --csv DIR --every K --jobs J --horizon H --h X
markets-tape --inst ID [options] [--perturb NAME --ticks L] [PATH]
```

Build under WSL with `CARGO_TARGET_DIR=/root/scratch/target-p2m-<label>`, and put large outputs
under `D:/rustyecon-p2m/runs/`.

### Run names (MARKETS-SPEC §7.7)

A name is one or more terms joined by `+`:

| term | what it does |
|---|---|
| `hold` | nothing: mode A |
| `p[M]*F`, `w*F`, `r*F` | market M's genesis price times F (`w` is labour's, `r` land's) |
| `s[D]*F` | category desk D's genesis human share times F |
| `x*/2` | every category desk's x = x\*/2 |
| `JA(F)` | w and every good and type price times F, r fixed; every s times F |
| `JB(F)` | w times F, every type price times 1/F, every good price times F, every s times 1/F |
| `N(F)` | every price times F, coin unchanged |
| `RC(F)` | the first half of the categories' prices times F, the rest times 1/F (manufactures and food, care and shelter) |
| `RT(F)` | the first type's price times F, the second's times 1/F (engine, power) |
| `C=V@genesis` | cost coefficient C (MARKETS-SPEC §1.5's name, or its param) is V from tick 0; genesis stays at the registered point |
| `C=V@dated` | C becomes V by a dated `SetParam` at L/4; the scored clock restarts there |
| `coin.A*F`, `stock.D*F` | actor A's genesis coin, or desk D's stock of its output, times F |
| `joint(F,SEED)` | every price times F^u, then every s times 2^u, u uniform on [−1, 1] from SplitMix64 seeded by SEED |
| `cycle(C,P,N)` | N dated changes of C, P ticks apart from tick P, cycling ×1.1, ×0.9, ×2, ×0.5, ×1 of its registered value |

On I0 the grammar is P2.0's with the markets named: `p[mach]` is `pm`, `p[good]` is `p`,
`s[good]` is `s`, `stock.mach` is `mach`, `land.mach` is `b`, and `joint` draws in P2.0's order.

The cost coefficients (MARKETS-SPEC §1.5): I0 and G1 `land.mach`; I1 `land.mach`, `b.food`; I2
`land.power`, `a.engine.power`; I3 `land.power`, `b.food`; L2 `land.engine`, `land.power`; L3
`land.engine`, `b.food`. Each is shocked to the values §1.5 writes.

### The battery (MARKETS-SPEC §7.7)

`markets list battery --inst ID` prints it, with each run's tier and its slack mark. A slack run
is a technique displacement that leaves the desk's recipe unchanged: no segment the desk has
tasks on meets the open interval between x\* and the displaced threshold.

| id | runs | Tier 1 | Tier 2 | Tier 3 | slack |
|---|---|---|---|---|---|
| I0 | 57 | 16 | 20 | 21 | 0 |
| I1 | 107 | 30 | 38 | 39 | 10 |
| I2 | 77 | 20 | 28 | 29 | 0 |
| I3 | 119 | 34 | 42 | 43 | 16 |
| L2 | 77 | 20 | 28 | 29 | 0 |
| L3 | 119 | 34 | 42 | 43 | 9 |
| G1 | 99 | 30 | 34 | 35 | 17 |

The slack runs are the frame's (`slack.out`) for I1, I3, L3 and G1. I0's 57 runs are P2.0's 57
by name, translated. G1 has one cost coefficient, so 99 runs, not §7.7's 107. In G1, whose
1 − x\* is 0.545, six runs would put 1 − x above 1 and cannot be expressed: each `s[j]*2`, `JA(2)`
and `JB(0.5)`. The generator refuses them rather than clamp. Two of them, `s[care]*2` and
`s[shelter]*2`, are also slack.

The families of §7.10: `stocks` (each desk's coin ×0.02, 0.1, 0.5, 2; each desk's stock ×0.1,
0.5, 2; each type's stock ×0.01 and ×10; the workers' coin ×0.1, 2; the provider's ×0.5, 2),
`joint2` (60), `joint4` (40), `basin` (1.05^j for j = ±1 … ±43, for w, r, the largest-share
good, each type and the technique of the desk with the most tasks at x\*), and `history`
(`cycle(<first coefficient>,1500,80)`). The rate scans and the map are `--set` options, and
the tick lengths `--tpy`.

### Observables, targets and classes (MARKETS-SPEC §7.1–§7.5)

Every tick the harness reads, from the `TickReport` through certify's `Obs` and the actors' own
state (agents see none of it):

- the observables O: v = w/r; π_k = p_k/r for each type; π_j = p_j/r for each category;
  s_j, the share each category desk's production used; the cleared volume of every market; and
  each desk's output. On I0 that is P2.0's set of ten, in P2.0's order. I1 has 22, I2 13, I3 25,
  L2 13, L3 25 and G1 22 (§7.1).
- the oracle's value of each at the coefficients in force, solved in the harness (unit 1c) at
  genesis and at each dated shock. A type's market clears its task services plus what the other
  types buy of it; its own input is kept, not traded.
- each gap |ln(o/o\*)| and D̂ = max_o gap/1e-3 (every tol_o is the 1e-3 floor: no dial is a
  band).
- S, D, both fills and spoilage per market; what `Consumption` burned of each basket item; each
  actor's coin; each category desk's planned share; the transfer, due and paid; each household's
  baskets and the item that bound them; the ledger margin; the dead flag.

With `--csv DIR` each run writes `DIR/<run>.csv`, one row per tick (or every K-th and the last).

The classes are PROBE-SPEC §4.5's, with P2.0's definitions (`probe::harness::classify`) kept
tick by tick, so a run of any length holds a fixed amount of memory: ERROR, DIVERGED (a
non-finite price, the runaway bound [1e-6, 1e6] × genesis, or a growing envelope), DEAD, VACUOUS,
CONVERGED, STUCK, ORBITING. On I0 the classes, D̂_0, the envelope, W's largest D̂, the dead
ticks, the troughs and the transfer shortfall are P2.0's bit for bit (§6.1).

A tick is dead if any market does not trade or clears less than 0.5 of its oracle volume (§7.2).

**The kick** (§7.5, MG7): `markets kick NAME` runs the named run to its end, takes a
checkpoint, and runs certify's kick set there, `certify::kick::kick_segment`: the base
continuation and each market's posted price × (1 ± 1e-9) through `ScalePrice`, each for H ticks,
judged by `certify::battery::kick` with `criteria/appb-2026-09-26.ron`'s bars (`gain_tail` ≤
1e-3 over the last tenth of H, `gain_peak` ≤ 1e6). `--csv DIR` writes each kick's readings.
Scoring CONVERGED against the kick set of its target is the run step's (§7.5).

### Transient statistics (MARKETS-SPEC §7.11, O14)

`summary.tsv` has one line per run: the class and PROBE-SPEC's numbers, then the peak D̂ and
its tick, the worst fill, the trough of baskets eaten over Y\* with its tick and the ticks with
none, the transfer shortfall and its ticks, the path depth for a cost shock, the market with the
lowest cleared volume, and mode A's PASS or FAIL. `stats.tsv` holds every statistic of §7.11 in
long form (run, statistic, where, value), over the scored run and against the oracle at the
coefficients in force:

- `dead.no_trade`, `dead.below_floor`, `dead.no_supply`, `dead.no_demand` per market;
- `trough.cleared`, `trough.tick`, `trough.end` per market (cleared over oracle volume, certify's
  first-minimum rule);
- `trough.output`, `trough.output_tick` per desk;
- `baskets.trough`, `baskets.trough_tick`, `baskets.none_ticks`; `item.trough` and
  `item.none_ticks` per item (what `Consumption` burned over z_j·Y\*);
- `binding.ticks` per item and `binding.short_ticks`: among household ticks with baskets below
  0.99 of that household's oracle baskets, how often each item bound;
- `ration.worst`, `ration.rationed_ticks`, `ration.budget_short`, `ration.market_short` per
  (market, class, side), as CERTIFY §6's reports define them;
- `spoiled.total` and `spoiled.share` (Σ spoiled over Σ supplied) per market good;
- `transfer.short`, `transfer.short_ticks`;
- for a cost shock, `depth.equilibrium` (ln Y′/Y) and the trough of baskets in log against Y and
  against Y′.

A household's baskets are computed in the harness from its class's filled quantity of each item
that tick, min_j `max_scale`(filled_j, z_j), as its rule eats them. It holds nothing else of
them when it eats, since every item lives one tick, except at tick 1, when an unused purchase of
a genesis lot is still held.

### The probes (MARKETS-SPEC §7.8)

- `markets elasticity`: one tick from the mode-A genesis with each market's price × e^(±0.01);
  the central difference of ln S and ln D of every market; each market's multiplier 1 + k·(ε_D −
  ε_S) and τ = 1/(k·|ε_D − ε_S|); L = 200·τ_max rounded up to a thousand ticks, at least 20,000
  (PROBE-SPEC §4.4).
- `markets openloop`: every price rate 0; each market's price × e^(±0.01) at genesis; the
  response d ln(D/S) of every market and d ln(coin) of every actor at lags 0, 1, 5, 20 and 200;
  and the undisplaced frozen run's ln(D/S).

## 6. Checks of the build

All runs below are on WSL, in a release build of the P2.1.1 tree, at 52 ticks a year unless
said. Scripts and outputs are in `D:/rustyecon-p2m/build/`.

### 6.1 Nesting: I0 is P2.0

- **Run level** (`markets_i0_nests_appb`, in the gate). I0 in the new kinds against appb in the
  Appendix B kinds, through the two harnesses. Every tick's posted prices, observables, targets,
  gaps, D̂, supply, demand, both fills, spoilage, coins, planned share, transfer and ledger margin
  are equal bit for bit: for 20,000 ticks from the registered genesis, and for 3,000 ticks each
  from `w*2`, `JB(0.5)`, `b=0.2@genesis` (`land.mach=0.2@genesis`) and `mach*0.1`
  (`stock.mach*0.1`). The classes, D̂_0, envelopes, W's largest D̂, dead ticks, troughs and
  transfer shortfalls agree too. The state hashes differ, because the spec variants differ.
- **Rule level** (`many_roles_nest_the_appendix_b_roles`). From 3,000 random states (prices over
  twelve decades, coins, stocks, the technique at 0, 1 and between, spending, turnover and
  technique rates, tilts up to 8), every actor's `decide` and `produce` on I0's tape equal its
  Appendix B kind's on appb's, order for order and delta for delta.
- **The elasticity probe** on I0 gives P2.0's to every digit printed (τ 7.4, 47.5, 44.6 and 67.7
  ticks at 52 a year; 13.0 at 12; 519.4 at 365).

### 6.2 The rest point is the oracle's

`markets_rest_point_is_the_oracles`: at every instance, at C2m and C2L, and at every registered
cost target (62 points per dial set), three ticks from genesis at the oracle's f64 point leave
every observable within 1e-12 of the oracle in log, with every market trading and filled.

### 6.3 Arithmetic that cannot fail

`many_roles_never_overbudget_or_overdraw` draws 3,000 states on I2 and 3,000 on L3 (prices over
twelve decades, coins over eighteen, spending shares that round to 1, tilts up to 8, arbitrary
holdings of every input) and checks that admission accepts every order and every burn fits. It
found the budget chain's order defect of §3 before the fix.

### 6.4 The trace diff (MARKETS-SPEC §7.8)

`D:/rustyecon-p2m/build/tracediff.py` runs a float64 mirror of §3's roles from each Rust run's
own genesis and compares every observable in log, every tick, for 2,000 ticks. The mirror is
`mm_carry.py`, which `make_mm_carry.py` writes from the predictor's `mm.py` (sha256
`528756c1…b407`, read-only). It adds the one carry mm.py leaves out: a buyer's unused purchase
of a genesis lot, which the engine keeps to tick 1 (PREDICTION §10 discloses it). mm.py already
carries the seller's unsold lot and a type desk's unburned kept input, as P2.0's
`model_genesis2.py` did. The spec names the frame's `mirror.py` with the carry added; mm.py is
the independent mirror that has the carry, and the two mirrors agree on PL to 5e-4 (PREDICTION
§1.2). Output: `tracediff.out`.

| run | ticks | largest gap in log | first above 1e-12 | largest gap but cleared volumes | the mirror against itself, one ulp apart |
|---|---|---|---|---|---|
| I1 hold | 2,000 | 4.4e-16 | — | 4.4e-16 | 3.3e-16 |
| I1 w×2 | 2,000 | 1.7e-14 | — | 1.7e-14 | 1.5e-14 |
| I1 JB(0.5) | 2,000 | 1.1e-14 | — | 1.1e-14 | 1.4e-14 |
| I1 land.mach = 0.8 at genesis | 2,000 | 7.2e-15 | — | 7.2e-15 | 1.0e-14 |
| I2 hold | 2,000 | 2.2e-16 | — | 2.2e-16 | 5.6e-16 |
| I2 w×2 | 2,000 | 2.5e-14 | — | 2.5e-14 | 2.8e-14 |
| I2 JB(0.5) | 2,000 | 3.6e-14 | — | 3.6e-14 | 3.7e-14 |
| I2 land.power = 1 at genesis | 2,000 | 1.8e-14 | — | 1.8e-14 | 4.4e-16 |
| L2 hold | 2,000 | 2.2e-16 | — | 2.2e-16 | ∞ |
| L2 w×2 | 292 | 7.0e-10 | tick 189, cleared land | 3.3e-13 | 1.7e-13 |
| L2 JB(0.5) | 291 | 6.7e-10 | tick 199, cleared land | 1.7e-13 | 1.4e-13 |
| L2 land.engine = 0.8 at genesis | 286 | 6.0e-10 | tick 181, cleared land | 8.0e-14 | 2.0e-13 |

**The build is the mirror's map.** I1 and I2 agree within 3.6e-14 everywhere, as closely as the
mirror agrees with itself from one ulp away.

**L2's three displaced runs** agree within 3.3e-13 on every price, technique and output until
they stop at the runaway bound (DIVERGED at ticks 286–292). Every coin, S and D agrees within
about 1e-14. One observable departs: the cleared land volume, from ticks 181–199, and only on
isolated ticks. By then the loop has collapsed and the land market clears 0.2% of what is
offered, most of it the workers' space. The cause is §3's budget chain, which the mirror does not
model. The workers' land budget is the second of their pair, capped at `max_remainder(outlay,
the good's budget)`, which can be an ulp of the outlay below p·q. When land is 1e-3 of the
outlay, that ulp is 1e-12 of the land bought. `tracediag.py` and `tracediag2.py` split the land
demand by buyer to show it. It is an explained disagreement in a measured quantity, not in the
map, and it does not block scoring. The "∞" in L2 hold's last column is the mirror itself: one
ulp away from L2's point, at C2m, it collapses within 2,000 ticks, while the build and the mirror
from the exact point stay there to the bit.

### 6.5 The elasticity probe (MARKETS-SPEC §7.8, §6.4)

From the engine, per instance and dial set (`D:/rustyecon-p2m/build/probes/el-*.out`); τ in
ticks, and L by PROBE-SPEC §4.4's rule:

| id | dials | the slowest markets at 52/yr (τ) | L at 52 | L at 12 | L at 365 |
|---|---|---|---|---|---|
| I0 | C2m | good 67.7, land 47.5, mach 44.6; labour 7.4 | 20,000 | 20,000 | 104,000 |
| I1 | C2m | care 719.7, manufactures 595.7, mach 82.6; labour 6.1 | 144,000 | 27,000 | 1,137,000 |
| I2 | C2m | good 258.6, power 40.4, land 39.4, engine 38.4; labour 4.4 | 52,000 | 20,000 | 392,000 |
| I3 | C2m | manufactures 2,419.4, care 1,041.6, engine 180.5; labour 7.5 | 484,000 | 91,000 | 3,767,000 |
| L2 | C2m | power 88.6, good 84.3, land 48.4, engine 43.4; labour 7.2 | 20,000 | 20,000 | 129,000 |
| L2 | C2L | good 84.3, land 43.0, power 11.4, engine 6.2; labour 6.8 | 20,000 | 20,000 | 129,000 |
| L3 | C2m | care 783.3, manufactures 762.8, engine 92.9, power 88.3; labour 5.5 | 157,000 | 29,000 | 1,232,000 |
| L3 | C2L | care 783.3, manufactures 762.8, food 37.6; labour 4.9 | 157,000 | 29,000 | 1,232,000 |
| G1 | C2m | care 952.6, manufactures 519.7, mach 69.6; labour 10.2 | 191,000 | 35,000 | 1,521,000 |

These equal MARKETS-SPEC §6.4's τ and L at 52 a year for every instance to the digits it prints
(the frame's came from its mirror), and its I1 and I3 figures at 365. As the frame says, the rule is conservative for
small-share categories: their within-tick demand elasticity is −z_j·p_j/P_s.

### 6.6 The open-loop probe

Prices frozen (`D:/rustyecon-p2m/build/probes/ol-*.out`), a 1% move in the land price:

- **I0–I3 and G1** drift slowly. The response of any market at lag 200 is at most about 70, as
  P2.0's I0 was.
- **L2 and L3 grow.** L2's engine, power and good markets go from about 1 at lag 5 to 22–25 at
  lag 20 at C2m, about 1.2 a tick, and reach ±∞ (a market with no trade) by lag 200. The same
  holds from every price. It is the quantity loop MARKETS-SPEC §6.3 predicts (1.19 a tick in its
  mirror). C2L's tilt does not damp it with prices frozen.
- **Coins in the loop.** In L2, as the engine and power markets go short, power's coin falls
  with them (−8.7 per unit at lag 20 from the land price) and the good desk's barely moves
  (+0.27). That is PREDICTION §3.5's reading, that the growth runs through the power desk's
  cash rule, rather than the frame's rationed desk that keeps its coin.

### 6.7 Mode A (MARKETS-SPEC §7.6)

From the oracle's f64 point with the stationary coins, at L by the rule (§6.5), every
observable within 1e-9 of the oracle in log, every market trading with every fill at least
1 − 1e-9, spoilage at most 1e-9 of volume, the ledger clean
(`D:/rustyecon-p2m/build/modea/`):

Each cell is PASS with the largest gap in log over the run, or FAIL with the first tick a gap
passed 1e-9 and the run's class.

| id | dials | 12 a year | 52 a year | 365 a year |
|---|---|---|---|---|
| I0 | C2m | PASS, 1.9e-15 | PASS, 8.9e-16 | PASS, 8.9e-16 |
| I1 | C2m | PASS, 3.3e-16 | PASS, 7.8e-16 | PASS, 1.2e-15 |
| I2 | C2m | **FAIL** at tick 2,427 (π_good); DIVERGED, the envelope grows | PASS, 7.8e-16 | PASS, 1.2e-15 |
| I3 | C2m | PASS, 3.0e-13 | PASS, 1.7e-15 | PASS, 5.6e-16 |
| L2 | C2m | **FAIL** at tick 251 (y.power); DIVERGED, runaway at tick 720 | PASS, 3.3e-16 (a bit-exact fixed point) | PASS, 3.3e-16 (a bit-exact fixed point) |
| L2 | C2L | PASS, 5.1e-15 | PASS, 2.2e-16 | PASS, 3.3e-16 |
| L3 | C2m | **FAIL** at tick 252 (y.manufactures); DIVERGED, runaway at tick 777 | **FAIL** at tick 112 (y.power); DIVERGED, runaway at tick 559 | PASS, 6.3e-15 (a fixed point to rounding) |
| L3 | C2L | PASS, 8.1e-14 | PASS, 2.2e-14 | PASS, 1.1e-13 |
| G1 | C2m | PASS, 8.4e-15 | PASS, 3.3e-16 | PASS, 3.6e-15 |

The registered prediction (PREDICTION §4) had mode A PASS everywhere but I2 at 12 a year (the
mirror leaves 1e-9 at tick 2,558) and L2 and L3 at C2m (the mirror leaves at ticks 102 and 115 at
52 a year, 195 and 253 at 12, and 324 for L3 at 365). Where both fail, the engine leaves at
nearly the same tick: I2 at 12 a year at 2,427, L3 at 112 and 252, L2 at 12 a year at 251. The
engine passes three cells the mirror fails, all on the loop at C2m: L2 at 52 and 365 a year and
L3 at 365. There the engine's genesis rounds onto the point and stays there, and mode A cannot
see that the point is unstable (REPORT §5). The kick set can.

The base kick set, which MARKETS-SPEC §7.6 makes part of mode A, at 52 a year with H = L
(`D:/rustyecon-p2m/build/kicks/`):

| id | dials | H | kicks | verdict | largest gain_tail | largest gain_peak |
|---|---|---|---|---|---|---|
| I0 | C2m | 20,000 | 8 | PASS | 6.0e-6 | 3.3 |
| I1 | C2m | 144,000 | 14 | PASS | 5.3e-6 | 5.1 |
| I2 | C2m | 52,000 | 10 | PASS | 4.1e-6 | 5.9 |
| I3 | C2m | 484,000 | 16 | PASS | 6.4e-6 | 5.5 |
| G1 | C2m | 191,000 | 14 | PASS | 8.0e-6 | 4.2 |
| L2 | C2m | 20,000 | 10 | **FAIL**: every kicked run diverges | — | — |
| L2 | C2L | 20,000 | 10 | PASS | 7.3e-6 | 6.3 |
| L3 | C2m | 157,000 | — | **not made**: the base run diverges | — | — |
| L3 | C2L | 157,000 | 16 | PASS | 1.3e-5 | 16.3 |

A passing kick ends at its rounding floor: P2.0's I0 kicks read 1.9e-6 to 6.0e-6 and peaked below
6, and PREDICTION §6's mirror kicks read up to 2.4e-5 and 17.9. At L2 under C2m the point the
engine's genesis rounds onto is unstable: each 1e-9 kick grows until the loop dies, and each
kicked run stops between ticks 34,329 and 34,482, when the provider's due N·P_s overflows (a
kicked run has no runaway bound, and `SetState` refuses a non-finite value, G10). Certify counts
each as a failed kick. L3's base run under C2m stops the same way at tick 14,455, so there is no
end state to kick; its mode A fails at tick 112 anyway.

### 6.8 Determinism, conservation, platforms

- `markets_tapes_load_and_run_deterministically`: two runs of each tape give identical reports
  and hash streams for 500 ticks.
- `markets_conserve_every_tick`: at rest and from w×2 with every desk's coin ×0.1, each tape's
  ledger closes every tick and the money stock drifts by at most 1e-12 of itself over 2,000
  ticks.
- **The gate at P2.1.1** (`d0ceaa6`), and again at P2.1.2 (`455b813`) with the same counts.
  `scripts/gate.sh` is green in WSL
  (`CARGO_TARGET_DIR=/root/scratch/target-p2m-build`) and on Windows under Git Bash
  (`D:/rustyecon-targets/p2m-build`, a fresh target). On each machine 569 tests pass in the
  workspace (549 before, and the 20 of §6) with 2 ignored and run by name, and zero warnings;
  certify alone, Parquet-free, passes 65 with 1 ignored. The gate hash is `0x61f9c8529131ff17`,
  the stamp matches the clean checkout, both certificates recompute, the probe's pins hold
  (`probe_battery_csv_unchanged`), and telemetry is identical from two processes. The GUI's
  recorded check passes in WSL. Logs: `D:/rustyecon-p2m/build/gate/`.
- **The GUI's gate.** `scripts/gui.sh` is green in WSL: its 42 named tests pass, and the cli's
  hashes equal the GUI's for gate (2,080 ticks, final `0x61f9c8529131ff17`), appb (20,000, final
  `0xe1fa082b26995867`) and the two branch tapes.
- **Platforms (recorded).** The cli's per-tick hashes of the seven markets tapes over 2,000 ticks
  are byte-identical on WSL and Windows (finals: I0 `0x6c5bf916f3b69d35`, I1
  `0x8e3f1bc77c0e5a91`, I2 `0x958d5fb01a4847b5`, I3 `0x6749dd3193ba1712`, L2
  `0xf45a65d427f9f629`, L3 `0x010ae4da4eecfef5`, G1 `0xe71ac0e96df39297`), and appb's 20,000 end
  at `0xe1fa082b26995867` on both.

## 7. Where a result depends on a binding decision

- **60, one task line.** The category desks share the line's edges and the schedule γ, and
  every desk's threshold targets the same X, so all rest at one x\*. That is a consequence of 60,
  not a finding. With a schedule per category each desk would have its own target, and the
  oracle could not score it (unit-1b.md §2.3). The slack runs exist because of 60: a desk's
  threshold can move where the desk has no tasks.
- **61, cells deferred.** The technique is a continuous threshold per desk, so the rest point
  has no task-discretisation gap (MARKETS-SPEC §4.2 b), and the nesting on I0 is exact. With
  cells, the desk's recipe would jump, and 1b's open question 2 would come first.
- **67, machine recipes from services, labour and land.** The type desk buys no category goods,
  and L2's and L3's loop is a loop of machine services. With PLAN §3.1's build bundle of goods
  every machine type would buy from the category desks, every instance would have a
  goods-and-machines loop, and the type desk would need a list of bought goods.
- **70, one equilibrium.** Every instance and cost target has one interior equilibrium
  (MARKETS-SPEC §1.3), so the harness's target is well defined. At ρ = 0 that is proved.

## 8. Departures from MARKETS-SPEC, and what stays open

1. **The budget chain runs in admission's order, by good** (§3), not in list order. For I0 it is
   list order. The budgets it gives differ from a list-order chain only at rounding, since the
   wants sum to the outlay by construction.
2. **The type desk posts its hours and land orders even where a coefficient is 0** (I2's engine
   buys no land), at quantity 0, as the machine desk does; admission skips a zero quantity. The
   category desk posts land only where b_j > 0, as §2.4 says.
3. **G1** has 99 runs, of which six are not expressible (§5). It stays optional and last.
4. **Mode A at C2m on L2 passes because its genesis is a fixed point to the bit**, at 52 and 365
   a year, as PREDICTION §4 anticipated. The mirror shows the point unstable (§6.4's ∞), and the
   base kick set fails (§6.7). L3 at C2m at 365 a year passes the same way.
5. **A diverging run without the runaway bound ends in an error.** Under `Saturate` a one-sided
   price steps by e^k every tick, and after some thousands of ticks the provider's due N·P_s
   overflows; `SetState` refuses it (G10) and the run stops. The harness's runaway bound
   ([1e-6, 1e6] × genesis) classes such a run DIVERGED long before. A kicked run has no bound,
   so certify scores it as a failed kick (§6.7).
6. **The trace diff's mirror** is mm.py with one carry added, not the frame's mirror.py (§6.4).
7. **Not built here:** the run itself (P2.1.3), Tier 3 at 10·L, the kick sets at every target
   and tick length, the families, and scoring CONVERGED against the kick set. The harness and
   `markets kick` provide each.

## 9. Registration

The run's registration is `D:/rustyecon-p2m/run/registration.md` (MARKETS-SPEC §7.12's
`registration-markets.md`), written at P2.1.2 before any mode-B run, with its sha256 in
`registration.sha256` beside it and in the P2.1.2 commit's message. It names P2.1.1
(`d0ceaa6`, clean) and freezes the frame, the prediction, the rules, the harness, the tapes, this
file at P2.1.1, the dials, the tolerance, L per instance, dial set and tick length (§6.5), the
kick, the batteries with their slack runs, the verdict rules and the families, with §6's checks
made before it.
