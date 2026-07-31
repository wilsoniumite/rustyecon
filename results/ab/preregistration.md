# A/B pre-registration — desk kernel vs legacy agents

**Registered 2026-07-31, before the kernel arm existed.** METHODOLOGY R6: a
series a run is scored against may never be paintable by dial choices, and a
criterion fixed after seeing results is not a criterion. The commit that adds
this file contains no kernel code; `tools/ab.py` implements it and
`tools/test_ab.py` shows it can report LOSS.

## Why the gate in PLAN.md needed replacing

Phase 4's gate reads "kernel ships if its pass-rate ≥ legacy's". Phase 3.5 left
the lr corpus at **0 of 72 regions passing**, so that gate is satisfied by a
kernel that also scores zero. It cannot distinguish an improvement from a
no-op, and a gate that cannot fail is not a gate.

The replacement is continuous. `DRIFTING` carries `LevelRange` — the ratio of a
metric's 95th to its 5th percentile over the scored window — which is both a
real number and exactly what the corpus fails on: it trips in **72 of 72
regions**. Certificates did not previously persist that number for regions the
`DEAD` short-circuit stopped scoring, which is why P4.0a came first.

## Definitions

Everything below is read from the persisted certificates of both arms. Nothing
is recomputed from telemetry, so the receipt is reproducible from what is
committed.

**Region.** A `(scenario, region)` pair. The corpus is 24 lr scenarios × 3
regions = 72. Both arms must produce the same 72, or the comparison is INVALID.

**Live.** No class in {`DEAD`, `DEAD_EMPLOYMENT`, `POP_DESTITUTION`} tripped.
These say the economy is *not running*; `UNSTABLE`, `DRIFTING`, `SWINGING` and
`CURRENCY_DRAIN` say it is running badly, which is a different claim and one a
region can survive. Liveness is read from `tripped`, not `counted`: `DEAD`
short-circuits, so a dead region never *counts* `POP_DESTITUTION`, and reading
`counted` would make liveness depend on scoring order instead of on the series.

**Band.** The region's worst `DRIFTING` reading: `max` over that class's metrics
of `LevelRange`. Max rather than mean — the verdict is decided by the worst
metric, and an average lets a well-behaved metric pay for a runaway one. A
missing, NaN or non-positive reading is `+∞`, not absent; the criteria are
fail-closed and so is this.

**Pairing.** Every comparison is paired by region, because the two arms run
identical scenarios. `dlog(r) = ln(band_kernel) − ln(band_legacy)`, with both
arms unbounded counting as 0 (they did not differ) and one arm unbounded as
±∞.

## The decision rule

Three gates, **all necessary**. Each exists to close a specific hole, named.

| gate | statement | the hole it closes |
|---|---|---|
| **G1** | `#live(kernel) ≥ #live(legacy)`, **and** no region goes live → dead | A dead economy has flat metrics and therefore a *perfect* band. Without G1 the rule ranks collapse above every real improvement. |
| **G2** | over regions live under **both** arms: `median dlog ≤ 0` **and** `#tighter ≥ #wider` | The improvement must not come entirely from dead regions while the working ones get worse. |
| **G3** | over **all 72** regions: `median dlog ≤ 0` | Reviving regions into unbounded drift is not an improvement. Paired over a fixed set, so it cannot be moved by composition. |

**Verdict.** `WIN` — all gates pass and at least one is strictly better. `TIE` —
all gates pass, nothing strictly changed. `LOSS` — any gate fails. `INVALID` —
the arms are not comparable. **The Phase 4 gate is met by WIN or TIE**, which is
what PLAN.md's "wins or ties" means; INVALID is not a pass.

**Comparability is checked, not assumed.** R10 requires identical scenarios. The
tool refuses to produce a verdict unless the two arms agree on the region set,
on every scenario's `tape_sha`, and on the criteria date, version and window.

**There is one registered constant in the whole rule:** `TIE_TOL = 1e-9`, the
log-space distance below which two readings are the same reading. Every other
threshold is either zero or a comparison between the arms. That is deliberate —
a rule with dials is a rule that can be turned until the answer comes out right.

## Reported, but not gating

The receipt also carries: regions passing the full criteria under each arm; the
band distribution (min / median / max / count within 2.2×) for all regions and
for live ones; per-class trip counts; every revived and every killed region by
name; and the per-scenario median `dlog`.

**No p-value is reported.** The 72 regions are 24 scenarios of 3, so regions
inside a scenario share a world and are not independent draws. A sign test on
them would be arithmetic wearing the costume of inference. The scenario-level
aggregate (24 units) is reported as the more defensible summary, but the
pre-registered gates are the region-level ones, fixed here in advance.

## The legacy baseline, as of registration

Measured from the certificates committed at `2b2ef77`, so a later re-baselining
cannot pass unnoticed:

| | value |
|---|---|
| regions | 72 |
| passing the full criteria | **0** |
| live | **29** |
| median band, all regions | 1.296e5 × |
| median band, live regions | **13.34 ×** |
| min / max band, live regions | 2.316 × / 2.019e7 × |
| bands within the 2.2× bar | 0 of 72 |
| classes tripped | DRIFTING 72, UNSTABLE 45, DEAD_BUILDING 34, SWINGING 34, DEAD 33, POP_DESTITUTION 10, CURRENCY_DRAIN 6 |

### Re-baselined 2026-07-31 — the genesis-scale port

The table above is what the corpus read before `tools/port_genesis_scale.py`
gave every building a genesis `chosen_size` of its own `recipe_size`. This
section exists because the paragraph above promised a later re-baselining could
not pass unnoticed, and one happened the same day. **The rule is unchanged**;
only the world it is measured on moved.

| | before the port | after |
|---|---|---|
| passing the full criteria | 0 | **0** |
| live | 29 | **33** |
| median band, all regions | 1.296e5 × | 1.525e5 × |
| median band, live regions | 13.34 × | **23.61 ×** |
| min band, live regions | 2.316 × | 2.715 × |

Both A/Bs are reported — `receipt-preport.md` and `receipt-ported.md` — rather
than one being asserted to be the fair comparison. The port's justification is
in the tool's header: it is one uniform rule, it fixes a tape field that was
unobservable under the legacy agent and load-bearing under the kernel, and it
cannot flatter a scored series, since nameplate is the value a kernel must
adjust *down* from if capacity is excessive.

## What the rule was shown to detect

`tools/test_ab.py`, run against the shipped certificates. A comparison that
cannot report LOSS is a rubber stamp, so each case perturbs one thing and
asserts the rule notices:

- an unchanged arm reports **TIE**, and the gate is met;
- **every economy collapsed** — every band a perfect 1.0 — reports **LOSS**, and
  the test asserts that G3 *was* fooled by it, so G1 is demonstrably
  load-bearing rather than decorative;
- every band doubled reports LOSS; every band halved reports WIN;
- reviving dead regions reports WIN;
- every band halved *at the cost of one living region* reports **LOSS**, and the
  receipt names the casualty;
- a changed `tape_sha` reports INVALID rather than a verdict;
- a certificate predating the measurement table is refused, not scored as if
  every band were missing.

## Changing this

Requires a new dated file beside this one, and the reason stated before the new
run (R6, R14). Specifically, the following are **not** available later: relaxing
G1's no-kill clause because it blocked an otherwise-good kernel; moving the band
class or metric set; switching `max` to `mean`; dropping a gate; or restricting
the corpus. Any of those may be the right change — but as a dated amendment
written before the number is known, never as an edit made after seeing it.
