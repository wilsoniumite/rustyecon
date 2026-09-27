# Unit 1d: worker types and the wall

Dated 2026-09-27. This is the unit's specification, written before any code. A 70-digit
scratch prototype built on `generate_1c.py` (in the run's scratch directory, not kept)
checked every derivation below and supplied the numbers in §7; `goldens/generate_1d.py`
must reproduce them. The build records where it departs from this draft in §12.

- **Pinned to:** laborformal `31b3482`. Every laborformal path and line below is at that
  commit.
- **Paper:** SSRN 7226858 (posted 2026-09-23), cited as "SSRN p.N (eq k)".
- **Older revision:** `pinning/paper/main.tex` (2026-09-21), cited as "main.tex:N". It
  states the human-required set, the wall and the heterogeneity paragraphs in the same
  words as the SSRN version, and Appendix E's two lemmas formally, where the SSRN version
  keeps them as prose. Its Appendix B is not used (unit-1a.md §0).
- **Reference checks:** `pinning/checks/check_pinning.py` D1 (:147-167) with its CES
  limits (:168-178), D×C (:180-194), D.2 (:202-213) and D.3 (:215-223);
  `pinning/checks/corner/check_kset.py` P9-i to P9-v (:39-108) and its κ block (:110-120).
- **Gate (PLAN Phase 1):** constructed wall and interior cases recognised correctly; the
  wall regime solved; 1a to 1c as the case with no human-required tasks and one worker
  type, exactly. The gate is constructed (ADDENDUM §5 item 4).
- **Built on:** units 1a, 1b and 1c (docs/unit-1a.md, unit-1b.md, unit-1c.md). Every
  equation, convention, constant and precision rule of those units carries over unless
  this document says otherwise.
- **Goldens:** a new generator, `goldens/generate_1d.py`, writing `goldens/goldens_1d.txt`
  (§7).

## 0. What the sources give

| source | what 1d takes from it |
|---|---|
| SSRN §3.1, p.6 (main.tex:147-162) | labour holds task x when w ≤ p_m·γ(x); tasks closed to machines have γ_M = 0; "Let H denote the set of tasks closed to machines. The relabeling places these tasks last … a wall at the right edge"; where labour holds a contestable task the indifference condition applies; "If labor holds only tasks in H, there is no machine-contestable marginal task and the machine comparison does not pin the wage"; machines take the tasks below x*, labour those above, "while labor performs the required tasks in H" |
| SSRN p.7, Figure 1's caption | "If labor holds no machine-contestable task, the wall marks the boundary assignment in which wages are determined within the human-required sector" |
| SSRN Prop 1 and its proof, pp.7 and 26 (main.tex:164-175) | w = p_m·γ(x*) at an interior contestable margin; "final-task labor demand is Y∫_{x*}^1 dx/γ_L(x), including the human-required tail" |
| SSRN §5, pp.11-13 (eq 10-11; main.tex:431) | N_a = N·F(log(1 + w/P_s)); T·L_s/B_s = N·F(…); "Heterogeneity and skill premia. Workers differ in task productivity, access to support, living costs, and preferences. An active contestable margin prices each type's wage against machine services. For a type employed only at human-required tasks in H, demand, supply, institutions, and participation determine the wage" |
| SSRN §8, pp.17-18 (main.tex:582-586) | the eras: pre-industrial, "the margin sits at the wall for entrant and master alike, so the machine prices no one's hour"; industrial, the entrant's margin on the steep stretch, while "the trained worker … held tasks closed to engines; that margin stayed at the wall, so no machine set the wage, and it was instead a scarcity price on trained hands and heads" |
| SSRN A.1, pp.24-25 (main.tex:686-722) | "Tasks in H are closed to machines by capability, preference, or law"; "Types may differ in task productivity, access to support, living requirements, and preferences"; "With type-specific schedules, replacement applies at each type's active contestable margin. … Reservation wages depend on each type's support, living costs, and preferences"; labour clearing N_a = λᵀy |
| SSRN Appendix E, pp.34-36 (main.tex:908-940) | H of positive measure with H-wage w_H; γ → 0 outside H; Prop E.1 (Baumol concentration): (i) a good with no additional direct land whose task list contains H has unit cost → \|H\|·w_H/γ_L, labour's share of its cost → 1; (ii) under CES the H-content share → 1 for σ_H < 1; (iii) the three-way split; the premises: w_H and r converge to positive finite values, p_m stays bounded, γ_L constant on H |
| main.tex:922-938 (SSRN p.36 as prose) | the fraud bound v·f/(1 − v); the superstar lemma, median = (1 − ψ)·mean |
| check_pinning.py D1 (:147-167) and :168-178 | c = br/(1 − a − λγ) → br/(1 − a) as γ → 0; the unit cost \|H\|w_H/γ_L + (1 − \|H\|)cγ/γ_L → \|H\|w_H/γ_L; labour's share of cost → 1; an H-free good's price → 0; the relative price of H-content diverges; the CES share (α = 1/4) → 1, the taste weight, 0 for σ < 1, = 1, > 1 |
| check_kset.py P9-i (:39-47) | "a good whose checklist meets K (measure k, wage w_K) has unit cost C(m) = k·w_K/γ_L + (1 − k)·m"; C → k·w_K/γ_L and labour's cost share → 1 as the machine cost m → 0 |
| check_pinning.py D.2, D.3; check_kset.py P9-iv, P9-v | the fraud bound's algebra; the superstar and barbell instance (N 100, K holds 5 with 60% of income) |
| check_pinning.py D×C (:180-194); check_kset.py κ (:110-120) | an H-service component in the subsistence bundle lowers coverage at every q |

### 0.1 There is no equilibrium with a human-required set at the pin

laborformal has no equilibrium instance with H of positive measure and none with worker
types (ADDENDUM §5 item 4). The sources give the wall's definition, the limit statements of
Appendix E and check_pinning D1 (price-side limits, not equilibria), and the eras figure
(`pinning/code/fig_eras_workers.py`, schematic: "the curvatures and wage ratios are
schematic illustrations"). Every 1d golden is therefore constructed here, and the gate's
pins are the constructions of §3.3, each built so that a wrong unit fails it.

### 0.2 What "the wall" means here

Three things in the sources are easy to conflate, and 1d keeps them apart:
- **The wall** (SSRN §3.1): labour holds no machine-contestable task. Every contestable task
  is automated (x* = 1), labour works only at human-required tasks and in machine building,
  and the wage is above labour's replacement value at every contestable task. 1a classified
  it as `BoundaryNoMargin`; 1d solves it (§4.5).
- **A type at its wall** (SSRN §5 and §8): one worker type holds only tasks that no machine
  and no other type can do, and its wage is a scarcity price set by the demand for those
  tasks, while other types may hold an interior margin (§4.3).
- **The all-human corner**: the mirror of the wall, labour holds every contestable task
  (x* = 0) at a wage below its replacement value at the bottom task. 1a classified it as
  `NoInteriorAtZero` (with a root in (0, 10^-12) mixed in); 1d solves both (§4.5, §5.3).

A gap (1b §2.3) is none of these: there x* is interior, no produced task sits at it, and
labour clearing sets w/p_m.

## 1. Scope

**In scope:**
- human-required tasks of two kinds: tasks closed to machines that any worker can do (the
  paper's H, "a wall at the right edge"), given per category as hours per unit of output;
  and tasks closed to machines that only one worker type can do (reserved tasks), given per
  category and type;
- I worker types, each with its own number of workers N_i, work-cost distribution F_i
  (uniform on [0, χ_max,i], as in 1a), task efficiency ε_i on the contestable line, and
  support ν_i (the baskets the provider gives each of its workers: its living cost), and
  so its own participation and its own exit value in the dependence form
  s_i = (e^χ − 1)·ν_i·P_s (SSRN eq 9 per type);
- the assignment of types to tasks: types that sell hours on the contestable line are
  perfect substitutes there in efficiency units (the pool), each type does its reserved
  tasks, and a type whose reserved work takes all its hours at the pool's wage is priced
  above the pool, at its own wall;
- the three stretches of one equilibrium path: the all-human corner (x* = 0), the task line
  (1a-1c's threshold, and roots in (0, 10^-12)), and the wall (x* = 1), with 1c's machine
  types, techniques, switches and ties throughout;
- the limits of check_pinning D1 and SSRN Prop E.1(i)-(ii) along an automation path to the
  wall;
- refusing an economy that has no equilibrium with land fully rented (`LaborShort`).

**Out of scope** (§9 says where each goes): distinct schedule shapes per worker type
(open question 1); machine recipes that use a type's reserved labour (open question 6);
training as a choice (types' numbers are inputs); a basket per worker type and
price-responsive baskets (1f); the Baumol three-way split, Prop E.1(iii), which needs a
CES closure (1f); the fraud bound and the superstar lemma (not equilibrium objects of this
closure); idle land at zero rent, which would resolve `LaborShort` (1e); the default exit
form s(q) (1e).

## 2. Decisions

### 2.1 One path, three stretches, one unknown on each

The pool's wage v (1a-1c's v = w/r, in efficiency units, §2.2) runs from 0 to ∞ along one
path:
- **the all-human corner**: x = 0, v ∈ (0, v(0)], where v(0) = γ(0)·π is the replacement
  value at the bottom task (π the delivered machine-task price, 1c §4.4);
- **the task line**: x ∈ [0, 1], with v = γ(x)·π_τ(x) fixed by the task margin (1a-1c);
- **the wall**: x = 1, v ∈ [v(1), ∞), v(1) = γ(1)·π.

The corners meet the line where it ends, so the path is continuous. At fixed x the
quantities do not depend on v (the assignment fixes the recipes), so on a corner stretch
labour clearing is an equation in one price. 1d keeps 1a's unknown and bisection on the
line, so the one-type case nests bit for bit and every 1a-1c precision statement holds
there, and it adds a bisection on each corner stretch (§5.3). With a switch of technique
inside the wall stretch (1c's envelope continued above γ(1)), the wall has techniques and
ties like the line. 1c's count of sign changes runs over the whole path, so a corner is an
equilibrium like any other and more than one is refused (1c §2.5, decision 70).

### 2.2 Worker types: one shape, an efficiency per type, and reserved tasks

The sources give each type "type-specific schedules" and a margin of its own. 1d takes the
structure that the paper's eras describe (main.tex:582-586) and that keeps one threshold,
as decision 68 did for machine types:
- **The contestable line is common.** Type i does task x with ε_i efficiency hours per hour,
  so its relative productivity against the machine is γ_i(x) = ε_i·γ(x): one shape, a
  scale per type. At any task the cost of an efficiency hour is the same from every type
  selling there, so those types are perfect substitutes on the line and in the common
  human-required tasks, and they form **the pool**: one wage v per efficiency hour, type i
  paid v_i = ε_i·v. Each pooled type's margin is the pool's: v_i = π·γ_i(x*), p_m·γ_i(x*)
  with 1a's one machine (SSRN A.1, "replacement applies at each type's active contestable
  margin"). Which pooled type does
  which pool task is indeterminate, as in any tie (§4.6).
- **Reserved tasks** are closed to machines and to every other type: a category needs
  R_ji hours of type i per unit. They are the eras' "tasks closed to engines" held by the
  trained worker. A type does its reserved tasks first; if they take all the hours it
  offers at the pooled wage ε_i·v, its wage rises above ε_i·v until its supply equals its
  reserved demand, and it leaves the pool: it is **at its wall**, "a scarcity price on
  trained hands and heads" (main.tex:584), set by demand, supply and participation (SSRN
  §5). The pool's own margin may be interior meanwhile (the industrial era).
- A type with ε_i = 0 does only reserved tasks.

Distinct shapes per type (comparative advantage along the line) would need a threshold per
type and an I-dimensional solve; that is open question 1, and it binds Phase 2.

### 2.3 Living costs and exit per type, on one basket

Every household still buys the fixed basket z (decision 59), so eq 11 holds and Y = T/B_s.
Types differ in living cost by ν_i, the baskets of support the provider gives each type-i
worker in work and out of it. SSRN eq 8 with support ν_i gives utility
ln(ν_i + v_i·n/P_s) − χ·n, so a type-i worker works when χ ≤ ln(1 + v_i/(ν_i·P_s)): exit
s_i = (e^χ − 1)·ν_i·P_s, and hours n_S,i = N_i·F_i(ln(1 + v_i/(ν_i·P_s))). At ν_i = 1 this
is 1a. A basket per type is 1f's (1b §9), since with it the basket's composition would move
with the income distribution and eq 11 would not hold.

### 2.4 No equilibrium with land fully rented is refused

On the wall stretch labour demand is fixed and supply rises with v toward a ceiling: the
pool's real wage v/P_s is bounded by the embodied labour per basket, and each type's
supply by its workers. If demand at the wall exceeds what supply can reach (1a's G8 row
N = 0.25 is one), no wage clears the market with land fully rented at r = 1. The solve
returns `SolveError::LaborShort` (§6), by the precedent of decision 70. Unit 1e's idle
margin (land unused at zero rent) is the closure that has an equilibrium there.

### 2.5 The solved corners are reported inside `Interior`

A wall or all-human equilibrium is `Regime::Interior(Eq1d)` with `Eq1d::margin` saying
which stretch holds it, as decisions 62 and 69 put a gap and a tie inside `Interior`: 1a's
`Regime` keeps its variants, and `regime.interior()` returns the equilibrium whatever the
stretch. 1d never returns `BoundaryNoMargin` or `NoInteriorAtZero`. Open question 2.

### 2.6 Bitwise nesting through a normative evaluation order

As 1b and 1c (decision 63): with one worker type of efficiency 1 and support 1 and no
human-required or reserved hours, every new operation of §5.1 is an exact no-op
(x + 0.0, 1.0·x, x/1.0, 0.0 + x, x − 0.0), and wherever 1c returns `Interior`, 1d returns
the same equilibrium bit for bit (§8, d1), unless the wall stretch holds further
equilibria, which needs a switch on the wall that raises labour demand (with interest,
§5.4) and makes the economy `MultipleEquilibria`. Where 1c returns a boundary regime, 1d
solves it, and its line values at 1 and lo are 1c's diagnostics bit for bit.

## 3. Parameters, validation and instances

### 3.1 Parameters

1c's parameters stay (unit-1c.md §3.1), except `workers` and `work_cost`, which move into
the worker types. New:

| symbol | name in code | meaning |
|---|---|---|
| N_i | `worker_types[i].workers` | potential workers of type i, one hour each per period |
| F_i | `worker_types[i].work_cost` | cdf of type i's work cost χ, uniform on [0, χ_max,i] (1a's `UniformWorkCost`) |
| ε_i | `worker_types[i].efficiency` | efficiency hours per hour at contestable and common human-required tasks; 0 = reserved tasks only |
| ν_i | `worker_types[i].support` | baskets of support per type-i worker, in work and out of it (living cost) |
| L^H_j | `human_required[j]` | efficiency hours per unit of category j at tasks closed to machines that any pooled worker can do (the paper's H) |
| R_ji | `reserved[j][i]` | hours of type i per unit of category j at tasks closed to machines and to every other type |

Derived at construction (x-free), with 1c's chain (I − A_cc)⁻¹ and basket outputs ŷ:

| symbol | meaning |
|---|---|
| L̃^H = (I − A_cc)⁻¹L^H | common human-required hours per unit, through the chain |
| R̃_{·i} = (I − A_cc)⁻¹R_{·i} | type i's reserved hours per unit, through the chain |
| L^H_ŷ = Σ_j ŷ_j L^H_j | common human-required hours per basket |
| R_ŷi = Σ_j ŷ_j R_ji | type i's reserved hours per basket |
| ν = Σ_i ν_i·N_i | baskets of support the provider funds |

**Notation.** v is the pool's wage per efficiency hour (1a-1c's v); v_i is type i's wage per
hour; ω = v/P_s is the pool's real wage; ω_i = v_i/(ν_i·P_s) is type i's real wage per
basket of support; ζ_i is the ω_i that clears type i's reserved market (§4.3); π = p_τ/θ_τ
is the delivered machine-task price (1c §4.4); g = v/π is the pool's wage in machine-task
units, equal to γ(x*) on the line.

### 3.2 Validation

1c's checks (unit-1c.md §3.2 and §12 item 4) in 1c's order, less N and χ_max, then:
- at least one worker type; each type's `workers` and `work_cost.chi_max` in
  [`SCALE_FLOOR`, `SCALE_CEIL`], `support` in that range, `efficiency` 0 or in it, all
  finite, −0.0 stored as +0.0 (1a §2's rules); errors are `ParamError::Item { kind:
  "worker type", index, .. }`;
- at least one type with ε_i > 0 (the pool; machine building and the contestable line need
  it);
- `human_required`: C entries, `reserved`: C rows of I entries, each 0 or in the scale
  range (`Invalid { name: "human_required" | "reserved" }`, `Item` per category);
- every type has work: ε_i > 0 or R_ŷi > 0;
- each category priced (1b's rule through the chain): L̄_j + L̃^H_j + Σ_i R̃_ji > 0 or
  b̄_j > 0;
- the basket needs pool work at x = 0: Σ_j ŷ_j(L̄^dir_j + L^H_j) > 0 (1c's chain-hours rule
  with the common tail). It makes the excess demand positive at the start of the path.

With one type and no human-required or reserved hours these are 1c's rules, and 1c's
rejected rows are rejected, the worker's parameters named as type 0's (§8, d1).

### 3.3 Instances

All are constructed on 2026-09-27. Scalars not named are 1a's G1 (T 10, a 0.3, λ 0.05,
b 0.4, γ = η(0.2 + 0.8x), χ_max 1, (ρ, δ, J_b) = (0, 1, 1)). Worker types are written
(N, χ_max, ε, ν).

**D0, 1a to 1c in one-type form.** `WorkerParams::from_machines(MachineParams)`: one type
(N, χ_max, 1, 1), `human_required` zeros, `reserved` a zero column. Every 1a, 1b and 1c
golden instance, random draw, regime row and rejected row goes through it (§8, d1).

**W, the corners in Appendix B's closure** (1a's G8 rows, one type):

| instance | change from G1 | 1a's regime | 1d's result |
|---|---|---|---|
| W1 | λ 0.6 | `BoundaryNoMargin`, f(1) +0.719 | Wall |
| W2 | N 0.25 | `BoundaryNoMargin`, f(1) +0.226 | `LaborShort`: n_D(1) = 15/47 > N, supply saturates |
| W3 | λ 0.6, χ_max 3 | `BoundaryNoMargin` | `LaborShort`: the real-wage ceiling (below) |
| W4 | N 20, χ_max 0.05 | `NoInteriorAtZero`, f(lo) −10.000000000011 | all-human corner |
| W5 | N 10, T 10.000000000005, χ_max 1e-3 (the doubles) | `NoInteriorAtZero` although T/h > N | a root at 4.5e-13, below lo |
| W6 | a = 1 − 2^-53, λ 0 | `NoInteriorAtZero`, f(lo) −2.7587 | a root at 3.6e-15, below lo |

W5 takes every input as the double the oracle reads: its root is set by f(0) = T − N,
which is 5.0004445e-12 for the double 10.000000000005 and 5e-12 for the decimal, so the
root moves by 8.9e-5 relative between the two.

**B, the human-required economy** (Baumol's economy; one type). Edges (0, 1); categories
services (z 1, b 0, μ 0.75, L^H 0.25), goods (z 1, b 0, μ 1, L^H 0) and space (z 1, b 1,
μ 0, L^H 0). Services is check_kset P9-i's good: a task list of measure 1 of which
k = |H| = 0.25 is closed to machines, all at γ_L = 1; goods is its H-free good.
- **B1**: N 8, η 1: a contestable margin with the human-required tail.
- **BP**, the automation path of check_pinning D1: N 8, η ∈ {0.3, 0.1, 0.01, 0.001,
  10^-6}, every point at the wall; and the limit η → 0.
- **B2**, the tail decides the regime: N 4, with L^H_services 0 (contestable) and 0.25
  (wall). Both are funded.

**E, entrant and trained** (B's economy, η 1 unless stated). Entrant (N_E, 1, 1, 1);
trained (N_T, 0.8, 1.5, 1.2) with reserved hours 0.1 per unit of services and 0.02 per
unit of goods.

| instance | N_E, N_T, η | result |
|---|---|---|
| E1 | 8, 3, 1 | contestable, both pooled: v_T = 1.5·v exactly |
| E2 | 8, 1, 1 | contestable, trained at its wall (premium 2.20 over 1.5·v) |
| E3 | 8, 2, 0.3 | wall, trained at its wall |
| E4 | 8, 3, 0.3 | wall, both pooled |
| E5 | 40 (χ_max,E 0.05), 2, 1 | all-human corner, trained at its wall |
| E6 | 8, 0.5, 1 | `LaborShort`: trained's reserved demand exceeds N_T along the whole path |

**E7 and E8, three types and the walk.** E's economy with a third type, master
(N_M, 0.6, 1.8, 1.5), reserved hours 0.05 per unit of goods, and trained's reserved hours
0.1 per unit of services only.
- **E7**: N_E 8, N_T 3, N_M 0.6: trained pooled, master at its wall. The master's
  threshold is below the trained's, so a walk in index order that stops at the first pooled
  type misses it.
- **E8**: N_T 1.53, N_M 0.5: walling the master raises P_s past the trained's threshold
  (P_s from 1.944076 to 1.956623, the trained's threshold 1.952732), so both are at their
  walls. A walk that does not recompute P_s after walling a type finds the trained pooled.

**F, the full economy.** 1c's M4 (1b's fork economy with intermediate inputs, the loom,
engine and power types, ρ 0.04) with L^H_care 0.2 and two types: entrant (N_E, 1, 1, 1),
trained (N_T, 0.8, 1.4, 1.2) with reserved hours 0.05 per unit of food and 0.3 per unit of
care. Care buys food (a_care,food 0.2), so care's reserved cost carries food's through the
chain.
- **F1**: N_E 4, N_T 3, η 1: contestable with the engine, trained at its wall.
- **F2**: N_E 4, N_T 3, η 0.25 (1c's M4 is `BoundaryNoMargin` at η 0.25): the loom
  holds the whole line; the switch to the engine lies on the wall, at v_s 0.0650, below
  the equilibrium, which is at the wall with the engine.
- **F3**: N_E 16, N_T 1.5, η 0.5 (near 1c's M4t): a tie at the line's switch with the
  trained at its wall. The walled wage moves with the split, so the split is not 1c's
  closed form (§4.6).

**X, switches at the wall.** 1c's M5 (the good and space, a flow and a durable task type)
at ρ 0.15 and χ_max 3. The flow type holds the whole line; the durable type, labour-light,
is cheaper above g_s = 5.794 (v_s = 6.888), on the wall.
- **X1**: N 0.5: a tie at the wall's switch.
- **X2**: N 1: at the wall below the switch (flow).
- **X3**: N 0.003: at the wall above the switch (durable).

**Random economies** (§8, d7).

**What each instance catches.** Each is built so that a plausible wrong unit misses its
goldens by far more than the gate's tolerance (the prototype's values):

| instance | a unit that … | gets |
|---|---|---|
| W1 | pins the wage at the top task's replacement value, as if the margin held there | v(1) = 4, not 12.348 |
| W2, W3 | ignores saturation or the real-wage ceiling | a wall wage, where there is none |
| W4 | stops at lo with the margin's wage v(lo) | v 0.1159, not 0.02597 |
| W5, W6 | keeps 1a's `NoInteriorAtZero`, or bisects arithmetically below lo | no root, or 128 steps without one |
| B1 | leaves the tail out of labour demand (keeping it in prices) | x* 0.8549, not 0.9756 (0.8442 is the economy without the tail) |
| B2 | the same | contestable at x* 0.9392, not the wall |
| BP | prices the machine at the wall by the margin's b/(1 − a − λγ) | p_m off by λv/(1 − a), 5% at η 10^-6 |
| E1 | walls a type whose pooled wage covers its reserved work | a premium above 1 |
| E2 | treats types as efficiency units only | premium 1 and x* 0.9289, not 2.204 and 0.9942 |
| E6 | caps a reserved supply at N_i and carries on | an equilibrium, where there is none |
| E7 | walks the types in index order and stops at the first pooled one | the master pooled |
| E8 | does not recompute P_s after walling a type | the trained pooled, premium 1 not 1.0021 |
| F1 | prices reserved hours directly, not through the chain | care's reserved cost 0.12352, not 0.12763 |
| F2 | keeps the line's technique on the wall | v 0.3109, not 0.1165 |
| F3 | uses 1c's closed form for σ with a walled type | σ 0.89374, not 0.89336 |
| X1-X3 | stops the envelope at γ(1) | the flow type on the whole wall: X1 and X3 wrong |

## 4. Equations

Per period, r = 1. x is a candidate threshold on the contestable line, τ a technique (1c),
i a worker type, j a category, k a machine type.

### 4.1 Tasks and human-required hours

1b §4.1 and 1c §4.1 unchanged on the line: H_j(x) and M_j(x). The common human-required
hours add to the human hours of every threshold (SSRN p.26: final-task demand "including
the human-required tail"):

    h_j(x) = H_j(x) + L^H_j                         efficiency hours at pool tasks, per unit of j
    R_ji                                            hours of type i at its reserved tasks, per unit of j

L̄_j, the all-human hours of 1b, become L̄_j + L̃^H_j for the pool's bound (§4.7).

### 4.2 Price block

**Machines.** On the line, 1c §4.2 unchanged: the (O, V) system with the margin column
gives p_k, O_k, V_k and v = γ(x)·p_τ/θ_τ. At a corner v is given, and the machine prices
come from the same system without the margin column:

    p = Âp + λ̂v + b̂,   p_k = v·λ̃_k + b̃_k,   π = p_τ/θ_τ                 SSRN eq 4, A.1

with τ the cheapest task type at v, min_t p_t/θ_t (1c §4.3; at the wall this is the
envelope continued above γ(1), §5.2).

**Categories.** Reserved labour is priced at its type's wage v_i, which may be set outside
the pool:

    p_j = Σ_l a_jl p_l + v·h_j + π·M_j + b_j + Σ_i v_i·R_ji                 SSRN eq 2, row j
    p_c = p⁰_c + Σ_i v_i·R̃_{·i},    p⁰_c = (I − A_cc)⁻¹(v·h + π·M + b_c)

**The basket.**

    P_s = zᵀp_c = P⁰_s + Σ_i v_i·R_ŷi,    P⁰_s = zᵀp⁰_c = v·L_s + B_s

L_s and B_s are 1c's price-side basket totals with h in place of H (they include the
common human-required hours). P⁰_s is P_s without the reserved costs.

### 4.3 Reserved markets and type wages

Type i's reserved demand is D_i = Y·R_ŷi hours (Y from §4.4, which does not depend on
wages). If its wage is v_i, it offers n_S,i = N_i·F_i(ln(1 + ω_i)) with ω_i = v_i/(ν_i·P_s).
The real wage per support basket that makes it offer exactly D_i is

    ζ_i = e^{F_i⁻¹(D_i/N_i)} − 1 = expm1(χ_max,i·D_i/N_i)           (uniform F; D_i ≤ N_i)

If D_i > N_i, no wage clears type i's reserved market at that point of the path: the
point is **short** and its excess demand is +∞ (§5.3). At D_i = N_i the type's supply is
vertical, and every real wage at or above expm1(χ_max,i) clears its market; the path takes
the least, ζ_i, except at the edge of a shortage, where the pool's clearing sets it (§12
item 16).

Type i sells at the pool's wage if that covers its reserved work, and otherwise at the
wage that clears its reserved market:

    v_i = max(ε_i·v, ζ_i·ν_i·P_s)

At the first, type i is **pooled**, does its reserved hours and offers the rest,
ε_i·(n_S,i − D_i) efficiency hours, to the pool. At the second it is **at its wall**:
all its hours are reserved, and its premium v_i/(ε_i·v) exceeds 1. The two agree when
ε_i·ω/ν_i = ζ_i, where the type's pool hours are 0, so the switch is continuous. (At
equality the type counts as pooled.)

**The walk.** P_s appears on both sides:

    P_s = P⁰_s + Σ_i max(ε_i·v·R_ŷi, ζ_i·ν_i·R_ŷi·P_s)

The right side is increasing, convex and piecewise linear in P_s. Its least fixed point is
found in at most I steps. Let e_i = ε_i·v (type i's pooled wage) and c_i = ζ_i·ν_i (its
walled wage per unit of P_s). The candidates are the types with R_ŷi > 0 and ζ_i > 0,
ordered by their thresholds t_i = e_i/c_i, the P_s above which each is walled (ties to the
lower index). Start with every type pooled, P = P⁰_s + Σ_i e_i·R_ŷi. For each candidate in
order: if c_i·P > e_i, wall it and recompute

    P = (P⁰_s + Σ_{pooled} e_i·R_ŷi) / (1 − Σ_{walled} c_i·R_ŷi)

and continue; otherwise stop. If a denominator is ≤ 0, the walled types' wages cost more
than the basket they buy, no finite price system exists at that point, and the point is
short. *Proof that this is the least fixed point.* The right side is at least each of its
linear pieces, so every fixed point is at least the pieces' fixed points reached so far; P
never falls along the walk; when it stops, the walled types have t_i < P and the others
t_i ≥ P, so P is a fixed point. ∎ E7 and E8 are built to fail a walk that goes in index
order or does not recompute P.

In real-wage terms, type i is walled exactly when ε_i·ω/ν_i < ζ_i, which is how the corners
decide it (§4.5).

### 4.4 Quantities, supply and the pool's clearing

1c §4.5 with h for H: the quantities depend only on x and the technique (and a tie's
split), not on any wage.

    Y = T/(B_ŷ + b^qᵀ_m x̂),  X = Y·x̂,  final_hours = Y·H_ŷ  (H_ŷ = Σ_j ŷ_j h_j)
    n_D = final_hours + machine_hours                 the pool's demand, in efficiency hours
    D_i = Y·R_ŷi                                      reserved demand, in hours of type i
    n_S,i = N_i·F_i(ln(1 + v_i/(ν_i·P_s)))            SSRN eq 10 per type
    S = Σ_{pooled} ε_i·(n_S,i − D_i)                  the pool's supply, in efficiency hours
    f = n_D − S                                       the excess demand

Labour clearing is f = 0 for the pool and n_S,i = D_i for every walled type (which the
walk's ζ_i gives). The final hours include the tail Y·L^H_ŷ (SSRN p.26).

### 4.5 The corners

At x ∈ {0, 1} and a fixed technique the quantities (n_D, D_i, ζ_i) are fixed, and the
excess demand depends on prices only through the pool's real wage ω = v/P_s:

    S(ω) = Σ_{i: ε_i·ω ≥ ζ_i·ν_i} ε_i·(N_i·F_i(ln(1 + ε_i·ω/ν_i)) − D_i),      f(ω) = n_D − S(ω)

S is nondecreasing. Given ω, the wage follows in closed form. With the walled set at ω,
P_s(1 − C) = v·L + B, where

    L = L_s + Σ_{pooled} ε_i·R_ŷi,   C = Σ_{walled} ζ_i·ν_i·R_ŷi,   B = B_s
    v = ω·B / ((1 − C) − ω·L)                                          (§4.2)

v is increasing in ω up to the **ceiling** ω_max, where (1 − C) − ω·L = 0: the pool's real
wage cannot exceed what the basket's embodied labour allows (SSRN eq 12: w/p_i =
1/(λ̃_i + b̃_i·r/w) < 1/λ̃_i, here for the basket with the walled types' reserved hours
priced at their own wages).

**The wall** (x = 1, v ≥ v(1); SSRN §3.1). Every contestable task is automated: H_j(1) = 0,
so pool demand is the human-required tail and machine building,
n_D = Y·(L^H_ŷ + λ^qᵀ_m x̂). The pool's wage is set by labour clearing, not by the task
margin, and v > γ(1)·π: "the machine comparison does not pin the wage". With one type and
no reserved hours the wall has a closed form:

    ζ = expm1(χ_max·n_D/N),   ω = ζ,   v = ζ·B_s/(1 − ζ·L_s)                    (W1, B2)

It exists when n_D < N and ζ·L_s < 1. At the wall the machine price is the recursion
priced at the wall's own wage, p_m = v·λ̃_m + b̃_m, which is (λv + b)/(1 − a) in Appendix
B's closure: bounded and positive (SSRN Appendix E's premise, "the rental p_m remains
bounded").

**The all-human corner** (x = 0, v ≤ v(0)). Every contestable task is human: M_j(0) = 0,
no machine services are used (X = 0, K = 0, interest 0), and n_D = Y·Σ_j ŷ_j(L̄^dir_j +
L^H_j) with Y = T/B_ŷ. Machine prices are shadow prices, reported at the cheapest task type
at the corner's own wage; every delivered machine cost exceeds labour's (SSRN A.1: "unused
feasible methods have unit costs at least as high"): v ≤ γ(0)·π. Each category is at its
upper bound, p_j = v·(L̄_j + L̃^H_j) + b̄_j + Σ_i v_i·R̃_ji.

**The end of the wall** (v → ∞, under the wall's last technique τ_e). P_s/v tends to ρ_∞,
the least fixed point of the walk with P⁰_s replaced by its v-coefficient L_s and e_i by
ε_i: ρ = L_s + Σ_i max(ε_i·R_ŷi, ζ_i·ν_i·R_ŷi·ρ). Then ω_∞ = 1/ρ_∞ (+∞ when ρ_∞ = 0), and

    f_∞ = n_D − S(ω_∞)                  (+∞ if a reserved market is short at x = 1, or the walk has no fixed point)

f_∞ ≥ 0 means that no finite wage on the wall clears the pool. W2 has n_D(1) = 15/47 > N:
f_∞ = 13/188. W3 has ζ·L_s ≥ 1: the pool's supply at the ceiling, N·F(ln(1 + 1/L_s)), is
below n_D(1), and f_∞ = 2.3899.

### 4.6 A tie at a switch

1c §4.7 at a switch of the line (x_i, with τ_{i−1}'s prices) or of the wall (v_s): a share
σ of the machine tasks goes to the type above. The quantities are the σ-mix, and so are Y
and every D_i. When no type is walled under either pure technique at the switch, prices do
not depend on σ, and 1c's closed form holds with the pool's supply at each technique:

    σ = B_a·f_a/(B_a·f_a − B_b·f_b),   f_t = n_D,t − S_t

(S_t differs between the two only through D_i = Y_t·R_ŷi, which keeps f linear-fractional
in σ.) For one type with no reserved hours S_a = S_b = n_S, and this is 1c's expression. When
some type is walled at either end, its wage ζ_i·ν_i·P_s moves with Y(σ), P_s moves with it,
and σ is found by bisection of f(σ) on [0, 1] (§5.3). At F3 1c's formula would be off by
4.3e-4 relative (0.89374 against 0.89336).

### 4.7 Outputs and identities at the equilibrium

1c §4.8's outputs, with these meanings (the one-type case is 1c's):
- **Wages.** v (the pool's, 1c's v); v_i per type; `real_wage` v/P_s; per type v_i/P_s,
  the premium v_i/(ε_i·v) (absent when ε_i = 0) and the marginal work cost
  ln(1 + v_i/(ν_i·P_s)), at which a type-i worker's exit value (e^χ − 1)·ν_i·P_s equals v_i.
- **Hours.** Type i's hours are D_i plus, if pooled, its share of the pool's demand split
  in proportion to the pooled types' net supplies:

      s_i = ε_i·(n_S,i − D_i),  S = Σ s_i,  pool_i = s_i + (n_D − S)·(s_i/S)  (0 when S = 0),
      hours_i = D_i + pool_i/ε_i

  so that Σ_pooled pool_i = n_D. With one type, pool = n_S + (n_D − n_S) = n_D exactly
  (Sterbenz: at an equilibrium n_D and n_S are within a factor of two), and hours = n_D,
  1c's n_a. `n_a` = Σ_i hours_i, `n_pool` = n_D (efficiency hours at pool tasks),
  `participation` = min(n_a/Σ_i N_i, 1), per type hours_i/N_i.
- **Income** (SSRN App. C with reserved labour):

      I = Σ_i v_i·hours_i + T + interest = v·n_D + Σ_i v_i·D_i + T + interest = Y·P_s = Σ_j p_j·z_j·Y
      labour share = (Σ_i v_i·hours_i)/I,  support cost = ν·P_s,
      worker baskets = ν + (Σ_i v_i·hours_i)/P_s,  provider baskets = (T + interest)/P_s − ν

  funded = provider baskets > 0; lemma_b1 = f_line(1) < 0 and T > ν·P_s(1) (eq 25 with the
  support ν).
- **The fork with reserved costs.** Reserved labour enters a category's price as a direct
  requirement priced outside the pool, R_j = Σ_i v_i·R̃_ji:

      p_j = v·L*_j + b̄_j + R_j = v·λ̃_j + b̃_j + R_j
      L*_j = (I − A_cc)⁻¹(h + M/g)_j,   λ̃_j = (I − A_cc)⁻¹(h + M·λ̃_τ/θ_τ)_j       (g = γ(x*) on the line, v/π at a corner)
      b̄_j + R_j ≤ b̃^q_j + R_j ≤ b̃_j + R_j ≤ p_j ≤ v·(L̄_j + L̃^H_j) + b̄_j + R_j,     L*_j ≤ L̄_j + L̃^H_j

  At the all-human corner the upper bound is attained, and b̃_j = b̄_j. At the wall
  g = v/π > γ(1), and the bound is strict for every category with contestable tasks.
- **Cost-system identities** (1c §4.5-4.6): p = Ap + λv + b + R over every row (R the
  reserved costs on the category rows); f = (I − A^qᵀ)y = (Yz, 0); n_D = λ^qᵀy (the pool's
  hours); T = b^qᵀy; eq 11 for the pool, Y = T/B^q_s and n_D = T·L^q_s/B^q_s; pᵀf =
  v·n_D + Σ_i v_i·D_i + T + interest.
- **The margin.** On the line v = γ(x*)·π (1c's closure); at the wall v ≥ γ(1)·π; at the
  all-human corner v ≤ γ(0)·π; everywhere p_t/θ_t ≥ π for every task type.
- **φ at u = 1** (1b, 1c): per category φ_w,j = (v·λ̃_j + R_j)/p_j, labour's share of the
  price, pool and reserved; φ_r,j = b̃_j/p_j.

### 4.8 The Baumol limits (check_pinning D1; SSRN Prop E.1)

Along BP, as η → 0 with the economy at the wall:
- the machine price stays bounded: p_m = (λv + b)/(1 − a) exactly at every point (D1(i)'s
  "c pinned … not zero"; D1's c = br/(1 − a − λγ) is the margin's, which the wall does not
  have; the two agree at λ = 0);
- services' unit cost tends to |H|·v_H/γ_L: p_services/v → 0.25 (D1(i); P9-i's C → k·w_K/γ_L),
  and labour's share of services' cost, φ_w,services, tends to 1;
- goods' price tends to 0, and the relative price of H-content, p_services/p_goods,
  diverges (D1(iii), D1(ii));
- the CES share of H-content at α = 0.3, σ_H = 0.5, `ces_share(α, σ_H, p_services/p_goods)`
  (1b §4.5), tends to 1 (D1(ii), Prop E.1(ii); a comparison conditional on prices, as 1b
  uses it; check_pinning.py:168-178 checks the limits at α = 1/4);
- the wall's wage converges to a positive limit (SSRN E's premise, "w_H and r converge to
  positive finite values"): with J(1) → 0 the machine sector vanishes, Y → T/B_d = 10,
  n_D → Y·L^H_ŷ = 2.5, and

      v_∞ = ζ/(1 − 0.25·ζ),   ζ = expm1(χ_max·2.5/N) = expm1(5/16)

  = 0.40387732254620515981.

## 5. Solution method

### 5.1 The evaluation (normative)

**At (x, τ) on the line**, 1c §5.1 steps 1-4, with these changes:
1. γ, J (1a).
2. The machine block with the margin column (1c step 2): p_k, O_k, V_k, v, π, d.
3. Categories (1c step 3) with h_j = H_j + L^H_j, one addition (`h + human_required[j]`):
   rhs_j = (v·h_j + π·M_j) + b_j; p⁰_c by 1c's substitution; then, in category order,
   P⁰ += z_j·p⁰_j, H_ŷ += ŷ_j·h_j, M_ŷ += ŷ_j·M_j.
4. Quantities (1c step 4): Y, X, machine hours, final_hours = Y·H_ŷ, n_D.
5. Reserved demand, in type order: D_i = Y·R_ŷi; if D_i > N_i the point is short;
   ζ_i = expm1(F_i⁻¹(D_i/N_i)).
6. The walk (§4.3): e_i = ε_i·v, c_i = ζ_i·ν_i; both sums from 0.0 in type order;
   P = (P⁰ + Σ_{pooled} e_i·R_ŷi)/(1.0 − Σ_{walled} c_i·R_ŷi), the candidates visited in
   the order of §4.3; P_s = P; v_i = e_i or c_i·P_s.
7. Supply, in type order: n_S,i = N_i·F_i(ln1p(v_i/(ν_i·P_s))); S from 0.0,
   S += ε_i·(n_S,i − D_i) over pooled types; f = n_D − S.
8. Prices: p_j = p⁰_j + Σ_i v_i·R̃_ji (the sum from 0.0 in type order, added once).

For one type with ε = ν = 1 and no human-required or reserved hours, every change is exact:
h_j = H_j + 0.0; D = Y·0.0 = 0.0, ζ = expm1(0.0) = 0.0 and there is no candidate; P =
(P⁰ + (1.0·v)·0.0)/(1.0 − 0.0) = P⁰; v_0 = 1.0·v; n_S = N·F(ln1p(v/(1.0·P))), 1c's;
S = 0.0 + 1.0·(n_S − 0.0) = n_S; p_j = p⁰_j + 0.0.

**At (x, v, τ), a corner's evaluation**, x ∈ {0, 1}: step 2 is the wage-given system,
the (O, V) system without the margin column and with right-hand sides b^op_k + λ^op_k·v and
b^I_k + λ^I_k·v; its factors do not depend on x or v and are stored at construction; its
least pivot is `d`. Steps 3-8 as above.

**At the root**: 1b step 5 (the top segment's hours from the carried 1 − x*). At a corner
x* is exact: 1.0 − x* is 0.0 at the wall and 1.0 at the all-human corner.

### 5.2 At construction

1c §5.2, then: L̃^H, R̃ (substitution with the stored factors of I − A_cc), L^H_ŷ, R_ŷi, ν,
and the factors of the wage-given (O, V) system. The envelope is 1c's walk (unit-1c.md §5.2
step 3) continued above γ(1): a crossing counts when γ_c < γ_τl and θ_l − γ_τl·λ̃_l > 0,
with no upper limit. The walk's choices up to γ(1) are 1c's, since every crossing it takes
there is below every crossing above γ(1); the switches with γ_s ≤ γ(1) are the line's and
the rest the wall's. A wall switch sits at the wage v_s = γ_s·b̃_a/(θ_a − γ_s·λ̃_a) (the
closure wage of the type below, 1c §4.3), where p_a/θ_a = p_b/θ_b. The wall's last piece,
τ_e, has the least λ̃_t/θ_t (the cheapest as v → ∞).

### 5.3 The solve

1. **Not viable.** 1c's test: d(1) ≤ 0 under the line's last technique is
   `NotViable { d_at_1 }` (decision 64's convention, unchanged).
2. **The sequence.** Evaluate, in this order: f_line(0) under τ_0 at x = 0; f_line(lo)
   under τ_0 at lo = `BRACKET_LO`; at each line switch x_i, f under the technique below and
   above (1c's switch points); f_line(1) under τ_m; at each wall switch v_s, f under the
   technique below and above (the corner's evaluation at x = 1); f_∞ (§4.5). A short point
   is +∞.
3. **Sides.** Put a positive side first, for v → 0, where S → 0 and f → n_D(0) > 0 (§3.2).
   A value is on the positive side when > 0, except f_line(1) and f_∞, which are when ≥ 0
   (1a's convention at f(1) = 0; an equilibrium at v = ∞ is none), and f_∞ = 0 after a last
   piece that starts at an exact zero, which is not (§12 item 17). Each change of side is
   one equilibrium. None is `SolveError::LaborShort { excess: f_∞, reserved }` (`reserved`
   the first type short at the end of the path). More than one is
   `SolveError::MultipleEquilibria { sign_changes, switches }` (the line's x_i; the count
   covers the whole path). One:
   - between the first side and f_line(0): the **all-human corner**. Bisect f(ω) (§4.5) at
     x = 0 on [0, ω_line(0)], with f(0) = n_D(0) and f(ω_line(0)) = f_line(0)
     (ω_line(0) = v(0)/P_s(0) from the line's evaluation); an exact zero there is the
     junction (x = 0, v = v(0)). Then v from ω (§4.5), and the technique reported is the
     cheapest task type at v, ties to the lower index;
   - between f_line(0) and f_line(lo): a **root in [0, lo]** under τ_0, by bisection on the
     doubles' bit patterns (below); an exact zero at lo is `Root::exact(lo)`;
   - inside [lo, 1]: 1c §5.3 step 3 unchanged (a root by 1a's `bisect`, or a tie at a line
     switch, §4.6);
   - between f_line(1) and the next value: a **root on the wall's first piece**, under τ_m:
     bisect f(ω) on [ω_line(1), ω(v_s1)] (or [ω_line(1), ω_∞) when there is no wall switch),
     the ends' values being the sequence's; an exact zero at f_line(1) is the junction
     (x = 1, v = v(1));
   - between a wall switch's two values: a **tie on the wall** at v_s (§4.6);
   - after a wall switch: a root on that piece, bisecting f(ω) under its technique, up to the
     next switch's ω or ω_∞;
   - where a bisection in x or in a tie's σ closes on a short point (+∞): the **edge of that
     reserved shortage**, at the bracket's finite end, with the short type's κ_i by bisection
     on bit patterns over [ζ_i, +∞] until the pool clears (§12 item 16).
4. **Report** at the equilibrium (§4.7), the finiteness check of every output (1c's errors,
   and `SolveError::NonFiniteInWorker { worker, what }`), and 1a's labour net on the pool:
   |n_pool − S|/n_pool above `LABOR_RESIDUAL_NET` is `LaborNotCleared`.

**Bisection on bit patterns.** For doubles 0 ≤ a < b, the double whose bits are the integer
midpoint of a's and b's bits lies in [a, b], strictly inside while a and b are not adjacent,
since the bits of nonnegative doubles are ordered as their values (+∞'s included). It halves
the count of doubles between the ends, so it reaches adjacent doubles in at most 64 steps
from any bracket in [0, +∞], within `MAX_BISECTION_STEPS`. It takes the [0, lo] stretch
(roots down to the subnormals, where 1a's arithmetic midpoint would need up to about 1100
steps), the corners' ω (the wall's last piece may reach ω_∞ = +∞) and a tie's σ. It stops
and chooses its end as 1a's `bisect` does. 1a's arithmetic bisection stays on [lo, 1], so
the line nests.

**Which equation fixes which unknown.**
- The envelope, continued above γ(1), fixes the technique at each point of the path.
- On the line the task margin fixes v given x; at a corner the pool's clearing fixes ω, and
  so v, given x ∈ {0, 1}.
- Each type's reserved market fixes its wage when it is walled (ζ_i·ν_i·P_s); otherwise
  the pool's wage does (ε_i·v). The walk fixes P_s given v.
- Machine rows fix p_k; zero profit fixes each p_j, with reserved costs; land clearing
  fixes Y; service clearing fixes X_k.
- The pool's labour clearing is the root equation (in x on the line, in ω at a corner), or
  fixes σ at a tie.
- Walras gives the income identity and the basket count, which are checked.

### 5.4 Uniqueness

**Lemma 1'** (1c's Lemma 1 on the whole path). The machine totals are x-free and wage-free,
so the envelope continued above γ(1) still visits each type once: at most K − 1 switches
along the whole path.

**Lemma 2'** (within a technique on the line f is nonincreasing). n_D is nonincreasing and
Y with it (1c's Lemma 2; the tail L^H_ŷ is x-free), so every D_i and ζ_i is nonincreasing.
P⁰_s/v = H_ŷ + M_ŷ/γ + B_ŷ/v falls (1b §5.3; the tail adds a constant). The walk's least
fixed point is nondecreasing in P⁰_s and in each c_i, so P_s/v is nonincreasing, and every
pooled type's real wage ω_i = ε_i·v/(ν_i·P_s) is nondecreasing. So each pooled type's net
supply ε_i·(n_S,i − D_i) is nondecreasing; a type changes status where its net supply is 0,
continuously; S is nondecreasing and f nonincreasing. Where a point is short (+∞), it is so
on a left part of the stretch, since D_i and the walled types' shares only fall along it.

**Lemma 3'** (the corners). At fixed x and technique the quantities are fixed, S(ω) is
nondecreasing, and ω is increasing in v (§4.5), so f is nonincreasing in v.

**Lemma 4'** (the junctions). The corners meet the line at (x, v) = (0, v(0)) and (1, v(1)),
where the two evaluations agree in exact arithmetic (the prototype found them equal at 70
digits on four instances): f is continuous there.

**Proposition (ρ = 0).** At a switch on the line or the wall, 1c's Proposition gives less
labour and more land per basket above it, so n_D and Y jump down; D_i and ζ_i fall with Y,
P_s/v falls, and S jumps up. So f is nonincreasing along the whole path, the equilibrium is
unique if it exists, and it exists exactly when f_∞ < 0. The prototype found all 29
switches in 160 random ρ = 0 draws downward, and f nonincreasing on a grid of every stretch
of 63 other random ρ = 0 economies and of every constructed instance.

**At a tie with a walled type** (§4.6) f(σ) is monotone at ρ = 0 by the same argument (Y
falls with σ). With interest it is not proved; the bisection returns one σ. F3 (ρ 0.04) is
monotone on a 50-point grid of σ, and the build records any draw where it is not.

**With interest** a switch can raise labour demand (1c §5.5), on the line or on the wall;
the count decides, and a corner counts as one equilibrium like any other.

### 5.5 Precision

Everything in unit-1a.md §4 step 4, 1b §5.4 and 1c §5.6 carries over on the line, with the
one-type case bit for bit. New:
- **Roots below lo.** x* is resolved to about 2^-52·n_D/|f'| absolute (1a §4 step 4's
  conditioning), not relative: at W5 about 2e-16 against x* = 4.5e-13. The outputs that do
  not scale with x* keep full precision. The gate compares W5's and W6's x* at
  4·2^-52·n_D/|f'| absolute and the rest at 1e-12 relative.
- **The corners.** ω is resolved to adjacent doubles, or to the rounding of S over its
  slope. v = ω·B/((1 − C) − ω·L) carries ω's error amplified by (C + ω·L)/((1 − C) − ω·L),
  which grows without bound at the ceiling: 4.7 at W1 (ζ·L_s = 0.83). A wall near its
  ceiling has a wage that is ill-conditioned in the data, and so do its prices.
- **The walk.** P_s carries its inputs' rounding amplified by 1/(1 − C), C the walled types'
  share of the basket.
- **Walled types.** ζ_i from `expm1` and the supply's `ln1p` round-trip within a few ulps,
  so a walled type's residual |n_S,i − D_i|/D_i is of order 2^-52.
- **A type at its threshold** (ε_i·ω/ν_i within rounding of ζ_i): its status is not
  decidable in f64, but its pool hours are within rounding of 0 either way, and every
  output is continuous across the threshold.
- **Ties with a walled type.** σ by bisection to adjacent doubles, then 1c §5.6's
  statements about ties.
- **Ties at a wall switch.** Every output is evaluated at v = v_s, which carries the
  crossing's rounding amplified by (|b̃_b·θ_a| + |b̃_a·θ_b|)/|b̃_b·θ_a − b̃_a·θ_b|, large when
  the two delivered costs are nearly parallel, and σ carries that times |dlog σ/dlog v|: 1c
  §5.6's statement for a line switch, at the wall (§12 item 18 has a measured case).
- **The edge of a reserved shortage** (§12 item 16). x* (or σ) is the double on the edge's
  finite side, resolved to the rounding of D_i over its slope, absolute: about 1e-15 at J1b,
  whose x* is 5e-13, as W5's root. κ_i is resolved to adjacent doubles, or to the rounding of S
  over its slope in κ_i, and the type's reserved residual is of rounding order.
- **The Baumol path.** At η = 10^-6 the machine sector's outputs are of order η and keep
  their relative precision (no cancellation); the H-free good's price is p_m·J(1).
- **Decidability.** 1c's statement extends to every value of the sequence: a value within
  rounding of 0 (relative to n_D) makes the count not decidable in f64, and so does f_∞
  within rounding of 0. f_line(0) ≤ 0 < f_line(lo), excluded in exact arithmetic, can occur
  only there.
- **To be measured** by the build: the f64 values against every golden, as 1b and 1c did.

## 6. Result type

Proposed; the build may rename, and records any departure in §12.

- `src/workers.rs`: `WorkerType { workers, work_cost, efficiency, support }`;
  `WorkerParams<S>` (1c's `MachineParams` less `workers` and `work_cost`, plus
  `worker_types: Vec<WorkerType>`, `human_required: Vec<f64>` and
  `reserved: Vec<Vec<f64>>`, so that 1b's `Category` is unchanged), with
  `WorkerParams::from_machines(MachineParams)` building D0's form; `WorkerEconomy<S>` with
  `new`, `at(x)`, `at_with(x, technique)` (the line), `at_wage(x, v, technique)` (the
  corners' evaluation, for G1's sweeps over v), `solve() -> Result<Regime<Eq1d>,
  SolveError>`, and the x-free `human_required_per_basket()`, `reserved_per_basket()`,
  `support()`.
- `Margin { Contestable, Wall, AllHuman }`.
- `Eq1d`: 1c's `Eq1c` fields, with §4.7's meanings (`v` the pool's wage, `n_a` total hours,
  `final_hours` with the tail, `participation` over Σ N_i, `support_cost` ν·P_s, `d` the
  least pivot of the system that priced the machines), plus `margin`, `g` (v/π),
  `replacement_top` (γ(1)·π) and `replacement_bottom` (γ(0)·π), `n_pool`,
  `required_hours` (Y·L^H_ŷ), `reserved_hours` (Σ D_i), `wage_bill`, `f_line_0`, `f_line_lo`,
  `f_line_1` (the line's values at 0, lo and 1), `f_end` (f_∞), `wall_switches` (γ_s, v_s and
  the two types), `workers: Vec<WorkerEq>`, `categories: Vec<CategoryEq1d>` and
  `residuals: Residuals1d`. `margin_active` is false at a corner; `bisection_steps` counts
  the bisection that found the equilibrium (0 at a tie or an exact zero).
- `WorkerEq`: `wage` v_i, `real_wage` v_i/P_s, `efficiency`, `pooled`, `premium`
  (v_i/(ε_i·v), `None` at ε_i = 0), `hours`, `reserved_hours` D_i, `pool_hours` (pool_i/ε_i),
  `supply` n_S,i at the reported prices, `participation`, `clearing_real_wage` ζ_i,
  `marginal_work_cost` ln(1 + v_i/(ν_i·P_s)), `at_wall` (walled, or pooled with the
  margin at the wall), and `edge` (at the edge of its reserved shortage, §12 item 16).
- `CategoryEq1d`: 1c's `CategoryEq1c`, with `price` including the reserved costs, `l_bar`
  the pool's L̄_j + L̃^H_j, `wage_floor` 1/(L̄_j + L̃^H_j + (b̄_j + R_j)/v), `wage_ceiling`
  v/(b̃_j + R_j), φ per §4.7; plus `human_required` (L̃^H_j) and `reserved_cost` (R_j).
- `Residuals1d`: 1c's twelve, with `labor` = |n_pool − S| (1a's for one type), `closure`
  on the line only (0 at a corner), plus `corner` ((γ(1)π − v)⁺/v at the wall,
  (v − γ(0)π)⁺/v at the all-human corner, 0 on the line), `reserved` (the largest
  |n_S,i − D_i|/D_i over walled types) and `basket` (|P_s − Σ_j z_j·p_j|/P_s). Each nests:
  for one type the new ones are 0 and the old ones are 1c's bit for bit.
- `Eq1d::outputs()`: economy keys in 1c's order, then 1d's, then `switch<i>.`, `type<k>.`,
  `cat<j>.` as 1c, then `worker<i>.<field>` and `wall_switch<s>.<field>`. 1c's `Item` gains
  `Worker(i)` and `WallSwitch(s)`.
- `SolveError` gains `LaborShort { excess, reserved: Option<usize> }` and
  `NonFiniteInWorker { worker, what }`. `Regime::Interior`'s documentation says that in 1d
  it holds any equilibrium, and `BoundaryNoMargin`'s and `NoInteriorAtZero`'s that 1d
  solves them.
- No new `core::num` function: `expm1` is there (decision 5), and the bit-pattern midpoint
  is `f64::from_bits`/`to_bits`.

## 7. Goldens

`goldens/generate_1d.py` computes every golden with mpmath at 70 digits from §4's equations:
its own evaluation on the path, the walk as in §4.3, the corners by bisection of S(ω) to
2^-250 (and the one-type closed form asserted beside it), line roots and ties as
`generate_1c.py` does, a tie with a walled type by bisection of f(σ). It writes
`goldens/goldens_1d.txt` with 30 significant digits and a header of FNV-1a digests of
`generate_1d.py`, `generate_1c.py`, `generate_1b.py`, `generate.py` and its own body; the
Rust constants in `tests/gate/goldens_1d.rs` carry 20 digits (`d9`). It imports
`generate_1c.py` (and so the other two) only to assert the nesting. Key prefixes: `W1_` to
`W6_`, `B1_`, `BP_`, `BLIM_`, `B2_`, `E1_` to `E8_`, `F1_` to `F3_`, `X1_` to `X3_`.

The generator asserts as it goes:
- the one-type form of every 1c golden instance (and so 1a's and 1b's) equals
  `generate_1c.py`'s interior solve to 1e-65; its line values f(1), f(lo) equal 1c's
  boundary diagnostics; the prototype found them equal;
- every identity of §4.7 at every equilibrium to 1e-65 (both fork forms with reserved
  costs, the cost system over every row, eq 11 for the pool, income four ways, the basket
  count, land and services, the reserved markets, the walk's fixed point, the margin or the
  corner's inequality, the cheapest type) and every bound;
- f nonincreasing on a grid of every stretch within each technique, every ρ = 0 switch
  downward, the envelope continued above γ(1) against a scan of argmin_t p_t/θ_t on the wall,
  and the count equal to the instance's result;
- the corners' evaluation equals the line's at both junctions;
- W1's and B2's wall wages equal the closed form of §4.5, and BP's limit v_∞ (§4.8);
- W5 takes T and χ_max as the doubles; the prototype's W5 root with decimal inputs,
  4.5454545454521331974e-13, differs by 8.9e-5 relative.

The prototype's values, to 20 significant digits:

**W, the corners in Appendix B's closure.**
- W1 (λ 0.6), wall: v 12.347776347728814794, g = v/π 1.1069040032228707651 > γ(1) = 1,
  p_m = π 11.155236869481841252, P_s 7.6931421216891047511, Y 350/47
  (7.4468085106382978723), n_a 180/47 (3.8297872340425531915), I 57.289356225344397083,
  labour share 0.82544750615340180269, v/P_s = ζ = expm1(45/47) 1.6050368175205035052;
  f_line_1 0.71896895969051973502 (1a's G8 golden), f_end −8/47; funded false.
- W2 (N 0.25): `LaborShort`, excess 13/188 (0.069148936170212765957); f_line_1
  0.22635492751282969682 (1a's).
- W3 (λ 0.6, χ_max 3): `LaborShort`, excess 2.3898936931679436689 (the ceiling:
  ω_∞ = 1/L_s = 35/18).
- W4 (N 20, χ_max 0.05), all-human: v = ω/(1 − ω) with ω = expm1(1/40),
  0.025972620543831183633 < γ(0)·π 0.11465675172205473119; P_s 1.0259726205438311836,
  Y 10, n_a 10, K 0, π 0.57328375861027365597; f_line_lo −10.000000000011 (1a's), f_line_0
  −10.
- W5, a root below lo: x* 4.5458586390082282758e-13, v 0.1159420289857211242, P_s
  1.1159420289857211242, Y 10.000000000004480918, n_a 10; f_line_0 5.0004445029117050581e-12,
  f_line_lo −5.999555497094538288e-12.
- W6, a root below lo: x* 3.6175751314293239386e-15, v 720575940379289.78695, Y
  2.7725887222397884922, n_a 2.7725887222397784621; f_line_0 7.2274112777602215379, f_line_lo
  −2.7587301670408429713 (1a's).

**B, the human-required economy.**
- B1 (N 8, η 1), contestable with the tail: x* 0.97556565124183680037 (1 − x*
  0.024434348758163199633), v 0.60244952296500789847, p_m 0.61446068021178627846, P_s
  1.7955392957591152601, Y 6.3459650067529959603, n_a 2.3145997896628033234, final hours
  1.8578454155069278184 of which the tail 1.5864912516882489901 (= Y/4), I
  11.394429539137263482; p = (0.42700963003462195409, 0.36852966572449330596, 1).
- BP (N 8), every point at the wall:

| η | v | p_m = (λv + b)/(1 − a) | p_services/v | φ_w,services | p_goods | p_services/p_goods | CES share (0.3, 0.5) |
|---|---|---|---|---|---|---|---|
| 0.3 | 0.43450367471671539576 | 0.6024645481940510997 | 0.43718533061710840711 | 0.59389654446172425849 | 0.10844361867492919795 | 1.7516810579218691866 | 0.46422101645782538085 |
| 0.1 | 0.41390571390730221459 | 0.60099326527909301533 | 0.31534023578040307229 | 0.8029875575111173409 | 0.03605959591674558092 | 3.6195947873551328935 | 0.5546636639207168106 |
| 0.01 | 0.40487078336474958949 | 0.60034791309748211353 | 0.25667266130316155473 | 0.97525551533425132932 | 0.0036020874785848926812 | 28.849732847396458839 | 0.77857845517218100888 |
| 0.001 | 0.40397656969686354193 | 0.60028404069263311014 | 0.25066867199380989783 | 0.99746067535442658199 | 0.00036017042441557986608 | 281.15653973210352567 | 0.91650699681441674216 |
| 1e-6 | 0.40387742178231295406 | 0.60027695869873663958 | 0.25000066882825542059 | 0.99999745326522020177 | 3.6016617521924198375e-7 | 280341.77698321327824 | 0.99712330810402835357 |

  BLIM: v_∞ 0.40387732254620515981. Along the path v/(γ(1)·π) = g/γ(1) grows as 1/η while v
  converges: the machine no longer prices the hour.
- B2 (N 4): without the tail, contestable at x* 0.9321231698467138939, v
  0.57954753900939173627, P_s 1.6414896533693263696, Y 6.5190489932451689352, n_a
  1.2094805430763968357, provider baskets 2.0920274334193769204; with it, wall at v
  1.3486543727559672398 (g 2.0196661964108324913), P_s 2.0383126711456893529, Y 6.25,
  n_a 65/32, provider baskets 0.90601866021822189607, f_line_1 0.85496251779779224568.

**E, entrant and trained.**

| | margin | x* | v | P_s | Y | n_a | trained: v_T | premium | hours | reserved |
|---|---|---|---|---|---|---|---|---|---|---|
| E1 | contestable | 0.93526838038380481533 | 0.58120153393571462686 | 1.8917089345791874112 | 6.5064160111838815002 | 3.3621110178126007371 | 0.87180230090357194029 | 1 | 1.2187896932493356419 | 0.78076992134206578002 |
| E2 | contestable | 0.99416996327993848043 | 0.6122947695538158456 | 2.0418839426825230478 | 6.2728033691443532844 | 2.8508355049050816456 | 2.0241898240265697611 | 2.2039382823206903753 | 0.75273640429732239413 | (= hours) |
| E3 | wall | 1 | 0.47204913358823543131 | 1.4105971789611369616 | 500/59 | 3.3262711864406779661 | 0.84969825048619005462 | 1.2000138545286472029 | 60/59 | (= hours) |
| E4 | wall | 1 | 0.40186205291748006285 | 1.3618425789451597284 | 500/59 | 3.2460381913181589815 | 0.60279307937622009428 | 1 | 1.1774151427874108505 | 60/59 |
| E5 | all-human | 0 | 0.029413782758211337152 | 1.1619056970251169848 | 10 | 21.2 | 0.85898442923911925401 | 19.468977889270175376 | 1.2 | (= hours) |

E1's entrant works 2.1433213245632650952 hours, and the pool 2.8003509824241698881
efficiency hours. E2's f_line_lo is +∞: low on the line Y is large and the trained's
reserved demand exceeds N_T. E5's γ(0)·π is 0.11470591118226016196. E6 is `LaborShort`
with every value of the sequence +∞ and `reserved` Some(1).

E7: contestable at x* 0.92712986242639353397, v 0.5769229810714520861, P_s
1.9276746171072055141; trained pooled at 0.86538447160717812915; master walled at
1.1182679714505371933, premium 1.0768508180856191135. E8: x* 0.98812228934750603464, v
0.60909191982741945457, P_s 1.9568213969246348103; trained walled at 0.91555117221039191283,
premium 1.0020941474863157432; master walled at 1.3474426859707907976, premium
1.2290087023250540863.

**F, the full economy.**
- F1: contestable with the engine at x* 0.98889368696731094579, v 0.15114482308841387646,
  P_s 1.6036808524929569565, Y 6.4914476902294503478, n_a 1.0873141491115720841, n_pool
  0.36027200780587364517, I 10.410210365780601306, interest 0.056415486787913071726;
  trained walled at 0.4117252812492722115, premium 1.9457463418022066925; reserved costs
  food 0.020586264062463610575, care 0.12763483718727438557.
- F2: wall with the engine, v 0.1165053534428324599, g 0.76791810604669908158 > γ(1) 0.25,
  P_s 1.5215351426172399225, Y 6.8110704414527195923, n_a 1.0579635716708509343; the wall
  switch loom → engine at γ_s 0.43150661211919930493, v_s 0.064963057288499518554 (1c's
  M4t wage); trained premium 2.5252998838906632167.
- F3: a tie at x* 0.82876653029799826234 (1c's M4t switch point) with v
  0.064963057288499518554, σ toward the engine 0.8933594540375401821, P_s
  1.6060227418448017258, Y 6.715477852810912082, n_a 1.3865811223593493908; trained premium
  10.457936453153349916.

**X, switches at the wall.** The wall switch at γ_s 5.7939090413420924643, v_s
6.8875222841006163301.
- X1: a tie at v_s, σ toward the durable type 0.47890358118953065926, P_s
  1.7132513370460369798, Y 8.5692697142403971175, n_a 0.26890987054373998522, interest
  2.8291900696458505285.
- X2: the flow type, v 4.7431304705568019961, P_s 1.5845878282334081198, Y 100/13, n_a 6/13.
- X3: the durable type, v 16.892543391434617742, P_s 1.7850520472154803024, Y
  9.7828213656818626492, n_a 0.0023478771277636470358.

## 8. Tests

The groups are the modules `d1_nesting`, `d2_one_type_corners`, `d3_human_required`,
`d4_worker_types`, `d5_full_economy`, `d6_wall_switches`, `d7_random_workers`,
`d8_regimes_and_validation` and `d9_goldens_file` of `tests/gate/`, cited as `d1::` to
`d9::`. The gate maps onto them:
- constructed wall and interior cases recognised correctly: `d2::*`,
  `d3::the_tail_decides_the_regime`, `d4::*`, `d6::*`, and in every golden test the
  `margin`, each type's `pooled`, and the corner's inequality;
- the wall regime solved: `d2::wall_goldens`, `d2::closed_form_at_the_wall`,
  `d3::baumol_path`, `d5::wall_past_a_switch`, `d6::*`;
- nesting with 1a to 1c: d1.

Identities and goldens at 1e-12 relative (ADDENDUM A7) unless stated; "bitwise" means
`to_bits` equality; an inequality a ≤ b as a ≤ b·(1 + 1e-12), strictly where the instance
has room. A shared identity check (`support_1d::check_identities_1d`) runs §4.7's identities
and bounds at every equilibrium and pins every p_k, p_j, P_s, v and v_i to `at_with` or
`at_wage` at the equilibrium bit for bit.

**d1, nesting (R1; §2.6).**
- `d1::every_1c_golden_instance_is_bit_identical`: 1a's 27 instances, 1b's C3, C3d, C3z,
  C4, gap and near-edge roots, 1c's M3, M3z, M4 path, M4t and M5 path through
  `from_machines`: `Interior`, `Contestable`, every shared output (§6's name map) and
  `bisection_steps` bit for bit; the one worker's wage, hours and supply equal v, n_a and
  n_S; `n_pool` = `n_a`; the new residuals 0. [§5.1]
- `d1::random_economies_are_bit_identical`: 1a's G5, 1b's C5 and 1c's m6 draws: every
  `Interior` result bit for bit (or `MultipleEquilibria` where the wall holds further
  equilibria, which the test counts and the build records), every other result as the next
  test says. [R1]
- `d1::boundary_rows_are_solved`: every row where 1a, 1b or 1c returns `BoundaryNoMargin`
  or `NoInteriorAtZero`: 1d returns a wall, an all-human corner, a root below lo or
  `LaborShort`, and its `f_line_1` or `f_line_lo` equals 1c's diagnostic (`f_at_1`, or
  `f_at_0`, which is f at lo) bit for bit. [§5.3]
- `d1::refusals_nest`: `NotViable` rows with `d_at_1` bit for bit; M5m is
  `MultipleEquilibria` with 3; M5b (unit-1c.md §3.3) with 2, where 1c counts 3: its wall is
  labour-short to the end (f_∞ = 6.23 at H4), so the boundary is not an equilibrium in 1d;
  1c's rejected rows are rejected, N and χ_max named as worker type 0's. [§5.3]
- `d1::points_nest_on_a_grid`: `at(x)` equals 1c's `at(x)` bit for bit on 1b's grid. [§5.1]

**d2, the corners in Appendix B's closure (W).**
- `d2::wall_goldens`: W1; `Wall`, x* 1.0 and 1 − x* 0.0, v > γ(1)·π, `f_line_1` 1a's golden;
  the identities. [SSRN §3.1 p.6; main.tex:147-162; §4.5]
- `d2::closed_form_at_the_wall`: W1, B2, and 60 random one-type economies with f(1) > 0: v
  = ζ·B_s/(1 − ζ·L_s) at 1e-12. [§4.5]
- `d2::labor_short`: W2 and W3, the excesses and `reserved` None. [§2.4, §4.5]
- `d2::all_human_goldens`: W4; `AllHuman`, x* 0.0, v < γ(0)·π, K = X = interest = 0, the
  good's price at its upper bound bit for bit, `f_line_lo` 1a's golden. [§4.5]
- `d2::roots_below_the_bracket`: W5 and W6; `Contestable`, x* against the golden at
  4·2^-52·n_D/|f'| absolute, the rest at 1e-12; the bisection took at most 64 steps. [§5.3,
  §5.5]

**d3, the human-required economy (B).**
- `d3::interior_with_the_tail`: B1's goldens; final hours = Y·(H_ŷ(x*) + L^H_ŷ) and
  `required_hours` = Y·L^H_ŷ bit for bit. [SSRN p.26; main.tex:173-175]
- `d3::the_tail_decides_the_regime`: B2, contestable without the tail and at the wall with
  it, both funded. [SSRN §3.1]
- `d3::baumol_path`: BP's table; at every point `Wall`, p_m = (λv + b)/(1 − a) at 1e-12;
  p_services/v, φ_w,services and the CES share strictly monotone toward 0.25, 1 and 1;
  p_goods strictly falling; at η = 1e-6 p_services/v within 1e-6 of 0.25, φ_w within 1e-5
  of 1, v within 1e-6 of BLIM's v_∞; v/(γ(1)·π) growing. [check_pinning.py:147-178;
  main.tex:912-918; SSRN Prop E.1 p.35; check_kset.py:39-47]

**d4, worker types (E).**
- `d4::both_pooled`: E1; v_T = 1.5·v, premium 1.0 exactly; hours and pool hours; the pool
  clears. [§2.2]
- `d4::trained_at_its_wall`: E2; premium > 1; trained hours = reserved hours; its market
  clears to a few ulps (`residuals.reserved`); its wage equals ζ_T·ν_T·P_s. [SSRN §5 p.13;
  main.tex:584]
- `d4::the_wall_with_types`: E3 and E4. [§4.5]
- `d4::all_human_with_a_walled_type`: E5. [§4.5]
- `d4::reserved_shortage`: E6 is `LaborShort` with `reserved` Some(1). [§4.3]
- `d4::walk_order` and `d4::walk_cascade`: E7 and E8; the unit tests of the walk also run
  both on constructed inputs. [§4.3]
- `d4::supply_and_exit_per_type`: at every E equilibrium, each type's supply is
  N_i·F_i(ln(1 + v_i/(ν_i·P_s))), and (e^{χ*_i} − 1)·ν_i·P_s = v_i at its marginal work cost.
  [SSRN eq 8-10 p.11; A.1 p.25]
- `d4::splitting_a_type_changes_nothing`: E1 with the entrant split into two types of half
  its workers each: the same equilibrium within 1e-12, and the two halves' hours equal.
  [§2.2]

**d5, the full economy (F).**
- `d5::full_goldens`: F1, per type and per category; care's reserved cost includes food's
  through the chain. [§4.2]
- `d5::cost_system_with_reserved_costs`: at F1 to F3, p = Ap + λv + b + R over every row,
  f = (I − A^qᵀ)y, n_D = λ^qᵀy, T = b^qᵀy, pᵀf = v·n_D + Σ v_i·D_i + T + interest. [SSRN
  eq 3, A.1, App. C]
- `d5::wall_past_a_switch`: F2; the line's technique is the loom and the wall's the engine;
  the wall switch's γ_s and v_s against the closed form. [§5.2]
- `d5::tie_with_a_walled_type`: F3; σ's golden; 1c's closed form evaluated at the same
  switch differs by more than 1e-4; f(σ) monotone on a grid. [§4.6]

**d6, switches at the wall (X).**
- `d6::tie_at_a_wall_switch`: X1; the two delivered costs equal at v_s; σ's golden. [§4.6]
- `d6::roots_either_side`: X2 and X3; the technique at the wall is the cheapest at the
  wage. [§5.2]

**d7, random economies.** SplitMix64 seeded 931 to 935 (1c's were 926-929). 1c's m6 table
for the scalars, segments, categories, intermediate inputs and machine types, and:

| draw | range |
|---|---|
| I | an integer in 1..3 |
| type 0 | N ~ U(1, 10), χ_max ~ U(0.3, 3), ε = ν = 1 |
| types 1.. | N ~ U(0.5, 5), χ_max ~ U(0.3, 3), ε 0 with probability 0.15 else U(0.5, 2.5), ν ~ U(0.5, 2) |
| L^H_j | 0 with probability 0.6, else U(0.05, 0.4); none for a site |
| R_ji | type 0: 0 with probability 0.8; others: 0 with probability 0.5; else U(0.02, 0.3); none for a site; a type with ε = 0 gets one |
| T | U(2, 12) |

Sets of 60 equilibria each: (a) ρ = 0, one machine type; (b) ρ = 0, K in 1..3;
(c) ρ ~ U(0, 0.1), K in 1..3; (d) abundant labour, type 0 with N ~ U(20, 60) and
χ_max ~ U(0.02, 0.2), ρ = 0, for the all-human corner; (e) the wall-switch set, M5's two
types with the durable type's build land drawn so that the crossing lies at g ∈ (γ(1),
3γ(1)) and N set, half the time, so that the pool's supply at the switch lies between the
two one-sided demands. A 32-digit scratch run of 60 draws in each of (a)-(d), without
intermediate inputs, gave: (a) 22 contestable (10 with a walled type), 18 wall (7), 13
`LaborShort`, 7 invalid; (b) 18 contestable, 19 wall, 1 all-human, 11 `LaborShort`, 11
invalid; (c) 19 contestable, 21 wall (one a tie), 1 all-human, 9 `LaborShort`, 10 invalid;
(d) 26 all-human, 22 contestable, 4 `LaborShort`, 8 invalid; every identity held, and no
`MultipleEquilibria`. The build records its own tallies (`MAX_DRAWS_1D`). For each:
- `d7::identities`: §4.7's identities and bounds. [§4.7]
- `d7::residuals_recompute`: each residual equals its recomputation bit for bit, and each
  is nonzero somewhere. [§6]
- `d7::path_and_count`: f nonincreasing on a grid of every stretch within each technique;
  the count equals the result; at ρ = 0 every switch downward and no `MultipleEquilibria`.
  [§5.4]
- `d7::type_status`: every walled type's wage is above ε_i·v and its supply clears its
  reserved demand; every pooled type's supply covers its reserved demand. [§4.3]
- `d7::the_draws_cover_the_regimes`: every margin, walled and pooled types, a type with
  ε = 0, ties on the line and the wall, both causes of `LaborShort`, and
  `MultipleEquilibria` at ρ > 0 (in set (e), or recorded as absent).

**d8, regimes, validation and reductions.**
- `d8::exact_zeros_at_the_junctions`: with χ_max small so that supply saturates at N, N set
  in f64 so that f_line(1) = 0 (the wall at v(1)), f_line(0) = 0 (the all-human corner at
  v(0)) and f_line(lo) = 0 (a root at lo, `Root::exact`). [§5.3 step 3]
- `d8::not_viable_stands`: 1c's `NotViable` rows (M4 at η 50 among them) with F's worker
  types and human-required hours added are still `NotViable` with the same `d_at_1`: the
  new inputs do not enter the machine block. [§5.3 step 1; decision 64]
- `d8::validation`: every rule of §3.2, and −0.0 stored as +0.0.
- `d8::permutation_of_types`: reversing the worker types gives the same outputs within
  1e-12, permuted.
- `d8::efficiency_units`: measuring efficiency in units of c (every ε_i, μ, L^H and machine
  labour coefficient times c, and η divided by c, so that M_j = μ·J is unchanged): v/c and
  the same v_i, x*, prices and quantities; bit for bit at c = 4 (expected; the build records
  it if not) and within 1e-12 at c = 3.
- `d8::walk_ceiling`: a point whose walled types' shares reach 1 is short, and an economy
  short everywhere is `LaborShort` with excess +∞. [§4.3]

**d9, the goldens file.** `goldens_1d.txt`'s five digests recompute, and the Rust constants
match it to 20 digits.

**Unit tests**: the walk (sorted order, a cascade, the ceiling, no candidates is P⁰ bit for
bit); bisection on bit patterns (adjacent doubles in at most 64 steps from [0, lo], from
[1, +∞] and between subnormals; an exact zero; the end with the smaller |f|); the tie's
closed form against bisection when no type is walled.

## 9. Out of scope, and where each goes

- **Distinct schedule shapes per worker type**, comparative advantage along the line: open
  question 1, a 1d addendum before Phase 2's multi-type instances if ruled in.
- **Machine recipes using a type's reserved labour** (the millwright and the engineer of
  main.tex:584): open question 6.
- **Training** (entry into the trained type at a cost, "open entry disciplines the
  long-run premium", SSRN p.13): the types' numbers are inputs; PLAN §3.5 moves them on the
  tape. Phase 3 and later.
- **A basket per worker type**, price-responsive baskets, owners' demand for human-required
  services (PLAN §3.2, "owners' demand is what priced the wall"), and Prop E.1(iii)'s
  three-way split under CES: **1f**.
- **Idle land at zero rent** (which gives `LaborShort` economies an equilibrium) and the
  default exit form s(q): **1e**.
- **The fraud bound and the superstar lemma** (main.tex:922-938; check_pinning D.2-D.3;
  check_kset P9-iv, P9-v): they concern a taste premium for verified human work and the
  distribution of H-income within a type, neither of which this closure has (a continuum of
  identical workers per type, no provenance premium). They are symbolic checks in
  laborformal and stay there.
- **Coverage with an H-service in the bundle** (D×C, κ): 1f's, with support funding.
- **A dump interface for worker types**: there is no other solver to compare against.
- **Cells in the equilibrium** (decision 61): unchanged.

## 10. Pitfalls

- **Three walls.** The wall (the pool holds no contestable task), a type at its wall (its
  reserved tasks take all its hours), and the all-human corner are different (§0.2). A gap is
  none of them.
- **Two human-required sets.** The common one (L^H, the paper's H) is pool work in
  efficiency hours; the reserved one (R) is one type's, in its hours.
- **v is per efficiency hour.** A pooled type earns ε_i·v; `real_wage` is the pool's, and
  each type has its own.
- **`n_a` is hours, `n_pool` efficiency hours.** They agree for one type of efficiency 1.
- **1d never returns `BoundaryNoMargin` or `NoInteriorAtZero`.** Their diagnostics live on as
  `f_line_1` and `f_line_lo`. 1a's `f_at_0` is f at lo, not at 0; 1d's `f_line_0` is f at
  x = 0.
- **`LaborShort` is not `NotViable`.** The prices exist; no wage clears labour with land
  fully rented.
- **`funded` is a flag, scale-free in (N, T).** Many of the constructed economies are not
  funded (E, F, W1), as 1a's G8 N = 8 row is not; the equilibrium conditions do not use it.
  B2 and X are funded.
- **The machine price at the wall is not br/(1 − a − λγ).** That is the margin's closure;
  at the wall p_m = v·λ̃_m + b̃_m at the wall's own wage.
- **Notation.** ν_i is support (not 1c's tie share σ or the CES σ); ε_i efficiency; ζ_i a
  real wage per support basket; R̃ the chain's reserved hours, not a rent; "E", "F", "W",
  "X" instances are unrelated to 1a's E case or to Figure 4's eras.

## 11. Open questions

As built, 1d takes the first choice on each, and STATE.md records each as a decision open to
veto, numbered from 135. The ones marked **Phase 2** bind it.

1. **Worker types as one shape with an efficiency each, plus reserved tasks** (§2.2;
   proposed decision 135). **Phase 2**: the agents' task cells carry productivity by worker
   type (PLAN §3.1); the oracle can price and solve a world whose cells scale one common
   human productivity by a type's efficiency, or reserve a cell to one type, but not one
   where the ranking of types changes across cells. The alternative, a schedule per type,
   needs a threshold per type and an I-dimensional solve whose uniqueness is not proved.
   Accept, or rule the alternative into a 1d addendum?
2. **The solved corners inside `Interior`** with `margin` (§2.5; proposed 136), as decisions
   62 and 69. Alternative: new variants `Wall` and `AllHuman`, which change 1a's `Regime`
   and every match on it.
3. **No equilibrium with land fully rented is `SolveError::LaborShort`** (§2.4; proposed
   137), as decision 70 refuses: only where the excess demand changes side nowhere on the
   path; the edge of a reserved shortage is an equilibrium and is solved (§12 item 16).
   **Phase 2**: its wall-regime instance must be a solved wall, not a short one. **1e**
   resolves these economies with idle land.
4. **Living costs as support baskets on one basket** (§2.3; proposed 138). Alternative: a
   basket per type, 1f's.
5. **Type hours**: the pool's demand split among pooled types by their net supplies (§4.7;
   proposed 139). **Phase 2**: pooled types are perfect substitutes, so which of them works
   which pool task is indeterminate in the model; Phase 2 should compare per-type hours only
   through each type's supply at the oracle's wages, or in total.
6. **Machine recipes use pool labour only** (proposed 140). **Phase 2**: the desks' recipes
   buy labour by type (PLAN §3.1), and the sources name trained machine builders
   (main.tex:584). With a reserved type in machine recipes, p_m depends on a walled wage and
   the walk and the (O, V) system couple. Rule it in as a 1d addendum?
7. **The [0, lo] stretch and the corners by bisection on bit patterns** (§5.3; proposed 141).
   Alternative: keep 1a's `NoInteriorAtZero` for roots below lo.
8. **Training is exogenous** (proposed 142): N_i are inputs; the premium of a type at its
   wall is a scarcity price, not a cost of training recovered (SSRN p.18's reading of the
   industrial era, main.tex:584; p.17's pre-industrial craftsman has the other).
9. **Viability at the top of the line stands** (decision 64), although the wall stretch
   prices machines at any wage (proposed 143).
10. **A tie with a walled type is solved by bisection on σ** (§4.6; proposed 144), with 1c's
    closed form where no type is walled; with interest its uniqueness within the switch is
    measured, not proved.
11. **The count covers the whole path, and `MultipleEquilibria::switches` lists the line's
    switch points only** (proposed 145). Alternative: add the wall's switch wages to the
    error, which changes its fields.
12. **The random draws' ranges** (§8, d7; proposed 146): the build tunes and records them.

## 12. Changes during the build (P1.8, 2026-09-27)

1. **The generator builds on 1c's.** `generate_1d.py`'s `Economy` extends `generate_1c.py`'s:
   the machine block, the tasks on the line, the quantities and the Leontief solves are 1c's
   code, and 1d adds the worker types, the tail, the reserved hours, the walk, the corners, the
   path and the report. §7 had it import 1c's generator only to assert the nesting.
   `goldens_1d.txt` records all five digests, and `d9` checks them. `generate_1c.py`'s own
   solve stops at 1a's regime tests (it predates P1.6's count), so for M5b it says
   `BoundaryNoMargin`; the generator counts 1c's three changes of side from the line's part of
   1d's sequence instead.
2. **A change of side at a reserved shortage** (added; corrected by the verification, item
   16). Where a type's reserved demand crosses its workers along the line, f is +∞ on one side
   and can be far below 0 on the other. §5.3 as drafted bisects onto that edge, and the labour
   net then refuses the result as `LaborNotCleared`: two draws of d7's set (d) did this, f
   falling from +∞ to −33 and to −47 across one double. The build returned
   `SolveError::LaborShort` there, holding that no point clears both the pool and the reserved
   market. That was wrong: at the edge the short type's supply is vertical, and a wage above
   its own ζ clears the pool (item 16). `Root` records a bracket that closes on +∞ (`jump`,
   always false in 1a-1c), and the solve reads it after the root in [0, lo], after a root on
   the line and after a tie's σ, whose Y moves every D_i.
3. **The corners are not short in part.** At a corner D_i and ζ_i are fixed, so a reserved
   shortage makes the whole corner short. The walk's ceiling cannot bind at an ω below one
   where it does not: pooling a type j that takes the walled share C below 1 needs
   ε_j·ω/ν_j ≥ ζ_j, while the ceiling leaves ω < (1 − C')/L' ≤ ζ_j·ν_j·R_ŷj/(L_s + ε_j·R_ŷj)
   < ζ_j·ν_j/ε_j. So f(ω) = n_D − S(ω) is finite on every corner bracket the solve bisects, as
   §4.5 has it, and the corners need no jump check. A draft of the build carried one; it was
   dead code and was removed.
4. **The saturated knife edge.** With supply saturated at N = n_D(1) (1c's "f(1) = 0 exactly"
   row, from G8), f is 0 on the whole wall and at its end. On the row's own doubles the exact
   n_D(1) is 0.31914893617021278857, 4.9e-17 below N, so the exact economy has its root just
   below x = 1: the row is not decidable in f64 (§5.5). The build counted f_∞ = 0 on the
   positive side and returned `LaborShort`, the one label wrong under every reading, although
   its own f_line(1) was 0, the junction by §5.3's exact-zero rule. The junction rule now takes
   precedence (item 17): the row is the wall at x* = 1 and v = v(1) in d1 and in
   `d8::exact_zeros_at_the_junctions`, which builds the exact zeros at x = 1 and x = 0 without
   saturation too: G1 with χ_max 1, and N chosen among the doubles next to n_D(x)/c, where
   c = F(ln(1 + v/P_s)) does not depend on N, so that N·c = n_D(x) exactly. The saturated
   construction stays for lo, where n_D(0) > n_D(lo) makes the root determinate.
5. **Type hours.** pool_i = n_pool·(s_i/S), not s_i + (n_D − S)·(s_i/S): the same in exact
   arithmetic, and for one type n_pool·1.0 = n_pool exactly, without Sterbenz's condition.
   The wage bill is the demand side, v·n_pool + Σ_i v_i·D_i; Σ_i v_i·hours_i + T + interest = I
   is checked.
6. **Result types** (§6). The line's values are `f_line_0`, `f_line_lo` and `f_line_1:
   Option<f64>`, `None` where the point is short (+∞), so that every output the finiteness
   check reads is a number. `f_end` is a number at every equilibrium, since a nonnegative f_∞
   after an equilibrium would be a second change of side. `margin` prints as a count, 0, 1 or
   2 (`Margin::code`). Beside §6's API there are `WorkerEconomy::wall_end()` (`WallEnd`: τ_e,
   ω_∞, f_∞ and the short type), `wall_technique_at(v)`, `wall_switches()`, `machines()` (1c's
   economy underneath, whose N and F are unused placeholders), `human_required_chain()`,
   `reserved_chain(i)` and `excess_at_share(q, above, σ)` (a tie's f(σ), which d5 reads), and
   `WorkerPoint::short` with `Shortage::{Reserved(i), Ceiling}`.
7. **1c's code changed its API, not its behaviour.** `MachineEconomy::new` is now `assemble`
   (1c's checks, less N and χ_max when 1d asks) followed by 1c's last three checks, in 1c's
   order. `Cleared`, `clear`, `worse` and `in_category` are crate-visible, with accessors for
   the chain's factors, J at the edges and the direct hours. `Item` gains `Worker` and
   `WallSwitch`, `SolveError` gains `LaborShort` and `NonFiniteInWorker`, `Root` gains `jump`,
   `solve.rs` gains `bisect_bits`, and `MachineBlock` gains the crate-private `wage_system` and
   `prices_at_wage`. In the tests, `m5_interest::hidden` (M5b) and `m6_random_leontief::
   all_sets` are crate-visible for d1. 1a-1c's 242 tests pass unchanged.
8. **Validation names.** A worker type's parameters are `ParamError::Item { kind: "worker
   type", .. }` named `workers`, `chi_max`, `efficiency` and `support`; a type with no work is
   its item with `Invalid { name: "reserved" }`; no types, and no type with ε > 0, are
   `Invalid { name: "worker types" }`; `human_required` and `reserved` of the wrong shape are
   `Invalid` by those names, and a bad entry is `Item { kind: "category" }`. The order is 1c's
   (T, the schedule, ρ, the machine types, the line, the categories, the intermediate inputs,
   the basket's land), then the worker types, the two new inputs, work per type, each category
   priced, and the basket's pool work.
9. **BP's growing golden** is v/(γ(1)·π) (`BP_*_V_OVER_REPLACEMENT`, §4.8's g/γ(1)), not g =
   v/π, which converges (to v_∞/0.6 = 0.673).
10. **F3's closed form.** "1c's closed form" for σ (§4.6 and §3.3's table) is unit 1c's
    expression, with the supply of the type below's evaluation on both sides: 0.893744 against
    0.893359, 4.3e-4 relative. The goldens record it (`F3_SHARE_CLOSED_FORM`).
11. **Tests.** d1 also runs 1c's regime rows (M4 at η 0.25 and at N 200, against D0 goldens
    and 1c's), all of 1c's m6 draws beside 1a's G5 and 1b's C5 draws, and the two overflow
    rows, which fail with 1c's errors. d2's 60 random walls come from a W-like distribution
    (seed 930: λ ~ U(0.2, 0.9), N ~ U(0.5, 6), η ~ U(0.2, 1.5), the rest G5's). d7's switch
    direction at ρ = 0 is strict only where the basket has machine tasks (M_ŷ > 0); without
    them both techniques' quantities are the same. W5's and W6's line values are compared at
    8 ulps of n_D absolute: f is n_D − n_S with n_D ≈ 10, which leaves 9.8e-16 absolute (1.6e-4
    relative) on W5's f(lo).
12. **The random draws** (§8, d7) take 1c's m6 table with T ~ U(2, 12), 1-3 segments, 2-4
    categories and a site, k ~ U(0.5, 4) and build lags 1-4, and §8's worker table. Set (e)
    draws ρ ~ U(0, 0.15) and puts the crossing on the line (x ~ U(0.15, 0.85)) on even draws
    and on the wall (g ~ U(1, 3)·γ(1)) on odd ones; half the time type 0's N is set so that
    the pool's supply at the switch is midway between the two demands. `MAX_DRAWS_1D` is 3000.
    The tallies:

    | set | draws | invalid | contestable | wall | all-human | with a walled type | ties | short, pool | short, reserved | not viable | multiple |
    |---|---|---|---|---|---|---|---|---|---|---|---|
    | (a) ρ = 0, K = 1 | 87 | 10 | 33 | 26 | 1 | 16 | 0 | 11 | 3 | 3 | 0 |
    | (b) ρ = 0 | 107 | 16 | 18 | 41 | 1 | 22 | 2 | 24 | 5 | 2 | 0 |
    | (c) ρ > 0 | 94 | 12 | 25 | 34 | 1 | 27 | 2 | 14 | 6 | 2 | 0 |
    | (d) abundant | 88 | 18 | 27 | 1 | 32 | 39 | 0 | 3 | 5 | 2 | 0 |
    | (e) switches | 101 | 18 | 36 | 24 | 0 | 16 | 31 | 7 | 14 | 0 | 2 |

    Both `MultipleEquilibria` are at ρ > 0 in set (e). Set (d)'s equilibria include two at the
    edge of a reserved shortage (item 16); P1.8 counted them as reserved shortages and needed 90
    draws. d1's random nesting, over 1116 draws: 600 give the
    same equilibrium bit for bit (1a's 180, 1b's 180 and 1c's 240), 366 of 1c's boundary
    regimes are solved at the wall and 2 at the all-human corner, 131 are `LaborShort`, and 12
    are `NotViable` and 5 `MultipleEquilibria` in both; none has further equilibria on the wall.
13. **Numbers.** `generate_1d.py` reproduces every value of §7 to the digits shown and writes
    240 goldens in about 14 s; the one-type form nests `generate_1c.py`'s solves within
    8.7e-72. The oracle's f64 values match the 233 goldens compared by `close` within 2.0e-15
    relative (F3's σ; then W1's wage and the wall switch's wage, 1.4e-15). W5's x* is within
    1.1e-16 absolute of its golden (2.3e-4 relative, inside the bound 4·2^-52·n_D/|f'| ≈
    8e-16), W6's within 3e-16 relative, and W5's f_line_lo within 9.8e-16 absolute. Unit 1d
    adds 9 unit tests (7 in `workers.rs`, 2 in `solve.rs`) and 43 gate tests: 294 in the
    package (66 unit, 227 gate, 1 doc).
14. **Mutation.** 61 mutants of `workers.rs`, the wage-given system in `machine_block.rs`,
    `bisect_bits` and `Root::jump` in `solve.rs`, and the goldens, each run against the whole
    package: the tail left out of the line's, a corner's or the report's hours; the reserved
    shortage, ζ and the walk (index order, no recompute, no ceiling); the walled wage, supply
    without support, the pool's net supply; reserved costs without the chain; the corners'
    supply, wage and end; the sides of f(1) and f_∞; the junctions; the jump checks; the tie's
    closed form with walled types or with one side's supply; the report's state, hours, wage
    bill, support, participation, φ's g, `margin_active`, the new residuals; the wall piece's
    technique; every new validation rule; `at_wall` and `premium`; the envelope stopped at
    γ(1); the wage-given system's u and labour; the bit bisection's stopping rule and −0.0; and
    the three goldens files. The first run left two survivors of 60: the jump check after a
    root in [0, lo], now killed by the second half of `a_jump_to_a_shortage_is_labor_short`
    (since P1.9 `d8::the_edge_of_a_reserved_shortage`, item 16); and a wall switch's wage
    taken as the closure wage of the type above rather than below, equivalent in exact
    arithmetic (the two are equal at the crossing) and within rounding of it in f64. A 61st,
    the jump check of a tie's σ, is killed by `a_tie_across_a_shortage` (since P1.9
    `d8::the_edge_in_a_ties_share`).
15. **Decisions.** The build takes §11's first choice on each question, as proposed decisions
    135-146 for STATE.md at the unit's close, open to veto; 135 (one shape and an efficiency per
    type), 137 (Phase 2's wall instance a solved wall), 139 (type hours split by net supply) and
    140 (machine recipes on pool labour) bind Phase 2. The knife edge of item 4 extends 137: an
    instance for Phase 2 should not be one. The edge of a reserved shortage (item 16) is solved,
    not refused, and also binds Phase 2.
16. **The edge of a reserved shortage is an equilibrium** (the verification's blocker, P1.9,
    2026-09-27). An independent derivation, with the pool's wage v as its unknown and the
    walled set enumerated, found equilibria where P1.8 returned `LaborShort`. Where D_i = N_i,
    type i's supply is vertical at N_i for every real wage per support basket κ_i ≥
    expm1(χ_max,i) (uniform F), so its reserved market clears at any such wage. Raising κ_i
    raises P_s through the walk and lowers the pool's real wage, so f rises with κ_i,
    continuously, from its value at the edge's finite side (< 0) toward n_D > 0 at the walk's
    ceiling, and a κ_i clears the pool: every market clears, and the type's wage is "a scarcity
    price … set by demand" (main.tex:584; SSRN §5). The case is not a knife edge: of the
    derivation's 60 targeted draws (seed 77, N_i between D_i(0) and D_i(1)) 7 were such edges,
    and d7's set (d) has 2. Now, where a bisection in x (the root in [0, lo] or on the line) or
    in a tie's σ closes on a short point, the equilibrium is the bracket's finite end, where
    D_i is within rounding of N_i, with the short type walled at κ_i·ν_i·P_s: κ_i by bisection
    on bit patterns over [ζ_i, +∞], ζ_i the type's own at the finite end (where f < 0) and +∞
    short (the walk has no fixed point). `bisection_steps` adds κ_i's steps to x's on the line
    (0 at a tie, as before). `WorkerEq::edge` flags the type, whose `clearing_real_wage` is
    κ_i, and `WorkerEconomy::at_edge(x, τ, Edge { worker, clearing })` is the evaluation that
    prices it; its reserved residual is of rounding order. A change of side cannot close on the
    walk's ceiling: approaching it from the finite side, 1 − C → 0+, P_s → ∞, the pool's real
    wage → 0 and f → n_D > 0 (P1.8's documentation of `LaborShort` said it could), and the
    solve treats such a close as a non-finite value. `LaborShort` is now an economy whose
    excess demand changes side nowhere on the path, and `check_count_1d` requires no change of
    side for it. `generate_1d.py` finds the edge by the same bracket in x or σ at 70 digits,
    then κ_i, and asserts every identity there. The goldens:
    - J1, E's economy with N_E 40, χ_E 0.05 and N_T 1: D_T reaches N_T at x* = 0.5 exactly with
      decimal inputs (the derivation, on the doubles: 0.50000000000000006774; the oracle gives
      the next double above 0.5); v 0.35820895522388059701, P_s 29.723913718591018298, the
      trained walled at κ 6.5678443117457383283 (above expm1(0.8) = 1.2255), v_T
      234.26644516736296343, all its N_T hours reserved;
    - J1b, N_T the double 1.1999999999998798 midway between D_T(0) and D_T(10^-12): x*
      5.0071058410549431977e-13, v_T 27.900563586212314773;
    - J2, F at η 0.5 and N_E 30 with N_T the double 0.7542146710911733 midway between the
      trained's demand at the line's switch under the loom and under the engine: a tie at σ
      0.49824924569934011727, v_T 4.0382429662537090722. With N_E 16 the same N_T is a tie
      past the edge, the trained at its own ζ.
    The derivation's values agree (J1 within 1.5e-15 of the oracle; J2's v_T within 8.7e-14),
    and its targeted set now agrees on all 60 draws, within 1.2e-13. **Phase 2** (decision
    137): at an edge a type's hours are its workers and its wage is set through the pool's
    clearing and the basket's price, not by its own supply; an agent market needs some wage
    mechanism for a vertical supply to reach it, or should avoid the edge.
17. **f_∞ = 0 after an exact zero** (the verification's minor on item 4). f_∞ = 0 counted on
    the positive side made the saturated knife edge `LaborShort`. Now f_∞ = 0 is on the
    negative side when the wall's last piece starts at an exact zero (f_line(1), or the last
    wall switch's value above): f is then 0 on the whole piece (Lemma 3'), and its start is the
    equilibrium by §5.3's exact-zero rule. `sequence_1d` in the tests and the generator's count
    take the same rule.
18. **The rest of the verification.** The mutation pass's survivors are now killed:
    - f_∞'s walk with the worker efficiencies: `check_path_1d` asserts that f_∞ is the limit of
      f on the wall's last piece (its value at 10^10 times the piece's start, within 1e-6 of
      max(n_D, 1)), so d7 runs it on every draw; and E9, E4 with χ_max 3 for both types, the
      trained walled at the equilibrium and pooled at the end of the wall, pins f_∞
      −1.2628620845098934739 and ω_∞ 2.2099447513812154696
      (`d4::the_end_of_the_wall_counts_efficiency`; a walk with every ε_i = 1 gives −1.689);
    - φ_w and the technique's closure wage at a corner: `check_identities_1d` asserts
      φ_w = v·λ̃_τ/p_τ and the closure wage of τ equal to v on every stretch; d, the wage-given
      system's least pivot, is pinned at W1 to 1 − a (`d2::wall_goldens`);
    - lemma B.1 with ν ≠ Σ N_i: `check_identities_1d` recomputes it from `at_with(1, τ_m)`, and
      `d4::lemma_b1_counts_the_support` separates ν from Σ N_i (E1 with both supports 0.4);
    - `LaborShort`'s `reserved` with two types short at the end: `d4::reserved_shortage`
      (E7's economy with N_T = N_M = 0.01 names the trained);
    - `human_required` longer than C: `d8::validation`;
    - the rules at exact equality, in `d8::rules_at_exact_equality` and the walk's unit test
      `walk_keeps_a_type_at_its_threshold_pooled`: D_i = N_i is not short (E3 with N_T the
      double D_T(1)), c_i·P = e_i stays pooled, the all-human corner's cheapest type with two
      identical types is the lower index, and a wall switch's value below that is exactly 0 is
      the piece's end, v = v_s without a bisection step (X with N among the doubles next to
      n_D/c).
    The fix round ran 20 mutants, the survivors above and nine of the fix (the edge ignored on
    the line or in a tie, κ_i left at ζ_i, the short type read at the root, the edge not
    reported, its flag, its steps, the override, and f_∞ = 0 always positive): all are killed.
    The minors also corrected §3.3's B1 row (0.8442 is the economy without the tail; a unit
    that leaves the tail out of labour demand only gets x* 0.8549 at B1 and 0.9392, contestable,
    at B2), the eras' pages (SSRN pp.17-18) and decision 142's (p.18: the industrial era's
    "scarcity price on trained hands and heads"; p.17's pre-industrial premium "is the cost of
    those years, recovered"), and §5.5 at a wall switch's tie, where the derivation measured,
    for nearly parallel delivered costs (slopes 0.18784 and 0.18710, an amplification of 234),
    v off by 4.4e-14 relative and σ by 8.8e-12 (dlog σ/dlog v = −201), and at a line tie with
    a walled type σ off by 3.1e-12 and interest by 1.4e-12; the gate's 1e-12 holds on every
    golden. `goldens_1d.txt` now has 269 goldens (J1, J1b, J2, E9): the oracle is within
    8.0e-14 relative of the new ones (J2's v_T; its σ 3.6e-14; J1, J1b and E9 within 3.1e-15), and
    J1b's x* within 5.6e-16 absolute. The package has 298 tests (67 unit, 230 gate, 1 doc).
