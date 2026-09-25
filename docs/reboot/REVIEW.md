# Review — rustyecon at the reboot

Dated 2026-09-25. Scope: the whole repository at `f614792` (engine, tooling, scenario
corpus, the July v2 design), read against laborformal at `31b3482`. It is the input to
[PLAN.md](../PLAN.md). Verdicts: **KEEP** (carries forward as is), **ADAPT** (the idea
carries, the form changes), **ARCHIVE** (leaves HEAD, kept at a tag), **RETIRE** (does
not return).

## Rulings after this review (2026-09-25)

The review is kept as written. These rulings, made after reading it, override it where
they conflict:

1. **Rust stays**, for the performance headroom. §6 item 6 is overruled.
2. **Agents stay**, with a granular set of regions. The desk kernel moves from RETIRE to
   ADAPT: agents now make the pinning paper's decisions. Region sharding and the
   parallel map with ordered reduce move from RETIRE to KEEP. July commitments 1, 2 and
   7 (§5) stand. §3.2 (b) and (c) remain as risks; the plan answers them with an
   equilibrium oracle that the agents must converge to. §3.2 (d) is overruled: the
   regions are now justified by geography (enclosure by county, coalfields, towns), not
   by a game's scale.
3. **The span is 1750–2050**: history scored 1750–2025, forward branches 2025–2050.
4. **The v1 verdict (§2.5) narrows.** Salvage the substrate (ids, inventories, the
   delta pattern, clearing, price update) with its defects fixed; archive the v1 agents,
   tooling, suites and corpus.

## The verdict

rustyecon has a sound engineering discipline attached to two things that no longer
hold. The v1 engine is a posted-price agent model that failed its own stability suite;
no code has changed since the July ruling, so all seven of the July Phase 0 defects are
still open. The July v2 design repairs the engineering on paper, but its economics are
the two-economies framework — overflow beyond an absorption cap, a minting queue, the
perimeter ledger — which laborformal retired on 2026-09-04, and it places an untested
agent kernel in front of every economic result.

laborformal now has what rustyecon lacked: a closed, checked price system for wages,
rents and machines (*Pinning the Wage to Scarcity and Technology*, SSRN 2026-09-23); a
capital-dynamics engine with build lags; a dynamic macro layer validated on 1929–33 and
2008–10; a seven-century fitting spec with identification by era; and working methods
that produced checked results every few days. The reboot should make rustyecon the
running-history engine of that economy: the pinning price system solved every year from
1725 to 2025, moved by slow states and a scripted tape, and scored against the long
record. Archive the v1 code, tooling and scenario corpus at a tag. About a third of the
July design (the tape, the registry, certification, the era ladder, the standing rules
worth keeping) carries forward.

## 1. Inventory

| Component | Size | State |
|---|---|---|
| v1 engine (`src/`, `tests/`) | ~2,950 lines of Rust + 280 of tests | Posted-price markets, pro-rata clearing, three building strategies, employed/unemployed pop pairs, wealth-tier baskets, lots and spoilage, RON scenarios, CSV telemetry, checkpoints. Last changed 2026-07-18. |
| Python tooling (`tools/`) | ~2,060 lines | Streamlit app (1,210), RON parser, results readers, scenario helpers |
| Suites (`notebooks/`) | 757 lines, plus a 3.8 MB notebook committed with outputs | Supply-chain napkin sim; the 24-cell labour factorial generator (06); the stability analyser (07) |
| Scenario corpus (`data/scenarios/`) | 27 scenarios, ~50k lines | 3 hand-built, 24 generated `lr_*` cells |
| July v2 design | ~1,200 lines | ARCHITECTURE, 8 subsystem specs, METHODOLOGY (R1–R14), PLAN (Phases 0–9). No code was written against it. |
| v1 docs, triaged | ~3,050 lines | DESIGN, glossary, five system docs, each with inline KEEP/REWRITE/CUT verdicts |
| Era research (`docs/timeline/eras.md`) | 228 lines | Goods, recipes, technologies and trade by era, 1836–2036 |

About 4,500 lines of design documents describe about 3,000 lines of code. The README's
own estimate is that the docs are ~90% aspiration.

**Environment.** The Windows machine this review ran on has no Rust toolchain (no
`cargo`, no `~/.cargo`, no `target/`) and no rustyecon venv. The repository was last
built on a Linux machine (`/home/wilso/github/rustyecon`, per
`.claude/settings.local.json`). laborformal's Python venv (numpy, pandas, scipy, sympy,
matplotlib, pyarrow) is present and working. Every statement below about v1 comes from
reading the code. None of it was run for this review.

## 2. The v1 engine

### 2.1 What is good

- The delta discipline: systems return `Vec<StateDelta>`, a single pass applies them,
  and the enum is `#[non_exhaustive]` with an exhaustive in-crate match. Clean and
  testable.
- Sparse sorted inventories with FIFO lots; name-resolved RON scenarios; one `SimState`
  type for genesis, checkpoints and saves.
- Small, readable code, and a candid triage of its own docs.

### 2.2 Defects

The July plan's Phase 0 list is still open, item for item:

1. [`apply.rs:88`](../../src/state/apply.rs#L88): the `return` in `RedistributePopPair`
   drops the rest of the batch.
2. [`raw.rs:359`](../../src/scenario/raw.rs#L359): the loader hardwires `CapacityControl`
   whatever strategy the recipe declares.
3. [`transactions/mod.rs:180`](../../src/systems/transactions/mod.rs#L180): `HashSet`
   iteration makes the delta order nondeterministic.
4. [`systems/mod.rs:51`](../../src/systems/mod.rs#L51): spoilage mutates state
   directly.
5. [`inventory.rs`](../../src/types/inventory.rs#L101): lot lives are not serialized, so
   checkpoints turn perishables into indefinite goods.
6. [`pop_agent.rs:103`](../../src/systems/decisions/pop_agent.rs#L103) and
   [`:150`](../../src/systems/decisions/pop_agent.rs#L150): per-pop printing in the hot
   loop (line 103 has no newline).
7. The 07 suite's `is np.nan` guards fail open.

This review found five more. Each is latent in today's runs; each is the fail-open kind
that R3 was written to forbid:

8. **Settlement is asymmetric.** A seller is paid for its full filled quantity and gives
   it up ([`transactions/mod.rs:116-147`](../../src/systems/transactions/mod.rs#L116)).
   A pop buyer gets, and pays for, only what its remaining cash covers
   ([`:71-84`](../../src/systems/transactions/mod.rs#L71)). Clearing matches seller
   fills to buyer fills, so any cash-capped pop order would destroy goods and mint
   currency silently. It is dormant only because the pop agent's orders never exceed
   its cash today. Any order logic that could overspend would switch it on, including
   the July design's logit substitution.
9. **Shortfalls are invisible.** `Inventory::remove` returns the quantity actually
   removed, and `apply_state_deltas` discards it
   ([`apply.rs:21-23`](../../src/state/apply.rs#L21)). An overdraw of goods or currency
   would be clamped to zero and never recorded.
10. **Id collision.** Building and recipe-instance currency flows share one `(u32, u32)`
    key space ([`transactions/mod.rs:85-101`](../../src/systems/transactions/mod.rs#L85)),
    and all of them settle as recipe instances
    ([`:182-185`](../../src/systems/transactions/mod.rs#L182)). Buildings post no orders
    today. If one did, the recipe instance with the same number would be paid.
11. **Dead and overloaded state.** `sub_state` starts at the target fractions and only
    ever moves toward them, so it never changes. `DividendPayout` reuses `chosen_size`
    as a currency amount ([`building_agent.rs:205-216`](../../src/systems/decisions/building_agent.rs#L205)).
12. **Magic producers break conservation by design.** Goods come from nothing and the
    buyers' payment reaches no one
    ([`transactions/mod.rs:45,139`](../../src/systems/transactions/mod.rs#L45)). They
    appear only in `supply_chain`, not in the lr corpus.

### 2.3 Constants

The agent layer has about 34 unregistered behavioural literals. There are 24 in
`building_agent.rs`: the 0.9 recovery threshold, the 2% balance discount, the 1.2×
sell cap, the 1.1/2.2 market-share knees, the 0.2/0.8 throughput blend, the 5% margin
gate, KP 0.6 and KD 0.2, the 0.1/0.9 anchor, and a reserve multiple defaulting to 26.
Seven are named constants in `pop_agent.rs`. The EMA spans in `price_update` and the
magic producers' 1.05 add the rest. The July rules R1 and R2 would delete almost all of
them.

### 2.4 Tests and tooling

The two integration tests are smoke tests. test_01's header promises that "price should
stabilise immediately and remain constant", but its assertions check only a startup
spike, finiteness and positivity. There is no conservation, determinism or resume
test. The stability suite writes a RON checkpoint every tick (1,000 files per scenario,
25 scenarios) and parses them in Python. The Streamlit app is the largest file in the
repository: tooling built around a core that never stabilised.

### 2.5 Verdict

**ARCHIVE** the engine, tests, tools, suites and corpus. The next engine shares no code
with this one (see the plan). Migrating v1 would cost effort and transfer nothing, and
fixing its defects first would repair a model the reboot does not run. What carries
forward is the lessons: canonical ordering, apply-once mutation, fail-closed metrics,
and the failure classes the stability suite named.

## 3. The July v2 design

### 3.1 What it got right, and what carries forward

- **The tape.** History as one dated event stream; a scenario as (tape, code, seed);
  alternate histories and policy experiments as tapes that differ. KEEP.
- **The scripted/emergent registry** (METHODOLOGY §4) and R6, targets are never inputs.
  KEEP.
- **Certification.** Verdict-first, fail-closed and persisted; criteria dated before the
  results; NaN fails, with a registered declared-missing convention. KEEP.
- **The era ladder.** Certified legs, failures that are local and dated, and a permanent
  full-span tracer. KEEP.
- **The worldgen compiler.** Small human-editable tables compiled into a validated tape.
  ADAPT (smaller).
- **R2** (every constant registered), **R4** (determinism tested), **R7** (one engine),
  **R10** (receipts for complexity and simplicity), **R12** (costs are goods). KEEP,
  reworded.
- **The stated kill condition.** KEEP the habit.

### 3.2 What fails now

**(a) Its economics belong to a retired theory.** METHODOLOGY §1 and §5 anchor the design
in "the sister two-economies project". That means overflow beyond an absorption cap
funding a minting queue, claims that are minted but never traded, a perimeter ledger with
measured and true aggregates, and the readouts P, Q, B, μ, r\* and dispersal. The
framework was the long draft *The Link*, retired from laborformal on 2026-09-04 (tag
`pre-cleanup-2026-09-04`). None of overflow, absorption, minting or perimeter appears in
laborformal now.

The current theory is the pinning paper. Its objects are:

- the task schedule γ(x) and the assignment margin x\*;
- the machine-cost recursion c = ac + λw + br;
- non-produced inputs and their rents;
- the priced outside option s(q) and its dependency floor;
- category prices p_j = wL_j\* + rb_j;
- the income identity I = wN_a + rT.

None of them appears anywhere in rustyecon. July Phases 6–7 (the perimeter ledger; land,
minting and growth) would build machinery for a theory that no longer exists. Several
ideas survive in translation (§3.3). The machinery does not.

**(b) The biggest risk sits in front of everything.** v1 failed stability. The v2 answer
is a desk kernel whose stability rests on the literature (Mark-0, Lengnick) but has never
been tested; its phase diagram was due in Phase 5, with a kill condition if it failed.
Every economic result was scheduled behind five phases of plumbing and stabilisation. The
plan's own estimate was four to six months part-time to Phase 8.

**(c) The theory's objects are equilibrium prices, and the kernel would have to find
them.** A posted-price agent model reaches w = cγ(x\*) and c = ac + λw + br only if it
converges, and a run that fails to converge is indistinguishable from an economic
result. The programme now asks where the wage sits between replacement cost and exit
value, how rents enter both, and how that moves over three centuries. Solving the
equilibrium directly is cheaper and more honest for those questions. The disequilibrium
that matters (unemployment, crises) is better carried by an explicit short-run layer, as
laborformal's `paths/` does.

**(d) The grain is a game's, not the data's.** 673 regions at weekly ticks is a Victoria
3 target. The long record is annual at best and decadal for most series before 1800. The
research questions need one well-measured economy first and a handful of macro-regions
later.

**(e) The capability ladder stands in for the schedule.** Four frozen labour buckets make
the wage distribution an output. But the theory's object is a continuous
relative-productivity schedule per worker type, with a wall of closed tasks and a margin
that moves. The ladder cannot separate task automation (γ falls) from recursive
automation (λ falls), and the programme turns on that distinction.

**(f) Data were deferred to Phase 8.** laborformal's credibility comes from sourcing and
validating data early: the fork, κ, the λ series, the long-record source probe. A
300-year engine without its data spine until the end has nothing to be wrong against.

**(g) Money and crises were deferred to Phase 9 and later.** Three centuries include
1797, 1825, 1847, 1866, 1873–96, 1914, 1929–33, the 1970s and 2008. laborformal's
`paths/` has since built a reduced-form crisis layer validated on 1929–33 and 2008–10
with one parameter set.

### 3.3 July concepts, translated

| July v2 concept | Counterpart in the pinning economy | Verdict |
|---|---|---|
| Desk kernel (Rules 1–3, stagger, dead-band) | Equilibrium conditions (cost-minimising assignment, free entry, clearing, participation) plus named, swappable dynamic behaviours | RETIRE (parked; it may return only as a micro layer that nests the core) |
| Overflow → minting queue | Investment at the user cost with build lags (`dynamics/`); owners' saving as a named behaviour (`paths/`) | RETIRE, replaced |
| μ and r\* as queue readouts | r\* as the neutral-rate bridge; ρ as the required return in the user cost | RETIRE, replaced |
| Claims minted, never traded | Ownership shares of land and capital; land price as capitalised rent (`paths/code/bank.py`); no secondary market as a behaviour | ADAPT |
| Parcels, ω, enclosure | Non-produced inputs with quality schedules; the idle margin and enclosure (Proposition 3): zero rent while suitable land lies idle | ADAPT (now central) |
| Perimeter ledger; measured vs true GDP; P | The exit life s₀, household production and participation, kept as readouts rather than a ledger | ADAPT (smaller) |
| Capability ladder (4 buckets) | Task schedules per worker type; the wall H | RETIRE, replaced |
| Wealth-tier baskets, absorption cap | Category demand with a subsistence basket (Allen's); owners' and workers' demand | RETIRE, replaced |
| Rationing is the point (R8) | Prices clear in the core. Rationing survives as unemployment and credit rationing in the short-run layer, and as deprivation below the welfare-ratio floor | ADAPT |
| Conservation ledger with provenance (R3) | Accounting identities every period (clearing, zero profit, the income identity); stock-flow consistency in the money layer | ADAPT |
| Credit through a double-entry postings seam | The same idea for the money layer, with content from `paths/` | ADAPT (Phase 6) |
| Region sharding; parallel map, ordered reduce | Parallelism across ensemble draws and fits | RETIRE |
| Parquet telemetry, tidy long format | Same | KEEP |
| Tape, registry, certificates, era ladder, compiler | Same | KEEP / ADAPT |

## 4. What changed in laborformal

laborformal's whole git history (65 commits, 2026-08-09 to 2026-09-25) postdates the
July plan. This is what each thread now offers rustyecon.

| Thread | What exists now | What rustyecon can take |
|---|---|---|
| `pinning/` (P1) | The paper (SSRN 7226858, 2026-09-23): the task margin; the replacement closure c = br/(1−a−λγ(x\*)); priced exit s(q) = max(s₀ − q·h_e, s̲); the fork at an interior margin with category bounds; the income identity; the rent-tax and uniform-transfer pair; history as four configurations; Appendix A's general system c = (I−A)⁻¹(Λw + Br) with durability and interest; Appendix B's solved interior equilibrium; 51 checks plus 11 corner checks; a Lean formalization. Measurements: the fork, 4.8× over 1964–2024; κ = 0.33 [0.18–0.59] in 2025, about 0.05 in the 1950s; labour-origin financing of consumption at 69.6% against 48.9% human effort in production (2004); 0.68 of tax revenue from labour bases. | The per-period core, its gate numbers, and the modern moments to score against. |
| `progress_and_prosperity/` | The book programme (P1–P5). The λ gate passed on 2026-08-20: the labour content of machine production falls on every referee, by 0.48 points per decade at the median. | λ becomes a measured, falling series to score against. The figure spine's tier labels (accounting / calibrated / argued) gain a fourth: simulated. |
| `dynamics/` | Capital as time: the user cost u_K = (ρ+δ)(1+ρ)^(J−1). A durability line prices every produced object by its build lag J and depreciation δ; consumables sit at J = 0, δ = 1 and land at J = ∞. A transition engine with a validation ladder and 54 checks. Verified experiments: the windfall (land takes about 98% of the released value), the waterfall, speed × lag. The sloped wage path's sign depends on the shock: frontier extension or efficiency deepening. A steady-state equivalence lemma ties the dynamics to the static paper. | Capital vintages and build lags; the nesting discipline (every dynamic layer reduces exactly to the static paper). |
| `paths/` | The macro block: Appendix B along a technology clock, capital at user cost, calibrated to four public targets. Around it: a demand gap with a central bank that learns; households (state vs family support, floating-rate debt, rigid money wages); a stylised bank; sovereign spreads. A four-region crash model (US, euro area, Sweden, China), validated on US 1929–33 and 2008–10 with one parameter set, with two automation waves, care as the absorber (measured ceiling, gate and pace) and financing limits. The data since 2024 bound the pace of displacement at 0.15 of the paper's at most. Design principle: structure fixed, behaviour swappable. | The short-run layer, the forward branch, and the design principle itself. |
| `long-record/` | *One Schedule, Seven Centuries*: a spec to fit the model's configurations to the English record. It adds three slow states: population (φ), the idle margin, and a piecewise technology path capped at six knots. Three discriminators: D1, q's determinant flips at the switch; D2, the switch is dated twice (the wage's escape and land's exit) and the dates must agree; D3, floor-era welfare variance is explained by N and T. Identification runs era by era. Sources probed: the Bank of England millennium set is reachable, Clark and BNS via the fetcher, Allen blocked. Parked at Breakpoint A since 2026-08-17. | The scoring spine for 1725–2025 and the fitting discipline. The engine this spec needs is what rustyecon should become. |
| `three-taxes/` | The fiscal architecture. Every unit of spending resolves into wage claims and scarcity claims, with wage share φ_w = λρ\*/(1−a). There are three coherent places to tax: the rent source, the consumption gate, and a targeted externality. The gate tax retires into the rent tax as automation proceeds. 35 checks. | The founding VAT/UBI question, now with a theory: the policy lab. |
| `capability/`, `companion/` | The empirical schedule: the w/c waterline up ×8 over 1999–2025; the revealed-adoption envelope (11–33% of 1999 employment flipped by 2025, depending on the rule); LLM exposure; the education-premium race in the CPS MORG. | Worker-type structure (entrant vs trained) and modern moments for the task margin. |
| Methods | Checks gate absolutely (sympy plus numeric). STATE.md is the resume point, and numbered decisions form a veto window. Primary sources; stop and report, never substitute. Bands over specification grids; claim-status tags; public data only. | The process and the standing rules. |

## 5. The seven July commitments, re-judged

| July commitment | Verdict |
|---|---|
| 1. Tick-based, posted price | RETIRE. Annual periods, each an equilibrium; stickiness lives in the short-run layer. |
| 2. Highly parallelizable | ADAPT. Parallel across ensemble draws and fits, not within a period. |
| 3. A historic world built by placing regions and initial conditions | KEEP. |
| 4. Technology, war and policy as scripted events | KEEP. |
| 5. Running 1800–2025 | ADAPT. 1725–2025, the 300 years, with forward branches. |
| 6. Monetary systems and trade; almost everything is a good | ADAPT. The goods are categories, machine services, non-produced services and labour types; money becomes a later layer. |
| 7. Radically simple agents, one kernel | RETIRE. Price-taking equilibrium plus named, swappable behaviours. |

## 6. What the plan takes from this

1. Archive v1 wholesale at a tag. Do not fix its defects.
2. Build the pinning economy as the per-period core, gated on the paper's published
   numbers.
3. Bring in time through slow states and the tape, gated on `dynamics/`' closed forms.
4. Start with England alone. Build the data spine early, and register the fitted and
   scored moments in advance.
5. Add the short-run layer, the regions and the forward branch after the first
   certified 300-year run.
6. Use Python, in laborformal's style, unless Rust is worth its cost to you (the
   plan's decision 2).
