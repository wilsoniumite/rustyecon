# The subsistence trap's remedy: registration of participation at a rate (P2.4.4, O100)

Dated 2026-09-30. Step P2.4.4 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`), on top of
`ec66cb7` (P2.4.3), clean but for this registration. It registers the mirror's predictions for the
engine run of O100's remedy, the workers' participation at a rate (the paced instances C1P, C2P
and C1PN), before any engine code for it exists (R5): at this commit `RawPricedExit` has no
`pace` field, `crates/probe` has no paced instance, and `tapes/` has no paced tape. PHASE2-S1 §6
item B. Any later change is a new dated amendment with its own sha256 (decision 282); earlier
results stay reported.

## 1. What is registered

The scan's frame, in [docs/probe/trap/](../../trap/), as its author left it in
`D:/rustyecon-p24/scan-trap/` (label `scan-trap`). The scan wrote `SPEC.md` on Windows with CRLF
line endings; the repository stores text with LF (`.gitattributes`), so the copy here is the same
text with LF. Both sha256s are listed: the registered one (as in `SHA256SUMS`) and the committed
one. They are equal wherever the scan wrote LF. `SHA256SUMS` (184 entries) was checked entry by
entry against the scan's files on 2026-09-30, all OK; its own sha256 is
`78904c579b206384ef1303bf89f05756ee18175651c97ff7bc8e95d49ff39ce5`.

| file here | the scan's file | sha256 as registered | sha256 as committed (LF) | what it is |
|---|---|---|---|---|
| `SPEC.md` | `SPEC.md` | `1dba64d3f69df3a7093e4ba966c0874eec9a791f4de327688a95047a65417ff7` | `7a27f121791ce504000250c0a2e020458cbd8aede11a33579038b76859915bff` | the spec: the trap, the candidates, the scan, the choice, the rule, the instances, the proof, the engine run, the predictions |
| `SHA256SUMS` | `SHA256SUMS` | (itself) | `78904c579b206384ef1303bf89f05756ee18175651c97ff7bc8e95d49ff39ce5` | sha256 of every file of the scan |
| `registered/runs.jsonl` | `pred/runs.jsonl` | `8f6f0fa7f74d7545e234ab15d83cbff762cef04ade39c00c3bb1dcd6597d4314` | the same | 3,901 runs: C1P and C2P's battery, Tier 3 at 10·L, Tier 3S at L and 10·L, stocks, pace, joint2, joint4, basin, history, enclose, Hold, tilt 1, 12 and 365 a year, the 17 dial settings; C1PN's battery |
| `registered/controls.jsonl` | `pred/controls.jsonl` | `f3b278ccf1fa11b6abe47d2841da346d449cf6b060181e940b565d9016dbded1` | the same | 817 control runs: the registered C1's Tier 3 at the 17 settings; C1P and C2P at every tilt 2 |
| `registered/modea.json`, `modea.out` | `pred/modea.json`, `modea.out` | `b62c9c22…5218`, `ea7d57c1…ef21` | the same | mode A at L at 12, 52 and 365 a year |
| `registered/predictions.json` | `pred/predictions.json` | `f9d6f4661f9aa6348943fb03fa11cb7604e6a5ae06e4655c03570a9f203babc8` | the same | SPEC §9's tables, as numbers |
| `registered/predictions.out` | `pred/predictions.out` | `d68708e3b79d92e254faeb9ec7af48a13597bed86d2667c3add9270428423ca5` | the same | the same, as text |
| `registered/lin-P1.3r.json`, `.out` | `lin/lin-P1.3r.json`, `.out` | `e9a37952…f2282`, `dc462f6e…19f3a` | the same | the largest root per tick and the kick bars at every target, paced |
| `registered/lin-base.json`, `.out` | `lin/lin-base.json`, `.out` | `c3f1ae03…e1ae3`, `5b819bea…767d4` | the same | the same, the registered rule |
| `registered/elasticity-P1.3r.json` | `lin/elasticity-P1.3r.json` | `1ed3de1fd887f82a777a110b7f766790f08a05d09cc68426bbfe232793e77a53` | the same | τ and L, paced (L 141,000 and 142,000) |
| `registered/restcheck.out` | `model/restcheck.out` | `fc08406514840377e7a1c757b171fb233ea7049aa9d5356c370aba4f0a39e0be` | the same | the rest point at the 26 targets, every candidate (SPEC §3.2) |
| `registered/tm_nest.out` | `model/tm_nest.out` | `6655e18af6f9bce9edd8d56f0786f18e77ad2220a8436326ec7ee7ca744c70cc` | the same | the mirror with every candidate off is the registered `cm.py` (SPEC §3.1) |
| `evidence/scan.out` | `runs/scan.out` | `ef6bd2322b04e242b79b67c2c0a4ae6d3429b4cea54a391e8c46b25d7398d0a9` | the same | the candidates' scan table (SPEC §3.3) |
| `evidence/edges.out` | `runs/edges.out` | `167cc0728916487efda9bb342d8ee8b216448108f95e27c17599eb0a7d2ce0bb` | the same | the finer dial grid (SPEC §3.6) |
| `evidence/held.out`, `heldout.out` | `runs/held.out`, `scan/heldout.out` | `813f6438…bb41`, `6846920e…2e57` | the same | the held-out instance X4 (SPEC §3.8) |
| `evidence/former_trap.out`, `zero_tick.out`, `tiers.out`, `trace1.out` | `scan/…` | `e13bc1cc…eb8b`, `f40cb82c…aac`, `e4008346…2509`, `edfd077c…da28` | the same | the trap's runs under the pace, the zero-hour ticks, the paths by tier, the mechanism traced (SPEC §1.1, §3.5, §3.7) |

The scorer (SPEC §8; decision 311) reads `registered/` from this copy.

The mirror the predictions come from stays in `D:/rustyecon-p24/scan-trap/model/`, as the
frames' did (sha256, from `SHA256SUMS`; written with CRLF, read in place):

```
79c5118569400c2254792acb25a8dc541387989993c2833eb130eb58447b8804  model/tm.py            (the candidates' mirror, E0's map: `tick`, `exit_rule`, `genesis`; variant P1.3r)
2985f36d3783a743b58b9c09be1e113b5c520c871c349c95007d96b52d705c25  model/tb.py            (the runner: run grammar with part.workers, observables, classes, readouts)
a25e8a1e169194faafb43467526933d896ba0ed0e0d1a64fe142c74c4734b1ae  model/ti.py            (C1 and C2's commons)
5c546727f7840f3465c6376a870994bf6d095becee35b569e4125938006197cc  model/variants.py      (the candidates as variants: P1.3r is prate 1.3 with the rule's plots)
4b3610fc4920d1980d8cd32dddd798109d9a909881ce6006a6a448a94da12c0c  model/predict.py       (the registered runs)
e03621c0be4de60bdab0ddcde0cc2fbf23678b87bd86838b5dc8920af2f931bf  model/predict_ctl.py   (the controls)
564ad8cca7afdd127ecf4c38c694dffd6d0a3cb52d2d2bf7859a6f8888e20590  model/modea.py         (mode A)
9b017cb2410b24964b5408adde4ea0752d1c84d988c92757827a2c0f0d991db1  model/pred_summary.py  (SPEC §9's tables)
69c36a83dc0dd2a1658fd53576a35daa0384eeb53c2a5b7b51284fcb95401427  model/restcheck.py     (the rest point)
319e2e98c84dd4edcfa657cbb6b30a07e6ef2e5e99211d691162ab14a1b54ee8  model/tm_nest.py       (the nesting check)
7b7c06b425b96f9e13764bc23f72f7cce78755ffba60c1b8c8538d4c63d14041  model/lin_t.py         (the largest root and the kick bars)
f67aa461549bd4820a3b8c6972a6d9987a489e19afda01a24dba006d54db3caa  model/elasticity_t.py  (the elasticity probe)
5131d2a2af0dc7cd4e08e3180c1af657483297920dfe5ccc46ff993ee3badc14  model/scanrun.py       (the scan's sets and the dial settings, `over_for`)
4d28d20bb6475ea93050743e718c54bab3e61fa3c92c10cac9f9b7a4dd9bdd86  frame/model/cm.py      (the registered commons mirror, copied unedited)
c3ec4a1c346cfa7475b659afbd0ec18788bf5ff43c84ed9becfb140d5c58dc29  frame/model/battery_c.py
```

## 2. The order of work

1. The scan ran on 2026-09-30 in scratch (label `scan-trap`), reading the worktree at `a483ed0`
   and changing nothing. Wave A's P2.4.1–P2.4.3 landed during and after it; none touches the
   workers' role or the markets harness's code. No engine code for the pace and no engine run of
   a paced instance exist.
2. This commit fixes the registration in the repository, and STATE.md numbers the scan's
   proposals (§5).
3. Next: the build (SPEC §5: the optional `exit.pace` on `BasketWorkers`, its load checks and
   genesis state; the rule line; the instances C1P, C2P and C1PN, the dial, the generator and the
   tapes; the harness's grammar and readouts; the tests; `docs/probe/TRAP-RULES.md`), with every
   committed tape's text, hashes and streams unchanged. Then E0, the trace diff against `tm.py`
   (variant P1.3r, the genesis carry), before any scored run. A real departure found by E0 is a
   dated amendment committed before scoring. The scorer is committed before the wave (decision
   311).

The sources at the base (sha256 at `ec66cb7`), which the build will change:

```
5f98b9f42e25bff427e04d5bff5ef4109e7dcca8d1465c6dbf1a1dbf2cffb9ce  crates/agents/src/roles/many/spec.rs
e026a615f1e9c5f8b25f1017a93b4d5cd1229166b6a69fdd1817e2fa51074b21  crates/agents/src/roles/many/rules.rs
0d550ce26ea1b889a77351acdb73d048b945f091e9ea936bd017c63546467440  crates/agents/src/ext.rs
4b0d6a1552762639b25062116c1ce5e1c3be2ce2a6d35e501f05b9327c18ab14  crates/agents/src/lib.rs
c41423bca61c1afeebe95bbb5257b709101f4ac3733538236213afb09ba3b882  crates/probe/src/markets/instance.rs
f145ac7a660171cd17c1bf9a035089a62a6ab5ac970411ea26e78cdb3ac9c1c6  crates/probe/src/markets/setup.rs
05fad8bf78530eb50c960f6f1d01cd2f1e766377b4540a2e8f90052ececb9226  crates/probe/src/markets/harness.rs
999fdd3da0c3a67222cc565a7c72f6d15d9f8073816492ddb8e0f5d3f08c4f70  crates/probe/src/markets/perturb.rs
0d09afc3ffc31f8175bfc36afa9770445061eaafda74811db1ab23995d959ff5  crates/probe/src/bin/markets.rs
c9670ea2265542ee796650e758b1c759d7538a8bbe8627ddf84f2232f5858b35  crates/probe/src/bin/markets-tape.rs
```

## 3. Readings the build takes, declared before it

SPEC is registered as written. Where its text leaves a choice, is inexact, or leaves out a ruling
that binds it, the build takes these readings. Each follows the registered mirror, so none moves a
prediction of SPEC §9.

- **The field and the dial.** `exit.pace` is `Some((adjust: "adjust.participation.workers",
  share: S/N))` on the workers' exit block, last, and absent (not written) on every other tape.
  The dial `adjust.participation.workers` (1.3, `RatePerYear`) is written after the techniques'
  `adjust.technique.*` in C2m, for a paced instance only, so `--set adjust.*=f` scales it with
  them, as the mirror's `over_for("adjust", f)` scales `adjust.workers`.
- **The genesis share** is unit 1e's supply S at the point over N per tick, as the harness's
  oracle gives it; the mirror's is its own f64 oracle's (within 1e-13). E0 seeds the mirror from
  the tape's genesis, share included, as the commons' E0 seeded prices and coins.
- **The run grammar.** `part.workers*F` sets the genesis share to min(F·S/N, 1), as the mirror's
  `displace`; `part.workers=V` sets V exactly. Both are refused at an instance without a pace.
- **The pace family's start distance** is the largest D̂ of its first scored year, and a run of it
  is VACUOUS where that is at most 1, as the prediction reads it (`predict.py`, `fy`) and as Tier
  3S is read (the commons registration §3): `markets family pace` sets it, as `family tier3s`
  does. A `part.workers` run outside the family without `--first-year` reads
  |ln(genesis share / S/N)| over the tolerance, as a stock's factor. The prediction reads E4's
  stocks family by the same first-year rule (`fy`); the wave's job list runs it with
  `--first-year` (every one of its 82 runs is predicted CONVERGED either way).
- **The pace family's order** is SPEC §8 E4's: `part.workers*0.1`, `*0.5`, `*2`, `=0`, `=1`
  (`runs.jsonl` holds them in the pool's order).
- **Names**, as the commons registration read them: a desk's coin `coin.desk.<d>*F` (the
  mirror's `coin.<d>*F`, which `runs.jsonl` carries as `engine_name` for Tier 3S); the history
  `cycle(land.mach,1500,80)` and `cycle(commons,1500,80)` (the mirror's `cycle(land.mach)` and
  `cycle(exit.To)`); the commons' coefficient `commons` (the mirror's `exit.To`).
- **The tapes** are `tapes/markets-c1p.ron` and `tapes/markets-c2p.ron`, `markets-tape --inst
  c1p|c2p`'s output. C1PN is `--inst c1pn`, C1N's χ_max 0.25 with the pace, its own oracle point
  and no committed tape, run at C1's L as the mirror ran it.
- **The readouts.** The CSV's `part_workers` is the workers' state share, the hours offered over N
  (at the commons without a pace it is the rule's, as before); `part_target` (paced instances
  only) is the rule's F\* = hours/N at the tick's posted prices, through `workers_participation`.
  The regime, the shadow rent and T_p stay the rule's at posted prices. `stats.tsv` adds, at a
  paced instance only, `pace.gap_max` (the largest |ln(F/F\*)| over the scored run), `pace.low`
  (the least F/F\*) and `pace.zero_hours` (the scored ticks with labour's supply 0).
- **E0's comparison** adds the paced share (the mirror's `st["F"]` after the tick) and the rule's
  F\* (the mirror's `exit_rule` hours over N) to the commons' E0's quantities, and its eleven runs
  are SPEC §8's, the `rate.*=0.9` run with every price rate × 0.9 in both (`--set rate.*=0.9`; the
  mirror's `over` on every market's rate).
- **Heads of 0.** The rule divides by N (F\* = hours/N), as the mirror; no instance has N = 0, and
  a run that sets it errs rather than guessing a share.

## 4. The predictions

Quoted from SPEC §0 and §9:

> **GO: participation at a rate** (the workers' `exit.pace`). The share of heads offering hours
> moves a share a = 1 − exp(−1.3/tpy) of its gap to the participation rule's share each tick
> (0.02469 at 52 a year: half the gap closes in 27.7 ticks), instead of jumping to it. Only the
> hours lag; the plots follow the rule at posted prices. The rest point is the oracle's, unchanged,
> so no oracle addendum is needed. With the field absent the role is P2.3's bit for bit.

> | | predicted | Tier 1 | Tier 2 | Tier 3 | Tier 3 at 10·L | Tier 3S (L; 10·L) | E10: 17 settings × 43 |
> |---|---|---|---|---|---|---|---|
> | **C1P** | **GO with margin** | 30/30 (27 + 3 slack) | 42/42 (39 + 3) | 43/43 (40 + 3) | 43/43 | 24/24; 24/24 | 731/731 |
> | **C2P** | **GO with margin** | 30/30 (27 + 3) | 38/38, 4 VACUOUS | 41/41, 2 VACUOUS | 41/41, 2 VACUOUS | 24/24; 24/24 | 697/697, 34 VACUOUS |

Mode A passes at 12, 52 and 365 a year (largest gaps 2.2e-16 to 2.2e-14). Every family run of C1P
and C2P converges or is VACUOUS by construction (O103), with no tick of zero hours: 1,893 and 1,831
CONVERGED, 0 and 62 VACUOUS. C1PN's battery: 109 CONVERGED, 6 VACUOUS. The controls: the
registered C1's 18 dial-neighbourhood trap runs DIVERGED at SPEC §1.2's ticks (harness's
reference), its other 713 CONVERGED; C1P at every tilt 2 loses `p[mach]*0.5`, `JB(2)` and
`N(0.5)` (DIVERGED at 308, 308 and 314), C2P `p[mach]*0.5` and `N(0.5)` (314, 305). The largest
root per tick is 0.986127 (C1P) and 0.986524 (C2P) at the base.

**The verdict** per instance is the commons frame's §5.5 with Tier 3S: GO where mode A passes and
every non-vacuous run of Tiers 1–3 and 3S is CONVERGED, with Tiers 3 and 3S CONVERGED again at
10·L; **with margin** where, in addition, every Tier-3 run of E10 is CONVERGED or VACUOUS.

**What would refute it** (SPEC §8), each sending the remedy back to the scan with home output sold
as the alternative: a class change in E3, E4 or E10 at C1P or C2P; an engine run at a registered
target resting off the oracle's point, or a paced share ending more than 1e-12 from the rule's;
an E0 parting not explained by a named rounding; the trap (a runaway) in any C1P or C2P run of
E3–E8 or E10, or any tick with no hours offered in them; none of the controls' predicted trap runs
(E9) falling into it.

## 5. Decisions and open items

SPEC §10's proposals, numbered: decisions 404 (TR1: O100's remedy is participation at a rate),
405 (TR2: the paced instances are new, C1P, C2P and C1PN), 406 (TR3: the dial
`adjust.participation.workers` at 1.3 a year), 407 (TR4: the engine run E0–E10 and the margin
line) and 408 (TR5: the controls); open items O113 (O-TR1: the trap's attractor remains), O114
(O-TR2: the machine's absorbing zero), O115 (O-TR3: home output sold, unanswered), O116 (O-TR4:
the pace's rate has no data) and O117 (O-TR5: a storable exit good is unstable at C1). STATE.md
has them in full, each open to veto.
