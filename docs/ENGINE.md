# ENGINE — the Phase 0 engine contract (session 1)

Final, 2026-09-25, committed as P0.2. It folds in the frontend requirement of 2026-09-25 (an
engine crate, a read-only observation API, no frontend mutation, `Send` types, a stable tape
schema) and an adversarial review of the draft. Steps follow it in order, one commit each:
**P0.3 core**, **P0.4 markets**, **P0.5 agents + engine + cli**, **P0.6 fixes from adversarial
review, round 1**, **P0.7 fixes from adversarial review, round 2**, **P0.8 housekeeping** and
**P0.9 fixes from adversarial review, round 3** (P0.1 is the skeleton; P0.2 is this file and the
empty `crates/engine`). The draft had agents and engine + cli as two steps; they were merged at
P0.5 (amendment 1 below). Housekeeping was P0.6 until the reviews' fixes took that number and the
next (P0.6 amendment 8, P0.7 amendment 7).

It covers PLAN §3 (phase order), §3.1, §3.3, §3.8 and §3.9; R2–R4, R8 and R12–R14; and ADDENDUM
A2, A3, A5, A12 and A13. Salvage comes from tag `july-v2-phase-3` (`ff01284`); `v2p3:` paths name
that tree. Defects 1–12 are REVIEW §2.2's, N1–N15 ADDENDUM §2.3's, F8 is July's finding on
one-sided markets. Certify is session 2 (A4), except the ledger and the state hash, which are core.

A step is done when `cargo test --workspace --release` is green and `cargo clippy --workspace
--all-targets` and `cargo fmt --check` are clean, with zero warnings, on WSL (primary) and on
Windows (secondary). Commit as `P0.n: <what>`.

**Amended at P0.3** (core), where the text below was wrong or silent; each change is made in
place in the section named.

1. `NoExt`'s `RawActor` and `Actor` are `()`, not `Never`, so a tape with no behaviour can still
   declare actors that hold goods: core's tests and markets' (which sits below agents) need
   holders (§2.3).
2. `Ext` has the supertraits `Clone + Debug + PartialEq` (its implementors are unit markers, and
   the std derives on `StateDelta<E>`, `SimState<E>` and `Tape<E>` need them) and three more
   required methods: `validate` (a resumed checkpoint's extension state is checked, §7.6), and
   `canonical_actor` and `canonical_action` (`to_ron` writes the extension's own lists in
   canonical order too) (§2.3).
3. The `num` helpers return the exact largest fitting value by a search over bit patterns, not
   by at most 4 `next_down` steps. The stepping rule could not meet §11's own test (the result
   fits and its `next_up` does not) without stepping up as well, and a small remainder such as
   `max_remainder(1, 0.999)` lies hundreds of ulps of the difference from its answer, so a
   4-step cap would fail ordinary payouts. `NumError` is `Invalid` or `Exceeded`; there is no
   `Precision`. A zero bound gives exactly 0 and an infinite quotient gives `+∞` (§2.5).
4. `Inventory::take` returns `Result<Vec<Lot>, TakeError>`, since an invalid request (NaN,
   negative, `-0.0`) must be refused and is not a shortfall (§2.2).
5. `SimState::params(&registry)` takes the registry, which lives in the `World`; book readers
   return `Option<f64>` instead of panicking on an id outside the book (§2.3).
6. `Firing` and `Recurring` carry `source: Option<ParamId>`, the param a `SetParam` copied its
   value from: the key "that stays on the firing" (§2.6). A `SetParam`'s source is a fixed use,
   since its value becomes structure at load (§2.1).
7. Errors from the RON parser itself (syntax, an unknown or missing field, a malformed key or
   date) carry the parser's line and column, not a tape path: serde gives no path without a
   crate outside §1's list. Every resolution error names its path. The schema number is probed
   first, so a newer tape is refused as a schema error even if it has fields this loader does not
   know (§2.6).
8. `apply` also refuses `Spoilage` on a mint or burn (it is ageing's), a book write to a currency
   slot (`NoMarket`) and a `SetParam` on a fixed param (`FixedParam`, behind the load check)
   (§2.4). The loader also refuses `ticks_per_year: 0`, a channel from a node to itself, a
   genesis price on a currency, a recurring `last` before its `first`, and duplicate genesis
   holdings (§2.6).
9. Core's tests are unit tests inside `crates/core/src`, each module's after its first
   `#[cfg(test)]`, on the fixture `crates/core/testdata/core.ron`. Source comments cite sections
   such as §2.4, so the source scans of §11 strip comments before looking for literals.

**Amended at P0.4** (markets), the same way. Core did not change.

1. `clear`, `settle` and `SettlePlan::realize` return `Result<_, OrderError>`: a line or fill for
   a market the world does not have is `UnknownMarket`, and a moved list whose length is not the
   plan's is `Moved`, never a panic or a silent skip (N2). `OrderError` also has `Core` (a state
   that does not fit its world) and `Num`; `PriceError` also has `NonFiniteEma` and `Core` (a
   param read that fails) (§3.1–§3.3).
2. `Line` is `#[non_exhaustive]`: its fields are public to read, but only `admit` makes one, so a
   budget binds at admission and nowhere else (§3.1).
3. Admission runs the takes settlement will make, in the same order, on a copy of each actor's
   holding. For a currency, which is one lot, that is exactly `rem = rem − budget`. For a
   perishable good held in several lots it is not the same as subtracting from the lot sum: with
   lots of 0.1 and 0.2, a sell of 0.1 leaves 0.2, but `0.30000000000000004 − 0.1 =
   0.20000000000000004`, so a second sell of that much would pass the subtraction and fall short
   at settlement (§3.1).
4. Duplicate keys are checked before zero-quantity orders are dropped, so a zero-quantity second
   post is still `Duplicate`, and a dropped order binds no cash. Every check runs in canonical
   order, so which error is reported does not depend on input order either (§3.1).
5. A market trades when `S > 0`, `D > 0` and both fills are positive (`MarketFill::trades`). With
   both sides posted, a fill is 0 only when `S/D` or `D/S` underflows below 2⁻¹⁰⁷⁴; nothing then
   trades, rather than one side alone (sellers shipping what the last buyer would take unpaid,
   say). A transfer whose
   nominal quantity is exactly 0 is left out of the plan, except each escrow's last taker, whose
   `All` is always planned (§3.2).
6. The no-shortfall argument of §3.2 assumes normal numbers. Where a payment or a fill falls into
   the subnormal range its rounding is no longer relative, and a non-last take could exceed its
   escrow by a subnormal amount. That would be a `CoreError::Shortfall`, which stops the run with
   its ledger line (R2); no guard is added (A12) (§3.2). The same rounding can also settle one
   side alone, with no error (found at P0.6): with a subnormal `seller_fill` every shipment can
   round to 0 while the buyers pay a subnormal amount (4.94e-24 coin for no bread, at a price of
   1e300), and a last buyer can receive up to a fifth more than its feasible quantity. Both sides
   still conserve; the run records it and goes on. Session 2's certificate can flag a market
   whose fills are subnormal.
7. The pure rules are public: `imbalance`, `step` (`p·exp(k·x)`) and `next_price(rule,
   one_sided, p, k, S, D)`. Under `Saturate` a one-sided market's `x` is the imbalance itself,
   which is exactly ±1. `Ratio` computes `p·(D/S)`, July's order of operations (§3.3).
8. Markets' tests are integration tests in `crates/markets/tests/` (`admission`, `settlement`,
   `prices`, `time`) on the fixture `crates/markets/testdata/markets.ron`, except July's five
   clearing tests, which stay unit tests of `prices` (§11).

**Amended at P0.5** (agents, engine and cli), the same way. Core and markets did not change.

1. One commit, P0.5, delivers the agents, the engine, the cli, `tapes/gate.ron` and all their
   tests; the steps table merges the draft's P0.5 and P0.6, and housekeeping is P0.6 (§12).
2. Hooks return `Result<_, AgentError>`: a param read or a `num` helper can refuse, and nothing
   panics. `AgentError` is `Core`, `Num`, `Take` or `Mismatch` (an actor whose state is not of its
   spec's kind); the engine reports it as `RunErrorKind::Agent { actor, hook, error }` (§4, §7.5).
3. `View` also carries `currency`, the home node's currency, since a node's currency is not
   otherwise visible to an actor. `Hook` lives in agents, next to `Behaviour`. `Cast::new(&World)`
   returns `Result<Cast, LoadError>` and makes the two checks that need the whole world: a
   scripted actor budgets in its home currency, so every buy line's node must quote in it, and a
   payout never names the payer (§4).
4. The scripted spec (§4, §5): `spend` is `Option<Key>`, `Some` exactly when there are buy lines,
   so an actor that buys nothing registers no spending dial. A sell line's `qty` is
   `Flow(param)` or `AllHeld`; lines post in (node, good) order, each at most what a copy of the
   holding still holds after the lines before it, taken as admission takes it, so a `Flow` sell
   never over-posts and `AllHeld` means what is left at its turn. Recipe coefficients and all
   weights are finite and positive. The tape action is `Actor(SetActive(actor, active))`.
5. The accessors `price`, `ema`, `supply`, `demand` and `param` return `Option<f64>`, `None`
   outside the book or the registry, like core's readers (P0.3 amendment 5). `Trace` is
   `Trace(pub Vec<TraceEntry>)` with `TraceEntry { phase, delta, moved }`. `Sim` is `Clone`. The
   registry listing is the engine's `registry(&Tape) -> Result<Vec<RegistryLine>, LoadError>`,
   which the cli prints (§7.1).
6. `ReplayError::Shadow { tick, phase, error }` is a traced delta the shadow cannot apply, a
   divergence like `Mismatch` (exit 4). After a failed step `hash()` is of the state as the
   failure left it (§7.5).
7. Hook outputs are collected in any visiting order and then sorted by `ActorId`. A payout split
   runs in `ActorId` order (the last recipient takes the remainder), so renaming keys moves its
   last-bit rounding; the renaming half of `decisions_read_phase_start_state` compares tick-0
   orders exactly and payout recipients as a set (§11, §14).
8. The cli (§8): a `--hashes` line is `{t} 0x{hash:016x}`, where `t` is the state's tick after
   each step, so a run to `T` writes ticks 1 to `T` and a resume from `c` writes `c+1` to `T`;
   stdout's last line has the same form. Checkpoints are written only for states a step
   reached, never the starting state, so a resume cannot overwrite its own input.
   `--checkpoint-every` needs `--out` and N ≥ 1. The output directory is made at the first
   write, so an `--out` that names a file fails there (exit 3). An unreadable tape is a load
   error (exit 1); an unreadable or undecodable checkpoint is exit 3; a resume whose tape does
   not load is exit 1; `--until` before the starting tick is exit 1; clap's own argument errors
   become exit 1 (clap's default, 2, is the run-error code here). `replay` exits 2 when the live
   run fails and 4 on a mismatch or a shadow failure.
9. The gate world (§10). As first written, with the mill buying grain only in the village and fuel
   only in town and selling bread only in town, three of the six markets would never trade, which
   fails §10's own bar. So every desk trades at both nodes: the farm and the mine sell a
   registered flow in town and the rest in the village; the mill buys grain and fuel at both
   nodes and sells a flow of bread in town and the rest in the village; the dormant oven buys in
   town and sells in the village. The pensioners buy bread at both nodes, so each bread market
   always has a cash-short buyer; the workers buy in town. The mill's fuel stock carries it for
   about 16 ticks after the 1760 cut, and then the village's bread rations hard.
10. Tests (§11): the engine's are integration tests in `crates/engine/tests/` (`determinism`,
   `gate`, `edits`, `frontend`, `time`, `scans`), except the two hooks tests, which need the
   crate's internals and are unit tests in `tick.rs`; the agents' are in
   `crates/agents/tests/seam.rs` and the cli's in `crates/cli/tests/cli.rs`. Added:
   `engine_runs_on_a_worker_thread`, `no_public_api_hands_out_mut_state`,
   `gate_world_meets_its_bar`, `the_scanner_reads_what_it_should`,
   `scripted_specs_are_checked_at_load`, `specs_round_trip_in_canonical_form`,
   `agents_types_cross_threads`, `registry_lists_every_number` and `no_behaviour_switches`. The
   scan for E2 also bans `std::io`, which nothing on the engine path uses.
   `gate_rations_and_records_by_class` reads "rations again after the cut" as: over both bread
   markets, the year after the cut has more than twice the bread shortfall of the year before and
   less than half its worst fill. `gate_ids_apart` compares each holding's change with the signed
   sum of the traced quantities under a relative bar of 1e-12, named in its file (the two round
   differently), and matches every settlement transfer to its settle line exactly.
11. Dependencies (§1): agents and engine use `serde` (specs and reports are serialisable).
   `clap` and `tempfile` join the workspace dependencies, and `Cargo.lock` pins clap 4.6.1 and
   tempfile 3.27.0 (July's versions) with a dependency closure both machines have cached.

**Amended at P0.6** (fixes from adversarial review, round 1), the same way.

1. Checkpoints (R2, E1, N11). Format 2 stores the state's hash as a digest in both forms, and
   the decoders refuse a state that does not hash to it (`CheckpointError::Digest`), before any
   resume sees it: under format 1 a RON checkpoint with a million coin added resumed, and the
   coin appeared in no ledger line. `Checkpoint`'s fields are private. It is made only by
   `Checkpoint::of` and the decoders and read through `world_id()`, `prefix_id()`, `state()` and
   `digest()`, so nothing reaches `&mut` its state (a doc test pins that `&mut cp.state` does not
   compile). The in-memory `format` field is gone, since a value in memory is always of this
   build's format, and with it `ResumeError::Checkpoint` and the resume's unreachable format
   check. §7.6 states the trust boundary that remains (§2.5, §7.5, §7.6).
2. Schedule params (E1). E1 promised that a dial change, a dated `SetParam`, keeps a checkpoint
   taken before it; but a `SetParam`'s value was a registered param, in `world_id` and in the
   state, so any new value refused every checkpoint as `WrongWorld` and changed the hash stream
   from tick 1. A param read by the schedule alone (a value a `SetParam` copies, a recurring
   period) is now a `ScheduleParam` held by the `Schedule`: not registered, not in the state and
   not in `world_id`, while the firings it shapes stay in `prefix_id`. The loader learns which
   params those are in a first pass and resolves again without them. `Firing::source` is the
   source's key. A source the world also reads stays registered and fixed (§2.1, §2.6, §7.6).
3. Load checks (§2.6, §4): a ledger tolerance of 1 or more (`ToleranceNotBelowOne`), which would
   make R2 vacuous, and a currency on either side of a recipe (`CurrencyInRecipe`), which would
   make a cost in money or money with a production provenance (R14).
4. Settlement (§3.2): not only the last taker's quantity differs from nominal. Any take from a
   holding of several lots moves the sum of the lots taken, so `shipped <= offered` and `filled
   <= feasible` hold only up to rounding. Payments stay nominal.
5. Income (§2.5): Phase 0 asserts the market-level identity, expenditure equals receipts, on
   every gate tick (`gate_settles_by_one_filled_quantity`). The income identity proper,
   `I = wN_a + rT`, needs wages and rents, which arrive with Phase 2.
6. The registry listing (§5) gives each param's use as `live`, `fixed` or `schedule`.
7. Tests (§11). Added: `checkpoint_digest_refuses_an_edited_state`,
   `schedule_params_stay_out_of_the_world`, `ledger_tolerances_must_be_below_one` (core);
   `one_filled_quantity_serves_many_takers`, `a_market_whose_fill_underflows_does_not_trade`
   (markets); `tiny_cash_and_stocks_still_pay_out_and_produce` (agents);
   `resume_after_a_new_dial_value_equals_full_rerun`, `resume_refuses_an_invalid_state`,
   `gate_settles_by_one_filled_quantity`, `hooks_read_the_phase_start_state_every_tick`
   (engine); `resume_refuses_an_edited_checkpoint` (cli). Strengthened:
   `rationing_is_recorded_per_class` (two members of one class on one side, one cash-short),
   `gate_rations_and_records_by_class` (every class line is the fold of its members; `D` against
   the class lines under a relative bar, since the two fold in different orders, and against the
   actor-order fold exactly), `clock_conversions` (`ticks` at 12, 52 and 365 ticks a year),
   `a13_annual_quantities_invariant` (every pension occurrence and the bread's life),
   `no_public_api_hands_out_mut_state` (no `&mut self` method on `Sim` but `step`,
   `step_traced` and `run_until`; no `&mut` to a `SimState`, `World` or `Inventory` anywhere in a
   signature; regression fixtures), the source scans (a `#[cfg(test)]` item is cut out rather
   than the file cut at it, so shipped code after an early test helper is scanned; named float
   constants such as `f64::EPSILON` are refused outside `core::num`), and
   `scripted_specs_are_checked_at_load` (a currency in a recipe). Each was checked against the
   mutation that showed the gap.
8. Steps (§12): P0.6 is this round's fixes; the housekeeping of §12 follows as its own step.

**Amended at P0.7** (fixes from adversarial review, round 2), the same way.

1. Rounding (R2; §2.2, §2.4, §2.5). Lots are `f64`, so a take's split (`old − rest`) and a
   merge (`a + b`) round, and the rounding creates or destroys up to half an ulp of the larger
   operand: a transfer of 1 from a lot of 1e17 (ulp 16) left the source at 1e17 while the taker
   gained 1, and 6 merged into such a lot vanished. The claim that a transfer "moves exactly what
   it takes, so it cannot mint" was false, and the ledger let the units through: its walk rounds
   the same way, and its tolerance, `rel_stock` of the stock, is millions of units at 1e17. Now
   `Inventory::take` returns a `Taken { lots, rounding }` and `put` returns the merges'
   rounding, each measured exactly by TwoSum (`num::two_sum`), and `apply` declares them as a
   ledger line with the new provenance `Rounding`, reserved to it like `Spoilage`. Takes stay
   nominal, so payments stay nominal and §3.2's argument is unchanged; the state and every hash
   are unchanged too. Nothing appears or vanishes without a provenance (for one lot; not
   across several, P0.8 amendment 2), and the ledger's tolerance now covers only the walk's
   own rounding and the fold of the declared lines. The
   alternatives were exact quantities (integer quanta), which would give up the range that §3.3's
   tiny and huge prices test, and exact takes (the taker gets `old − fl(old − rest)`), which make
   payments non-nominal and let a buyer pay for goods whose shipment rounded to nothing.
2. The run's ledger (R2; §2.5, §7.4). A leak below each tick's tolerance could add up over a run
   unseen. `RunLedger` closes each tick's ledger, folds it into run totals and checks the run as
   one tick, from its first open to this close, with the same registered tolerances and the
   run's gross flow; the walks' rounding telescopes, a leak does not. A run breach stops the run
   in phase 7 like a tick's (`CoreError::Conservation`, whose `Breach` now carries `since`). The
   `Sim` holds one from its starting tick (genesis, or a resumed checkpoint's), and every
   `TickReport` carries its `RunAudit`: the run's lines, each good's drift over the run, and its
   margin.
3. E1's boundary (§0, §1, §7.6). The engine re-exported core whole, so a frontend depending on
   the engine alone could reach `apply` and `resolve`, write a state and resume it through
   `Checkpoint::of`. The engine now re-exports markets and agents, and core's read-only types in
   its prelude (`SimState`, `RunAudit`, `Breach`, `Clock` and `Life` added), never core itself;
   its tests use core as the engine's own dependency. §7.6 said the re-export was needed by the
   replay audit, which was wrong: the audit is engine code and uses core directly. What a resume
   still trusts is stated in E1 and §7.6.
4. Load checks: none new. A ledger tolerance of 1 or more and a currency in a recipe were refused
   at P0.6 (amendment 3 there).
5. The API (§2.2, §2.5): `Taken`, `RunLedger`, `RunAudit` and `num::two_sum` are public;
   `Inventory::put` and `put_qty` return `Result<f64, CoreError>`, the rounding they created.
6. Tests (§11). Added: `two_sum_is_exact`, `split_and_merge_rounding_is_measured`,
   `transfer_rounding_is_declared`, `run_ledger_catches_a_leak_below_each_ticks_tolerance`,
   `recurring_last_is_inclusive` (core); `gate_breach_stops_the_run`,
   `gate_rounding_is_declared`, `resume_boundary_is_the_firing_tick`,
   `no_reexport_hands_out_core_writer` and the engine's doc tests (engine);
   `conservation_breach_stops_the_run` (cli). Strengthened: `two_kinds_same_number_settle_apart`
   (a pop sells beside the desk with its number, which holds the same good and coin),
   `no_forgiveness_currency_moves_only_by_transfers` (its only ledger lines are `Rounding`, and
   they are exact: each good's lots, summed exactly, move by exactly those lines),
   `gate_conserves_every_tick` (the run's ledger every tick). `atomic_transfer_cannot_mint` keeps
   its name: a transfer mints nothing unrecorded. Each was checked against the mutation that
   showed the gap.
7. Steps (§12): P0.7 is this round's fixes; the housekeeping of §12 follows as its own step.

**Amended at P0.8** (housekeeping), the same way. No crate changed.

1. Steps and CI (§12). Housekeeping is P0.8. The CI skeleton is `scripts/gate.sh` and
   `.github/workflows/ci.yml`, not `ci/gate.sh` and `ci/gate.ps1`. The script runs the whole gate
   in WSL, or on any Linux machine, with the build outside the tree: formatting, clippy with
   warnings denied, the tests, `gate_repeat_identical_hashes` by name (so a rename cannot drop it),
   and two runs of the gate tape through the binary, whose hash streams must be identical; then
   the wasm32 check, recorded and not gated. Hosted CI runs the same script on GitHub's Ubuntu
   runner once the branch is pushed. The Windows check is run by hand with the same commands, and
   the two platforms' hash streams are compared by hand and recorded in STATE.md.
2. This contract was checked against the code after round 3 of the adversarial review, whose
   findings are open in STATE.md (O5 to O13). Where the text claimed more than the code does, it
   now says what the code does, each place marked "open at P0.8":
   - §2.2, §2.4, §2.5: a split or a merge of one lot is declared exactly, which covers every
     currency. A burn declares the float fold of the lots it takes, and `put` returns the float
     sum of its merges' errors, so a burn or a transfer across several lots of a perishable good
     can leave the rounding of that fold undeclared. It is below half an ulp of the larger operand,
     so no tolerance can see it (O5).
   - §2.5, §7.6: the digest covers the state, not `world_id` or `prefix_id`. Edited to the value
     a refusal prints, either one lets a refused checkpoint resume (O7).
   - §2.5, §7.4: the run's ledger lives in the `Sim`, not in the checkpoint. A resumed run audits
     from its checkpoint, so a slow leak that stops an uninterrupted run can pass a run resumed in
     short segments (O8).
   - §2.6: two dated events that fall in one tick fire in key order, not in date order, so their
     order can change with `ticks_per_year` (O9).

   P0.9 closed all four, and O6 and O10 to O13 (amendments below).

**Amended at P0.9** (fixes from adversarial review, round 3), the same way. Each place P0.8
marked "open at P0.8" now says what the code does. The gate world's hash stream is unchanged:
no fix changes what a state holds or how a tick moves it.

1. Rounding across several lots (O5; R2; §2.2, §2.4, §2.5). A burn declared `−q`, `q` the float
   fold of the lots it took, and `Inventory::put` returned the float sum of its merges' errors,
   so the rounding of either fold had no line: a burn of `All` from bread held as [1e17, 7]
   declared −1e17 while 1e17 + 7 were destroyed, and a transfer of [7, 1e-17] into [1e17, 1]
   declared −7 while 7 + 1e-17 vanished. Now a burn folds the lots it takes with TwoSum and
   declares the sum under its provenance and each step's loss as `Rounding`, and `put` returns
   each merge's rounding (`Result<Vec<f64>, CoreError>`), which `apply` declares one by one. Every
   delta's declarations now add up exactly to what it created or destroyed; the ledger's fold of
   them into lines, and its walk, are what the tolerance covers. No state changes.
2. The tolerance's flow term (O6; A12; §2.5) is pinned: `gross` is Σ moved + Σ |declared|
   exactly, for a tick and for a run, and the tolerance is `rel_flow·gross + rel_stock·max(|open|,
   |close|)` exactly. Before, counting a moved quantity as `q·q + q`, or the run's fold of gross
   twice, widened the registered tolerance and every test passed.
3. Checkpoints, format 3 (O7, O8; R2, E1, N11; §2.5, §7.6). The digest is FNV-1a 64 over bincode
   1 of `(world_id, prefix_id, state, run)`, where `run` is the run's ledger. Under format 2 it
   covered the state alone, so a refused checkpoint whose `prefix_id` (or `world_id`) was edited
   to the value the refusal prints resumed under another past (or another world, with the
   world's edit silently not applied). A checkpoint now also carries the run's ledger, and
   `Sim::resume` continues it instead of opening a new one (in the review's run, a leak of 1e-9
   coin a tick stopped an uninterrupted run at its fourth tick and passed all 2,080 ticks when
   resumed every 2). A resumed
   run's reports now equal the uninterrupted run's in full, `run` included (§7.4's exception is
   gone). `RunLedger` is serialisable (its lines as a list in (good, provenance) order) and has
   `next` and `fits`; `Checkpoint::of(&world, state, run)` returns `Result` and refuses a run's
   ledger that does not fit the state; `Checkpoint::run` reads it; `validate` checks it. Formats 1
   and 2 are refused.
4. Dated events in one tick fire in date order (O9; A13, E8; §2.6, §9). A `Firing` carries its
   `day` (`Date::days`: a dated event's date, and a recurring occurrence's tick's first day,
   `Clock::first_day`), and firings sort by (tick, day, key). Before, a firing kept no date: at
   12 ticks a year a cut (`z.cut`) and a restore (`a.restore`) twenty days later shared a tick and
   fired in key order, so the mine stayed cut for nine years instead of twenty days, while at 52
   the restore won. The gate world has no two firings in one tick, so its `prefix_id`s and hashes
   are unchanged.
5. The scripted actor's per-tick conversions (O10; A13; §4) are pinned at 12, 52 and 365 ticks a
   year: every buy and `Flow` sell line posts `q/ticks_per_year`, `AllHeld` what is left, and the
   payout and spending total are `1 − exp(−r/ticks_per_year)` of the cash they draw on. Hard-wired
   weekly forms, per-year quantities and `v·Δ` for a share all passed before.
6. The frontend guard (O11; E1, E4; §7.6). The scans read what a frontend can reach, not only
   the engine's `pub fn` signatures: every public function of the engine, markets and agents
   (`&mut` to a `Sim`, `Checkpoint`, `SimState`, `World` or `Inventory` anywhere in a signature,
   a free `fn(&mut Sim)` included); every `impl` of `Sim`, core's `Checkpoint` and core's
   `SimState`, wherever it lives, trait impls included (`DerefMut`, `AsMut`, `BorrowMut`,
   `IndexMut`, or any method with `&mut self`); the three types' fields; and every re-export of
   core, now an allow-list of the prelude's read-only types, with `{self ..}`, `self as`, a
   leading `::`, a private alias and `pub extern crate` treated as core whole. Each of the five
   writers the review built is a fixture, and the engine's compile-fail doc tests add
   `Checkpoint::state_mut`, `SimState::holdings_mut`, `&mut **sim` and `sim.as_mut()`. None of
   these writers exists; the guard would now see one.
7. The final checkpoint write (O12; N11; §8) is tested: `--out` naming a regular file with no
   `--checkpoint-every` (or an `--until` that is not a multiple of it) exits 3 with the error and
   no final hash. The code was right; ignoring the error passed every test.
8. The literal scan (O13; R4, A12; §11) also flags an integer other than 0 and 1 made float
   (`as f64`, `f64::from`, `.into()`, a named integer constant made float), a float spelled as
   its bits (`from_bits` of a literal) and `parse::<f64>`; clock.rs's calendar constants may be
   converted, and an integer type's limits are not behavioural. Shipped code has none.
9. Tests (§11). Added in core: `several_merges_report_each_rounding`,
   `multi_lot_rounding_is_declared_exactly`, `flow_tolerance_is_pinned_for_a_tick_and_a_run`
   and `checkpoint_digest_covers_identity_and_run`. In markets:
   `tiny_imbalances_move_the_price`. In agents: `per_tick_conversions_follow_the_clock`. In
   engine: `resumed_run_stops_where_the_uninterrupted_run_does` (a unit test in `sim.rs`, since
   it needs the `Sim`'s own state to leak into), `resume_refuses_an_edited_identity`,
   `same_tick_events_fire_in_date_order` and five doc tests. In cli:
   `failed_final_checkpoint_save_stops_the_run` and `resume_refuses_an_edited_identity`.
   Strengthened: `unsorted_events_fire_in_order` (two events in one tick whose key order and date
   order disagree), `gate_resume_from_checkpoints` (every resumed report equals the uninterrupted
   run's in full), `no_public_api_hands_out_mut_state`, `no_reexport_hands_out_core_writer`,
   `the_scanner_reads_what_it_should` and `no_behavioural_float_literals`. A test-only record of
   the ledger's posts (`Ledger::posts`, `#[cfg(test)]`) lets core's tests check each delta's
   declarations before the lines fold them, with an exact expansion sum
   (`testkit::exactly_zero`). Each was checked against the mutation that showed the gap.
10. Steps (§12): P0.9 is this round's fixes.

**Amended at P1.1** (oracle unit 1a joins the workspace), the same way. Nothing on the engine
path changed but core's `num`, which gained one function; no hash moved.

1. `core::num::fma(a, b, c)`: `a·b + c` rounded once, libm's `fma`, which is correctly rounded
   (the FMA instruction where the CPU has one, an exact software form where not), so it gives
   the same double on every platform (§2.5, §9). The oracle needs it for `1 − u·a` and
   `1 − aδ`, which must not round the product first. `f64::mul_add` stays denied, and the state
   path still writes `a * b + c`. Test: `fma_rounds_once` in core (an exact product-error check
   in integers, and golden bits).
2. `crates/oracle` (`rustyecon-oracle`, library `oracle`) is a member (§1). It depends on core
   alone, for `num`, and nothing on the engine path depends on it (R13). Its 114 tests run under
   `cargo test --workspace`. The source scans of §11 do not read it: it is not on the run's
   path.

**Amended at P0.11** (the GUI's design and plan amendment A14, docs/GUI.md), §13 only; no crate
changed. The rest of A14's ENGINE amendments (the step definition in the preamble, §1 and §12:
`default-members`, the non-blocking GUI check, the lockfile fetch, and the GUI's own gate script)
land with the commit that adds `crates/gui`, and §11's scan list gains `crates/observe` when
Phase 2 creates it.

1. §13: `crates/gui` is built with egui (eframe), its stack pinned in GUI.md §3.1, from G0 after
   session 2.
2. §13's session-2 row gains D10's engine items 1, 2 and 4 (GUI.md §7.2, each with its test).
   Item 3, chunked stepping equal to `run_until`, is met already.

**Amended at P2.0.1** (2026-09-26: the Phase 2 probe's agents, docs/probe/RULES.md), the same way.
The user ruled a time-boxed probe before session 2: can agents at the paper's margins reach the
oracle's equilibrium of the SSRN Appendix B economy? Its frame is PROBE-SPEC
(`D:/rustyecon-probe/frame/`). Core, markets and engine did not change; the agents crate grew,
and `crates/probe` joined. The gate world's hash stream did not move: its final hash is
`0x61f9c8529131ff17` on WSL and on Windows, as at P0.9.

1. Four behaviour kinds (§4, §5): `Provider`, `Workers`, `GoodDesk` and `MachDesk`, new variants
   of `RawSpec` and `Spec` after `Scripted`, resolved in `agents::roles::spec` and run by
   `agents::roles::rules`. Every rate, flow, coefficient and dial is a registered param read at
   use time (R4); the only inline numbers are genesis state (the good desk's human share, a step
   rule's scale), listed by `inline_numbers`. The rules, the dials and their lineage are
   docs/probe/RULES.md.
2. Their state (§4; PROBE-SPEC G10): `ActorState` gains `Provider`, `Workers`, `GoodDesk` and
   `MachDesk` after `Scripted`, so the scripted state keeps its encoding and every existing hash.
   One new delta, `AgentDelta::SetState { actor, state }`, replaces an actor's state with one of
   the same kind; its owner is the actor, so the whitelist (§7.3) admits it from any hook
   unchanged. `apply` refuses a state of another kind, a value that is not finite or has its sign
   bit set (so not `-0.0`, since the state is hashed, R8), and a share above 1; nothing changes on
   a refusal. `validate` checks each actor's state against its spec's kind and the same values,
   so a resume checks them too. `AgentDelta` loses `Eq`, which `f64` fields do not have.
   `SetActive` on an actor that is not scripted is an `Ext` error.
3. Load checks (§4), each a `LoadError` with its path: a role's goods are distinct and not
   currencies; its params have their units; the good desk's genesis share lies in [0, 1];
   `Cast::new` refuses a desk role declared a Pop or a household role declared a Desk, an
   endowment minted in `decide` that is not an `Instant` good, and a transfer or payout to the
   payer.
4. What the hooks emit is §7.3's list as it stood: endowments of `Instant` goods, transfers and
   `Actor` deltas in `decide`; `Production` and `Consumption` burns, `Production` mints and
   `Actor` deltas in `produce`. Like the scripted actor, no role posts an order admission refuses
   or a burn that falls short, whatever the prices (docs/probe/RULES.md §2).
5. `crates/probe` (`rustyecon-probe`, library `probe`, PROBE-SPEC G12) is a member (§1): the
   generator of `tapes/appb.ron`, named perturbations, the harness that reads each tick's
   observables and scores them against the oracle outside the Sim, and the binaries `appb-tape`
   and `probe`. It depends on the engine, the oracle and core's read-only helpers; nothing depends
   on it, so no agent reaches the oracle (R13). The source scans of §11 do not read it: it is not
   on the run's path. Clippy's workspace lints do.
6. The tape schema stays 1 (§5; docs/TAPE.md): the kinds are new variants of the agents' spec,
   as §5 foresaw ("Phase 2's kinds are new variants, not a new schema shape"), and every schema-1
   tape loads and means what it did.
7. Tests (§11). In agents (`tests/roles.rs`): `roles_never_overbudget_or_overdraw`,
   `good_desk_makes_output_at_either_corner`, `ex_post_assignment_uses_up_both_inputs`,
   `technique_moves_toward_the_task_measure`,
   `households_offer_the_cdf_and_the_provider_funds_one_basket_per_head`,
   `cash_rule_outlay_follows_coin_and_markup`, `role_state_rejects_unclean_values`,
   `roles_read_only_their_view`, `role_specs_are_checked_at_load` and
   `role_specs_round_trip_in_canonical_form`. In probe (`tests/appb.rs`):
   `appb_tape_is_its_generators_output`, `appb_tape_loads_and_runs_deterministically`,
   `appb_conserves_every_tick`, `appb_holds_at_the_oracle_point` (mode A for 20,000 ticks),
   `appb_variants_hold_at_the_oracle_point`, `perturbations_parse_and_apply` and
   `rows_carry_the_oracle_target`. The seam and determinism tests match the scripted spec with
   `let .. else`, since the spec enum is no longer irrefutable.

**Amended at S2.2** (2026-09-26: Phase 0 session 2, the GUI's engine asks, D10 items 1, 2 and
4; docs/GUI.md §7.2, docs/CERTIFY.md §2), the same way. Core, markets, agents and engine changed.
No state hash and no `prefix_id` moved: all 2,080 gate and 20,000 appb per-tick hashes equal
`cf3c0ff`'s byte for byte, on WSL and on Windows (finals `0x61f9c8529131ff17` and
`0xe1fa082b26995867`). Every `world_id` changed once (item 4 below).

1. `world_id` without the `fixed` flag (D10 item 1; §2.6). The run content hashes each param as
   (key, unit, genesis value). The flag is schedule structure: a param the world reads turns
   fixed when a dated `SetParam` copies it. Hashing it moved the world, so a new source event
   refused every checkpoint as `WrongWorld`. The structure a fixed use makes is hashed where it
   lands: a shelf life as ticks in the goods, the tolerances in `tol`, and a period or a copied
   value in the schedule, under `prefix_id`. The flag still bars a `SetParam` on the param, at
   load and in `apply`.
2. `FiredEvent` names its source (D10 item 2; §7.4): `source: Option<Key>`, from
   `Firing::source`. It is the param a `SetParam` copied, which carries the new value's basis,
   and `None` for any other action. It is a `Key`, not a `ParamId`, since a schedule param has
   no id.
3. Each use of a param, with its method (D10 item 4; §2.1, §2.6, §4, §5, §6, §7.1).
   - `core::clock` gains `ClockMethod` (`Value`, `Flow`, `Share`, `LogStep`, `Compound`,
     `Fraction`, `Weight`, `Ticks`: §6's rows, each implying its unit), `Site { param, method }`
     and `ParamSite { path, method }`.
   - `Resolver::param(key, method, use_, field)` takes the method in place of the unit. It
     records a `ParamSite` at the reference's path and returns a `Site` with the same method.
   - `ParamDef` and `ScheduleParam` gain `sites`, in (path, method) order. A shelf life is
     `Ticks` at the good's `life`, and a period `Ticks` at the entry's `every`. A `SetParam`'s
     source takes its target's methods at the event's `to`. A `SetParam`'s target is written
     there, not read, so its `act.param` is not a site.
   - The world and every resolved spec hold a `Site` where they held a bare `ParamId`:
     `GoodDef::price_rate`, `MarketConfig::ema_time_constant`, `Tolerances`, and every param
     field of the scripted and role specs. The run reads a param only through it,
     `Site::per_tick(&Params, &Clock)`. One rule scales an annual rate before converting it,
     the cash rule's `share(v·μ^κ)`; it reads `Site::value` and converts with `Site::convert`,
     which applies the site's own method. So the method a site declares is the conversion the
     run uses, and a source scan keeps it so.
   - `engine::registry`'s `Entry::Param` lists `sites: Vec<SiteLine { path, method, per_tick }>`
     in place of one guessed `per_tick`. The guess read the unit and whether some good's price
     moved at the param, so a rate that was both a price rate and a spending rate showed as
     `log_step` alone. `Spec::sites` lists a spec's sites with their paths.
   - `ClockMethod` and `Site` join the engine's prelude and the scans' allow-list.
   - The registry's `sites` are not hashed. A `Site` inside the world or a spec is, since its
     method decides a number. That, with item 1, moved every `world_id` once.
4. Re-baselines, from `cf3c0ff`, equal on WSL and on Windows. Gate `0xbecdc746fc86ce97` became
   `0x43628a8e0fd5f695`; appb `0x3f689d670fe877c6` became `0x26f12f8a0bc27540`; core's fixture
   `0x66d1181c6802a7fd` became `0x85336968874fbf6d`; markets' fixture `0xbdd0ee95c0bb590f`
   became `0xc3b1c948a42f06c6`. The genesis state hashes are unchanged (gate
   `0xf05d0f23826edf87`, appb `0x8d12ce44b614110a`). A checkpoint made before S2.2 is refused
   as `WrongWorld`; none is committed.
5. D10 item 3 was met already. `observing_changes_no_hash` gains a loop that advances the gate by
   `run_until` seven ticks at a time, observed between chunks, and gets the same stream (§11).
6. Tests (§11). In core: `clock_methods_convert_as_the_clock` and `param_sites_are_recorded`;
   `schedule_params_stay_out_of_the_world` also checks that a registered source keeps the
   world. In agents: `role_sites_name_their_methods` (`tests/roles.rs`) and
   `each_site_converts_as_registered` (`tests/seam.rs`). In engine:
   `new_source_event_keeps_world_id` (`edits.rs`), `fired_event_names_its_source` (`gate.rs`),
   `registry_names_each_use` (a new file, `tests/registry.rs`) and
   `params_are_read_only_through_sites` (`scans.rs`). The cli's `registry_lists_every_number`
   checks the new listing. Each was checked against the mutation it guards: hashing the flag, or
   hashing the sites (`new_source_event_keeps_world_id`); `source: None`
   (`fired_event_names_its_source`); one method per param, in the listing or in the record, and
   a source without its target's methods (`registry_names_each_use`); the step rule's `up`
   resolved as a share (`role_sites_name_their_methods`); a returned `Site` whose method differs
   from the recorded one (`each_site_converts_as_registered`); and a rule that converts around
   its site (`params_are_read_only_through_sites`).
7. docs/CERTIFY.md §2.4's `ScalePrice` and `MarketLine::trades` are not in this step. They land
   with the kick, which is their only user (CERTIFY §14).

**Amended at S2.3** (2026-09-26: Phase 0 session 2, certification; docs/CERTIFY.md §2.4 and C1),
the same way. Core and engine changed; markets and agents did not. No state hash and no
`prefix_id` moved: the new action and delta are appended, so every existing encoding stands, and
the gate world's final hash is `0x61f9c8529131ff17` as before.

1. The kick's action (§2.4, §2.6): `RawAct::ScalePrice { node, good, by }`, appended to the tape's
   actions, resolves to `StateDelta::ScalePrice { node, good, factor }`, appended after
   `AdvanceTick`. `by` is a `Dimensionless` param the schedule reads, as a `SetParam`'s source is:
   a param nothing else reads is a `ScheduleParam`, outside `world_id`, and a registered one is
   fixed by the use. Its one site is `Value` at the event's `act.by`, and the firing's `source`
   names it, so a fired `ScalePrice` names its factor in `FiredEvent::source` (§7.4). Load
   errors, each at its path: a factor that is not finite and positive (`BadValue`), a good with
   no market, a currency (`NoMarket`, new), and a `ScalePrice` in a recurring entry
   (`RecurringPriceShock`, new): a periodic price nudge would be an exogenous stabiliser (R3),
   and it would not restart a certify window (CERTIFY C5). The schema stays 1 (docs/TAPE.md).
2. `apply` (§2.4) takes a `ScalePrice` in `Phase::Events` only (`WrongPhase` otherwise), sets the
   posted price to `fl(p·factor)`, refuses a factor or a result that is not finite and positive
   (`BadValue`, the state untouched), and leaves the EMA alone. Hooks cannot emit it (§7.3): the
   whitelist is an allow-list, and `behaviour_output_is_whitelisted` gains the arm.
3. `MarketLine::trades()` (§7.4) is markets' `MarketFill::trades` predicate on the line's fields,
   so certify and the probe read one definition of a market that traded.
4. `core::hash::Fnv`, the incremental FNV-1a 64 hasher, is public with `new`, `write`, `finish`
   and `resume(digest)`, which continues a hash from the digest of what came before (FNV-1a's
   running state is its digest). Certify's manifest folds a hash file's lines with it (CERTIFY
   §9). It hashes bytes; it is not a writer of anything.
5. `crates/certify` (§1) is certify, `rustyecon-certify`, lib `certify`: criteria, the
   batteries, the kick check, the sealed certificate and the manifest (docs/CERTIFY.md). It
   depends on core (read-only helpers) and the engine; nothing on the engine path depends on it.
   The source scans read it too (§11): no threshold, no I/O, no core writer.
6. Tests (§11). In core: `scale_price_is_an_event_only_delta` (`apply.rs`: the phase, the EMA, a
   currency, bad factors and results, the tape's load checks, the factor as a schedule param
   with its `Value` site). In engine: `kicked_tape_keeps_world_and_past` (`edits.rs`: the gate
   with a kick at tick 1,000 keeps `world_id` and every prefix through 1,000; the tick-1,000
   checkpoint resumes under it, that tick's posted price is `fl(p·factor)`, the event names its
   factor, and the resumed run's hashes equal the kicked tape's run from genesis),
   `market_line_trades_as_markets_do` (`gate.rs`), and three scans of `crates/certify/src`,
   `certify_holds_no_threshold`, `certify_does_no_file_io` and `certify_hands_out_no_core_writer`
   (`scans.rs`, each checked on fixtures first). `behaviour_output_is_whitelisted` gains
   `ScalePrice`. Each was checked against the mutation it guards (CERTIFY, amended at S2.3).

**Amended at S2.4** (2026-09-26: Phase 0 session 2, the cli; docs/CERTIFY.md §3, §10, §12), the
same way. Only the cli changed, and certify gained its telemetry writer; no state hash,
`prefix_id` or `world_id` moved.

1. The cli (§8) depends on certify, with its `parquet` feature, and gains `rustyecon certify`,
   which writes the certificate, the manifest and the hash file (and with `--telemetry` the
   Parquet) and exits on the verdict: 0 on PASS, and the new code 5 on FAIL or UNSCORED.
2. Hashes name their run (§8, R16): a `--hashes` file opens with `#` lines naming the build, the
   tape by `tape_hash`, the `world_id` and the starting tick, and stdout prints a `run …` line
   before the final hash, in `run`, `resume` and `replay`. `build.rs` stamps the commit, the
   dirty flag, the target and the compiler.
3. `--out DIR` writes the run's manifest, and `resume` verifies its checkpoint against it (§8,
   §7.6's closing sentence): a missing, unreadable or disagreeing manifest is exit 3. The
   `--tape` help drops "a dated edit at or after its tick is allowed" until CERTIFY §15.1,
   question 2, is ruled. The engine's own `Sim::resume` is unchanged, and still allows it.
4. Tests (§11), in the cli: `hash_output_names_its_run`, `resume_requires_a_recorded_checkpoint`,
   `resume_records_its_parent`, `certify_command_exits_on_its_verdict` and
   `manifest_names_every_input`; `resume_refuses_another_tape` gains a dated edit after the
   checkpoint, which keeps world and past but not the tape. Tests that compare hash files skip
   the `#` header, and `gate_resume_through_product_path` gives each run its own directory, since
   each writes its own manifest.

**Amended at S2.5** (2026-09-26: Phase 0 session 2, the fix round of the bounded verification;
docs/CERTIFY.md's S2.5 amendment), the same way. Core, markets, agents and engine did not
change, so no state hash, `prefix_id` or `world_id` moved. Certify, the cli's build script and
tests, and `crates/probe` changed.

1. Certify's fixes are CERTIFY's: a scored run fails when it fires more `ScalePrice` events than
   its criteria register (none when the field is absent), the kick fires at every dated shock as
   well as at the end, and each reading carries its comparison, so the seal holds a pass flag to
   its readings. The scans of §11 still read `crates/certify/src`, and still pass.
2. The build stamp (§8) also watches the worktree's `logs/HEAD` and the common directory's
   `refs/heads` tree. On a branch whose ref was packed, a commit that touched neither `crates/`
   nor the index wrote a new loose ref that nothing watched, and the binary still named the
   parent commit.
3. `crates/probe` (§1) depends on certify, without its `parquet` feature (CERTIFY C11). Its
   harness reads each tick through `certify::Obs` and calls certify's oracle-free measures, so
   the two crates have one definition of each. Nothing on the engine path depends on either.
4. `scripts/gate.sh` runs `probe_battery_csv_unchanged` by name, after the committed
   certificates.
5. Tests (§11). In the cli, `resume_records_its_parent` also resumes against a parent manifest
   whose `genesis_hash` was edited: it verifies, and the child records the tape's own genesis
   hash. In probe, `probe_summaries_unchanged` and `probe_battery_csv_unchanged` (ignored, run
   by name) pin its report's tables. Certify's tests are CERTIFY §13's.
6. Recorded at S2.5. The committed certificates changed format, so `daa62af` regenerated them
   from a clean build of `05533b9`; both are PASS, with every reading's value as at S2.4, and
   appb gains one reading, `kick.errors`, 0.
   `scripts/gate.sh` is green in WSL and on Windows: 421 tests pass in the workspace on each
   machine, 2 ignored and run by name.

**Amended at S2.6** (2026-09-26: session 2 closed), the same way. No crate changed; docs, the
spine scripts' default cache (STATE O15) and `.gitignore` did.

1. §1 names what session 2 added: the cli depends on certify with its `parquet` feature, probe
   depends on certify without it, and the external crates gain `parquet` 60.0.0 (certify's
   feature, pure Rust) and `tempfile` for certify's tests.
2. §11's cli table gains the tests S2.4 and S2.5 added. Certify's tests are listed in CERTIFY
   §13, not here.
3. §12 gains session 2's steps, and §13's session-2 rows are marked done.

## 0. Engine invariants

Numbered so tests and reviews can cite them. Each has at least one test in §11.

- **E1 — A frontend never mutates state.** Every live intervention (a dial, a shock, a world
  edit) is a dated tape event, or a tape edit followed by a rerun from genesis or a resume from a
  compatible checkpoint (§7.6). A dial change is a dated `SetParam` event (§2.4) that copies a
  param only the schedule reads, so a checkpoint taken before it fires resumes under the edited
  tape, whatever the new value (§2.6). Nothing enters a run except through the tape: no
  command-line override, no setter on `Sim`, no `&mut` to a checkpoint's state, no checkpoint
  whose fields do not match its digest, and no writer of the state in the engine's API (it
  re-exports core's read-only types, never `apply`, `resolve` or the ledger). So every
  experiment is reproducible (R8) and every number keeps its provenance (R4). A checkpoint is
  the one input besides the tape, and its digest catches corruption, not forgery; a program that
  depends on core directly holds core's writer. §7.6 states what a resume trusts.
- **E2 — No global state and no I/O on the engine path.** `core`, `markets`, `agents` and
  `engine` hold no `static mut`, `thread_local!`, `Rc`, `RefCell`, `Cell`, `OnceCell`,
  `OnceLock`, `LazyLock`, `Mutex`, `RwLock` or atomics. They use no `std::fs`, `std::time`,
  `std::thread`, `std::env`, `std::process` or `std::net`, and no print macro (defect 6). All I/O
  is bytes in and bytes out (`Tape::from_ron`/`to_ron`, `Checkpoint::to_bytes`/`from_bytes`/
  `to_ron`/`from_ron`); paths and file extensions live only in the cli. So the engine can run
  in-process, behind a local server, or compiled to WASM.
- **E3 — Engine types cross threads.** `Sim`, `Tape`, `World`, `Checkpoint`, `TickReport`,
  `HoldingTotals` and every error type are `Send + Sync + 'static`. A frontend can run a `Sim` on
  a worker thread and send reports over a channel.
- **E4 — Reading is read-only and free.** No `Sim` method returns `&mut SimState`, `&mut World` or
  anything that reaches them, and no trait impl of `Sim`, `Checkpoint` or `SimState` hands out
  `&mut` either (checked since P0.9). Reading never changes a hash, and each `TickReport` agrees
  with the accessors read after the step.

- **E5 — A failed step poisons the `Sim`.** After any `RunError` the `Sim` refuses to step or
  checkpoint until it is rebuilt from a tape or a checkpoint (§7.5).
- **E6 — Hooks read the phase-start state.** Within a phase, every actor's hook reads the state
  as it stood when the phase began. Outputs are collected, checked, then applied in canonical
  order, so an actor's id never decides what it sees.
- **E7 — Hook output is whitelisted.** Each hook may emit only the deltas and orders §7.3 lists
  for it; anything else is `RunError::ForeignWrite` (R13, R2).
- **E8 — The tape is stable and keyed.** It carries a schema version. Every entity has a stable
  key, and the loader orders everything by key, never by file position. Reformatting a tape or
  reordering its lists changes no id and no hash (§2.6).

## 1. Crates

```
Cargo.toml            [workspace] members = ["crates/*"], resolver = "2"; edition 2021
rust-toolchain.toml   channel = "1.97.1", components = ["rustfmt", "clippy"]
clippy.toml           disallowed types (std hash containers), methods (transcendentals, powi, mul_add)
crates/core     rustyecon-core     ids, keys, goods, clock, units, registry, inventory, state, deltas,
                                   apply, ledger, hash, checkpoint, the tape schema and its resolver,
                                   num (libm)
crates/markets  rustyecon-markets  orders, admission, clearing, settlement, price update
crates/agents   rustyecon-agents   the Behaviour seam, View, the scripted actor
crates/engine   rustyecon-engine   Sim: the tick loop, step, run, checkpoint, resume, the replay audit;
                                   TickReport; the hook whitelist
crates/cli      rustyecon-cli      bin `rustyecon`: argument parsing, file I/O, exit codes; depends on
                                   certify with its `parquet` feature (S2.4)
crates/certify  rustyecon-certify  lib `certify`: criteria, batteries, the kick, the certificate and
                                   the manifest (docs/CERTIFY.md; S2.3); telemetry behind the feature
                                   `parquet` (S2.4); nothing on the engine path depends on it
crates/oracle   rustyecon-oracle   the equilibrium solver, Phase 1 (unit 1a at P1.1); lib `oracle`
crates/worldgen rustyecon-worldgen empty until Phase 4; nothing depends on it yet
crates/probe    rustyecon-probe    the Phase 2 probe's harness (P2.0.1); lib `probe`; reads certify's
                                   oracle-free measures, Parquet-free (S2.5); nothing depends on it
tapes/appb.ron                     the probe's Appendix B world, generated (docs/probe/RULES.md §4)
tapes/gate.ron                     the gate world (§10)
```

Dependencies run one way: core ← markets ← agents ← engine ← cli, and a future frontend crate
depends on engine the same way. Core depends on no workspace crate (N14). `crates/oracle` came
from another run and joined through the glob at P1.1; it depends on core alone, for `num`, and
nothing on the engine path depends on it (R13). The engine re-exports markets and agents under their own names, plus a
`prelude` of the types a frontend names (ids, keys, `Holder`, `Date`, `Tape`, `World`,
`Checkpoint`, the `SimState` a checkpoint holds, `TickReport` and its lines, the errors), so a
frontend depends on the engine alone. It does not re-export core: core's writer (`apply`,
`resolve`, the ledgers) stays out of a frontend's reach (E1; amended at P0.7). P0.2 points the
cli's `Cargo.toml` at the engine only.

External crates, all cached on both machines: `serde 1.0.228` (derive), `ron 0.8.1`, `bincode
1.3.3`, `libm 0.2.16`, `clap 4.6.1` (cli only), `tempfile 3.27.0` (dev only: the cli's and
certify's tests), and from S2.4 `parquet =60.0.0` with the feature `lz4` only, behind certify's
feature `parquet`, which only the cli turns on (pure Rust, no C build; docs/CERTIFY.md §12, C7).
Not used: `rayon`, `thiserror`, `HashMap`, `HashSet` (R8). Error enums are written by hand, with
`Display`.

## 2. rustyecon-core

### 2.1 Ids, keys, goods, nodes, clock, units, registry (defect 10)

```rust
// one macro: `pub struct X(pub u32)` + idx(); Copy, Eq, Ord, Hash, Debug, Serialize, Deserialize
id!(GoodId); id!(NodeId); id!(ChannelId); id!(ClassId); id!(DeskId); id!(PopId); id!(ParamId); id!(EventId);
pub enum ActorId { Desk(DeskId), Pop(PopId) }              // derived Ord: all Desks, then all Pops
pub enum Holder { Actor(ActorId), Escrow(NodeId, GoodId) } // an escrow exists only inside phase 3
pub struct Key(String);                                    // [a-z0-9_.-]+, checked at load
pub struct GoodDef { pub id: GoodId, pub key: Key, pub life: Life, pub price_rate: Option<Site> } // LogStep
pub enum Life { Indefinite, Instant, Ticks(u32) }          // Ticks(L), L >= 1, from a Years param
pub struct NodeDef { pub id: NodeId, pub key: Key, pub currency: GoodId }
pub struct ChannelDef { pub id: ChannelId, pub key: Key, pub from: NodeId, pub to: NodeId } // static only
pub struct Clock { pub start: Date, pub ticks_per_year: u32 }                               // §6
pub struct Date { pub y: i32, pub m: u8, pub d: u8 }       // "YYYY-MM-DD" on the tape
pub enum Unit { Dimensionless, Years, FlowPerYear, RatePerYear, CompoundPerYear, FractionPerYear }
pub struct ParamDef { pub id: ParamId, pub key: Key, pub unit: Unit, pub genesis: f64,
                      pub basis: Basis, pub fixed: bool,
                      pub sites: Vec<ParamSite> }                    // each use (S2.2), not hashed
pub enum Basis { Measured { source: String, vintage: String }, Literature(String),
                 Approximate(String), Fitted { fit: String }, Assumed(String) }  // R4's five tags
pub struct Registry { params: Vec<ParamDef> }                        // by ParamId
pub struct Params<'a> { values: &'a [f64], registry: &'a Registry }  // current values, from SimState
impl Params<'_> { pub fn get<U: UnitKind>(&self, p: ParamId) -> Result<U, CoreError>; }
// UnitKind: one newtype per Unit (Dimensionless(f64), Years(f64), FlowPerYear(f64), ...).
// get checks the registered unit against U. Since S2.2 the run reads through a Site instead:
pub enum ClockMethod { Value, Flow, Share, LogStep, Compound, Fraction, Weight, Ticks } // §6's rows
impl ClockMethod { pub fn unit(self) -> Unit; pub fn name(self) -> &'static str;
                   pub fn per_tick(self, c: &Clock, v: f64) -> Result<f64, ClockError>; }
pub struct Site { pub param: ParamId, pub method: ClockMethod }     // one use, as a spec holds it
impl Site { pub fn per_tick(&self, p: &Params, c: &Clock) -> Result<f64, CoreError>;  // the read
            pub fn value(&self, p: &Params) -> Result<f64, CoreError>;               // unit-checked
            pub fn convert(&self, c: &Clock, v: f64) -> Result<f64, CoreError>; }    // by its method
pub struct ParamSite { pub path: String, pub method: ClockMethod }  // one use, as the registry lists it
```

- **Keys (E8).** Every tape entity has a key: goods, nodes, channels, classes, params, actors,
  events and recurring entries. A key is unique within its kind; desks and pops share one
  namespace, and so do events and recurring entries. A key is never reused for a different
  entity, and a rename is a new entity. The loader numbers each kind densely in key byte order,
  never in file order. Dense ids belong to core and are valid against one `World`, which maps
  both ways (`id_of`, `key_of`). Anything kept across a tape edit is kept by key.
- **No bare numbers on the delta path.** Every map is keyed by `ActorId` or `Holder`, never by a
  bare `u32`. July's `(u32, u32)` keys were defect 10 (`v2p3: systems/transactions/mod.rs:200-219,
  390-395`).
- **Goods.** July's `movement_type`, `divisible` and `storage_cost_per_tick` go (nothing reads
  them), and `alpha` becomes the registered `price_rate` (`RatePerYear`). A currency is
  `Indefinite`, has no `price_rate`, has no market, and is priced at 1 by definition. Every cost
  is a recipe input (R14); no floor or guard is an amount of currency.
- **Instant goods.** A lot of an `Instant` good is minted with life `Some(0)`. It trades in
  phases 2–3 of the tick it was minted, can be used in phase 4, and dies in phase 5a. It may be
  minted only in phases 0 and 1; a mint anywhere else is `CoreError::WrongPhase`. That rule
  prevents July's defect, where a good minted in phase 4 spoiled in phase 5 and never traded. The
  gate world has no Instant good. Phase 2's labour hours and parcel services are Instant, minted
  as `Endowment` in `decide`.
- **Params.** The genesis value, unit and basis live in `World`. The current value lives in
  `SimState` (hashed) and changes only by `SetParam`. Every reader reads at use time, and nothing
  caches a param's value beyond the call that read it. A param is `fixed` when the loader turns
  it into structure (a shelf life, a recurring period, or the value a `SetParam` copies) or when
  changing it would change what a past tick meant (the ledger tolerances). A `SetParam` on a
  fixed param is a load error, and `apply` refuses one too (`FixedParam`). A param that only the
  schedule reads (a value a `SetParam` copies, a recurring period) is not registered at all: it
  is a `ScheduleParam`, schedule content like the events (§2.6; amended at P0.6). The `fixed`
  flag is not part of `world_id` (amended at S2.2, D10 item 1).
- **Sites** (amended at S2.2, D10 item 4). Each reference to a param is a use with a
  `ClockMethod`, one of §6's rows. The resolver records it in the param's `sites` as a
  `ParamSite` at its tape path, and the world or spec that makes it holds a `Site` with the same
  method. The run reads a param only through a `Site`, so the method the registry lists at a
  path is the conversion the run applies there. A `RatePerYear` is a `Share` where it draws on
  a stock and a `LogStep` where it moves a price or a scale; only the use knows which.
- **Units.** `Clock`'s methods each take one unit type (§6), so using a rate as a flow does not
  compile. A `ClockMethod` implies its unit, and `Site::value` checks the registered unit
  against it at every read.

### 2.2 Inventory (defect 5; N9)

```rust
pub struct Lot { pub qty: f64, pub life: Option<u32> }  // None = indefinite; Some(0) dies at the next 5a
pub struct Inventory(Vec<(GoodId, Vec<Lot>)>);          // goods ascending; lots by life, None last
pub enum Amount { Qty(f64), All }
impl Inventory { pub fn get(&self, g: GoodId) -> f64;   // lot sum, left fold in lot order
    pub fn put(&mut self, g: GoodId, lots: Vec<Lot>) -> Result<Vec<f64>, CoreError>; // P0.9
    pub fn take(&mut self, g: GoodId, a: Amount) -> Result<Taken, TakeError>;
    // TakeError: Shortfall(Shortfall { requested, held }) | Invalid(q); nothing moves on either
    // Taken { lots: Vec<Lot>, rounding: f64 }: the lots taken, and what the split created
    pub fn age(&mut self) -> Vec<(GoodId, f64)>;        // spoiled per good
    pub fn lot_count(&self) -> usize; }
```

- **Coalescing (N9).** `put` merges lots of equal life. All lives fall together, so a good holds
  at most `max_life + 1` lots, and an indefinite good (currency included) holds exactly one. The
  form is canonical and lands before the first golden hash (A2).
- **Atomic `take`.** `Qty(q)` with `q > get(g)` returns `Err(TakeError::Shortfall)` and changes
  nothing; a non-finite or sign-bit-set `q` is `Err(TakeError::Invalid)`.
  `q == get(g)` or `All` takes every lot. Otherwise lots go soonest-expiring first (July's FIFO
  when a good has one life), splitting the last one taken. If rounding uses up the lots while a
  remainder is left, every lot is taken, so a request with `q <= get(g)` never falls short. A
  single-lot good (every currency) is taken as `lot.qty − q`, which is the arithmetic admission
  mirrors (§3.1).
- **Rounding** (amended at P0.7). Lots are `f64`, so the split's `fl(old − rest)` and a merge's
  `fl(a + b)` round, and the rounding creates or destroys up to half an ulp of the larger
  operand: 1 taken from a lot of 1e17 (ulp 16) leaves the lot at 1e17, and 6 merged into it
  vanish. The taker always gets exactly `rest`, so takes are nominal, and each rounding is
  measured exactly by TwoSum (`num::two_sum`: `a + b = s + e` exactly, with no fused
  multiply-add): `take` returns it as `Taken::rounding` (a take splits at most one lot), and
  `put` returns each merge's that rounded, `fl(..) − exact`, never `-0.0`, in merge order. They
  are not summed, since a float sum of two merges' errors rounds in turn (amended at P0.9, O5).
  `apply` declares each (§2.4).
- **Rules.** Non-finite quantities, and any quantity whose sign bit is set (`-0.0` included), are
  rejected in every build profile. A lot at exactly `+0.0` is dropped, with no epsilon. `age`
  drops lots at `Some(0)` and then decrements the rest, so a lot minted in tick t with `Ticks(L)`
  can sell in ticks t+1 … t+L (July's rule). A genesis holding of a perishable good is one lot
  with the good's full life. Serde keeps July's `Vec<(GoodId, Vec<(qty, life)>)>` form (`v2p3:
  types/inventory.rs:119-152`); loading re-sorts, re-coalesces, and rejects NaN and `-0.0`.

### 2.3 State and the extension seam (N14)

```rust
pub trait Ext: Clone + Debug + PartialEq + Send + Sync + 'static {   // unit markers
    type State: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;  // hashed
    type Delta: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;
    type RawActor: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync; // tape form
    type Actor: Clone + Debug + Serialize + Send + Sync;                                 // resolved
    type RawAction: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;
    fn resolve_actor(raw: &Self::RawActor, r: &mut Resolver) -> Result<Self::Actor, LoadError>;
    fn resolve_action(raw: &Self::RawAction, r: &mut Resolver) -> Result<Self::Delta, LoadError>;
    fn genesis(actors: &[ActorDecl<Self::Actor>]) -> Result<Self::State, LoadError>;
    fn apply(s: &mut Self::State, d: &Self::Delta) -> Result<(), CoreError>;
    fn owner(d: &Self::Delta) -> Option<ActorId>;  // None: a world-level delta, legal only from the tape
    fn validate(s: &Self::State, actors: &[ActorDecl<Self::Actor>]) -> Result<(), CoreError>; // resume
    fn canonical_actor(raw: &mut Self::RawActor);   // for to_ron: the spec's own lists in canonical order
    fn canonical_action(raw: &mut Self::RawAction);
}
pub struct NoExt;   // no behaviour: State = (), RawActor = Actor = () (actors hold goods, spec `()`),
                    // Delta = RawAction = `enum Never {}`
pub struct SimState<E: Ext> {           // fields private to core
    tick: u64,
    params: Vec<f64>,                    // current value per ParamId
    book: MarketBook,                    // flat [node·n_goods + good]: price, ema, supply, demand
    holdings: BTreeMap<Holder, Inventory>, // every declared actor, even if empty; no escrow between ticks
    ext: E::State,
}
```

- **Resolver.** It turns keys into ids and records each param reference with its use (`live` or
  `fixed`) and its unit, checked against the registry. The unused-param and fixed-param load
  checks read that record.
- **Readers.** `tick()`, `param(p)`, `params(&registry)` (typed by unit), `param_values()`,
  `book()`, `price(n, g)`, `ema`, `supply`, `demand` (each `Option<f64>`, `None` for an id outside
  the book), `holding(h)`, `holdings()`, `ext()` and `validate(&world)` (§7.6's shape check) are
  public. `apply` is the only writer (`v2p3:
  state/apply.rs:7-9`). Core holds no v1 or July agent struct (N14), and no constructor takes a
  default price.
- **Genesis book.** Each (node, non-currency good) gets its price from the tape, exactly once.
  `ema₀` is the genesis price, by definition, and `S₀ = D₀ = 0`. This removes July's genesis price
  and EMA of 1.0 (`v2p3: state/sim_state.rs:56,59`), one of the audit's three constants to
  register.
- **Growth without reopening core.** Phases 2 and 3 add behaviour kinds (variants of the agents'
  `RawActor` and `Actor`), actor state and `Actor` deltas. The tape-driven world deltas of Phase 3
  and later (enclosure status, recipe-version publication, law and tax rates) are `Ext` deltas
  whose `owner` is `None`. Their costs use the provenances already declared (§2.4). Entrants are
  declared at load as dormant desks, one per recipe version per region (PLAN §3.1), and a dated
  event activates them. The actor set is fixed at load.

### 2.4 Deltas, provenance and phases

```rust
pub enum Phase { Events, Decisions, Clearing, Settlement, Production, Upkeep, Prices, Measure } // 0–7
pub enum StateDelta<E: Ext> {
    SetPrice { node: NodeId, good: GoodId, price: f64 },                  // finite and > 0
    SetEma { node: NodeId, good: GoodId, ema: f64 },                      // finite and > 0
    SetVolumes { node: NodeId, good: GoodId, supply: f64, demand: f64 },
    Transfer { from: Holder, to: Holder, good: GoodId, amount: Amount },  // atomic; lots keep lives
    Mint { to: Holder, good: GoodId, qty: f64, prov: Provenance },        // life from GoodDef
    Burn { from: Holder, good: GoodId, amount: Amount, prov: Provenance },
    SetParam { param: ParamId, value: f64 },                              // unit fixed by the registry
    Age { holder: Holder },
    Actor(E::Delta),
    AdvanceTick,
    ScalePrice { node: NodeId, good: GoodId, factor: f64 },               // appended at S2.3: a price shock
}
pub enum Provenance { Production, Consumption, Spoilage, Endowment, Depreciation, Construction, Event,
                      Rounding }   // last, so the others keep their encoding (P0.7)
```

These are July's seven core arms (`v2p3: types/delta.rs`), with Add and Remove replaced by
`Transfer`, `Mint` and `Burn`, plus `SetParam`, and one seam, `Actor`, in place of July's other 19
variants. A transfer moves the lots it takes, or nothing when its source is short; July's paired
Remove/Add trap (`v2p3: docs/architecture/engine.md:440-445`) is gone. It is not exact: with
`f64` lots, the split of the source's last lot and the merge into the destination's can each
create or destroy up to half an ulp of the larger operand (§2.2), and `apply` declares that
exactly as a `Rounding` line, as it does for a mint's merge and a burn's split (amended at P0.7).
A transfer declares each merge's rounding on its own, and a burn of several lots declares their
float sum under its provenance and what that sum rounded away, measured by TwoSum at each step,
as `Rounding` (amended at P0.9, O5). So every delta's declarations add up exactly to what it
created or destroyed: no unit moves without a provenance, and the ledger's registered tolerance
covers only its own walk's rounding and its fold of the declared lines (§2.5). Every mint and burn
carries a typed provenance, with no `Default` and no serde default; `Transfer` carries none.
`apply` matches exhaustively.

| Provenance | Used for |
|---|---|
| `Production` | a desk's recipe inputs (burn) and outputs (mint) |
| `Consumption` | a pop's recipe inputs (burn) |
| `Spoilage` | lots dropped by `Age` |
| `Endowment` | output of a recipe with no inputs (Phase 0); Instant labour and parcel services (Phase 2) |
| `Depreciation`, `Construction` | Phase 3's capital; declared now so the ledger lines exist |
| `Event` | tape mints and burns |
| `Rounding` | what splitting or merging `f64` lots created or destroyed, and what a burn's float sum of several lots rounded away, measured exactly; `apply`'s alone (P0.7, P0.9) |

**Phase rules in `apply`** (`CoreError::WrongPhase`): an escrow holder appears only in
`Settlement`; `SetVolumes` only in `Clearing`; `SetPrice`, `SetEma` and `AdvanceTick` only in
`Prices`; `SetParam` and `ScalePrice` only in `Events`; `Age` only in `Upkeep`; a `Mint` of an Instant good only in
`Events` or `Decisions`. Which actor may emit what inside a phase is the engine's whitelist (E7,
§7.3), not core's. Besides: `Spoilage` is ageing's alone and `Rounding` is `apply`'s, so a `Mint`
or `Burn` carrying either is `ReservedProvenance`; a `SetPrice`, `SetEma` or `SetVolumes` on a
(node, currency) slot is `NoMarket`; a price or EMA must be positive. A `ScalePrice` sets the
posted price to `fl(p·factor)`, leaves the EMA alone, and refuses a factor or a result that is not
finite and positive (amended at S2.3; docs/CERTIFY.md §2.4). `apply` returns the quantity
each delta moved: what a transfer delivered, a mint created, a burn destroyed, or an `Age`
spoiled (summed over goods); 0 for the rest. A burn's is the float sum of the lots it took. The
rounding of a delta's split, merges and sum is not in that quantity; it is the delta's `Rounding`
lines (P0.7, P0.9).

### 2.5 Apply, ledger, conservation, hash, checkpoints (R2; defect 9; N2, N3, N11)

```rust
pub fn apply<E: Ext>(s: &mut SimState<E>, w: &World<E>, phase: Phase, ds: &[StateDelta<E>],
                     l: &mut Ledger) -> Result<Vec<f64>, CoreError>; // quantity moved, per delta
pub struct Tolerances { pub rel_flow: ParamId, pub rel_stock: ParamId } // Dimensionless, fixed; no absolute term
pub struct Ledger { opening: Vec<f64>, declared: Vec<f64>, gross: Vec<f64>,
                    lines: BTreeMap<(GoodId, Provenance), f64>, shortfall: Option<ShortfallLine> }
impl Ledger { pub fn open<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<Ledger, CoreError>;
              pub fn close<E: Ext>(self, s: &SimState<E>, w: &World<E>) -> Result<TickAudit, CoreError>; }
pub struct ShortfallLine { pub tick: u64, pub phase: Phase, pub holder: Holder, pub good: GoodId,
                           pub requested: f64, pub held: f64, pub prov: Option<Provenance> }
pub struct TickAudit { pub lines: Vec<(GoodId, Provenance, f64)>, pub max_margin: f64, pub max_drift: f64 }
pub struct RunLedger { /* since, next, opening, declared, gross, lines: the run's totals */ }   // P0.7
// Serialize + Deserialize: a checkpoint carries it (P0.9)
impl RunLedger { pub fn open<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<RunLedger, CoreError>;
                 pub fn close_tick<E: Ext>(&mut self, l: Ledger, s: &SimState<E>, w: &World<E>)
                     -> Result<(TickAudit, RunAudit), CoreError>;
                 pub fn since(&self) -> u64;  pub fn next(&self) -> u64;  // next, fits: P0.9
                 pub fn fits<E: Ext>(&self, w: &World<E>, tick: u64) -> Result<(), CoreError>; }
pub struct RunAudit { pub since: u64, pub lines: Vec<(GoodId, Provenance, f64)>,
                      pub drift: Vec<(GoodId, f64)>, pub max_margin: f64 }
// Breach { tick, since, good, opening, closing, declared, gross, drift, tol, lines }: since == tick
// for a tick's breach, the run's first tick for a run's
pub fn state_hash<E: Ext>(s: &SimState<E>) -> u64; // FNV-1a 64 over bincode 1 (v2p3: certify/hash.rs:16-30)
pub struct Fnv;  // incremental FNV-1a 64: new, write, finish, resume(digest) (public at S2.3)
pub struct Checkpoint<E: Ext> { world_id: u64, prefix_id: u64, state: SimState<E>,
                                run: RunLedger }                    // private; run since P0.9
impl<E: Ext> Checkpoint<E> {
    pub fn of(w: &World<E>, state: SimState<E>, run: RunLedger) -> Result<Self, CoreError>; // P0.9
    pub fn world_id(&self) -> u64;  pub fn prefix_id(&self) -> u64;  pub fn state(&self) -> &SimState<E>;
    pub fn run(&self) -> &RunLedger;                        // P0.9
    pub fn digest(&self) -> u64;             // FNV-1a of (world_id, prefix_id, state, run)
    pub fn to_bytes(&self) -> Vec<u8>;  pub fn from_bytes(b: &[u8]) -> Result<Self, CheckpointError>;
    pub fn to_ron(&self) -> String;     pub fn from_ron(s: &str) -> Result<Self, CheckpointError>; }
// CheckpointError: Magic | Format { found, expected } | Decode(String) | Digest { stored, computed }
pub mod num {
    pub fn exp(x: f64) -> f64; pub fn expm1(x: f64) -> f64; pub fn ln(x: f64) -> f64;
    pub fn ln1p(x: f64) -> f64; pub fn pow(x: f64, y: f64) -> f64;       // one libm call each (A5)
    pub fn fma(a: f64, b: f64, c: f64) -> f64;                           // a·b + c, rounded once (P1.1)
    pub fn max_qty(budget: f64, price: f64) -> Result<f64, NumError>;    // largest q: fl(price·q) <= budget
    pub fn max_scale(held: f64, coef: f64) -> Result<f64, NumError>;     // largest x: fl(coef·x) <= held
    pub fn max_remainder(total: f64, spent: f64) -> Result<f64, NumError>; // largest d: fl(spent+d) <= total
    pub fn is_clean(x: f64) -> bool;                                     // finite, sign bit clear
    pub fn two_sum(a: f64, b: f64) -> (f64, f64);                        // (s, e): a + b = s + e exactly (P0.7)
    pub enum NumError { Invalid { what, value }, Exceeded { total, spent } }
}
```

- **The `num` helpers** return the exact largest value whose rounded product or sum stays within
  the bound: they start from the quotient or difference and search the bit patterns of the
  non-negative doubles (ordered like their values), galloping to bracket the boundary and then
  bisecting, so the result fits and its `next_up` does not. Usually 2 to 4 evaluations, at most
  about 130, and no precision failure. A zero budget, holding or remainder gives exactly 0
  (nothing is bought with nothing). An infinite quotient (a tiny or zero price, a zero
  coefficient) gives `+∞`, so the quantity binds (N6). An input that is not finite or has its
  sign bit set is `NumError::Invalid`, and spending more than `total` is `NumError::Exceeded`;
  callers turn these into their own errors. Nothing in the engine panics.
- **Undefined ids (N2).** Every good, node, holder and param is checked against `World` before
  anything moves: `UnknownGood`, `UnknownHolder` or `UnknownParam` in every profile, never a
  `debug_assert` (July's defect, `v2p3: certify/ledger.rs:99-108, 160-181`).
- **Signs and finiteness.** Every quantity, price, EMA, volume and param value is finite and has a
  clear sign bit, or `apply` rejects it. No state-path value is `-0.0`, so `min` and `max` never
  meet a signed-zero tie.
- **Shortfalls (N3).** A failed `take` in a `Transfer` or `Burn` writes its `ShortfallLine` to the
  ledger and returns `CoreError::Shortfall(line)`. Nothing of that delta applies and the tick ends.
  There is no partial burn: `Burn { amount: All }` means "whatever is there". `apply` drops an
  escrow once it is empty, and `close` fails if one remains.
- **Close.** `closing` is a walk of every holder. Per good, `drift = (closing − opening) − declared`
  and `tol = rel_flow·gross + rel_stock·max(|opening|, |closing|)`, with both tolerances read from
  the current params. `drift.abs() <= tol` must hold, and NaN fails it; a failure is
  `CoreError::Conservation { good, drift, tol, lines }`. With no stock and no flow, `tol = 0` and
  the drift must be exactly zero. The margin is 0 when `drift == 0` and `|drift|/tol` otherwise, so
  `max_margin <= 1` is the pass condition and 0/0 never occurs. July's `ABS_TOLERANCE` does not move
  (A12). A tolerance must be below 1, checked at load: at 1 or more a leak of the whole stock or
  flow would pass (amended at P0.6). `declared` includes the `Rounding` lines (§2.4), so what lots
  create or destroy is declared exactly (amended at P0.9, O5), and the drift is only the rounding of
  the walk and of the declared fold. `gross` is the quantity moved plus the magnitude of every
  declaration, exactly (pinned at P0.9, O6). The walk is a float sum, so it cannot see a unit below
  half an ulp of the good's total; that is why the tolerance remains, and why rounding is declared
  by `apply` rather than found by the walk (amended at P0.7).

- **The run's ledger** (amended at P0.7). A leak below each tick's tolerance, every tick, would
  pass every tick and add up. `RunLedger::close_tick` closes the tick's ledger (the tick's check
  first), folds its declared quantities, gross flows and lines into run totals, and checks the
  run as if it were one tick: `drift = (closing − opening) − declared` from the run's first open
  to this close, against `rel_flow·gross + rel_stock·max(|opening|, |closing|)` with the run's
  gross flow. Each tick opens on the state the last closed on, so the walks' rounding
  telescopes and the run's drift stays the size of one tick's; a leak grows with the run. A
  run breach is `CoreError::Conservation` with `since` the run's first tick, and stops the run
  like a tick's. A ledger of any tick but the next is a shape error. A checkpoint carries the
  run's ledger inside its digest, and a resumed run continues it, so a leak stops a run resumed at
  any cadence at the tick where it stops the uninterrupted run (amended at P0.9, O8; before, a
  resumed run's ledger opened at the checkpoint's tick, and a leak of 1e-9 a tick that stopped an
  uninterrupted run at its fourth tick passed a run resumed every 2 ticks).
- **Income.** Phase 0 asserts the market-level identity: in every market and tick, what the
  sellers received equals what the buyers paid, and the buyers paid the posted price times what
  they received, both under the gate test's relative bar (`gate_settles_by_one_filled_quantity`).
  The income identity proper, `I = wN_a + rT`, needs wages and rents, which arrive in Phase 2; it
  can then be checked from the (good, provenance) lines, the settlement lines with their moved
  quantities (§3.2) and the audit, which always reaches the caller (§7).
- **Hash.** It covers the whole `SimState`: tick, params, market book, every holding with its lot
  lives, and the extension state. It does not cover the tape.
- **Checkpoints (N11).** A checkpoint carries `world_id` and `prefix_id` (§2.6), and `Sim::resume`
  refuses a mismatch (§7.6); since P0.9 it also carries the run's ledger (O8). The bytes form opens
  with the magic `RUSTYECK` and the format, 3, as a little-endian `u32`, then bincode 1 of
  `(world_id, prefix_id, digest, state, run)` with no trailing bytes; the RON form's first field is
  `format`, read by a probe that skips the rest undecoded, then `world_id`, `prefix_id`, `digest`,
  `state` and `run`. Either way a wrong format (formats 1 and 2 included) is refused before the
  state is decoded. Decoding re-sorts and re-coalesces lots and rejects NaN, `-0.0` and negative
  values (and non-positive prices, and a run's ledger whose columns differ in length or whose lines
  are out of order); the digest is FNV-1a 64 over bincode 1 of `(world_id, prefix_id, state, run)`,
  and the decoded fields must hash to it, or the decoder returns `CheckpointError::Digest`. So a
  checkpoint whose identity, state or run's ledger was edited or corrupted after it was saved is
  refused (R2; amended at P0.9, O7: format 2's digest covered the state alone, and a `prefix_id` or
  `world_id` edited to the value a refused resume prints let the checkpoint resume).
  `Checkpoint::validate(&world)` then checks every id and shape of the state against the `World`
  (`SimState::validate`) and that the run's ledger fits it (`RunLedger::fits`: its goods, and its
  next tick is the state's). The fields are private: a `Checkpoint` is made only by
  `Checkpoint::of(&world, state, run)`, which records `world_id` and `prefix_id(tick)` and refuses a
  run's ledger that does not fit, and by the decoders. `to_bytes` and `to_ron` cannot fail for
  core's types; should an extension's `Serialize` fail, they return output the loaders reject
  (amended at P0.6).

### 2.6 The tape: schema, resolution, identity (N1, N2, N10 in part; E8)

```rust
pub struct Tape<E: Ext> {                   // the editable document; every raw type is pub,
    pub schema: u32,                        // Serialize + Deserialize, #[serde(deny_unknown_fields)]
    pub header: RawHeader, pub params: Vec<RawParam>, pub goods: Vec<RawGood>,
    pub nodes: Vec<RawNode>, pub channels: Vec<RawChannel>, pub classes: Vec<Key>,
    pub actors: Vec<RawActorEntry<E::RawActor>>, pub genesis: RawGenesis,
    pub events: Vec<RawEvent<E::RawAction>>, pub recurring: Vec<RawRecurring<E::RawAction>> }
impl<E: Ext> Tape<E> { pub fn from_ron(s: &str) -> Result<Self, LoadError>; pub fn to_ron(&self) -> String; }
pub fn resolve<E: Ext>(t: &Tape<E>) -> Result<(World<E>, SimState<E>), LoadError>;
pub struct World<E: Ext> { pub name: String, pub world_id: u64, pub clock: Clock, pub registry: Registry,
    pub tol: Tolerances, pub market: MarketConfig, pub goods: Vec<GoodDef>, pub nodes: Vec<NodeDef>,
    pub channels: Vec<ChannelDef>, pub classes: Vec<Key>, pub actors: Vec<ActorDecl<E::Actor>>,
    pub schedule: Schedule<E>, keys: KeyIndex }
impl<E: Ext> World<E> { pub fn prefix_id(&self, tick: u64) -> u64; /* id_of, key_of per kind */ }
pub struct ActorDecl<A> { pub id: ActorId, pub key: Key, pub class: ClassId, pub home: NodeId, pub spec: A }
pub struct MarketConfig { pub rule: PriceRule, pub one_sided: OneSided, pub ema_time_constant: Site } // Weight
pub struct Tolerances { pub rel_flow: Site, pub rel_stock: Site }  // Value sites, fixed (S2.2)
pub enum PriceRule { Imbalance, Ratio }     // N13: required, no Default
pub enum OneSided { Saturate, Hold }        // F8: required, no Default
pub struct Firing<E: Ext> { pub tick: u64, pub day: i64,   // day: Date::days of its date (P0.9)
                           pub event: EventId, pub occurrence: u32, pub action: StateDelta<E>,
                           pub source: Option<Key> }  // a SetParam's source param, for its basis
pub struct ScheduleParam { pub key: Key, pub unit: Unit, pub value: f64, pub basis: Basis,
                           pub sites: Vec<ParamSite> }  // a source's target methods; a period's Ticks
pub struct Schedule<E: Ext> { once: Vec<Firing<E>>, every: Vec<Recurring<E>>,
                             params: Vec<ScheduleParam>, clock: Clock }  // clock: P0.9
impl<E: Ext> Schedule<E> { pub fn fire(&self, tick: u64) -> Vec<Firing<E>>;
                           pub fn params(&self) -> &[ScheduleParam]; pub fn param(&self, key: &str) -> Option<&ScheduleParam>; }
```

- **Schema version.** `schema: 1`. The loader reads only its own version; anything else is
  `LoadErrorKind::Schema`, found by a probe of `schema` alone before the rest is parsed. Every
  schema change bumps the number, since there are no defaults to absorb one.
- **No silent fields.** Unknown fields are rejected (`deny_unknown_fields`), and nothing carries
  `#[serde(default)]`. An `Option` field must be written as `None` or `Some(..)`: serde would
  otherwise read a missing one as `None`, so each raw `Option` field uses a `deserialize_with`
  helper, which makes serde treat it as required (probed on serde 1.0.228 and ron 0.8.1). The
  helper is public, `core::tape::raw::required`, for the extension's raw types.
- **Documented.** `core::tape::raw` carries `#![deny(missing_docs)]`. Each field's rustdoc gives
  its type, its unit (or "dimensionless"), that it is required, and that it has no default.
  `docs/TAPE.md` (written in P0.3) points to that rustdoc, keeps the schema-version history, and
  uses the gate tape as its worked example.
- **Keys and order (E8).** Keyed data is always a list of entries carrying a `key`, never a RON
  map, because serde keeps the last of two equal map keys without complaint. Lists can be in any
  order; duplicates are load errors. The loader reads each list into canonical order:
  - goods, nodes, channels, classes, params, actors (per kind), events and recurring entries by key;
  - genesis prices by (node, good), genesis holdings by (holder, good);
  - recipe inputs and outputs by good, buy and sell lines by (node, good), payout recipients by actor.

  `to_ron` writes that canonical form (comments are not kept).
- **World identity.** `world_id` is FNV-1a 64 over bincode 1 of the World's run content together
  with the genesis `SimState`. The run content is everything except the schedule, the tape's
  `name` and the basis texts, none of which changes a number the run computes. Reformatting,
  comments, CRLF line endings, list order and a `to_ron` round trip therefore keep `world_id`. Any
  edit to a number, a unit, a key or the structure changes it, except the schedule's. It is also
  the identifier N10's manifest needs in session 2. Each param enters as (key, unit, genesis
  value): not its `fixed` flag, which the schedule sets when a `SetParam` copies it, and not its
  `sites` (amended at S2.2, D10 items 1 and 4). The structure a fixed use makes is hashed where
  it lands: a shelf life as ticks in the goods, the tolerances in `tol`, a period and a copied
  value in the schedule, under `prefix_id`. So a registered param that becomes a new event's
  source keeps the world (`new_source_event_keeps_world_id`). A `Site` held by the world or a
  spec is hashed with it, since its method decides a number.
- **Schedule params** (amended at P0.6). A param whose every reference is the schedule's (the
  `to` of a `SetParam`, the `every` of a recurring entry) belongs to the schedule: the loader
  copies its value into each firing or turns it into a period, and keeps it, with its unit and
  basis, as a `ScheduleParam`. It is not registered, has no `ParamId`, is not in the state and is
  not in `world_id`. The firings it shapes are in `prefix_id`. So adding a dial and a dated
  `SetParam` that copies it, or changing such a param's value or a period, keeps the world and
  every checkpoint taken before the first firing that changes. A param the world also reads
  stays registered, and is fixed. Since the extension's specs reference params too, the loader
  learns which params are the schedule's alone in a first pass, with every param registered,
  and then resolves again with those params moved out; the first pass reports every load
  error.
- **Prefix identity.** `prefix_id(t)` is FNV-1a 64 over bincode 1 of every firing with `tick < t`,
  in firing order, each as (tick, event key, occurrence, resolved action), with recurring entries
  expanded. A checkpoint of the state whose tick is t stores `world_id` and `prefix_id(t)`.
- **Sorting (N1).** Dates become ticks (§6). `once` is sorted by (tick, day, key), and `fire`
  returns its `partition_point` range merged with the recurring firings due that tick, in (day,
  key) order, where a dated event's day is its date and a recurring occurrence's is the first day
  of its tick (`Clock::first_day`). So two dated events in one tick fire in date order, as they
  would in two shorter ticks, and events of one day fire in key order (amended at P0.9, O9: a
  firing kept no date, and at 12 ticks a year a cut and a restore twenty days apart fired in key
  order, which left the mine cut for nine years). July binary-searched an unsorted list (`v2p3:
  scenario/mod.rs:14, 25-42`).
- **Recurring entries** are `(key, first: Date, every: Years param, last: Option<Date>, act)`,
  firing at `first_tick + k·period`. A period that rounds to 0 ticks is a load error, so `every =
  0` cannot load.
- **Actions.** Each resolves to one of:
  - `Mint` or `Burn` (provenance `Event`);
  - `Transfer`;
  - `SetParam { param, to }`, where `to` is another param's key with the same unit. The value
    comes from that param at load, and its key stays on the firing, so the new value keeps a
    basis (R4). The target must not be fixed. A source nothing else reads is a schedule param.
  - `Actor(E::RawAction)`;
  - `ScalePrice { node, good, by }` (amended at S2.3), a dated price shock: the posted price times
    `by`, a `Dimensionless` param the schedule reads (a `Value` site at `act.by`; the firing's
    `source`). Only a dated event may carry it.
- **Resolver.** The extension resolves through `Resolver`: `good`, `node`, `class`, `actor`,
  `channel` (key to id), `param(key, method, ParamUse::Live | Fixed, field) -> Site` (amended at
  S2.2: the `ClockMethod` replaces the unit, which it implies, and the use is recorded as a
  `ParamSite` at the field's path), `ticks(key, field)` (a fixed `Years` param as whole ticks, a
  `Ticks` site), `value(p, field)`, `quantity(v, field)`, `is_currency(g)`, `tick_of(date,
  field)`, `clock()`, and `enter(segment)`, `leave()` and `error(field, kind)` for paths. A
  `SetParam`'s source is given its target's methods at the event's `to` once every use of the
  target is known; a recurring period is a `Ticks` site at its `every`.
- **Load errors (N2)** occur in every profile, and each names its tape path, for example
  `actors[mill].spec.buy[village/grain].qty`. The RON parser's own errors (syntax, an unknown or
  missing field, a malformed key or date) carry its line and column instead. They are:
  - a wrong schema version, an unknown field or a missing field;
  - an invalid, unknown or duplicate key;
  - a missing or duplicate genesis price, or a genesis price that is not finite and > 0;
  - a unit mismatch, an unreferenced param, or a `SetParam` on a fixed param or across units;
  - an event before `start`;
  - an order line on a currency; a currency that is not `Indefinite` or has a `price_rate`; a
    non-currency good without a `price_rate`;
  - a good on both sides of a recipe;
  - a life or period that rounds to 0 ticks;
  - `Ratio` with `Saturate`;
  - buy or payout weights whose left fold in canonical order is not exactly 1.0 (§4);
  - an actor `home` that is not a node;
  - `ticks_per_year: 0`, a channel from a node to itself, a genesis price on a currency, a
    recurring `last` before its `first`, a duplicate genesis holding or held good;
  - a ledger tolerance of 1 or more; a currency on either side of a recipe (§4) (both amended
    at P0.6);
  - a `ScalePrice` on a currency (`NoMarket`), with a factor that is not finite and positive, or
    in a recurring entry (`RecurringPriceShock`) (amended at S2.3).

## 3. rustyecon-markets (N5–N8, N13; R12; F8)

### 3.1 Orders, admission and clearing: cash binds at the order (N7)

```rust
pub struct Order { pub actor: ActorId, pub class: ClassId, pub node: NodeId, pub good: GoodId,
                   pub qty: f64, pub side: Side }
pub enum Side { Buy { budget: f64 }, Sell }         // budget in the node's currency
pub enum SideTag { Buy, Sell }                      // Side without the budget; Buy < Sell
#[non_exhaustive] pub struct Line { pub order: Order, pub feasible: f64 }  // made by admit only
pub fn admit<E: Ext>(orders: Vec<Order>, s: &SimState<E>, w: &World<E>) -> Result<Vec<Line>, OrderError>;
pub enum OrderError { UnknownActor(ActorId), UnknownNode(NodeId), UnknownGood(GoodId),
    NoMarket { node, good }, WrongClass { actor, class, registered }, BadValue { order, what, value },
    Duplicate { actor, node, good, side }, OverBudget { order, currency, remaining },
    OverPosted { order, remaining }, UnknownMarket { node, good }, Moved { deltas, moved },
    Core(CoreError), Num(NumError) }
```

- **Feasible quantity.** A buy's `feasible = min(qty, num::max_qty(budget, price)?)`. No price
  guard applies (N6): when `budget/price = ∞`, `qty` binds.
- **Admission.** Orders are first sorted by (actor, node, good, side), so their input order is
  irrelevant. Each failure is an `OrderError`:
  - Per actor and currency, `rem` starts at `get(currency)`. Each budget, in that order, must be
    at most `rem`, and then `rem = rem − budget`. This is the same left-to-right subtraction the
    single currency lot performs at settlement, so a payment that fits here fits there
    (`OverBudget`).
  - The sells of one good are checked cumulatively across nodes against `get(good)` in the same
    way (`OverPosted`). Both checks run the inventory's own takes, in settlement's order, on a
    copy of the actor's holding: for a currency that is the subtraction above, and for a good
    held in several lots it is the lot-by-lot arithmetic settlement will perform (amendment 3).
  - One order per (actor, node, good, side), checked before zero-quantity orders are dropped.
  - The order names a declared actor, a node and a non-currency good, and its class is the
    actor's registered class.
  - Values are finite with a clear sign bit, and zero-quantity orders are dropped; a dropped
    order binds no cash.
- Budgets bind here and nowhere else. Settlement never cuts a buyer, so the volumes the price
  reads are all demand that can pay (N7).
- **Phase 0 trades across nodes.** An actor may post at any node, and nothing crosses a channel or
  pays a crossing cost. The simplification lasts until transport desks exist (§13).
- **Clearing** (salvaged, `v2p3: systems/clearing/mod.rs:15-56`). Per market in (node, good)
  order, `S` is the sum of sells and `D` the sum of feasible buys, both left folds in canonical
  line order. `buyer_fill = if D > 0 { (S/D).min(1.0) } else { 0.0 }`, and `seller_fill` mirrors
  it. Every (node, non-currency good) gets `SetVolumes { supply: S, demand: D }`, with 0 and 0
  where nothing was posted. A line for a market the world does not have is `UnknownMarket`.

### 3.2 Settlement: one fill, both sides, through an escrow

```rust
pub fn clear<E: Ext>(lines: &[Line], w: &World<E>) -> Result<(Vec<StateDelta<E>>, Fills), OrderError>;
pub struct Fills { /* one MarketFill per market, (node, good) order */ }  // markets(), get(node, good)
pub struct MarketFill { pub node: NodeId, pub good: GoodId, pub supply: f64, pub demand: f64,
                        pub buyer_fill: f64, pub seller_fill: f64 }      // trades(): S, D, both fills > 0
pub fn settle<E: Ext>(lines: &[Line], f: &Fills, s: &SimState<E>, w: &World<E>)
    -> Result<SettlePlan<E>, OrderError>;
impl<E: Ext> SettlePlan<E> { pub fn deltas(&self) -> &[StateDelta<E>];
    pub fn realize(self, moved: &[f64]) -> Result<(Vec<SettleLine>, Vec<RationLine>), OrderError>; }
pub struct SettleLine { pub actor: ActorId, pub class: ClassId, pub node: NodeId, pub good: GoodId,
                        pub side: SideTag, pub qty: f64, pub value: f64 }       // moved, one per order
pub struct RationLine { pub node: NodeId, pub good: GoodId, pub class: ClassId, pub side: SideTag,
                        pub requested: f64, pub feasible: f64, pub filled: f64 } // R12
pub fn update_prices<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<Vec<StateDelta<E>>, PriceError>; // §3.3
```

For each market that trades (`S > 0`, `D > 0` and both fills positive; amendment 5), in (node,
good) order, with posted price `p`, currency `c` and escrow `X = Holder::Escrow(node, good)`:

1. Each seller, in actor order, transfers `ship_i = qty_i·seller_fill` of the good to `X`.
2. Each buyer, in actor order, transfers `pay_j = p·r_j` of `c` to `X`, where `r_j =
   feasible_j·buyer_fill` is its one filled quantity.
3. `X` transfers `r_j` of the good to each buyer. One buyer goes last and takes `All`: the one
   with the largest `r_j`, compared by `f64::total_cmp` and then by `ActorId`, so among equal
   quantities the highest id goes last. Every other buyer takes `Qty(r_j)`, in actor order, first.
4. `X` transfers `p·ship_i` of `c` to each seller the same way. The largest shipper, chosen by the
   same order, goes last and takes `All`.

**Why no step falls short.** Every fill is at most 1 and rounding is monotone, so `ship_i <=
qty_i` and `pay_j <= budget_j`. A buyer pays in the order its budgets were admitted, and receipts
only add. The largest taker goes last, so no earlier take can exceed the escrow, and every escrow
ends empty. Payments are nominal. A quantity can differ from nominal by rounding in two
places: the last taker's `All`, and any take from a holding of several lots, which moves the sum
of the lots it takes (a seller's bread, a bread escrow). So `ship_i <= qty_i` and `filled <=
feasible` hold only up to that rounding, a few ulps, and a consumer of R12's lines must not treat
them as exact invariants (amended at P0.6). The argument assumes normal numbers; in the subnormal
range a take could still fall short, which stops the run with its ledger line, or one side could
round to nothing (amendment 6). A transfer whose nominal quantity is exactly 0 is not planned,
and its line reports 0. Every split and merge that rounds in these transfers, a payment out of a
buyer's cash or a receipt into a seller's, is a `Rounding` line in the tick's ledger (§2.4;
amended at P0.7); settlement declares nothing else.

**Moved quantities.** `apply` returns what each delta moved, and `realize` turns the plan into
`SettleLine`s and `RationLine`s from those actual quantities. A buyer's line has `qty` = the good
received and `value` = the currency paid; a seller's has `qty` = shipped and `value` = received.

**Gone:** balance state and forgiveness (N8, `v2p3: transactions/mod.rs:396-406`), and the
absolute epsilons (`:125, 190, 357-433`). Soonest-expiring-first out of the escrow gives the oldest
lots to the lowest id (open question 4).

There is one `RationLine` per (node, good, class, side) for each market that had an order. For a
buyer class, the cash shortfall is `requested − feasible` and the market shortfall is `feasible −
filled`. For a seller class, `requested = feasible` = offered, and `filled` = shipped.
`SettleLine`s come one per admitted line in (actor, node, good, side) order, and `RationLine`s in
(node, good, class, side) order, each sum a left fold in actor order.

### 3.3 Price update (N5, N13; F8; A13)

For each non-currency market, with `k`, `w` and the one-sided rule read from the current params:

- **Two-sided or idle.** `x = if S == 0 && D == 0 { 0 } else { (D − S)/S.max(D) }`.
- **`Imbalance`.** `p' = p·num::exp(k·x)`, with `k = clock.log_step(price_rate)`. This replaces
  July's `p·(1 + α·x)`; the two differ only at second order in `α·x`, and only this form is
  tick-invariant (open question 1).
- **One-sided markets** (exactly one of S and D is zero) follow the tape's `one_sided` field. July
  treated them by formula; F8 asked for a decision.
  - `Saturate`: `x = ±1`, as in July. An idle one-sided market's price then moves by `exp(±k)`
    every tick until it leaves the finite positive range, which is a `PriceError`. At `k = 0.1` a
    tick that takes about 7,450 ticks. Session 2's runaway detector watches for it.
  - `Hold`: `p' = p`. A one-sided market carries no price evidence.
- **`Ratio`.** `p' = p·(D/S)` when both are positive (July's order of operations), else `p' =
  p`. `Ratio` with `Saturate` does not load.
- **EMA.** `ema' = w·p + (1 − w)·ema`, with `w = clock.weight(ema_time_constant)` read at use time
  and `p` this tick's posted price.
- **Changes and errors.** `SetPrice` and `SetEma` are emitted only when the bits change. That is an
  exact test, not a tolerance; July's `1e-12` gates were N5's floor. A non-finite or non-positive
  price is `PriceError::NonFinite { node, good, from, to }` and stops the run, and an EMA likewise
  `NonFiniteEma`; a param read that fails is `PriceError::Core`. Nothing is emitted on an error.
  There is no floor, ceiling or fallback; N13's fallback, `v2p3: clearing/mod.rs:124-132`, goes.
- **Public rules.** `imbalance(S, D)`, `step(p, k, x) = p·exp(k·x)` and `next_price(rule,
  one_sided, p, k, S, D)` are the pure functions `update_prices` applies to each market.

## 4. rustyecon-agents: the seam and the scripted actor

```rust
pub struct View<'a, S> { pub tick: u64, pub clock: &'a Clock, pub me: ActorId, pub class: ClassId,
    pub home: NodeId, pub currency: GoodId,   // the home node's currency (amended at P0.5)
    pub own: &'a Inventory, pub own_state: &'a S,
    pub posted: Posted<'a>,            // price and ema per (node, good), read-only
    pub params: Params<'a> }           // current values, typed by unit
pub trait Behaviour: Send + Sync {     // Phase 2 implements it for new kinds
    type Own;                          // the actor's own extension state
    fn decide(&self, v: &View<Self::Own>) -> Result<Decision, AgentError>;                  // phase 1
    fn produce(&self, v: &View<Self::Own>) -> Result<Vec<StateDelta<Agents>>, AgentError>;  // phase 4
    fn upkeep(&self, v: &View<Self::Own>) -> Result<Vec<StateDelta<Agents>>, AgentError>; } // phase 5b
pub enum Hook { Decide, Produce, Upkeep }
pub enum AgentError { Core(CoreError), Num(NumError), Take(TakeError), Mismatch(ActorId) }
pub struct Decision { pub orders: Vec<Order>, pub deltas: Vec<StateDelta<Agents>> }
pub struct Agents;  // impl Ext: State = BTreeMap<ActorId, ActorState>, Delta = AgentDelta,
                    //           RawActor = RawSpec, Actor = Spec, RawAction = RawAgentAction
pub enum ActorState { Scripted(ScriptState) }   // one variant per behaviour kind
pub struct ScriptState { pub active: bool }
pub enum AgentDelta { SetActive { actor: ActorId, active: bool } } // owner: Some(actor)
pub enum RawAgentAction { SetActive { actor: Key, active: bool } } // Actor(SetActive(..)) on the tape
pub struct Cast { /* one Behaviour per actor, by kind, built from World.actors */ }
impl Cast { pub fn new(w: &World<Agents>) -> Result<Cast, LoadError>;
    pub fn decide(&self, a: ActorId, s: &SimState<Agents>, w: &World<Agents>) -> Result<Decision, AgentError>;
    /* produce and upkeep alike */ }
```

`Cast` dispatches each actor to its kind's `Behaviour`, and checks that its `ActorState` variant
matches (`AgentError::Mismatch` otherwise). Since P2.0.1 the kinds are the scripted actor and the
four Appendix B roles (`Provider`, `Workers`, `GoodDesk`, `MachDesk`; docs/probe/RULES.md), whose
state a `SetState` delta replaces. `Cast::new` makes the load checks that need the
whole world: every buy line's node quotes in the actor's home currency, and no payout names the
payer; each is a `LoadError` with its tape path. R13 holds by construction: a `View` cannot reach
another actor's holdings, orders or state, nor the cleared volumes, and agents never depends on
the oracle. The engine enforces the rest (§7.3).

The scripted spec (`RawSpec::Scripted(RawScript)`, documented field by field in the rustdoc of
`rustyecon_agents::spec`) has `active`, `recipe: Option<(inputs, outputs, capacity)>`, `buy`
lines `(node, good, qty: FlowPerYear param, weight)`, `sell` lines `(node, good, qty:
Flow(FlowPerYear param) | AllHeld)`, `spend: Option<RatePerYear param>` (`Some` exactly when there
are buy lines) and `payout: Option<(rate: RatePerYear param, to: [(actor, weight)])>`.
Coefficients and weights are finite and positive, no recipe names a currency (R14; amended at
P0.6), and a payout never names the payer.

The scripted actor (either kind) reads its lines' quantities and rates from the current params,
so a dated `SetParam` retargets it. It does nothing while `active` is false.

Resolved specs hold each param they read as a `Site` (amended at S2.2, D10 item 4), and every
rule reads it through `Site::per_tick`. The scripted actor's capacity, buy and `Flow` sell
quantities are `Flow` sites, and its payout and spend rates `Share` sites. Each role site names
the method its rule applies (docs/probe/RULES.md §2): the step rule's `up` and `down` are
`LogStep`, every other rate a `Share`, flows `Flow` and coefficients and dials `Value`. The cash
rule's `share(v·μ^κ)` tilts the annual turnover and converts it with the turnover site's own
method (`Site::value`, `Site::convert`). `Spec::sites(&World)` lists a spec's sites with their
paths under `actors[key].spec`.

- **`decide`.** Two steps, payouts first:
  - **Payouts.** `P = fl(share(payout)·cash)`, split by recipient weights in `ActorId` order. Each
    recipient but the last gets `fl(w·P)`, and the last gets `num::max_remainder(P, Σ earlier)`.
    Each part is capped by the actor's own `rem`, walked by subtraction as admission walks it. The
    payouts are transfers from `me`.
  - **Buy lines**, in (node, good) order, post `qty = flow(line)`. The spending total is `B =
    fl(share(spend)·cash_after)`, where `cash_after` is the cash less the payouts, subtracted the
    way the inventory will subtract them. `rem` walks down from `cash_after` the same way.
    Each non-last line gets `min(fl(w·B), rem)`. The last gets `min(max_remainder(B, Σ earlier),
    rem)`.

  A valid tape can therefore never produce `OverBudget`, even when `share` rounds to exactly 1.
  Last, each sell line, in (node, good) order, posts `min(flow(line), left)`, or `left` for
  `AllHeld`, where `left` is what a copy of the holding still holds after the earlier sell lines,
  taken as admission takes it. Scripted actors move only currency in phase 1, so admission sees
  the same holding of every good, and a valid scripted tape never produces `OverPosted` either.
- **`produce`** is Leontief: `x = min(flow(capacity), min_k max_scale(held_k, a_k))`. It burns
  `a_k·x` of each input in good order (`Production` for a Desk, `Consumption` for a Pop). It then
  mints `o_l·x` of each output in good order, as `Production`, or as `Endowment` when the recipe
  has no inputs. A recipe with no outputs is consumption. `max_scale` means no burn falls short,
  which is what July's shared-input reservation was for (`v2p3:
  systems/production/mod.rs:34-77`).
- **`upkeep`** does nothing in Phase 0.

## 5. The tape runtime format (RON, schema 1)

```ron
Tape(
  schema: 1,
  header: (name: "gate", start: "1750-01-01", ticks_per_year: 52,
    market: (rule: Imbalance, one_sided: Hold, ema_time_constant: "price.ema_tc"),
    ledger: (rel_flow: "ledger.rel_flow", rel_stock: "ledger.rel_stock")),
  params: [   // one entry per dial; five shown
    (key: "ledger.rel_flow", value: 1e-12, unit: Dimensionless, basis: Assumed("July REL_FLOW")),
    (key: "price.ema_tc", value: 0.5, unit: Years, basis: Approximate("≈ July 2/53 at 52 ticks/yr")),
    (key: "rate.grain", value: 5.2, unit: RatePerYear, basis: Assumed("July alpha 0.1 per weekly tick")),
    (key: "life.bread", value: 0.0577, unit: Years, basis: Assumed("three weeks")),
    (key: "mine.capacity", value: 520.0, unit: FlowPerYear, basis: Assumed("gate world")), …],
  goods: [(key: "bread", life: Years("life.bread"), price_rate: Some("rate.bread")),
          (key: "coin", life: Indefinite, price_rate: None),
          (key: "grain", life: Indefinite, price_rate: Some("rate.grain")), /* fuel likewise */],
  nodes: [(key: "town", currency: "coin"), (key: "village", currency: "coin")],
  channels: [],
  classes: ["households", "pensioners", "producers"],
  actors: [(key: "mill", kind: Desk, class: "producers", home: "town", basis: Assumed("gate world"),
    spec: Scripted((active: true,
      recipe: Some((inputs: [("grain", 2.0), ("fuel", 1.0)], outputs: [("bread", 3.0)], capacity: "mill.capacity")),
      buy: [(node: "village", good: "grain", qty: "mill.buy.grain", weight: 0.6),
            (node: "town", good: "fuel", qty: "mill.buy.fuel", weight: 0.4)],
      sell: [(node: "town", good: "bread", qty: AllHeld)],
      spend: "mill.spend", payout: Some((rate: "mill.payout", to: [("workers", 1.0)]))))), …],
  genesis: (basis: Assumed("gate world"),
    prices: [(node: "town", good: "bread", price: 2.0), (node: "village", good: "grain", price: 1.0), …],
    holdings: [(holder: "mill", goods: [("coin", 100.0), ("grain", 16.0)]), …]),
  events: [   // any order: the loader sorts by (tick, date, key)
    (key: "mine.restored", at: "1770-06-15", basis: Assumed("gate world"),
     act: SetParam(param: "mine.capacity", to: "mine.capacity.base")),
    (key: "mine.cut", at: "1760-03-01", basis: Assumed("gate world"),
     act: SetParam(param: "mine.capacity", to: "mine.capacity.cut")),
    (key: "oven.opens", at: "1768-04-01", basis: Assumed("gate world"),
     act: Actor(SetActive(actor: "oven", active: true))),
    (key: "bread.line.up", at: "1765-09-01", basis: Assumed("gate world"),
     act: SetParam(param: "workers.buy.bread", to: "workers.buy.bread.high"))],
  recurring: [(key: "pension", first: "1751-01-01", every: "pension.period", last: None,
               basis: Assumed("gate world"), act: Mint(holder: "pensioners", good: "coin", qty: 5.0))],
)
```

**Registration (R4).** Every dial with a time unit, every behavioural rate and every tolerance is
a named entry in `params`, referenced by key. Dimensionless structural data may sit inline under
its entry's `basis`: recipe coefficients, weights, genesis stocks and prices, and event
quantities (open question 3). `spec: Scripted(...)` is an enum, so Phase 2's kinds are new
variants, not a new schema shape. `rustyecon registry <tape>` prints both kinds of number, each
param with its unit, its use (`live`, `fixed`, or `schedule` for a param only the schedule reads),
each site where the run reads it, with that site's method and per-tick value (§6; amended at
S2.2, D10 item 4), and its basis.

## 6. Time (A13)

`ticks_per_year` is a `u32 >= 1` in the header, and Δ = 1/ticks_per_year. All conversion happens
in `Clock`, and nothing else divides by `ticks_per_year`.

| Unit | Used as | `Clock` method (`ClockMethod`, S2.2) | Per tick |
|---|---|---|---|
| `Dimensionless` | a ratio, weight or tolerance | none (`Value`) | `v` |
| `FlowPerYear` | a flow of goods (capacity, order line) | `flow` (`Flow`) | `v·Δ` |
| `RatePerYear` | a continuous draw on a stock (spend, payout) | `share` (`Share`) | `−expm1(−v·Δ)` |
| `RatePerYear` | a price rate (`Imbalance`), a step rule's scale step | `log_step` (`LogStep`) | `v·Δ` |
| `CompoundPerYear` | an effective annual rate ρ | `compound` (`Compound`) | `expm1(Δ·ln1p(ρ))`, i.e. `(1+ρ)^Δ − 1` |
| `FractionPerYear` | an annual fraction δ (e.g. depreciation) | `fraction` (`Fraction`) | `−expm1(Δ·ln1p(−δ))`, i.e. `1 − (1−δ)^Δ` |
| `Years` | an EMA time constant τ | `weight` (`Weight`) | `−expm1(−Δ/τ)` |
| `Years` | a period or shelf life (fixed) | `ticks` (`Ticks`) | `round(v·tpy)` ticks; 0 is a load error |
| date `YYYY-MM-DD` | an event time | `tick_of` | `floor(days·tpy·10⁴ / 3,652,425)`, in integers |
| a tick | its date | `date_of` | `start + ceil(t·3,652,425 / (tpy·10⁴))` days, in integers |
| a tick | its first day | `first_day` | the day of `date_of`, as `Date::days` counts, saturating; dates a recurring occurrence (P0.9) |

The row a param takes is decided by its use, not its unit: two rows share `RatePerYear` and two
share `Years`. Since S2.2 each use carries its row as a `ClockMethod` (§2.1), recorded at its
path, and `ClockMethod::per_tick` is the one dispatch from a method to its `Clock` conversion.
No shipped code outside `clock.rs` calls a conversion or reads a param by type
(`params_are_read_only_through_sites`).

Dates are proleptic Gregorian, and the year is the mean Gregorian year. Days come from the integer
`days_from_civil`, so the date-to-tick map uses no floating point. `tick_of(date_of(t)) == t`
whenever `ticks_per_year <= 365`.

Changing only `ticks_per_year` leaves these invariant:
- annual endowment output;
- a share's decay `exp(−v·t)`;
- the EMA's step response `exp(−t/τ)`;
- the `Imbalance` path `exp(r·x·t)` under a constant `x`;
- the annual values of compound and fractional rates;
- event dates, to within one tick, and the order of any two dated events (P0.9).

`Ratio` is a one-step rule and is not invariant; that is recorded, not fixed.

The oracle solves per period. Its adapter either takes the tick as its period or restates its
inputs through `Clock`: `compound(ρ)`, `fraction(δ)`, and `J_b` via `ticks`. These methods exist
now, so the oracle need not wait for Phase 3.

## 7. rustyecon-engine: the Sim, the tick and the observation

### 7.1 The Sim

```rust
pub type Tape = rustyecon_core::Tape<Agents>;  pub type World = rustyecon_core::World<Agents>;
pub type Checkpoint = rustyecon_core::Checkpoint<Agents>;
pub struct Sim { world: World, state: SimState<Agents>, cast: Cast, status: Status,
                 hash: u64, last: Option<TickReport> }            // every field private
impl Sim {
    pub fn new(tape: &Tape) -> Result<Sim, LoadError>;
    pub fn resume(tape: &Tape, cp: &Checkpoint) -> Result<Sim, ResumeError>;          // §7.6
    pub fn step(&mut self) -> Result<TickReport, RunError>;
    pub fn step_traced(&mut self) -> Result<(TickReport, Trace), RunError>; // every applied delta,
                                                                            // with phase and moved qty
    pub fn run_until(&mut self, until: u64, on_tick: &mut dyn FnMut(&TickReport)) -> Result<(), RunError>;
    pub fn checkpoint(&self) -> Result<Checkpoint, RunError>;     // RunErrorKind::Poisoned if poisoned
    pub fn status(&self) -> Status;                               // Ready | Poisoned { tick, phase }
    pub fn last_report(&self) -> Option<&TickReport>;
    // read-only accessors (E4)
    pub fn world(&self) -> &World;  pub fn tick(&self) -> u64;  pub fn hash(&self) -> u64; // cached
    pub fn price(&self, n: NodeId, g: GoodId) -> Option<f64>;  /* ema, supply, demand alike */
    pub fn param(&self, p: ParamId) -> Option<f64>;            // None outside the book or registry
    pub fn holding(&self, h: Holder) -> Option<&Inventory>;
    pub fn holdings_of(&self, g: GoodId) -> impl Iterator<Item = (Holder, f64)> + '_;
    pub fn actor_state(&self, a: ActorId) -> Option<&ActorState>;
    pub fn observe_holdings(&self) -> HoldingTotals;   // owned, for another thread
}
pub struct Trace(pub Vec<TraceEntry>);
pub struct TraceEntry { pub phase: Phase, pub delta: StateDelta<Agents>, pub moved: f64 }
pub fn audit_replay(tape: &Tape, until: u64) -> Result<u64, ReplayError>;   // §7.6; the final hash
pub fn registry(tape: &Tape) -> Result<Vec<RegistryLine>, LoadError>;       // §5's listing
pub struct RegistryLine { pub path: String, pub value: f64, pub entry: Entry, pub basis: Basis }
pub enum Entry { Param { unit: Unit, use_: Use, sites: Vec<SiteLine> }, Inline }  // sites: S2.2
pub struct SiteLine { pub path: String, pub method: ClockMethod, pub per_tick: f64 }
pub enum Use { Live, Fixed, Schedule }
```

`run_until(T)` steps until the state's tick is `T`, calling `on_tick` after each step; it runs
nothing when the tick is already at or past `T`. `Sim`, `Tape`, `World`, `Checkpoint` and
`TickReport` are `Clone`, so a frontend on another thread keeps its own `World` for key lookups.
A frontend on a worker thread steps and sends each `TickReport` (and, at its own cadence,
`HoldingTotals`) over a channel; the cli is one such frontend, on the main thread.

### 7.2 The tick

Tick t runs on the state whose tick is t. One ledger opens before phase 0, and every phase's
deltas pass through `apply` with that phase.

| # | Phase | In Phase 0 |
|---|---|---|
| 0 | events | `schedule.fire(t)`, applied in firing order |
| 1 | decisions | `decide` for every actor on the phase-start state; checked (E7), then applied in `ActorId` order |
| 2 | clearing | `admit`, then `clear` |
| 3 | settlement | `settle`, apply, `realize`; no escrow may remain |
| 4 | production | `produce` for every actor on the phase-start state; checked, then applied in `ActorId` order |
| 5 | upkeep | 5a, core ageing: `Age` per holder in `Holder` order. 5b, the `upkeep` hook, handled as in phase 4 (a no-op for scripted actors) |
| 6 | price update | `update_prices`, then `AdvanceTick` |
| 7 | measure | the run's ledger closes the tick's ledger (the tick's audit) and checks the run (the run's audit); `state_hash`; the `TickReport` |

The report's `hash` is of the state after `AdvanceTick`, whose tick is t + 1. A checkpoint "at
tick t" holds the state whose tick is t, before tick t runs.

### 7.3 Hooks: phase-start reads and the whitelist (E6, E7)

In phases 1, 4 and 5b every actor's hook receives a `View` of the state as it stood when the phase
began; the engine collects all outputs, checks each one, and only then applies them in `ActorId`
order. This is safe because hooks write only their own holdings and state: another actor's
deltas can only add to an actor's holdings, never overdraw them. It is also what PLAN §3.8's
ordered reduction across region shards will need.

| Hook | May emit |
|---|---|
| `decide` | `Transfer { from: Actor(me), to: Actor(_) }` of any good; `Mint { to: Actor(me) }` of an Instant good with `Endowment`; `Actor(d)` with `owner(d) == Some(me)`; orders with `actor == me` and `class ==` my class |
| `produce` | `Burn { from: Actor(me) }` with `Production` (a Desk) or `Consumption` (a Pop); `Mint { to: Actor(me) }` with `Production` or `Endowment`; `Actor(d)` owned by me |
| `upkeep` | `Burn { from: Actor(me) }` with `Depreciation`; `Actor(d)` owned by me |

Anything else — a mint to or burn from another holder, any escrow, `SetPrice`, `SetEma`,
`SetVolumes`, `SetParam`, `Age`, `AdvanceTick`, `ScalePrice` (S2.3), a foreign order — is `RunError::ForeignWrite`
(or `ForeignOrder`), and core's phase rules still apply underneath. Phase 3 adds `Construction`
burns to `produce`'s list; the list lives in the engine, so core does not reopen. Tape events
(phase 0) are the world's hand and are not whitelisted; an `Ext` delta with `owner == None` is
legal only there.

### 7.4 The observation: `TickReport` (E4)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]   // on every type below
pub struct TickReport {
    pub tick: u64, pub date: Date, pub hash: u64,            // date = clock.date_of(tick)
    pub markets: Vec<MarketLine>,       // every (node, non-currency good), in (node, good) order
    pub settlements: Vec<SettleLine>,   // one per admitted order, with moved quantities (§3.2)
    pub rationing: Vec<RationLine>,     // one per (node, good, class, side) that had an order
    pub audit: TickAudit,               // the ledger's (good, provenance) lines and margins
    pub run: RunAudit,                  // the run's ledger from its first tick (P0.7, P0.9)
    pub events: Vec<FiredEvent>,        // in firing order
}
pub struct MarketLine { pub node: NodeId, pub good: GoodId,
    pub price: f64,       // posted, used by this tick's settlement
    pub next_price: f64, pub ema: f64,  // after the update
    pub supply: f64, pub demand: f64,   // S and feasible D (N7)
    pub cleared: f64,                   // the good that left the escrow
    pub buyer_fill: f64, pub seller_fill: f64 }
pub struct FiredEvent { pub key: Key, pub occurrence: u32, pub action: StateDelta<Agents>,
                        pub source: Option<Key> }  // Firing::source: a SetParam's source (S2.2)
pub struct HoldingTotals(pub Vec<(Holder, GoodId, f64)>);   // (holder, good) order
impl MarketLine { pub fn trades(&self) -> bool; }  // markets' MarketFill::trades (S2.3)
```

A fired `SetParam` names the param its value was copied from, which carries the new value's
basis, so a frontend can explain a number from the report alone, and a fired `ScalePrice` names
its factor (amended at S2.3); every other firing names none (amended at S2.2, D10 item 2;
`fired_event_names_its_source`). `MarketLine::trades` is markets' own predicate, both sides
posted and both fills positive, so certify and the probe read one definition of a market that
traded (S2.3; `market_line_trades_as_markets_do`).

`run` counts from the run's first tick: genesis for `Sim::new`, and for `Sim::resume` whatever
the checkpoint's run counted from, since a checkpoint carries the run's ledger (amended at P0.9,
O8). So a resumed run's reports equal the uninterrupted run's in full, `run` included
(`gate_resume_from_checkpoints`), and a leak stops both at the same tick. A report costs O(markets +
orders + events) and never copies holdings; `observe_holdings` does that, O(holders × goods
held), when a frontend asks. Ids in a report are dense and are read
against `Sim::world()`; a frontend that compares runs of two tapes maps them by key. Each report
agrees with the accessors read after its step: each line's `next_price`, `ema`, `supply` and
`demand` equal `Sim::price`, `ema`, `supply` and `demand` for its (node, good), and `hash` equals
`Sim::hash()`.

### 7.5 Errors and poisoning (E5)

```rust
pub struct RunError { pub tick: u64, pub phase: Phase, pub kind: RunErrorKind }
pub enum RunErrorKind { Core(CoreError), Order(OrderError), Price(PriceError),
    Agent { actor: ActorId, hook: Hook, error: AgentError },           // a hook that cannot run
    ForeignWrite { actor: ActorId, hook: Hook, delta: StateDelta<Agents> },
    ForeignOrder { actor: ActorId, order: Order }, Poisoned }
pub enum ResumeError { Load(LoadError),   // a wrong format or digest is refused at decoding
    WrongWorld { tape: u64, checkpoint: u64 }, WrongPrefix { tick: u64, tape: u64, checkpoint: u64 },
    Invalid(CoreError) }
pub enum ReplayError { Load(LoadError), Run(RunError), Mismatch { tick: u64, live: u64, shadow: u64 },
    Shadow { tick: u64, phase: Phase, error: CoreError } }   // a traced delta the shadow refuses
```

`Display` gives the ledger line for a shortfall or a conservation breach. `apply` is atomic per
delta, not per tick, so a failed step leaves the state partway through its tick. The `Sim` is
then `Poisoned { tick, phase }`: `step`, `run_until` and `checkpoint` return
`RunErrorKind::Poisoned`, and `last_report` still returns the last good tick's report. The
accessors still answer, reading the state as the failed delta left it, for diagnosis only; no
invariant holds for it, and `hash()` is of that state. A `Sim` is rebuilt with `Sim::new` or
`Sim::resume`. No step discards the audit (N3); July's did (`v2p3: systems/mod.rs:37`). A
breach of the tick's ledger or of the run's is `RunErrorKind::Core(CoreError::Conservation(_))`
in phase 7, and poisons the `Sim` like any other (`gate_breach_stops_the_run`).

### 7.6 Checkpoints, resume and the replay audit (N11, E1)

- **`checkpoint`** stores `world_id`, `prefix_id(state.tick)`, the state and the run's ledger;
  its encodings add the format and the digest over all four (format 3, amended at P0.9).
- **`Sim::resume(tape, cp)`** resolves the tape. It then requires `cp.world_id() ==
  world.world_id` (`WrongWorld` otherwise) and `cp.prefix_id() ==
  world.prefix_id(cp.state().tick())` (`WrongPrefix`), and validates the state against the world:
  every declared actor holds an inventory, every id is in range, no escrow is held, every lot's
  life fits its good, and params, book and extension state have the right shape (`Invalid`). A
  checkpoint's format and digest were checked when it was decoded, and its fields cannot change
  since. So a checkpoint stays valid across an edit dated at or after its tick, such as a new
  shock or a dated `SetParam` to a new value (§2.6). It is refused after any edit to the world or
  to anything that fired before it, and editing its own `world_id` or `prefix_id` to match breaks
  its digest (amended at P0.9, O7). The resumed run continues the run's ledger the checkpoint
  carries, once `validate` has checked that it fits (O8).
- **What a resume trusts** (amended at P0.6, P0.7 and P0.9). A checkpoint is the one input
  besides the tape. The digest catches any field, its identity, state or run's ledger, edited or
  corrupted after it was saved (the identity and the run's ledger since P0.9, O7 and O8), and
  validation catches a state or run's ledger that does not fit the world, but the digest is
  FNV-1a, not a signature: bytes or RON written with a recomputed digest (a decoder's `Digest`
  error even reports the digest the edited checkpoint has) resume as given if they fit the world.
  That is the trust boundary, and it is the only way across it for a frontend that depends on the
  engine alone: `Sim` has no setter, `Checkpoint` hands out no `&mut`, no trait impl hands out
  `&mut` to either or to the state, and the engine re-exports none of core's writer (`apply`,
  `resolve`, the ledgers), so `Checkpoint::of`, which the type alias does expose, can only wrap a
  state some `Sim` made or a decoder accepted, with a run's ledger some checkpoint carried at
  that tick, for any world whose shape it fits: no more than a forged digest allows
  (`no_reexport_hands_out_core_writer`, `no_public_api_hands_out_mut_state` and the engine's doc
  tests; since P0.9 the scans read every public function a frontend can reach, every impl of
  `Sim`, `Checkpoint` and `SimState`, trait impls included, and their fields, O11). Until P0.7
  the engine re-exported core whole, and ENGINE said the replay audit needed that; it did not,
  since the audit is engine code and uses core directly. A program that depends on core itself
  holds core's writer and builds its own states; that is outside the engine's API. A run that
  must be trusted from its tape is proved by `audit_replay` from genesis, and session 2's
  manifest can record the digest of any checkpoint a run resumed from.
- **`audit_replay(tape, until)`** runs a `Sim` with `step_traced`. Beside it runs a shadow
  `SimState` from the same genesis, which applies each tick's trace through `core::apply` alone,
  with no behaviour, clearing or settlement code, and a ledger of its own. The two `state_hash`es
  are compared every tick (test_03's design; July's `v2p3: runner.rs:240-255`). This proves `apply`
  is the only writer. It is not how an edited tape is rerun; that is `Sim::new` or `Sim::resume`.

## 8. rustyecon-cli: a thin binary

The cli holds argument parsing (clap), file reads and writes, the mapping from extension to
format, and exit codes. Nothing else.

```
rustyecon run      <tape.ron> --until <tick> [--out DIR] [--checkpoint-every N] [--format bin|ron] [--hashes FILE]
rustyecon resume   <checkpoint.bin|.ron> --tape <tape.ron> --until <tick> [--manifest FILE] [same options]   # N11
rustyecon replay   <tape.ron> --until <tick>                                              # audit_replay
rustyecon registry <tape.ron>
rustyecon certify  <tape.ron> (--criteria FILE | --until <tick>) --out DIR [--telemetry]   # S2.4
```

From S2.4 the cli depends on certify too, with its `parquet` feature, and `build.rs` stamps the
build (docs/CERTIFY.md §3, §10). Every hash it prints or writes names its run, `--out DIR` writes
`DIR/manifest.ron`, and `resume` verifies its checkpoint against that manifest.

- `--until T` runs until the state's tick is `T`; a `T` before the starting tick is exit 1.
- **Checkpoints.** With `--out DIR`, the final state always goes to `DIR/tick_{:08}.{bin|ron}`,
  named by the state's tick (`--format`, default `bin`). With `--checkpoint-every N` (which needs
  `--out`, N ≥ 1) as well, so does every state a step reaches whose tick is a multiple of `N`;
  the starting state is never written, so a resume cannot overwrite its own input. Without
  `--out`, nothing is written. `DIR` is made at the first write. A failed write stops the run with
  exit 3, and no further tick runs; July only printed it (`v2p3: runner.rs:488-505`).
- **Hashes.** `--hashes` writes one `{t} 0x{hash:016x}` line per step, `t` the state's tick after
  it (1 to `T` for a run, `c+1` to `T` for a resume from tick `c`), and stdout ends with the same
  line for the final state. From S2.4 the file opens with four `#` lines naming the build, the
  tape by `tape_hash`, the `world_id` and the starting tick (with the resumed checkpoint's
  digest), and stdout prints a `run …` line with the same fields before the final hash, `replay`
  too (R16; docs/CERTIFY.md §10). The body is unchanged.
- **Manifest and resume** (S2.4). With `--out DIR`, `DIR/manifest.ron` records the run and each
  checkpoint it wrote, rewritten after each. `resume` verifies its checkpoint against the manifest
  of the run that made it, beside the checkpoint or `--manifest FILE`: the same `tape_hash` and
  world, and a record at its tick with its digest and state hash. So `--tape` must be that run's
  tape (R16's letter); a resume under a dated edit waits for docs/CERTIFY.md §15.1, question 2.
- **Exit codes:**

  | Code | Meaning |
  |---|---|
  | 0 | ok; for `certify`, PASS |
  | 1 | load or argument error: an unreadable tape, an unknown checkpoint extension, a resume whose tape does not load, criteria that do not load or fit, an `--out` holding the manifest a resume verifies against; clap's own argument errors are mapped here |
  | 2 | run error: the error or ledger line and the last good tick go to stderr (`replay` too, when the live run fails) |
  | 3 | I/O or checkpoint error: an unreadable or undecodable checkpoint (one whose state does not match its digest included), a failed write, a refused resume, an unverified checkpoint |
  | 4 | replay mismatch, or a traced delta the shadow refuses |
  | 5 | `certify`'s verdict is FAIL or UNSCORED; the certificate is written first (S2.4) |

- July's `--agents`, `--price-rule` and `--supply-rule` switches do not return (N13), and no
  command-line override of any kind exists (E1).

## 9. Determinism and hashing (R8, A5)

- **Canonical order.** Phases run in order. Within a phase:
  - events by (tick, date, key) (amended at P0.9);
  - decisions and production outputs by `ActorId`;
  - admission by (actor, node, good, side);
  - clearing and prices by (node, good);
  - settlement by (node, good), then steps 1–4, then the taker order of §3.2.

  Shuffling `orders` before `admit`, or any tape list before loading, leaves the stream unchanged.
- **Collections.** `Vec` in id order, or `BTree*`. No hashed container, no `rayon`, no RNG.
- **Floats on the state path.**
  - Allowed: `+ − × ÷`, `sqrt`, comparisons, `total_cmp`, `min`, `max`, `abs`, `floor`, `round`,
    `next_down`, `next_up`, and int↔float casts.
  - Only through `core::num` (libm): `exp`, `expm1`, `ln`, `ln1p`, `pow`; and `fma`, correctly
    rounded, which the oracle uses and the state path does not (P1.1).
  - Banned: `mul_add`, `powi`, `powf`, the inherent transcendentals, `f32`, and fast-math.
  - Sums are left folds in canonical order, and no value is `-0.0` (§2.5).

  The verdict path follows the same rules in session 2.
- **Enforcement.** `clippy.toml` denies the std hash containers and, by primitive path, the
  inherent transcendentals, `powi` and (from P0.2) `mul_add`; a probe confirmed clippy rejects
  `f64::exp`, `ln` and `powi`. The source scans of §11 back it up. Linux–Windows hash equality is
  recorded in STATE.md, not gated.

## 10. The gate world (`tapes/gate.ron`)

It starts on 1750-01-01 at 52 ticks a year and runs 2,080 ticks, 40 years.

- **Goods:** `coin` (currency), `grain` and `fuel` (indefinite), `bread` (three-week life).
  **Nodes:** `town` and `village`, both in coin. **Classes:** producers, households, pensioners.
  **Market:** `Imbalance`, `one_sided: Hold`.
- **Desks** (key order gives `Desk(0)` … `Desk(3)`); every desk trades at both nodes, so that
  all six markets trade (amended at P0.5):
  - `farm` (home village) and `mine` (home town) are endowments of grain and fuel. Each sells a
    registered flow in town and the rest of what it holds in the village.
  - `mill` (home town) is Leontief: grain 2 + fuel 1 → bread 3. It buys grain and fuel at both
    nodes, and sells a registered flow of bread in town and the rest in the village.
  - `oven` (home town) is a second bread desk, declared dormant; the entrant pattern. Awake, it
    buys grain and fuel in town and sells bread in the village.
  - Mine, mill and oven pay the workers; the farm pays the workers and the pensioners.
- **Pops:** `pensioners` (home village, `Pop(0)`) hold bread lines at both nodes that their budget
  cannot cover at genesis prices, which makes them the cash-short buyer in both bread markets.
  `workers` (home town, `Pop(1)`) buy bread in town. Both consume bread.
- **Shared numbers.** `Desk(0)` (farm) and `Pop(0)` (pensioners) share a number, as do `Desk(1)`
  (mill) and `Pop(1)` (workers). At genesis, bread demand exceeds the mill's capacity.
- **Events.** Four dated events, listed out of order:
  - the mine's capacity is cut in 1760;
  - the workers' bread line rises in 1765;
  - the oven activates in 1768;
  - the mine's capacity is restored in 1770.

  Bread rations again after the cut: the mill's fuel stock carries it for about 16 ticks, and then
  the village's bread fills at under a quarter for weeks. A yearly recurring mint pays the
  pensioners coin.

The implementer tunes the numbers, registered as `Assumed`, to one bar: every market trades in
every year, and no price leaves [1e-3, 1e3] × its genesis value (`gate_world_meets_its_bar`; the
widest excursion is about 16× genesis, the village's bread in 1769). The pension grows the money
stock, so prices drift up about 1% a year; over the 20,000 ticks of `gate_lots_bounded` they reach
about 70× genesis, still finite. Failure tests make variants of this one file, by text
substitution or by editing the parsed `Tape`. No second hand-kept tape exists.

## 11. Tests (the A3 gate)

Run with `cargo test --workspace --release`. Test names are binding: a test may be split, not
merged. Test bars are relative, named once at the top of their test file with the reason for their
size; none is absolute (A12). Gate tests read `tapes/gate.ron` through `include_str!`.

**core**

| Area | Tests | Checks |
|---|---|---|
| inventory (salvaged, `v2p3: types/inventory.rs:159-251`, renamed for the new API) | `add_and_get`, `take_partial` (was `remove_partial`), `take_more_than_held_is_a_shortfall_and_moves_nothing` (was `remove_more_than_held`), `stays_sorted`, `age_removes_expired_and_decrements` (was `spoil_lots_…`), `serde_preserves_lot_lives`, `take_fefo` (was `remove_fifo`) | defect 5; atomic take |
| inventory (new) | `lots_coalesce_by_life`, `take_all_of_multi_lot_holding_never_falls_short` (lots 0.1 + 0.2), `nonfinite_negative_or_negative_zero_qty_is_rejected`, `instant_lot_dies_at_5a`, `split_and_merge_rounding_is_measured` (P0.7: 1 from 1e17, 6 and 9 into it; before + rounding = after + taken, exactly, in integers), `several_merges_report_each_rounding` (P0.9: [7, 1e-17] into [1e17, 1] reports −7 and −1e-17, whose float sum is −7; checked by an exact expansion sum) | N9, R2 |
| ledger (salvaged: the 11 tests at `v2p3: certify/ledger.rs:359-488`, names kept) | `balanced_transfer_conserves`, `unbalanced_transfer_is_a_breach`, `declared_mint_and_burn_conserve`, `untagged_creation_is_a_breach`, `stock_appearing_beyond_what_was_declared_is_a_breach`, `mutation_that_skips_the_ledger_is_caught`, `tolerance_covers_scan_noise_on_large_stock`, `tolerance_still_catches_a_real_leak_at_the_same_stock`, `margin_ratio_reports_headroom`, `shortfall_is_recorded` (now also returns the error), `reset_clears_state` (now: a fresh ledger has no lines). An unbalanced move or untagged creation can only be a direct state write now, so those tests write state directly | defect 9, A12 |
| ledger through apply (salvaged, `v2p3: tests/test_04:42-121`) | `untagged_creation_is_caught_through_apply` (the untagged add is a direct write, since every `Mint` needs a provenance), `shortfall_is_captured_and_not_silently_clamped` (and stops with `CoreError::Shortfall`). The third, `unbalanced_transfer_through_apply_is_a_breach`, is replaced by `atomic_transfer_cannot_mint`, since a `Transfer` can no longer be unbalanced unrecorded: its rounding is a declared line (`transfer_rounding_is_declared`, P0.7) | defect 9, N3 |
| ledger (new) | `undefined_good_is_an_error_in_release`, `nan_drift_is_a_breach`, `zero_stock_zero_flow_requires_exact_zero`, `margin_is_zero_when_drift_is_zero`, `direct_state_write_is_caught`, `burn_shortfall_stops_with_a_ledger_line`, `atomic_transfer_cannot_mint`, `escrow_outside_settlement_is_an_error`, `instant_mint_outside_phases_0_and_1_is_an_error`, `book_writes_outside_their_phase_are_errors`; from P0.7 `transfer_rounding_is_declared` (the limit pinned: a thousand transfers of 1 from a lot of 1e17 each move 1 and declare +1 as `Rounding`, 6 merged into it declare −6, a mint and a burn declare theirs beside their own lines; every total checked exactly in integers; `Rounding` reserved) and `run_ledger_catches_a_leak_below_each_ticks_tolerance` (a leak of 1e-10 a tick on a stock of 16 passes each tick and breaches the run at its second tick; a clean run's lines are the ticks' sums); from P0.9 `multi_lot_rounding_is_declared_exactly` (a burn of [1e17, 7] declares −1e17 and −7 as `Rounding`, a transfer of [7, 1e-17] into [1e17, 1] declares −7 and −1e-17, and 3,000 awkward multi-lot burns and transfers, whole and partial: every delta's posts add up exactly, by expansion, to what its lots gained or lost) and `flow_tolerance_is_pinned_for_a_tick_and_a_run` (with the flow term dominant, a drift at 0.9 of the tolerance passes and at 1.1 breaches, for a tick and for a run, and the breach's `gross` and `tol` are exactly Σ moved + Σ \|declared\| and the formula) | N2, N3, A12, R2 |
| hash | `identical_states_hash_equal_and_tick_changes_it` (salvaged), `hash_covers_holdings_lives_params_and_ext_state` | R8 |
| checkpoint (salvaged, `v2p3: output/checkpoint.rs:84-102`) | `binary_round_trip` and `human_readable_round_trip` (now by full hash, not by tick), `checkpoint_format_is_checked` (format 1 refused), `checkpoint_rejects_nan_and_unknown_ids`, `checkpoint_digest_refuses_an_edited_state` (P0.6: one lot changed, in each form), `checkpoint_digest_covers_identity_and_run` (P0.9: `world_id`, `prefix_id` and the run's ledger each edited, in each form, are refused; a run's ledger that does not fit the state is refused by `of` and `validate`), and the module's doc tests (`&mut cp.state` does not compile) | N11, R2, E1, O7, O8 |
| tape | `unsorted_events_fire_in_order` (ticks listed 5, 2, 9, 9 all fire; key order within a day, and from P0.9 date order within a tick, the prefix hashing the same order), `recurring_last_is_inclusive` (P0.7: a `last` on an occurrence's date fires there and never after, and `prefix_id` at every tick around it is the hash of what `fire` produced), `every_zero_is_rejected`, `undefined_good_is_a_load_error`, `unknown_actor_or_param_is_a_load_error`, `duplicate_keys_are_rejected`, `missing_genesis_price_is_rejected`, `unused_param_is_rejected`, `unit_mismatch_is_rejected`, `event_before_start_is_rejected`, `set_param_on_fixed_or_other_unit_is_rejected`, `ratio_with_saturate_is_rejected`, `date_to_tick_is_integer_exact`, `date_of_inverts_date_to_tick`; from S2.3 `scale_price_is_an_event_only_delta` (in `apply.rs`: phase 0 only, `fl(p·factor)` with the EMA left alone, bad factors and results refused with nothing moved, and at load a currency, a bad factor, a factor in another unit and a recurring one refused, the factor a schedule param with its `Value` site, and the world kept) | N1, N2, R4, N13, C1 |
| tape schema | `schema_version_is_checked`, `unknown_field_is_rejected`, `missing_optional_field_is_rejected`, `tape_round_trips` (parse, `to_ron`, parse: equal `Tape`, equal `world_id`), `file_order_is_irrelevant` (permuting every list: same `world_id` and prefix ids), `reformatted_tape_keeps_its_ids` (whitespace, comments, CRLF: same ids), `new_entity_keeps_existing_ids` (adding a good, an actor and an event leaves every existing key naming the same entity with the same resolved content), `prefix_id_covers_only_past_firings`, `schedule_params_stay_out_of_the_world` (P0.6: a new source value, a new dial and its event, a new period keep `world_id` and the past before they fire; a source the world reads stays registered, and from S2.2 keeps the world), `ledger_tolerances_must_be_below_one` (P0.6), `param_sites_are_recorded` (S2.2: every use at its path with its method; a source takes its target's methods at the event's `to`, a period `Ticks` at its `every`; a target's `act.param` is not a use) | E8, N11, E1, R2, D10 |
| other | `clock_conversions` (all units and methods of §6; `ticks` at 12, 52 and 365 ticks a year), `num_matches_libm_bits`, `num_helpers_are_exact` (each result fits and its `next_up` does not), `two_sum_is_exact` (P0.7), `clock_methods_convert_as_the_clock` (S2.2: each `ClockMethod` is its `Clock` conversion bit for bit at 1, 12, 52 and 365 ticks a year, with its unit and name; a `Site` reads only in its method's unit), `core_has_no_workspace_dependencies` (core's `Cargo.toml` names no `rustyecon-` crate; a `NoExt` tape resolves, fires its events through `apply` and closes its ledger) | A13, A5, N14 |
| added at P0.3 | `desk_and_pop_sharing_a_number_are_distinct_holders`, `transfers_keep_lot_lives`, `set_param_respects_the_registry`, `unknown_holders_and_reserved_provenance_are_errors`, `currencies_and_prices_rates_are_checked`, `genesis_holdings_start_with_full_lives`, `extension_errors_name_their_path`, `core_types_cross_threads`, `keys_check_their_character_set`, `actor_and_holder_orders_are_canonical`, `dates_parse_print_and_count_days`, `fnv_matches_the_published_vectors` | defect 10, E3 |

**markets**

| Area | Tests | Checks |
|---|---|---|
| clearing (salvaged, `v2p3: systems/clearing/mod.rs:147-176`, retargeted) | `price_rises_on_excess_demand`, `price_falls_on_excess_supply`, `balanced_market_stable`, `imbalance_excess_demand`, `imbalance_excess_supply`. July's `abs() < 1e-12` checks become bit equality: `exp(0) = 1` exactly, and the imbalances are exactly ±0.5 | A12 |
| settlement | `cash_short_buyer_settles_both_sides_from_one_fill`: the buyer asks 10 at price 2 on budget 6, so 3 is feasible; the seller offers 5. Asserts `SetVolumes { supply: 5, demand: 3 }`, `buyer_fill` 1 and `seller_fill` 0.6; the buyer gets 3 and pays 6, the seller ships 3 and gets 6; the class line is 10 / 3 / 3; `update_prices` lowers the price (x = −0.4), where July's settlement would have read D = 10 and raised it | N7, R12 |
| admission, settlement | `over_budget_is_an_order_error`, `over_posting_seller_is_an_order_error`, `sells_at_two_nodes_are_checked_cumulatively`, `duplicate_order_is_rejected`, `two_kinds_same_number_settle_apart` (from P0.7 also the sell side: each pop sells grain beside the desk with its number, which holds grain and coin, and ships from and is paid into its own holding), `no_forgiveness_currency_moves_only_by_transfers` (from P0.7 its only ledger lines are `Rounding`, exact against each good's lots summed exactly), `escrow_is_empty_after_settlement`, `settlement_is_invariant_to_order_input_order`, `all_taker_is_largest_then_highest_id`, `settle_lines_carry_moved_quantities` | defect 10, N8, R8 |
| prices | `tiny_prices_move_by_rule` (at 1e-15 and 1e-300 each step is exactly `p·exp(k·x)`), `huge_price_overflow_is_a_run_error`, `tiny_and_huge_prices_settle_without_guards` (1e-200 flows move at prices 1e-15 and 1e200), `price_rule_comes_from_the_tape` (no rule, no load; `Ratio` and `Imbalance` hash apart), `one_sided_rule_comes_from_the_tape` (at S = 0 < D, `Saturate` multiplies by `exp(k)` each tick and `Hold` keeps the bits), `ratio_nonfinite_is_an_error`, `genesis_ema_is_the_genesis_price`, `params_are_read_at_use_time` | N5, N6, N13, F8 |
| time | `ema_is_tick_length_invariant`, `imbalance_path_is_tick_length_invariant` | A13 |
| added at P0.9 | `tiny_imbalances_move_the_price` (imbalances of 1e-12, 1e-9 and 1e-6 each move the price by exactly `p·exp(k·x)`: no dead band) | N5, A12, O13 |
| added at P0.4 | `orders_are_checked_against_the_world`, `budgets_bind_per_currency`, `feasible_quantity_is_what_the_budget_buys`, `rationing_is_recorded_per_class` (from P0.6 a class line of two members, one cash-short, with exact sums), `markets_types_cross_threads` | N2, N6, N7, R12, E3 |
| added at P0.6 | `one_filled_quantity_serves_many_takers` (two rationed buyers in one market and two rationed sellers in another: every payment, and every non-last taker's quantity and receipt, exactly nominal), `a_market_whose_fill_underflows_does_not_trade` (amendment 5) | §3.2 |

**agents**

| Area | Tests | Checks |
|---|---|---|
| roles (P2.0.1) | `roles_never_overbudget_or_overdraw`, `good_desk_makes_output_at_either_corner`, `ex_post_assignment_uses_up_both_inputs`, `technique_moves_toward_the_task_measure`, `households_offer_the_cdf_and_the_provider_funds_one_basket_per_head`, `cash_rule_outlay_follows_coin_and_markup`, `role_state_rejects_unclean_values`, `roles_read_only_their_view`, `role_specs_are_checked_at_load`, `role_specs_round_trip_in_canonical_form`, `role_sites_name_their_methods` (S2.2: appb under July's step rule; every `RatePerYear` site against docs/probe/RULES.md §2, `up` and `down` log steps and every other rate a share, in the registry and in the spec, each read through the view equal to the `Clock`'s conversion) | R13, R4, R8, R12, G10, D10 |
| seam | `leontief_never_overdraws`, `scripted_actor_reads_only_its_view` (foreign holdings change; the decision does not), `extreme_spend_rate_never_overbudgets` (v·Δ = 40, so `share` = 1.0; payouts and budgets pass admission), `weights_must_sum_exactly_to_one`, `dormant_actor_does_nothing`, `tiny_cash_and_stocks_still_pay_out_and_produce` (P0.6: 1e-300 coin, grain and fuel), `per_tick_conversions_follow_the_clock` (P0.9: at 12, 52 and 365 ticks a year every buy and `Flow` sell line posts `q/ticks_per_year`, `AllHeld` what is left, and the payouts and spending totals are `1 − exp(−r/ticks_per_year)` of the cash, exactly, against values computed from the params), `each_site_converts_as_registered` (S2.2: on gate and appb, the sites every spec holds are exactly the registry's sites under `actors[..]`, path and method, and each reads through the actor's view as its registry line converts, at genesis and after every live param is doubled) | R13, R4, A12, A13, D10 |

**engine**

| Area | Tests | Checks |
|---|---|---|
| determinism | `gate_repeat_identical_hashes` (2,080 ticks; the state evolves), `gate_resume_from_checkpoints` (checkpoints at ticks 1, 520, 1,040 and 2,079 through `to_bytes`/`from_bytes` and `to_ron`/`from_ron`; the tails equal the uninterrupted run, and from P0.9 every resumed report equals the uninterrupted run's in full, its run audit included), `resumed_run_stops_where_the_uninterrupted_run_does` (P0.9, a unit test in `sim.rs`: a leak of 1e-9 coin before every tick stops the uninterrupted run early; resumed from a round-tripped checkpoint every 1, 2, 3 or 7 ticks, the run stops at the same tick with the same breach), `gate_replay_matches_every_tick` (`audit_replay`), `file_order_is_irrelevant_to_the_hash_stream` | R8, N11, E8 |
| conservation, rationing | `gate_conserves_every_tick` (every report has `max_margin <= 1` and no shortfall; from P0.7 the run's audit too, whose lines are the fold of the ticks', and some tick declares `Rounding`). `gate_breach_stops_the_run` (P0.7: the gate tape with both tolerances at 0 stops in phase 7 with `CoreError::Conservation`, the `Sim` poisoned and its checkpoint refused). `gate_rounding_is_declared` (P0.7: the mill's genesis coin at 1e17 and its payout off; every tick its coin moves beyond its traced transfers by the tick's coin `Rounding` line, within `REL` of the tick's coin flow, on more than 100 of 520 ticks, and the run's line is their sum). `gate_events_fire_in_date_order`. `fired_event_names_its_source` (S2.2: each fired `SetParam` names the param it copied, `mine.cut` `mine.capacity.cut`; `oven.opens` and all 40 pensions name none). `gate_ids_apart`: for `Desk(0)`/`Pop(0)` and `Desk(1)`/`Pop(1)`, every tick and good, the holding's change equals the signed sum of the trace entries naming that holder, and each of its settle lines names it. `gate_rations_and_records_by_class`: bread rations at tick 0 and after the 1760 cut; pensioners' requested > feasible; every class line is the fold, in actor order, of its members' order quantities, feasible quantities and settle lines (from 1768 two producers share a side); the recorded demand is the actor-order fold of the feasible buys exactly, and at every rationed tick equals the class lines' Σ feasible up to fold order (a relative bar), which is below Σ requested. `gate_settles_by_one_filled_quantity` (P0.6): §3.2 on every tick and market, and expenditure equals receipts. `gate_lots_bounded`: 20,000 ticks; lots per good ≤ life + 1; currency one lot. `market_line_trades_as_markets_do` (S2.3): on every gate tick and market `MarketLine::trades` is `MarketFill::trades` of its fields, and a line with no supply does not trade | R2, N1, defect 10, R12, N7, N9 |
| edits (E1) | `resume_after_future_event_edit_equals_full_rerun`, `resume_after_past_edit_is_refused`, `resume_boundary_is_the_firing_tick` (P0.7: at the 1760 cut's tick, the pension's first occurrence and tick 0 with a new start-date event, the checkpoint at the firing tick resumes under the edit and equals its full rerun, and the one a tick later is `WrongPrefix`), `world_edit_refuses_every_checkpoint`, `dated_param_change_takes_effect_at_its_tick` (the 1760 cut: capacity changes at `tick_of(1760-03-01)` and not before), `entrant_activates_on_its_date`, `resume_after_a_new_dial_value_equals_full_rerun` (P0.6), `resume_refuses_an_invalid_state` (P0.6: five edits of a RON checkpoint, refused by the digest, then as `Invalid` once the digest is forged), `resume_refuses_an_edited_identity` (P0.9: the `prefix_id` a refused resume under an earlier cut reports, and the `world_id` one under an edited price rate reports, each written into the checkpoint in each form, are refused by the digest), `new_source_event_keeps_world_id` (S2.2: a 1780 event copying `mill.capacity`, live, into `workers.buy.bread` keeps `world_id` and every `prefix_id` through 1780; the tick-1,040 checkpoint resumes under it and equals its full rerun), `kicked_tape_keeps_world_and_past` (S2.3: the gate with a dated `ScalePrice` at tick 1,000, its factor a schedule param, keeps `world_id` and every prefix through 1,000; the tick-1,000 checkpoint resumes under it, that tick's posted price is `fl(p·factor)`, the event names its factor, and the resumed run equals the kicked tape's run from genesis) | E1, N11, O7, D10, C1 |
| frontend contract | `observation_matches_accessors` (every gate tick), `observing_changes_no_hash` (from S2.2 also by `run_until` seven ticks at a time, D10 item 3), `no_reexport_hands_out_core_writer` (P0.7: no `pub use` in engine, markets or agents hands out core whole or its writer, the scanner checked on fixtures first; the engine's doc tests show `rustyecon_engine::rustyecon_core::apply` does not compile; from P0.9 core's re-exports are an allow-list equal to the prelude's, and `{self ..}`, `self as`, `::`, a private alias and `pub extern crate` count as core whole), `no_public_api_hands_out_mut_state` (from P0.9 also every public function of markets and agents, `&mut` to a `Sim` or `Checkpoint`, every impl of `Sim`, `Checkpoint` and `SimState` with trait impls, and their fields; the review's five writers are fixtures, and the engine's doc tests show `state_mut`, `holdings_mut`, `&mut **sim` and `sim.as_mut()` do not compile), `engine_types_are_send` (compile time: `Send + Sync + 'static` for `Sim`, `Tape`, `World`, `Checkpoint`, `TickReport`, `HoldingTotals`, `RunError`, `LoadError`, `ResumeError`, `ReplayError`), `failed_step_poisons_the_sim` | E3, E4, E5 |
| hooks | `hooks_read_the_phase_start_state_every_tick` (P0.6, in `gate.rs`): on every gate tick the applied phase-1 and phase-4 deltas are what each hook returns on its phase's start state, and the cleared volumes are what admission makes of those orders. `decisions_read_phase_start_state`: from a fixed state, visiting actors in id order and in reverse gives identical outputs; renaming keys so that the id order reverses leaves every actor's tick-0 orders unchanged. `behaviour_output_is_whitelisted`: each forbidden arm from each hook (a mint to or burn from another holder, an escrow transfer, `SetPrice`, `SetEma`, `SetVolumes`, `SetParam`, `Age`, `AdvanceTick`, from S2.3 `ScalePrice`, a foreign `Actor` delta, a foreign order) is `ForeignWrite` or `ForeignOrder` | E6, E7, R13, R2 |
| time | `a13_annual_quantities_invariant`: at `ticks_per_year: 12` the 1750 farm and mine mints equal the 52-tick run's within a relative bar of 1e-12, event ticks match their dates, every pension occurrence falls within a month of the 52-tick run's, and a bread lot lasts 3 weekly or 1 monthly tick. `same_tick_events_fire_in_date_order` (P0.9): a cut keyed `z.cut` and a restore twenty days later keyed `a.restore` fire in date order at 12, 52 and 365 ticks a year, and the restore wins at each | A13, E8, O9 |
| source scans (read the sources of core, markets, agents and engine, comments stripped and `#[cfg(test)]` items cut out) | `the_scanner_reads_what_it_should`, `no_raw_transcendentals`, `no_hashed_collections`, `no_behavioural_float_literals` (shipped code may hold only `0.0` and `1.0`, and no named float constant outside `core::num`; from P0.9 no integer but 0 and 1 made float, by `as`, `from` or `into`, no named integer constant made float but clock.rs's calendar, no `from_bits` of a literal and no `parse::<f64>`), `engine_path_does_no_io` (E2's list), `params_are_read_only_through_sites` (S2.2: outside `core::clock` no conversion is called and no param is read by type; checked on fixtures first). From S2.3 three read `crates/certify/src` too: `certify_holds_no_threshold` (no float but `0.0` and `1.0`, no named float constant, no integer made float: every bar is in criteria), `certify_does_no_file_io` (E2's list; only `telemetry.rs` may name `std::io` or `parquet`) and `certify_hands_out_no_core_writer` (no re-export of core or the engine, no `apply`, `resolve`, `Ledger`, `RunLedger` or `Checkpoint::of`, no public function returning a `Checkpoint`, `Sim` or `SimState`), each checked on fixtures first | A5, R8, R4, A12, E2, D10, E1 |
| registry (S2.2) | `registry_names_each_use` (a gate variant whose mill spends at `rate.bread` lists `share` at `actors[mill].spec.spend` and `log_step` at `goods[bread].price_rate`, each equal to the `Clock`'s; one row of each kind of use on the gate, schedule params included; every param of gate and appb is read somewhere, and each site's per-tick value is its method's conversion) | R4, D10 |

**cli**

| Area | Tests | Checks |
|---|---|---|
| product path | `gate_resume_through_product_path`: `run` with checkpoints in both formats, then `resume` from ticks 1, 520, 1,040 and 2,079; the `--hashes` tails equal the uninterrupted run. `replay_command_passes_on_the_gate` (exit 0) | N11 |
| failures | `shortfall_stops_the_run` (exit 2, with the ledger line on stderr), `conservation_breach_stops_the_run` (P0.7: zero tolerances, exit 2, the breach line and the last good tick on stderr, the hashes file ending there), `undefined_good_fails_to_load` (exit 1), `resume_refuses_another_tape` (exit 3; from S2.4 also a dated edit after the checkpoint, which keeps world and past but not the tape), `unknown_extension_errors` (exit 1), `failed_checkpoint_save_stops_the_run` (`--out` names a regular file: exit 3, and the `--hashes` file ends at the failed checkpoint's tick), `resume_refuses_an_edited_checkpoint` (P0.6: a digest mismatch and, with a forged digest, an invalid life, each exit 3), `failed_final_checkpoint_save_stops_the_run` (P0.9: `--out` naming a regular file and no periodic save: exit 3, the error on stderr, no final hash, every tick in the hashes file), `resume_refuses_an_edited_identity` (P0.9: the value a refused resume prints, written into the RON checkpoint's `prefix_id` or `world_id`, is refused as an edit, exit 3) | N3, N2, N11, R2, O7, O12 |
| names, manifests, certify (S2.4) | `hash_output_names_its_run` (the `--hashes` header and the stdout `run …` line name the build, `tape_hash`, `world_id` and the starting tick, in `run`, `resume` and `replay`), `resume_requires_a_recorded_checkpoint` (no manifest, another world, a record whose hash differs: exit 3), `resume_records_its_parent` (the child's manifest records the parent's run key and the checkpoint's digest; from S2.5 a parent manifest with an edited `genesis_hash` verifies, and the child records the tape's own), `certify_command_exits_on_its_verdict` (exit 0, 5 and 5, each with its certificate written; neither `--criteria` nor `--until` is exit 1), `manifest_names_every_input` (the build with its target, `tape_hash`, `world_id`, `genesis_hash`, a resumed checkpoint's digest, the certificate's criteria hash) | R16, N4, N10, D4, C9 |

## 12. Steps

| Step | Delivers |
|---|---|
| P0.3 core | §2 and its tests; `docs/TAPE.md` |
| P0.4 markets | §3 and its tests |
| P0.5 agents + engine + cli | §4, §7 and §8, `tapes/gate.ron` (§10), and the agents, engine and cli tests (one commit, amended at P0.5) |
| P0.6 review fixes | the fixes from adversarial review, round 1 (amended at P0.6) |
| P0.7 review fixes | the fixes from adversarial review, round 2 (amended at P0.7) |
| P0.8 housekeeping | the items below (P0.7 in the draft; P0.6, then P0.7, until the review fixes took those numbers; amended at P0.8) |
| P0.9 review fixes | the fixes from adversarial review, round 3 (amended at P0.9) |

Session 2's steps are `S2.n`, on branch `phase0-s2` from `cf3c0ff`; their contract is
docs/CERTIFY.md, whose §14 has the plan and its amendments. As landed (amended at S2.6):

| Step | Delivers |
|---|---|
| S2.1 contract | docs/CERTIFY.md (`785ab19`) |
| S2.2 engine asks | D10 items 1, 2 and 4, with item 3's loop; every `world_id` re-baselined once (`7c14c37`; amended at S2.2) |
| S2.3 certify | `ScalePrice` and `MarketLine::trades`; `crates/certify`: criteria, batteries, the kick, the seal, the manifest; the testdata (`0e8c2e7`; amended at S2.3) |
| S2.4 cli, criteria, results | three commits: telemetry behind `parquet`, the build stamp, named hashes, verified resume and `rustyecon certify` (`6a80d6d`); the criteria alone (`1ee9b80`); the certificates from a clean build of the criteria commit (`4d59604`) (amended at S2.4) |
| S2.5 verification fixes | the fixes of the bounded verification's one adversarial pass, each with a test, and the probe's delegation to certify (`05533b9`); the certificates regenerated from a clean build of it (`daa62af`) (amended at S2.5) |
| S2.6 close | PLAN §3.2 (decision 39), this file, CERTIFY, TAPE, GUI.md, README, STATE; the spine scripts' portable cache (amended at S2.6) |

Housekeeping (PLAN Phase 0 steps 2 and 6; A3):

- Move `docs/reboot/PLAN.md` to `docs/PLAN.md`, and fix the links to it in REVIEW.md, ADDENDUM.md,
  the root `Cargo.toml` and this file.
- Write `STATE.md` at the root as the resume point. It records:
  - the toolchain, 1.97.1 on both machines, with each `rustc -V`;
  - `test_01` failing at every commit where its tests compile, and ADDENDUM §1.4's corrections (A3);
  - the gate world's final hash on WSL and on Windows, and whether the two are equal;
  - the result of `cargo check --target wasm32-unknown-unknown -p rustyecon-engine` once the target
    is installed, recorded and not gated;
  - what session 2 starts from.
- Put up a CI skeleton (amended at P0.8):
  - `scripts/gate.sh` runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --
    -D warnings` and `cargo test --workspace --release` with warnings denied, then
    `gate_repeat_identical_hashes` by name, then `rustyecon run tapes/gate.ron --until 2080` twice,
    requiring identical `--hashes` files and printing the gate hash, and last the wasm32 check,
    recorded and not gated. Its target directory is outside the tree. In WSL it runs under
    `wsl -d ubuntu --exec bash -lc`.
  - `.github/workflows/ci.yml` runs the same script on `ubuntu-latest` with the toolchain file. It
    runs only once the branch is pushed.
  - The Windows check runs the same commands by hand, and STATE.md records whether the two
    platforms' hash streams agree, without gating on it.

  Whether hosted CI is wanted is the user's choice (A5); the workflow is inert until a push.

## 13. Left for later: do not build now

| Item | Phase |
|---|---|
| Certificate, criteria, verdicts, NaN scan, manifest (it records `world_id`), telemetry, BalanceWatch, price-runaway detector (A12); N4, N10, N12, N15; the GUI's engine asks (A14, D10; GUI.md §7.2): `world_id` without the `fixed` flag the schedule sets (§2.6, `new_source_event_keeps_world_id`), `FiredEvent` naming its source (§7.4, `fired_event_names_its_source`), and `engine::registry` listing each use of a param with its `ClockMethod` (§2.6, §6, §4, §7.1, `registry_names_each_use`); the three engine asks landed at S2.2, and certify with the kick's `ScalePrice` at S2.3 (docs/CERTIFY.md). Done: session 2 closed at S2.6 | Phase 0 session 2 (A4) |
| Parquet writer (moves as is; `TickReport` is its input). Done at S2.4, behind certify's feature `parquet` | Phase 0 session 2 |
| Oracle | Phase 1 (other run; unit 1a joined at P1.1) |
| Pops as rules (pairs, participation, logit); labour and parcel services as Instant goods; machines and (A, Λ, B) desks; the task margin; the income-identity check | Phase 2 |
| Investment, vintages, depreciation, construction, build lags, user cost; population, technology and enclosure timelines (as world-level `Ext` deltas) | Phase 3 |
| Worldgen compiler and its human-editable tables (the runtime tape is its target) | Phase 4 |
| Transport desks, channel state, pass-through recipes; home-node trading (Phase 0 lets an actor post at any node with no channel and no crossing cost) | Phases 4 and 9 |
| Credit, banks, monetary regimes | Phase 8 |
| Region shards and parallel reduction; sweeps of registered params (each point a tape edit, E1) | when §3.9's budget needs them; Phase 6 |
| crates/gui — the interactive frontend, built with egui (eframe); the stack is pinned in docs/GUI.md §3.1 (A14) | G0, after session 2, beside Phase 1 |

## 14. Open questions

1. Should `Imbalance` be `p·exp(k·x)` rather than July's `p·(1+α·x)`? The two differ only at
   second order in `α·x`, and only the exponential form is tick-invariant (A13).
2. Spoilage runs as core ageing in phase 5a, while the agent hook in 5b stays a no-op. The brief
   asked for phase 5 as a no-op, but lot lives need an ageing step. Is that split right?
3. How far does R4 reach? Is inline structural data under an entry's `basis` enough, or must every
   number on the tape be a named param?
4. Soonest-expiring-first out of the escrow favours the lowest id. Is pro-rata freshness needed
   before Phase 2?
5. `Saturate` or `Hold` for the historical runs is Phase 2's choice (F8); the gate world uses
   `Hold`, and both are tested.
6. Review items taken in part:
   - **Home-node trading is not enforced in Phase 0.** The gate world's mill buys grain in the
     village and no transport desk exists yet, so the rule waits for channels and §13 records the
     simplification.
   - **`SetParam` takes a param key, not a literal**, so that the new value carries a basis (R4).
   - **`world_id` leaves out the tape name and the basis texts.** They change no number a run
     computes, so correcting a note should not invalidate checkpoints.
   - **The permutation half of `decisions_read_phase_start_state` is asserted at tick 0 only.** From
     tick 1 the canonical sums fold in the new id order and may round differently, which is
     legitimate. Even at tick 0 a payout split runs in `ActorId` order (the last recipient takes
     the remainder), so the test compares orders exactly and payout recipients as a set.
   - **The steps start at P0.3, not P0.2**, because this contract is P0.2.
7. Is the checkpoint boundary of §7.6 enough? A checkpoint's digest catches edits (of its identity
   and run's ledger too, since format 3) but not a forger, who can recompute it, and a frontend that
   depends on core directly can build a state with core's writer and resume it. Since P0.7 the
   engine alone offers no writer. The alternatives are a keyed digest, which needs a secret the
   engine does not have, or a resume that replays from genesis, which costs what a checkpoint saves.
