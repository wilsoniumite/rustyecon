# TRAP-RULES: the subsistence trap's remedy, participation at a rate, as built

Dated 2026-09-30. Step P2.4.5 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`, scratch
`D:/rustyecon-p24/build-trap/`). It is the build of the trap scan's choice,
[docs/probe/trap/SPEC.md](trap/SPEC.md) (sha256 `1dba64d3…7ff7` as registered), registered at
P2.4.4 ([results/trap/registration.md](results/trap/registration.md)) before any of this code
existed; decisions 404–408. It says what was built, where the build reads the spec, and what was
checked. The spec's §5 is the specification; this file does not restate its argument, only what
the code does.

## 1. What changed, and what did not

- **Core, markets and the engine: nothing.**
- **Agents: one optional field**, the workers' exit's `pace` (decision 404), and no new kind,
  state or market rule:
  - `RawPricedExit.pace: Option<RawPace>`, `RawPace { adjust, share }`: `adjust` a param key,
    `RatePerYear`, live, read as a `Share` (a = −expm1(−rate/tpy)); `share` the workers' share at
    genesis, an inline number in [0, 1].
  - Resolved `PricedExit.pace: Option<Pace>`, `Pace { adjust: Site, share: f64 }`.

  Both are `#[serde(default, skip_serializing_if = "Option::is_none")]`, as the exit itself, so a
  tape without the field keeps its canonical text, `tape_hash` and `world_id`. `WorkersState {
  share }` is unchanged: under the pace it holds the paced share, which the rule reads; at genesis
  it is `pace.share` (`genesis_state`), and 0.0 without a pace, as before. `BasketWorkers::sites`
  lists `exit.pace.adjust`.
- **Probe:** `probe::markets` gains the paced instances C1P, C2P and C1PN (C1, C2 and C1N with
  `Commons::paced`), the dial `adjust.participation.workers`, the genesis share and its
  displacement, the generator's pace block and header, the grammar's `part.workers*F` and
  `part.workers=V`, the family `pace`, and the readouts `part_target` and `pace.*`. P2.1's, the
  wall's and the commons' instances keep every number, name and file they had (§6.1).
- **Tapes:** `tapes/markets-c1p.ron` and `tapes/markets-c2p.ron`, `markets-tape --inst c1p` and
  `--inst c2p`'s output. C1PN is `--inst c1pn`, with no committed tape.

## 2. The rule

The spec's §5.2, as `crates/agents/src/roles/many/rules.rs` (`BasketWorkers::decide_with_exit`)
makes it. After `p = workers_participation(...)`, unchanged, with N = `p.heads` and s₀ the
workers' own state share:

```
without pace:  hours = p.hours;                  share = p.share         (P2.3's, bit for bit)
with pace:     a = param(pace.adjust)            (a Share: −expm1(−rate/tpy))
               F* = p.hours / N                  (the rule's hours over the heads)
               s  = s₀ + a·(F* − s₀)             (the technique's form, rounded as written)
               hours = N·s;                      share = s
```

The hours are minted and offered as before. The plots are the rule's, `p.plots`: their land buy,
budget chain and burn are unchanged. The state holds `share`. Nothing else in the role changes.

*Note added at P2.4.20 (2026-10-01; the fidelity review, minor).* Away from rest the paced hours
and the rule's plots do not share one time budget: the plots are sized for the rule's hours F\*,
while N·s hours are offered, so the heads used can pass N. In the mirror (a copy of `tm.py`, Pr
1.3), C1P's `p[mach]*0.5` at rate × 0.9, a CONVERGED run, uses up to 1.11·N on 1,117 ticks,
82 of them with plots rented. C1P's `JB(2)` at tilt 2, a trap control, uses 1.12·N on 816 ticks.
On those ticks up to 0.975 of T_p is rented for heads still offering hours. In the Commons and
Crowded regimes nothing is rented, so this is bookkeeping. In the Enclosed regime it rents land
no one farms. The engine's paced run at that setting rents at most 0.073 of land 10
(`commons.tp_max`), so the effect on the land market is small. P, the plots at the hours offered,
is the consistent form, and the scan found it equal in class. A later registration that pairs the
pace with enclosed plots should prefer P (O116).

The rule reads posted prices (through the participation rule), its own state and its params
(R13). s is a convex combination of s₀ and F\*, both in [0, 1], so no clamp is needed (R3): in
floating point s₀ + fl(a·fl(F\* − s₀)) with a ≤ 1 cannot pass F\* by a rounding that reaches the
next double above 1, nor fall below 0 when F\* = 0, and core's check that a state share is at most
1 never fires (`many_roles_never_overbudget_or_overdraw` draws the rate up to 1e4 a year, a = 1).
With `pace` absent no new branch runs (R1). The rule divides by N, as the mirror; no instance has
N = 0 (registration §3).

## 3. The instances, the dial, genesis and tapes

- **C1P, C2P and C1PN** (`Instance::named`; decision 405) are C1, C2 and C1N with
  `Commons::paced` set, their own ids and titles. Their oracle, targets, cost coefficients,
  genesis prices and coins are their twins' (`paced_grammar_dials_and_families_are_the_registered_ones`
  checks the three points equal).
- **The dial** (decision 406): `adjust.participation.workers`, 1.3 `RatePerYear`, basis
  `Assumed("the trap scan's §4: participation at land's and the types' rate")`, written in C2m
  after the techniques' `adjust.technique.*` at a paced instance only. `--set adjust.*=f` scales
  it with them; `--set adjust.participation.workers=v` sets it.
- **Genesis**: the workers' share is the point's S/N, unit 1e's supply S over N per tick
  (`Genesis::pace` holds (S/N, the share written)), displaced as the grammar says (§4).
- **The tapes'** workers' block writes `exit: Some((good: "food", …, land: "land", pace:
  Some((adjust: "adjust.participation.workers", share: 0.13461538461538466))))` at C1P (C2P:
  0.14681190682890408, the point's), and the header names the pace, the step P2.4 and the test
  `commons_paced_tapes_are_their_generators_output`. Against C1's and C2's tapes the paced tapes
  differ in the header, the name, the dial line, the workers' block and the genesis basis's id.

## 4. The harness at a paced instance

- **Grammar** (registration §3): `part.workers*F` sets the genesis share to min(F·S/N, 1), as the
  scan's `displace`; `part.workers=V` to V; both are refused at an instance without a pace, and a
  share outside [0, 1] is refused at genesis. Parsed before the `C=V@when` form. Its start is a
  stock's (`Start::Stocks`): |ln(share/(S/N))| over the tolerance, or with `--first-year` the
  first scored year's largest D̂.
- **The family `pace`** (the spec's §8 E4): `part.workers*0.1`, `*0.5`, `*2`, `=0`, `=1`, in that
  order. `markets family pace` sets `--first-year`, as `family tier3s` does, since the prediction
  reads its start so (`predict.py`'s `fy`). The other families and the battery are C1's and C2's
  lists, equal to the registered runs name for name.
- **Readouts.** The CSV's `part_workers` is the workers' state share (the paced share, the hours
  offered over N); a new last column `part_target`, at a paced instance only, is the rule's F\* at
  the tick's posted prices (`Plots::target`, through `workers_participation`). The regime, the
  shadow rent and T_p stay the rule's. `stats.tsv` adds, at a paced instance only,
  `pace.gap_max` (the largest |ln(F/F\*)| over the scored run), `pace.low` (the least F/F\*) and
  `pace.zero_hours` (the scored ticks with labour's supply 0). `summary.tsv` keeps its 30 columns.

## 5. Load checks

Each is a `LoadError` at `actors[workers].spec.exit.pace…` (`pace_is_checked_at_load`):

- `.share`: finite and in [0, 1] (the roles' `share` check; NaN, ±inf, −0.1 and 1.5 refused);
- `.adjust`: a `RatePerYear` param (the resolver's `UnitMismatch`); its value finite and not
  negative is the registry's (`params[adjust.participation.workers].value`).

The block's two fields are required and no other is read.

## 6. Checks of the build

All on WSL in release unless said; scripts and outputs in `D:/rustyecon-p24/build-trap/`.

### 6.1 R1: every old tape, id and stream

- **Streams.** Every committed tape before the build (the 25 of `tapes/`) run 2,000 ticks through
  the `rustyecon` binary gives the same `--hashes` file, header and stream as the binary built at
  `e29b9c6` before any change, on WSL and on Windows (`streams.sh`; `streams-*.log`). The two
  base streams equal each other.
- **Ids and text.** `commons_paced_tapes_are_their_generators_output` pins C1's and C2's
  `tape_hash` and `world_id` to the pre-build binary's; `paced_genesis_state_is_the_tapes` pins
  C1's `world_id` in agents; the generator writes P2.1's seven tapes, IW1's, C1's and C2's byte for
  byte (their tests, unchanged).
- **Nesting.** `pace_absent_is_p23s` (agents): over 3,000 random states of C1's world, the
  workers without a pace offer and mint `workers_participation`'s hours and set its share, and
  their own state share is never read.

### 6.2 The rest point

`paced_rest_is_the_oracles`: at C1P, C2P and their 24 targets and at C1PN, at 12, 52 and 365
ticks a year, three ticks from genesis leave every observable within 1e-12 of unit 1e's point in
log, every market trading and filled, the paced share within 1e-15 of the rule's and of S/N, and
the readout in 1e's regime with its plot rent.

### 6.3 The rules and the harness

- `paced_share_moves_at_its_rate` (agents): over 3,000 random states of C1P's world (the own
  share uniform, the rate over three decades), the decision offers and mints N·(s₀ + a·(F\* −
  s₀)), sets that share and posts the rule's plots; a = share(rate) exactly.
- `paced_workers_leave_the_trap`: at every price rate × 0.9, C1's `p[mach]*0.5` has ticks with no
  hours and runs away at tick 325; C1P's converges from tick 723, the mirror's, with no tick
  without hours.
- `harness_reads_the_paced_share`: on C1P displaced (the machine × 0.5, the commons at 12.15 a
  year, the share at 0.3 of S/N), tick by tick against an independent run of the same tape, the
  labour supply is N·s, the state s, `part_workers` s and `part_target` F\*, the land asked for the
  rule's T_p, and the `pace.*` readouts the lag's largest and least.
- `paced_grammar_dials_and_families_are_the_registered_ones`: the dial and `adjust.*`, the
  grammar, the start distances, a tick without hours counted, the families against
  `registered/runs.jsonl`, every run's tape.
- `markets_runs_the_pace_family_and_reports_the_pace`: the binary's `family pace` reads the first
  year's D̂ and writes the `pace.*` lines, at a paced instance only.
- `commons_paced_tapes_are_their_generators_output`; `each_site_converts_as_registered` and
  `many_roles_never_overbudget_or_overdraw` read C1P's tape too.

### 6.4 Each test fails without its change

`mutants/mutate_p.py` undoes each change in turn and runs the agents' `pace`, `many`, `seam` and
`commons` tests and the probe's `trap`, `commons`, `markets` and `wall` tests: 32 mutants (the
rule's branch, F\*, the hours, the state, the unpaced path, the rate, the jump; the raw and the
resolved skip, the share check, the adjust's method, the site, the resolved pace, the genesis
state; the instance's flag, the dial's reach and value, the displacement, the cap, the share
written; the readout's F\*, the CSV column, the stats' push, the zero-hour count, the start
distance, the stats' reach; the start's kind, the grammar's refusal, `=V`, the family's order;
the binary's first-year and stats lines). All 32 are killed by the committed tests
(`mutants/mutants-final.out`). The first pass (`mutants-1.out`) left one alive, and its test was
strengthened before this commit: the readout's F\* taken as the rule's share rather than hours/N
is the same double when N is a power of 2, and N is 4 a tick at 52 ticks a year, so
`harness_reads_the_paced_share` now runs at 12 ticks a year too (N = 208/12), where the two part
in some ticks and the readout must be the workers' hours/N.

The rest point and the trap run kill the mutants that move the map; the rule, load, grammar,
family, readout and id tests kill the rest. Two mutants are seen by one test only: F\* taken as
the rule's share in the rule (`paced_share_moves_at_its_rate`, whose heads are drawn off a
power of 2) and the site left unlisted (the same test).

### 6.5 Development runs before E0

The build was exercised on the engine before E0: `markets run hold` for 3,000 ticks at C1P, C2P
and C1PN, and `p[mach]*0.5` at every price rate × 0.9 at C1 and C1P (DIVERGED at 325; CONVERGED
from 723, the mirror's); and a development trace diff (`e0/tracediff-dev1.out`) on the uncommitted
build, with E0's eleven runs: no parting above 1e-12. Disclosed here, as the commons' and the
wall's were.

### 6.6 The gates

`scripts/gate.sh` and `scripts/gui.sh` are green on WSL (`/root/scratch/target-p24`) and on
Windows (`D:/rustyecon-targets/p24`), 1,019 tests passed and 4 ignored on each (P2.3's 1,009 and
this build's 10). The gate hash is `0x61f9c8529131ff17`, and gui.sh's hash check gives appb
`0xe1fa082b26995867` and demo-gb `0xfad880fe08d06645`, on both. The stamp is `e29b9c6`, dirty:
this build before its commit. The two new tapes' `tape_hash` are `0x9b48c13de53083fd` (C1P) and
`0x089042f8c45c28f0` (C2P), their `world_id` `0xad31767448313bd3` and `0x39915cb52d06d287`, and
their 2,000-tick finals `0x7d5133fb7b903a8f` and `0xd5ad10fe6f6dfc86`, on both machines. Logs in
`D:/rustyecon-p24/build-trap/` (`gate-*.log`, `gui-*.log`, `streams-*.log`).

## 7. Readings and departures

- **The registration's readings** (registration §3), each taken: the field and the dial; the
  genesis share from the harness's oracle; `part.workers*F` and `=V`; the pace family's
  first-year start and order; the names; the tapes; the readouts; E0's comparison.
- **The header of a paced tape** names its step (P2.4) and its test; C1's and C2's headers are
  unchanged.
- **`Genesis` and `Displacement` gain a field each** (`pace`), and the stocks probe's flow path
  (`horses::setup`) writes the undisplaced value where it builds a markets setup; its genesis is
  unchanged (the horses' tapes and streams, §6.1).

## 8. How to run

```sh
markets-tape --inst c1p tapes/markets-c1p.ron      # the committed tapes (and c2p)
markets point --inst c1p                           # unit 1e's point, as C1's
markets run 'p[mach]*0.5' --inst c1p --set 'rate.*=0.9' --ticks 141000 --csv DIR
markets family battery --inst c1p --ticks 141000 --jobs 46 --csv DIR
markets family pace --inst c1p --ticks 141000 --csv DIR      # first-year start, as tier3s
markets family stocks --inst c1p --ticks 141000 --first-year --csv DIR   # as the prediction reads it
markets family battery --inst c1pn --ticks 141000 …          # the stress control, at C1's L
```

`stats.tsv` holds the pace's readouts as `pace.*` lines beside the commons' `commons.*`. The
scorer, its gather script and its job list are committed before the wave (decision 311).
