# Markets

## Purpose

The market system determines how goods are priced and how they move between
entities and geographic locations. It is the core of the simulation — all other
systems either produce inputs to markets or respond to market outputs. Price
signals from markets drive investment, production, migration, and government
policy decisions throughout the simulation.

## Scope

This system covers:
- Price formation for all goods in all markets
- Movement of goods between regional, national, and world market tiers
- Currency exchange rate determination
- Shortage detection and signalling

Explicitly deferred to other systems:
- What buildings produce and consume (Production)
- What pops need and buy (Pop Needs)
- How taxes and transfers affect purchasing power (Government)
- How financial instruments are created and mature (Financial Markets)
- How central banks affect money supply (Monetary)

## Goods

Everything tradeable is a **good** — physical commodities, services, currencies,
financial instruments, and debt. No separate type hierarchy. Goods vary by
attributes, not by type.

Confirmed good attributes:
- `base_price` — the reference price used in the price update formula
- `alpha` — price adjustment speed per tick; varies by market type (financial goods adjust fast, wages adjust slow)
- `shelf_life` — continuous spectrum: `None` (instant, cannot be stored), short (days–weeks), long (months–years), indefinite. Determines spoilage recipe.
- `storage_cost_per_tick` — holding cost for storable goods
- `movement_type` — determines which channel code path applies:
  - `Physical` — transport cost scales with distance and infrastructure
  - `Financial` — near-zero transport cost; subject to capital controls
  - `Local` — cannot cross regional borders at all (services)
- `divisible` — continuous quantity (grain) vs. discrete units (ships)

Currencies are goods. Exchange rates are just the market price of one currency
in terms of another, cleared by the same mechanism as all other goods.

**Open question:** How finely to split goods where quality tiers matter
(e.g. basic food vs. luxury food). Current lean: use wealth-tiered demand
curves on a single good rather than separate goods, unless the production
chains genuinely differ.

## Market Structure

Three tiers of market nodes connected by channels:

```
[Regional markets]  ←→  [National / customs-union market]  ←→  [World market]
       ↕                                                               ↑
  (neighbor channels)                                    (country→world channels)
```

- **Regional market**: the smallest unit. Has its own price for all goods,
  especially local services which exist nowhere else. Pops and buildings
  transact here.
- **National market**: an aggregation node. Pools net excess supply and demand
  from member regions. Countries in a customs union sharing a currency may
  effectively merge their national nodes (near-zero crossing cost, high
  capacity between members).
- **World market**: a clearing house for international trade. Has no producers
  or consumers of its own. "World price" is the price at which all countries'
  net export offers and import demands balance.

Regions can connect to:
- Their geographic neighbours (land or sea border)
- Their national market node
- The world market node directly, if they have port/hub access

The world market is not a separate physical place — it is the emergent price
from cross-border channel flows.

## Channels

Channels are the edges connecting market nodes. A channel has:

- `capacity_per_tick` — maximum flow per tick; creates the lag for price
  equalisation. Determined by the throughput of infrastructure buildings
  (shipping companies, banks) occupying the channel's slots.
- `crossing_cost` — cost per unit to cross (transport, tariff, compliance).
  Sets the floor spread at which arbitrage becomes profitable.
- `regulatory_factor` — `0.0` = completely blocked, `1.0` = free flow.
  Modified by laws (trade policy, capital controls, embargoes).
- `currency_pair` — if the channel crosses a currency boundary, the crossing
  involves buying the seller's currency with the buyer's currency. Exchange
  rate determined by forex market supply/demand.

**One slot per transport type per channel.** A given channel (e.g. the
London–New York route) has at most one oil tanker building, one dry-bulk
building, etc. This prevents within-channel competition and the oscillation
it causes. Competition is geographic — between routes — not within a route.
Overland channels may have 2–3 slots given the fungibility of road/rail.

Local goods (`movement_type = Local`) have no outgoing channels. Their
regional price is fully independent of all other markets.

## Price Mechanism

**Posted-price market with lagged adjustment.**

Last tick's price is this tick's transaction price. No within-tick price
discovery. All buyers and sellers act on the same posted price.

After clearing, the price for next tick updates:

```
price_next = price_current × (1 + alpha × clip(imbalance, −1, 1))

where imbalance = (demand − supply) / max(demand, supply)
```

`alpha` is a per-good attribute controlling adjustment speed.  
Prices have no hard floor or ceiling — stability comes from supply response
and storage behaviour, not from clamping. (This is an open design decision;
implications need observation.)

## Tick Flow (Market Phases)

Within each tick, all market decisions use last tick's prices as fixed inputs.
There is no ordering dependency between agents.

**Phase 1 — Decisions** (simultaneous, all based on last tick's prices)
- Each building:
  - Post sell orders for all output currently in inventory, subject to the
    reservation price floor. Always. Withholding output inventory would hide
    supply from the market the same way hiding buy orders hides demand.
  - Choose `chosen_size` based on two signals read from own state:
    - Output inventory level: accumulating output means the market cannot
      absorb current production → reduce chosen_size. Draining output
      inventory means supply is being absorbed → can sustain or increase.
    - Cash level: if cash is falling (output not selling fast enough to cover
      input costs and fixed costs), reduce chosen_size. If cash is healthy,
      sustain.
    - Sustained reduction in chosen_size eventually triggers recipe_size
      reduction (exit/downsize). Sustained healthy state allows upsize.
  - Post buy orders for `chosen_size × input_qty_per_unit`. This expresses
    the building's genuine demand — no cash-hedged adjustment. Buildings
    never buy more inputs than they will consume this tick; input inventory
    is zero in steady state.
  - // TODO: exact chosen_size adjustment formula and the thresholds that
    // trigger recipe_size changes (exit, upsize) are not yet decided.
- Each pop group: compute income; post buy orders for consumption goods
- Channel operator buildings (shipping companies, financial intermediaries):
  observe spreads across their channels; decide buy volume using hedged agent
  logic (see below)

**Phase 2 — Clearing** (per good, per market; parallelisable across nodes)
- Sum all sell orders and buy orders for each (good, market) pair, including
  orders from channel operators committed in Phase 1. Channel operator orders
  appear as supply at the destination node and demand at the source node —
  they are entries in the order book, not a cascading mechanism.
- If supply ≥ demand: all buyers satisfied; sellers hold excess as inventory
- If supply < demand: buyers rationed pro-rata; each gets `supply/demand`
  fraction of their order
- Record imbalance for price update
- Each market node clears independently. No intra-tick inter-node dependency
  exists because all channel volumes were committed from last tick's prices.

**Phase 3 — Transactions**
- Transfer goods and currency per clearing outcomes
- Update inventories

**Phase 4 — Production**
- Buildings produce based on inputs obtained this tick
- Effective recipe size = chosen_size × (actual inputs received / inputs ordered).
  If buy orders were partially filled, throughput scales down proportionally.
- All inputs received are consumed entirely. Input inventory returns to zero.
- Output goes to inventory; available for sell orders next tick

**Phase 5 — Recipes (auto-firing)**
- Time-decay recipes fire: spoilage reduces inventory quantities, credit
  instrument maturity recipes convert instruments to currency (at face value
  or recovery fraction on default), depreciation advances
- Pop satisfaction updated from consumption that occurred in Phase 3

**Phase 6 — Price update**
- Apply imbalance formula from Phase 2
- New prices become next tick's posted prices

## Agent Decision Model

All buildings — production buildings and channel operators alike — use the same
decision framework. The structural difference is which markets they read and
write, not the logic they apply.

A production building buys inputs and sells outputs in the same regional market.
A channel operator buys a good in the source market and sells the same good in
the destination market. Both compute expected margin per unit of activity and
set their throughput proportionally.

**Shared constraints:**
- All computation is O(k) where k is the agent's own information set
- Agents never simulate other agents
- All decisions based on last tick's observable state

**Decision formula:**
```
margin         = revenue_per_unit − cost_per_unit
                 (production:   output_price × qty_out − Σ input_price × qty_in)
                 (channel op:   price_destination − price_source − crossing_cost)

p_term         = kp × clip(margin / reference_price, 0, 1)
d_term         = kd × clip(margin_delta / reference_price, −0.5, 0.5)
target_frac    = clip(p_term − d_term, 0, 1.0)
chosen_size    = target_frac × recipe_size × noise
```

`noise` (~N(1.0, 0.15) per tick) desynchronises agents reacting to the same
price signal, preventing coordinated oscillation.

The D-term dampens overshoot: if margin is already narrowing (others are
responding), the agent backs off rather than piling in.

Buy orders express the building's genuine demand for inputs at chosen_size —
not a hedged or cash-adjusted version of it. A building that wants to run at
chosen_size 10 posts buy orders for 10 units' worth of inputs. The demand
signal must be honest; suppressing it distorts market prices for everyone.
Cash constraints affect next-tick chosen_size decisions, not this tick's signal.

// TODO: exact formula parameters (kp, kd) are calibration targets.
// TODO: whether channel operators maintain a minimum floor volume to keep
// a channel warm (even at near-zero margin) is not yet decided.

## Multi-Seller Markets

One building per recipe per region means at most one local producer of any good
in a regional market. However, incoming channel operators each post sell orders
for the goods they are moving. A regional wheat market may have one local farm
building AND several channel operators all posting wheat sell orders simultaneously.

Each seller independently decides whether to post an order based on its own
reservation price. The market aggregates all sell orders and clears pro-rata
against total demand. A local producer sees last tick's market price and decides
whether it can profitably sell at that price; channel operators do the same. No
seller needs to know how many others are also selling — the posted price encodes
that competition already.

// TODO: reservation price interaction when local production cost is above
// posted price but incoming channel supply is still profitable at that price.
// Does the local producer drop out entirely (post zero sell order) while
// channel supply continues to fill demand? This should fall out naturally
// from individual reservation price logic but needs verification in testing.

## Reservation Prices

Sellers will not sell below their variable cost of production. If the posted
price is below this floor, they offer zero supply and hold inventory.

Fixed costs are paid regardless of throughput. A building operating below
variable cost will eventually exit (reduce to zero level), but does not
immediately cease. This models the difference between short-run shutdown
(price < variable cost) and long-run exit (price < average total cost).

## Shortage Mechanism

When `demand > 2 × supply` for a good, a shortage condition is flagged.
Buildings that cannot obtain a critical input experience throughput penalties
that cascade through supply chains. Short-run efficiency gains partially offset
shortages (waste reduction, overtime), modelled as a small throughput bonus
when the shortage flag is active.

## Inventories

All entities — pop groups, buildings, shipping companies, banks, governments —
hold explicit inventories. Inventory is a sparse mapping from good to quantity.
Most entries are zero; only non-zero entries are stored and processed.

The decision to store rather than sell is governed by the reservation price.
A building with excess output holds it if the current price is below their
minimum acceptable price, waiting for conditions to improve.

## Static vs. Scenario Data

**In `game_data` (static, not in save state):**
- Good definitions: base_price, alpha, shelf_life, movement_type, divisibility
- Channel topology: which markets are connected, slot counts per channel type
- Transport type definitions

**In `SimState` (runtime, varies tick to tick):**
- Current price of every good in every market
- All entity inventories
- Channel occupancy (which building occupies which slot)
- Channel capacity (derived each tick from occupying building throughput)

**Scenario-variable:**
- Which goods exist (a scenario could omit financial instruments entirely)
- Starting prices
- Which channels exist (a landlocked scenario has no maritime channels)
- Regulatory factors (trade policy laws set these)

## Known Simplifications

- **Single world market node**: in reality, world prices differ by location
  (e.g. Brent vs WTI crude). We aggregate to one world price per good.
- **Pro-rata rationing**: real markets don't ration this simply; priority
  queues, relationship-based allocation, and auction mechanisms all exist.
  Pro-rata is neutral and tractable.
- **One-tick transit**: all channel crossings take exactly one tick. In reality
  transoceanic shipping takes weeks; this is absorbed into the alpha/capacity
  parameters rather than modelled explicitly.
- **Agent decision rules are heuristic**: real traders use more sophisticated
  models. Our agents use PID-style rules with noise. Aggregate behaviour should
  still be approximately correct.
- **No within-tick price discovery**: real markets (especially financial) clear
  continuously. Our weekly (or configurable) tick collapses this. For fast
  markets, a higher alpha compensates.

## Calibration Targets

- Price of a good at a well-connected port region should track world price
  within ~5% under normal conditions.
- Inland/landlocked regions should show systematically higher prices for
  imported goods, proportional to infrastructure quality.
- After a 30% supply shock to a storable good, price should return within
  20% of pre-shock level within 1–2 years of simulated time.
- Non-storable goods (services) should show higher price volatility than
  storable goods given the same demand variance.
- Shipping company profit margins should compress toward operating cost as
  more capacity enters a profitable route.

## Tick-Time Sensitivity

`alpha` is the primary sensitivity lever. All rate-based changes must be
normalised by `tick_duration_days / 7.0` so that behaviour is consistent
across tick durations.

Known sensitivities:
- Very short ticks with high alpha may cause price oscillation (alpha should
  scale down as tick duration shrinks).
- Storage dynamics are sensitive to tick duration: spoilage rates and
  inventory accumulation must be normalised per-tick.
- Channel flows are bounded by capacity per tick; at longer tick durations,
  more can cross per tick (capacity should be specified per-week and scaled).

## Stability Conditions

After each tick the following should hold:
- Total money (all currencies, all entities) changes only by: government
  emission/destruction events and recipe outputs.
- No entity has negative inventory for any good.
- Price of any good is strictly positive.
- A good with `movement_type = Local` has the same price after clearing as
  it would if no channels existed.

## Test Coverage Plan

- **Unit**: price update formula; pro-rata rationing arithmetic; channel
  crossing cost deduction; reservation price floor.
- **Scenario**: two-region market with shipper converges spread toward op_cost
  within N ticks; local service prices diverge freely across regions; demand
  shock propagates to adjacent markets within expected tick window.
- **Sensitivity**: run identical scenario at 7-day, 14-day, 30-day tick
  durations; verify that equilibrium prices and convergence time scale correctly.
- **Property**: for any positive supply and demand, cleared price is strictly
  positive; total goods conserved across clearing.

## Future Extensions

- **Quality tiers**: if production chains genuinely diverge (basic vs. luxury
  goods requiring different inputs), separate goods per tier. Not needed while
  wealth-tiered demand on a single good suffices.
- **Auction mechanisms**: priority-based or bid-based clearing instead of
  pro-rata rationing, for financial markets where order priority matters.
- **Multi-hop routing**: shipping companies optimising multi-leg journeys.
  Currently each leg is decided independently; a coordinated multi-hop company
  would improve efficiency at the cost of decision complexity.
- **Price manipulation / monopoly behaviour**: government price caps are
  already supported via regulatory_factor. Firm-level price setting (monopoly
  rents) deferred but the one-slot design naturally supports it — a monopolist
  on a channel earns the full spread above op_cost.
- **Trade finance**: explicit modelling of credit used to finance goods in
  transit. Currently absorbed into building working capital. Explicit credit
  would allow trade finance crises to emerge.
