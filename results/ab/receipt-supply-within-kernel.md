# A/B receipt — LOSS

Gate NOT MET. Legacy `tmp/res_ki` vs kernel `tmp/res_kg`, 72 regions, identical scenarios (tape fingerprints checked) under one criteria file.

Decision rule pre-registered in `results/ab/preregistration.md` before the
kernel arm existed (METHODOLOGY R6). All three gates are necessary.

| gate | what it forbids | result | reading |
|---|---|---|---|
| G1 | liveness may not regress | **FAIL** | live regions 8 -> 5; 4 revived, 7 killed (lr_09/Leeds, lr_10/Manchester, lr_12/Birmingham, lr_12/Leeds, lr_13/Birmingham, lr_14/Birmingham) |
| G2 | band may not widen where both arms keep the economy alive | **FAIL** | n=1, median dlog=2.501 (x12.19), 0 tighter / 1 wider / 0 unchanged |
| G3 | band may not widen across the corpus | PASS | n=72, median dlog=-3.02 (x0.04881), 50 tighter / 22 wider / 0 unchanged |

## What moved

| | legacy | kernel |
|---|---|---|
| regions passing the full criteria | 0 | 0 |
| regions alive | 8 | 5 |
| median band, all regions | 3.27e+15x | 6.533e+08x |
| worst band, all regions | 9.277e+35x | 2.729e+16x |
| bands within 2.2x, all regions | 0/72 | 0/72 |
| median band, live regions | 6.016e+06x | 2.253e+05x |
| worst band, live regions | 3.695e+16x | 1.841e+06x |
| bands within 2.2x, live regions | 0/8 | 0/5 |

Failure classes tripped, by region count:

| class | legacy | kernel |
|---|---|---|
| CURRENCY_DRAIN | 9 | 28 |
| DEAD | 28 | 0 |
| DRIFTING | 72 | 72 |
| POP_DESTITUTION | 60 | 67 |
| SWINGING | 51 | 65 |
| UNSTABLE | 72 | 72 |

## Caveat on the units

The 72 regions are 24 scenarios of 3 regions each, so regions inside
one scenario share a world and are not independent draws. No p-value is
reported, because the assumption it would need is false here. The
scenario-level aggregate is the more defensible unit and is reported for
comparison, though the pre-registered gates are the region-level ones:

- per-scenario median dlog: -3.562 (20 scenarios tighter / 4 wider / 0 unchanged)

**Regions the kernel killed that legacy kept alive:** lr_09/Leeds, lr_10/Manchester, lr_12/Birmingham, lr_12/Leeds, lr_13/Birmingham, lr_14/Birmingham, lr_20/Birmingham

Regions the kernel revived (4): lr_15/Birmingham, lr_18/Birmingham, lr_19/Birmingham, lr_23/Birmingham
