# Pops

> **v2 triage (2026-07-18).** Section verdicts against [ARCHITECTURE.md](../ARCHITECTURE.md): **KEEP** = survives into v2 (light edits allowed later); **REWRITE** = concept survives, text must be redrafted; **CUT** = does not carry into v2 (may return later; see [PLAN.md](../PLAN.md)). Rewriting happens in the phase that touches each section; these are only the rulings.

## Purpose
> **v2: KEEP** — unchanged; welfare under policy is still the question.

Pop groups are the simulation's population model. They are the labour supply,
the source of consumer demand, and the human consequence of economic change.
Their spending drives prices; their wages drive production costs; their savings
drive investment. The primary research question — what happens to welfare under
high VAT and UBI — is ultimately answered by observing what pops can afford.

---

## Scope
> **v2: KEEP** — unchanged.

This system covers:
- Pop groups as aggregate entities
- Inventory and the liquid pool available for spending
- Need tiers: how pops decide what they want and at what quantity
- Buying: substitution within tiers, minimum demand
- Savings and the wealth signal
- Wealth drift: how W changes over time
- Labour supply and the employed/unemployed pair model
- Peasants as a distinct subsistence pop type

Explicitly deferred to other systems or future design:
- Pop growth, mortality, and migration (Pop Evolution — stub)
- Job switching and skill accumulation (Pop Evolution)
- Political pressure and radicalisation (Politics — seam only)
- The exact calibration of savings fractions and deposit fractions
  (calibration, not design)

---

## Pop Groups
> **v2: KEEP** — plus a frozen capability bucket (architecture/pops.md).

A pop group is an aggregate of people sharing a job category, cultural identity,
and region. It is not a collection of individuals. Its size is a continuous
value — smooth dynamics, no integer discontinuities. Pop groups are the atomic
unit of the population model: they supply labour, buy goods, and save.

A pop group is not the same as a wealth tier. Wealth W is a continuous signal
per pop group that changes over time. A group of factory workers in Manchester
in 1840 starts at some W and drifts based on their wages and the cost of
living. Two factory worker groups in different regions with different wages will
have different W values even if they share a job category.

---

## Inventory and the Liquid Pool
> **v2: REWRITE** — no deposits in v1 — liquid pool = cash; deposits return with the credit module (architecture/money.md).

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
> **v2: KEEP** — the tier/table design is implemented and good; the latent minimum-demand idea stays.

Each pop group has a set of active **need tiers** determined by its current
wealth level W. A need tier is a specific consumption target that unlocks at
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
> **v2: REWRITE** — substitution finally gets its price term (logit over preference x exp(-beta p/p-bar)); the stateful drift is cut (architecture/pops.md).

In Phase 1, the pop group posts buy orders based on current targets, limited by
the liquid pool L.

**Budget allocation**: the liquid pool is allocated across active tiers by
weights derived from the current wealth level (interpolated from config). A
poor pop allocates most of L to food and energy tiers; a rich pop allocates
less to basic tiers and more to luxury and financial tiers.

For each tier, the budget determines how much currency is available for goods
in that tier. The quantity ordered for each good is the minimum of the current
target and what the tier budget can afford at last tick's price — surplus tier
budget goes to savings.

**Within-tier substitution**: each good's share of the tier budget drifts
toward the allocation that minimises effective cost per unit of satisfaction,
where effective cost accounts for the good's price and the pop's preference
weight for it. Shares are clamped to configured min and max bounds per good.
Actual shares drift toward the ideal at a bounded rate per tick, preventing
abrupt switches. Cultural and religious modifiers adjust preference weights
via laws and scripted events.

Prices appear only to compute quantities from spend and to determine the ideal
share ordering. The pop does not compare prices across tiers.

---

## Savings and the Wealth Signal
> **v2: REWRITE** — replaced by the absorption cap + overflow routing (architecture/ownership.md, architecture/pops.md); no deposit split in v1.

After market clearing the pop computes savings as the liquid pool minus actual
spend. Savings are split: a wealth-dependent fraction goes to bank deposits
(1-tick credit instruments that mature next tick), the rest is retained as cash.
The deposit fraction rises with wealth and is modified by laws governing
financial sector access.

The wealth signal compares savings to a target buffer proportional to last
tick's spend. The savings fraction of that target rises with wealth — a
subsistence pop needs roughly one week's food budget in reserve; a high-wealth
pop needs several months of lifestyle spend. The gap between actual savings and
target determines the direction and speed of wealth drift.

---

## Wealth Drift
> **v2: CUT** — the PD controller dies; the consumption desk's scale is the wealth tier, nudged by the kernel (architecture/pops.md).

W is a continuous value that drifts each tick based on the savings signal.

**When savings exceed the target** (pop is flush), W rises. As W rises,
active tiers may unlock and targets jump to the aspirational upper bound at
the new wealth level. The pop immediately tries to reach new higher targets,
which typically depresses savings temporarily until income catches up.

**When savings fall short of the target** (pop is stretched), targets are
trimmed first before W falls. Targets trim uniformly toward their lower bounds;
only once all targets are at their lower bounds does W itself decrease. A lower
W has lower lower-bounds, easing cost pressure and allowing savings to rebuild.
This is the genuine downward mobility path: deprivation sustained long enough
that the pop revises down its baseline expectations.

The trimming mechanism makes downward mobility slower and stickier than upward
mobility, which is realistic. W is bounded below at zero and above at a
configured maximum.

---

## Labour Supply
> **v2: KEEP** — the employed/unemployed pair + fill-rate stickiness is v1's best mechanism; carried forward, extended by the capability ladder and the participation scale π; the unemployment-shock paragraph's wealth-controller damping goes with Wealth Drift's CUT (architecture/pops.md).

Labour is a good with `movement_type = Local`. Pop groups are stored as
**employed/unemployed pairs**: one employed half and one unemployed half,
sharing a job category, culture, and region. The pair is the atomic unit of
the population model. Both halves post labour sell orders to the regional
market; buildings buy from that pool as recipe inputs. There is no direct
employer-employee assignment — labour clears as a pooled regional market good
and wages emerge from imbalance.

**Employed pops** always post their full labour supply (size × labour_rate).
They do not reduce supply in response to falling wages; employed workers don't
typically quit over a pay cut.

**Unemployed pops** post supply proportionally to their last-tick fill rate.
If 60% of their posted supply sold last tick, they post 60% of full supply this
tick. This gives downward wage stickiness without hardcoding any price level:
when demand falls and fill rates drop, the unemployed withdraw partially from
the market, reducing downward pressure on wages.

After clearing each tick, employed and unemployed sizes are redistributed so
that employed size equals fill_rate × total_pair_size, with the remainder
unemployed. W belongs to each half independently and does not cross the
boundary — only size and inventory move. Total inventory is redistributed
proportionally to the new sizes in the same step.

A large unemployment shock deposits substantial savings into the unemployed
half. The savings signal reads as flush, which would normally push W up — but
the derivative term of the wealth controller (which reflects that W was already
falling) dampens this, giving the cohort time to survive on their savings
rather than immediately spending down.

Each half is always retained even at zero size, so that W is preserved across
periods of full employment or full unemployment. If the unemployed half of a
pair has never existed and pops flow into it for the first time, it initialises
with W equal to the employed half's current W.

// TODO: labour_rate (supply per unit of pop size) — likely a per-job-category
// config value rather than per-pair.

---

## Peasants
> **v2: REWRITE** — generalized into the self-provision margin available to every pop (architecture/pops.md); peasant becomes scenario content, not a type.

Peasants are a special pop type that exists outside the normal
employed/unemployed building-labour loop. They do not work at buildings. They
are a model of subsistence agriculture: self-sufficient, market-peripheral, and
a structural buffer against famine and rapid urbanisation.

**Role in the economy:**
- Are themselves famine-immune: their subsistence production covers their own
  consumption regardless of market prices. Extreme cases (plague, conquest) can
  be modelled via scripted events.
- Protect others against famine by providing a small but unconditional local
  food supply that is not dependent on market prices or supply chains.
- Exert little pressure on wages — their labour supply to market is small and
  driven by savings surplus, not wage rates.
- Act as a stickiness against urbanisation: a peasant leaving subsistence means
  giving up land security and accepting market dependency.
- Are displaced by industrial farming recipes, which compete for agricultural land.
- Have no wealth signal W and never post buy orders to any market. They do not
  participate as consumers.

**Land holdings:**
Peasants hold agricultural land in their inventory like any other good. Regional
laws govern the maximum fraction of their land they may post for sale. Within
that legal ceiling, peasants modulate their posted sell quantity between zero
and the maximum, using bespoke logic that targets the current EMA price of land
— posting more when the market price is above EMA, less when below, to avoid
both dumping and hoarding. If demand is strong enough, prices will still rise
despite this modulation; the mechanism prevents collapse, not appreciation.

When land sells, the proceeds and the corresponding peasant size flow to the
unemployed half of the unskilled pop pair for the same region. The profits from
the sale go to that newly-displaced unemployed half, not back to the remaining
peasants. The peasant size shrinks by the amount displaced.

**Subsistence production:**
Peasants are magic producers of a very small quantity of the most basic staple
goods — approximately 10% of the lowest-wealth-tier need per unit of size. This
supply is not market-responsive. It is enough to keep the peasant alive and
create a latent price signal for basic goods, but not enough to supply an
industrial population.

**Labour supply and graduation:**
Peasants do not post labour supply based on wages or cost of living. Their
market participation is governed by their savings surplus relative to
subsistence: they compute how much of their size their savings could support at
the lowest-wealth-level spending rate and their savings target, and post that
as labour supply. As savings accumulate beyond what local subsistence requires,
more supply is posted; as savings deplete, they withdraw.

When a peasant's posted labour is purchased, that fraction of their size
graduates into the unskilled employed half for the same region. They carry a
proportional share of their total inventory with them, including any land held.

---

## Open Questions
> **v2: REWRITE** — subsistence-failure and satisfaction questions restate against the self-provision margin.

**Pop satisfaction beyond wealth.** The wealth signal captures material welfare.
Other dimensions — safety, freedom, community — may matter for political
pressure and migration but are not yet designed.

**Subsistence failure at W=0.** At zero wealth with empty savings, a non-peasant
pop is in genuine crisis. The model has W floored at zero but does not yet
specify what happens next. Shrinkage, emigration, and death are all Pop Evolution
questions with major scenario implications.

**Cultural preference weight dynamics.** Preference weights are modified by
laws and scripted events. Should they also drift based on consumption history?
Not decided.

---

## Known Simplifications
> **v2: KEEP** — unchanged.

- **Buy = consume for all goods**: durable goods are modelled as recurring
  maintenance spend rather than one-time purchases.
- **No pop-level skill tracking**: all labour within a job category is treated
  as homogeneous. Skill differentiation is deferred to Pop Evolution.
- **Single wealth signal per half**: a pop half is modelled with a single W
  rather than a distribution of wealth within the group.

---

## Calibration Targets
> **v2: KEEP** — destined for criteria.ron.

- A subsistence pop receiving a 20% wage increase should drift up one wealth
  tier within 2–5 simulated years, not immediately.
- A middle-wealth pop facing a 30% sustained price increase should begin
  trimming luxury targets within a few ticks and begin losing wealth tier
  within a few simulated years if wages do not recover.
- In a region with abundant cheap food and scarce luxury goods, wealthy pops
  should show sustained high wealth but frustrated luxury demand — not
  spontaneously downgrade their W due to unavailability.
- Within a tier, a good that becomes significantly cheaper relative to peers
  should capture a meaningfully larger share of that tier's budget within
  roughly 5–10 ticks.
- A large sudden unemployment shock should leave the newly-unemployed cohort
  able to sustain spending for at least several months before W begins falling
  noticeably, if they had meaningful savings before displacement.

---

## Future Extensions
> **v2: KEEP** — parking lot.

- **Pop Evolution**: growth, mortality, migration, job switching, skill
  accumulation. The current doc treats pop size and job as fixed.
- **Skill-differentiated labour**: recipes specifying skill-tier labour
  inputs. Requires Pop Evolution to define skill accumulation.
- **Pop satisfaction as a system output**: tracking how well each pop group's
  needs are met as a primary output for policy analysis.
- **Cultural preference drift**: pops acquiring taste for goods they consume.
- **Political pressure**: pop satisfaction and wealth gap feed into political
  pressure on government. Deferred to Politics seam.
