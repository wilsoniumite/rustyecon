# ENGINE — the Phase 0 engine contract (session 1)

Final, 2026-09-25, committed as P0.2. It folds in the frontend requirement of 2026-09-25 (an
engine crate, a read-only observation API, no frontend mutation, `Send` types, a stable tape
schema) and an adversarial review of the draft. Five steps follow it in order, one commit each:
**P0.3 core**, **P0.4 markets**, **P0.5 agents**, **P0.6 engine + cli**, **P0.7 housekeeping**
(P0.1 is the skeleton; P0.2 is this file and the empty `crates/engine`).

It covers PLAN §3 (phase order), §3.1, §3.3, §3.8 and §3.9; R2–R4, R8 and R12–R14; and ADDENDUM
A2, A3, A5, A12 and A13. Salvage comes from tag `july-v2-phase-3` (`ff01284`); `v2p3:` paths name
that tree. Defects 1–12 are REVIEW §2.2's, N1–N15 ADDENDUM §2.3's, F8 is July's finding on
one-sided markets. Certify is session 2 (A4), except the ledger and the state hash, which are core.

A step is done when `cargo test --workspace --release` is green and `cargo clippy --workspace
--all-targets` and `cargo fmt --check` are clean, with zero warnings, on WSL (primary) and on
Windows (secondary). Commit as `P0.n: <what>`.

**Amended at P0.3** (core), where the text below was wrong or silent; each change is made in
place in the section named.

1. `NoExt`'s `RawActor` and `Actor` are `()`, not `Never`, so a tape with no behaviour can still
   declare actors that hold goods: core's tests and markets' (which sits below agents) need
   holders (§2.3).
2. `Ext` has the supertraits `Clone + Debug + PartialEq` (its implementors are unit markers, and
   the std derives on `StateDelta<E>`, `SimState<E>` and `Tape<E>` need them) and three more
   required methods: `validate` (a resumed checkpoint's extension state is checked, §7.6), and
   `canonical_actor` and `canonical_action` (`to_ron` writes the extension's own lists in
   canonical order too) (§2.3).
3. The `num` helpers return the exact largest fitting value by a search over bit patterns, not
   by at most 4 `next_down` steps. The stepping rule could not meet §11's own test (the result
   fits and its `next_up` does not) without stepping up as well, and a small remainder such as
   `max_remainder(1, 0.999)` lies hundreds of ulps of the difference from its answer, so a
   4-step cap would fail ordinary payouts. `NumError` is `Invalid` or `Exceeded`; there is no
   `Precision`. A zero bound gives exactly 0 and an infinite quotient gives `+∞` (§2.5).
4. `Inventory::take` returns `Result<Vec<Lot>, TakeError>`, since an invalid request (NaN,
   negative, `-0.0`) must be refused and is not a shortfall (§2.2).
5. `SimState::params(&registry)` takes the registry, which lives in the `World`; book readers
   return `Option<f64>` instead of panicking on an id outside the book (§2.3).
6. `Firing` and `Recurring` carry `source: Option<ParamId>`, the param a `SetParam` copied its
   value from: the key "that stays on the firing" (§2.6). A `SetParam`'s source is a fixed use,
   since its value becomes structure at load (§2.1).
7. Errors from the RON parser itself (syntax, an unknown or missing field, a malformed key or
   date) carry the parser's line and column, not a tape path: serde gives no path without a
   crate outside §1's list. Every resolution error names its path. The schema number is probed
   first, so a newer tape is refused as a schema error even if it has fields this loader does not
   know (§2.6).
8. `apply` also refuses `Spoilage` on a mint or burn (it is ageing's), a book write to a currency
   slot (`NoMarket`) and a `SetParam` on a fixed param (`FixedParam`, behind the load check)
   (§2.4). The loader also refuses `ticks_per_year: 0`, a channel from a node to itself, a
   genesis price on a currency, a recurring `last` before its `first`, and duplicate genesis
   holdings (§2.6).
9. Core's tests are unit tests inside `crates/core/src`, each module's after its first
   `#[cfg(test)]`, on the fixture `crates/core/testdata/core.ron`. Source comments cite sections
   such as §2.4, so the source scans of §11 strip comments before looking for literals.

**Amended at P0.4** (markets), the same way. Core did not change.

1. `clear`, `settle` and `SettlePlan::realize` return `Result<_, OrderError>`: a line or fill for
   a market the world does not have is `UnknownMarket`, and a moved list whose length is not the
   plan's is `Moved`, never a panic or a silent skip (N2). `OrderError` also has `Core` (a state
   that does not fit its world) and `Num`; `PriceError` also has `NonFiniteEma` and `Core` (a
   param read that fails) (§3.1–§3.3).
2. `Line` is `#[non_exhaustive]`: its fields are public to read, but only `admit` makes one, so a
   budget binds at admission and nowhere else (§3.1).
3. Admission runs the takes settlement will make, in the same order, on a copy of each actor's
   holding. For a currency, which is one lot, that is exactly `rem = rem − budget`. For a
   perishable good held in several lots it is not the same as subtracting from the lot sum: with
   lots of 0.1 and 0.2, a sell of 0.1 leaves 0.2, but `0.30000000000000004 − 0.1 =
   0.20000000000000004`, so a second sell of that much would pass the subtraction and fall short
   at settlement (§3.1).
4. Duplicate keys are checked before zero-quantity orders are dropped, so a zero-quantity second
   post is still `Duplicate`, and a dropped order binds no cash. Every check runs in canonical
   order, so which error is reported does not depend on input order either (§3.1).
5. A market trades when `S > 0`, `D > 0` and both fills are positive (`MarketFill::trades`). With
   both sides posted, a fill is 0 only when `S/D` or `D/S` underflows below 2⁻¹⁰⁷⁴; nothing then
   trades, rather than one side alone (sellers shipping what the last buyer would take unpaid,
   say). A transfer whose
   nominal quantity is exactly 0 is left out of the plan, except each escrow's last taker, whose
   `All` is always planned (§3.2).
6. The no-shortfall argument of §3.2 assumes normal numbers. Where a payment or a fill falls into
   the subnormal range its rounding is no longer relative, and a non-last take could exceed its
   escrow by a subnormal amount. That would be a `CoreError::Shortfall`, which stops the run with
   its ledger line (R2); no guard is added (A12) (§3.2).
7. The pure rules are public: `imbalance`, `step` (`p·exp(k·x)`) and `next_price(rule,
   one_sided, p, k, S, D)`. Under `Saturate` a one-sided market's `x` is the imbalance itself,
   which is exactly ±1. `Ratio` computes `p·(D/S)`, July's order of operations (§3.3).
8. Markets' tests are integration tests in `crates/markets/tests/` (`admission`, `settlement`,
   `prices`, `time`) on the fixture `crates/markets/testdata/markets.ron`, except July's five
   clearing tests, which stay unit tests of `prices` (§11).

## 0. Engine invariants

Numbered so tests and reviews can cite them. Each has at least one test in §11.

- **E1 — A frontend never mutates state.** Every live intervention (a dial, a shock, a world
  edit) is a dated tape event, or a tape edit followed by a rerun from genesis or a resume from a
  compatible checkpoint (§7.6). A dial change is a dated `SetParam` event (§2.4). Nothing enters a
  run except through the tape: no command-line override, no setter on `Sim`. So every experiment
  is reproducible (R8) and every number keeps its provenance (R4).
- **E2 — No global state and no I/O on the engine path.** `core`, `markets`, `agents` and
  `engine` hold no `static mut`, `thread_local!`, `Rc`, `RefCell`, `Cell`, `OnceCell`,
  `OnceLock`, `LazyLock`, `Mutex`, `RwLock` or atomics. They use no `std::fs`, `std::time`,
  `std::thread`, `std::env`, `std::process` or `std::net`, and no print macro (defect 6). All I/O
  is bytes in and bytes out (`Tape::from_ron`/`to_ron`, `Checkpoint::to_bytes`/`from_bytes`/
  `to_ron`/`from_ron`); paths and file extensions live only in the cli. So the engine can run
  in-process, behind a local server, or compiled to WASM.
- **E3 — Engine types cross threads.** `Sim`, `Tape`, `World`, `Checkpoint`, `TickReport`,
  `HoldingTotals` and every error type are `Send + Sync + 'static`. A frontend can run a `Sim` on
  a worker thread and send reports over a channel.
- **E4 — Reading is read-only and free.** No `Sim` method returns `&mut SimState`, `&mut World`
  or anything that reaches them. Reading never changes a hash, and each `TickReport` agrees with
  the accessors read after the step.
- **E5 — A failed step poisons the `Sim`.** After any `RunError` the `Sim` refuses to step or
  checkpoint until it is rebuilt from a tape or a checkpoint (§7.5).
- **E6 — Hooks read the phase-start state.** Within a phase, every actor's hook reads the state
  as it stood when the phase began. Outputs are collected, checked, then applied in canonical
  order, so an actor's id never decides what it sees.
- **E7 — Hook output is whitelisted.** Each hook may emit only the deltas and orders §7.3 lists
  for it; anything else is `RunError::ForeignWrite` (R13, R2).
- **E8 — The tape is stable and keyed.** It carries a schema version. Every entity has a stable
  key, and the loader orders everything by key, never by file position. Reformatting a tape or
  reordering its lists changes no id and no hash (§2.6).

## 1. Crates

```
Cargo.toml            [workspace] members = ["crates/*"], resolver = "2"; edition 2021
rust-toolchain.toml   channel = "1.97.1", components = ["rustfmt", "clippy"]
clippy.toml           disallowed types (std hash containers), methods (transcendentals, powi, mul_add)
crates/core     rustyecon-core     ids, keys, goods, clock, units, registry, inventory, state, deltas,
                                   apply, ledger, hash, checkpoint, the tape schema and its resolver,
                                   num (libm)
crates/markets  rustyecon-markets  orders, admission, clearing, settlement, price update
crates/agents   rustyecon-agents   the Behaviour seam, View, the scripted actor
crates/engine   rustyecon-engine   Sim: the tick loop, step, run, checkpoint, resume, the replay audit;
                                   TickReport; the hook whitelist
crates/cli      rustyecon-cli      bin `rustyecon`: argument parsing, file I/O, exit codes
crates/certify  rustyecon-certify  empty until session 2 (A4); nothing depends on it yet
crates/worldgen rustyecon-worldgen empty until Phase 4; nothing depends on it yet
tapes/gate.ron                     the gate world (§10)
```

Dependencies run one way: core ← markets ← agents ← engine ← cli, and a future frontend crate
depends on engine the same way. Core depends on no workspace crate (N14). `crates/oracle` comes
from another run, joins through the glob and depends at most on core; nothing on the engine path
depends on it (R13). The engine re-exports the three crates under their own names, plus a
`prelude` of the types a frontend names (ids, keys, `Holder`, `Date`, `Tape`, `World`,
`Checkpoint`, `TickReport` and its lines, the errors), so a frontend depends on the engine alone.
P0.2 points the cli's `Cargo.toml` at the engine only.

External crates, all cached on both machines: `serde 1.0.228` (derive), `ron 0.8.1`, `bincode
1.3.3`, `libm 0.2.16`, `clap 4.6.1` (cli only), `tempfile 3.27.0` (dev only). Not used: `rayon`,
`thiserror`, `HashMap`, `HashSet` (R8). Error enums are written by hand, with `Display`.

## 2. rustyecon-core

### 2.1 Ids, keys, goods, nodes, clock, units, registry (defect 10)

```rust
// one macro: `pub struct X(pub u32)` + idx(); Copy, Eq, Ord, Hash, Debug, Serialize, Deserialize
id!(GoodId); id!(NodeId); id!(ChannelId); id!(ClassId); id!(DeskId); id!(PopId); id!(ParamId); id!(EventId);
pub enum ActorId { Desk(DeskId), Pop(PopId) }              // derived Ord: all Desks, then all Pops
pub enum Holder { Actor(ActorId), Escrow(NodeId, GoodId) } // an escrow exists only inside phase 3
pub struct Key(String);                                    // [a-z0-9_.-]+, checked at load
pub struct GoodDef { pub id: GoodId, pub key: Key, pub life: Life, pub price_rate: Option<ParamId> }
pub enum Life { Indefinite, Instant, Ticks(u32) }          // Ticks(L), L >= 1, from a Years param
pub struct NodeDef { pub id: NodeId, pub key: Key, pub currency: GoodId }
pub struct ChannelDef { pub id: ChannelId, pub key: Key, pub from: NodeId, pub to: NodeId } // static only
pub struct Clock { pub start: Date, pub ticks_per_year: u32 }                               // §6
pub struct Date { pub y: i32, pub m: u8, pub d: u8 }       // "YYYY-MM-DD" on the tape
pub enum Unit { Dimensionless, Years, FlowPerYear, RatePerYear, CompoundPerYear, FractionPerYear }
pub struct ParamDef { pub id: ParamId, pub key: Key, pub unit: Unit, pub genesis: f64,
                      pub basis: Basis, pub fixed: bool }
pub enum Basis { Measured { source: String, vintage: String }, Literature(String),
                 Approximate(String), Fitted { fit: String }, Assumed(String) }  // R4's five tags
pub struct Registry { params: Vec<ParamDef> }                        // by ParamId
pub struct Params<'a> { values: &'a [f64], registry: &'a Registry }  // current values, from SimState
impl Params<'_> { pub fn get<U: UnitKind>(&self, p: ParamId) -> Result<U, CoreError>; }
// UnitKind: one newtype per Unit (Dimensionless(f64), Years(f64), FlowPerYear(f64), ...).
// get checks the registered unit against U.
```

- **Keys (E8).** Every tape entity has a key: goods, nodes, channels, classes, params, actors,
  events and recurring entries. A key is unique within its kind; desks and pops share one
  namespace, and so do events and recurring entries. A key is never reused for a different
  entity, and a rename is a new entity. The loader numbers each kind densely in key byte order,
  never in file order. Dense ids belong to core and are valid against one `World`, which maps
  both ways (`id_of`, `key_of`). Anything kept across a tape edit is kept by key.
- **No bare numbers on the delta path.** Every map is keyed by `ActorId` or `Holder`, never by a
  bare `u32`. July's `(u32, u32)` keys were defect 10 (`v2p3: systems/transactions/mod.rs:200-219,
  390-395`).
- **Goods.** July's `movement_type`, `divisible` and `storage_cost_per_tick` go (nothing reads
  them), and `alpha` becomes the registered `price_rate` (`RatePerYear`). A currency is
  `Indefinite`, has no `price_rate`, has no market, and is priced at 1 by definition. Every cost
  is a recipe input (R14); no floor or guard is an amount of currency.
- **Instant goods.** A lot of an `Instant` good is minted with life `Some(0)`. It trades in
  phases 2–3 of the tick it was minted, can be used in phase 4, and dies in phase 5a. It may be
  minted only in phases 0 and 1; a mint anywhere else is `CoreError::WrongPhase`. That rule
  prevents July's defect, where a good minted in phase 4 spoiled in phase 5 and never traded. The
  gate world has no Instant good. Phase 2's labour hours and parcel services are Instant, minted
  as `Endowment` in `decide`.
- **Params.** The genesis value, unit and basis live in `World`. The current value lives in
  `SimState` (hashed) and changes only by `SetParam`. Every reader reads at use time, and nothing
  caches a param's value beyond the call that read it. A param is `fixed` when the loader turns
  it into structure (a shelf life, a recurring period, or the value a `SetParam` copies) or when
  changing it would change what a past tick meant (the ledger tolerances). A `SetParam` on a
  fixed param is a load error, and `apply` refuses one too (`FixedParam`).
- **Units.** `Clock`'s methods each take one unit type (§6), so using a rate as a flow does not
  compile.

### 2.2 Inventory (defect 5; N9)

```rust
pub struct Lot { pub qty: f64, pub life: Option<u32> }  // None = indefinite; Some(0) dies at the next 5a
pub struct Inventory(Vec<(GoodId, Vec<Lot>)>);          // goods ascending; lots by life, None last
pub enum Amount { Qty(f64), All }
impl Inventory { pub fn get(&self, g: GoodId) -> f64;   // lot sum, left fold in lot order
    pub fn put(&mut self, g: GoodId, lots: Vec<Lot>) -> Result<(), CoreError>;
    pub fn take(&mut self, g: GoodId, a: Amount) -> Result<Vec<Lot>, TakeError>;
    // TakeError: Shortfall(Shortfall { requested, held }) | Invalid(q); nothing moves on either
    pub fn age(&mut self) -> Vec<(GoodId, f64)>;        // spoiled per good
    pub fn lot_count(&self) -> usize; }
```

- **Coalescing (N9).** `put` merges lots of equal life. All lives fall together, so a good holds
  at most `max_life + 1` lots, and an indefinite good (currency included) holds exactly one. The
  form is canonical and lands before the first golden hash (A2).
- **Atomic `take`.** `Qty(q)` with `q > get(g)` returns `Err(TakeError::Shortfall)` and changes
  nothing; a non-finite or sign-bit-set `q` is `Err(TakeError::Invalid)`.
  `q == get(g)` or `All` takes every lot. Otherwise lots go soonest-expiring first (July's FIFO
  when a good has one life), splitting the last one taken. If rounding uses up the lots while a
  remainder is left, every lot is taken, so a request with `q <= get(g)` never falls short. A
  single-lot good (every currency) is taken as `lot.qty − q`, which is the arithmetic admission
  mirrors (§3.1).
- **Rules.** Non-finite quantities, and any quantity whose sign bit is set (`-0.0` included), are
  rejected in every build profile. A lot at exactly `+0.0` is dropped, with no epsilon. `age`
  drops lots at `Some(0)` and then decrements the rest, so a lot minted in tick t with `Ticks(L)`
  can sell in ticks t+1 … t+L (July's rule). A genesis holding of a perishable good is one lot
  with the good's full life. Serde keeps July's `Vec<(GoodId, Vec<(qty, life)>)>` form (`v2p3:
  types/inventory.rs:119-152`); loading re-sorts, re-coalesces, and rejects NaN and `-0.0`.

### 2.3 State and the extension seam (N14)

```rust
pub trait Ext: Clone + Debug + PartialEq + Send + Sync + 'static {   // unit markers
    type State: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;  // hashed
    type Delta: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;
    type RawActor: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync; // tape form
    type Actor: Clone + Debug + Serialize + Send + Sync;                                 // resolved
    type RawAction: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;
    fn resolve_actor(raw: &Self::RawActor, r: &mut Resolver) -> Result<Self::Actor, LoadError>;
    fn resolve_action(raw: &Self::RawAction, r: &mut Resolver) -> Result<Self::Delta, LoadError>;
    fn genesis(actors: &[ActorDecl<Self::Actor>]) -> Result<Self::State, LoadError>;
    fn apply(s: &mut Self::State, d: &Self::Delta) -> Result<(), CoreError>;
    fn owner(d: &Self::Delta) -> Option<ActorId>;  // None: a world-level delta, legal only from the tape
    fn validate(s: &Self::State, actors: &[ActorDecl<Self::Actor>]) -> Result<(), CoreError>; // resume
    fn canonical_actor(raw: &mut Self::RawActor);   // for to_ron: the spec's own lists in canonical order
    fn canonical_action(raw: &mut Self::RawAction);
}
pub struct NoExt;   // no behaviour: State = (), RawActor = Actor = () (actors hold goods, spec `()`),
                    // Delta = RawAction = `enum Never {}`
pub struct SimState<E: Ext> {           // fields private to core
    tick: u64,
    params: Vec<f64>,                    // current value per ParamId
    book: MarketBook,                    // flat [node·n_goods + good]: price, ema, supply, demand
    holdings: BTreeMap<Holder, Inventory>, // every declared actor, even if empty; no escrow between ticks
    ext: E::State,
}
```

- **Resolver.** It turns keys into ids and records each param reference with its use (`live` or
  `fixed`) and its unit, checked against the registry. The unused-param and fixed-param load
  checks read that record.
- **Readers.** `tick()`, `param(p)`, `params(&registry)` (typed by unit), `param_values()`,
  `book()`, `price(n, g)`, `ema`, `supply`, `demand` (each `Option<f64>`, `None` for an id outside
  the book), `holding(h)`, `holdings()`, `ext()` and `validate(&world)` (§7.6's shape check) are
  public. `apply` is the only writer (`v2p3:
  state/apply.rs:7-9`). Core holds no v1 or July agent struct (N14), and no constructor takes a
  default price.
- **Genesis book.** Each (node, non-currency good) gets its price from the tape, exactly once.
  `ema₀` is the genesis price, by definition, and `S₀ = D₀ = 0`. This removes July's genesis price
  and EMA of 1.0 (`v2p3: state/sim_state.rs:56,59`), one of the audit's three constants to
  register.
- **Growth without reopening core.** Phases 2 and 3 add behaviour kinds (variants of the agents'
  `RawActor` and `Actor`), actor state and `Actor` deltas. The tape-driven world deltas of Phase 3
  and later (enclosure status, recipe-version publication, law and tax rates) are `Ext` deltas
  whose `owner` is `None`. Their costs use the provenances already declared (§2.4). Entrants are
  declared at load as dormant desks, one per recipe version per region (PLAN §3.1), and a dated
  event activates them. The actor set is fixed at load.

### 2.4 Deltas, provenance and phases

```rust
pub enum Phase { Events, Decisions, Clearing, Settlement, Production, Upkeep, Prices, Measure } // 0–7
pub enum StateDelta<E: Ext> {
    SetPrice { node: NodeId, good: GoodId, price: f64 },                  // finite and > 0
    SetEma { node: NodeId, good: GoodId, ema: f64 },                      // finite and > 0
    SetVolumes { node: NodeId, good: GoodId, supply: f64, demand: f64 },
    Transfer { from: Holder, to: Holder, good: GoodId, amount: Amount },  // atomic; lots keep lives
    Mint { to: Holder, good: GoodId, qty: f64, prov: Provenance },        // life from GoodDef
    Burn { from: Holder, good: GoodId, amount: Amount, prov: Provenance },
    SetParam { param: ParamId, value: f64 },                              // unit fixed by the registry
    Age { holder: Holder },
    Actor(E::Delta),
    AdvanceTick,
}
pub enum Provenance { Production, Consumption, Spoilage, Endowment, Depreciation, Construction, Event }
```

These are July's seven core arms (`v2p3: types/delta.rs`), with Add and Remove replaced by
`Transfer`, `Mint` and `Burn`, plus `SetParam`, and one seam, `Actor`, in place of July's other 19
variants. A transfer moves exactly what it takes, or nothing when its source is short, so it
cannot mint; July's paired Remove/Add trap (`v2p3: docs/architecture/engine.md:440-445`) is gone.
Every mint and burn carries a typed provenance, with no `Default` and no serde default;
`Transfer` carries none. `apply` matches exhaustively.

| Provenance | Used for |
|---|---|
| `Production` | a desk's recipe inputs (burn) and outputs (mint) |
| `Consumption` | a pop's recipe inputs (burn) |
| `Spoilage` | lots dropped by `Age` |
| `Endowment` | output of a recipe with no inputs (Phase 0); Instant labour and parcel services (Phase 2) |
| `Depreciation`, `Construction` | Phase 3's capital; declared now so the ledger lines exist |
| `Event` | tape mints and burns |

**Phase rules in `apply`** (`CoreError::WrongPhase`): an escrow holder appears only in
`Settlement`; `SetVolumes` only in `Clearing`; `SetPrice`, `SetEma` and `AdvanceTick` only in
`Prices`; `SetParam` only in `Events`; `Age` only in `Upkeep`; a `Mint` of an Instant good only in
`Events` or `Decisions`. Which actor may emit what inside a phase is the engine's whitelist (E7,
§7.3), not core's. Besides: `Spoilage` is ageing's alone, so a `Mint` or `Burn` carrying it is
`ReservedProvenance`; a `SetPrice`, `SetEma` or `SetVolumes` on a (node, currency) slot is
`NoMarket`; a price or EMA must be positive. `apply` returns the quantity each delta moved: what a
transfer delivered, a mint created, a burn destroyed, or an `Age` spoiled (summed over goods); 0
for the rest.

### 2.5 Apply, ledger, conservation, hash, checkpoints (R2; defect 9; N2, N3, N11)

```rust
pub fn apply<E: Ext>(s: &mut SimState<E>, w: &World<E>, phase: Phase, ds: &[StateDelta<E>],
                     l: &mut Ledger) -> Result<Vec<f64>, CoreError>; // quantity moved, per delta
pub struct Tolerances { pub rel_flow: ParamId, pub rel_stock: ParamId } // Dimensionless, fixed; no absolute term
pub struct Ledger { opening: Vec<f64>, declared: Vec<f64>, gross: Vec<f64>,
                    lines: BTreeMap<(GoodId, Provenance), f64>, shortfall: Option<ShortfallLine> }
impl Ledger { pub fn open<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<Ledger, CoreError>;
              pub fn close<E: Ext>(self, s: &SimState<E>, w: &World<E>) -> Result<TickAudit, CoreError>; }
pub struct ShortfallLine { pub tick: u64, pub phase: Phase, pub holder: Holder, pub good: GoodId,
                           pub requested: f64, pub held: f64, pub prov: Option<Provenance> }
pub struct TickAudit { pub lines: Vec<(GoodId, Provenance, f64)>, pub max_margin: f64, pub max_drift: f64 }
pub fn state_hash<E: Ext>(s: &SimState<E>) -> u64; // FNV-1a 64 over bincode 1 (v2p3: certify/hash.rs:16-30)
pub struct Checkpoint<E: Ext> { pub format: u32, pub world_id: u64, pub prefix_id: u64, pub state: SimState<E> }
impl<E: Ext> Checkpoint<E> {
    pub fn to_bytes(&self) -> Vec<u8>;  pub fn from_bytes(b: &[u8]) -> Result<Self, CheckpointError>;
    pub fn to_ron(&self) -> String;     pub fn from_ron(s: &str) -> Result<Self, CheckpointError>; }
pub mod num {
    pub fn exp(x: f64) -> f64; pub fn expm1(x: f64) -> f64; pub fn ln(x: f64) -> f64;
    pub fn ln1p(x: f64) -> f64; pub fn pow(x: f64, y: f64) -> f64;       // one libm call each (A5)
    pub fn max_qty(budget: f64, price: f64) -> Result<f64, NumError>;    // largest q: fl(price·q) <= budget
    pub fn max_scale(held: f64, coef: f64) -> Result<f64, NumError>;     // largest x: fl(coef·x) <= held
    pub fn max_remainder(total: f64, spent: f64) -> Result<f64, NumError>; // largest d: fl(spent+d) <= total
    pub fn is_clean(x: f64) -> bool;                                     // finite, sign bit clear
    pub enum NumError { Invalid { what, value }, Exceeded { total, spent } }
}
```

- **The `num` helpers** return the exact largest value whose rounded product or sum stays within
  the bound: they start from the quotient or difference and search the bit patterns of the
  non-negative doubles (ordered like their values), galloping to bracket the boundary and then
  bisecting, so the result fits and its `next_up` does not. Usually 2 to 4 evaluations, at most
  about 130, and no precision failure. A zero budget, holding or remainder gives exactly 0
  (nothing is bought with nothing). An infinite quotient (a tiny or zero price, a zero
  coefficient) gives `+∞`, so the quantity binds (N6). An input that is not finite or has its
  sign bit set is `NumError::Invalid`, and spending more than `total` is `NumError::Exceeded`;
  callers turn these into their own errors. Nothing in the engine panics.
- **Undefined ids (N2).** Every good, node, holder and param is checked against `World` before
  anything moves: `UnknownGood`, `UnknownHolder` or `UnknownParam` in every profile, never a
  `debug_assert` (July's defect, `v2p3: certify/ledger.rs:99-108, 160-181`).
- **Signs and finiteness.** Every quantity, price, EMA, volume and param value is finite and has a
  clear sign bit, or `apply` rejects it. No state-path value is `-0.0`, so `min` and `max` never
  meet a signed-zero tie.
- **Shortfalls (N3).** A failed `take` in a `Transfer` or `Burn` writes its `ShortfallLine` to the
  ledger and returns `CoreError::Shortfall(line)`. Nothing of that delta applies and the tick ends.
  There is no partial burn: `Burn { amount: All }` means "whatever is there". `apply` drops an
  escrow once it is empty, and `close` fails if one remains.
- **Close.** `closing` is a walk of every holder. Per good, `drift = (closing − opening) −
  declared` and `tol = rel_flow·gross + rel_stock·max(|opening|, |closing|)`, with both
  tolerances read from the current params. `drift.abs() <= tol` must hold, and NaN fails it; a
  failure is `CoreError::Conservation { good, drift, tol, lines }`. With no stock and no flow,
  `tol = 0` and the drift must be exactly zero. The margin is 0 when `drift == 0` and `|drift|/tol`
  otherwise, so `max_margin <= 1` is the pass condition and 0/0 never occurs. July's
  `ABS_TOLERANCE` does not move (A12).
- **Income.** It is not checked in Phase 0. Phase 2 can check it from the (good, provenance)
  lines, the settlement lines with their moved quantities (§3.2) and the audit, which always
  reaches the caller (§7).
- **Hash.** It covers the whole `SimState`: tick, params, market book, every holding with its lot
  lives, and the extension state. It does not cover the tape.
- **Checkpoints (N11).** A checkpoint carries `world_id` and `prefix_id` (§2.6), and
  `Sim::resume` refuses a mismatch (§7.6). The bytes form opens with the magic `RUSTYECK` and
  `format` as a little-endian `u32`, then bincode 1 of `(world_id, prefix_id, state)` with no
  trailing bytes; the RON form's first field is `format`, read by a probe that skips the rest
  undecoded. Either way a wrong format is refused before the state is decoded. Decoding re-sorts
  and re-coalesces lots and rejects NaN, `-0.0` and negative values (and non-positive prices);
  `Checkpoint::validate(&world)` (that is, `SimState::validate`) then checks every id and shape
  against the `World`. `Checkpoint::of(&world, state)` records `world_id` and `prefix_id(tick)`.
  `to_bytes` and `to_ron` cannot fail for core's types; should an extension's `Serialize` fail,
  they return output the loaders reject.

### 2.6 The tape: schema, resolution, identity (N1, N2, N10 in part; E8)

```rust
pub struct Tape<E: Ext> {                   // the editable document; every raw type is pub,
    pub schema: u32,                        // Serialize + Deserialize, #[serde(deny_unknown_fields)]
    pub header: RawHeader, pub params: Vec<RawParam>, pub goods: Vec<RawGood>,
    pub nodes: Vec<RawNode>, pub channels: Vec<RawChannel>, pub classes: Vec<Key>,
    pub actors: Vec<RawActorEntry<E::RawActor>>, pub genesis: RawGenesis,
    pub events: Vec<RawEvent<E::RawAction>>, pub recurring: Vec<RawRecurring<E::RawAction>> }
impl<E: Ext> Tape<E> { pub fn from_ron(s: &str) -> Result<Self, LoadError>; pub fn to_ron(&self) -> String; }
pub fn resolve<E: Ext>(t: &Tape<E>) -> Result<(World<E>, SimState<E>), LoadError>;
pub struct World<E: Ext> { pub name: String, pub world_id: u64, pub clock: Clock, pub registry: Registry,
    pub tol: Tolerances, pub market: MarketConfig, pub goods: Vec<GoodDef>, pub nodes: Vec<NodeDef>,
    pub channels: Vec<ChannelDef>, pub classes: Vec<Key>, pub actors: Vec<ActorDecl<E::Actor>>,
    pub schedule: Schedule<E>, keys: KeyIndex }
impl<E: Ext> World<E> { pub fn prefix_id(&self, tick: u64) -> u64; /* id_of, key_of per kind */ }
pub struct ActorDecl<A> { pub id: ActorId, pub key: Key, pub class: ClassId, pub home: NodeId, pub spec: A }
pub struct MarketConfig { pub rule: PriceRule, pub one_sided: OneSided, pub ema_time_constant: ParamId }
pub enum PriceRule { Imbalance, Ratio }     // N13: required, no Default
pub enum OneSided { Saturate, Hold }        // F8: required, no Default
pub struct Firing<E: Ext> { pub tick: u64, pub event: EventId, pub occurrence: u32, pub action: StateDelta<E>,
                           pub source: Option<ParamId> }  // a SetParam's source param, for its basis
pub struct Schedule<E: Ext> { once: Vec<Firing<E>>, every: Vec<Recurring<E>> }
impl<E: Ext> Schedule<E> { pub fn fire(&self, tick: u64) -> Vec<Firing<E>>; }
```

- **Schema version.** `schema: 1`. The loader reads only its own version; anything else is
  `LoadErrorKind::Schema`, found by a probe of `schema` alone before the rest is parsed. Every
  schema change bumps the number, since there are no defaults to absorb one.
- **No silent fields.** Unknown fields are rejected (`deny_unknown_fields`), and nothing carries
  `#[serde(default)]`. An `Option` field must be written as `None` or `Some(..)`: serde would
  otherwise read a missing one as `None`, so each raw `Option` field uses a `deserialize_with`
  helper, which makes serde treat it as required (probed on serde 1.0.228 and ron 0.8.1). The
  helper is public, `core::tape::raw::required`, for the extension's raw types.
- **Documented.** `core::tape::raw` carries `#![deny(missing_docs)]`. Each field's rustdoc gives
  its type, its unit (or "dimensionless"), that it is required, and that it has no default.
  `docs/TAPE.md` (written in P0.3) points to that rustdoc, keeps the schema-version history, and
  uses the gate tape as its worked example.
- **Keys and order (E8).** Keyed data is always a list of entries carrying a `key`, never a RON
  map, because serde keeps the last of two equal map keys without complaint. Lists can be in any
  order; duplicates are load errors. The loader reads each list into canonical order:
  - goods, nodes, channels, classes, params, actors (per kind), events and recurring entries by key;
  - genesis prices by (node, good), genesis holdings by (holder, good);
  - recipe inputs and outputs by good, buy and sell lines by (node, good), payout recipients by actor.

  `to_ron` writes that canonical form (comments are not kept).
- **World identity.** `world_id` is FNV-1a 64 over bincode 1 of the World's run content together
  with the genesis `SimState`. The run content is everything except the schedule, the tape's
  `name` and the basis texts, none of which changes a number the run computes. Reformatting,
  comments, CRLF line endings, list order and a `to_ron` round trip therefore keep `world_id`. Any
  edit to a number, a unit, a key or the structure changes it. It is also the identifier N10's
  manifest needs in session 2.
- **Prefix identity.** `prefix_id(t)` is FNV-1a 64 over bincode 1 of every firing with `tick < t`,
  in firing order, each as (tick, event key, occurrence, resolved action), with recurring entries
  expanded. A checkpoint of the state whose tick is t stores `world_id` and `prefix_id(t)`.
- **Sorting (N1).** Dates become ticks (§6). `once` is sorted by (tick, key), and `fire` returns
  its `partition_point` range merged with the recurring firings due that tick, all in key order.
  July binary-searched an unsorted list (`v2p3: scenario/mod.rs:14, 25-42`).
- **Recurring entries** are `(key, first: Date, every: Years param, last: Option<Date>, act)`,
  firing at `first_tick + k·period`. A period that rounds to 0 ticks is a load error, so `every =
  0` cannot load.
- **Actions.** Each resolves to one of:
  - `Mint` or `Burn` (provenance `Event`);
  - `Transfer`;
  - `SetParam { param, to }`, where `to` is another registered param's key with the same unit. The
    value comes from that param at load, and its key stays on the firing, so the new value keeps
    a basis (R4). The target must not be fixed.
  - `Actor(E::RawAction)`.
- **Resolver.** The extension resolves through `Resolver`: `good`, `node`, `class`, `actor`,
  `channel` (key to id), `param(key, unit, ParamUse::Live | Fixed, field)`, `ticks(key, field)` (a
  fixed `Years` param as whole ticks), `value(p, field)`, `quantity(v, field)`, `is_currency(g)`,
  `tick_of(date, field)`, `clock()`, and `enter(segment)`, `leave()` and `error(field, kind)` for
  paths.
- **Load errors (N2)** occur in every profile, and each names its tape path, for example
  `actors[mill].spec.buy[village/grain].qty`. The RON parser's own errors (syntax, an unknown or
  missing field, a malformed key or date) carry its line and column instead. They are:
  - a wrong schema version, an unknown field or a missing field;
  - an invalid, unknown or duplicate key;
  - a missing or duplicate genesis price, or a genesis price that is not finite and > 0;
  - a unit mismatch, an unreferenced param, or a `SetParam` on a fixed param or across units;
  - an event before `start`;
  - an order line on a currency; a currency that is not `Indefinite` or has a `price_rate`; a
    non-currency good without a `price_rate`;
  - a good on both sides of a recipe;
  - a life or period that rounds to 0 ticks;
  - `Ratio` with `Saturate`;
  - buy or payout weights whose left fold in canonical order is not exactly 1.0 (§4);
  - an actor `home` that is not a node;
  - `ticks_per_year: 0`, a channel from a node to itself, a genesis price on a currency, a
    recurring `last` before its `first`, a duplicate genesis holding or held good.

## 3. rustyecon-markets (N5–N8, N13; R12; F8)

### 3.1 Orders, admission and clearing: cash binds at the order (N7)

```rust
pub struct Order { pub actor: ActorId, pub class: ClassId, pub node: NodeId, pub good: GoodId,
                   pub qty: f64, pub side: Side }
pub enum Side { Buy { budget: f64 }, Sell }         // budget in the node's currency
pub enum SideTag { Buy, Sell }                      // Side without the budget; Buy < Sell
#[non_exhaustive] pub struct Line { pub order: Order, pub feasible: f64 }  // made by admit only
pub fn admit<E: Ext>(orders: Vec<Order>, s: &SimState<E>, w: &World<E>) -> Result<Vec<Line>, OrderError>;
pub enum OrderError { UnknownActor(ActorId), UnknownNode(NodeId), UnknownGood(GoodId),
    NoMarket { node, good }, WrongClass { actor, class, registered }, BadValue { order, what, value },
    Duplicate { actor, node, good, side }, OverBudget { order, currency, remaining },
    OverPosted { order, remaining }, UnknownMarket { node, good }, Moved { deltas, moved },
    Core(CoreError), Num(NumError) }
```

- **Feasible quantity.** A buy's `feasible = min(qty, num::max_qty(budget, price)?)`. No price
  guard applies (N6): when `budget/price = ∞`, `qty` binds.
- **Admission.** Orders are first sorted by (actor, node, good, side), so their input order is
  irrelevant. Each failure is an `OrderError`:
  - Per actor and currency, `rem` starts at `get(currency)`. Each budget, in that order, must be
    at most `rem`, and then `rem = rem − budget`. This is the same left-to-right subtraction the
    single currency lot performs at settlement, so a payment that fits here fits there
    (`OverBudget`).
  - The sells of one good are checked cumulatively across nodes against `get(good)` in the same
    way (`OverPosted`). Both checks run the inventory's own takes, in settlement's order, on a
    copy of the actor's holding: for a currency that is the subtraction above, and for a good
    held in several lots it is the lot-by-lot arithmetic settlement will perform (amendment 3).
  - One order per (actor, node, good, side), checked before zero-quantity orders are dropped.
  - The order names a declared actor, a node and a non-currency good, and its class is the
    actor's registered class.
  - Values are finite with a clear sign bit, and zero-quantity orders are dropped; a dropped
    order binds no cash.
- Budgets bind here and nowhere else. Settlement never cuts a buyer, so the volumes the price
  reads are all demand that can pay (N7).
- **Phase 0 trades across nodes.** An actor may post at any node, and nothing crosses a channel or
  pays a crossing cost. The simplification lasts until transport desks exist (§13).
- **Clearing** (salvaged, `v2p3: systems/clearing/mod.rs:15-56`). Per market in (node, good)
  order, `S` is the sum of sells and `D` the sum of feasible buys, both left folds in canonical
  line order. `buyer_fill = if D > 0 { (S/D).min(1.0) } else { 0.0 }`, and `seller_fill` mirrors
  it. Every (node, non-currency good) gets `SetVolumes { supply: S, demand: D }`, with 0 and 0
  where nothing was posted. A line for a market the world does not have is `UnknownMarket`.

### 3.2 Settlement: one fill, both sides, through an escrow

```rust
pub fn clear<E: Ext>(lines: &[Line], w: &World<E>) -> Result<(Vec<StateDelta<E>>, Fills), OrderError>;
pub struct Fills { /* one MarketFill per market, (node, good) order */ }  // markets(), get(node, good)
pub struct MarketFill { pub node: NodeId, pub good: GoodId, pub supply: f64, pub demand: f64,
                        pub buyer_fill: f64, pub seller_fill: f64 }      // trades(): S, D, both fills > 0
pub fn settle<E: Ext>(lines: &[Line], f: &Fills, s: &SimState<E>, w: &World<E>)
    -> Result<SettlePlan<E>, OrderError>;
impl<E: Ext> SettlePlan<E> { pub fn deltas(&self) -> &[StateDelta<E>];
    pub fn realize(self, moved: &[f64]) -> Result<(Vec<SettleLine>, Vec<RationLine>), OrderError>; }
pub struct SettleLine { pub actor: ActorId, pub class: ClassId, pub node: NodeId, pub good: GoodId,
                        pub side: SideTag, pub qty: f64, pub value: f64 }       // moved, one per order
pub struct RationLine { pub node: NodeId, pub good: GoodId, pub class: ClassId, pub side: SideTag,
                        pub requested: f64, pub feasible: f64, pub filled: f64 } // R12
pub fn update_prices<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<Vec<StateDelta<E>>, PriceError>; // §3.3
```

For each market that trades (`S > 0`, `D > 0` and both fills positive; amendment 5), in (node,
good) order, with posted price `p`, currency `c` and escrow `X = Holder::Escrow(node, good)`:

1. Each seller, in actor order, transfers `ship_i = qty_i·seller_fill` of the good to `X`.
2. Each buyer, in actor order, transfers `pay_j = p·r_j` of `c` to `X`, where `r_j =
   feasible_j·buyer_fill` is its one filled quantity.
3. `X` transfers `r_j` of the good to each buyer. One buyer goes last and takes `All`: the one
   with the largest `r_j`, compared by `f64::total_cmp` and then by `ActorId`, so among equal
   quantities the highest id goes last. Every other buyer takes `Qty(r_j)`, in actor order, first.
4. `X` transfers `p·ship_i` of `c` to each seller the same way. The largest shipper, chosen by the
   same order, goes last and takes `All`.

**Why no step falls short.** Every fill is at most 1 and rounding is monotone, so `ship_i <=
qty_i` and `pay_j <= budget_j`. A buyer pays in the order its budgets were admitted, and receipts
only add. The largest taker goes last, so no earlier take can exceed the escrow, and every escrow
ends empty. Only the last taker's quantity differs from nominal, by rounding; payments are
nominal. The argument assumes normal numbers; in the subnormal range a take could still fall
short, and that stops the run with its ledger line (amendment 6). A transfer whose nominal
quantity is exactly 0 is not planned, and its line reports 0.

**Moved quantities.** `apply` returns what each delta moved, and `realize` turns the plan into
`SettleLine`s and `RationLine`s from those actual quantities. A buyer's line has `qty` = the good
received and `value` = the currency paid; a seller's has `qty` = shipped and `value` = received.

**Gone:** balance state and forgiveness (N8, `v2p3: transactions/mod.rs:396-406`), and the
absolute epsilons (`:125, 190, 357-433`). Soonest-expiring-first out of the escrow gives the oldest
lots to the lowest id (open question 4).

There is one `RationLine` per (node, good, class, side) for each market that had an order. For a
buyer class, the cash shortfall is `requested − feasible` and the market shortfall is `feasible −
filled`. For a seller class, `requested = feasible` = offered, and `filled` = shipped.
`SettleLine`s come one per admitted line in (actor, node, good, side) order, and `RationLine`s in
(node, good, class, side) order, each sum a left fold in actor order.

### 3.3 Price update (N5, N13; F8; A13)

For each non-currency market, with `k`, `w` and the one-sided rule read from the current params:

- **Two-sided or idle.** `x = if S == 0 && D == 0 { 0 } else { (D − S)/S.max(D) }`.
- **`Imbalance`.** `p' = p·num::exp(k·x)`, with `k = clock.log_step(price_rate)`. This replaces
  July's `p·(1 + α·x)`; the two differ only at second order in `α·x`, and only this form is
  tick-invariant (open question 1).
- **One-sided markets** (exactly one of S and D is zero) follow the tape's `one_sided` field. July
  treated them by formula; F8 asked for a decision.
  - `Saturate`: `x = ±1`, as in July. An idle one-sided market's price then moves by `exp(±k)`
    every tick until it leaves the finite positive range, which is a `PriceError`. At `k = 0.1` a
    tick that takes about 7,450 ticks. Session 2's runaway detector watches for it.
  - `Hold`: `p' = p`. A one-sided market carries no price evidence.
- **`Ratio`.** `p' = p·(D/S)` when both are positive (July's order of operations), else `p' =
  p`. `Ratio` with `Saturate` does not load.
- **EMA.** `ema' = w·p + (1 − w)·ema`, with `w = clock.weight(ema_time_constant)` read at use time
  and `p` this tick's posted price.
- **Changes and errors.** `SetPrice` and `SetEma` are emitted only when the bits change. That is an
  exact test, not a tolerance; July's `1e-12` gates were N5's floor. A non-finite or non-positive
  price is `PriceError::NonFinite { node, good, from, to }` and stops the run, and an EMA likewise
  `NonFiniteEma`; a param read that fails is `PriceError::Core`. Nothing is emitted on an error.
  There is no floor, ceiling or fallback; N13's fallback, `v2p3: clearing/mod.rs:124-132`, goes.
- **Public rules.** `imbalance(S, D)`, `step(p, k, x) = p·exp(k·x)` and `next_price(rule,
  one_sided, p, k, S, D)` are the pure functions `update_prices` applies to each market.

## 4. rustyecon-agents: the seam and the scripted actor

```rust
pub struct View<'a, S> { pub tick: u64, pub clock: &'a Clock, pub me: ActorId, pub class: ClassId,
    pub home: NodeId, pub own: &'a Inventory, pub own_state: &'a S,
    pub posted: Posted<'a>,            // price and ema per (node, good), read-only
    pub params: Params<'a> }           // current values, typed by unit
pub trait Behaviour: Send + Sync {     // Phase 2 implements it for new kinds
    type Own;                          // the actor's own extension state
    fn decide(&self, v: &View<Self::Own>) -> Decision;                     // phase 1
    fn produce(&self, v: &View<Self::Own>) -> Vec<StateDelta<Agents>>;     // phase 4
    fn upkeep(&self, v: &View<Self::Own>) -> Vec<StateDelta<Agents>>; }    // phase 5b
pub struct Decision { pub orders: Vec<Order>, pub deltas: Vec<StateDelta<Agents>> }
pub struct Agents;  // impl Ext: State = BTreeMap<ActorId, ActorState>, Delta = AgentDelta,
                    //           RawActor = RawSpec, Actor = Spec, RawAction = RawAgentAction
pub enum ActorState { Scripted(ScriptState) }   // one variant per behaviour kind
pub struct ScriptState { pub active: bool }
pub enum AgentDelta { SetActive { actor: ActorId, active: bool } } // owner: Some(actor)
pub struct Cast { /* one Behaviour per kind, built from World.actors */ }
```

`Cast` dispatches each actor to its kind's `Behaviour` by its `ActorState` variant. R13 holds by
construction: a `View` cannot reach another actor's holdings, orders or state, and agents never
depends on the oracle. The engine enforces the rest (§7.3).

The scripted actor (either kind) reads its lines' quantities and rates from the current params,
so a dated `SetParam` retargets it. It does nothing while `active` is false.

- **`decide`.** Two steps, payouts first:
  - **Payouts.** `P = fl(share(payout)·cash)`, split by recipient weights in `ActorId` order. Each
    recipient but the last gets `fl(w·P)`, and the last gets `num::max_remainder(P, Σ earlier)`.
    Each part is capped by the actor's own `rem`, walked by subtraction as admission walks it. The
    payouts are transfers from `me`.
  - **Buy lines**, in (node, good) order, post `qty = flow(line)`. The spending total is `B =
    fl(share(spend)·cash_after)`, where `cash_after` is the cash less the payouts, subtracted the
    way the inventory will subtract them. `rem` walks down from `cash_after` the same way.
    Each non-last line gets `min(fl(w·B), rem)`. The last gets `min(max_remainder(B, Σ earlier),
    rem)`.

  A valid tape can therefore never produce `OverBudget`, even when `share` rounds to exactly 1.
  Last, each sell line posts `flow(line)`, or `own.get(good)` for `AllHeld`.
- **`produce`** is Leontief: `x = min(flow(capacity), min_k max_scale(held_k, a_k))`. It burns
  `a_k·x` of each input in good order (`Production` for a Desk, `Consumption` for a Pop). It then
  mints `o_l·x` of each output in good order, as `Production`, or as `Endowment` when the recipe
  has no inputs. A recipe with no outputs is consumption. `max_scale` means no burn falls short,
  which is what July's shared-input reservation was for (`v2p3:
  systems/production/mod.rs:34-77`).
- **`upkeep`** does nothing in Phase 0.

## 5. The tape runtime format (RON, schema 1)

```ron
Tape(
  schema: 1,
  header: (name: "gate", start: "1750-01-01", ticks_per_year: 52,
    market: (rule: Imbalance, one_sided: Hold, ema_time_constant: "price.ema_tc"),
    ledger: (rel_flow: "ledger.rel_flow", rel_stock: "ledger.rel_stock")),
  params: [   // one entry per dial; five shown
    (key: "ledger.rel_flow", value: 1e-12, unit: Dimensionless, basis: Assumed("July REL_FLOW")),
    (key: "price.ema_tc", value: 0.5, unit: Years, basis: Approximate("≈ July 2/53 at 52 ticks/yr")),
    (key: "rate.grain", value: 5.2, unit: RatePerYear, basis: Assumed("July alpha 0.1 per weekly tick")),
    (key: "life.bread", value: 0.0577, unit: Years, basis: Assumed("three weeks")),
    (key: "mine.capacity", value: 520.0, unit: FlowPerYear, basis: Assumed("gate world")), …],
  goods: [(key: "bread", life: Years("life.bread"), price_rate: Some("rate.bread")),
          (key: "coin", life: Indefinite, price_rate: None),
          (key: "grain", life: Indefinite, price_rate: Some("rate.grain")), /* fuel likewise */],
  nodes: [(key: "town", currency: "coin"), (key: "village", currency: "coin")],
  channels: [],
  classes: ["households", "pensioners", "producers"],
  actors: [(key: "mill", kind: Desk, class: "producers", home: "town", basis: Assumed("gate world"),
    spec: Scripted((active: true,
      recipe: Some((inputs: [("grain", 2.0), ("fuel", 1.0)], outputs: [("bread", 3.0)], capacity: "mill.capacity")),
      buy: [(node: "village", good: "grain", qty: "mill.buy.grain", weight: 0.6),
            (node: "town", good: "fuel", qty: "mill.buy.fuel", weight: 0.4)],
      sell: [(node: "town", good: "bread", qty: AllHeld)],
      spend: "mill.spend", payout: Some((rate: "mill.payout", to: [("workers", 1.0)]))))), …],
  genesis: (basis: Assumed("gate world"),
    prices: [(node: "town", good: "bread", price: 2.0), (node: "village", good: "grain", price: 1.0), …],
    holdings: [(holder: "mill", goods: [("coin", 100.0), ("grain", 16.0)]), …]),
  events: [   // any order: the loader sorts by (tick, key)
    (key: "mine.restored", at: "1770-06-15", basis: Assumed("gate world"),
     act: SetParam(param: "mine.capacity", to: "mine.capacity.base")),
    (key: "mine.cut", at: "1760-03-01", basis: Assumed("gate world"),
     act: SetParam(param: "mine.capacity", to: "mine.capacity.cut")),
    (key: "oven.opens", at: "1768-04-01", basis: Assumed("gate world"),
     act: Actor(SetActive(actor: "oven", active: true))),
    (key: "bread.line.up", at: "1765-09-01", basis: Assumed("gate world"),
     act: SetParam(param: "workers.buy.bread", to: "workers.buy.bread.high"))],
  recurring: [(key: "pension", first: "1751-01-01", every: "pension.period", last: None,
               basis: Assumed("gate world"), act: Mint(holder: "pensioners", good: "coin", qty: 5.0))],
)
```

**Registration (R4).** Every dial with a time unit, every behavioural rate and every tolerance is
a named entry in `params`, referenced by key. Dimensionless structural data may sit inline under
its entry's `basis`: recipe coefficients, weights, genesis stocks and prices, and event
quantities (open question 3). `spec: Scripted(...)` is an enum, so Phase 2's kinds are new
variants, not a new schema shape. `rustyecon registry <tape>` prints both kinds of number, each
param with its unit, its per-tick value (§6) and its basis.

## 6. Time (A13)

`ticks_per_year` is a `u32 >= 1` in the header, and Δ = 1/ticks_per_year. All conversion happens
in `Clock`, and nothing else divides by `ticks_per_year`.

| Unit | Used as | `Clock` method | Per tick |
|---|---|---|---|
| `Dimensionless` | a ratio, weight or tolerance | none | `v` |
| `FlowPerYear` | a flow of goods (capacity, order line) | `flow` | `v·Δ` |
| `RatePerYear` | a continuous draw on a stock (spend, payout) | `share` | `−expm1(−v·Δ)` |
| `RatePerYear` | a price rate (`Imbalance`) | `log_step` | `v·Δ` |
| `CompoundPerYear` | an effective annual rate ρ | `compound` | `expm1(Δ·ln1p(ρ))`, i.e. `(1+ρ)^Δ − 1` |
| `FractionPerYear` | an annual fraction δ (e.g. depreciation) | `fraction` | `−expm1(Δ·ln1p(−δ))`, i.e. `1 − (1−δ)^Δ` |
| `Years` | an EMA time constant τ | `weight` | `−expm1(−Δ/τ)` |
| `Years` | a period or shelf life (fixed) | `ticks` | `round(v·tpy)` ticks; 0 is a load error |
| date `YYYY-MM-DD` | an event time | `tick_of` | `floor(days·tpy·10⁴ / 3,652,425)`, in integers |
| a tick | its date | `date_of` | `start + ceil(t·3,652,425 / (tpy·10⁴))` days, in integers |

Dates are proleptic Gregorian, and the year is the mean Gregorian year. Days come from the integer
`days_from_civil`, so the date-to-tick map uses no floating point. `tick_of(date_of(t)) == t`
whenever `ticks_per_year <= 365`.

Changing only `ticks_per_year` leaves these invariant:
- annual endowment output;
- a share's decay `exp(−v·t)`;
- the EMA's step response `exp(−t/τ)`;
- the `Imbalance` path `exp(r·x·t)` under a constant `x`;
- the annual values of compound and fractional rates;
- event dates, to within one tick.

`Ratio` is a one-step rule and is not invariant; that is recorded, not fixed.

The oracle solves per period. Its adapter either takes the tick as its period or restates its
inputs through `Clock`: `compound(ρ)`, `fraction(δ)`, and `J_b` via `ticks`. These methods exist
now, so the oracle need not wait for Phase 3.

## 7. rustyecon-engine: the Sim, the tick and the observation

### 7.1 The Sim

```rust
pub type Tape = rustyecon_core::Tape<Agents>;  pub type World = rustyecon_core::World<Agents>;
pub type Checkpoint = rustyecon_core::Checkpoint<Agents>;
pub struct Sim { world: World, state: SimState<Agents>, cast: Cast, status: Status,
                 hash: u64, last: Option<TickReport> }            // every field private
impl Sim {
    pub fn new(tape: &Tape) -> Result<Sim, LoadError>;
    pub fn resume(tape: &Tape, cp: &Checkpoint) -> Result<Sim, ResumeError>;          // §7.6
    pub fn step(&mut self) -> Result<TickReport, RunError>;
    pub fn step_traced(&mut self) -> Result<(TickReport, Trace), RunError>; // every applied delta,
                                                                            // with phase and moved qty
    pub fn run_until(&mut self, until: u64, on_tick: &mut dyn FnMut(&TickReport)) -> Result<(), RunError>;
    pub fn checkpoint(&self) -> Result<Checkpoint, RunError>;     // RunErrorKind::Poisoned if poisoned
    pub fn status(&self) -> Status;                               // Ready | Poisoned { tick, phase }
    pub fn last_report(&self) -> Option<&TickReport>;
    // read-only accessors (E4)
    pub fn world(&self) -> &World;  pub fn tick(&self) -> u64;  pub fn hash(&self) -> u64; // cached
    pub fn price(&self, n: NodeId, g: GoodId) -> f64;  /* ema, supply, demand alike */
    pub fn param(&self, p: ParamId) -> f64;
    pub fn holding(&self, h: Holder) -> Option<&Inventory>;
    pub fn holdings_of(&self, g: GoodId) -> impl Iterator<Item = (Holder, f64)> + '_;
    pub fn actor_state(&self, a: ActorId) -> Option<&ActorState>;
    pub fn observe_holdings(&self) -> HoldingTotals;   // owned, for another thread
}
pub fn audit_replay(tape: &Tape, until: u64) -> Result<u64, ReplayError>;   // §7.6; the final hash
```

`run_until(T)` steps until the state's tick is `T`, calling `on_tick` after each step. `Tape`,
`World`, `Checkpoint` and `TickReport` are `Clone`, so a frontend on another thread keeps its own
`World` for key lookups. A frontend
on a worker thread steps and sends each `TickReport` (and, at its own cadence, `HoldingTotals`)
over a channel; the cli is one such frontend, on the main thread.

### 7.2 The tick

Tick t runs on the state whose tick is t. One ledger opens before phase 0, and every phase's
deltas pass through `apply` with that phase.

| # | Phase | In Phase 0 |
|---|---|---|
| 0 | events | `schedule.fire(t)`, applied in firing order |
| 1 | decisions | `decide` for every actor on the phase-start state; checked (E7), then applied in `ActorId` order |
| 2 | clearing | `admit`, then `clear` |
| 3 | settlement | `settle`, apply, `realize`; no escrow may remain |
| 4 | production | `produce` for every actor on the phase-start state; checked, then applied in `ActorId` order |
| 5 | upkeep | 5a, core ageing: `Age` per holder in `Holder` order. 5b, the `upkeep` hook, handled as in phase 4 (a no-op for scripted actors) |
| 6 | price update | `update_prices`, then `AdvanceTick` |
| 7 | measure | `ledger.close` (the audit), `state_hash`, the `TickReport` |

The report's `hash` is of the state after `AdvanceTick`, whose tick is t + 1. A checkpoint "at
tick t" holds the state whose tick is t, before tick t runs.

### 7.3 Hooks: phase-start reads and the whitelist (E6, E7)

In phases 1, 4 and 5b every actor's hook receives a `View` of the state as it stood when the phase
began; the engine collects all outputs, checks each one, and only then applies them in `ActorId`
order. This is safe because hooks write only their own holdings and state: another actor's
deltas can only add to an actor's holdings, never overdraw them. It is also what PLAN §3.8's
ordered reduction across region shards will need.

| Hook | May emit |
|---|---|
| `decide` | `Transfer { from: Actor(me), to: Actor(_) }` of any good; `Mint { to: Actor(me) }` of an Instant good with `Endowment`; `Actor(d)` with `owner(d) == Some(me)`; orders with `actor == me` and `class ==` my class |
| `produce` | `Burn { from: Actor(me) }` with `Production` (a Desk) or `Consumption` (a Pop); `Mint { to: Actor(me) }` with `Production` or `Endowment`; `Actor(d)` owned by me |
| `upkeep` | `Burn { from: Actor(me) }` with `Depreciation`; `Actor(d)` owned by me |

Anything else — a mint to or burn from another holder, any escrow, `SetPrice`, `SetEma`,
`SetVolumes`, `SetParam`, `Age`, `AdvanceTick`, a foreign order — is `RunError::ForeignWrite`
(or `ForeignOrder`), and core's phase rules still apply underneath. Phase 3 adds `Construction`
burns to `produce`'s list; the list lives in the engine, so core does not reopen. Tape events
(phase 0) are the world's hand and are not whitelisted; an `Ext` delta with `owner == None` is
legal only there.

### 7.4 The observation: `TickReport` (E4)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]   // on every type below
pub struct TickReport {
    pub tick: u64, pub date: Date, pub hash: u64,            // date = clock.date_of(tick)
    pub markets: Vec<MarketLine>,       // every (node, non-currency good), in (node, good) order
    pub settlements: Vec<SettleLine>,   // one per admitted order, with moved quantities (§3.2)
    pub rationing: Vec<RationLine>,     // one per (node, good, class, side) that had an order
    pub audit: TickAudit,               // the ledger's (good, provenance) lines and margins
    pub events: Vec<FiredEvent>,        // in firing order
}
pub struct MarketLine { pub node: NodeId, pub good: GoodId,
    pub price: f64,       // posted, used by this tick's settlement
    pub next_price: f64, pub ema: f64,  // after the update
    pub supply: f64, pub demand: f64,   // S and feasible D (N7)
    pub cleared: f64,                   // the good that left the escrow
    pub buyer_fill: f64, pub seller_fill: f64 }
pub struct FiredEvent { pub key: Key, pub occurrence: u32, pub action: StateDelta<Agents> }
pub struct HoldingTotals(pub Vec<(Holder, GoodId, f64)>);   // (holder, good) order
```

A report costs O(markets + orders + events) and never copies holdings; `observe_holdings` does
that, O(holders × goods held), when a frontend asks. Ids in a report are dense and are read
against `Sim::world()`; a frontend that compares runs of two tapes maps them by key. Each report
agrees with the accessors read after its step: each line's `next_price`, `ema`, `supply` and
`demand` equal `Sim::price`, `ema`, `supply` and `demand` for its (node, good), and `hash` equals
`Sim::hash()`.

### 7.5 Errors and poisoning (E5)

```rust
pub struct RunError { pub tick: u64, pub phase: Phase, pub kind: RunErrorKind }
pub enum RunErrorKind { Core(CoreError), Order(OrderError), Price(PriceError),
    ForeignWrite { actor: ActorId, hook: Hook, delta: StateDelta<Agents> },
    ForeignOrder { actor: ActorId, order: Order }, Poisoned }
pub enum ResumeError { Load(LoadError), Checkpoint(CheckpointError),
    WrongWorld { tape: u64, checkpoint: u64 }, WrongPrefix { tick: u64, tape: u64, checkpoint: u64 },
    Invalid(CoreError) }
pub enum ReplayError { Load(LoadError), Run(RunError), Mismatch { tick: u64, live: u64, shadow: u64 } }
```

`Display` gives the ledger line for a shortfall or a conservation breach. `apply` is atomic per
delta, not per tick, so a failed step leaves the state partway through its tick. The `Sim` is
then `Poisoned { tick, phase }`: `step`, `run_until` and `checkpoint` return
`RunErrorKind::Poisoned`, and `last_report` still returns the last good tick's report. The
accessors still answer, reading the state as the failed delta left it, for diagnosis only; no
invariant holds for it. A `Sim` is rebuilt with `Sim::new` or `Sim::resume`. No step discards the
audit (N3); July's did (`v2p3: systems/mod.rs:37`).

### 7.6 Checkpoints, resume and the replay audit (N11, E1)

- **`checkpoint`** stores `world_id`, `prefix_id(state.tick)` and the state.
- **`Sim::resume(tape, cp)`** resolves the tape and checks the format. It then requires `cp.world_id
  == world.world_id` (`WrongWorld` otherwise) and `cp.prefix_id == world.prefix_id(cp.state.tick)`
  (`WrongPrefix`), and validates the state against the world: every declared actor holds an
  inventory, every id is in range, and params, book and extension state have the right shape.
  So a checkpoint stays valid across an edit dated at or after its tick, such as a new shock or a
  dated `SetParam`. It is refused after any edit to the world or to anything that fired before it.
- **`audit_replay(tape, until)`** runs a `Sim` with `step_traced`. Beside it runs a shadow
  `SimState` from the same genesis, which applies each tick's trace through `core::apply` alone,
  with no behaviour, clearing or settlement code, and a ledger of its own. The two `state_hash`es
  are compared every tick (test_03's design; July's `v2p3: runner.rs:240-255`). This proves `apply`
  is the only writer. It is not how an edited tape is rerun; that is `Sim::new` or `Sim::resume`.

## 8. rustyecon-cli: a thin binary

The cli holds argument parsing (clap), file reads and writes, the mapping from extension to
format, and exit codes. Nothing else.

```
rustyecon run      <tape.ron> --until <tick> [--out DIR] [--checkpoint-every N] [--format bin|ron] [--hashes FILE]
rustyecon resume   <checkpoint.bin|.ron> --tape <tape.ron> --until <tick> [same options]   # N11
rustyecon replay   <tape.ron> --until <tick>                                              # audit_replay
rustyecon registry <tape.ron>
```

- `--until T` runs until the state's tick is `T`.
- **Checkpoints.** With `--out DIR`, the final state always goes to `DIR/tick_{:08}.{bin|ron}`,
  named by the state's tick. With `--checkpoint-every N` as well, so does every state whose tick is
  a multiple of `N`. Without `--out`, nothing is written. A failed write stops the run with exit 3,
  and no further tick runs; July only printed it (`v2p3: runner.rs:488-505`).
- **Hashes.** `--hashes` writes one `tick 0x{hash:016x}` line per tick, and stdout ends with the
  final tick and hash.
- **Exit codes:**

  | Code | Meaning |
  |---|---|
  | 0 | ok |
  | 1 | load or argument error, an unknown checkpoint extension included |
  | 2 | run error: the error or ledger line and the last good tick go to stderr |
  | 3 | I/O or checkpoint error, a refused resume included |
  | 4 | replay mismatch |

- July's `--agents`, `--price-rule` and `--supply-rule` switches do not return (N13), and no
  command-line override of any kind exists (E1).

## 9. Determinism and hashing (R8, A5)

- **Canonical order.** Phases run in order. Within a phase:
  - events by (tick, key);
  - decisions and production outputs by `ActorId`;
  - admission by (actor, node, good, side);
  - clearing and prices by (node, good);
  - settlement by (node, good), then steps 1–4, then the taker order of §3.2.

  Shuffling `orders` before `admit`, or any tape list before loading, leaves the stream unchanged.
- **Collections.** `Vec` in id order, or `BTree*`. No hashed container, no `rayon`, no RNG.
- **Floats on the state path.**
  - Allowed: `+ − × ÷`, `sqrt`, comparisons, `total_cmp`, `min`, `max`, `abs`, `floor`, `round`,
    `next_down`, `next_up`, and int↔float casts.
  - Only through `core::num` (libm): `exp`, `expm1`, `ln`, `ln1p`, `pow`.
  - Banned: `mul_add`, `powi`, `powf`, the inherent transcendentals, `f32`, and fast-math.
  - Sums are left folds in canonical order, and no value is `-0.0` (§2.5).

  The verdict path follows the same rules in session 2.
- **Enforcement.** `clippy.toml` denies the std hash containers and, by primitive path, the
  inherent transcendentals, `powi` and (from P0.2) `mul_add`; a probe confirmed clippy rejects
  `f64::exp`, `ln` and `powi`. The source scans of §11 back it up. Linux–Windows hash equality is
  recorded in STATE.md, not gated.

## 10. The gate world (`tapes/gate.ron`)

It starts on 1750-01-01 at 52 ticks a year and runs 2,080 ticks, 40 years.

- **Goods:** `coin` (currency), `grain` and `fuel` (indefinite), `bread` (three-week life).
  **Nodes:** `town` and `village`, both in coin. **Classes:** producers, households, pensioners.
  **Market:** `Imbalance`, `one_sided: Hold`.
- **Desks** (key order gives `Desk(0)` … `Desk(3)`):
  - `farm` (home village) and `mine` (home town) are endowments of grain and fuel.
  - `mill` (home town) is Leontief: grain 2 + fuel 1 → bread 3. It buys grain in the village and
    fuel in town, and sells all its bread in town.
  - `oven` (home town) is a second bread desk, declared dormant; the entrant pattern.
  - Farm, mine and mill pay the workers, and the farm pays the pensioners a little.
- **Pops:** `pensioners` (home village, `Pop(0)`) hold a bread line that their budget cannot cover
  at genesis prices, which makes them the cash-short buyer. `workers` (home town, `Pop(1)`) buy and
  consume bread.
- **Shared numbers.** `Desk(0)` (farm) and `Pop(0)` (pensioners) share a number, as do `Desk(1)`
  (mill) and `Pop(1)` (workers). At genesis, bread demand exceeds the mill's capacity.
- **Events.** Four dated events, listed out of order:
  - the mine's capacity is cut in 1760;
  - the workers' bread line rises in 1765;
  - the oven activates in 1768;
  - the mine's capacity is restored in 1770.

  Bread rations again after the cut. A yearly recurring mint pays the pensioners coin.

The implementer tunes the numbers, registered as `Assumed`, to one bar: every market trades in
every year, and no price leaves [1e-3, 1e3] × its genesis value. Failure tests make variants of
this one file, by text substitution or by editing the parsed `Tape`. No second hand-kept tape
exists.

## 11. Tests (the A3 gate)

Run with `cargo test --workspace --release`. Test names are binding: a test may be split, not
merged. Test bars are relative, named once at the top of their test file with the reason for their
size; none is absolute (A12). Gate tests read `tapes/gate.ron` through `include_str!`.

**core**

| Area | Tests | Checks |
|---|---|---|
| inventory (salvaged, `v2p3: types/inventory.rs:159-251`, renamed for the new API) | `add_and_get`, `take_partial` (was `remove_partial`), `take_more_than_held_is_a_shortfall_and_moves_nothing` (was `remove_more_than_held`), `stays_sorted`, `age_removes_expired_and_decrements` (was `spoil_lots_…`), `serde_preserves_lot_lives`, `take_fefo` (was `remove_fifo`) | defect 5; atomic take |
| inventory (new) | `lots_coalesce_by_life`, `take_all_of_multi_lot_holding_never_falls_short` (lots 0.1 + 0.2), `nonfinite_negative_or_negative_zero_qty_is_rejected`, `instant_lot_dies_at_5a` | N9 |
| ledger (salvaged: the 11 tests at `v2p3: certify/ledger.rs:359-488`, names kept) | `balanced_transfer_conserves`, `unbalanced_transfer_is_a_breach`, `declared_mint_and_burn_conserve`, `untagged_creation_is_a_breach`, `stock_appearing_beyond_what_was_declared_is_a_breach`, `mutation_that_skips_the_ledger_is_caught`, `tolerance_covers_scan_noise_on_large_stock`, `tolerance_still_catches_a_real_leak_at_the_same_stock`, `margin_ratio_reports_headroom`, `shortfall_is_recorded` (now also returns the error), `reset_clears_state` (now: a fresh ledger has no lines). An unbalanced move or untagged creation can only be a direct state write now, so those tests write state directly | defect 9, A12 |
| ledger through apply (salvaged, `v2p3: tests/test_04:42-121`) | `untagged_creation_is_caught_through_apply` (the untagged add is a direct write, since every `Mint` needs a provenance), `shortfall_is_captured_and_not_silently_clamped` (and stops with `CoreError::Shortfall`). The third, `unbalanced_transfer_through_apply_is_a_breach`, is replaced by `atomic_transfer_cannot_mint`, since a `Transfer` can no longer be unbalanced | defect 9, N3 |
| ledger (new) | `undefined_good_is_an_error_in_release`, `nan_drift_is_a_breach`, `zero_stock_zero_flow_requires_exact_zero`, `margin_is_zero_when_drift_is_zero`, `direct_state_write_is_caught`, `burn_shortfall_stops_with_a_ledger_line`, `atomic_transfer_cannot_mint`, `escrow_outside_settlement_is_an_error`, `instant_mint_outside_phases_0_and_1_is_an_error`, `book_writes_outside_their_phase_are_errors` | N2, N3, A12 |
| hash | `identical_states_hash_equal_and_tick_changes_it` (salvaged), `hash_covers_holdings_lives_params_and_ext_state` | R8 |
| checkpoint (salvaged, `v2p3: output/checkpoint.rs:84-102`) | `binary_round_trip` and `human_readable_round_trip` (now by full hash, not by tick), `checkpoint_format_is_checked`, `checkpoint_rejects_nan_and_unknown_ids` | N11 |
| tape | `unsorted_events_fire_in_order` (ticks listed 5, 2, 9, 9 all fire; key order within a tick), `every_zero_is_rejected`, `undefined_good_is_a_load_error`, `unknown_actor_or_param_is_a_load_error`, `duplicate_keys_are_rejected`, `missing_genesis_price_is_rejected`, `unused_param_is_rejected`, `unit_mismatch_is_rejected`, `event_before_start_is_rejected`, `set_param_on_fixed_or_other_unit_is_rejected`, `ratio_with_saturate_is_rejected`, `date_to_tick_is_integer_exact`, `date_of_inverts_date_to_tick` | N1, N2, R4, N13 |
| tape schema | `schema_version_is_checked`, `unknown_field_is_rejected`, `missing_optional_field_is_rejected`, `tape_round_trips` (parse, `to_ron`, parse: equal `Tape`, equal `world_id`), `file_order_is_irrelevant` (permuting every list: same `world_id` and prefix ids), `reformatted_tape_keeps_its_ids` (whitespace, comments, CRLF: same ids), `new_entity_keeps_existing_ids` (adding a good, an actor and an event leaves every existing key naming the same entity with the same resolved content), `prefix_id_covers_only_past_firings` | E8, N11 |
| other | `clock_conversions` (all units and methods of §6), `num_matches_libm_bits`, `num_helpers_are_exact` (each result fits and its `next_up` does not), `core_has_no_workspace_dependencies` (core's `Cargo.toml` names no `rustyecon-` crate; a `NoExt` tape resolves, fires its events through `apply` and closes its ledger) | A13, A5, N14 |
| added at P0.3 | `desk_and_pop_sharing_a_number_are_distinct_holders`, `transfers_keep_lot_lives`, `set_param_respects_the_registry`, `unknown_holders_and_reserved_provenance_are_errors`, `currencies_and_prices_rates_are_checked`, `genesis_holdings_start_with_full_lives`, `extension_errors_name_their_path`, `core_types_cross_threads`, `keys_check_their_character_set`, `actor_and_holder_orders_are_canonical`, `dates_parse_print_and_count_days`, `fnv_matches_the_published_vectors` | defect 10, E3 |

**markets**

| Area | Tests | Checks |
|---|---|---|
| clearing (salvaged, `v2p3: systems/clearing/mod.rs:147-176`, retargeted) | `price_rises_on_excess_demand`, `price_falls_on_excess_supply`, `balanced_market_stable`, `imbalance_excess_demand`, `imbalance_excess_supply`. July's `abs() < 1e-12` checks become bit equality: `exp(0) = 1` exactly, and the imbalances are exactly ±0.5 | A12 |
| settlement | `cash_short_buyer_settles_both_sides_from_one_fill`: the buyer asks 10 at price 2 on budget 6, so 3 is feasible; the seller offers 5. Asserts `SetVolumes { supply: 5, demand: 3 }`, `buyer_fill` 1 and `seller_fill` 0.6; the buyer gets 3 and pays 6, the seller ships 3 and gets 6; the class line is 10 / 3 / 3; `update_prices` lowers the price (x = −0.4), where July's settlement would have read D = 10 and raised it | N7, R12 |
| admission, settlement | `over_budget_is_an_order_error`, `over_posting_seller_is_an_order_error`, `sells_at_two_nodes_are_checked_cumulatively`, `duplicate_order_is_rejected`, `two_kinds_same_number_settle_apart`, `no_forgiveness_currency_moves_only_by_transfers`, `escrow_is_empty_after_settlement`, `settlement_is_invariant_to_order_input_order`, `all_taker_is_largest_then_highest_id`, `settle_lines_carry_moved_quantities` | defect 10, N8, R8 |
| prices | `tiny_prices_move_by_rule` (at 1e-15 and 1e-300 each step is exactly `p·exp(k·x)`), `huge_price_overflow_is_a_run_error`, `tiny_and_huge_prices_settle_without_guards` (1e-200 flows move at prices 1e-15 and 1e200), `price_rule_comes_from_the_tape` (no rule, no load; `Ratio` and `Imbalance` hash apart), `one_sided_rule_comes_from_the_tape` (at S = 0 < D, `Saturate` multiplies by `exp(k)` each tick and `Hold` keeps the bits), `ratio_nonfinite_is_an_error`, `genesis_ema_is_the_genesis_price`, `params_are_read_at_use_time` | N5, N6, N13, F8 |
| time | `ema_is_tick_length_invariant`, `imbalance_path_is_tick_length_invariant` | A13 |
| added at P0.4 | `orders_are_checked_against_the_world`, `budgets_bind_per_currency`, `feasible_quantity_is_what_the_budget_buys`, `rationing_is_recorded_per_class`, `markets_types_cross_threads` | N2, N6, N7, R12, E3 |

**agents**

| Area | Tests | Checks |
|---|---|---|
| seam | `leontief_never_overdraws`, `scripted_actor_reads_only_its_view` (foreign holdings change; the decision does not), `extreme_spend_rate_never_overbudgets` (v·Δ = 40, so `share` = 1.0; payouts and budgets pass admission), `weights_must_sum_exactly_to_one`, `dormant_actor_does_nothing` | R13, R4 |

**engine**

| Area | Tests | Checks |
|---|---|---|
| determinism | `gate_repeat_identical_hashes` (2,080 ticks; the state evolves), `gate_resume_from_checkpoints` (checkpoints at ticks 1, 520, 1,040 and 2,079 through `to_bytes`/`from_bytes` and `to_ron`/`from_ron`; the tails equal the uninterrupted run), `gate_replay_matches_every_tick` (`audit_replay`), `file_order_is_irrelevant_to_the_hash_stream` | R8, N11, E8 |
| conservation, rationing | `gate_conserves_every_tick` (every report has `max_margin <= 1` and no shortfall). `gate_events_fire_in_date_order`. `gate_ids_apart`: for `Desk(0)`/`Pop(0)` and `Desk(1)`/`Pop(1)`, every tick and good, the holding's change equals the signed sum of the trace entries naming that holder, and each of its settle lines names it. `gate_rations_and_records_by_class`: bread rations at tick 0 and after the 1760 cut; pensioners' requested > feasible; at every rationed tick the recorded demand equals Σ feasible, which is below Σ requested. `gate_lots_bounded`: 20,000 ticks; lots per good ≤ life + 1; currency one lot | R2, N1, defect 10, R12, N7, N9 |
| edits (E1) | `resume_after_future_event_edit_equals_full_rerun`, `resume_after_past_edit_is_refused`, `world_edit_refuses_every_checkpoint`, `dated_param_change_takes_effect_at_its_tick` (the 1760 cut: capacity changes at `tick_of(1760-03-01)` and not before), `entrant_activates_on_its_date` | E1, N11 |
| frontend contract | `observation_matches_accessors` (every gate tick), `observing_changes_no_hash`, `engine_types_are_send` (compile time: `Send + Sync + 'static` for `Sim`, `Tape`, `World`, `Checkpoint`, `TickReport`, `HoldingTotals`, `RunError`, `LoadError`, `ResumeError`, `ReplayError`), `failed_step_poisons_the_sim` | E3, E4, E5 |
| hooks | `decisions_read_phase_start_state`: from a fixed state, visiting actors in id order and in reverse gives identical outputs; renaming keys so that the id order reverses leaves every actor's tick-0 orders unchanged. `behaviour_output_is_whitelisted`: each forbidden arm from each hook (a mint to or burn from another holder, an escrow transfer, `SetPrice`, `SetEma`, `SetVolumes`, `SetParam`, `Age`, `AdvanceTick`, a foreign `Actor` delta, a foreign order) is `ForeignWrite` or `ForeignOrder` | E6, E7, R13, R2 |
| time | `a13_annual_quantities_invariant`: at `ticks_per_year: 12` the 1750 farm and mine mints equal the 52-tick run's within a relative bar of 1e-12, and event ticks match their dates | A13 |
| source scans (read the sources of core, markets, agents and engine, comments stripped) | `no_raw_transcendentals`, `no_hashed_collections`, `no_behavioural_float_literals` (source before the first `#[cfg(test)]` may hold only `0.0` and `1.0`), `engine_path_does_no_io` (E2's list) | A5, R8, R4, E2 |

**cli**

| Area | Tests | Checks |
|---|---|---|
| product path | `gate_resume_through_product_path`: `run` with checkpoints in both formats, then `resume` from ticks 1, 520, 1,040 and 2,079; the `--hashes` tails equal the uninterrupted run. `replay_command_passes_on_the_gate` (exit 0) | N11 |
| failures | `shortfall_stops_the_run` (exit 2, with the ledger line on stderr), `undefined_good_fails_to_load` (exit 1), `resume_refuses_another_tape` (exit 3), `unknown_extension_errors` (exit 1), `failed_checkpoint_save_stops_the_run` (`--out` names a regular file: exit 3, and the `--hashes` file ends at the failed checkpoint's tick) | N3, N2, N11 |

## 12. Steps

| Step | Delivers |
|---|---|
| P0.3 core | §2 and its tests; `docs/TAPE.md` |
| P0.4 markets | §3 and its tests |
| P0.5 agents | §4 and its tests |
| P0.6 engine + cli | §7 and §8, `tapes/gate.ron` (§10), the engine and cli tests |
| P0.7 housekeeping | the items below |

P0.7 (PLAN Phase 0 steps 2 and 6; A3):

- Move `docs/reboot/PLAN.md` to `docs/PLAN.md`, and fix the links to it in REVIEW.md, ADDENDUM.md,
  the root `Cargo.toml` and this file.
- Write `STATE.md` at the root as the resume point. It records:
  - the toolchain, 1.97.1 on both machines, with each `rustc -V`;
  - `test_01` failing at every commit where its tests compile, and ADDENDUM §1.4's corrections (A3);
  - the gate world's final hash on WSL and on Windows, and whether the two are equal;
  - the result of `cargo check --target wasm32-unknown-unknown -p rustyecon-engine` once the target
    is installed, recorded and not gated;
  - what session 2 starts from.
- Put up a CI skeleton:
  - `ci/gate.sh` runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`
    and `cargo test --workspace --release`, then prints the gate hash from `rustyecon run
    tapes/gate.ron --until 2080`.
  - `ci/gate.ps1` runs it under `wsl -d ubuntu --exec bash -lc` with a target directory outside the
    tree, then natively on Windows, and reports whether the two hashes agree without failing on a
    difference.

  Whether hosted CI is wanted is the user's choice (A5).

## 13. Left for later: do not build now

| Item | Phase |
|---|---|
| Certificate, criteria, verdicts, NaN scan, manifest (it records `world_id`), telemetry, BalanceWatch, price-runaway detector (A12); N4, N10, N12, N15 | Phase 0 session 2 (A4) |
| Parquet writer (moves as is; `TickReport` is its input) | Phase 0 session 2 |
| Oracle | Phase 1 (other run) |
| Pops as rules (pairs, participation, logit); labour and parcel services as Instant goods; machines and (A, Λ, B) desks; the task margin; the income-identity check | Phase 2 |
| Investment, vintages, depreciation, construction, build lags, user cost; population, technology and enclosure timelines (as world-level `Ext` deltas) | Phase 3 |
| Worldgen compiler and its human-editable tables (the runtime tape is its target) | Phase 4 |
| Transport desks, channel state, pass-through recipes; home-node trading (Phase 0 lets an actor post at any node with no channel and no crossing cost) | Phases 4 and 9 |
| Credit, banks, monetary regimes | Phase 8 |
| Region shards and parallel reduction; sweeps of registered params (each point a tape edit, E1) | when §3.9's budget needs them; Phase 6 |
| crates/gui — the interactive frontend, stack to be chosen | — |

## 14. Open questions

1. Should `Imbalance` be `p·exp(k·x)` rather than July's `p·(1+α·x)`? The two differ only at
   second order in `α·x`, and only the exponential form is tick-invariant (A13).
2. Spoilage runs as core ageing in phase 5a, while the agent hook in 5b stays a no-op. The brief
   asked for phase 5 as a no-op, but lot lives need an ageing step. Is that split right?
3. How far does R4 reach? Is inline structural data under an entry's `basis` enough, or must every
   number on the tape be a named param?
4. Soonest-expiring-first out of the escrow favours the lowest id. Is pro-rata freshness needed
   before Phase 2?
5. `Saturate` or `Hold` for the historical runs is Phase 2's choice (F8); the gate world uses
   `Hold`, and both are tested.
6. Review items taken in part:
   - **Home-node trading is not enforced in Phase 0.** The gate world's mill buys grain in the
     village and no transport desk exists yet, so the rule waits for channels and §13 records the
     simplification.
   - **`SetParam` takes a param key, not a literal**, so that the new value carries a basis (R4).
   - **`world_id` leaves out the tape name and the basis texts.** They change no number a run
     computes, so correcting a note should not invalidate checkpoints.
   - **The permutation half of `decisions_read_phase_start_state` is asserted at tick 0 only.** From
     tick 1 the canonical sums fold in the new id order and may round differently, which is
     legitimate.
   - **The steps start at P0.3, not P0.2**, because this contract is P0.2.
