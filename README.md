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

Phase 0, the reboot, is done: both its sessions are closed and its gate is green (see
[STATE.md](STATE.md)). The second session built the certification stack
([docs/CERTIFY.md](docs/CERTIFY.md)): dated criteria, sealed verdict-first certificates,
the run's manifest and Parquet telemetry. The gate world and the Appendix B world both
certify PASS, and their certificates are in `results/`. A Phase 2 probe found that agents
at the paper's margins reach the oracle's equilibrium of the SSRN Appendix B economy
([docs/probe/REPORT.md](docs/probe/REPORT.md)). Oracle unit 1a, the first of Phase 1, has
landed. The GUI is designed ([docs/GUI.md](docs/GUI.md)), and its shell, G0, is under way:
G0.1, the viewer, is built, its seams and its panels; G0.2, the editor, comes next. The crates
fill in phase by phase:

| Crate | What it holds | Fills in |
|---|---|---|
| `crates/core` | ids and keys, goods, the clock and time units, inventories, deltas, state, the conservation ledger, the state hash, checkpoints, the tape's schema and runtime form, the `libm`-backed maths | Phase 0 |
| `crates/markets` | orders and admission, clearing, settlement, the price update | Phase 0 |
| `crates/agents` | the behaviour seam and the scripted actor; the agent rules | Phase 0; rules in Phase 2 (the probe's four Appendix B roles since P2.0.1) |
| `crates/engine` | `Sim`: the tick loop, checkpoints, resume, the replay audit, the read-only per-tick report a frontend drives and reads | Phase 0 |
| `crates/cli` | the `rustyecon` binary (`run`, `resume`, `replay`, `registry`, `certify`): arguments, files, exit codes, the build stamp | Phase 0 |
| `crates/certify` | dated criteria, the batteries and the kick check, sealed certificates, the run's manifest, and Parquet telemetry behind the feature `parquet` ([docs/CERTIFY.md](docs/CERTIFY.md)) | Phase 0, second session |
| `crates/oracle` | the equilibrium solver (library `oracle`): unit 1a, one category with durability and interest, reproduces the SSRN Appendix B ([its README](crates/oracle/README.md)) | Phase 1: 1a landed; 1b–1f to come |
| `crates/worldgen` | the tape compiler | Phase 4 |
| `crates/probe` | the Phase 2 probe's harness: the Appendix B tape's generator, named perturbations, per-tick observables against the oracle ([docs/probe/RULES.md](docs/probe/RULES.md)); its oracle-free measures are certify's | the probe, P2.0.1 |
| `crates/gui` | the interactive frontend, in egui: live runs, plots, lenses, the tape editor, and a county map ([docs/GUI.md](docs/GUI.md)); the binary `rustyecon-gui` | from G0, after Phase 0's second session; one stage beside each phase (the seams since G0.1) |

Packages are named `rustyecon-<crate>`. `tapes/gate.ron` is the Phase 0 gate world, and
`tapes/appb.ron` the probe's Appendix B world. `criteria/` holds each tape's dated
criteria, registered before its first certified run, and `results/` the certificates and
manifests they gave.

## Documents

- [STATE.md](STATE.md): the resume point. Where things stand, the decisions open to veto,
  and the next steps in order.
- [docs/PLAN.md](docs/PLAN.md): the plan, amended by the addendum's rulings, A14 among them.
  Architecture, the standing rules R1–R16, the phases and their gates, and the GUI's stages
  beside them.
- [docs/ENGINE.md](docs/ENGINE.md): the Phase 0 engine contract, with each step's
  amendments.
- [docs/CERTIFY.md](docs/CERTIFY.md): the Phase 0 session 2 contract, with each step's
  amendments: certification, the manifest, telemetry and the GUI's engine asks.
- [docs/TAPE.md](docs/TAPE.md): the tape's schema, with the gate tape as its example.
- [docs/probe/](docs/probe/): the Phase 2 probe's rules (RULES.md) and report (REPORT.md).
- [docs/spine/](docs/spine/): the data spine's notes (DATA_NOTES.md) and Breakpoint B's
  pre-look (EYEBALL.md).
- [docs/GUI.md](docs/GUI.md): the GUI's design (A14): its rules, architecture, panels, editor,
  map and roadmap, with its review ledger in `docs/reboot/`.
- [docs/reboot/REVIEW.md](docs/reboot/REVIEW.md): the review of the repository at the
  reboot, with the rulings made after it.
- [docs/reboot/ADDENDUM.md](docs/reboot/ADDENDUM.md): what the review missed (the July
  branches), with the rulings on amendments A1–A14.
- [docs/timeline/eras.md](docs/timeline/eras.md): era research that feeds worldgen.

## Build and test

`rust-toolchain.toml` pins Rust 1.97.1 with rustfmt and clippy; rustup installs it on
first use. The gates ask for zero warnings, a clean clippy and clean formatting on both
machines.

WSL Ubuntu is the primary build machine, and the gates run there. `scripts/gate.sh` runs
the whole gate with the build directory outside the tree: formatting, clippy, the tests,
the repeat-hash test by name, certify without Parquet, two runs of the gate tape through
the binary, whose hash streams must agree, the build stamp, the committed certificates
recomputed, the probe's report pinned, and telemetry written from two processes. Its
header lists each step. From a Windows shell:

```sh
wsl -d ubuntu --exec bash -lc '/mnt/c/<path to the repository>/scripts/gate.sh'
```

Use `--exec`: with a plain `--` the exit code is lost. Its main steps by hand, and a
certified run:

```sh
cargo fmt --all --check
cargo clippy --workspace --exclude rustyecon-gui --all-targets -- -D warnings
cargo test --workspace --exclude rustyecon-gui --release
cargo run --release -p rustyecon-cli -- run tapes/gate.ron --until 2080
cargo run --release -p rustyecon-cli -- certify tapes/gate.ron \
    --criteria criteria/gate-2026-09-26.ron --out <dir>
```

`certify` writes the certificate, the manifest and the hash file, prints the verdict
first, and exits 0 only on PASS.

Windows is the secondary check: the same script under Git Bash. Hash equality between the
two platforms is recorded in STATE.md, not gated. `.github/workflows/ci.yml` runs
`scripts/gate.sh` on GitHub's Ubuntu runner; it runs only once the branch is pushed.

The GUI never gates engine work (docs/GUI.md, D1): the workspace's default members leave
`crates/gui` out, and `scripts/gate.sh` excludes it from clippy and the tests and checks it
once without gating. `scripts/gui.sh` is the GUI's own gate, run at each of its stages:
formatting, clippy, its tests in release, and the GUI's hashes of the gate and Appendix B
worlds against the cli's. The window opens with

```sh
cargo run --release -p rustyecon-gui -- tapes/gate.ron
```

paused at tick 0 with every price plotted: Space runs and pauses it, `.` steps a tick. It keeps
its session in `$RUSTYECON_GUI_DIR`, or else the platform's configuration directory. The smoke
mode runs a tape to a tick, prints the CPU each frame took, and closes:

```sh
cargo run --release -p rustyecon-gui -- --smoke 2080 tapes/gate.ron
```

`clippy.toml` enforces two standing rules: no std hash containers (R8, no unordered
iteration on the delta path), and no platform transcendentals (`exp`, `ln`, `powf` and
the rest go through one module backed by the `libm` crate; `mul_add` is banned too).

## History

The v1 engine, its Python tooling and notebooks, the scenario corpus, and the v1 and July
design documents left HEAD at the reboot. They stay reachable at the tag
`pre-reboot-2026-09-25`. The July v2 engine, never merged, is at the tags
`july-v2-phase-0`, `july-v2-phase-1` and `july-v2-phase-3`; Phase 0 salvages from
`july-v2-phase-3`. Read any of them with `git show <tag>:<path>`.
