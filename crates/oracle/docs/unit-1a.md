# Unit 1a: the one-category oracle

Dated 2026-09-25. This is the unit's specification, adapted from the working spec
checked that day by two independent 50-digit solves. §9 lists what changed, including
what changed when the package joined the workspace (P1.1, 2026-09-26).

- **Pinned to:** laborformal `31b3482`. Every laborformal path and line below is at that
  commit.
- **Paper:** SSRN 7226858 (posted 2026-09-23), at `pinning/paper/ssrn-7226858.pdf`.
  Cited below as "SSRN p.N (eq k)".
- **Reference implementation:** `paths/code/macro.py`: `Economy` at :48-68, `_parts` at
  :71-95, `solve` at :98-114.
- **Reference gate:** `paths/checks/check_macro.py`: P1 at :29-34, the random identities
  I1/I2 at :45-60, and A1-A3 at :108-133.
- **Gate source (ADDENDUM A7, accepted):** the SSRN Appendix B, whose only executable
  form at the pin is `paths/code/macro.py` with check_macro P1. Never main.tex's
  Appendix B.
- **Code:** this package. The goldens come from `goldens/generate.py`.

## 0. Use the SSRN Appendix B, not main.tex's

- `pinning/paper/main.tex` is the older 2026-09-21 revision. Its Appendix B
  (main.tex:724-786) is a different economy:
  - each household owns T/N of the land;
  - participation is cut at log(1+v/A), where A = (T-Nh)/N;
  - all residual land goes to machine production.
- That economy solves to x* 0.96791, v 0.59841, Y 18.47540, N_a 1.34285.
- **Do not implement it.** The PLAN's numbers belong to the SSRN Appendix B (SSRN
  pp.28-30), which is specified below.
- main.tex is still the source for two things the SSRN dropped:
  - the replacement-closure worked instance (main.tex:325-336);
  - the λ = 0 durability forms (main.tex:690).
- Notation map between sources:
  - main.tex `c` = SSRN `p_m` = macro.py `pm`;
  - SSRN `X` = macro.py `X` = `K`.
- Exit is priced as dependence, s = (e^χ − 1)·P_s (SSRN eq 9). The ADDENDUM's A8 ruling
  keeps this form for unit 1a, because the published numbers use it. Unit 1e adds s(q).

## 1. Scope

**In scope:**
- one final good (the category);
- one machine type producing machine services;
- one non-produced input: land services, with rent r and endowment T per period;
- space rented directly by households;
- N potential workers:
  - each supplies at most one hour per period;
  - they differ only in a work cost χ;
- one non-working **provider** household:
  - it owns the land (and receives interest when ρ > 0);
  - it gives every worker one basket whether they work or not;
  - it spends the rest of its income on baskets;
- durability and interest, entering through one scalar user-cost factor u.

**Out of scope, and where each goes:**
- no government (1f);
- no human-required task set: H = ∅ here (1d);
- one worker type (1d);
- one land quality (1e);
- many categories (1b);
- matrices and separate operating and build recipes (1c).

## 2. Parameters and the Appendix B instance

| symbol | name in code | meaning | instance |
|---|---|---|---|
| N | `workers` | potential workers, one hour each per period | 4 |
| T | `land` | land-service endowment per period | 10 |
| h | `space` | space per basket; a basket = 1 good + h space, z = (1, 0, h) | 1 |
| a | `a` | machine services used per machine service (flow case) or per machine built (durable case) | 0.3 |
| λ | `lam` | hours per machine service, or per machine built | 0.05 |
| b | `b` | land services per machine service, or per machine built | 0.4 |
| γ(x) | `schedule` | relative human productivity γ_L/γ_M, with γ_L = 1 and γ_M = 1/γ | 0.2 + 0.8x |
| F | `work_cost` | cdf of χ, uniform on [0, χ_max] | χ_max = 1 |
| ρ | `rho` | required return (interest) per period | 0 |
| δ | `delta` | depreciation per period: geometric, applied after installation | 1 |
| J_b | `build_lag` | periods from start of build to first service (integer, ≥ 1) | 1 |

**Schedule family** (macro.py:56-68):
- γ(x) = η(g0 + g1·x^k);
- J(x) = ∫₀ˣ γ = η(g0·x + g1·x^(k+1)/(k+1));
- the paper's instance is η = 1, g0 = 0.2, g1 = 0.8, k = 1;
- paper requirements (SSRN p.28): γ > 0 and γ' > 0 on [0,1]; 0 ≤ a < 1; λ ≥ 0; b > 0;
  h > 0.

**Validation.** The code also requires N, T, χ_max > 0, ρ ≥ 0, 0 < δ ≤ 1 and J_b ≥ 1,
all finite. For the power schedule it requires η, g0, g1, k > 0, which makes γ positive,
continuous and strictly increasing, and k ≤ `CURVATURE_CEIL` = 1024 (below). Viability is
not a parameter check: an economy with D(1) ≤ 0 is valid and solves to `NotViable`. A
custom `Schedule` without a closed form is checked by sampling: γ(0) in range, γ finite
and strictly increasing at x = i/`VALIDATION_SAMPLES` (16), J(0) = 0 and
γ(0) < J(1) < γ(1). Sampling cannot see a jump between samples; the `Schedule` contract
asks for a continuous γ (the paper's γ' > 0), and the solve's labour-residual net (§4 step
5) refuses the sign change a jump can leave without a root.

**Scale bounds.** Every scale parameter (N, T, h, b, χ_max, and η, g0, g1) must lie in
[`SCALE_FLOOR`, `SCALE_CEIL`] = [1e-30, 1e30]; δ in [1e-30, 1]; λ and ρ in [0, 1e30]; k in
[1e-30, `CURVATURE_CEIL`] = [1e-30, 1024].
Outside these, products and quotients the solver compares can underflow, and an
underflowed zero can decide the regime:
- at δ = 5e-324, n_D(1) = λδK and n_S(1) both round to 0, so f(1) = 0 reads as
  `BoundaryNoMargin` although the exact f(1) is negative;
- at N = h = 1e300 with T = b = 1e-30, T/h = 1e-330 and v/P_s ≈ 1.5e-330 underflow, and
  f(1) = 0 reads as `BoundaryNoMargin`, although the exact f(1) is about −1.5e-30 and the
  economy is `NoInteriorAtZero`.

The quantities the regime tests compare are products and quotients of up to about six
parameters, so within the bounds they stay within about [1e-180, 1e180]. u and D are not
bounded: a long lag with a large ρ can make u huge (ρ = 1e30 and J_b = 10 give 1e300), and
D(1) can be arbitrarily close to 0. A price may then overflow, and the solve returns
`SolveError::NonFinite`, never a wrong regime. The bounds are a guard against absurd
inputs; the paper's scale parameters lie between 0.05 and 10.

**Curvature ceiling.** x* is a double, and γ and the prices are evaluated there. Across
one spacing of doubles at x, x^k moves by at most max(k, 1)·spacing/x ≤ max(k, 1)·2^-52
relative (Bernoulli for k ≥ 1; for k < 1, (1 − t)^k ≥ 1 − t), so γ = η(g0 + g1·x^k) moves
by less than max(k, 1)·2^-52 relative whatever g0 and g1 are. `CURVATURE_CEIL` = 1024 =
2^10 caps that at 2^-42 ≈ 2.3e-13, a factor of four inside the gate's 1e-12 for the
order-1 sensitivities that carry γ's error into the prices. The paper uses k = 1, and G5
draws k up to 6. Above the ceiling γ stops being resolved by the
doubles of x: at k = 1e20, x^k is 0 at 1 − 2^-53 and 1 at 1, and the review of 2026-09-25
found the solve returned `Interior` at x* = 1 − 2^-53, γ* = 0.2, v = 0.1159 and a labour
residual of 0.029 hours, against 1 − x* = 2.55e-20, γ(x*) = 0.2622 and v = 0.1527 from a
60-digit solve in s = 1 − x. At the ceiling, G8 solves the same economy against a 70-digit
golden; a scratch comparison of 2000 random interior economies with k up to 1024 against
generate.py's solve found every output within 5.1e-14 relative.
**Negative zero.** −0.0 passes every check that admits 0, so a, λ and ρ are stored with
−0.0 made +0.0; otherwise ρ = −0.0 would print interest = −0.0, a different line for the
same economy.

**Name clash:** J(x) is the capability integral of SSRN eq 21. The dynamics thread also
uses "J" for the build lag. The lag is `build_lag` (J_b).

**Numeraire:** r = 1. Every price is a ratio to r, and v = w/r.

## 3. Equations

All quantities are per period. x is a candidate threshold: machines perform tasks
[0, x) and people perform (x, 1].

### 3.0 User-cost factor

Source: dynamics/checks/check_dynamics.py:45 (derived in U3 at :47-64);
dynamics/code/dynamics/model.py:32-35.

    u = (ρ + δ)(1 + ρ)^(J_b - 1)

Special cases:
- (ρ, δ, J_b) = (0, 1, 1) gives u = 1. This is the SSRN Appendix B flow benchmark exactly
  (check_macro A3, :132).
- J_b = 1 gives u = ρ + δ, the SSRN A.4 durable asset.
- J_b = 1 and δ = 1 give u = 1 + ρ, SSRN A.4's "inputs advanced one period".
- J_b = 0 and δ = 1 give u = 1 (U4b).
- ρ = 0 gives u = δ for every J_b.

**Accuracy of u.** 1 + ρ is rounded once and raised to J_b − 1 by repeated squaring,
and each squaring doubles the relative error it inherits. The error in u is at most about
2·J_b·2^-53 ≈ J_b·2.2e-16 relative. Against an 80-digit u (review of 2026-09-25) it was at
most 4.4 ulps for J_b ≤ 4, 26 for J_b ≤ 20, 80 for J_b ≤ 60 and 298 for J_b ≤ 200. The
bound reaches the gate's 1e-12 at J_b ≈ 4500. V_m = b/D amplifies it by u(a + λγ)/D
near the viability edge: at J_b = 153 with D(1) = 1.5e-7 the review found x* off by
1.25e-8. A later unit that needs long lags near D → 0 should compute u in double-double
arithmetic, which keeps it platform independent.

The **timing that defines u** (check_dynamics.py:9-13):
- the build cost is paid at the start of period t;
- the first service comes in period t + J_b;
- the first service is undepreciated;
- survival is (1-δ) per service period after that.

### 3.1 Price block, with r = 1

    D(x)   = 1 - u(a + λγ(x))                viability; must be > 0
    V_m(x) = b / D(x)                        cost to build one machine; equals a·p_m + λ·v + b  (SSRN A.4, p.27)
    p_m(x) = u·V_m(x) = u·b / D(x)           machine-service price, main.tex "c"  (SSRN A.4, p.28)
    v(x)   = γ(x)·p_m(x)                     task margin w = p_m·γ(x*)  (SSRN Prop 1, eq 21)
    p(x)   = v(x)(1 - x) + p_m(x)·J(x)       price of the good = its unit cost  (SSRN eq 22)
    P_s(x) = p(x) + h                        basket price  (SSRN eq 22)

At u = 1 these reduce to v = bγ/(1-a-λγ) and p_m = b/(1-a-λγ) (SSRN eq 21, p.28).

**How D is computed.** D = (1 − u·a) − u·λγ, with 1 − u·a from one fused multiply-add
(core's `num::fma`, which is libm's and correctly rounded), in the solve and in `closure`
alike. The plain 1 − u(a + λγ) rounds a + λγ, which is of order 1, before the
cancellation: that costs up to 1.1e-16/D relative, and the prices scale as 1/D. The review of 2026-09-25 found p_m off by 8.1e-3 relative at
D(x*) ≈ 5.6e-15 with the plain form. The fused form's error is about
2^-53·(1 − u·a)/D, none when λγ = 0, and 1 − a is exact at u = 1 for a ≥ 0.5. The prices
are still conditioned as 1/D near the viability edge; G4 case G gates them at
D(x*) = 8.4e-7.

### 3.2 Quantity block

Source: macro.py:76-91; at u = 1 these are SSRN eq 23-24 (p.29).

    s_K(x)          = 1 - a·δ                        net service yield per installed machine
    Y(x)            = T / (h + b·δ·J(x)/s_K)         baskets = output of the good  (land clearing)
    K(x)            = Y(x)·J(x)/s_K                  installed machines = machine services per period
    final_hours(x)  = Y(x)(1 - x)
    machine_hours(x)= λ·δ·K(x)
    n_D(x)          = final_hours + machine_hours    = Y·[1 - x + λδJ/(1 - aδ)]
    n_S(x)          = N·F(ln(1 + v(x)/P_s(x)))       SSRN eq 10, 24;  F(z) = clamp(z/χ_max, 0, 1)

**Clearing identities** (tested):
- services: K = Y·J + a·δ·K;
- land: T = h·Y + b·δ·K;
- hours: N_a = Y(1-x) + λ·δ·K.

**Notes on the quantity block:**
- 1 − aδ is computed with one fused multiply-add (`num::fma`), so it keeps its relative
  precision as aδ → 1.
- New builds per period equal δK.
- For J_b > 1 the steady-state quantities are unchanged; only u changes.
- The SSRN states only the u = 1 quantities. The δ-terms are macro.py's extension of
  A.4 (its docstring, :72-78).

### 3.3 Equilibrium

Find x* in (0, 1) such that n_D(x*) = n_S(x*) (SSRN Lemma B.1, p.29). Then
N_a = n_D(x*).

**Worker choice.** Utility is ln(1 + w·n/P_s) - χ·n with n in {0, 1} (SSRN eq 8, p.11).
A worker works exactly when χ ≤ ln(1 + v/P_s), which is the same as
w ≥ s = (e^χ - 1)·P_s (eq 9).

### 3.4 Outputs and identities at x*

    final_hours  = Y·(1 - x*),   N_a = final_hours + λ·δ·K     with 1 - x* carried separately (§4 step 4)
    interest     = (u - δ)·V_m·K       = ρ·V_m·K when J_b = 1; ρ·W_K in general  (check_dynamics L1-L2, :155-169)
    I            = v·N_a + T + interest            SSRN eq 15 when interest = 0; macro.py:109
    Y·P_s        = I                               identity, not imposed  (SSRN App. C, p.30; check_macro I1 :59, A2 :129)
    labor_share  = v·N_a / I;   capital_share = interest / I
    real_wage    = v / P_s;     participation = min(N_a / N, 1);   support_cost = N·P_s
    worker_baskets   = N + v·N_a/P_s
    provider_baskets = (T + interest)/P_s - N
    funded       = provider_baskets > 0                     T + interest > N·P_s  (macro.py:113)
    lemma_b1     = n_S(1) > n_D(1)  and  T > N·P_s(1)      SSRN eq 25

Participation is capped at 1: where supply is saturated at the root (F = 1, so n_S = N),
n_D(x*) can exceed N by a few ulps (N_a is not capped).

Both flags are strict. `funded` is computed from `provider_baskets`, so the two never
disagree. macro.py's comparison T + interest > N·P_s is the same in exact arithmetic, but
in f64 it rounds differently: within a rounding of the boundary it can read false while
`provider_baskets` = 2^-51 > 0 (a probe of 2039 boundaries in N found 366 such points
within four ulps of them). There the sign is not decidable in f64, as for the regimes
(§4 step 3). G8 pins both flags at an exact f64 tie.

Machine wealth is W_K = V_m·K·ω with ω = (1+ρ)^(J_b−1) + δ·Σ_{i<J_b−1}(1+ρ)^i
(check_dynamics L2). Since u − δ = ρ·ω exactly, the code computes interest as ρ·ω·V_m·K.
That form has no cancellation when ρ is small against δ.

**Three-taxes shares (flow case, u = 1 only).** Sources:
three-taxes/checks/check_three_taxes.py:27-61; SSRN p.9.
- φ_w = λγ(x*)/(1-a) = w·λ̃_m/p_m.
- φ_r = 1 - φ_w = r·b̃_m/p_m.
- For u ≠ 1 the price-side analogue is u·λγ/(1-ua). Its "rent" part then includes
  interest carry, so φ is reported only when u = 1.

### 3.5 Cost-system view (bridge to 1b and 1c)

Source: SSRN eq 2-4 (p.8) and the matrix display on p.29.

Order the rows as (good, machine services, space):
- prices: p = (p, p_m, r);
- input matrix: A = [[0, J(x), 0], [0, a, 0], [0, 0, 0]];
- labour vector: λ = (1-x, λ, 0);
- land vector: b = (0, b, 1).

Then:
- p = (I-A)⁻¹(λw + br), with totals λ̃ = (I-A)⁻¹λ and b̃ = (I-A)⁻¹b;
- L_s = zᵀλ̃ = 1 - x + λJ/(1-a);
- B_s = zᵀb̃ = h + bJ/(1-a);
- P_s = w·L_s + r·B_s;
- Y = T/B_s;
- n_D = T·L_s/B_s (SSRN eq 11).

The price-side statements hold at u = 1. Y = T/B_s and n_D = T·L_s/B_s also need δ = 1:
at u = 1 with δ < 1 (for example ρ = δ = 0.5) the price side is this system, but the
clearing side is not. The code reports the view whenever u = 1 exactly in f64.

With u ≠ 1:
- the machine row on the **price** side is (u·a, u·λ, u·b);
- on the **clearing** side it is (δ·a, δ·λ, δ·b);
- the two agree only when u = δ, that is when ρ = 0 (J_b then drops out). The
  difference is interest.

### 3.6 Price block on its own (no equilibrium solve)

`closure(a, λ, γ*, b, r, u)` returns:
- p_m = u·b·r/(1 - u(a + λγ*));
- w = γ*·p_m;
- D, and (φ_w, φ_r) when u = 1.

Sources: main.tex:254-258 (u = 1); main.tex:690 and SSRN A.4 (u = 1+ρ and u = ρ+δ);
check_pinning.py:269-296.

## 4. Solution method

Mirrors macro.py:98-114. Existence and uniqueness are proved at SSRN p.29.

**Step 1: validate and compute u.** Validate the parameters listed in §2, then compute
u. The power (1+ρ)^(J_b−1) is taken by repeated squaring in the crate's own code.

**Step 2: check viability at x = 1.** Require D(1) > 0; otherwise return `NotViable`.
- macro.py:100 asserts this.
- D(1) = −∞, where u·λγ(1) overflows, is `NotViable`: 1 − u·a is finite (u is finite and
  a < 1), so D is finite or −∞, and −∞ is certainly negative. Only a NaN would be a
  `SolveError`.
- Because γ is increasing, D(1) > 0 implies D(x) > 0 on all of [0, 1].

**Step 3: set up the root function and bracket it.** Let f(x) = n_D(x) - n_S(x), with
lo = 1e-12 and hi = 1 (macro.py:102).
- If f(1) ≥ 0, return `BoundaryNoMargin`. Labour holds no machine-contestable task: x*
  would be 1 and workers would only build machines. This is SSRN §3.1's boundary case,
  which unit 1d handles. Classify it; do not solve it.
- If f(lo) ≤ 0, return `NoInteriorAtZero`. This means f(lo) ≤ 0, not that no root
  exists: a root can lie in (0, lo), where the solve does not look, as in macro.py:102.
  At x = 0 the case cannot happen when T/h > N, because n_D(0) = T/h and n_S ≤ N. At
  lo = 1e-12 it can. To first order in lo,

      n_D(0) − n_D(lo) = (T/h)·lo·[1 + δγ(0)(b/h − λ)/(1 − aδ)],

  which grows without bound as aδ → 1. G8 has two instances: T/h − N = 5e-12 against a
  gap of 1.1e-11, and a = 1 − 2^-53 with λ = 0, where n_D falls from T/h = 10 to 0.014 at
  lo although T/h − N = 6 (a root lies near 3.6e-15).

**Exact zeros.** The three tests are applied to the f64 values with the convention
above: D(1) = 0 is `NotViable`, f(1) = 0 is `BoundaryNoMargin` and f(lo) = 0 is
`NoInteriorAtZero`. G8 constructs each exact zero.

**Decidability.** When D(1) is within a few ulps of 0 (relative to 1), or f(1) or f(lo)
within a few ulps of 0 relative to n_D, the f64 sign can differ from the exact sign on the
same inputs, and from another implementation's. The regime is then not decidable in f64.
A comparison of regimes against a reference must exclude these cases, or report them as
ambiguous. The review of 2026-09-25 compared four-way regimes with macro.py (patched for
u and J_b) on 3616 instances with J_b from 1 to 12: all 25 disagreements had a true
|f(1)| or |f(1e-12)| of at most 3.8e-16, and they came from numpy's log1p, which differs
from glibc's in the last bit on 8.8% of arguments. With glibc's log1p there were none. (The
oracle then took ln_1p from the platform; since P1.1 it takes libm's, whose outputs equal the
glibc build's on 4930 of 5000 random economies and are within 7.0e-16 on the rest, with every
regime the same, §8.)

The band widens near the viability edge and with the build lag. u carries up to about
J_b·2.2e-16 relative error (§3.0), so D(1) = 1 − u(a + λγ(1)) is known only to about
J_b·2.2e-16·u(a + λγ(1)) absolute, which is about J_b·2.2e-16 near the edge, where
u(a + λγ) is close to 1: a D(1) within that of 0 is not decidable. The prices scale as 1/D
and carry u's error into v/P_s amplified by u(a + λγ)/D, so f(1) and f(lo) are known only
to about 2^-53·n_D·(1 + J_b·u(a + λγ)/D), not to a few ulps of n_D.

**Step 4: find the root.** There is one unknown, x; everything else is closed form in x.

The root is unique:
- n_D is strictly decreasing. Y falls with x, and the bracket's derivative is
  -1 + λδγ/(1-aδ) < 0, because δ(a+λγ) ≤ u(a+λγ) < 1.
- v/P_s is strictly increasing, so n_S is nondecreasing:
  d(v/P_s)/dx = v'(x)·[h + u·b·J/(1-u·a)] / P_s² > 0 (SSRN p.29 at u = 1).

The code uses deterministic bisection:
- it stops when the floating midpoint equals lo or hi, so they are adjacent doubles;
- it returns the endpoint with the smaller |f|;
- it has no tolerance to tune, and it gives the same double on every run;
- `MAX_BISECTION_STEPS` = 128 guards the loop. Doubles in [1e-12, 1] are at least 2^-92
  apart, so at most about 93 steps are needed. G1 takes 53;
- ln(1 + v/P_s) is computed with `ln1p` (core's `num::ln1p`, libm's `log1p`). G3 pins a
  point where z = v/P_s ≈ 7e-7, at which f64 ln(1 + z) is off by 1.2e-10.

`x_star` is one of the two adjacent doubles that bracket the sign change of the f64 f,
the one with the smaller |f|. It lies in [1e-12, 1] and can equal either end: when the
true root is within half an ulp of 1 (1 − x* < 2^-54 ≈ 5.6e-17), 1.0 is the nearest
double, and the result is `Interior` with x_star = 1.0. f(1) < 0 still holds, so the
economy is interior.

**1 − x* is carried separately.** The doubles near 1 are 1.1e-16 apart, so 1.0 − x_star
is good only to about 1e-16 absolute, all of 1 − x* once x* rounds to 1. Bisection ends
with adjacent doubles lo < hi and f(lo) > 0 > f(hi); the root of the straight line
through them is at 1 − x* = (1 − hi) + θ·(hi − lo) with θ = −f(hi)/(f(lo) − f(hi)) in
[0, 1]. 1 − hi is exact for hi ≥ 1/2, hi − lo is exact, and nothing cancels, so this
`one_minus_x_star` keeps full relative precision. It lies in [1 − hi, 1 − lo], and where
f's rounding noise swamps its slope over one spacing it is no worse than 1.0 − x_star.
final_hours = Y·(1 − x*), and through it N_a, participation, the labour share, the
worker baskets, λ̃_good and L_s, use it. Everything else is evaluated at the double
x_star.

macro.py uses `scipy.brentq` with xtol = 1e-15 and computes N_a = n_D(x*) from its x*.
Where 1 − x* is small, that tolerance and the conditioning of 1.0 − x* limit its outputs
proportional to 1 − x* (see §6 G3); the oracle's are not limited that way.

**Accuracy near the ends of the bracket.**
- Near full automation, 1 − x* is resolved to about the noise in f over |f'|. When the
  hours are mostly at final tasks that is a few ulps of 1 − x* relative: G3 gates N_a at
  1e-12 where 1 − x* is 4.6e-7 and where it is 4.6e-21 (x* rounds to 1.0). When the hours
  are mostly machine building (λ > 0), final_hours = Y(1 − x*) is known only to a few
  ulps of N_a absolute, since that is how well f is known; N_a itself keeps full
  precision. Against the review's 3616 instances with a 100-digit truth, final_hours was
  within 3.3·2^-52·N_a everywhere (732 before round 2), and N_a, participation and the
  labour share within 3.4e-15 relative (2.5e-13 before).
- Near x* = 0, the outputs proportional to x* (J, K, machine_hours, interest and
  capital_share) are resolved only to about eps·n_D/|f'| absolute, and cannot meet 1e-12
  relative once x* ≲ 1e-6. This is the conditioning of the root, not the representation
  of x, and no parameterisation removes it.
- Across 2475 instances the review found x* always a root of the true f to within 4.9
  times the f64 floor, max(eps·n_D, |f'|·ulp(x*)).
- Near the viability edge, prices scale as 1/D (§3.1). x* is a double, so γ(x*) is off
  by up to about max(k, 1)·2^-53 relative (§2, curvature ceiling), and D carries that
  times uλγ/D: at the double x* the outputs are within about
  max(k, 1)·2^-53·(1 + uλγ/D) relative of the exact equilibrium, besides D's own rounding
  (§3.1) and u's error (§3.0). The labour market clears only as well as the doubles of x
  resolve n_S there: an `Interior` result with D(x*) within about 1e-5 can carry a labour
  residual up to the 1e-9 net (step 5), and closer still the solve refuses.
- Consequences: a differential comparison should compare the x*-proportional fields at an
  absolute tolerance of about eps·n_D/|f'|, and final_hours at a few ulps of N_a. Phase 2
  tolerances near either end must be registered with this in mind.

**Step 5: evaluate and report residuals.** Evaluate §3 at x* and set N_a = n_D(x*).
Return the residuals:
- |N_a − n_S(x*)|: demand at the reported equilibrium (N_a, with the carried 1 − x*)
  against supply at the reported prices (at the double x*), in hours. It is not |f| at
  the double x*, which near full automation is N_a itself (G3 at η = 1e-20, where x*
  rounds to 1.0);
- |Y·P_s - I| / I;
- |h·Y + b·δ·K - T| / T;
- |K - Y·J - a·δ·K| / K;
- |p_m - u(a·p_m + λ·v + b)| / p_m.

Report `lemma_b1` and `funded` as well.

**The labour-residual net.** If |N_a − n_S(x*)| > `LABOR_RESIDUAL_NET`·N_a, with
`LABOR_RESIDUAL_NET` = 1e-9, the solve returns `SolveError::LaborNotCleared` instead of
`Interior`. Bisection finds a sign change of the f64 f; for a continuous f that is a root,
and the residual is the rounding in f plus n_S's change across one double of x (under
2.3e-13 times n_S's sensitivity to γ at the curvature ceiling). Over 109,281 random
interior economies with every parameter spread over several decades, the largest was
1.0e-13 of N_a. A jump in f leaves a residual of the jump's size (0.23 of N_a at the
review's k = 1e20). The net is a safety net, not a tolerance: it never changes a result,
it refuses one whose labour market does not clear to a part in 10^9. It trips on a
schedule that breaks the continuity contract (G8 constructs one), and very close to the
viability edge, where n_S's sensitivity to γ grows like 1/D and n_S can move by more
than 1e-9 of itself between adjacent doubles. In a probe of 188,336 interior economies
drawn close to the edge (D(1) log-uniform in [1e-12, 0.5]) it refused 3,419, all with
D(x*) < 5e-5, and in each n_S moved across the final bracket by at least twice the
residual: the equilibrium was not resolved in f64 to 1e-9, and neither were its prices.

**Which equation fixes which unknown:**
- the task margin fixes v;
- the user-cost price fixes p_m;
- zero profit fixes p;
- land clearing fixes Y;
- service clearing fixes K;
- labour clearing is the root equation;
- goods clearing follows by Walras' law and is checked through the income identity and
  the basket count (worker plus provider baskets equal Y), not imposed.

## 5. Result type

`Economy::solve` returns `Result<Regime, SolveError>`:

    enum Regime { Interior(Box<Eq1a>), BoundaryNoMargin { f_at_1 }, NotViable { d_at_1 }, NoInteriorAtZero { f_at_0 } }

`f_at_0` is f at lo = 1e-12. `SolveError` reports a quantity that overflowed or became
NaN (a bracket value, or the key of the first non-finite output in `Eq1a::outputs`, so an
`Interior` result never carries NaN or infinity), a bisection that hit its cap, or
(`LaborNotCleared`) a result that failed the labour-residual net of §4 step 5. Valid
parameters of ordinary size never produce one; G8 reaches both non-finite paths with
ρ = 1e30 and J_b = 10, which make u = 1e300, and the net with a γ that jumps. D(1) = −∞
is `NotViable`, not an error (§4 step 2).

`Eq1a` carries:
- `x_star`, `one_minus_x_star` (§4 step 4), `gamma_star`, `j_star`, `u`;
- prices: `v`, `p_m`, `v_m`, `p`, `p_s`;
- quantities: `y`, `k`, `final_hours`, `machine_hours`, `n_a`, `participation`;
- income: `income`, `interest`, `labor_share`, `capital_share`, `real_wage`,
  `support_cost`;
- baskets: `worker_baskets`, `provider_baskets`;
- flags: `funded`, `lemma_b1`;
- `phi_w` and `phi_r`, and `cost_system` (λ̃, b̃, L_s, B_s), all `Some` only when u = 1;
- `residuals`;
- `bisection_steps`.

`Eq1a::outputs()` lists every output with its key, in field order: the one list that the
dump prints and that the solve checks for finiteness.

`Economy::at(x)` returns the §3.1-3.2 quantities at any candidate threshold.

`closure` returns `ClosureError::NonFinite` if p_m or w overflows, so its `Ok` prices are
finite too.

Everything is expressed in r = 1 units. No agent may read the oracle (PLAN R13).

## 6. Tests and golden numbers

- Full-precision values are compared at 1e-12 relative, the income identity too
  (ADDENDUM A7).
- Values published to five decimals are compared at 5e-6 absolute, half a unit in the
  last place. Figure 3's two-decimal caption values use the same rule, 5e-3.
- The goldens come from `goldens/generate.py`, which solves each instance with mpmath at
  70 digits from §3's equations and imports nothing from laborformal. Its output,
  `goldens/goldens.txt`, carries 30 significant digits; the Rust constants carry 20.
  (70 digits, not 50, so that 1 − x* keeps 49 digits at G3's deepest point.)
- `goldens.txt` records FNV-1a digests of generate.py and of the goldens, and the gate
  recomputes both, so a hand-edited or stale goldens.txt fails even where mpmath is
  missing. Only `generate.py --check` proves the values are generate.py's output; it must
  be run after any change to either file.
- The 20-digit values agreed to the last digit shown (within 4e-20 relative) with the
  day's earlier 50-digit solves, which were written separately as scratch scripts and
  are not kept. The review of 2026-09-25 re-solved 40 goldens independently at 60 digits
  and matched them within 4.5e-30. Only generate.py is kept. macro.py's f64 values match
  the goldens within 3.0e-16 relative on the G1 values of the ADDENDUM's §5 table.
- The oracle's f64 values match the goldens within 9.3e-16 relative everywhere and within
  4.1e-16 on G1, except G4 case E's provider baskets, 8.8e-15: a difference of two terms
  near 7.3 that leaves 0.074; and G8's curvature-ceiling case's γ(x*), v and w/P_s,
  1.3e-14: x* is a double, off by 3.6e-17 relative, and at k = 1024 γ moves 340 times as
  much (§2, curvature ceiling). These maxima are the same with libm (P1.1) as with the
  platform's maths before it.

### G1: the SSRN Appendix B instance

Parameters: N 4, T 10, h 1, a 0.3, b 0.4, λ 0.05, γ = 0.2 + 0.8x, χ ~ U[0, 1], ρ 0,
δ 1, J_b 1.

**Published (SSRN p.30; check_macro.py:31-34):** x* 0.86315, v 0.54344, Y 7.88061,
N_a 1.34338, final-task hours 1.07846, machine-sector hours 0.26492, N·P_s 5.44630.

**Full precision:**

| quantity | value |
|---|---|
| x* | 0.86315041816243703192 |
| γ(x*) | 0.89052033452994962553 |
| J(x*) | 0.47064154138208336959 |
| v | 0.54343596069677832842 |
| p_m = V_m | 0.61024542576405559489 |
| p | 0.36157583177980927464 |
| P_s | 1.3615758317798092746 |
| Y | 7.8806055249729076768 |
| K | 5.2984861875677308081 |
| final-task hours | 1.0784575707193308057 |
| machine-sector hours | 0.2649243093783865404 |
| N_a | 1.3433818800977173461 |
| N_a / N | 0.33584547002442933653 |
| N·P_s | 5.4463033271192370986 |
| I | 10.730042022593547301 |
| labour share | 0.068037200698407852314 |
| w / P_s | 0.3991228016925181936 |
| φ_w | 0.063608595323567830395 |
| φ_r | 0.9363914046764321696 |
| L_s | 0.17046683479342606591 |
| B_s | 1.2689380236469047826 |
| λ̃ | (0.17046683479342606591, 1/14, 0) |
| b̃ | (0.26893802364690478262, 4/7, 1) |
| coverage T/(N·P_s) | 1.8361077963113360457 |
| worker baskets | 4.5361743397275634938 |
| provider baskets | 3.344431185245344183 |

`funded` and `lemma_b1` are both true.

**Bracket values:**
- f(1e-12) = 9.6046166614411005933.
- n_D(1) = 15/47 = 0.31914893617021276596 and n_S(1) = 1.4847041385181291062, so
  f(1) = -1.1655552023479163402.
- P_s(1) = 89/65 = 1.3692307692307692308 and N·P_s(1) = 5.4769230769230769231.
- v(1) = 8/13 and D(1) = 0.65.

### G2: SSRN Figure 3

Published only as caption values at 2 decimals (SSRN p.12). laborformal has no script
for the figure.

| case | x* | v | Y | N_a | w/P_s | N_a/N | caption |
|---|---|---|---|---|---|---|---|
| baseline | (G1) | (G1) | (G1) | (G1) | (G1) | (G1) | 0.40 / 0.34 |
| λ = 0 | 0.84058612875612342345 | 0.49855365885994213644 | 7.9518301153058752463 | 1.2676320221545510279 | 0.37287626910307467929 | 0.31690800553863775697 | 0.37 / 0.32 |
| γ halved (η = 0.5, λ = 0.05) | 0.92147755772173964136 | 0.27703901279652668537 | 8.6979344529225284357 | 0.84574124940438527541 | 0.23545004468505030702 | 0.21143531235109631885 | 0.24 / 0.21 |

Both channels lower w/P_s and N_a/N (SSRN p.12).

### G3: automation path

Setup: λ = 0 and γ = η(1+x), that is g0 = g1 = η (SSRN p.30; check_macro.py:35-43).

| η | participation | v |
|---|---|---|
| 1 | 0.43065595858144168109 | 0.98840122029213958952 |
| 0.3 | 0.22927792970224215316 | 0.32367862903339933844 |
| 0.1 | 0.098066745653698975909 | 0.11186280479118877161 |
| 0.03 | 0.032663064734812997025 | 0.034056082139179303478 |
| 0.01 | 0.011242317671650493691 | 0.011402655775197234064 |

Also assert:
- v ≤ 2bη/(1-a) at every η;
- participation strictly decreases along the path;
- lemma_b1 holds and support is funded at every η.

macro.py's participation here is off by up to 4.6e-14 relative (at η = 0.01, where
1 − x* ≈ 0.0045): its brentq xtol of 1e-15 moves 1 − x* by that much. The oracle's
bisection is within 4e-15.

A point evaluation pins `ln1p`: at η = 1e-6, n_S(0.25) = 2.8571398469420684487e-6
(z = v/P_s ≈ 7e-7), where f64 ln(1 + z) is off by 1.2e-10.

**Near full automation** (§4 step 4), gated at 1e-12 on 1 − x*, N_a (= final hours, since
λ = 0), N_a/N, the labour share and λ̃_good:

| η | 1 − x* | N_a | N_a/N | labour share |
|---|---|---|---|---|
| 1e-6 | 4.5714249142895853789e-7 | 4.5714209959311200557e-6 | 1.1428552489827800139e-6 | 5.2244799440381028585e-13 |
| 1e-20 | 4.5714285714285714285e-21 | 4.5714285714285714285e-20 | 1.1428571428571428571e-20 | 5.2244897959183673468e-41 |

At η = 1e-20, x_star is exactly 1.0 and the result is still `Interior`. Before round 2,
N_a came from 1.0 − x_star: off by 2.7e-11 at η = 1e-6 and 0 at η = 1e-20.

### G4: durability and interest

**(ρ, δ, J_b) = (0.05, 1, 1), u = 1.05:**
- x* 0.85606144502130589985, v 0.58200502938266013801;
- Y 7.9030007613895119881, N_a 1.3996714144162366065;
- p_m 0.6577449110110797916, V_m 0.62642372477245694438;
- I 10.978817061910314775, capital share 0.014956188659577909534;
- labour share 0.074198868428148129303, w/P_s 0.41895098209636805043.

**(0.05, 0.10, 1), u = 0.15:**
- x* 0.97912908840923014792, v 0.062258997183627810226;
- Y 9.7666856299694170224, N_a 0.23300392857175693251;
- p_m 0.063316170134949956625, V_m 0.4221078008996663775, K 5.8328592507645744403;
- I 10.13761136049759874, capital share 0.012143370384523498204;
- labour share 0.001430967356792732955, w/P_s 0.059980998630410869347.

**(0, 0.10, 1), u = 0.1:**
- x* 0.98694968535939866455, v 0.041015801922299965682;
- Y 9.7636520559177819562, N_a 0.15696222438135841406.
- Nesting test: this case equals the flow benchmark with (a, λ, b) = (0.03, 0.005, 0.04)
  in exact arithmetic. At ρ = 0 the economy is the flow benchmark with its coefficients
  scaled by δ. generate.py asserts the equality to 1e-65 at 70 digits; the f64 test
  compares at 1e-12.

**(0.05, 0.10, 3), u = 0.165375.** Derived here only; laborformal has no number for this
case in this closure.
- x* 0.97675691133921725842, v 0.06889724102020382201;
- Y 9.7676023542515746937, N_a 0.25607895324169275037;
- I 10.178880949831178601.

**Case E, (N, ρ, δ, J_b) = (7.3, 0.05, 1, 1): interest decides funding.**
- x* 0.74326159093471752974, N·P_s 10.034041293511349215 > T = 10;
- interest 0.13554159365380831009, so N·P_s < T + interest;
- provider baskets 0.073843845103477730290 > 0, and funded = true;
- labour share 0.10686847331204750894 and w/P_s 0.37743990302044527907. With interest in
  I, v·N_a/I is not v·N_a/(v·N_a + T), which the u = 1 goldens cannot tell apart.

**Case F, (0.5, 0.5, 1): u = 1 with δ < 1.** The price side is the flow cost system, so
φ_w = λγ(x*)/(1 − a) has no δ in it; the clearing side scales the recipe by δ (§3.5).
- x* 0.86447496527289638279, Y 9.0007632618460365696;
- φ_w 0.063684283729879793302, φ_r 0.9363157162701202067.

**Case G, the viability edge:** a = 1 − 2^-19, λ = 2^-20, b = 0.375, γ = 0.5 + x,
ρ = δ = 0.5 (so u = 1), the rest as Appendix B. Every input is dyadic, so the f64 inputs
equal the decimals exactly.
- D(x*) 8.3644619286869630732e-7, x* 0.62292259686651390085;
- p_m 448325.31153485298836, v 503434.62306970597671, P_s 416453.21351772792983;
- I 3500919.7380127001637.

The plain form of D (§3.1) misses these by up to 1e-10.

Also tested: interest = ρ·W_K (check_dynamics L1-L2) including at ρ = 1e-9; φ and the
cost system are absent when u ≠ 1; in case F, P_s = w·L_s + r·B_s and φ = w·λ̃_m/p_m hold
while Y = T/B_s does not; and a higher ρ automates fewer tasks with a higher v and labour
share (check_macro A5, :140-143).

With δ = 0.1 the instance automates almost every task. Choose other parameters for agent
tests that need an interior margin.

### G5: random-economy identities

Match check_macro.py:45-60 and :108-131. The draws come from a SplitMix64 generator
written into the test, seeded from 923 (check_macro's seed; the generator differs):

| parameter | range |
|---|---|
| N | U(1, 5) |
| T | U(2, 20) |
| h | U(0.1, 2) |
| a | U(0.05, 0.5) |
| b | U(0.1, 1) |
| λ | U(0, 0.1) |
| g0 | U(0.01, 0.5) |
| g1 | U(0.2, 1.5) |
| k | U(0.5, 6) |
| χ_max | U(0.5, 3) |
| ρ | U(0, 0.12) |
| δ | U(0.03, 1) |

Three sets of 60 interior economies each: the flow benchmark (ρ = 0, δ = 1), the table
as given (J_b = 1), and the table with J_b drawn from 1 to 5. Draws that are not
interior are skipped. For each economy:
- the income identity, land clearing, service clearing and the user-cost equation hold
  to 1e-12 relative (laborformal achieves ≤ 2.7e-16; the PLAN asks only 1e-10);
- interest equals ρ·W_K, worker plus provider baskets equal Y, and `closure` reproduces
  p_m and v;
- x* is the double at which f changes sign;
- funded holds exactly when provider baskets are positive, and participation is
  min(N_a/N, 1), never above 1;
- each residual equals its recomputation from the reported fields in §4 step 5's form,
  bit for bit (labour as |N_a − n_S(x*)|), and each residual is nonzero in some economy,
  so the comparison has teeth;
- at the flow benchmark, the §3.5 identities hold and check_macro A3's nesting holds;
- on the grid {1e-12, 0.01, ..., 1}, n_D strictly decreases, v/P_s strictly increases,
  n_S never falls, and f changes sign exactly once;
- labour, capital and land shares sum to 1; w/P_s = v/P_s, P_s = p + h and support cost
  = N·P_s at the draw's h; final_hours = Y·(1 − x*) with the carried 1 − x*, which is
  within the spacing of doubles at x* and at 1 − x* of 1.0 − x_star.

**Two general instances.** The identities hold for any γ, h or χ_max, so they cannot
catch a wrong exponent or a misread parameter. G5 therefore also pins every output of
two instances with every parameter off the paper's values:
- flow: N 5.2, T 12.5, h 0.7, a 0.22, λ 0.08, b 0.55, γ = 2.3(0.15 + 0.9x^4.5),
  χ_max 1.6, (ρ, δ, J_b) = (0, 1, 1): x* 0.86829772362082105489, and 32 more outputs
  including φ and the cost system;
- durable: the same with k 2.5, ρ 0.04, δ 0.35, J_b 2 (u = 0.4056), fourteen distinct
  parameters of which none is 1: x* 0.92239054108259182071, and 25 more outputs.

The values are in goldens.txt, and the schedule's unit tests check γ and J directly at
k = 0.5, 2.5 and 6.5.

### G6: replacement closure (price block only)

Sources: main.tex:325-336; check_pinning.py:59-73.

- (a, λ, γ*, b, r) = (0.5, 0.1, 3, 0.2, 1) with u = 1 gives p_m = 1 and w = 3.
- The recipe balances: 1 = 0.5·1 + 0.1·3 + 0.2·1.
- With λ = 0: p_m = 0.4 and w = 1.2, a 60% wage cut.
- D = 0.2.
- (a, λ, γ*) = (0.5, 0.2, 3) gives D = -0.1, which is `NotViable`. D = 0 exactly
  (0.5 + 0.25·2 in f64, or u = 1.25 with a + λγ* = 0.8) is `NotViable` too.
- The recursion p_m = u(a·p_m + λ·w + b·r) is checked directly at every u tested, not only
  the closed form.
- The user-cost forms with s = 1+ρ and s = ρ+δ reduce to the static form at s = 1
  (check_pinning.py:269-296), and at λ = 0 they give SSRN A.4's two displays.

The values are exact rationals; the f64 tests compare them at 1e-12.

### G7: three-taxes shares

On the G6 instance, (φ_w, φ_r) = (0.6, 0.4) exactly, and they equal w·λ̃_m/p_m and
r·b̃_m/p_m. With λ = 0 they are (0, 1).

### G8: regime recognition

Constructed here; laborformal has no instance.

| case | expected result |
|---|---|
| base | `Interior` |
| N = 0.25 | `BoundaryNoMargin`, f(1) = +0.22635492751282969682 |
| λ = 0.6 | `BoundaryNoMargin`, f(1) = +0.71896895969051973502 |
| λ = 0.8 | `NotViable`, D(1) = -0.1 |
| ρ = 0.1, a = 0.6, λ = 0.35 | `NotViable`, D(1) = -0.045; the flow economy is viable (D(1) = 0.05) |
| N = 20, χ_max = 0.05 | `NoInteriorAtZero`, f(1e-12) = -10.000000000011 |
| N = 8 | `Interior` at x* = 0.73337119933110234855, but lemma_b1 = false (N·P_s(1) = 10.953846153846153846 > T = 10) and funded = false |
| N = 7.35 | `Interior` at x* = 0.75208427516884260890, funded = true (N·P_s(x*) = 9.9135366206749867864 < T) but lemma_b1 = false (N·P_s(1) = 10.063846153846153846 > T) |
| λ = 0.7 | `NotViable` with D(1) = 0 exactly (0.3 + 0.7 rounds to 1 in f64 and is 1 in decimal) |
| a = 0.5, λ = 0.1, γ = 1 + 4x | `NotViable` with D(1) = 0 exactly in f64 |
| χ_max = 0.05, N = n_D(1) in f64 (0.31914893617021284) | `BoundaryNoMargin` with f(1) = 0 exactly |
| χ_max = 0.05, N = n_D(1e-12) in f64 | `NoInteriorAtZero` with f(1e-12) = 0 exactly |
| N = 10, T = 10.000000000005, χ_max = 1e-3 | `NoInteriorAtZero` although T/h > N |
| a = 1 − 2^-53, λ = 0 | `NoInteriorAtZero`, f(1e-12) = -2.7587301670408429713, although T/h − N = 6 |
| N = h = 1e300, T = b = 1e-30 | rejected: N is above `SCALE_CEIL` (it underflowed to `BoundaryNoMargin` before) |
| a = λ = 0, ρ = 1e30, J_b = 10, b = 1e10 | `SolveError::NonFinite` at f(1): u = 1e300 and p_m overflows |
| a = λ = 0, ρ = 1e30, J_b = 10, T = 1e10 | `SolveError::NonFinite` at income (the root is found; interest overflows) |
| λ = 1e10, ρ = 1e30, J_b = 10 | `NotViable` with D(1) = −∞ (u·λγ(1) overflows); the dump prints `d_at_1=-inf` |
| T = 4·P_s(1) in f64 (5.476923076923077) | `Interior` with N·P_s(1) = T exactly and n_S(1) > n_D(1): lemma_b1 = false (eq 25 is strict) |
| N = 7.41560525573509 | `Interior` with N·P_s(x*) = T = 10 exactly and provider baskets 0: funded = false; funded = (provider baskets > 0) on 64 ulps of N either side |
| N = 1, k = 1e20 | rejected: k is above `CURVATURE_CEIL` (it returned a wrong `Interior` before) |
| N = 1, k = `CURVATURE_CEIL` = 1024 | `Interior` at x* = 0.99798105131089916681, 1 − x* = 0.0020189486891008331935, v = 0.17577913033884822469, and eleven more outputs, at 1e-12 |
| γ = 0.2 + 0.1x below 7/8 and 1 + 0.1x from 7/8 (a custom `Schedule` with a jump) | `SolveError::LaborNotCleared`: f jumps from +0.70 to −0.51 across one double, and the residual is about 0.4 of N_a |

The exact-zero rows are constructed in f64 and test the ≥/≤ convention of §4 step 3. In
exact arithmetic on the same doubles the sign can differ: the λ = 0.7 case has
D(1) = +1.7e-17 and would be viable (the a = 0.5 case has −2.8e-17). They are f64
facts, so they carry no golden, except λ = 0.7, whose decimal D(1) is 0. The two strict-flag
rows are f64 ties in the same way.

### The dump interface

The gate runs `oracle::dump::line` on the G1 line, G4 cases B and D, G8's N = 7.35 and
N = 8 (where `funded` and `lemma_b1` differ, and where both are false), and G5's durable
instance. It requires exactly the documented keys, each float's bits equal to the
matching `Eq1a` or `Residuals` field (the expected keys are written out in the test,
independently of `Eq1a::outputs`), the flags and `bisection_steps` as printed values, and
`none` for the flow-only outputs when u ≠ 1. For the durable instance the reference
economy is built from a `Params` literal, not from the parser, and every one of its
fourteen values is distinct. It reproduces the G1, G4 case B and G5 durable goldens from
the printed values.

The unit tests parse a line of fourteen distinct values field by field, run
`dump::run` on raw bytes (a blank line, bytes that are not UTF-8, a CRLF line and a last
line without a newline give one output line each), and check that read, write and flush
failures are returned as errors, which `examples/dump.rs` turns into exit status 1.

### G9 (optional, for development only)

A 4096-task linear program that maximizes baskets at the equilibrium hours reproduces Y
within 2.7e-9 relative, and its dual wage-rent ratio is within 5e-6 of v. It was run in
scratch on 2026-09-25; the script is not kept, and G9 is not part of the gate.

## 7. Pitfalls

- **J means two things.** J(x) is the capability integral; J_b is the build lag.
- **ρ means two things.** ρ is interest in A.4, but three-taxes uses ρ for γ(x*) and ℓ
  for b.
- **δ also clashes.** In `pinning/checks/corner/check_repairs.py` and
  `check_baseline_props.py`, δ is time preference and d is wear.
- **Scaling differs by side.** The price side scales the machine recipe by u; the
  quantity side scales it by δ.
- **ρ is an input.** It is never solved for.
- **Ownership does not move the equilibrium.** Who receives interest does not change
  x*, v, Y or N_a; it changes only funding and who consumes what.
  - macro.py gives the interest to the provider, and so does the oracle.
  - dynamics/ finances machines entirely at a world rate and adds a net-export term
    (check_dynamics.py:14-21, EJ4).
- **Uniform F caps participation.** With uniform F, n_S is capped at N.
- **Compare ratios to r, never money levels.**
- **u − δ cancels.** For small ρ, compute interest as ρ·W_K, not (u − δ)·V_m·K.

## 8. Platforms

The gate runs on WSL Ubuntu and on Windows (ADDENDUM ruling 2). On 2026-09-25 the dump
example read the same 3004 lines on both: 3000 random economies, the Appendix B line and
three bad lines. After round 2, 2466 output lines were byte-identical. The others differ
in the last bits, and every regime agrees. On that set the largest difference was 1.65e-14
relative, on one_minus_x_star and final_hours at 1 − x* = 4e-4 with the hours mostly in
machine building, where f's noise sets 1 − x* (§4 step 4); every other field was within
8.3e-15. The review's own 3000-economy set reached 2.8e-14 on provider_baskets, a
difference of two terms. On the review's 3616 edge-heavy instances, 7 regimes differed
between the platforms, all with a true margin of at most 6e-17, where the regime is not
decidable in f64 (§4 step 3). The cause was the platform libm: `ln_1p`, and `powf` with a
non-integer exponent, differ between glibc and the MSVC runtime; `powf(x, 1)` and
`powf(x, 2)` agree. u is computed without libm and is identical. `mul_add` is correctly
rounded on both.

Since P1.1 (2026-09-26) the power, ln(1 + z) and the fused multiply-add go through core's
`num` module, which calls the pure-Rust `libm` crate (its `fma` is correctly rounded, in
hardware where the CPU has FMA and in software where not), so no output depends on the
platform. On 5000 random economies (every regime; J_b from 1 to 12; the Appendix B line
first), the dump output was byte-identical on WSL and Windows, where the build before P1.1
differed on 1021 lines, by up to 1.1e-13 relative on x* at x* = 9.7e-10, where the root is
ill-conditioned (§4 step 4). Against that build, libm changed no regime and no flag, and
moved the interior outputs (residuals aside) by at most 7.0e-16 relative from glibc's, and
by 1.1e-13 from MSVC's (on x* and the fields proportional to it, at that x* = 9.7e-10).
Cross-platform equality is still recorded, not gated.

## 9. Changes from the working spec

1. macro.py's f64 values match the goldens within 3.0e-16 **relative** on the
   G1 table. The working spec said 4.4e-16, which is the absolute figure.
2. The gate's source is the SSRN Appendix B through `paths/code/macro.py`, never
   main.tex's Appendix B (ADDENDUM A7).
3. u = δ exactly when ρ = 0, whatever J_b. The working spec said "ρ = 0 and J_b = 1".
4. Y = T/B_s and n_D = T·L_s/B_s need δ = 1 as well as u = 1 (§3.5).
5. γ(x*)'s 20th digit is 3, not 4: the working spec rounded x* first.
6. The G3 and G4 tables now carry the mpmath goldens. The working spec quoted
   macro.py's f64 values for some entries.
7. G8 gains a `NoInteriorAtZero` case and a case made not viable by u alone, and records
   funded = false at N = 8.
8. Interest is computed as ρ·W_K (§3.4).

### Round-1 review (2026-09-25)

9. Every parameter that must be positive has a floor, `SCALE_FLOOR` = 1e-30 (§2).
10. Participation is min(N_a/N, 1) (§3.4).
11. At exact zeros the regime follows §4 step 3's ≥/≤ convention on the f64 values, and
    regimes within a few ulps of a boundary are not decidable in f64 (§4 step 3).
12. `NoInteriorAtZero` is impossible when T/h > N only at x = 0, not at lo = 1e-12.
13. x_star can equal 1.0 in an `Interior` result, and the accuracy of outputs near the
    ends of the bracket is stated (§4 step 4). u's error grows with J_b (§3.0).
14. `SolveError::NonFinite` names the first non-finite output.
15. The dump prints floats with `{:?}`, so extreme values carry an exponent.
16. New goldens: G3's n_S point for `ln_1p`, G4 case E, and G8's N = 7.35 and λ = 0.7
    rows.

### Round-2 review (2026-09-25)

17. 1 − x* is carried separately, as `one_minus_x_star`, interpolated between the doubles
    that bracket the root, and final_hours, N_a, participation, the labour share, worker
    baskets, λ̃_good and L_s use it (§4 step 4). Near full automation these were limited
    to 1e-16/(1 − x*) relative before; N_a read 0 at 1 − x* = 2.3e-31.
18. D = (1 − u·a) − u·λγ and 1 − aδ are computed with a fused multiply-add (§3.1, §3.2).
19. Scale parameters, λ and ρ have a ceiling, `SCALE_CEIL` = 1e30, and −0.0 is stored as
    +0.0 (§2).
20. `NoInteriorAtZero` means f(lo) ≤ 0; the first-order gap n_D(0) − n_D(lo) is stated
    with its 1/(1 − aδ) factor (§4 step 3).
21. `closure` returns an error rather than infinite prices; the default schedule check
    samples γ for monotonicity; the build-lag error prints an integer, and the scale
    messages print the bounds from the constants.
22. The dump's loop is `dump::run`, tested on raw bytes, and I/O failures exit with
    status 1. The dump prints `one_minus_x_star`, and every output comes from one list,
    `Eq1a::outputs`, which the finiteness check shares.
23. New goldens: G3 near full automation, G4's labour shares and real wages and cases F and
    G, G5's two general instances, and G8's a = 1 − 2^-53 case. goldens.txt carries
    digests the gate checks, and generate.py works at 70 digits with `log1p`.

### Final verification round (2026-09-25)

24. The power schedule's k has its own ceiling, `CURVATURE_CEIL` = 1024 (§2). k = 1e20 was
    accepted under `SCALE_CEIL` and solved to a wrong `Interior`.
25. The labour residual is |N_a − n_S(x*)|, and an interior result must clear the labour
    market to `LABOR_RESIDUAL_NET` = 1e-9 of N_a, or the solve returns
    `SolveError::LaborNotCleared` (§4 step 5).
26. `funded` is computed as provider_baskets > 0, so the two cannot disagree in f64
    (§3.4); both flags are pinned at exact f64 ties (G8).
27. D(1) = −∞ is `NotViable`, not a `SolveError` (§4 step 2).
28. The dump says a `build_lag` above u32::MAX is too large, not that it is not an integer;
    the sampled schedule check prints its sampling from `VALIDATION_SAMPLES`.
29. New goldens: G8 at the curvature ceiling.

### Joining the workspace (P1.1, 2026-09-26)

30. The package is a workspace member: version, edition, publishing and lints come from
    the workspace, and its one dependency is `rustyecon-core`. Under the workspace's
    `clippy.toml`, `powf`, `ln_1p`, `mul_add` and the rest of the platform maths are
    denied, so x^k, ln(1 + z) and the fused multiply-add go through `core::num` (libm),
    which gains `fma` for the purpose (§8). No golden moved, the goldens' error maxima are
    unchanged (§6), and G8's exact tie at N = 7.41560525573509 still ties.
31. The tests spell their dyadic inputs with an exact `pow2(n)`, not `2f64.powi(n)`, and
    the default schedule check's dipping schedule is a tent, not 0.3·sin(20x), with its dip
    between the same two samples.
32. Precision near the viability edge and the widening band of undecidable regimes near
    the edge and at long build lags are stated (§4 steps 3 and 4). Three slips are
    corrected: the curvature bound is max(k, 1)·spacing/x (§2), the curvature-ceiling
    case's x* is off by 3.6e-17, not 3.3e-17 (§6), and its γ moves by about 3.8e-14
    across one double, with the factor g1·x^k/γ ≈ 0.34 (the comment in
    `tests/gate/g8_regimes.rs`).
