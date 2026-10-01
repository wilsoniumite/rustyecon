# The type switch at the wall: registration of the migration rule (P2.4.7, O97)

Dated 2026-09-30. Step P2.4.7 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`), on top of
`b7b9242` (P2.4.6), clean but for this registration. It registers the mirror's predictions for the
engine run of O97's rule, the migration rule on a worker type that sells both reserved and pool
hours (the instances IS1 and IS2), before any engine code for it exists (R5): at this commit
`RawBasketWorkers` has no `pool` field, `ActorState` has no `SwitchWorkers` variant,
`crates/probe` has no switch instance, and `tapes/` has no switch tape. PHASE2-S1 §6 item C. Any
later change is a new dated amendment with its own sha256 (decision 282); earlier results stay
reported.

## 1. What is registered

The scan's frame, in [docs/probe/switch/](../../switch/), as its author left it in
`D:/rustyecon-p24/scan-switch/` (label `scan-switch`). The repository stores text with LF
(`.gitattributes`). `SPEC.md` and every registered TSV and JSONL were written with LF, so their
committed sha256 is the registered one; four files were written with CRLF (the two 50-digit
switch tables and the two 50-digit solves' outputs), and both sha256s are listed for them.
`SHA256SUMS` (125 entries) was checked entry by entry against the scan's files on 2026-09-30, all
OK; its own sha256 is `0f434cbe39dabf0a53565bb7e824771fc593e1f7777b049ba7d75eee56fa1449`.

| file here | the scan's file | sha256 as registered | sha256 as committed (LF) | what it is |
|---|---|---|---|---|
| `SPEC.md` | `SPEC.md` | `ebf52cea1b38d4166be5bc2d8fc32d57b9ddd45ee27006560b7d70f5a0315f6f` | the same | the spec: the problem, the instances, the rule, the proof, the checks, the scan, the predictions, the proposals |
| `SHA256SUMS` | `SHA256SUMS` | (itself) | `0f434cbe39dabf0a53565bb7e824771fc593e1f7777b049ba7d75eee56fa1449` | sha256 of every file of the scan |
| `registered/runs_is1_battery.tsv` | the same | `fb2a02a553e2b4bdf7439066ff899025a9981cb5d0526541561292fbaf186cd2` | the same | IS1's battery, 109 runs (IW1's 103 and `sw[T]=V`) |
| `registered/runs_is1_tier3s.tsv` | the same | `cf8675239c85aa53b14cc9a09cde9e6e8a854dea79de59b33c6ceb0e46c685e8` | the same | Tier 3S, 20 runs, first-year start |
| `registered/runs_is1_tier3L10.tsv` | the same | `282c7fd38d7d5a2777a6020611210c547d8aeb23d4794070c0478d4cdb5cf0b3` | the same | Tiers 3 and 3S at 10·L, 61 runs |
| `registered/runs_is1_nbhd.tsv` | the same | `75cfdbfa961803df6697d105d6317e2d3fa14f4a50308884272aca89555cbda3` | the same | the dial neighbourhood, 17 settings × Tier 3, 697 runs |
| `registered/runs_is1_switch.tsv` | the same | `e5141701320f48eb0a3cc6e6519dc6871aff72784c31800176e7b2a090ddc724` | the same | `rate.switch.*` alone × 0.75–1.25 on Tier 3, 164 runs |
| `registered/runs_is1_tpy12.tsv`, `_tpy365.tsv`, `_hold.tsv` | the same | `56cff449…1977`, `2a489c62…aa25`, `37649ac7…5066` | the same | Tiers 1–2 at 12 and 365 a year and under `Hold`, 68 runs each |
| `registered/runs_is1_stocks.tsv`, `_joint2.tsv`, `_joint4.tsv`, `_basin.tsv` | the same | `f95cfbd8…8384`, `b7608e29…c91b`, `20f48ac5…88d2`, `7ab7d4b0…516f` | the same | the wall's families: 31, 60, 40 and 516 runs |
| `registered/runs_is2_<set>.tsv` (7) | the same | battery `b280c2f8…2d11`, tier3s `a48cdc14…f539`, tier3L10 `5eeb158f…b3b3`, nbhd `90287d3f…b0`, tpy12 `b45adf0c…61c9`, tpy365 `08ca997c…f03e`, hold `b6dc9f8f…f926` | the same | the control IS2's sets |
| `registered/never_pooled_is1.tsv` | the same | `0724cb7c0290caf89bc9ff2d2691ac58df4d9e96245d7a332b1977768a102dce` | the same | the 68 runs where no type pools, beside IW1's registered class, ticks and peak D̂ (E1) |
| `registered/local.json` | the same | `053377ff938dcd99ec01fe10dd03e5884ebdadcf547fbf69e773308861000e2e` | the same | the kick sets and largest roots at the 13 targets of IS1 and IS2 (E4) |
| `registered/points_is1.jsonl`, `points_is2.jsonl` | the same | `db6170b3…00e1`, `ceea6ee3…7887` | the same | the oracle's points, 39 each (unit 1d, the worktree's, unchanged) |
| `registered/switch_mp.json`, `switch_mp_is2.json` | the same | `59b58877ea38da70ea3165e1357e5283a6391beda14d1ba988fb4a38d7859724`, `dd5b10eb8e5d3c96e41fea082783230130a735568f6cc735ff44b50facee2f98` | `b57304008c6c4048ec03052cba645972b1f3e01d1886c2393a91708b570f27c6`, `037580cda6d48273d460f597ac243ef0c6997470c2960029145e3824d18c6b02` | the 50-digit solve's pooled flags, a\* and switch distances (E9) |
| `registered/rule_vectors.json` | the same | `0557240479f1ea36b4f87d3d1b1f77963be0147d0bf03786106573584b379c67` | the same | 24 of the rule's type-states, inputs and outputs as Python `repr` doubles (§3.10) |
| `evidence/scan_summary.out`, `tables_is1.out`, `tables_is2.out` | `out/…` | `32a3dbc8…504b`, `a737d28a…efa1`, `c77689da…291e` | the same | the scan's candidates and the chosen rule's tables (SPEC §6) |
| `evidence/local_26.out`, `local_max26.out`, `local_pass1.out` | `out/…` | `315aade0…823f`, `59e6ff12…bbb1`, `888feb19…4c` | the same | the largest roots and kick bars (SPEC §6.3, §6.4) |
| `evidence/is1_table.out`, `is2_table.out`, `is1_vs_iw1.out`, `reg_is1_vs_iw1.out` | `out/…` | `ed7a2fc0…7f92`, `3a0fd110…1f1d`, `d3c3b5d3…60a6`, `58a322a7…ba96` | the same | the points' table (SPEC §2.2) and IS1 against IW1 run for run |
| `evidence/solve_sw_mp.out`, `solve_sw_mp_is2.out` | `out/…` | `5e8bc614c4c233364ff7c5bbd44c12fe22c530fcce4140bb915ebfa5cbe7dc5e`, `aee3615b0196b8bac4a92bec5328a3b45f83e383dd004d389970579610ddd1e7` | `d1fc0048ed9b2a46dec66a0d1a3f841377b8d6b99cc79435252f9ac5c6708e44`, `368cb5897920d777b5ce391b7340dcb0823742d5003839b3e50b0f6f7a9a8635` | the independent 50-digit solve (SPEC §2.3) |
| `evidence/sw_nest.out`, `sw_rest.out`, `sw_rest_is2.out`, `sw_vectors.out`, `sw_money.out`, `sw_fullL.out`, `swel.out`, `never_margin.out`, `walled_same.out` | `model/…` | `cf3db3ad…8270`, `df1d7fb2…4d47`, `1c7a760d…d8df5`, `659e8242…8e7a`, `b3e044f6…107a`, `f6a217fd…26ed`, `fe65c10e…87d4`, `04f8ef88…bcd7`, `5deee695…d4d1` | the same | the mirror's checks (SPEC §2.2, §4, §5; E1, E2) |

The scorer (SPEC §7; decision 311) reads `registered/` from this copy.

The mirror the predictions come from stays in `D:/rustyecon-p24/scan-switch/model/`, as the
frames' did (sha256, from `SHA256SUMS`; read in place):

```
f1c6ed90977b85356e2d8dbaf123805359682f9c0c5711d1ce3ebfb291cfa11b  model/wms.py         (the switch mirror, E0's map: `tick`, `switch_share`)
d60dfe98a49da676f3f77b504eee6373181ef1b1d0fb71d2ff115504d41cb260  model/make_wms.py    (writes wms.py from wm.py by 8 checked replacements)
3f93de64e403fbe022a1b33af0f8958ac82b92d9178dca8aba6798c338764c4b  model/wm.py          (the wall frame's registered mirror, copied unedited)
a8faf21a8708edbbb01ab52f162f5bad78b9af8017d08c64a0ed37f2660e82ee  model/sw.py          (IS1 and IS2, genesis, observables, targets)
841e9155afca81621c09393c905a01596dbc3326197aecaab21cd8c3d09cf7c1  model/swb.py         (the battery, the grammar with sw[T]=V, the readouts, the classes)
9fdd58e4dd07881f2cf9c48d4994bdf0318531d3f272de77c64cca642887d1b5  model/swrun.py       (the sets and the dial settings)
f211fa52942314fa8d9f7ddf6b1e957c1259581033f82956af096fe3ef7d566f  model/sw_vectors.py  (the rule written again from §3.3; the vectors)
738f6432eb2f0f744582ee832e37720f11e442600021e057949d2ec5bf0eadc0  model/sw_rest.py     (the rest point at 78 points)
a5aaaa2f4b7351124af74a240813d0529fd3161a6b8e0d71613315d896eb0e11  model/sw_nest.py     (the switch off is wm.py)
c30d47a8ee922095d7f82a5e3eded607288e1c4d1dfb6385f31caee63f9c2f72  model/swel.py        (the elasticity probe and L)
a81d77d2151101e5517309e4260682d8bec2d592d5730d8e3527a2a08ffe91ea  model/swlocal.py     (the kick sets and roots)
2fdd1a5a71d609856a6dcd38606148fbd8b752ab6e851615a80139683bbb374d  oracle/src/main.rs   (the oracle's points: the worktree's unit 1d, a `sw` mode)
49904ec3756834064211fd00ea19ee016535ac5e100251eef64aa6b2c7acd008  hp/solve_sw_mp.py    (the independent 50-digit solve)
```

## 2. The order of work

1. The scan ran on 2026-09-30 in scratch (label `scan-switch`), reading the worktree at
   `a483ed0` and changing nothing. P2.4.1–P2.4.6 (wave A's registration and machinery, the trap's
   remedy) landed during and after it; the trap's build (P2.4.5) touched the workers' role
   (`RawPricedExit.pace`) and the markets harness, and none of it runs at IS1 or IS2, which have
   no exit. No engine code for the switch and no engine run of a switch instance exist.
2. This commit fixes the registration in the repository, and STATE.md numbers the scan's
   proposals (§5).
3. Next: the build (SPEC §3: the optional `pool` on `BasketWorkers`, the appended state
   `SwitchWorkers`, the rule in §3.3's order, its load checks and genesis state; IS1 and IS2, the
   generator and `tapes/markets-is1.ron`; the harness's observables, grammar, families and
   readouts; the tests of §3.10; `docs/probe/SWITCH-RULES.md`), with every committed tape's text,
   hashes and streams unchanged. Then E0, the trace diff against `wms.py` on SPEC §7's seventeen
   runs, before any scored run. A real departure found by E0 is a dated amendment committed
   before scoring. The scorer is committed before the wave (decision 311).

The sources at the base (sha256 at `b7b9242`), which the build will change:

```
514f834891884ef87ec8d340185078a68615f9d904de6dd86fe620a2b51bd24d  crates/agents/src/roles/many/spec.rs
be8dbec477a7ebdf14e620991e95432c66bf4b61e492c8807bbbfed552de2e93  crates/agents/src/roles/many/rules.rs
639c3ed9528dbfdee8587c767290bc93f1d06f9aa88ec76a7193ae96cf457a47  crates/agents/src/roles/many/mod.rs
ac8ea7e7f5ff17b802c6f804513628c681c4963c67c73360857e5084ec4b0e15  crates/agents/src/ext.rs
026dcd5f7843a9f704327ebdaaedc7e42e64d33c6ae41fd83a7861b98c032449  crates/agents/src/cast.rs
7603a4415b5fa06fec8173dfc1fa4eac5795cb7bc307cd22e29a8ce996bd5c38  crates/agents/src/lib.rs
c9d5e666d1c1a8d5cb990af95739c0ea5c5f74078dd0b287559882fee17d11e8  crates/probe/src/markets/instance.rs
eff9443d7ba7a223db1b5ee06078b593cc7de6a19dd0242f0af778e9b325eb98  crates/probe/src/markets/setup.rs
2a4a4ee968c8394dc7f1ee46cdc010cf36ba386c71dbf5718da3ac67224113a9  crates/probe/src/markets/harness.rs
3779ad1e72163bdbc6bbccdcfacbb77b706375aa37941b288539d660ad9edece  crates/probe/src/markets/perturb.rs
ce0382595b349f6d1d417b7476cdf18c8b46febe7913d5c68d6b7c1cf784e15e  crates/probe/src/bin/markets.rs
37a6886f1d70d6465829f90f4a06fbb3a46c8bd68cb63581ff9c73b6579c0d29  crates/probe/src/bin/markets-tape.rs
46b3cc86b9851a1e517e9824d147ed7fb0f99a6a915ede9075f2006d85e1599d  crates/gui/src/run/extract.rs
```

The pre-build binaries (`b7b9242`'s code) and every committed tape's 2,000-tick hash stream from
them are in `D:/rustyecon-p24/build-switch/base-*` on WSL and Windows; the two machines' streams
equal each other for all 27 tapes.

## 3. Readings the build takes, declared before it

SPEC is registered as written. Where its text leaves a choice, is inexact, or leaves out a ruling
that binds it, the build takes these readings. Each follows the registered mirror, so none moves a
prediction of SPEC §7, but the first, which is E0's third item read as the engine's arithmetic
allows.

- **The rule vectors, bit for bit where the platforms' `exp` agree.** SPEC §3.10 asks the rule to
  match `registered/rule_vectors.json` bit for bit. The vectors are Python's doubles, whose
  `exp`, `expm1`, `log` and `log1p` are glibc 2.35's; the engine's transcendentals are libm
  0.2.16's (`rustyecon_core::num`), one call each, the same on every platform. A check made before
  this registration in scratch (`D:/rustyecon-p24/build-switch/libmcheck/`), with no engine code,
  wrote §3.3's steps 3–6 in libm and compared: 21 of the 24 vectors agree in all six outputs bit
  for bit; at vectors 5, 13 and 23 (k·g = −0.28943, −0.18946, −0.19466, where the share decays)
  libm's `exp` is one ulp off glibc's, and a′, v, F, the reserved hours and the pool's efficiency
  hours part by one or two ulps (at most 2.4e-16 relative). g agrees in all 24. The test reads
  the vectors so: the rule's outputs bit for bit at the 21, within 4 ulps at the three; and the
  engine's decision bit for bit against §3.3 written again in the engine's arithmetic, on every
  vector and on random states. E0's 1e-12 absorbs the ulp (O110's cousin).
- **The field.** `pool: Some((good: "labour", efficiency: "inst.<type>.efficiency", rate:
  "rate.switch.<type>", share: a*))`, written last in each reserved pop's block, and absent (not
  written) on every other tape. The resolved `Pool { good, efficiency, rate, share }` keeps the
  genesis share, as the trap's `Pace` does, since `genesis_state` reads the resolved spec.
- **The load checks** are §3.6's, each a `LoadError` at `actors[<pop>].spec.pool…`, and one more,
  as the roles' other goods are checked: the pool's good is not an item of the pop's basket (a
  good named twice in one role). `pool` with `exit` is refused at resolve; `Instant` is checked
  with the world (`Cast::new`), where the other endowments are.
- **The orders.** The pop's own labour is minted when r > 0 and always offered, r ≥ 0, as P2.3's
  workers offer theirs; the pool's good is minted and offered only when p > 0 (§3.3 step 7). So a
  walled type's orders and deltas are P2.3's but for its state.
- **The instance.** `Instance::named("is1")` and `("is2")`: IW1's with each reserved type's
  efficiency (trained 1.5, master 1.8) and the switch on for both reserved pops. The params
  `inst.<type>.efficiency` (`Dimensionless`, basis `Assumed("scan-switch SPEC §2.1: unit-1d.md
  §3.3 E's efficiencies")`) are written for every reserved type at a switch instance and at no
  other instance, so IW1's tape is unchanged. IS2 is IS1 with `inst.services.reserved.trained`
  0.02 and its own values for that coefficient, 0.022, 0.018, 0.04 and 0.01, as the mirror's
  TSVs name them.
- **The dials.** `rate.switch.<type>`, 26 `RatePerYear`, basis `Assumed("scan-switch SPEC
  §6.3")`, read as a `LogStep` (k = rate/tpy), written in C2m after the reserved labour markets'
  rates at a switch instance only. `--set rate.*=f` scales them with every price rate (the
  mirror's `rate` dial); `--set rate.switch.*=f` scales them alone (the mirror's `switch` dial).
- **Genesis.** Each switch pop's pool share is a\* = `WorkerEq::pool_hours / WorkerEq::hours`
  where the type is pooled with pool hours above 0, else 0.0 (the mirror's `pool_share_of`);
  `sw[T]=V` sets it to V exactly, refused outside [0, 1]. Every other genesis number is the
  wall's, on 1d's point with the efficiencies (coins from `WorkerEq::hours` and the posted wage).
- **The observables.** IW1's 18 in IW1's order, then `rs.<type>` for each switch pop in pop
  order: 1 − a after the tick, in log, target 1 − a\*. Each reserved market's volume target is
  `WorkerEq::reserved_hours`, D_i, which equals `hours` at a wall bit for bit (unit 1d's code
  sets one from the other), so IW1's targets do not move.
- **The start distances.** `sw[T]=V` is a price start: |ln((1 − V)/(1 − a\*))| joins the
  genesis prices' and thresholds' gaps (the mirror's `d0`). A cost shock's start is the wall's
  largest |ln| over every observable of the two targets, the reserved shares included, as the
  mirror's `targets_of` keys are.
- **The grammar and lists.** `sw[T]=V` with T the type's key, refused at an instance with no
  switch pop. The battery is the wall's 103 runs in the wall's order, then `sw[T]=V` for each
  switch pop in pop order at 0.05, 0.2 and 0.5 (tiers 1, 2, 3). Tier 3S, stocks, joint2, joint4
  and basin are the wall's lists at IS1.
- **The readouts,** at a switch instance only. The CSV adds `pool_<pop>` (the pop's pool share
  after the tick) and `swgap_<pop>` (g at the tick's posted prices, through the rule's own
  function) for each switch pop, last. `stats.tsv` adds `switch.live_ticks` (scored ticks with a
  above 1e-9), `switch.max`, `switch.band` (sign changes of g between scored ticks with |g| above
  1e-6), `switch.end` and `switch.gap_end`, each with the pop as `where`. The wall's
  participation readout reads each switch pop's F, its state's `share`.
- **The tapes.** `tapes/markets-is1.ron` is `markets-tape --inst is1`'s output; IS2 is `--inst
  is2`, with no committed tape.
- **E0's comparison** is the wall's (every price, ratio, threshold, stock, coin, S and D in log;
  the shares absolute), with each switch pop's pool share (the mirror's `new["ps"]`) absolute and
  in log where both are above 1e-6, and its gap g (the mirror's `gap_r`). The mirror is seeded
  from the tape's genesis, the pool shares from its `pool.share`. Its runs are SPEC §7 E0's
  seventeen. The wall's amendment A1 (a tick-1 residue of a genesis lot) applies as at IW1.

## 4. The predictions

Quoted from SPEC §0 and §7:

> **GO for the migration rule, on IS1.** A type with pool efficiency ε > 0 and reserved tasks
> keeps one state, its pool share a: the share of its hours it sells to the pool. Each tick,
> before it offers, it reads two posted wages, the pool's wage for an hour of its own, e = ε·w,
> and its reserved wage w_i, and moves a toward the market that pays more.

> **E3. The verdict.** IS1 is GO: Tier 1 28/28, Tier 2 40/40, Tier 3 41/41, Tier 3S 20/20
> CONVERGED, and Tiers 3 and 3S again at 10·L with the same classes and ticks. Medians (slowest)
> 338 (452), 446 (575), 561 (734); Tier 3S 404 (558). Every other statistic per run as the TSVs.

E0: the engine within 1e-12 of `wms.py` over 2,000 ticks on the seventeen runs. E1: every
committed pin, tape and stream unchanged; at ε 0 IS1's tape gives IW1's numbers bit for bit; the
68 never-pooled runs are IW1's run for run. E2: the rest point within 1e-12 at 78 points, mode A
at L, L 22,000 (IS1) and 24,000 (IS2) at 52 a year. E4: every kick set decays at the 13 targets
of IS1 and IS2, the largest root 0.992722 (at tail.services 0.11). E5: a type pools in 61 of the
129 battery and Tier-3S runs; each pooled pop ends at its a\* within 1e-10 relative. E6: RW(2)
bottoms at 0.026 of Y\* with 53 dead ticks. E7: the neighbourhood 697/697, the switch's rate alone
164/164, 12 and 365 a year 68/68 each, Hold as Saturate, stocks 31/31, joint2 60/60, joint4
40/40, basin 516/516. E8: IS2's sets as its TSVs. E9: the generator's solve reproduces the pooled
flags, a\* and switch distances within 1e-12.

**What would refute it** (SPEC §7), each sending the rule back to the mirror: a class change in
IS1's battery or Tier 3S at L or 10·L; a CONVERGED run ending more than 1e-12 in log off its
oracle point; a switch pop ending on the wrong side of its switch, or a pooled pop's share more
than 1e-9 relative from a\*; the rest point off unit 1d's by more than 1e-12; a never-pooled run
differing from IW1's; a kick set failing, or the slowest mode at or above 1; any committed pin
moving.

## 5. Decisions and open items

SPEC §8's proposals, numbered: decisions 409 (SW1: O97's rule is the migration rule, participation
at the split's wage), 410 (SW2: `rate.switch` 26 a year), 411 (SW3: one optional field and one
appended state), 412 (SW4: IS1 is O97's instance), 413 (SW5: IS2 a reported control), 414 (SW6:
the harness at the switch) and 415 (SW7: the switch distance registered, not bounded); open items
O118 (OS1: O97's other two cases), O119 (OS2: the switch's path cost), O120 (OS3: a type near its
switch is slow), O121 (OS4: the margin at monthly ticks), O122 (OS5: the walled share's subnormal
stall) and O123 (OS6: the switch under the priced exit). STATE.md has them in full, each open to
veto.
