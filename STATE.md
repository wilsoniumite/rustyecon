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
**State as of:** 2026-09-28, on `reboot`'s line. **`oracle-goods` and `g1` are merged** into it
by this commit, on branch `merge-og-g1` from `reboot` at `16eb728`, not pushed: `oracle-goods`
came in by a fast-forward and `g1` by this merge; while `reboot` stays at `16eb728`, taking the
merge is a fast-forward, on your word. The two tracks both started at `16eb728` and numbered
apart, so nothing is renumbered: track 1g's decisions are 179–190 (its range 179–199) and its
open items O31–O35, and G1's decisions are 200–219 and its open items O36–O40. No code
conflicted; README.md and this file were joined by hand ("Where things stand").
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
Next, in order: your look at the windows (the G0 gate's item and G1's p90, both checked by
hand) and at the demo's map; G1's remainder (what G0 moved to it, O36; the five majors its
re-check left, O37; unit 1g in the lab, O38); your rulings on 1g's decisions and its re-check
(O35); the goods chain in the engine, the horse economy first, with a plant stock as its loop
damper (CAPACITY.md) and GOODS-CHAIN's staging, on design evidence kept outside the repository;
Phase 2 proper on loop-free wall and commons instances, after your rulings on the decisions that
bind it; the demo's second pass, with goods, machine types and carriers, on the many-market roles
(O27). "Next steps" has each.

## Where things stand

**`oracle-goods` and `g1` are merged** (2026-09-28, this commit, on branch `merge-og-g1` from
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
- **Decisions 59–75 and 135–178** (Phase 1), open to veto. Those that bind Phase 2 proper:
  from 1b and 1c, cells in the equilibrium (61) and machines built from categories (67), each a
  1c addendum if ruled in, and whether multiple equilibria are refused or all reported (70),
  which decides what Phase 2 compares the agents against and which 155 makes matter for the
  default exit form; from 1d, one shape with an efficiency per type (135), a solved wall as the
  wall instance (137), type hours compared through supply or in total (139), machine recipes on
  pool labour (140); from 1e, parcels as efficiency units (147), the participation rule under
  the default form (149), one exit good the tape names (151), idle land in wage units (153), no
  reserved tasks with the priced form (154), multiple equilibria where support is low (155), one
  land service (158), a free exit good at r = 0 (160), `Idle` plots (161); from 1f, the tax
  bases (162), the closure (164), supplement or replace (165), walled types and in-work
  benefits (167), `SurplusLabour` (168), one consumption rule for the oracle and the agents
  (169), one basket for every type (173). 172 binds Phase 3, and 177 Phases 6–8. The markets
  probe recommends keeping 67 for Phase 2 proper: its GO depends on it, and a veto would give
  every multi-category instance a goods-and-machines loop, the structure it found NO-GO, and so
  put those instances behind Phase 3. It confirmed 70 on every instance and leaves 61 to a
  probe of its own (MARKETS §6). GOODS-CHAIN, outside the repository ("Where things stand"),
  proposes rewording 67 so that machines are goods built from and run on goods, its mathematics
  kept (D-G1), and asks for a ruling on each of its D-G1 to D-G15.
- **Decisions 118–123** (the many-markets probe's), open to veto, with MARKETS-SPEC §9's frame
  decisions M1–M9. Of its questions, Q1 is answered by 120 and Q2 by the GO at 52 a year. Q3, a
  switch between machine types, waits for durable machines; Q4, category inputs, is untested.
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
- **Decisions 179–190** (unit 1g's, track 1g), open to veto. The ones that bind Phase 2: 179,
  decision 67 reworded (D-G1: machines are durable goods built from and run on goods, never
  from a category, E1), and 186, E2 enforced and G1 not approximated (hearth coal and carters'
  fodder wait for G1). 180 (D-G10) amends 71 and keeps every result of 1c bit for bit; 181, the
  mapping per period in the oracle, leaves where tapes are built to O32.
- **Decisions 200–219** (G1's, branch `g1`), open to veto. The ones that shape later work: the
  lab's form outside the model and the session (201), the fields read from the oracle's `Debug`
  rather than a `Point::outputs()` in the oracle (202), goldens paired by name (204), the
  breakpoints' semantics (209), and G0's leftovers deferred to a G1 second part (217).
- **Landing the branches.** `phase0-s2`, `phase1`'s P1.2–P1.7, the `g0` merge (`708167f`),
  `phase2-markets` (P2.1.1–P2.1.4, `b2a55e3`), the `demo-world` merge (`2398b6a`) and the
  `phase1` merge (`16eb728`) are in the local `reboot` by fast-forwards. `oracle-goods`
  (P1g.1–P1g.7) and `g1` (G1.1–G1.11) are merged by this commit, on `merge-og-g1`: while `reboot`
  stays at `16eb728`, it takes the merge by a fast-forward. Nothing is renumbered: the two
  tracks numbered apart (179–199 and O31–O35; 200–219 and O36–O40).
- **Pushing `reboot`** (locally at `16eb728`, with session 2, Phase 1, G0, the many-markets probe
  and the demo world; `origin/reboot` is at `cf3c0ff`), and this merge once it has landed.
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
  for loop-free economies, paths are now Phase 2 proper's first (O24).
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
and G1, the oracle lab, are merged by this commit, G1 but for its window check by hand. What
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
   built (G1.1–G1.10), verified once and fixed (G1.11), and merged by this commit, every gate
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
   1g (P1g.1–P1g.7, "Where things stand"), verified with one fix round (P1g.6), and merged by
   this commit. What is left: your rulings on decisions 179 (67 reworded, D-G1) and 180
   (D-G10), and on 181–190; the re-check of the fixed items (O35), before Phase 2 compares
   agents against the goods chain; the mapping's call where tapes are built (O32); and the rest
   of the addendum when its instances need it (O31). Machine recipes stay on pool labour
   (decision 140).
6. **The goods chain in the engine, the horse economy first, with a plant stock as its loop
   damper** (GOODS-CHAIN §3–§6, staged as its §5 and §6 set out; CAPACITY.md, in place of
   LOOPS.md's diminishing returns, which stay the registered alternative). First the mirror's
   own next step, in Python (GOODS-CHAIN §6 step 3): a material market, the fodder–horse-days
   loop with stocks, build lags and interest, with the plant on both sides of every loop at
   θ 0.8 and δ 10% a year under C2g, its predictions registered. Then, on the engine, the horse
   economy: GOODS-CHAIN's rule A (stage v2a.1), A0's economy with the horse held as a stock and
   rented out wet, fodder from land alone, so no loop, in a registered probe of machine stocks
   without loops (P2.2a, with S1 without its pumping loop). Its oracle is 1c with D-G10 and the
   mapping, now built: 1g's A0 at 52 ticks a year and CHAIN's horse at weekly periods are
   goldens, and the tape's goods need O32's schema and builder. Then the loops (P2.2b): rule
   B's horse, fodder raised with horse-days, with CAPACITY.md's plant on every loop desk, both
   sides: a plant good for each loop desk, held and never traded (D-G2's `Indefinite` good, worn
   by `Depreciation` burns), s1 bundles of the desk's own recipe per plant unit, ordered by
   M3's rule with s_K = 2δ, the form y = K^(1−θ)·z^θ, and a plan that reads only the desk's own
   coin, plant and recipe and the posted prices (R13); at θ = 1 it must be the flow run bit for
   bit. The oracle needs no new solve at ρ = 0, only closed-form readouts per plant desk (K\*,
   V and the user cost), which 1g's plants give (decision 184). Loops in Phase 2 need your
   ruling against decision 120, and machine stocks brought forward from Phase 3 your ruling on
   D-G11.
7. **Phase 2 proper** (PLAN Phase 2), after your rulings on the decisions that bind it: 61, 67
   and 70 from 1b and 1c, 118–123 from the markets probe, and 135, 137, 139, 140, 147, 149, 151,
   153–155, 158, 160–162, 164, 165, 167–169 and 173 from 1d–1f, and 179 and 186 from 1g for the
   goods chain's instances. The markets probe recommends keeping 67 and 70 (MARKETS §6). It opens on loop-free wall and commons instances, which need
   neither step 5 nor step 6, with the markets probe's roles, the `probe::markets` harness and
   C2m at 52 ticks a year (decisions 119–121): beside the Appendix B instance (1a), a
   wall-regime instance, a solved wall (1d; decision 137), and an open-commons instance under
   the default exit form, with the commons' shadow rent and idle land at zero rent (1e;
   decisions 149, 153, 160 and 161). The 1750-like instance follows (1e and 1f, with the common
   basket unless a basket per type is ruled in, decision 173), as the goods chain at ρ = 0 if
   D-G1 and D-G11 are ruled in. The agents' participation rule under the default form is
   decision 149's, and under a government 165's. Its battery runs O22's families first, stocks
   first. The commons' idle land needs an answer, without a clamp, for a market at zero rent,
   whose unsold price `Saturate` runs to the runaway bound. Many markets is no longer the first
   untested risk. The risks now are paths (O14, O24) and the new margins of 1d and 1e: several
   labour markets and the wall at x\* = 1. O30's precision sets the bands of any comparison
   against them.
8. **The demo's second pass, on the many-market roles** (O27; WORLD.md §7). It takes O26
   first, the clock held to 52 among it. Then come goods as unit 1b's categories and machine
   types as unit 1c's, on the many-market roles at C2m with no loop of produced inputs, with
   genesis from 1b and 1c at each county; GOODS-CHAIN §5 proposes these as a chain of goods
   instead, stage by stage from its rule A (D-G12). After that, the probe's battery county by
   county, the long run's dead ticks and shortfalls checked again, and lenses by category and
   type. It gets the same bounded verification as D.2 and D.3. Carriers come later, when
   transport desks, home-node trading (Phases 4 and 9) and an oracle with trade exist. They
   switch on from zero capacity, and the map then draws flows on channels.

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
                         the many-market roles in roles/many/ (P2.1.1)
crates/probe             the Phase 2 probe's harness and tape generator (P2.0.1); reads certify's
                         measures (S2.5); the markets probe's harness, probe::markets (P2.1.1)
docs/probe/RULES.md      the probe's rules, dials and lineage, as built
docs/probe/REPORT.md     the probe's report: verdict, battery, dial map, reviews, what it means
docs/probe/figs/         the report's plots; docs/probe/results/ its three summary tables (CSV)
docs/probe/MARKETS*.md   the markets probe's rules as built (MARKETS-RULES.md) and its report
                         (MARKETS.md), with figs/markets/ and results/markets/ (six CSVs)
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
tapes/markets-<id>.ron   the markets probe's seven worlds (I0–I3, L2, L3, G1), from the oracle
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
