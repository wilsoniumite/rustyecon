# FUNDED: a funded county for rule B's horse (O41)

Dated 2026-09-29. Label `funded`, for P2.2b's frame. It reads the worktree `D:/rustyecon-wt/p2l`
at `401f7b1` and changes nothing there. `crates/oracle` and `crates/core` are the same at
`ba6938d` (L0.1). Scratch is `D:/rustyecon-p2l/funded/`. Every solve here is
the worktree's oracle as it stands: unit 1g's `ChainEconomy` and `PlantEconomy`, and 1c's
`MachineEconomy`, called from a scratch crate. A 50-digit solve that reads no oracle code
confirms every point. The machine-readable copy is [instances.json](instances.json).

## 0. The answer

**The county, `chain8`.** It is 1g's HORSE county with 8 heads instead of 12. Only N changes:
N 8 and T 10 a tick at 52 a year (416 and 520 a year), h 1, χ_max 1/20, γ = 0.2 + 0.8x. CHAIN's
horse is unchanged:

- fodder is made from 1 land, 8 labour and 2 horse-days a ton (rule B, the loop);
- a head is made from 8 t fodder, 20 labour and 3 pasture;
- a head gives 250 horse-days a year;
- a horse-day takes 0.0176 t fodder and 0.1 labour;
- δ is 8% a year, J_b 1 tick and ρ 0.

**Why 1g's county cannot be funded.** A basket is one good plus h = 1 of space, so P_s ≥ 1
whatever the horse costs. Funding needs T > N·P_s, but N·h = 12 is already above T = 10. So the
fault is the county's, not the horse's. At 1g's point the good costs 0.015 of a basket's 1.015.
At every land factor from 0.25 to 4 the provider's baskets stay between −2.05 and −2.33.

**Why N 8.** Each head then has 1.25 of land against a basket's 1 of space. N 8 is the largest
round population on CHAIN's land that meets all four of these, at every cost target of P2.2a's
battery and at δ 4%, 8% and 10% a year:

- **Rent covers support by at least 21.5%.** Coverage T/(N·P_s) is 1.215 to 1.240. N 8.5 would
  give 1.145.
- **Rent still covers the transfer at genesis by 20.9%** under the battery's worst price
  displacement (p × 2, r × 0.5, JA(2) or JB(2)).
- **No displacement puts every head to work at genesis.** The largest z/χ_max is 0.908 (w × 2).
- **The equilibrium is unique and interior, with the loop live.** x\* is 0.747 at the base and
  0.523–0.940 over the targets. 4.05% of horse-days go into fodder. The oracle's sign sequence
  changes sign once at every point, and at ρ 0 uniqueness is also proved (unit-1c.md §5.5).

N 5 or fewer puts the horse on the whole line at b × 0.5.

**Funding, rule B (δ 8% a year).** Provider baskets, with coverage in brackets:

| | b ×1 | ×1.1 | ×0.9 | ×2 | ×0.5 |
|---|---|---|---|---|---|
| **B**, chain8 | +1.8387 (1.2298) | +1.8260 (1.2282) | +1.8519 (1.2315) | +1.7305 (1.2163) | +1.9109 (1.2389) |
| B, 1g's county | −2.1476 | −2.1581 | −2.1365 | −2.2329 | −2.0850 |

**The controls are funded too.**

- **The like-for-like flow control, B-flow**, takes the stocks layer off. It is the same point
  as B within 9.7e-16 relative, at every target and every δ.
- **The loop-cut control, B-cut**, makes fodder from farmland and labour alone. It is funded
  from +1.7399 (b × 2) to +1.9155 (b × 0.5) at δ 8%, with x\* 0.758 at the base.

**The independent check.** Over 105 points, the oracle's doubles agree with the 50-digit solve
within 9.5e-16 relative. Both count one sign change at every point. The same code reproduces
1g's HORSE goldens to their 30 digits.

**Plants.** Section 8 gives K\*, V and the user cost at every loop desk, at θ 0.8 and 0.7 and a
plant δ of 10% and 4% a year. It also gives s1: 40.47, 104.39, 64.48 and 166.33 bundles a plant
unit. At s1 every planted long run is the unplanted point: bit for bit in 67 of 140 runs, and
within 1.3e-15 relative in the rest.

## 1. How funding is computed

HORSES-SPEC §1.1 and unit-1a.md §3.4 define it. The provider owns the land. At rest it earns
T·r + interest, and interest is 0 at ρ 0. From that it pays each of N heads a basket's worth,
N·P_s, each tick. The rest buys its own baskets:

    provider baskets = (T + interest)/P_s − N,    funded ⇔ provider baskets > 0

Here r = 1 is the numeraire, and P_s = p + h is the basket's price: one good plus h of space. The
probe's `BasketProvider` pays N·(p + h·r) at posted prices from the coin it holds, so an unfunded
county drains that coin at rest. No agent run can then rest at the oracle's point (O41).

**Coverage** is T/(N·P_s), rent over the support due. **"Funded with a margin"** is read here
(decision 240) as three things:

- coverage of at least 1.2 at every cost target, at δ 4%, 8% and 10%;
- rent T·r′ of at least 1.2 times the due N·(p′ + h·r′) at the genesis of every battery price
  displacement;
- no displacement at genesis that lifts z = ln(1 + v′/P_s′) to χ_max. At that point every head
  offers labour, and supply sits at its bound.

The battery's displacements all start from the base point, so their target is the base point.
Only b × 1.1, × 0.9, × 2 and × 0.5 move the target (HORSES-RULES §5).

## 2. Why 1g's HORSE county is unfunded

1g built the HORSE county to put CHAIN's horse at the task margin (x\* 0.62), and 1g does not
require funding (decision 187). Its land is scarce against its labour: v = 0.0193 of a land
unit's rent, so the good costs 0.015. Its space per basket is h = 1. So P_s = 1.015, which is
98.5% space, and N·P_s = 12.18 against T = 10.

At the horse's margin the horse-day costs λ̃·v + b̃ = 0.28·v + 0.022, 80% of it land. So v/p_hd
stays between γ(0) = 0.2 and γ(1) = 1 only while v is between 0.0047 and 0.031 of r. Any county
that keeps CHAIN's horse on the task margin therefore pays labour little and land much. The good
is then cheap, and funding is decided by T against N·h.

The land factor (fodder's farmland and the head's pasture together) cannot fix it. On the grid
from 0.25 to 4, 1g's county runs from −2.05 to −2.33 with the loop and from −2.04 to −2.32 with
it cut. The frame recorded −2.08 to −2.23 on 0.5–2.

## 3. The search

Each county below was solved as rule B (stock, δ 8%) at P2.2a's five cost targets. The last
column takes the battery's worst genesis displacements from the base (`candidates.py` on
`out/scan.jsonl`). Only N/T matters to prices, since N and T scale together, so "fewer heads"
and "more land per head" are the same move.

| N | h | χ_max | regimes at the targets | coverage, least | x\* | N_a/N, most | worst displacement: coverage, z/χ_max |
|---|---|---|---|---|---|---|---|
| 4 | 1 | 0.05 | horse on the whole line at b ×0.5 | | | | |
| 5 | 1 | 0.05 | horse on the whole line at b ×0.5 | | | | |
| 6 | 1 | 0.05 | interior, 1 change | 1.618 | 0.630–0.984 | 0.741 | 1.611, 0.966 |
| 7 | 1 | 0.05 | interior, 1 change | 1.389 | 0.581–0.953 | 0.692 | 1.382, 0.919 |
| 7.5 | 1 | 0.05 | interior, 1 change | 1.297 | 0.559–0.938 | 0.670 | 1.291, 0.897 |
| **8** | **1** | **0.05** | **interior, 1 change** | **1.216** | **0.538–0.924** | **0.650** | **1.210, 0.876** |
| 8.5 | 1 | 0.05 | interior, 1 change | 1.145 | 0.518–0.910 | 0.631 | 1.140, 0.857 |
| 9 | 1 | 0.05 | interior, 1 change | 1.082 | 0.499–0.897 | 0.613 | 1.077, 0.839 |
| 9.5 | 1 | 0.05 | interior, 1 change | 1.026 | 0.482–0.884 | 0.596 | 1.020, 0.821 |
| 10 | 1 | 0.05 | interior, 1 change | 0.975 | 0.465–0.871 | 0.580 | 0.970, 0.805 |
| 12 (1g) | 1 | 0.05 | interior, 1 change | 0.814 | 0.406–0.824 | 0.526 | 0.809, 0.746 |
| 12 | 0.5 | 0.05 | interior, 1 change | 1.588 | 0.437–0.827 | 1.000 | 1.572, 1.455 |
| 12 | 0.6 | 0.05 | interior, 1 change | 1.335 | 0.412–0.826 | 0.865 | 1.323, 1.222 |
| 12 | 0.65 | 0.05 | interior, 1 change | 1.236 | 0.411–0.826 | 0.801 | 1.225, 1.132 |
| 12 | 0.7 | 0.05 | interior, 1 change | 1.151 | 0.410–0.825 | 0.745 | 1.142, 1.054 |
| 12 | 0.8 | 0.05 | interior, 1 change | 1.011 | 0.408–0.825 | 0.654 | 1.004, 0.926 |
| 7 | 1 | 7/240 | interior, 1 change | 1.395 | 0.406–0.824 | 0.901 | 1.387, 1.278 |
| 8 | 1 | 1/30 | interior, 1 change | 1.221 | 0.406–0.824 | 0.789 | 1.214, 1.119 |
| 9 | 1 | 0.0375 | interior, 1 change | 1.085 | 0.406–0.824 | 0.701 | 1.079, 0.994 |

Three one- or two-number moves fund the loop with a 20% margin. They differ in how close the
battery pushes labour supply to its bound.

- **N 8, χ_max 1/20 (chosen).** This is one number. The point moves: x\* goes from 0.624 to
  0.747, because labour is scarcer and horses take more tasks. Supply stays below its bound in
  every displacement: z/χ_max reaches 0.876 at δ 8% and 0.908 at δ 10%.
- **N 8 with χ_max 1/30 (not chosen).** This is two numbers, holding N/χ_max at 1g's 240. Labour
  supply per unit of z is then unchanged, so every price and quantity is 1g's HORSE point at
  every target, within 5.7e-16. What changes is that the four heads who never work at any target
  (χ above 1/30) are removed. It is the most surgical move. But the heads that remain have the
  full range of χ below 1/30, so a doubled real wage saturates them. w × 2, r × 0.5, JA(2) and
  JB(2) start with every head offering labour (z/χ_max 1.12–1.15), a regime no earlier probe ran.
- **h 0.65 (not chosen).** This is one number, and x\* stays near 0.627. But it saturates the
  same four runs (1.13–1.17).

At h ≤ 0.5 participation reaches 1 at b × 2, and the rest point loses its labour margin.

## 4. The chosen county and its three instances

### 4.1 Parameters, in a form the tape builder can take

These are P2.2a's keys where one exists (HORSES-SPEC §2.10; `probe::horses::setup`). Two keys
are new, because rule B's goods take goods: fodder's horse-days, and a head's fodder.

| key | value | unit | B | B-cut | note |
|---|---|---|---|---|---|
| `inst.workers` | 416 | FlowPerYear | ✓ | ✓ | 8 a tick at 52 a year (was 624) |
| `inst.land` | 520 | FlowPerYear | ✓ | ✓ | 10 a tick |
| `inst.chi_max` | 0.05 | Dimensionless | ✓ | ✓ | |
| `inst.eta`, `inst.g0`, `inst.g1`, `inst.k` | 1, 0.2, 0.8, 1 | Dimensionless | ✓ | ✓ | γ = 0.2 + 0.8x |
| `inst.good.weight`, `inst.space.weight` | 1, 1 | Dimensionless | ✓ | ✓ | h 1 |
| `inst.fodder.own` | 0 | Dimensionless | ✓ | ✓ | |
| `inst.fodder.traction` (new) | 2 | Dimensionless | 2 | **0** | horse-days a ton: the loop |
| `inst.fodder.labour` | 8 | Dimensionless | ✓ | ✓ | |
| `inst.fodder.land` | 1 | Dimensionless | ✓ | ✓ | × f under b × f |
| `inst.horse.run.fodder` | 0.0176 | Dimensionless | ✓ | ✓ | a horse-day's running recipe |
| `inst.horse.run.labour` | 0.1 | Dimensionless | ✓ | ✓ | |
| `inst.horse.kappa` | 250 | FlowPerYear | ✓ | ✓ | 250/52 = 4.8076923076923075 a tick |
| `inst.horse.delta` | 0.08 | FractionPerYear | ✓ | ✓ | `Clock::fraction`: 0.0016022075724025087 a tick |
| `inst.horse.own_hours` | 0 | Dimensionless | ✓ | ✓ | rule B's head takes no horse-days directly |
| `inst.horse.fodder` (new) | 8 | Dimensionless | ✓ | ✓ | fodder a head |
| `inst.horse.labour` | 20 | Dimensionless | ✓ | ✓ | |
| `inst.horse.land` | 3 | Dimensionless | ✓ | ✓ | pasture, × f under b × f |

Units and ticks:

- **Every recipe is per unit made:** a ton, a head or a horse-day. None depends on the tick
  length. Only N, T and κ are per year, and δ is a fraction per year; `Clock` converts them.
  (Rule A's head recipe, a·κ/δ, was per tick; rule B's is not.) Other tick lengths are
  untested here, and weekly is the floor anyway (decision 237).
- **J_b is 1 tick.** M3's hours use J 2 (decision 232's rule). At ρ 0, B at J 2 equals B at J 1
  bit for bit at every target and δ.
- **The cost shock b × f** multiplies `inst.fodder.land` and `inst.horse.land` together. This
  is the frame's HORSE-A land factor (decision 242).
- **The basis** is `Assumed("CHAIN.md §1.3-1.4 via unit-1g.md §3.3")` for the chain and
  `Assumed("funded 2026-09-29: 1g's HORSE county at N 8")` for the county. Neither is ever
  scored (R5).

### 4.2 The three instances

- **B, rule B: the loop, P2.2b's instance.**
  - A maker (M2) breeds heads from fodder, labour and pasture.
  - A wet capacity desk (M3) holds them at δ and sells horse-days to the good desk (tasks) and
    to the fodder desk (the loop).
  - The fodder desk buys horse-days, labour and land.
  - The oracle is `ChainEconomy` on the chain above.
- **B-flow, the like-for-like flow control: the stocks layer off** (decision 243).
  - In operating form it has two flow types on the loop, the FH shape CAPACITY ran: fodder as
    in B, and the horse-day as a flow good. The horse-day is made from:
    - fodder 0.020266073400477775 (0.0176 + 8δ/κ);
    - labour 0.10666518350119444 (0.1 + 20δ/κ);
    - land 0.0009997775251791654 (3δ/κ).

    Here δ/κ = 0.0003332591750597218 of a head's recipe is folded into each horse-day. In the
    engine that is a P2.1 `TypeDesk` for horse-days.
  - The same point on M1's δ = 1 path: B's chain with a one-tick head built from δ_tick times a
    head's recipe (fodder 0.01281766057922007, labour 0.03204415144805017, pasture
    0.004806622717207526), δ 1, κ 4.8077. It equals B on every aggregate compared, bit for
    bit, at all 15 points.
  - The operating form equals B within 9.3e-16 relative. A third form, the head as its own flow
    good, is within 9.7e-16.
  - M3 has no δ = 1 reduction (HORSES-SPEC §1.6). So the wet path's control is the flow desk,
    and M1's path is the δ = 1 form.
- **B-cut, the loop cut** (the frame's HORSE-A).
  - It is B with `inst.fodder.traction` 0: fodder from 1 land and 8 labour.
  - Its point differs from B's, since fodder no longer carries horse-days: x\* 0.758 against
    0.747. It has a flow form too (`B-cut flow op` in `out/point_C.jsonl`).

## 5. The solves

### 5.1 B at its base (δ 8%): prices relative to r = 1, volumes a tick at 52 a year

x\* 0.7470390416580641, v 0.02276607195182097, P_s 1.0163947061113316, Y 9.918084987225372,
N_a 3.5442686796783422 (participation 0.4430), income 10.080689075778142. Provider baskets are
+1.8386974468407367 and coverage 1.229837. Lemma B.1 holds (1g's county failed it). The sign
sequence is (9.2539, −3.1065), and the largest residual 4.4e-16.

| market | price | traded a tick | who |
|---|---|---|---|
| labour | 0.02276607195182097 | 3.5442686796783422 | good desk 2.508888, fodder desk 0.624511, capacity desk 0.385195, maker 0.025674 |
| land | 1 | 10 | households 9.918085, fodder desk 0.078064, maker (pasture) 0.003851 |
| fodder | 1.2392127794436407 | 0.07806391887609918 | capacity desk 0.067794, maker 0.010270 |
| horse-days | 0.028542101914536447 (O 0.024086752113390175, δV 0.004455349801146272) | 3.851950860607206 | good desk (tasks) 3.695823, fodder desk 0.156128 |
| horses (a head) | 13.369023674585545 | 0.001283697966176543 made and sold | capacity desk, which holds 0.8012057790062989 heads |
| the good | 0.016394706111331647 | 9.918084987225372 | households |

**The loop.** 4.05% of all horse-days go into fodder: 2·0.0176 + 2·8·δ/κ = 0.0405 horse-days
per horse-day, through fodder. The goods matrix's per-period spectral radius is 0.23968,
1g's HORSE radius (δ-weighted, D-G10). A horse-day's totals are λ̃ 0.28015 labour and b̃
0.022164 land, as 1g's.

**B-cut at its base:** x\* 0.758418, v 0.021906, P_s 1.015659. Fodder costs 1.17525, a head
12.8401 and a horse-day 0.0271541. 3.786884 horse-days a tick, all on tasks. Provider baskets
+1.8458.

### 5.2 Every instance at every target (δ 8%)

| instance | b × | x\* | v | P_s | Y | N_a/N | provider baskets | coverage |
|---|---|---|---|---|---|---|---|---|
| B | 1 | 0.747039 | 0.022766 | 1.016395 | 9.91808 | 0.4430 | +1.8387 | 1.2298 |
| B | 1.1 | 0.719682 | 0.024165 | 1.017711 | 9.91512 | 0.4693 | +1.8260 | 1.2282 |
| B | 0.9 | 0.776535 | 0.021277 | 1.015028 | 9.92153 | 0.4149 | +1.8519 | 1.2315 |
| B | 2 | 0.537749 | 0.033925 | 1.027698 | 9.90202 | 0.6496 | +1.7305 | 1.2163 |
| B | 0.5 | 0.923717 | 0.014120 | 1.008988 | 9.94204 | 0.2779 | +1.9109 | 1.2389 |
| B-flow | 1 | 0.747039 | 0.022766 | 1.016395 | 9.91808 | 0.4430 | +1.8387 | 1.2298 |
| B-flow | 1.1 | 0.719682 | 0.024165 | 1.017711 | 9.91512 | 0.4693 | +1.8260 | 1.2282 |
| B-flow | 0.9 | 0.776535 | 0.021277 | 1.015028 | 9.92153 | 0.4149 | +1.8519 | 1.2315 |
| B-flow | 2 | 0.537749 | 0.033925 | 1.027698 | 9.90202 | 0.6496 | +1.7305 | 1.2163 |
| B-flow | 0.5 | 0.923717 | 0.014120 | 1.008988 | 9.94204 | 0.2779 | +1.9109 | 1.2389 |
| B-cut | 1 | 0.758418 | 0.021906 | 1.015659 | 9.91947 | 0.4268 | +1.8458 | 1.2307 |
| B-cut | 1.1 | 0.731606 | 0.023285 | 1.016936 | 9.91639 | 0.4528 | +1.8335 | 1.2292 |
| B-cut | 0.9 | 0.787251 | 0.020441 | 1.014334 | 9.92302 | 0.3990 | +1.8587 | 1.2323 |
| B-cut | 2 | 0.551774 | 0.032964 | 1.026705 | 9.90223 | 0.6320 | +1.7399 | 1.2175 |
| B-cut | 0.5 | 0.929809 | 0.013447 | 1.008520 | 9.94377 | 0.2649 | +1.9155 | 1.2394 |

Each good's price and volume at every target is in `instances.json` (`points`): fodder, a head,
the horse-day with its O and δV, the good, and each market's split by buyer. At b × 2 fodder is
2.37907, a head 25.7110 and a horse-day 0.053833. At b × 0.5 they are 0.64304, 6.9267 and
0.015038.

### 5.3 The families: the horse's δ at 10% and 4% a year

At ρ 0 rule B's point depends on δ, because a head's recipe is fixed per head. (In rule A the
build scales as 1/δ.) The table gives provider baskets, with coverage in brackets.

| δ | instance | b ×1 | ×1.1 | ×0.9 | ×2 | ×0.5 |
|---|---|---|---|---|---|---|
| 10% | B, B-flow | +1.8313 (1.2289) | +1.8182 (1.2273) | +1.8450 (1.2306) | +1.7207 (1.2151) | +1.9062 (1.2383) |
| 10% | B-cut | +1.8390 (1.2299) | +1.8262 (1.2283) | +1.8522 (1.2315) | +1.7306 (1.2163) | +1.9112 (1.2389) |
| 4% | B, B-flow | +1.8531 (1.2316) | +1.8412 (1.2301) | +1.8655 (1.2332) | +1.7501 (1.2188) | +1.9200 (1.2400) |
| 4% | B-cut | +1.8593 (1.2324) | +1.8477 (1.2310) | +1.8713 (1.2339) | +1.7585 (1.2198) | +1.9239 (1.2405) |

x\* at the base is 0.733943 at 10% and 0.772978 at 4%. Over the targets it runs 0.5225–0.9152
at 10% and 0.5688–0.9398 at 4%.

### 5.4 Beyond the targets

On the grid of land factors 0.4–4 (δ 8%) B and B-cut are interior with one sign change, and
funded. Coverage falls slowly as land dearens (1.199 at × 4). At × 0.33 and below the horse takes
the whole line (`BoundaryNoMargin`). So a target below about × 0.35 is a corner, not an interior
point (O52).

## 6. The funding table

**At rest.** Section 0's table and §5.2–§5.3 cover it. Over 5 targets × 3 δ × 3 instances
(45 points), the least provider baskets are +1.7207 (B at δ 10%, b × 2) and the least coverage
is 1.2151. 1g's county at the same points runs from −2.08 to −2.24.

**At the battery's genesis, from B's base (δ 8%).** Each row gives rent over the transfer due,
T·r′/(N·(p′ + h·r′)), and z/χ_max = ln(1 + v′/P_s′)/χ_max. Displacements of fodder's, the
horse's or the horse-day's price, of the technique, and of stocks and coins move neither.

| run | coverage | z/χ_max |
|---|---|---|
| (base) | 1.2298 | 0.4430 |
| w × 1.2 | 1.2298 | 0.5305 |
| w × 2 | 1.2298 | 0.8765 |
| w × 0.5 | 1.2298 | 0.2227 |
| r × 1.2 | 1.2332 | 0.3709 |
| r × 2 | 1.2398 | 0.2245 |
| r × 0.5 | 1.2103 | 0.8628 |
| p × 1.2 | 1.2259 | 0.4416 |
| p × 2 | 1.2103 | 0.4361 |
| p × 0.5 | 1.2398 | 0.4466 |
| JA(2), JB(2) | 1.2103 | 0.8628 |
| JA(0.5), JB(0.5) | 1.2398 | 0.2245 |
| N(2), N(0.5) | 1.2298 | 0.4430 |

The worst over the three δ bases is coverage 1.2085 (p × 2, r × 0.5, JA(2) or JB(2)) and
z/χ_max 0.908 (w × 2), both at δ 10%.

The provider's coin sets how a displaced run crosses the first ticks. N(2) doubles the due in
coin before rent catches up. That is a question for genesis, not for funding.

## 7. The independent check

`mp/solve_mp.py` writes the equilibrium from the model's definition and reads no oracle code:
the three price rows, the margin v = γ(x)·p_S, the good's unit cost, land clearing,
labour clearing and the labour-supply rule. It solves at 50 digits (mpmath 1.3.0 in
laborformal's venv), bisecting x\* to 2^-175. It counts the sign changes of labour's excess
demand on a grid of 4,000 points at the base and 400 elsewhere, both corners included. It solves
the flow control in its own operating form. Its plants come from the cost minimisation of
y = K^(1−θ)·z^θ, not from a closed form, and it checks the planted price row to 1e-45.

`compare_mp.py` (output in `out/compare_mp.out`):

- **105 points** (chain8, the χ_max 1/30 alternative and 1g's county; B, B-flow and B-cut):
  - x\*, v, P_s, Y, N_a, provider baskets, participation, fodder's, the head's and the
    horse-day's prices, and their volumes agree within 9.5e-16 relative.
  - Every sign count is 1 in both.
- **80 plant readouts** (chain8, 5 targets × 4 desks × 2 θ × 2 δ): s1, ζ, κ, K\*, c_f, P_K, V,
  u·V and the plant's value agree within 5.9e-16 relative.
- **1g's goldens.** The 50-digit solve of 1g's county matches `goldens_1g.txt`'s HORSE_X_STAR,
  _V, _P_S, _Y, _N_A and _FODDER_PRICE within 4.8e-30 relative.
- **The loop's radius**, 0.2396807664115420847468, is 1g's HORSE_RADIUS_PER_PERIOD.

**Uniqueness.** At ρ 0 there is one land and one line with a single task type. So labour's
excess demand does not rise in x (unit-1c.md §5.5, Lemma 2 on the chain's totals, which a loop
leaves x-free), and the interior root is the only one. Both counts agree with this.

## 8. Plant readouts at each loop desk

CAPACITY's plant makes y = K^(1−θ)·z^θ from s1 bundles of the desk's own recipe per plant unit,
worn at the plant's δ. At ρ 0 and s1 its long run is the unplanted point. So the oracle needs
only readouts (decision 184; unit-1g.md §4.5):

- s1 = θ^(θ/(1−θ))·(1 − θ)/δ_p;
- ζ = θ and κ_p = θ^(−θ/(1−θ)), which is 2.44141 at θ 0.8 and 2.29847 at 0.7;
- K\* = κ_p·X plant units;
- P_K = s1·c_f per plant unit;
- V = κ_p·P_K per unit of the desk's capacity a tick;
- u = δ_p, and the user cost u·V = (1 − θ)·c_f per unit of output.

**Where the plants go.** The loop's two sides are the fodder desk and the capacity desk. The
maker is on the loop too, through a head's 8 t of fodder, with weight δ/κ.

1g plants only flow types, so each desk's readout comes from the form where its type is a flow
type:

- **Fodder desk:** `PlantEconomy` on B's embedding, type FODDER.
- **Maker:** the same, type HORSE (the good).
- **Capacity desk:** two readings of its bundle, since the plant on a desk that also holds horses
  is still untested (O49; decision 244).
  - **Full recipe.** The bundle is fodder, labour and δ/κ heads per horse-day. `PlantEconomy`
    runs on the flow form, where the head's wear is folded in. This is CAPACITY's FH reading.
  - **Running recipe.** The bundle is 0.0176 fodder and 0.1 labour. The plant sits beside the
    horse holding and the horse keeps its own user cost. §4.5's closed form at s1 gives it.

Both readings leave the point unchanged at s1.

**At B's base (horse δ 8%), per tick at 52 a year.** Plant δ 10% a year is 0.0020241124785003953
a tick, and 4% is 0.0007847302941671989.

| desk (bundle) | θ | δ_p a year | s1 | K\* | c_f | P_K | V | u·V | plant's value |
|---|---|---|---|---|---|---|---|---|---|
| fodder | 0.8 | 10% | 40.4721 | 0.190586 | 1.23921 | 50.1535 | 122.445 | 0.247843 | 9.55854 |
| fodder | 0.8 | 4% | 104.3926 | 0.190586 | 1.23921 | 129.365 | 315.832 | 0.247843 | 24.6550 |
| fodder | 0.7 | 10% | 64.4835 | 0.179427 | 1.23921 | 79.9088 | 183.668 | 0.371764 | 14.3378 |
| fodder | 0.7 | 4% | 166.3271 | 0.179427 | 1.23921 | 206.115 | 473.747 | 0.371764 | 36.9826 |
| capacity (full recipe) | 0.8 | 10% | 40.4721 | 9.40418 | 0.0285421 | 1.15516 | 2.82021 | 0.00570842 | 10.8633 |
| capacity (full recipe) | 0.8 | 4% | 104.3926 | 9.40418 | 0.0285421 | 2.97958 | 7.27437 | 0.00570842 | 28.0205 |
| capacity (full recipe) | 0.7 | 10% | 64.4835 | 8.85357 | 0.0285421 | 1.84050 | 4.23031 | 0.00856263 | 16.2950 |
| capacity (full recipe) | 0.7 | 4% | 166.3271 | 8.85357 | 0.0285421 | 4.74732 | 10.9116 | 0.00856263 | 42.0308 |
| capacity (running recipe) | 0.8 | 10% | 40.4721 | 9.40418 | 0.0240868 | 0.974840 | 2.37998 | 0.00481735 | 9.16757 |
| capacity (running recipe) | 0.8 | 4% | 104.3926 | 9.40418 | 0.0240868 | 2.51448 | 6.13886 | 0.00481735 | 23.6466 |
| capacity (running recipe) | 0.7 | 10% | 64.4835 | 8.85357 | 0.0240868 | 1.55320 | 3.56997 | 0.00722603 | 13.7514 |
| capacity (running recipe) | 0.7 | 4% | 166.3271 | 8.85357 | 0.0240868 | 4.00628 | 9.20829 | 0.00722603 | 35.4699 |
| maker | 0.8 | 10% | 40.4721 | 0.00313403 | 13.3690 | 541.072 | 1320.98 | 2.67380 | 1.69573 |
| maker | 0.8 | 4% | 104.3926 | 0.00313403 | 13.3690 | 1395.63 | 3407.29 | 2.67380 | 4.37393 |
| maker | 0.7 | 10% | 64.4835 | 0.00295053 | 13.3690 | 862.082 | 1981.46 | 4.01071 | 2.54360 |
| maker | 0.7 | 4% | 166.3271 | 0.00295053 | 13.3690 | 2223.63 | 5110.94 | 4.01071 | 6.56090 |

Reading the table:

- **The plant stock and its user cost do not depend on δ_p.** K\* and u·V are the same at 10%
  and 4%. The plant's price and value scale as 1/δ_p. A plant is worth (1 − θ)/δ_p periods of
  the desk's revenue: 99 weeks at θ 0.8 and 10%.
- **The capacity desk's plant user cost** is 20% (θ 0.8) or 30% (θ 0.7) of the horse-day's
  price under the full recipe. Under the running recipe it is 16.9% or 25.3%, and the horse's
  own δV is the other 15.6%.
- **The targets.** Every readout at every cost target is in `instances.json` (`plants`, 80
  rows).
- **At s1 each planted long run is the unplanted point.** It is bit for bit on x\* and v in 67
  of 140 runs of `PlantEconomy` (7 placements × 2 θ × 2 δ_p × 5 targets), and within 1.3e-15
  relative in the rest.

## 9. What P2.2b's frame should carry

- **The county is space-heavy (O51).** The good is 1.6% of income: p·Y is 0.163 against
  I = 10.08, and the horse chain uses 0.8% of the land. This is 1g's construction, since CHAIN's
  labour-heavy horse sits at the margin only where labour is cheap against land. So funding
  hardly moves with the horse's costs: coverage is 1.216 at b × 2 and 1.199 at × 4. The loop's
  markets are also small in coin against the land market and the provider. A county whose goods
  side weighs more would need γ's line or h moved, which is a larger change than O41 asks. P2.2b
  should size its coins, dead-tick bars and runaway bounds on these volumes, not A0's.
- **Labour supply's headroom.** w × 2 reaches z/χ_max 0.88 at δ 8% (0.91 at δ 10%), against
  0.59 for w × 2 on A0. A larger wage displacement than the battery's, or a displacement stacked
  on a cost shock, can reach the support's bound (every head at work). That is not a clamp, but
  no probe has run it.
- **The human share at b × 0.5 is small.** It is 1 − x\* = 0.076 (0.060 at δ 4%), against A0's
  0.063 in P2.2a. At land × 0.33 and below the point is a corner (O52).
- **The loop's gain.** 0.0405 horse-days per horse-day pass through fodder, and the per-period
  radius is 0.240 (L2's ρ(A) is 0.279). LOOPS.md's bullwhip is behavioural, so the mirror's loop
  step, not this radius, predicts the dynamics (O49).
- **Rule B's point depends on the horse's δ.** It does not under rule A. So a δ family is a
  different oracle point: 10% moves x\* from 0.747 to 0.734.
- **The plant on the capacity desk.** The mirror's loop step should choose the bundle (running
  or full recipe) before the frame registers readouts. Both are tabled here, and both leave the
  point unchanged at s1.

## 10. Decisions proposed, and open items

Each decision is open to veto; the registered alternative is named. They are numbered from 240
and the open items from O51, as asked. If other work lands first, STATE.md's merge renumbers them.

240. **The funded rule-B county is 1g's HORSE county at N 8 (`chain8`).** N 12 → 8, and T 10,
     h 1, χ_max 1/20 and γ stay 1g's. CHAIN's horse is unchanged. It is funded with coverage
     1.215–1.240 at every P2.2a cost target at δ 4–10%. Rent covers the transfer by 20.9% or
     more at every genesis displacement, and no displacement saturates labour supply (z/χ_max
     ≤ 0.908). It is unique and interior with the loop live (x\* 0.52–0.94). "With a margin" is
     read as coverage ≥ 1.2 at rest and at genesis, and z/χ_max < 1 at genesis. Alternatives:
     - N 8 with χ_max 1/30, which is exactly 1g's HORSE point, but four Tier-3 displacements
       start with every head at work (z/χ_max 1.12–1.15);
     - h 0.65, which saturates the same four runs;
     - N 7, with coverage 1.389 but x\* up to 0.953 and z/χ_max 0.92.
241. **P2.2b's rule-B instance keeps CHAIN's δ 8% a year, J_b 1 tick and M3's hours at J 2.** At
     ρ 0, J 2 equals J 1 bit for bit. δ 10% and 4% are families, each a different oracle point,
     and all are funded. Alternative: δ 10% as the verdict δ, P2.2a's H1.
242. **Rule B's cost shock b × f multiplies every land coefficient**, fodder's farmland and the
     head's pasture together. This is the frame's HORSE-A land factor, at P2.2a's four factors.
     Alternative: fodder's land alone, which leaves the head's cost to labour and fodder.
243. **The like-for-like flow control (decision 235) is the operating form.** It has two flow
     types on the loop (the FH shape), with a head's wear folded into the horse-day at δ/κ
     heads, run by a flow desk. M1's δ = 1 form, a one-tick head built from δ_tick times a
     head's recipe, is the same point for the owner path. Alternative: the δ = 1 form for both.
244. **The capacity desk's plant readouts are tabled under both bundles.** One is the running
     recipe (fodder, labour), beside the horse holding. The other is the full recipe (with δ/κ
     heads), as CAPACITY's FH. Neither is chosen here; the mirror's loop step (O49) chooses.
     Alternative: choose the running recipe now, which is what a desk that already holds
     horses would build a plant of.
245. **Rule B's recipes are per unit made at every tick length.** Only N, T, κ (per year) and δ
     (a fraction per year) go through `Clock`. So rule B's tape params need no per-tick
     restatement, unlike rule A's head. It is untested at other tick lengths. Alternative:
     restate per tick as rule A does, which changes nothing here.

- **O51. The funded rule-B county is space-heavy.** The good is 1.6% of income and the horse
  chain 0.8% of the land, since CHAIN's labour-heavy horse sits at the task margin only where
  labour is cheap against land (v 0.023 of r). Funding is robust for that reason, but the loop's
  markets are small against the land market and the provider. A county where they weigh more
  needs γ's line or h moved, with its own funding and uniqueness check. Its labour supply sits
  0.88–0.91 of the way to its bound at w × 2.
- **O52. The corner below land × 0.35.** chain8's horse takes the whole line at land factors of
  0.33 and below (`BoundaryNoMargin`). At b × 0.5 the human share is 0.060–0.092. A P2.2b target
  below × 0.4 must be registered as a corner.

## 11. Files, and how to rerun

Under `D:/rustyecon-p2l/funded/`:

- `oracle/`: the scratch crate. It reads `/mnt/d/rustyecon-wt/p2l/crates/{oracle,core}` by path
  and builds to `/root/scratch/target-p2l-funded`, with libm 0.2.16 as the workspace's lockfile.
  Its commands:
  - `p2l-funded sanity` gives the frame's −2.1476 and −2.1419;
  - `scan` gives the county search;
  - `point N T H CHI` gives every instance at every target and δ, and the grid;
  - `plants N T H CHI` gives the plant readouts.
- `run.sh` runs all of that in WSL, then `margins.py` and `build_instances.py`:
  `wsl -d ubuntu --exec bash -lc 'bash /mnt/d/rustyecon-p2l/funded/run.sh'`.
- `mp/solve_mp.py` is the 50-digit solve. `compare_mp.py` holds the oracle to it. `candidates.py`
  builds §3's table. Run them on Windows under laborformal's venv (mpmath 1.3.0):
  `…/laborformal/venv/Scripts/python.exe mp/solve_mp.py > out/mp.jsonl`, then
  `PYTHONIOENCODING=utf-8 …/python.exe compare_mp.py > out/compare_mp.out`.
- `out/`: every JSON line, and `tables.md`, `candidates.md`, `margins.out` and
  `compare_mp.out`.
- `instances.json` holds:
  - the county, the chain and the three instances;
  - the tape params;
  - every point at 3 δ × 5 targets, with each market's split and the 50-digit values to 40
    digits;
  - the genesis displacements, the 80 plant readouts and the loop's numbers;
  - the alternatives, and the check's figures.
- `SHA256SUMS`.
