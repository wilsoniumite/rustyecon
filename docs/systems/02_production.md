# Production

> **v2 triage (2026-07-18).** Section verdicts against [ARCHITECTURE.md](../ARCHITECTURE.md): **KEEP** = survives into v2 (light edits allowed later); **REWRITE** = concept survives, text must be redrafted; **CUT** = does not carry into v2 (may return later; see [PLAN.md](../PLAN.md)). Rewriting happens in the phase that touches each section; these are only the rulings.

## Purpose
> **v2: KEEP** — unchanged.

The production system transforms goods into other goods. It determines what can
be produced where, at what cost, and how productive capacity responds to market
signals and historical change over time.

Technology in this simulation is not a research tree. Recipe unlocks are scripted
per region with historical dates. The Bessemer converter becomes available in
Sheffield in 1856 and in Ruhr in 1863; the economic impact emerges from the
recipe change entering the market, not from a discovery mechanism.

---

## Scope
> **v2: REWRITE** — components out; minting and growth in (architecture/ownership.md).

This system covers:
- Buildings and what they hold
- Components: non-market fixed assets
- Recipes: inputs, outputs, cost structure, component requirements
- Recipe size, chosen recipe size, effective recipe size
- Component transfer: how downsizing buildings fund new construction
- Construction: how new capacity enters
- Exit: how capacity leaves
- Capital vintage: efficiency decay and refresh

Explicitly deferred:
- How prices form and goods clear (Markets)
- What labour costs and what pops supply (Pop Needs / Pop Evolution)
- How governments tax production and set laws (Government)
- The scripted schedule of when recipes unlock per region (Technology)
- Financial instruments used to fund construction (Financial Markets)

---

## Buildings
> **v2: REWRITE** — building becomes desk; the two asset registers collapse to one inventory + size (architecture/objects.md).

A building is the atomic unit of productive capital. It runs exactly one recipe
at all times. It is an abstraction over however many real enterprises share a
recipe in a region — Lancashire's 300 cotton mills are one building. There is
at most one building per recipe per region.

A building holds two asset registers:

**Inventory** — any goods including currency. Input goods buffered from market
purchases, output goods pending sale, working cash. Fully liquid.

**Component register** — the components belonging to the building's current
recipe. Not tradeable; cannot appear in any inventory. Includes land, structures,
specialised machinery, and any other components the recipe requires.

Buildings are owned. Ownership may be split among multiple actors. Ownership
determines profit distribution and who makes decisions on the building's behalf.
Ownership mechanics are an open question.

### One Building per Recipe per Region
> **v2: KEEP** — unchanged, as one desk per recipe version per region.

Multiple buildings running the same recipe in the same region are not permitted.
Buildings are price-takers in a posted-price market: two buildings with the same
recipe in the same region see identical prices and make near-identical decisions.
Intra-region competition on the same recipe adds noise without meaningful
dynamics. Split ownership within one building handles profit distribution across
multiple actor groups.

---

## Components
> **v2: CUT** — deferred; capital cost k covers v1 grain (architecture/objects.md; deferral list in ARCHITECTURE.md).

A component is a non-market fixed asset held in a building's component register.
Components cannot be traded — they never enter any inventory or market. They are
built on-site from market goods plus construction service goods, and they are the
primary store of a recipe's capital.

Examples: land (purchased from the regional market but then held as a component),
factory structures, blast furnace vessels, power loom frames, Bessemer converters,
IP, trained workforce expertise.

Each component definition specifies:
- **Build specification**: goods consumed + construction service goods + time per
  unit of component
- **Recovery fraction**: the fraction of build cost recoverable as market goods
  when dismantled (approximately: land ~100%, structures ~90%, general industrial
  machinery ~20–50%, specialised equipment ~10–20%, IP/expertise ~0%)

When a building reduces its recipe size, components are either dismantled
(recovering goods at the recovery fraction) or transferred directly to another
building (see Component Transfer below). Direct transfer is always more efficient
than dismantlement because it skips both the recovery loss and the rebuild cost.

### Construction Services as Goods
> **v2: KEEP** — construction goods are how k gets paid; the tier list is scenario content.

Building components requires construction capability. Construction services are
market goods (`movement_type = Local`) produced by construction company buildings.
This is a non exhaustive, toy example list:

- `construction` — basic labour and simple tools. Available everywhere from
  simulation start.
- `heavy_industry_construction` — cranes, heavy equipment, specialist labour.
  Required for large industrial components.
- `precision_engineering` — required for high-tolerance machinery.
- `digitization_consulting` — required for IT infrastructure (Era 5+).
- `ai_services` — required for AI-enabled components (Era 7).

Construction company buildings are themselves built from components using
lower-tier services. The bootstrap resolves because basic construction is
labour-intensive and requires only land and simple tools.

### Land
> **v2: REWRITE** — land becomes the Parcel object with omega and claims, not a component (architecture/objects.md, architecture/ownership.md).

Land is purchased from the regional land market and held as a component. It is
owned by actors (primarily aristocrat pop groups in early eras) who post it to
the regional market via normal sale logic. Recipes that require land specify
which type is acceptable.

Three land types in approximate order of typical market value: urban land,
agricultural land, rural land. Land type is set by scenario state and does not
change. Recovery on building exit is ~100% — it returns to the regional land
market.

---

## Recipes
> **v2: KEEP** — plus versioning for technology (architecture/objects.md).

A recipe is a production method a building runs. Recipes are defined in game
data (static). Multiple buildings in different regions can run the same recipe.

A recipe specifies:

**Inputs** — goods consumed per unit of effective recipe size per tick. Each
input has a scaling behaviour: fully variable (scales with chosen recipe size),
fixed (paid at recipe size regardless of chosen size), or semi-variable (fixed
floor plus variable portion). Labour is an input; its scaling curve is defined
per recipe.

**Outputs** — goods produced per unit of effective recipe size per tick. May be
multiple at fixed ratios. Ratios do not respond to relative output prices. The
recipe either runs or it does not. Overproduction of one output depresses its
price regardless of shortage in another — this is intentional and creates
historically accurate dynamics (coal tar as waste before synthetic dyes; hide
surpluses before industrial leather demand).

**Component requirements** — the components that must be held at the building's
full recipe size. Shortfall reduces recipe size proportionally.

**Recipe unlock condition** — which regions can run this recipe and from when.
Stored in the scenario's state delta schedule, not in the recipe definition.

### Recipe Justification
> **v2: KEEP** — unchanged.

A recipe earns a distinct definition if it meets one or more of:
1. Meaningfully different input mix (different goods or significantly different
   labour-to-material ratio)
2. Different output structure (different goods or different multi-output ratios)
3. Different component requirements (implying non-trivial reconstruction cost
   relative to other recipes)
4. Historical non-substitutability for pops (goods pops distinguish even when
   economically similar)

Cosmetic variation alone is not sufficient.

---

## Recipe Size, Chosen Recipe Size, Effective Recipe Size
> **v2: KEEP** — renamed size / scale / effective; same trichotomy.

**Recipe size** is set by the components held. Holding full components at size N
means the recipe can run at rate N. Recipe size increases by acquiring more
components (construction cost + time) or decreases by releasing them.

**Chosen recipe size** is what the building elects to run each tick, in
[0, recipe size]. Fixed costs are paid at recipe size; variable costs scale with
chosen recipe size.

**Effective recipe size** is what actually gets processed after market rationing.
If the building chose size X but obtained fraction f of its variable material
inputs, effective recipe size = f × X for those inputs. Fixed inputs are
unaffected by rationing.

### Cost Structure
> **v2: KEEP** — InputScaling is implemented and correct — one of v1's best parts.

Each recipe defines scaling behaviour per input:

- **Fixed**: paid at recipe size regardless of chosen size. Land rent, structural
  maintenance, fixed-term labour contracts.
- **Variable**: scales with chosen recipe size. Most material inputs, piece-rate
  labour.
- **Semi-variable**: fixed floor plus variable component.

A building below variable cost sets chosen recipe size to zero but holds recipe
size (paying fixed costs) while awaiting recovery. A building unable to cover
average total cost over a sustained period exits.

---

## Component Transfer
> **v2: CUT** — goes with components.

When a building reduces its recipe size, it has two options for the released
components:

1. **Dismantle** — components are converted back to market goods at the
   recovery fraction. Some value is lost; the goods enter the building's
   inventory and are sold on the market.

2. **Transfer** — components are moved directly to a target building's
   component register, bypassing the recovery loss and any rebuild cost.
   Transfer is always more efficient than dismantle + rebuild.

Each tick, a building may submit a transfer action: move N units of a component
to a specified target building. As part of the transfer, the source building's
owners receive proportional ownership shares in the target building equal to
the value transferred.

A building holds an optional `transfer_target` attribute — a reference to
another building in the same region (new or existing). This persists tick to
tick and is changed by actor decision. If no transfer target is set, released
components are dismantled by default.

**Transfer to a new building under construction**: the most common case. The
source building is downsizing recipe A; the target is a new building under
construction with recipe B. Components that B requires and that A is releasing
transfer directly, funded by the source's owners who gain proportional stake in B.

The "transition" in design and documentation refers to the combination of a
source building downsizing and a target building growing via component transfer.
It is not a simulation construct — there is no transition object in SimState,
only the source building's `transfer_target` attribute and the per-tick transfer
decisions.

---

## Construction
> **v2: REWRITE** — becomes the minting queue: equity-financed, claims minted pro-rata, best yield first (architecture/ownership.md).

New buildings enter via construction decisions made by actors. Construction
consumes market goods and construction service goods over time, per the component
build specifications of the target recipe. On completion the building enters
with full recipe size.

### The Opportunity Signal and Capital Allocation
> **v2: REWRITE** — simplified to a public yield ranking; mu and r-star are its telemetry readouts (architecture/ownership.md).

Each positive-opportunity (region, recipe) pair posts a **currency buy order**:
"I will accept up to X currency and return ownership shares proportional to
contribution." X is the opportunity value — derived from last tick's prices, it
represents how much capital could profitably be invested here before expected
returns fall to zero.

Capital suppliers (pop groups with surplus currency, financial institutions that
have channelled currency to this region) fill these buy orders. Allocation is by
return signal: the highest-return opportunity in the region gets funded first,
then the next, until capital runs out or all opportunities are filled. Multiple
suppliers filling the same opportunity receive ownership shares proportional to
their contribution.

This is an allocation mechanism, not a posted-price market. There is no "price
of opportunity" that adjusts to clear the market. Interest rate discovery happens
in credit instrument markets (see Financial Markets). Here, the return signal
attracts capital; capital flows until the opportunity is filled or capital runs
out.

Actors observe the opportunity signal, construction already underway (which
offsets available opportunity), and their own available capital. They do not
speculate about other actors' future plans.

**New construction is the primary mechanism for growing total productive capacity.
Component transfer is the mechanism for reallocating existing capital more
efficiently.** In a healthy market, supply gaps are filled by new entrants, not
by existing buildings transitioning away from profitable recipes.

---

## Exit
> **v2: REWRITE** — kernel decline to epsilon-size, then registered release at recovery fractions (architecture/ownership.md).

A building exits when it cannot sustain average total cost over a sustained
period (default ~1 simulated year). Exit sequence:

1. **Reduced operation**: chosen recipe size drops toward zero. Fixed costs
   continue.
2. **Sustained loss**: if average total cost remains uncovered after the holding
   period, begin exit.
3. **Component release**: components are dismantled or transferred. Land returns
   to the regional market. Irrecoverable components (specialised earthworks,
   expertise) are lost.
4. **Labour release**: employed pops return to the regional labour market.

---

## Capital Vintage and Efficiency Decay
> **v2: CUT** — dormant in v1 code; parking lot until a failing criterion demands it (METHODOLOGY R10).

Without decay, a building constructed in 1840 operates at identical efficiency
in 1920. Over a 200-year simulation this produces unrealistic late-era states.

**Efficiency** is a modifier on outputs only, without changing inputs. An
efficiency of 0.9 produces 10% less output for the same inputs. It applies after
effective recipe size and does not affect cost calculations.

Each building accumulates a slow negative drift in efficiency over time. The
rate is slow — negligible within a decade, meaningful across a generation. The
effect is not year-to-year competitiveness loss but ensuring Victorian-era
capital does not persist at full productivity into much later eras.

**Refresh** resets efficiency toward 1.0. It consumes construction goods and
construction service goods proportional to recipe size, and takes time. During
refresh, chosen recipe size is partially reduced. A building refreshes when the
output loss from continued decay exceeds the refresh cost.

Decay rate, recipe-specificity, and refresh cost calibration are open.

---

## State It Owns
> **v2: REWRITE** — object list changes with the desk/claims model.

**In SimState (runtime):**
- All building instances: id, owner(s) with shares, region, active recipe,
  chosen recipe size, component register (component → quantity held),
  inventory (good → quantity), employed labour (pop group → count),
  optional transfer target (building id or construction project id)
- Per-building efficiency modifier and decay accumulator
- In-progress construction projects: actor(s) with investment shares, target
  recipe, region, components acquired so far, ticks elapsed

**In game data (static):**
- Recipe definitions: inputs with scaling behaviour, outputs with ratios,
  component requirements
- Component definitions: build specification, recovery fraction
- Construction service good definitions and their building definitions

**Scenario-variable:**
- Recipe unlock schedule per region (in state delta schedule)
- Starting building distributions per region
- Starting land ownership per region (aristocrat pop inventories)

---

## Open Questions
> **v2: REWRITE** — several are resolved by v2 decisions (ownership = claims register; opportunity = minting queue).

**Input scaling specification.** The fixed/variable/semi-variable distinction
per input needs a concrete data representation. Semi-variable needs at minimum
two parameters (floor + slope). Must be expressible in scenario data files.

**Ownership mechanics.** Ownership shares are fractional and may be held by
multiple actors (pop groups, governments, financial institutions). Shares are
not tradeable on secondary markets in the base simulation. Profit distributes
pro-rata each tick. Owners receive residual income; production decisions are
made by the building's agent logic independent of ownership. Secondary equity
markets (stock exchanges) are a future extension. See Financial Markets for
how ownership shares are issued and how investment companies hold portfolios
of ownership stakes.

**Opportunity signal formula.** Described qualitatively; not yet specified. Must
account for expected margin, existing and in-progress supply, and a risk term.
Will require calibration runs to tune.

**Efficiency decay calibration.** Rate, recipe-specificity, and refresh cost are
unquantified. Target: negligible within 20 years, meaningful at 80 without
refresh.

**Pop expertise as a component.** Accumulation, decay, and behaviour on exit are
unresolved. See pop design doc.

---

## Known Simplifications
> **v2: KEEP** — except the construction-financing bullet: v2 makes equity-financed minting core (architecture/ownership.md); credit later adds leverage only.

- **Buildings as aggregates**: one building represents all enterprises sharing a
  recipe in a region. Firm-level behaviour is abstracted.
- **Deterministic recipe unlocks**: adoption timing is scripted; economic
  consequences are emergent.
- **No mothballing**: buildings either operate or exit. Holding components at
  zero production while paying maintenance is a natural future extension.
- **Single efficiency modifier**: real capital heterogeneity within a building
  is not modelled.
- **Construction financing not modelled**: working capital during construction
  is implicit. Deferred to Financial Markets.

---

## Calibration Targets
> **v2: KEEP** — destined for criteria.ron; the efficiency-decay target drops with the CUT above.

- A region building its first blast furnace should see local iron prices fall
  within 2–5 simulated years and iron imports decline.
- After the Bessemer unlock, steel should displace iron in downstream recipes
  within 10–20 simulated years in industrialising regions.
- Power loom mills should not be viable in regions with expensive coal even
  when the recipe is unlocked.
- Multi-output byproducts should price near zero before downstream demand
  exists, rising once a downstream recipe unlocks.
- A building below variable cost should set chosen recipe size to zero within
  a few ticks. One below average total cost should exit within ~1 simulated year.
- Efficiency decay: negligible within 20 years, meaningful at 80 without refresh.

---

## Tick-Time Sensitivity
> **v2: KEEP** — unchanged, minus the efficiency-decay line (cut).

All time-denominated quantities normalise by `tick_duration_days / 7.0`:
- Recipe inputs and outputs are per-week
- Construction time is in weeks
- Efficiency decay rate is per-week
- The ~1-year exit holding period is in ticks

Known instability: at very short ticks, the opportunity signal may fire before
the market responds to in-progress construction. The signal must discount
in-progress capacity fully even before completion.

---

## Stability Conditions
> **v2: REWRITE** — becomes enforced asserts (architecture/engine.md).

After each tick:
- Goods conservation: goods consumed in production = goods produced; goods
  consumed in construction = progress made; no goods created or destroyed
  outside recipes and component build/dismantle/transfer
- Component conservation: components transferred from A to B appear fully in
  B's register and are fully absent from A's; no partial accounting
- No building holds negative inventory or negative recipe size
- Efficiency modifier in (0, 1] for all active buildings
- Total labour employed ≤ total labour available in region

---

## Test Coverage Plan
> **v2: REWRITE** — folds into certification + phase gates (PLAN).

- **Unit**: cost calculation at varying chosen recipe size; multi-output ratio
  conservation; component dismantle goods recovery; component transfer
  conservation (full value moves, no loss); efficiency decay and refresh.
- **Scenario**: single building construction, operation, and exit; verify
  component acquisition, output, and release at each stage.
- **Scenario**: component transfer — building A downsizes handloom, transfers
  components to new building B constructing power loom; verify no goods lost
  in transfer; verify A's owners receive proportional stake in B; verify B
  reaches full recipe size faster than without transfer.
- **Scenario**: multi-output — blast furnace coke_iron; coal tar accumulates
  with no demand; aniline recipe unlocks; coal tar price rises and drains.
- **Scenario**: efficiency decay — 80-year run without refresh; verify output
  decline; trigger refresh, verify recovery.
- **Calibration**: opportunity signal triggers construction; multiple investors
  share ownership proportionally; no construction into recipes with negative
  expected margin.

---

## Future Extensions
> **v2: KEEP** — parking lot.

- **Mothballing**: suspending output while retaining components, paying
  maintenance only.
- **Firm abstraction**: multiple buildings under one entity, enabling internal
  transfer pricing and coordinated investment.
- **Skill-differentiated labour**: recipes specifying skill tiers. Depends on
  Pop Evolution.
- **Ownership market**: building shares as tradeable financial goods. Depends
  on Financial Markets.
