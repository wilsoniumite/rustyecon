# The wall instance's registration (P2.3, IW1)

Dated 2026-09-30. Step P2.3.1 on branch `phase2-proper` (worktree `D:/rustyecon-wt/p23`), on top of
`f0c6667` (P2.3.0, the rulings), clean. It registers the mirror's predictions for the engine run of
Phase 2 proper's wall instance, IW1, before any engine code for it exists (R5): at this commit
`crates/agents` has no `tail`, `reserved` or `more` field, `crates/probe` has no wall instance, and
`tapes/` has no wall tape. It is written by `D:/rustyecon-p23/build-wall/make_registration.py`,
which checks the frame's `SHA256SUMS` entry by entry and copies every quote below from its file.
Any later change is a new dated amendment with its own sha256 (decision 282); earlier results stay
reported.

## 1. What is registered

The frame, in [docs/probe/wall/](../../wall/), byte for byte as its author left it in
`D:/rustyecon-p23/frame-wall/`:

| file | sha256 | what it is |
|---|---|---|
| `SPEC.md` | `cc2b6d1c55a5d04a4d44c0447dce0158431f5e74b0b8d9c6e8a3cb238cc89be0` | the frame: IW1, the roles' additions, the harness at the wall, the mirror's battery, §7's predictions |
| `SHA256SUMS` | `43fccf3a2dff88f8c15b36d2c910b02837b272c129fc27830ae4371a47bbf669` | sha256 of SPEC.md and every registered output and script (56 entries, each verified on 2026-09-30) |
| `registered/edges.json` | `ab13d3ac506fdfd535364438b9ff9436f59e0b79358e6b576e259b073aa41873` | a registered output; CRLF in scratch, committed with LF (below) |
| `registered/history_land.json` | `26de3622788fbadf550326bfdb3a407eefdc7bc2c614429efea1c68dcded3cba` | a registered output |
| `registered/instances.json` | `c1f4bae629fda3fd41ccba28e671695ec7ff6291ff7f16151cfdc6c130530ce7` | a registered output; CRLF in scratch, committed with LF (below) |
| `registered/local.json` | `e5b84861d45fec11efe906409437ac260e75b9dc19492fc0de0b44e85549062a` | a registered output |
| `registered/points.jsonl` | `4e6d510d1c7262d17fc586935dd79a58c4d7fb476509965a7ecc6bee32d7710f` | a registered output |
| `registered/runs_basin.tsv` | `5ba6d6d2eb82c236f0a0de597f8bda7a63cf8df23528e71a070a8aeb7e758893` | the run-by-run outputs of one set |
| `registered/runs_battery.tsv` | `0b6e50bf31e4473ba1e0b81c82b72fc4c50dca6f1b36f48182f39e9ec4c39710` | the run-by-run outputs of one set |
| `registered/runs_hold.tsv` | `975b3a33e2b76319532e9756ba874c373cb2792325586331b1512060ecebd3fb` | the run-by-run outputs of one set |
| `registered/runs_joint2.tsv` | `0c3933c0d1bd28629c1059b13c6b82c520e5e0ba03736d92742194178135dacf` | the run-by-run outputs of one set |
| `registered/runs_joint4.tsv` | `695cee77ec5f81f4a7e4f9c09514b91d00eef68fc6c4c8267e1bde3fd3abacd7` | the run-by-run outputs of one set |
| `registered/runs_line.tsv` | `ea84c5bff8e557e71437be3d6ad7806cd603ab5fdeade9e436764cce17780982` | the run-by-run outputs of one set |
| `registered/runs_stocks.tsv` | `1fdc4dd56cd4caba4e6065c94ad4b495ad21e775735e2b97672404f42a31a9db` | the run-by-run outputs of one set |
| `registered/runs_tier3s.tsv` | `b45296636402ac5d5353d75f1dbb27499f5e21359473c18a72d3288ea3a7319f` | the run-by-run outputs of one set |
| `registered/runs_tier3s_line.tsv` | `9ace2986c7006f638ad32e2b809170a6987a66b8177e3488a1ef0feb1d6da6e2` | the run-by-run outputs of one set |
| `registered/runs_tilt1.tsv` | `d46f5d8ed8e4c2faabdd7ba40b073212cbbce9f337dfd1a743c9fe09a575dc49` | the run-by-run outputs of one set |
| `registered/runs_tpy12.tsv` | `60a7dd902e77828205f5e57b4add80187105650dc638be006bd1a5e89dd9b80e` | the run-by-run outputs of one set |
| `registered/runs_tpy365.tsv` | `b87c95dd62c39ab48485660d4e8c7ae3d771a63045125c415945c7b46ee70529` | the run-by-run outputs of one set |
| `registered/solve_mp.out` | `abf997ba7a09cb4ca495dd876801d590c6f0f6f1377e78baaa858871a6ced847` | a registered output; CRLF in scratch, committed with LF (below) |

The repository stores text with LF endings (`.gitattributes`). Three files were written on
Windows with CRLF endings; git commits them with LF, so their committed blobs have the
sha256s below, and the registered ones above are those of the same text with CRLF. Nothing
else differs, and the scorer parses either.

```
f796b77e61fd2c5dabc589be1bf61f902e0e56d6ca75a6009acee682d23f50cf  registered/edges.json  (as committed, LF)
c93ca71a44b6f0ff414f0eb22bc5393ab724dd53dda21c8d8fc2e91870f58757  registered/instances.json  (as committed, LF)
ff06de418e565749da29ca2793d577ad73ce9d2cc8dc577c250b52e32b28c259  registered/solve_mp.out  (as committed, LF)
```

The scorer (SPEC §7, decision 311) reads `registered/runs_*.tsv`, `local.json`,
`history_land.json` and `edges.json` from this copy.

The mirror the predictions come from stays in `D:/rustyecon-p23/frame-wall/model/`, as P2.2b's
did (sha256, from `SHA256SUMS`):

```
88649e463d157e522bb75675b1b84511b577780ede46923f23b6fb886bd88497  model/make_wm.py       (writes wm.py from it by 17 checked replacements)
dbd50ad9c1e948aa9bb23545df2dfcb89923cd0d45a0106e8ccfcbbe2798538c  model/mm_carry.py      (the markets probe's trace mirror, copied unedited)
a44bd25dc12dfa059a4440e28b047566789d2e33455ff569d4cfbe5c3a21df06  model/p21_battery.py   (the markets predictor's battery, copied unedited)
32ab35f20b4b3c1b824e3e22866fbb7e7573656c9d3bba74ba8f53c0282a5e8c  model/run_battery.py   (the sets and families, run by run)
28e61b4c85ae866348b8234328f2d4b911aa8700f6d94a6c2fc439b29035890a  model/wall.py          (IW1, its targets, genesis from the oracle's doubles, observables, depth)
00ec06ff27040bfd864eefb0fe0f9c6abc4cd1701dbb9964bf7ee316f61ecb9d  model/wb.py            (the battery runner, run grammar and readouts)
3f93de64e403fbe022a1b33af0f8958ac82b92d9178dca8aba6798c338764c4b  model/wm.py            (the wall mirror: E0's map)
7b08b2b746a9f7890c80d2bb4b0f3f5e104f4ce69876d82d0ccdfabfec132a0f  model/wm2.py           (the second map, written apart)
```

## 2. The order of work

1. The frame was written on 2026-09-30 in scratch (label `frame-wall`), reading the worktree and
   changing nothing. It began at `f7d1eae`; P2.3.0 (`f0c6667`, STATE.md only) landed during it,
   and the frame follows those rulings (SPEC §8). No engine code for IW1 and no engine run exist.
2. This commit fixes the registration in the repository, and STATE.md numbers the frame's
   proposals (§5 below).
3. Next: the build (the roles' three optional fields, the instance, its generator and tape, the
   harness, the tests, `docs/probe/WALL-RULES.md`), with every committed tape's text, hashes and
   streams unchanged. Then E0, the trace diff against `wm.py`, before any scored run. A real
   departure found by E0 is a dated amendment committed before scoring. The scorer is committed
   before the wave (decision 311).

The sources at the base (sha256), which the build will change:

```
e07e4b0738d47dfc87815bab40e7da075982485ad171031a807224f2bb588ad7  crates/agents/src/roles/many/spec.rs
d58637e9302ce0525c5574cac399ad4f757cd173d311816560151bfb88c0d9b7  crates/agents/src/roles/many/rules.rs
b0a9c9c7a2fb3762b3298bf4084de0eb0e43914672420e47b737acd60bccf8b5  crates/agents/src/cast.rs
8f80ba29c867ed5e5867e67be57a942c393bdc47b007abf95fe637bb2f9063b2  crates/probe/src/markets/instance.rs
12e456ccdaf6e62b3ce471900f0612475efd6785cf3f11c1e3f6edffd5545269  crates/probe/src/markets/setup.rs
ac3a9dbe1d84f544f6c1eafaf4d56cd0d41eb21a0ef232304f6bd0c4f162041b  crates/probe/src/markets/harness.rs
f6d072d1b41a60e50561fd94b0499759e22d7b5582e1aa6e91bb7cf3f73d6267  crates/probe/src/markets/perturb.rs
5234583ceba5585d309de0d2031099feb3ef9373942f073e32ef6836a2c8f76c  crates/probe/src/markets/probes.rs
5d20a4a922dc69bfe58e3469d859e5a10f18f1c02d5fb071ac2ddf2ec597966e  crates/probe/src/bin/markets.rs
1bd58cedf057dfa629d84c0961d7781807527d9b546fd3e8cfd727074b3b0a90  crates/probe/src/bin/markets-tape.rs
```

## 3. Readings the build takes, declared before it

SPEC is registered as written. Where its text leaves a choice or is inexact, the build takes these
readings, each from the registered mirror, so none moves a prediction:

- **E0's coefficient runs.** SPEC §7 E0 names "the three coefficients at ×2 or ×0.5 at genesis".
  They are read as `wm2_check.py`'s three: `land.mach=0.8@genesis`, `tail.services=0.2@genesis`
  and `res.services.trained=0.02@genesis`. With the nine displaced runs and I0's hold, E0 is 13
  runs of 2,000 ticks.
- **The observables.** §5.1 counts 16 but lists 18 (v, two reserved wages, three prices, two
  thresholds, seven volumes, three outputs), and `wall.py` reads 18. The harness reads the 18.
- **Run names.** The battery and families are named as `wb.run_list` and `run_battery.py` name
  them (`p[labour]*1.05`, not `w*1.05`), so the engine's lists equal the registered TSVs' `run`
  columns name for name.
- **`joint(F,SEED)` at the wall** draws its factors in the mirror's market order (labour, land,
  the categories, the type, the reserved labour markets), then the shares, as `wb.displace` does.
  P2.1's instances keep their order.
- **`s[D]=V`** sets desk D's genesis human share to V exactly, as `wb.displace` does (not as
  1 − (1 − V)).
- **The tape** is `tapes/markets-iw1.ron`, SPEC §3.1's name, written by `markets-tape --inst iw1`;
  IC1 is `--inst ic1` and has no committed tape.
- **The history family's shock values** are `base·f`, as P2.1's `cycle` writes them; the mirror
  rounds `base·f` to 12 digits. The two differ by at most an ulp, and the family is reported, not
  scored.

## 4. The predictions

Quoted from SPEC §0, the answer:

> **The verdict in the mirror: GO.**
>
> | id | what | mode A (largest gap) | Tier 1 | Tier 2 | Tier 3 | Tier 3S | kick sets | PL a tick: base, largest | families | verdict |
> |---|---|---|---|---|---|---|---|---|---|---|
> | **IW1** | the wall, three labour markets | PASS (3.3e-16) | 26/26 | 38/38 | 39/39 | 20/20 | 13/13 targets pass | 0.987132, 0.991911 | stocks 31/31, joint2 60/60, joint4 40/40, basin 516/516, history 79/79 windows; 12 and 365 a year, Hold and tilt 1 each 64/64 | **GO** |
> | IC1 | IW1 with the entrant's χ_max 0.25: the task margin active (x\* 0.947), a reported control | PASS (2.2e-16) | 26/26 | 38/38 | 39/39 | 20/20 | not run | 0.989164, 0.991127 | — | control |
>
> Ticks to tolerance at IW1, median (slowest):
> - Tiers 1–3: 340 (452), 416 (575) and 561 (705), which is 6.5, 8.0 and 10.8 years. I0's are 320,
>   440 and 523 (MARKETS §2).
> - Tier 3S (decision 368): 404 (550).

Quoted from SPEC §7, in full. This is what the engine run is scored against:

> ## 7. Registered predictions for the engine run
>
> Registered before any engine code for IW1 exists. The conditions are those of the header and
> §2–§5. The engine is expected to match the mirror as the markets probe's did (MARKETS §2) and as
> P2.2 set the bands:
> - **Classes** exactly.
> - **Ticks and years** within 10%, with three runs a set allowed within 25%.
> - **Troughs and lowest depths** within 5%.
> - **Dead-tick and breach-tick counts** within 10% or 5 ticks, whichever is larger.
>
> **E0. Before any mode-B run: a trace diff.**
> - `wm.tick` against the engine for 2,000 ticks. `wm.py` carries the genesis carry, including a
>   buyer's unused genesis-lot purchase.
> - At IW1 the runs are hold, p[labour]\*2, JB(0.5), x\*/2 and s[goods]=0.5; RW(2),
>   p[labour.trained]\*0.5, coin.workers.master\*0.1 and stock.mach\*0.01; and the three
>   coefficients at ×2 or ×0.5 at genesis. At I0 the run is hold.
> - Every price, share, coin, stock, S and D must agree within 1e-12 in log. The one allowed
>   parting is the budget chain's ulp (MARKETS-RULES §6.4).
>
> **E1. Nesting (R1).**
> - With `tail`, `reserved` and `more` absent, every committed tape keeps its text, `tape_hash`,
>   `world_id` and per-tick hash stream. Gate `0x61f9c8529131ff17`, appb `0xe1fa082b26995867`,
>   demo-gb `0xfad880fe08d06645` and the probe, markets, horses and loops pins do not move.
> - `markets_i0_nests_appb` and `many_roles_nest_the_appendix_b_roles` pass unchanged, and so do
>   §3.8's nesting tests.
>
> **E2. The rest point and mode A.**
> - Genesis from unit 1d's `WorkerEconomy` is a fixed point within 1e-12 after 3 ticks. This
>   holds at the base and the 12 targets, at 12, 52 and 365 a year.
> - Mode A passes at L. The mirror's largest gaps are 3.3e-16 (52 a year), 1.6e-15 (12) and
>   4.4e-16 (365), and the engine's must be below 1e-9.
>
> **E3. The verdict.**
> - IW1 is GO. Tier 1 26/26, Tier 2 38/38, Tier 3 39/39 and Tier 3S 20/20. Tiers 3 and 3S are
>   also run at 10·L, with the same classes and the same ticks to tolerance.
> - The speeds, troughs, dead ticks by market and the wall readouts per tier are §6.2's table and
>   its Tier 3S lines.
> - The engine's own elasticity probe sets L. The mirror's τ are §6.1's (goods 106.4 ticks), so L
>   is 22,000.
>
> **E4. The kick sets and the slowest mode.**
> - Every kick set decays at every target: gain_tail at most 1e-3, where the mirror gives at most
>   1.5e-5.
> - The engine's fitted decay at the base is within 0.03 a year of 0.534.
> - The mirror's largest root per tick over the targets is 0.991911, and the technique's rate is
>   exactly 0.951229.
>
> **E5. The cost shocks.** §6.2's cost-shock rows and §6.4's depths:
> - land.mach × 2 bottoms at 0.165 of the new Y\* with 93 dead ticks;
> - tail.services × 2 at 0.516 and res.services.trained × 2 at 0.500, with no dead tick, although
>   Y\* does not move.
>
> **E6. The wall's own readouts.**
> - In the registered battery only JB(0.5) breaches the wall: 11 ticks from tick 0, lowest depth
>   −0.444, largest share 0.101.
> - No other run's depth falls below 0.128, which is land.mach 0.8's path.
> - The displaced shares follow s_t = s_(t−1)·(1 − share(adjust)) exactly while the wall binds.
>   Their thresholds are back within tolerance at ticks 78, 105 and 124.
> - No pop saturates in the registered battery.
> - Every CONVERGED run ends at the wall, and every labour market trades at the end.
>   - A share never displaced stays 0.0 exactly.
>   - A displaced share decays to 5e-323, ten subnormal ulps, where a·s rounds to 0 and the
>     update stops (s = 5e-323 after 30,000 ticks from 0.5).
>   - The threshold x then reads 1.0.
>
> **E7. The families.** Each is run and reported:
> - stocks 31/31, joint2 60/60, joint4 40/40 and basin 516/516;
> - history 79/79 windows in tolerance at their end, with each × 0.5 window's no-machine tick;
> - 12 and 365 a year 64/64 each, Hold identical to Saturate, and tilt 1 64/64.
>
> §6.3's numbers, within the bands above.
>
> **E8. The line control.** IC1 converges 103/103 and its Tier 3S 20/20 at L = 25,000, with §6.4's
> numbers (reported, not scored).
>
> **E9. The edges.** The generator's own solve reproduces §2.4's gaps within 1e-12 at genesis and at
> every target. A gap below 0.2 at a cost target means the tape is not the registered instance.
>
> **What would refute the frame and send it back to the mirror:**
> - a class change in the registered battery;
> - a CONVERGED run ending more than 1e-12 in log off its target's oracle point;
> - a converged run ending off the wall (a desk's share not back to 0), or a labour market dead at
>   the end;
> - the engine's rest point differing from unit 1d's by more than 1e-12;
> - a kick set failing, or the slowest mode at or above 1 at any target;
> - any committed pin moving.
>
> **The scorer** is written and committed before the wave (decision 311). It reads the
> registration's run-by-run outputs and scores the engine's runs against them in the bands above:
> - `registered/runs_*.tsv`, one per set, Tier 3S included;
> - `registered/local.json`;
> - `registered/history_land.json`;
> - `registered/edges.json`.

## 5. Decisions and open items

SPEC §8's proposals FW1–FW9 and open items OW1–OW3 are numbered in STATE.md at this commit, in
this line's range: FW1–FW9 as decisions 394–397 (grouped, since four numbers of the range remain
after them for the commons instance), and OW1–OW3 as O97–O99. Each is Claude's, on the user's
delegation, and open to veto; the alternative named is the registered one (R6).

| STATE | the frame's | what |
|---|---|---|
| 394 | FW1, FW2 | IW1 is the wall instance: 1d's B economy with E7's three types, the trained and the master reserved-only (ε 0), every type's support one basket |
| 395 | FW3 | the roles' additions are three optional fields: the category desk's `tail` and `reserved`, the provider's `more` |
| 396 | FW4–FW7 | the cost targets (land.mach, tail.services, res.services.trained); thresholds in log at the wall; the wall's run grammar; the wall's readouts reported, not scored |
| 397 | FW8, FW9 | IC1 a reported control; the verdict rule P2.1's with 368's Tier 3S |
| O97 | OW1 | a type that sells reserved and pool hours |
| O98 | OW2 | at the wall, labour-demand shocks move wages, not output, but halve output on the way |
| O99 | OW3 | O21's one-type cousin at the wall: the machine desk keeps its whole stock for a tick |

