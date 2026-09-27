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
**State as of:** 2026-09-27. **G0, the GUI's shell, is under way** on branch `g0`, from
`reboot` at `397d7cd` (S2.6): G0.1, the viewer, is built, in two commits named G0.1, the crate
and its seams and then the panels, and a third that fixes what its verification found; G0.2,
the editor, is built in one commit (below).
**Phase 0 is closed.** Session 1 closed at P0.9, on
`reboot`.
Session 2 closed at S2.6, built on branch `phase0-s2` (S2.1–S2.6, from `reboot` at `cf3c0ff`);
`reboot` has since moved to it by a fast-forward, at `397d7cd`, and neither is pushed
(`origin/reboot` is at `cf3c0ff`). Session 2 built the
certification stack, the GUI's engine asks and the probe's criteria, and both tapes certify
PASS. Before it: oracle unit 1a joined at P1.1 (O3); the GUI's design, plan amendment A14 and
R16 landed at P0.11 (O1); by your ruling of 2026-09-26 the Phase 2 probe ran first, with verdict
GO ([docs/probe/REPORT.md](docs/probe/REPORT.md)) and no fallback (decision 38); and Breakpoint
B's pre-look passed beside it (S5.0, docs/spine/EYEBALL.md; decision 35).
Next, in order: G0.2's verification, and your look at the window (the G0 gate's item checked
by hand); Phase 1's units 1b–1f alongside.

## Where things stand

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

**Remote** (as the local remote-tracking refs show on 2026-09-26): `origin` has `main` and
`reboot` at `cf3c0ff`, the three `july-v2-*` tags and `pre-foundations` (A1 done). Not pushed:
`phase0-s2` (S2.1–S2.6), and the local branches `phase2-probe`, `spine-eyeball` and
`reboot-phase0`, whose work is in `reboot`.

## Decisions — veto window (your one-word calls)

The first nine record how your rulings and the standing rules were carried out; 10–21 were made
while building. 22–34 are the GUI's D1–D13 (A14; [docs/GUI.md](docs/GUI.md) says where each is
carried out): each stands unless vetoed before G0. D10's window closed with session 2, which
built its items. 41–58 are session 2's.

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

Decisions 59–64 were made while building G0.1's first part, where GUI.md was silent or wrong
against the code as it stood after S2.6. GUI.md's G0.1 amendment carries each; all are open to
veto.

59. **The GUI's gate is `scripts/gui.sh`,** beside `scripts/gate.sh` (decision 20), not
    `ci/gui.sh`. It tests in release, as the engine's gate does, and checks by name that each
    test G0's gate names ran and passed.
60. **The Runner takes the build and hands out the world.** `Runner::new(build, sink, wake)`
    mints each run key and ring checkpoint; `Obs::Loaded` carries the run key and the `World`;
    `Obs::Running` is new; `Obs::Refused` carries a `Refusal` (a tape that does not load, ring
    bytes that do not decode, a refused resume). A refused ring resume reruns from genesis.
61. **The store holds the run's status, and vm/ reads run types only.** No builder takes the
    model, so vm/ moves into observe with the Runner at G2 (D13); a scan holds run/ and vm/ free
    of the model, files, threads and clocks. `drive::Host` carries out the model's effects, so
    tests drive the whole seam without egui.
62. **The GUI shares the cli's build script** (`build = "../cli/build.rs"`), so a GUI run and a
    cli run of one checkout name the same build, by one definition.
63. **The session** lives in `$RUSTYECON_GUI_DIR`, else the platform's configuration directory.
    A new one plots every price and has the breakpoint on error; `layout.ron` is saved when the
    window closes; a file that does not read is set aside, never written over.
64. **The engine gate's GUI check runs on Linux only** (D1 says WSL), in
    `$CARGO_TARGET_DIR-gui`, so Windows engine steps build no GUI.

Decisions 65–72 were made while building G0.1's second part, the panels, where GUI.md was
silent or wrong against the code. GUI.md's amendment for that part carries each; all are open to
veto.

65. **The cursor is a report tick.** A panel reads the report of the cursor's tick and the state
    it left. Live follows the last tick that ran.
66. **The plots stack one panel per unit,** so every y axis names its unit and no two units
    share one. Overlay, difference and ratio against a parent wait for G0.2's branches.
67. **"Run until" reads a tick or a date.** A tick is a state tick, as the cli's `--until`
    takes it. A date runs through the tick it falls in, so an event of that date has fired.
68. **The registry's current value is the run's record.** Each use's per-tick value at it comes
    from `ClockMethod::per_tick`, the function `engine::registry` calls, and the copied basis
    from `FiredEvent.source`.
69. **The model asks for the actor inspector's snapshot** when an actor is selected and the run
    is not running, once per state tick. `Obs::Loaded` carries the tape, and a `Snapshot` its
    lots.
70. **Rationing onsets are logged once per class line per load,** compared exactly. One line per
    episode flooded the log: 39,990 lines on appb's 20,000 ticks, one-ulp shortfalls at rest,
    and 378 on the gate. A fill tolerance is certify's registered `rationed_below`, not the
    log's.
71. **The GUI depends on core for `num` alone,** for the inspector's ln(p′/p), and a scan holds
    it there. Alternative: the engine re-exports `num`, which is an engine change.
72. **The smoke mode** is `rustyecon-gui --smoke UNTIL TAPE`, at ten model years a second, with
    running and paused frames reported apart. D2's "no log axes" stays for G1: each of appb's
    prices has its own panel and scale.

Decisions 73–79 were made while fixing what G0.1's verification found. GUI.md's amendment for
those fixes carries each; all are open to veto.

73. **A poisoned run's own state is never read.** A snapshot of its current tick is rebuilt on
    a scratch `Sim` from the ring. Alternative: refuse the snapshot, and show no lots.
74. **Plots are keys kept across tapes.** A run opens with every price plotted when the
    session's plots name nothing of its world; another world's keys stay in the session, out of
    the stack. Alternative: plots per base, which changes `session.ron`'s format.
75. **A log line's tick is a report tick**, as the cursor's is; a line about a state says
    "state tick".
76. **`Obs::Ended`**: a driver reports once that its worker ended without a Stop. The run shows
    `Ended` and takes no command.
77. **The ledger line by key** is painted beside the engine's line, which stays as the cli
    prints it.
78. **Year gridlines** fall on the tick 1 January falls in, every 1, 5, 10, 50, 100, 500 or
    1,000 years; below two years the axis takes egui's marks, labelled by date.
79. **The core edge's guard is the scan.** At the next engine step the engine's prelude should
    re-export the pure `num` functions, and the GUI's edge to core then goes.

Decisions 80–91 were made while building G0.2, where GUI.md was silent or wrong against the
code after S2.6. GUI.md's amendment for G0.2 carries each; all are open to veto.

80. **`edit/` names core's raw schema, `Basis` and `Unit`,** and no other module does; the scan
    that held the GUI to `core::num` holds this too. An entry is written in those types, and
    none writes a state. Alternative: the engine re-exports them, an engine change, beside
    decision 79's `num`.
81. **A branch resumes whenever the world is kept.** A param only the schedule reads is outside
    `world_id` (decision 16), so removing an event with its source param, or copying a new value
    from a new param, resumes from the ring; only an edit of what the world reads reruns.
    `removal_only_branch_is_an_experiment` resumes at state tick 519, where GUI.md said genesis.
    D9's levels are then a schema question, not the GUI's way to resume a dial change.
82. **`plan` reads the parent's store,** each ring checkpoint carries the `prefix_id` its
    checkpoint stores, and a branch whose only agreeing checkpoint is genesis reruns.
83. **The lineage names the tape it describes** (its `tape_hash`) and has a format. A lineage
    beside a tape edited after it was saved is logged; the tape is still an experiment.
84. **Keys are new to every tape of the run tree, the staged edits and the runs closed this
    session.** The serial of `gui.<s>.<n>` is in `session.ron` (format 2), one more at each
    launch that reads it; a format-1 session is set aside, as any session that does not read.
85. **The stamp's date is today's in UTC,** read at launch and handed to the model.
86. **An act is typed as the tape writes it,** read by the tape's own parser; the raw pane shows
    the text it read, with the parser's line, column and a caret. Alternative: a form per act,
    which the agents' actions would multiply. The editor's form is the one way in; the
    registry's and the inspector's wait for G1.
87. **"Ledger changed"** compares tolerances by key, value and unit with the parent run's, or
    with the ancestor a reopened tape's lineage names when this session holds it; the manifest
    envelope gains `ledger_changed`.
88. **An export is five files at most** (`series.csv`, `manifest.ron`, `tape.ron`,
    `tape.lineage.ron`, `ancestor.ron`), and nothing is written over, by export or "Save tape
    as".
89. **Compare's "pinned series" are the plotted series;** the session pins entities. The plots'
    overlay, difference and ratio against a parent move to G1.
90. **Branches are not re-made at launch** (G1): a branch lives in memory until saved, so
    §5.1 item 7's refusal to re-apply edits to a changed base waits with it.
91. **A layout saved before a pane existed gains it** beside the inspector, rather than being
    set aside.

## Open — your calls

- **The GUI's decisions**, 22–34 (D1–D13): open to veto before G0. D10's items are built.
- **Hosted CI** (A5): the push of 2026-09-26 started it; it runs on every push unless you turn it
  off.
- **Decisions 35–40** (Breakpoint B's three calls and the probe's proposals), taken by Claude on
  your word and open to veto.
- **Decisions 41–58** (session 2's), open to veto.
- **Decisions 59–64** (G0.1's first part), **65–72** (its second part), **73–79** (its
  verification fixes) and **80–91** (G0.2), open to veto.
- **The G0 gate's window check, by hand** (the command is in "Where things stand", under G0.1's
  second part): the window opens `tapes/gate.ron` and runs to 2,080 with every price plotted
  and no panic. The smoke mode ran it on Windows; nobody has looked at it.
- **CERTIFY §15.1's questions:** (1) is a registered `price_shocks` count above 0 ever
  acceptable in a certified tape, or does `ScalePrice` belong to the kick alone; (2) should the
  cli resume under a dated edit behind an explicit `--edited` flag that records the parent and
  marks the run; (3) BalanceWatch's bars are absolute on the imbalance, a number in [−1, 1],
  read as allowed by A12; (4) C11 edited probe code that REPORT cites at `55c9e88`, guarded by
  its pins; (6) the kick's horizon is one L, so an instability slower than L passes.
- **Pushing `reboot`** (at `397d7cd`, session 2 merged by a fast-forward), and merging `g0` into
  it when G0 closes.
- **The Phase 2 session budget** that A11's kill condition needs (PLAN Phase 2), now for Phase 2
  proper's other instances.
- **The decisions above**, especially 10 (the engine crate, not in PLAN's crate list), 11, 15
  and 17, and the GUI's 22–34.

## Open — work

- **O1. The GUI.** Designed ([docs/GUI.md](docs/GUI.md); A14), with egui in `crates/gui`. G0,
  the shell, is under way (two to three sessions; GUI.md §9): G0.1 the viewer is built, on
  2026-09-27, in two parts (the crate's seams, the Runner and `ThreadDriver`, the Extractor, the
  store and the ring; then the toolbar, timeline, outliner, plots, inspector, registry and log,
  the goldens, the kittest scripts and the smoke mode), and its verification's twelve issues
  are fixed; G0.2 the editor is built, on the same day (`materialise`, the lineage, branches,
  compare and export); next is G0.2's verification. G1, the
  oracle lab, starts after G0 and Phase 1's gate. `crates/engine` was built for it: a frontend
  depends on the engine alone, steps a `Sim` on a worker thread and reads each `TickReport` over
  a channel. Session 2 gave it what §7.2 asked: `FiredEvent.source`, the registry's sites with
  their methods, a `world_id` that a new source event keeps, and from certify `RunKey`,
  `tape_hash` and a manifest that needs no Parquet or I/O (GUI.md, updated at S2.6).
- **O2. Phase 0 session 2: closed at S2.6** (2026-09-26; "Where things stand" above;
  docs/CERTIFY.md). The certification stack moved from `july-v2-phase-3` into `crates/certify`
  with N4, N10, N12 and N15 fixed, every threshold in dated criteria, A12's runaway detector,
  the probe's three criteria, and D10's items 1, 2 and 4. Both tapes certify PASS.
- **O3. The oracle, unit 1a, landed at P1.1.** Built by another run and verified there (114
  tests), it joined through the members glob. It depends on `core` alone, for `num`, and nothing
  on the engine path depends on it (R13). The workspace's `clippy.toml` denies the platform
  maths, so x^k, ln(1 + z) and its fused multiply-add now go through `core::num` (libm), which
  gained `fma`. Its outputs are then byte-identical on WSL and Windows (5000 random economies,
  every regime), no golden moved, and G8's exact tie still ties. Units 1b–1f follow (PLAN
  Phase 1), alongside G0.
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

1. **G0.2's verification** (O1; docs/GUI.md §9), bounded as G0.1's was, then G0 closes when
   its gate's last item, the window by hand, is checked. G0.1, the viewer, and G0.2, the
   editor, are built (2026-09-27). The plots' overlay, difference and ratio against a parent,
   and re-making branches at launch, are G1's (decisions 89, 90).
2. **Your look at the window**, the G0 gate's item checked by hand (the command is under G0.1's
   second part in "Where things stand"; G0.2's entry says how to try the editor).
3. **At the next engine step, the engine re-exports `num`** (decision 79), and the tape's raw
   entry types, `Basis` and `Unit` (decision 80), so the GUI's edge to core goes and core's
   writer is out of its reach by type again.
4. **Alongside, Phase 1's units 1b–1f** (O3; PLAN Phase 1), each with its gate: many categories
   and the fork, many machine types, worker types and the wall, parcels and s(q), households and
   government. Units 1b–1e also give Phase 2 proper its other instances; it starts from the
   probe's roles and harness, with many markets as its first untested risk.

## File map

```
STATE.md                 you are here; start here next session
README.md                what rustyecon is, the crates, how to build and test
docs/PLAN.md             the plan, amended by the addendum's rulings (2026-09-25) and decision 39
docs/ENGINE.md           the Phase 0 engine contract, with each step's amendments (P0.3–G0.1)
docs/CERTIFY.md          session 2's contract: criteria, batteries, kick, seal, manifest, cli,
                         telemetry, with each step's amendments (S2.2–S2.6)
docs/TAPE.md             the tape's schema guide
docs/GUI.md              the GUI's design (A14): stack, architecture, panels, editor, map, roadmap;
                         amended at G0.1, in its two parts, and at G0.2
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
crates/oracle            the equilibrium solver, unit 1a (P1.1); its README and docs/unit-1a.md
crates/certify           criteria, batteries, the kick, the sealed certificate, the manifest;
                         Parquet telemetry behind the feature `parquet` (S2.3–S2.5)
crates/certify/testdata  appb variants from `appb-tape --perturb`: bcycle, freeze, july, buffer16
crates/worldgen          empty until Phase 4
crates/gui               the GUI (G0.1, G0.2): model/, run/, edit/, vm/, drive/, platform/, ui/,
                         app.rs, the binary rustyecon-gui; its tests run under scripts/gui.sh
                         only (D1)
crates/gui/tests/golden  the view-model goldens, one RON file per builder and point
                         (UPDATE_GOLDEN=1 rewrites them)
tapes/gate.ron           the gate world
tapes/appb.ron           the probe's Appendix B world, generated from the oracle
criteria/                each tape's dated criteria, registered before its first certified run
results/                 committed verdicts: results/<tape>/certificate.ron and manifest.ron
data/spine/              the spine's fetch, extract and eyeball scripts, manifests, CC0 files;
                         their cache is $SPINE_ROOT or the ignored data/spine/.cache/
docs/spine/              DATA_NOTES.md and EYEBALL.md, Breakpoint B's pre-look (S5.0)
scripts/gate.sh          the gate as one script; the GUI excluded, checked once on Linux (D1)
scripts/gui.sh           the GUI's gate, run at each G-stage (G0.1); diffs the editor's branch
                         tapes against the cli too (G0.2)
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
  --until 2080 --hashes <file>` and the same for `tapes/appb.ron --until 20000`, and a byte
  comparison of each file's body (the `#` header names the build and target) with WSL's.
- Session 2 ran from a worktree, `D:/rustyecon-wt/s2` (`/mnt/d/rustyecon-wt/s2` in WSL), with
  targets outside it (`/root/scratch/target-s2-*`, `D:/rustyecon-targets/s2-*`) and its logs in
  `D:/rustyecon-s2/`. The build stamp reads git through the worktree's `.git` file, mapping its
  Windows path for WSL.
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
