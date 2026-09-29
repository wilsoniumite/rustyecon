# The loop step's registration (P2.2b's frame inputs)

Dated 2026-09-29. Step L0.8 on branch `phase2-loops`. It registers the mirror's predictions for
P2.2b's engine run before any P2.2b engine code or run exists: at this commit `crates/agents`,
`crates/probe` and `crates/engine` hold no plant code, and `tapes/` holds no rule-B tape. It is
written by `D:/rustyecon-p2l/fix-report/make_registration.py`, which copies every quote below from
its file.

## 1. What is registered

The frame inputs, in [docs/probe/loops/](../../loops/), byte for byte as their authors left them:

| file | sha256 | what it is |
|---|---|---|
| `LOOP-SPEC.md` | `e16e0be2ab521ed28c5d79811fabbb250ca5bbb107bd00f656d45d0993fb89ad` | the mirror's loop step (GOODS-CHAIN §6 step 3), registered 2026-09-29 in `D:/rustyecon-p2l/loop-mirror/` |
| `LOOP-SPEC-A1.md` | `8815d8652f6356423d02e06e2e5be814397f86e923688a78e1ef4d55afda4f2b` | its amendment after the mirror review: the engine's genesis carry, and §8 registered again (its §A1.5) |
| `FUNDED.md` | `ab86a2e419687fc5a33991120dcbcd8cf2be45909a07510726ee5d98025346cc` | the funded rule-B county, chain8 (O41), 2026-09-29, `D:/rustyecon-p2l/funded/` |
| `FUNDED-A1.md` | `45a23bf7d3633e0aa1571b228bba867286242125794d1eff54544df0bff49eb5` | its amendment after the mirror review |
| `instances.json` | `cb722dcb3f3de0d4b1914c62165cd3d00bf5b4cc00416a5d5552166e5e9952b2` | FUNDED's machine-readable copy: the county, the chain, the tape params, every point |

**LOOP-SPEC-A1 §A1.5 replaces LOOP-SPEC §8 for scoring.** §8 as first registered is kept in the
appendix below; nothing in it is scored where A1 differs. Each amendment is dated and carries its
own sha256; the registered files are unedited (decision 282).

The mirror the predictions come from (`D:/rustyecon-p2l/fix-report/loop-carry/model/`, a copy of
`D:/rustyecon-p2l/loop-mirror/model/` with the carry added; sha256):

```
8ae28374ef6a9243b03439bca4335e7280452387ef097c5d7c25c9851e1eb728  lm_mirror.py      (the registered map, unedited)
e691dfbe1bc1e26ca9261470ed9882519c072478c4819d0aec5364c7d16ae2af  make_lm_carry.py  (writes lm_carry.py from it)
f94c33e17b4130c6af2717c6fd339926fb2b8b2155748a001bd45dcc0a868a2e  lm_carry.py       (the map with the engine's genesis carry; E0's mirror)
094558322ab5bd6a73f18a2b5d51e09b34e3da9636087e0464cae79dbfa550be  lm_run.py         (the scoring, unedited)
7888b1c927973e6954d7bcfd871be4d7719916bd95ee3df148a60007049429c9  lm_inst.py        (the instances, unedited)
```

## 2. The order of work

1. FUNDED and LOOP-SPEC were written in scratch on 2026-09-29, beside L0.1–L0.6, with no P2.2b
   engine code. LOOP-SPEC's sha256 was fixed in its `SHA256SUMS` before any review.
2. The mirror review (2026-09-29) reproduced FUNDED independently and found one major fault:
   the mirror had no genesis carry (STATE O57), and with it registered values moved past §8's
   tolerances.
3. L0.8's fix round added the carry (`lm_carry.py`), checked it (LOOP-SPEC-A1 §A1.2), ran the
   loop battery again on it, and registered §8 again as A1 §A1.5.
4. This commit fixes the registration in the repository. P2.2b's build comes after it, and its
   trace diff (E0) comes before any scored run.

## 3. The county: FUNDED's chain8

Quoted from FUNDED §0:

**The county, `chain8`.** It is 1g's HORSE county with 8 heads instead of 12. Only N changes:
N 8 and T 10 a tick at 52 a year (416 and 520 a year), h 1, χ_max 1/20, γ = 0.2 + 0.8x. CHAIN's
horse is unchanged:

- fodder is made from 1 land, 8 labour and 2 horse-days a ton (rule B, the loop);
- a head is made from 8 t fodder, 20 labour and 3 pasture;
- a head gives 250 horse-days a year;
- a horse-day takes 0.0176 t fodder and 0.1 labour;
- δ is 8% a year, J_b 1 tick and ρ 0.

Quoted from FUNDED §4.1, the tape builder's parameters:

##### 4.1 Parameters, in a form the tape builder can take

These are P2.2a's keys where one exists (HORSES-SPEC §2.10; `probe::horses::setup`). Two keys
are new, because rule B's goods take goods: fodder's horse-days, and a head's fodder.

| key | value | unit | B | B-cut | note |
|---|---|---|---|---|---|
| `inst.workers` | 416 | FlowPerYear | ✓ | ✓ | 8 a tick at 52 a year (was 624) |
| `inst.land` | 520 | FlowPerYear | ✓ | ✓ | 10 a tick |
| `inst.chi_max` | 0.05 | Dimensionless | ✓ | ✓ | |
| `inst.eta`, `inst.g0`, `inst.g1`, `inst.k` | 1, 0.2, 0.8, 1 | Dimensionless | ✓ | ✓ | γ = 0.2 + 0.8x |
| `inst.good.weight`, `inst.space.weight` | 1, 1 | Dimensionless | ✓ | ✓ | h 1 |
| `inst.fodder.own` | 0 | Dimensionless | ✓ | ✓ | |
| `inst.fodder.traction` (new) | 2 | Dimensionless | 2 | **0** | horse-days a ton: the loop |
| `inst.fodder.labour` | 8 | Dimensionless | ✓ | ✓ | |
| `inst.fodder.land` | 1 | Dimensionless | ✓ | ✓ | × f under b × f |
| `inst.horse.run.fodder` | 0.0176 | Dimensionless | ✓ | ✓ | a horse-day's running recipe |
| `inst.horse.run.labour` | 0.1 | Dimensionless | ✓ | ✓ | |
| `inst.horse.kappa` | 250 | FlowPerYear | ✓ | ✓ | 250/52 = 4.8076923076923075 a tick |
| `inst.horse.delta` | 0.08 | FractionPerYear | ✓ | ✓ | `Clock::fraction`: 0.0016022075724025087 a tick |
| `inst.horse.own_hours` | 0 | Dimensionless | ✓ | ✓ | rule B's head takes no horse-days directly |
| `inst.horse.fodder` (new) | 8 | Dimensionless | ✓ | ✓ | fodder a head |
| `inst.horse.labour` | 20 | Dimensionless | ✓ | ✓ | |
| `inst.horse.land` | 3 | Dimensionless | ✓ | ✓ | pasture, × f under b × f |

Units and ticks:

- **Every recipe is per unit made:** a ton, a head or a horse-day. None depends on the tick
  length. Only N, T and κ are per year, and δ is a fraction per year; `Clock` converts them.
  (Rule A's head recipe, a·κ/δ, was per tick; rule B's is not.) Other tick lengths are
  untested here, and weekly is the floor anyway (decision 237).
- **J_b is 1 tick.** M3's hours use J 2 (decision 232's rule). At ρ 0, B at J 2 equals B at J 1
  bit for bit at every target and δ.
- **The cost shock b × f** multiplies `inst.fodder.land` and `inst.horse.land` together. This
  is the frame's HORSE-A land factor (decision 242).
- **The basis** is `Assumed("CHAIN.md §1.3-1.4 via unit-1g.md §3.3")` for the chain and
  `Assumed("funded 2026-09-29: 1g's HORSE county at N 8")` for the county. Neither is ever
  scored (R5).

FUNDED's own decision numbers, 240–245 (its "decision 242" above among them), are STATE's
254–259 (FUNDED-A1 §A1.4); "decision 237" and "decision 232" are STATE's. FUNDED-A1 amends one
decision's reason (labour supply's headroom, weighed over the run), one check (the good's price,
within 1.4e-14 of the 50-digit solve) and one range; the county and every number of
`instances.json` stand.

## 4. The registered predictions

Quoted from LOOP-SPEC-A1:

#### A1.5 Registered predictions for P2.2b's engine run (replaces §8)

Registered before any P2.2b engine code or run exists. The conditions are this file's header and
LOOP-SPEC §2, with §A1.1's carry. The engine is expected to match the mirror as P2.2a's did:
- every class exactly at LB1–LB3, the families and the flow controls;
- ticks and years within 10%, with three runs per instance allowed within 25%;
- lowest prices and troughs within 5% of the value given;
- dead ticks within 10% or 5 ticks, whichever is larger, market by market for fodder, horse-days,
  the good and land; labour's and the union's are reported, not scored (decision 274);
- ticks at labour's bound within 10% or 5 ticks; a run whose peak is within 5% of the bound may
  cross it or not.

**E0. Before any mode-B run: a trace diff.** `lm_carry.tick` against the engine for 2,000 ticks at
LB1 on hold, w × 2, w × 0.5, r × 2, r × 0.5, b × 2 at genesis, heads × 2, heads × 10 and Zs × 2 (a
one-tick stock), and at LW1 on hold, r × 0.5 and b × 2, each from the engine's own genesis. They
must agree within 1e-12 in log, the allowed partings being the order's cancellation
(HORSES-RULES §6.4) and, after a glut, the reservation's chatter within the mirror's own one-ulp
spread (O55). A parting at tick 1 names the carry (§A1.1): the registration is amended before any
scored run, not after.

**E1. Nesting (R1).**
- With every plant and the loop absent, every committed tape keeps its text, `tape_hash`,
  `world_id` and per-tick hash stream. The gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
  demo-gb `0xfad880fe08d06645` and the probe, markets and horses pins do not move.
- A planted tape at θ 1 equals the same tape without plants, value for value, for 2,000 ticks at
  LB1 on hold, w × 2 and r × 2.
- A fixed plant (no wear, no order) equals fixed-Q drs, with Q the plant held.

**E2. The rest point and mode A.** At LB1–LB3 and every cost target, genesis from 1g's
`ChainEconomy` with the plants at κ_p·X is a fixed point within 1e-12. Mode A passes at L, with
largest gaps of 6.7e-16, 1.0e-15 and 4.5e-14 in the mirror and below 1e-9 in the engine.

**E3. The verdict instances.**

| id | Tier 1 | Tier 2 | Tier 3 (10·L) | Tier 3S (10·L) | stocks family | L (the mirror's rule) |
|---|---|---|---|---|---|---|
| LB1 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 211,000 |
| LB2 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 202,000 |
| LB3 | 20/20 | 24/24 | 25/25 (25/25) | 28/28 (28/28) | 28/28 | 232,000 |

Every run is CONVERGED, and so every instance is GO. The years, lowest baskets, dead ticks by
market and lowest horse prices per tier are §A1.4's §7.1 table. The engine's own elasticity probe
sets its L; the mirror's τ are LOOP-SPEC §6's.

**E4. The kick sets and the slowest mode.** Every kick set decays. The engine's fitted g at the
base is within 0.03 a year of the mirror's: 0.847 (LB1), 0.830 (LB2), 0.901 (LB3), 0.836 (LW1).
The mirror's largest root per tick over the targets is 0.997950, 0.997501 and 0.999068.

**E5. The cost shocks.** §A1.4's §7.2 rows: troughs, years to tolerance, and the years for the
heads and each plant to come within 5%, at LB1–LB3 and LW1–LW3.

**E6. The glut.** heads × 10 and heads × 2 converge at LB1–LB3 with no dead tick. The lowest horse
price is 0.214–0.221 of target, the highest 2.02–3.76, and the withheld ticks are LOOP-SPEC §7.3's
(303, 1,315; 221, 1,034; 662, 2,694).

**E7. The pass-through.** After r × 2 at LB1: horse-days at most 5 dead ticks (mirror 0), fodder
175, the good 11, land 12; the fodder low 0.409 and the tasks' horse-days low 0.509. B-cut without
plants (LN7) has at least 100 dead horse-day ticks (mirror 146) and does not converge.

**E8. The flow control.** LW1–LW3 are GO, with §A1.4's §7.1, §7.2 and §7.6 numbers. The O14
comparison is read as §7.6 reads it: a b × 2 trough 0.036 higher in log with stocks at δ 8%, and
1.36 times the median years.

**E9. Negative controls** (each at L). Each keeps its verdict: NO-GO for LN1, LN2, LN3, LN5,
LN6, LN7, LN8, LN9, LF3, LF6 and LW0; LN4 GO in Tiers 1–3S with its stocks family 25/26. Each
tier's converged count is §A1.4's verdict table's within 2 runs; a run that does not converge may
be DEAD, DIVERGED or ORBITING where §7.7's table has another of the three (the carry alone moved
19 such runs at LN7). Beside that:
- LN1 and LN7 converge in no tier (the roots 1.0147 and 1.0104 a tick);
- LN2 orbits in Tier 3 (at least 10 of 25);
- LN3 is DEAD in every tier but two Tier-3 runs (b × 0.5 at genesis and dated), which orbit;
- LN5 orbits in every tier; LN6 is DEAD in every tier and in mode A;
- LF3 diverges on r × 2, heads × 2, heads × 10 and the workers' coin × 0.02 and × 0.1, the horse
  price crossing 1e-6 at ticks 156–173; at ψ 0, heads × 2 and × 10 cross it at ticks 155–158 at
  LB1–LB3;
- LF6 converges in 11, 4, 3, 5 and 5 runs (Tiers 1, 2, 3, 3S and the stocks family);
- LW0 fails Tier 3 (15 of 23 converge);
- LB1 at 12 ticks a year fails Tiers 1–2 (0 of 40).

**E10. Families, if built.** LF1, LF2, LF5, LF7, LF8 and LC1 are GO with §A1.4's numbers. LF4
needs STATE O51's roles and is GO in the mirror.

**E11. Labour supply's bound.** The runs of LB1–LB3 and LW1–LW3 that reach it are §A1.4's, with
their ticks at the bound; none starts there, and each converges.

**What would refute the design and send it back to the mirror:**
- a class change at LB1–LB3;
- a runaway of the horse price at LB1–LB3 with ψ 0.25;
- a converged run ending more than 1e-12 off its target's oracle point;
- a θ = 1 tape differing from the plant-free tape;
- the engine's slowest mode above 1 at any target.

## Appendix. LOOP-SPEC §8 as first registered (superseded by A1 §A1.5)

#### 8. Registered predictions for P2.2b's engine run

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
