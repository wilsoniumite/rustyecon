# TAPE — the runtime tape, schema 1

The tape is the document a run is made from, and the only way anything enters a run (ENGINE
E1). It is RON, read by `rustyecon_core::Tape::from_ron` and resolved by
`rustyecon_core::resolve` into a `World` and a genesis `SimState`. This file is the reader's
guide; the contract is `docs/ENGINE.md` §2.6 and §5.

## Where the fields are documented

Every field of every raw type is documented in the rustdoc of `rustyecon_core::tape::raw`
(`cargo doc -p rustyecon-core --open`, then `tape::raw`), which carries
`#![deny(missing_docs)]`. Each field's doc gives its type, its unit (or says it has none), that it
is required, and that it has no default. The top-level fields are on `rustyecon_core::Tape`. An
actor's `spec` and the `Actor(..)` tape action belong to the extension (the agents crate, P0.5),
whose raw types are documented the same way in `rustyecon_agents::spec` (which also carries
`#![deny(missing_docs)]`) and `rustyecon_agents::ext`.

## Schema versions

| Schema | Date | Step | Change |
|---|---|---|---|
| 1 | 2026-09-25 | P0.3 | First version. |
| 1 | 2026-09-25 | P0.5 | The agents' spec, `Scripted(..)`, and tape action, `Actor(SetActive(..))`, are defined. No tape with an agent spec existed before, and core's fields are unchanged, so the number stays 1. |
| 1 | 2026-09-25 | P0.6 | No field changes, so the number stays 1. Two load checks are added (a ledger tolerance must be below 1; a recipe may not name a currency), and a param read by the schedule alone leaves the world's identity (see Params). |

The loader reads its own version only; anything else is refused as a schema error before any
other field is looked at. Since no field has a default, every change to the schema bumps the
number and adds a row here.

## Rules

- **Every field is required, and none has a default.** An `Option` field is written as `None` or
  `Some(..)`; leaving it out is an error, as leaving out any other field is. Unknown fields are
  errors.
- **Keys.** Every entity has a key of one or more of `a-z`, `0-9`, `_`, `.` and `-`. A key is unique
  within its kind; desks and pops share one namespace, and so do events and recurring entries. A
  rename is a new entity. Keyed data is always a list of entries carrying a `key`, never a RON
  map.
- **Order.** Any list may be in any order: the loader sorts every kind by key and numbers it
  densely in key byte order, genesis prices by (node, good) and holdings by (holder, good).
  Reordering, reformatting, comments and CRLF line endings change no id and no hash.
  `Tape::to_ron` writes the canonical order (and drops comments).
- **Numbers.** Every number is finite with a clear sign bit (so not `NaN`, `inf` or `-0.0`), and a
  price is positive. Every dial with a time unit, every behavioural rate and every tolerance is a
  named entry in `params`, referenced by key; dimensionless structural data (recipe
  coefficients, weights, genesis stocks and prices, event quantities) may sit inline under its
  entry's `basis` (R4).
- **Params.** Each has a key, a genesis value, a unit and a basis (`Measured(source, vintage)`,
  `Literature(..)`, `Approximate(..)`, `Fitted(fit)` or `Assumed(..)`). Each must be referenced at
  least once, with the unit it is registered in. A reference is *live* (read at use time, and a
  dated `SetParam` may change it) or *fixed* (turned into structure at load: a shelf life, a
  recurring period, a value a `SetParam` copies, or a ledger tolerance). A fixed param cannot be
  the target of a `SetParam`. A ledger tolerance must be below 1.
- **Schedule params.** A param that only the schedule reads, the value a `SetParam` copies or a
  recurring entry's period, and nothing in the world, belongs to the schedule: it is not in the
  run's state or in `world_id`, and the firings it shapes are in `prefix_id` (ENGINE §2.6). So a
  dial change, a new param with a dated `SetParam` that copies it, keeps every checkpoint taken
  before it fires, and so does a new value for such a param. A param the world also reads (a
  price rate a `SetParam` copies, say) stays registered, and changing it is a world edit.
- **Dates** are `"YYYY-MM-DD"`, proleptic Gregorian. A date maps to the tick it falls in,
  `floor(days·ticks_per_year·10⁴ / 3,652,425)` in integers. No event may be dated before `start`.
- **Currencies.** A node's `currency` good is a currency: `Indefinite`, with `price_rate: None`, no
  market and no genesis price. Every other good has a `price_rate` and a genesis price at every
  node.

## Units

| Unit | Meaning | Per tick (Δ = 1/`ticks_per_year`) |
|---|---|---|
| `Dimensionless` | a ratio, weight or tolerance | the value |
| `FlowPerYear` | a flow of goods | `v·Δ` |
| `RatePerYear` | a continuous draw on a stock, or a price rate | `−expm1(−v·Δ)` as a share, `v·Δ` as a log step |
| `CompoundPerYear` | an effective annual rate ρ | `(1+ρ)^Δ − 1` |
| `FractionPerYear` | an annual fraction δ | `1 − (1−δ)^Δ` |
| `Years` | an EMA time constant τ, a period or a shelf life | `−expm1(−Δ/τ)` as a weight; `round(v·ticks_per_year)` ticks as a span, where 0 does not load |

## Load errors

A tape that does not load names where: `LoadError.path` is a tape path such as
`genesis.prices[village/labour]`, `recurring[pension].every` or
`actors[mill].spec.buy[village/grain].qty`. Errors from the RON parser itself (a syntax error, an
unknown or missing field, a malformed key or date) come with the parser's line and column
instead. The full list is in `docs/ENGINE.md` §2.6.

## Worked example: the gate world

The gate world, `tapes/gate.ron` (written in P0.5, ENGINE §10), is the worked example; read the
file itself for every entry. Its shape, annotated:

```ron
Tape(
  schema: 1,                                   // this loader's version
  header: (name: "gate", start: "1750-01-01", ticks_per_year: 52,
    market: (rule: Imbalance, one_sided: Hold, // both required: no default price rule (N13, F8)
             ema_time_constant: "price.ema_tc"),   // a Years param, live
    ledger: (rel_flow: "ledger.rel_flow", rel_stock: "ledger.rel_stock")),  // Dimensionless, fixed
  params: [   // one entry per dial
    (key: "ledger.rel_flow", value: 1e-12, unit: Dimensionless, basis: Assumed("July REL_FLOW")),
    (key: "price.ema_tc", value: 0.5, unit: Years, basis: Approximate("≈ July 2/53 at 52 ticks/yr")),
    (key: "rate.grain", value: 5.2, unit: RatePerYear, basis: Assumed("July alpha 0.1 per weekly tick")),
    (key: "life.bread", value: 0.0577, unit: Years, basis: Assumed("three weeks")),   // 3 ticks
    (key: "mine.capacity", value: 52.0, unit: FlowPerYear, basis: Assumed("gate world")), …],
  goods: [(key: "bread", life: Years("life.bread"), price_rate: Some("rate.bread")),
          (key: "coin", life: Indefinite, price_rate: None),           // the currency
          (key: "grain", life: Indefinite, price_rate: Some("rate.grain")), …],
  nodes: [(key: "town", currency: "coin"), (key: "village", currency: "coin")],
  channels: [],
  classes: ["households", "pensioners", "producers"],
  actors: [(key: "mill", kind: Desk, class: "producers", home: "town", basis: Assumed("gate world"),
    spec: Scripted((                           // the agents' spec (below)
      active: true,
      recipe: Some((inputs: [("grain", 2.0), ("fuel", 1.0)], outputs: [("bread", 3.0)],
                    capacity: "mill.capacity")),          // a FlowPerYear param, live
      buy: [(node: "village", good: "grain", qty: "mill.buy.grain.village", weight: 0.375), …],
      sell: [(node: "town", good: "bread", qty: Flow("mill.sell.town")),
             (node: "village", good: "bread", qty: AllHeld)],
      spend: Some("mill.spend"),                          // a RatePerYear param, live
      payout: Some((rate: "mill.payout", to: [("workers", 1.0)]))))), …],
  genesis: (basis: Assumed("gate world"),
    prices: [(node: "town", good: "bread", price: 2.0), …],   // every (node, non-currency good)
    holdings: [(holder: "mill", goods: [("coin", 100.0), ("grain", 16.0), ("fuel", 8.0),
                                        ("bread", 3.0)]), …]),
  events: [   // any order: the loader sorts by (tick, key)
    (key: "mine.cut", at: "1760-03-01", basis: Assumed("gate world"),
     act: SetParam(param: "mine.capacity", to: "mine.capacity.cut")),   // the value keeps a basis
    (key: "oven.opens", at: "1768-04-01", basis: Assumed("gate world"),
     act: Actor(SetActive(actor: "oven", active: true))), …],
  recurring: [(key: "pension", first: "1751-01-01", every: "pension.period", last: None,
               basis: Assumed("gate world"), act: Mint(holder: "pensioners", good: "coin", qty: 5.0))],
)
```

Core's own actions are `Mint(holder, good, qty)` and `Burn(holder, good, amount)` (both with
provenance `Event`), `Transfer(from, to, good, amount)` and `SetParam(param, to)`; an `amount` is
`Qty(x)` or `All`. `Actor(..)` carries the extension's action; the agents have one,
`SetActive(actor: "oven", active: true)`, which wakes or puts to sleep a scripted actor.

## The scripted actor's spec

`spec: Scripted((..))` is the agents' only behaviour kind in Phase 0; Phase 2's kinds are new
variants. Every field is required and none has a default:

- `active`: whether it acts from genesis. A dormant actor (`false`) posts and produces nothing
  until a dated `SetActive` wakes it.
- `recipe`: `None`, or `Some((inputs, outputs, capacity))`. `inputs` and `outputs` are
  `(good, coefficient)` pairs, each coefficient finite and positive; no good is on both sides,
  and no currency is on either (R14).
  `capacity` is a `FlowPerYear` param. Each tick it runs at `x = min(capacity per tick, held_k /
  a_k)` (to the last representable bit), burning `a_k·x` of each input and minting `o_l·x` of
  each output. No inputs is an endowment; no outputs is consumption.
- `buy`: `(node, good, qty, weight)` lines, one per (node, good). `qty` is a `FlowPerYear` param;
  `weight` is dimensionless and positive, and the weights, added left to right in (node, good)
  key order, must be exactly 1. Every node must quote in the actor's home currency.
- `sell`: `(node, good, qty)` lines, one per (node, good), with `qty` either `Flow(param)` (a
  `FlowPerYear` param) or `AllHeld`. Lines post in (node, good) key order, each at most what is
  still held after the lines before it.
- `spend`: `Some(param)`, a `RatePerYear` param, exactly when there are buy lines: each tick the
  budget across the buy lines is `share(spend)` of the cash the payouts left.
- `payout`: `None`, or `Some((rate, to))`: each tick `share(rate)` of the actor's cash is split
  across `to`, `(actor, weight)` pairs whose weights, added left to right in actor order (desks,
  then pops, each by key), must be exactly 1. An actor never pays itself.

No cost, floor or budget in the spec is a currency amount (R14): budgets are shares of cash, and
every quantity is a registered flow of goods.

A complete tape with no behaviour (every actor's spec is `()`) is core's test fixture,
`crates/core/testdata/core.ron`; it loads with the empty extension, `rustyecon_core::NoExt`.
