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
**State as of:** 2026-09-30, on branch `phase2-proper` from `reboot` at `f7d1eae`, not pushed or
merged. **Phase 2 proper's first scored wave is run: IW1, C1 and C2 are GO, as registered**
(P2.3.10–P2.3.15; "Where things stand";
[docs/probe/results/wall/README.md](docs/probe/results/wall/README.md),
[docs/probe/results/commons/README.md](docs/probe/results/commons/README.md)): 3,721 jobs on
WSL, every class and every tick to tolerance the mirror's in 3,633 compared runs, the subsistence
trap exactly where the mirror put it, no refutation criterion met on one reading taken after the
result (O108, open to veto). 124 of 59,788 lines fail, none a class, a tick or a verdict: the
runaway bound's reference (O107), two end-of-run lines registered against the wrong value (O108).
Three faults of the machinery were found and fixed in their own commits, each disclosed: the wall's
basket count before the wave (P2.3.10), dated shocks of the commons (P2.3.12) and the runner's
quoting (P2.3.13) during it. O22's families on I1–I3 are still to run (O109). The reviews are next.
**The open-commons instances C1 and C2 are registered** (P2.3.5, docs only; "Where things
stand"; [docs/probe/results/commons/registration.md](docs/probe/results/commons/registration.md)):
the frame `frame-commons` (I1 with one priced worker type in food and a commons the workers hold,
full at C1 and with room at C2; the commons no market, its plots given out by the participation
rule) and its mirror's predictions, GO for both, fixed before any engine code for them, with 368's
Tier 3S added from the same mirror; decisions 398–399, O96 amended, O100–O106. **They are
built** (P2.3.6; [docs/probe/COMMONS-RULES.md](docs/probe/COMMONS-RULES.md)): the workers'
optional `exit`, C1, C2 and C1N on unit 1e, `tapes/markets-c1.ron` and `markets-c2.ron`, the
harness at the commons and 16 tests, with every committed tape's text, ids and streams unchanged.
**E0–E2 pass** (P2.3.7–P2.3.9; [docs/probe/results/commons/e0.md](docs/probe/results/commons/e0.md)):
the engine is the mirror's map to 2.6e-14 at C1 and C2; the negative control's trap run parts from
tick 41 while staying the mirror's one-step map, under amendments A1 and A2 (A2 written after the
official run, disclosed). The scorer and the scored wave are next.
**The wall instance IW1 is built, and E0–E2 pass** (P2.3.2–P2.3.4; "Where things
stand"; [docs/probe/WALL-RULES.md](docs/probe/WALL-RULES.md),
[docs/probe/results/wall/e0.md](docs/probe/results/wall/e0.md)): the roles' three optional
fields, the instance on unit 1d, `tapes/markets-iw1.ron`, the harness at the wall and 14 tests,
with every committed tape's text, ids and streams unchanged. The engine is the mirror's map to
1.31e-14, but for a tick-1 rounding residue that amendment A1, committed before the run, allows.
The scorer and the scored wave are next.
**The wall instance IW1 is registered** (P2.3.1, docs only; "Where things stand";
[docs/probe/results/wall/registration.md](docs/probe/results/wall/registration.md)): the frame
`frame-wall` (1d's B economy with E7's three types, a solved wall whose pool wage is 0.94 in log
above the top task's replacement value) and its mirror's predictions, GO, fixed before any engine
code for it; decisions 394–397 and O97–O99. **Phase 2 proper opens with its rulings** (P2.3.0,
docs only; "Where things stand"): the
decisions next step 7 waited on are taken by Claude on your word as decisions 360–393, each open
to veto, with the probe reports' recommendations (MARKETS §6 keeps 67 and 70); O95 and O96 are
new. It opens on two loop-free instances at C2m, 52 ticks a year and ρ 0: a solved wall under
1d's dependence form, and an open commons under s(q) with idle land at zero rent (next step 7).
Before it, on branch `phase2-plants` from `reboot` at `8b07c8a`, now in `reboot` at `f7d1eae`:
**P2.2b, the loops, is closed** (P2.2b.4; "Where things stand";
[docs/probe/LOOPS.md](docs/probe/LOOPS.md)): GO for rule B's horse loop at chain8 at C2g and 52
ticks a year, with CAPACITY's plant on every loop desk and the maker's reservation together, as
its two reviews narrow it. Both reviews found that the verdict holds. Their three majors and five
minors are answered with tables and disclosures, and no code changed. A loop enters Phase 2 only
through its own registration (decision 308, narrowing 284). Decisions 307–311 and O79–O80; O49,
O65 and O68 are closed. Next on this line: storable running goods (O51), a funded steam county
(O62), then the goods chain's 1750-like instance with its own loop registration (next step 6).
**P2.2b's scored runs pass** (P2.2b.3; "Where things stand";
[docs/probe/results/loops/README.md](docs/probe/results/loops/README.md)): E3–E11 ran as
registered, 3,549 runs and 24 kick sets on WSL, and every one of the 1,276 scored lines holds. LB1,
LB2 and LB3 are GO, as are the flow controls LW1–LW3 and the families LF1, LF2, LF5, LF7, LF8 and
LC1; every negative control keeps its verdict; no refutation criterion was hit. At the GO
instances the engine is the mirror to the tick in every run's ticks to tolerance. Decisions
305–306, O77–O78; O72 closed. The reviews and the report (`docs/probe/LOOPS.md`) are next.
**P2.2b's harness is built and E0–E2 pass** (P2.2b.2). The rule-B
harness is `probe::horses::loops`, which `horses` runs for a loop id. E0's trace diff against the
registered mirror with the carry passes: nothing parts at tick 1, and the only partings are the
horse market's cancellation. So the registration stands, with no amendment. E1's nesting holds,
and mode A passes at L. Decisions 302–304, O75–O76; O73 closed. The scored runs (E3–E11) are
next. **P2.2b's plant layer is built** (P2.2b.1): core's untraded good,
CAPACITY's plant on the type desk, the maker and the capacity desk, the loop instances and their
six tapes, with every committed tape's text, hashes and streams unchanged; decisions 297–301,
O73–O74. **P2.2b's frame is
written** (P2.2b.0, docs only): the build's spec,
[docs/probe/LOOPS-RULES.md](docs/probe/LOOPS-RULES.md), and one dated amendment before any code,
[docs/probe/loops/LOOP-SPEC-A2.md](docs/probe/loops/LOOP-SPEC-A2.md) (LN5 waits for O51, as LF4
does); decisions 286–296, O69–O72. Before it, on `phase2-loops`
from `reboot` at `401f7b1`: **the loop stage's groundwork (L0) is done** (L0.1–L0.9, "Where
things stand"; decisions 240–283; O51–O68). L0.1 closes O48, M6 narrowed to the durable good and
M5 checked at load.
**L0.3–L0.6 close O47, the idle machine market**: the maker's reservation at ψ 0.25, registered
before it was built, converges every heads × 10 run and P8, changes no class in P2.2a's 1,980
battery runs, and meets its registered predictions with five explained misses and no refutation;
its engine review found no refutation, and L0.7 fixes its five minor findings
([docs/probe/IDLE.md](docs/probe/IDLE.md)). **L0.8 registers the loop step for P2.2b**: a funded
rule-B county, chain8 (O41 closed), and the mirror's loop step with CAPACITY's plant on every loop
desk, registered again with the engine's genesis carry after the mirror review
([docs/probe/results/loops/registration.md](docs/probe/results/loops/registration.md)). P2.2b's
frame is written (P2.2b.0) and its build is next (next step 6).
**The stocks probe (P2.2a) is closed** (P2.2.1–P2.2.4, on branch `phase2-goods` from `reboot` at `92ba68e`;
"Where things stand"): the horse as a durable good, bred by a maker and hired out by a wet
capacity desk, finds unit 1g's
equilibrium of the goods chain's first stage, v2a.1: GO at 52 ticks a year, narrowed by its two
reviews ([docs/probe/HORSES.md](docs/probe/HORSES.md)). Its decisions are 220–239 and its open
items O41–O50. **`oracle-goods` and `g1` are merged** into `reboot` at `43ad8c5`, on branch
`merge-og-g1` from `reboot` at `16eb728`, and `reboot` took the merge by a fast-forward:
`oracle-goods` came in by a fast-forward and `g1` by the merge. The two tracks both started at
`16eb728` and numbered apart, so nothing is renumbered: track 1g's decisions are 179–190 (its
range 179–199) and its open items O31–O35, and G1's decisions are 200–219 and its open items
O36–O40. No code conflicted; README.md and this file were joined by hand ("Where things stand").
**Unit 1g, machines as goods, is built** (track 1g, branch `oracle-goods`, P1g.1–P1g.7): the
oracle's addendum for machines built from goods. It is D-G10 (productivity and the chain to land
checked on the per-period recipes, which accepts every economy 1c accepted with every result bit
for bit), the mapping of a chain of goods to unit 1c, plants as machine types (the capacity
damper's long run), 210 goldens at 70 digits, and tests for some of O28's mutants. Its bounded
verification found no wrong result and five surviving mutants, each killed by a test at P1g.6;
the re-check of the fixed items is still to come (O35).
**G1, the oracle lab, is built** (G1.1–G1.10; "Where things stand"): the lab solves any of the
oracle's units 1a–1f from 16 golden presets and shows the regime and every output beside its
golden, a field over x with its root and bracket, and one-knob sweeps; the market inspector
explains each price step with markets' own `next_price` and draws its log waterfall; plots take
log axes, the outliner a watchlist, runs event and date breakpoints; the toolbar saves PNG
snapshots marked never citable. The engine now re-exports `num` and the tape's raw schema, so the
GUI's edge to core is gone (next step 4), and O26's map items and two of O20's are done. Every
gate item of G1 is met but the window's p90 checked by hand, which is yours. **G1's bounded
verification ran, and its findings are fixed at G1.11** ("Where things stand"): six major and
four minor, each a test that passed with the code it guards broken, a way past a scan, or a doc;
no number the GUI shows was wrong. A re-check of those fixes ran after G1.11 and left five
majors open, none a wrong number (O37). The lab does not yet show unit 1g (O38).
**`phase1` is merged** into `reboot` at `16eb728`, on branch `merge-p1` from `reboot` at
`2398b6a`, and the local `reboot` took the merge by a fast-forward. Two lines of work that both
started at `503897e` meet there. **Phase 1 is closed** at P1.14 ("Where things stand"), built on
branch `phase1`: units 1d (worker types and the wall), 1e (parcels, the idle margin and the priced
exit) and 1f (households and government) are built and verified (P1.8–P1.13), and PLAN Phase
1's gate is met item by item. `phase1`'s copy of this file numbered its new decisions from 76
and its open items from O20, as this line's copy had, so Phase 1's are renumbered after this
line's 134 and O27: 76–119 become 135–178, and O20–O22 become O28–O30. With G0 closed and
Phase 1's gate met, G1, the oracle lab, is unblocked.
**`demo-world` is merged** into `reboot` at `2398b6a`, on branch `merge-demo` from `b2a55e3`,
and the local `reboot` took the merge by a fast-forward. Two lines of work that both started at
`708167f` meet there. **The many-markets probe (P2.1) is closed** (P2.1.1–P2.1.4; "Where things
stand"), built on branch `phase2-markets` and fast-forwarded into the local `reboot` at
`b2a55e3`: many markets GO for loop-free economies at 52 ticks a year, a loop of produced inputs
NO-GO, no fallback ([docs/probe/MARKETS.md](docs/probe/MARKETS.md)). **The demo world and its
map are closed** at D.5 ("Where things stand"), built on branch `demo-world`, at your request of
2026-09-27 for a map of the United Kingdom with Victoria-style lenses over a world of regions,
goods and history: an illustrative world of 93 historic counties, 1750–1901, each running the
probe's four roles, with the GUI's map and 25 lenses brought forward from G4 and G2. Its window
is yours to look at, and the command is in "Where things stand". Both copies of this file
numbered their new decisions from 118 and their open items from O21, so the probe's keep
118–123 and O21–O24, and the demo's are 124–134 and O25–O27.
**`g0` is merged** into `reboot` at `708167f`, on branch `merge-g0` from `503897e`, and the local
`reboot` took the merge by a fast-forward. Two lines of work that both started at `397d7cd`
(S2.6) meet there. **Phase 1's units 1b and 1c are closed** (P1.2–P1.7), built on
branch `phase1` and fast-forwarded into `reboot` at `503897e`. **G0, the GUI's shell, is
closed** at G0.3, built on branch `g0`: G0.1, the viewer, and G0.2, the editor, are built and
verified, and what each verification found is fixed. Every item of G0's gate is met but the
window checked by hand, which is yours; the command is in "Where things stand". Both copies of
this file numbered their new decisions from 59, so Phase 1's keep 59–75 and G0's are 76–117,
and G0's open item O18 is O20, after Phase 1's O18 and O19.
**Phase 0 is closed.** Session 1 closed at P0.9, on `reboot`. Session 2 closed at S2.6, built
on branch `phase0-s2` (S2.1–S2.6, from `reboot` at `cf3c0ff`); `reboot` moved to it by a
fast-forward, at `397d7cd`, and on to Phase 1's work since; neither is pushed (`origin/reboot`
is at `cf3c0ff`). Session 2 built the certification stack, the GUI's engine asks and the
probe's criteria, and both tapes certify PASS. Before it: oracle unit 1a joined at P1.1 (O3);
the GUI's design, plan amendment A14 and R16 landed at P0.11 (O1); by your ruling of 2026-09-26
the Phase 2 probe ran first, with verdict GO ([docs/probe/REPORT.md](docs/probe/REPORT.md)) and
no fallback (decision 38); and Breakpoint B's pre-look passed beside it (S5.0,
docs/spine/EYEBALL.md; decision 35).
Next, in order: your look at the windows (the G0 gate's item and G1's p90, both checked by hand)
and at the demo's map; G1's remainder (what G0 moved to it, O36; the five majors its re-check
left, O37; unit 1g in the lab, O38); your rulings on 1g's decisions and its re-check (O35); your
rulings on P2.2b's decisions 286–311 (the loops, closed GO at chain8) and on 284 as 308 narrows
it; on the goods chain's line, storable running goods (O51) and a funded steam county (O62)
before a loop enters the 1750-like instance by its own registration; Phase 2 proper on loop-free
wall and commons instances, whose rulings Claude took at P2.3.0 (decisions 360–393, open to
veto); the demo's second pass, with goods, machine types and carriers, on the
many-market roles (O27), which may now start from the goods chain's stage v2a.1 on your ruling
on D-G12. "Next steps" has each.

## Where things stand

**Phase 2 proper's first scored wave: IW1, C1 and C2 are GO (P2.3.10–P2.3.15; 2026-09-30).**
Branch `phase2-proper`, label `run`, scratch `D:/rustyecon-p23/run/`, raw runs
`D:/rustyecon-p23/runs/`. The records: [docs/probe/results/wall/README.md](docs/probe/results/wall/README.md)
and [docs/probe/results/commons/README.md](docs/probe/results/commons/README.md); the machinery in
[docs/probe/results/p23-wave/](docs/probe/results/p23-wave/README.md).
- **The verdicts, as registered.** IW1 GO: mode A, Tiers 1–3 26/26, 38/38, 39/39, Tier 3S 20/20,
  Tiers 3 and 3S again at 10·L, 13/13 kick sets; the base's envelope decays at 0.5338 a year
  (0.534 registered). C1 and C2 GO: every non-vacuous run of Tiers 1–3 and 3S CONVERGED at L and
  10·L, C2's six VACUOUS runs VACUOUS, 26/26 kick sets. The subsistence trap falls exactly where
  the mirror put it, 203 runs name for name (joint, basin, tilt 1, C1's land.mach history at tick
  7,788, C1N's nine), and nowhere in the verdict battery. No refutation criterion is met, on one
  reading taken after the result (O108, open to veto).
- **The engine is the mirror.** In 3,633 compared runs every class is the mirror's, and every
  tick to tolerance of the 3,397 CONVERGED ones is the mirror's to the tick; troughs, depths,
  dead and breach ticks, regime ticks and switches, the thresholds' return, the edges (5.6e-16)
  all the mirror's or within its printed digits.
- **The lines.** 59,788 read: 37,073 pass, 22,591 reported, **124 fail**, none a class, a tick
  to tolerance or a verdict: 65 runaway ticks of trap runs at the commons, where the harness
  holds the bound to the displaced genesis and the mirror to the undisplaced point (O107; the
  mirror on the harness's reference gives the engine's tick in 190 of 195); 58 shadow rents at the
  end at C1 at 12 and 365 a year, read against the mirror's early-stopped value, up to 7.0e-8 off
  the oracle's where the engine is within 3.0e-13 (O108); and E6's "every CONVERGED run ends at
  the wall" at four runs at 365 a year, whose shares stall at 70 subnormal ulps, not 10 (O108).
- **P2.3.10, the harness's baskets count every household.** While the scorer was written, the
  markets harness's baskets eaten over Y\* were found to sum the provider's and the workers'
  baskets only, P2.1's two households; at the wall (IW1, IC1) they left out the trained and the
  master and read 0.725 at rest (E2's mode-A output, unread). The frame's mirror sums every
  household, so the wall's registered troughs (§6.2, E5) need it. `Stats::push` now sums every
  household, the first two first, so every other instance keeps its numbers bit for bit; one test
  fails without it; the gates are green on WSL and Windows (1,008 passed, 4 ignored; gate
  `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`). WALL-RULES §4,
  amended. No scored run had been made.
- **P2.3.11, the wave's machinery** ([docs/probe/results/p23-wave/](docs/probe/results/p23-wave/README.md)):
  the job list (3,721 jobs: the wall's E2–E9 on IW1 and IC1, the commons' E2–E9 on C1, C2 and
  C1N), the job and gather scripts, the scorer and its self-test, with the readings the scorer
  takes of each registered tolerance. On the mirrors' own outputs every line the registered files
  feed passes, and its E9 reproduces the frame's 117 edge gaps within 1e-12. O22's families on
  I1–I3, which decision 368 orders first, are not in this wave (reported only, no registered
  predictions); they are their own wave.
- **P2.3.12, dated shocks of the commons load.** Twelve minutes into the wave (started 09:29 on
  the P2.3.11 binary), twelve jobs had failed alike: the tape writer wrote each dated shock's
  schedule param `Dimensionless`, and `inst.commons` and `inst.land` are `FlowPerYear`, so every
  `commons=V@dated`, `enclose=F@dated` and `cycle(commons,P,N)` tape was refused at load, 54 of
  the 3,721 jobs, all at C1, C2 and C1N; no wall job. No test had loaded such a tape. The
  schedule param now takes its target's unit, every other tape is the same text, and
  `commons_dated_shocks_load_and_fire` fails without it; gates green on both machines (1,009
  passed, 4 ignored; the pins unmoved). COMMONS-RULES §4, amended. The wave ran to its end
  unchanged; the 54 jobs run again on the fixed binary, with an R1 check of 24 untouched jobs
  byte for byte ([rerun/](docs/probe/results/p23-wave/rerun/README.md)).
- **P2.3.13, the runner kept the jobs' quotes.** The wave ended at 10:08 with 703 jobs never
  started: its runner, `xargs -I{} bash -c '{}'`, stripped the job lines' quotes, so every run name
  with a parenthesis (JA, JB, N, RC, RW, joint, cycle) was a shell syntax error before its job
  script. They ran from 10:17 on the binary their list names, with `xargs -d '
'`; every run's
  printed name is its job's. Nothing had been scored.
- **P2.3.14**: one column of the commons' `verdicts.csv` (kick sets passed, 1/13 for 13/13); no
  line moved (`p23-wave/FIXES.md`).
- **P2.3.15, the scoring.** 3,721 jobs and 24 R1 checks, every exit 0; `runs.jsonl` regenerates
  from the gzipped archive byte for byte. Tables and READMEs in `results/wall/` and
  `results/commons/`, plots in `figs/wall/` and `figs/commons/`, diagnostics in
  `results/commons/diag/`. Open items O107–O109; O106 closed.
- **Next** (next step 7): the reviews of this wave and its report; O22's families on I1–I3
  (O109); your rulings on O107 and O108; then the 1750-like instance, which waits for the trap's
  remedy (O100) and the addenda of O95.

**The open-commons instances' E0–E2, before any scored run (P2.3.7–P2.3.9; 2026-09-30).** Branch
`phase2-proper`, scratch `D:/rustyecon-p23/build-commons/e0/`. The record:
[docs/probe/results/commons/e0.md](docs/probe/results/commons/e0.md), with its outputs in `e0/`.
- **E0 at C1 and C2**: the committed build (`d666f2d`) against the frame's mirror `cm.py`, seven
  runs of 2,000 ticks, agrees within 2.6e-14 in log in every price, ratio, share, stock, coin, S
  and D; the workers' participation within 1.0e-14; the plots' regime, rented land and shadow
  rent equal the mirror's at every tick, through the regime switches. The joint draws are the
  mirror's (O106's per-seed predictions stand).
- **E0's trap run.** The negative control's `p[mach]*0.5` parts from tick 41 and by 1.6e-7 at tick
  78, and runs away at the mirror's tick, 284. The engine is the mirror's one-step map within
  2.3e-15 at every tick, and before the parting its prices and coins agree within 1.2e-14 and
  2.9e-14: near zero participation F = ln1p((w − e)/(P_s + e))/χ_max is a small difference of
  large numbers, and the trap's inflation carries it.
- **A1** (P2.3.7, `d666f2d`; [registration-A1.md](docs/probe/results/commons/registration-A1.md),
  sha256 `4fefa0a4…f98e`), written after the development trace diff and committed before the
  official run, reads such a run. Its second condition named a mirror move of 1e-15, which parts
  the mirror by 4.7e-8, short of the engine's 1.6e-7, as A1's own evidence said. **A2** (P2.3.8,
  `bdd9e78`; [registration-A2.md](docs/probe/results/commons/registration-A2.md), sha256
  `a4d08a07…1eee`), written after the official run and disclosed as such, replaces it: before the
  first parting every price and coin within 1e-13. Under A1 alone E0 would not pass on that run.
- **E1 holds** (P2.3.6): every committed tape's text, ids and stream on both machines, the pins,
  the nesting tests.
- **E2 holds.** The rest point within 1e-12 at 78 points. Mode A at L passes: largest gaps 2.0e-15
  (C1) and 5.6e-16 (C2) at 52 a year, 3.1e-14 and 2.2e-16 at 12, 5.1e-15 and 6.7e-16 at 365. The
  base kick sets pass (largest tail gain 1.3e-5). The engine's elasticity probe gives the frame's
  τ to the printed digit (care's 703.3 and 705.4 ticks), so L is the registered 141,000 and
  142,000.
- **Next** (next step 7): the scorer, its gather script and job list committed before the wave
  (decision 311), then the scored wave E3–E9 on C1, C2 and C1N, beside the wall's.

**The open-commons instances' build (P2.3.6; 2026-09-30).** Branch `phase2-proper` (worktree
`D:/rustyecon-wt/p23`, scratch `D:/rustyecon-p23/build-commons/`). As built:
[docs/probe/COMMONS-RULES.md](docs/probe/COMMONS-RULES.md); ENGINE.md and TAPE.md, "Amended at
P2.3.6".
- **The roles' addition** (decision 398): one optional field, the workers' `exit` (the exit good,
  s₀, s̲, h, the commons T_o and the plots' land). Their hours become min(max(n(0), N − T_o/h),
  n(r̂)) through `workers_participation`, which the harness reads too; plots that spill onto
  enclosed land are bought at r in the chain of their baskets and burned in `produce`. No new
  kind, state or market rule; the field is skipped when absent, raw and resolved.
- **The instances** (decision 399): C1, C2 and the negative control C1N in `probe::markets`,
  solved by unit 1e's `ParcelEconomy`; at the 26 targets the harness's point is the frame's
  50-digit solve within 1e-13. The tapes are `tapes/markets-c1.ron` and `markets-c2.ron` (the
  frame's names in lower case; the run's brief said `p2-commons*`).
- **The harness at the commons**: targets from 1e (labour's S, land's T in force); the plots'
  regime, shadow rent and T_p each tick through the workers' own rule, with `commons.*` lines in
  `stats.tsv`; `commons=V` and `enclose=F`; the battery (115) and the tier3s, stocks, joint,
  basin, history and enclose families equal the registered runs name for name.
- **R1.** The 23 committed tapes' 2,000-tick hash streams, `tape_hash` and `world_id` equal the
  pre-build binary's (`7ea124d`); with the exit switched off the workers are P2.1's bit for bit,
  rule by rule over 3,000 states and run by run over 2,000 ticks.
- **Tests**: four in `crates/agents/tests/commons.rs` and twelve in `crates/probe/tests/commons.rs`
  (COMMONS-RULES §6); two existing tests read C1's tape too. 30 mutants, one per change undone,
  each killed; three survived a first pass and their tests were strengthened before this commit.
- **The gates**: `scripts/gate.sh` and `scripts/gui.sh` are green on WSL and Windows, 1,007 tests
  passed and 4 ignored on each; gate hash `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb
  `0xfad880fe08d06645`. The stamp is `7ea124d`, dirty: this build before its commit.
- **Development runs** before E0, disclosed (COMMONS-RULES §6.5): a trace diff on the
  uncommitted build found C1 and C2 within 2.6e-14 of the mirror, and the negative control's trap
  run parting from tick 41, where the engine is still the mirror's one-step map. A1 (next)
  answers it.

**The open-commons instances' registration (P2.3.5; 2026-09-30).** Branch `phase2-proper`
(worktree `D:/rustyecon-wt/p23`, scratch `D:/rustyecon-p23/frame-commons/` and `build-commons/`).
Docs only: [docs/probe/results/commons/registration.md](docs/probe/results/commons/registration.md),
the frame's files in [docs/probe/commons/](docs/probe/commons/), Tier 3S's mirror runs in
`docs/probe/results/commons/tier3s/`, and this file.
- **The frame** (`SPEC.md`, sha256 `60f21f56…b40d`; its `SHA256SUMS`, 97 entries, each checked).
  C1 and C2 are the markets probe's I1 (C3's four categories, 1a's machine, N 208 and T 520 a
  year, χ_max 1, support one basket) with one priced worker type whose exit good is food, and a
  commons T_o the workers hold and never trade.
  - C1: exit (s₀ 0.3, s̲ 0, h 0.135), commons 24.3 a year. The commons is full, rationed by a
    shadow rent of 0.43792 of the land rent that nobody receives; participation 7/13 of the
    heads' hours; x\* 0.748338.
  - C2: exit (0.2, 0, 0.09), commons 31.2 a year. The commons has room; 48.8% of it idles at zero
    rent; x\* 0.743502.
  - 26 targets (land.mach, b.food and the commons at ×1.1, ×0.9, ×2, ×0.5) span the three regimes,
    each interior, funded and alone. Unit 1e's oracle and a 50-digit solve that reads no oracle
    code agree within 5.6e-15.
- **The zero-rent land market** (O96). The frame's mirror scan found that the commons posted as a
  market runs to the runaway bound whenever it has room (Saturate and Hold alike), a reservation
  orbits off the point, and a floor order leaves a continuum of rest points. The chosen answer is
  no market: the commoners' participation rule gives out the plots, and the shadow rent is a
  readout. It rests exactly in all three regimes and is the fastest locally (half-life 49 ticks
  against 876). Idle enclosed land at r = 0 is unfunded under the probe's transfer (the provider's
  baskets are −N there), so no instance of this run holds it (O96, amended).
- **The additions** the build will make: one optional `exit` block on `BasketWorkers`, hours =
  min(max(n(0), N − T_o/h), n(r̂)) (decision 149 for one priced type), and a land buy for plots
  that spill onto enclosed land. Nothing else changes, and no state is added.
- **The mirror's verdict: GO for both.** Mode A passes at 12, 52 and 365 a year. Tiers 1–3 and
  Tier 3 at 10·L: C1 115/115, C2 109/109 with 6 VACUOUS by construction. Largest root per tick
  0.98538–0.99139. Medians 319/372/508 (C1) and 315/360/514 (C2) ticks by tier.
- **Tier 3S added** (decision 368, which the frame's verdict left out): 24 runs an instance, run on
  the frame's mirror unedited before any code, all CONVERGED at L and 10·L (medians 379 and 340).
- **The subsistence trap**, registered as the families' prediction: with exit valued at food's
  posted price, nobody working is a second absorbing state. C1 4/60 joint2, 12/40 joint4, 89/430
  basin; C2 8/40 joint4, 77/430 basin; never in the verdict battery; I1 0/530 (O100).
- **Readings declared before the build** (registration §3): Tier 3S, the tapes' names
  (`markets-c1.ron`, `markets-c2.ron`; the brief said `p2-commons*`), the mirror's run names,
  `joint` in the mirror's order, `enclose` as two changes at one tick, labour's target S and
  land's T in force, the workers' state share the regime's F, the readouts in `stats.tsv`.
- **Decisions 398–399, O96 amended and O100–O106**, below: the frame's 360–371 grouped (two numbers
  of the range were left) and its O95–O102 renumbered.

**The wall instance's E0–E2, before any scored run (P2.3.4; 2026-09-30).** Branch
`phase2-proper`, scratch `D:/rustyecon-p23/build-wall/e0/`. The record:
[docs/probe/results/wall/e0.md](docs/probe/results/wall/e0.md), with its outputs in `e0/`.
- **E0 passes under A1.** The trace diff of the committed build (`1ecbb7a`) against the frame's
  mirror `wm.py`, 2,000 ticks, 12 runs at IW1 and I0's hold: every price, ratio, threshold, stock
  and coin within 1.31e-14 in log, every share to the bit, every S and D within 4.4e-15 but at one
  tick. Four partings pass 1e-12, all at tick 1 of `stock.mach*0.01`: services' and goods' S and
  cleared volume, a leftover genesis lot, 3.4e-17 absolute, where the mirror one ulp from itself
  parts without bound. A1 allows them; without it E0 would fail on them alone.
- **A1** (P2.3.3, `1ecbb7a`; [registration-A1.md](docs/probe/results/wall/registration-A1.md),
  sha256 `0f609f20…0dfd`) was written after the development trace diff found the residue and
  committed before this run and any scored run. It allows a market's S, D or cleared volume to
  part where it is a residue at most 1e-12 of its target volume and the mirror one ulp apart parts
  too, reported in absolute terms. No prediction moves. Its text names the services market; the
  goods market's residue is the same one.
- **E1 holds** (P2.3.2): every committed tape's text, ids and stream, the pins, the nesting tests.
- **E2 holds.** The rest point within 1e-12 at 39 points. Mode A at L passes: largest gaps 3.3e-16
  (52 a year, L 22,000), 4.4e-16 (12, 20,000) and 4.4e-16 (365, 164,000), and IC1's 3.3e-16 at
  25,000. The engine's elasticity probe gives SPEC §6.1's τ to the printed digit (goods 106.4
  ticks), so L is the registered one at every tick length.
- **Next** (next step 7): the scorer, its gather script and job list committed before the wave
  (decision 311), then the scored wave E3–E9 on IW1 and IC1, O22's families on I1–I3 first
  (decision 368).

**The wall instance's build (P2.3.2; 2026-09-30).** Branch `phase2-proper` (worktree
`D:/rustyecon-wt/p23`, scratch `D:/rustyecon-p23/build-wall/`). As built:
[docs/probe/WALL-RULES.md](docs/probe/WALL-RULES.md); ENGINE.md and TAPE.md, "Amended at P2.3.2".
- **The roles' additions** (decision 395): the category desk's optional `tail` (h = H + L^H) and
  `reserved` (each reserved type's hours in the cost, the orders and the Leontief), and the
  provider's optional `more` (further transfers N_i·P_s in list order, its state the sums). No
  new kind, state or market rule; each field is skipped when absent, raw and resolved.
- **The instance** (decision 394): IW1 and the control IC1 in `probe::markets`, solved by unit
  1d's `WorkerEconomy`; P2.1's instances keep unit 1c. Genesis equals the frame's registered
  point and its mirror's coins bit for bit. The tape is `tapes/markets-iw1.ron` (the frame's
  name; the run's brief said `p2-wall*`).
- **The harness at the wall** (decision 396): 18 observables with each reserved wage and each
  desk's threshold x = 1 − s; every household's baskets; the wall's readouts in `stats.tsv`;
  `s[D]=V`, `RW(F)`, JA and JB on every labour market, joint in the mirror's order; the battery
  (103 runs) and the tier3s, stocks, joint and basin families equal the registered TSVs name for
  name; Tier 3S reads its first year's D̂ (`family tier3s`, `--first-year`).
- **R1.** The 22 committed tapes' 2,000-tick hash streams, `tape_hash` and `world_id` equal the
  pre-build binary's on WSL and on Windows; P2.1's generators write their tapes byte for byte.
- **Tests**: five in `crates/agents/tests/wall.rs` and nine in `crates/probe/tests/wall.rs`
  (WALL-RULES §6); 20 mutants, one per change undone, each killed.
- **Readings** (WALL-RULES §7): the basin's factors 1.05^j correctly rounded (`pow_whole`), since
  libm's `pow` is an ulp off the mirror's at some j; the joint draws keep libm's `pow` (reported
  families); mode A's spoilage check reads produced goods only.
- **Development runs** before E0, disclosed (WALL-RULES §6.5): six IW1 runs of 3,000 ticks and a
  trace diff on the uncommitted build, which found one tick-1 residue (E0, next).
- **The gates**: `scripts/gate.sh` and `scripts/gui.sh` are green on WSL (`/root/scratch/target-p23`)
  and Windows (`D:/rustyecon-targets/p23`), 991 tests passed and 4 ignored on each; the gate hash
  is `0x61f9c8529131ff17`, and gui.sh's hash check gives appb `0xe1fa082b26995867` and demo-gb
  `0xfad880fe08d06645`. The stamp is `044deb1`, dirty: this build before its commit. Logs in
  `D:/rustyecon-p23/build-wall/`.

**The wall instance's registration (P2.3.1; 2026-09-30).** Branch `phase2-proper` (worktree
`D:/rustyecon-wt/p23`, scratch `D:/rustyecon-p23/frame-wall/` and `build-wall/`). Docs only:
[docs/probe/results/wall/registration.md](docs/probe/results/wall/registration.md), the frame's
files in [docs/probe/wall/](docs/probe/wall/), and this file.
- **The frame** (`SPEC.md`, sha256 `cc2b6d1c…9be0`; its `SHA256SUMS`, 56 entries, each checked).
  IW1 is unit 1d's B economy (services with a human-required tail of 0.1 hours a unit, goods,
  space 1, Appendix B's machine, η 0.5, T 520 a year) with E7's three worker types: the entrant
  (130 a year, χ_max 1) sells to the pool; the trained (52, χ_max 2) and the master (26, χ_max 2)
  do only their reserved hours, 0.04 a unit of services and 0.03 a unit of goods, each on its own
  labour market. Every type's support is one basket.
- **Why it is a solved wall.** The oracle (`WorkerEconomy::solve`) gives margin Wall at x\* = 1.
  The pool's wage 0.807310 is set by its own clearing, 0.94 in log above γ(1)·π; the reserved
  wages 1.303538 and 2.325461 by their own markets, at participation 0.31 and 0.46. Every one of
  the 12 cost targets (land.mach, tail.services, res.services.trained at ×1.1, ×0.9, ×2, ×0.5) is
  a funded wall at least 0.216 in log from every edge (decision 370). A 50-digit solve that reads
  no oracle code agrees within 1.13e-15 at all 39 points, one sign change each.
- **The roles' additions** are three optional fields, absent on every committed tape: the category
  desk's `tail` and `reserved`, and the provider's `more`. No new kind, state or market rule.
- **The mirror's verdict: GO.** Mode A passes; Tiers 1–3 26/26, 38/38, 39/39 and Tier 3S 20/20;
  every target's kick set passes; the largest root per tick is 0.9832–0.9919. Only JB(0.5)
  breaches the wall (11 ticks). The families all converge.
- **Readings declared before the build** (registration §3): E0's coefficient runs, 18
  observables, the registered run names, the joint draw order and `s[D]=V` exact.
- **Decisions 394–397 and O97–O99**, below: the frame's FW1–FW9, grouped, and OW1–OW3.

**Phase 2 proper's rulings (P2.3.0; 2026-09-30).** Branch `phase2-proper` (worktree
`D:/rustyecon-wt/p23`) from `reboot` at `f7d1eae`. Docs only: this file.
- **The rulings.** Next step 7 waited on 34 rulings: 61, 67 and 70 from 1b and 1c, 118–123 from
  the markets probe, those of 1d–1f marked "Binds Phase 2", 179 (D-G1) and 186 from 1g, and
  GOODS-CHAIN's D-G13 to D-G15. Claude took them on your word as decisions 360–393, each open to
  veto. 30 are taken as proposed, some with what they mean for the agents: the pool's market
  form is the frame's call (371), and a crowded commons' shadow rent must be posted (374). Four
  are amended: O22's families run first and the new instances carry a stocks tier (368); the
  wall's distance from each edge is registered (370); the commons instance is certified by 1e's
  Proposition 5 (378); and ρ 0 extends to Phase 2 proper's first instances (393).
- **What they fix for this run.** The wall instance takes 1d's dependence form, since a walled
  type has no solve under s(q) (377). The commons instance takes s(q), every type pooled, food as
  its exit good. Both run at C2m, 52 ticks a year and ρ 0, with the fixed basket in the oracle
  and the agents (364, 366, 387, 393). Machine stocks, when they come, run at C2g (391).
- **New open items.** O95 lists the oracle addenda that later instances need. O96 is the land
  market at zero rent, whose remedy a mirror scan chooses before the commons instance is
  registered.

**P2.2b's reviews, fix round and report (P2.2b.4; 2026-09-30). P2.2b is closed.** Branch
`phase2-plants` (worktree `D:/rustyecon-wt/p2b`, scratch `D:/rustyecon-p2b/fix-report/`). The
report: [docs/probe/LOOPS.md](docs/probe/LOOPS.md); as run: LOOPS-RULES §17.
- **The verdict, corrected.** GO for rule B's horse loop at chain8 (C2g, 52 ticks a year, ρ 0,
  J_b 1, fodder that cannot be stored), with CAPACITY's plant on every loop desk and the maker's
  reservation at ψ 0.25 together; capital's time reported, not scored. No scored line or class
  moved.
- **Two bounded reviews** (`D:/rustyecon-p2b/review-measurement/`, `review-fidelity/`), both
  finding that the verdict holds.
  - Measurement: a fresh build gives the frozen binaries byte for byte; 147 wave jobs on WSL and
    8 on Windows rerun byte for byte; an independent rescore reproduces every count and table.
  - Fidelity: the plant layer matches `lm_carry.py` line by line, with no hidden clamp.
  - Findings: three majors, all fidelity's, and five minors (three measurement, two fidelity).
    None is a code defect, so no code changed and nothing was rerun for scoring.
- **The majors, answered in LOOPS.md §4–§6 with new tables** (results README, "Added at P2.2b.4").
  - The reservation's part (decision 307). It acts in 117 of LB1–LB3's 375 runs. Without it, LB1
    loses r × 2, heads × 2, heads × 10 and the workers' coin × 0.02 and × 0.1. Through r × 2 and
    the gluts at LB1–LB3 the maker's coin falls to 1e-5 of genesis or below (4e-90 at worst),
    and the horse price rests unmoved at 0.15–0.22 of target for up to 48 years. Decision 264 is
    reworded; O52 and O54 are amended.
  - Capital's time (decision 309). The maker's plant is the last observable into tolerance in 59
    of LB1's 97 converged battery runs and 82 of LB3's. The heads take 1.06–2.1 times the paper's time after b × 2. O46 is
    amended.
  - Decision 284's reach (decision 308, O79): a loop enters Phase 2 only through its own
    registration at its dials and in its own funded county.
- **The minors.**
  - Decision 304 was taken after E0's first output. It is disclosed, and the three partings under
    the registered reading are written up (e0.md §9; decision 310).
  - The scorer's stamp was a scratch time, and four patches edited the scorer during the wave. This
    is disclosed; from now on the scorer is committed before a wave (decision 311).
  - The archive now rescores. `gather_gz.py` and a fail-closed `score_fc.py` reproduce `runs.tsv`
    and every scored table byte for byte.
  - O14's ratio is 1.17 on the observables both economies share (1.94 at δ 4%). The b × 2 trough
    is 217–239 times the equilibrium's move in log (O14 amended).
  - o14.csv gains `T3_peak_ex_runaway`.
- **Reruns, none scored**: 21 E6 and E7 runs for 10,400 ticks with a row every tick, on the frozen
  binary (`/root/scratch/p2b-fix/fine`, 340 MB), for the maker's lowest coin and the frozen price.
- **The gates**, logs in `D:/rustyecon-p2b/fix-report/`: `scripts/gate.sh` and `scripts/gui.sh`
  are green on WSL (`/root/scratch/target-p2b`) and on Windows (`D:/rustyecon-targets/p2b`).
  The stamp is `6ae6674`, clean (docs-only changes since); gate.sh was green again at
  `2cede8a`. The gate hash is `0x61f9c8529131ff17`.
- **Decisions 307–311 and O79–O80**, below. O49, O65 and O68 are closed. O14, O46, O52, O54 and
  O67 are amended.
- **Housekeeping.** `/root/scratch/p2b-runs` (2.9 GB, the plain duplicate of the archive) is no
  longer needed to rescore and can be deleted on your word. So can `/root/scratch/p2b-fix`.

**P2.2b's scored runs (P2.2b.3; 2026-09-30).** Branch `phase2-plants` (worktree
`D:/rustyecon-wt/p2b`, scratch `D:/rustyecon-p2b/run/`, raw runs `D:/rustyecon-p2b/runs/`). The
results: [docs/probe/results/loops/README.md](docs/probe/results/loops/README.md), its CSVs and
[docs/probe/figs/loops/](docs/probe/figs/loops/). As run: LOOPS-RULES §16.
- **The runs.** LOOP-SPEC-A1 §A1.5's E3–E11 as LOOPS-RULES §9 names them, on P2.2b.2's harness
  (`3205025`, `horses` sha256 `e0e8f2a6…893a`), unchanged. 3,549 runs and 24 kick-set jobs, one
  process each, 46 at a time on WSL: 74 minutes, about 2.3 billion ticks. Every job exited 0.
- **The scoring.** `score.py` was stamped before any scored output was read and first scored the
  mirror's own outputs. It reads A1's tables from the registered file and the mirror's run-by-run
  outputs. 1,377 lines: 1,276 pass, 100 reported, LN5 not run (A2). **None fails.**
- **E3.** LB1–LB3 GO: 20/20, 24/24, 25/25 (25/25), 28/28 (28/28) and 28/28 each; all 537 runs
  CONVERGED. A1 §7.1's years, lowest baskets, dead ticks by market and horse price lows hold to
  their printed precision, but for four one-tick ties.
- **E4.** Every kick set decays (24 of 24). Base g 0.8478, 0.8310, 0.9008 and 0.8463 a year at
  LB1–LB3 and LW1, against 0.847, 0.830, 0.901 and 0.836; the largest g at any target is 0.998971
  a tick (LB3, b × 0.5).
- **E5–E7.** A1 §7.2's cost shocks, §7.3's glut (withheld 303, 1,315; 221, 1,034; 662, 2,694) and
  §7.5's pass-through (LB1 r × 2: 0 horse-day dead ticks, fodder 175, good 11, land 12; LN7 146,
  ORBITING) hold to their printed precision.
- **E8.** LW1–LW3 GO. O14 like for like: stocks lift the b × 2 trough by 0.0362 in log and take
  1.361 times the flow control's median years (registered 0.036 and 1.36).
- **E9.** Every control keeps its verdict, with A1's converged counts in every tier but LN7's
  stocks family (0/22 against 1/22, within 2). Every named outcome holds: LF3's five runaways at
  ticks 156–173; ψ 0's at 155–158; LB1 at 12 a year 0 of 40.
- **E10, E11.** The six families GO with A1's numbers; LF4 is not built. The 28 runs at labour's
  bound have A1's ticks and peaks; none starts there, and each converges.
- **The engine is the mirror** at the 12 GO instances, in 2,052 runs: every class, every tick to
  tolerance, every withheld tick and every tick at the bound; lowest baskets within 2.4e-6 and
  horse prices within 1.0e-5. Dead ticks part by one tick in 13 runs (26 with their 10·L twins),
  each a market at exactly half its target at tick 0 or 1 (O77).
- **No refutation:** no class change or runaway at LB1–LB3; the largest end gap of a converged
  run 1.9e-13; the largest slowest mode 0.998971 a tick.
- **No code changed**, so no gate ran; the pins and streams are P2.2b.2's.
- **Decisions 305–306 and O77–O78**, below. O72 is closed.

**P2.2b's harness, and E0–E2 (P2.2b.2; 2026-09-30).** Branch `phase2-plants` (worktree
`D:/rustyecon-wt/p2b`, scratch `D:/rustyecon-p2b/build-harness/` and `D:/rustyecon-p2b/e0/`).
As built: [docs/probe/LOOPS-RULES.md](docs/probe/LOOPS-RULES.md) §15. The evidence:
[docs/probe/results/loops/e0.md](docs/probe/results/loops/e0.md).
- **The harness** (LOOPS-RULES §8; decision 302) is `probe::horses::loops`:
  - `perturb`: the run grammar with the plants, `b*F` at genesis and dated, and the families;
  - `harness`: the observables with the plants, and §8.3's readouts: dead ticks by market,
    labour's bound, the horse-days by buyer, `pk_high`, each plant's 5% tick and range, the
    reservation's;
  - `probes`: the elasticity probe, the kick set and the run length's rule;
  - `cmd`: `horses` sends a command with a loop id there.

  It shares P2.2a's row, statistics, classifier, kick set and reports (now
  `probe::horses::report`). The running cost is read from the tape's recipe, in P2.2a's harness
  too; P2.2a's outputs are byte-identical before and after, on 17 runs.
- **E0 passes; no amendment.** The trace diff against `lm_carry.py`: nine runs at LB1 and three
  at LW1, 2,000 ticks each from the engine's own genesis.
  - Nothing parts at tick 1, so the planted desks' carry is the mirror's (decision 273).
  - Eight runs agree within 8.7e-13.
  - Four part on the horse market's cleared volume, by 3.9e-12 to 2.9e-11 (r × 2, b × 2,
    heads × 2, heads × 10). There the volume is a small difference, the capacity desk's order
    or the maker's offer: its inputs' gaps (at most 1.6e-13) times the cancellation (3–507),
    within a factor of 6.4. The mirror with an ulp each tick parts farther.
  - The maker's coin and output and, after heads × 10, the horse price (1.8e-12) follow that
    volume.
  - The withheld ticks agree tick for tick.

  The allowance is read on both sides of the horse market (decision 304).
- **E1 holds.**
  - The pins, and the 13 horses and markets tapes' 2,000-tick streams on WSL and Windows.
  - θ = 1 is the plain tape (P2.2b.1's test).
  - The fixed plant's trace diff against lm_carry at plant rule "none" agrees within 7.4e-14 but
    the horse market's volume (1.7e-12 and 2.4e-12, the maker's offer's cancellation).
- **E2 holds.** Mode A at L (211,000, 202,000 and 232,000 ticks) passes at LB1–LB3 and LW1–LW3,
  with largest gaps 8.9e-16 to 2.4e-15; the mirror's are 6.7e-16 to 4.5e-14. The engine's τ are
  the mirror's (the good's 1,052 ticks at LB1), so its L is the mirror's.
- **O72 timed**: one LB1 run at 10·L takes 65 s on one WSL core.
- **Tests**: LOOPS-RULES §10's five remaining ones, in `crates/probe/tests/loops.rs`. 16 mutants
  were run in two passes (`D:/rustyecon-p2b/build-harness/mutants/`). Two survived the first,
  and their tests were strengthened. Build-plant's two plan-cap mutants survive the carry test
  by construction and stay killed by `planted_desks_take_carried_goods` (LOOPS-RULES §15.4).
- **The gates**: `scripts/gate.sh` and `scripts/gui.sh` green on WSL and Windows
  (`D:/rustyecon-p2b/build-harness/`).
- **Decisions 302–304 and O75–O76**, below. O73 is closed.

**P2.2b's plant layer (P2.2b.1; 2026-09-30).** Branch `phase2-plants` (worktree
`D:/rustyecon-wt/p2b`, scratch `D:/rustyecon-p2b/build-plant/`). As built:
[docs/probe/LOOPS-RULES.md](docs/probe/LOOPS-RULES.md) §14; ENGINE.md and TAPE.md, "Amended at
P2.2b.1".
- **Core's untraded good** (decision 286): `untraded: true` on a good, absent `false`. It is
  `Indefinite`, has no price rate, market or genesis price, and no order, role good or price shock
  may name it. `World::markets()` walks the goods with a market only, so clearing, the price rule,
  the report, certify's kick set and the GUI never see a plant.
- **The plant** on the type desk, the maker and the capacity desk, an optional last field
  (`plant: Some((good, theta, delta, size, adjust, order, target))`), with three `ActorState`
  variants after `Owner`. The kinds' own rules take the plant as an option, so a desk without one
  runs its kind's code; a planted desk at θ = 1 is its kind bit for bit.
- **The loop instances** (`probe::horses::loops`): LOOPS-RULES §7.1's rows but LF4 and LN5,
  FUNDED's point, genesis at the price-free rest ratio, and `tapes/loops-{lb1,lb2,lb3,lw1,lw2,
  lw3}.ron`, written by `horses-tape --inst <id>`. Their params equal `instances.json`'s bit for
  bit, their points FUNDED's within 1e-15, and each genesis plant κ_p·X its K* within 1e-12.
- **R1 held:** the 13 horses and markets tapes' 2,000-tick streams equal the P2.2 report's on WSL
  and Windows; the gate world's final is `0x61f9c8529131ff17`; every committed tape keeps its
  `tape_hash` and `world_id` (now pinned for all 13).
- **What the tests show:** the oracle's point is a fixed point at all 23 built instances and 5
  targets within 1e-12, but LN6 (its whole-gap order, 3.0e-12; decision 301); LB1 at θ = 1 is LN1
  and LW1 at θ = 1 is LW0 bit for bit over 2,000 ticks, at hold and from w × 2 and r × 2; a fixed
  plant stays at Q with no order; every tape conserves, each wear exactly δ·H or u·P. Each of the
  build's 28 mutants is killed by a named test, two after a test was strengthened (LOOPS-RULES
  §14.4).
- **One engine change** (decision 297): `RunErrorKind::ForeignWrite` boxes its delta, since the
  planted states grew every delta. No run, hash or message moved.
- **The gates**, logs in `D:/rustyecon-p2b/build-plant/`: `scripts/gate.sh` and `scripts/gui.sh`
  green in WSL (`/root/scratch/target-p2b`) and on Windows (`D:/rustyecon-targets/p2b`) on the
  tree committed as P2.2b.1. 900 workspace tests pass on each (L0.8's 877 and 23 new), 3 ignored
  and run by name; certify alone 68; the GUI's 118, its 87 named ones among them; zero warnings.
- **Not built:** the harness's rule-B readouts (LOOPS-RULES §8) and the five tests that need them
  or E0 (O73).
- **Decisions 297–301 and O73–O74**, below.

**P2.2b's frame (P2.2b.0; 2026-09-30).** Branch `phase2-plants` from `reboot` at `8b07c8a`
(worktree `D:/rustyecon-wt/p2b`, scratch `D:/rustyecon-p2b/`). Docs only; no code, so no gate ran.
- **The spec:** [docs/probe/LOOPS-RULES.md](docs/probe/LOOPS-RULES.md), in HORSES-RULES' shape,
  written from LOOP-SPEC §2–§5 as A1 amends it, FUNDED, and the registered mirror read line by
  line (`lm_carry.py`, `lm_mirror.py`, `lm_run.py`, `lm_inst.py`; their sha256s checked). It names
  every new field, state, load check, readout and test, and how E0–E11 are run and scored.
- **What the build must add beyond the plant.** Core refuses a good that is not a currency and has
  no price rate, and LOOP-SPEC §2.2 gives the plant no market. So core gets an untraded good
  (decision 286). A plant market that no one trades would not do: certify's kick set kicks every
  market, and a kicked plant price never moves back, so every kick set would fail.
- **One departure, registered before any code:**
  [docs/probe/loops/LOOP-SPEC-A2.md](docs/probe/loops/LOOP-SPEC-A2.md) (sha256 `95de4411…c6b8a9`,
  in `SHA256SUMS`). LN5 needs storable fodder's seller and buyer netting (O51), which P2.2b does
  not build, so E9's LN5 items are withdrawn, as E10 already conditions LF4.
- **A readout to fix on the way.** P2.2a's harness forms a horse-day's running cost as rule A's
  one unit of fodder, so under rule B its quasi-rent and hour-over-O readouts would be wrong.
  The build reads the tape's running recipe, P2.2a's value bit for bit on P2.2a's instances
  (LOOPS-RULES §8.3; reported readouts, not scored).
- **Decisions 286–296 and O69–O72**, below.

**The loop stage's groundwork (L0; 2026-09-29).** Branch `phase2-loops` from `reboot` at
`401f7b1` (worktree `D:/rustyecon-wt/p2l`, scratch `D:/rustyecon-p2l/`). Its decisions are
240–283 and its open items O51–O68.
- **L0.1 (`ba6938d`): M6 narrowed, M5 checked** (O48 closed; decisions 240–244; ENGINE,
  amended at L0.1). The capacity and owner desks are exempt from M6 only for the durable good
  they hold, and a role that offers a good in full sells only one that lives at most a tick; the
  maker, selling under its cover, is exempt. The review's tape, which at `401f7b1` runs 520 ticks
  to a fodder desk holding 49 times its genesis fodder, is refused at load. No rule or state
  changed: the six horses tapes and the seven markets tapes give the P2.2 report's WSL hash
  streams for 2,000 ticks bit for bit (`D:/rustyecon-p2l/m6/tapes-bit.log`). Storable running
  goods now have no role that handles them (O51).
- **The gates**, logs in `D:/rustyecon-p2l/m6/`. On the tree committed as L0.1, before the
  commit: `scripts/gate.sh` and `scripts/gui.sh` green in WSL (`/root/scratch/target-p2l`) and on
  Windows under Git Bash (`D:/rustyecon-targets/p2l`, fresh: gate 312 s, gui 279 s). 867
  workspace tests pass on each (P2.2.4's 865 and the two new), 3 ignored and run by name;
  certify alone, Parquet-free, 67 with 1 ignored; zero warnings; the gate hash is
  `0x61f9c8529131ff17`; the probe's pin passes; the GUI's 87 named tests pass, and the cli's
  hashes of gate, appb and demo-gb equal the GUI's (finals `0x61f9c8529131ff17`,
  `0xe1fa082b26995867`, `0xfad880fe08d06645`); G1's sweep takes 0.73 ms, median of 20, in WSL.
  At `ba6938d`, `scripts/gate.sh` green again on both (175 s and 168 s, warm), its stamp
  `ba6938d`, clean.
- **L0.3–L0.6: the idle machine market's remedy** (O47 closed; decisions 245–253; O52–O57;
  [docs/probe/results/idle/README.md](docs/probe/results/idle/README.md)). The mirror scan
  (`D:/rustyecon-p2l/idle-scan/IDLE-SPEC.md`, sha256 `c5c6527a…`) chose the maker's reservation:
  the maker offers no finished heads while its net markup p_K·(1 − δ·a/κ)/c_m at posted prices is
  below ψ = 0.25. Four commits:
  - **L0.3 (`96cba59`), the registration**, written before the rule existed in the engine: the
    spec's §8 unedited, the protocol E1–E8 at P2.2a's L, and the tolerances.
  - **L0.4 (`882e9f8`), the build** (HORSES-RULES §10; ENGINE and TAPE amended): an optional
    `reserve` param on the maker, absent on every committed tape. It is refused on the flow path
    and under `Hold`, and set by `--reserve PSI` on both binaries. The harness reads the markup
    outside the Sim. Six tests, each failing with its change undone.
  - **L0.5 (`fac7db4`), a fix before any scored run.** `world_id` hashes the resolved actors, so
    L0.4's resolved `reserve: None` moved the `world_id` of every tape with a maker. Their text,
    `tape_hash` and hash streams did not move. E1's hash check found it. The resolved field is now
    left out when `None`, and a test pins the six P2.2 `world_id`s.
  - **L0.6, the runs and this record.** Release, WSL, 99 minutes on 48 cores; raw runs in
    `D:/rustyecon-p2l/idle-engine/runs/`.
- **The verdict: the registered predictions hold, and none of the refutations occurred.**
  - E3: heads × 10 is CONVERGED at all 14 instances, where P2.2a ran away at ticks 138–167. The
    ticks to tolerance equal the mirror's to the tick at 12 instances and differ by 1 at two. The
    lowest horse price is 0.135–0.193 of target, within 0.4% of the mirror's.
  - E4: P8 is STUCK at 40,000 ticks and CONVERGED at 400,000, in tolerance from tick 24,601
    (473 years). Its lowest horse price is 0.2146 of target.
  - E5: all 1,980 battery runs converge, as in P2.2a. The rule acts in 85 runs at L and the same
    85 at 10·L; the 85th, F3 x\*/2, was registered as free to flip. The other 1,810 runs equal
    P2.2a's to every CSV row. The battery's lowest horse price at ω 1 rises from 0.001–0.009 of
    target to 0.187–0.214.
  - E6, E7: in the stocks, tick-length, P7 and P3 families, only the 14 heads × 10 runs change
    class.
  - E1, E2: every committed tape and pin is unchanged. Mode A, the slowest mode and the base kick
    set equal P2.2a's at all 14 instances.
  - The trace diff against the scan's mirror, carried as P2.2a's was, agrees within 1.6e-12 on
    H2 r × 2. On P8 it agrees to rounding against the oracle's volumes and outputs. On H2's glut it
    parts once orders resume against the withheld pile, where the mirror parts from itself by
    0.11 one ulp apart (O55). The withheld ticks agree tick for tick in all three runs.
- **Five misses, none on the refutation list** (the README's §"The misses"):
  - H2 r × 2's lowest horse price is 0.220 of target against the mirror's 0.204. The scan's mirror
    has no genesis carry (O57).
  - The rule acts in 85 battery runs at L against 84, and in 87 stocks runs against 88. Both
    differences are runs at the step.
  - E8's ψ 1 kicks are classed VACUOUS where the mirror said ORBITING, though they orbit exactly
    as the mirror's do (O56).
  - F2's post-glut horse price reaches 119.7 of target, against the spec's range of 8–112.
  - Not fixed, as the spec said (O52, O53): after a long glut, a shortage. Installed heads fall to
    0.40–0.53 of target and the horse price rises to 8–120 times it. P8 still needs 473 years.
- **The gates**, logs in `D:/rustyecon-p2l/idle-engine/gate/`.
  - Before L0.4 and again before L0.5, `scripts/gate.sh` and `scripts/gui.sh` were green in WSL
    and on Windows. The tests: 873 and then 874 workspace tests, 3 ignored; certify alone 67; the
    GUI's 117.
  - At `fac7db4`, `scripts/gate.sh` was green on both with a clean stamp (96 s and 139 s).
  - The gate hash is `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb
    `0xfad880fe08d06645`. The six horses tapes' and seven markets tapes' 2,000-tick hash streams
    and headers equal P2.2's (`D:/rustyecon-p2l/idle-engine/runs/e1/tapes-bit.log`).
- **Beside the engine work, in scratch: a funded county and the mirror's loop step.**
  - **FUNDED** (`D:/rustyecon-p2l/funded/`, now `docs/probe/loops/FUNDED.md` and
    `instances.json`; decisions 254–259; O41 closed): `chain8`, 1g's HORSE county at N 8. 1g's
    county can never be funded, since a basket is a good plus h = 1 of space, so N·P_s ≥ 12 > T.
    At N 8 coverage is 1.215–1.240 at every cost target and δ 4–10%, and the point is unique and
    interior with the loop live (x\* 0.747). A 50-digit solve that reads no oracle code agrees
    with the oracle within 9.5e-16 at 105 points (the good's price within 1.4e-14).
  - **LOOP-SPEC** (`D:/rustyecon-p2l/loop-mirror/`, sha256 `e16e0be2…`, now
    `docs/probe/loops/LOOP-SPEC.md`; decisions 260–271): GOODS-CHAIN §6 step 3 in the mirror.
    GO for one design only: rule B's horse on chain8 with CAPACITY's plant on all three loop
    desks (fodder, capacity, maker; θ 0.8, plant δ 10% a year, s1), beside the horses on the
    capacity desk, and the reservation on. NO-GO for every other arrangement tried: no plant
    (1.0147 a tick), the fodder plant alone, no maker plant (the reservation's relay cycle), the
    reservation off (runaway at ticks 155–173), a 4-week fodder store, 12 ticks a year. The
    instability starts in CHAIN's maker, which buys fodder at C2g's 5.2 a year, not in the loop;
    the capacity plant stops the pass-through into horse-days (146–150 dead ticks to 1–2).
- **The two reviews** (2026-09-29, `D:/rustyecon-p2l/review-engine/`, `review-mirror/`).
  - **Engine** (L0.1–L0.6): holds. It rebuilt `15b05af` on both machines and reproduced every
    check it tried; no refutation, five minor findings.
  - **Mirror** (FUNDED and LOOP-SPEC): the funded county holds, reproduced by a third,
    independent 60-digit solve; the mirror holds on composition, nestings, rest point and
    reproducibility. One major finding: the mirror had no genesis carry, and with it registered
    values moved past the registration's own tolerances (33 values at LB1 and 34 at LB3 in its
    sample). Eight minor.
- **L0.7 (`ea8e246`): the engine review's fixes** (decisions 278, 279; ENGINE, HORSES-RULES §10
  and the results' README amended; IDLE-SPEC-A1). Off is structural (ψ > 0); the harness reads
  the maker's reservation through the rule's own code; M6 gets a test only M6 can pass; the
  dated-target kick sets and the price bound are restated. Three tests, each failing with its fix
  undone (`D:/rustyecon-p2l/fix-report/mutants/`). The review's 17-run sample, rerun from this
  build, is L0.6's evidence row for row (4 of them on Windows too), and the 13 tapes give P2.2's
  hash streams (`D:/rustyecon-p2l/fix-report/`).
- **L0.8 (`452c0e7`): the loop step's registration, with the carry** (decisions 272–277, 280–282;
  O57 done for the loop step;
  [docs/probe/results/loops/registration.md](docs/probe/results/loops/registration.md)).
  - `lm_carry.py` adds the engine's genesis carry to the registered mirror. With every new layer
    off it is the trace diff's `i_carry.py` bit for bit over 6,000 ticks in 22 runs; at θ 1 its
    plant layer is plant-free bit for bit in 94 runs; from tick 2 it is `lm_mirror.py` bit for
    bit. A second map with the plant live, written from LOOP-SPEC's text, meets `lm_mirror.py`
    within 4.8e-12 over 2,000 ticks in 110 runs.
  - The loop battery ran again on it (3,794 runs, about 70 minutes on 47 cores). No class
    changes at any verdict, family or flow-control instance, and each tier's years move by at
    most 0.1 year. Registered values move past §8's tolerances in 9–12 runs per verdict instance
    at L, the one-tick goods' stock displacements most (ticks to tolerance up 11–38%, the lowest
    horse price down 6–46%). 29 runs flip class among the unstable negative controls, each
    keeping its verdict.
  - LOOP-SPEC-A1 (sha256 `8815d865…`) registers §8 again as its §A1.5, E0–E11: the design GO at
    LB1–LB3 and LW1–LW3, with every tier's years, lowest baskets, dead ticks by market and lowest
    horse price; after r × 2 at LB1 no dead horse-day tick (fodder 175); a b × 2 trough 0.036
    higher in log than the like-for-like flow control's, at 1.36 times its years; E0, the trace
    diff against `lm_carry.py`, before any scored run.
  - The review's minor findings: per-market dead ticks with labour reported (274); every
    negative control registered (275); S1 with pumping out of P2.2b (276); labour's bound
    registered run by run (277; chain8 reaches it in 5 of LB1's 125 runs, the χ_max 1/30
    alternative in 9); the good's price in FUNDED's check; the numbering.
  - LOOP-SPEC, FUNDED and instances.json are in `docs/probe/loops/` byte for byte, with
    LOOP-SPEC-A1 and FUNDED-A1 beside them, each with its sha256.
- **L0.9: the report, [docs/probe/IDLE.md](docs/probe/IDLE.md), and this record**
  (decision 283). O47 and O48 in one place: GO for the reservation at ψ 0.25, as registered and
  no wider; M6 and M5.
- **The gates for L0.7**, logs in `D:/rustyecon-p2l/fix-report/gate/`. On the tree committed as
  L0.7, before the commit: `scripts/gate.sh` and `scripts/gui.sh` green in WSL
  (`/root/scratch/target-p2l`, 306 s and 169 s) and on Windows (`D:/rustyecon-targets/p2l`, 300 s
  and 187 s). 877 workspace tests pass on each (L0.6's 874 and the three new), 3 ignored and run
  by name; certify alone 67; the GUI's 117; zero warnings. The gate hash is `0x61f9c8529131ff17`,
  appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`. At `452c0e7` (L0.8), `scripts/gate.sh`
  green again on both with a clean stamp (97 s and 99 s, warm), 877 workspace tests each. The
  six horses and seven markets tapes give P2.2's 2,000-tick hash streams
  (`D:/rustyecon-p2l/fix-report/tapes-bit/tapes-bit.log`).

**The stocks probe (P2.2a; 2026-09-28; P2.2.1–P2.2.4).** You asked on 2026-09-27 for a
production chain where goods use other goods, and delegated the design; GOODS-CHAIN §6 put a
registered probe of machine stocks without loops before its first demo stage (next step 6). It
ran on branch `phase2-goods` from `reboot` at `92ba68e` (worktree `D:/rustyecon-wt/p2g`, scratch
`D:/rustyecon-p2g/`): a frame (`frame/HORSES-SPEC.md`, sha256 `0e23e809…`, with its mirror and
70 solves of 1g's `ChainEconomy`), the build (P2.2.1, `d636b76`: three agent kinds,
`probe::horses`, six tapes; [HORSES-RULES.md](docs/probe/HORSES-RULES.md)), the registration
before any mode-B run (P2.2.2, `run/registration.md`, sha256 `2baaa34d…`), the run, two reviews
and the report (P2.2.3, `c361b9e`, tables and plots; P2.2.4, `98534fe`).
- **What it is.** Stage v2a.1, GOODS-CHAIN's rule A: Appendix B's county with its flow machine
  made a horse, bred by a maker (M2) from its own horse-days, labour and pasture, and held, worn
  at δ, fed and hired out by the day by a wet capacity desk (M3) ordering
  max(δK\* + 2δ(K\* − H), 0); fodder a traded good made from land alone, so no loop; one category;
  ρ 0, J_b 1 tick, dials C2g. Core, markets and engine are unchanged; three `ActorState` variants
  are appended, so every pinned hash stays.
- **Verdict** ([docs/probe/HORSES.md](docs/probe/HORSES.md)). **GO for v2a.1** at H1–H4 (δ 10%
  and 8% a year, ω ½ and 1), C2g, 52 ticks a year, as registered: all 1,980 scored mode-B runs at
  52 a year (H1–H4 and ten families) converge with their kick sets, within 2.4e-13 in log, the
  class the mirror's in every run; the stocks layer off is P2.1's I0 bit for bit, by
  construction. Beside it: 12 a year fails everywhere and 24 a year at ω 1 (H4 unpredicted); a
  tenfold glut of horses diverges at every instance through the horse price's unfloored fall,
  P8's idle runaway reached at A0; P7 converges where STUCK was registered.
- **The reviews** (measurement; fidelity) confirm the verdict and narrow it. The run's count of
  2,548 is 1,980. O14 like for like, against the flow county at C2g with ex-post assignment,
  gives stocks about a quarter of the claimed softening of a cost shock's trough, and 2.3–2.8
  times the years, not 6–7. At ω 1 the GO passes through horse-price falls of 96–99.7%. P1's kick
  rates at ω 1 and P2's miss on the fast side, the slowest mode being wear, 1 − δ. M6's load
  check had a hole (O48, closed at L0.1), and v2a.1 is a partial base for v2a.1b (O49).
- **What follows** (decisions 234–239; O41–O50): stage v2a.1 may enter the demo's second pass on
  funded counties, after your ruling on D-G12; before P2.2b's frame come a remedy for the idle
  machine market (O47), the mirror's loop step with CAPACITY's plant composed with M3's horse
  holding, a funded rule-B county (O41) and M6 narrowed (O48, done at L0.1); Phase 2 proper
  opens as planned on loop-free instances, and its goods-chain 1750-like instance waits for O47.
- **The gates at `98534fe`** (P2.2.4's report; this record changes docs only), logs in
  `D:/rustyecon-p2g/report/gate/`. `scripts/gate.sh` is green in WSL
  (`CARGO_TARGET_DIR=/root/scratch/target-p2g-build`, the build step's target, warm, 122 s: WSL has
  about 1.5 GB free) and on Windows under Git Bash (`D:/rustyecon-targets/p2g-report`, fresh,
  224 s): 865 workspace tests pass on each, 3 ignored and run by name; certify alone,
  Parquet-free, 67 with 1 ignored; zero warnings; the gate hash is `0x61f9c8529131ff17` and the
  stamp names `98534fe`, clean; both certificates recompute; the probe's pin
  (`probe_battery_csv_unchanged`) and the markets probe's (`markets_i0_nests_appb`,
  `markets_tapes_are_their_generators_output`) hold; `demo_runs_to_1901` ends at
  `0xfad880fe08d06645`, stream `0xdb63cc96f769fb3e`; telemetry is identical from two processes.
  `scripts/gui.sh` is green in WSL (`/root/scratch/target-p2g-build-gui`, 129 s) and on Windows
  (`D:/rustyecon-targets/p2g-report-gui`, fresh, 198 s): 117 tests with 4 ignored, and the cli's
  hashes equal the GUI's for gate, appb (`0xe1fa082b26995867`), demo-gb and the two branch tapes.
  Recorded, not gated (`D:/rustyecon-p2g/report/hashes/`): the per-tick hashes of the seven
  markets tapes and the six horses tapes over 2,000 ticks are byte-identical on WSL and Windows
  and equal the build step's, ending as MARKETS-RULES §6.8 and HORSES-RULES §6.9 record.

**`oracle-goods` and `g1` are merged** (2026-09-28, `43ad8c5`, on branch `merge-og-g1` from
`reboot` at `16eb728`). Track 1g (P1g.1–P1g.7, `1a70eab` to `3f3a1bc`, on `oracle-goods`) and
G1 (G1.1–G1.11, `d8b8a32` to `3d1ad6f`, on `g1`) both started at `16eb728`; `oracle-goods` came
in by a fast-forward, and `g1` by this merge. No code conflicted: track 1g changed only
`crates/oracle`, README.md and this file, and G1 changed `crates/gui`, the engine's re-exports
and its frontend guard, `scripts/gui.sh`, docs/GUI.md, docs/ENGINE.md, the workspace's manifest
and lockfile, README.md and this file, never `crates/oracle`. Two files were joined by hand,
each side's content kept whole:
- README.md: the status paragraph names unit 1g on `oracle-goods`, then G1 on `g1`, beside G0's
  close and the demo's map; the crate table's oracle row is track 1g's and its GUI row G1's;
  "Running the GUI" and the GUI gate's count of named tests are `g1`'s alone;
- this file: the header says both have landed; below this block come 1g's record, then G1's two
  (its verification's fixes, then the build), then `phase1`'s merge and what came before, as
  they were. The lists (the decisions, the open calls and items, the next steps, the file map and
  the repro notes) are joined, and the open calls carry each hand check's command.

The two tracks numbered apart, as each copy of this file said: 1g's decisions are 179–190 of its
range 179–199 and its open items O31–O35, G1's are 200–219 and O36–O40. No number is used twice,
and nothing is renumbered. The lab builds the oracle's parameter types for 1a–1f as struct
literals and reads their points from the oracle's `Debug` (decisions 202 and 203, O38); track 1g
added `goods.rs` and `plants.rs` and changed only `MachineBlock::new`'s validation (D-G10), which
accepts every economy 1c accepted with every result bit for bit, and no type or field the lab
builds or reads. So the lab builds unchanged, and `lab_presets_solve_to_their_goldens` and every
other test of G1's passes on the merge. Unit 1g in the lab is not a trivial addition (O38,
amended at this merge), so it is left to G1's second part. The re-check of G1.11's fixes ran
after `g1`'s last commit and is recorded here, in O37, since `g1` did not record it.

The gates on the committed merge, with a clean build stamp (logs in `D:/rustyecon-merge-og-g1/`):
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-merge-og-g1`,
  202 s) and on Windows under Git Bash (`D:/rustyecon-targets/merge-og-g1`, 206 s), the two run
  at once on fresh targets. 846 tests pass in the workspace on each machine, with 3 ignored and
  run by name: the `phase1` merge's 795, 1g's 50 and G1's engine doc test. Certify alone,
  Parquet-free, passes 67 with 1 ignored; zero warnings. The gate hash is `0x61f9c8529131ff17`,
  and the stamp names the merge's code, clean, on both (`320619d`, the merge before this record
  was added; only this file differs). Both certificates PASS and recompute byte-equal, the
  probe's pins hold (`probe_battery_csv_unchanged` by name, and the markets probe's
  `markets_tapes_are_their_generators_output` among the workspace's tests), `derive.py --check`
  passes, `demo_runs_to_1901` ends at `0xfad880fe08d06645` with hash stream
  `0xdb63cc96f769fb3e`, and telemetry is identical from two processes. The GUI's non-blocking
  check passes in WSL (skipped on Windows), as do the wasm32 checks of the engine and certify.
- **`scripts/gui.sh`** is green in WSL (the same target, 136 s) and on Windows under Git Bash
  (159 s): 117 tests pass with 4 ignored measurements, the 87 named ones by name, among them the
  lab's (`lab_presets_solve_to_their_goldens`, `a_points_fields_are_the_oracles_doubles`,
  `every_knob_is_the_field_its_path_names`, `the_lab_shows_appendix_b_bit_for_bit`); fmt and
  clippy clean with `-D warnings`; in WSL the sweep measurement by name, a median of 0.727 ms of
  20 (0.680 to 0.787 ms). Five hash diffs are equal, the same on both machines: gate (2,080
  ticks, `0x61f9c8529131ff17`), appb (20,000, `0xe1fa082b26995867`), demo-gb (7,852,
  `0xfad880fe08d06645`), and the two branch tapes as at G0.3 (`branch`, final
  `0x9fc2f964a8510756`; `removal`, `0xd057e3ea708da495`).
- **The oracle's generators:** all seven pass `--check` under laborformal's venv on Windows, run
  beside the gates (`generate.py` 1 s, `generate_1b.py` 1 s, `generate_1c.py` 12 s,
  `generate_1d.py` 32 s, `generate_1e.py` 39 s, `generate_1f.py` 22 s, `generate_1g.py` 25 s).
- **Recorded, not gated:** the cli's per-tick hashes of gate (2,080), appb (20,000) and demo-gb
  (7,852) are byte-identical on WSL and Windows but for the header line naming the target
  (`D:/rustyecon-merge-og-g1/hashes/`).

**Unit 1g, machines as goods, is built** (2026-09-27, branch `oracle-goods`, worktree
`D:/rustyecon-wt/og`; [crates/oracle/docs/unit-1g.md](crates/oracle/docs/unit-1g.md)). You asked
on 2026-09-27 that goods use other goods rather than abstract machine services; GOODS-CHAIN §2 made
the oracle's part an addendum to unit 1c, and D-G1 rewords decision 67: machines are durable goods
built from and run on goods, never from a category (E1). The unit was built as Phase 1's were but
for the re-check (O35): a spec checked against the generator's draft at 70 digits, then a build
whose goldens come from the committed generator, with the build's own mutation check, then the
bounded verification (1g-r1) and one fix round (P1g.6). Only `crates/oracle` changed, besides this
file and the root README.

| Commit | What landed |
|---|---|
| `1a70eab` P1g.1 | the spec, docs/unit-1g.md, its open questions proposed as decisions 179-190 |
| `af68f3c` P1g.2 | D-G10 in `machine_block.rs`; `src/goods.rs`, the mapping; `src/plants.rs`, plants as machine types; `generate_1g.py` and `goldens_1g.txt`; the gate groups h1-h7 and h9; `m7::validation`'s two rows on A^op + A^I amended, and a note in unit-1c.md |
| `f054556` P1g.3 | O28: eight tests for mutants the re-checks of 1d-1f left (h8) |
| `8cce6b1` P1g.4 | h8's worker-type test asserts its reason, after the build's mutation check found its mutant alive |
| `85453d8` P1g.5 | the spec's §12, the oracle's README, this file, the root README's status line |
| `623d1a8` P1g.6 | the verification's fixes: a test for each of its five surviving mutants (h1 1, h2 1, h6 2 and P2's steps), the goods made worded as the good's gross output in the spec and the goldens' notes; no source file changed |
| P1g.7 | this file's record of the round, the root README's status line |

- **D-G10** (decision 180, amending 71). 1c asked that I − (A^op + A^I) be a nonsingular M-matrix:
  a machine buildable from one period of its own chain's services. At weekly ticks that refuses
  A0 (a^I = a/δ = 148, radius 148) and CHAIN's horse (radius 1.50 on its chain), though their
  per-period matrices A^q = A^op + Δ·A^I are productive (0.3; 0.24). 1g checks A^q, which 1c
  already factored and refused on a nonpositive pivot, and the chain to land as a pattern by
  reachability, so no rounding refuses a chain that reaches land. Every economy 1c accepted is
  accepted and nothing computed after validation changed, so every result is bit for bit: the
  1a-1f gate passes unchanged but for `m7::validation`, whose two rows on A^op + A^I now read per
  period (0.6 + 4.1 refused, 0.6 + 0.6 valid). Of 3000 random blocks with δ down to 1e-3, 498
  are accepted by both rules, 470 by D-G10 only (279 of them interior, every 1c identity holding)
  and none by 1c only.
- **The mapping** (decisions 181-183, 186). `GoodsChain` (categories; materials; machines, each a
  good with a build recipe per unit of stock, its hours, κ hours a period per unit, θ, an
  operating recipe per hour, δ and J, all per period) maps to unit 1c: materials and machine goods
  are flow types, each machine's hours a type built from 1/κ of its good. E1 and E2 (a category
  using a material or hours directly, G1) are refused, and every other error is named by its good.
  `ChainEconomy` solves it and reads every good's price and output back, with each machine's
  stock, goods made, hour price, O, V, hours, wealth and interest.
- **Plants** (decisions 184-185). `PlantEconomy` rewrites a flow type to CAPACITY's long run:
  operating ζ times its bundle, build κ times the plant's recipe, the plant's δ and J, at any ρ.
  A plant of s bundles of its own recipe is closed form, and at s1 and ρ 0 its long run is the
  flow economy (P1 is L2's within 1e-13); any other recipe is a fixed point in half steps of
  ln(κ/ζ) to 1e-13 (P2, labour and land: 42 steps, v 0.13869, as CAPACITY's check 2c found);
  θ = 1 is the type bit for bit.
- **Goldens**: `generate_1g.py` writes 210 at 70 digits in about 26 s, importing generate_1c.py.
  Each chain is solved directly, by its embedding and by its fold, agreeing within 9.9e-71, and
  against the earlier unit where it is one of its economies: S1 = 1c's M3, S1 at ρ 0 = M3z, A0 at
  ρ 0 = 1a's Appendix B, A0 in the fork economy = 1b's C3, P1 = L2's flow economy. The instances:
  S1, S1Z, S2 and S2H (ORACLE-GOODS §1.6: the switch at x 0.36148547771379892814, x*
  0.66836997221771948458), A0 at 52 ticks a year (ρ 0 and 5% a year, and in the fork economy),
  CHAIN's horse at weekly periods (J 156, on a constructed county), and L2 with plants (P1, P1S,
  P1R, P2). Through 1d's, 1e's and 1f's forms every 1g economy is 1c's bit for bit (h7).
- **Largest errors**: every golden within 4.5e-16 relative, but S1Z's 1 − x* against M3z's golden
  (2.9e-15) and P2, the fixed point, within 9.1e-14 (its ratio).
- **Tests**: 50 (4 unit, 46 gate in h1-h9, h8's 8 for O28; 46 and 42 before P1g.6); the oracle
  has 488 (88 unit, 399 gate, 1 doc) and the workspace 845 (3 ignored and run by name).
- **The mutation check** (the build's own, `D:/rustyecon-og/mut/`): 37 mutants of D-G10 (8), the
  mapping (13) and the plants (16), each applied alone with the package's tests run in release: 36
  killed. The survivor adds an input's coefficient where the mapping sets it, equivalent since an
  input named twice is refused before. The damping of the fixed point is guarded by a test that each
  step halves the move, and its tolerance by P2's goldens (docs/unit-1g.md §12 item 7).
- **The verification** (1g-r1, `D:/rustyecon-verify/1g-r1-derive/`; P1g.6, docs/unit-1g.md §12
  item 11). Its derivation, a chain solver of its own that does not read the crate, agreed with
  the oracle on 444 interior random chains within 1.1e-13 relative (N_a; x* within 3.9e-15, the
  goods made within 1.5e-14) and on 1,258 boundary regimes, and found no wrong result. Of its 49
  mutants 44 were killed. The five survivors were four majors, each now killed by a test:
  - either half of D-G10's productivity check dropped (D1, D10): at the edge of productivity the
    row-order factorisation of I − A^q and its transpose round differently.
    `h1::both_factorisations_guard_productivity` takes two blocks from the verification's float
    search, each passing one factorisation and not the other, and both must be "not productive";
  - the goods made read as δX/κ (G15), the same unless another recipe uses the good, which no
    instance did. `h2::a_machine_good_used_by_another_recipe` builds an engine into a mill
    (ORACLE-GOODS §3.2(c)): the engines made are 1.14641 a period, their own δX/κ 1.04672 and
    the mills' use 8.7%, the identity within 1.9e-16;
  - a plant's J dropped in the long run (P21): every plant test had J 1.
    `h6::a_plant_with_a_build_lag` takes P1R's plants at J 2 and checks the long run's J,
    u = (ρ + δ)(1 + ρ), ω = (1 + ρ) + δ, O = θp and uV = (1 − θ)p; the mutant's capital share
    was 0.19985;
  - the fixed point started from ratios of 1 (P15), which took P2 47 steps, not 42, within the
    tolerance. `h6::the_fixed_point_starts_at_the_unplanted_prices` replays step 1 from the
    unplanted equilibrium and matches `solve_within(1)`'s gap bit for bit, and P2's 42 steps are
    asserted.
  The two minors: the spec and the goldens' notes said the goods made are δX/κ without the
  condition; they now say the good's gross output, δX/κ when nothing else uses it (the generator
  asserts it where it says so, and no value changed); and this file's map listed P1g.1-P1g.3.
  The five mutants, each run again alone against P1g.6's tests (the package's tests in release),
  are killed, and so is a sixth, the fixed point started 1% off in ln r
  (`D:/rustyecon-og/mut/mutate_fix.py`, `mutate_fix.out`). No decision is new; the re-check of
  these items passed (O35, closed).
- **O28** (h8): eight tests: the 1d re-check's four probe tests with the assertions its probes
  printed; 1e's wall's-end frame with space's land at 2 and 0.5; a CES economy with an intermediate
  input and required hours (1f); and two of 1d's first-pass survivors, an economy with no worker
  types (whose test now asserts the reason, P1g.4: without its own check the economy is still
  refused, by the next check, with the wrong reason) and the jump schedule refused as
  `LaborNotCleared`. Of the ten O28 mutants run on this tree nine are killed, and Lemma B.1's flag
  without its shortage check survives, equivalent (a short point's P_s is NaN). What is left is O33.
- **The gates on `623d1a8`** (P1g.6), with a clean build stamp (logs in
  `D:/rustyecon-og/gate/fix/`; `8cce6b1`'s in `gate/final/`): `scripts/gate.sh` is green in WSL
  (`CARGO_TARGET_DIR=/root/scratch/target-og`, 91 s, warm) and on Windows under Git Bash
  (`D:/rustyecon-targets/og-gate`, 92 s, warm): 845 tests pass in the workspace on each machine, 3
  ignored and run by name, which is the `phase1` merge's 795 and 1g's 50; certify alone,
  Parquet-free, passes 67 with 1 ignored; zero warnings. The gate hash is `0x61f9c8529131ff17`,
  the stamp names `623d1a8`, clean, on both; both certificates PASS and recompute byte-equal, the
  probe's pins hold, `derive.py --check` passes, `demo_runs_to_1901` passes, and telemetry is
  identical from two processes. The GUI's non-blocking check and the wasm32 checks of the engine
  and certify pass in WSL (skipped on Windows). `scripts/gui.sh` is green in WSL (82 s) and on
  Windows (87 s): 84 tests pass with 2 ignored, and the five hash diffs are equal on both
  machines: gate (2,080 ticks, `0x61f9c8529131ff17`), appb (20,000, `0xe1fa082b26995867`),
  demo-gb (7,852, `0xfad880fe08d06645`), `branch` (`0x9fc2f964a8510756`) and `removal`
  (`0xd057e3ea708da495`). The same held on `8cce6b1` with 841 tests. No file outside
  `crates/oracle` changed but this file and the root README.
- **The generators**: all seven pass `--check` under laborformal's venv on Windows
  (`generate.py` 1 s, `generate_1b.py` 2 s, `generate_1c.py` 12 s, `generate_1d.py` 14 s,
  `generate_1e.py` 41 s, `generate_1f.py` 23 s, `generate_1g.py` 24 s); after P1g.6, whose
  change to `generate_1g.py` is its notes on the goods made and an assertion, `generate_1g.py
  --check` passes again (25 s), and it imports nothing that changed.

**G1's bounded verification ran, and its findings are fixed** (2026-09-27, G1.11 on `g1`;
GUI.md, the block "Amended after G1's verification"; decisions 200, 204, 208, 210, 214–216
and 218 amended; O36, O37 and O39 amended). One adversarial pass (`D:/rustyecon-verify/g1-r1/`,
on a clone of `92048db`) ran 44 mutants, two scan probes and four gate variants. It found no wrong
number: every paired output of the 16 presets agreed with its golden to at most 8.5e-16
relative, and the explainer equalled `next_price` at all 12,480 (tick, market) pairs of each
variant. It found six major issues and four minor, and each is fixed by a test that fails
without its fix:
- *The lab's golden checks* (its mutants L3–L6 and L8 had survived; the letters are the
  verifier's, in its `mutations.txt`): `vm::lab::build_beside` takes goldens a test
  doctored; G1's, doctored either side of each bar, agree or not by the bar with the exact
  difference, and the table drawn alone paints each disagreement in the error colour.
- *The explainer off the gate's easy path* (E1, E2, E4–E7): four gate variants (bread's rate
  doubled on 1755-01-01, bread's price scaled by 1.5 on 1756-01-01, `Ratio`, `Saturate`), the
  explainer equal to the engine at all 12,480 pairs of each; the waterfall flags `rate.up` and
  `bread.shock` alone, its bins' residuals add up, and bread's residual is the holds, or the
  holds and ln 1.5, within 1e-9; one bit of a recorded next price flipped is said to differ and
  painted "DIFFERS".
- *What is painted* (L9–L12, E3, E8, P4): `ui::charts` records every line, mark and bar the
  lab's field over x and its sweep and the waterfall lend egui, with the plot's transform; the
  scripts hold each vertex, mark and bar to its view-model bit for bit and each painted path
  and bar to what was lent; the lab script paints `BoundaryNoMargin` (G1 at N 0.4)
  with its f(1); the explainer script reads p, S, D, x, k and k·x back from the screen; the watch
  script paints the change with its sign.
- *The knobs* (X2): every preset's knobs equal its parameter type's `Debug` path by path and bit
  for bit, and each of 552 knobs set to a value of its own moves its own path alone.
- *The scans*: `extern crate self as g` (and of this crate or std by name) is a root, and a
  `path` attribute and `include!` are refused in the egui-free and pure modules; the core scan
  reads the manifest for `package = "rustyecon-core"` and the lockfile's dependencies of
  `rustyecon-gui`, so a renamed core (N1) is refused wherever the rename is made.
- *The minors*: decision 209's event before a date (P2); the credit's clamp (A5), a unit test at
  every width from 60 to 1,000 points, where at 169 the canvas's edge decides its x; under
  `Ratio` the waterfall sums ln(D/S), the rule's own steps (bread's residual was 2.71 of 2.91,
  now under 1e-9); `session.ron` format 3 must have `watch` and `log_axes`, and format 2 may have
  neither nor an event or date breakpoint; the docs (decisions 214, 215, 218, O37, O39, ENGINE's
  G1.1 item 3). The toolbar's test opens the demo world at 1,600 and 1,024 too, where the chip
  note was made: one or two lines, so that note is closed.

**Mutation** (`D:/rustyecon-g1/fix-r1/`, `mutate.py`, a clone with the fixes, the whole suite in
release per mutant, split between WSL and Windows): 35 mutants, the verifier's 22 survivors
re-aimed where the fix moved their code and 13 of the fixes' own (a mark or a line painted off
its record, a bar recorded negated, the level line scaled, bars unstacked, a parcel knob bound
to another field, the verifier's `extern crate self` and `#[path]` probe, each scan fix undone,
core renamed in the workspace's table, `Ratio` read as `Imbalance`, each session check undone):
all killed, each by a test G1.11 added or extended (`mutations.txt`; the probe's first build
did not compile, and one that does was refused by both scans). Logs in `logs-wsl/` and
`logs-win/`.

**Tests.** The GUI's suite grows from 110 (106 run, 4 ignored) to 121 (117 run, 4 ignored):
three in `tests/lab.rs`, four in `tests/pricestep.rs`, two scripts in `tests/app.rs`, one in
`tests/watch.rs` and a unit test in `ui/map.rs`, with the lab, explainer, watch and toolbar
scripts, the persist test and the scans' fixtures extended. `scripts/gui.sh` names 87, up from
75.

**The gates**, on the committed tree at G1.11 (`2fdc7a5`), logs in `D:/rustyecon-g1/gates/`:
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-g1`, warm,
  90 s) and on Windows under Git Bash (`D:/rustyecon-targets/g1`, warm, 91 s), the two run at
  once: 796 tests pass in the workspace on each, as at G1.9, with 3 ignored and run by name;
  zero warnings; the gate hash `0x61f9c8529131ff17`, the stamp naming `2fdc7a5` clean; both
  certificates PASS and recompute byte-equal, the probe's pins hold, `demo_runs_to_1901` ends
  at `0xfad880fe08d06645` with hash stream `0xdb63cc96f769fb3e`, and telemetry is identical
  from two processes.
- **`scripts/gui.sh`** is green in WSL (90 s) and on Windows (99 s), the two run at once: 117
  tests pass with 4 ignored, the 87 named ones by name, fmt and clippy clean with `-D
  warnings`; in WSL the sweep measurement by name, a median of 0.688 ms of 20 (0.625 to 0.756
  ms); five hash diffs equal on both machines: gate (2,080 ticks, `0x61f9c8529131ff17`), appb
  (20,000, `0xe1fa082b26995867`), demo-gb (7,852, `0xfad880fe08d06645`), and the two branch
  tapes (`0x9fc2f964a8510756`, `0xd057e3ea708da495`).

**Pending:** the window's p90 by hand, yours (below); a re-check of these fixes, and what the
verification did not reach (O37); what G0 moved to G1 (O36).

**G1, the oracle lab, is built** (2026-09-27, G1.1–G1.10, branch `g1` from `reboot` at
`16eb728`, worktree `D:/rustyecon-wt/g1`; [docs/GUI.md](docs/GUI.md), the block "Amended at
G1", and §9; ENGINE, amended at G1.1; decisions 200–219, O36–O40). G0 and Phase 1's gate were
both met, so §9's G1 was unblocked; it was built as §9 lists it, with STATE's next step 4 (the
engine's re-exports), O26's map items and two of O20's. Nothing on the engine path changed
behaviour: no hash, `world_id` or `prefix_id` moved.

| Commit | What landed |
|---|---|
| `d8b8a32` G1.1 | the engine re-exports `num` and the tape's raw schema (`raw`), with `Basis` and `Unit` in its prelude; its frontend guard allows exactly those two modules; the GUI drops `rustyecon-core` |
| `f539365` G1.2 | the oracle lab: `lab/` (instances of units 1a–1f, 16 golden presets, knobs, the bundled goldens, a point's fields from the oracle's `Debug`), `vm/lab.rs` and the Lab tab |
| `ee69769` G1.3 | the price-step explainer and the log waterfall, `vm/pricestep.rs`, in the market inspector |
| `01a7af5` G1.4 | event and date breakpoints; the watchlist; log axes; `session.ron` format 3 |
| `ff7fd50` G1.5 | PNG snapshots of the window, never citable |
| `64383f3` G1.6 | O26's map items: the credit wraps; the eight surviving mutants killed |
| `27bda7f` G1.7 | O20's two ways past the no-egui scan closed |
| `c1658ce` G1.8 | `scripts/gui.sh` names 74 tests and runs G1's sweep measurement by name |
| `a902bb3` G1.9 | the toolbar keeps its chips whole (the gates' first run on G1.8 failed the branch script on it); every preset's sweep time recorded |
| G1.10 | this file, GUI.md's block and §9, the README |

**What it is.**
- **The lab** (the Lab tab, beside the plots and the map; it needs no tape and drives no run).
  Pick a unit and one of its presets: 16 instances the goldens were computed on, two to four a
  unit (1a: G1, G3 η 0.3, G5 flow, G5 durable; 1b: C3, C7 gap; 1c: M3, M4; 1d: B1, E1; 1e: K1,
  Q1; 1f: TX, GB, C1), each built as the oracle's gate tests build it. Every number of the
  instance is a knob by its path. The oracle solves it through the unit's own `new` and
  `solve`, and every output its `outputs()` lists is shown as the dump prints it, the float's
  shortest round-trip digits, beside the golden of the same name (1e-12 relative) and the
  paper's published value (5e-6 absolute); an edited instance shows no golden. At G1 the screen
  shows x\* `0.863150418162437`, v `0.5434359606967785`, Y `7.880605524972908` and N_a
  `1.3433818800977175`, the oracle's doubles, beside 0.86315, 0.54344, 7.88061 and 1.34338.
  Every preset's paired outputs agree with their goldens (`lab_presets_solve_to_their_goldens`:
  G1 pairs 29 outputs, M3 15, TX 9). Any field of the unit's point plots over x in [0, 1], f
  by default, with x\* and the bracket [1e-12, 1] marked; a knob sweeps over a range, 200
  points by default.
- **"Why is this price 12.3?"** The market inspector recomputes the tick's step with markets'
  own `imbalance` and `next_price` on the recorded p, S, D and the rate's per-tick value, and
  says whether it equals the run's next price bit for bit; below it, the log waterfall of
  ln(p/p₀) as Σ k·x a year at a time, the residual (the gate holds one-sided markets, so its
  residual is those holds) and the events.
- **The panels.** Breakpoints on an event by key (every occurrence) or on a date, from the
  log's field or the event's inspector; a run pauses after the tick, and the log names the
  breakpoint. A watchlist at the top of the outliner. A "log scale" toggle on each plot panel.
  `session.ron` format 3 keeps them; format 2 still reads.
- **Snapshots.** The toolbar's Snapshot writes a PNG of the window to `snapshots/` beside the
  session, with a banner across its top and text chunks that say NEVER CITABLE, the build, the
  run and the lab's preset.

**Tests.** The GUI's suite grows from 86 (84 run, 2 ignored) to 110 (106 run, 4 ignored): new
files `tests/lab.rs` (6, and 2 ignored measurements), `tests/pricestep.rs` (3), `tests/watch.rs`
(3) and `tests/snapshot.rs` (2); kittest scripts of the lab, the explainer, the watchlist and
breakpoints and the snapshot, and checks of the toolbar at four widths and of the credit on a
narrow canvas; the lab's scan; unit tests of the `Debug` reader and the charts' colours; and
extensions of `gui_equals_cli` (event and date breakpoints),
`every_drawn_vertex_is_recorded` (a log panel), the goldens (each market's explainer, equal bit
for bit), `lens_values_equal_the_engine` (every county's card), the demo script (the cursor
behind live), `rebuilt_mesh_colours_are_the_lens_colours` (every painted vertex and legend
segment) and the two O20 fixtures. The engine's frontend guard gains five fixtures and a doc
test. `scripts/gui.sh` names 75, up from 55.

**The toolbar.** The gates' first run, on G1.8, failed `the_branch_script_…` on both machines:
the Snapshot button made the toolbar's first row wider, the health chip began a row with 16
points left, and its text wrapped into a column one word wide and 1,150 points tall, which
pushed every tile below the window. G1.9 starts a chip on a new row whenever less than 360
points of its row are left; `the_toolbar_keeps_its_chips_whole` and the branch script fail
without it. This is the toolbar half of O26's note that the chip wraps on a narrow window.

**G1's gate** (GUI.md §9):
- **The lab shows x\* 0.86315, v 0.54344, Y 7.88061 and N_a 1.34338, bit for bit the oracle's
  outputs:** met. `the_lab_shows_appendix_b_bit_for_bit` parses the view's text back to the
  oracle's doubles, called directly; `the_lab_script_shows_appendix_b_and_its_goldens` finds the
  same texts painted, with the published values and the 30-digit goldens beside them.
- **The explainer equals `next_price` on every gate tick:** met.
  `the_explainer_equals_next_price_on_every_gate_tick` holds the explainer's next price to a
  `Sim`'s own report bit for bit at all 12,480 (tick, market) pairs of the gate, and appb's
  80,000 beside it.
- **A 200-point sweep's view-model builds in under 16 ms on WSL (median of 20):** met,
  `a_200_point_sweep_builds_in_under_16_ms`, ignored and run by name by `scripts/gui.sh`: G1
  over N from 2 to 8, reading x\*, v, Y and N_a: a median of 0.675 ms of 20 (0.632 to 0.733
  ms) in the gate's run, and 0.75 ms alone. On Windows, not gated, 0.82 ms.
- **The window's p90 frame stays under 16 ms, checked by hand:** PENDING, yours (below).
- U1–U12 and the scans hold: `scripts/gui.sh` is green on both machines (below).

**The gates**, on the committed tree at G1.9 (`a902bb3`; G1.10 changes docs only), with logs in
`D:/rustyecon-g1/gates/`:
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-g1`, warm, 90 s)
  and on Windows under Git Bash (`D:/rustyecon-targets/g1`, warm, 94 s), the two run at once.
  796 tests pass in the workspace on each machine, with 3 ignored and run by name: the merge's
  795 and the engine's new doc test. Certify alone, Parquet-free, passes 67 with 1 ignored;
  zero warnings. The gate hash is `0x61f9c8529131ff17`, and the stamp names `a902bb3`, clean,
  on both. Both certificates PASS and recompute byte-equal, the probe's pins hold,
  `demo_runs_to_1901` ends at `0xfad880fe08d06645` with hash stream `0xdb63cc96f769fb3e`, and
  telemetry is identical from two processes. The GUI's non-blocking check passes in WSL
  (skipped on Windows), as do the wasm32 checks of the engine and certify.
- **`scripts/gui.sh`** is green in WSL (85 s) and on Windows under Git Bash (95 s): 106 tests
  pass with 4 ignored measurements, the 75 named ones by name, fmt and clippy clean with `-D
  warnings`, and in WSL the sweep measurement by name. Five hash diffs are equal, the same on
  both machines: gate (2,080 ticks, `0x61f9c8529131ff17`), appb (20,000,
  `0xe1fa082b26995867`), demo-gb (7,852, `0xfad880fe08d06645`), and the two branch tapes as at
  G0.3 (`branch`, final `0x9fc2f964a8510756`; `removal`, `0xd057e3ea708da495`).
- **The first run, on G1.8, was red on both machines:** the branch script, the toolbar's chip
  (above); G1.9 fixed it, and the second run is the one recorded.
- **Recorded, not gated:** the cli's per-tick hashes of gate (2,080), appb (20,000) and demo-gb
  (7,852) are byte-identical on WSL and Windows (`D:/rustyecon-g1/hashes/`).
- **The smoke mode on Windows**, `rustyecon-gui --smoke 2080 tapes/gate.ron` (release; the
  outliner, plots, inspector and timeline drawn), twice: 479 frames running, CPU per frame p50
  0.93 and 0.99 ms, p90 1.22 and 1.78 ms, max 35.5 and 33.4 ms (the first frame); paused, p90
  1.15 and 2.03 ms. As at G0.3's close (p90 1.62 ms running). The window opened and closed by
  itself; nobody looked at it.

**Measured, recorded and not gated** (release, alone on the machine): a 200-point sweep of each
preset's first real knob, ±10% about its value, median of 5
(`every_presets_sweep_time_is_recorded`). On WSL: 1a 0.7–1.6 ms, 1b 3.3–3.7 ms, 1c 6.3–11.4
ms, 1d 10.4–11.8 ms, 1e 172–185 ms (its paths are scanned, `EXIT_SCAN`), 1f 11.8–14.5 ms. On
Windows: 1a 0.8–1.7 ms, 1b 5.7–5.8 ms, 1c 10.7–16.7 ms, 1d 17.8–18.3 ms, 1e 294–328 ms, 1f
18.2–28.5 ms. Every point interior. So a 1e sweep holds the window for about a third of a
second on Windows (O40). Logs in `D:/rustyecon-g1/gates/`.

**Checked by hand: PENDING, yours** (G1's gate: the window's p90 frame under 16 ms). On
Windows, from the repository once this merge is in `reboot` (or from `merge-og-g1`), in
PowerShell:

```powershell
$env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/g1-hand'
cargo run --release -p rustyecon-gui -- tapes/gate.ron
```

1. Open the Lab tab (beside Plots and Map): G1 is solved, the outputs beside their goldens.
   Pick another unit and preset; edit a knob in "Instance" and press Enter; press "Run the
   sweep". Watch the frame stay smooth; the smoke mode's figures are above for comparison.
2. Press Space to run the gate, pause, select town/bread in the outliner: the inspector's "Why
   this price" and the waterfall. Tick "log scale" on a plot panel.
3. In the Log pane, type `mine.cut` in "break at" and press "Add breakpoint", then run: the run
   pauses at 1760-03-01's tick and the log says why.
4. Press Snapshot: the banner shows, and a PNG appears in `%APPDATA%\rustyecon\gui\snapshots\`;
   its picture carries the banner. This is the one path no headless test can reach (a headless
   window has no renderer).

**What G1 leaves** (O36–O40): what G0 moved to G1 and G1 did not build (overlay, difference and
ratio against a parent; re-making branches at launch; the registry's and the inspector's ways
into the editor; a lock on `session.ron`) and the rest of O20; a re-check of G1.11's fixes of
its verification (above), and what that did not reach; the lab's two copies (presets
transcribed from the oracle's gate tests, and goldens paired by name), which the next change to
the oracle's parameter types must carry; O26's clock and two layout notes; and the snapshot's
renderer path and the lab's heavier sweeps on the UI thread.

**`phase1` is merged** (2026-09-27, `16eb728`). Phase 1's units 1d–1f and its close
(P1.8–P1.14, `a2a9b93` to `78d6edc`, on `phase1`) and `reboot`'s line since `503897e` (the `g0`
merge, the many-markets probe and the demo world, to `2398b6a`) both started at `503897e`, and no
code conflicted: `phase1` changed only `crates/oracle`, README.md and STATE.md, and `reboot`
never touched `crates/oracle`. Two files were joined by hand, each side's content kept whole:
- README.md: the status paragraph says Phase 1 is closed, beside G0's close and the demo's map,
  and the crate table's oracle row is `phase1`'s, its other rows `reboot`'s;
- this file: Phase 1's record of units 1d–1f first below, after a note of the goods chain's
  design evidence, then `reboot`'s records as they were, with 1b and 1c's heading as `phase1`
  wrote it. The lists (the decisions, the open calls and items, the next steps, the file map
  and the repro notes) are joined.

Phase 1's decisions move from n to n + 59, 76–119 to 135–178, and its open items O20–O22 to
O28–O30, with every citation of them: in this file; in crates/oracle/README.md; in
crates/oracle/docs/unit-1d.md, unit-1e.md and unit-1f.md, which cite them as proposed
decisions (their open questions and the records of each build and verification); and in five
comments of the oracle's code and gate tests (`households.rs`, `f1_nesting.rs`,
`f3_coverage.rs`, and `f9_random_households.rs` twice), which cite 174–176. Both lines had used
76–119 and O20–O22, for different things, so each citation was read in its context: `reboot`'s
76–134 and O20–O27 keep their numbers, and 59–75, which the two lines share, are unchanged. No
other code changed. `probe::markets` and `worldgen` call the oracle through 1a's and 1c's API,
which 1d–1f extended without changing it, and they build and pass as they were. The gates on
the committed merge, with a clean build stamp (logs in `D:/rustyecon-merge-p1/`):
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-merge-p1`, 93 s)
  and on Windows under Git Bash (`D:/rustyecon-targets/merge-p1`, 108 s), the two run at once,
  each target warm from a green run on the merge before it was committed (fresh targets, 206 s
  and 220 s). 795 tests pass in the workspace on each machine, with 3 ignored and run by name,
  which is the demo merge's 599 and P1.14's 745 less the 549 they shared at `503897e`; certify
  alone, Parquet-free, passes 67 with 1 ignored; zero warnings. The gate hash is
  `0x61f9c8529131ff17`, and the stamp names the merge's code, clean, on both (`ac8ed51`, the
  merge before this record was added; only this file differs). Both certificates PASS and
  recompute byte-equal, the probe's pins hold, `derive.py --check` passes, `demo_runs_to_1901`
  ends at `0xfad880fe08d06645` with hash stream `0xdb63cc96f769fb3e`, and telemetry is
  identical from two processes. The GUI's non-blocking check passes in WSL (skipped on
  Windows), as do the wasm32 checks of the engine and certify.
- **`scripts/gui.sh`** is green in WSL (the same target, 129 s) and on Windows under Git Bash
  (173 s): 84 tests pass with 2 ignored measurements, the 55 named ones by name, fmt and clippy
  clean with `-D warnings`, and five hash diffs equal, the same on both machines: gate (2,080
  ticks, final `0x61f9c8529131ff17`), appb (20,000, `0xe1fa082b26995867`), demo-gb (7,852,
  `0xfad880fe08d06645`), and the two branch tapes as at G0.3 (`branch`, final
  `0x9fc2f964a8510756`; `removal`, `0xd057e3ea708da495`).
- **The oracle:** its 438 tests pass in both gates, the code `78d6edc`'s but for the five
  renumbered comments, and all six generators pass `--check` under laborformal's venv on
  Windows (about 105 s together: `generate_1e.py` 48 s, `generate_1f.py` 29 s).
- **The markets probe's record is unchanged:** `crates/agents`, `crates/probe`,
  `tapes/markets-*.ron` and docs/probe/, with its six pinned CSVs in results/markets/, and every
  file outside `crates/oracle`, README.md and this file, are `2398b6a`'s byte for byte, and
  `markets_tapes_are_their_generators_output` passes on both machines.
- **Recorded, not gated:** the cli's per-tick hashes of gate (2,080), appb (20,000) and demo-gb
  (7,852) are byte-identical on WSL and Windows.

**The goods chain's design evidence, outside the repository** (2026-09-27; it changed no
repository or branch). Python mirrors of the many-markets probe, which the engine has not run,
on what "Next steps" 5 and 6 build:
- **`D:/rustyecon-goods/GOODS-CHAIN.md`**, at your request of 2026-09-27 that goods use other
  goods rather than abstract machine services. Machines become durable goods built from goods
  and run on goods, with the task margin unchanged. At rest the equilibrium is a unit 1c
  economy with more rows (its steam chain reproduces 1c's M3 to 70 digits, and the unchanged
  Rust oracle to 3.9e-16) once one check changes, D-G10: productivity on the per-period matrix,
  since 1c's check per machine built rejects, at weekly periods, a machine whose build embodies
  more than about a period of its own chain's services. Its §2 is the
  oracle addendum, timed for after `phase1` lands (it edits `machine_block.rs`), and its §7
  proposes decisions D-G1 (67 reworded) to D-G15, none ruled.
- **`D:/rustyecon-loops/LOOPS.md`**, on what damps the markets probe's loop (L2, L3; decision
  120, O21). Diminishing returns on each loop desk's own output does, but only when strong: at
  θ 0.7 every C2m battery run converges (77/77, 119/119), and the rest point then lies off the
  constant-returns oracle (7% in log at θ 0.8 after a doubled land coefficient). Graded land
  (1e's parcels) is far too weak, buffer stocks change the loop's clock and not its gain, and a
  planning rule removes the amplification but not the absorbing zero.
- **`D:/rustyecon-loops/capacity/CAPACITY.md`**, on a plant stock as the damper: each loop desk
  makes y = K^(1−θ)·z^θ with a plant K built from its own recipe, worn at δ and ordered by
  GOODS-CHAIN's rule (s_K = 2δ). At θ 0.8 and δ 10% a year it passes the goods chain's dials
  C2g on L2 and L3 (77/77, 119/119), and every converged run ends at the constant-returns
  oracle of its shocked instance (583 runs, within 5.3e-10 in log), so the long run is the
  registered instance and no new solve is needed at ρ = 0. Its costs: 15–98 years to recover
  from a cost shock, a user cost of 1 − θ of each loop price, and the absorbing zero kept. It
  recommends the plant as the goods chain's default loop damper, with LOOPS.md's drs θ 0.8 the
  registered alternative.

**Phase 1 is closed, and its gate is green in WSL and on Windows** (2026-09-27;
[crates/oracle/README.md](crates/oracle/README.md), with the specs
[unit-1d.md](crates/oracle/docs/unit-1d.md), [unit-1e.md](crates/oracle/docs/unit-1e.md) and
[unit-1f.md](crates/oracle/docs/unit-1f.md)). Units 1d, 1e and 1f were built as 1b and 1c were:
a spec from laborformal `31b3482` checked against a 70-digit mpmath prototype in scratch; a
build whose goldens come from a committed mpmath generator computing from the equations; one
adversarial pass (an independent derivation that does not read the crate, and mutation
testing); one fix round, each fix with a test; and one re-check of exactly the fixed items. Each
pass found something: a blocker in 1d, a major error in 1e, two blockers and a major error in
1f. The re-checks confirmed every fix and left a few mutants alive, which are recorded (O28),
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
exactly; decisions 75 and 172). Three-taxes' ledger holds inside the full closure: (φ_w, φ_r) =
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
  re-check's own probe tests kill four of these (O28).
- **1e** (P1.11; re-checked on `a680dd4`). The derivation found that an exit good made of land
  alone, free at r = 0, put its plot-takers on the floor there while they rented on the wall, so
  the idle stretch did not start where the wall ends, and L1, with one equilibrium, was refused
  as three. Such a good's plots are now decided at the wall's end (decision 160). It also found
  free plots on idle land labelled `Enclosed`; they are `Idle` (161). Thirteen of the pass's 30
  mutants survived; each has a test, and six mutants of the fixes are killed. The re-check's
  derivation agreed on 1,092 draws (189 on idle land, 120 of them decided in the wall's-end
  frame, 15 ties) within 5.1e-14, with 48 invalid in both. Its mutation run killed the thirteen
  and 11 of 13 mutants of the fix; the two left change the frame's price, which the gate cannot
  tell, and the re-check's probe kills both (O28).
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
  six more economies, at σ = 64 with eq 26's weights, were refused as invalid (O29). Its
  mutation run killed 16 of 18 mutants of the seven new tests; the two left are O28's.

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

**`demo-world` is merged** (2026-09-27, `2398b6a`). The many-markets probe (P2.1.1–P2.1.4,
`d0ceaa6` to `b2a55e3`, on `phase2-markets`) and the demo world (D.1–D.5, `a8d3f51` to
`5cd2758`, on `demo-world`) both started at `708167f`, and no code conflicted. `phase2-markets`
changed `crates/agents`, `crates/probe`, one match in `crates/gui/src/vm/inspector.rs`,
`tapes/markets-*.ron`, docs/ENGINE.md, docs/TAPE.md, docs/probe/, README.md and STATE.md.
`demo-world` changed `crates/worldgen`, `crates/gui` but its inspector, `crates/certify`,
`crates/cli`, the workspace manifest, the lockfile, `data/atlas/`, `worlds/`,
`tapes/demo-gb.ron`, `scripts/`, docs/GUI.md, docs/ENGINE.md, docs/demo/, README.md and
STATE.md. Three files were joined by hand, each side's content kept whole:
- README.md: the crate table's worldgen row is the demo's and its probe row the probe's, and
  the tapes paragraph names the markets probe's worlds and the demo world;
- docs/ENGINE.md: both amendment blocks, "Amended at P2.1.1" and then "Amended at D.2";
- this file: the probe's record first below, then the demo's.

The demo's decisions move from n to n + 6, 118–128 to 124–134, and its open items O21–O23 to
O25–O27, with every citation of them in this file, in docs/GUI.md (its blocks "Amended at D.4"
and "The map and lenses, brought forward"), in docs/demo/WORLD.md (§8) and in README.md. The
probe's 118–123 and O21–O24 keep their numbers. No code, test or script cites a number of either
series. The demo's O27 (its O23) planned its second pass for when the two lines met, and now
reads as met. The gates on the committed merge, with a clean build stamp (logs in
`D:/rustyecon-merge-demo/`):
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-merge-demo`,
  warm from a check before the commit, 91 s) and on Windows under Git Bash
  (`D:/rustyecon-targets/merge-demo`, fresh, 211 s), the two run at once. 599 tests pass in the
  workspace on each machine, with 3 ignored and run by name, which is the probe's 569 and the
  demo's 579 less the 549 they shared at `708167f`; certify alone, Parquet-free, passes 67 with
  1 ignored; zero warnings. The gate hash is `0x61f9c8529131ff17`, and the stamp names the
  merge's code, clean, on both (`84d75e9`, `2398b6a` before its record was added; only this
  file differs). Both certificates PASS and recompute byte-equal, the probe's pins hold,
  `derive.py --check` passes, `demo_runs_to_1901` ends at `0xfad880fe08d06645` with hash stream
  `0xdb63cc96f769fb3e`, and telemetry is identical from two processes. The GUI's non-blocking
  check passes in WSL (skipped on Windows), as do the wasm32 checks of the engine and certify.
- **`scripts/gui.sh`** is green in WSL (the same target, 120 s) and on Windows under Git Bash
  (162 s): 84 tests pass with 2 ignored measurements, the 55 named ones by name, fmt and clippy
  clean with `-D warnings`, and five hash diffs equal, the same on both machines: gate (2,080
  ticks, final `0x61f9c8529131ff17`), appb (20,000, `0xe1fa082b26995867`), demo-gb (7,852,
  `0xfad880fe08d06645`), and the two branch tapes as at G0.3 (`branch`, final
  `0x9fc2f964a8510756`; `removal`, `0xd057e3ea708da495`).
- **The probe's record is unchanged:** `crates/agents`, `crates/probe`, `tapes/markets-*.ron`
  and docs/probe/, with its six pinned CSVs in results/markets/, are `b2a55e3`'s byte for byte,
  and `markets_tapes_are_their_generators_output` passes. Every file of the demo's but the
  Markdown is `5cd2758`'s, but for the probe's four lines in the GUI's inspector.
- **Recorded, not gated:** the cli's per-tick hashes of gate (2,080), appb (20,000) and demo-gb
  (7,852) are byte-identical on WSL and Windows.

**Many-markets probe (2026-09-27; P2.1.1–P2.1.4).** The Phase 2 probe's report named many
markets as Phase 2 proper's first untested risk (REPORT §7 Q2); units 1b and 1c give their
known answers. The probe ran on branch `phase2-markets` from `reboot` at `708167f` (worktree
`D:/rustyecon-wt/p2m`, scratch `D:/rustyecon-p2m/`): a frame (`frame/MARKETS-SPEC.md`), an
independent prediction (`predict/PREDICTION.md`), the build (P2.1.1–P2.1.2: four new agent
kinds, `probe::markets`, seven tapes, [MARKETS-RULES.md](docs/probe/MARKETS-RULES.md)), a
registered run (`run/registration.md`, sha256 `a92d9a9c…`), two reviews and the report
(P2.1.3–P2.1.4).
- **Verdict** ([docs/probe/MARKETS.md](docs/probe/MARKETS.md)). **Many markets GO** at C2 copied
  per role (C2m) and 52 ticks a year: I1 (four categories, desks buying land), I2 (a chain of two
  machine types) and I3 (both); all 303 runs end within 1.03e-14 in log and every kick decays.
  **A loop of produced inputs is NO-GO** (L2, L3: two machine types buying each other's
  service): every run diverges at C2m, and at C2L (type rates 5.2/yr, tilt 1) ±20% displacements
  reach an absorbing zero. Every verdict is the predictor's, run for run; the frame's C2L
  prediction was wrong.
- **The reviews** confirm every verdict and narrow it: stationary coins and stocks at every start
  (families 5–9 unrun, O22); the weekly tick only (at 12 a year I2 is unstable, O23); one task
  margin (decision 60); and the GO depends on decision 67, which keeps goods out of machine
  recipes, while the loop's NO-GO does not. Transients are worse in the tails, not the medians
  (O24).
- **What follows:** A11 not met, no fallback; Phase 2 proper opens after 1d and 1e on loop-free
  instances with these roles, the markets harness and C2m at 52/yr; the loop goes to Phase 3
  (decisions 118–123, O21).
- **The gate at `fef01cd`** (P2.1.4's report; `b2a55e3` changes docs only):
  `scripts/gate.sh` is green in WSL (`/root/scratch/target-p2m-report`) and on Windows under Git
  Bash (`D:/rustyecon-targets/p2m-report`), clean stamps; 569 workspace tests pass, 2 ignored
  and run by name, zero warnings. Core, markets and engine are unchanged since `708167f`; the gate world
  ends at `0x61f9c8529131ff17` and appb's 20,000 ticks at `0xe1fa082b26995867`, both per-tick
  streams byte-identical on the two machines (logs in `D:/rustyecon-p2m/report/gate/`).

**The demo world and its map are closed** (2026-09-27, D.5; [docs/GUI.md](docs/GUI.md), the
block "The map and lenses, brought forward"; [docs/demo/WORLD.md](docs/demo/WORLD.md);
decisions 124–134). You asked on 2026-09-27 for "a nice looking map of the UK, and a fairly
complex setup of regions, goods, and history", with lenses like Victoria's that change colour
as a run goes. It was built on branch `demo-world`, from `reboot` at `708167f`, in five
commits:

| Commit | What landed |
|---|---|
| `a8d3f51` D.1 | The atlas, `data/atlas/`: the United Kingdom's 93 historic counties from HCBP Definition B's UK file, with Yorkshire's three ridings from OpenStreetMap (ruling 7), under the ODbL with its own LICENSE and ATTRIBUTION; the loader `rustyecon_worldgen::atlas` |
| `f17b447` D.2 | The demo world's tables (`worlds/demo-gb/`), the compiler (`rustyecon_worldgen::compile`, `rustyecon worldgen`) and `tapes/demo-gb.ron` |
| `8327c1a` D.3 | The map pane and 25 lenses in the GUI; the lean catalogue; series kept in stretches; panels that take 30,000 events |
| `018cf0b` D.4 | The verification's fixes, each with a test: the atlas's credit on the map and `licences` in both binaries; lens values held to the engine; the fresh mesh, the palettes and the legend; the hit test at every part; U4 on the demo tape; `no.trade`; rationing's domain; `certify` sealing `[illustrative]` UNSCORED (O25); the compiler bounding quantities, each trailing year and the dials |
| `5cd2758` D.5 | GUI.md's block, this file, the README and two screenshots |

**What it is.** 93 counties: England 41 with the three ridings, Wales 13, Scotland 33 and
Northern Ireland 6. Northern Ireland is in because HCBP's UK file covers it on the same
permissive terms. Each county is a node running the probe's four roles (GoodDesk, MachDesk,
Provider, Workers) at C2, from its own oracle point. Each has one good and one machine type,
and none trades with another yet. The history's 605 ramp rows become 30,078 dated `SetParam`
steps, 1750–1901: population, sites, enclosure, improvement, mines and coal, steam, canals,
railways, machine tools, textiles by kind, threshing and the Poor Law. The compiler holds each
county date to 3% in log at the oracle and each year to 10%, so no step is a shock (O14). The
run to 1901 has no dead tick and no shortfall. D̂ against each county's moving oracle point has
a median of 11.4, which is 1.1% in log. Every number is `Assumed("illustrative demo …")` and the
name carries `[illustrative]`, so nothing from it is scored or cited (O25).

**The gates at the close** (logs in `D:/rustyecon-demo/close/`), on `018cf0b`'s code; D.5
changes only docs:
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-demo-close`,
  fresh, 183 s) and on Windows under Git Bash (`D:/rustyecon-targets/demo-close`, fresh, 200 s).
  - 579 tests pass in the workspace with 3 ignored and run by name, and certify alone passes 67
    with 1 ignored, with zero warnings.
  - The gate hash is `0x61f9c8529131ff17`, and the stamp matches the checkout.
  - Both certificates PASS, byte-equal, and the probe's pins hold.
  - `derive.py --check` passes. `demo_runs_to_1901` ends at `0xfad880fe08d06645`, with hash
    stream `0xdb63cc96f769fb3e`.
  - Telemetry is identical from two processes.
- **`scripts/gui.sh`** is green in WSL (123 s) and on Windows (162 s).
  - 84 tests pass with 2 ignored measurements, the 55 named ones by name, and fmt and clippy
    are clean with `-D warnings`.
  - Five hash diffs are equal: gate at 2,080 (`0x61f9c8529131ff17`), appb at 20,000
    (`0xe1fa082b26995867`), demo-gb at 7,852 (`0xfad880fe08d06645`), and the two branch tapes as
    at G0.3.
- **Recorded, not gated:** the cli's per-tick hashes of gate, appb and demo-gb are byte-identical
  on WSL and Windows.
- **The smoke mode on Windows,** `rustyecon-gui --smoke 7852 tapes/demo-gb.ron`, ran to 1901 in
  16.1 to 16.7 s, three times. Running, CPU per frame had p50 5.0 to 5.9 ms and p90 6.1 to
  8.3 ms, against G4's bar of a p90 under 8 ms. Paused, p90 was 4.8 to 6.6 ms. The window opened
  and closed by itself.

**Two screenshots** are in `docs/demo/`: the map in 1801 on "Wage in land", and in 1901 on
"Output per head since 1750". egui_kittest's wgpu renderer drew them headlessly on Windows'
software adapter (WARP), from a scratch crate at `D:/rustyecon-demo/close/shot/`, so no
dependency or lockfile changed (decision 133). WSL has no software adapter, so none is taken
or gated there.

**Checked by hand: PENDING, yours** (the demo map's window). On Windows, from the repository
(`reboot` has had the demo since `2398b6a`), in PowerShell:

```powershell
$env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/demo-hand'
cargo run --release -p rustyecon-gui -- tapes/demo-gb.ron
```

The map opens paused at 1750 on "Wage in land".
1. Press Space, and the counties recolour as the history runs; press Space again to pause.
2. `[` and `]` step through the 25 lenses, and `1`–`9` and `0` pick the first ten.
3. Hover a county for its card, and click one to select it.

The README's "The demo world's map" has the WSL command. WSLg is off on this machine
(`guiApplications=false`), so a window from WSL was not tried. Three things the screenshots
show are worth a look (GUI.md's block, item 9): the legend covers Cornwall at the fitted view,
the ranked table's values are cut when the lens's name is long, and the health chip wraps on a
narrow window.

**The verification.** D.2 and D.3 had one bounded verification, one adversarial pass per part
(`D:/rustyecon-demo/verify-map-r1/`, `verify-world-r1/`). On the map it found three major
issues and four minor ones: no ODbL credit, lens values not held away from tick 0, and a
rebuilt mesh's colours untested. On the world it found, among others, that `certify` passed
the illustrative tape and that abrupt histories got past the compiler's bound. D.4 fixed each
with a test, and reran the verifier's mutants against the new tests
(`D:/rustyecon-demo/fix-r1/mutations.txt`: twelve, all killed). The re-check of D.4
(`verify-*-r2/`) found the fixes in place. What it left is carried, not fixed: O26, decision
134.

**`g0` is merged** (2026-09-27, `708167f`). `g0` (G0.1–G0.3, `f897ca8` to `428bdcd`) and
Phase 1's P1.2–P1.7 (`30ff1ce` to `503897e`) both started at `397d7cd`, and no code conflicted:
`reboot` changed only `crates/oracle`, README.md and STATE.md, and `g0` only `crates/gui`, the
workspace manifest, the lockfile, `scripts/`, the CI workflow's comment, docs/ENGINE.md,
docs/GUI.md, README.md and STATE.md. README.md and this file were joined by hand, each side's
record kept whole: Phase 1's units 1b and 1c first below, then G0's steps. Phase 1's decisions
keep 59–75, and G0's move from n to n + 17: 59–64 to 76–81 (G0.1's first part), 65–72 to 82–89
(its second part), 73–79 to 90–96 (its verification fixes), 80–91 to 97–108 (G0.2), 92–96 to
109–113 (its verification fixes) and 97–100 to 114–117 (G0's close), with every citation of
them in this file. G0's O18 is O20, here and in docs/GUI.md's one citation of it (its block
"Closed at G0.3", item 5). No other file cites a number of either series. The gates on the
committed merge, with a clean build stamp (logs in `D:/rustyecon-merge-g0/`):
- **`scripts/gate.sh`** is green in WSL (`CARGO_TARGET_DIR=/root/scratch/target-merge-g0`, a
  fresh target, 141 s) and on Windows under Git Bash (`D:/rustyecon-targets/merge-g0`, fresh,
  138 s): 549 tests pass in the workspace on each machine, with 2 ignored and run by name, and
  zero warnings, as at P1.7, since G0 changed no crate the gate tests; certify alone,
  Parquet-free, passes 65 with 1 ignored. The gate hash is `0x61f9c8529131ff17` on both, both
  certificates PASS and recompute byte-equal on both, the probe's pins hold, and telemetry is
  identical from two processes. The GUI's non-blocking check passes in WSL (skipped on
  Windows), as do the wasm32 checks of the engine and certify.
- **`scripts/gui.sh`** is green in WSL (the same target, 77 s) and on Windows under Git Bash
  (100 s): 71 tests pass and one measurement is ignored, the 42 named ones by name, fmt and
  clippy clean with `-D warnings`, zero warnings, and four hash diffs equal, the same on both
  machines: gate (2,080 ticks, final `0x61f9c8529131ff17`), appb (20,000, final
  `0xe1fa082b26995867`), and the two branch tapes run by the cli, their `tape_hash` the GUI's
  (`branch`, `0x1b061337f44ea5c1`, final `0x9fc2f964a8510756`; `removal`,
  `0xe15acd82a00d52fe`, final `0xd057e3ea708da495`; both marked `[GUI experiment 2026-09-27]`,
  as at G0.3).
- **Recorded, not gated:** the cli's 2,080 gate and 20,000 appb per-tick hashes are
  byte-identical on WSL and Windows.

**Phase 1's units 1b and 1c were closed at P1.7, with the gate green in WSL and on Windows**
(2026-09-27; [crates/oracle/README.md](crates/oracle/README.md), with the specs
[unit-1b.md](crates/oracle/docs/unit-1b.md) and [unit-1c.md](crates/oracle/docs/unit-1c.md)).
Each unit had a spec written from laborformal `31b3482` and checked against a 70-digit mpmath
prototype in scratch; a build whose goldens come from a committed mpmath generator computing
from the equations; an adversarial pass (an independent derivation that does not read the
crate, and mutation testing); a fix round, each fix with a test; and a re-check of exactly the
fixed items. Unit 1c had two passes, and the second found a blocker. Only `crates/oracle`
changed on `phase1` since `397d7cd`.

| Commit | What landed |
|---|---|
| `30ff1ce` P1.2 | unit 1b, many categories and the fork, with its spec (docs/unit-1b.md, its §12 the build's departures), `generate_1b.py` and `goldens_1b.txt` |
| `f4de477` P1.3 | 1b's verification fixes: four surviving mutants caught, precision near an interior edge stated and measured by 24 new goldens; no code change but `ces_share` checking α through `Requirement::OpenUnit` |
| `c9b1920` P1.4 | unit 1c, many machine types and the Leontief inverse, with its spec (docs/unit-1c.md), `generate_1c.py` and `goldens_1c.txt` |
| `9e2a7e6` P1.5 | 1c's first verification: five surviving mutants killed by new tests; code unchanged |
| `5081345` P1.6 | 1c's second verification: the blocker fixed (a boundary regime hid equilibria), seven tests, precision at ties stated |
| `503897e` P1.7 | STATE.md, the oracle's README, the root README's status lines |

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

**G0 is closed** (2026-09-27, G0.3; [docs/GUI.md](docs/GUI.md), closed at G0.3; ENGINE,
amended at G0.3; decisions 114–117). G0 built `crates/gui`, the GUI's shell, on branch `g0`
from `397d7cd`, in six commits:

| Commit | What landed |
|---|---|
| `f897ca8` G0.1 | `crates/gui` and its seams: the model and `reduce`, the Runner, `ThreadDriver`, the Extractor, the store and its decimator, the ring, `session.ron` and `layout.ron`; D1 in the workspace; `scripts/gui.sh` |
| `1d6dd2b` G0.1 | the panels (toolbar, timeline, outliner, plots, inspector, registry, log), the view-model goldens, G0's kittest scripts, `every_drawn_vertex_is_recorded` and the smoke mode |
| `2cced21` G0.1 | the verification's fixes, five majors and seven minors, each with a test |
| `b2757c6` G0.2 | the editor: tape edits, branches, the lineage, compare and export |
| `22dff0f` G0.2 | the verification's fixes, six majors and four minors, each with a test |
| `428bdcd` G0.3 | STATE.md, GUI.md, ENGINE and README; `scripts/gui.sh` names the reducer's tests |

No crate but `crates/gui` changed, so no hash, `prefix_id` or `world_id` moved. The lockfile
has not changed since `1d6dd2b`, where it gained only the GUI's edge to core.

**The gate** (GUI.md §9), item by item:
- **The named tests pass under `scripts/gui.sh`,** on WSL and on Windows. It names 42, up from
  27: `gui_equals_cli`; `failed_run_shows_its_ledger_line`; the two branch tests; the five
  editing tests; the reducer's fifteen state-machine tests, named at the close; the two
  goldens; the eight kittest scripts, `every_drawn_vertex_is_recorded` among them;
  `decimation_keeps_extremes` and `nonfinite_ingest_stops_with_the_series_named`; and six scans,
  the two §9 names, the two copied ones, `the_gui_names_core_for_num_alone` and
  `edit_reaches_no_model_file_thread_or_clock`.
- **Clippy** with `-D warnings` under the workspace lints: clean on both machines.
- **The window on Windows, checked by hand: PENDING, yours.** See below.
- **Smoke mode** on Windows: recorded at each step, and again at the close (below).
- **`scripts/gate.sh` carries D1,** with ENGINE's amendment, since G0.1.
- **The costs, the recount and the rerun time** are recorded below, at G0.1 and again at the
  close.
- **One key press gives a live price plot** from `rustyecon-gui tapes/gate.ron`:
  `one_key_press_gives_a_live_price_plot`.

**The measurements** (WSL, 48 threads, release where it applies, alone on the machine; logs in
`D:/rustyecon-g0/close/measure.txt`). G0.1 took the first three on the crate's seams; the
close took them again on the finished crate:
- **The clean check,** `cargo check -p rustyecon-gui` in a fresh target with certify in the
  tree: 17.3 s, a 0.85 GB target (`du -sb`), 0.87 GB peak RSS. At G0.1: 16.1 s, 0.81 GB and
  0.87 GB.
- **The warm check** after touching `crates/engine/src/lib.rs`: 2.92, 2.89 and 2.92 s, median
  2.92 s. At G0.1: median 2.55 s.
- **The recount** (`cargo tree -e normal`, root included): 274 crates on Linux, 187 on Windows,
  125 on wasm32; 281, 196 and 129 with build dependencies, the same as at G0.1. The licences
  are G0.1's, below.
- **The gate world's rerun time,** with no threshold: `materialise` of `mill.spend`'s genesis
  value, then `Sim::new` and `run_until(2080)` through `ThreadDriver` with the G0 Extractor
  (`gate_rerun_time_is_recorded`, ignored): median 41.3 ms of 5 (40.1 to 43.5 ms), and 40.2 ms
  when run again. At G0.2, before its fixes: 42.5 ms.

**Tests.** 72 in `crates/gui`, 71 run and one ignored measurement: app 8 (the kittest
scripts), branch 3 (one ignored), edit 6, failed 1, goldens 2, hashes 1, model 15, persist 4,
runner 5, scans 8, store 4, and 15 unit tests. The engine's gate is unchanged: 421 tests pass
with 2 ignored and run by name.

**Mutation over G0,** each mutant alone against the whole suite, each killed by a named test
that guards it: 23 at G0.1's first part, 21 at its second, 33 at its fixes, 45 at G0.2 and 72
at its fixes (G0.2's 45 rewritten among them). At the close, ten of `reduce`, one for each
state-machine test that no earlier round had killed by name, all killed
(`D:/rustyecon-g0/close/mutants.txt`): the new run's breakpoints dropped, an unreadable tape
not logged, Space not running a paused run, a year's step one short, a poisoned run taking
Run, a speed change not resent, a base dropped while another run reads it, a closed run's late
observation logged, a changed base keeping its old hash, and the serial not advanced. Under
the poisoned-run mutant `a_run_whose_worker_ended_says_so_and_stops` also hung, and was
stopped by hand.

**The verifications.** G0.1 and G0.2 each had a bounded verification: one adversarial pass,
one fix round, one re-check of the fixed items. Both re-checks found the fixes in place, and
both left findings the close carries, not fixes (decision 116; O20).

**The gates at the close** (logs in `D:/rustyecon-g0/close/`, `gate-*-pre.log` and
`gui-*-pre.log`), on G0.3's tree before it was committed. Its only changes to
`22dff0f` are docs and `scripts/gui.sh`, so the build stamp reads `22dff0f` clean:
- **`scripts/gui.sh`** is green in WSL (76 s, `CARGO_TARGET_DIR=/root/scratch/target-g0-close`)
  and on Windows under Git Bash (115 s, `D:/rustyecon-targets/g0-close`): 71 tests pass and
  one measurement is ignored, the 42 named ones by name, fmt and clippy clean with `-D
  warnings`, zero warnings, and four hash diffs equal on both machines: gate (2,080 ticks,
  final `0x61f9c8529131ff17`), appb (20,000, final `0xe1fa082b26995867`), and the two branch
  tapes run by the cli, their `tape_hash` the GUI's (`branch`, `0x1b061337f44ea5c1`, final
  `0x9fc2f964a8510756`; `removal`, `0xe15acd82a00d52fe`, final `0xd057e3ea708da495`).
- **`scripts/gate.sh`** is green in WSL (125 s, a fresh target) and on Windows under Git Bash
  (127 s, a fresh target): 421 tests pass with 2 ignored and run by name, zero warnings, the
  gate hash `0x61f9c8529131ff17`, the stamp matching the checkout, both certificates PASS
  and byte-equal on both machines, the probe's pins, and telemetry identical from two
  processes. The GUI's non-blocking check passes in WSL and is skipped on Windows; the wasm32
  checks of the engine and certify pass in WSL.
- **Smoke mode on Windows** (release, the gate to 2,080 at ten years a second, 5.2 s; panes
  drawn: Outliner, Plots, Inspector, Timeline): 484 frames running, CPU per frame p50 1.05 ms,
  p90 1.62 ms, max 36.39 ms (the first frame); 120 paused, p50 1.60 ms, p90 2.31 ms, max
  2.78 ms. No panic. The window opened and closed by itself; nobody looked at it.

**Checked by hand: PENDING, yours** (the G0 gate's window item). On Windows, from the
repository, in PowerShell:

```powershell
$env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/g0-hand'
cargo run --release -p rustyecon-gui -- tapes/gate.ron
```

Type `2080` in "until" and press "Run until", or press Space and Space again to pause. Check
that every price is plotted, that the run reaches tick 2,080, and that nothing panics. `cargo
run --release -p rustyecon-gui -- --smoke 2080 tapes/gate.ron` prints the CPU per frame of the
same run. To try the editor with the window open: in the Editor tab, Mint key, date
`1765-06-01`, act `SetParam(param: "mine.capacity", to: "mine.capacity.base")`, a note, Add
edit, Apply; run the branch to 2080, and the Compare tab shows the first differing hash at
report tick 801. If the window fails your look, the fix is a G0.4 on `g0` before G1 (decision
114).

**G0.2's verification fixes** (2026-09-27; [docs/GUI.md](docs/GUI.md), amended at G0.2's
verification fixes; decisions 109–113). A bounded verification of G0.2 found six major issues
and four minor ones. Each is fixed or answered, with a test that fails without its fix,
checked by mutation:
- **"Ledger changed" said "no" when it could not know.** A reopened hand-edited tape whose
  base was not open exported `# ledger changed: no`. It now has three answers. A reopened
  tape reads the ancestor its lineage names from its path and keeps it only when its
  `tape_hash` matches. Without it, the chip says "ledger unchecked" and why, the CSV says
  `unknown (…)`, and the manifest says `None` (`ledger_tolerances_are_not_editable`).
- **U3's other clauses had no test.** A lineage alone and a marked basis alone now make an
  experiment, and neither makes a run (`saved_tape_carries_its_lineage`).
- **A reopened experiment's export had no `ancestor.ron`,** and the test that claimed it
  exported another run. The reopened run keeps its ancestor, and the test exports it, the
  in-memory branch and a fresh session's reopen.
- **Minted keys collided across one family:** a reopened saved branch, one file opened twice,
  a closed parent. Keys are now new to every tape open in the session. A minted `n` only
  grows, and a set-aside session's serial is carried forward (`minted_keys_never_collide`,
  with params, recurring entries and Apply's re-check).
- **Compare gave a false first difference under a resumed parent.** It now reads the parent's
  earlier states in the records it resumed from. When they are gone, it says the hashes may
  differ earlier, and it always shows the range compared (`branch_resume_equals_rerun`).
- **The minors.** RemoveRecurring and its offered RemoveParam are tested through the model.
  A lineage that describes another tape is flagged in the CSV and the manifest, and logged
  when saved. Compare's identities name their origin. `scripts/gui.sh` checks the cli's
  `tape_hash` of each branch tape against the GUI's. Two windows on one `session.ron` can
  still share a serial (decision 110).
- **Tests:** 72 in `crates/gui`, one of them an ignored measurement, as before. The fixes
  extend six named tests and the branch script, which now also opens the saved tape, edited
  by hand, in two new windows ("ledger changed" with the base on disk, "ledger unchecked" and
  why without it). `scripts/gui.sh` still names 27.
- **Mutation** (WSL, each mutant alone against the whole suite, on a copy of the tree;
  `D:/rustyecon-g0/g02-fix/`, `round1/`, `round2.txt`, logs in `mut/`): 72 mutants. They are
  G0.2's 45 again, rewritten where the fixes changed their text; the verification's five
  (MX1, MX2, MX3, MX6, M5); and 22 of the fixes' own. In the first round 67 were killed. One
  survived: the chip's "ledger unchecked" was painted by no test. The branch script's two new
  windows kill it, and three more mutants of the chip and of the host's read of the ancestor.
  So all 72 are killed, each by a named test that guards it. A 73rd, a branch test writing
  another `tape_hash` than the run key's, turns `scripts/gui.sh` red at its new check.
- **The gates** (logs in `D:/rustyecon-g0/g02-fix/`). `scripts/gui.sh` is green in WSL and
  on Windows under Git Bash (99 s with the build): 71 tests pass and one measurement is
  ignored, the 27 named ones by name, clippy clean with `-D warnings`, and four hash diffs
  equal. They are gate (final `0x61f9c8529131ff17`), appb (final `0xe1fa082b26995867`), and
  the two branch tapes run by the cli, whose `tape_hash` is now checked against the GUI's
  (`branch`, `0x1b061337f44ea5c1`, final `0x9fc2f964a8510756`; `removal`,
  `0xe15acd82a00d52fe`, final `0xd057e3ea708da495`), the same on both machines.
  `scripts/gate.sh` is green in WSL (44 s) and on Windows under Git Bash (41 s): the gate
  hash `0x61f9c8529131ff17`, both certificates PASS and byte-equal, and the GUI's
  non-blocking check passing in WSL (skipped on Windows). No engine file
  changed, and the lockfile did not change. The editor's, branch, reducer and kittest tests
  ran 20 times in WSL and 10 on Windows, all clean.
- **Checked by hand: still PENDING, yours.** The command is under G0.1's second part, below.

**G0.2: the editor** (2026-09-27; [docs/GUI.md](docs/GUI.md), amended at G0.2). The editor is
built at G0's scope, in `crates/gui/src/edit/` (no egui, model, file, thread or clock) with
the model's branches and two new panes:
- **Edits.** A `TapeEdit` is an `EditOp` and a required note; no arm carries a basis.
  `materialise` applies the edits, stamps every entry it adds or changes
  `Assumed("GUI experiment <date>: <note>")`, marks the name `[GUI experiment <date>]`
  (replacing any marker), refuses the ledger's tolerances, round-trips the text and validates
  through `Sim::new`, which it drops. A removal that leaves params unreferenced lists them all,
  and the editor offers a `RemoveParam` for each, with a note of its own.
- **Branches.** Apply makes a branch of the focused run: a new run with a parent. `plan`
  resumes it from the parent's latest ring checkpoint whose stored `prefix_id` its world
  shares, else reruns from genesis and the log says why; the new Runner, on its own worker,
  receives the checkpoint in `Cmd::Load`.
- **The lineage:** the nearest ancestor on disk by `tape_hash` and path, and every edit since,
  with notes, dates and what each replaced or removed. "Save tape as" writes the canonical
  tape and `<name>.lineage.ron`, never over a file; a saved tape is its children's ancestor.
- **Keys:** `[a-z0-9_.-]+`, new to every tape of the run tree, the staged edits and the runs
  closed this session; minted `gui.<s>.<n>`, `s` the session's serial (`session.ron` format 2).
- **The form** reads the act as the tape writes it; an empty note, a malformed key or date, a
  taken key, a ledger tolerance and an act that does not read are refused, the last with the
  parser's line, column and caret on the read-only raw pane.
- **Compare:** both identities, the first differing hash and the report tick that left it, the
  lineage, the tape diff by section and key, and the plotted series' difference at the cursor,
  its largest and its first tick.
- **Export** of the focused run: `series.csv` (the plotted series at full resolution, `#` lines
  with the stamp, the cli's `run …` line, the ticks, the origin, the lineage and "ledger
  changed"), `manifest.ron` (`GuiManifest` around certify's `Manifest`, with `resumed_from` for
  a resumed branch), `tape.ron`, and for an experiment `tape.lineage.ron` and `ancestor.ron`.
- **"Ledger changed"** compares a run's tolerances with its parent's, or with the ancestor a
  reopened tape's lineage names; the health chip and the export show it.

- **Tests** (72 in `crates/gui`, one of them an ignored measurement, up from 54;
  `scripts/gui.sh` names 27, up from 17):
  `branch_resume_equals_rerun` (through `ThreadDriver`: the branch of `SetParam(mine.capacity,
  mine.capacity.base)` on 1765-06-01 resumes at state tick 780, equals its rerun, and first
  differs at report tick 801), `removal_only_branch_is_an_experiment` (resumes at 519; the name
  carries the only marker; chip, CSV and manifest say "experiment"; a world edit reruns),
  `gui_edits_are_always_assumed`, `saved_tape_carries_its_lineage`,
  `removing_the_last_use_offers_remove_param`, `ledger_tolerances_are_not_editable`,
  `minted_keys_never_collide`, the kittest scripts
  `the_editor_refuses_an_empty_note_a_malformed_key_and_a_malformed_date` and
  `the_branch_script_applies_compares_exports_and_saves`, and the scan
  `edit_reaches_no_model_file_thread_or_clock`; with them `the_form_checks_first`, the reducer's
  `apply_branches_from_the_parents_ring_and_files_are_effects`, and unit tests. `scripts/gui.sh`
  also runs the cli on the two branch tapes the branch tests write and diffs their hashes.
- **Mutation** (WSL, on a copy of the tree, each mutant alone against the whole suite;
  `D:/rustyecon-g0/g02-editor/mutants-round*.txt`, logs in `mut/`): 45 mutants, all killed, and
  each by a named test that guards it. `plan` taking the oldest checkpoint, ignoring the prefix
  or the world, a branch loaded from genesis, a ring prefix read a tick late, compare's first
  difference a tick early, and compare flagging a rerun or never flagging a forged start; no
  marker in the name, an export or manifest stamped "run", a branch of origin run; a stamp
  without its note, a changed genesis value keeping its basis, an empty note passed, an added
  event stamped as the parent's; a lineage dropping earlier edits or naming an unsaved parent,
  a saved tape not on disk, no lineage file, files written over; no orphan offer, an offer
  taken without a note, only the first orphan; the ledger editable, never changed, or not
  exported; keys of the run alone, staged keys not taken, the serial ignored, closed runs'
  keys reused; the form taking an empty note or any key, the raw pane's caret off, Add edit
  and Apply dead, compare without the lineage, the chip saying run; `edit/` writing a file or
  reading the model, vm/ reading `edit/`, core's writer in `edit/` and its raw schema outside
  it; the export forgetting the resume, and the store forgetting its start hash.
- **The gates** (logs in `D:/rustyecon-g0/g02-editor/`). `scripts/gui.sh` is green in WSL
  (35 s warm) and on Windows under Git Bash (46 s): 71 tests pass and one measurement is
  ignored, the 27 named ones by name, clippy clean under the workspace lints with `-D
  warnings`, zero warnings, and four hash diffs equal: gate (2,080 ticks, final
  `0x61f9c8529131ff17`), appb (20,000, final `0xe1fa082b26995867`), and the two branch tapes
  run by the cli from genesis (`branch`, final `0x9fc2f964a8510756`; `removal`, final
  `0xd057e3ea708da495`), the same finals on both machines. `scripts/gate.sh` is green in WSL
  (57 s) and on Windows under Git Bash (123 s), with the engine unaffected: 421 tests pass
  with 2 ignored and run by name, zero warnings, the gate hash `0x61f9c8529131ff17`, both
  certificates PASS and byte-equal, and the GUI's non-blocking check passing in WSL (skipped
  on Windows). No engine file changed, and the lockfile did not change. The editor's kittest
  scripts, the branch tests and the editing tests ran 20 times in WSL and 10 on Windows, all
  clean, after one Windows failure was fixed: its longer temporary paths wrapped compare's
  lines and pushed the tape diff below the fold, so the scripts now scroll each line they
  check into view.
- **Smoke mode on Windows** (release, the gate to 2,080 at ten years a second; panes drawn:
  Outliner, Plots, Inspector, Timeline): 483 frames running, CPU per frame p50 0.92 ms, p90
  1.29 ms, max 49.12 ms (the first frame); 120 paused, p50 1.24 ms, p90 1.46 ms, max 2.13 ms.
  No panic; nobody looked at the window.
- **The gate world's rerun time** (G0's gate item, no threshold; WSL, release, alone):
  `materialise` of `mill.spend`'s genesis value, then `Sim::new` and `run_until(2080)` through
  `ThreadDriver` with the G0 Extractor, median 42.5 ms of 5 (41.3 to 43.6 ms), by the ignored
  test `gate_rerun_time_is_recorded`.
- **Checked by hand: still PENDING, yours.** The command is under G0.1's second part, below;
  with the window open, the editor can be tried too: in the Editor tab, Mint key, date
  `1765-06-01`, act `SetParam(param: "mine.capacity", to: "mine.capacity.base")`, a note, Add
  edit, Apply; run the branch to 2080 and the Compare tab shows the first differing hash at
  report tick 801.

**G0.1's verification fixes** (2026-09-27; [docs/GUI.md](docs/GUI.md), amended at G0.1's
verification fixes). A bounded verification of G0.1 found five major issues and seven minor
ones. All are fixed, each with a test that fails without its fix, checked by mutation:
- **A failed run's inspector read the failure's state.** After a failed step the Runner took
  the actor snapshot of the poisoned `Sim`, which holds what the failed tick applied before it
  failed: the workers' lots showed 1,117.29 coin beside a holding of 117.29. A poisoned run's
  current tick is now rebuilt from the ring, like any earlier tick
  (`failed_run_shows_its_ledger_line`, on a theft that a gift precedes).
- **The scans missed group and glob imports.** `use crate::{ui as _}`, `use crate::*` with a bare
  `ui::` path, `use std::{thread as _}` and `use crate::{model as _}` passed. The scans now read
  `use` trees, and their fixtures hold each form.
- **`every_drawn_vertex_is_recorded` read the cache, not what egui got.** One function now
  makes each segment's `Line`, hands it over and records its points; the test also matches
  each segment to the path egui paints, per panel on one map of ticks, with the cursor.
- **The scripts left the chip, the axes, a failed run and panel plotting unpainted.** The gate
  script now checks the painted identity chip, units, year labels and cursors, and plots from
  the inspector and the outliner; `the_theft_script_shows_a_failed_run` is new.
- **A session another tape wrote plotted nothing.** A run now opens with every price plotted
  when the session's plots name nothing of its world, and another world's keys are left out of
  the stack with no made-up unit (`a_second_tapes_session_still_plots_every_price`).
- **Minors:** the observe scan's group imports (as above); U10's test poisons NaN, +inf and
  −inf; a worker that panics is reported once (`Obs::Ended`); the core edge's guard is named as
  the scan; year gridlines fall on year starts; log lines carry report ticks; and the ledger
  line is painted by key beside the engine's. The last minor arrived cut off after "names
  dense ids", and was read as U7 on the ledger line.
- **Tests:** 54 in `crates/gui`, up from 46; `scripts/gui.sh` names 17. The goldens' log,
  toolbar and plots files changed (report ticks, `ledger_keys`, `absent`), rewritten on WSL and
  equal on Windows.
- **Mutation** (WSL, on a copy of the tree, each mutant alone against the whole suite;
  `D:/rustyecon-g0/g01-fix/mutants*.txt`, logs in `mut/`): 33 mutants, all killed, three of
  them again after the scripts read the y axes by their rotated labels. They include the
  verification's own: the poisoned snapshot; C1 (`use crate::{ui as _}`) and C3 (a glob and a
  bare `ui::` path); C4 and C5; M1 (segments joined where they are lent), M2 (a vertex lent a
  tick late), M13 (no line handed over) and a new M14 (one set recorded, another lent); M3 to
  M11; S1 (`is_nan` for `is_finite`); and one or more for each other fix.
- **The gates** (logs in `D:/rustyecon-g0/g01-fix/`). `scripts/gui.sh` is green in WSL (34 s)
  and on Windows under Git Bash (33 s): 54 tests, the 17 named ones by name, clippy clean with
  `-D warnings`, and both hash diffs equal (gate 2,080 ticks, final `0x61f9c8529131ff17`; appb
  20,000, final `0xe1fa082b26995867`). `scripts/gate.sh` is green in WSL (122 s, a fresh
  target): 421 tests pass with 2 ignored and run by name, both certificates PASS and
  byte-equal, and the GUI's non-blocking check passes. No engine file changed. The kittest
  scripts ran 40 times concurrently and 20 times serially in WSL, and the failed-run test 30
  times, all clean.
- **Smoke mode on Windows** (release, the RTX 4090, panes drawn: Outliner, Plots, Inspector,
  Timeline). The gate to 2,080: 477 frames running, CPU per frame p50 0.85 ms, p90 1.18 ms, max
  36.57 ms (the first frame); 120 paused, p50 1.19 ms, p90 1.56 ms, max 2.37 ms. appb to
  20,000 in 39.5 s: 4,612 frames running, p50 0.83 ms, p90 1.11 ms, max 32.88 ms; paused p50
  0.93 ms, p90 1.10 ms, max 1.31 ms. No panic. Nobody looked at the window.
- **Checked by hand: still PENDING, yours.** The command is under G0.1's second part, below.

**G0.1's second part: the panels** (2026-09-27; [docs/GUI.md](docs/GUI.md), amended at G0.1's
second part; ENGINE, amended at G0.1, item 5). G0.1, the viewer, is built. `rustyecon-gui
tapes/gate.ron` opens paused at tick 0 with every price plotted, and Space gives a live price
plot. Each panel is a pure view-model in `vm/` that `ui/` draws:
- **Toolbar:** Open (rfd), Run and Pause (Space), Step (`.`), Step a year, run until a tick or
  a date, the speed cap; the date, tick and ticks per year; the identity chip (commit and dirty
  flag, `world_id`, `tape_hash`, origin); the health chip (status, `max_margin`, hash, "ledger
  changed", a failed run's ledger line and last good tick).
- **Timeline:** a scrubber with the cursor, each year, the fired and scheduled events, and the
  ring's checkpoints.
- **Outliner:** the tape's entities by key, to select, pin and plot.
- **Plots:** any series, one panel per unit, thinned by the plot cache, the x axis labelled by
  date, the cursor linked.
- **Inspector:** a market (p, next p, ema, S, D, cleared, fills, rationing by class, ln(p′/p),
  the rule and its rate param with unit, basis and per-tick step); an actor (its spec's params,
  inline numbers, holdings, and lots and state from a snapshot, settle lines); a param; an event;
  goods, nodes and classes.
- **Registry:** `engine::registry`'s rows with each use and its per-tick value, the current
  value, and the basis the last `SetParam` copied, read from `FiredEvent.source`.
- **Log:** loads, runs, pauses, fired events, rationing onsets by class, errors, and the
  breakpoint on error.
The tile layout persists in `layout.ron`. The smoke mode, `rustyecon-gui --smoke UNTIL TAPE`,
times the frames.

- **Tests** (46 in `crates/gui`, up from 33; `scripts/gui.sh` names 15):
  `every_drawn_vertex_is_recorded` (the gate run to 2,080 in the app, with every price, a param
  that steps and a series with gaps plotted, at 1,600 and 640 pixels wide: every vertex lent to
  egui equals a recorded point bit for bit, one segment per unbroken stretch, each line's
  extremes drawn); the view-model goldens `gate_view_models_equal_their_goldens` (four points)
  and `appb_view_models_equal_their_goldens` (two), 42 RON files in `crates/gui/tests/golden`,
  equal byte for byte on WSL and Windows, each point asserting its claim; G0's kittest scripts
  `one_key_press_gives_a_live_price_plot`, `the_gate_script_runs_pauses_steps_and_inspects` and
  `the_appb_script_runs_pauses_steps_and_inspects`; `the_gui_names_core_for_num_alone`; and
  `a_selected_actor_asks_for_the_snapshot_its_cursor_reads`,
  `rationing_onsets_are_logged_once_a_class_line` and five unit tests. The Runner's snapshot
  test checks the lots.
- **Mutation** (WSL, in a copy of the tree; `D:/rustyecon-g0/g01-panels/mutants*.txt`): 21
  mutants, all killed, and the scripts' five killed again after the scripts changed. For
  `every_drawn_vertex_is_recorded`: a vertex lent one tick to the right, segments joined across
  a gap, a plotted line dropped, a column keeping one extreme. For the goldens: the copied basis
  read from the target, ln(p/p′) for ln(p′/p), the price unit reversed, every event marked
  fired, the onset watch logging nothing. For the scripts: `.` stepping two, Space ignored, a
  date not run through, a value without its unit, no snapshot asked, the speed control ignored,
  the inspector without the copied basis. Also the model's snapshot request, the core scan (a
  group import), the snapshot's lots, and onsets logged at a full fill or not logged.
- **A flaky script, found and fixed.** The gate script failed about one run in fifteen. The
  uncapped gate run could pass tick 480 before the Space pause landed, so a year's step passed
  the cut, and "run until 1760-03-01" paused at once. The script now caps the speed through the
  toolbar while it runs and pauses. It checks the text each frame paints, which is what is on
  screen. After the fix, 60 repeats in WSL and 10 on Windows ran clean.
- **Smoke mode on Windows** (release, a 1,600 × 1,000 window on the RTX 4090, panes drawn:
  Outliner, Plots, Inspector, Timeline; logs in `D:/rustyecon-g0/g01-panels/`). The gate to
  2,080 at 520 ticks a second took 5.0 s: 482 frames running, CPU per frame p50 1.04 ms, p90
  1.49 ms, max 43.15 ms (the first frame); 120 frames paused, p50 1.31 ms, p90 1.83 ms, max
  2.26 ms. appb to 20,000 took 39.5 s: 4,613 frames running, p50 0.77 ms, p90 1.00 ms, max
  30.12 ms; paused p50 0.74 ms, p90 0.89 ms, max 1.07 ms. No panic. The window opened and closed
  by itself; nobody looked at it.
- **The gates** (logs in `D:/rustyecon-g0/g01-panels/`, `*-final.log`). `scripts/gate.sh` is
  green in WSL (44 s warm; 138 s from a fresh target) and on Windows under Git Bash (48 s), with
  the engine unaffected: 421 tests pass with 2 ignored and run by name, as at S2.5, zero
  warnings, the gate hash `0x61f9c8529131ff17`, both certificates PASS and byte-equal, and the
  GUI's non-blocking check passing in WSL (skipped on Windows). `scripts/gui.sh` is green in WSL
  (30 s warm) and on Windows (40 s): 46 tests, the 15 named ones by name, clippy clean under the
  workspace lints with `-D warnings`, and both hash diffs equal (gate 2,080 ticks, final
  `0x61f9c8529131ff17`; appb 20,000, final `0xe1fa082b26995867`).
- **The lockfile** gained one line: the GUI's own edge to core. No package was added.
- **Checked by hand: PENDING, yours** (the G0 gate's window item). On Windows, from the
  repository, in PowerShell: `$env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/g0-hand'; cargo run
  --release -p rustyecon-gui -- tapes/gate.ron`. Type `2080` in "until" and press "Run until",
  or press Space and Space again to pause. Check that every price is plotted, that the run
  reaches tick 2,080, and that nothing panics. `cargo run --release -p rustyecon-gui -- --smoke
  2080 tapes/gate.ron` prints the CPU per frame of the same run.
- **Not yet recorded**: the gate world's rerun time through `ThreadDriver` (it times
  `materialise`, so it waits for G0.2). Recorded at G0.2, above.

**G0.1's first part: `crates/gui` and its seams** (2026-09-27; [docs/GUI.md](docs/GUI.md),
amended at G0.1; ENGINE, amended at G0.1). The crate `rustyecon-gui` (lib and binary) holds the
seam GUI.md §3 fixes, with no panels yet: `model/` (the `Session`, `Intent`, `Effect` and
`reduce`, a pure state machine), `run/` (`Cmd`, `Obs`, the `Runner` that owns the `Sim`, the
`Extractor`, the in-memory `Store` with a decimator that keeps each column's extremes, the ring
of checkpoints, the log lines), `vm/` (the toolbar and the log), `drive/` (`ThreadDriver`, one
worker per run, and the `Host` that carries out effects), `platform/` (`session.ron`,
`layout.ron`), and `ui/` with the tile layout and a status line. D1 is in the workspace:
`default-members` leave the GUI out, `scripts/gate.sh` excludes it from clippy and the tests
and checks it once on Linux without gating, and `scripts/gui.sh` is its own gate.

- **Tests** (33, all in `crates/gui`, run by `scripts/gui.sh` only): `gui_equals_cli` (gate to
  2,080 ticks and appb to 20,000, through `ThreadDriver` by a fixed script of steps of 1 and 7,
  run-untils, speed caps, a pause, snapshots at 300 and 250 and the breakpoint on error; every
  hash equal to `Sim::new` plus `run_until`, and `scripts/gui.sh` diffs the files against the
  cli's `--hashes`), `failed_run_shows_its_ledger_line` (the theft variant pauses on the
  breakpoint at tick 73; the toolbar shows `Poisoned`, the ledger line and last good tick 72),
  `decimation_keeps_extremes`, `nonfinite_ingest_stops_with_the_series_named`,
  `model_run_edit_vm_import_no_egui`, `no_trig_outside_ui`, and the copied
  `no_raw_transcendentals` and `no_hashed_collections`; with them the reducer's state machine
  (9 tests), the Runner (5: pauses, the ring's 41 checkpoints on the gate, a resume from the
  ring, a refused resume that reruns from genesis, snapshots), persistence (4), the store (2
  more), the scanner's own test, a fifth scan that keeps run/ and vm/ free of the model, files,
  threads and clocks (D13), two unit tests and one headless app test (egui_kittest, no GPU:
  the gate opens paused at tick 0 with every price among its plots, Space runs and pauses,
  `.` steps).
- **Mutation** (checked 2026-09-27, WSL): 23 mutants of what the eight named tests guard, all
  killed. One first survived: naming `catalogue[0]` for the non-finite series was equivalent on
  the price of bread in town, which is the first series; the test now poisons the price of bread
  in the village.
- **Hashes**: the GUI's path gives the gate's 2,080 and appb's 20,000 per-tick hashes byte for
  byte as the cli does (finals `0x61f9c8529131ff17` and `0xe1fa082b26995867`), on WSL and on
  Windows.
- **The gates** (logs in `D:/rustyecon-g0/g01-seams/`): `scripts/gate.sh` is green in WSL and on
  Windows under Git Bash, with the engine unaffected: 421 tests pass with 2 ignored and run by
  name, as at S2.5, zero warnings, the gate hash `0x61f9c8529131ff17`, and the GUI's
  non-blocking check passing in WSL (skipped on Windows). `scripts/gui.sh` is green in WSL and on
  Windows: 33 tests, clippy clean under the workspace lints with `-D warnings`, and both hash
  diffs equal.
- **D1's costs, with certify in the tree** (WSL, 48 threads, 2026-09-27): a clean `cargo check -p
  rustyecon-gui` took 16.1 s, 0.81 GB of target and 0.87 GB peak RSS (GUI.md: 14.9 s, 0.76 GB,
  0.88 GB without it); the warm check after touching `crates/engine/src/lib.rs` took 2.51, 2.57
  and 2.55 s, median 2.55 s. The non-blocking check passes.
- **The recount** (`cargo tree -e normal`, root included): 274 crates on Linux, 187 on Windows,
  125 on wasm32; 281, 196 and 129 with build dependencies (GUI.md §3.1: 268, 181, 120 and 275,
  190, 124 without certify). **Licences**: all permissive or with a permissive choice (18 under
  Unicode-3.0, 2 BSL-1.0 on Windows, `self_cell` Apache-2.0 or GPL-2.0, `r-efi` MIT or Apache-2.0
  or LGPL-2.1); the default fonts' OFL-1.1 and Ubuntu Font Licence remain the exception.
- **The lockfile** was resolved offline from the cache: every existing entry kept, every crate of
  the GUI's closure at the spike's version, 100 packages to 473. One `cargo fetch` ran on each
  machine. C: had 114 GB free, so Windows' `CARGO_HOME` stayed on C:.
- **Checked by hand**: moved to G0.1's second part, above, where the panels and the smoke mode
  landed; it is still yours.
- **Not yet recorded**: the gate world's rerun time through `ThreadDriver` (it times
  `materialise` too, so it waits for G0.2). CPU per frame is recorded with the second part.

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
Not pushed:
- the local `reboot` at `16eb728` (S2.1–S2.6, Phase 1's P1.2–P1.7, the `g0` merge, the
  many-markets probe, P2.1.1–P2.1.4, the `demo-world` merge and the `phase1` merge);
- this merge, on `merge-og-g1`;
- `oracle-goods` (P1g.1–P1g.7) and `g1` (G1.1–G1.11), whose work is in this merge;
- the local branches `merge-p1`, `phase1`, `merge-demo`, `demo-world`, `phase2-markets`,
  `merge-g0`, `g0`, `phase0-s2`, `phase2-probe`, `spine-eyeball` and `reboot-phase0`, whose work
  is in `reboot`.

## Decisions — veto window (your one-word calls)

The first nine record how your rulings and the standing rules were carried out; 10–21 were made
while building. 22–34 are the GUI's D1–D13 (A14; [docs/GUI.md](docs/GUI.md) says where each is
carried out): each stands unless vetoed before G0. D10's window closed with session 2, which
built its items. 41–58 are session 2's. 59–75 are Phase 1's units 1b and 1c.
76–117 are G0's, numbered 59–100 on `g0` and renumbered after Phase 1's when `g0` was merged
(n became n + 17). 135–178 are Phase 1's units 1d, 1e and 1f, numbered 76–119 on `phase1` and
renumbered after the demo's when `phase1` was merged (n became n + 59). 179–190 are unit 1g's
(track 1g's range is 179–199) and 200–219 G1's, numbered apart on their branches and merged
together without renumbering.

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

Decisions 76–81 were made while building G0.1's first part, where GUI.md was silent or wrong
against the code as it stood after S2.6. GUI.md's G0.1 amendment carries each; all are open to
veto.

76. **The GUI's gate is `scripts/gui.sh`,** beside `scripts/gate.sh` (decision 20), not
    `ci/gui.sh`. It tests in release, as the engine's gate does, and checks by name that each
    test G0's gate names ran and passed.
77. **The Runner takes the build and hands out the world.** `Runner::new(build, sink, wake)`
    mints each run key and ring checkpoint; `Obs::Loaded` carries the run key and the `World`;
    `Obs::Running` is new; `Obs::Refused` carries a `Refusal` (a tape that does not load, ring
    bytes that do not decode, a refused resume). A refused ring resume reruns from genesis.
78. **The store holds the run's status, and vm/ reads run types only.** No builder takes the
    model, so vm/ moves into observe with the Runner at G2 (D13); a scan holds run/ and vm/ free
    of the model, files, threads and clocks. `drive::Host` carries out the model's effects, so
    tests drive the whole seam without egui.
79. **The GUI shares the cli's build script** (`build = "../cli/build.rs"`), so a GUI run and a
    cli run of one checkout name the same build, by one definition.
80. **The session** lives in `$RUSTYECON_GUI_DIR`, else the platform's configuration directory.
    A new one plots every price and has the breakpoint on error; `layout.ron` is saved when the
    window closes; a file that does not read is set aside, never written over.
81. **The engine gate's GUI check runs on Linux only** (D1 says WSL), in
    `$CARGO_TARGET_DIR-gui`, so Windows engine steps build no GUI.

Decisions 82–89 were made while building G0.1's second part, the panels, where GUI.md was
silent or wrong against the code. GUI.md's amendment for that part carries each; all are open to
veto.

82. **The cursor is a report tick.** A panel reads the report of the cursor's tick and the state
    it left. Live follows the last tick that ran.
83. **The plots stack one panel per unit,** so every y axis names its unit and no two units
    share one. Overlay, difference and ratio against a parent wait for G0.2's branches.
84. **"Run until" reads a tick or a date.** A tick is a state tick, as the cli's `--until`
    takes it. A date runs through the tick it falls in, so an event of that date has fired.
85. **The registry's current value is the run's record.** Each use's per-tick value at it comes
    from `ClockMethod::per_tick`, the function `engine::registry` calls, and the copied basis
    from `FiredEvent.source`.
86. **The model asks for the actor inspector's snapshot** when an actor is selected and the run
    is not running, once per state tick. `Obs::Loaded` carries the tape, and a `Snapshot` its
    lots.
87. **Rationing onsets are logged once per class line per load,** compared exactly. One line per
    episode flooded the log: 39,990 lines on appb's 20,000 ticks, one-ulp shortfalls at rest,
    and 378 on the gate. A fill tolerance is certify's registered `rationed_below`, not the
    log's.
88. **The GUI depends on core for `num` alone,** for the inspector's ln(p′/p), and a scan holds
    it there. Alternative: the engine re-exports `num`, which is an engine change.
89. **The smoke mode** is `rustyecon-gui --smoke UNTIL TAPE`, at ten model years a second, with
    running and paused frames reported apart. D2's "no log axes" stays for G1: each of appb's
    prices has its own panel and scale.

Decisions 90–96 were made while fixing what G0.1's verification found. GUI.md's amendment for
those fixes carries each; all are open to veto.

90. **A poisoned run's own state is never read.** A snapshot of its current tick is rebuilt on
    a scratch `Sim` from the ring. Alternative: refuse the snapshot, and show no lots.
91. **Plots are keys kept across tapes.** A run opens with every price plotted when the
    session's plots name nothing of its world; another world's keys stay in the session, out of
    the stack. Alternative: plots per base, which changes `session.ron`'s format.
92. **A log line's tick is a report tick**, as the cursor's is; a line about a state says
    "state tick".
93. **`Obs::Ended`**: a driver reports once that its worker ended without a Stop. The run shows
    `Ended` and takes no command.
94. **The ledger line by key** is painted beside the engine's line, which stays as the cli
    prints it.
95. **Year gridlines** fall on the tick 1 January falls in, every 1, 5, 10, 50, 100, 500 or
    1,000 years; below two years the axis takes egui's marks, labelled by date.
96. **The core edge's guard is the scan.** At the next engine step the engine's prelude should
    re-export the pure `num` functions, and the GUI's edge to core then goes.

Decisions 97–108 were made while building G0.2, where GUI.md was silent or wrong against the
code after S2.6. GUI.md's amendment for G0.2 carries each; all are open to veto.

97. **`edit/` names core's raw schema, `Basis` and `Unit`,** and no other module does; the scan
    that held the GUI to `core::num` holds this too. An entry is written in those types, and
    none writes a state. Alternative: the engine re-exports them, an engine change, beside
    decision 96's `num`.
98. **A branch resumes whenever the world is kept.** A param only the schedule reads is outside
    `world_id` (decision 16), so removing an event with its source param, or copying a new value
    from a new param, resumes from the ring; only an edit of what the world reads reruns.
    `removal_only_branch_is_an_experiment` resumes at state tick 519, where GUI.md said genesis.
    D9's levels are then a schema question, not the GUI's way to resume a dial change.
99. **`plan` reads the parent's store,** each ring checkpoint carries the `prefix_id` its
    checkpoint stores, and a branch whose only agreeing checkpoint is genesis reruns.
100. **The lineage names the tape it describes** (its `tape_hash`) and has a format. A lineage
     beside a tape edited after it was saved is logged; the tape is still an experiment.
101. **Keys are new to every tape of the run tree, the staged edits and the runs closed this
     session.** The serial of `gui.<s>.<n>` is in `session.ron` (format 2), one more at each
     launch that reads it; a format-1 session is set aside, as any session that does not read.
102. **The stamp's date is today's in UTC,** read at launch and handed to the model.
103. **An act is typed as the tape writes it,** read by the tape's own parser; the raw pane shows
     the text it read, with the parser's line, column and a caret. Alternative: a form per act,
     which the agents' actions would multiply. The editor's form is the one way in; the
     registry's and the inspector's wait for G1.
104. **"Ledger changed"** compares tolerances by key, value and unit with the parent run's, or
     with the ancestor a reopened tape's lineage names when this session holds it; the manifest
     envelope gains `ledger_changed`.
105. **An export is five files at most** (`series.csv`, `manifest.ron`, `tape.ron`,
     `tape.lineage.ron`, `ancestor.ron`), and nothing is written over, by export or "Save tape
     as".
106. **Compare's "pinned series" are the plotted series;** the session pins entities. The plots'
     overlay, difference and ratio against a parent move to G1.
107. **Branches are not re-made at launch** (G1): a branch lives in memory until saved, so
     §5.1 item 7's refusal to re-apply edits to a changed base waits with it.
108. **A layout saved before a pane existed gains it** beside the inspector, rather than being
     set aside.

Decisions 109–113 were made while fixing what G0.2's verification found. GUI.md's amendment
for G0.2's verification fixes carries each; all are open to veto.

109. **"Ledger changed" has three answers:** yes, no, or unchecked with the reason. A reopened
     tape reads the ancestor its lineage names from its path, and keeps it only when its
     `tape_hash` is the lineage's. `GuiManifest.ledger_changed` becomes an `Option<bool>`.
     Alternative: say unchecked whenever the ancestor is not open, and read no file.
110. **Keys are new to every tape open in the session,** not only to the run tree, and a minted
     `n` only grows within a session. A set-aside session's serial is carried forward when it
     reads. Two windows on one `session.ron` can still share a serial. A lock file would fix
     that; it is left for G1.
111. **Compare reads a resumed parent's earlier states in the records it resumed from,** while
     they are open, and says when the records do not reach back far enough. Alternative: only
     say so, and never walk the records.
112. **A lineage that describes another tape is still saved beside a copy,** and is flagged in
     the export and the log. Refusing it, or writing it under another name, would let a tape
     whose only mark was its lineage reopen as a run (U3).
113. **The toolbar's goldens changed their field,** `ledger: NoParent` for `ledger_changed:
     false`, in a commit that did not retune the gate world. The view-model's shape changed;
     its values did not.

Decisions 114–117 were made at G0's close (G0.3). GUI.md's block "Closed at G0.3" carries
each; all are open to veto.

114. **G0 closes with its window item pending yours.** Every other item of GUI.md §9's gate is
     met and recorded, and nobody could look at a window. If the window fails your look, the
     fix is a G0.4 on `g0` before G1 starts. Alternative: keep G0 open until you look.
115. **`scripts/gui.sh` names the reducer's fifteen state-machine tests,** which §9 names as a
     group, so every test the gate names is checked by name: 42. Eight of the fifteen had
     killed no mutant by name; each now kills one of `reduce`.
116. **What the two re-checks left is carried, not fixed** (O20). The bounded verification is
     one pass, one fix round and one re-check, as in session 2, which carried O16. What is left
     is test gaps, two ways past a scan, and two edge cases of provenance; none is a gate
     item. G1 takes them first. Alternative: a third fix round before the close.
117. **The measurements are taken again on the finished crate,** beside G0.1's on its seams
     and G0.2's rerun time before its fixes; both sets stand. §9 does not say when in G0 to
     take them, and the finished crate is what an engine step's check builds.

Decisions 118–123 are the many-markets probe's (P2.1, 2026-09-27; docs/probe/MARKETS.md §6).
MARKETS-SPEC §9's frame decisions M1–M9 stand as the frame states them, open to veto with these.

118. **The many-markets probe closes: many markets GO, a loop NO-GO, no fallback.** I1, I2 and
     I3 are GO at C2m and 52 ticks a year; L2 and L3 are NO-GO at C2m and at C2L. A11's kill
     condition is not met: the verdict instances pass mode B at the first dials tried, and no
     Phase 2 configuration needs a loop. Alternative: read the loop's NO-GO as A11's failing
     mode B, and plan the fallback for economies with loops.
119. **C2m, C2 copied per role, is Phase 2 proper's default; C2L is not adopted** (MARKETS-SPEC
     Q1). C2L makes the loop's point locally stable, but no dial set tried near it is GO, and it
     was never run on I1–I3. Alternative: C2L's type rates and tilt as the default.
120. **No Phase 2 instance carries a loop of non-storable produced inputs; the loop goes to
     Phase 3** (O21). Its absorbing zero needs a stock to draw on (durable machines, or a
     storable service) or entry for a desk at zero coin; a keep rule that shares the shortfall
     does not close it. Alternative: a storable machine service in Phase 2 as a named variant
     (R6), under its own registration.
121. **The weekly tick is Phase 2 proper's default** (REPORT §7 Q5; O23). At 12 a year I2's
     point is unstable, and I1 takes a median of 90 years to reach tolerance, against 7 at 52 a
     year; 365 a year agrees with 52 in years, except for the loop. Alternative: monthly ticks
     with the dials restated per tick length, which needs a probe of its own.
122. **The report takes its task's names:** docs/probe/MARKETS.md, results/markets/ and
     figs/markets/, not MARKETS-SPEC §8's MARKETS-REPORT.md and results/markets-*.csv.
     MARKETS-RULES.md stays as registered; the reviews' corrections are in the report (the
     loop's NO-GO comes from the frame's restrictions, not from decision 67; 132 no-trade ticks,
     not 131). Alternative: amend MARKETS-RULES §7 as well.
123. **The families the time box left are carried, not run** (O22). The verdicts do not need
     them (MARKETS-SPEC §7.9), and Phase 2 proper's battery runs them first. Alternative: run
     them before closing the probe.

Decisions 124–134 were made on branch `demo-world` (D.1–D.5, 2026-09-27), where they were
numbered 118–128; the `demo-world` merge renumbered them after the many-markets probe's, from n
to n + 6.
GUI.md's blocks "Amended at D.3", "Amended at D.4" and "The map and lenses, brought forward",
and docs/demo/WORLD.md, carry each; all are open to veto.

124. **The map and lenses come forward over an illustrative world,** at your request, ahead of
     D8's trigger. The tape's name carries `[illustrative]` and every basis says so. G4 proper
     still opens on D8's trigger, with the research world. Alternative: wait for Phase 4.
125. **The atlas is the United Kingdom's 93 historic counties.** Northern Ireland comes from
     HCBP Definition B's UK file, on the same permissive terms as Great Britain. Yorkshire is
     three ridings from OpenStreetMap (ruling 7). Ross and Cromarty is one region, the City of
     York goes to the North Riding, and detached parts stay with their counties. The atlas is
     under the ODbL in `data/atlas/`, with its own LICENSE and ATTRIBUTION, and is bundled by
     `include_str!` with its digest checked. Alternative: Great Britain alone, 87 regions.
126. **Each county is one node running the probe's four roles at C2,** from its own oracle
     point (unit 1a), with one good, one machine type and no channels. The base county moves η,
     λ, b, h and N/T off Appendix B so that 1750 looks like 1750, with a labour share of 0.40
     and x\* of 0.70 (WORLD.md §2). Goods, machine types and carriers have reserved columns
     (WORLD.md §7; O27). Alternative: Appendix B's instance in every county.
127. **The history is dated `SetParam` steps.** A step is emitted when a county's composed
     value moves 1% in log, and each step is a schedule param outside `world_id`. The compiler
     holds every county date to 0.03 in log at the oracle, in prices and in quantities, every
     trailing year to 0.1, and the dials to C2 exactly, because abrupt change gives violent
     paths (O14). Alternative: wait for Phase 3's timelines, which `history.csv` can compile
     to when they land.
128. **The lens measures live in `rustyecon_worldgen::lens`** until `crates/observe` exists.
     The GUI gathers the readings and computes no measure. The cli calls none yet, which is
     U6's departure, recorded. Alternative: build observe first.
129. **Lens domains are fixed for the run and chosen without it.** They are registered in
     `lenses.csv` from the oracle's range with a margin, and held by
     `lens_domains_hold_the_oracle_range`. The palettes are neutral: viridis, and purple to
     orange through white. Alternative: domains from each run's range, which would change what
     a colour means between runs.
130. **A world of more than 16 nodes records the lean catalogue,** and series keep stretches
     at 8 bytes a point: 5,970 series, about 0.4 GB to 1901. Alternative: the whole catalogue,
     11,447 series.
131. **`certify` seals a tape whose name carries `[illustrative]` UNSCORED** whatever its
     criteria, and refuses a tape whose bases say illustrative once its name has lost the
     marker (O25). The scorecard's refusal waits for Phase 6. Alternative: leave it all to the
     scorecard.
132. **The map always paints the atlas's credit,** and both binaries print the atlas's LICENSE
     and ATTRIBUTION (`rustyecon licences`, `rustyecon-gui --licences`), as the ODbL
     attribution asks. Alternative: a link in an About view only.
133. **The screenshots are taken headlessly on Windows' software adapter** from a scratch
     crate. They are not gated, and no script remakes them. The workspace turns on no `wgpu`
     feature for egui_kittest, so its lockfile is unchanged. Alternative: a committed, ignored
     test with the feature on, which changes the lockfile and still could not run in WSL.
134. **What D.4's re-check left is carried, not fixed** (O26), as decision 116 carried G0's.
     These are test gaps in the county card, the painted legend, the mesh's other vertices and
     the credit's paint; the credit clipped on a narrow canvas; and the clock not held to 52
     ticks a year. None is a gate item, and the demo's next pass takes them first.
     Alternative: a second fix round before the close.

Decisions 135–178 are Phase 1's units 1d, 1e and 1f (2026-09-27), made on branch `phase1`,
where they were numbered 76–119; the `phase1` merge renumbered them after the demo's, from n to
n + 59,
and the specs cite them so. Most are the open questions of each spec (1d's §11 Q1–Q12, 1e's §11
Q1–Q15, 1f's §12 Q1–Q16), which the builds took as their drafts proposed; 160, 161 and 178 came
from the verifications. Each spec gives the reasoning. All are open to veto; a veto of one means
a change to the oracle and its goldens, not to any engine path. **Binds Phase 2** marks those
that decide what the agents must do or which instances Phase 2 may use; "Open — your calls"
lists them together.

135. **Worker types: one capability shape, an efficiency per type, and reserved tasks** (1d Q1).
     The oracle solves a world whose task cells scale one common human productivity by a type's
     efficiency, or reserve a cell to one type, but not one where the ranking of types changes
     across cells. Alternative: a schedule per type, with a threshold each and an I-dimensional
     solve whose uniqueness is not proved. **Binds Phase 2**: the agents' cells carry
     productivity by type in this form.
136. **The solved corners are reported inside `Interior`**, `margin` saying which (1d Q2), as 62
     and 69 do. Alternative: new regimes `Wall` and `AllHuman`, which change 1a's `Regime`.
137. **No equilibrium with land fully rented is `LaborShort`** (1d Q3), only where the excess
     demand changes side nowhere on the path; the edge of a reserved shortage is solved (P1.9),
     and the saturated knife edge is the junction. 1e resolves `LaborShort` economies on idle
     land. **Binds Phase 2**: its wall-regime instance is a solved wall, not a knife edge and not
     the edge of a reserved shortage, where a type's supply is vertical and its wage is set
     through the pool's clearing, which an agent market may not reach.
138. **Living costs are support baskets on the one basket** (1d Q4): s_i = (e^χ − 1)·ν_i·P_s.
     Alternative: a basket per type, which 1f deferred (173).
139. **Type hours are split by net supply** (1d Q5). Pooled types are perfect substitutes, so
     the model does not say which of them works which pool task. **Binds Phase 2**: compare
     per-type hours only through each type's supply at the oracle's wages, or in total.
140. **Machine recipes use pool labour only** (1d Q6). **Binds Phase 2**: the desks buy labour
     by type (PLAN §3.1), and the sources name trained machine builders (main.tex:584); a
     reserved type in machine recipes couples the walk and the (O, V) system, a 1d addendum.
141. **Bisection on bit patterns** (1d Q7) for a root in [0, 10⁻¹²], the corners' real wage and a
     walled tie's σ, at most 64 steps each; 1a's arithmetic bisection stays on [10⁻¹², 1], so
     the line nests bit for bit. Alternative: keep 1a's `NoInteriorAtZero` below 10⁻¹².
142. **Training is exogenous** (1d Q8): the N_i are inputs, and a walled type's premium is a
     scarcity price, not a cost of training recovered (SSRN p.18's industrial era; p.17's
     pre-industrial craftsman reads the other way).
143. **Viability is checked at the top of the line still** (1d Q9; decision 64), although the
     wall prices machines at any wage.
144. **A tie with a walled type is solved by bisection on σ** (1d Q10), with 1c's closed form
     where no type is walled; with interest its uniqueness within the switch is measured, not
     proved.
145. **The count covers the whole path; `MultipleEquilibria::switches` lists the line's switch
     points only** (1d Q11). Alternative: add the wall's switch wages to the error.
146. **1d's random draws** (1d Q12): the ranges and tallies of unit-1d.md §12 item 12.
147. **Parcels are efficiency units** (1e Q1): rent r·Q_z per acre, the worst idling first by
     convention. Alternative: a Ricardian working cost per acre, with a margin at positive rent.
     **Binds Phase 2**: a tape's parcel quality scales its service, and comparisons use totals,
     not which parcel idles.
148. **The commons is for exit only** (1e Q2). Alternative: production on open land.
149. **One participation rule for both exit forms**, SSRN eq 8 with the exit life's value (1e
     Q3). **Binds Phase 2**: under the default form a pop's participation share is
     F(ln((ν·P_s + w)/(ν·P_s + p_g·s(q)))), its s(q) at the rent its plot actually pays.
150. **Support stays positive** (1e Q4). Alternative: main.tex's pure form with ν = 0, which
     needs a regime for surplus labour at every wage and often has several equilibria.
151. **The exit good is one category** (1e Q5). Alternative: a bundle. **Binds Phase 2**: the
     tapes name it (food).
152. **Plots rented on enclosed land leave production, and the home account is in kind** (1e
     Q6). Alternatives: plots outside the land market (land counted twice), or rent paid in money
     from home goods sold.
153. **Idle land at zero rent with the pool's wage as numeraire; `NoMarket`; no `LaborShort`**
     (1e Q7). At zero rent a good made of land alone is free, and its real wage, wage floor and
     price shares are reported absent. **Binds Phase 2**: an idle-land instance is compared in
     wage units, and rests neither on a free good's absent outputs nor on an equilibrium at the
     walk's ceiling, which is refused (unit-1e.md §12 items 4 and 8).
154. **No reserved tasks with the priced form, for now** (1e Q8). **Binds Phase 2**: the eras'
     trained type at its wall under the default exit form needs a 1e addendum; until then such an
     economy takes the dependence form.
155. **The count scans where supply can fall** (1e Q9): exact where certified, resolved to
     `EXIT_SCAN` elsewhere, and multiple equilibria refused. Alternative: refuse uncertified
     economies, which refuses the race and the commons (Q, K: Appendix B's good carries no
     land). **Binds Phase 2**: the
     default form has several equilibria in a few per cent of economies with little support and
     land-heavy exits, so decision 70 now matters for the historical runs.
156. **The commons clears by a shadow rent that nobody receives** (1e Q10). Alternative:
     congestion that lowers each plot's yield.
157. **The enclosure tie is `Interior` with `enclosure` set** (1e Q11), as 62, 69 and 136.
158. **One land service** (1e Q12): SSRN A.1's vector of non-produced services is not in 1e.
     **Binds Phase 2 and later**: the tape's land classes need an addendum before a region has
     two scarce classes.
159. **1e's random draws** (1e Q13): the ranges and tallies of unit-1e.md §12 item 12.
160. **A free exit good at r = 0 is decided at the wall's end** (1e Q14, the verification):
     q = 1/b̃_g, its limit there. Alternative: §4.6 read literally, every such type on a plot at
     q = 0, which breaks the junction when the type is on its floor on the wall's last piece.
     **Binds Phase 2**: an idle-land instance whose exit good is land alone decides its plots at
     the wall's end; food, which embodies labour, is decided as before.
161. **Free plots on idle land are `ExitLand::Idle`** (1e Q15, the verification). Alternatives:
     `Commons`, or `Enclosed` by convention. **Binds Phase 2**: `Enclosed` means closed by
     price with no suitable land idle; `Idle`, plots free on idle enclosed land at r = 0.
162. **Where each tax is levied** (1f Q1): payroll on gross wages of every hour sold;
     consumption on final purchases at producer value (support and space included,
     intermediates and home output not); rent on market rent in money. Alternative: a tax on
     every purchase, which breaks the ledger. **Binds Phase 2**: the tapes' tax bases.
163. **Transfers in composites at consumer prices** (1f Q2). Alternative: in rent units, which
     R14 forbids for the historical runs.
164. **The budget closes by the owners' levy** (`RentRate`, 1f Q3). Alternative: every rate
     given and the uniform transfer the residual (`Dividend`). **Binds Phases 2 and 6**: a tape
     that gives every rate needs the Dividend closure or a deficit; one that gives relief scales
     in baskets (the poor law) is RentRate's.
165. **A transfer supplements the support by default, or replaces it** (1f Q4). **Binds Phase
     2**: the agents' participation rule reads a transfer as supplementing unless the tape says
     it replaces.
166. **The Dividend closure only where the budget does not depend on who works** (1f Q5).
     Alternative: an inner fixed point in d at each point, with its own uniqueness condition.
167. **Walled types only under RentRate and without in-work benefits** (1f Q6). **Binds Phase
     2**: the eras' trained type at its wall under a wage supplement (Speenhamland) needs an
     addendum.
168. **The path's start is evaluated; `SurplusLabour`** (1f Q7). **Binds Phase 2**: an agent
     economy with an in-work benefit large enough to overfill it has no oracle equilibrium to
     reach.
169. **The price-responsive basket is a CES over the categories with the basket's weights**,
     P = Z·M (1f Q8), the fixed basket kept as the default. Alternatives: a nested CES, or
     Stone-Geary around a subsistence basket, which PLAN §3.2 suggests for the agents. **Binds
     Phase 2**: the oracle and the agents share the consumption rule on any instance compared.
170. **CES only with the dependence form and without reserved hours** (1f Q9).
171. **A category free at the wall's end under CES leaves no idle stretch** (1f Q10).
172. **The land-share household reproduces A-joint in the generator only** (1f Q11), amending
     75: A-joint is `NotViable` in the oracle (D(1) = 0 exactly), which keeps 64, and its viable
     neighbours AJ1 and AJW are the goldens; check_dynamics' ρ > 0 targets also need external
     finance. **Binds Phase 3**: the dynamics thread's steady states need both.
173. **A basket per worker type is deferred** (1f Q12). **Binds Phase 2**: the 1750-like
     instance, with owners buying domestic service, needs it, or takes the common basket in both
     the oracle and the agents.
174. **The rent base is market rent in money; κ keeps 1e's definition** (1f Q13), so eq 16's τ_R
     is 1/κ only where no plot is rented.
175. **`Eq1f::base` keeps 1e's accounts without a government, labelled** (1f Q14).
     Alternative: recompute them with the government, which breaks `base`'s identity with 1e.
176. **1f's random draws** (1f Q15): the ranges and tallies of unit-1f.md §14 item 11.
177. **No government purchases, deficits or taxes on interest** (1f Q16). **Binds Phases 6–8**:
     wars and debt.
178. **A CES basket's numerics** (the verification): the power mean's direct sum where
     1 + S < 1/2, the corners bisected in v, a point beyond every double read +∞. No
     fixed-basket result changes.

Decisions 179-190 are unit 1g's (2026-09-27, branch `oracle-goods`; track 1g's range is 179-199,
track G1's 200-219), the open questions of its spec (docs/unit-1g.md §11), which the build took as
proposed. All are open to veto; a veto of one means a change to the oracle and its goldens, not to
any engine path.

179. **Decision 67 reworded (D-G1)** (1g Q1): machines are durable goods built from and run on
     goods; machine recipes use materials, machine goods, machine hours, labour and land, never
     a category (E1), which keeps 67's mathematics (x-free machine totals, closed-form switches,
     no type returning). Alternative: GOODS-CHAIN's G5, machines built from categories. **Binds
     Phase 2**: the goods chain's instances keep E1.
180. **D-G10: productivity and the chain to land per period** (1g Q2), amending 71: I − A^q a
     nonsingular M-matrix with A^q = A^op + Δ·A^I, and the chain to land a pattern found by
     reachability. It accepts every economy 71's rule accepted, with every result bit for bit.
     Alternative: the chain to land by b̃^q > 0 in f64, which can refuse a few economies 1c
     accepts.
181. **The mapping is the oracle's, per period** (1g Q3): `GoodsChain`, keyed by strings; the tape
     builder converts yearly δ, ρ and J with core's `Clock`. Alternative: in worldgen, on a tape
     schema that does not exist yet (O32).
182. **The embedding keeps every good's row** (1g Q4): materials and machine goods as flow types,
     so every good's price and output is an output of the equilibrium. Alternative: the fold,
     one type per machine, which loses the goods' prices.
183. **A machine good's unit is its stock, with κ hours a period** (1g Q5): a head, an engine; the
     hours type is built from 1/κ of it, and the good's price is per unit of stock.
184. **Plants on flow types, at any ρ, θ 1 as no plant** (1g Q6).
185. **A plant of any other recipe by a damped fixed point, every step interior** (1g Q7): half
     steps in ln(κ/ζ) to 1e-13 within 200 steps, refused otherwise. Alternative: Newton's method,
     which the generator uses at 70 digits.
186. **E2 enforced; G1 not approximated** (1g Q8): a category that uses a material, a machine good
     or hours directly is refused. Alternative: G1 now. **Binds Phase 2**: hearth coal and
     carters' fodder wait for G1.
187. **The illustrative counties** (1g Q9): A0 on Appendix B's (GOODS-CHAIN's rule A), the horse on
     a constructed one (N 12, T 10, h 1, χ_max 1/20). Never scored (R5).
188. **The weekly tick for the goldens** (1g Q10): δ and ρ by `Clock`'s formulas at 52 ticks a year,
     J in ticks (the horse's three years are 156).
189. **O28's cheap survivors get tests** (1g Q11): eight (h8); the rest are carried (O33).
190. **The test groups are h1-h9** (1g Q12), since "g" is 1a's.

200–219 are G1's (branch `g1`), numbered after 178 and apart from track 1g's 179–199.

200. **The engine re-exports `num` and the tape's raw schema, whole, and `Basis` and `Unit` in
     its prelude** (G1.1; STATE's next step 4, decisions 96 and 97 carried out). The frontend
     guard allows exactly these two modules of core, each only as itself; the GUI drops its
     edge to core. Alternative: a narrower `num` (the logs alone), which would need a list the
     guard keeps. *Amended at G1.11:* the GUI's scan holds, besides its sources, that neither
     its manifest (`package = "rustyecon-core"` under any key) nor the lockfile's list of its
     dependencies gives it core under another name.
201. **The lab needs no tape and drives no run, and its form is the panels' state** (G1.2): not
     the model's, which holds runs, and not the session's, so nothing of it persists and no
     intent reaches `reduce`. Its numbers are origin "oracle" (U3) and are fed to no run (U5).
     Alternative: the lab's instance in `session.ron`, which a later stage can add.
202. **A point's fields are read from the oracle's own `Debug`** (G1.2), not from a
     `Point::outputs()` in the oracle (§7.3): the derived `Debug` names every field and prints
     every float in round-trip digits, so the lab keeps no copy of the fields and reads them bit
     for bit (`a_points_fields_are_the_oracles_doubles`), and the oracle is untouched while
     track 1g edits it. Alternative: `outputs()` on each point type, which the next change to
     the oracle can add; the lab then reads that instead.
203. **The presets are the goldens' instances, transcribed from the oracle's gate tests**
     (G1.2): 16, two to four a unit, each named by its goldens file and prefix. A copy of the
     instances, not of any equation, held to its goldens by `lab_presets_solve_to_their_goldens`.
204. **An output pairs with the golden of its own name** (G1.2): `PREFIX_KEY` and
     `PUB_PREFIX_KEY`, the key upper-cased with `.` read as `_`; a generator's golden agrees
     within 1e-12 relative, a published one within 5e-6 absolute (the oracle gate's `FULL` and
     `PUBLISHED`). A golden named otherwise is listed apart, never guessed at, and an instance
     edited from its preset pairs with none. *Amended at G1.11:* `build_beside` pairs with
     goldens it is handed, so a test holds a disagreement to being shown as one, either side of
     each bar.
205. **Every number of an instance is a knob; its structure is the preset's** (G1.2): counts,
     access, the exit form's kind, the budget's rule and the basket's kind are not edited. A
     whole-number knob reads `3` or `3.0`.
206. **Sweeps run on the UI thread when asked for, and the view-model includes the solves**
     (G1.2). The gate's measurement is G1 over N, 200 points: well under 16 ms. The heavier units
     take longer ("Where things stand" records each) and can hold a frame; a worker waits for
     need.
207. **The explainer recomputes from the record, and compares bit for bit** (G1.3): p, S, D at
     the tick and the rate's value at the tick converted by the good's own site, through
     markets' `imbalance` and `next_price`; beside the run's recorded next price where the
     catalogue records one, and "not recorded" under the lean catalogue.
208. **The waterfall is Σ k·x, and the residual is shown, not explained away** (G1.3): over the
     ticks before the cursor's, a bar a year from two years up; the residual is what the rule's
     steps do not give (held one-sided ticks, an event's price move, rounding). An event is
     flagged when it scales this price or sets its rate. Alternative: the rule's own log steps,
     ln(next/p), which would hide the holds. *Amended at G1.11:* under `Ratio`, which ignores
     k and holds a one-sided market by the rule, the steps are its own, ln(D/S) where both
     sides posted, and a rate set flags nothing; the rule is read by the name the tape gives
     it, since the engine's prelude does not name its type (alternative: `PriceRule` in the
     prelude, an engine change).
209. **Breakpoints on an event by key and on a date** (G1.4): after the tick an event fires in,
     every occurrence; after the tick a date falls in, once, not in a run begun past it; an
     event before a date in one tick; a key the world lacks is kept (U7) and the log says so.
     `PauseReason` and `RunStatus` lose `Copy`.
210. **`session.ron` format 3, reading format 2** (G1.4): the watchlist and the log scales, and
     the breakpoints' new kinds; a format-2 file reads with none of them. A G0 build sets a
     format-3 file aside. *Amended at G1.11:* a format-3 file must have `watch` and `log_axes`,
     as it must `speed`; a format-2 file may have neither, nor an event or date breakpoint,
     since G0 wrote none.
211. **The watchlist opens the outliner** (G1.4; §4's "Outliner: G1 watchlist"): by key, the
     value at the cursor and the change from the tick before.
212. **Log axes per unit, as ln v through `num`, labelled by v** (G1.4): a value v ≤ 0 splits
     the line; D2's "no log axes" is lifted. `every_drawn_vertex_is_recorded` holds a log
     panel's vertices to ln of the record.
213. **Snapshots say never citable in the picture and in the file** (G1.5): a banner painted
     while the picture is taken, tEXt chunks, `snapshots/` beside the session, never over a file;
     `png =0.18.1`, already in the lockfile. With no session directory there is nowhere to write.
214. **Other charts paint in colours of their own** (G1.4), so the plots' tests, which find the
     plots by their palette and their axes, stay exact. The lab's field over x names its field
     on a rotated y label (this said none until G1.11); it is drawn on the Lab tab, which the
     default layout shows in place of the plots, never beside them. *Amended at G1.11:* they
     record what they lend egui (`ui::charts`), and the scripts hold it to the view-models and
     to what was painted.
215. **O26's map items, by tests that read what was painted** (G1.6): the credit wraps; the eight
     surviving mutants are killed. The clock and two of the three layout notes are O39's; the
     third, the chip on a narrow window, is closed (G1.9, and G1.11 on the demo world).
216. **O20's scan escapes closed** (G1.7): a group's branches each have their root, and a root's
     alias is a root. The rest of O20 stays (O36). *Amended at G1.11:* an `extern crate` alias
     is a root too, and a `path` attribute and `include!` are refused in the egui-free and pure
     modules.
217. **What G0 moved to G1 waits for a G1 second part** (O36): overlay, difference and ratio
     against a parent; re-making branches at launch; the registry's and the inspector's ways into
     the editor; a lock on `session.ron`. §9's list came first, as the task set it.
218. **G1's bounded verification ran once, and each finding is fixed by a test that fails without
     its fix** (G1.11; O37). At G1.10 none had run. One adversarial pass found six major and
     four minor issues and no wrong number; the fixes' mutants are all killed. A re-check of
     the fixes, as G0's steps had, is O37's.
219. **The window's p90 frame under 16 ms stays yours, checked by hand** (§9), with the smoke
     mode's figures beside it.

Decisions 220–239 are the stocks probe's (P2.2a, 2026-09-28, branch `phase2-goods`;
docs/probe/HORSES.md §6). 220–233 are HORSES-SPEC §9's, taken as the frame registered them before
any run, each with what the run found; 234–239 are the report's. GOODS-CHAIN's D-G2 to D-G8,
D-G11 and D-G13 to D-G15 were taken as proposed, not ruled; they stay yours (next step 6).
*Amended at P2.3.0:* D-G11 was ruled by Claude as 285, and D-G13 to D-G15 as 391–393.

220. **The instances** (HORSES-SPEC §1.1): A0, Appendix B's county under GOODS-CHAIN's rule A, at δ
     10% and 8% a year and ω ½ and 1, J_b 1 tick, ρ 0, carries the verdict (H1–H4); δ 4% (P2),
     the chain's ω 0.85, v1's base county and fodder's rate at 1.3 are families. Alternative: v1's
     base county as the verdict county (its b × 2 target is unfunded, O43).
221. **CHAIN's horse is not a P2.2a instance**: it has a loop (P2.2b's), a three-year build (E2,
     Phase 3) and an unfunded county with or without its loop (O41). Alternative: a funded county
     for the loop-cut horse as a fifth verdict instance.
222. **The good desk is P2.0's `GoodDesk` with `assign: ExPost` (D-G6), the fodder desk P2.1's
     `TypeDesk`**: no new code for either. The fidelity review found the verdict does not lean
     on D-G6 (H1–H4 converge 93/93 under planned assignment too), though the paths do (HORSES
     §4). Alternative: a category desk with ex-post assignment now, which v2a.2 needs anyway
     (O42).
223. **Three new kinds, `Maker` (M2), `CapacityDesk` (M3 wet) and `OwnerDesk` (M1)**, and three
     `ActorState` variants appended after `MachDesk`. M1 serves R1a and P7 only; its paths are not
     scored (D-G4). Alternative: M1 unbuilt, R1 checked on the maker's flow path alone.
224. **Wear is an upkeep burn of δ times a recorded stock** (`Depreciation`): the capacity desk's
     phase-start holding, the maker's and the owner desk's serving stock. Conserved within 4e-16 a
     tick (measurement review). Alternative: δ times the holding at 5b, which puts the capacity
     desk on M1's timing and was not run in the mirror.
225. **The δ = 1 flow path is chosen by the stock good's life** (`Ticks(1)`), with δ = 1, no cover
     and no running recipe checked at load; the new kinds then run `GoodDesk`'s and `TypeDesk`'s
     code, so R1a holds by construction, and its tests check plumbing, not the stock formulas
     (measurement review). Alternative: a spec field naming the path.
226. **Fodder's price rate under C2g is 5.2 a year**, C2L's type rate. Alternative: 1.3, C2m's
     (F7–F10: GO, with deeper dips after a cost rise, −0.98 and −0.86 in log against −0.56).
227. **κ is a `FlowPerYear`** (52 horse-days a head a year at A0), and the build per head is
     written by the generator per tick length. Alternative: one horse-day a head a tick at every
     tick length, with per-tick flows unscaled.
228. **L adds 3·T6 from the slowest local mode** to P2.1's rule (D-G11). In the event the engine's
     elasticity term, land's τ under C2g, set L at 75,000–258,000 ticks. Alternative: P2.1's rule
     alone.
229. **Tier 3S** (stocks and coins × 0.5 and × 2) **is in the verdict**, its start distance read
     over the first year. Alternative: stocks as a family, as P2.1 had them.
230. **The horse market is idle, not dead, when its orders stop.** Alternative: dead, which would
     class every glut run DEAD by design of the order rule.
231. **The maker's band stays** (D-G5), although finished heads offered in full are stable under
     the capacity desk (P3 (v), held in the engine); b_K 0 is a variant. Alternative: drop the band
     for M3 instances.
232. **The harness solves M3's hours at J = J_b + 1 = 2** (D-G8 item 4): at ρ 0 every price and
     quantity is J 1's bit for bit. Alternative: J 1 until ρ > 0.
233. **GOODS-CHAIN's P1–P8 are re-predicted for v2a.1's economy with its fodder market**, each
     against its registered value; P3 (i), P3 (vi), P4 and P6 are carried, not run. Alternative:
     GOODS-CHAIN's values as registered.
234. **The stocks probe closes: GO for v2a.1, read narrower** (HORSES §0, §5). H1–H4 are GO at
     C2g and 52 ticks a year (1,980 runs). Stocks give about a quarter of the claimed softening of
     a cost shock's trough and cost 2.3–2.8 times a like-for-like flow county's years; at ω 1 the
     GO passes through horse-price falls of 96–99.7%; P1 at ω 1 and P2 missed their kick rates on
     the fast side, and the slowest mode is wear, 1 − δ. Alternative: read the ω 1 instances as
     LOCAL until the idle market has its remedy (O47).
235. **O14 comparisons are made like for like.** Each stocks or loops probe registers a flow
     control at its own dials and assignment beside P2.0's and P2.1's numbers, as the fidelity
     review did (results/horses/o14_like_for_like.csv). Alternative: compare with I0 as
     registered, naming what else moved.
236. **The idle machine market's remedy comes before the loops and the goods-chain 1750-like
     instance** (O47; GOODS-CHAIN open question 4). heads × 10 diverges at every instance, the ω 1
     GO passes through the same fall, and a switch of technique idles a machine market by
     construction. The candidate is the maker's reservation at replacement cost, a step R3 allows,
     chosen by a mirror scan and run on heads × 10, P8 and the ω 1 battery. Alternative: wait for
     P2.1's fix for one-sided runaways.
237. **Weekly ticks are the floor for instances with machine stocks** (O45; GOODS-CHAIN open
     question 6): 12 a year fails at every instance, 24 a year at ω 1. Decision 121 makes weekly
     the default; this makes it a floor. Alternative: dials restated per tick length, with a probe
     of their own.
238. **M6's load check is narrowed to the durable good before any storable running good** (O48).
     The capacity and owner desks net only their durable good, so a storable good they buy as a
     running input is refused until they net it. Not changed here: no v2a.1 tape has one.
     Alternative: build netting of running goods into M3 now.
239. **The report takes its task's names**, docs/probe/HORSES.md, results/horses/ and
     figs/horses/, not HORSES-SPEC §8's STOCKS.md. HORSES-RULES.md stays as registered; the
     reviews' corrections are in the report (the count, R1a by construction, O14 like for like,
     the 1 − δ mode). Alternative: amend HORSES-RULES as well.

240–244 are the loop stage's groundwork (L0, branch `phase2-loops`, 2026-09-29).

240. **M6 is narrowed to the durable good** (L0.1; O48, carrying out decision 238). The capacity
     and owner desks' running goods and labour are checked as every other kind's buys are: a
     good that lives more than one tick is bought only as the durable good a capacity or owner
     desk holds. Every committed tape loads; the fidelity review's tape is refused at
     `actors[desk.capacity].spec.running.goods`. Alternative: build netting of running goods
     into M3 and M1 now (238's alternative), which no v2a.1 tape needs.
241. **M5 is a load check on every role that offers a good in full** (L0.1): the households'
     labour and land, the output of the good, machine, category, type and owner desks, and the
     capacity desk's hours each live at most one tick (`Instant` or one tick), the line M6 draws.
     No role yet offers I/(1 + b_G). Alternative: check the type desk alone, the seller that
     v2a.4 and v2a.6 would give a storable good; or build the share-rule seller now.
242. **The maker is not held to M5.** It offers its durable good under its cover, M2's band
     (D-G5), and `cover: None` stays loadable as P3 (v)'s control, which HORSES found stable
     under the capacity desk (decision 231); a test holds the exemption. Alternative: refuse a
     stock-path maker without a cover, which retires that control.
243. **M5 runs after M6, over every actor**, so a tape an older check refuses reports the older
     error, as M6 runs after each actor's own checks (HORSES-RULES §8 item 5). On the review's
     tape both would fire, and M6 reports first, at the capacity desk. Alternative: M6 and M5 per
     actor, whose first error would follow the order the actors are declared in.
244. **HORSES-RULES.md stays as registered** (decision 239): ENGINE.md (amended at L0.1) and
     TAPE.md say what the checks are now. Alternative: amend HORSES-RULES' SG7 paragraph as well.

245–250 are IDLE-SPEC §9's 240–245 (the scan numbered them before L0.1 took 240–244), taken as the
registration froze them (L0.3) and borne out by the run (L0.6); 251–253 are the build's and the
run's.

245. **O47's remedy is the maker's reservation** (IDLE-SPEC's 240; L0.3–L0.6). It is candidate
     (a), a step on M2's offer, chosen by the mirror scan and registered before it was built. On
     the engine every heads × 10 run and P8 converges, and no class changes in P2.2a's 1,980
     battery runs or its families. Whatever the bids, the idle horse price falls at most one
     step below ψ times the replacement cost at the maker's last offer (reworded at L0.7, after
     the engine review): costs keep moving while the maker withholds, so its markup at current
     costs can go lower, 0.144–0.206 at heads × 10. The lowest horse price there is 0.135–0.193
     of target. Alternative: the world's one-sided rule `Hold` (candidate (c)), with no code,
     which bounds the fall only while no bid stands.
246. **ψ = 0.25, not 1** (IDLE-SPEC's 241). At ψ 1 the engine orbits from mode A at all six
     instances run: its genesis markup is 1 to an ulp, so rounding puts it on the step.
     Alternative: ψ 0.4, which the scan found gives the same classes, 127 acting runs and a lowest
     price of 0.30.
247. **The maker keeps its cash rule while it withholds** (IDLE-SPEC's 242; tilt 0).
     Alternative: tilt 1 with the reservation, which brings P8 to tolerance 2.4 times faster but
     moves every local root (O53).
248. **The reservation is refused at load on the flow path and under `Hold`** (IDLE-SPEC's 243).
     Alternative: allow both and document each as a market that is dead by construction.
249. **`reserve` is the one optional field of a stock kind** (IDLE-SPEC's 244). It is left out
     when absent in both the raw and the resolved spec (L0.5), so every committed tape keeps its
     text, `tape_hash`, `world_id` and hash stream. Alternative: a required field, with every
     horses tape regenerated with `reserve: None` and new hashes.
250. **The engine run was IDLE-SPEC §8's list** (IDLE-SPEC's 245), at P2.2a's registered L, with
     E8's ψ 1 runs at the mirror's lengths. Alternative: decision 236's list alone (heads × 10,
     P8 and the ω 1 battery).
251. **The registration came before the build** (L0.3 before L0.4). The trace diff came after
     both and before any scored run. The spec asked for build, trace diff, registration; this
     order is stricter, since nothing of the rule on the engine was seen before registering.
     Alternative: the spec's order, as P2.2a registered after its build.
252. **The param is `reserve.<maker>`** (`reserve.maker`), as `cover.<maker>` is. The spec's
     "(at A0 `reserve.desk.maker`)" was read as a slip at registration. Alternative:
     `reserve.desk.maker`.
253. **The reservation is on in every later stocks or loops instance by default**, at ψ 0.25:
     P2.2b, the goods-chain 1750-like instance and the demo's second pass. Each frame registers it
     as a dial, and P2.2a's tapes keep it off (R1). Alternative: opt in per registration, each
     frame arguing for it.

254–259 are FUNDED's 240–245 (`docs/probe/loops/FUNDED.md` §10, which numbered them before L0.1
took 240; its amendment FUNDED-A1 §A1.4 maps them), taken as proposed; 254's reason is FUNDED-A1's.

254. **The funded rule-B county is `chain8`: 1g's HORSE county at N 8** (FUNDED's 240; O41
     closed). Only N moves, 12 → 8: T 10, h 1, χ_max 1/20 and γ stay 1g's, and CHAIN's horse is
     unchanged. Coverage T/(N·P_s) is 1.215–1.240 at every P2.2a cost target and δ 4–10%, rent
     still covers the transfer by 20.9% at the worst genesis displacement, and the point is
     unique and interior with the loop live. The reason over its alternative is headroom on
     labour supply, weighed over the run as well as at genesis (FUNDED-A1 §A1.1): over LB1's 125
     battery runs chain8 reaches the bound in 5 runs and 649 ticks, none at genesis. Alternative:
     N 8 with χ_max 1/30, exactly 1g's HORSE point, which reaches it in 9 runs and 1,189 ticks, 4
     of them from genesis.
255. **P2.2b's verdict δ is CHAIN's 8% a year**, J_b 1 tick, M3's hours at J 2 (FUNDED's 241).
     δ 10% and 4% are further verdict instances in the loop step (LB2, LB3), each its own oracle
     point. Alternative: δ 10% as the verdict δ, P2.2a's H1.
256. **Rule B's cost shock b × f multiplies every land coefficient**, fodder's farmland and the
     head's pasture together (FUNDED's 242). Alternative: fodder's land alone.
257. **The like-for-like flow control is the operating form** (FUNDED's 243): two flow types on
     the loop, a head's wear folded into the horse-day at δ/κ heads, run by a flow desk; M1's
     δ = 1 form is the same point for the owner path. Alternative: the δ = 1 form for both.
258. **The capacity desk's plant readouts are tabled under both bundles** (FUNDED's 244), the
     running recipe and the full one; the mirror's loop step chose the running recipe (261).
     Alternative: choose the running recipe in FUNDED.
259. **Rule B's recipes are per unit made at every tick length** (FUNDED's 245): only N, T, κ
     and δ go through `Clock`, so rule B's tape params need no per-tick restatement. Untested at
     other tick lengths. Alternative: restate per tick as rule A does.

260–271 are LOOP-SPEC §9's L1–L12 (`docs/probe/loops/LOOP-SPEC.md`), taken as proposed and
registered with the carry at L0.8 (LOOP-SPEC-A1).

260. **The plant sits on every loop desk: the fodder desk, the capacity desk and the maker**
     (L1). CAPACITY's plant, θ 0.8, plant δ 10% a year, s1 bundles of the desk's own recipe,
     ordered by M3's rule at s_K = 2δ_p, installed next tick. Alternative: the fodder and capacity
     desks only (LN2), which orbits under the reservation in 16 of 25 Tier-3 runs.
261. **On the capacity desk the plant sits beside the horses, built from the running recipe**
     (L2; the plant's composition with M3). Horse-days are min(κ·H, P^(1−θ)·z^θ); use stays M3's
     at the running cost, the targets use O_f, which is O at s1. Alternative: FUNDED's full-recipe
     reading, which buys heads twice and fits only the flow form.
262. **The capacity plant's target is the herd's plant, κ_p·κ·K\*** (L3). Alternative: CAPACITY's
     kr·z at the bundles in use (LF6): 0.9996 a tick, and most runs orbit.
263. **M3's horse stock is not the plant** (L4): it is a Leontief cap that damps an upswing and
     passes a shortfall one for one. Alternative: the horses as a Cobb-Douglas factor, which would
     move the long run 2% off the oracle at b × 2 and needs a scale-dependent oracle.
264. **No damper beyond the plant and the reservation for the pass-through** (L5; reworded at
     P2.2b.4 after the fidelity review, decision 307). The capacity plant takes dead horse-day
     ticks after r × 2 from 146–150 to 1–2; fodder's 156–261 are the county's, not the loop's
     (the flow control has them too). But the run converges only with the maker's reservation
     on: at ψ 0 (LF3) LB1 r × 2 runs away at tick 170. A 13-week fodder store is a family (LF4),
     a 4-week one a negative control. Alternative: the 13-week store in the default, which lifts
     the b × 2 trough from 0.70 to 0.96 but needs O51's roles first.
265. **The reservation stays on in every loop run** (L6; decision 253). Without it rule B's
     gluts run away at ticks 155–173. Alternative: the world's `Hold`, not run in the loop step.
266. **The instances** (L7): LB1–LB3 (horse δ 8%, 10%, 4%) carry the verdict; LF1–LF8 and LC1 are
     families; LW1–LW3 the like-for-like flow controls; LN1–LN9, LF6 and LW0 the negative
     controls. Alternative: LB1 alone as the verdict instance.
267. **L follows HORSES-RULES §6.7's rule** (L8): the good's τ of about 1,000 ticks sets it at
     202,000–232,000 ticks. Alternative: L from land's τ, 56,000 ticks.
268. **Weekly ticks stay the floor for the loop** (L9; decision 237): at 12 a year LB1 grows
     1.0053 a tick. Alternative: 24 a year, which passes 40/40.
269. **CHAIN's maker from bought fodder is the tested maker from bought inputs** (L10). S1
     without pumping waits for a funded steam county. Alternative: frame S1 on M3's unfunded
     county.
270. **Fodder's price rate stays C2g's 5.2 a year** (L11), 1.3 a family. Alternative: 1.3 as rule
     B's default; it does not replace the plants (LN8, LN9).
271. **P2.2b's O14 reference is the flow control with the same plants** (L12; decision 235).
     Alternative: the flow control without plants (LW0), which is itself NO-GO.

272 on are L0.7–L0.9's (the fix round after the reviews, the loop registration and the report).

272. **The loop mirror carries genesis, and the loop step is registered again from it**
     (LOOP-SPEC-A1; O57; the mirror review's major finding). `lm_carry.py` is the registered
     `lm_mirror.py` with the engine's genesis carry: unsold genesis lots of a one-tick good are
     offered again at tick 1, and a buyer's unused genesis purchases are used at tick 1. With
     every new layer off it is the trace diff's `i_carry.py` bit for bit, and from tick 2 it is
     `lm_mirror.py` bit for bit. The carry changes no class at any verdict, family or
     flow-control instance, but it moves registered values past §8's tolerances in 9–12 runs per
     verdict instance at L, the one-tick goods' stock displacements most (ticks to tolerance up
     11–38%, the lowest horse price down 6–46%). So A1's §A1.5 replaces §8 for P2.2b's scoring.
     Alternative: keep §8 and treat the carry's differences as explained misses, as L0.6 did for
     H2 r × 2, which would leave those runs outside the tolerances before P2.2b is built.
273. **A planted desk takes carried goods into its bundles as it takes any it holds**
     (LOOP-SPEC-A1 §A1.1): B = max_scale over the bundle's goods held, split between running and
     build in proportion to its plan (LOOP-SPEC §2.2). It is a convention the engine's build must
     follow for E0 to hold at tick 1. Alternative: carried goods to the running bundle only (the
     review's `lm_carry`), which leaves a plant order's unused lots idle at tick 1.
274. **Dead ticks are scored per market; labour's are reported, not scored** (LOOP-SPEC-A1 §A1.5;
     O66). Fodder, horse-days, the good and land each against their registered largest count per
     tier, at the harness's bar (half the oracle volume). Labour's and the union's are reported.
     After r × 2 and N(2) chain8's labour sits within 2% of the bar, so its count flips between 0
     and about 240 on rounding. Alternative: labour's bar at 0.45 on this county, which moves the
     bar every probe since P2.0 has used.
275. **Every negative control has a registered engine outcome** (LOOP-SPEC-A1 E9): LN1–LN9, LF6
     and LW0, each with its classes by tier from the carry battery, at L. Alternative: the
     controls as mirror-only, scored by nobody.
276. **S1 with its pumping loop is out of P2.2b** (GOODS-CHAIN §6 step 5 names it with rule B's
     horse). M3's county is unfunded at S1's point, as it is for S1 without pumping (269, O62). It
     waits for a funded steam county, found and checked as FUNDED did the horse's. Alternative:
     frame it on M3's unfunded county, its transfer shortfall reported.
277. **Labour supply's bound is registered run by run, not avoided** (LOOP-SPEC-A1 E11;
     FUNDED-A1 §A1.1). At each of LB1–LB3 five runs put every head to work, for 18–273 ticks (r ×
     0.5, N(0.5), the workers' coin × 2, heads × 0.1, JB(2); peaks 1.03–2.13), none from genesis,
     and all converge; four or five at the flow controls. The engine is scored on the same runs.
     Alternative: a county that never reaches it, which would need N or χ_max moved far from 1g's
     point.
278. **The reservation's off is structural** (L0.7): the maker withholds only when ψ > 0.
     L0.4's `markup < ψ` withheld at ψ 0 wherever δ·a/κ > 1. Alternative: refuse δ·a/κ ≥ 1 at
     load, which a dated `SetParam` could still reach.
279. **The harness reads the maker's reservation through the rule's own code** (L0.7,
     `maker_reservation`), so it reads the tape's recipes, rule B's maker from bought fodder
     included. Alternative: keep `MakerCost` and add a test tying it to the engine's sell orders,
     which rule B would still have broken.
280. **The plant's dynamic lines have a second implementation before the engine build**
     (`lm_drs2.py`, written from LOOP-SPEC §2.2–§2.5 on the independent `lm_drs.py`): against
     `lm_mirror.py` within 4.8e-12 in log over 2,000 ticks in 110 runs at eight instances, and
     1.4e-11 over 6,000 (the map against itself one ulp apart: 6.4e-11). Alternative: CAPACITY's
     own mirror on a shared case, which has no horse holding.
281. **The loop step's registration is committed before P2.2b's build** (L0.8): LOOP-SPEC.md,
     FUNDED.md and instances.json byte for byte in `docs/probe/loops/`, their amendments beside
     them, and `docs/probe/results/loops/registration.md`. Alternative: keep them in scratch until
     P2.2b's frame, as IDLE-SPEC stayed.
282. **A spec fixed after registration gets a dated amendment with its own sha256**
     (IDLE-SPEC-A1, FUNDED-A1, LOOP-SPEC-A1); the registered file stays byte for byte and its
     sha256 keeps meaning what it meant. Alternative: a new version of each spec under a new
     sha256.
283. **The report of the remedy and M6 is `docs/probe/IDLE.md`**, in HORSES.md's shape, with the
     results where L0.6 put them. Alternative: a §8 of HORSES.md.

Decisions 284 and 285 are the two calls STATE's step 6 left to you, taken by Claude on
2026-09-29 on your word of 2026-09-26 ("I leave all those calls up to you") and 2026-09-27 ("You
can keep going beyond the gates"), so that P2.2b can run. Both are open to veto.

284. **Decision 120 is amended: a loop of produced inputs may enter Phase 2 when every loop desk
     holds a plant** (CAPACITY's y = K^(1−θ)·z^θ, built from its own recipe and worn at δ) and
     the instance passes P2.2b's registered battery in the engine. 120's reason stands: the
     absorbing zero needs a stock to draw on, and the plant is that stock. It holds for no other
     loop; a loop without plants stays in Phase 3. Alternative: 120 as written, with P2.2b run
     as a probe whose verdict waits on a ruling before any Phase 2 instance carries a loop.
     *Narrowed at P2.2b.4 by decision 308*: "P2.2b's registered battery" is undefined for a new
     instance, so each loop instance needs its own registration first.
285. **D-G11 is taken: machine stocks come forward from Phase 3 for the goods chain's
     instances** (GOODS-CHAIN §6, "What changes" item 2): stocks, geometric wear, the maker and
     its band, wet capacity desks at s_K = 2δ, the maker's reservation and plants, at J_b 1 tick
     and ρ 0, with capital's time an unscored departure (D-G14). Entry, build lags, ρ > 0,
     versions and scored path windows stay in Phase 3. PLAN §3.2's fixed free-entry capacity
     still holds for every instance that is not a goods chain. Alternative: machine stocks stay
     in Phase 3, and the goods chain's 1750-like instance waits for it.

286–296 are P2.2b.0's (`docs/probe/LOOPS-RULES.md` §12), taken by Claude on your standing word and
open to veto.

286. **The plant good is core's new untraded good**: `untraded: true`, `Indefinite`, no price
     rate, no market, no genesis price, and no order may name it. LOOP-SPEC §2.2 gives the plant
     no market, and core refuses a plain good without a price rate. Alternative: a plant market
     at price rate 0 that no one trades, with the kick set restricted to the instance's markets;
     certify's kick set kicks every market, and a kicked plant price never decays.
287. **Planted states nest the kind's state**: `PlantedType`, `PlantedMaker` and
     `PlantedCapacity`, each {desk, plant}, appended after `Owner`, with one `PlantState` {held,
     target, order, run, built}. Alternative: flat structs per kind.
288. **Budgets**: the planted TypeDesk and maker cut theirs from outlay + P_K·I, the capacity desk
     from its coin, as now. At I = 0 each is the plant-free chain bit for bit. Alternative: every
     planted desk from its coin, which parts by an ulp where the chain binds at the outlay and so
     breaks θ = 1's value-for-value.
289. **One burn per bundle good in produce**, the running and the build bundles together, the
     build at most what the bundles held leave, the capacity desk's used bundles at most its
     share. Alternative: two burns per good, which can overdraw by an ulp on a carried holding.
290. **P2.2b extends `probe::horses`**: loop ids, the flow control as a config, the land factor as
     the instance's b, the binaries `horses` and `horses-tape`, and `tapes/loops-<id>.ron` for
     LB1–LB3 and LW1–LW3. Alternative: a new module `probe::loops` with binaries of its own.
291. **LN5 and LF4 wait for O51** (LOOP-SPEC-A2). Alternative: build O51's seller and netting in
     P2.2b, after their own mirror scan.
292. **The new readouts are appended**: nine `summary.tsv` columns with the same names and order
     for every instance, and CSV and `stats.tsv` additions; P2.2a's columns keep their place and
     meaning. Alternative: columns for loop instances only.
293. **E4's engine g is P2.2a's reading** (HORSES-RULES §6.6), the mirror's reported beside it.
     Alternative: score the mirror's reading (to 1e-4 of the envelope's peak).
294. **E1's fixed plant is a plant at δ_p 0**, started from the design's plants
     (`--fixed-plants`). Alternative: an order rule `Fixed`.
295. **A planted desk's margin reads the bundle cost c, not c_full**, and so do its tilt and the
     maker's reservation, as in the mirror. Alternative: c_full, at which the maker's markup rests
     at θ (0.8) and ψ 0.25 acts where the markup on c is below 0.3125.
296. **Planted desks take the cash rule only, and no own input**: the step rule, a TypeDesk's own
     input and a maker's own horse-days are refused with a plant. Alternative: allow them,
     untested.

297–301 are P2.2b.1's (`docs/probe/LOOPS-RULES.md` §14.2), taken by Claude on your standing word
and open to veto.

297. **The engine's `RunErrorKind::ForeignWrite` boxes its delta.** The capacity desk's planted
     state holds 11 numbers, so every `StateDelta` grew and clippy's `result_large_err` refused
     every step's `Result`. No run, hash or message moves. Alternative: allow the lint on the
     engine, or keep the plant's record outside `ActorState`.
298. **The flow control's horse-day desk reads its own input from `inst.traction.own`**, a new
     param at 0. Alternative: reuse `inst.fodder.own`, which names another desk's recipe.
299. **No float constant on the engine path for the plant**: `p_inverse` on no plant is "none"
     (the mirror's +∞), so the capacity desk's z is outlay/O, or 0 where O is 0; a capacity desk
     whose running recipe has no coefficient above 0 bounds its bundles by κ·H. The first arises
     only where a capacity desk at θ < 1 holds no plant (its output is then 0 whatever it runs),
     the second only after a dated `SetParam` zeroes every running coefficient. Alternative: +∞,
     which needs an exception in the engine's literal scan.
300. **The loop instances are a module of their own, `probe::horses::loops`**, with their own
     instance, point, setup, genesis and tape; `horses-tape` knows the loop ids, and the harness
     step decides how `horses` reads them. Alternative (decision 290's letter): fold rule B into
     P2.2a's `Instance` now, which moves every harness readout in this step.
301. **LN6's rest-point bar is 1e-11**, not §10's 1e-12: its order takes the whole gap each tick
     and passes the plant's rounding to its order at 1/u times the order's size (measured
     3.0e-12). Every other instance is held to 1e-12. Alternative: 1e-12 everywhere, which LN6
     fails by rounding alone.

302–304 are P2.2b.2's (`docs/probe/LOOPS-RULES.md` §15.2), taken by Claude on your standing word
and open to veto.

302. **The rule-B harness sits beside the loop instances**, as `probe::horses::loops::{perturb,
     harness, probes, cmd}`, on P2.2a's shared row, statistics, classifier, kick set and reports
     (`probe::horses::report`). `horses` sends a loop id there. P2.2a's `Instance` and outputs
     are unchanged, and its running cost is read from the tape. Alternative (decision 290's
     letter): fold rule B into P2.2a's `Instance` and harness.
303. **The fodder desk's horse-days by buyer are its one buy line**, running and plant bundles
     together (2·q_f at rest), reported, not scored. The mirror's `minHf` reads the running line
     alone (θ at rest). Alternative: split the line by the plan's shares z/(z + s·I), to
     reproduce `minHf`.
304. **E0's allowed cancellation is the horse market's cleared volume on either side**: the
     capacity desk's order, or the maker's offer (its finished stock less its cover band),
     whichever is short. A parting is accepted where it is its inputs' gaps times the
     cancellation, within a factor of 10, and the mirror with an ulp each tick parts as far.
     HORSES-RULES §6.4's text names the maker's finished stock among the rounding sources.
     Alternative: the demand side only. Then LB1 b × 2 (3.9e-12) and the two fixed-plant runs
     (1.7e-12, 2.4e-12) would be partings to write up, though they are the same rounding.
     *Disclosed at P2.2b.4 (decision 310):* taken after E0's first output had been read.

305–306 are P2.2b.3's (`docs/probe/LOOPS-RULES.md` §16.2), taken by Claude on your standing word
and open to veto.

305. **A1's tolerances are read run by run.** "Every class exactly" compares each run with the
     mirror's run of the same name. "Three runs per instance within 25%" charges a run once when
     any of its tick or year readouts is beyond 10%; the tiers' medians and largest are scored at
     10% and charge no run. The registered values are A1's tables and, run by run, the mirror
     outputs they were made from. Alternative: score the tables' aggregates alone. The verdict is
     the same (no run was charged, every aggregate holds).
306. **LB1 at 12 ticks a year runs 49,000 ticks**, LOOPS-RULES §8.6's L·tpy/52 (the mirror's),
     not the 38,000 that `horses elasticity --tpy 12` prints; and E9's "0 of 40" is read on the
     mirror's 40 runs, the engine's four Tier-2 cost targets reported beside them. Alternative:
     38,000 and 0 of 44. Every one of the 45 runs orbits or diverges either way.

307–311 are P2.2b.4's, the fix round after the two reviews
([docs/probe/LOOPS.md](docs/probe/LOOPS.md) §5; `docs/probe/LOOPS-RULES.md` §17), taken by Claude
on your standing word and open to veto.

307. **The loop's GO is read as the plants and the maker's reservation together** (the fidelity
     review's first major). The reservation acts in 117 of LB1–LB3's 375 runs at L, and without
     it (LF3) LB1 loses five runs, r × 2, heads × 2, heads × 10 and the workers' coin × 0.02 and
     × 0.1. Decision 264's reason is reworded to match, O52 and O54 are amended, and the E6 and
     E7 tables carry the horse market's idle ticks, the maker's lowest coin, output and plant, and
     the unmoved horse price. Alternative: read the GO as the plants' alone, as registered.
308. **Decision 284 is narrowed: a loop instance enters Phase 2 through its own registration**
     (the fidelity review's third major). A loop of produced inputs may enter Phase 2 when every
     loop desk holds a plant, the reservation is on, and the instance has its own mirror
     registration, at the dial set it will run (C2m or C2g) and in its own funded county, which
     its engine battery then passes. P2.2b's GO covers rule B's horse at chain8: C2g, 52 ticks a
     year, ρ 0, J_b 1, fodder that cannot be stored, a loop on under 1% of the land (O79).
     Alternative: 284 as taken, with "P2.2b's registered battery" read as a template.
309. **Capital's time is read per stock in the loop step's report, and registered per stock
     from now on** (the fidelity review's second major; D-G14). LOOPS.md §4 gives the heads', each
     plant's and the paper's 5% times, the quasi-rents, and the last observable into tolerance:
     the maker's plant, which A1 §7.2 did not register, is last in most runs. It stays unscored
     (decision 285). The next goods-chain registration names each stock's 5% time, the maker's
     plant included. Alternative: leave D-G14 to Phase 3's path windows.
310. **Decision 304 stands as a disclosed reading, not an amendment** (the measurement review's
     first minor). It was taken after E0's first output. Under the registered reading, E0's LB1
     b × 2 (3.9e-12) and E1's two fixed-plant runs (1.7e-12, 2.4e-12) are partings, written up
     as the offer's cancellation; none is at tick 1, so E0's amendment clause does not apply.
     From now on a reading of a registered tolerance taken after output is seen is a dated
     amendment with its own sha256, before any scored run (decision 282). Alternative: a
     LOOP-SPEC-A3 now, dated after the scored runs.
311. **A wave's scorer and job list are committed before it starts, and the archive rescores**
     (the measurement review's second and third minors). P2.2b.3's stamp was a scratch file's
     time, and four patches edited the scorer during the wave (LOOPS-RULES §17.2). From now on
     the scorer, the gather script and the job list, or their sha256s, are committed before the
     first job; a later fix is dated and listed. The archive (gzipped CSVs) regenerates
     `runs.tsv` byte for byte with `gather_gz.py`, and `score_fc.py` fails E11's aggregate closed.
     Alternative: the scratch stamp, as at P2.2b.3.

Decisions 360–393 are P2.3.0's (2026-09-30, branch `phase2-proper` from `reboot` at `f7d1eae`):
the rulings next step 7 named before Phase 2 proper, taken by Claude on your word of 2026-09-26
("I leave all those calls up to you") and 2026-09-27 ("You can keep going beyond the gates"). They
are 61, 67 and 70 (1b, 1c); 118–123 (the markets probe); 135, 137, 139, 140, 147, 149, 151,
153–155, 158, 160, 161, 162, 164, 165, 167–169 and 173 (1d–1f); 179 (which is D-G1) and 186 (1g);
and GOODS-CHAIN's D-G13 to D-G15. The probe reports' recommendations weigh: MARKETS §6 keeps 67 and
70. Each line says whether the ruling is taken as proposed or amended, why, and the alternative.
All are open to veto; a veto of one reopens the instance or the oracle unit it names, not a
committed run. This line numbers 360–399 and O95–O109; other lines number below them.

360. **61 taken: cells stay out of the equilibrium, and their addendum will be 1b's.** Phase 2
     proper's instances keep 1a's continuous line, on which the markets probe's desks converge and
     every stability number rests (MARKETS §6); human-required work enters as 1d's hours, not as
     cells. When an instance first needs the agents' own cells, a 1b addendum solves the
     marginal cell's split (1b's cell block already prices cells, and machine types enter only
     through π), nests through 1c–1g (decision 63) and gets a probe of its own (O95).
     Alternative: a 1c addendum, before any instance needs it.
361. **67 taken, as 179 rewords it.** Machine recipes use machine services, labour and land only,
     so the wall and commons instances carry no loop, as I1–I3 carried none; the markets probe's
     GO depends on it (MARKETS §6). Alternative: machines built from categories (G5), which gives
     every multi-category instance the goods-and-machines loop P2.1 found NO-GO.
362. **70 taken: multiple equilibria are refused.** It held on every markets instance, and the
     agents' zero is a dead state, not an equilibrium. Phase 2 proper uses only economies the
     oracle solves with one equilibrium, so each run has one point to reach (378 for the priced
     form). Alternative: report every equilibrium and let Phase 2 say which the agents reach,
     which needs a basin map per instance first.
363. **118 taken: P2.1 stays closed, many markets GO for loop-free economies, a loop NO-GO, no
     fallback.** A11's kill condition is not met, and P2.2a and P2.2b since passed on stocks and
     plants. Alternative: read the loop's NO-GO as A11's failing mode B.
364. **119 taken: C2m is the default for flow instances, and C2L is not adopted.** The wall and
     commons instances run at C2m, C2 per role, so each labour market takes labour's 5.2 a year;
     instances with machine stocks run at C2g (391). Alternative: C2L's type rates and tilt, never
     run on I1–I3.
365. **120 taken as 284 amends it and 308 narrows it.** No instance of this run carries a loop of
     produced inputs; a loop enters Phase 2 only on plants, with the reservation on, through its
     own registration. Alternative: 120 as first written, every loop in Phase 3.
366. **121 taken: the weekly tick is Phase 2 proper's default** (and, by 237, the floor with
     machine stocks). At 12 a year I2 is unstable and I1 needs 13 times the years. Alternative:
     monthly ticks with the dials restated per tick length, which needs a probe of its own.
367. **122 taken: the markets report keeps its task's names**, MARKETS-RULES.md as registered.
     Alternative: amend MARKETS-RULES §7 as well.
368. **123 taken, amended: O22's families run first, and a stocks tier joins the new verdicts.**
     The families run on I1–I3 as registered, stocks first, before the wall and commons
     batteries. The two new instances carry a stocks tier in the verdict (every desk's coin and
     stock × 0.5 and × 2, P2.2a's Tier 3S, decision 229), since O22 finds stocks matter most for
     the chain and the zero. Alternative: 123 as written, with stocks a family.
369. **135 taken: worker types share one capability shape, with an efficiency each and reserved
     tasks.** The wall instance's desks carry productivity by type in this form: ε_i times the
     common line, and cells reserved to one type. Alternative: a schedule per type, whose
     I-dimensional solve has no uniqueness proof.
370. **137 taken, amended: the wall instance is a solved wall, with its distance from each edge
     registered.** It sits strictly inside its stretch: the pool's wage above v(1) and below the
     ceiling where 1d turns `LaborShort`, or a type at its wall with its wage above ε_i·v and
     short of a reserved shortage's edge. The frame registers both gaps at genesis and at every
     cost target, since a shocked point at either edge has a vertical supply the agents may not
     reach, and near the ceiling a wage ill-conditioned in the data (O30). Alternative: 137 as
     written, the gaps unregistered.
371. **139 taken: per-type hours are compared through each type's supply at the oracle's wages,
     or in total.** Pooled types are perfect substitutes, so who works which pool task is no
     oracle output. Whether the pool trades as one market in efficiency hours or as one market
     per type is the frame's call; neither split is scored. Alternative: per-type hours by task,
     which the model does not determine.
372. **140 taken: machine recipes buy pool labour only.** The wall instance's type desks buy pool
     hours. Trained machine builders couple the walk and the (O, V) system and wait for a 1d
     addendum (O95). Alternative: that addendum before the wall instance.
373. **147 taken: parcels are efficiency units.** A tape's parcel quality scales its service, and
     the commons instance compares land in totals, not by which parcel idles, which the oracle
     sets by convention. Alternative: a Ricardian working cost per acre, which no source gives.
374. **149 taken: one participation rule for both exit forms.** Under s(q) a pop's share is
     F(ln((ν·P_s + w)/(ν·P_s + p_g·s(q)))), at the rent its plot pays: 0 on a commons with room,
     the shadow rent when it is crowded, the market's rent on enclosed land. Agents read only
     posted prices (R13), so a crowded commons' shadow rent must be posted, a price that moves
     with plot demand and pays no one (156); how is the frame's. Alternative: a rule per form.
375. **151 taken: the exit good is one category, named on the tape.** The commons instance names
     food, a category with tasks and direct land, and q = r/p_g in its units. Alternative: a
     bundle of goods.
376. **153 taken: idle land at zero rent is compared in wage units.** The commons instance rests
     neither on a free good's absent outputs nor on the walk's ceiling, which is refused. The
     agents' land market at zero rent needs an answer first: under `Saturate` its unsold price
     falls to the runaway bound. It is chosen without a clamp by a mirror scan and registered
     before its build, as L0 chose the maker's reservation (O96). Alternative: 1d's
     `LaborShort` refusal kept, with no idle-land instance.
377. **154 taken: no reserved tasks under the priced form, for now.** The wall instance needs a
     walled type, since only a walled type's labour market has a price of its own (371), so it
     takes 1d's dependence form, the named alternative with its own gate (ADDENDUM A8); the
     commons instance under s(q) pools every type. A walled type under s(q) needs 1e's addendum
     before the 1750-like instance carries one (O95). Alternative: that addendum first, and the
     wall instance under s(q).
378. **155 taken, amended: the commons instance is chosen certified.** Where the priced form lets
     supply fall, the count is exact only under 1e's Proposition 5 (ρ 0, an exit good with
     direct land, h_i ≤ s₀,i·b̄_g and h_i·ℓ₀ ≤ ε_i, one plot-taking type on a crowded commons);
     elsewhere two equilibria inside one scan cell go unseen (O30). The verdict instance meets
     Proposition 5; an uncertified one is a family, checked first on a scan 16 times finer.
     Alternative: 155 as written, any instance the scan finds unique.
379. **158 taken: one land service.** Both instances have one scarce land class; a region with
     two needs 1e's addendum or 1f's substitution first (O95). Alternative: SSRN A.1's vector of
     non-produced services now.
380. **160 taken: a free exit good at r = 0 is decided at the wall's end.** It binds only an exit
     good of land alone; the commons instance's food embodies labour, so its plots are decided as
     before. Alternative: §4.6 read literally, which breaks the junction.
381. **161 taken: free plots on idle land are `ExitLand::Idle`.** The harness reads `Idle` as
     plots free on idle enclosed land at r = 0, and `Enclosed` as closed by price with no
     suitable land idle. Alternative: `Commons`, or `Enclosed` by convention.
382. **162 taken: the tax bases** (payroll on gross wages, consumption on final purchases at
     producer value, rent on market rent in money). No instance of this run has a government;
     the 1750-like instance's tapes use these bases. Alternative: a tax on every purchase, which
     breaks the ledger.
383. **164 taken: the budget closes by the owners' levy (RentRate).** Relief in baskets, the poor
     law, is RentRate's; a tape that gives every rate needs Dividend or a deficit. Alternative:
     Dividend as the default closure.
384. **165 taken: a transfer supplements the support unless the tape says it replaces it,** and
     the agents' participation rule reads it so. Alternative: replacement by default.
385. **167 taken: walled types only under RentRate and without in-work benefits.** A trained type
     at its wall under Speenhamland needs an addendum (O95). Alternative: that addendum before
     the 1750-like instance.
386. **168 taken: the path's start is evaluated, and `SurplusLabour` refused.** An agent economy
     whose in-work benefit overfills it has no point to reach and is not an instance.
     Alternative: 1a–1e's positive start assumed, which miscounts a benefit-driven start.
387. **169 taken: the price-responsive basket is a CES with the basket's weights, the fixed basket
     the default.** Every instance of this run uses the fixed basket in the oracle and in the
     agents' basket provider, as I0–I3 did; a CES instance needs the agents' households on the
     same rule, and the dependence form (170). Alternative: Stone-Geary around a subsistence
     basket (PLAN §3.2), which needs an oracle addendum (O95).
388. **173 taken: a basket per worker type is deferred.** The 1750-like instance takes the common
     basket in the oracle and the agents. Owners buying domestic service need 1f's addendum,
     which breaks eq 11 and needs its own uniqueness (O95). Alternative: that addendum before the
     1750-like instance.
389. **179 (D-G1) taken: decision 67 reworded.** Machines are durable goods built from and run on
     goods, and their recipes never use a category (E1); 67's mathematics is kept. The loop-free
     instances keep 67's letter, a special case; the goods chain's keep E1. Alternative: G5,
     machines from categories.
390. **186 taken: E2 enforced, G1 not approximated.** Hearth coal and carters' fodder wait for G1,
     which D-G9 builds with the goods chain's 1750-like instance. Alternative: G1 now.
391. **D-G13 taken, reconciled with 119: C2g for instances with machine stocks.** C2g puts the
     machine and hour markets at C2L's type rate, 5.2 a year, land at 0.1625 and every tilt at
     0, not C2L's 1; P2.2a and P2.2b are GO on it. Flow instances stay at C2m (364). So the goods
     chain's 1750-like instance registers at C2g (step 6's item 3; decision 308) unless its own
     scan finds otherwise. Alternative: C2 with M1 at s_K 0, for 1a's maker only.
392. **D-G14 taken: capital inertia under the cash rule is a named departure (R6).** Engels'
     pause, the gap's closing time, the windfall and the waterfall stay unscored until a rule
     meets GOODS-CHAIN §3.4's lifting criterion; each stock's 5% time is registered (309). No
     instance of this run holds a stock. Alternative: score them now against the agents' own
     timing.
393. **D-G15 taken, extended: ρ 0 for v2a and the stocks probes, with factor shares unscored at
     ρ 0, and for Phase 2 proper's first instances too.** They are the flow case (M9), where 70's
     uniqueness and Proposition 5 are proved. Interest waits for the mirror's runs with fuel, wet
     M3 and J_b > 1. Alternative: a registered ρ now, uniqueness measured, not proved.

Decisions 394–397 are P2.3.1's (2026-09-30): the wall frame's proposals FW1–FW9
(`docs/probe/wall/SPEC.md` §8), numbered at its registration and grouped, since this line's range
has six numbers left and the commons instance needs some. Claude's, on your delegation, each open
to veto; the alternative named is the registered one (R6).

394. **IW1 is Phase 2 proper's wall instance (FW1, FW2).** Unit 1d's B economy with E7's three
     worker types: η 0.5, L^H 0.1, R 0.04 (trained, services) and 0.03 (master, goods), N 130, 52
     and 26 a year, χ_max 1, 2 and 2, T 520, h 1 and Appendix B's machine. The trained and the
     master sell only their reserved hours (ε 0), and every type's support is one basket (ν 1),
     so each type trades on one market and no rule chooses between markets (371, 377). With E's
     efficiencies the base point is the same double for double, but at four targets the trained
     would join the pool. Alternatives: 1d's E3 as written (unfunded under the probe's transfer,
     10.4 baskets of support against T 10); a one-type wall (W1), which has one labour market;
     E's ε and ν with a market-choice rule, its nesting and its mirror first (O97).
395. **The roles' additions are three optional fields (FW3):** the category desk's `tail` (the
     pool's hours at tasks closed to machines, added to the line's hours) and `reserved` (each
     reserved type's hours, bought on its own labour market, in the cost, the orders and the
     Leontief), and the provider's `more` (further transfers of N_i·P_s, paid in list order, the
     state holding the sums). No new kind, state or market rule; every committed tape keeps its
     hashes. Alternative: a new desk kind for categories with human-required or reserved tasks.
396. **The wall's targets, observables, grammar and readouts (FW4–FW7).** The cost targets are
     land.mach, tail.services and res.services.trained at ×1.1, ×0.9, ×2 and ×0.5, dated and at
     genesis, each a funded wall at least 0.216 in log from every edge. At the wall the harness
     reads each desk's threshold x_j = 1 − s_j in log, not s_j (whose target 0 has no log); P2.1's
     instances keep s_j. The grammar replaces s[D]\*F (a no-op at s\* = 0) by s[D]=V at 0.05, 0.2
     and 0.5, scales every labour market in JA and JB, adds RW(F) where there are several labour
     markets, and puts the reserved wages in the basin family in place of the technique: 103
     battery runs. The wall's own readouts (depth, breach ticks, each desk's largest share, each
     pop's participation range and saturated ticks) are reported, not scored. Alternatives: land
     alone as I0; an absolute tolerance on s_j; s[D]\*F kept and marked slack (VACUOUS); breach
     ticks scored.
397. **IC1 is a reported control, and the verdict rule is P2.1's with Tier 3S (FW8, FW9).** IC1 is
     IW1 with the entrant's χ_max 0.25, the task margin active (x\* 0.947), like for like and not a
     verdict instance. IW1's verdict: mode A passes; every run of Tiers 1–3 and of Tier 3S (each
     stock and coin × 0.5 and × 2, the start distance the first year's largest D̂, decision 229) is
     CONVERGED, Tiers 3 and 3S again at 10·L; every target's kick set passes; O22's families are
     reported, stocks first; L comes from the engine's elasticity probe; the scorer is committed
     before the wave (311). Alternatives: no control; score the families as well.

Decisions 398–399 are P2.3.5's (2026-09-30): the commons frame's proposals 360–371
(`docs/probe/commons/SPEC.md` §7), numbered at its registration and grouped, since two numbers of
this line's range were left. Claude's, on your delegation, each open to veto; the alternative
named is the registered one (R6).

398. **C1 and C2 are Phase 2 proper's open-commons instances, and the commons is no market (the
     frame's 360–366 and 368).** I1's economy with one priced worker type in food, χ_max 1 (I1's),
     support one basket (ν 1), and a commons the workers hold: C1 full (Crowded), C2 with room
     (Commons). Phase 2's idle land at zero rent is the commons' idle part; idle enclosed land at
     r = 0 enters no instance under the probe's transfer, which it leaves unfunded (O96). The
     commons is no market: the commoners' participation rule gives out its plots, hours =
     min(max(n(0), N − T_o/h), n(r̂)) in SPEC §3.1's evaluation order with fma, and the shadow rent
     is a readout nobody receives. This amends 374's reading that a crowded commons' shadow rent
     must be posted: the rule rations the commons from the pop's own params and posted prices, so
     nothing is posted (R13). Plots that spill onto enclosed land are rented in money on the land
     market, in one budget chain with the baskets. The exit good must be in the workers' basket
     and land must not be. No state is added. Alternatives: 1e's K1 and K2 on Appendix B's economy
     (uncertified); χ_max 0.25; a funded support and a free-good rule for idle enclosed land; the
     commons as a market under `Saturate`, exact while crowded or spilling and running away where
     it has room; the shadow rent by bisection; plots rented in kind through an untraded home
     good; a joint land order for space and plots; ν a param and the regime in state.
399. **The commons' targets, harness and verdict (the frame's 367 and 369–371, with 368's Tier
     3S).** The cost coefficients are land.mach, b.food and the commons at ×1.1, ×0.9, ×2, ×0.5,
     at genesis and dated; enclosure by law is a family. The harness's targets come from unit 1e's
     `ParcelEconomy`, land's volume T (plots included) and labour's the supply S. The subsistence
     trap is the families' prediction, not the verdict's. The engine run is SPEC §5.5's E0–E9 with
     §5.6's tolerances, the scorer committed before its wave. The verdict is §5.5's with Tier 3S
     (decision 368, which the frame left out; its predictions run on the frame's mirror before any
     code, registration §3). Alternatives: the commons shocked only by enclosure; the dependence
     form's harness, scored on prices only; the families in the verdict, which would make C1 and
     C2 LOCAL at joint4; the verdict alone (E0–E3).

## Open — your calls

- **The GUI's decisions**, 22–34 (D1–D13): G0 carried them out, none vetoed; a veto now reopens
  what G0 built on it. D10's items are built.
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
- **Decisions 360–393** (P2.3.0, branch `phase2-proper`, 2026-09-30): the rulings Phase 2
  proper waited on, taken by Claude on your word, each open to veto, one line each with its
  alternative. 30 are taken as proposed; 123, 137, 155 and D-G15 are amended (368, 370, 378,
  393). The ones that shape this run: O22's families first and a stocks tier in the new
  verdicts (368); the wall instance under 1d's dependence form, its distance from each edge
  registered (377, 370); the commons instance certified by 1e's Proposition 5, its shadow rent
  posted, its zero-rent land market's remedy chosen by a mirror scan first (378, 374, 376, O96);
  C2m, 52 ticks a year, ρ 0 and the fixed basket (364, 366, 393, 387). For later: C2g for the
  goods chain's stock instances (391), the common basket for the 1750-like instance (388), and
  the oracle addenda of O95.
- **Decisions 394–397** (P2.3.1, 2026-09-30): the wall frame's FW1–FW9, grouped: IW1 with
  reserved-only types (394), the roles' three optional fields (395), the wall's targets,
  thresholds, grammar and readouts (396), IC1 a control and the verdict rule with Tier 3S (397).
  Each open to veto before the wall's scored runs.
- **Decisions 398–399** (P2.3.5, 2026-09-30): the commons frame's 360–371, grouped: C1 and C2 and
  the commons as no market, its plots given out by the participation rule, which amends 374's
  posted shadow rent (398); the commons' targets from unit 1e, the trap in the families, and the
  verdict with 368's Tier 3S, which the frame had left out (399). Each open to veto before the
  commons' scored runs.
- **Decisions 59–75 and 135–178** (Phase 1), open to veto. Those that bound Phase 2 proper (61,
  67, 70; 135, 137, 139, 140; 147, 149, 151, 153–155, 158, 160, 161; 162, 164, 165, 167–169,
  173) are ruled by Claude at P2.3.0 as 360–393, with the markets probe's advice: keep 67 (its
  GO depends on it) and 70 (it held on every instance), and leave 61 to a probe of its own
  (MARKETS §6). 172 binds Phase 3, and 177 Phases 6–8. GOODS-CHAIN, outside the repository
  ("Where things stand"), proposes D-G1 to D-G15: D-G1 is 179 (ruled as 389), D-G10 is 180,
  D-G11 is 285, and D-G13 to D-G15 are ruled as 391–393.
- **Decisions 118–123** (the many-markets probe's), with MARKETS-SPEC §9's frame decisions
  M1–M9: 118–123 are ruled by Claude at P2.3.0 as 363–368, open to veto; M1–M9 stay open to
  veto as the frame states them. Of its questions, Q1 is answered by 120 and Q2 by the GO at 52
  a year. Q3, a switch between machine types, waits for durable machines; Q4, category inputs,
  is untested.
- **Decisions 76–81** (G0.1's first part), **82–89** (its second part), **90–96** (its
  verification fixes), **97–108** (G0.2), **109–113** (its verification fixes) and **114–117**
  (G0's close), open to veto.
- **Decisions 124–134** (the demo world and its map, branch `demo-world`), open to veto. The
  ones that shape later work: Northern Ireland in the atlas (125), the base county's departure
  from Appendix B (126), and the lens measures in worldgen until observe (128).
- **Three windows to check by hand, all PENDING, yours.** Each ran headless or in the smoke
  mode on Windows; nobody has looked at any of them. On Windows, from the repository once this
  merge is in `reboot` (or from `merge-og-g1`), in PowerShell:
  - *The G0 gate's window* (G0's gate; how to try the editor is under "G0 is closed" in "Where
    things stand"):

    ```powershell
    $env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/g0-hand'
    cargo run --release -p rustyecon-gui -- tapes/gate.ron
    ```

    Type `2080` in "until" and press "Run until": every price is plotted, the run reaches tick
    2,080, and nothing panics. `g0` proposed merging after your look; the merge came first, so
    if the window fails your look, the G0.4 that fixes it (decision 114) lands on top of
    `reboot`'s line.
  - *The demo's map* (under "The demo world and its map are closed" in "Where things stand"):

    ```powershell
    $env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/demo-hand'
    cargo run --release -p rustyecon-gui -- tapes/demo-gb.ron
    ```

    The map opens paused at 1750, runs to 1901 and recolours on Space, the lenses switch on `[`
    and `]`, and hover and click work; two screenshots are in `docs/demo/`. If it fails your
    look, a D.6 fixes it on top of `reboot`'s line.
  - *G1's window frame time* (G1's gate: the window's p90 frame under 16 ms; the four steps to
    take are under "G1, the oracle lab, is built" in "Where things stand"):

    ```powershell
    $env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/g1-hand'
    cargo run --release -p rustyecon-gui -- tapes/gate.ron
    ```

    The frame stays smooth with the lab solving and sweeping, the explainer and its waterfall,
    a log panel and a breakpoint in use, and a snapshot's PNG carries its banner. The smoke
    mode's p90 was 1.22 and 1.78 ms running. If it fails your look, a G1.12 fixes it on top of
    `reboot`'s line.
- **Decisions 179–190** (unit 1g's, track 1g), open to veto. The two that bind Phase 2 are
  ruled by Claude at P2.3.0: 179, decision 67 reworded (D-G1), as 389, and 186, E2 enforced and
  G1 not approximated, as 390. 180 (D-G10) amends 71 and keeps every result of 1c bit for bit;
  181, the mapping per period in the oracle, leaves where tapes are built to O32.
- **Decisions 200–219** (G1's, branch `g1`), open to veto. The ones that shape later work: the
  lab's form outside the model and the session (201), the fields read from the oracle's `Debug`
  rather than a `Point::outputs()` in the oracle (202), goldens paired by name (204), the
  breakpoints' semantics (209), and G0's leftovers deferred to a G1 second part (217).
- **Decisions 220–239** (the stocks probe's, branch `phase2-goods`), open to veto, with
  GOODS-CHAIN's D-G2 to D-G8, which the probe took as proposed, and D-G9 and D-G12, all still
  yours (D-G1, D-G11 and D-G13 to D-G15 are ruled by Claude as 389, 285 and 391–393). The
  ones that shape later work: C2g and its fodder rate (226), windows from the slowest mode (228),
  the probe's closing reading (234), O14 compared like for like (235), the idle market's remedy
  before the loops and the goods-chain 1750-like instance (236), and weekly ticks as the floor
  for machine stocks (237). D-G12, the demo's second pass starting at stage v2a.1, is the ruling
  the demo waits on.
- **Decisions 240–283** (the loop stage's groundwork, branch `phase2-loops`), open to veto. The
  ones that shape later work: M6 and M5 (240, 241); the maker's reservation at ψ 0.25 and on by
  default (245, 246, 253) with its structural off (278); the funded county chain8 (254); the
  plant on every loop desk, beside the horses on the capacity desk (260–263) with no further
  damper for the pass-through (264); the genesis carry in the loop mirror and its convention
  for planted desks (272, 273); per-market dead ticks (274); and S1 out of P2.2b (276).
  Decisions 284 and 285, taken by Claude on 2026-09-29 so that P2.2b can run: loops enter
  Phase 2 on plants (284, amending 120), and machine stocks come forward for the goods chain
  (285, D-G11).
  `phase2-loops` (L0.1–L0.9) starts at `401f7b1`, where `reboot` and `phase2-goods` both stand,
  and touches `crates/agents`, `crates/probe`, docs/ENGINE.md, docs/TAPE.md, docs/probe/ and
  this file; while `reboot` stays at `401f7b1`, it lands by a fast-forward, on your word.
- **Decisions 286–311** (P2.2b's, branch `phase2-plants`), open to veto. The ones that shape
  later work: core's untraded good (286); the loop's GO read as the plants and the reservation
  together (307); decision 284 narrowed, so a loop enters Phase 2 only through its own
  registration at its dials and county (308); capital's time registered per stock from now on
  (309); a reading of a registered tolerance after output is seen goes by dated amendment, and a
  wave's scorer is committed before it starts (310, 311). `phase2-plants` (P2.2b.0–P2.2b.4)
  starts at `8b07c8a`, where the local `reboot` and `phase2-loops` stand, and lands by a
  fast-forward on your word.
- **Landing the branches.** `phase0-s2`, `phase1`'s P1.2–P1.7, the `g0` merge (`708167f`),
  `phase2-markets` (P2.1.1–P2.1.4, `b2a55e3`), the `demo-world` merge (`2398b6a`) and the
  `phase1` merge (`16eb728`) are in the local `reboot` by fast-forwards. `oracle-goods`
  (P1g.1–P1g.7) and `g1` (G1.1–G1.11) are merged at `43ad8c5`, on `merge-og-g1`, and `reboot`
  took the merge by a fast-forward. Nothing is renumbered: the two tracks numbered apart
  (179–199 and O31–O35; 200–219 and O36–O40). `phase2-goods` (P2.2.1–P2.2.4, `d636b76` to
  `401f7b1`) started at `92ba68e` and is in the local `reboot` by a fast-forward too: `reboot`
  is at `401f7b1` (amended at L0.9; this line said it would land on your word).
- **Pushing `reboot`** after `phase2-loops` has landed. `reboot` is at `401f7b1` locally and on
  `origin` (with session 2, Phase 1, G0, the many-markets probe, the demo world, unit 1g, G1 and
  the stocks probe; amended at L0.9, since this line said `origin/reboot` was at `cf3c0ff`).
- **The Phase 2 session budget** that A11's kill condition needs (PLAN Phase 2), now for Phase 2
  proper's other instances.
- **The decisions above**, especially 10 (the engine crate, not in PLAN's crate list), 11, 15
  and 17, and the GUI's 22–34.

## Open — work

- **O1. The GUI.** Designed ([docs/GUI.md](docs/GUI.md); A14), with egui in `crates/gui`. G0,
  the shell, is closed at G0.3 (2026-09-27; GUI.md §9): G0.1 the viewer (the crate's seams,
  the Runner and `ThreadDriver`, the Extractor, the store and the ring; the toolbar, timeline,
  outliner, plots, inspector, registry and log, the goldens, the kittest scripts and the smoke
  mode) and G0.2 the editor (`materialise`, the lineage, branches, compare and export), each
  verified and fixed. Its window check by hand is yours. G1, the oracle lab, is built on branch
  `g1` (G1.1–G1.10, 2026-09-27; "Where things stand"): the lab, a field over x, sweeps, the
  price-step explainer and the log waterfall, log axes, the watchlist, event and date
  breakpoints and PNG snapshots, with the engine's re-exports (the GUI's edge to core is gone),
  O26's map items and O20's scan escapes; verified once and its findings fixed at G1.11. Its
  window's p90 by hand is yours; what G0 moved to it and it did not build is O36, and a
  re-check of its fixes O37. `crates/engine` was built for it: a frontend
  depends on the engine alone, steps a `Sim` on a worker thread and reads each `TickReport` over
  a channel. Session 2 gave it what §7.2 asked: `FiredEvent.source`, the registry's sites with
  their methods, a `world_id` that a new source event keeps, and from certify `RunKey`,
  `tape_hash` and a manifest that needs no Parquet or I/O (GUI.md, updated at S2.6). On branch
  `demo-world` (D.1–D.5), part of G4 and G2 came forward over the illustrative demo tape: the
  atlas, the map and 25 lenses with a ranked table (GUI.md, "The map and lenses, brought
  forward"; decision 124). G4 proper still waits for D8's trigger.
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
  O18, O19 and O28–O30, and the decisions that bind Phase 2.
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
  path needs the oracle's reference, so it waits for Phase 2 proper and `crates/observe`. The
  many-markets probe measured them on six economies, and with many markets retired as a risk
  for loop-free economies, paths are now Phase 2 proper's first (O24). *Amended at P2.2.4:* the
  stocks probe measured them with the horse held as a stock (HORSES §4). Against I0 alone its
  paths look far softer (a cost doubling bottoms at 55–57% of old output against 13.5%), but
  like for like, against the flow county at C2g with ex-post assignment (39%), stocks give about
  a quarter of that gain, cut the transfer shortfall tenfold, and cost 2.3–2.8 times the years;
  the worst peaks are worse (decision 235). *Amended at P2.2b.4:* in the loop (LOOPS.md §4) b × 2
  moves the good's equilibrium by −0.16% in log, while the good falls to 0.70 (stocks) and 0.68
  (flow) of it for about half a year, 217 and 239 times the move in log. Like for like, stocks
  lift that trough by 0.036 in log and take 1.36 times the years, 1.17 on the observables both
  economies share (1.94 at δ 4%).
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
- **O20. What G0's two re-checks left** (decision 116; GUI.md, closed at G0.3, item 5). Each
  re-check found the fixes in place; these are left, none of them a gate item. G1 takes them,
  each with a test that fails without its fix. G1.7 closed the first item's two scan escapes;
  the rest is O36's:
  - *G0.1's re-check, on `2cced21`* (`D:/rustyecon-g0/verify-G01-*-r2/`). Two ways past the
    no-egui scan survive: a reach into `ui` through a renamed crate root (`use crate as g;`
    then `g::ui::…`), and a `use` group whose first root is another crate (`use {std::fmt as
    _, crate::{ui as _}};`). Three mutants of the plots survive
    `every_drawn_vertex_is_recorded`: the village's lines' y scaled by 1.5 where they are lent,
    every line's y moved to 2y + 1 where it is lent, and each segment's last vertex dropped
    from both the record and the line. So the painted y, and the end of each segment, are not
    held to the store. G0.2 changed neither the scans' reading of roots nor the plots, and at
    the close all five, rerun on `22dff0f`'s code, still survive
    (`D:/rustyecon-g0/close/recheck.txt`).
  - *G0.2's re-check, on `22dff0f`* (`D:/rustyecon-g0/verify-G02-*-r2/`): 31 of 36 mutants
    killed. Five survive: compare's hedge without its check of the branch's own start, and
    the walk to earlier records taking any resumed run, both of which look equivalent (a run
    that resumed starts at its checkpoint's tick, where its state is its parent's, and a run
    that reran starts at 0, which the walk refuses); the compare pane handed no earlier
    records, and the pane never painting the hedge, which no kittest script reaches (none
    builds a branch of a resumed branch); and the ancestor never taken from an open run, which
    the host then reads from disk, where every test leaves it.
  - *Two edge cases of provenance,* from G0.2's re-check's probes. A branch of a hand-edited
    tape compares its tolerances with that tape, its parent, and its export says `# ledger
    changed: no`, though its tolerance differs from the base's: "ledger changed" reads one
    generation. And a tape with no marker, beside a lineage file that does not read, opens as
    a run, with the log saying the lineage does not read; U3 read strictly makes it an
    experiment.
- **O21. The loop's absorbing zero** (decision 120; MARKETS §3, §6). In L2 and L3, two machine
  types buy each other's non-storable service. They diverge at C2m. At C2L, ±20% displacements
  kill them:
  - a type desk whose plan exceeds 1/a_kk of its stock keeps all of it;
  - its partner then makes nothing, and neither restarts, since services cannot be stored, the
    genesis lots are gone and no role enters;
  - the prices then run away under `Saturate`.

  A keep rule that shares the shortfall does not help. The fix is a stock (Phase 3's durable
  machines, or a storable service) or entry at zero coin. Two I1 runs show the one-type cousin, a
  tick with no machines traded and no baskets. Carried to Phase 3.
- **O22. The markets probe's unrun families** (decision 123). Each family is built into
  `markets family`, and the variants are flags (`--one-sided hold`, `--set tilt.*=1`):
  - stocks: every desk's coin and stock, and each type's stock ×0.01 and ×10, which matter most
    for the chain and the zero;
  - joint2 and joint4, basin, history (`cycle`), and the `Hold` and tilt-1 variants;
  - five map cells of I3.

  Phase 2 proper's battery runs them first, stocks first.
- **O23. Tick length and the kick's horizon in many markets** (decision 121).
  - At 12 a year I2's point is unstable, and I1 takes a median of 90 years to reach tolerance
    (7 at 52 a year).
  - At 12 a year I3's base kick misses the bar at H = L, though its slow mode is stable: it
    passes at 5L. Two of its cost targets are slowly unstable there. This is CERTIFY §15.1 (6) from the
    other side: H = L can fail a slow stable mode as well as pass a slow unstable one.
  - The mirror's PL from random directions misses such slow cones, so a registration that
    relies on it should search for them.
- **O24. Paths in many markets** (O14; MARKETS §4). The medians are as on Appendix B, but the
  tails are deeper:
  - Tier 2's worst consumption trough is 0.28–0.34 of Y\*, against 0.54, with up to 99 dead
    ticks against 12;
  - a machine-land shock cuts consumption to 4% of Y in I1, where Appendix B fell to 13.5%;
  - one small, fully automated category binds up to three quarters of short household ticks.

  Planned assignment turns a missing machine service into zero output while labour clears.
  Ex-post assignment, the registered alternative, is untested in many markets.
- **O25. The illustrative marker in scoring** (branch `demo-world`, D.4, 2026-09-27;
  docs/GUI.md U5 and its block "Amended at D.4", item 8; docs/demo/WORLD.md §8). The demo
  world's tape, `tapes/demo-gb.ron`, is named `demo-gb [illustrative]` and every basis in it
  begins "illustrative demo". `certify` now seals any run of a tape whose name carries
  `[illustrative]` UNSCORED, never PASS, and refuses a tape whose bases say illustrative once its
  name has lost the marker (`certify_refuses_illustrative_tape`). Left for Phases 6–7: the
  scorecard refuses it beside the GUI-experiment marker (`scorecard_refuses_gui_edited_tape`
  gains the demo tape), and the identity chip shows it. Until then keeping its figures out of
  citation is procedural. It was O21 on `demo-world`; the `demo-world` merge renumbered it
  after the many-markets probe's O21–O24.
- **O26. What D.4's re-check left** (decision 134; GUI.md, "The map and lenses, brought
  forward", items 8 and 9). The re-check (`D:/rustyecon-demo/verify-map-r2/`,
  `verify-world-r2/`) found D.4's fixes in place, and 17 of its 25 mutants of the map were
  killed. G1.6 (branch `g1`) took the first two items: the eight mutants are killed and the
  credit wraps, each by a test that fails without it (decision 215). The clock and the layout
  notes are O39's, for the demo's next pass:
  - *Eight mutants of the map survive* (`verify-map-r2/mutate.py`, `mutations.txt`):
    - V7, the county card's value read a tick early;
    - V8, the card's change lens read against the cursor instead of genesis;
    - V9, the app handing the map no cursor;
    - C6, the legend painted in a reversed scale while `MapFrame::legend` records the right one;
    - C7, a fresh mesh whose vertices after each region's first take a neighbour's colour;
    - C8, the in-place recolouring skipping each region's last vertex;
    - A2, the credit painted transparent;
    - A3, the credit placed off the canvas.

    The fixes: hold the card to the engine as `lens_values_equal_the_engine` holds the map.
    Record the legend's and the credit's colours and rectangles from the shapes painted, not
    from what the painter meant. Check every vertex of each region. Run a script with the
    cursor behind live.
  - *The credit clips on a narrow canvas* (`verify-map-r2/probe2.log`). A canvas narrower than
    its longer line, about 470 points, clips it. At a 1,280 × 800 window its second line loses
    19% of its width, and at 1,024 × 768 its lines lose 19% and 37%.
    `the_credit_is_painted_clear_of_the_legend` tries four widths, none that narrow. The fix is
    to wrap the credit to the canvas, and test it at 1,024 × 768.
  - *The clock is not held to 52 ticks a year* (`verify-world-r2/runs/tpy*.log`). The compiler
    accepts `ticks_per_year` 1, 2, 4 or 12, and holds the dials to C2, which was registered at
    52. At 4 a year the compiled tape runs to 46,548 dead county-ticks and 46,298 ticks with a
    transfer shortfall, and every county ends far from its oracle point. At 1 and 2 D̂ is
    infinite. At 12 no tick is dead. The fix is to hold the clock to 52 as the dials are held
    to C2. Decision 121, the many-markets probe's, makes the weekly tick Phase 2 proper's
    default too.
  - *Seen in the close's screenshots* (`docs/demo/`):
    - at the fitted view the legend and the credit cover Cornwall and part of Devon, so the
      fit should leave the legend's corner free;
    - with a long lens name, the ranked table's value column and the card's header are cut at
      the pane's edge;
    - on a narrow window the health chip wraps and the toolbar grows.

  The re-check also ran histories at the guard's edge (N and T ×4.42 over 20 years, η ×0.374
  over 10 years, a yearly square wave, a combination). Each compiled and ran with no dead tick
  and no shortfall, so the compiler's bounds hold what they were set for.
- **O27. The demo's second pass: goods, machine types and carriers** (WORLD.md §7; decision
  126). The tables reserve their columns (`categories`, `machine_types` and `carriers` in
  `regions.csv`), and the compiler refuses any other value in them until then. The
  many-markets probe built the many-market roles: basket providers and basket workers buying
  many items, category desks and type desks. They are GO for loop-free economies at 52 ticks a
  year under C2m (docs/probe/MARKETS.md; decisions 118–121). Since the `demo-world` merge the
  demo and those roles are on one line, and the pass goes:
  1. **Categories** (unit 1b): a `categories.csv` of food, textiles, metal goods, shelter
     (space as a category) and services, each with its basket weight, direct land and segment
     of the task line. Genesis comes from unit 1b at each county. A history row's param takes a
     qualifier (`eta@textiles`), so the textile ramps move only textiles, which is what they
     meant.
  2. **Machine types** (unit 1c): a `machine_types.csv` of horse and water power, the steam
     engine and the railway, with their recipes and θ. Genesis comes from unit 1c. No type buys
     another's service in a loop, which is decision 120. The services cannot be stored until
     Phase 3's durable machines (O21). The coal and steam ramps then move steam's b.
  3. **The dials and the clock:** C2m, and 52 ticks a year (decisions 119 and 121; O26's
     clock item).
  4. **The battery** again, county by county, as WORLD.md §3.3 ran it, and the long run's dead
     ticks and shortfalls checked again: O24 finds many markets' tails deeper than Appendix B's.
  5. **Lenses** for each category's price in rent and share of output, and for each type's
     share of machine tasks.
  6. **Carriers, later:** a `channels.csv` from the atlas's 201 land borders and 27 port
     sites, with road, canal, coast and rail. They need transport desks, home-node trading
     (Phases 4 and 9) and an oracle with trade. So they switch on gradually from zero
     capacity, and the railway ramps on b come out as they go in. The map then draws flows on
     channels (G4, G9).

- **O28. What the re-checks of 1d–1f left** (recorded, not fixed: the bounded verification ends
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
- **O29. A CES weight below the scale floor is refused** (the 1f re-check). A weight must be 0
  or within [1e-30, 1e30]; eq 26's weights α_j^σ fall below that near `SIGMA_CEIL` = 64
  (0.3^64 = 3.4e-34), and six such economies, each with an equilibrium, were refused as
  `Invalid`. Either lower the floor for weights, with the CES evaluation checked there, or state
  the σ each weight allows. No Phase 2 instance needs σ near 64.
- **O30. Precision and refusals of 1d–1f that Phase 2's bands must allow for** (recorded, as
  O18). A tie with a walled type, or at a wall switch with nearly parallel delivered costs,
  carries v's error into σ amplified (8.8e-12 measured, dlog σ/dlog v = −201; unit-1d.md §12
  item 18); a wall near its real-wage ceiling has a wage ill-conditioned in the data (§5.5);
  with walled types on idle land P_s is the walk's fixed point, and one ulp of T_m moved it by
  4.5e-10 on one draw, where another, ill-conditioned, was refused by the labour net at 5.0e-9
  (unit-1e.md §12 item 19); an idle-land equilibrium at the walk's ceiling is refused, as 1d
  refuses one on the line (item 8); an uncertified economy's scan finds two equilibria only
  when they are more than one cell apart (§5.5); a consumption tax equals a wage tax in real
  numbers, and in f64 the two allocations agree to the supply's sensitivity (unit-1f.md §5.5).

- **O31. The rest of the goods addendum** (GOODS-CHAIN §2, D-G9; docs/unit-1g.md §9). 1g built
  D-G10, the mapping and plants. Not built: G1 (categories buying machine-side goods; E2 is
  refused instead), G3 (several recipes for one good: a chain has one recipe per good by
  construction; until G3, one task type per recipe), G2 with G3 (several non-produced inputs
  cleared by recipe mixes), G4 (retirement by lot life; the engine burns δ geometrically,
  D-G2), G5 (machine goods on the task line; E1 is refused), O3 (a machine good per task
  segment) and O6 (the spatial equilibrium). Each is 1c bit for bit when unused, and each with
  interest or several recipes must count its roots or refuse (decision 70).
- **O32. The mapping where tapes are built.** `GoodsChain` is per period and keyed by strings; a
  tape's goods (materials, machine goods with κ, hours) and its yearly δ, ρ and J need a schema
  and a builder in worldgen or the probe's generator, which converts with `Clock::fraction`,
  `Clock::compound` and `Clock::ticks` (h4 checks the three against the goldens) and takes D-G8's
  J (J_b under M1, J_b + 1 under M3). With the goods chain's engine work (next step 6) or the
  demo's second pass (O27).
- **O33. What O28 still leaves** (after h8). 1d: the edge's κ bracket started at 0 rather than ζ
  (by reading, the same root, and only the bisection's step count would differ), a tie edge's
  share one double up (a change in the last bit of σ), Lemma B.1's flag without its shortage check (equivalent: a short
  point's P_s is NaN, so its funding test is false either way), and five of the first pass's
  survivors (the walk's and the corners' orders at a tie, the corner supply at equality, a tie's
  σ where f(1) ≥ 0, the all-human corner's ω at the base price, the pool share's zero guard). 1f:
  the exit-free scan's sides not passed to the count, which only `count_changes`' unit test sees.
  The O28 mutants that h8 kills are listed in docs/unit-1g.md §12 item 8.
- **O34. The plant's fixed point** (decision 185). Its uniqueness is not proved: with a recipe
  other than the bundle the ratio κ/ζ moves with the equilibrium's prices, and the map was a
  contraction on every instance tried (P2's moves halve each step, as the damping sets them).
  CAPACITY's recommendation, the bundle plant at s1, needs no fixed point. A plant built from the
  chain's own machine goods, CAPACITY's "different 1c economy", is a fixed recipe over machine
  services and is covered, not tried on a chain. Since the start could decide which fixed point is
  found, it is pinned by test: the unplanted economy's ratios (P1g.6).
- **O35. 1g's re-check, closed (2026-09-28).** The bounded verification's re-check of exactly
  the items P1g.6 fixed (the two factorisations of D-G10's check, `MachineEq.made`, the plant's
  build lag in the long run, and the fixed point's start) ran against `3f3a1bc` and passed with no
  issue: all 12 re-aimed mutants are killed (`D:/rustyecon-verify/1g-r2/mutants_r2.out`). 1g's
  bounded verification is complete.

O36–O40 are G1's (branch `g1`), numbered after O30 and apart from track 1g's O31–O35.

- **O36. What G0 moved to G1 that G1 did not build** (decision 217; GUI.md, the block "Amended
  at G1", item 15). The plots' overlay, difference and ratio against a parent (decision 106);
  re-making branches at launch, with §5.1 item 7's refusal of a base whose hash changed (107);
  the registry's and the inspector's ways into the editor (103); a lock on `session.ron`, so two
  windows cannot share a serial (110). And O20 but for its first item's two scan escapes
  (closed at G1.7): the three plot mutants that `every_drawn_vertex_is_recorded` misses (a y
  changed only where a line is lent, and a segment's last vertex dropped from record and line),
  G0.2's five surviving mutants of compare and the ancestor, and its two edge cases of
  provenance. A G1 second part takes them, each with a test that fails without it. One limit
  of any token scan stays (G1.11): a macro exported from `ui/` and invoked in `vm/` is not
  followed; `crates/observe` at G2, with no egui dependency, closes it by type.
- **O37. G1's verification: the re-check** (decision 218). One bounded verification ran over
  G1.1–G1.10 (`D:/rustyecon-verify/g1-r1/`), and G1.11 fixed what it found. It covered the
  presets and their goldens (every paired output within 8.5e-16), the `Debug` reader on every
  point type (every number token read), the knobs against the parameter types, the explainer
  under a rate change, a shock, `Ratio` and `Saturate`, the session's formats, and 44 mutants.
  Left: a re-check of G1.11's fixes; the waterfall at a run resumed from a ring checkpoint (its
  p₀ is the record's first tick, not genesis); breakpoints under a branch that resumes past a
  date; and a golden named after another output's key (O38), which pairing by name would take
  for that output's. *Amended at the merge of `g1` (2026-09-28):* the re-check ran after
  G1.11, on clones of `3d1ad6f` (`D:/rustyecon-verify/g1-r2/`, `mutate.py` and
  `mutations.txt`; the GUI's whole suite in release per mutant on Windows, 117 passing at
  baseline). The 19 mutants of round 1 that had survived, re-aimed where G1.11 moved their
  code, are all killed, each by a test G1.11 added or extended. Of its 22 new mutants and
  probes, two are killed (a flag painted the wrong way, a log step inverted) and one is not a
  probe (cargo refuses a second name for egui). It found no wrong number, and it left five
  majors open:
  - *painted values that no test reads back.* The lab's count of agreeing outputs, its "at x\*"
    note and an unpaired golden's text (LM1, LM6, LM7); a sweep's gaps bridged (LP1); the
    explainer's next-price, ln(next/p) and rate rows and its one-side note (EM1–EM3, EM5); and the
    waterfall's event colours, its count of moving events and its one-sided count (EW1, EW2,
    EW4) each survive: the scripts hold the charts, the headline figures and the verdict to
    their view-models, but not every row of text. And a golden of 0.0 (3 of the 216 paired
    cells: G5 flow's interest and capital share, K1's plot rent) takes the relative
    difference's zero branch, which a mutant can make always agree (L6b);
  - *five more ways past the no-egui and reach scans*, each a file in `vm/` naming
    `ui::layout::Pane`: an alias of an alias (`use crate as g; use g as h;`, Z4),
    `self::super::super` as a root (Z5), a group's `self` alias (`use crate::{self as g};`,
    Z8), a re-export at the crate root (`pub use ui::layout as lay;` in `lib.rs`, Z9), and a
    root alias exported by another module (`pub use crate as root;` in `vm/mod.rs`, Z11).
    Besides them, a path built from a `macro_rules!` argument (Z6) is the macro limit above
    (O36), and egui's colour crate under a second name in the manifest (`paint = { package =
    "ecolor" }`, Z7b) is not refused, as a renamed core is.
  Each wants a test that fails without its fix; G1's second part (O36) takes them, before G2.
- **O38. The lab's copies of the oracle** (decisions 202–204). The presets build the oracle's
  parameter types as struct literals (`MachineType`, `Recipe`, `WorkerType`, `Category`,
  `Parcel`, `PricedExit`, `Params`) and read `Eq1d::margin`, `Eq1e::land_market` and
  `exit_land` by name. Track 1g's addendum, or any change to those types, breaks the GUI's build
  where it adds a field, and the merge that brings it fixes the presets;
  `lab_presets_solve_to_their_goldens` then says whether each is still its golden's economy.
  Goldens are paired by name only: a generator that names a golden after another output's key
  pairs them, which O37's pass should look for. *Amended at the merge of `g1` (2026-09-28):*
  track 1g changed none of these types, fields or names (it added `goods.rs` and `plants.rs`
  and changed `MachineBlock::new`'s validation, D-G10), so the presets build unchanged and
  `lab_presets_solve_to_their_goldens` passes on all 16. The lab does not show unit 1g, and
  adding it is not a merge's change: a chain is a `GoodsChain` keyed by strings and a plant
  economy a `PlantEconomy` over `MachineParams`, whose equilibria (`ChainEq`, `PlantEq`) read
  out per good, machine and plant and have no `outputs()`, and whose 210 goldens
  (`goldens_1g.txt`) are named per good (`S1_COAL_PRICE`). A 1g unit in the lab needs presets
  transcribed from 1g's gate support (S1, S2, S2H, A0, the horse, L2's plants), knobs by good,
  a readout per good, machine and plant, pairing by those names, and a `Debug` reader check on
  their points. The cheapest first step is a preset per chain built through
  `GoodsChain::to_machine_params` and shown under unit 1c, without the goods' readouts. G1's
  second part (O36) takes it.
- **O39. O26's remainder** (decision 215). The demo compiler holds the dials to C2 but not the
  clock to 52 ticks a year; at 4 a year the tape runs to 46,548 dead county-ticks. And two of the
  close's three layout notes: the legend and the credit cover Cornwall at the fitted view, and
  the ranked table's value column is cut under a long lens name. The third, the health chip
  wrapped on a narrow window, is closed: G1.9 keeps a chip whole, and G1.11's toolbar test
  opens the demo world at 1,600 and 1,024 points, the chip in one or two lines. The demo's next
  pass (O27) takes the rest.
- **O40. What no headless test reaches** (decisions 206, 213). A snapshot's picture comes from
  the renderer, which kittest's harness does not have; the test hands the app a picture. The
  lab's sweeps of the heavier units (1e and 1f scan their paths) run on the UI thread and can
  hold a frame. Both are for the window check by hand; a sweep worker waits for need.

O41–O50 are the stocks probe's (P2.2a, branch `phase2-goods`), numbered after G1's O36–O40;
O41–O46 are HORSES-SPEC §9's, each with what the run found.

- **O41. CHAIN's horse county is unfunded: closed at L0.8 (2026-09-29)** (decision 221;
  HORSES-SPEC §1.1; decision 254). Provider baskets are −2.1476 at 1g's HORSE point and −2.1419
  with the loop cut: rent does not cover N·P_s, and it never can, since a basket is 1 good plus
  h = 1 of space, so P_s ≥ 1, while N·h = 12 exceeds T = 10. `chain8`, the same county at N 8,
  is funded with coverage 1.215–1.240 at every cost target and δ 4–10%, checked by the oracle and
  by a 50-digit solve that reads no oracle code (docs/probe/loops/FUNDED.md, amended by
  FUNDED-A1). 1g's goldens are unaffected.
- **O42. An ex-post category desk for v2a.2** (decision 222). P2.1's `CategoryDesk` is planned
  only, and D-G6 asks for ex-post assignment wherever machine-hours are used. With several
  segments the cutoff that uses up hours and services is not 1a's quadratic; it needs a rule, its
  nesting on one segment (P2.0's `assign_ex_post`) and a mirror.
- **O43. v1's b × 2 is unfunded on the demo's base county** (provider baskets −0.8026), so F5 and
  F6 ran 91 runs, not 93. Demo v1's check (WORLD.md §3.3) ran the probe's battery with b × 2 on
  every county without checking funding; the demo's second pass must.
- **O44. Slow makers with a fodder market.** The maker from bought inputs (a 0.005, ω 1, δ 4%) is
  near neutral once fodder is a market good (0.999986 a tick at b × 2 in the mirror). P8 diverges
  as registered, and its base kick set fails its tail bar at 40,000 ticks (1.4e-3, g 0.983 a
  year): a slow mode, stable. A CHAIN-like maker that buys its build's fodder at low δ needs its
  own dial scan (O49).
- **O45. The tick floor with a material market** (decision 237; GOODS-CHAIN open question 6). In
  the engine wet hire fails at 12 a year everywhere, and at 24 a year at both ω 1 instances: H2
  as predicted (1.00045 a tick) and H4 not (1.00038, predicted 0.99983). 52 and 365 a year agree.
- **O46. Capital's time scale, measured in the engine** (D-G14). After b × 2 the stock comes
  within 5% in 29–47 years at δ 8–10% (the paper's zero builds, 3.1–3.9) and 40–78 at 4%; after
  b × 0.5, which the paper fills in a tick, 34–44 years, overshooting 39–83%; the quasi-rent is
  +0.4% (ω ½) and +6.2% (ω 1) three ticks on. The departure stands, nothing in P2.2a scores it,
  and its candidate fix stays Phase 3's entry rule. *Read for the loop at P2.2b.4* (LOOPS.md §4;
  decision 309): after b × 2 the heads come within 5% in 10.1–12.5 years, 1.06–2.1 times the
  paper's zero builds, but the economy takes 43–67 years, and the maker's plant (19.5–36.7 years
  to 5%, not registered) is the last observable in for most runs (59 of LB1's 97, 82 of LB3's);
  after b × 0.5 the heads take 14–24 years and the whole 54–135. At δ_p 4% (LF2) b × 2 takes 76
  years. So Phase 2 proper's path scoring (O14) inherits a time set mostly by the assumed plant.
- **O47. The idle machine market: closed at L0.6 (2026-09-29)** (decision 236; GOODS-CHAIN open
  question 4; HORSES §3–§4; decisions 245–253). When orders stopped, the horse's price fell at its
  full rate with no floor. heads.capacity × 10 crossed the runaway bound at all 14 instances
  (ticks 138–167), and P8 at tick 138. The ω 1 verdict runs fell to 0.1–0.9% of target. The
  maker's reservation at ψ 0.25 now withholds the finished heads while the markup is below ψ. So
  the price falls at most one step below ψ times the replacement cost at the maker's last offer;
  the markup at current costs can go lower, 0.144–0.206 at heads × 10 (reworded at L0.7, after
  the engine review). Every heads × 10 run and P8 converge, and the battery's lowest horse price
  is 0.187–0.251 of target (docs/probe/results/idle/; the report, docs/probe/IDLE.md). The engine
  review found no refutation; its five minor findings are answered at L0.7, and the review's
  17-run sample, rerun from L0.7's build, is L0.6's evidence row for row. What it leaves is
  O52–O55.
- **O48. M6's load check had a hole: closed at L0.1 (2026-09-29).** `cast.rs` exempted the
  capacity and owner desks from M6 for every good they buy; now only for the durable good they
  hold (decision 240). No check refused a seller offering a storable good in full; M5 now does
  (decisions 241–243). The review's tape (`D:/rustyecon-p2g/review-fidelity/m6/`, H1 with fodder
  `Indefinite`, bought by the capacity desk alone) reproduced at `401f7b1`: it loads and runs
  520 ticks to the review's final hash, `0xb2b4c91c24a1006e`, the fodder desk holding 260.4
  fodder against 5.3 at genesis (49 times) and fodder's price at 3.6e-6 against 0.2, while the
  capacity desk holds none. So the pile-up is the seller's unit root, which M5 refuses; the
  buyer's hole let the tape in. It is now refused at `actors[desk.capacity].spec.running.goods`.
  Tests: `m6_nets_only_the_durable_good` and `m5_refuses_a_stored_good_offered_in_full`, each
  failing with its check undone (scratch `D:/rustyecon-p2l/m6/`). The engine review found that
  the review's tape is refused by M5 as well as M6, so the first test could not show M6 alone;
  L0.7 adds `m6_refuses_a_stored_good_no_role_offers_in_full`, a scripted seller of stored fodder
  that M5 does not check, which loads with the old exemption and is refused with M6.
- **O49. What v2a.1b needs beyond v2a.1: closed at P2.2b.4 (2026-09-30)**, its engine half GO
  at chain8 (P2.2b.0–P2.2b.4; [docs/probe/LOOPS.md](docs/probe/LOOPS.md)), read with the maker's
  reservation as part of the damper (decision 307) and narrowed by decision 308. As first
  written, its mirror half done at L0.8 (2026-09-29) (HORSES §6; decisions 260–271). CAPACITY's
  plant was unbuilt, how it composes with an M3 horse holding was
  untested, and the capacity desk passed a fodder shortfall one for one into horse-days (H2 after
  r × 2: 120 fodder ticks, 38 on horse-days, 21 on the good). The mirror's loop step
  (docs/probe/loops/LOOP-SPEC.md, re-registered with the engine's genesis carry by LOOP-SPEC-A1)
  settles each in the mirror: the plant on all three loop desks, beside the horses on the
  capacity desk, built from the running recipe (261); the capacity desk's plant takes the
  pass-through into horse-days from 146–150 dead ticks to 1–2; CHAIN's maker from bought fodder is
  the tested maker from bought inputs (269). Its engine half is P2.2b (next step 6). S1, with and
  without its pumping loop, waits for a funded steam county (269, 276).
- **O50. The kick with stocks** (HORSES §3, §5). At 365 a year the kick sits on its rounding
  floor (passing tails 3.0e-4 to 5.6e-4 against 1e-3), and at δ 4% it would likely fail on the
  floor alone. Price kicks read faster than the stock's slowest mode, which is exactly 1 − δ a
  year, so a kick set of stock and coin kicks, with horizons from 1 − δ, would measure what they
  miss. D̂₀ is infinite for Tier-3S runs whose orders stop in year one, so the vacuity test cannot
  fire there. P7 converged where the frame registered STUCK, on the mirror's path.

O51 on are the loop stage's groundwork's (L0, branch `phase2-loops`).

- **O51. No role handles a storable running good yet** (decisions 240, 241). Since L0.1 a tape
  whose fodder or fuel lives more than a tick is refused at load twice over: its seller offers it
  in full (M5) and its buyers do not net it (M6). v2a.4 and v2a.6 need a seller that offers
  I/(1 + b_G) (GOODS-CHAIN E1's seller with `post: Share` and a cover) and netting of storable
  running goods wherever they are bought, each with a mirror scan and a test before its stage.
- **O52. The maker's collapse through a long glut** (IDLE-SPEC's O51; decision 245).
  - The reservation bounds the price, not the maker. Through years without sales the maker spends
    its coin on heads it cannot sell, and its own herd wears. The glut then ends in a shortage.
  - Engine, heads × 10: the horse price reaches 8.1–120 times target, and installed heads fall to
    0.40–0.53. Horse-days are dead for 309–1,452 ticks, and at ω ½ labour for a further 998–2,706.
  - In the mirror, the unconditional floor order (IDLE-SPEC candidate (d), 0.5·δK\*) avoids it,
    at the cost of every glut path and a P8 fall to 0.049.
  - Entry and exit (Phase 3) is the other route. The 1750-like instance's switch of technique is
    where it will bind.
  - *Under rule B (P2.2b.4; LOOPS.md §4; decision 307):* the maker's coin collapses the same way,
    but no deep shortage follows. Through heads × 10 at LB1–LB3 its coin falls to 1.3e-43,
    3.4e-34 and 4.3e-90 of genesis and its output to about 1e-27 of target or below, for up to 50
    years; after it the heads stay at 0.89 of target or above and the horse price peaks at
    3.3–3.8 times target. After r × 2 at LB1 the coin falls to 4.0e-7. The reservation, not the
    plants alone, carries these runs: at ψ 0 each runs away (ticks 155–170).
- **O53. P8's slow mode stays** (IDLE-SPEC's O52). It is 0.999986 a tick at b × 2. So P8 is STUCK
  at 40,000 ticks and reaches tolerance only after 473 years (tick 24,601). The mirror has tilt 1
  on the maker's reservation move it to 0.9992 a tick, and P8 to 199 years. That belongs with O44
  and the loop mirror.
- **O54. A market that never reopens is argued, not run** (IDLE-SPEC's O53). At a switch of
  technique (the 1750-like instance's horse to steam), the old machine's price should rest within
  one step of ψ·p_rep, with neither offer nor bid. That instance's frame must run it and score the
  price as an idle market's. *Seen in part at P2.2b.4* (LOOPS.md §4): through a glut in the loop,
  with the maker withholding and no order, the horse price does not move, at 0.223 of target for
  68–2,510 ticks (up to 48 years at LB3 heads × 10), and from 0.149–0.150 (drifting up as v moves) for 66–282 ticks after
  r × 2. Those markets reopen; one that never does is still unrun.
- **O55. The chatter after a glut, and the engine's sensitivity there** (IDLE-SPEC's O54).
  - When orders resume against a withheld pile, the offer switches on and off: 82–200 switches at
    heads × 10.
  - The step makes the path ulp-sensitive from that point. At H2 heads × 10 the mirror against
    itself, one ulp apart, parts by 0.11 in log, and the engine by 0.09 on the horse volume and
    1e-2 on prices.
  - The run's summary statistics still matched the mirror's to the tick. A scored path window,
    or a lens on the price, would read rounding there.
- **O56. Price-kick negative controls and D̂₀.** The harness takes a price displacement's D̂₀ as
  its genesis gap (PROBE-SPEC §4.5), 1e-6 for a 1e-9 kick. So such a run is VACUOUS unless it
  diverges or dies, however it orbits. The scan's `i_run.py` takes the first tick's D̂. E8's ψ 1
  kicks therefore missed their registered label (ORBITING), though their orbits match the
  mirror's tick for tick. A future control should register its label in the harness's reading, or
  the harness should report the first tick's D̂ beside D̂₀.
- **O57. The scan's mirror has no genesis carry.** P2.2a's `h_mirror.py`, and the scan built on
  it, leave out the one-tick goods' genesis carry (HORSES-SPEC §6.1). So an early transient can
  miss the engine by more than the tolerances: H2 r × 2's lowest horse price is 0.204 of target in
  the mirror and 0.220 in the engine. The trace diff's `i_carry.py` agrees with the engine within
  1.6e-12 on that run. A mirror that registers predictions for the engine should carry genesis.
  Done for the loop step at L0.8 (decisions 272, 273): its mirror carries genesis
  (`lm_carry.py`), and its predictions are registered again from it (LOOP-SPEC-A1).

O58 on are the loop registration's (L0.8): FUNDED's O51 and O52 and LOOP-SPEC's O-L1–O-L7,
renumbered (FUNDED-A1 §A1.4, LOOP-SPEC-A1 §A1.6), and the fix round's.

- **O58. The funded rule-B county is space-heavy** (FUNDED's O51; decision 254). The good is 1.6%
  of income and the horse chain 0.8% of the land, since CHAIN's labour-heavy horse sits at the
  task margin only where labour is cheap against land (v 0.023 of r). Funding is robust for that
  reason, but the loop's markets are small against land and the provider, and the good's price
  response sets L at 202,000–232,000 ticks. A county where the goods side weighs more needs γ's
  line or h moved, with its own funding and uniqueness check.
- **O59. The corner below land × 0.35** (FUNDED's O52). chain8's horse takes the whole line at
  land factors of 0.33 and below. At b × 0.5 the human share is 0.060–0.092. A P2.2b target below
  × 0.4 must be registered as a corner.
- **O60. The loop still shows under frozen prices** (LOOP-SPEC's O-L1). LB1's quantities grow
  1.0616 a tick with every price held (1.19 without plants). The plant's damping needs live
  prices, as CAPACITY found; the engine's open-loop probe will read the fodder market at order
  10^3 per unit of log price at lag 200.
- **O61. The absorbing zero is kept** (O-L2). heads × 0.1 takes the tasks' horse-days to 0.033 of
  target and comes back; without the fodder desk's plant the fodder desk's coin × 0.02 does not
  (LN4).
- **O62. S1 is not framed, with or without its pumping loop** (O-L3; decisions 269, 276). M3's
  county is unfunded at S1's point (P_s 4.749, N·P_s 19.0 against T 10). It needs a funded steam
  county first.
- **O63. Why the maker's plant removes the reservation's relay cycle** (O-L4). Shown, not
  derived: without it large displacements orbit though a 1e-8 kick decays (0.9973). A scan of ψ
  with and without the maker's plant would settle it.
- **O64. GOODS-CHAIN's stage table, v2a.1b's R1 row** (O-L5). "Fodder without horse-days is
  v2a.1" does not hold on rule B's numbers: the loop cut is CHAIN's maker on v2a.1's roles,
  NO-GO without plants (LN7), GO with them (LC1). The row should read "fodder without horse-days
  is LC1". GOODS-CHAIN is outside the repository and is not edited here.
- **O65. P2.2b's engine build: closed at P2.2b.4 (2026-09-30)**, built at P2.2b.1–P2.2b.2 and
  run at P2.2b.3, E0 first (LOOPS-RULES §14–§16; LOOPS.md). As first written (O-L6; next step
  6): the plant on three kinds as an optional
  field with appended `ActorState` variants; the genesis carry of a planted desk's bundles as
  decision 273 sets it; rule-B instances in `probe::horses` from `docs/probe/loops/instances.json`
  (the TypeDesk's horse-day input and the maker's fodder build need no new code); the plant
  observables, the 5% readouts, per-market dead ticks and ticks at labour's bound in the harness;
  and the trace diff first (E0).
- **O66. Labour's knife-edge dead counts** (O-L7; decision 274). After r × 2 chain8's labour
  supply opens at 0.508 of target and after N(2) the desks' coin buys half the labour, both
  within 2% of the dead bar. Labour's count is reported, not scored; a county with headroom there
  would let it be scored.
- **O67. Labour supply at its bound** (decision 277). Five runs at each of LB1–LB3 put every head
  to work for 18–273 ticks, and four or five at the flow controls (LOOP-SPEC-A1 §A1.4). No probe
  has scored runs in that regime. If P2.2b's engine parts from the mirror there, the frame must
  say whether the bound or the loop did it. *At P2.2b.3 it did not part*: the 28 runs at the
  bound have A1's ticks and peaks and each converges (E11). Labour's headroom elsewhere is O66.
- **O68. The planted desk's genesis carry: closed at P2.2b.4 (2026-09-30).** E0 found nothing
  parting at tick 1 (P2.2b.2; e0.md), the carry test binds where the carry does, and every scored
  run's ticks to tolerance are the mirror's with the carry. As first written: a convention no
  engine has run (decision 273). The mirror sets it; E0's trace diff at tick 1 tests it. If the
  engine's build does otherwise, the
  registration must be amended before any scored run, not after. LOOPS-RULES §5 (P2.2b.0): the
  engine needs no carry code, since a planted desk reads its bundles over every unit it holds;
  the test `loops_carry_meets_the_mirror_at_tick_one` holds it in the gate.

O69 on are P2.2b's (`docs/probe/LOOPS-RULES.md` §12).

- **O69. The untraded good has no valuation.** A plant shows in holdings, but no lens or report
  values it; at replacement cost a unit is worth P_K, about 99 weeks of its desk's revenue at
  θ 0.8 and δ_p 10%. GOODS-CHAIN's work in progress (its E5) should reuse the untraded good.
- **O70. Two readings of the lowest baskets.** The engine's is the sum of each household's
  baskets (P2.2a's); the mirror's the smaller of the goods and the space eaten over both
  households. They agree when both bind on the same item; the report names any row where they
  part.
- **O71. LN5 and LF4 are registered and unrun** (LOOP-SPEC-A2; decision 291). They run when O51's
  roles exist, against A1's numbers.
- **O72. The run's cost.** Closed at P2.2b.3: the whole protocol, 3,549 runs and 24 kick sets,
  took 74 minutes on 46 WSL threads; a 10·L run took 220–400 s there, three to six times its 65 s
  alone, as the threads share cores. As first written: L is 202,000–232,000 ticks and Tier 3 and
  3S run again at 10·L, about 2.3 million ticks for each of 53 runs per verdict instance, beside
  E4's kick sets at H = L; one LB1 run at 10·L should be timed on WSL before the waves are
  planned. Timed at P2.2b.2: LB1
  w × 2 at 10·L (2,110,000 ticks) takes 65 s on one WSL core, 4.6 MB resident. So one verdict
  instance's Tier 3 and 3S at 10·L (53 runs) take about an hour of one core.

O73 on are P2.2b.1's (`docs/probe/LOOPS-RULES.md` §14).

- **O73. The harness's rule-B readouts are not built.** Closed at P2.2b.2 (LOOPS-RULES §15):
  the harness, its readouts and the five tests are built. LOOPS-RULES §8 (observables with the
  plants, per-market dead ticks, labour's bound, horse-days by buyer, `pk_high`, the plants' 5%
  readouts, the running cost read from the tape, the run grammar's plant stocks and `b*F`, the
  families and the outputs) and five §10 tests (`loops_batteries_are_registered`,
  `loops_runs_apply_as_named`, `loops_harness_readouts_are_the_rows`,
  `harness_reads_the_running_cost_from_the_tape`, `loops_carry_meets_the_mirror_at_tick_one`).
  `horses` reads rule A's instances alone until then. E0's trace diff needs only the tapes, which
  exist.
- **O74. §10's reservation test was worded backwards.** Since c_full > c_m, "a markup below ψ on
  c_m but above it on c_full" cannot happen; the test takes the case that tells the two apart,
  above ψ on c_m and below it on c_full (LOOPS-RULES §14.2 item 5). Nothing to do but read §10
  with §14.

O75 on are P2.2b.2's (`docs/probe/LOOPS-RULES.md` §15).

- **O75. The mirror's run-length term is a table in the code.** `mirror_three_t6` copies
  `lm_lin.json`'s 3·T6 per instance. If the mirror is registered again, the table must follow.
  The engine's own term, 200·τ_max, comes from `horses elasticity`, which prints L.
- **O76. E0's single-ulp column understates the rounding spread.** The mirror against itself with
  one ulp at genesis (HORSES-RULES §6.4's column) parts less than the engine does wherever the
  horse market's volume cancels, since the two sides round apart every tick, not once. The trace
  diff now also reports the mirror with an ulp each tick, which covers every parting. Later trace
  diffs (the goods chain's next stages) should report both.

O77 on are P2.2b.3's (`docs/probe/LOOPS-RULES.md` §16).

- **O77. The dead bar's ties.** A stock or coin at × 0.5 can leave its market at exactly half its
  target at tick 0 or 1 (the good at 0.49999999999999983 of Y\* after the good desk's coin
  × 0.5), and the last ulp then decides whether the tick is dead. The engine and the mirror part
  there by one
  tick in 13 runs (26 with their 10·L twins), inside the 5-tick tolerance. A later bar compared
  across two codes should expect it, or count a market dead below half its target less a margin.
- **O78. Two fits of one kick envelope.** The engine's g (P2.2a's reading, to ten times the
  rounding floor) and the mirror's `lm_kick` (to 1e-4 of the peak over 30,000 ticks) agree within
  0.005 a year at LB1–LB3 but part by 0.010–0.016 at LW1, inside E4's 0.03. A registration that
  needs g finer than 0.02 should fix one reading for both sides.

O79 on are P2.2b.4's ([docs/probe/LOOPS.md](docs/probe/LOOPS.md); `docs/probe/LOOPS-RULES.md`
§17).

- **O79. What the loop's GO does not cover** (decision 308; the fidelity review). P2.2b's GO is
  rule B's horse at chain8, with the design chosen in the mirror on the same battery. Untested:
  - C2m, Phase 2 proper's default (decision 119), where CAPACITY's plant at θ 0.8 is not GO in
    the mirror (74/77, 113/119) and θ 0.7 is;
  - storable fodder or grain (O51; LF4 and LN5, O71);
  - a loop that weighs on land: chain8's horse chain takes 0.5% of it (O58);
  - other funded counties: chain8 was funded only by moving N from 12 to 8;
  - 12 ticks a year (0 of 40), and S1 (O62).

  Each goods-chain instance with a loop registers its own predictions at its dials first.
- **O80. Idle ticks belong beside dead ticks.** The harness counts a machine market that clears
  below half its target as idle, not dead (P2.2a's convention). So E6's "no dead tick" stood
  beside 353–3,047 idle horse-market ticks, and a frozen price, until the fidelity review. A
  later registration or report that scores dead ticks should print the machine market's idle
  ticks, its no-order ticks and the maker's lowest coin in the same table.

O95 on are Phase 2 proper's (P2.3.0, branch `phase2-proper`; this line's range is O95–O109).

- **O95. The oracle addenda that later instances need** (decisions 360, 372, 377, 379, 385, 387,
  388; P2.3.0). The wall and commons instances need none. Each is a spec, a generator and
  goldens, as units 1d–1g were, with nesting bit for bit, before the first instance that needs
  it:
  - cells in the equilibrium, the marginal cell's split (a 1b addendum, 360), with a probe of its
    own, before an instance on the agents' own cells;
  - a reserved type in machine recipes (1d, 372), before trained machine builders;
  - reserved tasks under the priced form, 1e §2.9's fixed point in Y (377), before a walled type
    under s(q), the eras' trained type in the 1750-like instance;
  - a second scarce land class (1e §9, 379), before a region has two;
  - walled types under in-work benefits (1f §2.8, 385), before Speenhamland;
  - a basket per worker type (1f §10, 388), before owners buy domestic service, and Stone-Geary
    around a subsistence basket (387) if the agents' consumption rule is to be PLAN §3.2's.
- **O96. The land market at zero rent** (decision 376; MARKETS §6; next step 7). At an idle-land
  point the oracle's rent is 0 and the market's land is not all used. In the agents' land market
  there, demand stays below supply at every positive rent, so under `Saturate` the unsold price
  falls without end, past the harness's runaway bound (1e-6 of genesis; MARKETS-RULES §5). It
  needs an answer without a clamp (R3); a step on the seller's offer is allowed, as L0's maker's
  reservation is (IDLE.md). The candidates are chosen by a mirror scan, and the one chosen is
  registered with its predictions and built before the commons instance's registration. PLAN
  §3.1 puts an idle parcel's reservation rent at zero, so a remedy with a positive floor moves
  the point off r = 0 (GOODS-CHAIN's open question 2); the comparison is in wage units (decision
  153). *Amended at P2.3.5:* the commons frame's scan answered it for the commons (decision 398:
  no market, the participation rule gives out the plots). What stays open is idle enclosed land at
  r = 0, the frame's O96: under the probe's transfer the provider's only income is rent, so its
  baskets are −ν·N there and the idle stretch is unfunded by construction. A positive-price market
  cannot rest at 0 there either (the commons frame's §1.5: Saturate runs away, Hold never acts, a
  reservation orbits, a floor order leaves a continuum). It needs a funded support (a 1f transfer,
  or the workers' own) and a free-good rule for land in markets, in its own frame, when an
  instance needs it.
- **O97. A type that sells reserved and pool hours** (the wall frame's OW1; decision 394). 1d's
  pooled type with reserved work (E1, E4), pooled types with ε ≠ 1, and support ν ≠ 1 (decision
  138): IW1 has none. Each needs a rule that splits a pop's hours between two markets, or scales
  them to efficiency units, with its nesting, a mirror and its own registration before a Phase 2
  instance uses it. The eras' trained worker, who holds engines' tasks at some dials and the pool
  at others, is such a type. It joins O95's list where it needs the oracle.
- **O98. At the wall, labour-demand shocks move wages, not output, but halve output on the way**
  (OW2). In the mirror tail.services × 2 and res.services.trained × 2 move Y\* by 0 in log and
  bottom at 0.516 and 0.500 of it. O14 and O24 carry this as the wall's path cost. Ex-post
  assignment might lift it, and is untested here.
- **O99. O21's one-type cousin at the wall** (OW3). In the mirror's history family a step that
  cuts the machine desk's cost 3.5 times at the prices in force (land.mach 0.8 → 0.2) makes it
  keep its whole stock for a tick (a·q ≥ held), and no baskets follow the next tick, in all 16
  such windows. It recovers each time. A keep rule that shares the shortfall, or a stock (Phase
  3), would remove it.
- **O100. The subsistence trap** (the commons frame's O95; decision 399). Under the priced exit,
  with the exit good made with labour and valued at its posted price, the agents have a second
  absorbing state: nobody works, food is not supplied, and every price inflates together. The
  mirror predicts it in C1's joint2 (4/60), joint4 (12/40) and basin (89/430), C1's tilt-1 Tier 3
  (3), C1's land.mach history (window 5), C2's joint4 (8/40) and basin (77/430), and the negative
  control's Tier 3 (9); never in the verdict battery, and never at I1 (0/530). It needs a
  structural answer, chosen by its own mirror scan, before the 1750-like instance, where food is
  the exit good by decision 151. Candidates, unscanned: participation adjusting toward its target
  at a rate, entry for food at zero output, a storable exit good (O51).
- **O101. A commons shared by several types** (the frame's O97). The rule gives out one pop's
  commons from that pop's own demand; several types sharing one (1e's instance T) would need each
  other's plot demand, which R13 forbids. They need the commons as a market (exact only while
  crowded) or a commons actor. The 1750-like instance with several types needs it.
- **O102. The Enclosed regime's accounts** (the frame's O98). Plots on enclosed land pay rent in
  money, not in kind (decision 152's home account). The households' split of baskets differs from
  the home account's by r·T_p/P_s, and the provider's baskets from the oracle's; the market
  allocation is the oracle's. At rest it matters only at C1's four Enclosed targets.
- **O103. C2's commons shocks × 1.1, × 0.9 and × 2 are VACUOUS by construction** (the frame's O99).
  In the Commons regime the commons' size does not move the point.
- **O104. 12 ticks a year is slow here too** (the frame's O100): 76–80 years to tolerance, median,
  over Tiers 1–2, as I1's 90 (decision 121).
- **O105. C1's x\* lies 0.0017 below care's edge 0.75, and C2's 0.0065** (the frame's O101).
  b.food × 0.9 crosses it; which s[care] runs are slack depends on the side x\* is on.
- **O106. The joint family's draws** (the frame's O102). The mirror draws in its own market order;
  the registration's reading makes the harness draw in the same order at C1 and C2, and E0 checks
  it. If E0 finds the draws unequal, the per-seed predictions fall and the counts are the
  prediction (C1 4/60 and 12/40, C2 0/60 and 8/40), within 25%. *Closed at P2.3.9 and P2.3.15:*
  E0 found the draws equal, and every seed's class is the mirror's.
- **O107. The runaway bound's reference** (P2.3.15; results/commons/README.md). PROBE-SPEC §4.5's
  bound, every posted price within [1e-6, 1e6] × genesis, is read by the harness against the run's
  displaced genesis prices (since P2.1) and by the commons mirror's runner against the undisplaced
  point. In the trap, where labour's or food's price was displaced, the runaway ticks differ by up
  to 29 ticks (10.5%), and 65 lines of the commons fail the registered 5%; the mirror against the
  harness's reference gives the engine's tick in 190 of 195 trap runs and one tick apart in 5.
  Classes do not move. A registration that scores runaway ticks should name the reference; the
  harness's (displaced) is the one every P2 wave has used. Alternative: read the bound against the
  oracle's point in the harness, which moves every runaway tick P2.1–P2.3 reported.
- **O108. Two end-of-run lines registered against the wrong value** (P2.3.15). (1) The wall's E6:
  a displaced share stalls where a·s rounds to 0, at 10 subnormal ulps (5e-323) at 52 ticks a year
  but 70 (3.46e-322) at 365, so four runs at 365 a year fail the scorer's "at most 5e-323" line,
  with their thresholds at 1.0 exactly and every labour market trading. Read after the result as
  the wall, not a refutation of "a desk's share not back to 0"; **open to veto.** (2) The commons'
  r_o at the end within 1e-9 of the mirror's, which the mirror reads where it stopped early: at 12
  and 365 a year that is up to 7.0e-8 off the oracle's, so 58 lines at C1 fail while the engine
  ends within 3.0e-13 of the oracle's. A registration should put an end value against the oracle,
  or run its mirror to L. Neither changes a class, a tick or a verdict.
- **O109. O22's families on I1–I3 are still unrun** (decision 368 orders them before the wall and
  commons batteries). P2.3's wave ran the two frames' registered protocols only; the families
  there have no registered predictions and are reported. They are their own wave, stocks first.

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

G0 is closed but for your look at its window, and the demo world but for your look at its map.
The many-markets probe is closed, Phase 1 is closed (O3) and merged at `16eb728`, and unit 1g
and G1, the oracle lab, are merged at `43ad8c5`, G1 but for its window check by hand. The stocks
probe (P2.2a) is closed on branch `phase2-goods`, GO for the goods chain's stage v2a.1. What
follows joins the lines' lists. Steps 5 and 6 build on the goods chain's design evidence, which
is outside the repository ("Where things stand"), and each needs your rulings first.

1. **Your look at the window**, the G0 gate's item checked by hand (the command is under "G0
   is closed" in "Where things stand", with how to try the editor). G0 is otherwise closed
   (G0.3, 2026-09-27; O1). If the window fails, a G0.4 fixes it first (decision 114), on top of
   `reboot`'s line.
2. **Your look at the demo's map** (the command is under "The demo world and its map are
   closed" in "Where things stand"). If it fails your look, a D.6 fixes it first, on top of
   `reboot`'s line.
3. **G1: your look at its window, then its remainder and its re-check's findings** (GUI.md §9,
   the block "Amended at G1"; decisions 200–219). The lab, a field over x, sweeps, the explainer
   and the waterfall, log axes, the watchlist, event and date breakpoints and PNG snapshots are
   built (G1.1–G1.10), verified once and fixed (G1.11), and merged at `43ad8c5`, every gate
   item met but the window's p90 by hand (the command is in "Open — your calls", and under "G1,
   the oracle lab, is built" in "Where things stand"). If the window fails your look, a G1.12
   fixes it first, on top of `reboot`'s line. Then a G1 second part takes what G0 moved to it
   (O36: overlay, difference and ratio against a parent, re-making branches at launch, the ways
   into the editor, a lock on `session.ron`, and the rest of O20), the five majors the re-check
   of G1.11 left (O37: painted values not read back, and five more ways past the scans), and
   unit 1g in the lab (O38), each with a test that fails without it, before G2.
4. **The engine re-exports `num`, the tape's raw schema, `Basis` and `Unit`: done at G1.1**
   (decision 200; ENGINE, amended at G1.1). The GUI's edge to core is gone, and
   `scripts/gate.sh`'s check of the GUI builds.
5. **The oracle's addendum for machines built from goods** (GOODS-CHAIN §2): **built** as unit
   1g (P1g.1–P1g.7, "Where things stand"), verified with one fix round (P1g.6), and merged at
   `43ad8c5`. What is left: your rulings on decisions 179 (67 reworded, D-G1) and 180
   (D-G10), and on 181–190; the re-check of the fixed items (O35), before Phase 2 compares
   agents against the goods chain; the mapping's call where tapes are built (O32); and the rest
   of the addendum when its instances need it (O31). Machine recipes stay on pool labour
   (decision 140).
6. **The goods chain in the engine: P2.2b, the loops, is closed** (P2.2b.0–P2.2b.4,
   2026-09-30; [docs/probe/LOOPS.md](docs/probe/LOOPS.md); GOODS-CHAIN §3–§6; CAPACITY.md).
   Stage v2a.1b, rule B's horse loop at chain8, is GO at C2g and 52 ticks a year with CAPACITY's
   plant on every loop desk and the maker's reservation together, read narrower by its reviews
   (decisions 286–311). Before it: **P2.2a** made stage v2a.1 GO
   ([docs/probe/HORSES.md](docs/probe/HORSES.md); decisions 220–239), and **L0** built the
   maker's reservation (O47; [docs/probe/IDLE.md](docs/probe/IDLE.md)), narrowed M6, found the
   funded county chain8 and registered the mirror's loop step (decisions 240–285).

   **What follows on this line**, in order, each after your rulings on 284/308, 285 and 286–311:
   1. **Storable running goods (O51)**: GOODS-CHAIN E1's seller with a cover and buyer netting,
      each with a mirror scan and a test before its stage, then LF4 and LN5 against A1's
      registered numbers (O71). v2a.4 and v2a.6 need them.
   2. **A funded steam county for S1** (O62; decisions 269, 276), found and checked as FUNDED did
      the horse's, with and without its pumping loop.
   3. **The goods chain's 1750-like instance with a loop** (step 7), only through its own mirror
      registration (decision 308): at its dials (C2m at θ 0.7, CAPACITY's alternative, or your
      ruling that the goods chain runs at C2g, decision 226), in a funded county, with the
      horse-to-steam switch's idle market run and scored as one (O54), each stock's 5% time
      registered (decision 309), idle ticks beside dead ticks (O80), and the scorer committed
      before its wave (decision 311).
   4. **Left open:** the maker's collapse through a long glut (O52, now seen under rule B), P8's
      slow mode (O53), why the maker's plant removes the relay cycle (O63), and O79's untested
      conditions.

   **P2.2b, as it ran**:
   1. **The frame** (P2.2b.0): **done**, 2026-09-30.
      [docs/probe/LOOPS-RULES.md](docs/probe/LOOPS-RULES.md) is the build's spec: each new field,
      `ActorState` variant, load check, readout and test, and how E0–E11 are run and scored.
      One departure, LOOP-SPEC-A2 (LN5 waits for O51), dated and hashed before any code. The
      build must also add an untraded good to core (decision 286).
   2. **The build** (P2.2b.1): **built**, 2026-09-30 (LOOPS-RULES §14), but the harness's rule-B
      readouts (O73). CAPACITY's plant on the TypeDesk, the capacity desk and the maker,
      an optional `plant` field with appended `ActorState` variants so every committed tape keeps
      its hashes; a plant good per loop desk, held and never traded (D-G2's `Indefinite` good, worn
      by `Depreciation` burns); s1 bundles of the desk's own recipe per plant unit, ordered by M3's
      rule with s_K = 2δ_p, installed next tick; on the capacity desk beside the horses, from the
      running recipe (261), its target the herd's plant (262); the genesis carry of a planted
      desk's bundles as decision 273 sets it (O68); rule-B tapes from `instances.json` at chain8
      (254), the reservation on (265). At θ = 1 it must be the plant-free run bit for bit, and with
      no plant every committed tape's text, `world_id` and hash stream. The oracle needs no new
      solve at ρ = 0: 1g's plants give K\*, V and the user cost per plant desk (decision 184); the
      tape's goods need O32's schema and builder.
   3. **The harness's rule-B readouts, and E0–E2** (P2.2b.2): **done**, 2026-09-30
      (LOOPS-RULES §15; [results/loops/e0.md](docs/probe/results/loops/e0.md)). E0's trace diff
      passes with no amendment; E1 and mode A hold.
   4. **The scored runs** (P2.2b.3): **done**, 2026-09-30 (LOOPS-RULES §16;
      [results/loops/README.md](docs/probe/results/loops/README.md)). E3–E11 hold, every scored
      line; LB1–LB3 GO; no refutation.
   5. **The reviews and the report** (P2.2b.4): **done**, 2026-09-30
      ([docs/probe/LOOPS.md](docs/probe/LOOPS.md); LOOPS-RULES §17). Two bounded reviews,
      measurement and fidelity, both found that the verdict holds. The fix round answers their
      three majors and five minors with tables and disclosures; no code changed. LN5 and LF4
      wait for O51 (O71).

   Loops enter Phase 2 on plants, each loop instance through its own registration (decision 284,
   amending 120, narrowed by 308), and machine stocks come forward for the goods chain (decision
   285, D-G11), all taken by Claude on your word and open to veto.
7. **Phase 2 proper** (PLAN Phase 2), after the rulings on the decisions that bind it: 61, 67
   and 70 from 1b and 1c, 118–123 from the markets probe, and 135, 137, 139, 140, 147, 149, 151,
   153–155, 158, 160–162, 164, 165, 167–169 and 173 from 1d–1f, and 179 and 186 from 1g for the
   goods chain's instances. The markets probe recommends keeping 67 and 70 (MARKETS §6). It
   opens on loop-free wall and commons instances, which need neither step 5 nor step 6, with the
   markets probe's roles, the `probe::markets` harness and C2m at 52 ticks a year (decisions
   119–121): beside the Appendix B instance (1a), a wall-regime instance, a solved wall (1d;
   decision 137), and an open-commons instance under the default exit form, with the commons'
   shadow rent and idle land at zero rent (1e; decisions 149, 153, 160 and 161). The 1750-like
   instance follows (1e and 1f, with the common basket unless a basket per type is ruled in,
   decision 173), as the goods chain at ρ = 0 if D-G1 and D-G11 are ruled in. The agents'
   participation rule under the default form is decision 149's, and under a government 165's.
   Its battery runs O22's families first, stocks first. The goods chain's 1750-like instance
   waits for the idle market's remedy (O47), since a horse-to-steam switch idles a machine
   market by construction; P2.2a is its first engine evidence, machine stocks GO at weekly ticks
   (the floor, decision 237) with L from the slowest mode and O14 read like for like (decision
   235). *Amended at P2.2b.4:* the remedy is built (L0), and P2.2b is the first engine evidence
   for a loop with plants and the reservation. A loop enters this instance only through its own
   registration at its dials and county (decision 308; step 6's list, item 3). The commons'
   idle land needs an answer, without a clamp, for a market at zero rent,
   whose unsold price `Saturate` runs to the runaway bound. Many markets is no longer the first
   untested risk. The risks now are paths (O14, O24) and the new margins of 1d and 1e: several
   labour markets and the wall at x\* = 1. O30's precision sets the bands of any comparison
   against them.

   *Amended at P2.3.0 (2026-09-30):* the rulings are taken, by Claude on your word, as decisions
   360–393, each open to veto. What they set for this run, on branch `phase2-proper`:
   1. **O22's families first**, on I1–I3 as registered, stocks first (368).
   2. **Two instances at C2m, 52 ticks a year and ρ 0**, the fixed basket in the oracle and the
      agents, no loop and no government (361, 364–366, 387, 393). Each carries a stocks tier in
      its verdict (368) and is registered, trace-diffed and scored as P2.2a and P2.2b were, its
      scorer committed before its wave (decision 311).
      - **The wall instance** (1d): a trained type walled on its reserved tasks beside pooled
        types, so 1d's dependence form, the named alternative (377); a solved wall with its
        distance from each edge registered at genesis and at every cost target (370); types in
        one shape with an efficiency each (369); pooled hours compared through supply or in
        total, the pool's market form the frame's call (371); type desks on pool labour (372).
        *Registered at P2.3.1:* IW1 (decisions 394–397;
        docs/probe/results/wall/registration.md). *Built at P2.3.2* (docs/probe/WALL-RULES.md).
        E0, the scorer and the scored wave follow, in that order. *E0–E2 pass at P2.3.4; scored
        at P2.3.15:* GO (docs/probe/results/wall/README.md).
      - **The open-commons instance** (1e): s(q), every type pooled (377), food as its exit good
        (375, 380), one land service (379), certified by 1e's Proposition 5 (378), the crowded
        commons' shadow rent posted (374), idle land compared in wage units, `Idle` plots read
        as such (376, 381). Its zero-rent land market's remedy comes first: a mirror scan
        chooses it, and it is registered and built before the instance's registration (O96).
        *Registered at P2.3.5:* C1 and C2, with the scan's answer, the commons as no market,
        built into the same step (decisions 398–399; docs/probe/results/commons/registration.md).
        *Built at P2.3.6* (docs/probe/COMMONS-RULES.md); *E0–E2 pass at P2.3.9* under
        amendments A1 and A2 (docs/probe/results/commons/e0.md). The scorer and the scored wave
        follow, in that order. *Scored at P2.3.15:* C1 and C2 GO
        (docs/probe/results/commons/README.md).
   3. **Later**: the 1750-like instance takes the common basket (388), 1f's tax bases and
      closure if it has a government (382–386), C2g for any machine stock (391), capital's time
      unscored (392), and each addendum of O95 before the feature that needs it.
8. **The demo's second pass, on the many-market roles** (O27; WORLD.md §7). It takes O26 first,
   the clock held to 52 among it. Then come goods as unit 1b's categories and machine types as
   unit 1c's, on the many-market roles at C2m with no loop of produced inputs, with genesis from
   1b and 1c at each county; GOODS-CHAIN §5 proposes these as a chain of goods instead, stage by
   stage from its rule A (D-G12). P2.2a makes stage v2a.1 ready for it on these terms: GO on the
   demo's own county (F5, F6) on its funded targets, at C2g with the clock held at 52 (O39),
   each county's cost targets checked for funding first (O43), and the horse's price shown as an
   idle market's, not a valuation (O47). Stage v2a.1b (rule B's loop, P2.2b) joins only on
   counties funded and registered as chain8 was (LOOPS.md §6; decision 308); its horse price
   can rest at 0.15–0.22 of target for years. After that, the probe's battery county by county, the
   long run's dead ticks and shortfalls checked again, and lenses by category and type. It gets
   the same bounded verification as D.2 and D.3. Carriers come later, when transport desks,
   home-node trading (Phases 4 and 9) and an oracle with trade exist. They switch on from zero
   capacity, and the map then draws flows on channels.

## File map

```
STATE.md                 you are here; start here next session
README.md                what rustyecon is, the crates, how to build and test
docs/PLAN.md             the plan, amended by the addendum's rulings (2026-09-25) and decision 39
docs/ENGINE.md           the Phase 0 engine contract, with each step's amendments (P0.3–G0.3,
                         P2.1.1, D.2, G1.1)
docs/CERTIFY.md          session 2's contract: criteria, batteries, kick, seal, manifest, cli,
                         telemetry, with each step's amendments (S2.2–S2.6)
docs/TAPE.md             the tape's schema guide
docs/GUI.md              the GUI's design (A14): stack, architecture, panels, editor, map, roadmap;
                         amended at G0.1, in its two parts, and at G0.2; closed at G0.3; the map
                         and lenses brought forward at D.3–D.5 (branch demo-world); amended at
                         G1 and after its verification (G1.10, G1.11)
docs/demo/WORLD.md       the illustrative demo world: its tables, history, lenses, compiler, run
docs/demo/*.png          two screenshots of the demo's map, rendered headlessly (D.5)
docs/reboot/             REVIEW.md and ADDENDUM.md, kept as written (links fixed) but for A14 and
                         rulings 5–8 (P0.11); GUI-review-ledger.md, the GUI design's two reviews
docs/timeline/eras.md    era research for worldgen
crates/core              ids, clock, inventory, deltas, apply, ledgers, hash, checkpoints, tape
crates/markets           admission, clearing, settlement, prices
crates/agents            the behaviour seam, the scripted actor, the Appendix B roles (P2.0.1),
                         the many-market roles in roles/many/ (P2.1.1), the stock roles in
                         roles/stock/ (P2.2.1), CAPACITY's plant in roles/plant/ (P2.2b.1)
crates/probe             the Phase 2 probe's harness and tape generator (P2.0.1); reads certify's
                         measures (S2.5); the markets probe's harness, probe::markets (P2.1.1),
                         with the wall's instances IW1 and IC1 (P2.3.2);
                         the stocks probe's, probe::horses (P2.2.1); the loop step's instances
                         and tapes, probe::horses::loops (P2.2b.1)
docs/probe/RULES.md      the probe's rules, dials and lineage, as built
docs/probe/REPORT.md     the probe's report: verdict, battery, dial map, reviews, what it means
docs/probe/figs/         the report's plots; docs/probe/results/ its three summary tables (CSV)
docs/probe/MARKETS*.md   the markets probe's rules as built (MARKETS-RULES.md) and its report
                         (MARKETS.md), with figs/markets/ and results/markets/ (six CSVs)
docs/probe/HORSES*.md    the stocks probe's rules as built (HORSES-RULES.md) and its report
                         (HORSES.md), with figs/horses/ and results/horses/ (the registration,
                         its run lists and fifteen CSVs); HORSES-RULES §10 is the maker's
                         reservation (L0.4, L0.5)
docs/probe/results/idle/ the idle market's remedy on the engine (L0.3–L0.6): its registration,
                         README (the verdict, prediction by prediction), seven CSVs and the trace
                         diff, and IDLE-SPEC-A1 (L0.7); its plots in docs/probe/figs/idle/
docs/probe/IDLE.md       the report of the remedy and M6 (L0.1–L0.7)
docs/probe/loops/        P2.2b's frame inputs, byte for byte as registered (L0.8): LOOP-SPEC.md
                         (the mirror's loop step) and FUNDED.md with instances.json (chain8),
                         each with its dated amendment (-A1), LOOP-SPEC-A2 (P2.2b.0), and
                         SHA256SUMS
docs/probe/LOOPS-RULES.md P2.2b's build spec (P2.2b.0): the plant, the planted roles, the
                         untraded good, load checks, instances, harness readouts, E0-E11, tests;
                         as built and run (§14-§16), and the reviews' fix round (§17)
docs/probe/LOOPS.md      P2.2b's report (P2.2b.4): verdict, E0-E11, the reservation's part,
                         O14 like for like, capital's time, the reviews, what it means
docs/probe/wall/         the wall frame's inputs, byte for byte as registered (P2.3.1):
                         SPEC.md, SHA256SUMS and registered/ (the run-by-run outputs the scorer
                         reads)
docs/probe/WALL-RULES.md the wall instance's build as built (P2.3.2): the roles' three optional
                         fields, IW1 and IC1 on unit 1d, the harness at the wall, the checks
docs/probe/results/wall/ the wall's registration (P2.3.1) and its amendments, E0's record
docs/probe/commons/      the commons frame's inputs, as registered (P2.3.5, LF endings): SPEC.md,
                         SHA256SUMS and registered/ (the run-by-run outputs the scorer reads)
docs/probe/COMMONS-RULES.md the open-commons instances' build as built (P2.3.6): the workers'
                         exit, C1, C2 and C1N on unit 1e, the harness at the commons, the checks
docs/probe/results/commons/ the commons' registration (P2.3.5) with Tier 3S's mirror runs, its
                         amendments, E0's record
docs/probe/results/loops/ the loop step's registration (L0.8), quoting LOOP-SPEC-A1's
                         predictions and FUNDED's county, with its sha256; E0-E2 (e0.md,
                         P2.2b.2); the scored runs' README and CSVs (P2.2b.3) with the reviews'
                         tables (P2.2b.4); plots in docs/probe/figs/loops/
crates/engine            Sim, the tick, reports, resume, the replay audit, the registry listing
crates/cli               the rustyecon binary: run, resume, replay, registry, certify, worldgen,
                         licences
crates/oracle            the equilibrium solver, units 1a (P1.1) and 1b–1f (P1.2–P1.13),
                         Phase 1 closed at P1.14; unit 1g, machines as goods (P1g.1–P1g.7,
                         branch oracle-goods); its README, docs/unit-1{a,…,g}.md and
                         goldens/generate{,_1b,…,_1g}.py; tests/gate/p1_gate.rs, the gate
crates/certify           criteria, batteries, the kick, the sealed certificate, the manifest;
                         Parquet telemetry behind the feature `parquet` (S2.3–S2.5)
crates/certify/testdata  appb variants from `appb-tape --perturb`: bcycle, freeze, july, buffer16
crates/worldgen          the atlas's loader (D.1), the demo world's compiler (D.2) and its lens
                         measures (D.3); Phase 4's research compiler later
crates/gui               the GUI (G0.1, G0.2): model/, run/, edit/, vm/, drive/, platform/, ui/,
                         app.rs, the binary rustyecon-gui; its tests run under scripts/gui.sh
                         only (D1); the map pane and lenses since D.3 (ui/map.rs, vm/map.rs);
                         the oracle lab since G1 (lab/, vm/lab.rs, ui/lab.rs), the price-step
                         explainer (vm/pricestep.rs), the watchlist (vm/watch.rs), snapshots
                         (platform/snapshot.rs), and what the other charts lend egui
                         (ui/charts.rs; tests/common/paint.rs reads it back, G1.11)
crates/gui/tests/golden  the view-model goldens, one RON file per builder and point
                         (UPDATE_GOLDEN=1 rewrites them)
tapes/gate.ron           the gate world
tapes/appb.ron           the probe's Appendix B world, generated from the oracle
tapes/markets-<id>.ron   the markets probe's seven worlds (I0–I3, L2, L3, G1), from the oracle;
                         markets-iw1.ron, Phase 2 proper's wall instance, from unit 1d (P2.3.2)
tapes/horses-<id>.ron    the stocks probe's six worlds (H1–H4, R1a, P7), from 1g's ChainEconomy
tapes/loops-<id>.ron     the loop step's six worlds (LB1–LB3, LW1–LW3), rule B at chain8 with
                         CAPACITY's plants, from 1g's ChainEconomy (P2.2b.1)
tapes/demo-gb.ron        the illustrative demo world, compiled from worlds/demo-gb (D.2)
worlds/demo-gb/          the demo world's tables and derive.py
data/atlas/              the county atlas, gb.atlas.ron, under the ODbL: LICENSE, ATTRIBUTION,
                         README, its build script and pinned venv (D.1)
criteria/                each tape's dated criteria, registered before its first certified run
results/                 committed verdicts: results/<tape>/certificate.ron and manifest.ron
data/spine/              the spine's fetch, extract and eyeball scripts, manifests, CC0 files;
                         their cache is $SPINE_ROOT or the ignored data/spine/.cache/
docs/spine/              DATA_NOTES.md and EYEBALL.md, Breakpoint B's pre-look (S5.0)
scripts/gate.sh          the gate as one script; the GUI excluded, checked once on Linux (D1);
                         derive.py --check and the demo's long run by name (D.2)
scripts/gui.sh           the GUI's gate, run at each G-stage (G0.1); diffs the editor's branch
                         tapes against the cli too (G0.2); names 42 tests (G0.3), 55 with the
                         map's and the demo tape's hashes (D.3, D.4), 75 with G1's, 87 after
                         its verification (G1.11), and runs G1's sweep measurement by name on
                         Linux
.github/workflows/ci.yml hosted CI, on every push
```

## Repro notes

- The gate in WSL, from a Windows shell:
  `wsl -d ubuntu --exec bash -lc '<repo>/scripts/gate.sh'`. Always `--exec`: with `--` the exit
  code is lost. The script puts the build in `$HOME/scratch/target-rustyecon-gate` unless
  `CARGO_TARGET_DIR` says otherwise, and refuses a target directory inside the tree. With no
  network and a warm cache, set `CARGO_NET_OFFLINE=true`.
- The GUI's gate the same way: `wsl -d ubuntu --exec bash -lc '<repo>/scripts/gui.sh'` (default
  target `$HOME/scratch/target-rustyecon-gui`), and on Windows under Git Bash with
  `CARGO_TARGET_DIR` on D:. G0 runs from a worktree, `D:/rustyecon-wt/g0`, with targets in
  `/root/scratch/target-g0-*` and `D:/rustyecon-targets/g0-*` and logs in `D:/rustyecon-g0/`.
- On Windows, the same script under Git Bash with `CARGO_TARGET_DIR` outside the tree (it skips
  the wasm32 check, since the target is not installed there), then `rustyecon run tapes/gate.ron
  --until 2080 --hashes <file>` and the same for `tapes/appb.ron --until 20000` and
  `tapes/demo-gb.ron --until 7852`, and a byte comparison of each file's body (the `#` header
  names the build and target) with WSL's.
- The demo world ran from a worktree, `D:/rustyecon-wt/demo` (`/mnt/d/rustyecon-wt/demo` in
  WSL), on branch `demo-world`. Its targets are `/root/scratch/target-demo-<label>` and
  `D:/rustyecon-targets/demo-<label>`. Its scratch, map research, verification and gate logs
  are in `D:/rustyecon-demo/<label>/`, with the close's in `D:/rustyecon-demo/close/`.
  `rustyecon worldgen worlds/demo-gb --out tapes/demo-gb.ron` recompiles the tape, and
  `derive.py --check` in `worlds/demo-gb/` checks the derived tables. The atlas rebuilds with
  `data/atlas/build_atlas.py` under its pinned venv (`D:/rustyecon-demo/venv-atlas`).
- The screenshots: build `D:/rustyecon-demo/close/shot/` on Windows (`cargo build --release
  --offline`, target `D:/rustyecon-targets/demo-close-shot`), then run `demo-shot.exe OUT.png
  DATE LENS [SELECT|-] [HOVER|-] [W H PPP]`. For example, `demo-shot.exe map.png 1901-01-01
  since.output.per.head county.wry - 2000 1250 1.0`. It renders on WARP. The `adapters` binary
  beside it lists what wgpu offers.
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
- The `g0` merge ran from a worktree, `D:/rustyecon-wt/merge-g0` (`/mnt/d/rustyecon-wt/merge-g0`
  in WSL), on branch `merge-g0`, with both gates' targets `/root/scratch/target-merge-g0` and
  `D:/rustyecon-targets/merge-g0` (the engine gate's GUI check in `…-gui` beside them) and its
  logs in `D:/rustyecon-merge-g0/`.
- The `demo-world` merge ran from a worktree, `D:/rustyecon-wt/merge-demo`
  (`/mnt/d/rustyecon-wt/merge-demo` in WSL), on branch `merge-demo`, with both gates' targets
  `/root/scratch/target-merge-demo` and `D:/rustyecon-targets/merge-demo` (the engine gate's GUI
  check in `…-gui` beside them) and its logs in `D:/rustyecon-merge-demo/`.
- The `phase1` merge ran from a worktree, `D:/rustyecon-wt/merge-p1`
  (`/mnt/d/rustyecon-wt/merge-p1` in WSL), on branch `merge-p1`, with both gates' targets
  `/root/scratch/target-merge-p1` and `D:/rustyecon-targets/merge-p1` (the engine gate's GUI
  check in `…-gui` beside them) and its logs in `D:/rustyecon-merge-p1/`.
- The `oracle-goods` and `g1` merge ran from a worktree, `D:/rustyecon-wt/merge-og-g1`
  (`/mnt/d/rustyecon-wt/merge-og-g1` in WSL), on branch `merge-og-g1`, with both gates'
  targets `/root/scratch/target-merge-og-g1` and `D:/rustyecon-targets/merge-og-g1` (the engine
  gate's GUI check in `…-gui` beside them), its logs, the generators' and the cross-machine
  hashes in `D:/rustyecon-merge-og-g1/`, and the runners `run-wsl.sh` and `run-win.sh` there.
- G1 ran from a worktree, `D:/rustyecon-wt/g1` (`/mnt/d/rustyecon-wt/g1` in WSL), on branch
  `g1`, with targets `/root/scratch/target-g1` and `D:/rustyecon-targets/g1` (the engine gate's
  GUI check in `…-gui` beside them), and its gate logs in `D:/rustyecon-g1/gates/`. The ignored
  measurements run by name: `cargo test --release -p rustyecon-gui --test lab -- --ignored
  --nocapture`. G1's verification is in `D:/rustyecon-verify/g1-r1/`; G1.11's mutants run from
  `D:/rustyecon-g1/fix-r1/`: `mutate.py ROOT CARGO_WRAPPER LOGDIR [NAME…]` on a clone with the
  fixes committed locally (`mut/` in WSL through `c-wsl.sh`, `mutw/` on Windows through
  `c-win.sh` with laborformal's venv python and `BASH_EXE` set to Git's bash); it restores the
  clone with git after each.
- The goods chain's design evidence is outside the repository: `D:/rustyecon-goods/`
  (GOODS-CHAIN.md over its `oracle/`, `agents/`, `chain/` and `synthesis/` passes) and
  `D:/rustyecon-loops/` (LOOPS.md over `diminishing/`, `buffers/`, `planning/` and
  `synthesis/`, and `capacity/CAPACITY.md` over `capacity/model/` and `capacity/test/`). Each
  pass keeps its scripts and their outputs beside its report, and the mirrors' passes keep
  `SHA256SUMS` of the unedited files they copied.
- Unit 1g ran from a worktree, `D:/rustyecon-wt/og` (`/mnt/d/rustyecon-wt/og` in WSL), on
  branch `oracle-goods`, with targets `/root/scratch/target-og` and `D:/rustyecon-targets/og-*`,
  and scratch in `D:/rustyecon-og/` (the mutation runs in `mut/`, the fix round's in
  `mut/mutate_fix.py` on a copy in `/root/scratch/og-fixmut/`, the gates' logs in `gate/`, P1g.6's
  in `gate/fix/`, the error log of the golden comparisons in `errors-h.tsv`); the verification's
  derivation and mutants are in `D:/rustyecon-verify/1g-r1-derive/`. Its design sources are
  `D:/rustyecon-goods/` and `D:/rustyecon-loops/capacity/`, read-only.
- The markets probe ran from a worktree, `D:/rustyecon-wt/p2m` (`/mnt/d/rustyecon-wt/p2m` in
  WSL), on branch `phase2-markets`, with targets `/root/scratch/target-p2m-<label>` and
  `D:/rustyecon-targets/p2m-<label>`. Its frame, prediction, build checks, registration, runs,
  reviews and report scripts are in `D:/rustyecon-p2m/<label>/`. The binary `markets` (release,
  `-p rustyecon-probe`) lists, runs and kicks the batteries and families (`markets list`, `run`,
  `kick`, `family`, `point`, `elasticity`), and `D:/rustyecon-p2m/report/make_results.py`
  (WSL's python3) remakes docs/probe/results/markets/ from `D:/rustyecon-p2m/runs/`.
- The stocks probe ran from a worktree, `D:/rustyecon-wt/p2g` (`/mnt/d/rustyecon-wt/p2g` in
  WSL), on branch `phase2-goods`, with targets `/root/scratch/target-p2g-<label>` and
  `D:/rustyecon-targets/p2g-<label>`; WSL's disk had about 1.5 GB free, so the report's gates
  reused the build step's warm targets there. Its frame, build checks, registration, runs,
  reviews and report scripts are in `D:/rustyecon-p2g/<label>/`. The binary `horses` (release,
  `-p rustyecon-probe`) lists, runs and kicks the batteries and families (`horses list`, `run`,
  `family`, `kick`, `slowest`, `point`, `elasticity`, `openloop`), `horses-tape` writes a tape,
  and `D:/rustyecon-p2g/report/make_results.py` (WSL's python3, which has matplotlib) remakes
  docs/probe/results/horses/ and figs/horses/ from `D:/rustyecon-p2g/runs/` and the fidelity
  review's flow controls.
- The idle market's remedy (L0.3–L0.6) on branch `phase2-loops`. Any `horses` or `horses-tape`
  command takes `--reserve PSI`; without it the tape is P2.2a's. The run's waves are
  `D:/rustyecon-p2l/idle-engine/runs/wave*.sh`, with the binary copied to
  `/root/scratch/idle-bin/`, about 99 minutes on 48 cores. `analysis/analyze.py` (WSL python3,
  `JOBS=40`) scores them against P2.2a's runs and the scan's mirror into `runs/tables/`, and
  `analysis/plots.py` draws `runs/figs/`. The trace diff is `tracediff/tracediff.py BIN OUT`,
  after `make_i_carry.py`.
- The loop stage's groundwork (L0) ran from a worktree, `D:/rustyecon-wt/p2l`
  (`/mnt/d/rustyecon-wt/p2l` in WSL), on branch `phase2-loops`, with one shared target a
  machine, `/root/scratch/target-p2l` and `D:/rustyecon-targets/p2l`, and scratch in
  `D:/rustyecon-p2l/<label>/` (`m6`, `idle-scan`, `idle-engine`, `funded`, `loop-mirror`, the two
  reviews, `fix-report`). The loop mirror with the genesis carry, its checks and its battery are
  rerun as LOOP-SPEC-A1 §A1.7 says (about 70 minutes on 47 cores); the loop registration is
  rewritten by `D:/rustyecon-p2l/fix-report/make_registration.py WORKTREE`, which quotes and
  hashes the frame inputs in `docs/probe/loops/`.
- P2.2b runs from a worktree, `D:/rustyecon-wt/p2b` (`/mnt/d/rustyecon-wt/p2b` in WSL), on branch
  `phase2-plants`, with one shared target a machine, `/root/scratch/target-p2b` and
  `D:/rustyecon-targets/p2b`, and scratch in `D:/rustyecon-p2b/<label>/`. The frame's step
  (`rules`) keeps a read-only copy of the registered loop mirror in
  `D:/rustyecon-p2b/rules/mirror/`.
- The oracle's goldens: from `crates/oracle`, run `goldens/generate.py`, `generate_1b.py`, …,
  `generate_1f.py` and `generate_1g.py` with `--check` under laborformal's venv
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
