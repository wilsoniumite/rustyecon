# The switch registration's amendment A1: E5's pooled end, read at full precision

Dated 2026-10-01. Step P2.4.18 on branch `phase2-s2`, label `run`. It amends how one line of the
switch scan's §7 E5 ([../../switch/SPEC.md](../../switch/SPEC.md), sha256 `ebf52cea…315f6f`) is
read, as registered at P2.4.7 ([registration.md](registration.md)). The registered files are
unedited (decision 282); the committed scorer and its outputs are unedited and stand as the
wave's record. This file's sha256 is in `registration-A1.sha256`.

**Disclosed: A1 was written after the wave was scored** (P2.4.17;
[README.md](README.md)), to explain lines that failed. It moves no class, tick, verdict or
registered number.

## What it changes

E5 as registered: "At the pooled targets each switch pop ends at its a\* within 1e-10 relative".
The refutation list: "a pooled pop's share more than 1e-9 relative from a\*".

The committed scorer (`../p24-wave/score_switch.py`) reads the end share from the harness's
`stats.tsv` (`switch.end`), which prints 7 significant digits (`%.6e`). Its last digit is worth
up to 1.2e-7 of a\*, a thousand times the band. So every scored pooled line failed, 148 of 148
(10 in the battery, 4 under Hold, 4 each at 12 and 365 a year, 102 at the 17 dial settings and
24 at the switch's four rates), and the refutation line printed "148 pooled pops more than 1e-9
relative from a\*". The scorer's README declared this resolution for E1's peak D̂ (reading 5)
and missed it for E5. The self-test could not see it: these lines need an engine run to L, so
they were "missing" there.

A1 reads the same quantity at the precision the harness writes it: the run's CSV's last row,
which is the run's last tick (tick L − 1, or 1.25·L − 1 after a dated shock), every double in
its shortest round-trip form (`pool_workers.trained`, `pool_workers.master`; `gather.py` keeps
this row as each record's `end`). The band stays 1e-10 relative to a\*, the 50-digit solve's
(`registered/switch_mp.json`), and the refutation stays 1e-9.

## What it does to the wave

`../p24-wave/amend.py` re-reads the lines (`amended.csv`, `../p24-wave/amend.out`):
- the 148 failed pooled lines pass. The largest gap is 8.4e-13 relative (IS1 at 365 a year,
  `tail.services=0.11@dated`, the trained, a\* 0.00703; at 52 a year at most 1.3e-13);
- IS2's 994 pooled lines are reported, as registered; every one is within the band;
- the refutation reads 0 pooled pops beyond 1e-9 relative;
- no line that passed fails.

The other 88 failed E5 lines, a walled pop's end above 1e-300, are not re-read. The scorer's
reading 4, declared before the wave, expected them: at 22,000 ticks a small gap (land.mach 0.8)
or a slower switch (`rate.*` or `rate.switch.*` × 0.75) leaves the share above the subnormal
stall. Every one is the trained, on its wall side with its gap below 0 (the refutation's
reading, 88 of 88): land.mach 0.8 at every setting (1.5e-252, as the registration's own full-L
run had it, 1.47e-252), and every target at `rate.*` and `rate.switch.*` × 0.75 (at most
2.2e-189). They stay failed lines.

After A1 the switch's wave has 88 failed lines, all of that one kind, and no refutation.
