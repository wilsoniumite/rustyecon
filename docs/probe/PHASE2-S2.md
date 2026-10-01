# PHASE2-S2: Phase 2 proper's second session, the families, the trap, the switch, a zero price (P2.4)

Dated 2026-10-01. Steps P2.4.1–P2.4.19 on branch `phase2-s2`, from `reboot` at `a483ed0`. The
brief was PHASE2-S1 §6, in its order: (A) one wave of O22's families on I1–I3 and the dial
neighbourhood of C1, C2 and IW1; (B) the subsistence trap's remedy, O100; (C) the type switch,
O97; (D) a zero price markets can hold, O96 and O101. Each of B–D was chosen by a mirror scan,
registered before any code, built off on every old tape (R1), passed E0 against its mirror, and
ran its registered battery. Rulings: decisions 400–424 at registration and build, 425–430 here.
The 1750-like instance was not in this session.

| part | registered (frame, sha256) | built | scored | machinery |
|---|---|---|---|---|
| A | [families/SPEC.md](families/SPEC.md) `23bb3f3d…dd50` (P2.4.1) | no code | [results/families/](results/families/README.md) (P2.4.16) | [families-wave/](results/families-wave/README.md) |
| B | [trap/SPEC.md](trap/SPEC.md) `7a27f121…5bff` (P2.4.4) | [TRAP-RULES.md](TRAP-RULES.md) (P2.4.5) | [results/trap/](results/trap/README.md) (P2.4.17–18) | [p24-wave/](results/p24-wave/README.md) |
| C | [switch/SPEC.md](switch/SPEC.md) `ebf52cea…315f6f` (P2.4.7) | [SWITCH-RULES.md](SWITCH-RULES.md) (P2.4.8) | [results/switch/](results/switch/README.md), amendment A1 | the same |
| D | [free/SPEC.md](free/SPEC.md) `ef6c888f…3a04` (P2.4.10) | [FREE-RULES.md](FREE-RULES.md) (P2.4.11) | [results/free/](results/free/README.md), amendments A1 (E0) and A2 | the same |

**Conditions of every number below, unless its line says otherwise.** The many-market roles with
the session's three optional fields (the exit's `pace`, the workers' `pool`, a good's `free` with
the exit's `market`); C2m, 52 ticks a year, ρ 0, `Saturate`, planned assignment, no government,
no loop; tolerance 1e-3 in log; L from the engine's elasticity probe as registered. Wave A ran on
`markets` built from `66ae453` (sha256 `be3266ab…42e9`), the B–D waves on `markets` built from
`2b68736` (`aa97c626…88d4`), both WSL release, each from a `git archive` export in its own target
directory. "The mirror" is each frame's registered model.

## 0. The verdict

| id | what it is | registered | engine | verdict |
|---|---|---|---|---|
| I1–I3 | O22's families (stocks, joint2, joint4, basin, Hold, tilt 1, histories) | all 3,063 CONVERGED; I3's five map cells GO | the same, to the tick | **holds** (O109 closed) |
| C1, C2, IW1 | the dial neighbourhood, 17 settings × Tier 3 and 3S | C2, IW1 everywhere; C1 trapped in 18 runs at 7 settings | the same 18, to the tick | **holds** |
| C1P | C1 with participation at a rate (O100) | GO with margin | GO with margin | **GO with margin** |
| C2P | C2 the same | GO with margin | GO with margin | **GO with margin** |
| IS1 | IW1 with E's efficiencies, the migration rule (O97) | GO | GO (on A1, after the result) | **GO** |
| IS2 | IS1 pooled at its base, a control | reported | as predicted | control |
| IL1 | idle enclosed land at r = 0 under the free step (O96) | GO | GO | **GO** |
| CT2 | two types on one commons market under the free step (O101) | GO | LOCAL: 2 of 13 kick sets fail | **LOCAL, not as registered** |

**In every scored run of the session's 16,637 jobs the class is the mirror's.** Wave A's 6,443 jobs
and the trap's 4,770 hold every scored line. The switch's 3,071 failed 236 lines as scored and
88 after its amendment A1; the free step's 2,353 failed 27 and 4 after its A2. Both amendments
were written after the result and are disclosed as such; neither moves a class, a tick or a
verdict. Of the four registrations' refutation criteria, one is met: the free step's "a kick set
that fails", at CT2.

## 1. What was built

Three optional fields, no new actor kind or market rule, each off when absent, every committed
tape's text, `tape_hash`, `world_id` and 2,000-tick stream unchanged on WSL and Windows, the pins
unmoved (gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645`):
- **the exit's `pace`** (P2.4.5): the share of heads offering hours moves 1 − exp(−1.3/tpy) of its
  gap to the participation rule's each tick; the plots stay the rule's. C1P, C2P, C1PN. 10 tests.
- **the workers' `pool`** and the appended state `SwitchWorkers` (P2.4.8): a type's pool share
  moves toward the market that pays more by share(k·|g|) of the worse market's hours,
  g = ln(ε·w/w_i). IS1, IS2. 17 tests.
- **a good's `free`** and the exit's `market` (P2.4.11): p′ = p·e^(kx) + (c·p_ref)·expm1(kx),
  posting 0 where that is not positive, c 0.5, labour the reference. IL1, CT2. 18 tests.

Each passed E0 against its mirror before any scored run (within 3.6e-14, 1.35e-13 and 6.1e-14;
the free step's two partings at a free price near 0 under its A1, P2.4.12).

## 2. The prediction and the result

**Wave A** ([results/families/README.md](results/families/README.md)): 32,411 lines, none failing.
O22's families converge on all three of P2.1's instances, I3's rate-by-buffer map is complete
(eight of nine cells GO), and I2's history measures its slowness, not its recovery (O111).

**The trap's remedy** ([results/trap/README.md](results/trap/README.md)): 107,209 lines, none
failing. Paced, C1 converges in all 731 runs of the dial neighbourhood where the registered rule
loses 18; C2P in all 697. The basin, joint and history families converge in full. Beyond the
slice, at every tilt 2, the trap still takes 3 of C1P's and 2 of C2P's Tier-3 runs, as registered
(O113). The point does not move: every CONVERGED run ends within 6.8e-14 in log of the oracle's.

**The type switch** ([results/switch/README.md](results/switch/README.md)): IS1's battery,
Tier 3S, 10·L, kick sets, envelope and corner rate as registered; a type crosses its switch in the
registered 61 of 129 runs; the 68 that never pool are P2.3's IW1 runs; the dial neighbourhood and
the switch's own rate hold in 697 + 164 runs. The 236 failed lines are all E5's end of a pop's
share (§3).

**The free step** ([results/free/README.md](results/free/README.md)): IL1's land is free on every
tick at its point at three tick lengths, reopens where land is scarce and every kick set decays.
CT2 converges in every run at every dial, its commons free, crowded or enclosed as the oracle's,
but two kick sets fail (§3).

**The dial neighbourhood across the session's nine instances**
([results/families/README.md](results/families/README.md), "The dial-neighbourhood map"):

![the dial map](figs/families/dialmap.png)

Every ±10% and ±25% move of the price rates, the buffers and the technique rates, and every tilt
from 0.05 to 1, keeps Tier 3 converging at IW1, IS1, IS2, C2, C1P, C2P, IL1 and CT2. C1 loses it
at seven settings (rate × 0.75 and × 0.9, buffer × 1.1 and × 1.25, tilt 0.25, 0.5 and 1), to the
subsistence trap; the pace restores all seven. IS1 also holds at its switch rate × 0.75–1.25. The
paced instances' edge is at tilt 2.

## 3. The failures, and how they were read

The B–D waves' agent stopped at a usage limit after its scorers ran. On 2026-10-01 the record
was checked before it was read, and nothing was run again: every job ran once under its own name;
a fresh build gives the wave's binary byte for byte; the committed gather and scorers regenerate
every output byte for byte, from the raw runs and from their archive
([results/p24-wave/README.md](results/p24-wave/README.md), "Files added after the waves").

- **The switch, 236 lines** ([A1](results/switch/registration-A1.md)). 148 pooled ends failed
  "within 1e-10 of a\*" because the scorer read a field printed to 7 digits; at the CSV's last
  row they are within 8.4e-13. The other 88 are walled ends above 1e-300 at a small gap or a slow
  switch rate, which the scorer's reading 4 expected before the wave from the registration's own
  full-L run; each is on its wall side. They stand.
- **The free step, 27 lines** ([A2](results/free/registration-A2.md)). 8 end regimes were scored
  against the mirror's bids-against-offers readout, which the registration's own OF6 rules out in
  exactly those 8 runs. 3 runaway ticks were registered on the undisplaced genesis, not the
  harness's reference the registration names (O107 again); on it the mirror gives the engine's
  tick in 8 of 8. 12 end D̂ above 1e-9 are runs at a slow dial root still falling at the
  registered root, which by the registration's own numbers could not reach 1e-9 by L.
- **Four lines stand, all CT2's.** Its kick sets at `b.food=1.2@dated` (Crowded, r_o 0.0118·r)
  and `exit.To=9.75@dated` (Enclosed), its verdict, and one switch count. Diagnostics after the
  result: at the Enclosed target the failing kicks are the commons' own price, a neutral direction
  of the continuum of rest points the registration names (§4.4); at b.food × 2 the kicked runs
  settle up to 2.7e-12 in log from the unkicked run and stay (the same tail at three horizons).
  The free mirror, run the harness's way, does both. The registration predicted each kick set from
  the mirror's largest root, which sees neither. The switch count parts where a pop's coin cannot
  cover its commons bid: the engine's budget chain cuts the bid, the mirror spends coin it lacks,
  in 12 of CT2's 1,225 runs.

![CT2's kick sets](figs/free/ct2_kicks.png)

## 4. What this means

**For the 1750-like instance's prerequisites** (PHASE2-S1 §6): O100 (the trap) is answered by
participation at a rate; O97 (the type switch) for a type with reserved tasks; O96 (idle land at
r = 0) by the free step. O101 (several types on one commons) is not: CT2 is LOCAL. Its runs say the
rule works; its kick sets say the verdict rule, as registered, cannot pass at a point with a
continuum of rest points or a resting offset at a small price. That is a question about the
measure, and it binds the 1750-like instance if its commons is shared.

**For PLAN's Phase 2 gate.** "Green with margin" is now met at C1P and C2P on the dial slice
(decision 407's line), and the slice documents a stable region over nine instances at C2m. The
phase diagram proper is not yet run: only the base's kick set ran at each setting (O112), and the
1750-like instance is not built.

**Recommendation.** Take C1P, C2P, IS1 and IL1 as GO and CT2 as LOCAL, with A1 and A2 read as
disclosed (decisions 425–430). Before the 1750-like instance: rule how a kick set reads a point
with a neutral direction and a free step at a small price (a registration of the measure, not an
amendment of this wave), and make the mirrors cap a pop's commons budget as the engine does. Then
the 1750-like instance as a flow instance, as PHASE2-S1 §6 recommends. An independent review of
this session's two after-result amendments, as P2.3.16 had, would come first.

## 5. Open questions

1. **CT2's kick sets** (O101, O124, O127). Should the kick skip the neutral direction of a
   continuum (as the registration's PL did), and read a tail that does not fall with the horizon as
   a rest rather than a mode? Either is a change to certify's measure.
2. **The resting offset at a small rent** (O127). Its size is up to 2.7e-12 in log at
   r_o 0.0118·r, and 1.9e-13 at r_o 0.64·r. Its source is not established: the free step's
   increment c·p_ref·expm1(kx) is large against a small price, a candidate. Whether a larger c
   removes it is unscanned.
3. **The budget corner** (O124). The free mirror lets a pop spend coin it lacks; the engine does
   not. Every several-pops mirror should follow the engine before its next registration.
4. **Not tested:** ρ > 0, a government, 12 ticks a year as a default, targets' kicks at each dial
   (O112), a pooled type without reserved tasks or with ν ≠ 1 (O118), the 1750-like instance.
