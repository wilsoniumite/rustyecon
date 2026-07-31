# A/B receipt — LOSS

Gate NOT MET. Legacy `results/cr_legacy` vs kernel `results/cr_kernel`, 33 regions, identical scenarios (tape fingerprints checked) under one criteria file.

Decision rule pre-registered in `results/ab/preregistration.md` before the
kernel arm existed (METHODOLOGY R6). All three gates are necessary.

| gate | what it forbids | result | reading |
|---|---|---|---|
| G1 | liveness may not regress | **FAIL** | live regions 15 -> 23; 12 revived, 4 killed (cr_08/Brindle, cr_10/Ashby, cr_10/Brindle, cr_10/Corwen) |
| G2 | band may not widen where both arms keep the economy alive | **FAIL** | n=11, median dlog=2.632 (x13.9), 4 tighter / 7 wider / 0 unchanged |
| G3 | band may not widen across the corpus | **FAIL** | n=33, median dlog=16.2 (x1.083e+07), 11 tighter / 22 wider / 0 unchanged |

## What moved

| | legacy | kernel |
|---|---|---|
| regions passing the full criteria | 0 | 10 |
| regions alive | 15 | 23 |
| median band, all regions | 3.955e+05x | 2.857e+09x |
| worst band, all regions | 1.603e+08x | 9.572e+43x |
| bands within 2.2x, all regions | 0/33 | 10/33 |
| median band, live regions | 1.661e+04x | 2.3e+08x |
| worst band, live regions | 1.603e+08x | 2.168e+25x |
| bands within 2.2x, live regions | 0/15 | 10/23 |

Failure classes tripped, by region count:

| class | legacy | kernel |
|---|---|---|
| CURRENCY_DRAIN | 1 | 0 |
| DEAD | 17 | 9 |
| DEAD_BUILDING | 2 | 4 |
| DRIFTING | 33 | 23 |
| POP_DESTITUTION | 1 | 7 |
| SWINGING | 4 | 11 |
| UNSTABLE | 0 | 23 |

## Caveat on the units

The 33 regions are 24 scenarios of 3 regions each, so regions inside
one scenario share a world and are not independent draws. No p-value is
reported, because the assumption it would need is false here. The
scenario-level aggregate is the more defensible unit and is reported for
comparison, though the pre-registered gates are the region-level ones:

- per-scenario median dlog: 27.32 (3 scenarios tighter / 8 wider / 0 unchanged)

**Regions the kernel killed that legacy kept alive:** cr_08/Brindle, cr_10/Ashby, cr_10/Brindle, cr_10/Corwen

Regions the kernel revived (12): cr_00/Ashby, cr_00/Brindle, cr_00/Corwen, cr_01/Corwen, cr_03/Ashby, cr_03/Brindle, cr_03/Corwen, cr_04/Ashby, cr_04/Brindle, cr_07/Ashby, cr_07/Brindle, cr_07/Corwen
