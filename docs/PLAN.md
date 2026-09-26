# rustyecon v2 — the reboot plan

Amended on 2026-09-25 by the rulings in [ADDENDUM.md](reboot/ADDENDUM.md), folded in
where they change the text: A1, A5 and A8 as ruled (A5's ruling replaces ruling 7), and
the rest of A2–A13 as written. A14, an interactive GUI early, was ruled the same day
(ADDENDUM rulings 5–8) and folded in on 2026-09-26: R16 (§4), the GUI's stages beside the
phases (§6) and §9. Its design is [GUI.md](GUI.md). The Phase 2 probe's scale rule
(STATE.md decision 39) was folded into §3.2 on 2026-09-26. REVIEW.md and ADDENDUM.md cite
this plan's lines as they stood at `87d95d7`, in `docs/reboot/`.

Dated 2026-09-25, revised the same day after your rulings. On your go, this supersedes
the July v2 (`docs/ARCHITECTURE.md`, `docs/METHODOLOGY.md`, `docs/PLAN.md`,
`docs/architecture/`). The reasons are in [REVIEW.md](reboot/REVIEW.md), read together
with its rulings block.

**Fixed:** rustyecon runs 300 years of economic history, 1750–2050. History is scored
from 1750 to 2025; from 2025 to 2050 the engine runs forward branches.

## Rulings (2026-09-25)

1. **Span: 1750–2050.**
2. **Language: Rust**, for the performance headroom.
3. **Engine: agents.**
4. **Regions: granular.**
5. **The long-record work:** rustyecon is the engine and holds the data spine. The
   paper (*One Schedule, Seven Centuries*) stays a laborformal thread that cites
   rustyecon's runs by commit. The long-record thread's Breakpoint A questions still
   need answers before Phase 5.
6. **Public data only, and a public repository**, as in laborformal.
7. **The toolchain: WSL primary, and Windows too.** WSL Ubuntu on this machine is the
   Linux machine, and the gates run there. Windows is a secondary build and test check.
   Cross-platform hash equality is recorded, not gated. (ADDENDUM ruling 2; the first
   ruling 7's premise, that Windows had no Rust toolchain, was false.)

## 1. The idea

Agents in a granular world, where every decision an agent makes is one of the pinning
paper's margins:

- a producer chooses, task by task, between people and machines at posted prices;
- a machine maker sells machine-hours at whatever its own inputs cost it;
- land, sites and deposits rent for what their users will pay;
- a household chooses between work and a life outside it, and that outside life itself
  needs land and a site to live on.

Nothing is solved centrally. Agents read posted prices and their own state, and markets
clear by rationing. The paper's equilibrium then serves as the check: when history
stands still (fixed technology, fixed population), the agent economy must settle where
the paper's equilibrium is. A small equilibrium solver in Rust, the **oracle**,
computes that point independently of the agents. This answers the review's two open
risks. Whether the agents find the theory's prices becomes a test with a known answer,
and a run that fails to converge shows up as a measured gap from the oracle, not a
mystery.

The machine-cost recursion reads the whole span with one equation. In 1750 the dominant
machine is the horse, whose recipe is mostly fodder (land) and grooms' labour: b and λ
are both large, so a machine-hour is a claim on farmland rent and wages. Coal and steam
move the recursion's terminal input from fields to deposits: Wrigley's shift from an
organic to a mineral economy, stated structurally. Electricity, computers and
datacenters change a and λ, and change which inputs are terminal: sites, power,
fabrication capacity. The same c = (I−A)⁻¹(Λw + Br) prices a plough team in 1750 and a
GPU-hour in 2050.

Granularity pays for itself because the mechanisms are geographic. Parliamentary
enclosure ran county by county. Coal sat in particular fields. Cities are where site
rent concentrates, and the canals and railways that moved goods between them are
channels on the tape.

## 2. What rustyecon is for, in order

1. **Generate the long record** for England and the UK, 1750–2025, from one set of
   parameters: the welfare ratio, rents, land's share, labour's share, participation,
   the category-price fork, λ and κ. The long-record thread's discriminators D1–D3 are
   pre-registered tests. Engels' pause (roughly 1780–1840: output rising, real wages
   flat; Allen 2009) is the flagship target. The switch between configurations is
   dated twice.
2. **Run the forward branches, 2025–2050**, from a certified 2025 state: the AI
   configuration, with `paths/`' clocks, two waves, care absorber and financing rules
   written as tape generators. Those branches then continue a world that has lived
   through its history instead of starting from a calibration.
3. **Run the policy lab.** The founding VAT/UBI question, now three-taxes' question: a
   rent tax with a uniform transfer, a consumption gate tax, payroll and income taxes,
   Poor Law regimes. Each experiment is a tape that differs in its law entries.
   Historical counterfactuals are included: an 1880 land-value tax, Speenhamland kept,
   no parliamentary enclosure.
4. **Explain.** The book's figure spine gains a fourth tier label: simulated.

## 3. Architecture

```
tape (dated history) ─► phase 0: events
                         phase 1: decisions   (agents read posted prices + own state)
                         phase 2: clearing    (pro-rata, per node and good)
                         phase 3: settlement  (conserved, tagged, rationing recorded)
                         phase 4: production  (task assignment → output; machine services)
                         phase 5: upkeep      (vintages, depreciation, population, enclosure)
                         phase 6: price update
                         phase 7: certify & measure ──► Parquet, certificate
                                                        oracle gap (diagnostic only)
```

### 3.1 The ontology

- **Goods:** categories (food, clothing and manufactures, fuel and light, shelter
  services, services and care, finer goods where a chain needs them); machine services
  by machine type; labour by worker type; services of non-produced inputs (land by
  quality class, urban sites, extraction rights on deposits); currency. Almost
  everything is a good, as before.
- **Recipes:** a category's recipe is a list of task cells. Each cell carries human
  productivity by worker type and machine productivity by machine type (infinite machine
  cost where the task is closed: the wall H). It also carries a direct requirement b_j
  for non-produced services: acreage for food, sites for shelter. A machine type's
  recipe is (A, Λ, B) per unit of service. Recipes have versions that the tape
  publishes.
- **Desks:** one per recipe version per region (the July uniqueness rule), carrying
  size, scale and an inventory. Producer desks make categories; machine-service desks
  make machine-hours; transport desks operate channels; government desks follow the
  law on the tape.
- **Capital:** a desk's size is capital. It is built from goods (the build bundle),
  delivered after J ticks, and depreciates at δ. Vintages are tracked, so sunk capital
  competes at its operating cost.
- **Parcels:** non-produced inputs, each in a region with a quality, an endowment of
  service per tick and an enclosure status. They are owned by owner pops and sell their
  service on the regional market. An idle parcel's reservation rent is zero. Parcel
  quality and the idle margin come from the paper's 2026-09-21 revision (`main.tex`),
  together with the default exit form (§3.2).
- **Pops:** region × worker type (entrant, trained) × class (workers, owners). Each
  worker pop is an employed/unemployed pair (v1's pair survives) with a shared
  inventory. Owners hold claims on parcels and desks.
- **Nodes and channels:** regional market nodes; channels between them operated by
  transport desks, whose recipe carries the crossing cost. Tiers are topology.

### 3.2 Agent rules: the paper's margins as decisions

- **Scale** (every desk). The default is the cash rule of the Phase 2 probe
  ([REPORT](probe/REPORT.md), [RULES](probe/RULES.md) §2). Each tick a desk spends a
  registered share of its coin on inputs, share(v·μ^κ) of it, where v is its turnover,
  μ its markup at posted prices and κ a registered tilt, and plans its output at its
  unit cost. At tilt 0, the probe's setting, it reads no margin. Its coin is constant
  only where revenue equals outlay, so its only rest point is zero profit, the oracle's.
  On the Appendix B flow instance it returned to the oracle from every registered start,
  from each price ×2 or ÷2 and from cost shocks, within 1e-14 in log; its transients
  are violent. It carries no
  clamps, caps or forgiveness (R3). The named alternative (R6) is the July stability
  package: small asymmetric multiplicative steps, a dead-band, output and cash buffers,
  and staggered activation. On July's rules it held a solved fixed point exactly, and
  displaced beyond about 5% it did not return but for one configuration (ADDENDUM §3).
  Run in the probe's engine as the negative control, it diverged in 57 of 57 runs. A
  re-test of July's price-responsive posting (`47381ed`; basin about ±5%) takes the
  changes July named, a rent-aware reservation price and a rule for one-sided markets,
  and two more: the user cost in the reservation price, and stagger on the scale
  decision only, with non-storables bought every tick. Until Phase 3, desk capacity is
  fixed at the oracle's free-entry stock, or the oracle takes installed capacity as
  state (tentative).
- **Technique: the task margin** (producer desks). For each task cell, the share done
  by labour type k or machine type m moves toward the input that is cheaper at posted
  prices, w_k/γ_Lk against c_m/γ_Mm. Installed machine capacity bounds how far it can
  move, so a technique switch needs investment first. The inertia comes from capital
  and build lags, which is structural, not a tuned smoother. x\* per desk is an output.
- **Machine services: the recursion as production.** Machine-service desks buy machine
  services, labour and non-produced services per their (A, Λ, B), and sell machine-hours
  on the market. λ is never set: it is measured from the desks' own purchases.
- **Investment.** A desk (or an entrant desk for a newly published version) builds when
  its expected margin exceeds the user cost u = (ρ+δ)(1+ρ)^(J−1), valued at posted
  prices (`dynamics/`). ρ is registered and on the tape until the credit layer makes it
  a market price.
- **Participation: the priced outside option** (worker pops). The pair posts labour
  scaled by a participation share. That share moves with the gap between the wage and
  the outside option s = max(s₀ − q·h_e, s̲), where q is the market price of site and
  land services relative to goods. Exit is real: hours withheld produce home output
  using land and site services the pop must hold or rent. On unenclosed land those
  services are free, which is what a commons is. This s(q), with the idle margin and
  parcel quality, is `main.tex`'s form and the default for the historical runs. The
  posted SSRN version prices exit as dependence, s = (e^χ − 1)P_s; that form is the
  named alternative (R6).
- **Consumption.** Category demand around a subsistence basket; the Allen basket anchors
  the welfare ratio. Within a category, a logit over goods. Owners also demand
  human-required services; domestic service was among England's largest occupations
  through the 19th century, and owners' demand is what priced the wall.
- **Rent and profit** flow to claim holders every activation.

No agent ever reads the oracle (R13).

### 3.3 Markets and settlement

These are the July markets spec, kept. Last tick's price is this tick's price. Clearing
is pro-rata per node and good, prices update by imbalance, and there are no floors or
clamps. Settlement is symmetric by construction (the v1 asymmetry, REVIEW §2.2 item 8,
is fixed at the design level): buyers and sellers are settled from the same filled
quantity, and cash constraints bind at the order, never at settlement. Rationing is
recorded per buyer class: who goes short is an output.

### 3.4 The oracle

The oracle is a static equilibrium solver for the pinning economy, written in Rust,
sharing types but not logic with the agents. It starts from Appendix B (SSRN version),
extends to Appendix A's general system, and must reproduce the paper's published
numbers. It has three jobs:

1. **In CI:** in stationary configurations, the agent economy's long-run averages must
   converge to the oracle's equilibrium within a registered tolerance, from perturbed
   starts.
2. **At genesis:** seed prices and quantities at the oracle's equilibrium for the 1750
   states, so the burn-in is short and the transient is measurable.
3. **In every run:** report the oracle gap, the distance between the agent economy's
   prices and the equilibrium at the current states. It is a readout, never a force.
   How the gap widens in crises, and how long it takes to close after a technology
   arrives, is itself data.

### 3.5 Slow states

- **Population:** N′/N = φ(W − 1) in the floor configuration (Bouscasse, Nakamura and
  Steinsson's Malthusian block as the benchmark). Mortality shocks and the demographic
  transition are a scripted envelope until P4's household-formation closure exists.
  1750 is late in the floor era, so the floor block is identified from a pre-sample: the
  long-record reduced-form fit over 1250–1750, pinned to a commit, or an unscored burn-in
  world from 1700. Decide at Phase 6.
- **Enclosure:** the tape switches open access off per parish or county and land class,
  dated by Turner's acreage. Price closes it too, once no suitable land lies idle: the
  idle margin of the default exit form (§3.2).
- **Technology:** the tape publishes machine types, schedule shifts and recipe versions,
  per region and date. A schedule shift is either how far machines reach or how good
  they are where they already reach; `dynamics/` T5 shows the two have opposite wage
  signs on impact. Adoption is emergent, through investment. The knot budget is capped
  and pre-registered (R7).
- **Training:** type shares move slowly: apprenticeship, then mass schooling from the
  1870 Education Act, on the tape.
- **Migration** between regions is a scripted envelope at first. Rural-to-urban
  migration within England is the first candidate to make emergent.

### 3.6 Money, credit and crises (Phase 8)

The July money design, kept:

- currency is a good, conserved with provenance;
- credit is double-entry postings, and banks are desks;
- monetary regimes are laws gating the mint: specie, the suspension of 1797–1821, gold
  1821–1914 and 1925–31, Bretton Woods, floating, inflation targeting.

Under a metallic standard the nominal anchor is the relative price of gold. Gold is a
mined good on a non-produced deposit, so the discoveries of 1848–51 and the gold
scarcity of 1873–96 enter through the same recursion as everything else. `paths/`
supplies measured behaviours for bank and household rules: deposit pass-through that
rises more slowly than it falls, rigid money wages, support networks with finite
capacity. It also supplies the episodes to score against. Its reduced-form equations
are references, not code to port.

### 3.7 Worldgen: regions, channels, the tape

The July compiler, kept and built early. Human-editable tables compile into one
validated tape: regions with coordinates, endowments, parcels, deposits and population;
channels; timelines for technology, enclosure, laws and taxes, monetary regimes, wars
and world prices. The compiler generates ids, checks references and units, and signs
the genesis. Granularity is a compiler setting.

- **The first world is England by historic county** (about 40 regions, the grain of
  Turner's enclosure data and the 1841+ census), with coalfields, ports and towns placed.
  The rest of the world enters as scripted prices at the ports, registered as scripted
  and therefore never scored.
- **The second world adds macro-regions:** continental Europe, the US, China, India, a
  periphery.
- **Finer partitions** come after that.

### 3.8 Certification and telemetry

The July engine spec, kept. Most of it was built and tested on the July branch; the
ledger and the state hash move into `core` in Phase 0, and the rest in Phase 0's second
session:

- a conservation ledger with provenance, where a shortfall is a ledger line and never a
  clamp;
- golden-hash tests for repeat, resume and replay;
- canonical delta order and ordered reduction across region shards;
- verdict-first certificates persisted to `results/`, with dated criteria files;
- tidy long Parquet telemetry.

New batteries: the oracle convergence battery, the oracle gap, a price-runaway detector
(R14), and the scorecard against the record. The observables are the record's own:
welfare ratio, rent/wage, land's share, labour's share, participation, category
deflators (the fork), λ, κ, the labour-income share of tax revenue, GDP per head,
urbanisation. Analysis runs in Python over the Parquet.

### 3.9 Scale budget

Tick length stays configurable: weekly by default, as in July. It is new work: July's
tick-time rule was three v1 doc sections, and no branch had a tick-length parameter. The
tape registers the tick length as `ticks_per_year`, and every dial carries a time unit,
the EMA span in years first. A criterion checks that results do not depend on the tick
length. The oracle solves per period, so either the tick is its period or its inputs
are restated at tick length, with Δ = 1/`ticks_per_year`: ρ_tick = (1+ρ)^Δ − 1,
δ_tick = 1 − (1−δ)^Δ, and J_b in ticks. 1750–2050 at weekly ticks is 15,600 ticks.
Initial performance targets, recorded as gates:

- the England county world (about 40 regions) runs 1750–2050 in minutes on one machine;
- a world of several hundred regions runs in under an hour.

Ensembles for fitting run as parallel processes. This is where the Rust headroom is
spent: fitting an agent model means many full runs.

## 4. Standing rules

These replace R1–R14 and are numbered so checks and certificates can cite them.

- **R1 — One economy, nested.** The oracle reproduces the paper's published numbers. In
  stationary configurations the agent economy converges to the oracle. Each optional
  layer, switched off, reproduces the run without it. The reductions are checks, not
  intentions.
- **R2 — Conservation is asserted.** Every unit of every good and of currency is created
  and destroyed only with provenance, and a failure stops the run. The income identity
  follows from conservation and is checked as well. (July R3.)
- **R3 — Structural stability only.** Stability comes from buffers, dead-bands, small
  asymmetric steps, stagger, capacity and build lags. It never comes from price clamps,
  sell caps, demand caps, balance forgiveness or an added smoothing layer. (July R1.)
- **R4 — Every number has a provenance.** Each is measured (with source and vintage),
  from the literature, approximate, fitted (with the fit's id) or assumed. All are
  registered in scenario data, sweepable, and enumerated in one place. No behavioural
  literal lives in code. (July R2, plus `paths/`' named defaults.)
- **R5 — Targets are never inputs.** The scripted/emergent registry is dated. Fitted and
  scored moments are disjoint and registered before the run. (July R6.)
- **R6 — Structure fixed, behaviour swappable.** Each agent rule is named and has a
  default plus at least one alternative, one of them extreme. (`paths/`.)
- **R7 — Parameters are budgeted.** Moments must comfortably outnumber parameters. The
  technology path's knots are capped, and fixed before the fit they serve. (Long-record
  risk 3.)
- **R8 — Determinism is a tested property.** Repeat, resume and replay give identical
  hashes. There is no unordered iteration on the delta path, and any randomness is
  seeded per actor. Transcendentals on the state and verdict paths go through one
  pure-Rust implementation (the `libm` crate). Hash equality across platforms is
  recorded, not gated. (July R4; ADDENDUM A5.)
- **R9 — Checks gate; runs certify themselves.** Certificates are verdict-first,
  fail-closed and persisted, and NaN fails. (July R5; laborformal.)
- **R10 — Complexity and simplicity both need receipts.** A mechanism ships with the
  check or criterion that demands it; a simplification ships when the certified suite
  says it is no worse. (July R10.)
- **R11 — Primary sources, validated; stop and report.** A series is never substituted
  silently. Contestable constructions run as labelled grids and report bands.
  (laborformal.)
- **R12 — Rationing is recorded.** Shortage resolves by pro-rata rationing, recorded per
  buyer class. (July R8.)
- **R13 — Bounded rationality.** Agents read posted prices and their own state. No agent
  simulates another, and no agent reads the oracle. (July R13.)
- **R14 — Costs are quantities of goods.** No cost, floor or instrument is a hardcoded
  currency amount. The rule covers engine thresholds too: no guard on a price change, a
  currency flow or a quantity is an absolute epsilon, and a tolerance is relative and
  registered. (July R12; ADDENDUM A12.)
- **R15 — The firewall, revised.** Scoring this model against the record is a
  rejectable test of the configuration reading. It is never evidence for the paper's
  theorems, which stand on their proofs and checks. Anything borrowed from laborformal
  is pinned to a commit and read from that commit, never from a checkout; a re-pin is a
  dated entry. (July R11; ADDENDUM A6.)
- **R16 — Frontends observe; the tape decides.** No frontend mutates engine state. Every
  intervention (a dial, a shock, a world edit) is a tape edit with provenance. Each entry an
  edit adds or changes carries its own basis. A number without a basis of its own is not
  edited until the schema gives it one. A saved edit names its parent tape. The engine
  runs an edited tape as it runs any tape, and on the same platform and build the hashes
  equal the cli's.

  A checkpoint is a cache of a run, not an input. A frontend resumes only from a
  checkpoint that a run of the same tape made, verified by its state hash.

  A simulated number that a frontend shows or exports names its run (build commit and
  dirty flag, `world_id`, tape hash) and its tick. A record number names its source and
  vintage. An oracle number names its build and instance. The cli's `resume` and its hash
  output are held to this paragraph and the one before once ENGINE rules on its §7.6 and
  §8.

  A frontend copies no record or oracle value into a tape, and the scorecard refuses a
  tape edited in one. (R4, R5, R8; ENGINE E1, E4; ADDENDUM A14.)

## 5. The registry at founding (2026-09-25)

**Scripted:**

- technology (machine types, schedule shifts, recipe versions, per region and date),
  within the knot budget;
- enclosure;
- mortality shocks, and the population envelope after the demographic transition;
- migration envelopes;
- laws and taxes (rates and dates) and monetary regimes;
- wars (government purchases, destruction, debt);
- world prices at the ports until the world phase;
- the 1750 genesis;
- after 2025, the forward scenarios.

**Emergent and scored (1750–2025):**

- all prices and wages, by region and worker type, and the welfare ratio;
- rents and land's share; labour's share;
- category prices and the fork;
- participation, and the use of unenclosed land;
- each desk's task margin, the regime (wall or interior) and their dates;
- adoption of each technology version;
- λ, measured from the model's own purchases;
- κ;
- population in the floor era (while φ is on);
- the composition of tax revenue by base;
- rationing: who goes short.

**In between:**

- Population is endogenous in the floor configuration and scripted after the
  transition.
- Technology is scripted as availability; adoption is emergent.
- Migration is scripted in total and emergent in its destination, once that rule
  exists.

Any change to these lists is a dated entry with a reason.

## 6. Phases

Each phase leaves one engine green under one harness, and each has an explicit gate. A
unit is one working session with its own tests. Session counts are honest guesses.

The GUI's stages, G0–G10 and W1 (A14; [GUI.md](GUI.md) §9), sit beside the phases, built
with egui in `crates/gui`. Each starts after its phase's gate, with three exceptions: G4
starts on Phase 4's gate or Phase 5's first validated county series, whichever comes
first; G10 runs beside Phase 10, which is open-ended and has no gate; and W1, the reader
build, starts when the paper needs it, after G4 at the earliest. No stage gates engine work. Their session counts are guesses too.

### Phase 0 — Reboot (2 sessions)

1. Tag the July work first: `july-v2-phase-0` (`cf7e78f`), `july-v2-phase-1` (`bb57eea`)
   and `july-v2-phase-3` (`ff01284`), pushed to `origin` with `pre-foundations`. Then tag
   `pre-reboot-2026-09-25`.
2. Build in WSL Ubuntu, where the gates run through `wsl --exec`, and on Windows as the
   secondary check (ruling 7). `rust-toolchain.toml` pins one version for both; record
   it in STATE.md.
3. Lay out a cargo workspace: `core` (ids, goods, inventories, deltas, state, ledger),
   `markets` (clearing, settlement, price update), `oracle`, `agents`, `worldgen`
   (the tape compiler), `certify`, and a `cli`.
4. Salvage from the July branch, tag `july-v2-phase-3` (`ff01284`), not from v1: the
   `core`, `markets` and `cli` rows of ADDENDUM §2.4's salvage map, with defect 10 and
   N1–N3, N5–N9, N11, N13 and N14 fixed as the code moves: ids; inventories (with lot
   lives serialized, and lots coalescing by life before the first golden hash); the
   delta pattern (canonical order, shortfalls returned to the ledger); clearing; the
   price update (its EMA span registered in years); the event schedule, as the tape's
   runtime form. July's `test_03` is the hash suite's template. The 11 absolute-epsilon
   sites behind N5 and N6 are removed, not registered (R14). The tape registers the tick
   length, and every dial carries a time unit (§3.9).
5. Archive the rest: the v1 agents, tools, notebooks, the lr corpus, the July and v1
   docs. `test_01` leaves with the v1 agents, and STATE.md logs it as failing at every
   commit where its tests compile. `docs/timeline/eras.md` stays and extends back to
   1750.
6. Move this plan to `docs/PLAN.md`, write STATE.md as the resume point, and put up a
   CI skeleton with the repeat-hash test.
7. In a second session, move the certification stack from the July branch (certificate,
   criteria, verdicts, the NaN scan, the manifest, telemetry, Parquet), with N4, N10,
   N12 and N15 fixed, its thresholds moved into criteria, and a price-runaway detector
   added (R14). The July tracer certifies FAIL today, so nothing certified in July
   carries as a verdict.

**Gate:** on a scripted-desk world, `cargo test` is green in WSL and on Windows, with the
salvaged unit tests and:

- repeat, resume and replay give identical hashes, the resume through the product's own
  resume path (N11);
- conservation holds every tick;
- one test per fixed defect, among them: a cash-short buyer settles both sides from one
  fill; two actor kinds sharing an id settle apart; a burn shortfall stops the run;
  unsorted events fire or fail.

The second session keeps this gate green and adds the certification stack's tests.

**GUI:** G0, the shell (2–3 sessions), follows the second session, while Phase 1
continues: G0.1 the viewer (run, plots, inspector, registry, log), then G0.2 the editor,
branches, compare and export. The second session lands G0's engine asks first
([GUI.md](GUI.md) §7.2).

### Phase 1 — The oracle (2–3 sessions)

Build it outward from the SSRN version's Appendix B:

- 1a. One category, with durability and interest, and the scalar user cost
  u = (ρ+δ)(1+ρ)^(J_b−1). Exit takes the SSRN dependence form, s = (e^χ − 1)P_s, since
  the published Appendix B numbers use it.
- 1b. Many categories and the fork.
- 1c. Many machine types, the Leontief inverse, and per-type user costs and build
  recipes.
- 1d. Worker types and the wall.
- 1e. Parcels with quality schedules, the idle margin, and exit as s(q), the default
  form (§3.2).
- 1f. Households and government.

The golden numbers are pinned to laborformal `31b3482` and read from that commit (R15).
Their source is the SSRN version (SSRN 7226858), whose only executable form is
`paths/code/macro.py` with `paths/checks/check_macro.py` P1; `main.tex`'s Appendix B is
another economy and is never the source.

**Gate:**

- the SSRN Appendix B instance: the published values (x\* 0.86315, v 0.54344,
  Y 7.88061, N_a 1.34338) to 5e-6, and ADDENDUM §5's full-precision values to 1e-12
  relative;
- the replacement closure's worked instance (c = 1, w = 3; at λ = 0, c = 0.4 and
  w = 1.2);
- the fork identity and the category bounds on random instances;
- the income identity to 1e-12;
- three-taxes' resolution ledger, (φ_w, φ_r) = (0.6, 0.4) on its worked instance;
- constructed wall and interior cases recognised correctly;
- each exit form on its own gate: the dependence form in 1a, s(q) in 1e. The 1d and 1e
  gates are constructed (ADDENDUM §5 item 4).

**GUI:** G1, the oracle lab (1–2 sessions), after G0 and this gate: an instance's regime
and outputs beside their goldens, any field over x, one-parameter sweeps, and the
price-step explainer.

### Phase 2 — Agents meet the oracle (3–5 sessions)

One region, stationary history. The phase opens with July's battery (ADDENDUM §3): the
five solvable worlds and their closed forms, re-expressed as tapes; `test_12`'s mode A
(hold) and mode B (2× displacement, on the four worlds that register one); `test_15`'s
sweep and basin scan; and the elasticity probe, which measures ε_s per market before any
battery. Build the ontology and agent rules of §3.1–3.2 on the salvaged markets, with
the conservation ledger live; the scale rule is re-tested first, with §3.2's changes.
Then run the convergence battery. Four stationary configurations: the Appendix B
instance, a wall-regime instance, an open-commons instance, and a 1750-like instance.
From perturbed starts, which exceed ±dead and include cost shocks, the agent economy
must reach each oracle equilibrium within tolerance and stay there. Then map the phase
diagram over the stability dials (dead, July's axis, and α; step ratio, buffers,
stagger), the July Phase 5 folded in here.

**Gate:** the convergence battery is green with margin, judged jointly: cleared volumes
against the oracle's quantities, contraction over the whole series, the band, and
relative prices against a numeraire whose market cleared. Every A/B gate has an
absolute liveness floor. A stable region is documented and referenced by the default
dials.

**Kill condition, stated in advance:** once §3.2's changes are measured on the solvable
family and the basin is mapped against dead and α, within a session budget you set, a
failing mode B triggers the fallback. The fallback is to run the oracle as the
per-period core, with agents as an overlay for adjustment and rationing; it is planned
as a real branch.

**GUI:** G2, lenses (2–3 sessions), after this gate: the oracle line and gap, the
decision inspector and binding marks, the battery panel, the stability sweep as a
heatmap, and the first browser build.

### Phase 3 — Time, capital and the tape (2–3 sessions)

Vintages, investment at the user cost, population, enclosure, technology on the tape.
Checkpoints (lossless), Parquet and certificates arrive earlier, in Phase 0.

**Gate:**

- `dynamics/`' windfall and waterfall reproduced in shape, with the oracle's
  steady states at both ends;
- resume and replay hashes equal;
- a 1750–2050 single-region tracer at weekly ticks passes its certificate within the
  §3.9 performance target, over a window named relative to the run; a frozen or ramping
  window is rejected (`31f27e5`).

**GUI:** G3, long runs (1–2 sessions), after this gate: the chunked store and its spill,
seek, and the vintage, investment, population and enclosure panels.

### Phase 4 — Worldgen and county England (2–4 sessions)

The compiler; England by historic county with parcels, coalfields, ports, towns and
channels (roads, canals and railways on the tape); world prices at the ports.

**Gate:** the compiler's validation and signed genesis; after a short burn-in, the 1750
county world sits within tolerance of the oracle at its states; the England world meets
its performance target.

**GUI:** G4, the map (2 sessions), starts on this gate or on Phase 5's first validated
county series, whichever comes first: the county atlas, with Yorkshire in its three
ridings (42 regions), lenses on the map and the world editor. The atlas ships under ODbL
with its attribution, in its own data directory.

### Phase 5 — The data spine (2–4 sessions; can run alongside Phases 1–4)

England 1700–2025, annual where the sources allow, decadal where not, and by county
where the source has it.

**Sources:** the Bank of England's millennium dataset first (reachable); then

- Clark's aggregates and farmland rents;
- the BNS replication files;
- Broadberry et al. for output and Feinstein for wages;
- Allen's welfare ratios (blocked from this machine; alternates per long-record §7);
- Wrigley–Schofield for population;
- Turner's enclosure acreage by county;
- census occupations by county from 1841;
- Mitchell for taxes and prices;
- the ONS for the modern end.

Validate series by series. Write `DATA_NOTES.md` and a one-page descriptive sheet.

**Breakpoint B — the eyeball test (kill point for the long-record purpose).** Before any
fit, the raw series must show three things: the floor era's opposition between
population and the wage, the escape, and land's exit.

**GUI:** G5, the record (1 session), after this phase: overlays with source, vintage and
bands, and Breakpoint B's eyeball sheet.

### Phase 6 — The floor and the escape, 1750–1870 (3–5 sessions)

1. Register the moment table and the fitted/scored split first.
2. Settle the floor block from its pre-sample (§3.5).
3. Freeze the technology knots (six at most).
4. Fit with ensembles.
5. Score Engels' pause; don't fit it.
6. Run D1–D3, dating the switch from the wage side and from the land side
   independently.

**Breakpoint C — magnitudes and novelty.** Are the fitted values sane? Does one history
of schedules and recipes generate both the pause and the escape? Decide the paper's
vessel with the long-record thread.

**GUI:** G6–7, fitting and scoring (1–2 sessions), after this gate, its ensemble parts
after Phase 7's: the scorecard viewer, ensemble fans and figure data bundles.

### Phase 7 — The machine era, to 2025 (3–4 sessions)

Score, don't fit:

- the fork;
- the U-turn in site rent (Rognlie 2015; Knoll, Schularick and Steger 2017);
- λ from the model's own purchases;
- κ, labour's share, and the composition of tax revenue.

**Gate:** the first certified England run, 1750–2025, with its scorecard committed.

### Phase 8 — Money, credit and crises (3–5 sessions)

§3.6. **Gate:**

- off, the layer reproduces Phase 7 exactly;
- on, one parameter set scores the UK episodes: the crises of 1825, 1847 and 1866;
  1920–21; 1929–32; 1973–75; 2008–10. Scoring is qualitative, with bands.

**GUI:** G8, money (1 session), after this gate: balance sheets, credit lenses and crisis
overlays.

### Phase 9 — The world (3–5 sessions, then open-ended)

Macro-regions, then finer ones through the compiler. Trade on transport desks;
migration envelopes; the first globalisation.

**Gate:** score the Atlantic grain gap closing (1870–1913) and the divergence in real
wages across regions (Allen's international welfare ratios). `paths/`' US 1929–33 and
2008–10 episodes are scored once the US exists.

**GUI:** G9, the world (1–2 sessions), after this gate: the world atlas, and trade on
channels.

### Phase 10 — Forward to 2050, and the policy lab (open-ended)

From the certified 2025 state:

- `paths/`' scenarios as tape generators: AI's two waves, the care absorber, the
  financing rules. The pace seen so far, at most 0.15 of the paper's, is stated beside
  every central run.
- three-taxes' counterfactuals;
- historical counterfactuals.

**GUI:** G10, forward branches and the policy lab (1–2 sessions), beside this phase: a
branch tree from the certified 2025 state, and generators as forms.

**Total:** roughly 35–50 sessions to the first certified England run (the end of Phase
7): the phases' own ranges sum to 19–30 (Phase 0 at two sessions), which this plan put at
roughly 25–35, and G0–G7 add about 10–15 (A14). Phases 0–2 stand on their own: a checked
oracle for the paper's economy, and an agent economy known to find it.

## 7. Risks and the parking lot

**Risks:**

1. **Agent convergence (Phase 2).** This is the review's main open risk. Tested on
   July's rules: it held, and did not return beyond about 5% but for one configuration.
   It is now tested again early, against a known answer, with a kill condition stated in
   advance and the fallback planned as a real branch.
2. **Fitting an agent model is expensive.** The parameter budget (R7), ensembles on the
   Rust headroom, and scoring by bands rather than point-matching are the defence.
3. **County-level data before 1841 are thin.** Where a county series does not exist,
   score the England aggregate and say so.
4. **The raw record may not show the switch (Breakpoint B).** Then the long-record
   purpose ends, and the engine serves the forward branches and the policy lab from a
   calibrated 2025 world.
5. **Data access.** Allen's site is blocked, and curl is blocked for several academic
   hosts from this machine. Stop and report; never substitute (R11).

**Parked**, each with its re-entry condition:

- **Endogenous technology; war logic; firm-level market power.** The wedges μ(x) stay
  institutional parameters.
- **Secondary markets in claims and asset bubbles:** these enter through the credit
  layer's leverage, per the July money design.
- **Emergent migration beyond England:** after the world phase.
- **Individual agents below pop grain.**

## 8. Relationship to laborformal

| laborformal thread | Role here |
|---|---|
| `pinning/` | The oracle's specification and golden numbers |
| `dynamics/` | The user cost, vintages and build lags; the transition shapes to reproduce |
| `long-record/` | The scoring spine and fitting discipline; the paper that cites the runs |
| `paths/` | Measured behaviours for agent rules; crisis episodes to score; forward scenarios as tape generators |
| `three-taxes/` | The policy lab's instruments and its checked ledger |
| `capability/`, `companion/` | Worker types; modern moments for the margin (the w/c waterline, the revealed-adoption envelope) |
| `progress_and_prosperity/` | The figure spine. P4's demographic closure replaces the population envelope once it exists. |

laborformal is Python and rustyecon is Rust, so what crosses between them is
specifications, golden numbers and data, each pinned to a commit, not code. What flows
back is runs a paper can cite by commit hash.

## 9. The first session

Phase 0 runs on WSL (ruling 7). Its first session, the salvage and the engine, is followed
by a second, the certification stack with G0's engine asks (A14), with Phase 1's units
built in parallel. Then comes G0, the GUI's shell, while Phase 1 continues.
Every ruling is in; nothing blocks the start.
