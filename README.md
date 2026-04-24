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
