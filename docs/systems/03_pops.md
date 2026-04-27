# Pops

## Purpose

Pop groups are the simulation's population model. They are the labour supply,
the source of consumer demand, and the human consequence of economic change.
Their spending drives prices; their wages drive production costs; their savings
drive investment. The primary research question — what happens to welfare under
high VAT and UBI — is ultimately answered by observing what pops can afford.

---

## Scope

This system covers:
- Pop groups as aggregate entities
- Inventory and the liquid pool available for spending
- Need tiers: how pops decide what they want and at what quantity
- Buying: substitution within tiers, minimum demand
- Savings and the wealth signal
- Wealth drift: how W changes over time
- Labour supply to buildings

Explicitly deferred to other systems or future design:
- Pop growth, mortality, and migration (Pop Evolution — stub)
- Job switching and skill accumulation (Pop Evolution)
- Political pressure and radicalisation (Politics — seam only)
- The exact calibration of savings fractions and deposit fractions
  (calibration, not design)

---

## Pop Groups

A pop group is an aggregate of people sharing a job category, cultural identity,
and region. It is not a collection of individuals. Its size is an `f64` — smooth
dynamics, no integer discontinuities. Pop groups are the atomic unit of the
population model: they employ at buildings, they buy goods, and they save.

A pop group is not the same as a wealth tier. Wealth `W` is a continuous signal
per pop group that changes over time. A group of factory workers in Manchester
in 1840 starts at some `W` and drifts based on their wages and the cost of
living. Two factory worker groups in different regions with different wages will
have different `W` values even if they share a job category.

---

## Inventory and the Liquid Pool

A pop group holds an inventory containing currency. Goods purchased for
consumption are treated as immediately consumed — they do not persist in
inventory across ticks. This is accurate for non-durable goods (food, energy,
services) and is adopted as a simplification for durables (clothing, household
goods) by treating durable spending as a recurring maintenance and renewal cost
rather than a one-time purchase.

At the start of each tick, after wages, dividends, and maturing deposits have
been applied to inventory, the pop group's **liquid pool** is:

```
L = cash_in_inventory + matured_deposits + wages + dividends
```

Matured deposits are 1-tick credit instruments that expired this tick and
returned to currency. Wages and dividends were received this tick through
transactions. The liquid pool is the total available for spending this tick.
Nothing else — not longer-duration credit, not investment company shares — is
liquid on this timescale and therefore does not enter L.

---

## Need Tiers

Each pop group has a set of active **need tiers** determined by its current
wealth level `W`. A need tier is a specific consumption target that unlocks at
a wealth threshold and does not cross wealth thresholds for substitution
purposes.

Each tier specifies:
- **Unlock threshold**: the wealth level at which this tier becomes active
- **Goods in this tier**: which goods can satisfy it (may be one or several)
- **Upper target**: the satisfying quantity at each wealth level (interpolated
  continuously between discrete config points)
- **Lower target**: the floor quantity below which the pop is genuinely deprived
- **Preference weights**: per-good multipliers within this tier that persist
  regardless of price

Goods within the same tier substitute for each other on price. Goods in
different tiers do not substitute. A wealthy pop consuming both basic
intoxicants and luxury intoxicants is buying them with separate budgets from
separate tiers — a price drop in liquor does not reduce their wine budget.

**Minimum demand**: for any good whose recipe exists somewhere in the world
(globally known), all pop groups post a small minimum buy order if
that good is in their active tier.
This creates a latent demand signal that prevents nascent industries
from having zero price signals. The minimum quantity is very small — enough to
create a nonzero price, not enough to sustain an industry.

---

## Buying: Decisions and Substitution

In Phase 1, the pop group posts buy orders based on current targets, limited by
the liquid pool `L`.

**Budget allocation**: the liquid pool is allocated across active tiers by
weights derived from the current wealth level (interpolated from config). A
poor pop allocates most of `L` to food and energy tiers; a rich pop allocates
less to basic tiers and more to luxury and financial tiers.

For each tier, the budget determines how much currency is available for goods
in that tier. The buy order for each good within a tier is:

```
ordered_qty[i] = min(current_target[i], tier_budget × share[i] / price_last_tick[i])
```

The `min` prevents buying beyond the current target even when budget is
abundant — surplus tier budget goes to savings. `share[i]` is the within-tier
allocation share for good `i`, which drifts toward the price-optimal allocation.

**Within-tier substitution**: shares drift toward the cheapest price per unit of
satisfaction. For good `i` in a tier where each unit provides
`satisfaction_per_unit[i]` of the tier target:

```
effective_cost[i] = price_last_tick[i] / (pref_weight[i] × satisfaction_per_unit[i])
ideal_share[i] ∝ 1 / effective_cost[i]   (normalised, then clamped to [min_share, max_share])
```

Actual shares drift toward `ideal_share` at a maximum rate of 10% of total
tier budget per tick, preventing abrupt switches. Preference weights and clamp
bounds are configured per good per tier. Cultural and religious modifiers adjust
preference weights via laws and scripted events.

Prices appear only to compute quantities from spend and to determine the ideal
share ordering. The pop does not compare prices across tiers.

---

## Savings and the Wealth Signal

After market clearing, the pop knows `actual_spend`:

```
savings = L − actual_spend
```

Savings are split between cash reserve and new deposits:

```
new_deposits = savings × deposit_fraction(W, laws)
cash_reserve  = savings × (1 − deposit_fraction)
```

`deposit_fraction` rises with wealth and is modified by laws (financial sector
access, deposit interest rates). In Era 1, subsistence and low-wealth pops
typically hold all savings as cash; middle and upper pops access bank deposits.

Deposits are 1-tick credit instruments. They appear in the pop's buy order for
that instrument and mature next tick, returning currency to the liquid pool.
They are not a consumption category — they are a savings vehicle. Longer-
duration credit instruments and investment company shares are purchased as
consumption at higher wealth tiers (financial tier) but do not count as liquid
savings.

**The wealth signal** compares total savings to a target buffer:

```
savings_target(W) = savings_fraction(W) × last_tick_spend
```

where `savings_fraction(W)` rises from ~10% at low wealth to ~70% at high
wealth. A low-wealth pop has enough savings signal when it holds one week's food
budget in reserve; a high-wealth pop needs several months of lifestyle spend.

```
savings_ratio = savings / savings_target(W)
drift_signal  = savings_ratio − 1.0
```

Positive: flush, wealth pressure upward. Negative: tight, pressure downward.
The magnitude of `drift_signal` scales with how far savings are from target —
further from target means faster wealth adjustment.

---

## Wealth Drift

`W` is a continuous value drifting based on the savings signal.

### Going up (drift_signal > 0)

```
W ← W + α × drift_signal
current_target[i] ← upper_bound(W, i)   for all i
```

As `W` rises, active tiers may unlock and targets jump to the aspirational
upper bound at the new wealth level. The pop immediately tries to reach new
higher targets — which typically depresses savings temporarily until income
catches up.

### Going down (drift_signal < 0)

First, trim above-lower-bound targets uniformly before reducing `W`:

```
for each good i where current_target[i] > lower_bound(W, i):
    excess = current_target[i] − lower_bound(W, i)
    current_target[i] ← current_target[i] − trim_rate × |drift_signal| × excess
```

Targets cannot be trimmed below `lower_bound(W, i)`. If all targets are already
at their lower bounds and `drift_signal` is still negative, `W` decreases:

```
W ← W + α × drift_signal   (negative, so W falls)
current_target[i] ← lower_bound(W, i)   for all i
```

A lower `W` has lower lower-bounds, easing cost pressure and allowing savings
to rebuild. This is the genuine downward mobility path: deprivation sustained
long enough that the pop revises down its baseline expectations.

`W` is clamped to [0, W_max] at all times. `α` and `trim_rate` are calibration
parameters. The trimming mechanism makes downward mobility slower and stickier
than upward mobility, which is realistic.

---

## Labour Supply

Labour is a good in the simulation with `movement_type = Local`. Pop groups
supply labour to buildings in their region. A building's recipe specifies how
much labour it demands per unit of recipe size; the pop group posts sell orders
for that labour at the prevailing wage.

// TODO: the mechanics of how wages are set, how pops decide which buildings
// to work at, and what happens when a pop is unemployed or underemployed are
// not yet decided. These are core to Pop Evolution and will be designed there.

---

## State It Owns

**In SimState (runtime):**
- Per pop group: id, region, job category, cultural identity, size (f64)
- W: continuous wealth level
- current_target[i]: target quantity per good (may be below upper_bound(W,i))
- share[c][i]: within-tier allocation shares, one per good per tier
- Inventory: currency (primarily); goods are consumed immediately and not stored
- last_tick_spend: total currency spent last tick (used for savings_target)

**In game data (static):**
- Need tier definitions: unlock threshold, goods, satisfaction_per_unit,
  pref_weight, min_share, max_share
- Upper and lower target bounds per tier per discrete wealth level
- Category weights per discrete wealth level
- savings_fraction per discrete wealth level
- deposit_fraction per discrete wealth level (also law-modified)

**Scenario-variable:**
- Starting pop group distributions per region (size, job, culture, W)
- Cultural and religious preference weight modifiers (via state deltas)

---

## Open Questions

**Labour mechanics.** How wages are set, how pops choose employers, and what
the consequences of unemployment are. Deferred to Pop Evolution.

**Pop satisfaction beyond wealth.** The wealth signal captures material welfare.
Other dimensions — safety, freedom, community — may matter for political
pressure and migration but are not yet designed.

**Subsistence failure.** What happens when a pop cannot afford even lower-bound
targets and has no savings? The model has W falling, but at W=0 with empty
savings, the pop is in genuine subsistence crisis. Does it shrink? Emigrate?
Die? These are Pop Evolution questions with major scenario implications.

**Cultural preference weight dynamics.** Preference weights are modified by
laws and scripted events. Should they also drift based on consumption history
(pops acquire taste for goods they've consumed)? Not decided.

---

## Known Simplifications

- **Buy = consume for all goods**: durable goods are modelled as recurring
  maintenance spend rather than one-time purchases. This avoids tracking
  depreciation of pop-owned capital.
- **No pop-level skill tracking**: all labour within a job category is treated
  as homogeneous. Skill differentiation is deferred to Pop Evolution.
- **Immediate consumption**: goods in pop inventory exist only for the duration
  between Phase 3 (transactions) and the start of the next tick. They are not
  held across ticks.
- **Single wealth signal per group**: a pop group is modelled as having a
  single W rather than a distribution of wealth within the group. In reality,
  a factory worker group contains some workers doing better than others.

---

## Calibration Targets

- A subsistence pop receiving a 20% wage increase should drift up one wealth
  tier within 2–5 simulated years, not immediately.
- A middle-wealth pop facing a 30% sustained price increase should begin
  trimming luxury targets within 1–3 ticks and begin losing wealth tier
  within 2–4 simulated years if wages do not recover.
- In a region with abundant cheap food and scarce luxury goods, wealthy pops
  should show sustained high wealth but frustrated luxury demand — not
  spontaneously downgrade their W due to unavailability.
- Within a tier, a good that becomes 50% cheaper relative to peers should
  capture roughly 20–40% more of that tier's budget within 5–10 ticks, with
  the drift cap preventing faster switching.
- UBI equal to 50% of subsistence cost should measurably shift the average W
  of subsistence pops upward within 1–2 simulated years.

---

## Tick-Time Sensitivity

- `last_tick_spend`, `savings_target`, and all per-tick quantities are already
  in per-tick terms and scale naturally with tick duration.
- `α` (wealth drift rate) and `trim_rate` must be specified in per-week terms
  and scaled by `tick_duration_days / 7.0`. Drift too fast at short ticks
  causes oscillation; drift too slow at long ticks makes wealth unresponsive.
- The 10% per tick cap on within-tier share drift was derived from Vic3's
  weekly tick. At longer tick durations, this cap should scale accordingly
  (e.g., 30% per month at 30-day ticks).

---

## Stability Conditions

After each tick:
- Total currency across all pop group inventories changes only by: wages
  received, dividends received, credit maturity payouts, and goods purchased
  (currency transferred to sellers). No currency created or destroyed by pop
  decisions alone.
- All `share[c][i]` values remain in [min_share, max_share] and sum to 1
  within each tier.
- `W` remains in [0, W_max].
- `current_target[i]` remains in [lower_bound(W,i), upper_bound(W,i)] for
  all goods i.

---

## Test Coverage Plan

- **Unit**: liquid pool computation; savings_target formula; drift_signal
  calculation; target trimming arithmetic; within-tier share drift with cap.
- **Scenario**: isolated pop group with fixed wage and fixed prices — verify
  W converges to a stable level; verify savings stabilise at savings_target.
- **Scenario**: price shock — food prices double; verify targets trim toward
  lower bounds; verify W eventually falls if wages don't cover new costs.
- **Scenario**: wage increase — verify W rises over time; verify luxury tier
  unlocks; verify new tier targets are purchased once W crosses threshold.
- **Scenario**: within-tier substitution — two goods in same food tier, one
  becomes 50% cheaper; verify share drifts toward cheaper good over ~10 ticks;
  verify more expensive good retains min_share.
- **Calibration**: UBI scenario — add UBI equal to 50% subsistence cost;
  verify subsistence pops gain wealth tier within expected timeframe.

---

## Future Extensions

- **Pop Evolution**: growth, mortality, migration, job switching, skill
  accumulation. The current doc treats pop size and job as fixed.
- **Skill-differentiated labour**: recipes specifying skill-tier labour
  inputs. Requires Pop Evolution to define skill accumulation.
- **Pop satisfaction as a system output**: tracking how well each pop group's
  needs are met as a primary output for policy analysis (the UBI/VAT question
  is ultimately a satisfaction question).
- **Cultural preference drift**: pops acquiring taste for goods they consume.
  Relevant for modelling how consumer preferences shift over eras.
- **Political pressure**: pop satisfaction and wealth gap feed into political
  pressure on government. Deferred to Politics seam.
