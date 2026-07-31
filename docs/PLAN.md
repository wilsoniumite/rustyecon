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

> **GATE HALF MET, 2026-07-20.** Telemetry: cleared decisively (2.2 MB against
> ~100 MB, and per-region figures for extrapolating to 673 regions). Certificate
> PASS: **not honestly met.** `data/scenarios/tracer_2r` does print PASS at
> 11,700 ticks, but the PASS is an artifact of window placement over a world
> whose real economy is frozen for 87% of the scored span — full evidence in
> architecture/engine.md, "The 225-year horizon".
>
> The phase delivered its *purpose* rather than its gate, which is the trade it
> was written to allow ("expose long-horizon unknowns now"). What it exposed:
> unbounded currency-lot fragmentation, an accidental price floor with no
> ceiling, resume-equality holding at full span, `price_ema` having been
> write-only state — and the substantive one, that posted supply is computed
> from demand (`building_agent.rs:54`), so a market clearing 100% of demand
> reports 16.7% excess supply and deflates geometrically forever.
>
> Two items below now block Phase 4 rather than Phase 3, and are listed there.

## Phase 3.5 — Close the two gaps Phase 3 found (1 session, blocking Phase 4)

Neither is engine work; both invalidate Phase 4's A/B if skipped.

1. **Split `Rule::Damping` and re-register criteria.** As implemented it is a
   trend detector on a level series: it passes monotone deflation (0.149) and
   fails a stationary economy with any ripple (1.009). Phase 4 ships the kernel
   only if its pass-rate ≥ legacy's, so as it stands **fixing the economics
   loses the A/B**. Needs a trend test on the log-level slope plus an
   oscillation test on the detrended residual, a new dated `criteria.ron`, and a
   re-score of the Phase 1 baseline so both arms are judged by one bar.
2. **Specify `parity`, and price formation generally, in kernel.md.**
   `markets.md` delegates price stability to `kernel.md`; `kernel.md` never
   addresses how a posted price reaches equilibrium — its whole "why this is
   stable" argument is about *scale*. `parity` appears zero times in `src/` and
   `data/` and has no stated units, yet the labour margin is the only kernel
   mechanism that compares a price to a level.

**Gate:** a criteria file that passes a synthetic stationary series and fails a
synthetic deflation ramp; the Phase 1 corpus re-scored under it; kernel.md
carrying a price-formation section with `parity` registered in `game_data.ron`.

> **MET 2026-07-31**, with one part of the gate deliberately not done — see
> below.
>
> 1. **Criteria split and re-registered as generation 4** across all 28
>    scenarios. `DRIFTING(LevelRange 2.2)` and `SWINGING(ResidualDamping 1.5)`
>    replace `Damping(0.72)`, which is retained-but-superseded so older
>    certificates stay reproducible. Generation 3's `LogDrift` was itself wrong —
>    a trend fit blind to any excursion that returns — and was replaced after
>    adversarial review; see engine.md. Thresholds were fixed from synthetic ground
>    truth in `tests/test_09_detectors.rs` *before* the corpus was scored, and
>    that test is the standing record. Corpus re-scored: **pass-rate 4/79 →
>    0/79** — not one region keeps a scored metric inside a 2.2× band, and
>    `DRIFTING` fires 84 times against `SWINGING`'s 19.
> 2. **`kernel.md` has a "Price formation" section**, stating as a checkable
>    invariant that every posted quantity is a function of the posting desk's own
>    state, why inventory satisfies it for storables, where labour and services
>    need their own state variable instead, and what anchors the price *level*.
>    `parity` is resolved to a **real** quantity — subsistence basket per hour,
>    with the wage deflated — in `pops.md` and `kernel.md`.
>
> **Not done, deliberately: the `game_data.ron` field.** Registering `parity`
> means adding schema no code reads, for a consumer (the labour desk) that Phase
> 4 has not built. R2 exists so constants can be swept and defended, and a field
> with no consumer can be swept by nothing — while a schema commitment made ahead
> of its code is exactly the mistake Phase 3 made twice. The units and location
> are now specified; the field lands with the desk that reads it, and the Phase 4
> gate's "zero behavioral constants in code" already covers it.
>
> Two other R2 debts are recorded in engine.md rather than paid here, for the
> same reason: `FLAT`/`FLAT_REL` in `certify::verdict` and `EMA_ALPHA` in
> `price_update` are live constants with live consumers, so unlike `parity` they
> can and should be registered — but they are scoring and engine dials, not
> kernel ones, and moving them is its own change.
>
> **One consequence for Phase 4's gate.** With legacy now at 0/79, "kernel ships
> if its pass-rate ≥ legacy's" is satisfied by a kernel that also scores zero.
> The A/B needs a finer statistic than a region pass count — the distribution of
> `LevelRange` across regions is the natural candidate, being continuous and
> being exactly what the corpus fails on. Decide that before running the A/B, not
> after seeing it.

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

> **IN PROGRESS, 2026-07-31 — first session of the 2–3.** Four sub-phases
> landed; the pop side and the A/B itself remain.
>
> **Done.**
>
> - **P4.0a** — certificates carry a `measurements` table: every class × metric
>   statistic, computed whether or not the rule fired and *past* the `DEAD`
>   short-circuit. 33 of 72 regions are DEAD, so without this the two arms'
>   tables would be missing readings exactly where the arms differ. Verified
>   behaviour-neutral across all 28 scenarios.
> - **P4.0b** — the A/B gate replaced, **pre-registered before the kernel arm
>   existed** in `results/ab/preregistration.md`. Three necessary gates: liveness
>   may not regress; the paired band may not widen on regions both arms keep
>   alive; the paired band may not widen across all 72. `tools/test_ab.py`
>   shows it can report LOSS — including that "collapse every economy" loses on
>   G1 *while fooling G3*, which is what makes G1 load-bearing rather than
>   decorative. One registered constant in the whole rule.
> - **P4.1** — ten kernel dials registered in `game_data.ron` across all 28
>   scenarios, with **no `Default` impl in code**: a scenario missing the block
>   fails to load. Storability is *derived* (`b_out · S` ticks) rather than a
>   further threshold.
> - **P4.2** — Rules 1–3 for producer desks, behind `--agents kernel`. Every
>   engine battery passes under the new arm; the economics do not.
>
> **The finding, which is the session's main output.** kernel.md's producer
> pressure signal cannot expand a desk, and Rule 1 in the same document is why:
> it posts everything above the band, so inventory rests at `band + production`
> and σ is pinned at `−1/b_out` = **−0.5**, below the dead-band, for a desk
> selling every unit it offers. Proof and measurement in kernel.md, "Producer σ,
> corrected"; the corrected signal takes contraction from unsold offers and
> expansion from margin, with the fill steering *scale* and never the posted
> quantity, so the price-formation invariant still holds.
>
> **Where the kernel stands after the correction: still losing badly**, for two
> reasons that are not the σ defect and are both known work.
>
> 1. **Cold start.** The tapes set `chosen_size: 0.0`, which the legacy PD
>    controller overwrites on tick 1 but which the kernel reads as its state
>    variable — so desks open at `ε·size` = 1% of nameplate and need ~230
>    activations (~900 ticks) of multiplicative growth to return. The scored
>    window is 150..1000. Whether the port should set a genesis scale is a real
>    question, and it must be decided the R6 way: state the rule first, apply it
>    uniformly, re-run **both** arms, and report the A/B both with and without.
> 2. **The labour side is still legacy** (P4.3). Pops post their whole supply
>    inelastically, so with desks small the wage collapses, incomes vanish, and
>    demand with them — every desk then reads fill ≈ 0 and contracts. The
>    participation margin π that kernel.md calls "the load-bearing mechanism" is
>    exactly what is missing.
>
> **Remaining: P4.6 the A/B proper, P4.7 the deletions — and the decision
> below, which now blocks both.**
>
> ---
>
> **P4.4 and P4.5 landed 2026-07-31. The A/B is still a LOSS, and the gap
> narrowed on one axis while widening on another.**
>
> **P4.4 — Rule 3's overflow routing replaced the dividend desk.** Cash above the
> band leaves a firm because the rule says so, not because a separate desk
> decided to pay out; the claim-holder pointer is read from the existing
> `DividendPayout` wiring until Phase 6's `OwnershipRegister` replaces it. The
> effect is visible and large: `DEAD_BUILDING` went **27 → 0**, and live regions
> 0 → 8. Buildings now run.
>
> **P4.5 — both of kernel.md's "check rather than assume" items are batteries**,
> B6 and B7, falsified in `tests/test_10_invariants.rs` before being trusted.
> Details in kernel.md; the two results that matter:
>
> - **B6 fires on all 28 legacy scenarios** and names the mechanism —
>   `node0/grain posted supply moved 1.200000 → 1.386000 on foreign volumes
>   alone`, the `1.2 × demand` cap caught in the act — and **passes under the
>   kernel**. The invariant Phase 3 paid a phase to discover violated is now
>   enforced by the certificate.
> - **B7 reproduces the Phase 3 pin unaided**: `node0/grain pinned at -0.166667
>   for 10159 of 11551 ticks unbroken`. Its first version, whole-window
>   constancy, could not — the pin holds only while the cap binds — and that
>   near-miss is recorded rather than quietly fixed.
>
> **A/B after P4.4 + P4.5, all three gates still failing:**
>
> | | legacy | kernel |
> |---|---|---|
> | live regions | 33 | **8** |
> | median band, all regions | 1.53e5 × | 3.27e15 × |
> | DEAD_BUILDING | 27 | **0** |
> | POP_DESTITUTION | 9 | **60** |
> | UNSTABLE | 39 | **72** |
>
> Buildings stopped being idle and pops became destitute instead: the kernel now
> produces, and the goods do not reach anyone. Every region is UNSTABLE.
>
> **The decision that now blocks P4.6, stated so it is not taken by drift.**
> METHODOLOGY's kill condition — stop and rethink the agent design if the kernel
> cannot pass the Phase 4/5 gates after honest effort — is now in scope, because
> the effort *is* honest-complete: every mechanism kernel.md specifies is
> implemented, two spec defects have been found and repaired, and the result is
> 33 live regions to 8. Three options, and the next session should pick one
> rather than continue:
>
> 1. **Take the timescale ratio to Phase 5 first.** Prices move at the registered
>    α (10%/tick on the corpus's goods) against scale's ~1%/tick. A kernel whose
>    quantities cannot answer a price move before the next one arrives is being
>    tested outside its stable region, and α is not currently a phase-diagram
>    axis. On this evidence it should be, and the map should come before more
>    kernel edits.
> 2. **Rethink the agent design**, per the kill condition.
> 3. **Neither yet** — accept the kernel as an implemented, certified, losing arm
>    and keep the legacy layer, which is what R7 already guarantees.
>
> P4.7's deletions are **not** available under any of these: the A/B has not been
> won, and deleting the incumbent because the challenger is finished would be
> exactly the move R10 exists to forbid.
>
> ---
>
> ### Zooming out: the α sweep, and what the criterion is actually testing
>
> Before choosing among those three, the obvious question was asked — **is the
> kernel the problem at all, or is something both arms share?** An α sweep
> answers it, with outcomes fixed before the run: if both arms improve, the price
> rule binds; if only the kernel, a timescale mismatch; if neither, the
> hypothesis is wrong. Diagnostic on a modified tape; the registered A/B stands.
>
> **1. Half the DRIFTING criterion is a statement about the price rule, not
> about agents.** `RealWage` is a ratio of two prices, so its log-drift is
> mechanically `α · Σ imbalance` over the window. At the registered α = 0.1 and
> a window of 850 ticks, **a mean imbalance of 0.0093 exhausts the entire 2.2×
> band** — the criterion demands markets clear to within 1% and stay there for
> 850 ticks. Zero-mean imbalance *noise* alone exhausts it at a standard
> deviation of 0.082. Legacy's best scenario measures a median |imbalance| of
> 0.0097; most sit 3–40× above. Scaling α by 1/100 brings RealWage to **2.59×
> (legacy) and 2.24× (kernel)** — both arms, both landing on the threshold. No
> agent design changes this; α × window does.
>
> **2. The other half is not price-driven at all.** `Velocity` is nominal
> turnover over regional currency, and its band does *not* fall with α. At
> α/100 on `lr_00`: price level band **1.21** — effectively frozen — while
> cleared units band **4.4e5** and 23.5% of ticks fall below the DEAD threshold.
> Velocity is measuring real trade volume collapsing and restarting.
>
> **3. There is no α that fixes both, and this is the finding.** Fast α:
> prices diverge, bands 1e5 and up. Slow α: markets **stop clearing entirely** —
> under the kernel at α/100, region 0 of `lr_00` trades **zero units for the
> whole scored window**, with desks sitting at full nameplate and zero output
> stock. The sweep's apparent improvement was prices freezing, not an economy
> working. Numbers in `examples/velocity_probe.rs`.
>
> **The hypothesis this points at, stated as a hypothesis.** Posted quantities
> are price-inelastic on *both* sides: sellers post from own stock, buyers post a
> fixed physical basket capped by cash. Neither side is a schedule in price, so
> the price does no allocative work within a tick — it is a slow outer feedback
> loop acting through budget constraints and margins, with a lag. Whether that
> loop converges is a timescale question, and neither α = 0.1 nor α = 0.001
> answers it. If that is right, the binding constraint is in
> [markets.md](architecture/markets.md)'s clearing design, which both arms share,
> and no amount of Rule 1/2/3 work reaches it.
>
> **Not established, and it must be before this is acted on:** whether the
> kernel's zero-trade at low α is that mechanism or a bug in the kernel. It has
> to be run down first — this project has twice had a clean analytical story
> turn out to be wrong, and a redesign launched off an unverified one would be
> the third.
>
> ---
>
> **P4.3 LANDED, and the first A/B is a decisive LOSS.** Pop desks are in: the
> labour pair scaled by participation π driven by σ_π, the consumption desk on
> `(cash − reserve)/reserve` capped at the top basket tier, and a price logit
> replacing the stored `sub_state`. `parity` is registered per pop, derived from
> **technology** by `tools/derive_parity.py` — never from the genesis real wage,
> which would have set part of a scored series from a dial. Every lr scenario
> derives 1.0 baskets per unit of labour.
>
> Two receipts, both persisted, neither hidden:
>
> | | live regions | median band | verdict |
> |---|---|---|---|
> | pre-port, legacy → kernel | 29 → **0** | 1.30e5 → 5.9e12 × | **LOSS** |
> | ported, legacy → kernel | 33 → **0** | 1.53e5 → 2.99e14 × | **LOSS** |
>
> The pre-registered rule works exactly as intended: G1 catches the kernel
> killing every live region, G3 catches the bands widening, and G2 correctly
> reports `n=0` rather than inventing a comparison on an empty set.
>
> **A correction, and it was mine, not the spec's.** The cash reserve was
> implemented as `b_cash · S · outlay`, argued from the parameter table's
> "activations of outlay" and from `b_cash · S = 26` matching the legacy dividend
> `reserve_multiple`. Wrong: legacy's 26 is a *dividend retention* threshold,
> Rule 3's reserve is a floor on *spending*. A building holding 50 against a
> 228.8 reserve had `budget = 0`, so it bought nothing, produced nothing, earned
> nothing, and could never climb out. Under kernel.md's literal formula the
> reserve is 14.3 against a budget of 35.7 and inputs costing 35.2 — close enough
> that the corpus's genesis cash was plainly sized against that reading. Fixed;
> the absorbing state is recorded rather than guarded, because a guard would hide
> it from the Phase 5 phase map that ought to find it.
>
> **What the kernel now does, and what it does not.** With a genesis scale and a
> working reserve the economy runs — desks trade, inventories build, scale moves
> both ways, every engine battery passes. It is nonetheless violently unstable:
> on `lr_00` flour goes 0.6 → 2.3 → 3.6e3 → 4.97e7 → 1.8e4 while wheat and labour
> go to zero. Prices adjust ~10%/tick (registered α) against scale's ~1%/tick, so
> quantities cannot answer a price move before the next one arrives. That
> timescale ratio is a Phase 5 question — α is not currently a phase-diagram axis
> and on this evidence it should be.
>
> **The kill condition is in view and is not yet met.** METHODOLOGY's standing
> rule is to stop and rethink the agent design if the kernel cannot pass the
> Phase 4/5 gates after honest effort. Two known mechanisms are still missing
> (P4.4 overflow routing, P4.5's checks), so the effort is not yet honest-complete
> — but a kernel that takes 33 live regions to 0 is not a near miss, and the next
> session should decide between finishing P4.4–P4.5 and taking the timescale
> ratio to Phase 5 first.
>
> **One decision P4.3 must take deliberately.** `parity` is registered per pop
> in *goods per unit labour* (Phase 3.5 specified the units and deferred the
> field). Setting it from the genesis real wage would make σ_π ≈ 0 at t=0 — and
> `RealWage` is one of the two metrics the A/B's band is computed on, so that
> choice paints part of the scored series. It must instead be derived from
> *technology* (labour embodied in one basket, times a stated home-production
> penalty), and the derivation written down before the run.

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
