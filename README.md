# rustyecon

A headless global economic simulation in Rust. Inspired by Victoria 3 but
stripped of game concerns and extended with modern economic systems. Primary
motivation: testing high-VAT / UBI policy. Broader goal: a high-fidelity,
extensible simulator fast enough for real economic experiments.

## Status

Design phase. No Rust code yet.

## Documentation

- [Design](docs/DESIGN.md) — architecture, key decisions, design principles
- [Glossary](docs/glossary.md) — canonical term definitions
- [Systems](docs/systems/) — one doc per simulation system
- [Timeline](docs/timeline/) — historical era reference for scenario design

## Key Constraints

- Global scale: 673+ regions, 100+ countries
- 200-year simulation (1836–2036) completes in under 24 hours
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
- Try and stabilize test regions

### TODOs for later, put here to not forget them
- the ./docs might be a bit out of date
- we should have more tests in the rust code itself and in ./tests
- we should probably use the ema smoothed prices in simstate more, instead of storing our own smoothed variables on all recipes.
- storage_cost_per_tick should be removed from goods, will be handled later some other way.