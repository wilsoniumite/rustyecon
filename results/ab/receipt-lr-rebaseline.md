# A/B receipt — LOSS

Gate NOT MET. Legacy `results/lr_legacy` vs kernel `results/lr_kernel`, 72 regions, identical scenarios (tape fingerprints checked) under one criteria file.

Decision rule pre-registered in `results/ab/preregistration.md` before the
kernel arm existed (METHODOLOGY R6). All three gates are necessary.

| gate | what it forbids | result | reading |
|---|---|---|---|
| G1 | liveness may not regress | **FAIL** | live regions 33 -> 8; 1 revived, 26 killed (lr_00/Leeds, lr_00/Manchester, lr_04/Leeds, lr_04/Manchester, lr_08/Birmingham, lr_08/Leeds) |
| G2 | band may not widen where both arms keep the economy alive | **FAIL** | n=7, median dlog=11.48 (x9.636e+04), 2 tighter / 5 wider / 0 unchanged |
| G3 | band may not widen across the corpus | **FAIL** | n=72, median dlog=22.59 (x6.47e+09), 10 tighter / 62 wider / 0 unchanged |

## What moved

| | legacy | kernel |
|---|---|---|
| regions passing the full criteria | 0 | 0 |
| regions alive | 33 | 8 |
| median band, all regions | 1.525e+05x | 3.27e+15x |
| worst band, all regions | 1.055e+234x | 9.277e+35x |
| bands within 2.2x, all regions | 0/72 | 0/72 |
| median band, live regions | 23.61x | 6.016e+06x |
| worst band, live regions | 6.08e+07x | 3.695e+16x |
| bands within 2.2x, live regions | 0/33 | 0/8 |

Failure classes tripped, by region count:

| class | legacy | kernel |
|---|---|---|
| CURRENCY_DRAIN | 3 | 9 |
| DEAD | 30 | 28 |
| DEAD_BUILDING | 27 | 0 |
| DRIFTING | 72 | 72 |
| POP_DESTITUTION | 9 | 60 |
| SWINGING | 37 | 51 |
| UNSTABLE | 39 | 72 |

## Caveat on the units

The 72 regions are 24 scenarios of 3 regions each, so regions inside
one scenario share a world and are not independent draws. No p-value is
reported, because the assumption it would need is false here. The
scenario-level aggregate is the more defensible unit and is reported for
comparison, though the pre-registered gates are the region-level ones:

- per-scenario median dlog: 23.24 (0 scenarios tighter / 24 wider / 0 unchanged)

**Regions the kernel killed that legacy kept alive:** lr_00/Leeds, lr_00/Manchester, lr_04/Leeds, lr_04/Manchester, lr_08/Birmingham, lr_08/Leeds, lr_08/Manchester, lr_10/Birmingham, lr_10/Leeds, lr_12/Manchester, lr_13/Leeds, lr_14/Manchester, lr_15/Birmingham, lr_15/Leeds, lr_15/Manchester, lr_16/Birmingham, lr_16/Leeds, lr_16/Manchester, lr_18/Leeds, lr_18/Manchester, lr_20/Leeds, lr_20/Manchester, lr_21/Birmingham, lr_22/Leeds, lr_22/Manchester, lr_23/Birmingham

Regions the kernel revived (1): lr_13/Birmingham
