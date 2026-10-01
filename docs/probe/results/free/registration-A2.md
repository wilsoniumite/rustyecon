# The free registration's amendment A2: three readings of the scored wave

Dated 2026-10-01. Step P2.4.18 on branch `phase2-s2`, label `run`. It amends how three kinds of
line of the free scan's §9.2–§9.3 and §10 ([../../free/SPEC.md](../../free/SPEC.md), sha256
`ef6c888f…3a04`) are read, as registered at P2.4.10 ([registration.md](registration.md)) with A1
(E0 only, P2.4.12, [registration-A1.md](registration-A1.md)). The registered files, A1, the
committed scorer and its outputs are unedited (decision 282) and stand as the wave's record. This
file's sha256 is in `registration-A2.sha256`.

**Disclosed: A2 was written after the wave was scored** (P2.4.17; [README.md](README.md)), to
explain lines that failed. 27 of the free step's 42,231 lines failed. A2 re-reads 23 of them, of
three kinds; the other 4 stand as failures (the end of this file). A2 moves no class, no tick to
tolerance, no verdict and no registered number. Its re-reading is `../p24-wave/amend.py`
(`amended.csv`, `../p24-wave/amend.out`).

## (a) The regime at the end, from the commons' posted price (8 lines)

§9.2 as registered: "The regime at the end is the oracle's in every CONVERGED run (… CT2:
`Commons` 734, `Crowded` 38, `Enclosed` 36, read from the commons' posted price, §6.4)". OF6
(O129): the readout "must come from its posted price … not from bids against offers, which differ
by rounding at rest (8 of the mirror's Crowded runs read `Commons` that way)".

The scorer compared the engine's regime with the mirror record's `regime_end`, which is the
bids-against-offers readout OF6 rules out. The 8 failed lines are OF6's 8 runs exactly: CT2's
`b.food=1.2` at genesis and dated, at buffer × 0.75 and × 0.9 and rate × 1.1 and × 1.25. In each
the mirror's commons ends priced (0.0218 of the reference) and its record reads `Commons`; the
engine reads `Crowded` from its posted price, which is the oracle's regime there.

A2 reads the line against the oracle's regime, `regime_star` in each registered record. On A2 the
8 pass, and the 2,203 regime lines that passed still pass.

## (b) The runaway tick on the harness's reference (3 lines)

§9.3 as registered: "runaway ticks of the trap within 5% against the harness's reference
(decision 399 as amended)". The registered ticks are not on that reference: the mirror's
`fb.run` holds the runaway bound to the undisplaced genesis prices (the oracle's point), as the
commons registration's runner did before its A3 (O107). The trap runs are joint4's, displaced by
up to a factor 4 a price, so the two references part by up to 20 ticks.

A2 replaces each registered tick by the registered mirror's tick on the harness's reference:
`../p24-wave/diag/runaway_ref.py` runs the mirror, copied from `D:/rustyecon-p24/scan-free/model`
with every file checked against the scan's `SHA256SUMS` and not edited, and wraps two of its
functions from outside so that the bound reads the displaced genesis prices
(`diag/runaway_ref.out`). On that reference the mirror gives the engine's tick in 8 of 8 runs,
to the tick: CT2's joint(4, s) for s 10, 12, 18, 21, 22, 34, 39 at 256, 279, 264, 263, 284, 279
and 279, and IL1's joint(4,10) at 269. The band stays 5%. On A2 the 3 failed lines pass and the
5 that passed still pass.

## (c) An end D̂ above 1e-9 at a slow root (12 lines, and one refutation line)

§10 as registered: "Any IL1 or CT2 target where the engine's live rest point differs from the
oracle's point beyond the tolerance". The scorer read it, as its README declared, as every
CONVERGED run's last D̂ at most 1e-9 (a gap of 1e-12 in log), and listed "12 CONVERGED runs
resting off the oracle's point".

The 12 are CT2's `b.food=1.2` (r_o 0.0118·r, OF4's slow mode) at genesis and dated, at rate ×
0.75 and × 0.9, buffer × 0.75, × 1.1 and × 1.25, and adjust × 0.75. Their end D̂ is 1.0e-9 to
6.9e-7. The registration's own largest root at these dials (`registered/lin_dial05.json`:
0.999884 at rate × 0.75, 0.999861 at × 0.9, 0.999846 at the others) carries each run from the
tick it entered tolerance to its end at L to 1.0e-9 to 6.9e-7, within 3% of the engine's. So by
the registration's own numbers these runs could not reach 1e-9 by L. They are still converging,
not resting off the point.

A2 reads "rests off the point" as a D̂ that has stopped falling: over the run's CSV's last 40
rows (56,400 ticks) D̂ falls on every row, at a root a tick within §9.3's PL band (2e-5 above
0.999) of the mirror's root at that target and dial. On A2 the 12 pass: every one falls on every
row, at a root within 2e-7 of the mirror's, and the refutation line reads none.

## What A2 does not re-read: 4 lines that stand

- **CT2's two kick sets and its verdict.** `b.food=1.2@dated` (tail gain 2.69e-3) and
  `exit.To=9.75@dated` (1.0) fail the bar of 1e-3 that §9.2 predicted they would pass, and with
  them CT2's verdict is LOCAL, not the registered GO. §10's "a kick set that fails" is met. The
  diagnostics of the README (run after the result, reported, not scored) bear on why, not on
  the verdict: at the Enclosed target the failing kicks are the commons' own price, a neutral
  direction of the continuum of rest points §4.4 names, gain 1.0 in the mirror too; at b.food × 2
  the kicked runs settle up to 2.7e-12 in log from the unkicked one and stay there (the same tail
  at H 70,500, 141,000 and 282,000), and the mirror, run the harness's way, settles the same way
  (tails 8e-5 to 1.1e-3, one above the bar). The registration predicted each kick set from the
  mirror's largest root, which sees neither.
- **One switch count.** CT2's dial run `JB(2)` at tilt 1: the commons switches between free and
  priced 8 times in the engine and 2 in the mirror (band 10% or 2). The paths part at tick 95,
  where pop `wb`'s coin (0.056) cannot cover both its baskets and its commons bid (0.048): the
  engine's budget chain caps the commons' budget at what the baskets leave (FREE-RULES, "the
  orders are the mirror's … while the coin covers it"), and the mirror lets the coin fall. 12 of
  CT2's 1,225 runs reach that corner; this is the only line among theirs that fails after A2
  (`diag/ct2_switches.out`, `diag/budget_short.out`).

After A2 the free step's wave has these 4 failed lines, and one refutation criterion of §10 met:
two kick sets fail.
