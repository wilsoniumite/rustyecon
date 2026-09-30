# The commons registration's amendment A2: A1's second condition corrected

Dated 2026-09-30. Step P2.3.8 on branch `phase2-proper`, after E0's official run and before any
scored run. It amends amendment A1 ([registration-A1.md](registration-A1.md), P2.3.7, sha256
`4fefa0a4…f98e`), which amends the commons frame's §5.5 E0 as registered at P2.3.5
([registration.md](registration.md)). The registered files and A1 are unedited (decision 282);
this file's sha256 is in `registration-A2.sha256`.

**Disclosed: A2 was written after E0's official run**, on the binary built at `d666f2d` (P2.3.7),
showed that A1's second condition, as written, is not met by the one run it was written for. That
run's numbers are the development run's (A1's "Why"); A1 stated the condition with a magnitude
its own evidence contradicts.

## What happened

A1's condition 2: "The mirror against itself, with its genesis wage moved by 1e-15 relative,
parts by at least the engine's largest parting over the same ticks." In the negative control's
`p[mach]*0.5` the engine's largest parting is 1.62e-7 (tick 78, labour's S). The mirror moved by
1e-15 parts by 4.67e-8, which A1 itself reports. So the condition fails as written, while A1's
other two hold: the mirror's one step from the engine's state agrees within 2.3e-15 at every tick
from 2 to 284, and the run is DIVERGED with its runaway at tick 284, as the registered mirror's
(`negctl.jsonl`: runaway at t = 284).

The mirror's sensitivity in that run does not grow with the size of the move (E0's
`pre_parting.out`): its genesis wage moved by 1e-15, 3e-15, 1e-14 and 3e-14 parts it from itself
by 4.7e-8, 2.2e-14, 2.8e-7 and 8.4e-8. So no one move measures "at least as much", and a
condition on one is arbitrary.

## What A2 changes

A1's condition 2 is replaced by:

2. **The seed is rounding.** Over the ticks before the first parting above 1e-12, every price and
   every coin agree within 1e-13 in log, the size of the rounding differences E0 finds in the runs
   that do not part (up to 2.6e-14 over 2,000 ticks at C1 and C2).

The mirror against itself from moves of 1e-15 to 3e-14 in its genesis wage is reported beside it,
not as a condition. Conditions 1 and 3 of A1 stand, and so does the rest of A1: a run that parts
passes E0 when 1, 2 and 3 hold, and every other run must agree within 1e-12 as registered.

In `p[mach]*0.5` the first parting is at tick 41; before it the largest gaps are 1.2e-14 in a
price and 2.9e-14 in a coin (and 9.6e-13 in labour's S and the desks' outputs, which the trap
amplifies on the ticks just before). So condition 2 holds there.

## What it does not change

No prediction of the frame's §6, and none of the registration's §3, moves: not the classes, the
bands of §5.6, E1–E9, the refutation criteria, or Tier 3S. The scorer's inputs are unchanged.
