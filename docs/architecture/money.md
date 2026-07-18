# Money and trade

## Currency now

Currency is a good: held in inventories, moved by settlement, priced against
everything else by the market. The total stock changes **only** through tape
events, every change carrying a provenance tag into the conservation ledger
([engine.md](engine.md)). No mechanism creates or destroys currency as a side
effect — not clearing, not intermediation, not any agent rule. A silent money
leak is a panic, not a mystery.

Trade settles through transport desks ([objects.md](objects.md)): the desk
pays sellers at its origin node and collects from buyers at its destination
node. With a single currency per connected component of the channel graph —
the standing simplification — no currency-locality machinery is needed.
Multi-currency worlds and FX arrive with the monetary module below.

Costs are quantities of goods, never fixed currency amounts (METHODOLOGY R12):
every recipe input, crossing cost, enclosure cost, and policy instrument prices
through the market. A cost hardcoded in currency units would pin a price
outside the price system and is forbidden everywhere.

## The credit and regimes seam

Credit, banking, and monetary regimes are one deferred module with its
interface designed now, so that nothing elsewhere has to be unbuilt when it
lands — and so that nothing fakes credit in the meantime (no implicit
forgiveness constants, no balance write-downs, anywhere).

**Representation: double-entry postings.**

```rust
struct Posting {
    debit:  (HolderId, Instrument),
    credit: (HolderId, Instrument),
    amount: f64,
    tag: EventTag,
}
```

over an instrument set {cash, deposits, loans, bonds, gold}. The standing
invariant: every instrument row sums to zero — every asset is someone's
liability. The posting log is the module's delta stream; conservation of the
monetary aggregates is structural, not asserted after the fact.

**Regimes are laws gating the mint.** A monetary regime — specie, gold
standard, gold-exchange, managed float, fiat — is a flag set on which postings
the mint desk may emit and under what coverage constraints. The mint itself is
a passive-converter desk: its scale is set by incoming demand at the statutory
rate. Debasement is a rate change; suspension is a flag change; a currency
crisis is the mint's gold inventory hitting zero. All tape events.

**Banks are desks.** A bank's recipe issues claims-on-cash (deposits) against
currency and deploys the proceeds; maturity and default are posting
transformations with registered recovery fractions. Leverage against claim
collateral is where asset-cycle dynamics enter the model — deliberately here,
under posting discipline, rather than through secondary claim markets
([ownership.md](ownership.md)).

**What the seam promises**: when this module is built, capital allocation does
not change — the minting queue remains the only allocator, and credit adds
leverage to pledges, not a second mechanism. Interest-rate discovery happens in
the priced markets for the credit instruments; the emergent r\* from the
minting queue and the credit yields must be allowed to disagree — their spread
is information, not an error to reconcile.
