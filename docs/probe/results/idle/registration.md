# L0 registration: the maker's reservation on the engine (IDLE-SPEC §8)

Dated 2026-09-29. Step L0.3 on branch `phase2-loops`, written on top of `ed8fc6c` (L0.2), clean.
It is written **before the rule exists in the engine**, so before any engine run with it on:
no build, trace diff or scored run of the remedy has been made. Any later change is a new dated
registration, and earlier results stay reported.

## 1. What is frozen

- **The spec.** `D:/rustyecon-p2l/idle-scan/IDLE-SPEC.md`, sha256
  `c5c6527a7226ed462b0fe321d92e86339ed286e02ef880fc0f11da2fd373634e`, with every file of its
  mirror scan listed in `SHA256SUMS` beside it (sha256 of that list
  `a888d03bcbf87d2497f360bd957f1a0a04bd87453cedafd1aed3751d42cabde4`; all 118 entries verified on
  2026-09-29). Its §8, the predictions, is copied unedited in §6 below. Its §6 is the build, as
  written, with the one reading in §2.
- **The base.** The stocks probe as registered at P2.2.2
  (`docs/probe/results/horses/registration.md`, sha256
  `2baaa34d93fa1ba154abf11c1d07521a53493f16ea7e0395a3c8acf2c75af4c6`), as amended at L0.1 (two
  load checks; no rule or state changed, and the six horses tapes give P2.2's hash streams). It
  fixes everything this registration does not: the instances, the targets, the dials, the
  tolerances, L per instance and tick length, the kick, the classes, the start distance and the
  run lists (`docs/probe/results/horses/lists/`, the 28 files together sha256
  `b9bd7b328b662810f1d1e7c11f9b8cf33ea488b41aa2cab1988d6992370b438d`).
- **The reference runs.** "P2.2a's run" means the engine's run recorded under
  `D:/rustyecon-p2g/runs/` (P2.2.3, the harness of `d636b76`), read-only. L0.1 changed no rule,
  so these are the base's runs.
- **The sources at the base** (sha256), which the build will change:

```
b6cea2c91d5fdeeefc32fc9d6b732b33c718cc94fcfe3bcbb32dbd64f21ccc7f  docs/probe/HORSES-RULES.md
9e5688594ab322a07734e80ed9c10d41432aeb7fbe707deedfb660fb24dde44c  crates/agents/src/roles/stock/rules.rs
64efe789e82f7f8ae789dac4d928d8b7c64209a1ea55d516b66ae8efb13e7fe8  crates/agents/src/roles/stock/spec.rs
18dd66d08cefa4f6829762fcb4341d3a4ee993d62ef1a8da0add8d8622714183  crates/agents/src/cast.rs
fef3952ca6fbf318b9475ee2667e1194fbe83672d4f864ae121105df3a5d5809  crates/probe/src/horses/cli.rs
e90d08e439e815a498dade258f543ed96ea40abb84a5ae2e0b912d5af347cdd5  crates/probe/src/horses/harness.rs
3ca4a10a8fffb36a0960a60c78334cb491405675ce843960c9d53b6ad73de847  crates/probe/src/horses/setup.rs
673215e3aefe99b9c234f7f1282eba7dfd045e98bdd9cf7059b3f916aaee68a0  crates/probe/src/bin/horses.rs
be4fe26a97e94969531500307fa77deca43da0b81e6168be3ad8c57b8d4c7414  crates/probe/src/bin/horses-tape.rs
```

  The six committed horses tapes keep P2.2a's sha256 (`a9897fd0…` H1, `92b28e52…` H2,
  `6d5b0475…` H3, `0c8c73be…` H4, `d08d21a8…` P7, `cef45808…` R1a); the build must not move them.
- **The design sources**, read-only: `D:/rustyecon-goods/GOODS-CHAIN.md` (sha256
  `d4be4b2d57bea59cb318920fdae20af13b978020f734d45607fa363d82a65ace`).

## 2. The build to come (L0.4), and its one reading

The next commit builds IDLE-SPEC §6 as written: the optional field `reserve` on `RawMaker` and
`Maker`, the one changed line in `MakerRole::decide`, the two load checks, `--reserve PSI` on
`horses` and `horses-tape`, and the five tests, each failing without the change it guards. The
results name that commit; this file does not change when it lands.

- **The param's key** is `reserve.<maker>`, as `cover.<maker>` is: `reserve.maker` at every stock
  instance. The spec's parenthesis "(at A0 `reserve.desk.maker`)" is read as a slip, since no
  other maker param carries the actor's `desk.` prefix but the cash rule's, which the spec does
  not follow. Unit `Dimensionless`, value 0.25, basis `Assumed("IDLE-SPEC 2026-09-29, mirror
  scan")`, written after `cover.<maker>` among the dials.
- **The harness's readouts** (spec §6, "The harness") are computed from the tick's posted prices
  (those the settlement used, which `decide` read) and the coefficients in force, in the rule's
  own evaluation order, over the scored ticks, as the mirror's `i_run.py` counts them. The three
  columns `withheld`, `switches` and `markup_low` are appended after P2.2a's last column, so P2.2a's
  columns keep their places; a run's CSV gains `markup` and `withheld` at the end of each row.
- **The numbering.** The spec's §9 numbers its decisions 240–245 and its open items O51–O54. L0.1
  took 240–244 and O51 first. STATE.md gives them the next free numbers when it records them,
  and says which is which.

## 3. What must hold before any scored run

1. The five tests of spec §6 pass, and each fails with its change undone (checked by hand and
   recorded).
2. `scripts/gate.sh` and `scripts/gui.sh` are green on WSL and on Windows. The gate's
   `0x61f9c8529131ff17`, appb's `0xe1fa082b26995867`, demo-gb's `0xfad880fe08d06645` (stream
   `0xdb63cc96f769fb3e`), the probe's and the markets probe's pins, and every committed tape's
   canonical text, `tape_hash` and `world_id` are unchanged.
3. **E1's hashes.** The cli's per-tick hashes of the six horses tapes over 2,000 ticks equal
   P2.2's (`D:/rustyecon-p2g/report/hashes/`).
4. **The trace diff** (spec §6): the scan's mirror `i_mirror.py`, with P2.2a's genesis carry added
   the way `make_h_carry.py` added it to `h_mirror.py`, against the engine for 2,000 ticks at
   ψ 0.25 on H2 `heads.capacity*10`, P8 `heads.capacity*2` and H2 `r*2`. It passes when every
   observable agrees within 1e-12 in log, or where it does not, the parting is on the horse
   market's volume at the order's cancellation and within the mirror's own one-ulp sensitivity,
   as HORSES-RULES §6.4 explains it for P2.2a. If it fails, the build is fixed and no run is
   scored; the rule and the predictions stay.

## 4. The protocol

Release build on WSL, 52 ticks a year unless said, every run with `--reserve 0.25` unless said,
at P2.2a's L per instance (78,000 at H1 and H3; 75,000 at H2, H4, F8 and F10; 178,000 at F1;
258,000 at F2; 77,000 at F3 and F4; 84,000 at F5 and F6; 128,000 at F7 and F9) and with
P2.2a's `--every`, so each CSV row can be set beside P2.2a's. Raw outputs under
`D:/rustyecon-p2l/idle-engine/runs/`.

| item | runs | length |
|---|---|---|
| E1 | ψ 0 (`--reserve 0`) at H1–H4 `hold`, H2 `heads.capacity*10` and P8 `heads.capacity*2`, against P2.2a's rows and against the same runs without the flag | 2,000 ticks (the test) and P2.2a's L |
| E2 | mode A (`hold`) at H1–H4 and F1–F10; `horses slowest hold` at the 14; the kick set at `hold` and at every funded dated cost target at the 14 | L; horizon L |
| E3 | `heads.capacity*10` at the 14 | L |
| E4 | P8 `hold` and `heads.capacity*2` at 40,000 and at 400,000 ticks; its kick set and slowest mode at `hold` | 40,000 and 400,000 |
| E5 | the battery (Tiers 1, 2, 3 and 3S, P2.2a's lists) at the 14; Tiers 3 and 3S again at 10·L | L and 10·L |
| E6 | the `stocks` family at the 14 | L |
| E7 | Tiers 1–2 at 12, 24 and 365 a year at H1–H4 (P2.2a's lists, the dated and genesis b shocks included); P7 `hold` and `b*0.5@genesis` at C2 and C2g; P3 (ii), (iv) and (v) as P2.2a ran them, with their `w*1.000000001` kick runs | P2.2a's lengths |
| E8 | ψ 1 (`--reserve 1`): `hold`, `p[horse]*0.999999999` and `p[horse]*1.000000001` at H1–H4 at the lengths the mirror ran (24,000, 20,000, 30,000, 25,000), and at F1 and F2 at L; ψ 0.75: P8 `heads.capacity*2` at 40,000 and 400,000 | as said |

**How each prediction is read** (spec §8's tolerances):
- a class is the harness's, exactly; ticks and years to tolerance within 10% of the mirror's
  (three runs per instance within 25%); the horse price's lowest over its target within 5%;
- "acts" means `withheld` > 0; "unchanged" means P2.2a's run with every shared column of every
  CSV row equal as text (full precision), and the same summary and statistics;
- the 84 acting runs are compared by name with `i_cmp_025.out`'s list, the mirror's names mapped
  to the engine's as P2.2a's `analyze.py` maps them;
- E3's and E4's dead ticks, highs and withheld counts are reported beside the mirror's, not
  scored: the spec gives them as ranges from the mirror, and the trace diff bounds how far the
  engine can differ on a glut path.

**What would refute the choice** is spec §8's list, below. Any of it sends O47 back to the scan
with the alternative (`Hold`). A miss outside the tolerances that is not on that list is
reported as a miss, with its cause, and the choice stands.

## 5. Runs read before registration, disclosed

None on the engine. The spec's numbers are the mirror's (`i_*.out`), read before this was
written; the scan made no engine run (spec §0).

## 6. The prediction: IDLE-SPEC §8, copied unedited

Its section numbers are the spec's: §0 the choice, §6 the build, §7 the proof. "HORSES §2" is
docs/probe/HORSES.md.

The engine runs follow the build (§6) and its trace diff. Each prediction is registered before
any engine run with the rule on. The conditions are those of §0, with ψ 0.25.

The tolerances follow how P2.2a's engine matched its mirror (HORSES §2):
- every class exactly;
- ticks and years within 10%, allowing three runs per instance within 25%;
- the horse price's lowest over its target within 5% of the value given.

"Unchanged" means the engine's own P2.2a result, bit for bit, wherever the maker's markup never
falls below ψ.

**E1. Nesting.**
- With `reserve` absent, every committed tape keeps its canonical text, `tape_hash`, `world_id`
  and per-tick hash stream, and every pin in §6 holds.
- With ψ = 0, H1–H4, H2's heads × 10 and P8 are P2.2a's runs, value for value, for 2,000 ticks.

**E2. Mode A and the local map.**
- At H1–H4 and F1–F10, mode A with ψ 0.25 is P2.2a's bit for bit: the largest gap, and the base
  kick set's tail.
- `horses slowest` gives each instance's P2.2a envelope and g bit for bit.
- The mirror's largest root per tick, which is unchanged, is at the base target:

  | H1 | H2 | H3 | H4 | F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9 | F10 | P8 |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
  | 0.998182 | 0.997874 | 0.998579 | 0.998318 | 0.999709 | 0.999652 | 0.998076 | 0.998475 | 0.998415 | 0.998000 | 0.998196 | 0.997884 | 0.998602 | 0.998421 | 0.999894 |

  The largest over every target is 0.999986 a tick (P8 at b × 2).

**E3. heads.capacity × 10 at the 14 instances** (`i_x10.out`).
- **No runaway** anywhere: the horse price stays within [1e-6, 1e6] of genesis.
- Class **CONVERGED** at L at every instance, where P2.2a had DIVERGED.
- Dead ticks are on the horse-days market at every instance, and on labour as well at the ω ½
  ones (H1, H3, F1, F7, F9).
- The post-glut horse price's highest over target is 8–112.
- The withheld ticks are 824–2,618.

  | inst | P2.2a runaway tick | lowest horse price / target (year) | years to tol | years to within 5% | dead ticks (labour, horse-days) | installed heads' lowest / target |
  |---|---|---|---|---|---|---|
  | H1 | 164 | 0.141 (17.4) | 128 | 69 | 1,720 (1,016, 704) | 0.400 |
  | H2 | 138 | 0.175 (17.7) | 118 | 66 | 558 (0, 558) | 0.439 |
  | H3 | 164 | 0.138 (22.2) | 157 | 81 | 2,132 (1,294, 838) | 0.407 |
  | H4 | 138 | 0.174 (22.5) | 150 | 79 | 701 (0, 701) | 0.440 |
  | F1 | 164 | 0.135 (45.9) | 305 | 137 | 4,108 (2,706, 1,402) | 0.439 |
  | F2 | 138 | 0.173 (46.2) | 313 | 139 | 1,452 (0, 1,452; good 13) | 0.447 |
  | F3 | 138 | 0.166 (17.5) | 126 | 69 | 597 (0, 597) | 0.425 |
  | F4 | 138 | 0.164 (22.3) | 157 | 82 | 740 (0, 740) | 0.427 |
  | F5 | 142 | 0.193 (19.0) | 136 | 64 | 361 (0, 361) | 0.527 |
  | F6 | 142 | 0.193 (15.0) | 110 | 53 | 309 (0, 309) | 0.520 |
  | F7 | 167 | 0.156 (17.6) | 128 | 69 | 1,713 (998, 715) | 0.396 |
  | F8 | 138 | 0.176 (17.7) | 118 | 49 | 568 (0, 568) | 0.435 |
  | F9 | 167 | 0.153 (22.5) | 158 | 81 | 2,130 (1,278, 852) | 0.403 |
  | F10 | 138 | 0.174 (22.5) | 151 | 61 | 712 (0, 712) | 0.437 |

**E4. P8** (a 0.005, ω 1, δ 4%, heads × 2).
- **No runaway**, where P2.2a had DIVERGED at tick 138.
- The horse price's lowest is **0.215** of target, at year 10.2. It stays within 5% from year 246.
- In tolerance from tick **24,601 (473 years)**.
- At L = 40,000 the class is **STUCK**: at rest over the last tenth, but not inside tolerance
  over the whole second half.
- At 10·L = 400,000 it is **CONVERGED** by the stream classifier.
- The base kick set is P2.2a's at the same horizon, because the end state is the rest point and
  the local map is unchanged: P2.2a measured a tail of 1.4e-3 at 40,000 and g 0.983 a year.
- No dead ticks. Baskets' lowest is 0.887 of target, the horse price's highest 4.39 of target,
  and installed heads' lowest 0.954.

**E5. P2.2a's battery.** H1–H4 and F1–F10 cover Tiers 1–3S and Tier 3 and 3S at 10·L: 1,980
scored runs.
- **No class changes: 1,980/1,980 CONVERGED**, as in P2.2a.
- The rule acts (μ < ψ) in **84 runs at L** and the same 84 at 10·L, all in Tier 3 or 3S. The
  count per instance is H1 3, H2 10, H3 3, H4 10, F1 3, F2 10, F3 7, F4 8, F5 4, F6 4, F7 1, F8
  10, F9 1 and F10 10. They are listed in `i_cmp_025.out`.
- The other 1,214 runs at L, and their 10·L twins, are P2.2a's runs bit for bit.
- Two runs sit within 2% of the step and may flip either way with no class change: F4 x\*/2 at
  μ 0.2453 acts, and F3 x\*/2 at 0.2535 does not (`i_mu.out`).
- In the runs where it acts:
  - the horse price's lowest over target is at least **0.189**;
  - ticks to tolerance move by −6.3% to +0.5% from P2.2a's;
  - dead ticks are equal or fewer (H2 N × 2: 45 to 10; F8 N × 2: 109 to 62; H2 r × 2: 119 to
    86).
- Per instance, the lowest horse price over target, P2.2a's then the rule's:

  | H1 | H2 | H3 | H4 | F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9 | F10 |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|---|
  | 0.126 → 0.239 | 0.0041 → 0.189 | 0.118 → 0.236 | 0.0033 → 0.190 | 0.107 → 0.233 | 0.0011 → 0.191 | 0.016 → 0.207 | 0.015 → 0.209 | 0.034 → 0.205 | 0.036 → 0.206 | 0.121 → 0.251 | 0.0086 → 0.214 | 0.114 → 0.250 | 0.0059 → 0.211 |

- The ticks to tolerance, median and largest, are P2.2a's at every instance within 35 ticks.

**E6. The stocks family** (every coin × 0.02 and × 0.1, every stock × 0.1 and × 10, 14
instances).
- Only the 14 heads × 10 runs change class, as E3 gives. The other 322 are CONVERGED as in
  P2.2a.
- The rule also acts in finished × 10 and in the capacity, fodder and good desks' coin × 0.02
  and × 0.1, with no class change.

**E7. The other families.**
- Tick length (Tiers 1–2 at 12, 24 and 365 a year, H1–H4): no class changes. The failures at 12
  a year and at 24 a year at ω 1 (O45) stay.
- P7: unchanged, since the rule never acts there.
- P3 (ii)–(v): no class changes.

**E8. Negative controls for the reservation's range.**
- **ψ = 1**, the reservation at replacement cost itself. A 1e-9 downward kick on the horse price
  ends **ORBITING** at H1–H4: a 2-cycle 8–9% below target, withholding on 3,600–12,100 ticks of
  20,000–30,000. The upward kick is classed VACUOUS and then orbits too. F1 and F2 may orbit from
  mode A, depending on the ulps of μ at genesis.
- **ψ = 0.75** on P8: **ORBITING** at 40,000 and 400,000 ticks, a limit cycle with the horse
  price at 0.6–2.2 of target.

**What would refute the choice.** Any of these would send O47 back to the scan with the
alternative:
- a class change in E5–E7;
- any runaway in E3 or E4;
- any engine run in which the maker's markup never falls below ψ and still differs from P2.2a's.
