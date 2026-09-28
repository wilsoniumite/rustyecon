# P2.2a registration: the stocks probe (HORSES-SPEC §7.12)

Dated 2026-09-28. Written at step P2.2.2, after P2.2.1 and the checks HORSES-SPEC §7.8 allows
before registration (the probes, the trace diff, mode A and the nesting checks), and **before any
mode-B run**: no battery, family or displaced run has been scored. Any later change is a new dated
registration, and earlier results stay reported.

## 1. What is frozen

- **The build.** Branch `phase2-goods`, commit `d636b76` (P2.2.1: the stock roles, the horses
  tapes, their generator and harness), clean. The registration commit (P2.2.2) adds documents and
  run lists only; no source, tape or test changes.
- **The frame.** `D:/rustyecon-p2g/frame/HORSES-SPEC.md`, sha256
  `0e23e80936db3175741fceed6905de39131ad0ab65f97e86bf2bda42c71cd5dd`, with every derivation file
  beside it listed in its `SHA256SUMS` (sha256 of that list
  `f2ebda01ffe6a9ad4eef3a3927e2a5c0d78d0a2f328c44169c3f57b00217d57d`; every entry verified on
  2026-09-28). Its §6 (the prediction) and §7 (the measurement protocol) are copied unedited in
  §6 and §7 below; where they and this registration differ, §3–§5 below say so and why.
- **The rules as built.** `docs/probe/HORSES-RULES.md` at P2.2.1 (sha256
  `a57df7a0e7f71a0daadcb495d55ed8e6ee95f9419e8b9b9bac2b35ffbef7ca55`), with its departures from the
  frame (§8 there), and the sources, tapes and scripts in §2.
- **The design sources**, read-only: `D:/rustyecon-goods/GOODS-CHAIN.md` (sha256
  `d4be4b2d57bea59cb318920fdae20af13b978020f734d45607fa363d82a65ace`).

## 2. Sources, tapes and scripts (sha256)

The roles (crates/agents, at `d636b76`):

```
1adc15f85beab24a4981398f7299ed7f202a0536529ee2dcf95bbae228832eb5  crates/agents/src/roles/stock/mod.rs
9e5688594ab322a07734e80ed9c10d41432aeb7fbe707deedfb660fb24dde44c  crates/agents/src/roles/stock/rules.rs
64efe789e82f7f8ae789dac4d928d8b7c64209a1ea55d516b66ae8efb13e7fe8  crates/agents/src/roles/stock/spec.rs
e7ec282496525457b0749b5f4e6f399ccaa7a5c70efd3c3539a9d66e51748c6e  crates/agents/src/cast.rs
2fa6edecd414876b590880fa0c078e18b79f54c921d69cb6b4d6705a2655fb22  crates/agents/src/ext.rs
186df36683611a50c0aac8630fd1e62636a4d2b19513a49d93d9b73a5a94766c  crates/agents/src/spec.rs
```

The harness (crates/probe, at `d636b76`):

```
fef3952ca6fbf318b9475ee2667e1194fbe83672d4f864ae121105df3a5d5809  crates/probe/src/horses/cli.rs
e90d08e439e815a498dade258f543ed96ea40abb84a5ae2e0b912d5af347cdd5  crates/probe/src/horses/harness.rs
a3155d19c9559d23f7d4a4ab47a76553b360bbf2cb86ff9908c1a397d7809f80  crates/probe/src/horses/instance.rs
fef7ecf4448cada631e41b3e55f4484d07df4638b2957aa3deae72da498a899e  crates/probe/src/horses/kick.rs
da673de05bd434cd69d8b1d4ab0e6b9f9d37c8fd35af7ce229cde11a7d5b89a9  crates/probe/src/horses/mod.rs
cd07a5e7a6d3c94fdb026b6a6bf9a2ba11fd6c7bdde486b8d78b1df60dd5fc2e  crates/probe/src/horses/perturb.rs
f04f34fb9af5af979a6dfac6df8129f5fee91dacd85ba609b36ae07377a03248  crates/probe/src/horses/probes.rs
3ca4a10a8fffb36a0960a60c78334cb491405675ce843960c9d53b6ad73de847  crates/probe/src/horses/setup.rs
673215e3aefe99b9c234f7f1282eba7dfd045e98bdd9cf7059b3f916aaee68a0  crates/probe/src/bin/horses.rs
be4fe26a97e94969531500307fa77deca43da0b81e6168be3ad8c57b8d4c7414  crates/probe/src/bin/horses-tape.rs
ac3a9dbe1d84f544f6c1eafaf4d56cd0d41eb21a0ef232304f6bd0c4f162041b  crates/probe/src/markets/harness.rs
```

The tapes, each its generator's output (`horses_tapes_are_their_generators_output`):

```
a9897fd038ed2a43d46dc4bca5cd050d2896d371d2a824d1218af75f857f3124  tapes/horses-h1.ron
92b28e52b3a6e540b6ebefa0d9fae174ed42739d903842497c54b7a5b71975fd  tapes/horses-h2.ron
6d5b04759a7725ceb014a7438b96c2dff315f94a8151cc41e6f35c03cf215bb2  tapes/horses-h3.ron
0c8c73be6efd68c07eb38ce631ccf4d395236aeff3358ccb9a906aa61f748969  tapes/horses-h4.ron
d08d21a8acf2823e21af3f8ef71009c6578afe8bcd30ece57e3bf19ec1d8aa54  tapes/horses-p7.ron
cef45808bd0a146590cc618dd0c5c54b4d24d332ba16529becba75c98f198f77  tapes/horses-r1a.ron
```

The trace diff (`D:/rustyecon-p2g/build/`):

```
45a67ef72d5923c7acc7511342fc9c43941e73d7ca316d8a1c68aed219af15a0  make_h_carry.py
6b745a16a48101fd533e6b009e807e7c710df87928382b22960e033282dce5ae  h_carry.py
0bfcca60d845716e8c91c781e3d66a5dd426db98034fa4632f7907fdc9840709  tracediff.py
32d63dfaeca5867657437e1790345ffefced164bff0275cfe7ccf19fec4fbd31  tracediff.out
```

The run lists (`lists/`, from `horses list battery|stocks --inst ID`; the battery's A0 lists
are one list, F5's and F6's the same less b ×2):

```
d5af52abe63bf9852473bb68ee5b74d894f2f4e8ec572f05b9c5237e629a529d  battery-{h1,h2,h3,h4,f1,f2,f3,f4,f7,f8,f9,f10}.tsv
d57be093797dfc51fe9d53be36d7db4986a8f23514797360d83c7f2c4c2f9f39  battery-{f5,f6}.tsv
5f226bc3770446fe33695f3a7c81fe81e957f50a95dd149e0749cda81f310528  stocks-{every family}.tsv
```

## 3. What is fixed

- **The instances** (HORSES-SPEC §1; `probe::horses::instance`): H1–H4 (A0 under rule A, δ 10%
  and 8% a year, ω ½ and 1) carry the verdict; F1–F10 are the families (δ 4%; ω 0.85; v1's base
  county at δ 8% and 10%; fodder's rate at 1.3 a year); R1a is the stocks layer off; P7 is M1's
  first-draft default (δ 4%, ω 1, C2); P8 the maker from bought inputs (a 0.005, δ 4%, ω 1).
  κ 52 horse-days a head a year, J_b 1 tick, ρ 0. The coefficients per tick are rule A's at the
  tape's tick length (HORSES-RULES §4).
- **The targets**: 1g's `ChainEconomy` at the coefficients in force (the horse's hours at J = 2,
  decision 232), solved in the harness at genesis and at each dated shock; b's cost targets ×1.1,
  ×0.9, ×2, ×0.5, each moving fodder's land and the build's pasture together; F5's and F6's b ×2
  are not targets (unfunded, provider baskets −0.8026). R1a's targets are P2.1's I0's.
- **The rules and their evaluation orders**: HORSES-RULES §3 and the sources above; the good desk
  P2.0's `GoodDesk` with `assign: ExPost`, the fodder desk P2.1's `TypeDesk`, the households
  P2.1's; R1a's owner desk `assign: Planned` (P2.1's I0), with `ExPost` its nesting variant;
  P7's `ExPost`.
- **Genesis**: HORSES-RULES §4, the tapes above.
- **The dials**: C2g for H1–H4, F1–F6 and P8; `c2g13` for F7–F10; C2 for R1a and P7; the stock
  dials of HORSES-RULES §4 (s_K = 2δ a year, s_Km = 0, the owner desk's s_K = 0, the cover 4
  weeks).
- **The tolerance**: every tol_o is the 1e-3 floor; mode A's bar 1e-9; the live floor 0.5; the
  runaway bound [1e-6, 1e6] × genesis; DEAD's share 1% of W or any tick of F.
- **L per instance and tick length** (HORSES-SPEC §7.4, with the engine's elasticity probe and
  the engine's g; HORSES-RULES §6.5–§6.7):

  | id | L at 52 a year | at 12 | at 24 | at 365 |
  |---|---|---|---|---|
  | H1, H3 | 78,000 | 16,000 | 34,000 | 585,000 |
  | H2, H4 | 75,000 | 15,000 | 33,000 | 561,000 |
  | F1 | 178,000 | | | |
  | F2 | 258,000 | | | |
  | F3, F4 | 77,000 | | | |
  | F5, F6 | 84,000 | | | |
  | F7, F9 | 128,000 | | | |
  | F8, F10 | 75,000 | | | |
  | P7 (M1), P8 (single runs) | 40,000, the frame's (HORSES-SPEC §6.5) | | | |
  | the negative controls (P3) | their instance's L at their tick length | | | |

  A dated shock restarts the clock and runs L after it. Tier 3 and 3S also run at 10·L.
- **The kick** (§7.5): at the end state of each target's mode-A run or dated run, each market's
  posted price × (1 ± 1e-9) through `ScalePrice`, 12 kicks at a wet instance (10 under M1), H = L,
  judged by `criteria/appb-2026-09-26.ron`'s bars (`gain_tail` ≤ 1e-3 over the last tenth,
  `gain_peak` ≤ 1e6). One kick set per (instance, dials, target, tick length).
- **The start distance**: PROBE-SPEC's, but a stock or coin displacement's D̂₀ is the largest D̂
  of its first year (HORSES-SPEC §7.5); R1a keeps P2.0's.
- **The run lists**: the battery (93 runs at A0's instances, 91 at F5 and F6; lists above), the
  `stocks` family (§7.10 item 9), and the families of HORSES-SPEC §7.10 in its priority order.
- **The verdicts** (§7.9): GO, LOCAL and NO-GO per instance and dial set, the answer read from
  H1–H4 at C2g and 52 a year.
- **The prediction**: HORSES-SPEC §6, copied in §6 below, as registered by the frame before any
  engine run.

## 4. The checks made before registration (HORSES-SPEC §7.8)

All in HORSES-RULES §6, from the P2.2.1 build:

- **The nesting (R1a)**: bit for bit against P2.1's I0 over 20,000 ticks and four displaced
  starts of 3,000, and against appb's ex-post variant on the same runs (gate tests); the rule
  level over 3,000 random states.
- **The rest point**: at every instance and funded target, three ticks within 1e-12 in log.
- **The trace diff** (`tracediff.out`): eight of the twelve H1 and H2 runs within 1e-12 on every
  observable; the four others part first on the horse market's cleared volume where the capacity
  desk's order is a small difference of large terms (every volume within 4.0e-13 of its oracle
  volume in absolute terms on H2's JB(0.5) and b ×2), and on the glut runs within the mirror's
  own sensitivity (the mirror against itself one ulp apart parts by 1.3e-9 at H1 and 3.1e-4 at
  H2). Explained, per MARKETS-RULES §6.4's standard; it does not block scoring.
- **The probes**: the elasticity probe gives land's τ 374–389 ticks under C2g, which sets L; the
  open-loop probe shows no quantity loop.
- **Mode A**: PASS at every verdict instance at 52 a year with its base kick set; PASS at 24 and
  365 a year; FAIL at 12 a year everywhere (wet hire unstable, P3 (iii)); the base kick set fails
  at 24 a year at H2 (predicted) and at H4 (predicted stable at 0.99983 a tick: a disagreement,
  reported).
- **The engine's g**: the kick envelopes decay faster than the mirror's PL at every instance
  (0.891 against 0.910 a year at H1), and the mirror's own kick envelope matches the engine's, so
  the mirror's 3·T6 stands in L.
- **The gate** at `d636b76`: `scripts/gate.sh` and `scripts/gui.sh` green in WSL and on Windows;
  865 workspace tests pass on each (3 ignored and run by name), every pinned hash unchanged
  (gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`, the
  branch tapes, the seven markets tapes), the probe's and the markets probe's pins hold, and the
  horses tapes' hash streams are byte-identical across the two machines (HORSES-RULES §6.9).

## 5. What differs from the frame's text, and why

1. **L** (§3 above): the frame's rule with the engine's elasticity probe gives L 75,000–258,000
   ticks at 52 a year, against the 20,000–258,000 the frame quoted from its floor and 3·T6 alone.
   The frame's §6 predictions were made at its shorter L; a longer L can only let a slow run
   finish converging or a slow growth show, and §7.9's classes are read at the registered L.
2. **The engine's g** is read over the kick envelope's whole descent above its rounding floor, not
   over the second half of a 3·T6 horizon, which lies on the floor (HORSES-RULES §6.6).
3. **Names**: `horses` for the frame's `stocks`; the horse-days `traction` (HORSES-RULES §8).
4. **The trace diff's runs** are the frame's six; "Hc x2" and "F x2" are `heads.capacity*2` and
   `finished.maker*2`.
5. **P7's and P8's run length** stays the frame's 40,000 ticks, the length its registered
   readouts are stated at.

---

## 6. HORSES-SPEC §6, the prediction (copied unedited)

HORSES-SPEC.md lines 655–882, as registered on 2026-09-28 before any engine run.

Written on 2026-09-28 before any engine run: none has been made on this branch. Every number is the
frame's mirror's, scored by §7's protocol (`h_run.py`).

### 6.1 The mirror and its checks

`h_mirror.py` is the P2.2a tick map: §2's roles, phase by phase, as GOODS-CHAIN's synthesis mirror
`s_wet.py` writes them, plus what v2a.1 adds: fodder as a traded good made by a fodder desk a tick
before it is eaten; the maker buying fodder for its own horses' days; κ and a horse-day's labour
for generality; M1 on the same markets; and the registered options of the negative controls (the
order on the stock held, the band at 0). It reuses `model.py`'s and `goods.py`'s helpers unedited.
Its checks (`h_check.out`, `h_chaos.out`):

1. **With fodder bought as land the same tick** it is `s_wet.tick` along 15 paths of 3,000 ticks
   (hold, w × 2, heads × 2, b′ = 0.8 and 0.2; δ 10% and 4%; ω ½ and 1): within 4.1e-11 in log on
   14, and 9.0e-6 on the glut at δ 4%, ω 1, where `s_wet` against itself one ulp apart differs by
   1.2e-5. The two differ in evaluation order only.
2. **Its M1 with fodder as land** is `goods.py`'s `own` within 3.0e-14 on five paths; on P7's path
   they part by 0.17 in log, but `goods.py` against itself one ulp apart parts by 0.022 there, and
   all three give P7's statistics to every digit printed: trough 0.3978, 446 dead ticks, goods
   price × 247.48 (GOODS-CHAIN's registered [0.398, 446, × 247]).
3. **R1 in the mirror:** M1 at δ = 1 a tick, ω 0, is model.py's probe map along b′ = 0.8 within
   7.8e-15 (planned) and 4.0e-15 (ex-post) in log over 3,000 ticks.
4. **The rest:** one tick from genesis moves no state by more than 2.2e-15 relative, at 60
   instances and targets under M3 and M1.
5. **Its oracle** (unit 1a on the fold, with the goods' readouts) matches `ChainEconomy` within
   8.9e-16 relative on x\*, v, Y, N_a, a head's price, the horse-day's, fodder's, the heads
   installed, the capacity desk's heads, heads made and fodder made, at eight instances.

Two limits, as P2.1's frame had: the mirror carries no genesis lot's second tick (the trace diff's
mirror adds it, §7.8), and its evaluation orders are not the engine's.

### 6.2 Local rates

PL, the largest mean log growth per tick of a 1e-8 displacement at the point over four random
directions (s_wet's `growth`, T 100 years, burn 20), as a rate per year (`h_lin.out`). The
synthesis mirror's value (fodder bought as land the same tick) is beside it where it exists.

| id | base | b × 1.1 | × 0.9 | × 2 | × 0.5 | synthesis: base, × 2, × 0.5 |
|---|---|---|---|---|---|---|
| **H1** (10%, ω ½) | 0.9097 | 0.9101 | 0.9092 | 0.9111 | 0.9044 | 0.9179, 0.9181, 0.9104 |
| **H2** (10%, ω 1) | 0.8952 | 0.8955 | 0.8949 | 0.8964 | 0.8944 | 0.9013, 0.9050, 0.8968 |
| **H3** (8%, ω ½) | 0.9287 | 0.9289 | 0.9284 | 0.9300 | 0.9247 | 0.9354, 0.9360, 0.9317 |
| **H4** (8%, ω 1) | 0.9162 | 0.9168 | 0.9159 | 0.9171 | 0.9136 | 0.9225, 0.9259, 0.9170 |
| F1 (4%, ω ½) | 0.9850 | 0.9844 | 0.9856 | 0.9805 | 0.9879 | 0.9827, 0.9805, 0.9806 |
| F2 (4%, ω 1) | 0.9821 | 0.9811 | 0.9832 | 0.9777 | 0.9917 | 0.9720, 0.9715, 0.9718 |
| F3 (10%, ω 0.85) | 0.9047 | 0.9051 | 0.9043 | 0.9062 | 0.9010 | — |
| F4 (8%, ω 0.85) | 0.9237 | 0.9248 | 0.9233 | 0.9274 | 0.9200 | — |
| F5 (v1, 8%) | 0.9208 | 0.9209 | 0.9208 | not a target | 0.9219 | — |
| F6 (v1, 10%) | 0.9011 | 0.9012 | 0.9011 | not a target | 0.9012 | — |
| F7–F10 (fodder 1.3) | 0.9104, 0.8957, 0.9299, 0.9211 | | | 0.9118, 0.8972, 0.9316, 0.9182 | 0.9047, 0.8939, 0.9250, 0.9166 | — |
| P8's (a 0.005, 4%, ω 1) | 0.9945 | | | 0.9993 | 0.9782 | 0.9742, 0.9786, 0.9708 |

Every instance and target is locally stable at 52 a year. **The slowest mode is about 1 − δ a
year** (0.894–0.911 at 10%, 0.914–0.930 at 8%, 0.978–0.992 at 4%): capital's own turnover, as
GOODS-CHAIN §3.4 read it. The fodder market moves it by less than 1% at 8% and 10% (slightly
faster), slows it at 4% and ω 1 (0.982 against 0.972), and slows P8's slow maker to near neutral
(0.9993 a year at b × 2, 0.999986 a tick; O44). Fodder's rate at 1.3 changes the local rate by at
most 0.6%.

**Tick length** (per tick; per year in brackets):

| id | 12 a year | 24 | 52 | 365 |
|---|---|---|---|---|
| H1 | 1.0197 (1.26) | 0.99600 (0.908) | 0.99818 (0.910) | 0.99974 (0.910) |
| H2 | 1.0642 (2.11) | **1.00023** (1.006) | 0.99787 (0.895) | 0.99970 (0.895) |
| H3 | 1.0196 (1.26) | 0.99687 (0.928) | 0.99858 (0.929) | 0.99980 (0.929) |
| H4 | 1.0642 (2.11) | 0.99983 (0.996) | 0.99832 (0.916) | 0.99978 (0.922) |
| F1 | 1.0194 (1.26) | 0.99931 (0.984) | 0.99971 (0.985) | 0.99996 (0.985) |
| F2 | 1.0638 (2.10) | 0.99948 (0.988) | 0.99965 (0.982) | 0.99995 (0.982) |

Wet hire is unstable at 12 a year everywhere, and at 24 a year at H2; 52 and 365 agree within 0.6%
a year.

### 6.3 The batteries

`h_battery.out`, `h_summary.out`: every run of §7.7 at the instance's L, Tier 3 and 3S again at
10·L. "Years" are years to tolerance (ticks/52); "dead" counts §7.2's dead ticks, with the ticks on
labour, land or the good alone in brackets; baskets are eaten over Y\*. No run is ERROR, DIVERGED,
DEAD, STUCK or ORBITING anywhere.

| id | L | Tier 1 | Tier 2 | Tier 3 (at 10·L) | Tier 3S (at 10·L) | predicted verdict |
|---|---|---|---|---|---|---|
| **H1** | 24,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| **H2** | 20,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| **H3** | 30,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| **H4** | 25,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | **GO** |
| F1, F2 | 178,000, 258,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | GO |
| F3, F4 | 22,000, 29,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | GO |
| F5, F6 | 27,000, 21,000 | 20/20 | 24/24 | 23/23 (23/23) | 24/24 (24/24) | GO on the funded targets |
| F7–F10 | 20,000–31,000 | 20/20 | 24/24 | 25/25 (25/25) | 24/24 (24/24) | GO |

**Speeds** (years to tolerance, median and slowest; GOODS-CHAIN's nine in `h_table.out`):

| id | Tier 1 | Tier 2 | Tier 3 | Tier 3S |
|---|---|---|---|---|
| H1 | 34, 70 | 52, 83 | 66, 98 | 66, 98 |
| H2 | 37, 63 | 53, 76 | 67, 89 | 66, 91 |
| H3 | 38, 82 | 61, 100 | 79, 118 | 82, 122 |
| H4 | 44, 79 | 65, 95 | 83, 112 | 81, 116 |
| F1 | 53, 145 | 103, 178 | 137, 212 | 144, 235 |
| F2 | 82, 156 | 122, 189 | 158, 220 | 157, 242 |

Against P2.0's I0 (7.9 years median, 9.8 slowest in Tiers 1–2 at 52 a year) and P2.1's I1 (6.8,
10.8), capital makes the economy four to eight times slower at 8–10% and seven to fifteen times at 4%.

**Transients** (median and worst over each tier's runs):

| id | T2 peak D̂ | T2 dead | T2 lowest baskets | T3 peak D̂ | T3 dead | T3 lowest baskets | T3 worst transfer shortfall |
|---|---|---|---|---|---|---|---|
| I0 (P2.0, P2.1) | 346 / 857 | 0 / 12 | 0.77 / 0.54 | 1,290 / 2,321 | 35 / 111 | 0.40 / 0.135 | 770 |
| **H1** | 235 / 555 | 0 / 0 | 0.94 / 0.77 | 1,048 / 3,669 | 0 / 45 (22) | 0.67 / 0.444 | 20.6 |
| **H2** | 417 / 1,258 | 0 / 0 | 0.93 / 0.76 | 1,340 / 7,757 | 0 / 119 (20) | 0.68 / 0.444 | 28.7 |
| **H3** | 236 / 559 | 0 / 0 | 0.94 / 0.77 | 1,043 / 3,722 | 0 / 43 (22) | 0.67 / 0.444 | 20.6 |
| **H4** | 423 / 1,283 | 0 / 0 | 0.93 / 0.76 | 1,343 / 8,203 | 0 / 86 (19) | 0.68 / 0.444 | 28.6 |
| F1 | 238 / 567 | 0 / 0 | 0.94 / 0.77 | 1,032 / 3,812 | 0 / 42 (23) | 0.67 / 0.444 | 20.6 |
| F2 | 436 / 1,330 | 0 / 0 | 0.93 / 0.77 | 1,355 / 10,322 | 0 / 86 (19) | 0.68 / 0.444 | 28.5 |
| F5 (v1) | 326 / 754 | 0 / 0 | 0.92 / 0.67 | 1,096 / 4,749 | 0 / 253 (24) | 0.64 / 0.323 | 157 |
| F7 (fodder 1.3) | 277 / 716 | 0 / 0 | 0.94 / 0.74 | 1,156 / 4,579 | 0 / 98 (88) | 0.66 / 0.366 | 146 |

Peak D̂ leaves out the horse market's volume, which is zero while orders stop. The worst Tier-3
baskets are N(2)'s 0.444 in every A0 instance, as in GOODS-CHAIN's battery. The worst dead ticks are
`r*2` (H2: 119, on every one of which fodder clears below half its volume, and on 20 the good) and JB(0.5).

**Stocks soften O14's medians and troughs, and cost time.** Tier 3's median trough of baskets is
0.67–0.68 against I0's 0.40, its worst 0.444 against 0.135, and its worst dead ticks on labour, land
and the good 19–23 against 111. Tier 2 has no dead tick. Peaks of D̂ are of I0's size.

### 6.4 Paths: O14, the stock's time, the glut

From the battery's cost shocks at genesis and its heads × 2 (`h_table.out`; the dated shocks give
the same numbers). b × 2 is the registered demand fall for horses: the horse-days the good desk
wants fall to 0.723 of their old volume, so the installed stock is a 38% glut. "5%" is the time until the installed heads stay within 5% of the target; the
paper's time is one tick for an expansion and ln(K′/K)/ln(1 − δ) of zero builds for a contraction.

**b × 2 (b′ = 0.8: costs rise, output −12%, the stock's target × 0.723):**

| id | goods trough, peak | dead (on) | years with no order | p max | p_K range | heads, lowest | 5% (paper) | tol |
|---|---|---|---|---|---|---|---|---|
| H1 | 0.646, 1.14 | 11 (fodder) | 0.1 | 1.15 | 0.21–2.3 | 0.880 | 29.1 y (3.08) | 89 y |
| H2 | 0.625, 1.14 | 7 (fodder) | 0.9 | 1.22 | 0.014–2.3 | 0.824 | 39.8 y (3.08) | 81 y |
| H3 | 0.646, 1.14 | 11 (fodder) | 0.1 | 1.14 | 0.20–2.3 | 0.897 | 33.0 y (3.89) | 105 y |
| H4 | 0.625, 1.14 | 7 (fodder) | 1.0 | 1.24 | 0.013–2.1 | 0.839 | 47.2 y (3.89) | 101 y |
| F1, F2 | 0.647, 0.626 | 11, 7 | 0.1, 2.7 | 1.10, 1.26 | | | 39.9, 77.6 y (7.95) | 181, 196 y |
| F7, F8 (fodder 1.3) | 0.426, 0.481 | 83, 61 | 0.3, 0.8 | 1.39, 1.53 | | | 26.7, 37.6 y (3.08) | 86, 79 y |

**The fodder market deepens the cost rise's dip.** GOODS-CHAIN's mirror, with fodder as land bought
the same tick, bottomed at 0.899 (ω ½) and 0.829 (ω 1) after b′ = 0.8 at δ 10% (P1). Here the fodder
desk's cost doubles at once, its cash rule halves what it makes, and a tick later the horses are
short of fodder: goods bottom at 0.625–0.646, with 7–11 ticks on which fodder clears below half its
volume. With fodder's price at 1.3 a year the dip reaches 0.43–0.48. Against old Y the trough is
0.57 (H1): ln −0.565 for an equilibrium change of −0.128, where I0 fell to 0.135 (ln −2.00) and I1
to 0.043.

**b × 0.5 (b′ = 0.2: costs fall, output +10%, the target × 1.258):**

| id | goods trough, peak | dead | p max | p_K max | heads, highest | 5% (paper) | quasi-rent at 3 ticks, at 1/δ | tol |
|---|---|---|---|---|---|---|---|---|
| H1 | 0.744, 1.15 | 0 | 2.50 | 4.2 | 1.39 | 37.9 y (1 tick) | +0.004, +0.415 | 85 y |
| H2 | 0.726, 1.37 | 0 | 2.96 | 9.6 | 1.80 | 34.3 y | +0.062, +0.544 | 82 y |
| H3 | 0.747, 1.14 | 0 | 2.56 | 4.2 | 1.39 | 43.5 y | +0.004, +0.420 | 104 y |
| H4 | 0.724, 1.37 | 0 | 3.14 | 9.8 | 1.83 | 41.2 y | +0.062, +0.576 | 104 y |
| F2 (4%, ω 1) | 0.709, 1.40 | 0 | 4.30 | 11.1 | 2.02 | 93.6 y | +0.062, +0.752 | 218 y |

**Capital's time scale (D-G14), with the fodder market.** After a contraction the stock takes 29–48
years to come within 5% at 8–10% (8.5 to 12.9 times the paper's 3.1–3.9 years of zero builds), and
40–78 at 4%. After an expansion the paper fills the gap in one tick; here it takes 34–44 years, and
the stock **overshoots** its new target by 39% (ω ½) to 83% (ω 1) on the way. The quasi-rent is
within 5% of zero three ticks after the expansion at ω ½ (+0.4%) but not at ω 1 (+6.2%), and
at 1/δ (9.5 to 12 years on) the hour price is still 42–58% above its full cost at posted prices.
Neither of D-G14's lifting criteria is met: the departure stands, as registered.

**The glut (the capacity desk's heads × 2):**

| id | goods trough | years with no order | p_K lowest | hour price lowest, over O | finished heads, highest | utilisation, lowest | 5% | tol |
|---|---|---|---|---|---|---|---|---|
| H1 | 0.857 | 0.1 | 0.54 | 0.41, 1.08 | 1.25 | 0.59 | 53 y | 98 y |
| H2 | 0.762 | 2.2 | 0.040 | 0.63, 0.98 | 2.27 | 0.56 | 45 y | 91 y |
| H3 | 0.846 | 0.1 | 0.53 | 0.39, 1.04 | 1.32 | 0.59 | 64 y | 122 y |
| H4 | 0.751 | 3.2 | 0.036 | 0.63, 0.98 | 2.30 | 0.56 | 55 y | 116 y |
| F1 (4%, ω ½) | 0.821 | 3.9 | 0.51 | 0.33, 0.96 | 1.51 | 0.59 | 112 y | 235 y |
| F2 (4%, ω 1) | 0.718 | 8.8 | 0.026 | 0.63, 0.91 | 2.57 | 0.56 | 117 y | 242 y |

In a glut the hour price falls to about the running cost O (0.91–1.08 of it at its lowest):
installed horses compete at their operating cost, as PLAN §3.1 asks. Orders stop for up to 3.2 years
at 8–10% and 8.8 at 4%; the maker's price falls to 0.03–0.5 of its target and its finished stock
piles up to 2.6 times its rest value, but no A0 instance reaches the runaway bound. One new dead
state: at 4% and ω ½ (F1), where fodder is cheap and horses wear slowly, labour clears below half
its volume for 550 ticks (10.6 years) while the doubled stock does the tasks. AGENTS-GOODS found this
technological unemployment under M1 and GOODS-CHAIN said wet hire removed it; the cheap running cost
brings it back. The run still converges.

### 6.5 GOODS-CHAIN's P1–P8 for v2a.1

Each registered prediction, its value in GOODS-CHAIN's mirror, and what this frame registers for
P2.2a's engine run.

| # | as GOODS-CHAIN registered | this frame registers (the mirror's value) |
|---|---|---|
| **P1** (H1, H2) | mode A holds; kicks decay 0.90–0.93 a year [0.918, 0.901]; 79/79, tier-3 goods ≥ 0.44, ≤ 11 dead ticks; b′ 0.8 trough ≥ 0.8 [0.899, 0.829]; b′ 0.2 ≥ 0.7 [0.812, 0.731], no dead tick | mode A PASS; kicks decay at **0.89–0.92** a year [0.910, 0.895]; **GO**, 93/93 with Tier 3 and 3S at 10·L; tier-3 baskets ≥ 0.44 [0.444]; dead ticks ≤ **45 (H1), 119 (H2)**, ≤ 22 on labour, land and the good; b′ 0.8 trough **≥ 0.6** [0.646, 0.625], with **≤ 12 dead ticks on fodder**; b′ 0.2 ≥ 0.7 [0.744, 0.726], no dead tick |
| P1 on H3, H4 | — | the same, kicks **0.91–0.94** [0.929, 0.916]; b′ 0.8 [0.646, 0.625]; b′ 0.2 [0.747, 0.724] |
| **P2** (F1, F2) | 79/79; no dead tick after b′ 0.2 [0.811, 0.712]; kicks 0.97–0.99 [0.983, 0.972] | GO, 93/93; no dead tick after b′ 0.2 [0.753, 0.709]; kicks **0.975–0.995** [0.985, 0.982] |
| **P3 (ii)** | wet M3 at s_K 0, C2, ω 1 fails [1.0003–1.0013 a tick] | fails: **1.0014–1.0017** (δ 10%) and 1.0008–1.0010 (δ 4%) a tick; from w × 1.05 ORBITING; a 1e-9 kick grows to a peak D̂ of 590–700 |
| **P3 (iii)** | wet M3 at 12 a year fails [1.01–2.36] | fails: **1.020** (ω ½) and **1.064** (ω 1) a tick; a 1e-9 kick ends DEAD or DIVERGED; also **24 a year fails at H2** (1.00023) |
| **P3 (iv)** | CHAIN's A3 as written [a runaway] | fails: 1.012–1.023 a tick; p_K reaches the runaway bound at ticks 1,823–3,456 after b′ 0.8 or a 1e-9 kick |
| P3 (v) | a storable good offered in full [1.016] | not applicable (no storable consumer good in v2a.1). **On the horse, the maker's finished heads offered in full (b_K 0) are stable under M3** (0.894–0.992 a year at δ 10% and 4%, both ω, every target; heads × 2, b × 2 and × 0.5 converge at δ 10% and 8%): D-G5's unit root holds under M1 at s_K 0, not under the capacity desk at s_K = 2δ |
| P3 (i), (vi) | M1's split margins; hire from cash flow | carried: neither is built |
| P4 | July's band on the good | carried: no storable consumer good (v2a.6) |
| **P5** | 52 and 365 agree within 2% [1.3%] | agree within 2% [0.6%, at H4] |
| P6 | interest | carried: ρ = 0 (D-G15) |
| **P7** (M1 + M2, C2, 4%, ω 1, b′ 0.2) | trough ≈ 0.40 [0.398]; ≥ 300 dead [446]; goods price > × 100 [× 247] | trough ≈ 0.43 [0.425]; ≥ 250 dead, all on the good [322]; goods price > × 100 [× 140]; p_K × 434; STUCK at 40,000 ticks |
| **P8** (a 0.005, 4%, ω 1, heads × 2) | p_K < 1e-10 of target within 6 years [4.8]; goods, hours, wage, land within 5% | p_K < 1e-10 of target within 6 years [4.4]; DIVERGED by the runaway bound at tick 138; until then goods within 4% [3.5%], land [0.03%], the hour price, the good's price and the wage within **7%** [5.7%, 5.9%, 6.9%] |

**What changes against GOODS-CHAIN, and why.** Every verdict and every local rate holds. What the
fodder market changes is the contraction's dip (0.63–0.65 against 0.83–0.90) and the dead ticks,
now counted on fodder and horse-days too; the kicks are a little faster at 8–10%. None of it is a
tuning: GOODS-CHAIN's mirror had no material market, which its §3.6 lists among what it could not
settle.

### 6.6 What the mirror cannot settle

- **The genesis carry and evaluation orders** (§6.1, §7.8): the trace diff decides.
- **Kicks at the rounding floor.** PL is a random-direction estimate; MARKETS §2 found it can miss
  slow cones (I3 at 12 a year). The engine's kick sets are the test.
- **Tier 3's stock displacements beyond × 2**, the coins × 0.02 (family 9), and basins.
- **The idle runaway's remedy** (GOODS-CHAIN open question 4): P8 is registered to fail.
- **Loops, interest, lags, entry**, as §0 says.

## 7. HORSES-SPEC §7, the measurement protocol (copied unedited)

HORSES-SPEC.md lines 884–1068.

MARKETS-SPEC §7 holds except where this section says otherwise. §6's predictions were made with
this protocol in the mirror (`h_run.py`), so the engine's run and the prediction are scored alike.

### 7.1 Observables

For each tick, from the `TickReport` and `Sim::actor_state` (agents see none of them), 18
observables, the set O:

- **Relative prices (5):** v = w/r, and p_f, p_K, p_h and p over r.
- **Technique (1):** s, the share the good desk's production used (ex-post: 1 − x_u).
- **Cleared volumes (6):** labour, land, fodder, horses, horse-days, the good.
- **Outputs (4):** the good desk's y, the capacity desk's horse-days z, the maker's heads y_b, the
  fodder desk's fodder.
- **Stocks (2):** the capacity desk's heads at the tick's start (its `held`), and the maker's
  serving stock (its `serving`).

Also recorded, not in O: S, D and both fills per market; `RationLine`s; spoilage per good; every
coin; the provider's due and paid; baskets eaten and the binding item; the ledger margin; the
maker's finished stock and own record; the capacity desk's K\*, order and utilisation z/(κ·held);
the horses bought (investment); the running cost O and the full cost O + δ·p_K/κ at posted prices.

GOODS-CHAIN's mirror scored nine (v, p_h, p, 1 − x, cleared labour, land and goods, output and the
total stock). Every table in §6 gives ticks to tolerance on both sets.

### 7.2 Numeraire, liveness, idleness

r is the numeraire. **A tick is dead** if labour, land, fodder, horse-days or the good fails to
trade or clears less than ℓ = 0.5 of its oracle volume at the target in force. **The horse market
is judged idle, not dead** (GOODS-CHAIN §4, D-G11): its volume is a replacement flow, δ of the
stock, and in a glut the rule stops orders by design. A tick is idle if it clears less than 0.5 of
its oracle volume; idle ticks and ticks with no order are reported (§7.11), never scored as dead.
Its price still counts for the runaway bound and its volume for D̂.

### 7.3 Distance and tolerance

e_o(t) = |ln(o_t/o\*)|, D̂_t = max_o e_o(t)/1e-3. No dial is a band, so every tol_o is the 1e-3
floor.

### 7.4 Windows and run length

W is the second half of a run and F its last tenth (PROBE-SPEC §4.4). **L**, per instance and tick
length, is the largest of:

- 20,000·tpy/52, rounded up to 1,000 (the probes' floor);
- 200·τ_max from the engine's one-tick elasticity probe (MARKETS-SPEC §7.8);
- **3·T6**, rounded up to 1,000, where T6 = ln(10⁶)/(−ln g) is the ticks in which the slowest local
  mode falls by 10⁶, g the largest per-tick growth over the instance's base and four cost targets.

The first two are P2.1's rule. The third is D-G11's "windows from the chosen rule's measured
slowest mode": the elasticity rule cannot see a stock's mode, which is slower than any market's.
g is the mirror's (§6.2) until registration, when the engine's is measured: the base kick set run
for H₀ = 3·T6 (mirror), g fitted as the decay of its largest gain over the second half. If the
engine's g gives a longer L, that L is registered. The mirror's L: H1 24,000; H2 20,000; H3 30,000;
H4 25,000; F1 178,000; F2 258,000; F3 22,000; F4 29,000; F5 27,000; F6 21,000; F7–F10 20,000–31,000.
A dated shock restarts the clock and runs L after it. Tier 3 and 3S also run at 10·L.

### 7.5 Classes, with the kick

PROBE-SPEC's classes in order (ERROR, DIVERGED, DEAD, VACUOUS, CONVERGED, STUCK, ORBITING), the
runaway bound [1e-6, 1e6] × genesis on every posted price, and MARKETS-SPEC §7.5's kick: CONVERGED
stands only if its target's kick set decays. One kick set per (instance, dials, target, tick
length): at the end state of the target's mode-A run or dated run, each market's price × (1 ± 1e-9)
through `ScalePrice`, 12 kicks, each for H = L, judged by `criteria/appb-2026-09-26.ron`'s bars
(`gain_tail` ≤ 1e-3 over the last tenth, `gain_peak` ≤ 1e6).

**One change, for stock and coin displacements (Tier 3S):** their start distance D̂₀ is the largest
D̂ of their first year, not of tick 0. A stock or coin moves no observable until it is traded or
used: doubling the finished horses, the fodder held or a desk's coin leaves every tick-0 observable
unchanged, and the first run of the mirror classed those runs VACUOUS although they move the
economy for decades. The vacuity test and κ = max_F D̂/D̂₀ read that D̂₀.

### 7.6 Mode A

Genesis at the oracle's f64 point with §5.3's coins and stocks, L ticks: PASS if every e_o ≤ 1e-9,
every market live (the horse market trading at every tick too), every fill ≥ 1 − 1e-9, spoilage
≤ 1e-9 of volume, the ledger clean; and the base kick set passes. Also at 12, 24 and 365 ticks a
year, reported, with the generator's restated δ, build coefficients and cover.

### 7.7 Mode B: the battery

Stocks and coins at the point unless displaced. Factors f ∈ {1.05, 0.95} (Tier 1), {1.2, 0.8}
(Tier 2), {2, 0.5} (Tier 3). The run grammar is P2.1's (MARKETS-RULES §5) with the markets named:

| family | runs | what is displaced |
|---|---|---|
| singles | 6 each for w, r, p_f, p_K, p_h, p, s | one price, or the good desk's human share |
| JA(f) | 6 | w, p, p_h, p_f, p_K × f, r fixed; s × f |
| JB(f) | 6 | w × f, p × f, the machine side's p_h, p_f, p_K × 1/f; s × 1/f (the technique target pushed by f²) |
| N(f) | 6 | every price × f, coins unchanged (nominal) |
| x\*/2 | 1 | the good desk's x halved |
| cost shocks | 8 | b × 1.1, 0.9 (Tier 2), × 2, 0.5 (Tier 3), each at genesis and dated at L/4: fodder's land and the build's pasture together |
| **stocks and coins (Tier 3S)** | 24 | × 0.5 and × 2 of: the capacity desk's heads, its horse-days in hand, the maker's own and its finished heads, the fodder desk's fodder, the good desk's good, and the coin of the good desk, capacity desk, maker, fodder desk, workers and provider |

| id | Tier 1 | Tier 2 | Tier 3 | Tier 3S | total |
|---|---|---|---|---|---|
| H1–H4, F1–F4, F7–F10 | 20 | 24 | 25 | 24 | 93 |
| F5, F6 | 20 | 24 | 23 (b × 2 not a target) | 24 | 91 |

Tier 3S is in the verdict: the horse stock and the coins are displaced in the battery, since a
stock's restoring force is the new structure under test. There are no slack runs (one category, one
segment).

### 7.8 First: the probes and the trace diff

Before registration, per verdict instance:

- **The one-tick elasticity probe** (every market), for τ and L's first term.
- **The open-loop probe** (prices frozen, lags 1, 5, 20, 200), which should show no quantity loop:
  there is none in v2a.1.
- **The trace diff.** `h_mirror.py`'s map with the engine's genesis carry added (a Ticks(1)
  genesis lot sells at ticks 0 and 1, and a buyer's unused purchase of it lives to tick 1, as P2.1's
  `mm_carry.py` added to mm.py), against the build, for 2,000 ticks on H1 and H2 from hold, `w*2`,
  `JB(0.5)`, `b x2 genesis`, `Hc x2` and `F x2`. It must agree to 1e-12 in log on every observable,
  or the disagreement is explained in the registration before scoring (MARKETS-RULES §6.4's
  standard). The mirror's evaluation orders are not the engine's, so a chaotic stretch can part
  them at rounding (§6.1), which the registration then shows against the mirror one ulp from
  itself, as P2.1's did.
- **The nesting checks (R1a)** and the rest-point test, as gate tests (§8).

### 7.9 Verdicts

Per instance and dial set:

- **GO:** mode A PASS with its base kick set, and every non-vacuous run in Tiers 1, 2, 3 and 3S
  CONVERGED with its target's kick set, with Tier 3 and 3S also CONVERGED at 10·L.
- **LOCAL:** mode A PASS, Tiers 1–2 CONVERGED, some Tier 3 or 3S run not.
- **NO-GO:** otherwise.

**The answer** is read from H1–H4 at C2g, 52 a year: "GO for v2a.1" means all four GO. Families are
reported beside it. A verdict that disagrees with §6 is reported as such; a disagreement the trace
diff did not catch is a finding about the mirror.

### 7.10 Families, reported beside the verdicts

In priority order; the run stops where the time box ends and says where.

1. Tier 3 and 3S at 10·L (part of GO).
2. **Tick length:** Tiers 1–2 and mode A at 12, 24 and 365 a year, on H1–H4 (P3's and P5's).
3. **δ 4% (F1, F2):** the full battery (P2).
4. **The chain's ω (F3, F4)** and **v1's base county (F5, F6):** the full battery.
5. **Fodder's rate at 1.3 (F7–F10):** the full battery.
6. **The negative controls (P3):** wet M3 at s_K 0 under C2 at ω 1; the order on the stock held
   (CHAIN's A3 as written, s_K 0); 12 a year; the maker's finished heads offered in full (b_K 0), on
   H1–H4.
7. **The owner-operator (M1):** R1a's two nesting checks; the first draft's default (P7); its local
   rate at C2 and C2g.
8. **The idle machine market (P8):** a = 0.005, ω 1, δ 4%, heads × 2.
9. **Stocks beyond the battery** (O22's family, stocks first): every coin × {0.02, 0.1}, every stock
   × {0.1, 10}.

Determinism: one repeat per instance with identical hashes in WSL; Windows is a recorded check.

### 7.11 Transient and stock statistics (O14, D-G14)

Reported for every run, never scored. P2.1's §7.11 set (peak D̂ and ticks to tolerance, dead ticks
per market, troughs of cleared volume and output, baskets eaten and the binding item, rationing,
spoilage, the transfer shortfall, the path depth for a cost shock), plus:

| statistic | definition |
|---|---|
| peak D̂ without the horse market | the horse market's volume is zero whenever orders stop, which makes the plain peak ∞ |
| the stock's time to 5% | the first tick from which the installed heads (the capacity desk's and the maker's serving) stay within 5% of their target, against the paper's time: 1 tick (J_b) for an expansion, ln(K′/K)/ln(1 − δ) ticks of zero builds for a contraction (GOODS-CHAIN §3.4) |
| the stock's range | lowest and highest installed heads over the target |
| D-G14's readouts | (i) the time to 5% over the paper's time; (ii) the quasi-rent, p_h/(O + δ·p_K/κ) − 1 at posted prices, at 3·J_b and at 1/δ ticks after the shock |
| the glut, after heads × 2 and after b × 2 (the demand fall) | ticks with no horse order and idle ticks; the lowest p_K over its target and whether it reaches the runaway bound (P8's idle runaway); the highest finished stock over its rest value; the lowest utilisation z/(κ·held); the lowest hour price over the running cost O (installed horses competing at their operating cost) |
| investment | horses bought: trough and peak over δ·K\* |
| the nine | ticks to tolerance on GOODS-CHAIN's nine observables |

**The O14 comparison.** Per instance and tier, the median and worst of each statistic beside P2.0's
I0 (REPORT §3: Tier 2 peak D̂ 857, 12 dead, lowest good 0.54; Tier 3 2,320, 111, 0.135; b′ = 0.8
cutting output 85% on the way to a 12% fall) and P2.1's (MARKETS §4). Two questions are asked of the
table: **do stocks soften O14** (compare H1–H4's b × 2 with I0's and I1's machine-land shocks of
similar size), and **how long does capital take** (the D-G14 readouts against the paper's times).

### 7.12 Frozen before any mode-B run

`D:/rustyecon-p2g/run/registration.md` carries the worktree commit, its dirty flag, and sha256 of
this file, the roles' and harness's source, the tapes, and `model/h_*.py` with their outputs. It
fixes, before the first mode-B run: the instances, coefficients and targets (§1); the rules and
their evaluation orders, genesis coins and stocks (§2, §5.3); the dials (§5); the tolerance, L per
instance and tick length (§7.4, with the engine's g); the run lists (§7.7); and §6's prediction.
The probes, the trace diff, mode A and the nesting checks may run before it. Any later change is a
new dated registration, and earlier results stay reported.

