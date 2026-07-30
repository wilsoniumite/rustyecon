# Pops

Pops are the labour supply, the consumer demand, and the only owners. A pop is
an employed/unemployed pair, one shared inventory, a frozen capability bucket,
and two kernel rule-carriers (the labour margin π and the consumption desk).

## The capability ladder

Labour is four goods, `labour_r1..r4`, one market each per region. Every pop
carries a frozen capability bucket — scenario data approximating a fixed bell
curve over the population — and posts labour **to its own bucket's market
only** (cross-bucket mobility is a named deferral). Recipes demand specific
buckets; technology versions rewrite which buckets a recipe needs. This makes
the wage *distribution* — and the ratio R = ln(w_high/w_low) — a measured
output rather than an aggregate assumption.

## Posting: the employed/unemployed pair

The pair is the posting entity:

- **Employed** halves post their full supply — employed workers do not quit
  over a pay cut.
- **Unemployed** halves post supply scaled by their last fill rate — when
  demand falls, the unemployed partially withdraw, which is downward wage
  stickiness without any hardcoded price level.
- After clearing, sizes redistribute so employed size = fill × pair size.
  Inventory moves proportionally; each half persists even at zero size.

Both halves' postings are multiplied by the pop's participation scale **π**.

Labour is endowed, not produced: posted hours are minted at posting with
provenance `LabourMint`, and unsold hours expire at end of tick — the day
cannot be stored.

## The participation / self-provision margin

π is a kernel scale ([kernel.md](kernel.md)) whose pressure signal is
`(w_posted / P_basket − parity) / parity`, where `parity` is the pop's registered
self-provision wage-equivalent. Wages above parity draw hours into the market;
wages below it push them out.

> **Units made explicit 2026-07-31 (v2 Phase 3.5).** This line previously read
> `(w_posted − parity) / parity`, subtracting a nominal wage from a quantity
> whose units were stated nowhere — and `parity` appeared zero times in `src/`
> and `data/`, so nothing forced the question. It is a **quantity of the
> subsistence basket per hour**, the same registered rate the withheld hours
> produce at below, so the comparison is real on both sides and `w_posted` is
> deflated by `P_basket`, that basket's price at the pop's node. Registering it
> as a currency amount instead would pin a price outside the price system (R12).
> π is the labour market's only corrective state variable, so this is
> load-bearing rather than cosmetic — see kernel.md, "Price formation". Hours not offered (`1 − π`) produce subsistence
goods directly into the pop's own inventory at a registered rate — real
provision, unpriced, provenance `SelfProvision`, visible to *true* telemetry
and invisible to *measured* ([markets.md](markets.md)).

Measured participation is therefore the net of two motions — conversion pulling
hours into the priced zone, displacement pushing them out — and a "peasantry"
is not a special type but a population living mostly on this margin. Famine
resistance, urbanization stickiness, and the participation series all fall out
of the same rule.

## Consumption

Basket quantities come from the **wealth-tier tables**: per tier, quantities
per need category, linearly interpolated between tiers — a clean, data-driven
demand system that scenario data fully controls.

- **The tier is the scale.** The consumption desk's kernel scale *is* the
  pop's wealth tier: its pressure signal is cash-vs-reserve, so sustained
  income climbs the tiers and sustained shortfall descends them. No separate
  wealth-drift controller exists.
- **The absorption cap is the top tier.** Top-tier quantities are finite by
  construction — satiation lives in the data. A pop at the top tier with cash
  above its band generates overflow, which routes to the minting queue
  ([ownership.md](ownership.md)). The saving gradient — the rich save a rising
  share — needs no taste assumption: consumption is bounded and income is not.
- **Within-category substitution** splits spending by logit:
  `share_g ∝ preference_g · exp(−beta · p_g / p̄)` — cheap goods within a
  category gain share smoothly; categories never substitute for each other.
- Buys are budget-capped in Rule 3's single per-inventory pass; rationing
  shortfalls are recorded per pop class (who goes short is output).

## Destitution

A pop descending tiers hits the bottom tier's minimum basket. Below that, fills
below need are recorded as deprivation (a criteria-battery input —
DESTITUTION is a registered failure class), the self-provision margin is the
built-in floor, and mortality/migration responses are deliberately left to the
tape until an emergent treatment is designed and registered.
