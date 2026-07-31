# The `cr_*` consistency corpus — measurements, 2026-07-31

**This file is NOT an A/B receipt.** `receipt-cr-corpus.md` beside it is: it runs
the pre-registered rule (`preregistration.md`) on legacy vs kernel *within* the
`cr_*` corpus, under identical tapes and one criteria file, and it reports
**LOSS**. What follows is descriptive, and the distinction is load-bearing —
comparing `cr_*` against `lr_*` compares two different sets of worlds, so no
gate is applied to it and none should be.

Corpus: `data/scenarios/cr_00 … cr_10`, 11 scenarios × 3 regions.
Generator and auditor: `tools/gen_regions.py` (`--write`, `--audit`,
`--selftest`). Criteria: `lr_00/criteria.ron` copied byte for byte, generation 4,
2026-07-31, window 150..1000. Runs: 1,000 ticks, `--price-rule imbalance`,
`kernel.supply_rule: inelastic` as registered. `lr_*` was re-run under the same
build so the two corpora are read off the same engine.

---

## 1. What the audit says about the corpus that already existed

Five conditions, computed from the tape before a tick is run. A1 labour balance,
A2 goods balance, A3 money against Rule 3's bands, A4 genesis prices against the
technology's Leontief vector, A5 whether Rule 1 and Rule 3 can both be satisfied
by any stock level. Ratios; 1.000 is exact, and A5 needs > 1.

| tape | A1 labour | A2 goods | A3 money | A4 prices | A5 window |
|---|---|---|---|---|---|
| `lr_00`–`lr_07` | 2.15–2.42 | **inf** | 0.178–24.7 | 1.20 / 13.3 | 2 |
| `lr_08`–`lr_23` | **16.1–18.1** | **inf** | 0.178–24.7 | 1.20 / 13.3 | 2 |
| `multi_region` | 6.56 | inf | 0.178 | 1.20 | 2 |
| `big_region` | 1.00 | inf | 0.198 | **inf** (goods with no route to labour) | — |
| `supply_chain` | 1.00 | inf | 0.593 | **inf** | — |
| `tracer_2r` | 0.90 | 1.000 | 30.8 | 2.00 | — |
| `solv_1g` | 1.000 | 1.000 | 0.462 (one wealth tier, level unpinned by design) | 1.000 | — |
| `solv_1g_money` | **1.000** | **1.000** | **1.000** | **1.000** | — |
| `solv_1g_money_2x` | **1.000** | **1.000** | **1.000** | **1.000** | — |
| `solv_chain` | **1.000** | **1.000** | **1.000** | **1.000** | — |
| `solv_labour` | 1.000 | 1.000 | 1.13 | **2.00** | — |
| `cr_00`,`cr_02`,`cr_03`,`cr_09` | 1.000 | 1.000 | 1.000 | 1.000 | unconstrained |
| `cr_01` | 1.000 | 1.000 | 1.000 | 1.000 | **0.5 DEADLOCK** |

Two rows calibrate the instrument against worlds it did not build:

- The three `solv_*` tapes whose equilibria were **hand-derived before the engine
  was ever pointed at them** read exactly 1.000 on all four. Nothing was fitted
  to make that happen.
- `solv_labour` reads **A4 = 2.00**, which is *wrong* — and it is the registered
  counterexample to reading a Leontief benchmark as truth (its `recipe_size`
  binds and the firm earns a capacity rent). A4 reproduces B8's registered wrong
  answer to the digit, on the tape B8 is registered as being wrong on. Two
  independent implementations, same known blind spot.

`lr_*` fails four conditions out of five, on every one of its 24 tapes.
`A2 = inf` is a market with demand and **no supply at all**: the services desks
draw 50 labour per region to make a good the tier-0 basket does not contain.

---

## 2. The corpora, side by side

Descriptive. Each cell is over that corpus's regions (33 for `cr_*`, 72 for `lr_*`).

| | cr / kernel | cr / legacy | lr / kernel | lr / legacy |
|---|---|---|---|---|
| regions | 33 | 33 | 72 | 72 |
| **passing the full criteria** | **10** | 0 | 0 | 0 |
| alive | 23 | 15 | 8 | 33 |
| **alive AND band ≤ 2.2×** | **10** | 0 | **0** | **0** |
| median band | 2.86e9× | 3.96e5× | 3.27e15× | 1.52e5× |
| worst band | 9.57e43× | 1.60e8× | 9.28e35× | 1.05e234× |
| DEAD | 9 | 17 | 28 | 30 |
| POP_DESTITUTION | 7 | 1 | 60 | 9 |
| UNSTABLE | 23 | 0 | 72 | 39 |

**Ten regions are alive and in-band. Across every configuration ever run on
`lr_*`, that number has been zero, and the five regions that ever got inside the
band were all dead.**

All ten are in the consistent worlds. Not one is in a defect world.

---

## 3. The controlled comparison — the part that is not confounded

`cr_04`…`cr_08` differ from `cr_00` by exactly one named condition each, in the
same technology, the same basket, the same region sizes and the same criteria
file. `--selftest` asserts that the named condition moves and the other four do
not. So these differences are attributable in a way `cr_00`-vs-`lr_00` never is.

| tape | condition broken | reading | kernel worst band | live |
|---|---|---|---|---|
| `cr_00` | none | 1.000 ×4 | **1.000×** | 3/3 |
| `cr_04` | A1 labour (`participation` 0.45) | 2.222 | 2.17e25× | 2/3 |
| `cr_05` | A2 goods (chain halved, labour held) | 0.500 | 2.23e9× | 3/3 |
| `cr_06` | A3 money (all cash ×⅛) | 0.125 | 3.95e9× | 3/3 |
| `cr_07` | A4 prices (lr's own 0.35 / 0.60) | 1.400 | 2.38e20× | 3/3 |
| `cr_08` | all four | — | 1.35e23× | 2/3 |
| `cr_09` | none — probe, `s = 4` | 1.000 ×4 | **9.57e43×** | 0/3 |
| `cr_10` | none — probe, rentier pops | A2 0.956 | 3.41e27× | 0/3 |

Every named defect takes a world from a band of **1.000×** to between **2.2e9×**
and **2.2e25×**. The conditions are not decorative.

---

## 4. Findings that came out of building the worlds rather than running them

### 4.1 There is no basin of attraction. There is a dead band.

`cr_00` with its genesis flour price scaled by `x` and its cash re-solved, so
A1–A3 and A5 stay exact and only A4 moves:

| x | 1.00 | 1.01 | 1.02 | 1.05 | 1.10 | 1.20 | 1.50 | 2.00 |
|---|---|---|---|---|---|---|---|---|
| verdict | PASS | PASS | PASS | PASS | FAIL | FAIL | FAIL | FAIL |
| worst band | 1.000 | 1.000 | 1.000 | 1.000 | 3.4e9 | 1.0e29 | 6.3e9 | 5.9e26 |
| B8 | 1.000× | 1.010× | 1.020× | 1.050× | — | — | — | — |

**Inside the passing region B8 reads the displacement back verbatim with
`sd 0.000`: the wrong price never moves.** Those are full-PASS certificates —
all eight batteries, 3/3 regions — on a world that is permanently and exactly 5%
wrong. Nothing converges; the price freezes wherever it starts.

The edge was derived before it was measured. The mill's relative margin is
`(p_f − 0.5)/((p_f + 0.5)/2)`, which reaches `dead = 0.05` at
`p_f = 1.025/1.95`, i.e. **×1.051282**. Bisected: **×1.0512 PASS, ×1.0514
FAIL**. The "basin" is the dead band and nothing else.

Phase 4's RESULT 2 said the mechanism cannot walk to a fixed point one factor of
two away. This is sharper and worse: it cannot walk to one **6%** away, and
inside 5% it does not walk at all.

### 4.2 Rule 1 and Rule 3 contradict each other on every channel operator

Found by building `cr_01` — a trade world exact on A1–A4 — and watching its
channels stop trading on tick 2. A pass-through desk reads **one** inventory slot
with two opposite meanings:

    Rule 1 posts     stock > b_out · flow
    Rule 3 buys      stock < s     · flow

so a stock level doing both exists **iff `b_out < s`**. Registered `b_out = 2.0`.
Wheat cleared at the buying node over 12 ticks, one dial moved:

| s | 1 | 2 | 3 | 4 |
|---|---|---|---|---|
| cleared | **0.00** | 18.77 | 93.32 | 86.77 |

Exactly where the arithmetic puts it. Within `cr_01` the **autarkic** region
(Corwen) passes the full criteria while both **traded** regions die — one tape,
one difference, and the difference is the channel.

This also explains a defect already in the record. Phase 4 traced lr's collapse
to *"flour has zero posted supply from tick 0 — that part is pre-existing"*
without saying why: lr's six `flour_transport` desks open holding no flour, so
they post nothing and the importing node's imbalance pins at +1 for ever. lr's
window **is** open (`s = 4`), so **seeding those desks is a cheap experiment that
should work**, and it is the one concrete repair this exercise hands forward.

Registered as check **A5** in `tools/gen_regions.py`, with its verdict shown
flipping on `s` in both directions and on `lr_00` forced to `s = 1`.

### 4.3 `s` is not a free stability dial — and the two constraints on it conflict

`cr_09` is `cr_00` with `s = 4` (lr's registered value) and the cash re-solved.
It is the **worst world in the corpus**: 0/3 alive, bands 1.8e42–9.6e43, worse
than every named defect. Labour cleared at Ashby over 40 ticks is **373.8**
against `cr_00`'s **1280.0** — 29% of the identical economy's throughput.

The pre-registered explanation was *spoilage*: Rule 3 buys `desired · s` while
production consumes `desired · 1`, and `labour` is `Instant`. **That prediction
is half wrong and the falsification test says so.** Re-run with labour's
`shelf_life` changed to `Ticks(8)` so purchased hours survive the activation
period: the world still dies, all three regions still DEAD, labour cleared
*falls* to 329.2. (Flour cleared does roughly double, 245 → 505, so the waste is
real — it is not what kills it.)

The actual mechanism: a non-storable input's supply is a **per-tick flow** and
Rule 3 asks for **S ticks of it in one tick**. Ashby's three desks sit on
inventory indices 0,1,2, so exactly one buys on each of three ticks in four and
none buys on the fourth. Measured labour demand, ticks 1–4: **40, 0, 48, 40**
against a flat supply of 32. Per-tick clearing takes 96 of the 128 hours the
economy needs *at genesis, before any dynamics*, and the rationing compounds.

**The two `s` findings point opposite ways.** §4.2 needs `s > b_out` for a
channel to function; §4.3 needs `s = 1` for a labour market to clear. With
`b_out = 2` there is **no admissible `s`** for a world containing both a channel
and a labour market — which is every `lr_*` tape. That is a design contradiction,
not a tuning problem, and `α·b_out` was already the Phase 5 axis.

### 4.4 The tape format cannot express a consistent transport wedge

`RawSimState.prices` is one price per good, written to every node by
`NameResolver::prices`. A priced transport's zero-profit condition is
`p_to = p_from + a_tr·w`, so a world with one **cannot open on its own
equilibrium vector**: it opens exactly `a_tr·w` low on every importing node and
every transport desk opens at a loss of exactly that. Each of `lr_*`'s six
`flour_transport` desks opens at `−0.1·w`. `cr_01` uses costless channels because
that is the only trade structure this format can start consistent.

### 4.5 A rentier class has no consistent version, and that is a modelling gap

`cr_10` adds one non-labouring pop per region at lr's own sizes. A rentier
consumes a basket that embodies labour; at the zero-profit vector there is no
profit, so the overflow that funds it is **identically zero**. A2 breaks by
exactly their share (0.956); A1, A3, A4 stay exact — they neither supply nor
demand labour. Rent, land and the claim register are Phase 6–7 work; until they
exist a rentier is a pop with a cash balance and a countdown. Every `lr_*` tape
carries three of them.

---

## 5. The honest summary

**Consistency helps the kernel, decisively and only where it already has nothing
to do.** Ten regions alive and in-band against a standing zero; every named
defect worth nine to twenty-five orders of magnitude of band. But every one of
those ten is at a hand-solved rest point, and §4.1 shows the rest point has no
basin — a 6% displacement is already outside it. `cr_00` is a *necessary*
condition passing, not a sufficient one.

**Consistency does not help the legacy arm.** Its median band on `cr_*` (3.96e5×)
is *worse* than on `lr_*` (1.52e5×), and it cannot hold `cr_00` at all (4.4e5×,
all three regions dead) — a world that is provably a fixed point of the rules the
kernel obeys. One exception, and it is the best legacy result in the repository:
**`cr_02` (services-heavy, consistent) runs alive with a band of 3.17× against a
2.2× bar, B7 clean at median |imbalance| 0.045, and flour only 4.56× too dear** —
against lr/legacy's 209×–2.9e6×. Four to five orders of magnitude closer than
anything legacy has managed, and still a fail.

**The pre-registered A/B is still a LOSS** (`receipt-cr-corpus.md`): G1 fails on
four killed regions even though liveness rises 15 → 23, and G2/G3 fail because
the defect worlds' bands are enormous under the kernel and merely large under
legacy. The rule counts liveness first and does not care that the kernel is the
only arm that produced a passing region. That is the rule working as written.
