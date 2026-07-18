# Objects

The complete ontology. Everything in the simulation is one of the objects on
this page or a readout computed from them.

## Good

Anything holdable in an inventory and tradeable on a market node: physical
goods, services, labour, currency. Attributes:

- `base_price` — reference price at genesis
- `alpha` — price adjustment speed per tick (fast for financial-class goods,
  slow for wages)
- `shelf_life` — lot decay: `None` (indefinite) or a tick count; non-storable
  goods (labour, services) expire at end of tick

There is no type hierarchy over goods and no locality flag: where a good can go
is determined entirely by channel topology.

## Recipe

A production method:

- **Inputs** per unit of scale, each with a scaling class — `Variable` (scales
  with scale), `Fixed` (paid at size regardless), or `SemiVariable` (floor +
  slope). Labour is an input like any other.
- **Outputs** at fixed ratios. Multi-output recipes produce all outputs
  together; overproduction of one output depresses its price regardless of
  shortage in another — deliberately, since that is how byproducts behave.
- **Capital cost k** — the bundle of goods (including a construction-service
  good) consumed to mint one unit of desk size, plus a build time in ticks.
- **Version** — technology events publish new versions (lower k, different
  inputs, different labour buckets). A desk adopts a version at minting and
  keeps it for life: sunk capital competes at its operating cost alone, and
  cost inversions bite only through new minting. Diffusion lags fall out of
  this rule; nothing schedules them.

A recipe earns a distinct definition only if its input mix, output structure, or
capital cost differs meaningfully from every existing recipe. Cosmetic variety
is forbidden — it multiplies desks without adding dynamics.

## Desk

The only actor type: a recipe attached to an inventory.

```rust
struct Desk {
    id: DeskId,
    region: RegionId,          // transport desks register in their channel's
                               // from-region (a convention; feeds sharding)
    recipe: RecipeId,          // includes version
    inv: InventoryId,          // may be shared (a pop's desks share one)
    size: f64,                 // capacity; changes only via minting/destruction
    scale: f64,                // the one decision variable, in [ε·size, size]
    last_fill: f64,            // EMA of own sell fill rate; initialized to 1.0
    node_buy: MarketNodeId,
    node_sell: MarketNodeId,
}
```

Desk kinds — producer, transport, parcel, labour margin, consumption,
government, chartered (construction/enforcement) — differ only in which
pressure signal the kernel reads and where their overflow routes
([kernel.md](kernel.md)). **Uniqueness rule**: at most one producer or
transport desk per recipe version per region/channel — price-takers with
identical information make identical choices, so duplication is noise. Pop
desks are per-pop by construction and exempt.

## Pop

A population cohort:

- an **employed/unemployed pair** — the labour-posting entities; sizes
  redistribute by market fill each tick ([pops.md](pops.md))
- one **shared inventory**
- a frozen **capability bucket** (see the labour ladder in [pops.md](pops.md))
- two kernel rule-carriers: the **labour margin** (a participation scale π on
  the pair's posted supply) and the **consumption desk** (basket buys from the
  wealth-tier tables)

Only pops own claims. Only pops consume. Every chain of provision terminates in
some pop's consumption or terminates nowhere.

## Parcel

Land as a first-class stock:

```rust
struct Parcel {
    region: RegionId,
    service_good: GoodId,     // farmland service, urban site service, spectrum…
    yield_per_tick: f64,
    omega: CostBundle,        // one-time enclosure cost: a goods bundle + time,
                              // like k — never a currency amount (R12);
                              // events can scale it
    enclosed: bool,
}
```

k = 0: parcels are never produced, only claimed. An **enclosed parcel operates
as a frozen desk** (recipe `∅ → service_good`, no scale response, size =
yield): it sells its service on the regional market like any producer and pays
its revenue to its claim holders. Land rent is market-priced, and recipes
consume land services as `Fixed` inputs bought at market — no side channel.
Unenclosed parcels serve only self-provision. Parcels never exit; their claims
persist.

## Claim and the ownership register

```rust
struct Claim { asset: AssetId /* Desk | Parcel */, holder: PopGroupId, share: f64 }
struct OwnershipRegister { claims: Vec<Claim> }
```

**Only pops hold claims.** Governments hold through a public pop; firms are
perimeters (colorings of inventories), never owners. Claims are **minted, never
traded**: they come into existence when a real cost is paid — k for desk
capacity, ω for enclosure — and change hands only through scripted transfer
events (estates, expropriation, abolition). There is no secondary market in
claims: this removes the strongest known oscillator in economic ABMs (asset-
price feedback) and makes saving mean what it says — building or enclosing,
not bidding up paper.

## MarketNode and Channel

A **market node** is a posted-price order-book location. Topology is data:
regional nodes and optional aggregation nodes (national, world) are just nodes
with channels between them — tiers are topology, not types.

A **channel** is a directed edge between nodes, operated by transport desks:
one per (channel, good), generated by the world compiler, with `node_buy =
from` and `node_sell = to`. The transport recipe is `good@from → good@to` plus
crossing-cost inputs (fuel, tariff goods, service fees); desk size is that
good's channel capacity; laws modify the recipe (tariffs, embargoes,
capital controls).

## Tape

The single input format: a dated event list. Genesis (the tick-0 world) and
history (technology versions, wars, laws, population paths, claim transfers)
are the same stream, applied in phase 0 of each tick. A scenario is fully
specified by (tape, code version, seed). See [worldgen.md](worldgen.md).

## SimState, GameData, StateDelta

- **GameData** — static definitions: goods, recipes and versions, need
  categories, wealth-tier tables, nodes, channels, regions, and every kernel
  parameter. Loaded once; never mutates during a run.
- **SimState** — the complete runtime state: prices, volumes, desks, pops,
  parcels, claims, inventories. One type serves genesis, checkpoints, and
  saves.
- **StateDelta** — the only mutation mechanism. Systems are pure functions
  `(&SimState, …) -> Vec<StateDelta>`; a single `apply_state_deltas` pass
  applies them. Delta streams are canonically ordered (bit-identical across
  runs), and every delta that creates or destroys goods or currency carries a
  **provenance tag** feeding the conservation ledger
  ([engine.md](engine.md)). The delta enum is `#[non_exhaustive]` with an
  exhaustive in-crate match, so an unhandled variant is a compile error.
