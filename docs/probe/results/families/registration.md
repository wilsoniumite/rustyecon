# Wave A's registration: O22's families on I1–I3 and the dial neighbourhood (P2.4.1)

Dated 2026-09-30. Step P2.4.1 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`), on top of
`a483ed0` (P2.3.17), clean. It registers the mirrors' predictions for wave A of Phase 2 proper's
second session, run by run, before any run of the wave (R5): O22's families on I1–I3 (O109;
decision 368, stocks first) and the dial-neighbourhood family on C1, C2 and IW1 (decision 399 as
amended, item 4; PHASE2-S1 §6). No engine run of the wave exists, and none of its code: the wave
adds none (SPEC §3.4). Any later change is a new dated amendment with its own sha256 (decision
282); earlier results stay reported.

## 1. What is registered

The frame, in [docs/probe/families/](../../families/), byte for byte as its author left it in
`D:/rustyecon-p24/families/` (label `families`; every file written with LF endings):

| file | sha256 | what it is |
|---|---|---|
| `SPEC.md` | `23bb3f3d061fcb470a54f9ef785f50b8a1b2a9450ec8f14999827a2f7d9edd50` | the frame: the sets, the mirrors and their checks, the predictions, the bands, the readings, decisions 400–403 and O110–O112 |
| `SHA256SUMS` | `b46896994e892e83d517c9a705cf4e52fa8aeb084099054d4b69e8378c264cf2` | sha256 of the SPEC, every registered output, the harness's lists, the copied mirrors and the runners written for the frame (111 entries, each checked) |
| `registered/i_stocks.jsonl` | `ed0e84269608acc8044ccc8448e56726c30c97349bdbb39edf0b97f12af0b4d0` | stocks at I1–I3, run by run (120) |
| `registered/i_joint2.jsonl` | `03c7c2f9e4f022fcb4338d424db0d8c61aae2a4909af1a17b58c88e551cfda18` | joint2 (180) |
| `registered/i_joint4.jsonl` | `eacec6a8498101f412d53eae0d601b855fe19a8978f17cdba65c9ff01b1f0844` | joint4 (120) |
| `registered/i_basin.jsonl` | `a22b109cdd7dd29b826ffd34e3c29b5713e28c73f58f74748f8477e1a1ef84dd` | basin (1,437) |
| `registered/i_hold.jsonl` | `f385c0feb53980c66d1db76136e7bbfbd2439120cb6dda9585590cc0dd597836` | the battery under `Hold` (303) |
| `registered/i_tilt1.jsonl` | `af94975d731ef032f60cc074abeaee360bb37ae5cf55858f3e14a27dcc41dcb1` | the battery with every tilt 1 (303) |
| `registered/history.json` | `a4a5ce9b83a003a56f5fe45f4277952b2d6ebcb34e472ad82020982137685efb` | the three histories, window by window (243 windows) |
| `registered/i3_map.jsonl` | `a09ca602c6913fb038395ab412c20d5350460dce6bbfae4177e97a5bc3d20f2d` | I3's five map cells: mode A and the battery (600) |
| `registered/i3_map_pl.jsonl` | `b30cfb55205b1934058af8cdf09524c649dfbab53657d8f1179b4b14ed7c1317` | PL at each map cell's nine targets (45) |
| `registered/dial.jsonl` | `b260f636ff164d0c5eca59eee2b0b5f361f3b8202ec44ccbf36aceb5d361ed7f` | the dial family, Tier 3 and 3S at 17 settings (3,281) |
| `registered/dial_pl.jsonl` | `53a27747f10031e8381a8b961bb5485c564a1f447dd2121b74f1b30b2acf31c1` | PL at the base at each setting (51) |
| `registered/summary.out` | `f035ed065e408cb5331efce4a68b6ca679f5b38c97342de3a4e49d24a489bf7b` | SPEC §4's tables, as text |
| `registered/dial_table.md` | `e851dc6dfbd3fd58e6a1e63f2a1730e3e2c2be09a79ed2b1fc4094e564abb00b` | SPEC §4.2's table |
| `registered/draws.json` | `279da17a2a1eb5e4e8d7dc26c5098d6d9951bd3aa0e99e743b68d19d5a907336` | the joint factors as the harness draws them (libm 0.2.16's `pow`) |
| `registered/check_i.out`, `check_i.jsonl` | `bf2e9c6a…081d6d`, `babf355c…7004` | check 1: the I1–I3 runner against P2.1's engine battery, 303 of 303 |
| `registered/check_dial.out`, `check_dial.jsonl` | `75d87a69…8c99`, `4d55b35a…dd84` | check 2: the dial runner at C2m against P2.3's engine, 193 of 193 |
| `registered/check_runaway.out` | `93fc092a3cd87bb79aa3df7c65879f301aa6c9d4acf9865e2cd935675b379a9d` | check 3: the runaway reading, 185 of 190 to the tick, 5 one tick apart |
| `registered/check_joint.out` | `9dc8862b1d7abf3a16a61fa1fa8f548116c085c7d531ed7223f3f7ad908570da` | check 4: the joint draws at I1–I3, 300 of 300 bit for bit |
| `registered/check_joint_c.out` | `3f40745d8f67d1ae41a3089ae74e1c67678394200afd14c5e6840786848cca20` | the same at C1, C2 and IW1 (O110) |

The scorer (SPEC §5; decision 311) reads `registered/` from this copy.

The mirrors and runners stay in `D:/rustyecon-p24/families/model/`, as the frames' did; their
sha256 are in `SHA256SUMS`. The mirrors are copied unedited from the registered frames (SPEC §2);
the runners written for this frame are:

```
84a3d4bbdc7907126d374e572dbe0d2722044d91ac7d51ab455d0561d1045836  model/fam_i.py        (O22's families on I1-I3, on mm_carry.py)
1b0840eb2270ec31e23ae9c29eb30d165d2eac2a5c3fec874bfa13d125fa04a3  model/fam_dial.py     (the dial family, on the frames' runners)
80c0c1706df047fc98d1fde873fbc986166450966b998f68b20d4b591fd89464  model/run_all.py      (every set, to registered/)
91104afab8a0f289003ce4f4c2bf20b40c0f0b8d9e4de1de5e01682256a12feb  model/summary.py      (SPEC §4's tables)
57b224f7b9dfdefd71fa733f2fa520a0d9bf20764d7b4b98b33da2e264412404  model/dial_table.py   (SPEC §4.2's table)
7e03e93ac0bae4384028280c71cdd8c3604140d61c0c0b5fd224bceb3555f599  model/make_draws.py   (draws.json)
33044bdc460e82f3096b7dcdc930c25791d53675b8c26f0f241b73fca347739d  model/check_i.py
0969a786dcaba4cc5e1dc3832350726da81ed4775e86fc9e42f62ce482356a55  model/check_dial.py
cffa0936f195e462ed88240f0dd407a7a24d11eafb8590e49ecde4253c3a3c79  model/check_runaway.py
67e28589a3ae5def94fa281e1e4a3389797f11455411efb77c2134c96af94b5b  model/check_joint.py
19867bf8e2a2ff95b429f4e08897f70f4b59bcd938ca15f3f16cca74dab43608  model/check_joint_c.py
4b5b889ee6f21f878575e29318c114d8fe81b3b7be4f0c81d77b7903c824ebc4  draws/Cargo.toml     (the scratch program, outside the workspace)
29b61df11d5a029f58bf6497f72451c64587c69997e91f38d90158318b0b9d96  draws/src/main.rs
```

## 2. The order of work

1. The frame was written on 2026-09-30 in scratch (label `families`), reading the worktree at
   `a483ed0` and changing nothing. The harness was built there (WSL release, `/root/scratch/
   target-p24`, sha256 `be3266ab…42e9`, the P2.3.12 binary's), and only its `list` and `tape`
   commands were run, which run no tick.
2. The checks of SPEC §2 ran first, each against engine runs already committed (P2.1's battery,
   P2.3's scored wave) or against the harness's tapes; then the predictions (`run_all.py`).
3. This commit fixes the registration in the repository, and STATE.md numbers the frame's
   proposals as decisions 400–403 and open items O110–O112.
4. Next: the wave's machinery, committed before its first job (decision 311): the job list, the
   job and runner scripts, the gather script, the scorer and its self-test on these records
   (`docs/probe/results/families-wave/`, P2.4.2). Then the wave.

## 3. Readings declared before the wave

Quoted from SPEC §6:

> - **×0.75, not ×0.8.** Decision 399 as amended (item 4) lists "× 0.9, 1.1, 0.8 and 1.25"; its
>   source, PHASE2-S1 §5 and §6, says "±10% and ±25%", and the session's brief says ×0.75. The
>   family takes ×0.75, the −25% (decision 401).
> - **Five tilts.** 399 lists tilt 0.05, 0.1, 0.25, 0.5 and 1; the brief 0.05, 0.1 and 1. The
>   family runs all five.
> - **L at every setting is the instance's registered L**, not the elasticity rule's at that
>   setting (at ×0.75 of every rate the rule would give about 188,000 at C1). Every run the mirror
>   predicts converges in under 1,400 ticks or runs away in under 400, far inside L.
> - **The variants run Tiers 1–3**, as the commons ran its; the wall ran Tiers 1–2.
> - **The history runs 81 windows of 1,500 ticks**, the commons' form; the wall ran 80 windows and
>   a last one of L.
> - **The map cells' L is 484,000/fr**, P2.1's map rule (MARKETS `families.csv`).
> - **The joint draws are the harness's own doubles** (`draws.json`), not Python's `**`.

**Disclosed**: six of the dial family's 51 (instance, setting) cells overlap engine runs made at
P2.3.16 (the fidelity review and its recheck: Tier 3 at every price rate × 0.9 and every buffer ×
1.1 at C1, C2 and IW1, at 6,000 ticks). The predictions here come from the registered mirrors,
unedited, and give those runs' classes and runaway ticks (SPEC §2).

## 4. The predictions

Quoted from SPEC §0:

> **A1, O22's families on I1–I3: every run converges.** 3,063 runs, 3,063 CONVERGED; no DIVERGED,
> DEAD, STUCK or ORBITING run; none of the three histories runs away; all five of I3's unrun map
> cells are GO, with every kick set predicted to pass.
>
> **A2, the dial neighbourhood: C2 and IW1 converge at every setting; C1 falls into the subsistence
> trap with slower prices, faster buffers or a tilt of 0.25 and more, and nowhere else.** 3,281 runs: 3,229
> CONVERGED, 34 VACUOUS (C2's `commons=62.4` twice at each setting, by construction, O103) and 18
> DIVERGED, all at C1, all Tier-3 price runs, all through the trap.

SPEC §0 and §4 have the tables; `registered/` has every run. The bands (SPEC §5): every class
exactly; ticks to tolerance within 10%, three runs a set within 25%; runaway ticks within 5% on the
harness's reference; dead ticks within 10% or 5; the lowest baskets within 0.05 where the mirror's
is above 0.1; every CONVERGED run ending within 1e-12 in log of its point; the history window by
window; I3's map cells' mode A, kick sets and verdicts. What would refute the frame (SPEC §5.5): a
class not the mirror's; a CONVERGED run ending off its point; a scored kick set or a map cell's
verdict not as registered.

## 5. Decisions and open items

SPEC §7's proposals, numbered: decisions 400 (the wave's composition, and no new grammar), 401
(the dial neighbourhood's settings, tiers and L), 402 (the mirrors and their runners) and 403 (the
bands and the refutations); open items O110 (the frames' joint draws are glibc's, the harness's
libm's), O111 (I2's history is slower than its window) and O112 (the dial family reads its runs
without their targets' kicks). STATE.md has them in full, each open to veto.
