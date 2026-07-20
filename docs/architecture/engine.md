# The engine

Runs are experiments; an unverified run is an anecdote. The engine's guarantees
— conservation, determinism, certification — are built into the tick loop, not
into notebooks, and the same design gives parallelism for free.

## The conservation ledger

Every delta that creates or destroys goods or currency carries a provenance
tag:

```
Event | LabourMint | SelfProvision | Production | Minting | Recovery | Spoilage
```

`Inventory::remove` returns the quantity actually removed; shortfalls become
ledger lines, never silent clamps. Per tick, per good, the ledger asserts:

```
Σ inventory[g] = opening[g] + minted[g] − burned[g]
```

A conservation failure panics the run (METHODOLOGY R3). Money leaks, goods
duplication, and phantom consumption are bugs by definition and impossible to
miss.

## Determinism as a tested property

Three golden-hash tests run in CI, not as intentions (METHODOLOGY R4):

- **Repeat**: same (tape, code, seed) twice → identical per-tick state-hash
  sequence.
- **Resume**: checkpoint at tick T, reload, run to T+k → hashes equal the
  uninterrupted run (checkpoints are lossless, lot lives included).
- **Replay**: record the delta stream, apply it to a fresh copy of the genesis
  state with no systems running → identical end hash.

Supporting rules: delta streams are canonically ordered; no unordered
container iteration anywhere on the delta path; floating point stays f64 with
no non-associative parallel reductions (see below); any RNG, if ever admitted,
is seeded per-actor and replayable.

The three tests run at 60 ticks, which proves the equalities hold but says
nothing about whether they survive the horizon the project actually targets. As
of v2 Phase 3 they are joined by `tests/test_08_long_horizon.rs`, which runs
resume-equality over the full 11,700 ticks — the case where inventories carry
thousands of accumulated lots and a checkpoint that silently reordered them
would still round-trip the totals while changing every later float sum.

> **BUILT 2026-07-20.** Resume-equality holds at 11,700 ticks: a run checkpointed
> at 5,850 and resumed reaches the same state hash as one that never stopped.

## The run certificate

Every run ends verdict-first: batteries print PASS/FAIL *before* any result is
read, and the certificate JSON is persisted to `results/` (METHODOLOGY R5).

```
RUN CERTIFICATE  run=…  git=…  tape_sha=…  seed=…
B1 conservation: max drift 0.0e0                    PASS
B2 delta-only mutation (debug audit)                PASS
B3 determinism: state hash 0x…, replay MATCH        PASS
B4 metrics computable (0 NaN)                       PASS
B5 clamp/shortfall events: 0                        PASS
VERDICT: …
```

- Any battery failure → exit nonzero, no verdict read.
- **NaN = FAIL.** A value *declared missing* under a registered convention —
  the μ/r\* staleness flag on quiet ticks ([ownership.md](ownership.md)) — is
  not a NaN and passes: the distinction between a bug and a quiet market is
  itself registered.
- Scenario criteria live in a dated `criteria.ron`: registered thresholds over
  named failure classes (DEAD, UNSTABLE, SWINGING, DESTITUTION), the transient
  window, and the fail-closed NaN policy. Changing a criterion after seeing
  results requires a new dated file (METHODOLOGY R6).

## Parallelism

- State is sharded by region (transport desks shard with their registered
  from-region). Decisions, production, and upkeep run `par_iter` over shards;
  each shard emits deltas; shards **concatenate in fixed region order** before
  a sequential apply. Parallel map, ordered reduce — full speedup with
  bit-identical delta streams.
- Clearing parallelizes per (node, good) trivially.
- Stagger means only 1/S of desks think per tick — a further constant-factor
  win, and the desynchronizer ([kernel.md](kernel.md)).
- The scale arithmetic: 673 regions × ~30 desks × 11,700 weekly ticks is on
  the order of 10⁸–10⁹ kernel activations — minutes on one machine. Compute is
  not the binding constraint; telemetry volume and world authoring are, and
  both are designed for below and in [worldgen.md](worldgen.md).

## Telemetry

> **v2 Phase 2: BUILT 2026-07-20.** The long table, the manifest and the
> DataFrame API ship; the research readouts below do not, and cannot until the
> settlement records and the ownership register exist (Phases 6–7). Two
> deviations from the spec as written were taken deliberately and are marked in
> place rather than edited away.

- The engine writes **Parquet** directly: one tidy long table per run —
  `(tick, region, entity_kind, id, good, metric, value)` — plus a manifest (git
  sha, tape hash, seed, criteria hash, and the dimension tables). Checkpoints
  are sparse and exist for resume, not analysis.

  > **Amended from `(tick, region, entity_kind, id, metric, value)`.** Prices,
  > supplies and inventories are all keyed by *(entity, good)*. Six columns can
  > only express that by folding the good into `id` or into the metric name,
  > and both put structure inside a string, which makes the obvious groupby
  > wrong. `good` is therefore a seventh column, nullable: null means "this
  > observation is not per-good", never "good zero". `region` is nullable for a
  > related reason — a market node can serve several regions, so a price
  > belongs to a node and to no single region.
  >
  > **Manifest extended beyond identity.** The long table holds integer ids, so
  > a manifest carrying only identity would leave the analysis layer parsing
  > `game_data.ron` to say the word "flour" — the gate would have been met on a
  > technicality. The manifest carries goods, recipes, regions, market nodes and
  > channels, and `readers.py` imports no RON parser at all.

- What is emitted today is the *primitives* the analysis layer used to
  reconstruct from checkpoints, plus the per-region series the certification
  stack already computes. Derived quantities that are cheap downstream
  (imbalance, price index) stay derived: a stored second copy is a copy that can
  disagree.

| `entity_kind` | keyed by | metrics |
|---|---|---|
| `market` | node × good | `price`, `price_ema`, `supply`, `demand` |
| `building` | building | `recipe_id`, `recipe_size`, `chosen_size`, `efficiency`, `balance`, `last_margin`, `last_throughput`, `channel_id` |
| `building` / `pop` | entity × good | `inventory` |
| `pop` | pop | `size`, `wealth`, `savings_target`, `is_employed` |
| `region` | region | `velocity`, `employment`, `real_wage`, `concentration`, `bld_util`, `real_income` |

  The `region` rows are read from the same samples the verdict is scored
  against, so the telemetry and the certificate cannot tell two stories about
  one tick. They are only emitted under `--certify`, which is what computes
  them.

- **Research readouts are first-class columns**, computed in-engine from the
  settlement records and the register:

  > **UNBUILT.** `P`, `Q`, the dual GDP and labour-share columns, `R`, `B`,
  > `mu`, `r_star`, `concentration` and `dispersal` all read from structures
  > that do not exist yet — the `SettlementRecord` of [markets.md](markets.md)
  > and the `OwnershipRegister` of [ownership.md](ownership.md). They land with
  > Phases 6 and 7. The long format was chosen so that adding them is adding
  > rows, not migrating a schema. (The `concentration` emitted today is the
  > 07-suite's currency-concentration metric, a different quantity from the
  > claims-weighted one specified here; they will need distinct names when both
  > exist.)

| column | definition |
|---|---|
| `P` | priced share of total pop hours |
| `Q` | enclosed share of total parcel value |
| `gdp_measured` / `gdp_true` | crossing settlements / all provision incl. self-provision at parity |
| `labor_share_measured` / `_true` | wage settlements over each GDP |
| `R` | ln(w_high / w_low) across the labour ladder |
| `B` | Σ positive unfunded opportunity value (the backlog) |
| `mu` | best unfunded yield (staleness-flagged) |
| `r_star` | marginal funded yield (staleness-flagged) |
| `concentration` | top-class share of Σ claims × value |
| `dispersal` | labor content of Capital + Ownership settlements |

- Analysis tooling consumes the Parquet through a tidy-DataFrame API; the
  factorial suite loop (scenario generator → manifest → batch run → per-region
  metrics → persisted verdicts) is the standard experiment shape.

  > **BUILT.** `tools/readers.py` pivots the long table into the frames the
  > notebooks and `tools/app.py` already used; its public API is unchanged.
  > Measured against the RON checkpoint path it replaces, at the suite's own
  > 1000-tick horizon: lr_00 46.16s → 0.129s (357×, 181 MB → 1.1 MB),
  > multi_region 9.34s → 0.102s (91×, 48 MB → 1.1 MB). The full 25-scenario
  > suite loads in 2.28s.

## The 225-year horizon

> **v2 Phase 3: BUILT 2026-07-20.** `data/scenarios/tracer_2r` — two identical,
> uncoupled regions, one produced good, one labour good, one currency — runs
> 11,700 weekly ticks under `--certify` and gets **VERDICT: PASS**, 2/2 regions,
> scored over the full span against `criteria.ron` 2026-07-20 **v2**. The
> thresholds are copied verbatim from the corpus v1 file; only the window is
> extended, so the tracer passes the same bar the corpus fails. Certificate at
> `results/tracer_2r/`.

The tracer is not economics and is not trying to be. It exists to run long
enough that things which are invisible at 1,000 ticks become measurable.

**What it cleared.** Conservation drift does not compound with horizon: it steps
twice, early, then is flat for the remaining 9,000 ticks — driven by the largest
single transaction, not by accumulated error. On `multi_region` the run maximum
is 5.847e-10 from tick ~2,400 onward, holding at 6.3% of tolerance; the tracer's
own is 3.4e-13. Wall clock is a non-issue — 11,700 ticks certified and recorded
in 0.64s — confirming the claim above that compute is not the binding constraint.
Telemetry is 2.0 MB against the ~100 MB gate, a 50× margin.

**A tracer must be alive, or it proves nothing.** The first draft pinned every
good at `alpha: 0.0` and certified PASS. That PASS was hollow: with prices frozen,
the SWINGING detector is being asked whether a constant is settling. The shipped
tracer moves its staple across a ~30× range on *every* tick of the window and
passes on the merits, with first-half standard deviation ten orders of magnitude
clear of any degeneracy guard. `tests/test_08_long_horizon.rs` asserts the
movement, so the degenerate version cannot come back silently.

**What it exposed, for Phase 4 rather than now:**

- **Currency inventories fragment without bound.** In the tracer, `gbp` holds
  exactly 320.0 units at every sample — conservation is perfect — but that value
  is spread across 22 lots at tick 1,000 and **7,674 at tick 11,700**, growing at
  roughly one lot per tick after the transient. The physical economy is
  stationary over the same span (grain: 4 lots, 2.772 units, identical at ticks
  4,000 and 11,700). Lots exist to carry shelf life; an `Indefinite` good has
  none, so for currency the lot structure is pure representational overhead.
  `Inventory::get` sums the vector and runs in the hot loop, so per-tick cost
  grows with it — measured on `multi_region`, marginal cost rises from
  0.025 ms/tick early to 0.218 ms/tick by tick 11,700. Checkpoints go 1.7 KB →
  68.6 KB across one tracer run. Coalescing lots that share a life would bound
  the vector; it changes float summation order and therefore every state hash,
  so it needs a deliberate re-baseline.
- **The price rule has an accidental floor and no ceiling.** `price_next` is
  purely multiplicative with no reference term, and `price_update` suppresses the
  delta when `|next − current| ≤ 1e-12`. That absolute epsilon against a
  multiplicative step freezes any price below ≈ `1e-12/alpha` — predicted 2.0e-11
  at alpha 0.05, measured minimum 1.92e-11 across seven scenarios — while nothing
  bounds the upside: `multi_region` reaches **1.15e+139** by tick 11,700 and still
  certifies B1–B5 clean. The same epsilon guards the EMA. A run in exponential
  free-fall is a broken run, and no battery currently sees it.
- **The events tape cannot express an atomic transfer.** A paired
  `RemoveFromInventory` / `AddToInventory` mints currency whenever the source is
  short: the remove takes what is there and logs a shortfall, the add credits the
  full amount regardless. The ledger catches it and panics, correctly — but the
  idiom is a trap, and it is why `tracer_2r` recycles currency through a recipe
  instead of a tape entry.
- **Telemetry scales linearly and the target world will need subsampling.** At
  the full span, per-region volume is 1.6–2.6 MB, so the 673-region world of the
  scale arithmetic above extrapolates to ~1.5 GB per run. `--telemetry-every`
  is the existing lever and it scales as expected: quarterly (13) gives ~118 MB,
  annual (52) ~31 MB.

**One certifier hole closed.** `damping_ratio` returned 0.0 whenever the first
half of the window was flat, without ever inspecting the second — so a run that
was quiet and then erupted scored as perfectly settled and SWINGING could not
fire. It is now two-sided: flat throughout still scores 0.0, flat-then-moving
scores infinite. All 27 committed certificates are unchanged by the fix, so it
closes a latent hole rather than one the corpus was hitting.
