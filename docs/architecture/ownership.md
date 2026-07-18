# Ownership, minting, and growth

How the economy's capacity grows, who ends up holding it, and where the rate of
return comes from. The mechanism is deliberately minimal: claims are minted by
paying real costs, never traded; rent flows back through the kernel's overflow
rule; and the wedge and the visible rate of return are readouts of the minting
queue, not fitted quantities.

## The loop

1. **Overflow accumulates.** Pops whose income exceeds the absorption cap and
   the cash band generate overflow ([kernel.md](kernel.md) Rule 3) — saving
   with no taste parameter: the day refuses to stretch, so surplus income has
   nowhere else to go.
2. **Overflow is pledged** to the region's **minting queue**, which ranks the
   region's open opportunities by expected yield from posted prices:
   - *Build* capacity for recipe R: yield = margin(R) / cost(k(R))
   - *Enclose* parcel P: yield = `p_service · yield_per_tick / cost(ω(P))`
   Both are per-tick flow returns on the cost of the claim, costs valued at
   posted prices — commensurable by construction, and the ranking never reads
   the emergent rate, so it cannot be circular.
   Best yield funds first until pledges or opportunities run out. Unfunded
   pledges never leave the pop's inventory; unfunded opportunities persist and
   re-rank next tick.
3. **A funded opportunity charters a desk**: a construction desk (for k) or an
   enforcement desk (for ω) — a real, kernel-run desk seeded with the pledged
   currency, whose recipe buys the k/ω goods *on the market* (nothing bypasses
   prices — METHODOLOGY R12) and whose output is capacity delivered to the
   target desk, or the enclosure of the parcel, with provenance `Minting`.
   Claims on the target mint pro-rata to the pledgers as their currency is
   spent. On completion the chartered desk retires.
4. **Rent returns.** Desk overflow flows to claim holders every activation
   (Rule 3), closing the loop.

Constraints that keep the loop honest: opportunities are public and identical
for all observers (no agent models another — METHODOLOGY R13); chartered desks
in progress discount their opportunity fully; growth is equity-financed from
overflow — there is no borrowing until the credit module exists
([money.md](money.md)), and nothing simulates it in the meantime.

## Readouts

- **B (the backlog)** — Σ positive unfunded opportunity value.
- **μ (the wedge)** — the best *unfunded* yield: the return to expanding the
  moneyed economy at the margin.
- **r\*** — the marginal *funded* yield this tick: the emergent visible rate of
  return.

On ticks where a readout is undefined (empty queue, zero pledges, everything
funded), the column carries the last defined value with a staleness flag — a
*declared-missing* convention the certificate distinguishes from NaN
([engine.md](engine.md)).

## Exit and recovery

A desk held at `ε·size` for a registered period releases its size: a registered
fraction of k returns as goods (provenance `Recovery`) and its claims
extinguish. Parcels never die; their claims persist.

## Scripted counterforces

Concentration's counterweights are historical facts, so they enter on the tape,
operating directly on the register:

- **Estate events** — yearly fractional redistribution of claims at registered
  death and retention rates;
- **Expropriation / abolition / land reform** — dated claim transfers;
- **Policy** — taxes on rent flows, a per-capita dividend, changes to ω
  (enforcement subsidies or their withdrawal).

## What this design refuses

No secondary market in claims: overflow cannot bid up existing assets; it can
only build or enclose. This removes the asset-price-spiral oscillator entirely
and makes the concentration dynamics legible — holdings change through minting,
rent, estates, and scripted transfers, each separately measurable. The cost is
real and accepted: no speculative bubbles in claims, no market-priced wealth
revaluations. When that phenomenon becomes the research target, it enters
through the credit module's postings — as leverage against claim collateral —
not by reopening claim trading.
