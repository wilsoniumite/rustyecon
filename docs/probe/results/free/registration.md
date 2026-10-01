# A zero price markets can hold: registration of the free step (P2.4.10, O96, O101)

Dated 2026-09-30. Step P2.4.10 on branch `phase2-s2` (worktree `D:/rustyecon-wt/p24`), on top of
`adf1ec6` (P2.4.9), clean but for this registration. It registers the mirror's predictions for the
engine run of the free step, an optional per-good price step that may post 0, on the instances
IL1 (idle enclosed land at r = 0) and CT2 (two plot-taking types on one commons market), before
any engine code for them exists (R5): at this commit `RawGood` has no `free` field, a posted price
of 0 stops the run (`PriceError::NonFinite`), `RawPricedExit` has no `market` field,
`crates/probe` has no IL1 or CT2, and `tapes/` has no tape of either. PHASE2-S1 §6 item D. Any
later change is a new dated amendment with its own sha256 (decision 282); earlier results stay
reported.

## 1. What is registered

The scan's frame, in [docs/probe/free/](../../free/), as its author left it in
`D:/rustyecon-p24/scan-free/` (label `scan-free`). The repository stores text with LF
(`.gitattributes`). `SPEC.md` and every registered file were written with LF but two (the 50-digit
points and the fine scan's table, written with CRLF), and both sha256s are listed for those two.
`SHA256SUMS` (170 entries) was checked entry by entry against the scan's files on 2026-09-30, all
OK; its own sha256 is `4c564efcb223b0c514fe6f982d43ef86d45e2114a6367946a1510862a4ddaca9`.

| file here | the scan's file | sha256 as registered | sha256 as committed (LF) | what it is |
|---|---|---|---|---|
| `SPEC.md` | `SPEC.md` | `ef6c888fdf654453f9d81d8748bc3a42497ec926c044cbc091a5a52fc3323a04` | the same | the spec: the problem, the candidates, the instances, the scan, the choice, the rule, the proof, the predictions, the proposals |
| `SHA256SUMS` | `SHA256SUMS` | (itself) | `4c564efcb223b0c514fe6f982d43ef86d45e2114a6367946a1510862a4ddaca9` | sha256 of every file of the scan |
| `registered/scanF.jsonl` | `runs/scanF.jsonl` | `3b714d0aadeebc71797c4193959a6b67425a520750af71832971ec5eec99f7b4` | the same | at c 0.5: mode A at 52, the battery (IL1 100, CT2 115) and the dial family (646, 731), 1,594 runs |
| `registered/regH.jsonl` | `runs/regH.jsonl` | `f06064e035843c56fc6b131e8f87a067860f180eaa64242845770195ce7dea84` | the same | at c 0.5: Tier 3 and Tier 3S at L and 10·L, stocks, joint2, joint4, Tiers 1–2 at 12 and 365 a year, mode A at 12 and 365, 725 runs |
| `registered/pred05.out` | `runs/pred05.out` | `86cb60144e98756724813971a2f99d5e852e104687fb4388f8b4ce785ea4b7d3` | the same | the predictions' summary, set by set, with the local roots (SPEC §9.2) |
| `registered/lin_all05.json`, `.out` | `runs/…` | `a2858ebb…cada`, `7d6cbf7e…b35d` | the same | the largest root and kick bar at all 26 targets (SPEC §4.4) |
| `registered/lin_dial05.json`, `.out` | `runs/…` | `2eaa62b5…aede`, `5b1b4aaa…38c2` | the same | the roots over the 17 dials at both bases and the four priced targets |
| `registered/lin_enclosed.json`, `.out` | `runs/…` | `5f959077…c115`, `90a09d83…81be` | the same | CT2's Enclosed point with the commons held at 1.05·r |
| `registered/elasticity_free.json`, `.out` | `model/…` | `35edff3a…5b9a`, `e87b761d…a618` | the same | the elasticity probe and L at 12, 52 and 365 a year |
| `registered/points_fine.json` | `solve/points_fine.json` | `ace69324abb50102ffcf32ac781ed5207482da376d3f23b1eb57f3ddc4ddedfb` | `1ca259d4ab313d7296a057d8b8de006e7285322643d04ca5330fd8257f041459` | the independent 50-digit points at all 26 targets (SPEC §3.3) |
| `registered/check.out`, `compare.out` | `check/…` | `c920c86d…e545`, `4876aefc…3bdf` | the same | the worktree's `ParcelEconomy` at all 26 targets, and its agreement with the 50 digits and the mirror's f64 oracle |
| `registered/trap_check.out` | `runs/…` | `7f205ecf…c9f2` | the same | joint(4,10) under Saturate and the free step: the trap at tick 254 either way |
| `evidence/table_all.out`, `table_E.out`, `table_FG.out`, `scanF.out`, `regH.out` | `runs/…` | `c25cef9a…1526`, `2e4fee8b…0062`, `dbc77f80…5742`, `60b09165…ee03`, `848fe49d…a56a` | the same | the scan's tables (SPEC §4) |
| `evidence/nest_free.out` | `model/…` | `06b3c60e…b3ee5` | the same | the mirror's nesting: 33 checks bit for bit (SPEC §4.1) |
| `evidence/fine.out` | `solve/fine.out` | `a623dffbe4a1b295857ef8bbeff8cd1b9e769e4d78d1ea3e6cecdb6b2dadb98f` | `831f4e7c08b09f2058ee625bf4520a487baa5be41aa92333f456a05ac4067e11` | uniqueness: one sign change at every target on a scan 16 times finer (SPEC §3.3) |
| `evidence/lin_c2m.out`, `lin_all.out` | `runs/…` | `cb7aab3c…53e5`, `fdb24696…8ea7` | the same | the local roots of every candidate (SPEC §4.4) |

The scorer (SPEC §9; decision 311) reads `registered/` from this copy.

The mirror the predictions come from stays in `D:/rustyecon-p24/scan-free/model/`, as the frames'
did, and so do E0's traces (`e0/`, 2,001 rows each, 9.2 MB): read in place, their sha256 in the
committed `SHA256SUMS`.

```
87a9ef6aef7b0187abec6bf53c54755db54cb339850f7cf867400f9461257c96  model/fm.py          (the free mirror: `tick`, `free_step`, `pop_decide`, `genesis`)
4d28d20bb6475ea93050743e718c54bab3e61fa3c92c10cac9f9b7a4dd9bdd86  model/cm.py          (the commons frame's registered mirror, unedited)
624a82bfbdbbc3b8b55f479b4fdce0fbe7cdc838501959c5710b16e82906b7a1  model/fb.py          (the battery: observables, grammar, runaway bound, readouts, classes)
aa1ea0804f64d9e5f8d3ebed8005c444d192adb7b0c5c2d7052d108059294972  model/ofree.py       (the mirror's f64 oracle in wage units)
e299bcd72bda7bd800ca9056418ff581c83acf7afa5dfe74721f5ae2fd85df41  model/run_free.py    (the sets, the dial family, Tier 3S's lists)
0cd0db74902cab86a1d8ee347a3c707da5261916be296c6320012bbafd77e6e4  model/trace_free.py  (E0's traces)
d445df0ebd70383a2a3d20859736f3fb768523a9617b1ae7d9b7fa79a5074410  model/lin_free.py    (the local roots)
cda41430c4b342114e6c00a64c473e90cf9dccf6d1c4da78f17fe0b79f3a11bd  model/elasticity_free.py
2b8afafdbbcd456913c00a6707fc3f836ddc2ffca8a6915bdfffd8a17cd73a6c  solve/fsolve.py      (the independent 50-digit solve)
b591afa29340a582636301d674f9ba936a93568f27f700a5e71d8dbfa7d1facc  e0/CT2_hold.csv
1b6efed769ae2f31598eec2957a14a061c5b05603a4693b0041c526d14041b14  e0/CT2_exit.To_17.55_genesis.csv
74ecf32db29fa54d3db819518c4199294a1df7964741edbf0563cd60e2e3f1a1  e0/CT2_b.food_1.2_genesis.csv
0b37ce3b745ef71b662ea3a3e8f480f99adbec60bd75529707c3b791ad06d42d  e0/CT2_exit.To_9.75_genesis.csv
7b77d3a73437cadb9e43eade94f903ce34bff733ed06d13f0738d1294fd641fd  e0/CT2_p_labour__2.csv
a40b15807715759d25d303fe9bd2e731ee95540f953d8d62715e7b1c2783be63  e0/CT2_p_land__2.csv
a83e2f30c7debd8f7a9fa6e668a96303a37e23bb509240b5705c7c1b8547dc6c  e0/IL1_hold.csv
280b5913cf90d369c80d06f14ac20fe2d04d54ed47dd585606b64b9591d628d6  e0/IL1_p_land__0.025.csv
42cae5a8311ce3e4bb0f010a2abac5bd6272140282c1fa278828b1286af5eb79  e0/IL1_inst.land_260_genesis.csv
3fabefec35c5498fbc10503e9d07da5f0d6f5858e19b6fecff41c761fb150c11  e0/IL1_p_labour__2.csv
8b05f218a3cfcbac2a611affc658a7657e06ed7b0c139424d6d8594c0075b151  e0/IL1_JB_2_.csv
```

## 2. The order of work

1. The scan ran on 2026-09-30 in scratch (label `scan-free`), reading the worktree at `a483ed0`
   and changing nothing. P2.4.1–P2.4.9 (wave A's registration and machinery, the trap's remedy,
   the type switch) landed during and after it. The trap's build added the exit's optional
   `pace` and the switch's the workers' optional `pool`; neither runs at IL1 or CT2, and neither
   touches the price update, admission or the tape's goods. No engine code for the free step and
   no engine run of IL1 or CT2 exist.
2. This commit fixes the registration in the repository, and STATE.md numbers the scan's
   proposals (§5).
3. Next: the build (SPEC §6: the good's optional `free` in core and markets, admission at a price
   of 0, the workers' optional `exit.market` in the agents; IL1 and CT2, their generator and
   tapes; the harness in wage units and at the commons market; the tests of §6.5;
   `docs/probe/FREE-RULES.md`), with every committed tape's text, hashes and streams unchanged.
   Then E0, the trace diff against the registered traces on SPEC §9.1's eleven runs, before any
   scored run. A real departure found by E0 is a dated amendment committed before scoring. The
   scorer is committed before the wave (decision 311).

The sources at the base (sha256 at `adf1ec6`), which the build will change:

```
77e7d6279b815e7cc2a7cdb59784df757c5d0abe54ba2b151b04bdc4f2476471  crates/core/src/tape/raw.rs
9ac641fe37bcb9a3c8eb2a31b85ee7c0ab263fee98ba5e1933a675d7db01df73  crates/core/src/tape/mod.rs
7a955d7fc00a6b144179f39101166fac365869737a36c4c3de359f4fe3ee1d4d  crates/core/src/world.rs
3ddb982539c9f581781a1c2be4206f113ad493bbb3709c689f36b76d4afd430c  crates/core/src/apply.rs
d27de1d860027c7d46905f428c304c87775d0e541187a97a8e369bebbb2fa669  crates/core/src/state.rs
5c829b83a59818cfc284784cdd0413847449f9e3582cf458a5ca0b66a6c7d670  crates/markets/src/prices.rs
851198317363cafff64b36d59ec2a1b8a3694488c35bcb8d354ab988ca8c1acd  crates/markets/src/order.rs
05b6cae818490589feda9d9e716091b69fad4fa2755c5795f26e273585bb487c  crates/markets/src/lib.rs
a1a88c518b01f017372a5a006944f2d1389886338ed799ba743e56ff4945d27d  crates/certify/src/kick.rs
dcff6c16a8b697f9f0f0184a63a3eaf54ca519750d005cda16fc9bc1a8daddb5  crates/certify/src/battery.rs
a8085dc825de7755cb43e75de11103feaea8427b539df990e4040c1348731a07  crates/agents/src/roles/many/spec.rs
95c8dbb9578735059afd8511e85d5f77695d801a443783ea3c1b87eaade7c43f  crates/agents/src/roles/many/rules.rs
dd5de12956efcf062e452978578467aaf5a2b3887f0abca2dbd9b48aafc9024c  crates/agents/src/cast.rs
cab1787c6cfc8f365bab64defb1966df79e9d90f1bd72147bb6ead9a5ced0195  crates/agents/src/lib.rs
3f1f251e9dd85ee21ea2791f3f996443693ab0a2c19ab106b7369492d664328f  crates/probe/src/markets/instance.rs
927c8eda693d4937e43f66236d2de979ba82611678c49ab737f23117203b120d  crates/probe/src/markets/setup.rs
35bf0daab7fd8ee523ceb7f11f425255e1c144a8f0c3fbbc0a818e269346524b  crates/probe/src/markets/harness.rs
8541b3e8d2765c1e6bd3453c1407b4efdb9079614b6add657df3c5fe56bb55f3  crates/probe/src/markets/perturb.rs
5234583ceba5585d309de0d2031099feb3ef9373942f073e32ef6836a2c8f76c  crates/probe/src/markets/probes.rs
bc344ad83730ace633e30a59b8d0df03036d3c02f8df8cf1ea3eed5d363b5e56  crates/probe/src/bin/markets.rs
4574d509b889a2d90dd241fadaa84162e7a494aebabdcd945992f7cb34cb803e  crates/probe/src/bin/markets-tape.rs
ff80317c3ee21033fe391917e7fb17e521d322ac857df6be57bbc723b231a162  crates/gui/src/vm/pricestep.rs
```

The pre-build binaries (`adf1ec6`'s code) and every committed tape's 2,000-tick hash stream from
them are in `D:/rustyecon-p24/build-free/base-*` on WSL and Windows; the two machines' streams
equal each other for all 30 tapes.

## 3. Readings the build takes, declared before it

SPEC is registered as written. Where its text leaves a choice, is inexact, or leaves out a ruling
that binds it, the build takes these readings. Each follows the registered mirror, so none moves a
prediction of SPEC §9.

- **The field and its dial.** `RawGood.free: Option<RawFreeStep { reference, scale }>`, absent
  not written, as `untraded`; resolved `GoodDef.free: Option<FreeStep { reference, scale }>`, left
  out of `world_id`'s bincode when `None`. IL1's tape writes `free: Some((reference: "labour",
  scale: "free.land"))` on `land`, CT2's `free: Some((reference: "labour", scale:
  "free.commons"))` on `commons`. c is the param `free.<good>`, `Dimensionless` 0.5, basis
  `Assumed("FREE-SPEC: the scan's window 0.3–1 at C2m, c 0.5")`, written with the dials of a free
  instance and at no other; `--set free.<good>=V` sets it, and no dial family scales it (the
  mirror's dial family does not).
- **The step.** For a good with `free`, `markets::next_price_free(one_sided, p, k, S, D, shift)`
  with shift = c·p_ref: under `Hold` a one-sided market keeps p; else x = imbalance(S, D),
  q = p·exp(k·x) + shift·expm1(k·x), each product rounded once and then the sum, and the posted
  price is q where q > 0 or q is NaN (a non-finite result still stops the run), else +0.0. c is
  read at use time through its `Value` site; p_ref is the reference's posted price at the same
  node in the phase-start book. `next_price` itself is untouched.
- **Zero where the good is free-able, nowhere else.** `update_prices`, `apply`'s `SetPrice`,
  `SetEma` and `ScalePrice`, and the genesis price check accept +0.0 exactly for a good with
  `free`; every other keeps finite and positive. A `ScalePrice` of a free price posts
  0·factor = 0, its factor still positive. The checkpoint's book decode has no world, so it accepts
  a clean 0 and `SimState::validate` refuses a zero price or EMA for a good without `free`; a
  checkpoint of any world without a free-able good loads and refuses as before.
- **Admission.** A buy at a posted price of exactly 0.0 is feasible in full, its quantity, its
  budget (0 or more) still taken from the dry holding: a branch before `num::max_qty`. Every other
  admission is today's.
- **The load checks** at `goods[<key>].free`, each a `LoadError` with its path: the good has a
  market and the world's rule is `Imbalance` (`.free`); the reference has a market and is not the
  good (`.free.reference`); the scale is `Dimensionless` (the resolver's `UnitMismatch`) with a
  genesis value above 0 (`.free.scale`).
- **The kick set and the probes.** Certify's kick set does not kick a market posting 0 at the
  kick tick: 0 has no log price to move, and the mirror's roots hold such a market at 0 (SPEC
  §4.4). `kick_gain` reads a market whose kicked and base prices are equal as a gap of 0, which is
  |ln 1| = 0 bit for bit wherever they are positive and gives 0 where both are 0. The engine's
  elasticity probe leaves out a market posting 0 at genesis, with τ 0, as `elasticity_free.py`.
- **The commons market's field** (FG4). `RawPricedExit.market: Option<Key>`, absent not written;
  resolved `PricedExit.market: Option<GoodId>`, skipped when `None`. At resolve: a traded good,
  not the pop's labour, a basket item, the exit good or the plots' land. At `Cast::new`: `Instant`
  and free-able. `market` with `pace` is refused (not scanned together). The pop's support may come
  through its provider's `transfer` or a `more` transfer (decision 395): the exit's check that a
  provider pays the pop reads both.
- **The pop's rule** is `fm.pop_decide` in its order, on the pop's own params (N_i, χ_i, s₀, s̲, h,
  T_o,i per tick) and posted w, P_s, p_g, r and r_o: e₀ = p_g·s₀; a plot pays at r where
  r·h < p_g·(s₀ − s̲); r̂ = r there, else p_g·(s₀ − s̲)/h. While r_o < r̂: hours n(fma(−r_o, h, e₀))
  and bid h·(N − hours). Else: hours n(fma(−r̂, h, e₀)) where a plot pays at r, n(p_g·s̲) where not;
  G = h·(N − hours); bid min(G, T_o,i); T_p = G − T_o,i where a plot pays at r and G > T_o,i.
  n(e) = N·F(e) with P2.3's F. The state keeps F; the hours offered are N·F.
- **Its orders and budget.** The pop mints its share T_o,i as an endowment when above 0 and offers
  all of it; it bids for `bid` of the commons, and where T_p > 0 for T_p of land at r with
  P = min(r·T_p, C), C its coin as the phase began. The baskets' budget is share(spend)·(C − P),
  decision 398's, and the commons' budget is min(r_o·bid, (C − P) − that budget): the rent is paid
  from what the baskets leave, and the chain's total is P + baskets + commons. This is the mirror's
  arithmetic, whose fills carry no budgets and whose baskets' budget leaves the commons' rent out
  (`fm.tick`, `cm.commons_market_decide`), wherever the coin covers the rent; E0 checks that it
  does on every tick of its runs. SPEC §6.3's "the commons bid and the land buy go first" is read
  as: both are posted in the chain beside the baskets, the land's rent reserved before the
  baskets as 398's, the commons' not, so that the baskets are the mirror's.
- **Produce.** The pop eats its baskets, burns every unit of land it holds (398's), and burns
  min(held, bid) of the commons as `Consumption`, bid read again through the rule at the same
  prices and params: what it bought. The unsold part of its own share dies at ageing, as the
  provider's unsold land does.
- **IL1** (`Instance::named("il1")`): I1's economy with the workers one priced type (N 10.4 a
  year, χ_max 2, exit (6, 0, 2.7) in food), a commons of 0 (`inst.commons` 0, no open parcel for
  the oracle), and land free-able. Its cost coefficients are land.mach, b.food and `inst.land` (T a
  year, 572, 468, 1040, 260). Its point is unit 1e's at the idle stretch in wage units (w 1, r 0);
  at a Scarce target the harness reads prices over the oracle's v. Genesis: w 1, land 0, the
  other prices the point's, the provider's coin 0, the workers' w·S/share(spend), the desks'
  P2.1's with r = 0. Tape `tapes/markets-il1.ron`.
- **CT2** (`Instance::named("ct2")`): I1's economy with two plot-taking pops `workers.wa` and
  `workers.wb` (classes `wa_workers`, `wb_workers`; params `inst.<t>.workers`, `.chi_max`,
  `.exit.gross`, `.exit.floor`, `.exit.plot` and `.commons`, its share a year, 9.75 each), no
  `workers` pop and no `inst.workers` or `inst.chi_max`; the good `commons` (`Instant`,
  free-able, `rate.commons` 1.3 a year in C2m after land's); the provider paying wa by `transfer`
  and wb by `more`. Its coefficient `exit.To` (the commons a year: 21.45, 17.55, 39, 9.75) sets
  each pop's share to V/2, two changes at one tick, as `enclose` makes two. Its point is unit
  1e's with two worker types; genesis prices in rent units with the commons at r_o, and each pop's
  coin the mirror's: r·T_p,i + (N_i·P_s + w·h_i + r_o·(T_o,i − bid_i) − r·T_p,i)/share(spend), its
  hours, bid and T_p from the pop's rule at the genesis prices; the provider's
  τ + (r·T − τ)/share(spend.provider), τ = Σ N_i·P_s in pop order. Tape `tapes/markets-ct2.ron`.
- **IL1's observables are 21, not 20.** SPEC §6.4 lists them (every price over w, each desk's
  threshold, each market's cleared volume, each desk's output) and counts 20; the list is 5 prices
  (the type's and the four categories'), 4 thresholds, 7 volumes and 5 outputs, which is the
  mirror's `fb.targets_of` and what the predictions were scored on. Land's volume target is the
  land in use, T_m + T_p (T at a Scarce target). CT2's are the commons frame's 22, the commons'
  price and volume not among them (the mirror's).
- **IL1's starts.** A price start's distance is over the 5 prices over w and the thresholds (the
  mirror's `fb.d0`); `p[land]=V` has distance ∞ (the mirror's); JA is nominal at IL1, never
  VACUOUS, as N. `p[M]=V` sets market M's genesis price to V times labour's (the mirror's `p[M]=V`
  term), applied after the factors of the same run.
- **The runaway bound.** A free-able market's next price stays within [0, 1e6·max(its genesis
  price, its reference's)], the harness's genesis (decision 399 as amended); every other price
  keeps [1e-6, 1e6] times its genesis.
- **Mode A and the dead-tick rule.** Mode A's check does not read a free-able market's fills while
  it posts 0 (its idle part is its sellers' rationing at a price of 0), and still requires it to
  trade. The commons at CT2 is out of the dead-tick rule, as the mirror's `fb.run` leaves it out;
  IL1's land is in it, against the land in use.
- **The readouts** (`free.*` in `stats.tsv`, one set per free-able market; CSV columns last): the
  ticks it posts 0, its first such tick, its switches between 0 and a positive price, and its price
  over its reference at the end, all on next prices as the mirror's `free_ticks`, `free_first`,
  `free_switches` and `free_end`; the oracle's (r/w at IL1, r_o/w at CT2); the regime at the end,
  at IL1 through the workers' rule with `Idle` where it says `Enclosed` at r = 0, at CT2 from the
  commons' posted price (0 `Commons`, below r `Crowded`, else `Enclosed`); the provider's lowest
  coin, absolute. At CT2 the CSV adds each pop's commons bid, offer and net rent r_o·(bid − offer)
  through the pop's rule.
- **Tier 3S and the families.** At CT2 `coin.workers*F` scales both pops' coins, as the mirror's;
  at IL1 the provider's coin is 0, so Tier 3S has 22 runs and stocks 39, as the mirror's lists
  (`run_free.tier3s_jobs`, `stocks_jobs`). `joint` draws in the mirror's order: labour, land, the
  categories, the type, then the commons (O106).
- **E0's comparison.** Against the registered traces in `e0/`, read in place: every positive
  price within 1e-12 in log, a free-able market's price exactly 0.0 on exactly the trace's ticks,
  each desk's share absolute, each desk's coin and stock and each household's coin within 1e-12 in
  log. The engine's CSV row of tick t carries the prices before tick t (the trace's row t) and the
  coins, shares and stocks after it (the trace's row t + 1). The mirror's genesis is its own f64
  point, the engine's the worktree's oracle; they agree within 6.5e-16 (SPEC §3.3), which the
  1e-12 absorbs. E0's first joint draw at CT2 compares `joint(2,1)`'s genesis prices with the
  mirror's `fb.displace`. Beside it, a diagnostic diff seeds the mirror from the engine's own tape,
  as P2.4.6 and P2.4.9 did.
- **The GUI.** The explainer computes a free-able market's step through `next_price_free`, from
  the recorded reference price and the scale's recorded value, so `equal` holds; the waterfall's
  step for such a market is ln(p′/p) of the free step where both prices are positive, so the shift
  c·p_ref·expm1(kx) is explained and not left in the residual.

## 4. The predictions

Quoted from SPEC §0 and §9:

> **GO for the free step at c = 0.5, on both instances.** A market may carry an optional free
> step. Its price moves as p' = p·e^(kx) + (c·p_ref)·(e^(kx) − 1), with p_ref the posted price of a
> reference good (labour) at the same node. A step that would take the price to 0 or below posts 0:
> the good is free.

> **The verdict, per instance** (MARKETS-SPEC §7.9 with Tier 3S): GO where mode A passes, every
> non-vacuous run of Tiers 1–3 and Tier 3S is CONVERGED, Tiers 3 and 3S are CONVERGED again at 10·L,
> every target's kick set decays, and every CONVERGED run ends with its free-able market at the
> oracle's price (0 where the oracle's is 0, positive where it is positive).

SPEC §9.2 predicts IL1 and CT2 GO: IL1 Tier 1 25/25, Tier 2 33/33 (+4 VACUOUS), Tier 3 36/36
(+2 V), Tier 3S 22/22, the same at 10·L; CT2 30/30, 40/40 (+2 V), 41/41 (+2 V), 24/24, the same at
10·L. L 56,000 (IL1) and 141,000 (CT2) at 52 a year. Mode A exact, D̂ 2.2e-13 and 8.9e-13, the
free-able market at 0 on every tick. Every kick bar passes, the tightest e^-19.6 at CT2's b.food × 2
(PL 0.999846). The dial family: IL1 612/612 (+34 V), CT2 697/697 (+34 V). The families, reported:
stocks, joint2 and joint4 (1 of 40 at IL1 and 7 of 40 at CT2 in the subsistence trap, at ticks
254–276), Tiers 1–2 at 12 and 365 a year. Every CONVERGED run ends with its free-able market at the
oracle's price. The bands are SPEC §9.3's.

**What would refute it** (SPEC §10): a free-able market ending priced at a free target, or free at
a priced one, in a CONVERGED run, or leaving [0, 1e6 × its scale]; a live rest point off the
oracle's; a pin that moves, or a committed tape's `world_id` that moves when `free` is absent; an
engine Tier 1–3 or 3S run of either instance not CONVERGED, or a kick set that fails.

## 5. Decisions and open items

SPEC §11's proposals, numbered: decisions 416 (FG1: the free step is the zero price markets hold),
417 (FG2: c 0.5, labour the reference), 418 (FG3: IL1 under the stated transfer), 419 (FG4: CT2
and the pops' commons market), 420 (FG5: one plot-taking type keeps decision 398's rule), 421
(FG6: IL1's harness in wage units), 422 (FG7: the verdict) and 423 (FG8: decision 160 stays
untested); open items O124 (OF1: the Enclosed regime with several pops), O125 (OF2: c per
instance), O126 (OF3: IL1's accounts), O127 (OF4: the commons' slow mode at a small rent), O128
(OF5: the one-sided free state) and O129 (OF6: the regime readout of a commons market). STATE.md
has them in full, each open to veto.
