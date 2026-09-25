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
whose raw types are documented the same way.

## Schema versions

| Schema | Date | Step | Change |
|---|---|---|---|
| 1 | 2026-09-25 | P0.3 | First version. |

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
  the target of a `SetParam`.
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

The gate world, `tapes/gate.ron` (written in P0.6, ENGINE §10), is the worked example. Its shape,
from ENGINE §5, annotated:

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
    (key: "mine.capacity", value: 520.0, unit: FlowPerYear, basis: Assumed("gate world")), …],
  goods: [(key: "bread", life: Years("life.bread"), price_rate: Some("rate.bread")),
          (key: "coin", life: Indefinite, price_rate: None),           // the currency
          (key: "grain", life: Indefinite, price_rate: Some("rate.grain")), …],
  nodes: [(key: "town", currency: "coin"), (key: "village", currency: "coin")],
  channels: [],
  classes: ["households", "pensioners", "producers"],
  actors: [(key: "mill", kind: Desk, class: "producers", home: "town", basis: Assumed("gate world"),
    spec: Scripted(( … ))), …],                // the agents' spec (P0.5)
  genesis: (basis: Assumed("gate world"),
    prices: [(node: "town", good: "bread", price: 2.0), …],   // every (node, non-currency good)
    holdings: [(holder: "mill", goods: [("coin", 100.0), ("grain", 16.0)]), …]),
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
`Qty(x)` or `All`. `Actor(..)` carries the extension's action.

A complete tape with no behaviour (every actor's spec is `()`) is core's test fixture,
`crates/core/testdata/core.ron`; it loads with the empty extension, `rustyecon_core::NoExt`.
