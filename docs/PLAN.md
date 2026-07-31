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
> ### Can the price rule lose its α entirely?
>
> Asked directly, and the answer turns out to be *no, and the reason is the
> interesting part*.
>
> **The normaliser destroys magnitude.** `(d − s)/max(d, s)` saturates at ±1, so
> a market short by 10× and one short by 1000× both get a step of exactly α. At
> α = 0.1 a market mispriced by 1000× needs **73 ticks** to get there. The rule
> cannot tell a small shortage from a catastrophic one — the same class of
> mistake as the detectors Phase 3.5 retired.
>
> **There is a parameter-free rule, and it is derivable rather than invented.** A
> pop wanting `Q` with cash `C` posts `min(Q, C/p)`, so once its budget binds its
> demand is unit-elastic. Against supply posted from own stock — vertical — the
> market clears at `p* = C/s = p·d/s`. So `p ← p · d/s`: no α, one step, exact.
>
> **Measured, it does not work — and the shape of the failure is the finding.**
> Implemented as `--price-rule ratio` and run across the corpus under both arms:
>
> | arm | price rule | live | median band | within 2.2× |
> |---|---|---|---|---|
> | legacy | imbalance | 33 | ~~1.64e5~~ **1.525e5** | **0/72** |
> | legacy | ratio | 32 | ~~2.94e5~~ **1.725e5** | **5/72** |
> | kernel | imbalance | 8 | ~~3.32e15~~ **3.270e15** | 0/72 |
> | kernel | ratio | 0 | ~~2.21e12~~ **1.105e12** | 0/72 |
>
> > **The median-band column above is SUPERSEDED (2026-07-31,
> > `results/ab/receipt-foundations.md` §7).** The `live` and `within 2.2×`
> > columns reproduce exactly. The medians do not: a git worktree at `8ba33f9`
> > — *this* commit, its own tapes, its own binary — was built and all four
> > cells re-run, and it produces the struck-through values' replacements, not
> > the struck-through values. The cause was not determined; `2.21e12 /
> > 1.105e12` is exactly 2.000, which hints at a different aggregation over the
> > same readings rather than a different run, but that is a guess and is not
> > asserted. Nothing downstream of this table changes: the conclusion below
> > rests on `live` and `within 2.2×`, both of which stand.
>
> **Bimodal.** The ratio rule produced the first regions ever to land inside the
> 2.2× band — 5 of 72, against 0 for every configuration tried before — *and* a
> worse median and fewer live regions. That is exactly what the derivation
> predicts: where demand is budget-constrained the rule clears in one step and is
> perfect; where quantities are posted price-**in**elastically it re-derives the
> same ratio every tick and diverges geometrically. Damping is what stops that,
> which is why α was there.
>
> **So α is not a tuning parameter. It is a stabiliser standing in for a demand
> elasticity the model does not have.** No step-size formula removes it, because
> the loop gain depends on an elasticity that posted scalar quantities do not
> carry. Removing α needs the market to have something to *solve*, not something
> to step: agents posting **schedules** — quantity as a function of price, i.e.
> reservation prices on orders — so the clearing price is computed rather than
> groped toward. Then there is no step size and no one-tick lag, and the whole
> α/η timescale question disappears.
>
> That is a change to [markets.md](architecture/markets.md)'s clearing design,
> shared by both arms, and it is upstream of everything Phase 4 has been doing.
> It is also squarely inside METHODOLOGY R8's territory — rationing is the point,
> and a limit-order book rations by price and quantity together — so it needs its
> own design note and criteria before code (R14), not an opportunistic patch.
>
> The derivation above said the ratio rule would work. The measurement said it
> does not. Recorded in that order deliberately.
>
> ### Not schedules — a price-responsive Rule 1
>
> The "post schedules" conclusion above was an over-reach, and the correction is
> the actual finding. An order book is not needed. What is needed is only that
> **an equilibrium price exist** — that `s(p)` and `d(p)` cross somewhere — and
> for that an agent need only do what it can already do: read the posted price
> and decide its quantity from it. Reading a posted price was never forbidden;
> the invariant bans another agent's *demand or fill*, and names "a posted price"
> among the things a desk may read.
>
> **Measured** with `examples/elasticity_probe.rs`, which perturbs prices by 1%
> on a clone and re-runs the decision phase — the same differential trick B6
> uses:
>
> | arm | good | posted supply | ε_supply | ε_demand |
> |---|---|---|---|---|
> | legacy | flour | 10.94 | 7.03 | −0.013 |
> | legacy | labour | 21.80 | **0.000** | −0.001 |
> | legacy | services | 7.33 | **0.000** | 0.000 |
> | kernel | flour | 13.10 | **0.000** | −1.000 |
> | kernel | labour | 0.41 | **0.000** | 0.000 |
>
> **Posted supply has an instantaneous elasticity of exactly zero, for every
> good, under the kernel.** Rule 1 — `max(inventory − b_out·scale·qty, 0)` —
> mentions no price. The labour market is the extreme case: supply 0.41 against
> demand 73.7, a 180× gap, with *both* sides at zero. No price clears it, so the
> price rule is chasing a crossing that is not there, and neither α's value nor
> the ratio rule can matter.
>
> Flour's demand elasticity is exactly **−1.000** — the budget cap binding, unit
> elastic — which is precisely the case where `p·d/s` clears in one step. That is
> why 5 of 72 regions landed inside the band under the ratio rule and the rest
> diverged: one mechanism, both outcomes.
>
> **This is the same finding as the α/η timescale one, seen from the other side.**
> Supply *does* respond to price — through `scale` and `π`, at ~1%/tick, with a
> one-tick lag. A lagged response is what makes a loop oscillate; an instantaneous
> one is what gives the price rule a crossing to find *now*. The engine has only
> the former.
>
> **The direction, and it is small and inside the kernel's own idiom.** Make Rule
> 1's posted quantity a function of the posted price against the desk's **own
> reservation price** — its unit cost, or its own `price_ema`, both own-state.
> Release more of the buffer when the price is above your cost, hold when it is
> below. That is price-responsive (a crossing exists), own-state (B6 and the
> invariant still hold), and **stretched across many ticks by construction**: the
> inventory buffer is the intertemporal link, held through cheap ticks and
> released into dear ones. `b_out` stops being an inert constant and becomes the
> supply curve's slope. The symmetric change applies to Rule 3's buying.
>
> **LANDED AND MEASURED 2026-07-31** — see "The price-responsive Rule 1, landed
> and measured" at the end of this phase. The direction above is confirmed on the
> instrument (epsilon_s moves off zero, degree-0 in prices, B6 still clean) and
> the corpus still loses the A/B: the storable half tightens the median band by a
> factor of 20 and the labour half takes every region to DEAD.
>
> That wants its own design note and criteria before code (R14). It is a change
> to Rule 1, not to markets.md's clearing — the earlier "schedules" claim is
> **superseded** and left above as the record of a wrong turn.
>
> ### What the lr corpus actually is, and what that does to the A/B
>
> **Provenance, from its author (2026-07-31):** the lr set was generated as a
> *sweep over region types looking for one that was stable* — not as 24 worlds
> each expected to work. Most were expected to be unrealistically broken, fixable
> only by development, tech discovery and population dynamics that do not exist
> yet. Possibly only one or two cells are realistic enough to be a target at all.
>
> `lr_manifest.csv` confirms the shape: a factorial grid over
> `labour_supply × channel_size × wheat_supply × start_state`, 24 cells. That is
> a design-of-experiments search, and the evidence agrees that it has structure —
> `start=G` dominates `start=I` across every configuration tried.
>
> **But the data cannot yet name the viable cells, and that is the finding.**
> Ranking all 24 by median band across six configurations (two agent arms × two
> price rules, plus two α settings) gives a *smooth gradient* of mean rank from
> 7.2 to 17.7, not the bimodal split that one or two genuinely viable worlds
> would produce. Rank instability is severe: nearly every scenario has placed
> both top-3 and bottom-5 depending on configuration. **The engine's dynamics
> dominate the world's parameters.**
>
> And the sharpest number: across every configuration run, exactly **5 of 72
> regions have ever landed inside the 2.2× band — and all five were DEAD.**
> `lr_20/Birmingham` scores a perfect 1.000 because its metrics are flat, not
> because it is healthy. **No region has ever been both alive and in-band.** So
> the corpus currently cannot discriminate "broken by design" from "broken by the
> engine", because the engine breaks all of it.
>
> **Two consequences, and they pull in opposite directions.**
>
> 1. **"0/72" was never a fair indictment of the kernel** — nor of the legacy
>    layer. A search grid is not a validation set, and scoring an agent design on
>    cells that were never meant to clear measures the wrong thing.
> 2. **The A/B's pre-registered gates use the wrong unit.** They aggregate over
>    72 regions treated as equally valid targets. If most were never targets, G1
>    and G3 are averaging over noise.
>
> **What a legitimate fix looks like, and what it must not be.** Narrowing to the
> realistic cells requires a **dated amendment** to `results/ab/preregistration.md`
> naming the target set *and the rule that picked it*, written before it is
> scored (R6). The rule must come from **tape properties or stated design
> intent** — never "whichever cells the kernel does best on", which is the one
> move that would make the whole comparison worthless. The author's original
> design intent is a legitimate, non-fitted input; it is also the one input the
> repository does not contain, so it has to be supplied rather than inferred.
>
> Until then the A/B receipts stand as measured, with this caveat attached to
> them rather than quietly folded into them.
>
> ### The missing instrument: a correctness criterion
>
> **Every criterion in this project is a *stability* criterion.** `DRIFTING`,
> `SWINGING`, `UNSTABLE`, `DEAD` all ask *did it stay put*. Not one asks *did it
> go to the right place*. That is why the work keeps feeling like a chicken-and-
> egg problem: when something will not settle there is no way to tell a missing
> feature from a broken mechanism, because there is no case where the right
> answer is known.
>
> **The tape already contains the right answer.** Technology implies relative
> prices: in `lr_00`, `wheat_farm` embodies 0.3 labour per wheat and `grain_mill`
> adds 0.2, so flour embodies 0.5 and a zero-profit equilibrium has
> `RealWage = p_labour/p_flour = 1/0.5 = **2.0**`. `RealWage` is *already a scored
> metric*. Nothing has ever checked it against this. The same Leontief solve
> already exists in `tools/derive_parity.py`.
>
> **Measured against it, and the result separates two problems that have been
> tangled together all phase:**
>
> | configuration | median RealWage | band | vs. 2.0 |
> |---|---|---|---|
> | legacy, α×1 | 0.0066 | 11.5 | off by **300×** |
> | kernel, α×1 | 4.9e-10 | 1.25e17 | off by **4×10⁹** |
> | legacy, α×0.01 | **0.799** | **1.72** | off by **2.5×** |
>
> At slow α the legacy arm sits within a factor of 2.5 of the analytically
> correct relative price, with a band of 1.72 — **inside the 2.2× bar**. That is
> the closest anything in this project has come to being right, and it says the
> mechanism is not incapable of price formation; it is being run outside the
> region where it works.
>
> **But the same run has 23.5% dead ticks and a cleared-units band of 4.4e5.** So
> the honest decomposition is:
>
> - **relative prices** — nearly right at slow α, both arms improving with α;
> - **volumes and liveness** — broken at every setting tried.
>
> Two separable problems, which is exactly the tool the chicken-and-egg needs.
> And the error has a *direction*: the real wage is too **low** by 2.5×, i.e.
> labour is underpaid against its embodied value — consistent with the labour
> market having no crossing (supply 0.41 against demand 73.7, both elasticities
> zero). The correctness test and the elasticity finding point at the same market.
>
> **On the missing-feature worry, what the evidence says.** The α×0.01 run gets
> the relative price nearly right with a *fixed money stock and no credit*, so
> monetary machinery is not what stands between here and price formation — it
> would matter for the price *level* and for smoothing. Memory already exists
> (`price_ema`, `last_fill`). And the one ingredient actually identified as
> missing — a desk setting its quantity from the posted price against its own
> cost — is a **reservation price against own state, not a forecast of anybody
> else**. On present evidence inter-agent prediction is *not* required.
>
> **Proposed next artifact, ahead of any more kernel work:** a correctness
> battery. Solve the tape's labour values, derive the implied relative price
> vector, and report per-run how far the realised prices sit from it. It costs
> almost nothing — the solver exists — and it converts "it did not stabilise"
> into "*this* market is at the wrong price, by *this* factor, in *this*
> direction". Stability then becomes the second question rather than the only one.
>
> **LANDED as B8 (2026-07-31), `src/certify/technology.rs`.** The Leontief solve
> is ported from `tools/derive_parity.py` — one algorithm in two languages, so
> the labour values that set a pop's `parity` and the ones that score B8 cannot
> disagree. `lr_00` comes out at wheat 0.3, flour 0.5, services 1.0, implied
> `RealWage` **2.0**, as computed by hand above.
>
> B8 reports `ln(realised / implied)` per (region, good) over the scored window,
> and names the good and the direction. Measured on `lr_00`, 1,000 ticks,
> imbalance rule, window 150–1000:
>
> | arm | α | worst reading | Manchester flour |
> |---|---|---|---|
> | legacy | ×1 | wheat **2.9e6×** too dear (Birmingham) | 209× too dear |
> | legacy | ×0.01 | flour **2.27×** too dear (Leeds) | 2.25× too dear |
> | kernel | ×1 | flour **3.5e11×** too dear (Leeds) | 3.8e10× too dear |
> | kernel | ×0.01 | flour **2.37×** too dear (Leeds) | 2.14× too dear |
>
> Three findings the stability criteria could not have produced:
>
> 1. **The α claim survives contact with an absolute benchmark, on both arms.**
>    It was previously a claim about a median of one metric on one arm; it is now
>    a claim about the whole price vector, and the kernel improves by eleven
>    orders of magnitude when α is slowed. The timescale ratio is the finding,
>    not the agent layer.
> 2. **The error has a uniform sign.** Across all 26 labour-bearing scenarios at
>    the registered α under the legacy arm, *every* good in *every* region reads
>    **too dear** against labour — i.e. labour is underpaid against its embodied
>    value everywhere, never overpaid. That is the same market the elasticity
>    probe found has no crossing, seen from the price side.
> 3. **Stable and correct are genuinely different.** `lr_00`/Manchester has the
>    tidiest `RealWage` band in the corpus (11.5) and its flour is still 209×
>    too dear. Nothing before B8 could say that.
>
> `tracer_2r` at the 11,700-tick horizon reads grain **3.96×** too dear — the
> closest thing in the corpus to a run that is merely wrong rather than absurd,
> and note the known sign of the error there: `grow_grain`'s input is `Fixed`, so
> below full utilisation it really does draw more labour per grain than the
> benchmark credits, which flatters the producer in exactly this direction.
>
> **B8 carries no distance threshold, and that was the hard call.** The numbers
> above were already known when it was written, so any bar between ~3× and ~200×
> would have been a criterion fitted to a result — the R6 sin, and worse than
> usual here because this is the *first* correctness criterion and a bar chosen
> now would define what correctness means for everything after it. There is also
> no bar available from outside the data: a competitive-equilibrium vector is a
> limit, not a tolerance. So B8 **reports always and fails only fail-closed** —
> on a traded good whose implied value cannot be computed, or whose realised
> relative price has no logarithm anywhere in the window. The consequence is
> stated rather than hidden: **a run can be 3.5e11 off and see B8 PASS.** When
> the corpus produces a region that is both alive and in-band, a dated criteria
> file can register a bar against *that* evidence.
>
> > ##### [CORRECTED 2026-07-31, adversarial review] B8 was a REPORTER wearing a battery's label, and seven defects
> >
> > The paragraph above reaches the right *action* — no threshold — on an
> > argument that has since been superseded, and it left the instrument carrying
> > the label `prices match technology` while printing PASS beside a 3.5e11×
> > gap. Every certificate in the corpus therefore asserted something no code in
> > it had checked. **Renamed to `price/technology gap (report, no bar)`**; a
> > PASS now means *the distance was computable* and nothing else.
> >
> > **The argument is also replaced, and the new one is stronger.** "No evidence
> > for a bar yet" implies a bar arrives with more evidence. It does not, for
> > *this* statistic: the benchmark is zero-rent, and `solv_labour` — the one
> > world in the corpus whose answer is derived on paper and registered before
> > the run — is **exactly right** at `p_grain = p_labour` while B8 reads it
> > "2.000× too dear", because the firm earns a capacity rent a zero-rent
> > benchmark cannot see. A bar at 2× fails a correct world; a bar above 2× is
> > chosen to let a known false positive through. No threshold repairs a
> > specification error. What would make it a battery is recorded in
> > `PriceGapWatch::report`: a rent-aware implied vector and a per-node
> > valuation, after which a per-scenario bar can be registered against a
> > derivation rather than a measurement.
> >
> > Six further defects, all repaired, each with a falsification test in
> > `tests/test_11_correctness.rs` §6 that was watched failing first:
> >
> > 1. **R5 breach.** `log_sd` was `f64::NAN` at `n < 2` and went straight onto
> >    a passing certificate line; `certify::nan::scan` walks `SimState` and
> >    could never see it. Now `Option<f64>`, rendered as an absence, with a
> >    fail-closed sweep (`PriceGapWatch::uncomputable`) behind it.
> > 2. **The numeraire was never checked.** Every reading divides by
> >    `p_labour`, and `observe` gated only on the scored good. A region whose
> >    labour market posted nothing all window still produced 851 confident
> >    readings. Ticks without a traded numeraire now contribute nothing and are
> >    counted.
> > 3. **N findings that were one finding.** `lr_00`/Manchester's wheat 312×,
> >    flour 209×, services 43,415× is a **common factor of ~1,414×** — a
> >    statement about the wage — plus residuals of 4.5× *too cheap*, 6.8× too
> >    cheap and 30.7× too dear. Two of the three raw numbers point the opposite
> >    way to the truth about their own market. The line now carries the
> >    decomposition.
> > 4. **Silent sample loss.** A run that discarded 850 of 851 ticks reported
> >    `n=1`. The denominator and the reason are now on the line.
> > 5. **The benchmark's assumptions are now printed** — zero-rent,
> >    full-utilisation, single-node — because finding 5 above was mistaken for
> >    an error once already.
> > 6. **Single-node bias, disclosed and bounded rather than fixed.**
> >    `flour_transport`'s 0.1 labour is in no good's value, so a competitive
> >    importer scores 1.200× too dear. Per-node valuation was considered and
> >    rejected with reasons (`LabourValues::pass_through`); the line prints the
> >    tape's whole bias instead.
> > 7. **`NotConverged` marked the whole graph**, labour included, so one
> >    runaway cycle made the numeraire unpriceable. Only goods still moving and
> >    what they feed are marked now. Measured while fixing it: the 256-sweep cap
> >    trips on a *well-posed* cycle at round-trip gain 0.90 whose exact answer is
> >    19.0, so hitting the cap is a statement about the solver, not the graph.
> >    That remains open.
>
> ### The solvable scenarios: worlds whose answer is known before the run
>
> **LANDED 2026-07-31.** `data/scenarios/solv_1g`, `solv_1g_money`,
> `solv_1g_money_2x`, `solv_chain`, `solv_labour`; `tests/test_12_solvable.rs`;
> `src/scenario/equilibrium.rs`. Design and derivations:
> `docs/design/solvable-scenarios.md`. Five worlds whose competitive equilibrium
> — price vector, quantity vector, money stock and metric series — is written
> down in closed form, by hand, from the tape, **before the engine was run**. The
> target lives in each tape's own header and in an `equilibrium.ron` beside it,
> not in a test file.
>
> This closes the gap B8 could not: B8 measures a distance and carries no bar
> because no bar could be defended for `lr_*`. Here the bar *is* derivable —
> mode A's `1e-12` is float slack on a dyadic-rational fixed point, and mode B's
> `ln(1.10)` comes from `dead = 0.05` and `eta_dn = 0.05` and nothing else.
>
> **RESULT 1 — the kernel reproduces a hand-computed equilibrium exactly.** All
> five tapes, both price rules, worst `|ln(realised/target)|` over 1,000 ticks:
> **`0.000e0`**. Not "within tolerance" — identically zero, on the whole price
> vector including `solv_chain`'s intermediate `p_wheat = 0.25` and
> `solv_labour`'s interior labour market. Certificates persisted under
> `results/solv_*`: all eight batteries PASS and 1/1 regions pass on stability,
> the first full-PASS certificates in the repo that are also *correct*. B8 reads
> `1.000x, ln +0.000, sd 0.000`. **The mechanism can form prices. What it cannot
> do is find them.**
>
> **RESULT 2 — nothing attracts to the fixed point.** Displace one genesis price
> by 2× and every configuration's traded volume decays across the scored window
> (last tenth over first tenth; 1.0 would mean no decay):
>
> | | imbalance | ratio |
> |---|---|---|
> | `solv_1g` | 0.068 | **0.000** |
> | `solv_1g_money` | 0.356 | **0.000** |
> | `solv_chain` | 0.000 | **0.000** |
> | `solv_labour` | 0.204 | **0.000** |
>
> All four `ratio` columns traded exactly nothing for the last 800 ticks. Seven
> of the eight also miss the registered `ln(1.10)` on price, by 1.9× to 1.5e11×.
> **This is a sharper statement of Phase 4's central finding than the elasticity
> probe could make**: it is not only that the price rule chases a crossing that
> does not exist — even where a crossing provably *does* exist and sits one
> factor of two away, the mechanism cannot walk to it.
>
> **RESULT 3 — `solv_labour` is a trap for exactly the instrument that found
> these.** Its equilibrium `RealWage` is **1.0**, half the labour-value answer,
> because `recipe_size` binds and the firm earns a capacity rent. B8 duly reports
> grain **2.000× too dear** — and B8 is wrong, by construction, in a way
> registered in the tape before the run. A Leontief benchmark cannot see a scarce
> second factor. That limit is now demonstrated rather than suspected.
>
> **RESULT 4 — money is exactly neutral.** `solv_1g_money` against
> `solv_1g_money_2x` (every price and currency balance doubled, nothing else):
> every real series bit-identical and every price exactly 2× for 1,000 ticks, to
> 0 ulp. That is the sharpest available R2 audit and the kernel passes it. One
> absolute currency constant is known to exist — `overflow > 1e-12` in
> `rule_3_buy_and_route` — and is recorded in the 2x tape's header; it cannot
> bind at a fixed point where the overflow is identically zero.
>
> **RESULT 5 — the two arms fail differently, and no single number could have
> said so.** Legacy cannot hold any fixed point (worst `|ln|` 1.48 / 1.51 / 3.02
> / 20.62) **but keeps its markets alive** at 0.98–1.00 of target volume; the
> displaced kernel gets far closer on price and lets the market die. `solv_labour`
> under legacy reproduces its own pre-registered arm prediction: the legacy layer
> has no participation margin, so B7 catches `node0/labour pinned at -0.403032
> for all 851 ticks` and the wage decays geometrically to 9e8× wrong.
>
> **Two side findings, recorded where they were found rather than in a notebook.**
> (a) The wealth-tier dead band absorbs a 0.77% cash shock *completely and
> permanently* — the neutrality falsification control had to be widened from 1
> currency unit to 10 before the world diverged at all, because `sigma_c` inside
> `dead` leaves every order identical and the balanced cash flow holds the
> shortfall forever. (b) A **median** price gap cannot tell a converged run from a
> symmetric orbit: `solv_labour`/imbalance reads 1.006× on the median while its
> real-wage band is 8.1× and its volume falls to a seventh of target. Every
> reading in `test_12_solvable.rs` is therefore reported beside the band and the
> volume, and the mode-A test gates on volume independently of price.
>
> **One loader change was required and is registered:**
> `RawPopGroup.participation`, absent-means-1.0, because `solv_labour` is the
> first world with a scarce second factor and therefore the first with an
> *interior* equilibrium labour margin (`pi* = 0.5`). Genesis `pi` had been a
> hardcoded 1.0 since the loader was written, invisible because `sigma_pi = +1`
> identically in any one-factor economy. No existing tape changed.
>
> **What this does NOT say.** Nothing here is evidence that any `lr_*` scenario
> is fixable, and mode A passing is a *necessary* condition, not a sufficient
> one — a world that starts at rest and stays there has not demonstrated price
> formation, only the absence of a spurious response. Result 2 is the load-bearing
> one for Phase 5: the phase diagram should be swept for a region where mode B
> converges at all, and `alpha`, `b_out` and `b_cash` are the three dials the
> derivations name.
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

> ---
>
> ### The price-responsive Rule 1, landed and measured (2026-07-31)
>
> `docs/design/price-responsive-supply.md` is implemented behind a registered
> switch, `kernel.supply_rule`, in all 33 tapes. **Every tape registers
> `inelastic`** — the shipped rule — because R10 says a change ships when the
> certified suite says it is no worse, and on the pre-registered gates this one
> is not. `--supply-rule` runs the alternatives and the effective value is folded
> into the run identity (`kernel+imbalance+reservation`), so no two certificates
> for one tape can be confused. Three values:
>
> | value | Rule 1, storable outputs | Rule 1, pop labour |
> |---|---|---|
> | `inelastic` | `max(I − b_out·flow, 0)` | `π·H` |
> | `reservation_goods` | `max(I − b_out·flow/R, 0)` | `π·H` |
> | `reservation` | `max(I − b_out·flow/R, 0)` | `min(H, π·H·max(1 + b_out(1 − w_res/w), 0))` |
>
> `reservation_goods` is a **decomposition arm, not a third design**: the change
> hits two different markets and the first A/B could not say which half moved the
> result. It turned out to be the whole story. Rule 3's mirror (design §5) is
> deliberately **not** implemented: it moves `ε_d`, and two mechanisms landing
> together have one receipt between them.
>
> #### 1. The elasticity moved off zero, exactly as derived
>
> `examples/elasticity_probe.rs` was **blind and had to be fixed first**: it
> bumped every price at once, and the new rule is homogeneous of degree 0, so it
> would have read `ε_s = 0.000` for a working rule as loudly as for a broken one.
> It now perturbs one `(node, good)` at a time and keeps the uniform bump as a
> control. `lr_00`, kernel arm, own-price `ε_s`:
>
> | good | shipped | reservation (storables) | uniform control |
> |---|---|---|---|
> | wheat | **0.000** | 1.11 / 16.7 / 43.5 / 45.2 | 0.000 |
> | flour | **0.000** | 0.216 / 6.10 / 6.73 / 17.7 | 0.000 |
> | labour | **0.000** | 1.67 / 2.53 / 3.19 *(full rule only)* | 0.000 |
> | services | **0.000** | **0.000** | 0.000 |
>
> The spread is `ε_s = I/q − 1`: `b_out/R = 2` at a desk's resting point, larger
> the further it is from one. Several readings exceed `2/α = 20`, i.e. those
> markets are outside the derived stability bound — a desk far from rest posts
> less than a twentieth of its stock. Services stay at 0.000, which is §2.3's
> pre-registered prediction arriving as a measurement: a non-storable's cost is
> sunk and its supply is vertical, correctly.
>
> > ##### [CORRECTED 2026-07-31, second pass] the elasticity table above is struck: it is not reproducible as labelled
> >
> > Adversarial review could not reproduce it, and re-measurement says why. **The
> > table states no tick count and no evolution rule, and its cells do not share
> > either one.** Held to a single stated condition the figures do not appear
> > together; each was taken wherever it was largest:
> >
> > | quoted | actually exists only at |
> > |---|---|
> > | wheat `1.11` | tick **20**, state evolved under the **shipped** `inelastic` rule |
> > | wheat `16.7` / `45.2`, flour `0.216` / `6.10` | tick **20**, state evolved under **`reservation_goods`** |
> > | wheat `43.5`, flour `6.73` / `17.7` | tick **200**, state evolved under **`reservation_goods`** |
> > | labour `1.67` / `2.53` / `3.19` | tick **5**, posting under the full `reservation` rule |
> >
> > That is not a property of the rule. `ε_s = I/q − 1` is a function of the
> > **state**: the same rule reads 0.000 at a desk posting its whole stock and 65
> > at a desk posting a sixty-fifth of it. An elasticity with no stated state is
> > not a measurement, and this one silently mixed two trajectories — including
> > the one the corpus does not run, since all 33 tapes register `inelastic`.
> >
> > **Re-measured, with conditions.** `examples/elasticity_probe.rs` now prints
> > the scenario, tape sha, arm, evolution rule, posting rule, tick count and
> > `state_hash` above every table, takes a *list* of tick counts, and reports the
> > loop gain per market. Both blocks below: `data/scenarios/lr_00`,
> > `tape_sha=9c701ab41330554a`, arm **kernel**, own-price central ±1% difference
> > in log price (the figures the old table quoted were one-sided +1%; those are
> > given in parentheses where the two differ visibly). `—` = the market posted
> > nothing on that side, so there is no reading (R5: absent, not zero).
> >
> > **A. State evolved under `inelastic` — the rule every tape registers —
> > posting measured under `reservation`.**
> >
> > | node / good | t=5 `44a3a23f…` | t=20 `99a006a1…` | t=40 `0e1785ca…` | t=200 `f17fb500…` |
> > |---|---|---|---|---|
> > | 0 wheat | — | 1.123 (1.111) | 0.262 | — |
> > | 1 wheat | — | 0.549 | 0.265 | — |
> > | 2 wheat | — | 0.830 | **8.778** (8.352) | — |
> > | 0 flour | — | — | — | 0.000 |
> > | 1 flour | — | — | 0.249 | 0.000 |
> > | 2 flour | — | 0.377 | 0.012 | 0.000 |
> > | 0 labour | 2.580 (2.534) | — | — | — |
> > | 1 labour | 3.261 (3.192) | — | — | — |
> > | 2 labour | 1.876 (1.669) | — | — | — |
> > | 1 services | — | 0.000 | 0.000 | 0.000 |
> >
> > **B. State evolved under `reservation_goods`, posting measured under
> > `reservation_goods`** — where the large figures live. (At t=5 the two
> > trajectories are still the same state, hash `44a3a23f…`, and they are still
> > identical at t=6; the first tick whose `state_hash` differs is **t=7**, which
> > is measured, not explained — no desk posts a different quantity before it.)
> >
> > | node / good | t=5 `44a3a23f…` | t=20 `a1c9784a…` | t=40 `f049eb05…` | t=200 `c1e76165…` |
> > |---|---|---|---|---|
> > | 0 wheat | — | 1.118 (1.106) | 1.294 | **61.542** (43.498) |
> > | 1 wheat | — | **18.490** (16.707) | **21.818** (19.362) | — |
> > | 2 wheat | — | **65.482** (45.221) | — | — |
> > | 0 flour | — | — | 15.369 (14.123) | 7.008 (6.729) |
> > | 1 flour | — | 0.217 (0.216) | 0.046 | 0.000 |
> > | 2 flour | — | 6.333 (6.102) | 0.103 | **19.686** (17.674) |
> > | labour, every node that posted any | 0.000 | 0.000 | 0.000 | 0.000 |
> > | services, every node that posted any | — | 0.000 | 0.000 | 0.000 |
> >
> > Labour reads 0.000 throughout block B **because `reservation_goods` runs the
> > shipped `π·H` labour rule** — that is the arm's definition, not a finding. The
> > only labour elasticities this change produces are block A's `t=5` row, under
> > the full rule, and they are gone by `t=10` because that is when the pop is
> > shut out of the market (§4 below).
> >
> > **The series, so no single tick can stand in for "the" elasticity.** Maximum
> > `ε_s` over any market, 19 tick counts from 1 to 200:
> >
> > | evolved under | ticks with max < 1.2 | worst | where |
> > |---|---|---|---|
> > | `inelastic` (posting `reservation`) | 14 of 19 | **31.7** (26.6 one-sided) | t=7, node 1, wheat |
> > | `reservation_goods` | 5 of 19 | **125.2** | t=30 |
> >
> > What survives unchanged: **the uniform control reads exactly 0.000
> > everywhere** (degree-0 homogeneity, the reason the old probe was blind), and
> > **services read exactly 0.000 under every rule at every tick** — §2.3's
> > pre-registered prediction, arriving as a measurement. What does not survive
> > is the phrase "0.216 to 45.2" as a description of the rule. The correct
> > statement is the sentence the old text ends with: the reading *is* `I/q − 1`,
> > it moves with the state, and the corpus does not sit at a resting point.
> >
> > The one place a resting point exists, the derivation is exact: `solv_1g` under
> > `reservation` measures `ε_s = 2.000` at tick 50 and again at tick 200, which
> > is `b_out/R` at `R = 1` to three decimals
> > (`the_gain_instrument_reads_the_derived_number_where_a_desk_actually_rests`).
>
> #### 2. The solvable worlds: the first displaced market that does not die
>
> Mode A is **bit-identical** under both rules on the four zero-profit tapes
> (`solv_1g`, `solv_1g_money`, `solv_1g_money_2x`, `solv_chain`) — the
> reduction to the shipped rule at `R = 1` is exact, over 1,000 ticks of a
> multiplicative loop.
>
> Mode B, displaced 2× on one price, kernel arm, imbalance rule, shipped rule →
> `reservation_goods`:
>
> | tape | median price gap (bar 1.10×) | volume trend |
> |---|---|---|
> | `solv_1g` | 2.06× → **1.52×** | 0.068 → **1.134** |
> | `solv_1g_money` | 1.92× → 1.96× | 0.356 → 0.274 |
> | `solv_chain` | **1.5e11× → 2.12×** | 0.000 → 0.058 |
> | `solv_labour` | 1.006× → 1.024× | 0.204 → 0.113 |
>
> **`solv_1g` at 1.134× is the first configuration in this repository where a
> displaced world keeps its market**, and it directly amends RESULT 2 above,
> which is marked in place in `tests/test_12_solvable.rs` rather than rewritten.
> `solv_chain` going 1.5e11× → 2.12× against a bar of 1.10× is the largest single
> improvement any change in Phase 4 has produced on a world with a known answer.
> Under the `ratio` price rule every configuration still trades nothing, and the
> full `reservation` rule reads a volume trend of 0.0000 on all eight.
>
> **And mode A now BREAKS on `solv_labour`.** Its equilibrium markup is **R = 2**,
> not 1 — `recipe_size` binds and the firm earns a capacity rent — so the band
> halves, the desk posts more than it produces, and a fixed point the shipped rule
> holds to 0 ulp stops being one. `R = 1` is the reservation price only in a
> zero-rent world. That is the same tape that is already the registered
> counterexample to reading B8 as a correctness verdict, and it is now a second
> standing counterexample. Asserted as a failure in
> `mode_a_breaks_on_solv_labour_because_its_equilibrium_markup_is_two_not_one`
> rather than patched: a rent-aware reservation price is a new mechanism.
>
> #### 3. The lr corpus: the full rule destroys it, the goods-only arm does not
>
> 24 scenarios × 3 regions, 1,000 ticks, imbalance rule. Receipts persisted:
> `results/ab/receipt-supply-goods.md`, `-reservation.md`, `-within-kernel.md`.
>
> | | legacy | kernel/inelastic | kernel/reservation_goods | kernel/reservation |
> |---|---|---|---|---|
> | live regions | 33 | 8 | 5 | **0** |
> | median band | 1.53e5× | 3.27e15× | **6.53e8×** | inf |
> | worst band | 1.06e234× | 9.28e35× | **2.73e16×** | 2.11e19× |
> | median B8 worst gap | 1.90e7× | 1.78e11× | **1.61e8×** | 1.53e23× |
> | DEAD | 30 | 28 | **0** | **72** |
> | POP_DESTITUTION | 9 | 60 | 67 | **72** |
> | CURRENCY_DRAIN | 3 | 9 | 28 | 2 |
> | bands within 2.2× | 0 | 0 | 0 | 2 (all dead) |
>
> **All three A/Bs are a LOSS.** Against legacy, both new arms fail G1 (33 → 5
> and 33 → 0). Within the kernel — `inelastic` vs `reservation_goods`, which is
> this change's own A/B — G1 fails 8 → 5 and G2 fails on its single paired
> region, while G3 passes with the corpus band a factor of **20 tighter** in
> median log terms. So the honest summary is: *the change makes prices markedly
> better and liveness slightly worse, and the pre-registered rule counts liveness
> first, on purpose.*
>
> #### 4. Why the full rule kills everything, traced rather than guessed
>
> In `lr_00`, flour has **zero posted supply from tick 0 under both rules** — that
> is pre-existing — so its imbalance is pinned at `+1` and the saturating
> normaliser marks it up by the full `α` every tick, for ever. `P_basket` is
> mostly flour, so `w_res = parity · P_basket` inflates at 10%/tick while
> `p_labour` sits in a market that is frequently two-sided-empty and therefore
> frozen. The real wage falls through the shutdown wage `(2/3)·parity` at **tick
> 7** and never returns; labour supply is identically zero from then on, nothing
> is produced, and the economy is gone by tick 9.
>
> Two consequences worth carrying forward:
>
> 1. **The "structural price floor, not a clamp" argument is half an argument.**
>    Withholding does stop a price falling to zero — and converts the same
>    starvation into an unbounded mark-*up*. B7's pins flip across the corpus from
>    `node0/wheat pinned at -1.000000` to `node0/flour pinned at 1.000000 for all
>    851 ticks`. The design predicted B7 would "fail differently"; it did, in the
>    opposite direction from the one intended.
> 2. **A reservation price indexed to a collapsing basket is indexed to a
>    runaway.** The rule is degree-0 in prices by construction (that is what keeps
>    R12), which is exactly why a runaway in everything *except* the wage cannot
>    be escaped by inflation. Any future reservation-wage rule has to be robust to
>    its own index diverging.
>
> #### 5. What this does and does not settle
>
> It settles the Phase 4 central finding's *first* clause: posted supply can be
> given a genuine, instantaneous, own-state price elasticity inside the kernel's
> idiom, with no new magnitude registered, B6 still clean, and `b_out` becoming
> the supply curve's slope as derived. It does **not** settle the second: a
> crossing existing is not the same as the economy reaching it, and on the corpus
> the binding constraint has moved rather than gone — from "no fixed point" to
> "markets that starve at tick 0 inflate without bound, and every rule keyed on a
> relative price inherits that".
>
> The next three things this points at, in order:
>
> 1. **The zero-supply market is the real defect.** `imbalance = +1` with `s = 0`
>    is not a price signal, it is a division by nothing, and both price rules
>    handle it badly (imbalance marks up for ever; ratio freezes). That is
>    upstream of both agent arms and of this change.
> 2. **`α·b_out` is now a one-dimensional Phase 5 axis** with a derived optimum at
>    ≈ 1 against a registered 0.2. If the measured optimum is not near 1 the
>    linearisation is wrong and the design note is marked superseded, not retuned.
>
>    > **[CORRECTED 2026-07-31, second pass] "a registered 0.2" is not one
>    > number, and the loop gain it implies was never measured on the engine.**
>    > `alpha` is a per-*good* dial: 0.100 for `lr_00`'s wheat, flour and
>    > services, **0.050 for labour**, 0.050 for `solv_1g`'s grain. The sweep
>    > axis is therefore `α_g·b_out` per good, and `b_out` is the only end of it
>    > the kernel can move for all goods at once. Measured on live `lr_00` states
>    > at the registered dials, the worst gain in any market is **2.28** at tick 7
>    > with the state evolved under the shipped rule, and **7.25** at tick 75 with
>    > it evolved under the full `reservation` rule; on the same 19-tick grid the
>    > per-tick worst also falls as low as 0.006. The derived 0.800 is a
>    > resting-point value, the corpus does not rest, and the spread is the
>    > finding. See `the_engines_measured_loop_gain_is_not_the_argued_0_8`.
> 3. **A rent-aware reservation price**, without which `solv_labour` cannot be
>    held and any world with a binding capacity is mispriced in a known direction.

> ---
>
> ### P4.6 — the full matrix, and nothing improved (2026-07-31)
>
> `results/ab/receipt-foundations.md`, with every per-region band and every
> pairwise comparison persisted in `receipt-foundations.json`. 261 certified runs:
> 2 agent arms × 2 price rules × 3 supply rules (the supply rule is kernel-only,
> which is *checked* — the legacy arm's B3 state hash is identical with the
> override on, 24/24), over the 24 `lr` scenarios and the 5 solvable worlds at
> genesis. Mode B has no CLI and is not in the matrix.
>
> **The headline is that there is no headline. In all eight cells: 0 of 72 `lr`
> regions pass their criteria, and 0 of 72 are both alive and inside the 2.2×
> band.** The standing claim now holds across eight configurations rather than
> six. Every legacy → kernel A/B is a **LOSS** on the pre-registered rule — six
> for six, under both price rules and all three supply rules — with liveness
> going 33 → 8 / 5 / 0 under `imbalance` and 32 → 0 / 2 / 0 under `ratio`. The
> Phase 4 gate is met by no configuration measured here, and P4.7's deletions
> remain unavailable.
>
> Four results are new.
>
> **1. THE REGISTERED RULE HAS STOPPED MEASURING THE RIGHT THING, and the failure
> is now realised rather than hypothetical.** Reported as a finding; **the rule is
> not amended and no amendment is proposed** (that needs a dated file written
> before the number is known). `tools/test_ab.py` shows collapse losing *against a
> live baseline*; two cells here are the case it does not cover — collapse on
> **both** sides — and there G1 and G2 evaporate together:
>
> - `kernel+ratio+inelastic → kernel+ratio+reservation_goods` reports **WIN**,
>   gate met, on 0 → 2 live regions out of 72, with 0 passing on both sides, the
>   marginal median band **1,685× wider** (1.105e12 → 1.862e15), and G3's paired
>   median landing *exactly* on 0 because nine both-unbounded pairs sit on it. The
>   scenario-level median says +0.58, i.e. worse. Three summaries of one
>   comparison, two of them negative, and the rule reports the third.
> - `kernel+imbalance+reservation → kernel+ratio+reservation` reports **TIE**,
>   gate met, between two arms in which all 72 regions are `DEAD` and `UNSTABLE`
>   and the challenger has 21 *more* unbounded bands.
>
> The defect is precise: G1 forbids liveness *regressing* and says nothing about
> it being *zero*; G2 is conditioned on a set that empties; G3 scores
> unbounded-vs-unbounded as a tie, which is right for a handful of blow-ups and
> wrong when they decide the median. **All three gates are relative; the rule has
> no absolute floor.** What it needs is not a threshold but a floor stated in
> advance — plausibly "a baseline with no live region yields `INVALID`, as a
> changed `tape_sha` does". Recorded as the case for a dated amendment, not as one.
>
> **2. The two arms violate exactly one invariant each, and no cell satisfies
> both.** Over the 24 `lr` scenarios: legacy passes B6 **0/24** and B7 **24/24**;
> every kernel cell passes B6 **24/24** and B7 **0/24**. On the *solvable* worlds
> every kernel cell is 5/5 on both under `imbalance`, which localises the pin to
> the corpus's zero-supply markets rather than to the kernel rule. And swapping
> the price rule alone takes legacy's B7 from **24/24 to 1/24** — "ratio freezes"
> was traced from `price_next_ratio`'s `supply ≤ 0 → return price_current`; this
> is it measured, on 23 of 24 scenarios.
>
> **3. B8's fail-closed branch fired on the corpus for the first time, and caught
> an f64 overflow.** `kernel+ratio+reservation_goods` on `lr_05`:
> `Manchester/flour no computable relative price in 851 traded tick(s)`. Traced:
> that world's tick-1000 price vector spans 3.32e306 down to 3.74e-15, and
> `p_flour / p_labour` = 3.32e306 / 0.006912 **overflows f64**. Until now the
> branch had only been shown firing on defects planted in
> `tests/test_11_correctness.rs`. Two notes on the instrument: B8's "traded"
> means *posted on at least one side*, not *cleared* (that region is `DEAD`); and
> `price_next_ratio` guards against a non-finite **price** while nothing guards
> the **quotient of two** prices.
>
> **4. The one unambiguous positive, and it is not on the corpus.** The kernel
> with the shipped supply rule holds all five hand-computed equilibria under
> **both** price rules — band `1.000×`, B8 `1.000×`, **5/5 full-PASS
> certificates** each — while the legacy arm produces **0/5** on either rule. The
> `ratio` result is new and is partly a harness check (`d == s` is stationary
> under both rules by construction). The only tape that separates the supply
> rules is `solv_labour`, in the direction registered before the run.
>
> **One caution the matrix makes unavoidable: a corpse prices well.** The best B8
> median in the whole table (241×, four to twenty orders better than every other
> kernel cell) belongs to `kernel+ratio+reservation`, in which all 72 regions are
> `DEAD` and 66 of 72 bands are unbounded. B8 carries no liveness term by design.
> **B8 is a distance, never a verdict** — the `solv_labour` lesson, now on `lr`.
>
> One audit finding is folded back above: the median-band column of the
> "*Can the price rule lose its α entirely?*" table does not reproduce, and is
> marked superseded in place. Its `live` and `within 2.2×` columns do reproduce,
> so the conclusion that rests on them stands.

> ---
>
> ### P4.7 — the `cr_*` consistency corpus: worlds built to be viable (2026-07-31)
>
> **LANDED.** `tools/gen_regions.py` (generator + auditor + falsification
> selftest), `data/scenarios/cr_00 … cr_10`, receipts
> `results/ab/receipt-cr-corpus.md` (the pre-registered A/B) and
> `receipt-cr-vs-lr.md` (the descriptive comparison and every measurement below).
> **`data/scenarios/lr_*` is untouched** — it is the registered baseline and R7's
> regression suite; this corpus is an addition.
>
> `lr_*` is a factorial search grid built to LOOK FOR a stable region. It never
> asked whether any of its cells was a world that *could* be stable. The new tool
> asks that from the tape, before a tick runs, on **five** conditions: **A1**
> labour demanded at genesis capacity == hours offered; **A2** produced-or-
> imported == consumed-or-exported per (node, good) at the genesis wealth tier;
> **A3** every owner holds the cash Rule 3 requires; **A4** genesis relative
> prices == the technology's Leontief vector (the solver is *imported* from
> `tools/derive_parity.py`, so the generator and the scored benchmark cannot
> disagree); **A5** — added after the first run — whether Rule 1 and Rule 3 can
> both be satisfied by any stock level at all.
>
> **The instrument is calibrated on worlds it did not build.** The three `solv_*`
> tapes whose equilibria were hand-derived *before the engine was pointed at
> them* read exactly `1.000` on all four world conditions. `solv_labour` reads
> **A4 = 2.00** — which is *wrong*, and is the registered counterexample to
> reading a Leontief benchmark as truth; A4 reproduces B8's registered wrong
> answer to the digit. `lr_*` fails four of five on all 24 tapes: A1 **2.15×** to
> **18.1×**, A2 **inf** (services desks draw 50 labour per region to make a good
> the tier-0 basket does not contain), A3 0.178×–24.7×, A4 1.20×/13.3×.
>
> #### RESULT 1 — ten regions are alive and in-band. The standing number was zero.
>
> | | cr/kernel | cr/legacy | lr/kernel | lr/legacy |
> |---|---|---|---|---|
> | passing the full criteria | **10** | 0 | 0 | 0 |
> | alive **and** band ≤ 2.2× | **10** | 0 | **0** | **0** |
> | alive | 23 | 15 | 8 | 33 |
> | median band | 2.86e9× | 3.96e5× | 3.27e15× | 1.52e5× |
>
> All ten are in the consistent worlds; not one is in a defect world. Across every
> configuration ever run on `lr_*` the "alive and in-band" count has been zero,
> and the five regions that ever got inside the band were all dead.
>
> #### RESULT 2 — the controlled half, which is the part that is not confounded
>
> `cr_04`…`cr_08` differ from `cr_00` by exactly one named condition, in the same
> technology, basket, region sizes and criteria file; `--selftest` asserts in both
> directions that the named condition moves and the other four read 1.000. Worst
> band, kernel arm: `cr_00` **1.000×** → A1 broken **2.2e25×**, A2 **2.2e9×**, A3
> **3.9e9×**, A4 **2.4e20×**, all four **1.3e23×**. **A `cr_00`-vs-`lr_00`
> comparison could never have said that**, and building the defect controls was
> the only way to get it.
>
> #### RESULT 3 — there is no basin of attraction; there is a dead band
>
> This is the one that matters most and it is negative. `cr_00` with its genesis
> flour price scaled and its cash re-solved (only A4 moves):
>
> | ×1.00 | ×1.01 | ×1.02 | ×1.05 | ×1.10 | ×1.20 | ×2.00 |
> |---|---|---|---|---|---|---|
> | PASS | PASS | PASS | PASS | 3.4e9× | 1.0e29× | 5.9e26× |
>
> **Inside the passing region B8 reads the displacement back verbatim with
> `sd 0.000` — the wrong price never moves.** ×1.05 is a full-PASS certificate,
> all eight batteries and 3/3 regions, on a world permanently and exactly 5%
> wrong. The edge was derived before it was measured: the mill's relative margin
> `(p_f − 0.5)/((p_f + 0.5)/2)` reaches `dead = 0.05` at **×1.051282**; bisected,
> **×1.0512 PASS, ×1.0514 FAIL**. RESULT 2 of the solvable worlds said the
> mechanism cannot walk to a fixed point one factor of two away. **It cannot walk
> to one 6% away, and inside 5% it does not walk at all.**
>
> #### RESULT 4 — Rule 1 and Rule 3 contradict each other on every channel operator
>
> Found by building a trade world exact on A1–A4 and watching its channels stop
> trading on tick 2. A pass-through desk reads **one** inventory slot two ways:
> Rule 1 posts above `b_out·flow`, Rule 3 buys below `s·flow`, so a stock level
> doing both exists **iff `b_out < s`**. Wheat cleared at the buying node over 12
> ticks, one dial moved: `s=1` **0.00**, `s=2` 18.77, `s=3` 93.32, `s=4` 86.77 —
> exactly where the arithmetic puts it. Inside `cr_01` the **autarkic** region
> passes the full criteria while both **traded** regions die.
>
> **This explains a defect already in the record.** P4.5 traced lr's collapse to
> "flour has zero posted supply from tick 0 — that part is pre-existing" without
> saying why: lr's six `flour_transport` desks open holding no flour, so they post
> nothing and the importing node pins at +1 for ever. lr's window **is** open
> (`s = 4`), so **seeding those desks is a cheap experiment that should work** —
> the one concrete repair this session hands forward.
>
> #### RESULT 5 — `s` is not a free dial, and its two constraints conflict
>
> `cr_09` is `cr_00` at lr's registered `s = 4`: the **worst world in the
> corpus**, 0/3 alive, bands 1.8e42–9.6e43, worse than every named defect.
> Labour cleared over 40 ticks 373.8 against `cr_00`'s 1280.0 — 29% throughput.
> The pre-registered cause was *spoilage* (Rule 3 buys `desired·s`, production
> consumes `desired·1`, `labour` is `Instant`). **Half wrong, and the
> falsification test says so**: with labour's shelf life changed to `Ticks(8)` the
> world still dies and labour cleared *falls* to 329.2. The real mechanism is that
> a non-storable input's supply is a **per-tick flow** while Rule 3 asks for S
> ticks of it in one tick — measured demand at Ashby, ticks 1–4: **40, 0, 48, 40**
> against a flat supply of 32, taking 96 of the 128 hours needed *at genesis*.
>
> **The two `s` findings point opposite ways**: RESULT 4 needs `s > b_out`,
> RESULT 5 needs `s = 1`. With `b_out = 2` **no `s` is admissible** for a world
> containing both a channel and a labour market — which is every `lr_*` tape. A
> design contradiction, not a tuning problem, and `α·b_out` was already the
> Phase 5 axis.
>
> #### Three smaller findings, recorded where they were found
>
> 1. **The tape format cannot express a consistent transport wedge.**
>    `RawSimState.prices` is one price per good written to every node, so a
>    priced transport's zero-profit wedge `p_to = p_from + a_tr·w` is
>    unstateable: every `lr_*` `flour_transport` desk opens at a loss of exactly
>    `−0.1·w`. `cr_01` uses costless channels because that is the only trade
>    structure this format can start consistent.
> 2. **A rentier class has no consistent version.** At the zero-profit vector
>    there is no profit, so the overflow funding it is identically zero; `cr_10`
>    breaks A2 by exactly the rentiers' share and nothing else. A modelling gap
>    Phases 6–7 own, carried by all 24 `lr_*` tapes.
> 3. **Consistency does not help the legacy arm.** Its median band on `cr_*`
>    (3.96e5×) is *worse* than on `lr_*` (1.52e5×), and it cannot hold `cr_00` at
>    all. One exception, and it is the best legacy result in the repository:
>    `cr_02` runs **alive at a band of 3.17×** against the 2.2× bar, B7 clean at
>    median |imbalance| 0.045, flour only **4.56×** too dear — against lr/legacy's
>    209×–2.9e6×. Still a fail.
>
> #### What this does NOT say
>
> **The pre-registered A/B is still a LOSS** (`receipt-cr-corpus.md`): G1 fails on
> four killed regions even though liveness rises 15 → 23, and G2/G3 fail because
> the defect worlds blow up harder under the kernel than under legacy. The rule
> counts liveness first and does not care that the kernel is the only arm that
> produced a passing region. **The Phase 4 gate is not met.**
>
> And every one of the ten passing regions sits at a hand-solved rest point.
> RESULT 3 is the honest weight on RESULT 1: consistency buys the kernel a fixed
> point it can *hold*, and buys it no ability to *find* one. `cr_00` is a
> necessary condition passing, not a sufficient one.
>
> #### What Phase 5 should take from this
>
> 1. Seed lr's transport desks (RESULT 4) — cheap, and predicted to work.
> 2. Sweep `dead` as an axis. RESULT 3 says it is not a chatter-killer, it is the
>    entire width of the region in which the price system does anything at all.
> 3. Resolve the `s` contradiction (RESULT 5) before `s` appears in any sweep as
>    if it were a stability dial.
> 4. `cr_00` is now the reference world the phase map should be swept over: it is
>    the only multi-region tape in the repo that is green, correct, and solved.

> ---
>
> ### P4.8 — the audit pass: what 2026-07-31 actually established (2026-07-31)
>
> **This entry exists because a session that produced a great deal was found, on
> review, to have produced much less than it reported.** Two adversarial passes
> refuted or downgraded most of it. A third pass — this one — re-measured the one
> result that had survived both, from a clean build, and found that half of *that*
> was wrong too. The instruments are `examples/restoring_force.rs` (the sweep, the
> basin scan, the mechanism trace) and `tests/test_15_restoring_force.rs` (seven
> tests, every one asserting a measured number including the ones that contradict
> the record). Nothing below is a summary of an earlier summary; every figure was
> produced by a run made for this entry.
>
> #### The headline, in one paragraph
>
> The kernel can **hold** a hand-derived equilibrium exactly, and — with the
> reservation rule — can **walk back to one** from a displacement of a few
> percent. It cannot walk back from more than about 5%, the set of displacements
> it recovers from is not an interval, the recovery stops short of the registered
> exactness for a reason that is a constant in the source rather than economics,
> and on the corpus of record none of this buys a single live in-band region. The
> project's only correctness criterion turned out not to be one.
>
> #### 1. The correctness battery was a REPORTER, and had to be repaired
>
> **This is the item that must not be smoothed over.** B8 shipped labelled
> `"prices match technology"` and was counted as the project's first correctness
> criterion — the thing PLAN had been asking for since "the missing instrument"
> above. It was not one. Adversarial review found **seven** defects, and the
> repair (`src/certify/technology.rs`, `src/runner.rs::B8_LABEL`) renamed it to
> **`B8 "price/technology gap (report, no bar)"`**. A PASS now means *the distance
> was computable*, and nothing else. The seven, with the repair:
>
> | # | defect | state |
> |---|---|---|
> | 1 | a one-sample reading printed `sd NaN` on a **passing** line (R5) | fixed: `log_sd: Option<f64>`, fail-closed sweep over every statistic |
> | 2 | a region whose numeraire market never traded emitted 851 confident readings | fixed: no numeraire, no reading; the region now FAILS |
> | 3 | one wage error reported as N per-good errors, two of them pointing the wrong way | fixed: common-factor / residual decomposition |
> | 4 | 850 of 851 ticks silently discarded, reported as `n=851` | fixed: census with denominator and disposal |
> | 5 | benchmark assumptions (zero-rent, full-utilisation, single-node) undisclosed | fixed: stated on every line |
> | 6 | single-node valuation drops haulage labour — 1.200× on `lr_00` | measured and disclosed, **not** fixed |
> | 7 | one stalled good marked the whole vector `NotConverged` | fixed: only the goods still moving |
>
> **It carries no bar and the reason is now structural rather than procedural.**
> The old argument was "the corpus numbers were already known, so any bar picked
> now would be fitted" — true, but it implies a bar arrives with better evidence.
> It does not. B8's one hand-checked case is a **known false positive**:
> `solv_labour` sits on a closed-form equilibrium registered before any run, and
> B8 reads it "2.000× too dear" because a zero-rent benchmark cannot see a
> capacity rent. A bar at 2× fails a correct world; a bar above 2× is chosen to
> let a known false positive through. **No threshold repairs a specification
> error.** What would make it a battery is recorded in code: a rent-aware implied
> vector plus per-node valuation.
>
> Three threshold-free fail-closed paths do fire, and one of them fired on the
> corpus for the first time (P4.6 finding 3).
>
> #### 2. The displacement result, re-measured — half of it was wrong
>
> The claim carried forward by `src/kernel/mod.rs`, the design note and the
> summaries: *"a displaced `solv_1g` grain price returns to 1.000 under
> `reservation` while sitting at 1.010 with standard deviation exactly zero under
> `inelastic`."* It was measured at one displacement, in one direction, on one
> tape, under one price rule. All four of those were the problem.
>
> **STANDS, and understated.** Under `inelastic` a +1% displacement does not have
> a small spread; the price series is **bit-constant for all 1000 ticks** —
> `max − min = 0.0` exactly — on `solv_1g`, `solv_chain` and `solv_1g_money`,
> under **both** price rules. Rule 1 names no price, both markets stay balanced,
> every σ stays inside its dead band, and nothing in the kernel moves. Note that
> "sd exactly zero" is a claim about the *estimator*: a two-pass sd of a
> bit-constant series reads ~1e-15. Welford — what `certify::invariants` and B8
> use — returns exactly 0, so the original sentence is true as written, and is
> weaker than the fact.
>
> **STANDS, with a boundary nobody had stated.** The freeze ends exactly where
> `dead = 0.05` puts it: **frozen at ×1.0512, moving at ×1.0513**, against the
> dial-derived `(1+d/2)/(1−d/2) = 1.051282`. That reproduces P4.7's RESULT 3 edge
> (`cr_00`: ×1.0512 PASS, ×1.0514 FAIL) on a different tape by a different
> instrument — the same number from two independent measurements.
>
> **FALSE.** "Returns to 1.000" fails the tape's **own** registered
> `fixed_point_tol_log = 1e-12`. It returns to `|ln dev| = 1.86e-11`, nineteen
> times the gate — and reads **1.862e-11 on all three tapes**, which is the tell:
> it is not a property of any of those economies.
>
> **NOT MEASURED, AND IT IS ONE-SIDED.** Displace **down** 1% under `inelastic`
> and the price does not sit at 0.990. It walks *up*, crosses the target, and
> rests at **+0.87%** on the far side — with the standard deviation over the
> scored window still exactly 0.0, because all the motion is in the transient.
> The mechanism is not Rule 1 and not Rule 2 — it is Rule 3's cash band. Below
> zero profit the desk's revenue (0.495 × 20 = 9.9) is under its outlay (10), its
> balance drains, it rations its own labour purchases, labour demand falls under
> the 10 hours posted, and the wage follows; the rationed desk then produces less
> grain, so grain rises. Split at tick 1000: **`p_grain` 0.4950 → 0.499753** — it
> closes **95.1%** of its own 0.005 displacement — while **`p_labour` 1.0 →
> 0.990754**, a fall of 0.0092, nearly twice as far as the grain price's residual.
> The pop's cash goes 30.00 → 30.61, the mirror of the drain.
> **Almost all of the residual in the ratio is the numeraire**, under a rule whose
> supply elasticity is exactly zero and which therefore has no price-response with
> which to have done any of it. A relative price cannot say which of its two sides
> moved, and here the answer is "the other one"
> (`the_downward_drift_is_the_wage_falling_not_the_displaced_good_failing_to_return`).
>
> The consequence outranks the finding: **`sd = 0` over a post-transient window is
> not evidence of "no restoring force"** — it is evidence of "no motion in the
> window", and this configuration satisfies the second while violating the first.
> `max − min` over the *whole* series is the statistic that separates them.
>
> #### 3. The sweep table (`solv_1g`, kernel arm, 1000 ticks, window 150–1000)
>
> `ends` is the final relative price as a multiple of the registered target;
> `range(all)` is `max − min` of `ln(p/p_labour)` over the whole run — `0` is the
> unit root; `alive` is the fraction of window ticks on which every good with a
> registered `cleared` target actually traded.
>
> | ×factor | inelastic `ends` | range(all) | reservation `ends` | range(all) | alive (res.) |
> |---|---|---|---|---|---|
> | 0.500 | 1.334 | 2.004 | 1.291 | 1.447 | 1.000 |
> | 0.833 | 2.328 | 1.162 | 0.950 | 0.670 | 1.000 |
> | 0.952 | 1.038 | 8.60e−2 | 1.025 | 7.40e−2 | 1.000 |
> | 0.980 | 1.017 | 3.67e−2 | 1.012 | 3.20e−2 | 1.000 |
> | 0.990 | 1.009 | 1.87e−2 | 1.006 | 1.58e−2 | 1.000 |
> | 0.999 | 1.001 | 1.89e−3 | 1.0006 | 1.57e−3 | 1.000 |
> | **1.001** | **1.001000** | **0 (exact)** | **1.000000** | 9.99e−4 | 1.000 |
> | **1.010** | **1.010000** | **0 (exact)** | **1.000000** | 9.95e−3 | 1.000 |
> | **1.020** | **1.020000** | **0 (exact)** | **1.000000** | 1.98e−2 | 1.000 |
> | 1.050 | 1.050000 | 0 (exact) | 1.308 | 0.654 | 1.000 |
> | 1.100 | 2.314 | 1.561 | 1.310 | 0.671 | 1.000 |
> | 1.200 | 1.911 | 1.003 | 1.363 | 0.653 | 1.000 |
> | 1.500 | 2.672 | 1.606 | 1.396 | 0.845 | 1.000 |
> | 2.000 | 1.501 | 2.137 | 3.102 | 1.924 | **0.595** (dead from t=292) |
>
> Read across, the whole result is the three bold rows and their two neighbours.
> Outside `[0.95, 1.05]` **neither rule returns to the target.** `reservation`
> lands *nearer* it in six of the eight outside rows — but it has not converged
> there: its `sd` over the window sits at **0.18–0.19** in every one of them, i.e.
> a ±20% orbit that happens to be centred nearer the answer. That is precisely the
> artefact `mode_b_gap`'s doc comment warns about, arriving as a measurement: a
> proximity statistic and a convergence statistic are different questions.
>
> Under the **`ratio`** price rule the picture is worse and is not a near miss.
> On `solv_1g` under `reservation`, of the sixteen displacements: two overflow f64
> and are reported **`UNCOMPUTABLE`** rather than as numbers (R5), eight have the
> market **dead before the window opens** (`alive 0.000`), five stay alive, and
> the one survivor outside `[0.98, 1.02]` — ×0.9524 — sits at **0.133× of target**
> with a live market throughout. Inside the surviving band the price orbits rather
> than resting: `sd` 2.6e−4 over the window and still 1.7e−4 over its last tenth.
>
> #### 4. The basin — measured, and it is not an interval
>
> Nobody had measured it. `examples/restoring_force.rs` scans 40 geometric points
> per side out to ×8 and ×1/8, takes the **connected component containing the
> fixed point**, and bisects that first crossing in log-factor space. Three
> verdicts are reported side by side rather than one being chosen; the
> threshold-free one is `contracted` = the final `|ln dev|` is smaller than the
> displacement it started from.
>
> | `solv_1g`, imbalance | boundary down | boundary up |
> |---|---|---|
> | `inelastic`, contracted | 0.9493× | **none — no upward displacement contracts at all** |
> | `reservation`, contracted | **0.9359×** | **1.0457×** |
> | `reservation`, within ln(1.10) of target | 0.9359× | 1.0463× |
> | `reservation`, within 1e-12 (registered) | — | — (never, at any displacement) |
>
> So the honest basin is roughly **−6.4% to +4.6%**, and three things qualify it:
>
> 1. **The predicate is not monotone.** Beyond the boundary, ×1.2311, ×1.4389 and
>    ×2.5491 all contract while ×1.05 does not — eleven such islands on the
>    up side under `reservation`, nineteen on the down side. Those are separate
>    attractors, not a wider basin. **A single "basin width" is the wrong kind of
>    sentence for this mechanism**, which is why none is quoted without the
>    component qualifier. `the_set_of_displacements_that_come_back_is_not_an_interval`
>    asserts it.
> 2. **The upward boundary and the `inelastic` dead-band edge are nearly the same
>    number** (1.0457 vs 1.051282). The reservation rule buys recovery *inside a
>    region whose width is still set by `dead`* — which is P4.7 RESULT 3's point
>    arriving from the other direction, and is the sharpest available argument for
>    `dead` being the Phase 5 axis rather than `α·b_out`.
> 3. **Under `inelastic` the upward basin is empty by construction.** Contraction
>    is *exactly* 1.000 everywhere inside the dead band — the displacement neither
>    shrinks nor grows. That is the unit root, stated as a basin.
>
> #### 5. What actually stops the return: an unregistered absolute constant
>
> `src/systems/price_update/mod.rs:28` emits a price delta only when
> `|p_next − p| > 1e-12`. That is an **absolute threshold in currency units with
> behavioural meaning, registered nowhere** — arguably an R2 breach, and
> demonstrably an R12 one. Near the fixed point the per-tick step is
> `p·α·|imbalance|` with `imbalance ≈ −2e` in the log price error, so a run stalls
> at
>
> ```text
>     e ≤ 1e-12 / (2·α·p) = 1e-12 / (2 × 0.05 × 0.5) = 2.0e-11
> ```
>
> against a measured **1.86e-11**. The prediction that makes it a mechanism and
> not a coincidence: `solv_1g_money_2x` is the same real economy at exactly twice
> the price level, so the same algebra halves the residual. Measured **1.8617e-11
> at 1× and 9.8936e-12 at 2×** — ratio 1.882 against a predicted 2.
>
> **A redenomination changed a real outcome.** `money_is_exactly_neutral_between_the_1x_and_2x_tapes`
> cannot see this and is not wrong: it runs both tapes *at* the fixed point, where
> no price drifts and the guard never binds. Neutrality was only ever tested where
> nothing moves. `the_return_stalls_on_an_absolute_constant_…` asserts the
> violation. Making the guard relative is a mechanism change with its own A/B; it
> was not smuggled in beside a measurement.
>
> #### 6. Is `solv_1g` knife-edge? No — and that is not reassuring
>
> The headline transfers exactly: bit-frozen under `inelastic`, back to 1.86e-11
> under `reservation`, on `solv_1g`, `solv_chain` **and** `solv_1g_money`, to the
> same digits. **Nothing else transfers.** Displaced to ×0.9524 under `inelastic`,
> `solv_1g` rests quietly at 1.038× with its markets alive throughout while
> `solv_chain` runs its relative price to **1.6e20** and trades on 6% of the
> window; `solv_1g_money`'s reservation runs die from tick 171 at ×0.5 and from
> tick 332 at ×1.05, where `solv_1g`'s stay alive. The shared row is the +1% row
> and its immediate neighbours. **A summary saying "the family agrees" would be
> false**, and `the_headline_transfers_to_the_other_tapes_and_nothing_else_does`
> asserts both halves.
>
> #### 7. The honest overall position
>
> **Established, and re-verified independently:**
>
> * The shipped `inelastic` Rule 1 has supply elasticity exactly zero and the
>   price loop a literal unit root — a displaced price is bit-constant for 1000
>   ticks. This is the sharpest negative result the project has.
> * The reservation band removes it: three tapes, two price rules, a displacement
>   of 0.1–2% comes back to within 2e-11.
> * The dead band, not the supply rule, sets the width of the region in which
>   anything happens: ×1.051282, derived from `dead` and confirmed twice
>   independently (`solv_1g` here, `cr_00` in P4.7).
> * The kernel holds all five hand-derived equilibria bit-exactly (mode A), which
>   remains the only unambiguous positive in Phase 4.
>
> **Refuted or downgraded this session:**
>
> * B8 is a reporter, not a battery. Renamed. Seven defects, six repaired.
> * "Returns to 1.000 exactly" — false at the tape's own registered exactness; the
>   floor is a numerical guard, and it is level-dependent.
> * "No restoring force under `inelastic`" — true upward, false downward, where a
>   cash-channel drift moves the numeraire instead.
> * The `ε_s` range "0.216 to 45.2" — not reproducible as labelled (second pass).
> * "`α·b_out = 0.2` is a factor of ten inside the stability boundary" — the test
>   that appeared to check it never called the kernel; measured gain reaches 2.28
>   (second pass).
> * "No configuration recovers from a 2× displacement" — one does
>   (`solv_1g`/imbalance/`reservation_goods`, volume trend 1.134×).
>
> **Still unknown, in the order it matters:**
>
> 1. **Whether any of this survives contact with a multi-region world.** Every
>    positive result in Phase 4 lives on a one- or two-good tape with one desk per
>    market. On `lr_*` the count of live, in-band regions is **0 of 72 in all
>    eight configurations**; on `cr_*` ten regions pass, and every one of them
>    sits at a hand-solved rest point it never had to find.
> 2. **Whether the basin's islands are real attractors or an artefact of a
>    1000-tick horizon.** Nothing has been run long enough to tell, and a
>    displacement that "contracts" at tick 1000 may be mid-orbit.
> 3. **What the basin looks like as a function of `dead`.** It is now the obvious
>    Phase 5 axis and it has never been swept.
> 4. **Whether the 1e-12 guard has been distorting every stability measurement in
>    the repo.** It stops price motion below an absolute size, so every band,
>    every `LevelRange`, and every damping statistic taken at a low price level
>    has been measured on a series that was quantised. Unquantified.
> 5. **Whether `ratio` is a price rule at all.** On the solvable worlds it kills
>    the market before the scored window opens at almost every displacement, and
>    two cells overflow f64. It has never had its own write-up.
> 6. Rent-aware reservation pricing; per-node valuation for B8; the well-posed
>    cycle that trips the 256-sweep cap — all three carried forward unfixed with
>    their reasons recorded in code.
>
> **Process note, recorded because it is the actual lesson.** Every refuted claim
> above was refuted the same way: a number was measured under conditions nobody
> wrote down, then quoted as a property of a mechanism. The elasticity range, the
> loop gain, B8's confident readings and the displacement result all failed for
> that one reason. The instruments now print their own conditions
> (`elasticity_probe`'s CONDITIONS line, `restoring_force`'s per-table header,
> B8's benchmark-assumptions preamble), and it should be treated as a standing
> requirement rather than three separate repairs.

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
