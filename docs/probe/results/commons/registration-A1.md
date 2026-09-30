# The commons registration's amendment A1: E0's reading of a run that amplifies rounding

Dated 2026-09-30. Step P2.3.7 on branch `phase2-proper`, after the build (P2.3.6) and before E0's
run and any scored run. It amends the commons frame's §5.5 E0
([../../commons/SPEC.md](../../commons/SPEC.md), sha256 `60f21f56…b40d` as registered) as
registered at P2.3.5 ([registration.md](registration.md)). The registered files are unedited
(decision 282); this file's sha256 is in `registration-A1.sha256`.

## What it changes

E0 as registered: "`cm.tick`, with the genesis carry, against the engine for 2,000 ticks: … the
negative control's `p[mach]*0.5` up to its runaway. They must agree within 1e-12 in log. A parting
blocks scoring until it is explained in the registration, as P2.1's budget-chain ulp was."

A1 explains a parting above 1e-12 in one run, and E0 passes on that run, when all three hold:

1. **The engine is the mirror's map there.** From the engine's own state at the start of every
   tick from 2 on (its prices, shares, coins and stocks, read from its CSV; no genesis carry is
   left by then), one tick of `cm.tick` gives the engine's next row within 1e-12 in log, in every
   quantity E0 compares (`e0/onestep_c.py`).
2. **The run amplifies rounding at least as much.** The mirror against itself, with its genesis
   wage moved by 1e-15 relative, parts by at least the engine's largest parting over the same
   ticks.
3. **The outcome is the mirror's.** The run's class, and its runaway tick where it runs away, are
   the mirror's.

Each such run is reported with its first parting tick, its largest parting, and the numbers of
1–3. Every other run of E0 must agree within 1e-12 in log, as registered.

## Why

The development trace diff on the uncommitted build (COMMONS-RULES §6.5, §7) found one run of
this kind, the negative control's `p[mach]*0.5`, which falls into the subsistence trap (SPEC §6.8)
and runs away at tick 284:

- As the workers' participation falls toward 0, the wage nears the exit's money value, and
  F = ln1p((w − e)/(P_s + e))/χ_max is a small difference of large numbers. The price differences
  of a few 1e-15 that the engine's and the mirror's orders of rounding leave (the budget chain's,
  MARKETS-RULES §6.4) are multiplied there, and the trap's inflation multiplies them further.
- The engine and the mirror part above 1e-12 from tick 41 (the desks' outputs), by 1.6e-7 at
  tick 78 (labour's S), and run away at the same tick, 284, as the registered mirror's run
  (`negctl.jsonl`: runaway at t = 284).
- The mirror's one step from the engine's state agrees within 2.3e-15 at every tick from 2 to 284.
- The mirror against itself, its genesis wage × (1 + 1e-15), parts from tick 42 (the desks'
  outputs), by 4.7e-8 at tick 78 (labour's S); with 1e-14, by 2.8e-7.
- Every run at C1 and C2 agrees within 2.6e-14, and their regimes, rented plots and shadow rents
  equal the mirror's at every tick.

The frame's own check of its mirror (`nest.py`) compares states one ulp apart; it could not see
this, since a coin moved one ulp parts the trap run by only 5.7e-12.

## What it does not change

No prediction of the frame's §6, and none of the registration's §3, moves: not the classes, the
bands of §5.6, E1–E9, the refutation criteria, or Tier 3S. The scorer's inputs
(`docs/probe/commons/registered/`, `tier3s/`) are unchanged.
