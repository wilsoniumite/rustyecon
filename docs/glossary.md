# Glossary

> **v2 triage (2026-07-18).** Per-term verdicts against ARCHITECTURE.md are tagged inline below. New v2 terms (Desk, Claim, Parcel, Perimeter, Tape, Minting, Overflow, Settlement record, Certificate) get definitions when the glossary is redrafted.

Canonical definitions for terms used throughout the simulation. Where a term has
a common everyday meaning that differs from its meaning here, the distinction is
called out explicitly. Terms marked *(conceptual)* exist in documentation and
design discussion but have no direct equivalent as a named type in the codebase.

---

## Core Simulation Types

**Good**  — *v2: KEEP*
Anything that can be held in an inventory and traded on a market. This includes
physical commodities (grain, coal, steel), services (construction capacity,
banking), currencies, financial instruments, and debt. There is no separate
hierarchy for "services" or "currencies" — they are goods with particular
attributes (`movement_type = Local`, `movement_type = Financial`, etc.).
A good that cannot be stored has `shelf_life = instant`; it is destroyed at the
end of each tick if unsold.

**Component**  — *v2: CUT (v1) — capital cost k replaces it; may return with a later module*
A non-market fixed asset installed inside a recipe. Components are not traded —
they cannot appear in any entity's tradeable inventory. They are built on-site
from market goods (and possibly other inputs including construction services),
stored in the recipe's asset register, and released back into market goods
(partially) when the recipe is scaled down or abandoned. Examples: land
(acquired rather than built, but held as a component once purchased), factory
structures, specialised machinery (blast furnace, Bessemer converter, power
loom frames), IP, trained workforce expertise. Component definitions include
their build specification (goods + time to create) and their recovery fraction
(what fraction of build cost is recoverable as goods on dismantlement).

**Recipe**  — *v2: KEEP — plus versioning*
A production method that a building can run. A recipe specifies:
- Goods consumed per unit of throughput (inputs)
- Goods produced per unit of throughput (outputs; may be multiple at fixed ratios)
- Components required to exist in the recipe's asset register at full operation
- Cost behaviour per input (which costs scale with utilization, which are fixed)
Recipes are defined in game data (static). They are not owned by buildings — a
building adopts a recipe, and multiple buildings can run the same recipe.

**Building**  — *v2: REWRITE — becomes Desk (architecture/objects.md)*
The atomic unit of productive capital in the simulation. A building always runs
exactly one recipe. It holds two asset registers: an inventory (goods including
currency, fully liquid) and a component register (non-market fixed assets
belonging to the recipe). A building:
- Runs exactly one recipe at all times
- Holds an inventory of goods including currency
- Holds a component register of non-market fixed assets
- Employs pops (labour is an input to its recipe)
- Makes tick-by-tick decisions: chosen recipe size, input purchases, output
  sell orders
Buildings are owned by one or more actors with fractional shares. Ownership
determines profit distribution; it does not affect production decisions.

**Modifier**  — *v2: KEEP*
A named multiplier applied to a throughput or cost quantity. Modifiers are
sourced from laws, shortage flags, regional attributes, or scripted events.
They compose multiplicatively unless specified otherwise. A building's effective
throughput is the product of its base throughput and all active modifiers on it.
Modifiers are not goods and are not stored in inventories — they are attributes
of the simulation state that are read during the production phase.

**State Delta**  — *v2: KEEP — hardened: canonical order, provenance*
An atomic mutation to the simulation state, produced by a pure system function
and applied by `apply_state_deltas()`. No system function mutates state directly; all
mutations are expressed as state deltas and applied in a single pass. This gives
replay, determinism, and the ability to inspect or intercept mutations before
they are applied. "State delta" is the architectural concept.

**Law**  — *v2: KEEP*
A named policy choice for a region, selected from a fixed small menu of options.
Laws govern trade policy, labour regime, factory regulation, monetary standard,
and other region-level behaviours. Each region has one active value per law
domain. Regions inherit law values from their country's defaults but can hold
overrides. Laws are changed by scripted state deltas or (in future) by political
processes. Laws are distinct from modifiers: a law is a discrete named choice;
a modifier is a numeric multiplier that a law may apply.

**Region**  — *v2: KEEP*
The smallest geographic and policy unit. A region owns buildings, hosts pop
groups, holds regional market prices for all goods, and maintains its own set
of law values (which may override country defaults). All simulation logic that
acts on geography acts on regions. Regions do not aggregate up to countries
for any core simulation calculation; the country's national market node is the
only cross-regional aggregation that matters.

**Channel**  — *v2: REWRITE — operated by a transport desk*
A directed connection between two market nodes along which goods can flow. A
channel has a capacity per tick (set by the throughput of the building occupying
it), a crossing cost (transport, tariff, compliance), and a regulatory factor
(0 = blocked, 1 = free). One slot per transport type per channel prevents
within-channel competition. Channels carry goods; they do not carry components
or state deltas.

**Recipe size**  — *v2: KEEP — renamed size*
The maximum output rate a recipe instance can sustain, set by the components
held for that recipe. Increasing recipe size requires acquiring more components
(construction cost + time); decreasing it releases components back into market
goods at partial recovery.

**Chosen recipe size**  — *v2: KEEP — renamed scale*
How much of its recipe size the building elects to run each tick, in the range
[0, recipe size]. Fixed costs are paid at recipe size regardless; variable
costs scale with chosen recipe size.

**Effective recipe size**  — *v2: KEEP*
What the recipe actually processes after input rationing. Usually equals chosen
recipe size; reduced when market inputs are unavailable. Fixed inputs (land,
structures) are unaffected by rationing; variable inputs fall proportionally.

**SimState**  — *v2: KEEP*
The complete runtime state of the simulation at a single point in time. Contains
all prices, inventories, building states, pop states, and active laws. A
`SimState` is the same type whether it represents tick zero, a mid-run
checkpoint, or a save file. There is no special "starting state" type — that is
a conceptual term for the `SimState` used to initialise a run.

---

## Scenario and Data Concepts

**Scenario**  — *v2: REWRITE — becomes (tape, code version, seed)*
A `SimState` (the initial conditions) paired with a list of `StateDeltas` (the
scripted event schedule). Running a scenario means: start from the provided
state, then at each tick apply any scheduled deltas before running systems.
Scenarios are the primary unit of configuration for experiments. Alternate
history variants are scenarios that differ in their delta schedule.

**Event** *(conceptual in this context)*  — *v2: KEEP — tape entries*
A timeline-fixed trigger in a scenario's delta schedule. An event fires on a
specific date and produces one or more state deltas — law changes, recipe
unlocks, modifier applications, region transfers. Events are scripted, not
emergent. "Event" in this sense is subset form of a "state delta": an event is
the scenario-data record of when and why something changes; a state delta is
the mutation it generates at runtime.

**Game Data**  — *v2: KEEP*
Static definitions that do not vary during a run: good attributes, recipe
definitions, component build specifications, building type definitions, channel
topology, transport type definitions. Game data is loaded once at startup. It
can vary between scenarios (a scenario may use a different good set or different
channel topology), but it does not change during a simulation run.

**Country** *(conceptual)*  — *v2: KEEP*
A grouping of regions that share a national market node and a set of default law
values. Countries exist as a data concept (useful for scenario authoring — "set
all German regions to protective_tariff" — and for visualisation), but the core
simulation does not iterate over or act on countries as entities. A country's
influence on the simulation is entirely mediated through: (1) its national market
node, which aggregates regional supply and demand, and (2) its default law
values, which regions inherit unless overridden.

---

## Conceptual / Documentation Terms

**Starting State** *(conceptual)*  — *v2: REWRITE — genesis is tape*
Informal term for the `SimState` at tick zero of a run. Not a distinct type.

**Transition** *(conceptual)*  — *v2: CUT — goes with components*
A documentation term for the combination of one building downsizing its recipe
(releasing components) and another building constructing a new recipe (acquiring
those components directly via component transfer). Not a simulation construct —
there is no transition object in SimState. Buildings always run exactly one
recipe; the "transition" describes two independent buildings making complementary
decisions, with the source holding an optional transfer_target attribute pointing
to the destination.

**Building size** *(conceptual)*  — *v2: CUT — moot under Desk*
A building's recipe size. "Building size" has no separate meaning — it is just
the recipe size of the building's single recipe.

**Efficiency** *(open)*  — *v2: CUT (v1) — dormant; parking lot*
A modifier on outputs only, without changing inputs. Relevant to capital vintage
decay and recipe-level productivity improvements. How exactly input scaling
works per-input (vs. a single efficiency multiplier) is not yet resolved — see
Production doc open questions.

**Reversible recipe**  — *v2: KEEP (deferred) — returns with the mint*
A recipe that can run at negative chosen size, which swaps inputs and outputs.
The same components serve both directions. Not all recipes are reversible — this
is a per-recipe boolean. The canonical example is the gold mint: positive chosen
size mints gold into currency; negative chosen size redeems currency into gold.

**Passive converter**  — *v2: KEEP (deferred) — returns with the mint*
A building whose chosen recipe size is set by incoming demand rather than by
profit optimisation. It fills all orders at a fixed rate up to its recipe size.
The gold mint is the canonical example. Distinct from all other buildings, which
choose throughput to maximise expected margin.
