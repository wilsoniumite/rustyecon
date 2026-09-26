# RULES: the Phase 2 probe's agents, as built

Dated 2026-09-26. Step P2.0.1 on branch `phase2-probe`, based on `408b4e8`.

The probe asks one question early: can agents that decide at the pinning paper's margins reach
the oracle's equilibrium? The economy is the SSRN Appendix B flow instance, which oracle unit 1a
solves. The frame is `D:/rustyecon-probe/frame/PROBE-SPEC.md` (PROBE-SPEC below). It fixes the
economy, its mapping onto the engine, the known answer and the measurement protocol.

This file records what was built:

- the rules, as four behaviour kinds in `crates/agents/src/roles/` (§1 and §2);
- the dials, every one a tape param (§3), and the genesis they imply (§4);
- where the rules come from, and how the judges' conflicting grafts were resolved (§5);
- the harness, `crates/probe` (§6);
- how the build was checked (§7), and what stays open (§8).

## 1. The economy on the engine

One node, `home`, quotes in `coin`. Four goods trade there.

| good | life | who makes it | market rate |
|---|---|---|---|
| `labour` | Instant | `workers`, as an endowment in `decide` | `rate.labour` |
| `land` | Instant | `provider`, as an endowment in `decide` | `rate.land` |
| `mach` | Ticks(1), through `life.one_tick` | `desk.mach`, in `produce` | `rate.mach` |
| `good` | Ticks(1), through `life.one_tick` | `desk.good`, in `produce` | `rate.good` |

Four actors hold the roles. Each is in a class of its own, so rationing lines name the buyer.

| actor | kind | class | spec |
|---|---|---|---|
| `desk.good` | Desk | `good_desks` | `GoodDesk` |
| `desk.mach` | Desk | `mach_desks` | `MachDesk` |
| `provider` | Pop | `owners` | `Provider` |
| `workers` | Pop | `workers` | `Workers` |

The tick is ENGINE §7.2's. Every lag is one tick. Machine services made at t sell and are used
at t + 1. The good made at t sells and is eaten at t + 1. Coin received at t is spent from t + 1.
The provider's transfer lands in phase 1, after every `decide` has read its phase-start state.

The market rule is `Imbalance`, p' = p·exp(k·x) with x = (D − S)/max(S, D). The one-sided rule is
`Saturate`, as design-analytic-first registered it.

## 2. The four roles

Notation, per tick at posted prices:

- w, r, p_m and p are the prices of labour, land, mach and the good.
- P_s = p + h·r is the price of a basket: one good and h space.
- share(v) is `Clock::share`, 1 − exp(−v/ticks_per_year).
- C is the actor's coin as the phase began.

No rule reads a volume, a fill, another actor's holding, orders or state, or the oracle (R13).

### Workers

- **Participation.** s = min(ln1p(w/P_s)/χ_max, 1). This is F, the cdf of the uniform work cost
  χ ~ U[0, χ_max], at ln(1 + w/P_s). It moves instantly. The min is the support of χ, not a clamp.
- **Hours.** The workers mint N·s hours as an endowment and offer all of them. Only the offer is
  minted.
- **Baskets.** They spend B = share(`spend.workers`)·C. That buys n = B/P_s baskets. They post a
  buy of n of the good and a buy of h·n of land, from one budget.
- **Produce.** They eat min(good, land/h) baskets as `Consumption`. What is not eaten dies at 5a.
- **State.** `share`, the last s, a record.

### Provider

- **Land.** It mints T of land services and offers all of it. It buys its own space on the
  market, so the land volume is T.
- **Transfer.** It pays τ = min(N·P_s, C) to the workers. It records the due, N·P_s, and what it
  paid, τ, in its own state (R12: a short transfer rations the workers, and no market line shows
  that).
- **Baskets.** It spends share(`spend.provider`) of the coin the transfer left, as the workers do.
- **Produce.** It eats baskets, as the workers do.

### Good desk

- **Technique.** Its state carries the human share s = 1 − x, not x, so the approach to the
  corner x = 1 never cancels. The target is 1 − X, where X is the measure of the tasks i in
  [0, 1] on which a machine is cheaper at posted prices, γ(i)·p_m < w, γ(i) = η(g0 + g1·i^k):
  - X = ((w/(η·p_m) − g0)/g1)^(1/k) inside;
  - X = 1 when w/p_m ≥ γ(1), and X = 0 when w/p_m ≤ γ(0).

  These are the corners, reached without a clamp. Each tick the plan closes
  share(`adjust.technique`) of its gap: s ← s + a·(1 − X − s). At rest γ(x) = w/p_m, which is M3.
- **Unit cost.** c = s·w + J(x)·p_m, with J(x) = η(g0·x + g1·x^(k+1)/(k+1)).
- **Scale (the cash rule).** It spends O = share(v·μ^κ)·C on inputs, where:
  - v is the turnover `buffer.desk.good.cash`;
  - μ = p/c is its markup at posted prices;
  - κ is the tilt `tilt.desk.good`, registered at 0.

  Its planned output is q = O/c. It posts a buy of s·q hours (budget w·s·q) and a buy of
  J(x)·q machine services (budget p_m·J(x)·q). It offers all the good it holds.
- **Produce.** It makes y by Leontief at its planned technique:
  y = min(L/s, M/J(x)), over the inputs whose coefficient is not zero. At x = 1 or x = 0 exactly,
  the other input alone sets y. It burns s·y hours and J(x)·y machine services and mints y of the
  good, with life one tick. Leftover hours and machine services die at 5a.
- **State.** `share` (the plan), `used` (the share production used), `scale` (the last planned q,
  a record) and `output` (the last y).

### Machine desk

- **Unit cost.** c = λ·w + b·r, its purchased inputs only.
- **Markup.** μ = p_m·(1 − a)/c, the net form: one unit made earns p_m(1 − a) once its own input
  is kept.
- **Scale (the cash rule).** It spends O = share(v·μ^κ)·C and plans q = O/c. It keeps
  min(a·q, K) of the K machine services it made last tick and offers the rest. It orders λ·q hours
  and b·q land on the whole q (the judges' graft: an order capped at K/a loses the pro-rata contest
  for scarce inputs).
- **Produce.** y = min(K_held/a, L/λ, R/b). It burns a·y of its own machine services (last tick's
  lots, first), λ·y hours and b·y land, and mints y with life one tick.

### Why the rest point is the oracle's

The cash rule reads no margin. A desk's coin changes by its revenue less its outlay, so its coin
is constant only where revenue equals cost:

- the good desk rests at p = c, which is M2;
- the machine desk rests at p_m(1 − a) = λw + br, which is M1 in the net form.

The technique rests at M3 and participation at M4. The households and the transfer give M5 and M6.
No rule has a band. So the only live rest point is the oracle's (PROBE-SPEC §3.2), and every
R_o is 0.

### Arithmetic that cannot fail

These are the reservation design's hygiene notes, a judges' graft:

- A transfer or payout comes first, and is capped by the coin held.
- Each buy's budget is capped by what a copy of the holding still holds, and taken from it.
- The second budget of a pair is at most `num::max_remainder(outlay, first)`.
- Sells take from the same copy.
- Recipes use `num::max_scale`.

So no role posts an order that admission refuses, or a burn that falls short. This holds
whatever the prices, even when a share rounds to exactly 1. The test is
`roles_never_overbudget_or_overdraw`.

### Registered variants

Each variant is a field of the tape spec, off by default. Each was registered by the judges as an
alternative or a control (§5).

| variant | tape | harness flag |
|---|---|---|
| Ex-post task assignment: the cutoff x_u with L·J(x_u) = M·(1 − x_u), then Leontief at x_u | `assign: ExPost` | `--assign expost` |
| A payout above a ceiling | `Cash((.., payout: Some((to, rate, ceiling))))` | `--scale ceiling` |
| July's margin-step rule, the negative control | `Step((up, down, dead, buffer, payout, scale))` | `--scale step` |
| The markup tilt | `tilt.desk.*` ≠ 0 | `--set tilt.*=1` |
| `Hold` | the header's `one_sided` | `--one-sided hold` |

Details:

- **Ex-post assignment.** x_u is found by bisection over the bit patterns of the doubles, in the
  agents crate. It needs no core helper. When the root lies nearer x = 1, the rule resolves
  s = 1 − x_u in its own right, so a small s keeps its relative precision.
- **The ceiling payout.** The desk pays share(`payout.desk.*`) of its coin above
  e^`ceiling.desk.*`·target to the provider. The target is the value at posted prices of the
  output it holds to sell, net of its own input, over share(turnover). At rest that is the
  stationary coin, so the payout is inert there. A ceiling stated as a multiple of the cash rule's
  own outlay would never bind, because that target always equals the coin.
- **The margin-step rule.** The scale q steps by exp(step·(m ∓ d)) while the log margin m is
  outside ±d. The outlay c·q is capped by the coin less a payout of the coin above
  c·q/share(buffer). These are July's rule and design-analytic-first's ablation.

## 3. The dials

Every dial is a tape param with a unit and a basis (R4). Per-tick values are at 52 ticks a year.

| key | unit | value | per tick | role |
|---|---|---|---|---|
| `rate.labour` | RatePerYear | 5.2 | k = 0.1 | labour's price rate; the fastest price |
| `rate.good` | RatePerYear | 2.6 | k = 0.05 | |
| `rate.land` | RatePerYear | 1.3 | k = 0.025 | |
| `rate.mach` | RatePerYear | 1.3 | k = 0.025 | |
| `adjust.technique` | RatePerYear | 2.6 | share 0.048771 | the technique's partial adjustment |
| `spend.workers`, `spend.provider` | RatePerYear | 13 | share 0.221199 | household spending |
| `buffer.desk.good.cash`, `buffer.desk.mach.cash` | RatePerYear (turnover) | 5.2 | share 0.095163 | desk outlay per coin |
| `tilt.desk.good`, `tilt.desk.mach` | Dimensionless | 0 | — | markup tilt (graft) |
| `price.ema_tc` | Years | 0.5 | weight | the header needs one; no rule reads the EMA |
| `life.one_tick` | Years | 1/52 | 1 tick | the life of mach and the good |

- **Bases.** The `rate.*`, `adjust`, `spend` and `buffer` dials are
  `Assumed("design-analytic-first C2, registered 2026-09-26")`. The tilts are
  `Assumed("judges' graft from design-reservation")`. The instance's params (`inst.*`) are
  PROBE-SPEC §1.5's, with their `Literature` basis.
- **The ordering.** labour > good > land = mach, so the wage is the fast price. All three judges
  took this from their cross-design evidence. It is recorded here as the structural condition,
  with the tilt as its registered substitute.
- **Tolerance.** No dial is a band, so tol_o = 1e-3 (`probe.tol_floor`) on every observable.
- **Run length.** The slowest time constant is 1/(k_good·0.2954) = 68 ticks, below L/200, so
  L = 20,000.

Variant dials are registered only when the variant is on:

- `payout.desk.*` = 26/yr and `ceiling.desk.*` = ln 2, for the ceiling payout
  (design-adaptive's values);
- `step.desk.*.up` = `.down` = 2.6/yr, `dead.desk.*` = 0 and `payout.desk.*` = 26/yr, for the
  negative control (design-analytic-first's ablation).

## 4. Genesis: `tapes/appb.ron`

The tape is generated, not written by hand. The generator is
`cargo run -p rustyecon-probe --bin appb-tape -- tapes/appb.ron`. The test
`appb_tape_is_its_generators_output` checks the committed file against it, byte for byte.

Genesis is the oracle's point (PROBE-SPEC §4.6):

- **The solve.** `crates/oracle`, unit 1a, at N = 208/52 = 4, T = 520/52 = 10 and b = 0.4, with
  (ρ, δ, J_b) = (0, 1, 1).
- **Prices.** The oracle's ratios with r = 1 coin.
- **Technique.** The good desk's human share is 1 − x\*.
- **Stocks.** The good desk holds Y\* of the good and the machine desk K\* of machine services:
  one tick's output each.

Each actor's coin is its stationary balance: what it pays out per tick at the point, over its
share.

| actor | formula | coin |
|---|---|---|
| desk.good | p·Y/share(5.2) | 29.94282457255491 |
| desk.mach | (λw + br)·K/share(5.2) | 23.78417887464776 |
| provider | N·P_s + (rT − N·P_s)/share(13) | 26.03270836105176 |
| workers | (N·P_s + w·N·F)/share(13) | 27.922094099033625 |

These equal design-analytic-first's closed forms. The oracle is solved outside any Sim: the
generator writes its outputs as genesis data, and no agent reads it at run time (R13). The
tape's header states the derivation.

## 5. Lineage

**The design.** Three designs were judged:

| judge | analytic-first | reservation | adaptive |
|---|---|---|---|
| economist | 8 | 7 | 6.5 |
| dynamics | 8 | 6 | 4.5 |
| R3 | 8.5 | 6.5 | 5.5 |

All three judges chose design-analytic-first. Its registered dial set, C2, is built as registered.
The rules are PROBE-SPEC §1.3's structure with analytic-first's cash rule. There are no bands,
steps, stagger or payouts.

**Grafts adopted:**

1. **Leontief at the planned x as the default** (R3 judge), with analytic-first's ex-post
   assignment as the registered alternative. This keeps PROBE-SPEC §1.3's text and needs no core
   helper. Two mirrors agree it passes: analytic-first's own, PL 0.98694 and 57/57, and
   reservation's, 0.98703.
2. **The markup tilt share(v·μ^κ)** as a registered dial at κ = 0 (all three judges). κ = 1 is a
   reported cell.
3. **The rate ordering** as the registered structural condition (all three judges).
4. **The machine desk orders on the uncapped q, with keep = min(a·q, K)** (economist, dynamics).
5. **The technique carried as 1 − x** (dynamics).
6. **Arithmetic hygiene** (reservation, through all three judges): budgets by `max_remainder`,
   zero-coefficient inputs dropped from the Leontief min, `Eq` dropped on `AgentDelta`, and state
   values checked for non-finite values and `-0.0`.
7. **A ceiling payout** as a registered variant, with a target of its own (economist, R3).
8. **July's margin-step rule** as the registered negative control, run in the same engine (R3).
9. **Harness reading** (reservation): mode A's fills and spoilage are read relative to volume,
   and stock displacements get a D̂_0.
10. **Families** (adaptive, economist):
    - the stock family: 21 runs, adapted from adaptive's 22, plus K ×0.01, K ×10, and K ×0.1 with
      the machine desk's coin ×0.1;
    - joint random displacements: 60 within ×/÷2 and 40 within ×/÷4;
    - the minimum cleared volume per market over each run, and its tick (the trough diagnostic).
11. **Trace diffs against the mirror before any scoring** (adaptive, through all three judges).
    §7 has the result.
12. **A shock history** in its expressible form (dynamics): b cycled 80 times through the
    battery's values, `bcycle(1500,80)`.

**Conflicts, and how they were resolved:**

- **Ex-post or planned assignment.** The economist valued ex-post assignment as task economics.
  The R3 judge wanted planned-x Leontief as the default, because it follows §1.3 and needs no
  helper. The dynamics judge found the fallback sound. **Resolved:** planned x is the default,
  and ex-post is built as the alternative. Two of three judges prefer this, and the frame's text
  agrees.
- **Desk turnover ×0.25.** The dynamics judge proposed it, to be adopted only after a fresh
  registration and battery. The R3 judge said to freeze C2 before the first Rust mode-B run.
  **Resolved:** C2's turnover of 5.2/yr is registered. ×0.25 is a reported cell
  (`--set buffer.*=0.25`), not the default. Adopting it would take its own registration and
  battery, and the frame forbids changing dials once mode B has run.
- **Payouts.** The economist wanted a reported variant. The dynamics judge made it conditional on
  the shock history. The R3 judge noted that a ceiling tied to the cash rule's target never binds.
  **Resolved:** the payout is built as a variant with its own target (§2), off by default, and
  the harness reports it.
- **1 − x or log-odds.** The dynamics judge said either. The R3 judge said log-odds only with an
  R10 receipt. **Resolved:** 1 − x is carried. Log-odds is not built.
- **Saturate or Hold.** Analytic-first registered Saturate, claiming that the x = 0 corner absorbs
  under Hold. Two judges did not see that. **Resolved:** Saturate stays registered, as the design
  had it, and Hold is a flag.
- **The map's axis.** Adaptive's band axis is degenerate here: only the negative control has a
  band. The R3 judge's map is price rates ×{0.5, 1, 2} crossed with desk turnover ×{0.5, 1, 2}.
  **Resolved:** the harness takes `--set rate.*=F --set buffer.*=G`, which is the R3 judge's map.

**Grafts not built:**

- **80 sequential ×2 price displacements.** No tape action sets a price, and E1 forbids writing a
  run's state. Only the b history is expressible on the tape (§8).
- **The predictor check.** The local rate against battery outcomes over a dial grid needs a
  linearisation the harness does not compute. The designs did it on their mirrors.
- **A dedicated nominal-hysteresis run.** Coin factors on two actors approximate it, for example
  `coin.provider*0.999999+coin.desk.good*1.000001`. A transfer of exact size between two actors
  is not a perturbation term.

## 6. The harness: `crates/probe`

The crate depends on the engine, the oracle, and core's read-only helpers (`Clock`, the units
and `num`). Nothing depends on it. The binaries build under WSL with
`CARGO_TARGET_DIR=/root/scratch/target-probe-<label>`.

```sh
probe run NAME...    [options]   # named runs, one summary line each
probe family FAMILY  [options]   # battery (57), stocks (21), joint2 (60), joint4 (40), history
probe list FAMILY                # names, with the tier for battery runs
probe tape [NAME]    [options]   # the tape a run is made from
options: --ticks L --tpy N --set KEY=VALUE --assign planned|expost
         --scale cash|ceiling|step --one-sided saturate|hold
         --csv DIR --every K --jobs J
```

**Run names.** A name is one or more terms joined by `+`:

- `hold` is mode A.
- `w*F`, `r*F`, `pm*F`, `p*F` and `s*F` displace one genesis price, or the human share.
- `JA(F)`, `JB(F)` and `N(F)` are PROBE-SPEC §4.7's joint shapes, and `x*/2` halves x.
- `b=B@genesis` and `b=B@dated` are the cost shocks. A dated shock fires at L/4, and the scored
  clock restarts there.
- `coin.ACTOR*F`, `good*F` and `mach*F` displace a stock.
- `joint(F,SEED)` draws a random joint displacement from a fixed generator.
- `bcycle(P,C)` makes C dated changes of b, P ticks apart.

**Output.** With `--csv DIR` each run writes `DIR/<run>.csv`, one row per tick (or every K-th
tick and the last). Each row carries:

- the posted prices;
- the ten observables of PROBE-SPEC §4.3;
- the oracle's value of each for that tick, solved in the harness at the b in force;
- each gap |ln(o/o\*)| and D̂;
- S, D, both fills and spoilage per market;
- each actor's coin;
- the planned human share;
- the provider's transfer, due and paid;
- the ledger margin;
- the dead flag.

`DIR/summary.tsv` has one line per run with:

- the class (PROBE-SPEC §4.5);
- D̂_0, E_1 to E_4, κ and the largest D̂ over W;
- the tick from which D̂ stays within tolerance;
- dead ticks;
- July's band per relative price;
- r at the end;
- the trough per market;
- rationed ticks per market side;
- spoilage;
- the transfer shortfall;
- mode A's PASS or FAIL.

Put large outputs under `D:/rustyecon-probe/runs/`.

**The classifier.** It follows PROBE-SPEC §4.5 with tol_o = 1e-3. There are three departures:

- The numeraire fallback to w on ticks when land is not live is not implemented. Those ticks are
  dead ticks anyway.
- Stock runs get D̂_0 = max |ln f| over the displaced stocks, over tol.
- A run that stops at the runaway bound, [1e-6, 1e6] × genesis, is DIVERGED at that tick.

**The protocol's numbers** are in `probe::protocol`:

- the tolerance floor, 1e-3;
- the hold bar, 1e-9;
- the live floor, 0.5;
- the runaway bound, 1e6;
- L = 20,000.

**Registration comes first.** PROBE-SPEC §4.10 freezes the rules, dials, tolerance, L and
perturbation list, with the commit, before the first mode-B run that counts. This file and
`tapes/appb.ron` at the P2.0.1 commit are that content. A registration file names the commit.

## 7. Checks of the build

All runs below are on WSL, in a release build of the P2.0.1 tree, at 52 ticks a year.

**Trace diff against the design's own mirror** (a judges' graft, before any scoring).
`D:/rustyecon-probe/build/tracediff.py` ran `design-analytic-first/model.py` at C2, with planned-x
Leontief and Saturate, from each Rust run's own genesis. It compared the ten observables every
tick for 2,000 ticks. The output is `D:/rustyecon-probe/build/tracediff.out`.

- **Hold, w×2 and N(2)** agree to at most 1.4e-14 in log.
- **JB(0.5) and b = 0.2 at genesis** differed from tick 1, by up to 1.3e-2.
  - The cause: the genesis lot of machine services has life 1, so what the desk kept at tick 0
    and a labour ration left unused lives one more tick in the engine. The mirror drops it. This
    is PROBE-SPEC §1.2's "genesis stock sells twice", and the designer listed it as a known
    difference.
  - With that one carry added, in a copy (`D:/rustyecon-probe/build/model_genesis2.py`), all five
    runs agree to at most 1.44e-14 over 2,000 ticks.
- **So the build is the design's map.** The engine alone decides the genesis lot's second tick.

**Mode A.** The registered tape holds for L = 20,000 ticks:

- the largest gap is 8.9e-16 in log;
- every fill is at least 1 − 1e-9;
- spoilage is at rounding level;
- the ledger is clean.

It holds for 20,000 ticks under every variant and at 12 and 365 ticks a year:

| setup | largest D̂ over W | mode A |
|---|---|---|
| registered | 8.9e-13 | PASS |
| `--assign expost` | 1.9e-12 | PASS |
| `--scale ceiling` | 8.9e-13 | PASS |
| `--scale step` | 3.3e-13 | PASS |
| `--one-sided hold` | 8.9e-13 | PASS |
| `--tpy 12` | 1.8e-12 | PASS |
| `--tpy 365` | 8.9e-13 | PASS |
| `--set tilt.*=1` | 8.9e-13 | PASS |

The tests `appb_holds_at_the_oracle_point` (20,000 ticks) and
`appb_variants_hold_at_the_oracle_point` (2,000 ticks each) pin it.

**Determinism and platforms.** These were run on 2026-09-26, with outputs in
`D:/rustyecon-probe/build/xplat/`.

- `rustyecon run tapes/appb.ron --until 20000 --hashes` gives the same 20,000 per-tick hashes on
  WSL and on Windows. The final hash is `0xe1fa082b26995867`.
- The harness's CSVs for `w*2` and `JB(0.5)` over 3,000 ticks are byte-identical on the two
  platforms.
- The gate world's hash stream is unchanged, with final hash `0x61f9c8529131ff17` on both.

**One mode-B battery was run** before any registration file existed.

- **What.** The 57 runs of PROBE-SPEC §4.7, at the registered C2 dials, L = 20,000, as a smoke
  check that the harness runs end to end.
- **Where.** The outputs and their conditions are in `D:/rustyecon-probe/runs/build-smoke/`.
- **Nothing was changed after it.** No dial, rule or tolerance moved, and the tape above is the
  one it ran.
- **Its classes.** All 57 runs came out CONVERGED, with every final D̂ below 1.1e-11.
- **Its standing.** It is disclosed here as a run made before registration, not as the probe's
  verdict. The verdict belongs to a registered battery (§6).

## 8. Open

- **A price-shock history.** 80 sequential joint price displacements, the dynamics judge's
  history gate, needs a tape action that sets a posted price, or an agreed way to resume from an
  edited state. Neither exists (E1). The b history does exist.
- **The wedge solver** of PROBE-SPEC §3.3 (a) is not built. The registered rules have no bands,
  so no run needs it. The negative control's registered band is 0.
- **The oracle is solved at the start and at each shock only.** This suffices, because the
  instance changes only through `inst.b`.
- **The schema stays 1.** The four kinds are new variants of the agents' spec, and every schema-1
  tape loads and means what it did (docs/TAPE.md, the row for P2.0.1).
