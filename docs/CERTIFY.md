# CERTIFY — the Phase 0 session 2 contract

Dated 2026-09-26. Branch `phase0-s2`, from `reboot` at `cf3c0ff`. Steps are `S2.n` (§14), one
commit each. It covers PLAN Phase 0 step 7 and §3.8; STATE next step 1 and O2; D10's items 1, 2 and
4 (GUI.md §7.2); REPORT §6's three criteria; ADDENDUM N4, N10, N12, N15 and A12; and R2–R5, R8, R9,
R12, R14 and R16. July's source is tag `july-v2-phase-3` (`v2p3:`). A step is done when
`scripts/gate.sh` is green in WSL and the same commands are green on Windows, with zero warnings.
This revision takes the adversarial review of the first draft (§15.2), and the bounded
verification's fixes (amended at S2.5).

**Amended at S2.2** (2026-09-26: the engine asks), where this text was wrong or silent. Each
change is made in place in the section named.

1. Steps (§14). This contract was committed as S2.1, so every later step is one number up: the
   engine asks are S2.2, certify S2.3, and so on to the review at S2.10.
2. `ScalePrice` and `MarketLine::trades` (§2.4) land with the kick in S2.3, not with the engine
   asks. The kick is their only user, and the variant is appended, so no hash waits on it.
3. `Site::per_tick` (§2.3) takes `(&Params, &Clock)` and returns `Result<f64, CoreError>`. Core
   cannot name agents' `View`, and a read can fail (a unit that does not match the method).
   `Site::value` and `Site::convert` serve the one rule that scales an annual rate before
   converting it, the cash rule's `share(v·μ^κ)`; `convert` applies the site's own method.
4. The contract said specs read a param "only through `Site::per_tick`" and named no guard. A
   source scan, `params_are_read_only_through_sites` (engine), enforces it: outside
   `core::clock`, no shipped code calls a `Clock` conversion or reads a param by type.
5. `each_site_converts_as_registered` (agents) compares each site with core's record,
   `ParamDef::sites`, and `ClockMethod::per_tick`, not with engine's `SiteLine`: agents cannot
   depend on the engine. `registry_names_each_use` pins each `SiteLine` to the same values, and
   to the `Clock`'s own conversions.
6. Silent points, decided: a `SetParam`'s target (`act.param`) is written, not read, so it is not
   a site; a param only a `SetParam` targets lists no sites. A shelf life is a `Ticks` site at
   the good's `life`. `Spec::sites(&World)` lists a spec's sites with their paths. The registry
   prints each site as `<method> <per tick> per tick at <path>`, joined by `; `.
7. Added tests: `clock_methods_convert_as_the_clock` and `param_sites_are_recorded` (core), and
   the scan of point 4.

**Amended at S2.3** (2026-09-26: certify and the kick), where this text was wrong or silent.
Each change is made in place in the section named.

1. **The kick bounds its peak** (§4, §6, §7, §14). `BatterySpec::Kick` gains `max_peak`, a
   `Ratio` above 1, and Kick passes only when every kick's `gain_peak` is at most it. The tail
   alone passed the desk-turnover ×16 cell. Its converged run (w×1.05, in `appb-freeze.ron`)
   rests at an unstable point: six of its eight 1e-9 kicks swing out ×2.9e9 through dead
   markets and come back to the point they left, where rounding freezes them again, so each
   `gain_tail` read 1e-6. The measurement review's own criterion was the peak: a kick whose gap
   passes the probe's tolerance, 1e-3 in log, is unstable. So appb registers `max_peak` 1e6,
   1e-3 over the 1e-9 kick; C2's eight kicks peak below 6.
2. **H is checked where it runs** (§6, §7). Kick records `kick.horizon`, the ticks the base
   continuation ran, per segment, with `ticks(horizon)` as its bar, and fails any other length.
3. **The freeze testdata** (§13; §15.1 question 5, answered). `appb-freeze.ron` is `appb-tape
   --set buffer.*=16 --perturb w*1.05`: a run the classifier scored CONVERGED at turnover ×16.
   It passes Settles and fails Kick. No hand-built tape is needed.
4. **The certificate records why a run stopped** (§8). `Certificate` gains `stopped:
   Option<String>`, the `RunError` that ended the run early. The seal's first failure line is
   built from it, so readback can seal it again.
5. **The seal fails any battery that failed** (§8), listed or not. An unscored run's
   Conservation counts too. §8 said "a listed battery"; this is stricter.
6. **Criteria for another tape are still scored** (§4, §8). `certify` fits and runs criteria
   whenever they are given. With another tape's `tape_hash` the verdict is UNSCORED at best, and
   any failed battery still fails it. §4 said the fit runs "when the `tape_hash` matches".
7. **An unscored run** (`Scoring::Unscored`) runs Conservation alone, and has no segments and no
   reports, since both need the criteria's bars (§6, §10).
8. **Names and readings** (§5, §6). `MarketObs` carries its node and good, so `trades()` is
   `MarketFill::trades` itself. `Names::of(&World)` gives each market as `node/good`. Every
   battery's readings, and every report, are named in §6.
9. **Reports poison a non-finite input** (§6). A report sample made from any non-finite input
   is NaN, so a non-finite input reaches the seal even where a fold would not carry it (a
   `D > 0` test with D NaN, say).
10. **Determinism compares hash tails too** (§6). Each resumed run's hashes are compared with
    the base run's from the resume tick, as well as its last report.
11. **Core's hasher is public** (§3, §9). `core::Fnv` (`new`, `write`, `finish`, `resume`) folds
    the manifest's hash lines; `resume` continues FNV-1a from a digest. So `Manifest` holds no
    hidden state, and a manifest read back continues folding.
12. **What landed early** (§13, §14). `appb-tape`'s `--perturb NAME`, `--ticks L`, `--set`,
    `--scale`, `--assign` and `--one-sided` (`probe::setup::TapeArgs`), the three testdata files
    and `certify_testdata_is_generated` land in S2.3, since S2.3's tests read the testdata. The
    probe's delegation to certify (C11) and its pins stay at S2.6.
13. **The transient test's dead ticks** (§13). By certify's definition, a tick with no trade,
    bcycle's b′ = 0.8 segment has no dead tick. It has the trough (the good's volume falls 88%)
    and the transfer shortfall. The dead tick is at the 0.8 → 0.2 change: one tick with no supply
    of the good and no consumption (REPORT §5). The probe's 125 dead ticks per 0.8 segment use
    `LIVE_FLOOR`, which is relative to the oracle and waits for `crates/observe` (§11).
14. **Public for tests.** `kick::kick_segment` is public, so a test can feed it another world's
    checkpoint. `RawCertificate` is `Serialize`, so a test can name the path of a number in it.
15. **Added tests**, beyond §13's: `folds_keep_a_nan`, `every_scalar_has_its_path`,
    `a_listed_battery_with_no_result_fails`, `a_kick_that_swings_out_and_back_fails`,
    `manifest_records_the_hash_stream_and_reads_back`, `a_resume_verifies_against_its_manifest`,
    `obs_reads_the_report` (certify) and `market_line_trades_as_markets_do` (engine).
16. **Recorded at S2.3.** No state hash, `prefix_id` or `world_id` moved: all 2,080 gate and
    20,000 appb per-tick hashes, and the four fixtures' `world_id`s, equal S2.2's on WSL and on
    Windows (finals `0x61f9c8529131ff17` and `0xe1fa082b26995867`). `scripts/gate.sh` passes 398
    tests on each machine. `cargo check --target wasm32-unknown-unknown -p rustyecon-certify`
    passes in WSL (recorded, not gated). Every new test was checked against its mutation: 83
    mutants, 82 killed. The one left, `!(gain_tail > max_gain)`, is equivalent: a NaN in the
    tail is in the peak too, which `gain_peak ≤ max_peak` fails; with both comparisons mutated
    the finite tests kill it.

**Amended at S2.4** (2026-09-26: telemetry, the cli, the criteria and the certificates), where
this text was wrong or silent. Each change is made in place in the section named.

1. **One step, three commits** (§14). The session merged the old S2.4 (telemetry), S2.5 (the
   cli), S2.7 (criteria) and S2.8 (results) into S2.4. It lands as three commits so that §14's
   order still holds: the code first; then the two criteria files alone, before any certified
   run of either tape; then the certificates, made by a clean build of the criteria commit. The
   probe's delegation (the old S2.6), the docs (S2.9) and the review (S2.10) come after.
2. **`finish` hands back what the manifest records** (§12). `TelemetryWriter::finish` returns
   `Finished { sink, rows, bytes, digest }`, not `(W, u64)`. The manifest pins the file by
   FNV-1a 64 over its bytes, and the writer is the one place that sees every byte, through a
   digesting sink; the cli cannot hash without core. `rows()` gives the rows so far.
3. **Telemetry, where §12 was silent.** A report's rows come in the order market, ration,
   settle, ledger, run, event. A ledger line's metric is `declared`, with its provenance in
   lower case as `tag`. `tick` is DELTA_BINARY_PACKED with no dictionary. The footer's keys are
   `rustyecon.telemetry`, `rustyecon.tape`, `rustyecon.tape_hash`, `rustyecon.world_id` and
   `rustyecon.build.commit`, `.dirty`, `.target` and `.rustc`. A report that names an id the
   world lacks is refused whole (`TelemetryError::Report`), since keys replace ids.
4. **The run line** (§10). Stdout's line is `run build <commit> <clean|dirty> <target> tape
   <name> tape_hash 0x… world_id 0x… from <tick>`, with ` resumed <digest>` on a resume, built by
   `Manifest::run_line` beside the header. `replay` prints it too, before its final hash: a hash
   a frontend shows names its run (R16).
5. **Resume's order** (§10). The extension, the tape, the decode and `Sim::resume` come first,
   so their refusals and messages stand. Then the manifest is read, then an `--out` that holds it
   is refused (exit 1), then `verify` runs. A resumed run's `genesis_hash` is the tape's genesis
   state's hash, computed again, not copied from the parent's manifest.
6. **When `certify`'s telemetry fails** (§10). A telemetry file that cannot be created is exit 3
   before the run. A write that fails during the run is exit 3 after it: the certificate, the
   manifest and the hash file are still written, the partial Parquet file is removed, and the
   manifest names no telemetry. When `certify` returns an error (a tape, a reserved key or
   criteria that do not fit), it removes the telemetry file it created. Every file `certify`
   writes goes through `<file>.tmp` and a rename, not the manifest alone.
7. **The build stamp** (§3). `build.rs` also watches the worktree's `.git` file, runs git with
   `--no-optional-locks` so that a stamp never rewrites the index, and lists untracked files
   one by one (`--untracked-files=all`). A commit that is not 40 hex digits is `unknown`, and an
   unknown commit is dirty.
8. **The Parquet-free gate** (§14) also checks that `parquet` is absent from certify's tree with
   the feature off, and the wasm32 check covers certify as well as the engine.
9. **What pure Rust costs, measured** (§12). By `cargo tree -e normal`, root and the workspace's
   crates included, certify's closure is 17 crates on Linux, Windows and wasm32 with the feature
   off, and 36 on Linux and 35 on Windows with it on: parquet adds 19 and 18. With build
   dependencies it adds two more (autocfg and version_check). No crate compiles C on either
   machine: `cc` is in `Cargo.lock` only for `iana-time-zone-haiku`, which builds for Haiku
   alone. The 19 on Linux: parquet, ahash, bytes, cfg-if, chrono, getrandom, half, hashbrown,
   iana-time-zone, libc, lz4_flex, num-bigint, num-integer, num-traits, once_cell, seq-macro,
   twox-hash, zerocopy and zerocopy-derive. The gate world's 2,080 ticks write 276,145 rows in
   1,527,113 bytes. Windows fetched the crates it lacked, lz4_flex 0.14.0 and num-bigint 0.5.1
   among them.
10. **Added tests**, beyond §13's: `telemetry_refuses_a_report_it_cannot_name` (certify), and
    `registered_criteria_load_and_fit` (certify `results.rs`): each registered file loads,
    fits its tape, and names the tape's current `tape_hash`, so a tape edited without a new
    dated criteria file fails a fast test, not only the ignored recompute.
11. **The committed certificates** (§14; the old S2.8). Built from `1ee9b80`, the criteria
    commit, clean, in WSL. Both verdicts are **PASS**, and every listed battery passes.
    - `results/gate`: 2,080 ticks, final `0x61f9c8529131ff17`, in five segments opened by
      `mine.cut`, `bread.line.up`, `oven.opens` and `mine.restored`. Runaway's widest ratios
      are 18.3 (town/bread) and 0.0996 (village/bread), inside [1e-3, 1e3]. Every market trades
      in each of the 40 yearly windows, and no market is pinned in any segment.
    - `results/appb`: 20,000 ticks, final `0xe1fa082b26995867`, one segment (no dated shock).
      Settles: no dead tick in W or F, and price and volume ranges in F at most 4.4e-16 in log.
      The eight kicks of 1e-9 decay to gains of 1.9e-6 to 6.0e-6 in the last tenth of the
      20,020-tick horizon, with peaks at most 3.26, against bars of 1e-3 and 1e6.
    - `certify` took 0.15 s for the gate and 2.65 s for appb, kicks included (WSL, release).
    - `committed_certificates_recompute` gives byte-equal certificates and manifests in WSL
      (gated) and on Windows (recorded).
    - CI checks out full history (`fetch-depth: 0`), since the gate checks that each committed
      certificate's build commit is an ancestor of HEAD.
12. **Recorded at S2.4.** No state hash, `prefix_id` or `world_id` moved. `scripts/gate.sh` is
    green in WSL and on Windows: 409 tests pass in the workspace on each machine, then certify's
    Parquet-free tests and `committed_certificates_recompute` by name. The Parquet-free certify
    checks for wasm32 in WSL. Telemetry is byte-identical from two processes on each machine,
    and WSL's and Windows' files are byte-identical up to the footer's key-value metadata, which
    names the build: the first byte that differs is 1,526,863 of 1,527,111, after every data
    page. Every new test was checked against the mutation it guards: 31 mutants of the code, the
    committed files and the tape, all killed by a test, and four of the build stamp and the
    ancestor check, killed by `gate.sh`'s steps. The logs are in
    `D:/rustyecon-s2/build-telemetry-cli/`.

**Amended at S2.5** (2026-09-26: the fix round of the bounded verification), where this text was
wrong or silent. The verification made one adversarial pass over S2.1–S2.4 and found two
blockers, nine majors and six minors; two of them repeat another (Settles' cleared volume, and
the seal's rule for a failed battery that is not listed). Every one is taken; none is rejected.
Each change is made in place in the section named.

1. **A tape that sets its own prices** (§2.4, §4, §8, §15.1 question 1; blocker, the
   verification's E1). July's step rule from w×2 runs away at tick 415. With 211 dated
   `ScalePrice` events that reset a price to genesis whenever it left [1/2, 2] of it, the run
   reached tick 1,000, every battery of the minimal legal criteria passed (Conservation,
   Determinism, Runaway), and the verdict was PASS. Tighter bands, down to [0.999, 1.001] with
   3,988 events, passed too. The loader refuses a recurring `ScalePrice` only, so §2.4's premise,
   that a price shock "cannot recur", was false: dense dated events are a periodic nudge in all
   but name. `Criteria` gains `price_shocks: Option<Count>`, the firings the run may carry, with
   a basis. `CriteriaRef` records the count it allows. The seal fails a scored run that fires
   more. The field is the one that may be absent, and its absence means none: the strictest bar,
   so a file written before it cannot loosen a verdict, and the registered files keep their text
   and their hash. An unscored run is UNSCORED at best, and its count is still reported. Tests:
   `a_tape_that_clamps_its_prices_does_not_pass` (E1 itself: FAIL on the count alone, FAIL with
   one shock fewer registered, PASS with all of them registered), and `price_shocks_are_counted`
   gains the scored case.
2. **The kick fires at every dated shock** (C5, §5, §6, §7; blocker, the verification's E4).
   C5 merges a shock closer than m to the last boundary, and the kick ran at segment ends only, so
   a regime shorter than m was never kicked. appb at desk turnover ×16 from its exact genesis
   rests at an unstable point. With both turnovers restored by dated events at tick 1,100, the
   segment closed, the kick at 1,100 ran under ×16 (the deferred tape keeps the restore away),
   and the run failed. At tick 1,000, below m = 1,040, the restore merged, the only kick ran at
   the end under the restored turnover, and 19 years at an unstable point certified PASS. The
   kick now fires at the run's end and at every distinct tick a dated event fires inside
   (0, until), merged or not, each under the deferred tape of that tick, so it probes the regime
   in force before the shock. C5's merge governs Settles, Balance and the reports only. A kick's
   readings name their tick (`home/good + at tick 20000`), with the segment whose regime they
   probe, and `kick.horizon` is per kick tick. `obs::kick_ticks` and `obs::segment_before` are
   public. The cost grows with the dated events; a tape with many pays for them. Test:
   `a_merged_shock_is_kicked`, on new testdata `appb-buffer16.ron` (`appb-tape --set
   buffer.*=16`, added to `certify_testdata_is_generated`), restored at 1,000 and at 1,100, over
   60 years in 20-year segments with a 40-year horizon: both FAIL at the shock's kick, whose
   peaks pass 1e6, and the end's kick passes.
3. **A pass flag holds its readings** (§8; major, the verification's E3 and mutant S8). A real
   FAIL certificate with Kick's pass flag flipped, its verdict set to PASS and its failures
   emptied read back PASS: the seal recomputed the verdict from the flags alone, and the module
   doc's "a verdict cannot be edited into a file that disagrees with its numbers" was not so.
   `Reading.bar` is now `Option<Limit>`: `AtMost`, `AtLeast`, `Above` and `Exactly` a bar, or
   `Ref`, a bar that Balance's compound pin rule reads. The seal fails a battery marked pass that
   holds a reading outside its limit, a reading of the finite gate (`nonfinite.tick`), or, for
   Balance, a (market, segment) its readings show pinned by `BalanceWatch::judge`'s rule. So a
   pass flag, a verdict or a failure list edited alone is refused. Readback still checks
   consistency, not truth: numbers and bars edited together read back, and only a rerun (C10)
   or the criteria file can tell. Kick gains `kick.errors`, per kick tick, the kicked runs that
   failed or had nothing to score, bar 0, so every Kick failure is a reading. `samples` has the
   limit "at least 1", and `settles.f_ticks` F's length. `render` prints each limit (`(bar <=
   0.001)`). Tests: `a_pass_flag_holds_its_readings` and `a_forged_pass_flag_is_refused`
   (certify, on synthetic parts; the second also drops Kick from a PASS certificate's criteria
   and results, which C12 refuses on readback), and `kick_check_fails_a_rounding_freeze`, which
   now forges its own certificate.
4. **Tests that were missing** (§13; majors and minors). Each was run against its mutant:
   - `a_dead_tick_is_one_market_silent` (a dead tick is some market silent, not all: mutant B9)
     and `cleared_volume_rests_in_f_too` (at rest is price and cleared volume: mutant B4, and a
     cleared volume that falls to zero in F). Settles' note now names only what moved;
   - `batteries_fail_closed_on_nonfinite_samples` gains a tick in W before F, for every field
     Settles reads (the gate over F alone);
   - `kick_checks_its_horizon` (mutant B13: amended at S2.3, item 2);
   - `edited_verdict_is_refused` gains six rules: an unscored run with a failed battery is FAIL
     (mutant S5: amended at S2.3, item 5), Settles without Kick (mutant S8), a price-shock count
     above and at its bar, and a battery marked pass with a reading outside its bar or a reading
     of the finite gate;
   - `criteria_need_a_unit_and_basis_for_every_bar` gains the empty bases of `until_basis`,
     `min_samples` and `price_shocks`, and a value out of range for every bar, `dead_share` 1
     and `run_share` 0 among them, each refused at its own path;
   - `resume_records_its_parent` (cli) resumes against a parent manifest whose `genesis_hash`
     was edited: it verifies, and the child records the tape's own genesis hash (amended at
     S2.4, item 5);
   - `certify_reads_the_whole_run` (certify, the gate under its registered criteria): 40 Trades
     windows, Balance for every market and segment, one resume reading per `resume_at`, and
     Conservation's largest margin the run's, not the first tick's (the wiring minors R7, W1, W3
     and W6, which only the committed certificates' byte equality caught);
   - `a_kick_moves_its_price_up_and_down` (certify): at the kick's tick the kicked price is
     fl(p·(1 + size)) or fl(p·(1 − size)) and no other price moves (a − kick made a + kick passed
     every fast test).
5. **The probe delegates** (C11, §11, §13, §14; major). The old S2.6 lands here. crates/probe
   depends on certify, Parquet-free. Its harness reads each tick through `certify::Obs` (the
   markets, a market that traded, spoilage per good, the provider's due and paid, the ledger
   margin) and calls certify's `within_bound`, `rationed`, `fold::trough`, `fold::range` and
   `dead_share_ok`. `protocol::DEAD_SHARE` names the 0.01 the classifier held as a literal.
   `probe_summaries_unchanged` (fast: w×1.05, r×1.05 and b′ = 0.8 from genesis) and
   `probe_battery_csv_unchanged` (ignored, by name in `gate.sh` on both machines: all 57 rows;
   the negative control's row, 57 DIVERGED with at most 337 dead ticks; the history's 611 dead
   ticks, 368 in W, 125 in each b′ = 0.8 segment and 116 in each return to 0.4) print what
   `docs/probe/results/` prints, at its precision. The ignored test costs about 15 s of CPU in
   release, 1.3 s on 16 threads, not the 40 s §11 guessed.
6. **The registered bars ran in tests before registration** (§13; major). §13 said that before
   the criteria were committed no test applied gate's or appb's registered bars, except appb's
   kick, and `runs.rs`'s header said the same. That was not so. From S2.3 (`0e8c2e7`), before the
   criteria commit `1ee9b80`, `unscored_run_never_passes`, `runaway_bound_is_relative`,
   `finite_scan_reads_every_rendered_number` and `render_prints_only_serialised_numbers` scored
   the gate with exactly its registered batteries (Determinism at 0.25, 0.5 and 0.75, Runaway
   1e3, Trades every year, Balance at 1e-9, 1e-12, 32 and 0.5, in one-year segments, with
   `rationed_below` 1e-9), and `unscored_run_never_passes` asserted PASS.
   `kick_check_passes_a_stable_rest`, `kick_check_fails_a_rounding_freeze`,
   `a_history_whose_segments_return_passes` and `a_failed_kick_is_a_fail_not_an_error` scored
   appb with its registered Runaway (1e6) and Settles (0.5, 0.9, 0.01, 1e-4), not only the kick.
   No value was tuned to a run. Each basis predates the session (ENGINE §10 and decision 19 for
   the gate's bound and Trades; July's `invariants.rs` for Balance; PROBE-SPEC §4.5 and §4.6 for
   Runaway 1e6, Settles and `rationed_below`; REPORT §5 for the kick), and no value moved between
   S2.3 and the registration. From S2.5 the gate's tests and the appb tests read their bars from
   the registered files, so a test and a file cannot drift apart, and
   `the_bars_are_the_registered_ones` checks the constants that restate them.
7. **The build stamp watches the reflog** (§3; minor). `build.rs` watched the loose ref of the
   branch `HEAD` names only where the file existed. For a branch whose ref was packed, a commit
   that touched neither `crates/` nor the index wrote a new loose ref that nothing watched, and
   the binary still named the parent commit. The script now also watches the worktree's
   `logs/HEAD`, which git appends to on every commit, amend, reset and checkout, and the common
   directory's `refs/heads` tree, which cargo scans whole.
8. **What records the criteria** (§3, §9; minor). The certificate records the criteria: file,
   date, hash and, from S2.5, the price shocks they allow. The manifest records the run's inputs:
   the build, `tape_hash`, `world_id` and any checkpoint it resumed from. §3 and three doc
   comments said the manifest recorded the criteria; they now say the above.
9. **A near-dead economy that trades** (§11, §15.1; minor). Trades counts any positive fill, so
   a run frozen at tiny positive volumes passes it and, at rest, Settles. Under gate-style
   criteria, which list no Kick, such a run certifies until crates/observe brings the
   oracle-relative dead floor (`LIVE_FLOOR`); until then its troughs are reported, not scored.
   Under appb-style criteria the kick catches the probe's second false-GO cell, all rates ×1/16
   at b′ = 0.2 (the verification's E2: Settles passes, and Kick fails with a tail gain of 2.0 and
   a peak of 7.5e8 at 20,000, 60,000 and 217,000 ticks). It is not testdata: the turnover ×16
   cells already fail Kick in the fast tests.
10. **Recorded at S2.5.** No engine, core, markets or agents code changed, so no state hash,
    `prefix_id` or `world_id` moved. The certificate's format did (`Limit`, `CriteriaRef`'s
    `price_shocks`, `kick.errors` and the kick's `at`), so the committed certificates are
    regenerated from a clean build of this step's code commit; the criteria files keep their text
    and hash. Every new test was checked against its mutation: 38 mutants of the fixes and the
    moved measures, all killed. 32 are killed by the tests named above. Six mutants of the probe's
    moved measures (the lower runaway bound, `trades()` always true, either half of the
    dead-share rule, spoilage's sign, a zero range) leave every pinned row of the report as it
    was, since no registered run's class turns on them, and certify's own suite kills each. Two
    (the trough's first minimum, the provider's due and paid swapped) are killed by the pins,
    which shows the probe reads certify's code. The build stamp was reproduced in WSL: on a packed
    branch ref, `4d59604`'s `build.rs` stamped the parent commit after `git commit`, and this
    one stamps the new commit.
    - **The certificates**, regenerated from `05533b9`, the code commit, clean, in WSL. Both
      verdicts are **PASS**, with the criteria hashes of S2.4. Every reading's name and value
      equals S2.4's; the one new reading is appb's `kick.errors` at tick 20,000, 0. Finals
      `0x61f9c8529131ff17` and `0xe1fa082b26995867`. `certify` took 0.15 s for the gate and
      2.65 s for appb.
    - **The gate.** `scripts/gate.sh` is green in WSL and on Windows at `05533b9` with the
      regenerated certificates: 421 tests pass in the workspace on each machine (409 at S2.4),
      2 ignored; 65 in certify's Parquet-free run, 1 ignored; then
      `committed_certificates_recompute` (byte-equal on both machines, gated in WSL) and
      `probe_battery_csv_unchanged` by name. Telemetry is byte-identical from two processes on
      each machine. The Parquet-free certify and the engine check for wasm32 in WSL.
    - The logs are in `D:/rustyecon-s2/fix-r1/`.

## 0. Decisions this contract makes

Numbered for the veto window, each with its alternative.

| # | Decision | Alternative |
|---|---|---|
| C1 | The kick is a tape event: a new action, `ScalePrice`, multiplies one posted price by a schedule param in phase 0 (§2.4). A kicked run resumes the base run's checkpoint in memory, under a kicked tape that equals the base before the kick. It therefore equals the kicked tape's run from genesis, and a test shows it. The kick is a tape edit with provenance (R16, E1), and it answers REPORT §7 question 4. | Certify writes a kicked checkpoint with core's writer: no engine change, but a state no tape made. |
| C2 | `tape_hash` is `fnv1a_64` over the canonical `to_ron` (§3), as GUI.md proposed. | Hash the file's bytes, which move with comments. |
| C3 | Criteria are inputs, registered before the run: `criteria/<tape>-<date>.ron`. Verdicts are outputs: `results/<tape>/certificate.ron` and `manifest.ron`, at stable paths so a rerun is a diff. Committed results are made without `--telemetry`. Hash files and telemetry are not committed; a manifest pins them by digest. | Criteria beside the results. |
| C4 | Three verdicts: PASS, FAIL and UNSCORED. FAIL outranks UNSCORED, which outranks PASS. Only PASS exits 0; the others exit 5 (N4). | UNSCORED as a kind of FAIL. |
| C5 | Windows are shares of a segment. A segment ends at a dated event only once it has lasted the registered `min_segment`; a shorter one runs on into the next. Recurring entries are not shocks. The kick's horizon is a span in years, not a share of a segment. The merge governs the windows only: the kick fires at every dated shock, merged or not, so no regime goes unkicked (amended at S2.5). | Every dated event a boundary, which lets dense events shrink every window. |
| C6 | Measures are oracle-free now; oracle-relative ones wait for `crates/observe` (§11). Certificates and manifests are RON: no JSON crate joins, and a NaN in RON is visible text. | Certify depends on the oracle; July's JSON. |
| C7 | Parquet 60.0.0 with feature `lz4` only: LZ4_RAW, byte-stream-split on the values, no zstd. Columns carry keys, not ids. The writer sits behind certify's feature `parquet`. | July's zstd, which builds C. |
| C8 | BalanceWatch is ported. July's `FracBelow`, `RollingCv`, `LevelRange`, `ResidualDamping`, `Damping` and `MeanAndSlope` are not: no committed criterion uses them, so R10 leaves them at the tag until one does. | Port them unused. |
| C9 | A cli resume is verified against a manifest of the same tape: equal `tape_hash` and `world_id`, and a record at the checkpoint's tick with an equal digest and state hash (R16's letter; D4). A resume under a dated edit waits for a ruling (§15.1, question 2). | Require the same world only. |
| C10 | `committed_certificates_recompute` is `#[ignore]`d and run by name, by `gate.sh` in WSL and by the same command on Windows. Both machines gate the verdict and every battery's pass flag. WSL also gates byte equality except `run.build`; Windows records it (decision 2). | Gate it in WSL only. |
| C11 | `crates/probe` delegates the moved measures to certify. Tests pin its summaries to all 57 rows of `battery.csv`, the negative control and the history run (landed at S2.5). | Freeze probe with its own copies. |
| C12 | Conservation, Determinism and Runaway are in every criteria file, and Settles comes only with Kick. A file that breaks this does not load. | Criteria choose freely, so a Settles-only file certifies a rounding freeze. |
| C13 | A one-sided tick is a BalanceWatch observation, at ±1. Only a tick with S = D = 0 is skipped, since its 0 is a convention. A market that stays one-sided is not clearing, and dropping those ticks would hide it (July's rule, `v2p3: invariants.rs:158-168`). | Two-sided ticks only, leaving one-sidedness to Trades. |

## 1. Crates, features, dependencies

```
crates/certify  rustyecon-certify  lib `certify`; depends on core, engine, ron, serde; optional parquet (S2.4)
  src/criteria.rs     Criteria, Bar, BatterySpec: RON text in and out, load checks, fit to a tape
  src/manifest.rs     Hex, Build, RunKey, tape_hash, Manifest, the named hashes header, verify
  src/obs.rs          Obs from (&TickReport, &Sim); Names; segments from the World's dated firings
  src/leaves.rs       crate-private: every scalar a value serialises, with its path (the finite scan)
  src/fold.rs         NaN-propagating max, min and sum; ln_range; the finite gate
  src/battery.rs      the batteries and reports, each pure over its inputs; BalanceWatch; trough
  src/kick.rs         deferred_tape, kicked_tape, kick_gain, the kick runs
  src/certificate.rs  Verdict, BatteryResult, Reading, Certificate; seal (crate-private); render;
                      nonfinite_paths; RawCertificate for readback
  src/run.rs          certify(): the base run, the batteries, the certificate and the manifest
  src/telemetry.rs    #[cfg(feature = "parquet")] TelemetryWriter<W: std::io::Write + Send>
```

- **Features.** `parquet = ["dep:parquet"]`, off by default, with `parquet = { version = "=60.0.0",
  default-features = false, features = ["lz4"] }` in the workspace. The cli turns it on; the GUI
  natively only, its web build using the rest.
- **No I/O.** No module but `telemetry.rs` names `parquet` or `std::io`, and none names `std::fs`,
  `env`, `process`, `net`, `thread` or `time`. Paths live in the cli (E2's rule, carried here).
- **No writer handed out.** Certify depends on core directly (`fnv1a_64`, `num`, `Basis`, `Date`)
  and calls none of core's writer. It re-exports nothing of core, and no public function returns a
  `Checkpoint`, `Sim` or `SimState` it built (checked by §13's scans).
- **The verdict path follows ENGINE §9:** `core::num::ln`, left folds in canonical order, no hashed
  containers, and no float literal but `0.0` and `1.0`, since every threshold is in criteria (R4).

## 2. Engine changes, before any manifest records a `world_id`

§2.1–§2.3 and §2.5 are S2.2; §2.4 is S2.3, with the kick (amended at S2.2).

### 2.1 D10 item 1: `world_id` without the `fixed` flag (ENGINE §2.6)

`world_id` hashes `(key, unit, genesis)` per param (`world_id` in `tape/mod.rs`), not `p.fixed`. Before this
change, a registered live param that became a `SetParam` source turned fixed and changed the world,
so every checkpoint was refused. The structure that `fixed` guards is hashed elsewhere: a shelf life
as ticks in the goods, the tolerances in `tol`, and a period in the schedule under `prefix_id`.
Test: `new_source_event_keeps_world_id` (engine `edits.rs`). The test builds a variant of the gate
tape with a 1780 event, `SetParam(param: "workers.buy.bread", to: "mill.capacity")`; after it,
`mill.capacity` is live and a source. `tapes/gate.ron` itself is unchanged. The test checks that
`world_id` is unchanged, that `prefix_id` is unchanged through 1780, and that the tick-1,040
checkpoint resumes under the edit and equals the edit's full rerun.

### 2.2 D10 item 2: `FiredEvent` names its source (ENGINE §7.4)

```rust
pub struct FiredEvent { pub key: Key, pub occurrence: u32, pub action: StateDelta<Agents>,
                        pub source: Option<Key> }  // Firing.source: a Key, since a ScheduleParam has no ParamId
```
Test: `fired_event_names_its_source` (engine `gate.rs`). `mine.cut` fires with
`Some("mine.capacity.cut")`, and `oven.opens` and every pension occurrence fire with `None`.

### 2.3 D10 item 4: each use of a param, with its `ClockMethod` (ENGINE §2.1, §2.6, §4, §6, §7.1)

```rust
// core::clock
pub enum ClockMethod { Value, Flow, Share, LogStep, Compound, Fraction, Weight, Ticks }
impl ClockMethod { pub fn unit(self) -> Unit;  pub fn name(self) -> &'static str;   // "log_step", …
                   pub fn per_tick(self, c: &Clock, v: f64) -> Result<f64, ClockError>; }  // §6's table
pub struct Site { pub param: ParamId, pub method: ClockMethod }   // what a resolved spec holds
impl Site { pub fn per_tick(&self, p: &Params, c: &Clock) -> Result<f64, CoreError>;  // the read at use
            pub fn value(&self, p: &Params) -> Result<f64, CoreError>;   // unit-checked (S2.2)
            pub fn convert(&self, c: &Clock, v: f64) -> Result<f64, CoreError>; }  // by its method
pub struct ParamSite { pub path: String, pub method: ClockMethod }
// core::tape::Resolver: the method replaces the unit, whose unit it implies
pub fn param(&mut self, key: &Key, method: ClockMethod, use_: ParamUse, field: &str) -> Result<Site, LoadError>;
// ParamDef and ScheduleParam gain `pub sites: Vec<ParamSite>`, in path order. A SetParam source
// takes its target's methods at the firing's path; a period takes Ticks.
// engine::registry
pub enum Entry { Param { unit: Unit, use_: Use, sites: Vec<SiteLine> }, Inline }
pub struct SiteLine { pub path: String, pub method: ClockMethod, pub per_tick: f64 }
```
The sites replace `per_tick`'s guess (`per_tick` and `is_price_rate` in `engine/src/registry.rs`).
Core's sites are the EMA (`Weight`), the tolerances (`Value`), a life (`Ticks`) and a price rate
(`LogStep`); agents' scripted sites are capacity, buy and sell (`Flow`) and payout and spend
(`Share`); each role site names the method its rule applies (RULES §2: a step rule's `up` and `down`
are `LogStep`, other rates `Share`). Resolved specs hold a `Site`, not a bare `ParamId`, and read it
only through `Site::per_tick` (or, for the cash rule's tilt, `Site::value` then `Site::convert`).
So the method a site declares is the conversion the run uses. `ClockMethod` and `Site` join the
engine prelude and the scans' allow-list. The registry's `sites` lists are not hashed; a `Site`
inside a spec is, with the spec. A `SetParam`'s target is not a site, and a shelf life is a
`Ticks` site at the good's `life` (amended at S2.2).
Tests: `registry_names_each_use` (engine): a gate variant sets the mill's `spend` to
`Some("rate.bread")` and deletes `mill.spend`, and the `rate.bread` row lists `log_step` at
`goods[bread].price_rate` and `share` at `actors[mill].spec.spend`, each per-tick value equal to
`Clock`'s. `role_sites_name_their_methods` (agents): appb under July's step rule, every
`RatePerYear` site against RULES §2. `each_site_converts_as_registered` (agents): on gate and
appb, the sites every spec holds are exactly the recorded ones under `actors[..]`, and each
per-tick value the run reads through the `View` equals its record's `ClockMethod::per_tick`, which
is what its `SiteLine` lists (amended at S2.2: agents cannot depend on the engine).
`params_are_read_only_through_sites` (engine scan), `param_sites_are_recorded` and
`clock_methods_convert_as_the_clock` (core) were added at S2.2.
Item 3 is met; S2.2 adds a step-of-7 loop to `observing_changes_no_hash` and records it.

### 2.4 The kick's action: `ScalePrice` (C1; ENGINE §2.4, §2.6, §7.3), in S2.3

```rust
RawAct::ScalePrice { node: Key, good: Key, by: Key }                // appended to RawAct
StateDelta::ScalePrice { node: NodeId, good: GoodId, factor: f64 }  // appended after AdvanceTick
```
`by` is a `Dimensionless` param only the schedule reads: a `ScheduleParam`, outside `world_id`, and
the firing's `source`. These are load errors: a factor that is not finite and positive, a currency,
a market the world lacks, and a `ScalePrice` in a recurring entry. The last keeps it a dated shock:
a periodic price nudge would be an exogenous stabiliser, which R3 bans, and by C5 it would not
restart a window. It does not keep dated shocks from being dense, which are a periodic nudge in
all but name (the verification's E1), so the verdict counts them: a scored run that fires more
`ScalePrice` events than its criteria's `price_shocks` allows, none when the field is absent,
fails (amended at S2.5). `apply` takes the delta in `Phase::Events` only (`WrongPhase` otherwise), sets the
posted price to `fl(p·factor)`, refuses a result that is not clean and positive, and leaves the
EMA alone. Hooks cannot emit it, since the whitelist is an allow-list. The variant is appended, so
no existing encoding, hash or `prefix_id` moves, and the schema stays 1 (P2.0.1's precedent; a
TAPE.md row). `MarketLine::trades()` joins the engine as markets' `MarketFill::trades` predicate,
so certify and probe share one definition of a market that traded. Certify calls it only after the
finite gate (§6).
Tests: `scale_price_is_an_event_only_delta` (core: the phase, a currency, a bad factor at load, a
recurring `ScalePrice` at load, a non-finite result); `kicked_tape_keeps_world_and_past` (engine:
gate plus a kick at tick 1,000 keeps `world_id` and `prefix_id(1000)`; the tick-1,000 checkpoint
resumes under it, that tick's `MarketLine.price` is `fl(p·factor)`, and the resumed run's hashes
equal the kicked tape's run from genesis); `behaviour_output_is_whitelisted` gains the arm.

### 2.5 Re-baselines

Every `world_id` changes once (gate, appb, core's and markets' fixtures), for two reasons. Item 1
drops the `fixed` flag, which is schedule structure, so that a new source keeps the world (E1). And
§2.3's `Site` puts each spec's method into the hashed actors and goods, since the method decides a
number. No state hash or `prefix_id` moves: S2.2 compares all 2,080 gate and 20,000 appb per-tick
hashes with `cf3c0ff`'s on both machines (finals `0x61f9c8529131ff17`, `0xe1fa082b26995867`). That
comparison also shows the `Site` refactor changed no conversion. STATE records the new `world_id`s.
Older checkpoints are refused (`WrongWorld`); none is committed.

Recorded at S2.2 (2026-09-26). Both streams are byte-identical to `cf3c0ff`'s on WSL and on
Windows, and the two machines agree. The `world_id`s, equal on both machines:

| World | At `cf3c0ff` | From S2.2 | Genesis state hash (unchanged) |
|---|---|---|---|
| `tapes/gate.ron` | `0xbecdc746fc86ce97` | `0x43628a8e0fd5f695` | `0xf05d0f23826edf87` |
| `tapes/appb.ron` | `0x3f689d670fe877c6` | `0x26f12f8a0bc27540` | `0x8d12ce44b614110a` |
| core's fixture | `0x66d1181c6802a7fd` | `0x85336968874fbf6d` | `0xf1538ab1f6a0de5c` |
| markets' fixture | `0xbdd0ee95c0bb590f` | `0xc3b1c948a42f06c6` | `0x5e400bb3f1012434` |

STATE takes them at S2.9, with the rest of the docs.

## 3. Identity: the tape hash, the build and the run key (N10)

```rust
pub struct Hex(pub u64);                               // serialised as "0x%016x"
pub fn tape_hash(t: &Tape) -> u64;                     // core::fnv1a_64(t.to_ron().as_bytes())
pub struct Build { pub commit: String, pub dirty: bool, pub target: String, pub rustc: String }
pub struct RunKey { pub build: Build, pub tape_hash: Hex, pub world_id: Hex }   // the GUI reuses it
```
- **N10, confirmed.** `Tape` is `deny_unknown_fields` with no defaults, so the parsed `Tape` is
  everything the loader reads, and `to_ron` writes all of it in canonical order: the name, every
  basis, the schedule and the inline numbers. So `tape_hash` covers all that `world_id` and every
  `prefix_id` cover, and ignores only what the loader ignores (comments, whitespace, CRLF, list
  order). The other inputs a run reads are recorded beside it: the manifest records the build and
  a resumed checkpoint (its `digest`, over `world_id`, `prefix_id`, state and run), and the
  certificate records the criteria (their file, date and `hash`) (amended at S2.5).
  The hash is tied to ron 0.8.1's text; a ron upgrade that changes it re-baselines every
  `tape_hash`, as a dated entry.
- **Build.** The cli's `build.rs` stamps `RUSTYECON_COMMIT` (40 hex), `RUSTYECON_DIRTY`,
  `RUSTYECON_TARGET` (cargo's `TARGET`) and `RUSTYECON_RUSTC` (`$RUSTC -V`). Dirty means `git status
  --porcelain` is non-empty over `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` and
  `clippy.toml`, untracked files included. The script emits `rerun-if-changed` for each of those
  paths; for the worktree's `HEAD`, `index` and `logs/HEAD` (from `git rev-parse --git-dir`); and
  for the ref that `HEAD` names, `packed-refs` and the `refs/heads` tree (from
  `--git-common-dir`), each where it exists. So a commit, a checkout, a staged change and an edit
  to any covered path all restamp, on a branch whose ref is packed too (amended at S2.5). In WSL this worktree's
  `.git` names a Windows path (`gitdir: C:/…`) and git fails, so the script maps `X:/` to `/mnt/x/`
  and retries (checked: it works and reads clean). With no readable git the commit is `unknown`
  and dirty is true (fail closed). `gate.sh` checks the stamp against the checkout (§14).

## 4. Criteria: dated, registered, one file per tape

Every threshold is a `Bar` with a value, a unit and a basis (R4's five tags, core's `Basis`).
Unknown fields are refused, and nothing has a default but `price_shocks`, whose absence is the
strictest bar, none (amended at S2.5). A criteria file names the tape it was
registered for, why it exists, and the file it replaces.

```rust
pub struct Criteria { pub format: u32 /* 1 */, pub date: Date, pub reason: String,
    pub supersedes: Option<String>, pub tape: TapeRef, pub until: Date, pub until_basis: Basis,
    pub min_segment: Bar /* Years */,
    pub price_shocks: Option<Count>,          // ScalePrice firings allowed; absent, none (S2.5)
    pub batteries: Vec<BatterySpec>, pub reports: ReportSpec }
pub struct TapeRef { pub name: String, pub tape_hash: Hex }
pub struct Bar { pub value: f64, pub unit: BarUnit, pub basis: Basis }
pub struct Count { pub value: u64, pub basis: Basis }
pub enum BarUnit { Ratio, LogWidth, Share, Relative, Imbalance, Years, Gain }
pub enum BatterySpec {
    Conservation,                                      // no bar: margin <= 1 is the ledger's definition
    Determinism { resume_at: Vec<Bar> },               // Share of the run, open (0, 1)
    Runaway { bound: Bar },                            // Ratio to the genesis price, > 1 (A12)
    Trades { every: Bar },                             // Years
    Balance { level: Bar, spread: Bar, min_samples: Count, run_share: Bar },   // Imbalance, Share
    Settles { w_from: Bar, f_from: Bar, dead_share: Bar, band: Bar },          // Share, LogWidth
    Kick { size: Bar, horizon: Bar, tail: Bar, max_gain: Bar,    // Relative, Years, Share, Gain
           max_peak: Bar },                                    // Ratio (amended at S2.3)
}
pub struct ReportSpec { pub rationed_below: Bar }      // Relative: a fill below 1 − this is rationed
impl Criteria { pub fn from_ron(s: &str, file: &str) -> Result<Criteria, CriteriaError>;
                pub fn fit(&self, clock: &Clock) -> Result<Fit, CriteriaError>;   // ticks, checked
                pub fn to_ron(&self) -> String;  pub fn hash(&self) -> u64; }  // fnv1a_64 of to_ron
```
**Load checks** (`from_ron`), each a `CriteriaError` with its path:
- every value is finite and in its field's unit: a `Share` in [0, 1], with `w_from ≤ f_from < 1`,
  `tail` and `run_share` in (0, 1], and `dead_share` below 1; each `resume_at` in the open (0, 1);
  a `Relative` in (0, 1); a `Ratio` bound or peak above 1; a `Gain` in the open (0, 1); an `Imbalance` in
  (0, 1); a span in `Years` above 0;
- the kick's size moves a float: `fl(1 − size) < 1 < fl(1 + size)`;
- `min_samples` is at least 2, since a spread needs two samples;
- every basis says something: each bar's, `until_basis`, `min_samples`' and `price_shocks'`;
- Conservation, Determinism and Runaway are listed; Settles and Kick are listed together or not at
  all (C12); no battery is listed twice; `resume_at` is not empty;
- `format` is 1; `date` equals the file name's date; `supersedes`, when given, names an earlier
  file for the same tape name. A retune is a new dated file with its reason, never an edit.

**Fit checks** (`fit`, run by certify against the tape's clock whenever criteria are given, for
another tape's `tape_hash` too, which then seals UNSCORED at best (amended at S2.3); a
failure is exit 1 and nothing runs). With n = `tick_of(until)` ticks in the run and
m = `ticks(min_segment)`:
- n ≥ m, and n ≥ `ticks(every)`, so the run holds a full segment and a full Trades window;
- the resume ticks ⌊share·n⌋ are distinct and lie strictly inside (0, n);
- H = `ticks(horizon)` ≥ 2, and the tail ⌈tail·H⌉ ≥ 2;
- m ≥ `min_samples`, so a segment can hold enough samples to find a pin;
- ⌊(1 − f_from)·m⌋ ≥ 2, so F holds two ticks in the shortest segment.

An excerpt of `criteria/appb-2026-09-26.ron`:
```ron
Criteria(format: 1, date: "2026-09-26", reason: "first registration, before any certified run of appb",
  supersedes: None, tape: (name: "appb", tape_hash: "0x…"), until: "2134-08-14",
  until_basis: Literature("docs/probe/RULES.md §3: L = 20,000 ticks, 200 τ_max"),
  min_segment: (value: 20.0, unit: Years, basis: Assumed("about 15 τ_max of RULES §3's 68 ticks; REPORT §3: the slowest return took 783 ticks")),
  batteries: [Conservation, Determinism(resume_at: [(value: 0.5, unit: Share, basis: Assumed("mid-run"))]),
    Runaway(bound: (value: 1e6, unit: Ratio, basis: Literature("PROBE-SPEC §4.5; REPORT §6"))),
    Kick(size: (value: 1e-9, unit: Relative, basis: Literature("docs/probe/REPORT.md §5–§6")),
         horizon: (value: 385.0, unit: Years, basis: Literature("RULES §3: one L, 20,020 ticks at 52 a year")),
         tail: (value: 0.1, unit: Share, basis: Assumed("the horizon's last tenth")),
         max_gain: (value: 1e-3, unit: Gain, basis: Assumed("REPORT §3, §5: a decaying 1e-9 kick ends near its rounding floor, a gain near 1e-6; a neutral one stays near 1")),
         max_peak: (value: 1e6, unit: Ratio, basis: Literature("REPORT §5; the measurement review's kick check: a gap past the probe's tolerance, 1e-3 in log, is unstable"))), …],
  reports: (rationed_below: (value: 1e-9, unit: Relative, basis: Literature("PROBE-SPEC §4.6"))))
```

## 5. Observations, segments and windows (N15; REPORT §6, criterion 3)

```rust
pub struct Obs { pub tick: u64, pub markets: Vec<MarketObs>,   // the report's (node, good) order
    pub rationing: Vec<RationObs>,                             // the report's (node, good, class, side) lines
    pub consumed: Vec<f64>, pub spoiled: Vec<f64>,             // per good: −Σ Consumption, −Σ Spoilage lines
    pub transfers: Vec<(ActorId, f64, f64)>,                   // each Provider's (due, paid)
    pub margin: f64, pub run_margin: f64,                      // audit.max_margin, run.max_margin
    pub price_shocks: u32 }                                    // ScalePrice firings this tick
pub struct MarketObs { pub node: NodeId, pub good: GoodId,     // node and good: S2.3
    pub price: f64, pub next_price: f64, pub supply: f64, pub demand: f64,
    pub cleared: f64, pub buyer_fill: f64, pub seller_fill: f64 }
impl MarketObs { pub fn trades(&self) -> bool; }               // markets' predicate, after the finite gate
pub struct RationObs { pub market: u32, pub class: ClassId, pub side: SideTag,
    pub requested: f64, pub feasible: f64, pub filled: f64 }
impl Obs { pub fn of(r: &TickReport, sim: &Sim) -> Obs; }      // after each step: the report, and Providers' state
pub struct Segment { pub from: u64, pub to: u64, pub opened_by: Vec<Key>, pub merged: Vec<Key> }
pub fn segments(w: &World, from: u64, until: u64, min_len: u64) -> Vec<Segment>;
```
The candidate boundaries are the distinct ticks of `w.schedule.once()` inside `(from, until)`, in
order. A candidate b closes the current segment `[a, b)` only if b − a ≥ m = `ticks(min_segment)`.
Otherwise its event keys join the segment's `merged` list, and the segment runs on. At the end, a
last segment shorter than m joins the one before it. So every segment has at least m ticks, and
the certificate names each shock that fell inside one. The report of a closing shock's tick opens
the next segment. The merge sets the windows of Settles and Balance and the reports' spans only:
the kick fires at every dated shock, merged or not (§7; amended at S2.5). In a segment of n ticks, W is `[from + ⌈w_from·n⌉, to)` and F is
`[from + ⌈f_from·n⌉, to)`. No window is an absolute tick (N15: July's `analysis_end: 1000` scored
ticks 150–1000 of an 11,700-tick run). No window is set by the tape's event spacing either: with a
no-op event every 100 ticks, the segments are still m long.

## 6. Batteries and reports: what each reads

Conservation, Determinism and Runaway run on every scored run; Trades, Balance, and Settles with
Kick run when listed (C12). Each battery is a pure function of its inputs, and `certify` only
gathers them: `conservation(&[Obs])`, `determinism(&DeterminismInputs)`, `runaway(&[Obs],
genesis)`, `trades(&[Obs], &[Window])`, `balance(&[Obs], &[Segment], …)`, `settles(…)` and
`kick_gain(base, kicked, tail)`. So each has a test that fails it (§13).

- **The finite gate (N12).** Before any predicate, every field in a battery's "Reads" column is
  checked with `is_finite`, on every tick, market and run it scores. The first failure fails the
  battery, and its note names the tick and the path (`markets[town/bread].supply`). After the
  gate, Balance skips a tick only when `s == 0.0 && d == 0.0`; `trades()` sees finite fields only;
  every max and min over a series propagates NaN (`fold::max_nan`); and every comparison is written
  `x <= bar`, never `!(x > bar)`. This closes the four leaks of the first draft: Balance's `S > 0
  or D > 0` test with S = NaN, `trades()` reading a NaN tick as "no trade", `fold(0.0, f64::max)`,
  and ±inf past an `is_nan` guard.
- **No vacuous pass.** Every battery records at least one reading for each thing it scores: each
  (market, segment), each market, each window, each resume, each kick. A battery with none FAILs
  with `no samples`. A world with no non-currency market fails every per-market battery.

| Battery | Reads (the finite gate covers these) | Passes when |
|---|---|---|
| Conservation (R2) | each tick's `margin` and `run_margin`; whether the run reached `until` | the run reached `until`, and every margin is ≤ 1. The engine already stops on a breach; this is a second reader of the same numbers, and a test fails it (§13) |
| Determinism (R8) | the base run's `TickReport.hash`; a second `Sim::new` run's; `audit_replay(tape, until)`'s final hash; for each `resume_at`, the base run's checkpoint at ⌊share·n⌋ through `to_bytes`/`from_bytes` and `Sim::resume`, and every f64 of its last report and the base run's | the hash streams are equal tick by tick, the replay returns the final hash, and each resumed run's last report equals the base run's in full |
| Runaway (A12) | per market, `next_price` on every tick, and the genesis price (`Sim::price` before the first step) | every ratio lies in [1/bound, bound] (it does not stop the run) |
| Trades | per market, `supply`, `demand`, `buyer_fill` and `seller_fill`, then `trades()` | each market trades in every window. The windows are `[k·E, (k+1)·E)` from the start with E = `ticks(every)`, plus `[n − E, n)` when E does not divide n, so the last full window ends at `until`. Trades does not restart at shocks: it is a liveness check over the run |
| Balance (BalanceWatch) | per market, `supply` and `demand`, then `markets::imbalance`; per segment | in every segment, every market has at least `min_samples` observations and is not pinned. A tick is an observation unless S = D = 0; a one-sided tick counts at ±1 (C13). A market is pinned when sd < `spread` and \|mean\| > `level`, or when an unbroken run of one exact value v, \|v\| > `level`, covers at least `run_share` of its observations. Too few observations FAILs as "too few samples": it cannot tell a pin from a quiet market |
| Settles (REPORT §6, criteria 1 and 3) | per segment and market: `supply`, `demand`, the fills, `price` and `cleared` | a dead tick is one on which some market does not trade. In W, dead ticks ≤ ⌊dead_share·\|W\|⌋; in F, none; for every market, `ln_range` of price and of cleared over F is ≤ `band` |
| Kick (REPORT §6, criterion 1) | per kick tick (the run's end and every dated shock; amended at S2.5) and market, the `price` of every market in the base continuation and each kicked run, over the horizon | the realized kick is not zero, every kick's `gain_tail` ≤ `max_gain` and `gain_peak` ≤ `max_peak` (amended at S2.3), the base continuation ran H ticks, and no kicked or base run fails (§7) |

The readings (amended at S2.3), each at its market (`node/good`), market and sign (`home/good +`)
or `run`, per segment where the battery has segments: `conservation.reached`, `.max_margin` and
`.max_run_margin`; `determinism.repeat`, `.replay` and, per resume, `.resume` (counts of
differing hashes and report fields, bar 0); `runaway.max_ratio` and `.min_ratio`;
`trades.windows` and, per market, `trades.silent_windows`; `balance.samples`, `.mean`, `.sd`,
`.longest_run` (its bar `run_share` of the observations) and `.run_value`; `settles.dead_w`,
`.dead_f`, `.price_range` and `.cleared_range`; per kick tick `kick.horizon` and
`kick.errors`, and per kick `kick.size`, `.gain_tail` and `.gain_peak`, at `node/good ± at tick T`
(amended at S2.5). Each bar carries its comparison, a `Limit` (§8). A battery the finite gate fails records `nonfinite.tick`, and one
with nothing to score records `samples`, 0.

**Reports** are computed per segment and per market, good or class, and never scored. They are
REPORT §6's criterion 2, the transient statistics, and O14's first instrument:
- troughs: the minimum `cleared`, its tick, and the `cleared` at the segment's end;
- dead ticks per market: ticks with no trade, and among them the ticks with S = 0 and with D = 0;
- fills, over ticks where the side exists: the worst buyer fill over ticks with D > 0, the worst
  seller fill over ticks with S > 0, and the ticks on which each is below 1 − `rationed_below`.
  Markets set a fill to 0 when its side is empty (`clearing.rs:82-83`), so a dead tick is not
  counted as rationing;
- rationing per (market, class, side), from `TickReport.rationing` (R12): the worst
  filled/feasible over lines with feasible > 0, its ticks below 1 − `rationed_below`, Σ(requested −
  feasible), which is budget rationing (N7) and never shows in a fill, and Σ(feasible − filled);
- for each good consumed anywhere, the ticks with no `Consumption`; Σ spoiled per good;
- each provider's Σ(due − paid) and its short ticks;
- ticks with a subnormal fill or `cleared` (P0.4 amendment 6);
- the run's count of `ScalePrice` firings, so a tape that sets its own prices says so.

Reports have no gate. They fold with the same NaN-propagating max, min and sum, and a sample
made from any non-finite input is NaN (amended at S2.3), so a non-finite input reaches the
certificate and the seal fails it (§8). No report is a ratio that could be 0/0. Their names, per
segment: `trough.cleared`, `.tick` and `.end`; `dead.ticks`, `.no_supply` and `.no_demand`;
`fill.worst_buyer`, `.worst_seller`, `.rationed_buyer_ticks`, `.rationed_seller_ticks` and
`.subnormal_ticks`, per market; `ration.worst`, `.rationed_ticks`, `.budget_short` and
`.market_short`, at `node/good/class/side`; `consumption.none_ticks` and `spoiled.total`, per good;
`transfer.short` and `.short_ticks`, per provider. The run's `ScalePrice` count is the
certificate's `price_shocks`. An unscored run has no reports (amended at S2.3).

## 7. The kick check (C1; REPORT §5, and §6 criterion 1)

At each kick tick T, with H = `ticks(horizon)` from the criteria, the same for every T. The kick
ticks are the run's end and every distinct tick a dated event fires inside (0, until), whether or
not it closes a segment (amended at S2.5: a regime shorter than `min_segment` merged into its
segment and went unkicked, the verification's E4). A kick at T probes the regime in force before
T, and its readings belong to the segment that holds tick T − 1:
1. The base run keeps `sim.checkpoint()` at `T`, in memory.
2. `deferred_tape(base, T, T + H)` re-dates every dated event at a tick ≥ `T` to
   `clock.date_of(T + H)`, a tick no kicked run reaches, and changes nothing else. Every param
   keeps every reference, so the registry and `world_id` are unchanged, and so is `prefix_id(T)`,
   which covers firings before `T`. Certify asserts both; a mismatch fails Kick as uncomputable.
   Recurring entries stay. (The first draft dropped the events and their params instead. That
   moved `world_id` whenever a registered param's only reference was a dropped `SetParam`
   target, since the loader marks such a param live and registers it.)
3. For each market and sign, `kicked_tape(deferred, T, node, good, 1 ± size, basis)` adds a
   `ScheduleParam` `certify.kick.factor` (its basis cites the criteria file and REPORT §5) and an
   event `certify.kick` at `clock.date_of(T)` doing `ScalePrice(node, good, by:
   "certify.kick.factor")`. Keys under `certify.` are reserved: a base tape that uses one does not
   certify (exit 1). If `tick_of(date_of(T)) ≠ T`, the kick is uncomputable and the battery fails.
4. The base continuation (the deferred tape) and each kicked run resume the checkpoint for H
   ticks. Let g(t) be the NaN-propagating max over markets of |ln(p_kick(t)/p_base(t))| on
   `MarketLine.price`. The realized kick is g(T), from the kicked run's first report; g(T) = 0
   fails as a zero kick. The readings are `gain_tail`, the max of g(t)/g(T) over the last
   ⌈tail·H⌉ ticks, and `gain_peak`, the max over all H, which `max_peak` bounds (amended at
   S2.3: a kick that swings out and back reads as decayed in its tail). `kick_gain` is a pure function of the two
   price series. The notes give each kicked tape's `tape_hash`; a rerun of `certify` reruns them.

Every failure after the base tape loads is inside the certificate: a kicked tape that does not
load, a refused resume, a failed kicked or base run. Each fails Kick with a note. `certify` returns
an error only for the base tape, a reserved key or the criteria.

Why (REPORT §5): `Imbalance` freezes once |k·x| < 2⁻⁵³, so a trajectory can rest at an unstable
point and look settled. Two sweep cells scored GO that way. Scoring the tail against the realized
kick makes the rule robust: a decaying kick ends near its rounding floor, a neutral direction near
1, and a growing one above it; one tick near zero on an orbit does not pass. The resumes are
certify's own, in memory; the cli's resume rule (R16, C9) binds frontends. For appb the check costs
8 × 20,020 ticks plus the 20,020-tick base continuation: about 3 s, at 0.3 s per 20,000 ticks.

## 8. The certificate: verdict-first, fail-closed, persisted (R9; N4, N12)

```rust
pub enum Verdict { Pass, Fail, Unscored }
pub enum BatteryId { Conservation, Determinism, Runaway, Trades, Balance, Settles, Kick }
pub struct Reading { pub name: String, pub at: String /* "town/bread", "bread", "provider", "run" */,
                     pub segment: Option<u32>, pub value: f64, pub bar: Option<Limit> }
pub enum Limit { AtMost(f64), AtLeast(f64), Above(f64), Exactly(f64), Ref(f64) }   // S2.5
pub struct BatteryResult { pub id: BatteryId, pub pass: bool, pub readings: Vec<Reading>, pub notes: Vec<String> }
pub struct CriteriaRef { pub file: String /* base name */, pub date: Date, pub hash: Hex,
                         pub tape_hash: Hex, pub listed: Vec<BatteryId>,
                         pub price_shocks: u64 }   // the ScalePrice firings allowed (S2.5)
#[serde(try_from = "RawCertificate")]
pub struct Certificate { verdict: Verdict, failures: Vec<String>, run: RunKey, tape: String,
    criteria: Option<CriteriaRef>, until: u64, reached: u64,
    stopped: Option<String>,                           // the RunError, if the run stopped (S2.3)
    genesis_hash: Hex, final_hash: Hex,
    price_shocks: u64, segments: Vec<Segment>, batteries: Vec<BatteryResult>, reports: Vec<Reading>,
    nonfinite: Vec<String> }
impl Certificate { /* getters */ pub fn render(&self) -> String; pub fn to_ron(&self) -> String;
                   pub fn from_ron(s: &str) -> Result<Certificate, CertificateError>; }
pub fn nonfinite_paths<T: Serialize>(v: &T) -> Vec<String>;
pub enum Scoring<'a> { Criteria { criteria: &'a Criteria, file: &'a str }, Unscored { until: u64 } }
pub struct Certified { pub certificate: Certificate, pub manifest: Manifest, pub hashes: String }
pub fn certify(tape: &Tape, scoring: Scoring<'_>, build: &Build,
               on_tick: &mut dyn FnMut(&TickReport)) -> Result<Certified, CertifyError>;
```
- **The seal.** A crate-private `seal(parts) -> Certificate` makes every certificate; `certify`
  calls it, and so does readback. `parts` is the certificate without `verdict`, `failures` and
  `nonfinite`. Every listed battery has a result; one that did not run has `pass: false` and the
  note "did not run". The seal computes `nonfinite` and the failures, then the verdict:
  - **FAIL** if any f64 anywhere in the parts (reading values, bars, reports) is non-finite, each
    path a failure line (R9: NaN fails, with or without criteria); or `reached < until` (a
    `RunError`), its ledger line first; or the criteria lack a battery C12 requires; or a listed
    battery has no result, or any battery did not pass (amended at S2.3: listed or not); or a
    battery marked pass holds a reading outside its `Limit`, a reading of the finite gate, or,
    for Balance, a pinned (market, segment); or, with criteria, the run fired more `ScalePrice`
    events than they allow (amended at S2.5);
  - else **UNSCORED** if there are no criteria, or criteria for another `tape_hash` (N4; July
    certified PASS here, `v2p3: runner.rs:473-478`, `certificate.rs:78`);
  - else **PASS**. UNSCORED is never PASS. Every number a verdict reads is a `Reading`; notes are
    text, and no verdict reads them.
- **The finite scan (N12).** `nonfinite_paths` is a serde `Serializer` that visits every `f64` the
  certificate serialises and returns the path of each non-finite one. July's scan read state
  fields, not the numbers its certificate rendered, and its statistics dropped NaN one sample at a
  time (`v2p3: verdict.rs:104-106, 324-338`; `invariants.rs:167`).
- **Verdict-first.** `render()` opens `VERDICT: <v>  <tape>  tape_hash …  world_id …  build <commit>
  <clean|dirty> <target>`, then the failures, then a line per battery. It prints serialised fields
  only, each f64 in Rust's shortest round-trip form, and does no arithmetic: no ratio, percentage
  or difference the scan could not see. The RON opens with `verdict`.
- **Reading back.** Every path goes through `RawCertificate`, so `ron::from_str::<Certificate>`
  runs the same checks as `from_ron`. A non-finite number that `nonfinite` does not list is
  `NonFinite { path }`. The parts are then sealed again, and a verdict, failure list or
  `nonfinite` list that differs is `EditedVerdict`. `CriteriaRef` carries the criteria's
  `tape_hash`, listed batteries and allowed price shocks, so every sealing rule can be
  recomputed, and a pass flag edited alone disagrees with its readings (amended at S2.5).
  Readback checks consistency, not truth: numbers and bars edited together read back, and the
  truth is a rerun (C10).

The RON opens `Certificate(verdict: Pass, failures: [], run: (build: (commit: "…", dirty: false,
target: "x86_64-unknown-linux-gnu", rustc: "rustc 1.…"), tape_hash: "0x…", world_id: "0x…"), tape:
"appb", …`, and a reading is `(name: "kick.gain_tail", at: "home/good +", segment: Some(0), value:
…, bar: Some(AtMost(0.001)))` (amended at S2.5).

## 9. The manifest (N10; ENGINE §7.6; D4)

```rust
pub struct Manifest { pub format: u32, pub run: RunKey, pub tape: String, pub genesis_hash: Hex,
    pub start: Date, pub ticks_per_year: u32, pub from: u64, pub until: u64,
    pub resumed_from: Option<ResumedFrom>, pub hashes: HashRecord,
    pub checkpoints: Vec<CheckpointRecord>, pub telemetry: Option<TelemetryRecord> }
pub struct ResumedFrom { pub tick: u64, pub digest: Hex, pub state_hash: Hex, pub parent: RunKey }
pub struct HashRecord { pub count: u64, pub last: Option<(u64, Hex)>, pub digest: Hex }
pub struct CheckpointRecord { pub tick: u64, pub state_hash: Hex, pub digest: Hex, pub file: String }
pub struct TelemetryRecord { pub file: String, pub rows: u64, pub digest: Hex }
impl Manifest {
    pub fn begin(run: RunKey, genesis_hash: u64, sim: &Sim, resumed_from: Option<ResumedFrom>) -> Manifest;
    pub fn tick(&mut self, tick: u64, hash: u64);          // folds "{t} 0x{hash:016x}\n" into digest
    pub fn checkpoint(&mut self, cp: &Checkpoint, file: &str) -> Result<(), ManifestError>;
    pub fn hashes_header(&self) -> String;                  // the `#` lines of §10
    pub fn verify(&self, cp: &Checkpoint, tape_hash: u64) -> Result<ResumedFrom, VerifyError>;
    pub fn to_ron(&self) -> String;  pub fn from_ron(s: &str) -> Result<Manifest, ManifestError>; }
```
- **Files** are recorded relative to the manifest's directory, with `/` separators, so a manifest
  made on Windows and one made in WSL agree.
- **`hashes.digest`** is FNV-1a 64 over the hash file's body, so `grep -v '^#' | fnv` checks it.
- **`checkpoint`** records a checkpoint of this run only. It requires the run's `world_id`, and
  `state_hash(cp.state())` equal to the last hash `tick` folded (the `TickReport.hash` of tick − 1),
  or to the starting state's hash when no tick has run. Otherwise it refuses (`OffRun`). So a
  record is the run's hash for that tick, not the checkpoint compared with itself.
- **`verify`** (D4, R16) requires `run.tape_hash` equal to the resuming tape's, `run.world_id`
  equal to the checkpoint's, and a record at its tick with an equal `digest` and state hash. It
  returns the `ResumedFrom` the new run records. It catches mix-ups: a checkpoint of another tape,
  world, run or tick, or a file replaced since. It does not catch forgery, since `manifest.ron` is
  as easy to rewrite as the digest (ENGINE §7.6, §14 question 7). The only proof of a run is
  `audit_replay` from genesis.
- The module needs no Parquet and no I/O, so the web build can use it.

## 10. The cli (R16; ENGINE §8)

```
rustyecon run      <tape> --until T [--out DIR] [--checkpoint-every N] [--format bin|ron] [--hashes FILE]
rustyecon resume   <cp> --tape <tape> --until T [--manifest FILE] [same options]
rustyecon replay   <tape> --until T
rustyecon registry <tape>                                  # one line per param, its sites listed
rustyecon certify  <tape> (--criteria FILE | --until T) --out DIR [--telemetry]
```
- **What resume lacks.** Confirmed at `cf3c0ff`: resume decodes format 3 and checks its digest
  over `(world_id, prefix_id, state, run)`; `Sim::resume` then checks `world_id`, `prefix_id` and
  shapes. What is left is R16's "a run of the same tape": today a checkpoint resumes under any tape
  with the same world and past, and nothing records which run made it.
- **Resume verifies.** After the decode and `Sim::resume`, whose refusals and messages stand, it
  reads `manifest.ron` beside the checkpoint, or `--manifest FILE`, and calls `Manifest::verify`
  with the tape's `tape_hash`. A missing, unreadable or disagreeing manifest is exit 3
  ("unverified checkpoint: <why>"), and so is a manifest of another tape. The new run's manifest
  records `resumed_from`, and its `genesis_hash` is the tape's genesis state's, computed again
  (amended at S2.4). An `--out` holding the manifest it resumed from is exit 1, checked after the
  manifest is read and before `verify` (amended at S2.4). Existing cli
  tests that resume a checkpoint copied away from its directory pass `--manifest`; the refusals
  they assert come before verification, so their messages stand. The `--tape` help loses "a dated
  edit at or after its tick is allowed" until the ruling of §15.1, question 2.
- **Manifests.** `--out DIR` writes `DIR/manifest.ron` after every checkpoint and at the end, by
  `manifest.ron.tmp` and a rename.
- **Hash output names its run.** A `--hashes` file opens with `# rustyecon hashes 1`, `# build
  <commit> <clean|dirty> <target>`, `# tape <name> tape_hash 0x… world_id 0x…` and `# from <tick>`
  (with `resumed <digest>` on a resume). The body is unchanged. Stdout prints a `run …` line with
  the same fields before its final `{t} 0x{hash}`: `run build <commit> <clean|dirty> <target>
  tape <name> tape_hash 0x… world_id 0x… from <tick>`, with ` resumed <digest>` on a resume
  (`Manifest::run_line`; amended at S2.4). `replay` prints the same line before its final hash.
  Existing cli tests skip `#` lines; the body they compare is the same, so nothing is loosened.
- **`certify`** runs `certify::certify` from genesis, feeding the Parquet writer from `on_tick`
  under `--telemetry`. With `--criteria` it runs to their `until`. Without, it needs `--until` and
  always gives UNSCORED. It writes `certificate.ron`, `manifest.ron`, `hashes.txt` and, with
  `--telemetry`, `telemetry.parquet`, then prints `render()`. A FAIL or UNSCORED is written before
  the exit. Exit 0 is PASS, 5 is FAIL or UNSCORED; 1 is a tape or criteria that does not load or
  fit, or a reserved key; 3 is a failed write. Each file goes through `<file>.tmp` and a rename. A
  telemetry file that cannot be created is exit 3 before the run; a telemetry write that fails
  during the run is exit 3 after the certificate, manifest and hash file are written, the partial
  file removed and the manifest naming no telemetry (amended at S2.4).

## 11. crates/probe: what moves now, what waits for observe (D13)

**Moves now** (oracle-free; certify owns them and probe's harness calls certify, from S2.5,
amendment 5). At
S2.3 each is in certify with the entry point probe will call: `battery::within_bound` (the
runaway bound), `MarketLine::trades` and `MarketObs::trades` (the trades half of dead),
`battery::rationed` (rationed ticks), `obs::burned` and `Obs::spoiled` (spoilage per good),
`Obs::transfers` (the provider's due and paid), `fold::trough` (a trough of a series),
`fold::ln_range` and `fold::range` (at rest in F), `battery::dead_share_ok` (the dead-share rule)
and `Obs::margin` (the ledger margin). They are: the runaway
bound (`RUNAWAY`; A12); a tick with no trade (the `trades` half of `dead`); rationed ticks per
market side (`HOLD_TOL`, now `rationed_below`); spoilage per good; the provider's Σ(due − paid);
the trough, as a function of a series and a reference (probe passes the oracle volume, certify the
segment's end level); "at rest in F" as `fold::ln_range` over F (`TOL_FLOOR/10`, now `band`); the
dead-share rule over W and F; the ledger margin. Probe passes its own observables to `ln_range`:
the relative prices w/r, p_m/r and p/r, s and the outputs. Certify's Settles passes nominal price
and cleared volume. The kick and the no-consumption count are new.

**Waits for `crates/observe`** (Phase 2; each needs the oracle): `Target`, the gaps and D̂;
`shock_distance`, κ and D̂_0; the envelope E1–E4; VACUOUS, `in_tol_from` and `first_out_in_w`;
`hold_check`; the band per relative price (v, π_m, π); the oracle-relative dead floor
(`LIVE_FLOOR`) and troughs; and the classifier `Class` as a whole.

**Stays in probe**, as the harness of a closed report: the perturbation grammar, the setups, the
tape generator, the CSV rows and `protocol.rs`, whose constants remain REPORT's registered
protocol. `appb-tape` gains `--perturb NAME --ticks L`, which writes §13's testdata.
Tests: `probe_summaries_unchanged` (probe, fast) runs `w*1.05`, `r*1.05` and Tier 3's `b=0.8` and
gives the class, dead ticks, worst fill, troughs and transfer shortfall of their battery.csv rows.
`probe_battery_csv_unchanged` (probe, `#[ignore]`, by name in `gate.sh`, about 15 s of CPU; S2.5) gives all 57
rows of `battery.csv`; the negative control's row of `families.csv` (57 DIVERGED by the bound, at
most 337 dead ticks); and the history `bcycle(1500,80)` as DEAD, with 125 dead ticks in each
b = 0.8 segment and 116 in each return to 0.4 (REPORT §3). These are the classes the moved runaway
and dead measures decide.

## 12. Telemetry: tidy long Parquet (C7; PLAN §3.8)

```rust
#[cfg(feature = "parquet")]
impl<W: std::io::Write + Send> TelemetryWriter<W> {
    pub fn new(sink: W, world: &World, run: &RunKey) -> Result<Self, TelemetryError>;
    pub fn push(&mut self, r: &TickReport) -> Result<(), TelemetryError>;   // TickReport is the input
    pub fn rows(&self) -> u64;
    pub fn finish(self) -> Result<Finished<W>, TelemetryError>; }          // amended at S2.4
pub struct Finished<W> { pub sink: W, pub rows: u64, pub bytes: u64,
                         pub digest: u64 }             // FNV-1a 64 over the bytes: the manifest's
```

| column | type | null | holds |
|---|---|---|---|
| `tick` | INT64 | no | the report's tick |
| `kind` | UTF8 | no | `market`, `ration`, `settle`, `ledger`, `run`, `event` |
| `node`, `good`, `class` | UTF8 | yes | tape keys |
| `key` | UTF8 | yes | the actor (settle) or event (event) key |
| `side` | UTF8 | yes | `buy` or `sell` |
| `tag` | UTF8 | yes | a ledger line's provenance; an event's source param |
| `metric` | UTF8 | no | the field's name |
| `value` | DOUBLE | no | the f64 as held, bit for bit; NaN stays NaN; a null is never a sentinel |

- **Rows by kind**, in this order within a report (amended at S2.4). `market`: the eight
  `MarketLine` fields, in field order. `ration`: requested, feasible, filled. `settle`: qty,
  value. `ledger`: each (good, provenance) line as `declared`, its provenance in lower case as
  `tag`, then `max_margin` and `max_drift` with no good. `run`: each good's `drift`, then
  `max_margin`. `event`: `fired`, valued at the occurrence. Keys replace ids, so branches compare
  by key (E8) with no id table, and a report naming an id the world lacks is refused whole.
- **Encoding.** Strings are dictionary-encoded, `value` BYTE_STREAM_SPLIT, `tick`
  DELTA_BINARY_PACKED (amended at S2.4), all LZ4_RAW; row groups of 2¹⁸ rows (I/O batching, not
  behaviour). The footer holds `rustyecon.telemetry` = 1, `rustyecon.tape`, `rustyecon.tape_hash`,
  `rustyecon.world_id` and the build as `rustyecon.build.commit`, `.dirty`, `.target` and `.rustc`.
- **What pure Rust costs.** Files are about 9% larger: GUI §3.4 measured one chunk at 46.0 MB as
  LZ4_RAW with byte-stream-split and 42.1 MB as zstd-3. In return no C is compiled; July's zstd
  built `zstd-sys` on both machines. The direct dependencies are bytes, chrono, half, hashbrown,
  num-bigint, num-integer, num-traits, seq-macro, twox-hash, ahash and lz4_flex. WSL's cache holds
  the closure (GUI's data-first probe built 60.0.0 with LZ4_RAW there). Windows lacks at least
  lz4_flex 0.14.0 and num-bigint 0.5.1, which S2.4 fetches, and S2.4 records the crate count for G0
  (GUI §3.1): 17 crates Parquet-free on every target, 36 on Linux and 35 on Windows with the
  feature (amended at S2.4, item 9).
- **Reproducible bytes.** parquet hashes with a runtime-seeded ahash. `telemetry_is_reproducible`
  writes twice in one process; `gate.sh` also writes the same run's telemetry from two processes
  through the binary and `cmp`s the files, so the bytes are shown not to depend on the seed.

## 13. Tests

Test names are binding. Every test that guards a fix or a criterion names the mutation it kills,
and its step runs that mutation and sees the test fail. Test bars are relative and named at the top
of their file. This said that before the criteria were committed no test applied gate's or appb's
registered bars but appb's kick; that was not so, and the S2.5 amendment, item 6, says which tests
did and why no value was tuned. From S2.5 those tests read the registered files.
Certify's testdata, in `crates/certify/testdata/`, is written by `appb-tape --perturb` and checked
by the probe test `certify_testdata_is_generated`: `appb-bcycle.ron` (`bcycle(1500,4)`, to tick
7,500); `appb-freeze.ron` (a run of the sweep's desk-turnover ×16 cell that the classifier scored
CONVERGED, frozen at the unstable point, REPORT §5; else the all-rates ×1/16 cell at b′ = 0.2);
and `appb-july.ron` (July's step rule from `w*2`, the negative control).

| Item | Test (crate) | Kills |
|---|---|---|
| N4, mandatory batteries | `criteria_without_runaway_do_not_certify` (certify): criteria lacking Runaway, Conservation or Determinism do not load, and a hand-edited certificate whose `listed` lacks one is refused. `settles_without_kick_does_not_load` (certify), and Kick without Settles | the C12 check deleted |
| N4, unscored | `unscored_run_never_passes` (certify): no criteria, and criteria for another tape, are each UNSCORED. `a_listed_battery_that_did_not_run_fails` (certify): a run stopped early, so Kick has "did not run". `certify_command_exits_on_its_verdict` (cli): exit 0, 5 and 5, each with its certificate written; `certify` without `--criteria` or `--until` is exit 1 | UNSCORED sealed as PASS; a missing battery counted as passed |
| No vacuous pass | `a_run_of_no_ticks_does_not_load` (`until` at the start, and below `min_segment`); `determinism_needs_a_resume_inside_the_run` (empty, share 0, share 1, two shares on one tick); `kick_horizon_needs_two_ticks` (horizon 0, H = 1, ⌈tail·H⌉ < 2, a size below 2⁻⁵³); `balance_needs_two_samples` (0, 1, and above m); `trades_needs_one_full_window`; `a_world_without_markets_fails_no_samples` (all certify) | each load or fit check deleted; the `no samples` rule deleted |
| N15 | `windows_are_relative_to_the_run` (certify): synthetic runs of 1,000 and 11,700 ticks, each with an excursion at 0.95 of the run, fail Settles in both. `windows_restart_at_each_dated_shock` (certify): synthetic segments that each return pass, and a middle segment that never settles fails, although the same run scored as one window passes. `short_segments_do_not_shrink_windows` (certify): with a no-op dated event every 100 ticks, `segments` merges to m; a synthetic slow drift fails Settles, and a kick gain that grows over 1,000 ticks fails Kick, since H is `ticks(horizon)` | absolute windows; no restart; no merge; H as a share of the segment |
| N12, batteries | `batteries_fail_closed_on_nonfinite_samples` (certify), a matrix: for each battery, NaN, +inf and −inf in each field of its "Reads" column, including (S = NaN, D = 0) and (S = 0, D = NaN) for Balance, and a NaN kicked price for `kick_gain`; each fails and names its tick and path. `balance_watch_fails_on_a_nonfinite_imbalance` (certify) | the gate deleted; `.filter(is_finite)`; `fold(0.0, f64::max)`; `S > 0 or D > 0` before the gate; `!(x > bar)`; July's skip at `invariants.rs:167` |
| N12, the seal | `seal_fails_every_nonfinite_path` (certify): on synthetic parts, NaN, +inf and −inf at each f64 path (reading values, bars, reports) make the verdict FAIL, with criteria and without. `reports_carry_a_nonfinite_input_to_the_seal` (certify): a NaN `cleared` makes the trough NaN and the certificate FAIL. `finite_scan_reads_every_rendered_number` (certify): each float token of a real PASS certificate's RON, replaced by `NaN`, is refused by `from_ron` as `NonFinite` at that path, not as a parse error. `render_prints_only_serialised_numbers` (certify): every number in `render()` is bit-equal to one in `to_ron()` | seal step deleted; UNSCORED before the finite rule; a scan of a subtree; arithmetic in `render` |
| R2, R8, liveness | `conservation_fails_on_a_margin_over_one` (a tick margin above 1, a run margin above 1, a NaN margin, a run short of `until`); `determinism_fails_on_one_differing_hash` (one flipped hash, one differing field in a resumed report, a replay final-hash mismatch); `trades_fails_on_a_silent_window` (a whole window silent, the last window ending at `until` silent) (all certify) | each comparison deleted; the base stream compared with itself; the resumed report not compared; the last window skipped |
| A12 | `runaway_bound_is_relative` (certify): gate with every genesis price, coin holding and pension ×2⁴⁰ gives bit-identical runaway readings. `runaway_detector_catches_a_runaway` (certify): `appb-july.ron` fails and names the good market | an absolute bound; the bound read from code |
| REPORT §6 criterion 1, the kick | `kick_check_passes_a_stable_rest` (certify): appb's eight kicks decay below `max_gain`. `kick_check_fails_a_rounding_freeze` (certify): `appb-freeze.ron` passes Settles and fails Kick, six kicks at the peak with their tails decayed (amended at S2.3). `a_kick_that_swings_out_and_back_fails` (certify, S2.3). `a_zero_kick_fails` (certify): g(T) = 0 fails. `a_neutral_direction_fails` (certify): a synthetic pair whose gap stays at the kick fails; one whose gap is near zero only at the last tick of an orbit fails. `kicked_tape_keeps_world_and_past` (engine). `scale_price_is_an_event_only_delta` (core) | a kick of 0; the kicked run compared with itself; dividing by ln(1 ± size); scoring one tick |
| Kick, after the base run | `deferred_tape_keeps_every_param` (certify): a gate variant whose only reference to a registered param is a `SetParam` after T keeps `world_id` and the registry. `a_failed_kick_is_a_fail_not_an_error` (certify): the kick runner fed another world's checkpoint gives Kick `pass: false` with a note, and `certify` returns a sealed FAIL. `reserved_keys_do_not_load` (certify): a tape with a `certify.` key is refused | dropping events and params; `?` on a kick error; no reservation |
| REPORT §6 criterion 2, transients | `transient_statistics_are_reported` (certify): on `appb-bcycle.ron`, every report equals an independent fold of the `TickReport`s written in the test; the b′ = 0.8 segment has a trough and a transfer shortfall, and the segment the 0.8 → 0.2 change opens a dead tick with no supply and a tick with no consumption (amended at S2.3); fills skip ticks where their side is empty; rationing is reported per (market, class, side) with Σ(requested − feasible) | a report over the wrong segment, market or class; dead ticks counted as rationing |
| REPORT §6 criterion 3, windows per shock | `a_history_whose_segments_return_passes` (certify): `appb-bcycle.ron` passes Settles and Kick in all five segments, under test bars with m below 1,500 ticks | windows across shocks |
| D10 item 1 | `new_source_event_keeps_world_id` (engine) | hashing `p.fixed`; hashing a param's sites |
| D10 item 2 | `fired_event_names_its_source` (engine) | `source: None` |
| D10 item 4 | `registry_names_each_use`, `params_are_read_only_through_sites` (engine); `role_sites_name_their_methods`, `each_site_converts_as_registered` (agents); `param_sites_are_recorded`, `clock_methods_convert_as_the_clock` (core) | one method per param, in the listing or the record; a source without its target's methods; the step rule's `up` as a share; a held `Site` whose method is not the recorded one; a rule that converts around its site |
| N10 | `tape_hash_covers_every_input` (certify): one edit per raw field kind moves the hash (name, a basis text, a param value by one ulp, a date, a spec inline number, a recurring `last`); reformatting, comments, CRLF, list order and a `to_ron` round trip keep it. `manifest_names_every_input` (cli): the build with its target, `tape_hash`, `world_id`, `genesis_hash`, a resumed checkpoint's digest, and the certificate's criteria hash | hashing `world_id`'s content; dropping the digest |
| Manifest | `manifest_refuses_a_checkpoint_off_the_run` (certify): a checkpoint whose state hash is not the last folded hash, or of another world, is `OffRun`. `manifest_paths_are_relative` (certify): files are recorded relative, with `/` | recording `state_hash(cp)` unchecked; a platform path |
| R16, the cli | `hash_output_names_its_run`; `resume_requires_a_recorded_checkpoint` (no manifest, another world, a record whose hash differs: exit 3); `resume_refuses_another_tape` (existing; gains the same-world case: a dated edit after the checkpoint keeps world and past but not `tape_hash`, exit 3 under the manifest); `resume_records_its_parent` (cli) | an unnamed header; skipping verify; verifying the world only |
| ScalePrice | `scale_price_is_an_event_only_delta` (core) refuses a recurring one; `price_shocks_are_counted` (certify): a gate variant with one dated `ScalePrice` certifies with `price_shocks: 1` | a recurring price nudge loading |
| Criteria | `criteria_need_a_unit_and_basis_for_every_bar`, `criteria_date_matches_its_file` (and `supersedes` names an earlier file), `criteria_round_trip` (certify) | a bare number loading |
| Certificate | `verdict_is_first_in_every_rendering`; `edited_verdict_is_refused` (certify): for each sealing rule, a certificate edited to another verdict is refused by `from_ron` and by `ron::from_str::<Certificate>` | readback that skips the seal |
| BalanceWatch | July's `a_constant_imbalance_is_a_pin_however_small` and `a_market_that_moves_is_not_pinned_and_one_at_zero_is_not_either`, ported with their bars named in the file; `a_one_sided_market_is_observed` (certify): a market one-sided on every tick is pinned at +1 | skipping one-sided ticks |
| Telemetry | `telemetry_round_trips_bit_for_bit`, `telemetry_is_reproducible`, `telemetry_names_its_run` (certify, feature `parquet`); the two-process `cmp` in `gate.sh` | — |
| E1, E2, R4 on certify | `certify_holds_no_threshold`, `certify_does_no_file_io`, `certify_hands_out_no_core_writer` (engine `scans.rs`, on `crates/certify/src`) | — |
| C11 | `probe_summaries_unchanged`, `probe_battery_csv_unchanged`, `certify_testdata_is_generated` (probe) | a moved measure that changes a class |
| S2.5, the verification | `a_tape_that_clamps_its_prices_does_not_pass`, `a_merged_shock_is_kicked`, `a_pass_flag_holds_its_readings`, `a_forged_pass_flag_is_refused`, `a_dead_tick_is_one_market_silent`, `cleared_volume_rests_in_f_too`, `kick_checks_its_horizon`, `certify_reads_the_whole_run`, `a_kick_moves_its_price_up_and_down`, `the_bars_are_the_registered_ones` (certify); `probe_summaries_unchanged`, `probe_battery_csv_unchanged` (probe); the cases added to `edited_verdict_is_refused`, `criteria_need_a_unit_and_basis_for_every_bar`, `batteries_fail_closed_on_nonfinite_samples`, `price_shocks_are_counted`, `kick_check_fails_a_rounding_freeze`, `certify_testdata_is_generated` and `resume_records_its_parent` | the S2.5 amendment's mutants, each named there |
| Results | `committed_certificates_recompute` (certify, `#[ignore]`, by name in `gate.sh`; C10). `committed_certificates_name_a_clean_build` (certify): dirty false and a 40-hex commit; `gate.sh` checks the commit is an ancestor of HEAD | — |

## 14. Steps and the gate

| Step | Delivers |
|---|---|
| S2.1 | this contract (`785ab19`) |
| S2.2 | §2.1–§2.3 and §2.5: D10 items 1, 2 and 4 with `Site`; item 3's loop; ENGINE amendments (§2.1, §2.6, §4, §5, §6, §7.1, §7.4, §11, §13); TAPE.md's row; the hash comparison and the new `world_id`s recorded (§2.5) |
| S2.3 | §2.4: `ScalePrice` and `MarketLine::trades`, with their ENGINE amendments (§2.4, §2.6, §7.3) and TAPE.md's row; certify: criteria, manifest, obs, folds, batteries, kick, certificate and seal, the finite scan; their tests on synthetic observations and testdata; `appb-tape --perturb`, the testdata and `certify_testdata_is_generated` (amended at S2.3) |
| S2.4 | the old S2.4, S2.5, S2.7 and S2.8 in one step, three commits (amended at S2.4): (a) telemetry behind `parquet`, `Cargo.lock`, the cli (the build stamp, named hashes, the manifest, verified resume, `certify`) and `gate.sh`; (b) the criteria alone; (c) the certificates, from a clean build of (b) |
| S2.5 | the old S2.5 merged into S2.4. Now the fix round of the bounded verification (amended at S2.5): its code and tests, then the certificates regenerated from a clean build of it |
| S2.6 | probe: the moved measures delegated, and the pins (`appb-tape --perturb` and the testdata landed at S2.3; the rest landed at S2.5) |
| S2.7 | `criteria/gate-2026-09-26.ron` and `criteria/appb-2026-09-26.ron`, committed alone, before any certified run of either tape (landed as S2.4's second commit) |
| S2.8 | `results/{gate,appb}/{certificate,manifest}.ron` from a clean build of S2.7, without `--telemetry`. A FAIL is committed as a FAIL and reported, never retuned in place (landed as S2.4's third commit) |
| S2.9 | docs: this file's amendments, ENGINE, TAPE, README, PLAN §3.2's text for decision 39, STATE, and GUI.md: §3.3's `tape_hash` definition and `RunKey` (`Hex` fields, `Build` with target and rustc), and §7.2 (items 1, 2 and 4 met; `source: Option<Key>`) |
| S2.10 | one adversarial pass, one fix round, one re-check of exactly the fixed items; results regenerated if a verdict path changed |

**Registered bars** (S2.7):
- **gate.** Until `1790-01-01` (tick 2,080; ENGINE §10). `min_segment` 1.0 year (one pension
  period, so a segment sees the recurring cycle whole; the closest shocks are two years apart).
  Conservation. Determinism resumed at 0.25, 0.5 and 0.75 (`gate_resume_from_checkpoints`'
  spacing). Runaway at 1e3 and Trades every 1.0 year (ENGINE §10; decision 19). Balance at July's
  bars: level 1e-9, spread 1e-12, 32 samples, run share 0.5 (`v2p3: invariants.rs`, the −1/6 pin).
  `rationed_below` 1e-9. No Settles and no Kick: the gate world claims no rest, since its prices
  drift with the pension (ENGINE §10).
- **appb.** §4's excerpt (with `max_peak` 1e6, amended at S2.3), plus Trades every 1.0 year, Balance as gate, and Settles with W at 0.5,
  F at 0.9, dead share 0.01 and band 1e-4 in log (PROBE-SPEC §4.5; tol/10).

**Gate additions** (`scripts/gate.sh` in WSL; the same commands on Windows). Gated on both:
- the Parquet-free build: `cargo check --locked -p rustyecon-certify`, `cargo test --locked
  --release -p rustyecon-certify` and `cargo clippy --locked -p rustyecon-certify --all-targets --
  -D warnings`, with the feature off (selecting certify alone keeps the cli's feature out), and
  `parquet` absent from certify's `cargo tree` then (amended at S2.4);
- `committed_certificates_recompute` and `probe_battery_csv_unchanged`, by name, which must run and
  pass (C10 says what each machine gates). S2.4 adds the first; S2.5 adds the second with its
  test (amended at S2.4 and S2.5);
- the build stamp: the binary's `run …` line names `git rev-parse HEAD`, and its dirty flag equals
  a non-empty `git status --porcelain` over the paths of §3;
- the committed certificates' commit is an ancestor of HEAD;
- telemetry written twice through the binary, in two processes, and `cmp`ed, with the manifests,
  certificates and hash files beside it (amended at S2.4).

Recorded: `cargo check --target wasm32-unknown-unknown -p rustyecon-certify` (D11), and on Windows
the byte equality of the recomputed results. The count of hashed ticks skips `#` lines, and the
cross-platform comparison compares the bodies of the hash files.

## 15. Open questions and the review

### 15.1 Open questions

1. **C1.** `ScalePrice` reopens core, where REPORT §6 said no criterion needed an engine change.
   The alternative writes a state no tape made. Is a phase-0 price shock acceptable under R3 and
   E1? It shocks a price once, on a date. The first draft said it clamps nothing and cannot recur;
   dense dated shocks do clamp (the verification's E1), so the verdict now counts them against
   the criteria's registered `price_shocks`, none by default (amended at S2.5). The ruling asked
   for is whether a registered count above 0 is ever acceptable in a certified tape, or whether
   `ScalePrice` belongs to the kick alone.
2. **R16 and dated edits.** S2.5 builds R16's letter: a cli resume needs the same `tape_hash`.
   E1 lets the engine resume under a dated edit at or after the checkpoint. Should the cli allow
   that too, behind an explicit `--edited` flag that records `parent` and marks the run in the
   manifest and the hash header? Nothing is built until the ruling.
3. **BalanceWatch's bars are absolute on the imbalance,** a dimensionless number in [−1, 1], not
   a price, flow or quantity, so A12 is read as allowing them. C13 settles the one-sided case now.
   If gate certifies FAIL on it, S2.8 commits the FAIL and reports it.
4. **C11 edits probe code that REPORT cites at `55c9e88`.** `probe_battery_csv_unchanged` guards
   it; the alternative keeps two definitions until Phase 2 proper.
5. *(Answered at S2.3: `appb-freeze.ron` is the turnover ×16 cell's w×1.05 run; its kicks fail
   at the peak, amendment 1.)* **The freeze testdata depends on the sweep.** If no candidate freezes at rest within 20,000
   ticks, the Kick battery's negative test needs a hand-built unstable tape. S2.6 says which.
6. **The horizon is one L.** An instability slower than L passes: REPORT §5's rates ×4 cell needs
   about 55,000 ticks to grow a 1e-9 kick past tolerance. C2 sits 4× inside that edge. A longer
   horizon would be a new dated criteria file.

### 15.2 The review of 2026-09-26

The adversarial review of the first draft raised ten majors and twelve minors. All are taken; none
is rejected. Three are taken in another form than the one proposed:
- **Kicked runs cannot be rerun from the notes** (minor). The claim is dropped rather than writing
  `kicks/*.ron`: a kicked tape resumes another tape's checkpoint, which C9 refuses at the cli, and
  rerunning `certify` reruns every kick.
- **Trades with `every` longer than the run** (in the vacuous-battery major). Trades scores a last
  full window that ends at `until`, and the fit needs n ≥ E; a partial window is never scored.
- **Short segments** (major). A short segment runs on into the next (§5), rather than failing
  Settles, Kick and Balance as uncomputable. The merged keys are named in the certificate.
