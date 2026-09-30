# WALL-RULES: the wall instance's agents and harness, as built

Dated 2026-09-30. Step P2.3.2 on branch `phase2-proper` (worktree `D:/rustyecon-wt/p23`, scratch
`D:/rustyecon-p23/build-wall/`). It is the build of the wall frame,
[docs/probe/wall/SPEC.md](wall/SPEC.md) (sha256 `cc2b6d1c…9be0`), registered at P2.3.1
([results/wall/registration.md](results/wall/registration.md)) before any of this code existed.
It says what was built, where the build reads the frame, and what was checked. The frame's §3 is
the specification; this file does not restate its argument, only what the code does.

## 1. What changed, and what did not

- **Core, markets and the engine: nothing.**
- **Agents: three optional fields** on the many-market roles (decision 395), and no new kind,
  state or market rule:
  - `CategoryDesk.tail: Option<Key>`, the pool's hours per unit at tasks closed to machines;
  - `CategoryDesk.reserved: Vec<(good, coef)>`, each reserved type's hours per unit, bought on
    that type's own labour market;
  - `BasketProvider.more: Vec<(to, heads)>`, further transfers of N_i·P_s.

  Each is `#[serde(default, skip_serializing_if = …)]` on the raw and on the resolved spec, as
  the type desk's `plant` is (P2.2b.1; L0.5's lesson), so a tape without them keeps its
  canonical text, `tape_hash` and `world_id`. `ActorState` gains no variant: the provider's
  `ProviderState { due, paid }` holds the sums of its transfers. The M6 check (a bought good
  dies within the tick) covers the reserved goods.
- **Probe:** `probe::markets` gains the wall instances IW1 and IC1, unit 1d's point, their
  genesis and tape, the observables at the wall, the run grammar's new terms, the wall's battery
  and families, the wall's readouts and Tier 3S's start distance. P2.1's instances keep every
  number, name and file they had (§6).
- **Tapes:** `tapes/markets-iw1.ron`, `markets-tape --inst iw1`'s output. IC1 is `--inst ic1`,
  with no committed tape.
- **Horses harness:** two matches on the shared `ShareAt` take its new variant, and its flow
  genesis builds the markets `Setup` with the new field at `false`. No horses number moves.

## 2. The roles' rules

The frame's §3.2–§3.4, as `crates/agents/src/roles/many/rules.rs` makes them:

- **The tail.** h = H + L^H, one addition after the line's H (`CategoryDesk::with_tail`), and
  none without a tail. The hours are ordered whenever the line has hours (L̄ > 0) or the tail is
  positive, at quantity h·q. At the wall (s → 0) the services desk still buys its tail's hours.
- **Reserved hours.** Read in list order as (good, R_ji). The unit cost is
  c = ((w·h) + (p_τ·M/θ)) + (r·b), then c += p_i·R_ji for each in list order. Each reserved input
  whose coefficient is not zero is ordered at R_ji·q with budget p_i·(R_ji·q), in the one budget
  chain (admission's good order). Produce is Leontief over (labour, h), (service, M/θ), (land, b)
  and each (reserved good, R_ji), and burns R_ji·y of each.
- **Further transfers.** The provider pays N·P_s to `transfer`'s recipient from the copy of its
  coin, then each entry of `more` in list order, min(N_i·P_s, the coin left), each a `Transfer`
  delta when above 0. Its state is due = ((N·P_s) + N_1·P_s) + … and paid likewise, so certify's
  `Obs::transfers` reads the whole shortfall. The basket budget is share(spend) of the coin left.
- **The workers and the type desk** are unchanged. Each reserved type is a `BasketWorkers` pop
  on its own labour good; the type desk buys pool labour (decision 372).

With the three fields absent every rule is P2.1's code path; at zero they add 0.0 and order
nothing (§6.1).

## 3. The instance, its dials, genesis and tape

- **IW1** (`Instance::named("iw1")`; decision 394): categories services (z 1, b 0, μ 0.75, tail
  0.1, the trained's R 0.04) and goods (z 1, b 0, μ 1, the master's R 0.03) on one segment,
  space 1, Appendix B's machine (θ 1, own 0.3, λ 0.05, b 0.4), η 0.5, g0 0.2, g1 0.8, k 1,
  T 520 a year; workers 130 a year at χ_max 1; the reserved types `trained` (52, χ_max 2) and
  `master` (26, χ_max 2). **IC1** is IW1 with the entrant's χ_max 0.25 (decision 397).
- **Markets** in the harness's order: labour, land, mach, services, goods, labour.trained,
  labour.master. **Actors:** desk.services, desk.goods, desk.mach, provider, workers,
  workers.trained, workers.master, one class each (`trained_workers`, `master_workers` for the
  reserved pops).
- **Params** (`Instance::params`): P2.1's keys, and `inst.<type>.workers` (`FlowPerYear`),
  `inst.<type>.chi_max`, `inst.services.tail` and `inst.<category>.reserved.<type>`
  (`Dimensionless`), each written only where positive. The frame's own numbers carry the basis
  `Assumed("docs/probe/wall/SPEC.md §2.1: …")`; the machine's stay Appendix B's `Literature`.
- **The cost coefficients:** `land.mach` (`inst.mach.land`), `tail.services`
  (`inst.services.tail`) and `res.services.trained` (`inst.services.reserved.trained`), each at
  ×1.1, ×0.9, ×2 and ×0.5 (decision 396).
- **Dials: C2m** (decision 364), with `rate.labour.<type>` at labour's 5.2 a year after
  `rate.labour`.
- **The oracle** (`Instance::worker_params`, `worker_point`): unit 1d's `WorkerEconomy::solve` on
  `WorkerParams::from_machines(machine_params)` with the worker types in pop order, the pool's
  (N, χ_max, ε 1, ν 1) first and each reserved type (ε 0, ν 1) after it, `human_required` the
  tails and `reserved` each category's hours in its type's column. It is the frame's scratch
  crate's construction; P2.1's instances keep unit 1c.
- **Genesis** (the frame's §3.6): prices v, 1, p_mach, p_services, p_goods, v_trained, v_master;
  shares 1 − x\* = 0.0; stocks z·Y and X; coins p_j·y_j/share, ((λ·v) + b)·X/share, the provider
  τ + (T − τ)/share with τ = ((N·P_s) + N_T·P_s) + N_M·P_s, and each pop
  (N_i·P_s + v_i·hours_i)/share, hours_i unit 1d's `WorkerEq::hours`. Every genesis double equals
  the frame's `points.jsonl` and every coin the frame's mirror's `wall.genesis`, bit for bit
  (`markets_iw1_genesis_is_unit_1d`).
- **The tape** writes the category desks' `tail: Some(..)` and `reserved: [..]` and the
  provider's `more: [..]` only where present, the reserved labour goods as `Instant`, and a
  header deriving genesis from unit 1d. The actors' basis names this file.

## 4. The harness at the wall

A worker-form instance (`Instance::worker_form`) switches the harness's wall reading on; P2.1's
instances take none of it.

- **Observables** (the frame's §5.1; decision 396): v; π of the type and each category; each
  reserved wage `w.<type>`; each desk's threshold `x.<category>` = 1 − s_j, in place of s_j; the
  cleared volume of all seven markets; each desk's output. That is 18 (the frame's text says 16
  and lists 18; registration §3). Every tolerance is the 1e-3 floor. The targets: the pool's
  hours n_D on `labour`, each reserved type's D_i on its market, x\* = 1 − (1 − x\*) = 1.0.
- **Each household's baskets**, the provider's, the workers' and each reserved pop's, and their
  oracle values (support plus wage bill over P_s). The CSV adds `baskets_` and `bound_` columns
  for the reserved pops, then `part_<pop>` (each pop's participation) and `depth`.

  *Amended at P2.3.10 (2026-09-30, label `run`, before the scored wave):* the baskets eaten over
  Y\* (`low_baskets`, `no_basket_ticks`, `baskets.trough` and the cost shocks' `depth.*`) summed
  the provider's and the workers' baskets only, as at P2.1's two households, so at IW1 and IC1
  they left out the reserved pops' and read 0.725 at rest (E2's `e0/modea.out`, where nobody
  read it). The frame's mirror sums every household (`nb_p + nb_w + Σ nb_r`), and so the §6.2
  troughs and E5's bottoms are registered. `Stats::push` now sums every household in
  `Instance::households` order, the first two first, so every two-household instance keeps its
  numbers bit for bit; `markets_iw1_baskets_count_every_household` fails without it. It was
  found by reading the code while the scorer was written, before any scored run.
- **The wall's readouts** (the frame's §5.3; `Stats::wall`, `stats.tsv`'s `wall.*` lines),
  reported and never scored: the lowest depth ln(θ·w/(p_τ·γ(1))) at posted prices and its tick,
  breach ticks (depth below 0), the first and the longest run; each desk's largest human share;
  each pop's participation range and saturated ticks (F = 1); the worst buyer fill over every
  market, as the mirror reads it.
- **A cost shock's start distance** at the wall is the largest |ln| over every observable of the
  two targets, over the tolerance, as the mirror measures it (`target_distance`).
- **Tier 3S** (decisions 229, 368, 397): with `Setup::first_year_d0` a stock or coin start's
  distance is the largest D̂ of its first scored year. `markets family tier3s` sets it (and
  refuses to share a call with another family); `markets run --first-year` sets it for named
  runs. The stocks family keeps P2.1's |ln F|.
- **Run grammar** (the frame's §5.2): `s[D]=V` sets the share to V exactly (`ShareAt::Is`); JA and
  JB scale every labour market's price as w's; `RW(F)` is the pool's wage × F and each reserved
  wage × 1/F; `joint(F,SEED)` at the wall draws in the mirror's market order (labour, land, the
  categories, the type, the reserved labour markets). `x*/2`, `RC`, `N`, `p[M]*F`, `coin.A*F`,
  `stock.D*F` and the cost terms are P2.1's.
- **The battery** (`wall_battery`): the mirror's `wb.run_list` in its order and names: each
  market's price at the six factors, factor by factor, in the mirror's market order (`p[labour]*F`,
  not `w*F`); each desk's share at 0.05, 0.2 and 0.5; JA, JB, N, RC and RW at the six factors;
  x\*/2; the coefficients at their four values, at genesis and dated. 103 runs, 26, 38 and 39 by
  tier, none slack.
- **Families:** `tier3s` (20), `stocks` (P2.1's with each reserved pop's coin, 31), `joint2`,
  `joint4`, `history` (`cycle(land.mach,1500,80)`), and `basin` at the wall: w, r, services, mach
  and each reserved wage, 1.05^j for j = ±1 … ±43, named `p[M]*<factor>` (516). Every list
  equals the registered TSV's `run` column, name for name and in order
  (`wall_battery_and_families_are_the_registered_ones`).

## 5. Load checks

Each is a `LoadError` at its path (`wall_specs_are_checked_at_load`):

- a reserved good named twice, or equal to the desk's output, labour, service or land
  (`actors[desk.services].spec.reserved[<good>].good`);
- a further transfer to `transfer`'s recipient or to a recipient named twice
  (`actors[provider].spec.more[<key>].to`, at resolve), or to the provider itself (the same path,
  at `Cast::new`);
- a tail param, or a reserved coefficient, of a unit other than `Dimensionless`, and a `heads`
  other than `FlowPerYear` (the resolver's `UnitMismatch`);
- a reserved good that outlives the tick (M6), and a reserved pop's labour that is not `Instant`
  (its endowment check).

## 6. Checks of the build

All on WSL in release unless said; scripts and outputs in `D:/rustyecon-p23/build-wall/`.

### 6.1 R1: every old tape, id and stream

- **Streams.** Every committed tape (the 22 of `tapes/` before the build) run 2,000 ticks through
  the `rustyecon` binary gives the same `--hashes` file, header and stream, as the binary built
  at `f0c6667` before any change, on WSL and on Windows (`base-streams.sh`; `streams-*.log`).
  The gate, appb and demo-gb finals are `0x61f9c8529131ff17` (gate.sh), `0xe1fa082b26995867` and
  `0xfad880fe08d06645` (gui.sh's hash check), on both machines; the gates are green on both
  (991 tests passed, 4 ignored, on each).
- **Ids.** `markets_tapes_keep_their_world_ids` pins the seven P2.1 tapes' `tape_hash` and
  `world_id` as the pre-build binary printed them; `wall_specs_round_trip_and_old_worlds_keep_their_ids`
  checks that a tape without the fields writes none of them.
- **Generators.** `markets-tape --inst <id>` writes P2.1's seven tapes byte for byte
  (`markets_tapes_are_their_generators_output`, unchanged).
- **Nesting at zero.** `wall_fields_at_zero_leave_every_decision` (agents): I1 with a tail of 0,
  a reserved input at coefficient 0 on every category desk and a further transfer to a pop of 0
  heads; every old actor's `decide` and `produce` equal the field-free tape's from 500 random
  states, order for order and delta for delta. `wall_roles_nest_the_many_roles` (probe): the same
  two tapes run 2,000 ticks, from genesis and from w ×2, with every old market's prices, S, D,
  cleared volume and fills and every old actor's coin equal bit for bit.
- The existing tests pass unchanged: `markets_i0_nests_appb`, `many_roles_nest_the_appendix_b_roles`
  and every P2.1, P2.2 and P2.2b test. `each_site_converts_as_registered` now reads IW1's tape
  too, so the new sites' paths are the resolver's.

### 6.2 The rest point and mode A

- `markets_iw1_rest_point_is_the_oracles`: at the base and the 12 cost targets, at 12, 52 and
  365 ticks a year (39 points), three ticks from genesis leave every observable within 1e-12 of
  unit 1d's point in log, every market trading and filled, margin Wall. The depth at the base is
  0.942574842758311, the frame's ln(g/γ(1)).
- `markets_iw1_holds_at_the_oracle_point`: mode A at 52 a year for 20,000 ticks, largest gap
  within 1e-12, the thresholds at 1.0 exactly every tick, no breach.

### 6.3 The rules

`category_desk_buys_its_tail_and_reserved_hours` and `provider_pays_every_transfer_in_list_order`
(agents) check §2 against the formulas, written apart from the rules. `markets_iw1_conserves_and_is_deterministic`
checks R8 and R2 at rest and through w ×2 with the reserved wages ×0.5 and every desk's coin
×0.1. `wall_grammar_applies_as_named` checks §4's terms, the joint draw order included.
`many_roles_never_overbudget_or_overdraw` now draws its 3,000 states on IW1 too, so the budget
chain with reserved lines and the provider's further transfers pass admission and every burn
fits, whatever the prices, coins, holdings and dials.

### 6.4 Each test fails without its change

`mutants/mutate.py` undoes each change in turn (20 mutants: the tail in decide and in produce,
the reserved cost, order, Leontief and burn, the further transfers' payment, sums and take, a
zero coefficient ordered, the skip on the resolved tail and the raw `more`, three load checks,
the workers' genesis coin, the thresholds, JA's reserved wages, the joint order and the basin's
powers) and runs the two `wall` test files. All 20 are killed (`mutants/mutants.out`). The rest
point and mode A kill the seven that move the map (the tail in decide, the reserved cost and
order, the transfers' payment and take, the workers' coin, the thresholds); the rule, load,
grammar and id tests kill the rest. Undoing the tail in produce, or the reserved burn, leaves the rest point, since the
hours are `Instant` and the unused part dies anyway, and only `category_desk_buys_its_tail_and_reserved_hours`
sees it.

### 6.5 Development runs before E0

The build was exercised on the engine before E0: six IW1 runs of 3,000 ticks (hold, p[labour]×2,
JB(0.5), x\*/2, s[goods]=0.5, RW(2)) to see that the tape loads and the harness classifies, and a
development trace diff (`e0/tracediff-dev.out`) on the uncommitted build, with E0's runs. The
latter found one parting, below; its numbers are E0's (P2.3.4). They are disclosed here, as
P2.2a's smoke runs were (HORSES-RULES §8).

## 7. Readings and departures

- **The registration's readings** (registration §3), each taken: E0's three coefficient runs;
  18 observables; the registered run names; the joint draw order; `s[D]=V` exact; the tape's
  name; the history family's `base·f`.
- **The basin's factors.** The mirror names its basin runs by Python's `1.05 ** j`, which is
  correctly rounded at every j in ±1 … ±43; libm's `pow` is an ulp off at some (j = −17 gives
  0.43629668761085716 against 0.4362966876108571). The wall's basin takes `pow_whole`, x^j in
  double-double arithmetic with one rounding at the end, which gives the registered doubles at all
  86 (the test). P2.1's basin keeps `num::pow`.
- **The joint draws' factors** F^u are libm's `pow` where the mirror's are glibc's; the two may
  part by an ulp in a starting price. The joint families are reported, not scored.
- **Mode A's spoilage check** reads produced goods only (types and categories); a reserved type's
  unsold hours, like the pool's, are not produced. P2.1's instances have no other goods.
- **E0's tick-1 residue** (the development trace diff; E0, P2.3.4). In `stock.mach*0.01` the
  services and goods markets at tick 1 hold only what is left of their genesis lots, one ulp of
  7.69 (8.9e-16) in the engine, which keeps each lot's unsold part as held − sold, and 8.5e-16 in
  the mirror, which computes held·(1 − fill). S and the cleared volume there part by 3.9e-2 in
  log (3.4e-17 absolute); the mirror one ulp from itself parts without bound on the same tick.
  Every price, coin, share, stock and every other S and D agree within 1.4e-14 over the 2,000
  ticks. The frame's E0 allows only the budget chain's ulp, so the allowance is extended by a
  dated amendment before any scored run: A1 (P2.3.3, results/wall/registration-A1.md), under
  which E0 passes (results/wall/e0.md).

## 8. How to run

```sh
markets-tape --inst iw1 tapes/markets-iw1.ron     # the committed tape
markets point --inst iw1                          # unit 1d's point, base and every target
markets list battery --inst iw1                   # the 103 runs with their tiers
markets family battery --inst iw1 --ticks 22000 --jobs 46 --csv DIR
markets family tier3s --inst iw1 --ticks 22000 --jobs 20 --csv DIR   # first-year D̂
markets family stocks|joint2|joint4|basin|history --inst iw1 --ticks 22000 --csv DIR
markets kick hold "land.mach=0.8@dated" … --inst iw1 --ticks 22000 --horizon 22000
markets elasticity --inst iw1                     # L from the engine's τ
markets run NAME --inst ic1 --ticks 25000         # the line control
```

`stats.tsv` holds the wall's readouts as `wall.*` lines. The scorer, its gather script and its
job list are committed before the wave (decision 311).
