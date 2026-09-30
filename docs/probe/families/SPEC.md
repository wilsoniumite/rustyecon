# FAMILIES-SPEC: wave A of Phase 2 proper's second session, registered

Dated 2026-09-30. Label `families`, scratch `D:/rustyecon-p24/families/`, on branch `phase2-s2`
from `reboot` at `a483ed0` (worktree `D:/rustyecon-wt/p24`). This frame registers the mirrors'
predictions for wave A, run by run, before any run of the wave (R5): PHASE2-S1 §6's first item,
O22's families on I1–I3 (O109; decision 368, stocks first) and the dial-neighbourhood family on
C1, C2 and IW1 (decision 399 as amended at P2.3.16, item 4). No engine or harness code changes for
it (§3.4). A departure after registration is a dated amendment with its sha256 (decision 282).

**Conditions of every number below, unless its line says otherwise.** The many-market roles as
P2.3 left them (the wall's three optional fields at IW1, the workers' exit at C1 and C2); C2m,
with the one dial change a set names (tilt 1, a map cell, a dial setting); 52 ticks a year, ρ 0,
the fixed basket, planned assignment, no government, no loop; `Saturate`, except in the Hold
family. Tolerance 1e-3 in log on every observable (22 at I1, 13 at I2, 25 at I3,
18 at IW1, 22 at C1 and C2). Classes as PROBE-SPEC §4.5, the runaway bound [1e-6, 1e6] × the
displaced genesis prices (O107). "The mirror" is P2.1's trace mirror `mm_carry.py` at I1–I3, the
wall frame's `wm.py` at IW1 and the commons frame's `cm.py` at C1 and C2, each copied unedited
(§2). Every prediction is the mirror's, run by run, in `registered/`.

## 0. The predictions in brief

**A1, O22's families on I1–I3: every run converges.** 3,063 runs, 3,063 CONVERGED; no DIVERGED,
DEAD, STUCK or ORBITING run; none of the three histories runs away; all five of I3's unrun map
cells are GO, with every kick set predicted to pass.

| set | I1 | I2 | I3 |
|---|---|---|---|
| stocks | 41/41; 470 (586) | 29/29; 1,884 (2,685) | 50/50; 531 (801) |
| joint2 | 60/60; 566 (667) | 60/60; 2,416 (2,726) | 60/60; 691 (878) |
| joint4 | 40/40; 617 (821) | 40/40; 2,601 (2,862) | 40/40; 821 (1,022) |
| basin | 412/412; 539 (851) | 516/516; 2,358 (2,874) | 509/509; 668 (961) |
| Hold, Tiers 1–3 | 107/107; 464 (654) | 77/77; 1,914 (2,910) | 119/119; 574 (856) |
| tilt 1, Tiers 1–3 | 107/107; 355 (678) | 77/77; 655 (919) | 119/119; 380 (541) |
| history, windows in tolerance at their end | 81/81 | 16/81 | 81/81 |

Each cell: CONVERGED of runs; ticks to tolerance, median (slowest), over the non-slack runs.

| I3 map cell (rates, turnover) | L | PL at the base (largest of 9 targets) | Tiers 1–3 | medians (slowest) by tier | verdict |
|---|---|---|---|---|---|
| (×0.5, ×0.5) | 968,000 | 0.993364 (0.993527) | 34/34, 42/42, 43/43 | 739 (1,099), 935 (1,236), 1,182 (1,366) | GO |
| (×0.5, ×1) | 968,000 | 0.991083 (0.992292) | 34/34, 42/42, 43/43 | 581 (741), 706 (1,078), 831 (1,266) | GO |
| (×0.5, ×2) | 968,000 | 0.990009 (0.990894) | 34/34, 42/42, 43/43 | 474 (794), 564 (847), 728 (1,126) | GO |
| (×1, ×0.5) | 484,000 | 0.996518 (0.996940) | 34/34, 42/42, 43/43 | 1,227 (1,560), 1,528 (1,895), 1,881 (2,215) | GO |
| (×1, ×2) | 484,000 | 0.983504 (0.985228) | 34/34, 42/42, 43/43 | 336 (396), 387 (575), 430 (661) | GO |

**A2, the dial neighbourhood: C2 and IW1 converge at every setting; C1 falls into the subsistence
trap with slower prices, faster buffers or a tilt of 0.25 and more, and nowhere else.** 3,281 runs: 3,229
CONVERGED, 34 VACUOUS (C2's `commons=62.4` twice at each setting, by construction, O103) and 18
DIVERGED, all at C1, all Tier-3 price runs, all through the trap:

| C1, Tier 3 | ×0.75 | ×0.9 | ×1.1 | ×1.25 |
|---|---|---|---|---|
| every price rate | 40/43: `p[mach]*0.5`, `JA(0.5)`, `JB(2)` | 41/43: `p[mach]*0.5`, `JB(2)` | 43/43 | 43/43 |
| every buffer | 43/43 | 43/43 | 42/43: `JB(2)` | 40/43: `p[mach]*0.5`, `JA(0.5)`, `JB(2)` |
| every technique rate | 43/43 | 43/43 | 43/43 | 43/43 |

| C1, Tier 3 | tilt 0.05 | 0.1 | 0.25 | 0.5 | 1 |
|---|---|---|---|---|---|
| every tilt | 43/43 | 43/43 | 40/43 | 40/43 | 40/43 |

At tilt 0.25, 0.5 and 1 the three are `p[mach]*0.5`, `JA(0.5)` and `JB(2)`. Every Tier 3S run
converges at every setting, at all three instances (C1 24/24, C2 24/24, IW1 20/20 at each). At
C2m (not rerun here) all of C1's 67 converge (P2.3.15). So in these dials C1's stable region is
one-sided: it ends between C2m and ×0.9 of the price rates, and between C2m and ×1.1 of the
buffers; the technique rates do not reach it; a tilt between 0.1 and 0.25 does. That is the phase diagram's
first slice (§4.3), on the roles as they are. The trap's remedy (O100) is measured on it.

## 1. What is registered, and why

- **O22's families on I1–I3** (MARKETS-SPEC §7.10 items 4–9; P2.1's registration, "Reported
  beside the verdicts"). P2.1's time box stopped after the map's first cells, leaving stocks,
  joint2, joint4, the basin, the history, the `Hold` and tilt-1 variants, and five of I3's map
  cells unrun (MARKETS §3; its open question 2). Decision 368 orders them first, stocks first;
  O109 records that P2.3 did not run them. P2.3's wave ran the wall's and the commons' own
  families only.
- **The dial neighbourhood** (decision 399 as amended, item 4; PHASE2-S1 §5–§6). The fidelity
  review found C1's GO a point result: at every price rate × 0.9 two of its Tier-3 runs fall into
  the subsistence trap, and at every buffer × 1.1 one does. The next wave registers Tier 3 of C1,
  C2 and IW1 at ±10% and ±25% of every rate, buffer and adjust, and at five tilts, as the phase
  diagram's first slice and the measure for O100's remedy. Tier 3S is added, since decision 368
  puts it in both instances' verdicts.
- **Not in this wave**: the trap's remedy (O100), the type switch (O97), the free-good state
  (O96, O101) and the 1750-like instance, which are this session's later items.

## 2. The mirrors, their runners, and the checks made before any prediction was read

**The mirrors, copied unedited** into `model/` (sha256 as each frame registered them):

```
dbd50ad9c1e948aa9bb23545df2dfcb89923cd0d45a0106e8ccfcbbe2798538c  model/p21/mm_carry.py     (P2.1's trace mirror)
528756c1925a6a0a3cdd8dc906ac5a758416e7539e62a9e2afe9a93de3dab407  model/p21/mm.py           (P2.1's predictor's mirror; mstab.py reads it)
1f9ac0bb7815c070af7bcf781b634184e342bf3ac975599c7aba73fb27b1cce2  model/p21/mstab.py        (P2.1's largest root, PL)
a44bd25dc12dfa059a4440e28b047566789d2e33455ff569d4cfbe5c3a21df06  model/p21/battery.py      (P2.1's predictor's runner; fam_i's pattern)
3f93de64e403fbe022a1b33af0f8958ac82b92d9178dca8aba6798c338764c4b  model/wall/model/wm.py    (the wall mirror)
28e61b4c85ae866348b8234328f2d4b911aa8700f6d94a6c2fc439b29035890a  model/wall/model/wall.py  (IW1, its targets and genesis)
00ec06ff27040bfd864eefb0fe0f9c6abc4cd1701dbb9964bf7ee316f61ecb9d  model/wall/model/wb.py    (the wall's runner)
1041f34f0b8058b41f3ae290f332010b16190301bf182f514c7c6e18278a252a  model/wall/model/wstab.py (the wall's PL)
4e6d510d1c7262d17fc586935dd79a58c4d7fb476509965a7ecc6bee32d7710f  model/wall/out/points.jsonl (the wall's registered oracle points)
4d28d20bb6475ea93050743e718c54bab3e61fa3c92c10cac9f9b7a4dd9bdd86  model/commons/model/cm.py         (the commons mirror)
c3ec4a1c346cfa7475b659afbd0ec18788bf5ff43c84ed9becfb140d5c58dc29  model/commons/model/battery_c.py  (the commons' runner)
480f0a85f9e23e05ff8fe13db36814085036a5f0520f547cfc25f5cb6c2e15f7  model/commons/model/instances.py  (C1's and C2's commons)
21970bb5e95ecab27bd77cfcc1b6663d977b64c4eb87f4c6d4f5d73acb9a5115  model/commons/model/elasticity.json
84d9179f6cf7bfd4ebbf3572d2001daf6934753c5b2dd8dc1a97c539feb0e549  model/commons/model/lin_c.py      (the commons' PL)
```

**The runners written for this frame** (in `model/`, sha256 in `SHA256SUMS`):

- `fam_i.py`, O22's families on I1–I3. P2.1's runner (`p21/battery.py`'s `run`, and the commons'
  `battery_c.run`) with the same observables, tolerance, live floor, classes, early stop and
  ticks to tolerance, and six changes so the mirror reads a run as the harness does at I1–I3
  (its docstring lists them): the tick map is `mm_carry`'s; the run grammar is the harness's
  (`w*F`, `coin.desk.D*F`, `stock.D*F`, `joint(F,SEED)`, `cycle(C,P,N)`, and the rest); `joint`
  draws in the harness's order (labour, land, the types, the categories, then the shares) with
  the harness's own factors; a stocks start's distance is the largest |ln F| over 1e-3; the
  runaway bound is read against the displaced genesis prices; the dials and the one-sided rule
  pass through. `run_cycle` reads the history window by window, as the commons' `run_cycle`.
- `fam_dial.py`, the dial family. Each run is the frame's own runner, `wb.run` at IW1 and
  `battery_c.run` at C1 and C2, with the setting passed as its `over` dials (`rate.<market>`
  scales every price rate, the reserved labour markets included; `turnover.<desk>` every desk's
  buffer; `adjust.<desk>` every technique rate; `tilt.<desk>` sets every tilt), as the harness's
  `--set rate.*=F`, `buffer.*=F`, `adjust.*=F`, `tilt.*=V` do. Genesis coins follow the dials in
  both. Two readings are added after the runner, each the harness's: Tier 3S's start is the
  largest D̂ of its first year (`wb.run(d0_first_year=True)`; at the commons the pass of the
  commons registration's `tier3s_c.py`, with the dials, VACUOUS where it is at most 1); and a
  DIVERGED run's runaway tick is read on the displaced genesis prices by a second pass of the
  same map, as the commons' `diag/runaway_ref.py` did.
- `run_all.py` runs every set (§3) and writes `registered/`; `summary.py` writes §4's tables
  (`registered/summary.out`).
- `make_draws.py` and the scratch program `draws/` (outside the workspace, `libm = "=0.2.16"`, as
  `Cargo.lock` pins it) write `registered/draws.json`: each `joint(F,SEED)` factor as the harness
  draws it, SplitMix64 and libm's `pow`.

**The checks, made before any prediction of this frame was read** (outputs in `registered/`):

1. **`check_i.out`: the I1–I3 runner is the engine.** P2.1's verdict battery at C2m, 303 runs
   (I1 107, I2 77, I3 119), through `fam_i.run`: the class equal to the engine's (P2.1's
   `results/markets/battery.csv`) in 303 of 303, and the ticks to tolerance in 303 of 303, to the
   tick. (P2.1's predictor, on `mm.py` without the carry, matched the ticks in 571 of P2.1's
   623 CONVERGED runs over all its instances, MARKETS §2.)
2. **`check_dial.out`: the dial runner at C2m is the engine.** Tier 3 and Tier 3S of IW1 (59), C1
   (67) and C2 (67) through `fam_dial` with no setting: the class and the ticks to tolerance equal
   the engine's in P2.3's scored wave (`results/wall/battery.csv`, `results/commons/runs.csv`) in
   193 of 193.
3. **`check_runaway.out`: the runaway reading.** Every trap run of P2.3's commons joint and basin
   families (190) through `fam_dial.commons_runaway` at C2m: the engine's tick in 185, one tick
   apart in 5 (the commons' `diag/runaway_ref.out` found the same 5).
4. **`check_joint.out`: the joint draws are the harness's.** For every seed of joint2 and joint4
   at I1, I2 and I3 (300 draw sets), the harness's tape (`markets tape`, genesis prices and
   technique shares at full precision) equals the base tape times `draws.json`'s factors, rounded
   once, bit for bit in 300 of 300. With Python's `**` (glibc's `pow`) it does in 126 of 300.
   `check_joint_c.out` finds the same at C1, C2 and IW1, whose registered mirrors drew with
   Python's `**`: 43, 40 and 46 of 100 bit for bit with glibc's factors, 100 of 100 with libm's
   (O110).

These checks ran the mirrors and read the frozen binary's lists and tapes (`markets list`,
`markets tape`, which run no tick). No engine run of this wave's sets was made. **Engine results
known before this registration**, disclosed: the fidelity review and its recheck (P2.3.16) ran
Tier 3 (and IW1's Tier 3S) at every price rate × 0.9 and every buffer × 1.1 at C1, C2 and IW1, at
6,000 ticks (42,000 for dated runs), and I1's `p[mach]*0.5` and `JB(2)` at seven settings
(`D:/rustyecon-p23/recheck/dial.out`, `dial2.out`). Six of this family's 51 (instance, setting)
cells overlap them. The mirror's predictions here were made on the registered mirrors, unedited,
through the runners above; they give those engine runs' classes, and their runaway ticks to the
tick (C1 at rate × 0.9: `p[mach]*0.5` 325, `JB(2)` 335; at buffer × 1.1: `JB(2)` 309).

## 3. The runs (the engine's commands)

Run names are the frozen binary's own (`markets list <family> --inst <id>`, saved in
`build/lists/`, sha256 in `SHA256SUMS`); each set's list equals the registered rows name for name.

### 3.1 A1: O22's families on I1–I3, stocks first

L is P2.1's registered L at C2m and 52 a year: I1 144,000, I2 52,000, I3 484,000.

| set | command | runs (I1, I2, I3) |
|---|---|---|
| stocks | `markets run <name> --inst <id> --ticks L` over `markets list stocks` | 41, 29, 50 |
| joint2, joint4 | the same over `list joint2`, `list joint4` | 60, 60, 60; 40, 40, 40 |
| basin | the same over `list basin` (1.05^j, j = ±1 … ±43, for w, r, the largest-share good, each type and the technique of the desk with the most tasks; a share past 1 left out) | 412, 516, 509 |
| history | `markets run cycle(<first coefficient>,1500,80) --ticks 121500`, every tick in the CSV, read in 81 windows of 1,500 ticks (the commons' form; the first coefficient is `land.mach` at I1, `land.power` at I2 and I3) | 1, 1, 1 |
| Hold | the battery, Tiers 1–3, with `--one-sided hold` | 107, 77, 119 |
| tilt 1 | the battery, Tiers 1–3, with `--set tilt.*=1` | 107, 77, 119 |
| I3's map cells | `--set rate.*=fr --set buffer.*=ft`, (fr, ft) = (0.5, 0.5), (0.5, 1), (0.5, 2), (1, 0.5), (1, 2): mode A (`run hold`), the battery (Tiers 1–3) at L = 484,000/fr (P2.1's map rule), and `markets kick` at the base and at each of the eight cost targets (`<coef>=<v>@dated`), H = L | 5 × (1 + 119), 45 kick sets |

### 3.2 A2: the dial neighbourhood on C1, C2 and IW1

Each instance's registered L at every setting: C1 141,000, C2 142,000, IW1 22,000. The 17
settings: `--set rate.*=F`, `--set buffer.*=F`, `--set adjust.*=F` for F = 0.75, 0.9, 1.1, 1.25,
and `--set tilt.*=V` for V = 0.05, 0.1, 0.25, 0.5, 1. At each:

- Tier 3 (the battery's tier-3 runs: C1 and C2 43, IW1 39), `markets run <name> --inst <id> --set
  <setting> --ticks L`;
- Tier 3S (`markets list tier3s`: C1 and C2 24, IW1 20), the same with `--first-year`;
- the base kick set, `markets kick hold --inst <id> --set <setting> --ticks L --horizon L`,
  reported beside the mirror's PL at the base with that setting.

3,281 runs and 51 kick sets. Tier 3 at 10·L is not run: the family is reported, not a verdict.

### 3.3 The wave

6,347 runs and 96 kick sets, about 2,300 million ticks: one `markets` process a job, in the order
of §3.1 then §3.2 (stocks first), the scorer committed before the first job (decision 311). The
job list, job and runner scripts, gather script, scorer and self-test are
`docs/probe/results/families-wave/` (P2.4.2).

### 3.4 No new harness grammar

Every run above is expressible in the grammar P2.1–P2.3 built: the families by name, `--set` with
`rate.*`, `buffer.*`, `adjust.*` and `tilt.*`, `--one-sided hold`, `--first-year`, and `markets
kick` with `--set`. So the wave adds no code, and no engine, harness or tape changes: every pin
and every committed tape's text, ids and streams are as at `a483ed0`.

## 4. Registered predictions

Every run's prediction is in `registered/` (sha256 in `SHA256SUMS`), one JSON line a run: its
name, tier and slack mark where it has them, L, the class, the ticks to tolerance (`ttol`), D̂ at
the start (`D0`), the peak D̂, dead ticks, the lowest baskets eaten over Y\*, ticks with none, the
runaway tick on the harness's reference (`runaway_t`), the ticks the mirror ran and whether it
stopped early. The files: `i_stocks.jsonl`, `i_joint2.jsonl`, `i_joint4.jsonl`, `i_basin.jsonl`,
`i_hold.jsonl`, `i_tilt1.jsonl`, `history.json` (window by window), `i3_map.jsonl`,
`i3_map_pl.jsonl` (PL at each map cell's nine targets), `dial.jsonl`, `dial_pl.jsonl` (PL at the
base at each setting). `summary.out` has the tables.

**The early stop.** The mirrors stop a run once D̂ has stayed below 1e-3 (a gap of 1e-6 in log)
for 2,000 ticks, from tick 4,000 on, as every P2 mirror has; such a run is CONVERGED by
construction, and its class and ticks to tolerance are those of the run to L (P2.3's wall frame
ran ten of its slowest to L without the stop and each kept both).

### 4.1 A1 in detail

- **stocks, first** (MARKETS-SPEC §7.10 item 5): every desk's coin × 0.02, 0.1, 0.5, 2, every
  desk's stock × 0.1, 0.5, 2, each type's stock × 0.01 and × 10, the workers' coin × 0.1, 2, the
  provider's × 0.5, 2: 120 of 120 CONVERGED. The deepest: I1's worst dead ticks 101 and a trough
  of 0 baskets over Y\* in 2 ticks; I2 136 dead ticks at worst.
- **joint2 and joint4**: 300 of 300 CONVERGED. Worst dead ticks 154, 175 and 188 (joint2) and
  211, 284 and 308 (joint4) at I1, I2 and I3; at I1 up to 7 (joint2) and 16 (joint4) ticks with
  no baskets. No I1–I3 run enters a trap: these economies have no priced exit.
- **basin**: 1,437 of 1,437 CONVERGED, from ×/÷8.15 in every variable. At I1 up to 24 ticks with
  no baskets and 259 dead ticks.
- **Hold**: 303 of 303 CONVERGED. It equals `Saturate` run for run but in two I1 runs whose
  markets go one-sided for a few ticks, `JB(0.5)` (ticks to tolerance 531 against 538; dead 66
  against 67) and `x*/2` (527 against 525).
- **tilt 1**: 303 of 303 CONVERGED, faster than C2m's tilt 0: median ticks 355 against 464 at I1,
  655 against 1,914 at I2, 380 against 574 at I3 (over Tiers 1–3, non-slack; C2m's from
  `check_i.out`'s runs, which are P2.1's engine runs tick for tick). I1's worst dead ticks rise
  to 256 and its basket-less ticks to 25.
- **The history**: no runaway. I1: 81 of 81 windows in tolerance at their end, the slowest 577
  ticks, at most 127 dead ticks in a window. I3: 81 of 81, the slowest 699, at most 124 dead.
  **I2: 16 of 81**: I2 is slower than the window (its median Tier-1 time is 1,594 ticks, P2.1),
  so only the base window and 15 of the 16 × 1.1 windows end within tolerance, each from tick
  1,491 of 1,500 (its last D̂ 0.742); from window 6 on the windows repeat with a period of five,
  their last D̂ 0.742, 2.41, 10.8, 16.4 and 37.3 (O111).
- **I3's map cells**: GO in all five (§0's table). Mode A holds in each (the mirror's `hold`
  stays within a gap of 1.2e-14). The kick sets pass at all 45 targets by PL (the largest
  0.996940, whose PL^(0.9·L) is about 1e-580). The deepest cell is (×0.5, ×2): up to 460 dead
  ticks in Tier 3, and baskets over Y\* down to 0 in Tiers 2 and 3. P2.1's predictor gave the
  same cells GO on `mm.py` at 60,000 ticks (PREDICTION §5.2), its PL within 1e-4 of these.

### 4.2 A2 in detail

Tier 3 and Tier 3S at each setting, beside C2m (the frames' registered runs, which P2.3.15's
engine gave run for run). Ticks to tolerance over every CONVERGED run, slack runs included; the
most dead ticks over both tiers; a DIVERGED run's runaway tick on the harness's reference. PL is
the largest root per tick at the base with that setting (the frames' own measures: `lin_c.pl`, 8
directions over ticks 10,000–40,000, at C1 and C2; `wstab.pl`, 4 directions over 2,000–12,000,
at IW1); every one is below 1, so every base kick set is predicted to pass.

**C1**

| setting | PL at the base | Tier 3: CONVERGED; ticks, median (slowest) | Tier 3S | most dead ticks | DIVERGED (runaway tick) |
|---|---|---|---|---|---|
| C2m (P2.3.15) | 0.986116 | 43/43; 503 (828) | 24/24; 379 (534) | 183 | — |
| `rate.*=0.75` | 0.984868 | 40/43; 450 (820) | 24/24; 384 (499) | 388 | `p[mach]*0.5` (379), `JA(0.5)` (380), `JB(2)` (391) |
| `rate.*=0.9` | 0.985302 | 41/43; 438 (908) | 24/24; 377 (569) | 316 | `p[mach]*0.5` (325), `JB(2)` (335) |
| `rate.*=1.1` | 0.986929 | 43/43; 479 (916) | 24/24; 373 (557) | 154 | — |
| `rate.*=1.25` | 0.987840 | 43/43; 496 (862) | 24/24; 384 (595) | 127 | — |
| `buffer.*=0.75` | 0.990214 | 43/43; 600 (1175) | 24/24; 448 (737) | 159 | — |
| `buffer.*=0.9` | 0.987825 | 43/43; 522 (1012) | 24/24; 410 (605) | 172 | — |
| `buffer.*=1.1` | 0.984279 | 42/43; 440 (814) | 24/24; 354 (515) | 291 | `JB(2)` (309) |
| `buffer.*=1.25` | 0.981505 | 40/43; 349 (677) | 24/24; 327 (485) | 292 | `p[mach]*0.5` (287), `JA(0.5)` (287), `JB(2)` (297) |
| `adjust.*=0.75` | 0.985913 | 43/43; 491 (949) | 24/24; 388 (533) | 179 | — |
| `adjust.*=0.9` | 0.986080 | 43/43; 491 (830) | 24/24; 380 (534) | 182 | — |
| `adjust.*=1.1` | 0.986106 | 43/43; 491 (826) | 24/24; 376 (535) | 185 | — |
| `adjust.*=1.25` | 0.986150 | 43/43; 502 (825) | 24/24; 369 (539) | 187 | — |
| `tilt.*=0.05` | 0.984468 | 43/43; 407 (820) | 24/24; 350 (519) | 180 | — |
| `tilt.*=0.1` | 0.982817 | 43/43; 357 (665) | 24/24; 343 (455) | 183 | — |
| `tilt.*=0.25` | 0.979240 | 40/43; 338 (516) | 24/24; 228 (381) | 304 | `p[mach]*0.5` (295), `JA(0.5)` (288), `JB(2)` (317) |
| `tilt.*=0.5` | 0.977136 | 40/43; 307 (449) | 24/24; 259 (349) | 297 | `p[mach]*0.5` (293), `JA(0.5)` (287), `JB(2)` (305) |
| `tilt.*=1` | 0.983373 | 40/43; 412 (646) | 24/24; 338 (470) | 300 | `p[mach]*0.5` (295), `JA(0.5)` (287), `JB(2)` (297) |

**C2**

| setting | PL at the base | Tier 3: CONVERGED; ticks, median (slowest) | Tier 3S | most dead ticks | DIVERGED (runaway tick) |
|---|---|---|---|---|---|
| C2m (P2.3.15) | 0.985988 | 41/41; 511 (641) + 2 VACUOUS | 24/24; 340 (543) | 153 | — |
| `rate.*=0.75` | 0.984901 | 41/41; 474 (782) + 2 VACUOUS | 24/24; 394 (580) | 230 | — |
| `rate.*=0.9` | 0.985419 | 41/41; 504 (667) + 2 VACUOUS | 24/24; 358 (570) | 172 | — |
| `rate.*=1.1` | 0.986479 | 41/41; 497 (709) + 2 VACUOUS | 24/24; 430 (600) | 137 | — |
| `rate.*=1.25` | 0.987080 | 41/41; 529 (655) + 2 VACUOUS | 24/24; 414 (587) | 119 | — |
| `buffer.*=0.75` | 0.989421 | 41/41; 650 (820) + 2 VACUOUS | 24/24; 522 (737) | 147 | — |
| `buffer.*=0.9` | 0.987444 | 41/41; 527 (776) + 2 VACUOUS | 24/24; 366 (567) | 150 | — |
| `buffer.*=1.1` | 0.984465 | 41/41; 486 (601) + 2 VACUOUS | 24/24; 330 (519) | 154 | — |
| `buffer.*=1.25` | 0.982223 | 41/41; 398 (589) + 2 VACUOUS | 24/24; 311 (495) | 158 | — |
| `adjust.*=0.75` | 0.985893 | 41/41; 514 (637) + 2 VACUOUS | 24/24; 343 (543) | 151 | — |
| `adjust.*=0.9` | 0.985902 | 41/41; 513 (639) + 2 VACUOUS | 24/24; 341 (543) | 152 | — |
| `adjust.*=1.1` | 0.986018 | 41/41; 510 (641) + 2 VACUOUS | 24/24; 340 (543) | 153 | — |
| `adjust.*=1.25` | 0.986027 | 41/41; 508 (642) + 2 VACUOUS | 24/24; 342 (543) | 153 | — |
| `tilt.*=0.05` | 0.984159 | 41/41; 443 (609) + 2 VACUOUS | 24/24; 331 (527) | 151 | — |
| `tilt.*=0.1` | 0.982341 | 41/41; 386 (605) + 2 VACUOUS | 24/24; 316 (494) | 149 | — |
| `tilt.*=0.25` | 0.978552 | 41/41; 355 (481) + 2 VACUOUS | 24/24; 222 (394) | 146 | — |
| `tilt.*=0.5` | 0.976482 | 41/41; 306 (464) + 2 VACUOUS | 24/24; 246 (333) | 168 | — |
| `tilt.*=1` | 0.983217 | 41/41; 412 (640) + 2 VACUOUS | 24/24; 346 (456) | 213 | — |

**IW1**

| setting | PL at the base | Tier 3: CONVERGED; ticks, median (slowest) | Tier 3S | most dead ticks | DIVERGED (runaway tick) |
|---|---|---|---|---|---|
| C2m (P2.3.15) | 0.987132 | 39/39; 561 (705) | 20/20; 404 (550) | 117 | — |
| `rate.*=0.75` | 0.990265 | 39/39; 745 (951) | 20/20; 532 (737) | 170 | — |
| `rate.*=0.9` | 0.988383 | 39/39; 614 (773) | 20/20; 456 (599) | 134 | — |
| `rate.*=1.1` | 0.985911 | 39/39; 513 (620) | 20/20; 378 (513) | 104 | — |
| `rate.*=1.25` | 0.984346 | 39/39; 445 (556) | 20/20; 347 (438) | 89 | — |
| `buffer.*=0.75` | 0.986890 | 39/39; 560 (691) | 20/20; 416 (543) | 111 | — |
| `buffer.*=0.9` | 0.986964 | 39/39; 559 (684) | 20/20; 409 (556) | 115 | — |
| `buffer.*=1.1` | 0.987026 | 39/39; 556 (703) | 20/20; 402 (545) | 119 | — |
| `buffer.*=1.25` | 0.986870 | 39/39; 547 (697) | 20/20; 413 (539) | 122 | — |
| `adjust.*=0.75` | 0.987132 | 39/39; 561 (705) | 20/20; 404 (550) | 117 | — |
| `adjust.*=0.9` | 0.987132 | 39/39; 561 (705) | 20/20; 404 (550) | 117 | — |
| `adjust.*=1.1` | 0.987132 | 39/39; 561 (705) | 20/20; 404 (550) | 117 | — |
| `adjust.*=1.25` | 0.987132 | 39/39; 560 (705) | 20/20; 404 (550) | 117 | — |
| `tilt.*=0.05` | 0.987044 | 39/39; 555 (700) | 20/20; 400 (544) | 116 | — |
| `tilt.*=0.1` | 0.986956 | 39/39; 548 (695) | 20/20; 397 (541) | 115 | — |
| `tilt.*=0.25` | 0.986640 | 39/39; 521 (667) | 20/20; 388 (531) | 113 | — |
| `tilt.*=0.5` | 0.986245 | 39/39; 497 (653) | 20/20; 366 (497) | 108 | — |
| `tilt.*=1` | 0.985464 | 39/39; 457 (637) | 20/20; 328 (467) | 97 | — |

- **Every DIVERGED run is the trap** (the commons frame's §6.8): the three runs are the basin's
  side, a displacement that lowers the wage against food or cheapens machines. Their runaway
  ticks fall between 287 and 391 ticks (5.5–7.5 years), beside P2.3's 279–369.
- **The local root does not see it.** Where the trap appears at C1, PL at the base is
  0.977–0.985; where it does not, 0.983–0.990 (C2m's 0.986): the point is locally stable at
  every setting, and the trap is a state far from it, not a slow or unstable mode of it.
- **Tier 3S never enters it**: stocks and coins at × 0.5 and × 2 leave prices at the point.
- **IW1's technique rate moves nothing** in these runs: at the wall s\* = 0, and only `x*/2`
  displaces a share, which returns while prices are still the slowest observable (the wall
  frame's §6.2).


### 4.3 The phase diagram's first slice

At C2m and 52 ticks a year, on the roles as they are, and reading only Tier 3 and 3S:

- **C1** converges on this slice where every price rate is at C2m's or faster (× 1 to × 1.25),
  every buffer at C2m's or slower (× 0.75 to × 1), at any technique rate from × 0.75 to × 1.25,
  and at a tilt of at most 0.1. Outside it the trap takes two or three Tier-3 runs, always from
  the same three (`p[mach]*0.5`, `JA(0.5)`, `JB(2)`), each a price displacement that lowers the
  wage against food or cheapens machines (the basin's side, the commons SPEC §6.5). The edge lies
  within 10% of C2m on two dials. A reading, not tested: the three moves that reach the trap
  (slower prices, faster buffers, a tilt) each make quantities respond faster against prices,
  and the technique rate, which does not, leaves it alone.
- **C2 and IW1** converge in every non-vacuous run of their 2,142 (C2's 34 VACUOUS runs are its
  two `commons=62.4` runs at each setting): at these dials neither has an edge within ±25%.
- **The runaway ticks** fall between 287 and 391 ticks on the harness's reference (5.5–7.5
  years), beside the trap's 279–369 at C2m (P2.3).

## 5. How the engine run is scored (the bands)

The scorer (`results/families-wave/score.py`, committed before the wave) reads each band so:

### 5.1 Every run

- **The class exactly**, the mirror's, in every set, VACUOUS included.
- **Ticks to tolerance within 10%** where both are CONVERGED, with up to three runs a set charged
  within 25% (a set: an instance's family, one I3 map cell, one instance's dial setting).
- **A DIVERGED run's runaway tick within 5%** on the scored clock, on the harness's reference.
- **Dead ticks within 10% or 5**, where the classes agree and the run did not diverge.
- **The lowest baskets eaten over Y\* within 0.05 absolute** where the mirror's is above 0.1;
  reported below it.
- **Every CONVERGED engine run ends within 1e-12 in log of its oracle point** (its last D̂ at most
  1e-9): no dial here moves the point.

### 5.2 The history

Window by window, as `fam_i.run_cycle` reads it: in tolerance at the window's end as the mirror's;
ticks to tolerance within 10% (three windows an instance within 25%); dead ticks within 10% or 5;
no runaway.

### 5.3 I3's map cells

Mode A PASS; each kick set passing as its target's PL predicts (PASS iff PL^(0.9·L) ≤ 1e-3,
MARKETS-SPEC §7.5 on the mirror); the cell's verdict (MARKETS-SPEC §7.9 with the kick: GO,
LOCAL, NO-GO) as registered.

### 5.4 Reported, not scored

The dial family's 51 base kick sets (beside the mirror's PL); peak D̂, worst fills, transfer
shortfalls, the commons' regime readouts and the wall's depth readouts, which the engine writes
in `stats.tsv`.

### 5.5 What would refute the frame

A class not the mirror's in any run; a CONVERGED run ending off its oracle point; a scored kick
set or a map cell's verdict not as registered. Any one sends the family back to its mirror, with
the finding reported: at I1–I3 a finding about P2.1's mirror, at C1, C2 and IW1 about the frames'.

## 6. Readings, declared before the wave

- **×0.75, not ×0.8.** Decision 399 as amended (item 4) lists "× 0.9, 1.1, 0.8 and 1.25"; its
  source, PHASE2-S1 §5 and §6, says "±10% and ±25%", and the session's brief says ×0.75. The
  family takes ×0.75, the −25% (decision 401).
- **Five tilts.** 399 lists tilt 0.05, 0.1, 0.25, 0.5 and 1; the brief 0.05, 0.1 and 1. The
  family runs all five.
- **L at every setting is the instance's registered L**, not the elasticity rule's at that
  setting (at ×0.75 of every rate the rule would give about 188,000 at C1). Every run the mirror
  predicts converges in under 1,400 ticks or runs away in under 400, far inside L.
- **The variants run Tiers 1–3**, as the commons ran its; the wall ran Tiers 1–2.
- **The history runs 81 windows of 1,500 ticks**, the commons' form; the wall ran 80 windows and
  a last one of L.
- **The map cells' L is 484,000/fr**, P2.1's map rule (MARKETS `families.csv`).
- **The joint draws are the harness's own doubles** (`draws.json`), not Python's `**`.

## 7. Decisions proposed and open items

Numbered at registration from this session's range (400–449, O110–O129); each decision is open
to veto, and the alternative named is the registered one (R6).

400. **Wave A is one registration and one scorer: O22's families on I1–I3 and the dial
     neighbourhood of C1, C2 and IW1.** A1 runs MARKETS-SPEC §7.10 items 4–9 as P2.1 left them,
     stocks first, on I1–I3 at C2m and P2.1's L: stocks, joint2, joint4, basin, the history in
     81 windows of 1,500 ticks, the whole battery under `Hold` and with every tilt 1, and I3's
     five unrun map cells with mode A, the battery at L/fr and a kick set at each of their nine
     targets, each cell read as MARKETS-SPEC §7.9. The families are reported, not verdicts. No
     harness grammar is added: the grammar P2.1–P2.3 built expresses every run, so the wave adds
     no code. Alternatives: the variants on Tiers 1–2, as the wall's; the history as the wall's;
     the map cells without kick sets; a dial-family grammar with its test.
401. **The dial neighbourhood: every price rate, every buffer and every technique rate × 0.75,
     0.9, 1.1 and 1.25, and every tilt at 0.05, 0.1, 0.25, 0.5 and 1, over Tier 3 and Tier 3S
     of C1, C2 and IW1, at each instance's registered L,** with the base kick set at each setting
     reported beside the mirror's PL. It reads 399(4)'s "0.8" as the −25% PHASE2-S1 names and
     keeps its five tilts. Alternatives: 399(4) as written; L by the elasticity rule at each
     setting; kick sets at every target.
402. **The mirrors and their runners (§2).** I1–I3 on P2.1's `mm_carry.py` through `fam_i.py`
     with the harness's grammar, draws and runaway reference; C1, C2 and IW1 on the frames' own
     runners, unedited, with Tier 3S's first-year start and the harness's runaway reference read
     after them. Each checked against the engine's committed runs before any prediction was
     read (§2, checks 1–4). Alternatives: `cm.py` with the exit absent for I1–I3; glibc's `pow`
     for the draws; the frames' undisplaced runaway reference.
403. **The bands and the refutations (§5), P2.3's.** Alternative: classes only, the families
     being reported.

- **O110. The frames' joint draws are glibc's; the harness's are libm's.** At C1, C2 and IW1
  the registered mirrors drew `joint(F,SEED)` with Python's `**` (glibc's `pow`); the harness
  draws with libm 0.2.16's `pow`. Against the harness's tapes (`check_joint_c.out`, 2026-09-30),
  glibc's factors give its genesis bit for bit in 43, 40 and 46 of 100 draw sets, libm's in 100
  of 100; the rest differ by an ulp in one factor or more. P2.3's per-seed classes and ticks
  matched regardless (the commons' E0 compared seed 1 and found one 2.2e-16 difference). This
  registration draws with the harness's generator (`draws.json`); later ones should.
- **O111. I2's history is slower than its window.** In the mirror 65 of I2's 81 windows of
  1,500 ticks end out of tolerance, and the 15 × 1.1 windows that end within it get there at
  tick 1,491; I2's median Tier-1 time is 1,594 ticks (P2.1). The family then measures I2's
  slowness, a cycle of period five with no runaway, not its recovery from each step. A history
  at a longer period would; it is not registered here.
- **O112. The dial family reads Tier 3 and 3S without their targets' kicks.** Only the base's
  kick set runs at each setting, and the mirror's PL is computed at the base only. A setting
  that made a cost target slowly unstable could show as CONVERGED within L, as REPORT §5 found
  at P2.0 before the kick joined the classes. The phase diagram proper (next step 7 item 5)
  should run each target's kick at each dial it maps.

## 8. Files

`SHA256SUMS` lists every file of this frame with its sha256: this SPEC, `registered/` (the
predictions, the checks' outputs, `draws.json`, `summary.out`), `build/lists/` (the harness's
lists) and `model/` (the copied mirrors and the runners written here). To rerun (WSL):

```
cd /mnt/d/rustyecon-p24/families/model
python3 check_i.py && python3 check_dial.py && python3 check_runaway.py
python3 make_draws.py <families-draws>
python3 check_joint.py <markets> && python3 check_joint_c.py <markets> <families-draws>
python3 run_all.py && python3 summary.py > ../runs/summary.out && python3 dial_table.py > ../runs/dial_table.md
bash ../assemble.sh     # copies the outputs to ../registered/ and writes ../SHA256SUMS
```

`<markets>` is the harness at `a483ed0` (WSL release, sha256 `be3266ab…42e9`), `<families-draws>`
the scratch program `draws/` built with `cargo build --release --offline`.
