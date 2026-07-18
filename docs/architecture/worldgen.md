# Worldgen and the tape

## History as input

The tape is the simulation's only input: a dated event stream. Genesis — the
tick-0 world — and history — everything that changes it from outside — are the
same format, applied in phase 0 of each tick. A scenario is fully specified by
(tape, code version, seed); alternate histories are tapes that diverge at a
date; policy experiments are tapes that differ in their law entries.

What the tape may script and what must emerge is a registered boundary
(METHODOLOGY §4). Scripted: technology versions and capital-cost paths, wars
(capacity destruction, scripted demand, embargo modifiers, transfers), laws
and policy, population paths, monetary regime changes, estate and abolition
events, genesis endowments. Must emerge: all prices, wages, quantities, trade
patterns, participation, capacity allocation, concentration, r\*, and the
timing of booms and busts within a regime. **A series a run is scored against
may never be a tape input.**

## The compiler

Hand-authoring a many-region world is impossible at scale; the compiler makes
it a data problem. Small human-editable sources —

- `regions.csv` — name, coordinates, population, endowments per region
- `tech_timeline` — dated recipe versions and cost paths, per region
- `policy_timeline` — dated law changes
- `wars` — dated destruction, demand, embargo, transfer entries
- per-era IO sketches — which recipes exist where, at what rough scale

— compile into a validated tape: generated IDs (nothing hand-numbered),
dangling-reference checks, schema validation, and a conservation pre-check
(the genesis money and goods stocks printed and signed). The tape's serialized
form remains human-inspectable; it is a compilation target, not an authoring
surface.

## The era ladder

A 225-year run is validated in certified legs, not in one leap:

1. Run 1800–1850 against era-appropriate registered criteria (era criteria are
   `criteria.ron` files like any other — dated, fail-closed).
2. Snapshot losslessly. The certified snapshot is the opening state of the
   next leg.
3. Repeat per era. A failure is local and dated — "the 1850–1900 leg fails
   criterion 3" — never "the end state is garbage."

Early worlds use coarse macro-regions (~20); finer partitions are a compiler
setting, not a rewrite. The full-span tracer — a trivial world run end-to-end
under certification — stays in the regression suite permanently, so
long-horizon drift is caught by CI rather than by a research campaign.

## Validation standard

Phase-diagram robustness first, historical point-matching a distant second
(METHODOLOGY R6). A world "reproducing" history with enough tape freedom and
dial freedom is transcription, not generation; the era criteria therefore score
qualitative, era-appropriate stylized facts (regime behaviour, direction and
rough magnitude of the great trends) and the scripted/emergent boundary is what
keeps the scoring honest.
