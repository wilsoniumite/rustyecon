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
    run. It refuses any tape whose name or any basis carries the GUI-experiment marker (D3, §7.3).
  - From G5 the lineage lists the scored series that were overlaid during an edit.
  - Until Phase 6 registers the moment table, these guards are procedural.
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
                 manifest; native only if it brings in parquet), oracle (G1) and observe (G2).
                 Nothing depends on it, and default-members leave it out (D1).
  src/main.rs      native entry: mimalloc, a tape path, run_native; build.rs stamps commit and dirty flag
  src/web.rs       wasm entry (G2 proof, W1)
  src/app.rs       eframe::App: drains the drivers into reduce(); ui() draws; supplies the wake callback
  src/model/       no egui: Model, Intent, Effect, reduce(), Session (serde)
  src/run/         no egui: Cmd, Obs, Runner, Extractor, in-memory Store, checkpoint ring
  src/edit/        no egui: TapeEdit, materialise, plan, branch, lineage
  src/vm/          no egui: one pure view-model builder per panel; Serialize + Debug
  src/drive/       ThreadDriver (G0), InlineDriver (G2): the threads, clocks and channels
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
pub struct Model { session: Session, runs: BTreeMap<RunId, Run>, focus: RunId,
                   cursor: Cursor /* Live | At(tick) */, selection: Option<Sel>, log: RunLog }
pub fn reduce(m: &mut Model, i: Intent) -> Vec<Effect>;   // the only writer; Effect = Cmd | file op
pub struct RunKey { build: Build /* commit, dirty, target, rustc */, tape_hash: Hex, world_id: Hex }
// certify's RunKey, reused as is (docs/CERTIFY.md §3; S2.6); Hex(u64) serialises as "0x%016x"
pub enum Cmd { Load { tape: Tape, from: Option<ResumeFrom> }, Run { until: Option<u64>, max_tps: Option<u32> },
               Pause, Step(u64), Breakpoints(Vec<Breakpoint>), Snapshot(u64), Stop }
pub enum ResumeFrom { Ring(RingCheckpoint), File(VerifiedCheckpoint) }
// RingCheckpoint { run: RunKey, tick: u64, bytes: Vec<u8> /* Checkpoint::to_bytes */ }: opaque, minted only by a Runner
// VerifiedCheckpoint: made only by verify() (G3)
pub enum Obs { Loaded { tick: u64, hash: u64 }, Batch(ObsBatch), Paused { tick: u64, why: PauseReason },
               Checkpointed(RingCheckpoint), Snapshot(Snapshot), Failed { error: RunError, last: Option<TickReport> },
               Refused(ResumeError) }
impl Runner { pub fn new(sink: Box<dyn FnMut(Obs) + Send>, wake: Box<dyn Fn() + Send + Sync>) -> Runner;
              pub fn handle(&mut self, c: Cmd);  pub fn advance(&mut self, max_ticks: u32) -> Progress; }
pub trait Driver { fn send(&mut self, c: Cmd); fn poll(&mut self, out: &mut Vec<Obs>); }
```

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
- A failed run shows `Poisoned`, its ledger line and its last good tick.

**Web (G2).**
- `InlineDriver` calls `advance` inside `poll`, under an 8 ms `web_time::Instant` budget.
  `thread::spawn` and `Instant::now` trap on wasm32; this was checked in Node 22.
- Past about 5 ms a tick natively, the Runner moves into a Web Worker (gloo-worker 0.6.0, trunk
  0.21.14).

**Checkpoints (D4).**
- **The ring.** The Runner keeps `Checkpoint::to_bytes` at each ring tick. A ring tick is
  `tick_of(YYYY-01-01)` for every model year, thinned to decades beyond G3's memory budget.
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
  - Log axes (G1) plot `num::ln(v)`. The x axis is the tick, labelled by `Clock::date_of`.

## 4. Panels and lenses

Selection and cursor are global. The egui_tiles layout persists in `layout.ron`: outliner left,
plots centre, inspector right, timeline and log below. `rustyecon-gui tapes/gate.ron` opens paused
at tick 0 with every price plotted.

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
pub fn materialise(parent: &Tape, edits: &[TapeEdit], on: Date) -> Result<(Tape, Vec<Applied>), EditError>;
pub fn plan(parent: &Run, child: &World) -> Plan;               // Resume(RingCheckpoint) | Rerun { reason }
pub struct Applied { edit: TapeEdit, on: Date, replaced: Option<Replaced> }  // old value and basis, or the entry removed
pub struct Lineage { parent: (u64 /* tape_hash */, String /* path */),       // the nearest ancestor on disk
                     edits: Vec<Applied>, records_viewed: Vec<SeriesKey> }   // every edit since it, in order
```

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
   with its own note. If the user accepts it, the branch reruns from genesis, because removing a
   param changes `world_id`.
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
   re-apply edits to a base whose hash differs.

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
| Phases 6–7 | Certify scores only tapes whose `tape_hash` the fitting harness registered before the run. It refuses any tape whose name or any basis carries the GUI-experiment marker (decided now, D3). Test `scorecard_refuses_gui_edited_tape`, whose cases include a saved branch made only of removals. `Deserialize` on `Certificate`, `Criteria`, `Scorecard` and `FitId`. `rustyecon figure build --check`. | U5; the scorecard viewer |
| Phases 8, 10 | Credit postings in `TickReport`; `generate(spec, base) -> Result<Tape, GenError>`. | Balance sheets; generator forms |

Dropped: `markets::price_step`. P0.4's public `next_price` and `imbalance` serve the explainer.

## 8. Testing and CI

### 8.1 Tests

| Area | Stage | Tests | Checks |
|---|---|---|---|
| Hashes (U4) | G0 (`InlineDriver` from G2) | `gui_equals_cli`. It runs fixed command scripts: pauses, speed caps, steps of 1 and 7, run-untils, a snapshot at 300, and breakpoints. At G0 the only breakpoint is on error, and it never fires on the gate world. Event and date breakpoints join at G1. Through `ThreadDriver`, the gate world's per-tick hashes must equal `Sim::new` plus `run_until(2080)`. The test writes `{t} 0x{hash:016x}`, with t = `report.tick + 1` for t = 1..2080: the cli's convention (P0.5 amendment 8). `ci/gui.sh` diffs that output against `rustyecon run tapes/gate.ron --until 2080 --hashes <tmp>/gate.hashes`, both binaries built `--release` on the same machine. | R8 |
| Failed runs | G0 | `failed_run_shows_its_ledger_line`. A shortfall variant of the gate tape, made by text substitution as the cli's `shortfall_stops_the_run` makes one, pauses on the error breakpoint. It shows `Poisoned`, its ledger line and the last good tick. | E5, R2 |
| Branches | G0 | `branch_resume_equals_rerun`. It adds a `SetParam` of `mine.capacity` to `mine.capacity.base`, dated 1765-06-01, which restores capacity early. `mine.capacity.base` is already a source through `mine.restored`, so `world_id` is kept. The branch resumes from the largest ring tick at or before `tick_of(1765-06-01)`, which is `tick_of(1765-01-01)`. The first differing report hash exists and is at `tick_of(1765-06-01)`. The test saves the materialised tape and its hashes, and `ci/gui.sh` diffs them against the cli binary's run of that tape. After §7.2 item 1, a `SetParam` that copies a param not yet a source also resumes; a new value reruns from genesis until levels exist. | E1, N11 |
| Branches | G0 | `removal_only_branch_is_an_experiment`. `RemoveEvent(mine.cut)` plus `RemoveParam(mine.capacity.cut)` rerun from genesis. The branch's name carries the marker, and its export says "experiment". At G6–7, `scorecard_refuses_gui_edited_tape` refuses the saved tape. | D3, U3 |
| Editing (D3, D4) | G0, except as marked | `gui_edits_are_always_assumed`. `saved_tape_carries_its_lineage`, for a child and for a grandchild of an unsaved child: the lineage names the base file and lists both branches' edits in order. `removing_the_last_use_offers_remove_param`. `ledger_tolerances_are_not_editable`. `minted_keys_never_collide`, for siblings and for a grandchild after a removal. The reducer as a state machine. From G1 and G5, `no_intent_copies_a_record_or_oracle_value`. At G3, `edited_checkpoint_file_is_refused`: a RON checkpoint with one holding changed and its ids kept. | R4, R5, E8 |
| Goldens | G0 | Each `vm::*` builder on the gate world, saved as RON, at four points: tick 0, where bread rations; after the 1760 cut, where the registry shows `mine.capacity` with `mine.capacity.cut`'s basis, `mine.cut` and its date; 1768; and tick 2,080. `UPDATE_GOLDEN=1` rewrites them, in the commit that retunes the gate world. The tests live in `crates/gui/tests` and run under `ci/gui.sh` only, including after the builders move to observe (G2). | U6 |
| Headless egui | G0 | egui_kittest 0.36.2, probed in WSL with no GPU: `Harness::new_eframe` ran a full `App` with a worker thread. G0's scripts: open, run, pause, step. Select (town, bread) and see its inspector. An empty note, a malformed key and a malformed date are refused, and a parser line and column lands on the raw pane. Apply makes a branch. Export from a branch writes a CSV whose `#` lines say "experiment", a `manifest.ron` envelope whose `origin` says "experiment", and the lineage file. Image snapshots need a wgpu adapter, not a GPU. G0 tries mesa's lavapipe on WSL and gates snapshots if it runs; otherwise it records them. | U3 |
| Plots, ingest, scans | G0 (digests G3) | `decimation_keeps_extremes`, `every_drawn_vertex_is_recorded`, `nonfinite_ingest_stops_with_the_series_named`, and `chunk_digest_is_checked_on_read` (G3). `model_run_edit_vm_import_no_egui` and `no_trig_outside_ui` (D12). Copies in `crates/gui/tests` of the engine's `no_raw_transcendentals` and `no_hashed_collections` (`crates/engine/tests/scans.rs`, whose list of crates is written inside it), run over the egui-free modules. From Phase 2 the engine's own scans cover observe too, `engine_path_does_no_io` included (§7.3). | U9, U10, E2 |
| Map | G4 | Every node key has a region. Triangulated area equals polygon area within a relative 1e-9. Each label point hits its own region. Each shared border is drawn once. | §6 |
| Web, frames | G0 frames; G2 web | From G0, a smoke mode records CPU per frame (p50, p90, max) with its panel set, since fps is vsync-capped. From G2, the wasm32 `cargo check` passes with certify left out of the web build. A browser run (wasm-bindgen 0.2.129, trunk) records its gate hash in STATE.md beside Linux and Windows (D11). | A5 |

### 8.2 CI (D1)

- **Default members.** `default-members` lists every crate but `crates/gui`. Engine steps run
  clippy and test with `--workspace --exclude rustyecon-gui`, and `cargo fmt --check` covers
  everything. ENGINE's step definition (preamble), §1, and §12's `ci/gate.sh` are amended in the
  commit that adds `crates/gui`.
- **The lockfile.** Cargo resolves one lockfile for the whole workspace, `--exclude` or not. Offline
  engine steps therefore need the GUI's dependency closure cached on both machines (ENGINE §1; P0.5
  amendment 11).
  - The commit that adds `crates/gui` runs one `cargo fetch` on WSL and one on Windows. Windows'
    `CARGO_HOME` goes on D: while C: is short.
  - Lockfile changes land only at G-stage boundaries (D6).
- **The non-blocking check.** Each engine step also runs one `cargo check -p rustyecon-gui` on WSL,
  in its own target directory, and records the result in STATE.md. It never fails the step. An
  engine-caused GUI break is fixed at the next G-stage at the latest.
- **Measured costs** (facts review; WSL, 48 threads, the spike crate, clean target):

  | Build | Wall time | Target directory on disk | Peak RSS |
  |---|---|---|---|
  | check | 14.9 s | 0.76 GB | 0.88 GB |
  | release | 28.0 s | 1.1 GB | 1.02 GB |
  | kittest debug | – | 2.8 GB | 1.71 GB |

  G0 records the warm check: `cargo check -p rustyecon-gui` after touching
  `crates/engine/src/lib.rs`, in its own target directory, median of 3. It also re-runs the clean
  check with certify in the tree.
- **`ci/gui.sh`** gates G-stages only, on WSL. It runs `cargo clippy -p rustyecon-gui --all-targets
  -- -D warnings` (the crate sets `[lints] workspace = true`), `cargo test -p rustyecon-gui`, and
  the cli hash diffs of §8.1.
- **Windows, extending A5.**
  - `ci/gate.ps1` skips the GUI while C: is short: 3.3 GB free at the last check (df, the evening
    of 2026-09-25). `%TEMP%` is on C: too.
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
    wasm check (G2).
  - *G0.2, the editor.* `TapeEdit`, `materialise`, the lineage, `plan` and branches. Compare. CSV,
    manifest, tape and lineage export.
  - *If G0.2 overruns,* compare's tape diff moves to G1.
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
      p90, max) with its panel set.
    - `ci/gate.sh` carries D1's changes, with ENGINE's amendment.
    - STATE.md records the warm check, the clean check with certify in the tree, and the recount.
      It also records the gate world's rerun time, with no threshold: WSL, release, median of 5
      runs of `materialise`, `Sim::new` and `run_until(2080)` through `ThreadDriver` with the G0
      Extractor.
    - From `rustyecon-gui tapes/gate.ron`, one key press gives a live price plot.
- **G1 — the oracle lab, after G0 and Phase 1's gate. One to two sessions.**
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
    - `scorecard_refuses_gui_edited_tape` passes.
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
   Windows' `CARGO_HOME` and the spill therefore go to D:.
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
