# Unit 1f: households and government

Dated 2026-09-27. This is the unit's specification, written before any code. A 50-digit scratch
prototype (one good on the task line and direct space, one machine type, one worker type, the
path's three stretches, in the run's scratch directory, not kept) checked the derivations of §4
and §5.4 on the instances of Appendix B's form and supplied the numbers in §7;
`goldens/generate_1f.py` must reproduce them and compute the rest. The build records where it
departs from this draft in §14. Unit 1f closes Phase 1: §13 lists every item of PLAN Phase 1's
gate and the test that covers it.

- **Pinned to:** laborformal `31b3482`. Every laborformal path and line below is at that commit.
- **Paper:** SSRN 7226858 (posted 2026-09-23), cited as "SSRN p.N (eq k)": §4 (eq 7-9), §5
  (eq 10-11), §7 (Prop 6, eq 16, the corollary, eq 17), A.1 (Government), App. C (the income
  identity and eq 26), App. D (the corollary's proof, D.1 eq 27, D.2, D.3 eq 28, D.4, D.5).
- **Older revision:** `pinning/paper/main.tex` (2026-09-21), cited as "main.tex:N": §7
  :530-555 and App. D :813-906, the CES share :806-811.
- **Reference checks:** `three-taxes/checks/check_three_taxes.py` T1 (:27-61), T6 (:63-85) and T5
  (:105-134); `pinning/checks/check_pinning.py` P5 (:107-129) and A-joint (:225-267);
  `pinning/checks/check_interior.py` rent redistribution (:162-169); `pinning/checks/corner/`
  `check_conditionality.py` D-i (:36-57), `check_feasibility.py` F-i to F-iv (:43-99),
  `check_welfare.py` O-ii and O-iv (:52-89), `check_mix.py` M-i (:36-43);
  `dynamics/checks/check_dynamics.py` :14-21 (external finance), V1 (:181-189), :405 (the
  land-share price index).
- **Gate (the run's brief; PLAN Phase 1; ADDENDUM A7):** three-taxes' resolution ledger
  (φ_w, φ_r) = (0.6, 0.4) on its worked instance inside the full closure (1a has it as a price
  block); the income identity with government to 1e-12; Prop 6's identities and κ; each tax's
  incidence as the paper states; nesting with no government, exactly. Then Phase 1's gate, item
  by item (§13).
- **Built on:** units 1a to 1e (docs/unit-1a.md to unit-1e.md). Every equation, convention,
  constant and precision rule of those units carries over unless this document says otherwise.
- **Goldens:** a new generator, `goldens/generate_1f.py`, writing `goldens/goldens_1f.txt` (§7).

## 0. What the sources give

| source | what 1f takes from it |
|---|---|
| SSRN p.10 (eq 7) | a basket z per person per period, P_s = zᵀp; "a direct rental of space enters at p_h = r and is counted once" |
| SSRN pp.10-11 (eq 8, 9; Prop 3) | u_n = log(1 + wn/P_s) − χn with the basket kept "in both work and exit"; s = (e^χ − 1)P_s; "a proportional rescaling of prices leaves the choice unchanged" |
| SSRN pp.11-12 (eq 10, 11) | N_a = N·F(log(1 + w/P_s)); "all households consume in the basket's proportions"; T·L_s/B_s = N·F(·) with "the endowment T fully used at r > 0" |
| SSRN p.12 (§5) | "a lower willingness to work weakly raises x and w/r, provided an interior equilibrium remains" |
| SSRN pp.15-16 (§7, Prop 6) | R = Σ_j r_j T_j; a tax τ_R on pure ownership rents finances d = τ_R·R/N, "paid identically in and out of work"; (i) the fixed factor's physical supply is invariant and producers pay the gross rental; (ii) an unconditional transfer has a zero differential at the participation boundary, "while participation can respond through income effects"; (iii) (1 − τ_R)R + N·d = R |
| SSRN p.16 (eq 16) | κ = R/(N·P_s); one basket per person needs d = P_s, so τ_R = N·P_s/R = 1/κ; "a tax rate between zero and one can meet this budget exactly when κ ≥ 1" |
| SSRN p.16 | "Replacing existing household or public financing while maintaining total support and prices leaves the participation comparison unchanged. If the payment supplements the existing basket, ... the reservation wage (e^χ − 1)(P_s + d)"; a wage-funded transfer "draws on the labor market as both a financing base and a participation margin" |
| SSRN p.16 (corollary) | with N, T fixed and N_a ≤ N, along equilibria with w/r → 0 wage-tax revenue is a vanishing fraction of income, R/I → 1, and, with B_s bounded away from 0, it funds a vanishing fraction of one basket per person |
| SSRN p.17 (eq 17) | "in a fully automated economy" (no labour in the basket's totals), with r > 0 and B_s > 0, κ = T/(N·B_s), which funds one basket each exactly when N·B_s ≤ T |
| SSRN p.25 (A.1, Government) | "a payroll tax τ_w; a tax τ_R on pure ownership rents, returned as a uniform transfer d = τ_R·R/N; a uniform consumption tax; and a transfer program paying (m_w, m_e) per period in work and exit"; types differ in "access to support, living requirements, and preferences" |
| SSRN pp.30-31 (App. C) | I = pᵀf = w·N_a + rᵀT; "at zero rent, an unused part of a fixed endowment contributes no income"; eq 26, the CES expenditure share α(q) = α^σ q^(1−σ)/((1 − α)^σ + α^σ q^(1−σ)), q = r/p, with its limits as q → ∞ (1, α, 0 for σ <, =, > 1) |
| SSRN p.31 (App. D, the corollary's proof) | 0 ≤ w·N_a/(N·P_s) ≤ (w/r)/B_s since P_s ≥ r·B_s and N_a ≤ N; at full automation P_s = r·B_s and R = r·T |
| SSRN pp.31-32 (D.1, eq 27) | a program (m_w, m_e) "in addition to the supported basket", Δ = m_w − m_e: work iff w ≥ (e^χ − 1)(P_s + m_e) − Δ; "an unconditional payment m_w = m_e = d gives Δ = 0 and the reservation wage (e^χ − 1)(P_s + d)" |
| SSRN p.32 (D.2) | "Workers bear ε_D/(ε_D + ε_S) of a payroll tax, so the same fraction of a wage-funded transfer is circular"; "under a tax on pure rents, the factor's fixed physical supply is preserved while redistribution can change demand" |
| SSRN pp.32-33 (D.3, eq 28) | κ = R/(N·P_s) = T/(N[L_s(w/r) + B_s]); "N·B_s ≤ T is necessary"; "a lower wage–rent ratio raises coverage holding them fixed"; the two-item form (reproduced in 1e) |
| SSRN pp.33-34 (D.4, D.5) | the provider's budget and P_s; "a broad consumption tax reaches those other financing sources as they fund consumption, but its labor-origin slice is distortionary in the same direction as a wage tax"; the index D = φ_C(1 − κ)² |
| main.tex:534-545 (Prop welfare) | the same three claims, with receipts y_i = w·n_i + (1 − τ_R)Σ_j ω_ij r_j T_j + τ_R·R/N and Σ_i ω_ij = 1 |
| main.tex:552 | "with utility log g − χn and positive goods-unit resources A in exit, a common transfer d raises the reservation wage to (e^χ − 1)(A + d)" |
| main.tex:819-825 (conditionality) | "an in-work benefit (Δ > 0) lowers the reservation wage and expands participation. In the sloped regime, part of the benefit is therefore offset by a lower equilibrium wage through the task margin"; the reverse for Δ < 0 |
| main.tex:829-833 (incidence) | workers bear ε_D/(ε_D + ε_S); "a tax on pure rents of a fixed factor cannot shrink that factor's physical supply" |
| main.tex:806-811 | eq:ces-share, the same α(q) |
| check_three_taxes.py T1 :27-61 | W_u = λw/(1 − a), R_u = ℓr/(1 − a) (ℓ = b); the ledger closes, W_u + R_u = c; φ_w = λρ/(1 − a), φ_r = 1 − φ_w; λ → 0 gives (0, 1); the instance (a, λ, ρ, ℓ, r) = (0.5, 0.1, 3, 0.2, 1): (0.6, 0.4) |
| check_three_taxes.py T6 :63-85 | a consumption tax at rate t on one unit splits t·c into a wage leg t·c·φ_w and a rent leg t·c·φ_r: 0.15 and 0.10 at t = 0.25; at λ = 0, c = 0.4 is all rent content and a 30% consumption tax and a 30% rent tax each take 0.12 |
| check_three_taxes.py T5 :105-134 | R = R₀ + φ_r·τ·R, so R* = R₀/(1 − φ_r·τ); at τ = 1 the gap is φ_w; the multiplier 5/3 at φ_r = 0.4; "equal baskets move nothing"; κ(g) = κ₀·g/((1 − h) + h·g) under a rent-price scaling g |
| check_pinning.py P5 :107-129 | incomes (1 − τ_R)ω_i·R + τ_R·R/N sum to R for every τ_R; (w + d) − (s + d) = w − s |
| check_interior.py :162-169 | wage·hours + (1 − tax)·rent·ownership + dividend sums to wage·hours + rent·ownership at tax 0, 0.4, 1 |
| check_conditionality.py D-i :36-57 | R = s(y) − Δ; the unconditional payment's compensated response is 0; an in-work benefit lowers R one for one |
| check_feasibility.py F-i to F-iv :43-99; check_welfare.py O-iv :80-89 | κ = qT/(N(g_s + q·h_s)) from primitives; κ − 1 has the sign of the land slack T − N(g_s/q + h_s); the numeric world rT = 100, N = 50: κ = 20/19 |
| check_mix.py M-i :36-43 | at the pure corner consumption spending equals r·T, so a uniform consumption tax and a proportional rent tax share one base |
| check_pinning.py A-joint :225-267 | a Cobb-Douglas household with land share α = 0.3 and full participation, γ = 1 + 4x, a 0.5, λ 0.1, b 0.2, T 10, N 1: x* 0.8752410404997317, w 1.5160527572576763, c 0.33682844446030447, r 0.08404473252192293, Y 1.6495500577338336, X 7.942038511535193 (good numeraire) |
| check_dynamics.py :14-21, :181-189, :405 | the dynamics thread's household: hand-to-mouth, Inc = w·N_a + r·T, machines externally financed with a net-export term; land clearing T = bX + b_I·I + α·Inc/r; P = p^(1−α) r^α |

### 0.1 There is no equilibrium with a government at the pin

laborformal has no equilibrium instance with a tax, a transfer or a price-responsive basket
except check_pinning's A-joint, and A-joint is not viable at the top of the task line
(decision 64; §2.13). The government's statements are identities at given prices (Prop 6,
P5, check_interior), the participation rule at given prices (eq 27, D-i) or local and
partial-equilibrium (D.2's incidence). Every 1f golden is therefore constructed. The one
three-taxes instance, (a, λ, γ*, b, r) = (0.5, 0.1, 3, 0.2, 1), is a price block; §3.3 builds an
equilibrium around it (TX) in which the margin sits at γ(x*) = 3 exactly and the basket's own
ledger is (0.6, 0.4) too.

### 0.2 Two words the sources use loosely

- **Supplement or replace.** A transfer that supplements the supported basket adds to what a
  person has in both states and raises the reservation wage (SSRN p.16, main.tex:552). One that
  replaces existing financing, with total support and prices held, leaves the comparison as it
  was. 1f has both, per economy (§2.4).
- **Gross and net.** Producers pay the gross wage and the gross rental (Prop 6 (i)); the cost
  system, the task margin and every quantity see gross prices only. Workers receive the net wage,
  owners the after-tax rent, and households pay consumer prices. A tax enters the equilibrium
  only through who offers hours (§4.10).

## 1. Scope

**In scope:**
- the government of SSRN A.1: a payroll tax, a tax on market rent, a uniform consumption tax,
  a uniform transfer to every person and a transfer program (m_w, m_e), with a balanced budget
  closed by the owners' levy (the default) or by the uniform transfer (§2.5);
- each transfer supplementing the supported basket, or replacing the provider's support (§2.4);
- the participation rule with government, per worker type and under both exit forms, and the
  walled types' wages under it (§4.4, §4.5);
- the households' accounts: each type's workers and exiters, the provider, the government; the
  income identity with government (§4.7, §4.8);
- Prop 6's identities, κ, eq 16, D.3's identity, eq 17 and the fiscal-capacity corollary;
- three-taxes' ledger, legs and circular flow at the equilibrium (T1, T6, T5);
- the incidence of each tax as the paper states it: the rent tax (Prop 6 (i)-(ii)), the payroll
  tax (D.2), the consumption tax (D.5, T6), the program (D.1);
- a price-responsive basket: CES over the categories (SSRN eq 26), whose unit-elasticity case is
  check_pinning's and check_dynamics' land-share household (decision 75), as a named
  alternative (R6).

**Out of scope** (§10 says where each goes): a basket per worker type; external finance and net
exports (check_dynamics' household at ρ > 0); decision 64's alternative, which A-joint needs;
government purchases, deficits and debt; taxes on interest, on in-kind plot rents and on the
commons; nonlinear taxes and means-tested withdrawal; a program per type; D.5's index and
check_mix's frontier; the dynamic blocks of three-taxes (T3, T4); a dump interface.

## 2. Decisions

### 2.1 Where each tax is levied

- **The payroll tax τ_w** is a share of the gross wage of every hour sold, pooled and walled.
  The cost system uses the gross wage; a type-i worker keeps (1 − τ_w)·v_i. Its revenue is
  τ_w·W, W the gross wage bill, so it is bounded by w·N_a (SSRN p.31). A tax on the employer's
  side is the same economy with the gross wage renamed: the oracle's v is what producers pay.
- **The consumption tax t_c** is levied on every final purchase at its producer value: the
  categories and direct space bought by workers, exiters and the provider, and the support
  baskets. Intermediate inputs (machine services, categories bought by categories) are not taxed,
  so the cost system is untouched. Home output (1e's exit life) is in kind and untaxed. A
  composite costs a household P^c = (1 + t_c)·P.
- **The rent tax** falls on the market rent in money, R = r·T_m: enclosed land in use by
  production and direct space (1e's T_m). Rents paid in kind by exit plots, the commons' shadow
  rent and interest are not taxed. κ keeps 1e's definition with the whole enclosed base (§4.8,
  decision 115).

Taxing intermediate inputs would change the recursion and the ledger (at TX a consumption tax
of 25% on machine services too makes D(x*) = 1 − 1.25·0.8 = 0: not viable). Proposed decision 103.

### 2.2 Transfers are in composites at consumer prices

The uniform transfer (in the RentRate closure) and the program's payments are registered in
composites, d̂, μ_w and μ_e, and paid in money as d = d̂·P^c, m_w = μ_w·P^c, m_e = μ_e·P^c, so no
instrument is a currency amount (R14) and "funding exactly one basket per person requires
d = P_s" (SSRN p.16) reads d̂ = 1 at t_c = 0. In the Dividend closure d is money, the budget's
residual, and is reported with d/P^c. Proposed decision 104.

### 2.3 One participation rule with government

SSRN eq 8 with support, the exit life (1e §2.3) and the government. A type-i person has
unearned resources A_i in both states, the program's m_w or m_e, the net wage at work and the
home output ê_i = (1 + t_c)·e_i in exit, valued at the consumer price of the goods it replaces:

    u_1 = ln((A_i + m_w + (1 − τ_w)·v_i)/P^c) − χ,     u_0 = ln((A_i + m_e + ê_i)/P^c)
    n_S,i = N_i·F_i(ln1p(num_i/den_i)),
    num_i = (m_w − m_e) + ((1 − τ_w)·v_i − ê_i),    den_i = (A_i + m_e) + ê_i

and the reservation wage is v_i = [(e^χ − 1)·den_i + ê_i − (m_w − m_e)]/(1 − τ_w). Every
source's form is a case: SSRN eq 9 (A = P_s, the rest 0), p.16 and main.tex:552 (A = P_s + d),
eq 27 (A = P_s, m_w and m_e), 1d (A = ν_i·P_s) and 1e (e_i > 0). A proportional rescaling of
prices leaves the choice unchanged (SSRN p.11), and so does the consumption tax's factor where
it multiplies every term: dividing num and den by 1 + t_c leaves the net wage
κ_w·v_i, κ_w = (1 − τ_w)/(1 + t_c), against producer-priced composites (§4.10 (f)).

### 2.4 Supplement or replace

Each economy names a transfer mode:
- **Supplement** (default): A_i = ν_i·P^c + d. The provider keeps its support ν_i and the
  transfer adds to it: SSRN p.16's reservation wage (e^χ − 1)(P_s + d).
- **Replace**: A_i = max(ν_i·P^c, d). The provider tops each person up to ν_i composites, and
  pays (ν_i·P^c − d)⁺; a transfer above the support supplements by the excess.

In both A_i = s_i + d, s_i the provider's support per person. The program always adds to the
supported basket (D.1: "in addition to the supported basket"). Proposed decision 106.

### 2.5 The budget closes by the owners' levy, or by the uniform transfer

The budget balances at every equilibrium: G_w + G_R + G_c = N·d + M, with G_w = τ_w·W,
G_c = t_c·C (C = Y·P, the producer value of consumption) and M the program's cost. One
instrument is the residual:
- **RentRate** (default): the transfers d̂, μ_w, μ_e and the rates τ_w, t_c are given; the
  owners pay the levy L_R = N·d + M − G_w − G_c, and the rent-tax rate is τ_R = L_R/R, reported
  (`None` where R = 0). Eq 16 is this closure's output at d̂ = 1, and the poor rate is its
  historical form: relief set in baskets, the rate levied to cover it. A levy outside [0, R]
  is reported, not refused: `within_rent` is false, and the provider's own consumption, the
  residual, can turn negative (`funded` false, as in 1a).
- **Dividend** (alternative, R6): every rate is given, τ_R among them, and the uniform transfer
  is the residual, d = (G_w + τ_R·R + G_c − M)/N. It is A.1's "returned as a uniform transfer
  d = τ_R·R/N" and D.2's wage-funded transfer.

Under RentRate no transfer depends on the equilibrium, so a person's comparison sees only
prices and given composites, and the owners' levy enters no allocation: the owners bear it,
whatever it funds. This is why each tax's incidence is the paper's there (§4.10), and why the
closure needs no inner fixed point with any feature of 1a-1e. Proposed decision 105.

### 2.6 The Dividend closure only where the budget does not depend on who works

Under Dividend, d depends on the point's revenue, and in Supplement mode (or Replace above the
kink) supply depends on d. That is a loop unless the budget at a point is fixed by the demand
side alone. It is when there are no reserved hours (a walled wage is set by its own supply), no
plot-taking type (1e's plots take land from production through supply) and no conditional
program (M depends on who works); the uniform program (μ_w = μ_e) cancels against d. 1f
validates exactly that. W, R and C are then the point's demand-side values, W = v·n_D,
R = r·T_m, C = Y·P, and at the equilibrium they equal the supply side's to the labour net. An
inner fixed point in d for the other economies is open question 5. Proposed decision 107.

### 2.7 The provider, and who is N

The provider of 1a-1e owns the land and the machines (rent and interest), pays each type-i
person s_i in both states, and consumes the rest; it now pays the levy or the rent tax too.
N = Σ_i N_i counts the potential workers, as in Prop 6 and κ (Appendix B's N·P_s = 5.44630): the
provider receives no transfer (it is "one nonworking supporting household", SSRN p.28, not
persons in N). Its own consumption in composites is the residual of Walras, and `funded` is its
strict positivity, 1a's flag.

### 2.8 Walled types under a government

A type at its wall (1d §4.3) sells all its hours to its reserved market, at the wage that makes
its supply equal its reserved demand D_i. With government that condition is
num_i/den_i = ζ_i = expm1(χ_max,i·D_i/N_i), so its wage is still a multiple of P:

    v_i = c_i·P,   c_i = ((â_i + μ_e)·ζ_i − (μ_w − μ_e))·(1 + t_c)/(1 − τ_w)

with â_i = A_i/P^c its unearned composites. 1d's walk runs unchanged with c_i for ζ_i·ν_i. It
needs â_i constant along the path, which holds under RentRate (Dividend excludes reserved hours,
§2.6), and c_i > 0 wherever ζ_i > 0, which holds when μ_w ≤ μ_e. An in-work benefit with reserved
hours can make a type supply its reserved hours at no wage, a market 1d's walk does not have:
1f validates μ_w ≤ μ_e in an economy with reserved hours. Open question 6. Proposed decision 108.

### 2.9 The start of the path is evaluated

1a-1e take the path's start (v → 0 at the all-human corner) as positive, since every supply
vanishes at a zero wage. An in-work benefit gives positive supply at a zero wage, N_i·F_i(ln1p(
(μ_w − μ_e)/(â_i + μ_e))). So 1f evaluates the start, f_0 = n_D(x = 0) − S(v = 0), and gives it its
side (positive when > 0). Without an in-work benefit S(0) = 0 and f_0 = n_D(0) > 0 (1d's rule
that the basket needs pool work at x = 0), so the start's side is 1a-1e's and no count moves. A
negative start with no change of side along the path is `SolveError::SurplusLabour { f_start }`:
the benefit alone draws more people than the economy employs at any positive wage. Proposed
decision 109.

### 2.10 The price-responsive basket: CES over the categories

`Basket::Ces { sigma }` is a CES aggregate of the categories with the fixed basket's weights z_j
(1b's `weight`) and one elasticity σ > 0. With Z = Σ_j z_j and s̄_j = z_j/Z,

    M = (Σ_j s̄_j·p_j^(1−σ))^(1/(1−σ))    (σ ≠ 1),   M = Π_j p_j^(s̄_j)   (σ = 1)
    P = Z·M,   c_j = z_j·(p_j/M)^(−σ)      composites' content: c_j units of category j

so one composite is the basket z wherever the prices are equal, Euler's P = Σ_j c_j·p_j and
Shephard's c_j = ∂P/∂p_j hold, σ → 0 is the fixed basket, and the expenditure shares are
s_j = z_j·p_j^(1−σ)/Σ_k z_k·p_k^(1−σ). With two categories, a good g and direct space, and weights
z = ((1 − α)^σ, α^σ), space's share is SSRN eq 26's α(q) with q = r/p_g. At σ = 1 with Z = 1 the
index is P = Π_j p_j^(z_j), check_dynamics' P = p^(1−α)·r^α (:405). Every household has this
basket, so the composition of demand depends on prices, not on who holds the income, and eq 11
holds with the basket at the point's prices: Y = T_m/B_s(c(p)), n_D = Y·L_s(c(p)). The support ν_i
is in composites. `Basket::Fixed` stays the default (decision 59), and the historical runs keep
it. Proposed decision 110.

### 2.11 CES only with the dependence form and without reserved hours

With reserved hours the walk's P would be a CES of prices that contain the walled wages, and the
reserved demand D_i would move with the basket the walk sets: a fixed point the walk does not
solve. With a plot-taking type 1e's certification uses the basket's land content at x = 0, which
CES moves. 1f validates that a CES economy has every type without exit values (the dependence
form, or the priced form with s₀ = s̲ = 0) and no reserved hours. Open question 9. Proposed
decision 111.

### 2.12 Free categories under CES: no idle stretch

On the wall's last piece v/r → ∞, and a category that embodies no labour there (direct space,
any land-only good) becomes free relative to the rest. A CES household (σ > 0) then buys
unboundedly much of it per composite, B_s → ∞, and n_D → 0: the excess demand tends to −S_∞ and
land is never idle. So with a weighted category whose labour total is 0 under the wall's last
technique, f_∞ = −S_∞, the idle stretch is empty, and the equilibrium lies on the path at r = 1.
W3 (1d's `LaborShort`, 1e's idle land) is a wall equilibrium with land scarce under CES (CW). A
CES economy whose every weighted category embodies labour has 1e's idle stretch, with the basket
at the wage-unit prices of r = 0, fixed along it. Proposed decision 112.

### 2.13 The land-share household, and decision 75

check_pinning's A-joint and check_dynamics' household spend a share α of income on land
services and the rest on the good, with full participation. That is `Basket::Ces { sigma: 1 }`
with z = (1 − α, α) over (good, space) and saturated supply (χ_max small): the prototype's CES
closure reproduces A-joint's root to the digits check_pinning prints (x* 0.87524104049973185629
against 0.8752410404997317; §7). But the oracle cannot solve A-joint: 1 − a − λ·γ(1) =
1 − 0.5 − 0.1·5 = 0 exactly, in decimals and in f64, so decision 64 makes it `NotViable`.
`generate_1f.py` reproduces A-joint as a check of its own household; the oracle's goldens use
viable neighbours (AJ1, AJW). check_dynamics' two targets need, beyond this household, machines
financed abroad with a net-export term (:14-21), which changes what households receive and
buy, and for the flat target a flat schedule, which 1a's schedule does not admit; the sloped
target is A-joint with a build recipe, also `NotViable`. Decision 75's expectation that 1f makes
them reproducible is therefore met only for the household: open question 11, which binds Phase 3.
Proposed decision 113.

### 2.14 Bitwise nesting through a normative evaluation order

As 1b-1e (decision 63). With `Government::none()` (every rate 0, the RentRate closure with
d̂ = 0, no program, Supplement) and `Basket::Fixed`, every new operation of §5.1 is an exact no-op:
P^c = (1 + 0.0)·P, d = 0.0·P^c, A_i = ν_i·P^c + 0.0, m = 0.0·P^c, (A_i + 0.0) + 1.0·e_i,
(0.0 − 0.0) + (1.0·v_i − 1.0·e_i), c_i = ((ν_i + 0.0)·ζ_i − 0.0)·1.0/1.0, the corner ratio
(0.0 + (1.0·(ε_i·ω))/1.0)/(ν_i + 0.0), and the start value f_0 = n_D(0) > 0, whose side 1e assumed.
The government's accounts are computed only in the report. So `HouseholdEconomy::solve()` of
`HouseholdParams::from_parcels(p)` is `ParcelEconomy::solve()` of p bit for bit: every
equilibrium, error and count (§9 gives the argument operation by operation). The Dividend
closure with every rate 0 is the same, since its corner form's d/P^c is 0.0 then.

## 3. Parameters, validation and instances

### 3.1 Parameters

1e's parameters stay (unit-1e.md §3.1). New:

| symbol | name in code | meaning |
|---|---|---|
| — | `basket` | `Basket::Fixed` (1b's z; decision 59) or `Basket::Ces { sigma }` over the categories, weights z |
| σ | `basket.sigma` | elasticity of substitution between categories, in [`SCALE_FLOOR`, `SIGMA_CEIL`] |
| τ_w | `government.payroll` | payroll tax, share of the gross wage; [0, 1) |
| t_c | `government.consumption` | uniform tax on final purchases at producer value; [0, `SCALE_CEIL`] |
| — | `government.budget` | `Budget::RentRate { dividend: d̂ }` (default) or `Budget::Dividend { rent_tax: τ_R }` |
| d̂ | `budget.dividend` | the uniform transfer per person, composites at consumer prices; 0 or scale |
| τ_R | `budget.rent_tax` | the rent tax, share of market rent; [0, 1] |
| μ_w, μ_e | `government.program.work`, `.exit` | the program's payments per person in work and in exit, composites at consumer prices; 0 or scale |
| — | `government.mode` | `TransferMode::Supplement` (default) or `Replace` |

`Government::none()` is every rate 0, `RentRate { dividend: 0 }`, program (0, 0), Supplement.
`HouseholdParams::from_parcels(ParcelParams)` is 1e's economy with `Basket::Fixed` and
`Government::none()`.

Derived at construction: 1 + t_c, 1 − τ_w, κ_w = (1 − τ_w)/(1 + t_c), Δμ = μ_w − μ_e; under CES,
Z and s̄_j; the scan flag (§5.3); whether a weighted category is free at the wall's end (§2.12).

**Notation.** P is the composite's producer price (1a-1e's P_s under the fixed basket); P^c the
consumer price; c_j the composite's content (z_j under the fixed basket); W the gross wage bill,
R = r·T_m market rent, C = Y·P consumption at producer value, I = W + R + interest (App. C);
A_i a type-i person's unearned resources, â_i = A_i/P^c; s_i the provider's support per person;
d the uniform transfer in money; M = m_w·Σ_i n_S,i + m_e·Σ_i E_i the program's cost; G_w, G_R,
G_c the three taxes' revenue; L_R the owners' levy (RentRate); ω = v/P; ω_net = κ_w·v/P = the
net wage in producer-priced composites. 1e's symbols keep their meanings.

### 3.2 Validation

1e's checks (unit-1e.md §3.2 and §12 item 2) in 1e's order, then:
- `basket`: under CES, σ finite in [`SCALE_FLOOR`, `SIGMA_CEIL`] (`Invalid { name: "sigma" }`);
  every type without exit values and no reserved hours (`Invalid { name: "basket" }`, §2.11);
- `government`: τ_w finite in [0, 1); t_c finite in [0, `SCALE_CEIL`]; d̂, μ_w, μ_e each 0 or in
  [`SCALE_FLOOR`, `SCALE_CEIL`], finite; τ_R finite in [0, 1]; −0.0 stored as +0.0; errors
  `Item { kind: "government", name }` with the field's name;
- `Budget::Dividend`: no reserved hours, no plot-taking type (1e's 𝒫 empty) and μ_w = μ_e
  (`Invalid { name: "budget" }`, §2.6);
- reserved hours with a government: μ_w ≤ μ_e (`Invalid { name: "program" }`, §2.8).

`SIGMA_CEIL` is proposed at 64: the CES is evaluated in logs (§5.1), and 64·ln(2^16) ≈ 710 is
about where exp overflows, so a category's content stays finite for price ratios up to about
2^16; beyond, it is below 2^-1022 or above 2^1023 of another's and the point is a limit
(§2.12). The paper's σ are 0.5-2.

With `Government::none()` and `Basket::Fixed` these are 1e's rules, and 1e's rejected rows are
rejected with 1e's errors.

### 3.3 Instances

All are constructed on 2026-09-27. Scalars not named are 1a's G1 (N 4, T 10, h 1, a 0.3,
λ 0.05, b 0.4, γ = 0.2 + 0.8x, χ_max 1, ν 1, (ρ, δ, J_b) = (0, 1, 1)) in Appendix B's form (the
good and space). Governments are written with the fields that are not 0; RentRate and
Supplement unless named.

**H0, 1a to 1e in household form.** `HouseholdParams::from_parcels(ParcelParams)`: every 1e
golden instance, random draw, regime row and rejected row goes through it (§8, f1).

**TX, three-taxes' worked instance inside the closure.** N 4, T 8, h 1, a 0.5, λ 0.1, b 0.2,
γ = 2 + 2x (η 1, g0 2, g1 2), χ_max 1/8, ν 1/4. Labour demand is T·L_s/B_s = T/2 = 4 = N at
x = 1/2 and supply saturates, so x* = 1/2 in exact arithmetic, γ(x*) = 3, and the price block is
check_three_taxes' instance (0.5, 0.1, 3, 0.2, 1): p_m = 1, v = 3, λ̃_m = 0.2, b̃_m = 0.4,
φ_w = 0.6, φ_r = 0.4. The good costs 2.75 and the basket P = 3.75 with L_s = 0.75 and
B_s = 1.5, so the basket's own ledger is (0.6, 0.4) as well: v·L_s = 2.25 = 0.6·3.75. Y = 16/3,
N_a = 4, W = 12, R = 8, I = 20, κ = 8/15, and the provider funds its support
(ν·N·P = 3.75 < 8). Saturation holds on the grid τ_w ∈ {0, 1/8, 1/4, 1/2}, t_c ∈ {0, 1/4},
d̂ ∈ {0, 1} (the least margin, ln(1 + 0.256) = 0.228 against χ_max = 0.125), so every tax on it
leaves the allocation where it is.
- **TX1**: Dividend, t_c 1/4, τ_R 0: G_c = 5, d = 5/4: the legs.
- **TX2**: Dividend, τ_R 1: d = 2, N·d = R: T5 and Prop 6 (iii).
- **TX3**: λ 0, T 12: x* = 1/2 again, p_m = b/(1 − a) = 0.4, v = 1.2 (1a's G6 at λ = 0, a 60%
  cut), P = 2.1: T6's corner.
- **TXd**: TX with d̂ 1 (RentRate): τ_R = N·P/R = 15/8, outside the rent (κ < 1).

**G, Appendix B's economy with a government.**

| instance | government | what it shows |
|---|---|---|
| GA | d̂ 1, Replace | G1 bit for bit; τ_R = N·P/R = 1/κ (eq 16) |
| GB | d̂ 1 | the supplementing transfer: reservation wage (e^χ − 1)(P + d), participation falls, x* and v rise |
| GC | Dividend, τ_R 0.5 | d = τ_R·R/N = 5/4; Prop 6 (iii) |
| GR | Dividend, τ_R 0.1, 0.25, 0.375, Replace | d ≤ ν·P^c along the whole path: G1 bit for bit |
| GP | τ_w 0.1 | the payroll tax on the line; the incidence share at τ_w → 0 |
| GT | t_c 0.25 | against GT′ (τ_w 0.2 = t_c/(1 + t_c)): the same allocation |
| GW | μ_w 0.2 | an in-work benefit: participation up, the gross wage down |
| GE | μ_e 0.2 | an out-of-work benefit: the reverse |
| GU, GS | μ_w = μ_e = 0.2; d̂ 0.2 | a uniform program is a supplementing transfer, bit for bit |

**W, the corners.**
- **W1** (1d's, λ 0.6): at the wall; τ_w ∈ {0, 0.05, 0.1}: the net real wage does not move.
- **WI**: W1 at τ_w 0.2: the wall's real-wage ceiling (1 − τ_w)/L_s = 14/9 is below the net wage
  the wall needs, so land idles (1e's stretch), with ω_net = 14/9.
- **SL**: 1d's W4 (N 20, χ_max 0.05) with μ_w 0.1: `SurplusLabour` (S(0) = 20 > n_D(0) = 10).

**AP, SSRN's automation path** (App. B p.30): λ 0, γ = η(1 + x), η = 2^-k for
k ∈ {0, 4, 8, 12, 16, 20}, τ_w 1/2 (its revenue to the owners, RentRate d̂ 0). (25) holds
throughout (SSRN p.30).

**K, coverage with parcels** (1e's instances, RentRate d̂ 1, Replace, so each allocation is 1e's
bit for bit): **KR4** and **KR5** (Q4, Q5: every exiter on the floor, T_m = T): τ_R = 1/κ,
outside the rent at Q4 (κ 0.929) and within at Q5 (κ 1.0055). **KR1** (Q1, plots rented):
τ_R = N·P/(r·T_m) > 1/κ, the base difference of §2.1. **KT**: 1e's K3 with t_c 0.25 against
τ_w 0.2: the priced exit keeps the equivalence.

**E, walled types.** **ER**: 1d's E2 (trained at its wall) with τ_w 0.1, t_c 0.1, d̂ 0.5,
μ_e 0.1: the walled wage v_T = c_T·P of §2.8, its supply equal to its reserved demand; and ER′
with t_c 0 and τ_w 1 − 0.9/1.1: the same allocation.

**C, the CES household.**
- **C1, C2, C3**: G1 with `Ces` over (good, space), z = ((1 − α)^σ, α^σ), α 0.3 (decision 66),
  σ 0.5, 1, 2.
- **CP**: C1-C3's baskets on AP's path (no tax): space's share moving toward eq 26's limits.
- **CW**: 1d's W3 (λ 0.6, χ_max 3) with C1's and C3's baskets: a wall equilibrium with land
  scarce, where 1d is `LaborShort` and 1e idles land (§2.12).
- **CI**: W3 with space given one human-required hour per unit (1d's `human_required`), so every
  weighted category embodies labour, under C1's basket; the build tunes N so the equilibrium is
  on the idle stretch and records it.
- **AJ**: A-joint (check_pinning :235-236): `NotViable` with D(1) = 0 (decision 64).
- **AJ1**: A-joint with γ = 1 + 3.9x (D(1) = 0.01): an interior equilibrium.
- **AJW**: A-joint with λ 0.08: a wall equilibrium in closed form, v = 15.

**Random economies** (§8, f9).

**What each instance catches.** Each is built so that a plausible wrong unit misses its goldens
by far more than the gate's tolerance (the prototype's values):

| instance | a unit that … | gets |
|---|---|---|
| TX | taxes intermediate inputs as consumption (t_c 0.25) | D(x*) = 1 − 1.25·(0.5 + 0.3) = 0: not viable |
| TX | pins the net wage at the margin and prices hours gross in the recursion (τ_w 0.25) | p_m = 1.6, φ_w = 0.75, not 1 and 0.6 |
| GB | leaves a supplementing transfer out of the comparison | G1's x* 0.86315, not 0.93746 |
| GA, GR | lets a replacing transfer supplement | GB's or GC's x*, not G1's |
| GP | puts the payroll tax on the gross wage in supply | G1's x* 0.86315, not 0.87627; incidence share 0, not 0.88586 |
| GT | prices the support at producer prices | G1's x* 0.86315, not 0.89016 |
| W1 | the same | v 12.3478 at every τ_w, not 17.3046 (0.05) and 28.9103 (0.1) |
| GW | reads Δ with the wrong sign (the payment in exit) | GE's x* 0.94718, not 0.80373 |
| SL | assumes the start of the path positive | an equilibrium where there is none |
| C1 | freezes the composite at its reference content per good (a fixed basket of one good and (3/7)^½ space) | x* 0.87389 and space share 0.64361, not 0.89176 and 0.52043 |
| CW | keeps 1e's idle stretch under CES | idle land, not the wall at v 29.2675 |
| H0 | changes any 1e operation's order | 1e's goldens off in the last bits (§8, f1) |

## 4. Equations

Per period. x is a threshold on the contestable line, τ a technique, i a worker type, j a
category. Prices are in 1e's numeraire: r = 1 while land is scarce, the pool's wage on the idle
stretch.

### 4.1 The producer side is tax-free

At a point, 1e §5.1 step 1 gives the prices p_j, p_m,k, v, r and each category's totals through
the chain, price-side (λ̃ᵖ_j, b̃ᵖ_j: p_j = v·λ̃ᵖ_j + r·b̃ᵖ_j + Σ_i v_i·R̃_ji, with the user cost in
the machine rows) and quantity-side (λ̃^q_j, b̃^q_j; equal at ρ = 0). Producers pay the gross
wage and the gross rent (Prop 6 (i); main.tex:537), and no final-purchase tax reaches an
intermediate input, so the government enters none of these except through the walled wages
v_i, which are supply prices (§4.5). In an economy without walled types and without plot-taking
types, every price and every quantity at a point is the same with and without a government.

### 4.2 The basket

    Fixed:  c_j = z_j,   P = Σ_j z_j·p_j                                  1b; SSRN eq 7
    CES:    Z = Σ_j z_j,  s̄_j = z_j/Z
            ln M = (1/(1 − σ))·ln Σ_j s̄_j·p_j^(1−σ)   (σ ≠ 1),   ln M = Σ_j s̄_j·ln p_j   (σ = 1)
            P = Z·M,   c_j = z_j·exp(−σ·(ln p_j − ln M))                  §2.10; SSRN eq 26

Then, as 1b-1e do with z: the basket's outputs through the chain ŷ = (I − A_cc)⁻¹c, and its
totals L_s, B_s (quantity side, with 1d's tail and reserved hours) and Lᵖ_s, Bᵖ_s (price side),
P = v·Lᵖ_s + r·Bᵖ_s + Σ_i v_i·R_ŷi. The identities the CES satisfies at every point: Euler,
Σ_j c_j·p_j = P; Shephard, c_j = ∂P/∂p_j; the shares s_j = c_j·p_j/P =
z_j·p_j^(1−σ)/Σ_k z_k·p_k^(1−σ); at equal prices c = z; with (good, space) and
z = ((1 − α)^σ, α^σ), space's share is α(q), q = r/p_g (SSRN eq 26; main.tex:807-810).

### 4.3 Consumer prices and unearned resources

    P^c = (1 + t_c)·P
    d   = d̂·P^c                                                        RentRate
    d   = (τ_w·W + τ_R·R + t_c·C − μ·P^c·N)/N,  W = v·n_D, R = r·T_m, C = Y·P    Dividend (μ = μ_w = μ_e)
    m_w = μ_w·P^c,   m_e = μ_e·P^c
    A_i = ν_i·P^c + d          (Supplement),      A_i = max(ν_i·P^c, d)   (Replace)
    s_i = ν_i·P^c              (Supplement),      s_i = (ν_i·P^c − d)⁺    (Replace)
    ê_i = (1 + t_c)·e_i        (e_i = p_g·s_i(q), 1e §2.3; 0 in the dependence form)

In the Dividend closure the point's d uses the demand side (§2.6), so it is known before supply.
At a corner of a fixed-basket economy without exit values, d/P^c is written in the corner's
parameter ω = v/P, with r = 1 and P = r·Bᵖ_s/(1 − ω·Lᵖ_s) there:

    δ(ω) = d/P^c = (τ_w·ω·n_D + τ_R·T_m·(1 − ω·Lᵖ_s)/Bᵖ_s + t_c·Y)/(N·(1 + t_c)) − μ

(on the idle stretch, r = 0, the rent term is 0 and ω = 1/P). δ is d̂ in the RentRate closure.

### 4.4 Participation

The point form, for every type and both exit forms (§2.3):

    num_i = (m_w − m_e) + ((1 − τ_w)·v_i − ê_i)
    den_i = (A_i + m_e) + ê_i
    n_S,i = N_i·F_i(ln1p(num_i/den_i))                                   SSRN eq 8, 10 with government

The corner form, for fixed-basket economies without exit values (1d's `supply_at`), with
â_i = ν_i + δ (Supplement) or max(ν_i, δ) (Replace):

    ratio_i(ω) = ((μ_w − μ_e) + ((1 − τ_w)·(ε_i·ω))/(1 + t_c)) / (â_i + μ_e)
    n_S,i = N_i·F_i(ln1p(ratio_i))

Type i is walled where ratio_i < ζ_i (1d §4.5's test with the new ratio). The two forms agree in
exact arithmetic: num_i/den_i = ratio_i when e_i = 0 and v_i = ε_i·v.

The reservation wage of a person with work cost χ (the wage at which it is indifferent):

    (1 − τ_w)·v_i = (e^χ − 1)·(A_i + m_e + ê_i) + ê_i − (m_w − m_e)

which is SSRN eq 9 (A = P_s), p.16 ((e^χ − 1)(P_s + d)), eq 27 ((e^χ − 1)(P_s + m_e) − Δ) and 1e's
priced form (e_i > 0) as cases.

### 4.5 Walled types

1d §4.3 with the new supply: ζ_i = expm1(χ_max,i·D_i/N_i), D_i = Y·R_ŷi, and at its wall

    v_i = c_i·P,   c_i = ((â_i + μ_e)·ζ_i − (μ_w − μ_e))·(1 + t_c)/(1 − τ_w)

1d's walk, P = P⁰ + Σ_i max(ε_i·v·R_ŷi, c_i·R_ŷi·P), unchanged with this c_i. RentRate only, and
μ_w ≤ μ_e (§2.8), so c_i > 0 wherever ζ_i > 0.

### 4.6 The budget

    G_w = τ_w·W,   W = v·n_D + Σ_i v_i·D_i                        gross wages, demand side
    G_c = t_c·C,   C = Y·P                                         final purchases at producer value
    M   = m_w·Σ_i n_S,i + m_e·Σ_i E_i,   E_i = N_i − n_S,i
    RentRate:   L_R = N·d + M − G_w − G_c,   τ_R = L_R/R  (R > 0; else none),   within_rent = 0 ≤ L_R ≤ R
    Dividend:   G_R = τ_R·R,   d = (G_w + G_R + G_c − M)/N  (M = μ·P^c·N here)

The consumption tax's legs, by the income it is paid from (App. C): G_c = t_c·W + t_c·R +
t_c·interest.

### 4.7 The accounts

In money at consumer prices; a composite costs P^c.
- **Type i**: n_S,i workers each spend A_i + m_w + (1 − τ_w)·v_i, E_i exiters each A_i + m_e (their
  home output and the plots' rent in kind stay in 1e's home account). Composites: the spending
  over P^c.
- **The provider**: receives R − L_R (RentRate) or (1 − τ_R)·R (Dividend), plus interest; pays
  the support Σ_i N_i·s_i; spends the rest. Its composites are that over P^c, and `funded` is
  their strict positivity.
- **The government**: G_w + G_R + G_c (G_R = L_R under RentRate) against N·d + M.

### 4.8 Identities at the equilibrium

1e §4.8's identities hold unchanged with the basket at the equilibrium's prices, and:
- **Income with government.** (I1) I = W + R + interest = Σ_j p_j·f_j = Y·P (SSRN App. C; 1e's).
  (I2) The budget: G_w + G_R + G_c = N·d + M. (I3) Households spend what they receive:
  Σ_i [n_S,i·(A_i + m_w + (1 − τ_w)·v_i) + E_i·(A_i + m_e)] + provider = (1 + t_c)·Y·P.
  (I4) Walras in composites: every household's composites sum to Y. Given (I1) and (I2), (I3)
  is the statement that the hours households sell, n_S,i, are the hours production uses, W: it
  holds only at an equilibrium.
- **Receipts** (Prop 6 (iii); main.tex:541-545; check_pinning P5; check_interior :162-169):
  (1 − τ_w)·W + (R − G_R) + interest + N·d + M = W + R + interest + G_c, which is
  (1 − τ_R)·R + N·d = R when G_w = G_c = M = 0. At TX2, (1 − 1)·8 + 4·2 = 8.
- **Coverage** (SSRN eq 16, eq 28; 1e): κ = r·T/(N·P), N = Σ_i N_i. The rate that funds one
  composite per person, eq 16, is 1/κ; in the RentRate closure with d̂ = 1, t_c = τ_w = 0, no
  program and T_m = T, τ_R = 1/κ, and within_rent ⟺ κ ≥ 1 ("a tax rate between zero and one can
  meet this budget exactly when κ ≥ 1", SSRN p.16).
- **D.3's identity** (SSRN p.32): at r = 1, κ = T/(N·(Lᵖ_s·v + Bᵖ_s)) with the price-side totals
  (P = v·Lᵖ_s + Bᵖ_s exactly, the user cost inside the totals), and N·Bᵖ_s ≤ T wherever κ ≥ 1.
- **Eq 17** (SSRN p.17; the proof, p.31): where Lᵖ_s = 0 (every task automated and every recipe
  without labour: x = 1 and λ = 0), P = r·Bᵖ_s and κ = T/(N·Bᵖ_s) exactly.
- **The corollary's bound** (SSRN p.31): 0 ≤ W/(N·P) ≤ (v/r)·(N_a/N)/Bᵖ_s ≤ (v/r)/Bᵖ_s at every
  equilibrium with r = 1, since P ≥ r·Bᵖ_s.

### 4.9 Three-taxes at the equilibrium

- **T1, the ledger** (check_three_taxes :27-61). For any good k at u = 1, its price resolves into
  wage and rent claims, φ_w,k = v·λ̃ᵖ_k/p_k and φ_r,k = r·b̃ᵖ_k/p_k, φ_w,k + φ_r,k = 1 (with walled
  types, their reserved claims make a third share). For machine type m this is 1a's
  λγ*/(1 − a) at its margin; for the basket, φ_w,s = v·Lᵖ_s/P. At TX both are (0.6, 0.4).
- **T6, the legs** (:63-85). Per unit of good k at rate t: t·p_k = t·v·λ̃ᵖ_k + t·r·b̃ᵖ_k; at TX1,
  per unit of machine service, 0.15 and 0.10. In aggregate, G_c = t_c·W + t_c·R + t_c·interest;
  at TX1, 5 = 3 + 2. At λ = 0 the machine service is all rent content (TX3: p_m = 0.4), and a
  consumption tax and a rent tax at 30% each take 0.12 per unit of it.
- **T5, the circular flow** (:105-134). Every unit of consumption spending resolves into rent at
  the basket's rate φ^q_r = r·B^q_s/P, since R = r·T_m = r·B^q_s·Y and C = Y·P: R = φ^q_r·C
  exactly, with or without interest. A rent revenue G_R recycled into spending is part of C, so
  with τ̂ = G_R/R and R₀ = φ^q_r·(C − G_R), the rent generated by the rest of spending,
  R = R₀/(1 − φ^q_r·τ̂): T5's fixed point, convergent for τ̂ ≤ 1 since 1 − φ^q_r·τ̂ ≥ 1 − φ^q_r > 0
  where the basket embodies labour (at τ̂ = 1 and ρ = 0 the gap is the basket's wage share, T5
  :112-115). At TX2 (τ_R = 1, φ^q_r = 0.4): R₀ = 4.8 and R = 8 = (5/3)·4.8. T5's "equal baskets
  move nothing" is (c) below: a replacing transfer changes no allocation.

### 4.10 Incidence, as the paper states it

Each is a statement about equilibria of 1f, tested in §8.

- **(a) The producer side does not move** (Prop 6 (i), D.2's last sentence). §4.1: in an economy
  without walled or plot-taking types, the path's prices and quantities are the same at every
  government; only supply moves.
- **(b) The owners' levy moves nothing** (RentRate). L_R enters no allocation, so whatever it
  funds, the owners bear it.
- **(c) A replacing transfer moves nothing** (SSRN p.16). In Replace mode, while d ≤ ν_i·P^c at
  every point the solve evaluates, supply is the same function of prices as without it, so the
  equilibrium is the same bit for bit (GA, GR). The owners' after-tax rent falls by N·d and the
  provider's support by the same: the provider's composites do not change.
- **(d) A supplementing transfer raises the reservation wage** to (e^χ − 1)(P + d) at ν = 1
  (SSRN p.16; main.tex:552), lowers participation, and on the line raises x* and v (SSRN p.12:
  "a lower willingness to work weakly raises x and w/r") (GB, GC).
- **(e) Payroll: workers bear ε_D/(ε_D + ε_S)** (SSRN D.2 p.32; main.tex:829-833). Let transfers
  be fixed in composites (RentRate), and the economy without exit values with pooled types. The
  path gives labour demand as a function of the gross real wage, n_D(ω), with no tax in it (a);
  supply is a function of ω_net = κ_w·ω alone. So at the equilibrium n_D(ω) = S(κ_w·ω), and
  differentiating at τ_w = 0,

        −d ln ω_net/dτ_w = ε_D/(ε_D + ε_S),   ε_D = −d ln n_D/d ln ω (along the path),
                                               ε_S = d ln S/d ln ω_net

  exactly, as a derivative (the prototype: 0.885858418004257 both ways at G1, ε_D 6.5922,
  ε_S 0.84940). The cases: where n_D does not move with ω (a fixed-basket corner under one
  technique: the wall, the all-human corner) ε_D = 0 and ω_net is the same at every τ_w that
  keeps the equilibrium on that piece, the gross wage rising by 1/(1 − τ_w) (W1); where supply is
  saturated ε_S = 0 and the allocation is the same bit for bit, ω_net falling by the whole tax
  (TX); on the idle stretch labour demand is flat at the ceiling ω = 1/L_s and workers bear it
  all (WI). The Dividend closure recycles G_w as d, which adds an income effect in Supplement
  mode: D.2's "circular" fraction, which the formula leaves out.
- **(f) The consumption tax's wage leg is a wage tax** (SSRN D.5; three-taxes T6). In the RentRate
  closure the equilibrium at (τ_w, t_c) is the equilibrium at (1 − κ_w, 0) in every allocation
  (x*, v, P, every n_S,i, Y, T_m, the plots), whatever the exit forms, the mode or the walled
  types: dividing num_i and den_i (and c_i) by 1 + t_c leaves only κ_w, and every transfer is in
  composites. So t_c = t is τ_w = t/(1 + t) (GT, KT, ER). The revenues differ: G_c = t·(W + R +
  interest) against G_w = t·W/(1 + t). The difference is the rent and interest legs, a lump sum
  on their holders, and the tax on transfers' spending.
- **(g) The corner** (T6 :77-85; check_mix M-i). At λ = 0 a unit of machine service is rent
  content only (TX3). Along SSRN's automation path, C/R → 1 at ρ = 0 as the labour share
  vanishes (Prop 5): the consumption tax's base becomes the rent base (AP).
- **(h) Conditionality** (SSRN D.1 eq 27; main.tex:819-825; check_conditionality D-i). The
  marginal worker's reservation wage is (e^χ − 1)(P + m_e) − Δ at ν = 1; an in-work benefit
  (Δ > 0) raises participation and, on the line, lowers x* and the gross wage (GW); an
  out-of-work benefit does the reverse (GE); μ_w = μ_e = μ is the supplementing transfer d̂ = μ,
  bit for bit in the allocation (GU, GS; §5.1).
- **(i) The corollary** (SSRN p.16). Along AP, v/r → 0, R/I → 1, the labour share → 0, the
  payroll revenue per person over P → 0 inside the bound of §4.8, and κ → T/(N·Bᵖ_s), eq 17's
  value, as Lᵖ_s·v → 0.

## 5. Solution method

### 5.1 The evaluation (normative)

At (x, τ) on the line, a corner's (x, v, τ) or ω, or the idle stretch's (T_m, τ_e):
1. **Producer side**: 1e §5.1 step 1 (1d §5.1 steps 1-4) at the point.
2. **Basket**: fixed, c = z (no new operation); CES, §4.2 in logs, the prices scaled by the
   largest when σ < 1 and by the smallest when σ > 1, so that every exponent
   x_j = (1 − σ)·(ln p_j − ln p_scale) is ≤ 0 and S = Σ_j s̄_j·expm1(x_j) lies in (−1, 0]:
   ln M = ln p_scale + ln1p(S)/(1 − σ) where S ≥ −1/2 (σ ≠ 1; the ln1p/expm1 form keeps its
   precision as σ → 1, where S → 0), and ln p_scale + ln(Σ_j s̄_j·exp(x_j))/(1 − σ) below, where
   1 + S would cancel down to the weights of the categories near p_scale and the sum of positive
   terms does not (§14 item 18); Σ_j s̄_j·ln p_j (σ = 1); then c_j, ŷ = chain(c) and the totals,
   as 1b-1e from z.
3. **Consumer side**: P^c = (1 + t_c)·P; under RentRate d = d̂·P^c; m_w = μ_w·P^c, m_e = μ_e·P^c.
4. **Dividend closure only**: quantities first (Y = T_m/B_s, n_D; T_m = T at r = 1, since there
   are no plots), then W, R, C and d (§4.3).
5. **Supply**: A_i and s_i (§4.3), ê_i = (1 + t_c)·e_i, then §4.4's point form in type order, the
   product ν_i·P^c formed first, num_i and den_i evaluated in the order written. In an economy
   with exit values this runs inside 1e's exit sub-problem (1e §5.1 step 2) in place of 1e's
   supply, at each trial plot rent. At a corner of a fixed-basket economy without exit values,
   §4.4's corner form (1d's `supply_at`), with δ(ω).
6. **Walled types**: 1d's walk with §4.5's c_i.
7. **Quantities and f**: 1e §5.1 steps 3-5.

The government's accounts (§4.6-4.8) are computed only in the report.

**Nesting** (§9): with `Government::none()` and a fixed basket, steps 2-4 add no operation or
exact no-ops, step 5 is 1e's supply bit for bit (1e computes (v_i − e_i)/(ν_i·P_s + e_i); here
(0.0 − 0.0) + (1.0·v_i − 1.0·e_i) = v_i − e_i and ((ν_i·(1.0·P) + 0.0) + 0.0) + 1.0·e_i =
ν_i·P + e_i), the corner form is 1d's (ε_i·ω)/ν_i, and step 6's c_i is 1d's ζ_i·ν_i.

**The uniform program and the supplementing transfer.** With (μ, μ) and d̂ = 0, den_i =
(ν_i·P^c + 0.0) + μ·P^c; with (0, 0) and d̂ = μ, den_i = (ν_i·P^c + μ·P^c) + 0.0; both are
ν_i·P^c + μ·P^c, and num_i = 0.0 + ((1 − τ_w)·v_i − ê_i) in both. So GU and GS have the same
allocation bit for bit, with different accounts (M against N·d).

### 5.2 At construction

1e §5.2, then: 1 + t_c, 1 − τ_w, κ_w, μ_w − μ_e; under CES, Z, s̄_j, and whether any weighted
category has labour total 0 under the wall's last technique (§2.12); the scan flag (§5.3).

### 5.3 The solve

1e §5.3, with:
1. **The start.** f_0 = n_D(x = 0) − S(v = 0) at the all-human corner's start, positive when
   > 0, negative otherwise (§2.9). Without an in-work benefit it is n_D(0) > 0, 1e's assumption.
2. **The corners.** A fixed-basket economy without exit values bisects on ω with the corner form
   (1d); any other economy evaluates whole points, a fixed-basket one at each ω, v =
   ω·Bᶻ_s/(1 − ω·Lᶻ_s) with the reference basket z's price-side totals (1e §12 item 7). A CES
   economy bisects its corners on the bit patterns of v itself: its P is not v·Lᶻ_s + Bᶻ_s, so v
   is well conditioned there, while ω near 1/Lᶻ_s resolves v only to about 2^-52·v·Lᶻ_s/Bᶻ_s
   (§14 item 17). A CES point whose Y = T_m/B_s or n_D overflows, at the all-human corner as
   v → 0 where a weighted category free at v = 0 crowds space out of the composite, has labour
   demand beyond every double and is read +∞ (§14 item 16).
3. **The wall's end and the idle stretch.** Under CES with a free weighted category, f_∞ = −S_∞
   (the limit of n_D is 0) and the idle stretch is empty (§2.12); S_∞ is the supply at the limit
   of ω on the last piece, 1/(Z·M(λ̃ᵖ)) for σ < 1 and +∞ (saturated) for σ ≥ 1, M(λ̃ᵖ) the power
   mean of the labour totals there. Otherwise 1e §4.6, with the basket at the idle prices. On the
   idle stretch the RentRate closure keeps 1e's closed form (supply is fixed there); the Dividend
   closure bisects on the bit patterns of T_m, since d, and so supply, move with T_m.
4. **The scan.** 1e's rule (a plot-taking type), and also a CES basket or the Dividend closure
   at ρ > 0, where §5.4 does not reach: `EXIT_SCAN` interior points per piece, as 1e. The root's
   bits never depend on the scan (1e §5.3 step 4).
5. **Errors.** 1e's, and `SolveError::SurplusLabour { f_start }` where f_0 ≤ 0 and no change of
   side follows.
6. **Report**: 1e's report with the basket at the equilibrium's prices (the categories'
   quantities from ŷ = chain(c)), then §4.6-4.9's accounts and ledger, the finiteness check of
   every output, and 1a's labour net.

**Which equation fixes which unknown.** As 1e, and: the budget fixes L_R (RentRate) or d
(Dividend); each household's spending fixes its composites, the provider's being the residual;
the prices fix the basket's composition.

### 5.4 Uniqueness

**Lemma F1 (the CES composition lowers labour demand).** At ρ = 0, a fixed technique and fixed r,
as v rises the composition moves L_s/B_s by

    d(L_s/B_s)|_comp = −σ·d ln v · Σ_j c_j·b̃_j·(ℓ_j − ℓ_s)·(φ(ℓ_j) − φ(ℓ_s)) / B_s ≤ 0

with ℓ_j = λ̃_j/b̃_j (a category with b̃_j = 0 enters as c_j·λ̃_j·(φ_w,j − φ̄_w)/B_s with
φ_w,j = 1), ℓ_s = L_s/B_s and φ(ℓ) = v·ℓ/(v·ℓ + r), a category's wage share. *Proof.*
d ln p_j = φ_w,j·d ln v (dp_j = λ̃_j·dv at fixed technique and r), d ln P = φ̄_w·d ln v with
φ̄_w = Σ_j s_j·φ_w,j = v·L_s/P = φ(ℓ_s), so dc_j = −σ·c_j·(φ_w,j − φ̄_w)·d ln v. Then
d(L/B) = Σ_j dc_j·(λ̃_j·B − b̃_j·L)/B², and λ̃_j·B − b̃_j·L = b̃_j·B·(ℓ_j − ℓ_s); φ is
increasing, so every term of the sum is ≥ 0. ∎ Households buy relatively less of what embodies
relatively more labour, as it gets dearer. At a technique switch the prices, and so the basket,
are continuous, so 1c's argument at switches holds with the basket there.

**Lemma F2 (supply along the path with government).** At ρ = 0:
- Under RentRate, a dependence-form type's supply depends on ω = v/P alone, through
  ratio_i = (Δμ + κ_w·ε_i·ω)/(â_i + μ_e) with â_i constant, increasing in ω; and ω rises along
  the path (d(P/v) = −Bᵖ_s·dv/v², 1e's Lemma 5, which holds for the CES by Shephard). A priced
  type is 1e's Lemma 5 with ε_i → κ_w·ε_i and ν_i → â_i + μ_e when μ_w = μ_e.
- Under Dividend, (A_i + m_e)/v is nonincreasing along the path, so supply is nondecreasing. The
  uniform program cancels against d, leaving ν_i·P^c/v + (G_w + G_R + G_c)/(N·v), and
  G_w/v = τ_w·n_D, G_R/v = τ_R·r·T/v and G_c/v = t_c·T·(L_s/B_s + r/v) (C = Y·P = T·P/B_s with
  P = v·L_s + r·B_s) are each nonincreasing (1c's Lemma 2 and Lemma F1 for L_s/B_s). In Replace
  mode, max(ν_i·P^c, d)/v is nonincreasing for the same reason. On the idle stretch n_D and d both
  rise with T_m, so supply falls with it: f is increasing in T_m, and the stretch has one root. ∎

**Proposition F.** At ρ = 0, an economy without plot-taking types, under either closure and
either basket, has f nonincreasing along the whole path (1c-1e's technique part, Lemma F1 for the
composition, Lemma F2 for supply, 1d's walk for walled types under RentRate). So it has at most
one equilibrium, and 1e's count finds it without a scan. With an in-work benefit the start can be
negative, and then there is none (`SurplusLabour`). With plot-taking types, 1e's Proposition 5
holds with κ_w·ε_i in place of ε_i and â_i + μ_e in place of ν_i when μ_w = μ_e, and 1e's
`certified` is computed so.

The prototype checked the incidence formula of §4.10 (e) by an exact derivative at G1, the
equivalence of (f) at G1 (the two x* equal to 50 digits), the uniform program against the
supplementing transfer (equal), and Lemma F1's consequence on C1-C3 and AP.

### 5.5 Precision

- **The new operations** add one or two roundings each to supply's argument: A_i, den_i and
  num_i carry a few ulps more than 1e's, which moves x* by the supply's share of the excess
  demand's slope (1a §4 step 4); the goldens hold at 1e-12.
- **The levy's rate** τ_R = L_R/R: L_R is a difference of transfers and revenues and cancels when
  they are close; τ_R is reported with the absolute error of its terms, and its residual is
  relative to C, not to L_R.
- **CES in logs**: both forms of ln M (§5.1 step 2) give ln M − ln p_scale to about
  2(n + 2)·2^-53 relative, n the weighted categories: where S ≥ −1/2, 1 + S ≥ 1/2 carries S's
  rounding at most doubled and ln1p loses nothing as σ → 1, where the power mean's 1/(1 − σ)
  would amplify the rounding of the sum; below, the sum of positive terms carries a few ulps and
  its logarithm is at least ln 2 in size. So each c_j carries about (2 + σ·(|ln p_j| + |ln M| +
  2(n + 2)·|ln M − ln p_scale|))·2^-53 relative, the logarithms' own rounding included. The draft
  had ln1p alone, which at σ = 20 with eq 26's weights (space's 0.3^20) cancels 1 + S down to
  1.5e-7 and lost 2.6e-11 in P and 5.2e-10 in the content (CS; §14 item 18).
- **The CES corners**: bisected in v (§5.3 step 2), a corner's root is the adjacent pair of doubles
  of v, so the far wall keeps full precision: with C1's basket on W3 and N 0.001, v 4.3e8, the
  oracle's v is within 1.6e-15 of the generator's and P, whose logarithm is 18, within 3.7e-15
  (CF3). In ω it had lost digits in proportion to v·Lᶻ_s/Bᶻ_s, 3.8e-11 at v 4.3e6, and at 4.3e8
  the pool's residual exceeded the labour net (§14 item 17).
- **Walled wages** carry 1/(1 − τ_w): at τ_w near 1 they amplify their inputs' rounding.
- **The equivalence of (f)** is exact in real numbers; in f64, 1 − t/(1 + t) and 1/(1 + t) differ
  by an ulp, so the two allocations agree to the supply's sensitivity (tested at 1e-12).
- **The incidence share** is a derivative at τ_w = 0, where negative rates are invalid; the test
  takes the one-sided three-point difference (−3·g(0) + 4·g(h) − g(2h))/(2h) of g = ln ω_net over
  the oracle's equilibria at h = 2^-20 (truncation about 1e-11, rounding about 1e-10) against the
  golden at 1e-8.

## 6. Result type

Proposed; the build may rename, and records any departure in §14.

- `src/households.rs`: `Basket { Fixed, Ces { sigma } }`; `TransferMode { Supplement, Replace }`;
  `Budget { RentRate { dividend }, Dividend { rent_tax } }`; `Program { work, exit }`;
  `Government { payroll, consumption, budget, program, mode }` with `Government::none()`;
  `HouseholdParams<S> { economy: ParcelParams<S>, basket, government }` with
  `HouseholdParams::from_parcels(ParcelParams)` (H0); `HouseholdEconomy<S>` with `new`, `at(x)`,
  `at_with(x, τ)`, `at_wage(x, v, τ)`, `at_idle(t_m, τ)`, `at_enclosure(point, side)` (each a
  `HouseholdPoint`: 1e's `ParcelPoint` with the basket, P^c, d, each A_i and supply), `start()`
  (f_0), `solve() -> Result<Regime<Eq1f>, SolveError>` and `solve_scanned(scan)`, and the
  x-free `parcels()` (the 1e economy it wraps). `SIGMA_CEIL` a named constant with its reason.
- `Eq1f`:
  - `base: Eq1e`, the allocation in 1e's form (prices, quantities, supply, the land, the exit,
    κ). Its provider and worker baskets and its `funded` are 1e's accounts without a government,
    kept for comparison and labelled so (decision 116); with `Government::none()` they equal
    1f's.
  - `basket: BasketEq { price, consumer_price, content: Vec<f64>, shares: Vec<f64>, sigma:
    Option<f64> }`.
  - `government: GovernmentEq { payroll, consumption, rent_tax: Option<f64>, levy: Option<f64>,
    dividend, dividend_composites, program_work, program_exit, revenue_payroll, revenue_rent,
    revenue_consumption, consumption_wage_leg, consumption_rent_leg, consumption_interest_leg,
    transfers, program_cost, within_rent: Option<bool>, rate_for_one_composite: Option<f64> }`
    (τ_R given, or L_R/R and `None` at R = 0; the levy under RentRate; 1/κ, `None` at κ = 0).
  - `accounts: Accounts { workers: Vec<TypeAccount { unearned, support, net_wage, working,
    exiting, spending, composites }>, provider: ProviderAccount { receipts, support, spending,
    composites, funded }, spending, composites }`.
  - `ledger: Ledger { basket_wage_share, basket_rent_share: Option<f64> (u = 1), rent_share
    φ^q_r, recirculated_base R₀, multiplier }`.
  - `f_start`, and `residuals: Residuals1f { budget, spending, composites, rent_share, euler }`
    (relative: |G − N·d − M|/C, |Σ spending − (1 + t_c)·C|/((1 + t_c)·C), |Σ composites − Y|/Y,
    |R − φ^q_r·C|/R where R > 0, |Σ c_j·p_j − P|/P; each 0 in H0 form but the first, which is
    exactly 0 there too).
- `Eq1f::outputs()`: 1e's keys in 1e's order, then 1f's economy keys (`basket_price`,
  `consumer_price`, `payroll`, `consumption`, `rent_tax`, `levy`, `dividend`, …), then
  `category<j>.{content, share}` and `worker<i>.{unearned, support, net_wage, spending,
  composites}`.
- `SolveError` gains `SurplusLabour { f_start }`. `ParamError` uses `Item { kind: "government" }`
  and `Invalid { name: "sigma" | "basket" | "budget" | "program" }`.
- No new `core::num` function: `exp`, `ln`, `ln1p`, `expm1` and `fma` are there.

## 7. Goldens

`goldens/generate_1f.py` computes every golden with mpmath at 70 digits from §4's equations. Its
`Economy` extends `generate_1e.py`'s (as 1e's extended 1d's): the basket (fixed and CES, at 70
digits without the log scaling), the consumer side, the participation rule with government, the
walled c_i, the budget in both closures, the accounts, the path's start, the idle stretch under
the Dividend closure by bisection on T_m, and the count by a scan four times the oracle's where
§5.3 scans. It writes `goldens/goldens_1f.txt` with 30 significant digits and a header of FNV-1a
digests of `generate_1f.py`, `generate_1e.py`, `generate_1d.py`, `generate_1c.py`,
`generate_1b.py`, `generate.py` and its own body; the Rust constants in `tests/gate/goldens_1f.rs`
carry 20 digits (f11). Key prefixes: `TX_`, `TX1_` to `TX3_`, `TXD_`, `GA_` to `GU_`, `INC_`,
`W1_`, `WI_`, `AP_`, `KR_`, `KT_`, `ER_`, `C1_` to `C3_`, `CP_`, `CW_`, `CI_`, `AJ1_`, `AJW_`, and
`X_` for the wrong units of §3.3; the verification added `CA_`, `CF2_`, `CF3_` and `CS_` (§14
items 16-18).

The generator asserts as it goes:
- H0 form of every 1e golden instance equals `generate_1e.py`'s solve to 1e-65;
- every identity of §4.8 and §4.9 at 1e-65, and the CES's Euler, Shephard (against a numerical
  derivative) and eq 26;
- §4.10 (e) at G1, W1 and TX: the incidence share by an exact derivative (the equilibria at
  τ_w = ±1e-30) against ε_D/(ε_D + ε_S) from derivatives along the path, to 1e-35;
- §4.10 (f) on GT, KT and ER: the pairs' allocations equal to 1e-65; (c) on GA and GR, and (h)'s
  GU = GS, the same;
- A-joint (check_pinning :235-236, :254-267) under `Ces { sigma: 1 }` with z = (0.7, 0.3) and
  saturated supply: x*, w/p, c/p, r/p, Y and X against check_pinning's printed values to 3e-15
  relative (they come from an 80-halving float bisection); and D(1) = 0 exactly;
- Lemma F1's sign against a finite difference at grid points of C1-C3, and f nonincreasing on a
  grid of every instance at ρ = 0;
- the count equal to each instance's result on the scan.

The prototype's values, to 20 significant digits where it had them (the generator supplies the
rest, and the build records them in §14):

**TX** (exact): x* 1/2, γ* 3, p_m 1, v 3, p 11/4, P 15/4, L_s 3/4, B_s 3/2, Y 16/3, N_a 4, W 12,
R 8, I 20, φ_w 0.6 and φ_r 0.4 for the machine service and for the basket, κ 8/15. **TX1**: G_c 5
= 3 + 2, d 5/4; per unit of machine service 0.15 and 0.10; per basket 0.5625 and 0.375. **TX2**:
d 2, (1 − 1)·8 + 4·2 = 8, φ^q_r 0.4, R₀ 4.8, multiplier 5/3. **TX3**: x* 1/2, p_m 0.4, v 1.2, P 2.1,
Y 8, N_a 4; 0.12 either way at 30%. **TXD**: τ_R 15/8, not within the rent.

**G, Appendix B's economy.**

| | x* | v | P | N_a | other |
|---|---|---|---|---|---|
| G1 | 0.86315041816243703192 | 0.54343596069677832842 | 1.3615758317798092746 | 1.3433818800977173461 | κ 1.83610779631133604; Y 7.8806055249729076768 |
| GA | G1 bit for bit | | | | τ_R 0.54463033271192370986 = 1/κ |
| GB | 0.93746083398747312221 | 0.58235487285097562042 | 1.3668561558660557364 | 0.7724759646953691 | τ_R 0.54674246234642229456, κ 1.82901469863591559 |
| GC | 0.93360426237043449653 | 0.58032633255676976674 | 1.3666475439978276467 | 0.801243054935603 | d 1.25 |
| GP | 0.87627170039731020321 | 0.55028234488275430929 | 1.3627008536384357176 | 1.240030995775408 | τ_w 0.1 |
| GT = GT′ | 0.89015871146432740031 | 0.55754023103552006644 | 1.3638017912497742457 | 1.131835522471573 | ω_net 0.3270505931948342 |
| GW | 0.8037264408356073074 | 0.51256666115635029646 | 1.355454792774879952 | 1.825015776140309 | μ_w 0.2 |
| GE | 0.94717837462152922992 | 0.58747051381971391848 | 1.3673498760635844705 | 0.700409163185895 | μ_e 0.2 |
| GU = GS | 0.88543913357893768838 | 0.55507221628355679682 | 1.3634379975171398545 | 1.168469339971953 | |

**INC** (G1): −d ln ω_net/dτ_w = ε_D/(ε_D + ε_S) = 0.885858418004257, ε_D 6.592219192,
ε_S 0.8493979536.

**W1** (λ 0.6): v 12.347776347728814794 at τ_w 0, 17.304632676489533436 at 0.05,
28.910302576745416974 at 0.1; ω_net 1.6050368175205035052 at all three. **WI** (τ_w 0.2): idle
land, ω_net 14/9 = (1 − 0.2)·35/18.

**AP** (λ 0, γ = η(1 + x), τ_w 1/2):

| η | x* | v/r | labour share | κ | T/(N·B_s) | G_w/(N·P) | bound τ_w·(v/r)/B_s | C/R |
|---|---|---|---|---|---|---|---|---|
| 1 | 0.832997732783 | 1.0474273 | 0.094595044 | 1.351954721 | 1.493204464 | 0.070624871 | 0.31280462 | 1.10447816 |
| 2^-4 | 0.98606048927 | 0.070930732 | 9.3846789e-4 | 2.372889171 | 2.375118143 | 1.1144861e-3 | 0.033693774 | 1.000939349 |
| 2^-8 | 0.99910853518 | 4.4622958e-3 | 3.9647051e-6 | 2.491657399 | 2.491667278 | 4.939363e-6 | 2.2237113e-3 | 1.000003965 |
| 2^-12 | 0.999944201877 | 2.7901007e-4 | 1.5564981e-8 | 2.499476951 | 2.49947699 | 1.9452156e-8 | 1.3947585e-4 | 1.000000016 |
| 2^-16 | 0.999996512298 | 1.7438586e-5 | 6.0819793e-11 | 2.499967303 | 2.499967303 | 7.6023747e-11 | 8.7191788e-6 | 1 |
| 2^-20 | 0.999999782017 | 1.0899134e-6 | 2.3758198e-13 | 2.499997956 | 2.499997956 | 2.9697723e-13 | 5.4495625e-7 | 1 |

κ → T/(N·h) = 2.5, eq 17's T/(N·B_s) as B_s → h.

**C, the CES household** (G1's economy).

| | σ | x* | v | P | N_a | space's share (= eq 26) |
|---|---|---|---|---|---|---|
| C1 | 0.5 | 0.89176202197842091755 | 0.55837897627480756256 | 0.80009907662364586351 | 2.117538830254234 | 0.5204280297630535 |
| C2 | 1 | 0.9006603346572147 | | | | 0.3 |
| C3 | 2 | 0.90960722009951245118 | 0.56772550609720382536 | 0.23495322058734605667 | 4 (saturated) | 0.0628590661500034 |

**CP**: space's share at η = 1, 2^-8, 2^-16: 0.4172529098, 0.9187896979, 0.9945060782 (σ 0.5,
toward 1); 0.3 throughout (σ 1); 0.1330155577, 6.145998e-4, 2.402253e-6 (σ 2, toward 0), with
q = r/p_g 1.196, 298.7, 76459.

**CW** (W3 under CES): the wall with land scarce, v 29.267492270362484658, P 10.598475271797954213,
N_a 1.766417156851008 (σ 0.5); v 5.7935510403550846102, N_a 2.169702843467395 (σ 2).

**AJ, the land-share household.** A-joint in the generator: x* 0.87524104049973185629, w/p
1.5160527572576766324, c/p 0.33682844446030448223, r/p 0.084044732521922889367, goods
1.6495500577338338683, X 7.9420385115351964218 (check_pinning: 0.8752410404997317,
1.5160527572576763, 0.33682844446030447, 0.08404473252192293, 1.6495500577338336,
7.942038511535193). **AJ1** (γ = 1 + 3.9x): x* 0.89631061966178256886, v 17.825984034366138326,
P 5.5653499074688290542, w/p 1.5348440806825329101, r/p 0.08610150652685184878, goods
1.6771014021657359785. **AJW** (λ 0.08): the wall, in closed form: goods 25/12 (= N/λ̃_g,
λ̃_g = 0.08·3/0.5 = 0.48), housing 7.5 (= T − 1.2·25/12), r/p = (3/7)·(25/12)/7.5 = 5/42, p 8.4,
v = (8.4 − 1.2)/0.48 = 15, w/p 25/14.

**X, the wrong units of §3.3**: C1 with the frozen composite, x* 0.8738922168915403 and share
0.6436114755965608 (C3's: 0.9023687185568853 and 0.3349463544089041).

## 8. Tests

The groups are the modules `f1_nesting`, `f2_three_taxes`, `f3_coverage`, `f4_transfers`,
`f5_payroll`, `f6_consumption_tax`, `f7_conditionality`, `f8_ces`, `f9_random_households`,
`f10_regimes_and_validation` and `f11_goldens_file` of `tests/gate/`, cited as `f1::` to `f11::`.
The brief's gate maps onto them:
- three-taxes' ledger inside the full closure: `f2::*`;
- the income identity with government to 1e-12: `check_identities_1f` in every golden test and
  in `f9::identities`;
- Prop 6's identities and κ: `f3::*`, `f4::*`;
- each tax's incidence as the paper states: `f1::the_producer_side_is_tax_free`,
  `f1::replacing_support_moves_nothing`, `f4::*`, `f5::*`, `f6::*`, `f7::*`;
- nesting with no government, exactly: `f1::*`.

Identities and goldens at 1e-12 relative (ADDENDUM A7) unless stated; "bitwise" means `to_bits`
equality. A shared check (`support_1f::check_identities_1f`) runs §4.8-4.9's identities at every
equilibrium, 1e's `check_identities_1e` on `base`, the CES's Euler and shares, the corollary's
bound, and pins every price, P, P^c, d, each A_i and each supply to `at_with`, `at_wage`,
`at_idle` or `at_enclosure` at the equilibrium bit for bit.

**f1, nesting (§2.14, §9).**
- `f1::every_1e_golden_instance_is_bit_identical`: every golden instance of 1a-1e through
  `from_parcels`: every shared output, `bisection_steps`, `scan_points` bit for bit; `f_start`
  equal to n_D at x = 0 and positive; the new residuals 0. [§5.1]
- `f1::random_economies_are_bit_identical`: 1e's e1 and e8 draws (1a's G5, 1b's C5, 1c's m6,
  1d's d7 and 1e's sets): every result, equilibrium or error, bit for bit. [§2.14]
- `f1::refusals_nest`: `NotViable`, `NoMarket`, `MultipleEquilibria` and the rejected rows with
  1e's errors and diagnostics. [§3.2]
- `f1::points_nest_on_a_grid`: `at(x)` equals 1e's `at(x)` bit for bit on 1b's grid. [§5.1]
- `f1::zero_government_in_either_closure`: `Dividend { rent_tax: 0 }` in both modes, and RentRate
  with d̂ 0 in Replace mode, equal `Government::none()` bit for bit on 1e's golden instances
  without plot-taking or walled types. [§2.14; §4.3's δ(ω)]
- `f1::the_producer_side_is_tax_free`: on G1, B1 and 1c's M4 and a grid of x and corner v, the
  prices, totals and quantities of `at_*` are bitwise the same under GP's, GT's, GC's and GW's
  governments. [§4.1, §4.10 (a); Prop 6 (i)]
- `f1::replacing_support_moves_nothing`: GA and GR (τ_R 0.1, 0.25, 0.375): G1 bit for bit; the
  owners' after-tax rent R − N·d and the provider's support N·(P − d) change, its composites do
  not (to 1e-15). [SSRN p.16; §4.10 (c); T5 :118-122]
- `f1::ces_approaches_the_fixed_basket`: G1 and W1 under `Ces { sigma: 2^-30 }` with G1's z:
  within 1e-8 of the fixed basket's equilibrium. [§2.10]

**f2, three-taxes inside the closure (TX).**
- `f2::the_worked_instance_inside_the_closure`: TX: x* within 4 ulps of 1/2, γ(x*) 3, p_m 1, v 3,
  λ̃_m 0.2, b̃_m 0.4, (φ_w, φ_r) = (0.6, 0.4) for the machine service and for the basket, the ledger
  closing (W_u + R_u = p_m), and 1a's `closure(0.5, 0.1, γ(x*), 0.2, 1, 1)` agreeing.
  [check_three_taxes.py:27-61; main.tex:325-336]
- `f2::every_tax_leaves_the_worked_instance`: TX on the grid of §3.3: the allocation bitwise TX's,
  the ledger (0.6, 0.4). [§4.10 (e), saturated]
- `f2::the_legs_of_a_consumption_tax`: TX1: 0.15 and 0.10 per unit of machine service, 0.5625 and
  0.375 per basket, G_c = 5 = 3 + 2, d 5/4. [check_three_taxes.py:63-76; §4.9]
- `f2::the_corner`: TX3: p_m 0.4, v 1.2 (G6's λ = 0 inside the closure), (φ_w, φ_r) = (0, 1) for
  the machine service, 0.12 per unit for a 30% consumption tax and a 30% rent tax. [:77-85]
- `f2::the_circular_flow`: TX2: R₀ 4.8, multiplier 5/3, R = R₀/(1 − φ^q_r·τ_R); and
  R = φ^q_r·C at every golden equilibrium. [check_three_taxes.py:105-117; §4.9]

**f3, Prop 6 and coverage.**
- `f3::receipts`: (1 − τ_R)·R + N·d = R at GC and TX2; §4.8's receipts identity at every golden.
  [SSRN p.16 (iii); main.tex:541-545; check_pinning.py:107-129; check_interior.py:162-169]
- `f3::eq_16`: GA's τ_R = 1/κ = 0.54463033271192370986; TXD's 15/8 with within_rent false; KR4's
  and KR5's τ_R = 1/κ, outside and within the rent; within_rent ⟺ κ ≥ 1 on these and on (a)'s
  economies without rented plots re-solved with d̂ = 1 and no other instrument; KR1's
  τ_R = N·P/(r·T_m) > 1/κ. [SSRN eq 16, p.16]
- `f3::d3_identity`: κ = T/(N·(Lᵖ_s·v + Bᵖ_s)) at every golden with r = 1; N·Bᵖ_s ≤ T where κ ≥ 1.
  [SSRN D.3 p.32; check_feasibility F-iii]
- `f3::eq_17_at_full_automation`: `at(1.0)` of TX3 and AP's economies: Lᵖ_s = 0, P = Bᵖ_s and
  κ = T/(N·Bᵖ_s) exactly; along AP, κ·N·Bᵖ_s/T → 1. [SSRN eq 17, p.31]
- `f3::the_corollary`: AP's goldens; G_w/(N·P) below τ_w·(v/r)/Bᵖ_s and falling, R/I rising to 1,
  the labour share falling to 0, C/R → 1. [SSRN p.16 corollary, p.31; check_mix M-i]

**f4, transfers and the rent tax.**
- `f4::a_supplementing_transfer`: GB's goldens; the marginal worker's reservation wage
  (e^{χ*} − 1)·(P + d) equals v (χ* = N_a/N·χ_max); x*, v rising and N_a falling in d̂ on
  {0, 1/4, 1/2, 1}. [SSRN p.16; main.tex:552; SSRN p.12]
- `f4::the_dividend_closure`: GC's goldens, d = τ_R·R/N; Prop 6 (ii): the transfer is the same in
  both states. [A.1; Prop 6]
- `f4::replace_above_the_support`: G1 with RentRate d̂ 2 in Replace mode equals d̂ 1 in Supplement
  mode (A_i = 2·P^c either way) bit for bit in the allocation. [§2.4]

**f5, payroll incidence.**
- `f5::incidence_on_the_line`: G1: the one-sided three-point difference of ln ω_net at τ_w = 0,
  2^-20 and 2^-19 against INC's share, and ε_D, ε_S from `at` at x* ± 2^-20, at 1e-8. [SSRN D.2
  p.32; main.tex:829-833; §4.10 (e)]
- `f5::payroll_goldens`: GP. [§4.4]
- `f5::borne_by_employers_at_the_wall`: W1's three goldens; ω_net equal within 1e-14 while the
  gross wage v rises. [§4.10 (e), ε_D = 0]
- `f5::onto_idle_land`: WI on idle land with ω_net = 14/9. [§4.10 (e); 1e §4.6]
- `f5::borne_by_workers_when_participation_saturates`: TX at τ_w ∈ {1/8, 1/4, 1/2}: allocation
  bitwise TX's, ω_net = (1 − τ_w)·0.8. [§4.10 (e), ε_S = 0]

**f6, the consumption tax.**
- `f6::the_wage_leg_is_a_wage_tax`: GT against GT′, KT's pair, ER against ER′: x*, v, P, every
  n_S,i, Y, T_m and T_p within 1e-12; G_c − G_w′ equal to t_c·(R + interest) + the tax on the
  transfers' spending. [SSRN D.5; check_three_taxes T6; §4.10 (f)]
- `f6::support_at_consumer_prices`: GT's goldens (x* 0.89016, not G1's). [§2.1, §2.3]
- `f6::home_output_is_untaxed`: KT's pair holds only with ê_i = (1 + t_c)·e_i (a crate-private
  switch that taxes home output breaks it by more than 1e-6). [§2.1]

**f7, conditionality and walled types.**
- `f7::eq_27`: at GW and GE the marginal worker's (e^{χ*} − 1)·(P + m_e) − (m_w − m_e) equals v.
  [SSRN D.1 eq 27 p.32]
- `f7::in_work_benefits_lower_the_wage`: GW and GE goldens; N_a above and below G1's, v below
  and above. [main.tex:819-825; check_conditionality D-i]
- `f7::a_uniform_program_is_a_supplementing_transfer`: GU and GS: the allocation bit for bit,
  the program's cost equal to the transfers'. [D.1; §5.1]
- `f7::surplus_labour`: SL returns `SurplusLabour` with f_start −10. [§2.9]
- `f7::walled_types_under_a_government`: ER: the trained type walled, its supply equal to D_T,
  v_T = c_T·P with §4.5's c_T; the walk's unit tests (order, cascade, ceiling) rerun with c_i in
  place of ζ_i·ν_i. [§4.5; 1d §4.3]

**f8, the CES household.**
- `f8::eq_26_at_the_equilibrium`: C1-C3 goldens; space's expenditure share equal to `ces_share`
  (1b's price block) at the equilibrium's q; Euler and Shephard (a finite difference). [SSRN
  eq 26 p.31; main.tex:807-810]
- `f8::eq_26_along_the_path`: CP: the shares' limits (toward 1, α, 0). [App. C p.31]
- `f8::the_land_share_household`: AJ1 and AJW goldens; P = p^0.7·r^0.3 at AJ1 (check_dynamics
  :405). [check_pinning A-joint; decision 75]
- `f8::a_joint_is_not_viable`: A-joint is `NotViable` with D(1) = 0 exactly. [decision 64; §2.13]
- `f8::no_idle_stretch_with_a_free_category`: CW's goldens: W3 on the wall with land scarce,
  f_end = −S_∞. [§2.12]
- `f8::an_idle_stretch_without_one`: CI on idle land, the basket at the idle prices. [§2.12]
- `f8::composition_lowers_labour_demand`: Lemma F1 against finite differences at grid points of
  C1-C3 on the line and the wall. [§5.4]
- From the verification (§14 items 16-19): `f8::the_start_under_ces` (the evaluated start, and
  `SurplusLabour` under CES), `f8::the_good_free_at_the_all_human_corner` (CA),
  `f8::the_wall_far_out` (CF2, CF3), `f8::a_steep_basket_with_a_small_weight` (CS),
  `f8::the_free_end_with_transfers_and_types` (S_∞ under Dividend with a program, and with two
  efficiencies).

**f9, random economies.** SplitMix64 seeded 951 to 957 (1e's were 941-948), 60 equilibria per set,
the economies from 1e's e8 and 1d's d7 tables:
- (a) RentRate on 1e's sets (a), (b), (c) and (g): τ_w ~ U(0, 0.5), t_c ~ U(0, 0.5),
  d̂ ~ U(0, 1), μ_w and μ_e each U(0, 0.3) with probability 1/2 (μ_w ≤ μ_e forced with reserved
  hours), the mode at random;
- (b) Dividend on the eligible draws: τ_R ~ U(0, 1), τ_w, t_c as (a), a uniform program with
  probability 1/2, the mode at random;
- (c) CES on the eligible draws: σ log-uniform on [0.1, 10], exactly 1 with probability 1/4, with
  and without interest;
- (d) reserved hours (1d's d7 sets with reserved hours, in household form) under RentRate;
- (e) every (a) and (d) draw re-solved at (1 − κ_w, 0): the allocation within 1e-12;
- (f) (b)'s draws in Replace mode with τ_R scaled so that d ≤ min_i ν_i·P^c on a grid of the path:
  bitwise the draw without government;
- (g) in-work benefits μ_w ~ U(0, 1) > μ_e on (a)'s economies without reserved hours.

Tests: `f9::identities` (check_identities_1f at every equilibrium), `f9::residuals_recompute`,
`f9::paired_equivalence` (e), `f9::replace_neutrality` (f), `f9::count_against_a_fine_scan`
(a scan 16 times finer on the first 30 scanned draws of each set), `f9::the_draws_cover_the_regimes`
(the line, the wall, the all-human corner, idle land, a tie, `SurplusLabour`, both closures and
modes, within_rent both ways, κ both sides of 1). The build records the tallies (decision 117).

**f10, regimes and validation.**
- `f10::validation`: every rule of §3.2 with its error, and 1e's rows through `from_parcels`.
- `f10::exact_zeros`: L_R = 0 and L_R = R exactly (within_rent true at both); d = ν·P^c exactly at
  a point in Replace mode (the kink, the same value both ways); μ_w = μ_e with reserved hours
  (allowed); f_0 = 0 exactly (negative: `SurplusLabour`).
- `f10::units`: a proportional rescaling of every price leaves every allocation unchanged (SSRN
  p.11), through 1e's `units` with a government.
- `f10::permutation`: types permuted, the same equilibrium.

**f11, the goldens file.** `goldens_1f.txt`'s digests, and every Rust constant equal to its line
rounded to 20 digits.

## 9. The nesting argument

`HouseholdParams::from_parcels(p)` has `Basket::Fixed` and `Government::none()`: τ_w = t_c = 0,
RentRate with d̂ = 0, μ_w = μ_e = 0, Supplement. Its solve is `ParcelEconomy::new(p).solve()` bit
for bit, because every operation 1f adds to the path is an exact no-op on IEEE doubles and every
other operation is 1e's, in 1e's order:
1. **Validation**: 1e's, then rules that the zero government passes (0 ∈ [0, 1), 0 is "0 or
   scale", RentRate has no restriction, μ_w ≤ μ_e); the fixed basket adds none.
2. **Producer side** (§5.1 step 1) is 1e's code.
3. **Basket**: c = z, the same array; no operation.
4. **Consumer side**: P^c = (1.0 + 0.0)·P = 1.0·P = P; d = 0.0·P^c = 0.0 (P finite and
   positive); m_w = m_e = 0.0.
5. **Supply** (point form): A_i = ν_i·P + 0.0 = ν_i·P (a positive double); den_i =
   (ν_i·P + 0.0) + 1.0·e_i = ν_i·P + e_i, 1e's denominator with the product formed first;
   num_i = (0.0 − 0.0) + (1.0·v_i − 1.0·e_i) = 0.0 + (v_i − e_i) = v_i − e_i (v_i − e_i is
   never −0.0, since v_i ≥ 0 and e_i ≥ 0 and x − x = +0.0); ln1p and F_i are 1e's calls. The
   corner form: ((0.0 − 0.0) + ((1.0 − 0.0)·(ε_i·ω))/(1.0 + 0.0))/((ν_i + 0.0) + 0.0) =
   (ε_i·ω)/ν_i, 1d's. The walled-type test compares the same ratio with the same ζ_i.
6. **Walled wages**: c_i = ((ν_i + 0.0)·ζ_i − (0.0 − 0.0))·(1.0 + 0.0)/(1.0 − 0.0) = ν_i·ζ_i,
   which is 1d's ζ_i·ν_i (multiplication commutes exactly); the walk is 1d's code.
7. **The start**: f_0 = n_D(0) − S(0) with S(0) = Σ ε_i·N_i·F_i(ln1p(0.0/…)) = 0 (and 0 for a
   priced type, whose num is −e_i ≤ 0), so f_0 = n_D(0) > 0 (1d's and 1e's validation), positive:
   the side 1e assumed at the start, so no count or location changes.
8. **The scan**: the zero government adds no scan (the Dividend closure and CES are absent).
9. **The idle stretch**: RentRate keeps 1e's closed form.
10. **The report**: `base` is 1e's report on the same equilibrium; 1f's fields are new and change
    nothing in `base`.

The Dividend closure with every rate 0 is the same on the economies it admits: d =
(0.0·W + 0.0·R + 0.0·C − 0.0)/N = 0.0 at points, δ(ω) = 0.0 − 0.0 at corners (every term finite).
Replace with d̂ = 0 gives max(ν_i·P, 0.0) = ν_i·P. So `f1::zero_government_in_either_closure`
holds too. The chain continues downward through 1e's nesting (1e §2.12), so 1a's Appendix B
economy in household form is 1a's equilibrium bit for bit.

## 10. Out of scope, and where each goes

- **A basket per worker type** (A.1's "living requirements"). Every household buys the one basket,
  so aggregate demand does not depend on who holds income, which eq 11 and every lemma here need.
  Per-class baskets make demand's composition a function of the income distribution, a fixed
  point at each point of the path, and cost the uniqueness argument. PLAN §3.2 has owners buying
  human-required services ("owners' demand is what priced the wall"), which needs it. A 1f
  addendum before Phase 2's 1750-like instance (decision 114; open question 12).
- **External finance and net exports** (check_dynamics :14-21), which check_dynamics' ρ > 0
  targets need beside the land-share household. A Phase 3 addendum (decision 113).
- **Decision 64's alternative** (viability over the range in use), which A-joint and
  check_dynamics' sloped target need; and **flat schedules** (check_dynamics' flat target).
  Rulings on 1b's and 1a's conventions, not 1f's.
- **Government purchases, deficits and debt.** PLAN §5 scripts wars as government purchases, and
  Phase 8 brings debt. A 1f addendum when the first tape needs them (decision 118).
- **Taxes on interest, on in-kind plot rents and on the commons' use.** The rent base is market
  rent in money (§2.1); a tax in kind on plots would enter the exit sub-problem (decision 115).
- **Nonlinear taxes, withdrawal rates and means tests; a program per type.** The program is two
  lump sums per person, as in A.1 and D.1. The poor law's workhouse test and a type-specific
  relief scale are Phase 6's.
- **D.5's index** D = φ_C(1 − κ)² **and check_mix's frontier.** Descriptive, with an assumed
  quadratic cost; the oracle reports κ, the legs and the accounts, from which an analysis computes
  them.
- **three-taxes T3, T4, T7-T12** (the ceiling's hold-up, stock and flow, the political blocks):
  dynamic or outside the static equilibrium.
- **A dump interface** for 1f.

## 11. Pitfalls

- **Taxing intermediate inputs.** A consumption tax on machine services or on categories bought by
  categories changes the recursion: TX becomes not viable at t_c = 0.25. Only final purchases.
- **The payroll tax in supply at the gross wage, or in the recursion at the net.** The first
  leaves every equilibrium unmoved (GP = G1); the second pins the net wage at the margin
  (TX: φ_w 0.75).
- **The support at producer prices.** The provider buys the support at P^c; priced at P, the
  consumption tax moves nothing (GT = G1).
- **Home output taxed.** ê_i = (1 + t_c)·e_i: home output replaces purchases at consumer prices.
  With ê_i = e_i the equivalence of §4.10 (f) fails with priced exit (KT).
- **The Dividend's d from supply hours.** At the equilibrium it is the same, but off it the
  point's d would depend on supply: a loop, and in the corner form a different function of ω.
- **N counting the provider.** d goes to Σ_i N_i persons; the provider is not in N.
- **Replace read as "the transfer is the support".** A_i = max(ν_i·P^c, d), not d.
- **κ with P^c, or with market rent.** κ is 1e's, r·T/(N·P) at producer prices over the whole
  enclosed base; eq 16's τ_R equals 1/κ only with T_m = T and t_c = 0.
- **The CES normalization.** P = Z·M, so that one composite is the basket z at equal prices; with
  M alone the support ν_i changes meaning with Z. σ = 1 is its own branch; near it use ln1p/expm1;
  scale the prices so every exponent is ≤ 0.
- **Reporting quantities with z instead of c(p)** under CES: the categories' quantities, the
  totals and eq 26's share all use the content at the equilibrium's prices.
- **A free category under CES** at r = 0 is an infinite content, not a NaN: the idle stretch is
  empty (§2.12).
- **The start assumed positive** with an in-work benefit (SL).
- **The walled c_i's program term's sign**: − (μ_w − μ_e), which is ≥ 0 where it is allowed.
- **The uniform program's order**: (ν_i·P^c + 0.0) + μ·P^c against (ν_i·P^c + μ·P^c) + 0.0; both are
  ν_i·P^c + μ·P^c, which is what makes GU = GS bitwise.
- **τ_R at r = 0.** R = 0 on idle land: the levy is defined, its rate is not.

## 12. Open questions

As built, 1f takes the first choice on each, and STATE.md records each as a decision open to veto,
numbered from 103 (unit 1e's are 88-102). The ones marked **Phase 2** bind it.

1. **Where each tax is levied** (§2.1; proposed decision 103): the payroll tax on gross wages of
   every hour sold, the consumption tax on final purchases at producer value (support and space
   included, intermediates and home output not), the rent tax on market rent in money.
   Alternative: the consumption tax on every purchase (breaks the ledger; Diamond-Mirrlees).
   **Phase 2**: the tapes' tax bases follow these definitions.
2. **Transfers in composites at consumer prices** (§2.2; proposed 104). Alternative: in rent
   units, which R14 forbids for the historical runs.
3. **The budget closes by the owners' levy** (RentRate, §2.5; proposed 105). Alternative: by the
   uniform transfer (Dividend), with rates given. **Phase 2 and Phase 6**: PLAN §5 scripts "laws
   and taxes (rates and dates)"; a tape that gives every rate needs the Dividend closure or a
   deficit, and one that gives relief scales in baskets (the poor law) is RentRate's.
4. **Supplement by default, Replace as the alternative** (§2.4; proposed 106). **Phase 2**: the
   agents' participation rule reads the transfer as supplementing the support unless the tape
   says it replaces it.
5. **The Dividend closure only where the budget does not depend on who works** (§2.6; proposed
   107). Alternative: an inner fixed point in d at each point, with its own uniqueness condition
   (for an in-work benefit, Σ_i (N_i/N)·Δμ/(χ_max,i·(ν_i − Δμ)) < 1 suffices).
6. **Walled types only under RentRate and without in-work benefits** (§2.8; proposed 108).
   **Phase 2**: the eras' trained type at its wall under a wage-supplement scheme (Speenhamland)
   needs an addendum.
7. **The start of the path is evaluated; `SurplusLabour`** (§2.9; proposed 109). **Phase 2**: an
   agent economy with a large in-work benefit has no oracle equilibrium to converge to.
8. **CES over the categories with the basket's weights, P = Z·M** (§2.10; proposed 110).
   Alternatives: a nested CES (space against a fixed goods bundle), or Stone-Geary around a
   subsistence basket, which PLAN §3.2's "category demand around a subsistence basket" suggests
   for the agents. **Phase 2**: the oracle and the agents must share the consumption rule on any
   instance compared; the draft keeps the fixed basket as the default.
9. **CES only with the dependence form and no reserved hours** (§2.11; proposed 111).
10. **A free category under CES empties the idle stretch** (§2.12; proposed 112).
11. **The land-share household reproduces A-joint in the generator only** (§2.13; proposed 113,
    amending decision 75): the oracle keeps decision 64, and check_dynamics' ρ > 0 targets also
    need external finance. **Phase 3**: the dynamics thread's steady states are the natural
    Phase 3 targets and need both.
12. **A basket per worker type is deferred** (§10; proposed 114). **Phase 2**: the 1750-like
    instance with owners buying domestic service needs it, or must use the common basket in both
    the oracle and the agents.
13. **The rent base is market rent in money; κ keeps 1e's definition** (§2.1; proposed 115). So
    eq 16's τ_R is 1/κ only where no plot is rented (KR1).
14. **`Eq1f::base` keeps 1e's accounts without a government, labelled** (§6; proposed 116).
    Alternative: recompute them with the government, which breaks `base`'s bitwise identity with
    1e's report.
15. **The random draws' ranges** (§8 f9; proposed 117): the build tunes and records them.
16. **No government purchases, deficits or taxes on interest** (§10; proposed 118). **Phase 6-8**:
    wars and debt.

## 13. Phase 1's gate, item by item

PLAN Phase 1's gate, with the tests that cover each (module paths under `tests/gate/`):

| gate item | covered by |
|---|---|
| the SSRN Appendix B instance: the published values (x* 0.86315, v 0.54344, Y 7.88061, N_a 1.34338) to 5e-6, and ADDENDUM §5's full-precision values to 1e-12 relative | `g1_appendix_b::published_figures_to_five_decimals`, `g1_appendix_b::full_precision`, `g1_appendix_b::cost_system_and_shares` (1a); carried bit for bit through every later unit by `c1_nesting::appendix_b_is_bit_identical` (1b), `m1_nesting::appendix_b_is_bit_identical` (1c), `d1_nesting::every_1c_golden_instance_is_bit_identical` (1d), `e1_nesting::every_1d_golden_instance_is_bit_identical` (1e) and `f1::every_1e_golden_instance_is_bit_identical` (1f) |
| the replacement closure's worked instance (c = 1, w = 3; at λ = 0, c = 0.4 and w = 1.2) | `g6_closure::worked_instance`, `g6_closure::recursive_automation_cuts_the_wage` (1a, price block); `m2_machine_block::closure_per_type` (1c, per machine type); `f2::the_worked_instance_inside_the_closure` and `f2::the_corner` (1f, inside the full closure: TX and TX3) |
| the fork identity and the category bounds on random instances | `c5_random_categories::fork_identity_and_category_bounds`, `c6_price_block::cost_and_accounting`, `c6_price_block::exact_interior_prices` (1b; check_interior's batteries); `m6_random_leontief::fork_and_bounds` (1c); at every equilibrium of 1d-1f through `d7_random_workers::identities`, `e8_random_parcels::identities` and `f9::identities` |
| the income identity to 1e-12 | `g5_random_economies::flow_benchmark`, `durability_and_interest`, `build_lags` (1a, with interest); `c5_random_categories::identities` (1b); `m6_random_leontief::identities`, `m3_two_recipe::income_with_interest` (1c); `d7_random_workers::identities` (1d); `e8_random_parcels::identities` (1e, the market's land); `f9::identities` and every f-golden through `check_identities_1f` (1f, with government: (I1)-(I4)) |
| three-taxes' resolution ledger, (φ_w, φ_r) = (0.6, 0.4) on its worked instance | `g7_three_taxes::shares_on_the_worked_instance`, `g7_three_taxes::lambda_zero_corner` (1a, price block); `m2_machine_block::ledger` (1c); `f2::the_worked_instance_inside_the_closure`, `f2::every_tax_leaves_the_worked_instance`, `f2::the_legs_of_a_consumption_tax`, `f2::the_corner`, `f2::the_circular_flow` (1f, inside the full closure, with T6 and T5) |
| constructed wall and interior cases recognised correctly | `g8_regimes::*` (1a: `base_is_interior`, `few_workers_hold_no_contestable_task`, `labour_heavy_machines_hold_no_contestable_task`, `no_interior_at_zero`, `not_viable`, the exact-zero rows); `c7_gaps_and_regimes::regimes` (1b); `m7_regimes_and_validation::regimes` (1c); `d2_one_type_corners::wall_goldens`, `closed_form_at_the_wall`, `all_human_goldens`, `labor_short`, `roots_below_the_bracket`, `d3_human_required::the_tail_decides_the_regime`, `d4_worker_types::the_wall_with_types`, `trained_at_its_wall`, `d6_wall_switches::tie_at_a_wall_switch`, `d8_regimes_and_validation::exact_zeros_at_the_junctions` (1d, solved); `e5_idle_land::idle_goldens`, `the_wall_at_zero_rent`, `e9_regimes_and_validation::exact_zeros` (1e); `f5::borne_by_employers_at_the_wall`, `f5::onto_idle_land`, `f8::no_idle_stretch_with_a_free_category`, `f8::the_good_free_at_the_all_human_corner`, `f8::the_wall_far_out` (1f) |
| each exit form on its own gate: the dependence form in 1a, s(q) in 1e; the 1d and 1e gates constructed (ADDENDUM §5 item 4) | the dependence form: `g1_appendix_b::*`, `g2_figure_3::*`, `g3_automation_path::the_papers_claims_along_the_path` (1a), with 1d's constructed gate `d2`-`d8` and `d9_goldens_file::*`; s(q): `e2_exit_value::p3_floor`, `e2_exit_value::race_closed_forms`, `e3_race::race_goldens`, `e3_race::the_tie_is_enclosure_by_price`, `e4_commons::commons_goldens`, `e5_idle_land::*`, `e10_goldens_file::*` (1e, constructed); both forms nest through `e1_nesting::exit_option_off` and `f1::*`, and keep their gates under a government through `f6::the_wage_leg_is_a_wage_tax` (KT, priced) and `f4`, `f5` (dependence) |

Every test in the table runs in `scripts/gate.sh` on WSL and on Windows. When 1f's build and
verification pass, Phase 1's gate is complete, and G1, the oracle lab, may start (PLAN Phase 1).

## 14. Changes during the build (P1.12), 2026-09-27

The brief asked for the departures in a §12, as units 1d and 1e keep them; this draft's §12 holds
its open questions, so they are here, in the §14 the draft reserved.

1. **The generator builds on 1e's.** `generate_1f.py`'s `Economy` extends `generate_1e.py`'s:
   its own evaluation (`ev`, the basket at the point's prices, the consumer side, the transfer,
   the walk with §4.5's c_i and supply by §4.4's rule), the exit sub-problem's trial with the
   rule, 1d's corner functions with the rule and δ(ω), the start, the priced path with the corners
   in ω = v/P_z and a CES basket's end, the idle stretch under a moving Dividend transfer, and a
   report of its own that asserts every identity of §4.8-4.9 at 1e-65. A fixed-basket economy
   without exit values runs 1d's solve, as in 1e; a CES economy runs the priced path, as in the
   oracle (item 5). It writes 215 goldens in about thirty seconds and reproduces every value of
   §7 to the digits shown; X3 is C3's composite frozen at one good and (3/7)² space. Departures
   from §7: the nesting assertion runs on
   six of 1e's instances (G1, W1, W3, W4, Q1, K3), which agree to 0 at 70 digits, and `f1` runs
   every instance and draw in f64; the count's scan is the priced path's only (no golden has the
   Dividend closure at ρ > 0, where the oracle also scans an exit-free path, item 4), and `f9`
   compares that count with a scan 16 times finer; A-joint is item 15; the pool's clearing is
   asserted at 1e-65/(1 − x*) on the line, since 1 − x* loses digits near x = 1 (CP at η = 2^-16
   keeps 65); and a free composite on idle land (TX3's idle end, λ = 0, where every price is 0
   in pool wages) saturates supply rather than dividing by 0. f is checked nonincreasing at 96
   grid points (C1-C3's line and wall) and Lemma F1 at 14 points.
2. **Where the rule lives.** The participation rule (`households::Rule`) and the CES basket
   (`households::Ces`) are crate-private and set on unit 1d's `WorkerEconomy` underneath
   (`set_households`), whose every supply, corner ratio, walled rate and the walk read them; with
   `Rule::none()` each is 1d's operation (§9), and 1a-1e's 360 tests pass unchanged. So 1e's
   solve is 1f's with the rule, and `HouseholdEconomy::solve` is `ParcelEconomy::solve_scanned`
   under it, with the report of §4.6-4.9 on top. Unit 1d's module changed its API, not its
   behaviour: `WorkerPoint` and `Eq1d` carry the transfer d the supplies read and a CES basket's
   content as crate-private fields, `PriceSide` the content and P_z, `workers_at` takes n_D and the
   market (d under Dividend reads W = v·n_D + Σ_i v_i·D_i, R = r·T_m, C = Y·P), `Corner` carries Y,
   `Path` the start, and `corner_delta` is δ(ω); `MachineEconomy` keeps the transposed chain for a
   basket's gross outputs and clears land for a given basket (`clear_basket`); the free functions
   `workers::supply` and `marginal_cost` are the rule's. 1e's exit sub-problem reads the point's
   d, computed before the plots on the whole endowment (the Dividend closure has no plot taker),
   bit for bit the one `finish` computes.
3. **Result types** (§6). As drafted, with: `HouseholdPoint` holds 1e's `ParcelPoint` with
   `content`, `price`, `consumer_price`, `transfer` and `unearned`; `HouseholdEconomy` adds
   `at_idle_edge`, `parcels()`, `net_factor()` (κ_w) and `start()`. `Eq1f::f_start` is
   `Option<f64>`, absent where f_0 = +∞ (item 5). `BasketEq::shares` equal 1d's `cat<j>.share`,
   which with the point's content is c_j·p_j/P already, so `Eq1f::outputs` adds only
   `cat<j>.content`; `within_rent` is an optional 0/1; the provider's flag is `provider_funded`,
   beside 1e's `funded` without a government. The economy's new keys, after 1e's: `basket_price`,
   `consumer_price`, `sigma`, `payroll`, `consumption`, `rent_tax`, `levy`, `dividend`,
   `dividend_composites`, `program_work`, `program_exit`, `revenue_payroll`, `revenue_rent`,
   `revenue_consumption`, `consumption_wage_leg`, `consumption_rent_leg`,
   `consumption_interest_leg`, `transfers`, `program_cost`, `within_rent`,
   `rate_for_one_composite`, `provider_receipts`, `provider_support`, `provider_spending`,
   `provider_composites`, `provider_funded`, `spending`, `composites`, `basket_wage_share`,
   `basket_rent_share`, `rent_share`, `recirculated_base`, `multiplier`, `f_start`, `res_budget`,
   `res_spending`, `res_composites`, `res_rent_share`, `res_euler`; then `cat<j>.content` and
   `worker<i>.{unearned, support, net_wage, spending, composites}` (a type's hours and exiters are
   1e's). The ledger's wage share counts every labour claim, the pool's and the walled types', as
   1d's φ_w of a category does; the multiplier is absent where R = 0. σ out of range is
   `OutOfRange { name: "sigma", requirement: Elasticity }`, not an `Item`; `Requirement` gains
   `BelowOne` (τ_w), `UnitInterval` (τ_R) and `Elasticity`.
4. **The scan** (§5.3 step 4) runs at ρ > 0 under a CES basket, or under the Dividend closure
   with a positive rate: with every rate 0 the transfer is 0 at every point and nothing moves, so
   `f1::zero_government_in_either_closure` holds bit for bit, `scan_points` included. A
   fixed-basket Dividend economy without exit values keeps 1d's path (so that the zero-rate case
   is 1e's bit for bit, §2.14), and gains a scan of its own there: `EXIT_SCAN` interior points of
   the all-human corner and each wall piece by the corner form, and of each region of the line;
   the location stays 1d's. The idle stretch bisects on T_m only where the transfer moves with
   T_m there, which on idle land (R = 0, prices fixed) is a payroll or a consumption tax under
   Dividend; a rent tax alone leaves 1e's closed form.
5. **A CES economy runs 1e's priced path** (whole points at each ω of a corner, ω = v/P_z with
   the fixed basket's price P_z = Σ_j z_j·p⁰_j, which is P_s itself for the fixed basket without
   reserved hours, so 1e's priced path is unchanged), exit-free or not. Its start is +∞ where a
   weighted category has no land through the chain at x = 0 (it is free at v = 0, its content
   and n_D grow without bound there: Appendix B's good is one), else the evaluation at v = 0. With
   a weighted category free at the wall's end (§2.12) the wall's last piece ends the path, its
   value −S_∞ read positive when ≥ 0 as the idle stretch's end is, S_∞ the supply at ω_∞ =
   1/(Z·M(λ̃)) (σ < 1; +∞ otherwise, and where no weighted category embodies labour) with the
   transfer's limit, d̂ composites under RentRate and −μ under Dividend (W, R and C over P
   vanish there).
6. **Validation** is §3.2's in its order; the rejected rows of 1e keep 1e's errors, compared as
   printed (a NaN field is not equal to itself). A priced type that takes no plot (s₀ ≤ s̲ or
   h = 0) admits the Dividend closure, as §2.6 says; its exit value is part of its supply.
7. **The residuals in household form** (§6): `budget` is 0 exactly there (every term 0), and
   `euler` too for the fixed basket without reserved hours (P is Σ_j z_j·p_j in the same order);
   `spending`, `composites` and `rent_share` carry the income identity's rounding, within 1e-12,
   not 0 as §6 said.
8. **SurplusLabour** is returned by both paths: 1d's (the corner form's f_0) and the priced one.
   `f10::exact_zeros` builds f_0 = 0 exactly (W4's economy with N 10: a benefit of 0.1 draws all
   10 into work at a zero wage, against n_D(0) = 10), which is `SurplusLabour { f_start: 0 }`.
   §8's `L_R = R` exactly is not built (no instance gives it in f64); `within_rent`'s closed
   interval is checked against the levy at every equilibrium instead.
9. **Certification** (§5.4) is recomputed when the rule is set, with κ_w·ε_i and, where a type
   takes plots, μ_w = μ_e; `f10::certification_under_a_government` flips it with a consumption
   tax (h·ℓ₀ = 1/13 on K3 with the good given 0.3 of land) and with an in-work benefit.
10. **Tests.** Unit 1f adds 6 unit tests (`households.rs`) and 65 gate tests: 57 in f1-f11 and
    `p1_gate`'s 8, one per item of PLAN Phase 1's gate and one that checks every test its table
    names exists. 432 in the package (83 unit, 348 gate, 1 doc). §8's rerun of the walk's unit
    tests with c_i is not built: the walk reads rates, and c_i only changes them
    (`households::tests` checks c_i against the point form, `f7` a walled type's wage). `check_identities_1f` (support_1f.rs) runs 1e's identities under the rule
    (`check_identities_1e_under`, with 1d's own where there is no government and the basket is
    fixed), pins P, P^c, d, each A_i and the content to the evaluation bit for bit, recomputes the
    budget, the accounts, the ledger and the new residuals bit for bit, checks (I1)-(I4), the
    receipts, the rent share, coverage, D.3's identity and the corollary's bound to 1e-12, each
    walled type's c_i·P, a CES basket's shares and content, and that a type on the Plot branch has
    every exiter on a plot. Unit 1e's `e1`, `e8` and `e9` expose their instances, draws and units
    helper to 1f's tests. `f6::home_output_is_untaxed` shows the switch by evaluating the supply
    with ê = e at KT's equilibrium, not through a crate-private switch. `f8`'s Lemma F1 test reads
    the wall, where the technique is fixed and n_D = T·L_s/B_s, at CW's and AJW's equilibria.
11. **The random draws** (§8 f9; decision 117). Five drawn sets, seeds 951-955, 60 equilibria
    each, on e8's tables (a, b, c, g cycle its sets One, Types, Interest and Goods) and d7's draws
    with reserved hours (d): (a) RentRate, (b) Dividend with every exit made dependence or a floor
    without a plot, (c) CES with every exit made dependence, (d) reserved hours, (g) in-work
    benefits. (e) re-solves (a) and (d) at (1 − κ_w, 0), (f) re-solves (b) in Replace mode with
    the rent tax alone at τ_R ≤ ν·P(0)·N/(2T). The tallies, over 341 draws: 31 invalid, 7
    `NotViable`, 3 `SurplusLabour`; 142 on the line, 155 on the wall, 3 at the all-human corner,
    51 on idle land, 2 ties; 248 under RentRate and 62 under Dividend, 120 in Replace mode; within
    the rent 102 times and outside 138; κ ≥ 1 at 45 scarce equilibria and below at 204.
    `f1::random_economies_are_bit_identical` runs d7's and e8's 897 draws in household form:
    871 the same equilibrium bit for bit, 9 `NotViable` in both, 17 the same error.
12. **The instances.** CI needs no tuning: W3's N = 4 is on the idle stretch. KT is K3 with t_c
    0.25 against τ_w 0.2; GT against GT′ and ER against ER′ came out bit for bit in f64.
13. **Goldens.** `goldens_1f.txt` has 215 goldens; `goldens_1f.rs` rounds them to 20 digits
    (`f11`). The oracle's f64 values match the 444 golden comparisons of `f1`-`f11` and `p1_gate`
    within 1.9e-15 relative (CW's wage near the wall's real-wage ceiling), all but three within
    1.0e-15; the three made by finite differences are within their 1e-8 (§5.5): the incidence share
    by the three-point difference in τ_w (6.7e-10), ε_D (1.9e-10) and ε_S (2.2e-11). The near-x = 1
    outputs of AP and CP keep full precision (their tolerance 2^-53/(1 − x*) is not needed).
14. **Mutation.** 57 mutants of the new code, each alone, the unit and gate tests run: the rule
    (the transfer in each closure, Supplement and Replace, each term of num and den, the corner
    ratio, the walled rate, the fixed δ, the moving transfer), the CES basket (the content's sign,
    Z, the Cobb-Douglas weights, the power mean, the end's labour), the evaluation (the CES price
    and land, the report's outputs and weights, rent income at r = 0, δ(ω)'s terms, the corner's
    δ, the corner's walled rate, the start, a tie's land), the path (CES through the corner form,
    both `SurplusLabour`s, the scan, the free end, S_∞, the CES start, the certification's κ_w
    and Δμ, the exit sub-problem's d, ω in P), the report (the levy's sign, the wage share, the
    rent share, the multiplier, the support in Replace mode, the provider's receipts, an infinite
    start) and the validation (CES with reserved hours, Dividend with plot takers, the program
    rule, the consumption tax, the Dividend program). The first run killed 56. The survivor
    switched the scan at ρ > 0 off: nothing checked that it runs, and
    `f10::the_scan_where_monotonicity_is_not_proved` now does. The 22 mutants that the unit tests
    had caught first were run against the gate alone, which kills each, and the survivor with
    them.
15. **A-joint** (§2.13). The generator solves check_pinning's A-joint from its own equations
    (land clearing in x* at 70 digits), within 5.1e-16 of its printed values, and solves the
    household form with γ(1) 10^-29 below A-joint's (A-joint itself is `NotViable` in both, D(1)
    = 0), whose x* and w/p agree with the equations' to 1e-20. No AJ golden is written; AJ1 and
    AJW are.

### Changes from the verification (P1.13), 2026-09-27

The verification's derivation agreed with the oracle on 566 equilibria (67 of this document's
instances and 499 random draws) within 1.8e-13 relative, and on every golden of goldens_1f.txt
in Appendix B's family, ER, KT and KR to 1e-25; it found two blockers and one major error, below.
Its mutation pass left nine of 30 mutants alive, two of them equivalent.

16. **The all-human corner as v → 0 under CES** (blocker). The priced path bisects a corner on bit
    patterns, whose first midpoint lies near 2^-511 ≈ 1e-154 when the bracket starts at 0. A
    weighted category made of labour alone at x = 0 (Appendix B's good) is almost free there, and
    at σ ≥ 2 space's content underflows against it: B_s < 1e-308, Y = T_m/B_s = +∞, and n_D came
    out NaN through a task service Y·0. The solve returned `NonFinite` for economies with one
    equilibrium at the all-human corner: CA (N 40, T 3, γ = 1.5 + x, χ_max 0.02, C3's basket;
    v 0.63900965042269), and the verification's draws R7102_41 (σ 3.0), R7103_54 (σ 2.59) and
    corner7201_25. A variant of R7102_41 at σ 2 under Dividend showed the same with Y finite:
    n_D = Y·L_s overflowed, and the transfer read from W = v·n_D with τ_w = 0 was 0·∞, NaN.
    Where a CES basket's Y or n_D is +∞ the true labour demand is beyond every double (the
    category that sets the power mean keeps content of the order of its weight, so L_s is not
    tiny), and `WorkerPoint::excess_demand` now reads the point +∞ (`beyond_every_double`,
    always false for the fixed basket, whose B_s does not move).
    `f8::the_good_free_at_the_all_human_corner` has CA's goldens, the point at v = 1e-154, σ
    from 2 to 8 under four governments, and R7102_41's variant with its point at v = 2^-511.
17. **The CES corners in v** (blocker). §5.3 step 2 had a CES corner bisected in 1e's ω = v/P_z,
    v = ω·Bᶻ_s/(1 − ω·Lᶻ_s). Near ω = 1/Lᶻ_s that resolves v only to about 2^-52·v·Lᶻ_s/Bᶻ_s.
    For the fixed basket this is the wall's own conditioning (its P is v·L_s + B_s); under CES
    it is a loss, while dv/v per ulp of the data is about 2. C1's basket on W3 lost 1.5e-12 at
    N 0.1 (v 4.3e4) and 3.8e-11 at N 0.01 (v 4.3e6), and at N 0.001 (v 4.3e8) the pool's
    residual exceeded the labour net (`LaborNotCleared`); the draws idle7202_18 (v 5.0e7) and
    idle7203_38 lost 5.6e-10 and 1.2e-10, and idle7202_43 (v 1.2e11) was refused. A CES economy
    now bisects each corner piece on the bit patterns of v, from its v_lo to its v_hi (+∞ on the
    wall's last piece), with the piece's technique; f moves with v as with ω, and the scan keeps
    ω. The fixed basket keeps ω, so 1e's priced path is unchanged bit for bit. The generator
    bisects a CES corner in v too (400 halvings); in ω its own pool residual reached 1.1e-65 at
    N 0.01. `f8::the_wall_far_out` has CF2's and CF3's goldens (N 0.01 and 0.001): v within
    1.6e-15 and P within 3.7e-15 at v 4.3e8. CW's wage, 1.8e-15 from its golden before, is now
    within 2.5e-16.
18. **The power mean's direct sum** (major). §5.1 step 2 had ln M = ln p_scale + ln1p(S)/(1 − σ),
    S = Σ_j s̄_j·expm1(x_j), everywhere. Since 1 + S = Σ_j s̄_j·exp(x_j) ≥ the weight at
    p_scale, it cancels when that weight is small: W1 under σ 20 with eq 26's weights (space's
    0.3^20, 4.4e-8 of Z) has 1 + S = 1.5e-7 at its equilibrium, and P was off by 2.6e-11, the
    content, Y and the shares by 5.2e-10, which §5.5's claim did not allow. ln M now takes the
    ln1p form where S ≥ −1/2 (`households::LN1P_FLOOR`) and ln of the sum of positive terms
    below; §5.5 states the bound of both. `f8::a_steep_basket_with_a_small_weight` has CS's
    goldens (W1 at σ 20); every value is within 1.8e-15 of the generator's. Where S < −1/2
    without a small weight the two forms differ by a few ulps, and every earlier golden holds as
    it did.
19. **Tests for the mutation pass's survivors.** V13 (the all-human bisection's f_lo, which matters
    only for a root next to ω = 0) and V28 (the exit sub-problem's d, which only plot takers
    read, and the Dividend closure has none) are equivalent. Each other survivor has a test that
    fails with it:
    - V15, the CES required hours with z: `check_identities_1f` recomputes each category's final
      output Y·c_j and the common human-required hours Y·Σ_j ŷ_j(c)·L^H_j from the gross outputs
      at every equilibrium (unit 1d's report check, which pins them with z, lapses under a rule;
      the rest of it is not rerun under the rule, whose supply, walled wages and content it
      would need).
    - V23, a CES start through the fixed basket's corner form: `f8::eq_26_at_the_equilibrium`
      asserts C1-C3's start absent (+∞), and `f8::the_start_under_ces` pins the start of a CES
      economy whose good has land to the point at v = 0 bit for bit, and returns `SurplusLabour`
      with that value under an in-work benefit that draws all 100 workers in.
    - V26, a priced start of exactly 0 read positive: `f10::exact_zeros` adds the priced path's
      f_0 = 0 (W4's economy with N 10 and a floor without a plot, s̲ 0.5, worth nothing at v = 0
      where the good is free), `SurplusLabour { f_start: 0 }`, against a benefit of 0.01.
    - V24 and V29, S_∞ without −μ or without the efficiency:
      `f8::the_free_end_with_transfers_and_types`, CW's f_∞ under Dividend with and without a
      uniform program, and f_∞ of two types of efficiency 1 and 2 under RentRate with every
      instrument against S_∞ written from §5.3 step 3.
    - V21, V22 and V30, the exit-free scan: `f10::the_scan_where_monotonicity_is_not_proved`
      asserts `scan_points` = `EXIT_SCAN` × the scanned pieces (the corner, each region of the
      line, each piece of the wall); the exit-free count is a function (`count_changes`), and a
      unit test feeds it a gap with an extra pair of changes (three, `MultipleEquilibria`).
    - V27, `net_factor` inverted: `check_identities_1f` pins κ_w bit for bit, and f5 and f6
      assert 0.9 at GP and 0.8 at GT.
    - The minors: `p1_gate::wall_and_interior_cases` names its NoMarket instance I4 with exit
      (2, 0, 0.5); `f3::eq_17_at_full_automation` computes κ·N·Bᵖ_s/T from AP's equilibria.

    The nine mutants, rerun on the fixed code, are each killed, and so are seven mutants of the
    fixes (the +∞ reading off, and with Y alone; the corners in ω; the ln1p form everywhere; the
    direct sum everywhere, which `households::tests::the_ces_basket` kills near σ = 1; the CES
    all-human corner at x = 1). The derivation's 69 instances and 510 draws, run again through
    its own harness and solver on the fixed code, all agree, within 1.8e-13 (a levy, a
    difference), with no refusal left.
20. **Counts.** `goldens_1f.txt` has 235 goldens (CA, CF2, CF3 and CS added). The package has
    438 tests: 84 unit, 353 gate, 1 doc; 1f's are 7 unit, 62 gate and `p1_gate`'s 8. Proposed
    decision 119: a CES basket's evaluation keeps these numerics (the direct sum below
    S = −1/2, the corners bisected in v, a point beyond every double read +∞); it changes no
    fixed-basket result.
