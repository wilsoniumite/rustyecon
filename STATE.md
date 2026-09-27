# STATE — rustyecon (resume point for the next session)

**Project:** rustyecon v2, the reboot: 300 years of economic history, 1750–2050, as an agent
economy whose decisions are the pinning paper's margins, checked against an equilibrium oracle.
The plan is [docs/PLAN.md](docs/PLAN.md), amended by the rulings in
[docs/reboot/ADDENDUM.md](docs/reboot/ADDENDUM.md); the Phase 0 engine contract is
[docs/ENGINE.md](docs/ENGINE.md), session 2's contract is [docs/CERTIFY.md](docs/CERTIFY.md),
the tape's schema is [docs/TAPE.md](docs/TAPE.md), and the GUI's design is
[docs/GUI.md](docs/GUI.md).
**Collaboration:** as in laborformal. Sequencing, engineering and drafting are delegated to
Claude; checks gate absolutely; direct critique over validation. The numbered decisions below
are a veto window for your one-word calls.
**State as of:** 2026-09-27, on branch `phase1`. **Phase 1 is closed** (P1.14). Units 1d, 1e
and 1f are built and verified (P1.8–P1.13), and PLAN Phase 1's gate is met item by item and green
in WSL and on Windows ("Where things stand"). Units 1b and 1c closed at P1.7, which `reboot`
took by a fast-forward at `503897e`; P1.8–P1.14 are on `phase1` alone, not merged or pushed.
`reboot` has moved on since, to `2398b6a` (the `g0` merge with G0 closed at G0.3, the
many-markets probe P2.1, the demo world), so landing `phase1` is a merge, on your word, and
`reboot`'s copy of this file says how: Phase 1's decisions and open items are renumbered after
134 and O27, so this copy's 76–119 become 135–178 and its O20–O22 become O28–O30. Beyond
these lines, this copy does not record the work done on `reboot` after `503897e`. **Phase 0 is
closed.** Session 1
closed at P0.9, on `reboot`. Session 2 closed at S2.6, on branch `phase0-s2` (S2.1–S2.6, from
`reboot` at `cf3c0ff`), since fast-forwarded into the local `reboot`, which is not pushed.
Session 2 built the certification stack, the GUI's engine asks and the probe's criteria, and
both tapes certify PASS. Before it: oracle unit 1a joined at P1.1 (O3); the GUI's design, plan
amendment A14 and R16 landed at P0.11 (O1); by your ruling of 2026-09-26 the Phase 2 probe ran
first, with verdict GO ([docs/probe/REPORT.md](docs/probe/REPORT.md)) and no fallback (decision
38); and Breakpoint B's pre-look passed beside it (S5.0, docs/spine/EYEBALL.md; decision 35).
Next, in order: landing `phase1` into `reboot`; G1, the oracle lab, whose two conditions, G0
and Phase 1's gate, are both met; then Phase 2 proper, after your rulings on the decisions that
bind it ("Next steps").

## Where things stand

**Phase 1 is closed, and its gate is green in WSL and on Windows** (2026-09-27;
[crates/oracle/README.md](crates/oracle/README.md), with the specs
[unit-1d.md](crates/oracle/docs/unit-1d.md), [unit-1e.md](crates/oracle/docs/unit-1e.md) and
[unit-1f.md](crates/oracle/docs/unit-1f.md)). Units 1d, 1e and 1f were built as 1b and 1c were:
a spec from laborformal `31b3482` checked against a 70-digit mpmath prototype in scratch; a
build whose goldens come from a committed mpmath generator computing from the equations; one
adversarial pass (an independent derivation that does not read the crate, and mutation
testing); one fix round, each fix with a test; and one re-check of exactly the fixed items. Each
pass found something: a blocker in 1d, a major error in 1e, two blockers and a major error in
1f. The re-checks confirmed every fix and left a few mutants alive, which are recorded (O20),
not fixed, as the bounded verification has it. Each unit nests the earlier ones bit for bit in
its trivial case (decision 63). Only `crates/oracle` changed since `503897e`, besides this file
and the root README.

| Commit | What landed |
|---|---|
| `a2a9b93` P1.8 | unit 1d, worker types and the wall, with its spec (docs/unit-1d.md, its §12 the build's departures), `generate_1d.py` and `goldens_1d.txt` |
| `2856982` P1.9 | 1d's verification fixes: the edge of a reserved shortage solved, not refused (the blocker); f_∞ = 0 after an exact zero is the junction; a test for each surviving mutant |
| `03c9d7a` P1.10 | unit 1e, parcels, the idle margin and the priced exit, with its spec, `generate_1e.py` and `goldens_1e.txt` |
| `a680dd4` P1.11 | 1e's verification fixes: an exit good free at r = 0 decided at the wall's end, free plots on idle land labelled `Idle`; a test for each surviving mutant |
| `c713578` P1.12 | unit 1f, households and government, with its spec (its §14 the departures), `generate_1f.py`, `goldens_1f.txt`, and `p1_gate`, Phase 1's gate item by item |
| `44d7909` P1.13 | 1f's verification fixes: a CES point beyond every double reads +∞, a CES economy's corners bisected in v, the power mean's direct sum; a test for each surviving mutant |
| P1.14 | this file, the oracle's README, the root README's status lines |

**Unit 1d, worker types and the wall** (P1.8, P1.9). Worker types share the line's capability
shape, each with an efficiency ε_i and a support ν_i (support baskets on the one basket). The
paper's human-required set H is hours per unit of each category that any pooled worker can do;
reserved tasks are hours only one type can do. Types selling on the line are one pool with one
wage per efficiency hour; a type whose reserved work takes all its hours is walled, paid a
scarcity price at its own wall, and a walk over the types settles the basket's price. The
pool's wage runs along one path: the all-human corner, 1a–1c's task line, and the wall (x* = 1,
the wage set by labour clearing above γ(1)·π, where the machine comparison no longer pins it),
with 1c's switches and ties continued onto the wall. So 1a–1c's `BoundaryNoMargin` and
`NoInteriorAtZero` are solved, and a root below 10⁻¹² is found by bisection on bit patterns.
Where a type's reserved demand reaches its workers, its supply is vertical, and the equilibrium
sits at that edge, the type's wage set through the pool's clearing (P1.9). An economy whose
excess demand changes side nowhere on the path is `LaborShort`. The constructed gate builds on
check_kset P9-i and check_pinning D1: along D1's automation path to the wall, services' price
tends to |H|·v and labour's share to 1 (SSRN Prop E.1). One type without human-required or
reserved hours is 1c bit for bit: every 1a–1c golden instance, and of 1116 random draws 600
the same equilibrium, 366 of 1c's boundary rows solved at the wall and 2 at the all-human
corner, 131 `LaborShort`, and 17 refused as 1c refuses them.
- **Tests**: 56 (10 unit, 46 gate in groups d1–d9).
- **Goldens**: `generate_1d.py` writes 269 (240 at P1.8; J1, J1b, J2 and E9 at P1.9) and
  reproduces every number in the spec's §7; its one-type form nests `generate_1c.py`'s solves
  within 8.7e-72.
- **Largest errors**: 2.0e-15 relative on the 233 goldens compared at P1.8 (F3's tie share),
  3.1e-15 on J1, J1b and E9, and 8.0e-14 on J2's trained wage (its σ 3.6e-14), a tie at the
  edge of a reserved shortage. W5's root below 10⁻¹² is within 1.1e-16 absolute (unit-1d.md
  §5.5).

**Unit 1e, parcels, the idle margin and the priced exit** (P1.10, P1.11). Land is cut into
parcels of an acreage, a quality (land service per acre) and an access: enclosed parcels are
rented, open ones are a commons, free to exit plots and closed to production. Each worker type
names its exit form: SSRN's dependence form, or main.tex's priced form s(q) = max(s₀ − q·h, s̲)
in units of one exit good, the default of the historical runs (ADDENDUM ruling 3), both under
SSRN eq 8 with the exit life's value. Plots go on the commons while it has room, then at a
shadow rent nobody receives, then on rented enclosed land, which leaves production. Past the
wall's end the path goes on to an idle stretch at zero rent, with the pool's wage as numeraire,
where 1d's `LaborShort` economies have their equilibria; no one working at any wage is
`NoMarket`. Where q crosses a type's q_enc the economy can sit at the threshold with a share of
the exiters renting (an enclosure tie). Coverage κ, q* and N_crit are reported at every
equilibrium, and check_enclosure's race plays out inside equilibria: at its worked instance
q_enc = 1.5 and N_crit = 60, with κ = 1 at N 60 and 0.75 at N 80. The priced form can make
labour supply fall along the path, so where monotonicity is not certified the count scans every
piece (`EXIT_SCAN` = 256 points). The dependence form in parcel form, and a priced form with its
exit option switched off, are 1d bit for bit: of 1423 of 1a–1d's draws, 1172 the same
equilibrium, 222 of 1d's `LaborShort` on idle land with 1d's f_∞, 21 `NotViable` and 7
`MultipleEquilibria` in both, and one refused at the walk's ceiling (unit-1e.md §12 item 8).
- **Tests**: 63 (10 unit, 53 gate in groups e1–e10).
- **Goldens**: `generate_1e.py` writes 223 (208 at P1.10; L1 and Q6 at P1.11) in about a
  minute and reproduces every number in the spec's §7; its parcel form nests fifteen of
  `generate_1d.py`'s instances within 1.6e-71.
- **Largest errors**: 2.4e-14 relative on Q5's provider baskets (80.44 − 80, a cancellation),
  2.1e-14 on Q6's f below its enclosure point, 7.0e-15 on Q2's f above its own; x*, v, P_s, Y
  and N_a within 1.0e-15.

**Unit 1f, households and government** (P1.12, P1.13). The government has SSRN A.1's
instruments: a payroll tax on gross wages, a tax on final purchases at producer value, a tax on
market rent, a uniform transfer, and a program paying (μ_w, μ_e) in work and in exit, the
transfers counted in composites at consumer prices. Its budget balances at every equilibrium:
by default the owners pay the levy that balances it (`Budget::RentRate`, from which SSRN eq 16's
τ_R = 1/κ comes out), or every rate is given and the uniform transfer is the residual
(`Budget::Dividend`). A transfer supplements the provider's support or replaces it. Producers
pay gross prices, and the government enters through one participation rule that has SSRN eq 9,
p.16, eq 27 and 1d's and 1e's forms as cases. The path's start is evaluated, and an in-work
benefit that alone overfills the economy is `SurplusLabour`. The basket is 1b's fixed one or a
CES over the categories (SSRN eq 26), whose σ = 1 case is check_pinning's A-joint household:
its viable neighbours AJ1 and AJW are goldens, and A-joint itself is `NotViable` (D(1) = 0
exactly; decisions 75 and 113). Three-taxes' ledger holds inside the full closure: (φ_w, φ_r) =
(0.6, 0.4) at TX, each tax leaving the allocation bit for bit, T6's legs and T5's circular flow
(R₀ 4.8, the multiplier 5/3). The payroll tax's incidence at G1 is ε_D/(ε_D + ε_S) = 0.88586.
With the fixed basket and no government it is 1e bit for bit: every golden instance of 1a–1e
(78), and of 1d's and 1e's 897 draws 871 the same equilibrium and 26 the same refusal.
- **Tests**: 77 (7 unit, 62 gate in groups f1–f11, and `p1_gate`'s 8).
- **Goldens**: `generate_1f.py` writes 235 (215 at P1.12; CA, CF2, CF3 and CS at P1.13) in
  about half a minute and reproduces every number in the spec's §7; its household form of six
  of 1e's instances equals `generate_1e.py`'s solve to 0 at 70 digits.
- **Largest errors**: 1.9e-15 relative on the 444 golden comparisons at P1.12 (CW's wage near
  the wall's real-wage ceiling, 2.5e-16 since P1.13), and 3.7e-15 on P1.13's goldens (CF3's P at
  v 4.3e8). The three values computed by finite differences are within their 1e-8: the
  incidence share 6.7e-10, ε_D 1.9e-10, ε_S 2.2e-11.

**The verification**, per unit (one pass, one fix round, one re-check of the fixed items):
- **1d** (P1.9; re-checked on `2856982`). The derivation, with the pool's wage v as its unknown
  and the walled set enumerated, found a blocker: where a type's reserved demand reaches its
  workers its supply is vertical, and the wage that clears the pool is an equilibrium, which
  P1.8 refused as `LaborShort` (7 of the derivation's 60 targeted draws, 2 of d7's). It is
  solved now, and the saturated knife edge with f(1) = 0 is the junction. The mutation pass left
  eleven survivors; each has a test, and the fix round's 20 mutants are killed. The re-check's
  derivation agreed with the oracle on 850 economies, 33 of them at the edge of a reserved
  shortage and 66 ties, within 1.2e-13 (a tie's σ within 8.9e-12, where unit-1d.md §5.5 puts it),
  and with the fix taken out 13 of those edges are refused. Its mutation run killed all eleven
  survivors; of six variants of them and seven mutants of the fix, seven survive, and the
  re-check's own probe tests kill four of these (O20).
- **1e** (P1.11; re-checked on `a680dd4`). The derivation found that an exit good made of land
  alone, free at r = 0, put its plot-takers on the floor there while they rented on the wall, so
  the idle stretch did not start where the wall ends, and L1, with one equilibrium, was refused
  as three. Such a good's plots are now decided at the wall's end (decision 101). It also found
  free plots on idle land labelled `Enclosed`; they are `Idle` (102). Thirteen of the pass's 30
  mutants survived; each has a test, and six mutants of the fixes are killed. The re-check's
  derivation agreed on 1,092 draws (189 on idle land, 120 of them decided in the wall's-end
  frame, 15 ties) within 5.1e-14, with 48 invalid in both. Its mutation run killed the thirteen
  and 11 of 13 mutants of the fix; the two left change the frame's price, which the gate cannot
  tell, and the re-check's probe kills both (O20).
- **1f** (P1.13; re-checked on `44d7909`). The derivation agreed on 566 equilibria within
  1.8e-13 and found two blockers under a CES basket: at the all-human corner a bisection midpoint
  near v = 1e-154 overflowed Y (σ ≥ 2), and economies with one equilibrium were refused as
  `NonFinite`; on a wall far out the corners' parameter ω = v/P_z lost 3.8e-11 at v 4e6 and was
  refused at 4e8. Such a point now reads +∞, and a CES economy bisects its corners in v. A major
  error: the power mean's ln1p form cancelled with a small weight at large σ (2.6e-11 in P at σ
  20); it takes the direct sum there. Nine of the pass's 30 mutants survived, two equivalent;
  the other seven have tests, and seven mutants of the fixes are killed. The re-check's
  derivation agreed on 593 economies of the three fixed items within 5.7e-13 (a provider's
  receipts on a wall at v 9e8; walls out to v 1.4e46) and killed its six mutants of the fixes;
  six more economies, at σ = 64 with eq 26's weights, were refused as invalid (O21). Its
  mutation run killed 16 of 18 mutants of the seven new tests; the two left are O20's.

**Phase 1's gate, item by item** (PLAN Phase 1; checked at P1.14). `p1_gate` has one test per
item, which checks the item on its own instances, and a table naming every test that covers it
in full; `p1_gate::the_checklist_names_real_tests` fails if a named test is renamed or removed.
Every test the table names passed in `scripts/gate.sh` on WSL and on Windows, on `44d7909` and
again on the committed tree at P1.14; the count is of those tests, `p1_gate`'s own included.

| Gate item | Met | Tests |
|---|---|---|
| the SSRN Appendix B instance: x* 0.86315, v 0.54344, Y 7.88061, N_a 1.34338 to 5e-6, and ADDENDUM §5's full-precision values to 1e-12 | the published figures, both kinds of hours and N·P_s within 5e-6; x* and Y the golden's own double, v within 2.0e-16 and N_a 1.7e-16 of the 70-digit values; the same economy in 1b's, 1e's and 1f's forms bit for bit | 12 (g1, the nesting of c1, m1, d1, e1, f1) |
| the replacement closure's worked instance (c = 1, w = 3; at λ = 0, c = 0.4 and w = 1.2) | to 1e-12 as a price block (1a), per machine type (1c), and inside the full closure at TX, under every tax, and TX3 (1f) | 9 (g6, m2, f2) |
| the fork identity and the category bounds on random instances | both forms and the bounds at 1e-12 on 1b's 180 random economies and check_interior's batteries, 1c's 240, and every equilibrium of 1d's 300, 1e's 480 and 1f's 341 draws | 19 (c5, c6, m6, d7, e8, f9) |
| the income identity to 1e-12 | at every equilibrium of every unit's draws (1a's with interest and build lags), and with a government, (I1)–(I4), at every 1f golden instance | 12 (g5, c5, m3, m6, d7, e8, f9) |
| three-taxes' ledger, (φ_w, φ_r) = (0.6, 0.4) on its worked instance | (3/5, 2/5) to 1e-12 in the price block, and for TX's machine service and basket inside the full closure under every tax, with T6's legs and T5's circular flow | 9 (g7, m2, f2) |
| constructed wall and interior cases recognised correctly | the line, the wall, the all-human corner, a root below 10⁻¹², idle land, the edge of a reserved shortage, an enclosure tie, a CES wall, and the refusals (`NotViable`, `MultipleEquilibria`, 1d's `LaborShort`, `NoMarket`, `SurplusLabour`), each on a constructed economy against its goldens, with exact zeros at every junction | 77 (g8, c7, m7, d2–d8, e5, e9, f5, f8) |
| each exit form on its own gate, the 1d and 1e gates constructed | the dependence form on 1a's Appendix B, Figure 3 and automation path, and on 1d's constructed gate (check_kset P9-i, check_pinning D1); s(q) on 1e's (check_pinning P3, check_enclosure N-i to N-iii: q_enc 1.5, N_crit 60); both nest with the exit option off, and keep their gates under a government | 91 (g1–g3, d2–d9, e1–e5, e10, f1, f4–f6) |

**The gate at P1.14**, `scripts/gate.sh` in WSL (`CARGO_TARGET_DIR=/root/scratch/target-p1-close`)
and under Git Bash on Windows (`D:/rustyecon-targets/p1-close`), on the committed tree with a
clean build stamp: 745 tests pass in the workspace on each machine, with 2 ignored and run by
name, and zero warnings. Session 2's table below holds for every crate but the oracle, which has
438 (84 unit, 353 gate, 1 doc): 114 of 1a, 59 of 1b, 69 of 1c, 56 of 1d, 63 of 1e and 77 of 1f.
The count grew 549 (P1.7) → 601 (P1.8) → 605 (P1.9) → 660 (P1.10) → 668 (P1.11) → 739 (P1.12)
→ 745 (P1.13), and P1.14 changes no code. The committed certificates recompute byte-equal on
both machines, the probe's report pins hold, and the gate world's final hash is still
`0x61f9c8529131ff17` on both. All six generators pass `--check` under laborformal's venv
(`generate_1e.py` takes 49 s, `generate_1f.py` 28 s). The gate's golden comparisons, logged
again on Windows at the close, give the largest errors above. The logs are in
`D:/rustyecon-p1/close/phase1/`.

**Phase 1's units 1b and 1c were closed at P1.7, with the gate green in WSL and on Windows**
(2026-09-27; [crates/oracle/README.md](crates/oracle/README.md), with the specs
[unit-1b.md](crates/oracle/docs/unit-1b.md) and [unit-1c.md](crates/oracle/docs/unit-1c.md)).
Each unit had a spec written from laborformal `31b3482` and checked against a 70-digit mpmath
prototype in scratch; a build whose goldens come from a committed mpmath generator computing
from the equations; an adversarial pass (an independent derivation that does not read the
crate, and mutation testing); a fix round, each fix with a test; and a re-check of exactly the
fixed items. Unit 1c had two passes, and the second found a blocker. Only `crates/oracle`
changed since `397d7cd`.

| Commit | What landed |
|---|---|
| `30ff1ce` P1.2 | unit 1b, many categories and the fork, with its spec (docs/unit-1b.md, its §12 the build's departures), `generate_1b.py` and `goldens_1b.txt` |
| `f4de477` P1.3 | 1b's verification fixes: four surviving mutants caught, precision near an interior edge stated and measured by 24 new goldens; no code change but `ces_share` checking α through `Requirement::OpenUnit` |
| `c9b1920` P1.4 | unit 1c, many machine types and the Leontief inverse, with its spec (docs/unit-1c.md), `generate_1c.py` and `goldens_1c.txt` |
| `9e2a7e6` P1.5 | 1c's first verification: five surviving mutants killed by new tests; code unchanged |
| `5081345` P1.6 | 1c's second verification: the blocker fixed (a boundary regime hid equilibria), seven tests, precision at ties stated |
| P1.7 | this file, the oracle's README, the root README's status lines |

**Unit 1b, many categories and the fork** (P1.2, P1.3). Categories are bought in a fixed
basket (SSRN eq 7; Appendix B's z) and share 1a's task line, cut into segments, each category a
density of tasks on them with its own direct land. Space is a category with no tasks, so
Appendix B is the basket (1, h) over the good and space. Every equilibrium reports, per
category, the fork identity in both forms (SSRN eq 12 with total requirements, main.tex's with
direct land), the category bounds, the purchasing-power pair, price-side and clearing-side
totals and the basket's rent ceiling. Without an equilibrium, the price block prices categories
given as task cells, closed cells included (check_interior's setting), and gives SSRN eq 26's
CES share. A one-category economy solves to 1a's equilibrium bit for bit: every 1a golden
instance, G5's 180 random draws and every skipped one, G8's regime rows and rejections, and
`at(x)` on a grid. Along the task-automation path of the four-category fork economy, the wage in
manufactures rises 27% while the wage in shelter falls 96%.
- **Tests**: 59 (5 unit, 54 gate in groups C1–C8), among them check_interior's two price-block
  batteries and 180 random multi-category economies, where every identity, both fork forms, the
  bounds and the pair hold.
- **Goldens**: `generate_1b.py` writes 273 (249 at P1.2, 24 near-edge at P1.3). In mpmath its
  category form equals `generate.py`'s 1a solves exactly.
- **Largest errors**: 1.0e-15 relative on the 245 numeric goldens away from an interior edge
  (C6's parity cost; 9.0e-16 on an equilibrium value). Near an edge x*, v, P_s, Y and N_a are
  within 1.9e-16, and the outputs made of the sliver between x* and the edge miss by up to
  7.5e-8 at 1e-9 from it, within the bound docs/unit-1b.md §5.4 states (about 2^-53·x*/d).

**Unit 1c, many machine types and the Leontief inverse** (P1.4–P1.6). Each machine type has an
operating recipe and a build recipe over machine services, labour and land, its own δ and build
lag, and so its own user cost, at one interest rate (check_dynamics R1–R6); categories may use
each other as intermediate inputs. The types share the line's capability shape, each with a
task efficiency θ; the cheapest delivered cost takes the machine tasks, switching along the
line at closed-form points. An equilibrium on a switch is a tie, solved by the share of tasks
each type takes, and more than one equilibrium is refused (`SolveError::MultipleEquilibria`).
Every linear system is solved by Gaussian elimination without pivoting, in index order, whose
positive pivots are the productivity and viability tests. One type nests 1b and 1a bit for bit
(every golden instance, the 360 random draws of G5 and C5 with the skipped ones, the regime and
rejected rows, `at(x)` on a grid), and a flow-only type is 1a's flow economy bit for bit at any
(ρ, δ, J_b). check_dynamics' sloped and flat targets are reproduced as the machine-price block
and the quantities per unit of the good, the JSON's doubles within 2e-15, except the sloped x*
and the flat m (decision 75). The income identity with interest holds to 1e-12, and 240 random
interior economies (35 of them ties) satisfy every Leontief identity.
- **Tests**: 69 (10 unit, 59 gate in groups m1–m8).
- **Goldens**: `generate_1c.py` writes 210 and reproduces every number in the spec's §7; its
  one-type form nests `generate_1b.py`'s and `generate.py`'s solves within 6.1e-71.
- **Largest errors**: at P1.4, 191 of the 207 goldens the oracle computes were within 1.2e-15
  relative and 199 within 1e-14. The rest are where docs/unit-1c.md §5.6 puts them: the switch
  points (2.8e-14, where the closed form cancels), a least pivot near 0 (1.7e-14), a tie's
  quantities (2.0e-14), and M5m's excess demand at the switch, evaluated at the double below it
  where f is steep: 4.8e-13, the largest, with 4.4e-13 on its tie share. Logged again at the
  close, the maxima are the same. On the second pass's 432 random ties, x*, γ* and v are good to
  2.2e-13 (7.5e-13 near the viability edge) and the split to 1.7e-10.

**The verification**, per unit:
- **1b** (P1.3). Mutation testing found four survivors, each now caught: the carried 1 − x* in
  a category's final hours and in λ̃_j^q, the edge convention behind `margin_active`, and
  `Requirement::OpenUnit` admitting 0. The pass also found that outputs made of the sliver
  near an interior edge lose precision; the loss is stated and measured, and carrying an offset
  from each edge was tried and does not recover it. The build's own survivor, a
  reassociation that changes only the rounding, stands. The re-check applied the three code
  mutants to `f4de477`: all killed, 173 tests passing.
- **1c, first pass** (P1.5). An independent derivation in another formulation agreed with the
  oracle on 348 economies, on every regime, technique, tie and switch count, and on the values
  within 5.6e-14. Of 65 mutants six survived; five are killed by new tests, and one is
  equivalent in exact arithmetic.
- **1c, second pass** (P1.6). A second derivation found a blocker: with labour demand jumping up
  at a switch and f(1) > 0, the build returned `BoundaryNoMargin` while a root and a tie lay
  below. The solve now reads the whole sign sequence whenever there is a switch, each corner
  one equilibrium, and such an economy is `MultipleEquilibria`; without a switch it decides as
  before, so 1a and 1b nest unchanged. The pass also found that an unused type whose price
  diverges makes the economy `NotViable` (kept, decision 72), and measured the precision at
  ties. Nine surviving mutants and six of the eight made of the fix are killed; three
  survivors are equivalent on every economy the gate can build. The re-check applied the 17
  mutants to `5081345`: all killed, 242 tests passing.

**The gate at P1.7**, `scripts/gate.sh` in WSL
(`CARGO_TARGET_DIR=/root/scratch/target-p1-close`) and under Git Bash on Windows
(`D:/rustyecon-targets/p1-close`), on the committed tree with a clean build stamp: 549 tests
pass in the workspace on each machine, with 2 ignored and run by name, and zero warnings.
Session 2's table below holds for every crate but the oracle, which has 242 (57 unit, 184 gate,
1 doc): 114 of 1a, 59 of 1b, 69 of 1c. The count grew 421 (S2.5) → 476 (P1.2) → 480 (P1.3) →
539 (P1.4) → 542 (P1.5) → 549 (P1.6). The committed certificates recompute byte-equal on both
machines, the probe's report pins hold, and the gate world's final hash is still
`0x61f9c8529131ff17` on both. `generate.py`, `generate_1b.py` and `generate_1c.py` pass
`--check` under laborformal's venv. The logs are in `D:/rustyecon-p1/close/`.

**Phase 0 session 2 is closed, and the gate is green in WSL and on Windows** (2026-09-26;
[docs/CERTIFY.md](docs/CERTIFY.md), with each step's amendments). It moved July's certification
stack into `crates/certify` with N4, N10, N12 and N15 fixed and every threshold in dated
criteria, added A12's runaway detector and the probe's three criteria, and landed the GUI's
engine asks (D10 items 1, 2 and 4).

| Commit | What landed |
|---|---|
| `785ab19` S2.1 | docs/CERTIFY.md, the session's contract, revised once after an adversarial review (C1–C13, decisions 41–53 below) |
| `7c14c37` S2.2 | the GUI's engine asks: `world_id` without the `fixed` flag, `FiredEvent.source`, each use of a param as a `Site` with its `ClockMethod`, listed by `engine::registry`; every `world_id` re-baselined once, no state hash moved |
| `0e8c2e7` S2.3 | `crates/certify`: dated criteria, the batteries behind a finite gate, windows per segment between dated shocks, the kick check through a new dated action `ScalePrice`, the transient reports, the sealed certificate and the manifest; the probe's testdata |
| `6a80d6d` S2.4 | tidy long Parquet telemetry behind certify's feature `parquet` (pure Rust); the cli's build stamp, hash output that names its run, manifests, verified resume and `rustyecon certify` |
| `1ee9b80` S2.4 | `criteria/gate-2026-09-26.ron` and `criteria/appb-2026-09-26.ron`, alone, before any certified run |
| `4d59604` S2.4 | `results/{gate,appb}/`: both certificates PASS, from a clean build of `1ee9b80` |
| `05533b9` S2.5 | the bounded verification's fixes (two blockers, nine majors, six minors), each with a test; the probe reads certify's measures, pinned to its report |
| `daa62af` S2.5 | the certificates regenerated from a clean build of `05533b9`: both PASS, every reading's value as before |
| S2.6 | this file, PLAN §3.2 (decision 39), ENGINE, CERTIFY, TAPE, GUI.md, README; the spine scripts' portable cache (O15) |

**The certificates** (`results/`, committed verdicts, made in WSL by a clean build of `05533b9`
from the criteria registered at `1ee9b80`; C3, C10):

| Tape | Verdict | `tape_hash` | `world_id` | Criteria hash | Run |
|---|---|---|---|---|---|
| `tapes/gate.ron` | **PASS** | `0x54066d053474846b` | `0x43628a8e0fd5f695` | `0x8e1ec1cc31830009` | 2,080 ticks, genesis `0xf05d0f23826edf87`, final `0x61f9c8529131ff17` |
| `tapes/appb.ron` | **PASS** | `0x8973b237f4c00029` | `0x26f12f8a0bc27540` | `0x1d00ed972691a90f` | 20,000 ticks, genesis `0x8d12ce44b614110a`, final `0xe1fa082b26995867` |

- **gate**: Conservation, Determinism (resumed at a quarter, half and three quarters), Runaway at
  1e3, Trades every year and Balance at July's bars, in five segments opened by `mine.cut`,
  `bread.line.up`, `oven.opens` and `mine.restored`. The widest price ratios to genesis are 18.3
  (town/bread) and 0.0996 (village/bread). Every market trades in each of the 40 yearly windows,
  and no market is pinned in any segment. The largest ledger margin is 3.6e-5 of its tolerance.
- **appb**: Conservation, Determinism at half the run, Runaway at 1e6, Trades and Balance as the
  gate's, plus Settles and the 1e-9 kick over one L, in one segment. No dead tick in W or F;
  price ranges in F are 0 and volume ranges at most 4.4e-16 in log. The eight kicks decay to
  gains of 1.9e-6 to 6.0e-6 in the last tenth of the 20,020-tick horizon, with peaks at most
  3.26, against bars of 1e-3 and 1e6; no kicked run failed.
- `rustyecon certify` took 0.15 s for the gate and 2.65 s for appb, kicks included (WSL,
  release). `committed_certificates_recompute` reruns both, byte-equal in WSL (gated) and on
  Windows (recorded: byte-equal there too).

**Hashes and identities.** Session 2 moved no state hash and no `prefix_id`: at S2.2 all 2,080
gate and 20,000 appb per-tick hashes were compared with `cf3c0ff`'s on both machines and are
byte-identical, and later steps changed no engine-path code but core's appended `ScalePrice`,
which moves no existing encoding. Every `world_id` changed once, at S2.2, on purpose: item 1
drops the `fixed` flag, and each spec's `Site` carries the method that decides a number. Equal on
WSL and Windows:

| World | At `cf3c0ff` | From S2.2 | Genesis state hash (unchanged) |
|---|---|---|---|
| `tapes/gate.ron` | `0xbecdc746fc86ce97` | `0x43628a8e0fd5f695` | `0xf05d0f23826edf87` |
| `tapes/appb.ron` | `0x3f689d670fe877c6` | `0x26f12f8a0bc27540` | `0x8d12ce44b614110a` |
| core's fixture | `0x66d1181c6802a7fd` | `0x85336968874fbf6d` | `0xf1538ab1f6a0de5c` |
| markets' fixture | `0xbdd0ee95c0bb590f` | `0xc3b1c948a42f06c6` | `0x5e400bb3f1012434` |

A checkpoint made before S2.2 is refused as `WrongWorld`; none is committed. The final hashes are
still gate `0x61f9c8529131ff17` and appb `0xe1fa082b26995867` on both machines.

**What was fixed** (ADDENDUM §2.3's N-numbers, REPORT §6's criteria), each with a test that fails
without it, checked by mutation:
- **N4**, a run with no criteria never certifies PASS: `unscored_run_never_passes`,
  `criteria_without_runaway_do_not_certify`, `certify_command_exits_on_its_verdict` (exit 0 is
  PASS only; FAIL and UNSCORED exit 5).
- **N10**, `tape_hash` covers every input: confirmed, not only proposed. `Tape` refuses unknown
  fields and has no defaults, so `fnv1a_64` over the canonical `to_ron` covers everything the
  loader reads (`tape_hash_covers_every_input`). The manifest records the build, `tape_hash`,
  `world_id` and any checkpoint a run resumed from; the certificate records the criteria
  (`manifest_names_every_input`).
- **N12**, fail closed on NaN: a finite gate before every predicate, NaN-propagating folds, and a
  serde scan of every number a certificate renders, where July's scan read state fields and its
  statistics dropped NaN one sample at a time
  (`batteries_fail_closed_on_nonfinite_samples`, `seal_fails_every_nonfinite_path`,
  `finite_scan_reads_every_rendered_number`, `render_prints_only_serialised_numbers`).
- **N15**, windows relative to the run and restarting at each dated shock:
  `windows_are_relative_to_the_run`, `windows_restart_at_each_dated_shock`,
  `short_segments_do_not_shrink_windows`.
- **A12**, the price-runaway detector, relative and registered: `runaway_bound_is_relative`
  (every price ×2⁴⁰ gives bit-identical readings), `runaway_detector_catches_a_runaway`.
- **REPORT §6**: the 1e-9 kick in ± each price, at the end and at every dated shock
  (`kick_check_passes_a_stable_rest`, `kick_check_fails_a_rounding_freeze`,
  `a_merged_shock_is_kicked`); troughs, dead ticks, ticks with no consumption, the transfer
  shortfall and spoilage as reported outputs (`transient_statistics_are_reported`); windows per
  shock (`a_history_whose_segments_return_passes`).
- **D10** items 1, 2 and 4: `new_source_event_keeps_world_id`, `fired_event_names_its_source`,
  `registry_names_each_use`, with `params_are_read_only_through_sites` and three more (ENGINE's
  S2.2 amendment).
- **R16** in the cli: `hash_output_names_its_run`, `resume_requires_a_recorded_checkpoint`,
  `resume_records_its_parent`. Resume already checked a checkpoint's digest since P0.9; what was
  left was "a run of the same tape": it now verifies the checkpoint against the manifest of the
  run that made it (same `tape_hash` and `world_id`, and a record at its tick with the same
  digest and state hash).

**The probe's measures (D13).** Moved to certify, oracle-free, and read by the probe's harness
since S2.5: the runaway bound, a market that traded, rationed fills, spoilage per good, the
provider's due and paid, the trough, "at rest in F" as a log range, the dead-share rule and the
ledger margin. They wait for `crates/observe`, since each needs the oracle: the target and the
gaps D̂, shock distance and κ, the envelope, VACUOUS, the hold check, the bands per relative
price, the oracle-relative dead floor (`LIVE_FLOOR`) and troughs, and the classifier. The
probe's pins (`probe_summaries_unchanged`, and `probe_battery_csv_unchanged` by name in the
gate) show its 57 battery rows, its negative control and its shock history unchanged.

**Telemetry and what pure Rust costs.** Parquet 60.0.0 with LZ4_RAW and byte-stream-split, no
zstd and no C build on either machine, behind certify's feature `parquet`. With the feature off
certify's closure is 17 crates on every target and checks for wasm32; with it, parquet adds 19
crates on Linux and 18 on Windows. Files are about 9% larger than zstd-3 (GUI §3.4's
measurement). The gate world's 2,080 ticks write 276,145 rows in 1,527,113 bytes, byte-identical
from two processes on each machine, and WSL's and Windows' files differ only in the footer's key
that names the build.

**The verification** (bounded, as session 1 taught: one adversarial pass, one fix round, one
re-check of exactly the fixed items). The pass over S2.1–S2.4 found two blockers, nine majors and
six minors; all were taken at S2.5, none rejected, each with a test checked against its mutant
(38 mutants, all killed). The blockers were false PASSes: dense dated `ScalePrice` events clamped
July's runaway (now counted against the criteria's `price_shocks`, none by default), and a regime
shorter than `min_segment` went unkicked (the kick now fires at every dated shock). The re-check
on `daa62af` found both now FAIL and a pass flag edited alone refused; one forgery of bars still
reads back, carried as O16 (CERTIFY, amended at S2.6).

**The gate at S2.6**, `scripts/gate.sh` in WSL and the same script under Git Bash on Windows:
fmt, clippy with `-D warnings` and `cargo test --workspace --release` with warnings denied, then
the repeat-hash test by name, certify without Parquet (check, clippy, test, and `parquet` absent
from its tree), two runs of the gate tape through the binary, the build stamp against the
checkout, `committed_certificates_recompute` by name and each certificate's build an ancestor of
HEAD, `probe_battery_csv_unchanged` by name, and telemetry from two processes. 421 tests pass in
the workspace on each machine, with 2 ignored and run by name, and zero warnings; certify alone,
Parquet-free, passes 65 with 1 ignored:

| Crate | WSL | Windows |
|---|---|---|
| `rustyecon-core` | 93 unit + 2 doc | 93 unit + 2 doc |
| `rustyecon-markets` | 6 unit + 28 integration | 6 unit + 28 integration |
| `rustyecon-agents` | 1 unit + 21 integration | 1 unit + 21 integration |
| `rustyecon-engine` | 3 unit + 46 integration + 10 doc | 3 unit + 46 integration + 10 doc |
| `rustyecon-cli` | 18 integration | 18 integration |
| `rustyecon-certify` | 9 unit + 60 integration (1 ignored, run by name) | 9 unit + 60 integration (1 ignored, run by name) |
| `rustyecon-oracle` | 42 unit + 71 gate + 1 doc | 42 unit + 71 gate + 1 doc |
| `rustyecon-probe` | 10 integration (1 ignored, run by name) | 10 integration (1 ignored, run by name) |
| `rustyecon-worldgen` | none yet | none yet |
| **Total** | **421** | **421** |

The count grew 330 (P2.0.2) → 339 (S2.2) → 398 (S2.3) → 409 (S2.4) → 421 (S2.5). The wasm32
checks of the engine and of Parquet-free certify pass in WSL (recorded, not gated); Windows has no
wasm32 target. The logs of each step are in `D:/rustyecon-s2/`.

**Phase 0 session 1 is closed, and its gate is green in WSL and on Windows.** It salvaged
`core`, `markets` and `cli` from the July branch, tag `july-v2-phase-3` (`ff01284`), fixing
each listed defect as the code moved, and added `agents` (the scripted actor) and `engine` (the
library a frontend drives). The v1 and July agents did not carry.

| Commit | What landed |
|---|---|
| `fc3a2c0` P0.1 | v1 and the July design archived from HEAD (175 files, reachable at `pre-reboot-2026-09-25`); the workspace, `clippy.toml`, `rust-toolchain.toml` |
| `30a0795` P0.2 | docs/ENGINE.md, the contract, with the frontend requirement of 2026-09-25 (invariants E1–E8); an empty `crates/engine` |
| `3ff1ceb` P0.3 | `rustyecon-core`: typed holders, lots that keep their lives and coalesce, all-or-nothing takes, the ledger, the hash, checkpoints, the tape loader, `num` (libm), `Clock`; docs/TAPE.md |
| `88762c4` P0.4 | `rustyecon-markets`: budgets bind at admission, one fill settles both sides through an escrow, rationing per class, the price update with its rule from the tape |
| `a8f9ed8` P0.5 | `rustyecon-agents`, `rustyecon-engine`, `rustyecon-cli`, and `tapes/gate.ron`, the gate world |
| `7553a3d` P0.6 | fixes from adversarial review, round 1: checkpoint digest (format 2), schedule params outside `world_id`, new load checks, stronger tests |
| `443d44c` P0.7 | fixes from adversarial review, round 2: `Rounding` declared by TwoSum, the run's ledger, the engine no longer re-exports core |
| `4553e5f` P0.8 | housekeeping: docs/PLAN.md moved and amended, this file, the CI skeleton, the docs checked against the code |
| `57f3a25` P0.9 | fixes from adversarial review, round 3 (O5–O13): exact multi-lot rounding, the flow tolerance pinned, checkpoint format 3 (identity and the run's ledger in the digest), dated events in date order, the per-tick conversions pinned, a wider frontend guard and literal scan, the final save tested |
| `cc8bae2` P1.1 | oracle unit 1a joins the workspace (O3): `crates/oracle`, its maths through `core::num`, which gains `fma` |
| — | P0.10 is unused: it was held for a fourth fix round, which the bounded check of P0.9 did not need |
| `5d7efe9` P0.11 | the GUI's design, docs/GUI.md, and its review ledger; plan amendment A14 (ADDENDUM §6, rulings 5–8), R16 and the G-stages in PLAN, ENGINE §13; decisions 22–34 below (O1) |
| `408b4e8` P0.12 | docs made consistent after the final check: G10 beside Phase 10 as a third exception, stale session and hash lines in GUI.md, ruling numbers, a machine path in the ledger |
| `58e9e98` S5.0 | Breakpoint B's pre-look (2026-09-26): the spine's fetch and validation scripts, manifests, BNS's CC0 files, docs/spine/DATA_NOTES.md and EYEBALL.md; no third-party data or figures (decisions 35–37) |
| `3a4b28c` P2.0.1 | from branch `phase2-probe` (2026-09-26): the Phase 2 probe's build, the four Appendix B roles in agents, `crates/probe` and `tapes/appb.ron`; docs/probe/RULES.md; no core, markets or engine change, and the gate world's hash is unchanged |
| `6d8d2a5` P2.0.2 | docs/probe/REPORT.md, the probe's report, with its plots and three results tables; docs only |
| `e6d9ff9` P2.0.3 | this file after the probe; the gate rerun on both machines |
| `cf3c0ff` P2.0.4 | `phase2-probe` and `spine-eyeball` merged into `reboot` (rebased, fast-forward); decisions 35–40; this file |

Session 1 closed at P0.9 with 198 tests (186 `#[test]` functions and 12 doc tests) passing on
both machines and the gate world's final hash `0x61f9c8529131ff17` on both. P1.1 and P0.11 came
after it: the oracle, and the GUI's design, which is documents only.

**Fixed as the code moved** (REVIEW §2.2's numbers, ADDENDUM §2.3's N-numbers): defect 5 (lot
lives through checkpoints), 8 (settlement from one fill), 9 and N3 (a shortfall is a ledger
line and stops the run), 10 (a desk and a pop that share a number are distinct holders); N1
(events sorted; `every = 0` refused), N2 (ids checked in every profile), N5 and N6 (no absolute
epsilon anywhere), N7 (cash binds at the order), N8 (no forgiveness), N9 (lots coalesce by
life), N11 (checkpoints carry `world_id` and `prefix_id`; resume is a product path), N13 (the
price rule and the one-sided rule come from the tape, with no default) and N14 (core holds no
agent struct). Each has a test that fails when its fix is reverted, checked by mutation.

**The gate** (PLAN Phase 0 gate, as restated by A3), each requirement with its tests:

| Requirement | Tests |
|---|---|
| repeat | `gate_repeat_identical_hashes` (engine; also run by name in `scripts/gate.sh`, which then runs the binary twice and compares the hash files) |
| resume, through the product path (N11) | `gate_resume_from_checkpoints` (engine, both formats; every resumed report equal in full, the run's audit included), `resumed_run_stops_where_the_uninterrupted_run_does` (engine), `gate_resume_through_product_path` (cli) |
| replay | `gate_replay_matches_every_tick` (engine), `replay_command_passes_on_the_gate` (cli) |
| conservation every tick (R2) | `gate_conserves_every_tick`, `gate_breach_stops_the_run`, `gate_rounding_is_declared` (engine), `multi_lot_rounding_is_declared_exactly`, `flow_tolerance_is_pinned_for_a_tick_and_a_run` (core) |
| a cash-short buyer settles both sides from one fill | `cash_short_buyer_settles_both_sides_from_one_fill` (markets) |
| two actor kinds sharing an id settle apart | `desk_and_pop_sharing_a_number_are_distinct_holders` (core), `two_kinds_same_number_settle_apart` (markets), `gate_ids_apart` (engine) |
| a burn shortfall stops the run | `burn_shortfall_stops_with_a_ledger_line` (core), `shortfall_stops_the_run` (cli) |
| unsorted events fire or fail | `unsorted_events_fire_in_order`, `every_zero_is_rejected` (core), `gate_events_fire_in_date_order`, `same_tick_events_fire_in_date_order` (engine) |

The full list, with what each test checks, is ENGINE §11: 186 `#[test]` functions and 12 doc
tests at P0.9. Since P1.1 core has one more (`fma_rounds_once`) and the oracle brings its 114,
so `cargo test --workspace --release` passed 313 of 313 on both machines at P1.1. Session 2's
count, 421, is in its table above. The oracle's 114 are its own gate
(crates/oracle/README.md), and `goldens/generate.py --check` passes under laborformal's venv.

**Toolchain** (pinned by `rust-toolchain.toml`, installed by rustup on first use):

- WSL Ubuntu 22.04.4 (glibc 2.35): `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
  `1.97.1-x86_64-unknown-linux-gnu`; `cargo 1.97.1 (c980f4866 2026-06-30)`.
- Windows 11, MSVC: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, `1.97.1-x86_64-pc-windows-msvc`;
  `cargo 1.97.1 (c980f4866 2026-06-30)`.

**Hashes, Linux against Windows (recorded, not gated): identical.** All 2,080 per-tick hashes of
`rustyecon run tapes/gate.ron --until 2080 --hashes` agree byte for byte, and two runs on each
machine agree with each other. Tick 1 `0xf2371ea73f47ee1f`; 520 `0xb985a06b853fa899`; 1,040
`0x30f84912c6a744f9`; **2,080 `0x61f9c8529131ff17`**. At P0.3 the core fixture agreed too
(`world_id` `0x5482a99c926bdef7`, genesis state hash `0x7060047573ff37da`, and `0x5121b67d1feee116`
after 2,080 ticks of `num::exp`-driven price and EMA paths). The gate world's final hash at P0.5 was
`0x1b86507a195b2a40`; P0.6 changed what the state holds (schedule params left it), and P0.7 changed
no hash. Nor did P0.9: all 2,080 per-tick hashes equal P0.8's byte for byte (compared on WSL against
a build of `4553e5f`), and the Windows build's stream equals the WSL one byte for byte (final
`0x61f9c8529131ff17` on both). P0.9 changes what a checkpoint holds (format 3) and which ledger
lines a tick declares, not what a state holds or how a tick moves it; the gate world has no two
firings in one tick. P1.1 moved no hash: its 2,080 per-tick hashes equal P0.8's byte for byte on
WSL and on Windows. P0.11 changes no code. Nor did the probe or session 2 move a state hash
(above); session 2 moved every `world_id` once, at S2.2.

**The oracle, unit 1a (P1.1; O3).** Built and verified by its own run, it joined through the
members glob: 114 tests, the SSRN Appendix B to its published figures and to 70-digit goldens
at 1e-12 relative. Its maths goes through `core::num`, so its outputs no longer depend on the
platform: 5000 random economies (every regime) gave byte-identical output on WSL and Windows.

**The GUI's design (P0.11; O1).** [docs/GUI.md](docs/GUI.md), reviewed twice
([ledger](docs/reboot/GUI-review-ledger.md)), under your rulings of 2026-09-25 (ADDENDUM
rulings 5–8): egui in `crates/gui`; the shell right after Phase 0's two sessions, with Phase 1
in parallel; Yorkshire in its three ridings, 42 regions, the atlas under ODbL with attribution
in its own data directory; R16 in PLAN §4. The lead engineer's decisions D1–D13 are 22–34
below.

**Phase 2 probe (2026-09-26; P2.0.1–P2.0.3).** Your ruling of 2026-09-26: before session 2 and
the GUI, a time-boxed probe of the project's biggest risk, whether agents reach the oracle's
equilibrium, with Breakpoint B's eyeball test run in parallel; then session 2, then G0. The
probe ran on branch `phase2-probe`, since merged into `reboot`: a frame (PROBE-SPEC, in `D:/rustyecon-probe/frame/`), three rule designs judged three
ways, the build (P2.0.1), a registered battery, a 117-cell dial sweep and two reviews.

- **Verdict: GO** ([docs/probe/REPORT.md](docs/probe/REPORT.md)). At design-analytic-first's C2,
  all 57 registered runs return to the oracle's point, within 1.03e-14 in log, from every price
  ×2 or ÷2, x\*/2 and cost shocks b′ = 0.2–0.8, in 268–677 weekly ticks. July's step rule, run
  in the same engine as the negative control, diverges 57/57.
- **The reviews confirm it and narrow it.** Two sweep cells scored GO were rounding freezes at
  unstable points, so a converged run must survive a 1e-9 kick. The transients are violent: a
  12% fall in equilibrium output costs 85% on the way, and some shocks bring ticks with no
  consumption. The instance is the flow benchmark; durability and interest are untested.
- **The kill condition (A11) is not met.** The report proposes, for your ruling: no fallback;
  the cash rule as PLAN §3.2's scale rule; A9's July battery made optional; three criteria for
  session 2 (a kick check, transient statistics, windows per dated shock); `tapes/appb.ron` as
  G0's second world, with D2's "no log axes" revisited. The first three are decisions 38–40,
  and PLAN §3.2 carries decision 39 since S2.6. Session 2 built the three criteria.
- **Breakpoint B**, in parallel: its pre-look landed as S5.0 on `reboot` (`58e9e98`,
  docs/spine/EYEBALL.md) and passes (decision 35).
- **The gate at P2.0.2** is green on both machines: `scripts/gate.sh` in WSL, and fmt, clippy
  with `-D warnings` and `cargo test --workspace --release` on Windows. 330 tests pass on each
  (P1.1's 313, plus 10 role tests in agents and 7 in `crates/probe`), with zero warnings. The
  gate world's 2,080 per-tick hashes, and the 20,000 of `rustyecon run tapes/appb.ron --until
  20000 --hashes`, are byte-identical on the two machines; the final hashes are
  `0x61f9c8529131ff17` and `0xe1fa082b26995867`, as at P2.0.1.

**WASM (recorded, not gated):** `cargo check --target wasm32-unknown-unknown -p
rustyecon-engine` passes in WSL, so the engine can compile for a browser frontend (E2). Since
S2.4 so does `-p rustyecon-certify` with its `parquet` feature off, the web build's manifest.

**CI:** `scripts/gate.sh` is the gate as one script (WSL or any Linux; build outside the tree).
`.github/workflows/ci.yml` runs it on GitHub's `ubuntu-latest` on every push, and whether hosted
CI is wanted at all is your call (A5). Since S2.4 it checks out full history, since the gate
checks that each committed certificate's build commit is an ancestor of HEAD. Windows runs the
same script under Git Bash, by hand.

**Remote** (as the local remote-tracking refs show on 2026-09-27): `origin` has `main` at
`f614792`, `reboot` at `cf3c0ff`, the three `july-v2-*` tags and `pre-foundations` (A1 done).
Not pushed: the local `reboot`, at `2398b6a` when P1.14 was made (S2.1–S2.6, P1.2–P1.7 and
the work since recorded in its own copy of this file), `phase1` (P1.8–P1.14), and the local
branches whose work is in `reboot`.

## Decisions — veto window (your one-word calls)

The first nine record how your rulings and the standing rules were carried out; 10–21 were made
while building. 22–34 are the GUI's D1–D13 (A14; [docs/GUI.md](docs/GUI.md) says where each is
carried out): each stands unless vetoed before G0. D10's window closed with session 2, which
built its items. 41–58 are session 2's. 59–75 are Phase 1's units 1b and 1c, and 76–119 its
units 1d, 1e and 1f (on `reboot` these become 135–178 when this branch lands).

1. **Package names and layout.** Each crate is `crates/<short name>`, package
   `rustyecon-<short name>`; the root is a virtual workspace, `members = ["crates/*"]`, resolver
   2, edition 2021, version 0.2.0, `publish = false`, no licence (the old manifest had none).
   Alternative: bare names, which fails for `core` (it shadows Rust's `core`).
2. **One toolchain, two machines.** 1.97.1 with rustfmt and clippy, minimal profile. WSL is
   primary and Windows secondary; both must be green with zero warnings and a clean clippy;
   cross-platform hash equality is recorded here, never gated.
3. **Time is registered.** `ticks_per_year` is in the tape's header; every param carries a unit
   (`Dimensionless`, `Years`, `FlowPerYear`, `RatePerYear`, `CompoundPerYear`,
   `FractionPerYear`); `Clock` does every conversion, and dates map to ticks in integers over the
   mean Gregorian year (ENGINE §6). The gate tape registers the EMA span as 0.5 years.
4. **No absolute epsilon.** The ledger's tolerances are registered, fixed, dimensionless params
   (`rel_flow` 1e-12 and `rel_stock` 1e-11 on the gate tape, July's values), refused at load if
   1 or more; a price is re-emitted only when its bits change; test bars are relative and named
   in their files.
5. **One maths module.** `exp`, `expm1`, `ln`, `ln1p` and `pow` go through `core::num`, backed
   by `libm`; `clippy.toml` denies the platform transcendentals, `powi` and `mul_add`, and the
   std hash containers (R8). Since P1.1 `num` also has `fma`, libm's correctly rounded fused
   multiply-add, for the oracle; the state path still writes `a * b + c`.
6. **No behavioural literal in shipped code.** A source scan allows only `0.0` and `1.0`, no
   named float constant outside `core::num`, and, since P0.9, no integer but 0 and 1 made float
   (clock.rs's calendar constants may be).
7. **Conservation is asserted.** Takes are all or nothing; a shortfall is a ledger line and stops
   the run; every tick's ledger and the run's ledger must close within the registered
   tolerances. Lots stay `f64`, and what a split or merge rounds away, and what a burn's sum of
   several lots rounds away, is declared with the reserved provenance `Rounding`, measured by
   TwoSum, one line per rounding, so each delta's declarations are exact (P0.9). The run's
   ledger travels in checkpoints. Alternatives: integer quanta (gives up the range of tiny and
   huge prices), or exact takes (makes payments non-nominal).
8. **Rationing and costs.** Rationing is a `RationLine` per (market, class, side) (R12); a
   currency in a recipe is a load error (R14).
9. **Agents see only their view.** A `View` holds posted prices, the agent's own holding and
   state, and the current params; agents never depend on the oracle (R13).
10. **An engine crate, for the GUI.** `crates/engine` owns the run: `Sim` steps, checkpoints,
    resumes and audits replays, and hands out an owned, read-only `TickReport` per tick. No
    frontend mutates state (every intervention is a tape event or a tape edit and rerun), no
    global state or I/O, every type `Send`. The cli is a thin binary over it. PLAN Phase 0 step
    3's crate list does not name `engine`; it is left as written, since only the addendum's
    rulings were folded in.
11. **The price rule** `Imbalance` is `p·exp(k·x)` with `k` = rate·Δ, which is tick-invariant,
    in place of July's `p·(1 + α·x)` (ENGINE §14 question 1). Alternative: July's form.
12. **Price and one-sided rules are required on the tape**, with no default (N13, F8). The gate
    world uses `Imbalance` with `Hold`; `Saturate` against `Hold` for the historical runs is
    Phase 2's call.
13. **Settlement.** One filled quantity per order, through one escrow per market; the largest
    taker goes last and takes `All`. Soonest-expiring lots leave the escrow first, which gives
    the oldest to the lowest id (ENGINE §14 question 4).
14. **Spoilage is core ageing in phase 5a**; the agents' upkeep hook in 5b is a no-op in Phase 0
    (ENGINE §14 question 2).
15. **How far R4 reaches.** Every dial with a time unit, every rate and every tolerance is a
    named param; dimensionless structure (recipe coefficients, weights, genesis stocks and
    prices, event quantities) sits inline under its entry's `basis`. A `SetParam` copies another
    param by key, never a literal, so every new value has a basis (ENGINE §14 question 3).
16. **The tape's identity.** Schema 1: keyed lists, unknown and missing fields refused, no
    defaults, everything ordered by key. `world_id` leaves out the tape's name, the basis texts
    and the schedule; a param only the schedule reads lives outside `world_id`, so a dated dial
    change keeps earlier checkpoints. Events in one tick fire by (date, key), so a tape's meaning
    does not change with its tick length (P0.9); a recurring occurrence is dated its tick's first
    day. Alternative: refuse two same-tick firings of one target at load.
17. **Checkpoints.** Format 3, bincode or RON, carrying the run's ledger, with an FNV-1a digest
    of `(world_id, prefix_id, state, run)`: it catches corruption and edits of any field, not
    forgery (ENGINE §14 question 7). The alternatives were a keyed digest (needs a secret) or a
    replay from genesis on every resume; for the run's ledger, stating the limit and re-auditing
    from genesis in session 2's manifest.
18. **Phase 0 trades across nodes for free.** An actor may post at any node; nothing crosses a
    channel or pays a crossing cost until transport desks exist.
19. **The gate world** is `tapes/gate.ron`: coin, grain, fuel and three-week bread; a town and a
    village; four desks (farm, mine, mill, a dormant oven) and two pops (pensioners, workers);
    four dated events and a yearly pension; 2,080 weekly ticks. Its numbers are `Assumed` and
    tuned to one bar: every market trades every year, and no price leaves [1e-3, 1e3] of genesis.
20. **CI as one script.** `scripts/gate.sh` plus a GitHub workflow that runs it, in place of
    ENGINE's `ci/gate.sh` and `ci/gate.ps1`; the Windows comparison is by hand.
21. **Two dependencies beyond PLAN's list:** `clap` 4.6.1 (cli only) and `tempfile` 3.27.0 (cli
    tests only), July's versions.
22. **D1, the GUI never gates engine work.** `default-members` leaves `crates/gui` out; engine
    steps run clippy and test with `--exclude rustyecon-gui`, plus one non-blocking WSL
    `cargo check -p rustyecon-gui` whose result is noted here. Measured clean on WSL: the check
    takes 15 s, a 0.76 GB target directory and 0.88 GB peak RSS; a release build 28 s, 1.1 GB
    and 1.02 GB; a kittest debug build 2.8 GB and 1.71 GB. The GUI's own gate script gates
    G-stages only, and an engine-caused break is fixed by the next G-stage. Cargo resolves one
    lockfile for the workspace, so the commit that adds `crates/gui` fetches its closure on both
    machines, and lockfile changes land at G-stage boundaries.
23. **D2, G0's timing and cut.** G0 follows both Phase 0 sessions, and G1 starts after G0 and
    Phase 1's gate. G0.1 is the viewer: `ThreadDriver` only, with no watchlist, no event or date
    breakpoints and no log axes. G0.2 is the editor, branches, compare and export.
24. **D3, edits carry a required note, never a basis.** `materialise` stamps
    `Assumed("GUI experiment <date>: <note>")` on each entry it adds or changes, and marks every
    branch's name `[GUI experiment <date>]`, so a branch made only of removals is marked too.
    Removals carry notes, and `RemoveParam` exists. "Save tape as" writes a lineage file, which
    every export includes: the nearest saved ancestor and every edit since. Certify's scorecard
    (Phases 6–7) scores only tapes whose tape hash the fitting harness registered before the run,
    and refuses any tape whose name or any basis carries the marker
    (`scorecard_refuses_gui_edited_tape`).
25. **D4, resumes are verified.** Only from checkpoints a Runner of this process took, or from a
    file whose `state_hash` equals the hash recorded for that state tick, from an index or
    manifest naming the same tape hash and `world_id`. Otherwise the GUI reruns from genesis and
    logs why.
26. **D5, "origin", not "tier".** "Tier" stays with the book. The GUI's field is "origin" (run,
    experiment, record, oracle), beside "registry" (scripted, emergent).
27. **D6, exact pins.** The egui crates are pinned with `=` and upgraded only between G-stages.
28. **D7, region keys.** Proposed as `county.<chapman>`, the ridings `county.ery`, `county.nry`
    and `county.wry`. Phase 4 fixes them.
29. **D8, when the map arrives.** At G4, with Phase 4 or with Phase 5's first county series,
    whichever comes first. A schematic view carries lenses from G2.
30. **D9, levels wait.** Levels (keyed dial values with unit and basis, outside `world_id`) wait
    for a schema bump before Phase 3's long runs. Until then a new dial value is a new param,
    rerun from genesis.
31. **D10, G0's engine asks land in session 2** (O2; docs/GUI.md §7.2).
32. **D11, wasm hashes are recorded, not gated.**
33. **D12, trigonometry only in `ui/`.** f32 trigonometry is allowed only in the GUI's `ui/`,
    under `#[expect(clippy::disallowed_methods, reason = "display only")]`. A scan bars it from
    model, run, edit and vm, and `num` gains none.
34. **D13, `crates/observe`.** Phase 2 creates it, holding the measures and the oracle gap: a
    default member with no egui, thread, clock, file or `std::io`. At G2 only the Runner, the
    Extractor, an in-memory Store (writing to a byte sink, a closure) and the view-models move
    into it; the drivers, the spill, file I/O and the tile layout stay in `crates/gui`, and the
    Runner takes a wake callback. The view-model goldens stay GUI tests, run by the GUI's gate
    script.

Decisions 35–37 are the three calls Breakpoint B's pre-look raised, which you left to Claude on
2026-09-26 (docs/spine/EYEBALL.md §6). Decisions 38–40 are the probe's proposals (REPORT §6),
taken by Claude on the same footing. All six are open to veto.

35. **Breakpoint B passes.** The raw record shows the floor era's opposition in its two big
    swings, the escape, and land's exit as a fall. The long-record purpose continues. The
    escape's start is a band, land's exit date depends on the Clark vintage, and whether the two
    turns coincide is left to the fit (D2).
36. **The spine's figures stay local.** They plot Bank of England and Clark numbers, whose terms
    do not allow redistribution, so `data/spine/eyeball.py` builds them into the ignored
    `docs/spine/figs/`. Publishing them waits on the Bank's permission.
37. **The welfare ratio is ours, labelled as ours:** Allen's day wage over the cost of his
    respectable basket, from a verified mirror of his spreadsheet, never presented as his
    published series, which stays out of reach.
38. **The probe closes as GO, with no fallback.** A11's kill condition is not met on Appendix B.
    Phase 2 proper keeps agents as the engine and starts from the probe's roles and harness.
39. **The cash rule is PLAN §3.2's default scale rule,** with July's stability package as the
    named alternative (R6). PLAN §3.2 carries it since S2.6. PLAN Phase 2's opening paragraph
    (July's battery first, "the scale rule is re-tested first") is left as written, as decision
    10 left PLAN's crate list; decision 40 is recorded here only.
40. **A9's re-run of July's solvable family is optional.** The probe's battery against the
    oracle took its place as Phase 2's opener; July's worlds stay available as extra known
    answers.

Decisions 41–53 are session 2's contract decisions, C1–C13 of docs/CERTIFY.md §0, which gives
each with its alternative; the contract says where each was amended. 54–58 were made while
building. All are open to veto; a veto of one that shapes a verdict means new certificates.

41. **C1, the kick is a tape event.** A new core action, `ScalePrice(node, good, by)`, multiplies
    one posted price by a schedule param, once, on a date, in phase 0. A kicked run resumes the
    base run's checkpoint in memory under a kicked tape, and equals that tape's run from genesis.
    Alternative: certify writes a kicked state that no tape made.
42. **C2, `tape_hash` is `fnv1a_64` over the canonical `to_ron`.** It covers every field the
    loader reads and ignores what it ignores. Alternative: hash the file's bytes.
43. **C3, criteria are inputs, verdicts are outputs.** `criteria/<tape>-<date>.ron`, registered
    before the run; `results/<tape>/certificate.ron` and `manifest.ron`, committed, made without
    telemetry. A retune is a new dated file, never an edit.
44. **C4, three verdicts.** FAIL outranks UNSCORED, which outranks PASS. Only PASS exits 0.
45. **C5, windows are shares of a segment** between dated shocks, and a shock closer than
    `min_segment` to the last boundary merges into its segment. The kick fires at every dated
    shock, merged or not (S2.5), and its horizon is a span in years.
46. **C6, measures are oracle-free now;** oracle-relative ones wait for `crates/observe`.
    Certificates and manifests are RON, so no JSON crate joins.
47. **C7, Parquet 60.0.0 with LZ4_RAW only,** pure Rust, behind certify's feature `parquet`.
    Alternative: July's zstd, which builds C.
48. **C8, BalanceWatch is ported; July's six other statistics are not,** since no criterion uses
    them (R10).
49. **C9, a cli resume is verified against the manifest of the same tape** (R16's letter). A
    resume under a dated edit waits for your ruling (CERTIFY §15.1, question 2).
50. **C10, the committed certificates are recomputed by the gate.** Both machines gate the
    verdict and every pass flag; WSL also gates byte equality (all but the build), and Windows
    records it.
51. **C11, the probe delegates its moved measures to certify,** pinned to its report's tables.
    Alternative: freeze the probe with its own copies.
52. **C12, Conservation, Determinism and Runaway are in every criteria file,** and Settles comes
    only with Kick, so a Settles-only file cannot certify a rounding freeze.
53. **C13, a one-sided tick is a BalanceWatch observation at ±1** (July's rule); only S = D = 0
    is skipped.
54. **The kick bounds its peak** (S2.3). Kick also needs every kick's peak gain, its largest gap
    over the realized kick, at most a registered `max_peak` (1e6 for appb). The tail alone passed
    the desk turnover ×16 cell, whose kicks swing out ×2.9e9 and come back to a frozen point.
55. **A certified run counts its price shocks** (S2.5). Criteria may register `price_shocks`, a
    count with a basis; its absence means none, the strictest bar. Dense dated `ScalePrice`
    events are a periodic nudge in all but name (R3).
56. **Each reading carries its comparison** (S2.5), and the seal holds a battery marked pass to
    its readings, so a pass flag, verdict or failure list edited alone is refused. Readback
    checks consistency, not truth: numbers and bars edited together read back, and the truth is a
    rerun.
57. **The registered bars** (`criteria/*-2026-09-26.ron`; CERTIFY §14). gate: to 1790-01-01,
    `min_segment` one year, Determinism at 0.25, 0.5 and 0.75, Runaway 1e3, Trades every year,
    Balance at July's bars (level 1e-9, spread 1e-12, 32 samples, run share 0.5), and no Settles
    or Kick, since the gate world claims no rest. appb: to 2134-08-14, `min_segment` 20 years,
    Determinism at 0.5, Runaway 1e6, Trades and Balance as the gate's, Settles (W from 0.5, F
    from 0.9, dead share 0.01, band 1e-4 in log) and Kick (1e-9, one L of 385 years, the last
    tenth, gain 1e-3, peak 1e6). Both report rationing below 1 − 1e-9 and allow no price shock.
    Each bar has its basis in the file.
58. **The `world_id` re-baseline** (S2.2). A spec's `Site` is hashed with the world, since its
    method decides a number, and the `fixed` flag is not. Every `world_id` moved once and no
    state hash moved. Alternative: keep the methods out of `world_id`, which would let two
    worlds that convert a rate differently share an identity.

Decisions 59–75 are Phase 1's units 1b and 1c (2026-09-27). Most are the open questions of each
spec's §11 (1b's Q1–Q7, 1c's Q1–Q9), which the builds took as their drafts proposed; the rest
are the builds' main choices. Each spec gives the reasoning. All are open to veto; a veto of one
means a change to the oracle and its goldens, not to any engine path.

59. **1b's closure is a fixed basket** (SSRN eq 7; Appendix B's z = (1, 0, h)). Every household
    buys categories in fixed proportions, and the provider's support is one basket, which keeps
    SSRN Lemma B.1's uniqueness (generalised in unit-1b.md §5.3). Space is a category with no
    tasks. SSRN eq 26's CES share is price-block only; a basket that answers prices is 1f's.
    Alternative: a price-responsive basket now, without the uniqueness proof.
60. **One task line for all categories** (1b Q1). Categories are densities of tasks on segments
    of 1a's line, so the unknown stays 1a's threshold x. main.tex's per-category schedules are
    priced by the cell price block, not solved: by a first-order estimate they would lose 1e-11
    to 1e-9 relative on flat schedules and near full automation, which misses the gate.
61. **Cells in the equilibrium are deferred** (1b Q2). The price block prices task cells, closed
    cells included; an equilibrium on the agents' own cells, with the marginal cell split
    between people and machines, waits for your ruling on where it goes: a 1c addendum or a 1b
    addendum, before Phase 2 proper's multi-category instance needs it.
62. **A root in a gap is `Interior` with `margin_active = false`** (1b Q3), not its own regime.
63. **Nesting is gated bit for bit** (1b Q4, 1c Q5): one category is 1a, and one type is 1b and
    1a, to the bit. It fixes the evaluation order of unit-1b.md §5.1 and unit-1c.md §5.1.
    Alternative: 1e-12 relative, which cannot hold near 1a's viability edge.
64. **Viability is checked at the top of the line** (1b Q5), even when no bought category uses
    the top segment, as in 1a. Alternative: at the top of the range in use.
65. **1a's module changed its API, not its behaviour** (1b Q6, 1c Q7): `Regime<E = Eq1a>` is
    generic; the crate-private `classify` is split into the regime tests and the bisection;
    `SolveError` gains `NonFiniteInCategory`, `NonFiniteInType` and `MultipleEquilibria`, and
    `ParamError` gains `Item` and `Invalid`. 1a's uses and its 114 tests are unchanged.
    Alternative: a regime and an error type per unit.
66. **The CES goldens' parameters** (1b Q7) are α = 0.3, the land share of check_pinning's
    A-joint and dynamics' targets, and σ = 0.5 and 2; the paper gives none.
67. **Machine recipes use machine services, labour and land only** (1c §2.3, Q1), as main.tex:688
    and check_dynamics do, with an operating and a build recipe per type. Each type's totals
    then do not depend on x, which gives closed-form switch points and no type returning once
    it has lost. PLAN §3.1's build bundle of goods is not built. Alternative: machines built
    from categories, as a 1c addendum before Phase 2.
68. **One capability shape, a task efficiency per type** (1c Q2): one threshold, and 1a's
    unknown and precision. Alternative: a shape per type, with v the unknown.
69. **A tie is `Interior` with `tie` set** (1c Q3), its split in closed form. Alternative: its
    own regime, which changes 1a's `Regime`.
70. **Multiple equilibria are refused** (1c Q4). With interest, a switch can raise labour demand
    and make three; `MultipleEquilibria` counts the sign changes, each corner one of them
    (P1.6). At ρ = 0 the equilibrium is proved unique. Alternative: report all of them and let
    Phase 2 say which one the agents should reach.
71. **Physical productivity is validated**, ρ(A^op + A^I) < 1 (1c Q6), as 1a validates a < 1.
    It rejects some economies whose price side is viable at a user cost below 1.
72. **Every type is priced** (1c Q9; unit-1c.md §12 item 15). A type whose price recursion
    diverges makes the economy `NotViable` even when no technique would use it, as main.tex:688
    reads; SSRN A.1 asks productivity of the selected recipes only. Alternative: viability,
    prices and totals over each technique's input closure, with the other types unpriced.
73. **Gaussian elimination without pivoting, in index order,** for every Leontief system. On
    these M-matrices it is stable, its positive pivots are the productivity and viability tests
    (the least pivot generalises 1a's D), and its fixed order makes the nesting bitwise.
    Alternatives: partial pivoting, which reorders rows by the data; a Neumann series, slow near
    the edge.
74. **1c's random draws keep the spec's ranges** (1c Q8), with the tallies recorded (unit-1c.md
    §12 item 6); a fourth set is built to switch mid-line and gives the ties.
75. **check_dynamics' sloped x* and flat m are not reproduced in 1c.** They come from a
    Cobb-Douglas land-share household with full participation, which belongs to 1f as a named
    alternative household (R6); the sloped schedule γ = 1 + 4x is not viable at x = 1 in 1c's
    closure. The rest of the two targets is reproduced (m2).

Decisions 76–119 are Phase 1's units 1d, 1e and 1f (2026-09-27). Most are the open questions of
each spec (1d's §11 Q1–Q12, 1e's §11 Q1–Q15, 1f's §12 Q1–Q16), which the builds took as their
drafts proposed; 101, 102 and 119 came from the verifications. Each spec gives the reasoning.
All are open to veto; a veto of one means a change to the oracle and its goldens, not to any
engine path. **Binds Phase 2** marks those that decide what the agents must do or which
instances Phase 2 may use; "Open — your calls" lists them together.

76. **Worker types: one capability shape, an efficiency per type, and reserved tasks** (1d Q1).
    The oracle solves a world whose task cells scale one common human productivity by a type's
    efficiency, or reserve a cell to one type, but not one where the ranking of types changes
    across cells. Alternative: a schedule per type, with a threshold each and an I-dimensional
    solve whose uniqueness is not proved. **Binds Phase 2**: the agents' cells carry
    productivity by type in this form.
77. **The solved corners are reported inside `Interior`**, `margin` saying which (1d Q2), as 62
    and 69 do. Alternative: new regimes `Wall` and `AllHuman`, which change 1a's `Regime`.
78. **No equilibrium with land fully rented is `LaborShort`** (1d Q3), only where the excess
    demand changes side nowhere on the path; the edge of a reserved shortage is solved (P1.9),
    and the saturated knife edge is the junction. 1e resolves `LaborShort` economies on idle
    land. **Binds Phase 2**: its wall-regime instance is a solved wall, not a knife edge and not
    the edge of a reserved shortage, where a type's supply is vertical and its wage is set
    through the pool's clearing, which an agent market may not reach.
79. **Living costs are support baskets on the one basket** (1d Q4): s_i = (e^χ − 1)·ν_i·P_s.
    Alternative: a basket per type, which 1f deferred (114).
80. **Type hours are split by net supply** (1d Q5). Pooled types are perfect substitutes, so
    the model does not say which of them works which pool task. **Binds Phase 2**: compare
    per-type hours only through each type's supply at the oracle's wages, or in total.
81. **Machine recipes use pool labour only** (1d Q6). **Binds Phase 2**: the desks buy labour
    by type (PLAN §3.1), and the sources name trained machine builders (main.tex:584); a
    reserved type in machine recipes couples the walk and the (O, V) system, a 1d addendum.
82. **Bisection on bit patterns** (1d Q7) for a root in [0, 10⁻¹²], the corners' real wage and a
    walled tie's σ, at most 64 steps each; 1a's arithmetic bisection stays on [10⁻¹², 1], so
    the line nests bit for bit. Alternative: keep 1a's `NoInteriorAtZero` below 10⁻¹².
83. **Training is exogenous** (1d Q8): the N_i are inputs, and a walled type's premium is a
    scarcity price, not a cost of training recovered (SSRN p.18's industrial era; p.17's
    pre-industrial craftsman reads the other way).
84. **Viability is checked at the top of the line still** (1d Q9; decision 64), although the
    wall prices machines at any wage.
85. **A tie with a walled type is solved by bisection on σ** (1d Q10), with 1c's closed form
    where no type is walled; with interest its uniqueness within the switch is measured, not
    proved.
86. **The count covers the whole path; `MultipleEquilibria::switches` lists the line's switch
    points only** (1d Q11). Alternative: add the wall's switch wages to the error.
87. **1d's random draws** (1d Q12): the ranges and tallies of unit-1d.md §12 item 12.
88. **Parcels are efficiency units** (1e Q1): rent r·Q_z per acre, the worst idling first by
    convention. Alternative: a Ricardian working cost per acre, with a margin at positive rent.
    **Binds Phase 2**: a tape's parcel quality scales its service, and comparisons use totals,
    not which parcel idles.
89. **The commons is for exit only** (1e Q2). Alternative: production on open land.
90. **One participation rule for both exit forms**, SSRN eq 8 with the exit life's value (1e
    Q3). **Binds Phase 2**: under the default form a pop's participation share is
    F(ln((ν·P_s + w)/(ν·P_s + p_g·s(q)))), its s(q) at the rent its plot actually pays.
91. **Support stays positive** (1e Q4). Alternative: main.tex's pure form with ν = 0, which
    needs a regime for surplus labour at every wage and often has several equilibria.
92. **The exit good is one category** (1e Q5). Alternative: a bundle. **Binds Phase 2**: the
    tapes name it (food).
93. **Plots rented on enclosed land leave production, and the home account is in kind** (1e
    Q6). Alternatives: plots outside the land market (land counted twice), or rent paid in money
    from home goods sold.
94. **Idle land at zero rent with the pool's wage as numeraire; `NoMarket`; no `LaborShort`**
    (1e Q7). At zero rent a good made of land alone is free, and its real wage, wage floor and
    price shares are reported absent. **Binds Phase 2**: an idle-land instance is compared in
    wage units, and rests neither on a free good's absent outputs nor on an equilibrium at the
    walk's ceiling, which is refused (unit-1e.md §12 items 4 and 8).
95. **No reserved tasks with the priced form, for now** (1e Q8). **Binds Phase 2**: the eras'
    trained type at its wall under the default exit form needs a 1e addendum; until then such an
    economy takes the dependence form.
96. **The count scans where supply can fall** (1e Q9): exact where certified, resolved to
    `EXIT_SCAN` elsewhere, and multiple equilibria refused. Alternative: refuse uncertified
    economies, which refuses the race and the commons (Q, K: Appendix B's good carries no
    land). **Binds Phase 2**: the
    default form has several equilibria in a few per cent of economies with little support and
    land-heavy exits, so decision 70 now matters for the historical runs.
97. **The commons clears by a shadow rent that nobody receives** (1e Q10). Alternative:
    congestion that lowers each plot's yield.
98. **The enclosure tie is `Interior` with `enclosure` set** (1e Q11), as 62, 69 and 77.
99. **One land service** (1e Q12): SSRN A.1's vector of non-produced services is not in 1e.
    **Binds Phase 2 and later**: the tape's land classes need an addendum before a region has two
    scarce classes.
100. **1e's random draws** (1e Q13): the ranges and tallies of unit-1e.md §12 item 12.
101. **A free exit good at r = 0 is decided at the wall's end** (1e Q14, the verification):
     q = 1/b̃_g, its limit there. Alternative: §4.6 read literally, every such type on a plot at
     q = 0, which breaks the junction when the type is on its floor on the wall's last piece.
     **Binds Phase 2**: an idle-land instance whose exit good is land alone decides its plots at
     the wall's end; food, which embodies labour, is decided as before.
102. **Free plots on idle land are `ExitLand::Idle`** (1e Q15, the verification). Alternatives:
     `Commons`, or `Enclosed` by convention. **Binds Phase 2**: `Enclosed` means closed by
     price with no suitable land idle; `Idle`, plots free on idle enclosed land at r = 0.
103. **Where each tax is levied** (1f Q1): payroll on gross wages of every hour sold;
     consumption on final purchases at producer value (support and space included,
     intermediates and home output not); rent on market rent in money. Alternative: a tax on
     every purchase, which breaks the ledger. **Binds Phase 2**: the tapes' tax bases.
104. **Transfers in composites at consumer prices** (1f Q2). Alternative: in rent units, which
     R14 forbids for the historical runs.
105. **The budget closes by the owners' levy** (`RentRate`, 1f Q3). Alternative: every rate
     given and the uniform transfer the residual (`Dividend`). **Binds Phases 2 and 6**: a tape
     that gives every rate needs the Dividend closure or a deficit; one that gives relief scales
     in baskets (the poor law) is RentRate's.
106. **A transfer supplements the support by default, or replaces it** (1f Q4). **Binds Phase
     2**: the agents' participation rule reads a transfer as supplementing unless the tape says
     it replaces.
107. **The Dividend closure only where the budget does not depend on who works** (1f Q5).
     Alternative: an inner fixed point in d at each point, with its own uniqueness condition.
108. **Walled types only under RentRate and without in-work benefits** (1f Q6). **Binds Phase
     2**: the eras' trained type at its wall under a wage supplement (Speenhamland) needs an
     addendum.
109. **The path's start is evaluated; `SurplusLabour`** (1f Q7). **Binds Phase 2**: an agent
     economy with an in-work benefit large enough to overfill it has no oracle equilibrium to
     reach.
110. **The price-responsive basket is a CES over the categories with the basket's weights**,
     P = Z·M (1f Q8), the fixed basket kept as the default. Alternatives: a nested CES, or
     Stone-Geary around a subsistence basket, which PLAN §3.2 suggests for the agents. **Binds
     Phase 2**: the oracle and the agents share the consumption rule on any instance compared.
111. **CES only with the dependence form and without reserved hours** (1f Q9).
112. **A category free at the wall's end under CES leaves no idle stretch** (1f Q10).
113. **The land-share household reproduces A-joint in the generator only** (1f Q11), amending
     75: A-joint is `NotViable` in the oracle (D(1) = 0 exactly), which keeps 64, and its viable
     neighbours AJ1 and AJW are the goldens; check_dynamics' ρ > 0 targets also need external
     finance. **Binds Phase 3**: the dynamics thread's steady states need both.
114. **A basket per worker type is deferred** (1f Q12). **Binds Phase 2**: the 1750-like
     instance, with owners buying domestic service, needs it, or takes the common basket in both
     the oracle and the agents.
115. **The rent base is market rent in money; κ keeps 1e's definition** (1f Q13), so eq 16's τ_R
     is 1/κ only where no plot is rented.
116. **`Eq1f::base` keeps 1e's accounts without a government, labelled** (1f Q14).
     Alternative: recompute them with the government, which breaks `base`'s identity with 1e.
117. **1f's random draws** (1f Q15): the ranges and tallies of unit-1f.md §14 item 11.
118. **No government purchases, deficits or taxes on interest** (1f Q16). **Binds Phases 6–8**:
     wars and debt.
119. **A CES basket's numerics** (the verification): the power mean's direct sum where
     1 + S < 1/2, the corners bisected in v, a point beyond every double read +∞. No
     fixed-basket result changes.

## Open — your calls

- **The GUI's decisions**, 22–34 (D1–D13): open to veto before G0. D10's items are built.
- **Hosted CI** (A5): the push of 2026-09-26 started it; it runs on every push unless you turn it
  off.
- **Decisions 35–40** (Breakpoint B's three calls and the probe's proposals), taken by Claude on
  your word and open to veto.
- **Decisions 41–58** (session 2's), open to veto.
- **CERTIFY §15.1's questions:** (1) is a registered `price_shocks` count above 0 ever
  acceptable in a certified tape, or does `ScalePrice` belong to the kick alone; (2) should the
  cli resume under a dated edit behind an explicit `--edited` flag that records the parent and
  marks the run; (3) BalanceWatch's bars are absolute on the imbalance, a number in [−1, 1],
  read as allowed by A12; (4) C11 edited probe code that REPORT cites at `55c9e88`, guarded by
  its pins; (6) the kick's horizon is one L, so an instability slower than L passes.
- **Decisions 59–119** (Phase 1), open to veto. Those that bind Phase 2 proper: from 1b and 1c,
  cells in the equilibrium (61) and machines built from categories (67), each a 1c addendum if
  ruled in, and whether multiple equilibria are refused or all reported (70), which decides what
  Phase 2 compares the agents against and which 96 makes matter for the default exit form; from
  1d, one shape with an efficiency per type (76), a solved wall as the wall instance (78), type
  hours compared through supply or in total (80), machine recipes on pool labour (81); from 1e,
  parcels as efficiency units (88), the participation rule under the default form (90), one exit
  good the tape names (92), idle land in wage units (94), no reserved tasks with the priced form
  (95), multiple equilibria where support is low (96), one land service (99), a free exit good
  at r = 0 (101), `Idle` plots (102); from 1f, the tax bases (103), the closure (105), supplement
  or replace (106), walled types and in-work benefits (108), `SurplusLabour` (109), one
  consumption rule for the oracle and the agents (110), one basket for every type (114). 113
  binds Phase 3, and 118 Phases 6–8.
- **Landing `phase1`.** `reboot` took P1.2–P1.7 by a fast-forward at `503897e` and has moved on
  to `2398b6a` (the `g0` merge, the many-markets probe, the demo world), none of which touched
  `crates/oracle`. `phase1` (P1.8–P1.14) now needs a merge. Its code merges without conflict;
  STATE.md and the root README are reconciled by hand, and, as `reboot`'s copy of this file
  says, Phase 1's decisions and open items are renumbered after 134 and O27: 76–119 become
  135–178 and O20–O22 become O28–O30, in STATE.md, the oracle's README and unit-1d.md,
  unit-1e.md and unit-1f.md, which cite them as proposed decisions. `probe::markets` and
  `worldgen` call the oracle (1a's and 1c's API, which 1d–1f extended without changing), so the
  merged tree's gate runs on both machines before `reboot` takes it. Pushing is your call.
- **The Phase 2 session budget** that A11's kill condition needs (PLAN Phase 2), now for Phase 2
  proper's other instances.
- **The decisions above**, especially 10 (the engine crate, not in PLAN's crate list), 11, 15
  and 17, and the GUI's 22–34.

## Open — work

- **O1. The GUI.** Designed ([docs/GUI.md](docs/GUI.md); A14), with egui in `crates/gui`. G0,
  the shell, was built on branch `g0` and is recorded there and on `reboot`, not in this copy
  of the file: `reboot`'s history has G0.3, "G0 closed", at `428bdcd`, merged at `708167f`. G1,
  the oracle lab, starts after G0 and Phase 1's gate, and both are now met ("Next steps").
  `crates/engine` was built for it: a frontend
  depends on the engine alone, steps a `Sim` on a worker thread and reads each `TickReport` over
  a channel. Session 2 gave it what §7.2 asked: `FiredEvent.source`, the registry's sites with
  their methods, a `world_id` that a new source event keeps, and from certify `RunKey`,
  `tape_hash` and a manifest that needs no Parquet or I/O (GUI.md, updated at S2.6).
- **O2. Phase 0 session 2: closed at S2.6** (2026-09-26; "Where things stand" above;
  docs/CERTIFY.md). The certification stack moved from `july-v2-phase-3` into `crates/certify`
  with N4, N10, N12 and N15 fixed, every threshold in dated criteria, A12's runaway detector,
  the probe's three criteria, and D10's items 1, 2 and 4. Both tapes certify PASS.
- **O3. The oracle: Phase 1 closed at P1.14.** Unit 1a landed at P1.1, units 1b and 1c at
  P1.2–P1.7, units 1d–1f at P1.8–P1.13 ("Where things stand" above). Unit 1a was built by
  another run and verified there (114 tests), and joined through the members glob. The oracle
  depends on `core` alone, for `num`, and nothing on the engine path depends on it (R13). The
  workspace's `clippy.toml` denies the platform maths, so x^k, ln(1 + z) and its fused
  multiply-add go through `core::num` (libm), which gained `fma`; 1a's outputs are
  byte-identical on WSL and Windows (5000 random economies, every regime), no golden moved, and
  G8's exact tie still ties. Units 1b–1f needed no new `num` function. What Phase 1 leaves is
  O18–O22 and the decisions that bind Phase 2.
- **O4. `test_01` is retired with the v1 agents** (A3), not ported. It failed at every commit
  where its tests compile (from `03eb06a`; the April commits do not compile theirs) and on all
  three July branches: `building_inventory_cycles_correctly`, "farm should produce wheat on tick
  0, got 0" (`tests/test_01_single_region.rs:172`, `:153` on v1). Nothing to do; logged here.

Round 3 of the adversarial review (after P0.7) left nine major issues, O5 to O13, and no
blocker. P0.9 fixed all nine; ENGINE.md's P0.9 amendment has each, and each has a test that fails
when its fix is reverted, checked by mutation (the review's own mutants among them):

- **O5.** A burn of several lots declares their float sum and, as `Rounding`, what that sum
  rounded away; `Inventory::put` returns each merge's rounding, declared one by one. Each delta's
  declarations are now exact (`multi_lot_rounding_is_declared_exactly`,
  `several_merges_report_each_rounding`; T7 and four more mutants killed).
- **O6.** The tolerance's flow term is pinned for a tick and a run
  (`flow_tolerance_is_pinned_for_a_tick_and_a_run`; U1, U2, U3 killed).
- **O7, O8.** Checkpoint format 3: the digest covers `world_id`, `prefix_id`, the state and the
  run's ledger, which the checkpoint now carries and a resume continues
  (`checkpoint_digest_covers_identity_and_run` in core, `resume_refuses_an_edited_identity` in
  engine and cli, `resumed_run_stops_where_the_uninterrupted_run_does`,
  `gate_resume_from_checkpoints` comparing whole reports).
- **O9.** Dated events in one tick fire by (date, key) (`same_tick_events_fire_in_date_order`,
  `unsorted_events_fire_in_order`).
- **O10.** The scripted actor's conversions are pinned at 12, 52 and 365 ticks a year
  (`per_tick_conversions_follow_the_clock`; eight mutants killed).
- **O11.** The frontend guard reads every public function of engine, markets and agents, every
  impl of `Sim`, `Checkpoint` and `SimState` (trait impls included) and their fields; core's
  re-exports are an allow-list; four more compile-fail doc tests. The review's five writers and
  two more are killed.
- **O12.** `failed_final_checkpoint_save_stops_the_run` (cli).
- **O13.** The literal scan flags integers made float, `from_bits` of a literal and
  `parse::<f64>`; `tiny_imbalances_move_the_price` pins the price rule at tiny imbalances.

- **O14. Adjustment paths, not only rest points.** The probe shows the agents reach the
  oracle's point, but by violent paths: a 12% fall in equilibrium output costs 85% on the way,
  with ticks of no consumption (REPORT §5). The plan scores paths (Engels' pause, the crises of
  Phase 8), so path fidelity is Phase 2 proper's second untested risk, beside many markets.
  Carried. Session 2 built its first instrument: every scored certificate reports, per segment,
  the troughs of cleared volume, dead ticks, fills and rationing, ticks with no consumption,
  spoilage and the transfer shortfall (CERTIFY §6). They are reported, not scored. Scoring a
  path needs the oracle's reference, so it waits for Phase 2 proper and `crates/observe`.
- **O15. The spine scripts' default cache: done at S2.6.** The scripts read `$SPINE_ROOT`, else
  `data/spine/.cache/` beside them, which `.gitignore` keeps out; the finer overrides stand.
  With `SPINE_ROOT=D:/rustyecon-spine` every path equals the old default (checked by evaluating
  each script's path constants, not by running them); nothing was refetched
  (docs/spine/DATA_NOTES.md, "Where things are").
- **O16. Readback accepts a forged comparison** (the re-check, on `daa62af`; CERTIFY, amended at
  S2.6). A FAIL certificate whose Kick readings have their comparisons turned into `Ref`, with
  its pass flag, verdict and failures edited and no number changed, reads back PASS. `Ref` is
  Balance's alone, so the seal could refuse it elsewhere, or fix each reading's comparison by
  its name. It is within the stated limit (numbers and bars edited together read back; the truth
  is a rerun), and the committed certificates are recomputed by the gate, so nothing committed
  is affected. Take it at the next change to certify.
- **O17. What waits for `crates/observe`** (D13; CERTIFY §11). The oracle-relative measures:
  the target and the gaps, shock distance and κ, the envelope, VACUOUS, the hold check, the
  bands per relative price, the oracle-relative dead floor (`LIVE_FLOOR`) and troughs, and the
  classifier. Until the dead floor arrives, a run frozen at tiny positive volumes passes Trades,
  and certifies under criteria that list no Kick, as the gate's do; under appb-style criteria the
  kick catches the probe's known case (CERTIFY, amended at S2.5, item 9).
- **O18. Precision the oracle states but does not reach** (recorded, not scheduled). Near an
  interior edge of 1b's task line, outputs made of the sliver between x* and the edge are good
  to about 2^-53·x*/d relative (7.5e-8 at d = 1e-9; unit-1b.md §5.4); only more working
  precision would help. At a tie in 1c, the split and each type's quantities carry γ_i's error
  amplified by the excess demand's slope over the jump (1.7e-10 measured), and a tie near x = 1
  sets 1 − x* = 1.0 − x_i without interpolation (3.2e-7 relative at a tie 1e-9 below 1;
  unit-1c.md §5.6). Phase 2's comparisons against these outputs need bands that allow for them.
- **O19. Mutants that survive the gate** (recorded). 1b's reassociation of
  p_j = v·H_j + (p_m·M_j + b_j), which changes only the rounding; 1c's crossing counted at
  exactly γ_c, and γ_c not advanced to each switch, equivalent in exact arithmetic (they differ
  only where rounding separates a crossing from its tie or puts it below the previous one); and
  two mutants of P1.6's count, equivalent on every economy the gate can build (unit-1c.md §12
  items 12 and 17). A change to those lines should look at them again.
- **O20. What the re-checks of 1d–1f left** (recorded, not fixed: the bounded verification ends
  at the re-check). In each case the re-check found the oracle's results right; what is missing
  is an economy in the gate that tells a mutant apart. A test for each is cheap, and the next
  change to the oracle should add them, before Phase 2 proper compares against these lines.
  - 1d (on `2856982`; the logs in `D:/rustyecon-p1/verify-1d-mutation-r2/`). Four mutants the
    re-check's own probe tests kill: another type's closure wage at a corner taken at γ*, lemma
    B.1's flag at the base basket price, the wall's last piece started from f(1) instead of the
    last wall switch's value, and f_∞ = 0 read as negative after a positive start. Three survive
    without a probe: lemma B.1's flag without its shortage check, the edge's κ bracket started
    at 0 rather than ζ, and a tie edge's share one double up. Seven survivors of the first pass
    that the fix round did not take: the walk's and the corners' orders at a tie, the corner
    supply at equality, a tie's σ where f(1) ≥ 0, the all-human corner's ω at the base price, no
    worker types accepted, the labour net removed and the pool share's zero guard.
  - 1e (on `a680dd4`). The wall's-end frame's price b̃_g set to 1, or inverted: every free exit
    good the gate builds has b̃_g = 1 (Appendix B's space), and the re-check's probe with space's
    b at 0.5, 2 and 3 kills both.
  - 1f (on `44d7909`). A CES economy's required hours taken from final content rather than
    gross outputs, the same without intermediate inputs, which no CES instance has; and the
    exit-free scan's sides not passed to the count, which only the unit test of
    `count_changes` sees.
- **O21. A CES weight below the scale floor is refused** (the 1f re-check). A weight must be 0
  or within [1e-30, 1e30]; eq 26's weights α_j^σ fall below that near `SIGMA_CEIL` = 64
  (0.3^64 = 3.4e-34), and six such economies, each with an equilibrium, were refused as
  `Invalid`. Either lower the floor for weights, with the CES evaluation checked there, or state
  the σ each weight allows. No Phase 2 instance needs σ near 64.
- **O22. Precision and refusals of 1d–1f that Phase 2's bands must allow for** (recorded, as
  O18). A tie with a walled type, or at a wall switch with nearly parallel delivered costs,
  carries v's error into σ amplified (8.8e-12 measured, dlog σ/dlog v = −201; unit-1d.md §12
  item 18); a wall near its real-wage ceiling has a wage ill-conditioned in the data (§5.5);
  with walled types on idle land P_s is the walk's fixed point, and one ulp of T_m moved it by
  4.5e-10 on one draw, where another, ill-conditioned, was refused by the labour net at 5.0e-9
  (unit-1e.md §12 item 19); an idle-land equilibrium at the walk's ceiling is refused, as 1d
  refuses one on the line (item 8); an uncertified economy's scan finds two equilibria only
  when they are more than one cell apart (§5.5); a consumption tax equals a wage tax in real
  numbers, and in f64 the two allocations agree to the supply's sensitivity (unit-1f.md §5.5).

## Corrections logged (A3; ADDENDUM §1.4)

REVIEW.md is kept as written; these of its claims do not hold.

- "No code changed since July; all seven Phase 0 defects open": true of `main` only; `cf7e78f`
  fixed all seven on 2026-07-19.
- "No code was written against the July design": false; July Phases 0–3 and most of 4 were built.
- "The desk kernel's stability has never been tested": partly false; it was built and lost every
  pre-registered A/B.
- "Windows has no toolchain or venv": false; Windows has 1.97.1, and July was built there.
- "laborformal's venv is present and working": partly; it has no SciPy.
- "Defect 8 is dormant": false on `main`; the pop cash cap binds in all 24 lr scenarios.
- "Certification, Parquet and the tracer are design": built on `v2-phase-3`, tests passing; the
  tracer certifies FAIL today, and its July PASS was retracted.
- "The July tick-time rule": three v1 doc sections; no branch had a tick-length parameter.
- "s(q) is an object of the SSRN paper": false for the posted SSRN version, which prices exit as
  dependence; s(q) is in `main.tex` (now ruled the default, the SSRN form the alternative).
- Minor: the tag `pre-cleanup-2026-09-04` that laborformal cites exists nowhere; the fix belongs
  in laborformal.

## Next steps, in order

Phase 1 is closed (O3). What follows is in order. `reboot`'s copy of this file carries the steps
of the work done there since `503897e` (among them your looks at G0's window and at the demo's
map), which this copy does not record; the two lists are merged when `phase1` lands.

1. **Land `phase1` into `reboot`**, on your word ("Open — your calls"): a merge, with STATE.md
   and the root README reconciled by hand, Phase 1's decisions and open items renumbered after
   134 and O27, and the merged tree's gate green on both machines before `reboot` takes it.
2. **G1, the oracle lab** (docs/GUI.md §9; PLAN Phase 1's GUI line), after G0 and Phase 1's
   gate, both now met. G0 was built and closed on branch `g0` (G0.3), and `reboot` records what
   G0 moved to G1. The lab shows an instance's regime and outputs beside their goldens, any
   field over x, one-parameter sweeps and the price-step explainer; with 1d–1f it has the
   path's margins (the line, the wall, the all-human corner, idle land), worker types, parcels
   and the two exit forms, and a government to show. It builds on `reboot` once `phase1` has
   landed, since it reads 1d–1f.
3. **Phase 2 proper** (PLAN Phase 2), after your rulings on the decisions that bind it: 61, 67
   and 70 from 1b and 1c, and 76, 78, 80, 81, 88, 90, 92, 94–96, 99, 101–103, 105, 106, 108–110
   and 114 from 1d–1f. Its four stationary instances can now all come from the oracle: the
   Appendix B instance (1a); a wall-regime instance, a solved wall (1d; decision 78); an
   open-commons instance under the default exit form, with the commons' shadow rent and idle
   land at zero rent (1e; decisions 90, 94, 101 and 102); and a 1750-like instance (1e and 1f,
   with the common basket unless a basket per type is ruled in, decision 114). The agents'
   participation rule under the default form is decision 90's, and under a government 106's.
   It starts from the probe's roles and harness and from what `reboot` records of the
   many-markets probe. O20's tests come first, with the next change to the oracle, and O22's
   precision sets the bands of any comparison against the new margins.

## File map

```
STATE.md                 you are here; start here next session
README.md                what rustyecon is, the crates, how to build and test
docs/PLAN.md             the plan, amended by the addendum's rulings (2026-09-25) and decision 39
docs/ENGINE.md           the Phase 0 engine contract, with each step's amendments (P0.3–S2.6)
docs/CERTIFY.md          session 2's contract: criteria, batteries, kick, seal, manifest, cli,
                         telemetry, with each step's amendments (S2.2–S2.6)
docs/TAPE.md             the tape's schema guide
docs/GUI.md              the GUI's design (A14): stack, architecture, panels, editor, map, roadmap
docs/reboot/             REVIEW.md and ADDENDUM.md, kept as written (links fixed) but for A14 and
                         rulings 5–8 (P0.11); GUI-review-ledger.md, the GUI design's two reviews
docs/timeline/eras.md    era research for worldgen
crates/core              ids, clock, inventory, deltas, apply, ledgers, hash, checkpoints, tape
crates/markets           admission, clearing, settlement, prices
crates/agents            the behaviour seam, the scripted actor, the Appendix B roles (P2.0.1)
crates/probe             the Phase 2 probe's harness and tape generator (P2.0.1); reads certify's
                         measures (S2.5)
docs/probe/RULES.md      the probe's rules, dials and lineage, as built
docs/probe/REPORT.md     the probe's report: verdict, battery, dial map, reviews, what it means
docs/probe/figs/         the report's plots; docs/probe/results/ its three summary tables (CSV)
crates/engine            Sim, the tick, reports, resume, the replay audit, the registry listing
crates/cli               the rustyecon binary: run, resume, replay, registry, certify
crates/oracle            the equilibrium solver, units 1a (P1.1) and 1b–1f (P1.2–P1.13),
                         Phase 1 closed at P1.14; its README, docs/unit-1{a,…,f}.md and
                         goldens/generate{,_1b,…,_1f}.py; tests/gate/p1_gate.rs, the gate
crates/certify           criteria, batteries, the kick, the sealed certificate, the manifest;
                         Parquet telemetry behind the feature `parquet` (S2.3–S2.5)
crates/certify/testdata  appb variants from `appb-tape --perturb`: bcycle, freeze, july, buffer16
crates/worldgen          empty until Phase 4
tapes/gate.ron           the gate world
tapes/appb.ron           the probe's Appendix B world, generated from the oracle
criteria/                each tape's dated criteria, registered before its first certified run
results/                 committed verdicts: results/<tape>/certificate.ron and manifest.ron
data/spine/              the spine's fetch, extract and eyeball scripts, manifests, CC0 files;
                         their cache is $SPINE_ROOT or the ignored data/spine/.cache/
docs/spine/              DATA_NOTES.md and EYEBALL.md, Breakpoint B's pre-look (S5.0)
scripts/gate.sh          the gate as one script
.github/workflows/ci.yml hosted CI, on every push
```

## Repro notes

- The gate in WSL, from a Windows shell:
  `wsl -d ubuntu --exec bash -lc '<repo>/scripts/gate.sh'`. Always `--exec`: with `--` the exit
  code is lost. The script puts the build in `$HOME/scratch/target-rustyecon-gate` unless
  `CARGO_TARGET_DIR` says otherwise, and refuses a target directory inside the tree. With no
  network and a warm cache, set `CARGO_NET_OFFLINE=true`.
- On Windows, the same script under Git Bash with `CARGO_TARGET_DIR` outside the tree (it skips
  the wasm32 check, since the target is not installed there), then `rustyecon run tapes/gate.ron
  --until 2080 --hashes <file>` and the same for `tapes/appb.ron --until 20000`, and a byte
  comparison of each file's body (the `#` header names the build and target) with WSL's.
- Session 2 ran from a worktree, `D:/rustyecon-wt/s2` (`/mnt/d/rustyecon-wt/s2` in WSL), with
  targets outside it (`/root/scratch/target-s2-*`, `D:/rustyecon-targets/s2-*`) and its logs in
  `D:/rustyecon-s2/`. The build stamp reads git through the worktree's `.git` file, mapping its
  Windows path for WSL.
- Phase 1's units 1b–1f ran from a worktree, `D:/rustyecon-wt/p1` (`/mnt/d/rustyecon-wt/p1`
  in WSL), on branch `phase1`, with targets `/root/scratch/target-p1-<label>` and
  `D:/rustyecon-targets/p1-<label>`, and scratch, prototypes, gate and mutation logs in
  `D:/rustyecon-p1/<label>/` (each unit's `spec-`, `build-`, `verify-…-r1`, `fix-` and the
  re-check's `verify-…-r2`; the close's in `close/` and `close/phase1/`). The prototypes are
  scratch; the committed generators reproduce their numbers.
- The oracle's goldens: from `crates/oracle`, run `goldens/generate.py`, `generate_1b.py`, …,
  `generate_1f.py` with `--check` under laborformal's venv
  (`C:/Users/wilso/Documents/GitHub/laborformal/venv/Scripts/python.exe`,
  `PYTHONIOENCODING=utf-8`); together they take about two minutes (`generate_1e.py` 49 s,
  `generate_1f.py` 28 s). A change to `generate.py` means rerunning all six, since each later
  one records the earlier ones' digests.
- To certify a tape: `rustyecon certify <tape> --criteria criteria/<tape>-<date>.ron --out
  <dir>`. Committed results are made in WSL by a clean build, without `--telemetry` (C3), and a
  change that moves a verdict's path regenerates them in their own commit.
- The spine scripts: set `SPINE_ROOT=D:/rustyecon-spine` on this machine to use the cache the
  first pass fetched (DATA_NOTES).
- A log captured by redirecting `wsl.exe`'s output to a Windows file can interleave and lose
  lines; redirect inside the WSL command instead.
- The July engine is read with `git show july-v2-phase-3:<path>`; never check the tag out
  into this tree.
- laborformal is pinned at `31b3482` and read from that commit (`git show 31b3482:<path>` or
  `git archive`), never from its stale checkout (A6). Until one interpreter has both SciPy and
  SymPy, `paths/checks/check_macro.py` runs under WSL's python3 and the SymPy checks under the
  venv with `PYTHONIOENCODING=utf-8`.
