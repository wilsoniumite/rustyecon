# The commons registration's amendment A3: the runaway bound's reference, and r_o at the end

Dated 2026-09-30. Step P2.3.16 on branch `phase2-proper`, label `fix-report`, after the scored
wave (P2.3.15) and its two reviews. It amends two bands of the commons frame's §5.6
([../../commons/SPEC.md](../../commons/SPEC.md), sha256 `60f21f56…b40d`) as registered at P2.3.5
([registration.md](registration.md)), with A1 and A2 ([registration-A1.md](registration-A1.md),
[registration-A2.md](registration-A2.md)). The registered files, A1 and A2 are unedited (decision
282); this file's sha256 is in `registration-A3.sha256`.

**Disclosed: A3 was written after the wave was scored.** 123 of the commons' 44,484 lines failed,
of these two kinds and no other (results README; O107, O108 part 2). Neither is a class, a tick to
tolerance or a verdict. A3 moves none of those. It fixes what the two bands are measured against,
for any later wave that reuses this registration, and it re-reads this wave's lines.

## 1. The runaway bound's reference (O107)

§5.6 as registered: "runaway ticks of the trap within 5%". The registered ticks are the mirror's
runner's, which holds PROBE-SPEC §4.5's bound (every posted price within [1e-6, 1e6] × genesis)
to the undisplaced point. The harness holds it to the run's own genesis prices, displaced. It has
done so since P2.1, and every runaway tick P2.1–P2.3 reported uses it.

A3 rules for the harness's reference. The registered tick of a trap run displaced at genesis is
replaced by the registered mirror's tick against the displaced genesis prices: `cm.py` and
`battery_c.displace`, unedited, run by `diag/runaway_ref.py` into `diag/runaway_ref.out` (195
runs: C1's joint2, joint4 and basin, C2's joint4 and basin, and C1N's five at genesis). The band
stays 5%. Runs displaced by a dated shock keep the registered tick, since there the two references
are the same prices. The three tilt-1 runs keep it too: `runaway_ref.py` did not run them, and
their engine ticks are within 5% of the registered ones as scored.

Why the harness's reference: the bound detects a price that has moved six orders of magnitude from
where its run started, and the harness prints its tick on that reading in every P2 wave. The two
references differ only when the price that leaves first is the displaced one: the run
`p[labour]*0.1227` crosses the bound held to its own start at tick 248, and the one held to the
point at 277. The alternative, the oracle's point in the harness, would move every runaway tick
P2.1–P2.3 reported. On the harness's reference the mirror gives the engine's tick in 190 of the
195 runs and one tick apart in the other 5, the trap's own amplification of rounding (A1).

## 2. r_o at the end (O108, part 2)

§5.6 as registered: "r_o at the end within 1e-9 relative (Crowded targets)". The scorer read it
against the mirror's `ro_end`, which the mirror takes where it stops early: once in 1e-6 of its
target for 2,000 ticks, from tick 4,000. At 52 ticks a year that value is within 1e-9 of the
oracle's. At 12 and 365 ticks a year the mirror stopped at ticks 4,281–7,868, up to 7.0e-8 from
the oracle's, while the engine runs to L.

A3 reads the band against the oracle's r_o/r (`ro_star` in the registered records), as SPEC §6.7
states it for the battery: "the shadow rent is within 1e-9 of the oracle's at a Crowded target".
The band stays 1e-9 relative.

## What it does to the wave

The scorer reads A3 with `--amended` (`../p23-wave/score.py`, P2.3.16; `../p23-wave/FIXES.md`).
Without the flag every line is as committed. With it:
- the 65 failed runaway lines pass, and the 130 others it re-reads still pass; the 7 it leaves
  alone are as scored;
- the 58 failed r_o lines pass, and so do the 929 others; the largest gap of the 987 from the
  oracle's is 2.95e-13 relative;
- no line that passed fails, and no class, tick to tolerance or verdict moves. The commons'
  refutation list stays empty.
