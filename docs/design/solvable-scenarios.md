# Solvable scenarios: worlds whose equilibrium is known before the run

**Status: design only, 2026-07-31. No code written, no tape written. Every
number below is derived by hand from the source and is a pre-registered
prediction (R6). Several of the predictions are predicted to FAIL, and those are
the point.**

> **BUILT AND MEASURED, 2026-07-31 — status above superseded, marked in place
> rather than rewritten (R14).** The five tapes are `data/scenarios/solv_1g`,
> `solv_1g_money`, `solv_1g_money_2x`, `solv_chain`, `solv_labour`; the gate is
> `tests/test_12_solvable.rs`; the registered targets are `equilibrium.ron`
> beside each tape, parsed by `src/scenario/equilibrium.rs`. Certificates are
> persisted under `results/solv_*`. Full write-up in `docs/PLAN.md`, "The
> solvable scenarios". What this note got right and wrong:
>
> * **§6 mode A — CONFIRMED, exactly.** Worst `|ln(realised/target)|` is
>   `0.000e0` on all five tapes under both price rules and under the desk
>   kernel. Every genesis cash constant in §2.4, §3, §4.2 and §5.3 was right
>   first time, which §8.1 flagged as the likeliest place for a slip. All four
>   `parity` values (0.5, 1.0, 1.0, 2.0) match `tools/derive_parity.py` run
>   against the finished tapes, as §8.6 asked.
> * **§6 mode B — REFUTED, and worse than predicted.** The note expected an
>   orbit centred on the fixed point, scored on the median. What happens is that
>   the *market dies*: traded volume decays to 0.00–0.36 of its own opening level
>   across the window in all eight configurations, and under `--price-rule ratio`
>   every market trades exactly nothing for the last 800 ticks. Seven of eight
>   also miss `ln(1.10)` on price, by 1.9× to 1.5e11×.
> * **§7.1 — the SIGN is confirmed and the magnitude is far larger.** The note
>   predicted slow anti-damping, amplitude growing like `sqrt(t)`. Under the
>   `ratio` rule the `b_out` feedback is not slow at all; it runs away in under
>   100 ticks. §8.2's own warning about linearising a relay system was the right
>   warning.
> * **§7.3 — CONFIRMED exactly.** Money is neutral to 0 ulp across 1,000 ticks.
> * **§5.5 — CONFIRMED.** Legacy has no rest point on `solv_labour`. The note
>   predicted the labour imbalance pinned at −0.5; B7 measures it pinned at
>   **−0.403032 for all 851 ticks**, same phenomenon, and the geometric wage
>   decay it implies matches the measured 9e8× error.
> * **§5.2's warning that B8 would score `solv_labour` wrong — CONFIRMED.** B8
>   reads grain "2.000× too dear", which is exactly the capacity rent and not an
>   error.
> * **One thing the note did not anticipate.** Genesis `participation` was not
>   settable from the tape — the loader hardcoded 1.0 — so `solv_labour` could
>   not start at its own interior `pi* = 0.5`. `RawPopGroup.participation` was
>   added, absent-means-1.0, and no existing tape changed.
> * **§1.1's `s = 1` and §8.3's worry about it did not bite.** In mode B the
>   binding constraint is not R-F's expansion rationing (the displaced firm is
>   cash-rich from the price shock); it is the price-inelastic labour supply and,
>   under `ratio`, R-E's buffer feedback.

The instrument PLAN Phase 4 says is missing: *"Every criterion in this project
is a stability criterion... Not one asks did it go to the right place."* This
note specifies a family of minimal worlds whose competitive equilibrium — price
vector, quantity vector, money stock, and the metric series the certificate
already computes — can be written down exactly, in closed form, from the tape,
before the engine is run.

`data/scenarios/tracer_2r` was built small but its header says plainly it is
"not to be economics". `lr_*` is a factorial search grid. Neither has a known
right answer. These do.

---

## 0. What "solvable" has to mean in *this* engine

A closed-form equilibrium is only useful if it is a fixed point of the actual
tick loop, not of an economist's idealisation of it. Six rest conditions are
read directly off the source; every derivation below is an application of them.

**R-A — price stationarity is exact market balance.**
`price_update` is `p ← p·(1 + α·clamp(imbalance, ±1))` with
`imbalance = (d − s)/max(d, s)` (`src/systems/clearing/mod.rs:136`). For α > 0
this is stationary **iff `d == s` exactly**, or both are ≤ 0. Not "clears in a
welfare sense" — the *posted order quantities* must be equal, market by market,
every tick. Under `--price-rule ratio` the same state is also stationary
(`d/s = 1`), so every scenario here is fixed under **both** price rules; mode A
below therefore tests the agent layer with the price rule factored out.

**R-B — a desk's scale rests inside the margin dead-band.**
`sigma(fill, relative_margin, dead)` takes the margin branch when
`fill ≥ 1 − dead`, and `rule_2_nudge` steps only when `|σ| > dead`
(`src/kernel/mod.rs:539,593`). With `R` = revenue and `C` = cost per unit of
recipe size, `σ = (R−C)/((R+C)/2)`, so scale is at rest iff

```
(2 − dead)/(2 + dead)  ≤  R/C  ≤  (2 + dead)/(2 − dead)
    0.9512             ≤  R/C  ≤  1.0513              (dead = 0.05)
```

**Zero profit is the kernel's rest point, to within ±5%.** That band is not a
measurement allowance; it is the width of the rest *set*, and it is where the
tolerance in §5 comes from.

**R-C — participation rests inside its own dead-band, or at a clamp.**
`σ_π = (p_labour/P_subsistence − parity)/parity`, `π ∈ [ε, 1]`
(`src/kernel/mod.rs:396`). Two rest shapes: interior
(`|σ_π| ≤ dead`, a horizontal labour-supply curve at the reservation wage) or
clamped (`π = 1`, a vertical labour-supply curve at head-count).

> **A structural fact that decides the whole design.** `tools/derive_parity.py`
> sets `parity = HOME_PENALTY / (labour embodied in one subsistence basket)`
> with `HOME_PENALTY = 0.5`. In *any* economy whose only primary input is
> labour, the zero-profit price vector gives the market real wage exactly
> `1/embodied`, so
>
> ```
> σ_π = (1/embodied − 0.5/embodied)/(0.5/embodied) = +1     identically
> ```
>
> — independent of the basket, the chain length, and the price level. **At the
> zero-profit price vector the participation margin is always pinned at the
> π = 1 corner.** It can only be interior when something *other than labour* is
> scarce. That is scenario 3, and it is why scenario 3 is a different world
> rather than scenario 1 with different numbers.

**R-D — the pop's wealth tier rests where cash equals its band.**
`σ_c = (cash − reserve)/reserve`, `reserve = b_cash·basket_cost/s`, and the rule
is skipped entirely when `max_wealth_tier == 0` (`src/kernel/mod.rs:336,377`).
So a **one-tier** tape freezes the basket at a constant physical quantity and
leaves the price *level* unanchored; a **two-tier** tape switches the anchor on
and pins the level in closed form. Both are specified below, deliberately.

**R-E — Rule 1's buffer implies a stock, and a perverse short-run supply.**
Posted supply is `stock − b_out·scale·qty_out`; production adds `scale·qty_out`.
So `stock_{t+1} = (1 + b_out)·scale_t`, and at rest **posted supply = production
= scale** exactly. Substituting one step off rest, with `scale_t = scale_{t−1}(1+g)`:

```
posted_t = (1 + b_out)·scale_{t−1} − b_out·scale_t = scale_{t−1}·(1 − b_out·g)
```

**A desk that expands by `g` posts `b_out·g` LESS this tick**, because it must
first fill the wider band. The short-run own-supply elasticity with respect to
the desk's own expansion is `−b_out`. This is derived here, not measured, and it
is one of the things the family exists to measure.

**R-F — Rule 3 leaves a firm with exactly zero cash slack.**
Overflow is `(cash − reserve − planned)` and leaves every activation
(`src/kernel/mod.rs:706`), so at rest a firm holds *precisely*
`reserve + planned = (b_cash/s + s)·outlay`. At **zero profit there is no
cushion at all**, and the arithmetic of trying to grow out of it is:

```
grow by g without rationing   ⟺   g ≤ m·s² / (b_cash + s²),   m = profit/outlay
   s = 1, b_cash = 6.5, g = η_up = 0.04  ⟹  m ≥ 0.30, i.e. σ ≥ 0.26
   s = 4, b_cash = 6.5, g = η_up = 0.04  ⟹  m ≥ 0.056, i.e. σ ≥ 0.109
```

but Rule 2's expansion gate is `σ > dead = 0.05`. **For σ between 0.05 and
0.26 (at s = 1) the kernel expands and immediately rations its own inputs.**
There is no credit in the model, so a desk at zero profit cannot fund growth at
all: contraction is free and expansion is not. That asymmetry is a candidate
explanation for the corpus's collapse, and these scenarios make it measurable
rather than arguable.

---

## 1. Two design decisions that are forced, not chosen

### 1.1 `s = 1`. The stagger has no meaning with one desk per market.

`rule_3_buy_and_route` posts a desk's input demand only on ticks where
`(tick + inventory_index) % s == 0`, and it buys `s` ticks' worth at once. Pops
post labour **every** tick (`kernel/mod.rs` header: buying is per tick, only the
scales are staggered).

> **Proof that no constant-price equilibrium exists for `s > 1` with one desk
> per market.** On any tick that is not the desk's activation, labour demand is
> 0 while labour supply is `N·π > 0`. Then `imbalance = −1` and
> `p_labour ← p_labour·(1 − α)`. A constant price therefore requires `π = 0`,
> i.e. a dead economy. ∎

The two escapes are both worse for this purpose: `s` desks with input
inventories spread across the residues mod `s` (which reintroduces the
population these scenarios exist to remove), or a storable labour good (which
makes a desk stockpile labour-hours for `s` ticks and is not what labour is).
So the whole family registers `s: 1`, and everything else in the kernel block
stays at the corpus values so results transfer.

`s = 1` is not free: R-F above shows it makes the expansion-rationing threshold
2.4× harsher than the corpus's `s = 4`. That is stated here rather than
discovered later, and `b_cash` is the registered dial if it has to be revisited
— by a dated amendment, not an edit.

### 1.2 Every number in the tape is a dyadic rational.

All prices, quantities, stocks and cash balances below are exactly representable
in binary f64 (halves, quarters, and small integers). Every arithmetic path at
the equilibrium — `stock − b_out·flow`, `b_cash·outlay`, `qty·price`, the fill
ratios `20/20` — is therefore exact, and every rule's response is *identically*
zero rather than approximately zero. This is what lets mode A assert a bit-exact
fixed point and read any motion at all as a defect. It is also why `r = 2`,
`a = 0.5` and `q = 2` recur: they are the smallest interesting exact numbers.

---

## 2. Scenario 1 — `solv_1g`: one good

**Purpose.** The smallest world that is an economy: currency + labour + one
produced good, one region, one pop, one producer desk. It fixes one relative
price by technology and nothing else, so a failure here cannot be blamed on a
chain, a channel, or a substitution.

### 2.1 Technology and the closed form

One recipe: `a = 0.5` labour per unit of grain, i.e. `r = 1/a = 2` grain per
unit of labour.

**Prices, by zero profit.** The desk rests iff `R/C ≈ 1` (R-B), and per unit of
recipe size `R = p_grain·1`, `C = p_labour·a`:

```
p_grain = a·p_labour          ⟹   RealWage = p_labour/p_grain = 1/a = r = 2
```

**Quantities, by exact market balance (R-A).** Let `N` be head-count, `q` the
per-head basket, `K` the desk's scale.

| market | posted supply | posted demand | balance |
|---|---|---|---|
| labour | `N·π` | `a·K` | `a·K = N·π` |
| grain | `K` (R-E) | `N·q` | `K = N·q` |

`π = 1` at the corner (R-C), so `K = N/a = N·r` and therefore **`q = r`**: the
tape must feed each person exactly one unit of labour's worth of grain per tick,
which is the only basket a one-factor economy can sustain. With `N = 10`,
`a = 0.5`: `K = 20`, `q = 2`, labour cleared 10/tick, grain cleared 20/tick.

**Stocks.** `grain stock = (1 + b_out)·K = 3·20 = 60` (R-E), posting
`60 − 40 = 20` ✔.

**Cash.** Firm holds `(b_cash + 1)·outlay = 7.5·(a·K·p_labour) = 7.5·10 = 75`
(R-F, s = 1); revenue `20·0.5 = 10` equals outlay `10`, so overflow is exactly
zero and the balance is self-reproducing. The pop's income
(`p_labour·10 = 10`) equals its spending (`p_grain·20 = 10`), so its cash is
constant at whatever it starts with; 30 is registered (three ticks of
subsistence).

### 2.2 What pins the price LEVEL, and it is not sharp

With one wealth tier the pop's cash band is inert (R-D), and every posted
quantity is a physical quantity that no price enters. The excess-demand system
is therefore **homogeneous of degree zero**: `λ·p*` is a rest point for every λ
that leaves no cash constraint binding. The bounds:

* **From above, λ ≤ 1, and it binds exactly at the equilibrium.** The firm's
  requirement is `(b_cash + 1)·outlay(λ) = 75λ` against a cash stock fixed at 75
  (zero profit ⟹ no inflow). λ > 1 rations its labour purchase → excess labour
  supply → the wage falls. R-F's kink *is* the level anchor.
* **From below, nothing.** At λ < 1 the firm's overflow hands the surplus to the
  pop and everything else is unchanged; the pop's constraint
  (`105 ≥ 85λ`, i.e. λ ≤ 1.235) is slack.

**So the equilibrium is a half-open ray `λ ∈ (0, 1]`, and the level is pinned on
one side only. It is not sharp enough to test as a point prediction.** Two
consequences worth registering:

1. A correctness battery must score **relative** prices against a numeraire.
2. `Velocity = Σ(cleared·price)/Σ currency` is proportional to the price level
   with a fixed money stock. **The `DRIFTING` criterion on Velocity is therefore
   scoring an unpinned direction of the dynamics** — a run can have perfectly
   correct relative prices and still fail it. That is a criticism of the
   existing criteria set which this scenario *proves* rather than asserts, and
   it is the reason `solv_1g_money` (§3) exists.

### 2.3 `game_data.ron`

```ron
(
    // s = 1 is forced: with one desk per market, any s > 1 makes input demand
    // intermittent and no constant price is a fixed point (design note §1.1).
    // Everything else is the corpus value so results transfer.
    kernel: (
        s: 1,
        eta_up: 0.04, eta_dn: 0.05, dead: 0.05,
        b_out: 2.0, b_cash: 6.5, beta: 1.0,
        epsilon: 0.01, phi: 0.6180339887498949, fill_alpha: 0.25,
    ),
    goods: [
        ( name: "grain",  alpha: 0.05, shelf_life: Indefinite, movement_type: Physical,  divisible: true, storage_cost_per_tick: 0.0 ),
        ( name: "gbp",    alpha: 0.0,  shelf_life: Indefinite, movement_type: Financial, divisible: true, storage_cost_per_tick: 0.0 ),
        ( name: "labour", alpha: 0.05, shelf_life: Instant,    movement_type: Local,     divisible: true, storage_cost_per_tick: 0.0 ),
    ],
    recipes: [
        // a = 0.5 labour per grain, so r = 2 grain per labour and RealWage = 2.
        // Variable scaling: Fixed would draw recipe_size of labour regardless of
        // chosen_size and break the capacity -> labour-demand loop (the defect
        // tracer_2r records).
        ( name: "grow_grain",
          inputs:  [ ( good: "labour", qty_per_unit: 0.5, scaling: Variable ) ],
          outputs: [ ( good: "grain",  qty_per_unit: 1.0 ) ],
          reversible: false, strategy: CapacityControl ),
        // NOT a desk under the kernel: Rule 3's overflow routing reads this
        // instance purely as the claim-holder pointer (kernel/mod.rs:620), and
        // forces its chosen_size to 0. Under --agents legacy it is the dividend
        // desk. Both arms therefore have a currency return path.
        ( name: "dividend",
          inputs:  [ ( good: "gbp", qty_per_unit: 1.0, scaling: Variable ) ],
          outputs: [ ( good: "gbp", qty_per_unit: 1.0 ) ],
          reversible: false, strategy: DividendPayout( reserve_multiple: 26.0 ) ),
    ],
    // One entry: logit shares are [1.0] and beta is inert. One tier: the wealth
    // rule is skipped entirely and the basket is a constant physical quantity.
    need_categories: [ ( name: "food", entries: [ ( good: "grain", weight: 1.0 ) ] ) ],
    wealth_levels:   [ ( tier: 0, needs: { "food": 2.0 } ) ],
    market_nodes:    [ ( id: (0), tier: Regional, region: Some((0)), currency_good: Some((1)) ) ],
    channels:        [],
    regions:         [ ( id: (0), name: "Solo", market_node: (0) ) ],
)
```

### 2.4 `starting_state.ron`

```ron
(
    tick: 0,
    // The equilibrium price vector. p_grain = a·p_labour = 0.5.
    prices: { "grain": 0.5, "gbp": 1.0, "labour": 1.0 },
    buildings: [
        // stock 60 = (1 + b_out)·scale, so Rule 1 posts exactly 20 = production.
        // gbp 75 = (b_cash + 1)·outlay = 7.5·10, so overflow is exactly zero.
        // recipe_size 25 > chosen 20 deliberately: scale is interior and free to
        // move in both directions, so a defect is not masked by a clamp.
        ( id: (0), region: (0), recipe: "grow_grain",
          recipe_size: 25.0, chosen_size: 20.0, efficiency: 1.0,
          inventory: { "gbp": 75.0, "grain": 60.0 } ),
    ],
    pop_groups: [
        // parity 0.5 = HOME_PENALTY / (q·content(grain)) = 0.5/(2·0.5); it is
        // what tools/derive_parity.py emits for this tape, and it is derived from
        // technology, never from the genesis real wage (R6).
        // savings_target 0.9 is inert under the kernel; it keeps the LEGACY arm's
        // wealth PD controller resting at 0 instead of ramping a clamped value.
        ( id: (0), region: (0), size: 10.0, wealth: 0.0,
          inventory: { "gbp": 30.0 }, savings_target: 0.9,
          labour_good: Some("labour"), parity: Some(0.5) ),
    ],
    extra_recipe_instances: [
        ( recipe: "dividend", building: (0), output_pop: (0), recipe_size: 1000.0 ),
    ],
)
```

`events.ron` is `( recurring: [] )`. `criteria.ron` is `tracer_2r`'s
generation-4 file **verbatim** — same thresholds, same dated generation — with
only `analysis_end: 1000` and `staple_good: "grain"`.

### 2.5 The predicted equilibrium

| quantity | value | source |
|---|---|---|
| `p_labour` | 1.0 (numeraire) | free |
| `p_grain` | 0.5 | `a·p_labour` |
| `p_gbp` | 1.0 | `alpha: 0.0` |
| **RealWage** | **2.0** | `1/a` |
| labour cleared/tick | 10 | `N·π` |
| grain cleared/tick | 20 | `N·q = K` |
| `chosen_size` | 20 | `N/a` |
| firm grain stock | 60 | `(1+b_out)K` |
| firm gbp | 75 | `(b_cash+1)·outlay` |
| pop gbp | 30 | free (≥ 10) |
| **Velocity** | **20/105 = 0.190476…** | `(10+10)/M` |
| Employment | 1.0 | fill = 1 |
| Concentration | 75/105 = 0.714285… | |
| BldUtil | 0.8 | 20/25 |
| RealIncome | 6.0 | `(30/10)/0.5` |

Every registered failure class passes on these constants: `DEAD` needs Velocity
< 0.003 (0.19), `UNSTABLE` a rolling CV > 0.12 (0.0), `DRIFTING` a band > 2.2
(1.0), `SWINGING` a residual ratio > 1.5 (flat scores 0.0 by
`verdict.rs::residual_damping`), `CURRENCY_DRAIN` a mean concentration ≥ 0.82
(0.714), `POP_DESTITUTION` RealIncome < 0.8 (6.0). **B7 does not fire**: it
requires `|mean imbalance| > 1e-9`, and a market that clears exactly is
explicitly exempt (`invariants.rs:217`).

---

## 3. Scenario 1M — `solv_1g_money`: the same world with the level pinned

**Purpose.** Switch on the one mechanism that gives money demand two sides, and
get a closed-form *level* — hence a money-neutrality test that doubles as an R2
audit.

Identical to `solv_1g` except: **two wealth tiers**, `tier 0: food 1.0` and
`tier 1: food 3.0`, so `q(w) = 1 + 2w`, and the pop starts at `wealth: 0.5`.

**The tier is pinned by quantity balance.** Labour balance with π at the corner
still gives `K = N/a`, and grain balance gives `q = K/N = 1/a = r = 2`, so
`w* = 0.5` exactly. (`interpolated_qty(0.5)` = `1 + 0.5·(3−1)` = 2 ✔.)

**The level is then pinned by the pop's cash band (R-D).** `σ_c = 0` requires
`pop cash = (b_cash/s)·basket_cost = b_cash·p_grain·N·r`, and the firm holds
`(b_cash+1)·outlay = (b_cash+1)·N·p_labour = (b_cash+1)·N·r·p_grain`.
Money is conserved, so `M = pop + firm`:

```
M = N·r·p_grain·(b_cash + b_cash + 1)

           M                            140
p_grain = ----------------  =  ------------------  =  0.5
          N·r·(2·b_cash + 1)      10·2·(14)

p_labour = r·p_grain = 1.0
```

and, as a corollary that needs no scenario-specific arithmetic at all,

```
Velocity* = 2 / (2·b_cash + 1) = 2/14 = 0.142857…
```

**equilibrium velocity is a pure function of the cash band.** Genesis: firm
gbp 75, pop gbp 65, `M = 140`. `σ_c = (65 − 6.5·10)/65 = 0` exactly.

**The neutrality test.** Register a second tape, byte-identical except that every
currency balance and every price is doubled (firm gbp 150, pop gbp 130,
`p_grain 1.0`, `p_labour 2.0`). Prediction:

> Every real quantity, every fill rate, every metric series **except the
> currency stocks is bit-identical between the M and 2M runs, and every price is
> exactly 2× at every tick.**

Every kernel rule is a function of ratios (dead-bands on σ, clamps on `π` and
`scale/recipe_size`, `cash/reserve`), so homogeneity is exact — *unless some
absolute constant with behavioural meaning has leaked into the code.* **This is
the sharpest R2 audit available, and it costs one extra tape.** A failure names
the leak by which good and which tick.

---

## 4. Scenario 2 — `solv_chain`: labour → wheat → flour

**Purpose.** Does the engine propagate value through a production chain? The
labour value of flour is the sum along the chain, so the test is a price
*vector*, and the intermediate price is the thing no single-good world can
check.

### 4.1 Technology and the closed form

```
wheat_farm :  0.25 labour            -> 1 wheat
grain_mill :  1 wheat + 0.25 labour  -> 1 flour
```

**Labour content** (the relaxation in `tools/derive_parity.py`, by hand here):

```
content(labour) = 1
content(wheat)  = 0.25·1                    = 0.25
content(flour)  = 1·0.25 + 0.25·1           = 0.50
```

**Price vector by zero profit at each stage** (R-B applied twice), numeraire
`p_labour = 1`:

```
farm:  p_wheat·1 = 0.25·p_labour            ⟹  p_wheat = 0.25
mill:  p_flour·1 = 1·p_wheat + 0.25·p_labour ⟹ p_flour = 0.25 + 0.25 = 0.50
                                             ⟹  RealWage = 1/0.5 = 2.0
```

The price vector **is** the labour-content vector, which is the theorem the
scenario exists to test: `p = (labour 1.0, wheat 0.25, flour 0.5)`.

**Quantities.** Let `K_f`, `K_m` be the two scales, `N = 8`, `q` the basket.

```
wheat  :  K_f = 1·K_m                    ⟹  K_f = K_m = K
flour  :  K_m = N·q
labour :  0.25·K_f + 0.25·K_m = N·π = N  ⟹  0.5K = 8  ⟹  K = 16, q = 2
```

Two tiers (`tier 0: 1.0`, `tier 1: 3.0`) ⟹ `w* = 0.5`, and `parity` = 0.5/(1.0·0.5)
= **1.0** (which is again the corner: `real_wage = 1/(1·0.5) = 2 = 2·parity`,
`σ_π = +1`).

**Level, by the same construction as §3:**

```
M = b_cash·(p_flour·N·q)                              pop      = 6.5·8   = 52
  + (b_cash+1)·(0.25·K·p_labour)                      farm     = 7.5·4   = 30
  + (b_cash+1)·(K·p_wheat + 0.25·K·p_labour)          mill     = 7.5·8   = 60
  =  142        ⟹   p_flour = M/284, p_wheat = M/568, p_labour = M/142
```

**Stocks.** farm wheat `3·16 = 48`; mill flour `3·16 = 48`; mill wheat at the
decision point **0** (it buys `s = 1` tick's worth and consumes it the same
tick, so `planned_spend`'s `held` term is zero).

### 4.2 The tape (diffs from §2.3/§2.4 only)

```ron
goods: [
    ( name: "wheat",  alpha: 0.05, shelf_life: Indefinite, movement_type: Physical,  divisible: true, storage_cost_per_tick: 0.0 ),
    ( name: "flour",  alpha: 0.05, shelf_life: Indefinite, movement_type: Physical,  divisible: true, storage_cost_per_tick: 0.0 ),
    ( name: "gbp",    alpha: 0.0,  shelf_life: Indefinite, movement_type: Financial, divisible: true, storage_cost_per_tick: 0.0 ),
    ( name: "labour", alpha: 0.05, shelf_life: Instant,    movement_type: Local,     divisible: true, storage_cost_per_tick: 0.0 ),
],
recipes: [
    ( name: "wheat_farm",
      inputs:  [ ( good: "labour", qty_per_unit: 0.25, scaling: Variable ) ],
      outputs: [ ( good: "wheat",  qty_per_unit: 1.0 ) ],
      reversible: false, strategy: CapacityControl ),
    ( name: "grain_mill",
      inputs:  [ ( good: "wheat",  qty_per_unit: 1.0,  scaling: Variable ),
                 ( good: "labour", qty_per_unit: 0.25, scaling: Variable ) ],
      outputs: [ ( good: "flour",  qty_per_unit: 1.0 ) ],
      reversible: false, strategy: CapacityControl ),
    ( name: "dividend", ... as §2.3 ... ),
],
need_categories: [ ( name: "food", entries: [ ( good: "flour", weight: 1.0 ) ] ) ],
wealth_levels:   [ ( tier: 0, needs: { "food": 1.0 } ), ( tier: 1, needs: { "food": 3.0 } ) ],
market_nodes:    [ ( id: (0), tier: Regional, region: Some((0)), currency_good: Some((2)) ) ],
regions:         [ ( id: (0), name: "Solo", market_node: (0) ) ],
```

```ron
(
    tick: 0,
    prices: { "wheat": 0.25, "flour": 0.5, "gbp": 1.0, "labour": 1.0 },
    buildings: [
        ( id: (0), region: (0), recipe: "wheat_farm", recipe_size: 20.0, chosen_size: 16.0,
          efficiency: 1.0, inventory: { "gbp": 30.0, "wheat": 48.0 } ),
        ( id: (1), region: (0), recipe: "grain_mill", recipe_size: 20.0, chosen_size: 16.0,
          efficiency: 1.0, inventory: { "gbp": 60.0, "flour": 48.0 } ),
    ],
    pop_groups: [
        ( id: (0), region: (0), size: 8.0, wealth: 0.5, inventory: { "gbp": 52.0 },
          savings_target: 0.9, labour_good: Some("labour"), parity: Some(1.0) ),
    ],
    extra_recipe_instances: [
        ( recipe: "dividend", building: (0), output_pop: (0), recipe_size: 1000.0 ),
        ( recipe: "dividend", building: (1), output_pop: (0), recipe_size: 1000.0 ),
    ],
)
```

`criteria.ron` as §2.4 with `staple_good: "flour"`.

### 4.3 The predicted equilibrium

| | value |
|---|---|
| price vector | `labour 1.0, wheat 0.25, flour 0.5` |
| **RealWage** | **2.0** |
| cleared/tick | labour 8, wheat 16, flour 16 |
| scales | farm 16, mill 16 (util 0.8 each) |
| stocks | farm wheat 48, mill flour 48, mill wheat 0 |
| cash | farm 30, mill 60, pop 52, `M = 142` |
| Velocity | `(4+8+8)/142 = 0.140845…` |
| Concentration | 90/142 = 0.633802… |
| RealIncome | `(52/8)/0.5 = 13.0` |
| Employment / BldUtil | 1.0 / 0.8 |

**What only this scenario can catch.** A run can hit `RealWage = 2.0` with the
wrong `p_wheat` — the intermediate price is the free coordinate a one-good world
does not have. The gate is therefore on the whole vector, not on RealWage.

---

## 5. Scenario 3 — `solv_labour`: a labour market that must actually cross

**Purpose.** Phase 4 found `ε_supply = ε_demand = 0` on labour, with supply 0.41
against demand 73.7: no price clears that market. This scenario is built so that
the crossing exists, is interior, and is **provable** — and so that the answer is
*not* the labour-value vector, which makes it a falsification test for the
correctness battery itself.

### 5.1 Why a one-factor economy has no interior labour market

At the zero-profit price vector, labour's real wage is `1/embodied` and `parity`
is `0.5/embodied`, so `σ_π = +1` identically (R-C). Labour supply is vertical at
head-count. Meanwhile the desk's scale moves on margin until the margin is zero,
so **labour demand is horizontal at the technologically determined wage**. That
pair crosses — quantity from supply, price from technology — but the price is
doing no allocative work, and neither side has a slope. Scenarios 1 and 2 are
that case; and it is exactly the case Phase 4 diagnosed.

**The only way to get an interior crossing in this engine is to make something
other than labour scarce, and the engine has exactly one such object:
`recipe_size`, the capacity cap on `scale`.**

### 5.2 The design and the crossing proof

Set capacity below full-employment labour demand:

```
a = 0.5,   N = 20,   recipe_size = K = 20,   q* = K/N = 1.0
tiers: tier0 food 0.5, tier1 food 1.5   ⟹  q(w) = 0.5 + w,  w* = 0.5
parity = HOME_PENALTY/(0.5·0.5) = 2.0
```

**Labour demand.** `D(w_real) = a·scale`. For any real wage below the zero-profit
wage the margin is bounded away from zero (at the equilibrium below it is
`σ = 0.667`), so Rule 2 drives `scale` to its clamp: `D = a·K = 10`, a constant,
**strictly inside `[ε·N, N] = [0.2, 20]`**.

**Labour supply.** `S = N·π` with `π ∈ [ε, 1]` moving up when
`w_real > 1.05·parity` and down when `w_real < 0.95·parity` (R-C). So `S` is a
nondecreasing correspondence in `w_real` whose value set *at the reservation
wage* is the entire interval `[ε·N, N]`.

> **Crossing proof.** `D` is a constant lying strictly inside the range of `S`,
> and `S` attains every value in that range at `w_real = parity`. Therefore a
> crossing exists, at `w_real* = parity` and `L* = a·K = 10`, i.e.
> `π* = 10/20 = 0.5`, interior. Uniqueness: above the band `S = N = 20 > D`, so
> the labour market has excess supply and `p_labour` falls; below it
> `S = ε·N = 0.2 < D`, excess demand, `p_labour` rises. The sign of the
> restoring force is correct on both sides. ∎

**The wage, and why it is not the labour value.**

```
w_real* = parity   ⟹   p_labour /(q_tier0·p_grain) = 2.0
                   ⟹   p_labour = 2.0·0.5·p_grain = p_grain
                   ⟹   RealWage = 1.0,   against r = 1/a = 2
```

The firm earns the other half as a **capacity rent**: revenue `20·p_grain`
against outlay `10·p_labour`, profit `10·p_grain` per tick, routed out by Rule
3's overflow to the pop, who is the claim holder. **The predicted RealWage here
is 1.0, exactly half the labour-value answer.** A correctness battery that just
runs the Leontief solve will score this scenario wrong, which is the point of
including it.

**Level.** Firm cash at the decision point is
`(b_cash+1)·outlay + profit = 7.5·10·p_labour + 10·p_labour = 85·p_labour`;
pop cash at the decision point is `b_cash·cost = 6.5·20·p_grain = 130·p_grain`.
With `p_labour = p_grain`:

```
M = K·p_grain·(b_cash·a·θ + b_cash + 1) = 20·p_grain·10.75 = 215·p_grain
                                    θ = p_labour/p_grain = parity·q_tier0 = 1
⟹  p_grain = p_labour = M/215 = 1.0   for M = 215
```

The tick closes on the loop: firm 85 → overflow −10 → 75 → labour −10 → 65 →
grain revenue +20 → **85** ✔; pop 130 → dividend +10 → 140 → grain −20 → 120 →
wages +10 → **130** ✔.

### 5.3 The tape (diffs only)

Goods and the two recipes are `solv_1g`'s. Then:

```ron
wealth_levels: [ ( tier: 0, needs: { "food": 0.5 } ), ( tier: 1, needs: { "food": 1.5 } ) ],
```
```ron
buildings: [
    // chosen == recipe_size: the capacity cap is the scarce second factor and
    // MUST bind. This is the one scenario where BldUtil = 1.0 is correct.
    ( id: (0), region: (0), recipe: "grow_grain", recipe_size: 20.0, chosen_size: 20.0,
      efficiency: 1.0, inventory: { "gbp": 85.0, "grain": 60.0 } ),
],
pop_groups: [
    ( id: (0), region: (0), size: 20.0, wealth: 0.5, inventory: { "gbp": 130.0 },
      savings_target: 0.9, labour_good: Some("labour"), parity: Some(2.0) ),
],
```
prices: `{ "grain": 1.0, "gbp": 1.0, "labour": 1.0 }`.

### 5.4 The predicted equilibrium

| | value | note |
|---|---|---|
| `p_labour` = `p_grain` | 1.0 | reservation wage, **not** zero profit |
| **RealWage** | **1.0** | `parity·q_tier0`; labour value would say 2.0 |
| **π** | **0.5** | interior — the crossing |
| labour cleared/tick | 10 | `a·K` |
| grain cleared/tick | 20 | `N·q = K` |
| firm profit/tick | 10 | capacity rent, all paid out |
| cash (end of tick) | firm 85, pop 130, `M = 215` | |
| Velocity | 30/215 = 0.139534… | |
| Employment | 1.0 | supply is π-scaled, so fill = 1 |
| Concentration | 85/215 = 0.395348… | |
| BldUtil | 1.0 | cap binds |
| RealIncome | `(130/20)/1 = 6.5` | |

### 5.5 A pre-registered arm prediction

**Under `--agents legacy` this scenario has no rest point, and the failure is
computable.** `pop_agent.rs:32` posts `size·LABOUR_RATE = 20` unconditionally —
there is no participation margin — against demand fixed at 10. So
`imbalance = −0.5` every tick and

```
p_labour(t) = p_labour(0)·(1 − 0.5·α)^t     = (0.975)^t at α = 0.05
```

while `p_grain` stays put (the pop's income is wages + dividend = `20·p_grain`,
which still exactly funds its basket, so the grain market stays balanced).
RealWage decays geometrically to zero; `DRIFTING` fires enormously.
**If the legacy arm passes `solv_labour`, this analysis is wrong** — which is
the property that makes the prediction worth registering.

---

## 6. The correctness gate: two modes, and where the tolerance comes from

### Mode A — stationarity. Genesis **is** the equilibrium.

Run the tape exactly as specified. **Claim: the price vector never moves.**

Verified by hand across one tick of `solv_1g`: every rule's response is
identically zero — `stock − b_out·flow = 60 − 40 = 20` equals demand 20;
`imbalance = 0` exactly; `σ = margin/reference = 0` inside the dead band;
`fill_EMA = 0.25·1 + 0.75·1 = 1`, so no `SetDeskFill`; `π` already clamped at 1,
so no `SetPopParticipation`; `overflow = 75 − 65 − 10 = 0`, below the `1e-12`
gate; `price_ema` already equals `price`. The only genesis transient is
`AdjustInstanceBalance` settling the legacy `cs.balance` field to `−10` on tick
0 — a field the kernel never reads.

**Tolerance: `1e-12` relative, per good, per tick, over the whole run.** This is
float slack, not economic slack: every tape number is a dyadic rational (§1.2)
so the true answer is bit-exact, and the gate is ~12 orders of magnitude tighter
than any stability criterion in the repo. *Any* observed price motion is a
defect — either in the engine or in this note's derivation, and either is worth
knowing.

Mode A is invariant to `alpha` and to `--price-rule` (R-A). **That is itself a
check on the harness**: if a mode-A verdict changes when `alpha` changes,
something reads a dial it should not.

### Mode B — attraction. Genesis is displaced by a stated factor.

Second registration of each tape, identical except the genesis price of one good
is multiplied by 2 (`solv_1g`: `p_grain: 1.0`, so RealWage starts at 1.0 and
must climb to 2.0). Note this displacement leaves both markets *balanced* on
tick 0 — quantities are physical — so the run is driven purely by the desk's
margin, which is the cleanest possible excitation.

**Tolerance: `|ln(p̂_g/p̂_num) − ln(p*_g/p*_num)| ≤ ln(1.10) = 0.0953`, on the MEDIAN over
the analysis window.** Derivation, entirely from registered dials:

* the rest *set* is a band, not a point: `dead = 0.05` gives `R/C ∈ [0.951,
  1.051]` (R-B), so the correct price ratio is only defined to ±5%;
* the state variable is checked once per activation and moves multiplicatively,
  so it can sit up to one step of `eta_dn = 0.05` outside the band before being
  observed;
* `1.05 × 1.05 = 1.1025`, registered as **1.10**.

Median rather than mean or max, because §7 predicts a persistent orbit rather
than convergence, and the orbit is predicted to be centred on the fixed point.
The band is **reported, not gated** — gating it would be scoring stability
again, which is the thing this instrument exists to stop conflating.

### The sidecar the battery needs

Neither `criteria.ron` nor `Rule` can express "distance from a target vector",
so the family needs one new registered file per scenario, written before any run
(R6):

```ron
// equilibrium.ron — registered <date>, derived in docs/design/solvable-scenarios.md
(
    derivation: "docs/design/solvable-scenarios.md#4",
    numeraire: "labour",
    prices:     { "labour": 1.0, "wheat": 0.25, "flour": 0.5 },
    cleared:    { "labour": 8.0, "wheat": 16.0, "flour": 16.0 },
    real_wage:  2.0,
    velocity:   0.14084507042253522,
    mode:       FixedPoint,          // or Displaced( good: "flour", factor: 2.0 )
    tol_log:    1e-12,               // FixedPoint; Displaced registers 0.09531017980432486
)
```

Print `f64` as `{:.17e}` in any Python that touches these (Python's `{:e}`
truncates at 6 dp), and pass `encoding="utf-8"` explicitly in every file open.

---

## 7. What the family is predicted to find

Derived from the linearised rules; **stated as predictions, not results.** They
are the reason the family is worth building even if every scenario passes.

**7.1 The scale/price loop is anti-damped, and `b_out` is the term.** Let
`x = ln scale`, `y = ln(cost/revenue)`. Rule 2 is a relay: `Δx = −η·sgn(y)`
outside the dead band. The price rule is an integrator on the imbalance, and
R-E's buffer contributes `−b_out·Δx` to posted supply. Collecting terms with
`α_labour = α_grain = α`:

```
Δy = 2α·x + α·b_out·η·sgn(y)
V  = α·x² + η·|y|      ⟹      ΔV = +α·b_out·η²   per tick
```

The cross terms cancel exactly: **without the buffer the loop is conservative
(a centre — a displacement produces a permanent orbit, never convergence), and
the buffer makes it grow.** At `α = 0.05, b_out = 2, η = 0.045` that is
`2.0e-4` per tick, so the orbit's amplitude grows like `√t` — slow, but
unbounded over an 850-tick window. If `solv_1g` mode B shows a growing orbit
with `V` rising linearly in `t`, this is the mechanism; if `V` is flat, the
`b_out` term is wrong; if `V` falls, there is a damping term nobody has
identified and it should be found and named. **This is the cheapest available
test of the "no instantaneous supply elasticity ⟹ no damping" thesis in
`price-responsive-supply.md`, and it is quantitative.**

**7.2 Expansion is self-rationing between σ = 0.05 and σ = 0.26** (R-F). The
signature is a desk raising `chosen_size` and its *cleared* input volume falling
in the same tick.

**7.3 Money is exactly neutral, or R2 is violated** (§3). One extra tape, and
the failure names the leak.

**7.4 `Velocity` is measuring an unpinned direction** (§2.2) whenever the tape
registers a single wealth tier — including `tracer_2r` and, if the two-tier
reading of the corpus is wrong, more of it. The `solv_1g` / `solv_1g_money`
pair separates that cleanly: same real economy, one with the anchor and one
without.

---

## 8. What I am not sure of

Stated plainly, because the derivations above are only worth what their
assumptions are worth.

1. **Nothing here has been run.** Every number is hand-derived from the source
   at commit `a610c73`. The most likely place for an arithmetic slip is the
   genesis cash constants, since they encode R-F's `(b_cash/s + s)·outlay` and I
   have checked them by tracing one tick, not by execution.
2. **§7.1's anti-damping term is a linearisation of a relay system**, and relay
   systems are exactly where linearisation is least trustworthy. Treat the sign
   as the claim and the magnitude as an order of magnitude.
3. **`s = 1` is forced by §1.1 but it is a real departure from the corpus**, and
   it makes R-F's expansion-rationing 2.4× harsher than at `s = 4`. If mode B
   turns out to be dominated by that, the honest response is a registered
   `b_cash` sweep, not a quiet retune.
4. **Under `--agents legacy`, mode A cannot be exact and B6 fails by
   construction.** `building_agent.rs:54` caps posted supply at `1.2 × demand`
   using *last tick's* demand, which is 0 at genesis — so the legacy arm posts
   nothing on tick 0 and the grain market opens at `imbalance = +1`. And B6
   fires on every legacy scenario already, so the certificate's top-line verdict
   will be FAIL regardless of the prices. Legacy must therefore be read from the
   measurement table, not the verdict. That is a property of the existing
   arrangement, not of these scenarios, but it will be met immediately.
5. **Whether the two-tier level anchor is well behaved off-equilibrium.** It is
   locally restoring (cash above the band → tier up → basket up → demand up →
   price up), but the tier also changes the *real* basket, so the "level" anchor
   is not purely nominal off-equilibrium. On-equilibrium and under the
   neutrality test it is exact.
6. **The `parity` values are asserted to match `tools/derive_parity.py`.** I
   computed them by hand from that file's stated rule (0.5, 1.0, 1.0, 2.0 for
   the four tapes). Running the tool against the finished tapes is a cheap
   check and should be step one of writing them.
7. **`RawBuilding.transfer_target` and `.channel` are omitted above.** They are
   `Option` fields without `#[serde(default)]`; `tracer_2r` omits them and
   loads, so serde's missing-field-to-`None` path covers it — but if a loader
   error appears, that is the first thing to add back.
