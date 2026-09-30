# The open-commons instance's registration (P2.3, C1 and C2)

Dated 2026-09-30. Step P2.3.5 on branch `phase2-proper` (worktree `D:/rustyecon-wt/p23`), on top of
`49dd068` (the wall instance's E0–E2), clean. It registers the mirror's predictions for the engine
run of Phase 2 proper's open-commons instances, C1 and C2, before any engine code for them exists
(R5): at this commit `BasketWorkers` has no `exit` field, `crates/probe` has no commons instance,
and `tapes/` has no commons tape. It is written by `D:/rustyecon-p23/build-commons/make_registration.py`,
which checks the frame's `SHA256SUMS` entry by entry and copies every quote below from its file.
Any later change is a new dated amendment with its own sha256 (decision 282); earlier results stay
reported.

## 1. What is registered

The frame, in [docs/probe/commons/](../../commons/), as its author left it in
`D:/rustyecon-p23/frame-commons/` (label `frame-commons`). The frame wrote some files on Windows,
with CRLF line endings (`SPEC.md`, `points.json`, `targets_table.md`); the repository stores text
with LF (`.gitattributes`), so each copy here is the same text with LF. Both sha256s are listed:
the registered one (as in `SHA256SUMS`) and the committed one. They are equal where the frame
wrote LF. Nothing else differs, and the scorer parses either.

| file here | the frame's file | sha256 as registered | sha256 as committed (LF) | what it is |
|---|---|---|---|---|
| `SPEC.md` | `SPEC.md` | `60f21f56bb5c94e89cdbcd93709207296b5fb0c6a432c57e96ca330b1b05b40d` | `74ae87cdbe6eeba87de472f2aca582b28559dde3aa83cdff4ebaca1ced504cc3` | the frame |
| `SHA256SUMS` | `SHA256SUMS` | `50e4d06972569c79d300f9be7876aaa77a47c3809313b5a48718e7885baa7ee0` | `50e4d06972569c79d300f9be7876aaa77a47c3809313b5a48718e7885baa7ee0` | sha256 of every file of the frame |
| `registered/battery_v3.jsonl` | `model/runs/battery_v3.jsonl` | `f820fdad358757fe13bf3a2ce58074911f18b69009f633a6fe3de8b4e3231b25` | `f820fdad358757fe13bf3a2ce58074911f18b69009f633a6fe3de8b4e3231b25` | Tiers 1-3, Tier 3 at 10·L, stocks and enclose, run by run, on the final `cm.py` (SPEC §10) |
| `registered/families_v2.jsonl` | `model/runs/families_v2.jsonl` | `ca314ad22f0dbfe1dcfcadaf8e2ab7c57272e1cf5408d8180b9dfd7282e9d567` | `ca314ad22f0dbfe1dcfcadaf8e2ab7c57272e1cf5408d8180b9dfd7282e9d567` | joint2, joint4, basin, Hold, tilt 1 and 12 a year, run by run |
| `registered/tpy_v2.jsonl` | `model/runs/tpy_v2.jsonl` | `2ad9dd827b9780f0446f0e5400c63b85aba77a4d62f06c72085024d3462741a1` | `2ad9dd827b9780f0446f0e5400c63b85aba77a4d62f06c72085024d3462741a1` | Tiers 1-2 at 12 and at 365 a year, run by run |
| `registered/negctl.jsonl` | `model/runs/negctl.jsonl` | `a473cc636ab4c7d003b44339d8e95e2aa7a16dbd103aa27444bf81fd8d702674` | `a473cc636ab4c7d003b44339d8e95e2aa7a16dbd103aa27444bf81fd8d702674` | the negative control, χ_max 0.25, run by run |
| `registered/history.json` | `model/runs/history.json` | `27ec2ee2639b22a9f69e0d248b46be7b567e1a7bf7cb0b811503282ebdc008a7` | `27ec2ee2639b22a9f69e0d248b46be7b567e1a7bf7cb0b811503282ebdc008a7` | the history family, window by window |
| `registered/lin.json` | `model/lin.json` | `bbeb752d542236bc7a78e263d5d878d071d767e87aac676d7d23a5f90aeff153` | `bbeb752d542236bc7a78e263d5d878d071d767e87aac676d7d23a5f90aeff153` | the largest root per tick and the kick bars, by target |
| `registered/elasticity.json` | `model/elasticity.json` | `21970bb5e95ecab27bd77cfcc1b6663d977b64c4eb87f4c6d4f5d73acb9a5115` | `21970bb5e95ecab27bd77cfcc1b6663d977b64c4eb87f4c6d4f5d73acb9a5115` | τ and L by instance and tick length |
| `registered/predictions.json` | `model/predictions.json` | `65ff0a57008b42476c7ae003e282e63ebfc2e0bd9ca8b724315422c87819173f` | `65ff0a57008b42476c7ae003e282e63ebfc2e0bd9ca8b724315422c87819173f` | SPEC §6's tables, as numbers |
| `registered/predictions.out` | `model/predictions.out` | `fe968923126a3573bebdab8df759a69281704355e8d111c5d5576d63405f5588` | `fe968923126a3573bebdab8df759a69281704355e8d111c5d5576d63405f5588` | the same, as text |
| `registered/points.json` | `solve/points.json` | `89f4986dca2d87e0e06403ca18b26ffc0b32e5390a6c66baf96f47993af178d8` | `f668df88d832d7c4f733124b8fa5fc007ae7d545d5b22858905d9d982d6e62d7` | the 50-digit solve at the 26 targets |
| `registered/targets_table.md` | `solve/targets_table.md` | `4479797f338fdbdcfb61f74f47b6a82a921da5d2d73780ac850a1e30b960f376` | `693c97000373db757edbb313e440e611f2cc951792596d33a90295d42e9cd6a9` | SPEC §2.3's table |

`SHA256SUMS` has 97 entries, each verified on 2026-09-30 against the frame's files.
The scorer (SPEC §5.5, decision 311) reads `registered/` from this copy, and Tier 3S's runs from
[tier3s/](tier3s/) (§3 below).

The mirror the predictions come from stays in `D:/rustyecon-p23/frame-commons/`, as P2.2b's and the
wall's did (sha256, from `SHA256SUMS`):

```
4d28d20bb6475ea93050743e718c54bab3e61fa3c92c10cac9f9b7a4dd9bdd86  model/cm.py                (the commons mirror, E0's map: `tick`, `exit_rule`, `genesis`, `oracle_at`)
c3ec4a1c346cfa7475b659afbd0ec18788bf5ff43c84ed9becfb140d5c58dc29  model/battery_c.py         (the runner: run grammar, observables, classes, readouts)
e217eec840eb895aa05fbcd2819d393f6dcc2aed143e4e5d6c563191489ab3ab  model/run_battery_c.py     (the sets and families)
480f0a85f9e23e05ff8fe13db36814085036a5f0520f547cfc25f5cb6c2e15f7  model/instances.py         (C1 and C2's commons)
dbd50ad9c1e948aa9bb23545df2dfcb89923cd0d45a0106e8ccfcbbe2798538c  model/p21/mm_carry.py      (P2.1's trace mirror, copied unedited)
528756c1925a6a0a3cdd8dc906ac5a758416e7539e62a9e2afe9a93de3dab407  model/p21/mm.py            (P2.1's predictor's mirror, copied unedited)
02dcb36f394133e96a555b7e32f6c86b4bb264afd86f8c1257c3b01285bbf4c8  model/negctl.py            (the negative control)
417e8bc0dc470abf3485ffe0078924ac5f86aa496160d820e7c9f2e65befafaf  model/history.py           (the history family)
84d9179f6cf7bfd4ebbf3572d2001daf6934753c5b2dd8dc1a97c539feb0e549  model/lin_c.py             (the largest root and the kick bars)
bc0b86aa0135236034a53e63c580ee8e8871edd39f0ab73858deedde691a9ec2  model/elasticity.py        (the elasticity probe)
d600a61b5ee95a14f3db2d8fe58c7f87a76752e4ba28fc0110900617425f03a8  model/nest.py              (the nesting and rest-point checks)
bc789433df0005233a6cf491f31854d027b233d4c7fedfdba23060f1a0c303e5  model/predict_summary.py   (SPEC §6's tables)
9e8b3d1039c20e16c0f6510dc06d0ac6e4d80d608877a16f21a1609431c75f61  model/scan.py              (the idle-market scan, SPEC §1)
0d790806f2294aec2483d577c45306cd9aa8057d8f9034c7994cc7f3ef14e9f9  solve/csolve.py            (the 50-digit solve that reads no oracle code)
```

## 2. The order of work

1. The frame was written on 2026-09-30 in scratch (label `frame-commons`), reading the worktree at
   `f7d1eae` and changing nothing. P2.3.0 (`f0c6667`) and the wall's P2.3.1–P2.3.4 landed during
   and after it; none touches the many-market roles' households or the markets harness's P2.1
   instances. No engine code for C1 or C2 and no engine run exist.
2. This commit fixes the registration in the repository, adds Tier 3S's mirror runs (§3), and
   STATE.md numbers the frame's proposals (§5).
3. Next: the build (the `exit` block on `BasketWorkers`, its load checks, the plots' land buy, the
   instances on unit 1e, the generator and the tapes, the harness's grammar and readouts, the
   tests, `docs/probe/COMMONS-RULES.md`), with every committed tape's text, hashes and streams
   unchanged. Then E0, the trace diff against `cm.py`, before any scored run. A real departure
   found by E0 is a dated amendment committed before scoring. The scorer is committed before the
   wave (decision 311).

The sources at the base (sha256), which the build will change:

```
cc892197e54117e97fe88f3390275525778bd75de61d40894ac52ab01f8c484e  crates/agents/src/roles/many/spec.rs
768f445647db3e8624a058ff537df83fb82137fefa03bb02c4055258033e6641  crates/agents/src/roles/many/rules.rs
1f0d95ef6e0fd8c0a8d16fd917c3857fc257d67cec1f34d95c79c12d192cd938  crates/agents/src/cast.rs
13ab7218468288e30613db6caa8000bbe4f3e3b6c1e48133e706ed9a6f6f95a3  crates/agents/src/lib.rs
a651d932748465a33b3a4b4680cbc13ef106e862692d1fe08618f4691e12c5e6  crates/probe/src/markets/instance.rs
9d300436faf172af1d1563f82e972c1ff85235ca38b75c67769ec420a2da495a  crates/probe/src/markets/setup.rs
0e45c560293892ef662405301c5b7e704aef22d3b93b6b9641c67a8a77710b14  crates/probe/src/markets/harness.rs
753595055896b296c83c376516870707e23ace8c141d515562885b6c85438b86  crates/probe/src/markets/perturb.rs
f79c8c7c7f113775e02bdd053731dd59501c86420acfa3f54af7af2e66186a5a  crates/probe/src/bin/markets.rs
1bd58cedf057dfa629d84c0961d7781807527d9b546fd3e8cfd727074b3b0a90  crates/probe/src/bin/markets-tape.rs
```

## 3. Readings the build takes, declared before it

SPEC is registered as written. Where its text leaves a choice, is inexact, or leaves out a ruling
that binds it, the build takes these readings. Each follows the registered mirror, so none moves a
prediction of SPEC §6.

- **Tier 3S joins the verdict (decision 368).** 368 puts a stocks tier in the new instances'
  verdicts, P2.2a's Tier 3S (decision 229), as the wall's did (397). SPEC §5.5's verdict leaves it
  out, and SPEC §5.4 reports a stocks family instead. The registration follows 368. Tier 3S is
  the engine's `tier3s` list: each desk's stock and coin, the workers' coin and the provider's
  coin, at ×0.5 and ×2, 24 runs an instance, with the start distance the largest D̂ of the first
  year. 23 of the 24 are in the frame's stocks family (all CONVERGED there); `coin.workers*0.5` is
  not. So Tier 3S was run on the frame's mirror, unedited, before any code, by
  [tier3s/tier3s_c.py](tier3s/tier3s_c.py). It runs `battery_c.run` and measures the first year's
  D̂ with the mirror's own tick map and observables. The prediction, from
  [tier3s/tier3s_summary.out](tier3s/tier3s_summary.out):

  | | Tier 3S at L | at 10·L | ticks to tol, median (slowest) | dead ticks, worst | lowest baskets | first-year D̂, least |
  |---|---|---|---|---|---|---|
  | C1 | 24/24 CONVERGED | 24/24 | 379 (534) | 50 | 0.286 | 49.7 |
  | C2 | 24/24 CONVERGED | 24/24 | 340 (543) | 43 | 0.286 | 63.6 |

  So the verdict is SPEC §5.5's with Tier 3S: GO where mode A passes and every non-vacuous run of
  Tiers 1–3 and Tier 3S is CONVERGED, with Tiers 3 and 3S CONVERGED again at 10·L. Tier 3S's
  files, with their sha256 (in scratch, then as committed; all were written with LF):

  ```
  041d5c9f9449072212db0ccf24b278762952f243bd18e86176fad09ff93d3d89  041d5c9f9449072212db0ccf24b278762952f243bd18e86176fad09ff93d3d89  tier3s/tier3s_c.py  (the script, on the frame's mirror unedited)
  9499e0fe39104189aab687af6d4e556ba6bb72e5e35cbe03e298ad5ad282184c  9499e0fe39104189aab687af6d4e556ba6bb72e5e35cbe03e298ad5ad282184c  tier3s/tier3s.jsonl  (its runs: C1 and C2, at L and 10·L)
  56eeda9297cad8b5a8c9fb4defd9718fd82d3c2cfa648671fb419aa96cb9e44e  56eeda9297cad8b5a8c9fb4defd9718fd82d3c2cfa648671fb419aa96cb9e44e  tier3s/tier3s_summary.py  (the summary)
  d6fd46bc877c3fc0c3b201c65278c452ffad3590c62a19fb16cd868de4b84dff  d6fd46bc877c3fc0c3b201c65278c452ffad3590c62a19fb16cd868de4b84dff  tier3s/tier3s_summary.out  (its output)
  ```

- **The tapes** are `tapes/markets-c1.ron` and `tapes/markets-c2.ron`, written by `markets-tape
  --inst c1` and `--inst c2`. SPEC §3.7 names them `markets-C1.ron` and `markets-C2.ron`; every
  markets tape is `markets-<id>.ron` in lower case. The negative control (SPEC §5.4 item 8) is
  `--inst c1n`, C1 with `inst.chi_max` 0.25 and its own oracle point, run at C1's L as the mirror
  ran it (`negctl.py`); it has no committed tape.
- **Run names.** The battery is named and ordered as `battery_c.run_list` writes it (`p[labour]*1.05`,
  market by market), so the engine's list equals the registered runs name for name. The commons'
  cost coefficient is `commons` (SPEC §2.6 and §3.8; param `inst.commons`, a year), which the mirror
  names `exit.To`, with the mirror's values (`round(v·f, 12)`: C1 26.73, 21.87, 48.6, 12.15; C2
  34.32, 28.08, 62.4, 15.6). The stocks family names a desk's coin `coin.desk.<d>*F`, the engine's
  actor, where the mirror writes `coin.<d>*F`. The basin's factors are the mirror's `1.05**j`
  written to 12 significant digits (`%.12g`), which is the factor the mirror ran. The history
  family is `cycle(land.mach,1500,80)` and `cycle(commons,1500,80)`.
- **A share scaled past 1.** The mirror's displacement sets a desk's genesis share to
  min((1 − x\*)·F, 1) (`battery_c.displace`); at C1 and C2 only the basin's `s[food]*F` runs with
  F > 1/(1 − x\*) reach it (j ≥ 29 at C1, j ≥ 28 at C2). The harness does the same at C1 and C2, and
  P2.1's instances keep their refusal of a share above 1 (their basin skips those runs).
- **`joint(F,SEED)` at C1 and C2** draws its factors in the mirror's order (labour, land, the
  categories, the type), then the shares, as `battery_c.displace` does, so O102's per-seed
  predictions apply; E0 compares the harness's draws with the mirror's. P2.1's instances keep their
  order.
- **`enclose=F@genesis|dated`** moves F·T_o from the commons to the enclosed land in one tick: two
  `SetParam` changes at the same tick, `inst.commons` to T_o − F·T_o and `inst.land` to T + F·T_o,
  per year, as `cm.shocked` computes them. At genesis both are the tape's values from tick 0, and
  genesis stays at the registered point.
- **Targets.** Labour's target volume is the oracle's supply S (unit 1e's `n_s` at the point), and
  land's is T per tick of the instance in force (T moves under `enclose`), as `battery_c.targets_of`
  reads them. A cost shock's start distance at C1 and C2 is the largest |ln| over every observable
  between the two targets over the tolerance (`battery_c.run`, and the wall's `target_distance`), so
  C2's commons × 1.1, × 0.9 and × 2 read 0 and are VACUOUS. The workers' genesis coin reads S and
  T_p from the oracle's point (SPEC §3.8).
- **The workers' state.** SPEC §3.1 writes the share in `WorkersState` as hours/N. The build stores
  the regime's F: F(e₀) in Commons, (N − T_o/h)/N when Crowded, F(e_r̂) when Enclosed or split,
  F(p_g·s̲) with no plot. That is hours/N to one rounding, and with the exit switched off it is
  P2.1's share to the bit, so the nesting of SPEC §3.7 test 2 holds for the state too. No quantity
  of the mirror reads it.
- **The readouts** (SPEC §3.8) go to `stats.tsv` as `commons.*` lines, as the wall's `wall.*` do;
  `summary.tsv` keeps P2.1's columns for every instance. The split (a plot does not pay at r) is
  counted as its own regime and also with Crowded, as the mirror's readout counts it.
- **E0's runs** are SPEC §5.5's list: C1 `hold`, `p[labour]*2`, `JB(0.5)`, `commons=48.6@genesis`
  and `commons=12.15@genesis`; C2 `hold` and `p[labour]*2`; and `c1n`'s `p[mach]*0.5` until its
  runaway or 2,000 ticks. "The harness's first joint draw" is read as the displaced genesis of
  `joint(2,1)` and `joint(4,1)` at C1 and C2 against `battery_c.displace`'s.

## 4. The predictions

Quoted from SPEC §0:

> - **The prediction: C1 and C2 GO.**
>   - Mode A passes at 12, 52 and 365 a year.
>   - Every non-vacuous run in Tiers 1–3 and in Tier 3 at 10·L is CONVERGED: C1 115/115, C2
>     109/109, and 6 of C2's commons shocks are VACUOUS by construction.
>   - The largest root per tick is 0.98538–0.99139, and every kick bar passes.
>   - Ticks to tolerance, median by Tier 1/2/3: C1 319/372/508, C2 315/360/514. The I1 control
>     on the same mirror gives 324/381/525.
> - **The subsistence trap (§6.8), a prediction of the families.** It is not in the verdict.
>   - With exit valued at food's posted price and food made with labour, the agents have a second,
>     absorbing state. Nobody works, food is not supplied, and every price inflates together.
>   - C1 reaches it in 4/60 joint2 and 12/40 joint4 runs, in 89 of 430 basin runs, in 3 tilt-1
>     Tier-3 runs, and in the land.mach history's fifth window at tick 288. C2 reaches it in 8/40
>     joint4 runs and 77 of 430 basin runs.
>   - I1 never does: 0 of 530 on the same families.
>   - At χ_max 0.25, a negative control that changes one param, 9 of C1's Tier-3 runs fall in.

Quoted from SPEC §5.5 and §5.6, in full: the engine run and how close it must be.

> ### 5.5 The engine run, in order (decision 371)
>
> Each step is registered here before any code. The scorer is committed before its wave (decision
> 311).
> - **E0: the build's trace diff.** `cm.tick`, with the genesis carry, against the engine for 2,000
>   ticks: C1 `hold`, `p[labour]*2`, `JB(0.5)`, `commons=48.6@genesis` (to Commons) and
>   `commons=12.15@genesis` (Enclosed, plots rented); C2 `hold` and `p[labour]*2`; the negative
>   control's `p[mach]*0.5` up to its runaway. They must agree within 1e-12 in log. A parting blocks
>   scoring until it is explained in the registration, as P2.1's budget-chain ulp was. E0 also
>   compares the harness's first joint draw with the mirror's.
> - **E1: nesting.** Every pin of §3.6, and tests 1–2 of §3.7.
> - **E2: mode A and the base kick sets**, at C1 and C2, 52 a year (and 12 and 365, reported).
> - **E3: the verdict battery.** Tiers 1–3 at L, and Tier 3 at 10·L, at C1 and C2, with a kick set
>   at every target.
> - **E4: stocks** (first, O22), then **E5: joint2, joint4 and basin**, **E6: history**,
>   **E7: Hold and tilt 1**, **E8: tick length**, **E9: enclose and the negative control**.
>
> **The verdict**, per instance, is MARKETS-SPEC §7.9's. GO: mode A PASS, and every non-vacuous run
> in Tiers 1–3 CONVERGED, with Tier 3 CONVERGED again at 10·L. LOCAL: Tiers 1–2 CONVERGED, some Tier-3
> run not. NO-GO: otherwise.
>
> ### 5.6 How close the engine must be to the mirror
>
> As P2.1's and L0's engines matched their mirrors:
> - every class exactly;
> - ticks to tolerance within 10%, with up to three runs an instance within 25%;
> - lowest baskets and cleared volumes within 0.05 absolute where the mirror's is above 0.1;
> - dead ticks within 10% or 5 ticks;
> - regime ticks and switches within 10% or 2;
> - r_o at the end within 1e-9 relative (Crowded targets);
> - runaway ticks of the trap within 5%.

Quoted from SPEC §6, in full. This is what the engine run is scored against, with Tier 3S (§3):

> ## 6. Registered predictions
>
> The conditions are §0's, the mirror's runs in `model/runs/`, and `model/predictions.out`.
>
> ### 6.1 The verdicts
>
> | | predicted verdict | mode A (52/yr) | Tier 1 | Tier 2 | Tier 3 | Tier 3 at 10·L |
> |---|---|---|---|---|---|---|
> | **C1** | **GO** | PASS, 2.4e-15 | 30/30 (27 + 3 slack) | 42/42 (39 + 3) | 43/43 (40 + 3) | 43/43 |
> | **C2** | **GO** | PASS, 4.4e-16 | 30/30 (27 + 3) | 38/38, 4 VACUOUS | 41/41, 2 VACUOUS | 41/41, 2 VACUOUS |
>
> Mode A also passes at 12 a year (1.6e-14, 2.8e-14) and 365 (3.4e-15, 1.5e-14), with every fill at
> least 1 − 3e-14.
>
> ### 6.2 The largest root per tick and the kick bars
>
> `model/lin.out` (8 directions, ticks 10,000–40,000). Every kick bar PL^(0.9·L) ≤ 1e-3 passes.
>
> | | base | range over the targets | slowest target |
> |---|---|---|---|
> | C1 | 0.986116 (half-life 50 ticks) | 0.985378–0.991388 | land.mach × 0.5 (Enclosed), 80 ticks |
> | C2 | 0.985988 (49) | 0.985555–0.987888 | land.mach × 0.5, 57 ticks |
>
> (P2.1's I1: 0.98661.)
>
> ### 6.3 Speeds and paths, tier by tier
>
> Ticks to tolerance are for non-slack CONVERGED runs. "Lowest baskets" is baskets eaten over Y\*.
> "Provider coin" is its lowest over its genesis coin.
>
> | | Tier | ticks to tol, median (slowest) | years | peak D̂ median / worst | dead ticks median / worst | worst buyer fill | lowest baskets (ticks with none) | transfer short in (most ticks) | provider coin low |
> |---|---|---|---|---|---|---|---|---|---|
> | C1 | 1 | 319 (444) | 6.1 (8.5) | 71 / 504 | 0 / 0 | 0.524 | 0.632 (0) | 1 run (11) | 0.446 |
> | C1 | 2 | 372 (557) | 7.2 (10.7) | 311 / 2,184 | 0 / 96 | 0.085 | 0.131 (0) | 10 (94) | 0.247 |
> | C1 | 3 | 508 (828) | 9.8 (15.9) | 1,141 / ∞ | 30 / 183 | 0 | 0 (8) | 30 (293) | 0.047 |
> | C2 | 1 | 315 (494) | 6.1 (9.5) | 72 / 419 | 0 / 0 | 0.597 | 0.688 (0) | 1 (2) | 0.470 |
> | C2 | 2 | 360 (547) | 6.9 (10.5) | 232 / 1,739 | 0 / 79 | 0.134 | 0.205 (0) | 10 (73) | 0.278 |
> | C2 | 3 | 514 (641) | 9.9 (12.3) | 1,068 / ∞ | 23 / 153 | 0 | 0 (8) | 27 (196) | 0.100 |
>
> A peak D̂ of ∞ is a tick where some market does not trade (5 C1 and 4 C2 Tier-3 runs). The
> slowest runs are C1 b.food = 1.2 (828 ticks) and p[mach] × 0.5 (739), and C2 p[land] × 0.5 (641).
> The most dead ticks are C1 JB(2) 183 and p[mach] × 0.5 181, and C2 JB(2) 153.
>
> **The lowest cleared volume over the oracle's, by market** (Tier 1 / Tier 2 / Tier 3):
> - C1: labour 0.863 / 0.213 / 0.016; land 0.849 / 0.544 / 0.203; manufactures 0.632 / 0.131 / 0;
>   food 0.639 / 0.150 / 0; care 0.713 / 0.143 / 0; shelter 0.640 / 0.156 / 0; mach 0.604 / 0.113 / 0.
> - C2: labour 0.759 / 0.349 / 0.043; land 0.875 / 0.607 / 0.278; manufactures 0.688 / 0.205 / 0;
>   food 0.697 / 0.236 / 0; care 0.724 / 0.249 / 0; shelter 0.703 / 0.244 / 0; mach 0.658 / 0.176 / 0.
>
> **The regime's readouts** (the runs that visit each regime; switches median / most; r_o/r range;
> T_p's largest):
>
> | | Tier | Commons | Crowded | Enclosed | switches | r_o/r | T_p max |
> |---|---|---|---|---|---|---|---|
> | C1 | 1 | 0 | 30 | 4 | 0 / 2 | 0.168–1 | 0.0099 |
> | C1 | 2 | 13 | 38 | 14 | 0 / 6 | 0–1 | 0.057 |
> | C1 | 3 | 29 | 39 | 29 | 3 / 6 | 0–1 | 0.225 |
> | C2 | 1 | 30 | 0 | 0 | 0 / 0 | 0 | 0 |
> | C2 | 2 | 42 | 0 | 0 | 0 / 0 | 0 | 0 |
> | C2 | 3 | 41 | 2 | 0 | 0 / 0 | 0–0.731 | 0 |
>
> At every Crowded target the shadow rent ends at the oracle's r_o (C1's base 0.4379200472667323).
> At every Commons target it ends at exactly 0, with the commons partly idle.
>
> ### 6.4 O14, like for like with I1 (the same mirror and runner)
>
> | | Tier 1 / 2 / 3 median ticks | Tier 2 worst peak D̂ | Tier 2 lowest baskets | Tier 3 dead ticks, worst | largest root |
> |---|---|---|---|---|---|
> | I1 (dependence form) | 324 / 381 / 525 | 1,288 | 0.276 | 145 | 0.98661 |
> | C1 (commons full) | 319 / 372 / 508 | 2,184 | 0.131 | 183 | 0.98612 |
> | C2 (commons with room) | 315 / 360 / 514 | 1,739 | 0.205 | 153 | 0.98599 |
>
> The commons converges at I1's speed. Its paths are deeper: Tier 2's worst baskets fall to 0.47
> of I1's in C1 and 0.74 in C2, and Tier 3 has 26% (C1) and 6% (C2) more dead ticks at worst.
>
> ### 6.5 The families
>
> | family | C1 | C2 | I1 (control) |
> |---|---|---|---|
> | stocks (41) | 41 CONVERGED; median 419 ticks | 41; 444 | not run |
> | joint2 (60) | **56 CONVERGED, 4 DIVERGED** (seeds 3, 12, 34, 39) | 60 | 60 |
> | joint4 (40) | **28, 12 DIVERGED** (seeds 3, 6, 10, 12, 17, 18, 22, 26, 28, 31, 34, 39) | **32, 8 DIVERGED** (3, 10, 12, 18, 21, 22, 34, 39) | 40 |
> | basin (430) | **341, 89 DIVERGED** | **353, 77 DIVERGED** | 430 |
> | Hold (115) | 115 CONVERGED | 109, 6 VACUOUS | — |
> | tilt 1 (115) | **112, 3 DIVERGED** (Tier 3: p[mach] × 0.5, JA(0.5), JB(2)) | 109, 6 VACUOUS | — |
> | 12 a year, Tiers 1–2 (72) | 72; median 908 ticks = 75.6 years | 68, 4 VACUOUS; 80.2 years | — |
> | 365 a year, Tiers 1–2 (72) | 72; median 2,174 ticks = 6.0 years | 68, 4 VACUOUS; 5.9 years | — |
> | enclose (4) | 4; 323 ticks | 4; 324 | — |
> | negative control, χ_max 0.25 (115) | **100, 9 DIVERGED** (Tier 3), 6 VACUOUS | — | — |
>
> **The basin**, converging from ×/÷ at j contiguous from 1 (`basin_table.py`):
>
> | | w | r | food's price | machine's price | food's technique |
> |---|---|---|---|---|---|
> | C1 | × 0.231 to × 8.15 | ÷ 8.15 to × 2.407 | ÷ 8.15 to × 2.653 | × 0.481 to × 8.15 | the whole range |
> | C2 | × 0.326 to × 8.15 | ÷ 8.15 to × 3.920 | ÷ 8.15 to × 3.072 | × 0.359 to × 8.15 | the whole range |
>
> The Tier-3 factors (× 2, × 0.5) lie inside every basin. The boundary is always on the side that
> lowers the wage against food, which raises the exit's value against work: labour cheaper, food
> dearer, land dearer, machines cheaper (so the desks automate and hire fewer hours).
>
> ### 6.6 The history family
>
> | | cycle(land.mach, 1500, 80) | cycle(commons, 1500, 80) |
> |---|---|---|
> | C1 | **runaway in window 5 at tick 288** (after 0.2 → 0.4, tick 7,788 of the run); windows 0–4 end within tolerance, median 366 ticks | 81/81 windows within tolerance; median 497 ticks; the targets are Crowded 17, Commons 32, Enclosed 32 |
> | C2 | 81/81; median 551 ticks | 81/81; median 0 ticks (every commons shock but × 0.5 leaves the point where it is), at most 315 |
>
> ### 6.7 What the rule does through the regimes
>
> C1's commons shocks move the equilibrium across all three regimes, and each converges:
> - commons × 1.1 and × 2 → Commons, 438–443 ticks;
> - commons × 0.9 → Enclosed with T_p up to 0.039, 324 ticks;
> - commons × 0.5 → Enclosed with T_p up to 0.225, 448–449 ticks;
> - C2's commons × 0.5 → Crowded, 315 ticks, r_o ending within 5e-15 of the oracle's 0.46595.
>
> Every CONVERGED battery run ends in its target's regime. There, the shadow rent is within 1e-9
> of the oracle's at a Crowded target, exactly 0 at a Commons target and exactly r at an Enclosed
> one. The shadow rent needs no market: it is exactly 0 wherever the commons has room.
>
> ### 6.8 The subsistence trap
>
> **The mechanism** (`model/trace_cycle.py` on C1's history, window 5; and `trace1.py` on the first
> choice's p[food] × 0.5):
> 1. After a shock that leaves the wage low against food's price, the exit's money value
>    e₀ = p_food·s₀ approaches the wage (e₀/w 0.80 at the window's first tick).
> 2. Where the supply is elastic (Commons or Enclosed), participation falls: hours go from 0.43 to
>    0.10 in 18 ticks.
> 3. So food's output falls, to 0.10 of its target by tick 18 and 0.013 by tick 60, and its price
>    rises.
> 4. That raises e₀ further. By tick 60, e₀/w ≈ 1: nobody works.
> 5. From then on no good is supplied. Every one-sided price steps up under `Saturate`, the wage
>    and food's price together. e₀/w stays at 1.01–1.12, the provider's coin drains, and the posted
>    prices cross the runaway bound after 254–321 ticks.
>
> Food cannot be bought, so the exit is worth more than work, so nobody makes food. It is a
> coordination failure with no way back under these rules. It mirrors the loop's absorbing zero
> (O21), with the exit good in place of the machine service.
>
> **Where it is predicted** (§6.5, §6.6):
> - 4/60 joint2 and 12/40 joint4 at C1, and 8/40 joint4 at C2;
> - 89 and 77 basin runs;
> - C1's tilt-1 Tier 3 (3);
> - C1's land.mach history, window 5;
> - the negative control's Tier 3 (9: JA(0.5), JB(2), RC(0.5), p[land] × 2, p[mach] × 0.5,
>   land.mach = 0.8 and b.food = 1.2 at genesis and dated), each a runaway at ticks 279–314.
>
> **Never in** the verdict battery at χ_max 1, the stocks family, the commons history, 12 or 365 a
> year, `Hold`, or enclosure. I1 never enters it: 0 of 530 on joint2, joint4 and basin. It is the
> priced exit's, and its gain grows with the supply's elasticity. On C1's Tier 3 with the exit at
> its band's middle it takes 0/43 at χ_max 1, 4/43 at 0.5 and 11/43 at 0.25 (`diag_chi.out`).
>
> A clamp is no answer (R3). It is O95, for its own scan before the 1750-like instance.
>
> ### 6.9 What would refute the frame
>
> Each of these would send the instance back to the frame, with the alternative of decision 363 or a
> new instance:
> - a class change in E3 (the verdict battery);
> - any engine run at a registered target that rests off the oracle's point;
> - a trace diff (E0) parting that is not explained by a named rounding;
> - the trap in any E3 run, or none of the trap's predicted runs in E5–E6 and E9 falling into it
>   (the mechanism would then be the mirror's alone).

## 5. Decisions and open items

SPEC §7 numbers its proposals 360–371 and its open items O95–O102, as its brief asked. At this
commit those numbers are taken: 360–393 and O95–O96 by P2.3.0, 394–397 and O97–O99 by the wall. So
STATE.md numbers them in this line's range as below: the twelve proposals as decisions 398 and 399
(grouped, since two numbers of the range remain), the frame's O96 as an amendment of O96 (the land
market at zero rent, which the scan answers for the commons and leaves open for enclosed land), and
the rest as O100–O106. Within the quotes above, and in SPEC, the numbers are the frame's. Each is
Claude's, on the user's delegation, and open to veto; the alternative named is the registered one
(R6).

| STATE | the frame's | what |
|---|---|---|
| 398 | 360–366, 368 | C1 and C2 on I1's economy with one priced type in food, χ_max 1; Phase 2's idle land at zero rent is the commons' idle part; the commons is no market, the commoners' rule gives out its plots; the workers' rule min(max(n(0), N − T_o/h), n(r̂)); plots on enclosed land rented in money in one chain with the baskets; the exit good in the basket and land not; no new state, ν 1. It amends 374's reading that a crowded commons' shadow rent must be posted |
| 399 | 367, 369–371 | the cost coefficients land.mach, b.food and the commons, enclosure a family; the targets from unit 1e's `ParcelEconomy`, land's volume T and labour's S; the subsistence trap the families' prediction, not the verdict's; the engine run E0–E9 with §5.6's tolerances, and the verdict with 368's Tier 3S (§3) |
| O100 | O95 | the subsistence trap |
| O96, amended | O96 | idle enclosed land at r = 0: unfunded under the probe's transfer; needs a funded support and a free-good rule |
| O101 | O97 | a commons shared by several types |
| O102 | O98 | the Enclosed regime's accounts: rent in money, not in kind |
| O103 | O99 | C2's commons shocks × 1.1, × 0.9 and × 2 are VACUOUS by construction |
| O104 | O100 | 12 a year is slow here too |
| O105 | O101 | C1's x\* lies just below care's edge |
| O106 | O102 | the joint family's draws |

