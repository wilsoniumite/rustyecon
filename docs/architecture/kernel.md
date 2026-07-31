# The desk kernel

One kernel *shape* serves every actor: a single scale variable, nudged by small
asymmetric multiplicative steps, gated by a dead-band on one local **pressure
signal** σ, evaluated only on the desk's stagger phase
(`(tick + desk.id) % S == 0`). What varies by desk kind is only which σ it
reads and where its overflow routes. There are no other decision mechanisms in
the simulation.

> **Finding from v2 Phase 3, 2026-07-20 — two gaps found before Phase 4
> implements this spec. Both closed 2026-07-31 by the "Price formation" section
> below; the finding is kept in place as the record of why that section exists
> (R14).**
>
> **1. Nothing here says how a posted price reaches equilibrium.** The whole of
> "Why this is stable" below argues about *scale*; [markets.md](markets.md)
> keeps the imbalance price rule unchanged and explicitly delegates — "stability
> comes from the kernel's structure (kernel.md), never from price surgery". That
> is a circular reference, and it is where the defect Phase 3 measured got in:
> the legacy engine posts sell quantities as `1.2 × demand`, which pins the
> imbalance negative regardless of price and deflates the price forever. Rule 1's
> buffer band fixes that for storables — posted supply becomes a function of own
> stock, so the flow error accumulates in inventory where σ can see it, and
> imbalance 0 becomes reachable. The gap is that this is never *stated* as a
> property the kernel guarantees, so nothing checks it.
>
> **2. Rule 1 keeps the same construction for the two cases with no stock to
> integrate the error** — labour (the pair rule, deferred to
> [pops.md](pops.md)) and non-storable services (`scale · last_fill`). Both post
> a quantity keyed to *realized fill*, and both therefore have a strictly
> negative, price-independent fixed point: for labour, measured at
> `imbalance = f − 1` on the legacy engine and predicted to fifteen digits with
> no fitted parameters. `σ_π = (w_posted − parity)/parity` is the only kernel
> mechanism that could arrest it — and `parity` appears **zero** times in `src/`
> and `data/`, with no units given anywhere. If it is real-denominated, uniform
> deflation leaves the ratio unchanged and the pin survives; if nominal, it
> trips R12. The price-level determinacy of the kernel rests on that fork.

## Parameters

All registered in `game_data.ron`; no behavioral constant lives in code
(METHODOLOGY R2).

| param | meaning | default |
|---|---|---|
| `S` | activation stagger period (ticks) | 4 |
| `eta_up` / `eta_dn` | scale step up / down (the phase dial) | 0.04 / 0.05 |
| `dead` | dead-band width on σ | 0.05 |
| `b_out` | output buffer band, in activations of throughput | 2.0 |
| `b_cash` | cash band, in activations of outlay | 6.5 |
| `beta` | within-category consumption logit sensitivity | 1.0 |
| `alpha` | per-good market price step | per good |

## Rule 1 — SELL above the band

Post `max(inventory − b_out·scale·qty_out, 0)` as sell orders, for every
*marketable* output. Non-storable marketable outputs (services): post
`scale · last_fill`, floored at `ε·scale` so a collapsed market can restart;
`last_fill` is an EMA initialized at 1.0. Labour is the one specialization:
its posting is the pair rule ([pops.md](pops.md)) — employed halves post full
supply, unemployed halves post fill-scaled supply, both multiplied by the
participation scale π; the pair *is* Rule 1's fill-stickiness, resolved at pop
grain. Sink outputs (consumption utility) are not goods and are never posted.
No debt gates, no sell caps: the buffer band is the supply smoother.

## Rule 2 — NUDGE scale

```
u = frac(φ · (desk.id + tick/S))          // deterministic Weyl fraction
if σ >  dead : scale ← scale · (1 + eta_up · u)
if σ < −dead : scale ← scale · (1 − eta_dn · u)
scale ← clamp(scale, ε·size, size)
```

The pressure signal σ per desk kind — this table *is* the agent design:

> **SUPERSEDED for producer and transport desks, 2026-07-31 (v2 Phase 4).** The
> row below reads `(band_out − inventory_out)/band_out`. Implementing it showed
> that signal **cannot expand a desk**, and the reason is Rule 1 in this same
> document. The row is left in place as the record (R14); what the engine runs
> is stated under "Producer σ, corrected" below.
>
> **The proof.** Rule 1 posts everything above the band, so the buffer is swept
> flat every tick and inventory at the next decision is exactly
> `band + last tick's production`. Substituting into the row's formula:
>
> ```
> σ = (band − (band + scale·qty_out)) / band  =  −scale·qty_out / (b_out·scale·qty_out)  =  −1/b_out
> ```
>
> With the registered `b_out = 2.0`, **a desk that sells every unit it offers
> reads σ = −0.5**, clears the −0.05 dead-band, and shrinks — every activation,
> forever. Nothing about the desk being healthy enters the arithmetic, and
> `scale` cancels, so shrinking cannot escape it. `desk_buffer_sigma_is_pinned…`
> in `src/kernel/mod.rs` is the standing proof.
>
> **Measured before it was diagnosed.** Under the row as written, every desk in
> `lr_00` falls to `ε·size` inside ~80 ticks and sits there for the remaining
> 920, while flour goes 0.6 → 7,898 and wheat goes to 0. Quantities frozen,
> prices doing all the adjusting.
>
> **The deeper problem is informational**, and no rearrangement of the formula
> fixes it. Once Rule 1 has swept the surplus onto the market, output inventory
> says nothing about *excess* demand: a desk selling everything it makes reads
> identically whether demand is 1× or 100× its output. The quantity signal
> saturates at "sold out". This is the same information problem the price
> formation section already identifies for the two *flow* cases — it turns out
> not to be special to them.

### Producer σ, corrected

Each side of the signal comes from the reading that carries information:

| side | signal | why |
|---|---|---|
| contract | `fill − 1`, where `fill` is the desk's own EMA of the rate it realized on what it posted | Once Rule 1 has offered everything above the band, unsold offers are the only own-state evidence of a glut |
| expand | relative margin, `(Σ p_sell·qty_out − Σ p_buy·qty_in) / reference` | Selling out reveals only `d ≥ s`; what separates 1× from 100× demand is the *price*, and margin is already this table's expansion gate |

```
σ = if fill < 1 − dead  then  fill − 1  else  relative_margin
```

Both are own-state, and note where the fill enters: it steers **scale**, never
the posted quantity. Rule 1 still names only stock and scale, so the invariant
in "Price formation" holds for storables.

The buffer band keeps its job in Rule 1 — it is what makes posted supply a flow
commensurate with a flow of demand, rather than a stock dumped against one — but
it is no longer read as a pressure signal.

| desk kind | σ (pressure to expand) | extra gate |
|---|---|---|
| producer | `(band_out − inventory_out) / band_out` | margin > 0, where margin = Σ p_sell·qty_out − Σ p_buy·qty_in at posted prices |
| transport | same as producer (spread over crossing costs is its margin) | margin > 0 |
| parcel (enclosed), frozen | none — no scale response; events may set scale | — |
| labour margin π ([pops.md](pops.md)) | `(w_posted − parity) / parity`, w at the pop's bucket | — |
| consumption | `(cash − reserve) / reserve` | scale capped at the basket table's top tier (the absorption cap, [pops.md](pops.md)) |
| government | none — scale is set by law entries on the tape | — |
| chartered (construction / enforcement) | none — scale = the funded build rate; retires on completion ([ownership.md](ownership.md)) | — |

Bounds: the labour margin's size is 1 by definition, so π ∈ [ε, 1]; the
consumption desk's size is the basket table's top tier; frozen kinds have no
scale response and ignore the clamp.

## Rule 3 — BUY below the band, route OVERFLOW above it

Evaluated **once per inventory owner per activation** — a pop's desks share one
inventory and one Rule-3 pass, so there is no ordering race between them:

```
reserve  = b_cash · outlay(all resident desks' scale) / S
budget   = max(cash − reserve, 0)
buy inputs for scale activations, cash-capped at budget, pro-rata across inputs
overflow = max(cash − reserve − planned_spend, 0)
```

Overflow routing, by owner kind — this table closes every loop:

| owner | overflow goes to |
|---|---|
| producer / transport / parcel desk | its claim holders, pro-rata — this *is* the dividend system; cash cannot pool in firms, by construction |
| pop (shared inventory) | consumption first (up to the absorption cap), then the regional minting queue ([ownership.md](ownership.md)) |
| government desk | its treasury rule (a law; default: hold) |
| chartered construction / enforcement desk | none — it spends itself to zero and retires |

Special cases collapse into parameters, not code: a natural-resource source is
a frozen producer desk (claims assigned at genesis — to the public pop if
nobody else); a passive converter (the future mint) is a desk whose scale is
set by incoming demand; a government spender is a desk owned by the public pop.

## Why this is stable

Stability is structural, never tuned (METHODOLOGY R1). Four mechanisms, each
with a validated pedigree in the macro-ABM literature:

1. **Buffers as low-pass filters** — the output band absorbs demand shocks
   before they reach scale; the cash band absorbs revenue shocks before they
   reach spending (Lengnick 2013's inventory bands; buffer-stock households in
   Poledna et al. 2023).
2. **Small asymmetric steps** — ±4–5% multiplicative nudges, never
   best-response jumps. The `eta_up/eta_dn` ratio is the known
   phase-transition dial (Gualdi, Tarzia, Zamponi & Bouchaud 2015: the
   hiring/firing asymmetry separates full employment from collapse); it is one
   exposed parameter here, not a dozen implicit ones.
3. **Dead-band gating** — no adjustment inside ±`dead`; kills limit-cycle
   chatter at equilibrium (Mark-0's two-signal gate).
4. **Staggered activation** — synchronous full-strength updates of stiff
   dynamics oscillate as a numerical artifact (explicit Euler with too-large
   steps), not an economic one. Stagger divides the effective step size by S
   and desynchronizes cobweb spirals deterministically — the Weyl fraction
   provides heterogeneous step sizes with zero RNG.

Because a kernel of this class has a genuine collapse phase, the phase diagram
over (`eta_up/eta_dn`, `b_cash`, `S`) must be mapped and committed before any
historical run (PLAN Phase 5): know where the cliff is instead of discovering
it in 1893.

Every one of those four mechanisms acts on **scale**. None of them acts on
price, which is what the next section is for.

## Price formation

> **Added 2026-07-31 (v2 Phase 3.5), closing the gap flagged at the top of this
> file.** [markets.md](markets.md) keeps the imbalance price rule and delegates —
> "stability comes from the kernel's structure (kernel.md), never from price
> surgery" — and this document previously never took delivery. The section above
> argues only about scale. That circular reference is where a real defect got in
> and survived a full corpus of certificates.

**The invariant this kernel must maintain, stated so it can be checked:**

> Every posted quantity is a function of the posting desk's **own state**. No
> desk computes what it offers from another agent's demand, or from the fill it
> most recently received.

That is not a style preference, it is what makes the price signal mean anything.
`imbalance = (demand − supply)/max(demand, supply)` is only a *measurement* if
supply was decided independently of demand. The legacy engine violated this — it
posted `min(stock, 1.2 × demand)` — and the consequence was not subtle: whenever
the cap bound, the imbalance was pinned at `(d − 1.2d)/1.2d = −1/6` **exactly,
independent of price**, so a market clearing 100% of demand reported 16.7% excess
supply and marked its own price down forever. Measured across 10,100 consecutive
ticks with standard deviation exactly zero ([engine.md](engine.md), "The 225-year
horizon"). No price anywhere on that branch reports balance, because the rule is
reading back its own decision.

**Storables — why Rule 1 satisfies it.** `max(inventory − b_out·scale·qty_out, 0)`
names only the desk's own stock and scale. Inventory is a *stock*, so it
integrates the flow error: persistent `d < s` makes it grow without bound,
`σ = (band_out − inventory_out)/band_out` runs negative past `dead`, and Rule 2
nudges scale down until `s = d`. The dead-band cannot mask a persistent flow
imbalance, because the imbalance accumulates somewhere the dead-band is not
looking. A demand-keyed cap has no such integrator — it re-derives supply from
demand every tick, so the error never accumulates anywhere and never gets
corrected.

**The two flow cases — where the invariant is delicate.** Labour and
non-storable services have no stock to integrate anything, so each needs its own
state variable playing inventory's role:

| case | posts | state variable | fixed point if the state variable never moves |
|---|---|---|---|
| labour | pair rule ([pops.md](pops.md)) × π | **π** | `imbalance = f − 1`, strictly negative under any unemployment |
| services | `scale · last_fill` | **`scale`** | `imbalance = √(d/scale) − 1`, negative below capacity |

Both fixed points are price-independent, so neither self-corrects through the
price rule alone: they are corrected only by their state variable moving.
For services that is Rule 2, driven by the producer's own margin. For labour it
is π, driven by the participation margin — which makes σ_π the load-bearing
mechanism, and therefore makes `parity`'s units load-bearing too.

**`parity` is real, not nominal.** [pops.md](pops.md) defines it as the pop's
self-provision wage-equivalent, and the hours a pop withholds "produce
subsistence goods directly into the pop's own inventory at a registered rate."
That registered rate is a physical productivity — goods per hour — so parity is a
**quantity of the subsistence basket per hour**, and the margin is a comparison
between two real quantities:

```
σ_π = (w_posted / P_basket − parity) / parity
```

where `P_basket` is the price of that same basket at the pop's node. It is
registered per pop, in goods, alongside the self-provision rate it is derived
from — never as a currency amount, which would pin a price outside the price
system (R12).

The objection to the real reading is that under *uniform* deflation `w/P_basket`
is unchanged, so π never moves and the labour pin survives. That is true and it
is the right reason to state the invariant above rather than rely on σ_π to
rescue a broken market: uniform deflation of everything is exactly the symptom of
every market carrying a structural pin, which is the legacy engine's disease and
not a state the kernel should be able to reach. Once the goods markets satisfy
the invariant and settle, they supply a stable `P_basket`, a falling nominal wage
is a falling *real* wage, π moves, and labour clears. The nominal reading would
also arrest the pin — by nailing the wage to a constant — but it buys that by
importing the very thing R12 forbids, and it would make the labour market's
behaviour depend on the arbitrary units of the currency.

**What anchors the price level.** Nothing above fixes the absolute level, only
relative prices; the anchor is money demand. Rule 3 holds
`reserve = b_cash · outlay(scale)/S` per owner, so with a bounded money stock `M`
the economy at rest satisfies `M ≈ Σ b_cash · outlay/S`, and since `outlay` is
prices × quantities, the level is pinned against `M·S/b_cash`. This is why
`b_cash` is a phase-diagram axis and not merely a liquidity comfort dial.

**Two things Phase 4 must check rather than assume**, because the kernel is
unimplemented and everything in this section is derivation:

1. **An assertion that no posted quantity reads another agent's demand or fill.**
   The invariant is cheap to state and cheap to enforce; it is expensive to
   discover violated, as this project has now demonstrated at the cost of a
   phase. It belongs in the engine (R3/R5), not in a notebook.
2. **That a settled market actually reaches `imbalance ≈ 0`** rather than merely
   a smaller pin — reported as telemetry, per market, over the scored window.

> **BOTH DONE 2026-07-31 (v2 Phase 4, P4.5)**, as run-certificate batteries —
> `src/certify/invariants.rs`, falsified in `tests/test_10_invariants.rs`.
>
> **B6 — own-state posting.** The invariant is about *how* a quantity was
> computed, which cannot be read off the quantity, so it is established
> **differentially**: last tick's aggregated `supply` and `demand` are the only
> foreign-agent data a decision can reach, so they are perturbed on a clone and
> the decision phase re-run. A rule naming only its own state returns identical
> orders. Sampled every 64 ticks; the clone is discarded, so the check cannot
> touch the delta stream.
>
> It fires on **all 28 legacy scenarios**, and names the mechanism rather than
> merely the fact — on the tracer: `node0/grain posted supply moved 1.200000 →
> 1.386000 on foreign volumes alone`, which is the `1.2 × demand` cap caught in
> the act. It **passes under the kernel**, which is the assertion asked for here.
>
> **B7 — no pinned market.** The question is not whether the imbalance is small,
> since a pin can be small, but whether it *moves*. A market whose imbalance is
> a constant is not clearing at some level; it is reporting a number decided by
> something other than the market.
>
> Written first as whole-window constancy, and **that version was too weak to
> catch the defect it exists for**: the legacy pin holds only *while the cap
> binds*, a long unbroken stretch inside a series that also does other things,
> so "was it constant throughout" answered no and the run passed. As a
> longest-unbroken-run test it reproduces the Phase 3 finding unaided —
> `node0/grain pinned at -0.166667 for 10159 of 11551 ticks unbroken` — which
> [engine.md](engine.md) records as having taken an investigation to find. The
> threshold is a *fraction* of the market's history, not a tick count, so it
> means the same thing at 850 ticks and at 11,550.
>
> **The two arms fail opposite checks, and that is the finding.** Legacy
> violates the invariant everywhere but its lr-window markets are not pinned;
> the kernel satisfies the invariant everywhere and pins markets anyway —
> `node0/services pinned at -1.000000`, which is the signature of *no demand at
> all*, because the consumption desk drives pops to the bottom basket tier where
> services do not appear. Satisfying the invariant is necessary for the price
> signal to mean anything. It is not sufficient for a market to clear.

## Worked traces

One activation of each desk kind, to show the kernel is closed:

**Farm** (producer; recipe: labour_r1 + land-service → wheat). Rule 1: wheat
inventory above `b_out·scale` is posted for sale. Rule 2: σ from the wheat
buffer; expands only if the buffer is drawn down *and* wheat revenue exceeds
labour + land-service cost at posted prices. Rule 3: buys labour_r1 and
land-service (both market goods — the land-service sold by an enclosed
parcel's frozen desk) for `scale` activations, cash-capped; cash above the
band flows to the farm's claim holders.

**Shipper** (transport; recipe: wheat@A + fuel → wheat@B). Buys wheat at node
A and fuel, sells wheat at node B; margin is the A→B spread minus fuel.
Otherwise identical to the farm.

**Labour margin** (pop). σ = (posted wage at the pop's bucket − parity) /
parity. Wage above parity: π nudges up — more of the pair's supply is posted.
Wage below parity: π nudges down and the withheld hours produce subsistence
goods inside the pop's own perimeter ([pops.md](pops.md)). Posting itself is
done by the employed/unemployed pair, both halves scaled by π.

**Consumption desk** (pop). σ = (cash − reserve)/reserve: sustained income
pushes the wealth tier up, sustained shortfall pushes it down; the basket at
the current tier is bought subject to budget; no output is ever posted. At the
top tier (the absorption cap) surplus cash becomes overflow → the minting
queue.

**Enclosed parcel**. Frozen: posts its service flow (Rule 1), never adjusts
scale (no σ), pays revenue to claim holders (Rule 3).
