# The demo's second pass on the engine: the battery and the long run, scored

Dated 2026-09-30. Step D2.4 on branch `demo-v2`, label `run`. Scratch `D:/rustyecon-d2/run/`; raw
runs `D:/rustyecon-d2/runs/`. This runs WORLD-V2 §11.5's protocol on the committed build and scores
it line by line against the registration: [../registration.md](../registration.md) (sha256
`972c7d21…a5b0`) and its amendment [../registration-A1.md](../registration-A1.md). The design is
[../../WORLD-V2.md](../../WORLD-V2.md).

- **D2.4 `8e8781f`.** It holds the harness's county-dates, the long run's scorer, and this folder's
  tools and job list. All were committed before the first job (decision 311). E0–E2 ran before that
  commit, and amendment A1 set L per county-date.
- **D2.4b.** It holds the wave's results, this README, the figures, and `demo_v2_runs_to_1901` in
  `scripts/gate.sh`. No rule, tolerance, run list, scoring tool or binary changed after the first
  result was read. No engine or compiler bug was found, so there is no fix commit and no rerun.

**Corrected at D2.5 (2026-09-30), after the two bounded reviews.** No scored line, class, count or
verdict moves. Four statements were wrong or incomplete as written:
- **What had been read before A1.** A1 §A1.3 says no long-run output of the stage had been read,
  and that the long-run test had run only with the history removed. That is false. The long run
  had been run and read before D2.4: D2.2 pinned its final hash through 1901 (`demo_v2_pin`), D2.3
  timed the GUI to 1901, and D2.3b's table and two screenshots of the herd against its equilibrium
  (WORLD-V2 §16) came from the engine's run of the committed tape. The E5 scorer (D2.4) was written
  after that. E5's lines and tolerances are the registration's of D2.0, unchanged, and the engine
  meets them to printed precision. A1 is left as registered (its sha256 stands); this note
  corrects it.
- **The horse's lowest price.** The 0.200 of target and the maker's 0.212 in "Prediction against
  result" are posted prices, on ticks when the maker withheld and no horse traded, as the
  registration read them (over every tick). The map's horse-price lens shows no value on such a
  tick. In the West Riding 1900 J_b × 0.5 run, rerun on Windows with every tick written (D2.5's
  world review), the low 0.2003 falls at tick 36 with no trade; the lowest on a tick with a trade
  is 0.2188 (tick 35), and the markup's lowest is 0.2304 over all ticks and 0.2501 over traded
  ones.
- **"Healthy."** The long run has no dead, idle, no-order or withheld tick, and its ledger closes.
  That is all the word meant. It is not near its equilibrium: the county medians' median D̂ is 116.7
  against v1's 10.8, about 11 times, and the battery's own time to tolerance is on the history's
  scale (median 3,010 ticks, 58 years; largest 5,661, 109 years). The section is renamed below.
- **Capital's lowest herds** were quoted from every 13th tick. Read every tick
  ([long-run-engine.csv](long-run-engine.csv)): Glamorgan 0.717 (1863.3), Lanarkshire 0.725
  (1850.8), Durham 0.728 (1863.3), Renfrewshire 0.743 (1779.3), Lancashire 0.743 (1851.5).

Conditions of every number here, unless its line says otherwise:
- stage v2a.1 as registered: rule A, δ 8% a year, ω 0.85, κ 52, ρ 0, J_b 1, C2g, s_K 2δ, the
  maker's cover of 4 weeks and reservation ψ 0.25, ex-post assignment, 52 ticks a year;
- the battery at each county's instance on 1 January of 1750, 1800, 1825, 1850, 1875 and 1900,
  at L 84,000 to 95,000 by county-date (A1);
- the long run from 1750-01-01 through the first tick of 1901 on `tapes/demo-gb-v2.ron`;
- D̂ in units of 1e-3 in log.

## Verdict

**Every scored line holds, and no refutation criterion was hit.** All 558 county-dates are GO. The
scorer read 21 lines, and all 21 pass ([lines.csv](lines.csv)).

**At every county-date the engine is the mirror to the tick.**
- **Class.** All 51,260 scored runs CONVERGE, as the mirror predicted.
- **Ticks to tolerance.** 51,252 runs match the mirror's to the tick. Eight differ by one tick (all
  of them heads × 2, near 5,500 ticks), and none is off by more than 0.02%.
- **Dead ticks.** 50,635 runs match. The other 625 differ by one or two ticks, all in five Tier 3S
  runs that halve a stock or a coin, and each is a tie at the dead bar (below). The tolerance is 5
  ticks.
- **Lowest baskets.** Within 7.6e-5 of the mirror's in every run.
- **The withheld and shortfall ticks** match in every run.
- **The rest differ only in the glut.** In the heads × 2 runs, the maker withholds and the horse
  market flips between offer and order (the chatter of O90). There:
  - idle ticks differ by 1 or 2 in 5 runs;
  - ticks without an order differ by 1 to 4 in 18 runs;
  - the heads' 5% tick differs by 1 or 2 in 16 runs (three of them b × 2);
  - the heads' trough is within 7.2e-5 and the horse price's within 1e-6.
  Both troughs' largest gaps are at East Lothian 1900, heads × 2. None of these readouts is scored.

**The long run is the mirror's to rounding in every county.** No county has a dead tick, an idle
horse-market tick, a tick without an order, or a withheld tick. The shortfall ticks are the
registered 14,502, county for county. D̂'s median, 90th percentile, largest and end value equal the
registered table to its printed precision. Every 13th tick, the herd's ratio to its equilibrium
matches the mirror's within 2.0e-14 in all 93 counties.

**What the registration named as E0 is a miss at six runs, reported before the wave**
(registration-A1 §A1.3; decision 357; O90). It did not stop the run, and every one of those
county-dates is GO here.

## The scored lines

| E | registered | engine | |
|---|---|---|---|
| E0 trace diff | every observable within 1e-9 in log (decision 304's cancellation); a tick-1 parting stops the run | nothing parts at tick 1. Hold and w × 2 at five counties agree within 2.2e-13. The six b × 2 runs part after the maker withholds, up to 8.5e-2 (A1 §A1.3) | miss at six runs, reported (O90) |
| E1 nesting | v1's tape and pins; the flow path is v1 bit for bit | `demo_tape_is_its_compilers_output`, `demo_runs_to_1901` (`0xfad880fe08d06645`, stream `0xdb63cc96f769fb3e`) and `v2_flow_path_is_v1` pass on both machines. The gate pin is `0x61f9c8529131ff17` | holds |
| E2 rest and mode A | three ticks within 1e-12 of the oracle; mode A at L within 1e-9 | rest within 5.6e-14 (`v2_rests_at_every_county_oracle_point`); mode A PASSES at all 558 county-dates at their L, largest gap 8.4e-14 ([modea.csv](modea.csv)) | holds |
| E3 the battery | each class the mirror's; ticks to tolerance within 10%, at most three runs a county-date within 25%; dead ticks within 5 or 10%; baskets within 1e-3 | 51,260 of 51,260 classes; 0 runs beyond 10%; 0 dead-tick misses; baskets within 7.6e-5 | holds |
| E4 the kicks | every base kick set decays, g below 1 a tick | 558 of 558 PASS; the largest g is 0.997448 a tick ([kicks.csv](kicks.csv)) | holds |
| E5 the long run | per county: dead and idle ticks 0; withheld within 5; shortfall within 5 or 1%; D̂ median and largest within 5%; the heads' lowest within 1e-3 | 93 of 93 counties on each line | holds |
| refutation | a class other than the mirror's; a dead tick in the long run; a runaway; a CONVERGED run ending more than 1e-12 off; a kick set that grows | none; the largest end gap is D̂ 2.35e-10 (2.4e-13 in log) | none hit |

## By county class

The classes come from the world's own tags (`worlds/demo-gb/regions.csv`) and were fixed before
the battery's results were read. In order of precedence:
- **coal and textile**: coal or textile at least 0.5;
- **London's ring**: metro at least 0.1;
- **Highland**: highland at least 0.5;
- **rural**: the rest.

The full table is [by-class.csv](by-class.csv). In it:
- "engine" and "mirror" in the long-run columns are the class's median county median D̂, the
  largest D̂ in the class, and the lowest heads over their equilibrium;
- every class has 0 class differences, 0 runs off by more than 10%, 0 dead-tick misses and 0
  basket misses.

| class | counties | county-dates GO | runs CONVERGED | kick sets PASS | long run E5 | D̂ median (engine, mirror) | largest D̂ | lowest heads / equilibrium | shortfall ticks |
|---|---|---|---|---|---|---|---|---|---|
| coal and textile | 26 | 156 of 156 | 14,434 of 14,434 | 156 | 26 of 26 | 208.0, 208.0 | 543.5 (Glamorgan) | 0.717 | 4,734 |
| London's ring | 7 | 42 of 42 | 3,828 of 3,828 | 42 | 7 of 7 | 149.3, 149.3 | 444.4 (Surrey) | 0.764 | 7,072 |
| Highland | 5 | 30 of 30 | 2,762 of 2,762 | 30 | 5 of 5 | 80.2, 80.2 | 264.4 | 0.906 | 0 |
| rural | 55 | 330 of 330 | 30,236 of 30,236 | 330 | 55 of 55 | 107.3, 107.3 | 402.6 | 0.797 | 2,696 |
| England | 41 | 246 of 246 | 22,572 | 246 | 41 of 41 | 142.5, 142.5 | 531.5 | 0.728 | 7,072 |
| Wales | 13 | 78 of 78 | 7,164 | 78 | 13 of 13 | 120.4, 120.4 | 543.5 | 0.717 | 2,320 |
| Scotland | 33 | 198 of 198 | 18,218 | 198 | 33 of 33 | 91.3, 91.3 | 542.1 | 0.725 | 0 |
| Ireland (Ulster) | 6 | 36 of 36 | 3,306 | 36 | 6 of 6 | 152.9, 152.9 | 336.7 | 0.819 | 5,110 |

Every class is GO. The classes differ in the size of capital's lag over the history, not in whether
the agents find their equilibrium.

## Prediction against result

Registration §3.3 and §3.4 predicted these aggregates. The scorer computes each from both sides with
the same code, and they are reported, not scored (decision 359). The table is
[aggregates.csv](aggregates.csv).

| registered (the mirror) | engine |
|---|---|
| 51,260 CONVERGED of 51,260; every county-date GO | the same |
| ticks to tolerance: median 3,010, p90 4,526, largest 5,661 (West Riding 1850, heads × 2) | the same, the same run |
| by tier, median and largest: Tier 1 1,770 and 3,798; Tier 2 2,852 and 4,629; Tier 3 3,793 and 5,519; Tier 3S 3,403 and 5,661 | the same |
| end gap at most D̂ 2.4e-10 | 2.35e-10 |
| dead ticks in 6,479 runs, all in Tier 3 and 3S, none in a run's second half | in 6,638 runs, all in Tier 3 and 3S, none in the second half; the 159 more are ties at the bar (below) |
| most dead ticks 277 (Suffolk 1825, b × 0.5 dated); most in one run by market: horse-days 277, fodder 46, the good 37, land 21, labour 9 | the same |
| idle horse market in 19,589 runs, at most 418 ticks (Lancashire 1900, heads × 2); at most 88 ticks without an order | the same |
| the reservation acts in 2,803 runs; at most 49 withheld ticks (Lanarkshire 1900, heads × 2) | the same |
| the horse's lowest price 0.200 of target (West Riding 1900, JB × 0.5); the maker's lowest markup 0.212 (Selkirkshire 1875, b × 2) | the same; both posted on a tick when the maker withheld and no horse traded (corrected at D2.5, above) |
| the horse's highest price 7.65 of target (Lancashire 1900, b × 0.5) | not read: the harness writes `pk_high` only for the loop step's instances |
| transfer shortfall in 5,905 runs, at most 2,142 ticks (Clackmannanshire 1825, b × 2 dated); in 106 of the 482 b × 2 runs | the same |
| lowest baskets 0.311 of Y\* (Norfolk 1800, r × 0.5); heads 0.544 to 1.90 of target | the same |
| capital's time to within 5% of target: after b × 2, 8.0–25.8 years (median 9.5); after b × 0.5, 34.7–40.9; after b × 1.1, 0.8–2.4; after b × 0.9, 5.0–9.5 | the same, at genesis and dated |
| local growth at the base points 0.9197–0.9227 a year (the mirror's linearization) | the kick sets' fitted g is 0.856–0.876 a year (below) |
| long run: county medians' median D̂ 116.7; highest Lanarkshire 382.7, Lancashire 376.9, Glamorgan 361.9, Durham 330.8; lowest Peeblesshire 38.2 | 116.72; the same four, to the printed digit; Peeblesshire 38.2 |
| long run: largest D̂ 543.5 (Glamorgan); at 1901 median 60.4, largest 362.4 (Glamorgan) | the same |
| long run: lowest cleared volume 0.606 (Glamorgan, horse-days), then Lanarkshire 0.614, Durham 0.623 | the same |
| long run: heads over equilibrium lowest 0.717 (Glamorgan), highest 1.115 (Cornwall); the desk's own target at least 0.871 (Lanarkshire) | the same |
| long run: the horse's price over its target at most 1.722 (Glamorgan); the maker's markup at least 0.981; the horse-day's markup at most +0.233 (Glamorgan) | the same (Cheshire's lowest markup 0.98147) |
| long run: shortfall ticks 14,502 in six counties (Surrey 5,593, Armagh 3,619, Caernarfonshire 2,320, Sussex 1,479, Down 1,115, Tyrone 376); 0 dead, idle, no-order and withheld ticks | the same, county for county |

**The dead-bar ties.**
- **Where.** 625 runs have dead ticks that differ. In 253 of them only the engine has dead ticks,
  and in 94 only the mirror does. The differences are +1 (388 runs), +2 (109), −1 (117) and −2 (11).
  All are in five Tier 3S runs: the good desk's coin × 0.5 (298), the fodder desk's coin × 0.5
  (147), the capacity desk's heads × 0.5 (103), fodder's stock × 0.5 (66) and the good's stock ×
  0.5 (11).
- **What each is.** One run each of the first three kinds was rerun for 600 ticks with every row
  written. In each, the extra dead tick is a tick where a market clears half its target to the last
  ulp:
  - the good desk's coin × 0.5, Aberdeenshire 1750: the good clears 0.49999999999999994 of its
    target at tick 1, so that tick is dead;
  - heads × 0.5: the horse-days clear 0.4999999999999999 at tick 1, then run below half for 34 more
    ticks, as the mirror's do;
  - the fodder desk's coin × 0.5, Aberdeenshire 1875: fodder clears 0.49999999999999994 at tick 1.
- **Why.** A tick is dead below half, so the engine and the mirror round apart there, as P2.2b found
  (LOOPS.md).

**The kick sets' g.** Every base kick set decays, which is what E4 scores. Its fitted g,
0.856–0.876 a year, is faster than the mirror's slowest local mode, 0.920–0.923 a year (the horse's
wear, 1 − δ). The fit is P2.2a's reading (decision 293): from the envelope's peak down to ten times
its rounding floor. At Aberdeenshire 1750 a 1e-9 kick meets that floor, 3.3e-5 of the kick, about
4,000 ticks (77 years) after it. That happens while the faster modes still lead, at 0.82–0.84 a
year. So the engine's g reads the kick's early decay, not the slowest mode. [fig4](../figs/fig4_kicks.png).

## The long run: no dead, idle or withheld tick, and far from equilibrium

**The run.** [long-run.csv](long-run.csv) sets the engine beside the mirror by county;
[long-run-engine.csv](long-run-engine.csv) is the engine's full table.
- Every one of the 33,332 events fires, and the ledger closes every tick (largest margin 5.9e-4 of
  its tolerance).
- The run is the pinned run: final hash `0x45b7c1201f8ae633`, stream `0xacb3ca2bee63b3bf`.
- At the end every county's params are its plan's.
- 32.6 s on WSL, scoring included.

**What holds, in every county.**
- No market goes dead. Labour, land, fodder, horse-days and the good clear at least 0.606 of their
  oracle volume (Glamorgan's horse-days); then Lanarkshire 0.614 and Durham 0.623.
- The horse market never idles and always has an order. The maker never withholds: its markup is at
  least 0.981, far above the reservation's floor of ψ 0.25.

**Where it is thin.** Six counties have ticks in which the provider cannot pay the whole transfer:
Surrey 5,593, Armagh 3,619, Caernarfonshire 2,320, Sussex 1,479, Down 1,115 and Tyrone 376. These
are the thin-funded counties of WORLD-V2 §4, as registered (O82).

**The gap to each county's moving equilibrium.**
- The median county's median D̂ is 116.7, that is 0.117 in log.
- The coal and cotton counties run furthest: Lanarkshire 382.7, Lancashire 376.9, Glamorgan 361.9,
  Durham 330.8.
- The largest D̂ at any tick is 543.5 (Glamorgan).
- By the first tick of 1901 the median county is at 60.4 and Glamorgan at 362.4.
- Peeblesshire runs closest, with a median of 38.2.
- Against v1's run (county medians' median 10.8, WORLD.md §8) the second pass runs about 11 times
  further from its equilibrium, and its capital settles on the history's own time scale: in the
  battery a shocked county-date takes a median of 58 years to come within tolerance, and up to 109.

[fig2](../figs/fig2_long_run_dhat.png).

## Capital's lag

[capital-lag.csv](capital-lag.csv) and [fig3](../figs/fig3_long_run_heads.png). The herd is the
installed heads over their equilibrium at the params in force, sampled every 13th tick. The engine's
series equals the mirror's within 2.0e-14 in every county.
- **Coal and textile** (26 counties). The herd falls lowest. Sampled every 13th tick: 0.718 in
  Glamorgan in 1861, 0.726 in Lanarkshire in 1851, 0.729 in Durham in 1864, 0.744 in Lancashire in
  1851. Read every tick: Glamorgan 0.717 (1863), Lanarkshire 0.725 (1850), Durham 0.728 (1863),
  Renfrewshire 0.743 (1779), Lancashire 0.743 (1851). The median county of
  the class is below 0.9 for 69 years. 20 of the 26 are still more than 5% short in 1901 (median
  herd 0.91).
- **London's ring** (7). Down to 0.764 (Middlesex, 1827). The median county is below 0.9 for 62
  years, and 5 of the 7 are more than 5% short in 1901.
- **Highland** (5). Never below 0.9, and all are back within 5% by 1901 (median 1.015).
- **Rural** (55). The median county is below 0.9 for 12 years. 11 are more than 5% short in 1901,
  and the class median is 1.001.

The shape follows the history's big ramps. Lancashire's herd:
- falls to 0.76 of its equilibrium by the late 1770s, in the history's first ramps;
- recovers to 0.86 by 1800;
- falls again, to 0.74 in 1851, through the textile, steam and railway ramps;
- climbs to 0.88 by the mid-1880s, once the ramps slow;
- stands at 0.84 in 1901.

Glamorgan and Durham fall until the 1860s. Surrey stays near 0.78 from the 1830s to the 1870s.
Sutherland's herd, where the ramps are small, stays within 0.93–1.03. This is D-G14's departure
over a history (O81): a capacity desk's target grows only with the scarcity rent it keeps as coin,
at most +23% over full cost (Glamorgan). The agents lag a moving equilibrium; they do not fail to
find a fixed one, as the battery shows.

## How it ran

- **The build.** `8e8781f`, built on WSL in release (`D:/rustyecon-d2/runs/BIN.sha256`):
  - `horses` sha256 `3def0cf273305aa5baf9cd2d6af592c87f878c5675eacfd7ceac889b4406bf7c`, the binary
    of the pre-wave checks;
  - the `demo_v2` test binary sha256
    `36d449fa230aaf9316e4dea12b295c9bb73d98b65a273b2a31cf0f1cfe71adc1`.
- **The jobs** ([tools/jobs.txt](tools/jobs.txt), sha256 `82d68a8e…73e1`, from
  [tools/make_jobs.py](tools/make_jobs.py)) number 1,675:
  - the long run;
  - 558 batteries (`horses family battery` per county-date at its L);
  - 558 kick sets (`horses slowest hold --ticks L --horizon L`);
  - 558 mode A runs.
- **The wave.** 46 jobs at a time on WSL's 48 threads ([tools/wave.sh](tools/wave.sh)), from
  07:28:03Z to 10:40:25Z, 3 hours 12 minutes:
  - for its first hour and a half another branch's wave shared the machine (load 60–92);
  - a battery job took 535–1,490 s (median 840);
  - every job exited 0 with an empty stderr.
- **Gathered and scored** by [tools/gather.py](tools/gather.py) and [tools/score.py](tools/score.py),
  as committed. Before the wave, the scorer's self-test ([tools/selftest.py](tools/selftest.py))
  wrote the mirror's own outputs as the gathered tables. The scorer passed all 21 lines on them and
  reproduced the registration's aggregates.
- **The tables here** come from [tools/tables.sh](tools/tables.sh), [tools/engine_runs.py](tools/engine_runs.py),
  [tools/report.py](tools/report.py), [tools/lag.py](tools/lag.py) and
  [tools/plots.py](tools/plots.py). They were written after the wave and regroup its output; none
  of them scores.
- **The raw runs** are in `D:/rustyecon-d2/runs/`:
  - `wave.tar.gz`: every job's summary.tsv, stats.tsv, each run's first and last rows, the kick
    envelopes, the long run's tables and every exit file;
  - `gathered-scored.tar.gz`;
  - `pre/`: the elasticity probe and mode A before the wave.
- **The kick envelopes' names.** The wave wrote them as `envelope-demo:county.gla@1850-hold.tsv`, on
  WSL. On Windows a colon in a file name opens an alternate data stream, so every county-date's
  envelope would go into one file. From D2.5 the harness writes
  `envelope-demo_county.gla_1850-hold.tsv` (`probe::horses::report::envelope_name`); the archive
  keeps the wave's names, and no tool here reads them. The wave's jobs ran on WSL only.

## Files

| file | what |
|---|---|
| [lines.csv](lines.csv) | every scored line: registered, engine, tolerance, verdict |
| [county-dates.csv](county-dates.csv) | each county-date: L, runs, CONVERGED, class differences, runs beyond 10% and 25%, dead and basket misses, kick set and g, mode A, E3, E4, GO |
| [battery-runs.csv](battery-runs.csv) | the engine's battery run by run, in the registered `battery-runs.csv`'s rows and columns, so the two diff line by line |
| [kicks.csv](kicks.csv), [modea.csv](modea.csv) | each county-date's base kick set and mode A |
| [long-run.csv](long-run.csv) | the long run by county, engine beside mirror, with E5's verdict |
| [long-run-engine.csv](long-run-engine.csv) | the engine's long-run table as `demo_v2_runs_to_1901` writes it (42 columns) |
| [aggregates.csv](aggregates.csv) | registration §3.3's and §3.4's aggregates, both sides |
| [by-class.csv](by-class.csv) | the verdict and the long run by county class and nation |
| [capital-lag.csv](capital-lag.csv) | the herd against its equilibrium over the history, by county |
| [e0.tsv](e0.tsv), [e0-sensitivity.txt](e0-sensitivity.txt) | E0's trace diff, and the mirror's own spread (A1 §A1.3) |
| [../figs/](../figs/) | fig1 ticks to tolerance, fig2 the long run's D̂, fig3 the herd in six counties, fig4 the kick sets' g |
