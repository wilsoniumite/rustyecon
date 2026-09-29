# FUNDED-A1: an amendment to FUNDED (O41), after the mirror review

Dated 2026-09-29. Label `fix-report`, step L0.8 on branch `phase2-loops`. It amends
`D:/rustyecon-p2l/funded/FUNDED.md` (listed in that folder's `SHA256SUMS`), which stays as
written. This file has its own sha256 in `SHA256SUMS` beside it. The county, chain8, and every
number of FUNDED's tables and `instances.json` stand; what changes is one decision's reason, one
check's coverage, one range and the numbering.

## A1.1 Decision 240's reason, weighed over the run as well as at genesis

FUNDED chose chain8 (N 8, χ_max 1/20) over N 8 with χ_max 1/30 because χ_max 1/30 starts four
runs with every head at work, "a regime no earlier probe ran", while chain8 keeps z/χ_max ≤ 0.908
at genesis. The review found that chain8 reaches the bound too, later in the run. So the
criterion was re-run over the run, on the loop mirror with the genesis carry (`lm_carry.py`),
LB1's design and its 125 battery runs (Tiers 1–3, 3S and the stocks family), 20,000 ticks each
(`D:/rustyecon-p2l/fix-report/loop-carry/lmc_supply.py` and `.out`):

| county | runs at the bound at genesis | runs that reach it | ticks at the bound, all runs | the highest z/χ_max |
|---|---|---|---|---|
| chain8 (χ_max 1/20) | 0 | 5 | 649 | 2.05 (r × 0.5) |
| N 8, χ_max 1/30 | 4 | 9 | 1,189 | 3.46 (r × 0.5) |

chain8's five: r × 0.5 (262 ticks, peak 2.05), N(0.5) (238, 1.86), the workers' coin × 2 (82,
1.19), heads × 0.1 (45, 1.05) and JB(2) (22, 1.19). **Decision 240 stands with its reason
restated:** chain8 has more headroom than the alternative at genesis and over the run. It does
not keep labour supply off its bound: five of LB1's 125 runs reach it. LOOP-SPEC-A1 registers
the ticks at the bound per run, so P2.2b scores the regime it runs.

## A1.2 The good's price in the independent check (§7)

§7 says the oracle's doubles agree with the 50-digit solve within 9.5e-16 relative. The good's
price was not among the compared outputs. Compared now
(`D:/rustyecon-p2l/fix-report/funded/compare_good.py` and `.out`, 105 points): the point's
`p_good`, which `instances.json` carries as `prices.good`, is within 1.36e-14 relative (worst at
B-cut, δ 4%, b × 0.5); it is P_s − h, a difference, and the cancellation costs about 50 ulps. The
chain's own `goods.GOOD` price is within 3.7e-16. Every other output stands at 9.5e-16. All are
far inside the 1e-12 bars any use of them sets.

## A1.3 One range (§6)

§6's "1g's county at the same points runs from −2.08 to −2.24" should read −2.07 to −2.24 (the
provider's baskets over the 45 points, B, B-flow and B-cut at 5 targets and 3 δ;
`out/point_chain.jsonl`).

## A1.4 Numbering

FUNDED numbered its decisions from 240 and its open items from O51. STATE.md had taken 240–253
and O51–O57 by the time it was merged (L0.1–L0.6). They are renumbered there:

| FUNDED | STATE.md |
|---|---|
| 240 (chain8) | 254 |
| 241 (δ 8% the verdict δ) | 255 |
| 242 (the cost shock on every land coefficient) | 256 |
| 243 (the flow control in operating form) | 257 |
| 244 (the capacity plant's two bundles, left to the mirror) | 258, which LOOP-SPEC's L2 settles |
| 245 (rule B per unit made at every tick length) | 259 |
| O51 (space-heavy) | O58 |
| O52 (the corner below land × 0.35) | O59 |
