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
| `market` | node × good | `price`, `supply`, `demand` |
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
