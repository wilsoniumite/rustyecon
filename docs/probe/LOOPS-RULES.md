# LOOPS-RULES: the loop step's agents, as the build must make them

Dated 2026-09-30. Step P2.2b.0 on branch `phase2-plants` (worktree `D:/rustyecon-wt/p2b`, from
`reboot` at `8b07c8a`). Written before any P2.2b code exists. The build (P2.2b.1) amends this
file "as built", as HORSES-RULES was.

**Amended at P2.2b.1 (2026-09-30): as built.** §14 says what the build made, where it departs
from §1–§13, what it checked, and what it leaves to the harness step. The text above §14 is the
spec as P2.2b.0 wrote it.

**Amended at P2.2b.2 (2026-09-30): the harness as built, and E0–E2.** §15 says what the harness
step made, where it departs from §8–§10 and §13, and what E0, E1 and E2 found
([results/loops/e0.md](results/loops/e0.md)). E0 passes and needs no amendment.

P2.2b asks whether rule B's horse runs on the engine as the mirror registered it. Fodder is
raised with horse-days (the loop). The horses are held by M3's wet capacity desk and bred by M2
from bought fodder. CAPACITY's plant, y = K^(1−θ)·z^θ, sits on the fodder desk, on the capacity
desk beside the horses, and on the maker. The maker's reservation is on at ψ 0.25. The county is
FUNDED's chain8, at C2g, 52 ticks a year, ρ 0, J_b 1 tick, ex-post assignment.

**The frame**, registered at L0.8 (`452c0e7`) before any plant code, in `docs/probe/loops/`:

| file | sha256 | what it fixes |
|---|---|---|
| `LOOP-SPEC.md` | `e16e0be2…fb89ad` | §2 the economy and roles, §3 the composition with M3, §4 instances, §5 nestings |
| `LOOP-SPEC-A1.md` | `8815d865…fa4f2b` | §A1.1 the genesis carry; §A1.5 predictions E0–E11 (replaces §8) |
| `LOOP-SPEC-A2.md` | `95de4411…c6b8a9` | this step's amendment: LN5 waits for O51, as LF4 does |
| `FUNDED.md`, `FUNDED-A1.md` | `ab86a2e4…46cc`, `45a23bf7…9eb5` | the county, chain8 |
| `instances.json` | `cb722dcb…52b2` | the tape params, every point, the plant readouts |
| `../results/loops/registration.md` | `f65f2a2a…db58` | the registration, quoting A1 §A1.5 |

**The mirror** is `D:/rustyecon-p2l/fix-report/loop-carry/model/` (copied read-only to
`D:/rustyecon-p2b/rules/mirror/`): `lm_carry.py` (`f94c33e1…`), written by `make_lm_carry.py`
(`e691dfbe…`) from the registered `lm_mirror.py` (`8ae28374…`, also in
`D:/rustyecon-p2l/loop-mirror/model/`); the scoring `lm_run.py` (`09455832…`) and the instances
`lm_inst.py` (`7888b1c9…`). Each sha256 was checked against the registration on 2026-09-30. A
reference like `lm_mirror:305` is a line of `lm_mirror.py`; `lm_run:167` one of `lm_run.py`.

This file says:

- what changes, and what does not (§1);
- the economies on the engine (§2);
- the plant every planted desk shares, line by line against the mirror (§3);
- the three planted roles (§4) and the genesis carry (§5);
- the load checks (§6);
- the instances, the dials and the generator (§7);
- the harness's observables and new readouts (§8);
- how E0–E11 are run and scored (§9);
- every test, with what it guards (§10);
- where the build's arithmetic parts from the mirror's (§11);
- decisions 286–296 and open items O69–O72 (§12);
- the files and the order of work (§13);
- the build as built (§14, P2.2b.1);
- the harness as built, and E0–E2 (§15, P2.2b.2).

## 1. What changes, and what does not

| crate | change |
|---|---|
| core | **An untraded good.** A good marked `untraded: true` is `Indefinite`, has no price rate, no market, no genesis price and no book slot that is ever written. `World::markets()` leaves it out, so nothing that walks the markets (clearing, the price rule, the report, certify's kick set, the GUI) sees it. The resolver says whether a good has a market, for the roles' checks (§6). |
| markets | Admission refuses an order on an untraded good (`NoMarket`), as on a currency. |
| engine | Nothing. The tick walks `World::markets()`. |
| agents | An optional `plant` field on `TypeDesk`, `Maker` and `CapacityDesk`; three planted roles; three `ActorState` variants appended after `Owner`; the load checks of §6. |
| probe | `probe::horses` extended: rule-B instances and the flow control, plants in the setup and the generator, the harness's new readouts (§8), the run grammar's plant stocks. |
| gui | `run/extract.rs` and the inspector read the planted states' fields. |
| certify | No code: its kick set walks `World::markets()`. One test (§10). |
| tapes | `tapes/loops-{lb1,lb2,lb3,lw1,lw2,lw3}.ron`, each its generator's output. |
| docs | ENGINE.md and TAPE.md amended for the untraded good and the plant; this file "as built". |

**What must not move (R1).**
- Every committed tape keeps its canonical text, `tape_hash`, `world_id` and per-tick hash
  stream. The gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867` and demo-gb
  `0xfad880fe08d06645` stay. So do the probe's pin, the markets pins, the horses tapes'
  generator test and `world_id`s, and the 2,000-tick streams of the six horses and seven markets
  tapes (`D:/rustyecon-p2l/l010/tapes-bit.sh` against `D:/rustyecon-p2g/report/hashes/`).
- How:
  - `untraded` is left out of a good's raw form when false. The resolved `GoodDef` needs no new
    field: a good that is not a currency and has no price rate is untraded.
  - `plant` is left out of the raw and the resolved specs when absent
    (`#[serde(default, skip_serializing_if = "Option::is_none")]`), as the maker's `reserve` is.
    Resolved kinds are hashed into `world_id` by bincode (L0.5).
  - A world with no untraded good has the markets it had.
  - The planted states are new variants, so no old encoding moves.
- The tape schema stays 1.

## 2. The economies on the engine

One node, `home`, quotes in `coin`.

### 2.1 Rule B (B, and B-cut): the goods

| good | life | made by | market |
|---|---|---|---|
| `labour` | Instant | `workers`, endowed in decide | `rate.labour` |
| `land` | Instant | `provider`, endowed in decide | `rate.land` |
| `fodder` | `life.one_tick` | `desk.fodder` (TypeDesk, planted) | `rate.fodder` |
| `horse` | Indefinite | `desk.maker` (Maker, planted, reservation on) | `rate.horse` |
| `traction` (horse-days) | `life.one_tick` | `desk.capacity` (CapacityDesk, planted) | `rate.traction` |
| `good` | `life.one_tick` | `desk.good` (P2.0's GoodDesk, ex post) | `rate.good` |
| `plant.capacity`, `plant.maker`, `plant.fodder` | Indefinite, `untraded` | each desk, in produce | none |

Per unit made, at every tick length (decision 259): fodder takes 2 horse-days, 8 labour and 1·b
land (B-cut: no horse-days); a head takes 8 fodder, 20 labour and 3·b pasture; a horse-day takes
0.0176 fodder and 0.1 labour; κ is 250 horse-days a head a year. b is the land factor, 1 at the
registered point.

### 2.2 The flow control (LW0–LW3)

The stocks layer off (decision 257; LOOP-SPEC §2.7): no horse, no maker, no capacity desk. A
second TypeDesk, `desk.traction`, makes horse-days from 0.0176 + 8·(δ/κ) fodder,
0.1 + 20·(δ/κ) labour and (3·b)·(δ/κ) land, with δ and κ per tick at the tape's tick length.
At δ 8% and 52 a year that is 0.020266073400477775, 0.10666518350119444 and
0.0009997775251791654 (FUNDED §4.2). It carries `plant.traction`; the fodder desk carries
`plant.fodder`.

### 2.3 The actors

| actor | kind | class | spec |
|---|---|---|---|
| `desk.good` | Desk | `good_desks` | `GoodDesk`, `assign: ExPost` |
| `desk.capacity` | Desk | `capacity_desks` | `CapacityDesk`, `order: Target`, plant (rule B) |
| `desk.maker` | Desk | `maker_desks` | `Maker`, cover 4 weeks, `reserve.maker`, plant (rule B) |
| `desk.traction` | Desk | `traction_desks` | `TypeDesk` making `traction`, plant (flow control) |
| `desk.fodder` | Desk | `fodder_desks` | `TypeDesk` making `fodder`, plant |
| `provider` | Pop | `owners` | `BasketProvider`, basket [(good, 1), (land, h)] |
| `workers` | Pop | `workers` | `BasketWorkers`, the same basket |

### 2.4 The tick

ENGINE §7.2's, as P2.2a's (HORSES-RULES §2). Fodder, horse-days and the good are made at t, sold
and used at t + 1, and die at 5a of t + 1. A genesis lot of them lives one tick longer. A horse
ordered at t is built at t, sells at t + 1 and serves from t + 2. A plant unit ordered at t is
bought as bundles at t, built in produce at t after the output, and serves from t + 1 (J 2,
M3's timing). Horses and plants wear by `Depreciation` burns at 5b.

## 3. The plant: what every planted desk shares

LOOP-SPEC §2.2, and `lm_mirror:117–186`. Notation: c the cost of one bundle at posted prices,
summed in the kind's own order; C the coin the desk held when decide ran; P the plant it held
then; z the bundles it plans to run; I the plant units it orders. `sign(x)` is x where
positive, else 0: the sign of an order, never a cap (R3). Every power is `num::pow` (libm), never
`f64::powf`, so both machines make the same bits.

### 3.1 The good

- One untraded, `Indefinite` good per planted desk, keyed `plant.<desk>`. No other actor's spec
  names it (§6). It has no market and no price.
- The desk mints it in produce (`Production`), after its output. It burns it at upkeep
  (`Depreciation`). Nothing else moves it.

### 3.2 The field and its params

```ron
plant: Some((good: "plant.fodder", theta: "plant.fodder.theta", delta: "plant.fodder.delta",
             size: "plant.fodder.size", adjust: "adjust.plant.fodder", order: Target, target: Bundles)),
```

| field | unit, method | what |
|---|---|---|
| `good` | good key | the plant good (§3.1) |
| `theta` | Dimensionless, value | θ, in (0, 1] |
| `delta` | FractionPerYear, `Clock::fraction` | the plant's wear a year; per tick u |
| `size` | Dimensionless, value | s, bundles a plant unit; the generator writes s1 |
| `adjust` | RatePerYear, `Clock::share` | s_Kp, 2·δ_p a year; per tick s_p |
| `order` | `Target` or `Gap` | M3's rule, or LN6's whole gap each tick |
| `target` | `Herd` or `Bundles` | the capacity desk's κ_p·κ·K\*, or kr·z (LF6; the only form on the other two kinds) |

Every param is live and read at use, so a dated `SetParam` retargets the plant. At θ 0.8, δ_p
10% a year and 52 a year: u = 0.0020241124785003953, s1 = 40.47205917167808,
s_p = 0.003838766869966628.

### 3.3 The cost ratios (`ratios`, `lm_mirror:134–147`)

- P_K = s·c.
- Where θ < 1 and u > 0:
  - kr = ((1 − θ)·c)/((θ·u)·P_K), the plant units per bundle at the cheapest plant;
  - c_full = c + (u·P_K)·kr;
  - ζ = kr^(−(1−θ)) and κ_p = kr^θ.
- Otherwise (θ = 1, or u = 0) the formulas are not evaluated: kr = 0, c_full = c, ζ = 1,
  κ_p = 0 (`lm_mirror:138–140`).

At s1, kr = θ^(−1/(1−θ)), ζ = θ, κ_p = θ^(−θ/(1−θ)) (2.44140625 at θ 0.8) and c_full·ζ = c.

### 3.4 The plan and the order

- **The plan** is the cash rule at the full cost: z = outlay/c_full. On the capacity desk z is
  M3's use at the running cost (§4.2).
- **The target** K\*_p is kr·z (`Bundles`), or on the capacity desk (κ_p·κ)·K\* (`Herd`,
  `lm_mirror:330`).
- **The order** (`p_order`, `lm_mirror:159–166`):
  - `Target`: I = sign((u·K\*_p) + (s_p·(K\*_p − P)));
  - `Gap`: I = sign((u·P) + (K\*_p − P)).
- **The cap.** I is at most sign(C − c·z)/P_K, the coin left after the running bundle
  (`lm_mirror:333, 409, 433`). On the capacity desk the horse order then takes what is left
  (§4.2).
- **What it reads (R13):** its own coin, holdings, plant and state; the posted prices; θ, δ_p,
  s, s_Kp and the kind's own params. No volume, fill, other actor or oracle.

### 3.5 Buy lines and budgets

- **One line per bundle good**, a_g·(z + s·I), with z + s·I formed once. Labour and land alike.
  Its want is p_g·qty, as the kinds form it now. (The mirror keeps a running line and a plant
  line; the engine's one line fills at the same ratio, so the two agree to rounding, §11.)
- **Budgets** are the budget chain's (MARKETS-RULES §3), in admission's good order. Its total:
  - on the TypeDesk and the maker, outlay + P_K·I. At I = 0 that is the outlay exactly, the
    plant-free total. The running lines cost about c·z = outlay·c/c_full, so the total leaves
    room of outlay·(1 − c/c_full) beside the plant's P_K·I;
  - on the capacity desk, its coin, as now. The lines sum to about O·z + P_K·I + p_K·m ≤ C.
  - Either way no budget binds beyond rounding, which the mirror (no budgets) needs.

### 3.6 Produce: bundles, split, output, burn and mint

1. **Bundles held.** B = min over the bundle's goods whose coefficient is not zero of
   `max_scale`(held_g, a_g): the kind's own Leontief, read over every unit held, carried units
   included (§5). The maker's list includes (κ·serving, a), skipped at a = 0.
2. **The split**, by the plan recorded at decide (z, I):
   - I > 0: z_got = min((B·z)/(z + s·I), B) and I_got = (B·I)/(z + s·I);
   - I = 0: z_got = B and I_got = 0, with no division.
3. **Output.** θ < 1: y = P^(1−θ)·z_got^θ, and 0 at z_got = 0 (`p_make`, `lm_mirror:169–171`).
   θ = 1: y = z_got, with no power. On the capacity desk y is also at most κ·H.
4. **Bundles used.** On the TypeDesk and the maker, used = z_got. On the capacity desk, at
   θ < 1, used = min((y/P^(1−θ))^(1/θ), z_got), and 0 at y = 0 (`p_inverse`,
   `lm_mirror:174–181`); at θ = 1, used = y.
5. **The build.** I′ = min(I_got, `max_scale`(`max_remainder`(B, used), s)). So
   fl(used + fl(s·I′)) ≤ B: the running and the build bundles fit what is held. I′ differs from
   I_got by an ulp at most.
6. **One `Production` burn per bundle good**, a_g·T with T = used + s·I′. Since T ≤ B, every
   burn fits. At I′ = 0 it is a_g·used, the plant-free burn.
7. **Mints** (`Production`): y of the output, then I′ of the plant good, each only if positive.
   The build is minted after the output, so it serves from the next tick.

On the capacity desk the bundles not used are one-tick goods: they die at 5a, or carry at tick 0
(§5).

### 3.7 Upkeep

- The plant wears by min(u·P, held), P the plant recorded at decide (`Depreciation`; no burn at
  0). Units built this tick do not wear this tick. So P′ = P + I′ − u·P (the mirror writes
  (1 − u)·P + I′, `lm_mirror:630, 641, 656`).
- The capacity desk's horses and the maker's serving stock wear as in P2.2a.

### 3.8 State: three appended variants

```rust
pub struct PlantState { held: f64, target: f64, order: f64, run: f64, built: f64 }
ActorState::PlantedType(PlantedTypeState { desk: MachDeskState, plant: PlantState })
ActorState::PlantedMaker(PlantedMakerState { desk: MakerState, plant: PlantState })
ActorState::PlantedCapacity(PlantedCapacityState { desk: CapacityState, plant: PlantState })
```

- `held` is P at decide, which upkeep wears and the harness reads. `target` is K\*_p, `order` is
  I, `run` is z (the maker's and the TypeDesk's q; the capacity desk's z_plan, the same number as
  `desk.run`). `built` is I′ at the last produce. All are 0 before the first tick.
- The variants are appended after `Owner`, so the eight old encodings stay. A desk without a
  plant keeps its old variant. `check` requires every value finite with a clear sign bit.
- A desk's genesis state is the planted variant exactly when its spec has a plant.

### 3.9 Off, θ = 1, and u = 0

- **No plant.** The kind's own code runs, unchanged: its role, its state, its resolved spec.
- **θ = 1.** kr = 0, c_full = c, κ_p = 0, so no order and z_got = B. y = z_got, the budgets'
  total is the outlay, and each burn is the plant-free burn. The genesis plant is κ_p·X = 0 and
  the genesis coins are the plant-free ones. So a planted tape at θ 1 is the plant-free tape
  value for value (E1). Its state variants and the empty plant goods differ; nothing measured
  does.
- **u = 0** (δ_p 0). kr = 0, c_full = c, κ_p = 0; the order is 0 and nothing wears. Output stays
  P^(1−θ)·z^θ with P fixed: fixed-Q drs (E1; the mirror's plant rule "none"). The rest ratio
  at u = 0 gives κ_p = 0 and so an empty plant, so E1's run starts from the design's plants,
  Q = κ_p·X at the registered δ_p, and writes δ_p 0 on the tape (`--fixed-plants`, §8.4).

The plant goods sort between `land` and `traction` by key, which moves the `GoodId`s of the goods
after them. The traded goods keep their order among themselves, and so every sum, budget chain
and clearing keeps its order. The θ = 1 test (§10) holds the claim.

## 4. The three planted roles

### 4.1 The TypeDesk: the fodder desk, and the flow control's horse-day desk

`lm_mirror:417–438` and `592–610`; the flow control's `340–363` and `550–560`.

- **Decide.**
  - c = ((0.0 + Σ a_g·p_g) + λ·w) + b·r, the kind's order. On the fodder desk a_traction is 2,
    so c = ((0.0 + 2·p_h) + 8·w) + b·r. At B-cut it is 0: the desk still lists the horse-days,
    buys none, and a zero coefficient binds nothing (FUNDED's form, §7.2).
  - The margin is the kind's: cost c, markup p·(1 − a)/c, worth. So is the outlay (the cash
    rule; C2g's tilt is 0).
  - §3.3's ratios at c; q = outlay/c_full.
  - keep = min(a·q, held) (a is 0, §6); it offers the rest of what it holds.
  - K\*_p = kr·q; I by `order`, at most sign(C − c·q)/P_K.
  - It buys a_g·(q + s·I) of each input, λ·(q + s·I) labour and b·(q + s·I) land; budgets from
    outlay + P_K·I.
  - It records `desk` as now and `plant` {held P, target K\*_p, order I, run q}.
- **Produce.** §3.6 with used = z_got. It mints y of its output (fodder, or horse-days) and I′
  plant units.
- **Upkeep.** The plant's wear.
- **At rest (s1).** θ·y bundles run and (1 − θ)·y rebuild the plant: one bundle a unit. The
  plant is κ_p·y (0.19058574 at LB1) and the price is c (LOOP-SPEC §2.3).

### 4.2 The capacity desk: M3 wet, with the plant beside the horses

`lm_mirror:305–339` and `527–549`. The plant's bundle is the running recipe (decision 261).

- **Decide.**
  - O = (0.0 + Σ run_g·p_g) + run_lab·w; full = O + δ·p_K/κ. The margin is the kind's (cost
    full), and so is the outlay.
  - H is the horses held, C the coin, P the plant.
  - §3.3's ratios at O give P_K, kr, ζ, κ_p. Where θ < 1 and u > 0,
    O_f = (ζ·O) + ((u·P_K)·κ_p); otherwise O_f = O (`lm_mirror:311`). O_f is O at s1.
  - zcap: at θ < 1, the bundles that make κ·H on P, (κ·H/P^(1−θ))^(1/θ); 0 where κ·H is 0,
    else +∞ where P is 0 (`p_inverse`). At θ = 1, κ·H.
  - z = min(zcap, outlay/O) where O > 0, else zcap, as P2.2a's. §6 keeps O above 0 at genesis.
  - K\* = outlay/((κ·O_f) + (δ·p_K)). The horse order is M3's: want = sign(base + s_K·(K\* − H)),
    base δ·K\* (`Target`) or δ·H (`Held`).
  - K\*_p = (κ_p·κ)·K\* (`Herd`, decision 262) or kr·z (`Bundles`, LF6).
  - I by `order`, at most sign(C − O·z)/P_K. Then the horses: m = min(want,
    sign((C − O·z) − P_K·I)/p_K) (`lm_mirror:326–336`). At I = 0 that is P2.2a's cap.
  - It offers every horse-day it holds. It buys run_g·(z + s·I) of each running good,
    run_lab·(z + s·I) labour and m heads; budgets from C.
  - It records `desk` {scale, held H, target K\*, order m, run z} and `plant` {held P, target
    K\*_p, order I, run z}.
- **Produce.** B over the running goods and labour held. The split (§3.6). y = min(κ·H,
  P^(1−θ)·z_got^θ), from the horses held at decide. used = min(p_inverse(P, y), z_got). The
  build I′. One burn per running good and labour of a·(used + s·I′). It mints y horse-days and
  I′ plant units.
- **Upkeep.** min(δ·H, held) heads and min(u·P, held) plant, both `Depreciation`.
- **At rest (s1).** z = θ·κ·H; the plant is κ_p·κ·H (9.4041769 at LB1); the hour price is
  O + δ·p_K/κ, the oracle's (LOOP-SPEC §2.4).

### 4.3 The maker: M2 from bought inputs, with a plant and the reservation

`lm_mirror:374–416` and `563–591`.

- **Decide.**
  - c_m is P2.2a's: the bought goods, each a·run_g + build_g in list order, then labour, then
    land. With a = 0 (§6), c_m = ((0.0 + 8·p_f) + 20·w) + 3·b·r.
  - The margin is the kind's: cost c_m, markup p_K·(1 − δ·a/κ)/c_m. So the reservation's μ reads
    c_m, not c_full, as the mirror's does (`lm_mirror:284–288`; decision 295).
  - The outlay is the cash rule's. §3.3's ratios at c_m; q = outlay/c_full.
  - Its own stock's target a·q/κ is 0, so it keeps nothing.
  - The offer: none where ψ > 0 and μ < ψ (L0.7's `withholds`); otherwise
    sign(finished − keep − ((cover·q)·c_full)/p_K) (`lm_mirror:390`). At θ = 1 that is P2.2a's
    b_K·q·c_m/p_K.
  - K\*_p = kr·q; I by `order`, at most sign(C − c_m·q)/P_K.
  - It buys each good, labour and land on q + s·I; budgets from outlay + P_K·I.
  - It records `desk` as now and `plant` {held P, target K\*_p, order I, run q}.
- **Produce.** B over (κ·serving, a) (skipped at a = 0), the goods, labour and land held. The
  split. y = P^(1−θ)·z_got^θ heads, which join its finished stock; used = z_got; the build I′;
  one burn per good; mints of y heads and I′ plant units.
- **Upkeep.** Its serving stock's wear (0 under rule B) and the plant's.
- **At rest.** μ = 1 > ψ, so the reservation is inactive (IDLE-SPEC §7).

## 5. The genesis carry (A1 §A1.1; decision 273)

The engine needs no carry code. Its lives give it.

- **Sellers.** A genesis lot of a one-tick good lives one tick longer, and each seller offers
  every unit it holds. So the good desk's good, the fodder desk's fodder and the capacity desk's
  horse-days (the flow control's horse-day desk's) that do not sell at tick 0 are offered again
  at tick 1, beside tick 0's output.
- **Buyers.** What a desk buys at tick 0 comes from genesis lots. What it does not burn is still
  held at tick 1: the good desk's horse-days; the capacity desk's fodder; the maker's fodder; the
  fodder desk's horse-days; the flow control's horse-day desk's fodder. Labour and land are
  `Instant` and carry nothing.
- **Planted desks (decision 273).** B reads every unit held (§3.6), so a carried good enters the
  bundles at tick 1, as far as that tick's labour allows. The split uses tick 1's plan; with no
  plant order all of B runs. This is `lm_carry`'s carry branch (`make_lm_carry.py`, its `CARRY`
  blocks for the capacity desk, the flow control's horse-day desk, the maker and the fodder
  desk).
- **What carries.** At tick 0 a planted desk holds what it bought less a_g·(used + s·I′). That is
  `lm_carry`'s C_Fc, C_Fm and C_Hf: the fodder or horse-days received less the coefficient times
  (used + s·Ig).

At rest the carry is rounding. Displaced it is not: after w × 0.5 or r × 0.5 it reaches 0.60 of
the capacity desk's and the maker's fodder flow, and 0.96 of the fodder desk's horse-day flow
(A1 §A1.1). E0 tests it at tick 1; `loops_carry_meets_the_mirror_at_tick_one` holds it in the
gate (§10).

## 6. Load checks

Each is a `LoadError` with its path, as SG7's are (HORSES-RULES §3).

**Core, at the tape's goods and genesis** (`goods[<key>]`, `genesis.prices[…]`):
- `untraded: true` requires `Indefinite`, `price_rate: None` and a good that is no node's
  currency.
- An untraded good has no genesis price at any node; one given is refused.
- A good that is not a currency, has no price rate and is not marked untraded is refused
  (`NoPriceRate`), as now. So a typo is still caught.

**Agents, at resolve** (`actors[<key>].spec.plant.<field>`):
- `good` names an untraded good. Every role good (output, labour, land, inputs, running, build,
  stock, hours, basket items) must have a market: `traded()` refuses an untraded good, and so do
  a scripted actor's lines.
- `theta`, `delta`, `size` and `adjust` have the units and methods of §3.2; another unit is a
  unit mismatch at the same path.
- `plant` on any kind but the three is an unknown field (`deny_unknown_fields`).

**Agents, in `Cast::new`**, among each actor's own checks, so M6 and M5 still run last
(HORSES-RULES §8 item 5):
- θ at genesis is in (0, 1].
- The plant wears less than all of it a tick: u < 1 at genesis. δ_p 0 is allowed (§3.9).
- s at genesis is positive.
- `target: Herd` is on a capacity desk only.
- A planted desk's scale is the cash rule (`Scale::Cash`). The step rule sizes its own state;
  the mirror ran no plant on it.
- A planted TypeDesk's own input a is 0 at genesis (LOOP-SPEC runs none).
- A planted maker is on the stock path, with `own_hours` 0 at genesis (LOOP-SPEC §2.5).
- The bundle has a coefficient above 0 at genesis. Otherwise c = 0 and kr is 0/0.
- A plant good belongs to one desk: no two `plant` fields name it.
- The reservation's checks stay (the flow path, and a world under `Hold`).
- M6 and M5 need nothing new: no role buys or sells a plant good.

**At run time** (markets, core): an order on an untraded good, or a `ScalePrice` of one, is
`NoMarket`; a written untraded book slot fails validation, as a currency's does.

## 7. The instances, the dials and the generator

### 7.1 The instances (`lm_inst.py`; LOOP-SPEC §4)

Every instance is chain8 under rule B: C2g, 52 ticks a year, ρ 0, J_b 1, ex post. The design is
plants on the fodder desk (f), the capacity desk (c; the flow control's horse-day desk) and the
maker (m), θ 0.8, δ_p 10% a year, s1, s_Kp 2δ_p, `order: Target`, the capacity target `Herd`,
and ψ 0.25. A row changes only what it names.

| id | horse δ | changed | config | role |
|---|---|---|---|---|
| `lb1` | 8% | — | wet | verdict |
| `lb2` | 10% | — | wet | verdict |
| `lb3` | 4% | — | wet | verdict |
| `lf1`, `lf7`, `lf8` | 8%, 10%, 4% | θ 0.7 | wet | family |
| `lf2` | 8% | δ_p 4% a year (s1 104.39255449789697) | wet | family |
| `lf3` | 8% | no reservation | wet | sensitivity |
| `lf4` | 8% | storable fodder, 13 weeks | — | **not built** (A2; O51) |
| `lf5` | 8% | `rate.fodder` 1.3 (c2g13) | wet | family |
| `lf6` | 8% | capacity target `Bundles` | wet | negative control |
| `lc1` | 8% | loop cut (B-cut) | wet | the loop cut |
| `lw1`, `lw2`, `lw3` | 8%, 10%, 4% | plants f and c only | flow control | O14's reference |
| `lw0` | 8% | no plants | flow control | negative control |
| `ln1` | 8% | no plants | wet | negative control |
| `ln2` | 8% | plants f, c | wet | placement |
| `ln3` | 8% | plant f | wet | placement |
| `ln4` | 8% | plants c, m | wet | placement |
| `ln5` | 8% | storable fodder, 4 weeks | — | **not built** (A2; O51) |
| `ln6` | 8% | `order: Gap` | wet | negative control |
| `ln7` | 8% | loop cut, no plants | wet | negative control |
| `ln8` | 8% | loop cut, no plants, c2g13 | wet | negative control |
| `ln9` | 8% | no plants, c2g13 | wet | negative control |

Each instance's oracle point is FUNDED's at its horse δ and land factor. At s1 the plants leave
it unchanged.

### 7.2 Keys and params

FUNDED §4.1's keys (`instances.json`, `tape_params`), with FUNDED's bases:

- **The county:** `inst.workers` 416 and `inst.land` 520 (FlowPerYear), `inst.chi_max` 0.05,
  `inst.eta` 1, `inst.g0` 0.2, `inst.g1` 0.8, `inst.k` 1, `inst.good.weight` 1,
  `inst.space.weight` 1.
- **Rule B's chain:** `inst.fodder.own` 0, `inst.fodder.traction` 2 (0 at B-cut, FUNDED's
  "B's with `inst.fodder.traction` 0"), `inst.fodder.labour` 8,
  `inst.fodder.land` 1·b, `inst.horse.run.fodder` 0.0176, `inst.horse.run.labour` 0.1,
  `inst.horse.kappa` 250 (FlowPerYear), `inst.horse.delta` δ (FractionPerYear),
  `inst.horse.own_hours` 0, `inst.horse.fodder` 8, `inst.horse.labour` 20, `inst.horse.land`
  3·b.
- **The flow control:** the county, `inst.fodder.*`, and `inst.traction.fodder`,
  `inst.traction.labour`, `inst.traction.land` of §2.2, computed as fo + fI·(δ/κ),
  lo + lI·(δ/κ) and (bI·b)·(δ/κ) with δ/κ formed first (`flow_recipe`, `lm_mirror:68–71`).
- **The plants**, per planted desk d in {capacity, maker, fodder, traction}: `plant.<d>.theta`,
  `plant.<d>.delta`, `plant.<d>.size` (s1 at the tape's tick length, from the oracle's
  `s1_size(θ, u)` with u = `Clock::fraction`(δ_p); 1 at θ = 1, as `lm_mirror:127–128`) and
  `adjust.plant.<d>` (2·δ_p a year). Basis `Assumed("CAPACITY 2026-09-27; LOOP-SPEC 2026-09-29,
  mirror scan")`.
- **The land factor** b moves `inst.fodder.land` and `inst.horse.land` together (decision 256),
  or `inst.fodder.land` and `inst.traction.land` in the flow control. `b*F` sets it.

The maker's spec: `output: "horse"`, `running` (the horse-day's recipe, keyed by the horse),
`build: (goods: [(good: "fodder", coef: "inst.horse.fodder")], labour: "inst.horse.labour",
land: "inst.horse.land")`, `own_hours`, `kappa`, `delta`, `adjust: "adjust.invest.maker"`,
`cover: Some("cover.maker")`, `reserve: Some("reserve.maker")`, `own: 0.0`, and its plant. Its
bought fodder is then 0·0.0176 + 8 a head.

### 7.3 The dials

C2g, as P2.2a registered it (HORSES-RULES §4): `rate.labour` 5.2, `rate.land` 0.1625,
`rate.fodder` 5.2 (1.3 under `c2g13`), `rate.horse` and `rate.traction` 5.2, `rate.good` 2.6,
`adjust.technique.good` 2.6, `buffer.desk.<d>.cash` 5.2 and `tilt.desk.<d>` 0 for every desk,
`spend.*` 13, `price.ema_tc` 0.5, `adjust.invest.capacity` 2δ a year, `adjust.invest.maker` 0,
`cover.maker` 4/52 a year. And **`reserve.maker` 0.25, on by default** in every rule-B instance
(decisions 253, 265). `--reserve none` removes it (LF3, and the glut runs at ψ 0 of E9).

### 7.4 Genesis (`genesis`, `lm_mirror:199–258`)

From the oracle's point at the coefficients the tape registers from tick 0. A cost shock at
genesis keeps genesis at the registered point (HORSES-RULES §5).
- **Prices** relative to r = 1: labour v, land 1, fodder p_f, a head p_K, a horse-day p_h, the
  good p. The good desk's genesis share is 1 − x\*.
- **Stocks.** The good desk holds Y of the good. The capacity desk holds H = X/κ heads (X every
  horse-day, the tasks' and the fodder desk's) and κ·H horse-days. The maker holds
  q_b + ((cover·q_b)·c_m)/p_K heads, with `own` 0. The fodder desk holds q_f. The flow control's
  horse-day desk holds X horse-days.
- **Plants.** plant.<d> = κ_p·X_d, with κ_p = kr^θ and kr = (1 − θ)/((θ·u)·s) from the tape's own
  params: the price-free rest ratio, which is the harness's target (`rest_ratios`,
  `lm_mirror:150–156`; `lm_run:46–50`). X_d is κ·H on the capacity desk, q_b on the maker, q_f on
  the fodder desk, X on the horse-day desk.
- **Coins**, each the actor's stationary balance, summed in the order its rule sums:
  - the good desk p·Y/share(turnover), and the capacity desk (p_h·κ·H)/share(turnover), as P2.2a;
  - a planted TypeDesk or maker (c_full·ζ)·X/share(turnover), with c, c_full and ζ at genesis
    prices (`lm_mirror:217–233`); unplanted, P2.2a's c·X/share(turnover);
  - the provider and the workers as P2.2a's.
- At θ = 1 every plant is 0 and every coin is the plant-free one (c_full = c, ζ = 1).

### 7.5 The oracle

`ChainEconomy` (unit 1g) on FUNDED's chain, solved in the harness, outside any `Sim` (R13):
material FODDER (HORSE_DAYS 2, or none at B-cut; labour 8; land b); machine HORSE (build FODDER
8, labour 20, land 3·b; hours HORSE_DAYS, κ per tick; operating FODDER 0.0176, labour 0.1; δ per
tick; build lag 2, decision 232). At ρ 0, J 2 is J 1 bit for bit (FUNDED §4.1). The FUNDED
scratch crate builds the same chain (`D:/rustyecon-p2l/funded/oracle/src/main.rs`, `chain`).

For rule B the harness's point reads: `capacity` = every horse-day over κ; `serving` 0; `sold` =
`made`; `task_hours`, the good desk's Y·J(x\*); the fodder desk's horse-days 2·q_f are the rest.
The flow control's point is B's (within 9.3e-16, FUNDED §4.2), and its horse-day desk's cost
equals p_h to rounding. No new solve is needed (decision 184).

### 7.6 The committed tapes

`tapes/loops-<id>.ron` for LB1–LB3 and LW1–LW3, each `tape_ron(&Setup::registered(id, 52))`,
written by `horses-tape --inst <id>`. The other instances are generated at run time, as P2.2a's
families were.

## 8. The harness

### 8.1 Observables and targets (`lm_run:21–66`)

- **Wet (20 at LB1):** v; `pi.fodder`, `pi.horse`, `pi.traction`, `pi.good`; `s.good`;
  `vol.labour`, `vol.land`, `vol.fodder`, `vol.horse`, `vol.traction`, `vol.good`; `y.good`,
  `y.traction`, `y.horse`, `y.fodder`; `heads.capacity`; then `plant.<d>` for each planted desk
  with θ < 1, in desk order (capacity, maker, fodder).
  - `heads.maker` is dropped. Rule B's maker holds no serving stock (a = 0), so its target is 0
    ("HORSES' 18 less the maker's own heads", `lm_run:7`).
- **Flow control (15):** v; `pi.fodder`, `pi.traction`, `pi.good`; `s.good`; `vol.labour`,
  `vol.land`, `vol.fodder`, `vol.traction`, `vol.good`; `y.good`, `y.traction`, `y.fodder`;
  `plant.traction`, `plant.fodder`.
- **A plant observable** is the planted state's `plant.held` after the tick: the plant held at
  decide, the mirror's st[key] at the tick's start (`lm_run:63–65`).
- **Targets** come from the oracle at the coefficients in force (`lm_run:38–51`). `vol.horse` is
  q_b (a = 0). `vol.traction` and `y.traction` are X, every horse-day. `heads.capacity` is X/κ.
  `vol.fodder` and `y.fodder` are q_f. `plant.<d>` is κ_p·X_d, as at genesis.

### 8.2 Dead, idle and the classes

P2.2a's (HORSES-RULES §5):
- A tick is **dead** when labour, land, fodder, the horse-days or the good does not trade or
  clears below half its target. That union is the classifier's input, as in the mirror
  (`lm_run:166–175`).
- The horse market is **idle**, not dead.
- The classes are the markets probe's streaming classifier (PROBE-SPEC §4.5). A stock or coin
  displacement's D̂₀ is the largest D̂ of its first year.

### 8.3 New readouts

Each is per run, over the scored ticks unless it says otherwise. Each is read outside the Sim.

- **Dead ticks by market** (decision 274). Per market, the scored ticks on which it does not trade
  or clears below half its target (`lm_run:167–170`). Fodder, horse-days, the good and land are
  scored; labour is reported. The union is P2.2a's `dead`.
- **Labour supply's bound** (decision 277). On every tick of the run, the ticks before a dated
  shock included (as `lmc.py` counts): z = `num::ln1p`(w/P_s)/χ_max from the tick's posted
  prices, P_s = (0.0 + 1·p) + h·r. `bound_ticks` counts z ≥ 1; `bound_peak` is the largest z.
- **The horse-days by buyer.** The good desk's filled buys of `traction` over Y·J(x\*), and the
  fodder desk's over 2·q_f, from the report's rationing lines by class. Their lowest are the
  mirror's `minHt` and `minHf` (`lm_run:207–208`); E7 scores the tasks' (0.509 at LB1 after
  r × 2).
- **The horse price's highest** over its target, `pk_high` (the mirror's `pKmax`), beside
  P2.2a's `pk_low`.
- **Each plant's 5% readout:** the scored tick from which |P/P\* − 1| ≤ 0.05 holds to the end
  (`lm_run:196–203`), and its lowest and highest over target. The heads' readout is P2.2a's
  `in_5pct_from`, with the rule-B target X/κ.
- **The reservation's** are P2.2a's (HORSES-RULES §10): `markup` and `withheld` per tick,
  `withheld`, `switches` and `markup_low` per run, read through `maker_reservation`, which reads
  c_m, as the rule does.
- **The running cost, read from the tape.** P2.2a's harness forms a horse-day's running cost O,
  and so the CSV's `running_cost` and `full_cost`, the quasi-rent and the hour price over O, as
  rule A's one unit of fodder (`crates/probe/src/horses/harness.rs`, `row`). L0.7 moved the
  maker's readout to the tape's recipes but not this one. Under rule B it would read
  O = p_f, not (0.0 + 0.0176·p_f) + 0.1·w. The build reads O from the resolved running recipe
  (the capacity desk's, or the owner desk's under M1) and the params in force, summed in the
  rule's order. At every P2.2a instance that is (0.0 + 1·p_f) + 0·w = p_f, P2.2a's value bit for
  bit. These readouts are reported, not scored.

### 8.4 The run grammar and the families

P2.2a's (HORSES-RULES §5), with:
- **Rule B's stocks:** `stock.good`, `heads.capacity`, `hours.capacity`, `finished.maker`,
  `stock.fodder`, and `plant.<d>` for each planted desk. `own.maker` is dropped (0 under rule
  B). The flow control's: `stock.good`, `stock.traction`, `stock.fodder`, `plant.traction`,
  `plant.fodder`.
- **Coins:** `coin.<actor>` for every actor.
- **`b*F@genesis`, `b*F@dated`** set the land factor (§7.2).
- **Setup options:** `--inst` takes the loop ids; `--reserve PSI|none`; and, for E1's runs and
  exploration:
  - `--theta X` and `--plant-delta D` set every plant's θ or δ_p; s, s_Kp and genesis follow;
  - `--fixed-plants` writes δ_p 0 on every plant and keeps genesis at the registered design's
    plants, Q = κ_p·X (§3.9).
- **The families** are `battery`, `tier3s` and `stocks`, built from the instance's markets,
  stocks and actors. The counts are §9's E3 and `loops_batteries_are_registered`.

### 8.5 The outputs

P2.2a's columns keep their place and meaning (decision 292):
- **The CSV** appends, after `withheld`: for each planted desk in desk order `plant.<d>.held`,
  `.target`, `.order`, `.built`; then `supply_ratio` (z of §8.3), `hours_tasks` and
  `hours_fodder`.
- **`summary.tsv`** appends nine columns after `markup_low`, for every instance: `dead_by`
  (`market:count` for each market but the horse's, joined by `;`), `bound_ticks`, `bound_peak`,
  `hours_tasks_low`, `hours_fodder_low`, `pk_high`, `plants_in_5pct_from` (`desk:tick`, joined by
  `;`, or `-`), `plants_low` and `plants_high` (the same form). A P2.2a instance writes `-` or 0.
- **`stats.tsv`** gains the same readouts in long form (`dead.by_market`, `bound.*`,
  `plant.<d>.*`).

### 8.6 Run lengths

HORSES-RULES §6.7's rule (decision 267): L = max(20,000·tpy/52, 200·τ_max from the engine's own
`horses elasticity`, 3·T6 from the mirror's `lm_lin.json`), each rounded up to 1,000. The good's
τ of about 1,000 ticks sets the mirror's L:
- 211,000 at horse δ 8% (LB1, LF1–LF3, LF5, LF6, LW0, LW1, LN2, LN4, LN9);
- 221,000 with the loop cut (LC1, LN8);
- 202,000 at δ 10% (LB2, LF7, LW2) and 232,000 at δ 4% (LB3, LF8, LW3).

An instance whose mirror T6 is infinite (LN1, LN3, LN6, LN7) runs at LB1's engine L
(`lm_battery:70–74`). Tier 3 and 3S run again at 10·L at every instance but the LN ones
(`lm_battery:83–87`). At other tick lengths L scales as L·tpy/52, rounded up to 1,000
(`lm_extra:38`).

## 9. How E0–E11 are run and scored

The tolerances are A1 §A1.5's:
- every class exactly at LB1–LB3, the families and the flow controls;
- ticks and years within 10%, with three runs per instance allowed within 25%;
- lowest prices and troughs within 5% of the value given;
- dead ticks within 10% or 5 ticks, whichever is larger, market by market for fodder, horse-days,
  the good and land; labour's and the union's reported, not scored;
- ticks at labour's bound within 10% or 5 ticks; a run whose peak is within 5% of the bound may
  cross it or not.

The "registered" values are A1 §A1.4's tables.

| E | runs | read | scored against |
|---|---|---|---|
| **E0** | the trace diff (§13), before any scored run: `lb1` hold, `w*2`, `w*0.5`, `r*2`, `r*0.5`, `b*2@genesis`, `heads.capacity*2`, `heads.capacity*10`, `hours.capacity*2`; `lw1` hold, `r*0.5`, `b*2@genesis`; 2,000 ticks each from the engine's own genesis | every observable in log, every coin and plant, each tick | `lm_carry.tick` within 1e-12 in log. Allowed partings: the order's cancellation (HORSES-RULES §6.4) and the reservation's chatter after a glut within the mirror's own one-ulp spread (O55). A parting at tick 1 names the carry: amend before any scored run |
| **E1** | the gate's pins and tests; `tapes-bit.sh` on WSL and Windows; `loops_theta_one_is_the_plain_tape`; `loops_fixed_plant_is_drs`, and the trace diff at `lb1 --fixed-plants` hold and `w*2` against `lm_carry` at plant rule "none" with P = Q | hashes and streams; values | bit for bit (committed tapes); value for value (θ 1); fixed-Q drs within 1e-12 |
| **E2** | `loops_rest_point_is_the_oracles`; `horses run hold --ticks L` at LB1–LB3 | the fixed point; mode A's hold check | 1e-12 at every instance and target; mode A PASS below 1e-9 (mirror 6.7e-16, 1.0e-15, 4.5e-14) |
| **E3** | `horses family battery`, then Tier 3 and 3S again at 10·L, and `family stocks`, at LB1–LB3 | classes; years; lowest baskets; dead by market; lowest horse price, per tier | every run CONVERGED (20, 24, 25 (25), 28 (28), 28); A1 §A1.4's §7.1 table |
| **E4** | `horses slowest hold` and `horses slowest b*F@genesis` (F 1.1, 0.9, 2, 0.5) at LB1–LB3 and LW1; `horses kick` at the base | g a year, P2.2a's reading (§6.6; decision 293); every kick set's verdict | every set decays; base g within 0.03 a year of 0.847, 0.830, 0.901, 0.836; no g above 1 a tick at any target (a refutation) |
| **E5** | `b*2@genesis`, `b*0.5@genesis` at LB1–LB3, LW1–LW3 | trough (lowest cleared good over the new Y\*); years to tolerance; heads and each plant to 5%; lowest horse price | A1 §A1.4's §7.2 rows |
| **E6** | `heads.capacity*10` (stocks family) and `*2` (Tier 3S) at LB1–LB3, at L and 10·L | class; union dead; `pk_low`, `pk_high`; withheld ticks | CONVERGED with no dead tick; lowest 0.214–0.221, highest 2.02–3.76; withheld 303, 1,315; 221, 1,034; 662, 2,694 |
| **E7** | `r*2` at LB1; `r*2` at LN7 | dead by market; fodder's lowest (stats' `trough.cleared`); the tasks' horse-days lowest | LB1: horse-days ≤ 5 (mirror 0), fodder 175, good 11, land 12, fodder low 0.409, tasks low 0.509. LN7: ≥ 100 horse-day dead ticks (mirror 146), not CONVERGED |
| **E8** | the batteries of LW1–LW3 | as E3 and E5; O14's columns (A1 §A1.4's §7.6) | GO; §7.1, §7.2, §7.6 numbers; LB1's b × 2 trough 0.036 higher in log, 1.36 times the median years |
| **E9** | the batteries at L (LB1's L where the mirror's is infinite) of LN1–LN4, LN6–LN9, LF3, LF6, LW0; `heads.capacity*2`, `*10` at LB1–LB3 with `--reserve none`; LB1 at `--tpy 12`, Tiers 1–2 | classes by tier; the runaway tick | each keeps its verdict; converged counts within 2 runs of A1's; the named outcomes of A1 §A1.5 E9. **LN5 is not run** (A2) |
| **E10** | the batteries of LF1, LF2, LF5, LF7, LF8, LC1 | as E3 | GO with A1 §A1.4's numbers. LF4 is not built (A2) |
| **E11** | every battery run of LB1–LB3 and LW1–LW3 | `bound_ticks`, `bound_peak` | A1 §A1.4's "labour supply's bound" rows; none at genesis; each converges |

**What would refute the design** (A1 §A1.5): a class change at LB1–LB3; a runaway of the horse
price at LB1–LB3 with ψ 0.25; a converged run ending more than 1e-12 off its target's oracle
point; a θ = 1 tape differing from the plant-free tape; the engine's slowest mode above 1 at any
target.

## 10. Tests

Each fails with the change it guards undone; the build's mutants show it (as
`D:/rustyecon-p2l/fix-report/mutants/` did for L0.7).

**Core** (`crates/core`):
- `untraded_goods_load_without_a_market`. An untraded good loads with a genesis holding and no
  price. `World::markets()` leaves it out. Its book slot stays a currency's. Guards the new kind
  of good.
- `untraded_goods_are_checked`. Each refusal with its path: with a price rate; not
  `Indefinite`; a currency; given a genesis price. The existing
  `currencies_and_prices_rates_are_checked` still refuses a plain good with no price rate. Guards
  the kind's limits and the typo check.
- `untraded_flag_keeps_canonical_text`. A tape without the flag writes none in canonical form. A
  tape with it round-trips. The gate's `tape_hash` and `world_id` are unchanged. Guards R1.
- `untraded_slots_are_never_written`. A `ScalePrice` of an untraded good is `NoMarket`. A state
  whose untraded slot was written fails validation. Guards the book.

**Markets** (`crates/markets`):
- `admission_refuses_orders_on_an_untraded_good`. A buy and a sell of it are `NoMarket`, as on a
  currency. Guards "never traded".

**Certify** (`crates/certify`):
- `kick_skips_untraded_goods`. A planted tape's kick set has two runs per traded market and none
  for a plant. Guards E4: a kicked plant price would never decay.

**Agents** (`crates/agents/tests/plant.rs`):
- `plants_are_checked_at_load`. Each refusal of §6 with its path. `plant` on a GoodDesk is an
  unknown field. Canonical text without a plant has none; a planted spec round-trips; `sites`
  lists the four params with their methods. Guards §6.
- `old_states_keep_their_encoding`. The bincode bytes of one value of each of the eight old
  `ActorState` variants equal pinned bytes. Guards "appended, not inserted".
- `planted_at_theta_one_is_the_kind`. From 3,000 random states per kind (prices over twelve
  decades, coins, stocks and plants over eighteen, spending shares that round to 1), at θ = 1
  the planted TypeDesk, maker and capacity desk make the plant-free kind's orders, budgets and
  produce deltas bit for bit, the plant's wear and the planted state aside. Guards §3.9's
  branches: c_full = c, kr 0, no order, y = z_got, the budget total the outlay, one burn.
- `planted_rules_are_the_spec`. One tick per planted kind at fixed states, with the plant below
  target, the coin cap binding and not. Each quantity equals a literal computed in the test from
  LOOP-SPEC §2.2–§2.5: P_K, kr, c_full, O_f and zcap, q or z, K\*_p (`Herd` and `Bundles`), I
  (`Target` and `Gap`), the horse order after the plant's cost, the lines, the budget total, B,
  the split, y, used, I′, the burns, the mints and the wear. Guards every dynamic line.
- `planted_arithmetic_never_fails`. 3,000 random states per planted kind, with carried goods
  beyond the plan and holdings one ulp either side of the bundle. Admission accepts every order;
  every burn in produce fits; every wear fits. Guards the budget totals (§3.5) and the build's
  bound (§3.6 step 5).
- `planted_desks_take_carried_goods`. A planted desk holding a carry: B is read over every unit,
  bound by that tick's labour; the split is the plan's; with no plant order all of B runs.
  Guards decision 273.
- `plant_wears_what_it_held_at_decide`. Upkeep burns u·P as recorded at decide; units built
  this tick do not wear; P′ = P + I′ − u·P. Guards the timing.
- `fixed_plant_is_drs`. At δ_p 0, over 200 random states: no order, no wear, and output
  P^(1−θ)·B^θ (at most κ·H on the capacity desk). Guards E1's third item.
- `a_planted_maker_reads_its_reservation_at_c_m`. At a markup below ψ on c_m but above it on
  c_full, the planted maker offers nothing; `maker_reservation` reads the same. Guards
  decision 295.

**Probe** (`crates/probe/tests/loops.rs`):
- `loops_tapes_are_their_generators_output`. The six committed tapes equal the generator's
  output.
- `loops_instances_are_fundeds`. The generator's params equal `instances.json`'s `tape_params`
  bit for bit. The harness's points (B, B-cut, B-flow at 3 δ × 5 targets) equal its `points`
  within 1e-15 relative. s1 equals 40.47205917167808, and each κ_p·X equals the `plants` rows'
  K\* within 1e-12. The test reads the file with a small reader of its own; no new dependency.
- `loops_rest_point_is_the_oracles` (E2). At every instance and funded target, three ticks from
  genesis leave every observable within 1e-12 of the oracle in log, every market trading with
  both fills 1, and each plant order equal to u·K\*_p within 1e-12. LB1 also at 12, 24 and 365
  ticks a year.
- `loops_theta_one_is_the_plain_tape` (E1). LB1 on hold, `w*2` and `r*2` with every plant at
  θ = 1, against the same tape without plants: every price, observable, coin and holding but
  the plant goods, bit for bit over 2,000 ticks.
- `loops_fixed_plant_is_drs` (E1). LB1 with `--fixed-plants` over 2,000 ticks: the plant
  holdings stay at Q, no plant good is minted or burned, and no plant order is placed.
- `loops_conserve_every_tick`. On every committed loops tape, at rest and from `w*2` with every
  desk's coin × 0.1 and every plant × 2: the ledger each tick, the money stock within 1e-12 of
  itself over 2,000 ticks, every `Depreciation` burn in upkeep and exactly δ·H of heads or u·P of
  plant, every plant mint in produce.
- `loops_tapes_load_and_run_deterministically`. Two runs of each tape give identical reports and
  hash streams for 500 ticks.
- `loops_batteries_are_registered`. The tier counts: wet with three plants 20, 24, 25, 28 and a
  stocks family of 28; with two plants Tier 3S and stocks 26; with one 24; with none 22; the
  flow control with plants 18, 22, 23, 20, 20, and LW0's 3S and stocks 16.
- `loops_runs_apply_as_named`. The plant stocks, the coins, `b*F` on the land factor (fodder
  land, pasture, the horse-day desk's land) and `--reserve none` each change what their names
  say.
- `loops_harness_readouts_are_the_rows`. Over a short run with dead ticks and the bound: each
  market's dead count equals its CSV rows below the bar; `bound_ticks` equals the rows with
  `supply_ratio` ≥ 1; the lows, `pk_high` and the plants' 5% ticks equal what the rows give.
- `harness_reads_the_running_cost_from_the_tape`. At LB1, each row's `running_cost` is
  (0.0 + 0.0176·p_f) + 0.1·w at that row's prices; at H1 it is still p_f. Guards §8.3's fix.
- `loops_carry_meets_the_mirror_at_tick_one`. At LB1 `w*0.5`, where the carry is 0.60 of the
  capacity desk's fodder flow, the engine's tick-1 outputs, fills and coins equal `lm_carry`'s
  within 1e-12. The literals come from the mirror at build time, with the script named beside
  them. Guards decision 273 in the gate.
- The existing pins stay and must pass unchanged: `horses_tapes_are_their_generators_output`,
  `horses_tapes_keep_their_world_ids`, `markets_i0_nests_appb`,
  `markets_tapes_are_their_generators_output`, `probe_battery_csv_unchanged` and the gate's hash.

**GUI** (`crates/gui`):
- `inspector_reads_planted_states`. `state_fields` lists each planted variant's desk fields and
  its plant record; the inspector names the three planted kinds.

## 11. Where the build's arithmetic parts from the mirror's

Each is rounding at most, inside E0's 1e-12.

1. **One buy line per good** where the mirror has a running and a plant line. Market demand is
   summed in the engine's order, not the mirror's (`lm_mirror:448`).
2. **The split** B·z/(z + s·I) where the mirror takes the two lines' Leontiefs apart
   (`lm_mirror:529–543`). With one fill ratio per good they agree.
3. **The build's bound** (§3.6 step 5) and z_got ≤ B. They move I′ by an ulp where the split
   rounds past B.
4. **The capacity desk's used bundles** are at most z_got. The mirror's carry can go an ulp
   negative (`make_lm_carry.py`, C_Fc).
5. **The plant's law of motion**, P + I′ − u·P against (1 − u)·P + I′ (as HORSES-RULES §6.4's
   horse stock).
6. **Powers** in libm against Python's `**`.
7. **The genesis plant** from the rest ratio against `ratios` at genesis prices
   (`lm_mirror:218–220`). E0 starts from the engine's own genesis, so this moves nothing there.
8. **The technique** carried as 1 − x (P2.2a's).
9. **Budgets** bind only at rounding (§3.5).

The build departs from the registration in one place: LN5 and LF4 are not run (A2).

## 12. Decisions proposed, and open items

Each decision is open to veto; the alternative named is the registered one (R6).

286. **The plant good is core's new untraded good**: `untraded: true`, `Indefinite`, no price
     rate, no market, no genesis price, and no order may name it (§1, §6). LOOP-SPEC §2.2 says
     it has no market, and core refuses a plain good without a price rate. Alternative: a plant
     market at price rate 0 that no one trades, with the kick set restricted to the instance's
     markets. Certify's kick set kicks every market, and a kicked plant price never decays.
287. **Planted states nest the kind's state**: `PlantedType`, `PlantedMaker` and
     `PlantedCapacity`, each {desk, plant}, appended after `Owner`, with one `PlantState`
     {held, target, order, run, built} (§3.8). Alternative: flat structs per kind, which repeat
     the plant's five fields three times.
288. **Budgets**: the planted TypeDesk and maker cut theirs from outlay + P_K·I, the capacity
     desk from its coin, as now (§3.5). At I = 0 each is the plant-free chain bit for bit.
     Alternative: every planted desk from its coin, which parts from the plant-free budgets by an
     ulp where the chain binds at the outlay, and so breaks θ = 1's value-for-value.
289. **One burn per bundle good in produce**, of the running and the build bundles together,
     the build at most what B leaves (`max_remainder`, `max_scale`), and the capacity desk's
     used bundles at most z_got (§3.6). Alternative: a running and a build burn per good, which
     can overdraw by an ulp on a two-lot holding (a carry).
290. **P2.2b extends `probe::horses`**: the loop ids, the flow control as a config, the land
     factor as the instance's b, the binaries `horses` and `horses-tape`, and
     `tapes/loops-<id>.ron` for LB1–LB3 and LW1–LW3 (§7, §8). Alternative: a new module
     `probe::loops` with binaries of its own, which would copy most of P2.2a's harness.
291. **LN5 and LF4 wait for O51** (LOOP-SPEC-A2). Alternative: build O51's seller and buyer
     netting in P2.2b, which needs its own mirror scan first (O51).
292. **The new readouts are appended** to the CSV, `summary.tsv` (nine columns, the same names
     and order for every instance) and `stats.tsv`; P2.2a's columns keep their place and meaning
     (§8.5). Alternative: columns for loop instances only, which gives `summary.tsv` two shapes.
293. **E4's engine g is P2.2a's reading** (HORSES-RULES §6.6); the mirror's reading, to 1e-4 of
     the peak (`lm_run:303–311`), is reported beside it. Alternative: score the mirror's reading.
294. **E1's fixed plant is a plant at δ_p 0**, started at the design's plants
     (`--fixed-plants`; §3.9): no order, no wear, no user cost. Alternative: an order rule
     `Fixed`, a third variant that only the check would use.
295. **A planted desk's margin reads the bundle cost c, not c_full**: so do its tilt and the
     maker's reservation, as in the mirror (`lm_mirror:284–288, 376`). Alternative: c_full. The
     maker's markup would then rest at θ (0.8), not 1, and ψ 0.25 would act where the markup on
     c is below 0.3125.
296. **Planted desks take the cash rule only, and no own input**: the step rule, a TypeDesk's
     own input and a maker's own horse-days are refused with a plant (§6). The mirror ran none
     of them. Alternative: allow them, untested.

Open items:

- **O69. The untraded good has no valuation.** A plant is held and shows in holdings, but no
  lens or report values it. At replacement cost it is worth P_K a unit, about 99 weeks of its
  desk's revenue at θ 0.8 and δ_p 10% (FUNDED §8). A wealth lens would need it. GOODS-CHAIN's
  work in progress (its E5) should reuse the untraded good.
- **O70. Two readings of the lowest baskets.** The engine's is the sum of each household's
  baskets, each bound by its own scarcest item (P2.2a's). The mirror's is the smaller of the
  goods and the space eaten over both households (`lm_mirror:680`). They agree when both
  households bind on the same item. E3 scores the lowest baskets within 5%; the report names any
  row where the two part.
- **O71. LN5 and LF4 are registered and unrun** (A2; decision 291). They run when O51's roles
  exist, against A1's numbers.
- **O72. The run's cost.** L is 202,000–232,000 ticks, and Tier 3 and 3S run again at 10·L:
  about 2.3 million ticks for each of 53 runs per verdict instance, beside E4's kick sets at
  H = L (twelve kicks and a base). The build should time one LB1 run at 10·L on WSL before the
  waves are planned.

## 13. Files, and the order of work

**P2.2b.1, the build.** Core's untraded good; markets' admission; the agents' plant; the probe's
instances, generator and readouts; the GUI's fields; the six tapes; ENGINE.md and TAPE.md
amended; this file amended "as built". Before the commit: `scripts/gate.sh` and `scripts/gui.sh`
green on WSL (`CARGO_TARGET_DIR=/root/scratch/target-p2b`) and on Windows
(`D:/rustyecon-targets/p2b`); the pins of §1 unchanged; the 13 tapes' 2,000-tick streams equal
P2.2's on both machines.

**P2.2b.2, E0, before any scored run.** A trace diff in `D:/rustyecon-p2b/<label>/tracediff/`,
after `D:/rustyecon-p2l/idle-engine/tracediff/tracediff.py`:
- **The mirror** is `lm_carry.py` with its imports (`lm_mirror.py`, `h_mirror.py`,
  `i_mirror.py`, and `ag/model.py`, `goods.py`, `designs.py` from
  `D:/rustyecon-p2l/loop-mirror/ag/`), copied unedited, their sha256s checked against the
  registration.
- **Genesis from the tape** (`horses tape NAME --inst ID`): w, r, p, p_f, p_h, p_K from the
  genesis prices; x = 1 − the good desk's share; Mg, Mc, Mm, Mf, Mw, Mp from the coins; IG, If,
  Zs, Hc from the holdings; F from the maker's heads (own 0); Pc, Pf, Pm (the flow control's Pc
  is `plant.traction`) from the plant goods; `genesis` True.
- **The map from the tape's params**: `rule_b` at the tape's δ, tick length and land factor, the
  loop on where `inst.fodder.traction` is 2 and off where it is 0; `cfg` "flow" for the flow
  control; `dials(C2G, …)` with ψ from `reserve.maker` (0 where absent), `rate_F`
  from `rate.fodder`, and `pc_target` from the capacity plant's `target`; each plant
  `plant(θ, δ_p, tpy, rule, size=s)` with s from the tape, rule "a" for `Target`, "b" for `Gap`,
  and "none" at δ_p 0.
- **The comparison.** Every observable of §8.1 (`lm_run.observe`), every coin and plant, each
  tick, in log. The columns are HORSES-RULES §6.4's: the largest gap, the largest but volumes,
  volumes absolute over their oracle volume, and the mirror against itself with one coin, horse
  stock or plant moved an ulp.
- **If it parts at tick 1**, the carry is named, and an A3 comes before any scored run.

**P2.2b.3, the runs** of §9, in waves on WSL, scored into `docs/probe/results/loops/`.

**P2.2b.4, the reviews and the report**, `docs/probe/LOOPS.md`, in HORSES.md's shape.

Scratch is `D:/rustyecon-p2b/<label>/`; this step's is `D:/rustyecon-p2b/rules/` (the mirror's
read-only copy is in `mirror/`).

## 14. As built (P2.2b.1, 2026-09-30)

Label `build-plant`, scratch `D:/rustyecon-p2b/build-plant/`. The build is core's untraded good,
markets' admission, the agents' plant, the GUI's fields, the loop instances and their six tapes,
and ENGINE.md and TAPE.md amended ("Amended at P2.2b.1" in each). The harness (§8) is not built
(item 4).

### 14.1 What was built

- **Core** (`crates/core`). `RawGood.untraded` (absent `false`, not written when `false`). The
  goods' load checks of §6, with the new `LoadErrorKind::UntradedGood`. `World::has_market` and
  `World::is_untraded`; `World::markets()` walks the goods with a market only. `apply` refuses a
  book write to an untraded good (`NoMarket`), `validate` refuses a state whose untraded slot was
  written, and no escrow of one is a holder. The resolver's `has_market` and `is_untraded`.
- **Markets.** Admission and clearing's slot refuse an untraded good, as a currency.
- **Agents** (`crates/agents/src/roles/plant/`). `spec.rs`: `RawPlant`, `Plant`, `PlantOrder`,
  `PlantTarget`, `resolve_plant`. `rules.rs`: the plant's arithmetic (`PlantNow`, `Ratios`,
  `Plan`, `split`, `built`, `burned`, `bundles_held`, the wear) and the three planted roles,
  `PlantedType`, `PlantedCapacity` and `PlantedMaker`. The kinds' own rules take the plant as an
  option, so a desk without one runs its kind's code: `TypeDesk::plan`/`run`,
  `CapacityDesk::plan`/`run`/`wear`, `MakerRole::plan_stock`/`run_stock`/`wear_stock`. The states
  (`PlantState` and the three planted variants after `Owner`), the cast's members and §6's
  checks.
- **Engine.** One change (item 1).
- **Probe** (`crates/probe/src/horses/loops.rs`). §7.1's instances (all but LF4 and LN5), the
  oracle point (§7.5), the dials (§7.3), genesis (§7.4), the params (§7.2) and the tape.
  `horses-tape --inst <loop id>` writes them, with `--tpy`, `--dials c2g|c2g13`, `--one-sided`,
  `--reserve PSI|none`, `--theta X`, `--plant-delta D` and `--fixed-plants`
  (`probe::horses::cli::parse_loops`). `tapes/loops-{lb1,lb2,lb3,lw1,lw2,lw3}.ron`.
- **GUI.** `state_fields` lists a planted state's desk fields, then `plant.held`, `.target`,
  `.order`, `.run`, `.built`; the inspector names the planted kinds (`TypeDesk, planted` and so
  on) and an untraded good; the outliner marks one.

### 14.2 Where the build departs from §1–§13

1. **The engine changed in one place** (§1 said nothing would). `RunErrorKind::ForeignWrite`
   carries its delta boxed. The capacity desk's planted state holds 11 numbers, so `ActorState`
   and every `StateDelta` grew, and clippy's `result_large_err` refused every step's `Result`. No
   run, hash or message changes (decision 297).
2. **The flow control's horse-day desk reads its own input from `inst.traction.own`**, a new
   param at 0 (§2.2 and §7.2 listed `inst.traction.fodder`, `.labour` and `.land` only; the
   type desk needs a param for `recipe.own`). Decision 298.
3. **No float constant on the engine path.** The engine's scan refuses a named float constant,
   so `p_inverse` on no plant is "none", not +∞: the capacity desk's z is then outlay/O, or 0
   where O is 0 (§4.2's zcap). And a capacity desk whose running recipe has no coefficient above
   0 bounds its bundles by κ·H, the plain desk's bound, not +∞ (§3.6 step 1). The first arises
   only where a capacity desk at θ < 1 holds no plant, and there its output is 0 whatever it
   runs; the second only where a dated `SetParam` zeroes every running coefficient, which §6
   refuses at genesis. Decision 299.
4. **The loop instances are a module of their own, and the harness is not built** (decision 290
   extended `probe::horses`; decision 300). `probe::horses::loops` has its own `Instance`,
   `Point`, `Setup`, `Displacement`, genesis and tape, so P2.2a's `Instance` and harness are
   untouched. The harness's rule-B observables (§8.1), dead ticks, idle and classes (§8.2), new
   readouts (§8.3, the running cost read from the tape included), run grammar and families
   (§8.4), outputs (§8.5) and run lengths (§8.6) come with the harness step, which folds these
   instances into the harness or reads them from this module (O73). Until then `horses` reads
   rule A's instances alone; only `horses-tape` knows the loop ids.
5. **Tests.** Built as §10 lists them, with these differences:
   - `loops_rest_point_is_the_oracles` reads the fixed point from the Sim, not the harness's
     observables: every price (posted and next), both fills, every coin, stock and plant, each
     plant's target and order, against genesis. LN6's bar is 1e-11, not 1e-12: its order takes
     the whole gap K*_p − P each tick, so it passes the plant's rounding to its order at 1/u
     (about 500) times the order's size, and s (about 40) times that into its bundles' demand.
     Its worst gap is 3.0e-12 (fodder's buyer fill at b × 0.5, tick 2). Every other instance and
     target is within 1e-12. Decision 301.
   - §10's `a_planted_maker_reads_its_reservation_at_c_m` is worded backwards. c_full > c_m, so
     the markup on c_full is always the smaller, and "below ψ on c_m but above it on c_full"
     cannot happen. The test takes a markup above ψ on c_m and below it on c_full (the planted
     maker offers, as it reads c_m) and half ψ on c_m (it offers nothing).
   - `planted_rules_are_the_spec` binds the coin cap with s_Kp at 1e4 a year and under `Gap`. At
     the registered s_Kp the cap does not bind near the rest: the coin left after the running
     bundle buys about 0.1·K*_p of plant, some fifty times the order u·K*_p. A fourth case sets
     every plant at twice s1, where O_f parts from O (the mutant O_f = O, §14.4).
   - Added: `loops_tapes_name_their_plants_untraded` (each plant untraded, held by its desk
     alone, its slot never written), and `horses_tapes_keep_their_world_ids` now pins the
     `world_id` and `tape_hash` of all 13 horses and markets tapes.
   - Not built, since they need the harness or E0: `loops_batteries_are_registered`,
     `loops_runs_apply_as_named`, `loops_harness_readouts_are_the_rows`,
     `harness_reads_the_running_cost_from_the_tape` and `loops_carry_meets_the_mirror_at_tick_one`
     (O73).
6. **The agents crate gains `bincode` as a dev-dependency**, for `old_states_keep_their_encoding`
   (the lockfile's one change is that edge).

The arithmetic parts from the mirror only where §11 says. The genesis plant on the capacity desk
is κ_p·(κ·H), as §7.4 writes it; the mirror writes (κ_p·κ)·K_c.

### 14.3 What was checked

- **R1.** The 13 horses and markets tapes' 2,000-tick hash streams equal the P2.2 report's on WSL
  and on Windows (`D:/rustyecon-p2b/build-plant/tapes-bit.sh`, after l010's). The gate world's
  final hash is `0x61f9c8529131ff17`; appb's and demo-gb's pins pass in the gate and the GUI's
  gate. Every committed tape keeps its `tape_hash` and `world_id` (the extended pin).
- **The rest point** (E2): every built instance at every funded target (b × 1, 1.1, 0.9, 2, 0.5;
  all 115 funded), three ticks, within 1e-12 but LN6 (item 5); LB1 at 12, 24 and 365 ticks a
  year.
- **θ = 1** (E1): LB1 against LN1 and LW1 against LW0, at hold, from w × 2 and from r × 2: every
  market line, every holding but the plants, and every state read as its kind's, bit for bit
  over 2,000 ticks. And from 3,000 random states per tape, each planted kind's decide, produce
  and upkeep are its kind's bit for bit (`planted_at_theta_one_is_the_kind`).
- **The fixed plant** (E1): LB1 with `--fixed-plants`, 2,000 ticks: every plant at Q bit for bit,
  no plant minted, burned or ordered. And from 200 random states per tape, output
  P^(1−θ)·B^θ, at most κ·H on the capacity desk (`fixed_plant_is_drs`).
- **Conservation**: the six tapes at rest and from w × 2 with every desk's coin × 0.1 and every
  plant × 2, 2,000 ticks: the ledger each tick, the money stock within 1e-12, every
  `Depreciation` burn in upkeep and exactly δ·H of heads or u·P of a plant, every plant mint in
  produce.
- **FUNDED**: the generator's params equal `instances.json`'s `tape_params` bit for bit; the 45
  points (B, B-cut and the flow control at three δ and five land factors) within 1e-15 relative
  (the good's price within the rounding of P_s, since the file writes it as P_s − h); s1 is
  40.47205917167808; and the 80 plant rows' K* equal κ_p·X within 1e-12
  (`loops_instances_are_fundeds`).
- **The build's mutants**: see §14.4.

### 14.4 The build's mutants

Each change was undone, one at a time, in a copy of the tree
(`D:/rustyecon-p2b/build-plant/mutants/mutants.py`), and the named tests run.

28 mutants, each killed by at least one named test (the log is
`D:/rustyecon-p2b/build-plant/mut/mutants.log`). Two survived the first pass and a test was
strengthened for each: the split's cap at B (`planted_desks_take_carried_goods` now holds B an ulp
from its share, where fl(B·z)/z rounds above B) and O_f = O (`planted_rules_are_the_spec` now has
a case off s1, where O_f parts from O).

| mutant | killed by |
|---|---|
| `World::markets()` keeps untraded goods | `untraded_goods_load_without_a_market`, `kick_skips_untraded_goods` |
| `apply` writes an untraded slot | `untraded_slots_are_never_written` |
| `validate` passes a written untraded slot | `untraded_slots_are_never_written` |
| admission takes an untraded good | `admission_refuses_orders_on_an_untraded_good` |
| a genesis price on an untraded good loads | `untraded_goods_are_checked` |
| an untraded good may be perishable | `untraded_goods_are_checked` |
| a `ScalePrice` of an untraded good loads | `untraded_slots_are_never_written` |
| `untraded: false` is written | `untraded_flag_keeps_canonical_text`, `horses_tapes_keep_their_world_ids` |
| a role may name an untraded good | `plants_are_checked_at_load` |
| a scripted line may name an untraded good | `plants_are_checked_at_load` |
| the resolved type desk writes `plant: None` | `horses_tapes_keep_their_world_ids` |
| the raw capacity desk writes `plant: None` | `horses_tapes_keep_their_world_ids` |
| the planted variants inserted before `Owner` | `old_states_keep_their_encoding` |
| the type desk's budgets from the outlay alone | `planted_rules_are_the_spec` |
| the plan at c, not c_full | `planted_rules_are_the_spec`, `loops_rest_point_is_the_oracles` |
| the horses ignore the plant's cost | `planted_rules_are_the_spec` |
| the maker's cover at c, not c_full | `planted_rules_are_the_spec` |
| the maker's reservation at c_full | `a_planted_maker_reads_its_reservation_at_c_m` |
| the plant wears what it holds now | `plant_wears_what_it_held_at_decide`, `loops_conserve_every_tick` |
| the build unbounded by B | `planted_arithmetic_never_fails` |
| the split without its cap at B | `planted_desks_take_carried_goods` (second pass) |
| B over the plan's bundles only | `planted_desks_take_carried_goods` |
| θ out of range loads | `plants_are_checked_at_load` |
| two desks share a plant | `plants_are_checked_at_load` |
| the herd's plant on a type desk | `plants_are_checked_at_load` |
| the plant order's coin cap dropped | `planted_rules_are_the_spec` |
| the herd's target without κ | `planted_rules_are_the_spec`, `loops_rest_point_is_the_oracles` |
| O_f = O always | `planted_rules_are_the_spec` (second pass) |

Not mutated: clearing's slot check, which no path reaches with an untraded good (admission
refuses its orders first), and the engine's boxed delta, which clippy guards.

### 14.5 Decisions and open items

Decisions 297–301 and open items O73–O74 are in STATE.md.

## 15. The harness as built, and E0–E2 (P2.2b.2, 2026-09-30)

Label `build-harness`, scratch `D:/rustyecon-p2b/build-harness/` and `D:/rustyecon-p2b/e0/`.
The harness (§8) is built, with the five tests §14.2 item 5 left. E0, E1 and E2 ran before any
scored run ([results/loops/e0.md](results/loops/e0.md)). No scored run (E3–E11) has been made.

### 15.1 What was built

- **`probe::horses::loops`** is a module directory now (`loops.rs` moved to `loops/mod.rs`). It
  gains:
  - `perturb`: §8.4's grammar on a loop instance (P2.2a's terms, the plants among the stocks,
    `b*F@genesis` and `b*F@dated` on the land factor), `battery`, `tier3s` and `family`;
  - `harness`: §8.1's observables and targets, the rows, §8.3's readouts (`Readouts`), `run`, the
    CSV, and the summary and stats lines;
  - `probes`: the one-tick elasticity probe, the kick set and §8.6's rule (`run_length`, with
    the mirror's 3·T6 from `lm_lin.json` as a table, `mirror_three_t6`);
  - `cmd`: the `horses` binary's commands for a loop instance.
- **The setup** gains dated shocks: `Setup::shocks`, `b_at`, `instance_at` and `b_params`. The
  tape writes their params and `SetParam` events as P2.2a's does. A tape without a shock keeps
  `events: []`, so the six committed tapes are unchanged. The displacement's share is P2.2a's
  `ShareAt`, for `x*/2`.
- **Shared with P2.2a.** The loop harness reads P2.2a's row, statistics (`Stats`,
  `StockTrack`), classifier, mode A check, kick set (`kick_set_of`, split out of `kick_set`)
  and reports. The reports moved out of the binary into `probe::horses::report`: `SUMMARY` (59
  columns), `summary_fields` and `stats_fields`.
- **The running cost is read from the tape** (§8.3), in P2.2a's harness too:
  `probe::horses::harness::running_cost` sums the capacity desk's (or M1's owner desk's) running
  recipe in the rule's order, at the params in force. At every P2.2a instance it is p_f bit for
  bit.
- **`horses`** sends a command whose `--inst` names a loop id to `loops::cmd::main`. The
  commands are `run`, `family`, `list`, `tape`, `kick`, `slowest`, `elasticity` and `point`;
  `openloop` is not ported. `elasticity` prints L by §8.6's rule.
- **The outputs** are §8.5's. A P2.2a instance writes the nine new `summary.tsv` columns as `-`.

### 15.2 Where the harness departs from §8–§10 and §13

1. **The loop instances keep their own module** (decision 300), and the harness sits beside
   them, on P2.2a's shared parts, not inside P2.2a's `Instance` (decision 302). `horses` and
   `horses-tape` take the loop ids as §7 and §8 say.
2. **The fodder desk's horse-days are its one buy line** (`hours_fodder`, `hours_fodder_low`):
   the running and the plant bundles together (§3.5), 2·q_f at rest. The mirror's `minHf` reads
   its running line alone, θ of the line at rest. Reported, not scored; E7 scores the tasks'
   (decision 303).
3. **`dead_by`, `pk_high` and the plants' readouts in the flow control**: `pk_high` is `-` (no
   horse market). `markup_low` is `inf`, P2.2a's form when no maker reads a markup.
4. **The run length's third term** is the mirror's 3·T6 at 52 a year, scaled by tpy/52 and
   rounded up to a thousand at other tick lengths. Where the mirror's T6 is infinite, the
   instance runs at LB1's L at the same tick length (§8.6).
5. **E0's script** (§13) is `D:/rustyecon-p2b/e0/tracediff.py`, not
   `D:/rustyecon-p2b/<label>/tracediff/`. It adds two columns to HORSES-RULES §6.4's:
   - the mirror with an ulp added after every tick;
   - at each tick where the horse market's volume parts, its short side, the cancellation and
     the gap over (the cancellation × the inputs' gaps).

   The flow control's mirror P is rule B at the instance's δ and the tape's land factor. The
   flow tape has the horse-day desk's recipe but no horse params, so the script asserts the
   mirror's flow recipe equals the tape's bit for bit.
6. **The allowed parting is read on both sides of the horse market** (decision 304). HORSES-RULES
   §6.4's cancellation is read as the cleared volume's, whichever side is short: the capacity
   desk's order or the maker's offer (its finished stock less its cover band). §6.4's own text
   names the maker's finished stock among the rounding sources.
7. **`loops_carry_meets_the_mirror_at_tick_one` does not run at LB1 w × 0.5** (§10). There
   labour binds every planted desk at tick 1, so the carried units never reach the bundles, and a
   build without the carry would pass. The test runs where the carry binds instead:
   - LB1 p[traction] × 0.5 (the capacity desk and the maker);
   - LB1 p[fodder] × 0.5 (the fodder desk);
   - LW1 p[traction] × 0.5 (the horse-day desk).

   The no-carry mirror makes 3.2–6.4% less at tick 1 there. The literals come from
   `D:/rustyecon-p2b/e0/carry_literals.py`.
8. **`loops_harness_readouts_are_the_rows`** runs LB1 w × 0.3 (dead ticks in four markets),
   LB1 w × 3 (labour's bound reached) and LW1 r × 2, for 300 ticks each. It also reads the
   horse-days by buyer apart from the readouts: the two buyers' fills sum to the cleared
   horse-days every tick, and at rest they are Y·J(x\*) and 2·q_f at LB1, LW1 and LC1.

### 15.3 E0, E1 and E2

The numbers are in [results/loops/e0.md](results/loops/e0.md).

- **E0 passes.** At LB1's nine runs and LW1's three, 2,000 ticks each from the engine's own
  genesis, nothing parts at tick 1, and nothing parts above 1e-12 before tick 140.
  - Eight runs agree within 8.7e-13 on every observable, coin and readout.
  - Four (r × 2, b × 2, heads × 2, heads × 10) part on the horse market's cleared volume, by 9.4e-12,
    3.9e-12, 7.1e-12 and 2.9e-11. Where it parts, the volume is a difference of terms 3 to 507
    times its size. Each gap is its inputs' gaps (at most 1.6e-13) times that ratio, within a
    factor of 6.4, and the mirror with an ulp each tick parts farther.
  - The maker's coin and next output (the volume's revenue) and, after heads × 10, the horse
    price (1.8e-12) follow the volume.
  - The withheld ticks agree tick for tick.

  No amendment is needed.
- **E1 holds.** The pins and the 13 tapes' streams on both machines. θ = 1 is the plain tape
  (P2.2b.1's test). The fixed plant is drs, and its trace diff against lm_carry at plant rule
  "none" parts only on the horse market's volume (1.7e-12, 2.4e-12), by the same cancellation.
  P2.2a's CSV, `stats.tsv` and first 50 summary columns are byte-identical before and after this
  step, on 17 runs.
- **E2 holds.** Mode A at L passes at LB1–LB3 and LW1–LW3, with largest gaps 8.9e-16, 2.4e-15,
  5.6e-16, 2.2e-16, 2.0e-15 and 8.9e-16. The engine's L equals the mirror's: 211,000, 202,000
  and 232,000.

### 15.4 Tests and mutants

**Tests.** The five §10 tests are built in `crates/probe/tests/loops.rs`:
`loops_batteries_are_registered`, `loops_runs_apply_as_named`,
`loops_harness_readouts_are_the_rows`, `harness_reads_the_running_cost_from_the_tape` and
`loops_carry_meets_the_mirror_at_tick_one`.

**Mutants.** Each change was undone, one at a time, in a copy of the tree
(`D:/rustyecon-p2b/build-harness/mutants/`). 16 mutants were run in two passes.
- Two survived the first pass, and their tests were strengthened:
  - swapping the horse-days' buyers (the test now reads the buyers apart from the readouts);
  - the genesis carry (the test moved to runs where the carry binds).
- Build-plant's two mutants that cap the bundles at the plan (on the type desk and the capacity
  desk) survive the carry test by construction: at its runs the carry fills the shortfall up to
  the plan, not past it. They stay killed by `planted_desks_take_carried_goods`.

| mutant | killed by |
|---|---|
| the running cost without its labour | `harness_reads_the_running_cost_from_the_tape` |
| the loop rows' running cost is fodder's price | `harness_reads_the_running_cost_from_the_tape` |
| a dead tick below a quarter of the target | `loops_harness_readouts_are_the_rows` |
| labour's reach capped at 1 | `loops_harness_readouts_are_the_rows` |
| the horse-days' buyers swapped | `loops_harness_readouts_are_the_rows` (second pass) |
| `pk_high` keeps the lowest | `loops_harness_readouts_are_the_rows` |
| a plant's band at 10% | `loops_harness_readouts_are_the_rows` |
| the herd's plant target without κ | `loops_harness_readouts_are_the_rows` |
| a dated shock a tick late | `loops_runs_apply_as_named` |
| the flow control's shock leaves the horse-day's land | `loops_runs_apply_as_named` |
| `x*/2` sets x, not 1 − x | `loops_runs_apply_as_named` |
| the battery without `x*/2` | `loops_batteries_are_registered` |
| Tier 3S without the plants | `loops_batteries_are_registered` |
| a genesis lot lives no longer than a lot made at tick 0 | `loops_carry_meets_the_mirror_at_tick_one` (second pass) |

### 15.5 Decisions and open items

Decisions 302–304 and open items O75–O76 are in STATE.md. O73 is closed.
