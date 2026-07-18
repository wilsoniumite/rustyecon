# Plan: from the current repo to ARCHITECTURE.md

Dated 2026-07-18. Phases are ordered by dependency; every phase leaves one running
engine under a green harness (METHODOLOGY R7) and each has an explicit gate.
"Session" = one working session; estimates are honest guesses, not commitments.

Current baseline, for the record: ~3.3k lines of Rust implementing the market core
plus a three-strategy agent layer; Python factorial stability suite (failing when
work stopped); docs ~90% aspiration. Known defects listed in Phase 0.

---

## Phase 0 — Bugfix floor (1 session)

Fix the known silent-failure defects before anything is built on top:

1. `state/apply.rs:88` — `return` inside the `RedistributePopPair` arm drops every
   remaining delta in the batch; make it a per-delta skip.
2. `scenario/raw.rs:359` — loader hardwires `CapacityControl` regardless of the
   recipe's declared strategy; honor the declaration or hard-error.
3. `transactions/mod.rs:180` — `HashSet` iteration makes the delta stream
   nondeterministic; use ordered iteration.
4. `systems/mod.rs:51` — spoilage mutates state directly; route through deltas.
5. `types/inventory.rs` — lot lives are not serialized; checkpoints silently make
   goods indefinite; serialize lots.
6. `decisions/pop_agent.rs:103,150` — per-pop `print!` in the hot loop; delete.
7. Python suite: `is np.nan` guards never fire → failing metrics silently pass;
   fix to fail-closed.

**Gate:** existing tests pass; a run's delta stream is byte-identical across two
invocations.

## Phase 1 — Certification stack (2–3 sessions)

Spec: architecture/engine.md. Golden-hash suite (repeat / resume / replay equality as CI
tests); conservation ledger with provenance (`Inventory::remove` returns actual
quantity; shortfall = ledger line); run certificate printed verdict-first and
persisted to `results/`; `criteria.ron` per scenario porting the 07-suite
taxonomy (registered thresholds, NaN = FAIL).

**Gate:** first committed certificates for the lr corpus — expected to FAIL on
stability; failures are evidence and the baseline for Phase 4's A/B.

## Phase 2 — Telemetry (1–2 sessions)

Parquet writer in the engine (tidy long format + manifest); rewrite
`tools/readers.py` input plumbing (keep the `ScenarioResults` API); retire
per-tick RON dumps (checkpoints become sparse, resume-only).

**Gate:** the suite runs off Parquet end-to-end, ≥10× faster load, no RON parsing
in the analysis path.

## Phase 3 — 225-year tracer bullet (1 session)

A trivial 2-region world run for 11,700 ticks under `--certify`. Purpose: expose
long-horizon unknowns now — float drift, EMA behaviour across regime-scale price
moves, telemetry volume, checkpoint size — not to be economics.

**Gate:** certificate PASS at full span; telemetry for the full run under ~100 MB.

## Phase 4 — The desk kernel (2–3 sessions)

Spec: architecture/kernel.md. Implement `Desk` + the three rules beside the existing
strategies; port scenarios; then the A/B with a receipt (METHODOLOGY R10): run the
24-cell lr suite under legacy agents and under the kernel, identical scenarios,
persisted certificates. Kernel ships if its pass-rate ≥ legacy's. Then delete:
all three `StrategyState` arms, the balance ledger and its forgiveness constant,
the sell caps and debt gates, the pop wealth PD-controller, `sub_state`,
`DividendPayout` and its per-scenario wiring, `MagicProducer` (a frozen desk),
and the six now-unemitted delta variants. Constants land in `game_data.ron`.

**Gate:** kernel wins or ties the A/B; agent layer is one file; zero behavioral
constants in code.

## Phase 5 — Phase map (1–2 sessions)

Sweep `eta_up/eta_dn × b_cash × S` over a reference world; commit the phase
diagram (where the collapse phase lives) to `results/`. This is the validation
standard (METHODOLOGY R6) and the guard against discovering the collapse phase
in 1893.

**Gate:** a documented stable operating region with margin, referenced by default
parameters.

## Phase 6 — Perimeter ledger (2–3 sessions)

Spec: architecture/markets.md + objects.md. `OwnershipRegister` (claims replace the single owner
pointer; rent = Rule-3 overflow pro-rata to holders); settlement tagging
(crossing, flow class, labor content); self-provision margin on the labour desk;
dual measured/true telemetry columns.

**Gate:** a toy conversion scenario (home production entering the market) shows
measured GDP rising while true GDP is flat — the moving frame demonstrated end to
end.

## Phase 7 — Land, minting, growth (3–4 sessions)

Spec: architecture/ownership.md + pops.md. Parcels with ω; the minting queue (build k / enclose ω,
claims minted pro-rata, best yield first); overflow routing from pop consumption
desks; the labour ladder (4 buckets) and capability assignment; μ, r\*, B, R,
concentration, dispersal as telemetry columns.

**Gate:** a growth scenario where capacity at least doubles endogenously from
overflow with conservation green; R and r\* series live and sane; a no-minting
control run stays flat.

## Phase 8 — Worldgen and the era ladder (4–6 sessions, then open-ended)

Spec: architecture/worldgen.md. The compiler (tables + timelines → validated tape), the
validator (IDs, dangling refs, conservation pre-check), a ~20-macro-region 1800
world, era criteria for 1800–1850, first certified era leg and snapshot.
Data sourcing (Maddison, Clio-Infra, era research in `docs/timeline/`) is real
work budgeted here, not an afterthought.

**Gate:** `snapshots/1850` exists with a committed era certificate.

## Phase 9+ — Deferred modules (unscheduled, in likely order)

Credit and banks as postings through the money seam (architecture/money.md) (leverage, runs, the discount
window — the v1 financial/monetary docs are the design record); monetary regimes
and FX; estates/frictions as tape operators; the policy layer as experiments
(VAT × dividend sweeps — the founding motivation, now two-economies-native);
migration; more regions toward 673.

Each deferred module enters through its own short design note + criteria before
code (METHODOLOGY R14).

---

## Standing constraints during the migration

- One running sim at all times (R7); the lr corpus is the regression suite until
  Phase 8 replaces it with compiled worlds.
- Docs are rewritten per their triage headers as the phase that touches them
  lands — not before (the headers are the contract; rewriting ahead of code
  re-creates aspiration-docs).
- Rough total to the end of Phase 8: ~4–6 months part-time. Phases 0–3 are pure
  hygiene value even if everything after is abandoned; that is deliberate.
- Kill condition, stated per METHODOLOGY: if the kernel cannot pass the Phase 4/5
  gates after honest effort, stop and rethink the agent design *before* building
  the theory freight on top; the perimeter/land layers assume a stable substrate.
