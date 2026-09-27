# Unit 1e: parcels, the idle margin and the priced exit

Dated 2026-09-27. This is the unit's specification, written before any code. A 70-digit
scratch prototype built on `generate_1d.py` (in the run's scratch directory, not kept) checked
every derivation below and supplied the numbers in §7; `goldens/generate_1e.py` must
reproduce them. The build records where it departs from this draft in §12.

- **Pinned to:** laborformal `31b3482`. Every laborformal path and line below is at that
  commit.
- **Paper:** SSRN 7226858 (posted 2026-09-23), cited as "SSRN p.N (eq k)". It prices exit as
  dependence (eq 9) and keeps land's idle margin only as "unused fixed inputs have zero rent"
  (A.1), zero income from an unused endowment (App. C), the enclosure paragraph of §4 and the
  coverage ratio (eq 16, eq 28).
- **Older revision:** `pinning/paper/main.tex` (2026-09-21), cited as "main.tex:N". The priced
  exit s(q), the idle margin and parcel quality are there only (ADDENDUM §5 item 3): :140-145,
  :355-385, :394-419, :692-698, :804, :835-864. Its Appendix B is not used (unit-1a.md §0).
- **Reference checks:** `pinning/checks/corner/check_enclosure.py` N-i to N-vi (:42-126);
  `pinning/checks/check_pinning.py` P3 (:131-145) and P5 (:107-129);
  `dynamics/checks/check_dynamics.py` P1-P4 (:397-419).
- **Gate (PLAN Phase 1; ADDENDUM ruling 3, A7 and §5 item 4):** each exit form on its own gate.
  The dependence form keeps 1a's and 1d's. The priced form s(q), with the idle margin and
  parcel quality, gets this unit's constructed gate: the idle-margin regime and the enclosed
  regime recognised and solved; check_enclosure's worked instance reproduced; both exit forms
  nest, with the exit option switched off, into 1a to 1d exactly.
- **Built on:** units 1a to 1d (docs/unit-1a.md to unit-1d.md). Every equation, convention,
  constant and precision rule of those units carries over unless this document says otherwise.
- **Goldens:** a new generator, `goldens/generate_1e.py`, writing `goldens/goldens_1e.txt` (§7).

## 0. What the sources give

| source | what 1e takes from it |
|---|---|
| main.tex:140-145 | workers "supply one hour or exit to an outside option worth s"; "The baseline has one non-produced input with a fixed stock"; "parcels differ in quality, and idle parcels have reservation rent zero" |
| main.tex:369-373 (eq:exit :370-372) | q = r/p, "the machine-made good is the numeraire"; an exit life yields s₀ in goods and needs h_e units of the input's services, "quality-adjusted access to space, so no particular parcel is needed"; a dependency floor s̲ < s₀; s(q) = max(s₀ − q·h_e, s̲); "the only essential property is ∂s/∂q ≤ 0"; an exit bundle with produced goods "adds a goods term" |
| main.tex:375-381 (Prop exit and its proof) | (i) while suitable parcels lie idle, "the marginal such parcel rents at zero: the exit plot is free and s = s₀", which is what "a commons" is; (ii) once none is idle, "the exit plot pays the ruling rent and s = s(q)"; (iii) a rise in q shifts labour supply outward; proof (ii): "the exit plot must outbid other uses for the marginal suitable parcel" |
| main.tex:383 | "A parcel rents at zero because little can be done with it"; the floor s̲ is "otherwise-funded" (dependency, public provision, tolerated use); "the flat segment at s̲ is an upper bound" |
| main.tex:385 | enclosure, by law or by price: "the outside option falls, participation rises, and the wage falls at given technology" |
| main.tex:394-419 | the two sides, w = c(r)·γ(x*) and w ≥ s(q) with q = r/p; the channels r → c(r) → labour demand and q → s(q) → labour supply "can pull the equilibrium wage in opposite directions" |
| main.tex:692-698 (App. A) | "Each input supplies a fixed service endowment T_j at rent r_j"; "parcels differ in quality, idle parcels have reservation rent zero, and r(z) is the rent schedule over parcels z. Housing and independent exit use these services alongside production"; s(q) is "the reduced-form outside option"; in equilibrium "the non-produced input markets clear, and each worker participates exactly when work beats exit" |
| main.tex:804 (App. C) | "With several non-produced inputs or qualities, total rent income is the sum or integral of rental payments for their service endowments" |
| main.tex:835-852 (App. D coverage) | P_s = p·g_s + r·h_s; κ = rT/(N·P_s) = qT/(N(g_s + q·h_s)); q* = N·g_s/(T − N·h_s) when T > N·h_s; the instance T = 100, g_s = h_s = 1: fifty covered at q = 10/9 (κ = 20/19), eighty need q ≥ 4, 120 never |
| main.tex:854-864 (App. D, the race) | q_enc = (s₀ − s̲)/h_e; independent exit outlasts the fiscal replacement iff q_enc ≥ q*, iff N ≤ q_enc·T/(g_s + q_enc·h_s); "q_enc = 1.5 implies a threshold of 60 people" |
| SSRN §4 pp.10-11 (eq 7-9, Prop 3) | the basket P_s = zᵀp; u_n = log(1 + wn/P_s) − χn with support "in both work and exit"; s = (e^χ − 1)P_s; "Access to common land is a historical example of support through shared resources. Enclosure could remove that access" |
| SSRN §5 pp.11-12 (eq 10-11) | N_a = N·F(log(1 + w/P_s)); "the endowment T is fully used at r > 0" (the premise 1e drops) |
| SSRN A.1 pp.24-25 | "Input k supplies a fixed service endowment T_k at rental r_k"; types differ in "access to support, living requirements"; "fully employed non-produced inputs satisfy T = Bᵀy. Unused fixed inputs have zero rent" |
| SSRN App. C pp.30-31 | I = wN_a + rᵀT; "At zero rent, an unused part of a fixed endowment contributes no income" |
| SSRN p.16 (eq 16), D.3 pp.32-33 (eq 28) | κ = R/(N·P_s); with a two-item basket κ = qT/(N(g_s + q·h_s)), increasing in q toward T/(N·h_s), q* = N·g_s/(T − N·h_s); the T = 100 instance |
| check_enclosure.py N-i (:42-54) | the marginal parcel rents at zero while land demand falls short of the stock; the closure's land clearing T_P + T_H = T "closes that margin at the corner" |
| check_enclosure.py N-ii, N-iii (:56-80) | ds/dq = −h_e; q_enc = (s₀ − s_d)/h_e; N_crit = q_enc·T/(g_s + q_enc·h_s); the instance (s₀ 1.5, h_e = h_s = 1, s_d 0, g_s 1, T 100): q_enc 1.5, N_crit 60, q*(50) = 1, q*(80) = 4 |
| check_enclosure.py N-iv, N-v, N-vi (:82-126) | a uniform transfer u cancels from (w + u) − (s(q) + u), and past q_enc the floor is s_d + u; lower s means more participants and a lower wage (a linear toy: s 25 → 20, n 0.5 → 0.6, w 30 → 26); the take s₀ − s(q) caps at h_e·q_enc (1.0 at q = 1, 1.4 for every q > q_enc at (1.5, 0.1, 1)) |
| check_pinning.py P3 (:131-145) | s(q) meets the floor exactly at q_enc; (s₀, s̲, h_e) = (10, 4, 6): q_enc = 1, weakly decreasing on (1/3, 2/3, 1, 2), 4 above q_enc |
| check_pinning.py P5 (:107-129); check_dynamics.py P1-P4 (:397-419) | the dividend d cancels from the participation comparison; with a working life's land h_w = h_e the comparison is q-free on the s₀ branch (1e has no h_w, §9) |
| dynamics/latex/main.tex:138 | a later draft at the pin: "Land is fixed in total, heterogeneous in quality, with the worst parcel's reservation rent zero" |

### 0.1 There is no equilibrium with parcels or a priced exit at the pin

laborformal has no equilibrium instance with a parcel-quality schedule or with s(q)
(ADDENDUM §5 item 4). check_enclosure and P3 are symbolic; check_pinning's A-joint checks
only that w > s(q) at full participation in a Cobb-Douglas closure, which is 1f's household
(decision 75). Every 1e golden is therefore constructed. The worked instances of
check_enclosure (N-iii) and SSRN D.3 are reproduced twice: as the closed forms they are
(P), and inside equilibria of Appendix B's closure, where the race of main.tex:854-864 is
visible along an automation path (Q).

### 0.2 Three idle margins, and "enclosed"

Three things in the sources are called idle or free, and 1e keeps them apart:
- **Idle suitable land for exit, the commons** (Prop exit (i)): open-access parcels not all
  occupied by exit plots. The exit plot is free, and s = s₀. The market's rent can be
  positive meanwhile: the evidence "lives" in the sloped regime (dynamics/latex/main.tex:238).
- **Idle enclosed land** (SSRN A.1, App. C): the market's land not fully used, at zero rent.
  This is where 1d's `LaborShort` economies have their equilibrium (§2.8).
- **The floor** s̲: not land. It is "otherwise-funded" (main.tex:383) and stays outside the
  market (§2.6).

**Enclosed** means no suitable land idles: exit plots pay the ruling rent, and s = s(q)
(Prop exit (ii)). An economy passes from one to the other by law (a parcel's access changed)
or by price (the commons filled, or q rising past a type's q_enc).

## 1. Scope

**In scope:**
- parcels with an acreage, a quality (land service per acre) and an access (enclosed, or open:
  a commons); the rent schedule over parcels; which parcels idle;
- the idle margin of the market's land: land idle at zero rent, with the pool's wage as
  numeraire, as the path's fourth stretch after the wall, which resolves 1d's `LaborShort`;
- both exit forms, per worker type: the dependence form (1a-1d, SSRN eq 9) and the priced form
  (main.tex eq:exit), under one participation rule;
- exit plots on land: on the commons while it has room, at its shadow rent when it is full,
  and rented on enclosed land, which they take from production;
- enclosure by price (the commons filling; q crossing a type's q_enc along the path, and the
  equilibrium that sits there) and by law (a parcel's access switched between two solves);
- coverage κ, q*, N_crit and the race (main.tex:835-864; SSRN D.3; check_enclosure N-iii), as
  closed forms and at every equilibrium;
- counting equilibria where the priced form makes labour supply fall along the path (§5.4).

**Out of scope** (§9 says where each goes): several non-produced services with a requirement
matrix (open question 12); a Ricardian working cost per acre (open question 1); the priced form
with reserved tasks (open question 8); the pure main.tex form without support (open question
4); production on open-access land; a working life's own land h_w; a quality threshold of
suitability; the dated switching of access (the tape's); price-responsive baskets and the fiscal
side (1f); a dump interface.

## 2. Decisions

### 2.1 Parcels are efficiency units

Parcel z supplies A_z·Q_z units of land service per period: A_z acres of quality Q_z,
services per acre. Every use takes land service in these units: production and housing through
1a-1d's direct requirements, and an exit plot h_e units ("quality-adjusted access ... no
particular parcel is needed", main.tex:369). Parcels of one access are then perfect
substitutes per unit of service: an enclosed parcel in use rents at r·Q_z per acre, which is
main.tex:692's rent schedule r(z), and rent income is Σ_z r·Q_z·A_z, main.tex:804's sum over
qualities. A parcel of quality 0 supplies nothing and always idles: the "truly idle margin" of
main.tex:383.

The extensive margin appears only where land idles. At zero rent every idle parcel's
reservation rent is met, so which parcels idle is indeterminate; the oracle reports the worst
idle first (quality descending in use, ties to the lower index), the order "little can be done
with it" suggests. It is a convention, and Phase 2 should compare totals. A Ricardian model,
where working an acre costs labour and the worst parcel in use rents at zero while better ones
earn a differential at positive rent, needs that working cost, which no source gives: open
question 1.

### 2.2 Open access is a status: the commons, for exit only

An open-access parcel is not rented. Its services are free to exit households and not
available to production, since no one can rent it out. The commons is T_o = Σ_{open} A_z·Q_z.
Enclosure by law switches a parcel from open to enclosed between two solves: T_o falls and the
market's endowment T rises by the parcel's services (PLAN §3.5's dated switch is the tape's).
Production on the commons is open question 2.

### 2.3 Both exit forms under one participation rule

Each worker type names its exit form. Both are SSRN eq 8 with the exit life's yield added in
the exit state. A type-i worker with work cost χ has

    u_n = ln(ν_i + (v_i·n + e_i·(1 − n))/P_s) − χ·n,        n ∈ {0, 1}

where ν_i is its support in baskets (1d §2.3), v_i its wage and e_i the money value of its exit
life. It works exactly when

    χ ≤ ln((ν_i·P_s + v_i)/(ν_i·P_s + e_i)),   n_S,i = N_i·F_i(ln1p((v_i − e_i)/(ν_i·P_s + e_i)))

and its reservation wage is (e^χ − 1)(ν_i·P_s + e_i) + e_i.
- **The dependence form**: e_i = 0. This is SSRN eq 9 per type, 1d's rule, bit for bit.
- **The priced form**: e_i = p_g·s_i, where s_i = max(s₀,i − q_o·h_i, s̲_i) is main.tex's s(q)
  at the rent q_o = r_o/p_g the exit plot pays, in units of the exit good g (§2.5).

The limits are the two sources. With s₀ = s̲ = 0 (the exit option switched off) the priced form
is the dependence form bit for bit. As χ_max → 0 (main.tex's identical workers) a worker works
exactly when v_i ≥ p_g·s(q), main.tex:396 and Prop margin (iii) (:170), whatever ν_i: the
support cancels, as check_enclosure N-iv's uniform transfer does. A proportional rescaling of
prices leaves the choice unchanged (SSRN p.11).

ADDENDUM ruling 3 makes the priced form the default of the historical runs. The oracle takes
the form each worker type names; `ParcelParams::from_workers`, 1a-1d in parcel form, gives the
dependence form, whose gate is theirs.

### 2.4 Support stays positive

ν_i > 0 for every type, 1d's rule, under both forms. main.tex's own form has no support; with
ν_i = 0 the exit value falls with goods prices as the wage does, and at the all-human corner,
where w/p_g = 1/L̄_g whatever the wage, supply need not vanish as v → 0: the excess demand can
be negative at the start of the path (surplus labour at every wage), and more than one
equilibrium is common (22 of 120 scratch draws). With ν_i > 0 the support basket keeps its land
content as v → 0 and supply goes to 0, so the path starts positive (§5.3). The support is
SSRN's dependence, which is main.tex's floor "through someone else's household" (:369); the
priced form reads as dependence plus an independent exit life when it pays. The pure form is
open question 4.

### 2.5 The exit good is one category

s₀ and s̲ are in units of one category g, `exit_good`, and q = r/p_g. main.tex:369 names "the
machine-made good"; SSRN D.3's basket has "a consumption good" and space; the historical tapes
will name food. The basket is not a usable deflator: q = r/P_s ≤ 1/h in Appendix B's basket,
so check_enclosure's q_enc = 1.5 at h = 1 is never reached, and Q3 goes to the wall with 1.3%
participation instead of its tie (§3.3). A bundle of goods is open question 5.

### 2.6 Exit plots take land; the home account stays outside the market

An exit household on a plot uses h_i units of land service: on the commons while it has room,
then on the commons at its shadow rent, then rented on enclosed land at r ("the exit plot must
outbid other uses", main.tex:380). Plots rented on enclosed land leave production: the market's
land is T_m = T − T_p and Y = T_m/B^q. The exit life's home output (s₀ per plot household), the
rent it pays and the floor's s̲ are in kind and stay outside the market's accounts: SSRN
App. C's income identity holds with the market's land, I = w·N_a + r·T_m + interest. They are
reported as the home account.

Coverage κ (SSRN eq 16) counts the whole enclosed rent base r·T, as check_enclosure and main.tex
do; `funded` stays 1d's flag on the provider's market budget, (r·T_m + interest)/P_s > ν. They
differ when plots are rented: Q1 has κ 1.033 and a provider short of 16 baskets. Rents paid in
money out of home goods sold on the market would change goods clearing: open question 6.

### 2.7 The commons clears by a shadow rent

The rent an exit plot pays per unit of service, r_o, lies in [0, r]. It is 0 while the commons
has room (**Commons**). When the commons is full, r_o is the rent at which plot demand equals
T_o (**Crowded**): plots are rationed by their value, and the shadow rent is nobody's income.
At r_o = r plots spill onto enclosed land (**Enclosed**). The regime changes continuously with
prices. A type whose plot demand drops at its q_enc (plot or floor) can split between commons
plots and the floor. Open question 10.

### 2.8 Idle enclosed land at zero rent; the pool's wage as numeraire

At r = 0 machines cost labour alone (and themselves). Labour holds task x only if
1 ≤ γ(x)·λ̃_τ/θ_τ, which is impossible wherever the technique is viable (d > 0, decision 64):
every contestable task is automated, x* = 1. So the idle margin continues the wall's end,
where v = w/r → ∞: the path gets a fourth stretch, at r = 0, along which the market's land in
use T_m falls from its value at the wall's end toward 0. Prices there are per unit of the pool's
wage; real quantities are unchanged. Every economy 1d calls `LaborShort` has its equilibrium on
this stretch. 1e never returns `LaborShort`; an economy where no one works even at the ceiling
of the real wage is `SolveError::NoMarket`. Open question 7.

### 2.9 No reserved tasks with the priced form, for now

With plots on enclosed land the market's land depends on participation, and with a walled type
participation depends on its reserved demand D_i = Y·R_ŷi, which depends on the market's land:
a fixed point in Y at each point of the path, and not a monotone one (a higher Y raises the
walled wages and P_s, which lowers pooled supply and adds plots, but it also lowers the walled
types' own exits). 1e validates that an economy with a priced type has no reserved hours; all
its types are pooled. Open question 8, which binds Phase 2.

### 2.10 Enclosure by price: points and ties

Along the path the exit good's price rises (dp_g = λ̃_g·dv, §5.4), so the market's q = r/p_g
falls, and it crosses each priced type's q_enc,i at most once: that type's **enclosure point**.
Below it the type prefers its floor to a rented plot; above it, a plot. If plots spill onto
enclosed land there, the market's land jumps down, and with it the labour demand and f. An
equilibrium in the jump is an **enclosure tie**: the type's exiters are indifferent between a
rented plot and the floor, and the share ψ of them that rents clears the pool, in closed form
(§4.7). It is enclosure completed by price with q = q_enc exactly, and over a range of
parameters the economy sits there (Q2, Q3). It is reported inside `Interior`, as decisions 62,
69 and 136 put a gap, a tie and a corner there. Open question 11.

### 2.11 A labour supply that falls along the path, and the scan

Along the path a priced type's supply can fall where its plot is more land-intensive, at the
rent it pays, than the market's own version of its exit goods, h_i·r_o > s₀,i·b̃_g (Lemma 5,
§5.4): the plot gets cheaper relative to the wage faster than the goods do. These are main.tex's
two channels (:413-419) pulling apart. The excess demand can then rise along a stretch, and the
count of 1a-1d, which evaluates f only at the ends of stretches and at switches, misses
equilibria: M has three, two of them inside one region of the line whose ends have the same
sign. So an economy with a priced type that can take a plot also evaluates f at `EXIT_SCAN`
interior points of every piece. The count is exact where §5.4's certification holds and
limited by the scan's resolution elsewhere; more than one equilibrium is refused (decision 70).
Open question 9, which binds Phase 2.

### 2.12 Bitwise nesting through a normative evaluation order

As 1b-1d (decision 63). With every type in the dependence form (or the priced form with
s₀ = s̲ = 0), no commons, and one enclosed parcel of quality 1, every new operation of §5.1 is an
exact no-op: T = 0.0 + A·1.0, no exit sub-problem, T_p = 0.0 and T − 0.0 = T, e_i = 0.0 in the
supply, no enclosure point and no scan. The sequence is 1d's with one value appended, the idle
stretch's end −S_∞ < 0. So wherever 1d returns an equilibrium, 1e returns it bit for bit; 1d's
`LaborShort` economies become idle-land equilibria, with f_∞ bit for bit 1d's `excess`; 1d's
`MultipleEquilibria` stay (with one more change of side where f_∞ > 0), except the knife edge
f_∞ = 0 after an equilibrium, which 1d counts as a second change and 1e does not (§5.3).

## 3. Parameters, validation and instances

### 3.1 Parameters

1d's parameters stay (unit-1d.md §3.1) except `land`, which the parcels replace. New:

| symbol | name in code | meaning |
|---|---|---|
| A_z | `parcels[z].acreage` | acres of parcel z |
| Q_z | `parcels[z].quality` | land service per acre per period; 0 or scale |
| — | `parcels[z].access` | `Enclosed` (rented on the market) or `Open` (a commons, for exit plots) |
| — | `exits[i]` | type i's exit form: `Dependence`, or `Priced { gross: s₀,i, floor: s̲_i, plot: h_i }` |
| g | `exit_good` | the category in whose units s₀ and s̲ are measured; q = r/p_g |

Derived at construction:

| symbol | meaning |
|---|---|
| T = Σ_{enclosed} A_z·Q_z | the market's land-service endowment (1a-1d's T) |
| T_o = Σ_{open} A_z·Q_z | the commons |
| Δ_i = s₀,i − s̲_i | the plot's gross advantage over the floor, in goods |
| q_enc,i = Δ_i/h_i | where a rented plot stops paying (main.tex:858), for h_i > 0 and Δ_i > 0 |
| 𝒫 | the plot-taking types: priced, with h_i > 0 and Δ_i > 0 |
| p*_g,i = r·h_i/Δ_i | the exit good's price at type i's enclosure point |
| b̄_g, ℓ₀ | the exit good's land through the chain, and n_D per unit of market land, at x = 0 (§5.4) |

**Notation.** r is the market's rent, 1 (scarce land, the numeraire) or 0 (idle land);
r_o the rent an exit plot pays per unit of service; q = r/p_g and q_o = r_o/p_g; s_i the exit
life's yield in goods and e_i = p_g·s_i in money; E_i = N_i − n_S,i the exiters of type i;
G(r_o) the land exit plots ask for; T_m the market's land in use, T_p the enclosed land in
rented plots, T_idle the idle enclosed land, T_oc the commons occupied; ψ the renting share at an
enclosure tie; κ coverage. 1d's symbols keep their meanings (v the pool's wage, ε_i, ν_i, F_i,
π, L_s, B_s, B^q).

### 3.2 Validation

1d's checks (unit-1d.md §3.2 and §12 item 8) in 1d's order, less `land`, then:
- at least one parcel; each acreage in [`SCALE_FLOOR`, `SCALE_CEIL`], each quality 0 or in that
  range, all finite, −0.0 stored as +0.0 (`ParamError::Item { kind: "parcel", .. }`);
- T in [`SCALE_FLOOR`, `SCALE_CEIL`] (some enclosed parcel has positive quality), T_o at most
  `SCALE_CEIL` (`Invalid { name: "parcels" }`);
- one exit form per worker type (`Invalid { name: "exits" }`); a priced form's s₀, s̲ and h each
  0 or in the scale range, finite (`Item { kind: "worker type" }` named `gross`, `floor`,
  `plot`);
- `exit_good` names a category (`Invalid { name: "exit_good" }`);
- an economy with a priced type has no reserved hours (`Invalid { name: "reserved" }`, §2.9);
- **land for every plot**: T + T_o > Σ_{i∈𝒫} h_i·N_i (`Invalid { name: "parcels" }`), so that
  some land is left for production when everyone exits, and Y > 0 at every point of the path;
- ν_i > 0 for every type, as 1d.

With one enclosed parcel (A = T, Q = 1) and every type in the dependence form these are 1d's
rules, and 1d's rejected rows are rejected, `land`'s rules named for parcel 0.

### 3.3 Instances

All are constructed on 2026-09-27. Scalars not named are 1a's G1 (a 0.3, λ 0.05, b 0.4,
γ = η(0.2 + 0.8x), χ_max 1, (ρ, δ, J_b) = (0, 1, 1)), in Appendix B's form (the good and space,
h 1), with the good as exit good. Parcels are written (name, A, Q, access); worker types
(N, χ_max, ε, ν) as in 1d; exits (s₀, s̲, h).

**E0, 1a to 1d in parcel form.** `ParcelParams::from_workers(WorkerParams)`: one parcel
(LAND, T, 1, enclosed), every type `Dependence`, exit good 0. Every 1a-1d golden instance,
random draw, regime row and rejected row goes through it (§8, e1).

**P, the exit value alone** (closed forms): check_pinning P3's (10, 4, 6); check_enclosure's
(1.5, 0, 1) with T 100 and g_s = h_s = 1 at N 50, 60, 80; SSRN D.3's q = 10/9 at N 50; N-vi's
(1.5, 0.1, 1).

**Q, the race** (check_enclosure N-iii and SSRN D.3 inside equilibria). (LAND, 100, 1,
enclosed), no commons; one type (N, 1, 1, 1) with exit (1.5, 0, 1). In this basket (one good,
one unit of space) κ = qT/(N(1 + q)), q* = N/(100 − N), q_enc = 1.5 and N_crit = 60 hold at
every equilibrium.

| instance | N | η | result |
|---|---|---|---|
| Q1 | 50 | 2.5 | contestable, plots rented: q 1.069 ∈ [q*, q_enc) = [1, 1.5), κ 1.033: safe |
| Q2 | 60 | 2 | an enclosure tie: q = 1.5 = q_enc and κ = 1: both thresholds at once (N_crit) |
| Q3 | 80 | 2 | the same tie, κ = 0.75: the gap opens |
| Q4 | 80 | 1 | contestable, exit on the floor, q 2.890 ∈ (q_enc, q*) = (1.5, 4), κ 0.929: inside the gap |
| Q5 | 80 | 0.7 | contestable, q 4.113 > q* = 4, κ 1.0055: the floor fundable |

**Q6** (added by the verification, §12 item 18): N 14, η 1.5 and χ_max 3. The enclosure point
lies on the wall, and the economy sits at it: an enclosure tie on the wall, q = 1.5, κ 4.29.

**K, the commons.** G1's economy (FIELDS, 10, 1, enclosed), one type (4, 1, 1, 1) with exit
(0.5, 0, 0.1), so q_enc = 5:
- **K1**: a commons (WASTE, 1, 1, open), which the equilibrium does not fill: Commons, s = s₀;
- **K2**: (WASTE, 0.3, 1, open), full: Crowded, with a shadow rent, supply vertical at
  N − T_o/h = 1;
- **K3**: no commons: Enclosed, plots rented, s = s(q);
- **K4**: K1 with WASTE enclosed by law (T 11): participation rises and the wage falls
  (main.tex:385, check_enclosure N-v's sign);
- **K5**: K1 with WASTE recut as (5, 0.2, open): the same services, the same equilibrium.

**D, a dead exit.** G1 with exit (1, 0, 1): q_enc = 1 < q = 2.77 at G1, so no plot pays and every
exiter stands on the floor, which is the dependence form: G1 bit for bit (N-iv, N-vi: past q_enc
the floor is the support).

**I, idle land.**
- **I1**: 1d's W2 (N 0.25): `LaborShort` in 1d, supply saturated; idle land.
- **I2**: 1d's W3 (λ 0.6, χ_max 3): `LaborShort` in 1d, the real-wage ceiling; idle land.
- **I3**: 1d's E6 (the trained type short everywhere with the land fully used): idle land, the
  trained at the edge of its reserved shortage (1d §12 item 16).
- **I4**: W3 with exit (0.5, 0, 0.5) and parcels (FIELDS, 5, 1.5, enclosed),
  (HEATH, 5, 0.5, enclosed): idle land holds free plots; HEATH idles first.

**T, two priced types share a commons.** G1's economy with (WASTE, 0.34, 1, open); the entrant
(4, 1, 1, 1) with exit (0.5, 0, 0.1) and the trained (1, 0.8, 1.5, 1.2) with exit
(1.2, 0, 0.05), no reserved hours: Crowded with both types on plots.

**F, the fork.** 1c's M4 (the fork economy with intermediate inputs, the loom, engine and power,
ρ 0.04), one type (4, 1, 1, 1) with exit (0.1, 0, 0.05) in food. Food carries 0.6 of land per
unit, so h ≤ s₀·b̄_food = 0.06: F meets §5.4's land condition, at a ρ where the Proposition is
not proved. **F1**: with (WASTE, 1, 1, open): Commons. **F2**: without it: Enclosed.

**L, an exit good made of land alone** (added by the verification, §12 item 16). W3's economy
(λ 0.6, χ_max 3) on (LAND, 10, 1, enclosed) with Appendix B's space as exit good, which costs
r·1 and is free at r = 0. **L1**: N 5 and exit (3.2, 0, 1.8): on the wall q = 1 < q_enc = 16/9
and the type rents its plots; the one equilibrium is on the wall, and f_∞ is the wall's limit.
**L2**: N 4 and exit (0.5, 0, 1): q = 1 > q_enc = 0.5 on the wall and at its end, every exiter
on the floor s̲ = 0: W3 (I2) bit for bit on idle land.

**M, three equilibria.** (LAND, 8.6, 1, enclosed), h 1.24, a 0.075, λ 0.063, b 0.88,
γ = 1.16(0.41 + 1.88x); one type (7, 0.67, 1, 0.1) with exit (1.1, 0.17, 0.36). One equilibrium
on the all-human corner and two on the line, where f rises from f_line(0) = −2/31 to +1.017
and falls to f_line(1) = −5.48: the line's two ends have the same sign.

**Random economies** (§8, e8).

**What each instance catches.** Each is built so that a plausible wrong unit misses its goldens
by far more than the gate's tolerance (the prototype's values):

| instance | a unit that … | gets |
|---|---|---|
| K1 | charges the market rent on commons plots while the commons has room | x* 0.88948, v 0.55719, participation 0.2843, not 0.92054, 0.57346, 0.2249 |
| K2 | has no crowded regime, going from s₀ straight to s(q) | K1's wrong answer (x* 0.88948) with 0.014 of the commons idle while exit pays rent |
| K3 | leaves rented plots in production (Y = T/B^q) | x* 0.88948, Y 7.7973, not 0.88563, 7.5856 |
| Q1 | the same | x* 0.81355, Y 62.087, not 0.70580, 44.196 |
| Q2, Q3 | has no enclosure tie and bisects across the jump | f from +11.405 to −1.342 (Q2), +3.287 to −13.708 (Q3) across one double: `LaborNotCleared` |
| Q3 | values the exit in baskets (q = r/P_s) | the wall with participation 3/224, not the tie at x* 0.57644 |
| D | gives floor households a plot | 2.66 of land withdrawn from production: not G1 |
| I1-I3 | keeps 1d's `LaborShort`, or keeps rent as numeraire at r = 0 | no equilibrium, or non-finite prices |
| I4 | idles land in proportion to the parcels | FIELDS and HEATH each 39.0% used, not 52.0% and 0 |
| M | counts sign changes only at the sequence points | one equilibrium (the corner's), not three |
| L1 | reads a free exit good's 0 < 0 at r = 0 as the floor (the build's first reading, §12 item 16) | three changes of side with f_∞ +2.03, not the wall at v 10.086 |
| E0 | changes any 1d operation's order | 1d's goldens off in the last bits (§8, e1) |

## 4. Equations

Per period. x is a threshold on the contestable line, τ a technique, i a worker type, j a
category, z a parcel.

### 4.1 Land

    T = Σ_{z enclosed} A_z·Q_z,   T_o = Σ_{z open} A_z·Q_z                    main.tex:692, :804
    T = T_m + T_p + T_idle,       T_o = T_oc + T_oi

With scarce land (r = 1) T_idle = 0. The rent schedule per acre: an enclosed parcel in use,
r·Q_z; an idle parcel, 0 (SSRN A.1: "unused fixed inputs have zero rent"); an open parcel, 0,
with the shadow rent r_o·Q_z when occupied, which nobody receives. Land in use is filled best
first: the enclosed parcels by Q_z descending with T_m + T_p, the open ones with T_oc (§2.1).

### 4.2 The exit value alone

    s(q) = max(s₀ − q·h, s̲),   q_enc = (s₀ − s̲)/h                            main.tex:370, :858
    ds/dq = −h below q_enc, 0 above;   s₀ − s(q) = q·h below, h·q_enc above   check_enclosure N-ii, N-vi

and, for coverage, with a basket of g_s units of the exit good and h_s of space
(main.tex:841-848; SSRN eq 16, 28):

    κ = rT/(N·P_s) = qT/(N(g_s + q·h_s)),   q* = N·g_s/(T − N·h_s)  (T > N·h_s)
    N_crit = q_enc·T/(g_s + q_enc·h_s)                                         main.tex:862; N-iii

The race: independent exit outlasts the fiscal replacement iff q_enc ≥ q*, iff N ≤ N_crit.

### 4.3 Participation

§2.3's rule for every type:

    n_S,i = N_i·F_i(ln1p((v_i − e_i)/(ν_i·P_s + e_i))),   E_i = N_i − n_S,i

with e_i = 0 for the dependence form. For a priced type at plot rent r_o:
- **plot branch** when r_o·h_i < p_g·Δ_i (q_o < q_enc,i): s_i = s₀,i − q_o·h_i,
  e_i = p_g·s₀,i − r_o·h_i, and its E_i exiters each take a plot of h_i;
- **floor branch** otherwise: s_i = s̲_i, e_i = p_g·s̲_i, no plot. At equality the two values
  agree, and the type counts as on its floor.

Where v_i ≤ e_i nobody of the type works (F_i clamps at 0). In an economy with a priced type
every type is pooled (§2.9): v_i = ε_i·v and S = Σ_i ε_i·n_S,i.

### 4.4 Exit land

At given prices the land exit plots ask for is

    G(r_o) = Σ_{i ∈ 𝒫 on the plot branch at r_o} h_i·E_i(r_o)

nonincreasing in r_o: a dearer plot lowers e_i, raises supply and so lowers E_i, and past
p_g·q_enc,i the type leaves plots for the floor. At a point with market rent r = 1:

| regime | condition | r_o | T_oc | T_p |
|---|---|---|---|---|
| **Unused** | G(0) = 0 | 0 | 0 | 0 |
| **Commons** | 0 < G(0) ≤ T_o | 0 | G(0) | 0 |
| **Crowded** | G(r) < T_o < G(0) | where G crosses T_o, in (0, r) | T_o | 0 |
| **Enclosed** | G(r) ≥ T_o, G(0) > 0 | r | T_o | G(r) − T_o |

In the Crowded regime, where the crossing is a drop of one type's plot demand at
r_o = p_g·q_enc,i, that type splits between commons plots (T_o − G_{−i} of land) and its floor,
and its supply is the same either way. With one plot-taking type its supply is vertical at
N_i − T_o/h_i there. At r = 0 (§4.6) plots are free everywhere: r_o = 0,
T_p = (G(0) − T_o)⁺ on idle enclosed land. That regime is **Idle**, not Enclosed, since
suitable land still idles (§0.2; §12 item 17). Where the exit good is free at r = 0 the regime
is decided at the wall's end (§4.6).

### 4.5 Quantities and the market's land

1d §4.4 with the market's land in place of T:

    T_m = T − T_p  (r = 1),   Y = T_m/B^q_s,   n_D = Y·(H_ŷ + machine hours per basket)
    f = n_D − S                                             SSRN eq 11 with T_m: n_D = T_m·L^q_s/B^q_s

In a priced economy D_i = 0; in a dependence economy 1d §4.3-4.4 unchanged, with this Y.

### 4.6 The idle stretch

At the wall's end, under 1d's last technique τ_e, with r = 0 and the pool's wage as numeraire
(w = 1): every price is its labour total, p_k = λ̃_k, p_j = λ̃_j (plus reserved costs in wage
units), P_s = L_s (plus reserved costs); q = 0, r_o = 0, s_i = max(s₀,i, s̲_i). Here q = 0 is the
limit of r/p_g at the wall's end, where p_g grows with the wage. An exit good that embodies no
labour (λ̃_g = 0, as Appendix B's space) is free at r = 0, p_g = 0, and the limit is instead
q = 1/b̃_g: the plots are then decided as on the wall's last piece, in its units, while every
exit value in money is p_g·s_i = 0 (§12 item 16). The market's land in use T_m runs over
(0, T_m,∞], where

    T_m,∞ = T − (G(0) − T_o)⁺

and the excess demand is f(T_m) = n_D(T_m) − S. At T_m,∞ this is 1d's f_∞, the wall's end
(unit-1d.md §4.5) with the plots' land: the stretch starts where the wall ends.
- Without walled types (every priced economy, and dependence ones without reserved hours) S is
  fixed and f linear in T_m: the equilibrium is Y* = S_∞/L^q (L^q the pool's hours per basket),
  T_m* = Y*·B^q.
- With walled types, D_i = Y·R_ŷi, 1d's walk in wage units, and f nondecreasing in T_m
  (1d's Lemma 2' along Y): bisection, with 1d's edge of a reserved shortage where it closes on a
  short point (I3).

At the equilibrium T_idle = T − T_m − T_p > 0 (0 at the junction, f_∞ = 0), and the rent income
is 0 (SSRN App. C). Real quantities are as at any equilibrium; the real wage is at 1d's ceiling
ω_∞ = 1/ρ_∞.

### 4.7 Enclosure points and ties

For a type i ∈ 𝒫 the enclosure point is where p_g = p*_g,i = r·h_i/Δ_i, that is q = q_enc,i.
Along the path p_g rises (strictly where λ̃_g > 0), so there is at most one:
- on the all-human corner, p_g = v·L̄_g + b̄_g (x = 0, all tasks human; L̄_g with the
  human-required hours, 1d's `l_bar`): v_e = (p*_g − b̄_g)/L̄_g when it lies in (0, v(0)];
- on the line, x_e is the largest double with p_g(x) < p*_g, in the region where it lies, by
  bisection (as 1c finds a switch point);
- on a wall piece under τ: v_e = (p*_g − b̃_g,τ)/λ̃_g,τ when it lies in the piece.

The point has two values. **Below**: the type on its floor at the market rent, T_p = (G_{−i} −
T_o)⁺, G_{−i} the other types' plots. **Above**: the type renting, T_p = (G_{−i} + h_i·E_i −
T_o)⁺. S is the same on both sides, since the type's exit value is the same on both branches
there. If f changes side between them, the equilibrium is an **enclosure tie**: f is linear in
T_p, so

    T_m* = S/(n_D per unit of market land) = S·B^q/L^q,   T_p* = T − T_m*
    ψ = (T_p* + T_o − G_{−i})/(h_i·E_i)                                   the renting share

It is evaluated at x_e (or v_e), with the type's supply from its floor value. The prices at a
tie are fixed by q(x_e) = q_enc alone: Q2 and Q3 share x_e, v and P_s = 5/3, and N moves only
ψ and quantities.

### 4.8 Outputs and identities at the equilibrium

1d §4.7's outputs and identities, with r·T_m for 1d's T wherever rent income enters:
- **Income** (SSRN App. C): I = Σ_i v_i·hours_i + r·T_m + interest = v·n_D + Σ_i v_i·D_i + r·T_m
  + interest = Y·P_s = Σ_j p_j·z_j·Y; pᵀf equals the same; provider baskets
  (r·T_m + interest)/P_s − ν; `funded` as 1d.
- **The cost system**: 1d's rows; T_m = b^qᵀy; eq 11 for the market, Y = T_m/B^q_s and
  n_D = T_m·L^q_s/B^q_s.
- **Land**: T = T_m + T_p + T_idle, T_o = T_oc + T_oi; each regime's condition (§4.4); at an
  enclosure tie q = q_enc,i to rounding and ψ ∈ [0, 1].
- **Exit per type**: its branch and its q_enc,i; e_i and s_i; each supply equal to §4.3's
  formula; E_i; its plot land on the commons and rented.
- **The home account**: home output Σ p_g·s₀,i per plot household; the rent in kind r·T_p; the
  floor Σ p_g·s̲_i per floor household; the commons' shadow rent r_o·T_oc. None enters I.
- **Coverage**: κ = r·T/(Σ_i N_i·P_s) (SSRN eq 16 with the whole enclosed rent base); in
  Appendix B's basket form, one unit of the exit good and h of space, κ = qT/(N(1 + q·h))
  exactly; κ = 0 at r = 0.
- **The margin**: on the line v = γ(x*)·π; at the wall and on the idle stretch x* = 1 (at r = 0
  every delivered machine cost is below labour's at every task); at the all-human corner
  v ≤ γ(0)·π.
- **Numeraire**: at r = 1 prices are per unit of rent (1a-1d's); at r = 0 per unit of the
  pool's wage. v/P_s, x*, Y, hours, participation, T_m, T_p and T_idle do not depend on it.
- **Flags**: `funded` as above; `lemma_b1` as 1d's, with T_m at x = 1 in place of T.

### 4.9 The race in equilibrium (Q)

In Q's closure the race of main.tex:854-864 plays out along the automation path (η falls, p_g
falls, q rises). At N 50, q* = 1 < q_enc = 1.5: at Q1 κ ≥ 1 while plots are still rented. At
N 60, q* = q_enc: at Q2 the enclosure tie has q = 1.5 and κ = 1 exactly. At N 80,
q_enc = 1.5 < q* = 4: at Q3 the tie has κ = 0.75; at Q4 exit is on the floor, the dependence
form, and κ = 0.929; only at Q5, past q* = 4, is κ ≥ 1. The equilibria satisfy κ = qT/(N(1 + q))
exactly, and each type's branch flips exactly at q_enc.

## 5. Solution method

### 5.1 The evaluation (normative)

**At (x, τ) on the line** (r = 1), a corner's (x, v, τ), or the idle stretch's (T_m, τ_e) with
w = 1 and r = 0:
1. γ, J, the machine block, categories and quantities per basket: 1d §5.1 steps 1-4. At r = 0
   the wage-given system runs with every land right-hand side and every b_j multiplied by
   r = 0.0, so the rent-unit code is unchanged.
2. In an economy with a priced type, the exit sub-problem at the point's prices: p_g is the
   exit good's price from step 1. For a trial r_o, per type in index order: the branch
   (r_o·h_i < p_g·Δ_i, Δ_i stored at construction); e_i (p_g·s₀,i − r_o·h_i by a fused
   multiply-add on the plot branch, p_g·s̲_i on the floor); n_S,i = N_i·F_i(ln1p((ε_i·v − e_i)/
   (ν_i·P_s + e_i))); and G += h_i·(N_i − n_S,i) on the plot branch, G from 0.0. Then the regime
   of §4.4: G(0) = 0 is Unused; G(0) ≤ T_o is Commons; else G(r) ≥ T_o is Enclosed with
   T_p = G(r) − T_o; else Crowded, with r_o by bisection on the bit patterns of [0, r] (the least
   double with G(r_o) ≤ T_o) and the supplies at that r_o. At r = 0, r_o = 0 and
   T_p = (G(0) − T_o)⁺.
3. T_m = T − T_p (scarce; T − 0.0 is T), or the stretch's T_m (idle); Y = T_m/B^q; n_D as 1d.
4. In a dependence economy, 1d §5.1 steps 5-8 with this Y, the supply as
   N_i·F_i(ln1p((v_i − e_i)/(ν_i·P_s + e_i))) with e_i = 0.0: (v_i − 0.0)/(ν_i·P_s + 0.0) is 1d's
   v_i/(ν_i·P_s) bit for bit, the product ν_i·P_s formed first.
5. f = n_D − S.

At an enclosure point the evaluation takes type i's branch and plot land from its side (below,
above, or the tie's ψ) instead of step 2's test.

With one enclosed parcel (A = T, Q = 1.0), no commons and every type in the dependence form,
steps 2 and 3 are exact no-ops (T = 0.0 + T·1.0, T_p = 0.0) and step 4 is 1d's, so every point
is 1d's bit for bit.

### 5.2 At construction

1d §5.2, then: T and T_o (sums in parcel order from 0.0); Δ_i, q_enc,i, p*_g,i and 𝒫; b̄_g and
L̄_g (the exit good's land and labour through the chain at x = 0); ℓ₀ = (L̄_ŷ + L^H_ŷ)/B_ŷ; the
certification flags of §5.4; the per-parcel order for the best-first report.

### 5.3 The solve

1. **Not viable.** 1d's test, unchanged (decision 64's convention).
2. **The sequence.** 1d §5.3 step 2's values in path order, with:
   - each enclosure point (§4.7) on the all-human corner, the line and the wall, as a pair of
     values (below, above), in path order among the switches;
   - f_∞ (§4.6), then the idle stretch's end, −S_∞ (0 when S_∞ = 0);
   - in an economy with 𝒫 non-empty, `EXIT_SCAN` interior points in every piece between
     consecutive sequence points of the all-human corner, of the line on [lo, 1] and of the
     wall: equally spaced in x on the line, in ω on a corner piece, and on the bit patterns of ω
     where a wall piece ends at ω = +∞. The [0, lo] stretch and the idle stretch are not
     scanned (§5.4).
3. **Sides.** v → 0 is positive: ν_i > 0 drives every supply to 0 while the market keeps land
   (§3.2), so n_D(0) > 0 = S. 1d's rules otherwise (f_line(1) ≥ 0 positive; an exact zero at a
   junction is the junction), and:
   - an enclosure point's values are on the positive side when > 0, like a switch's;
   - f_∞ is positive when > 0 and negative when ≤ 0: an exact zero is the junction of the wall
     and the idle stretch, the equilibrium at r = 0 with no land idle (1d item 17's rule is
     subsumed: a wall piece that starts at an exact zero is still the first change);
   - the idle end is negative, or positive when S_∞ = 0 (no production is no equilibrium).

   Each change of side is one equilibrium. None is `SolveError::NoMarket { f_end }`, which
   needs S_∞ = 0: no one works even at the ceiling of the real wage. More than one is
   `SolveError::MultipleEquilibria { sign_changes, switches }`, the count over the whole path.
4. **Location** of the one equilibrium:
   - inside a piece: bisection on the piece's own bracket, the ends of the piece, never the scan
     cell (1a's arithmetic bisection on the line, bit patterns on [0, lo] and in ω at the
     corners). With one change the piece's ends have opposite sides, so the root's bits do not
     depend on `EXIT_SCAN`;
   - between a technique switch's two values: 1c's tie, its closed form with T_m (spill does not
     depend on σ, since prices and so supply do not), or 1d's bisection on σ with a walled type;
   - between an enclosure point's two values: the enclosure tie (§4.7);
   - between f_∞ and the idle end: the idle equilibrium (§4.6);
   - 1d's edges of reserved shortages, unchanged, also on the idle stretch.
5. **Report** (§4.8), the finiteness check of every output, and 1a's labour net on the pool.

**Which equation fixes which unknown.**
- The envelope fixes the technique; the task margin fixes v on the line; the pool's clearing
  fixes ω at a corner, T_m on the idle stretch, and σ or ψ at a tie.
- Participation per type fixes n_S,i given prices and r_o.
- The commons fixes r_o (Crowded), or its occupancy (Commons); enclosed land fixes T_p
  (Enclosed) and so T_m; land clearing fixes Y.
- The rent is the numeraire (r = 1) while enclosed land is fully used, and 0 once it idles.
- Walras gives the income identity and the basket count, which are checked.

### 5.4 Uniqueness

**Lemma 5 (the supply slope along the path).** Let v rise along the path (x rising on the line;
x fixed at a corner). Then, at a fixed plot rent r_o,

    d(P_s/v) = −B_s·dv/v²,   d(p_g/v) = −b̃_g·dv/v²,   d(r_o/v) = −r_o·dv/v²

with B_s and b̃_g the price-side land totals at the point, and a type's supply is nondecreasing
along the path where σ_i ≥ 0 and nonincreasing where σ_i ≤ 0 (strictly so inside F_i's
support), where

    σ_i = ν_i·B_s·(ε_i·v − e_i) + (s₀,i·b̃_g − r_o·h_i)(ν_i·P_s + ε_i·v)      plot branch
    σ_i = ν_i·B_s·(ε_i·v − e_i) + s̲_i·b̃_g·(ν_i·P_s + ε_i·v)                  floor branch
    σ_i = ν_i·B_s·ε_i·v                                                     dependence

*Proof.* At a corner prices are affine in v with the point's totals. On the line the assignment
minimizes cost at every point, so a price moves with the wage by its labour total, dp = λ̃·dv
(the envelope property: the marginal task costs the same by hand or machine), and p = v·λ̃ + b̃
gives d(p/v) = −b̃·dv/v². Differentiating ln((ν_i·P_s/v + ε_i)/(ν_i·P_s/v + e_i/v)) gives σ_i/v³
times a positive factor. ∎

The prototype checked σ_i's sign against finite differences on the line, the wall and the
all-human corner of K1, K3 and M (33 points, both signs).

So a dependence type and a floor type always supply more along the path, and so does a
plot-taking type wherever its plot, at the rent it pays, is no more land-intensive than the
market's version of its exit goods, r_o·h_i ≤ s₀,i·b̃_g. Elsewhere its supply can fall: the plot
gets cheaper relative to the wage faster than the goods do, against the support term
ν_i·B_s·(ε_i·v − e_i). On the commons with room (r_o = 0) it always supplies more.

**Proposition 5 (certified economies).** At ρ = 0, let every plot-taking type satisfy
h_i ≤ s₀,i·b̄_g and h_i·ℓ₀ ≤ ε_i, and let at most one type take plots. Then f is nonincreasing
along the whole path, and the equilibrium is unique if it exists. (Only the crowded commons
needs the last condition; the Commons and Enclosed regimes allow any number of plot-taking
types.)

*Proof.* b̃_g is nondecreasing along the path at ρ = 0 (within a technique machine tasks add
land; at a switch the type above has more land per delivered task, 1c's Proposition), so
b̃_g ≥ b̄_g and every σ_i ≥ 0. n_D per unit of market land, L^q/B^q, is at most ℓ₀ (1c's Lemma 2,
the Proposition at switches, constant on corners). In the Enclosed regime
f = (T + T_o − Σ h_i·N_i)·L^q/B^q − Σ_i n_S,i·(ε_i − h_i·L^q/B^q), which falls. In the Commons
regime T_m = T and S rises. In the Crowded regime the one plot-taking type's supply is fixed and
the others rise. f jumps down at an enclosure point (plots appear, T_m falls, S does not move)
and at a ρ = 0 switch (1d's Proposition, T_m continuous), and it is linear and decreasing on the
idle stretch. ∎

F meets the land conditions, but has interest (ρ 0.04), where the Proposition is not proved
(with interest b̃_g can fall at a switch); its f is nonincreasing on the prototype's grid of
every stretch. The other priced instances fail them (Appendix B's good carries no land of its
own, b̄_g = 0), and are unique on the prototype's scans. With several plot-taking types a crowded
commons can shift its occupants toward types that supply less per unit of plot, so S can fall
while the commons stays full; not proved.

**Measured.** The prototype's scratch draws (one priced type, one machine type, ρ = 0, no
commons; a scan of 60 points per stretch at 25 digits), by support and the exit good's land:

| support ν | exit good | draws | three equilibria | certified, all unique |
|---|---|---|---|---|
| 1 | land-free | 270 | 0 | — |
| 1 | with direct land | 120 | 0 | 86 of 86 |
| 0.2 | land-free | 150 | 5 | — |
| 0.05 | land-free | 150 | 14 | — |
| 0.05 | with direct land | 150 | 2 | 107 of 107 |
| 0 (refused, §2.4) | land-free | 120 | 22 with two changes, 26 with none | — |

In the land-free sets the excess demand rose within some stretch in 37-53% of draws, most often
on the all-human corner, where a plot-taker's supply falls with the wage whenever b̄_g = 0. With
support 1 it still crossed zero once. **Phase 2**: the default form with a land-extensive exit
and little support has multiple equilibria often, and decision 70 refuses them.

**The idle stretch** is monotone (§4.6), and the [0, lo] stretch spans 10^-12 of the line, so
neither is scanned.

### 5.5 Precision

Everything in unit-1a.md §4, 1b §5.4, 1c §5.6 and 1d §5.5 carries over, with the dependence
economies' points bit for bit. New:
- **The crowded commons.** r_o is resolved to adjacent doubles, or to the rounding of G over
  its slope; S carries that times ∂S/∂r_o. With one plot-taking type S is N_i − T_o/h_i to
  rounding, whatever r_o's error.
- **Near q_enc.** e_i = p_g·s₀,i − r_o·h_i cancels as the plot's net value goes to the floor;
  with the fused multiply-add it keeps 2^-53·p_g·s₀,i absolute. The branch test compares
  r_o·h_i with p_g·Δ_i, both products correctly rounded, so a type within an ulp of its q_enc is
  on either branch with the same supply to rounding.
- **Enclosure ties.** x_e is the largest double below the crossing, so every output carries
  p*_g's rounding over p_g's slope; ψ carries f's rounding over the jump |f_below − f_above|
  (12.7 at Q2, 17.0 at Q3). At a wall's enclosure point v_e carries the cancellation in
  p*_g − b̃_g.
- **The idle stretch.** T_m* in closed form keeps full precision; with walled types, bisection to
  adjacent doubles and 1d's edge statement. Without walled types prices at r = 0 are the labour
  totals, at full precision; with them P_s is the walk's fixed point and carries its
  conditioning, one ulp of T_m moving P_s by up to about 1e-9 relative on the verification's
  draws (§12 item 19). f_∞ is 1d's value.
- **The scan.** Where §5.4's certification fails, a pair of equilibria inside one scan cell
  (width 1/`EXIT_SCAN` of its piece) is not seen: the count is then two short and names the
  wrong equilibrium. M's pair is 0.36 apart on the line. The build measures the misses against a
  scan 16 times finer on e8's draws.
- **Decidability.** 1d's statement extends to the new values: an enclosure point's or a scan
  point's f within rounding of 0, and a regime boundary (G(0) or G(r) within rounding of T_o),
  are not decidable in f64; every output is continuous across a regime boundary.
- **Technique ties** keep 1c's and 1d's statements (STATE O18): the split and each type's
  quantities carry γ_i's error amplified by the slope over the jump; with rented plots the jump
  also moves T_p, which does not depend on σ.
- **To be measured** by the build: the f64 values against every golden.

## 6. Result type

Proposed; the build may rename, and records any departure in §12.

- `src/exit.rs`: `PricedExit { gross, floor, plot }` with `value(q)` (s(q)), `threshold()`
  (q_enc, `None` when the plot never pays or h = 0) and `take(q)`; `coverage(q, land, people,
  goods, space)`, `coverage_threshold(land, people, goods, space)` (q*, `None` when
  T ≤ N·h_s) and `crowding_limit(q_enc, land, goods, space)` (N_crit), each validating its
  arguments with `ParamError`.
- `src/parcels.rs`: `Access { Enclosed, Open }`; `Parcel { acreage, quality, access }`;
  `ExitForm { Dependence, Priced(PricedExit) }`; `ParcelParams<S>` (1d's `WorkerParams` less
  `land`, plus `parcels`, `exits`, `exit_good`) with `ParcelParams::from_workers(WorkerParams)`
  (E0); `ParcelEconomy<S>` with `new`, `at(x)`, `at_with(x, τ)`, `at_wage(x, v, τ)`,
  `at_idle(t_m, τ)`, `at_enclosure(point, side)`, `solve() -> Result<Regime<Eq1e>,
  SolveError>`, and the x-free `enclosed_land()`, `commons()`, `enclosure_targets()` and
  `certified()`.
- `LandMarket { Scarce, Idle }`, `ExitLand { Unused, Commons, Crowded, Enclosed }` (the build
  adds `Idle`, §12 item 17),
  `Branch { Dependence, Plot, Floor }`, `EnclosureTie { worker, share }`.
- `Eq1e`: 1d's `Eq1d` fields in the numeraire's units (§4.8), plus `land_market` (which also says
  the numeraire), `rent` (1 or 0), `exit_land`, `plot_rent` r_o, `q`, `exit_good_price` p_g,
  `land: LandEq { enclosed, commons, market, rented_plots, idle, commons_occupied }`,
  `parcels: Vec<ParcelEq { rent_per_acre, shadow_rent_per_acre, used }>`, `coverage` κ,
  `enclosure: Option<EnclosureTie>`, `home: HomeAccount { output, rent_in_kind, floor,
  commons_shadow_rent }`, `workers: Vec<WorkerEq1e>` (1d's `WorkerEq` plus `branch`,
  `exit_value` e_i, `exit_goods` s_i, `threshold` q_enc,i, `exiters`, `plot_land`,
  `rented_land`), `f_end` as `Option<f64>` (`None` where the wall's end is short, as at I3),
  `scan_points` (how many the count used) and `residuals: Residuals1e` (1d's,
  plus `land` = |T − T_m − T_p − T_idle|/T and `commons` = |G − T_o|/T_o when Crowded, 0
  otherwise; each 0 in E0 form, the old ones 1d's bit for bit).
- `Eq1e::outputs()`: 1d's keys, then 1e's economy keys, then `parcel<z>.<field>`, and the new
  fields under `worker<i>.`. 1c's `Item` gains `Parcel(z)`.
- `SolveError` gains `NoMarket { f_end }`. `Regime::Interior`'s documentation says that
  in 1e it also holds the idle-land equilibrium and the enclosure tie.
- `EXIT_SCAN` is a named constant with its reason; the draft proposes 256.
- No new `core::num` function: `fma` and `ln1p` are there.

## 7. Goldens

`goldens/generate_1e.py` computes every golden with mpmath at 70 digits from §4's equations. Its
`Economy` extends `generate_1d.py`'s (as 1d's extended 1c's): the parcels, the exit forms, the
exit sub-problem with the Crowded r_o by bisection to 2^-400, the idle stretch, the enclosure
points and ties in closed form, and the count by a scan four times the oracle's. It writes
`goldens/goldens_1e.txt` with 30 significant digits and a header of FNV-1a digests of
`generate_1e.py`, `generate_1d.py`, `generate_1c.py`, `generate_1b.py`, `generate.py` and its
own body; the Rust constants in `tests/gate/goldens_1e.rs` carry 20 digits (`e10`). Key
prefixes: `P_`, `Q1_` to `Q6_`, `K1_` to `K5_`, `D_`, `I1_` to `I4_`, `T_`, `F1_`, `F2_`, `M_`,
`A1_` to `A4_` (the wrong units of §3.3), and `L1_` (added by the verification, §12 item 16).

The generator asserts as it goes:
- E0 form of every 1d golden instance equals `generate_1d.py`'s solve to 1e-65, and its
  `LaborShort` rows become idle-land equilibria with f_∞ equal to 1d's excess; Priced(0, 0, h)
  equals the dependence form wherever there are no reserved hours; D equals G1;
- every identity of §4.8 at 1e-65, each regime's condition, each type's branch and supply;
- κ = qT/(N(1 + q)) at every Q equilibrium, and each branch flipping at q_enc;
- Lemma 5's sign against a finite difference at every point of a grid on every stretch of every
  instance, and f nonincreasing on the grid of every certified instance;
- the count equal to each instance's result on a scan of 1024 points per piece;
- the closed forms of P and of §4.6 and §4.7 against bisection.

The prototype's values, to 20 significant digits:

**P, the exit value alone.** P3 (10, 4, 6): q_enc 1; s at q = 1/3, 2/3, 1, 2, 5 is 8, 6, 4, 4, 4.
check_enclosure (1.5, 0, 1), T 100, g_s = h_s = 1: q_enc 1.5, N_crit 60, q* 1, 1.5 and 4 at
N 50, 60 and 80. SSRN D.3: κ(q = 10/9, N 50) = 20/19. N-vi (1.5, 0.1, 1): the take 1 at q = 1,
1.4 at q = 10, the cap h·q_enc = 1.4.

**Q, the race.**

| | x* | v | P_s | q | κ | Y | n_a | T_p | provider |
|---|---|---|---|---|---|---|---|---|---|
| Q1 | 0.70579667000425371499 | 1.2650754686042984246 | 1.9354049316010742152 | 1.0690557278636135043 | 1.0333754799030553077 | 44.196148236528373027 | 15.689297662407120614 | 34.310702337592879386 | −16.059145251807696265 |
| Q2 | 0.57643645580848169258 | 0.83440886282971784135 | 5/3 | 3/2 | 1 | 53.055826834488699395 | 24.35371364352931986 | 31.894576449496718503 | −19.136745869698031102 |
| Q3 | 0.57643645580848169258 | 0.83440886282971784135 | 5/3 | 3/2 | 3/4 | 70.741102445984932527 | 32.471618191372426481 | 9.1927685993289580039 | −25.515661159597374802 |
| Q4 | 0.73337119933110234855 | 0.47630601193448258737 | 1.3460534470801926986 | 2.8897270304267912443 | 0.92864068860820635539 | 82.867398902794997659 | 24.236410321153980672 | 0 | −5.708744911343491569 |
| Q5 | 0.79095995667054840485 | 0.34757985454775912373 | 1.2431319882017187786 | 4.1129923190951419075 | 1.0055247647582589377 | 85.956760819946089544 | 19.723809903767576589 | 0 | 0.44198118066071501502 |

Q2's renting share ψ is 0.89475173179455385939, its f below +11.405132530402671205 and above
−1.3415681745269859419; Q3's ψ 0.19341640193734186373, below 3.2872279825595645849, above
−13.708372957346644944. Participation at both ties is 0.40589522739215533101. In Q1 every
exiter rents (T_p = E = 34.31, h = 1); in Q4 and Q5 every exiter is on the floor.

**K, the commons.**

| | regime | x* | v | P_s | q | Y | n_a | r_o | T_oc | T_p |
|---|---|---|---|---|---|---|---|---|---|---|
| K1 | Commons | 0.9205384426937588319 | 0.57346088900036187046 | 1.365887351684820653 | 2.7330816312595874696 | 7.6988590039587485682 | 0.89940595044089532838 | 0 | 0.31005940495591046716 | 0 |
| K2 | Crowded | 0.90729619707291098295 | 0.56651394336138253463 | 1.3650328667538438818 | 2.7394793485112111898 | 7.7408494701591972669 | 1 | 0.43259131873953323891 | 0.3 | 0 |
| K3 | Enclosed | 0.88563381350455919867 | 0.55517399235015002504 | 1.3634532149772006247 | 2.7513857596849979067 | 7.5856235387047286096 | 1.1335047027580957161 | 1 | 0 | 0.28664952972419042839 |
| K4 | Enclosed | 0.89830323382720585152 | 0.56180260690877409395 | 1.3644044502663948906 | 2.7442035882628716126 | 8.3244976566862513306 | 1.1453288958098360554 | 1 | 0 | 0.28546711041901639446 |

K5 is K1 (5 × 0.2 is 1.0 in f64 too), WASTE used 0.31005940495591046716 of its services.
Participation runs 0.2249 (K1), 0.25 (K2), 0.2834 (K3), 0.2863 (K4) while v falls: enclosure
manufactures labour supply, by price and by law.

**D.** G1's values bit for bit: x* 0.86315041816243703192, v 0.54343596069677832842, Y
7.8806055249729076768, n_a 1.3433818800977173461; q 2.7656715745563858098 > q_enc = 1; Enclosed,
T_p 0.

**I, idle land** (prices per unit of the pool's wage).
- I1 (W2): Y 35/6, T_m 47/6 (T_idle 13/6), n_a 0.25, P_s 3/70, v/P_s 70/3 (1d's ceiling);
  f_∞ = 13/188, 1d's excess.
- I2 (W3): Y 2.7997929961450740717, T_m 3.7597220233948137534, n_a 1.4398935408746095226
  (= (4/3)·ln(53/18)), P_s 18/35, v/P_s 35/18; f_∞ 2.3898936931679436689, 1d's excess.
- I3 (E6): the trained at its edge, Y 25/6 (its reserved demand 0.12·Y = N_T), T_m 20/3, κ_T
  6.5281716214787166668, trained wage 42.473262323990767457 pool wages, P_s
  5.4217914788788920949, entrant hours 65/48, participation 0.21813725490196078431.
- I4 (W3 priced, two parcels): Y 1.7485871603090923406, T_m 2.3481027581293525717, free plots
  1.5503633016348048267, n_a 0.89927339673039034661, participation 0.22481834918259758665;
  FIELDS used 0.51979547463522098645 of its 7.5 services, HEATH 0.

**T.** Crowded: x* 0.86506676292016438128, v 0.54443518165663000746, P_s 1.361745269288688487,
Y 7.8745487927760029095, n_pool 1.3282197600553773998, hours 0.9858901199723113001 and
0.22821976005537739981, exiters 3.0141098800276886999 and 0.77178023994462260019, r_o
0.5283717704572644768 (0.1·3.0141 + 0.05·0.7718 = 0.34).

**F** (engine technique). F1, Commons: x* 0.92359857105587254217, v 0.14300589383721726198, P_s
1.5513223353284310685, p_food 0.66813076665541326948, Y 6.4987458559929469679, n_a
0.18404732476679986349, T_oc 0.19079763376166000683. F2, Enclosed: x* 0.8404787252281769818, v
0.13267354180711575492, P_s 1.5509189905174605692, p_food 0.66802617267694203727, Y
6.3866171288999906867, n_a 0.2852262490184478048, T_p 0.18573868754907760976.

**M.** `MultipleEquilibria` with 3 changes of side: the all-human corner at v
0.2029756936648907802, the line at x 0.0044736729880859003628 (v 0.47752981945473418015) and at
x 0.36831293358199533671 (v 1.332677398275829529); f_line(0) = −2/31, f_line(1)
−5.4802545807212104398.

**A, the wrong units of §3.3.** A1 (K1 with the market rent on commons plots) and A2 (K3 with
rented plots left in production) both give x* 0.88948157717185110772, v 0.55718604886354229323,
participation 0.28427070795822870941; A2's Y is 7.7972976856809805896. A3 (Q1, plots left in
production): x* 0.81354993934237555025, Y 62.086713726091331904. A4 (Q3, the basket as
deflator): the wall, participation 3/224, Y 12.5.

## 8. Tests

The groups are the modules `e1_nesting`, `e2_exit_value`, `e3_race`, `e4_commons`,
`e5_idle_land`, `e6_types_and_fork`, `e7_multiplicity`, `e8_random_parcels`,
`e9_regimes_and_validation` and `e10_goldens_file` of `tests/gate/`, cited as `e1::` to `e10::`.
The gate maps onto them:
- the idle-margin regime and the enclosed regime recognised and solved: `e4::*` (Commons,
  Crowded, Enclosed), `e5::*` (idle land), `e3::*`, and in every golden test the regime labels;
- check_enclosure's worked instance reproduced: `e2::race_closed_forms`, `e3::*`;
- both exit forms nest, 1a-1d exactly: `e1::*`; the dependence form keeps 1a's and 1d's gates.

Identities and goldens at 1e-12 relative (ADDENDUM A7) unless stated; "bitwise" means `to_bits`
equality. A shared check (`support_1e::check_identities_1e`) runs §4.8's identities and bounds at
every equilibrium and pins every price, P_s, v, each e_i and each supply to `at_with`, `at_wage`,
`at_idle` or `at_enclosure` at the equilibrium bit for bit.

**e1, nesting (§2.12).**
- `e1::every_1d_golden_instance_is_bit_identical`: 1a's 27 instances, 1b's, 1c's and 1d's golden
  instances through `from_workers`: `Interior`, every shared output and `bisection_steps` bit
  for bit; `land_market` Scarce, `exit_land` Unused, the new residuals 0. [§5.1]
- `e1::random_economies_are_bit_identical`: 1a's G5, 1b's C5, 1c's m6 and 1d's d7 draws: every
  1d equilibrium bit for bit; every 1d `LaborShort` an idle-land equilibrium; every 1d
  `MultipleEquilibria` still one, with the count 1d's or 1d's plus one where f_∞ > 0. [§2.12]
- `e1::labor_short_rows_are_idle`: 1d's W2, W3, E6 and its `LaborShort` draws: `f_end` bitwise
  1d's `excess`. [§4.6]
- `e1::refusals_nest`: `NotViable` with `d_at_1` bit for bit; rejected rows rejected, `land`'s
  rules named for parcel 0. [§3.2]
- `e1::points_nest_on_a_grid`: `at(x)` equals 1d's `at(x)` bit for bit on 1b's grid. [§5.1]
- `e1::exit_option_off`: every E0 instance without reserved hours with every type
  `Priced(0, 0, h)` (h 0 and 1): bit for bit the dependence result, no scan. [§2.3]
- `e1::a_dead_exit_is_dependence`: D, G1 bit for bit with `exit_land` Enclosed and every exiter
  on the floor. [check_enclosure N-iv, N-vi]
- `e1::a_floor_without_a_plot_is_priced`: G1 with exit (0, 0.5, 0.1), no plot but not the option
  switched off: K1's equilibrium, not G1's. [§2.3; §12 item 18]
- `e1::the_exit_good_is_any_category`: G1, W2 and B1 without exit values and with every exit
  good: 1d's outputs unchanged, p_g and q the exit good's, q = 1 at W2's idle equilibrium with
  space. [§2.5; §12 items 16 and 18]

**e2, the exit value alone (P).**
- `e2::p3_floor`: P3's q_enc = 1 exactly, s weakly decreasing on (1/3, 2/3, 1, 2), 4 at q 2 and 5.
  [check_pinning.py:131-145; main.tex:370]
- `e2::slope_and_cap`: ds/dq = −h below q_enc (exact on a grid of dyadic q), 0 above; the take
  q·h below, h·q_enc above. [N-ii, N-vi]
- `e2::race_closed_forms`: q_enc 1.5, N_crit 60, q* 1, 1.5, 4 at N 50, 60, 80, κ(10/9, 50) = 20/19,
  q* `None` at N 120 (T ≤ N·h_s); κ increasing in q toward T/(N·h_s). [N-iii; main.tex:841-862;
  SSRN eq 28]
- `e2::identical_workers_ignore_support`: K3's economy with χ_max = 10^-12, through `at_wage` at
  wages either side of e: the type supplies N or 0 as v ≷ e, for ν ∈ {0.1, 1, 10}: the support
  cancels. [N-iv; P5; main.tex:396]
- `e2::bad_arguments`: every helper refuses non-finite or out-of-range arguments.

**e3, the race (Q).**
- `e3::race_goldens`: Q1-Q5, x*, v, P_s, q, κ, Y, n_a, T_p, provider baskets; `funded` false at
  Q1-Q4 and true at Q5. [§4.9]
- `e3::coverage_identity`: κ = qT/(N(1 + q)) at each, and κ ≥ 1 exactly when q ≥ q*. [SSRN eq 16,
  28]
- `e3::branch_flips_at_q_enc`: every exiter rents below q_enc (Q1), none above (Q4, Q5).
  [main.tex:375-381]
- `e3::the_tie_is_enclosure_by_price`: Q2 and Q3 are enclosure ties with q = 1.5 to rounding,
  sharing x_e, v and P_s bitwise; ψ's goldens; κ = 1 at Q2 (N_crit) and 0.75 at Q3 (the gap).
  [main.tex:854-864; §4.7]
- `e3::an_enclosure_tie_on_the_wall`: Q6's goldens, the point on the wall at the equilibrium's
  v bitwise, ψ the root between its two values. [§4.7; §12 item 18]
- `e3::two_enclosure_points_in_one_piece`: Q's economy with two types, q_enc 1.5 and 2: both
  points on the line in path order, and the first type's tie at Q2's x_e. [§4.7, §5.3; §12
  item 18]

**e4, the commons (K).**
- `e4::commons_goldens`: K1-K4 goldens and regimes; K1's s = s₀ with the commons partly idle
  and its idle parcels at zero rent; K2's r_o golden and n_a = 1 (N − T_o/h); K3's T_p and
  T_m = T − T_p. [Prop exit (i), (ii); check_enclosure N-i; §4.4]
- `e4::enclosure_manufactures_labour`: participation strictly rises and v strictly falls from K1
  to K2 to K3, and from K1 to K4 (enclosure by law with T rising 10%). [main.tex:376 (iii), :385;
  N-v]
- `e4::quality_is_efficiency`: K5 equals K1 bit for bit; per-parcel rent per acre r·Q_z; the
  commons' shadow rent per acre r_o·Q_z at K2. [§2.1; main.tex:692]
- `e4::rented_plots_leave_production`: at K3, Y·B^q = T − T_p, and A2's value is not reached.
  [main.tex:380; §2.6]

**e5, idle land (I).**
- `e5::idle_goldens`: I1-I4, `land_market` Idle, rent 0, prices per pool wage, v/P_s at 1d's
  ceiling; T_idle > 0; rent income 0. [SSRN A.1, App. C; check_enclosure N-i; §4.6]
- `e5::the_wall_at_zero_rent`: at I1-I4 x* = 1 and every delivered machine cost below labour's at
  every task. [§2.8]
- `e5::worst_parcels_idle_first`: I1's LAND used 47/60 of its services; at I4 HEATH idles
  while FIELDS is used 0.5198. [§2.1]
- `e5::edge_on_the_idle_stretch`: I3, the trained's hours N_T and its κ_T golden. [1d item 16]
- `e5::no_market`: I4 with exit (2, 0, 0.5), whose exit life is worth more than the ceiling wage
  (p_g·s₀ = 2·18/35 > 1 pool wage at r = 0), returns `NoMarket`. [§5.3]
- `e5::a_free_exit_good_is_decided_at_the_wall`: L1's goldens on the wall, f_∞ the wall's limit
  with the plots free on idle land (`Idle`, q = 1); L2 is W3 bit for bit. [§4.6; §12 item 16]
- `e5::idle_goldens` also reads `exit_land`: Unused at I1-I3, `Idle` at I4. [§12 item 17]

**e6, types and the fork (T, F).**
- `e6::two_types_share_a_commons`: T's goldens; the plots sum to T_o; one r_o for both types. [§4.4]
- `e6::fork_goldens`: F1, F2; food's price through the chain; the cost system over every row with
  T_m. [§4.5]
- `e6::certified_is_monotone`: F's f nonincreasing on a 256-point grid of every stretch. [§5.4]
- `e6::certification_needs_both_conditions`: F at ρ 0 certified, not with ε below h·ℓ₀; ℓ₀
  equal to n_D per unit of market land at x = 0 with B's common hours. [§5.4; §12 item 18]
- `e6::the_all_human_corner_takes_the_cheapest_type`: G1 with a second, labour-intensive
  machine type, on the all-human corner under the type cheapest at its wage, not the
  envelope's first. [SSRN A.1; §12 item 18]

**e7, multiplicity and the scan (M).**
- `e7::three_equilibria`: M is `MultipleEquilibria` with 3 changes of side; with the scan off (a
  crate-private switch) it would count 1. [§2.11, §5.3]
- `e7::supply_slope`: Lemma 5's σ_i against a finite difference at grid points of K1, K3 and M,
  both signs. [§5.4]
- `e7::root_does_not_depend_on_the_scan`: K3 and Q1 solved with `EXIT_SCAN` 256 and 17 give the
  same equilibrium bit for bit. [§5.3 step 4]

**e8, random economies.** SplitMix64 seeded 941 to 946 (1d's were 931-935), and 947-948 for the
sets (g) and (h) the verification added (§12 item 18). 1d's d7 table for the
scalars, segments, categories and machine types, and:

| draw | range |
|---|---|
| parcels | one enclosed parcel with A ~ U(2, 12), Q 1; half the time a second, A ~ U(1, 5), Q ~ U(0.2, 2); a commons with probability 0.65, A·Q ~ U(0.02, 1.5) |
| exit | priced with probability 0.7 (sets b, c); s₀ ~ U(0.05, 1.5), s̲ 0 or U(0, s₀) with equal odds, h ~ U(0.01, 0.4) |
| support | 1, or U(0.05, 2), with equal odds |
| exit good | category 0; in the verification's sets (g) and (h), uniform among the categories |

Sets of 60 equilibria each: (a) one priced type, K = 1, ρ = 0; (b) 1-3 types, some in the
dependence form, K in 1..3; (c) ρ ~ U(0, 0.1); (d) 1d's d7 draws with reserved hours, the
dependence form, for the idle stretch and its edges; (e) certified draws, the exit good given
direct land ≥ h/s₀; (f) enclosure ties, N set so that the equilibrium lies in an enclosure
point's jump; and, added by the verification (§12 item 18), (g) as (a) with the exit good drawn
among the categories and (h) as (f) on (g)'s draws. A 30-digit scratch run of 100 draws in each of (a)-(c) (a scan of 30 per stretch)
gave: (a) 91 solved: 55 on the line (15 Commons, 8 Crowded, 31 Enclosed of which one a tie, 1
Unused), 28 on the wall (11, 3, 14), 6 on idle land, 2 on the all-human corner (one a tie); 6
not viable, 3 rejected by a stability rule this draft does not keep; f rose within the
all-human corner in 20; (b) 93 solved, 18 of them dependence-only; (c) 94 solved and one with
three equilibria. The build records its own tallies (`MAX_DRAWS_1E`). For each:
- `e8::identities`: §4.8's identities, the regime conditions and bounds, with every residual of
  1d bounded, no task type cheaper, and each category's wage floor, φ_w and φ_r with the land
  at the rent (§12 item 18). [§4.8]
- `e8::residuals_recompute`: each residual equals its recomputation bit for bit, and each is
  nonzero somewhere. [§6]
- `e8::count_against_a_fine_scan`: the count equals a scan 16 times finer, or the miss is recorded;
  set (e) is unique, and f nonincreasing along its whole path (64 points per piece, the idle
  stretch included). [§5.4, §5.5]
- `e8::supply_slope`: Lemma 5's sign at the equilibrium and at the scan's points. [§5.4]
- `e8::the_draws_cover_the_regimes`: every exit-land regime (`Idle` among them), both land
  markets, every margin, enclosure ties on the line, the wall and the all-human corner,
  `NoMarket`, and `MultipleEquilibria`, each asserted; and in sets (g) and (h) exit goods other
  than category 0 on the wall, at enclosure ties (on the wall among them) and free on idle land.

**e9, regimes, validation and reductions.**
- `e9::exact_zeros`: G(0) = T_o exactly (the Commons-Crowded boundary), G(r) = T_o exactly
  (Crowded-Enclosed), f_∞ = 0 exactly (the idle junction, T_idle 0), and a type exactly at its
  q_enc (on its floor). [§4.3, §4.4, §5.3]
- `e9::validation`: every rule of §3.2, and −0.0 stored as +0.0.
- `e9::permutation`: parcels reversed (the same equilibrium within 1e-12, the per-parcel outputs
  permuted), worker types reversed.
- `e9::units`: land service in units of 1/c (every Q_z, every direct land requirement b_j and
  machine b, and every h_i times c): the same x*, Y, hours and participation, every price and v
  times c (the rent per unit is 1/c of what it was), q over c and κ unchanged; bit for bit at
  c = 4.
- `e9::enclosure_by_law_moves_services`: switching an open parcel to enclosed changes T and T_o by
  its A·Q exactly.

**e10, the goldens file.** `goldens_1e.txt`'s six digests recompute, and the Rust constants match
it to 20 digits.

**Unit tests**: the exit sub-problem (each regime, a split at a q_enc drop, r_o's bisection on bit
patterns); `PricedExit`; the enclosure tie's closed form against bisection; the idle stretch's
closed form against bisection; the scan's grid on bit patterns to +∞.

## 9. Out of scope, and where each goes

- **Several non-produced services** (SSRN A.1's vector r; land classes such as arable, sites and
  coalfields, PLAN §3.1): open question 12. Under a fixed basket and fixed recipes only one
  service binds generically and the others idle at zero rent, so the relative rents need
  substitution, 1f's price-responsive baskets, or a technique choice between classes.
- **A Ricardian working cost per acre**: open question 1.
- **The priced form with reserved tasks**: open question 8.
- **The pure main.tex form without support** (ν_i = 0): open question 4.
- **Production on the commons**: open question 2.
- **A working life's land** h_w (dynamics P1, check_dynamics.py:399-404): 1f's household.
- **A quality threshold of suitability** (parcels of positive quality on which s₀ is still not
  attainable): not in the sources beyond Prop exit's word; quality 0 is the only unsuitable land.
- **Dated switches of access** (PLAN §3.5, Turner's acreage): the tape's; the oracle solves each
  status as given.
- **Who funds the floor** s̲ (main.tex:383) and the support's source when plots are rented in
  kind: 1f, with transfers and government; check_enclosure N-iv's transfer u that "becomes the
  commons", and N-v's linear instance, are reproduced only in sign (e4).
- **Coverage as a fiscal instrument** (τ_R = 1/κ, SSRN eq 16's funding): 1f.
- **check_pinning A-joint** (w > s(q) in a Cobb-Douglas closure): 1f's household (decision 75).
- **A dump interface for parcels**: there is no other solver to compare against.

## 10. Pitfalls

- **Three idle margins.** The commons with room, idle enclosed land at zero rent, and the floor
  are different (§0.2). "Enclosed" means no suitable land idles, not that no parcel is open.
- **q is rent over the exit good's price**, not over P_s, and not over the wage.
- **Coverage is not funding.** κ counts the whole enclosed rent base; `funded` counts the
  provider's market budget. Rented plots pay in kind (Q1: κ 1.033, the provider 16 baskets
  short).
- **At r = 0 the numeraire is the pool's wage.** A consumer comparing prices across solves must
  read `land_market`.
- **The market's land is not T.** Y = T_m/B^q, and T_m < T whenever plots are rented or land
  idles.
- **An enclosure point is not a technique switch.** It moves the market's land, not prices; its
  tie's share ψ is of one type's exiters, not of machine tasks.
- **A falling supply is not an error.** Under the priced form it is Lemma 5's plot channel; the
  count handles it by the scan.
- **Quality is efficiency.** A_z·Q_z is what counts; which parcel idles is a reported convention.
- **Notation.** Q_z is quality, not q; A_z acreage, not 1c's matrices; ψ the renting share, not
  1c's σ; r_o the plot rent, not interest ρ; h_i the plot, not Appendix B's space h; E_i exiters,
  unrelated to 1d's E instances; T_o the commons. "P", "Q", "K", "D", "I", "T", "F", "M" instances
  are unrelated to 1a's or 1d's letters.

## 11. Open questions

As built, 1e takes the first choice on each, and STATE.md records each as a decision open to veto,
numbered from 147 (unit 1d's proposals are 135-146). The ones marked **Phase 2** bind it. Questions
14 and 15 came from the verification (P1.11).

1. **Parcels as efficiency units** (§2.1; proposed decision 147): rent r·Q_z per acre, the worst
   idle first by convention. Alternative: a Ricardian working cost per acre, with a margin at
   positive rent. **Phase 2**: the tape's parcel quality must scale its service, and comparisons
   should use totals, not which parcel idles.
2. **The commons is for exit only** (§2.2; proposed 148). Alternative: production on open land.
3. **One participation rule for both forms**, SSRN eq 8 with the exit life's yield (§2.3; proposed
   149). **Phase 2**: under the default form a pop's participation share is
   F(ln((ν·P_s + w)/(ν·P_s + p_g·s(q)))), and its s(q) uses the rent its plot actually pays.
4. **Support stays positive** (§2.4; proposed 150). Alternative: main.tex's pure form with ν = 0,
   which needs a regime for surplus labour at every wage and has multiple equilibria often.
5. **The exit good is one category** (§2.5; proposed 151). Alternative: a bundle. **Phase 2**: the
   tapes name it (food).
6. **Plots rented on enclosed land leave production, and the home account is in kind** (§2.6;
   proposed 152). Alternatives: plots outside the land market (the land is then counted twice);
   rents paid in money from home goods sold (which changes goods clearing).
7. **Idle land at zero rent with the pool's wage as numeraire; `NoMarket`; no `LaborShort`**
   (§2.8; proposed 153). **Phase 2**: an idle-land instance is compared in wage units.
8. **No reserved tasks with the priced form** (§2.9; proposed 154). **Phase 2**: the eras' trained
   type at its wall under the default exit form needs a 1e addendum (the fixed point in Y with
   walled types); until then such an economy takes the dependence form.
9. **The count scans where supply can fall** (§2.11; proposed 155), exact where certified, resolved
   to `EXIT_SCAN` elsewhere, multiple equilibria refused. Alternative: refuse uncertified
   economies, which refuses Q and K (Appendix B's good carries no land). **Phase 2**: the default
   form has multiple equilibria in a few per cent of economies with little support and
   land-extensive exits; decision 70 (refuse, or report all) now matters for the historical runs.
10. **The commons clears by a shadow rent that nobody receives** (§2.7; proposed 156).
    Alternative: congestion that lowers each plot's yield.
11. **The enclosure tie is `Interior` with `enclosure` set** (§2.10; proposed 157), as decisions 62,
    69 and 136.
12. **Several non-produced services are not in 1e** (§9; proposed 158). **Phase 2 and later**: the
    tape's land classes need an addendum, or 1f's substitution, before a region has two scarce
    classes.
13. **The random draws' ranges** (§8, e8; proposed 159): the build tunes and records them.
14. **A free exit good at r = 0 is decided at the wall's end** (§4.6; the verification, §12 item
    17; proposed 160): q = 1/b̃_g, its limit there. Alternative: §4.6 read literally, every type
    with Δ_i > 0 on a plot at q = 0 whatever p_g, which breaks the junction when the type is on
    its floor, or crowded out, on the wall's last piece. **Phase 2**: the tapes' exit good is
    food, which embodies labour; an idle-land instance whose exit good is made of land alone
    decides its plots at the wall's end.
15. **Free plots on idle land are `ExitLand::Idle`** (§4.4; the verification, §12 item 17;
    proposed 161). Alternatives: `Commons` (Prop exit (i)'s meaning, but the plots are on
    enclosed parcels), or `Enclosed` with a convention (the build's first reading, which
    contradicts §0.2). **Phase 2**: a consumer reads `Enclosed` as "closed by price, no suitable
    land idle", and `Idle` as plots free on idle enclosed land at r = 0.

## 12. Changes during the build (P1.10) and the verification (P1.11), 2026-09-27

1. **The generator builds on 1d's.** `generate_1e.py`'s `Economy` extends `generate_1d.py`'s,
   as the prototype did: the parcels, the exit forms, the exit sub-problem, the idle stretch, the
   enclosure points and ties, the scan, the report and its identities. An economy without exit
   values is solved by `generate_1d.py`'s own solve, with −S_∞ after the wall's end: its
   `LaborShort` rows become the idle equilibrium (or the junction where f_∞ = 0), and its count
   gains one where f_∞ > 0. `goldens_1e.txt` records six digests, and `e10` checks them. It
   writes 208 goldens in about a minute and reproduces every value of §7 to the digits shown
   (223 since the verification added Q6 and L1, items 16 and 18).
   Departures from §7: the crowded commons' plot rent is not bisected to 2^-400 but found by the
   Illinois variant of regula falsi to working precision, between the rents where a type's plot
   demand drops, which are located in closed form (a split there is exact); the values are
   those of the bisection to 30 digits, and the run is four times faster. The scan's 1024 points
   per piece are evaluated at 30 digits, since they decide signs only. The E0 nesting runs on
   fifteen of 1d's instances (W1-W4, G1, B1, B2, E1, E3, E5, E6, E7, F1, X1, J1), within 1.6e-71,
   not on every 1d golden instance, and the exit option switched off on G1, W1, W2 and B1 with
   h 0 and 1; `e1` runs every instance and draw in f64. Lemma 5's sign is checked at 292 grid
   points (5 of them negative, all on M). No golden instance is certified (F has ρ > 0, the rest
   an exit good without land), so the generator's monotonicity check for certified instances is
   vacuous; `e6` checks F and a certified variant of it in f64.
2. **Validation order.** The parcels are checked in 1d's place of `land`, first, since 1d's
   checks need T; §3.2 put them after 1d's. The rest is §3.2's order: 1d's checks on T, the exit
   forms, the exit good, the priced form with reserved hours, land for every plot. A parcel's
   parameters are `Item { kind: "parcel" }` named `acreage` and `quality`; T out of range, T_o
   above `SCALE_CEIL` and too little land for every plot are `Invalid { name: "parcels" }`.
3. **Result types** (§6). `Eq1e` holds 1d's outputs as `base: Eq1d` rather than flattening
   them, and `Eq1e::workers[i]` (`WorkerEq1e`) only the exit's fields: the rest of each type is
   `base.workers[i]`. `WorkerEq1e` gains `plot_households` (the exiters on plots, which the home
   account needs), and `Eq1e` gains `certified`. `Residuals1e` has `partition`
   (|T − T_m − T_p − T_idle|/T, §6's `land`, renamed because 1d's residuals already have one) and
   `commons`; 1d's are `base.residuals`. `f_end` is `Eq1e::f_end: Option<f64>`, while
   `base.f_end` keeps 1d's +∞ where the wall's end is short. The economy's new output keys are
   `land_market` (0 scarce, 1 idle), `rent`, `exit_land` (0-3), `plot_rent`, `q`,
   `exit_good_price`, `land_enclosed`, `land_commons`, `market_land`, `rented_plots`, `idle_land`,
   `commons_occupied`, `coverage`, `enclosure`, `enclosure_worker`, `enclosure_share`,
   `home_output`, `home_rent_in_kind`, `home_floor`, `home_commons_shadow_rent`, `scan_points`,
   `certified`, `res_partition` and `res_commons`; then `parcel<z>.{rent_per_acre,
   shadow_rent_per_acre, used}` and `worker<i>.{branch, exit_value, exit_goods, threshold,
   exiters, plot_households, plot_land, rented_land}`. Beside §6's API there are `ParcelPoint`
   (1d's point on the market's land with the exit sub-problem's results), `EnclosurePoint`
   (worker, margin, x, v, technique) and `EnclosureSide::{Below, Above, Share(ψ)}` for
   `at_enclosure`, `enclosure_points()`, `at_idle_edge`, `thresholds()`, `plot_takers()`,
   `certification_totals()` (b̄_g and ℓ₀) and `workers()`. The crate-private switch of §8 e7 is
   the public `solve_scanned(scan)`, documented for the gate: `solve()` is `solve_scanned(EXIT_SCAN)`,
   and 0 is 1a-1d's count. `PricedExit::{value, threshold, take}` validate the form and the
   argument and return `Result`; `PricedExit::validated()` checks the fields.
4. **A free good at zero rent** (not in the draft). On the idle stretch a category made of land
   alone, as Appendix B's space, costs nothing, and its real wage, wage floor, φ_w and φ_r are
   undefined: `Eq1e::outputs` reports them absent. A machine type whose recipes use no labour is
   free too, and with it the pool's wage in machine-task units g and the economy's φ_w and φ_r
   (one of d7's draws). Residuals relative to a price use the absolute error where the price is
   0 (`workers::relative`), which leaves 1d's operations unchanged wherever prices are positive.
5. **Enclosure points** are located by bisection on the bit patterns of the path's own
   parameter (x on the line, ω = v/P_s at a corner) for the branch test itself, r·h_i ≥ p_g·Δ_i,
   rather than §4.7's closed forms on the corners, so the point is the largest double at which
   the type is on its floor, and the two sides agree with the natural evaluation around it.
   Below is the natural evaluation at that double; Above forces the type onto a plot at every
   r_o ≤ r; Share(ψ) is §4.7's tie. At the wall's end the type rents when the exit good's labour
   total is positive (q → 0), else its branch does not change on the last piece. The [0, lo]
   piece is not searched, and an enclosure point exactly at a switch point is read as the
   switch: neither occurs in the gate or the draws.
6. **A crowded commons' split.** Where the crossing of G and T_o is a drop of one type's plot
   demand, the bisection's final two doubles differ in that type's branch: its plots are
   T_o − G_{−i}, its households those over h_i, its supply the floor's (equal to the plot's to
   rounding). Plot land is shared between the commons and enclosed land in proportion
   (`rented_land` = plot land·T_p/G), a convention like §2.1's.
7. **Two solve paths.** An economy without exit values (every type in the dependence form, or
   priced with s₀ = s̲ = 0) runs 1d's own path and location (`WorkerEconomy::path` and `locate`,
   1d's corner functions `supply_at` and `wage_at`), with −S_∞ appended and the junction rule of
   §5.3; the rest of 1e reports on it. An economy with exit values has no reserved hours (§2.9),
   and evaluates the whole point at each ω of a corner (v = ω·B_s/(1 − ω·L_s)), since the plots
   and so the market's land move with the wage. The idle stretch's end is the evaluation at
   T_m = 0, whose f is exactly −S_∞.
8. **The idle stretch with walled types** is bisected on the bit patterns of T_m. A bracket that
   closes on a short point is the edge of a reserved shortage (I3), as in 1d. One that closes on
   the walk's ceiling, a jump to +∞ that is not a reserved shortage, is refused with 1d's error
   for the same event on the line (`NonFinite { what: "n_D - n_S beside a change of side" }`):
   one of 1d's 300 d7 draws (set (b), draw 75, `LaborShort` in 1d) does this. It is recorded,
   not solved; the same limit holds in 1d on the line and in a tie's σ.
9. **The wall's last piece at ω = +∞.** Where the basket embodies no labour at the wall's end
   (L_s = 0), ω_∞ = +∞ and the scan runs on bit patterns up to 2^512 (`OMEGA_SCAN_TOP`), above
   which ω·B_s and the prices it sets can overflow while f is within rounding of f_∞. One of e8's
   draws needs it.
10. **Unit 1d's code changed its API, not its behaviour.** `evaluate` is split into `price_side`
    (at a rent) and `finish` (on a `Market`: rent, T_m and each type's exit value); `report` into
    `report` and `report_at`; `solve` into `path`, 1d's sides and `locate`; `tie_share` and the
    tie's mix take a market; the supply is `workers::supply` with the exit value; `Corner`,
    `corner`, `supply_at`, `wage_at`, `omega_end`, `cheapest_at`, `short_type` and
    `edge_clearing` are crate-visible; `Context` carries the market's land at x = 1 for
    Lemma B.1's flag. `MachineEconomy::clear_with(task, T_m)`,
    `MachineBlock::prices_at_wage_and_rent`, `Item::Parcel` and `SolveError::NoMarket` are new.
    With r = 1.0, T and every e_i = 0.0 each operation is 1d's (v − 0.0 = v, ν·P_s + 0.0 = ν·P_s,
    1.0·b = b), and 1a-1d's 298 tests pass unchanged; d7's module exposes its draws to e1 and e8.
11. **Tests.** Unit 1e adds 9 unit tests (2 in `exit.rs`, 7 in `parcels.rs`) and 46 gate tests in
    e1-e10: 353 in the package (76 unit, 276 gate, 1 doc). `e1`'s random nesting, over 1423 of
    1a-1d's draws: 1172 the same equilibrium bit for bit, 222 of 1d's `LaborShort` on idle land
    with 1d's f_∞, 21 `NotViable` in both, 7 `MultipleEquilibria` (1d's count, plus one where
    f_∞ > 0) and item 8's one refusal. `e9::exact_zeros` builds the commons' two boundaries and a
    type at its q_enc at a point of the line rather than at an equilibrium, and the junction at
    an equilibrium (f_∞ = 0 exactly with supply saturating between x = 1 and the wall's end). §8's
    `e2::identical_workers_ignore_support` reads K3's wall at wages either side of the exit value.
12. **The random draws** (§8 e8; decision 159). Seeds 941-946 (947-948 for the verification's
    sets (g) and (h), item 18), 60 equilibria per set, the
    scalars, segments, categories and machine types from d7's table (2-3 categories and a site,
    category 0 the exit good), and §8's parcels, exits and supports. Set (d) reads d7's draws
    with reserved hours in parcel form; set (e) sets the exit good's direct land to U(1, 2)·h/s₀
    and keeps certified draws; set (f) draws as (a) and sets N midway between the two values at
    which f vanishes on either side of the first enclosure point. The tallies:

    | set | draws | invalid | line | wall | all-human | idle | Commons | Crowded | Enclosed | Idle | Unused | ties | other |
    |---|---|---|---|---|---|---|---|---|---|---|---|---|---|
    | (a) one type | 69 | 6 | 22 | 38 | 0 | 9 | 25 | 10 | 22 | 2 | 1 | 1 (wall) | 1 NoMarket, 2 NotViable |
    | (b) types | 79 | 18 | 37 | 21 | 2 | 7 | 14 | 5 | 27 | 4 | 10 | 0 | 1 NotViable |
    | (c) interest | 67 | 6 | 43 | 17 | 0 | 2 | 7 | 13 | 29 | 1 | 10 | 0 | 1 NotViable |
    | (d) reserved | 63 | 0 | 25 | 34 | 1 | 14 | 0 | 0 | 0 | 0 | 60 | 0 | 3 NotViable |
    | (e) certified | 74 | 12 | 22 | 38 | 0 | 9 | 20 | 5 | 30 | 5 | 0 | 0 | 1 not certified, 1 NotViable |
    | (f) ties | 297 | 0 | 14 | 43 | 3 | 0 | 0 | 0 | 60 | 0 | 0 | 60 (14 line, 43 wall, 3 all-human) | 236 without a point, 1 MultipleEquilibria |
    | (g) exit goods | 70 | 6 | 17 | 43 | 0 | 8 | 19 | 8 | 27 | 5 | 1 | 1 (line) | 1 NoMarket, 1 MultipleEquilibria, 2 NotViable |
    | (h) their ties | 681 | 0 | 15 | 42 | 3 | 0 | 0 | 0 | 60 | 0 | 0 | 60 (15 line, 42 wall, 3 all-human) | 621 without a point |

    (line, wall and all-human count the margins, idle the equilibria on idle land among them;
    the exit-land columns Enclosed and Idle were one column, Enclosed, before item 17.)
    The count against a scan 16 times finer, on the first 30 draws of each set with a plot-taking
    type, found no miss, and no certified draw has more than one equilibrium. No e8 draw has an
    equilibrium at the edge of a reserved shortage on the idle stretch; I3 is the gate's. The
    verification's own 240 draws with reserved hours in parcel form put 53 on the idle stretch,
    6 of them at such an edge, all in agreement (item 19).
13. **Numbers.** The oracle's f64 values match the goldens within 2.4e-14 relative on Q5's
    provider baskets (80.44 − 80, a cancellation), 7.0e-15 on Q2's f above its enclosure point
    (a difference), 5.1e-15 on K2's shadow rent (resolved to adjacent doubles), within 2.1e-15 on
    ψ, T_p, T_idle and the rest, and within 1.0e-15 on x*, v, P_s, Y and N_a.
14. **Mutation.** 49 mutants of `parcels.rs`, `exit.rs`, the changed lines of `workers.rs` and
    `machine_block.rs`, each run against the package's tests: the branch test at equality, the plot
    value without its rent, the floor value, the regimes' boundaries, the rented land without the
    commons, the crowded bisection's side, the split's land, rented plots left in production,
    quality ignored, the land-for-every-plot rule, the junction and exact-zero rules of the sides,
    the idle closed form and its edge, the scan switched off, the tie's share, the floor test and
    the wall's end branch at an enclosure point, coverage, q, the best-first fill, the home
    account, the rented share, `f_end`'s option, the idle prices with rent, Above unforced, the
    exit values ignored, the certification, the tie's forced share, idle land counting plots, the
    reserved-hours rule, the supply's exit value, income and provider baskets without the rent,
    the land residual and the quantities on T, the direct land at r = 0, the tie on T, Lemma B.1's
    flag on T, the price guard, the wage ceiling with land, the machine land at r = 0, and five of
    `exit.rs`'s formulas. The first run left nine survivors; seven are now killed by new tests
    (`e9::saturated_knife_edges` for the junction's start rule and f(1) = 0 on the positive side of
    a priced economy, `e9::lemma_b1_uses_the_market_land`, the rented share in `e4`, two plot takers
    in `e6`, and check_identities_1e's res_land, wage ceiling, rented-share and Lemma B.1 checks).
    Two are equivalent: the idle end's side at S_∞ = 0 in an economy without exit values (some
    type has ε > 0 and a finite support, so S_∞ > 0 always), and the closed form of a technique
    tie on T in place of T_m (σ = B_a·f_a/(B_a·f_a − B_b·f_b) is unchanged when both B's are scaled
    by T/T_m; only the rounding differs).
15. **Decisions.** The build takes §11's first choice on each question, as proposed decisions
    147-159 for STATE.md at the unit's close, open to veto: 147 parcels as efficiency units, 148 the
    commons for exit only, 149 one participation rule for both forms, 150 support kept positive, 151
    one exit good, 152 rented plots leaving production with the home account in kind, 153 idle land
    at zero rent with the pool's wage as numeraire and `NoMarket`, 154 no reserved tasks with the
    priced form, 155 the scan with multiple equilibria refused, 156 the commons' shadow rent, 157 the
    enclosure tie inside `Interior`, 158 one land service, 159 the draws of item 12. Those that bind
    Phase 2 are 147 (compare totals, not which parcel idles), 149 (the agents' participation rule
    under the default form), 151 (the tapes name the exit good), 153 (idle-land instances compared
    in wage units), 154 (a trained type at its wall needs a 1e addendum before it can take the
    default form), 155 (multiple equilibria in the historical runs where support is low) and 158
    (several land classes). Item 4's free goods at zero rent, reported absent, and item 8's
    refusal at the walk's ceiling bind Phase 2 too: an idle-land instance for the agents should
    not rest on either. The verification (P1.11) adds proposed decisions 160, a free exit good
    at r = 0 decided at the wall's end (item 16), and 161, free plots on idle land labelled
    `Idle` (item 17); both are §11's questions 14 and 15, and both bind Phase 2.

**The verification (P1.11, 2026-09-27).** One adversarial pass (an independent derivation at
40-50 digits, and 30 mutants) and one fix round. Items 16 and 17 change the oracle; items 18
and 19 add tests and amend the text.

16. **A free exit good at r = 0 is decided at the wall's end** (derivation finding 1). At r = 0
    the exit good's price is its labour total, so an exit good that embodies no labour, as
    Appendix B's space or any land-only category, costs 0 there and q = r/p_g is 0/0. The build's
    branch test r_o·h_i < p_g·Δ_i read 0 < 0 and put every such type on its floor at r = 0, while
    on the wall's last piece q = 1/b̃_g and the type could rent: the idle stretch did not start
    where the wall ends, and L1 (N 5, exit (3.2, 0, 1.8) in space; §3.3) was refused as three
    equilibria with f_∞ = +2.03 against the wall's limit −0.176. Section 4.6's own reading, a plot
    at q = 0 whatever p_g, repairs L1 but breaks the junction the other way wherever the type is
    on its floor, or crowded out of the commons, on the wall's last piece (h_i ≥ b̃_g·Δ_i). So
    at r = 0 with p_g = 0 the exit sub-problem is decided where the limit lies: in the wall's
    end's units, where the exit good costs p_wall = b̃_g per unit of rent (its price at x = 1
    with v = 0 and r = 1, which every point of the last piece repeats bit for bit), trial rents
    in [0, 1], the branch test h_i·ρ < p_wall·Δ_i and the goods s₀ − (ρ/p_wall)·h_i, while
    every exit value in money is p_g·s_i = 0 and every rent reported in money is 0. The reported
    q at r = 0 is the limit, 1/p_wall (0 where the exit good embodies labour, as before), for
    economies without exit values too. The junction then holds by construction: L1 solves on
    the wall at v 10.086 (the derivation's value), f_∞ is the wall's limit within e/v far up
    the last piece, and L2 (exit (0.5, 0, 1), on the floor at q = 1) is W3 bit for bit. With
    the frame, a commons can be crowded at r = 0 (a unit test builds it). `generate_1e.py`
    decides the same way and writes L1's six goldens; its other 208 are unchanged. An exit good
    with labour, food among them, is decided as before, bit for bit. Proposed decision 160.
17. **Free plots on idle land are `ExitLand::Idle`** (derivation finding 2). At r = 0 the build
    labelled plots past the commons `Enclosed`, which §0.2 defines as no suitable land idle and
    Prop exit (i) calls a commons. They are now `Idle` (code 4): at r = 0, G ≥ T_o puts the
    spill free on idle enclosed land. `Enclosed` occurs only at r = 1. I4 is `Idle`, and
    `e5::idle_goldens` asserts it; the other idle instances are `Unused`. The generator's regime
    names follow. Proposed decision 161.
18. **Tests for the mutants that survived** (mutation findings, and the minor findings). Of the
    pass's 30 mutants, 13 survived; each is now killed by a test that fails with it:
    - A4, the exit-free test without its floor clause: `e1::a_floor_without_a_plot_is_priced`
      (G1 with exit (0, 0.5, 0.1) is K1's equilibrium, not G1's);
    - B4, two enclosure points out of path order: `e3::two_enclosure_points_in_one_piece`;
    - B6, the all-human corner under the envelope's first type:
      `e6::the_all_human_corner_takes_the_cheapest_type`, and `check_identities_1e` now checks
      that no task type is cheaper than the technique at every 1e equilibrium;
    - C1, C4 and C5, three of 1d's residuals with land priced at 1 on idle land:
      `check_identities_1e` bounds every field of 1d's residuals at every 1e equilibrium (e5's
      idle instances catch C1 and C4; C5 needs a machine that operates on land, which e1's and
      e8's idle draws have);
    - C2 and C3, φ_r and the wage floor with land priced at 1 on idle land:
      `check_identities_1e` pins each category's wage floor, φ_w and φ_r to their formulas with
      the land at the rent, and φ_w + φ_r = 1, wherever the price is positive (e5 catches C2;
      C3 needs a priced category with chain land, which e1's and e8's idle draws have);
    - D4 and D5, certification's second condition and ℓ₀'s common hours:
      `e6::certification_needs_both_conditions`;
    - I4, I5 and I7, the exit good's index: e8's sets (g) and (h) (seeds 947-948) draw the exit
      good among the categories, the land-only site included, and ties on them; the regimes test
      asserts exit goods other than category 0 on the wall (34), at ties (33, 26 on the wall) and
      free on idle land (2); `e1::the_exit_good_is_any_category` runs the p_g identity on
      economies without exit values whose exit good is not category 0.

    Two fixed goldens stand where the gate had only random tallies: L1 is `Enclosed` on the
    wall, and Q6 (the race with N 14, η 1.5 and χ_max 3; §3.3) an enclosure tie on the wall,
    tested by `e3::an_enclosure_tie_on_the_wall`. Building Q6 showed that `generate_1e.py`
    placed an enclosure point at the midpoint of its last bracket, where the type can already
    rent, so that its value below was the renting one; it now takes the bracket's floor end, as
    the oracle takes the largest double on the floor, which moves no earlier golden.
    `check_identities_1e` also checks each priced type's exit goods s_i against its branch and
    q_o, the branch at r = 0 in the wall's units (item 16), q at r = 0, and `Enclosed` only at
    r = 1. `e8::the_draws_cover_the_regimes` asserts what §8 promised and did not check: ties on
    the wall and the all-human corner, `NoMarket` and `MultipleEquilibria`, and now `Idle`.
    `e8::count_against_a_fine_scan` checks set (e)'s f nonincreasing along its whole path
    (§8), and `e1::random_economies_are_bit_identical` asserts item 11's tallies exactly. Six
    mutants of the fixes are killed too: the frame off, the spill labelled `Enclosed`, q = 0 at
    idle, a rent charged in money in the frame, the frame's rent 0, and every free-good type on a
    plot (the reading of §4.6 that item 16 declines). The package now has 361 tests (77 unit,
    283 gate, 1 doc); unit 1e 63 (10 unit, 53 gate).
19. **Two statements amended** (the minor findings). Item 12 said no draw with reserved hours
    reaches the edge of a reserved shortage on the idle stretch; e8's do not, but the
    verification's 240 such draws put 6 there, all agreeing with its solver, and one draw,
    ill-conditioned (P_s about 4.5e4 pool wages on a pool of 0.0087 hours, one ulp of κ moving f
    by 7.3e-9 of the pool), was refused by the labour net at 5.0e-9: the net working, not a
    wrong number. §5.5 said prices at r = 0 are the labour totals at full precision; with walled
    types P_s is the walk's fixed point instead (one ulp of T_m moved it by 4.5e-10 on the
    derivation's draw b035), and §5.5 now says so.
