# Demo v2, stage v2a.1: amendment A1 to the registration

Dated 2026-09-30. Step D2.4 (label `run`) on branch `demo-v2`, before any scored run. The
registration is [registration.md](registration.md) (sha256 `972c7d21…a5b0`), and it stays byte
for byte as registered. Decision 282 says a change after registration is a dated amendment with its
own sha256; this is that amendment. Its sha256 and the sha256 of its table are in
[SHA256SUMS-A1](SHA256SUMS-A1).

## A1.1 What changes: L at each county-date

WORLD-V2 §11.3 sets the battery's length at L = 84,000. That was F5's registered L (HORSES-RULES
§6.7), which is 200·τ_max for land's market on v1's base county under C2g. The same section says:
"Before any scored run the engine's elasticity probe runs at each county-date, and if its 200·τ_max
passes 84,000 anywhere, a dated amendment lengthens L there first."

The probe ran at all 558 county-dates (`horses elasticity`, on the build of D2.4's code, at
2026-09-30; raw output in `D:/rustyecon-d2/runs/pre/elasticity/`). Each time, land's market sets
τ_max. It ranges from 409.3 ticks (Norfolk 1800) to 471.4 (the West Riding 1850), with a median of
438.2. At 501 of the 558 county-dates, 200·τ_max rounded up to a thousand ticks is above 84,000.
The largest is 95,000, at the West Riding in 1850 and Dunbartonshire in 1900; the mean L is
88,115. Counties differ from v1's base county in how their land market answers its price, so their
land τ differs.

So **L at each county-date is the larger of 84,000 and the engine's 200·τ_max, rounded up to a
thousand ticks.** That is `probe::markets::probes::run_length`, the rule P2.2a and P2.2b used. The
table is [lengths.csv](lengths.csv): for each county and year, τ_max, the market that sets it, the
rule's L and the L used. By county-date, 57 run at 84,000. The other 501 run at 85,000 to 95,000:

| L | 84,000 | 85,000 | 86,000 | 87,000 | 88,000 | 89,000 | 90,000 | 91,000 | 92,000 | 93,000 | 94,000 | 95,000 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| county-dates | 57 | 45 | 61 | 74 | 69 | 86 | 63 | 42 | 30 | 23 | 6 | 2 |

What follows from L, as the registration defines each:
- a dated shock fires at L/4, and the scored clock starts there;
- the scored length is L;
- the kick set's horizon is H = L.

## A1.2 What does not change

- **The predictions.** The mirror ran at L 84,000. The runs predicted to converge settle within
  5,661 ticks, and a CONVERGED run stays in tolerance after that. So a longer run leaves each
  run's class, ticks to tolerance, dead, idle, withheld and shortfall ticks and troughs as they
  are. The one exception is the end gap, which can only shrink.
- **The dated shocks.** They fire from rest, after at least 21,000 ticks at the oracle point. Mode
  A holds there to 8.4e-14 at 84,000 ticks (A1.3). So the run after the shock is the one the
  mirror ran, to rounding.
- Every rule, tolerance, run list, dial and refutation criterion of registration §4.

## A1.3 What was run before this amendment, disclosed

These are the checks WORLD-V2 §11.5 puts before any scored run. None is a scored run (E3–E5).
- **The build.** D2.4's code: `horses --counties PATH --inst demo:<key>@<year>`; `rustyecon
  worldgen worlds/demo-gb --stage v2a1 --instances PATH`; and the tests
  `battery_instances_are_the_harness_instances`, `demo_county_table_reads_back_and_refuses_bad_rows`
  and `the_long_run_scorer_reads_the_rest_point`, which pass. The binary is `horses` sha256
  `3def0cf2…bf7c`, built on WSL in release from the working tree of D2.4's commit.
- **The county table.** It holds 558 rows, equal bit for bit to the mirror's `inst_at_month` on the
  registered `counties.json`. Checked with `D:/rustyecon-d2/run/tools/check_instances.py`: 0 values
  differ.
- **The elasticity probe** at all 558 county-dates (A1.1).
- **E2, mode A** (hold at L 84,000) at all 558 county-dates. It PASSES at all 558, and the largest
  gap in log is 8.4e-14 (Rutland 1825). The wave runs mode A again at each county-date's L.
- **E0, the trace diff** (`D:/rustyecon-d2/run/e0/tracediff.py`; output `tracediff.out`, sha256
  `a76fd2c9…a21a`). It covers five counties (Lancashire, Glamorgan, Surrey, Sutherland,
  Bedfordshire). Each runs hold and w × 2 at its 1850 instance, and b × 2 at genesis where b × 2 is
  funded, at both 1850 and 1750. That is 16 runs of 2,000 ticks each, every run starting from the
  engine's own genesis. Nothing parts at tick 1, so the run does not stop.
  - **Hold and w × 2** (10 runs) agree within 2.2e-13 on every observable.
  - **b × 2 at genesis** (6 runs, where the maker withholds for 31 to 33 ticks) parts, and on the
    same pattern each time:
    - the withheld ticks agree tick for tick;
    - the first parting above 1e-12 is the horse market's volume, at ticks 10 to 96, where the
      maker's offer is its finished stock less its cover band. At Lancashire 1750, tick 73, the
      offer is 1/24 of the finished stock and the gap is 3.3e-13. That is the inputs' gap (2.3e-14)
      times the cancellation, within decision 304's factor of 10;
    - through the flip-flop that follows, where the market clears the offer on one tick and the
      order on the next, the gap grows about 1.5 times a tick. It reaches 8.5e-2 in the horse
      market's volume and 7e-3 on the other observables (Lancashire 1750). Sutherland 1750
      reaches 7.9e-2 and 7e-3, Glamorgan 1750 1.7e-2 and 1.7e-3, Sutherland 1850 3.1e-3,
      Glamorgan 1850 8.9e-6 and Lancashire 1850 2.6e-8.
  - **Decision 304's second test fails here.** That test asks that the mirror with an ulp a tick
    part as far. It parts at most 6.2e-4 (Lancashire 1750). Read after the output
    (`sensitivity.py`, not a scoring rule): the mirror moved by up to 5e-13 relative before the
    flip-flop, about 25 ulps, parts from itself as far as the engine parts from it. It reaches
    6.5e-2, 3.9e-2, 2.7e-3, 7.3e-3, 1.1e-5 and 1.5e-8 at the same six runs. Moved by 2e-14, it
    reaches 1/10 to 1/40 of that.
  - So **E0 misses its 1e-9 line at these six runs.** No reading of E0 changes. The miss does not
    stop the run, because registration §4 stops the run only for a parting at tick 1. E3 scores
    the b × 2 runs as registered. The miss is reported in the results with its evidence.
- No battery, kick or long-run output of the stage has been read. The long-run test
  (`demo_v2_runs_to_1901`) has run only on the stage with its history removed (the rest-point
  test above).
