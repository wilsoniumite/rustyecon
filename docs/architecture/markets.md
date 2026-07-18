# Markets and settlement

## Posted prices

Last tick's price is this tick's transaction price. All decisions read the
state left by the previous tick; there is no within-tick price discovery and no
ordering dependency among agents. Orders carry quantities only — no limit
prices; reservation behaviour lives in the kernel (scale falls when its
pressure signal says so), not in the order book.

## Clearing

Per (node, good): sum sell orders into supply `s` and buy orders into demand
`d`; then

```
buyer_fill  = min(s / d, 1)
seller_fill = min(d / s, 1)
```

Pro-rata rationing, recorded per buyer class. **Who goes short is a first-class
output** — the distributional consequence of scarcity is the research question,
not a nuisance (METHODOLOGY R8). Volumes are recorded for the price update and
telemetry.

## Price update

```
p' = p · (1 + α · clamp((d − s) / max(d, s), ±1))
d = s = 0  ⇒  imbalance = 0   (price unchanged; quiet markets are routine under stagger)
```

`alpha` is per-good (fast for financial-class goods, slow for wages). No
floors, no ceilings, no clamps to base price: stability comes from the kernel's
structure ([kernel.md](kernel.md)), never from price surgery.

## Settlement

Settlement moves goods and currency per the fills — and tags every transaction:

```rust
struct SettlementRecord {
    good: GoodId, qty: f64, value: f64,
    crossing: bool,        // buyer perimeter != seller perimeter
    flow: FlowClass,       // Consumption | Capital | Ownership
    labor_content: f64,    // direct wage share of the seller's unit cost (one hop)
}
```

`flow` is derived from the buyer, never declared: pop consumption buys →
Consumption; desk input buys and construction-desk buys of k-goods → Capital;
enforcement-desk buys of ω-goods and scripted claim transfers → Ownership.
`labor_content` is the seller recipe's direct wage share of unit cost — the
one-hop dispersal measure; full input-output attribution is an analysis-side
computation over the records.

## The perimeter ledger

A **perimeter** is derived, never stored: the set of inventories reachable from
one pop's claims. Provision that settles inside a single perimeter —
self-provision, a wholly-owned chain — is real activity that money never sees.
Provision that crosses perimeters is the priced zone.

Every aggregate is therefore computed twice, as parallel telemetry columns:

- **measured** — over crossing settlements only: what a statistical office
  would see;
- **true** — over all provision, including self-provision valued at a
  registered parity.

The boundary between the two moves over history — enclosure and
market-conversion pull activity in, displacement pushes hours out — and that
motion is itself output, never an accounting error (METHODOLOGY R9). GDP,
labor share, and participation all exist in both columns; the gap between them
is a first-class research object.
