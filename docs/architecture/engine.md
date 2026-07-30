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

> **v2 Phase 3: GATE NOT MET, 2026-07-20.** `data/scenarios/tracer_2r` runs
> 11,700 ticks under `--certify` and prints **VERDICT: PASS**, and the
> certificate at `results/tracer_2r/` is real. It does not discharge the gate.
> Adversarial review found the PASS is an artifact of where the scoring window
> is placed, over a world whose real economy is frozen for 87% of that window.
> The measurements below stand; the verdict does not. Retained deliberately —
> the scenario is a working long-horizon substrate and the negative result is
> the phase's actual product.
>
> **Superseding an earlier claim in this section.** It first read "BUILT … the
> tracer passes the same bar the corpus fails". That was wrong in the way this
> phase was built to catch, and is left recorded rather than deleted (R14).

The tracer is not economics and is not trying to be. It exists to run long
enough that things which are invisible at 1,000 ticks become measurable.

### Why the gate is not met

**The PASS exists only at the exact window `(150, 11700)`.** Score the same run
over either half alone and it fails:

| window | verdict |
|---|---|
| ticks 150..1600 (the live phase) | FAIL — 8 issues, incl. `SWINGING(velocity damp=9.30)` |
| ticks 1550..11700 (the settled phase) | FAIL — `SWINGING(realwage damp=2.13)` |
| ticks 150..11700 (as registered) | **PASS — 0 issues** |
| ticks 150..46800 (4× the horizon) | FAIL — `UNSTABLE(velocity CV=0.30)`, `SWINGING(velocity damp=19.39)` |

The mechanism is `transient`, which was carried over from the corpus file
unchanged and is **not window-neutral**. At 1,000 ticks a 150-tick transient
discards 15% of the span; at 11,700 it discards 1.3%, leaving roughly 1,400
ticks of the tracer's own unfinished transient *inside* the first scored half.
That inflates `std(first half)`, which is the denominator SWINGING divides by.
Sweeping `transient` with `analysis_end` fixed shows the pass evaporating as the
transient is excluded: 0.582 at 150, 0.703 at 600, 1.005 at 900, 2.13 at 1550.
So "only `analysis_end` changed, therefore the same bar" is true of the literals
and false of the effect.

**The real economy is frozen for most of the scored window.** After tick ~1,550
the tracer clears *exactly* 1.0 grain and 1.8 labour every tick (0 and 2.2e-16
standard deviation), employment takes 2 distinct f64 values (σ 7.9e-17), and the
staple price falls monotonically with **zero** direction changes over the
remaining 10,163 ticks. The detectors spend 87% of the window looking at either a
constant or a one-way deflation ramp. SWINGING's 0.58 measures where a smooth
non-oscillatory trajectory was cut, not damping.

This is the *second* instance of the same error. The first draft pinned every
good at `alpha: 0.0`, certified PASS, and was rejected for asking the SWINGING
detector whether a constant was settling. The shipped version moves its nominal
staple price — and nothing else. The guard written against the first instance
(`tests/test_08_long_horizon.rs`) could not catch the second, because it asked
only whether the staple price changed, which a monotone ramp satisfies. It now
pins the frozen behaviour explicitly instead.

**`alpha` was swept until the scenario passed.** The passing band is
`0.0005 ≤ alpha ≤ 0.0025` jointly on grain and labour; it fails at 1e-5, 1e-4,
2e-4, 3e-4, 0.003, 0.005, 0.01 and 0.05. The registered thresholds were never
touched, so R6's letter holds — but choosing a world parameter against the
scored outcome is the same move R6 exists to prevent, and it belongs stated
beside the verdict rather than only in the scenario file.

**"2/2 regions" carries one region's worth of information.** The two regions are
identical clones with no channel between them and the engine has no RNG, so every
emitted series is bit-identical between them. `2/2` is one trajectory scored
twice. The scenario file's claim that this "proves the mechanism replicates" was
wrong and has been corrected there.

**And it is not dying slowly — it is in a lull.** `velocity` last exceeds the
DEAD threshold of 0.003 at tick 9,673; every one of the final 2,027 ticks is
below it, and the class passes only because the live early phase dilutes the
fraction. Run the same world to 45,000 ticks and it does not fade out: at tick
~22,500 the imbalance flips sign and stays at **+0.697**, and the staple price
hyperinflates from 1.2e-2 to **1.4e+13**. Within the scored window 95.1% of ticks
sit at the pinned deflation value; across the full 45,000 only 47.8% do, and 51%
show *excess demand*. So the registered window is not merely placed where the
statistics happen to pass — it is placed in the quiet stretch between two
crises.

### Why: posted supply is computed from demand

The deflation ramp is not an emergent economic result. It is arithmetic.

`decisions/building_agent.rs:54` caps a producer's posted sell quantity at
`demand * 1.2`. `clearing/mod.rs:59-65` then computes
`imbalance = (demand − supply) / max(demand, supply)`. When the cap binds, that
is `(d − 1.2d)/1.2d = −1/6` **exactly, independent of price**, and
`clearing/mod.rs:54-57` reads it as "too expensive" and marks the price down by
`α/6` — forever. In the tracer this is measured at
`imbalance = −0.166666666667, std exactly 0.0` across 10,100 consecutive ticks,
with the per-tick price ratio matching `1 + α·(−1/6)` to twelve digits. A market
clearing 100% of demand every tick reports 16.7% excess supply, because the
seller is *required* to post 20% more than is wanted.

Two further constants share the construction: `decisions/mod.rs:22` posts
`min(1.05·demand, qty_per_tick)` for magic producers, and the employed/unemployed
pair rule (`decisions/pop_agent.rs:32-36`) posts labour supply as a function of
realized fill. The pair rule's fixed point is `imbalance = f − 1` — strictly
negative under *any* unemployment, price-independent, and in the tracer it
predicts the measured labour imbalance `−0.092271562623196` to all fifteen
printed digits with no fitted parameters.

> **An earlier draft of this section over-claimed and is corrected in place
> (R14).** It said the market has *no* equilibrium and that deflation is
> unconditional. Adversarial review refuted both. The cap is one branch of four —
> `sell_proportion` (`building_agent.rs:37-48`) can zero the posting entirely,
> and a producer whose stock sits *below* the cap posts its stock instead, at
> which point balance is reachable. It is reached in the shipped corpus:
> `big_region`'s wine market holds `supply = demand = 1.0` with imbalance exactly
> 0 and a price constant at 0.1102425 across 9,701 ticks, on unmodified code.
> Nor is the exact `−1/6` general — posting reads the *previous* tick's demand,
> so it requires demand to be constant tick-over-tick, which is a property of the
> tracer's single wealth tier. With K capped sellers the pin is `1/(1.2K) − 1`.
>
> The corrected claim: **whenever a seller is stocked above the cap, the rule
> converts perfect clearing into a permanent negative signal and a geometric
> price ramp, and no price anywhere on that branch reports balance.** That is
> narrower than "no equilibrium" and still a defect — an agent's own decision
> rule is being fed back to it as though it were a market observation.

METHODOLOGY R1 already bans this by name ("Never from price clamps, **demand
caps**…"), R2 bans the unregistered literals, and `PLAN.md` already schedules
"the sell caps and debt gates" for deletion in Phase 4. `kernel.md:35` is
explicit: *"No debt gates, no sell caps: the buffer band is the supply
smoother."* So the design already knew. What this phase adds is the measurement
of what the cap costs, and the observation that the kernel keeps the same
construction for the two cases where there is no stock to absorb the error —
labour (the pair rule) and non-storable services (`scale · last_fill`).

### Before Phase 4 can be scored honestly

**The stability criteria reward the disease.** `Rule::Damping(max_ratio: 0.72)`
is implemented (`certify/verdict.rs`) as `std(second half)/std(first half)` of a
**level** series. On a level series that is a trend detector, not a settling
detector. Measured on synthetic series over the tracer's own window length:

| series | damping | verdict |
|---|---|---|
| stationary, mean-reverting with ripple | 1.009 | **FAIL** |
| stationary with a slow oscillation | 0.971 | **FAIL** |
| monotone geometric deflation (what the tracer does) | 0.149 | **PASS** |
| geometric inflation / recovery | 6.724 | **FAIL** |

A healthy economy that reaches a stationary price with any residual ripple fails;
a dying one that deflates smoothly passes. `PLAN.md` Phase 4 ships the kernel
only if "its pass-rate ≥ legacy's" — so under that bar, **fixing the economics
could lose the A/B**.

> **FIXED 2026-07-31 (v2 Phase 3.5).** `Damping` is superseded by two rules that
> separate the claims it conflated, registered as criteria **generation 3**
> across all 28 scenarios:
>
> | class | rule | asks |
> |---|---|---|
> | `DRIFTING` | `LogDrift(max_factor: 2.0)` | did the level stay put? |
> | `SWINGING` | `ResidualDamping(max_ratio: 2.0)` | did the wobble grow? |
>
> `LogDrift` is the OLS slope of ln(metric) against sample index, as a total
> factor across the window, folded so a 3× fall and a 3× rise both score 3.0.
> `ResidualDamping` removes that fitted trend first, so a series is judged on how
> it moves about its own path rather than on where the path went — which is why
> its threshold sits *above* 1.0: an economy in equilibrium fluctuates
> persistently, and `0.72` had encoded the assumption that a settled economy
> stops moving.
>
> **Both thresholds were fixed from synthetic ground truth before any scenario
> was scored** (`tests/test_09_detectors.rs`): healthy cases reach at most 1.20×
> drift and 1.03 oscillation, sick ones start at 3.0× and 4.2, and 2.0 sits in
> both gaps. That ordering is the point — a threshold chosen to make particular
> scenarios pass is not a threshold (R6) — and the test is the standing record
> of it. `Damping` itself is retained but marked superseded, so certificates
> registered before this date stay reproducible.
>
> **What re-scoring the corpus showed.** Region pass-rate barely moves (4/79 →
> 3/79) and every scenario still fails, so the Phase 1 baseline keeps its shape
> for the A/B. What changes is the *diagnosis*: `SWINGING` drops 39 → 12 while
> `DRIFTING` accounts for 59. The corpus's dominant pathology was never
> oscillation — it is the deflation ramp above, and the old rule was reporting it
> under the wrong name or not at all.
>
> Five regions changed verdict, and they are the validation:
>
> | region | v1 | v3 | measured |
> |---|---|---|---|
> | `lr_18`/Birmingham | passed the fatal classes | `DRIFTING(realwage 2169×)` | real wage moved **1218×** |
> | `lr_06`/Birmingham | **clean** | `DRIFTING(realwage 10.9×)` | real wage moved **12.7×** |
> | `lr_12`/Manchester | `SWINGING(damp 2.09)` | clean | level moved **1.3×** |
> | `lr_12`/Leeds | `SWINGING(damp 1.81)` | clean | level moved 1.3× |
> | `lr_08`/Manchester | `SWINGING(damp 0.74)` | clean | — |
>
> Two regions whose real wage moved by factors of 13 and 1200 were being
> certified as stable; three that merely wandered around a stable level were
> being failed. The independent Python reimplementation in
> `notebooks/07_stability_suite.py` was updated in step and agrees with the
> engine region-for-region, so the cross-check still holds.
>
> The tracer itself now correctly **FAILS** — `DRIFTING(velocity 3.10×)`,
> `DRIFTING(realwage 2.28×)`, with `SWINGING` silent, which is the right
> attribution: it is a ramp, not an oscillation.

**`parity` is undefined.** `kernel.md:53` and `pops.md` make the labour margin
`σ_π = (w_posted − parity)/parity` — the only place in the kernel that compares a
price to a level, and therefore the only thing that could arrest the labour pin.
`grep -rn parity src/ data/` returns **zero hits**, and no doc gives its units. If
`parity` is real (goods-denominated), uniform deflation leaves the ratio
unchanged and the pin survives; if nominal, it anchors the level but trips R12.
The price-level determinacy of the whole kernel rests on which, and the spec does
not say.

### What the horizon did establish

**Conservation drift does not compound on `multi_region`, but does not
generalise.** There it rises in seven discrete steps, all before tick 2,620, and
is then flat at 5.847e-10 (6.3% of tolerance) for the remaining 9,000 ticks. Two
other scenarios are still climbing at the horizon: `supply_chain` 1.7e-12 →
4.2e-11 over 14 steps, `big_region` 4.1e-12 → 3.6e-11 over 17, the last near tick
11,200. So "drift is bounded" is a measured property of one scenario, not of the
engine. The tracer's own maximum is 3.4e-13. What *is* general is that no
scenario approaches its tolerance.

**Compute is not the binding constraint.** 11,700 ticks certified and recorded in
0.64s, confirming the scale arithmetic above.

**Telemetry clears the ~100 MB gate by a wide margin** — 2.21 MB for the tracer
with `--certify --record` (1.62 MB with `--record` alone; region series are only
emitted under `--certify`). That much is unambiguous.

### What it exposed, for Phase 4 rather than now

- **Currency inventories fragment without bound, and faster than linearly.** In
  the tracer, `gbp` holds exactly 320.0 units at every sample — conservation is
  perfect — but that value is spread across **296 lots at tick 2,000 and 7,674 at
  tick 11,700**, at a rate rising from ~0.15 to ~2.2 lots/tick: roughly geometric,
  not the "one lot per tick" an earlier draft of this line claimed. The count is
  also non-monotonic before tick ~1,566 (1,322 lots at tick 500, 22 at tick
  1,000), so early samples are transient rather than baseline. Grain is stationary
  only from tick 1,566 onward, holding 4 lots / 2.772 units to the horizon. Lots
  exist to carry shelf life; an `Indefinite` good has none, so for currency the
  lot structure is pure representational overhead. `Inventory::get` sums the
  vector and runs in the hot loop, so per-tick cost grows with it — on
  `multi_region`, marginal cost rises about **7×** across the run (0.020 ms/tick
  early to 0.14–0.22 ms/tick at the horizon; wall clock on unnamed hardware, so
  the ratio is the claim, not the absolute). Checkpoints go 1.7 KB → 68.6 KB
  across one tracer run. Coalescing lots that share a life would bound the
  vector; it changes float summation order and therefore every state hash, so it
  needs a deliberate re-baseline.
- **The price rule has an accidental floor and no ceiling.** `price_next` is
  purely multiplicative with no reference term, and `price_update` suppresses the
  delta when `|next − current| ≤ 1e-12`. That absolute epsilon against a
  multiplicative step freezes any price below ≈ `1e-12/alpha`. Across all 28
  scenarios at the full span the minimum non-zero price is 1.11e-11 (`lr_16`),
  and 18 of 28 sit below 2e-11 — the two predicted floors, 1e-11 at alpha 0.1 and
  2e-11 at alpha 0.05, bracket the whole corpus. Nothing bounds the upside:
  `multi_region` peaks at **1.147e+139 at tick 6,193** (8.8e+126 by 11,700) and
  still certifies B1–B5 clean throughout. The same epsilon guards the EMA. A run
  in exponential free-fall is a broken run, and no battery currently sees it.
- **The events tape cannot express an atomic transfer.** A paired
  `RemoveFromInventory` / `AddToInventory` mints currency whenever the source is
  short: the remove takes what is there and logs a shortfall, the add credits the
  full amount regardless. The ledger catches it and panics, correctly — but the
  idiom is a trap, and it is why `tracer_2r` recycles currency through a recipe
  instead of a tape entry.
- **The target world will need subsampling, and volume does not scale with row
  count.** At the full span, per-region volume across all 28 scenarios runs
  1.10 MB (`tracer_2r`) to 4.91 MB (`lr_09`), mean 2.93 — so the 673-region world
  of the scale arithmetic above extrapolates to **~2 GB** per run, ~3.3 GB at the
  observed worst case. It is not linear in rows: `lr_00` has *more* rows than
  `multi_region` (3.24 M vs 3.12 M) in a *smaller* file (5.22 vs 7.08 MB),
  because dictionary encoding and ZSTD both do better on a series that has gone
  flat. `--telemetry-every` is the existing lever and scales as expected —
  7.08 → 0.57 → 0.15 MB at every 1 / 13 / 52 — putting the target world at
  roughly 160 MB quarterly, 40 MB annually.

**One certifier hole closed.** `damping_ratio` returned 0.0 whenever the first
half of the window was flat, without ever inspecting the second — so a run that
was quiet and then erupted scored as perfectly settled and SWINGING could not
fire. It is now two-sided: flat throughout still scores 0.0, flat-then-moving
scores infinite. All 27 committed certificates are unchanged by the fix, so it
closes a latent hole rather than one the corpus was hitting.

**Two R2 debts this phase incurred or sharpened, neither yet paid.** The `FLAT`
epsilon in `certify::verdict` is now a pass/fail switch — below it a class always
passes or always fails, with nothing in between — and it is still a code literal
while `max_ratio` beside it lives in `criteria.ron`. And `EMA_ALPHA = 2/53` in
`price_update` is the sole dial governing `price_ema`, which this phase promoted
to a first-class telemetry metric while leaving its coefficient unregistered and
unsweepable. Both belong in data (R2: a constant that cannot be swept cannot be
defended).

**A note on `criteria.ron` versioning.** `Criteria::version` is documented in
`certify::criteria` as tracking the file's *shape*. `tracer_2r` uses it instead
to distinguish a re-registration made on the same calendar day as the corpus v1
file, whose shape is byte-identical from `classes:` down. Nothing in code depends
on the shape meaning, and the manifest's `criteria_sha` makes the applied bar
traceable regardless — but the field now carries two meanings, and a genuine
schema change has no way to announce itself. Worth a `registration` field
separate from `version` before a third file needs one.
