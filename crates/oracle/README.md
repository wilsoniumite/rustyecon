# rustyecon-oracle

Dated 2026-09-25; joined the workspace on 2026-09-26 (P1.1). Units 1b and 1c were added on
2026-09-27, on branch `phase1`: 1b at P1.2, verified at P1.3; 1c at P1.4, verified at P1.5
and P1.6. Both were closed at P1.7, the same day. Unit 1d was added at P1.8 and verified at
P1.9, unit 1e at P1.10, verified at P1.11, and unit 1f at P1.12, the same day. Unit 1f completes
Phase 1's oracle; its verification follows.

The oracle is a static equilibrium solver for the pinning paper's economy (PLAN §3.4).
It shares types but not logic with the agents, and no agent may read it (PLAN R13).
It is built outward in units 1a-1f. This package holds all six:

- **Unit 1a**: one category, one machine type, one land input, with durability and
  interest through the scalar user cost u = (ρ + δ)(1 + ρ)^(J_b − 1). At
  (ρ, δ, J_b) = (0, 1, 1) it is the SSRN Appendix B economy (SSRN 7226858, pp.28-30).
- **Unit 1b**: many categories bought in a fixed basket (SSRN eq 7), on one task line cut
  into segments, each category a density of tasks on them with its own direct land. Space
  is a category. It reports the fork identity in both its forms (SSRN eq 12, main.tex eq
  composites), the category bounds and the purchasing-power pair for every category, and a
  one-category economy solves to 1a's equilibrium bit for bit. On its own, without an
  equilibrium, it prices categories given as task cells (`cell_cost`), and gives SSRN eq
  26's CES share (`ces_share`).
- **Unit 1c**: many machine types and the Leontief inverse. Each type has an operating
  recipe and a build recipe over machine services, labour and land, its own depreciation and
  build lag, and so its own user cost, at one interest rate (check_dynamics R1-R6); categories
  may use each other as intermediate inputs. The cheapest task type takes the machine tasks
  (SSRN A.1), switching along the line; an equilibrium on a switch is a tie, solved by the
  share of tasks each type takes, and more than one equilibrium is refused
  (`SolveError::MultipleEquilibria`). Every Leontief identity over categories and types, and
  the income identity with interest, hold at every equilibrium, and a one-type economy
  solves to 1b's (and so 1a's) equilibrium bit for bit. The machine block alone, at any
  margin, is `MachineBlock`, which reproduces check_dynamics' steady-state price blocks.
- **Unit 1d**: worker types and the tasks closed to machines. The paper's human-required set
  H (SSRN §3.1) is hours per unit of each category that any pooled worker can do; reserved
  tasks are hours only one type can do. Types share the line's shape, each with an efficiency
  ε_i and a living cost ν_i (support baskets on the one basket, so exit is
  s_i = (e^χ − 1)·ν_i·P_s); those selling on the line are one pool with one wage per
  efficiency hour, and a type whose reserved work takes all its hours is paid a scarcity
  price at its own wall. The pool's wage runs along one path: the all-human corner, 1a-1c's
  task line, and the wall (x* = 1, the wage set by labour clearing above γ(1)·π, where "the
  machine comparison does not pin the wage"), with 1c's machine switches and ties continued
  onto the wall. 1a-1c's `BoundaryNoMargin` and `NoInteriorAtZero` are solved; an economy
  with no wage that clears labour with land fully rented is `SolveError::LaborShort`. One
  type without human-required or reserved hours is 1c's equilibrium bit for bit.
- **Unit 1e**: parcels, the idle margin and the priced exit. Land is cut into parcels of an
  acreage, a quality (land service per acre) and an access: enclosed parcels are rented on the
  market, open ones are a commons, free to exit plots and closed to production. Each worker type
  names its exit form: SSRN's dependence form (units 1a-1d), or main.tex's priced form s(q) =
  max(s₀ − q·h, s̲) in units of one exit good, the default of the historical runs (ADDENDUM ruling
  3), both under SSRN eq 8 with the exit life's value. An exit plot takes h of land service: on
  the commons while it has room, at a shadow rent when it is full, and rented on enclosed land,
  which it takes from production. Idle enclosed land earns zero rent: the path continues past the
  wall's end onto an idle stretch with the pool's wage as numeraire, where 1d's `LaborShort`
  economies have their equilibria, exit plots there stand free on idle land (`ExitLand::Idle`),
  and an exit good made of land alone, free at r = 0, has its plots decided as at the wall's end;
  no one working at any wage is `SolveError::NoMarket`. Where q crosses a type's q_enc the economy
  can sit at the threshold with a share of the exiters renting (an enclosure tie). Coverage κ, q*
  and N_crit are reported, and check_enclosure's race plays out inside equilibria. The priced form
  can make labour supply fall along the path, so the count also scans every piece (`EXIT_SCAN`).
  The dependence form in parcel form (one enclosed parcel of quality 1) is 1d's equilibrium bit
  for bit, and so is a priced form with its exit option switched off.
- **Unit 1f**: households and government. The government has SSRN A.1's instruments: a payroll
  tax on gross wages, a uniform tax on final purchases at producer value, a tax on market rent, a
  uniform transfer and a program paying (m_w, m_e) in work and in exit, the transfers counted in
  composites at consumer prices (R14). Its budget balances at every equilibrium: by default the
  owners pay the levy that balances it (`Budget::RentRate`, whose rate at one composite per person
  is SSRN eq 16's 1/κ), or every rate is given and the uniform transfer is the residual
  (`Budget::Dividend`, SSRN A.1's d = τ_R·R/N). A transfer supplements the provider's support (the
  reservation wage (e^χ − 1)(P_s + d), SSRN p.16) or replaces it. Producers pay gross prices, so
  the government enters only through one participation rule that has SSRN eq 9, p.16, eq 27 and
  1d's and 1e's forms as cases; a walled type's wage is still a multiple of P. The path's start
  is evaluated, and an in-work benefit that alone overfills the economy is
  `SolveError::SurplusLabour`. The basket is 1b's fixed one or a CES over the categories
  (`Basket::Ces`, SSRN eq 26), whose σ = 1 case is check_pinning's A-joint household; a category
  that becomes free at the wall's end empties the idle stretch. Every equilibrium reports the
  basket, the budget, the households' and the provider's accounts, the income identity with
  government and three-taxes' ledger, legs and circular flow (`Eq1f`). With the fixed basket and
  `Government::none()` it is 1e's equilibrium bit for bit.

Phase 1's gate is complete with 1f: `tests/gate/p1_gate.rs` names each item of PLAN Phase 1's
gate in one test and lists every test that covers it (docs/unit-1f.md §13).

The package is `rustyecon-oracle`, its library `oracle`, a member of the rustyecon
workspace. Its one dependency is `rustyecon-core`, for `core::num`: the power, `ln1p` and
the fused multiply-add go through the pure-Rust `libm` crate there, so every output is the
same double on every platform (R8, ADDENDUM A5). Nothing on the engine path depends on
the oracle (R13; docs/ENGINE.md §1).

The specifications, with every equation and golden, are [docs/unit-1a.md](docs/unit-1a.md),
[docs/unit-1b.md](docs/unit-1b.md), [docs/unit-1c.md](docs/unit-1c.md),
[docs/unit-1d.md](docs/unit-1d.md), [docs/unit-1e.md](docs/unit-1e.md) and
[docs/unit-1f.md](docs/unit-1f.md). Unit 1b's open
questions (its §11: the shared task line, cells in the equilibrium, gaps as a regime, bitwise
nesting, viability at the top of the line, the generic `Regime`, the CES parameters) and unit
1c's (its §11: machines built from categories, one capability shape per line, ties inside
`Interior`, refusing multiple equilibria, bitwise nesting, physical productivity, the changes
to 1a's module, the random draws, every type priced) are not ruled, nor are unit 1d's (its
§11, proposed as decisions 76-87: one shape with an efficiency per worker type, the corners
inside `Interior`, `LaborShort`, living costs as support baskets, type hours split by net
supply, machine recipes on pool labour, bisection on bit patterns, exogenous training,
viability at the top of the line, a walled tie by bisection, the count over the whole path,
the random draws), nor unit 1e's (its §11, proposed as decisions 88-100: parcels as
efficiency units, the commons for exit only, one participation rule for both forms, support
kept positive, one exit good, rented plots leaving production, idle land with the wage as
numeraire and `NoMarket`, no reserved tasks with the priced form, the scan, the commons'
shadow rent, the enclosure tie inside `Interior`, one land service, the random draws; and from
its verification, 101-102: a free exit good at r = 0 decided at the wall's end, free plots on
idle land labelled `Idle`), nor unit 1f's (its §12, proposed as decisions 103-118: where each tax
is levied, transfers in composites at consumer prices, the owners' levy as the default closure,
supplement or replace, the Dividend closure only where the budget does not depend on who works,
walled types without in-work benefits, the evaluated start and `SurplusLabour`, the CES basket
and its limits, the land-share household and decision 75, a basket per worker type deferred,
the rent base, 1e's accounts kept in `Eq1f::base`, the random draws, no government purchases or
debt). The
build takes the draft's choice on each, and the repository's STATE.md lists them as decisions
open to veto.
Each spec's §12 records where its build and its verification departed from the draft.

## The gate

The oracle is green when the workspace's gate is (`scripts/gate.sh`): `cargo test --workspace
--release` passes on WSL and on Windows, and `cargo clippy --workspace --all-targets -- -D
warnings` and `cargo fmt --all --check` are clean, under the workspace's lints and
`clippy.toml`. The three generators' `--check` (below) are run by hand before any commit
that touches a generator or its goldens. Unit 1a's tests cover, per docs/unit-1a.md §6:

- **G1**, the SSRN Appendix B instance: the published figures to 5e-6 (x* 0.86315,
  v 0.54344, Y 7.88061, N_a 1.34338, hours 1.07846 and 0.26492, N·P_s 5.44630), and
  every full-precision value, bracket value and cost-system total to 1e-12 relative;
- **G2**, SSRN Figure 3's caption values;
- **G3**, the automation path γ = η(1 + x), down to η = 1e-20, where x* rounds to 1.0
  and 1 − x* is 4.6e-21;
- **G4**, durability, interest and the build lag, with the ρ = 0 nesting test, the
  labour share and real wage with interest, φ at u = 1 with δ < 1, and an economy at
  the viability edge (D(x*) = 8.4e-7);
- **G5**, 180 random interior economies (the identities to 1e-12, and single crossing on
  a grid), and two general instances with every parameter off the paper's values
  (k = 4.5 and 2.5, h, χ_max, η ≠ 1, J_b = 2) pinned field by field;
- **G6** and **G7**, the replacement closure (c = 1, w = 3; at λ = 0, 0.4 and 1.2) and
  the three-taxes shares (0.6, 0.4);
- **G8**, the four regimes recognised on constructed cases, including an exact f64 zero
  at each boundary, funding without Lemma B.1, `NoInteriorAtZero` with T/h > N, the
  parameter ceilings (k = 1e20 rejected; k = `CURVATURE_CEIL` = 1024 solved against
  70-digit goldens), `funded` and `lemma_b1` false at exact f64 ties, D(1) = −∞ as
  `NotViable`, the non-finite errors, and the labour-residual net refusing a γ that jumps;
- the dump interface: every key it prints equals the solve's field bit for bit, at u = 1,
  with interest, where `funded` and `lemma_b1` differ, and on a line of fourteen distinct
  values checked against a solve built without the parser;
- `goldens.txt` carries digests of `generate.py` and of itself, which the gate checks.

Unit 1b's tests, per docs/unit-1b.md §8, cover the PLAN's "fork identity and category
bounds on random instances", the income identity at 1e-12, and "1a's results as the
one-category case exactly":

- **C1**, nesting: 1a's G1, 27 golden instances, G5's 180 random draws and every skipped
  one, G8's regime rows and rejections, and `at(x)` on a grid, all bit for bit in category
  form; the fork at G1 and along G3's path;
- **C2**, SSRN eq 26's CES share: goldens on G3's path and at q = 1, σ = 1, limits,
  monotonicity, extreme prices, bad arguments;
- **C3**, the fork economy (manufactures, food, care, shelter on three segments) in flow,
  durable and ρ = 0 versions, pinned field by field, with care on its human bound,
  manufactures fully automated, and near full automation, where the hours carry 1 − x*;
- **C4**, task and recursive automation on it: the wage in manufactures rises 27% while
  the wage in shelter falls 96%;
- **C5**, 180 random multi-category economies: every identity, the fork identity in both
  forms, the bounds and the pair for every category, the residuals, the root and single
  crossing;
- **C6**, check_interior.py's price-block batteries on task cells, its parity instance,
  the flat case at four user costs, and cells against the line;
- **C7**, the gap economy, roots on and near its edges, reductions (an unbought category,
  split categories and segments, permutation, rescaling), the regime rows and validation;
- **C8**, `goldens_1b.txt`'s three digests and the Rust constants.

Unit 1c's tests, per docs/unit-1c.md §8, cover the PLAN's "check_dynamics' targets", "1a and
1b as the one-type case, exactly", "the income identity to 1e-12 with interest" and "random
instances satisfying the Leontief identities":

- **m1**, nesting: 1a's G1 and 27 golden instances, 1b's C3, C3d, C3z, C4, gap, near-edge
  and on-edge roots, 1a's G5 and 1b's C5 random draws with every skipped one, 1a's and 1b's
  regime rows and rejections, and `at(x)` on a grid, all bit for bit in one-type form; a
  flow-only type is 1a's flow economy bit for bit at any (ρ, δ, J);
- **m2**, the machine block alone: check_dynamics' sloped and flat targets (the price block
  and the quantities per unit of the good; the JSON's doubles within 2e-15), U1-U6, R1-R6,
  S1-S6 and L1-L4, the closure per type, the Leontief totals, the envelope (with two
  switches in sequence) and the gross services of three types;
- **m3**, check_dynamics' machine in Appendix B's closure, with and without interest: goldens,
  the income identity with interest, and its corners as 1a economies;
- **m4**, three types (loom, engine, power) on 1b's fork economy with intermediate inputs:
  goldens, the automation path, the Leontief identities by multiplication, the fork through
  the chain, and the tie at the switch as N moves across it;
- **m5**, interest selecting the technique, uniqueness at ρ = 0, and three equilibria refused,
  among them an economy with f(1) > 0 whose boundary hides a root and a tie;
- **m6**, 240 random interior economies in four sets (ρ = 0, ρ > 0, with intermediate inputs,
  and a set built to switch mid-line, with ties and multiple equilibria);
- **m7**, M4's regime rows and exact zeros at each end of a technique's region, every
  validation rule, an unused type (and one that cannot be priced, which is `NotViable`), a
  duplicate type, permutation, unit rescaling (bit for bit at c = 4), the envelope's edge
  cases, two switches with a tie at the second, and φ where the technique is not type 0 and
  where u = 1 with interest;
- **m8**, `goldens_1c.txt`'s four digests and the Rust constants.

Unit 1d's tests, per docs/unit-1d.md §8, cover the PLAN's "constructed wall and interior cases
recognised correctly", "the wall regime solved" and "1a to 1c as the case with no
human-required tasks and one worker type, exactly":

- **d1**, nesting: 1a's 27 golden instances, 1b's C3, C4, gap, near-edge and sliver roots, 1c's
  M3, M4 path, M4t and M5 path, bit for bit through `WorkerParams::from_machines`; 1a's G5, 1b's
  C5 and 1c's m6 draws, every draw; every boundary row solved, with 1c's diagnostics bit for bit
  as the line's values; the refusals (NotViable, M5m's three equilibria, M5b's two where 1c
  counts three) and the rejected rows; `at(x)` on a grid;
- **d2**, the corners in Appendix B's closure: the wall (λ 0.6) against its goldens and its
  closed form, with 60 random walls; labour shortage at the wall (N 0.25) and at the real-wage
  ceiling (χ_max 3); the all-human corner (N 20, χ_max 0.05); roots below 10^-12;
- **d3**, the human-required economy: the tail in labour demand, the tail deciding the regime,
  and check_pinning D1's automation path to the wall with SSRN Prop E.1's limits (services'
  price to |H|·v, labour's share to 1, the CES share of H-content to 1, the wage to v_∞);
- **d4**, the entrant and the trained: pooled, at the trained's wall, at the wall and at the
  all-human corner; a reserved shortage; the walk's order and its cascade with a master type;
  supply and exit per type; a type split in two; the end of the wall with each type's
  efficiency; lemma B.1 with the support;
- **d5**, 1c's M4 with worker types: reserved costs through the chain, the cost system with
  them, a wall past a machine switch, and a tie with a walled type (σ by bisection);
- **d6**, switches on the wall: a tie at a wall switch, and roots either side of it;
- **d7**, 300 random equilibria in five sets (one machine type, several, with interest,
  abundant labour, and a set built to switch on the line or the wall): every identity and
  bound, the residuals, f nonincreasing on every stretch, the count, the types' status;
- **d8**, exact zeros at the junctions, viability unchanged, every validation rule, the types
  permuted, efficiency in other units (bit for bit at c = 4), the walk's ceiling, the edge
  of a reserved shortage solved on the line, below 10^-12 and in a tie's split, and the rules
  at exact equality;
- **d9**, `goldens_1d.txt`'s five digests and the Rust constants.

Unit 1e's tests, per docs/unit-1e.md §8, cover the gate of ADDENDUM ruling 3 and A7: "the
idle-margin regime and the enclosed regime recognised and solved", "check_enclosure's worked
instance reproduced" and "both exit forms nest, with the exit option switched off, into 1a to
1d exactly":

- **e1**, nesting: 1a's 27 golden instances, 1b's and 1c's, and 1d's W, B, E, F, X and J
  instances through `ParcelParams::from_workers`, bit for bit, with W2, W3 and E6 (1d's
  `LaborShort`) on idle land with f_∞ 1d's excess; 1a's G5, 1b's C5, 1c's m6 and 1d's d7 draws;
  the refusals and rejected rows; `at(x)` on a grid; every type priced with s₀ = s̲ = 0; D,
  an exit priced out of use, which is G1 bit for bit; a floor without a plot, which is K1; and
  every exit good in economies without exit values;
- **e2**, the exit value alone: check_pinning P3, check_enclosure N-ii, N-iii and N-vi, SSRN
  D.3's coverage, the identical-workers limit where the support cancels, bad arguments;
- **e3**, the race (Q1-Q6): plots rented with κ ≥ 1, the enclosure ties at q = q_enc = 1.5 with
  κ = 1 at N_crit = 60 and 0.75 at N 80, the floor inside the gap and past q*, an enclosure tie
  on the wall (Q6), and two enclosure points in one piece;
- **e4**, the commons (K1-K5): with room, crowded, enclosed by price and by law, quality as
  efficiency, rented plots leaving production;
- **e5**, idle land (I1-I4, L1-L2): 1d's W2, W3 and E6 solved at zero rent, the wall at r = 0,
  free plots on idle land, the worst parcels idling first, the edge of a reserved shortage on
  the idle stretch, `NoMarket`, and an exit good made of land alone, decided at the wall's end;
- **e6**, two priced types sharing a commons, 1c's fork economy with a priced exit in food,
  both conditions of the certification, and the all-human corner under the cheapest type;
- **e7**, M's three equilibria (one when the scan is off), Lemma 5's σ against a finite
  difference with both signs, and the root independent of the scan;
- **e8**, 480 random equilibria in eight sets, among them certified draws, enclosure ties on the
  line, the wall and the all-human corner, and exit goods drawn among the categories: every
  identity with 1d's residuals bounded, the count against a scan 16 times finer, f monotone
  along a certified path, Lemma 5 at the equilibrium, every regime asserted;
- **e9**, the regimes at exact equality, 1d's saturated knife edge and a priced one, Lemma B.1's
  flag on the market's land, every validation rule, permutation, land service in other units
  (bit for bit at c = 4), enclosure by law;
- **e10**, `goldens_1e.txt`'s six digests and the Rust constants.

Unit 1f's tests, per docs/unit-1f.md §8, cover the brief's gate for 1f (three-taxes' ledger
inside the full closure, the income identity with government to 1e-12, Prop 6's identities and
κ, each tax's incidence as the paper states it, nesting with no government exactly) and close
Phase 1's:

- **f1**, nesting: every golden instance of 1a-1e (78 of them) and 1d's and 1e's random draws
  through `HouseholdParams::from_parcels`, every equilibrium, error and count bit for bit; the
  refusals and rejected rows; `at(x)` on a grid; the Dividend closure and Replace mode at zero
  rates; the producer side the same under every government; a replacing transfer moving nothing
  (GA, GR); CES at σ = 2^-40 within 1e-8 of the fixed basket;
- **f2**, three-taxes inside the closure (TX): x* = 1/2 to 4 ulps, (φ_w, φ_r) = (0.6, 0.4) for
  the machine service and the basket, every tax on a grid leaving the allocation bit for bit,
  T6's legs, λ = 0's corner, T5's circular flow (R₀ 4.8, the multiplier 5/3);
- **f3**, Prop 6's receipts, eq 16 (GA's τ_R = 1/κ, TXD outside the rent, KR1-KR5 with parcels),
  D.3's identity, eq 17 at full automation, and the corollary along SSRN's automation path;
- **f4**, a supplementing transfer (GB, the reservation wage (e^χ − 1)(P + d)), the Dividend
  closure (GC), a transfer above the support in Replace mode;
- **f5**, payroll incidence: ε_D/(ε_D + ε_S) = 0.88586 by a three-point difference at G1, none on
  the wall (W1), all on idle land (WI) and where participation saturates (TX);
- **f6**, a consumption tax t as a payroll tax t/(1 + t) (GT, KT with plots rented, ER with a
  walled type), the support at consumer prices, home output untaxed;
- **f7**, eq 27 at GW and GE, an in-work benefit lowering the wage, a uniform program as a
  supplementing transfer bit for bit, `SurplusLabour`, a walled type's c_T·P (ER);
- **f8**, the CES household: eq 26 at C1-C3 and along the path (CP), A-joint's land-share
  household (AJ1, AJW) and A-joint itself `NotViable`, a free category on the wall (CW), the
  idle stretch without one (CI), Lemma F1 against a finite difference, the frozen composite;
- **f9**, 341 random draws in five sets (RentRate, Dividend, CES, reserved hours, in-work
  benefits), seeds 951-955: every identity, the residuals, the consumption tax as a wage tax
  on every (a) and (d) equilibrium, a replacing rent tax's neutrality, the count against a scan
  16 times finer, the regimes and their tallies;
- **f10**, every validation rule, exact zeros (L_R = 0, the Replace kink, f_0 = 0), units,
  permutation, the certification under a government, and the scan where Proposition F does not
  reach (ρ > 0 under a CES basket or a moving Dividend transfer);
- **f11**, `goldens_1f.txt`'s seven digests and the Rust constants;
- **p1_gate**, Phase 1's gate item by item, and a check that every test its table names exists.

The goldens are pinned to laborformal `31b3482`.

## Verification

Each unit was checked after its build by an adversarial pass, an independent derivation
that does not read the crate and mutation testing, then one fix round, each fix with a test
that fails without it. Where the pass found nothing wrong in the code, the fix is a test.

- **1b** (P1.3): the build's own mutation check caught 44 of 45 mutants; the survivor
  reassociates p_j = v·H_j + (p_m·M_j + b_j), which changes only the rounding. The pass found
  four more survivors, now caught: the carried 1 − x* in a category's final hours and λ̃_j^q,
  the edge convention of `margin_active`, and `Requirement::OpenUnit` admitting 0. It also
  found that outputs made of the sliver between x* and an interior edge of the task line lose
  precision as 2^-53·x*/d; that is stated with its bound (docs/unit-1b.md §5.4) and measured
  by 24 near-edge goldens, and no code changed.
- **1c** (P1.5): an independent derivation in another formulation agreed with the oracle on
  348 economies, every regime, technique, tie and switch count, and the values within
  5.6e-14. Of 65 mutants six survived; five are now killed and one is equivalent in exact
  arithmetic.
- **1c, second pass** (P1.6): a second derivation found a blocker. With an upward jump of
  labour demand at a switch and f(1) > 0, the build returned `BoundaryNoMargin` where a root
  and a tie lay below; such an economy is now `MultipleEquilibria`, with the boundary
  counted as one. It measured the precision at ties on 432 of them (docs/unit-1c.md §5.6),
  and found that an unused type whose price diverges makes the economy `NotViable`, which
  is kept and recorded as a departure from SSRN A.1 (§12 item 15, open question 9). Nine
  mutants that survived the gate as it was are now killed, and six of the eight made of the
  fix; three survivors are equivalent on every economy the gate can build (§12 items 12
  and 17).
- **1d** (P1.9): an independent derivation found a blocker. Where a type's reserved demand
  reaches its workers, its supply is vertical, and a wage above its own clearing wage clears
  the pool: an equilibrium, which P1.8 had refused as `LaborShort`. It is now solved (seven of
  the derivation's 60 targeted draws, two of d7's), and the saturated knife edge with f(1) = 0
  is the junction rather than `LaborShort`. The mutation pass left eleven survivors among f_∞'s
  efficiencies, a corner's φ and closure wage, d, lemma B.1's support, the first short type,
  `human_required`'s length and four rules at exact equality; each now has a test, and the fix
  round's 20 mutants are all killed (docs/unit-1d.md §12 items 16-18).
- **1e** (P1.10, the build's own check, before its verification): 49 mutants of the new code,
  of which the first run left nine; seven now have a test, and two are equivalent (the idle
  end's side where S_∞ cannot be 0, and a technique tie's closed form scaled by T/T_m)
  (docs/unit-1e.md §12 item 14).
- **1f** (P1.12, the build's own check, before its verification): 57 mutants of the new code
  (the participation rule, the CES basket, the evaluation, the path, the report and the
  validation). The first run killed 56; the survivor, the scan at ρ > 0 switched off, now has a
  test (`f10::the_scan_where_monotonicity_is_not_proved`). The 22 that the unit tests caught
  first were run again against the gate alone, which kills each; it kills the survivor now too
  (docs/unit-1f.md §14 item 14).
- **1e** (P1.11): the derivation found that an exit good made of land alone, free at r = 0,
  put its plot-takers on the floor there while they rented on the wall, so the idle stretch
  did not start where the wall ends and an economy with one equilibrium was refused; such a
  good's plots are now decided at the wall's end (L1, L2). It also found free plots on idle
  land labelled `Enclosed`; they are `Idle`. Thirteen of the pass's 30 mutants survived; each
  now has a test, and six mutants of the fixes are killed (docs/unit-1e.md §12 items 16-19).

Each spec's §12 has the details.

## Layout

| path | what it is |
|---|---|
| `src/params.rs` | `Params`, validation (`ParamError`), `Economy`, the user cost u |
| `src/schedule.rs` | the `Schedule` trait (γ and its integral J) and `PowerSchedule`, γ = η(g0 + g1·x^k) |
| `src/solve.rs` | `Economy::at(x)`, `Economy::solve`, `Regime<E>`, `Eq1a`, residuals, the cost-system view, and the regime tests and bisection both units share |
| `src/closure.rs` | `closure(a, λ, γ*, b, r, u)`, the price block alone |
| `src/categories.rs` | unit 1b: `Category`, `CategoryParams`, `CategoryEconomy`, `Eq1b` and its outputs |
| `src/fork.rs` | unit 1b's price block alone: `Cell`, `cell_cost`, `ces_share` |
| `src/leontief.rs` | Gaussian elimination without pivoting, in index order, on unit 1c's M-matrices (crate-private) |
| `src/machine_block.rs` | unit 1c's machine block alone: `Recipe`, `MachineType`, `MachineBlock` (totals, closure, envelope, gross services) |
| `src/machines.rs` | unit 1c: `MachineParams`, `MachineEconomy`, `Eq1c` and its outputs, ties and the sign-change count |
| `src/workers.rs` | unit 1d: `WorkerType`, `WorkerParams`, `WorkerEconomy`, `Eq1d` and its outputs, the walk, the corners, the path and its count |
| `src/exit.rs` | unit 1e's exit value alone: `PricedExit` (s(q), q_enc, the take), `coverage`, `coverage_threshold`, `crowding_limit` |
| `src/parcels.rs` | unit 1e: `Parcel`, `ExitForm`, `ParcelParams`, `ParcelEconomy`, the exit sub-problem, the idle stretch, enclosure points and ties, the scan, `Eq1e` and its outputs |
| `src/households.rs` | unit 1f: `Basket`, `Government`, `HouseholdParams`, `HouseholdEconomy`, the participation rule and the CES basket (crate-private), `Eq1f` with the budget, the accounts and the ledger |
| `src/dump.rs` | the one-line text interface behind `examples/dump.rs` |
| `examples/dump.rs` | reads economies on stdin, writes one result line each |
| `tests/gate/` | the gate: one test crate, one module per golden group (1a's `g*`, 1b's `c*`, 1c's `m*`, 1d's `d*`, 1e's `e*`, 1f's `f*`), and `p1_gate`, Phase 1's gate item by item |
| `goldens/generate.py` | computes every golden with mpmath at 70 digits |
| `goldens/goldens.txt` | its output, 30 significant digits |
| `goldens/generate_1b.py` | unit 1b's goldens, at 70 digits; imports `generate.py` to assert the nesting |
| `goldens/goldens_1b.txt` | its output, 273 goldens |
| `goldens/generate_1c.py` | unit 1c's goldens, at 70 digits; imports `generate_1b.py` (and so `generate.py`) to assert the nesting |
| `goldens/goldens_1c.txt` | its output, 210 goldens |
| `goldens/generate_1d.py` | unit 1d's goldens, at 70 digits; builds on `generate_1c.py` (and so the other two), and asserts the nesting |
| `goldens/goldens_1d.txt` | its output, 269 goldens |
| `goldens/generate_1e.py` | unit 1e's goldens, at 70 digits; builds on `generate_1d.py` (and so the other three), and asserts the nesting |
| `goldens/goldens_1e.txt` | its output, 223 goldens |
| `goldens/generate_1f.py` | unit 1f's goldens, at 70 digits; builds on `generate_1e.py` (and so the other four), and asserts the nesting |
| `goldens/goldens_1f.txt` | its output, 215 goldens |

## Running the tests

Keep the build directory outside the repository, as the workspace's gate does, and
building on a Windows drive from WSL is slow. The whole gate is `scripts/gate.sh` (see the
root README). For this package alone, from the repository root, on the primary platform,
WSL Ubuntu, and on the secondary one, Windows with the MSVC toolchain (both Rust 1.97.1,
pinned by `rust-toolchain.toml`):

```sh
export CARGO_TARGET_DIR=<a directory outside the repository>
cargo test --release -p rustyecon-oracle
cargo clippy -p rustyecon-oracle --all-targets -- -D warnings
cargo fmt --all --check
```

In PowerShell, set `$env:CARGO_TARGET_DIR` instead. To drive WSL from Windows, use
`wsl -d ubuntu --exec bash -lc '…'`; `wsl -- …` loses exit codes.

The counts, the same on WSL and on Windows at each step:

| Step | Date | Unit | Gate | Doc | Total |
|---|---|---|---|---|---|
| unit 1a, after its final verification round | 2026-09-25 | 42 | 71 | 1 | 114 |
| P1.1, 1a in the workspace | 2026-09-26 | 42 | 71 | 1 | 114 |
| P1.2, unit 1b | 2026-09-27 | 46 | 122 | 1 | 169 |
| P1.3, 1b's verification | 2026-09-27 | 47 | 125 | 1 | 173 |
| P1.4, unit 1c | 2026-09-27 | 56 | 175 | 1 | 232 |
| P1.5, 1c's verification | 2026-09-27 | 57 | 177 | 1 | 235 |
| P1.6, 1c's second verification | 2026-09-27 | 57 | 184 | 1 | 242 |
| P1.7, units 1b and 1c closed | 2026-09-27 | 57 | 184 | 1 | 242 |
| P1.8, unit 1d | 2026-09-27 | 66 | 227 | 1 | 294 |
| P1.9, 1d's verification | 2026-09-27 | 67 | 230 | 1 | 298 |
| P1.10, unit 1e | 2026-09-27 | 76 | 276 | 1 | 353 |
| P1.11, 1e's verification | 2026-09-27 | 77 | 283 | 1 | 361 |
| P1.12, unit 1f | 2026-09-27 | 83 | 348 | 1 | 432 |

Of the 432, 1a has 114, 1b 59 (5 unit, 54 gate), 1c 69 (10 unit, 59 gate), 1d 56 (10 unit,
46 gate), 1e 63 (10 unit, 53 gate) and 1f 71 (6 unit, 57 gate, and `p1_gate`'s 8).

## The dump example

`examples/dump.rs` is for differential testing against other solvers. Each input line
is whitespace-separated `key=value` pairs, one for each of `workers land space a lam b
eta g0 g1 k chi_max rho delta build_lag`. Each output line starts with `regime=<name>`,
then every output of `Eq1a::outputs` as `key=value` (every `Eq1a` field, including
`one_minus_x_star`, and each residual), or the non-interior regime's diagnostic. Floats
use Rust's `{:?}` formatting: the shortest digits that parse back to the same double,
always with a decimal point or an exponent (`1.0`, `-6.0005e-12`, `1e30`). A bad line,
including one that is not UTF-8, gives `error=<message>`. Every input line gives exactly
one output line. The loop is `oracle::dump::run`, which the unit tests drive on raw
bytes; if stdin cannot be read or stdout cannot be written or flushed, the example says
so on stderr and exits with status 1.

```sh
echo "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1" \
  | cargo run --release -p rustyecon-oracle --example dump
```

This prints `regime=Interior x_star=0.863150418162437 one_minus_x_star=0.136849581837563 …
u=1.0 … v=0.5434359606967785 … y=7.880605524972908 … n_a=1.3433818800977175 …`, which
is G1.

On 2026-09-25, 2001 random economies through the dump example agreed with laborformal's
`macro.py` on every regime and on every interior value within 9e-14 relative. That run
had J_b = 1 only and pooled `BoundaryNoMargin` with `NoInteriorAtZero`. The review of the
same day compared all four regimes on 3616 economies with J_b from 1 to 12, against a
macro.py patched for u; it found no disagreement except where the true |f| at a bracket
end is at most 3.8e-16, where the regime is not decidable in f64 (docs/unit-1a.md §4).
The largest value gaps are where 1 − x* is small: there both macro.py's brentq xtol and
the f64 conditioning of 1 − x* matter.

The dump is unit 1a's only. Units 1b to 1e have none, since there is no other
multi-category, multi-type, multi-worker or parcel solver to compare against (docs/unit-1b.md
§9 to docs/unit-1e.md §9); their independent checks are the verifications' derivations.

## Regenerating the goldens

`goldens/generate.py` needs a Python with mpmath (1.3.0 was used; laborformal's analysis
venv has it, WSL's python3 does not). From `crates/oracle`:

```sh
python goldens/generate.py           # writes goldens/goldens.txt
python goldens/generate.py --check   # exits 1 if goldens.txt is not what it writes
```

On Windows, set `PYTHONIOENCODING=utf-8` first. The generator works at 70 digits and
asserts the published figures, the identities at 65 digits, the ρ = 0 nesting and each G8
regime as it goes. The constants in `tests/gate/goldens.rs` are `goldens.txt` rounded to
20 significant digits, and the gate test `goldens_file::constants_match_goldens_txt`
fails if the two disagree. After a change, update those constants by hand, with a
provenance comment each.

`goldens.txt`'s header records FNV-1a digests of `generate.py` and of the goldens below
it, and `goldens_file::goldens_txt_is_from_generate_py` recomputes both. So the gate,
even where mpmath is missing, fails if `goldens.txt` was edited by hand or not rewritten
after `generate.py` changed. It cannot prove that the values are what `generate.py`
computes: **run `generate.py --check` before committing any change to either file.**

Unit 1b's goldens work the same way:

```sh
python goldens/generate_1b.py           # writes goldens/goldens_1b.txt
python goldens/generate_1b.py --check   # exits 1 if goldens_1b.txt is not what it writes
```

`generate_1b.py` imports `generate.py` to assert that the category form of six 1a
instances equals 1a's solve, so `goldens_1b.txt` records the digests of both generators
and of its own goldens, and `c8_goldens_file` recomputes all three: a change to
`generate.py` means rerunning both. The constants in `tests/gate/goldens_1b.rs` are
`goldens_1b.txt` rounded to 20 significant digits (`c8_goldens_file::
constants_match_goldens_1b_txt`). On 2026-09-27 the oracle's f64 values matched all 245
numeric goldens of unit 1b away from an interior edge within 1.0e-15 relative. Of the 24
near-edge goldens, the three N are inputs, the 15 full-precision outputs match within
1.9e-16, and the six outputs made of the sliver by the edge miss by up to 7.5e-8, within
the bound of docs/unit-1b.md §5.4.

Unit 1c's goldens work the same way, with four digests (`generate_1c.py`, `generate_1b.py`,
`generate.py` and its own goldens), checked by `m8_goldens_file`:

```sh
python goldens/generate_1c.py           # writes goldens/goldens_1c.txt
python goldens/generate_1c.py --check   # exits 1 if goldens_1c.txt is not what it writes
```

It asserts as it goes that the one-type form of 1b's instances equals `generate_1b.py`'s solve
and M3's corners equal `generate.py`'s (within 6.1e-71), that check_dynamics' targets match the
JSON's doubles within 2e-15, every identity of docs/unit-1c.md §4 at 1e-65, the envelope
against a scan of 10^4 points, and f nonincreasing in every technique region. On 2026-09-27
the oracle's f64 values matched 191 of the 207 goldens it computes within 1.2e-15 relative
and 199 within 1e-14; the rest are where docs/unit-1c.md §5.6 says precision goes: the switch
points (up to 2.8e-14, from the cancellation in γ_i's closed form), a least pivot near 0
(1.7e-14), and the excess demand at M5m's switch, evaluated at the double below it where f is
steep (4.8e-13).

Unit 1d's goldens work the same way, with five digests (`generate_1d.py`, `generate_1c.py`,
`generate_1b.py`, `generate.py` and its own goldens), checked by `d9_goldens_file`:

```sh
python goldens/generate_1d.py           # writes goldens/goldens_1d.txt
python goldens/generate_1d.py --check   # exits 1 if goldens_1d.txt is not what it writes
```

Its `Economy` extends `generate_1c.py`'s (docs/unit-1d.md §12 item 1). It asserts as it goes
that the one-type form of 1c's instances equals `generate_1c.py`'s solve (within 8.7e-72) and
that its line values equal 1c's boundary diagnostics, every identity of docs/unit-1d.md §4.7 at
1e-65, f nonincreasing on a grid of every stretch, every switch at ρ = 0 downward, the envelope
on the wall against a scan of p_t/θ_t, the corners' evaluation equal to the line's at both
junctions, the wall's closed form and BP's limit. On 2026-09-27 the oracle's f64 values matched
the 233 goldens compared by `close` within 2.0e-15 relative (F3's tie share); W5's root below
10^-12 is within 1.1e-16 absolute (2.3e-4 relative, inside docs/unit-1d.md §5.5's bound), and
its f on the line, n_D − n_S with n_D ≈ 10, within 9.8e-16 absolute.

Unit 1e's goldens work the same way, with six digests (`generate_1e.py`, `generate_1d.py`,
`generate_1c.py`, `generate_1b.py`, `generate.py` and its own goldens), checked by
`e10_goldens_file`:

```sh
python goldens/generate_1e.py           # writes goldens/goldens_1e.txt, in about a minute
python goldens/generate_1e.py --check   # exits 1 if goldens_1e.txt is not what it writes
```

Its `Economy` extends `generate_1d.py`'s (docs/unit-1e.md §12 item 1); an economy without exit
values is solved by `generate_1d.py`'s own solve with the idle stretch after it. It asserts as
it goes that the parcel form of fifteen of 1d's instances equals `generate_1d.py`'s solve
(within 1.6e-71) and 1d's `LaborShort` rows are idle-land equilibria with 1d's f_∞, the exit
option switched off is the dependence form, D is G1, L1's f_∞ is the wall's limit and L2 is
W3, every identity of docs/unit-1e.md §4.8 at
1e-65, κ = qT/(N(1 + q)) at the Q instances, each enclosure tie against its linear root and each
idle closed form against bisection, Lemma 5's sign at 292 points against a finite difference,
and the count on a scan of 1024 points per piece. On 2026-09-27 the oracle's f64 values matched
the goldens within 2.4e-14 relative (Q5's provider baskets, 80.44 − 80), 7.0e-15 (Q2's f above
its enclosure point) and 5.1e-15 (K2's shadow rent), ψ, T_p and T_idle within 2.1e-15, and x*,
v, P_s, Y and N_a within 1.0e-15.

Unit 1f's goldens work the same way, with seven digests (`generate_1f.py`, `generate_1e.py`,
`generate_1d.py`, `generate_1c.py`, `generate_1b.py`, `generate.py` and its own goldens), checked
by `f11_goldens_file`:

```sh
python goldens/generate_1f.py           # writes goldens/goldens_1f.txt, in about half a minute
python goldens/generate_1f.py --check   # exits 1 if goldens_1f.txt is not what it writes
```

Its `Economy` extends `generate_1e.py`'s with the basket (fixed and CES), the participation rule,
the walled c_i, the budget in both closures, the start and the accounts (docs/unit-1f.md §14 item
1). It asserts as it goes that the household form of six of 1e's instances equals
`generate_1e.py`'s solve (to 0 at 70 digits), every identity of docs/unit-1f.md §4.8-4.9 at
1e-65 with the CES's shares and Shephard's lemma, the incidence share against ε_D/(ε_D + ε_S)
to 1e-35, the equivalences of §4.10 (c), (f) and (h) at 1e-65, A-joint's root from
check_pinning's equations against its printed values (within 5.1e-16) and the household form's
with γ(1) 10^-29 below A-joint's (within 1e-20), Lemma F1 against a derivative at 14 points, and
f nonincreasing at 96 grid points. On 2026-09-27 the oracle's f64 values matched the 444 golden
comparisons of `f1`-`f11` and `p1_gate` within 1.9e-15 relative (CW's wage near the wall's
real-wage ceiling), all but three within 1.0e-15; the three made by finite differences are
within their 1e-8: the incidence share by a three-point difference in τ_w (6.7e-10), ε_D
(1.9e-10) and ε_S (2.2e-11).

At the close (P1.7) all three `--check` passed, and the gate's golden comparisons were logged
again on WSL: the largest errors are as above, 1.0e-15 on 1b's goldens away from an edge (the
cells of `c6::cells_approach_the_line` are compared with the line's closed forms at the
midpoint rule's error, up to 3.5e-9, by design) and 4.8e-13 on 1c's, then 4.4e-13 on M5m's
tie share and 2.8e-14 on the switch points.

## Numerics

- The root is found by bisection on [1e-12, 1] until lo and hi are adjacent doubles.
  There is no tolerance. `BRACKET_LO` and `MAX_BISECTION_STEPS` are named constants
  with their reasons in `src/solve.rs`.
- u and the machine-wealth factor are computed by repeated squaring in this crate, so
  they are identical on every platform. u's error grows with the build lag, to about
  J_b·2.2e-16 relative (the `user_cost` doc).
- Every scale parameter (N, T, h, b, χ_max, and η, g0, g1) must lie in
  [`SCALE_FLOOR`, `SCALE_CEIL`] = [1e-30, 1e30], δ in [1e-30, 1], and λ and ρ in
  [0, 1e30]. Within these bounds the products and quotients the regime tests compare stay
  far from underflow and overflow. u and D are not bounded: a huge u can still overflow
  a price, and the solve then returns `SolveError::NonFinite`, never a wrong regime.
  D(1) = −∞ is `NotViable`.
- k must lie in [1e-30, `CURVATURE_CEIL`] = [1e-30, 1024]. Across one double of x, γ moves
  by less than max(k, 1)·2^-52 relative, 2.3e-13 at the ceiling, so γ stays resolved by
  the doubles x* is chosen from. At k = 1e20 it jumped from g0 to g0 + g1 across the last
  double below 1, and the solve returned a wrong `Interior`.
- An `Interior` result must clear the labour market: |N_a − n_S(x*)| (`res_labor`) at most
  `LABOR_RESIDUAL_NET` = 1e-9 of N_a, or the solve returns `SolveError::LaborNotCleared`.
  This is a safety net, not a tolerance. A root leaves at most about 1e-13 away from the
  viability edge; a jump in n_D − n_S leaves the jump. It trips on a schedule that breaks
  the continuity contract, and within about 5e-5 of the viability edge where the root is
  not resolved in f64 (docs/unit-1a.md §4 step 5).
- `funded` is `provider_baskets > 0`, so the two never disagree in f64.
- A negative zero in a, λ or ρ is stored as +0.0, so the same economy prints the same.
- At an exact f64 zero, the regime follows the spec's convention: D(1) = 0 is
  `NotViable`, f(1) = 0 `BoundaryNoMargin`, f(1e-12) = 0 `NoInteriorAtZero`. Within a
  few ulps of a boundary the regime is not decidable in f64. `NoInteriorAtZero` means
  f(1e-12) ≤ 0; a root can still lie in (0, 1e-12). The band where the regime is not
  decidable widens near the viability edge and with the build lag: u's error (J_b·2.2e-16)
  makes D(1) uncertain by about that much, and the prices carry it into f amplified by
  u(a + λγ)/D (docs/unit-1a.md §4 step 3).
- Near the viability edge the prices scale as 1/D, and x*'s representation reaches them
  amplified: at the double x*, the outputs are within about max(k, 1)·2^-53·(1 + uλγ/D)
  relative of the exact equilibrium, besides D's own rounding, 2^-53·(1 − u·a)/D. An
  `Interior` result within about 1e-5 of the edge (D(x*) ≲ 1e-5) can carry a labour
  residual up to the 1e-9 net (docs/unit-1a.md §4 step 4).
- `x_star` is a double in [1e-12, 1]; it is exactly 1.0 when the root is within half an
  ulp of 1. 1 − x* is carried separately, as `one_minus_x_star`, interpolated between
  the two doubles that bracket the root, and every output proportional to 1 − x*
  (final hours, N_a, participation, the labour share) uses it, so they keep full relative
  precision however close x* is to 1. In unit 1c this holds for a root, not for a tie at a
  switch near 1 (below).
- In unit 1b only the top segment carries its offset. Within a distance d of an interior
  edge of the task line, an output made mostly of the sliver between x* and the edge is
  good to about 2^-53·x*/d relative (hours) or 2^-53·(x* + 2J(x*)/γ(x*))/d (machine
  services, and K, M_s and interest when all machine use is in the sliver): 2.1e-8 on a
  category's hours at d = 1e-9, 1.9e-11 on K at d = 1e-6. Prices, v, x*, P_s, Y and N_a
  keep full precision. Interpolating an offset from each edge would not recover it
  (docs/unit-1b.md §5.4).
- D = 1 − u(a + λγ) is computed as (1 − u·a) − u·λγ with a fused multiply-add
  (`core::num::fma`, correctly rounded), and 1 − aδ likewise, so neither loses precision
  near the viability edge or as aδ → 1.
- Interest is computed as ρ·W_K, not (u − δ)·V_m·K, which cancels when ρ is small.
- Unit 1c's Leontief systems (the categories' I − A_cc, the machine block's I − Â and
  I − A^q, and the (O, V) system at each x) are solved by Gaussian elimination without
  pivoting, in index order. On these M-matrices it is stable, its pivots being positive is
  the productivity or viability test itself (the least pivot generalises 1a's D), and its
  fixed order makes one type repeat 1a's and 1b's operations bit for bit
  (`src/leontief.rs`; docs/unit-1c.md §5.1).
- A switch of the cheapest type is a closed-form γ_i; its point on the line is the largest
  double with γ(x) < γ_i, found by bisection, so γ is never inverted. γ_i carries the totals'
  rounding amplified by the cancellation in its numerator, up to 2.8e-14 on M5's switch.
  x_i is good in absolute terms: near x = 0 its relative error grows (1.6e-12 at 0.0042).
- A tie is evaluated at x_i, so every output carries γ_i's error through γ*: up to about
  2e-13 relative on x*, v and the prices on the second verification's 432 ties (7.5e-13
  near the viability edge). The split σ, and each type's services, builds, hours, land and
  wealth, carry it amplified by |f'|·x*/|jump| (the excess demand's slope over the jump of
  labour demand at the switch): up to 1.7e-10 there. A tie near x = 1 sets 1 − x* = 1.0 − x_i
  without interpolation, so 1 − x* and the outputs proportional to it are good only to about
  2^-53/(1 − x*) relative plus γ_i's error over γ'·(1 − x*): 3.2e-7 at a tie 1e-9 below 1
  (docs/unit-1c.md §5.6).
- With a switch of technique, a boundary regime is returned only when the excess demand
  changes side nowhere inside the bracket: f(1) ≥ 0 with a root or a tie below is
  `SolveError::MultipleEquilibria`, the boundary counted as one of them (docs/unit-1c.md
  §5.3). In unit 1c `NotViable` also covers a type no technique would use whose price
  recursion diverges (docs/unit-1c.md §12 item 15).
- Unit 1d bisects the stretches the line's bisection does not cover on the bit patterns of
  doubles: a root in [0, 1e-12] (down to the subnormals), the corners' real wage ω (up to
  +∞ at the end of the wall), and a tie's σ with a walled type. Each halves the count of
  doubles in the bracket, so it takes at most 64 steps; 1a's arithmetic bisection stays on
  [1e-12, 1], so the line nests bit for bit (docs/unit-1d.md §5.3).
- At a corner the wage follows from ω as v = ω·B/((1 − C) − ω·L), which carries ω's error
  amplified by (C + ω·L)/((1 − C) − ω·L), without bound at the real-wage ceiling: a wall
  near its ceiling has a wage ill-conditioned in the data (docs/unit-1d.md §5.5).
- In unit 1d a point where a reserved market cannot clear (demand above the type's workers)
  or where the walled types' shares of the basket reach 1 is short, its excess demand +∞.
  A change of side that closes on a short point is the edge of that reserved shortage, where
  the type's supply is vertical at its workers: the equilibrium is there, with the type's
  wage set by the pool's clearing (docs/unit-1d.md §12 item 16). `LaborShort` is an economy
  whose excess demand changes side nowhere on the path.
- Unit 1e's nesting is bitwise through the same device as 1b-1d: with r = 1.0, the market's
  land T and every exit value 0.0, each new operation is exact (v − 0.0 = v, ν·P_s + 0.0,
  1.0·b = b, T − 0.0 = T). A crowded commons' shadow rent is the least double in [0, r] with
  G(r_o) ≤ T_o, by bisection on bit patterns; the exit value on a plot is p_g·s₀ − r_o·h with a
  fused multiply-add, so it keeps 2^-53·p_g·s₀ absolute as it nears the floor. At zero rent a
  good made of land alone is free, and its real wage, wage floor and price shares are reported
  absent; a machine type without labour is free too, and the pool's wage in machine-task units
  with it. The count's scan finds two equilibria only when they are more than one cell apart
  where the excess demand can rise; where §5.4's certification holds it cannot, and the count is
  exact (docs/unit-1e.md §5.4-5.5).
- Unit 1f's nesting is bitwise through the same device: with no government each new operation
  is exact (1.0·P = P, ν·P + 0.0, (0.0 − 0.0) + (1.0·v − 1.0·e) = v − e, ((ν + 0.0)·ζ − 0.0)·1.0/1.0
  = ν·ζ), and the start's f_0 = n_D(0) − 0.0 is the value 1e assumed positive. A CES basket is
  evaluated in logs, the prices scaled by the largest (σ < 1) or the smallest (σ > 1) so that no
  exponent is positive, with ln1p and expm1 near σ = 1; a category's content stays finite for price
  ratios up to about 2^16 at `SIGMA_CEIL` = 64. Its corners are parametrized by ω = v/P_z with the
  fixed basket's P_z, and a weighted category that embodies no labour at the wall's end (or no
  land at the all-human corner's start) is free there: the wall's end is then −S_∞ and the start
  +∞, limits, not evaluations. The consumption-tax equivalence is exact in real numbers and came
  out bit for bit at GT and ER; in general the two allocations agree to the supply's sensitivity.
- The power x^k, ln(1 + z) and the fused multiply-add come from `core::num`, that is from
  the `libm` crate, so the outputs do not depend on the platform (docs/unit-1a.md §8).
  On 2026-09-26, 5000 random economies (every regime, J_b up to 12) gave byte-identical
  dump output on WSL and Windows. Before P1.1 they came from the platform libm, where glibc
  and MSVC differ in the last bits: the two platforms' outputs differed by up to 2.8e-14
  relative on the review's set, and by 1.1e-13 on x* (at x* = 9.7e-10, where the root is
  ill-conditioned) on the 5000. The move to libm changed no regime and no flag on that
  set, moved the interior outputs (residuals aside) by at most 7.0e-16 relative from the
  glibc build, and left the goldens' error maxima as they were (docs/unit-1a.md §6); the
  exact-tie input of G8 still ties. Cross-platform equality is recorded, not gated
  (ADDENDUM ruling 2).
