# FREE-RULES: a zero price markets can hold, the free step, as built

Dated 2026-09-30. Step P2.4.11 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`, scratch
`D:/rustyecon-p24/build-free/`). It is the build of the free scan's choice,
[docs/probe/free/SPEC.md](free/SPEC.md) (sha256 `ef6c888f…3a04` as registered), registered at
P2.4.10 ([results/free/registration.md](results/free/registration.md)) before any of this code
existed; decisions 416–423. It says what was built, where the build reads the spec, and what was
checked. The spec's §6 is the specification; this file does not restate its argument, only what
the code does.

## 1. What changed, and what did not

- **Core: one optional field on a good, and zero allowed where it is set** (decision 416).
  - `RawGood.free: Option<RawFreeStep>`, `RawFreeStep { reference, scale }`: `reference` a good
    key, `scale` a param key (`Dimensionless`, live). Resolved `GoodDef.free:
    Option<FreeStep { reference: GoodId, scale: Site }>`. Both are `#[serde(default,
    skip_serializing_if = "Option::is_none")]`, core's second raw field with a default after
    `untraded`, so a tape without it keeps its canonical text, `tape_hash` and `world_id`.
  - `World::free_step(g)` and `World::price_allowed(g, v)`: finite, a clear sign bit, and above 0
    unless `g` has a free step, whose price or EMA may be +0.0 (never −0.0). It is the check of
    `apply`'s `SetPrice`, `SetEma` and `ScalePrice` (a free price scaled stays 0) and of the genesis
    price.
  - The checkpoint's book decode accepts a clean 0 (it has no world); `SimState::validate` refuses
    a zero price or EMA of a good without a free step, so a resumed checkpoint keeps the invariant.
- **Markets: the step and admission.** `next_price_free(one_sided, p, k, S, D, shift)` (§2);
  `update_prices` calls it for a free good with shift = c·p_ref, and `next_price` for every other,
  unchanged. Admission: a buy at a posted price of exactly 0 is feasible in full, whatever its
  budget. Clearing and settlement are unchanged: a trade at 0 pays 0·qty = 0, and no transfer is
  planned for it.
- **Certify: the kick.** `kick_segment` does not kick a market posting 0 at the kick tick, and
  records which markets are free (`KickSegment::free`); the verdict reads the gain through
  `kick_gain_free`, where equal prices of a free market (0 in both included) read a gap of 0.
  `kick_gain` is unchanged, so a 0/0 anywhere else still fails closed (N12).
- **Agents: one optional field on the workers' exit** (decision 419), and no new kind or state:
  `RawPricedExit.market: Option<Key>`, resolved `PricedExit.market: Option<GoodId>`, both skipped
  when absent. A pop with it is on a commons market (§3). The exit's support check now also reads
  a provider's `more` (decision 395), so several plot-taking pops can each be paid.
- **Probe:** `probe::markets` gains IL1 and CT2 (§4), unit 1e's point in wage units on the idle
  stretch and with several worker types, the harness in wage units at IL1 and at the commons
  market at CT2, the free-able market's runaway bound, mode A's reading of it, the readouts
  `free.*`, the grammar's `p[M]=V` and CT2's `exit.To`, the batteries and families, the
  elasticity probe's reading of a price of 0, and `markets point`'s columns. Every other
  instance keeps every number, name and file it had (§6.1).
- **The GUI:** the explainer computes a free good's step by `next_price_free`, and the waterfall's
  step is ln(p'/p) of the free step while the price is positive.
- **Tapes:** `tapes/markets-il1.ron` and `tapes/markets-ct2.ron`, `markets-tape --inst il1` and
  `--inst ct2`'s output.

## 2. The step

The spec's §6.1, as `crates/markets/src/prices.rs` makes it:

```
next_price_free(one_sided, p, k, S, D, shift):
  one-sided (exactly one of S, D is 0) and Hold:  p                          (0 stays 0)
  else:  kx = k·imbalance(S, D)
         q  = p·exp(kx) + shift·expm1(kx)       each product rounded once, then the sum
         q if q > 0 or q is NaN, else +0.0      (the good is free)
update_prices, a good with free = (reference, scale):
  shift = c·p_ref, c = scale's value now (a Value site), p_ref = the reference's posted price at
  the node, from the phase-start book
```

With shift 0 the step is `step`'s bit for bit (`free_step_absent_is_saturate`, 3,000 random
states). At 0 with S ≥ D it stays 0; with D > S it posts shift·expm1(kx) > 0. A NaN step is posted
as NaN so that the range check stops the run, as today. Nothing else floors, caps or smooths a
price (R3): the one nonlinearity is the price's domain, p ≥ 0, whose reading under R3 is decision
416's main risk. The EMA follows the posted price and may be 0 at a good with a free step (it
decays into the subnormals and stays there, since (1 − weight)·ema rounds back to the smallest
one).

## 3. The pops on a commons market

The spec's §6.3, as `crates/agents/src/roles/many/rules.rs` makes it
(`pop_market_participation`, `pop_market`; the scan's `fm.pop_decide`), in its order, on the pop's
own params (N_i, χ_i, s₀, s̲, h, T_o,i per tick) and posted w, P_s, p_g, r and r_o:

```
e0 = p_g·s0;  plot_at_r = r·h < p_g·(s0 − s̲);  r̂ = r if plot_at_r else p_g·(s0 − s̲)/h
if r_o < r̂:  F = F(fma(−r_o, h, e0)); hours = N·F; bid = h·(N − hours); T_p = 0
else:        F = F(fma(−r̂, h, e0)) if plot_at_r else F(p_g·s̲); hours = N·F
             G = h·(N − hours); bid = min(G, T_o,i); T_p = G − T_o,i if plot_at_r and G > T_o,i
mint hours of labour where > 0 and sell them; mint T_o,i of the commons where > 0 and sell it
P = min(r·T_p, C) where T_p > 0, else 0;  baskets' budget B = share(spend)·(C − P)   (398's)
the commons' budget = min(r_o·bid, (C − P) − B);  chain total = P + B + the commons' budget
buy T_p of land at budget r·T_p (in the chain), bid of the commons (in the chain), the baskets
state: Workers { share: F }
produce: eat the baskets; burn all land held and min(held, bid) of the commons (Consumption)
```

F(e) is P2.3's, min(max(ln1p((w − e)/(P_s + e))/χ_max, 0), 1). The commons' rent is paid from what
the baskets leave, as the mirror's baskets' budget leaves it out (registration §3): the baskets
leave (1 − share(spend))·(C − P), so the chain never cuts the rent below its want while the coin
covers it, and the orders are the mirror's. The bid is read again in produce at the same prices
and params, so the pop burns what it bought; the unsold part of its own share dies at ageing, as
the provider's unsold land does. Without `market` the exit's rule is P2.3's, untouched; `market`
with `pace` is refused.

At IL1 the roles gain nothing (the spec's §6.2): at r = 0 the workers' rule takes the plot branch
(0·h < p_g·Δ), its regime `Enclosed` in the rule's code, and rents T_p = h·(N − n(e₀)) on the idle
land with a budget of 0·T_p = 0, which admission fills in full; the provider, whose only income is
rent, holds no coin and pays min(N·P_s, 0) = 0.

## 4. The instances, the dials, genesis and tapes

- **IL1** (`Instance::named("il1")`; decisions 418, 421): I1's economy (C3's categories, 1a's
  machine, T 520 a year) with the workers one priced type in food, N 10.4 a year, χ_max 2, exit
  (s₀ 6, s̲ 0, h 2.7), a commons of 0 (`inst.commons` 0: no open parcel), and land free-able
  (`free.land`). Its cost coefficients are land.mach, b.food and `inst.land` (T a year: 572, 468,
  1040, 260). Its scalars' basis is the free scan's (`FREE_BASIS`), written `Assumed`.
- **CT2** (`Instance::named("ct2")`; decision 419): I1's economy with two plot-taking pops,
  `workers.wa` (C2's exit (0.2, 0, 0.09)) and `workers.wb` ((0.25, 0, 0.1125)), 104 a year each at
  χ_max 1 (`Instance::commoners`), sharing a commons of 19.5 a year, 9.75 each (`inst.<t>.commons`),
  traded on the market `commons` (`Instant`, `rate.commons`, `free.commons`). It has no `workers`
  pop and no `inst.workers` or `inst.chi_max`; the provider pays wa by `transfer` and wb by `more`.
  Its coefficients are land.mach, b.food and `exit.To` (the commons a year: 21.45, 17.55, 39,
  9.75), which sets each pop's share to V/2 (`Instance::changes_for`).
- **The oracle** (`Instance::parcel_point`): unit 1e's `ParcelEconomy`, at CT2 with one worker
  type per pop (`commoner_params`). `Point` gains `rent`, r in the point's units: 1 while land is
  scarce (every older point: `(w, r) = (v, 1)` as before, bit for bit), 0 on the idle stretch,
  where the point is in wage units (w = v = 1); `CommonsPoint` gains `market_land` (T_m) and
  `idle`. The plots' rent in the households' accounts is r·T_p (T_p at r = 1, as before). At CT2
  each pop's baskets are N_i + (w·S_i + r_o·(T_o,i − its commons plots) − r·its rented plots)/P_s.
- **The dials** (decision 417): `rate.commons`, 1.3 `RatePerYear` (land's; the mirror's
  `fm.dials`), after `rate.land` at CT2, a `rate.*` dial; and `free.<good>`, 0.5 `Dimensionless`,
  basis `Assumed("FREE-SPEC: the scan's window 0.3–1 at C2m, c 0.5")`, last in the dials at a free
  instance only, which no dial family scales.
- **Genesis** (the mirror's `fm.genesis`): at IL1, on the idle stretch, the prices are the point's
  in wage units, w 1 and land 0, each desk's human share 1 − x\* = 0, the provider's coin 0 and the
  workers' w·S/share(spend). At CT2, in rent units, the commons at the point's r_o, and each pop's
  coin r·T_p,i + (N_i·P_s + w·h_i + r_o·(T_o,i − bid_i) − r·T_p,i)/share(spend), its hours, bid and
  plots by `pop_market` at those prices; the provider's τ + (r·T − τ)/share(spend.provider) with
  τ = Σ N_i·P_s in pop order.
- **The tapes** write the free step on the free-able good (`free: Some((reference: "labour", scale:
  "free.<good>"))`), CT2's commons good and each pop's exit with `market: Some("commons")` last;
  the header (`free_header`) gives unit 1e's derivation, the land market and regime, T_m and
  T_idle, the provider's baskets and `funded`, each pop's genesis decision, and the free step.

## 5. The harness at a free instance

- **Observables** (decision 421): at IL1, in wage units, every price over w (the type's and the
  four categories'), each desk's threshold x_j = 1 − s_j, the cleared volumes of labour, land, the
  type and the categories, and each desk's output: 21, the mirror's `fb.targets_of` (the spec's
  §6.4 counts 20; registration §3). Land's volume target is the land in use, T_m + T_p. At CT2 the
  commons frame's 22 for I1's economy: the commons' price and volume are not observed, and its
  trade is out of the dead-tick rule (`Instance::observed_markets`), as the mirror's `fb.run`.
- **Starts**: at IL1 a price start's distance is over the prices over w and the thresholds;
  `p[land]=V` has distance ∞ (the mirror's); JA alone is a nominal start, never VACUOUS.
- **The runaway bound**: a free-able market's next price within [0, 1e6·max(its genesis price, its
  reference's)], the harness's genesis; every other price keeps [1e-6, 1e6] times its genesis.
- **Mode A**: its check does not read a free-able market's fills while it posts 0 (its sellers'
  rationing at 0 is its idle part), and still asks it to trade.
- **Grammar** (registration §3): `p[M]=V` sets market M's genesis price to V times labour's, after
  the factors; `inst.land=V@genesis|dated` at IL1; `exit.To=V@genesis|dated` at CT2, two changes at
  one tick; `coin.workers*F` at CT2 scales both pops' coins; `joint` draws the commons' price last
  (O106).
- **The battery**: IL1's 100 runs in the mirror's order and names (`idle_battery`): each price but
  land's at the six factors, market by market; `p[land]=0.00125`, `0.005`, `0.025`; each desk's
  share set to 0.05, 0.2 and 0.5; JA, JB, N and RC; x\*/2; the three coefficients at genesis and
  dated. CT2's 115, the commons battery's order. Tier 3S has 22 runs at IL1 (no provider's coin)
  and 24 at CT2; stocks 39 and 41; joint2 and joint4 as ever.
- **Readouts**, reported and never scored: the CSV adds `free_<good>_posted`, `_next` and
  `_ref_next`, and at CT2 `commons_regime` and each pop's `commons_bid_`, `commons_offer_` and
  `commons_net_` (r_o·(bid − offer)), through the pop's rule; `stats.tsv` adds `free.zero_ticks`,
  `free.first_zero`, `free.switches`, `free.end_over_ref`, `free.star_over_ref`,
  `free.regime_ticks` (by label), `free.regime_switches`, `free.regime_end`, `free.regime_star`
  and `free.provider_coin_low`. The regime is, at IL1, the workers' rule's with `Idle` where it
  says `Enclosed` at r = 0, and at CT2 the commons' posted price's (0 `Commons`, below r
  `Crowded`, else `Enclosed`; O129). At IL1 the commons' r_o/r reads 0 where r = 0, and its
  provider's lowest coin is the coin itself (its genesis coin is 0); the summary's `r_end` is r/w.
- **The elasticity probe** leaves out a market posting 0 at genesis (τ 0), as the mirror's
  `elasticity_free.py`, and gives L 56,000 (IL1) and 141,000 (CT2) at 52 a year, as registered.

## 6. Checks of the build

All on WSL in release unless said; scripts and outputs in `D:/rustyecon-p24/build-free/`.

### 6.1 R1: every old tape, id and stream

- **Streams.** Every committed tape before the build (the 28 of `tapes/`) run 2,000 ticks through
  the `rustyecon` binary gives the same `--hashes` file, header and stream as the binary built at
  `adf1ec6` before any change, on WSL and on Windows (`streams-*.log`). The two base streams equal
  each other.
- **Ids and text.** `free_field_moves_no_world_id` pins every committed tape's `tape_hash` and
  `world_id` and the markets tapes' 2,000-tick final hash to the pre-build binary's, checks that
  no canonical text writes `free`, and that I1 with a free step on land has another `world_id`.
- **The generator** writes every older markets tape byte for byte (their tests, unchanged).
- **Nesting.** `free_step_absent_is_saturate` (markets): with shift 0 the free step is
  `Imbalance`'s over 3,000 random states under both one-sided rules; a world without a free step
  moves by `step`; a free good with c set to 0 by `SetParam` moves by `step` bit for bit.

### 6.2 The points and the rest point

- `free_points_are_the_registered_ones`: at the 26 registered targets the harness's point is the
  scan's independent 50-digit solve (`registered/points_fine.json`) within 1e-13 in x\*, P_s/w,
  every price over w, Y, S, each pop's hours, T_m, T_p, r_o/w and r/w, with the same regime and
  land market; each CT2 pop's money account is the formula of §4, and the households' sum is Y.
- `il1_ct2_rest_at_the_oracle`: at the 26 targets and both bases at 12 and 365 a year, three ticks
  from genesis leave every observable within 1e-12 in log and every market trading, mode A's check
  passes, and the free-able market posts the oracle's price: exactly 0 where it is 0, within 1e-12
  of r/w or r_o/w where positive.

### 6.3 The rules and the harness

- Markets (`tests/free.rs`): `free_step_posts_zero_and_reopens` (0 in finite ticks, kept while
  S ≥ D, reopened to c·p_ref·expm1(kx), held under `Hold`, reopened under `Saturate`, a NaN step not
  posted as 0, the EMA into the subnormals); `free_market_takes_buys_for_nothing` (budget 0 and a
  positive budget at 0 filled in full, no coin moved; `max_qty` at a positive price);
  `only_free_goods_may_be_zero` (genesis 0 and −0.0, a step to 0, `SetPrice`/`SetEma` of 0,
  `ScalePrice` of 0, and the six load checks at their paths).
- Core: `a_free_price_of_zero_checkpoints_and_no_other_does` (decode and validate).
- Certify: `a_free_markets_zero_prices_read_a_gap_of_zero` (the mask, and 0/0 still NaN elsewhere).
- Agents (`tests/free.rs`): `pops_on_a_commons_market_rule` (3,000 random states on CT2's tape:
  hours, share, bid, plots against the rule written again from the spec, the offer, the land and
  commons budgets, the baskets, the state, admission, and produce's burns; every branch met);
  `commons_market_is_checked_at_load` (the market a basket good, the labour, the land or the exit
  good; not `Instant`; no free step; with a pace; a pop no transfer pays);
  `idle_land_is_taken_for_nothing` (IL1's plots at r = 0).
- Probe (`tests/free.rs`): `harness_reads_free_state` (IL1 at `p[land]=0.025`: free again at tick
  4, the zero ticks, switches and end read from the rows, `Idle`, the provider's coin, `r_end`, the
  commons' readouts at r = 0; CT2 at b.food × 2: 22 free ticks, then Crowded from the posted price,
  each pop's bid and offer summing to the commons' demand and supply; the commons out of the
  dead-tick rule at `exit.To=0`); `free_battery_and_families_are_the_registered_ones` (battery with
  tiers, Tier 3S, stocks, joint2, joint4, Tier 3, against `scanF.jsonl` and `regH.jsonl`);
  `free_grammar_applies_as_named`; `free_elasticity_and_kick_leave_out_a_free_market` (τ 0 at the
  free market, the registered L, and a kick set at IL1's rest that kicks every market but land and
  passes); `il1_ct2_tapes_are_their_generators_output`.
- GUI (`tests/pricestep.rs`): `the_explainer_and_the_waterfall_read_a_free_step` (the explainer
  equals the engine's next price on every tick and market of IL1 displaced, the free step's inputs,
  the waterfall's term and residual).

### 6.4 Each test fails without its change

`mutants/mutate_f.py` undoes each change in turn and runs core's lib tests, markets' `free`,
`prices` and `admission` tests, certify's batteries, the agents' `free`, `commons`, `many`, `seam`
and `pace` tests, the probe's `free`, `commons`, `wall`, `markets`, `trap` and `switch` tests, and,
for the GUI's two, its pricestep test. There are 75 mutants: 13 in core, 8 in markets, 4 in
certify, 18 in the agents, 11 in the probe's genesis and tape, 13 in its harness, 5 in its grammar,
1 in its elasticity probe and 2 in the GUI. Each is run on its own copy of the worktree, six shards
at a time (`mutants/shards.sh`).

- **The first pass** (`mutants-1-shard*.out`): 71 killed, four alive. The readout's end written as
  the posted price, not over its reference (`H9`), and `r_end` read over its genesis price
  (`H13`): no test read either at a run's end, so `harness_reads_free_state` now asserts CT2's
  free end and IL1's `r_end` at `inst.land=260`. A pop's genesis coin without the commons' rent and
  offer (`P9`): the pops' aggregate demand hid it, so `il1_ct2_rest_at_the_oracle` now asserts every
  actor's coin stationary within 1e-12 over its three ticks. The elasticity probe's skip of a market
  at 0 (`E1`) was equivalent, its τ already 0 there: the skip was removed, and the mutant moved to
  the rule that sets τ 0.
- **The final pass** on the final tree (`mutants-final-shard*.out`): all 75 killed.

### 6.5 Development runs before E0

The build was exercised on the engine before E0: `markets point` at both instances, `markets run
hold` for 300 ticks (mode A: D̂ 2.2e-13 at IL1 and 5.6e-13 at CT2, the free-able market at 0 on
every tick), and two development trace diffs on the uncommitted build with E0's eleven runs
(`e0/tracediff-dev1.out`, and `tracediff-dev2.out` with A1's checks). Every value of every run was
within 6.1e-14 in log of the registered traces, the free ticks exact in all eleven, and CT2's first
joint draw the mirror's within 4.4e-16 (libm's `pow` against glibc's, O110), but one: at IL1's
`JB(2)` tick 32 land's last positive price before it goes free (6.05e-5 of w) and the provider's
coin part by 1.06e-12. The machine desk's two budgets sum to an ulp more than its outlay, so the
chain cuts land's by 1.2e-17, which at r = 1.85e-4 of w is 6.57e-14 of land (N6); the free step,
whose next price there is the difference of numbers three times its size, multiplies the
imbalance's error by 150. The step recomputed from the engine's own inputs gives its next price
bit for bit on every tick. It is amendment A1's subject (P2.4.12), committed before E0 on the
committed build. Disclosed here, as the commons', the wall's, the trap's and the switch's were.

### 6.6 The gates

`scripts/gate.sh` and `scripts/gui.sh` are green on WSL (`/root/scratch/target-p24`) and on
Windows (`D:/rustyecon-targets/p24`), 1,053 tests passed and 4 ignored on each: P2.4.8's 1,035 and
this build's 17 outside the GUI, certify's counted twice as the gate runs certify's tests again
without Parquet; the GUI's own test runs in `gui.sh` (120 passed there). The gate hash is
`0x61f9c8529131ff17`, and gui.sh's hash check gives appb `0xe1fa082b26995867` and demo-gb
`0xfad880fe08d06645`, on both. The stamp is `99b1ad3`, dirty: this build before its commit. The
new tapes, on both machines: IL1's `tape_hash` `0x8c0b34885c544444`, `world_id`
`0x3e6c156dea5b795a`, 2,000-tick final `0xf42fcb94a6eb8fd3`; CT2's `0x677bc17e15c89d90`,
`0x5bd87c8fcdecef19`, `0x81fe3e335cb85a36`. Logs in `D:/rustyecon-p24/build-free/` (`gate-*.log`,
`gui-*.log`, `streams-*.log`). A first pass of the gates ran on the tree before the four tests were
strengthened after the mutants' first pass, green alike on Windows; the logs are the final tree's.

## 7. Readings and departures

- **The registration's readings** (registration §3), each taken: the field and its dial; the step's
  arithmetic, NaN posted; zero for a free good only, the checkpoint's book; admission; the load
  checks; the kick and the elasticity probe; the commons market's field, rule, orders, budget and
  produce; IL1 and CT2; IL1's 21 observables; the starts; the runaway bound; mode A and the dead
  rule; the readouts; Tier 3S and the families; E0's comparison; the GUI.
- **Certify's kick gain is masked, not changed.** The registration read "equal prices read a gap
  of 0" as a change of `kick_gain`; an existing test (`batteries_fail_closed_on_nonfinite_samples`)
  asks a kick whose prices fall to 0 in its tail to fail (N12), so the gap of 0 is read only at a
  market whose good has a free step (`kick_gain_free`, the segment's mask), and `kick_gain` is
  unchanged. For every committed world the two agree.
- **M6's list is unchanged.** The commons is `Instant`, checked where the pop's own labour is, so
  adding it to the goods a pop buys and does not net could not be seen by any test.
- **`Point`, `CommonsPoint`, `Displacement`, `Genesis`, `Row` and `Stats` gain fields**; the stocks
  probe's flow path (`horses::setup`) writes an empty `set` where it builds a markets setup; its
  genesis is unchanged (the horses' tapes and streams, §6.1).

## 8. How to run

```sh
markets-tape --inst il1 tapes/markets-il1.ron       # the committed tapes (and ct2)
markets point --inst il1                            # unit 1e's point in wage units, and every target
markets list battery --inst il1                     # the 100 runs with their tiers
markets family battery --inst il1 --ticks 56000 --jobs 46 --csv DIR
markets family tier3s --inst il1 --ticks 56000 --jobs 22 --csv DIR   # first-year D̂
markets family battery --inst ct2 --ticks 141000 --set 'rate.*=0.75' …   # the dial family
markets run 'p[land]=0.025' --inst il1 --ticks 56000 --csv DIR
markets kick hold --inst ct2 --ticks 141000 --horizon 141000
markets elasticity --inst ct2                        # L from the engine's τ, the commons left out
```

`stats.tsv` holds the free-able market's readouts as `free.*` lines, beside the commons' at IL1.
The scorer, its gather script and its job list are committed before the wave (decision 311).
