# The wall registration's amendment A2: where a share stalls, at each tick length

Dated 2026-09-30. Step P2.3.16 on branch `phase2-proper`, label `fix-report`, after the scored
wave (P2.3.15) and its two reviews. It amends the wall frame's §7 E6 and one refutation criterion
([../../wall/SPEC.md](../../wall/SPEC.md), sha256 `cc2b6d1c…9be0`) as registered at P2.3.1
([registration.md](registration.md)), with A1 ([registration-A1.md](registration-A1.md)). The
registered files and A1 are unedited (decision 282); this file's sha256 is in
`registration-A2.sha256`.

**Disclosed: A2 was written after the wave was scored.** The committed scorer read E6's end line
as registered, and its printout (`../p23-wave/score.out`) reads: "the wall: IW1 GO ; refutations:
['CONVERGED runs ending at the wall (every share 0 or at most 5e-323, x = 1)']". The wave's
README read the four runs behind it as at the wall, after the result (O108, part 1). Both reviews
found that reading right; the measurement review asked for this amendment before any later wave
scores the line. A2 moves no class, no tick to tolerance and no verdict.

## What it changes

E6 as registered: "A displaced share decays to 5e-323, ten subnormal ulps, where a·s rounds to 0
and the update stops". The refutation criterion: "a converged run ending off the wall (a desk's
share not back to 0)".

A2 states the stall at each tick length. With a = share(adjust) = 1 − e^(−2.6/tpy), the update
s·(1 − a) leaves a share of k subnormal ulps unmoved once k·a < 1/2. So a displaced share stalls
at the largest such k:

| ticks a year | a | k (ulps) | the stalled share |
|---|---|---|---|
| 12 | 0.19480 | 2 | 1e-323 |
| 52 | 0.048771 | 10 | 4.94e-323 (the registered 5e-323) |
| 365 | 0.0070980 | 70 | 3.46e-322 |

A run ends at the wall when, on its last row, every share is 0.0 or at most its own tick length's
stall, every threshold x reads 1.0, and every labour market trades. The refutation criterion reads
"a desk's share not back to 0" as a share above that stall. Everything else in E6 stands,
including "a displaced share ends at 5e-323" in the battery and after 30,000 ticks, both at 52
ticks a year.

The measurement review simulated s·(1 − a), s − a·s and s·e^(−2.6/tpy) from 0.05: all three stop
at 2, 10 and 70 ulps. The fidelity review found the same, floor(0.5/a) ulps. The engine's shares
equal the mirror's to the bit (E0), so the mirror stalls there too.

## What it does to the wave

The scorer reads A2 with `--amended` (`../p23-wave/score.py`, P2.3.16; `../p23-wave/FIXES.md`).
Without the flag every line is as committed, `lines.csv` byte for byte (sha256 `ef87b114…`). With
it, E6's end line passes: 1,086 of 1,086 CONVERGED runs end at the wall. The four runs it moves are
`s[services]=0.05`, `s[services]=0.2`, `s[goods]=0.05` and `s[goods]=0.2` at 365 ticks a year,
each ending at 3.46e-322 with x = 1.0 and every labour market trading. No other line of the wall
moves, and the refutation list is empty.
