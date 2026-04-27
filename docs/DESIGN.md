# Design

This document captures the major architectural decisions, design principles, and
conceptual rules of rustyecon. It is the first thing to read before working on
any system. Individual system docs go deeper on their own mechanics; this doc
explains why the system is shaped the way it is.

---

## What This Is

A headless global economic simulation spanning 1836–2036. The primary research
goal is testing redistributive policy (high VAT, UBI) in a high-fidelity
environment. The simulation must be fast enough to run 200 simulated years in
under 24 hours on a single machine, and honest enough that its outputs are worth
interpreting.

It is not a game. There is no player, no win condition, no UI loop. The
simulation runs, produces output, and stops.

---

## Architecture

### Event Sourcing

All simulation systems are pure functions:

```
system(&SimState) -> Vec<StateDelta>
```

No system mutates state directly. All mutations are expressed as state deltas
and applied in a single pass by `apply_state_deltas()`. This gives:
- **Replay**: feed the same starting state and delta sequence, get identical
  results
- **Determinism**: no hidden mutable state
- **Testability**: any system can be tested by inspecting the deltas it produces
- **Introspection**: the delta stream can be logged, filtered, or intercepted

### Tick Structure

Each simulated week (configurable) is one tick. Within a tick, all agents act
on last tick's state simultaneously — there is no ordering dependency.

Prices update after clearing, not during. Last tick's price is this tick's
transaction price. This posted-price model is stable under the bounded-rational
agent model and avoids within-tick price discovery complexity.

### Scale

673+ geographic regions, 100+ countries. Every system must be designed with this
scale in mind. Three-tier market structure (regional → national → world) keeps
channel evaluation tractable (~660K per tick).
One building per recipe per region bounds the building count.

### Output

The simulation is headless. Output goes to Parquet files and checkpoint saves
(same format as starting states). Python notebooks are the primary analysis
interface. A minimal TUI shows live progress. A richer dashboard is a future
extension.

---

## Core Design Decisions

### Everything Is a Good

Services, currencies, financial instruments, debt — all are goods. There is no
separate type hierarchy. Goods vary by attributes not by type. This unifies the market mechanism: one price formula,
one clearing algorithm, one channel infrastructure, across all tradeable things.

Consequence: exchange rates are just the price of one currency good in terms of
another. Credit instruments and perishables are goods with a life attribute and a
recipe that decrements that life. When life is one, they have special recipes for the
last tick. Construction services are goods produced by construction company buildings
that have no cross channel recipe. Nothing needs a special-cased path.

### Channels Are the Only Movement Mechanism

Goods move between markets only through channels. A channel has a capacity,
a crossing cost, a regulatory factor, and one slot per operator type. No good
teleports; no good moves without a channel operator (shipping company, bank,
investment bank) running a building in that slot.

Capital channels use the same graph structure as trade channels but with
different properties: crossing cost is dominated by information asymmetry rather
than transport, regulatory factor responds to capital controls rather than
tariffs, and the operators are financial intermediaries rather than shipping
companies.

This means: geographic isolation is real. A region without a channel to the
world market pays higher prices for imported goods and gets lower prices for
exported ones. A region without capital channels cannot receive foreign
investment.

### Recipes Must Justify Their Existence

A recipe is tied to a production building, a cross channel building, or attached to a good.
It runs every tick. Two production recipes exist only if:
1. Their input mixes are meaningfully different (different goods, or different
   labour-to-material ratio)
2. Their output structures differ (different goods or different multi-output ratios)
3. Their component requirements differ (implying non-trivial conversion cost)
4. The goods pops buy from them are historically non-substitutable on short
   timescales

Cosmetic variation is not sufficient. If two "recipes" would produce identical
outputs from near-identical inputs with near-identical components, they should
be one recipe with a modifier.

Multi-output recipes produce all outputs at fixed ratios. This is intentional:
it creates the historically accurate dynamic where overproduction of one output
depresses its price regardless of shortage in the other (coal tar as waste before
synthetic dyes, hides surplus before industrial leather demand).

### One Building per Recipe per Region/channel

There is at most one building per recipe per region. Buildings are price-takers
in a posted-price market — two buildings with the same recipe in the same region
would see identical prices and make near-identical decisions. Intra-region
competition on the same recipe adds noise without meaningful dynamics.

Split ownership within one building handles profit distribution across multiple
actor groups without needing multiple buildings.

Channels are usually bidirectional, meaning they can have one building in each direction
for the same recipe. That recipe is conceptually input good x output good x, but these
buildings may only buy from node a and sell in node b.

### Recipe Components Aren't Goods (most of the time).

Recipe components are things recipes need to run. They aren't traded so they can
be much more numerous without hampering performance significantly. They are
produced on site from tradeable goods when a recipe is being created or scaled up.
When a recipe is dismantled, recipe goods convert back to tradeable goods at a
recovery rate. They also have a transfer rate, the efficiency by which they can
transfer to another recipe if a transition is occuring. Transfer rate is logically
always at least as high as recovery rate. They also have a construction time. The
construction time of the slowest component dictates the construction time of the
recipe. Some conceptual examples:
1. tools: 1 tool purchased from the market could become 1 tool as a recipe component. When dismantled, it has perfect recovery and perfect transfer. It has a construction time of a single week.
2. Real estate: produced from eg construction services, steel, glass, concrete, tools. Might have a recovery rate of 0.3 and a transfer rate of 0.95. It has a 1 month construction time
3. Precision machinery: produced from construction services, steel, and precision tools, recovery 0.2 and transfer 0.8. It may have a construction time of 2 months.
4. EUV Lithography Machines: recovery is probably very low, 0.1, and transfer might be 1.0 but there are likely no other recipes that could use this component anyways.

### Transitions Of Recipes Are Conceptual

Buildings always run exactly one recipe. A "transition" is a documentation term
for one building making a decision: one downsizing and releasing
components and another constructing and absorbing those components via direct
transfer. There is no transition object in SimState — only the source building's
optional `transfer_target` attribute.

The efficiency advantage of a transition (relative to building from scratch) is
that components transfer directly without going through dismantlement and
reconstruction. The cost savings emerge from the component transfer mechanic,
not from a special transition discount.

### Technology, Is Scripted

Recipe unlocks have historical dates per region. The Bessemer converter becomes
available in Sheffield in 1856, Ruhr in 1863, Lorraine in 1870. The dates are
scenario data, not emergent from research investment. The economic consequences
of adoption are emergent; the timing is not.

This is a deliberate simplification. Technology diffusion is historically
well-documented and region-specific; modelling it as emergent from investment
would require a technology system of comparable complexity to the rest of the
simulation and would not produce more accurate timing.

### Wars Are Scripted Demand Shocks

The simulation does not model warfare. Wars appear in the scenario as:
- Scripted government demand shocks (military purchasing consumes goods)
- Scripted region transfers (territory changes hands on a fixed date)
- Scripted destructions (capital stock reduced in conflict zones)

The economic consequences of war (resource diversion, trade disruption,
demographic effects) are significant and modelled via these mechanisms. The
military and political logic of why wars happen is not.

### Governments Are Scripted

Changes in Laws may come as Delta events, following historical dates, in much
the same way as technology

### Scripted Is For Now

We may implement real dynamics for any scripted system, but for now they are
scripted.

### Capital Allocation Is Not a Posted-Price Market

Investment into productive opportunities is an allocation mechanism, not a
price-clearing market. Opportunities post currency buy orders expressing how
much capital they can absorb profitably. Capital flows to the highest-return
opportunities first. There is no "price of opportunity" that adjusts to clear
the market.

Interest rate discovery happens in the credit instrument markets, where banks
and other financial intermediaries issue instruments as goods with posted prices.
The yield emerges from supply and demand for those instruments. The two
mechanisms are connected but distinct.

---

## Design Principles

### Agents Are Bounded Rational

All agent decisions (building production, channel operator flow volumes, pop
consumption, investment) are based on last tick's observable state. Agents never
simulate other agents. All computation is O(k) where k is the agent's own
information set.

Bounded rationality is not a compromise — it is the accurate model. Real
economic actors have limited information and use heuristics. The aggregate
behaviour of bounded-rational agents with noise produces realistic market
dynamics without the instabilities of perfect-information equilibrium models.

### Noise Desynchronises Agents

Sometimes agents draw a per-tick noise multiplier (~N(1.0, 0.15)) for their
decisions. This prevents all operators from reacting identically to the
same price signal, which would cause oscillation. The noise is not randomness
for its own sake — it is a model of the heterogeneous information, timing, and
risk preferences of real agents.

### Calibration Targets Are Real

Every system doc has a "Calibration Targets" section with specific, testable
stylised facts the system should reproduce. "Inland regions should show higher
prices for imported goods" is a calibration target. "The simulation should be
realistic" is not.

### Deferred Is Not Forgotten

Each system doc explicitly lists what it defers to other systems. A system that
says "labour cost is deferred to Pop Evolution" is not ignoring labour — it is
making an interface contract. The deferred item will be implemented in its
correct home and the interface will be honoured.

---

## System Map

```
Markets
  ↑ prices drive all decisions
  ├── Production (buildings, recipes, components)
  │     └── Financial Markets (ownership, construction funding)
  ├── Pop Needs (consumption demand)
  ├── Pop Evolution (labour supply, migration)
  ├── Government (taxes, laws, transfers, state construction)
  ├── Financial Markets (capital channels, credit instruments)
  │     └── Monetary (base money, central bank, gold standard)
  └── Technology (recipe unlock schedule — scenario data)
```

Systems are not truly independent — they share the SimState and communicate
through market prices. The map above shows primary information flow, not strict
dependency.

### Planned System Docs

- [Markets](systems/01_markets.md) ✓
- [Production](systems/02_production.md) ✓
- [Pops](systems/03_pops.md) stub
- [Financial Markets](systems/04_financial_markets.md) ✓
- [Monetary](systems/05_monetary.md) ✓
- Government
- Pop Needs
- Pop Evolution
- Technology
- Services
- Diagnostics
- Politics (seam only — not implemented)

---

## What We Deliberately Did Not Do

**No individual agents.** Population is modelled as pop groups (aggregates
sharing job, culture, and wealth tier). The dynamics of interest — wage
formation, consumption patterns, migration, political pressure — all emerge from
group-level mechanics without tracking individuals.

**No dynamic technology research.** Recipe unlocks are scripted. The alternative
(modelling R&D investment, knowledge spillovers, patent races) would be a
simulation-within-a-simulation of comparable complexity to everything else.

**No warfare simulation.** Wars are scripted demand shocks and region transfers.
The alternative (modelling military logistics, force attrition, strategic
decision-making) is a different simulation entirely.

**No per-firm strategy.** Buildings are the capital unit; firms are a future
extension. Strategic pricing, vertical integration, and market power are
abstracted: the one-slot-per-channel design gives monopoly rents to the slot
occupant without requiring explicit monopoly modelling.

**No within-tick price discovery.** Financial markets in reality clear
continuously. Our weekly tick collapses this. High alpha for financial goods
compensates. This is a known simplification with known implications.

---

## Scenario Structure

A scenario is a `SimState` (starting conditions) paired with a `Vec<StateDelta>`
(the scripted event schedule). Running a scenario: start from the state, apply
scheduled deltas at their specified ticks, run systems each tick.

Alternate history variants differ only in their delta schedule. Policy
experiments differ in starting laws. The same simulation engine runs all of
them.

Starting states are not special — they are `SimState` objects at tick zero. Save
files and checkpoints are also `SimState` objects. The format is unified.
