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
| 1 | 2026-09-26 | P0.9 | No field changes, so the number stays 1. Two dated events that fall in one tick now fire in date order, not key order (see Dates). |
| 1 | 2026-09-26 | P2.0.1 | The agents' spec gains four variants, `Provider`, `Workers`, `GoodDesk` and `MachDesk` (see below). No existing field or variant changes, and every schema-1 tape loads and means what it did, so the number stays 1: ENGINE §5 has the spec an enum so that Phase 2's kinds are new variants, not a new schema shape. |
| 1 | 2026-09-26 | S2.2 | No field changes, so the number stays 1, and every tape loads and means what it did. A registered param that becomes a `SetParam`'s source keeps the world (see Params), and each use of a param now carries the conversion it takes, so every `world_id` changed once (ENGINE, amended at S2.2). `rustyecon registry` lists each use of a param with its method, per-tick value and path. |
| 1 | 2026-09-27 | P2.1.1 | The agents' spec gains four variants, `BasketProvider`, `BasketWorkers`, `CategoryDesk` and `TypeDesk` (see below). No existing field or variant changes, and every schema-1 tape loads and means what it did, so the number stays 1, as at P2.0.1. |
| 1 | 2026-09-28 | P2.2.1 | The agents' spec gains three variants, `Maker`, `CapacityDesk` and `OwnerDesk` (see below). No existing field or variant changes, and every schema-1 tape loads and means what it did, so the number stays 1, as at P2.0.1 and P2.1.1. |
| 1 | 2026-09-26 | S2.3 | Core's actions gain `ScalePrice(node, good, by)`, appended last, a dated price shock: the posted price times a `Dimensionless` param the schedule reads (see Actions). No existing field or variant changes, and every tape loads and means what it did, so the number stays 1. Certify's kick is this action (docs/CERTIFY.md §2.4, §7). |
| 1 | 2026-09-29 | L0.4 | The maker gains an optional field, `reserve` (`Option` of a `Dimensionless` param, the reservation; see below), after `cover`. Absent, it is off and the canonical form omits it, so every tape keeps its text, `tape_hash`, `world_id` and meaning, and the number stays 1. It is the agents' spec's one field with a default, as criteria's `price_shocks` is certify's (docs/probe/HORSES-RULES.md §10). |
| 1 | 2026-09-30 | P2.2b.1 | A good gains an optional field, `untraded` (a `bool`, absent `false`; see Currencies and untraded goods), and the type desk, the maker and the capacity desk an optional `plant` (see below), each last. Absent, each is off and the canonical form omits it, so every tape keeps its text, `tape_hash`, `world_id` and meaning, and the number stays 1. `untraded` is core's one field with a default (docs/probe/LOOPS-RULES.md §1). |

| 1 | 2026-09-30 | P2.3.2 | The category desk gains two optional fields, `tail` (`Option` of a `Dimensionless` param) and `reserved` (a list of `(good, coef)`), and the basket provider one, `more` (a list of `(to, heads)`), each last (see below). Absent, each is off and the canonical form omits it, so every tape keeps its text, `tape_hash`, `world_id` and meaning, and the number stays 1 (docs/probe/WALL-RULES.md §1). |
| 1 | 2026-09-30 | P2.3.6 | The basket workers gain an optional field, `exit` (`Option` of `(good, gross, floor, plot, commons, land)`), last (see below). Absent, it is off and the canonical form omits it, so every tape keeps its text, `tape_hash`, `world_id` and meaning, and the number stays 1 (docs/probe/COMMONS-RULES.md §1). |
| 1 | 2026-09-30 | P2.4.5 | The workers' `exit` gains an optional field, `pace` (`Option` of `(adjust, share)`), last (see below). Absent, it is off and the canonical form omits it, so every tape keeps its text, `tape_hash`, `world_id` and meaning, and the number stays 1 (docs/probe/TRAP-RULES.md §1). |

The loader reads its own version only; anything else is refused as a schema error before any
other field is looked at. Since no field has a default, every change to the schema bumps the
number and adds a row here; the one exception is a field whose absence is the older meaning, the
maker's `reserve` (L0.4), a good's `untraded` and a desk's `plant` (P2.2b.1), and the category
desk's `tail` and `reserved` and the provider's `more` (P2.3.2), the workers' `exit` (P2.3.6),
and the exit's `pace` (P2.4.5), which add a row and keep the number.

## Rules

- **Every field is required, and none has a default**, but those whose absence is the older
  meaning: the maker's `reserve`, a good's `untraded`, a desk's `plant`, the wall's `tail`,
  `reserved` and `more` (P2.3.2), the workers' `exit` (P2.3.6) and its `pace` (P2.4.5). An `Option` field is
  written as `None` or `Some(..)`; leaving it out is an error, as leaving out any other field is.
  Unknown fields are errors.
- **Keys.** Every entity has a key of one or more of `a-z`, `0-9`, `_`, `.` and `-`. A key is unique
  within its kind; desks and pops share one namespace, and so do events and recurring entries. A
  rename is a new entity. Keyed data is always a list of entries carrying a `key`, never a RON
  map.
- **Order.** Any list may be in any order: the loader sorts every kind by key and numbers it
  densely in key byte order, genesis prices by (node, good) and holdings by (holder, good).
  Reordering, reformatting, comments and CRLF line endings change no id and no hash.
  `Tape::to_ron` writes the canonical order (and drops comments).
- **The tape's hash.** Certify's `tape_hash` is FNV-1a 64 over `Tape::to_ron` (since S2.3;
  docs/CERTIFY.md §3). It covers every field the loader reads: the name, every basis text, the
  schedule and every inline number, and so all that `world_id` and `prefix_id` cover. What the
  loader ignores (comments, whitespace, CRLF, list order) does not move it. A manifest and a
  certificate name their tape by it, and a cli resume needs the `tape_hash` of the run that made
  its checkpoint (since S2.4).
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
- **Schedule params.** A param that only the schedule reads, the value a `SetParam` copies, a
  recurring entry's period or a `ScalePrice`'s factor, and nothing in the world, belongs to the
  schedule: it is not in the
  run's state or in `world_id`, and the firings it shapes are in `prefix_id` (ENGINE §2.6). So a
  dial change, a new param with a dated `SetParam` that copies it, keeps every checkpoint taken
  before it fires, and so does a new value for such a param. A param the world also reads (a
  price rate a `SetParam` copies, say) stays registered, and changing its value is a world edit.
  Making it a source is not: since S2.2 the flag that makes it fixed is not in `world_id`.
- **Dates** are `"YYYY-MM-DD"`, proleptic Gregorian. A date maps to the tick it falls in,
  `floor(days·ticks_per_year·10⁴ / 3,652,425)` in integers. No event may be dated before `start`.
  Events that fall in one tick fire in date order, and events of one date in key order; a
  recurring entry's occurrence counts as dated the first day of its tick. So the order of two
  events, and what a tape means, does not change with `ticks_per_year` (since P0.9).
- **Currencies and untraded goods.** A node's `currency` good is a currency: `Indefinite`, with
  `price_rate: None`, no market and no genesis price. A good marked `untraded: true` (amended at
  P2.2b.1) is held, minted and burned but never traded: `Indefinite`, with `price_rate: None`,
  no node's currency, no market and no genesis price; no order, role good or `ScalePrice` may name
  it, and its book slot stays 1 and is never written. Each refusal names its path
  (`goods[<key>].life`, `.price_rate` or `.untraded`, `genesis.prices[<node>/<key>].good`). Every
  other good has a `price_rate` and a genesis price at every node; a good with neither a price
  rate nor the flag is still refused (`NoPriceRate`), so a typo is caught.

## Units

| Unit | Meaning | Per tick (Δ = 1/`ticks_per_year`) |
|---|---|---|
| `Dimensionless` | a ratio, weight or tolerance | the value |
| `FlowPerYear` | a flow of goods | `v·Δ` |
| `RatePerYear` | a continuous draw on a stock, or a price rate or scale step | `−expm1(−v·Δ)` as a share, `v·Δ` as a log step; the use decides which, and `rustyecon registry` lists each use (since S2.2) |
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
  events: [   // any order: the loader sorts by (tick, date, key)
    (key: "mine.cut", at: "1760-03-01", basis: Assumed("gate world"),
     act: SetParam(param: "mine.capacity", to: "mine.capacity.cut")),   // the value keeps a basis
    (key: "oven.opens", at: "1768-04-01", basis: Assumed("gate world"),
     act: Actor(SetActive(actor: "oven", active: true))), …],
  recurring: [(key: "pension", first: "1751-01-01", every: "pension.period", last: None,
               basis: Assumed("gate world"), act: Mint(holder: "pensioners", good: "coin", qty: 5.0))],
)
```

Core's own actions are `Mint(holder, good, qty)` and `Burn(holder, good, amount)` (both with
provenance `Event`), `Transfer(from, to, good, amount)`, `SetParam(param, to)` and, since S2.3,
`ScalePrice(node, good, by)`; an `amount` is `Qty(x)` or `All`. `ScalePrice` multiplies the posted
price of (node, good) by the value of `by`, a `Dimensionless` param, once, in the tick its date
falls in, and leaves the EMA alone. The factor must be finite and positive, the good must have a
market there (not a currency), and only a dated event may carry it: in a recurring entry it does
not load, since a periodic price nudge would be an exogenous stabiliser (R3). Dense dated shocks
are a periodic nudge in all but name, so a certified run fails when it fires more `ScalePrice`
events than its criteria register, none by default (since S2.5; docs/CERTIFY.md §2.4). Like a
`SetParam`'s source, a factor nothing else reads is a schedule param, so a price shock keeps the
world's identity and every checkpoint taken before it fires. `Actor(..)` carries the extension's action; the agents have one,
`SetActive(actor: "oven", active: true)`, which wakes or puts to sleep a scripted actor.

## The scripted actor's spec

`spec: Scripted((..))` is the agents' behaviour kind of Phase 0; Phase 2's kinds are new
variants (the four Appendix B roles, below). Every field is required and none has a default:

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

## The Appendix B roles (P2.0.1)

The probe's four kinds, documented field by field in the rustdoc of `rustyecon_agents::roles::spec`;
the rules they run are docs/probe/RULES.md, and `tapes/appb.ron` is the worked example (generated:
`cargo run -p rustyecon-probe --bin appb-tape -- tapes/appb.ron`). Every field is required and
none has a default. Every key but a good or an actor names a registered param, live:

- `Provider((land, endowment, transfer: (to, heads), basket: (good, space, per_basket), spend))`:
  a Pop endowed with `endowment` (`FlowPerYear`) of the `Instant` good `land`, all offered; it
  transfers `heads` (`FlowPerYear`) × P_s to the actor `to`, and spends `spend` (`RatePerYear`)
  of the rest on baskets of one `good` and `per_basket` (`Dimensionless`) of `space`.
- `Workers((labour, heads, chi_max, basket, spend))`: a Pop that mints and offers
  `heads`·min(ln1p(w/P_s)/`chi_max`, 1) of the `Instant` good `labour`, and buys baskets.
- `GoodDesk((output, labour, mach, schedule: (eta, g0, g1, k), technique: (adjust, share),
  assign, scale))`: a Desk; `share` is its genesis human share 1 − x, an inline number in [0, 1];
  `assign` is `Planned` or `ExPost`.
- `MachDesk((output, labour, land, recipe: (own, labour, land), scale))`: a Desk whose recipe
  coefficients are `Dimensionless` params.
- `scale` is `Cash((turnover, tilt, payout))`, with `payout` `None` or `Some((to, rate,
  ceiling))`, or `Step((up, down, dead, buffer, payout: (to, rate), scale))`, whose `scale` is its
  genesis output per tick, an inline number.

## The many-market roles (P2.1.1)

The markets probe's four kinds carry the Appendix B roles to many categories and machine types,
documented field by field in the rustdoc of `rustyecon_agents::roles::many::spec`; the rules are
docs/probe/MARKETS-RULES.md, and `tapes/markets-<id>.ron` are the worked examples (generated:
`cargo run -p rustyecon-probe --bin markets-tape -- --inst <id> tapes/markets-<id>.ron`). Every
field is required and none has a default; every key but a good or an actor names a registered
param, live. Their lists are evaluation order, and the canonical form keeps them as written.

- `BasketProvider((land, endowment, transfer: (to, heads), basket: [(good, weight), ...],
  spend))`: the provider with a basket of items, each `weight` (`Dimensionless`) units of `good`
  per basket; space is an item whose good is the land it sells.
- `BasketWorkers((labour, heads, chi_max, basket: [..], spend))`: the workers with a basket.
- `CategoryDesk((output, labour, service, land, theta, direct_land, schedule: (eta, g0, g1, k),
  line: (edges: [..], density: [..]), technique: (adjust, share), scale))`: a Desk on its
  segments of the shared task line, `edges` the interior edges and `density` one more of them;
  `share` is its genesis human share, an inline number in [0, 1].
- `TypeDesk((output, labour, land, recipe: (own, inputs: [(good, coef), ...], labour, land),
  scale))`: a Desk whose recipe keeps `own` of its output and buys each input's `good`.
- `scale` is the Appendix B roles'.
- The wall's optional fields (P2.3.2; docs/probe/WALL-RULES.md), each absent (off) where not
  written: a `CategoryDesk`'s `tail: Some(key)`, a `Dimensionless` param, the hours per unit at
  tasks closed to machines, added to the line's on `labour`; its `reserved: [(good, coef), ...]`,
  each a reserved type's hours (a good that dies within the tick, not the desk's other goods,
  each once) at `coef` (`Dimensionless`) a unit, whatever the technique; and a
  `BasketProvider`'s `more: [(to, heads), ...]`, further transfers of `heads`
  (`FlowPerYear`)·P_s, paid after `transfer` in list order, each to its own actor. The wall
  tape `tapes/markets-iw1.ron` is the worked example.
- The commons' optional field (P2.3.6; docs/probe/COMMONS-RULES.md), absent (off) where not
  written: a `BasketWorkers`' `exit: Some((good, gross, floor, plot, commons, land))`, the priced
  exit in `good` (one of their basket goods) at s₀ `gross`, s̲ `floor` and h `plot` (each
  `Dimensionless`), with the commons `commons` (`FlowPerYear`) they hold and never trade, and
  `land` (an `Instant` good, the land of the provider that pays their support) for plots rented
  on enclosed land. The commons tapes `tapes/markets-c1.ron` and `markets-c2.ron` are the worked
  examples.
- The trap's remedy (P2.4.5; docs/probe/TRAP-RULES.md), absent (off) where not written: the
  exit's `pace: Some((adjust, share))`, participation at a rate: each tick the workers' share
  moves share(`adjust`) (a `RatePerYear` param, read as a `Share`) of its gap to the
  participation rule's hours over N, and they offer N times it; `share` is its genesis value, an
  inline number in [0, 1]. The paced tapes `tapes/markets-c1p.ron` and `markets-c2p.ron` are the
  worked examples.

## The stock roles (P2.2.1)

The stocks probe's three kinds hold a durable good as a stock (the horse), documented field by
field in the rustdoc of `rustyecon_agents::roles::stock::spec`; the rules are
docs/probe/HORSES-RULES.md, and `tapes/horses-<id>.ron` are the worked examples (generated:
`cargo run -p rustyecon-probe --bin horses-tape -- --inst <id> tapes/horses-<id>.ron`). Every
field is required and none has a default but the maker's `reserve` (L0.4); every key but a good
or an actor names a registered param, live. Their lists (running and build goods) are evaluation
order, and the canonical form keeps them as written.

- `Maker((output, labour, land, own_hours, kappa, running: (goods: [(good, coef), ...], labour),
  build: (goods: [..], labour, land), delta, adjust, cover, reserve, own, scale))`: a Desk that
  builds `output`, a durable good, from `own_hours` (`Dimensionless`) of its own serving stock's hours
  per unit, the running recipe of those hours and the build recipe. `kappa` is a `FlowPerYear`
  (hours a unit a year), `delta` a `FractionPerYear`, `adjust` a `RatePerYear`; `cover` is
  `None` or `Some` of a `Years` param read as whole ticks (at least one); `reserve`, which may be
  absent, is `None` or `Some` of a `Dimensionless` param, ψ: while its net markup
  p_K·(1 − δ·a/κ)/c at posted prices is below ψ > 0 the maker offers none of its finished stock (absent
  or `None`, off; refused on the flow path and under `Hold`; L0.4); `own` is its genesis serving
  stock after wear, an inline number.
- `CapacityDesk((stock, hours, labour, kappa, running, delta, adjust, order, scale))`: a Desk
  that holds `stock`, buys its running inputs and sells its `hours`; `order` is `Target` or
  `Held`.
- `OwnerDesk((output, labour, stock, schedule: (eta, g0, g1, k), technique: (adjust, share),
  assign, kappa, running: [(good, coef), ...], delta, adjust, scale))`: the good desk holding its
  own `stock`, whose hours run on the listed goods.
- A durable good is `Indefinite`, and it wears by δ < 1 a tick; or, for the maker and the owner
  desk, it lives one tick at δ = 1, the flow path (the stocks layer off), with no cover and no
  running recipe. A good that lives more than one tick is bought only as the durable good a
  capacity or owner desk holds, and no role offers it in full: the maker sells its durable good
  under its cover (amended at L0.1, 2026-09-29). `Cast::new` refuses any other.
- `scale` is the Appendix B roles'.

## The plant (P2.2b.1)

The loop step's desks carry CAPACITY's plant, y = P^(1−θ)·z^θ over their own Leontief bundle z
(docs/probe/LOOPS-RULES.md §3, §4), documented field by field in the rustdoc of
`rustyecon_agents::roles::plant::spec`. `tapes/loops-<id>.ron` are the worked examples
(generated: `cargo run -p rustyecon-probe --bin horses-tape -- --inst <id>
tapes/loops-<id>.ron`).

- `plant: Some((good, theta, delta, size, adjust, order, target))`, the last field of
  `TypeDesk`, `Maker` and `CapacityDesk`, and absent (off) where not written. `good` names an
  untraded good no other desk's plant names; `theta` (θ, in (0, 1]) and `size` (s, bundles a
  plant unit, positive) are `Dimensionless`, `delta` (the plant's wear, less than all of it a
  tick) a `FractionPerYear` and `adjust` (s_Kp) a `RatePerYear`, each live; `order` is `Target`
  (M3's rule) or `Gap`; `target` is `Bundles`, or `Herd` on a capacity desk alone.
- A planted desk takes the cash rule; a planted type desk keeps no own input and a planted maker
  is on the stock path with no hours of its own stock, each at genesis. `Cast::new` refuses any
  other, naming the path (docs/probe/LOOPS-RULES.md §6).
- A planted desk's state is the planted variant of its kind's (`PlantedType`, `PlantedMaker`,
  `PlantedCapacity`): the kind's record and the plant's, `{held, target, order, run, built}`.

A complete tape with no behaviour (every actor's spec is `()`) is core's test fixture,
`crates/core/testdata/core.ron`; it loads with the empty extension, `rustyecon_core::NoExt`.
