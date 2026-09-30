# The wall registration's amendment A1: E0's allowance for a rounding residue

Dated 2026-09-30. Step P2.3.3 on branch `phase2-proper`, after the build (P2.3.2) and before any
scored run. It amends the wall frame's §7 E0 ([../../wall/SPEC.md](../../wall/SPEC.md), sha256
`cc2b6d1c…9be0`) as registered at P2.3.1 ([registration.md](registration.md)). The registered
files are unedited (decision 282); this file's sha256 is in `registration-A1.sha256`.

## What it changes

E0 as registered: "Every price, share, coin, stock, S and D must agree within 1e-12 in log. The
one allowed parting is the budget chain's ulp (MARKETS-RULES §6.4)."

A1 adds one allowed parting. A market's S, D or cleared volume may part from the mirror in log
where it is a rounding residue: at most 1e-12 of that market's registered target volume, on a
tick where the mirror one ulp from itself (P2.1's check) also parts from the mirror by more than
1e-12 in log. Such a parting is reported in absolute terms and against its target volume. It
must leave every price, coin, share and stock, and every other S, D and volume, within 1e-12 in
log over the 2,000 ticks, as E0 asks. E0 is otherwise as registered.

## Why

The development trace diff on the uncommitted build (WALL-RULES §6.5, disclosed there) found
one parting of this kind, in `stock.mach*0.01` at tick 1:

- At tick 0 the machine desk holds 0.01 of its stock and keeps all of it (a·q ≥ held), so the
  category desks buy no machine services and make nothing. Their genesis lots nearly all sell.
- At tick 1 the services market holds only what is left of the genesis lot. In the engine that
  is held − sold, one ulp of 7.69, 8.9e-16; in the mirror it is held·(1 − fill), 8.5e-16. Both
  are the same zero in the frame's arithmetic.
- The services S and cleared volume there part by 3.9e-2 in log, 3.4e-17 absolute, 4.4e-18 of
  the target volume 7.69. The mirror one ulp from itself parts without bound on that tick.
- Nothing follows from it: over the 2,000 ticks every price, coin, share and stock agrees within
  4.7e-15 in log, and every D within 2.2e-15, in the development run.

It is the kind of parting the frame's E0 already allows for the budget chain: a measured
quantity that is a small difference, not the map. The frame did not name it, because the mirror
check `wm2_check.py` compares states, not a market's S at a tick where it is a residue.

## What it does not change

No prediction of the frame's §0–§7 moves: not the classes, the bands, E1–E9 or the refutation
criteria. The scorer's inputs (`docs/probe/wall/registered/`) are unchanged.
