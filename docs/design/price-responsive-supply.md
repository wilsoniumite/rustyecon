# A price-responsive Rule 1

> **STATUS: IMPLEMENTED AND MEASURED, 2026-07-31.** The text below is left as
> written, before any code existed, because it is the pre-registration (R6) and
> rewriting it would destroy the record (R14). Five of its claims survived
> contact with the engine and three did not. Corrections are marked **[MEASURED]**
> in place at the point they bear on; the full verdict is in `docs/PLAN.md`,
> Phase 4, "The price-responsive Rule 1, landed and measured".
>
> Headline: **ε_s moved off zero exactly as derived, and the corpus got worse.**
> The full rule takes the lr corpus from 8 live regions to **0**. The
> decomposition arm (`reservation_goods`: the band on storable outputs, the
> shipped rule on labour) is where the value is — median `LevelRange`
> 3.27e15× → 6.53e8×, median B8 gap 1.78e11× → 1.61e8×, and the first displaced
> solvable world in this repository that keeps its market.
>
> Superseded/wrong, in order of how much they cost:
> * **§2.2's labour rule is the thing that kills the corpus.** Not predicted; §8
>   predicted volume falling, not a permanent labour-market shutdown.
> * **§2's `R = 1` is only the reservation price in a zero-rent world.**
>   `solv_labour`'s equilibrium markup is 2, and the rule destroys a fixed point
>   the shipped rule holds exactly.
> * **§2.1's "structural price floor, not a clamp" is half an argument.** It
>   stops a starved price falling; it turns the same starvation into an
>   unbounded mark-*up*. B7's pins flip from −1.000000 to +1.000000.
>
> Confirmed: §3's `ε_s = I/q − 1` and the degree-0 blindness of the old probe;
> §4's loop-gain bound and its negative control; §2.3's prediction that services
> stay unresponsive; §6's "no new parameters" (the one field added is the
> A/B switch itself, which is a mechanism selector, not a magnitude).
>
> > **[SECOND PASS 2026-07-31 — adversarial review; two of the "confirmed" items
> > were overstated and are corrected in place below.]**
> >
> > * **"§4's loop-gain bound … confirmed" confirmed the ARITHMETIC, not the
> >   engine.** The test that appeared to check it never called the kernel. On
> >   the engine the measured gain reaches **2.28** at the registered dials.
> >   Marked at §4; the renamed test and the new engine measurement are in
> >   `tests/test_13_elastic_supply.rs`.
> > * **The headline `ε_s` range 0.216–45.2 is not reproducible as labelled.**
> >   Its figures come from three tick counts and two evolution rules, none of
> >   them recorded. Marked at §3; the re-measured table with its conditions is
> >   in PLAN Phase 4.
> >
> > ~~Unaffected: the displacement result (`solv_1g` returns to 1.000 under
> > `reservation`, sits at 1.010 with standard deviation exactly zero under
> > `inelastic`), which was independently re-verified and is the finding this
> > design is carried by.~~
> >
> > > **[THIRD PASS 2026-07-31 — the sentence above is struck. It is half right,
> > > and the wrong half is the half the design is carried by.]** Re-measured
> > > from a clean build over the whole displacement grid, both price rules, all
> > > three supply rules and three tapes
> > > (`examples/restoring_force.rs`, `tests/test_15_restoring_force.rs`;
> > > full table in PLAN Phase 4 "P4.8"):
> > >
> > > * **STANDS, and is stronger than stated.** Under `inelastic` the +1%
> > >   displacement does not merely have zero standard deviation — the price
> > >   series is **bit-constant for all 1000 ticks**, on all three tapes, under
> > >   both price rules. `max − min = 0` exactly.
> > > * **STANDS, with a boundary the sentence does not carry.** The freeze holds
> > >   exactly as far as `dead = 0.05` predicts: frozen at ×1.0512, moving at
> > >   ×1.0513, against the dial-derived `(1+d/2)/(1−d/2) = 1.051282`. Beyond
> > >   it the price does not stay and does not return; it leaves.
> > > * **FALSE.** "Returns to 1.000" fails the tape's own registered
> > >   `fixed_point_tol_log = 1e-12`. It returns to **1.0000000000186**, and the
> > >   residual is set by the **absolute** `1e-12` price-delta guard in
> > >   `src/systems/price_update/mod.rs:28`, not by the mechanism this design
> > >   note proposes. Doubling the price level halves the residual
> > >   (1.86e-11 → 9.89e-12), so a redenomination changes a real outcome.
> > > * **NOT MEASURED AT ALL, AND IT IS ONE-SIDED.** The displacement was only
> > >   ever applied *upward*. Displaced **down** 1% under `inelastic`, the price
> > >   does not sit at 0.990: it walks up through the target and rests at
> > >   **+0.87%** on the far side, driven by Rule 3's cash band and not by any
> > >   supply elasticity — while the standard deviation over the scored window
> > >   is still exactly 0.0. **`sd = 0` means "no motion in the window", not "no
> > >   restoring force"**, and this configuration satisfies the first while
> > >   violating the second.
> > >
> > > What the design may still be carried by is the first two bullets: the
> > > shipped rule provably cannot respond, and the reservation rule provably
> > > does. What it may *not* be carried by is "and therefore the world returns
> > > to its equilibrium", which holds over a measured range of roughly
> > > `[0.936×, 1.046×]` on `solv_1g` and is not an interval even there.

**Status as written: design only, 2026-07-31. No code written. Criteria below are
pre-registered predictions (R6); they are falsifiable and several of them are
predicted to fail.**

Spec for making the desk kernel's posted supply a function of the posted price
against the desk's own reservation price, so that an equilibrium price *exists*.
Follows PLAN Phase 4's "Not schedules — a price-responsive Rule 1", and is the
design note R14 asks for before code.

---

## 0. The defect, restated in one line

`examples/elasticity_probe.rs` measures posted supply elasticity at **exactly
0.000 for every good under the kernel**, because Rule 1 is
`max(inventory − b_out·scale·qty_out, 0)` and mentions no price. With
`ε_s = 0` and `ε_d = 0` (labour, services) the price loop's gain is

```
g = |1 + α(ε_d − ε_s)| = |1 − 0| = 1     exactly
```

— a **unit root**. `ln p` is then a random walk driven by imbalance noise plus a
deterministic drift `α·mean(imbalance)` per tick, which is precisely the measured
disease: bands of 1e5 (legacy) and 1e15 (kernel), and PLAN's arithmetic that a
mean imbalance of 0.0093 exhausts the whole 2.2× band over an 850-tick window.
The corpus has not been failing a stability *tuning*; it has been failing to have
a fixed point at all.

Everything below is aimed at one number: move `ε_s` off zero, by a mechanism, and
show what that does to `g`.

---

## 1. The reservation price: unit input cost at posted prices

**Chosen: the desk's own break-even price, computed from the recipe's input cost
at posted prices — the quantity `unit_margin` in `src/kernel/mod.rs:550` already
returns.** It is used in the scale-free form

```
R  =  Σ_out p_sell·qty_out  /  Σ_in p_buy·qty_in       (the desk's markup ratio)
```

so `R = 1` *is* "posted price equals reservation price", and `R` is homogeneous
of degree 0 in prices — no units of currency enter, so R12 holds under any
redenomination.

### Why cost and not `price_ema`

`price_ema` (SimState, emitted in telemetry) is the other candidate named in
PLAN. It is rejected, for five reasons, in descending order of how much they
matter:

1. **It has no fixed point in the level.** `p = p_ema` is satisfied by *any*
   constant price. A supply curve anchored on the price's own history therefore
   drifts to wherever the price went; the loop acquires an instantaneous
   crossing but keeps a zero eigenvalue in the level. That is the disease being
   treated, moved one derivative along. Cost anchors the crossing to technology,
   which does not move when the price does.
2. **It cannot deliver correctness.** The instrument PLAN just adopted — the
   tape's labour values imply a relative price vector; `lr_00` implies
   `RealWage = 2.0` — is only reachable if cost propagates into price. A
   reservation equal to yesterday's price propagates yesterday's price into
   today's, which is the definition of not going to the right place.
3. **It is a smoothing layer on price, which R1 forbids by name.** "Stability
   comes from buffers, dead-bands, small asymmetric steps and staggered
   activation. Never from … an added smoothing layer." A `p/p_ema` rule buys
   stability by low-passing the price itself. The cost rule buys it from a
   buffer, which is on R1's allowed list.
4. **It would promote a code literal into a behavioural constant.**
   `const EMA_ALPHA: f64 = 2.0/53.0` lives at `src/systems/price_update/mod.rs:6`
   and currently has *no behavioural consumer* — only telemetry and the NaN
   sweep read `price_ema`. Making Rule 1 read it makes it behavioural, and R2
   then requires it registered, giving the kernel an eleventh dial whose only job
   is to set how fast a desk forgets. `KernelParams` has no `Default` impl
   precisely to make that kind of drift loud; better not to start it.
5. **Its timescale is wrong.** A 52-tick span against a price loop with an
   ~10-tick time constant at α = 0.1 means the reservation *lags* the price by
   about five loop time constants. The response would be effectively lagged, not
   instantaneous — and §4 shows a lagged supply response is stable only up to
   `ε_s < 1/α`, half the instantaneous bound, and converges as a damped
   oscillation rather than monotonically. The whole point of moving the response
   into Rule 1 is that it is *instantaneous*.

### What is wrong with the cost reading, recorded rather than hidden

- **`cost = 0` desks have no reservation price.** A frozen source, an enclosed
  parcel, any desk with no purchased inputs: `R = ∞`, the band collapses to zero,
  the desk posts its whole stock, and `ε_s = 0`. Those markets keep their unit
  root. This is recorded, not guarded — a guard would hide it from the Phase 5
  phase map. No such desk is registered in the lr corpus; `local_services` and
  the farms and mills all buy labour.
- **Pass-through recipes co-move.** `flour_transport` has flour on both sides. At
  a single node `cost` moves with `p_flour`, so `∂ln R/∂ln p = w_out − w_in` and
  the elasticity is scaled by the *margin's* sensitivity, not the price's. That
  is correct economics — a transporter's supply responds to the spread — but it
  means transport desks are much less elastic than producers on their sell node.
  In the corpus `flour_transport` is a channel operator (buy node ≠ sell node) so
  `w_in = 0` at the sell node and the reduction does not bite. Measure it before
  assuming.
- **Unit cost is marginal cost only under constant returns.** The recipes are
  linear (`qty_per_unit · scale`), so it is. If a `SemiVariable` input with a
  fixed floor is ever registered, average and marginal cost diverge and this
  reservation becomes the wrong one. Named here so the day it happens is not a
  mystery.

---

## 2. The posting function

**One law for every seller:**

```
posted = max( capacity − reserve · (reservation / price) , 0 )
```

read as: *hold back a fixed **value** of stock, and let the price decide how many
**units** that value is.* The buffer is the intertemporal link, exactly as the
brief asks: cheap ticks make the withheld value expensive in units, so the desk
holds; dear ticks make it cheap in units, so the desk releases.

### 2.1 Storable producer / transport output `g`

```
flow_g   = s_eff · qty_per_unit          (this tick's production, unchanged)
R        = Σ p_sell·qty_out / Σ p_buy·qty_in           (unit_margin, unchanged)
band_g   = b_out · flow_g / R
posted_g = max( inventory_g − band_g , 0 )
```

Equivalently, and this is the sentence that explains it: with a single marketable
output, `band_g = b_out · outlay_per_tick / p_g`, so **the desk withholds stock
worth `b_out` ticks of its own input outlay, valued at the posted price.** Both
forms are already-computed own state — `outlay_per_tick` is
`src/kernel/mod.rs:733` and `unit_margin` is `:550`. The `R` form is preferred in
code because it never divides by a price.

Multi-output recipes are split by revenue share, which is what using one
recipe-level `R` for every output *is*: `band_g ∝ p_g·qty_g / Σ_j p_j·qty_j`. This
is a registered choice, not a discovery; a physical-share allocation would give
different per-good elasticities.

**The four properties the brief asks for.**

| property | check |
|---|---|
| monotone increasing in `p` | `∂posted/∂p = band/p > 0` wherever `posted > 0` |
| sane at `p = reservation` | `R = 1` ⇒ `posted = max(I − b_out·flow, 0)` — **exactly the shipped rule**. The change is a strict generalisation; every existing resting-state test of Rule 1 holds unchanged at zero profit |
| never more than held | the subtracted term is ≥ 0 for `R > 0`, so `posted ≤ inventory` identically |
| own state + posted prices | inventory, scale, and `state.price` at the desk's own buy and sell nodes. No foreign demand, no foreign fill |

**Guards (R5: a NaN metric FAILS, so the rule must not make one).**
`cost ≤ 0` ⇒ treat `R = +∞` ⇒ `band = 0` ⇒ post everything (the no-inputs case
above). `revenue ≤ 0` ⇒ `R = 0` ⇒ `band = ∞` ⇒ post nothing. Neither branch
divides by zero in the `R` form; both are reachable and both are the economically
right answer.

**The shutdown price falls out and is worth naming.** `posted = 0` when
`inventory ≤ b_out·flow/R`, i.e. at `p_shut = b_out · outlay_per_tick /
inventory`: *the price at which the entire stock is worth exactly `b_out` ticks
of input outlay.* It is decreasing in inventory — a desk sitting on a mountain of
stock will sell it down to a low price; a desk with two days' cover holds out.
That is storage arbitrage, obtained with no forecast, no order book, and no new
constant.

**A structural price floor, not a clamp (R1).** As `p → 0`, `band → ∞` and
supply → 0, so imbalance → +1 and the price rule marks *up*. The "price marks
itself down forever" failure mode that B7 catches (`node0/services pinned at
−1.000000`) becomes unreachable through this channel, because the seller
withdraws before the price reaches zero. No clamp was added; a supply response
was.

### 2.2 Labour (pop Rule 1)

Same law. Capacity is hours, the reserve is the hours the pop keeps for
self-provision, and the reservation wage is **already registered**: `parity` is
the pop's self-provision rate in baskets per hour (`tools/derive_parity.py`
derives it from technology), so

```
w_res = parity · P_basket            (P_basket = subsistence_price(), already in kernel/mod.rs:284)
H     = pop.size · (employed ? 1 : last_labour_fill_rate)      (the pair rule, unchanged)
posted = min( H,  π·H · max(1 + b_out·(1 − w_res/w), 0) )
```

At `w = w_res` this is `π·H` — **exactly today's rule**, again. The capacity is
`(1 + b_out)·π·H` capped at the pop's physical hours `H`, and the reserve is
`b_out·π·H`, chosen so the shape matches the storable case term for term: at the
reservation price a desk posts one unit of throughput out of `1 + b_out` units of
capacity, whatever it is selling.

Shutdown wage `w < w_res·b_out/(1 + b_out)` = ⅔ `w_res` at `b_out = 2`. Maximum
release `(1 + b_out)·π·H`, capped at `H`.

**Why not the naive `H·max(1 − w_res/w, 0)`.** It posts *zero* at `w = w_res`,
which collapses the labour market at exactly the wage the design wants it to
clear at, and its elasticity at the corpus's operating point (π at its floor of
0.01) is `(1−π)/π = 99` — loop gain `|1 − 0.1·99| = 8.9`, violently unstable. The
`π`-relative construction above keeps the elasticity at `b_out` regardless of
where π sits. Recorded because it was the first form written down.

> **[MEASURED 2026-07-31] THIS SECTION IS THE DEFECT.** The rule shuts a pop
> out of the labour market below `w = (2/3)·w_res`, and in `lr_00` the real
> wage falls through that at tick 7 and never returns — `P_basket` is mostly
> flour, flour's market is starved from tick 0 and so inflates at the full `α`
> per tick, while `p_labour` sits in a market that is often two-sided-empty and
> therefore frozen. Labour supply is identically zero from tick 7, nothing is
> produced, and every one of the corpus's 72 regions is DEAD. The section is
> right that π supplies the level and wrong that the within-tick rule can be
> layered on top of a level that is 2.5x-300x off: below the shutdown wage the
> rule does not supply a crossing, it supplies an exit. `reservation_goods`
> exists to run the rest of the design without it.

**π stays the slow state and stays load-bearing.** The within-tick rule supplies
the *crossing*; π supplies the *level*. From the floor, π at `eta_up = 0.04` per
activation with `S = 4` needs ~460 ticks to reach 1.0, inside the 850-tick scored
window but not comfortably. Whether the port should set a genesis π is the same
R6 question PLAN already raised for `chosen_size`: state the rule first, apply it
to both arms, report with and without.

### 2.3 Non-storable outputs (services) — and why the answer is "not here"

**A non-storable output's reservation price is zero, and this is not a modelling
convenience — it is correct.** Once the service has been produced, its input cost
is sunk; an unsold unit is worth nothing at the end of the tick. Selling it at
any positive price beats holding it, because it cannot be held. So the
profit-maximising supply of an already-produced perishable is **vertical**, and
any Rule 1 that made a service desk withhold at a low price would be modelling an
*irrational* desk destroying its own output. That is a sharper objection than
"awkward", and it rules the whole family out.

**Therefore: services keep `posted = min(flow, stock)` unchanged.** The
elasticity for a non-storable has to be bought at the point where the cost is not
yet sunk, which is Rule 3's input purchase (§5) and Rule 2's scale — both lagged
by at least a tick. §4 gives the stability cost of that lag.

`local_services` in the corpus is the hard case and should be stated plainly:
output `services` is `Ticks(1)` (non-storable, since `b_out·S = 8 > 1`) and its
only input `labour` is `Instant`. **There is no buffer on either side, so this
market has no structural source of supply elasticity at any lag.** Its crossing
must come from the demand side — the pops' budget ration, `ε_d = −(cost share)`,
which is zero whenever the budget does not bind. Predicted consequence: services
remain the worst-behaved market in the corpus after this change, and B7 should
still be able to pin it. If that prediction fails, the analysis here is wrong
somewhere.

---

## 3. Supply elasticity, symbolically

Write `q = I − K/p` with `K = band·p` the withheld **value** (constant in `p` when
input prices are held fixed). Then

```
∂q/∂ln p = K/p = band
ε_s      = (∂q/∂ln p)/q = band/q = (I − q)/q = I/q − 1
```

> **`ε_s = capacity/posted − 1`.** The same expression for every branch: a desk
> posting a third of what it holds is elastic at 2; posting all of it, at 0;
> posting a tenth, at 9.

Aggregating over the sellers at a node, `S(p) = Σ_i q_i` and
`ε_S = Σ_{i posting} band_i / S` — quantity-weighted, so the market's elasticity
is the total withheld value over the total posted.

**At a desk's resting point.** A desk selling everything it posts settles at
`I = band + flow` (Rule 1 sweeps the surplus every tick; this is the same
recursion that pins the superseded σ at `−1/b_out`), so `q = flow` and

```
ε_s* = band/flow = b_out / R
```

and at the zero-profit markup `R = 1`,

```
ε_s* = b_out = 2.0     (registered)
```

**`b_out` is now literally the slope of the supply curve**, which is what the
brief asked for, and it is the same number in all three branches: the labour rule
in §2.2 was built to reproduce it, and the aggregate over identical desks is
`Σ band / Σ flow = b_out` too.

> **[MEASURED] Confirmed, both halves.** `ε_s = I/q − 1` matches a numerically
> differentiated posting function to 1e-9 over a grid
> (`elasticity_equals_capacity_over_posted_minus_one`), and the corpus reads
> own-price `ε_s` of 0.216 to 45.2 on storables against exactly 0.000 on a
> uniform bump. Note the readings ABOVE `2/α = 20`: those markets are outside
> §4's bound, because a desk far from its resting point posts far less than
> `1/20` of its stock. The bound is a statement about the resting point, and
> the corpus does not sit at one.
>
> > **[CORRECTED 2026-07-31, second pass — "0.216 to 45.2" is not reproducible
> > as labelled and is struck.]** The closed form is confirmed and stands; the
> > *range* is not a measurement of anything stated. The four figures were taken
> > at three different tick counts and under two different evolution rules, and
> > nothing recorded that: `0.216`, `16.7` and `45.2` exist only at **tick 20
> > with the state evolved under `reservation_goods`**; `6.73`, `17.7`, `43.5`
> > only at **tick 200** under the same; `1.111` only at **tick 20 with the state
> > evolved under the SHIPPED rule**. Held to one stated condition — state
> > evolved under the shipped `inelastic` rule, which is what all 33 tapes
> > register — the maximum `ε_s` anywhere on `lr_00` over a 19-tick grid is
> > **31.7** (at tick 7, central difference; 26.6 one-sided), and at 14 of those
> > 19 tick counts the maximum is below 1.2. The honest table, with its
> > conditions, is in PLAN Phase 4 under
> > "[CORRECTED] the elasticity table". The sentence after it survives intact and
> > is now the *whole* content of the finding: `ε_s` is a function of the state,
> > `I/q − 1`, and the corpus does not sit at a resting point.

**A dimensional caveat that stops being cosmetic.** `kernel.md`'s parameter table
calls `b_out` a band "in activations of throughput", and an activation is `S = 4`
ticks — but `rule_1_sell` computes `flow = s_eff · qty_per_unit`, which is
*per-tick* production, so the shipped band is 2 ticks of cover, not 8. Under the
old rule that discrepancy only moved a threshold. Under the new one it *is* the
elasticity: the two readings give `ε_s* = b_out = 2` or `ε_s* = b_out·S = 8`.
Both are stable at α = 0.1 (§4), with very different convergence speeds and very
different margins to the instability boundary. **Pick one deliberately and record
the pick**; do not inherit it from a variable name.

**The elasticity is a partial, and the probe cannot currently see it.** `q`
depends on prices only through `1/R`, which is homogeneous of degree 0 — a
uniform inflation of every price leaves the posted quantity *identically*
unchanged. That is a requirement, not a defect (a supply that responded to
uniform inflation would be money illusion, and would break R12), but
`examples/elasticity_probe.rs:79-84` bumps **every good at every node by 1%
simultaneously**, so it would report `ε_s = 0.000` for the fixed rule as loudly
as it does for the broken one. **The probe must be re-instrumented to perturb one
(node, good) at a time before it is used to evaluate this change.** Given the
house rule about guards that cannot fail: the honest test is to bump one good,
confirm the *new* rule moves and the *old* rule does not, and additionally to
keep the all-goods bump as a check that the new rule reports exactly 0 there.

---

## 4. Loop gain, and the condition for stability

Linearise the price rule about a crossing. Let `x = ln p`, `s(p) = s*(p/p*)^{ε_s}`,
`d(p) = d*(p/p*)^{ε_d}`, `s* = d*`. Near the crossing
`imbalance = (d−s)/max(d,s) ≈ (d−s)/s ≈ (ε_d − ε_s)(x − x*)`, so
`p ← p·(1 + α·imbalance)` gives

```
x_{t+1} − x* = [1 + α(ε_d − ε_s)] (x_t − x*)
g            = |1 + α(ε_d − ε_s)| = |1 − α(ε_s + |ε_d|)|          (ε_d ≤ 0 ≤ ε_s)
```

**A crossing exists** iff `ε_s − ε_d > 0`, i.e. iff `ε_s > 0` (since `ε_d ≤ 0`),
i.e. iff `q < I` — iff the desk withholds anything at all. `b_out > 0` is already
validated at load (`game_data.rs:101`). **The stability condition is**

```
0 < α (ε_s − ε_d) < 2        ⟺        ε_s < 2/α + ε_d
```

At the registered `α = 0.1`: `ε_s < 20 + ε_d`, and with the measured
`ε_d ∈ [−1, 0]`, `ε_s < 19`. In state terms, since `ε_s = I/q − 1`:

> **the desk must post more than 1/20th of its stock.** At the resting point it
> posts `R/(R + b_out)` of it — one third at `R = 1, b_out = 2`.

At the resting markup `ε_s* = b_out/R`, so the entire condition collapses to a
single product:

```
α · b_out  <  2 R + α R |ε_d|        →  at R = 1:   α · b_out < 2
```

**Registered values: `α·b_out = 0.1 × 2.0 = 0.2`, a factor of 10 inside the
boundary.** And PLAN's open "α/η timescale" worry now has a closed form: `α` and
`b_out` enter only as their product, so the Phase 5 sweep over that axis is
one-dimensional.

> **[CORRECTED 2026-07-31, second pass — the bolded sentence above is true of
> the resting point and false of the engine, and it was quoted as if it were
> about the engine.]** Three things it gets wrong, all now measured:
>
> 1. **There is no single registered `α`.** `alpha` is a per-*good* dial
>    (`GoodDef::alpha`), not a kernel one. In `lr_00` it is 0.100 for wheat,
>    flour and services and **0.050 for labour**; `solv_1g`'s grain registers
>    0.050. So `α·b_out` is 0.2 on some markets and 0.1 on others, and the
>    resting-point gain is 0.800 or 0.900 depending which market you are
>    standing in.
> 2. **The claim was argued, never measured**, and the test that appeared to
>    check it did not. `the_loop_gain_bound_is_real_in_both_directions` passed
>    `eps_s = B_OUT` into a scalar map of analytic curves; no kernel code ran in
>    it. It is renamed
>    `the_loop_gain_bound_is_arithmetic_and_this_test_checks_only_the_arithmetic`.
> 3. **Measured on the engine, the gain is not 0.8 and is not always below 1.**
>    Central differences on live `lr_00` states (kernel arm, evolved under the
>    shipped `inelastic` rule, posting under `reservation`), first 20 ticks:
>    most readings sit between 0.9 and 1.0, three of 25 sit at **exactly
>    1.000** — the unit root, still there wherever a desk posts its whole stock —
>    and the worst is **g = 2.28** at tick 7, node 1, wheat (`ε_s = 31.7`,
>    `ε_d = −1.16`, `α = 0.100`). That is outside the boundary this section says
>    the registered dials sit a factor of ten inside.
>
> The derivation is not wrong; its *scope* was overstated. `g = 0.900` is
> reproduced exactly where the derivation applies — `solv_1g` at its
> hand-computed fixed point measures `ε_s = 2.000` — which is the positive
> control (`the_gain_instrument_reads_the_derived_number_where_a_desk_actually_rests`).
> The row of the table below labelled "proposed, per-tick `b_out`" is a
> statement about a desk at rest, and should be read only that way.

| configuration | ε_s | ε_d | g | time constant |
|---|---|---|---|---|
| **shipped, labour/services** | 0 | 0 | **1.000** | ∞ — unit root, no fixed point |
| shipped, flour (budget binds) | 0 | −1 | 0.900 | 10 ticks |
| **proposed, per-tick `b_out`** | 2 | 0 | **0.800** | 4.5 ticks |
| proposed, per-tick, budget binds | 2 | −1 | **0.700** | 2.8 ticks |
| proposed, activation `b_out·S` | 8 | 0 | **0.200** | 0.6 ticks |
| proposed, activation, budget binds | 8 | −1 | 0.100 | 0.4 ticks |
| deadbeat (fastest, g = 0) | 10 | 0 | 0 | one step |
| boundary | 20 | 0 | 1.000 | — |
| corner, `q → 0` | → ∞ | any | → ∞ | diverges |

The first row is the finding of §0 stated as a number: **the current engine sits
exactly on the unit circle for every market whose buyers are not budget-bound.**

**The corner, taken seriously.** `ε_s = I/q − 1 → ∞` as `q → 0`, which happens
when `R → α·b_out/2 ≈ 0.105` — a market priced below a tenth of break-even. The
gain exceeds 1 there and the linearisation predicts divergence. Two structural
reasons it does not become an explosion, plus one honest admission:

1. Below the shutdown price supply is exactly **zero**, imbalance is `+1`, and the
   price rises by the full `α` every tick, monotonically, until the desk
   re-enters. The high-gain region is entered only from below and is escaped
   monotonically upward.
2. The imbalance normaliser saturates at ±1, so the per-tick step is bounded by
   `α` whatever the elasticity. That bound is *pre-existing*, not something this
   design adds — it is leaned on for the *shape* of the failure, not for
   stability.
3. **Admission:** the result is a bounded limit cycle straddling the shutdown
   price, not convergence. It is confined to markets more than 9.5× underpriced
   against cost. Telemetry should report `min R` and `time below p_shut` per
   market per window so that this is measured rather than argued.

**Lagged responses (services, and Rule 3's buying) are strictly worse, with a
number.** If the supply response arrives a tick late,
`x_{t+1} = (1 + αε_d)x_t − αε_s x_{t−1}`, characteristic `z² − (1+αε_d)z + αε_s = 0`,
and the Jury conditions give `α·ε_s < 1`, i.e. `ε_s < 1/α = 10` — **half the
instantaneous bound** — with convergence as a damped oscillation instead of
monotone decay. This is the quantitative form of PLAN's "a lagged response is
what makes a loop oscillate", and it is the argument for putting the response in
Rule 1 rather than leaving it to Rule 2's scale.

**Why the dead-band must NOT be extended to Rule 1.** An obvious-looking move is
to freeze the posted quantity while `|R − 1| < dead`, to stop the offer
chattering. It must not be made: it would set `ε_s = 0` in exactly the
neighbourhood of equilibrium where the crossing is needed, restoring the unit
root at the fixed point and nowhere else — the worst possible place for it.
`dead` belongs on the *state variable* (scale), where chatter is costly, and
nowhere else. Recorded because it was considered.

> **[MEASURED] The bound and its negative control both hold** as a property of
> the scalar map (`the_loop_gain_bound_is_real_in_both_directions`, **renamed
> 2026-07-31 to `…_is_arithmetic_and_this_test_checks_only_the_arithmetic`**,
> because "the bound is real" was read as a statement about the engine and the
> test contains no engine): converges
> at `α·b_out = 0.2`, does not settle at 2.5, and is exactly stationary at
> `ε_s = ε_d = 0`. What the table could not show is that a market with **zero**
> supply is not on this curve at all: the normaliser saturates, the step is
> `+α` regardless, and the price runs away geometrically. That regime, not the
> corner at `R < 0.105`, is what the corpus actually enters.

### The payoff: this is what makes the existing dead-band deliver *correctness*

Rule 2 stops moving scale when `|relative_margin| ≤ dead`, and
`relative_margin = 2(R−1)/(R+1)`. So a scale-stationary desk satisfies

```
(2 − dead)/(2 + dead) ≤ R ≤ (2 + dead)/(2 − dead)      →      0.9512 ≤ R ≤ 1.0513
```

at the registered `dead = 0.05`. **Any scale-stationary state of this kernel has
every desk's markup within ±5% of break-even, and therefore has the relative
price vector within ~5% of the zero-profit / labour-value vector the tape
implies.** That is `RealWage = 2.0 ± 5%` in `lr_00` — the correctness criterion,
derived from dials that are already registered.

That property is true of the *shipped* kernel too. It has never been observed
because nothing reaches a scale-stationary state, and nothing reaches one because
the price loop has unit gain and never stops moving. Giving the loop a crossing
is what converts an already-present dead-band into a correctness guarantee. **If
this change lands and RealWage does not move toward 2.0 on the regions that go
scale-stationary, this derivation is wrong and should be marked in place.**

---

## 5. Rule 3 — the symmetric change on the buying side

**Needed, but only for storable inputs, and for a different reason than Rule 1.**

Rule 1's fix supplies a crossing on every market whose seller holds stock. It
supplies nothing on the markets whose sellers hold none — **labour and
services** — which are exactly the markets PLAN measured as having no crossing at
all (labour: supply 0.41 against demand 73.7, both elasticities zero). So the
buy-side change is aimed at the flow markets, the complement of Rule 1's target.

**Storable inputs — the mirror of Rule 1.** Today the target cover is
`need = desired(scale) · S`, price-independent. Mirror it:

```
target_i = S · desired_i · R        (R the same recipe markup as Rule 1)
buy_i    = max(target_i − held_i, 0) · ration        (ration unchanged)
```

`R = 1` reduces to today's rule exactly, as everywhere in this note. Monotone
decreasing in `p_i` (a cheaper input raises `R`), and
`ε_d,i = −(target/buy)·share_i`, which is `−share_i` when the desk holds nothing.
The unbounded-hoarding worry is answered structurally rather than with a cap: the
existing cash reserve `b_cash·outlay/S` and the pro-rata `ration` already bound
total spending, and buying into cheap ticks is precisely the intertemporal
behaviour the design wants on the input side.

**Non-storable inputs — cover cannot be the instrument.** Labour is `Instant`:
bought hours are destroyed at end of tick, so buying `S` ticks' worth already
wastes `(S−1)/S` of it, and scaling that by `R` would only waste more. (That is a
pre-existing defect worth its own investigation and is *not* fixed here.) The
available instrument is buying *fewer* inputs and producing proportionally less —
`production/mod.rs` already scales output by the input fill — so:

```
buy_i = desired_i · min(R, 1) · ration        for non-storable i
```

A desk operating below break-even cuts its input purchases within the tick; a
desk above break-even is capacity-bound by its own `chosen_size` and cannot use
more, so the kink at `R = 1` is a physical bound (like "never post more than you
hold"), not an R1 clamp. Elasticity `−share_i` below break-even, `0` above.

**What this does and does not fix for labour.** It gives the labour market a
demand-side crossing whenever buyers are unprofitable, and §2.2 gives it a
supply-side one at all times. It does **not** close the measured 180× level gap —
that is π's job and π moves at ~1%/tick. Do not expect the labour market to clear
in the first hundred ticks.

---

## 6. New registered parameters: none

Nothing new is required. The change reuses:

| dial | old role | new role |
|---|---|---|
| `b_out` | inert band width | **the supply curve's slope**: `ε_s* = b_out/R`, and the whole stability condition is `α·b_out < 2R` |
| `dead` | Rule 2 chatter gate | unchanged — and now, via §4's corollary, the width of the band inside which steady-state relative prices must lie |
| `epsilon`, `s`, `eta_*`, `phi` | unchanged | unchanged |
| `parity` (per pop, already registered) | σ_π's target | also the labour reservation wage, `w_res = parity · P_basket` |

**What a `b_out` sweep would now show, stated before it is run (R6).** `b_out`
and `α` enter the linearised loop only as their product, so a one-dimensional
sweep over `α·b_out` should show, at `R ≈ 1` and `ε_d ≈ 0`:

- monotone improvement in `LevelRange` from `α·b_out = 0.2` (registered, g = 0.8)
  toward `α·b_out = 1` (g = 0, deadbeat, one-step convergence);
- degradation past 1 as `g` grows again with the opposite sign;
- instability at `α·b_out = 2`, and clear divergence by 2.5.

**If the measured optimum is not near `α·b_out ≈ 1`, the linearisation in §4 is
wrong** and this note should be marked superseded rather than retuned. That is
the falsification surface for the whole design, and it is cheap: it is one axis
of the Phase 5 phase diagram, which was going to be swept anyway.

The dimensional question in §3 (`b_out` in ticks or in activations) changes which
registered value sits where on that axis — 0.2 or 0.8 — and both are inside the
stable region, which is why the pick must be recorded rather than derived from
the sweep after the fact.

---

## 7. Falsification tests, before any of this is trusted

House rule: a guard that cannot fail is not a guard. Each of these must be shown
firing on a defect known to exist before it is trusted to say something is clean.

1. **`posted_supply_is_price_inelastic_under_the_shipped_rule`** — the defect
   test. Bump one good's price by 1% and assert the *current* Rule 1 returns a
   bit-identical quantity. It must pass today. The new rule must make it fail.
2. **`the_all_goods_probe_cannot_see_the_fix`** — assert that under the *new*
   rule a uniform 1% bump on every price still gives exactly 0.000 elasticity
   (degree-0 homogeneity), so the existing `elasticity_probe` is proven blind
   before it is fixed. This is the test that would catch someone declaring
   victory off the old probe.
3. **`elasticity_equals_capacity_over_posted_minus_one`** — a closed-form check
   of `ε_s = I/q − 1` against a numerically differentiated posting function, over
   a grid of `(I, R, b_out)`, to 1e-9.
4. **`loop_gain_bound_is_real`** — iterate the scalar map with analytic `s(p)`,
   `d(p)`; assert convergence at `α·b_out = 0.2` and *divergence* at
   `α·b_out = 2.5`. The negative control is the point: a bound nothing violates
   is decoration.
5. **B6 must still pass under the kernel.** B6 perturbs last tick's aggregated
   `supply` and `demand` on a clone. The new rule reads `state.price`, which B6
   does not touch, so it must remain clean — and it must be re-run rather than
   assumed, because "reads a price" is exactly the kind of claim that turns out
   to have a demand term hiding in it.
6. **B7 should now be able to *fail differently*.** Predicted: the storable
   markets stop pinning; `services` still pins. If services stops pinning too,
   §2.3's reasoning is wrong.

---

## 8. Risks, in the order they are likely to bite

1. **The probe is blind (§3).** Re-instrument it first. Nothing else in this note
   can be measured until that is done.
2. **Cleared volume falls before it rises.** Withholding is the mechanism, so the
   first-order effect on `DEAD` and on the liveness gate G1 is *negative*. The
   last A/B traded `DEAD_BUILDING` 27→0 for `POP_DESTITUTION` 9→60; this change
   could trade bands for liveness in the same way. Predict the direction and
   pre-register it (R6), because discovering it after the run and then arguing
   about which gate matters is the failure mode the preregistration exists to
   stop.
3. **Endogenous famine.** A desk withholding flour at `p < c` while pops are
   destitute is now a *deliberate* mechanism. R8 says rationing is the point and
   who goes short is a first-class output, so this is arguably the model working
   — but `POP_DESTITUTION` is a scored class and it will move. It must be
   reported, not explained away.
4. **The corner limit cycle (§4).** Bounded, confined to `R < 0.105`, but real.
   Needs `min R` telemetry to be visible at all.
5. **σ's contraction channel changes character.** Today a glut shows up as unsold
   offers (`fill < 1`). Under the new rule a desk facing a low price withholds
   and therefore *sells everything it posts*, so `fill → 1` and `sigma` falls
   through to the margin branch — which is negative, so it still contracts, but
   through a different term. `sigma()` is unchanged; the *path* through it is
   not. Expect the fill EMA to sit much closer to 1 across the corpus, and check
   that the contraction still happens.
6. **Price-level determinacy is unaffected but no longer the whole story.** The
   anchor is still money demand (`M ≈ Σ b_cash·outlay/S`); the new rule is
   homogeneous of degree 0 and contributes nothing to the level, by construction.
   Worth stating because a reader will reasonably expect a cost-anchored supply
   rule to pin a level, and it does not — it pins *relative* prices.
7. **Two rules now read `R`.** Rule 1 and Rule 2 must see the *same* `R` in the
   same tick, or the desk can post as if profitable while shrinking as if not.
   Compute `unit_margin` once in `kernel::run` and pass `(revenue, cost)` into
   both. `rule_1_sell` currently takes only `sell_node` and will need `buy_node`
   as well — the caller already has it.

---

## 9. What this note does not do

- It does not re-propose the buffer-band σ. That signal is proven unable to
  expand a desk (`desk_buffer_sigma_is_pinned…`), and nothing here touches Rule
  2's σ.
- It does not change `markets.md`'s clearing. PLAN's earlier "post schedules"
  conclusion is already marked superseded there; this is a change to Rule 1 and
  to Rule 3's target cover, and to nothing else.
- It does not fix the `Instant`-input waste in Rule 3 (buying `S` ticks of a good
  that is destroyed each tick). That is a separate, pre-existing defect,
  identified here and left where it is.
- It does not claim the corpus will pass. The lr set is a search grid, five of
  seventy-two regions have ever scored in-band and all five were dead, and this
  change is aimed at one specific measured number — `ε_s` — not at the gate.
