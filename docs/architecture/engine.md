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

- The engine writes **Parquet** directly: one tidy long table per run —
  `(tick, region, entity_kind, id, metric, value)` — plus a manifest (git sha,
  tape hash, seed, criteria hash). Checkpoints are sparse and exist for
  resume, not analysis.
- **Research readouts are first-class columns**, computed in-engine from the
  settlement records and the register:

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
