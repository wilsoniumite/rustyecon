# The free registration's amendment A1: E0's reading of a free price near 0

Dated 2026-09-30. Step P2.4.12 on branch `phase2-s2`, after the build (P2.4.11) and before E0's
run on the committed build and any scored run. It amends the free scan's §9.1 E0
([../../free/SPEC.md](../../free/SPEC.md), sha256 `ef6c888f…3a04` as registered) as registered at
P2.4.10 ([registration.md](registration.md)). The registered files are unedited (decision 282);
this file's sha256 is in `registration-A1.sha256`.

## What it changes

E0 as registered: "Every positive price and every other state value within 1e-12 in log of the
mirror's, and each free-able market's price exactly 0.0 on exactly the mirror's ticks. A parting
blocks scoring until a dated amendment explains it, as the frames' E0s did."

A1 explains a parting above 1e-12 in one run, and E0 passes on that run, when all hold:

1. **It is at a free price near 0.** The parting value is the free-able market's posted price, or
   a coin, on a tick where that posted price is positive and below 1e-3 of its step's scale
   c·p_ref.
2. **The engine's step is the mirror's.** On every tick of the run, the free step recomputed in
   the mirror's arithmetic from the engine's own posted price, its reference's posted price, c,
   and the engine's own S and D of that market gives the engine's next price within 4 ulps.
3. **The market's volumes are the mirror's to rounding.** On every tick of the run, one tick of
   the mirror (`fm.tick`) from the engine's own state (its prices, shares, coins and stocks, read
   from its CSV) gives that market's S and D within 1e-12 relative of the engine's.
4. **The free ticks are the trace's.** The market's price is 0.0 on exactly the trace's ticks, and
   every value outside item 1's scope agrees within 1e-12, as registered.

Each such run is reported with its partings and the numbers of 2–4 (`e0/tracediff_f.py`, which
reads A1). Every other run of E0 must agree within 1e-12 in log, as registered.

## Why

The development trace diff on the uncommitted build (FREE-RULES §6.5; `tracediff-dev1.out` and
`tracediff-dev2.out` in `D:/rustyecon-p24/build-free/e0/`) found one such parting in E0's eleven
runs, IL1's `JB(2)`, whose land market is priced for 30 ticks before it goes free:

- At tick 32 land's posted price is 6.0491316054274305e-5 in the engine and 6.049131605433814e-5 in
  the trace, 1.06e-12 apart in log, and the provider's coin after the tick, whose only income is
  that price times the land sold, parts by 1.05e-12. Land is free from tick 33 in both. Every other
  value of every run agrees within 6.1e-14, the free ticks exactly.
- The step is the mirror's: the free step recomputed from the engine's own p, p_ref, c, S and D
  gives the engine's next price bit for bit on all 2,000 ticks. What differs is land's demand D at
  tick 31, by 6.66e-15 relative (6.57e-14 of 9.86): the machine desk's two budgets, labour's and
  land's, sum to 1.2e-17 more than its outlay after rounding, so the budget chain cuts land's, the
  last in good order, by that ulp (MARKETS-SPEC §2.6), and at a land price of 1.85e-4 of the wage
  an ulp of the budget is 6.57e-14 of land (N6: a small price makes a budget's last ulp a
  quantity). The mirror's fills carry no budgets. From the engine's own state the mirror's D
  differs by the same 6.66e-15 on that tick, and by at most 5.7e-16 on every other of the 2,000.
- The free step amplifies it there. Its step is q = p·e^(kx) + c·p_ref·expm1(kx), and at tick 31
  q = 6.05e-5 is the difference of 1.85e-4 and 1.24e-4: d ln q/dx = k·(p + c·p_ref)·e^(kx)/q is
  150, so the imbalance's error of 6.57e-15 (4.8e-13 of x, since D − S cancels too) becomes 9.9e-13
  in one step, 1.06e-12 with the run's history, in the last positive price before land goes free.
  The mirror with one genesis price moved by an ulp parts from itself by 3.8e-14 at the same tick;
  the engine's one-step difference is a budget's ulp, not a genesis ulp.

The frame's own checks could not see this: the mirror has no budgets, and its nesting and rest
checks compare states at rest or one ulp apart.

## What it does not change

No prediction of the frame's §9, and none of the registration's §3, moves: not the classes, the
bands of §9.3, E1–E5, the refutation criteria, Tier 3S or the families. The scorer's inputs
(`docs/probe/free/registered/`) are unchanged.
