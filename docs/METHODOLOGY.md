# Methodology

Why the architecture is shaped the way it is, and the standing rules the
project works under. ARCHITECTURE.md says *what*; this document says *why* and
*never*.

---

## 1. Design rationale

Three commitments generate most of the design.

**Stability is structural.** Tick-based posted-price economies have a
well-documented failure mode: when every agent applies a full-strength
adjustment to the same signal on the same tick, the system oscillates — and
much of that oscillation is numerics (explicit synchronous updates of stiff
dynamics), not economics. The models with proven stable, rich behaviour —
Mark-0 (Gualdi, Tarzia, Zamponi & Bouchaud 2015), Lengnick (2013), the
EURACE lineage — all get stability from the same structural package: small
asymmetric multiplicative steps, dead-band gating, inventory and cash buffers
as low-pass filters, and desynchronized activation. The kernel is built from
exactly that package, and from nothing else. The alternative — stabilizing
with tuned clamps, caps, forgiveness constants, and layered smoothing — hides
the dynamics it suppresses, accumulates unregistered constants, and still
fails; it is banned outright (R1).

**Agents are radically simple, and the simplicity is the science.** Fewer free
parameters than target behaviours is the only regime in which reproducing
anything means anything: a handful of registered dials economy-wide is a
claim, while dozens of hidden constants are paint. One kernel shape is also
one thing to stabilize, one phase diagram to map, one object to hold in your
head, and one line per desk for the world compiler to emit. And it is
theory-shaped: the sister two-economies project closes its dynamics with a
single behavioral law — agents want to consume; income beyond the day's
absorption overflows mechanically into building and enclosing. The kernel is
that law as code. Simplifying toward the validated ABM literature and
simplifying toward the theory turn out to be the same motion, and the design
leans into the coincidence.

**Runs certify themselves.** A simulation result that is not conserved,
deterministic, replayable, and scored against pre-registered criteria is an
anecdote, whatever it looks like. Certification lives in the engine — asserts,
golden-hash tests, verdict-first certificates — because a check that lives in
a notebook is a check that eventually gets skipped.

## 2. Aggregation choices

Individuate what the research questions need; aggregate everything else.

- The questions are distributional — wages across a capability ladder, claims
  across holders, who is rationed, who crosses the market boundary — so pops
  carry capability, tier, claims, and a participation margin.
- Firm-level industrial texture is not the target, so production is one desk
  per recipe per region: price-takers with identical information make
  identical decisions, and duplicating them adds noise, not dynamics.
- Land is first-class (parcels, enclosure, rent) because land is where the
  theory says overflow terminates and where the policy layer bites.
- Almost everything is a good — physical goods, services, labour, currency —
  cleared by one mechanism; the deliberate exceptions are claims (minted,
  never traded) and consumption itself (a sink, not a market).

## 3. The standing rules

Numbered so criteria files and audits can cite them.

**R1 — Structural stability only.** Stability comes from buffers, dead-bands,
small asymmetric steps, and staggered activation. Never from price clamps,
demand caps, balance forgiveness, or an added smoothing layer. If a run
oscillates, the fix is a mechanism, not a constant.

**R2 — Every constant is registered.** No numeric literal with behavioral
meaning lives in code. All dials sit in scenario data, are sweepable by the
experiment harness, and are enumerated in one place. A constant that cannot be
swept cannot be defended.

**R3 — Conservation is asserted, not assumed.** Every unit of every good and
every unit of currency is created and destroyed only with provenance. Silent
clamps are bugs by definition. A conservation failure panics the run; it never
becomes a mystery in a notebook.

**R4 — Determinism is a tested property.** Same (tape, code, seed) → same
hashes; checkpoints resume identically; the delta stream replays identically.
These are CI tests, not intentions. Parallelism must preserve them (ordered
reduce, no unordered iteration on the delta path).

**R5 — Verdict-first, fail-closed, persisted.** Every run prints a
certificate: batteries pass/fail before any result is read; a metric that
cannot be computed FAILS (a value *declared missing* under a registered
convention — e.g. μ/r\* on a quiet tick, [ownership.md](architecture/ownership.md)
— is not a failure; the convention is registered like any constant); verdicts
are committed to the repo. Failures are evidence. A result without a
certificate does not exist.

**R6 — Targets are never inputs.** What the tape may script and what must
emerge is a registered, dated list (§4). A series a run is scored against may
never be paintable by tape entries or dial choices. Validation standard:
phase-diagram robustness first — which qualitative regimes exist, and are they
robust to micro-rule tweaks — with historical point-matching a distant second.
Reproducing a century with enough dials is transcription, not generation.

**R7 — One running sim.** Every change leaves one engine, under one test
harness, green. No sibling engines with a delete step at the end. The scenario
corpus of record is the regression suite until a certified replacement exists.

**R8 — Rationing is the point.** Shortage resolves by explicit pro-rata
rationing, recorded per buyer. Who goes short is a first-class output — the
distributional consequence of scarcity is the research question. No stateless
clearing, no price clamps that hide scarcity, no shortage malus bolted on
beside the price system.

**R9 — Money is what crosses perimeters.** Every settlement is tagged crossing
or internal; every aggregate exists twice (measured / true). Unpriced
provision is real activity the ledger cannot see, and the boundary between the
two moves — that motion is output, never an accounting error.

**R10 — Complexity and simplicity both need receipts.** A simplification ships
when the certified suite says the simple version is no worse (A/B under
identical scenarios, persisted verdicts) — not because it is prettier.
Symmetrically, complexity ships only with a failing test or criterion that
demands it.

**R11 — The two-economies firewall, both directions.** Simulation runs are
never evidence in the theory project: retrodiction is corroboration at most,
and a calibrated reproduction is worth nothing as support. Legitimate exports:
impossibility results within a mechanism class, mechanism discrimination,
theorems-by-computation about a model, candidate falsification surfaces (which
only data may judge), and pedagogy. In the other direction: any constant
borrowed from the theory (cohort cuts, estate parameters) is version-pinned to
a dated commit; the simulation does not silently chase rulings.

**R12 — Costs are quantities of goods.** No cost, floor, or policy instrument
is ever a hardcoded currency amount; everything prices through the market. A
currency-denominated cost pins a price outside the price system and prevents
equilibration.

**R13 — Bounded rationality, O(own information).** Agents read posted prices
and their own state; no agent simulates another; no global optimization
anywhere. Aggregate rationality is allowed to emerge; it is never assumed.

**R14 — Docs carry verdicts, not vibes.** Design documents state calibration
targets as executable criteria. Superseded text is marked with its verdict
in place, never silently rewritten or deleted; the marks are the record.

## 4. The scripted/emergent registry

The single most consequential modelling boundary, kept as a dated list.
Changing it is a design event: new dated entry, reason recorded.

**The tape may script:** technology (recipe versions, capital-cost paths,
regional availability dates); wars (capacity destruction, scripted demand,
embargo modifiers, scripted transfers); laws and policy (taxes, tariffs via
transport recipes, regulatory modifiers, estate rules, abolition, monetary
regime changes and money-stock events); population paths (size scaling per
region); genesis (initial endowments, claims, parcels, prices).

**Must emerge (never scripted, always scoreable):** all prices and wages; all
production, consumption, and trade quantities; capacity allocation (which
recipes get minted where and when); participation and the self-provision
margin; the wage distribution R; wealth concentration; μ and r\*; the timing
and depth of booms, busts, and shortages within a regime.

**Deliberately in between** (scripted envelope, emergent path): population is
scripted in total, but its employment, wealth, and market participation within
the envelope are emergent; recipe *availability* is scripted, but adoption is
emergent.

## 5. Relationship to the two-economies project

Same author, two instruments, one question. The theory project is the formal,
evidence-disciplined layer; rustyecon is the mechanism laboratory and the
prose layer's engine room — a running world with a visible money boundary is
the theory's best explainer. The simulation attacks exactly the joints where a
reduced aggregate model is degenerate — mechanism identification,
existence/impossibility results, policy exhibits (a VAT is a tax on perimeter
crossings; the dividend's stability role is a model theorem) — and never
competes with the theory for evidential authority (R11). Process discipline —
pre-registration, criteria batteries, persisted verdicts, mark-in-place
supersession — is shared across both.
