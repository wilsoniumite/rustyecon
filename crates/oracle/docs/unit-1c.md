# Unit 1c: many machine types, the Leontief inverse and the user cost per type

Dated 2026-09-27. This is the unit's specification, written before any code. A 70-digit
scratch prototype (not kept) checked every derivation below and supplied the numbers in
§7; `goldens/generate_1c.py` reproduces them. The build (P1.4, the same day) departed from
the draft where §12 says, and the text below is amended to match.

- **Pinned to:** laborformal `31b3482`. Every laborformal path and line below is at that
  commit.
- **Paper:** SSRN 7226858 (posted 2026-09-23), cited as "SSRN p.N (eq k)".
- **Older revision:** `pinning/paper/main.tex` (2026-09-21), cited as "main.tex:N". Its
  Appendix A states the machine sector's general form at :688 (many produced inputs) and
  :690 (durability). Its Appendix B is not used (unit-1a.md §0).
- **Reference checks:** `dynamics/checks/check_dynamics.py` U1-U6 (:55-90), R1-R6
  (:93-124), S1-S6 (:126-149), L1-L4 (:154-181), E1-E3 (:199-222), EJ1-EJ7 (:321-397) and
  the targets it writes (:480-500) to `dynamics/checks/dynamics_ss_targets.json`;
  `pinning/checks/check_pinning.py` A-joint (:225-267) and A-usercost (:269-296).
- **Gate (PLAN Phase 1, and the task of 2026-09-27):** check_dynamics' numeric targets
  where they are in this unit's closure (§0.2 says which); 1a and 1b as the one-type case,
  exactly; the income identity to 1e-12 with interest; random instances satisfying the
  Leontief identities.
- **Built on:** units 1a and 1b (docs/unit-1a.md, docs/unit-1b.md). Every 1a and 1b
  equation, convention, constant and precision rule carries over unless this document says
  otherwise.
- **Goldens:** a new generator, `goldens/generate_1c.py`, writing `goldens/goldens_1c.txt`
  (§7).

## 0. What the sources give

| source | what 1c takes from it |
|---|---|
| SSRN §3.2, p.8 (eq 2-4) | the unit-cost equation p_i = Σ_j a_ij p_j + λ_i w + b_i r; p = Ap + λw + br and p = (I − A)⁻¹(λw + br); "the spectral radius of A is below one"; the totals λ̃ = (I − A)⁻¹λ, b̃ = (I − A)⁻¹b; p_i = wλ̃_i + rb̃_i |
| SSRN §3.3, p.9 (Prop 2, eq 5-6) | the replacement closure p_m/r = b̃_m/(1 − γ(x*)λ̃_m), w/r = γ(x*)b̃_m/(1 − γ(x*)λ̃_m), under γ(x*) > 0, b̃_m > 0, 1 − γ(x*)λ̃_m > 0; (p_m/r)/b̃_m → 1 as γ(x*)λ̃_m → 0 |
| SSRN A.1, pp.24-25 | "with several machine types, the relevant alternative minimizes delivered task cost; an active margin satisfies w/γ_L(x*) = min_k p_mk/γ_Mk(x*)"; p = Ap + λw + Br, p = wλ̃ + B̃r, B̃ = (I − A)⁻¹B; recipes "nonnegative and productive, with spectral radius of A below one"; "unused feasible methods have unit costs at least as high as the equilibrium output price"; the closure against machine m, w = γ(x*)Σ_k B̃_mk r_k/(1 − γ(x*)λ̃_m); clearing f = (I − Aᵀ)y, N_a = λᵀy, T = Bᵀy; "unused fixed inputs have zero rent" |
| SSRN A.4, pp.27-28 | inputs advanced one period, p_m = (1 + ρ)(ap_m + λw + br); the durable asset V_m = ap_m + λw + br, p_m = (ρ + δ)V_m; both closed forms at the task margin, each with a positive denominator; "interest introduces an income component beyond the main two-factor flow accounting" |
| SSRN p.29, App. C p.30 | the three-row matrix display of Appendix B (good, machine services, space); I = pᵀf = pᵀ(I − Aᵀ)y = wλᵀy + rbᵀy |
| main.tex:688 | many produced inputs: c = Ac + Λw + Br, c = (I − A)⁻¹(Λw + Br) when the spectral radius of A is below one; "the task margin selects which machine type is the binding alternative at x*: the one minimizing delivered task cost" |
| main.tex:690 | the λ = 0 and λ > 0 closures with carrying factor 1 + ρ or ρ + δ, finite iff the factor times (a + λγ(x*)) is below 1; "the added term is interest, competitive and real" |
| check_dynamics U1-U6 | u_K = (ρ + δ)(1 + ρ)^(J−1) derived from free entry; its corners; the misread (ρ + δ)(1 + ρ)^J; the PV sum numerically at two points |
| check_dynamics R1-R6 | the two-recipe row c = ac + λw + br + u_K p_K, p_K = a_I c + λ_I w + b_I r, w = γ*c; θ_c = (b + u_K b_I)/Den with Den = 1 − a − λγ* − u_K(a_I + λ_Iγ*); θ_w = γ*θ_c; p_K/r = a_Iθ_c + λ_Iθ_w + b_I; the zero-build corner c = br/(1 − a − λγ*); the zero-operating corner c = u_K b'r/(1 − u_K(a' + λ'γ*)); the price block's determinant is Den |
| check_dynamics S1-S6 | signs of dθ_c/dγ*, dθ_w/dγ*, dθ_c/dλ, dθ_c/dλ_I, du_K/dJ, dθ_c/du_K, and the cross-effect at λ = 0 |
| check_dynamics L1-L4 | the steady-state ledger: machine cash u_K p_K K − p_K δK = ρW_K with W_K = p_K K[(1 + ρ)^(J−1) + δ((1 + ρ)^(J−1) − 1)/ρ]; zero NPV; W_K ≥ p_K K with equality iff J = 1 |
| dynamics_ss_targets.json | two one-type steady states, "flat" and "sloped", under a Cobb-Douglas land-share household (§0.2) |

### 0.1 There is no multi-type equilibrium instance at the pin

check_dynamics' steady states have one machine type. macro.py has one (unit-1a.md §0). The
many-type part of the gate is therefore identities on random instances and constructed
economies, as 1b's many-category part was.

### 0.2 Which dynamics targets are in 1c's closure

The targets (`dynamics_ss_targets.json`, written at check_dynamics.py:480-500) solve a
different household closure: full participation (N_a = N = 1), a Cobb-Douglas land share
α = 0.3 of income (check_dynamics.py:321-333, check_pinning.py A-joint :225-267),
external finance at ρ, and the good as numeraire. 1c's closure is Appendix B's: the
dependence exit, a fixed basket and a provider household (unit-1a.md §1). So:

| target | in 1c's closure? | how 1c reproduces it |
|---|---|---|
| sloped `c`, `w`, `r`, `pK`, `uK` | yes, as a price block | at the target's x* (taken as an input), 1c's machine row gives p_m = c/r, v = w/r, V = pK/r, and the good's price p = 1/r (the unit-cost normalisation p = 1, model.py:108) |
| sloped `Y`, `X` | yes, as clearing-side totals | with N_a = N = 1: Y = 1/(labour per unit of the good) and X/Y = machine services per unit of the good, both clearing-side totals at x* |
| sloped `xstar` | **no** | it is the root of the land-share closure; 1c takes it as an input. The generator recomputes it at 70 digits from check_dynamics' equations |
| flat `c`, `w`, `r`, `uK` | yes, as a price block at γ* = 3 | p_m = c/r, v = w/r |
| flat `X`, `Y` | yes, as clearing-side totals | at the target's machine-task share m (an input) |
| flat `m` | **no** | set by the land-share closure; an input |
| EJ4 (Walras with net exports), EJ5 (participation) | **no** | external finance and s(q) are not 1c's closure (1f, 1e) |
| EJ6 (c/r = θ_c at the root), EJ7 (zero build recipe gives A-joint's prices) | yes | R1 and R4 in the price block |

The sloped economy's schedule, γ = 1 + 4x, is not viable at x = 1 with this machine
(Den(γ(1) = 5) < 0; EJ1 notes that viability ends near x = 0.658), so under 1a's
convention it is `NotViable` as an equilibrium in any closure that requires viability on
the whole line. This is a second reason its targets are gated as a price block only.

## 1. Scope

**In scope:**
- K machine types, each with an operating recipe and a build recipe over machine services,
  labour and land, its own depreciation δ_k and build lag J_k, and so its own user cost
  u_k = (ρ + δ_k)(1 + ρ)^(J_k − 1) and wealth factor ω_k; one interest rate ρ;
- machine types that use each other's services (a K×K produced-input matrix per recipe);
- machine types that do tasks, with a task efficiency θ_k, and machine types that only
  supply other machines (θ_k = 0, "power");
- the choice of task technique: the type with the least delivered task cost, one threshold
  on the task line, technique switches along the line, and ties at a switch;
- categories that use other categories as intermediate inputs (a C×C matrix), 1b's
  categories otherwise;
- the full cost system over categories and machine types, its Leontief inverse and totals,
  the clearing system f = (I − Aᵀ)y, N_a = λᵀy, T = bᵀy, and the income identity with
  interest;
- the price block alone (the machine block at a given margin), for the dynamics targets.

**Out of scope** (§9 says where each goes): machine recipes that use categories (PLAN §3.1's
build bundle; open question 1); schedules of different shape per machine type; several
non-produced inputs, B as a matrix and unused fixed inputs at zero rent (1e); worker types
and the wall (1d); the land-share household and government (1f); cells in the equilibrium
(1b's open question 2); dynamics out of steady state (Phase 3).

## 2. Decisions

### 2.1 Goods, rows and units

The produced goods are the C categories (rows 0…C−1) and the K machine services (rows
C…C+K−1), in that order (1b §4.6). One unit of machine type k's capacity delivers one unit
of its service per period, so capacity and services are one number, X_k (check_dynamics:
K = X). There is one non-produced input, land, with r = 1, so every price is in land units
and v = w/r (1a). Space is a category (1b).

### 2.2 Two recipes per machine type

Type k's service costs its operating recipe, paid in the period, plus the user cost of the
capacity it occupies (check_dynamics R, :93-99; SSRN A.4):

    p_k = O_k + u_k·V_k
    O_k = Σ_l a^op_kl p_l + λ^op_k·v + b^op_k        operating cost per unit of service
    V_k = Σ_l a^I_kl p_l + λ^I_k·v + b^I_k           build cost per unit of capacity (p_K)

where l runs over machine types. 1a is the operating-free case (V is 1a's V_m). The
build-free case prices a machine service as SSRN's flow benchmark does, at every u
(check_dynamics R4). ρ is common to all types (check_dynamics' single world rate); δ_k and
J_k are per type.

### 2.3 Machine recipes use machine services only

1c's machine recipes use machine services, labour and land, not categories. This is
main.tex:688's form (c = Ac + Λw + Br over produced inputs) and check_dynamics' (the build
recipe uses machine services c, :93-96). It has three consequences the unit relies on:

1. **The machine totals do not depend on x.** λ̃_m = (I − Â)⁻¹λ̂ and b̃_m = (I − Â)⁻¹b̂ (§4.2)
   are constants of the economy, so each task type's closure wage v_k(x) = γ(x)b̃_k/(θ_k −
   γ(x)λ̃_k) is a function of γ(x) alone.
2. **The technique is a function of γ.** The cheapest type is the least v_k (§4.3), and two
   types' closure wages cross at most once in γ, so a type never returns once it has left
   (Lemma 1, §5.5). The switches are closed-form points in γ, computed once.
3. **The one-type proof of uniqueness carries over within a technique** (Lemma 2, §5.5).

PLAN §3.1 builds capital from goods ("the build bundle"). With categories in machine
recipes the totals depend on x and on the technique, the choice becomes a fixed point
across the category block, and Lemma 1 can fail. The cost-minimising prices still exist
(the non-substitution argument of §4.3 holds technique by technique), but the solver and
its proofs change. This is open question 1.

### 2.4 One task line, one shape, a task efficiency per type

Every task-doing type has the line's relative capability up to a constant: one hour of type
k's machine service completes θ_k/γ(x) units of task x, so γ_Mk(x) = θ_k/γ(x) with γ_L = 1
(SSRN A.1). The delivered cost of task x by type k is p_k·γ(x)/θ_k, against w for a
person. Because the shape is common, the cheapest type is the same at every task, and
people hold the tasks above one threshold x: 1a's and 1b's structure, with the same
unknown, bracket and precision. θ_k = 0 marks a type that does no tasks.

Schedules of different shape per type would make the cheapest type vary across tasks at
fixed prices, and the threshold would split into several, one per type, each found by
inverting a schedule: 1b §2.3's precision loss. That is not in 1c.

### 2.5 Ties are solved; multiple equilibria are refused

Where the cheapest type switches, labour demand jumps (§4.7) and the equilibrium can sit at
the switch with both types in use. 1c solves this in closed form (§4.7): the prototype met
it in 2 of 300 random draws and in 15 of 200 draws built to have a switch mid-line, so it
is not rare. With interest the jump can be upward, and then there can be three equilibria
(one each side of the switch and one at it; §3.3's M5m). 1c counts them and returns
`SolveError::MultipleEquilibria` rather than choose (open question 4). At ρ = 0 this
cannot happen (Proposition, §5.5).

### 2.6 Bitwise nesting through a normative evaluation order

1b nests 1a bit for bit by evaluating in 1a's operation order (1b §5.1). 1c does the same:
§5.1 fixes the formulation of the machine block (the unknowns O and V, not p), the
association of every entry, the order of Gaussian elimination, and a deferred division on
the clearing side, so that for one type, a zero operating recipe, no intermediate inputs
and θ = 1 every extra operation is an exact no-op (x + 0, 0 + x, x·1, x/1, x − (±0)) and
the rest are 1b's. The same order makes a zero build recipe reproduce 1a's flow economy
(u = 1) bit for bit at any (ρ, δ, J_b) (check_dynamics R4). The alternative, 1e-12 with
per-instance bounds, cannot hold near 1a's viability edge (unit-1a.md §4 step 4: two
correct evaluations differ there by up to 2^-53·uλγ/D). Open question 5.

## 3. Parameters, validation and instances

### 3.1 Parameters

1b's parameters stay (1b §3.1), except that a, λ, b, δ and J_b move into the machine types.
New:

| symbol | name in code | meaning |
|---|---|---|
| a_jl | `intermediate` | units of category l used per unit of category j (C×C; row j) |
| θ_k | `task_efficiency` | task units per unit of type k's service, relative to the line's γ; 0 = does no tasks |
| a^op_kl, λ^op_k, b^op_k | `operating: Recipe { machines, labor, land }` | per unit of service, paid in the period |
| a^I_kl, λ^I_k, b^I_k | `build: Recipe { machines, labor, land }` | per unit of capacity built |
| δ_k | `delta` | depreciation per period of type k's capacity |
| J_k | `build_lag` | periods from the start of a build to its first service |
| ρ | `rho` | the interest rate, common to all types |

Derived per type (x-free, computed at construction):

| symbol | meaning | source |
|---|---|---|
| u_k = (ρ + δ_k)(1 + ρ)^(J_k − 1) | user cost | check_dynamics U3 (:63), 1a's `user_cost` |
| ω_k = (1 + ρ)^(J_k − 1) + δ_k Σ_{i<J_k−1}(1 + ρ)^i | machine wealth per unit of installed build cost | L2 (:164), 1a's `wealth_factor` |
| Â = A^op + U·A^I, λ̂ = λ^op + U·λ^I, b̂ = b^op + U·b^I | the price-side machine block (U = diag u) | §4.2 |
| A^q = A^op + Δ·A^I, λ^q = λ^op + Δ·λ^I, b^q = b^op + Δ·b^I | the clearing-side machine block (Δ = diag δ) | §4.5 |

### 3.2 Validation

1b's checks (1b §3.2), in 1b's order, then:
- `intermediate`: C rows of C entries, each in [0, `SCALE_CEIL`], −0.0 stored as +0.0; and
  I − A_cc a nonsingular M-matrix (every pivot of Gaussian elimination without pivoting,
  in index order, is positive), which is ρ(A_cc) < 1 (SSRN p.8);
- at least one machine type, and at least one with θ_k > 0;
- per type: `task_efficiency` 0 or scale; each recipe's `machines` has K entries in
  [0, `SCALE_CEIL`]; `labor` in [0, `SCALE_CEIL`] (1a's λ); `land` 0 or scale; δ_k in
  [`SCALE_FLOOR`, 1]; J_k ≥ 1; u_k finite (1a's checks, per type; a non-finite ω_k or
  interest is the solve's `NonFinite`, as in 1a);
- the recipes are productive: I − (A^op + A^I) is a nonsingular M-matrix (SSRN A.1,
  "productive, with spectral radius of A below one"). For one type with a zero operating
  recipe this is 1a's a < 1. It makes A^q (δ ≤ 1) productive too;
- every type's chain reaches land: (I − (A^op + A^I))⁻¹(b^op + b^I) > 0 in every entry. A
  pattern condition, so the same at every u and δ. For 1a it is b > 0 (SSRN Prop 2's
  b̃_m > 0);
- each category priced: L̄_j > 0 or b̄_j > 0 with the chain totals of §4.4 (1b's rule
  through intermediate inputs); the basket's chain land B_ŷ = Σ ŷ_j b_j > 0 and chain hours
  Σ ŷ_j L̄^dir_j > 0 (1b's B_d > 0 and L̄_s > 0).

Price-side viability, ρ(Â + (γ/θ_τ)λ̂e_τᵀ) < 1 at x = 1, is not validated: an economy that
fails it solves to `NotViable` (§5.3), as in 1a.

### 3.3 Instances

**M1, 1a and 1b in one-type form.** `MachineParams::from_categories(CategoryParams)`: one
type with θ = 1, a zero operating recipe, the build recipe (a, λ, b) and 1b's (δ, J_b),
and a zero `intermediate`. Every 1a and 1b golden instance, random draw and regime row goes
through it (§8, m1).

**M2, the dynamics targets.** check_dynamics' machine (sloped :311-321, flat
:225-231): operating (a, λ, b) = (0.5, 0.1, 0.2), build (a_I, λ_I, b_I) =
(0.1, 0.2, 0.02), (ρ, δ, J) = (0.05, 0.1, 3), so u = 0.165375 and ω = 1.3075. Sloped:
γ = 1 + 4x at the target's x* = 0.5962490482385818 (the double). Flat: γ* = 3 and the
target's m = 0.3570926015168396 (the double).

**M3, the two-recipe economy** (constructed 2026-09-27). M2's machine in Appendix B's
closure: G1's household (N 4, T 10, h 1, χ ~ U[0, 1]), the good and space as in 1b's C1,
and γ = 1 + 2x (η 1, g0 1, g1 2, k 1), which keeps the machine viable at x = 1 (γ(1) = 3 <
3.633). **M3z**: the same at ρ = 0 (u = δ = 0.1, ω = 1.2). The corners (build recipe zero;
operating recipe zero) are 1a economies and are tested in m1 and m3.

**M4, the three-type fork economy** (constructed 2026-09-27). 1b's C3 (fork economy:
N 4, T 10, χ ~ U[0, 1], edges (0, 0.4, 0.75, 1), γ = η(0.2 + 0.8x), the four categories
with 1b's z, b and μ), with intermediate inputs and three machine types, ρ = 0.04:

| intermediate a_jl | manufactures | food | care | shelter |
|---|---|---|---|---|
| manufactures | 0 | 0 | 0 | 0 |
| food | 0.1 | 0 | 0 | 0 |
| care | 0 | 0.2 | 0 | 0 |
| shelter | 0.15 | 0 | 0 | 0 |

| type | θ | operating: (loom, engine, power), λ, b | build: (loom, engine, power), λ, b | δ | J |
|---|---|---|---|---|---|
| 0 loom | 1 | (0, 0, 0.05), 0.3, 0.05 | (0.1, 0, 0), 1, 0.3 | 0.1 | 2 |
| 1 engine | 2 | (0, 0, 0.5), 0.02, 0 | (0, 0.1, 0), 0.1, 0.4 | 0.05 | 3 |
| 2 power | 0 | (0, 0, 0), 0.02, 0.5 | (0, 0.1, 0), 0.1, 0.1 | 0.05 | 3 |

The loom is labour-heavy and cheap at low wages; the engine is twice as capable at tasks
and runs on power, which uses land. The loom is the cheapest task type below a switch and
the engine above it. The basket's chain outputs are ŷ = (0.524, 1.04, 0.2, 0.8). Its
variants: **M4 path**, η ∈ {2, 1, 0.5}; **M4t**, η = 0.5 with N = 8, whose equilibrium is
a tie at the switch; **M4 regimes**, η = 0.25 (`BoundaryNoMargin`), η = 50 (`NotViable`:
no task type is viable at γ(1) = 50), and N = 200 with χ_max = 0.01 (`NoInteriorAtZero`).

**M5, interest selects the technique** (constructed 2026-09-27). G1's household with
γ = 0.2 + 0.8x and two task types (θ = 1 each): **flow**, operating (0, 0; 0.1; 0.5), no
build recipe, δ = 1, J = 1 (neither matters without a build recipe); **durable**, no operating recipe and build (0, 0; 0.02; 1.85), δ = 0.02,
J = 10. The flow type does not see ρ; the durable type's u rises steeply with it. **M5
path**: ρ ∈ {0, 0.05, 0.1, 0.15, 0.3} at N 4, h 1. **M5m**: ρ = 0.1, N = 60, h = 0.2,
where the switch raises labour demand and there are three equilibria.

**M6, random economies** (§8).

## 4. Equations

Per period, r = 1, v = w/r. x is a candidate threshold; τ is a task type (θ_τ > 0), the
technique. Categories are indexed j, l; machine types k, l.

### 4.0 User cost and wealth per type

    u_k = (ρ + δ_k)(1 + ρ)^(J_k − 1)                          check_dynamics U3 (:63)
    ω_k = (1 + ρ)^(J_k − 1) + δ_k·Σ_{i<J_k−1} (1 + ρ)^i        L2 (:164)
    u_k − δ_k = ρ·ω_k                                          L1-L2 (:155-169)

Corners (U4, :65; U4b, :74): J = 1 gives ρ + δ; J = 1, δ = 1 gives 1 + ρ; ρ = 0 gives δ.
The misread (ρ + δ)(1 + ρ)^J is u·(1 + ρ) (U5, :82). u_k is increasing in J_k when ρ > 0
(S4, :136). ω_k = 1 exactly at J_k = 1 and ω_k > 1 for J_k > 1, ρ > 0 (L4, :174). Zero NPV
(L3, :170): u_k(1 + ρ)/(ρ + δ_k) = (1 + ρ)^(J_k).

### 4.1 Task quantities per category

1b §4.1 unchanged: H_j(x), M_j(x) from the densities μ_js and J. M_j is task services at
efficiency 1; type k supplies them with M_j/θ_k units of its service.

### 4.2 The machine block, price side

At threshold x with technique τ (check_dynamics :93-99; SSRN A.1; main.tex:688):

    p_k = O_k + u_k·V_k,   O_k = Σ_l a^op_kl p_l + λ^op_k v + b^op_k,   V_k = Σ_l a^I_kl p_l + λ^I_k v + b^I_k
    v   = γ(x)·p_τ/θ_τ                                          the task margin (SSRN A.1)

Eliminating O and V: p = Âp + λ̂v + b̂ over the machine rows, with Â = A^op + U A^I,
λ̂ = λ^op + Uλ^I, b̂ = b^op + Ub^I. When I − Â is a nonsingular M-matrix:

    λ̃_m = (I − Â)⁻¹λ̂,   b̃_m = (I − Â)⁻¹b̂,   p_k = v·λ̃_k + b̃_k        SSRN eq 4 with the machine rows scaled
    v   = γ·b̃_τ/(θ_τ − γ·λ̃_τ)                                       SSRN A.1's closure, one fixed input
    p_τ/θ_τ = b̃_τ/(θ_τ − γ·λ̃_τ)                                    SSRN eq 5 at θ = 1

Technique τ is **viable** at x when Â + (γ/θ_τ)λ̂e_τᵀ has spectral radius below 1: I − Â is
a nonsingular M-matrix and θ_τ − γλ̃_τ > 0 (the rank-one M-matrix argument; SSRN Prop 2's
1 − γλ̃_m > 0; main.tex:690's "finite iff"). Since γ is increasing, a technique viable at
x = 1 is viable on [0, 1] (1a's convention).

As γλ̃_τ/θ_τ → 0, (p_τ/θ_τ)/b̃_τ → 1 (SSRN eq 6).

**One type** (check_dynamics R1-R6). With K = 1, θ = 1 and γ* = γ(x):

    Den = 1 − a − λγ* − u(a_I + λ_Iγ*)               R1 (:98)
    θ_c = (b + u b_I)/Den = p_m                        R1 (:101)
    θ_w = γ*θ_c = v                                    R2 (:103)
    p_K/r = a_Iθ_c + λ_Iθ_w + b_I = V                  R3 (:105)
    zero build recipe:     p_m = b/(1 − a − λγ*) at every u        R4 (:107), SSRN App B
    zero operating recipe: p_m = u b_I/(1 − u(a_I + λ_Iγ*))        R5 (:115), 1a, main.tex:690
    det of the (O, V) system = Den                     R6 (:122)

and the signs (S1-S6, :126-149), all on the viable set Den > 0: dθ_c/dγ* = θ_c(λ +
uλ_I)/Den > 0; dθ_w/dγ* > 0; dθ_c/dλ = γ*θ_c/Den > 0; dθ_c/dλ_I = uγ*θ_c/Den > 0;
dθ_c/du has the sign of b_I(1 − a − λγ*) + b(a_I + λ_Iγ*) > 0; at λ = 0, dθ_c/dγ* =
θ_c uλ_I/Den > 0.

### 4.3 The technique: least delivered task cost

SSRN A.1: w/γ_L(x*) = min_k p_mk/γ_Mk(x*). With γ_L = 1 and γ_Mk = θ_k/γ, people hold task
x when v ≤ γ(x)·min_k p_k/θ_k, and the machine tasks go to the type with the least
p_k/θ_k. Given v, the machine prices p_k(v) = vλ̃_k + b̃_k do not depend on which type does
tasks (§2.3), and each is increasing in v. So, at fixed x:

    v(x) = min over viable task types t of v_t(x),   v_t(x) = γ(x)·b̃_t/(θ_t − γ(x)·λ̃_t)
    τ(x) = the minimiser

*Proof.* Let g_t(v) = γ·p_t(v)/θ_t, affine with slope γλ̃_t/θ_t < 1 for a viable t, and
v_t its fixed point. The margin is v = min_t g_t(v); min_t g_t is increasing with slope
below 1, so it has one fixed point v*. For the minimiser t̂ at v*, v* = g_t̂(v*), so v* =
v_t̂. For any other t, g_t(v*) ≥ v* and g_t(v) − v is decreasing, so v_t ≥ v*. ∎

The same argument gives SSRN A.1's "unused feasible methods have unit costs at least as
high": at v*, p_t/θ_t ≥ p_τ/θ_τ for every task type t. A type that is not viable at x has
v_t = +∞ and never binds.

**The technique envelope.** For task types a and b, v_a(γ) = v_b(γ) where

    γ·(b̃_b·λ̃_a − b̃_a·λ̃_b) = b̃_b·θ_a − b̃_a·θ_b,

linear in γ: at most one crossing. With Δ_ab = b̃_b·λ̃_a − b̃_a·λ̃_b > 0, b is cheaper than
a for γ above the crossing γ_ab = (b̃_b·θ_a − b̃_a·θ_b)/Δ_ab, and a below it. The lower
envelope of the v_t over γ ∈ [γ(`BRACKET_LO`), γ(1)] is a sequence of pieces (τ_0; γ_1,
τ_1; …; γ_m, τ_m) with each type at most once and m ≤ K − 1 (Lemma 1, §5.5). The
technique at x is τ_i for γ_i ≤ γ(x) < γ_{i+1}: an exact switch point belongs to the piece
above, as 1b's edges do (1b §2.3).

### 4.4 Categories, price side

1b §4.2 with intermediate inputs and the technique's delivered task price π = p_τ/θ_τ =
v/γ:

    p_j = Σ_l a_jl p_l + v·H_j + π·M_j + b_j                  SSRN eq 2, row j
    p_c = (I − A_cc)⁻¹(v·H + π·M + b_c)

Chain totals, all through (I − A_cc)⁻¹ (x-free ones computed once):

    L̄   = (I − A_cc)⁻¹ L̄^dir,   b̄ = (I − A_cc)⁻¹ b_c             all-human method (SSRN A.3: L̄_i, b̄_i)
    L*  = (I − A_cc)⁻¹ (H + M/γ)                                   wage-equivalent task cost (main.tex eq effective-hours)
    λ̃_c = (I − A_cc)⁻¹ (H + M·λ̃_τ/θ_τ),   b̃_c = (I − A_cc)⁻¹ (b_c + M·b̃_τ/θ_τ)

**The fork identity** in both forms (1b §4.2; SSRN eq 12; main.tex eq composites):

    p_j = v·L*_j + b̄_j = v·λ̃_j + b̃_j

**Bounds and pair** (SSRN eq 13, 19; main.tex:465-474), with 1b's direct b_j replaced by
the chain's b̄_j (they agree when A_cc = 0):

    b̄_j ≤ b̃^q_j ≤ b̃_j ≤ p_j ≤ v·L̄_j + b̄_j,     L*_j ≤ L̄_j
    1/(L̄_j + b̄_j/v) ≤ v/p_j ≤ v/b̃_j   (the right side when b̃_j > 0);   b̄_j = 0: v/p_j ≥ 1/L̄_j

b̃^q_j is the clearing-side total of §4.5. The chain uses b̃^q_m ≤ b̃_m componentwise, which
holds because A^q ≤ Â, λ^q ≤ λ̂ and b^q ≤ b̂ (δ_k ≤ u_k when ρ ≥ 0) and the Neumann series
is monotone.

**The basket** (SSRN eq 7; 1b §4.2): ŷ = (I − A_ccᵀ)⁻¹z, the categories' gross outputs per
basket (x-free).

    P_s = zᵀp_c = v·H_ŷ + π·M_ŷ + B_ŷ = v·L_s + B_s = v·L*_s + zᵀb̄
    H_ŷ = Σ ŷ_j H_j,  M_ŷ = Σ ŷ_j M_j,  B_ŷ = Σ ŷ_j b_j = zᵀb̄,  L_s = zᵀλ̃_c,  B_s = zᵀb̃_c
    v/P_s ≤ v/B_s

### 4.5 The full cost system and clearing

Order the rows (categories, machine types). Price side, at technique τ:

    A = [[A_cc, M e_τᵀ/θ_τ], [0, Â]],   λ = (H; λ̂),   b = (b_c; b̂)
    p = Ap + λv + b                                     SSRN eq 3, A.1 (checked by multiplication)

Clearing side, with the machine rows at δ instead of u:

    A^q = [[A_cc, M e_τᵀ/θ_τ], [0, A^q_m]],   λ^q = (H; λ^q_m),   b^q = (b_c; b^q_m)
    y = A^qᵀ y + f,   f = (Y·z; 0)                      SSRN A.1: f = (I − Aᵀ)y
    N_a = λ^qᵀ y,   T = b^qᵀ y                          SSRN A.1, one fixed input fully employed

Solved per basket: the categories' gross outputs y_c = Y·ŷ; task services from type τ,
Y·M_ŷ/θ_τ; machine services X = Y·x̂ with

    x̂ = (I − A^qᵀ_m)⁻¹ e_τ·(M_ŷ/θ_τ)
    Y  = T/(B_ŷ + b^qᵀ_m x̂)                             land clearing
    N_a = Y·(H_ŷ + λ^qᵀ_m x̂)                            labour
    builds_k = δ_k·X_k;   n_D = N_a(x);   n_S = N·F(ln(1 + v/P_s))      SSRN eq 10

Clearing-side totals per unit, λ̃^q_m = (I − A^q_m)⁻¹λ^q_m, b̃^q_m = (I − A^q_m)⁻¹b^q_m, and
per category λ̃^q_c = (I − A_cc)⁻¹(H + M·λ̃^q_τ/θ_τ), b̃^q_c likewise. Then SSRN eq 11:

    Y = T/B_s^q,   n_D = T·L_s^q/B_s^q,   L_s^q = zᵀλ̃^q_c,   B_s^q = zᵀb̃^q_c

At ρ = 0, u_k = δ_k for every k, Â = A^q_m, and the two sides' totals are equal.

With 1 type, θ = 1, zero operating recipe and A_cc = 0, this is 1b §4.3 term for term:
x̂ = M_s/(1 − aδ), b^q = bδ, λ^q = λδ. With the good and space it is SSRN p.29's matrix
display.

### 4.6 Income with interest

SSRN App. C (p.30) with the price side and the clearing side distinguished. From
p = Âp + λ̂v + b̂ on the machine rows and λ̂ = λ^q + (U − Δ)λ^I, b̂ = b^q + (U − Δ)b^I,
Â = A^q + (U − Δ)A^I:

    I = pᵀf = pᵀ(I − A^qᵀ)y = ((I − A^q)p)ᵀy
      = v·λ^qᵀy + b^qᵀy + Σ_k (u_k − δ_k)·V_k·X_k
      = v·N_a + T + Σ_k ρ·ω_k·V_k·X_k                      L1-L2 per type

So interest = Σ_k ρ·W_k, W_k = ω_k·V_k·X_k (L2), computed as ρ·ω_k·V_k·X_k, not
(u_k − δ_k)V_kX_k, as 1a does (unit-1a.md §3.4). The operating recipe earns no interest.
Then, as in 1a and 1b:

    I = Y·P_s = Σ_j p_j·z_j·Y                            income and expenditure
    worker baskets + provider baskets = Y                basket count (Walras)
    labour share = vN_a/I,  capital share = interest/I,  real wage v/P_s

Per type, the ledger (check_dynamics L1-L4): cash u_kV_kX_k − V_k·δ_kX_k = ρW_k, and
W_k ≥ V_kX_k with equality iff J_k = 1.

### 4.7 A tie at a switch

At a switch point x_i (γ(x_i) = γ_i) types a = τ_{i−1} and b = τ_i deliver tasks at the
same cost, prices are continuous, and any split of the machine tasks between them is
cost-minimising. With a share σ of M_ŷ's task services from b and 1 − σ from a, land and
labour per basket are affine in σ:

    B(σ) = B_ŷ + (1 − σ)·ℓ_a + σ·ℓ_b,   L(σ) = H_ŷ + (1 − σ)·h_a + σ·h_b
    ℓ_t = b^qᵀ_m x̂_t,  h_t = λ^qᵀ_m x̂_t    (x̂_t the per-basket services under technique t)
    n_D(σ) = T·L(σ)/B(σ)

n_S is the same for both (the prices are). Labour clearing, T·L(σ) = n_S·B(σ), is linear
in σ:

    σ = B_a·f_a/(B_a·f_a − B_b·f_b),   f_t = T·L_t/B_t − n_S

with B_t = B(σ at t alone), and f_a > 0 ≥ f_b at a tie, so σ ∈ (0, 1]. The quantities are
the σ-mix of the two techniques' per-basket solutions, scaled by Y = T/B(σ). Every identity
of §4.4-4.6 holds with them, because the two delivered costs are equal. The categories'
chain totals take the same mix on both sides: per unit of task services,
(1 − σ)·λ̃_a/θ_a + σ·λ̃_b/θ_b on the price side and likewise with λ̃^q on the clearing side,
so that b̃^q_j ≤ b̃_j still holds (§12, item 2). The jump of labour
demand across the switch is T(L_b/B_b − L_a/B_a).

### 4.8 Outputs

**Economy** (1b's `Eq1b` names where the meaning is the same): x_star, one_minus_x_star,
gamma_star, u (of the technique at x*), v, p_s, y, final_hours, machine_hours (Σ_k
λ^q_kX_k), n_a, participation, income, interest, labor_share, capital_share, real_wage,
support_cost, worker_baskets, provider_baskets, funded, lemma_b1, phi_w and phi_r (only
when every u_k = 1: φ_w = γλ̃_τ/θ_τ, labour's share of the technique's price, computed as
(n_τ·γ)/(U_ττ·θ_τ) from the numerator and pivot of λ̃_τ's back substitution, which for 1a
is 1a's (λ·γ)/(1 − a); φ_r = 1 − φ_w), h_s (H_ŷ), m_s (M_ŷ), b_d (B_ŷ), l_star_s, l_s,
b_s, l_s_q, b_s_q, rent_ceiling, margin_active (1b §2.3 with ŷ for z), bisection_steps. New: `d` (the least
pivot of the technique's machine system at x*, §5.1; 1a's D), `technique` (τ at x*),
`tie` (`Some { above, share σ, gamma }` at a tie), `switches` (the envelope's switch
points on the line: γ_i, the double below each, and the two types).

**Per type**: price p_k, operating_cost O_k, build_cost V_k (1a's V_m), user_cost u_k,
wealth_factor ω_k, delivered_cost p_k/θ_k and closure_wage v_k(x*) (task types only;
absent where not viable), lambda_tilde, b_tilde (price side), lambda_tilde_q, b_tilde_q
(clearing side), services X_k (= capacity), task_services, builds δ_kX_k, hours λ^q_kX_k,
land b^q_kX_k, wealth W_k, interest ρW_k, and phi_w,k = vλ̃_k/p_k when u_k = 1.

**Per category**: 1b's fields (price, real_wage, l_star, l_bar, human, machine,
lambda_tilde, b_tilde, lambda_tilde_q, b_tilde_q, output, final_hours, machine_services,
share, wage_floor, wage_ceiling, phi_w, phi_r), with the chain meanings of §4.4, plus
chain_land b̄_j and gross_output Y·ŷ_j (output stays final: z_j·Y).

**Name map to 1b** (for nesting): 1b's p_m, v_m, k, lambda_tilde_machine and
b_tilde_machine are types[0].price, .build_cost, .services, .lambda_tilde and .b_tilde.
`d` is new; for one type the regime's `d_at_1` is 1a's D(1).

### 4.9 The machine block on its own

`MachineBlock` (the validated types and ρ) gives, with no equilibrium:
- `totals()`: u_k, ω_k, λ̃_m, b̃_m (when I − Â is a nonsingular M-matrix), λ̃^q_m, b̃^q_m;
- `closure(gamma_star, technique)`: the §5.1 step-2 system at γ*: p_k, O_k, V_k, v, the
  pivots and d, or `ClosureError::NotViable { d }`. At one type with a zero operating
  recipe it is 1a's `closure` at r = 1 up to association (§8, m2);
- `envelope(gamma_lo, gamma_hi)`: the pieces of §4.3;
- `gross_services(task_services)`: X = (I − A^qᵀ_m)⁻¹t, for M2's clearing-side targets.

## 5. Solution method

### 5.1 The evaluation at x with technique τ (normative)

The order is normative, as in 1b §5.1, so that the one-type case repeats 1b's (and 1a's)
floating-point operations. Sums start from 0.0 and run in index order. "GE" is Gaussian
elimination without pivoting in index order: for k, for i > k, m = G_ik/G_kk, G_ij ←
G_ij − m·G_kj for j > k, r_i ← r_i − m·r_k; back substitution z_i = (r_i − Σ_{j>i}
G_ij·z_j)/G_ii, the subtractions one at a time in increasing j. On a nonsingular M-matrix
every pivot is positive and the off-diagonal entries stay nonpositive.

1. **γ = γ(x), J = J(x)** (1a).
2. **The machine system.** Unknowns (O_0 … O_{K−1}, V_0 … V_{K−1}), 2K rows; with
   [l = τ] the margin column:

       G[O_i, O_l] = (i = l ? 1.0 − a^op_ii : −a^op_il) − [l = τ]·(λ^op_i·γ)/θ_τ
       G[O_i, V_l] = −(a^op_il·u_l)                     − [l = τ]·((u_τ·λ^op_i)·γ)/θ_τ
       G[V_i, O_l] = −a^I_il                            − [l = τ]·(λ^I_i·γ)/θ_τ
       G[V_i, V_l] = (i = l ? fma(−u_i, a^I_ii, 1.0) : −(a^I_il·u_l)) − [l = τ]·((u_τ·λ^I_i)·γ)/θ_τ
       right-hand side: b^op_i for O_i, b^I_i for V_i

   GE; the pivots G_kk after elimination; d = the first pivot that is not positive (NaN
   included), or else the least (§12, item 1); τ is viable at x when d > 0. Then p_k = O_k + u_k·V_k, v = (γ·p_τ)/θ_τ, π = p_τ/θ_τ.

   For 1a (K = 1, zero operating recipe, θ = 1): G[O,O] = 1.0, G[O,V] = −0.0,
   G[V,O] = −a − λγ, G[V,V] = fma(−u, a, 1.0) − (u·λ)·γ, which is 1a's D; the elimination
   subtracts m·(−0.0) = +0.0 from G[V,V] and m·0.0 from b, with m = −a − λγ, so neither
   moves; V = b/D, O = (0.0 − (−0.0)·V)/1.0 = 0.0, p = 0.0 + u·V = u·(b/D), and
   v = (γ·p)/1.0: 1a's V_m, p_m and v. The least pivot is min(1.0, D) = D, since D ≤ 1.
3. **Categories.** H_j, M_j by 1b's step 2; rhs_j = (v·H_j + π·M_j) + b_j (1b's p_j);
   p_c by forward and back substitution with the stored factors of I − A_cc (the identity
   when A_cc = 0, and then exact). Then, in one loop in category order: P_s += z_j·p_j,
   H_ŷ += ŷ_j·H_j, M_ŷ += ŷ_j·M_j (1b's step 3; ŷ = z exactly when A_cc = 0).
4. **Quantities.** m̂ = M_ŷ/θ_τ. Forward substitution with the stored factors of
   I − A^qᵀ_m on e_τ·m̂ gives r; back substitution with the **division deferred**: for k
   from K−1 down, n_k = r_k − Σ_{l>k} U_kl·x̂_l and x̂_k = n_k/U_kk. Then ℓ = Σ_k
   (b^q_k·n_k)/U_kk (k increasing), Y = T/(B_ŷ + ℓ), X_k = (Y·n_k)/U_kk, final_hours =
   Y·H_ŷ, machine_hours = Σ_k λ^q_k·X_k, n_D = final_hours + machine_hours, n_S =
   N·F(ln1p(v/P_s)). The stored factors have diagonal fma(−δ_k, a^I_kk, 1.0 − a^op_kk),
   off-diagonal −(a^op_lk + δ_l·a^I_lk) at (k, l), b^q_k = b^op_k + δ_k·b^I_k and λ^q_k =
   λ^op_k + δ_k·λ^I_k. For 1a: U = fma(−δ, a, 1.0) = 1a's s_K, ℓ = ((δ·b)·M_s)/s_K,
   X = (Y·M_s)/s_K, machine_hours = (δ·λ)·X: 1b's Y, K and machine hours.
5. **At the root**: 1b step 5 (the top segment's hours from the carried 1 − x*).

The residuals recompute from the recipes (§6), so they are not zero by construction; the
user-cost residual is |p_k − (Σ_l a^op_kl p_l + λ^op_k v + b^op_k + u_k·((Σ_l a^I_kl p_l +
λ^I_k v) + b^I_k))|/p_k, which for 1a is 1b's |p_m − u(ap_m + λv + b)|/p_m.

### 5.2 At construction (x-free)

In `new`, after validation: u_k, ω_k; the factors of I − A_cc and I − A_ccᵀ; ŷ, L̄, b̄,
B_ŷ, the basket's chain hours; the factors of I − A^q_m and I − A^qᵀ_m (always nonsingular
M-matrices after validation, since A^q_m ≤ A^op + A^I); λ̃^q_m, b̃^q_m; the factors of
I − Â with diagonal fma(−u_k, a^I_kk, 1.0 − a^op_kk) and, if every pivot is positive, λ̃_m
and b̃_m (for 1a: (u·λ)/fma(−u, a, 1.0), 1b's λ̃_m); and the envelope:

1. Task types are those with θ_t > 0. If I − Â has a nonpositive pivot, the envelope is
   the lowest task type alone (the solve will find `NotViable`).
2. At γ_c = γ(`BRACKET_LO`), the candidates are the task types with θ_t − γ_c·λ̃_t > 0.
   τ_0 = the least v_t(γ_c); ties go to the smaller λ̃_t/θ_t, compared as λ̃_t·θ_s <
   λ̃_s·θ_t (the one cheaper just above), then the lower index. No candidate: the lowest
   task type alone.
3. From the current τ at γ_c: for every unvisited task type l with Δ_τl > 0, the crossing
   γ_τl = (b̃_l·θ_τ − b̃_τ·θ_l)/Δ_τl counts when γ_c < γ_τl ≤ γ(1) and θ_l − γ_τl·λ̃_l > 0.
   The least counting crossing is the next switch; ties go to the smaller λ̃_l/θ_l (at a
   common crossing the one cheaper just above), then the lower index. Mark l visited and
   repeat until no crossing counts. At most K − 1 switches.

For one type the envelope is (τ_0 = 0), no switches.

### 5.3 The solve

1. **Regime tests** (1a's order and conventions): with τ_m the envelope's last piece,
   evaluate x = 1 under τ_m: d(1) ≤ 0 is `NotViable { d_at_1 }` (1a's D(1) for one type);
   f(1) ≥ 0 is `BoundaryNoMargin`. Evaluate x = `BRACKET_LO` under τ_0: f ≤ 0 is
   `NoInteriorAtZero`. `solve::classify` is split into these tests and `bisect`; 1a and 1b
   call both, unchanged.
2. **Switch points.** For each switch γ_i, x_i is the largest double with γ(x) < γ_i,
   found by bisection on [x_{i−1}, 1] for the predicate γ(x) ≥ γ_i until the two ends are
   adjacent doubles (x_0 = `BRACKET_LO`). No tolerance, as 1a's bisection.
3. **Sign changes.** With f_i(x) the excess demand under τ_i and x_{m+1} = 1, the sequence
   f_0(x_0), f_0(x_1), f_1(x_1), f_1(x_2), …, f_m(x_m), f_m(x_{m+1}) starts > 0 and ends
   < 0. A value is on the positive side when > 0. With more than one change of side:
   `SolveError::MultipleEquilibria { sign_changes, switches }`. With one:
   - between f_i(x_i) and f_i(x_{i+1}): 1a's `bisect` of f_i on [x_i, x_{i+1}] (an exact
     zero at x_{i+1} is a root there, 1a's `Root::exact`); the root carries 1 − x* from the
     line through its two doubles (1a);
   - between f_{i−1}(x_i) and f_i(x_i): a tie at x* = x_i, reported with technique τ_{i−1}
     and share σ toward τ_i (§4.7), from the two techniques evaluated at the same double
     x_i with τ_{i−1}'s prices (n_S included); 1 − x* = 1.0 − x_i. If f_b ≥ 0 there, which
     happens only within rounding of a root of f_i at x_i, σ = 1.
4. **Report** at x* (§4.8), the finiteness check of every output
   (`SolveError::NonFinite`, `NonFiniteInCategory`, and the new `NonFiniteInType
   { machine_type, what }`), and 1a's labour net (`LaborNotCleared` above
   `LABOR_RESIDUAL_NET`·N_a).

For one type there are no switches, the sequence is f(lo), f(1), and step 3 is 1a's
bisection on [`BRACKET_LO`, 1]: the same calls, the same steps. No constant changes:
`BRACKET_LO`, `MAX_BISECTION_STEPS` and `LABOR_RESIDUAL_NET` are 1a's, and each switch
search is a bisection on a sub-interval of the bracket, so it takes at most
`MAX_BISECTION_STEPS` too.

### 5.4 Which equation fixes which unknown

- Cost minimisation across types (the envelope) fixes the technique τ at each x.
- The task margin and τ's row fix v given x (the closure).
- Each type's two-recipe row fixes p_k, and with it O_k and V_k.
- Zero profit fixes each p_j, through the category chain.
- Land clearing fixes Y; service clearing fixes X_k for every type; builds are δ_kX_k.
- Labour clearing is the root equation, or fixes σ at a tie.
- Walras gives the income identity and the basket count, which are checked, not imposed.

### 5.5 Uniqueness

**Lemma 1 (no reswitching along the line).** The machine totals are x-free (§2.3), so each
pair of task types crosses at most once in γ (§4.3), and the lower envelope of functions
that pairwise cross at most once visits each at most once. So the technique is a step
function of γ(x) with at most K − 1 steps.

**Lemma 2 (within a technique f is nonincreasing).** Fix τ. 1b §5.3 holds with ŷ for z and
(λ̃^q_τ, b̃^q_τ)/θ_τ for (λδ/s_K, bδ/s_K): dL^q_s/dx = μ_ŷ(x)(−1 + γλ̃^q_τ/θ_τ) ≤ 0, since
viability gives γλ̃_τ < θ_τ and λ̃^q ≤ λ̃; dB^q_s/dx = μ_ŷγb̃^q_τ/θ_τ ≥ 0; and P_s/v =
H_ŷ + M_ŷ/γ + B_ŷ/v falls, since v is increasing in γ. So n_D is nonincreasing and n_S
nondecreasing.

**Lemma 3 (at a switch).** v and P_s are continuous, so n_S is; n_D jumps by
T(L_b/B_b − L_a/B_a) (§4.7).

**Proposition (ρ = 0).** Then u_k = δ_k, the price and clearing totals coincide, and at a
switch from a to b (b cheaper above), λ̃_b/θ_b < λ̃_a/θ_a and so b̃_b/θ_b > b̃_a/θ_a at the
tie: labour per basket falls and land per basket rises, so n_D falls. f is then
nonincreasing on the whole bracket and the equilibrium is unique: an interior root or a
tie. The prototype found every one of 100 switches at ρ = 0 downward.

**With interest** the incoming type can be dear on the price side (interest on a long
build) but cheap in land and labour per period, and n_D can jump up: 14 of 100 switches at
ρ ∈ [0.02, 0.12] in the prototype's switch set, and M5m, which has three equilibria. The
count of step 3 decides; a flat zero stretch of f (1b §5.3) is not constructed.

**Existence** is the regime tests' sign change, as in 1a.

### 5.6 Precision

Everything in unit-1a.md §4 step 4 and 1b §5.4 carries over (the viability edge, u's error,
thin segments, interior edges, gaps). New:

- **Elimination on M-matrices.** Off-diagonal updates add nonpositive terms and back
  substitution adds nonnegative ones, so the only cancellation is on the diagonal, where
  a pivot near 0 is the viability edge (1a's D, generalised). The (O, V) form keeps V free
  of cancellation (V = 0 exactly for a zero build recipe, which interest needs), where
  V = p/u − O/u would lose 2^-53·p/(uV) relative.
- **Switch points.** γ_i carries the rounding of the totals, amplified by (|b̃_lθ_τ| +
  |b̃_τθ_l|)/|b̃_lθ_τ − b̃_τθ_l|; for nearly equivalent types the switch is ill-located but
  their costs are then nearly equal. A tie's x* is the double below the computed γ_i.
- **Decidability.** A value of the sign sequence within rounding of 0 (relative to n_D)
  makes the count, and so the regime, not decidable in f64, as 1a's regime tests.
- **Nesting.** For the one-type case the evaluation is 1a's and 1b's (§5.1), so every 1a
  and 1b precision statement holds unchanged, G3's 1 − x* = 4.6e-21 included.
- **To be measured** by the build: the f64 values against every golden, away from the
  edges and near them, as 1b §5.4 did.

## 6. Result type

Proposed; the build may rename, and records any departure in a §12.

- `src/machines.rs`: `Recipe { machines: Vec<f64>, labor: f64, land: f64 }`,
  `MachineType { task_efficiency, operating: Recipe, build: Recipe, delta, build_lag }`,
  `MachineParams<S>` (1b's `CategoryParams` without a, λ, b, δ, J_b, plus `intermediate:
  Vec<Vec<f64>>` and `machine_types: Vec<MachineType>`), with
  `MachineParams::from_categories(CategoryParams)` building M1's form, and
  `MachineEconomy<S>` with `new`, `at(x)` (the envelope's technique), `at_with(x,
  technique)`, `solve() -> Result<Regime<Eq1c>, SolveError>`, `block()`.
- `src/leontief.rs` (crate-private): GE without pivoting and substitution in §5.1's order,
  the factors, pivots and deferred division.
- `src/machine_block.rs`: `MachineBlock` (§4.9), `BlockTotals`, `BlockPrices`, `Switch`.
- `Eq1c` (§4.8) with `types: Vec<TypeEq>`, `categories: Vec<CategoryEq1c>`, `tie:
  Option<Tie>`, `switches: Vec<Switch>`, and `residuals: Residuals1c`: 1b's eight, with
  `services` and `user_cost` the largest over types, plus `leontief_price` (largest
  |p_i − (Ap + λv + b)_i|/p_i over the C + K rows), `leontief_quantity` (largest
  |y_i − (A^qᵀy + f)_i|/y_i over rows with y_i > 0), `closure` (|v − γb̃_τ/(θ_τ − γλ̃_τ)|/v)
  and `cheapest` (largest (π − p_t/θ_t)⁺/π over task types, 0 unless a type undercuts the
  technique). Each residual's expression and order are fixed so that, for one type, 1b's
  eight residuals are its own bit for bit: the services residual is |(X_k − task_k) −
  Σ_l A^q_lk·X_l|/X_k with the task term(s) first (0 when X_k = 0), the land residual
  |(B_ŷ·Y + Σ_k b^q_k·X_k) − T|/T, the user-cost residual as in §5.1, and fork, totals and
  expenditure 1b's with the chain totals.
- `Eq1c::outputs()`: economy keys in 1b's order, then 1c's, then `type<k>.<field>`, then
  `cat<j>.<field>`, as `(OutputKey1c, Output1b)` (§12, item 3).
- `SolveError` gains `NonFiniteInType { machine_type, what }` and `MultipleEquilibria
  { sign_changes, switches: Vec<f64> }`. `ParamError::Item` gains the kind "machine type"
  (with names such as "operating.labor") and `ParamError::Invalid` covers "intermediate
  inputs are not productive", "machine recipes are not productive" and "machine type k's
  recipes use no land".
- No new function in `core::num`: γ is never inverted in the oracle, and fma, pow and
  ln1p are there.

## 7. Goldens

`goldens/generate_1c.py` computes every 1c golden with mpmath at 70 digits from §4's
equations, with its own Leontief solves (`mp.lu_solve`), the envelope in closed form, the
switch points by γ⁻¹ in closed form (x = ((γ/η − g0)/g1)^(1/k)), bisection to 2^-250 in
each region, and σ in closed form. It writes `goldens/goldens_1c.txt` with 30 significant
digits and a header of FNV-1a digests of `generate_1c.py`, `generate_1b.py`, `generate.py`
and its own body; the Rust constants in `tests/gate/goldens_1c.rs` carry 20 digits
(`m8`). It imports `generate_1b.py` (and so `generate.py`) only to assert the nesting.
The key prefixes are M2_ (the dynamics targets), M3_ and M3Z_, M4_, M4_ETA2_, M4_ETA05_,
M4T_, M4_REG_, M5_RHO*_ and M5M_.

The generator asserts as it goes:
- the one-type form of 1b's golden instances (G1, λ = 0, η = 0.5, G4's lags, G5's durable
  instance, C3, C3d, C3z, the gap economy) equals `generate_1b.py`'s solve to 1e-65;
- the zero-build form of M3's machine equals `generate.py`'s flow solve of (a, λ, b) =
  (0.5, 0.1, 0.2) on γ = 1 + 2x, and the zero-operating form equals its solve with
  (0.1, 0.2, 0.02) at u = 0.165375, to 1e-65;
- check_dynamics' targets, recomputed from its own equations (the land-share closure at
  70 digits for the sloped x*, the flat closed form of :212-214 for m, X and Y), match the
  JSON's doubles within 2e-15 relative (the prototype: r 1.4e-15, where Den = 0.033
  amplifies the JSON's own f64 rounding; every other value within 1.1e-16; the sloped x* is
  1.13e-16 below the 70-digit root, one spacing of doubles, so the next double up is the
  nearest; 1c uses the JSON's double, as an input);
- every identity of §4 at every instance to 1e-65: both fork forms, p = Ap + λv + b over
  all rows, f = (I − A^qᵀ)y = (Yz, 0), N_a = λ^qᵀy, T = b^qᵀy, pᵀf = vN_a + T + interest,
  eq 11, income by Y·P_s and Σ p_jz_jY, interest = Σ(u_k − δ_k)V_kX_k = Σρω_kV_kX_k, the
  basket count, the closure per task type, the cheapest-type inequality, and every bound;
- the envelope equals a scan of argmin v_t on 10^4 points of the line; the tie's two
  delivered costs are equal at x_sw to 1e-65;
- f is nonincreasing on a grid within each technique region, and the sign-change count is
  the instance's regime; at ρ = 0 every switch in M4 and M5 lowers n_D.

All values below are the prototype's, to 20 significant digits; the generator must
reproduce them to the digits shown.

**M2, the dynamics targets.** u = 0.165375, ω = 1.3075; price-side totals λ̃ =
0.27525402694107609173, b̃ = 0.42052382552938438865; clearing λ̃^q = 0.12/0.49 =
0.24489795918367346939, b̃^q = 0.202/0.49 = 0.41224489795918367347.
- Sloped, at x = 0.5962490482385818 (0.59624904823858182468…): γ* 3.3849961929543272987,
  J 1.3072749032894111761, Den 0.033004131622602894724; p_m = c/r 6.1600620893404983605,
  v = w/r 20.851786720779866155, V = p_K/r 4.8063635530900230671, O 5.3652097167482357957,
  p_good = 1/r 16.471823306540339961; labour per unit of the good 0.7238999076690290756,
  so Y (N = 1) 1.3814064477781441699; X/Y 2.6679079658967575023. The land-share root at
  70 digits is 0.596249048238581937458519110633.
- Flat, at γ* = 3: Den 0.0842375, p_m 2.4135034871642676955, v 7.2405104614928030865,
  V 1.7094524410149873869, O 2.1308027897314141564; at m = 0.3570926015168396: Y
  1.1046536171646545931, X 2.4150834730304906089. The land-share m at 70 digits is
  0.35709260151683956143.

**M3, the two-recipe economy, ρ = 0.05:** x* 0.91057468799335480604, 1 − x*
0.089425312006645193958, γ* 2.8211493759867096121; v 5.3088781572501366285, p_m
1.8818139168520038924, O 1.6717947741510156091, V 1.2699570231352277149, p_good
3.7485791815332800258, P_s 4.7485791815332800258; Y 5.8234637271311258276, X
20.675922142915218675, N_a 3.001875717907908079 (final hours 0.52076506075808183802,
machine hours 2.481110657149826241); interest 1.7165861894881998391, W
34.331723789763996783, I 27.653178619069065849, labour share 0.57630237193026765214,
capital share 0.062075547015216440453, v/P_s 1.117992973118317116.

**M3z, ρ = 0:** x* 0.93944579285911351352, v 4.0235521384808297705, p_m
1.3976046053422440254, O 1.3011575165192049898, V 0.96447088823039035664, Y
5.7106572226425542111, X 21.23437018493785044, N_a 2.8939287425630387991, I
21.643873180750653453 = vN_a + T; interest 0; λ̃ = λ̃^q = 0.24489795918367346939.

**M4, the three-type fork economy, η = 1:** the switch loom → engine at x
0.28938326514899913117; x* 0.81660019325202962523 with the engine at the margin; v
0.12971114414607967078, P_s 1.5507722758016523373, Y 6.5098827079475574343, N_a
0.3213138107587870914, I 10.095345622205656911, interest 0.05366764018219771385, v/P_s
0.083642934665585968587. Per category chain: ŷ = (0.524, 1.04, 0.2, 0.8), L̄ = (0.8, 0.805,
0.411, 0.26), b̄ = (0, 0.6, 0.22, 1).

| type | u | ω | λ̃ | b̃ | p | O | V | X | closure wage |
|---|---|---|---|---|---|---|---|---|---|
| loom | 0.1456 | 1.14 | 0.45371483821077908998 | 0.12107468709546726147 | 0.17992655787584084456 | 0.11474088460549940179 | 0.44770379993366375524 | 0 | 0.16857294780232336472 |
| engine | 0.097344 | 1.1836 | 0.045262504992904392782 | 0.29815838954071774421 | 0.30402944085026501671 | 0.26086963649967659899 | 0.44337405849963446875 | 2.2008417237567965685 | 0.12971114414607967078 |
| power | 0.097344 | 1.1836 | 0.030175003328602928521 | 0.51263679302714516281 | 0.51655082723351001114 | 0.50259422288292159342 | 0.14337405849963446875 | 1.1004208618783982843 | — |

The loom is unused and its closure wage is above v (SSRN A.1's unused method). Clearing
totals: λ̃^q = (0.40531257155942294481, 0.037783375314861460957, 0.025188916876574307305),
b̃^q = (0.10638246444291784342, 0.27455919395465994962, 0.50637279596977329975).

| category | p_j | v/p_j |
|---|---|---|
| manufactures | 0.043780239482438162407 | 2.9627783145889711812 |
| food | 0.66799618444616177099 | 0.19417946863517444161 |
| care | 0.26575731319520371529 | 0.48808118424498222654 |
| shelter | 1.0206131960896479681 | 0.1270913845157516329 |

**M4 path:**

| η | switch x | x* | technique | v |
|---|---|---|---|---|
| 2 | 0.019691632574499565584 | 0.74204429563077694458 | engine | 0.24544595555111700983 |
| 1 | 0.28938326514899913117 | 0.81660019325202962523 | engine | 0.12971114414607967078 |
| 0.5 | 0.82876653029799826234 | 0.89769898858934104979 | engine | 0.069157733897259938505 |

**M4t, the tie (η = 0.5, N = 8):** x* = x_sw 0.82876653029799826234 (γ_sw
0.43150661211919930493); f at x_sw under the loom 0.81345450186486179175 and under the
engine −0.067383662779535658336; σ (toward the engine) 0.92300430261428115677;
v 0.064963057288499518554, P_s 1.4968975100449137833, Y 6.7140878145988791705, N_a
0.33986513945300654882, I 10.050301331895958964, interest 0.028222653371309421724, X =
(0.17575741974157999639, 1.0508577148308887454, 0.53421672840252337253).

**M4 regimes:** η = 0.25, `BoundaryNoMargin`, f(1) 0.39646115974723321568; η = 50,
`NotViable`; N = 200 with χ_max 0.01, `NoInteriorAtZero`, f(1e-12) −190.75346260388830846.
d(1) for η = 50 is the generator's.

**M5 path** (the technique at x*; the switch lies below x* where shown):

| ρ | technique | x* | v | N_a | interest | I |
|---|---|---|---|---|---|---|
| 0 | durable | 0.98585935628627146194 | 0.036595909713248331463 | 0.14069985600957043599 | 0 | 10.00514903922719328 |
| 0.05 | durable | 0.93600030634593811309 | 0.1910047225509255231 | 0.62962264010530237988 | 0.8657707963246116886 | 10.986031694009706203 |
| 0.1 | durable (switch at 0.34233278653318684224) | 0.87478588741106422192 | 0.47343913004420151224 | 1.2321344454088485648 | 2.3110160893320139581 | 12.894356749263873923 |
| 0.15 | flow | 0.88954352648655180271 | 0.50153949761761870206 | 1.2820504100197906925 | 0 | 10.642998918561787894 |
| 0.3 | flow | 0.88954352648655180271 | 0.50153949761761870206 | 1.2820504100197906925 | 0 | 10.642998918561787894 |

Higher interest makes the durable machine dearer; below the switch rate the wage rises
with ρ (v 0.037 → 0.473), because the machine the wage is pinned to costs more, and above it
the flow machine takes the margin and ρ drops out.

**M5m, three equilibria (ρ 0.1, N 60, h 0.2):** switch at x 0.34233278653318684224;
f(1e-12) 32.536714584059839575 (flow), f at the switch −1.7185567914492680325 (flow) and
4.5089416135953250503 (durable), f(1) −42.190024304465227684: three sign changes. The
generator records the three equilibria for reference: a flow root at
0.32226807920428180529, the tie with σ 0.32468517480835597758, a durable root at
0.4031287106909996285.

## 8. Tests

The groups are the modules `m1_nesting`, `m2_machine_block`, `m3_two_recipe`,
`m4_many_types`, `m5_interest`, `m6_random_leontief`, `m7_regimes_and_validation` and
`m8_goldens_file` of `tests/gate/`, cited as `m1::` to `m8::`. The gate maps onto them:
- check_dynamics' targets in this closure: `m2::dynamics_sloped_target`,
  `m2::dynamics_flat_target`, and the U, R, S and L tests of m2;
- 1a and 1b as the one-type case exactly: m1;
- the income identity to 1e-12 with interest: `m3::income_with_interest`,
  `m6::identities`, and every golden group;
- random instances satisfying the Leontief identities: `m6::leontief_identities` and the
  rest of m6.

Identities and goldens are compared at 1e-12 relative (ADDENDUM A7); "bitwise" means
`to_bits` equality. An inequality a ≤ b is checked as a ≤ b·(1 + 1e-12), and strictly
where the instance has room. The shared identity check (`support_1c::check_identities_1c`)
also pins every p_k, p_j, P_s and v to `at_with(x*, technique)` bit for bit.

**m1, nesting (R1; §2.6).**
- `m1::appendix_b_is_bit_identical`: G1 through `from_categories(from_one_category(G1))`:
  every shared output (§4.8's name map), the regime and `bisection_steps` equal 1b's and
  1a's bit for bit, and `d` equals 1a's D(x*). [R1; §5.1]
- `m1::every_1b_golden_instance_is_bit_identical`: 1b's c1 set (1a's 27 instances,
  near-full automation with the carried hours included) and 1b's C3, C3d, C3z, the seven
  C4 points, the gap economy at N 4.5 and 5, the three near-edge roots and the two
  margin-on-an-edge roots. [R1]
- `m1::random_economies_are_bit_identical`: 1a's G5 180 draws and 1b's C5 180 draws, and
  every skipped draw's regime and diagnostic. [R1]
- `m1::regime_rows_nest`: 1a's G8 rows and 1b's c7 rows give the same regime and
  diagnostic bit for bit (`d_at_1` is the least pivot, which is D(1)), exact zeros
  included; the jump schedule's `LaborNotCleared`; the ρ = 1e30 `NonFinite`. [§5.3]
- `m1::rejected_rows_nest`: the rows 1a and 1b reject are rejected (a ≥ 1 as "machine
  recipes are not productive", b = 0 as "recipes use no land", b below the floor, λ 1e300,
  the overflowing u, h = 0 as the basket), each error named. [§3.2]
- `m1::points_nest_on_a_grid`: `at(x)` equals 1b's `at(x)` bit for bit on 1b's grid
  (prices, H, M, P_s, Y, K, n_D, n_S). [§5.1]
- `m1::flow_recipe_is_the_flow_economy`: a zero build recipe with operating (a, λ, b) =
  G1's at (ρ, δ, J) ∈ {(0, 1, 1), (0.05, 0.1, 3), (0.3, 0.5, 4)} equals 1a's G1 (u = 1)
  bit for bit on every shared output except u, φ (reported only when u = 1) and V_m, which
  is the flow type's operating cost O (its V is 0.0); interest and W are 0. [check_dynamics
  R4; SSRN App B]

**m2, the machine block alone.**
- `m2::dynamics_sloped_target`: M2 sloped goldens at 1e-12 (u, γ*, p_m, v, V, O, p_good,
  the labour and services per unit of the good, and Y and X from them); the JSON's doubles
  within 2e-15 of the goldens. [§0.2; R1-R3; EJ6]
- `m2::dynamics_flat_target`: M2 flat goldens, through `closure(3.0, 0)` and
  `gross_services`. [E1-E3]
- `m2::two_recipe_closed_forms`: on 200 random viable one-type blocks, p_m = θ_c, v =
  γθ_c, V = a_Iθ_c + λ_Iθ_w + b_I at 1e-12, the product of the 2K pivots = Den at 1e-12,
  and d > 0 exactly when Den > 0 away from 0. [R1-R3, R6]
- `m2::recipe_corners`: a zero build recipe gives b/(1 − a − λγ*) bit-equal at five u; a
  zero operating recipe gives 1a's `closure` within 2 ulps (1a's closure associates
  u·b·r/D) and 1a's solve's p_m bit for bit. [R4, R5; main.tex:690; SSRN A.4]
- `m2::user_cost_per_type`: u_k = `user_cost(ρ, δ_k, J_k)` bit for bit; the free-entry PV
  sum Σ_{s≥J}(1 + ρ)^(−s)(1 − δ)^(s−J) summed to convergence at (0.05, 0.10, 3) and
  (0.12, 0.04, 7) equals (1 + ρ)^(1−J)/(ρ + δ) at 1e-12; U4 and U4b's corners; the misread
  is u(1 + ρ). [U1-U6]
- `m2::comparative_statics`: on 200 random viable points, each S1-S6 sign by a forward
  step in the parameter (strict), and S1 and S3's closed-form derivatives against central
  differences at 1e-6. [S1-S6]
- `m2::ledger`: per type, (u − δ)VX = ρωVX at 1e-12 (ρ ≥ 1e-3), ω = 1.0 exactly at J = 1,
  ω > 1 at J > 1 and ρ > 0, and u(1 + ρ)/(ρ + δ) = (1 + ρ)^J at 1e-12. [L1-L4]
- `m2::closure_per_type`: for every task type, v_t = γb̃_t/(θ_t − γλ̃_t) from the totals
  equals the block's v with τ = t at 1e-12; the envelope's technique has the least; eq 6's
  limit as λ → 0. [SSRN A.1, eq 5-6]
- `m2::leontief_totals`: (I − Â)λ̃ = λ̂ and (I − Â)b̃ = b̂ by multiplication at 1e-12; on 200
  random nonnegative matrices, all pivots positive exactly when power iteration puts ρ(A)
  below 1 (away from 1 by 1e-6); on the identity and on diagonal matrices the solve is
  exact. [SSRN eq 3-4, A.1]
- `m2::envelope`: the switch γ's equal the closed form; a γ exactly at a switch takes the
  piece above; no type repeats; the tie-breaks of §5.2. [§4.3, Lemma 1]

**m3, the two-recipe economy (M3).**
- `m3::goldens`: M3 and M3z. [§4]
- `m3::price_and_clearing_totals`: with ρ > 0, λ̃ > λ̃^q and b̃ > b̃^q; at ρ = 0 they are
  bit-equal (u = δ exactly). [§4.5]
- `m3::income_with_interest`: I = vN_a + T + Σρω_kV_kX_k = Y·P_s = pᵀf = Σ_j p_jz_jY at
  1e-12; interest = Σ(u_k − δ_k)V_kX_k at 1e-12; worker plus provider baskets = Y.
  [§4.6; SSRN App C; L1-L2]
- `m3::corners`: M3's build-only and operating-only forms equal 1a solves bit for bit.
  [R4, R5]
- `m3::single_crossing_on_a_grid`. [§5.5]

**m4, many types (M4).**
- `m4::three_type_goldens`: the η = 1 goldens, per type and per category. [§4]
- `m4::task_automation_path`: the η table; the switch moves up the line and v falls as η
  falls; the loom is unused at x* and its closure wage is above v at each point. [§4.3]
- `m4::leontief_identities`: at M4 and M4t, p = Ap + λv + b row by row over the C + K
  rows, f = (I − A^qᵀ)y = (Yz, 0), N_a = λ^qᵀy, T = b^qᵀy, pᵀf = vN_a + T + interest.
  [SSRN eq 3, A.1, App C]
- `m4::fork_through_intermediate_inputs`: both fork forms, the bounds and the pair with
  chain totals, for every category. [§4.4; SSRN eq 12-13, 19]
- `m4::power_does_no_tasks`: the power type's services equal the demand of the engine's and
  its own recipes at the clearing coefficients; it has no closure wage. [§2.4]
- `m4::tie_goldens`: M4t's goldens; the two delivered costs equal at 1e-12; labour clears;
  `tie` names the engine and σ. [§4.7]
- `m4::tie_moves_with_workers`: N swept across the tie's range: as N rises σ falls from 1
  to 0 (more labour supply takes the more labour-using loom), and at the ends the tie meets
  the neighbouring regions' roots at 1e-12. [§4.7]
- `m4::single_crossing_per_region`. [Lemma 2]

**m5, interest and the technique (M5).**
- `m5::interest_selects_the_technique`: the ρ table; at ρ = 0.15 and 0.3 every output is
  bit-equal except u (the flow type's is 1 + ρ, though it prices nothing), the unused
  durable type's own outputs and the residuals that are maxima over types. [§4.2, R4]
- `m5::rho_zero_is_unique`: at ρ = 0 every switch of M4, M5 and m6's switch set lowers
  n_D. [Proposition, §5.5]
- `m5::multiple_equilibria_are_refused`: M5m gives `MultipleEquilibria` with three sign
  changes and its switch; the four f values are the goldens. [§5.3]

**m6, random economies.** SplitMix64 seeded 926 to 929 (1b's were 923-925); 1b's C5 table
for the scalars, segments and categories, and:

| draw | range |
|---|---|
| intermediate a_jl | 0 with probability 0.7 off the diagonal, 0.9 on it; else U(0.02, 0.3) |
| K | an integer in 1..4 |
| θ_k | type 0: 1; others 0 with probability 0.3, else U(0.5, 2) |
| recipe machines | 0 with probability 0.6, else U(0.02, 0.3) |
| recipe labour | 0 with probability 0.2, else U(0.01, 0.5) |
| recipe land | 0 with probability 0.2, else U(0.05, 1) |
| recipes | with probability 0.25 the operating recipe is zero, with 0.25 the build recipe |
| δ_k, J_k | U(0.05, 1); an integer in 1..5 |

A draw failing §3.2 is skipped and counted. Sets: (a) ρ = 0; (b) ρ ~ U(0, 0.1); (c) J_k
in 1..5 and ρ ~ U(0, 0.1) with categories' intermediate inputs forced on; (d) **the switch
set**: two task types, the second's build land set so that the crossing lies at a γ drawn
in the middle of the line, and N set, half the time, so that n_S at the switch lies between
the two one-sided n_D (a tie at ρ = 0, a tie or `MultipleEquilibria` at ρ > 0). Each set
takes 60 interior economies (ties count), within a cap of draws the build records. The
prototype's first 300 draws of sets (a)-(b) were 62% `BoundaryNoMargin`; the build may
narrow the land ranges to raise the interior share, and records the tallies. For each:
- `m6::identities`: income by Y·P_s, by Σp_jz_jY and by pᵀf, with interest; land,
  services and the user-cost rows; eq 11; the basket count; interest = Σρω_kV_kX_k.
  [§4.5-4.6]
- `m6::leontief_identities`: p = Ap + λv + b over every row, (I − Â)λ̃ = λ̂, f = (I −
  A^qᵀ)y, N_a = λ^qᵀy, T = b^qᵀy, all by multiplication. [SSRN eq 3-4, A.1]
- `m6::fork_and_bounds`: both fork forms, the bounds and the pair for every category
  (bought or not), and v/P_s ≤ v/B_s. [§4.4]
- `m6::closure_and_cheapest`: SSRN A.1's closure for the technique, and every other task
  type's delivered cost at least the technique's. [§4.3]
- `m6::residuals_recompute`: each residual equals its recomputation bit for bit, and each
  is nonzero somewhere. [§6]
- `m6::root_and_regions`: x* is the double where f_τ changes sign, or the tie's; f is
  nonincreasing on a grid in each region; the sign-change count equals the regime.
  [§5.3, §5.5]
- `m6::ties_and_multiplicity`: in the switch set, every tie's identities hold; at ρ = 0 no
  `MultipleEquilibria` and every jump downward; at ρ > 0 each `MultipleEquilibria` has an
  upward jump and an odd number of sign changes above one on re-evaluation. [§4.7, §5.5]
- `m6::the_draws_cover_the_regimes`: every regime, ties, `MultipleEquilibria`, unused
  task types, θ = 0 types and every build lag in 1..5 occur.

**m7, regimes, validation and reductions.**
- `m7::regimes`: M4's regime rows and their diagnostics; `NotViable` decided by the least
  pivot of the envelope's last technique. [§5.3]
- `m7::validation`: every rule of §3.2 is an error, and −0.0 is stored as +0.0.
- `m7::unused_type_changes_nothing`: adding a θ = 0 type that nobody uses, last, leaves
  every other output bit for bit, except the residuals that are maxima over types. [R1]
- `m7::duplicate_task_type`: a copy of the technique's type at a higher index is never
  chosen, and every aggregate is bit-equal, except the residuals that are maxima over
  types. [§5.2 tie-break]
- `m7::permutation`: reversing the types (and the machine columns of every recipe) gives
  the same outputs within 1e-12, permuted.
- `m7::unit_rescaling`: measuring type k's service in units of c (θ_k, its recipe rows
  and the columns that use it rescaled): at c = 4 every scaling is by a power of two and
  the solve is expected bit-equal, with p_k = 4·p_k (the build records it if not); at c = 3
  within 1e-12.
- `m7::envelope_edge_cases`: a switch exactly at γ(1), two crossings at one γ, a type
  viable only low on the line.

**m8, the goldens file.** `goldens_1c.txt`'s four digests recompute, and the Rust
constants match it to 20 digits.

**Unit tests** of `leontief.rs`: GE reproduces a 2×2 closed form, the identity and
diagonal cases exactly, the deferred division equals the plain one to 1 ulp, and the
pivots of I − A are positive exactly when ρ(A) < 1 on constructed matrices.

## 9. Out of scope, and where each goes

- Machine recipes that use categories (PLAN §3.1's build bundle): open question 1, a 1c
  addendum before Phase 2 if ruled in.
- Machine types with schedules of different shape: open question 2.
- Several non-produced inputs (B as a matrix, SSRN A.1's vector r), unused fixed inputs at
  zero rent, land quality and idle land: **1e**. In 1c the one land input is fully
  employed and r = 1.
- Worker types, human-required tasks in the equilibrium and the wall: **1d**.
- check_dynamics' land-share household, external finance and net exports (EJ1-EJ5), the
  participation invariances (P1-P4), the rental tax on new capacity (T4) and government:
  **1f**, where the land-share closure could be a named alternative household (R6), which
  would make the sloped x* and the flat m reproducible.
- Transitions, vintages out of steady state, Q (T1): **Phase 3**.
- Cells in the equilibrium: 1b's open question 2.
- A dump interface for machine types: no other multi-type solver to compare against.

## 10. Pitfalls

- **Two A's and two sets of totals.** Price-side totals use u (Â); clearing-side totals
  use δ (A^q). They agree only at ρ = 0. SSRN eq 12-13 and the closure use the first; eq
  11, N_a and T use the second.
- **V is not p/u.** V_k is the build cost (p_K); p_k = O_k + u_kV_k. For a flow type V = 0
  and it earns no interest.
- **θ is a productivity, not a cost.** Type k needs M/θ_k units of its service for M task
  units; its delivered cost is p_k/θ_k.
- **A tie is an equilibrium; a jump is not a root.** At a switch labour demand jumps, and
  labour clears by the split σ, not by a threshold.
- **Interest can make the equilibrium not unique.** The count refuses rather than chooses.
- **b and b_j.** b^op_k, b^I_k are a machine's land; b_j a category's direct land; b̄_j a
  category's chain land. 1b's direct-land bounds become chain-land bounds.
- **"M" groups.** 1c's test groups m1-m8 and instances M1-M6 are unrelated to the machine
  count K or the task quantity M_j.
- **γ is not inverted.** The oracle never computes γ⁻¹; switch points in x are found by
  bisection on γ(x) ≥ γ_i. Only the generator inverts the power schedule.

## 11. Open questions

1. **Machines built from categories.** 1c's machine recipes use machine services only
   (§2.3), as main.tex:688 and check_dynamics do. PLAN §3.1 builds capital from goods.
   Allowing it makes the machine totals depend on x and on the technique; the technique is
   still the least delivered cost (the non-substitution argument), but Lemma 1 and the
   closed-form envelope go. Rule it into a 1c addendum before Phase 2, or leave it?
2. **One shape per line.** Types differ by a task efficiency θ_k on a common γ (§2.4).
   That keeps one threshold, 1a's unknown and precision, and the envelope. Distinct shapes
   need v as the unknown and several thresholds (1b §2.3's loss). Accept?
3. **Ties as `Interior`.** 1c solves a tie and reports it inside `Interior` with `tie`
   set. Should it be its own regime? That would change `Regime`, a 1a type.
4. **Multiple equilibria refused.** With interest they exist (M5m). The oracle could
   return all of them (one per sign change, computable in the same pass). PLAN §3.4's CI
   comparison needs to know which one the agents should reach; the tie at an upward jump
   is presumably unstable. Refuse, or report all?
5. **Bitwise nesting** fixes the formulation and association of §5.1, including the
   (O, V) system and the deferred division. The fallback is 1e-12 with per-instance
   bounds near the viability edge. Keep bitwise, as 1b's open question 4?
6. **Physical productivity** is validated as ρ(A^op + A^I) < 1 (SSRN's assumption; 1a's
   a < 1). It rejects some economies whose price side is viable with u < 1. Keep?
7. **API changes to 1a's module**: `classify` split into the regime tests and the
   bisection, and `SolveError` gains `NonFiniteInType` and `MultipleEquilibria`.
8. **The random draws** (§8, m6) are constructed and the interior share is low with the
   prototype's ranges; the build tunes and records them. Acceptable, or fix the ranges now?

## 12. Changes during the build (P1.4, 2026-09-27)

1. **The least pivot.** `d` is the first pivot of the elimination that is not positive,
   NaN included, and the least pivot only when all are positive: past a nonpositive pivot
   the elimination divides by it and its later pivots mean nothing (a zero pivot makes
   them infinite or NaN, and the plain minimum would skip a NaN). For one type the pivots
   are 1.0 and D, and both definitions give D, so 1a's `d_at_1` nests bit for bit, D(1) = −∞
   included. M2's sloped schedule shows the difference at γ(1) = 5: the operating row alone
   is at its edge, 1 − a − λγ = 0, and d(1) = 0.0 exactly, where Den is negative.
2. **A tie's price-side totals take the split.** The draft priced the categories' chain
   totals through the type below the switch. The generator's first run found b̃^q_j > b̃_j at
   M4t: the clearing side takes the engine's share σ = 0.92, whose land per task is above
   the loom's. At a tie either type's decomposition of the price holds, and so does any mix;
   the split's keeps §4.4's chain b̄ ≤ b̃^q ≤ b̃ ≤ p. The `leontief_price` residual and the
   tests' p = Ap + λv + b use the split's columns too.
3. **Result types.** `Eq1c::outputs` keys are `OutputKey1c { item, name }` with
   `Item::{Economy, Switch(i), Type(k), Category(j)}`, printed `name`, `switch<i>.name`,
   `type<k>.name` and `cat<j>.name`: 1b's `OutputKey` has no place for a type. The tie is
   the outputs `tie` (a flag), `tie_above`, `tie_share` and `tie_gamma` (optional numbers;
   `tie_above` is the type's index as a number). The block's `Switch` holds γ_i and the two
   types; `Eq1c::switches` holds `SwitchPoint`s, which add x_i. `MachineBlock::envelope`
   returns `Envelope { first, switches }`, with `technique_at(γ)` and `last()`.
   `MachinePoint`'s per-type prices are `type_prices` (the categories' keep 1b's `prices`).
   `bisection_steps` is 0 at a tie.
4. **Validation names.** A machine type's parameters are `ParamError::Item { kind:
   "machine type", index, .. }` with the names `task_efficiency`, `operating.machines`,
   `operating.labor`, `operating.land`, the `build.` ones, `delta`, `build_lag` and `u`;
   a type whose chain reaches no land is the same item with `Invalid { name: "land" }`.
   No types, no task type, and recipes that are not productive are `Invalid { name:
   "machine types" }`; the intermediate inputs' shape, range and productivity are
   `Invalid { name: "intermediate" }`. The validation order is 1b's scalars (N, T, the
   schedule, χ_max, ρ), then the machine types, then 1b's task line and categories, then the
   intermediate inputs, the categories' pricing through the chain and the basket.
5. **The corners of M3** (§3.3) are 1a economies as the draft said, but the operating-free
   one, build (0.1, 0.2, 0.02) at (0.05, 0.1, 3) on γ = 1 + 2x, is `BoundaryNoMargin` in
   both units; the generator compares the regime and f(1), and `m3::corners` their bits.
6. **The random draws** (§8, m6) keep §8's table as written, with labour U(0.01, 0.5):
   narrowing it to U(0.01, 0.1) left no `NotViable` draw, which the coverage test needs.
   Set (c) forces one input, a_10 ~ U(0.02, 0.3), when the table draws none off the
   diagonal. The switch set (d) follows the prototype's scan: ρ = 0 on even draws and
   U(0.02, 0.12) on odd ones; γ = 1·(g0 + g1·x) with g0 ~ U(0.1, 0.5), g1 ~ U(0.5, 2);
   h ~ U(0.1, 1.5), N ~ U(1, 20), T ~ U(5, 20), χ_max ~ U(0.5, 3); type 0 with θ 1,
   operating (λ ~ U(0.05, 0.4), b ~ U(0.1, 0.8)) and build (λ ~ U(0, 0.2), b ~ U(0, 0.3)),
   δ ~ U(0.2, 1), J in 1..3; type 1 build-only with θ ~ U(0.5, 2), λ_I ~ U(0, 0.05),
   δ ~ U(0.02, 0.3), J in 1..6, and its build land set so that the two closure wages are
   equal at γ(x) for x ~ U(0.15, 0.85); a draw whose type 1 is not the flatter is skipped;
   and half the time N is set so that n_S at the switch is midway between the two n_D. The
   tallies (draws, invalid, interior, of which ties, `BoundaryNoMargin`, `NotViable`,
   `NoInteriorAtZero`, `MultipleEquilibria`):

   | set | draws | invalid | interior | ties | boundary | not viable | none at 0 | multiple |
   |---|---|---|---|---|---|---|---|---|
   | (a) ρ = 0 | 240 | 34 | 60 | 0 | 142 | 4 | 0 | 0 |
   | (b) ρ > 0 | 242 | 50 | 60 | 0 | 128 | 3 | 1 | 0 |
   | (c) chains | 228 | 36 | 60 | 0 | 126 | 5 | 1 | 0 |
   | (d) switches | 65 | 0 | 60 | 35 | 0 | 0 | 0 | 5 |

   Every `MultipleEquilibria` is at ρ > 0 with an upward jump and three sign changes; at
   ρ = 0 every switch lowers labour demand. `MAX_DRAWS_1C` is 2000.
7. **Tests.** `m1::regime_rows_nest` also runs 1b's fork-economy rows and N 6;
   `m1::every_1b_golden_instance_is_bit_identical` adds the sliver economy's near-edge root.
   `m6::leontief_identities` uses its own check (`support_1c::check_leontief_1c`), apart from
   `check_identities_1c`. `m7::unused_type_changes_nothing` and `duplicate_task_type` except
   the three residuals that are maxima over types (services, user cost, Leontief price);
   the duplicate is of a type without machine inputs (M5's), whose totals are then its
   original's bit for bit, as the tie-break needs. `m4::tie_moves_with_workers` compares the
   tie with the neighbouring roots 2e-13 of N away at 1e-12, the types' services against the
   economy's total (the engine's own are tiny as σ → 0). `m5::multiple_equilibria_are_refused` recomputes the tie's σ at the
   switch from both techniques.
8. **The algorithm.** Gaussian elimination without pivoting, in index order, for every
   system (`src/leontief.rs` states why): on these M-matrices it is stable, its positive
   pivots are the productivity and viability tests, and its fixed order is what makes the
   nesting bitwise. Partial pivoting would reorder rows by the data; a Neumann series is slow
   near the edge and not exact at A = 0.
9. **Numbers.** The generator reproduces all of §7's values to the digits shown and writes
   210 goldens; the one-type form nests `generate_1b.py`'s and `generate.py`'s solves within
   6.1e-71. On 2026-09-27 the oracle's f64 values matched 191 of the 207 goldens it computes
   within 1.2e-15 relative and 199 within 1e-14. The rest are where §5.6 expects: the switch
   points (2.8e-14 at M5's, whose closed form cancels 0.52 against 0.50), a least pivot near 0
   (1.7e-14 at M4's η = 50), the tie's quantities (2.0e-14), and M5m's excess demand at the
   switch, evaluated at the double below it where f is steep (4.8e-13 on f, 4.4e-13 on σ).
   Unit 1c adds 50 gate tests and 9 unit tests: 232 in the package.
10. **1a and 1b.** Their behaviour and their tests are unchanged. `solve::classify` is now
    `regime_tests` and `bisect` (both crate-private, with `Root::exact`), and 1b's line and
    category checks, `tasks` and `segment_of` are free functions that 1c shares; their test
    files only make the helpers that m1 reuses visible (`golden_instances`, 1b's C5 draw,
    `with_eta`, `sliver_economy`).

### Verification (2026-09-27)

11. **An independent derivation** that does not read the crate or the generator solved 348
    economies from a scratch probe of the oracle (the named instances, M4t and M5 across N,
    and 400 random draws of which 73 were invalid) in another formulation: the whole price
    system with the task margin as one more row, the technique as the viable argmin of v with
    viability by eigenvalues, the whole clearing system, switches located where the argmin
    changes, and a tie's σ by root-finding. It agreed on every regime, technique, tie and
    switch count, and on the values within 5.6e-14 relative: x*, v, P_s, Y and every price
    within 2.4e-15, 1 − x* within 9.7e-15, N_a 1.5e-14, the switch points 2.3e-14, the tie's
    services 4.3e-14 and a boundary diagnostic 5.6e-14.
12. **Mutation testing**: 65 mutants of `leontief.rs`, `machine_block.rs`, `machines.rs` and
    the goldens; six survived, and five are now killed by the tests below. The sixth, a
    crossing at exactly γ_c counting (`gamma_c <= crossing`), is equivalent in exact
    arithmetic: a type crossing the technique exactly at γ_c and cheaper above it would have
    won that point's tie-break (§5.2), so the two differ only where rounding separates a
    crossing from its tie.
13. **Tests added**, each killing its mutant: `m7::envelope_edge_cases` now has a crossing
    beyond both types' viability, which must not count (the viability check on a crossing);
    `m7::unit_rescaling` has a flow economy (u = 1) whose type is measured in units of 4, so φ_w =
    γλ̃_τ/θ_τ must not move (φ's θ); `m7::phi_only_when_every_type_is_flow` adds an unused type
    with u ≠ 1, which removes φ everywhere (φ reported when the technique alone has u = 1);
    `m7::a_category_bought_only_as_an_input_makes_a_margin` puts the root on a segment where
    only a category outside the basket, used by a bought one, has tasks (`margin_active` by
    ŷ, not z); and the unit test `machines::a_tie_share_is_one_when_the_type_above_does_not_clear_labour`
    (σ = 1 when f_b ≥ 0). `check_identities_1c` now checks φ wherever it is reported:
    γλ̃_τ/θ_τ = vλ̃_τ/p_τ, φ_r = 1 − φ_w, and each type's vλ̃_k/p_k. The code is unchanged.
    The package has 235 tests: 57 unit, 177 gate and 1 doc.
