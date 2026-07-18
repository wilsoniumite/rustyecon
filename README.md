# rustyecon

A headless global economic simulation in Rust. Inspired by Victoria 3 but
stripped of game concerns and extended with modern economic systems. Primary
motivation: testing high-VAT / UBI policy. Broader goal: a high-fidelity,
extensible simulator fast enough for real economic experiments.

## Status

v1 engine implemented (~3.3k lines: market core + three-strategy agent layer);
stability suite was failing when v1 work paused. **v2 redesign ruled 2026-07-18**
— see the three documents below; the fourth deliverable is the section-by-section
triage verdicts inline in every existing doc.

## Documentation

**v2 (authoritative):**
- [Architecture](docs/ARCHITECTURE.md) — overview: the modules and how they fit
- [architecture/](docs/architecture/) — subsystem specs: objects, kernel, markets, ownership, pops, money, engine, worldgen
- [Methodology](docs/METHODOLOGY.md) — why the design is shaped this way + the standing rules (R1–R14)
- [Plan](docs/PLAN.md) — the phased route from the current code to the architecture, with gates

**v1 (triaged, kept as record):**
- [Design](docs/DESIGN.md) — v1 architecture; superseded by ARCHITECTURE.md, verdicts inline
- [Glossary](docs/glossary.md) — canonical term definitions, per-term verdicts inline
- [Systems](docs/systems/) — one doc per simulation system, verdicts inline (04/05 = deferred module design records)
- [Timeline](docs/timeline/) — historical era reference; feeds worldgen (kept)

## Key Constraints

- Global scale: 673+ regions, 100+ countries
- 225-year simulation (1800–2025, ~11,700 weekly ticks) completes in under 24 hours
- Weekly ticks by default; tick duration is configurable
- Headless-first; Python notebooks are the primary analysis interface
- Rust, intermediate level

## Current project state:

- ./src contains the rust program that runs scenarios. You don't need to pass --scenario to the binary
- We generally output to ./tmp
- We store example scenarios in ./data/scenarios
- When running python make sure to use the venv in ./venv
- We have some test and analysis scripts in python in ./notebooks
- There are python utilities in ./tools
- There is a streamlit app for creating, editing, running, and visualizing scenarios in ./tools
- There is a scenario, multi_region, that is used as a base by ./notebooks/06_labour_test_suite.py to generate other scenarios with the prefix lr. They represent the stability suite that can be evaluated by ./notebooks/07_stability_suite.py


### next steps:
- Superseded by [docs/PLAN.md](docs/PLAN.md) — start at Phase 0 (bugfix floor).

### old TODOs (all absorbed by the v2 docs)
- "stabilize test regions" → PLAN Phases 4–5 (desk kernel + phase map)
- "docs out of date" → triage verdicts inline in every doc, 2026-07-18
- "more tests" → PLAN Phase 1 (certification stack)
- "use ema prices more / remove storage_cost_per_tick" → both moot under the kernel (architecture/objects.md + kernel.md; dead attributes cut)