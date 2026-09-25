# STATE — rustyecon (resume point for the next session)

**Project:** rustyecon v2, the reboot: 300 years of economic history, 1750–2050, as an agent
economy whose decisions are the pinning paper's margins, checked against an equilibrium oracle.
The plan is [docs/PLAN.md](docs/PLAN.md), amended by the rulings in
[docs/reboot/ADDENDUM.md](docs/reboot/ADDENDUM.md); the Phase 0 engine contract is
[docs/ENGINE.md](docs/ENGINE.md), and the tape's schema is [docs/TAPE.md](docs/TAPE.md).
**Collaboration:** as in laborformal. Sequencing, engineering and drafting are delegated to
Claude; checks gate absolutely; direct critique over validation. The numbered decisions below
are a veto window for your one-word calls.
**State as of:** 2026-09-25 (the session ran past midnight, so P0.7 and P0.8 carry
2026-09-26). Phase 0 session 1 is done: P0.1–P0.8 on branch `reboot-phase0`, not pushed. Next:
round 3's findings (O5–O13), then session 2, the certification stack.

## Where things stand

**Phase 0 session 1 is DONE and its gate is green in WSL and on Windows.** It salvaged
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
| P0.8 | this housekeeping: docs/PLAN.md moved and amended, this file, the CI skeleton, the docs checked against the code |

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
| resume, through the product path (N11) | `gate_resume_from_checkpoints` (engine, both formats), `gate_resume_through_product_path` (cli) |
| replay | `gate_replay_matches_every_tick` (engine), `replay_command_passes_on_the_gate` (cli) |
| conservation every tick (R2) | `gate_conserves_every_tick`, `gate_breach_stops_the_run`, `gate_rounding_is_declared` (engine) |
| a cash-short buyer settles both sides from one fill | `cash_short_buyer_settles_both_sides_from_one_fill` (markets) |
| two actor kinds sharing an id settle apart | `desk_and_pop_sharing_a_number_are_distinct_holders` (core), `two_kinds_same_number_settle_apart` (markets), `gate_ids_apart` (engine) |
| a burn shortfall stops the run | `burn_shortfall_stops_with_a_ledger_line` (core), `shortfall_stops_the_run` (cli) |
| unsorted events fire or fail | `unsorted_events_fire_in_order`, `every_zero_is_rejected` (core), `gate_events_fire_in_date_order` (engine) |

The full list, with what each test checks, is ENGINE §11: 175 `#[test]` functions and 7 doc
tests. `cargo test --workspace --release` passes 182 of 182 on both machines, with zero
warnings (built with `-D warnings`), `cargo clippy --workspace --all-targets -- -D warnings`
clean and `cargo fmt --all --check` clean:

| Crate | WSL | Windows |
|---|---|---|
| `rustyecon-core` | 85 unit + 2 doc | 85 unit + 2 doc |
| `rustyecon-markets` | 6 unit + 27 integration | 6 unit + 27 integration |
| `rustyecon-agents` | 1 unit + 8 integration | 1 unit + 8 integration |
| `rustyecon-engine` | 2 unit + 35 integration + 5 doc | 2 unit + 35 integration + 5 doc |
| `rustyecon-cli` | 11 integration | 11 integration |
| `rustyecon-certify`, `rustyecon-worldgen` | none yet | none yet |
| **Total** | **182** | **182** |

**Toolchain** (pinned by `rust-toolchain.toml`, installed by rustup on first use):

- WSL Ubuntu 22.04.4 (glibc 2.35): `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
  `1.97.1-x86_64-unknown-linux-gnu`; `cargo 1.97.1 (c980f4866 2026-06-30)`.
- Windows 11, MSVC: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, `1.97.1-x86_64-pc-windows-msvc`;
  `cargo 1.97.1 (c980f4866 2026-06-30)`.

**Hashes, Linux against Windows (recorded, not gated): identical.** All 2,080 per-tick hashes of
`rustyecon run tapes/gate.ron --until 2080 --hashes` agree byte for byte, and two runs on each
machine agree with each other. Tick 1 `0xf2371ea73f47ee1f`; 520 `0xb985a06b853fa899`; 1,040
`0x30f84912c6a744f9`; **2,080 `0x61f9c8529131ff17`**. At P0.3 the core fixture agreed too
(`world_id` `0x5482a99c926bdef7`, genesis state hash `0x7060047573ff37da`, and
`0x5121b67d1feee116` after 2,080 ticks of `num::exp`-driven price and EMA paths). The gate
world's final hash at P0.5 was `0x1b86507a195b2a40`; P0.6 changed what the state holds
(schedule params left it), and P0.7 changed no hash.

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

The first nine record how your rulings and the standing rules were carried out; the rest were
made while building.

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
   std hash containers (R8).
6. **No behavioural literal in shipped code.** A source scan allows only `0.0` and `1.0` and no
   named float constant outside `core::num` (O13 is a gap in it).
7. **Conservation is asserted.** Takes are all or nothing; a shortfall is a ledger line and stops
   the run; every tick's ledger and the run's ledger must close within the registered
   tolerances. Lots stay `f64`, and what a split or merge rounds away is declared with the
   reserved provenance `Rounding`, measured by TwoSum. Alternatives: integer quanta (gives up the
   range of tiny and huge prices), or exact takes (makes payments non-nominal).
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
    change keeps earlier checkpoints.
17. **Checkpoints.** Format 2, bincode or RON, with the state's FNV-1a hash as a digest: it
    catches corruption, not forgery (ENGINE §14 question 7; O7 is a hole in it). The alternatives
    were a keyed digest (needs a secret) or a replay from genesis on every resume.
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

## Open — your calls

- **The GUI** (asked for on 2026-09-25): live runs, graphs, region lenses, a Victoria 3-style
  map, and the editor built in. Its stack waits for your choice. `crates/engine` was built to
  serve it: a frontend depends on the engine alone, steps a `Sim` on a worker thread and reads
  each `TickReport` over a channel; the editor edits the tape and reruns or resumes (E1); the
  engine does no I/O and checks for wasm32, so it can run in-process, behind a local server or in
  a browser. ENGINE §13 lists `crates/gui` with no phase.
- **Pushing.** `reboot-phase0`, `reboot`'s addendum commit and `pre-reboot-2026-09-25` are local
  only. The repository is public (ruling 6), and CI waits on a push.
- **Hosted CI** at all (A5): the workflow is in place and inert.
- **The Phase 2 session budget** that A11's kill condition needs (PLAN Phase 2).
- **The decisions above**, especially 10 (the engine crate, not in PLAN's crate list), 11, 15
  and 17.

## Open — work

- **O1. The GUI**, once its stack is chosen (above).
- **O2. Phase 0 session 2: the certification stack** (A4; PLAN Phase 0 step 7), moved from
  `july-v2-phase-3` into `crates/certify`: certificate, criteria, verdicts, the NaN scan, the
  manifest (it records `world_id`, and the digest of any checkpoint a run resumed from),
  telemetry, `BalanceWatch`, and Parquet with `TickReport` as its input. With N4 (a run with no
  criteria must not certify PASS), N10 (`tape_sha` must cover every input), N12 (statistics
  must fail closed on NaN) and N15 (stability windows relative to the run) fixed, thresholds
  moved into criteria, and a price-runaway detector (A12).
- **O3. The oracle** (`crates/oracle`, Phase 1) is being built by another run in the main tree
  at `C:/Users/wilso/Documents/GitHub/rustyecon`. It joins through the members glob when merged,
  depends at most on `core`, and nothing on the engine path may depend on it (R13). Merge it and
  `reboot-phase0` into `reboot` together, and rerun the gate after.
- **O4. `test_01` is retired with the v1 agents** (A3), not ported. It failed at every commit
  where its tests compile (from `03eb06a`; the April commits do not compile theirs) and on all
  three July branches: `building_inventory_cycles_correctly`, "farm should produce wheat on tick
  0, got 0" (`tests/test_01_single_region.rs:172`, `:153` on v1). Nothing to do; logged here.

Round 3 of the adversarial review (after P0.7) left nine major issues open, and no blocker.
They are unfixed at `443d44c`. O5, O7, O8 and O9 are defects, confirmed by reading the code at
P0.8. O6 and O10 to O13 are test gaps, as the reviewers' mutation runs reported them; the code
behind O12 was read and is correct. ENGINE.md now says what the code does at each place it
overclaimed (its P0.8 amendment 2).

- **O5. Rounding across several lots is not declared exactly.** A burn declares `−q`, where `q`
  is the float fold of the lots taken (`core/src/apply.rs`, the `Burn` arm and `total`), and
  `Inventory::put` sums its merges' errors in `f64`. A burn of `All` from mill bread held as
  [1e17, 7] declares −1e17 while 1e17 + 7 are destroyed. It is below half an ulp of the larger
  operand, so no breach is possible, but R2's "every unit with provenance" is false there.
  Mutation T7 (only the last merge counts) survives every test. Fix: fold with `two_sum` and
  declare the error as `Rounding`; carry a compensated sum in `put`; test exactly with a
  perishable good in several lots.
- **O6. The ledger's flow term is unpinned.** Inflating `gross` (for example `+= q*q + q`)
  survives every test, so a regression could widen A12's registered tolerance silently. Fix:
  boundary tests with transfers and mints that assert `Breach.gross` and `Breach.tol` exactly,
  pass at 0.9 of the tolerance and breach at 1.1, for a tick and for the run.
- **O7. The checkpoint digest covers the state only**, not `world_id` or `prefix_id`
  (`core/src/checkpoint.rs`, `checked`). Editing `prefix_id` to the value a refused resume
  prints makes that checkpoint resume under another past, exit 0 (checked on WSL and Windows by
  the reviewers). Fix: digest (world_id, prefix_id, state), checkpoint format 3, a test per form
  in core, engine and cli.
- **O8. The run's ledger is not carried across a resume** (`engine/src/sim.rs`, `from_parts`
  opens a new one). A leak of 1e-9 coin a tick stops an uninterrupted run at tick 4 but passes
  all 2,080 ticks when resumed every 2 ticks. Fix: carry the run's ledger in the checkpoint
  (inside the digest, outside the state hash) and continue it, or state the limit and have O2's
  manifest re-audit from genesis; either way a test that resumes under a leak.
- **O9. Dated events in one tick fire by key, not by date** (`core/src/world.rs`, `once` sorted
  by (tick, event)). At 12 ticks a year a cut (`z.cut`) and a restore (`a.restore`) dated 20
  days apart share a tick, key order puts the cut last, and the mine stays cut for about 9.6
  years instead of 20 days; at 52 ticks a year the restore wins. Fix: keep each firing's day and
  sort by (tick, day, key), or refuse two same-tick firings that write the same target; update
  ENGINE §2.6, §9 and TAPE.md.
- **O10. A13's per-tick conversions in the scripted actor are untested across tick lengths.**
  Replacing `clock.flow`, `share` or the payout share with hard-wired weekly forms survives every
  test. Fix: run the gate world at 12 and 365 ticks a year and check annual posted quantities and
  the first tick's spend and payout against the registered values.
- **O11. The API guard tests miss five routes to a writer**: a `DerefMut` impl on `Sim`, a free
  `fn(&mut Sim)`, `pub use rustyecon_core::{self as internals}`, and `state_mut` or
  `holdings_mut` added to core's `Checkpoint` or `SimState`. None exists; the tests would not
  notice one. Fix: scan the whole frontend-visible surface including trait impls, treat `{self
  ...}` re-exports as whole-crate, and add compile-fail doc tests.
- **O12. A failed final checkpoint write is untested** (`cli/src/main.rs`, the `--out` save
  after the loop). The code propagates it correctly, but ignoring it passes every test. Fix: a
  cli test with `--out` naming a file and no `--checkpoint-every`, expecting exit 3.
- **O13. The literal scan misses integers made float** (`f64::from(2u8)`, `52 as f64`), so a
  hidden absolute dead band or fraction would pass it. None exists in shipped code. Fix: flag
  integer literals other than 0 and 1 that reach `f64`, with an allow-list for the calendar.

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

## Next session's first step

Fix round 3's findings, O5 to O13, on `reboot-phase0` as P0.9, each with a test checked against
the mutation that exposed it. Start with O7 and O8: both change what a checkpoint holds (format
3), and session 2's manifest records checkpoint digests, so they should land first. Then, on
your go, merge `reboot-phase0` and the oracle (O3) into `reboot`, rerun
`scripts/gate.sh` in WSL and the same commands on Windows, and start session 2 (O2).

## File map

```
STATE.md                 you are here; start here next session
README.md                what rustyecon is, the crates, how to build and test
docs/PLAN.md             the plan, amended by the addendum's rulings (2026-09-25)
docs/ENGINE.md           the Phase 0 engine contract, with each step's amendments (P0.3–P0.8)
docs/TAPE.md             the tape's schema guide
docs/reboot/             REVIEW.md and ADDENDUM.md, kept as written (links fixed)
docs/timeline/eras.md    era research for worldgen
crates/core              ids, clock, inventory, deltas, apply, ledgers, hash, checkpoints, tape
crates/markets           admission, clearing, settlement, prices
crates/agents            the behaviour seam and the scripted actor
crates/engine            Sim, the tick, reports, resume, the replay audit, the registry listing
crates/cli               the rustyecon binary: run, resume, replay, registry
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
