# COMMONS-RULES: the open-commons instances' agents and harness, as built

Dated 2026-09-30. Step P2.3.6 on branch `phase2-proper` (worktree `D:/rustyecon-wt/p23`, scratch
`D:/rustyecon-p23/build-commons/`). It is the build of the commons frame,
[docs/probe/commons/SPEC.md](commons/SPEC.md) (sha256 `60f21f56…b40d` as registered), registered
at P2.3.5 ([results/commons/registration.md](results/commons/registration.md)) before any of this
code existed. It says what was built, where the build reads the frame, and what was checked. The
frame's §3 is the specification; this file does not restate its argument, only what the code does.

## 1. What changed, and what did not

- **Core, markets and the engine: nothing.**
- **Agents: one optional field**, the workers' `exit` (decision 398), and no new kind, state or
  market rule:
  - `BasketWorkers.exit: Option<(good, gross, floor, plot, commons, land)>`: the exit good g (a
    basket good), s₀, s̲ and h (`Dimensionless` params), the commons T_o (a `FlowPerYear`
    param), and the land plots rent on enclosed land.

  It is `#[serde(default, skip_serializing_if = "Option::is_none")]` on the raw and on the
  resolved spec, as the type desk's `plant` and the wall's fields are, so a tape without it keeps
  its canonical text, `tape_hash` and `world_id`. `WorkersState { share }` is unchanged; the
  share it holds under the exit is §2's F. The M6 check (a bought good dies within the tick) now
  lists the plots' land among the workers' bought goods.
- **Probe:** `probe::markets` gains the open-commons instances C1, C2 and the negative control
  C1N, solved by unit 1e's `ParcelEconomy`, their genesis and tapes, the targets at the commons,
  the plots' readouts, the run grammar's `commons=V` and `enclose=F`, the commons' battery and
  families, and a share scaled past 1 set to 1 at the commons. P2.1's and the wall's instances
  keep every number, name and file they had (§6).
- **Tapes:** `tapes/markets-c1.ron` and `tapes/markets-c2.ron`, `markets-tape --inst c1` and
  `--inst c2`'s output. The negative control is `--inst c1n`, with no committed tape.

## 2. The workers' rule

The frame's §3.1–§3.2, as `crates/agents/src/roles/many/rules.rs` makes them. With `exit` absent
the workers take P2.1's path, unchanged. With it, each tick, at posted prices w (labour), r (the
plots' land), p_g (the exit good) and the basket's P_s, N and T_o per tick:

```
Δ    = s₀ − s̲
F(e) = clamp(ln1p((w − e)/(P_s + e))/χ_max, 0, 1)        the support of χ, not a clamp on a price
n(e) = N·F(e)
e₀   = p_g·s₀;   a plot pays at r where r·h < p_g·Δ (each product rounded once)
e_r̂  = fma(−r, h, e₀) where a plot pays at r, else p_g·s̲
Sc   = N − T_o/h
hours = n(e₀)       if n(e₀) ≥ Sc                  Commons: the plots fit the commons, free
      = Sc          else if n(e_r̂) > Sc             Crowded: the commons full, a shadow rent
      = n(e_r̂)      else, with T_p = h·(N − n(e_r̂)) − T_o where a plot pays at r (Enclosed)
                    and T_p = 0 where it does not (the split)
```

A pop that takes no plot (h = 0 or Δ ≤ 0) offers n(p_g·s̲). `workers_participation` forms all of
this from the workers' spec, a param reader and a price reader, and the rule calls it, so an
observer outside the Sim reads the same numbers (§4). Its evaluation order is the frame's: P_s
summed from 0.0 in item order, e₀, the branch test, e_r̂ by `num::fma`, each n by `num::ln1p`,
then Sc.

- **The hours** are minted and offered in full, as P2.1's.
- **The plots on enclosed land.** Where T_p > 0 the workers post one buy of T_p land at r in the
  same `budget_chain` as their baskets. With C their coin as the phase began and P = min(r·T_p,
  C) the rent the coin covers, the baskets' budget is share(spend)·(C − P) and the chain's total
  P + share(spend)·(C − P): the rent first, the baskets from the rest, and admission refuses
  nothing. Where T_p = 0 no land order is posted and the baskets' budget is share(spend)·C,
  P2.1's.
- **Produce** eats the baskets as before and burns every unit of the plots' land the workers hold
  as `Consumption`, so none of it spoils.
- **The state** holds the regime's F: F(e₀) in Commons, Sc/N when Crowded, F(e_r̂) when Enclosed
  or split, F(p_g·s̲) with no plot. That is hours/N to one rounding (the frame writes hours/N;
  registration §3), and with the exit switched off (s₀ = s̲ = 0) it is P2.1's share to the bit.

The rule reads posted w, r, p_g and the basket's prices, and its own params (N, χ_max, s₀, s̲, h,
T_o); no fill, volume, other actor or oracle (R13). No price is floored, capped or smoothed (R3):
the bounds of F are its support, Sc is the commons' capacity, and the branch on r·h is the rent
at which a plot stops paying.

## 3. The instances, their dials, genesis and tapes

- **C1, C2 and C1N** (`Instance::named`; decision 398) are I1 (C3's categories, 1a's machine, N 208
  and T 520 a year) with the pool's workers one priced type, exit good food:
  C1 exit (0.3, 0, 0.135) and a commons of 24.3 a year; C2 (0.2, 0, 0.09) and 31.2; C1N is C1 with
  χ_max 0.25 and its own point. The exit's params are `inst.exit.gross`, `inst.exit.floor`,
  `inst.exit.plot` and `inst.commons`, written `Assumed` with the frame as their basis; I1's keep
  their bases. The cost coefficients are `land.mach`, `b.food` and `commons` (param
  `inst.commons`, a year), at the frame's values (C1 26.73, 21.87, 48.6, 12.15; C2 34.32, 28.08,
  62.4, 15.6).
- **The oracle** is unit 1e (decision 399): `Instance::parcel_params` gives 1d's economy with one
  worker type (N, χ_max, ε 1, ν 1), the enclosed land T and the commons T_o as parcels of quality
  1 per tick (a commons of 0, all of it enclosed, is no parcel), the type's exit
  `ExitForm::Priced`, and food the exit good. `Point::commons` carries 1e's readouts: the regime
  (`ExitLand`), the land market, the plot rent r_o, the rented plots T_p, the commons used, the
  exit value, funded and certified.
- **The dials** are C2m, I1's (decision 119).
- **Genesis** is 1e's point: prices its ratios with r = 1, every desk's share 1 − x\*, one
  tick's output in stock, and stationary coins as P2.1's, the workers' r·T_p + (N·P_s + w·S −
  r·T_p)/share(spend), with S the point's supply and T_p its rented plots (P2.1's formula where
  T_p = 0).
- **The tapes'** headers state the derivation from unit 1e (`commons_header`). The workers' spec
  line adds `exit: Some((good: "food", gross: "inst.exit.gross", floor: "inst.exit.floor", plot:
  "inst.exit.plot", commons: "inst.commons", land: "land"))`.

## 4. The harness at the commons

An instance with an exit (`Instance::exit`) switches the harness's commons reading on; P2.1's and
the wall's instances take none of it.

- **Observables** are P2.1's 22 for I1's economy (v, the type's and each category's price, each
  desk's share s_j, every market's cleared volume, each desk's output), every tolerance the 1e-3
  floor. **Targets** are unit 1e's: labour's volume is the supply S at the point, land's the
  enclosed land T of the instance in force (it moves under `enclose`), each good's z_j·Y.
- **Each household's target baskets** are its money accounts at the point, the provider's
  (T_m + T_p)/P_s − N and the workers' N + (w·S − T_p)/P_s, which sum to Y; they feed the
  binding readout only (O102).
- **The plots' readout.** Each tick the harness forms the workers' participation through
  `workers_participation` from the resolved spec, the params in force and the prices the tick
  settled at, and keeps on its `Row` the regime, the shadow rent over r and T_p (`Plots`). The
  shadow rent is 0 with room, r when the plots spill, and when the commons is full (or at the
  split) the plot rent at which the rule's supply is its hours: e = (w − k·P_s)/(1 + k),
  k = expm1(χ_max·hours/N), r_o = (p_g·s₀ − e)/h, at most r (`shadow_rent`, the mirror's
  `cm.shadow_rent`). Nobody receives it; no agent reads it.
- **The commons' readouts** (`Stats::commons`, `stats.tsv`'s `commons.*` lines), reported and never
  scored: ticks in each regime (unused, Commons, Crowded, Enclosed, and the split apart), switches
  between the mirror's labels (the split counted as Crowded, as `cm.exit_rule` labels it), the
  regime at the end and the target's, r_o/r's lowest, highest and last and the target's, T_p's
  largest, and the provider's lowest coin over its genesis coin. `summary.tsv` keeps P2.1's 30
  columns. The CSV adds `part_workers`, `regime`, `ro_over_r` and `plots_rented`.
- **A cost shock's start distance** at the commons is the largest |ln| over every observable of
  the two targets, over the tolerance (`target_distance`), as the mirror's `battery_c.run`: C2's
  commons × 1.1, × 0.9 and × 2 read 0 and are VACUOUS.
- **Run grammar** (registration §3): `commons=V@genesis|dated` sets `inst.commons`, a year, as any
  cost coefficient; `enclose=F@genesis|dated` moves F·T_o of the commons to the enclosed land,
  `inst.commons` to T_o − F·T_o and `inst.land` to T + F·T_o, both from tick 0 or both at L/4, as
  `cm.shocked` computes them. `joint(F,SEED)` draws in the mirror's order (labour, land, the
  categories, the type). A share scaled past 1 is 1, as the mirror's `displace`; P2.1's instances
  keep their refusal.

  *Amended at P2.3.12 (2026-09-30, label `run`, during the scored wave):* a dated shock is an
  event that sets its param from a schedule param, and the tape writer wrote every schedule param
  `Dimensionless`. `inst.commons` and `inst.land` are `FlowPerYear`, so every tape with
  `commons=V@dated`, `enclose=F@dated` or `cycle(commons,P,N)` was refused at load ("cannot set a
  FlowPerYear param from a Dimensionless param"): 54 of the wave's 3,721 jobs, each an error
  before its first tick. No test loaded such a tape (`commons_grammar_applies_as_named` checked the
  shocks' list, not the tape), and E0 ran genesis shocks only. The schedule param now carries the
  unit of the param it sets, so every other tape is the same text;
  `commons_dated_shocks_load_and_fire` fails without it. The 54 jobs were run again on the fixed
  binary (results/p23-wave/README.md, "The rerun").
- **The battery** (`commons_battery`): the mirror's `battery_c.run_list` in its order and names:
  each market's price at the six factors, market by market (labour, land, the categories, the
  type; `p[labour]*1.05`); each category desk's share at the six factors; JA, JB, N and RC at the
  six factors; x\*/2; the three coefficients at their four values, at genesis and dated. 115 runs,
  30, 42 and 43 by tier; the slack runs are P2.1's `slack` (every s[manufactures] run, s[care] ×
  1.05, × 1.2 and × 2).
- **Families:** `tier3s` (24: each desk's stock and coin, the workers' and the provider's coin, at
  ×0.5 and ×2; decision 368), `stocks` (P2.1's, 41), `joint2`, `joint4`, `history`
  (`cycle(land.mach,1500,80)` and `cycle(commons,1500,80)`), `enclose` (0.5 and 1, at genesis and
  dated), and `basin` at the commons: w, r, food, mach and food's technique, at 1.05^j for j =
  ±1 … ±43 written to 12 significant digits (`g12`, Python's `%.12g`), as the mirror ran them
  (430). The battery under `Hold` is `--one-sided hold`, with every tilt 1 is `--set tilt.*=1`,
  and the tick-length sets are Tiers 1–2 at `--tpy 12` and `--tpy 365`.

## 5. Load checks

Each is a `LoadError` at its path (`exit_is_checked_at_load`), at `actors[workers].spec.exit…`:

- `.good`: the exit good not one of the workers' basket goods, or the currency (at resolve);
- `.land`: the plots' land a basket item or the workers' hours (at resolve); not `Instant`, or not
  the land of the basket provider that pays the workers' support (`Cast::new`);
- `.exit`: workers with an exit that no basket provider's `transfer.to` names (ν = 1);
- `.plot`: too little land for every plot, T + T_o ≤ h·N at genesis, T the provider's endowment;
- `.gross`, `.floor`, `.plot`, `.commons`: a param of the wrong unit (the resolver's `UnitMismatch`).

The frame's check that s₀, s̲, h and T_o are finite and not negative is the registry's: every
param's value is finite with a clear sign bit (`params[<key>].value`).

## 6. Checks of the build

All on WSL in release unless said; scripts and outputs in `D:/rustyecon-p23/build-commons/`.

### 6.1 R1: every old tape, id and stream

- **Streams.** Every committed tape (the 23 of `tapes/` before the build) run 2,000 ticks through
  the `rustyecon` binary gives the same `--hashes` file, header and stream, as the binary built at
  `7ea124d` before any change, on WSL and on Windows (`streams.sh`; `streams-*.log`).
- **Ids.** `markets_tapes_keep_their_world_ids` (the wall's) pins P2.1's seven tapes' ids;
  `commons_leave_the_other_markets_tapes_ids` pins IW1's; `commons_specs_round_trip_and_old_worlds_keep_their_ids`
  checks that a tape without the field writes none and keeps I1's `world_id`.
- **Generators.** `markets-tape` writes P2.1's seven tapes and IW1's byte for byte (their tests,
  unchanged).
- **Nesting.** `workers_without_exit_are_p21s` (agents): over 3,000 random states, the exit-free
  workers offer N·min(ln1p(w/P_s)/χ_max, 1) hours bit for bit, and the workers with an exit
  switched off (s₀ = s̲ = 0; h 0 or 0.12; a commons of 0 or 30 a year) decide and produce
  exactly what the exit-free workers do, order, delta and state. `exit_switched_off_is_the_dependence_form`
  (probe): I1's tape and I1's with those four exit blocks run 2,000 ticks from genesis and from
  w × 2 with every market's prices, S, D, cleared volume and fills, every actor's coin and the
  workers' share equal bit for bit.
- The existing tests pass unchanged, `markets_i0_nests_appb`, `many_roles_nest_the_appendix_b_roles`
  and every P2.1, P2.2, P2.2b and wall test among them. `each_site_converts_as_registered` now
  reads C1's tape too, and `many_roles_never_overbudget_or_overdraw` draws 3,000 states on C1,
  its commons over six decades, so the workers' land buy passes admission and their burns fit.

### 6.2 The oracle, the rest point and mode A

- `commons_points_are_the_registered_ones`: at the 26 registered targets the harness's point is
  the frame's 50-digit solve (`registered/points.json`, which reads no oracle code) within 1e-13
  in x\*, v, P_s, Y, S, r_o and T_p, in the same regime; each funded; certified but b.food × 0.5.
- `commons_rule_is_unit_1e_supply`: at 20 points of the line of each target (and a constructed
  h 0.5 for the split) the rule's hours at the point's prices are 1e's supply within 1e-12, in
  1e's regime; the points visit Commons, Crowded, Enclosed and the split.
- `commons_rest_at_the_oracle`: at C1, C2 and their 24 targets, at 12, 52 and 365 ticks a year
  (78 points), three ticks from genesis leave every observable within 1e-12 of 1e's point in log,
  every market trading and filled, and the readout gives 1e's regime and r_o/r within 1e-12.
- `commons_hold_at_the_oracle_point`: mode A at 52 a year for 20,000 ticks at C1 and C2, largest
  gap within 1e-12, the plots in the oracle's regime every tick, no switch.
- `commons_genesis_is_unit_1e`: the genesis prices and the workers' coin, with and without rented
  plots, and the households' money accounts.

### 6.3 The rules and the harness

`plots_rent_enclosed_land` (agents) checks §2 against the formulas, written apart from the rule,
in each regime: the hours, the land buy of T_p at budget r·T_p, the baskets' budget and the
chain's total, the state, a coin too short for the rent, and produce's burn of the land.
`harness_reads_the_rule_the_workers_act_on` runs C1 with its commons at 12.15 a year, w × 0.8 and
r × 2 for 600 ticks and checks, tick by tick against an independent run of the same tape, that
the labour supply is the rule's hours, that the land the workers asked for is the readout's T_p,
and that a Crowded tick's shadow rent gives back the hours through n(p_g·s₀ − r_o·h).
`commons_conserve_and_run_deterministically` checks R8 and R2 on C1, C2 and through a transient
with rented plots. `commons_battery_and_families_are_the_registered_ones` checks §4's lists
against the registered runs (the frame's `exit.To` read as `commons`, a desk's `coin.<d>` as
`coin.desk.<d>`), with the battery's tiers and slack flags. `commons_grammar_applies_as_named`
checks §4's terms, enclosure's target land and the start distance.

### 6.4 Each test fails without its change

`mutants/mutate_c.py` undoes each change in turn and runs the agents' `commons`, `many` and
`seam` tests and the probe's `commons`, `wall` and `markets` tests: 30 mutants (the rule's
branches, fma, the crowded hours, T_p, the split, the land order, the baskets' budget, the rent's
cap, produce's burn, the state, the skip on the raw and the resolved field, five load checks, the
genesis rent, the target land, the start distance, the shadow rent, the split's label, the joint
order, enclosure, the share past 1, the history family, the exit good's index and the households'
money accounts). All 30 are killed by the committed tests (`mutants/mutants-final.out`). The first
pass left three alive, and each test was strengthened before the build's commit:
- e_r̂ rounded twice instead of by fma gave the same double at the tape's genesis prices, so
  `plots_rent_enclosed_land` now checks the hours and T_p to the bit over 2,000 draws at which the
  plots spill;
- a basket item as the plots' land is also refused by `Cast::new` (not `Instant`) at the same
  path, so `exit_is_checked_at_load` now reads resolve's own message;
- the point-based start distance equals the observables' at C1's land.mach shock, so
  `commons_grammar_applies_as_named` now checks the distance at all 26 shocks at genesis, some of
  which part the two.

The rest point, mode A and the rule's supply kill the mutants that move the map; the rule, load,
grammar, family and id tests kill the rest. Produce's burn of the land and the rent's cap by the
coin leave the rest point (the land is `Instant` and dies anyway; the coin covers the rent at
rest), and only `plots_rent_enclosed_land` and the overdraw test see them.

### 6.5 Development runs before E0

The build was exercised on the engine before E0: `markets point` and `markets run hold` for 300
ticks at C1 and C2, and a development trace diff (`e0/tracediff-dev1.out`) on the uncommitted
build, with E0's runs. That diff found the partings of §7's last item, in the negative control's
trap run only; they are disclosed here, as the wall's were (WALL-RULES §6.5).

### 6.6 The gates

`scripts/gate.sh` and `scripts/gui.sh` are green on WSL (`/root/scratch/target-p23`) and on
Windows (`D:/rustyecon-targets/p23`), 1,007 tests passed and 4 ignored on each (the wall's 991 and
this build's 16). The gate hash is `0x61f9c8529131ff17`, and gui.sh's hash check gives appb
`0xe1fa082b26995867` and demo-gb `0xfad880fe08d06645`, on both. The stamp is `7ea124d`, dirty: this
build before its commit. The two new tapes' 2,000-tick finals are `0x28df3904f206ca92` (C1) and
`0xe155f829dbd4fb37` (C2) on both machines. Logs in `D:/rustyecon-p23/build-commons/`
(`gate-*.log`, `gui-*.log`, `streams-*.log`).

## 7. Readings and departures

- **The registration's readings** (registration §3), each taken: Tier 3S in the verdict; the
  tapes' names; the mirror's run names, `commons` for `exit.To` and `coin.desk.<d>`; the basin's
  factors to 12 digits; a share past 1 set to 1; the joint order; `enclose` as two changes at one
  tick; the targets; the workers' state F; the readouts in `stats.tsv`; E0's runs.
- **Enclosure of the whole commons** leaves no open parcel in 1e's solve (a parcel's acreage must
  be at least its scale floor); the plots then all stand on enclosed land, as the mirror's T_o 0.
- **The negative control's trap run parts from the mirror** (the development trace diff; E0). In
  `c1n`'s `p[mach]*0.5` the workers' participation falls toward 0 as the wage nears the exit's
  value, so F = ln1p((w − e)/(P_s + e))/χ_max is a small difference of large ones; the price
  differences of a few 1e-15 that the budget chain's order leaves (MARKETS-RULES §6.4) are
  multiplied, and the trap's inflation multiplies them further. The engine and the mirror part
  above 1e-12 from tick 41 and by 1.6e-7 at tick 78, and run away at the same tick, 284. The
  mirror's one-step map from the engine's own state agrees within 2.3e-15 at every tick, and the
  mirror against itself with its genesis wage moved by 1e-15 parts by 4.7e-8 at the same tick and
  name. Every C1 and C2 run agrees within 2.6e-14. The frame's E0 allows no such parting, so the
  allowance is set by a dated amendment before E0 and any scored run (A1, P2.3.7).

## 8. How to run

```sh
markets-tape --inst c1 tapes/markets-c1.ron       # the committed tapes (and c2)
markets point --inst c1                           # unit 1e's point, base and every target
markets list battery --inst c1                    # the 115 runs with their tiers and slack
markets family battery --inst c1 --ticks 141000 --jobs 46 --csv DIR
markets family tier3s --inst c1 --ticks 141000 --jobs 24 --csv DIR  # first-year D̂
markets family stocks|joint2|joint4|basin|history|enclose --inst c1 --ticks 141000 --csv DIR
markets family battery --inst c1 --one-sided hold …   # the Hold variant; --set tilt.*=1 for tilt 1
markets family battery --inst c1 --tpy 12 --ticks 26000 …  # Tiers 1–2 at 12 a year
markets kick hold "land.mach=0.44@dated" … --inst c1 --ticks 141000 --horizon 141000
markets elasticity --inst c1                      # L from the engine's τ
markets family battery --inst c1n --ticks 141000 …    # the negative control, at C1's L
```

`stats.tsv` holds the commons' readouts as `commons.*` lines. The scorer, its gather script and
its job list are committed before the wave (decision 311).
