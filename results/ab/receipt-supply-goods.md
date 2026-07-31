# A/B receipt — LOSS

Gate NOT MET. Legacy `tmp/res_leg` vs kernel `tmp/res_kg`, 72 regions, identical scenarios (tape fingerprints checked) under one criteria file.

Decision rule pre-registered in `results/ab/preregistration.md` before the
kernel arm existed (METHODOLOGY R6). All three gates are necessary.

| gate | what it forbids | result | reading |
|---|---|---|---|
| G1 | liveness may not regress | **FAIL** | live regions 33 -> 5; 2 revived, 30 killed (lr_00/Leeds, lr_00/Manchester, lr_04/Leeds, lr_04/Manchester, lr_08/Birmingham, lr_08/Leeds) |
| G2 | band may not widen where both arms keep the economy alive | PASS | n=3, median dlog=-1.351 (x0.2591), 2 tighter / 1 wider / 0 unchanged |
| G3 | band may not widen across the corpus | **FAIL** | n=72, median dlog=12.84 (x3.771e+05), 14 tighter / 58 wider / 0 unchanged |

## What moved

| | legacy | kernel |
|---|---|---|
| regions passing the full criteria | 0 | 0 |
| regions alive | 33 | 5 |
| median band, all regions | 1.525e+05x | 6.533e+08x |
| worst band, all regions | 1.055e+234x | 2.729e+16x |
| bands within 2.2x, all regions | 0/72 | 0/72 |
| median band, live regions | 23.61x | 2.253e+05x |
| worst band, live regions | 6.08e+07x | 1.841e+06x |
| bands within 2.2x, live regions | 0/33 | 0/5 |

Failure classes tripped, by region count:

| class | legacy | kernel |
|---|---|---|
| CURRENCY_DRAIN | 3 | 28 |
| DEAD | 30 | 0 |
| DEAD_BUILDING | 27 | 0 |
| DRIFTING | 72 | 72 |
| POP_DESTITUTION | 9 | 67 |
| SWINGING | 37 | 65 |
| UNSTABLE | 39 | 72 |

## Caveat on the units

The 72 regions are 24 scenarios of 3 regions each, so regions inside
one scenario share a world and are not independent draws. No p-value is
reported, because the assumption it would need is false here. The
scenario-level aggregate is the more defensible unit and is reported for
comparison, though the pre-registered gates are the region-level ones:

- per-scenario median dlog: 10.58 (2 scenarios tighter / 22 wider / 0 unchanged)

**Regions the kernel killed that legacy kept alive:** lr_00/Leeds, lr_00/Manchester, lr_04/Leeds, lr_04/Manchester, lr_08/Birmingham, lr_08/Leeds, lr_08/Manchester, lr_09/Leeds, lr_10/Birmingham, lr_10/Leeds, lr_10/Manchester, lr_12/Birmingham, lr_12/Leeds, lr_12/Manchester, lr_13/Leeds, lr_14/Birmingham, lr_14/Manchester, lr_15/Leeds, lr_15/Manchester, lr_16/Birmingham, lr_16/Leeds, lr_16/Manchester, lr_18/Leeds, lr_18/Manchester, lr_20/Birmingham, lr_20/Leeds, lr_20/Manchester, lr_21/Birmingham, lr_22/Leeds, lr_22/Manchester

Regions the kernel revived (2): lr_18/Birmingham, lr_19/Birmingham
