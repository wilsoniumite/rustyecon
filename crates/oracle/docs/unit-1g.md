# Unit 1g: machines as goods

Dated 2026-09-27. This is the unit's specification, written before any Rust. The draft of
`goldens/generate_1g.py`, run at 70 digits before the build, checked every derivation below and
supplied the numbers in §7; the committed generator must reproduce them. The build records where
it departs from this draft in §12.

Unit 1g is the oracle's addendum for machines built from goods, your request of 2026-09-27 that
"goods use other goods rather than abstract machine services". Machines become durable goods
(a horse, a steam engine) built from goods (fodder, iron) and run on goods (fodder, coal), held as
stocks whose hours of use are the paper's machine services. The task margin is unchanged. At rest
such an economy is a unit 1c economy with more rows, once one check changes (D-G10). 1g builds
that check, the mapping from a chain of goods to 1c, plants as machine types (the capacity
damper's long run), and their goldens, and it adds tests for some of O28's surviving mutants.

- **Design sources**, read-only, outside the repository: `D:/rustyecon-goods/GOODS-CHAIN.md`
  (§1 the equivalence, §2 the addendum and its gates, §7 decisions D-G1 to D-G15), its passes
  `oracle/ORACLE-GOODS.md` (§1.2-1.6 the chain, S1 and S2; §2.1 the mapping; §2.9 the gates),
  `agents/AGENTS-GOODS.md` (§1, A0) and `chain/CHAIN.md` (§1.3-1.4, the horse); and
  `D:/rustyecon-loops/capacity/CAPACITY.md` with `capacity/model/cap.py` (the plant, its long
  run, s1, and check 2c's fixed points).
- **Paper:** SSRN 7226858 (posted 2026-09-23): A.1 (p = Ap + λw + Br over produced goods,
  "productive, with spectral radius of A below one", the closure, clearing), A.4 (the durable
  asset and the user cost).
- **Reference checks** (laborformal `31b3482`): `dynamics/checks/check_dynamics.py` U3 (the user
  cost) and L1-L2 (interest as (u − δ)VX).
- **Built on:** units 1a to 1f (docs/unit-1a.md to unit-1f.md). Every equation, convention,
  constant and precision rule of those units carries over unless this document says otherwise.
  1g changes one rule of 1c (§2.1) and adds two modules beside the solver; it does not change a
  solve.
- **Goldens:** a new generator, `goldens/generate_1g.py`, writing `goldens/goldens_1g.txt` (§7).

## 0. What the sources give

| source | what 1g takes from it |
|---|---|
| GOODS-CHAIN §0-1, ORACLE-GOODS §1.2-1.3 | the stacked system over categories, materials, machine goods and hours; the stock entering its hours' row at u on the price side and δ on the clearing side; E1-E6; the claim that under E1 the chain is a 1c economy with more rows, and its proof row by row |
| GOODS-CHAIN §1, ORACLE-GOODS §1.1, §2.1 | the embedding: each material and machine good a flow type (θ 0, its recipe as the operating recipe, no build recipe, δ 1, J 1), each machine's hours a type built from its good, over hours a period per machine |
| ORACLE-GOODS §1.4 | the fold (a) through Φ = (I − A_GG)⁻¹, the five numbers (b) per task type, and the smallest 1c economy with them (c) |
| ORACLE-GOODS §1.6 | S1, the steam chain calibrated to 1c's M3 (four coefficients solved as fractions), and S2, S2h (horse and engine); their numbers at 70 digits (`steam.py`), and the unchanged Rust oracle on them within 3.9e-16 |
| GOODS-CHAIN §1, §2, §7 D-G10 | 1c checks productivity per machine built, ρ(A^op + A^I) < 1, which rejects at weekly periods a machine whose build embodies more than about a period of its own chain's services, though A^op + ΔA^I is productive: A0 (a_I ≈ 148) and CHAIN's horse (3.3 horse-days per unit of weekly capacity); D-G10 checks productivity and the chain to land on the per-period matrix, "a pure relaxation: results are unchanged" |
| GOODS-CHAIN §2 (gates) | nesting bit for bit on 1c's instances, draws and rows; `generate_1g.py` at 70 digits importing generate_1c.py, with S1, S2, S2h, and A0 at 52 ticks a year and CHAIN's horse at weekly periods as D-G10's validation goldens; identities over every row; 1e-12 relative |
| GOODS-CHAIN §7 D-G1 | decision 67 reworded: machines are durable goods built from and run on goods; machine recipes use materials, machine goods, machine-hours, labour and non-produced inputs, never a category (E1) |
| AGENTS-GOODS §1 | A0: SSRN Appendix B's flow machine (a, λ, b) = (0.3, 0.05, 0.4) as a durable good built from a/δ of its own hours, λ/δ labour and (1 − ω)b/δ land, run on ω·b of land an hour as fodder; at ρ = 0 the equilibrium is Appendix B's |
| CHAIN §1.3-1.4 | the horse: fodder (farmland 1.0 acre-yr, 8 pd, 2 horse-days a ton); a head reared from 8 t of fodder, 3 acre-yr of pasture and 20 pd; 250 horse-days a year; a horse-day from 0.0176 t of fodder and 0.1 pd; δ 8% a year, J 3 years |
| CAPACITY.md, cap.py | the plant: y = K^(1−θ)·z^θ; its long run at ρ = 0 is unit 1c's price row p = O + uV with the operating recipe ζ times the bundle and the build recipe κ times the plant's recipe; the bundle plant's size s1 = θ^(θ/(1−θ))(1 − θ)/δ at which the long run is the registered instance; any other recipe a fixed point in κ/ζ (check 2c: L2 with a plant of labour 0.5 and land 0.5, v 0.13869); "D-G10, only if these rows are solved as 1c machines" |
| STATE decisions 63, 67, 71, 140; O28 | bitwise nesting; 67's wording; 71's rule, which D-G10 amends; machine recipes on pool labour; the mutants the re-checks of 1d-1f left |

### 0.1 There is no chain instance at the pin

laborformal's machine sector is main.tex:688's services form. Every chain here is constructed:
S1, S2 and S2h by ORACLE-GOODS, A0 by AGENTS-GOODS, the horse by CHAIN, the plants by CAPACITY.
They are illustrative (R5), never scored.

## 1. Scope

**In scope:**
- D-G10: productivity and the chain to land checked on the per-period recipes, in 1c's
  `MachineBlock::new`, and so in 1d, 1e and 1f, which validate through it;
- the mapping from a chain of goods (categories, materials, machines with a durable good and its
  hours) to unit 1c's parameters, with E1 and E2 enforced, and the equilibrium read back per good:
  each material's price and output, each machine good's price, output and stock, each machine's
  hour price, operating cost, capacity cost, hours, wealth and interest;
- plants as machine types: a plant on a flow type rewritten to its long run, the bundle plant in
  closed form (with s1's identity), any other plant recipe as a fixed point;
- goldens at 70 digits (S1, S1 at ρ 0, S2, S2h, A0 at 52 ticks a year with and without interest
  and in the fork economy, CHAIN's horse at weekly periods, L2 with plants), nesting with 1a, 1b
  and 1c in the generator and with 1d, 1e and 1f bit for bit in the gate;
- tests for O28's surviving mutants where a test is cheap.

**Out of scope** (§9 says where each goes): categories that buy machine-side goods directly
(GOODS-CHAIN's G1; E2 is enforced instead); several non-produced inputs (G2); several recipes for
one good (G3); retirement by lot life (G4); machine goods made on the task line (G5, E1 enforced);
a machine good per task segment (O3); the spatial equilibrium (O6); materials held at interest
(ORACLE-GOODS §3.3's carry, a build-only type); the tape's schema for goods and the mapping's
call where tapes are built (worldgen); the agents' stock rules (GOODS-CHAIN §3, the engine).

## 2. Decisions

### 2.1 D-G10: productivity and the chain to land per period

1c validates (unit-1c.md §3.2; decision 71):
- productivity: I − (A^op + A^I) a nonsingular M-matrix, ρ(A^op + A^I) < 1;
- the chain to land: (I − (A^op + A^I))⁻¹(b^op + b^I) > 0 in every entry.

A^op + A^I asks every machine to be buildable from one period of its own chain's services. At
yearly periods a durable machine passes; at weekly periods one whose build embodies more than
about a week of its own chain's services fails, though nothing in the economy is unproductive:
A0 at 52 ticks a year has a^I = a/δ = 148, ρ(A^op + A^I) = 148, and CHAIN's horse, through the 8 t
of fodder in its build, 1.50. The rate at which a machine's build recurs is δ a period, and what
the economy must reproduce each period is A^q = A^op + ΔA^I, the clearing side's matrix (1c §4.5):
0.3 for A0, 0.24 for the horse's chain.

**Decision (proposed 180, amending 71):**
- **productivity**: I − A^q a nonsingular M-matrix, with A^q = A^op + ΔA^I, every pivot of
  Gaussian elimination without pivoting in index order positive (1c's `Factors`);
- **the chain to land**: every type's recipes use land, or use, to operate or to build, the
  service of a type whose chain reaches land. A pattern condition on A^op, A^I, b^op and b^I,
  checked by reachability, not by a solve.

**It is a pure relaxation.** In exact arithmetic ΔA^I ≤ A^I (δ ≤ 1), so ρ(A^q) ≤ ρ(A^op + A^I).
In floating point 1c already factored I − A^q and refused the economy when a pivot was not
positive (`machine_block.rs`, "a rounding that says otherwise is reported as the same failure");
the new productivity rule is that check alone. The chain to land was (I − P)⁻¹(b^op + b^I) > 0
for P = A^op + A^I; on a nonsingular M-matrix, forward and back substitution add only nonnegative
terms, so a zero of the pattern is a zero of the solve, and a positive solve implies the pattern.
Every economy 1c accepts is accepted, with the same error on each it refuses for another reason.

**Results are unchanged, bit for bit.** Only validation changes: every quantity computed after
it (u, ω, Â, the price totals, A^q, its factors and the clearing totals) is computed as before,
in the same order. So every 1a-1f golden, draw, regime row and nesting test holds as it was.

**Why a pattern for the chain to land.** In exact arithmetic (I − A^q)⁻¹b^q > 0 and
(I − P)⁻¹(b^op + b^I) > 0 are both the pattern condition, since every δ is positive. In
floating point a clearing-side solve can underflow where the physical one did not (δ·a^I far
below 1e-300 along the only path to land), so "b̃^q > 0 in f64" would refuse some economies 1c
accepts. The pattern refuses none of them.

**The price side is not checked, as before.** At ρ > 0, u > δ, and I − Â can fail where I − A^q
passes: the economy is then valid and `NotViable` (1c §3.2, decision 72). A machine whose build
embodies much of its own service becomes `NotViable` once ρ/δ is large: A0's u·a^I = a(1 + ρ/δ),
0.44 at 5% a year and weekly ticks (SSRN A.4's "finite iff").

### 2.2 Decision 67 reworded (D-G1), and E1 and E2 enforced

Decision 67 ("machine recipes use machine services, labour and land only") protected E1: no
machine-side recipe uses a category, so A is block triangular and the machine side's totals do
not depend on x or on the technique (1c §2.3). Machines built from goods keep E1 when those goods
are materials and machine goods made by fixed recipes (ORACLE-GOODS §1.3). **Decision (proposed
179):** 67 reworded as D-G1: machines are durable goods built from and run on goods; machine
recipes use materials, machine goods, machine hours, labour and land, never a category. The
mapping enforces it: a material's recipe, a machine good's build recipe or a machine's operating
recipe that names a category is refused (`ChainError::CategoryInMachineRecipe`).

E2, that categories use materials and hours only through tasks, is enforced too: a category
whose inputs name a material, a machine good or hours is refused (`ChainError::DirectUse`), since
G1 (a category × machine-side matrix: hearth coal) is not built. Categories' inputs are other
categories, 1c's `intermediate`.

Machine recipes stay on pool labour (decision 140): a chain's labour is 1d's pool labour.

### 2.3 The mapping is in the oracle, per period

GOODS-CHAIN put the mapping "where tapes are built" (worldgen). **Decision (proposed 181):** the
mapping is the oracle's (`src/goods.rs`), from its own chain type, `GoodsChain`, keyed by strings:
a pure function of parameters to parameters that worldgen, the probe's tape generator or the lab
can call. It is per period: δ, ρ and J are per period, and a tape's yearly values are converted by
the tape builder with core's `Clock::fraction`, `Clock::compound` and `Clock::ticks`, as the engine
converts them (A0, the horse and the plants here are per week, decision 121's tick). The oracle
does not read a tape (R13 is unaffected: nothing on the engine path depends on the oracle).
Alternative: the mapping in worldgen, on the tape's schema, which does not exist yet (O32).

### 2.4 The embedding keeps every good's row

**Decision (proposed 182):** the chain maps to the embedding (ORACLE-GOODS §1.3, form b), not
to the fold (form c): each material and each machine good is its own flow type, so the
equilibrium gives every good's price and output, which are what Phase 2's markets are compared
against (ORACLE-GOODS §2.1's table). The order of the 1c types is fixed: the materials in the
chain's order, then the machine goods in the machines' order, then each machine's hours in the
same order. The fold, one type per machine, is checked against the embedding in the generator and
in the gate (the five numbers).

### 2.5 A machine good's unit is its stock

A machine good is counted in units of stock (a head, an engine). **Decision (proposed 183):**
each machine names κ, its hours a period per unit of stock (250/52 horse-days a week for a head),
and its hours type is built from 1/κ unit of its good per unit of capacity, so that 1c's unit of
capacity is one hour a period (1c §2.1). The good's price is per unit of stock, and the hours
type's V, per unit of capacity, is that price over κ; the stock installed is X/κ and the goods
made a period are δX/κ. κ = 1 is the embedding of ORACLE-GOODS §2.1 as written.

### 2.6 Plants as machine types

CAPACITY.md's plant desk makes y = K^(1−θ)·z^θ from its recipe's bundle z and a plant K built
from a recipe R, worn at δ. At its long run the plant is at the cost-minimising size for its
output, and the desk's price row is 1c's p = O + uV with, per unit of capacity (one unit of
service a period), the operating recipe ζ times the bundle and the build recipe κ times R, where
ζ^θ·κ^(1−θ) = 1 and κ/ζ = (1 − θ)·c_f/(θ·u·P_K) (c_f the bundle's cost, P_K the plant's). So a
plant is a rewrite of a flow type. **Decision (proposed 184):** a plant applies to a flow type
(a zero build recipe: the type holds no stock of its own), with its bundle share θ in (0, 1], its
δ and J per period, and a recipe: the bundle (s bundles of the type's own recipe per plant unit)
or a fixed recipe over machine services, labour and land. At any ρ, with the plant's user cost
u = (ρ + δ)(1 + ρ)^(J−1). θ = 1 is no plant: the type is left as it is, bit for bit.

**The bundle plant is closed form.** With R = s times the bundle, P_K = s·c_f and κ/ζ =
(1 − θ)/(θ·u·s), free of prices: one 1c economy. At ρ = 0 and s = s1 = θ^(θ/(1−θ))(1 − θ)/δ,
ζ = θ and κ·s1 = (1 − θ)/δ, so A^op + ΔA^I, the price side and the clearing side are the flow
recipe's: the long run is the registered flow economy exactly (CAPACITY's "at s1 exactly the
registered instance"). At another s it is the flow economy with the planted recipes times
m = θ^(−θ)(1 − θ)^(−(1−θ))(δ·s)^(1−θ).

**Any other recipe is a fixed point** (decision proposed 185). κ/ζ depends on the equilibrium's
prices, so the long run is a 1c economy solved as a fixed point in the ratios: start at the
unplanted economy's prices, solve the planted economy at the ratios, recompute them, and move
each ln(κ/ζ) half way to its new value (cap.py's `solve_cap`), until the largest change is below
`PLANT_TOL` = 1e-13, within `MAX_PLANT_STEPS` = 200 steps. Every step must be interior (its prices
are an equilibrium's); a step that is not refuses the plant (`PlantError::NotInterior`), and so
does a cap reached (`PlantError::NoFixedPoint`). Uniqueness of the fixed point is not proved
(O34). Alternative: Newton's method in ln(κ/ζ), which the generator uses at 70 digits.

**D-G10 is what admits it.** The bundle plant's build recipe is (1 − θ)/δ times its own bundle,
99 bundles at θ 0.8 and 10% a year per week, whose own service makes ρ(A^op + A^I) far above 1
for any desk that uses its own service; ρ(A^q) is the flow recipe's.

### 2.7 Nesting

1g adds no solve, so its nesting is D-G10's: every economy 1a-1f accept they still accept, with
every output bit for bit (§2.1), which the unchanged 1a-1f gate shows. A chain is a
reparameterisation, not an extension, so its check is equality: in the generator the direct
system, the embedding, the fold and the earlier unit where the chain is one of its economies
(S1 and 1c's M3; A0 and 1a's Appendix B; A0 in the fork economy and 1b's C3; the plants at s1 and
the flow L2) agree to 1e-65; in the gate the oracle's f64 values agree with the goldens to 1e-12,
and 1g's economies solve bit for bit alike through 1d's, 1e's and 1f's forms (§8, h7).

## 3. Parameters, validation and instances

### 3.1 Parameters

A chain, `GoodsChain<S>`:

| field | meaning |
|---|---|
| `workers`, `land`, `schedule`, `work_cost`, `rho`, `edges` | 1c's (`MachineParams`) |
| `categories: Vec<ChainCategory>` | `key`, 1b's `Category` (weight, direct land, densities), `inputs: Vec<(String, f64)>`, categories used per unit |
| `materials: Vec<Material>` | `key`, `recipe: GoodsRecipe`: per unit made |
| `machines: Vec<Machine>` | `key` (the good), `build: GoodsRecipe` per unit of stock, `hours` (the service's key), `hours_per_period` κ, `task_efficiency` θ, `operating: GoodsRecipe` per hour, `delta`, `build_lag` |

`GoodsRecipe { inputs: Vec<(String, f64)>, labor, land }`: goods by key and units per unit made.

A plant, `Plant { bundle_share: θ, delta, build_lag, recipe }`, with `PlantRecipe::Bundle { size }`
(s bundles of the type's own recipe per plant unit) or `PlantRecipe::Fixed(Recipe)` (1c's recipe
over machine services, labour and land per plant unit), applied to a type of a `MachineParams` by
`PlantEconomy::new(params, plants)`, the plants as `(type index, Plant)`.

### 3.2 Validation

A chain, in order:
1. every key once, over categories, materials, machine goods and hours;
2. in each recipe, each input once and a known key;
3. E1: no material, build or operating recipe names a category;
4. E2: every category input names a category;
5. each machine's κ scale (in [`SCALE_FLOOR`, `SCALE_CEIL`]);
6. then 1c's validation of the mapped parameters (1c §3.2 with §2.1's rule), its errors named by
   the good: `ChainError::Param { good, error }`, with `good` the key of the machine type or
   category the error's item names, `None` for the economy's own parameters.

A plant, in order: its type index in range and not planted twice; the type a flow type (a zero
build recipe); θ in (0, 1]; δ in [`SCALE_FLOOR`, 1]; J ≥ 1 and a finite u; a bundle's size scale;
a fixed recipe's machines one entry per type in [0, `SCALE_CEIL`], labour in [0, `SCALE_CEIL`],
land 0 or scale, and not all zero. The unplanted economy must be valid (`MachineEconomy::new`);
errors are `ParamError::Item { kind: "plant", index, .. }` by the plant's position.

### 3.3 Instances

**S1, the steam chain** (ORACLE-GOODS §1.6). M3's household and line (G1's N 4, T 10, h 1, χ_max 1;
the good and space; γ = 1 + 2x), ρ 0.05. Coal (a ton): 0.1 engine-hour, 0.2 labour, seams; iron
(a ton): 0.5 t coal, 0.5 labour, 0.1 ore; the engine good (one unit of capacity, κ 1): 0.1 t iron,
labour, site; an engine-hour: 0.4 t coal, labour, and the engine's user cost; θ 1, δ 0.1, J 3. Four
coefficients are solved exactly so that the four totals per engine-hour are M3's: engine-hour
labour 855257/7735400, engine-good labour 14560827/37903460, seams 14836821/15470800, site
186348663/3032276800. One land holds seams, ore, site and space. **S1Z**: S1 at ρ 0, whose totals
are M3z's.

**S2, horse and engine** (ORACLE-GOODS §1.6). S1's engine (θ 1.3) beside a horse: fodder (0.5
meadow land, 0.3 labour a unit), the horse good (2 fodder, 1.5 labour a unit of capacity, κ 1),
a horse-hour (0.1 fodder, 0.15 labour, the user cost; θ 1, δ 0.08, J 4); γ = 0.5 + 1.5x, N 10,
ρ 0.05: the engine at the margin, the horse priced and unused. **S2H**: θ 1.2, γ = 0.5 + x, N 16:
the horse at the margin.

**A0** (AGENTS-GOODS §1). Appendix B's household and line (N 4, T 10, h 1, γ = 0.2 + 0.8x, χ_max 1)
at 52 ticks a year: fodder a material of ω·b = 0.2 land; the horse built from a/δ = 148.2 of its
own hours, λ/δ labour and (1 − ω)b/δ land (κ 1); a horse-day from one unit of fodder; θ 1,
δ = 1 − 0.9^(1/52) a tick (10% a year), J 1, ω 1/2, ρ 0. Also as one 1c type, operating
(0; 0; ω·b) and build (a/δ; λ/δ; (1 − ω)b/δ). **A0R**: ρ = 1.05^(1/52) − 1 a tick. **A0F**: A0's
machine as one type in 1b's fork economy C3 (N 4, T 10, edges (0, 0.4, 0.75, 1), γ = 0.2 + 0.8x),
ρ 0.

**HORSE, CHAIN's horse at weekly periods** (CHAIN §1.3-1.4). Fodder (a ton): 1.0 land, 8 labour, 2
horse-days; the horse (a head): 8 t fodder, 20 labour, 3 land, κ = 250/52 horse-days a week; a
horse-day: 0.0176 t fodder, 0.1 labour, the user cost; θ 1, δ = 1 − 0.92^(1/52), J 156 ticks
(three years by `Clock::ticks`), ρ 0. Farmland and pasture are one land. The county is
constructed (the chain gives none; R5): the good and space on Appendix B's line with N 12, T 10,
h 1, χ_max 1/20, which puts the horse at the margin at x* 0.62.

**L2 and its plants** (the markets probe's L2; CAPACITY.md). L2: Appendix B's household with M4's
engine (θ 2; engine 0.1, power 0.5, labour 0.12, land 0.4 a unit) and power (θ 0; engine 0.1,
labour 0.12, land 0.6) as flow types, a loop. **P1**: a bundle plant on both at θ 0.8,
δ = 1 − 0.9^(1/52), J 1, ρ 0, s = s1 = 40.47. **P1S**: the same at s = 1. **P1R**: P1 at
ρ = 1.05^(1/52) − 1. **P2**: a fixed plant of labour 0.5 and land 0.5 a unit on both, ρ 0.

## 4. Equations

Per period, r = 1, v = w/r. Categories C, materials F, machine goods K, hours S; τ the
technique.

### 4.1 The chain

Price side (SSRN A.1, A.4):

    p_c = A_cc p_c + v·H(x) + (p_τ/θ_τ)·M(x) + b_c                 categories
    p_f = A_fG p_G + A_fS p_S + λ_f v + b_f                         materials (G = F ∪ K)
    p_k = A_kG p_G + A_kS p_S + λ_k v + b_k                         machine goods, per unit of stock
    p_s = A_sG p_G + A_sS p_S + λ_s v + b_s + (u_m/κ_m)·p_k(m)       hours of machine m
    v   = γ(x)·p_τ/θ_τ                                              the task margin

with u_m = (ρ + δ_m)(1 + ρ)^(J_m − 1). Clearing side: the same matrix with δ_m/κ_m in place of
u_m/κ_m, y = A^qᵀy + (Y·z; 0), N_a = λᵀy, T = bᵀy. Interest is Σ_m (u_m − δ_m)(p_k(m)/κ_m)X_m
(check_dynamics L1-L2). E1 makes A block triangular with the machine side upstream: the machine
side's totals are x-free.

### 4.2 The embedding (the mapping)

1c types in the order materials, machine goods, hours:
- material f: θ 0, operating = its recipe written over the types, build 0, δ 1, J 1;
- machine good k: θ 0, operating = its build recipe written over the types, build 0, δ 1, J 1;
- hours of machine m: θ_m, operating = its operating recipe over the types, build = 1/κ_m of
  its good (one entry), δ_m, J_m.

A flow type's price is its unit cost and its services its gross output (1c: p = O + u·0); the
hours type's V is p_k/κ and its builds δX. Categories: 1c's categories, `intermediate` from their
inputs. This is §4.1 row by row (ORACLE-GOODS §1.3), so the chain's equilibrium is the 1c
economy's, every good's price and output included.

### 4.3 The fold

Φ = (I − A_GG)⁻¹ over the goods G = F ∪ K. For machine m, with its direct goods g and hours h:

    a^op_m = A_mS + A_mG Φ A_GS,   λ^op_m = λ_m + A_mG Φ λ_G,   b^op_m = b_m + A_mG Φ b_G
    a^I_m  = (1/κ_m)(Φ A_GS)_k(m),  λ^I_m = (1/κ_m)(Φ λ_G)_k(m),  b^I_m = (1/κ_m)(Φ b_G)_k(m)

One 1c type per machine, the same δ and J. It is the chain price for price and quantity for
quantity on the hours, since the goods carry no user cost (A_GG is the same on both sides). The
five numbers per task type (θ, λ̃, b̃, λ̃^q, b̃^q) are the fold's, the embedding's and the direct
system's alike.

### 4.4 Readouts

Per material f: price p_f, output X_f (a period). Per machine m, with good k and hours s: the
good's price p_k per unit of stock (= κ·V_s), goods made δ_m X_s/κ_m (the good's output X_k when
nothing else uses it), stock installed X_s/κ_m, the hour price p_s = O_s + u_s V_s, O_s, V_s,
hours X_s, u_s, wealth W_s = ω_s V_s X_s and interest ρW_s. Per category: 1c's.

### 4.5 Plants

A plant on flow type k with bundle (a_k, λ_k, b_k), share θ, δ, J and recipe R per plant unit:

    u     = (ρ + δ)(1 + ρ)^(J − 1)
    r     = κ/ζ = (1 − θ)·c_f/(θ·u·P_K),   c_f = a_k·p + λ_k v + b_k,   P_K = R_a·p + R_λ v + R_b
    ζ     = r^(θ−1),  κ = r^θ          (ζ^θ κ^(1−θ) = 1)
    type k at the long run: operating ζ·(a_k, λ_k, b_k), build κ·R, δ, J

At the fixed point the price row gives O = θ·p (the bundle is θ of the price), u·V = (1 − θ)·p
(the plant's user cost is 1 − θ of it), and marginal cost c_f·ζ/θ equals the price. The plant
stock is K* = κ·X, bundles ζ·X a period, and the plant's price per unit P_K = V/κ.

Bundle plant: R = s·(a_k, λ_k, b_k), P_K = s·c_f, r = (1 − θ)/(θ·u·s). At ρ = 0 (u = δ) and
s = s1 = θ^(θ/(1−θ))(1 − θ)/δ: r = θ^(−1/(1−θ)), ζ = θ, κ = θ^(−θ/(1−θ)) (2.4414 at θ 0.8), and the
planted row per period is θ(a, λ, b) + δ·κ·s1·(a, λ, b) = (a, λ, b): the flow type. At another s
it is m·(a, λ, b) with m = θ^(−θ)(1 − θ)^(−(1−θ))(δs)^(1−θ).

### 4.6 D-G10's condition

    productive:      I − (A^op + ΔA^I) a nonsingular M-matrix
    chain to land:   reach_k ⇔ b^op_k + b^I_k > 0, or a^op_kl + a^I_kl > 0 and reach_l for some l

## 5. Solution method

### 5.1 D-G10 in `MachineBlock::new`

After each type's own checks and "at least one task type": factor I − A^q (1c's clearing matrix,
diagonal fma(−δ_k, a^I_kk, 1 − a^op_kk)) and its transpose, and refuse the block as "not
productive" if either has a pivot that is not positive; then the reachability of §4.6 by repeated
passes over the types in index order until nothing changes, and refuse the lowest type that does
not reach land. The physical factorisation and its solve are gone. Everything after is as before.

### 5.2 The mapping

`GoodsChain::to_machine_params` checks §3.2's items 1-5, builds the key → type index map in §4.2's
order, writes each recipe's inputs as a vector over the types (a missing key 0.0), each hours
type's build as 1/κ (one division) at its good's index, and the categories' `intermediate`
from their inputs. `ChainEconomy::new` validates the parameters with `MachineEconomy::new` and
names its errors by good. `ChainEconomy::solve` is 1c's solve, its interior equilibrium read per
good (§4.4); a boundary regime or an error is 1c's.

### 5.3 Plants

`PlantEconomy::new` validates (§3.2). `long_run(ratios)` writes each planted type's recipes
(§4.5), with ζ and κ by `num::pow`. `solve`:
- only bundle plants: the ratios in closed form, one 1c solve, its regime returned;
- otherwise: the unplanted economy's interior equilibrium gives the first ratios; each step
  builds and solves the long run, which must be interior, recomputes the ratios from its prices
  and moves each ln r half way; it stops when the largest |ln r_new − ln r| ≤ `PLANT_TOL`, and
  returns that step's equilibrium, its ratios, the step count and the gap.

The readouts of §4.5 come with every interior long run.

### 5.4 Precision

- **Weekly periods.** A0's V = 202.7 and a^I = 148 make the (O, V) system's V row large, but
  its pivot is 1 − u·a^I = 0.7: well conditioned. The price p = O + uV adds two positive terms.
  No new cancellation.
- **The mapping** adds one division per machine (1/κ) and nothing else: the f64 economy is 1c's
  on the rounded recipes.
- **Plants.** ζ and κ carry pow's error (a few ulps); the fixed point's gap bounds the ratio's
  error to about `PLANT_TOL` times the map's contraction, which the build measures (§12).
- **Goldens** are compared at 1e-12 relative (ADDENDUM A7). δ per tick is the generator's
  1 − 0.9^(1/52) at 70 digits against the oracle's `Clock::fraction` double: the golden
  `A0_DELTA` checks the double to 1e-15.

## 6. Result types

- `src/machine_block.rs`: D-G10 (no new type).
- `src/goods.rs`: `GoodsRecipe`, `ChainCategory`, `Material`, `Machine`, `GoodsChain<S>` with
  `to_machine_params`; `ChainEconomy<S>` with `new`, `economy`, `rows`, `row(key)`, `solve`,
  `readout(&Eq1c)`; `Row::{Category(j), Material(k), MachineGood(k), Hours(k)}` (1c indices);
  `ChainEq { eq: Box<Eq1c>, goods: Vec<GoodEq>, machines: Vec<MachineEq> }`; `ChainError`.
- `src/plants.rs`: `Plant`, `PlantRecipe`, `PlantEconomy<S>` with `new`, `base`, `plants`,
  `long_run`, `ratios_at`, `solve`, `solve_within(steps)`; `PlantEq { eq, ratios, plants:
  Vec<PlantReadout>, steps, gap }`, `PlantReadout`, `PlantError`; `s1_size`, `PLANT_TOL`,
  `MAX_PLANT_STEPS`.

## 7. Goldens

`goldens/generate_1g.py` computes every 1g golden with mpmath at 70 digits. Its `Economy` is
generate_1c.py's with D-G10's validation in place of 1c §3.2's (its constructor copied line for
line but for that rule; every other method inherited), and the generator asserts that the two
constructors give the same object and the same solve on M3, M3z, M4 (η 1, 0.5) and M5 (ρ 0.1). Each
chain is solved directly (§4.1: the price system with the margin as one more row, bisection to
2^-250), by its embedding (§4.2, 1c's solve) and by its fold (§4.3), which agree to 1e-65 on every
aggregate, and the direct system and the embedding on every good's price and output, and the
five numbers of each hours type. The plants' fixed point is found by Newton's method in ln(κ/ζ)
after damped steps, with each economy's root by bisection and the Illinois method, and then
checked by 1c's full solve. It writes `goldens/goldens_1g.txt` with 30 significant digits and a
header of FNV-1a digests of itself, generate_1c.py, generate_1b.py, generate.py and the body; the
Rust constants in `tests/gate/goldens_1g.rs` carry 20 digits (h9).

The generator asserts, at 1e-65:
- D-G10 changes no 1c instance (above);
- S1 = M3 and S1Z = M3z on every aggregate, and S1's engine-hour price and V·X are M3's;
- S2's and S2H's switch (1c's closed form) is the same in the embedding, the fold and the
  representative economy of ORACLE-GOODS §1.4(c); at ρ 0 both have the horse on the whole line;
- A0 = Appendix B (generate.py's G1) on x*, 1 − x*, v, P_s, Y, N_a, income and interest, its
  horse-day price = G1's p_m and hours = G1's K; A0's one-type form = its chain; A0R's one-type
  form = its chain; A0F = 1b's C3 (generate_1b.py);
- the per-period spectral radius below 1 and the physical above 1 for A0 (one type and chain)
  and the horse, A0's per-period radius exactly a = 0.3;
- P1 = L2's flow economy (ζ = θ, κ·X = θ^(−θ/(1−θ))·X); P1S = the flow economy scaled by m; every
  plant's O = θp and uV = (1 − θ)p; P2 at a fixed point (gap below 1e-65) and within CAPACITY's
  printed digits (v 0.13869; ζ/κ 0.3783/48.8149 and 0.3723/52.0700).

The draft's values, to 20 significant digits (the goldens file has 30):

**D-G10's radii.** A0: physical 148.21310731815708760, per period 0.3. A0's chain: 12.174280566758640609
and 0.54772255750516611346. The horse's chain: 1.5008632641098051053 and 0.23968076641154208475.
S1's chain: 0.30575266521078003997 and 0.22360679774997896964. δ a tick: 0.0020241124785003951700
(10% a year), 0.0016022075724025086666 (8%); ρ a tick 0.00093871270311172379084 (5%).

**S1** (ρ 0.05): x* 0.91057468799335480604, v 5.3088781572501366285, P_s 4.7485791815332800258,
Y 5.8234637271311258276, N_a 3.0018757179079080790, income 27.653178619069065849, interest
1.7165861894881998391. Coal 2.2089779528867596331 a ton, 4.2763280346654769631 t; iron
3.8589280550684481308, 0.10558834653495004847 t; the engine 2.4867831912209203928 a unit,
1.0558834653495004847 made, 10.558834653495004847 installed; an engine-hour 1.8818139168520038924
(O 1.4705621466038441825), 10.558834653495004847 hours. The fold: operating (0.04;
0.19056403030224681335; 0.38360837190061276728), build (0.005; 0.44415561534487880526;
0.11940607691223967416). **S1Z**: coal 1.9034918179819222748, the engine-hour 1.3976046053422440254
(M3z's p_m).

**S2** (the engine at the margin): switch γ 1.0422282165706983922, x 0.36148547771379892814; x*
0.66836997221771948458, v 0.71282600394783768684, Y 8.2493376387291514382, N_a
3.7757255126562787945; the horse-hour 0.55407549108050234304, fodder 0.71384780118435130605, both
unused. **S2H** (the horse at the margin): switch x 0.67292051156339654511; x*
0.64555993278226703026, v 0.53194257753558310662; 4.9684653111078042529 horses at
2.1170794128247245239.

**A0** (ρ 0): Appendix B's x* 0.86315041816243703192 and v; the horse 202.67916438517006167 a head,
5.2984861875677308081 installed, 0.010724732009417829295 reared a week; fodder 0.2.
**A0R** (5% a year): x* 0.81738052839262692215, v 0.84427235530539978830, Y 8.0248707640064784596,
N_a 1.7123888131395095847, interest 1.2339143960604518983; the horse 266.20536878386680905.
**A0F**: 1b's C3, the horse's V 199.90388517634383393.

**HORSE**: x* 0.62424404534133174456, v 0.019278964656802269677, P_s 1.0149823159707379272,
Y 9.9381651174203159807, N_a 4.5158984898161836425; fodder 1.2093621032133959202 a ton, a head
13.060476118843212755, 0.60480324166369828284 heads, a horse-day 0.027565192979488881377;
λ̃ 0.28014880311915418251 and b̃ 0.022164214105509250446 a horse-day (CHAIN's 0.268 and 0.021).

**L2 and its plants**: L2's v 0.42570092667810366689, x* 0.91480039929112051662; s1
40.472059171678095387; P1's κ 2.44140625 (0.8^(−4)), the engine's plant 6.1284434141399148904 at
36.978423737748777123 a unit. **P1S** (s = 1): m 0.47705553571010563622, v 0.13943139644608053975.
**P1R**: x* 0.90211329973015549186, v 0.48512753202383114607. **P2**: v 0.13868993703845577967,
x* 0.98268463578459683797; the engine's κ/ζ 129.03010594613441754 (ζ 0.37832216752801874966,
κ 48.814949357911473326), power's 139.87335085107186726 (ζ 0.37226568401874200253, κ
52.070048630567757493); Newton took 4 steps from the damped iteration's 1e-8.

## 8. Tests

The groups are the modules `h1_d_g10`, `h2_mapping`, `h3_two_machines`, `h4_a0`, `h5_horse`,
`h6_plants`, `h7_nesting`, `h8_o28` and `h9_goldens_file` of `tests/gate/` ("g" is 1a's), cited as
`h1::` to `h9::`, with `goldens_1g.rs` and `support_1g.rs`. Goldens and identities at 1e-12
relative; "bitwise" means `to_bits`.

**h1, D-G10.**
- `the_rule_is_on_the_per_period_matrix`: m7's loom with 0.6 of its own service to operate and 0.6
  to build at δ 0.1, refused by 1c, is accepted (A^q = 0.66) and solves with 1c's identities; the
  build at 4.1 (A^q = 1.01) is "not productive"; at δ 1 the 0.6 + 0.6 is. [§2.1]
- `a0_and_the_horse_were_refused_and_are_accepted`: for A0 (one type and chain) and the horse's
  chain, a test-local elimination of I − (A^op + A^I) has a nonpositive pivot, 1g accepts, and
  power iteration on I + A gives the goldens' radii. [§2.1; §7]
- `every_economy_accepted_before_is_accepted`: on random blocks (1c's m6 ranges, and δ small),
  1c's rule reimplemented in the test (physical elimination and solve) accepting implies 1g
  accepting; the draws accepted only now are counted, and those interior satisfy
  `check_identities_1c`. [§2.1]
- `the_chain_to_land_is_a_pattern`: land reached through a build recipe's machine input is
  enough; a landless loop is refused naming its lowest type; a path through δ·a^I below 1e-300,
  where a clearing-side solve underflows, is accepted. [§2.1, §4.6]
- `the_price_side_is_not_checked`: an A0 at a ρ/δ that makes u·a^I > 1 is valid and
  `NotViable`. [§2.1]

**h2, the mapping (S1, S1Z).**
- `s1_maps_to_its_embedding`: the types' order, θ, recipes, build 1/κ, δ, J, the categories'
  `intermediate`; `rows` and `row`. [§4.2]
- `s1_is_m3`: S1's aggregates are 1c's M3 goldens, S1Z's M3z's; every good's readout is S1's
  golden. [§7]
- `readouts`: the good's price κ·V, goods made δX/κ, the stock X/κ, p = O + uV, interest as
  Σ(u − δ)VX; the fold's golden recipes give S1's aggregates (the five numbers). [§4.3-4.4]
- `e1_and_e2_are_enforced`: a category in a material's, a build or an operating recipe;
  a material or hours among a category's inputs. [§2.2]
- `keys_and_errors`: a duplicate key, an unknown input, an input named twice, κ out of range, a
  1c error named by its good. [§3.2]
- `units_of_stock`: the horse measured per head (κ 250/52) and per unit of capacity (κ 1, build
  divided by κ) give the same equilibrium, and prices per head κ times. [§2.5]

**h3, two machines (S2, S2H).** Goldens; the switch in closed form; the unused machine priced
with its closure wage above v; at ρ 0 the horse on the whole line; only hours types do tasks.

**h4, A0.** `a0_is_appendix_b` (A0's chain and one type against 1a's G1 goldens and A0's);
`a0_with_interest` (A0R); `a0_in_the_fork_economy_is_c3` (1b's C3 goldens); `the_clock_gives_the_tick`
(`Clock::fraction`, `compound` and `ticks` against A0_DELTA, A0_RHO_TICK and J 156).

**h5, the horse.** Goldens; λ̃, b̃ and ω; D-G10 needed (physical radius 1.50).

**h6, plants.** `a_bundle_plant_at_s1_is_the_flow_economy` (P1 against L2 at 1e-12, the readouts,
O = θp, uV = (1 − θ)p, ζ = θ, κ = θ^(−θ/(1−θ))); `a_bundle_plant_at_another_size` (P1S);
`a_plant_with_interest` (P1R); `a_fixed_recipe_is_a_fixed_point` (P2's goldens, gap, steps);
`no_plant_is_the_type_bit_for_bit` (θ 1); `plants_need_d_g10`; `every_step_must_be_interior`;
`the_cap_refuses` (`solve_within(1)`); `validation`.

**h7, nesting.** S1, S2, A0, A0R, the horse and P1's long run through 1d's, 1e's and 1f's forms
(one worker type, parcel form, no government): every 1c output bit for bit in 1d, 1d's in 1e's
base, 1e's in 1f's base.

**h8, O28.** The cheap tests of §12's list.

**h9, the goldens file.** `goldens_1g.txt`'s five digests recompute, and the Rust constants match
it to 20 digits.

## 9. Out of scope, and where each goes

- G1 (categories buying machine-side goods), G2 with G3, G3, G4, G5, O3 and O6: GOODS-CHAIN §2's
  table, each when its instance needs it (D-G9); O31.
- The mapping's call where tapes are built, on the tape's goods schema: with the goods chain's
  engine work (STATE next step 6) or the demo's second pass (O27); O32.
- Materials held at interest (carry): ORACLE-GOODS §3.3, a build-only type, when a rule charges
  it.
- Plants in the agents (the engine's plant good, y = K^(1−θ)z^θ, the plan): STATE next step 6.

## 10. Pitfalls

- **Two matrices, one rule.** D-G10 uses A^q, the clearing side's; the price side's Â is still
  not validated, and at ρ > 0 can fail where A^q passes (`NotViable`).
- **θ twice.** A type's `task_efficiency` θ_k and a plant's `bundle_share` θ are different
  things; the plant keeps the type's task efficiency.
- **κ.** A machine good's price is per unit of stock; the hours type's V is per unit of capacity,
  price/κ.
- **The good's output is builds.** A machine good's flow-type output is the goods made a period
  (δX/κ), not the stock (X/κ).
- **A pattern is not a solve.** The chain to land is checked by reachability; b̃^q can still be
  tiny.
- **Plants are rewrites.** A plant replaces its type's recipes, δ and J; the unplanted type's
  output per unit of bundle is not the planted one's.

## 11. Open questions, proposed as decisions 179-190

As built, 1g takes the first choice on each, and STATE.md records each as a decision open to veto,
numbered from 179 (track 1g's range, 179-199; G1's is 200-219).

1. **D-G1: decision 67 reworded** (§2.2; proposed 179): machines are durable goods built from and
   run on goods, never from a category (E1). Alternative: G5, machines from categories.
2. **D-G10: productivity and the chain to land per period** (§2.1; proposed 180, amending 71),
   the chain to land as a pattern. Alternative: b̃^q > 0 by a solve, which can refuse a few
   economies 1c accepts.
3. **The mapping is the oracle's, per period** (§2.3; proposed 181). Alternative: in worldgen.
4. **The embedding keeps every good's row** (§2.4; proposed 182). Alternative: the fold.
5. **A machine good's unit is its stock, with κ hours a period** (§2.5; proposed 183).
6. **Plants on flow types, at any ρ, θ 1 as no plant** (§2.6; proposed 184).
7. **Plants other than the bundle by a damped fixed point, every step interior** (§2.6; proposed
   185). Alternative: Newton's method.
8. **E2 enforced; G1 not approximated** (§2.2; proposed 186). Alternative: G1 now.
9. **The illustrative counties of A0 and the horse** (§3.3; proposed 187): Appendix B's for A0 (by
   rule A), and a constructed one for the horse (N 12, T 10, h 1, χ_max 1/20), never scored.
10. **The weekly tick for the goldens** (§2.3; proposed 188): δ and ρ by `Clock`'s formulas at 52
    ticks a year, J in ticks.
11. **O28: which survivors get tests** (§8 h8; proposed 189): the cheap ones (§12), the rest
    carried.
12. **The test groups are h1-h9** (proposed 190), since "g" is 1a's.

## 12. Changes during the build (P1g.2-P1g.4), 2026-09-27

1. **D-G10 as drafted.** `MachineBlock::new` factors I − A^q and its transpose where it factored
   I − (A^op + A^I), refuses a nonpositive pivot as "not productive" (the message now names the
   per-period matrix), then checks the chain to land by reachability (`reaches_land`); the
   physical factorisation and its solve are gone, and everything after is computed as before.
   The only order that changed is between the two refusals: an economy refused on both counts is
   now refused as "not productive" first, as an economy 1c refused on productivity was. No
   economy 1c accepted changes, and the 1a-1f gate passes unchanged but for
   `m7::validation`'s two rows on A^op + A^I (§8): 0.6 + 0.6 of the loom's own service is now
   valid, and the rows refused are 0.6 + 4.1 (1.01 a period) and 12 to build (1.2). unit-1c.md
   §3.2 and §8 carry a note.
2. **The chain to land by reachability** needs more than one pass when a type reaches land
   through a higher-indexed landless type (`h1::the_chain_to_land_is_a_pattern` has one), and
   the lowest type that does not reach is named, as 1c named it.
3. **The mapping's API** is §6's: `GoodsChain::rows` and `to_machine_params`, `ChainEconomy`
   with `readout` on an `Eq1c` by value, `ChainEq::good(key)` and `machine(key)`. §3.2's checks
   1-5 run good by good in the mapping's order (the keys first; then each material's recipe,
   each machine good's, each machine's κ and operating recipe; then the categories' inputs), so
   the first error in that order is the one reported. A unit-1c error that names a machine type
   or a category is named by its good; the economy's own parameters by none.
4. **Plants.** `PlantEconomy::new` validates the unplanted economy first; θ takes δ's range,
   [SCALE_FLOOR, 1]. `ratios_at(&Eq1c)` gives the ratios an equilibrium's prices imply; a
   bundle plant's stays in closed form beside a fixed one (`h6::a_fixed_recipe_beside_a_bundle`).
   Each readout's capital share u·V/p is reported, 1 − θ at the long run.
5. **Numbers.** The generator writes 210 goldens in about 26 s and reproduces every number of
   §7; its three forms and the earlier units agree within 9.9e-71. The oracle's f64 values match
   the goldens within 4.5e-16 relative, but for S1Z's 1 − x* against M3z's golden (2.9e-15, its
   x* 0.94) and P2's, whose fixed point stops at `PLANT_TOL`: 42 steps, a last move of 9.1e-14
   in ln r, and the ratio, κ and the plant within 9.1e-14, 1 − x* within 3.7e-14. The radii by power
   iteration on I + A agree with mp.eig's to 1e-9, the test's tolerance. P1 matches L2's flow
   economy within 1e-13 (4e-16 measured).
6. **h1's draws**: of 3000 random blocks (1c's m6 ranges with machine inputs up to 3 and δ
   log-uniform down to 1e-3, ρ up to 0.002, seed 1979), 498 are accepted by both rules, 470 by
   D-G10 only, 279 of them interior with every 1c identity, 2032 by neither, and none by 1c
   only.
7. **The build's mutation check** (scratch `D:/rustyecon-og/mut/`, `mutate.py`): 37 mutants of
   1g's code, each applied alone and the package's tests run in release: 8 of D-G10 (1c's rule
   restored beside it, the chain to land by a clearing-side solve, build inputs, operating inputs
   or build land ignored, one pass over the types, the highest landless type named, no
   productivity check), 13 of the mapping (1/κ, the good's index, E1's and E2's errors, a repeated
   input accepted, the readouts, the categories' inputs transposed, a row index, an error named by
   another good, a machine good's δ) and 16 of the plants (the ratios, ζ's and κ's exponents,
   damping, the tolerance, θ 1, a durable type, the stock, P_K, u at ρ 0, the capital share, the
   closed form, s1, the base's validation, a recipe's cost). 36 are killed. The survivor adds an
   input's coefficient where the mapping sets it, which is equivalent, since an input named twice
   is refused before; a machine good's δ (a flow type's δ changes nothing) is killed only by the
   test of the mapped structure. The damping is guarded by `h6::the_cap_refuses`, which checks
   that each step halves the move; the tolerance by P2's goldens.
8. **O28** (§8 h8): eight tests. They port the 1d re-check's four probe tests (the other types'
   closure wages at a corner, Lemma B.1 at the basket's price with the reserved costs, the last
   piece started at the last wall switch's value, f_∞ = 0 after a positive start) with the
   assertions the probes printed; take 1e's frame probe with space's land at 2 and 0.5 and assert
   the wall's end's branch, q = 1/b̃_g and the junction; add a CES economy with an intermediate
   input and required hours (1f's required hours from gross outputs); and add two first-pass
   survivors of 1d: an economy with no worker types, and the jump schedule, which 1d must refuse
   as `LaborNotCleared` as 1a-1c do. The O28 mutants, run on this tree: the four 1d probe tests kill
   their mutants, and so do the 1e frame test (its price inverted, and 1; the second is now also
   killed by 1f's draws), the CES test, the jump test and, after P1g.4 made it assert its reason,
   the worker-type test: without its own check an economy with no worker types is still refused,
   by the next check, under the same name with the wrong reason. Lemma B.1's flag without its
   shortage check survives, and is equivalent: a short point's P_s is NaN, so its funding test
   is false either way. The rest are carried (STATE.md O33).
9. **Tests.** 46: 4 unit (`goods::tests`, `plants::tests`) and 42 gate, h1 5, h2 6, h3 3, h4 4,
   h5 2, h6 10, h7 2, h8 8 and h9 2. The package has 484: 88 unit, 395 gate and 1 doc, and the
   workspace 841.
10. **The generators.** All seven pass `--check` under laborformal's venv on Windows:
   `generate.py` 1 s, `generate_1b.py` 2 s, `generate_1c.py` 12 s, `generate_1d.py` 14 s,
   `generate_1e.py` 41 s, `generate_1f.py` 23 s, `generate_1g.py` 24 s.
