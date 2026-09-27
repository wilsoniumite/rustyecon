# GUI — the interactive frontend, `crates/gui`

Dated 2026-09-25 and revised twice the same day: first after the rulings, decisions D1–D13 and two
adversarial reviews, then after two checks of that revision.
[`reboot/GUI-review-ledger.md`](reboot/GUI-review-ledger.md) records what became of each issue in
both rounds. This file is read against PLAN.md, ADDENDUM's rulings and branch
`reboot-phase0` as committed at `a8f9ed8` (P0.5): `docs/ENGINE.md` with its P0.3, P0.4 and P0.5
amendments, and the public APIs of core, markets, agents, engine and the cli. P0.6 (housekeeping)
and Phase 0's second session are cited from ENGINE's contract. The oracle is unit 1a, uncommitted in
the main checkout. Stack figures come from a compile spike on Windows and WSL (Rust 1.97.1), re-run
by the facts review; each is labelled with what was measured, where and how. Plan amendment A14 goes
with this file: [ADDENDUM](reboot/ADDENDUM.md) §6 has it and its rulings, PLAN.md carries R16 and
the G-stages, and STATE.md lists D1–D13 among its decisions (22–34), open to veto.

**Landed at P0.11 (2026-09-26),** unchanged but for its links and the local paths it cited.
Read it against what moved since `a8f9ed8`, where its line numbers are. Session 1 closed at P0.9:
P0.6, P0.7 and P0.9 were fixes from adversarial review, and P0.8 the housekeeping this file calls
P0.6. ENGINE's `ci/gate.sh` and `ci/gate.ps1` became `scripts/gate.sh` and a Windows check by hand
(STATE.md decision 20), so read `ci/gate.sh` below as that script and `ci/gui.sh` as a sibling
beside it. The gate world's final hash has been `0x61f9c8529131ff17` since P0.6, which took
schedule params out of the state. Oracle unit 1a landed at P1.1. STATE.md checks D10's four items
again at P0.9.

**Updated at S2.6 (2026-09-26),** when Phase 0's second session closed (docs/CERTIFY.md). D10's
items 1, 2 and 4 landed at S2.2 and item 3 was met already (§7.2). Session 2 chose the tape hash
and built the three raised items: §3.3 and §7.2 say how. The rest of this file is unchanged.

**Amended at G0.1 (2026-09-27),** when `crates/gui` landed with its seams and no panels: the
model and `reduce`, the Runner, `ThreadDriver`, the Extractor, the in-memory store with its
decimation, the ring, a view-model skeleton, `session.ron` and `layout.ron`, and D1 in the
workspace. This file was written before S2.2–S2.6, and where it was wrong or silent against the
code the build decided. Each change is made in place, in the section named:
1. **The GUI's gate script** is `scripts/gui.sh`, beside `scripts/gate.sh`, where the
   repository's gate lives (STATE decision 20). Read `ci/gui.sh` below as that script. It tests
   in release, as the engine's gate does, and checks by name that each test G0's gate names ran
   (§8.2).
2. **The Runner takes the build** (§3.3): `Runner::new(build, sink, wake)`. It mints the run key
   and every ring checkpoint, so it names the binary. The stamp is the cli's `build.rs`, which the
   GUI's manifest names as its own build script (§3.2).
3. **Observations** (§3.3). `Obs::Loaded` carries the run key and the run's `World`, for key
   lookups on the model's side. `Obs::Running { tick }` is new: the store holds the run's status,
   so it needs the start of a run as well as its pause. `Obs::Refused` carries a `Refusal`: a
   tape that does not load, ring bytes that do not decode, or a resume `Sim::resume` refused.
4. **Effects** (§3.3). `reduce` returns `ReadTape`, `Spawn`, `Send { run, cmd }`, `Close`,
   `SaveSession` or `SetAsideSession`. `drive::Host` carries them out and pumps observations
   back, so tests drive the whole seam without egui; app.rs holds a Host.
5. **vm/ reads run types only** (§3.2): the store and the log lines, never `model/`, so it moves
   into observe with the Runner at G2 (D13). A scan holds it, with run/'s freedom from files,
   threads and clocks (§8.1).
6. **Series** (§3.4). A value is stamped with the tick that ran; holdings and params are read
   from the state it left. A series is a column of (tick, value) pairs, and a gap is a tick with
   no pair. The decimator lives in run/, free of egui, and splits a line at every gap.
7. **Ring ticks** (§3.3). A ring tick is the tick 1 January falls in, which can begin in the old
   year: 1751-01-01 falls in tick 51, which begins on 1750-12-26. The gate world has 41, from
   tick 0 to tick 2,080.
8. **The scans** (§8.1) also read `drive/` and `platform/`, which are free of egui too.
9. **The session** (§4, U8). `session.ron` holds the bases, the plots, the pins, the breakpoints
   and the speed cap, in `$RUSTYECON_GUI_DIR` or else the platform's configuration directory. A
   new session plots every price and has the breakpoint on error. `layout.ron` is saved when
   the window closes. A file that does not read is set aside, never written over.
10. **Windows** (§8.2). C: had 114 GB free on 2026-09-27, and `ci/gate.ps1` does not exist:
    Windows runs `scripts/gate.sh` under Git Bash, which skips the GUI's non-blocking check off
    Linux, and `scripts/gui.sh` the same way, with `CARGO_TARGET_DIR` on D:.

**Amended at G0.1's second part (2026-09-27),** when the panels landed: the toolbar, timeline,
outliner, plots, inspector, registry and log, the view-model goldens, G0's kittest scripts,
`every_drawn_vertex_is_recorded` and the smoke mode. Where this file was silent or wrong
against the code, the build decided, and each change is made in place, in the section named:
1. **The cursor** (§4). A cursor is a report tick: the tick that ran. A panel reads that tick's
   report and the state it left. Live follows the last tick that ran. Before any tick runs,
   a panel shows no report.
2. **Stacked plots** (§3.4, §4). The plots stack one panel per unit, so every y axis names its
   unit and no two units share one. The x axis is the report tick, labelled by its date. The
   panels share their x axis and hover cursor, and each draws the model's cursor. Colours are
   one neutral palette. Overlay, difference and ratio against a parent need a branch, so they
   wait for G0.2.
3. **The plot cache** (§3.4). Columns are `2^k` ticks wide: the fewest that fit the visible
   span into the panel's pixel columns. A cache is rebuilt when its width changes or the store
   starts afresh, and extended otherwise. Each frame it records the vertices it lent egui.
4. **Run until** (§4). A bare number is a state tick, as the cli's `--until` takes it. A date
   runs through the tick it falls in, so an event of that date has fired.
5. **The registry** (§4). Its rows are `engine::registry`'s listing, made once per load. The
   current value is the run's record after the cursor's tick. Each use shows its per-tick
   value at genesis, from the listing, and at the current value through
   `ClockMethod::per_tick`, the function the listing calls. "Fixed" is the listing's use:
   live, fixed or schedule. The copied basis comes from the last fired `SetParam`'s `source`,
   dated by the event's own date.
6. **The actor inspector** (§3.3, §4). When an actor is selected and the run is not running,
   the model asks the run for a snapshot of the state the cursor's tick left, once per state
   tick. `Snapshot` carries each holding's lots. Holdings come from the record every tick.
7. **Observations** (§3.3). `Obs::Loaded` carries the tape. The store keeps it and the listing,
   and the event inspector reads each event's basis from it.
8. **The log's rationing onsets** (§4). A class line's onset is logged once per load: the first
   tick it is filled below its request, compared exactly. One line per episode flooded the log:
   39,990 lines on appb's 20,000 ticks, where fills differ from requests by one ulp at rest,
   and 378 on the gate, whose sellers are rationed on and off. A tolerance on a fill is a
   criterion's registered bar (certify's `rationed_below`), not the log's.
9. **Core, for `num` alone** (§3.2). The inspector shows ln(p′/p), and the engine re-exports
   no `num`. So the GUI depends on `rustyecon-core` and names `rustyecon_core::num` and nothing
   else, held by the scan `the_gui_names_core_for_num_alone`. Core's writer stays out of reach.
10. **Open** (§3.3). `Intent::PickTape` becomes `Effect::PickTape`, which `drive::Host` carries
    out through `platform::pick_tape` (rfd's native dialog), answering with `Intent::Open`.
11. **"Ledger changed"** (§4). Only a branch has a parent, so a base run's health chip says
    `false` until G0.2's branches.
12. **The smoke mode** (§8.1): `rustyecon-gui --smoke UNTIL TAPE`. It runs the tape to state
    tick UNTIL at ten model years a second with every price plotted, prints CPU per frame
    (p50, p90, max) running and then for 120 paused frames, with the panes drawn, and closes.
    It keeps no session.
13. **The goldens** (§8.1) are one RON file per builder and point, in
    `tests/golden/<tape>-<point>/`. The gate's points are the state ticks 1, `mine.cut`'s tick
    plus 1, `oven.opens`'s tick plus 1 (1768) and 2,080; appb's are 1 and 20,000. The build is
    named `golden`. Each point also asserts what this file says it shows.
14. **Image snapshots** (§8.1). WSL has no lavapipe, only NVIDIA's ICD, so no image snapshot is
    taken or gated.
15. **D2's "no log axes"**, revisited on appb as STATE asked. Kept for G1: one panel per unit
    gives each of appb's four prices its own linear scale, and `num::ln` is now in reach.

**Amended at G0.1's verification fixes (2026-09-27),** after a bounded verification of G0.1
found five major issues and seven minor ones. Each change is made in place, in the section
named, and each fix has a test that fails without it, checked by mutation:
1. **A failed run's snapshot** (§3.3). A failed step leaves the `Sim` poisoned at the tick it
   began, holding what the failure left, a state no tick of the tape left. The Runner never
   reads it: a snapshot of a poisoned run's current tick is rebuilt on a scratch `Sim` from
   the ring, as an earlier tick's is, and a later tick is not asked for.
2. **The scans read `use` trees** (§8.1). A reach into `ui/` or `app.rs`, and from run/ or vm/
   into the model, the drivers, the files, threads or clocks, is refused in a group, a rename,
   a glob import and the bare path a glob allows, as well as in a path.
3. **The plots record what they lend** (§3.4). One function makes each segment's `Line`,
   hands it to the plot and records the points that `Line` is made of. The drawn list is that
   record, not the cache. `every_drawn_vertex_is_recorded` also checks that each segment lent
   is painted as one path of as many points, on one affine map of its ticks per panel, with
   the panel's cursor on the same map.
4. **Plots across tapes** (§4). A run opens with every price plotted when the session's plots
   name nothing of its world: a new session, or one another tape wrote. A plotted key of
   another world stays in the session, is left out of the stack and is named apart. No unit is
   made up for it.
5. **The scripts** (§8.1). The gate script checks the painted identity chip, each panel's unit,
   year labels and cursor, and plots from the inspector and from the outliner. A theft script
   and a script whose session another tape wrote join the named tests. Each plot button's
   accessible name names its series.
6. **The ledger line by key** (§3.3, U7). The engine's ledger line names dense ids, as the cli
   prints it. The health chip paints it, and beside it the same failure by key: a shortfall's
   holder and good, a hook's actor, and for the events phase the tape events due in that tick.
7. **A worker that ends unasked** (§3.3). A worker whose Runner panicked says nothing. Its
   driver now reports `Obs::Ended` once, and the run shows `Ended`, logs it and takes no
   command.
8. **The log's ticks** (§4). A log line's tick is a report tick, the tick that ran, as the
   cursor counts. A line about a state names the tick that left it and says "state tick", so a
   click moves the cursor to the state the line names.
9. **Year gridlines** (§3.4). The x axis marks year starts, the tick 1 January falls in, every
   1, 5, 10, 50, 100, 500 or 1,000 years, labelled by that year. Below two years it takes
   egui's marks, labelled by date. A year never labels a tick without a 1 January.
10. **U10's named test** poisons NaN, +inf and −inf.
11. **Core for `num`** (§3.2). With the GUI's edge to core, core's pub `apply`, `SimState` and
    `Checkpoint::of` compile in the GUI. U1's guard for the GUI is therefore the scan
    `the_gui_names_core_for_num_alone`, not the type system. At the next engine step the
    engine's prelude should re-export the pure `num` functions, and the edge then goes.
12. **The gate script's list** (§8.2) names 17 tests.

**Amended at G0.2 (2026-09-27),** when the editor landed: `edit/` with `TapeEdit`,
`materialise`, the lineage, `plan`, the form and export; branches in the model; the editor and
compare panes. This file was written before S2.2–S2.6, and where it was wrong or silent against
the code the build decided. Each change is made in place, in the section named, and each named
test fails without what it guards, checked by mutation:
1. **Edit names core's raw schema** (§3.2). An entry is written in core's raw types, which the
   engine does not re-export. So `edit/`, and no other module, names `rustyecon_core::tape::raw`,
   `rustyecon_core::Basis` and `rustyecon_core::Unit`: plain data, none of which writes a state.
   The scan `the_gui_names_core_for_num_alone` holds that, and holds `num` as before everywhere
   else. `edit/` reads run types and reaches no model, driver, file, thread or clock
   (`edit_reaches_no_model_file_thread_or_clock`); run/ and vm/ may not reach `edit/`.
2. **`materialise` as built** (§5.1). It returns a `Branch { tape, applied, world }`: the world
   serves `plan`. It builds a `Sim` to validate, reads its world and drops it; only a Runner
   keeps one (U1). `EditError::Orphans` lists every param the removals leave unreferenced. A
   load error or a parse error of the round trip names the edit it comes from.
3. **`plan` reads the parent's store** (§5.1 item 5): `plan(parent: &Store, child: &World)`, so
   `edit/` needs no model. Each ring checkpoint stores its `prefix_id`, read off the checkpoint
   when the Runner takes it. A checkpoint at genesis gains nothing over `Sim::new`, so a branch
   whose only agreeing checkpoint is genesis reruns.
4. **The world's identity since S2.2** (§5.1 items 4 and 5, §5.2, §8.1). A param only the
   schedule reads is outside `world_id` (STATE decision 16). So `RemoveEvent(mine.cut)` with
   `RemoveParam(mine.capacity.cut)` keeps the world, and the branch resumes from 1760-01-01's
   ring tick, 519, not from genesis. A new value, `AddParam` of a param only a `SetParam`
   reads and the event that copies it, resumes too. Only an edit of what the world reads
   reruns: a genesis value the world reads, or a param the world reads added or removed.
5. **The lineage as built** (§5.1 item 6): `Lineage { format, tape_hash, parent: Ancestor {
   tape_hash, path }, edits, records_viewed }`. Its `tape_hash` names the tape it describes, so
   a lineage beside a tape edited after it was saved is logged as not describing it. A
   branch's lineage is made with the branch. A tape opened with a lineage beside it is an
   experiment.
6. **Keys** (§5.1 item 2). "New to the run tree" means new among the keys of every kind in
   every tape of the tree, the staged edits, and the runs closed in this session. The serial
   `s` of `gui.<s>.<n>` lives in `session.ron`, whose format is now 2; it grows by one at each
   launch that reads the session. A format-1 session does not read and is set aside. (Revised
   after G0.2's verification: item 4 of that block.)
7. **The stamp's date** is today's date in UTC, read by `platform::today` at launch and handed
   to the model as `Intent::Today`, so `reduce` stays pure.
8. **The form and the raw pane** (§5.1 items 2 and 3). An act is typed as the tape writes it,
   `SetParam(param: "mine.capacity", to: "mine.capacity.base")`, and read by the tape's own
   parser. An act that does not read is refused with the parser's line and column, and the
   read-only raw pane shows the text it read with a caret under the column. A parse error of
   the round trip is reported with its line, column and edit. The editor's form is the one way
   in at G0.2; the registry's and the inspector's ways in (§5.1 item 1) wait for G1.
9. **"Ledger changed"** (§4, §5.1 item 2) compares the tolerances, by key, value and unit, with
   the parent run's, or with the ancestor's that a reopened tape's lineage names, when this
   session holds that tape. The health chip, the CSV's `#` lines and the manifest envelope
   show it. `GuiManifest` gains the field `ledger_changed`. (Revised after G0.2's
   verification: item 1 of that block.)
10. **Export as built** (§4). The directory gets `series.csv`, `manifest.ron`, `tape.ron`, and
    for an experiment `tape.lineage.ron` and `ancestor.ron`, the tape its lineage names when
    this session holds it. The manifest is begun from a `Sim` at genesis, as the cli begins
    one, moved to the run's start and fed every recorded tick. A resumed branch's
    `resumed_from` records the ring checkpoint's tick, digest and state hash and the parent's
    run key. No file is written over, by export or by "Save tape as"; a path this session
    opened or saved is refused by name.
11. **Compare as built** (§4). "The difference of pinned series" reads as the plotted series:
    the session pins entities, not series. The plots' overlay, difference and ratio against a
    parent move to G1, as §9 lets compare's parts move when G0.2 overruns.
12. **Branches are not re-made at launch** (§5.1 item 7). `session.ron` keeps the bases and the
    serial. A branch lives in memory until "Save tape as" writes it with its lineage. Re-making
    branches at launch, and refusing to re-apply edits to a base whose hash changed, wait for
    G1.
13. **The layout** (§4). A `layout.ron` saved before a pane existed gains the pane beside the
    inspector; it was set aside before.
14. **The gate script** (§8.2) names 27 tests, and diffs the cli's runs of the two branch
    tapes the branch tests write against the GUI's hashes.

**Amended at G0.2's verification fixes (2026-09-27),** after a bounded verification of G0.2
found six major issues and four minor ones. Each change is made in place, in the section
named. Each fix has a test that fails without it, checked by mutation. Items 1 and 4 revise
items 9 and 6 of the G0.2 block:
1. **"Ledger changed" has three answers** (§4, §5.1 item 2). The model returns a
   `LedgerCheck`: no parent, same, changed, or unknown with the reason. "No parent" is a tape
   no GUI edit made, with no lineage. A branch compares with its parent run. A reopened tape
   compares with the ancestor its lineage names. That is a run of this session with that path
   and `tape_hash`, or else the file at that path, which the host reads
   (`Effect::ReadAncestor`). The file is kept only when its `tape_hash` is the lineage's. When
   neither is held, the check is unknown. So is a tape that carries the marker and has no
   lineage. The chip then says "ledger unchecked" and why. The CSV says `# ledger changed:
   unknown (<why>)`, and `GuiManifest.ledger_changed` is an `Option<bool>`, `None` when
   unknown. Before, all of these said "no".
2. **U3's other clauses are tested** (§2, §8.1). A tape with a lineage beside it and no marker
   in its text is an experiment. So is a tape whose only marker is in one basis. The same tape
   without the lineage is a run.
3. **A reopened experiment exports its ancestor** (§4). The reopened run keeps the ancestor
   tape it found or read, so its export carries `ancestor.ron`, as a branch's does.
4. **Keys are new to the whole session** (§5.1 item 2, E8). A new key must be new to every
   tape open in this session, to every run closed in it and to the staged edits. That covers
   every run tree, a saved branch reopened as its own root, and a second root of one file. The
   form checks a typed key so, and Apply checks it again. A minted key's `n` is one more than
   every `n` under the session's serial, in those keys and in every key staged this session,
   so no `n` is minted twice in a session. When a session is set aside and its serial still
   reads, the new session takes the next serial. Two windows launched from one `session.ron`,
   or a session whose file is lost, can still share a serial. Their keys are then new only to
   the tapes each one opened. That is recorded here, not fixed.
5. **Compare reaches back** (§4, §5.1 item 5). A parent that itself resumed from its own
   parent's ring holds no record before its start. Compare reads the states before it, and the
   plotted series, in the records of the runs it resumed from, while they are open. When those
   records do not reach far enough, a difference at the first state both hold says the hashes
   may differ earlier. The range compared is always shown, and each identity names its origin.
6. **A stale lineage is flagged** (§4, §5.1 item 6). A lineage can describe another
   `tape_hash` than its tape's. The CSV's lineage line then says which, and
   `GuiManifest.lineage_stale` is true. "Save tape as" still writes it beside the new file, and
   logs that it describes another tape. Without it, a tape whose only mark was its lineage
   would reopen as a run.
7. **The gate script** (§8.2) also checks the cli's `tape_hash` of each branch tape against
   the GUI run key's.
8. **The toolbar's goldens** say `ledger: NoParent` where they said `ledger_changed: false`.
   The view-model's field changed; the gate world did not.

**Closed at G0.3 (2026-09-27).** G0 is built, and its gate is met but for the window checked
by hand, which is the user's (§9; STATE.md has the command). G0 is five commits on branch `g0`
from `397d7cd` and this one; the five blocks above say what each built and where the build
departed from this file. This block adds what the close did, what G0 leaves to G1, and an index
of every deviation:
1. **What G0 built.** `crates/gui`, a library and the binary `rustyecon-gui`. `model/` holds the
   session, the intents, the effects and `reduce`. `run/` holds the Runner, the Extractor, the
   store and its decimator, the ring and the log. `edit/` holds the edits, `materialise`,
   `plan`, the lineage, the form, the keys and export. `vm/` has one builder per panel.
   `drive/` holds `ThreadDriver`, its pace, the `Host` and the smoke mode's frame timings.
   `platform/` holds the files, rfd and the session's directory. `ui/` draws the toolbar,
   timeline, outliner, plots, inspector, registry, editor, compare and log in an egui_tiles
   layout. The crate has 72 tests, one of them an ignored measurement. `scripts/gui.sh` is its
   gate.
2. **The gate's tests by name** (§8.2, §9). §9 names "the reducer's state-machine tests" as a
   group, and `scripts/gui.sh` did not check them by name. It now names the fifteen in
   `tests/model.rs`, so it names 42 tests. Eight of the fifteen had killed no mutant by name.
   At the close each of the eight kills a mutant of `reduce` (STATE.md has the ten).
3. **The measurements** (§3.1, §5.2, §8.2) are taken again on the finished crate, on WSL. G0.1
   took D1's costs and the recount on the crate's seams alone, and G0.2 took the rerun time
   before its fixes. §3.1, §5.2 and §8.2 give the numbers.
4. **Left to G1**, collected from the blocks above: the plots' overlay, difference and ratio
   against a parent (G0.2, item 11); re-making branches at launch, with §5.1 item 7's refusal
   (G0.2, item 12); the registry's and the inspector's ways into the editor (G0.2, item 8); a
   lock on `session.ron`, so two windows cannot share a serial (G0.2's fixes, item 4); and what
   the two re-checks left (item 5). The watchlist, event and date breakpoints and log axes
   were G1's from the start (§9).
5. **What the two re-checks left.** The bounded verification is one pass, one fix round and
   one re-check of the fixed items; G0.1 and G0.2 each had one. Both re-checks found the fixes
   in place. They left these, which the close records and does not fix (STATE.md, O20); G1
   takes them first:
   - After G0.1's fixes, two ways past the no-egui scan: a reach into `ui` through a renamed
     crate root (`use crate as g;`, then `g::ui::…`), and a `use` group whose first root is
     another crate (`use {std::fmt as _, crate::{ui as _}};`).
   - After G0.1's fixes, three mutants of the plots that `every_drawn_vertex_is_recorded`
     misses: a y changed only where a line is lent (scaled on some lines, or moved by one
     affine map on all), and each segment's last vertex dropped from the record and the line
     alike. The painted y and the end of a segment are not yet held to the store. Rerun at the
     close, these five still survive.
   - After G0.2's fixes, five mutants that pass the whole suite. Compare's hedge drops its
     check of the branch's own start. The model's walk to earlier records takes any resumed
     run, not only one whose resume tick is its store's start. The compare pane is handed no
     earlier records. The pane never paints the hedge. And the ancestor is never taken from an
     open run, so the host reads the file instead. The first two look equivalent: a run that
     resumed starts at its checkpoint's tick, where its state is its parent's, and a run that
     reran starts at 0, which the walk refuses. The last three are what no test paints or
     takes.
   - After G0.2's fixes, a branch of a hand-edited tape compares its tolerances with that
     tape, its parent, and says "no". "Ledger changed" reads one generation, not the chain
     back to the base.
   - After G0.2's fixes, a tape with no marker, beside a lineage file that does not read,
     opens as a run, and the log says the lineage does not read. Read strictly, U3 makes it an
     experiment.
6. **Every deviation, by section.** Each is made in place, in the section named, and the item
   named in its block says why. "G0.1" is the first part's block and "second part" is G0.1's
   second part's:
   - §2, U3: a lineage alone, or a marker in one basis alone, makes an experiment (G0.2's
     fixes, 2).
   - §3.1: the recount and licence read with certify in the tree (here, 3).
   - §3.2: the gate script is `scripts/gui.sh` (G0.1, 1); the build script is the cli's (G0.1,
     2); vm/ reads run types only (G0.1, 5); the GUI depends on core, for `num` everywhere and
     for the raw schema, `Basis` and `Unit` in `edit/` alone, held by a scan (G0.1's second
     part, 9; G0.1's fixes, 11; G0.2, 1).
   - §3.3: `Runner::new` takes the build (G0.1, 2); `Loaded`, `Running`, `Refused`, `Ended`
     and `Snapshot` as built (G0.1, 3; second part, 6 and 7; G0.1's fixes, 7); the effects
     and `drive::Host` (G0.1, 4; second part, 10; G0.2, 10); ring ticks (G0.1, 7); each ring
     checkpoint's `prefix_id` (G0.2, 3); a poisoned run's snapshot from the ring (G0.1's fixes,
     1); the ledger line by key (G0.1's fixes, 6).
   - §3.4: series stamped with the tick that ran, gaps as missing pairs (G0.1, 6); stacked
     plots, the `2^k` cache and the record of what is lent (second part, 2 and 3; G0.1's
     fixes, 3); year gridlines (G0.1's fixes, 9).
   - §4: the cursor is a report tick (second part, 1); run until (second part, 4); the
     registry (second part, 5); the log's onsets and ticks (second part, 8; G0.1's fixes, 8);
     plots across tapes (G0.1's fixes, 4); "ledger changed" with three answers (G0.2, 9; G0.2's
     fixes, 1); export (G0.2, 10; G0.2's fixes, 3 and 6); compare (G0.2, 11; G0.2's fixes, 5);
     the layout gains a pane (G0.2, 13); the session and its serial (G0.1, 9; G0.2, 6).
   - §5.1: `materialise` returns the branch's world (G0.2, 2); `plan` reads the parent's store
     (G0.2, 3); a removal of an event and its source param resumes (G0.2, 4); the lineage
     names its tape (G0.2, 5); keys new to the whole session (G0.2, 6; G0.2's fixes, 4); the
     stamp's date (G0.2, 7); the act as text, with the raw pane's caret (G0.2, 8); branches not
     re-made at launch (G0.2, 12).
   - §5.2: a new value copied from a new param resumes; only a change to what the world reads
     reruns (G0.2, 4).
   - §8.1: the scans read `drive/` and `platform/` and `use` trees (G0.1, 8; G0.1's fixes, 2);
     the smoke mode (second part, 12); the goldens' points (second part, 13); no image
     snapshot, since WSL has no lavapipe (second part, 14); U10's test poisons NaN and both
     infinities (G0.1's fixes, 10).
   - §8.2: `scripts/gui.sh` tests in release, names its tests and checks the branch tapes'
     `tape_hash` (G0.1, 1; G0.2's fixes, 7; here, 2); Windows runs both scripts under Git
     Bash, with no `ci/gate.ps1` (G0.1, 10).
   - §9: D2's "no log axes" stays for G1 (second part, 15); what moves to G1 (here, 4).

**Amended at D.3 (2026-09-27), on branch `demo-world`,** when the map landed ahead of its
stage. The user asked for a map of the United Kingdom with Victoria-style lenses over an
illustrative world (docs/demo/WORLD.md), so the map (§6, G4) and the lenses (§4, G2) were
brought forward over the demo tape `tapes/demo-gb.ron`: 93 historic counties, each running
the probe's four roles, 1750–1901. Where this file was silent or wrong against the code, the
build decided, and each change is made here:
1. **The map pane** (§6). `ui/map.rs` triangulates each region's parts once with earcut,
   holes included (`Geo::build`), and draws them each frame as one mesh in the lens's colours.
   The atlas stores each shared border once and the map strokes it once, thinned in screen
   space so that no two kept points lie within 0.75 points: thin county lines, heavier lines
   where England meets Wales and Scotland, the coast in dark ink over a pale band on a muted
   sea. Names are drawn where they fit, the full name or else the Chapman code. Drag pans, the
   wheel zooms about the pointer, a double click fits. A region the tape lacks is hatched grey;
   one with no value is plain grey and its card says why. The hover card names the county,
   the lens's value with its unit and rank, the report tick and date, and the run (tape,
   origin, build, `tape_hash`, `world_id`). A click selects the county as
   `Entity::Node(county.<chapman>)`, which the inspector and the outliner follow by key, and
   the county's card beside the map plots its recorded inputs. A tape whose nodes are not the
   atlas's regions has no map, and one with a node the atlas lacks is refused (§6); either says
   why in the pane.
2. **Lenses** (§4, "Lenses (G2)"). A `LensVm` per lens (`vm/map.rs`) holds every region's
   value, its place on the scale and whether it lies beyond the domain, the legend's marks,
   the counts, and the ranked table as indices into the same regions, so the map's values are
   the table's by construction; `map_values_equal_table` holds what each draws to the other.
   Scales are colorous' viridis for a sequential lens and purple to orange through white for a
   diverging one, which centres on its reference. **Each domain is fixed for the run and
   chosen without the run:** `worlds/demo-gb/lenses.csv` registers it, from the oracle's range
   over every county and step of the history with a margin (WORLD.md §6), and the legend
   follows from it alone. A value outside takes the end colour and is marked below or above.
   The selector lists the 25 lenses by group; `1`–`9` and `0` pick the first ten and `[` and
   `]` step. The two oracle lenses (`gap.oracle`, `gap.wage`) are listed and disabled: they
   wait for `crates/observe`'s `oracle_gap` (§7.3), which does not exist.
3. **Where the measures are defined** (U6, §7.3). observe does not exist, so the lens
   measures live in `rustyecon_worldgen::lens`, beside the table that lists them and the
   compiler that writes the keys they read: `value(measure, readings, first)` over a county's
   recorded prices, cleared volumes, trade flags, rationing lines, params and states. The GUI
   gathers the readings (`vm::map::readings`) and computes no measure; N per tick goes through
   the engine's `ClockMethod::per_tick` for the param's recorded use. The cli calls none of
   them yet. This is WORLD.md §6's departure, recorded: the measures move to observe's
   `measure` when it lands. worldgen's table check now refuses a lens key that names no
   measure, a change lens before its level, and a diverging scale whose reference is not inside
   its domain.
4. **Two measures in the catalogue** (§3.4): `Trades`, the engine's `MarketLine::trades` as 1
   or 0, and `State(field)`, each number of an actor's own state after the tick
   (`Sim::actor_state`), at the new `At::Actor(key)`. Both are recorded for every world.
5. **The lean catalogue** (§3.4, §10's catalogue filter, brought forward). A world of more
   than 16 nodes records each market's price, supply, demand, cleared volume and trade flag,
   each class line's requested and filled, each actor's state, every param, and the run's
   margins; no next price, EMA, fills, feasible line, settle line, holding, audit line or drift
   by good. The model sends `Cmd::Catalogue(Catalogue::Lean)` before such a tape's load, logs
   it, and opens it with nothing plotted: 372 prices would stack four panels of 93 lines. The
   demo's whole catalogue would be 11,447 series; the lean one is 5,970. The Extractor finds
   each cell's series at its place in the last tick's layout before it searches its index.
6. **Series in stretches** (§3.4). A series keeps each unbroken run of ticks as its first tick
   and the index of its first value, so a series with no gap costs 8 bytes a point, not 16. The
   points and the gaps are the same. `Series::ticks` now makes a list; `iter`, `points(from,
   to)`, `tick(i)` and `first` read it in place.
7. **Panels at 30,000 events** (§4). The demo tape has 30,078 dated steps and 31,116 params.
   The timeline lists every firing when the span holds at most 2,000 (the gate's and appb's
   are unchanged), and otherwise the firings within a year of the cursor, with every firing
   counted a year at a time on the strip. The outliner's view-model is kept while the run's
   load, the selection, the pins and the plots are unchanged; a filter narrows it, a section
   lays out at most 300 rows and says how many more, and long sections start closed. The
   registry finds each param's last copy in one pass over the fired events, and
   `firing_date` finds a tick's events by bisection. The log's view-model grows with the log
   rather than being remade each frame, and the rationing watch learns each class line's
   series once and reads a row through a column by catalogue index (a demo row holds 5,970
   cells and 1,116 class lines); its onsets and their order are unchanged.
8. **The layout** (§4). The map is a tab beside the plots in the centre. The pane a tape
   needs is brought forward once per load: the map for a tape with one, the plots for one
   without, so the gate's scripts open on their plots as before. A new layout gives the centre
   half the width and the top row two thirds of the height. A `layout.ron` saved before the
   map gains it as a tab beside the plots.
9. **Dependencies** (§3.1, D6). `earcut =0.4.11` (G4's pin) and `colorous =1.0.16` (G2's)
   join the workspace, and the GUI depends on `rustyecon-worldgen` for the atlas and the
   lenses. worldgen names core for `num` and its own tables; the GUI's scan
   `the_gui_names_core_for_num_alone` still holds.
10. **Tests** (§8.1's map row, G4's gate). `tests/map.rs`:
    `the_atlas_triangulates_and_labels_hit_their_regions` (every node key has a region; each
    region's triangles cover its drawn area to a relative 1e-9; each label point lies in its
    own triangles), `every_region_drawn_once` (each fill once, each border stroked once, and a
    one-county tape hatches the other 92), `map_values_equal_table` (every lens at report
    ticks 0, 51 and 259), `lens_domains_are_fixed_for_the_run`,
    `demo_lens_view_models_equal_their_goldens` (25 files in
    `tests/golden/demo-gb-tick0/`), the kittest script
    `the_demo_script_switches_lenses_runs_hovers_and_selects`, and the ignored measurement
    `map_mesh_frame_time_is_recorded`. worldgen's `tests/lens.rs` holds the measures.
    `scripts/gui.sh` names 48 tests. On the committed tree both gates are green on WSL and on
    Windows: the GUI passes 77 tests with 2 ignored, the workspace 575 with 3 ignored, and the
    gate world's final hash is still `0x61f9c8529131ff17` and appb's `0xe1fa082b26995867`.
11. **Measured,** on Windows in release, with the machine shared with branch
    `phase2-markets`'s runs throughout, so each figure is one run's and loose:
    - A frame's map at 93 regions, the fill mesh of 124,779 vertices and 2,446 thinned strokes
      on a 1,260 × 900 canvas: a median of 1.71 ms over 60 frames
      (`map_mesh_frame_time_is_recorded`; 1.52 ms on WSL, load average 150). Triangulating the
      atlas once: 21 ms (80 ms on WSL).
    - The Extractor on the demo world: 0.28 ms a tick for the whole catalogue of 11,447
      series, and 0.21 ms for the lean one of 5,970, beside the engine's step of 2.5 ms.
    - A Runner ran the demo to 1901 into a store in 22.7 s, 346 ticks a second, 0.8 s of it
      ingest, and held 46.9 million points, 375 MB of values. Its final hash is the cli's,
      `0x9c78e47631ea8224`.
    - Headless (kittest, the run capped at 520 ticks a second), a running frame took p50
      4.51 ms and p90 7.61 ms with the map in front, and 2.56 ms and 4.52 ms with the plots in
      front instead. Before the map's view-model was made at most 30 times a second, it was
      p50 7.19 ms with the map.
    - The smoke mode, `rustyecon-gui --smoke 7851 tapes/demo-gb.ron`, with the outliner, the
      map, the inspector and the timeline drawn, ran to 1901 in 19.2 s. CPU per frame while
      running was p50 8.49 ms and p90 10.62 ms, with a first frame of 1.8 s that reads the
      tape; paused, p50 was between 5.0 and 7.5 ms over three runs. G4's bar, a p90 under
      8 ms while a run streams, is not met on this machine. Most of what is left is
      tessellating a 125,000-vertex mesh every frame; a coarser mesh for the country-wide view
      is the next lever.
    - Opening the tape parses 12 MB of RON on the UI thread (0.2 s here, 1.7 s on a loaded
      WSL), and the run loads on its worker in about 4 s.

    Nobody has looked at the window; the images checked were rasterised from the shapes a
    headless frame paints.
12. **Not done, and still G4's:** channels on hover, ports and coalfields drawn, a difference
    lens against a parent run, the world editor, a coarser mesh for the country-wide view, the
    frame's p90 under 8 ms on Windows while a run streams, and an E42 run's spill. The map of a
    branch shows the branch; comparing two runs on the map waits for the difference lens.

**Amended at D.4 (2026-09-27), on branch `demo-world`,** after the bounded verification of D.2
and D.3 (`D:/rustyecon-demo/verify-map-r1/`, `verify-world-r1/`). Each change has a test that
fails without it; the verifier's mutants were rerun against the new tests
(`D:/rustyecon-demo/fix-r1/mutations.txt`).
1. **The atlas's credit** (§6; ruling 7; data/atlas/ATTRIBUTION). D.3's map showed none, though
   the ODbL attribution asks every map drawn from the atlas to show its two sources. The canvas
   now paints `rustyecon_worldgen::atlas::CREDIT` in its lower right corner, always, with the
   whole ATTRIBUTION on its hover, and records it in `MapFrame::credit`; the demo script paints
   it, and `the_credit_is_painted_clear_of_the_legend` finds it inside the canvas and clear of
   the legend at four widths (above the legend on a narrow canvas). worldgen bundles `LICENSE`
   and `ATTRIBUTION` beside the atlas (`include_str!`), and both binaries print them:
   `rustyecon licences` (`licences_prints_the_atlas_licence_and_attribution`) and
   `rustyecon-gui --licences`.
2. **Lens values against the engine** (U3, U6). `map_values_equal_table` compared the frame with
   the view-model that made it, and the goldens pin tick 0 only, so a value read a tick early, a
   change lens against the cursor rather than genesis, and a trailing window that read past the
   cursor all passed. `lens_values_equal_the_engine` checks sixteen lenses for every county at
   report ticks 0, 51 (behind live) and 259 against each formula applied to the engine's own
   report, params and actor states from a `Sim` the test steps, both change lenses included, and
   that `no.trade`'s window is the ticks up to the cursor and no further.
3. **The mesh built afresh, the palettes and the legend** (U-rules). The harness's first frame
   had no values, so every coloured frame came through the in-place recolouring, and a mesh built
   in the wrong colours on a pan passed. `rebuilt_mesh_colours_are_the_lens_colours` checks each
   region's colour after the first frame, a pan, a switch of lens and another pan, and, for every
   lens, the legend: `MapFrame::legend` records its 64 segments, its marks and its reference, and
   each segment must be the scale's colour at its place, each region's colour inside the segment
   at its value, and each mark where the view-model puts it. `the_scales_are_the_neutral_palettes`
   pins viridis's ends and middle (#440154, #20908c, #fde725) and purple to orange through near
   white, so a reversed or a red-to-green scale fails even when the map, the table and the legend
   move together. The legend's header names the reference only where its mark is drawn (the
   participation lens's 1 lies off its scale).
4. **The hit test at every part.** `every_triangle_hits_its_region` (122,126 triangles, each
   centroid hits its own region) and `hover_and_click_name_every_part` (a hover card at one point
   of each of 1,181 parts, and a click on each riding and on the 235 detached parts of English
   counties) hold detached parts, enclaves and the ridings, which the label points did not.
5. **U4 on the demo tape.** `gui_equals_cli_demo_gb` runs `tapes/demo-gb.ron` through a
   `ThreadDriver` with the lean catalogue to state tick 7,852 and writes its hashes;
   `scripts/gui.sh` diffs them with `rustyecon run tapes/demo-gb.ron --until 7852 --hashes`.
   7,852 is the first tick of 1901, whose steps the run to 7,851 never fired (WORLD.md §8).
6. **`dead` renamed `no.trade`**, "Ticks without trade" (U6). It counts ticks in which a market
   did not trade, not the probe's dead tick, which also counts a market clearing below half its
   oracle volume and waits for observe (O17). The cli still calls no lens measure: that waits for
   observe's `measure`, as item 3 of D.3's block says.
7. **Rationing's domain** is [0, 0.05], not [0, 0.5]: on this history it reaches at most 0.032
   at the first tick of each year, so it showed one colour all run. The domains' derivation is now a test, worldgen's
   `lens_domains_hold_the_oracle_range`, with WORLD.md §6's table of ranges and margins.
8. **Scoring** (U5, §7.3). `certify` seals a run of a tape whose name carries `[illustrative]`
   UNSCORED whatever its criteria (D.2's verification had certified the demo tape PASS against
   criteria written for it), and refuses a tape whose bases say illustrative once its name has
   lost the marker (`certify_refuses_illustrative_tape`); the demo compiler writes illustrative
   worlds only. The scorecard's refusal is still Phase 6's (STATE.md O25).
9. **Tests.** `scripts/gui.sh` names 55 tests, and diffs the demo tape's hashes as it does the
   gate's and appb's.

**The map and lenses, brought forward (2026-09-27).** Closed at D.5 on branch `demo-world`, from
`708167f`. On 2026-09-27 the user asked for "a nice looking map of the UK, and a fairly complex
setup of regions, goods, and history", with lenses "like in victoria" that change colour as a
run goes. So the map (§6, G4) and the lenses (§4, G2) came forward over an illustrative world,
ahead of D8's trigger. The two blocks above say how the map and lenses were built and fixed, and
[docs/demo/WORLD.md](demo/WORLD.md) says what the world is. This block says what landed, what
the close did, and what G4 proper keeps (STATE.md decisions 124–134, O26 and O27):
1. **What landed.**

   | Commit | What landed |
   |---|---|
   | `a8d3f51` D.1 | The atlas, `data/atlas/`: 93 historic counties of the United Kingdom (England 41 with Yorkshire's three ridings, Wales 13, Scotland 33, Northern Ireland 6), from HCBP Definition B's UK file and OpenStreetMap's riding lines, under the ODbL with its own LICENSE, ATTRIBUTION and build script; its loader, `rustyecon_worldgen::atlas` |
   | `f17b447` D.2 | The demo world's tables, `worlds/demo-gb/`; the compiler, `rustyecon_worldgen::compile` and `rustyecon worldgen`; and `tapes/demo-gb.ron`. Each county runs the probe's four roles at C2 from its own oracle point, and 605 ramp rows become 30,078 dated steps, 1750–1901 |
   | `8327c1a` D.3 | The map pane and 25 lenses, the lean catalogue, series kept in stretches, and panels that take 30,000 events |
   | `018cf0b` D.4 | The verification's fixes, each with a test (the block above) |
   | D.5 | This block, STATE.md, the README and two screenshots |

   Northern Ireland is in because a permissive source exists. HCBP Definition B's UK file
   covers its six counties on the same terms as Great Britain (WORLD.md §1;
   data/atlas/README.md).
2. **How to open it.** `cargo run --release -p rustyecon-gui -- tapes/demo-gb.ron` opens paused at
   1750 on "Wage in land". Space runs, and `[`, `]` and the digits pick lenses. The README's
   "Running the GUI" has the commands for PowerShell and WSL.
3. **Two screenshots**, in `docs/demo/`, each 2,000 × 1,250 at one pixel a point:
   - `map-1801-wage-in-land.png`: 1801-01-01's tick, with Inverness-shire hovered;
   - `map-1901-output-since-1750.png`: the first tick of 1901, on a diverging change lens, with
     the West Riding selected.

   egui_kittest rendered them headlessly through its wgpu renderer, which prefers a software
   adapter. On Windows it took DX12's WARP, "Microsoft Basic Render Driver". WSL still has only
   NVIDIA's ICD and no lavapipe, so no image is taken there, and none is gated (§8.1; G0.1's
   second part, item 14). The renderer ran from a scratch crate, `D:/rustyecon-demo/close/shot/`.
   It depends on the worktree by path and turns on egui_kittest's `wgpu` feature for itself, so
   the workspace's dependencies and lockfile are unchanged. The chip names build `018cf0b`,
   clean. D.3's images were rasterised from the shapes a frame paints, so these are the first
   from the real renderer.
4. **The gates at the close**, on `018cf0b`'s code (this commit changes only docs), with logs in
   `D:/rustyecon-demo/close/`:
   - `scripts/gate.sh` is green in WSL (183 s, fresh target) and on Windows under Git Bash (200
     s, fresh target). 579 tests pass in the workspace with 3 ignored and run by name, and
     certify alone passes 67 with 1 ignored. There are zero warnings, and the gate hash is
     `0x61f9c8529131ff17`. Both certificates PASS and are byte-equal, the probe's pins hold, and
     `derive.py --check` passes. `demo_runs_to_1901` ends at `0xfad880fe08d06645`, hash stream
     `0xdb63cc96f769fb3e`. Telemetry is identical from two processes.
   - `scripts/gui.sh` is green in WSL (123 s) and on Windows (162 s). 84 tests pass with 2
     ignored measurements, the 55 named ones by name, and fmt and clippy are clean. Five hash
     diffs are equal: gate at 2,080 ticks (`0x61f9c8529131ff17`), appb at 20,000
     (`0xe1fa082b26995867`), demo-gb at 7,852 (`0xfad880fe08d06645`), and the two branch tapes.
   - Recorded, not gated: the cli's per-tick hashes of all three tapes are byte-identical on WSL
     and Windows.
5. **The smoke mode on Windows** at the close. `rustyecon-gui --smoke 7852 tapes/demo-gb.ron`
   drew the outliner, map, inspector and timeline, and ran to 1901 in 16.1 to 16.7 s. Over
   three runs, CPU per frame while running was p50 5.0 to 5.9 ms and p90 6.1 to 8.3 ms, and
   paused it was p90 4.8 to 6.6 ms. G4's bar is a p90 under 8 ms while a run streams. It held in
   two runs of three; D.3's loaded runs gave 10.6 ms. So the bar is not yet held, and the
   coarser mesh stays the lever. The window opened and closed by itself; nobody looked at it.
6. **G4's gate, on the demo tape** (§9).
   - §8.1's map tests pass: twelve in `tests/map.rs`, all named by `scripts/gui.sh`.
   - The map's values equal the table's, because one `LensVm` makes both. Both equal the
     engine's own numbers (`lens_values_equal_the_engine`).
   - The ODbL licence and attribution are in `data/atlas/`. The map paints the credit, and both
     binaries print the two files.
   - Not met: the frame's p90 (item 5), and an E42 run's spill, which needs E42.

   G4 is not closed. It opens on D8's trigger, with the research world.
7. **What G4 proper keeps, and the stages after it:**
   - **The research world from Phase 4.** `worldgen::compile` over the research tables, with
     `Measured` bases from the atlas and the spine, per-line bases (§7.3's Phase 4 row) and
     D8's trigger. The demo compiler writes illustrative worlds only. The atlas, the map and
     the lens machinery carry over; the demo's tables do not.
   - **Record overlays** (G5). County series on the map and in the card, with source, vintage
     and bands, the origin "record" (§6), and provenance chips.
   - **Flows on channels** (G4, G9). Channels on hover and trade flows need carriers (WORLD.md
     §7), which need transport desks, home-node trading (Phases 4 and 9) and an oracle with
     trade. Ports and coalfields are drawn with them.
   - **From D.3's item 12:** a difference lens against a parent run, the world editor, a
     coarser mesh for the country-wide view, the p90 bar, and an E42 run's spill.
   - **The oracle lenses and the measures.** `gap.oracle` and `gap.wage`, and the lens measures
     themselves, move to `crates/observe` (G2, O17). The cli calls them there.
   - **Scoring.** The scorecard refuses the illustrative marker, and the identity chip shows it
     (Phases 6–7; O25).
8. **What the re-check left** (decision 134; O26). D.4's fixes had one bounded re-check, in
   `D:/rustyecon-demo/verify-map-r2/` and `verify-world-r2/`. It found the fixes in place, and
   17 of its 25 mutants of the map were killed. It left these, which the close carries and does
   not fix:
   - **Eight mutants of the map pass the whole suite:**
     - the county card's value read a tick early, and its change lens read against the cursor
       instead of genesis;
     - the app handing the map no cursor;
     - the legend painted in a reversed scale while `MapFrame::legend` records the right one;
     - a fresh mesh whose vertices after each region's first take a neighbour's colour;
     - the in-place recolouring skipping each region's last vertex;
     - the credit painted transparent, or placed off the canvas.

     So the card is not held to the engine, and `MapFrame` records what the painter should paint
     rather than what it painted.
   - **The credit clips on a narrow canvas.** A canvas narrower than the credit's longer line,
     about 470 points, clips it. At a 1,280 × 800 window its second line loses 19% of its
     width. At 1,024 × 768 its two lines lose 19% and 37%.
   - **The clock is not held to 52 ticks a year.** The compiler accepts `ticks_per_year` of 1,
     2, 4 or 12, and the dials are held to C2, which was registered at 52. At 4 a year the tape
     runs to 46,548 dead county-ticks and 46,298 ticks with a transfer shortfall. At 1 and 2 D̂
     is infinite. At 12 no tick is dead.
   - **Histories at the guard's edge ran cleanly.** The re-check bisected to the largest move
     each bound lets through. N and T ×4.42 over 20 years (Middlesex), η ×0.374 over 10 years
     (Bedfordshire), a yearly square wave in g0 of 0.906, and a combination each compiled and
     ran with no dead tick and no shortfall.
9. **Seen in the screenshots**, for your look; none is a gate item:
   - at the fitted view, the legend and the credit cover Cornwall and part of Devon;
   - with a long lens name, the ranked table's value column and the selected county's header
     are cut at the pane's edge (0.846806 shows as 0.846);
   - on a 1,600-wide window the health chip wraps to five lines, and the toolbar grows with it.
10. **Every deviation, by section.** The D.3 and D.4 blocks list their changes, and U5 was
    amended in place at D.4:
    - §2, U5: `certify` seals the illustrative marker (D.4, 8).
    - §3.1: `earcut` and `colorous` pinned, and the GUI depends on worldgen (D.3, 9).
    - §3.4: the lean catalogue, series in stretches, and the catalogue's `Trades` and `State`
      (D.3, 4–6).
    - §4: lenses as built (D.3, 2; D.4, 2, 3, 6 and 7), and panels at 30,000 events and the
      layout (D.3, 7 and 8).
    - §6: the map as built and its credit (D.3, 1; D.4, 1 and 4), and Northern Ireland in the
      atlas (here, 1).
    - §7.3: the measures live in worldgen until observe (D.3, 3).
    - §8: the tests and the gate script (D.3, 10; D.4, 5 and 9), and the screenshots (here, 3).
    - §9: G4 brought forward in part (here), and the Phases 6–7 row (D.4, 8).

**Amended at G1 (2026-09-27), on branch `g1`,** when the oracle lab and G1's panels landed:
G1.1–G1.10, from `reboot` at `16eb728`, where G0 and Phase 1's gate were both met. §9's G1 list
is built (the lab, a field over x, sweeps, the explainer and the waterfall, log axes, the
watchlist, event and date breakpoints, PNG snapshots), with the engine step G0's close asked
for, O26's map items and two of O20's. Where this file was silent or wrong against the code, the
build decided; each change is here, and STATE.md's decisions 200–219 record each choice:
1. **No edge to core** (§3.2; G1.1). The engine re-exports `num` and the tape's raw schema
   (`rustyecon_engine::raw`), each whole, and its prelude gains `Basis` and `Unit`; ENGINE is
   amended at G1.1 and its frontend guard allows exactly these two modules. The GUI drops
   `rustyecon-core`, so core's writer is out of its reach by type again, and the scan
   `the_gui_names_core_for_num_alone` becomes `the_gui_reaches_core_through_the_engine_alone`:
   no source names core, and `raw` is named in `edit/` alone.
2. **The oracle lab** (§9; G1.2). A Lab tab beside the plots and the map, which needs no tape
   and drives no run: its form is the panels' own state, never the model's or the session's.
   `lab/` (egui-free, reaching the oracle and nothing of the runs, the model or the files; the
   scan `lab_reaches_no_run_model_file_thread_or_clock`) holds an instance of any unit 1a–1f
   as the oracle's own parameter types, with 16 presets, two to four a unit, built as the
   oracle's gate tests build the goldens' instances (G1, G3 η 0.3, G5 flow and durable; C3, C7
   gap; M3, M4; B1, E1; K1, Q1; TX, GB, C1). Every number of an instance is a knob by its path
   (`schedule.eta`, `categories[1].weight`, `machine_types[0].build.labor`, …); the structure
   is the preset's. A solve goes through the unit's own `new` and `solve`, and every output its
   `outputs()` lists is shown as the dump prints it (a float's shortest round-trip digits), so
   the screen is the oracle's double. Each output is paired with the golden of its preset's
   prefix and its own key (`G1_X_STAR`, `PUB_G1_X_STAR`), the six goldens files bundled: a
   generator's golden agrees within 1e-12 relative, a published one within 5e-6 absolute (the
   oracle gate's bars); a golden no output pairs with is listed apart; an edited instance shows
   none. §7.3's `Point::outputs()` is not built: the lab reads a point's numbers from the
   oracle's own `Debug`, which prints every field by name and every float in round-trip digits,
   so it keeps no copy of the fields and the oracle is unchanged (track 1g edits it on another
   branch). f(x) is the point's own `excess_demand`.
3. **A field over x** (§9; G1.2): any field of the unit's point at 401 even x in [0, 1], with
   the bracket's end 1e-12 and x* among them, the root and the bracket [1e-12, 1] marked, the
   field at both ends and at x*.
4. **One-parameter sweeps** (§9; G1.2): a knob, a range and a count (200 by default), each
   point validated and solved by the oracle, the outputs asked for plotted with a gap where a
   point has no equilibrium, and the regimes counted. They run on the UI thread when asked for.
5. **The price-step explainer and the log waterfall** (§4, "Why is this price 12.3?"; G1.3),
   in `vm/pricestep.rs` and the market inspector. The explainer calls markets' own `imbalance`
   and `next_price` on the tick's recorded p, S and D and on k, the price rate's value at that
   tick converted by the good's own site (`Site::convert`), and says whether its result equals
   the run's recorded next price bit for bit; the lean catalogue records no next price, and it
   says so. The waterfall is ln(p_t/p₀) as Σ k·x over the ticks before t, a bar a year stacked
   from the year's start (a tick each below two years), the line of ln(p/p₀), the residual
   (held one-sided ticks, prices an event moved, rounding) and every event fired in the span,
   flagged where it acts on this price or its rate. Both are display transforms of recorded
   numbers and of markets' own functions (U6).
6. **Breakpoints by key and date** (§3.3, §4; G1.4). `Breakpoint` gains `OnEvent(Key)` and
   `OnDate(Date)`. A run pauses after a tick in which the event fired, every occurrence of a
   recurring one, and after the tick a date falls in, once, as "run until" a date does; a
   step stops there too, and a run already past a date does not pause at it. An event beats a
   date in one tick. `PauseReason` and `RunStatus` are no longer `Copy`. The log pane lists
   the breakpoints and reads one from its field, a date `YYYY-MM-DD` or else an event's key
   (`Intent::BreakAt`, which logs a key the world lacks); the event inspector sets one on its
   event (`Intent::Breakpoint`). A breakpoint changes when `step` is called, never what it
   computes: `gui_equals_cli`'s script now pauses at `mine.cut` and at a date, and still hashes
   as the cli.
7. **The watchlist** (§4, the outliner at G1; G1.4). Watched series by key open the outliner,
   each with its value at the cursor, its unit and the change from the tick before
   (`vm/watch.rs`); "watch" sits beside "plot" in the outliner and the inspector.
8. **Log axes** (§3.4; G1.4). A panel's "log scale" toggle, by unit. The cache lends each kept
   vertex (t, v) with v > 0 as (t, ln v) through the engine's `num` and splits the line where
   v ≤ 0; the axis is labelled "<unit> (log scale)" and marked by v. ln keeps order, so each
   column's least and greatest stay drawn, and `every_drawn_vertex_is_recorded` holds a log
   panel's vertices to ln of the record, bit for bit.
9. **The session** (§4, U8; G1.4). `session.ron` format 3 keeps the watchlist, the log scales
   and the new breakpoints. This build reads format 2, with none of either, and writes 3; a G0
   build sets a format-3 file aside, as it does any format it does not know.
10. **PNG snapshots, never citable** (§9; G1.5). The toolbar's Snapshot paints a banner across
    the window's top ("NOT CITABLE · rustyecon GUI snapshot", the build, the focused run's
    tape, origin, `tape_hash`, `world_id` and tick, and the lab's unit and preset) and asks egui
    for a picture of a later frame. The PNG's tEXt chunks say the same (`Comment`: "NEVER
    CITABLE: …"), and it is written to `snapshots/` beside the session under a new name, never
    over a file. The encoder is `png =0.18.1`, which eframe's image loader already brings in; the
    lockfile gains only the GUI's edge to it. With no session directory, or no picture after
    600 frames (a headless window has no renderer), the log says why.
11. **Other charts' colours** (§3.4; G1.4). The lab's curves and sweeps and the waterfall paint
    in colours of their own (`ui::plots::OTHER`) and with no rotated axis label, so the plots'
    tests tell the plots' lines, cursors and axes from theirs.
12. **O26, the map's part** (G1.6). The credit wraps each line to the canvas and keeps its panel
    inside it; `the_credit_is_painted_whole_on_a_narrow_window` reads the painted text at four
    sizes down to 480 × 600. The eight surviving mutants of `verify-map-r2` are each killed by a
    test that reads what was painted or what the engine gives: the county card held to the
    engine (V7, V8), the cursor behind live in the demo script (V9), every painted mesh vertex
    and legend segment (C6–C8), the credit painted and seen inside its clip (A2, A3). Left: the
    clock held to 52 in the compiler, and the screenshots' three layout notes (STATE, O26).
13. **O20, the scans' part** (§8.1; G1.7). A `use` tree that opens with a group gives each
    branch its own root, and a name a `use` gives a root (`crate as g`) is a root of that file,
    so both ways past the no-egui scan G0.1's re-check found are closed. The rest of O20 stays.
14. **The toolbar's chips** (§4; G1.9). A chip starts a new row whenever less than 360 points
    of its row are left, measured before the wrap; before, a chip that began a row with a few
    points left wrapped into a column one word wide and pushed the tiles below the window (the
    Snapshot button made the gate's branch script do so at 1,600 points).
    `the_toolbar_keeps_its_chips_whole` checks four widths down to 800 × 700.
15. **Tests** (§8.1, §8.2). `scripts/gui.sh` names 75 tests, up from 55, and on Linux runs
    `a_200_point_sweep_builds_in_under_16_ms` by name (G1 over N from 2 to 8, x*, v, Y and N_a:
    a median of 0.75 ms of 20 on WSL). The new files are `tests/lab.rs`, `tests/pricestep.rs`,
    `tests/watch.rs` and `tests/snapshot.rs`, with scripts in `tests/app.rs` and `tests/map.rs`;
    `every_presets_sweep_time_is_recorded` is ignored and not gated (1e's sweeps take about
    180 ms).
16. **Not built at G1, still G1's** (the block "Closed at G0.3", item 4): the plots' overlay,
    difference and ratio against a parent; re-making branches at launch with §5.1 item 7's
    refusal; the registry's and the inspector's ways into the editor; a lock on `session.ron`;
    and the rest of O20. And §9's window check by hand, the p90 frame under 16 ms, which is the
    user's.

## 0. Rulings and decisions

**Rulings (2026-09-25),** numbered here 1–4; they are ADDENDUM's rulings 5–8.
1. The GUI is built with egui (eframe) in Rust, as `crates/gui`.
2. The shell comes right after Phase 0's two sessions, and Phase 1 runs in parallel. The ruling's
   wording is "session 1 = P0.1–P0.7, session 2 = the certify stack". ENGINE's P0.5 amendment 1
   merged two of those steps; session 1 closed at P0.9 after three rounds of review fixes.
3. Yorkshire is split into its three ridings, giving 42 regions. The geometry comes from HCBP
   Definition B, with OpenStreetMap's riding lines. The atlas ships under ODbL with attribution, in
   its own data directory with its own licence file, and the code licence is unaffected.
4. R16 joins PLAN §4: frontends observe; the tape decides. PLAN §4 has its text.

**Decisions D1–D13** (A14; STATE.md's decisions 22–34) are carried out here:
- D1, the GUI never gates engine work (§8.2).
- D2, G0's timing and cut (§9).
- D3, edits are always `Assumed`, with a lineage, a branch marker and the scorecard's refusal
  (§5.1, §7.3).
- D4, resumes are verified (§3.3).
- D5, the fields "origin" and "registry" (§2).
- D6, exact pins (§3.1).
- D7 and D8, the region keys and when the map arrives (§6).
- D9, levels are deferred (§5.2).
- D10, four engine prerequisites, one of them already met at `a8f9ed8` (§7.2).
- D11, wasm hashes are recorded (§8).
- D12, trigonometry only in `ui/` (§2).
- D13, observe (§3.2).

**From the design contest.** Two of the three judges chose shell first. One seam stays fixed as the
GUI grows (Intent → Cmd → Runner → Obs → Store → view-model), and each G-stage sits beside one
phase. Data first's full recording becomes a hypothesis that G3 tests (§3.4). Workspace first's
levels wait (D9).

## 1. Purpose and non-goals

The GUI is a research instrument for the author and, later, for readers of the paper. It runs the
engine live, plots any recorded series, explains where a number came from, compares a branch with
its parent, and edits the tape.

It never carries a number into a run except through the tape (R16). It computes no measure the
engine defines and tunes no dial against a scored target. It offers no action that copies a record
or oracle value into a tape. The 1750 genesis is seeded at the oracle's equilibrium through
worldgen, with its own basis (PLAN §3.4).

It draws no geography, flow or trend that no source supplies. It hides no depth in tooltips only,
and it colours nothing good or bad. It is not a game: no goals, no score, no narrative
notifications, no decorative 3D.

## 2. Rules the GUI obeys

R16 is the plan-level rule (A14). U1–U12 carry it out. They were called G1–G12 in the first draft,
which clashed with the G-stages.

- **U1 — The GUI never mutates engine state.** *(E1, E4; R8.)* One struct, the `Runner`, owns a
  `Sim`. Pause, speed and seek change when `step()` is called, never what it computes. A resume
  starts only from a checkpoint that a Runner of this process took, or from a hash-verified file
  (D4, §3.3).
- **U2 — Every intervention is a tape edit with provenance.** *(E1; R4, R8, R16.)*
  - Every edit carries a required note.
  - Every entry the edit adds or changes carries its own basis, stamped `Assumed("GUI experiment
    <date>: <note>")`. The GUI chooses no basis (D3).
  - The GUI does not edit a number that has no basis of its own.
  - Every branch's name carries the marker `[GUI experiment <date>]`.
  - A saved tape carries a lineage file (§5.1).
- **U3 — Every number names where it came from.** *(R4, R9, R11; N10.)*
  - A simulated number names its run (build commit and dirty flag, `world_id`, `tape_hash`) and its
    tick. A record number names its source and vintage. An oracle number names its build and
    instance.
  - Each number has an **origin**: run, experiment, record or oracle. A run is an experiment when
    either holds:
    - `materialise` made its tape. The tape then has a lineage, held in memory or as
      `<name>.lineage.ron` beside the opened file.
    - Its name, or any basis in it, carries the GUI-experiment marker.
  - From Phase 3 each number also has a **registry** class (R5). "Tier" keeps the book's meaning
    (D5).
- **U4 — A GUI run hashes equal to the cli's run of the same tape, on the same platform and
  build.** *(R8.)* Across platforms, and between native and wasm, equality is recorded, not gated
  (A5, D11).
- **U5 — The GUI shows the oracle and the record; no GUI-edited tape is scored.** *(R5, R13, R15.)*
  - The scorecard scores only tapes whose `tape_hash` the fitting harness registered before the
    run. It refuses any tape whose name or any basis carries the GUI-experiment marker (D3, §7.3),
    or the illustrative marker, `[illustrative]` in the name or a basis beginning
    "illustrative" (docs/demo/WORLD.md; amended at D.4).
  - From G5 the lineage lists the scored series that were overlaid during an edit.
  - Until Phase 6 registers the moment table, these guards are procedural. One part is built
    (D.4): `certify` seals a run of a tape whose name carries `[illustrative]` UNSCORED, never
    PASS, and refuses a tape whose bases say illustrative once its name has lost the marker.
  - The oracle gap is computed beside the run and never fed to it.
- **U6 — The GUI computes nothing the engine defines.** *(R4, R10.)* It may difference, divide,
  take logs and decimate for display. Price steps, per-tick values, derived measures and the oracle
  gap come from functions the cli also calls.
- **U7 — Keys, not ids.** *(E8.)* Selections, pins, plots, diffs, breakpoints and exports name
  entities by tape key. Dense ids are resolved against one `World` at the time of use.
- **U8 — Session state stays out of the tape.** *(E1.)* Runs, pins, plots and breakpoints live in
  `session.ron`. The tile layout lives beside it in `layout.ron`.
- **U9 — The workspace rules apply.** *(R8; A5.)*
  - No hashed containers.
  - Transcendentals go through `num`.
  - Drawing trigonometry is allowed only in `ui/`, under `#[expect(clippy::disallowed_methods,
    reason = "display only")]`. A scan forbids it in `model/`, `run/`, `edit/` and `vm/`, and
    `num` gains none (D12).
- **U10 — NaN fails.** *(R9.)* A non-finite value at ingest stops ingestion and names the series
  and tick.
- **U11 — Build only what the engine produces.** *(R10.)* A panel arrives with its data's phase.
- **U12 — Scoring is labelled.** *(R1, R15.)* A verdict is a rejectable test of the configuration
  reading, never evidence for the paper's theorems. The Appendix B match is a reproduction check.

## 3. Architecture

### 3.1 The stack, pinned with `=` (D6)

| Crate | Version | Licence | Min Rust | Role |
|---|---|---|---|---|
| eframe, egui | 0.36.2 | MIT OR Apache-2.0 | 1.95 | default features: wgpu 30.0.1, winit 0.30.13, accesskit |
| egui_plot | 0.37.0 | MIT OR Apache-2.0 | 1.95 | time series; no decimation, no log axis |
| egui_extras | 0.36.2 | MIT OR Apache-2.0 | 1.95 | `TableBuilder`; `default-features = false` |
| egui_tiles | 0.17.1 | MIT OR Apache-2.0 | 1.95 | tiling and tabs; serde on |
| rustyecon-certify (G0) | workspace | workspace | 1.97.1 | the manifest (§4). Native only (`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`) if session 2's move brings in parquet or zstd; July's certify pins `parquet = 59.1.0` with snap and zstd |
| web-time (G2) | 1.1.0 | MIT OR Apache-2.0 | 1.60 | an `Instant` that works on wasm |
| rfd | 0.17.2 | MIT | – | file dialogs; `AsyncFileDialog` on the web |
| mimalloc | 0.1.52 | MIT | – | allocator, native binary only |
| colorous (G2) | 1.0.16 | Apache-2.0 | 1.60 | neutral sequential and diverging scales |
| parquet (G3) | certify's exact pin (the probe used 60.0.0) | Apache-2.0 | – | the spill, native only: `default-features = false`, feature `lz4` (lz4_flex, pure Rust) for LZ4_RAW with byte-stream-split |
| earcut (G4) | 0.4.11 | MIT OR Apache-2.0 | – | triangulates regions once |
| egui_kittest (dev) | 0.36.2 | MIT OR Apache-2.0 | 1.95 | headless UI tests, feature `eframe` |

- **Pins (D6).** The crates are upgraded together, and only at a G-stage boundary. egui has broken
  its API every 6 to 24 weeks (0.33 on 2025-10-09; 0.34–0.36 on 2026-03-26, 06-25 and 08-05), and
  it needs Rust 1.95.
- **Licences and lints.**
  - Without certify, the chosen stack has 268 crates on Linux, 181 on Windows and 120 on wasm32.
    That is `cargo tree -e normal`, root included; with build dependencies the counts are 275, 190
    and 124. The count was made offline, on a scratch manifest with the spike's lockfile. The same
    method gives the spike's own tree (with egui_dock and colorgrad) 273, 186 and 127.
  - Session 2 fixes certify's tree. G0 re-runs the count, the licence read and D1's check timing
    with certify in the tree.
  - As re-run at G0, with certify in the tree (WSL, the same method): 274 crates on Linux, 187
    on Windows and 125 on wasm32; 281, 196 and 129 with build dependencies. G0.1 counted them
    on the crate's seams and the close again on the finished crate, with the same result: the
    lockfile has not changed since G0.1's second part. The licences are as above, with 2 crates
    under BSL-1.0 on Windows, and `self_cell` and `r-efi` each offering a permissive licence
    among others (STATE.md).
  - All the crates are permissive (18 under Unicode-3.0). The exception is the default fonts
    (OFL-1.1, Ubuntu Font Licence 1.0), whose notices ship with every build.
  - The spike was clippy-clean under the default lints only. The workspace's `clippy.toml` rejects
    its f32 `exp`, `sin` and `cos` (D12).
- **wasm.** The raw `.wasm` was 12.3 MB (3.7 MB gzipped), measured before wasm-bindgen and with the
  rejected crates still in. A toy kernel ran 1.4–1.7× slower in Node than natively (1.5–1.8× in the
  re-run). G2 measures the gate world.
- **mimalloc** (headless, median of 60 frames):
  - It cut mesh building 3–10×.
  - On Windows it cut the thinned outlines 2.6×, and the 100k-point line from 6.7–6.9 to 3.4 ms.
  - It left decimated plots alone.
  - It slowed WSL's thinned outlines at 700 regions from 1.7–1.9 to 2.4–2.6 ms.

  It stays, because the window runs on Windows.
- **Not chosen:**
  - egui_dock 0.21.1 (MIT only, one owner).
  - walkers 0.60.0 (Web-Mercator tiles, about 100 more crates).
  - colorgrad.
  - glow.
  - eframe's `persistence` feature, which pulls ron 0.12 in beside 0.8.1.

### 3.2 Crates and modules

```
crates/gui       rustyecon-gui: lib + bin. Depends on rustyecon-engine, rustyecon-certify (the
                 manifest; native only if it brings in parquet), oracle (G1) and observe (G2),
                 and worldgen (D.3). No edge to core since G1.1: the engine re-exports `num`
                 and the tape's raw schema, with `Basis` and `Unit` in its prelude, so core's
                 writer is out of reach by type; a scan holds that no source names core and
                 that edit/ alone names the raw schema.
                 Nothing depends on it, and default-members leave it out (D1).
  src/lab/         no egui, model, run, file, thread or clock (G1): an oracle instance of any
                   unit, its presets, knobs and goldens, and a point's fields read from the
                   oracle's Debug
  src/main.rs      native entry: mimalloc, a tape path, run_native; the cli's build.rs, shared
                   (`build = "../cli/build.rs"`), stamps the commit and dirty flag (G0.1)
  src/web.rs       wasm entry (G2 proof, W1)
  src/app.rs       eframe::App: its drive::Host drains the drivers into reduce(); ui() draws;
                   supplies the wake callback
  src/model/       no egui: Model, Intent, Effect, reduce(), Session (serde)
  src/run/         no egui: Cmd, Obs, Runner, Extractor, in-memory Store and its decimator,
                   checkpoint ring, log lines; no model, file, thread or clock (G0.1)
  src/edit/        no egui, model, file, thread or clock: TapeEdit, materialise, plan, lineage,
                   the form, keys, and export as text (G0.2); reads run types
  src/vm/          no egui: one pure view-model builder per panel; Serialize + Debug; reads run
                   types, never model/ (G0.1)
  src/drive/       ThreadDriver (G0), InlineDriver (G2): the threads, clocks and channels; Host,
                   which carries out the model's effects (G0.1)
  src/ui/          egui only: panels draw view-models and return Intents; plot cache; tile layout
  src/platform/    rfd and std::fs natively, downloads on the web; the Parquet spill (G3)
crates/observe   rustyecon-observe. Phase 2 creates it (§7.3) as a default member, engine-gated,
                 holding measure and oracle_gap. At G2 the Runner, the Extractor, the in-memory
                 Store (which writes chunks to a byte sink, Box<dyn FnMut(&[u8]) + Send>) and vm/
                 move into it (D13). It has no egui, thread, clock, file or std::io: E2's list, as
                 P0.5 amendment 10 extended it. It is wasm-safe.
```

`model/` and `edit/` stay in `crates/gui`, and scans hold the seams (§8.1). Observe depends on the
engine and the oracle. Certify and the cli depend on it from Phase 2. If Phase 2 places the
measures elsewhere, G2 creates observe.

Observe is not on the run's path. Nothing in core, markets, agents or engine depends on it, so the
oracle still reaches no run (R13; ENGINE §1). ENGINE §11's scans cover it all the same (§7.3).

Observe is a default member, so from G2 the Runner and the view-model builders compile at every
engine step, just as the cli's path does. An engine change that breaks them is fixed in that step.
Their goldens, however, live in `crates/gui/tests` and run under `ci/gui.sh` only, so retuning a
world never fails an engine step on a GUI golden (D1).

The cli gains `rustyecon figure build --check` at G6–7, and a sweep entry if Phase 2 rules one in
(§7.3).

### 3.3 State, threads and the web path

```rust
pub struct Model { session: Session, runs: BTreeMap<RunId, Run>, focus: Option<RunId>,
                   cursor: Cursor /* Live | At(tick) */, selection: Option<Entity>, log: Vec<Entry> }
pub fn reduce(m: &mut Model, i: Intent) -> Vec<Effect>;   // the only writer
pub enum Effect { PickTape, ReadTape(String), Spawn(RunId), Send { run: RunId, cmd: Cmd },
                  Close(RunId), SaveSession, SetAsideSession,    // G0.1; drive::Host carries them out
                  PickSaveTape, PickExportDir, Write { job: Job, files: Vec<(String, String)> } }  // G0.2
pub struct RunKey { build: Build /* commit, dirty, target, rustc */, tape_hash: Hex, world_id: Hex }
// certify's RunKey, reused as is (docs/CERTIFY.md §3; S2.6); Hex(u64) serialises as "0x%016x"
pub enum Cmd { Load { tape: Box<Tape>, from: Option<ResumeFrom> }, Run { until: Option<u64>, max_tps: Option<u32> },
               Pause, Step(u64), Breakpoints(Vec<Breakpoint>), Snapshot(u64), Stop }
pub enum ResumeFrom { Ring(RingCheckpoint) /* , File(VerifiedCheckpoint) from G3 */ }
// RingCheckpoint { run: RunKey, tick: u64, prefix_id: u64 /* G0.2 */, bytes: Vec<u8> /* Checkpoint::to_bytes */ }:
// opaque, minted only by a Runner
// VerifiedCheckpoint: made only by verify() (G3)
pub enum Obs { Loaded { run: RunKey, tape: Box<Tape>, world: Box<World>, tick: u64, hash: u64 }, Running { tick: u64 },
               Batch(ObsBatch), Paused { tick: u64, why: PauseReason }, Checkpointed(RingCheckpoint),
               Snapshot(Box<Snapshot>), Failed { error: RunError, last: Option<Box<TickReport>> },
               Refused(Refusal /* Load(LoadError) | Decode(CheckpointError) | Resume(ResumeError) */),
               Ended /* a driver's: its worker ended without a Stop */ }
impl Runner { pub fn new(build: Build, sink: Box<dyn FnMut(Obs) + Send>, wake: Box<dyn Fn() + Send + Sync>) -> Runner;
              pub fn handle(&mut self, c: Cmd);  pub fn advance(&mut self, max_ticks: u32) -> Progress /* { ran, busy } */;
              pub fn busy(&self) -> bool; }
pub trait Driver { fn send(&mut self, c: Cmd); fn poll(&mut self, out: &mut Vec<Obs>); }
```

As built at G0.1 (amendment items 2–4 above): the Runner names the build in every run key and
ring checkpoint it mints; `Loaded` hands the model the run's `World`; `Running` lets the store
hold the run's status; and a `Refusal` also covers a tape that does not load and ring bytes that
do not decode, which `ResumeError` cannot carry. A resume from the ring that is refused is
reported, and the Runner then loads the tape from genesis.

As built at G0.1's second part (amendment items 6, 7 and 10 of its block): `Loaded` also hands
over the tape, whose listing (`engine::registry`) the store makes once; a `Snapshot` carries each
holding's lots; the model asks for a snapshot when an actor is selected and the run is not
running; and `Effect::PickTape` opens the native file dialog.

As fixed after G0.1's verification (items 1, 6 and 7 of its block): a poisoned run's own
state is never read, so a snapshot of its current tick is rebuilt from the ring like any
earlier tick's; the health chip paints the ledger line by key beside the engine's; and
`Obs::Ended` is a driver's report that its worker ended without a Stop, which a Runner never
sends.

As built at G0.2 (items 3, 5, 7 and 10 of its block): each ring checkpoint carries the
`prefix_id` its checkpoint stores, and the store keeps the hash of the state a run loaded at,
so compare and a manifest can name a resumed run's start. The model gains the editor's intents
(`Today`, `Stage`, `Unstage`, `Apply`, `RemoveOrphans`, `SaveTape`, `Export` and the two
dialogs) and `Effect::Write`, which the host carries out without writing over any file and
answers with `Intent::Written`. `Intent::TapeRead` carries the text of a lineage beside the
tape. A branch is a run with a parent, and its `Cmd::Load` carries the parent's
`RingCheckpoint` when `plan` found one.

**The run key.** It leaves out `prefix_id`, which changes every tick; `tape_hash` fixes the
schedule. Each value is stamped with its tick, and `prefix_id` names chunks only.

**`tape_hash`.** It is the tape hash that certify's manifest computes. That is session 2's
definition: this crate calls it and never redefines it.
- Session 2 chose core's `fnv1a_64` over the canonical `to_ron` text, as this design proposed:
  `certify::tape_hash(&Tape)` (CERTIFY C2, §3). It covers every field the loader reads, the name
  and the basis texts included, and survives reformatting, comments and list order.
- So the GUI applies it to a parsed `Tape`, a branch's or a base file's alike, and "Save tape as"
  writes that same `to_ron` text.
- Each run's key equals what certify computes for the same tape.

**Native (G0).**
- `ThreadDriver` spawns one worker per run, so a baseline and a branch run side by side. It times
  itself, applies the speed cap, and calls `advance` in slices of about 8 ms.
- The Runner has no clock (E2). It steps, extracts and checks breakpoints. It hands one `ObsBatch`
  per slice to its sink and then calls `wake`. app.rs sets `wake` to `ctx.request_repaint()`, which
  took 17–37 ns headless.
- Channels are bounded from G3.
- A failed run shows `Poisoned`, its ledger line and its last good tick. Its inspector reads
  the state its last good tick left, never the state the failure left.

**Web (G2).**
- `InlineDriver` calls `advance` inside `poll`, under an 8 ms `web_time::Instant` budget.
  `thread::spawn` and `Instant::now` trap on wasm32; this was checked in Node 22.
- Past about 5 ms a tick natively, the Runner moves into a Web Worker (gloo-worker 0.6.0, trunk
  0.21.14).

**Checkpoints (D4).**
- **The ring.** The Runner keeps `Checkpoint::to_bytes` at each ring tick. A ring tick is
  `tick_of(YYYY-01-01)` for every model year, thinned to decades beyond G3's memory budget. It is
  the tick 1 January falls in, which can begin in the old year (G0.1): at 52 ticks a year from
  1750-01-01, 1751-01-01 falls in tick 51, which begins on 1750-12-26. Genesis is one when the
  tape starts on 1 January; the gate world has 41, from tick 0 to tick 2,080.
- **Passing the ring on.** Each ring checkpoint goes out as an opaque `RingCheckpoint` (the run's
  key, the tick and the bytes) in `Obs::Checkpointed`. A branch's new Runner, on its own worker,
  receives its parent's checkpoint in `Cmd::Load`.
- **Deep inspection.** Inspecting tick t resumes a scratch `Sim` from the latest ring tick at or
  before t.
- **Why files need checking.** `Sim::resume` checks the format, `world_id`, `prefix_id` and shapes,
  not the numbers (ENGINE §7.6; `sim.rs:68–94`). A hand-edited checkpoint could therefore carry a
  number into a run from outside the tape.
- **Checkpoint files (from G3).** A checkpoint file, such as a cli `--out` file, is used only when
  `state_hash(cp.state)` passes one of two checks:
  - it equals the genesis hash; or
  - it equals the hash recorded for state tick `cp.state.tick`, taken from a chunk index or a
    manifest that names the same `tape_hash` and `world_id`. That hash is `--hashes` line
    `cp.state.tick` (P0.5 amendment 8), or `TickReport.hash` of tick `cp.state.tick − 1`.
- **Hash files.** A bare `--hashes` file names no tape, so it is not accepted until it names one
  (§7.2, raised). Since S2.4 it names one: its `#` header gives the build, the tape with its
  `tape_hash` and `world_id`, and the starting tick. The cli writes a `manifest.ron` beside its
  checkpoints, and its `resume` verifies against it (CERTIFY §9, §10, C9).
- **When a check fails.** The GUI reruns from genesis and logs why.
- **G0.** It opens no checkpoint file.

### 3.4 The observation store

- **Series.** `SeriesKey { measure, at }` names a series by tape key.
  - The catalogue: every `MarketLine` field and `RationLine` count; settlement quantity and value by
    actor; the audit's margins and lines; holdings; params, with the source of the current value;
    fired events. From G2 it adds derived measures and the oracle gap.
  - The Extractor, on the worker, turns (`&Sim`, `&TickReport`) into columnar rows plus the tick's
    hash.
  - G0–G2 keep f64 columns, with explicit gaps, in memory.
  - As built at G0.1: a value is stamped with the tick that ran, and holdings and params are
    read from the state it left. A series is a column of (tick, value) pairs, so a gap is a tick
    with no pair, never a filler. The catalogue is every `MarketLine` field; each class line's
    requested, feasible and filled; each order's settled quantity and value; every holding;
    every registered param; the tick's audit lines, largest margin and largest drift; and the
    run's drift per good and margin. Fired events travel in the row, with their source (§7.2
    item 2). The store refuses a batch that does not continue the run's ticks or its catalogue.
- **Full recording is a hypothesis, tested at G3.** Its one measurement is data first's probe, run
  on WSL with synthetic MarketLine series only: 7 fields × 23 goods × 42 regions = 6,762 series,
  over 15,600 ticks.
  - Ingest took 0.051–0.053 ms a tick.
  - The 64-tick overview was 5.1% of the raw data.
  - 40 full-span series decimated in 1.6–2.0 ms.
  - One 1,024-tick chunk was 46.0 MB as LZ4_RAW with byte-stream-split (42.1 MB with zstd-3). It
    read back bit-exact in pyarrow 25.0.0 and polars 1.43.1.
  - A whole E42 run of those series alone is therefore about 0.70 GB (0.64 GB with zstd-3). The
    rest of the catalogue was not measured.
  - At 700 regions, ingest took 1.13–1.16 ms a tick.

  A run over §9's budgets brings the catalogue filter forward.
- **Chunks, from G3.**
  - Chunks are 1,024 ticks. Each is named by the clean build, `world_id` and the `prefix_id` at the
    chunk's end, and carries its `tape_hash`. A branch therefore shares every chunk that ends by its
    first changed firing. Dirty builds never share.
  - Memory holds a 64-tick overview (min, max, last) and the recent chunks.
  - The rest goes through observe's byte sink to `crates/gui`. There it is written as tidy long
    Parquet (LZ4_RAW, byte-stream-split; `world_id` and `prefix_id` in the footer) to a spill
    directory off C:.
  - The run's index names the `tape_hash` and `world_id`, and holds every tick's state hash and each
    chunk's content digest. The digest is checked on every read, and the index is D4's record.
  - The format is proposed as certify's telemetry; Phase 3 rules on it.
- **Plots.** egui_plot does not decimate.
  - Headless (median of 60 frames, one line, 1600×1000 panel), a 100k-point line cost 3.4–3.7 ms on
    WSL and 6.7–6.9 ms on Windows (3.4 ms with mimalloc).
  - Decimated to about 3,200 points, the line drew in 0.13–0.22 ms. Decimating every frame would
    add 0.17–0.28 ms.
  - So the plot cache decimates once, to about two points per pixel column, extends as ticks
    arrive, and lends `PlotPoints::Borrowed`.
  - Every drawn vertex is a recorded (tick, value), and each column keeps its true min and max.
  - The decimator lives in run/, free of egui (G0.1). It takes points in tick order, keeps each
    closed column's minimum and maximum, and splits the line at every gap in the record, so a
    gap is never bridged. The plot cache in ui/ holds one per plotted series.
  - As built at G0.1's second part: columns are `2^k` ticks wide, the fewest that fit the
    visible span into the panel's pixel columns, so a growing record rebuilds a decimator a few
    times at most. The cache records the vertices it lends each frame, and
    `every_drawn_vertex_is_recorded` checks each one against the store, bit for bit.
  - As fixed after G0.1's verification: the record is made where a segment is lent, from the
    points its `Line` is made of, and the test also checks each segment against the path
    egui paints for it.
  - Log axes (G1) plot `num::ln(v)`. The x axis is the tick, labelled by `Clock::date_of`. Its
    gridlines fall on year starts, the tick 1 January falls in, labelled by the year; below two
    years they are egui's, labelled by date.

## 4. Panels and lenses

Selection and cursor are global. The egui_tiles layout persists in `layout.ron`: outliner left,
plots centre, inspector right, timeline and log below. `rustyecon-gui tapes/gate.ron` opens paused
at tick 0 with every price plotted.

As built at G0.1's second part (its amendment block has each change): the cursor is a report
tick, and live follows the last tick that ran. The plots stack one panel per unit. "Run until"
takes a state tick, or a date it runs through. The registry's rows are `engine::registry`'s,
with the current value from the record. The actor inspector reads a snapshot the model asks
for. The log logs each class line's rationing onset once per load. "Ledger changed" waits for
G0.2's branches, and so do overlay, difference and ratio against a parent.

As fixed after G0.1's verification: a run opens with every price plotted whenever the
session's plots name nothing of its world, so a session another tape wrote still gives one key
press a live price plot, and another world's keys are left out of the stack. Every log line's
tick is a report tick, and a line about a state says "state tick".

As built at G0.2 (items 9–13 of its block): the editor and compare are panes beside the
inspector and the registry. The editor holds the form, the raw pane, the staged edits, the
orphan offer, Apply, "Save tape as…" and export. Compare shows a branch against its parent:
both identities, the first differing hash and the report tick that left it, the lineage, the
tape diff by section and key, and, for each plotted series, parent, branch and their difference
at the cursor, the largest difference and its tick, and the first tick they differ. "Ledger
changed" compares a run's tolerances with its parent's. The plots' overlay, difference and
ratio against a parent move to G1.

As fixed after G0.2's verification (items 1, 3, 5 and 6 of its block): "ledger changed" says
yes, no, or unchecked and why, and never "no" when the parent is not held. A reopened tape's
parent is the ancestor its lineage names, open in this session or read from its path with its
`tape_hash`. The export carries that ancestor, and `GuiManifest` gains `lineage_stale`, with
`ledger_changed` now an `Option<bool>`. Compare names each run's origin, shows the range it
compared, and reads a resumed parent's earlier states in the records it resumed from. When
those do not reach back far enough, it says the hashes may differ earlier.

| Panel | At G0 | Grows at |
|---|---|---|
| Toolbar | Open. Run and pause (Space), step 1 (`.`), step a year, run until a date or tick, speed cap. Date, tick, ticks/year. Identity chip: commit and dirty flag, `world_id`, `tape_hash`, origin. Health: status, `max_margin`, hash, "ledger changed". | G3 seek |
| Timeline | Scrubber and cursor. Fired and scheduled events. Ring checkpoints. | G3 300-year zoom; G10 branch tree |
| Outliner | Tape entities by key; select, pin, plot. | G1 watchlist; G2 lenses; G4 regions |
| Plots | Any series, stacked, decimated, date-labelled, with a linked cursor. Overlay, difference or ratio against the parent. Units always shown. | G1 log axes; G2 oracle; G5 record bands; G6–7 fans |
| Inspector | Market: p, next p, ema, S, D, cleared, fills, rationing by class, ln(p′/p), and the rule and its rate param with unit and basis. Actor (by snapshot): spec lines with param keys, holdings, lots, settle lines. Param. Event. | G1 price-step explainer; G2 decisions; G3 vintages |
| Registry | Every param: key, unit, genesis, current value, fixed, and its basis. The per-tick value is shown per use (§7.2 item 4); until that lands, it is `engine::registry`'s one method per param. Beside the current value: the basis of the param that the last `SetParam` copied, with that event's key and date. | G3 levels (if D9's bump lands), registry class |
| Editor | §5 | G4 world tables; G10 generators |
| Compare | A branch against its parent: identities, first differing hash, tape diff by key, lineage, difference of pinned series. | G2 battery; G6–7 ensembles |
| Log | Load, run, pause, fired events, rationing onset by class, errors. Breakpoint on error. | G1 event and date breakpoints; G2 conditions |
| Export | CSV of exactly the plotted series at full resolution: shortest round-trip f64, with `#` lines giving the run key, ticks and origin. `manifest.ron`, an envelope `GuiManifest { manifest: certify::Manifest, origin, draft: true, lineage: Option<String> /* file name */ }`, so certify's type does not change. The tapes and the lineage. Every export is stamped "run, draft" or "experiment, draft". Native only; the web build's manifest waits for W1 (§7.2). | G6–7 figure data bundles, citable |

**Why is this price 12.3?** Under `Imbalance` a price has memory, p_t = p_0·exp(Σ k·x), so there are
two answers.
- **The last step.** The inspector shows it at G0.
- **The explainer (G1).** It calls P0.4's public `markets::imbalance` and `next_price(rule,
  one_sided, p, k, S, D)` on the recorded inputs, taking k from `clock.log_step` of the rate at that
  tick. It asserts that the result equals the report's `next_price` bit for bit.
- **The history.** A log waterfall of ln(p_t/p_0) as Σ k·x, with events marked and the residual
  shown.

**Lenses (G2).**
- A lens is `{ measure, unit, scale: Sequential | Diverging { centre: Reference }, domain:
  Fixed(lo, hi) }`. The reference is the baseline, the oracle, the record or a named value (W = 1,
  κ = 1).
- One `LensVm` feeds a ranked table, a chart and a map view, with shared selection. Scales are
  neutral and fixed for the run.
- From G3, once Phase 3 makes the registry data, scripted series are hatched everywhere (R5).
- Before geography, the map view is a schematic (nodes on a circle, channels as chords), labelled
  "schematic, not geography".

**Binding constraints (G2)** are marked where the number is (ADDENDUM §3.1 item 4):
- capacity or an input on the production line;
- cash and rationing on the order;
- a desk inside its dead-band on the decision.

**From Victoria 3.**
- Taken: map modes with a ranked table, pinning, lockable drill-down, flows on hover, balance
  sheets, speeds and hotkeys.
- Left out: lenses that change state, predictive tooltips, good and bad colours, narrative
  notifications, depth only in tooltips, and one timeline with no rewind.

## 5. The editor

### 5.1 Edits, lineage and branches (D3)

```rust
use rustyecon_core::tape::raw::RawAct;   // Mint, Burn, Transfer, SetParam { param, to }, Actor(A)
pub struct TapeEdit { pub op: EditOp, pub note: String }       // note: required, not empty
pub enum EditOp {                                               // no arm carries a basis
    AddEvent { key: Key, at: Date, act: RawAct<RawAgentAction> },  RemoveEvent(Key),
    AddRecurring { key: Key, first: Date, every: Key, last: Option<Date>, act: RawAct<RawAgentAction> },
    RemoveRecurring(Key),  AddParam { key: Key, value: f64, unit: Unit },  RemoveParam(Key),
    SetGenesisParam { key: Key, value: f64 } }                  // a world edit: reruns from genesis
pub fn materialise(parent: &Tape, edits: &[TapeEdit], on: Date) -> Result<Branch, EditError>;
// Branch { tape: Tape, applied: Vec<Applied>, world: World }                   (as built at G0.2)
pub fn plan(parent: &Store, child: &World) -> Plan;             // Resume(RingCheckpoint) | Rerun { reason }
pub struct Applied { edit: TapeEdit, on: Date, replaced: Option<Replaced> }  // old value and basis, or the entry removed
pub struct Lineage { format: u32, tape_hash: Hex /* the tape it describes, G0.2 */,
                     parent: Ancestor { tape_hash: Hex, path: String },      // the nearest ancestor on disk
                     edits: Vec<Applied>, records_viewed: Vec<SeriesKey> }   // every edit since it, in order
```

As built at G0.2 (items 1–8 and 12 of its block): `edit/` names core's raw types, `Basis` and
`Unit`, and no other module does. `materialise` returns the branch's world beside its tape, and
lists every orphan a removal leaves. `plan` reads the parent's store. A param only the schedule
reads is outside `world_id`, so removals of events and of their source params, and a new value
copied from a new param, resume from the ring; only an edit of what the world reads reruns. The
lineage names the tape it describes. Keys are new to every tape of the tree, the staged edits
and the runs closed this session. The stamp's date is today's, in UTC. The act is typed as the
tape writes it, and a parser error lands on the raw pane with its line and column. A branch
lives in memory until it is saved; the session does not re-make it at launch.

As fixed after G0.2's verification (items 1, 4 and 6 of its block): a new key is new to every
tape open in the session, to every run closed in it and to the staged edits, and a minted `n`
only grows within a session. A hand-edited tape whose parent is not held shows "ledger
unchecked" and why. A lineage that describes another `tape_hash` than its tape's is flagged in
the export and logged when saved.

1. **The user acts** in the registry, inspector or editor. For example: "set `mine.capacity` from
   `mine.capacity.base` on 1765-06-01". That is an `AddEvent` whose `act` is `RawAct::SetParam {
   param, to }`. In a live run, "from now" means the next unrun tick's date.
2. **The form checks first.**
   - A key matches `[a-z0-9_.-]+` and is new to the whole run tree, siblings and ancestors included
     (E8). Minted keys have the form `gui.<s>.<n>`, where s is a session serial.
   - A date parses, and the note is not empty.
   - The params in `header.ledger` are not editable. A hand-edited tape whose tolerances differ from
     its parent's shows "ledger changed" in the health chip and the export.
   - No edit is offered on a number that has no basis of its own until a schema bump gives it one
     (R16; §7.3 Phase 4). Such numbers are a good, node, channel or class, a genesis line, and an
     actor's inline number.
3. **`materialise` builds the branch.**
   - It applies the edits.
   - It stamps every entry it adds or changes with `Assumed("GUI experiment <date>: <note>")`.
   - It sets `header.name` to `<name> [GUI experiment <date>]`, replacing any marker the parent's
     name carries. A branch made only of removals is therefore marked too. The name is outside
     `world_id` (ENGINE §14 item 6), so no checkpoint is refused.
   - It round-trips the result through `to_ron` and `from_ron`.
   - `Sim::new` checks it. That is `resolve` plus `Cast::new`, which makes the whole-world checks
     `resolve` alone misses: every buy line's node quotes in the home currency, and no payout names
     its payer (P0.5 amendment 3).
   - A resolution error names its tape path (`actors[mill].spec.buy[village/grain].qty`), which maps
     to a row.
   - A parser error has a line and column but no path (P0.3 amendment 7). One that gets past the
     form is mapped to its entry and shown on the raw pane.
4. **Orphaned params.** A removal that leaves a param unreferenced is a load error (ENGINE §2.6).
   For example, removing `mine.cut` orphans `mine.capacity.cut`. The editor offers a `RemoveParam`
   with its own note. If the user accepts it, the branch reruns from genesis when removing the
   param changes `world_id`. As built at G0.2: `mine.capacity.cut` is a param only the schedule
   reads, outside `world_id`, so that branch resumes from the ring tick before the cut.
5. **`plan` picks the resume point.**
   - It requires the same `world_id`.
   - It scans the parent's ring checkpoints from the newest. The first whose stored `prefix_id`
     equals the child's `prefix_id(cp.tick)` is the resume point. That is about 40 checks on the
     gate world.
   - `Sim::resume` has the final word. On `WrongWorld` or `WrongPrefix` the GUI reruns from genesis
     and logs why.
   - The branch shares the parent's chunks before the fork. Compare asserts that the first
     differing hash is not before the resume tick.
6. **Saving.** The base file is never overwritten. "Save tape as…" writes two files:
   - The canonical tape, which the cli runs unchanged (E1).
   - `<name>.lineage.ron`. This names the nearest ancestor that exists on disk (the base file or a
     saved tape), with its `tape_hash` and path. It lists every edit since that ancestor, through
     each unsaved intermediate branch, in order. Each edit carries its note, its date, and the value
     and basis it replaced or the entry it removed.

   Once saved, the tape becomes its children's nearest ancestor on disk. Every export includes the
   lineage.
7. **A changed base.** A base file that changes on disk becomes a new root run. Existing runs keep
   their tape and `tape_hash`. `session.ron` records each base's hash. On reopen, the GUI refuses to
   re-apply edits to a base whose hash differs. As built at G0.2 no edit is re-applied at launch,
   so this waits for G1 with the re-making of branches.

`tape_hash` is defined once, in §3.3. The raw RON pane is read-only. G4's world tables and G10's
generator forms make branches by the same steps.

### 5.2 The dial problem, and levels (D9)

A dial change to a new value needs a new param to copy from (ENGINE §2.6; open question 6). That
changes the registry and `SimState.params`, so `world_id` changes and every checkpoint is refused.

At `a8f9ed8`, `world_id` also hashes each param's `fixed` flag (`crates/core/src/tape/mod.rs:938`).
A param gains that flag by becoming a `SetParam` source. So a new event that points at an existing
param that is not yet a source changes `world_id` too. Session 2 removes that (D10 item 1).

Until levels exist, a new value reruns from genesis. On the gate world that is expected to take
well under a second. This is an expectation, measurable since `a8f9ed8`; G0 records it (§9). A
1750–2050 county run takes minutes (PLAN §3.9's target).

As built at G0.2 (item 4 of its block): a new param that only a `SetParam` reads is outside
`world_id`, so a new value copied from it resumes from the ring; only a change to what the
world reads reruns. G0.2 measured the gate world's rerun (`gate_rerun_time_is_recorded`, an
ignored test): `materialise` of a genesis value, then `Sim::new` and the run to 2,080 through
`ThreadDriver` with the G0 Extractor took a median of 42.5 ms over 5 runs (WSL, release). At
the close, on the finished crate, the same test gave a median of 41.3 ms (40.1 to 43.5 ms),
and 40.2 ms when run again. There is no threshold.

**Levels.** D9 defers them to a schema bump before Phase 3's long runs, ruled then.
- **The schema.** `SetParam { param: Key, to: SetTo }`, with `enum SetTo { Param(Key), Level(Key)
  }`, and `levels: Vec<RawLevel { key, value, unit, basis }>`.
- **Outside the world's identity.** Levels are out of `world_id` and `SimState`. `prefix_id` covers
  a level through the resolved value, so a level used after tick c leaves every checkpoint at or
  before c valid.
- **Sources.** `Firing.source`, `Recurring.source` and `FiredEvent.source` become
  `Option<Source>`, with `enum Source { Param(ParamId), Level(LevelId) }`. A firing that copies a
  level therefore names its basis.
- **Rules.** Levels share the params' namespace, and an unreferenced level does not load. Levels
  count toward R7, and they appear in the registry listing (§7.2 item 4) and in R5's tables.
- **Tests.** `level_edit_after_checkpoint_resumes`, `level_unit_mismatch_is_rejected`,
  `level_source_names_its_basis`, `unused_level_is_rejected`.

## 6. The map and its data (ruling 3, ADDENDUM's 7; D7, D8)

- **Sources.**
  - HCBP Definition B: Historic Counties Trust, release 2026-09-24, OSGB, simplified. It was
    digitised from the OS First Edition maps (1840s–1890s) and assigns every detached part to its
    parent county, so it approximates the pre-1844 geography. Its README makes the data free "for
    all personal, educational, non-commercial and commercial use", with acknowledgement requested.
  - HCBP Definition B holds Yorkshire whole, as do HCBP A and OS Boundary-Line. The riding lines
    therefore come from OpenStreetMap's `boundary=traditional` relations (Overpass, 2026-09-25,
    ODbL 1.0).
  - The atlas is therefore a derivative database. It ships under ODbL in `data/atlas/`, with its
    own `LICENSE` and an attribution file (OpenStreetMap contributors; the Historic Counties
    Trust). Every build that bundles it (W1) carries both. The code licence is unaffected.
- **Checks and fallback.**
  - CAMPOP's 1831 ancient counties (55 counties, pre-1844, after Kain and Oliver) validate the
    riding lines and detached parts. They are under the UK Data Service End User Licence and are
    never committed.
  - OS Boundary-Line historic counties (c.1888; OGL v3; 88 counties, Yorkshire whole, no detached
    parts) serve if HCBP goes.
  - A prototype atlas built from these sources during the design, and not committed, has 42 regions
    (38 counties, Monmouthshire and the three ridings), 370 parts and 153 holes. It has 7,072
    vertices at 1,000 m and 22,694 at 250 m.
- **Pipeline (Phase 4).**
  - Pinned downloads (SHA-256) go through a pinned offline Python script (shapely 2, pyproj) under
    `data/atlas/`.
  - The script writes the region table (key, codes, name, area, label point, adjacency with border
    lengths) and the geometry. Both are committed and hashed in CI.
  - Areas, adjacency and border lengths stay in the table and the atlas, outside `world_id`, until a
    rule reads them. The loader rejects an unreferenced param, so a number enters the tape with its
    first reader (R10).
  - A tape entry carrying an atlas-derived number has the basis `Measured { source: "data/atlas
    <hash> (ODbL; OpenStreetMap contributors; Historic Counties Trust)", vintage }`. Phase 4 states
    the licence status of `tapes/`.
  - The rest of the tape's layout is Phase 4's.
- **Region keys (D7).**
  - One crosswalk, `data/regions/regions.ron`, gives the tape key, the Chapman code, the HCS code
    and number, and the name.
  - The proposed key is `county.<chapman>` in lower case, such as `county.bdf`. The ridings are
    `county.ery`, `county.nry` and `county.wry`, the Chapman codes the atlas uses; HCS codes have
    none.
  - Keys are permanent (E8), so Phase 4 fixes the scheme.
  - Another geography reaches the key through a dated crosswalk with stated weights. That is a
    labelled construction in the spine (R11), never the GUI.
- **When (D8), and drawing.**
  - The map arrives at G4, with Phase 4 or with Phase 5's first county series, whichever is first.
    At the user's request it came forward in part on 2026-09-27, over the illustrative demo
    tape (the block "The map and lenses, brought forward"). D8 still governs G4 proper and the
    research world.
  - On record data alone, the map shows the origin "record" only. An atlas whose keys differ from
    the tape's node keys is refused.
  - egui fills only convex polygons, so each part is triangulated once with earcut, holes included,
    into one `Mesh`. Shared borders are drawn once, thinned. Coordinates are OSGB metres.
  - Phase 9's world atlas (Natural Earth v5.1.2, public domain) is projected equal-area by the atlas
    build.
- **Measured.**
  - Map tessellation, headless (median of 60 frames, mimalloc): 0.03 ms for 40 regions' fill; 0.82–
    0.93 ms for 700 regions (140,800 vertices); 2.1–2.6 ms with thinned outlines. Hit tests took
    0.14–0.44 µs with either allocator.
  - A real window on Windows (RTX 4090, Vulkan; re-run by the facts review) showed the spike's one
    plot, one table and one controls pane, with a fake run streaming. It sat at the vsync cap of 118
    fps, which shows no headroom.
  - CPU per frame was p50 3.13 ms / p90 5.09 ms at 40 regions with outlines, and p50 4.60 ms / p90
    6.36 ms at 700 regions with thinned outlines.
  - G4's bar is a p90 under 8 ms with the full layout, and it is unproven. The spike left about 3 ms
    at 40 regions, and 1.6 ms at 700, for every other panel. G9 applies the same bar at 700 regions.

## 7. What the engine must provide

### 7.1 Committed at `a8f9ed8`, or contracted for P0.6

- **Committed, read in the source.**
  - **Core:** `Tape::{from_ron, to_ron}` and `resolve`; `LoadError { path, kind }`;
    `World::{world_id, prefix_id, id_of, key_of, ..}` and `Registry`; `Schedule::{fire,
    firings_before}` and `Firing.source` (P0.3 amendment 6); `Clock` and `Checkpoint`;
    `state_hash`, `fnv1a_64` and `Inventory::lots`; `tape::raw::RawAct`.
  - **Markets:** `imbalance`, `step` and `next_price` (P0.4 amendment 7).
  - **Agents:** `RawAgentAction::SetActive { actor, active }` and `Cast::new` (P0.5 amendment 3).
  - **Engine (P0.5):**
    - `Sim`, which is `Clone`, with `new`, `resume`, `step`, `step_traced`, `run_until`,
      `checkpoint`, `status` and `last_report`.
    - The accessors `price`, `ema`, `supply`, `demand` and `param`, which return `Option<f64>`
      (amendment 5), and `holding`, `holdings_of`, `actor_state`, `observe_holdings`, `world`,
      `tick` and `hash`.
    - `audit_replay`, and `engine::registry(&Tape) -> Result<Vec<RegistryLine>, LoadError>`.
    - `TickReport`, `MarketLine`, `FiredEvent { key, occurrence, action }`, `HoldingTotals` and
      `Trace`, all serde.
    - `RunErrorKind::Agent` and `ReplayError::Shadow`.
    - The prelude.
  - **The cli:** `run`, `resume`, `replay` and `registry`. `--hashes FILE` writes `{t}
    0x{hash:016x}`, where t is the state's tick after each step (amendment 8).
  - **`tapes/gate.ron`:** its final hash was `0x1b86507a195b2a40` at `a8f9ed8` and has been
    `0x61f9c8529131ff17` since P0.6, equal on WSL and Windows.
  - **Tests the GUI leans on:** `engine_runs_on_a_worker_thread`, `observing_changes_no_hash` and
    `gate_resume_from_checkpoints`.
- **Contracted for P0.6** (ENGINE §12): the move of PLAN.md, `STATE.md`, `ci/gate.sh` and
  `ci/gate.ps1`, and the engine's wasm32 check, recorded.
- **Not needed:** an engine clock, thread or setter.

### 7.2 G0's prerequisites, in Phase 0's second session (D10)

D10 lists four items, each with its ENGINE amendment and test. Each was checked against `a8f9ed8`.

| # | Item | At `a8f9ed8` | ENGINE | Test |
|---|---|---|---|---|
| 1 | `world_id` hashes (key, unit, genesis) per param, not the `fixed` flag that the schedule sets. The structural uses that also set the flag are covered already: a shelf life (as ticks, in the goods) and the ledger tolerances (in `tol`) are in `world_id`, and a recurring period is in the schedule, which `prefix_id` covers. The item changes every `world_id` once and no state hash, so the gate's recorded hashes stand but older checkpoints are refused (`WrongWorld`). It lands before session 2's manifest records a `world_id`. It reopens core (P0.3). | Unmet: `tape/mod.rs:938` hashes `p.fixed` | §2.6 | `new_source_event_keeps_world_id` |
| 2 | `FiredEvent { key, occurrence, action, source: Option<ParamId> }`, taken from `Firing.source` | Unmet: `report.rs:63–70` has no source | §7.4 | `fired_event_names_its_source` (`mine.cut` names `mine.capacity.cut`) |
| 3 | Stepping in chunks changes nothing | Met. `run_until` is a loop over `step` (`sim.rs:151–162`). `observing_changes_no_hash` steps one tick at a time and compares against `hashes()`, which calls `run_until`. `gate_resume_from_checkpoints` calls `run_until` in chunks. Session 2 may add a step-of-7 loop to the first in one line, and records it. | §11 (no change) | the two existing tests |
| 4 | `engine::registry`'s `Entry::Param` gains `sites: Vec<ParamSite { path, method: ClockMethod, per_tick: f64 }>`, recorded by the resolver. The sites replace `per_tick`'s guess from the unit and from whether the param is a good's `price_rate` (`registry.rs:81–95`). A `RatePerYear` has two per-tick forms (`share`, `log_step`), and only the use knows which applies. The item reopens core and agents. | Partly met: `engine::registry` and `RegistryLine` exist and `rustyecon registry` prints them, with one method per param | §2.6 (`Resolver::param` takes the method; core's own sites at `tape/mod.rs:314`, `441–442`, `597–633`, `748`), §6 (a new `ClockMethod` enum), §4 (the five sites at `agents/src/spec.rs:263–375`), §7.1 (`RegistryLine`) | `registry_names_each_use`: on a gate variant where the mill's `spend: Some("mill.spend")` becomes `Some("rate.bread")` and `mill.spend` is deleted, the row for `rate.bread` lists both `log_step` (its `price_rate` use) and `share` (its `spend` use) |

**Where the items land.** ENGINE §13's session-2 row gains items 1, 2 and 4 (A14). If an earlier
step meets one of them, session 2 records it.

**Met at S2.2** (`7c14c37`; ENGINE, amended at S2.2), each with its named test:
- Item 1: `world_id` hashes (key, unit, genesis) per param. Every `world_id` changed once and no
  state hash moved: the gate's is `0x43628a8e0fd5f695`, appb's `0x26f12f8a0bc27540`.
- Item 2: the field is `source: Option<Key>`, not `Option<ParamId>`, since a schedule param has
  no id.
- Item 3: met already; `observing_changes_no_hash` gains a loop that steps seven ticks at a time.
- Item 4: `Entry::Param` lists `sites: Vec<SiteLine { path, method: ClockMethod, per_tick }>`.
  Core records each use as a `ParamSite { path, method }`, and a resolved spec holds a
  `Site { param, method }`, read only through it, so the method listed is the conversion the run
  applies. `rustyecon registry` prints each use.

**Raised with session 2, not asked** (D10 lists exactly four):
- **(a) The tape hash.** The tape hash that certify's manifest records. This design proposes
  `fnv1a_64` over the canonical `to_ron` text (§3.3).
- **(b) The cli and R16.** Under ENGINE §7.6 and §8, the cli's `resume` would verify a checkpoint's
  state hash against a hash file that names its tape. Its `--hashes` file and stdout would name
  their run (build, `world_id`, tape hash). The cli would then meet R16 as the GUI does. R16 holds
  the cli to this once ENGINE rules on it.
- **(c) A Parquet-free manifest.** The manifest types would sit in a module or feature of certify
  that is free of Parquet. The web build's export needs this at W1.

The manifest itself is session 2's own work (ENGINE §13, N10). The GUI wraps it (§4) and changes
nothing in it.

**The raised items, as session 2 built them** (docs/CERTIFY.md):
- (a) `tape_hash` is `fnv1a_64` over the canonical `to_ron` (C2, §3; §3.3 above).
- (b) Since S2.4 a `--hashes` file opens with `#` lines naming the build, the tape by
  `tape_hash`, the `world_id` and the starting tick, and stdout prints a `run …` line with the
  same fields before its final hash, `replay` too. `resume` verifies its checkpoint against the
  manifest of the run that made it: the same `tape_hash` and `world_id`, and a record at its tick
  with the same digest and state hash (C9). A resume under a dated edit waits for a ruling
  (CERTIFY §15.1, question 2).
- (c) Certify's telemetry sits behind its feature `parquet`, off by default. With it off,
  certify has 17 crates in its closure on every target and checks for wasm32; the manifest
  module needs no I/O. `scripts/gate.sh` builds, tests and lints certify that way.

### 7.3 Later items, each ruled at its own phase

| When | Item | Why |
|---|---|---|
| Before Phase 3's long runs (D9) | Levels and `Option<Source>`, in a schema bump (§5.2). | Dial changes resume |
| Phase 1 (the oracle's run) | `Point::outputs() -> Vec<(&'static str, Output)>` beside `Eq1a::outputs()`; params through `dump::parse`. | The lab keeps no copy of the fields |
| Phase 2 | `Sim::step_observed(&mut self, focus: &[ActorId]) -> Result<(TickReport, Vec<DecisionRecord>), RunError>`. `DecisionRecord { actor, hook, view: ViewCopy, orders, deltas, rule: Key, params: Vec<ParamId>, dead_band: bool }`. `params` are the params the actor's resolved spec references, fixed at load; recording reads would need interior mutability, which E2 bans. Test `observed_step_equals_step`. | The decision inspector shows exactly the `View` (R13) |
| Phase 2 | `TickReport.production: Vec<ProduceLine { actor, x, binding }>`, with `enum Binding { Capacity, Input(GoodId), None }`. | Binding marks |
| Phase 2 | Create `crates/observe`: a default member, engine-gated, with no egui, thread, clock, file or `std::io`. It holds `measure::derived(r, sim)` and `oracle_gap(sim, setup) -> Result<Gap, GapError>`, which is never fed back. ENGINE §11's scans cover observe: the list in `crates/engine/tests/common/scan.rs` gains it, though observe is not on the run's path and nothing there depends on it (R13). | One definition for the GUI, the cli and Parquet |
| Phase 2 | The stability sweep (dead × α, each point a tape edit) as an observe or certify function with a cli entry. The GUI shows its results. | The phase diagram is reproducible without the GUI |
| Phase 3 | The scripted/emergent registry as data (a tape section with a schema bump, or a spine file), with its ENGINE amendment and test. Observe's `registry_class(w, s: &SeriesKey, tick)`. | Hatching (R5); no engine function takes a GUI type |
| Phase 3 | Certify's telemetry in §3.4's chunked tidy long Parquet (session 2 moves the writer as is), with an index that names `tape_hash` and `world_id` and holds every tick's hash. | One format for the GUI, the cli and Python; D4's record |
| Phase 4 | `worldgen::compile(&Tables) -> Result<Compiled, CompileError>`, with `Compiled { tape, atlas }` and `Atlas { schema, crs, nodes (key, parts with holes, label), borders, sites, sources }`. The region keys (D7). Test `atlas_keys_equal_node_keys`. | §6 |
| Phase 4 | Per-line bases for genesis lines and world-table rows (goods, nodes, channels, classes, an actor's inline numbers), in a schema bump. Test `world_edit_keeps_other_bases`. | R16: a world edit stamps only its own number |
| Phase 5 | A record `Series { key, unit, source, vintage, points, band }`, readable without arrow. | Overlays |
| Phases 6–7 | Certify scores only tapes whose `tape_hash` the fitting harness registered before the run. It refuses any tape whose name or any basis carries the GUI-experiment marker (decided now, D3), or the illustrative marker (amended at D.4; certify's seal already reads the name's, `certify_refuses_illustrative_tape`). Test `scorecard_refuses_gui_edited_tape`, whose cases include a saved branch made only of removals and the demo world's tape. `Deserialize` on `Certificate`, `Criteria`, `Scorecard` and `FitId`. `rustyecon figure build --check`. | U5; the scorecard viewer |
| Phases 8, 10 | Credit postings in `TickReport`; `generate(spec, base) -> Result<Tape, GenError>`. | Balance sheets; generator forms |

Dropped: `markets::price_step`. P0.4's public `next_price` and `imbalance` serve the explainer.

## 8. Testing and CI

### 8.1 Tests

| Area | Stage | Tests | Checks |
|---|---|---|---|
| Hashes (U4) | G0 (`InlineDriver` from G2) | `gui_equals_cli`. It runs fixed command scripts: pauses, speed caps, steps of 1 and 7, run-untils, a snapshot at 300, and breakpoints. At G0 the only breakpoint is on error, and it never fires on the gate world. Event and date breakpoints join at G1. Through `ThreadDriver`, the gate world's per-tick hashes must equal `Sim::new` plus `run_until(2080)`. The test writes `{t} 0x{hash:016x}`, with t = `report.tick + 1` for t = 1..2080: the cli's convention (P0.5 amendment 8). `ci/gui.sh` diffs that output against `rustyecon run tapes/gate.ron --until 2080 --hashes <tmp>/gate.hashes`, both binaries built `--release` on the same machine. As built at G0.1 it runs the same script on `tapes/appb.ron` to 20,000 ticks too, feeds every observation into a store, and checks the snapshots (300 ahead, 250 behind, from the ring) and every ring checkpoint against the engine's run; the files go where `RUSTYECON_GUI_HASHES` names. | R8 |
| Failed runs | G0 | `failed_run_shows_its_ledger_line`. A shortfall variant of the gate tape, made by text substitution as the cli's `shortfall_stops_the_run` makes one, pauses on the error breakpoint. It shows `Poisoned`, its ledger line and the last good tick. As built at G0.1 it runs through the model, a `drive::Host` and `ThreadDriver`, and reads the toolbar's and the log's view-models. After G0.1's verification its tape adds a gift of 1,000 coin that fires before the burn, so the failed tick leaves a state no tick left; the test selects the workers and checks the snapshot's hash against the cli's state 73 and each holding's lots against the record, and checks the ledger line by key. | E5, R2 |
| Branches | G0 | `branch_resume_equals_rerun`. It adds a `SetParam` of `mine.capacity` to `mine.capacity.base`, dated 1765-06-01, which restores capacity early. `mine.capacity.base` is already a source through `mine.restored`, so `world_id` is kept. The branch resumes from the largest ring tick at or before `tick_of(1765-06-01)`, which is `tick_of(1765-01-01)`. The first differing report hash exists and is at `tick_of(1765-06-01)`. The test saves the materialised tape and its hashes, and `ci/gui.sh` diffs them against the cli binary's run of that tape. After §7.2 item 1, a `SetParam` that copies a param not yet a source also resumes; a new value reruns from genesis until levels exist. As built at G0.2 it runs through the model, a `drive::Host` and `ThreadDriver`: the edit is typed into the form, and the branch's Runner receives the parent's ring checkpoint at state tick 780 (1765-01-01 falls in it). Its hashes from there equal a rerun of its tape from genesis, the first differing report hash is at tick 801, and compare says the same. The files are `branch.ron` and `branch.hashes`, the parent's record before the resume and the branch's after. After G0.2's verification it also branches twice under the resumed branch: a world edit, which reruns, and an event dated 1755-05-05, before the branch's start. Compare reads the branch's parent's record before 780 and finds the first difference at genesis and at that date's tick, as the engine does. With the branch's own record alone it says the hashes may differ earlier. `ci/gui.sh` also checks the cli's `tape_hash` of each branch tape against the GUI run key's. | E1, N11 |
| Branches | G0 | `removal_only_branch_is_an_experiment`. `RemoveEvent(mine.cut)` plus `RemoveParam(mine.capacity.cut)` rerun from genesis. The branch's name carries the marker, and its export says "experiment". At G6–7, `scorecard_refuses_gui_edited_tape` refuses the saved tape. As built at G0.2 the removals keep the world (item 4 of its block), so the branch resumes from state tick 519, where 1760-01-01 falls, and equals its rerun; `ci/gui.sh` diffs `removal.ron`'s run by the cli too. The name carries the only marker in the tape, the identity chip, the CSV's `#` lines and the manifest envelope say "experiment", the export carries the lineage, and the base's export says "run". A genesis value the world reads reruns from genesis, and the log gives the reason. | D3, U3 |
| Editing (D3, D4) | G0, except as marked | `gui_edits_are_always_assumed`. `saved_tape_carries_its_lineage`, for a child and for a grandchild of an unsaved child: the lineage names the base file and lists both branches' edits in order. `removing_the_last_use_offers_remove_param`. `ledger_tolerances_are_not_editable`. `minted_keys_never_collide`, for siblings and for a grandchild after a removal. The reducer as a state machine. From G1 and G5, `no_intent_copies_a_record_or_oracle_value`. At G3, `edited_checkpoint_file_is_refused`: a RON checkpoint with one holding changed and its ids kept. As built at G0.2 (`tests/edit.rs`, on a host of real Runners on the test's thread): `gui_edits_are_always_assumed` checks every basis of the branch, stamped exactly where an entry was added or changed, with that edit's note, at two dates; `saved_tape_carries_its_lineage` also saves a child of the saved child, whose ancestor is the saved tape, refuses to write over the base or any file, reopens the saved tape as an experiment, and exports its lineage and the ancestor's tape; `ledger_tolerances_are_not_editable` also reopens a saved tape whose tolerance was edited by hand and sees "ledger changed" in the chip and the export; `minted_keys_never_collide` also refuses a typed key the tree has, keeps a closed run's keys and mints under another session's serial. `the_form_checks_first` and the reducer's `apply_branches_from_the_parents_ring_and_files_are_effects` join them. After G0.2's verification: `gui_edits_are_always_assumed` also removes the pension and its offered period through the model, and the lineage keeps both entries. `saved_tape_carries_its_lineage` exports the reopened run itself, the in-memory one and a fresh session's, each with `ancestor.ron`, and checks U3's clauses: a lineage alone, a marked basis alone, and neither. `ledger_tolerances_are_not_editable` opens the hand-edited tape in fresh sessions: with the base read from disk it says "changed"; with the base gone or changed on disk it says "unknown", never "no"; and the export and a save flag its stale lineage. `minted_keys_never_collide` also covers a reopened saved child, two roots of one file, a closed parent, param and recurring keys, Apply's re-check, a dropped staged key and a set-aside session's serial. | R4, R5, E8 |
| Goldens | G0 | Each `vm::*` builder on the gate world, saved as RON, at four points: tick 0, where bread rations; after the 1760 cut, where the registry shows `mine.capacity` with `mine.capacity.cut`'s basis, `mine.cut` and its date; 1768; and tick 2,080. `UPDATE_GOLDEN=1` rewrites them, in the commit that retunes the gate world. The tests live in `crates/gui/tests` and run under `ci/gui.sh` only, including after the builders move to observe (G2). As built at G0.1's second part: `gate_view_models_equal_their_goldens` and `appb_view_models_equal_their_goldens`, one file per builder and point in `tests/golden/<tape>-<point>/`; the gate at state ticks 1, the cut's plus 1, the oven's plus 1 (1768) and 2,080, and appb at 1 and 20,000. Each point also asserts its claim: bread rations at tick 0; after the cut the registry names `mine.cut`, 1760-03-01 and `Assumed("gate world: half")`; `oven.opens` has fired by 1768; the final hashes are the cli's. They equal byte for byte on WSL and Windows. | U6 |
| Headless egui | G0 | egui_kittest 0.36.2, probed in WSL with no GPU: `Harness::new_eframe` ran a full `App` with a worker thread. G0's scripts: open, run, pause, step. Select (town, bread) and see its inspector. An empty note, a malformed key and a malformed date are refused, and a parser line and column lands on the raw pane. Apply makes a branch. Export from a branch writes a CSV whose `#` lines say "experiment", a `manifest.ron` envelope whose `origin` says "experiment", and the lineage file. Image snapshots need a wgpu adapter, not a GPU. G0 tries mesa's lavapipe on WSL and gates snapshots if it runs; otherwise it records them. As built at G0.1's second part: `one_key_press_gives_a_live_price_plot`, `the_gate_script_runs_pauses_steps_and_inspects` (open, set and lift the speed cap, run, pause, step, step a year, run until a date, select (town, bread) and see its price and unit, select the mill and see its snapshot, the timeline's cursor, the registry's link to `mine.capacity` and the basis its value was copied with, the log's breakpoint) and `the_appb_script_runs_pauses_steps_and_inspects`, each through the panels' keys, clicks and typed text, checking the text each frame paints. The gate script caps the speed while it runs and pauses, since an uncapped gate run can pass tick 480 before a pause lands; 60 repeats in WSL and 10 on Windows ran clean. The editor's steps wait for G0.2. WSL has no lavapipe (only NVIDIA's ICD), so no image snapshot is taken. After G0.1's verification the gate script also checks the painted identity chip against the build, `world_id` and `tape_hash`, each panel's unit, year labels and cursor, and plots from the inspector and from the outliner; `the_theft_script_shows_a_failed_run` (the failed variant from a file: `Poisoned`, the ledger line, the line by key, the last good tick, no more ticks) and `a_second_tapes_session_still_plots_every_price` (the gate opened with a session appb wrote) join them. As built at G0.2, the editor's steps are two scripts. `the_editor_refuses_an_empty_note_a_malformed_key_and_a_malformed_date` also refuses a key the tree has and a ledger tolerance, and paints the parser's line, column and caret on the raw pane. `the_branch_script_applies_compares_exports_and_saves` runs the gate to its end, applies an edit typed into the form, sees the branch resume from the ring and its chip say "experiment", runs it, sees compare's first differing hash, lineage and tape diff, exports (CSV and manifest say "experiment", the lineage is there) and saves the tape with its lineage. Each line they check in those panes is scrolled into view first: Windows' longer temporary paths wrap compare's lines and pushed its tape diff below the fold. After G0.2's verification the branch script also sees each compare identity's origin and the range compared. It then opens the saved tape, its tolerance edited by hand, in two new windows: the chip paints "ledger changed" while the base is on disk, and "ledger unchecked" and why once it is gone. | U3 |
| Plots, ingest, scans | G0 (digests G3) | `decimation_keeps_extremes`, `every_drawn_vertex_is_recorded`, `nonfinite_ingest_stops_with_the_series_named`, and `chunk_digest_is_checked_on_read` (G3). `model_run_edit_vm_import_no_egui` and `no_trig_outside_ui` (D12). Copies in `crates/gui/tests` of the engine's `no_raw_transcendentals` and `no_hashed_collections` (`crates/engine/tests/scans.rs`, whose list of crates is written inside it), run over the egui-free modules. From Phase 2 the engine's own scans cover observe too, `engine_path_does_no_io` included (§7.3). As built at G0.1 the egui-free modules are model, run, edit, vm, drive and platform; `run_and_vm_reach_no_model_file_thread_or_clock` keeps run/ and vm/ ready to move into observe (D13); and each scan is checked on a fixture first. At G0.1's second part `every_drawn_vertex_is_recorded` runs the gate to 2,080 in the app with every price, a stepping param and a series with gaps plotted, at 1,600 and 640 pixels wide, and checks every vertex lent to egui against the store bit for bit, one segment per unbroken stretch, and each line's extremes; `the_gui_names_core_for_num_alone` holds the GUI's use of core to `core::num`. After G0.1's verification the scans read `use` trees, so a group, a rename, a glob import and the bare path a glob allows are refused too; `nonfinite_ingest_stops_with_the_series_named` poisons NaN, +inf and −inf; and `every_drawn_vertex_is_recorded` reads the vertices where they are lent and checks each segment against the path egui paints for it. As built at G0.2, `edit_reaches_no_model_file_thread_or_clock` keeps `edit/` pure, run/ and vm/ may not reach `edit/`, and `the_gui_names_core_for_num_alone` lets `edit/` alone name core's raw schema, `Basis` and `Unit`. | U9, U10, E2 |
| Map | G4 | Every node key has a region. Triangulated area equals polygon area within a relative 1e-9. Each label point hits its own region. Each shared border is drawn once. | §6 |
| Web, frames | G0 frames; G2 web | From G0, a smoke mode records CPU per frame (p50, p90, max) with its panel set, since fps is vsync-capped. As built at G0.1's second part it is `rustyecon-gui --smoke UNTIL TAPE`, at ten model years a second (its amendment item 12). From G2, the wasm32 `cargo check` passes with certify left out of the web build. A browser run (wasm-bindgen 0.2.129, trunk) records its gate hash in STATE.md beside Linux and Windows (D11). | A5 |

### 8.2 CI (D1)

- **Default members.** `default-members` lists every crate but `crates/gui`. Engine steps run
  clippy and test with `--workspace --exclude rustyecon-gui`, and `cargo fmt --check` covers
  everything. ENGINE's step definition (preamble), §1, and §12's `ci/gate.sh` are amended in the
  commit that adds `crates/gui`. Done at G0.1 (ENGINE, amended at G0.1).
- **The lockfile.** Cargo resolves one lockfile for the whole workspace, `--exclude` or not. Offline
  engine steps therefore need the GUI's dependency closure cached on both machines (ENGINE §1; P0.5
  amendment 11).
  - The commit that adds `crates/gui` runs one `cargo fetch` on WSL and one on Windows. Windows'
    `CARGO_HOME` goes on D: while C: is short. At G0.1 the lockfile was resolved offline from the
    cache, which left every existing entry as it was and gave every crate of the GUI's closure the
    version the spike's lockfile has; the lockfile grew from 100 packages to 473. Both fetches
    ran. C: had 114 GB free, so Windows' `CARGO_HOME` stayed where it was.
  - Lockfile changes land only at G-stage boundaries (D6).
- **The non-blocking check.** Each engine step also runs one `cargo check -p rustyecon-gui` on WSL,
  in its own target directory, and records the result in STATE.md. It never fails the step. An
  engine-caused GUI break is fixed at the next G-stage at the latest. `scripts/gate.sh` runs it in
  `$CARGO_TARGET_DIR-gui`, on Linux only (G0.1).
- **Measured costs** (facts review; WSL, 48 threads, the spike crate, clean target):

  | Build | Wall time | Target directory on disk | Peak RSS |
  |---|---|---|---|
  | check | 14.9 s | 0.76 GB | 0.88 GB |
  | release | 28.0 s | 1.1 GB | 1.02 GB |
  | kittest debug | – | 2.8 GB | 1.71 GB |

  G0 records the warm check: `cargo check -p rustyecon-gui` after touching
  `crates/engine/src/lib.rs`, in its own target directory, median of 3. It also re-runs the clean
  check with certify in the tree.

  As recorded at G0 (WSL, 48 threads, certify in the tree), on the seams at G0.1 and on the
  finished crate at the close:

  | Build | At G0.1 | At the close |
  |---|---|---|
  | check, clean | 16.1 s, 0.81 GB, 0.87 GB peak RSS | 17.3 s, 0.85 GB, 0.87 GB peak RSS |
  | check, warm (median of 3) | 2.55 s | 2.92 s |
- **`ci/gui.sh`** gates G-stages only, on WSL. It runs `cargo clippy -p rustyecon-gui --all-targets
  -- -D warnings` (the crate sets `[lints] workspace = true`), `cargo test -p rustyecon-gui`, and
  the cli hash diffs of §8.1. As built at G0.1 it is `scripts/gui.sh`: `cargo fmt -p
  rustyecon-gui --check`, the clippy above, `cargo test -p rustyecon-gui --release` with warnings
  denied (release, as the engine's gate tests, so appb's 20,000 ticks take a second), a check that
  each test G0's gate names ran and passed, and the diffs of gate's and appb's hashes. At G0.1's
  second part it names 15 tests: the first part's eight, `every_drawn_vertex_is_recorded`, the
  two goldens, the three kittest scripts and `the_gui_names_core_for_num_alone`. After G0.1's
  verification it names 17: `the_theft_script_shows_a_failed_run` and
  `a_second_tapes_session_still_plots_every_price` join them. At G0.2 it names 27: the two
  branch tests, the five editing tests, the editor's two kittest scripts and
  `edit_reaches_no_model_file_thread_or_clock`. It also runs the cli binary on the two branch
  tapes the branch tests write, `branch.ron` and `removal.ron`, to 2,080, and diffs each hash
  file's body against the GUI's. After G0.2's verification it also checks the `tape_hash` in
  the cli's hash file against the GUI run key's, which the branch tests write beside each tape.
  At G0's close it names 42: the reducer's fifteen state-machine tests join them, since §9
  names them as a group.
- **Windows, extending A5.**
  - `ci/gate.ps1` skips the GUI while C: is short: 3.3 GB free at the last check (df, the evening
    of 2026-09-25). `%TEMP%` is on C: too. At G0.1 there is no `ci/gate.ps1`: Windows runs
    `scripts/gate.sh` under Git Bash (STATE decision 20), which skips the GUI's check off Linux,
    and `scripts/gui.sh` the same way. C: had 114 GB free on 2026-09-27.
  - Window gates are manual and recorded. The manual Windows build sets `CARGO_TARGET_DIR` on D:
    (at least 5 GB free) before its first build.
  - WSL's disk is on D: already.

## 9. Roadmap

Each stage starts after its phase's gate, with three exceptions. G4 starts on D8's trigger: Phase 4's
gate or Phase 5's first validated county series, whichever comes first. G10 runs beside Phase 10,
which is open-ended and has no gate. W1 starts when the paper needs it, after G4 at the earliest. G0's phase is Phase 0, so G0 follows its second session. No
stage gates engine work (D1).

Sessions are guesses. G0–G7 add about 10–15 sessions. PLAN §6's ranges for Phases 0–7 sum to 19–30
(Phase 0 at two sessions, A4). PLAN's stated total, roughly 25–35 sessions to the first certified
England run, becomes roughly 35–50.

- **G0 — the shell, after Phase 0's second session, with Phase 1 in parallel (D2). Two to three
  sessions.**
  - *G0.1, the viewer.* The crate and its seams: `reduce`, the Runner, `ThreadDriver`, the
    Extractor, the in-memory store and the ring. The toolbar, timeline, outliner, plots,
    inspector, registry, and a log with a breakpoint on error. `session.ron` and `layout.ron`.
    Left out: the watchlist, event and date breakpoints and log axes (G1); `InlineDriver` and the
    wasm check (G2). G0.1 lands in two parts. The first (2026-09-27) is the crate and its seams,
    `session.ron` and `layout.ron`, and D1 in the workspace, with `gui_equals_cli`,
    `failed_run_shows_its_ledger_line`, `decimation_keeps_extremes`,
    `nonfinite_ingest_stops_with_the_series_named` and the four scans. The second (2026-09-27)
    is the panels, the view-model goldens, G0's kittest scripts,
    `every_drawn_vertex_is_recorded` and the smoke mode; G0.1 is then built.
  - *G0.2, the editor.* `TapeEdit`, `materialise`, the lineage, `plan` and branches. Compare. CSV,
    manifest, tape and lineage export. Built on 2026-09-27 (its amendment block has each
    change), with compare's tape diff. The plots' overlay, difference and ratio against a
    parent, and re-making branches at launch, move to G1.
  - *If G0.2 overruns,* compare's tape diff moves to G1.
  - *Closed at G0.3* (2026-09-27). Every gate item below is met and recorded in STATE.md but
    the window checked by hand, which is the user's. `scripts/gui.sh` names 42 tests: the
    fifteen of the reducer's state machine joined at the close (the block "Closed at G0.3"
    above).
  - *If a §7.2 item slips,* G0 ships without what needs it. Without item 4, the registry panel
    shows `engine::registry`'s one method per param. Without item 2, the copied basis is joined
    from `World::schedule`. Without item 1, more dial edits rerun from genesis.
  - **Gate.**
    - Under `ci/gui.sh`, these tests pass:
      - `gui_equals_cli` (`ThreadDriver`);
      - `failed_run_shows_its_ledger_line`;
      - `branch_resume_equals_rerun` and `removal_only_branch_is_an_experiment`;
      - `gui_edits_are_always_assumed`, `saved_tape_carries_its_lineage`,
        `removing_the_last_use_offers_remove_param`, `ledger_tolerances_are_not_editable`,
        `minted_keys_never_collide`, and the reducer's state-machine tests;
      - the vm goldens and G0's kittest scripts;
      - `decimation_keeps_extremes`, `every_drawn_vertex_is_recorded` and
        `nonfinite_ingest_stops_with_the_series_named`;
      - `model_run_edit_vm_import_no_egui` and `no_trig_outside_ui`;
      - the copied `no_raw_transcendentals` and `no_hashed_collections`.
    - `cargo clippy -p rustyecon-gui --all-targets -- -D warnings` is clean under the workspace
      lints.
    - On Windows, checked by hand and recorded in STATE.md, the window opens `tapes/gate.ron` and
      runs to 2080 with every price plotted and no panic. Smoke mode records CPU per frame (p50,
      p90, max) with its panel set. At the close the smoke mode is recorded and the check by
      hand is PENDING, the user's.
    - `ci/gate.sh` carries D1's changes, with ENGINE's amendment.
    - STATE.md records the warm check, the clean check with certify in the tree, and the recount.
      It also records the gate world's rerun time, with no threshold: WSL, release, median of 5
      runs of `materialise`, `Sim::new` and `run_until(2080)` through `ThreadDriver` with the G0
      Extractor.
    - From `rustyecon-gui tapes/gate.ron`, one key press gives a live price plot.
- **G1 — the oracle lab, after G0 and Phase 1's gate. One to two sessions.**
  - *Built at G1.1–G1.10* (2026-09-27, branch `g1`; the block "Amended at G1"): every item
    below, with the engine's re-export of `num` and the raw schema (the GUI's edge to core
    goes), O26's map items and two of O20's. Its gate: the first three items met, by
    `the_lab_shows_appendix_b_bit_for_bit` and the Lab script, by
    `the_explainer_equals_next_price_on_every_gate_tick` (12,480 pairs) and by
    `a_200_point_sweep_builds_in_under_16_ms` (0.75 ms on WSL, run by name); the window's p90
    by hand is PENDING, the user's. Left for G1 from G0: overlay, difference and ratio against
    a parent, re-making branches, the ways into the editor, the session's lock, and the rest of
    O20.
  - Solve an instance and show the regime and outputs beside their goldens.
  - Plot any `Point` field over x in [0, 1], so f(x) = n_D − n_S shows its root and bracket.
  - One-param sweeps.
  - The price-step explainer and the log waterfall.
  - Log axes, the watchlist, and event and date breakpoints.
  - PNG snapshots, never citable.
  - **Gate.**
    - The lab shows x\* 0.86315, v 0.54344, Y 7.88061 and N_a 1.34338, bit for bit the oracle's
      outputs.
    - The explainer equals `next_price` on every gate tick.
    - A 200-point sweep's view-model builds in under 16 ms on WSL (median of 20). This is an
      `#[ignore]` test, run with `cargo test --release -- --ignored`.
    - The window's p90 frame stays under 16 ms, checked by hand.
- **G2 — agents meet the oracle, after Phase 2's gate. Two to three sessions.**
  - The Runner, Extractor, Store and vm/ move into observe (D13).
  - Lenses as a table, a chart and a schematic. The oracle line and gap.
  - The decision inspector and binding marks. Condition breakpoints.
  - A battery panel (July's solvable worlds, modes A and B). The stability sweep as a heatmap.
  - Named layouts.
  - `InlineDriver`, the wasm32 check and the first browser proof.
  - **Gate.**
    - The battery panel's numbers equal the battery test's, because one function makes both.
    - `observed_step_equals_step` passes, if Phase 2 adopts `step_observed`.
    - A 20×20 dead × α sweep of 520-tick runs of one Phase 2 battery world (for example
      `solv_1g`), made by Phase 2's sweep function, equals its heatmap view-model golden.
    - The browser's gate hash is recorded (D11).
- **G3 — long runs, after Phase 3's gate. One to two sessions.**
  - The chunked store and the spill; bounded channels; checkpoint thinning; seek.
  - cli runs opened by their chunks, with D4 applied to their checkpoints.
  - Vintage, investment, population and enclosure panels.
  - Levels, if D9's bump has landed. Hatching.
  - **Budgets (set now).** Under 2 GB resident. A 1750–2050 single-region run spills at most
    0.5 GB.
  - **Gate.**
    - A headless Runner with the full Extractor runs the 1750–2050 tracer within 10% of `rustyecon
      run --until … --hashes FILE` with certify's telemetry, with equal hashes. That is WSL,
      median of 5 runs, process start excluded.
    - A `SetParam` to an existing source dated 1900 resumes from a ring checkpoint at or before
      1900. So does a new value once levels exist; otherwise it is a logged rerun.
    - `edited_checkpoint_file_is_refused` passes.
    - The full catalogue's bytes per run are recorded and inside the budgets.
- **G4 — the map, on D8's trigger. Two sessions.**
  - The atlas and the mesh. Lenses on the map, channels on hover, ports and coalfields, a
    difference lens.
  - The world editor, limited to numbers that have their own basis (§7.3 Phase 4).
  - *Brought forward in part* (2026-09-27, branch `demo-world`, D.1–D.5). The atlas, the mesh
    and 25 lenses on the map and in a ranked table came forward over the illustrative demo tape.
    The block "The map and lenses, brought forward" has what landed and each gate item. G4
    proper keeps the research world, record overlays, flows on channels, ports and coalfields,
    the difference lens, the world editor, the p90 bar and the spill.
  - **Gate.**
    - §8.1's map tests pass.
    - The map's values equal the table's, because one view-model makes both.
    - Frame p90 stays under 8 ms on Windows while a run streams (unproven, §6).
    - An E42 run spills at most 4 GB.
    - The ODbL licence and attribution files are in `data/atlas/`.
- **G5 — the record, after Phase 5's gate. One session.**
  - Overlays with source, vintage and bands (R11). Provenance chips.
  - "Records viewed" in the lineage.
  - Breakpoint B's eyeball sheet, as a saved session.
  - **Gate.** Every overlay names its source and vintage, and nothing is interpolated silently. An
    edit made while a scored series was overlaid lists that series in the lineage.
- **G6–7 — fitting and scoring, after Phase 6's gate; its Phase 7 parts (ensembles) after Phase
  7's. One to two sessions.**
  - The scorecard viewer: the verdict first, with fitted and scored moments apart (R5), and U12's
    label.
  - Ensemble fans with the fit id. Figure data bundles.
  - The citable mark, which fails closed. It needs a clean build, a registered tape, origin "run"
    and a certified verdict.
  - **Gate.**
    - The scorecard's view-model equals the committed scorecard.
    - `scorecard_refuses_gui_edited_tape` passes, the illustrative demo tape among its cases
      (D.4).
    - The figure data bundles (CSV, manifest, tapes, lineage) rebuild byte for byte under
      `rustyecon figure build --check` in CI. Rendered figures are laborformal's own check (PLAN
      §8).
- **G8 — money, after Phase 8's gate. One session.** Balance sheets, credit lenses and crisis
  overlays. **Gate:** with the layer off, the balance-sheet and lens view-models equal their
  Phase 7 goldens.
- **G9 — the world, after Phase 9's gate. One to two sessions.** The world atlas; trade on
  channels. **Gate:** the grain-gap chart equals its scoring function, and frame p90 stays under
  8 ms on Windows at 700 regions.
- **G10 — forward branches and the policy lab, beside Phase 10. One to two sessions.** A
  branch tree from the certified 2025 state; generators as forms; the pace so far beside every
  central run. **Gate:** every branch's hashes equal the cli's on the same platform and build.
- **W1 — the reader build, when the paper needs it, after G4 at the earliest. One to two
  sessions.**
  - trunk, `InlineDriver`, bundled tapes, precomputed chunks, and short branches live.
  - A Web Worker only past about 5 ms a tick.
  - The manifest in export, from a Parquet-free module of certify (§7.2, raised).
  - **Gate.** WebGPU with the WebGL2 fallback, IME and `AsyncFileDialog` work in a real browser.
    Hashes are recorded against native (D11). The atlas's licence files ship.

## 10. Risks and the parking lot

**Risks:**
1. Until levels exist, a new dial value on a long world reruns from genesis.
2. egui breaks its API every 6 to 24 weeks (D6).
3. C: is nearly full: 3.3 GB free, and one build failed with os error 112. The Windows GUI build,
   Windows' `CARGO_HOME` and the spill therefore go to D:. At G0 C: had 114 GB free; the
   targets went to D: all the same, and `CARGO_HOME` stayed on C:.
4. Full recording may not fit (§3.4).
5. The full layout's frame time is unproven (§6). The spike left about 3 ms of headroom at 40
   regions and 1.6 ms at 700, against G4's 8 ms p90 bar and G9's same bar at 700 regions.
6. Only the Windows window was measured. rfd under WSLg is untested, and no browser has run the
   build.
7. Certify's tree is unknown until session 2. If it brings in parquet with zstd, it must stay out
   of the web build (§3.1).
8. The GUI can outgrow the engine and compete with the phases for sessions. U11, D1 and PLAN §6's
   totals guard against this.

**Parked,** each with its condition for re-entry:
- egui_dock: if a need appears.
- walkers: never, for the history.
- The wgpu paint callback: past millions of vertices.
- wasm threads: once they need neither a nightly build nor cross-origin isolation.
- A catalogue filter: when a run exceeds its disk budget.
- A structured `LoadError` path: if the string proves fragile.
- `ControlFlow` from `run_until`: if a frontend needs it.
- Gated image snapshots: when lavapipe runs on WSL.

An SVG writer is rejected (PLAN §3.8).
