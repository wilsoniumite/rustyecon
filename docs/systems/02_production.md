# Production

## Purpose

The production system transforms goods into other goods — the engine of economic
growth, technological change, and industrial development. It determines what can
be produced where, how efficiently, at what cost, and how production capacity
responds to market signals over time.

Production is also the primary mechanism through which historical change enters
the simulation. Technology is not a separate research tree — it is a schedule of
recipe unlocks scripted per region. The Bessemer converter is not an invention to
be discovered; it becomes available in Sheffield in 1856, and in Ruhr in 1863, and
so on. The economic impact emerges from the recipe change, not from the unlock
mechanism.

## Scope

This system covers:
- Buildings: what they are, what they hold, how they produce
- Recipes: inputs, outputs, switching costs between them
- Throughput: how output scales with capital and inputs
- Construction and exit: how new capacity enters and leaves
- The transition state: how buildings blend two recipes during conversion

Explicitly deferred to other systems:
- What labour costs (Pop Evolution / Pop Needs)
- How prices form and goods are bought and sold (Markets)
- How governments tax production and set labour laws (Government)
- Recipe unlock schedules — the scripted timeline of when recipes become available
  per region (Technology)
- Financial instruments used to fund construction (Financial Markets)

## Buildings

A building is the simulation's unit of productive capital. It is not a physical
factory but an abstraction over a coherent productive enterprise — a textile mill,
a blast furnace, a shipyard, a farm. It holds:

- **A recipe** — the single production method currently being run (or two recipes
  during a transition, see below)
- **An inventory** — input goods buffered from last tick's purchases, output goods
  pending sale
- **A labour relationship** — it employs a number of pop-slots at a wage; the wage
  is set by the labour market at the time of employment
- **Capital stock** — a currency-denominated measure of productive capacity, set at
  construction and modified by depreciation, reinvestment, and disinvestment
- **Fixed assets** — goods consumed during construction that are tied up for the
  building's operating life (land, physical structure, specialised equipment)

Buildings are owned. Ownership determines who receives profit (residual income
after input costs and labour). The owner entity may be a pop group, a private
firm abstraction, or the government. Ownership matters for where profit flows,
not for how production works.

## Recipes

A recipe defines one way a building can produce. It has:

- **Inputs**: goods consumed per unit of throughput, including labour
- **Outputs**: goods produced per unit of throughput; may be multiple (fixed ratio)
- **Capital requirements**: goods required to exist as fixed assets when the
  building is operating this recipe (e.g. `engines` for a power loom)
- **Switching costs to other recipes**: expressed as a fraction of new-build
  cost, in goods consumed and time elapsed. Default assumption is 100% (full
  rebuild). Some transitions are cheaper; a few are free.

Multi-output recipes produce all outputs at fixed ratios. These ratios do not
respond to relative prices — the recipe either runs or it doesn't. If one output
is in glut and the other is in shortage, the building still produces both.
This is intentional: it creates exactly the price dynamics that occurred
historically (coal tar as waste until synthetic dyes, tallow oversupply before
industrial soap, hide surpluses before the leather industry scaled).

### Recipe Justification Criteria

A recipe earns a distinct entry in the simulation if it meets one or more of:

1. **Different capital structure** — switching from it to another recipe costs
   less than building new (otherwise they are separate buildings, not recipes)
2. **Meaningfully different input mix** — different goods required, or
   significantly different ratio of labour to physical inputs
3. **Different output structure** — different output goods, or different
   multi-output ratios
4. **Historical non-substitutability for pops** — goods that pops distinguish
   even if economically similar (e.g. cotton fabric vs. wool fabric)

Aesthetic differentiation alone is not sufficient. If two "recipes" would produce
the same output from near-identical inputs with no significant switching cost
difference, they should be one recipe with a throughput modifier instead.

## Throughput

A building's output per tick is:

```
throughput = capital_stock × input_fill_rate × efficiency_modifier
```

- `capital_stock` — set at construction; scales total potential output
- `input_fill_rate` — fraction of desired inputs obtained this tick (from
  market clearing; if rationed, throughput scales proportionally)
- `efficiency_modifier` — net of all active modifiers (labour law, shortage
  flag, regional bonuses, etc.)

Labour is one of the inputs and participates in the same fill-rate calculation.
If the labour market cannot supply the required workers at the posted wage, the
throughput shortfall is treated identically to a material input shortage.

**Fixed costs are paid regardless of throughput.** Capital depreciation, land
rent, and fixed asset maintenance occur every tick irrespective of whether the
building is producing. A building that cannot cover variable costs shuts down
output (reservation price logic from Markets), but continues paying fixed costs
until it exits entirely.

## The Transition State

A building can hold at most two recipes simultaneously, representing a transition
in progress. This is not the normal operating state — it is a bounded period
during which the building is converting from one recipe to another.

During transition, the building allocates its throughput capacity as a blend:

```
total_output = alpha × recipe_A_output + (1 − alpha) × recipe_B_output
total_inputs = alpha × recipe_A_inputs + (1 − alpha) × recipe_B_inputs
```

where `alpha` starts at or near 1.0 (fully on recipe A) and moves toward 0.0
(fully on recipe B). The pace at which `alpha` changes is governed by two
constraints:

1. **Conversion speed** — how fast the physical transformation of fixed assets
   can occur; denominated in goods consumed per tick during conversion
2. **Owner decision** — the owner can slow, pause, or reverse the transition
   at any point; the decision logic mirrors the sell-order hedging in Markets
   (proportional signal, derivative damping, noise)

A building can only be in one transition at a time. Starting a new transition
requires either completing or explicitly abandoning the current one.

The transition state smooths large supply/demand shocks. A region with many
large textile mills converting from handloom to power loom does so gradually —
cotton demand rises slowly, coal demand rises slowly, fabric supply dips slowly —
rather than in a single-tick cliff.

### Conversion Costs

Switching from recipe A to recipe B consumes:

- Goods: the goods defined in the switching cost entry for that (A→B) pair,
  drawn from the building's own reserves or purchased in the market
- Time: N ticks of reduced effective throughput during conversion
- Currency: labour hired specifically for the conversion work

The switching cost matrix is sparse — most (A, B) pairs have the same cost as
new construction (100%). Named lower-cost transitions are listed explicitly in
scenario data.

If a building exits mid-transition, the conversion goods consumed so far are
lost. This is not recoverable.

## Construction

New buildings enter via explicit construction decisions. The actor is either:

- A pop group or private firm entity (motivated by expected profit margin)
- A government (motivated by strategic objectives — arms, infrastructure)

Construction is modelled as a recipe run by a construction company building
(which itself exists in the simulation). Inputs are construction goods — tools,
structural materials, labour — consumed over N ticks. On completion, the new
building enters with full capital stock.

**New construction, not transition, is the primary mechanism by which gaps in
supply are filled in a healthy market.** The decision to build new should be
triggered earlier and more reliably than the decision to transition an existing
building. The trigger condition is approximately:

```
expected_margin(new_recipe) > construction_amortised_cost_per_tick + risk_premium
```

Existing buildings are sticky: they switch only when the margin gap is large
enough to justify conversion costs. New entrants see the full margin available
and enter first. This replicates the historical pattern — new industrial capacity
was built in greenfield sites while existing craft producers continued until
undercut.

## Exit

A building exits when it cannot cover average total cost over a sustained period
(configurable — default ~1 simulated year). Exit is gradual: the building first
drops to minimum throughput (covering fixed costs if it can), then, if still
unprofitable, begins selling off fixed assets and releasing labour.

On exit, fixed assets that are resaleable (engines, structural iron) return to
the market as goods. Fixed assets that are not resaleable (site-specific
earthworks, specialised furnace structures) are destroyed — their value is simply
lost.

Land is recovered and re-enters the regional land market.

## Fixed Assets

Fixed assets are goods that are consumed at construction and are not in the
building's tradeable inventory. They are held by the building as a separate
register, distinct from the working inventory.

Categories in Era 1:
- **Land** — a non-storable, non-moveable regional good. Acquired over a few
  weeks. All agricultural and most industrial buildings require land.
- **Structures** — physical construction: factory buildings, mine shafts, docks.
  Produced by the construction industry. Months to acquire.
- **Specialised equipment** — machinery with limited alternative use: blast
  furnace vessel, Bessemer converter, power loom frames, ship-building
  dry dock. Months to years to acquire depending on complexity.

Switching costs between recipes are primarily determined by how much of the
current recipe's fixed assets can be reused. A recipe transition that reuses
the same structure but replaces equipment costs less than one that requires a
new structure. A transition that reuses both structure and equipment is cheap
or free.

## State It Owns

In `SimState` at runtime:
- All building instances: id, owner, region, current recipe(s), capital stock,
  fixed assets register, inventory, employed labour count, current wage
- Transition state per building: recipe A, recipe B, alpha, conversion goods
  consumed so far, ticks elapsed
- Construction projects: inputs consumed, ticks remaining, target building spec

In `game_data` (static):
- Recipe definitions: inputs, outputs, switching cost matrix
- Building type definitions: which recipes are available, construction cost
- Fixed asset type definitions: resaleability, depreciation rate

Scenario-variable:
- Which recipes are unlocked per region (the scripted tech schedule)
- Starting building distributions per region

## Known Simplifications

- **Capital stock as a scalar**: real capital is heterogeneous. Our `f64`
  capital stock collapses this. A blast furnace and a textile mill have
  incomparable capital, but the simulation treats them symmetrically.
- **Profit motive only for construction**: in reality, government industrial
  policy, prestige, and strategic motives drive significant investment.
  Government-initiated construction is supported but the decision logic is
  simpler (scripted or threshold-based, not margin-optimising).
- **One construction recipe per building type**: real construction projects
  are highly complex. We use a single recipe per building type.
- **Labour as a homogeneous input**: skilled vs. unskilled labour matters
  enormously historically. If pop types include skill levels, this can be
  addressed; otherwise it is absorbed into the efficiency modifier.
- **No inter-building dependencies within a region**: in reality, a blast
  furnace and a toolworks might be integrated under one firm. We model them
  as separate buildings transacting through the market.

## Calibration Targets

- A region that builds its first blast furnace should see iron goods prices
  fall and imported iron imports decline within 2–5 simulated years.
- After the Bessemer unlock, steel output should displace iron goods in
  downstream uses within 10–20 years (matching the 1860s–80s historical
  pattern).
- Power loom mills should be unprofitable to build in regions where cotton is
  expensive and coal is absent, even if the recipe is unlocked — the input
  cost prevents adoption without cheap coal access.
- Multi-output recipe byproducts should be priced near zero when the byproduct
  good has no demand, and should rise to a positive price as downstream demand
  develops (e.g., coal tar price should be negligible before 1856 and positive
  after aniline dye production unlocks).
- A building running below variable cost should not persist more than ~1
  simulated year before exiting; one running above variable cost but below
  average total cost should persist several years (waiting for conditions to
  improve).

## Tick-Time Sensitivity

- **Construction time** must be denominated in real time (weeks), then
  converted to ticks. A building that takes 6 months to construct should take
  ~26 ticks at weekly granularity and ~6 ticks at monthly.
- **Transition speed** same normalisation. The blend fraction `alpha` changes
  per tick at a rate proportional to `tick_duration / conversion_time`.
- **Exit persistence**: the N-tick unprofitability threshold must scale with
  tick duration.
- **Throughput**: all recipe quantities are per-week; scale by
  `tick_duration_days / 7.0`.

## Stability Conditions

After each tick:
- Total goods across all inventories is conserved (inputs consumed = outputs
  produced; construction goods consumed = construction progress; no goods
  created or destroyed outside of recipes).
- No building holds negative inventory.
- A building in transition has `0.0 ≤ alpha ≤ 1.0`.
- A building's fixed asset register does not increase without corresponding
  construction goods consumption.
- Total labour employed ≤ total labour available in the region (this is a
  Markets / Pop constraint but Production must not over-employ).

## Test Coverage Plan

- **Unit**: throughput formula with rationed inputs; multi-output recipe
  proportions; transition blend arithmetic; switching cost deduction.
- **Scenario**: single building, isolated market — build a blast furnace, verify
  iron goods price falls, verify coal is consumed and iron ore is consumed in
  correct ratio. Run until exit; verify fixed assets are released correctly.
- **Scenario**: recipe transition — textile mill handloom → power loom; verify
  throughput blends smoothly; verify coal demand rises continuously; verify
  fabric supply dip is bounded; verify mid-transition reversal works.
- **Scenario**: multi-output — blast furnace with coke_iron running; verify coal
  tar accumulates (no buyers) and depresses coal tar price; then unlock aniline
  dye recipe and verify coal tar price rises and inventory drains.
- **Calibration**: new construction triggered by margin; verify buildings enter
  a profitable market within N ticks; verify they do not enter an unprofitable
  market even when the recipe is unlocked.

## Future Extensions

- **Firm abstraction**: multiple buildings owned by one firm entity, allowing
  internal transfer pricing and firm-level investment decisions. Currently each
  building is independent.
- **Skill-differentiated labour**: recipes could specify a mix of unskilled and
  skilled labour inputs. Skill acquisition by pops would then be a genuine
  bottleneck for industrial upgrading.
- **Intangible fixed assets (IP)**: relevant from Era 4 onward. A software
  company's capital is primarily IP; construction looks like hiring engineers
  over several years, not building a factory. The fixed asset framework extends
  to this without structural change.
- **Supply chain integration**: a firm owning both a blast furnace and a
  toolworks could bypass the market for internal transfers. Deferred; worth
  revisiting if market noise creates too much inefficiency in tightly coupled
  chains.
- **Partial exit / mothballing**: a building that suspends output but retains
  fixed assets, paying only maintenance costs, waiting for market conditions to
  improve. Currently buildings either run or exit; a mothball state would be
  more realistic for large capital-intensive facilities.
