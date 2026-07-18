# Architecture

rustyecon is a headless economic simulation: a world of regions, markets, and
simple agents, driven through 1800–2025 by a scripted history, producing
certified, research-grade output. This document is the overview — the parts and
how they fit. Each subsystem has its own specification in `architecture/`.

The seven commitments every design decision answers to:

1. Tick-based, posted price
2. Highly parallelizable
3. A historic world constructed by placing regions and initial conditions
4. Technology, war, and policy as scripted events through history
5. Running 1800–2025
6. Monetary systems and trade. Almost everything is a good.
7. Agents are radically simple — one decision kernel, every constant registered
   in scenario data

---

## The shape

Six runtime modules — **tape**, **desks**, **markets**, **settlement**,
**ownership**, **certify & measure** — plus the offline **worldgen** compiler
that produces the tape. Credit & monetary regimes are a deferred seventh runtime
module with a designed seam.

```
                 ┌────────────────────────────────────────────┐
                 │              worldgen (compiler)            │
                 │  data tables + timelines → validated tape   │
                 └──────────────────┬─────────────────────────┘
                                    ▼
   ┌──────────┐   events   ┌────────────────┐   orders    ┌──────────┐
   │   TAPE    ├──────────▶│     DESKS       ├────────────▶│  MARKETS  │
   │ (history) │           │  (one kernel)   │◀────────────┤ (posted   │
   └──────────┘            └───┬────────┬───┘   prices     │  prices)  │
                               │        │                  └────┬─────┘
                     overflow  │        │ rent                  │ fills
                               ▼        ▼                       ▼
                      ┌────────────────────┐          ┌──────────────────┐
                      │     OWNERSHIP      │          │    SETTLEMENT     │
                      │ claims · parcels · │◀─────────┤ (tagged, conserved)│
                      │   minting queue    │  records └──────────────────┘
                      └────────┬───────────┘
                               ▼
                      ┌────────────────────┐
                      │ CERTIFY & MEASURE  │
                      │ asserts · verdicts │
                      │ dual telemetry     │
                      └────────────────────┘
```

**The tape** ([worldgen.md](architecture/worldgen.md)) is the single input
format: a dated event stream covering genesis, technology, wars, laws, and
population paths. A scenario is (tape, code version, seed).

**Desks** ([objects.md](architecture/objects.md),
[kernel.md](architecture/kernel.md)) are the only actor type: a recipe attached
to an inventory, running one shared decision kernel — producers, transport
operators, land parcels, pops' labour and consumption, governments alike.

**Markets** ([markets.md](architecture/markets.md)) post prices, clear orders
pro-rata, and update prices by imbalance. No within-tick price discovery, no
price clamps, no limit orders.

**Settlement** ([markets.md](architecture/markets.md)) moves goods and currency
per the fills, tags every transaction with whether it crosses an ownership
perimeter, and records who was rationed.

**Ownership** ([ownership.md](architecture/ownership.md)) holds the claims
register and the minting queue: savings overflow becomes new capacity and new
enclosures, claims are minted — never traded — and rent flows back to claim
holders.

**Pops** ([pops.md](architecture/pops.md)) supply labour on a capability ladder,
consume from wealth-tier baskets up to an absorption cap, and move hours across
the market boundary at the self-provision margin.

**Money** ([money.md](architecture/money.md)): currency is a good, conserved
with provenance; trade settles through transport desks; credit and monetary
regimes arrive later through a double-entry postings seam.

**The engine** ([engine.md](architecture/engine.md)) checks itself: conservation
asserts, tested determinism, per-run certificates with pre-registered criteria,
region-sharded parallelism, and Parquet telemetry with the research readouts
(P, Q, R, B, μ, r\*, concentration, dispersal) as first-class columns.

## Tick phases

| phase | owner | reads | writes |
|---|---|---|---|
| 0 tape | tape | schedule | any delta (provenance-tagged) |
| 1 decisions | desks (kernel) | prices, own inventory | orders, scale deltas, rent-payout deltas, overflow pledges |
| 2 clearing | markets | orders | fills, volumes |
| 3 settlement | settlement | fills | inventories, settlement records |
| 4 production | desks | scale, inputs | outputs (provenance Production / SelfProvision) |
| 5 upkeep | ownership + state | — | spoilage deltas, pair redistribution, minting-queue resolution |
| 6 price update | markets | volumes | prices |
| 7 certify & measure | certify | everything | ledger asserts, telemetry, certificate |

All decisions read the state left by the previous tick. Desks chartered in
phase 5 begin trading in the next tick's phase 1.

## Deliberately not built

Each is a named deferral with a seam, not a hole:

- **Credit and banks** — the postings seam is designed
  ([money.md](architecture/money.md)); until it is built, nothing fakes credit.
- **Secondary asset markets** — claims mint only; no trading of existing claims.
- **Component registers** — capital cost k (goods + build time) carries capital
  at this grain.
- **Efficiency decay**, **endogenous technology**, **warfare logic**,
  **individual agents**, **within-tick price discovery**, **multiple currencies
  / FX**, **migration** (population paths are scripted), **cross-bucket labour
  mobility** (pops post to their own capability bucket).

## Reading order

1. [objects.md](architecture/objects.md) — the ontology: goods, recipes, desks,
   pops, parcels, claims, nodes, channels, tape, state
2. [kernel.md](architecture/kernel.md) — the one agent kernel: rules, signals,
   routing, worked traces
3. [markets.md](architecture/markets.md) — clearing, prices, settlement, the
   perimeter ledger
4. [ownership.md](architecture/ownership.md) — claims, minting, growth, μ and r\*
5. [pops.md](architecture/pops.md) — labour, participation, consumption
6. [money.md](architecture/money.md) — currency now; credit and regimes later
7. [engine.md](architecture/engine.md) — certification, determinism,
   parallelism, telemetry
8. [worldgen.md](architecture/worldgen.md) — the tape, the compiler, the era
   ladder

METHODOLOGY.md holds the design rationale and the standing rules (R1–R14);
PLAN.md holds the build sequence.
