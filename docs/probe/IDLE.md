# IDLE: the idle machine market's remedy, and M6 (L0.1–L0.7)

Dated 2026-09-29. Steps L0.1–L0.7 on branch `phase2-loops`, from `reboot` at `401f7b1`. Frame:
`D:/rustyecon-p2l/idle-scan/IDLE-SPEC.md` (IDLE-SPEC, sha256 `c5c6527a…634e`), amended after the
engine review by [results/idle/IDLE-SPEC-A1.md](results/idle/IDLE-SPEC-A1.md) (sha256
`cfa55ad1…fdf8d`). Registration, before the rule existed in the engine:
[results/idle/registration.md](results/idle/registration.md) (L0.3, `96cba59`). Build: L0.4
(`882e9f8`), L0.5 (`fac7db4`), [HORSES-RULES.md](HORSES-RULES.md) §10. Runs and tables: L0.6
(`15b05af`), [results/idle/](results/idle/README.md). Fix round: L0.7. M6: L0.1 (`ba6938d`).

**Conditions of every number below, unless its line says otherwise.** P2.2a's instances and
battery (HORSES §0): rule A's horse, bred by M2 and hired out wet by M3, C2g, 52 ticks a year,
ρ 0, J_b 1 tick, P2.2a's registered L (75,000–258,000 ticks), Tier 3 and 3S again at 10·L, tol
1e-3 in log. The maker's reservation at ψ 0.25. Release build on WSL. Raw runs:
`D:/rustyecon-p2l/idle-engine/runs/`.

## 0. The verdict

| inst | heads × 10: P2.2a → now | battery runs converged | runs where the rule acts at L | lowest horse price / target in the battery: P2.2a → now |
|---|---|---|---|---|
| H1, H3 (δ 10%, 8%; ω ½) | runaway at 164 → CONVERGED | 142/142 each | 3 each | 0.116–0.122 → 0.235–0.239 |
| H2, H4 (δ 10%, 8%; ω 1) | runaway at 138 → CONVERGED | 142/142 each | 10 each | 0.003–0.004 → 0.187–0.188 |
| F1–F4, F7–F10 (families) | runaway at 138–167 → CONVERGED | 142/142 each | 1–10 | 0.001–0.13 → 0.195–0.251 |
| F5, F6 (v1's county) | runaway at 142 → CONVERGED | 138/138 each | 4 each | 0.03–0.04 → 0.20–0.21 |
| P8 (the maker from bought inputs, δ 4%) | runaway at 138 → STUCK at 40,000, CONVERGED at 400,000 | — | — | low 0.215 |

**O47 is closed: GO for the maker's reservation at ψ 0.25, as registered, and no wider.** The
maker offers none of its finished heads while its net markup p_K·(1 − δ·a/κ)/c_m at posted
prices is below ψ. With no offer the horse price holds or rises, so it falls at most one step
below ψ times the replacement cost at the maker's last offer. Every heads × 10 run converges
where P2.2a's ran away, and so does P8. No class changes among P2.2a's 1,980 battery runs, and
the 1,810 where the rule never acts are P2.2a's to every CSV row. The class is the mirror's in
every scored run. There are five misses, none on the refutation list (§2).

What it does not do, as the spec said: it bounds the price, not the maker. Through a long glut
the maker spends its coin on heads it cannot sell and its own herd wears, so the glut ends in a
shortage: installed heads fall to 0.40–0.53 of target and the horse price rises to 8–120 times it
(O52). P8 still needs 473 years to reach tolerance (O53).

**O48 is closed** (L0.1). M6 now exempts the capacity and owner desks only for the durable good
they hold, and a new load check, M5, refuses a role that offers a storable good in full. The
fidelity review's tape, which ran to a fodder desk holding 49 times its genesis fodder, is refused
at load.

## 1. What was built

- **M6 and M5** (L0.1; `crates/agents/src/cast.rs`). A good that lives more than a tick is
  bought only as the durable good a capacity or owner desk holds and nets. A role that offers
  every unit it holds sells only a good that lives at most a tick. The maker, which sells under
  its cover, is exempt. Scripts are their own affair.
- **The reservation** (L0.4). One optional param on the maker, `reserve.<maker>`, off when
  absent, so every committed tape keeps its text and hashes. The rule changes one line of
  `MakerRole::decide`. It is refused on the flow path and under `Hold`. `--reserve PSI` on both
  binaries; the harness adds `markup`, `withheld`, switches and the lowest markup.
- **L0.5.** The resolved `reserve: None` moved every maker tape's `world_id`; it is now left out
  when absent. E1's own hash check found it before any scored run.
- **L0.7, the engine review's fixes.** Off is structural: the maker withholds only when ψ > 0
  (L0.4's `markup < ψ` withheld at ψ 0 wherever δ·a/κ > 1, which no load check rules out). The
  harness reads the tape's own recipes through the rule's own code, `maker_reservation`, where
  L0.4's harness had rule A's recipe written in. M6 gets a test that only M6 can pass.

Every change is off on every committed tape. The gate `0x61f9c8529131ff17`, appb
`0xe1fa082b26995867`, demo-gb `0xfad880fe08d06645` and the probe, markets and horses pins do not
move, and P2.2a's tapes give their 2,000-tick hash streams bit for bit.

## 2. The prediction and the result

| | registered (IDLE-SPEC §8) | engine | |
|---|---|---|---|
| E1 nesting | tapes, hashes, pins unchanged; ψ 0 is P2.2a | as registered, after L0.5 | holds |
| E2 local map | mode A, roots, kick sets P2.2a's | mode A and the base kick sets P2.2a's at 14/14; at b × 2@dated the kick sets pass but move (F4's slowest mode by 0.0066 a year) | holds |
| E3 heads × 10 | CONVERGED × 14; lowest price 0.135–0.193 | 14/14; ticks to tolerance equal at 12, ±1 at two; prices within 0.4% | holds |
| E4 P8 | STUCK at 40,000, CONVERGED at 400,000 | the same; in tolerance from tick 24,601 | holds |
| E5 battery | 1,980 CONVERGED; acts in 84 | 1,980; acts in 85 (F3 x\*/2, named free to flip) | holds |
| E6, E7 families | only heads × 10 changes class | the same; acts in 87 stocks runs, not 88 | holds |
| E8 controls | ψ 1 orbits; ψ 0.75 on P8 orbits | both orbit; the ψ 1 kicks are labelled VACUOUS (O56) | holds in substance |

The five misses: H2 r × 2's lowest horse price is 0.220 of target against the mirror's 0.204,
because the scan's mirror has no genesis carry (the carry mirror meets the engine within 1.6e-12;
O57); the two counts at the step (85 against 84, 87 against 88); E8's labels; and F2's post-glut
horse price at 119.7 times target against a range of 8–112, reported and not scored. The trace
diff before any scored run agreed within 1.6e-12 on H2 r × 2 and to rounding on P8, and on the
glut the engine stays inside the mirror's own one-ulp spread (0.11 in log, O55).

## 3. What the reviews found

The engine review rebuilt `15b05af` on both machines, reproduced every check it tried (17 runs
byte for byte, 1,980 battery rows re-scored, 2,654 non-acting runs equal to P2.2a's) and found no
refutation. Its five minor findings, each answered at L0.7:

1. **Off was not structural.** Fixed; test `a_maker_without_a_reservation_offers_at_any_markup`.
2. **L0.1's M6 test passed on M5.** Both checks refuse the review's tape. A scripted seller of
   stored fodder now isolates M6; test `m6_refuses_a_stored_good_no_role_offers_in_full`.
3. **E2 at the dated target.** The kick sets there are not P2.2a's bit for bit. Reported in the
   results' README and in IDLE-SPEC-A1.
4. **The bound was overstated.** STATE's line said the price stays within one step of ψ times
   replacement cost "whatever the bids". That holds for the cost at the maker's last offer. At
   current costs the markup reaches 0.144–0.206 at heads × 10, about five steps below ψ.
5. **The harness's markup was rule A's.** Fixed; test
   `harness_reads_the_reservation_the_maker_acts_on`, on H1 with a running labour cost the old
   readout ignored. The review's 17-run sample, rerun from L0.7's build, equals L0.6's evidence in
   every row, so no result moves.

Each fix's test fails with the fix undone (`D:/rustyecon-p2l/fix-report/mutants/`). The mirror
review's major finding, a mirror without the genesis carry, is the same fault as this run's first
miss; it is fixed for the loop step's registration (LOOP-SPEC-A1).

## 4. What this means

- **The idle market has a floor that needs no buyer.** Of the scanned remedies, the reservation
  is the only one whose bound does not depend on bids. `Hold` bounds the fall only while no bid
  stands; a floor order has the buyer compute the seller's cost. It is on by default in P2.2b,
  the goods chain's 1750-like instance and the demo's second pass (decision 253).
- **The rule is a step on a quantity, not a price clamp** (R3). The price moves only by
  `next_price`. The maker reads its own recipe, posted prices and one param (R13).
- **The floor is relative to cost, and costs move.** A market that idles for decades rests near
  ψ times the cost at the last sale, which may be far from the cost at the next.
- **The glut still ends in a shortage.** The rule protects the price, not the maker's herd or
  coin. Entry and exit (Phase 3) is the structural answer; the mirror's floor order is the only
  scanned rule that avoided it, at the cost of every glut path (O52).
- **For rule B.** The loop mirror runs the reservation on its maker from bought fodder. Without
  it rule B's gluts run away at ticks 155–173 (LOOP-SPEC LF3), and with it heads × 10 converges
  with no dead tick. The harness now reads rule B's maker correctly by construction.
- **M5 and M6 leave a gap.** No role handles a storable running good yet (O51). Stored fodder
  (the loop's 13-week family) needs a seller that offers I/(1 + b_G) and buyer netting first.

## 5. Open questions

- **O52.** The maker's collapse through a long glut.
- **O53.** P8's slow mode (0.999986 a tick): 473 years to tolerance.
- **O54.** A market that never reopens (the 1750-like instance's switch of technique) is argued,
  not run.
- **O55.** The chatter after a glut, 82–200 switches, where an ulp can decide a path.
- **O56.** Price-kick negative controls read D̂₀ at genesis, so a 1e-9 kick is VACUOUS however it
  orbits.
- **O57.** A mirror that registers for the engine must carry genesis; the loop step's
  registration now does (LOOP-SPEC-A1).
- **O51.** Storable running goods have no role that handles them.

Figures: [fig1](figs/idle/fig1_x10_price_and_markup.png) (heads × 10, price and markup),
[fig2](figs/idle/fig2_x10_glut_and_shortage.png) (the glut and the shortage),
[fig3](figs/idle/fig3_p8.png) (P8), [fig4](figs/idle/fig4_battery_lowest_price.png) (the
battery's lowest price), [fig5](figs/idle/fig5_x10_engine_vs_mirror.png) (engine against mirror).
