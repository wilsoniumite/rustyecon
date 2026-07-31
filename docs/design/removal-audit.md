# Removal audit — what Phase 4 made dead, wrong, or misleading

Dated **2026-07-31**, against `v2-phase-3` at `a610c73`. Audit only: no code was
changed to produce it. Every claim below was checked against the source, and the
ones that needed a run were run.

**The standing constraint on everything here.** METHODOLOGY R7 leaves one
running engine at all times, and R10 forbids deleting the incumbent because the
challenger is finished. The A/B has not been won — PLAN P4.5 records legacy 33
live regions to the kernel's 8 — so **both arms must stay runnable**. Every item
is marked `SAFE NOW`, `BLOCKED BY THE A/B`, or `NEEDS A RE-BASELINE`, and the
reason is given rather than assumed.

Three classes of trap were found and are called out where they arise:

1. Things PLAN's Phase 4 deletion list names that are **not** actually unused
   (`SetMagicProducerQty`, `MagicProducer`, the `DividendPayout` wiring).
2. Things that look like dead scaffolding but **encode a finding** in a test
   (`Rule::Damping`, the 1.2× sell cap).
3. One live defect in **shared** code that both arms run, found by running it.

---

## 0. The one new measurement this audit made

`src/systems/decisions/mod.rs:27-43` posts magic-producer supply as
`(demand * 1.05).min(qty_per_tick)`. That is a demand-keyed posted quantity —
the exact construction B6 exists to ban — and it sits **outside** the
`match arm`, so it runs under *both* arms. Run this session:

```
$ ./target/release/rustyecon data/scenarios/supply_chain --ticks 200 --certify --agents kernel
B6 own-state posting: 1 violation(s): node0/wheat posted supply moved
   0.000000 -> 1.050000 on foreign volumes alone            FAIL
```

So the claim in `docs/architecture/kernel.md:311` and in
`tests/test_10_invariants.rs:69-76` that B6 "**passes under the kernel**" is
true of `lr_00` and **false of `big_region` and `supply_chain`**, the two corpus
scenarios that carry magic producers. `engine.md:272` already named this rule as
sharing the cap's construction; nothing acted on it. Details in §2.1.

---

## 1. Dead code — nothing calls it

### 1.1 `run_dividends_only` — `src/systems/decisions/building_agent.rs:195-210`
`SAFE NOW.` Zero callers (`grep run_dividends_only` → the definition only). Its
doc comment states "The kernel arm calls this until Rule 3's overflow routing
replaces it (P4.4)". P4.4 landed; `decisions/mod.rs:50-56` now routes overflow
inside the kernel and calls nothing here. The function and its comment are both
dead. Neither arm reaches it, so removing it cannot move the A/B.

### 1.2 `src/testing.rs` (whole file), `src/lib.rs:10-11`, `Cargo.toml:16`
`SAFE NOW.` `testing::MagicProducer` and `testing::MagicConsumer` are referenced
by nothing (`grep 'testing::'` → 0 hits outside the file's own doc comment;
`MagicConsumer` → its own definition only). The `test-utils` feature exists only
to gate this module and is never enabled. It is also actively confusing: a
*second* type called `MagicProducer` that is unrelated to the live
`types::magic_producer::MagicProducer`.

### 1.3 The six unemitted `StateDelta` variants — `src/types/delta.rs`
`SAFE NOW.` Verified by grepping each variant across `src/`, `tests/`,
`examples/`, `data/**/*.ron` and `tools/`:

| variant | line | emitters |
|---|---|---|
| `SetRecipeSize` | 71 | none |
| `SetEfficiency` | 76 | none |
| `SetTransferTarget` | 81 | none |
| `RemoveRecipeInstance` | 87 | none |
| `SetPopSize` | 141 | none |
| `SetPopLabourFillRate` | 144 | none |

That is exactly PLAN's "six now-unemitted delta variants", and PLAN is right.
Two notes:

- **`SetMagicProducerQty` (delta.rs:166) is NOT one of them.** It is emitted
  from the tape at `data/scenarios/big_region/events.ron:16,22` and
  `data/scenarios/supply_chain/events.ron:17-21`, and `tools/app.py:837,874,945,973`
  authors it. A `grep` limited to `src/` makes it look unemitted. Deleting it
  breaks two of the 28 corpus scenarios at load.
- **Deleting `SetPopLabourFillRate` does not delete the state it sets.**
  `PopGroup::last_labour_fill_rate` is still written, directly, by
  `state/apply.rs:176` inside the `RedistributePopPair` arm, and read by both
  `pop_agent.rs:35` and `kernel/mod.rs:321`.

Both arms are indifferent: neither emits any of the six.

### 1.4 Per-building utilisation series — `src/certify/metrics.rs:31-32, 91-92, 181-189`
`SAFE NOW, but read §5.2 first.` `RegionSeries::building_ids` / `building_util`
are appended to every tick for every capacity-control instance and read by
nothing: `RegionSeries::get` (metrics.rs:36-45) does not expose them, and no
`Rule` in `certify::criteria` consumes them. They are the vestige of a
per-building `DEAD_BUILDING` rule the Rust port never implemented — which is
precisely the one place the Rust and Python scorers disagree (§5.2). Deleting
them settles that disagreement in favour of the weaker rule; that may be the
right call, but it is a decision, not a cleanup.

### 1.5 `fn posted` — `examples/elasticity_probe.rs:28-39`
`SAFE NOW.` Hardcodes `AgentArm::Legacy`; superseded by `posted_with` at :41,
which takes the arm and is what `main` calls (:77, :85). Unused.

### 1.6 `RecipeDef::component_reqs` and the component types
`SAFE NOW.` `src/types/recipe.rs:23` is always constructed as `vec![]`
(`scenario/raw.rs:294`) and read nowhere. `ComponentReq` (`recipe.rs:68-72`) and
`ComponentId` (`ids.rs:15`) exist only to type it. `docs/systems/02_production.md:20,209`
already ruled components **CUT** in the 2026-07-18 triage; the ruling was never
executed. Pure code with no tape field, so no scenario changes and no
`tape_sha` churn.

### 1.7 `CountryId` — `src/types/ids.rs:22`
`SAFE NOW.` No uses anywhere.

### 1.8 `OwnerId::Building` and `OwnerId::Government` — `src/types/ids.rs:30,33`
`SAFE NOW.` No order producer emits either: `decisions/mod.rs:39`,
`building_agent.rs:61,175`, `pop_agent.rs:42,111` and `kernel/mod.rs:329,359,493,795`
emit only `RecipeInstance`, `PopGroup` and `MagicProducer`. The handling arms at
`transactions/mod.rs:43,46,200,305` are unreachable. (`Government` is a Phase 7
concept; if it is being held open deliberately, say so in a comment — right now
it reads as live.)

### 1.9 Tape fields parsed and never read — `NEEDS A RE-BASELINE`
These are dead in code but present in **every** `game_data.ron`, so removing
them edits 28 tapes and changes `certificate::tape_sha`
(`certify/certificate.rs:163-174`), which invalidates the identity of every
committed certificate in `results/`. Do it as one deliberate re-baseline with
both arms re-run, not as a cleanup:

| field | declared | read by |
|---|---|---|
| `GoodDef::movement_type` | `types/good.rs:11` | nothing |
| `GoodDef::divisible` | `types/good.rs:13` | nothing |
| `GoodDef::storage_cost_per_tick` | `types/good.rs:14` | nothing |
| `RecipeDef::reversible` | `types/recipe.rs:25` | nothing |
| `ChannelDef::channel_type` | `types/channel.rs:10` | nothing |
| `ChannelState::regulatory_factor` | `types/channel.rs:25` | nothing |
| `ChannelState::occupant` | `types/channel.rs:27` | nothing |
| `RecipeInstance::transfer_target` | `types/recipe_instance.rs:97` | only its own (unemitted) setter, `apply.rs:72` |

The first three are already ruled **CUT** by `docs/systems/01_markets.md:31`
("were never read and are cut"), which is a ruling the code never took delivery
of.

### 1.10 `src/main.rs:13` — default scenario
`SAFE NOW.` `data/scenarios/minimal` does not exist. A bare
`./target/release/rustyecon` fails with "error loading scenario".

---

## 2. Called, and now known wrong

### 2.1 The magic-producer posting rule — `src/systems/decisions/mod.rs:26-43`
`SAFE NOW to fix; do NOT delete the entity.` Two defects in three lines:

- It reads `state.demand(mp.node, mp.good)` to size a posted quantity. That is
  the invariant `docs/architecture/kernel.md:210-214` bans and B6 enforces.
  Measured this session under `--agents kernel` (§0): B6 fails on
  `supply_chain`. It will fail on `big_region` for the same reason.
- `1.05` is an unregistered behavioural constant (R2), the same species as the
  `1.2` sell cap, and `engine.md:272` already says so.

The fix is one line — post `qty_per_tick` flat, which is own-state — and it
touches only `big_region` and `supply_chain`, neither of which is in the lr
corpus the A/B scores. The A/B is untouched.

**Do not delete `MagicProducer` while doing it.** PLAN's Phase 4 deletion list
names it, but `tests/test_04_conservation.rs:148-151` asserts a scenario *has*
one or `Provenance::Magic` is untested, and the two scenarios carrying them are
2 of the 28 that R7 makes the regression suite.

### 2.2 The pop consumption desk's `budget` is inert — `src/kernel/mod.rs:343-350`
`BLOCKED BY THE A/B` (it is live kernel behaviour). As written:

```rust
let reserve = k.b_cash * cost / k.s as f64;
let budget = (cash - reserve).max(0.0);
let affordable = cash.min(f64::INFINITY);
let spend = if cost <= budget { cost } else { cost.min(affordable) };
```

`cost.min(f64::INFINITY)` is `cost`, so the else-branch is `cost.min(cash)` only
by way of the later `ration` clamp — and when `cost <= budget` we already have
`cost <= cash`, so **both branches reduce to the same thing** and `budget` never
restrains anything. Pops spend to zero. `kernel.md:150-155` specifies
`budget = max(cash − reserve, 0)` with buying "cash-capped at budget", and
`rule_3_buy_and_route` (kernel/mod.rs:679-692) *does* implement it for producer
owners. So Rule 3's cash band is half-implemented while the code reads as though
it were whole.

This is not cosmetic: `reserve` is still live at :378 as σ_c's denominator, so
the consumption desk's pressure signal is measured against a band the same desk
does not respect. Fixing it changes the kernel arm's A/B numbers, so it must be
a stated change, re-run and re-reported (R6), not a tidy-up.

### 2.3 The balance ledger runs under both arms — `src/systems/transactions/mod.rs:400-406`
`BLOCKED BY THE A/B.` `new_balance = (old_balance + revenue).min(0.0) - buy_cost`
maintains `CapacityControlState::balance`, which only `building_agent.rs:37-48,68`
reads. Under `--agents kernel` this is dead work that still pushes
`AdjustInstanceBalance` deltas into the hashed, replayed stream.

Two documentation defects sit on top of it:
- The `.min(0.0)` **is** the forgiveness: accumulated profit is discarded every
  tick, so `balance` can only ever be ≤ 0. It is a loss counter, not a P&L.
- `types/recipe_instance.rs:11-13` says "Tracks accumulated revenue minus input
  costs. Decays by 0.95 each tick." Both halves are wrong: the `.min(0.0)`
  above, and the decay is `-0.02` (2%) at `building_agent.rs:68`, not 5%.

### 2.4 The 1.2× sell cap — `src/systems/decisions/building_agent.rs:50-55`
`BLOCKED BY THE A/B, and load-bearing for two tests.` Settled wrong
(engine.md, "Why: posted supply is computed from demand"; B6 and B7 both fire on
it). It nonetheless cannot go yet, for three separate reasons:

1. It is the incumbent arm's behaviour and the A/B has not been won (R10; PLAN
   states explicitly that P4.7's deletions are unavailable).
2. `tests/test_10_invariants.rs:53-66` **requires B6 to fail here** — that is
   the falsification test that makes B6 a guard rather than a decoration.
3. `tests/test_10_invariants.rs:79-99` requires B7 to reproduce
   `-0.166667 … unbroken`, which is `(d − 1.2d)/1.2d` and nothing else.
   `tests/test_08_long_horizon.rs:57-68` excludes B6/B7 for the same reason.

If it is ever removed, those two tests need a replacement defect to fire on
first, or the batteries become guards that cannot fail.

### 2.5 The legacy PD capacity controller — `src/systems/decisions/building_agent.rs:89-125`
`BLOCKED BY THE A/B.` A second, independent B6 violation lives here: :92-108
reads `state.supply` and `state.demand` to size the capacity target. Nine
unregistered literals ride along (§4).

### 2.6 The pop wealth PD controller — `src/systems/decisions/pop_agent.rs:118-153`
`BLOCKED BY THE A/B.` Already ruled **CUT** by `docs/systems/03_pops.md:157`
("the PD controller dies"). Six unregistered constants at :12-23.

### 2.7 `is_frozen`'s doc comment claims coverage that does not exist — `src/kernel/mod.rs:169-174`
`SAFE NOW to correct.` The comment reads "this branch is exercised only by unit
tests, and says so rather than implying coverage." There is no such test:
`grep -rn 'is_frozen\|AlwaysRun' tests/` returns nothing, and no scenario
registers `AlwaysRun` (`grep -ho 'strategy: *[A-Za-z]*' data/scenarios/*/game_data.ron`
→ 1 `CapacityControl`, 26 `DividendPayout`, 0 `AlwaysRun`). So
`StrategyKind::AlwaysRun`, `StrategyState::AlwaysRun`, `is_frozen`, and
`building_agent.rs:186-190` have **zero coverage from any source**. Either write
the test or correct the comment; a comment asserting coverage that does not
exist is worse than silence, and this house style says so.

### 2.8 `efficiency` is a frozen multiplier — `src/systems/production/mod.rs:43-46,107`
`BLOCKED BY THE A/B` (it multiplies legacy output). Read every tick, but
`SetEfficiency` is emitted by nothing (§1.3), so it holds whatever the tape set
— 1.0 in every scenario — for the whole run. The decay that would have moved it
is ruled **CUT** at `docs/systems/02_production.md:304`.

### 2.9 `price_ema`'s doc is aspirational — `src/state/sim_state.rs:26-29`
`DO NOT REMOVE.` "Decisions and metrics that need medium-term price signals
should read this rather than the raw posted price" — no decision reads it. Only
`certify/nan.rs:30` and `output/telemetry.rs:160` do. It is still the write-only
state Phase 3 found. **Keep it**: PLAN's own named next step ("Rule 1's posted
quantity as a function of the posted price against the desk's own `price_ema`")
makes it the first real consumer. Correct the comment, not the field.

### 2.10 Unreachable dividend fallback — `src/systems/decisions/building_agent.rs:241-244`
`BLOCKED BY THE A/B` (it is inside a legacy path). The `_ => 26.0` arm cannot be
reached: `raw.rs:367-376` builds `StrategyState` *from* `RecipeDef::strategy`, so
an instance whose `strategy_state` is `DividendPayout` always has a recipe whose
`strategy` is `DividendPayout { .. }`. The `26.0` is a code default for a
registered dial that can never be missing — an R2 violation with no live path.

### 2.11 Cosmetic, but user-facing — `src/scenario/raw.rs:412`
The loader's parity error contains 18 literal spaces mid-sentence:
`"…registers no parity — see                  tools/derive_parity.py"`.

---

## 3. Documentation refuted by measurement, needing a mark in place (R14)

**No deletions are proposed here.** R14 says superseded text is marked with its
verdict in place; every item below is a missing marker.

### `docs/architecture/kernel.md`

| line | what it says | what refutes it |
|---|---|---|
| 53-56 | Rule 1 for services posts `scale · last_fill`, floored at `ε·scale` | Not implemented. `src/kernel/mod.rs:470-477` deliberately posts `flow.min(stock)` because the same document's price-formation invariant forbids fill-keyed posting and B6 now enforces it. The contradiction is discussed at `kernel/mod.rs:69-78`; Rule 1's text was never marked. |
| 131-139 | the σ-by-desk-kind table **restates** `(band_out − inventory_out)/band_out` for producer and transport | It sits *after* "Producer σ, corrected". The supersession note at 77-108 is above the correction, so a reader arriving at line 133 sees the retired formula presented as current. Needs its own in-place mark. |
| 227-235 | "Storables — why Rule 1 satisfies it": inventory integrates the flow error, `σ = (band_out − inventory_out)/band_out` runs negative past `dead`, Rule 2 nudges until `s = d` | That σ is the one proved pinned at `−1/b_out`. The conclusion may survive under the corrected σ; the stated mechanism is the retired one. |
| 241-244 | flow-cases table: services post `scale · last_fill`, fixed point `√(d/scale) − 1` | Same as 53-56; the quoted fixed point is not the engine's. |
| 45-57 | the parameter table lists seven dials | `KernelParams` (`src/state/game_data.rs:29-68`) registers **ten**. `epsilon`, `phi` and `fill_alpha` are absent from the table; `alpha` is listed but is a per-good *market* dial, not a kernel one. The "default" column also publishes values R2 says live only in data. |
| — | nothing anywhere | **The central Phase 4 finding is not in the spec.** Posted supply has instantaneous price elasticity exactly zero for every good under both arms; labour is supply 0.41 against demand 73.7 with both elasticities zero, so no price clears it. `examples/elasticity_probe.rs` measures it and only `PLAN.md` records it. The document that specifies how prices form should carry it. |
| 311 | "It **passes under the kernel**" (B6) | True of `lr_00`; false of `big_region` and `supply_chain` — see §0. |

### `docs/architecture/markets.md`

| line | what it says | what refutes it |
|---|---|---|
| 5-9 | "Orders carry quantities only — no limit prices; reservation behaviour lives in the kernel" | This is exactly the design choice the elasticity probe shows leaves `s(p)` and `d(p)` non-crossing. PLAN's "Not schedules — a price-responsive Rule 1" section is the current reading; markets.md has no marker. |
| 26-35 | the price update, with `alpha` as a per-good speed dial and "stability comes from the kernel's structure … never from price surgery" | Three things are now known and unrecorded here: α is a stabiliser standing in for a demand elasticity the model lacks; the `max(d,s)` normaliser destroys the magnitude the price needs (`systems/clearing/mod.rs:86-91` records this, markets.md does not); and a **second registered price rule exists** (`--price-rule ratio`, `clearing/mod.rs:97-133`). markets.md still describes one rule where the engine has two. |

### `docs/architecture/pops.md`

| line | what it says | what refutes it |
|---|---|---|
| 31-33 | "posted hours are minted at posting with provenance `LabourMint`" | `Provenance::LabourMint` (`types/provenance.rs:33`) is **never emitted**. `transactions/mod.rs:327-340` deliberately emits no removal for a labour seller at all. `provenance.rs:17-18`'s own list of unemitted tags (`SelfProvision`, `Minting`, `Recovery`) is therefore also incomplete. |
| 51-54 | "Hours not offered (`1 − π`) produce subsistence goods directly into the pop's own inventory … provenance `SelfProvision`" | Not implemented. π *is* implemented (`kernel/mod.rs:396-416`); its counterpart is not, so withheld hours simply vanish. That matters because `parity` is denominated in exactly what those hours would have produced. |
| 80 | "Buys are budget-capped in Rule 3's single per-inventory pass" | Pops are not in that pass. `rule_3_buy_and_route` (`kernel/mod.rs:650-657`) iterates inventories whose residents are recipe instances; the consumption desk buys inside `pop_desks`, and its budget is inert (§2.2). |

### `docs/architecture/engine.md`

| line | what it says | what refutes it |
|---|---|---|
| 384-391 | "**`parity` is undefined.** … `grep -rn parity src/ data/` returns **zero hits**" | False since P4.3. `parity` is a field on `PopGroup` (`types/pop_group.rs:61`), required by the loader (`scenario/raw.rs:410-421`), registered per pop in every lr `starting_state.ron`, consumed by `kernel/mod.rs:396-416`, and derived by `tools/derive_parity.py`. Needs a "RESOLVED 2026-07-31" mark. |
| 58-66 | the certificate specimen prints B1–B5 and labels B2 "(debug audit)" | The engine emits B1–B7 and labels B2 "(replay audit)" (`runner.rs:325-397`). |
| 74 | registered classes are "(DEAD, UNSTABLE, SWINGING, DESTITUTION)" | The registered set is DEAD, DEAD_EMPLOYMENT, UNSTABLE, DRIFTING, SWINGING, CURRENCY_DRAIN, POP_DESTITUTION, DEAD_BUILDING. |
| 464-471 | "Two R2 debts … neither yet paid" (FLAT, EMA_ALPHA) | Still unpaid, but the list is now materially incomplete — see §4. In particular, `FLAT` guards a rule **no `criteria.ron` in the repo registers**, and the price-delta epsilon that engine.md itself measured as an accidental price floor (430-439) is not on the list at all. |

### `docs/systems/*` (v1 records under triage headers)

| line | what it says | what refutes it |
|---|---|---|
| `01_markets.md:112` | price rule ruled "**v2: KEEP** — implemented verbatim in v1 and still the law" | The α sweep, the `ratio` A/B and the elasticity probe all bear on that ruling. Whatever the answer, "still the law" is no longer supportable unexamined. |
| `01_markets.md:31` | `movement_type` / `divisible` / `storage_cost_per_tick` "were never read and are cut" | Still present in `GoodDef` and in every tape (§1.9). Mark the ruling outstanding rather than done. |

### `docs/PLAN.md:146-150` — the Phase 4 deletion list
Two corrections this audit found, both worth writing into the list itself:
- "the six now-unemitted delta variants" is **correct** (§1.3), but a reader
  greping only `src/` will wrongly add `SetMagicProducerQty` to it.
- "`MagicProducer` (a frozen desk)" cannot be deleted without a replacement for
  `Provenance::Magic`'s only test and for two of the 28 corpus scenarios (§2.1).

### `results/*/certificate.json`
All 28 committed certificates record `"agents": "legacy"`. The engine now writes
`"legacy+imbalance"` (`runner.rs:143`). `tools/ab.py` does not read the field, so
nothing is broken — but the committed evidence is one revision stale.

---

## 4. R2 debt: unregistered behavioural constants

PLAN records three (`FLAT`/`FLAT_REL`, `EMA_ALPHA`, `LABOUR_RATE`). The complete
list, with whether each has a live consumer and under which arm:

| constant | site | live consumer | arm |
|---|---|---|---|
| `FLAT = 1e-12` | `certify/verdict.rs:157` | **none in production.** Its only reader is `damping_ratio`, and no `criteria.ron` in the repo registers `Rule::Damping` (checked: `grep 'rule: Damping(' data/scenarios/*/criteria.ron` → no matches). Reached only from `tests/test_06_criteria.rs:161` and `test_09_detectors.rs:175`. | scoring |
| `FLAT_REL = 1e-9` | `certify/verdict.rs:198` | `residual_damping`, i.e. SWINGING on all 28 | scoring |
| `EMA_ALPHA = 2/53` | `systems/price_update/mod.rs:6` | `price_ema`, emitted as telemetry | both |
| price-delta epsilon `1e-12` | `systems/price_update/mod.rs:28, 32` | **the accidental price floor `engine.md:430-439` measured** — an absolute epsilon against a multiplicative step, freezing any price below ≈`1e-12/α`; 18 of 28 scenarios sit inside the predicted band. Behavioural, not an optimisation, and **not on PLAN's list**. | both |
| `1.05` magic-producer markup | `systems/decisions/mod.rs:30` | both arms; also a B6 violation (§0) | both |
| `PIN_SPREAD 1e-12`, `PIN_LEVEL 1e-9`, `PIN_MIN_SAMPLES 32`, `PIN_RUN_FRACTION 0.5` | `certify/invariants.rs:27-50` | B7, every certified run | both |
| `REL_FLOW 1e-12`, `REL_STOCK 1e-11`, `ABS_TOLERANCE 1e-12` | `certify/ledger.rs:117-131` | B1's tolerance — **this one panics the run**, which makes it the single most consequential unregistered number in the repo | both |
| `REPLAY_CHECK_EVERY 64`, `OWN_STATE_CHECK_EVERY 64` | `runner.rs:62, 71` | instrumentation cadence; `runner.rs:66-70` argues it is not behavioural, and that argument holds | both |
| `LABOUR_RATE 1.0` | `decisions/pop_agent.rs:9` | legacy labour posting | legacy |
| `MAX_DRIFT 3.0`, `KP 0.03`, `KD 0.2`, `SPEND_ALPHA 0.3`, `BALANCE_ALPHA 0.02`, `MAX_SUB_SHIFT 0.05` | `decisions/pop_agent.rs:12-23` | legacy pop desk | legacy |
| `KP 0.6`, `KD 0.2` | `decisions/building_agent.rs:118-119` | legacy PD controller | legacy |
| sell cap `1.2` | `decisions/building_agent.rs:54` | legacy | legacy |
| balance discount `-0.02`; recovery band `0.9` / `0.1` | `decisions/building_agent.rs:68, 40-47` | legacy | legacy |
| margin gate `0.05`; share ramp `1.1` / `2.2`; headroom `+0.05`; blends `0.2/0.8`, `0.1/0.9` | `decisions/building_agent.rs:89-124` | legacy | legacy |
| dividend EMA `0.9/0.1`; fallback `reserve_multiple = 26.0` | `decisions/building_agent.rs:238, 244` | legacy; the `26.0` arm is unreachable (§2.10) | legacy |

The scoring and engine dials (`FLAT_REL`, `EMA_ALPHA`, the price epsilon, B7's
four, the ledger's three) are the payable ones — they have live consumers under
**both** arms, so registering them is arm-neutral and does not touch the A/B.
Everything marked `legacy` dies with the legacy layer if the kernel ever wins,
so paying that debt now is work the A/B may discard.

---

## 5. Scorer and test drift

### 5.1 `notebooks/07_stability_suite.py:200-215` — the retired `damping_ratio` now contradicts Rust
`SAFE NOW.` Python returns `0.0` whenever the first half is flat. Rust
(`certify/verdict.rs:186-188`) returns `INFINITY` when the first half is flat and
the second is not — which is exactly the hole `engine.md:457-462` records
closing, and which `tests/test_06_criteria.rs:139-192` and
`verdict.rs:590-600` exist to keep closed. The Python copy is unreachable from
`detect_issues`, so it is dead — but it reads as a reference implementation and
is wrong. Delete it or fix it; do not leave it.

### 5.2 `DEAD_BUILDING` is a different rule in the two scorers — the one substantive drift
`NOT SAFE to "fix" silently.` Python (`07_stability_suite.py:421-429`) evaluates
it **per building**, `bld_df.groupby("building_id")`. Rust registers it on
`Metric::BldUtil`, which `certify/metrics.rs:201-205` computes as the
**regional mean** over instances. A region with one idle plant among three fires
in Python and not in Rust. Rust *collects* the per-building series it would need
(`metrics.rs:181-189`) and never scores it (§1.4).

This matters to a number already in PLAN: P4.4 reports `DEAD_BUILDING 27 → 0`,
which is a count of *regions* under the weaker rule. Changing the rule is a
criteria change and needs a new dated generation (R6) — it must not be folded in
as a bug fix. Everything else agrees: DRIFTING and SWINGING cross-check exactly
(engine.md:366 records 78+6=84 and 18+1=19), and `frac_below`, `rolling_cv`,
`level_range`, `residual_damping`, `linear_slope` and `MeanAndSlope` all match on
inspection, including the ddof=1 and numpy-linear-percentile conventions.

### 5.3 The Python suite can only score one of four cells — `tools/scenario.py:88-131`
`run_simulation` has no `agents` or `price_rule` parameter, so
`notebooks/07_stability_suite.py` can only ever run **legacy+imbalance**. It is
presented as the corpus scorer and is now one cell of a 2×2. `tools/ab.py` reads
persisted certificates instead and is unaffected, so this is a gap in the
exploratory path, not in the evidence path — but the suite's own header says
"Runs all 24 lr_XX scenarios plus the multi_region baseline", which now reads as
more than it is.

### 5.4 `tools/scenario.py:16` — debug binary
`BINARY = target/debug/rustyecon`, and `07_stability_suite.py:76-78` runs
`cargo build` (debug) then passes `build=False`. The project's documented build
is `--release`. Not wrong, but it means the suite runs an order of magnitude
slower than the A/B path for the same work.

### 5.5 `tests/test_10_invariants.rs:69-76` asserts something narrower than it reads
`b6_passes_under_the_kernel_because_every_posting_names_only_own_state` runs
`lr_00` only. Given §0 it is a scenario-specific result stated as a general one.
The file's own opening argument — a guard that cannot fail is not a guard — says
the right move is to extend it across the corpus, where it fails today on
`big_region` and `supply_chain`, and fix §2.1.

---

## 6. Looks removable; is load-bearing

Listed so a later pass does not have to rediscover them.

| thing | why it must stay |
|---|---|
| `certify::verdict::damping_ratio` + `Rule::Damping` (`criteria.rs:62`) | No `criteria.ron` registers the rule, so it looks unused. `tests/test_09_detectors.rs:175-176` uses it as the **negative control** that fixed the Phase 3.5 thresholds — "the old rule passed the collapse" / "failed the healthy economy". That test is the standing record R6 relies on. `tests/test_06_criteria.rs:139-192` also uses the rule to prove an infinite reading survives the JSON boundary (re-pointable at `ResidualDamping`, but currently load-bearing). |
| `types::magic_producer::MagicProducer`, `SetMagicProducerQty`, `decisions/mod.rs:27-43` | `tests/test_04_conservation.rs:148-151`, 2 of the 28 corpus scenarios, `tools/app.py`. Fix the rule (§2.1); do not delete the entity. |
| `building_agent.rs` sell cap, PD controller, balance ledger; `pop_agent.rs` entire | The incumbent A/B arm (R7); and the sell cap is the known defect two batteries are falsified against (§2.4). |
| `StrategyState::DividendPayout` and its per-scenario wiring | `kernel/mod.rs:620-627` `claim_holders` **reads** it to find where Rule 3's overflow routes. Deleting it removes the *kernel* arm's overflow routing until Phase 6's `OwnershipRegister` exists. PLAN lists it for deletion; it cannot go first. |
| `RecipeInstance::last_fill`, `PopGroup::participation`, `PopGroup::parity` | Kernel-arm state variables; all three live. |
| `SimState::price_ema` | Write-only today, but the input to the named next step (§2.9). |
| `PopGroup::last_labour_fill_rate` | Read by both arms (`pop_agent.rs:35`, `kernel/mod.rs:321`), even though its delta variant is dead (§1.3). |

---

## 7. Suggested order

1. **§2.1 first** — it is a live B6 failure in shared code, one line, and it does
   not touch the lr corpus or the A/B.
2. **§1.1, §1.2, §1.3, §1.5, §1.6, §1.7, §1.8, §1.10, §5.1** — arm-neutral dead
   code; no scenario file changes, no `tape_sha` churn, no A/B effect.
3. **§3** — the R14 markers. Cheap, and they stop the next session
   re-deriving what Phase 4 already measured.
4. **§4's both-arms rows** — the payable R2 debt. Registering the ledger
   tolerance and the price epsilon is arm-neutral.
5. **§2.2** — a real kernel behaviour change; state the rule, re-run both arms,
   report the A/B with and without (the R6 shape P4.3 used for genesis scale).
6. **§5.2** — a criteria change; new dated generation, or an explicit decision to
   keep the regional-mean rule and delete the unused per-building series (§1.4).
7. **§1.9** — the tape-field re-baseline, as one deliberate act.
8. **Everything marked BLOCKED** — only after the A/B has a winner. PLAN already
   says P4.7's deletions are unavailable, and this audit found nothing that
   changes that.
