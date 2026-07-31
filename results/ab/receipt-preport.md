# A/B receipt — LOSS

Gate NOT MET. Legacy `tmp/ab/preport/legacy` vs kernel `tmp/ab/preport/kernel`, 72 regions, identical scenarios (tape fingerprints checked) under one criteria file.

Decision rule pre-registered in `results/ab/preregistration.md` before the
kernel arm existed (METHODOLOGY R6). All three gates are necessary.

| gate | what it forbids | result | reading |
|---|---|---|---|
| G1 | liveness may not regress | **FAIL** | live regions 29 -> 0; 0 revived, 29 killed (lr_01/Leeds, lr_02/Birmingham, lr_06/Birmingham, lr_08/Birmingham, lr_08/Leeds, lr_08/Manchester) |
| G2 | band may not widen where both arms keep the economy alive | PASS | n=0, median dlog=NaN (xnan), 0 tighter / 0 wider / 0 unchanged |
| G3 | band may not widen across the corpus | **FAIL** | n=72, median dlog=17.63 (x4.545e+07), 7 tighter / 65 wider / 0 unchanged |

## What moved

| | legacy | kernel |
|---|---|---|
| regions passing the full criteria | 0 | 0 |
| regions alive | 29 | 0 |
| median band, all regions | 1.296e+05x | 4.275e+09x |
| worst band, all regions | 7.756e+303x | 1.425e+27x |
| bands within 2.2x, all regions | 0/72 | 0/72 |
| median band, live regions | 13.34x | NaNx |
| worst band, live regions | 2.019e+07x | -x |
| bands within 2.2x, live regions | 0/29 | 0/0 |

Failure classes tripped, by region count:

| class | legacy | kernel |
|---|---|---|
| CURRENCY_DRAIN | 6 | 0 |
| DEAD | 33 | 4 |
| DEAD_BUILDING | 34 | 72 |
| DRIFTING | 72 | 72 |
| POP_DESTITUTION | 10 | 72 |
| SWINGING | 34 | 46 |
| UNSTABLE | 45 | 72 |

## Caveat on the units

The 72 regions are 24 scenarios of 3 regions each, so regions inside
one scenario share a world and are not independent draws. No p-value is
reported, because the assumption it would need is false here. The
scenario-level aggregate is the more defensible unit and is reported for
comparison, though the pre-registered gates are the region-level ones:

- per-scenario median dlog: 18.96 (0 scenarios tighter / 24 wider / 0 unchanged)

**Regions the kernel killed that legacy kept alive:** lr_01/Leeds, lr_02/Birmingham, lr_06/Birmingham, lr_08/Birmingham, lr_08/Leeds, lr_08/Manchester, lr_09/Leeds, lr_09/Manchester, lr_10/Birmingham, lr_10/Manchester, lr_11/Manchester, lr_12/Birmingham, lr_12/Leeds, lr_12/Manchester, lr_13/Birmingham, lr_13/Leeds, lr_13/Manchester, lr_14/Birmingham, lr_14/Leeds, lr_14/Manchester, lr_16/Birmingham, lr_16/Leeds, lr_16/Manchester, lr_18/Birmingham, lr_20/Birmingham, lr_20/Leeds, lr_21/Birmingham, lr_22/Birmingham, lr_23/Birmingham
