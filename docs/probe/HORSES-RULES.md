# HORSES-RULES: the stocks probe's agents, as built

Dated 2026-09-28. Steps P2.2.1 and P2.2.2 on branch `phase2-goods`, based on `92ba68e`. Amended at
L0.4 (2026-09-29, branch `phase2-loops`): the maker's reservation, §10.

The goods chain's design (GOODS-CHAIN, `D:/rustyecon-goods/GOODS-CHAIN.md`) puts a registered
probe of machine stocks without loops before its first demo stage (STATE next step 6). The
stocks probe (P2.2a) asks whether the probe's roles, with the horse as a durable good that a
maker builds and a wet capacity desk holds, wears, feeds and hires out by the horse-day, find
unit 1g's equilibrium of v2a.1's horse economy from displaced prices, techniques, stocks and
coins. Its frame is `D:/rustyecon-p2g/frame/HORSES-SPEC.md` (HORSES-SPEC below, sha256
`0e23e809…d5dd`), which fixes the instances, the roles as engine behaviours, the engine's gaps,
the argument that the agents rest at 1g's equilibrium, the protocol and the prediction, written
before any engine run.

This file records what was built, the rules as the frame registers them, and how the build was
checked before registration:

- what changed, and what did not (§1);
- the economies on the engine (§2) and the three new roles (§3);
- the dials and the genesis they imply (§4);
- the harness (§5);
- the checks of the build: nesting, the rest point, the arithmetic, the trace diff, the probes,
  mode A and the run lengths (§6);
- where a result depends on a binding decision (§7);
- where the build departs from HORSES-SPEC, and what stays open (§8);
- the registration (§9);
- the maker's reservation, the idle market's remedy (§10, L0.4).

## 1. What changed, and what did not

- **Core, markets and engine: nothing.**
- **Agents** gained three behaviour kinds, `Maker` (M2), `CapacityDesk` (M3, wet) and
  `OwnerDesk` (M1), in `crates/agents/src/roles/stock/`, new variants of `RawSpec` and `Spec`
  after `TypeDesk`. Every older kind is untouched: the one helper they share with the new kinds,
  the good desk's ex-post assignment, became crate-visible, and nothing else in `roles/rules.rs`
  changed.
- **Three new states**, `ActorState::Maker`, `Capacity` and `Owner`, appended after `MachDesk`,
  so every existing encoding and hash stays (SG9).
- **Load checks** in `Cast::new` (SG7), including M6: a good that lives more than one tick is
  bought only by the capacity and owner desks. It runs after every actor's own checks, so every
  older refusal keeps its path.
- **The probe** gained `probe::horses` (`crates/probe/src/horses/`) and two binaries,
  `horses-tape` and `horses`. The markets probe's classifier became crate-visible so the new
  harness classifies with it; nothing else of `probe::markets` or of P2.0's harness changed, and
  both still make their pinned outputs.
- **The GUI** names the three kinds in its inspector and reads the new states' fields
  (`crates/gui/src/vm/inspector.rs`, `run/extract.rs`, `vm/mod.rs`).
- **Tapes:** `tapes/horses-<id>.ron` for H1–H4, R1a and P7, each its generator's output.
- **The schema stays 1** (docs/TAPE.md, the row for P2.2.1).
- **Hashes.** Every committed tape keeps its canonical form, `tape_hash` and `world_id`; the gate
  world's, appb's and the demo tape's per-tick hash streams are unchanged (finals
  `0x61f9c8529131ff17`, `0xe1fa082b26995867`, `0xfad880fe08d06645`), and the markets probe's
  pins hold (§6.9).

## 2. The economies on the engine

One node, `home`, quotes in `coin`. The goods of a verdict instance (HORSES-SPEC §2.1):

| good | life | who makes it | market rate |
|---|---|---|---|
| `labour` | Instant | `workers`, as an endowment in `decide` | `rate.labour` |
| `land` | Instant | `provider`, as an endowment in `decide` | `rate.land` |
| `fodder` | Ticks(1), through `life.one_tick` | `desk.fodder` (P2.1's `TypeDesk`), in `produce` | `rate.fodder` |
| `horse` | **Indefinite** | `desk.maker` (`Maker`), in `produce` | `rate.horse` |
| `traction` (the horse-days) | Ticks(1) | `desk.capacity` (`CapacityDesk`), in `produce` | `rate.traction` |
| `good` | Ticks(1) | `desk.good` (P2.0's `GoodDesk`, `assign: ExPost`), in `produce` | `rate.good` |

The horse-days are keyed `traction` so that `labour` sorts before them (§8 item 1). The actors:

| actor | kind | class | spec |
|---|---|---|---|
| `desk.good` | Desk | `good_desks` | `GoodDesk` (P2.0), `assign: ExPost` (D-G6) |
| `desk.capacity` | Desk | `capacity_desks` | `CapacityDesk` |
| `desk.maker` | Desk | `maker_desks` | `Maker` |
| `desk.fodder` | Desk | `fodder_desks` | `TypeDesk` (P2.1), own input 0, no bought service |
| `provider` | Pop | `owners` | `BasketProvider` (P2.1), basket [(good, 1), (land, h)] |
| `workers` | Pop | `workers` | `BasketWorkers` (P2.1), the same basket |

Under M1 (P7) `desk.good` is an `OwnerDesk` holding the horses and there is no capacity desk or
horse-day market. R1a (the stocks layer off) keeps I0's keys: the horse is `mach` with life
`Years("life.one_tick")`, its maker `desk.mach` in `mach_desks`, the owner desk `desk.good`, and
there is no fodder (ω 0), so its goods, classes and actors have I0's ids.

The tick is ENGINE §7.2's, as for P2.0 and P2.1 (HORSES-SPEC §2.9). Fodder, horse-days and the
good are made at t, sold and used at t + 1, and die at 5a of t + 1; a genesis lot of them sells
twice, at ticks 0 and 1, and a buyer's unused purchase of it lives to tick 1. A horse ordered at
t is built at t, sells at t + 1 and serves from t + 2. Horses do not age: they leave a holding
by sale or by wear, a `Depreciation` burn at 5b.

## 3. The roles

Notation, per tick at posted prices at the home node: w, r, p_f, p_K, p_h and p are labour's,
land's, fodder's, a head's, a horse-day's and the good's prices; κ and δ are the horse's hours a
head and wear, per tick (`Clock::flow` of a `FlowPerYear`, `Clock::fraction` of a
`FractionPerYear`); share(v) is `Clock::share`; C is the actor's coin as the phase began. No
rule reads a volume, a fill, another actor's holding, orders or state, or the oracle (R13).
`sign(x)` is x where positive and 0 otherwise: the sign of an order, never a cap (R3).

The households, the good desk and the fodder desk are P2.1's and P2.0's kinds, unchanged
(HORSES-SPEC §2.3–§2.5; MARKETS-RULES §3; RULES §2).

### The maker (M2): `Maker`

HORSES-SPEC §2.6. Its params: `own_hours` a (horse-days of its own horses per head built), κ,
the running recipe of those horse-days (`running`: goods and labour per horse-day), the build
recipe (`build`: goods, labour and land per head), δ, s_Km (`adjust`), the cover b_K (`cover`,
`Some` of a `Years` param read as whole ticks, or `None`), its genesis record `own`, and the scale
rule. Its state: `scale`, `output`, `own` (its serving stock after the last wear) and `serving`.

- **Decide.** The bought goods are the running goods then the build's not already listed, each
  a·run_g + build_g per head; labour a·run_lab + build_lab; land build_land. The cost per head
  sums them from 0.0 in that order, then labour, then land: c_m. Its finished stock is
  F = sign(held − own). Its markup is p_K·(1 − δ·a/κ)/c_m; the cash rule gives the outlay and
  q_b = outlay/c_m. Its own stock's target is K*_m = a·q_b/κ, and it moves
  keep = min(sign(δ·K*_m + s_Km·((1 − δ)·K*_m − own)), F) into service: serving = own + keep,
  recorded. It offers sign(F − keep − b_K·q_b·c_m/p_K), at most what the copy holds, and orders
  every input on the whole q_b, budgets p·(coef·q_b) cut from the outlay by P2.1's budget chain.
- **Produce.** y = Leontief over (κ·serving, a), each good held with its coefficient, labour and
  land, over the inputs whose coefficient is not zero. It burns the goods, labour and land
  (`Production`), not its horses, and mints y heads, which join its finished stock.
- **Upkeep.** It burns min(δ·serving, held) heads (`Depreciation`) and records own = serving −
  that.
- **The flow path** (the durable good lives one tick, δ = 1): it runs P2.1's `TypeDesk` code on
  its own params, `own_hours` as the kept own input and the build as the recipe, its state read
  as the type desk's (`scale`, `output`); no upkeep. A load check requires no cover and no
  running recipe then.

### The capacity desk (M3, wet): `CapacityDesk`

HORSES-SPEC §2.7. Its params: `stock` (the horse), `hours` (the horse-days), `labour`, κ, the
running recipe, δ, s_K (`adjust`), the order rule (`Target`; `Held`, CHAIN's A3 as written) and
the scale rule. Its state: `scale`, `output`, `held` (H, its holding when decide ran), `target`
(K*), `order` (m) and `run` (z_plan).

- **Decide.** O = (0.0 + Σ run_g·p_g) + run_lab·w; the full cost O + δ·p_K/κ; the markup
  p_h/(O + δ·p_K/κ); the cash rule gives B. z_plan = κ·H where O is not positive, else
  min(κ·H, B/O). K* = B/(κ·O + δ·p_K). m = sign(δ·K* + s_K·(K* − H)) (`Held`: δ·H in place of
  δ·K*), at most sign(C − O·z_plan)/p_K. It offers every horse-day it holds and orders
  run_g·z_plan of each running good, run_lab·z_plan labour and m horses, budgets p·(coef·qty) cut
  from C by the budget chain.
- **Produce.** z = min(κ·held, `max_scale` of each running input held over its coefficient),
  from the phase-start holding: horses bought this tick serve from the next. It burns the running
  inputs and mints z horse-days (`Production`).
- **Upkeep.** It burns min(δ·held, holding) heads (`Depreciation`): horses bought this tick do not
  wear this tick, so H′ = (1 − δ)·H + m_filled.

### The owner desk (M1): `OwnerDesk`

HORSES-SPEC §2.8, registered for R1a and the P7 family; its paths are not scored (D-G4). Its
params: the good desk's (output, labour, schedule, technique, assign, scale), its durable good
`stock`, κ, the goods an hour of its horses burns (`running`: goods only), δ and s_K. Its state:
the good desk's four fields and `serving`.

- **Decide (stock path).** p_h = δ·p_K/κ + O, O = (0.0 + Σ run_g·p_g); the technique, the cost
  s·w + J(x)·p_h and the cash rule as the good desk's; K* = J(x)·q/κ;
  m = sign(δ·K* + s_K·((1 − δ)·K* − H)), at most the coin left after the hours and the running
  inputs over p_K. Orders: s·q hours, run_g·J(x)·q of each running good, m horses, budgets cut
  from C by the budget chain; it offers every unit of the good it holds.
- **Produce.** serving = the horses it holds now (bought ones included, M1's convention); the
  hours M = min(κ·serving, the running inputs held); the task assignment as the good desk's
  (ex-post on (L, M), or planned); it burns hours and the used hours' running goods, mints y and
  records `used`, `output` and `serving`.
- **Upkeep.** It burns min(δ·serving, holding) heads (`Depreciation`).
- **The flow path**: it runs P2.0's `GoodDesk` code with its durable good as the machine
  services, its state read as the good desk's; no upkeep; no running recipe.

### Load checks (SG7)

At resolve, each a `LoadError` with its path: a stock kind's goods are distinct and not
currencies; a running or build good is none of them and is named once. In `Cast::new`: a
capacity desk's durable good is `Indefinite` and wears less than all of it a tick at genesis;
a maker's or owner desk's is that, or lives one tick at δ = 1 exactly, with no cover and no
running recipe; any other life is refused; and M6. `stock_specs_are_checked_at_load` names each
refusal.

### Why the rest point is the oracle's

HORSES-SPEC §4 argues it: at a live rest point each stock rule forces orders to equal wear, so
K* = H at the capacity desk and serving = a·q_b/κ at the maker; the coin rules then give each
desk's price row (p_h = O + δ·p_K/κ, p_K = a·p_h + build cost, p_f at cost, p = c), and clearing
gives 1g's clearing side, which has one solution at ρ = 0. §6.2 checks it on the engine at every
instance and funded target.

## 4. The dials and genesis

### The dials (HORSES-SPEC §5)

Every dial is a tape param with a unit and a basis (R4). **C2g** (D-G13; registered):

| key | unit | value |
|---|---|---|
| `rate.labour` | RatePerYear | 5.2 |
| `rate.land` | RatePerYear | 0.1625 |
| `rate.fodder`, `rate.horse`, `rate.traction` | RatePerYear | 5.2 |
| `rate.good` | RatePerYear | 2.6 |
| `adjust.technique.good` | RatePerYear | 2.6 |
| `buffer.desk.<desk>.cash` | RatePerYear | 5.2 |
| `tilt.desk.<desk>` | Dimensionless | 0 |
| `spend.workers`, `spend.provider` | RatePerYear | 13 |
| `price.ema_tc` | Years | 0.5 |
| `adjust.invest.capacity` | RatePerYear | 2δ a year (0.2, 0.16, 0.08) |
| `adjust.invest.<maker>`, `adjust.invest.good` | RatePerYear | 0 |
| `cover.<maker>` | Years | 4/52 (4 ticks at 52 a year, 1 at 12, 28 at 365) |

`c2g13` is C2g with `rate.fodder` at 1.3 (F7–F10); `c2` is P2.1's C2m per role (labour 5.2, land
and every machine-side market 1.3, the good 2.6), for R1a and P7. `--set KEY=VALUE` sets a dial,
and `rate.*`, `buffer.*` and `adjust.*` scale a family, as P2.1's `--set` does.

### The instances and their params (HORSES-SPEC §1.2, §2.10)

`probe::horses::instance` holds the 17 instances: H1–H4, F1–F10, R1a, P7 and P8. Each is a
county (N and T a year, h, χ_max, η, g0, g1, k, and the flow machine (a, λ, b)), δ and κ a
year, ω and a configuration (wet or M1). The generator writes rule A's coefficients at the
tape's tick length: fodder's land ω·b; a head's own horse-days a·κ/δ, labour λ·κ/δ and pasture
(1 − ω)·b·κ/δ, κ and δ per tick. Every coefficient is a `Dimensionless` param, live:
`inst.fodder.own` (0), `inst.fodder.labour` (0), `inst.fodder.land`; `inst.<horse>.run.fodder`
(1) and `inst.<horse>.run.labour` (0), the running recipe; `inst.<horse>.kappa` (FlowPerYear,
52), `inst.<horse>.delta` (FractionPerYear), `inst.<horse>.own_hours`, `inst.<horse>.labour`,
`inst.<horse>.land`; and the county's, keyed as I0's. On the flow path (R1a) a head's recipe is
the flow machine's (a, λ, b), written as I0's numbers. A cost shock of the county's b moves
fodder's land and the build's pasture together.

### Genesis (HORSES-SPEC §5.3)

The generator (`horses-tape`) writes each tape from unit 1g's `ChainEconomy`, solved in the
harness with the horse's hours at J = 2 (decision 232; at ρ = 0 every price and quantity is J
1's bit for bit, `horses_oracle_is_the_flow_county_at_rho_zero`):

- prices relative to r = 1 coin, and the good desk's human share 1 − x\*;
- the good desk's good Y and the fodder desk's fodder q_f, one tick's output; the capacity desk's
  heads H = Y·J(x\*)/κ and κ·H horse-days; under M1 the owner desk's heads after wear,
  (1 − δ)·H; the maker's record own = (1 − δ)·a·q_b/κ and its holding own + q_b + b_K·q_b·c_m/p_K;
- each actor's stationary coin, summed in the order its rule sums: the good desk
  p·Y/share(turnover), the capacity desk p_h·κ·H/share(turnover), the maker c_m·q_b/share(turnover),
  the fodder desk ((0.0 + λ_f·w) + b_f·r)·q_f/share(turnover), the provider and the workers as
  P2.1's.

For H1 they are 29.9428245725549 (good desk), 23.78417887464776 (capacity desk),
15.989223641720331 (maker), 11.135650332753471 (fodder desk), 26.03270836105176 (provider),
27.922094099033625 (workers), with the capacity desk holding 3.7089403312974114 heads and as
many horse-days and the maker 1.6270824183032815 heads (own 1.5863284366674937). Every value but
two equals HORSES-SPEC §5.3's to every digit printed; the good desk's and the capacity desk's
coins differ in the last digit (the frame's 29.94282457255491 and 23.78417887464777), since the
frame's came from its Python oracle and these from the Rust chain. The capacity desk's equals
I0's machine desk's to the bit, as §5.3 expects of it within an ulp. R1a's
genesis is P2.1's I0 genesis bit for bit (`horses_r1a_genesis_is_i0s`): the generator calls the
markets probe's genesis on the county's flow economy.

## 5. The harness: `probe::horses`

```sh
horses run NAME...     [options]  # named runs, one summary line each
horses family FAMILY   [options]  # battery, tier3s, stocks
horses list FAMILY     [options]  # names; the battery with its tier
horses tape [NAME]     [options]  # the tape a run is made from
horses kick NAME...    [options]  # the kick set at the end of each named run (§7.5)
horses slowest NAME    [options]  # the kick set's envelope and its slowest mode (§7.4)
horses elasticity      [options]  # the one-tick elasticity probe (§7.8)
horses openloop        [options]  # the open-loop probe, prices frozen (§7.8)
horses point           [options]  # the oracle's point at the base and each cost target
options: --inst ID --tpy N --dials c2g|c2g13|c2 --set KEY=VALUE --assign planned|expost
         --order target|held --cover yes|none --one-sided saturate|hold
         --ticks L --csv DIR --every K --jobs J --horizon H --h X
horses-tape --inst ID [options] [--perturb NAME --ticks L] [PATH]
```

Build under WSL with `CARGO_TARGET_DIR=/root/scratch/target-p2g-<label>`, and put large outputs
under `D:/rustyecon-p2g/`.

### Run names (HORSES-SPEC §7.7)

| term | what it does |
|---|---|
| `hold` | nothing: mode A |
| `w*F`, `r*F`, `p[M]*F` | market M's genesis price times F (`w` is labour's, `r` land's; M `fodder`, `horse`, `traction`, `good`) |
| `s*F` | the good desk's genesis human share times F |
| `x*/2` | the good desk's x = x\*/2 |
| `JA(F)` | w and every price but r times F; s times F |
| `JB(F)` | w and the good's price times F, fodder's, the horse's and the horse-day's times 1/F; s times 1/F |
| `N(F)` | every price times F, coin unchanged |
| `b*F@genesis`, `b=V@genesis` | the county's b is b·F (or V) from tick 0; genesis stays at the registered point |
| `b*F@dated`, `b=V@dated` | b becomes b·F (or V) by dated `SetParam`s at L/4; the scored clock restarts there |
| `coin.A*F` | actor A's genesis coin times F |
| `S*F` | stock S times F: `stock.good`, `heads.capacity`, `hours.capacity`, `heads.good` (M1), `own.<maker>` (record and holding together), `finished.<maker>`, `stock.fodder`; on the flow path `stock.<maker>` |

### The battery (HORSES-SPEC §7.7)

`horses list battery --inst ID` prints it. Each price (w, r, p_f, p_K, p_h, p) and the technique
alone at ±5% (Tier 1), ±20% (Tier 2), ×2 and ×0.5 (Tier 3); JA, JB and N at the same factors;
x\*/2 (Tier 3); b ×1.1 and ×0.9 (Tier 2) and ×2 and ×0.5 (Tier 3), at genesis and dated, where
the target is funded; and Tier 3S, each stock and each coin at ×0.5 and ×2:

| id | runs | Tier 1 | Tier 2 | Tier 3 | Tier 3S |
|---|---|---|---|---|---|
| H1–H4, F1–F4, F7–F10 | 93 | 20 | 24 | 25 | 24 |
| F5, F6 | 91 | 20 | 24 | 23 (b ×2 unfunded) | 24 |

`horses_batteries_are_registered` checks the counts. The family `stocks` (§7.10 item 9) is every
coin ×0.02 and ×0.1 and every stock ×0.1 and ×10.

### Observables, targets and classes (HORSES-SPEC §7.1–§7.5)

Every tick the harness reads, from the `TickReport` through certify's `Obs` and the actors' own
state (agents see none of it), the observables O, 18 at a verdict instance: v; fodder's, the
horse's, the horse-day's and the good's prices over r; the good desk's human share `s.good` (the
cutoff its ex-post production used); the cleared volume of labour, land, fodder, horses,
horse-days and the good; the good desk's output, the capacity desk's horse-days, the maker's
heads and the fodder desk's fodder; and `heads.capacity` (its `held`) and `heads.maker` (the
maker's `serving`). Under M1 the horse-day market and output go, and `heads.good` (the owner
desk's serving) replaces `heads.capacity`. On R1a they are I0's ten, in I0's order.

The targets are 1g's at the coefficients in force, solved in the harness at genesis and at each
dated shock: the horse market clears q_b·(1 − δ·a/κ), the heads the maker does not keep; the
horse-day market Y·J(x\*), the tasks' horse-days; on R1a the targets are P2.1's, from unit 1c in
operating form, so they are I0's bit for bit. D̂ = max_o |ln(o/o\*)|/1e-3. A tick is **dead** if
labour, land, fodder, horse-days or the good fails to trade or clears less than 0.5 of its
oracle volume; the horse market is **idle**, not dead, when it clears less than 0.5 of its
volume, and its idle ticks and its ticks with no order are reported (§7.2). On R1a every market
counts, as in I0.

The classes are PROBE-SPEC §4.5's, classified by the markets probe's streaming classifier
(ERROR, DIVERGED with the runaway bound [1e-6, 1e6] × genesis, DEAD, VACUOUS, CONVERGED, STUCK,
ORBITING). One change, §7.5's: a stock or coin displacement's start distance D̂₀ is the largest
D̂ of its first year (tpy ticks); on R1a it stays P2.0's, |ln F|/1e-3, so R1a nests I0.

**The kick** (§7.5): `horses kick NAME` runs the named run to its end and runs certify's kick set
there (`certify::kick::kick_segment`), each market's posted price × (1 ± 1e-9) through
`ScalePrice` for H ticks, judged by `criteria/appb-2026-09-26.ron`'s bars; 12 kicks at a verdict
instance. `horses slowest NAME` prints the kicks' envelope and its decay, g (§6.6).

### Statistics (HORSES-SPEC §7.11)

`summary.tsv` has one line per run: the class and PROBE-SPEC's numbers, P2.1's transient
statistics, and the stocks': the peak D̂ without the horse market's volume; the installed heads'
lowest and highest over their target; the tick from which they stay within 5% of it, and the
paper's time (1 tick for an expansion, ln(K′/K)/ln(1 − δ) for a contraction); the quasi-rent
p_h/(O + δ·p_K/κ) − 1 at posted prices 3 ticks and round(1/δ) ticks after the scored clock
starts (D-G14); the ticks with no horse order and the idle ticks; the horse's lowest price over
its target; the maker's highest finished stock over its rest value; the capacity desk's lowest
utilisation z/(κ·held); the hour price's lowest over its running cost; horses bought, trough
and peak over their volume; and the tick from which GOODS-CHAIN's nine stay within tolerance.
`stats.tsv` holds them and P2.1's in long form. The CSV of a run holds every price, observable,
target, gap and D̂, each market's S, D, fills and spoilage, every coin, the transfer, the
baskets and binding item, the ledger margin, the dead, idle and no-order flags, and the stock
records (the tasks' heads, the maker's serving, own and finished stock, the capacity desk's K\*,
order and planned hours, the running and full costs at posted prices).

## 6. Checks of the build

All runs below are on WSL, in a release build of the P2.2.1 tree, at 52 ticks a year unless
said. Scripts and outputs are in `D:/rustyecon-p2g/build/`.

### 6.1 Nesting: the stocks layer off (R1a)

- **Run level** (`horses_r1a_nests_i0`, in the gate). R1a in the stock kinds on the flow path,
  through this harness, against I0 in P2.1's kinds through the markets harness: every tick's
  posted prices, observables, targets, gaps, D̂, supply, demand, both fills, spoilage, coins,
  planned share, transfer and ledger margin are equal bit for bit, for 20,000 ticks from the
  registered genesis and 3,000 each from `w*2`, `JB(0.5)`, `b=0.2@genesis` (I0's
  `land.mach=0.2@genesis`) and `stock.mach*0.1`; the classes, D̂₀, envelopes, W's largest D̂,
  dead ticks, troughs and transfer shortfalls agree.
- **The ex-post variant** (`horses_r1a_expost_nests_appb_expost`). R1a with `assign: ExPost`
  against appb's registered ex-post variant, P2.0's roles through P2.0's harness: the same
  numbers bit for bit on the same five runs, with the classes, dead ticks and D̂₀.
- **Rule level** (`stock_roles_nest_the_flow_roles`). From 3,000 random states (prices over
  twelve decades, coins, stocks, the technique at 0, 1 and between, spending, turnover and
  technique rates, tilts up to 8), the owner desk decides and produces exactly what P2.0's good
  desk does, and the maker exactly what P2.1's type desk does, order for order and delta for
  delta once their states are read as the older kinds'; neither wears.

### 6.2 The rest point is the oracle's

`horses_rest_point_is_the_oracles`: at every instance (all 17) and every funded cost target (b
×1.1, ×0.9, ×2, ×0.5; F5's and F6's b ×2 are not targets), three ticks from genesis at the
oracle's f64 point leave every observable within 1e-12 of the oracle in log, every market
trading (the horse market too) with both fills 1 to rounding; and the verdict instances pass the
same hold check at 12, 24 and 365 ticks a year. `horses_oracle_is_the_flow_county_at_rho_zero`
checks the collapse (R1b): at ρ = 0 the chain's x\*, v, P_s, Y, N_a, horse-day price and
horse-days equal the county's flow economy within 1e-15 relative at H1–H4, F1–F6 and P8 and
every target, and J = 2 gives J = 1's prices and quantities bit for bit.

### 6.3 Arithmetic that cannot fail, and conservation

- `stock_roles_never_overbudget_or_overdraw` draws 3,000 states on H2 and 3,000 on P7 (prices
  over twelve decades, coins and stocks over eighteen, spending shares that round to 1, tilts up
  to 8, the maker's record one ulp above its holding) and checks that admission accepts every
  order, every burn in produce fits, and every wear at upkeep fits. With the horse-days keyed
  `hday`, which sorts before `labour`, the same test fails on H2: P2.0's good desk takes its
  labour budget before its machine services' while admission takes them in good order, and with
  a share that rounds to 1 the second budget can pass the coin by half an ulp (MARKETS-RULES
  §3's defect). Hence §8 item 1.
- `horses_conserve_every_tick`: on every committed tape, at rest and from w ×2 with every desk's
  coin ×0.1 (and the capacity desk's heads ×2), each tick's ledger and the run's close, the money
  stock drifts by at most 1e-12 of itself over 2,000 ticks, and every `Depreciation` burn is in
  upkeep, exactly δ times the stock its desk recorded as serving, and moved what it asked for.
  Stocks wear; R1a's flows do not.
- `horses_tapes_load_and_run_deterministically`: two runs of each tape give identical reports and
  hash streams for 500 ticks.

### 6.4 The trace diff (HORSES-SPEC §7.8)

`D:/rustyecon-p2g/build/tracediff.py` runs a float64 mirror of §3's roles from each Rust run's
own genesis (its tape, which also sets the mirror's coefficients) and compares every observable
in log, every tick, for 2,000 ticks. The mirror is `h_carry.py`, which `make_h_carry.py` writes
from the frame's `h_mirror.py` (sha256 `042f12b2…428e`, read-only) with the engine's genesis
carry added: the sellers' unsold genesis lots (the good, fodder, horse-days) offered again at
tick 1, and the buyers' unused purchases of them (the good desk's horse-days, the capacity
desk's, the maker's and the owner desk's fodder) used at tick 1. The last column is the mirror
against itself: one coin or one horse stock moved by an ulp, each in turn. "Volumes, absolute"
is the worst |build − mirror| of a cleared volume over its oracle volume. Output: `tracediff.out`.

| run | ticks | largest gap in log | largest but volumes | volumes, absolute | the mirror against itself |
|---|---|---|---|---|---|
| H1 hold | 2,000 | 3.3e-15 | 1.8e-15 | 3.2e-15 | 2.4e-15 |
| H1 w×2 | 2,000 | 1.3e-13 | 5.2e-14 | 1.3e-13 | 1.2e-14 |
| H1 JB(0.5) | 2,000 | 3.0e-13 | 6.6e-14 | 1.3e-13 | 1.5e-14 |
| H1 b ×2 at genesis | 2,000 | 7.3e-13 | 7.0e-14 | 1.2e-13 | 6.6e-14 |
| H1 heads ×2 (the glut) | 2,000 | 1.4e-8 | 1.3e-9 | 3.6e-9 | 1.3e-9 |
| H1 finished ×2 | 2,000 | 1.0e-13 | 6.7e-14 | 1.0e-13 | 1.0e-14 |
| H2 hold | 2,000 | 3.7e-14 | 2.3e-14 | 3.7e-14 | 2.8e-15 |
| H2 w×2 | 2,000 | 3.3e-13 | 1.7e-13 | 3.3e-13 | 2.8e-13 |
| H2 JB(0.5) | 2,000 | 1.1e-12 | 4.5e-13 | 4.0e-13 | 1.8e-13 |
| H2 b ×2 at genesis | 2,000 | 3.6e-11 | 1.2e-12 | 1.9e-13 | 2.0e-13 |
| H2 heads ×2 (the glut) | 2,000 | 5.5e-4 | 7.1e-6 | 1.2e-6 | 3.1e-4 |
| H2 finished ×2 | 2,000 | 3.0e-13 | 1.7e-13 | 3.0e-13 | 1.7e-13 |

**The build is the mirror's map.** Eight of the twelve runs agree within 1e-12 on every
observable. The four that do not part on the horse market's cleared volume first:

- **The cancellation.** The horse market clears the capacity desk's order,
  δ·K\* + s_K·(K\* − H). After b ×2 and after heads ×2 the stock is above its target and the order
  is a small difference of terms of order 1e-2 heads: 3.6e-6 at its smallest on H2's b ×2, 1/2000
  of its oracle volume. A rounding difference of the terms (the two sides' evaluation orders
  differ: the stock's law of motion (H + m) − δ·H against (1 − δ)·H + m, the maker's finished stock
  held − own against a state of its own, `max_scale` against division) is then magnified by their
  ratio in log. In absolute terms, over the oracle volume, every volume agrees within 4.0e-13 on
  H2's JB(0.5) and b ×2. Through the maker's revenue the same difference reaches its output:
  H2's b ×2 is 1.2e-12 on the maker's heads at tick 113, where it makes 5e-5 heads, 1/200 of its
  rest.
- **The glut runs** (heads ×2) cross the order's zero repeatedly, where `max(·, 0)` is a kink:
  the mirror against itself one ulp apart parts by 1.3e-9 at H1 and 3.1e-4 at H2, and the build
  stays within those on everything but the horse volume, whose absolute gap is 3.6e-9 and 1.2e-6
  of its oracle volume.

It is an explained disagreement in a measured quantity at the order's cancellation, not in the
map, and it does not block scoring (HORSES-SPEC §7.8; MARKETS-RULES §6.4's standard). The mirror
and the engine also agree on H1's base kick set: the engine's kick envelope and the mirror's
(w, r, p, p_f, p_K, p_h each × (1 ± 1e-9) at the same state) read 0.122 at 2,000 ticks, 6.39e-3 at
3,000 and 5.24e-4 at 4,000, both, before both reach their rounding floors (§6.6).

**M1**, which the frame's §7.8 does not ask for, was diffed too (P7's instance, with the owner
desk's own runs): hold and heads ×2 agree within 6.7e-14; w ×2, JB(0.5) and finished ×2 within
7.6e-12, 2.2e-10 and 2.1e-11, growing slowly from about 1e-14 as the per-tick evaluation-order
differences integrate along P7's slow mode, where the mirror against itself (a one-time ulp)
stays within 1.2e-13 to 4.1e-12; after b ×0.5 both the build and the mirror against itself part
by order 1 (0.17 in the twin) on P7's chaotic path (HORSES-SPEC §6.1 item 2 found `goods.py`
against itself parting by 0.022 there).

### 6.5 The probes (HORSES-SPEC §7.8)

**The one-tick elasticity probe** (`D:/rustyecon-p2g/build/probes/el-*.out`), τ in ticks:

| id | the slowest markets at 52 a year (τ) | 200·τ_max at 12 | at 24 | at 52 | at 365 |
|---|---|---|---|---|---|
| H1, H3 | land 388.6, fodder 159.5, good 67.7; labour 7.4 | 15,500 | 33,520 | 77,720 | 584,300 |
| H2, H4 | land 374.0, good 67.7, fodder 40.9; labour 7.3 | 15,000 | 32,340 | 74,800 | 560,840 |

Under C2g land's price moves at 0.1625 a year, one eighth of C2's, so its time constant is eight
times P2.1's (47.5 ticks at I0) and sets the run length's second term (§6.7). The horse market's
τ is 1.7–2.0 ticks: its demand and supply are stocks that answer a price at once.

**The open-loop probe** (`probes/ol-*.out`), prices frozen, a 1% move in each price: every
response stays finite at lag 200. The largest is the horse market's (−42 per unit of the wage at
H1, +90 per unit of land's price): with prices frozen the capacity desk's order integrates the
stock gap, so its demand drifts, but no market goes to a loop's ±∞ (P2.1's L2 did by lag 200).
There is no quantity loop, as v2a.1 has none.

### 6.6 The engine's slowest mode and the kick's floor

`horses slowest hold` runs the base kick set and fits g, the per-tick factor of the least-squares
line of ln G(t), G the largest g(t)/g(T) over the 12 kicks, from its peak in the first half of
the horizon to the first tick it falls below 10 times its rounding floor (the median of its last
tenth). A 1e-9 kick at a point whose slowest mode is 1 − ε a tick freezes about 1e-16/ε from the
base run: some 4e-5 of the kick at δ 10%, reached near tick 4,900. The frame's "second half" of a
3·T6 horizon lies on that floor, so the fit reads the whole descent instead (§8 item 8).
(`D:/rustyecon-p2g/build/lengths/`):

| id | g a year (engine) | the mirror's PL (HORSES-SPEC §6.2) | 3·T6 (engine) | 3·T6 (mirror) |
|---|---|---|---|---|
| H1 | 0.891 | 0.910 | 19,000 | 24,000 |
| H2 | 0.845 | 0.895 | 13,000 | 20,000 |
| H3 | 0.912 | 0.929 | 24,000 | 30,000 |
| H4 | 0.877 | 0.916 | 17,000 | 25,000 |
| F1 | 0.945 | 0.985 | 39,000 | 178,000 |
| F2 | 0.932 | 0.982 | 31,000 | 258,000 |
| F3, F4 | 0.872, 0.878 | 0.905, 0.924 | 16,000, 17,000 | 22,000, 29,000 |
| F5, F6 | 0.870, 0.852 | 0.921, 0.901 | 16,000, 14,000 | 27,000, 21,000 |
| F7–F10 | 0.894, 0.861, 0.913, 0.887 | 0.910, 0.896, 0.930, 0.921 | 20,000, 15,000, 24,000, 19,000 | 24,000, 20,000, 31,000, 27,000 |

The kick envelope decays faster than the mirror's PL at every instance. The mirror's own kick
envelope at H1 is the engine's (§6.4), so the difference is in the reading, not the map: PL is
the top Lyapunov growth of a random displacement of the whole state (coins and stocks included)
over 80 years, a kick displaces one price and is read over the 4.4 decades above its floor. The
engine's g never lengthens L, so the mirror's 3·T6 stands as §7.4's third term.

### 6.7 Run lengths

HORSES-SPEC §7.4: L is the largest of 20,000·tpy/52 rounded up to a thousand, 200·τ_max from the
engine's elasticity probe rounded up to a thousand, and 3·T6 (the mirror's, the engine's being
shorter). At 52 a year:

| id | floor | 200·τ_max | 3·T6 | **L** |
|---|---|---|---|---|
| H1, H3 | 20,000 | 78,000 | 24,000, 30,000 | **78,000** |
| H2, H4 | 20,000 | 75,000 | 20,000, 25,000 | **75,000** |
| F1, F2 | 20,000 | 78,000, 75,000 | 178,000, 258,000 | **178,000, 258,000** |
| F3, F4 | 20,000 | 77,000 | 22,000, 29,000 | **77,000** |
| F5, F6 | 20,000 | 84,000 | 27,000, 21,000 | **84,000** |
| F7, F9 | 20,000 | 128,000 | 24,000, 31,000 | **128,000** |
| F8, F10 | 20,000 | 75,000 | 20,000, 27,000 | **75,000** |

The frame's L (20,000–31,000 at H1–H4 and F3–F10) had the first and third terms only; the
second, which only the engine's probe gives, is two and a half to five times longer at every
C2g instance. At the other tick lengths, for the verdict instances: at 12 a year 16,000 (H1, H3)
and 15,000 (H2, H4), the elasticity term (wet hire is unstable there, so it has no T6); at 24,
34,000 and 33,000; at 365, 585,000 and 561,000.

### 6.8 Mode A (HORSES-SPEC §7.6)

From the oracle's f64 point with the stationary coins and rest stocks, at L (§6.7), every
observable within 1e-9 of the oracle in log, every market trading (the horse market too) with
every fill at least 1 − 1e-9, spoilage at most 1e-9 of volume, the ledger clean, and the base
kick set at H = L (`D:/rustyecon-p2g/build/modea/`):

| id | 12 a year | 24 a year | 52 a year | 365 a year |
|---|---|---|---|---|
| H1 | **FAIL** at tick 709 (π_horse); ORBITING; kicks FAIL | PASS, 1.2e-14; kicks PASS (2.3e-5, 2.2) | PASS, 2.4e-15; kicks PASS (4.0e-5, 2.0) | PASS, 7.5e-14; kicks PASS (2.1e-4, 1.8) |
| H2 | **FAIL** at tick 222 (π_horse); DEAD; kicks FAIL | PASS, 1.9e-14; kicks **FAIL** (3.6e6) | PASS, 3.7e-14; kicks PASS (6.8e-5, 4.8) | PASS, 7.2e-14; kicks PASS (2.9e-4, 4.4) |
| H3 | **FAIL** at tick 746 (the horse's fill); ORBITING; kicks FAIL | PASS, 1.7e-15; kicks PASS (2.5e-5, 2.2) | PASS, 4.9e-15; kicks PASS (4.5e-5, 2.0) | PASS, 6.2e-15; kicks PASS (3.3e-4, 1.8) |
| H4 | **FAIL** at tick 213 (the horse's fill); DEAD; kicks FAIL | PASS, 2.1e-15; kicks **FAIL** (3.4e5) | PASS, 1.8e-14; kicks PASS (4.6e-5, 4.8) | PASS, 4.1e-15; kicks PASS (3.3e-4, 4.4) |

Each PASS cell gives the largest gap in log over the run, and each kick set its largest
`gain_tail` and `gain_peak` (a failed set its largest `gain_tail`). Mode A at 52 a year passes
at every verdict instance with its base kick set, the verdict's precondition (§7.9). HORSES-SPEC §6.2 predicts wet hire unstable at 12 a year everywhere
and at 24 a year at H2 (1.00023 a tick), and stable at H4 at 24 a year (0.99983 a tick): the
engine agrees at 12 and at H2, and finds H4 at 24 a year unstable too, its kicks growing 3.4e5 in
33,000 ticks. H4 at 24 a year is the frame's slowest stable cell, 0.996 a year, where a PL from
four random directions can miss a slow cone (HORSES-SPEC §6.6); it is a family cell (P3 (iii),
P5), reported.

### 6.9 Determinism, platforms, the gate

- **The gate at P2.2.1** (`d636b76`). `scripts/gate.sh` is green in WSL
  (`CARGO_TARGET_DIR=/root/scratch/target-p2g-build`) and on Windows under Git Bash
  (`D:/rustyecon-targets/p2g-build`, a fresh target). On each machine 865 tests pass in the
  workspace (the 846 before P2.2.1 and the 19 of §6: 7 in agents' `tests/stock.rs`, 12 in
  probe's `tests/horses.rs`; the seam's sites test reads three horses tapes too), with 3 ignored
  and run by name, and zero warnings; certify alone, Parquet-free, passes 67 with 1 ignored. The gate
  hash is `0x61f9c8529131ff17`, the stamp matches the clean checkout, both certificates
  recompute, the probe's pins hold (`probe_battery_csv_unchanged`), the demo world runs to 1901,
  and telemetry is identical from two processes. The GUI's recorded check passes in WSL.
  Logs: `D:/rustyecon-p2g/build/gate/`.
- **The GUI's gate.** `scripts/gui.sh` is green in WSL and on Windows: 117 tests pass with 4
  ignored, its 87 named tests among them, and the cli's hashes equal the GUI's for gate (2,080
  ticks, final `0x61f9c8529131ff17`), appb (20,000, final `0xe1fa082b26995867`), demo-gb (7,852,
  final `0xfad880fe08d06645`) and the two branch tapes (finals `0x9fc2f964a8510756`,
  `0xd057e3ea708da495`).
- **The markets probe's pins.** `markets_i0_nests_appb` and
  `markets_tapes_are_their_generators_output` pass in the gate, and the cli's per-tick hashes of
  the seven markets tapes over 2,000 ticks end where MARKETS-RULES §6.8 recorded them (I0
  `0x6c5bf916f3b69d35`, I1 `0x8e3f1bc77c0e5a91`, I2 `0x958d5fb01a4847b5`, I3
  `0x6749dd3193ba1712`, L2 `0xf45a65d427f9f629`, L3 `0x010ae4da4eecfef5`, G1
  `0xe71ac0e96df39297`).
- **Platforms (recorded).** The cli's per-tick hashes of the six horses tapes over 2,000 ticks are
  byte-identical on WSL and Windows (finals: H1 `0x1fd71d01a36007e2`, H2 `0x2ed5d1f6af357256`,
  H3 `0x975cff07e2f45faa`, H4 `0x4af24f45e5cd1473`, R1a `0xb096a7fd8cc5d349`, P7
  `0xd5e3b838be169840`), as are I0's and G1's. `D:/rustyecon-p2g/build/hashes/`.

## 7. Where a result depends on a binding decision

- **60, 61, 70** as for P2.1: one task line, a continuous threshold, one equilibrium (by proof
  at ρ = 0, and by count: HORSES-SPEC §1.3's 70 solves).
- **179 (D-G1) and 186 (E2).** Horses are built from horse-days, labour and land, never from the
  good, and the good buys horse-days only through its tasks, so the machine side is upstream of
  x and there is no goods-and-machines loop; the maker's own horses' days are its kept input.
- **180 (D-G10).** Without it unit 1c refuses every instance (A0's physical radius 148.2).
- **121.** Weekly ticks: §6.8 finds wet hire unstable at 12 a year at every verdict instance and
  at 24 a year at H2 and H4.
- **D-G6.** The good desk assigns tasks ex post; P2.0's `GoodDesk` does it, unchanged.
- **232.** The harness solves the horse's hours at J = 2; at ρ = 0 it is J = 1 bit for bit.

## 8. Departures from HORSES-SPEC, and what stays open

1. **Names.** The run's names are `horses`: `probe::horses`, the binaries `horses-tape` and
   `horses`, `tapes/horses-<id>.ron`, the tests `horses_*` (the frame's `stocks`). The horse-days
   are keyed `traction`, not `hday`: HORSES-SPEC §2.1 requires `labour` to sort before the
   horse-days (GoodId order is key order), and `hday` sorts before it; §6.3 shows the defect it
   would bring back. The horse-day's running recipe is keyed by the horse,
   `inst.<horse>.run.fodder` and `.run.labour` (the frame's `inst.hday.*`), so that R1a, which has
   no horse-day, keys its maker's running labour as `inst.mach.run.labour`. The horse's own params
   are `inst.<horse>.kappa`, `.delta`, `.own_hours`, `.labour`, `.land`.
2. **The maker's cover is optional.** Core's `Ticks` conversion refuses 0 ticks (a span lasts at
   least one), and a 0-year cover would also stop the registry's listing, so no cover is `cover:
   None` rather than b_K 0: the flow path requires `None`, and `--cover none` is the P3 (v)
   control (the maker's finished heads offered in full).
3. **The capacity desk records `run`** (z_plan) beside `held`, `target` and `order`; the owner
   desk records `serving` only (its order is read from the market).
4. **The owner desk's running recipe is goods only by structure** (no labour field), not by a
   load check.
5. **M6 runs after every actor's own checks** (§1), so that a tape an older check refuses reports
   the older error (`role_specs_are_checked_at_load` found the order).
6. **A wear burn is at most what is held**: min(δ·recorded, held). At every reachable state it is
   δ·recorded (`horses_conserve_every_tick` checks it exactly); the min keeps a hand-set state
   from stopping a run.
7. **R1a's targets are P2.1's** (unit 1c in operating form) rather than `ChainEconomy` at δ = 1,
   which agrees with them within 4.1e-16 but not bit for bit (R1b); the run-level nesting needs
   bit-identical targets. R1a's stock displacements keep P2.0's D̂₀ for the same reason.
8. **The engine's g** is fitted over the kick envelope's whole descent above its rounding floor
   (§6.6), not over the second half of a 3·T6 horizon, which lies on the floor. It is shorter
   than the mirror's at every instance, so it changes no L.
9. **L is longer than the frame's numbers** (§6.7): the frame's rule, with the engine's
   elasticity probe, gives 75,000–258,000 ticks at 52 a year; the frame quoted 20,000–258,000 from
   the floor and 3·T6 alone. Tier 3 and 3S run at 10·L, and every kick horizon is L.
10. **The cost shock's names** are `b*F@genesis|dated` (and `b=V@…`); a shock's values are b·F in
    floating point, as the frame's `check` computed its targets (0.44000000000000006 for b ×1.1
    on A0).
11. **The quasi-rent's second reading** is at round(1/δ) ticks of δ a tick (494 at δ 10%).
12. **P7 registers `assign: ExPost`**, the frame's mirror's default for M1.
13. **The trace diff's mirror** is `h_mirror.py` with the carry added, and its runs are the
    frame's six on H1 and H2 (with `heads.capacity*2` and `finished.maker*2` for "Hc x2" and
    "F x2"); M1 was diffed beside them.
14. **Not built here:** the batteries and families (P2.2.3), the report and STATE.md's decisions
    and open items (P2.2.4). The harness, `horses kick` and `horses slowest` provide each.

## 9. Registration

The run's registration is `D:/rustyecon-p2g/run/registration.md` (sha256
`2baaa34d93fa1ba154abf11c1d07521a53493f16ea7e0395a3c8acf2c75af4c6`, in `registration.sha256`
beside it), written at P2.2.2 before any mode-B run and committed with it as
`docs/probe/results/horses/registration.md`, beside its sha256 and its run lists. It names P2.2.1
(`d636b76`, clean) and freezes the frame by sha256 with its prediction (§6) and protocol (§7)
copied unedited, this file at P2.2.1, the roles', the harness's and the tapes' sources by sha256,
the instances and targets, the dials, the tolerance, L per instance and tick length (§6.7), the
kick, the battery and its lists, the verdict rules and the families, with §6's checks made
before it.

**Runs read before registration, disclosed** (R5; HORSES-SPEC §5.4's forking path). Beside
§7.8's checks, the harness was smoke-tested while it was built, at the frame's L: seven H1 runs
(`w*2`, `JB(0.5)`, `b*2@genesis`, `b*0.5@genesis`, `heads.capacity*2`, `finished.maker*2`,
`coin.desk.capacity*0.5`, 24,000 ticks each), all CONVERGED, with the b ×2 trough of baskets at
0.645, 11 dead ticks and 29.1 years to 5% (the frame's 0.646, 11 and 29.1), the b ×0.5 quasi-rent
+0.004 and +0.415 (the frame's) and the glut's 53 years to 5%; and P7's `b*0.5@genesis` (40,000
ticks), trough 0.425 and 322 dead ticks (the frame's), CONVERGED where the frame registers STUCK.
No rule, dial, instance or scoring choice was made or changed after them; the one change to the
harness after them is the reading of the engine's g (§6.6, §8 item 8), made on the base kick set's
envelope, which moves no L.

## 10. The maker's reservation (L0.4, 2026-09-29)

Steps L0.4 and L0.5 on branch `phase2-loops`. O47, the idle machine market: in a glut the
capacity desk orders nothing for years, the maker still offers its finished heads, and the horse
market has an offer and no bid. Under `Saturate` its price falls by e^(−k) a tick (0.1 at C2g) until the
runaway bound, which heads.capacity × 10 reached at every instance and P8 at A0 (HORSES §3–§4).
The remedy is the one the mirror scan chose, `D:/rustyecon-p2l/idle-scan/IDLE-SPEC.md` (sha256
`c5c6527a…`), registered before it was built (`docs/probe/results/idle/registration.md`, L0.3).

**The rule.** The maker offers none of its finished heads while its net markup at posted prices
is below ψ:

    μ = p_K·(1 − δ·a/κ)/c_m;   offered = 0 if μ < ψ, else the offer of §3.

μ is the markup the maker already forms for its cash rule (§3), 1 at rest. Below ψ it holds its
finished heads, unworn (they are not its serving stock), and it still buys and breeds on its
whole q_b by its cash rule; its keep, serving stock and state are as they were. When it
withholds, markets' `next_price` does the rest, unchanged: with no bid the imbalance is 0 and the
price holds bit for bit; with a bid it is +1 and the price rises by e^k. So the price falls only
on ticks when the maker offers, which needs μ ≥ ψ, and then by at most e^(−k).

**Where.** `crates/agents/src/roles/stock/`: `RawMaker` and `Maker` gain `reserve`, an
`Option<Key>` of a live `Dimensionless` param (a value, `ClockMethod::Value`), listed among the
maker's sites as `reserve`; `MakerRole::decide` changes the one line that sets the offer's
quantity. It is the one field of a stock kind that may be absent (`#[serde(default,
skip_serializing_if = "Option::is_none")]`, as criteria's `price_shocks`): absent or at ψ 0 the
rule is off, since `μ < 0` is false for every markup the maker can form (p_K > 0, c_m > 0, and a
NaN compares false). Every committed tape keeps its canonical text, `tape_hash` and `world_id`,
and no state changes. The resolved `Maker` leaves the field out when it is `None` too, since
`world_id` hashes the resolved actors by bincode: L0.4 as first committed wrote a `None` there,
and every tape with a maker kept its text, `tape_hash` and per-tick hash stream but took a new
`world_id`. The registration's check of the hashes found it before any scored run, and L0.5 put
it back. The capacity desk, the owner desk, the markets and the engine are unchanged.

**The value.** ψ = 0.25 (IDLE-SPEC §5), the param `reserve.<maker>` (`reserve.maker` at every
stock instance, as `cover.<maker>`; the spec's parenthesis `reserve.desk.maker` was read as a
slip at registration), basis `Assumed("IDLE-SPEC 2026-09-29, mirror scan")`.

**Load checks** (`Cast::new`, beside the maker's others), each a `LoadError` at
`actors[<maker>].spec.reserve`:
- on the flow path: "the flow path holds no reservation";
- in a world whose `market.one_sided` is `Hold`: a market with a bid and no offer keeps its price
  there, so a maker that withholds would never see it rise.

A `reserve` param of any unit but `Dimensionless` is refused where the maker resolves it (a unit
mismatch at the same path). Every param is finite and not negative (core), so ψ is too.

**The harness.** `horses ... --reserve PSI` and `horses-tape --reserve PSI` write the param, after
the dials, and the field on the maker; without the flag the tape is P2.2a's byte for byte.
`Setup` carries it as `reserve: Option<f64>`; on the flow path the generator refuses it. Each tick
the harness forms the maker's markup from the tick's posted prices (those the settlement used,
which `decide` read) and the coefficients in force, summed in the rule's order
(`harness::MakerCost`), outside the Sim. The CSV gains `markup` and `withheld` at the end of each
row, and `summary.tsv` three columns after P2.2a's last: `withheld` (scored ticks with μ < ψ),
`switches` (changes between withholding and offering over the scored ticks) and `markup_low`
(μ's lowest over them); `stats.tsv` has them as `idle.*`. Every P2.2a column keeps its place and
meaning.

**Tests** (each fails with its change undone; `D:/rustyecon-p2l/idle-engine/build/mutants/`):
- agents, `tests/stock.rs`: `maker_withholds_below_its_reservation` (at a markup of 0.2 with
  ψ 0.25 a sell of 0 heads, every buy and the state as P2.2a's, and P2.2a's offer at 0.3, with
  `reserve` absent, and with ψ set to 0 at run time) and `reserve_is_checked_at_load` (the two
  refusals and the unit, each with its path; H1 with it loads, lists the site as a value and
  round-trips in canonical form; without it the canonical text has no `reserve`);
- probe, `tests/horses.rs`: `reserve_holds_the_idle_horse_price` (H2's glut at ψ 0.25 over 400
  ticks keeps the horse price within [0.1, 1] of genesis, holds it bit for bit on every tick with
  neither offer nor bid, offers nothing on every tick read as withheld; without the flag it
  leaves the bound at tick 138), `reserve_absent_or_zero_is_p22a` (ψ 0 against no field at H1–H4
  and H2's glut, every number bit for bit for 2,000 ticks or until P2.2a's runaway at 138),
  `reserve_leaves_the_rest_point` (mode A at H1–H4 with ψ 0.25 is mode A, bit for bit, for 2,000
  ticks; at ψ 1.01 it withholds from tick 0 and parts) and `reserve_conserves_every_tick` (H2's
  glut at ψ 0.25 for 2,000 ticks: each tick's ledger, the money stock, every wear burn δ times the
  recorded stock, and on each tick the maker offers nothing its heads move only by what it made
  and what wore), and, at L0.5, `horses_tapes_keep_their_world_ids` (the six committed horses
  tapes keep the `world_id` P2.2 recorded; with the reservation the world is another).

**What does not move.** The gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb
`0xfad880fe08d06645` (stream `0xdb63cc96f769fb3e`), the probe's pin, the markets probe's pins, the
horses tapes and their per-tick hash streams (§6.9), and the tape schema, 1 (docs/TAPE.md).

**The runs** are in `docs/probe/results/idle/` (its README), against the registration.
