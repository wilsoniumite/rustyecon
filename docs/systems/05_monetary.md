# Monetary

> **v2 module status: DEFERRED (PLAN Phase 9+).** v1 built none of this. The design is good and kept: the mint as a reversible passive-converter desk and regimes as laws map cleanly onto v2 (regime flags gate which postings the mint may emit — architecture/money.md).
>
> **v2 triage (2026-07-18).** Section verdicts against [ARCHITECTURE.md](../ARCHITECTURE.md): **KEEP** = survives into v2 (light edits allowed later); **REWRITE** = concept survives, text must be redrafted; **CUT** = does not carry into v2 (may return later; see [PLAN.md](../PLAN.md)). Rewriting happens in the phase that touches each section; these are only the rulings.

## Purpose
> **v2: KEEP (deferred)** — module charter for Phase 9+.

The monetary system governs the supply of base money, the regimes that constrain
or enable its creation, and the mechanisms through which monetary conditions
affect the real economy. It is the foundation beneath the financial markets
system: commercial banks create credit on top of base money, but only the central
bank can create base money itself.

The primary question this system answers is: how much currency exists, who
controls that quantity, and what are the consequences when the answer changes?

---

## Scope
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

This system covers:
- Base money and what distinguishes it from credit
- Central bank buildings and their recipes
- Monetary regime laws and what each implies
- The gold standard mechanism and currency crises
- The policy rate and how it floors credit instrument yields
- The price level as a diagnostic output

Explicitly deferred:
- Credit instrument creation by commercial banks (Financial Markets)
- Government deficit financing and sovereign bonds (Government)
- Exchange rate determination for non-pegged currencies (Markets — currencies
  are goods, exchange rates fall out of supply and demand)
- Inflation targeting as autonomous central bank decision logic (open question,
  see below)

---

## Base Money vs. Credit
> **v2: KEEP (deferred)** — the distinction becomes structural under postings (architecture/money.md).

**Base money** is currency — the good. It is the unit of account and the final
means of settlement in the simulation. All other financial instruments are claims
on base money.

**Credit** (low-yield, mid-yield, high-yield credit instruments) is created by
commercial banks and represents a promise to deliver base money in the future.
Credit instruments are goods; their maturity recipes convert them to base money
at face value (or at recovery fraction on default).

The distinction matters for stability: base money is always worth face value.
Credit can default. A system with a lot of credit relative to base money is
fragile — a wave of defaults converts credit to zero rather than to base money,
which contracts the effective money supply sharply.

Only the central bank can create net new base money. Commercial banks
redistribute existing base money through credit creation and maturity; they
do not change the total stock of base money (only the central bank's recipes
do that).

---

## Central Bank Buildings
> **v2: KEEP (deferred)** — a central bank as a cluster of special desks fits v2 exactly.

The central bank is not a single building. It is a cluster of government-owned
buildings in the financial centre region, each running a distinct recipe. All
are controlled by monetary regime laws.

### Gold Mint
> **v2: KEEP (deferred)** — the reversible passive-converter desk; regime flags gate its postings.

**Recipe**: `gold_convertibility` — a reversible recipe.

```
positive chosen size:  gold → [rate] currency   (minting)
negative chosen size:  [rate] currency → gold   (redemption)
```

A reversible recipe can run at negative chosen size, which swaps inputs and
outputs. The same components serve both directions; no separate recipe or
building is needed. The mint's chosen size is wherever demand takes it within
[−recipe_size, +recipe_size].

The mint is a **passive converter**: its chosen size is set by incoming demand,
not by profit optimisation. It fills all orders at the fixed rate up to its
physical capacity (recipe size). This is distinct from every other building,
which chooses throughput to maximise expected margin. The mint's agent logic
is simply: match demand.

The mint ratio is a recipe parameter set by the monetary regime law.
Citizens, banks, and foreign governments can exchange gold for currency or
currency for gold at this rate.

**Debasement** is a recipe switch: changing the `gold_convertibility` recipe to
one with a higher output rate (e.g., 100 currency per gold → 200 currency per
gold). This doubles the currency supply relative to gold and is exactly how
historical debasements worked. It is a law change that modifies the recipe
parameter, not a special mechanism.

**Currency crisis**: when the mint's gold inventory is depleted, it cannot
honour the currency → gold direction of the recipe. The government must either:
- Suspend convertibility (disable the reverse recipe via law change)
- Borrow gold (a channel flow from another country's reserves)
- Debase (switch to a more favourable mint ratio)

All three outcomes are law changes or market events; none requires special-cased
mechanics.

### Open Market Desk
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

**Recipe**: `sovereign_bond_purchase`

```
inputs:  1.0 sovereign_bond
outputs: [price] currency   (newly created)
```

And the reverse (sterilisation / bond sales):

```
inputs:  [price] currency
outputs: 1.0 sovereign_bond
```

This building creates or destroys base money by buying or selling sovereign
bonds. The price at which it posts orders implies the policy rate: higher bond
price = lower yield = lower policy rate. The open market desk posts standing
orders each tick; the market clears them against willing sellers (commercial
banks, pops holding bonds).

Under the gold standard, the open market desk is constrained: net currency
creation must not push total outstanding currency above the gold coverage ratio
required by the monetary regime law.

Under fiat regimes, the desk can create currency freely. The constraint is
political (inflation expectations, legal mandate) rather than mechanical.

### Discount Window
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

The discount window is not a separate building. It is the open market desk
operating with a standing, unconditional commitment: the desk always posts buy
orders for sovereign bonds at a price implying the policy rate, from any
commercial bank, in any quantity.

A commercial bank that needs reserves sells its sovereign bond holdings to the
open market desk. The desk creates currency to pay. This is mechanically
identical to a normal OMO purchase — the distinction is that OMOs are
discretionary (the desk chooses when to post orders) while the discount window
commitment is permanent and unconditional.

This is the lender of last resort function. A commercial bank facing a run can
always exchange sovereign bonds for reserves at the policy rate, preventing
a liquidity crisis from becoming insolvency. The constraint: the bank must hold
sovereign bonds to pledge. Banks that hold no government debt cannot access the
facility. This incentivizes banks to hold liquid government assets.

The differentiation from what pops do: pops buy `low_yield_credit` (commercial
bank debt, a claim on the bank). The discount window involves commercial banks
selling `sovereign_bond` (government debt) to the central bank. Different goods,
no ambiguity in the clearing mechanism.

Where there is genuinely no central bank (US 1836–1913): no open market desk
building exists. Bank runs cascade without mitigation. The Panics of 1837, 1857,
1873 emerge from the model without scripting.

---

## Monetary Regime Laws
> **v2: KEEP (deferred)** — becomes flags on which postings the mint may emit (architecture/money.md).

The monetary regime is a law at the country level. It determines which central
bank recipes are active, what constraints apply to money creation, and what
exchange rate regime governs the currency.

**`commodity_money`**
Gold or silver IS the currency. No paper money. No central bank in the modern
sense. The mint ratio (gold-to-silver exchange rate) is set by law; both metals
circulate. Relevant for Era 1 in most countries outside Britain.

**`gold_standard`**
Paper currency is convertible to gold at a fixed rate (the mint ratio). The
gold mint building runs both recipe directions. Outstanding currency is
constrained by gold reserves. This is the classical gold standard; it is the
dominant global regime from roughly 1870–1914.

**`gold_exchange_standard`**
Currency is pegged to a reserve currency (e.g., the dollar) which is itself
gold-convertible. Countries hold dollars rather than gold as reserves. The
central bank's gold mint recipe is replaced by a `reserve_currency_peg` recipe:
exchanging domestic currency for the reserve currency at a fixed rate. This is
the Bretton Woods system (1944–1971).

**`managed_float`**
The central bank does not maintain a fixed peg but intervenes in currency markets
to limit volatility. The open market desk buys and sells foreign currencies
(which are goods in our model) to influence the exchange rate. No gold
constraint; the constraint is the stock of foreign currency reserves.

**`fiat`**
No peg, no convertibility. The open market desk operates without a gold
coverage constraint. Currency value is determined entirely by supply and demand.
The policy rate is the primary instrument. This is the dominant regime from
1971 onward.

---

## The Policy Rate
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

The policy rate is the interest rate at which the central bank lends to
commercial banks via the discount window. It sets a floor on all credit
instrument yields through arbitrage:

- If low-yield credit instruments offer a yield below the policy rate, commercial
  banks sell their bond holdings to the central bank (earning the policy rate)
  instead of holding the lower-yielding credit. This pushes credit instrument
  prices down (yields up) until they meet the policy rate floor.
- Commercial banks cannot profitably lend at rates below the policy rate; they
  would earn more by simply holding central bank reserves.

In our simulation, the policy rate is a parameter on the discount window
building, set by the government through law. It transmits to all credit
instrument yields in the jurisdiction within a few ticks (the speed of
transmission is governed by the alpha of financial goods and the capacity of
capital channels).

**Historical note**: in Era 1, the policy rate as a conscious instrument barely
exists. The Bank of England has a discount rate but uses it crudely. Systematic
monetary policy using the rate as a target develops through Era 2–3. The scenario
data should reflect this: Era 1 central banks have a discount rate but change it
infrequently and reactively.

---

## The Price Level and Inflation
> **v2: KEEP** — diagnostic; lands with telemetry (architecture/engine.md), not with the monetary module.

The price level is not a first-class simulation variable but a computed
diagnostic: a weighted average of goods prices across the economy.

```
price_level = Σ (weight_i × price_i)  for all goods i in the basket
```

The basket composition is scenario-defined (reflects the consumption mix of a
typical household in each era). Inflation is the percentage change in the price
level per year of simulated time.

**Inflation emerges** from:
1. Base money creation exceeding goods supply growth (too much currency chasing
   the same goods)
2. Supply shocks reducing goods availability at existing money supply
3. Credit expansion increasing effective purchasing power faster than supply

The price mechanism (alpha × imbalance) already handles the mechanics. The
price level computation is a read-only aggregation over the SimState; it does
not require any new simulation logic.

**Real vs. nominal values**: all simulation prices are nominal. Policy analysis
(UBI, VAT) requires real values — deflated by the price level. This is a
post-processing step in analysis notebooks, not a simulation concept.

---

## Gold Flows and the Balance of Payments
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

Under the gold standard, international trade imbalances produce gold flows that
create monetary effects. This is the price-specie-flow mechanism (Hume, 1752):

1. Country A runs a trade deficit with Country B
2. Currency flows from A to B to pay for imports
3. Under gold standard, excess currency in B is converted to gold at the mint;
   gold flows from A to B
4. A's gold reserves fall → A's central bank must contract currency supply to
   maintain gold coverage ratio
5. A's money supply contracts → prices fall → A's exports become cheaper → trade
   deficit shrinks

In our simulation:
- Currency flows happen automatically via trade channels (paying for imports)
- If Currency_A is gold-backed, the gold mint in A converts excess currency
  returns to gold outflows (holders of A currency in B can demand gold at A's mint)
- A's gold inventory falls → gold coverage constraint tightens → money creation
  constrained

This creates automatic stabilisation under the gold standard: deficits are
self-correcting through monetary contraction. It also creates deflation and
unemployment in deficit countries, which is historically accurate and directly
observable in simulation output.

---

## The Money Multiplier
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

Commercial banks create credit on top of base money. Under a reserve requirement
law, each unit of base money can support `1/reserve_ratio` units of credit.
This credit multiplier is not a fixed parameter — it emerges from bank behaviour.

If reserve requirements are low and credit demand is high, banks expand credit
aggressively (high multiplier). If reserve requirements are high or credit demand
is low, the multiplier is smaller. In a financial crisis where banks hoard
reserves above the required minimum, the effective multiplier collapses even
with the same legal requirement.

Our simulation does not compute the multiplier directly — it emerges from the
interactions of central bank base money creation, commercial bank credit
issuance, and the reserve requirement law.

---

## Historical Monetary Regimes by Era
> **v2: KEEP** — worldgen research; feeds the policy timeline (PLAN Phase 8).

**Era 1 (1836–1880)**: Bimetallism in most countries (gold and silver
circulate at a legally fixed ratio). Britain on the gold standard since 1821.
France, US, Germany on bimetallism. Progressive adoption of gold monometallism:
Germany adopts gold in 1871 (using French reparations gold), US returns to gold
in 1879 after Civil War greenbacks. By 1880, the classical gold standard is
beginning.

**Era 2 (1880–1914)**: Classical international gold standard. Exchange rates
are stable and nearly fixed between gold-standard countries. Capital flows freely.
Bank of England rate is effectively the world's interest rate.

**Era 3 (1914–1945)**: Gold standard suspended in WWI. Interwar attempt to
restore gold exchange standard (1920s) collapses in the Great Depression
(1931–1933). FDR suspends dollar-gold convertibility domestically (1933) and
devalues (1934). World fractures into currency blocs.

**Era 4 (1945–1973)**: Bretton Woods. Dollar pegged to gold at $35/oz; other
currencies pegged to dollar. US runs increasing deficits → dollar glut → gold
drain → Nixon ends convertibility (1971). Transition period to floating rates.

**Era 5 (1973–1991)**: Managed float. Oil price shocks create inflation.
Volcker shock (1979–1982) contracts money supply sharply to break inflation.
Plaza Accord (1985) is a coordinated managed-float intervention.

**Era 6 (1991–2008)**: Inflation targeting becomes standard. Euro created
(1999): a fiat common currency with no gold backing and a supranational central
bank. Divergent fiscal policies within a single monetary policy creates tensions
modelled as scenario data.

**Era 7 (2008–2036)**: Quantitative easing expands base money far beyond
pre-crisis levels without comparable inflation (velocity of money falls).
Digital currencies and central bank digital currencies (CBDCs) emerge.

---

## State It Owns
> **v2: REWRITE** — restates against the postings instrument set when built.

**In SimState (runtime):**
- Central bank building instances per country: gold mint, open market desk,
  discount window (where they exist)
- Gold mint inventory (gold and currency held by the mint)
- Open market desk inventory (sovereign bonds and currency held for OMOs)
- Outstanding discount window loans (receivables from commercial banks)
- Policy rate parameter (on discount window building)
- Total base money outstanding per currency (computed from all entity inventories;
  a diagnostic, not stored separately)

**In game data (static):**
- Gold convertibility recipe definitions (rate is a parameter, not hardcoded)
- Sovereign bond purchase recipe definitions
- Discount window loan maturity and rate structure

**Scenario-variable:**
- Active monetary regime law per country, per era
- Initial gold reserve distributions
- Policy rate starting values
- Gold coverage ratio requirements per regime

---

## Open Questions
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

**Inflation targeting logic.** For central banks in Era 6–7, the policy rate
is adjusted in response to deviations from a target inflation rate. The decision
logic for this is not yet specified. Options: scripted rate changes (state deltas
on a schedule), a simple rule (Taylor rule: rate = neutral_rate + 1.5 ×
(inflation − target) + 0.5 × output_gap), or autonomous agent logic. The
Taylor rule is tractable and historically validated; specifying the output gap
in our model is the hard part.

**Bimetallism and the gold-silver ratio.** The ratio between gold and silver
prices was legally fixed but market prices diverged when new silver deposits
were discovered (1870s). This created arbitrage (Gresham's Law: bad money
drives out good). Modelling bimetallism accurately requires both metals as
goods with their own supply/demand, plus the mint's bidirectional conversion
recipe for each. The arbitrage dynamics are interesting but complex.

**Velocity of money.** In reality, the price level depends on money supply ×
velocity (how often each unit of currency is spent). Our simulation has implicit
velocity: currency circulates through transactions each tick. But we do not
compute it explicitly. In Era 7, QE expanded money supply enormously without
comparable inflation because velocity fell. Whether our model can reproduce
this without an explicit velocity concept is an open calibration question.

---

## Known Simplifications
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

- **One central bank per country**: in reality, the US had the First and Second
  Banks of the US, then no central bank (1836–1913), then the Federal Reserve.
  Absence of a central bank is modelled as: no central bank buildings exist, so
  no lender of last resort and no OMO capability. Bank runs are unmitigated.
- **Policy rate as a step function**: in reality, central banks signal intent
  through communications before changing rates. We model the rate as a law
  parameter that changes discretely.
- **No currency substitution**: in high-inflation regimes, economic actors
  substitute foreign currencies for the domestic currency (dollarisation). Not
  modelled; all domestic transactions use the domestic currency.
- **Gold physically immobile within a tick**: gold flows between countries
  through trade channels; in reality, gold shipments took weeks. Absorbed into
  channel capacity and crossing cost parameters.

---

## Calibration Targets
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

- Under the classical gold standard, exchange rates between two gold-standard
  currencies should remain within the "gold points" (the cost of physically
  shipping gold between countries) — roughly ±1% of the mint par rate.
- A country losing gold reserves at 5% per year should see its credit markets
  tighten measurably (higher yields, lower credit issuance) within 1–2 simulated
  years.
- Debasement (doubling the mint ratio) should produce roughly proportional
  price increases over 2–5 simulated years as the new money circulates.
- A liquidity crisis that the discount window resolves should not produce the
  same asset price collapse as one that it fails to resolve.

---

## Stability Conditions
> **v2: REWRITE** — becomes posting invariants (architecture/money.md).

After each tick:
- Total base money outstanding = sum of all currency holdings across all
  entities. This changes only via: gold mint recipes, open market desk recipes,
  discount window loan creation/repayment, government emission (if any).
- Gold mint inventory: gold held ≥ 0; currency held ≥ 0. Cannot mint more
  currency than gold coverage ratio allows under gold_standard regime.
- Open market desk: sovereign bonds held ≥ 0; currency held ≥ 0.
- Discount window: outstanding loans ≤ recipe size capacity.
- Policy rate ≥ 0 (zero lower bound; negative rates are not modelled in base
  simulation).

---

## Test Coverage Plan
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

- **Unit**: gold mint recipe (gold → currency at correct rate); debasement
  (recipe parameter change → correct new rate); gold coverage ratio check.
- **Scenario**: gold standard with trade deficit — verify gold outflows from
  deficit country, money supply contraction, price deflation, eventual trade
  rebalancing (price-specie-flow).
- **Scenario**: currency crisis — gold reserve depletion, forced convertibility
  suspension, exchange rate float.
- **Scenario**: open market operations — central bank buys bonds, currency supply
  expands, credit instrument yields fall toward policy rate floor.
- **Scenario**: discount window intervention — commercial bank hit by run, central
  bank lends reserves, bank survives; compare to same scenario without discount
  window (bank fails).
- **Calibration**: under gold standard, exchange rate between two currencies stays
  within ±1% of mint par for 20 simulated years of stable trade.

---

## Future Extensions
> **v2: KEEP (deferred)** — module deferred to PLAN Phase 9+; kept as the design record for monetary regimes.

- **Taylor rule for inflation targeting**: autonomous central bank decision logic
  adjusting policy rate in response to inflation and output gap. Requires
  defining output gap in simulation terms.
- **CBDC**: central bank digital currency as a new good alongside commercial
  bank deposits. Has different properties (no default risk, direct relationship
  with central bank, programmable constraints). Relevant from Era 7.
- **Currency substitution / dollarisation**: pops and buildings substituting
  foreign currencies when domestic currency inflates rapidly. Requires allowing
  multiple currencies to serve as media of exchange within a region.
- **SDR / supranational reserve asset**: an international currency created by
  agreement (IMF special drawing rights). Relevant for Era 4–7.
- **Negative interest rates**: zero lower bound is currently enforced. Some
  historical cases (Switzerland, Japan, ECB) had slightly negative rates.
  Removing the floor and observing simulation behaviour is a calibration exercise.
