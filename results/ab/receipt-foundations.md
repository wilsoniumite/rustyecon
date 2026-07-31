# Receipt — the foundations matrix, 2026-07-31

**Headline: nothing improved.** Across the whole matrix — 2 agent arms × 2 price
rules × 3 supply rules, 261 certified runs — **not one of the 72 `lr` regions
passes its criteria in any cell, and not one is both alive and inside the 2.2×
band in any cell.** Every legacy → kernel comparison is a **LOSS** on the
pre-registered rule, under both price rules and all three supply rules. The
Phase 4 gate is met by no configuration measured here.

The matrix does produce four things that are new, and one of them is a defect in
the measuring instrument rather than in the engine. They are in §4–§7.

## 0. What was run

```
./target/release/rustyecon <scenario> --ticks 1000 --certify \
    --agents {legacy|kernel} --price-rule {imbalance|ratio} \
    [--supply-rule {inelastic|reservation_goods|reservation}] \
    --results tmp/matrix/<cell> --output tmp/matrix/_out/<cell>/<scenario>
```

29 scenarios per cell: the 24 `lr_*` search-grid cells and the 5 solvable worlds
(`solv_1g`, `solv_1g_money`, `solv_1g_money_2x`, `solv_chain`, `solv_labour`).
Nine cells — the eight scored ones plus a falsification control — driven by
`tmp/matrix/run_matrix.py` on a thread pool. Certificates are scratch and live
under `tmp/`; the committed artifact is this file and
`results/ab/receipt-foundations.json`, which carries every per-region band, every
per-scenario battery verdict and every pairwise comparison, so the numbers below
are auditable without re-running anything.

The solvable tapes are run **at genesis**, which is mode A — *does the engine
hold a fixed point it is started on*. Mode B (displace one price 2×) is applied
in-process by `tests/test_12_solvable.rs` and has no CLI, so it is **not** in
this matrix. Its readings stand as recorded in PLAN; nothing here amends them.

**The supply rule is a kernel-only dial, and that is checked rather than
assumed.** A ninth cell ran the *legacy* arm with `--supply-rule
reservation_goods`; its B3 state hash is identical to the unswitched legacy arm
on **24 of 24** `lr` scenarios. A dial that silently moved the incumbent would
have invalidated every comparison in this receipt.

## 1. The `lr` corpus — 24 scenarios × 3 regions = 72 regions

`band` is the pre-registered statistic: the region's worst `DRIFTING` /
`LevelRange`, `max` over metrics, `+∞` when missing or non-positive. `B8` is the
worst (region, good) `|ln(realised/implied)|` in each scenario, reported as a
factor; the column is the **median** and the **max** of that over the 24
scenarios. `B6`/`B7`/`B8` are per-scenario battery pass counts out of 24.

| cell | alive | passing | median band | worst band | ∞ bands | ≤2.2× | **alive AND ≤2.2×** | B8 median | B8 worst | B6 | B7 | B8 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| legacy + imbalance | **33** | 0 | 1.525e5× | 1.055e234× | 0 | 0 | **0** | 1.74e7× | 3.52e9× | **0/24** | **24/24** | 24/24 |
| legacy + ratio | 32 | 0 | 1.725e5× | 7.694e229× | 0 | 5 | **0** | 3.09e15× | 2.01e149× | **0/24** | **1/24** | 24/24 |
| kernel + imbalance + inelastic | 8 | 0 | 3.270e15× | 9.277e35× | 0 | 0 | **0** | 1.75e11× | 6.56e26× | **24/24** | **0/24** | 24/24 |
| kernel + imbalance + reservation_goods | 5 | 0 | **6.533e8×** | 2.729e16× | 0 | 0 | **0** | **1.52e8×** | 8.35e9× | 24/24 | 0/24 | 24/24 |
| kernel + imbalance + reservation | **0** | 0 | ∞ | 2.106e19× | 45 | 2 | **0** | 1.53e23× | 2.84e23× | 24/24 | 0/24 | 24/24 |
| kernel + ratio + inelastic | **0** | 0 | 1.105e12× | 8.282e24× | 17 | 0 | **0** | 6.56e8× | 2.34e15× | 24/24 | 0/24 | 24/24 |
| kernel + ratio + reservation_goods | 2 | 0 | 1.862e15× | 1.175e304× | 10 | 0 | **0** | 1.13e13× | 6.90e238× | 24/24 | 0/24 | **23/24** |
| kernel + ratio + reservation | **0** | 0 | ∞ | 1.103e308× | 66 | 0 | **0** | 241.7× | 8.38e133× | 24/24 | 0/24 | 24/24 |

The `passing` column is zero eight times out of eight. So is `alive AND ≤2.2×`.
The standing claim — **no region has ever been both alive and in-band** — now
holds across eight configurations rather than six, and the two cells that put
regions inside the band (`legacy+ratio`, 5; `kernel+imbalance+reservation`, 2)
put only dead ones there, as before.

**The best `B8` reading in the whole table belongs to the worst economy.**
`kernel+ratio+reservation` has a median price gap of **241×** — four to twenty
orders of magnitude better than every other kernel cell — with every one of its
72 regions `DEAD`, `DRIFTING`, `SWINGING` and `UNSTABLE`, and 66 of 72 bands
unbounded. B8 carries no liveness term by design (it scores markets that posted
something, not markets that cleared), so a corpse prices well. That is the
`solv_labour` lesson arriving on the `lr` corpus: **B8 is a distance, never a
verdict.**

Failure classes tripped, by region count:

| class | leg+imb | leg+rat | ker+imb+inel | ker+imb+resg | ker+imb+res | ker+rat+inel | ker+rat+resg | ker+rat+res |
|---|---|---|---|---|---|---|---|---|
| DRIFTING | 72 | 67 | 72 | 72 | 70 | 72 | 72 | 72 |
| UNSTABLE | 39 | 38 | 72 | 72 | 72 | 72 | 72 | 72 |
| SWINGING | 37 | 9 | 51 | 65 | 68 | 57 | 63 | 72 |
| DEAD | 30 | 38 | 28 | **0** | 72 | 72 | 70 | 72 |
| DEAD_BUILDING | 27 | 38 | **0** | **0** | **0** | **0** | **0** | **0** |
| POP_DESTITUTION | 9 | 5 | 60 | 67 | 72 | 10 | 15 | 2 |
| CURRENCY_DRAIN | 3 | 1 | 9 | 28 | 2 | 0 | 2 | 0 |

`DEAD_BUILDING` is 0 in all six kernel cells and 27–38 in both legacy cells —
P4.4's overflow routing, reproduced under the price rule it was not measured
under. It is the only class the kernel dominates on, and it buys `POP_DESTITUTION`
and `UNSTABLE` with it.

## 2. The solvable worlds — 5 scenarios × 1 region, mode A

| cell | alive | passing | median band | worst band | full-PASS certificates | B8 median | B8 worst | B6 | B7 |
|---|---|---|---|---|---|---|---|---|---|
| legacy + imbalance | 5 | 0 | 3.591× | 5.881e6× | **0/5** | 1.362× | 3.12e5× | 0/5 | 4/5 |
| legacy + ratio | 1 | 0 | 2.899e14× | 3.482e14× | **0/5** | 3.49e19× | 1.21e28× | 1/5 | 4/5 |
| kernel + imbalance + inelastic | 5 | **5** | **1.000×** | **1.000×** | **5/5** | **1.000×** | 2.000× | 5/5 | 5/5 |
| kernel + imbalance + reservation_goods | 5 | 4 | 1.000× | 4.140× | 4/5 | 1.000× | 1.729× | 5/5 | 5/5 |
| kernel + imbalance + reservation | 4 | 4 | 1.000× | 104× | 4/5 | 1.000× | 17.5× | 5/5 | 5/5 |
| kernel + ratio + inelastic | 5 | **5** | **1.000×** | **1.000×** | **5/5** | **1.000×** | 2.000× | 5/5 | 5/5 |
| kernel + ratio + reservation_goods | 4 | 4 | 1.000× | 1.582e16× | 4/5 | 1.000× | 779.6× | 5/5 | 4/5 |
| kernel + ratio + reservation | 4 | 4 | 1.000× | 1.805e114× | 4/5 | 1.000× | 1.45e229× | 5/5 | 4/5 |

**The one unambiguous positive result in this matrix, and it is not on the
corpus.** The kernel with the shipped supply rule holds all five hand-computed
equilibria under **both** price rules: band `1.000×`, B8 `1.000×`, five full-PASS
certificates each. The `ratio` rule holding mode A is new here — it is a harness
check as much as a result, since `d == s` is stationary under both rules by
construction, and it confirms that the two rules are not accidentally different
at rest. The legacy arm produces **zero** full-PASS certificates on either rule.

The single scenario that separates the supply rules is `solv_labour`, in the
direction already registered before the run: its equilibrium markup is `R = 2`,
so `R = 1` is the wrong reservation price and the fixed point stops being one.
Under `ratio` the same defect is far worse (`779×` and `1.45e229×` against
`1.73×` under `imbalance`), which is the damping term doing its job in one column
and not existing in the other.

## 3. The A/B verdicts

`tools/ab.py` applied unchanged, against the rule pre-registered in
`results/ab/preregistration.md`. **No amendment was made and none is proposed
here.** All 14 comparisons were computed twice — once by importing `ab.compare`
into the summariser and once by invoking `tools/ab.py` from the command line —
and the verdict and all three gate strings agree on 14 of 14. That check exists
because a receipt that reimplemented the rule would be a receipt for a different
rule. The corpus is the `lr` set only (`--only lr_`), as registered.

### 3a. The pre-registered comparison: legacy → kernel

| price rule | supply rule | verdict | G1 liveness | G2 both-live band | G3 corpus band |
|---|---|---|---|---|---|
| imbalance | inelastic | **LOSS** | FAIL 33 → 8, 26 killed | FAIL n=7, +11.48 | FAIL n=72, +22.59 |
| imbalance | reservation_goods | **LOSS** | FAIL 33 → 5, 30 killed | pass n=3, −1.351 | FAIL n=72, +12.84 |
| imbalance | reservation | **LOSS** | FAIL 33 → 0, 33 killed | pass n=0 (empty) | FAIL n=72, +∞ |
| ratio | inelastic | **LOSS** | FAIL 32 → 0, 32 killed | pass n=0 (empty) | FAIL n=72, +9.232 |
| ratio | reservation_goods | **LOSS** | FAIL 32 → 2, 31 killed | FAIL n=1, +7.543 | FAIL n=72, +13.57 |
| ratio | reservation | **LOSS** | FAIL 32 → 0, 32 killed | pass n=0 (empty) | FAIL n=72, +∞ |

Six for six. The price rule does not rescue the kernel and neither supply rule
does; the closest cell to a pass is still 33 → 5 on liveness.

### 3b. Within the kernel: the supply rule

| price rule | comparison | verdict | G1 | G2 | G3 |
|---|---|---|---|---|---|
| imbalance | inelastic → reservation_goods | **LOSS** | FAIL 8 → 5 | FAIL n=1, +2.501 | pass −3.02 (**20× tighter**) |
| imbalance | inelastic → reservation | **LOSS** | FAIL 8 → 0 | pass n=0 | FAIL +∞ |
| ratio | inelastic → reservation_goods | **WIN** | pass 0 → **2** | pass n=0 | pass median **0** |
| ratio | inelastic → reservation | **LOSS** | pass 0 → 0 | pass n=0 | FAIL +∞ |

That `WIN` is not good news; see §4.

### 3c. Holding the agent layer fixed: the price rule

Reported because the matrix makes it available, and flagged: the registration
names the *agent* comparison. Applying the same paired rule to a price-rule
change is a legitimate use of the statistic but **is not the Phase 4 gate**, and
no verdict below should be read as one.

| arm | comparison | verdict | G1 | G2 | G3 |
|---|---|---|---|---|---|
| legacy | imbalance → ratio | **LOSS** | FAIL 33 → 32, **23 revived, 24 killed** | FAIL n=9, +5.661 | pass −0.132 |
| kernel + inelastic | imbalance → ratio | **LOSS** | FAIL 8 → 0 | pass n=0 | pass −2.581 |
| kernel + reservation_goods | imbalance → ratio | **LOSS** | FAIL 5 → 2 | pass n=0 | FAIL +5.565 |
| kernel + reservation | imbalance → ratio | **TIE** | pass 0 → 0 | pass n=0 | pass median 0 |

The legacy row is the informative one. The `ratio` rule does not improve the
corpus, it **reshuffles** it: 23 regions revived and 24 killed, a near-perfect
exchange, with the corpus band essentially unchanged (−0.13 in log terms, 36
tighter / 36 wider) and the scenario-level median moving the *other* way (+2.44,
11 scenarios tighter / 13 wider). Two summaries of one comparison disagree in
sign, and the region-level and scenario-level units disagree too.

## 4. FINDING — the registered rule no longer measures the right thing, and the failure is now realised rather than hypothetical

**This is reported as a finding. The rule is not amended, and amending it is not
proposed as a fix.** Any change needs a dated file written before the number is
known (preregistration.md, "Changing this").

The pre-registration argues G1 into existence with a specific hole: *"A dead
economy has flat metrics and therefore a perfect band. Without G1 the rule ranks
collapse above every real improvement."* `tools/test_ab.py` demonstrates it by
collapsing every economy **against a live baseline**. Two cells in this matrix
are the case that test does not cover — **collapse on both sides** — and there
G1 and G2 go vacuous together:

**(a) A `WIN` between two arms that are both entirely broken.**
`kernel+ratio+inelastic → kernel+ratio+reservation_goods` reports **WIN**, gate
met. What actually moved:

| | baseline | challenger |
|---|---|---|
| regions passing criteria | 0 | 0 |
| regions alive | 0 | **2** |
| regions alive and in-band | 0 | 0 |
| median band (marginal) | 1.105e12× | **1.862e15×** — 1,685× **wider** |
| unbounded bands | 17 | 10 |
| paired median dlog, all 72 (G3) | — | **exactly 0** (29 tighter / 34 wider / 9 unchanged) |
| per-scenario median dlog | — | **+0.581** (7 tighter / **13 wider**) |

The verdict's entire strictness comes from `lr_02/Manchester` and
`lr_22/Manchester` going dead → alive. G2 is vacuous (no region is alive under
both). G3's median lands **exactly** on zero because nine paired regions are
unbounded on both sides and contribute `dlog = 0` — and those nine sit on the
median. So the three available summaries of this comparison read *better*
(paired median 0, WIN), *worse* (marginal median 1,685× wider) and *worse again*
(scenario-level +0.58). The rule reports the first.

**(b) A `TIE` between two arms in which every region is dead.**
`kernel+imbalance+reservation → kernel+ratio+reservation` reports **TIE**, gate
met. Both arms: 0 alive, 0 passing, 72/72 `DEAD` and `UNSTABLE`. The challenger
has **21 more** regions with an unbounded band (45 → 66) and 8 scenarios wider
against 1 tighter. G1 passes on `0 ≥ 0` with nothing killed; G2 has no regions;
G3's median is 0 because 42 of 72 pairs are unbounded on both sides.

**The precise defect.** G1 forbids liveness *regressing*; it says nothing about
liveness being *zero*. G2 is conditioned on regions alive under both arms and
evaporates when that set is empty. G3 is a paired median over a distribution in
which unbounded-vs-unbounded is scored as a tie — which is right when a handful
of regions blow up and wrong when most of them do, because the ties then decide
the median. **All three gates are relative; the rule has no absolute floor.**
Once an arm reaches zero live regions, the rule can be satisfied by any change
that revives one region, and cannot distinguish two corpses from two identical
healthy arms.

This did not bite while the baseline was legacy at 33 live regions. It bites now
because six of the eight cells in this matrix have ≤ 8 live regions and three
have zero. **What the rule needs is not a new threshold but a floor stated in
advance** — e.g. a comparison whose baseline has no live region yields
`INVALID`, the way a changed `tape_sha` does, rather than a verdict. That is a
change to the decision rule and therefore requires a dated amendment written
before it is scored. It is recorded here as the case for one; it is not made
here.

## 5. FINDING — the two arms violate exactly one invariant each, and neither satisfies both

Over the 24 `lr` scenarios, B6 (own-state posting) and B7 (no pinned market) are
mirror images:

| | B6 pass | B7 pass |
|---|---|---|
| legacy + imbalance | **0/24** | **24/24** |
| legacy + ratio | **0/24** | **1/24** |
| every kernel cell | **24/24** | **0/24** |

The legacy layer reads foreign volumes in every scenario and pins no market; the
kernel pins a market in every scenario and reads nothing foreign. There is no
cell in the matrix in which both hold. On the solvable worlds the picture is
different and better — every kernel cell is 5/5 on both under `imbalance` — which
localises the pin to the `lr` corpus's zero-supply markets rather than to the
kernel rule itself.

The `legacy + ratio` row is the sharper one: swapping the price rule takes B7
from 24/24 to **1/24** on an unchanged agent layer. `price_next_ratio` returns
the current price whenever `supply ≤ 0` or `demand ≤ 0`, so a one-sided market
freezes, its imbalance never moves, and B7 reads a pin. PLAN's "ratio freezes"
was traced from the code; this is it measured, on 23 of 24 scenarios.

## 6. FINDING — B8's fail-closed branch fired on the corpus for the first time, and what it caught is an f64 overflow

`kernel + ratio + reservation_goods` on `lr_05`:

```
B8 prices match technology: 1 market(s) with no computable distance:
    Manchester/flour no computable relative price in 851 traded tick(s)   FAIL
```

Traced rather than guessed. At tick 1000 that world's price vector spans
`p(Manchester/flour) = 3.32e306` down to `p(Leeds/flour) = 3.74e-15`, and
Manchester's wage is `0.006912`. B8's `realised = p_good / p_labour` is therefore
`4.8e308` — **it overflows f64** — so the reading is non-finite on every one of
851 ticks and the criterion refuses to report a distance rather than reporting a
wrong one. Until now this branch had only been shown firing on defects
constructed in `tests/test_11_correctness.rs`; this is the guard catching
something nobody planted.

Two things it also exposes about the instrument, stated because they affect how
every other B8 number in this receipt should be read:

1. **"851 traded ticks" means 851 ticks with something posted on at least one
   side**, not 851 ticks in which anything cleared. `lr_05/Manchester` is `DEAD`.
   The word is imprecise in the message.
2. `price_next_ratio` guards against a non-finite or non-positive *price*, so no
   single price can become infinite — but nothing guards the *ratio of two*
   prices, and the corpus reaches a spread wide enough to overflow it. A price
   rule that cannot produce an infinity can still produce a pair whose quotient
   is one.

## 7. Reproduction check, and one audit finding against PLAN

The four cells measured in earlier sessions reproduce **exactly** here: legacy +
imbalance at 33 live / 1.525e5×, kernel + imbalance + inelastic at 8 / 3.270e15×
with DEAD 28 / POP_DESTITUTION 60 / CURRENCY_DRAIN 9, reservation_goods at 5 /
6.533e8× with DEAD 0, reservation at 0 / ∞ with DEAD 72.

The `ratio` rows do **not** match PLAN's table in *"Can the price rule lose its α
entirely?"*. That table records:

| | PLAN | this matrix |
|---|---|---|
| legacy + imbalance, median band | 1.64e5× | 1.525e5× |
| legacy + ratio, median band | 2.94e5× | 1.725e5× |
| kernel + imbalance, median band | 3.32e15× | 3.270e15× |
| kernel + ratio, median band | 2.21e12× | 1.105e12× |

Checked rather than asserted: a worktree at `8ba33f9` — the commit that wrote
that table, with its own tapes and its own binary — was built and all four cells
re-run. It reproduces **1.525e5 / 1.725e5 / 3.270e15 / 1.105e12**, i.e. this
matrix's numbers, not its own table's. The live counts (33 / 32 / 8 / 0) and the
within-2.2× counts (0 / 5 / 0 / 0) in that table *are* correct; only the median
band column is not. The cause was not determined — `2.21e12 / 1.105e12` is exactly
2.000, which hints at a different aggregation over the same readings rather than
a different run, but that is a guess and is not asserted. **The median column of
that one table is superseded by this receipt**, and is marked in place in PLAN
per R14.

## 8. What this receipt does not say

- It does not say the kernel is worse than the legacy layer *as an agent design*.
  It says that on the 72-region search grid the kernel kills regions the legacy
  layer keeps alive, and that on the five worlds with a known answer the kernel
  is the only arm that gets it right. Both are measured; neither settles the
  other, and §4 says the instrument that is supposed to weigh them has stopped
  being able to.
- It does not re-open mode B. Nothing here displaces a price.
- It does not touch `tracer_2r`, `big_region`, `multi_region` or `supply_chain`;
  the matrix is the corpus the rule is registered against plus the solvable set.
- It does not propose a threshold, a corpus restriction, or a gate change. §4
  makes the case that the rule needs a floor; making that change is a separate,
  dated act.
