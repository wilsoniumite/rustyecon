# rustyecon

rustyecon runs 300 years of economic history, 1750–2050, as an agent economy in a
granular world. History is scored against the record from 1750 to 2025; from 2025 to
2050 the engine runs forward branches.

Every decision an agent makes is one of the margins of the pinning paper (*Pinning the
Wage to Scarcity and Technology*): producers choose between people and machines task by
task, machine makers sell machine-hours at what their inputs cost them, land and sites
rent for what their users pay, and households choose between work and a life outside
it. Nothing is solved centrally. Agents read posted prices and their own state, and
markets clear by rationing. A separate equilibrium solver, the oracle, computes where the
economy must settle when history stands still, and the agents are tested against it.

What it is for, in order:

1. Generate the long record for England and the UK, 1750–2025, from one set of
   parameters.
2. Run the forward branches, 2025–2050, from a certified 2025 state.
3. Run the policy lab: a rent tax with a uniform transfer, a consumption gate tax,
   payroll and income taxes, Poor Law regimes, and historical counterfactuals.
4. Explain.

## Status

Phase 0, the reboot, is under way. The workspace is laid out and the crates fill in phase
by phase:

| Crate | What it holds | Fills in |
|---|---|---|
| `crates/core` | ids, goods, inventories, deltas, state, the conservation ledger, the state hash, checkpoints, the tape's runtime form | Phase 0 |
| `crates/markets` | clearing, settlement, the price update | Phase 0 |
| `crates/certify` | certificates, criteria, verdicts, Parquet telemetry | Phase 0, second session |
| `crates/cli` | the `rustyecon` binary: tick loop, replay audit, resume | Phase 0 |
| `crates/oracle` | the equilibrium solver | joins in Phase 1 |
| `crates/agents` | the agent rules | Phase 2 |
| `crates/worldgen` | the tape compiler | Phase 4 |

Packages are named `rustyecon-<crate>`.

## Documents

- [docs/reboot/PLAN.md](docs/reboot/PLAN.md): the plan. Architecture, the standing rules
  R1–R15, the phases and their gates.
- [docs/reboot/REVIEW.md](docs/reboot/REVIEW.md): the review of the repository at the
  reboot, with the rulings made after it.
- [docs/reboot/ADDENDUM.md](docs/reboot/ADDENDUM.md): what the review missed (the July
  branches), with the rulings on amendments A1–A13.
- [docs/timeline/eras.md](docs/timeline/eras.md): era research that feeds worldgen.

## Build and test

`rust-toolchain.toml` pins Rust 1.97.1 with rustfmt and clippy; rustup installs it on
first use. The gates ask for zero warnings and a clean clippy on both machines.

WSL Ubuntu is the primary build machine, and the gates run there:

```sh
cargo build --release
cargo test --workspace --release
cargo clippy --workspace --all-targets
```

From a Windows shell, run them through `wsl -d ubuntu --exec bash -lc '<command>'`. Use
`--exec`: with a plain `--` the exit code is lost.

Windows is the secondary check. The same three commands run there. Hash equality between
the two platforms is recorded, not gated.

`clippy.toml` enforces two standing rules: no std hash containers (R8, no unordered
iteration on the delta path), and no platform transcendentals (`exp`, `ln`, `powf` and
the rest go through one module backed by the `libm` crate).

## History

The v1 engine, its Python tooling and notebooks, the scenario corpus, and the v1 and July
design documents left HEAD at the reboot. They stay reachable at the tag
`pre-reboot-2026-09-25`. The July v2 engine, never merged, is at the tags
`july-v2-phase-0`, `july-v2-phase-1` and `july-v2-phase-3`; Phase 0 salvages from
`july-v2-phase-3`. Read any of them with `git show <tag>:<path>`.
