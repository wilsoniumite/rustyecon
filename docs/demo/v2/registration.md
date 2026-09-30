# Demo v2, stage v2a.1: the mirror's predictions, registered

Dated 2026-09-30. Step D2.0 (label `design`) on branch `demo-v2`, from `reboot` at `f7d1eae`. This
registers the mirror's predictions for the stage's battery and long run before any code of the pass
exists: at this commit `crates/worldgen` has no stage, `tapes/` holds no v2 tape, and no engine run
of the stage has been made. The design is [../WORLD-V2.md](../WORLD-V2.md); its §2–§3 fix the stage,
its §11 the battery and how the engine is scored. The CSVs beside this file are written by
`D:/rustyecon-d2/design/tools/make_registration.py` from the mirror's outputs; this file quotes
them.

## 1. What is registered

**The stage**, as WORLD-V2.md at this commit sets it: each county's v1 row under GOODS-CHAIN's rule
A (fodder ω·b land; a head a·κ/δ own horse-days, λ·κ/δ labour, (1 − ω)·b·κ/δ pasture), δ 8% a year,
ω 0.85, κ 52 a year, J_b 1, ρ 0; C2g (labour's, fodder's, the horse's and the horse-day's price
rates 5.2 a year, land's 0.1625, the good's 2.6), s_K 2δ, s_Km 0, the maker's cover 4 weeks and
reservation ψ 0.25, ex-post assignment, 52 ticks a year from 1750-01-01; genesis at each county's
oracle point; v1's history mapped by WORLD-V2 §3.

**The mirror**, `D:/rustyecon-d2/design/mirror/` (sha256):

```
042f12b24ce5f8b742b8923b677f57dce3e26b3ca3b898f83e3c308e8fdd428e  model/h_mirror.py   (P2.2a's frame map, copied unedited)
0790cb18e9101eab2997313ccb56a79d80626d69e2da7dd66f1ff6007f74dbaf  model/h_run.py      (its scoring, copied unedited)
54cec56abe129ae0708422b9f57d2ac97aa87fb8aeff8df3af5ebc1d36971cd3  model/h_battery.py  (its battery list, copied unedited)
26563491c40f0e056f571bfcdfac80f84b5d70292208077a99cb2a4c4267a03b  model/_path.py
79bc91e05976ec11be1c39f3213ac3d877336f88876054a8dc40f8ed45d6c5a8  model/i_mirror.py   (IDLE's reservation, copied unedited from D:/rustyecon-p2l/idle-scan/)
1c5e85d9c50313d927a8fe365ed058fb465c57aa3c0afda10bb2ac03b3a28fd9  make_i_carry.py     (copied unedited from D:/rustyecon-p2l/idle-engine/tracediff/)
bbece84c104bcf55c4a2a21151c2dfdcc63d60c971f602dcb65f7071ae55a16d  i_carry.py          (its output: the reservation with the engine's genesis carry; byte for byte the registered one)
7b7d0e20654c92e0ffa7edebac91e9757442c59e08594183a466ccd1767b6c2e  ag/model.py
bbf0acb59afc3985272ecb8893c6ba92f8925adadf259545f93261e20195c8bb  ag/goods.py
cabaee71b0b245f62d4bd975a9870b921a861e47f2b4790e1d6babc7a000fed7  ag/designs.py
600110582823759543612a4fe8106d3cf9450e73f31bceb1308e68f5ac052a23  d2_world.py         (new: the 93 counties under rule A, C2g, psi 0.25)
c841693f5d8915c22fe4834ce55399e3af4319aee544d989b0d2f80511632942  d2_long.py          (new: the long run)
48b59bd18715d1ac480d9606bdc446e629e4817e0751a72c61327d00c8a46bfd  d2_modes.py         (new: the flow controls)
3ba7d788263b27997f4a57fe441381f0e935bd07e4ddda976ac95cb4ace40733  d2_lin.py           (new: local growth)
4b61ab8bba0aef642555e82602f037ecebb8c450862b347e0424616bb9d9187c  d2_battery.py       (new: the battery)
```

**Its input**, `D:/rustyecon-d2/design/out-d08/counties.json` (sha256
`fe2fc0d745aaeaa08a52b75b1a1265009a4081f78f9bf408814cd83050832200`): every county's genesis row and
every step (the tick it fires in, its month, its param, its value), and its 1g point at genesis,
dumped from the compiler's own plans by `d2-fund` (`D:/rustyecon-d2/design/fund/`, `src/main.rs`
sha256 `7508fbac51d1ffac80f697b6147687a71f2f38dfceee92c54aed2f9055be77a6`) on this worktree's
worldgen, oracle and probe at `f7d1eae`.

**The predictions**, in this folder (sha256 in [SHA256SUMS](SHA256SUMS)):

| file | rows | what |
|---|---|---|
| `long-run.csv` | 93 | the long run per county (§3.4) |
| `battery-runs.csv` | 51,894 | the battery run by run: class, ticks to tolerance, dead, withheld and shortfall ticks, lowest baskets, heads and horse price (§3.3) |
| `battery-county-dates.csv` | 558 | the battery per county-date |
| `growth.csv` | 2,790 | local growth per point: 558 county-dates, base and four cost targets (§3.2) |
| `funding.csv` | 93 | funding per county (§3.1) |
| `targets.csv` | 2,232 | every cost target's margin, x\*, v and participation |

They are written from the mirror's outputs by `D:/rustyecon-d2/design/tools/` (sha256):

```
c75831741696088ec6b36aba499a001d8d4b89a41e2a930df36c43d36b576200  make_registration.py  (the CSVs)
246c72e0562c667ed4f62afed2657f1e2d862319d3735086fcd5ad890b019357  long_table.py         (long-run.csv's rows)
e477bf920cc699d9c396e53172c7e18523294abc84c14999b307bf62182a7bc6  fund_table.py         (funding.csv's rows)
```

Every other readout of each battery run (peaks, idle and no-order ticks, the maker's lowest markup,
the quasi-rents, the heads' 5% time) is in the scratch file
`D:/rustyecon-d2/design/out-mirror/battery.jsonl`, sha256
`f1bb332c98f26ae9bc474f7cc67fd71d754792b502055a029603a69911d0dbef`.

## 2. Checked before any prediction was taken

- **The mirror's oracle against 1g**: at every county's genesis, within 1.3e-15 relative on x\*, v,
  p, p_f, p_K, p_h, Y, N_a, q_f, q_b, the heads and the provider's baskets.
- **The rest point**: three ticks from every county's genesis move no coordinate more than 9.9e-16.
- **The battery's port** (`d2_battery.run` on `i_carry.tick`), with the reservation and the carry
  off, against P2.2a's frame on F5 (`h_battery_F5.json`, L 27,000): 91 of 91 runs equal in class,
  ticks to tolerance and dead ticks.
- **The history's driver against v1**: the stocks layer off (δ = 1 a tick, ω 0, the flow path), C2,
  planned: the county medians' median D̂ 10.8, Glamorgan's 34.1 and 56.3, the lowest cleared volume
  0.945 (Glamorgan), no dead tick and no shortfall: v1's committed long run (WORLD.md §8).
- **Reproducible**: the long runs and the growth scan rerun from the files above give their outputs
  byte for byte, and so do two county-dates of the battery (186 runs).

## 3. The predictions

Conditions of every number here, unless its line says otherwise: the stage of §1, the mirror of §1,
52 ticks a year; D̂ is the largest |ln(o/o\*)| over P2.2a's observables, without the horse market's
volume where the line says so, in units of 1e-3; o\* is unit 1g's point at the params in force (the
mirror's unit 1a on rule A's fold, within 1.3e-15 of it).

### 3.1 Funding (1g, `funding.csv`, `targets.csv`)

- All 93 counties are interior and funded at genesis and at all 25,480 step dates; the least margin
  is 0.1190 baskets per unit of N (Surrey, 1834-04-01).
- At the six battery dates, b × 1.1, × 0.9 and × 0.5 are funded at all 558 county-dates (least
  0.0498, Surrey 1825); b × 2 at 241 (unfunded at 81, 71, 68, 55, 21 and 21 counties in 1750, 1800,
  1825, 1850, 1875 and 1900). An unfunded target is not run.
- 1g equals v1's 1a point within 8.0e-16 relative on x\*, v, P_s, Y, N_a, p and the horse-day's
  price, and 5.3e-15 on the provider's baskets, at every point.

### 3.2 Local growth (`growth.csv`)

At all 2,473 funded points the largest growth of a displacement per tick is below 1: at most
0.9984870 (0.9243 a year; Brecknockshire 1850, b × 0.5), and 0.9197–0.9227 a year at the 558 base
points. The rest residual is at most 3.4e-15. Each county-date's base kick set is predicted to decay
(E4).

### 3.3 The battery (`battery-runs.csv`, `battery-county-dates.csv`)

P2.2a's 93 runs at each of the 558 county-dates, funded targets only, L 84,000 (dated shocks at
L/4), one length:

| | predicted |
|---|---|
| classes | 51,260 CONVERGED of 51,260 scored; 634 unfunded b × 2 runs not a target; every county-date GO |
| end gap | at most D̂ 2.4e-10 |
| ticks to tolerance | median 3,010, 90th percentile 4,526, largest 5,661 (West Riding 1850, heads × 2) |
| by tier, median and largest | Tier 1 1,770 and 3,798; Tier 2 2,852 and 4,629; Tier 3 3,793 and 5,519; Tier 3S 3,403 and 5,661 |
| dead ticks | in 6,479 runs, all in Tier 3 and 3S and none in a run's second half; largest 277 (Suffolk 1825, b × 0.5, horse-days); the most in one run by market: horse-days 277, fodder 46, the good 37, land 21, labour 9 |
| the idle horse market | idle in 19,589 runs, at most 418 ticks (Lancashire and the West Riding 1900, heads × 2); at most 88 ticks without an order |
| the reservation | acts in 2,803 runs (every county-date's r × 2, N × 2, JB × 0.5, heads × 2; all 482 funded b × 2; 55 p × 0.5; 33 x\*/2; one JA × 2); withholds at most 49 ticks (Lanarkshire 1900, heads × 2) |
| the horse's price | lowest 0.200 of target (West Riding 1900), highest 7.65 (Lancashire 1900, b × 0.5); the maker's lowest markup 0.212 (Selkirkshire 1875) |
| transfer shortfall | in 5,905 runs; at most 2,142 ticks (Clackmannanshire 1825, b × 2); in 106 of the 482 b × 2 runs |
| troughs | lowest baskets 0.311 of Y\* (Norfolk 1800, r × 0.5); heads 0.544 to 1.90 of target |
| capital's time (unscored) | heads within 5% after b × 2 in 8.0–25.8 years (median 9.5; the paper 7.1), after b × 0.5 in 34.7–40.9 (the paper one tick), after b × 1.1 in 0.8–2.4, after b × 0.9 in 5.0–9.5 |

### 3.4 The long run (`long-run.csv`)

Each county from genesis through the first tick of 1901 (7,852 ticks), with its history; D̂ without
the horse market's volume:

| | predicted |
|---|---|
| dead ticks (labour, land, fodder, horse-days, the good below half their oracle volume) | 0 in every county |
| idle horse-market ticks; ticks without an order; ticks the maker withholds | 0 in every county |
| transfer shortfall ticks | 14,502 in six counties: Surrey 5,593, Armagh 3,619, Caernarfonshire 2,320, Sussex 1,479, Down 1,115, Tyrone 376; 0 elsewhere |
| lowest cleared volume over its oracle's | 0.606 (Glamorgan, horse-days, tick 5,893); then Lanarkshire 0.614, Durham 0.623 |
| lowest baskets over Y\* | 0.663 (Glamorgan) |
| D̂, county medians | median 116.7; highest Lanarkshire 382.7, Lancashire 376.9, Glamorgan 361.9, Durham 330.8; lowest Peeblesshire 38.2 |
| D̂, largest | 543.5 (Glamorgan) |
| D̂ at the first tick of 1901 | median 60.4; largest 362.4 (Glamorgan) |
| heads over their equilibrium | lowest 0.717 (Glamorgan, 1863), highest 1.115 (Cornwall); the capacity desk's alone 0.611 to 1.059 |
| the capacity desk's heads over its own target | 0.871 (Lanarkshire) to 1.063 (Cornwall) |
| the horse's price over its target; the maker's markup | 0.989 to 1.722 (Glamorgan); markup at least 0.981 |
| the horse-day's markup over full cost | −0.076 (Cornwall) to +0.233 (Glamorgan) |

### 3.5 The controls (reported, not scored)

The same counties and history with the stocks layer off: under v1's C2 with planned assignment, v1's
long run (county medians' median D̂ 10.8, largest 56.3, no dead tick or shortfall); under C2g with
ex-post assignment, the like-for-like flow county (median 28.4, highest Glamorgan 123.7, largest
207.8, no dead tick or shortfall, lowest cleared 0.812). Per county in `long-run.csv`.

## 4. How the engine is scored

WORLD-V2 §11.5, quoted:

- **E0, the trace diff**, before any scored run: five counties (Lancashire, Glamorgan, Surrey,
  Sutherland, Bedfordshire), each from the engine's own genesis on its tape, hold and w × 2 at 1850
  and b × 2 at genesis where funded, 2,000 ticks: every observable within 1e-9 in log, with decision
  304's reading of the horse volume's cancellation. A parting at tick 1 stops the run for an
  amendment.
- **E1, nesting**: WORLD-V2 §9.3 (v1's tape and pins; the flow path is v1 bit for bit).
- **E2, rest and mode A**: three ticks from each county's genesis within 1e-12 of the oracle; mode A
  at each county-date to L, every observable within 1e-9 in log.
- **E3, the battery**: every run's class the mirror's; ticks to tolerance within 10% of the mirror's
  run for run, but at most three runs a county-date within 25%; dead ticks within 5 or 10%,
  whichever is larger; the lowest baskets within 1e-3.
- **E4, the kicks**: every county-date's base kick set decays, its g below 1 a tick.
- **E5, the long run**: per county, dead and idle ticks the mirror's (0), withheld ticks within 5,
  shortfall ticks within 5 or 1%, whichever is larger; D̂ median and largest within 5%; the heads'
  lowest over the oracle within 1e-3 absolute.
- **Refutation**: a class other than the mirror's at any county-date; any dead tick in the long run;
  a runaway with ψ 0.25; a CONVERGED run ending more than 1e-12 off its oracle point; a kick set
  that grows.

A reading of any line taken after engine output is seen is a dated amendment with its own sha256,
before any scored run (decisions 282, 310).

## 5. What was read before registering, disclosed

- The mirror's long runs (WORLD-V2 §6), its two dial families (land's rate 1.3, s_K 4δ) and its flow
  controls were run and read while the design was written. They shaped WORLD-V2 §8's lens design and
  the open items O81–O82; no dial of the stage was changed after them.
- The battery's partial outputs were read while it ran (the classes, and counts of dead, withheld
  and shortfall ticks by run name) to check that it ran. No rule, dial, run list or tolerance
  changed after.
- No engine run of the stage exists. The engine runs of this step were v1's tape (the GUI's smoke
  mode, twice, and `rustyecon run` once, for WORLD-V2 §9.4's timings) and three committed
  single-county tapes (appb, markets-i0, horses-h1) for 52,000 ticks each, for the cost of a tick.
