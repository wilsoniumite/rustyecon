# STATE — rustyecon (resume point for the next session)

**Project:** rustyecon v2, the reboot: 300 years of economic history, 1750–2050, as an agent
economy whose decisions are the pinning paper's margins, checked against an equilibrium oracle.
The plan is [docs/PLAN.md](docs/PLAN.md), amended by the rulings in
[docs/reboot/ADDENDUM.md](docs/reboot/ADDENDUM.md); the Phase 0 engine contract is
[docs/ENGINE.md](docs/ENGINE.md), the tape's schema is [docs/TAPE.md](docs/TAPE.md), and the
GUI's design is [docs/GUI.md](docs/GUI.md).
**Collaboration:** as in laborformal. Sequencing, engineering and drafting are delegated to
Claude; checks gate absolutely; direct critique over validation. The numbered decisions below
are a veto window for your one-word calls.
**State as of:** 2026-09-26. Phase 0 session 1 is closed: P0.1–P0.9 on branch
`reboot-phase0`, not pushed, with round 3's findings (O5–O13) fixed at P0.9. Oracle unit 1a
joined the workspace at P1.1 (O3), and the GUI's design, plan amendment A14 and R16 landed at
P0.11 (O1). Next, in order: session 2, the certification stack with the GUI's engine asks; then
G0, the GUI's shell; Phase 1's units 1b–1f alongside.

## Where things stand

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
| P0.11 | the GUI's design, docs/GUI.md, and its review ledger; plan amendment A14 (ADDENDUM §6, rulings 5–8), R16 and the G-stages in PLAN, ENGINE §13; decisions 22–34 below (O1) |

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
tests at P0.9. Since P1.1 core has one more (`fma_rounds_once`) and the oracle brings its 114.
`cargo test --workspace --release` passes 313 of 313 on both machines, with zero warnings
(built with `-D warnings`), `cargo clippy --workspace --all-targets -- -D warnings` clean and
`cargo fmt --all --check` clean:

| Crate | WSL | Windows |
|---|---|---|
| `rustyecon-core` | 90 unit + 2 doc | 90 unit + 2 doc |
| `rustyecon-markets` | 6 unit + 28 integration | 6 unit + 28 integration |
| `rustyecon-agents` | 1 unit + 9 integration | 1 unit + 9 integration |
| `rustyecon-engine` | 3 unit + 37 integration + 10 doc | 3 unit + 37 integration + 10 doc |
| `rustyecon-cli` | 13 integration | 13 integration |
| `rustyecon-oracle` (P1.1) | 42 unit + 71 gate + 1 doc | 42 unit + 71 gate + 1 doc |
| `rustyecon-certify`, `rustyecon-worldgen` | none yet | none yet |
| **Total** | **313** | **313** |

The oracle's 114 are its own gate (crates/oracle/README.md), and `goldens/generate.py --check`
passes under laborformal's venv.

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
WSL and on Windows. P0.11 changes no code.

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

**WASM (recorded, not gated):** `cargo check --target wasm32-unknown-unknown -p
rustyecon-engine` passes in WSL, so the engine can compile for a browser frontend (E2).

**CI:** `scripts/gate.sh` is the gate as one script (WSL or any Linux; build outside the tree).
`.github/workflows/ci.yml` runs it on GitHub's `ubuntu-latest`, but **it does not run until the
branch is pushed**, and whether hosted CI is wanted at all is your call (A5). Windows stays a
check run by hand.

**Remote:** `origin` has `main`, `reboot` at `43dfba0`, the three `july-v2-*` tags and
`pre-foundations` (A1 done). Not pushed: `reboot` at `87d95d7` (the addendum),
`pre-reboot-2026-09-25`, and `reboot-phase0`.

## Decisions — veto window (your one-word calls)

The first nine record how your rulings and the standing rules were carried out; 10–21 were made
while building. 22–34 are the GUI's D1–D13 (A14; [docs/GUI.md](docs/GUI.md) says where each is
carried out): each stands unless vetoed before G0, and D10's window closes at session 2.

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

## Open — your calls

- **The GUI's decisions**, 22–34 (D1–D13): open to veto before G0, D10's before session 2.
- **Pushing.** `reboot-phase0`, `reboot`'s addendum commit and `pre-reboot-2026-09-25` are local
  only. The repository is public (ruling 6), and CI waits on a push.
- **Hosted CI** at all (A5): the workflow is in place and inert.
- **The Phase 2 session budget** that A11's kill condition needs (PLAN Phase 2).
- **The decisions above**, especially 10 (the engine crate, not in PLAN's crate list), 11, 15
  and 17, and the GUI's 22–34.

## Open — work

- **O1. The GUI.** Designed ([docs/GUI.md](docs/GUI.md); A14), with egui in `crates/gui`. G0,
  the shell, follows session 2 (two to three sessions; GUI.md §9): G0.1 the viewer (the crate's
  seams, the Runner and `ThreadDriver`, the toolbar, timeline, outliner, plots, inspector,
  registry and log), then G0.2 the editor (`materialise`, lineage, branches, compare and
  export). G1, the oracle lab, starts after G0 and Phase 1's gate. `crates/engine` was built for
  it: a frontend depends on the engine alone, steps a `Sim` on a worker thread and reads each
  `TickReport` over a channel.
- **O2. Phase 0 session 2: the certification stack** (A4; PLAN Phase 0 step 7), moved from
  `july-v2-phase-3` into `crates/certify`: certificate, criteria, verdicts, the NaN scan, the
  manifest (it records `world_id`, and the digest of any checkpoint a run resumed from),
  telemetry, `BalanceWatch`, and Parquet with `TickReport` as its input. With N4 (a run with no
  criteria must not certify PASS), N10 (`tape_sha` must cover every input), N12 (statistics
  must fail closed on NaN) and N15 (stability windows relative to the run) fixed, thresholds
  moved into criteria, and a price-runaway detector (A12). With it, the GUI's engine asks (D10;
  docs/GUI.md §7.2, each with its ENGINE amendment and test; ENGINE §13), as they stand at P0.9:
  - item 1, `world_id` without the `fixed` flag the schedule sets, is partly met: since P0.6 a
    param only the schedule reads lives outside `world_id`, but a registered param that becomes
    a `SetParam` source still changes it, since `world_id` hashes each param's `fixed` (its
    test: `new_source_event_keeps_world_id`). It lands before the manifest records a `world_id`;
  - item 2, `FiredEvent` naming its source, is unmet (its test: `fired_event_names_its_source`);
  - item 3, chunked stepping equal to `run_until`, is met by `observing_changes_no_hash` and
    `gate_resume_from_checkpoints`;
  - item 4, `engine::registry` listing each use of a param with its `ClockMethod`, is partly met:
    it lists one per-tick method per param, guessed from the unit and the price-rate use (its
    test: `registry_names_each_use`).

  Raised with it, not asked: the manifest's tape hash (proposed: `fnv1a_64` over the canonical
  `to_ron`), the cli's `resume` verifying a state hash and its hash output naming its run (R16),
  and a Parquet-free manifest module for the web build.
- **O3. The oracle, unit 1a, landed at P1.1.** Built by another run and verified there (114
  tests), it joined through the members glob. It depends on `core` alone, for `num`, and nothing
  on the engine path depends on it (R13). The workspace's `clippy.toml` denies the platform
  maths, so x^k, ln(1 + z) and its fused multiply-add now go through `core::num` (libm), which
  gained `fma`. Its outputs are then byte-identical on WSL and Windows (5000 random economies,
  every regime), no golden moved, and G8's exact tie still ties. Units 1b–1f follow (PLAN
  Phase 1), alongside session 2 and G0.
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

1. On your go, merge `reboot-phase0` (which carries the oracle since P1.1 and the GUI's design
   since P0.11) into `reboot`, and rerun `scripts/gate.sh` in WSL and the same commands on
   Windows.
2. **Phase 0 session 2** (O2): the certification stack with N4, N10, N12 and N15 fixed, and the
   GUI's engine asks, D10's items 1, 2 and 4 as docs/GUI.md §7.2 lists them (item 3 is met).
   Item 1 lands before the manifest records a `world_id`. The manifest records the digest of any
   checkpoint a run resumed from; since P0.9 that digest covers the checkpoint's identity and the
   run's ledger too (format 3).
3. **G0.1, the viewer, then G0.2, the editor** (O1; docs/GUI.md §9).
4. **Alongside, Phase 1's units 1b–1f** (O3; PLAN Phase 1), each with its gate: many categories
   and the fork, many machine types, worker types and the wall, parcels and s(q), households and
   government.

## File map

```
STATE.md                 you are here; start here next session
README.md                what rustyecon is, the crates, how to build and test
docs/PLAN.md             the plan, amended by the addendum's rulings (2026-09-25)
docs/ENGINE.md           the Phase 0 engine contract, with each step's amendments (P0.3–P0.11)
docs/TAPE.md             the tape's schema guide
docs/GUI.md              the GUI's design (A14): stack, architecture, panels, editor, map, roadmap
docs/reboot/             REVIEW.md and ADDENDUM.md, kept as written (links fixed) but for A14 and
                         rulings 5–8 (P0.11); GUI-review-ledger.md, the GUI design's two reviews
docs/timeline/eras.md    era research for worldgen
crates/core              ids, clock, inventory, deltas, apply, ledgers, hash, checkpoints, tape
crates/markets           admission, clearing, settlement, prices
crates/agents            the behaviour seam and the scripted actor
crates/engine            Sim, the tick, reports, resume, the replay audit, the registry listing
crates/cli               the rustyecon binary: run, resume, replay, registry
crates/oracle            the equilibrium solver, unit 1a (P1.1); its README and docs/unit-1a.md
crates/certify           empty until session 2
crates/worldgen          empty until Phase 4
tapes/gate.ron           the gate world
scripts/gate.sh          the gate as one script
.github/workflows/ci.yml hosted CI (runs only once pushed)
```

## Repro notes

- The gate in WSL, from a Windows shell:
  `wsl -d ubuntu --exec bash -lc '<repo>/scripts/gate.sh'`. Always `--exec`: with `--` the exit
  code is lost. The script puts the build in `$HOME/scratch/target-rustyecon-gate` unless
  `CARGO_TARGET_DIR` says otherwise, and refuses a target directory inside the tree. With no
  network and a warm cache, set `CARGO_NET_OFFLINE=true`.
- On Windows, the same commands with `CARGO_TARGET_DIR` outside the tree: `cargo fmt --all
  --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace
  --release`, then `rustyecon run tapes/gate.ron --until 2080 --hashes <file>` and a byte
  comparison with the WSL file.
- A log captured by redirecting `wsl.exe`'s output to a Windows file can interleave and lose
  lines; redirect inside the WSL command instead.
- The July engine is read with `git show july-v2-phase-3:<path>`; never check the tag out
  into this tree.
- laborformal is pinned at `31b3482` and read from that commit (`git show 31b3482:<path>` or
  `git archive`), never from its stale checkout (A6). Until one interpreter has both SciPy and
  SymPy, `paths/checks/check_macro.py` runs under WSL's python3 and the SymPy checks under the
  venv with `PYTHONIOENCODING=utf-8`.
