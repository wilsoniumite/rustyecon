# REPORT: the Phase 2 probe

Dated 2026-09-26. Step P2.0.2 on branch `phase2-probe`. Frame:
`D:/rustyecon-probe/frame/PROBE-SPEC.md` (PROBE-SPEC). Build: P2.0.1 (`55c9e88`),
[RULES.md](RULES.md). Registration:
`D:/rustyecon-probe/run/registration-analytic-first-C2.md` (sha256 `4ea97124…`).

**Conditions of every number below, unless its line says otherwise.** The harness `probe` built
from `55c9e88`, clean, in release on WSL. Design-analytic-first's C2 as built: the cash rule,
tilt 0, Leontief at the planned x, `Saturate`. 52 ticks a year; L = 20,000 ticks; tol = 1e-3 in
log on each of ten observables. D̂ is the largest gap over tol, so D̂ ≤ 1 is inside tolerance.
Raw runs stay in `D:/rustyecon-probe/runs/`. The tables in [results/](results/) are derived from
them by `D:/rustyecon-probe/report/make_results.py`, and the plots are in [figs/](figs/).

## 0. The verdict

**Yes, on this instance and this design.** Agents that decide at the paper's margins find the
oracle's equilibrium of the SSRN Appendix B flow economy, and they find it from far. They return
from each price ×2 or ÷2, from single prices out to ×/÷128, from all prices displaced together
within ×/÷4, from the technique at x\*/2, from machine stocks ×0.01 to ×10, and from cost shocks
b′ = 0.2 to 0.8, whose targets lie up to 0.78 in log from the start. Every battery run ends
within 1.03e-14 in log of the oracle on all ten observables, inside tolerance after 268–677 ticks
(5–13 years). The dials are C2: price rates of 5.2 a year on labour, 2.6 on the good, 1.3 on
land and machines; desk turnover 5.2, technique 2.6, spending 13 a year; no bands. By §4.9 it is
**GO**, and both reviews confirm it. Three things bound it. The paths are violent: a cost shock
that lowers equilibrium output by 12% cuts output by 85% on the way, so GO says where the agents
end up, not that they adjust as the paper's economy would. The GO region is bounded: labour's
price rate cannot be halved, nor the good's quadrupled, without losing some Tier-3 run. And the
instance is the flow benchmark (ρ = 0, δ = 1, J_b = 1): durability and interest are untested.
PLAN Phase 2's kill condition is not met.

## 1. What was built

- **Rules** (RULES §2), four behaviour kinds in `crates/agents/src/roles/`. *Workers* offer
  N·F(ln(1 + w/P_s)) hours at once. *Workers and the provider* spend share(13/yr) of their coin
  on baskets of 1 good and h land; the provider offers all T of land and pays the workers
  min(N·P_s, coin), recording due and paid (R12). The *good desk* carries 1 − x and closes
  share(2.6/yr) of its gap to the task measure at posted w/p_m; it spends share(5.2/yr) of its
  coin on inputs and makes output by Leontief at the planned x. The *machine desk* runs the same
  cash rule, keeps min(a·q, K) of last tick's output and orders labour and land on the whole plan.
- **Why the rest is the oracle's.** No rule reads a margin for scale. A desk's coin is constant
  only where revenue equals outlay: p = s·w + J(x)·p_m (M2) and p_m(1 − a) = λw + br (M1). The
  technique rests at M3 and participation at M4. No rule has a band, so every R_o is 0 and unit
  1a's uniqueness proof carries over (PROBE-SPEC §3.2). No rule reads a volume, a fill, another
  actor or the oracle (R13).
- **Engine changes: none.** Core, markets and engine are unchanged; the gate world's hash is
  `0x61f9c8529131ff17` as before, and the schema stays 1. Agents gained the roles and a checked
  `AgentDelta::SetState`. The new crate `crates/probe` holds the tape generator, the runner, the
  classifier (PROBE-SPEC §4.5) and the elasticity probe; nothing depends on it.
- **The tape.** `tapes/appb.ron` is generated from the oracle's point: N = 4 and T = 10 a tick,
  the oracle's prices with r = 1 coin, one tick's output in stock, stationary coins. Machine
  services and the good live one tick: made at t, sold and used at t + 1, the paper's build lag.
- **Lineage.** Three designs, three judges; all chose analytic-first, with twelve grafts (RULES §5).

## 2. The prediction and the result

| prediction | source | result |
|---|---|---|
| The live rest point is exactly the oracle's | PROBE-SPEC §3.2 | Final gaps ≤ 1.03e-14 in log; volumes equal N_a, T, Y·J and Y to 5e-15; every coin equals its closed form to 1.3e-14 (review-fidelity) |
| Mode A tests linear stability | PROBE-SPEC §4.6 | It holds (D̂ 8.9e-13) but tests nothing: genesis is an exact f64 fixed point, and over 20,000 ticks w moves once, by one ulp, and r, p_m and p not at all (§5) |
| One-tick ε_d − ε_s: labour −1.35, land −0.84, mach −0.90, good −0.30 | the design's mirror | Equal to 4 digits (`run/elasticity.out`) |
| The same under instant best responses: −7.38, −0.58, −2.11, −0.27 | PROBE-SPEC §4.8 | Labour and machine demand respond less within the tick, because the technique moves gradually |
| Local growth PL = 0.98694 a tick: a ×2 start in 498 ticks, a 5% start in 298 | registered prediction | ×2 singles 407–602 ticks; Tier 1 268–449. Over the sweep, measured over predicted runs 1–2×, across two decades (fig6c) |
| C2 converges 57/57 | the design's mirror | 57/57 in Rust, and Tier 3 holds at 10·L |
| July's margin-step rule fails | registration | 57/57 DIVERGED: the good price passes ×1e6 by tick 290–1,055 (fig4) |
| Stable where PL < 1 | registered prediction, b = 0.4 | Right in 103 of 117 cells; at the worst target b, and with the kick check applied, consistent in 115 (table below) |
| Basin about ±5%, then orbits or death | July (ADDENDUM §3.1) | One price at ×/÷128 converges |

The sweep's cells against the prediction taken at the worst of the five b targets, with the
labels corrected by the kick check (§5; [results/sweep.csv](results/sweep.csv)):

| | GO | LOCAL | NO-GO |
|---|---|---|---|
| predicted stable at every target | 88 | 1 (rate.good ×4) | 1 (rate.good ×8) |
| predicted unstable at some target | 0 | 8 | 19 |

The two misses are basin limits the linear analysis cannot see: the good market goes one-sided
and its price runs away under `Saturate`.

## 3. Mode A and mode B

**Mode A** (`probe run hold`, 20,000 ticks). On the registered tape every fill is at least
1 − 1e-15, spoilage at most 9.6e-16 of volume, coins constant to 1e-15. Largest D̂ over the run:

| setup | registered | ex-post | ceiling | step | Hold | tilt 1 | 12/yr | 365/yr |
|---|---|---|---|---|---|---|---|---|
| D̂ | 8.9e-13 | 8.9e-13 | 8.9e-13 | 3.3e-13 | 8.9e-13 | 8.9e-13 | 1.9e-12 | 8.9e-13 |

Mode A is a consistency check only (§5); the stability evidence at the point is the kicks. 1e-6
kicks of w, r, p_m, p and s decay to about 5e-12. The measurement review's 1e-9 kicks, 14
directions at each of the 5 targets, all decay: peak D̂ ≤ 5e-6, no dead ticks.

**Mode B**, the registered battery ([results/battery.csv](results/battery.csv); fig1–fig3):

| tier | runs | CONVERGED | ticks to tol (median) | peak D̂ | dead ticks | worst fill | lowest good output | shortfall (coin) |
|---|---|---|---|---|---|---|---|---|
| 1 (±5%) | 16 | 16 | 268–449 (320) | 221 | 0 | 0.79 | 0.84 | 0 |
| 2 (±20%, b′ 0.36, 0.44) | 20 | 20 | 328–508 (441) | 857 | 12 | 0.45 | 0.54 | 4.4 |
| 3 (×/÷2, x\*/2, b′ 0.2, 0.8) | 21 | 21 | 407–677 (523) | 2,320 | 111 | 0.10 | 0.135 | 770 |

"Lowest good output" is cleared goods over the oracle's. Dead ticks all fall before tick 700,
none in the scored window. The nominal runs N(f) return the price level to the money stock's.

Families, variants and tick lengths ([results/families.csv](results/families.csv)):

| setup | CONVERGED | median / max ticks to tol | max dead ticks |
|---|---|---|---|
| Tier 3 at 10·L = 200,000 | 21/21 | 523 / 677 | 111 |
| stocks: machines, goods and coins displaced | 21/21 | 472 / 569 | 107 |
| joint ×/÷2 (60) and ×/÷4 (40) | 100/100 | 536 / 612 and 571 / 698 | 128 and 202 |
| basin: one price × 1.05^j, out to ×/÷8.15 | all, one connected range | — / 721 | — |
| ex-post assignment | 57/57 | 344 / 630 | 89 |
| ceiling payout | 57/57 | 445 / 677 | 165 |
| `Hold` | 57/57, identical to `Saturate` | 445 / 677 | 111 |
| tilt 1 (desks read their markup) | 57/57 | 297 / 422 | 88 |
| July's step rule (negative control) | 0/57, all DIVERGED | — | 337 |
| 12 a year | 57/57 | 276 / 725 ticks (23 / 60 yr) | 30 |
| 365 a year, L 20,000 and 104,000 | 57/57 | 2,618 / 3,698 ticks (7.2 / 10.1 yr) | 782 |

**The shock history**, bcycle(1500,80): b cycles through 0.44, 0.36, 0.8, 0.2 and 0.4, 80 changes
1,500 ticks apart, run for 121,500 ticks (fig5). Every segment re-enters tolerance before the
next change, the slowest in 783 ticks (b = 0.2). After every cycle the state is back at genesis
within 5.5e-9 in log, so there is no hysteresis. The classifier still scores it DEAD, because
each b = 0.8 segment has 125 dead ticks and each return to 0.4 has 116 (§5).

## 4. The dial map

The R3 judge's map, price rates × desk turnover. Each cell is the battery at its own L; the
entry is the slowest run's ticks to tolerance and the predicted PL. All nine are GO, and all nine
are stable to 1e-9 kicks at every target.

| rates \ turnover | ×1/2 | ×1 | ×2 |
|---|---|---|---|
| ×1/2 | 1,143 (0.992) | 1,061 (0.991) | 1,000 (0.989) |
| ×1 | 882 (0.987) | **677 (0.987), C2** | 521 (0.981) |
| ×2 | 1,064 (0.985) | 1,068 (0.987) | 649 (0.985) |

One dial at a time, ×1/16 … ×16, kick-corrected (fig6; fig7 predates the kick check):

| dial | GO | LOCAL | NO-GO |
|---|---|---|---|
| `rate.labour` (5.2/yr) | ×1 to ×8 | ×1/2, ×16 | ×1/4 and slower |
| `rate.good` (2.6/yr) | ×1/16 to ×2 | ×4 | ×8, ×16 |
| `rate.land`, `rate.mach` (1.3/yr) | ×1/16 to ×16 | — | — |
| all four rates | ×1/8 to ×2 | ×4, ×8, ×16 | ×1/16 (unstable at b′ 0.2, 0.36) |
| desk turnover (5.2/yr) | ×1/16 to ×8 | — | ×16 (unstable at every target) |
| `adjust.technique`, household spends | ×1/16 to ×16 | — | — |
| every dial together | ×1/8 to ×4 | — | ×8 |
| 12 a year, rates ×{1/2, 1, 2} | ×1/2, ×1 | — | ×2 |
| 365 a year, rates ×{1/2, 1, 2} | all three | — | — |

The `rate.labour` × `rate.mach` grid (fig8): with labour at ×1 to ×8 of 5.2 a year, every
machine rate from ×1/8 to ×8 converges. At 2.6 a year labour needs machines at ×1/2 or slower,
at 1.3 a year at ×1/8; slower still, nothing converges. So the wage must move at least as fast
as the goods price, and faster than the machine price once it is slow. Every LOCAL cell but
rate.good ×4 fails only at b′ = 0.2, the target the linearisation marks unstable there. C2 sits a
factor of 2 inside the LOCAL edge on `rate.labour`, and 4 inside it on `rate.good` and on all
rates together.

## 5. What the reviews found

Two reviews ran after the run step: measurement and economic fidelity. The measurement review
rebuilt `55c9e88`, reproduced every summary byte for byte, and re-scored all 57 runs every tick
from an independent scipy solve of M1–M6: no class or dead flag differs. The fidelity review
read the rules against the paper and the full end state of every run. **Both say the verdict
holds.** No blocker was raised. Their majors change the reading as follows.

- **Major (measurement): two GO cells were rounding freezes.** `Imbalance` stops moving a price
  once |k·x| < 2^-53. At a linearly unstable point a trajectory that reaches that floor rests
  there, and the classifier scores CONVERGED. At desk turnover ×16, one ulp on w grows to dead
  markets and freezes for good at tick 3,099. At all rates ×1/16, 1e-9 kicks at b′ = 0.2 grow to
  D̂ 1,104. Both cells were reported GO; both are NO-GO once kicked (§2, §4), and the prediction
  was right in both. The kicks also confirm 7 of the 8 LOCAL cells the prediction flags; the
  eighth, rates ×4 (PL 1.00025), needs about 55,000 ticks to grow a 1e-9 kick past tolerance,
  longer than the 20,000 run. **C2 is unaffected:** its 70 kicks decay.
- **Major (fidelity): the transients are large and at times perverse.** At ±5% goods output falls
  to 0.84 of target, and D̂ grows up to 4.3× before it decays. b′ = 0.44 lowers equilibrium output
  by 1.6% but cuts it by 27% on the way. b′ = 0.8 lowers it by 12% and cuts it by 85%, with p
  ×11. b′ = 0.2 raises it by 10% and cuts it by 43% at tick 1. A nominal shock with coin unchanged,
  N(0.95), cuts goods output by 4%. **So GO means the agents find the equilibrium, not that they
  adjust like the paper's economy.** History runs and the GUI will show these paths.
- **Major (fidelity): planned-x Leontief with keep-first self-use can stop consumption.** With no
  machines there is no output, though people can do every task, and the machine desk serves its
  own rescaled plan first. At each 0.8→0.2 change of the history the desk keeps all K, the good
  market has no supply, and nobody eats for a tick. Under tilt 1, three runs have a tick with no
  machines and no goods cleared, and are still CONVERGED. Ex-post assignment, the registered
  alternative, has shallower troughs (goods at b′ = 0.8: 0.36 against 0.15; 77 dead ticks, not 110).
- **Minors.** Mode A has no power as a stability test here: the negative control passes it. The
  sweep's labels are run-class tallies; no cell ran mode A or Tier 3 at 10·L. Transients leave
  the funded regime (716 ticks with a transfer shortfall), while participation still reads w/P_s
  as if the basket came. Profits pile up in the desks while the provider starves. C2 was chosen
  on an exact mirror after checking the battery's targets, so the Rust battery is a replication
  and the sweep is the out-of-sample evidence. The smoke battery ran at 18:38, before the
  registration at 19:02, against §4.10's letter; nothing changed after it, and the counted summary
  is byte-identical. Label fixes: RULES §7's ex-post mode-A figure is 8.9e-13, not 1.9e-12; the
  peak D̂ of 2,320 under b′ = 0.8 is the goods price ×10.1; tick-length invariance of x\* holds by
  construction and is not evidence.
- **From the judges.** The scale rule never reads its margin: zero profit arrives by accounting,
  as survival, not as a firm choosing at the margin. Tilt 1, where desks do read their markup,
  also converges 57/57 and faster (median 297), so convergence does not depend on that choice.

**The corrected verdict.** GO for C2 as built, with CONVERGED scored only where a 1e-9 kick
decays. The GO region is narrower than first reported: turnover up to ×8, all rates from ×1/8.
The verdict is about the end state; the paths carry the costs above.

## 6. What this means for the plan

- **The kill condition (A11).** Not triggered. A11 fires on a failing mode B; here mode B passes
  on the first registered design, from ×2 and beyond, where July's rules returned only within
  about ±5%. The fallback, the oracle as the per-period core, is not needed for Appendix B, and
  Phase 2 proper carries A11 forward for its other instances. PLAN §3.2 should change too:
  July's step package, run here as the negative control, diverges 57/57 while the cash rule
  converges. The cash rule should be §3.2's starting point, and "dead" drops out as a map axis.
  A9's re-run of July's solvable family becomes optional: its failure is reproduced here on a
  harder instance.
- **Phase 0 session 2 (certify).** Proceed as planned, with criteria the probe shows are needed.
  None needs an engine change.
  1. *A kick check.* A run counts as converged only if a 1e-9 kick in ± each price at its end
     state decays. Otherwise a rounding freeze at an unstable point certifies, and a hold test
     proves nothing, since an exact genesis never excites the loop.
  2. *Transient statistics as outputs:* troughs per market, dead ticks, ticks with no
     consumption, the transfer shortfall (R12) and spoilage. The end state alone hides §5's paths.
  3. *Windows per shock.* N15's relative windows should restart at each dated shock, so a
     history whose every segment returns is not scored DEAD as a whole.

  The probe's relative runaway bound, [1e-6, 1e6] of genesis, is the detector A12 asks for; it
  caught every run of the negative control. `crates/probe` prefigures D13's `crates/observe`:
  session 2 should say which of its measures move to certify and which wait.
- **G0 (the GUI).** GUI.md's architecture stands: agents drive and the oracle is a readout, which
  the fallback would have changed. Add `tapes/appb.ron` as G0's second world: it has a known
  answer, and its transients are what the viewer must show well. Revisit D2's "no log axes" in
  G0.1: every probe plot needed log D̂ over twelve decades, and a market with no trade (D̂
  infinite) needs a mark of its own.
- **Phase 2 proper.** It starts from these roles, not July's. Its other instances (the wall, the
  open commons, 1750-like) need oracle units 1b–1e first, which keeps the units on the critical
  path. Its battery inherits the harness, the kick check and the families.

**Recommendation.** Close the probe as GO and do not trigger the fallback. Keep the order of the
ruling of 2026-09-26: Breakpoint B's eyeball test in parallel, then session 2 with the three
criteria above, then G0 with the Appendix B world. Record the scale-rule change to PLAN §3.2 and
the demotion of A9 as proposed amendments for the user's ruling. Build units 1b–1e alongside, and
open Phase 2 proper with many markets, the untested risk the units make testable first.

## 7. Open questions

1. **Durability and interest.** Unit 1a solves ρ > 0, δ < 1 and J_b > 1; the agents ran only
   the flow benchmark. Durable machines need investment (Phase 3) and interest needs credit
   (Phase 8). Whether agents find the user cost's margin is untested.
2. **Many markets.** Does the rate condition (the wage no slower than the goods price) carry to
   many categories and machine types (units 1b–1c), or does each new market need its own ordering?
3. **Transient fidelity.** Is a tick with no goods acceptable in a history run? Ex-post
   assignment converges 57/57, faster (median 344), with shallower troughs; adopting it
   takes its own registration and battery. Profits paid to owners did not help (ceiling variant).
4. **The price-shock history.** 80 sequential joint price displacements need a tape action that
   sets a posted price, or a resume from an edited state. Neither exists (E1).
5. **Tick length.** At 12 a year the region is narrower (rates ×2 fail) and adjustment takes 23
   years at the median. Which tick length do the historical runs use? `RawLife::OneTick` (schema
   2) would let the tick-length check run without restating `life.one_tick`.
6. **Left unbuilt:** the wedge solver (PROBE-SPEC §3.3 a), needed once a design has bands.
