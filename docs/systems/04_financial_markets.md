# Financial Markets

## Purpose

The financial markets system allocates capital from holders to productive
opportunities. It determines how savings are intermediated into investment,
what financial instruments exist and how they are priced, and how financial
stress propagates through the economy when borrowers or intermediaries fail.

Financial markets are also the mechanism through which capital crosses
geographic boundaries — the channel infrastructure that carries currency across
regions under financial rather than physical constraints.

---

## Scope

This system covers:
- Financial goods: the full inventory of instruments
- The opportunity signal: how investment demand is expressed
- Capital allocation: how opportunities get funded
- Financial actors: banks, investment companies, PE firms
- Capital channels: how they differ from trade channels
- Credit instrument markets: how yields emerge from prices
- Bank balance sheets: assets, liabilities, solvency, reserve ratio
- Systemic risk: bank runs, default chains, cascading failures
- Government bonds: special-case sovereign credit

Explicitly deferred to Monetary:
- Base money supply and central bank operations
- Reserve requirements as a regulatory instrument
- Gold standard and currency convertibility constraints
- Inflation dynamics and money supply growth

The interface with Monetary: the central bank's policy rate is an input to
financial instrument prices. It is defined in Monetary and consumed here.

---

## Financial Goods

All financial instruments are goods with `movement_type = Financial` (near-zero
crossing cost, subject to capital controls rather than transport costs) and high
`alpha` (fast price adjustment). They hold value across ticks via their maturity
structure rather than physical storage.

**Building ownership shares**
A claim on a fraction of a building's future profits. Issued when a building is
first constructed (proportional to capital contributed) or when a building
upsizes via component transfer (proportional to component value transferred).
Not tradeable on secondary markets in the base simulation — owners hold shares
until the building exits. `shelf_life = indefinite`. Yield is variable: the
building's per-tick profit distributed pro-rata.

**Investment company shares**
A claim on a fraction of an investment company building's portfolio returns.
Issued when the investment company accepts currency from investors. `shelf_life
= indefinite`. Yield is variable, derived from the portfolio's building
ownership returns.

**Low-yield credit** (`low_yield_credit`)
A fixed-payout instrument issued by banks. Face value 1.0 currency, matures in
1 tick. Market price < 1.0; the gap is the implied yield. Low yield because
maturity is instant — there is minimal time-value risk. The primary instrument
for liquid savings. `shelf_life = 1 tick` (converted to currency at maturity
via auto-recipe).

**Mid-yield credit** (`mid_yield_credit`)
Issued by investment banks. Matures in ~5 simulated years (~260 ticks at weekly
granularity). Higher yield reflects the longer commitment and greater default
exposure. `shelf_life = 260 ticks`.

**High-yield credit** (`high_yield_credit`)
Issued by PE-class channel operators. Matures in ~10 simulated years. Highest
yield, highest default risk. `shelf_life = 520 ticks`.

**Sovereign bonds** (`sovereign_bond`)
Issued by governments. Maturity varies (scripted per issuance: 5, 10, 20 years).
Yield determined by market price at issuance. Distinct from private credit
because the issuer is a government entity, not a channel building. Default
mechanics differ (see Government Bonds below).

### Maturity Recipes

Each credit instrument has an auto-firing maturity recipe that converts it to
currency at the end of its shelf life:

```
1.0 low_yield_credit → 1.0 currency   (fires each tick: instant maturity)
1.0 mid_yield_credit → 1.0 currency   (fires at tick of maturity)
1.0 high_yield_credit → 1.0 currency  (fires at tick of maturity)
```

On default (see Solvency below), the maturity recipe fires at a recovery
fraction instead of face value. The instrument holder receives partial currency
rather than full.

---

## The Opportunity Signal

For each (region, unlocked recipe) pair, the opportunity signal is computed
each tick from last tick's observable prices:

```
opportunity_value = max(0, expected_profit_per_unit × max_profitable_size)
```

Where:
- `expected_profit_per_unit` = expected revenue per unit of recipe size minus
  expected variable cost per unit (using last tick's prices for inputs and
  outputs)
- `max_profitable_size` = the recipe size at which expected profit per unit
  falls to zero (accounting for regional input availability and existing supply)

The opportunity signal is public and identical for all observers. Actors do not
speculate about others' plans; they observe the signal and existing in-progress
construction (which partially offsets the opportunity).

A positive opportunity value means capital invested here should earn above-cost
returns. A zero or negative value means the region is at capacity for this
recipe at current prices. Opportunities are always expressed as currency amounts:
"up to X currency could profitably be invested in this recipe here."

---

## Capital Allocation

Investment into productive opportunities is an allocation mechanism, not a
posted-price market. The mechanism:

1. **Opportunities post currency buy orders.** Each positive-opportunity
   (region, recipe) pair posts: "I will accept up to X currency and return
   ownership shares proportional to contribution."

2. **Capital suppliers post currency sell orders** (into the local capital
   allocation pool). Suppliers are: pop groups with surplus currency, investment
   companies with uninvested holdings, banks that have channelled currency to
   this region and now hold it in their destination-side inventory.

3. **Allocation by return signal.** The opportunity buy orders are filled in
   descending order of expected return per unit of capital. The highest-return
   opportunity gets funded first, then the next, until capital runs out or all
   opportunities are funded.

4. **Ownership issues proportionally.** Each capital supplier who contributes
   to an opportunity receives ownership shares in the resulting building
   proportional to their contribution.

5. **Unsatisfied demand.** If total capital supply < total opportunity demand,
   lower-return opportunities go unfunded. They persist and re-post next tick
   (prices may have changed). If total capital supply > total opportunity demand,
   excess currency remains in holders' inventories.

There is no "price of opportunity." The return signal (expected profit rate)
attracts capital but does not adjust based on capital scarcity. Interest rate
discovery happens in the credit instrument markets (below), not here.

---

## Financial Actors

### Banks

A bank is a channel building that operates on capital channels. Its recipe:

**Inputs**: currency purchased from the source market (depositors' savings).

**Outputs**:
1. `low_yield_credit` issued into the source market (the depositor's
   receipt — a financial good held by the investor).
2. Currency delivered to the destination market (the capital the bank deploys).

This is a two-destination recipe: one output (credit) goes to the source market;
the other (currency) crosses the channel to the destination. The bank earns its
margin from the difference: it issues credit at a discount (buys currency for
0.99 on a 1.0-payout instrument) and earns returns on capital deployed at the
destination (building ownership stakes or other credit).

The bank maintains a **reserve** — currency held in its inventory at the source
side, not deployed. The reserve ratio is a building parameter: the fraction of
outstanding low-yield credit always held in liquid currency. Higher reserve ratio
reduces credit creation capacity but reduces bank run risk.

### Bank Runs

Low-yield credit matures every tick. If investors choose not to renew (do not
buy new low-yield credit next tick), the bank must pay out face value from its
reserves. If withdrawal demand exceeds reserves, the bank cannot meet obligations
and enters insolvency.

The bank run mechanism: simultaneous non-renewal by many holders drains reserves
faster than the bank can liquidate its destination-side assets (ownership stakes,
longer-term credit). Illiquidity triggers default even if the bank is
fundamentally solvent.

### Investment Banks

Same structure as banks but issue `mid_yield_credit` (5-year maturity). Lower
bank run risk (credit doesn't roll over every tick) but greater exposure to
borrower default over the longer term.

### PE-Class Operators

Issue `high_yield_credit` (10-year maturity). Highest return, highest default
exposure, no run risk (credit is long-dated).

### Investment Companies

An investment company is not a channel operator. It is a building that:
- Accepts currency from investors and issues equity (building ownership shares
  in itself) proportionally
- Holds an inventory of currency and building ownership stakes
- Each tick, deploys uninvested currency into opportunities via the capital
  allocation mechanism
- Distributes portfolio profits to its shareholders each tick

The investment company's "recipe" is portfolio management. It has no fixed
production inputs or outputs — it participates in the capital allocation
mechanism as a buyer of opportunities, and its returns are whatever the
portfolio earns. Its size is measured by assets under management (total
inventory value of currency + ownership stakes held).

### Government

The government is not a channel building but can issue sovereign bonds directly
into its local market (see Government Bonds). It also participates in capital
allocation as a buyer of opportunities for state-owned construction.

---

## Capital Channels

Capital channels use the same graph structure as trade channels but with
fundamentally different properties.

**Crossing cost**: dominated by information asymmetry rather than transport
cost. In Era 1, a London investor has much less information about a Sheffield
mill's prospects than a local investor — the information gap is the cost. Falls
as: telegraph arrives (faster price signals), accounting standards emerge
(better information quality), banks establish foreign offices (intermediary
reduces asymmetry).

**Regulatory factor**: subject to capital controls rather than trade tariffs.
Can swing sharply — from near-1.0 under the gold standard to near-0.0 under
wartime capital controls. Changed by law, not by infrastructure investment.

**Channel slots**: one per financial actor type per direction. A London–Sheffield
capital channel can have one bank (low-yield), one investment bank (mid-yield),
and one PE operator (high-yield) in each direction simultaneously. Each slot is
occupied by one building instance.

**Directionality**: all channel buildings are unidirectional. Capital flows in
one direction per building. A channel can have a bank flowing capital from A to
B and a different bank flowing capital from B to A simultaneously — useful when
one region has capital surplus and the other has opportunity surplus.

**Idle presence**: a bank can hold a channel slot while currently moving no
capital, paying a small fixed cost. This maintains optionality — if the spread
becomes attractive, the bank can resume flow without rebuilding presence.

### Topology Evolution

In Era 1, capital channels are sparse and centred on London. Cross-regional
capital flows exist but are limited to a few corridors (London–Edinburgh,
London–Manchester, London–Amsterdam). Most regions are capital-isolated.

By Era 3–4, the channel graph expands substantially as joint-stock banking
spreads and telegraph reduces information costs. By Era 6–7, capital channels
cover most of the same graph as trade channels, plus some long-range direct
connections (New York–London, London–Buenos Aires) that have no trade channel
equivalent because capital is weightless.

The channel graph is scenario-variable: which capital channels exist and what
their initial properties are is set in the scenario's game data. Channels do
not self-create — they require a bank building to be constructed on them, which
requires the recipe to be unlocked for that region.

---

## Credit Instrument Markets

Credit instruments (`low_yield_credit`, `mid_yield_credit`, `high_yield_credit`)
trade in standard posted-price goods markets in the region where they are issued.
They are `movement_type = Financial` — they can in principle cross capital
channels, but we treat them as non-transferable in the base simulation (their
value is a claim on a specific bank that may not be accessible from another
region). Secondary trading is deferred.

**Yield from price**: a `low_yield_credit` instrument with face value 1.0 and
market price 0.98 has an implied 1-tick yield of ~2%. Market price adjusts by
the standard price mechanism (alpha × imbalance). When banks issue more credit
than savers want to hold, price falls (yield rises) until the market clears.
When savers have more surplus than credit on offer, price rises (yield falls).

**Pop demand for credit instruments**: pops allocate surplus currency across
financial instruments based on wealth tier and liquidity preference:
- Low wealth: holds currency or low-yield credit only (needs short-term
  liquidity)
- Medium wealth: low-yield and some mid-yield credit
- High wealth: diversified across all maturities and investment company equity

This demand curve is defined per pop wealth tier, analogous to consumption
demand curves for physical goods.

---

## Bank Balance Sheets and Solvency

A bank's assets and liabilities are tracked through its inventory:

**Assets** (in destination-side inventory):
- Currency held (reserves available for deployment)
- Building ownership stakes (valued at current expected discounted returns)
- Other credit instruments purchased (from further intermediation)

**Liabilities** (outstanding credit issued, tracked separately from inventory):
- Low-yield credit outstanding: face value of all unmatured instruments issued

**Solvency condition**:
```
assets ≥ liabilities (at current valuations)
```

**Insolvency trigger**: when asset value falls below outstanding liabilities.
Causes: building failures reducing ownership stake values, cascading defaults
on held credit instruments, or reserve depletion from a bank run.

On insolvency, all outstanding credit instruments issued by the bank have their
maturity recipe modified: instead of converting to 1.0 currency on maturity,
they convert to `recovery_fraction` currency (where recovery_fraction ≤ 1.0,
set by the resolution process). This propagates losses to instrument holders.

### Default Propagation Chain

```
Building fails →
  Bank A's ownership stake value falls →
    Bank A becomes insolvent →
      Bank A's credit instruments pay partial value →
        Bank B (holding Bank A's credit) sees asset impairment →
          Bank B may become insolvent → ...
```

Loops in the credit network (Bank A holds credit from Bank B; Bank B holds
credit from Bank A) are real and create correlated failure risk. They do not
cause simulation loops — each tick processes a finite set of state deltas. Loop
unwind is: one bank defaults, impairing the other, which may cascade further.

---

## Government Bonds

Governments issue sovereign bonds directly into their local capital market
without a channel building. The mechanism:

- Government posts sell orders for `sovereign_bond` instruments (face value,
  maturity, coupon rate defined at issuance)
- Buyers purchase with currency; currency enters government inventory
- At maturity, government must pay face value + accrued interest from its
  treasury (tax revenue)
- If treasury is insufficient at maturity: partial payment (sovereign default),
  reducing the maturity recipe payout

Sovereign bonds can also be purchased by banks and investment companies at the
destination of capital channels — British banks buying Ottoman bonds, for
instance. This requires the sovereign bond to be accessible via a capital
channel from the government's region to the bank's source region.

**Sovereign default** differs from bank default: the government continues to
exist and operate, but its credit standing (the market price of future bond
issuances) collapses. Recovery is slow, requiring years of surplus budgets.

---

## State It Owns

**In SimState (runtime):**
- All financial actor building instances (banks, investment companies, PE firms):
  id, region, channel occupied (if applicable), inventory (assets), outstanding
  credit issued (liabilities), reserve currency held
- Credit instrument holdings per entity: (good → quantity → maturity tick)
- Building ownership stake register: (building id → owner id → share fraction)
- In-progress opportunity fills: (opportunity id → contributors with amounts)
- Sovereign bond issuances: (government id → bonds outstanding → maturity
  schedule)

**In game data (static):**
- Financial good definitions (face values, shelf_life/maturity, alpha)
- Maturity recipe definitions per credit instrument
- Capital channel slot definitions (which actor types can occupy which channels)
- Recovery fraction parameters for each instrument type on default

**Scenario-variable:**
- Capital channel topology per era (which channels exist, initial properties)
- Starting credit instrument distributions
- Pop wealth-tier financial preference curves

---

## Open Questions

**Opportunity return signal formula.** The opportunity value is described
qualitatively. The exact formula — how expected profit is computed from last
tick's prices, how the max_profitable_size is estimated, how in-progress
construction is discounted — is not yet specified. This is the most important
input to the capital allocation mechanism and requires careful calibration.

**Pop financial preference curves.** The allocation of surplus pop currency
across credit tiers and equity is described as wealth-tier-dependent but not
specified. What fraction of high-wealth pop surplus goes to each instrument
type? How does this respond to yield differentials? Needs to be defined before
the system can be calibrated.

**Asset valuation for solvency checks.** Bank solvency requires valuing
ownership stakes. A building's ownership stake value is some function of
expected future profits. Simple approximation: last tick's profit × some
capitalisation multiple. The formula matters for when banks become insolvent
and is not yet specified.

**Secondary credit markets.** Credit instruments are currently non-transferable.
Secondary trading (bond markets) would allow price discovery and liquidity but
requires tracking individual instrument lots. Deferred.

**Monetary system interface.** The central bank policy rate affects credit
instrument prices throughout the system. How this rate enters the price
mechanism — as a floor on yields, a reserve requirement, or direct open market
operations — is defined in the Monetary system doc.

---

## Known Simplifications

- **Credit instruments are non-transferable**: no secondary bond market. In
  reality, bond trading is central to financial history (the Amsterdam Beurs,
  the London gilt market). Adds complexity; deferred.
- **Single recovery fraction**: all creditors of a defaulted institution
  recover the same fraction. In reality, seniority determines recovery order.
- **No insurance**: no modelling of financial insurance (Lloyd's, credit default
  swaps). Relevant from Era 3 onward.
- **Investment company as price-taker**: investment companies deploy capital
  into the highest-return available opportunities without strategic behaviour.
  Real investment banks had significant market power and strategic deal-making.
- **Government is always the bond issuer**: in reality, large infrastructure
  companies (railways) also issued bonds directly to the public. Modelled as
  investment bank intermediation for simplicity; direct issuance is a future
  extension.

---

## Calibration Targets

- In Era 1, capital should flow from London to industrialising British regions
  at a meaningful rate, with almost no cross-border flows outside the
  Netherlands corridor.
- After telegraph (Era 2), cross-border capital flows should increase
  measurably as information costs fall.
- Bank run conditions should be rare in stable eras and emergent during
  demand shocks or cascading defaults — not scripted.
- Sovereign bond yields should reflect fiscal health: governments running
  sustained deficits should see yield rise; governments with surpluses should
  see yield fall.
- A chain of three bank defaults should be possible to trigger from a single
  large building failure if the credit network is sufficiently interconnected.

---

## Tick-Time Sensitivity

- `low_yield_credit` maturity of 1 tick is independent of tick duration — it
  is definitionally "instant" credit that rolls over each tick.
- `mid_yield_credit` and `high_yield_credit` maturities are denominated in
  real time (5 and 10 years) and must be converted to ticks by
  `maturity_years × 52 / tick_duration_weeks`.
- Bank run vulnerability scales with tick duration: at monthly ticks, "1-tick
  credit" represents a month of commitment (more stable); at weekly ticks, it
  represents one week (more fragile). Reserve ratio calibration must account
  for this.
- The opportunity signal is recomputed each tick from current prices; at longer
  tick durations, price signals are more stable and investment decisions less
  reactive.

---

## Stability Conditions

After each tick:
- Total currency across all inventories changes only by: government emission,
  credit maturity payouts, and default haircuts. No currency created or
  destroyed by financial intermediation alone.
- Total face value of outstanding credit instruments ≤ total currency that
  could be returned if all instruments matured simultaneously (a systemic
  solvency condition — violated in a credit bubble).
- No entity holds negative currency.
- All ownership share registers sum to 1.0 per building.
- A matured credit instrument is removed from the holder's inventory and
  replaced by currency (at face value or recovery fraction); no instrument
  persists past its maturity tick without resolution.

---

## Test Coverage Plan

- **Unit**: credit instrument maturity recipe (face value → currency); default
  maturity recipe (face value × recovery_fraction → currency); opportunity
  allocation priority ordering; bank reserve depletion arithmetic.
- **Scenario**: two-region market, bank on capital channel — verify currency
  flows from surplus region to opportunity region; verify credit instruments
  issued at source; verify bank earns spread; verify bank holds correct reserve.
- **Scenario**: bank run — pop non-renewal exceeds bank reserve; verify bank
  insolvency event fires; verify credit instrument holders receive partial
  recovery; verify no currency created or destroyed.
- **Scenario**: default chain — building fails, bank holding ownership stake
  becomes insolvent, second bank holding first bank's credit is impaired;
  verify propagation stops at unconnected actors.
- **Scenario**: sovereign default — government deficit persists until bond
  maturity; verify partial payout; verify subsequent bond issuance at higher
  yield.
- **Calibration**: verify capital flows from London toward high-opportunity
  regions faster than toward low-opportunity regions; verify investment company
  portfolio returns track underlying building profitability.

---

## Future Extensions

- **Secondary credit markets**: bond trading with price discovery, allowing
  market-implied default probabilities to emerge.
- **Seniority and tranching**: different creditor classes with different
  recovery priority on default.
- **Direct infrastructure bond issuance**: large projects (railways) issuing
  bonds directly to the public, bypassing bank intermediation.
- **Financial insurance**: instruments that pay out on counterparty default.
  Relevant from Era 3 (Lloyd's, mutual insurance societies).
- **Central bank open market operations**: central bank buying/selling
  sovereign bonds to affect money supply and interest rates. Defined in
  Monetary but manifests in this system.
- **Equity secondary markets**: building ownership shares become tradeable
  (stock exchanges). Requires tracking individual lot ownership rather than
  fractional shares. Major historical development (London Stock Exchange
  formalised 1801).
