# SPEC: the wall instance IW1, framed and registered (Phase 2 proper, P2.3)

Dated 2026-09-30. Work label `frame-wall`, for Phase 2 proper (STATE next step 7) on branch
`phase2-proper`. The worktree `D:/rustyecon-wt/p23` was read and not changed. It was at
`f7d1eae` when this work began. P2.3.0 (`f0c6667`, STATE.md only, so the same code) landed
during it with the rulings 360–393. This frame follows them. Its own proposals are numbered
FW1–FW9 and its open items OW1–OW3 (§8), for STATE.md to number in its range when the frame
lands.
Scratch: `D:/rustyecon-p23/frame-wall/`. This file frames the wall-regime instance and registers
the mirror's predictions for its engine run, before any engine code for it exists (R5).

**Evidence.**
- The oracle: the worktree's unit 1d (`WorkerEconomy::solve`), unchanged, called from a scratch
  crate (`oracle/`). Its doubles are in `registered/points.jsonl`.
- An independent 50-digit solve (`hp/solve_mp.py`, mpmath 1.3.0 in laborformal's venv). It is
  written from the model's equations and reads no oracle code.
- A Python mirror of the roles, `model/wm.py`. `model/make_wm.py` writes it from the markets
  probe's trace mirror `mm_carry.py` by 17 exact replacements. That file is copied from
  `D:/rustyecon-p2m/build/` with its recorded sha256 `dbd50ad9…538c`, and the original is
  untouched. A second map, `model/wm2.py`, is written from §3 and the engine's roles, not from
  either.
- No engine run was made. Every agent number below is the mirror's.

**Conditions of every number, unless its line says otherwise.**
- The economy is IW1 (§2), in the many-market roles with §3's additions.
- The dials are C2m at 52 ticks a year, with `Saturate` and planned assignment.
- L is 22,000 ticks, set by the elasticity probe's rule (§6.1). The tolerance is 1e-3 in log on
  every observable of §5.1.
- The classes are PROBE-SPEC §4.5's, as the markets predictor scored them
  (`model/p21_battery.py`, copied unedited, sha256 `a44bd25d…df06`).
- A run stops early only after it has stayed within 1e-6 in log of a locally stable target for
  2,000 ticks, from tick 4,000 on. Every target is locally stable (§6.1).
- Python 3.10.12 with numpy 2.2.6 in WSL, 46 processes.

## 0. The answer

**The instance.** IW1 is unit 1d's B economy (Baumol's economy) with E7's three worker types
(unit-1d.md §3.3):
- **The categories** are services, goods and space on one task segment, with Appendix B's
  machine.
- **Services** have a human-required tail L^H of 0.1 hours a unit (the paper's H).
- **The entrant** (130 a year) sells efficiency hours to the pool.
- **The trained** (52 a year) do only their reserved hours: 0.04 a unit of services.
- **The master** (26 a year) do only theirs: 0.03 a unit of goods.
- **The scalars:** η 0.5, T 520 a year, space 1 a basket, and χ_max 1, 2 and 2.
- **Its equilibrium is the solved wall.** x\* = 1: every contestable task is automated. The
  pool's wage is set by its own clearing at v = 0.807310, 2.567 times the top task's replacement
  value γ(1)·π = 0.314547 (depth ln(g/γ(1)) = 0.9426). Each reserved type's wage is set by its
  own market: 1.303538 and 2.325461.
- **The equilibrium is unique.** One sign change at every point. At ρ 0 uniqueness is also
  proved (unit-1d.md §5.4).
- **It is funded.** Rent covers the support 1.63 times (§2).

**The verdict in the mirror: GO.**

| id | what | mode A (largest gap) | Tier 1 | Tier 2 | Tier 3 | Tier 3S | kick sets | PL a tick: base, largest | families | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| **IW1** | the wall, three labour markets | PASS (3.3e-16) | 26/26 | 38/38 | 39/39 | 20/20 | 13/13 targets pass | 0.987132, 0.991911 | stocks 31/31, joint2 60/60, joint4 40/40, basin 516/516, history 79/79 windows; 12 and 365 a year, Hold and tilt 1 each 64/64 | **GO** |
| IC1 | IW1 with the entrant's χ_max 0.25: the task margin active (x\* 0.947), a reported control | PASS (2.2e-16) | 26/26 | 38/38 | 39/39 | 20/20 | not run | 0.989164, 0.991127 | — | control |

Ticks to tolerance at IW1, median (slowest):
- Tiers 1–3: 340 (452), 416 (575) and 561 (705), which is 6.5, 8.0 and 10.8 years. I0's are 320,
  440 and 523 (MARKETS §2).
- Tier 3S (decision 368): 404 (550).

**What the mirror found.**

1. **The wall is as stable as Appendix B, and a little faster than the same economy on the
   line.**
   - PL is 0.9832–0.9919 a tick over the 13 targets (0.42–0.66 a year). I0's is 0.98674.
   - At the wall the technique is a corner. Each desk's share returns to 0 at exactly
     1 − share(adjust) = 0.951229 a tick, decoupled from prices.
   - IC1, the same economy with the task margin active, has PL 0.9876–0.9911 and median ticks
     376, 442 and 575.
2. **Several labour markets add no slow mode.** The three labour markets are the fastest in the
   economy (τ 9.5–10.8 ticks). The slow markets are the goods and services markets (τ 106 and
   100 ticks), as in I0 and I1.
3. **The wall holds and is re-found.**
   - The rest point sits 0.9426 in log above the replacement value.
   - Only JB(0.5) breaches it in the registered battery: w × 0.5 and p_mach × 2 start 0.444
     below. The breach lasts 11 ticks, the desks' human share peaks at 0.101, and the run is
     in tolerance from tick 629.
   - In the families, 3 joint2, 16 joint4 and 86 basin runs breach it, for up to 49 ticks, and
     every one returns to the wall.
4. **A labour-demand shock at the wall moves wages, not output, yet costs half of output on the
   way.**
   - At the wall Y is fixed by land alone. Doubling the tail or the trained's reserved hours
     moves Y by 0 in log.
   - The paths bottom at 0.516 and 0.500 of Y (§6.4; OW2).
   - The machine's land × 2 moves Y by −0.208 in log and bottoms at 0.134 of the old Y. That is
     I0's 13.5% again.
5. **Families.**
   - Every run converges.
   - The deepest paths are in basin: w × 0.123 gives 18 ticks with no baskets, because the
     entrants stop working and the tail stops services.
   - Also in joint4: up to 13 ticks with no baskets.
   - The history family's step from the machine's land × 2 to × 0.5 makes the machine desk
     keep its whole stock for one tick (a·q ≥ held). No baskets follow the next tick. This is
     O21's one-type cousin, and it recurs in all 16 such windows (OW3). Every window is in
     tolerance at its end.
6. **Tick length.**
   - At 12 a year IW1 is stable (PL 0.963575 a tick, 0.641 a year), unlike I2. Tiers 1–2
     converge 64/64 in 8.8 and 11.1 years (median).
   - At 365 a year they converge 64/64 in 6.4 and 7.8 years.
   - Hold is identical to Saturate in Tiers 1–2: no market goes one-sided.
   - Tilt 1 is faster: 267 and 346 ticks.

**The roles' additions** (§3). There are three optional fields and no new kind, state or market
rule:
- the category desk's `tail`, the pool's hours at tasks closed to machines;
- the category desk's `reserved`, each reserved type's hours, bought on that type's own labour
  market;
- the provider's `more`, further transfers of N_i·P_s, paid in list order.

Each reserved type is a `BasketWorkers` actor on its own labour good, unchanged. The type desk is
unchanged, and machine recipes use pool labour (decision 140). With the three fields absent, the
mirror is the markets probe's map bit for bit (§4.1). The agents' rest point is the oracle's by
the argument of §3.7, and the mirror finds it at every target to 1.8e-15 (§4.2).

## 1. Which 1d economy, and why it is a wall

Decision 137 binds this instance, and 370 amends it to register the distance from each edge:
"its wall-regime instance is a solved wall, not a knife edge and not the edge of a reserved
shortage, where a type's supply is vertical and its wage is set through the pool's clearing,
which an agent market may not reach." Unit-1d.md §0.2 keeps three things apart: the wall, a
type at its wall, and the all-human corner. IW1 has the first two and not the third.

- **The wall** (SSRN §3.1; unit-1d.md §4.5). Every contestable task is automated (x\* = 1).
  Pool labour works only at the human-required tail and in machine building: 0.769 and 0.288
  hours a tick of n_D = 1.0577. The pool's wage is set by its clearing, ω = v/P_s =
  expm1(χ_E·n_D/N_E) = 0.526652, and not by the task margin. "The machine comparison does not
  pin the wage" (SSRN p.6).
- **Types at their walls** (SSRN §5, §8; unit-1d.md §4.3). The trained and the master do only
  reserved tasks (ε 0). Each wage is a scarcity price set by its own market: w_i/P_s =
  expm1(χ_i·D_i/N_i), 0.850368 and 1.517023. That is "a scarcity price on trained hands and
  heads" (main.tex:584). Unit-1d.md §4.3 walls a type with ε 0 by construction, and the oracle
  reports both `at_wall`, not pooled and not `edge`.
- **Not a knife edge.**
  - The line's excess demand at its end is f_line(1) = +0.5948, 56% of n_D. So the junction
    (x = 1 at v(1)) is not the equilibrium.
  - The wall's end is f_∞ = −1.4423, so a finite wage clears.
  - The wage is 2.725 times the junction's v(1) = 0.296296, and 2.567 times the replacement
    value at its own machine price, γ(1)·π(v\*) = 0.314547.
  - The agents' technique compares θ·w/p_τ with γ(1). So their margin is the depth
    ln(g/γ(1)) = 0.9426, with g = v/π = 1.283291.
- **Not the edge of a reserved shortage.** D_i/N_i is 0.308 (trained) and 0.462 (master) at
  the base, and at most 0.615 at any target. Every reserved supply has slope (elasticity 0.75
  and 0.65 at the point, §6.1). No type's wage needs the pool's clearing.
- **Every edge is registered** (decision 370): at genesis and at every cost target, 50 digits,
  §2.4.
- **Why B with E7's types, and not E3 as written.** E3 (N_E 8, N_T 2, η 0.3) is 1d's wall with
  a type at its wall. Under the probe's transfer, which pays one basket per head, it cannot be
  funded:
  - its support is 8 + 2·1.2 = 10.4 baskets, and P_s ≥ 1 from space alone, against T = 10;
  - it is O41's fault, the county's, again.

  IW1 keeps B's categories, machine and space and E7's pattern of three types. It scales the
  heads to 4 a tick (Appendix B's 208 a year, split 130/52/26), and sets η, the tail, the
  reserved hours and the reserved types' χ_max so that every cost target is a funded wall.
- **Why one pooled type and reserved-only types** (FW2; decisions 371, 377). A type with
  ε > 0 that is at its wall at the base can join the pool at another target. The agents would
  then need a rule that splits a type's hours between two markets. No such rule exists or has
  been mirrored (OW1). With ε 0 each type sells on exactly one market, so no agent chooses
  between markets.
  - The pool is one type trading on one market, so 371's call on how pooled types trade does
    not arise.
  - Every type is in 1d's dependence form, as 377 asks: its exit value is (e^χ − 1)·P_s, the
    probe's participation rule.
  - **Check with E's efficiencies** (trained 1.5, master 1.8). The oracle's base point is the
    same double for double (v = 0.8073099437840677 both ways). Both types stay at their walls,
    with premia 1.0764 and 1.6003, so IW1 is also 1d's "type at its wall" in the fuller reading
    at the base.
  - **Where that reading fails.** At tail.services 0.11 and 0.2 and res.services.trained 0.036
    and 0.02 the trained would join the pool in that reading. At tail.services 0.2 the master
    would too.

## 2. The instance

### 2.1 Parameters

Every number is per tick at 52 a year unless marked a year. The oracle takes N and T a year and
divides by `Clock::flow`, as `Instance::machine_params` does.

| item | value | basis |
|---|---|---|
| T, land services | 520 a year (10 a tick) | Appendix B's |
| space, land a basket | 1 | Appendix B's h |
| task line | one segment [0, 1]; γ(x) = 0.5·(0.2 + 0.8x), k 1 | Appendix B's shape at η 0.5 |
| services | z 1, b 0, μ 0.75, tail L^H 0.1, reserved: trained 0.04 | unit-1d.md §3.3 B, tail and reserved scaled |
| goods | z 1, b 0, μ 1, reserved: master 0.03 | B's H-free good, with E7's master |
| mach | θ 1, own 0.3, λ 0.05 (pool hours), b 0.4; flow (δ 1, J 1, ρ 0) | Appendix B's machine; decision 140 |
| entrant | 130 a year (2.5), χ_max 1, ε 1, ν 1; market `labour` | the pool |
| trained | 52 a year (1.0), χ_max 2, ε 0, ν 1; market `labour.trained` | reserved-only, E7's trained's tasks |
| master | 26 a year (0.5), χ_max 2, ε 0, ν 1; market `labour.master` | reserved-only, E7's master's tasks |
| support | ν_i 1: one basket a head, in work and out of it; the provider pays N_i·P_s to each pop | the probe's transfer (FW2) |

### 2.2 The oracle's point

The base point at 52 a year (r = 1) is `registered/points.jsonl`, tag `base`. The 50-digit values
are in `out/solve_mp.out`, and every double agrees with them within 2.8e-16 relative.

| value | oracle (f64) | 50-digit solve (first 30) |
|---|---|---|
| x\* | 1.0 (1 − x\* = 0.0 exactly) | 1 |
| v, the pool's wage | 0.8073099437840677 | 0.807309943784067885600234748824 |
| P_s | 1.5329104674395773 | 1.53291046743957724648653260484 |
| Y | 7.692307692307692 | 7.69230769230769213876572744503 |
| n_D, the pool's hours | 1.0576923076923077 | 1.05769230769230773923174237638 |
| D_trained, D_master | 0.30769230769230765, 0.23076923076923073 | 0.307692307692307691955…, 0.230769230769230755622… |
| w_trained, w_master | 1.3035382272281113, 2.325460702671447 | 1.30353822722811161479…, 2.32546070267144710700… |
| p_services, p_goods | 0.2744185761354895, 0.2584918913040878 | 0.274418576135489506064…, 0.258491891304087740422… |
| p_mach, X | 0.6290935674131478, 5.769230769230771 | 0.629093567413147731058…, 5.76923076923076933282… |
| depth ln(g/γ(1)), from the oracle's g = 1.2832907306678578 | 0.942574842758311 | 0.942574842758311344684151027961 |
| coverage T/(ΣN_i·P_s), from the oracle's P_s | 1.630884551382674 | 1.63088455138267397554783620167 |
| provider baskets | 2.523538205530696 | 2.52353820553069590219134480667 |

Other readouts at the base:
- Basket shares: services 0.1790, goods 0.1686, space 0.6524.
- Labour's share of income is 0.152.
- Each services unit costs 0.0521 in the trained's hours, and each goods unit 0.0698 in the
  master's.
- The residuals of unit-1d.md §6 are all at most 2.2e-16.
- The bisection took 54 steps, on the wall's first piece. There is no wall switch and no tie.

### 2.3 The cost targets

Three coefficients, each at ×1.1, ×0.9, ×2 and ×0.5 of its registered value, dated or at genesis
(FW4). Every target is a wall and funded, and every type's participation is interior.

| target | margin | depth | v | P_s | w_trained | w_master | Y | coverage | provider baskets | participation E / T / M |
|---|---|---|---|---|---|---|---|---|---|---|
| base | Wall | 0.9426 | 0.807310 | 1.532910 | 1.303538 | 2.325461 | 7.692308 | 1.6309 | 2.5235 | 0.4231 / 0.3077 / 0.4615 |
| land.mach=0.44 | Wall | 0.8460 | 0.798896 | 1.559880 | 1.286684 | 2.285469 | 7.518797 | 1.6027 | 2.4108 | 0.4135 / 0.3008 / 0.4511 |
| land.mach=0.36 | Wall | 1.0478 | 0.816318 | 1.506161 | 1.321600 | 2.368453 | 7.874016 | 1.6598 | 2.6394 | 0.4331 / 0.3150 / 0.4724 |
| land.mach=0.8 | Wall | **0.2164** | 0.742377 | 1.809678 | 1.173977 | 2.021411 | 6.250000 | 1.3815 | 1.5258 | 0.3438 / 0.2500 / 0.3750 |
| land.mach=0.2 | Wall | 1.6001 | 0.859751 | 1.401921 | 1.408953 | 2.578238 | 8.695652 | 1.7833 | 3.1331 | 0.4783 / 0.3478 / 0.5217 |
| tail.services=0.11 | Wall | 1.0342 | 0.893397 | 1.555476 | 1.322727 | 2.359693 | 7.692308 | 1.6072 | 2.4289 | 0.4538 / 0.3077 / 0.4615 |
| tail.services=0.09 | Wall | 0.8467 | 0.726831 | 1.512992 | 1.286600 | 2.295244 | 7.692308 | 1.6524 | 2.6094 | 0.3923 / 0.3077 / 0.4615 |
| tail.services=0.2 | Wall | 1.7637 | 2.105531 | 1.955582 | 1.662965 | 2.966663 | 7.692308 | **1.2784** | 1.1136 | **0.7308** / 0.3077 / 0.4615 |
| tail.services=0.05 | Wall | 0.3986 | 0.449548 | 1.455049 | 1.237327 | 2.207343 | 7.692308 | 1.7182 | 2.8726 | 0.2692 / 0.3077 / 0.4615 |
| res.services.trained=0.044 | Wall | 0.9518 | 0.815551 | 1.548558 | 1.498715 | 2.349198 | 7.692308 | 1.6144 | 2.4576 | 0.4231 / 0.3385 / 0.4615 |
| res.services.trained=0.036 | Wall | 0.9347 | 0.800348 | 1.519691 | 1.124468 | 2.305406 | 7.692308 | 1.6451 | 2.5803 | 0.4231 / 0.2769 / 0.4615 |
| res.services.trained=0.08 | Wall | 1.1304 | 0.994887 | 1.889080 | 4.578869 | 2.865778 | 7.692308 | 1.3234 | 1.2936 | 0.4231 / **0.6154** / 0.4615 |
| res.services.trained=0.02 | Wall | 0.9143 | 0.782571 | 1.485937 | 0.535357 | 2.254200 | 7.692308 | 1.6824 | 2.7298 | 0.4231 / 0.1538 / 0.4615 |

Why these three coefficients:
- The machine's land moves the wall's depth: land ×2 leaves 0.216.
- The tail moves the pool's demand at the wall.
- The trained's reserved hours move one reserved market.

At 12 and 365 a year every target has the same prices and the same regime, and the quantities
scale by 52/tpy.

### 2.4 The distance from each edge (decision 370)

Every gap is in log, at 50 digits (`hp/edges_mp.py` → `out/edges_mp.out`,
`registered/edges.json`), and taken at genesis (the base point) and at every cost target.
- **The pool above the junction.** ln(v/v(1)), where v(1) = γ(1)·b/(1 − a − γ(1)·λ) is the
  wage at which the wall meets the task line: the knife edge. Also the agents' own margin, the
  depth ln(g/γ(1)).
- **The pool below its ceiling.**
  - ln(ω_max/ω), where ω_max = (1 − C)/L is the real wage the basket's embodied labour allows
    (unit-1d.md §4.5; C the walled types' share, L the pool's hours a basket).
  - ln(S(ω_max)/n_D). The pool's supply at the ceiling over its demand: 1d turns `LaborShort`
    where this reaches 0. Here S(ω_max) is every entrant at work, so it is also the room before
    the pool's supply is vertical.
- **Each reserved type short of its shortage's edge.** ln(N_i/D_i). At ε 0 its wage is above
  ε_i·v by construction. The last column is the side reading with E's efficiencies,
  ln(v_i/(ε_i·v)): below 0 that type would join the pool.

| target | junction ln(v/v(1)) | depth ln(g/γ(1)) | ceiling ln(ω_max/ω) | LaborShort ln(S(ω_max)/n_D) | trained ln(N/D) | master ln(N/D) | with E's ε: trained, master |
|---|---|---|---|---|---|---|---|
| base (genesis) | 1.0023 | 0.94257 | 2.5425 | 0.8602 | 1.1787 | 0.77319 | 0.0737, 0.4702 |
| land.mach=0.44 | 0.89656 | 0.84603 | 2.5732 | 0.88302 | 1.2015 | 0.796 | 0.0711, 0.4633 |
| land.mach=0.36 | 1.1188 | 1.0478 | 2.5108 | 0.83685 | 1.1553 | 0.74984 | 0.0763, 0.4774 |
| land.mach=0.8 | **0.22535** | **0.21636** | 2.8139 | 1.0678 | 1.3863 | 0.98083 | 0.0528, 0.4139 |
| land.mach=0.2 | 1.7584 | 1.6001 | 2.3729 | 0.7376 | 1.0561 | **0.65059** | 0.0885, 0.5104 |
| tail.services=0.11 | 1.1037 | 1.0342 | 2.3856 | 0.79 | 1.1787 | 0.77319 | −0.0130, 0.3835 |
| tail.services=0.09 | 0.89733 | 0.84674 | 2.7099 | 0.93571 | 1.1787 | 0.77319 | 0.1656, 0.5621 |
| tail.services=0.2 | 1.961 | 1.7637 | **1.2808** | **0.31366** | 1.1787 | 0.77319 | −0.6414, −0.2449 |
| tail.services=0.05 | 0.41688 | 0.39858 | 3.5278 | 1.3122 | 1.1787 | 0.77319 | 0.6070, 1.0035 |
| res.services.trained=0.044 | 1.0125 | 0.9518 | 2.5331 | 0.8602 | 1.0833 | 0.77319 | 0.2030, 0.4702 |
| res.services.trained=0.036 | 0.99369 | 0.9347 | 2.5505 | 0.8602 | 1.284 | 0.77319 | −0.0654, 0.4702 |
| res.services.trained=0.08 | 1.2113 | 1.1304 | 2.3517 | 0.8602 | **0.48551** | 0.77319 | 1.1211, 0.4702 |
| res.services.trained=0.02 | 0.97122 | 0.91426 | 2.5712 | 0.8602 | 1.8718 | 0.77319 | −0.7851, 0.4702 |

The least gaps over the targets:
- the junction 0.225 and the depth 0.216, both at land.mach 0.8;
- the ceiling 1.281 and `LaborShort` 0.314, both at tail.services 0.2, where 73% of the entrants
  work;
- the trained's shortage edge 0.486, at res.services.trained 0.08;
- the master's 0.651, at land.mach 0.2.

So no target is within 0.2 in log of any edge. The ceiling is never near: the wage is at worst
28% of it (tail 0.2), so O30's ill-conditioning near the ceiling does not arise.

### 2.5 Funding (O41, O43)

Coverage is 1.278–1.783 over the targets (FUNDED's "margin" asks for 1.2). At genesis, under
every price displacement of the registered battery:
- the rent T·r still covers the transfer ΣN_i·P_s at least 1.210 times (worst: r × 0.5);
- no displacement puts every head of a type to work: the largest ln(1 + w_i/P_s)/χ_i is 0.719
  (entrant, w × 2), 0.497 (trained) and 0.697 (master).

### 2.6 Uniqueness and the independent solve

- **The oracle's count.** The oracle returns `Interior` with margin `Wall` at all 39 points (13
  targets at 12, 52 and 365 a year). So its count along the whole path found one sign change.
  - f is positive on the whole line, on a grid of 1,000 thresholds at every target. Its least
    value is +0.1437, at the land.mach 0.8 target.
  - It is negative at the wall's end: f_∞ ≤ −0.673.
- **The 50-digit solve** (`hp/solve_mp.py`) solves the wall in closed form. It reads every
  input as the double the oracle reads, and scans the path's sign sequence on 1,001 thresholds
  plus the wall's end.
  - It finds exactly one sign change at every point.
  - It agrees with the oracle's doubles within 1.13e-15 relative at every point, on v, P_s, Y,
    n_D, p_mach, X, both prices, both reserved wages and reserved demands, the entrant's hours
    and the provider's baskets.
- **At ρ 0 the equilibrium is proved unique** (unit-1d.md §5.4): f is nonincreasing along the
  whole path.

### 2.7 The line control IC1

IC1 is IW1 with the entrant's χ_max at 0.25 in place of 1. Cheaper work puts more entrants to
work, lowers the pool's wage below the top task's replacement value, and brings the task margin
back.
- At the base it is contestable at x\* = 0.947383, v 0.283, coverage 1.715, participation 0.711,
  0.314 and 0.471.
- Two of its targets are walls: land.mach 0.2 and tail.services 0.2.
- At land.mach 0.8 the entrants are all at work (participation 1).

It is a reported control, not a verdict instance (FW8). It asks what the wall itself
changes, like for like (§6.4). Its L by the elasticity probe is 25,000.

## 3. The roles, as the engine should build them

### 3.1 What changes, and what does not

- **Core, markets and engine: nothing.**
- **Agents:** three optional spec fields (below), no new kind, no new `ActorState`.
  - Each field is `#[serde(default, skip_serializing_if = …)]` on the raw and the resolved spec,
    as the type desk's `plant` is (P2.2b).
  - So every committed tape keeps its canonical text, `tape_hash`, `world_id` and hash stream
    (R1). The pins must not move: gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
    demo-gb `0xfad880fe08d06645`, and the probe, markets, horses and loops pins.
- **The probe** gains the wall's instance and harness readouts (§5) in `probe::markets`, and one
  tape, `tapes/markets-iw1.ron` (with `--inst ic1` for the control).

### 3.2 The category desk: the tail and reserved hours

Two new raw fields on `RawCategoryDesk`:
- `tail: Option<Key>`. A param key, unit `Dimensionless`, live: L^H_j, efficiency hours per unit
  at tasks closed to machines that any pooled worker can do (the paper's H). They are bought on
  the desk's `labour` market. Absent means none.
- `reserved: Vec<RawInput>`. Each entry is `good`, a reserved type's hours (an `Instant` good,
  distinct from `labour`, `service`, `land` and `output`, each good once), and `coef`, a param
  key, `Dimensionless`, live: R_ji, its hours per unit. The list is in worker-type order and is
  evaluated in list order. Empty means none.

The rules (MARKETS-RULES §3's, with these changes; unit-1d.md §4.1–§4.2):
- **Recipe.** (H, M) = the line's tasks at the planned s, as now. The pool's hours per unit are
  **h = H + L^H**: one addition, and none when `tail` is absent, so the old path is untouched.
  The reserved hours R_ji do not depend on x.
- **Technique.** Unchanged: s closes share(adjust) of its gap to 1 − measure(θ·w/p_τ). At the
  wall θ·w/p_τ ≥ γ(1), the measure is 1, the target is 0 and s → s·(1 − share(adjust)).
- **Unit cost.** c = ((w·h) + (p_τ·M/θ)) + (r·b). Then, for each reserved input in list order,
  c = c + p_i·R_i. The markup is p/c, and the scale is the cash rule, both unchanged.
- **Orders.**
  - Hours whenever L̄_j > 0 or L^H_j > 0, at quantity h·q.
  - Services and land as now.
  - Each reserved input whose coefficient is not zero, at quantity R_i·q.
  - Every budget is p·(coef·q), cut from the one outlay by `budget_chain` in admission's (good)
    order.
- **Produce.** Leontief at the planned x over every input whose coefficient is not zero:
  (labour, h), (service, M/θ), (land, b), and each (reserved good, R_i). It burns coef·y of
  each and mints y.
- **State.** The good desk's, unchanged.

### 3.3 The provider: several transfers

One new raw field on `RawBasketProvider`: `more: Vec<RawTransfer>`, further transfers after
`transfer`, each `to` a distinct actor (not the provider and not `transfer`'s) with its own
`heads` param. Empty means none.
- **Decide.** Pay `transfer` as now: paid_0 = min(N_0·P_s, coin held), taken from the copy.
  Then, for each entry of `more` in list order, paid_i = min(N_i·P_s, the coin the copy still
  holds), taken from the copy. The basket budget is share(spend) of the coin left.
- **State.** `ProviderState { due, paid }` holds the sums, due = ((N_0·P_s) + N_1·P_s) + …, so
  certify's `Obs::transfers` reads the whole shortfall. With `more` empty these are the old
  numbers bit for bit.

### 3.4 The workers and the type desk

- **Workers.** Each worker type is a `BasketWorkers` actor as it stands, with its own `labour`
  good (`labour`, `labour.trained`, `labour.master`) and its own `heads` and `chi_max` params.
  Its participation is min(ln1p(w_i/P_s)/χ_i, 1) of N_i (SSRN eq 10 with ν_i 1). It reads only
  its own labour price, the basket's prices and its params (R13).
- **The type desk** is unchanged. Machine recipes use pool labour (decision 140): the desk buys
  `labour`.

### 3.5 The tape

- **One node, `home`, quoting in `coin`.** The goods are those of P2.1's markets tapes, plus
  `labour.trained` and `labour.master` (`Instant`), appended so that every P2.1 id keeps its
  place.
- **Actors and classes** (one class an actor, R12):

  | actor | kind | class |
  |---|---|---|
  | `desk.services` | CategoryDesk | `services_desks` |
  | `desk.goods` | CategoryDesk | `goods_desks` |
  | `desk.mach` | TypeDesk | `mach_desks` |
  | `provider` | BasketProvider (`more`: trained, master) | `owners` |
  | `workers` | BasketWorkers | `workers` |
  | `workers.trained` | BasketWorkers | `trained_workers` |
  | `workers.master` | BasketWorkers | `master_workers` |

- **Params** (units and bases as MARKETS-RULES §4; R4):
  - `inst.workers` 130, `inst.trained.workers` 52 and `inst.master.workers` 26, all
    `FlowPerYear`;
  - `inst.chi_max` 1, `inst.trained.chi_max` 2 and `inst.master.chi_max` 2;
  - `inst.land` 520 `FlowPerYear`; `inst.eta` 0.5, `inst.g0` 0.2, `inst.g1` 0.8, `inst.k` 1;
  - services: `inst.services.weight` 1, `inst.services.land` 0, `inst.services.mu.0` 0.75,
    **`inst.services.tail` 0.1** and **`inst.services.reserved.trained` 0.04**;
  - goods: `inst.goods.weight` 1, `inst.goods.land` 0, `inst.goods.mu.0` 1 and
    **`inst.goods.reserved.master` 0.03**;
  - `inst.space.weight` 1;
  - the machine: `inst.mach.theta` 1, `inst.mach.own` 0.3, `inst.mach.labour` 0.05 and
    `inst.mach.land` 0.4.
- **The dials: C2m, copied per role** (decision 119):
  - `rate.labour`, `rate.labour.trained` and `rate.labour.master` 5.2 a year;
  - `rate.services` and `rate.goods` 2.6; `rate.land` and `rate.mach` 1.3;
  - `adjust.technique.<j>` 2.6; `buffer.desk.<d>.cash` 5.2; `tilt.desk.<d>` 0;
  - `spend.workers` 13, read by all three worker pops, and `spend.provider` 13.
- **The cost-shock names:** `land.mach` is `inst.mach.land`, `tail.services` is
  `inst.services.tail`, and `res.services.trained` is `inst.services.reserved.trained`.

### 3.6 Genesis (MARKETS-RULES §4, extended)

The generator solves unit 1d in the harness, with `WorkerEconomy::solve`, on
`Instance::worker_params(tpy)`:
- the categories in order, then space (weight h, direct land 1, no tasks);
- the machine's flow recipe as its operating recipe, a zero build recipe, δ 1, J 1, ρ 0;
- worker types in pop order: (N_E/tpy, χ_E, 1, 1), (N_T/tpy, χ_T, 0, 1), (N_M/tpy, χ_M, 0, 1);
- `human_required` = [tail, 0, 0], and `reserved` rows [[0, R_T, 0], [0, 0, R_M], [0, 0, 0]].

`oracle/src/main.rs::params` is exactly this construction. P2.1's instances keep 1c's
`MachineEconomy`, so their genesis and pins do not move. Genesis, relative to r = 1:
- **Prices:** `labour` v; `labour.<i>` the type's wage v_i; each category p_j; `mach` p_m.
- **Shares:** each desk's human share is `one_minus_x_star`, exactly 0.0 at the wall.
- **Stocks:** z_j·Y for a category desk, X for the type desk.
- **Coins:** a category desk p_j·y_j/share(turnover); the type desk ((0 + λ·v) + b·1)·X/share.
- **Each pop:** (N_i·P_s + v_i·hours_i)/share(spend.workers), hours_i from `Eq1d::workers`.
- **The provider:** τ + (T − τ)/share(spend.provider), with
  τ = ((N_E·P_s) + N_T·P_s) + N_M·P_s.

### 3.7 Why the rest point is the oracle's

Take a live rest point: every market trades and clears, every actor's coin and state are
constant, and nothing spoils. The rules then give unit 1d's equations at the wall, one for one.

1. **A category desk's coin is constant only where p_j = c_j** = w·h_j + p_τ·M_j/θ + r·b_j +
   Σ_i w_i·R_ji. That is unit-1d.md §4.2's category row, with π = p_τ/θ and each reserved hour
   priced at its type's wage (SSRN eq 2).
2. **The threshold is constant only where s = 1 − measure(θ·w/p_τ).** With s = 0 this is
   θ·w/p_τ ≥ γ(1), 1d's wall condition v ≥ γ(1)·π. At IW1 the inequality is strict (depth
   0.9426), so s = 0 is the unique rest of the technique.
3. **The type desk's coin is constant only where p_m(1 − a) = λ·w + b·r.** That is the machine
   row priced at the wall's own wage, p_m = (λv + b)/(1 − a) (unit-1d.md §4.5).
4. **The pool's market clears:** N_E·F(ln(1 + w/P_s)) = Σ_j h_j·y_j + λ·X, SSRN eq 10 with the
   tail (unit-1d.md §4.4).
5. **Each reserved market clears:** N_i·F_i(ln(1 + w_i/P_s)) = Σ_j R_ji·y_j = D_i. With uniform
   F_i this gives w_i/P_s = expm1(χ_i·D_i/N_i) = ζ_i, 1d's walled wage with ν_i 1 (§4.3).
6. **The other markets.** Land clears at T = h·Y + b·X. Each category clears at y_j = z_j·Y. The
   machine clears at (1 − a)·X = Σ_j M_j·y_j/θ. The provider pays N_i·P_s to each pop (support
   ν_i = 1). The fixed basket is decision 59's, so every household's baskets are z-proportional
   and Y = T/B (eq 11).

These are unit 1d's equations at x = 1 in r = 1 units. At ρ 0 they have exactly one solution
(§2.6). So a live rest point of the agents is the oracle's point, and no rule has a band. The
converse is checked: the oracle's f64 point with the stationary coins maps to itself (§4.2).

### 3.8 Tests the build adds, each failing without its change

- `category_desk_buys_its_tail_and_reserved_hours`. On a fixed state with a tail and a reserved
  input, the desk orders (H + L^H)·q hours and R·q reserved hours. Its cost includes w_i·R_i,
  and produce burns both. It fails with either addition removed.
- `provider_pays_every_transfer_in_list_order`. The provider pays every transfer in list order,
  including a short one, and the state holds the sums.
- `wall_roles_nest_the_many_roles`. With `tail`, `reserved` and `more` absent, decide and
  produce equal P2.1's on I0–I3 from random states; this rule-level form holds already. The
  run-level form writes I1's tape with `tail` at 0, a `reserved` input at coefficient 0 and a
  `more` transfer to a pop of 0 heads. Its original actors' prices, volumes and coins must
  equal the field-free tape's, bit for bit, for 2,000 ticks. It fails if an addition perturbs
  the old path.
- `markets_iw1_genesis_is_unit_1d`. The generator's genesis prices and coins equal §2.2's
  doubles.
- `markets_iw1_rest_point_is_the_oracles`, as `markets_rest_point_is_the_oracles`. At the base
  and the 12 targets, at 12, 52 and 365 a year, three ticks leave every observable within 1e-12
  of the oracle in log.
- `markets_iw1_conserves_and_is_deterministic`.
- The load checks: a reserved good repeated or equal to `labour`; a transfer to the provider or
  a repeated recipient; a `tail` param of the wrong unit. Each is refused with `LoadError`.

## 4. Checks on the mirror

### 4.1 The new layer off is the old map, and a second map agrees

- **`model/wm_nest.py` → `wm_nest.out`.** On every markets-probe instance, which has no tail, no
  reserved hours and no reserved type, `wm.tick` is `mm_carry.tick` bit for bit. That means
  every price, share, coin and stock equal as doubles, every tick.
  - The instances are I0–I3, L2 and L3 at C2m and C2L, and G1.
  - The starts are hold, w × 2, JB(0.5), every type price × 0.5, every desk coin × 0.1, and the
    land coefficient × 2 at genesis.
  - Each ran 3,000 ticks, and I1 also ran at 12 and 365 a year: 66 runs.
- **`model/wm2_check.py` → `wm2_check.out`.** `wm2.py` is a second map of IW1, written from §3
  and the engine's roles, not from `mm_carry.py` or `wm.py`. It agrees with `wm.py` within
  1.04e-14 in log over 2,000 ticks on every price, share, coin and stock, from 13 starts:
  - hold, w × 2, JB(0.5), x\*/2 and s[goods]=0.5;
  - RW(2), trained's wage × 0.5, the master's coin × 0.1 and the machine stock × 0.01;
  - joint(4,10), and the three coefficients' ×2 or ×0.5 at genesis.

### 4.2 The rest point is the oracle's (`model/wm_rest.py` → `wm_rest.out`)

Genesis from the oracle's doubles, at the base and all 12 targets, at 12, 52 and 365 a year:
- after 3 ticks, every observable is within 5.6e-16 in log;
- over 2,000 ticks it stays within 1.8e-15, and no state value drifts more than 1.5e-15
  relative;
- every fill is at least 1 − 2.2e-15, and money is conserved within 1.0e-15.

Mode A at L (`modea.out`) passes everywhere, with largest gaps of 1.1e-16 to 1.6e-15.

## 5. The harness for the wall

### 5.1 Observables and targets

The observables are P2.1's (MARKETS-RULES §5), with these changes:
- **Reserved wages.** Each reserved wage over r, `w.trained` and `w.master`, is added.
- **Thresholds, not shares.** The technique is read as each desk's threshold x_j = 1 − s_j, in
  log (FW5), not as s_j. s_j's target 0 has no log. At an interior point x_j's gap
  measures the same displacement as s_j's.
- **Volumes.** Every market's cleared volume, with the reserved labour markets' targets D_i.
  The pool's is n_D.
- **Outputs.** Each desk's output.
- **The count.** 16 observables: v, two reserved wages, three prices, two thresholds, seven
  volumes, three outputs. Every tolerance is the 1e-3 floor.

P2.1's instances keep s_j, so their pins do not move.

### 5.2 The run grammar at the wall (FW6)

| term | at the wall |
|---|---|
| `s[D]*F` | not in the battery: it is a no-op at s\* = 0 |
| `s[D]=V` | new: desk D's human share set to V at genesis (x = 1 − V); Tiers 1–3 take V 0.05, 0.2, 0.5 |
| `x*/2` | kept: every desk at x = 1/2 |
| `JA(F)`, `JB(F)` | every labour market's price scaled as `w` is; their s-parts are no-ops at s\* = 0 |
| `RW(F)` | new, where there are several labour markets (as RC and RT are): the pool's wage × F, each reserved wage × 1/F |
| `RC(F)` | services × F, goods × 1/F |
| basin | w, r, the largest-share good (services), each type, and each reserved wage, in place of the technique |
| `coin.workers.<type>*F` | each reserved pop's coin, in the stocks family |

The registered battery (`model/wb.py::run_list`) has 103 runs:
- 7 markets × 6 factors;
- 2 desks × 3 shares;
- JA, JB, N, RC and RW × 6;
- x\*/2;
- 3 coefficients × 4 values × (genesis, dated).

The tiers are 26, 38 and 39 runs. No run is slack.

**Tier 3S is in the verdict** (decisions 229 and 368; FW9). It has 20 runs, each stock and each
coin at × 0.5 and × 2:
- the three desks' stocks;
- the three desks' coins;
- the three pops' coins and the provider's.

A 3S run's start distance is the largest D̂ of its first year, as HORSES-RULES §5 reads it.

**The stocks family** (O22, reported) has 31 runs: each desk's coin × 0.02, 0.1, 0.5, 2; each
desk's stock × 0.1, 0.5, 2; the machine's stock × 0.01, 10; each pop's coin × 0.1, 2; the
provider's × 0.5, 2. Its start distance is P2.1's, |ln F|/1e-3.

### 5.3 The wall's own readouts (FW7)

These are reported every run and not scored, beside MARKETS-RULES §5's transient statistics:
- **depth**, ln(θ·w/(p_τ·γ(1))) at posted prices each tick: its lowest; the ticks it is below 0
  (breach ticks), the first and the longest run. Below 0 the machine comparison pins the wage
  again and the desks' target share turns positive;
- **each desk's largest human share** over the run;
- **each pop's participation range**, and its saturated ticks, where all its heads work and its
  supply is capped at N_i by the support of χ, not by a clamp;
- **dead ticks by market**, the reserved labour markets included;
- **the transfer shortfall** over every pop.

## 6. The mirror's battery

### 6.1 L, the local map and the kicks

**The elasticity probe** (`wel.out`; τ in ticks at 52 a year):

| market | ε_S | ε_D | τ |
|---|---|---|---|
| goods | 0 | −0.188 | 106.4 |
| services | 0 | −0.200 | 100.2 |
| mach | 0 | −0.638 | 62.7 |
| land | 0 | −0.769 | 52.0 |
| labour.master | +0.653 | −0.270 | 10.8 |
| labour.trained | +0.747 | −0.190 | 10.7 |
| labour | +0.815 | −0.239 | 9.5 |

- L is 22,000 at 52 a year, 20,000 at 12 and 164,000 at 365.
- Over the targets at 52 a year the rule gives 20,000–29,000. The largest is at land.mach 0.2,
  where goods' τ is 140.2.
- The registered L is the base's, 22,000, as P2.1 set L per instance from the base.

**The largest root per tick** (`local.out`). PL is the growth of a 1e-8 displacement of the
state, over ticks 2,000–12,000, from 4 random directions. The technique's shares are held at
their corner: s moves by exactly 0.951229 a tick while the wall binds.

| target | PL a tick | a year |
|---|---|---|
| base | 0.987132 | 0.5099 |
| land.mach 0.44 / 0.36 / 0.8 / 0.2 | 0.986745 / 0.987502 / 0.983603 / 0.988952 | 0.500 / 0.520 / 0.423 / 0.561 |
| tail.services 0.11 / 0.09 / 0.2 / 0.05 | 0.987292 / 0.986853 / 0.991206 / 0.986240 | 0.514 / 0.503 / 0.632 / 0.487 |
| res.services.trained 0.044 / 0.036 / 0.08 / 0.02 | 0.987333 / 0.986945 / **0.991911** / 0.986522 | 0.515 / 0.505 / 0.656 / 0.494 |
| base at 12 a year | 0.963575 (technique 0.805198) | 0.641 |
| base at 365 a year | 0.998149 (technique 0.992902) | 0.509 |
| IC1 base, and its largest target (res 0.08) | 0.989164, 0.991127 | 0.568, 0.629 |

**The kick sets** (each price × (1 ± 1e-9), H = 22,000; `local.out`). All 13 targets pass:
14 kicks each, gain_tail at most 1.47e-5, gain_peak at most 2.77. The envelope's decay between
1e-1 and 1e-3 is 0.988002 a tick at the base (0.534 a year). At the targets it is 0.984292
(land.mach 0.8, 0.439 a year) to 0.992066 (res.services.trained 0.08, 0.661 a year).

### 6.2 The registered battery (`registered/runs_battery.tsv`; `summary.out`)

Every run is CONVERGED: Tier 1 26/26, Tier 2 38/38, Tier 3 39/39.

**The early stop.** The mirror stops a run once it has stayed within 1e-6 in log of its target
for 2,000 ticks, from tick 4,000 on. Ten of the slowest and deepest were run to L = 22,000
without stopping (`model/fullL.py` → `fullL.out`): JA(0.5), JA(0.8), JA(0.95), x\*/2, JB(0.5),
s[goods]=0.5, RW(2) and three cost shocks. Each keeps its class and its ticks to tolerance, and
ends within 5.3e-15 to 1.6e-14 in log of the oracle.

| | Tier 1 | Tier 2 | Tier 3 |
|---|---|---|---|
| ticks to tol, median (slowest) | 340 (452, JA(0.95)) | 416 (575, JA(0.8)) | 561 (705, JA(0.5)) |
| years, median (slowest) | 6.55 (8.69) | 7.99 (11.06) | 10.79 (13.56) |
| peak D̂, median / worst | 56 / 370 | 215 / 1,113 | 853 / 2,401 finite; x\*/2 has an observable at 0 (∞) |
| dead ticks, worst (run) | 0 | 7 (s[goods]=0.2) | 117 (JA(0.5)) |
| dead ticks by market, worst over the tier | none | goods 7, mach 4, services 2 | services 115, goods 115, mach 84, labour.master 56, labour.trained 47, land 30, labour 0 |
| lowest baskets over Y\* (run) | 0.694 (s[goods]=0.05) | 0.330 (s[goods]=0.2) | 0.000 (x\*/2, one tick) |
| worst buyer fill | 0.676 | 0.314 | 0.000 |
| worst transfer shortfall | 0 | 22.25 | 706.14 |
| lowest depth (run) | +0.840 (JB(0.95)) | +0.496 (JB(0.8)) | −0.444 (JB(0.5)) |
| runs with a breach; most breach ticks | 0 | 0 | 1 (JB(0.5)): 11, ticks 0–10 |
| highest participation E / T / M | 0.457 / 0.320 / 0.483 | 0.517 / 0.362 / 0.532 | 0.748 / 0.618 / 0.697 |
| saturated ticks | 0 | 0 | 0 |

**Tier 3S** (`registered/runs_tier3s.tsv`): 20/20 CONVERGED.
- Ticks to tolerance: median 404, slowest 550 (coin.desk.mach × 2); 7.8 and 10.6 years.
- Peak D̂: median 506, worst 1,253 (stock.mach × 0.5).
- Dead ticks: at most 8 (coin.desk.services × 2, all in goods).
- Lowest baskets 0.286 of Y\* (stock.mach × 0.5), and no tick without baskets.
- Lowest depth +0.643 (coin.desk.mach × 0.5), with no breach and no saturated pop.
- Every start distance is at least 51.
- IC1's Tier 3S is also 20/20: median 478, slowest 602.

**The technique on the wall** is exact. While the wall binds, each displaced share falls as
s_t = s_(t−1)·(1 − 0.048771) exactly, whatever the prices.
- The thresholds are back within tolerance at tick 78, 105 and 124 of s[D]=0.05, 0.2 and 0.5,
  and at 124 in x\*/2.
- In JB(0.5), after the 11 breach ticks, they are back at 102.
- They are never the slowest observable: prices are.
- A share decays until it stalls at 5e-323, ten subnormal ulps, where a·s rounds to 0. The
  threshold x then reads 1.0 exactly.

**The cost shocks** (at genesis; dated runs within one tick of these):

| shock | ticks to tol | dead | lowest baskets over the new Y\* | lowest depth |
|---|---|---|---|---|
| land.mach 0.44 / 0.36 | 470 / 343 | 0 / 0 | 0.815 / 0.935 | 0.778 / 0.943 |
| land.mach 0.8 / 0.2 | 565 / 557 | 93 / 0 | 0.165 / 0.569 | 0.128 / 0.943 |
| tail.services 0.11 / 0.09 | 383 / 285 | 0 / 0 | 0.918 / 0.985 | 0.943 / 0.847 |
| tail.services 0.2 / 0.05 | 683 / 384 | 0 / 0 | 0.516 / 0.925 | 0.943 / 0.399 |
| res.services.trained 0.044 / 0.036 | 369 / 319 | 0 / 0 | 0.909 / 0.957 | 0.920 / 0.935 |
| res.services.trained 0.08 / 0.02 | 576 / 423 | 0 / 0 | 0.500 / 0.813 | 0.730 / 0.914 |

### 6.3 The families (O22, stocks first; `summary.out`)

| family | runs converged | ticks to tol, median (slowest) | worst dead ticks | lowest baskets over Y\*; most ticks with none | breaches: runs, most ticks | saturated ticks, most (E / T / M) |
|---|---|---|---|---|---|---|
| stocks | 31/31 | 457 (626, desk.services coin × 0.02) | 77 (desk.services coin × 0.02) | 0.000 (mach stock × 0.01); 2 | 0 | 0 |
| joint2 | 60/60 | 572 (729) | 140 | 0.033; 0 | 3, 10 | 1 / 0 / 0 |
| joint4 | 40/40 | 666 (834) | 203 | 0.000; 13 (joint(4,10)) | 16, 49 | 13 / 4 / 6 |
| basin (6 variables × 86) | 516/516 | 550 (1,025, p_mach × 0.123) | 250 | 0.000; 18 (w × 0.123) | 86, 48 | 158 / 1 / 8 |
| history, land.mach cycled 80 times, 1,500 ticks apart | 79/79 windows in tolerance at their end; the last window, 22,000 ticks at the base, in tolerance from tick 686 to its end | slowest by window kind: × 1.1 470, × 0.9 413, × 2 578, × 0.5 657, × 1 686 | 109 (× 2 windows) | 0.000 in each × 0.5 window (16 of them), one tick | 0 | — |
| 12 a year, Tiers 1–2 | 64/64 | 106 (149) and 134 (187) ticks: 8.8 (12.4) and 11.1 (15.6) years | 1 | 0.402; 0 | 0 | 0 |
| 365 a year, Tiers 1–2 | 64/64 | 2,319 (3,095) and 2,852 (3,943) ticks: 6.4 (8.5) and 7.8 (10.8) years | 56 | 0.281; 0 | 0 | 0 |
| Hold, Tiers 1–2 | 64/64 | identical to Saturate in every statistic: no market goes one-sided | | | | |
| tilt 1, Tiers 1–2 | 64/64 | 267 (392) and 346 (488) | 9 | 0.307; 0 | 0 | 0 |

**Why a basket-less tick.** Planned assignment turns a missing input into zero output (O24).
- The machine's stock × 0.01 leaves the category desks without services for a tick.
- w × 0.123 stops the entrants working, and the tail stops services.
- In the history family, the step from land.mach 0.8 to 0.2 cuts the machine desk's cost 3.5
  times at the prices in force. Its cash rule then plans a·q ≥ held, so it keeps its whole stock
  for a tick and sells none (OW3).

### 6.4 O14: like for like, and against I0 (`o14.out`)

Median / worst per tier (IW1 and IC1 from this mirror; I0 from MARKETS §4):

| | T1 peak D̂ | T2 peak D̂ | T2 dead | T2 lowest baskets | T3 peak D̂ | T3 dead | T3 lowest baskets | T3 worst transfer shortfall |
|---|---|---|---|---|---|---|---|---|
| IW1 (wall) | 56 / 370 | 215 / 1,113 | 0 / 7 | 0.853 / 0.330 | 853 / 2,401 | 0 / 117 | 0.502 / 0.000 | 706 |
| IC1 (line) | 61 / 272 | 222 / 1,047 | 0 / 20 | 0.848 / 0.476 | 819 / 2,664 | 2 / 124 | 0.504 / 0.000 | 1,082 |
| I0 (Appendix B) | — | 346 / 857 | 0 / 12 | 0.77 / 0.54 | 1,290 / 2,321 | 35 / 111 | 0.40 / 0.135 | 770 |

**Cost shocks at genesis.** Each row is ln(Y′/Y), the equilibrium's move, then the trough of
baskets in log against the old Y (its share of Y):

| shock | IW1 | IC1 |
|---|---|---|
| land.mach × 2 | −0.208 → −2.01 (0.134) | −0.144 → −2.37 (0.093) |
| land.mach × 0.5 | +0.123 → −0.44 (0.643) | +0.103 → −0.51 (0.600) |
| tail.services × 2 | **0.000** → −0.66 (0.516) | −0.020 → −0.45 (0.636) |
| res.services.trained × 2 | **0.000** → −0.69 (0.500) | −0.007 → −0.69 (0.500) |
| I0's land.mach × 2 (MARKETS §4) | −0.128 → −2.00 (0.135) | |

**Reading.**
- **The wall's medians are Appendix B's or better.** Its Tier-2 worst trough (0.330) is deeper
  than I0's (0.54) and IC1's (0.476). The run is s[goods]=0.2: pulling goods off the wall puts
  hours back into goods, and the pool is rationed.
- **At the wall, labour-demand shocks move no output in equilibrium,** since Y = T/B is fixed
  by land. Their paths still halve output for a while (OW2).

## 7. Registered predictions for the engine run

Registered before any engine code for IW1 exists. The conditions are those of the header and
§2–§5. The engine is expected to match the mirror as the markets probe's did (MARKETS §2) and as
P2.2 set the bands:
- **Classes** exactly.
- **Ticks and years** within 10%, with three runs a set allowed within 25%.
- **Troughs and lowest depths** within 5%.
- **Dead-tick and breach-tick counts** within 10% or 5 ticks, whichever is larger.

**E0. Before any mode-B run: a trace diff.**
- `wm.tick` against the engine for 2,000 ticks. `wm.py` carries the genesis carry, including a
  buyer's unused genesis-lot purchase.
- At IW1 the runs are hold, p[labour]\*2, JB(0.5), x\*/2 and s[goods]=0.5; RW(2),
  p[labour.trained]\*0.5, coin.workers.master\*0.1 and stock.mach\*0.01; and the three
  coefficients at ×2 or ×0.5 at genesis. At I0 the run is hold.
- Every price, share, coin, stock, S and D must agree within 1e-12 in log. The one allowed
  parting is the budget chain's ulp (MARKETS-RULES §6.4).

**E1. Nesting (R1).**
- With `tail`, `reserved` and `more` absent, every committed tape keeps its text, `tape_hash`,
  `world_id` and per-tick hash stream. Gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
  demo-gb `0xfad880fe08d06645` and the probe, markets, horses and loops pins do not move.
- `markets_i0_nests_appb` and `many_roles_nest_the_appendix_b_roles` pass unchanged, and so do
  §3.8's nesting tests.

**E2. The rest point and mode A.**
- Genesis from unit 1d's `WorkerEconomy` is a fixed point within 1e-12 after 3 ticks. This
  holds at the base and the 12 targets, at 12, 52 and 365 a year.
- Mode A passes at L. The mirror's largest gaps are 3.3e-16 (52 a year), 1.6e-15 (12) and
  4.4e-16 (365), and the engine's must be below 1e-9.

**E3. The verdict.**
- IW1 is GO. Tier 1 26/26, Tier 2 38/38, Tier 3 39/39 and Tier 3S 20/20. Tiers 3 and 3S are
  also run at 10·L, with the same classes and the same ticks to tolerance.
- The speeds, troughs, dead ticks by market and the wall readouts per tier are §6.2's table and
  its Tier 3S lines.
- The engine's own elasticity probe sets L. The mirror's τ are §6.1's (goods 106.4 ticks), so L
  is 22,000.

**E4. The kick sets and the slowest mode.**
- Every kick set decays at every target: gain_tail at most 1e-3, where the mirror gives at most
  1.5e-5.
- The engine's fitted decay at the base is within 0.03 a year of 0.534.
- The mirror's largest root per tick over the targets is 0.991911, and the technique's rate is
  exactly 0.951229.

**E5. The cost shocks.** §6.2's cost-shock rows and §6.4's depths:
- land.mach × 2 bottoms at 0.165 of the new Y\* with 93 dead ticks;
- tail.services × 2 at 0.516 and res.services.trained × 2 at 0.500, with no dead tick, although
  Y\* does not move.

**E6. The wall's own readouts.**
- In the registered battery only JB(0.5) breaches the wall: 11 ticks from tick 0, lowest depth
  −0.444, largest share 0.101.
- No other run's depth falls below 0.128, which is land.mach 0.8's path.
- The displaced shares follow s_t = s_(t−1)·(1 − share(adjust)) exactly while the wall binds.
  Their thresholds are back within tolerance at ticks 78, 105 and 124.
- No pop saturates in the registered battery.
- Every CONVERGED run ends at the wall, and every labour market trades at the end.
  - A share never displaced stays 0.0 exactly.
  - A displaced share decays to 5e-323, ten subnormal ulps, where a·s rounds to 0 and the
    update stops (s = 5e-323 after 30,000 ticks from 0.5).
  - The threshold x then reads 1.0.

**E7. The families.** Each is run and reported:
- stocks 31/31, joint2 60/60, joint4 40/40 and basin 516/516;
- history 79/79 windows in tolerance at their end, with each × 0.5 window's no-machine tick;
- 12 and 365 a year 64/64 each, Hold identical to Saturate, and tilt 1 64/64.

§6.3's numbers, within the bands above.

**E8. The line control.** IC1 converges 103/103 and its Tier 3S 20/20 at L = 25,000, with §6.4's
numbers (reported, not scored).

**E9. The edges.** The generator's own solve reproduces §2.4's gaps within 1e-12 at genesis and at
every target. A gap below 0.2 at a cost target means the tape is not the registered instance.

**What would refute the frame and send it back to the mirror:**
- a class change in the registered battery;
- a CONVERGED run ending more than 1e-12 in log off its target's oracle point;
- a converged run ending off the wall (a desk's share not back to 0), or a labour market dead at
  the end;
- the engine's rest point differing from unit 1d's by more than 1e-12;
- a kick set failing, or the slowest mode at or above 1 at any target;
- any committed pin moving.

**The scorer** is written and committed before the wave (decision 311). It reads the
registration's run-by-run outputs and scores the engine's runs against them in the bands above:
- `registered/runs_*.tsv`, one per set, Tier 3S included;
- `registered/local.json`;
- `registered/history_land.json`;
- `registered/edges.json`.

## 8. Decisions proposed and open items

P2.3.0's rulings (decisions 360–393, `f0c6667`) bind this frame. It follows each one that touches
it:
- 364: C2m;
- 366: 52 ticks a year;
- 368: a stocks tier in the new verdicts (Tier 3S, §5.2); O22's families on I1–I3 run before
  the wall's battery, which orders the run, not this frame;
- 369: ε per type on one line, with reserved tasks;
- 370: every edge's distance registered (§2.4);
- 371: how pooled types trade is the frame's call (one pooled type here);
- 372: machine recipes buy pool labour;
- 377: the wall instance in 1d's dependence form, with a walled type;
- 387: the fixed basket in the oracle and the agents;
- 393: ρ 0.

Its own proposals are below. Each is Claude's, on the user's delegation ("I leave all those calls
up to you"), and open to veto. The alternative named is the registered one (R6). They are labelled
FW1–FW9 and OW1–OW3 here, and STATE.md numbers them in its range when the frame lands, as
LOOP-SPEC's L1–L12 were numbered at their merge. 360–393 and O95–O96 are taken.

- **FW1. IW1 is Phase 2 proper's wall instance** (§1, §2). It is 1d's B economy with E7's three
  types, the trained and the master reserved-only: η 0.5, L^H 0.1, R 0.04 and 0.03, N 130/52/26 a
  year, χ_max 1/2/2, T 520, h 1 and Appendix B's machine. Alternatives:
  - 1d's E3 as written, which is unfunded under the probe's transfer (10.4 baskets of support
    against T 10);
  - a one-type wall (unit-1d.md's W1, λ 0.6), which has one labour market.
- **FW2. The reserved types sell only their reserved hours (ε 0), and every type's support is one
  basket (ν 1).** Each type has one market, and no rule chooses between markets (371, 377). With
  E's efficiencies the base point is the same double for double, both types walled (premia 1.076
  and 1.600), but at four targets the trained would join the pool. Alternative: E's ε and ν with
  a market-choice rule, its nesting and its mirror first (OW1).
- **FW3. The roles' additions are three optional fields**: the category desk's `tail` and
  `reserved` and the provider's `more` (§3). There is no new kind, state or market rule, and
  every committed tape keeps its hashes. Alternative: a new desk kind for categories with
  human-required or reserved tasks.
- **FW4. The cost targets are land.mach, tail.services and res.services.trained,** at ×1.1, ×0.9,
  ×2 and ×0.5, dated and at genesis. Every one is a funded wall at least 0.216 in log from every
  edge (§2.3, §2.4). Alternative: land.mach alone, as I0.
- **FW5. At the wall the harness observes each desk's threshold x_j = 1 − s_j in log,** not s_j,
  whose target 0 has no log. P2.1's instances keep s_j. Alternative: an absolute tolerance on
  s_j.
- **FW6. The wall's run grammar** (§5.2):
  - s[D]=V (V 0.05, 0.2, 0.5) replaces s[D]\*F, which is a no-op at s\* = 0;
  - JA and JB scale every labour market;
  - RW(F) is added where there are several labour markets;
  - basin takes the reserved wages in place of the technique.

  The battery is 103 runs. Alternative: keep s[D]\*F and mark those runs slack. They would be
  VACUOUS, and the technique would be tested only by x\*/2.
- **FW7. The wall's own readouts are reported, not scored** (§5.3), as O14's instruments are,
  beside the edges that 370 registers. Alternative: score breach ticks.
- **FW8. IC1 (the entrant's χ_max 0.25) is a reported like-for-like control,** not a verdict
  instance. Two of its targets are walls and one has the pool saturated. Alternative: no control.
- **FW9. The verdict rule is P2.1's, with 368's stocks tier.**
  - Mode A passes.
  - Every run of Tiers 1–3 and of Tier 3S (20 runs: each stock and coin × 0.5 and × 2, the start
    distance over the first year, decision 229) is CONVERGED, and Tiers 3 and 3S again at 10·L.
  - Every target's kick set passes.
  - O22's families are reported, stocks first.
  - L comes from the engine's elasticity probe.
  - The scorer is committed before the wave (decision 311).

  Alternative: score the families as well.

Open items:

- **OW1. A type that sells reserved and pool hours** (1d's pooled type with reserved work, E1 and
  E4; and pooled types with ε ≠ 1, or support ν ≠ 1, decision 138). IW1 has none. Each needs a
  rule that splits a pop's hours between two markets, or scales them to efficiency units. It
  needs that rule's nesting, a mirror and its own registration before a Phase 2 instance uses
  it. The eras' trained worker, who holds engines' tasks at some dials and the pool at others, is
  such a type. It joins O95's list where it needs the oracle.
- **OW2. At the wall, labour-demand shocks move wages, not output, but halve output on the way.**
  tail.services × 2 and res.services.trained × 2 move Y\* by 0 in log and bottom at 0.516 and
  0.500 of it. O14 and O24 carry this as the wall's path cost. Ex-post assignment might lift it,
  and is untested here.
- **OW3. O21's one-type cousin at the wall.** A step that cuts the machine desk's cost 3.5 times
  at the prices in force (history: land.mach 0.8 → 0.2) makes it keep its whole stock for a tick
  (a·q ≥ held). No baskets follow the next tick, in all 16 such windows. It recovers each time.
  A keep rule that shares the shortfall, or a stock (Phase 3), would remove it.

## 9. Files, and how to rerun

Everything is under `D:/rustyecon-p23/frame-wall/`; `run.sh` reruns it in WSL.

| file | what |
|---|---|
| `SPEC.md` | this file |
| `SHA256SUMS` | sha256 of this file and every registered output and script |
| `oracle/` (`Cargo.toml`, `src/main.rs`) | the scratch crate: IW1, its targets, the ε check and IC1 on unit 1d, by path from the worktree |
| `out/points.jsonl`, `registered/points.jsonl` | the oracle's points (52 lines: 13 targets × 3 tick lengths, and the ε check) |
| `hp/solve_mp.py`, `out/solve_mp.out`, `registered/solve_mp.out` | the 50-digit solve and its comparison |
| `hp/edges_mp.py`, `out/edges_mp.out`, `registered/edges.json` | the distance from each edge, at 50 digits (decision 370) |
| `explore/` | the search that chose the numbers (`wall_f64.py`, `search.py`, `search2.py`) and the edit scripts of this file; not registered |
| `model/mm_carry.py` | the markets probe's trace mirror, copied unedited (sha256 `dbd50ad9…538c`) |
| `model/p21_battery.py` | the markets predictor's battery, copied unedited (sha256 `a44bd25d…df06`) |
| `model/make_wm.py` → `model/wm.py` | the wall mirror, by 17 checked replacements of `mm_carry.py` |
| `model/wm2.py`, `wm2_check.py`, `wm2_check.out` | the second map and its agreement |
| `model/wall.py` | IW1, its targets, genesis from the oracle's doubles, observables and the depth |
| `model/wb.py` | the battery runner, run grammar and readouts |
| `model/wm_nest.py`, `wm_nest.out` | R1 in the mirror |
| `model/wm_rest.py`, `wm_rest.out`; `model/modea.py`, `modea.out` | the rest point and mode A |
| `model/fullL.py`, `fullL.out` | ten runs and IC1's mode A to the full L, without the early stop |
| `model/wel.py`, `wel.out`, `wel_line.out` | the elasticity probe and L |
| `model/wstab.py`, `run_local.py`, `local.out`, `local_line.out`, `registered/local.json` | PL and the kick sets |
| `model/run_battery.py`, `battery_*.json`, `summary.py`, `summary.out` | the battery and families |
| `model/o14.py`, `o14.out`, `registered/runs_*.tsv` | the O14 tables and the run-by-run outputs, one TSV per set (battery, tier3s, stocks, joint2, joint4, basin, tpy12, tpy365, hold, tilt1, line, tier3s_line) |
| `registered/history_land.json` | the history family's 81 windows |
| `registered/instances.json` | the instance, machine-readable |
| `run.sh` | rebuilds it all |
