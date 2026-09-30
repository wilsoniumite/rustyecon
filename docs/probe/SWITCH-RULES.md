# SWITCH-RULES: the type switch at the wall, the migration rule, as built

Dated 2026-09-30. Step P2.4.8 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`, scratch
`D:/rustyecon-p24/build-switch/`). It is the build of the switch scan's choice,
[docs/probe/switch/SPEC.md](switch/SPEC.md) (sha256 `ebf52cea…315f6f` as registered), registered
at P2.4.7 ([results/switch/registration.md](results/switch/registration.md)) before any of this
code existed; decisions 409–415. It says what was built, where the build reads the spec, and what
was checked. The spec's §3 is the specification; this file does not restate its argument, only
what the code does.

## 1. What changed, and what did not

- **Core, markets and the engine: nothing.**
- **Agents: one optional field and one state variant** (decision 411), and no new kind or market
  rule:
  - `RawBasketWorkers.pool: Option<RawPool>`, `RawPool { good, efficiency, rate, share }`:
    `good` the pool's labour (a key); `efficiency` a param key, `Dimensionless`, live (ε);
    `rate` a param key, `RatePerYear`, live, read as a `LogStep` (k = rate/tpy); `share` the pool
    share at genesis, an inline number in [0, 1].
  - Resolved `BasketWorkers.pool: Option<Pool>`, `Pool { good: GoodId, efficiency: Site, rate:
    Site, share: f64 }` (the resolved form keeps the genesis share, as the trap's `Pace` does;
    registration §3).
  - `ActorState::SwitchWorkers(SwitchWorkersState { share, pool })`, appended after
    `PlantedCapacity` (as P2.2b appended its planted states, decision 287): `share` is F, a
    record as `WorkersState::share` is; `pool` is a, which the rule reads and moves. At genesis
    `share` is 0.0 and `pool` the spec's `share` (`genesis_state`). `apply` and `validate` check
    both: finite, a clear sign bit, at most 1.

  Both fields are `#[serde(default, skip_serializing_if = "Option::is_none")]`, so a tape without
  them keeps its canonical text, `tape_hash` and `world_id`, and a pop without a pool keeps
  `Workers` and P2.3's rule. The cast runs a pop with a pool as `SwitchWorkers`
  (`roles::many::switch`), whose `Own` is the new state. `BasketWorkers::sites` lists
  `pool.efficiency` and `pool.rate`. The GUI's `state_fields` lists a switch pop's `share` and
  `pool` (`StateField::Pool`), and its inspector names the kind "BasketWorkers, switching".
- **Probe:** `probe::markets` gains the switch instances IS1 and IS2 (`Instance::switch`, each
  reserved type's `efficiency`), unit 1d's point with the efficiencies and its reserved hours,
  pooled flags, pool shares and switch distances (`Point`), the dials `rate.switch.<type>`, the
  genesis pool shares and their displacement, the generator's pool blocks and header, the
  grammar's `sw[T]=V` and `--set rate.switch.*=F`, the battery's six `sw[T]=V` runs, the
  observables `rs.<type>`, and the readouts `pool_<pop>`, `swgap_<pop>` and `switch.*`. P2.1's,
  the wall's, the commons' and the paced instances keep every number, name and file they had
  (§6.1).
- **Tapes:** `tapes/markets-is1.ron`, `markets-tape --inst is1`'s output. IS2 is `--inst is2`,
  with no committed tape.

## 2. The rule

The spec's §3.3, as `crates/agents/src/roles/many/switch.rs` makes it, in its order:

```
w = price(pool.good); w_i = price(labour); P_s = Σ z_j·p_j from 0.0; ε = param(efficiency); a = state.pool
if ε == 0.0:  F = min(ln1p(w_i/P_s)/χ_max, 1); own = N·F; pooled = 0; a' = a       (P2.3's hours)
else:         k = param(rate)                                                      (rate/tpy)
              e = ε·w; g = ln(e/w_i)                                               (switch_gap)
              a' = a + (−expm1(−(k·g)))·(1 − a)  if g > 0
                   a·exp(k·g)                    if g < 0
                   a                             if g = 0                          (switch_move)
              v = w_i + a'·(e − w_i); F = min(ln1p(v/P_s)/χ_max, 1); n = N·F
              own = (1 − a')·n; pooled = ε·(a'·n)                                  (switch_split)
mint own of labour where own > 0; sell own (always, as P2.3)
mint and sell pooled of pool.good only where pooled > 0
baskets: share(spend)·coin, P2.3's orders
state: SwitchWorkers { share: F, pool: a' }
```

`switch_split` (steps 3–6) is public, and the harness reads a pop's gap through `switch_gap`, so
the readout cannot part from the rule. Produce eats the baskets, as P2.3's workers do; upkeep does
nothing. The rule reads two posted prices, the basket's prices, its own params and its own state
(R13). a' stays in [0, 1] by construction and no clamp is needed (R3): for g > 0 a convex
combination of a and 1 with weight in [0, 1), for g < 0 a times a factor in (0, 1); core's check
that a state share is at most 1 never fires (`many_roles_never_overbudget_or_overdraw` draws ε over
three decades or 0, the rate up to 1e4 a year, the pool share at 0, 1 or uniform and prices over
twelve decades). At a pooled rest the oracle posts the type's wage as ε·v, so e = w_i bit for bit
and g = 0.0 (the spec's §4); at a few points of 12 ticks a year the posted wage is an ulp off (the
spec's §6.2), and a moves by less than 1e-13 relative in three ticks (§6.2 below).

## 3. The instances, the dials, genesis and tapes

- **IS1 and IS2** (`Instance::named`; decisions 412, 413) are IW1 (`Instance::wall` at χ_max 1)
  with the trained's ε 1.5 and the master's 1.8 (`SWITCH_EFFICIENCY`, unit-1d.md §3.3 E), and
  `switch` set. IS2 has the trained's reserved hours 0.02 a unit of services, and its registered
  values of that coefficient, 0.022, 0.018, 0.04 and 0.01, as the scan's TSVs name them.
- **The oracle**: `worker_params` gives each reserved type its `efficiency` (0 at IW1, as before),
  so `WorkerEconomy::solve` finds the switch. `Point` adds `reserved` (D_i), `pooled`,
  `pool_share` (a\* = pool hours over hours where pooled with pool hours above 0, else 0, the
  mirror's `pool_share_of`) and `switch_distance` (ln(ζ_i·P_s/(ε_i·v)), NaN at ε 0).
- **Params**: `inst.<type>.efficiency` (`Dimensionless`, basis `Assumed("scan-switch SPEC §2.1:
  unit-1d.md §3.3 E's efficiencies")`), written after each type's `chi_max` at a switch instance
  only, so IW1's params are unchanged.
- **The dials** (decision 410): `rate.switch.<type>`, 26 `RatePerYear`, basis
  `Assumed("scan-switch SPEC §6.3")`, in C2m after the reserved labour markets' rates, at a switch
  instance only. `--set rate.*=f` scales them with every price rate (the scan's `rate` dial);
  `--set rate.switch.*=f` scales them alone (its `switch` dial). The open-loop probe's `rate.*=0`
  freezes them with the prices.
- **Genesis** (the spec's §3.8): the wall's, on 1d's point with the efficiencies (prices, the
  reserved wages v_i = ε·v where pooled, coins (N_i·P_s + v_i·hours_i)/share(spend) with
  hours_i = `WorkerEq::hours`), and each switch pop's pool share a\*, or V under `sw[T]=V`
  (`Genesis::switch`). At IS1's base every genesis number is IW1's (both types walled).
- **The tape** writes each reserved pop's `pool: Some((good: "labour", efficiency:
  "inst.<type>.efficiency", rate: "rate.switch.<type>", share: a*))`, last in its block; its
  header names the frame, the step P2.4 and the test `markets_is1_tape_is_its_generators_output`,
  and gives the efficiencies, a\*, the pooled flags, the switch distances and the shares written.
  Against IW1's tape IS1's differs in the header, the name, the two efficiency params, the two
  dials, the roles' basis, the two pool blocks and the genesis basis; its genesis prices and
  holdings are IW1's line for line.

## 4. The harness at a switch instance

- **Observables** (decision 414): IW1's 18 in IW1's order, then `rs.<type>` for each switch pop
  in pop order: 1 − a after the tick, in log, target 1 − a\*. That is 20 at IS1. Each reserved
  market's volume target is its reserved hours D_i (`Point::reserved`), which equals `hours` at a
  wall bit for bit, so IW1's targets do not move.
- **Grammar** (registration §3): `sw[T]=V` sets switch pop T's genesis pool share to V exactly,
  refused at an instance with no switch pop, and V outside [0, 1] refused at genesis. It is a
  price start: |ln((1 − V)/(1 − a\*))| joins the genesis prices' and thresholds' gaps (the
  mirror's `swb.d0`; 693.147 for V = 0.5 at IS1's base). A cost shock's start reads every
  observable, the reserved shares among them (the wall's `target_distance`).
- **The battery**: the wall's 103 runs in the wall's order, then `sw[trained]=0.05`, `0.2`, `0.5`,
  `sw[master]=0.05`, `0.2`, `0.5` (tiers 1, 2, 3): 109 runs, 28, 40 and 41 by tier. Tier 3S,
  stocks, joint2, joint4, basin and history are the wall's lists. The registered TSVs are sorted
  by name; the lists equal them as sets, and the battery's tiers run for run.
- **Readouts**, reported and never scored: the CSV adds `pool_<pop>` (each switch pop's pool
  share after the tick) and `swgap_<pop>` (its gap at the prices the tick settled at, ε in force)
  last; `stats.tsv` adds `switch.live_ticks` (scored ticks with a above 1e-9), `switch.max`,
  `switch.band` (sign changes of g between scored ticks with |g| above 1e-6), `switch.end` and
  `switch.gap_end`, the pop as `where`. The wall's participation readout reads a switch pop's F.
  `markets point` prints each type's reserved hours, pooled flag, a\* and switch distance.

## 5. Load checks

Each is a `LoadError` at `actors[<pop>].spec.pool…` (`switch_pool_is_checked_at_load`):

- `.good`: declared (the resolver's), not the pop's own `labour` or an item of its basket (a good
  named twice in one role), and `Instant` (with the world, `Cast::new`, as the pop's own labour);
- `.efficiency`: a `Dimensionless` param; `.rate`: a `RatePerYear` param (the resolver's
  `UnitMismatch`); a negative value is the registry's (`params[rate.switch.trained].value`);
- `.share`: finite and in [0, 1] (NaN, ±inf, −0.1 and 1.5 refused);
- `pool` together with `exit`: refused at `actors[<pop>].spec.pool` (O123).

The block's four fields are required and no other is read.

## 6. Checks of the build

All on WSL in release unless said; scripts and outputs in `D:/rustyecon-p24/build-switch/`.

### 6.1 R1: every old tape, id and stream

- **Streams.** Every committed tape before the build (the 27 of `tapes/`) run 2,000 ticks through
  the `rustyecon` binary gives the same `--hashes` file, header and stream as the binary built at
  `b7b9242` before any change, on WSL and on Windows (`streams-*.log`). The two base streams
  equal each other.
- **Ids and text.** `markets_is1_tape_is_its_generators_output` pins IW1's `tape_hash` and
  `world_id` to the pre-build binary's and checks the generator writes IW1's tape byte for byte;
  `switch_genesis_state_is_the_tapes` pins IW1's `world_id` in agents and checks a tape without a
  pool writes none; the generator writes every older markets tape byte for byte (their tests,
  unchanged).
- **Nesting.** `switch_off_is_the_reserved_pop` (agents): over 500 random states, with ε 0 each
  switch pop of IS1 decides as IW1's reserved pop decides from the same state, order for order
  and delta for delta but its state, which holds P2.3's share and a unchanged.
  `markets_is1_never_crossing_is_iw1` (probe): IS1 from hold and four starts inside both walls,
  and IS1 with both efficiencies 0 from four starts that cross with them on, give IW1's markets
  and coins bit for bit for 2,000 ticks; with the switch on RW(2) parts from IW1.

### 6.2 The points and the rest point

- `markets_is1_points_are_the_registered_ones`: at IS1's and IS2's base and 12 targets at 12, 52
  and 365 ticks a year (78 points), the harness's unit 1d point is the scan's oracle's registered
  doubles (`points_is1.jsonl`, `points_is2.jsonl`) bit for bit in v, P_s, Y, n_D, 1 − x\*, every
  price and output and each type's wage, hours, reserved hours, pooled flag and a\*; a\* and the
  switch distances are the 50-digit solve's within 1e-12 and its pooled flags (E9); every walled
  point of IS1 is IW1's.
- `markets_is1_rest_point_is_the_oracles`: at the 78 points, three ticks from genesis leave every
  observable within 1e-12 of the oracle in log, every market trading and filled, margin Wall; a
  pooled pop's share stays within 1e-12 relative of a\* and its gap within 1e-15 of 0 (0.0 but at
  the few points of 12 a year where the posted wage is an ulp off ε·v); a walled pop's share stays
  0.0 with its gap below 0. 51 pooled rests: IS1's five and IS2's twelve, at three tick lengths.
- `markets_is1_holds_at_the_oracle_point`: mode A for 20,000 ticks at IS1's and IS2's base,
  every gap within 1e-12, no breach.

### 6.3 The rules and the harness

- `switch_rule_matches_the_registered_vectors` (agents): the registration's §3 reading, g bit for
  bit at all 24, every output bit for bit at 21 and within 4 ulps at vectors 5, 13 and 23, where
  libm's `exp` is an ulp off glibc's (the set of parted vectors is checked to be exactly those
  three); `switch_split` equals §3.3 written again in the test, bit for bit.
- `switch_moves_toward_the_better_market` (agents): over 3,000 random states of IS1's world
  (prices over two or twelve decades, the pool share at 0, 1 or uniform, ε over a decade, the
  rate over four decades by `SetParam`, exact ties), the decision offers and mints (1 − a')·n and
  ε·(a'·n), the latter only where positive, sets (F, a'), reads k = rate/52, and meets the corners
  (share(k·g) from 0, e^(k·g) from 1, a at a tie); the sites are listed.
- `switch_state_is_checked`, `switch_genesis_state_is_the_tapes` (agents): the state's checks,
  the genesis state, the round trip.
- `switch_pays_both_markets` (probe): through RW(2) and tail.services=0.2, every tick, each
  switch pop's coin moves by its transfer (the provider's rule, which runs short in RW(2)), plus
  price × quantity on each market it sold on, less its baskets; money within 1e-12 of its stock;
  every ledger closes; more than 1,000 pop-ticks sell on both markets.
- `harness_reads_the_switch` (probe): at tail.services=0.2 (both types cross) and at
  `sw[trained]=0.5` (the trained's share decays toward the subnormals), tick by tick against an
  independent run, the rows' pool shares, reserved shares, participation and gaps are the pops'
  own, each reserved market's supply is (1 − a)·N·F, and the `switch.*` readouts are the rows'
  (the live count leaving out the ticks with a in (0, 1e-9]); the CSV names its columns.
- `switch_battery_and_families_are_the_registered_ones`, `switch_grammar_and_dials_apply_as_named`
  and `markets_reports_the_switch` (probe): the lists against the TSVs, the grammar, the start
  distance, the dials and `rate.switch.*`, the binary's `switch.*` lines and `point` columns.
- `inspector_reads_switch_states` (GUI).

### 6.4 Each test fails without its change

`mutants/mutate_s.py` undoes each change in turn and runs the agents' `switch`, `many`, `seam`,
`pace` and `wall` tests, the probe's `switch`, `wall`, `markets`, `trap` and `commons` tests and
the GUI's inspector tests: 60 mutants (the rule's move up, down and at a tie, the wage it reads,
the gap, both offers, the off switch and the a it holds there, both mints, the pool's sell, the
state, the baskets eaten; the raw and the resolved skip, the exit refused, the own labour and the
basket item refused, ε's and the rate's methods, the share check, the sites, the resolved pool;
the state's checks and bound, the genesis state, the `Instant` check; the GUI's field, kind and
unit; the efficiency given to the oracle, the switch flag, IS2's values, the efficiency params and
their `set`, a\*, the reserved hours; the dial's value, `rate.switch.*`, the displacement, its
range, the share written; the observables' names, the volume targets, the reserved shares' target
and row, the gap's wage, the live count, the band, the largest a, the start distance, the CSV
columns, the stats' push; the grammar's guard, the battery's runs, `sw` parsed; the binary's
lines and point). All 60 are killed by the committed tests (`mutants/mutants-final-shard*.out`).
The first pass (`mutants-1-shard*.out`) left two alive, and their tests were strengthened before
this commit:
- the pool's good allowed to be a basket item: the load test's case was services, which a pop
  cannot mint anyway (one tick, not `Instant`), so the `Instant` check refused it at the same
  path; land, the basket's space and `Instant`, now tests the basket check alone;
- the live count at a > 0 in place of a > 1e-9: the harness test's run kept both shares far above
  1e-9; `sw[trained]=0.5`, whose share decays toward the subnormals with more than 1,000 of its
  2,000 ticks in (0, 1e-9], now tests it.

The rule, points, rest-point, mode-A and never-crossing tests kill the mutants that move the map;
the load, state, grammar, readout, list, id, binary and GUI tests kill the rest, most of them one
test each (a check's mutant by its check's test). Undoing the baskets eaten leaves the prices and
coins alone, and only mode A and the rest point see it, through the goods spoiled; the rate read as
a share is seen only by `switch_moves_toward_the_better_market`, which checks k = rate/52.

The mutants ran in four shards, each on its own copy of the worktree (`tar` of the uncommitted
sources, `.git` left out) with its own target directory, deleted after (`mutants/shards.sh`). A
first single run on the worktree itself was stopped by hand after five mutants, all killed, to
run the shards (`mutants/mutants-0-interrupted.out`); its Ctrl-C restored the mutated file, and
every mutant's original text was checked present before anything else ran.

### 6.5 Development runs before E0

The build was exercised on the engine before E0: four IS1 runs of 22,000 ticks (`hold`,
`sw[trained]=0.5`, `tail.services=0.2@genesis`, `res.services.trained=0.02@dated`) gave the
registered class, ticks to tolerance (470, 734, 415), peak D̂, start distance, dead ticks and
switch readouts of each (the mirror stops early, so its pooled-tick counts are over 4,001 ticks);
and a development trace diff (`e0/tracediff-dev1.out`) on the uncommitted build, with E0's
seventeen runs: no parting above 1e-12 but the wall's A1 residue (the largest 1.35e-13, at x\*/2's
tick 2 on the services desk's small output). Disclosed here, as the commons', the wall's and the
trap's were.

### 6.6 The gates

`scripts/gate.sh` and `scripts/gui.sh` are green on WSL (`/root/scratch/target-p24`) and on
Windows (`D:/rustyecon-targets/p24`), 1,035 tests passed and 4 ignored on each (P2.4.5's 1,019
and this build's 16; the GUI's own test runs in `gui.sh`). The gate hash is
`0x61f9c8529131ff17`, and gui.sh's hash check gives appb `0xe1fa082b26995867` and demo-gb
`0xfad880fe08d06645`, on both. The stamp is `b3b35a3`, dirty: this build before its commit. The
new tape's `tape_hash` is `0xcfc9a45a69b23b96`, its `world_id` `0xdd77db99d4d45166`, and its
2,000-tick final `0xa4b95240fa8d8f47`, on both machines. Logs in
`D:/rustyecon-p24/build-switch/` (`gate-*.log`, `gui-*.log`, `streams-*.log`). The first pass,
before the two tests were strengthened, was green alike (`pass1/`); a second failed clippy's
`eq_op` on the literal heads 52.0/52.0 in the harness test once it moved into a helper
(`pass2-failed/`), written 1.0 and 0.5 before this pass. The final mutation pass ran on the test
file with the literal, the same doubles.

## 7. Readings and departures

- **The registration's readings** (registration §3), each taken: the rule vectors under libm; the
  field and the resolved share; the load checks with the basket item; the orders; the instance;
  the dials and `rate.switch.*`; genesis; the observables and the reserved hours; the start
  distances; the grammar and lists; the readouts; the tape; E0's comparison.
- **The M5 list is unchanged.** A switch pop offers its pool's labour in full, and M5 asks such a
  good to die within the tick; the `Instant` check on the pool's good (as on the pop's own labour)
  is stronger and comes first, so the pool's good is not added to `sold_in_full`, where no test
  could see it.
- **`Genesis`, `Displacement` and `Point` gain fields** (`switch`, `reserved`, `pooled`,
  `pool_share`, `switch_distance`); the stocks probe's flow path (`horses::setup`) writes an empty
  `switch` where it builds a markets setup; its genesis is unchanged (the horses' tapes and
  streams, §6.1).
- **The wall's header** is IW1's text unchanged; a switch instance's names its frame, step and
  test.

## 8. How to run

```sh
markets-tape --inst is1 tapes/markets-is1.ron      # the committed tape
markets point --inst is1                           # unit 1d's point with E's efficiencies, and a*
markets list battery --inst is1                    # the 109 runs with their tiers
markets family battery --inst is1 --ticks 22000 --jobs 46 --csv DIR
markets family tier3s --inst is1 --ticks 22000 --jobs 20 --csv DIR   # first-year D̂
markets family battery --inst is1 --ticks 22000 --set 'rate.switch.*=0.75' …  # the switch's rate alone
markets run 'sw[trained]=0.5' --inst is1 --ticks 22000 --csv DIR
markets family battery --inst is2 --ticks 24000 …  # the control
```

`stats.tsv` holds the switch's readouts as `switch.*` lines beside the wall's `wall.*`. The
scorer, its gather script and its job list are committed before the wave (decision 311).
