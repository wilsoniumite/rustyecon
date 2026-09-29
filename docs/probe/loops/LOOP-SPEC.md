# LOOP-SPEC: the loop step in the mirror (GOODS-CHAIN §6 step 3)

Dated 2026-09-29. Work label `loop-mirror`, for the loop stage's groundwork (L0) on branch
`phase2-loops` (worktree `D:/rustyecon-wt/p2l`; nothing in it was changed, nothing committed).
Scratch: `D:/rustyecon-p2l/loop-mirror/`. It registers the mirror's predictions for P2.2b's engine
run before that run is built (O49, decision 235).

**Evidence.** A Python mirror of P2.2b's economy, `model/lm_mirror.py`. It imports P2.2a's horse
map unedited (`h_mirror.py`, copied from `D:/rustyecon-p2g/frame/model/` with its `SHA256SUMS`)
and idle-scan's reservation map unedited (`i_mirror.py`). It adds CAPACITY's plant
(`capacity/cap.py`, copied unedited, the reference) and the chain8 county
(`D:/rustyecon-p2l/funded/instances.json`, copied as `model/funded_instances.json`). No engine
run was made. Every number below is the mirror's.

**Conditions of every number, unless its line says otherwise.** The county is FUNDED's chain8
under rule B. The dials are C2g at 52 ticks a year, with ρ 0, J_b 1 and ex-post assignment. The
maker's reservation is on at ψ 0.25 (IDLE-SPEC). Plants sit on the fodder desk, the capacity desk
and the maker, at θ 0.8 and a plant δ of 10% a year. Each plant is built from s1 bundles of its
desk's own recipe and ordered by M3's rule at s_K = 2δ_p. L is 202,000–232,000 ticks, and Tiers 3
and 3S run again at 10·L. The tolerance is 1e-3 in log. The classes are HORSES-RULES §5's.

## 0. The answer

**GO in the mirror, on one condition: the plant sits on every loop desk and the reservation stays
on.** The loop is rule B's horse: fodder is raised with horse-days, the horses are held by M3's wet
capacity desk and bred by M2. Plants go on all three loop desks (the fodder desk, the capacity
desk and the maker), with the reservation on. On that design, every scored run converges at the
three verdict instances (horse δ 8%, 10% and 4%). Every converged run of any instance ends within
2.0e-13 in log of its target's oracle point.

| id | what | mode A (largest gap) | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | stocks family | PL a tick: base, largest | verdict |
|---|---|---|---|---|---|---|---|---|---|
| **LB1** | the design, horse δ 8% (CHAIN's) | PASS (6.7e-16) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.996648, 0.997950 | **GO** |
| **LB2** | the design, horse δ 10% | PASS (1.0e-15) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.996197, 0.997501 | **GO** |
| **LB3** | the design, horse δ 4% | PASS (3.7e-14) | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.997927, 0.999068 | **GO** |
| LF1 | LB1 at θ 0.7 | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.997420, 0.998526 | GO |
| LF2 | LB1, plants' δ 4% a year | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.998074, 0.998484 | GO |
| LF3 | LB1, reservation off | PASS | 20/20 | 24/24 | 24/25 (24/25) | 27/28 (27/28) | 25/28 | as LB1 | **NO-GO** |
| LF4 | LB1, storable fodder, 13-week cover | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.996724, 0.998197 | GO |
| LF5 | LB1, fodder's rate 1.3 (c2g13) | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.996725, 0.997370 | GO |
| LF6 | LB1, capacity plant's target at the bundles in use | PASS | 10/20 | 4/24 | 3/25 (3/25) | 5/28 (5/28) | 5/28 | 0.999286, 0.999594 | NO-GO |
| LF7 | LB2 at θ 0.7 | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.997062, 0.998141 | GO |
| LF8 | LB3 at θ 0.7 | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.998482, 0.999308 | GO |
| LC1 | B-cut (no loop), the design otherwise | PASS | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 0.996554, 0.997773 | GO |
| LW1 | flow control, horse δ 8%, plants on both loop desks | PASS | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | 0.995990, 0.996963 | GO |
| LW2 | flow control, horse δ 10% | PASS | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | 0.996067, 0.996986 | GO |
| LW3 | flow control, horse δ 4% | PASS | 18/18 | 22/22 | 23/23 (23/23) | 20/20 (20/20) | 20/20 | 0.995801, 0.996871 | GO |
| LW0 | flow control without plants | PASS | 18/18 | 22/22 | 15/23 (15/23) | 14/16 (14/16) | 12/16 | 0.996820, 0.996909 | NO-GO |
| LN1 | no plant | PASS | 0/20 | 0/24 | 0/25 | 0/22 | 1/22 | 1.014741, 1.020828 | NO-GO |
| LN2 | no plant on the maker | PASS | 20/20 | 22/24 | 9/25 | 21/26 | 14/26 | 0.995700, 0.997346 | NO-GO |
| LN3 | a plant on the fodder desk only | PASS | 0/20 | 0/24 | 0/25 | 0/24 | 0/24 | 1.006157, 1.006792 | NO-GO |
| LN4 | no plant on the fodder desk | PASS | 20/20 | 24/24 | 25/25 | 26/26 | 25/26 | 0.996127, 0.997907 | GO in tiers; stocks family 25/26 |
| LN5 | storable fodder at a 4-week cover | PASS | 0/20 | 0/24 | 0/25 | 0/28 | 0/28 | 1.003155, 1.003465 | NO-GO |
| LN6 | the plants order their whole gap each tick | DEAD | 0/20 | 0/24 | 0/25 | 0/28 | 0/28 | 11.57, 11.62 | NO-GO |
| LN7 | B-cut without plants | PASS | 0/20 | 0/24 | 1/25 | 0/22 | 1/22 | 1.010356, 1.012722 | NO-GO |
| LN8 | B-cut without plants, fodder's rate 1.3 | PASS | 20/20 | 18/24 | 14/25 | 16/22 | 17/22 | 0.996286, 0.997900 | NO-GO |
| LN9 | no plant, fodder's rate 1.3 | PASS | 16/20 | 9/24 | 6/25 | 10/22 | 8/22 | 0.996283, 0.997777 | NO-GO |

Tier 3S counts differ by instance because each plant is a stock (Pc, Pf, Pm) and rule B's maker
holds no horses of its own. The negative controls (LN) ran at L only, with no 10·L repeat.
Mode A is the hold run at L, and the gap is its largest over the run.

**What the mirror found.**

1. **The loop is not where the instability starts.** Take rule B's horse with no plant (LN1): it
   is locally unstable (1.0147 a tick). So is the same economy with the loop cut (LN7, 1.0104),
   where fodder takes no horse-days. So the cause is CHAIN's maker, which buys its build's fodder
   on a fodder market that moves at C2g's 5.2 a year.
   - A maker that buys no fodder is stable (0.9963, `lm_diag.out`).
   - Fodder at 1.3 a year is also locally stable, but the battery still fails without plants
     (LN8, LN9).
   - The P2.2a remedy tuned on A0 (C2g) does not carry over to this maker.
2. **M3's horse stock does not play the plant's role** (§3). It is a Leontief cap. It stops an
   upswing, but it passes a fodder shortfall into horse-days one for one. If the horses were made
   the plant's smooth factor, the technique would move with prices: 2.3% more bundles per
   horse-day at b × 2. The plant on the capacity desk therefore sits beside the horses and uses
   the running recipe as its bundle.
3. **The capacity desk's plant is the one that matters.**
   - With it alone the map is locally stable (0.9969). With the fodder desk's plant alone it is
     not (1.0062, LN3: 0 of 93 converge).
   - After r × 2 it cuts the pass-through from 146–150 dead horse-day ticks to 1–2 (§7.5).
4. **The maker needs a plant too, because of the reservation.**
   - Without one (LN2), the reservation's step turns large displacements into a relay cycle: the
     horse price swings between 0.25 and 3.5 of target, and 16 of 25 Tier-3 runs orbit.
   - The cycle is not local: LN2's largest root is 0.9973.
   - With the maker's plant, every run converges, and the reservation still acts in 23 of LB1's
     97 battery runs.
5. **The reservation must stay on.** Without it (LF3), r × 2, heads × 2, heads × 10 and the
   workers' coin at × 0.02 and × 0.1 run away through the horse price at ticks 156–173. That is
   P8's idle runaway.
   - With it on, heads × 10 converges at every verdict instance with no dead tick. The glut no
     longer ends in the shortage IDLE-SPEC's O51 found on rule A.
6. **The pass-through needs no damper beyond the plant.**
   - Fodder's dead ticks after r × 2 (156–261) come from the county, not the loop. Its labour
     supply opens at 0.508 of target, and land's price takes some 200 ticks to come back at
     C2g's 0.1625 a year. The flow control shows the same (203).
   - Storable fodder at 13 weeks (LF4) is GO. It lifts the b × 2 trough from 0.70 to 0.96 and the
     Tier-3 floor from 0.255 to 0.411.
   - Storable fodder at 4 weeks (LN5) is unstable (1.0032 a tick). It is registered as a family
     (decision L5).
7. **O14 like for like.** Stocks with plants against the flow control with the same plants
   (LW1–LW3): the b × 2 trough is 0.70 against 0.67 in the base (−0.356 against −0.394 in log),
   and Tier 1–2's median is 26.7 years against 19.5, 1.37 times slower. The plants, not the
   stocks, carry the loop: without plants the flow control's b × 2 trough is 0.09 (LW0).
8. **Weekly stays the floor.** At 12 ticks a year LB1 is unstable (1.0053 a tick; Tiers 1–2 0/40).
   At 24 and 365 a year it converges 40/40, and its rate a year is within 0.8% of the weekly rate.
9. **What the plant does not remove.**
   - The frozen-price probe still shows the loop: 1.0616 a tick at LB1, against 1.19 without
     plants.
   - The open-loop probe's fodder imbalance reaches 1,665 per unit of the good's price at lag 200,
     against 36 with the loop cut.
   - The absorbing zero stays. LN4 shows it: with no fodder plant, the fodder desk's coin at
     × 0.02 diverges.

**NO-GO, plainly.** The mirror says NO-GO for rule B:
- without plants (LN1), and even with the loop cut (LN7);
- with the plant on the fodder desk only (LN3);
- without the maker's plant, under the reservation (LN2);
- with the reservation off (LF3);
- with the capacity plant's target set to the bundles in use (LF6);
- with a 4-week fodder store (LN5), or whole-gap plant orders (LN6);
- for the flow loop without plants (LW0);
- at 12 ticks a year.

What changes each to GO is the design: plants on all three loop desks, the reservation on, the
herd-based capacity-plant target, and weekly ticks. A slower fodder rate is not enough (LN8, LN9).

**Not done.** S1 without its pumping loop was not run. M3's county is unfunded at S1's point
(P_s 4.749 against T/N = 2.5), the same fault as 1g's HORSE county. It needs its own funded county
first (O-L3). The maker from bought inputs is tested here as CHAIN's horse maker (finding 1).

## 1. What this step covers

GOODS-CHAIN §6 step 3's list, and what this mirror does with each item:

- **A material market, with makers buying fodder.** Done. The maker buys 8 t of fodder a head, and
  the capacity desk buys 0.0176 t a horse-day.
- **The fodder–horse-days loop with stocks.** Done at C2g. C2L is not run, since the P2.2a mirror
  carries no desk tilt but the maker's, and C2g is the registered set (D-G13).
- **J_b > 1 and ρ > 0.** Not run. They stay Phase 3's (D-G11, D-G15).
- **Buyer netting.** Done, with storable fodder (LF4, M6).
- **Grain's cover.** Not in rule B.

The additions are CAPACITY's plant, composed with M3's horse holding (§3), the pass-through (§7.5)
and the like-for-like flow control (§7.6).

## 2. The economy and the roles, as the engine should build them

### 2.1 The county and the goods

The county is FUNDED's `chain8`: 1g's HORSE county at N 8. Per year, N is 416 and T 520. The
rest are h 1, χ_max 1/20, γ = 0.2 + 0.8x and k 1. Rule B's chain is per unit made, at every tick
length:

- fodder (a ton): 2 horse-days, 8 labour, 1 land;
- a head: 8 t fodder, 20 labour, 3 pasture (land);
- a horse-day: 0.0176 t fodder, 0.1 labour, and the head's wear.

Also: κ is 250 horse-days a head a year; the horse's δ is 8% a year, with 10% and 4% as
instances, each its own oracle point; J_b is 1 tick; ρ is 0. The cost shock b × f multiplies
fodder's land and a head's pasture together (FUNDED decision 242).

| good | life | made by | rate (C2g) |
|---|---|---|---|
| `labour` | Instant | `workers` | 5.2 a year |
| `land` | Instant | `provider` | 0.1625 |
| `fodder` | one tick (LF4: a year, stored) | `desk.fodder`: a TypeDesk with a plant | 5.2 |
| `horse` | Indefinite | `desk.maker`: M2 with a plant and the reservation | 5.2 |
| `traction` (horse-days) | one tick | `desk.capacity`: M3 wet with a plant | 5.2 |
| `good` | one tick | `desk.good`: P2.0's GoodDesk, ex post | 2.6 |
| `plant.fodder`, `plant.capacity`, `plant.maker` | Indefinite; no market, never traded | each desk itself, in produce | none |

What is new beyond P2.2a's kinds:

1. **The fodder desk buys horse-days.** Its recipe lists `traction`, 2 a ton. It is P2.1's
   TypeDesk as it stands, and its cost sums the bought goods first, then labour, then land. This is
   the loop.
2. **The maker's build buys fodder** (8 t a head) and takes no horse-days of its own (`own_hours`
   0). P2.2a's Maker already lists build goods; rule B only fills them in.
3. **A plant on three kinds**: TypeDesk, CapacityDesk and Maker (§2.2–§2.5).
4. **The maker's reservation** (IDLE-SPEC §6), ψ 0.25, on in every loop run.

### 2.2 The plant: what every planted desk shares

The plant is CAPACITY's: a stock the desk builds of its own recipe, holds, wears and never trades.

- **The good.** One `Indefinite` good per planted desk (D-G2's machine good). It has no market and
  appears in no other actor's spec.
  - Wear is an upkeep burn tagged `Depreciation`: δ_p times the plant the desk held when decide
    ran. That holding is recorded, as the capacity desk records `held`.
  - The build is a `Production` mint in produce, after the output is made. So a unit built at t
    serves from t + 1 (J 2, M3's timing).
- **Params.** Each is live, with a unit and a basis:
  - θ, `Dimensionless`, in (0, 1];
  - δ_p, `FractionPerYear`;
  - s, `Dimensionless`, bundles a plant unit; the generator writes
    s1 = θ^(θ/(1−θ))·(1 − θ)/δ_p (δ_p per tick);
  - s_Kp, `RatePerYear`, 2·δ_p.

  Their values: θ 0.8, δ_p 10% a year, s1 = 40.47205917167808 at 52 a year, s_Kp 0.2 a year. The
  basis is `Assumed("CAPACITY 2026-09-27; LOOP-SPEC 2026-09-29, mirror scan")`.
- **The bundle** is the desk's own recipe, bought at posted prices. The running bundle and the
  plant's share one buy line per good: coef·(z + s·I).
- **The cost ratios at posted prices.** Let c be the bundle's cost, summed in the desk's own
  order, and u = δ_p a tick. Then:
  - P_K = s·c;
  - kr = (1 − θ)·c/(θ·u·P_K), the plant units per bundle at the cost-minimising plant;
  - c_full = c + u·P_K·kr.

  At θ = 1, kr = 0 and c_full = c. For the bundle plant, kr = (1 − θ)/(θ·u·s) at any prices.
- **Output.**
  - The bundles received are B = max_scale over the bundle's goods held.
  - The running share is z_got = B·z/(z + s·I), and the build is I_got = B·I/(z + s·I). With no
    plant order (I = 0), z_got = B exactly.
  - Output is P^(1−θ)·z_got^θ, with P the plant held at decide. At θ = 1 it is z_got.
- **The order** is M3's: I = sign(u·K\*_p + s_Kp·(K\*_p − P)). It is at most sign(C − c·z)/P_K,
  the coin left after the running bundle. `sign` is a sign, not a cap (R3).
- **What it reads (R13).** Its own coin, plant, holdings and recipe; the posted prices; and θ, δ_p,
  s and s_Kp. It reads no volume, fill, other actor or oracle.
- **State.**
  - The plant held at decide is recorded for upkeep.
  - An existing state cannot gain a field without changing every committed tape's encoding. So a
    planted desk takes an appended `ActorState` variant, as P2.2a appended three.
  - A desk without a plant keeps its old variant, so every committed tape keeps its bytes, hashes
    and `world_id`.
- **Off.**
  - With `plant` absent, the kind's code runs as it is.
  - At θ = 1 the plant layer is the plant-free map bit for bit (§5), since kr = 0, c_full = c,
    there is no order, and output is z_got.

### 2.3 The fodder desk: a TypeDesk with a plant

- **Decide.**
  - c = (0.0 + 2·p_h) + 8·w + b·r, with b the fodder's land (1 × f).
  - The outlay comes from the cash rule (C2g: tilt 0). z = outlay/c_full; K\*_p = kr·z; then the
    order I.
  - It buys 2·(z + s·I) horse-days, 8·(z + s·I) labour and b·(z + s·I) land.
  - It offers every ton it holds, made last tick.
- **Produce.** y = P^(1−θ)·z_got^θ tons, and I_got plant units.
- **At rest (s1).**
  - θ·y bundles run the desk and (1 − θ)·y rebuild the plant: 1 bundle a ton in all.
  - The plant is κ_p·y = 2.44140625·y, 0.19058574 at LB1, and the price is c.

### 2.4 The capacity desk: M3 wet, with the plant beside the horses

The plant's bundle is the running recipe: 0.0176 fodder and 0.1 labour a horse-day. The horses stay
the capacity, κ·H horse-days, and keep their own user cost δ·p_K/κ. Horse-days are
y = min(κ·H, P^(1−θ)·z^θ).

- **Costs.**
  - O = (0.0 + 0.0176·p_f) + 0.1·w, as now; P_K = s·O; kr.
  - At the target ratio, the bundles and the plant per horse-day are ζ = kr^−(1−θ) and κ_p = kr^θ.
  - O_f = ζ·O + u·P_K·κ_p is the running bundle plus the plant's user cost, per horse-day. It
    equals O at s1 and at θ = 1.
- **Use** (M3's, at the running cost; sunk horses and plant compete at operating cost).
  z = min((κ·H/P^(1−θ))^(1/θ), B/O): the bundles that make κ·H on the plant held, or what the
  outlay B buys. At θ = 1 this is P2.2a's min(κ·H, B/O).
- **Horses** (M3's, at the full cost). K\* = B/(κ·O_f + δ·p_K); m = sign(δ·K\* + s_K·(K\* − H)),
  with s_K = 2δ.
- **Plant.** K\*_p = κ_p·κ·K\*, the plant for the herd the cash rule targets. The order I is at
  most sign(C − O·z)/P_K.
- **Cap.** The horse order is at most sign(C − O·z − P_K·I)/p_K.
- **Orders.** It buys 0.0176·(z + s·I) fodder, 0.1·(z + s·I) labour and m heads, and offers every
  horse-day it holds.
- **Produce.**
  - y comes from the phase-start horses and plant.
  - It burns the bundles it used, (y/P^(1−θ))^(1/θ) of z_got. The rest are one-tick goods and die
    at 5a.
  - It mints y horse-days and I_got plant units.
- **Upkeep.** δ of the horses held at decide; δ_p of the plant held at decide.
- **At rest (s1).** z = θ·κ·H, the plant is κ_p·κ·H (9.4041769 at LB1), and the hour price is
  O + δ·p_K/κ, the oracle's.

### 2.5 The maker: M2 from bought inputs, with a plant and the reservation

- c_m = (8·p_f + 20·w) + 3·b·r, as now; its markup is μ = p_K/c_m (a = 0).
- q = outlay/c_full bundles; K\*_p = kr·q; the order I is at most sign(C − c_m·q)/P_K.
- **Offer.** sign(F − b_K·q·c_full/p_K). The 4-week cover is read in ticks of outlay over price;
  at θ = 1 it is P2.2a's b_K·q·c_m/p_K. It offers none while μ < ψ (IDLE-SPEC §6).
- It buys every input on q + s·I.
- **Produce.** y = P^(1−θ)·z_got^θ heads, which join its finished stock.
- A plant on a maker with its own horse-days (a > 0) is refused at load: that case is untested.

### 2.6 Storable fodder (family LF4 only)

- Fodder lives a year.
- The fodder desk offers I/(1 + b) of its stock, with b = 13 ticks (M5's share rule). Unsold fodder
  stays in its stock.
- The capacity desk and the maker order their planned fodder net of what they hold (M6). They use
  what they hold first and keep what they do not use.
- The mirror carries no expiry; at rest a ton waits 14 weeks.
- Before LF4 can run, the engine needs the E1 seller and the running-good netting that STATE's O51
  names.

### 2.7 The like-for-like flow control (LW1–LW3)

The flow control is the same county, dials, assignment and plants, with the stocks layer off. It
is FUNDED's B-flow (decision 243).
- A TypeDesk makes horse-days from 0.020266 fodder, 0.106665 labour and 0.00099978 land, with a
  head's wear folded in at δ/κ. It carries a plant.
- There are no horses, no maker and no horse market. Fodder is as in B, with its plant.
- The point is B's within 9.3e-16 (FUNDED).
- The δ = 1 path cannot serve: M3 has no δ = 1 reduction (HORSES-SPEC §1.6), so the control is
  the flow desk.

## 3. How the plant composes with M3's horse holding

The question was whether M3's horse stock already plays the plant's role on its side of the loop.
It does not, for three reasons.

1. **The horses are a Leontief cap, not a curvature.** Horse-days are min(κ·H, bundle), and at
   rest both bind: the desk runs every horse. A fodder surplus leaves output at κ·H, but a fodder
   shortfall of x cuts horse-days by x.
   - So the horses damp an upswing until more heads arrive, and never a shortfall.
   - With no plant the capacity desk passes a fodder shortfall straight on: B-cut without plants
     (LN7) has 146 dead horse-day ticks after r × 2. P2.2a's H2 had 38, with fodder from land
     alone.
2. **Making the horses the smooth factor would move the technique with prices.** Suppose
   horse-days were (κ·H)^(1−θ_h)·z^θ_h. To rest at the oracle's Leontief point, θ_h must be the
   running cost's share, O/p_h: 0.8439 at LB1, 0.8111 at LB2, 0.9164 at LB3. Cost minimisation
   then sets the bundles per horse-day in proportion to (δ·p_K/κ)/O. That ratio moves with the
   targets (`lm_compose.out`):
   - ×1.0234 at b × 2 and ×0.9804 at b × 0.5 (LB1);
   - ×1.0246 and ×0.9791 (LB2);
   - ×1.0210 and ×0.9829 (LB3).

   So the long run would leave the registered oracle by 2% in log, as drs did (LOOPS.md). The cause
   is that a head is built by the maker from fodder, labour and pasture, not from the desk's own
   bundle, so its price moves against the bundle's. CAPACITY's plant keeps the ratio price-free
   because it is made of the desk's own bundle.
3. **The full-recipe reading does not compose with a stock.** FUNDED §8's second reading puts δ/κ
   heads into each bundle, which makes the horse a flow input. That is the flow control's form
   (§2.7), not M3's. A plant built of bundles that contain heads would buy heads twice: as the
   herd and inside the plant.

**So the plant sits beside the horses, on the running bundle** (§2.4). The horses stay the
capacity and keep their user cost δ·V. The plant takes (1 − θ)·O of each horse-day at rest: 16.9%
of the hour price at LB1 (FUNDED §8's "running recipe" rows).

The composition leaves M3's rules as they were:
- use at the running cost O;
- targets at the full cost, with O_f in place of O. O_f is O at s1, so the herd's target is M3's
  exactly at rest;
- the plant's target is the herd's plant, κ_p·κ·K\*.

The alternative target, CAPACITY's K\*_p = kr·z at the bundles in use (LF6), is far worse: its
largest root is 0.9996 a tick and most runs orbit (LF6 row). Measured side by side (§7.7), the
capacity desk's plant is what closes the loop's pass-through (LN3 against LN4).

## 4. The instances

`model/lm_inst.py` holds them. Every instance is chain8 under rule B: C2g, 52 ticks a year, ρ 0,
J_b 1, ex post, fodder's rate 5.2, ψ 0.25. Its plants are θ 0.8, δ_p 10% a year, s1, s_Kp = 2δ_p,
M3's order, J 2, and the herd target, unless its row says otherwise.

| id | horse δ a year | plants on | changed | role |
|---|---|---|---|---|
| LB1 | 8% | fodder, capacity, maker | — | verdict (CHAIN's δ) |
| LB2 | 10% | fodder, capacity, maker | — | verdict |
| LB3 | 4% | fodder, capacity, maker | — | verdict (the slow one; P8's δ) |
| LF1, LF7, LF8 | 8%, 10%, 4% | all three | θ 0.7 | family |
| LF2 | 8% | all three | δ_p 4% a year (s1 104.3926) | family |
| LF3 | 8% | all three | reservation off | sensitivity |
| LF4 | 8% | all three | storable fodder, 13-week cover | family (the pass-through damper candidate) |
| LF5 | 8% | all three | fodder's rate 1.3 (c2g13) | family |
| LF6 | 8% | all three | capacity plant's target kr·z (at use) | negative control |
| LC1 | 8% | all three | fodder without horse-days (B-cut) | the loop cut |
| LW1–LW3 | 8%, 10%, 4% | fodder, horse-day desk | stocks layer off (B-flow) | the like-for-like flow control |
| LW0 | 8% | none | B-flow without plants | the undamped flow loop |
| LN1 | 8% | none | — | negative control |
| LN2 | 8% | fodder, capacity | — | placement |
| LN3 | 8% | fodder | — | placement |
| LN4 | 8% | capacity, maker | — | placement |
| LN5 | 8% | all three | storable fodder, 4-week cover | negative control |
| LN6 | 8% | all three | whole gap each tick | negative control |
| LN7 | 8% | none | B-cut | the loop-free maker without plants |
| LN8, LN9 | 8% | none | fodder's rate 1.3; B-cut and B | could the dial replace the plant? |

The oracle point of every instance is FUNDED's at its horse δ and cost target. At s1 the plants
leave it unchanged.

## 5. Checks: the nestings and the rest point

`model/lm_nest.py`; output in `lm_nest.out`.

1. **Every new layer off is P2.2a's map bit for bit.** Take the loop cut (hf 0), no plant and no
   store on P2.2a's instances. The map is then `i_mirror.tick` bit for bit over 6,000 ticks of 13
   runs, including H1 and H2 heads × 10, P8 heads × 2, a 1e-9 kick and M1's P7. That holds with the
   reservation off and on, and at ψ 0 the map is also `h_mirror.tick`.
2. **At θ 1 the plant layer is the plant-free run bit for bit.** This covers rule B and B-cut,
   with plants at θ 1 on the fodder desk, the capacity desk, the maker, and every combination,
   against no plant. It holds for 8 displaced runs each, plus the flow control's five, over 6,000
   ticks: 69 runs, every state value equal.
3. **With a fixed plant it is fixed-Q drs.**
   - `lm_drs.py` is a second rule-B map, written from HORSES-RULES §3, the engine's TypeDesk and
     dmirror's drs, not from `lm_mirror`. In it, desks buy as their cash rule plans and make
     Q^(1−θ)·z^θ.
   - It equals `lm_mirror` with plant rule "none" and P = Q bit for bit, at θ 0.8 and 0.7, over
     6,000 ticks of 6 runs each.
   - At θ 1 it equals `lm_mirror` with no plant, which checks the loop's lines a second time.
4. **The rest point at s1 is the oracle's.**
   - Genesis maps to itself within 1.8e-15 relative at 150 points: 3 horse δ × 5 targets × 10
     configurations. The configurations are plants on the fodder and capacity desks at θ 0.8 and
     0.7 and plant δ 10% and 4%; all three desks; none; the flow control with and without plants;
     B-cut; and the 13-week store.
   - Over 2,000 ticks nothing drifts more than 8.5e-13 in log, or 3.7e-11 with the store (δ 4%,
     b × 0.5). The exception is the plant-free rule B, whose roots exceed 1.
   - The mirror's oracle is FUNDED's Rust `ChainEconomy` within 6.7e-16 relative on x\*, v, P_s,
     Y, N_a, and fodder's, a head's and a horse-day's prices, at 3 δ × 5 targets for B and B-cut.
     The good's price, a small number, is within 1.35e-14.
   - The plants are FUNDED's `PlantEconomy` readouts within 5.6e-16 (s1, K\*, P_K, uV, ζ) at the
     fodder desk and the capacity desk (running recipe).
   - Money is conserved within 1.1e-14 relative over 3,000 ticks of 6 displaced runs at every
     instance.
5. **The long run after a shock is the new target's oracle point.** Every converged run of every
   instance (2,901 of 3,794 runs) ends within 2.0e-13 in log of its target. So the constant-returns
   oracle needs no new solve at ρ 0 (decision 184).

**Why the rest point is the oracle's.** CAPACITY's argument (`capacity/cap.py`, `solve_cap`)
applies desk by desk:
- At rest the plant is at its target and the order replaces wear (I = u·K\*_p).
- The desk's coin rests where revenue meets outlay. With own use valued at the price, that gives
  p = ζ·c + u·κ_p·P_K, unit 1c's row p = O + uV, with the operating recipe ζ times the flow recipe
  and the build κ_p·s times it.
- For the bundle plant at s1, ζ + u·κ_p·s = 1: the desk uses exactly one flow recipe per unit, and
  the point is the flow oracle's.
- On the capacity desk, O_f = O at s1, so M3's rest (K\* = H, z = θ·κ·H, p_h = O + δ·p_K/κ) is
  P2.2a's with the running recipe folded the same way.
- The maker rests at μ = 1 > ψ, so the reservation is inactive at rest (IDLE-SPEC §7).
- At ρ 0 the clearing side is 1g's, with one solution (unit-1c §5.5; FUNDED §7).

## 6. The local map

`model/lm_lin.py`, `lm_kick.py`; outputs in `lm_lin.out`, `lm_kick.out`, `lm_openloop.out`.

**The largest root per tick (PL).** This is the growth of a 1e-8 displacement of the whole state,
over 100 years with 20 burned, from 4 directions.

| id | b × 1 | × 1.1 | × 0.9 | × 2 | × 0.5 | largest a year | frozen prices, a tick |
|---|---|---|---|---|---|---|---|
| LB1 | 0.996648 | 0.996648 | 0.996744 | 0.996956 | 0.997950 | 0.8988 | 1.0616 |
| LB2 | 0.996197 | 0.996150 | 0.996213 | 0.996947 | 0.997501 | 0.8780 | 1.0606 |
| LB3 | 0.997927 | 0.997934 | 0.998078 | 0.997952 | 0.999068 | 0.9527 | 1.0638 |
| LF1 | 0.997420 | 0.997349 | 0.997510 | 0.997463 | 0.998526 | 0.9262 | 1.0261 |
| LF2 | 0.998074 | 0.998105 | 0.998030 | 0.998484 | 0.997714 | 0.9241 | 1.0570 |
| LF4 | 0.996724 | 0.996706 | 0.996849 | 0.997006 | 0.998197 | 0.9104 | 1.0047 |
| LF5 | 0.996725 | 0.996673 | 0.996724 | 0.997026 | 0.997370 | 0.8720 | 1.0616 |
| LF7 | 0.997062 | 0.997086 | 0.997184 | 0.997378 | 0.998141 | 0.9078 | 1.0255 |
| LF8 | 0.998482 | 0.998398 | 0.998509 | 0.998213 | 0.999308 | 0.9647 | 1.0275 |
| LC1 | 0.996554 | 0.996545 | 0.996616 | 0.996807 | 0.997773 | 0.8905 | 1.0002 |
| LW1 | 0.995990 | 0.996138 | 0.995811 | 0.996963 | 0.995961 | 0.8537 | 1.0591 |
| LW2 | 0.996067 | 0.996273 | 0.995921 | 0.996986 | 0.995919 | 0.8548 | 1.0582 |
| LW3 | 0.995801 | 0.995956 | 0.995782 | 0.996871 | 0.996038 | 0.8496 | 1.0612 |
| LW0 | 0.996820 | 0.996834 | 0.996804 | 0.996909 | 0.996806 | 0.8513 | 1.1890 |

LF3 is LB1's to the bit. The reservation acts only where μ < ψ, and at rest μ = 1. The negative
controls' roots are in the §0 table.

**The kick set** (12 kicks: each price × (1 ± 1e-9), H = 30,000). The fitted decay a year, at the
base and its slowest over the five targets:

| | LB1 | LB2 | LB3 | LF1 | LF2 | LF4 | LF5 | LF7 | LF8 | LC1 | LW1 | LW2 | LW3 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| base | 0.8474 | 0.8295 | 0.9011 | 0.8787 | 0.9014 | 0.8525 | 0.8478 | 0.8648 | 0.9227 | 0.8435 | 0.8355 | 0.8375 | 0.8312 |
| slowest | 0.8969 | 0.8753 | 0.9479 | 0.9217 | 0.9252 | 0.9056 | 0.8683 | 0.9037 | 0.9646 | 0.8892 | 0.8576 | 0.8595 | 0.8540 |

The envelope peaks at 4.0–7.9 in the loop instances and 2.5–7.3 in the flow control. Its tail at
30,000 ticks is 5e-5 to 4.1e-4, on the rounding floor.

**The one-tick elasticity probe** (the engine's `horses elasticity`, mirrored). τ is in ticks:

| id | good | land | fodder | horse-days | labour | heads | L = max(20,000, 200·τ, 3·T6) |
|---|---|---|---|---|---|---|---|
| LB1 | 1,052.2 | 276.2 | 19.3 | 13.8 | 7.1 | 2.2 | 211,000 (3·T6 21,000) |
| LB2 | 1,006.0 | 276.4 | 19.2 | 14.1 | 7.0 | 2.2 | 202,000 |
| LB3 | 1,155.6 | 275.7 | 19.2 | 13.2 | 7.1 | 2.4 | 232,000 (3·T6 45,000) |
| LC1 | 1,100.9 | 275.9 | 19.4 | 13.1 | 7.1 | 2.2 | 221,000 |
| LW1–LW3 | 1,006–1,156 | 276 | 8.1–8.3 | 13.2–14.1 | 7.0 | — | 202,000–232,000 |

**The good's market sets L, not land's.** In chain8 the good is 1.6% of a basket (P_s 1.016, p
0.016). A 1% move in p moves demand by 0.016%, so at 2.6 a year its τ is about 1,000 ticks. That
is 2.7 times P2.2a's L, and it is FUNDED's O51, the space-heavy county.

**Tick length** (PL at the base; Tiers 1–2 and mode A in `lm_extra.out`):

| | 12 a year | 24 a year | 52 a year | 365 a year |
|---|---|---|---|---|
| LB1 | **1.005298 a tick; Tiers 1–2 0/40** | 0.8464 a year; 40/40, median 27.5 y | 0.8398 a year | 0.8355 a year; 40/40, median 24.8 y |
| LB3 | **1.017005 a tick** | 0.9032 a year | 0.8977 a year | 0.8935 a year |
| LW1 | 0.8034 a year; 36/36 | 0.8099; 36/36 | 0.8537 | 0.8234; 36/36 |

**The frozen-price and open-loop probes.**
- With every price held, LB1's quantities and coins grow 1.0616 a tick. That is 1.19 without
  plants (the L2-shaped bullwhip; LOOPS.md), 1.005 with the 13-week store, and 1.0002 with the loop
  cut.
- The open-loop probe (every rate 0, each price × 1.01) reads the loop on the fodder market. The
  largest |ln(D/S)| per unit of a price's log, at lag 200, is:
  - 1,665 at LB1 and 1,726 at LB3 (per unit of the good's price);
  - 101 with the store (LF4);
  - 1,620 in the flow control;
  - 64,570 without plants (LN1);
  - 36 with the loop cut (LC1).
- The horse market's demand stops (ln(D/S) → −∞) by lag 200, since the order integrates the stock
  gap, as in P2.2a.
- As CAPACITY said, the damping needs live prices.

## 7. The battery

`model/lm_battery.py` runs the P2.2a battery (HORSES-RULES §5): mode A; Tiers 1–3; Tier 3S (every
stock and coin × 0.5 and × 2); Tier 3 and 3S again at 10·L; and the stocks family (every coin
× 0.02 and × 0.1, every stock × 0.1 and × 10). Results are in `lm_battery_<id>.json` and
`lm_battery.out`; the tables are `lm_summary.py` → `lm_summary.out`. 10·L gives the same class
and the same ticks to tolerance in every run.

### 7.1 Speeds, troughs and dead ticks

"Years to tol" is the median and the largest over converged runs. "Dead" is the largest count of
dead ticks in the tier, split into labour (L), fodder (Fo), horse-days (H) and the good (G). "Horse
price" is the horse's lowest price over its target in the tier. The reservation acted in 23 of
LB1's 97 battery runs, 20 of LB2's and 32 of LB3's.

| id | Tier 1: years to tol; lowest baskets | Tier 2 | Tier 3 | Tier 3S |
|---|---|---|---|---|
| LB1 | 22.2 / 35.1; 0.895; dead 0; horse 0.717 | 30.1 / 43.0; 0.656; dead 5 (Fo 5); 0.231 | 37.0 / 64.6; 0.255; dead 242 (L 168, Fo 175, H 50, G 21); 0.148 | 32.7 / 46.6; 0.500; dead 35 (H 35); 0.218 |
| LB2 | 17.9 / 33.5; 0.897; 0; 0.759 | 27.8 / 38.0; 0.660; 5; 0.223 | 32.1 / 54.0; 0.260; 321 (L 176, Fo 261, H 50, G 20); 0.152 | 30.2 / 43.4; 0.500; 33; 0.214 |
| LB3 | 34.7 / 58.3; 0.891; 0; 0.588 | 48.8 / 73.2; 0.650; 5; 0.203 | 60.7 / 135.4; 0.245; 230 (L 148, Fo 176, H 51, G 23); 0.148 | 57.6 / 78.6; 0.500; 54; 0.178 |
| LF1 | 20.8 / 44.4; 0.923; 0; 0.749 | 34.0 / 54.5; 0.733; 0; 0.223 | 41.3 / 90.5; 0.373; 315 (L 237, Fo 215, H 26); 0.130 | 37.5 / 60.3; 0.500; 31; 0.215 |
| LF2 | 19.1 / 50.0; 0.895; 0; 0.723 | 36.5 / 63.0; 0.657; 5; 0.220 | 46.8 / 79.7; 0.256; 240; 0.147 | 44.5 / 71.3; 0.500; 34; 0.216 |
| LF4 | 19.1 / 36.9; 0.934; 0; 0.774 | 29.8 / 44.2; 0.755; 0; 0.288 | 35.3 / 74.8; 0.411; 262 (L 176, Fo 156, H 25); 0.130 | 32.0 / 47.9; 0.500; 23; 0.170 |
| LF5 | 22.5 / 34.3; 0.870; 0; 0.666 | 31.7 / 44.3; 0.611; 12; 0.245 | 36.3 / 51.6; 0.222; 277 (Fo 251, H 79); 0.164 | 32.7 / 48.6; 0.500; 78; 0.188 |
| LF7 | 19.2 / 38.9; 0.925; 0; 0.782 | 29.5 / 50.0; 0.735; 0; 0.269 | 37.2 / 68.0; 0.375; 306; 0.131 | 35.2 / 55.0; 0.500; 30; 0.208 |
| LF8 | 38.8 / 70.3; 0.920; 0; 0.650 | 58.8 / 88.5; 0.730; 0; 0.224 | 72.5 / 204.5; 0.369; 298; 0.131 | 64.7 / 102.3; 0.500; 49; 0.170 |
| LC1 | 19.4 / 34.0; 0.934; 0; 0.712 | 28.8 / 41.5; 0.753; 0; 0.215 | 34.7 / 61.4; 0.411; 234 (L 163, Fo 170, H 28); 0.152 | 29.6 / 45.1; 0.500; 26; 0.186 |
| LW1 | 14.5 / 33.8; 0.897; 0 | 21.4 / 41.3; 0.662; 6 | 28.0 / 47.9; 0.262; 215 (L 115, Fo 203, H 65, G 24) | 28.9 / 45.9; 0.500; 10 |
| LW2 | 14.6 / 34.2; 0.899; 0 | 21.7 / 41.7; 0.665; 6 | 28.5 / 48.4; 0.268; 213 | 29.3 / 46.4; 0.500; 10 |
| LW3 | 14.2 / 33.1; 0.893; 0 | 20.7 / 40.4; 0.657; 4 | 27.0 / 47.1; 0.254; 220 | 28.0 / 45.0; 0.500; 10 |

- **Tier 3's worst dead count is N(2)** (242 at LB1): the provider's transfer is short while rent
  catches up (FUNDED §6).
- **Tier 3S's lowest baskets (0.500)** come from the good's stock × 0.5.
- **In the stocks family:**
  - the workers' coin × 0.02 gives the most dead ticks (535 at LB1, 432 at LB3, 430 in the flow
    control);
  - the good desk's coin × 0.02 gives the lowest baskets (0.02 of Y\* for a tick, as in P2.2a);
  - heads × 10 gives the largest peak D̂, 79,534 at LB1, because the maker's output falls toward
    zero while it withholds.

### 7.2 The cost shocks and capital's time (D-G14)

At genesis. "Trough" is the goods' lowest over the new Y\*. The 5% columns give the year from which
each stock stays within 5% of its target.

| id | b × 2: trough; years to tol; heads, capacity plant, fodder plant to 5%; horse price low | b × 0.5: the same |
|---|---|---|
| LB1 | 0.702; 46.3 y; 10.1, 9.4, 8.6 y; 0.194 | 0.859; 64.6 y; 14.4, 14.2, 13.7 y; 0.704 |
| LB2 | 0.714; 43.4 y; 10.6, 11.2, 9.1 y; 0.196 | 0.863; 54.0 y; 14.1, 14.1, 13.8 y; 0.791 |
| LB3 | 0.677; 66.6 y; 12.5, 24.3, 8.3 y; 0.196 | 0.850; 135.4 y; 24.0, 34.8, 31.7 y; 0.390 |
| LF1 (θ 0.7) | 0.822; 52.5 y; 13.0, 12.0, 9.1 y | 0.853; 90.5 y; 17.6, 17.3, 23.4 y |
| LF2 (δ_p 4%) | 0.703; 76.3 y; 26.5, 34.4, 24.2 y | 0.857; 48.2 y; 5.7, 9.8, 18.4 y |
| LF4 (store) | **0.964**; 46.8 y; 9.4, 8.6, 8.0 y | 0.859; 74.8 y; 14.6, 14.3, 18.9 y |
| LC1 (loop cut) | 0.799; 45.3 y; 9.8, 9.0, 8.1 y | 0.867; 61.4 y; 13.7, 13.5, 12.9 y |
| LW1 (flow) | 0.675; 38.8 y; —, 12.4, 8.7 y | 0.865; 31.5 y; —, 4.2, 12.6 y |
| LW2 | 0.684; 39.2 y; —, 12.8, 9.1 y | 0.871; 31.9 y; —, 4.4, 12.8 y |
| LW3 | 0.658; 37.8 y; —, 11.5, 8.1 y | 0.854; 31.0 y; —, 3.6, 12.1 y |

After b × 2 the installed heads run from 0.980 to 1.672 of their new target at LB1. After b × 0.5
they run from 0.707 to 1.137. P2.2a's heads came within 5% after b × 2 in 29–47 years; here it
takes 10–13, and the paper's zero builds take 3–8 (O46; D-G14's departure stands).

### 7.3 The glut, the idle horse market and P8-like

From `lm_extra.out` and the stocks family. The same numbers hold at L and at 10·L.

| id, ψ | heads × 2 (P8-like at LB3) | heads × 10 |
|---|---|---|
| LB1, 0.25 | CONVERGED; 45.4 y; horse price 0.218–2.16 of target; withheld 303 ticks; dead 0 | CONVERGED; 65.3 y; 0.219–3.76; withheld 1,315; dead 0; lowest goods 0.886 |
| LB2, 0.25 | CONVERGED; 39.8 y; 0.214–2.02; 221; dead 0 | CONVERGED; 58.0 y; 0.219–3.29; 1,034; dead 0; 0.874 |
| LB3, 0.25 | CONVERGED; 77.6 y; 0.221–2.24; 662; dead 0 | CONVERGED; 118.2 y; 0.221–3.56; 2,694; dead 0; 0.979 |
| LB1, LB2, LB3 at ψ 0 | **DIVERGED**: the horse price crosses 1e-6 at ticks 156, 158, 155 | **DIVERGED** at 156, 156, 155 |

- **The reservation's floor holds.** With it on, the horse price's lowest over its target is at
  least 0.130 in every loop instance and tier (§7.1; 0.130 in Tier 3 at LF1 and LF4). The price
  can fall only one step below ψ times the replacement cost it sees (IDLE-SPEC §6), and that cost
  moves with fodder and the wage.
- **The shortage O51 is absent.** With it off (LF3), five runs diverge. With it on, heads × 10
  leaves no dead tick. IDLE-SPEC's O51 (the glut ending in a shortage) does not appear here, since
  rule B's maker has no herd of its own to wear out.

### 7.4 heads × 0.1, the shortage side

At LB1, heads × 0.1 converges in 46.1 years. It has 194 dead ticks: 193 on horse-days, 89 on
fodder and 30 on the good. The tasks' horse-days fall to 0.033 of target. This is the loop's
absorbing zero approached and left: without horses there are no horse-days, and without horse-days
there is no fodder. The plants hold the desks' coin, and the herd is rebuilt. LB3 shows the same,
with 260 dead ticks in 78.3 years.

### 7.5 The pass-through (r × 2 at genesis)

Dead ticks by market, and the lowest volumes over their targets:

| id | fodder | horse-days | good | labour | fodder low | tasks' horse-days low | class |
|---|---|---|---|---|---|---|---|
| P2.2a H2 (engine; rule A) | 120 | 38 | 21 | — | — | — | CONVERGED |
| **LB1** | 175 | **2** | 11 | 168 | 0.376 | 0.497 | CONVERGED, 50.3 y |
| LB2 | 261 | 9 | 10 | 176 | 0.379 | 0.486 | CONVERGED |
| LB3 | 176 | 2 | 12 | 148 | 0.371 | 0.487 | CONVERGED |
| LF4 (13-week store) | 156 | **0** | 11 | 170 | 0.375 | 0.577 | CONVERGED |
| LC1 (loop cut, plants) | 170 | 1 | 11 | 163 | 0.387 | 0.480 | CONVERGED |
| LN4 (no fodder plant) | 299 | 57 | 38 | 0 | 0.160 | 0.289 | CONVERGED |
| LN7 (loop cut, no plant) | 164 | **146** | 91 | 0 | 0.242 | 0.279 | ORBITING |
| LN8 (loop cut, no plant, fodder 1.3) | 199 | **150** | 78 | 0 | 0.219 | 0.185 | CONVERGED |
| LW1 (flow control) | 203 | 3 | 11 | 115 | 0.372 | 0.481 | CONVERGED |
| LN1 (no plant) | DIVERGED at tick 287 | | | | | | |

- **The plant alone damps the pass-through into horse-days.** The capacity desk's plant takes it
  from 146–150 dead ticks to 1–2, and the fodder desk's plant halves what is left (57 → 2).
- **The fodder dead ticks are not the loop's.** They are 156–261 wherever the plants sit, and 203
  in the flow control. After r × 2 the fodder desk's land doubles in price (81% of its cost). Land's
  price comes back at 0.1625 a year, some 200 ticks at the full imbalance. And the county's labour
  supply opens at 0.508 of target (FUNDED §6: z/χ_max 0.2245 against 0.443).
- **Labour's dead ticks are a knife-edge count.** Supply sits 0.008 above the dead bar, so the count
  runs from 0 to 237 across instances.
- **Storable fodder is not needed for the pass-through.** At a 13-week cover it trims fodder's dead
  ticks (175 → 156) and lifts the tasks' horse-days (0.497 → 0.577). Its large gains are elsewhere:
  - the b × 2 trough, 0.70 → 0.96;
  - Tier 3's floor, 0.255 → 0.411;
  - the frozen-price growth, 1.06 → 1.005.

  It costs a new seller and buyer netting (O51 in STATE), and a 4-week cover is unstable (LN5).
  So it is a family, not the default (decision L5).

### 7.6 O14 like for like (decision 235)

These are HORSES §4's columns. The trough is the goods' lowest after b × 2, in ln of the old Y. The
T3 peak D̂ leaves out the horse volume.

| set | b × 2 trough, ln (of old Y) | T2 dead | T3 baskets / Y\* (median / lowest) | T3 dead | T3 peak D̂ | T1–2 years (median / worst) |
|---|---|---|---|---|---|---|
| flow control, no plants (LW0) | −2.363 (0.09), DIVERGED | 225 | 0.435 / 0.059 | 221 | 3,029 | 20.4 / 39.2 |
| **flow control, plants (LW1)** | **−0.394 (0.67)** | **6** | **0.675 / 0.262** | **215** | **2,587** | **19.5 / 41.3** |
| LW2 / LW3 | −0.381 / −0.421 | 6 / 4 | 0.684 / 0.268; 0.658 / 0.254 | 213 / 220 | 2,604 / 2,552 | 19.5 / 41.7; 19.3 / 40.4 |
| **LB1** (stocks, plants) | **−0.356 (0.70)** | **5** | **0.702 / 0.255** | **242** | **12,183** | **26.7 / 43.0** |
| LB2 / LB3 | −0.339 / −0.392 | 5 / 5 | 0.708 / 0.260; 0.677 / 0.245 | 321 / 230 | 9,429 / 20,703 | 24.1 / 38.0; 43.0 / 73.2 |
| LF1 (θ 0.7) | −0.197 (0.82) | 0 | 0.737 / 0.373 | 315 | 9,698 | 29.5 / 54.5 |
| LF4 (store) | −0.039 (0.96) | 0 | 0.781 / 0.411 | 262 | 11,795 | 23.4 / 44.2 |
| LC1 (loop cut) | −0.226 (0.80) | 0 | 0.757 / 0.411 | 234 | 11,904 | 25.2 / 41.5 |

- **Like for like, stocks add little to the loop's damping.** They lift the b × 2 trough by 0.038
  in log at δ 8% (0.042 at 10%, 0.029 at 4%). They cost 1.37 times the flow control's median years
  at δ 8%, 1.24 at 10% and 2.23 at 4%.
- **Their dead ticks are about the same.** The labour count is a knife edge.
- **The larger peak D̂ is the maker's output.** It swings toward zero while the maker withholds,
  and the flow control has no maker.
- **What carries the loop is the plant.** Plants take the flow control's trough from 0.09 to 0.67.
  The shortfall (T3, from N(2)) is 1,100–1,190 in every instance, set by the county.

### 7.7 The negative controls and placements, restated

- **No plant (LN1).** Locally unstable, 1.0147. The battery gives 0/91 and 1/22 in the stocks
  family: runs die or diverge, mostly on horse-days (up to 2,727 dead ticks).
- **The fodder desk's plant alone (LN3).** 1.0062; every run DEAD.
- **Capacity and maker plants, no fodder plant (LN4).** GO in the tiers. Its trough after b × 2 is
  0.505 (LB1's 0.702), its Tier-3 floor 0.163, and the fodder desk's coin × 0.02 diverges (the
  absorbing zero).
- **Fodder and capacity plants, no maker plant (LN2).** Locally stable (0.9973), but under the
  reservation 16 of 25 Tier-3 runs orbit: the relay cycle of §0 item 4.
- **The capacity plant's target at the bundles in use (LF6).** 0.9996; 22 of 97 converge.
- **The rest.** A 4-week store (LN5) has 1.0032 and every run orbits. Whole-gap plant orders (LN6)
  have 11.6 a tick, and even mode A dies. B-cut without plants (LN7) has 1.0104, 1/91. Fodder at
  1.3 without plants (LN8, LN9) is locally stable (0.9963) but fails the battery (68/91 and 41/91
  in Tiers 1–3S).

## 8. Registered predictions for P2.2b's engine run

Registered before any P2.2b engine code or run exists. The conditions are those of the header and
§2. The engine is expected to match the mirror as P2.2a's did (HORSES §2):
- every class exactly;
- ticks and years within 10%, with three runs per instance allowed within 25%;
- lowest prices and troughs within 5% of the value given;
- dead-tick counts within 10% or 5 ticks, whichever is larger, except the knife-edge labour counts
  of §7.5 and O-L7.

**E0. Before any mode-B run: a trace diff.** `lm_mirror.tick` against the engine for 2,000 ticks
at LB1 on hold, w × 2, r × 2, b × 2 at genesis, heads × 2 and heads × 10, and at LW1 on hold and
b × 2. The mirror needs the engine's genesis carry added, as P2.2a's `h_carry.py` did. They must
agree within 1e-12. The allowed parting is the order's cancellation (HORSES-RULES §6.4).

**E1. Nesting (R1).**
- With every plant and the loop absent, every committed tape keeps its text, `tape_hash`,
  `world_id` and per-tick hash stream. The gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
  demo-gb `0xfad880fe08d06645` and the probe, markets and horses pins do not move.
- A planted tape at θ 1 equals the same tape without plants, value for value, for 2,000 ticks at
  LB1 on hold, w × 2 and r × 2.
- A fixed plant (no wear, no order) equals fixed-Q drs, with Q the plant held.

**E2. The rest point and mode A.**
- At LB1–LB3 and every cost target, genesis from 1g's `ChainEconomy` with the plants at κ_p·X is a
  fixed point within 1e-12.
- Mode A passes at L: largest gaps of 6.7e-16, 1.0e-15 and 3.7e-14 in the mirror, and below 1e-9
  in the engine.

**E3. The verdict instances.**

| id | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | stocks family | L (the mirror's rule) |
|---|---|---|---|---|---|---|
| LB1 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 211,000 |
| LB2 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 202,000 |
| LB3 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 232,000 |

Every run is CONVERGED, and so every instance is GO. The speeds, troughs, dead ticks and lowest
horse prices per tier are §7.1's. The engine's own elasticity probe sets its L; the mirror's τ are
§6's (the good's 1,006–1,156 ticks).

**E4. The kick sets and the slowest mode.**
- Every kick set decays.
- The engine's fitted g at the base is within 0.03 a year of the mirror's kick g: 0.847 (LB1), 0.830
  (LB2), 0.901 (LB3), 0.836 (LW1).
- The mirror's largest root per tick over the targets is 0.997950, 0.997501 and 0.999068.

**E5. The cost shocks.** §7.2's rows: troughs, years to tolerance, and the years for the heads and
each plant to come within 5%, at LB1–LB3 and LW1–LW3.

**E6. The glut.** heads × 10 and heads × 2 converge at LB1–LB3 with no dead tick. The lowest horse
price is 0.214–0.221 of target, the highest 2.02–3.76, and the withheld ticks are §7.3's.

**E7. The pass-through.** After r × 2 at LB1 there are at most 5 dead horse-day ticks (mirror: 2),
fodder 175 and the good 11. B-cut without plants (LN7) has at least 100 horse-day ticks (mirror:
146) and ORBITS.

**E8. The flow control.** LW1–LW3 are GO, with §7.1's, §7.2's and §7.6's numbers. The O14
comparison is read as §7.6 reads it: a b × 2 trough 0.038 higher in log with stocks, and 1.37
times the years at δ 8%.

**E9. Negative controls** (each at L):
- LN1 and LN7 fail every tier (the roots 1.0147 and 1.0104);
- LN2 orbits in Tier 3 (at least 10 of 25);
- LN3 is DEAD throughout;
- LF3 diverges on r × 2, heads × 2, heads × 10 and the workers' coin × 0.02 and × 0.1, the horse
  price crossing 1e-6 at ticks 156–173; at ψ 0, heads × 2 and × 10 cross it at ticks 155–158 at
  LB1–LB3;
- LB1 at 12 ticks a year fails Tiers 1–2.

**E10. Families, if built.** LF1, LF2, LF5, LF7, LF8 and LC1 are GO with §7's numbers. LF4 needs
O51's roles and is GO in the mirror.

**What would refute the design and send it back to the mirror:**
- a class change at LB1–LB3;
- a runaway of the horse price at LB1–LB3 with ψ 0.25;
- a converged run ending more than 1e-12 off its target's oracle point;
- a θ = 1 tape differing from the plant-free tape;
- the engine's slowest mode above 1 at any target.

## 9. Decisions proposed and open items

Each decision is open to veto; the alternative named is the registered one (R6). They are
numbered L1–L12 here. STATE.md's merge numbers them after 244, and after idle-scan's and funded's
proposals, which also start at 240.

- **L1. The plant sits on every loop desk: the fodder desk, the capacity desk and the maker.** θ is
  0.8 and the plant's δ 10% a year. Each plant is s1 bundles of its desk's own recipe, ordered by
  M3's rule at s_K = 2δ_p, J 2. *Alternative:* the fodder and capacity desks only (LN2). It is
  locally stable, but under the reservation 16 of 25 Tier-3 runs orbit.
- **L2. On the capacity desk the plant's bundle is the running recipe, beside the horses.**
  Horse-days are min(κ·H, P^(1−θ)·z^θ). The horses stay the capacity with their own user cost, use
  is M3's at the running cost, and targets are M3's at the full cost O_f (§2.4, §3). *Alternative:*
  FUNDED's full-recipe reading, which composes only with the flow form.
- **L3. The capacity plant's target is the herd's, κ_p·κ·K\*.** *Alternative:* CAPACITY's kr·z at
  the bundles in use (LF6): 0.9996 a tick and 22 of 97 converge.
- **L4. M3's horse stock is not the plant** (§3). It is a Leontief cap, and as a smooth factor it
  would move the technique 2% at b × 2. *Alternative:* the horses as a Cobb-Douglas factor at
  θ_h = O/p_h, with a scale-dependent oracle.
- **L5. No damper beyond the plant for the pass-through.** Storable fodder at a 13-week cover (LF4)
  is a registered family. A 4-week cover is a negative control. *Alternative:* the 13-week store in
  the default, which lifts the b × 2 trough from 0.70 to 0.96 but needs O51's roles first.
- **L6. The reservation (ψ 0.25) stays on in every loop run.** With it off, gluts run away at
  ticks 155–173 (LF3, and heads × 2 and × 10 at LB1–LB3). *Alternative:* the world's `Hold` rule (IDLE-SPEC's R6), not run here.
- **L7. The instances.** LB1–LB3 (horse δ 8%, 10%, 4%) carry the verdict. The families are §4's;
  the flow controls LW1–LW3 are O14's like-for-like reference; LN1–LN9 and LF6 are the negative
  controls. *Alternative:* LB1 alone as the verdict instance, with δ as families.
- **L8. L follows HORSES-RULES §6.7's rule.** The good's τ of about 1,000 ticks sets it:
  202,000–232,000 ticks, because chain8's good is 1.6% of a basket. *Alternative:* L from land's
  τ (56,000 ticks), which the mirror's runs would have passed too (the longest to tolerance is
  10,636 ticks at LF8).
- **L9. Weekly ticks stay the floor for the loop** (decision 237). At 12 a year LB1 and LB3 are
  unstable (1.0053 and 1.0170 a tick). *Alternative:* 24 a year as the floor, which passes 40/40.
- **L10. CHAIN's maker from bought fodder is the tested "maker from bought inputs."** S1 without
  pumping waits for a funded steam county (O-L3). *Alternative:* frame S1 on M3's unfunded county
  with its transfer shortfall reported.
- **L11. Fodder's rate stays at C2g's 5.2.** 1.3 (LF5) is a family. Without plants it makes the map
  locally stable, but the battery still fails (LN8, LN9). *Alternative:* 1.3 as the default for
  rule B.
- **L12. The O14 reference for P2.2b is the flow control with the same plants (LW1–LW3).** *Alternative:*
  the flow control without plants (LW0), which is NO-GO and so no reference.

Open items (O-L; STATE numbers them after O51):

- **O-L1. The loop still shows under frozen prices.** LB1 grows 1.0616 a tick with every price
  held (1.19 without plants). The engine's open-loop probe will read the fodder market at order
  10^3 per unit of log price at lag 200. The plant's damping needs live prices (CAPACITY).
- **O-L2. The absorbing zero is kept.** heads × 0.1 takes the tasks' horse-days to 0.033 of target.
  Without the fodder plant, the fodder desk's coin × 0.02 does not recover (LN4).
- **O-L3. S1 without its pumping loop is not framed.** M3's county is unfunded at S1's point:
  P_s 4.749, N·P_s 19.0 against T 10. It needs its own funded county, found and checked by the
  oracle as FUNDED did for the horse.
- **O-L4. Why the maker's plant removes the reservation's relay cycle.** This is shown, not
  derived. Without it, a 1e-8 kick decays (0.9973) and large displacements orbit. A describing-function
  argument, or a scan of ψ with and without the maker's plant, would settle it.
- **O-L5. The loop-cut R1 check of GOODS-CHAIN's stage table.** "Fodder without horse-days is
  v2a.1" does not hold on rule B's numbers. B-cut is CHAIN's maker on v2a.1's roles, and without
  plants it is NO-GO (LN7). With the design it is GO (LC1). The stage table's R1 row for v2a.1b
  should read "fodder without horse-days is LC1".
- **O-L6. The engine build.** It needs:
  - the plant on three kinds, as an optional `plant` field with appended `ActorState` variants;
  - the TypeDesk's `traction` input (no code: P2.1's inputs list);
  - the maker's fodder build input (no code);
  - `probe::horses`' rule-B instances from FUNDED's instances.json;
  - the harness's plant observables and 5% readouts;
  - a trace diff first (E0).
- **O-L7. The knife-edge labour counts.** After r × 2 chain8's labour supply opens at 0.508 of
  target (z/χ_max 0.2245 against 0.443). After N(2) the desks' coin buys half the labour. Both
  sit within 2% of the dead bar, so labour's dead ticks flip between 0 and about 240 on small
  differences. Scoring should read them with that in mind, or read labour's dead bar at 0.45 on
  this county.

## 10. Files, and how to rerun

Under `D:/rustyecon-p2l/loop-mirror/`:
- `model/lm_mirror.py`: the loop map (LOOP, PLANT, STORE and FLOW layers).
- `model/lm_run.py`: the scoring (h_run's and i_run's, with the plants' observables and the kick
  set).
- `model/lm_inst.py`: the instances.
- `model/lm_drs.py`: the independent drs map.
- `model/lm_nest.py`: the checks of §5.
- `model/lm_lin.py`: roots, the frozen probe, τ and L.
- `model/lm_kick.py`: kick sets and the open-loop probe.
- `model/lm_battery.py`: the battery.
- `model/lm_extra.py`: P8-like and tick lengths.
- `model/lm_summary.py`: the tables.
- `model/lm_compose.py`: §3's numbers.
- `model/lm_explore.py`, `lm_diag.py`, `lm_sample.py`: the first look, the diagnosis of the
  plantless instability, and the placement samples (`lm_sample1.json`, `lm_sample2.json`).
- Outputs: `model/lm_*.out` and `model/lm_*.json`. The battery's runs are
  `model/lm_battery_<id>.json`.
- Copies, unedited:
  - `model/h_*.py`, `s_*.py`, `_path.py` and `h_lin.json` from P2.2a's frame, checked against
    `model/SHA256SUMS.frame`;
  - `model/i_mirror.py` and `i_run.py` from idle-scan;
  - `ag/*.py`, the agents' pass;
  - `capacity/*.py`, CAPACITY's model with its `SHA256SUMS`;
  - `model/funded_instances.json`.
- `sync.sh` copies the model to WSL-local scratch (`/root/scratch/loop-mirror`) and back.
  `launch.sh OUT SCRIPT ARGS` runs a script detached there. `sums.sh` writes `SHA256SUMS`.

Rerun in WSL (python3 with numpy):
1. `bash sync.sh`
2. In `/root/scratch/loop-mirror/model`:
   - `python3 lm_nest.py > lm_nest.out`
   - `python3 lm_lin.py > lm_lin.out`
   - `python3 lm_battery.py LB1 LB2 LB3 LF1 … LN9 > lm_battery.out` (about 95 minutes on 47 cores)
   - `python3 lm_kick.py`, `lm_extra.py` and `lm_summary.py`
3. `bash sync.sh back`

`SHA256SUMS` lists this file first, then every file it rests on.
