# The maker's reservation on the engine: results

Dated 2026-09-29. Steps L0.3–L0.6 on branch `phase2-loops`. O47, the idle machine market: the
remedy the mirror scan chose (`D:/rustyecon-p2l/idle-scan/IDLE-SPEC.md`, sha256 `c5c6527a…`), run
on the engine against its registration.

- **L0.3 `96cba59`**, the registration ([registration.md](registration.md), sha256 in
  `registration.sha256`): written on top of `ed8fc6c` before the rule existed in the engine.
- **L0.4 `882e9f8`**, the build (docs/probe/HORSES-RULES.md §10).
- **L0.5 `fac7db4`**, a fix found by the registration's own check before any scored run (below).
  Every run here is this commit's release build on WSL (`horses` sha256 `6a98bba7…`, in
  `D:/rustyecon-p2l/idle-engine/runs/bin.sha256`).

The rule: the maker offers none of its finished heads while its net markup
p_K·(1 − δ·a/κ)/c_m at posted prices is below ψ = 0.25. Nothing else changes.

## Verdict

**The registered predictions hold. None of the registration's refutations occurred.** No run
ran away. The 14 heads × 10 runs, which P2.2a's engine lost at ticks 138–167, all converge. So
does P8, which is STUCK at 40,000 ticks and CONVERGED at 400,000. All 1,980 battery runs converge,
as in P2.2a. Where the rule never acts, all 1,810 of those runs, the result is P2.2a's to every
CSV row. The class is the mirror's in every scored run. There are five misses. None is on the
refutation list, and each has a cause (below). So O47 is closed with the reservation at ψ 0.25.

| item | registered | engine | |
|---|---|---|---|
| E1 nesting | tapes, hashes, pins unchanged; ψ 0 is P2.2a | six horses tapes' 2,000-tick hash streams and headers equal P2.2's (after L0.5); ψ 0 at H1–H4 mode A, H2 heads × 10 and P8 heads × 2 equal P2.2a's to every CSV row, summary and statistic | holds |
| E2 mode A, local map | P2.2a's bit for bit | mode A PASS at all 14, every row P2.2a's; `slowest hold` envelope and g P2.2a's at 14/14; base kick set P2.2a's at 14/14; at b × 2@dated every kick set PASS but not P2.2a's bit for bit (L0.7 below) | holds |
| E3 heads × 10 | no runaway; CONVERGED ×14; ticks and prices as tabled | 14/14 CONVERGED; ticks to tolerance equal at 12, off by 1 tick at H4 and F2; lowest price within 0.4% | holds |
| E4 P8 | STUCK at 40,000, CONVERGED at 400,000; low 0.215; tol from 24,601 | the same, low 0.2146 at year 10.25, tol from tick 24,601 | holds |
| E5 battery | 1,980/1,980 CONVERGED; acts in 84 at L and the same at 10·L | 1,980/1,980; acts in 85 at L and the same 85 at 10·L (F3 x\*/2, named as free to flip); the other 1,810 are P2.2a's | holds, one miss |
| E6 stocks family | only heads × 10 changes class | 14 of 336 change, all heads × 10, DIVERGED → CONVERGED; acts in 87 (mirror 88) | holds, one miss |
| E7 other families | no class changes | none in 540 tick-length runs, 4 P7 runs or 59 P3 runs | holds |
| E8 negative controls | ψ 1 orbits (kick ORBITING, up-kick VACUOUS); ψ 0.75 on P8 ORBITING | ψ 1 orbits at all six instances, mode A included; both kicks classed VACUOUS; ψ 0.75 ORBITING at 40,000 and 400,000 | holds in substance; label miss |

## Before any scored run (registration §3)

- **Tests.** Seven new tests pass: six at L0.4 and one at L0.5 (HORSES-RULES §10). Each fails with
  its change undone. Seven mutants were caught (`D:/rustyecon-p2l/idle-engine/build/mutants/`):
  - the rule removed;
  - the load checks removed;
  - an absent reservation read as 0.25;
  - a threshold of ψ + 1e-3;
  - a threshold of 4.1ψ, past the rest's markup;
  - withholding by burning the heads (the engine refuses the decide-phase burn);
  - at L0.5, the `world_id` attribute removed.
- **Gates.** Both scripts ran on WSL and on Windows before each commit:
  - L0.4: 873 workspace tests, 3 ignored; the GUI's 117, 4 ignored.
  - L0.5: 874 workspace tests.
  - At `fac7db4`, `scripts/gate.sh` ran again with a clean stamp: 96 s on WSL, 139 s on Windows.
  - The gate hash is `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`
    (stream `0xdb63cc96f769fb3e`). The probe's and the markets probe's pins hold.
- **E1's hashes found a bug.** L0.4 left the six horses tapes' text, `tape_hash` and per-tick
  hash streams alone, but it moved their `world_id`. The cause: `world_id` hashes the resolved
  actors by bincode, and the resolved `Maker` wrote `reserve: None`. L0.5 leaves the field out
  when it is `None`, and a test pins the six P2.2 `world_id`s. The six horses tapes and the
  seven markets tapes now give P2.2's WSL hash streams, headers included, for 2,000 ticks.
- **The trace diff** ([tracediff.tsv](tracediff.tsv); `D:/rustyecon-p2l/idle-engine/tracediff/`).
  The scan's `i_mirror.py` gets P2.2a's genesis carry (`make_i_carry.py`) and runs against the
  engine for 2,000 ticks at ψ 0.25. The withheld ticks agree tick for tick in every run.

  | run | worst gap in log | prices, technique, stocks | volumes, absolute | outputs, absolute | the mirror against itself | withheld ticks (engine, mirror) |
  |---|---|---|---|---|---|---|
  | H2 r × 2 | 1.6e-12 (horse volume) | 1.7e-13 | 3.7e-13 | 1.1e-13 | 7.1e-13 | 65, 65 |
  | P8 heads × 2 | 8.8e-11 (the maker's output of 5e-8 heads) | 2.7e-12 | 8.1e-12 | 3.1e-12 | 1.1e-12 | 473, 473 |
  | H2 heads × 10 | 8.9e-2 (horse volume) | 9.8e-3 | 7.0e-3 | 3.4e-4 | **1.1e-1** | 977, 977 |

  All three pass HORSES-RULES §6.4's standard. Each run parts first on the horse market's volume.
  - **H2 r × 2 and P8:** the parting is at rounding once it is measured against the oracle's
    volumes and outputs.
  - **H2 heads × 10:** the run parts at tick 936 (year 18), when orders resume against the
    withheld pile. From then on the offer switches on and off tick by tick (the chatter, O55), and
    an ulp can decide a switch. The mirror against itself, one ulp apart, parts by 0.11 there. The
    engine stays inside that spread.

## E3: heads.capacity × 10 at the 14 instances ([heads_x10.csv](heads_x10.csv))

| inst | P2.2a | class | years to tol (mirror) | lowest pK / target, year (mirror) | pK within 5% from year | dead ticks: labour, horse-days | withheld | switches | pK high | installed heads low |
|---|---|---|---|---|---|---|---|---|---|---|
| H1 | runaway at 164 | CONVERGED | 127.6 (127.6) | 0.141, 17.4 (0.141, 17.4) | 69.1 | 1,016, 704 | 1,004 | 92 | 8.3 | 0.400 |
| H2 | 138 | CONVERGED | 117.6 (117.6) | 0.175, 17.7 (0.175, 17.7) | 65.8 | 0, 558 | 977 | 130 | 20.2 | 0.439 |
| H3 | 164 | CONVERGED | 157.4 (157.4) | 0.138, 22.2 (0.138, 22.2) | 81.4 | 1,294, 838 | 1,279 | 108 | 10.2 | 0.407 |
| H4 | 138 | CONVERGED | 149.6 (149.6) | 0.174, 22.5 (0.174, 22.5) | 79.0 | 0, 701 | 1,237 | 146 | 26.6 | 0.440 |
| F1 | 164 | CONVERGED | 304.6 (304.6) | 0.135, 45.9 (0.135, 45.9) | 137.4 | 2,706, 1,402 | 2,618 | 158 | 24.8 | 0.439 |
| F2 | 138 | CONVERGED | 312.7 (312.6) | 0.173, 46.3 (0.173, 46.2) | 139.0 | 0, 1,452 (good 19) | 2,525 | 200 | 119.7 | 0.447 |
| F3 | 138 | CONVERGED | 126.2 (126.2) | 0.166, 17.5 (0.166, 17.5) | 68.8 | 0, 597 | 978 | 128 | 14.5 | 0.425 |
| F4 | 138 | CONVERGED | 156.8 (156.8) | 0.164, 22.3 (0.164, 22.3) | 82.1 | 0, 740 | 1,240 | 144 | 18.8 | 0.427 |
| F5 | 142 | CONVERGED | 135.6 (135.6) | 0.193, 19.0 (0.193, 19.0) | 63.7 | 0, 361 | 1,040 | 122 | 9.3 | 0.527 |
| F6 | 142 | CONVERGED | 109.6 (109.6) | 0.193, 15.0 (0.193, 15.0) | 53.4 | 0, 309 | 824 | 108 | 8.1 | 0.520 |
| F7 | 167 | CONVERGED | 128.4 (128.4) | 0.156, 17.6 (0.156, 17.6) | 68.6 | 998, 715 | 999 | 82 | 8.5 | 0.396 |
| F8 | 138 | CONVERGED | 118.4 (118.4) | 0.176, 17.7 (0.176, 17.7) | 49.3 | 0, 568 | 975 | 126 | 19.6 | 0.435 |
| F9 | 167 | CONVERGED | 158.2 (158.2) | 0.153, 22.5 (0.153, 22.5) | 81.3 | 1,278, 852 | 1,273 | 96 | 10.4 | 0.403 |
| F10 | 138 | CONVERGED | 150.7 (150.7) | 0.174, 22.5 (0.174, 22.5) | 61.2 | 0, 712 | 1,235 | 142 | 25.7 | 0.437 |

Each run's class is its registered run's, at P2.2a's L. The years come from every-tick paths of
20,000 ticks (20,800 at F1 and F2). The dead ticks and withheld counts equal the mirror's at every
instance. The one exception is the good's dead ticks at F2: 19 against 13.

## Per instance: the battery (E5) with the glut (E3)

| inst | heads × 10 | battery runs converged | runs where it acts at L (mirror) | lowest horse price / target in the battery: P2.2a → spec → engine | ticks to tol, median and largest, against P2.2a |
|---|---|---|---|---|---|
| H1 | DIVERGED → CONVERGED | 142/142 | 3 (3) | 0.122 → 0.239 → 0.239 | +0, +0 |
| H2 | DIVERGED → CONVERGED | 142/142 | 10 (10) | 0.0038 → 0.189 → 0.187 | +0, +3 |
| H3 | DIVERGED → CONVERGED | 142/142 | 3 (3) | 0.116 → 0.236 → 0.235 | +0, +0 |
| H4 | DIVERGED → CONVERGED | 142/142 | 10 (10) | 0.0031 → 0.190 → 0.188 | +0, +2 |
| F1 | DIVERGED → CONVERGED | 142/142 | 3 (3) | 0.103 → 0.233 → 0.239 | +0, +0 |
| F2 | DIVERGED → CONVERGED | 142/142 | 10 (10) | 0.00097 → 0.191 → 0.195 | +0, −34 |
| F3 | DIVERGED → CONVERGED | 142/142 | 8 (7: x\*/2 flips) | 0.016 → 0.207 → 0.208 | +0, +0 |
| F4 | DIVERGED → CONVERGED | 142/142 | 8 (8) | 0.014 → 0.209 → 0.205 | +0, +0 |
| F5 | DIVERGED → CONVERGED | 138/138 | 4 (4) | 0.033 → 0.205 → 0.206 | +0, +16 |
| F6 | DIVERGED → CONVERGED | 138/138 | 4 (4) | 0.036 → 0.206 → 0.204 | +0, +3 |
| F7 | DIVERGED → CONVERGED | 142/142 | 1 (1) | 0.127 → 0.251 → 0.251 | +0, +0 |
| F8 | DIVERGED → CONVERGED | 142/142 | 10 (10) | 0.0090 → 0.214 → 0.214 | +0, +11 |
| F9 | DIVERGED → CONVERGED | 142/142 | 1 (1) | 0.120 → 0.250 → 0.250 | +0, +0 |
| F10 | DIVERGED → CONVERGED | 142/142 | 10 (10) | 0.0064 → 0.211 → 0.211 | +0, +7 |

**Every instance passes.** Its glut converges, its battery keeps every class, and its lowest
horse price is within 3% of the spec's (F1 +2.6%, F2 +2.1%, F4 −1.9%).
- In the 170 acting runs (85 at L, 85 at 10·L), ticks to tolerance move by −6.3% to +0.5% against
  P2.2a's.
- No acting run has more dead ticks than P2.2a's.
- The ticks to tolerance are within 10% of the mirror's in all 170.
- The lowest horse price is within 5% of the mirror's in 168 of the 170.

## The misses, with their causes

1. **E5, H2 r × 2: the lowest horse price is 0.220 of target against the mirror's 0.204 (+7.4%),
   at L and at 10·L.**
   - The scan's mirror has no genesis carry. The carry changes tick 1, and this run's low comes
     early: tick 40 in the engine, tick 46 in the mirror.
   - The carry mirror of the trace diff agrees with the engine within 1.6e-12 on this run. The
     class, ticks to tolerance (+0.4%) and withheld ticks (65 against 63) are within tolerance.
2. **E5 acts in 85 runs at L, not 84.** The one added run, F3 x\*/2, is the one §8 named as
   sitting within 2% of the step and free to flip.
3. **E6 acts in 87 runs of 336, not 88.** F3's capacity desk coin × 0.1 has its lowest markup at
   0.2503. There the mirror withheld on one tick and the engine on none. Both converge.
4. **E8's class labels.**
   - At ψ 1 the engine orbits from mode A at all six instances. The mirror orbited from mode A
     only at F1 and F2, and the spec expected that ulps could decide it. The engine's genesis
     markup is exactly 1 at H1 and H4, and 1 − 2e-16 and 1 − 1e-16 at H2 and H3. Rounding along
     the run then puts it under the step.
   - The ±1e-9 kicks give the mirror's orbit: 3,649, 9,631, 4,981 and 12,068 withheld ticks at
     H1–H4, as the mirror has them, and the horse price 8–9% below target (low 0.914–0.923).
   - The harness classes both kicks VACUOUS, where the mirror called the downward kick ORBITING.
     The harness's D̂₀ for a price displacement is the genesis gap, 1e-6 (PROBE-SPEC §4.5). The
     scan's `i_run.py` reads D̂₀ as the first tick's D̂, which is infinite once the horse market
     clears nothing. The negative control holds in substance, since ψ 1 orbits, but the label
     misses (O56).
   - ψ 0.75 on P8 is ORBITING at 40,000 and 400,000 ticks, with the horse price down to 0.645 of
     target.
5. **F2 heads × 10: the horse price's post-glut high is 119.7 times target.** The spec's range was
   8–112, and the mirror had 112.3 here. It is reported, not scored. It sits in the chatter the
   trace diff shows to be ulp-sensitive.

## What else the runs show

- **P3 (iv), the order on the stock held with s_K 0, is the unstable negative control.** It
  diverges in all 12 of its runs, as in P2.2a. But 11 of them now leave the bound upward, the
  horse price above 1e6 of genesis at ticks 2,922–6,694, instead of downward at 1,979–4,256. The
  reservation bounds the fall; it does not steady an unstable desk.
- **P3 (v), the maker without a cover, converges in all 30 runs.** The rule acts in 7 of them.
- **Tick length.** The rule acts in 3 of the 540 runs, all at 12 a year: r × 1.2 at H2 and H4,
  and N(1.2) at H4. The 12- and 24-a-year failures (O45) stay as they were.
- **P7 never acts.** Its lowest markup is 0.65 at C2 and 0.88 at C2g, and its runs are P2.2a's to
  the row.
- **What the rule does not fix** (O52, O53). After a glut the horse market swings to shortage:
  installed heads fall to 0.40–0.53 of target and the horse price rises to 8–120 times it. At
  ω ½ labour is dead for a further 998–2,706 ticks. P8 still needs 473 years to reach tolerance.
  See [fig1](../../figs/idle/fig1_x10_price_and_markup.png),
  [fig2](../../figs/idle/fig2_x10_glut_and_shortage.png) and
  [fig3](../../figs/idle/fig3_p8.png).

## Amended at L0.7 (2026-09-29): what the engine review found

The review rebuilt `15b05af` on both machines and reproduced every check it tried, with no
refutation. It found five minor issues. Each is answered here, and the code ones are fixed at
L0.7 (docs/probe/HORSES-RULES.md §10, "Amended at L0.7").

1. **The dated-target kick sets are not P2.2a's bit for bit.** At b × 2@dated the rule acts on the
   way to the new target (46 withheld ticks at H2), so the kicks start from another end state. At
   the 10 instances with that target, 30 kick and slowest-mode files differ from P2.2a's and 234
   are equal. Every kick set passes. The slowest mode a year, rule on against P2.2a:

   | H1 | H2 | H3 | H4 | F1 | F2 | F3 | F4 | F8 | F10 |
   |---|---|---|---|---|---|---|---|---|---|
   | 0.9013 / 0.9007 | 0.8546 / 0.8540 | 0.9132 / 0.9132 | 0.8824 / 0.8824 | 0.9449 / 0.9449 | 0.9313 / 0.9305 | 0.8802 / 0.8812 | 0.8917 / 0.8983 | 0.8729 / 0.8756 | 0.8936 / 0.8936 |

   The largest move is F4's, 0.0066 a year. E2's row above now says so. IDLE-SPEC §0's "mode A and
   every kick set" is a claim about the mirror (its dated amendment,
   [IDLE-SPEC-A1.md](IDLE-SPEC-A1.md), sha256 in `IDLE-SPEC-A1.sha256`).
2. **The bound on the price is one step below ψ times the replacement cost at the maker's last
   offer**, not at current costs. The price never falls on a withheld tick, but the wage and
   fodder keep moving while the maker withholds. So its markup at current costs goes lower: 0.144
   to 0.206 at heads × 10, up to about five steps below ψ. At H1 the last offer is at tick 39 (markup
   0.2502); the price falls one step, 49.98 to 45.22, and holds, while the markup falls to 0.1506
   at tick 903. The lowest horse price is still 0.135–0.193 of target.
3. **Off was not structural.** With ψ 0 or absent, L0.4's maker withheld wherever δ·a/κ > 1,
   which makes the markup negative. No committed tape and no run here has that. L0.7 withholds
   only when ψ > 0.
4. **The harness read rule A's running recipe from its own code.** It now reads the tape's
   recipes through the rule's own code. Rerun from L0.7's build on WSL, the review's sample of
   17 runs (H2 r × 2, N(2), JA(0.8) and hold; H1 r × 2 and b × 0.5@dated; H2 and F2 heads × 10;
   F3 x\*/2; P8 heads × 2; E8's ψ 1 at H1, hold and the downward kick; P3 (iv) at H2; H4 r × 1.2
   at 12 a year; H2 r × 2 at 10·L; ψ 0 heads × 10; the F2 path) equals this README's evidence in
   every CSV row, summary line and statistic, the `markup` and `withheld` columns included
   (`D:/rustyecon-p2l/fix-report/rerun/`; H2's hold against mode A's run, on the 2,887 ticks both
   wrote). Windows gives the same on H2 heads × 10, H2 r × 2 and hold, and P8 heads × 2. So no
   result here moves.
5. **L0.1's M6 test could not fail on M6 alone**, since M5 refuses the same tapes. A new test
   has a scripted seller of stored fodder, which M5 does not check, so only M6 refuses it.

## Files

- `registration.md`, `registration.sha256`: the registration (L0.3).
- `IDLE-SPEC-A1.md`, `IDLE-SPEC-A1.sha256`: the spec's amendment after the engine review (L0.7).
- `battery.csv`: the 1,980 battery runs. For each: the class (P2.2a, on, the mirror's), the
  withheld ticks, switches and lowest markup, and the ticks to tolerance, lowest horse price and
  dead ticks (P2.2a, on, the mirror's). Also whether the summary, the statistics and every CSV
  row equal P2.2a's.
- `heads_x10.csv` (E3), `p8.csv` (E4), `modea.csv` (E2), `nesting.csv` (E1),
  `negative_controls.csv` (E8).
- `families.csv` (E6, E7): the stocks family, the tick-length family, P7 and P3, each against
  P2.2a.
- `tracediff.tsv`: the trace diff.
- Figures, in `docs/probe/figs/idle/`:
  - fig1: heads × 10 at H2 and H1, price and markup, off against on;
  - fig2: the glut and the shortage at H1–H4;
  - fig3: P8;
  - fig4: the battery's lowest horse price per instance, and the acting runs;
  - fig5: heads × 10, the engine against the registered mirror.

The scripts and raw runs are in `D:/rustyecon-p2l/idle-engine/`, not in the repository (sha256):

```
2bcf02edc85e64f579c814118bb19904b154f138bca292f29311b2b394b8c6d3  analysis/analyze.py
c9e7d6692215ffbfc279b4f9210bc38bef38b121b24eb12fab245bd2094fb087  analysis/plots.py
3729bd82073b94309893a69c85f8f13bf68ab5ed670ef7ccad5281d498364eae  runs/lib.sh
49f72578bd42ff7f502ddf877f11e5e91d9f27d1b518f9773c45619753d32fca  runs/wave1.sh
b01ad76a9da5630aaed08ed7ad5327def3fa10c65541c050a0f229cf8c0db5a2  runs/wave2.sh
78b6d3fbaead786e26250acc1ed8be2e95ace31ac474200f9859096e34809e2f  runs/wave3p.sh
0b0fee1e8aa6c2d602fe443422eb7b8bd255009f72f16ed52e05300c7ad6d20a  runs/wave4p.sh
65a37c72f01aab712ac42b9c2397dc48b9edf55b639e7691dca29944064260f8  runs/wave5p.sh
d79bd300a96be503a046bdf2bb335cd61513a169ce983166596b283cc0928dde  runs/e1/tapes-bit.sh
1c5e85d9c50313d927a8fe365ed058fb465c57aa3c0afda10bb2ac03b3a28fd9  tracediff/make_i_carry.py
91f4f1e41ebd3d76eae69c67be29298df52ad60e2b5d7185a6a2b59ac28e0433  tracediff/tracediff.py
d23f946e4fb6b4b36bb0c3b9522b46eea91d386e150888a3d256ffa92ce147fb  build/mutants/run.sh
1d17f928981a70b15b13c998e777e042520123584d026e85a1ce32754ae73ebe  runs/bin.sha256
```

Waves 3–5 were started beside wave 2's last part, to use idle cores. They are the same scripts
under the names `wave3p.sh`–`wave5p.sh`. The whole run took 99 minutes on 48 cores.
