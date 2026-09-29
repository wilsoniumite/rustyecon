# LOOP-SPEC-A2: what P2.2b's build departs from, registered before its code

Dated 2026-09-30. Label `rules`, step P2.2b.0 on branch `phase2-plants` (worktree
`D:/rustyecon-wt/p2b`, from `reboot` at `8b07c8a`). It amends `LOOP-SPEC-A1.md` (sha256
`8815d8652f6356423d02e06e2e5be814397f86e923688a78e1ef4d55afda4f2b`), which amends `LOOP-SPEC.md`
(sha256 `e16e0be2ab521ed28c5d79811fabbb250ca5bbb107bd00f656d45d0993fb89ad`). Both stay as
registered (decision 282). This file's sha256 is in `SHA256SUMS` beside it.

It is written before any P2.2b engine code or run exists. At this commit `crates/core`,
`crates/markets`, `crates/agents`, `crates/probe` and `crates/engine` hold no plant code, and
`tapes/` holds no rule-B tape. The build's spec is `docs/probe/LOOPS-RULES.md`, committed with
this file.

## A2.1 LN5 and LF4 wait for storable running goods (STATE O51)

**What A1 registers.**
- E9 gives LN5, storable fodder at a 4-week cover, an engine outcome: it orbits in every tier and
  its verdict is NO-GO.
- E10 gives LF4, the 13-week cover, a GO "if built", and says it needs STATE O51's roles.

**Why the build cannot run LN5.** Both instances need the same two roles (LOOP-SPEC §2.6):
- a fodder seller that offers I/(1 + b) of its stock and keeps what does not sell;
- a capacity desk and a maker that order their fodder net of what they hold.

Neither exists. M5 refuses a role that offers a storable good in full, and M6 refuses a role that
buys one without netting it (decisions 240, 241). P2.2b's build list (STATE next step 6, O65) does
not include them. LOOP-SPEC §2.6 already says LF4 needs them; E9 did not say so of LN5.

**The amendment.**
- LN5 is not run in P2.2b, as LF4 is not.
- E9's items on LN5 are withdrawn from P2.2b's scoring: "LN5 orbits in every tier", LN5 in the
  NO-GO list, and LN5's row under the rule "each tier's converged count within 2 runs".
- Both instances stay registered, with A1's numbers, for the build that adds O51's roles.
- Nothing else in E0–E11 moves.

## A2.2 What was checked and does not depart

The build's spec (LOOPS-RULES) was read against LOOP-SPEC §2–§5, A1 §A1.1 and §A1.5, and the
mirror's code (`lm_mirror.py`, `lm_carry.py`, `lm_run.py`, `lm_inst.py`, sha256s in the
registration). These are build choices, not departures:

- **The plant good has no market.** LOOP-SPEC §2.2 says so. Core today refuses a good that is
  not a currency and has no price rate (TAPE.md, "Currencies"). So the build adds an untraded
  good to core: `Indefinite`, no price rate, no market, no genesis price, and no order may name
  it. That is §2.2 as registered. The other way, a plant market that no one trades, would break
  E4: certify's kick set kicks every market, and a kicked plant price never moves back.
- **One buy line per good**, coef·(z + s·I), with the bundles received split as B·z/(z + s·I)
  and B·I/(z + s·I). That is §2.2's text. The mirror keeps the two lines apart and takes a
  Leontief over each. With one fill ratio per good the two agree to rounding.
- **Budgets, and the burn that cannot fail.** The mirror has no budgets. The engine's budgets
  and burns are sized so that they bind only at rounding (LOOPS-RULES §3.5–§3.6).
- **The E1 fixed plant is a plant at δ_p 0**, started from the design's plants. With no wear
  there is no order and no user cost, so it is fixed-Q drs (the mirror's plant rule "none").

Each of these parts from the mirror by rounding at most. E0's tolerance, 1e-12 in log, covers
them. A parting at tick 1 still names the carry, and would be amended before any scored run.
