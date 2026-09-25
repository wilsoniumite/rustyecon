# Addendum — what the review missed

Dated 2026-09-25, the same day as [REVIEW.md](REVIEW.md) and [PLAN.md](PLAN.md). It
rewrites neither. It records what the review did not see, checks it, and proposes
amendments A1–A13 (§6) for you to accept or veto; each names the PLAN section it would
change. `v2p3:` paths are on branch `v2-phase-3` at `ff01284`, `main:` is `f614792`, `lf:`
is laborformal at `31b3482`; unmarked `src/` and `tests/` paths are `v2p3:`, and "July
PLAN" is `v2p3:docs/PLAN.md`. Everything was read or run today on copies, never in either
repository. Two adversarial checks were run against the draft; their corrections are in.

## Rulings (2026-09-25)

Made after reading this addendum. They amend PLAN.md where they conflict with it.

1. **A1, archive and push.** The July work is tagged `july-v2-phase-0` (`cf7e78f`),
   `july-v2-phase-1` (`bb57eea`) and `july-v2-phase-3` (`ff01284`), and the three tags and
   `pre-foundations` are pushed to `origin`.
2. **A5, WSL primary and Windows too.** WSL Ubuntu on this machine is the Linux machine;
   the gates run there. Windows is a secondary build and test check. Cross-platform hash
   equality is recorded, not gated. This replaces ruling 7.
3. **A8, both exit forms, s(q) the default.** s(q) = max(s₀ − q·h_e, s̲), with the idle
   margin and parcel quality from main.tex, is the default for the historical runs. The
   SSRN dependence form s = (e^χ − 1)P_s is the named alternative (R6). Unit 1a
   implements the dependence form, since the published Appendix B numbers use it; unit 1e
   adds s(q), and each form carries its own gate.
4. **A2–A4, A6, A7 and A9–A13 are accepted** as written in §6.

## The verdict

The review read `main` and missed three local branches: 36 commits made between
2026-07-19 and 2026-07-31, never pushed, present only on this machine. They fix all
seven July Phase 0 defects and fix or half-fix three of the review's five, and they
build most of the certification stack PLAN §3.8 lists as future work. They also build
and test the desk kernel the review calls untested. It lost every pre-registered A/B. On
five worlds with known equilibria it holds the fixed point exactly; displaced 2×, traded
volume decays in every cell under the shipped supply rule, and one later configuration
recovers volume but not price. The toolchain premise is wrong too: Windows has Rust
1.97.1, and July was built there. Separately, the posted SSRN version of the paper
prices exit as dependence, not as s(q), which PLAN §3.1–3.2 and unit 1e build on.

No ruling falls except ruling 7's stated premise. What changes is where Phase 0 salvages
from, what must happen before the first tag, and what Phase 2 starts from: a measured
failure with named causes, not an open question. The risk Phase 2 exists to test was
tested on July's rules; it held and did not return beyond about 5%, with one exception.

## 1. What the review missed

### 1.1 The July branches

| Ref | Tip | Date | Adds |
|---|---|---|---|
| `v2-phase-0` | `cf7e78f` | 2026-07-19 | The seven July Phase 0 defects fixed (1 commit after `main`) |
| `v2-phase-1` | `bb57eea` | 2026-07-20 | Golden-hash suite, conservation ledger with provenance, certificate, `criteria.ron`, Parquet (+14; 15 after `main`) |
| `v2-phase-3` | `ff01284` | 2026-07-31 | 225-year tracer, desk kernel, A/Bs, solvable worlds, restoring-force audit (+21; 36 after `main`) |
| tag `pre-foundations` | `afd29aa` | 2026-07-31 | Local tag inside `v2-phase-3` |

Each branch contains the one before. `git ls-remote origin` lists only `main` and
`reboot`, and no tags. No other clone holds these refs. `main..v2-phase-3` changes 369
files (+104,921/−1,042), 40 of them in `src/` (+7,991/−296).

`cf7e78f` is a behaviour change as well as a set of fixes: its loader fix activates the
dividend desk in every scenario for the first time. Every July measurement after
2026-07-19 is on a different engine from `main`'s v1.

### 1.2 The toolchain

Windows has cargo and rustc 1.97.1 (`stable-x86_64-pc-windows-msvc`), a July build at
`target/release/rustyecon.exe` (2026-07-31 07:51) and a repository `venv/` (2026-07-20);
`cf7e78f` names 1.97.1 as the dev toolchain. WSL Ubuntu 22.04 on the same machine has
1.95.0 through rustup. Its checkout, `/home/wilso/github/rustyecon`, is the one REVIEW:65
cites; it sits at `03eb06a` (2026-07-18) and lacks `f614792`, `reboot` and July; its
binary is dated 2026-05-01. `wsl -d ubuntu -- bash -lc 'false; echo $?'` prints 0; with
`--exec` in place of `--` it prints 1.

### 1.3 The laborformal checkout

The working tree is at `0291462` (2026-09-04), 31 commits behind the pin `31b3482`
(2026-09-25 14:10), with five untracked entries of yours. It has no `paths/` and no
`pinning/paper/ssrn-7226858.pdf`, the only executable source and the only text of
Phase 1's gate numbers (§5). `origin/main` has moved past the pin to `b6cfc06`. The
review's figures match `31b3482`, so the drift did not mislead it; it would mislead a
session that reads the checkout.

No single Python on this machine runs all of laborformal's gate checks. The venv is
Python 3.13.0 with SymPy but no SciPy, so `lf:paths/checks/check_macro.py` fails with
`ModuleNotFoundError`, although `lf:pinning/STATE.md:39-41` says SciPy was added. WSL's
python3 has SciPy 1.15.3 but no SymPy, pandas or pyarrow. The SymPy checks
(`check_pinning.py`, `check_three_taxes.py`, the corner checks) run in the venv, and
`check_pinning.py` needs `PYTHONIOENCODING=utf-8` when its output is piped.

### 1.4 Claims that do not hold

| Claim | Where | Finding |
|---|---|---|
| No code changed since July; all seven Phase 0 defects open | REVIEW.md:30-32, 83 | True of `main` only. `cf7e78f` fixed all seven on 2026-07-19. |
| No code was written against the July design | REVIEW.md:56 | False. July Phases 0–3 and most of 4 were built (`23f64fd`, `6d3a21b`, `9e96788`, `44cdc4f`, `ecb424f`..`097b73b`, `c4d9b7b`, `f52a887`, `47381ed`). |
| The desk kernel's stability has never been tested | REVIEW.md:197-199 | Partly false. The kernel was built (`f52a887`) and lost every pre-registered A/B (§3), but the tested arm departs from `kernel.md` and the Phase 5 phase diagram never ran. |
| Windows has no toolchain or venv; last built on Linux | REVIEW.md:63-66; PLAN ruling 7 | False (§1.2). |
| laborformal's venv is present and working | REVIEW.md:66-67 | Partly: no SciPy (§1.3). |
| Defect 8 is dormant | REVIEW.md:100-110 | False on `main` itself: the pop cash cap (`main:src/systems/transactions/mod.rs:71-84`) binds within 11,700 ticks in all 24 lr scenarios, earliest at tick 95 (`lr_07`; probe today). On the July engine `ced26f6` found 24 of 25 scenarios breaching conservation (earliest tick 838) from this cap and two other causes. |
| Certification, Parquet and the tracer are design to carry forward | REVIEW.md:163-166; PLAN §3.8 | Built on `v2-phase-3`, and their tests pass. The tracer runs but certifies FAIL today (B6 and B7; both regions DRIFTING); its July full-span PASS was retracted as a window artefact (`31f27e5`). |
| "the July tick-time rule" | PLAN.md:251-252 | Three v1 sections, all marked KEEP in July: `main:docs/systems/01_markets.md:345` (normalise rates by `tick_duration_days / 7`), `02_production.md:414`, `04_financial_markets.md:495`. No branch has a tick-length parameter. |
| The priced exit s(q) is an object of the SSRN paper | REVIEW.md:189, 258 | False for the SSRN version. s(q), the idle margin and parcel quality are in `lf:pinning/paper/main.tex` (:145, :370-385), the 2026-09-21 revision. The posted SSRN version prices exit as dependence (§5). |

`test_01` fails at every commit where its tests compile (from `03eb06a`; the April
commits do not compile theirs) and on all three July branches. PLAN's Phase 0 gate is
about the new workspace, and `test_01` leaves with the v1 agents (A3).

Minor: "~90% aspiration" is in `main:docs/PLAN.md:9`, not the README. `supply_chain`
and `big_region` (which also has a magic producer) do not load at `f614792`. The tag
`pre-cleanup-2026-09-04` exists nowhere, yet laborformal cites it
(`lf:README.md:35`, `lf:pinning/STATE.md:14`, `lf:dynamics/STATE.md:12`); the snapshot
is laborformal `585cdb9`, and the fix belongs there. PLAN Phase 10's "at most 0.15" is
the trend bound in `31b3482`'s message, which also gives 0.29 on the endpoint and "under
half" under one sensitivity.

## 2. The July engine, audited

### 2.1 What exists on `v2-phase-3`

`src/` is 11,003 lines in 52 files and `tests/` 5,292 in 15. New since `main`:
`certify/` (ledger, hash, certificate, criteria, verdict, detectors), `output/`
(Parquet, manifest, telemetry, checkpoint), the desk kernel (1,270 lines), the B8
reporter (`certify/technology.rs`, 1,568), provenance tags and equilibrium targets.
Around them: 12 examples, 44 scenarios (five solvable worlds, `solv_*`, and the `cr_*`
consistency corpus among them), 103 committed certificates, and A/B receipts. 81 of the
103 certificates carry a `tape_sha` that no longer matches today's tapes (edited at
`48c00bf` and `47381ed`); their state hashes still reproduce.

### 2.2 REVIEW §2.2 on `v2-phase-3`

| # | Status | Evidence |
|---|---|---|
| 1 RedistributePopPair `return` | fixed | `cf7e78f`; `src/state/apply.rs:130-131`. Untested; the pair is archived. |
| 2 Loader hardwires CapacityControl | fixed | `src/scenario/raw.rs:382-393, 542-550`. The loader is archived. |
| 3 HashSet delta order | fixed | BTreeSet, `transactions/mod.rs:388-392`; `tests/test_03_determinism.rs:43-51`. |
| 4 Spoilage mutates state | fixed | `SpoilInventory` delta (`types/delta.rs:60-62`, `apply.rs:52-59`), posted as a burn; `test_03:84-104`. |
| 5 Lot lives not serialized | fixed | `types/inventory.rs:119-152`; resume tests `test_03:53-82`, `test_08:206-245`. |
| 6 Printing in the hot loop | fixed | Only `runner.rs:493,503` print, on checkpoint failure. |
| 7 `is np.nan` fails open | fixed | Notebook fixed; the Rust scorer fails closed (`criteria.rs:95-103`). |
| 8 Asymmetric settlement | partly | Conserves since `ced26f6`/`02d8547`: sellers ship pro rata to what buyers took (`transactions/mod.rs:168-341`). Cash still binds at settlement (`:185-223`), not at the order (PLAN §3.3). No cash-short test. |
| 9 Shortfalls invisible | partly | A shortfall is a ledger line (`apply.rs:44-50`, `ledger.rs:176-193`); a burn shortfall never stops a run (N3). |
| 10 Id collision | open | `transactions/mod.rs:200-208` against `:211-219`; all settle as RecipeInstance at `:390-395`. |
| 11 Dead and overloaded state | open | `pop_agent.rs:50-73`, `building_agent.rs:247-252`; moot once archived, except forgiveness at `transactions/mod.rs:402` (N8). |
| 12 Magic producers | fixed as accounting | Minted and burned as `Provenance::Magic` (`transactions/mod.rs:226-259, 351-364`). Archived. |

### 2.3 Defects neither document lists

Numbered so Phase 0's tests can cite them. Probes ran in a scratch copy.

| # | Defect | Where |
|---|---|---|
| N1 | Events are deserialized unsorted, then binary-searched: with ticks 5, 2, 9, 9 a probe lost two events. `every = 0` never fires. Inherited from v1. | `src/scenario/mod.rs:14, 25-42` |
| N2 | The ledger skips an undefined good in release, behind a `debug_assert` only; 1e6 units of `GoodId(99)` ran on. | `ledger.rs:99-108, 160-165, 176-181` |
| N3 | A burn shortfall declares only what was removed, and `run_tick` discards the audit; a 1e9 shortfall ran on. Under `--certify` B5 then fails the certificate, but only at the end; plain runs and `run_tick` tests never see it. | `ledger.rs:185`; `systems/mod.rs:37` |
| N4 | A run with no `criteria.ron` certifies PASS: B8 "unscored" passes and a missing stability verdict counts as stable. Latent, since every shipped scenario has one. | `runner.rs:473-478`; `certificate.rs:78` |
| N5 | Absolute `1e-12` gates make a price floor near 1e-12/α (18 of 28 July scenarios sat on it; lr minima today 1.91–1.96e-11). Nothing bounds the top: July measured 1.147e139 on `multi_region` with B1–B5 clean (`engine.md:436-438`). Today legacy prices stay below 3e3, but no battery would catch a runaway, and the kernel+ratio receipts reach bands of 1e114–1e229. | `price_update/mod.rs:28,32` |
| N6 | Below price 1e-9 the buyer's cash cap is off (July's corpus prices reached 1.11e-11, today's about 1.9e-11); flows of 1e-12 or less are dropped. | `transactions/mod.rs:125, 190`; seven sites at `:357-433` |
| N7 | Cash binds at settlement, so clearing's volumes include demand that could not pay; the rationing goes unrecorded (R12). | `transactions/mod.rs:185-223`; `clearing/mod.rs:46` |
| N8 | Balance forgiveness `.min(0.0)` sits in settlement. | `transactions/mod.rs:396-406` |
| N9 | Lots never coalesce, so run cost grows with the horizon: `big_region` takes 10, 434, 3,900 and 30,630 ms for 1k, 11.7k, 40k and 117k ticks. Merging lots of equal life makes it linear (6, 47, 156, 455 ms) and takes `tracer_2r` from 198 to 48 ms. The tracer's checkpoints grow from 1.9 KB (tick 1,000) to 65.8 KB (11,500). | `types/inventory.rs:31-38`; `engine.md:413-429` |
| N10 | `tape_sha` skips unreadable files and omits `starting_state.bin`, which the loader prefers, `criteria.ron` and `equilibrium.ron`. | `certificate.rs:163-174`; `loader.rs:33-36` |
| N11 | No resume in the product: nothing outside the tests calls `checkpoint::load`, and a failed save is logged to stderr while the run goes on. `checkpoint.rs`'s own round-trip tests compare only `tick`; full-hash resume equality is tested in `test_03:53-82` and `test_08:205-245`. | `runner.rs:488-505`; `checkpoint.rs:91,101` |
| N12 | Statistics drop NaN samples one by one; `linear_slope` returns 0.0 when uncomputable; `BalanceWatch` skips non-finite values. | `verdict.rs:104-106, 324-338`; `invariants.rs:167` |
| N13 | `PriceRule` is a CLI switch with a code default, not a tape entry; `price_next_ratio` keeps the old price on a non-finite result. | `clearing/mod.rs:58, 124-132` |
| N14 | Core types need agents: `GameData` requires `KernelParams`; `SimState` embeds the v1 agent structs. | `game_data.rs:240`; `sim_state.rs:34-48` |
| N15 | Stability windows are absolute ticks. `criteria.ron` sets `analysis_end: 1000`, so an 11,700-tick certified run of `multi_region` is scored on ticks 150–1000 while B1–B5 cover the whole run. | `data/scenarios/*/criteria.ron` |

Once forgiveness and the 11 absolute-epsilon sites go (`transactions/mod.rs:125, 190,
357, 366, 375, 404, 416, 425, 433`; `price_update/mod.rs:28, 32`), the core and markets
salvage has three behavioural constants to register: `EMA_ALPHA`
(`price_update/mod.rs:6`), the genesis price level (`sim_state.rs:56,59`) and the
`PriceRule` choice.

### 2.4 Salvage map

| Crate | Take from `v2-phase-3` | Fix as it moves | Lines |
|---|---|---|---|
| `core` | `types/{ids,good,inventory,provenance,market_node,channel,delta}`; core arms of `state/{apply,sim_state,game_data}`; `certify/{ledger,hash}`; `output/checkpoint`; `scenario/mod.rs` as the tape's runtime form | OwnerId per actor kind (10); lot coalescing (N9); 7 of 26 delta variants plus an agent extension seam, an atomic Transfer, required provenance; genesis prices from the tape; N1–N3, N11, N14; tolerances registered | ≈1,650 |
| `markets` | `types/order`; `systems/{clearing,price_update}`; `systems/transactions`, about half rewritten | Budget and buyer class on orders; cash binds at the order (N7); rationing recorded; N5, N6, N8, N13; id collision | ≈550 |
| `cli` | The tick loop (`systems/mod.rs`); the runner's shadow-replay audit; `main.rs` exit codes | Return the audit; PLAN §3's phase order; drop the `--agents`, `--price-rule`, `--supply-rule` switches; add resume | ≈410 |
| `certify` | `output/parquet` as is; certificate, criteria, verdict, NaN scan, manifest, telemetry harness, `BalanceWatch` | N4, N10, N12, N15; thresholds into criteria; new metrics | ≈1,950 |
| `oracle` | `scenario/equilibrium.rs` as the target format | Its fields become the oracle's outputs | ≈200 |
| `agents` | `systems/production` (Leontief min-scale, shared-input reservation); `own_state_scan` as the R13 check | Retarget onto (A, Λ, B) desks, in Phase 2 | ≈170 |
| archive | kernel (as reference), v1 agents, `technology.rs` (a b = 0 cross-check for unit 1c), `metrics.rs`, `raw.rs`, `loader.rs`, v1 types, tests 01, 02, 05, 06, 08, 10–15, the lr and cr corpus | — | ≈6,400 |

The line counts are estimates and overlap where a file splits. About 45% of `src/`
carries, with about 700 test lines: the inventory, ledger, hash, clearing and checkpoint
unit tests, `test_03_determinism.rs` (repeat, resume and replay, the template for A3's
gate), `test_04:42-121`, `test_07:111-184` and `test_09`. The solvable worlds and the
designs of tests 12 and 15 carry as specifications (A9).

## 3. What July learned

The principal results were re-run today and reproduce. Where a re-run prints more
digits than July did, both are given.

### 3.1 Established

1. **The kernel was tested and lost.** The A/B rule was pre-registered (`d7b427b`)
   before the kernel landed (`f52a887`). All six lr comparisons are `LOSS`: live regions
   33 → 8, 5, 0 under the imbalance rule, 32 → 0, 2, 0 under ratio
   (`v2p3:results/ab/receipt-foundations.md:133-140`). On the cr corpus the kernel
   passes 10 of 33 regions to legacy's 0 and lifts live regions from 15 to 23, but kills
   4 that legacy keeps and widens the band (median ×13.9 where both are live, ×1.1e7
   corpus-wide): all three gates fail, `LOSS`. The arm departs from `kernel.md` (the pop
   desk's budget is inert, `src/kernel/mod.rs:578-583`), and the Phase 5 map never ran.
2. **It holds a solved fixed point exactly and rarely finds one.** All five solvable
   worlds hold at band 1.000×, 5/5 full PASS under both price rules; legacy gets 0/5
   (`receipt-foundations.md:95-104`). Four of the worlds register a 2× displacement of
   one genesis price (`solv_1g_money_2x` is a redenomination control). Under the shipped
   supply rule traded volume decays in all eight world-and-rule cells, and seven miss the
   ln(1.10) bar by 1.9× to 1.5e11× (July PLAN:684-700). In July's later matrix one
   configuration recovers volume: `solv_1g` under the imbalance rule and
   `reservation_goods`, volume trend 1.134×, but it misses on price (July
   PLAN:1547-1548).
3. **Under the kernel's shipped inelastic Rule 1, posted supply has own-price elasticity
   exactly 0** (`a9eae5e`; July PLAN:391-413), **so the price loop has a unit root**
   (P4.8, July PLAN:1366-1373, 1525-1528). Legacy flour measured ε_s = 7.03. Displaced
   +1%, a price stays bit-constant for 1,000 ticks on three tapes; displaced −1%, it
   drifts back through Rule 3's cash band, mostly by moving the numeraire wage (July
   PLAN:1366-1408).
4. **The dead band is the rest set.** At dead = 0.05 the edge is ×1.051282: `cr_00`
   passes at ×1.0512 and fails at ×1.0514 (`receipt-cr-vs-lr.md:108-131`); `solv_1g`
   freezes at ×1.0512 and moves at ×1.0513. A world 5% wrong for ever earns a full PASS.
5. **The reservation supply rule restores only locally** (July PLAN:1410-1507). The
   basin component around the fixed point runs from 0.9359× to 1.0457×; the set of
   displacements that contract is not an interval (11 islands above, 19 below). From
   ×1.05 to ×1.5 the price orbits 1.31–1.40× off target; at ×2 the market dies at tick
   292. Recovery stalls at 1.86e-11 on N5's guard, and a 2× redenomination moves the
   stall to 9.8936e-12: a nominal change moves a real outcome. With the reservation wage
   added, no lr region lives (July PLAN:1003-1026).
6. **Stagger contradicts non-storable inputs and channel operators.** With one desk per
   market and s > 1 no constant-price equilibrium exists
   (`docs/design/solvable-scenarios.md:159-182`). A pass-through desk needs b_out < s,
   so with b_out = 2 no s serves channels and labour together
   (`receipt-cr-vs-lr.md:133-189`).
7. **The instruments failed first.** Damping was inverted (`02b6432`); LogDrift missed a
   round trip (`c7f8a57`); the relative-only A/B gates scored a WIN between two dead arms
   (`receipt-foundations.md:177-236`); the best B8 median belongs to an all-dead cell. B8
   itself read `solv_labour`'s capacity rent as grain 2.000× too dear, and became a
   report with no bar (`ff01284`).

### 3.2 Retracted or narrowed by July itself

Do not cite: "the cash cap never binds" (`ced26f6`); the tracer's full-span PASS
(`31f27e5`); "5/72 in band" (all five dead, `484a3a0`); "agents must post schedules"
(`a9eae5e`); α·b_out = 0.2 as 10× inside the stability bound (measured gain up to 2.28);
the ε_s range 0.216–45.2; "no restoring force under inelastic" (upward only); "returns
to 1.000 exactly" (it stalls at 1.86e-11); "no configuration recovers from a 2×
displacement" (one does, July PLAN:1547-1548). `ff01284`'s commit message still states
"returns to 1.000 exactly"; P4.8 in the same commit (July PLAN:1300-1320) refutes it.
Read the PLAN, not the log.

### 3.3 What it means for Phase 2

Phase 2's battery ("from perturbed starts … reach each oracle equilibrium",
PLAN.md:400-401) is July's mode B. It was run on July's rules and, but for one
configuration, failed beyond about 5%. Tentative for the reboot: PLAN §3.2's rules are
all slow states, none a within-tick own-price posting response, so every market starts
with item 3's unit root. July found two classes with no source of supply elasticity:
desks with no purchased inputs (cost = 0, an enclosed parcel for instance) and
non-storable outputs such as services
(`docs/design/price-responsive-supply.md:173-178, 307-331`). In the reboot these are,
tentatively, parcel services and machine-hours. PLAN §3.1 also keeps stagger beside
non-storable inputs (item 6).

The tolerance cannot be tighter than about ±dead. July's last word made dead the Phase 5
axis, "rather than α·b_out" (July PLAN:1476-1479, 1563-1564); α as an axis was proposed
earlier (July PLAN:252-258). "Long-run averages converge" (PLAN.md:169-171) is the
statistic July found insufficient: `solv_labour`'s median gap is 1.006× while its band is
8.1× and its volume about a seventh of target (a re-run today: 8.09×, 0.136). With no
investment before Phase 3, a binding desk size earns a quasi-rent that a free-entry
oracle reads as error (tentative).

## 4. Builds and tests, as run today

| Tree | Platform | Toolchain | Build | `cargo test --release` |
|---|---|---|---|---|
| `v2-phase-3` `ff01284` | WSL Ubuntu 22.04, glibc 2.35 | 1.95.0 | OK, 17.6 s, 0 warnings | 180 of 181 |
| `v2-phase-3` `ff01284` | Windows 11, MSVC | 1.97.1 | OK, 25.8 s, 0 warnings | 180 of 181 |
| `reboot` `43dfba0` (v1 code) | WSL | 1.95.0 | OK, 9.1 s | 18 of 19 |
| `reboot` `43dfba0` | Windows | 1.97.1 | OK, 13.5 s | 18 of 19 |

`v2-phase-0` passes 19 of 20 and `v2-phase-1` 70 of 71. The one failure everywhere is
`building_inventory_cycles_correctly` ("farm should produce wheat on tick 0, got 0",
`tests/test_01_single_region.rs:172`, `:153` on v1). The source has 181 `#[test]`
functions. Every Linux result used 1.95.0; nothing has been built on WSL with 1.97.1.

**Cross-platform determinism.** Linux and Windows agree bit for bit on 218,976 per-tick
state hashes (44 scenarios, 1,000 ticks each and 11,700 for `tracer_2r`, in 4 of the
12 arm × price-rule × supply-rule configurations), 88 certificates, 24 telemetry and
manifest files, and six long runs of up to 46,800 ticks. Both reproduce the final state
hash recorded in all 103 committed certificates (`tracer_2r` at 11,700 ticks:
`0xd8e68411b286ea8f`). That is a side effect of the tapes, not a guarantee. The only
libm call on the state path, `.exp()` in `kernel::logit_shares`
(`src/kernel/mod.rs:440`), needs a multi-good need category, which no shipped tape has.
glibc 2.35 and the MSVC runtime differ by 1 ulp on about 0.5% of `exp` inputs and 0.004%
of `ln` inputs. With `solv_chain` patched to `food = {flour, wheat}`, the state hashes
split under some configurations (by tick 300 under kernel+ratio) and stayed identical
through 5,000 ticks under kernel+imbalance. PLAN §3.2's within-category logit puts `exp`
on the state path. Transcendentals also sit on the verdict path (`verdict.rs:212`,
`technology.rs:121, 817`), so a threshold verdict could split too.

**Performance** (Linux, five repetitions, wall time with process start; Windows adds
about 60 ms a run). `tracer_2r`, 11,700 ticks, 2 regions: 196–204 ms plain (about 58k
ticks/s), 312–328 ms with `--certify`, 485–510 ms with `--record` as well. `big_region`
grows faster than the horizon through N9 (§2.3). Kernel-arm timings are not throughput,
because its economies die: `tracer_2r` under the kernel finishes in about 25 ms on Linux
with both regions DEAD. Without lot growth a region-tick costs about 2 µs on 3–5-good
worlds, which does not threaten §3.9; lot growth does.

## 5. The oracle's source at `31b3482`

**Paper version.** The Phase 1 numbers are SSRN 7226858's Appendix B (2026-09-23), as
PLAN.md:165 already says. At the pin that version exists only as
`lf:pinning/paper/ssrn-7226858.pdf`, `lf:paths/code/macro.py` (`solve`, :98-114) and the
gate `lf:paths/checks/check_macro.py` P1 (:29-34, to 5e-6). `lf:pinning/paper/main.tex`
is the older 2026-09-21 revision, its port awaiting the author's export
(`lf:pinning/STATE.md:9-12`). Its Appendix B (main.tex:782) is another economy
(x\* 0.96791, v 0.59841, Y 18.47540, N_a 1.34285), the one `check_interior.py` checks; an
oracle built from it would fail the gate. The checks the SSRN text cites for its own
Appendix B, `check_interior.py` and `check_cost_system.py`, are stale or absent at the
pin (`lf:pinning/STATE.md:66-73`).

**Golden numbers.** Two 50-digit solves written independently from the SSRN equations
agree on every value below to the digits shown (residual 8e-51); `macro.py`'s f64 values
match them within 3.0e-16 relative.

| Quantity | PLAN / SSRN p.30 | Full precision |
|---|---|---|
| x\* | 0.86315 | 0.86315041816243703192 |
| v = w/r | 0.54344 | 0.54343596069677832842 |
| Y | 7.88061 | 7.8806055249729076768 |
| N_a | 1.34338 | 1.3433818800977173461 |
| final-task / machine hours | 1.07846 / 0.26492 | 1.0784575707193308057 / 0.2649243093783865404 |
| N·P_s | 5.44630 | 5.4463033271192370986 |
| Replacement closure | c = 1, w = 3; λ = 0: 0.4, 1.2 | exactly 1, 3; 2/5, 6/5 (`check_pinning.py`, 51 green) |
| (φ_w, φ_r) | (0.6, 0.4) | exactly (3/5, 2/5) (`check_three_taxes.py`, 35 green) |
| Income identity | 1e-10 | ≤ 2.7e-16 relative on 120 random economies |

**Where the source and the Phase 1 gate part.**

1. The replacement-closure instance is in `main.tex:325-336` and `check_pinning.py:59-73`,
   not in the SSRN version (which has an unscripted Figure 2 instead); it still holds
   under SSRN eq (5) with b̃ = b/(1−a) and λ̃ = λ/(1−a). A 4096-task LP built in scratch
   agrees with the SSRN Appendix B on Y within 2.7e-9.
2. The δ-scaled quantity side and the build lag J_b are not in the paper but in
   `macro.py` (:71-95) and `dynamics/`, so 1a's "durability and interest" needs the
   scalar user cost u = (ρ+δ)(1+ρ)^(J_b−1). laborformal asserts the income identity to
   1e-12 (`check_macro.py:59,129`), tighter than the PLAN's 1e-10.
3. **The exit form.** SSRN §4 eq (9) prices exit as dependence, s = (e^χ − 1)P_s, and
   keeps enclosure only as a historical example. s(q) = max(s₀ − q·h_e, s̲) survives in
   `main.tex:370`, `check_pinning.py` P3 and `corner/check_enclosure.py`. So do the idle
   margin and parcel quality (`main.tex:145, 375-385, 692`); main.tex's Proposition 3 is
   the endogenous outside option, SSRN's is dependence pricing. PLAN §3.1's parcels,
   §3.2's participation rule, §3.5's enclosure by price and unit 1e rest on main.tex's
   content.
4. laborformal has no equilibrium instance with a human-required set of positive
   measure and none with a parcel-quality schedule, so units 1d and 1e need constructed
   gates. There is symbolic material to build them on: `corner/check_kset.py` P9-i (a
   human-required set of measure k, with Leontief cost concentration) and
   `corner/check_enclosure.py` N-i to N-iii (the idle margin; worked instance
   q_enc = 1.5, N_crit = 60). Notation clashes: J is the capability integral and the
   build lag; ρ is interest and, in three-taxes, γ(x\*); main.tex's c is SSRN's p_m.

## 6. Proposed amendments

Each takes effect only if you accept it. A5 reopens ruling 7, whose premise is false;
no other ruling is touched.

**A1. Archive the July work first.** *(PLAN Phase 0 step 1.)* Before
`pre-reboot-2026-09-25`, tag `july-v2-phase-0` `cf7e78f`, `july-v2-phase-1` `bb57eea` and
`july-v2-phase-3` `ff01284`. They exist in one clone on one disk. Pushing them with
`pre-foundations` makes them public (ruling 6); a `git bundle` kept elsewhere is the
private alternative. Which, if either, is yours.

**A2. Phase 0 salvages from `v2-phase-3`, not v1.** *(PLAN Phase 0 step 4; honours
REVIEW ruling 4.)* The same substrate with defects 1–7 and 12 fixed and 8 and 9 half
fixed: §2.4's `core`, `markets` and `cli` rows, with defect 10, N1–N3, N5–N9, N11, N13
and N14 fixed as the code moves, and `test_03` carried as the hash-suite template. Lots
coalesce before the first golden hash, since that changes every hash and removes N9's
growth.

**A3. Restate the Phase 0 gate.** *(PLAN Phase 0 gate; STATE.md.)* `test_01` leaves with
the v1 agents, logged in STATE.md as failing at every commit where it compiles, beside
§1.4's corrections. On a scripted-desk world the gate asks for repeat, resume and replay
hash equality through a product resume path (N11), conservation every tick, and one test
per fixed defect (a cash-short buyer settles both sides from one fill; two actor kinds
sharing an id settle apart; a burn shortfall stops the run; unsorted events fire or
fail).

**A4. The certification stack moves in a second Phase 0 session.** *(PLAN §3.8,
Phase 3.)* It exists and its tests pass; it moves with N4, N10, N12 and N15 fixed, which
lowers Phase 3's estimate. The tracer certifies FAIL today (§1.4), so nothing certified
in July carries as a verdict. Phase 3's tracer gate names its window relative to the run
and rejects a frozen or ramping one (`31f27e5`).

**A5. Ruling 7 is yours to re-decide.** *(PLAN ruling 7; Phase 0 step 2; R8.)* Its
premise is false (§1.2), and its second sentence, that Windows "serves analysis only",
is a choice rather than a fact. Proposed: WSL Ubuntu on this machine is the Linux
machine, and the gates run there through `--exec`, in a fresh clone made after A1.
Windows becomes a secondary build and test check, since July and the analysis venv live
there; cross-platform hash equality is recorded, not gated. `rust-toolchain.toml` pins
one version for both. 1.97.1, the one `cf7e78f` named, has never been built on WSL,
which has 1.95.0, so the pin downloads it on first build. Transcendentals on the state
path and the verdict path go through one pure-Rust implementation (the `libm` crate, for
instance), or the within-category logit and threshold verdicts will split the platforms
(§4). Whether "CI" means local runs in WSL or hosted CI on the public repository
(ruling 6) is also yours.

**A6. The laborformal pin is read from the commit.** *(PLAN §8, Phase 1, R15.)* Use
`git show 31b3482:<path>` or `git archive 31b3482`; never check out, pull, reset or clean
the stale tree. A re-pin is a dated entry. Until one interpreter has both SciPy and
SymPy, `check_macro.py` runs under WSL python3 and the SymPy checks run under the venv
with `PYTHONIOENCODING=utf-8` (§1.3).

**A7. The Phase 1 gate names its executable source.** *(PLAN Phase 1.)* PLAN.md:165
already names the SSRN version; the addition is that its only executable form is
`macro.py` with check_macro P1, never main.tex's Appendix B. §5's full-precision values
gate at 1e-12 relative and the published ones at 5e-6; the income identity at 1e-12. The
scalar user cost joins 1a; per-type user costs and build recipes stay in 1c. The 1d and
1e gates are constructed (§5 item 4).

**A8. Rule on the exit form.** *(PLAN §3.1 Parcels, §3.2 Participation, §3.5 Enclosure;
Phase 1e.)* §5 item 3: s(q) with the idle margin and parcel quality from main.tex, the
SSRN dependence form, or both as named alternatives (R6). The agents' participation rule
and unit 1e wait on it; units 1a–1d do not.

**A9. Phase 2 opens with July's battery.** *(PLAN Phase 2.)* First the five solvable
worlds and their closed forms, re-expressed as tapes; test_12's mode A (hold) and mode B
(2× displacement, on the four worlds that register one); `test_15`'s sweep and basin
scan; and the elasticity probe, measuring ε_s per market before any battery.
Perturbations exceed ±dead and include cost shocks. The gate is joint: cleared volumes
against the oracle's quantities, contraction over the whole series, the band, and
relative prices against a numeraire whose market cleared. Every A/B gate gets an absolute
liveness floor (`receipt-foundations.md` §4). The phase diagram maps dead (July's axis)
and α (proposed here).

**A10. PLAN §3.2 records the tested failure.** *(PLAN §3.2 Scale; REVIEW §3.2(b).)* "The
July stability package survives" becomes "the package is the starting point, with a
measured failure". Phase 2 first re-tests July's price-responsive posting (built in
`47381ed`; basin about ±5%) with the changes July named, a rent-aware reservation price
(July PLAN:1061-1062, 1570) and a rule for one-sided markets
(`receipt-foundations.md:257-260`), plus two proposed here: the user cost in the
reservation price, and stagger on the scale decision only, with non-storables bought
every tick. Until Phase 3, desk capacity is fixed at the oracle's free-entry stock, or
the oracle takes installed capacity as state (tentative).

**A11. The kill condition is pre-stated.** *(PLAN Phase 2 kill condition; §7 risk 1.)*
"After honest work" is July's wording, and July's decision on it was posed and never
taken (July PLAN:246-266). Proposed: once A10's changes are measured on the solvable
family and the basin is mapped against dead and α, within a session budget you set, a
failing mode B triggers the fallback. Risk 1 reads "tested on July's rules: it held, and
did not return beyond about 5% but for one configuration", and the fallback is planned
as a real branch.

**A12. No absolute epsilons.** *(PLAN §4 R14; Phase 0 step 4.)* R14 covers engine
thresholds: no absolute guard on a price change, a currency flow or a quantity. Phase 0
removes the 11 sites behind N5 and N6 rather than registering them; `certify` gains a
price-runaway detector.

**A13. Tick time is new work.** *(PLAN §3.9; Phase 0 step 4.)* Phase 0 registers the tick
length and gives every dial a time unit, the EMA span in years first. The oracle solves
per period, so either the tick is its period or its inputs are restated at tick length:
ρ_tick = (1+ρ)^Δ − 1, δ_tick = 1 − (1−δ)^Δ, J_b in ticks.
