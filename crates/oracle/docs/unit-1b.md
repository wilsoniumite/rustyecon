# Unit 1b: many categories and the fork

Dated 2026-09-27. This is the unit's specification, written before any code and amended
where the build of the same day (P1.2) departed from it; §12 lists each change. The first
draft quoted a 70-digit scratch prototype (not kept). `goldens/generate_1b.py` reproduces
all 205 of its numbers, and every number below is the generator's (§7).

- **Pinned to:** laborformal `31b3482`. Every laborformal path and line below is at that
  commit.
- **Paper:** SSRN 7226858 (posted 2026-09-23), cited as "SSRN p.N (eq k)".
- **Older revision:** `pinning/paper/main.tex` (2026-09-21), cited as "main.tex:N". Its §6
  (main.tex:433-511) states the fork in category form. Its Appendix B is not used
  (unit-1a.md §0).
- **Reference checks:** `pinning/checks/check_interior.py`, `cost_and_accounting_checks`
  :10-59 and `replacement_and_interior_formula_checks` :128-171;
  `pinning/checks/check_fan.py` F1-F6 (:46-108); `dynamics/checks/check_dynamics.py`
  F1-F4 (:421-440); `paths/checks/check_macro.py` C3 (:77-80) and B1 (:86-88).
- **Gate (PLAN Phase 1):** the fork identity and the category bounds on random instances;
  the income identity to 1e-12; 1a's results as the one-category case exactly (R1).
- **Built on:** unit 1a (docs/unit-1a.md). Every 1a equation, convention, constant and
  precision rule carries over unless this document says otherwise.
- **Goldens:** a new generator, `goldens/generate_1b.py`, writing `goldens/goldens_1b.txt`
  (§7).

## 0. What the sources give

| source | what 1b takes from it |
|---|---|
| SSRN §3.2, p.8 (eq 2-4) | the cost system p = Ap + λw + br, the totals λ̃ = (I − A)⁻¹λ and b̃ = (I − A)⁻¹b, and p_i = wλ̃_i + rb̃_i. A directly rented service is a row with p_h = r, no labour or intermediate inputs, and b_h = 1 |
| SSRN §4-5, pp.10-12 (eq 7, 10, 11) | P_s = zᵀp; "suppose all households consume in the basket's proportions"; L_s = zᵀλ̃, B_s = zᵀb̃, P_s = wL_s + rB_s; n_D = T·L_s/B_s |
| SSRN §6.1, p.13 (Prop 4, eq 12-13) | p_i = wλ̃_i + rb̃_i and w/p_i = 1/(λ̃_i + b̃_i·r/w); the purchasing-power pair 1/(L̄_i + b̄_i·r/w) ≤ w/p_i ≤ (w/r)/b̃_i when b̃_i > 0; w/p_i ≥ 1/L̄_i when b̄_i = 0; both hold at boundary assignments too |
| SSRN p.14 | the corollary: w/r → 0 sends w/p_i → 0 for every good with b̃_i bounded away from 0; the proof's P_s ≥ rB_s |
| SSRN A.1, pp.24-25 | the general cost system; unused methods cost at least the price; a direct household rental is a pass-through row; clearing f = (I − Aᵀ)y, N_a = λᵀy, T = Bᵀy |
| SSRN A.2, p.27 (eq 19) | rb̃_i ≤ p_i ≤ wL̄_i + rb̄_i |
| SSRN A.3, p.27 (eq 20) | p_i = rb_i + w∫_{X_i}(1/γ_Li)·min{1, γ_i(x)/γ(x*)}dx; L̄_i = ∫dx/γ_Li and b̄_i = b_i for the all-human method; the uniform-capability parity remark |
| SSRN App. C, pp.30-31 (eq 26) | I = pᵀf = wN_a + rT; the CES expenditure share α(q), "conditional on the price path" |
| main.tex:441-511 | the category form: direct b_j, L̄_j (:445), L_j* (eq effective-hours, :450), p_j = wL_j* + rb_j (eq composites, :459), rb_j ≤ p_j ≤ rb_j + wL̄_j (eq category-bounds, :465), the pair (eq fork-pair, :469), b_j = 0 (:474), the proof by task-by-task minimum (:477-490), the corollary (:491-504), the flat case (:506-509) |
| check_interior.py | two random batteries of the price identity, the bounds and the income accounting, and an exact-parity instance (§8, C6) |
| check_fan.py F1-F6, check_dynamics.py F1-F4 | symbolic: parity for any machine rental, b_j = 0 invariance under the interest closures, the crossover at r/w = L̄/b |
| check_macro.py C3 (:77-80), B1 (:86-88) | the basket's rent bound w/P_s ≤ v/B_s along a path; the goods-versus-shelter inflation fork (macro.py:208-209) |

**There is no multi-category equilibrium instance at the pin.** `macro.py`'s two
"categories" are Appendix B's good and space (macro.py:112, 208-209). The gate's
multi-category part is therefore identities on random instances, and every
multi-category golden is constructed here.

**The fork has two exact forms.** SSRN writes it with total requirements (λ̃_i, b̃_i) and a
rent floor rb̃_i; main.tex writes it with the direct requirement b_j and the wage-equivalent
task cost L_j*. They agree: L_j* = λ̃_j + (b̃_j − b_j)·r/w. SSRN's floor rb̃_i is the
tighter; main.tex's rb_j is the one that stays put when machine supply chains change. 1b
reports and tests both.

## 1. Scope

**In scope:**
- C categories (final goods and services), each with a direct land requirement b_j ≥ 0
  and a task list on the economy's task line;
- the households' basket z over categories, fixed in proportions (§2.1);
- space as a category: the pass-through row with no tasks and b = 1;
- one machine type, one land input, one worker type, durability and interest through 1a's
  scalar user cost u;
- the fork identity, the category bounds and the purchasing-power pair, at the equilibrium
  and at any margin (the price block);
- the price block for categories whose tasks are a finite list of cells with arbitrary
  human and machine productivities, closed cells included (price block only, §2.2);
- the CES expenditure share of SSRN eq 26 (price block only).

**Out of scope** (§9 says where each goes): category-to-category intermediate inputs,
several machine types and separate build and operating recipes (1c); worker types,
human-required tasks in the equilibrium and the wall (1d); several non-produced inputs,
land quality and idle land (1e); households' demand beyond the fixed basket, and
government (1f); cells in the equilibrium (open question 2).

## 2. Decisions

### 2.1 Categories enter the equilibrium through a fixed basket

Every household consumes categories in the fixed proportions z = (z_1, …, z_C), and the
provider's support is one basket z. This is the paper's closure: SSRN eq 7 (P_s = zᵀp),
§5's "suppose all households consume in the basket's proportions" (p.12), and Appendix B's
z = (1, 0, h) (p.29). With it:
- eq 10-11 hold as written, with L_s and B_s the basket's totals;
- Lemma B.1's argument carries over (§5.3): labour demand is nonincreasing in the threshold
  and v/P_s strictly increasing, so the root is unique;
- z = (1, h) over (good, space) is Appendix B exactly, so 1a nests bit for bit (§5.1).

A price-responsive basket (CES over categories, or the agents' demand around a subsistence
basket, PLAN §3.2) is not a closure in 1b. With it, z moves with relative prices, labour
demand need not be monotone, and uniqueness can fail. It belongs to 1f, where households
are specified; a CES basket would nest at σ = 0.

### 2.2 What is price-block only

These hold at any margin and need no equilibrium solve. 1b computes and tests them on
their own and at every solved equilibrium:
1. The fork identity, the category bounds and the purchasing-power pair (SSRN eq 12, 13, 19;
   main.tex:459-474). They are cost identities: they need a margin w/p_m and the machine
   row, not market clearing.
2. Categories whose tasks are cells: a finite list with arbitrary γ_L and γ_M per cell and
   cells closed to machines (γ_M = 0). This is check_interior's setting and PLAN §3.1's
   recipe. The equilibrium takes only the task line of §2.3.
3. The uniform-capability parity remark (SSRN A.3; main.tex:506-509): it needs a flat γ,
   which the equilibrium's schedule may not be.
4. The CES share α(q) of SSRN eq 26. The paper uses it as a comparison "conditional on the
   price path" (p.31), not as a closure, and so does 1b: it is evaluated on prices and
   feeds nothing back.

The equilibrium part is the fixed basket, land and service clearing, and labour clearing
(§4.3-4.4).

### 2.3 Tasks: one line shared by all categories

The economy has one task line [0, 1] with 1a's schedule γ(x) (γ_L = 1, γ_M = 1/γ). It is
cut into segments 0 = e_0 < e_1 < … < e_S = 1. Category j uses μ_js ≥ 0 hours of the
tasks on segment s per unit of its output per unit length of line: the "task quantities
included in the integration measure" of main.tex:447. Machines do the tasks below the
threshold x in every category, people those above.

Why this and not a schedule per category (main.tex's γ_j):
- **Nesting.** The unknown stays 1a's threshold x on [1e-12, 1], so the one-category case
  runs 1a's bisection, bracket and all, and nests bit for bit.
- **Precision.** 1a keeps 1 − x* to full relative precision and never inverts γ. With
  per-category schedules the natural unknown is the margin g = w/p_m, and each category's
  threshold is γ_j⁻¹(g). By a first-order estimate (not measured), where γ_j is flat
  (k = 6, g0 = 0.5, g1 = 0.2, x = 0.1) x_j loses about 4e-11 relative through the inverse,
  and near full automation (G3 at η = 1e-6) 1 − x_j loses about 1e-9, because g's own
  spacing of doubles is then large against γ_j(1) − g. Both miss the gate.
- **Generality.** Any per-category task list sorted by relative capability is a density on
  a common capability axis. Piecewise-constant densities approximate it as finely as
  wanted, exactly in the limit. Categories differ in which tasks they need and how much
  (durables low on the line, care high), which is what the fork turns on.
- **Forward.** 1c's machine types each get a schedule on the same line; 1d's worker types
  likewise.

What the line cannot say: two categories whose versions of the same task have different
relative capability. The cell price block (§4.5) covers that fully, and cells in the
equilibrium are open question 2.

The margin is **active** when some basket category (z_j > 0) has tasks at x*
(Σ_j z_j·μ_{j,s(x*)} > 0, with s(x*) the segment holding x*). Otherwise x* lies in a
**gap**: no produced task is at parity, and labour clearing, not a task, sets w/p_m =
γ(x*). The equilibrium is still unique and competitive. 1b solves it and flags it (open
question 3).

## 3. Parameters, validation and instances

### 3.1 Parameters

1a's parameters (unit-1a.md §2) stay, with the same names, bounds and instance values,
except `space` (h), which becomes a category. New:

| symbol | name in code | meaning |
|---|---|---|
| e_0 … e_S | `edges` | the segments of the task line; e_0 = 0 and e_S = 1 exactly, strictly increasing |
| z_j | `weight` | units of category j per basket |
| b_j | `direct_land` | land services per unit of category j, used directly (acreage, sites) |
| μ_js | `density` | hours by hand on segment s per unit of j per unit length of line (S values) |

Space is the category with μ = 0 and b = 1 (SSRN p.8's pass-through row), and its weight is
1a's h.

### 3.2 Validation

On top of 1a's checks (in 1a's order and with 1a's names, less h), with the same error
type:
- `edges`: at least two, first exactly 0.0, last exactly 1.0, strictly increasing, finite;
  and J(0) = 0 exactly, which §5.1's nesting argument uses;
- at least one category; each `density` has S entries;
- `weight`, `direct_land` and every `density` entry lie in {0} ∪ [`SCALE_FLOOR`,
  `SCALE_CEIL`], and −0.0 is stored as +0.0 (1a §2's rule);
- each category has L̄_j > 0 or b_j > 0, so it is positively priced (SSRN Prop 4's
  premise);
- the basket uses land directly, B_d = Σ z_j·b_j > 0, the analogue of 1a's h > 0. It makes
  v/P_s strictly increasing (§5.3) and n_D(0) = T·L̄_s/B_d finite;
- the basket needs work, L̄_s = Σ z_j·L̄_j > 0.

Viability keeps 1a's convention: D(1) = 1 − u(a + λγ(1)) at the top of the line must be
positive, even when no basket category uses the top segment (open question 5).

### 3.3 Instances

**C1, Appendix B in category form.** 1a's G1 parameters; edges (0, 1); categories good
(z 1, b 0, μ = (1)) and space (z = h = 1, b 1, μ = (0)).

**C3, the fork economy** (constructed 2026-09-27). 1a's G1 scalars (N 4, T 10, a 0.3,
λ 0.05, b 0.4, γ = 0.2 + 0.8x, χ ~ U[0, 1], (ρ, δ, J_b) = (0, 1, 1)); edges
(0, 0.4, 0.75, 1):

| j | category | z_j | b_j | μ on [0, 0.4) | μ on [0.4, 0.75) | μ on [0.75, 1] | L̄_j |
|---|---|---|---|---|---|---|---|
| 0 | manufactures | 0.3 | 0 | 2 | 0 | 0 | 0.8 |
| 1 | food | 1 | 0.6 | 0.5 | 1.5 | 0 | 0.725 |
| 2 | care | 0.2 | 0.1 | 0 | 0 | 1 | 0.25 |
| 3 | shelter | 0.8 | 1 | 0 | 0.4 | 0 | 0.14 |

B_d = 1.42 and L̄_s = 1.127. Its variants: **C3d**, the same at (ρ, δ, J_b) =
(0.04, 0.35, 2), u = 0.4056 (1a's G5 durable lag); **C3z**, at (0, 0.1, 1) against the flow
economy with (a, λ, b) = (0.03, 0.005, 0.04); **C4**, task automation η ∈ {1, 0.5, 0.25,
0.1, 0.03} and recursive automation λ ∈ {0.05, 0.025, 0}.

**The gap economy** (constructed; C7's goldens). As C3 with N 5 and edges (0, 0.4, 0.6, 1), the
middle segment unused: manufactures (z 0.3, b 0, μ (2, 0, 0)), food (1, 0.6,
(0.5, 0, 0.2)), care (0.2, 0.1, (0, 0, 0.3)), shelter (0.8, 1, (0, 0, 0.1)). For N between
about 4.25 and 5.3 the root lies in [0.4, 0.6): at N 4.25 it is 0.593, and at N 5.4 it has
left the gap (0.39979).

**C6, the price-block batteries.** check_interior's two random batteries and its parity
instance (§8).

**Random multi-category economies** (C5, §8).

## 4. Equations

All quantities are per period, r = 1 and v = w/r, as in 1a. x is a candidate threshold on
the task line. Space is an ordinary category, so no equation has an h.

### 4.0 User cost

u = (ρ + δ)(1 + ρ)^(J_b − 1), unchanged (unit-1a.md §3.0).

### 4.1 Task quantities per category

Source: main.tex:445-455 and the proof at :477-490; SSRN A.3, p.27 (eq 20). On the line,
γ_j = γ on category j's tasks, 1/γ_Lj = μ_js on segment s, and machine services per unit
of task are μ_js·γ(t).

    L̄_j    = Σ_s μ_js (e_s − e_{s−1})                                       hours by the all-human method
    H_j(x) = Σ_s μ_js·|[e_{s−1}, e_s] ∩ [x, 1]|                             hours at human tasks, per unit of j
    M_j(x) = Σ_s μ_js·(J(clamp(x, e_{s−1}, e_s)) − J(e_{s−1}))              machine services, per unit of j
    L_j*(x) = H_j(x) + M_j(x)/γ(x)                                          wage-equivalent task cost (eq effective-hours)

L_j* is main.tex's ∫(1/γ_Lj)·min{1, γ(t)/γ(x)}dt in closed form. It is not a count of
hours (main.tex:455). The prototype checked the closed form against mpmath quadrature of
that integral on C3, within 5e-72, and `generate_1b.py` asserts it to 1e-60. Appendix B's good is one segment with μ = 1, so
H = 1 − x, M = J(x) and L̄ = 1 (SSRN p.28).

### 4.2 Price block

    D, V_m, p_m, v                        1a §3.1 unchanged, with γ = γ(x)
    p_j   = v·H_j + p_m·M_j + b_j         each task at its cheaper method (main.tex:477-490; SSRN eq 20)

**The fork identity** (main.tex eq composites, :459; SSRN eq 12, p.13):

    p_j = v·L_j* + b_j,        v/p_j = 1/(L_j* + b_j/v)

The equality of the two lines above is v/γ(x) = p_m, the task margin.

**Total requirements, price side** (SSRN eq 4 with the machine row scaled by u, A.4):

    λ̃_m = uλ/(1 − ua),   b̃_m = ub/(1 − ua)            so p_m = v·λ̃_m + b̃_m (the machine row)
    λ̃_j = H_j + M_j·λ̃_m,  b̃_j = b_j + M_j·b̃_m
    p_j = v·λ̃_j + b̃_j                                   SSRN eq 12
    L_j* = λ̃_j + (b̃_j − b_j)/v                          the two forms agree

At u = 1 these are eq 4's totals, and the good's are 1a §3.5's. With u ≠ 1 they carry the
interest in the machine row; they are price-side objects, not physical requirements.

**The category bounds** (SSRN eq 19, p.27; main.tex eq category-bounds, :465):

    b_j ≤ b̃_j^q ≤ b̃_j ≤ p_j ≤ v·L̄_j + b_j,        L_j* ≤ L̄_j

b̃_j^q is the clearing-side total of §4.3. The chain uses δ ≤ u, which holds because
ρ ≥ 0: t ↦ t·b/(1 − t·a) is increasing. SSRN's upper bound wL̄_i + rb̄_i is main.tex's,
since b̄_j = b_j for the all-human method (SSRN A.3). A category with all its tasks above x
(M_j = 0, H_j = L̄_j) attains the upper bound exactly.

**The purchasing-power pair** (SSRN eq 13, p.13; main.tex eq fork-pair, :469, and :474):

    1/(L̄_j + b_j/v) ≤ v/p_j ≤ v/b̃_j ≤ v/b_j      the middle when b̃_j > 0, the last when b_j > 0
    b_j = 0:  v/p_j ≥ 1/L̄_j
    L̄_j = 0 (space): v/p_j = v/b_j, both ends attained

**The basket** (SSRN eq 7, p.10; §5, p.12; the proof on p.14; check_macro C3):

    P_s = Σ_j z_j·p_j = v·L_s + B_s = v·L_s* + B_d
    L_s = Σ z_j λ̃_j,  B_s = Σ z_j b̃_j,  L_s* = Σ z_j L_j*,  B_d = Σ z_j b_j
    v/P_s ≤ v/B_s

### 4.3 Quantity block (clearing side)

Source: SSRN eq 23-24 and p.29's matrix display, generalised as in 1a §3.2 (macro.py:76-91
for the δ terms).

    s_K = 1 − aδ
    H_s = Σ z_j H_j,   M_s = Σ z_j M_j,   B_d = Σ z_j b_j
    Y   = T/(B_d + bδ·M_s/s_K)                     land clearing
    K   = Y·M_s/s_K                                services: K = Y·M_s + aδK
    final_hours = Y·H_s,  machine_hours = λδK,  n_D = final_hours + machine_hours
    n_S = N·F(ln(1 + v/P_s))                       SSRN eq 10
    y_j = z_j·Y                                    category outputs (goods clear by the basket)

Clearing-side totals: λ̃_j^q = H_j + M_j·λδ/s_K and b̃_j^q = b_j + M_j·bδ/s_K, with
L_s^q = Σ z_j λ̃_j^q and B_s^q = Σ z_j b̃_j^q. Then Y = T/B_s^q and n_D = T·L_s^q/B_s^q
(SSRN eq 11) hold at every u. When u = δ (ρ = 0) the two sides' totals are equal.

With z = (1, h), the good (μ = (1), b = 0) and space (μ = (0), b = 1), §4.1-4.3 are 1a
§3.1-3.2 term for term.

### 4.4 Equilibrium and outputs

Find x* in (0, 1) with n_D(x*) = n_S(x*); N_a = n_D(x*). The outputs are 1a §3.4's, with
h → B_d and J → M_s:

    interest = ρ·W_K,  I = v·N_a + T + interest,  Y·P_s = I = Σ_j p_j·y_j     SSRN App. C, p.30
    labor_share, capital_share, real_wage = v/P_s, participation, support_cost, worker and
    provider baskets, funded, lemma_b1 (eq 25 as written)                      1a §3.4

Per category j: p_j, v/p_j, L_j*, L̄_j, H_j, M_j, λ̃_j, b̃_j, λ̃_j^q, b̃_j^q, y_j, the
final hours y_j·H_j and machine services y_j·M_j, the expenditure share z_j·p_j/P_s, the
bounds' ends (1/(L̄_j + b_j/v), v/b̃_j) and, at u = 1 only, φ_w,j = v·λ̃_j/p_j and
φ_r,j = b̃_j/p_j (three-taxes T1 per category; 1a §3.4's reason for u = 1).

Basket-level additions: L_s*, B_d, L_s, B_s, L_s^q, B_s^q, the rent ceiling v/B_s, and
`margin_active` (§2.3).

### 4.5 Price block on its own

**Cells** (SSRN A.3 with a discrete measure; check_interior.py). A category is b_j and a
list of cells c, each with a weight q_c ≥ 0 (tasks per unit of output), a human
productivity γ_Lc > 0 and a machine productivity γ_Mc ≥ 0, where 0 means closed to
machines (SSRN §3.1's H). Given w, p_m and r, and g = w/p_m:

    cell c is human when γ_Mc = 0 or w/γ_Lc ≤ p_m/γ_Mc (ties to people, check_interior:27), else machine
    p_j = r·b_j + Σ_c q_c·min{w/γ_Lc, p_m/γ_Mc}                        task by task
    L̄_j = Σ_c q_c/γ_Lc,  H_j = Σ_human q_c/γ_Lc,  M_j = Σ_machine q_c/γ_Mc
    L_j* = Σ_c (q_c/γ_Lc)·min{1, γ_c/g},  γ_c = γ_Lc/γ_Mc (∞ when closed)

Here the fork identity p_j = w·L_j* + r·b_j is a real check: the price is summed from task
costs and L_j* from capability ratios. Given the machine's totals (λ̃_m, b̃_m) with
p_m = w·λ̃_m + r·b̃_m, the totals λ̃_j = H_j + M_j·λ̃_m and b̃_j = b_j + M_j·b̃_m give
SSRN eq 12. The bounds and the pair are those of §4.2.

**Accounting for any outputs** (SSRN App. C; check_interior:36-49). At u = 1, for any
category outputs y_j ≥ 0 and a direct household rental T_h:

    X = Σ_j y_j·M_j/(1 − a),  hours = Σ_j y_j·H_j + λX,  land = T_h + Σ_j y_j·b_j + bX
    I = r·T_h + Σ_j p_j·y_j = w·hours + r·land,     r·land/I = 1/(1 + v·hours/land)

This is the income identity as a cost-system fact, before any market clears.

**Uniform relative capability** (SSRN A.3, p.27; main.tex:506-509; check_fan F1, F4;
check_dynamics F4). If every cell has γ_c = γ̄ and g = γ̄, then L_j* = L̄_j, p_j =
w·L̄_j + r·b_j, and when b_j = 0, w/p_j = 1/L̄_j for any machine rental, so for any u.

**The CES share** (SSRN eq 26, p.31). For weight α ∈ (0, 1), elasticity σ ≥ 0 and
q = r/p > 0:

    α(q) = α^σ q^(1−σ) / ((1 − α)^σ + α^σ q^(1−σ)) = 1/(1 + ((1 − α)/α)^σ · q^(σ−1))

The code uses the second form, which cannot overflow: an infinite t gives 0 and a zero t
gives 1. σ = 1 gives α; as q → ∞ the share tends to 1 for σ < 1 and to 0 for σ > 1.
Along a path where the good has a human method with b̄ = 0, r/p ≥ 1/(v·L̄) (SSRN p.31), so
v → 0 sends q → ∞.

### 4.6 The cost system with categories (bridge to 1c)

Order the rows (categories 1…C, machine services). On the price side: A has M_j in row j's
machine column and u·a in the machine row; λ = (H_1, …, H_C, uλ); b = (b_1, …, b_C, ub).
Then p = Ap + λv + b holds row by row (SSRN eq 3), which the tests check by
multiplication, not inversion. At ρ = 0 (u = δ), with gross outputs y = (Y·z, K):
f = (I − Aᵀ)y = (Y·z, 0), λᵀy = N_a, bᵀy = T and pᵀf = v·N_a + T (SSRN A.1 and App. C).
Categories that use each other's outputs (nonzero category-to-category entries of A) are
not in 1b (§9).

## 5. Solution method

### 5.1 Steps

1a §4 steps 1-5, unchanged, on the task line: validate and compute u; `NotViable` when
D(1) ≤ 0; `BoundaryNoMargin` when f(1) ≥ 0; `NoInteriorAtZero` when f(1e-12) ≤ 0; bisect
f = n_D − n_S on [`BRACKET_LO`, 1] until lo and hi are adjacent doubles; carry 1 − x*
from the straight line through them; report residuals; refuse an `Interior` result whose
labour residual exceeds `LABOR_RESIDUAL_NET`·N_a. No constant changes: the bracket and its
doubles are 1a's, so `MAX_BISECTION_STEPS` still covers it. The only change is how f(x) is
evaluated.

**The evaluation order is normative**, so that the one-category case repeats 1a's
floating-point operations exactly (§8, C1):

1. γ = γ(x); D = `viability`(u, a, λ, γ); V_m = b/D; p_m = u·V_m; v = γ·p_m (1a's order).
2. For each category in order, H_j = M_j = 0.0, then for each segment in order: if
   x ≥ e_s, M_j += μ_js·(J(e_s) − J(e_{s−1})); else if x ≤ e_{s−1}, H_j +=
   μ_js·(e_s − e_{s−1}); else H_j += μ_js·(e_s − x) and M_j += μ_js·(J(x) − J(e_{s−1})).
   Then p_j = v·H_j + p_m·M_j + b_j.
3. Starting from 0.0, in category order: P_s += z_j·p_j, H_s += z_j·H_j, M_s += z_j·M_j,
   B_d += z_j·b_j.
4. s_K = `fma`(−a, δ, 1); Y = T/(B_d + b·δ·M_s/s_K); K = Y·M_s/s_K; final_hours = Y·H_s;
   machine_hours = λ·δ·K; n_D = final_hours + machine_hours; n_S = N·F(`ln1p`(v/P_s)).
5. At the root, prices (p_j, P_s, L_j*, v/p_j) come from step 1-3 at the double x*, as in
   1a. Hours-type outputs (H_j, H_s, final hours, N_a and what uses it, λ̃_j, λ̃_j^q,
   L_s, L_s^q) are recomputed with the top segment's e_S − x replaced by the carried
   1 − x*, when x* lies in the top segment: e_{S−1} < x* ≤ 1, including x* = 1.0, where
   step 2 gives that segment no hours at all (G3 at η = 1e-20). M_j is step 2's. The aggregates then follow 1a's `report`
   expressions in 1a's order, with h → B_d and J → M_s. At u = 1, λ̃_m = uλ/(1 − ua) with
   1 − ua from `fma` equals 1a's λ/(1.0 − a) bit for bit.

1b's validation requires J(0) = 0 exactly (1a's default schedule check does too, and the
power schedule's J(0) is 0 by construction), so for one segment step 2 gives
H = 1.0·(1.0 − x) and M = 1.0·(J(x) − 0.0), which are 1a's values. Multiplying by z = 1.0,
adding 0.0, and adding the zero terms of space, are all exact.

The services residual is |K − Y·M_s − aδK|/K when K > 0 and 0 when K = 0. K is 0 when no
basket category has tasks below x*, and then both sides are exactly 0.

### 5.2 Which equation fixes which unknown

- The task margin fixes v, given x (v/p_m = γ(x)).
- The user-cost machine row fixes p_m.
- Zero profit fixes each p_j.
- Land clearing fixes Y, given the basket.
- Service clearing fixes K.
- Labour clearing is the root equation.
- Each category clears by the basket (y_j = z_j·Y). Walras' law then gives the income
  identity and the basket count (worker plus provider baskets equal Y), which are
  checked, not imposed.

### 5.3 The root is unique (SSRN Lemma B.1, p.29, generalised)

Let μ_s(x) = Σ_j z_j·μ_{j,s(x)}, the basket's task density at x.
- **n_D is nonincreasing, and strictly decreasing where μ_s(x) > 0.** dL_s^q/dx =
  μ_s(x)·(−1 + γλδ/s_K), which is negative because δ(a + λγ) ≤ u(a + λγ) < 1, and
  dB_s^q/dx = μ_s(x)·γ·bδ/s_K ≥ 0.
- **v/P_s is strictly increasing when B_d > 0.** P_s/v = H_s + M_s/γ + B_d/v. The first
  two terms have derivative −M_s·γ'/γ² ≤ 0, and B_d/v strictly falls because v rises
  with γ.
- So f strictly decreases wherever μ_s(x) > 0 or 0 < ln(1 + v/P_s) < χ_max. f is constant
  only on a gap where supply is saturated. A root there would be an interval, which needs
  that constant to be exactly zero. The gate does not construct it.
- f is continuous: H_j and M_j are integrals of densities. n_D has kinks at the edges where
  μ_s jumps, and bisection does not mind them.

**Existence.** The regime tests give f(1e-12) > 0 > f(1). The lemma's conditions become
n_S(1) > n_D(1), T > N·P_s(1), and n_D(0) = T·L̄_s/B_d > N. The third follows from the
second when L̄_s ≥ 1, because P_s(1) > B_d; in general it is separate, since the basket's
hours by hand can be below one. `lemma_b1` stays eq 25's two conditions, so it nests with
1a. The third is what `NoInteriorAtZero` tests, at 1e-12 instead of 0.

### 5.4 Precision

Everything in unit-1a.md §4 step 4 carries over: the conditioning at the ends of the
bracket, near the viability edge, and u's error. What is new:
- **Thin segments.** J(e_s) − J(e_{s−1}) loses about log10(J(e_s)/(J(e_s) − J(e_{s−1})))
  digits. The instances keep widths at least 0.2, and the random draws at least 1/12.
- **Interior edges.** When x* lies in an interior segment, e_s − x* carries x*'s spacing of
  doubles, about 1e-16 absolute. A category's H_j loses relative precision only if most of
  its hours are in that sliver. Only the top segment carries 1 − x*.
- **Gaps.** When x* is in a gap, f's slope is n_S's alone. x* is resolved to about
  eps·n_S/|n_S'|, and the quantities do not depend on x* at all (§8, C5).
- **Nesting.** For the one-category case the evaluation is 1a's, so every 1a precision
  statement, including G3's 1 − x* = 4.6e-21, holds unchanged.
- **Measured.** On 2026-09-27 the oracle's f64 values matched all 245 numeric goldens within
  1.0e-15 relative (C6's parity cost; 9.0e-16 on D(1) = −0.1 and 8.8e-16 on food's H in C3,
  where 0.75 − x* carries x*'s spacing). The midpoint cells of `c6::cells_approach_the_line`
  miss the line's closed forms by up to 3.5e-9, the midpoint rule's error at 2^-12.

## 6. Result type

As built (P1.2). The proposal's names mostly stand; §12 lists the departures.

- `src/categories.rs`: `Category { weight, direct_land, density }`, `CategoryParams<S>`
  (1a's `Params` minus `space`, plus `edges` and `categories`, with
  `CategoryParams::from_one_category(Params)` building C1's form of any 1a economy), and
  `CategoryEconomy<S>` with `new`, `at(x) -> CategoryPoint`, `solve() ->
  Result<Regime<Eq1b>, SolveError>`, and the x-free quantities `all_human_hours()` (L̄_j),
  `basket_direct_land()` (B_d) and `basket_all_human_hours()` (L̄_s).
- `Eq1b` holds the aggregates under 1a's `Eq1a` names where they mean the same thing (x_star,
  one_minus_x_star, gamma_star, u, v, p_m, v_m, p_s, y, k, final_hours, machine_hours,
  n_a, participation, income, interest, labor_share, capital_share, real_wage,
  support_cost, worker_baskets, provider_baskets, funded, lemma_b1, phi_w, phi_r,
  bisection_steps). 1a's `j_star` is `m_s`, 1a's `p` is `categories[0].price`, and 1a's
  cost-system rows are `categories[j].lambda_tilde` and `.b_tilde` with the machine row's
  `lambda_tilde_machine` and `b_tilde_machine`, which 1b reports at every u. It adds
  `h_s`, `b_d`, `l_star_s`, `l_s`, `b_s`, `l_s_q`, `b_s_q`, `rent_ceiling` (v/B_s),
  `margin_active`, `categories: Vec<CategoryEq>` and `residuals: Residuals1b`: 1a's five,
  plus `fork`, the largest |p_j − (v·L_j* + b_j)|/p_j, `totals`, the largest
  |p_j − (v·λ̃_j + b̃_j)|/p_j, and `expenditure`, |Σ_j p_j·y_j − I|/I. The services residual
  is 0 when K = 0.
- `CategoryEq` has §4.4's fields: `price`, `real_wage` (v/p_j), `l_star`, `l_bar`, `human`
  (H_j), `machine` (M_j), `lambda_tilde`, `b_tilde`, `lambda_tilde_q`, `b_tilde_q`,
  `output` (y_j), `final_hours`, `machine_services`, `share`, `wage_floor`
  (1/(L̄_j + b_j/v)), `wage_ceiling` (v/b̃_j, `None` when b̃_j = 0), and `phi_w`, `phi_r` at
  u = 1.
- `Eq1b::outputs()` lists every number as `(OutputKey, Output1b)`: economy keys in 1a's
  order, then 1b's, then each category's as `cat<j>.<field>`. `Output1b` is 1a's `Output`
  with `Optional` in place of `FlowOnly`, since v/b̃_j can be absent at any u. The solve
  checks every number for finiteness and names the first that fails:
  `SolveError::NonFinite { what }` for an economy key, and the new
  `SolveError::NonFiniteInCategory { category, what }` for a category's.
- 1a's `Regime` is `Regime<E = Eq1a>`. The default keeps every 1a use compiling, the probe's
  included. 1a's regime tests and bisection are one shared function, `solve::classify`, and
  the labour-residual net `solve::labor_net`; `Root`, `viability`, `wealth_factor` and the
  validation helpers are shared `pub(crate)`. No 1a behaviour changed: 1a's 114 tests pass,
  and the only edits to 1a's test files make helpers visible to 1b's tests.
- `ParamError` gains `Item { kind, index, error }` (a category's or a cell's parameter) and
  `Invalid { name, reason }` (the task line, the basket, J(0)); `Requirement` gains
  `ZeroOrScale` and `OpenUnit`.
- `src/fork.rs`: `Cell { weight, human, machine }`, `cell_cost(cells, direct_land, w, p_m,
  r, machine_totals: Option<(λ̃_m, b̃_m)>) -> Result<CategoryCost, ParamError>` (price, H, M,
  L*, L̄, and λ̃ and b̃ when the totals are given), and `ces_share(alpha, sigma, q) ->
  Result<f64, ParamError>`. A `Cell` needs γ_L in [`SCALE_FLOOR`, `SCALE_CEIL`], γ_M and
  the weight in {0} ∪ that range; w, p_m and r must be in the range, λ̃_m in
  [0, `SCALE_CEIL`] and b̃_m in the range; α in (0, 1), σ in [0, `SCALE_CEIL`], and q
  finite and positive. Within these bounds no output of `cell_cost` can overflow, so there
  is no error for it.

## 7. Goldens

`goldens/generate_1b.py` computes every 1b golden with mpmath at 70 digits from §4's
equations and writes `goldens/goldens_1b.txt`: 249 goldens with 30 significant digits. The
Rust constants in `tests/gate/goldens_1b.rs` carry 20. The keys are `C1_*` (the fork at
G1), `C2_*` (G3's path and the CES share), `C3_*`, `C3D_*` and `C3Z_*` (the fork economy's
three versions), `C4_ETA_*` and `C4_LAM_*` (the paths), `C6_PARITY_*`, and `C7_*` (the gap
economy and the regime rows). Run it with laborformal's venv or any Python with mpmath:
`python goldens/generate_1b.py`, then `--check`, which exits 1 if `goldens_1b.txt` is not
its output. It imports the 1a generator (`generate.py`) only to assert the nesting, so
`goldens_1b.txt`'s header records FNV-1a digests of `generate_1b.py`, of `generate.py` and
of its own body, and the gate recomputes all three.

The generator asserts as it goes:
- the category form of G1, λ = 0, η = 0.5, G4's (0.05, 0.1, 1) and (0.05, 0.1, 3), and
  G5's durable instance equal generate.py's 1a solves to 1e-65 (the prototype found at most
  1.9e-71; the generator finds them equal, since at 70 digits the category form repeats
  generate.py's operations, and records the difference, 0, in the file's header);
- the closed-form L_j* equals mpmath quadrature of main.tex's integrand (1e-60);
- every identity of §4 holds to 1e-65 at every instance: both fork forms, the price-side
  and clearing-side basket forms, eq 11, income by Y·P_s and by Σ p_j·y_j, land, services,
  the user cost and the basket count; and every bound of §4.2 holds;
- C3z equals the flow economy with (δa, δλ, δb) to 1e-65 (the prototype found 0);
- n_D is nonincreasing, v/P_s increasing and f single-crossing on the 1a grid, for C3 and
  the gap economy.

All values below are the generator's, to 20 significant digits.

**C1, the fork at G1** (Appendix B in category form):
- the good: L* 0.66535131631003383848, λ̃ 0.17046683479342606591 (1a's L_s),
  b̃ 0.26893802364690478262, p 0.36157583177980927464 (1a's p);
- v/p 1.5029653890908211134, between the floor 1/L̄ = 1 and the ceiling
  v/b̃ = 2.0206735861577852136;
- the basket's rent ceiling v/B_s 0.42826044343359912019 ≥ v/P_s = 0.3991228016925181936;
- the good's expenditure share p/P_s 0.2655568814754655817; space's v/p is v exactly.

**C2, the fork along G3's path** (λ = 0, γ = η(1 + x)), with the CES share at α = 0.3:

| η | v/p_good | v (1a G3) | q = r/p_good | σ = 0.5 | σ = 2 |
|---|---|---|---|---|---|
| 1 | 1.1819187877999820791 | 0.98840122029213958952 | 1.1957884748975167441 | 0.41720785618326636565 | 0.13314863099244117043 |
| 0.3 | 1.264023705823887722 | 0.32367862903339933844 | 3.90518122743734574 | 0.56402241581444845787 | 0.044920517603712652238 |
| 0.1 | 1.3058486238553142227 | 0.11186280479118877161 | 11.673662450113833805 | 0.69104700183614749627 | 0.015490281344367582874 |
| 0.03 | 1.3244824123062040581 | 0.034056082139179303478 | 38.89121499335571966 | 0.80325049843134940416 | 0.0047005500620400046454 |
| 0.01 | 1.3303189612820227402 | 0.011402655775197234064 | 116.66746655421262523 | 0.8761010290122206556 | 0.001571858600187029765 |

v/p_good rises toward 4/3 while v falls to 0. The limit is γ/(γ(1 − x) + J) at x = 1,
2/(3/2), and the generator checks it. At q = 1 the share is 1/2, 0.39564392373896000165, 0.3 and 9/58 for
σ = 0, 0.5, 1 and 2.

**C3, the fork economy, flow:**
- x* 0.71090347785320059226 (the middle segment: the margin is active, with food's and
  shelter's tasks at parity), 1 − x* 0.28909652214679940774, γ* 0.76872278228256047381;
- v 0.4647912788060681259, p_m 0.60462794848614772328, P_s 1.7925374977968492847;
- Y 5.8178032129320888237, K 4.3467985940910846758;
- N_a 0.9221997776825773684, final hours 0.70485984797802313461, machine hours
  0.21733992970455423379;
- I 10.42863041398375686, labour share 0.041101314071788954885, v/P_s
  0.25929236034243538841;
- provider baskets 1.5786838558695041679, worker baskets 4.2391193570625846558;
- L_s* 0.80151567979890759228, L_s 0.15851340169648020715, B_s 1.7188618511144422805;
- bracket: f(1e-12) 7.6481961400442555623, f(1) −0.92326297508086722453, n_S(1)
  1.1740896553003717311, n_D(1) 0.25082668021950450653, P_s(1) 1.8038892307692307692.
  lemma_b1 and funded are both true.

| category | p_j | v/p_j | L_j* | λ̃_j | b̃_j | share |
|---|---|---|---|---|---|---|
| manufactures | 0.1741328491640105443 | 2.6691763273700016452 | 0.37464741079332217592 | 0.020571428571428571429 | 0.16457142857142857143 | 0.02914296343223475326 |
| food | 0.85248227834842577815 | 0.54522104518881160714 | 0.54321647126639130952 | 0.085252018666643812803 | 0.81285788357155760952 | 0.47557291236372158474 |
| care | 0.21619781970151703148 | 2.1498425814273220769 | 0.25 | 0.25 | 0.1 | 0.024121985728860777756 |
| shelter | 1.0557197509486461712 | 0.44026009590937090225 | 0.11988123161814953748 | 0.021362443073009778652 | 1.0457906737143201244 | 0.47116213847518288425 |

Manufactures is fully automated (H = 0, M = 2·J(0.4) = 0.288). Care is fully human and sits
on its upper price bound, p = v/4 + 0.1.

**C3d, (ρ, δ, J_b) = (0.04, 0.35, 2):** x* 0.76916199916626330199 (the top segment, so
1 − x* 0.23083800083373669801 is carried), v 0.15349426647317235432, p_m
0.18826038769933046651, P_s 1.5366380608625373639, Y 6.6180204392511955284, K
4.3029355447378739266, N_a 0.38083949356762359007, I 10.169502094519594853, interest
0.11104541581041802903, labour share 0.0057482340989613912222, v/P_s
0.099889668479911126888. L_s 0.05960375347228008124, B_s 1.5274892264442619331, L_s^q
0.057545832181007009178 and B_s^q 1.5110258561140773566 differ, as they must when u ≠ δ.

| category | p_j | v/p_j | λ̃_j | b̃_j | λ̃_j^q | b̃_j^q |
|---|---|---|---|---|---|---|
| manufactures | 0.054218991657407174355 | 2.8310055532396202833 | 0.0066497859550050095637 | 0.05319828764004007651 | 0.0056312849162011173184 | 0.045050279329608938547 |
| food | 0.67878697225216980023 | 0.22613024814528870307 | 0.0096629702158666545223 | 0.67730376172693323618 | 0.0081829608938547486034 | 0.66546368715083798883 |
| care | 0.13834591627205451626 | 1.1094961861492818199 | 0.23119534537396985697 | 0.10285875632186527168 | 0.23114061341900152634 | 0.10242090068211862665 |
| shelter | 1.0173952598234181351 | 0.15086984629731146294 | 0.0021334729938974405684 | 1.0170677839511795245 | 0.0018067039106145251397 | 1.0144536312849162011 |

**C3z, (0, 0.1, 1):** x* 0.93807870421979404719, v 0.039387319046324126847, Y
6.9193306663031228905, N_a 0.10750959088903672838, P_s 1.4458384773078020048, v/P_s
0.027241852851823800181; v/p_j = (3.3002186228327612422, 0.063801384732327754173,
0.36049009533485673741, 0.039237077568494895079).

**C4, the paths.** Task automation (η):

| η | x* | v | v/p manufactures | v/p food | v/p care | v/p shelter | v/P_s |
|---|---|---|---|---|---|---|---|
| 1 | 0.71090347785320059226 | 0.4647912788060681259 | 2.6691763273700016452 | 0.54522104518881160714 | 2.1498425814273220769 | 0.44026009590937090225 | 0.25929236034243538841 |
| 0.5 | 0.74133547685661099243 | 0.23319597890872611844 | 2.7537096579350305345 | 0.32251992691158850188 | 1.4731361959096763823 | 0.22702851882045977152 | 0.14557993052074431759 |
| 0.25 | 0.81580469774130705867 | 0.12368952077476531915 | 2.9605686048369640518 | 0.18720695492334621575 | 0.94657127911672200244 | 0.12205350488999499074 | 0.081908097335280514043 |
| 0.1 | 0.91636122404346763349 | 0.053677124048319267321 | 3.2398922890096323152 | 0.086010729866266939192 | 0.4759351154494526326 | 0.053393315781235546928 | 0.036869857767907098279 |
| 0.03 | 0.97320434605992943451 | 0.016810624452634077947 | 3.3977898501664706514 | 0.027685966727487033955 | 0.16185801659777499978 | 0.016783982758397044791 | 0.011749802080120372712 |

This is the fork: the wage in manufactures (b = 0) rises by 27%, the wage in shelter falls
by 96%, and v/p_shelter tracks v, bounded by v/1. Recursive automation (λ):

| λ | x* | v | v/p manufactures | v/p food | v/p care | v/p shelter | v/P_s |
|---|---|---|---|---|---|---|---|
| 0.025 | 0.70310159858561335066 | 0.44790058106251196756 | 2.6475044405155926407 | 0.53002086949055100003 | 2.1129863149816156787 | 0.42492434830200469621 | 0.25142221680166862341 |
| 0 | 0.69555988217240181918 | 0.43225594613595511734 | 2.62655522825667172 | 0.51574056118281099226 | 2.0775144864646863617 | 0.4106831579149892248 | 0.24405601380617598307 |

On this instance lower λ lowers x*, v and every v/p_j.

**C7, the gap economy (N 5):** x* 0.45534506624697703036, in the unused segment [0.4, 0.6);
γ* 0.56427605299758162429 = w/p_m, set by labour clearing; v 0.33598549968628947264; P_s
1.5600097496109288149; Y 6.6202617840659756374; K 1.4980706665657864871; N_a
0.97525913596126201105; I 10.32767292811956354. L_s^q = 1.0312/7 (0.14731428571428571429)
and B_s^q = 10.5736/7 (1.5105142857142857143) are exact rationals, since no task changes
hands in the gap. p_j = (0.17148313027926081201, 0.66974962254471836081,
0.14031825996235473672, 1.0134394199874515789), v/p_j = (1.9592918506860473066,
0.50165836362805285931, 2.3944531508331794588, 0.33152992972234063069). At N = 4.5 the
root moves to 0.5417 and N_a and Y are unchanged.

**C6, check_interior's parity instance** (:50-58): (a, λ, b) = (0.2, 0.1, 0.4), γ̄ = 0.35,
v = 0.14/0.765 = 0.18300653594771241830, p_m = 0.52287581699346405229, 321 cells with
γ_L = 0.2 + i·0.008125 (i = 0…320, weight 1/321), L̄ = 1.0202569012788672440. The cost is
land + v·L̄ = 0.18671368127979269825 + land, for land ∈ {0, 0.1, 0.5, 2}.

**C7, regimes on the fork economy:** N 0.2 → `BoundaryNoMargin`, f(1)
0.19212219745448591998; λ 0.6 → `BoundaryNoMargin`, f(1) 0.19428727505460366806; λ 0.8
→ `NotViable`, D(1) −0.1; N 20 with χ_max 0.05 → `NoInteriorAtZero`, f(1e-12)
−12.063380281698479298; N 6 → `Interior` at x* 0.67223405154979632125 with funded and
lemma_b1 both false.

## 8. Tests

The groups are the modules `c1_nesting`, `c2_ces_share`, `c3_fork_economy`, `c4_paths`,
`c5_random_categories`, `c6_price_block`, `c7_gaps_and_regimes` and `c8_goldens_file` of
`tests/gate/`, cited below as `c1::` to `c8::`. The PLAN's gate maps onto them:
- the fork identity and the category bounds on random instances:
  `c5::fork_identity_and_category_bounds`, `c6::cost_and_accounting` and
  `c6::exact_interior_prices`;
- the income identity to 1e-12: `c5::identities`, `c6::cost_and_accounting`, and every
  golden group;
- 1a's results as the one-category case exactly: C1.

Every test sits in `tests/gate/` unless it is a unit test of a private function. Identities
are compared at 1e-12 relative (ADDENDUM A7), goldens at 1e-12 relative. An inequality
a ≤ b is checked as a ≤ b·(1 + 1e-12). Where the maths is strict and the instance has room
(a margin above 1e-6), it is also checked strictly, without slack. The shared identity check
(`support_1b::check_identities_1b`) also pins every p_j, M_j, P_s and L_j* to `at(x*)` bit
for bit (§5.1 step 5).

**C1, nesting with 1a (R1).** The gate's "exactly" is bit equality.
- `c1::appendix_b_is_bit_identical`: G1 in category form, every shared output (§6's name
  map, the cost system at u = 1 included), the regime and `bisection_steps` equal 1a's bit
  for bit. [R1; §5.1]
- `c1::every_1a_golden_instance_is_bit_identical`: 27 instances: G1, G2's two cases, G3's
  five points and its two near-full-automation points (1 − x* = 4.6e-21 included, where
  x* rounds to 1.0 and the good's hours are the carried 1 − x*), G4 cases A-G and the two
  extra lags of G4's ρ·W_K test, G5's two general instances and its saturated-supply draw,
  and G8's `Interior` rows (N 8, N 7.35, the funded tie, the lemma tie, the curvature
  ceiling). [R1]
- `c1::random_1a_economies_are_bit_identical`: G5's three sets of 60 draws from the same
  SplitMix64 generator, and every skipped draw's regime and diagnostic. [R1]
- `c1::regime_rows_nest`: every G8 row gives the same regime and diagnostic bit for bit,
  exact ties included; the jump schedule gives the same `LaborNotCleared`; the ρ = 1e30
  rows give the same `NonFinite`. [1a §4 steps 2-5]
- `c1::rejected_rows_nest`: the same rows are rejected (N 1e300, k 1e20 and the next double
  above `CURVATURE_CEIL`, λ 1e300, the overflowing u) with the same error; h = 1e300 and
  h = 0 are rejected as category 1's `weight` and as the basket, since h is space's
  weight. [1a §2]
- `c1::points_nest_on_a_grid`: `at(x)`'s aggregates equal 1a's `Point` bit for bit on
  {1e-12, 0.01, …, 1}. [§5.1 order]
- `c1::fork_at_g1`: the C1 goldens; space's v/p equals v bit for bit. [SSRN eq 12, 13;
  main.tex:459, 469]
- `c1::fork_along_g3`: the C2 table; v/p_good ≥ 1/L̄ = 1 and r/p_good ≥ 1/(v·L̄) at each
  point; v/p_good rises and v falls. [SSRN p.14 corollary; App. C p.31]

**C2, the CES share (price block).**
- `c2::values_on_the_g3_path` and `c2::values_at_q_one` (the goldens). [SSRN eq 26]
- `c2::unit_elasticity_gives_alpha`: σ = 1 gives α at q ∈ {1e-6, 1, 1e6}, within 1e-15.
- `c2::limits_and_monotonicity`: at q = 1e12 the share is above 1 − 1e-5 for σ = 0.5 and
  below 1e-5 for σ = 2, monotone in q on a grid; q = 1e300 gives a finite share.
- `c2::rejects_bad_arguments`: α ∉ (0, 1), σ < 0 or above `SCALE_CEIL`, q ≤ 0 or
  infinite, and NaN.

**C3, the fork economy.**
- `c3::flow_goldens`, `c3::durable_goldens`, `c3::bracket_values` (with both flags true).
  [§4]
- `c3::rho_zero_is_the_scaled_flow_economy`: C3z against the flow economy with
  (0.03, 0.005, 0.04), every output at 1e-12. [unit-1a.md G4 nesting, per category]
- `c3::care_sits_on_its_human_bound`: L*_care = L̄_care and p_care = v·L̄_care + b_care bit
  for bit; its v/p equals the lower end of the pair. Without its direct land, care uses no
  land at all: b̃ = 0, the pair has no ceiling, and v/p = 1/L̄ = 4. [main.tex:465, 469, 474]
- `c3::manufactures_is_fully_automated`: H = 0 exactly, L* = M/γ(x*). [§4.1]
- `c3::durable_price_and_clearing_totals_differ`: λ̃^q < λ̃ and b̃^q < b̃ for every category
  with M_j > 0, and the bounds chain holds. [§4.2-4.3]
- `c3::single_crossing_on_a_grid`. [§5.3]
- `c3::cost_system_rows_hold`: p = Ap + λv + b by multiplication, at C3 and C3d; at C3z,
  f = (I − Aᵀ)y and pᵀf = v·N_a + T. [SSRN eq 3; A.1; App. C]

**C4, the paths.**
- `c4::task_automation`: the η table; v and v/p_shelter strictly fall; v/p_manufactures
  strictly rises and stays ≥ 1/L̄ = 1.25; the pair holds at every point. [the fork;
  main.tex:491-504]
- `c4::recursive_automation`: the λ table; x* and v fall. [SSRN §3.4]

**C5, random multi-category economies.** A SplitMix64 generator seeded from 923, 924 and
925, as in 1a's G5 (the draws are 1b's own). 1a's G5 table for N, T, a, b, λ, g0, g1, k,
χ_max, ρ and δ, and:

| draw | range |
|---|---|
| S | an integer in 1..4 |
| widths | u_s ~ U(0.5, 1.5); e_s = Σ_{i≤s} u_i / Σ u, with e_S = 1.0 set exactly |
| C | an integer in 2..5 |
| μ_js | 0 with probability 0.4, else U(0.2, 3) |
| b_j | category 0: 0; others 0 with probability 0.2, else U(0.05, 1.5) |
| z_j | categories 2 and up: 0 with probability 0.1; else U(0.1, 1.5) |
| site | with probability 0.5, add a category with μ = 0, b = 1, z ~ U(0.2, 1.5) |

A category drawn with μ = 0 and b = 0 gets b = 1, and a draw failing §3.2 is skipped and
counted. There are three sets of 60 interior economies: the flow benchmark, J_b = 1, and
J_b in 1..5. A 30-digit scratch run of 300 such draws gave 212 `Interior`, 76
`BoundaryNoMargin`, 12 invalid and 2 roots in a gap, and every identity held. The gate's
three sets took 267 draws for their 180 interior economies: 13 failed §3.2 (a basket with
no direct land), 74 were `BoundaryNoMargin`, none `NotViable` or `NoInteriorAtZero`; one
interior root lay in a gap, and 32 categories were not bought. For each economy:
- `c5::identities`: income as Y·P_s and as Σ p_j·y_j; land, services and the user cost;
  eq 11 (Y = T/B_s^q, n_D = T·L_s^q/B_s^q); P_s = v·L_s + B_s = v·L_s* + B_d; worker
  plus provider baskets equal Y; interest = ρ·W_K. [SSRN App. C, eq 11, §5]
- `c5::fork_identity_and_category_bounds`: for every category, p_j = v·L_j* + b_j and
  p_j = v·λ̃_j + b̃_j [main.tex:459; SSRN eq 12]; the chain b_j ≤ b̃_j^q ≤ b̃_j ≤ p_j ≤
  v·L̄_j + b_j, L_j* ≤ L̄_j, the pair, and v/p_j ≥ 1/L̄_j when b_j = 0; the basket's
  v/P_s ≤ v/B_s [SSRN eq 13, 19; main.tex:465-474]. Some drawn categories are wholly
  automated.
- `c5::residuals_recompute`: each residual equals its recomputation bit for bit, and each is
  nonzero in some economy. [1a §4 step 5]
- `c5::root_and_grid`: x* is the double where f changes sign; n_D is nonincreasing, v/P_s
  increasing and f single-crossing on the grid. [§5.3]
- `c5::margin_flag`: `margin_active` equals μ_s(x*) > 0, recomputed.
- `c5::unused_categories_are_priced`: a category with z_j = 0 has its cost price and
  satisfies the bounds, which are cost identities whether or not it is produced. [§4.2]
- `c5::the_draws_cover_the_regimes`: the sets meet invalid and `BoundaryNoMargin` draws, and
  every build lag in 1..5.

**C6, the price-block batteries.** Productivities are log-uniform, not lognormal, since
core has no normal generator. The ranges cover lognormal's ±2σ. The seeds are
check_interior's, 20260905 and 20260921, in SplitMix64.
- `c6::cost_and_accounting` (check_interior:10-49): 80 economies. a ~ U(0.05, 0.65), λ ~
  U(0, 0.3), b ~ U(0.1, 1), v = 10^U(−5, 1), r ~ U(0.1, 3), w = r·v and p_m =
  r(λv + b)/(1 − a). There are 4 categories of 193 cells each, γ_L log-uniform on
  [e^−1.4, e^1.4], γ_M log-uniform on [e^−2, e^2] and closed with probability 0.09, b_j = 0
  for category 0 and U(0.1, 2) otherwise, outputs y_j ~ U(0.1, 5), and a household rental
  T_h ~ U(0.2, 3). It checks the fork identity, SSRN eq 12 with (λ̃_m, b̃_m) =
  (λ, b)/(1 − a), the bounds and pair, and §4.5's accounting identity and land share, all
  at 1e-12 (check_interior asserts 5e-13; the test asserts the maxima below 5e-13 too).
- `c6::exact_interior_prices` (check_interior:145-161): 100 instances. g* ~ U(0.2, 1.5),
  257 cells, γ_L log-uniform on [e^−1, e^1], γ_c log-uniform on [e^−1.6, e^1.6] and
  closed with probability 0.1, w ~ U(0.3, 3), p_m = w/g*, r ~ U(0.2, 2), b_j ~ U(0, 1).
  It checks the fork identity and the pair at 1e-12, and L* ≤ L̄.
- `c6::parity_with_uneven_human_productivity` (check_interior:50-58): the C6 goldens, and
  cost = land + v·L̄ at 1e-12. [SSRN A.3 parity]
- `c6::flat_zero_land_category_is_invariant` (check_fan F4; check_dynamics F4): cells all
  at γ̄ with b = 0 and g = γ̄ give w/p = 1/L̄, with (a, b) = (0.3, 0.4), at u = 1, 1.05
  (1 + ρ), 0.15 (ρ + δ) and 0.165375 (u(0.05, 0.1, 3)), γ̄ ∈ {0.25, 1, 3} and
  λ ∈ {0, 0.05}; p_m and w come from 1a's `closure`. The inputs are dyadic where exactness
  is asserted. [main.tex:506-509]
- `c6::closed_cells_go_to_people`: a closed cell is human at any prices, and ties go to
  people. [SSRN §3.1; check_interior:27]
- `c6::cells_approach_the_line`: C3's categories cut into 2^12 midpoint cells per unit of
  line give L* and p_j within 1e-6 of the line's closed forms (the midpoint error is
  O(Δ²)). A sanity check, not a gate identity.
- `c6::cell_validation`: γ_L ≤ 0, a negative weight, NaN, a machine productivity below
  the floor, negative direct land, w, p_m or r out of range, and bad machine totals are
  errors; so is a category with neither tasks nor land. A site (no cells) costs r·b, and
  −0.0 is read as +0.0.

**C7, gaps, reductions and regimes.**
- `c7::gap_goldens`: the gap economy's goldens, with `margin_active` false, and the fork
  identity and bounds holding there. [SSRN p.13, "at boundary task assignments"]
- `c7::only_bought_tasks_make_a_margin`: an unbought category with tasks in the gap leaves
  it a gap, and x* bit for bit; bought, it moves the root. [§2.3]
- `c7::a_root_below_every_bought_task_uses_no_machines`: with tasks only on [0.5, 1] and N
  30, x* < 0.5: K = M_s = 0, the services residual is 0, and every identity holds. [§5.1]
- `c7::gap_quantities_do_not_move`: N 4.5 and N 5 give different x* and v, but bit-equal Y,
  K, N_a, H_s and M_s. [§5.4]
- `c7::unused_category_changes_nothing`: adding a category with z = 0, last or first, in
  the flow and the durable version, leaves every aggregate output bit for bit unchanged,
  except `res_fork` and `res_totals`, which are maxima over all categories, bought or not.
  [R1]
- `c7::split_category`: category j replaced by two copies with weights θ·z_j and
  (1 − θ)·z_j gives the same aggregates within 1e-12, and the copies' prices are bit-equal.
  [R1]
- `c7::split_segment`: cutting a segment in two with the same densities gives the same
  outputs within 1e-12. [R1]
- `c7::permutation`: reversing the category order gives the same aggregates and
  per-category outputs within 1e-12 (the sums round in another order, so x* can move by an
  ulp), and at any fixed x the same p_j, H_j and M_j, permuted, bit for bit.
- `c7::unit_rescaling`: (μ_j, b_j, z_j) → (c·μ_j, c·b_j, z_j/c): at c = 4 every scaling is
  exact and the solve is the same bit for bit, with p_j = 4·p_j exactly; at c = 3 the
  aggregates agree within 1e-12, p_j scales by c and v/p_j by 1/c.
- `c7::regimes`: the C7 rows, and viability decided at the top of the line when no basket
  category uses the top segment (§3.2; D(1) = 0 exactly at λ = 0.7 gives `NotViable`).
- `c7::validation`: edges not starting at 0.0 or ending at 1.0, not increasing, or
  containing NaN; a density of the wrong length; no categories; B_d = 0; L̄_s = 0; a
  category with neither tasks nor land; out-of-range weights, densities and land. Each is
  an error, and −0.0 is stored as +0.0.

**C8, the goldens file.** `goldens_1b.txt`'s three digests recompute, and the Rust
constants match it to 20 digits (1a's `goldens_file` tests, extended).

**Counts and teeth.** On 2026-09-27 the oracle has 169 tests: 46 unit tests (42 of 1a, 4
of 1b), 122 gate tests (71 of 1a, 51 of 1b) and 1 doc test. A mutation check of the same
day applied 45 mutants to `categories.rs`, `fork.rs`, the goldens files and the generator,
one at a time, and ran the oracle's tests: 44 were caught, and each of 1b's 55 tests failed
under at least one. One of the 44, L_j* from the carried hours, was caught only after the
bit-for-bit check of p_j and L_j* at `at(x*)` was added. The survivor reassociates
p_j = v·H_j + (p_m·M_j + b_j), which changes nothing for one category (the good has b = 0)
and only the rounding for many.

## 9. Out of scope, and where each goes

- Category-to-category intermediate inputs (a full A over categories), several machine
  types, the Leontief inverse, per-type user costs and separate build and operating recipes:
  **1c**. PLAN §3.1's machine recipe (A, Λ, B) lives there.
- Worker types, human-required tasks (closed cells) in the equilibrium, solving the
  boundary regimes (x* = 1 and the root below 1e-12), and the wall: **1d**. Closed cells
  are in 1b's price block only.
- Several non-produced inputs (SSRN A.1's vector r: land, sites, deposits), land quality,
  idle land and exit as s(q): **1e**.
- Price-responsive baskets (CES over categories, demand around a subsistence basket, a
  basket per worker type), government and transfers: **1f**.
- Cells in the equilibrium, with the marginal cell split between people and machines:
  open question 2.
- Per-category relative-capability schedules in the equilibrium (§2.3).
- A dump interface for categories: there is no other multi-category solver to compare
  against.
- The paper's measured fork (SSRN §9; check_fan D1's fan anchors): a Phase 7 scoring
  target, not an oracle gate.

## 10. Pitfalls

- **Two fork forms.** SSRN's floor is rb̃_i (totals); main.tex's is rb_j (direct). A good
  with b_j = 0 still has b̃_j > 0 through machines, so SSRN's ceiling (w/r)/b̃_j applies to
  it; main.tex's (w/r)/b_j does not.
- **L_j* is not hours.** It is wage-equivalent task cost (main.tex:455). The hours
  embodied are λ̃_j.
- **Price side and clearing side.** With u ≠ δ, λ̃_j and b̃_j (scaled by u) are not λ̃_j^q
  and b̃_j^q (scaled by δ). SSRN eq 12 and 19 use the first pair; eq 11 uses the second.
- **b means two things.** It is the machine's land per service; b_j is category j's direct
  land. μ_js is a task density; ω is still 1a's wealth factor.
- **"C" groups.** 1b's test groups C1-C8 are unrelated to check_macro's C-series and to SSRN
  Appendix C.
- **Two things called the fork.** The fork identity is a price identity. macro.py's B1
  fork is inflation of goods against shelter along a path (macro.py:208-209). C4 shows the
  second as a consequence of the first.
- **A gap is not a wall.** In a gap, w/p_m lies strictly between two categories' task
  capabilities, and no task is at parity. The wall (1d) is labour holding only
  human-required tasks.
- **Space is a category.** There is no h. Appendix B's h is space's weight.

## 11. Open questions

As built, 1b takes the first draft's choice on each; none is ruled here.

1. **The task line.** 1b replaces main.tex's per-category schedules γ_j with one line and
   category densities (§2.3). It keeps bit-for-bit nesting and 1a's precision, and the cell
   price block covers per-category schedules in full. Accept?
2. **Cells in the equilibrium.** PLAN §3.1's recipes are cells with their own
   productivities. Phase 2's multi-category stationary instance will need the oracle on the
   agents' own cells, with the marginal cell split between people and machines. That
   solver is different: the margin is either one cell's γ, with its split share found in
   closed form (n_D is linear-fractional in the share at fixed prices), or a gap between
   two cells' γ. Should it go in 1c, or in a 1b addendum before Phase 2?
3. **Gaps.** 1b solves a root in a gap and flags it with `margin_active = false` inside
   `Interior`. Should a gap be its own regime instead?
4. **Bit-for-bit nesting as the gate.** It requires the evaluation order of §5.1, which ties
   1b's code to 1a's operation order. The fallback is 1e-12 relative. Keep bitwise?
5. **Viability at the top of the line** is required even when no basket category uses the
   top segment, to keep 1a's convention. The alternative is viability at the top of the
   used range.
6. **The API change** `Regime<E = Eq1a>` touches 1a's public type, though not its uses. Is
   that acceptable, or should 1b have its own regime type?
7. **The CES table's parameters** (α = 0.3, σ = 0.5 and 2) are constructed; the paper gives
   none. α = 0.3 matches the Cobb-Douglas land share of check_pinning A-joint and dynamics'
   targets.

## 12. Changes during the build (P1.2, 2026-09-27)

1. **Labels.** The gap economy's goldens are C7's, and the random economies are test group
   C5; the draft called the gap economy C5 in §3.3 and §7 and the random economies C7 in
   §3.3. Golden keys are listed in §7.
2. **No `ForkError`.** `cell_cost` and `ces_share` return `ParamError`: within the input
   bounds no output of `cell_cost` can overflow, so its `NonFinite` case could never be
   reached or tested.
3. **`ces_share` takes any finite positive q.** The draft bounded q by the scale range and
   also asked for a finite share at q = 1e300. Along an automation path q grows without
   bound, so the bound went.
4. **Result types** (§6). `Output1b` replaces 1a's `Output` for 1b, with `Optional` for a
   number that can be absent at any u (v/b̃_j when b̃_j = 0), keyed by `OutputKey`; 1a's
   `Output` is unchanged, since 1a's tests match on it exhaustively. A non-finite category
   output is `SolveError::NonFiniteInCategory`. `ParamError` gains `Item` and `Invalid`,
   `Requirement` gains `ZeroOrScale` and `OpenUnit`, and `CategoryParams::from_one_category`
   builds C1's form of a 1a economy.
5. **J(0) = 0 is checked** by 1b's validation. 1a's default schedule check requires it, but
   a schedule may override the check, and the nesting argument of §5.1 needs it.
6. **Step 5 made precise.** The top segment takes the carried 1 − x* for
   e_{S−1} < x* ≤ 1, x* = 1.0 included, where step 2 gives it no hours; M_j, p_j and L_j*
   are step 2's at the double x*, and the tests pin them there bit for bit (§8).
7. **Reductions.** `c7::permutation` compares per-category values bit for bit at a fixed x,
   not at x*, which can move by an ulp when the sums round in another order.
   `c7::unused_category_changes_nothing` excepts the two residuals that are maxima over all
   categories. `c7::unit_rescaling` adds an exact case, c = 4.
8. **Tests added**: `c1::rejected_rows_nest` (split from `regime_rows_nest`),
   `c5::the_draws_cover_the_regimes`, `c7::only_bought_tasks_make_a_margin` and
   `c7::a_root_below_every_bought_task_uses_no_machines`, the only tests of
   `margin_active`'s weight condition and of the services residual at K = 0 (the mutation
   check confirms both); the bit-for-bit check of p_j and L_j* at `at(x*)`; care without land in `c3::care_sits_on_its_human_bound`; viability at the top of the
   line in `c7::regimes`. `c5::fork_identity` and `c5::category_bounds` are one test.
9. **Numbers.** The generator reproduces the prototype's 205 numbers and writes 249
   goldens; it finds the category form equal to generate.py's 1a solves, not within
   1.9e-71. The C5 tallies are the gate's own (§8).
10. **1a.** Its behaviour and its 114 tests are unchanged. Its solve now calls the shared
    `classify` and `labor_net`; its test files only make `SplitMix64`, G5's draw, G8's
    helpers and the goldens-file helpers visible to 1b's tests.

