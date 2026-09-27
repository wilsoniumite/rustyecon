# rustyecon-oracle

Dated 2026-09-25; joined the workspace on 2026-09-26 (P1.1). Units 1b and 1c were added on
2026-09-27, on branch `phase1`: 1b at P1.2, verified at P1.3; 1c at P1.4, verified at P1.5
and P1.6. Both were closed at P1.7, the same day.

The oracle is a static equilibrium solver for the pinning paper's economy (PLAN §3.4).
It shares types but not logic with the agents, and no agent may read it (PLAN R13).
It is built outward in units 1a-1f. This package holds three:

- **Unit 1a**: one category, one machine type, one land input, with durability and
  interest through the scalar user cost u = (ρ + δ)(1 + ρ)^(J_b − 1). At
  (ρ, δ, J_b) = (0, 1, 1) it is the SSRN Appendix B economy (SSRN 7226858, pp.28-30).
- **Unit 1b**: many categories bought in a fixed basket (SSRN eq 7), on one task line cut
  into segments, each category a density of tasks on them with its own direct land. Space
  is a category. It reports the fork identity in both its forms (SSRN eq 12, main.tex eq
  composites), the category bounds and the purchasing-power pair for every category, and a
  one-category economy solves to 1a's equilibrium bit for bit. On its own, without an
  equilibrium, it prices categories given as task cells (`cell_cost`), and gives SSRN eq
  26's CES share (`ces_share`).
- **Unit 1c**: many machine types and the Leontief inverse. Each type has an operating
  recipe and a build recipe over machine services, labour and land, its own depreciation and
  build lag, and so its own user cost, at one interest rate (check_dynamics R1-R6); categories
  may use each other as intermediate inputs. The cheapest task type takes the machine tasks
  (SSRN A.1), switching along the line; an equilibrium on a switch is a tie, solved by the
  share of tasks each type takes, and more than one equilibrium is refused
  (`SolveError::MultipleEquilibria`). Every Leontief identity over categories and types, and
  the income identity with interest, hold at every equilibrium, and a one-type economy
  solves to 1b's (and so 1a's) equilibrium bit for bit. The machine block alone, at any
  margin, is `MachineBlock`, which reproduces check_dynamics' steady-state price blocks.

Still to come (PLAN Phase 1): 1d, worker types and the wall; 1e, parcels with quality
schedules, the idle margin and exit as s(q), the default exit form (ADDENDUM ruling 3);
1f, households and government. The 1d and 1e gates are constructed (ADDENDUM §5 item 4).

The package is `rustyecon-oracle`, its library `oracle`, a member of the rustyecon
workspace. Its one dependency is `rustyecon-core`, for `core::num`: the power, `ln1p` and
the fused multiply-add go through the pure-Rust `libm` crate there, so every output is the
same double on every platform (R8, ADDENDUM A5). Nothing on the engine path depends on
the oracle (R13; docs/ENGINE.md §1).

The specifications, with every equation and golden, are [docs/unit-1a.md](docs/unit-1a.md),
[docs/unit-1b.md](docs/unit-1b.md) and [docs/unit-1c.md](docs/unit-1c.md). Unit 1b's open
questions (its §11: the shared task line, cells in the equilibrium, gaps as a regime, bitwise
nesting, viability at the top of the line, the generic `Regime`, the CES parameters) and unit
1c's (its §11: machines built from categories, one capability shape per line, ties inside
`Interior`, refusing multiple equilibria, bitwise nesting, physical productivity, the changes
to 1a's module, the random draws, every type priced) are not ruled. The build takes the
draft's choice on each, and the repository's STATE.md lists them as decisions open to veto.
Each spec's §12 records where its build and its verification departed from the draft.

## The gate

The oracle is green when the workspace's gate is (`scripts/gate.sh`): `cargo test --workspace
--release` passes on WSL and on Windows, and `cargo clippy --workspace --all-targets -- -D
warnings` and `cargo fmt --all --check` are clean, under the workspace's lints and
`clippy.toml`. The three generators' `--check` (below) are run by hand before any commit
that touches a generator or its goldens. Unit 1a's tests cover, per docs/unit-1a.md §6:

- **G1**, the SSRN Appendix B instance: the published figures to 5e-6 (x* 0.86315,
  v 0.54344, Y 7.88061, N_a 1.34338, hours 1.07846 and 0.26492, N·P_s 5.44630), and
  every full-precision value, bracket value and cost-system total to 1e-12 relative;
- **G2**, SSRN Figure 3's caption values;
- **G3**, the automation path γ = η(1 + x), down to η = 1e-20, where x* rounds to 1.0
  and 1 − x* is 4.6e-21;
- **G4**, durability, interest and the build lag, with the ρ = 0 nesting test, the
  labour share and real wage with interest, φ at u = 1 with δ < 1, and an economy at
  the viability edge (D(x*) = 8.4e-7);
- **G5**, 180 random interior economies (the identities to 1e-12, and single crossing on
  a grid), and two general instances with every parameter off the paper's values
  (k = 4.5 and 2.5, h, χ_max, η ≠ 1, J_b = 2) pinned field by field;
- **G6** and **G7**, the replacement closure (c = 1, w = 3; at λ = 0, 0.4 and 1.2) and
  the three-taxes shares (0.6, 0.4);
- **G8**, the four regimes recognised on constructed cases, including an exact f64 zero
  at each boundary, funding without Lemma B.1, `NoInteriorAtZero` with T/h > N, the
  parameter ceilings (k = 1e20 rejected; k = `CURVATURE_CEIL` = 1024 solved against
  70-digit goldens), `funded` and `lemma_b1` false at exact f64 ties, D(1) = −∞ as
  `NotViable`, the non-finite errors, and the labour-residual net refusing a γ that jumps;
- the dump interface: every key it prints equals the solve's field bit for bit, at u = 1,
  with interest, where `funded` and `lemma_b1` differ, and on a line of fourteen distinct
  values checked against a solve built without the parser;
- `goldens.txt` carries digests of `generate.py` and of itself, which the gate checks.

Unit 1b's tests, per docs/unit-1b.md §8, cover the PLAN's "fork identity and category
bounds on random instances", the income identity at 1e-12, and "1a's results as the
one-category case exactly":

- **C1**, nesting: 1a's G1, 27 golden instances, G5's 180 random draws and every skipped
  one, G8's regime rows and rejections, and `at(x)` on a grid, all bit for bit in category
  form; the fork at G1 and along G3's path;
- **C2**, SSRN eq 26's CES share: goldens on G3's path and at q = 1, σ = 1, limits,
  monotonicity, extreme prices, bad arguments;
- **C3**, the fork economy (manufactures, food, care, shelter on three segments) in flow,
  durable and ρ = 0 versions, pinned field by field, with care on its human bound,
  manufactures fully automated, and near full automation, where the hours carry 1 − x*;
- **C4**, task and recursive automation on it: the wage in manufactures rises 27% while
  the wage in shelter falls 96%;
- **C5**, 180 random multi-category economies: every identity, the fork identity in both
  forms, the bounds and the pair for every category, the residuals, the root and single
  crossing;
- **C6**, check_interior.py's price-block batteries on task cells, its parity instance,
  the flat case at four user costs, and cells against the line;
- **C7**, the gap economy, roots on and near its edges, reductions (an unbought category,
  split categories and segments, permutation, rescaling), the regime rows and validation;
- **C8**, `goldens_1b.txt`'s three digests and the Rust constants.

Unit 1c's tests, per docs/unit-1c.md §8, cover the PLAN's "check_dynamics' targets", "1a and
1b as the one-type case, exactly", "the income identity to 1e-12 with interest" and "random
instances satisfying the Leontief identities":

- **m1**, nesting: 1a's G1 and 27 golden instances, 1b's C3, C3d, C3z, C4, gap, near-edge
  and on-edge roots, 1a's G5 and 1b's C5 random draws with every skipped one, 1a's and 1b's
  regime rows and rejections, and `at(x)` on a grid, all bit for bit in one-type form; a
  flow-only type is 1a's flow economy bit for bit at any (ρ, δ, J);
- **m2**, the machine block alone: check_dynamics' sloped and flat targets (the price block
  and the quantities per unit of the good; the JSON's doubles within 2e-15), U1-U6, R1-R6,
  S1-S6 and L1-L4, the closure per type, the Leontief totals, the envelope (with two
  switches in sequence) and the gross services of three types;
- **m3**, check_dynamics' machine in Appendix B's closure, with and without interest: goldens,
  the income identity with interest, and its corners as 1a economies;
- **m4**, three types (loom, engine, power) on 1b's fork economy with intermediate inputs:
  goldens, the automation path, the Leontief identities by multiplication, the fork through
  the chain, and the tie at the switch as N moves across it;
- **m5**, interest selecting the technique, uniqueness at ρ = 0, and three equilibria refused,
  among them an economy with f(1) > 0 whose boundary hides a root and a tie;
- **m6**, 240 random interior economies in four sets (ρ = 0, ρ > 0, with intermediate inputs,
  and a set built to switch mid-line, with ties and multiple equilibria);
- **m7**, M4's regime rows and exact zeros at each end of a technique's region, every
  validation rule, an unused type (and one that cannot be priced, which is `NotViable`), a
  duplicate type, permutation, unit rescaling (bit for bit at c = 4), the envelope's edge
  cases, two switches with a tie at the second, and φ where the technique is not type 0 and
  where u = 1 with interest;
- **m8**, `goldens_1c.txt`'s four digests and the Rust constants.

The goldens are pinned to laborformal `31b3482`.

## Verification

Each unit was checked after its build by an adversarial pass, an independent derivation
that does not read the crate and mutation testing, then one fix round, each fix with a test
that fails without it. Where the pass found nothing wrong in the code, the fix is a test.

- **1b** (P1.3): the build's own mutation check caught 44 of 45 mutants; the survivor
  reassociates p_j = v·H_j + (p_m·M_j + b_j), which changes only the rounding. The pass found
  four more survivors, now caught: the carried 1 − x* in a category's final hours and λ̃_j^q,
  the edge convention of `margin_active`, and `Requirement::OpenUnit` admitting 0. It also
  found that outputs made of the sliver between x* and an interior edge of the task line lose
  precision as 2^-53·x*/d; that is stated with its bound (docs/unit-1b.md §5.4) and measured
  by 24 near-edge goldens, and no code changed.
- **1c** (P1.5): an independent derivation in another formulation agreed with the oracle on
  348 economies, every regime, technique, tie and switch count, and the values within
  5.6e-14. Of 65 mutants six survived; five are now killed and one is equivalent in exact
  arithmetic.
- **1c, second pass** (P1.6): a second derivation found a blocker. With an upward jump of
  labour demand at a switch and f(1) > 0, the build returned `BoundaryNoMargin` where a root
  and a tie lay below; such an economy is now `MultipleEquilibria`, with the boundary
  counted as one. It measured the precision at ties on 432 of them (docs/unit-1c.md §5.6),
  and found that an unused type whose price diverges makes the economy `NotViable`, which
  is kept and recorded as a departure from SSRN A.1 (§12 item 15, open question 9). Nine
  mutants that survived the gate as it was are now killed, and six of the eight made of the
  fix; three survivors are equivalent on every economy the gate can build (§12 items 12
  and 17).

Each spec's §12 has the details.

## Layout

| path | what it is |
|---|---|
| `src/params.rs` | `Params`, validation (`ParamError`), `Economy`, the user cost u |
| `src/schedule.rs` | the `Schedule` trait (γ and its integral J) and `PowerSchedule`, γ = η(g0 + g1·x^k) |
| `src/solve.rs` | `Economy::at(x)`, `Economy::solve`, `Regime<E>`, `Eq1a`, residuals, the cost-system view, and the regime tests and bisection both units share |
| `src/closure.rs` | `closure(a, λ, γ*, b, r, u)`, the price block alone |
| `src/categories.rs` | unit 1b: `Category`, `CategoryParams`, `CategoryEconomy`, `Eq1b` and its outputs |
| `src/fork.rs` | unit 1b's price block alone: `Cell`, `cell_cost`, `ces_share` |
| `src/leontief.rs` | Gaussian elimination without pivoting, in index order, on unit 1c's M-matrices (crate-private) |
| `src/machine_block.rs` | unit 1c's machine block alone: `Recipe`, `MachineType`, `MachineBlock` (totals, closure, envelope, gross services) |
| `src/machines.rs` | unit 1c: `MachineParams`, `MachineEconomy`, `Eq1c` and its outputs, ties and the sign-change count |
| `src/dump.rs` | the one-line text interface behind `examples/dump.rs` |
| `examples/dump.rs` | reads economies on stdin, writes one result line each |
| `tests/gate/` | the gate: one test crate, one module per golden group (1a's `g*`, 1b's `c*`, 1c's `m*`) |
| `goldens/generate.py` | computes every golden with mpmath at 70 digits |
| `goldens/goldens.txt` | its output, 30 significant digits |
| `goldens/generate_1b.py` | unit 1b's goldens, at 70 digits; imports `generate.py` to assert the nesting |
| `goldens/goldens_1b.txt` | its output, 273 goldens |
| `goldens/generate_1c.py` | unit 1c's goldens, at 70 digits; imports `generate_1b.py` (and so `generate.py`) to assert the nesting |
| `goldens/goldens_1c.txt` | its output, 210 goldens |

## Running the tests

Keep the build directory outside the repository, as the workspace's gate does, and
building on a Windows drive from WSL is slow. The whole gate is `scripts/gate.sh` (see the
root README). For this package alone, from the repository root, on the primary platform,
WSL Ubuntu, and on the secondary one, Windows with the MSVC toolchain (both Rust 1.97.1,
pinned by `rust-toolchain.toml`):

```sh
export CARGO_TARGET_DIR=<a directory outside the repository>
cargo test --release -p rustyecon-oracle
cargo clippy -p rustyecon-oracle --all-targets -- -D warnings
cargo fmt --all --check
```

In PowerShell, set `$env:CARGO_TARGET_DIR` instead. To drive WSL from Windows, use
`wsl -d ubuntu --exec bash -lc '…'`; `wsl -- …` loses exit codes.

The counts, the same on WSL and on Windows at each step:

| Step | Date | Unit | Gate | Doc | Total |
|---|---|---|---|---|---|
| unit 1a, after its final verification round | 2026-09-25 | 42 | 71 | 1 | 114 |
| P1.1, 1a in the workspace | 2026-09-26 | 42 | 71 | 1 | 114 |
| P1.2, unit 1b | 2026-09-27 | 46 | 122 | 1 | 169 |
| P1.3, 1b's verification | 2026-09-27 | 47 | 125 | 1 | 173 |
| P1.4, unit 1c | 2026-09-27 | 56 | 175 | 1 | 232 |
| P1.5, 1c's verification | 2026-09-27 | 57 | 177 | 1 | 235 |
| P1.6, 1c's second verification | 2026-09-27 | 57 | 184 | 1 | 242 |
| P1.7, units 1b and 1c closed | 2026-09-27 | 57 | 184 | 1 | 242 |

Of the 242, 1a has 114, 1b 59 (5 unit, 54 gate) and 1c 69 (10 unit, 59 gate).

## The dump example

`examples/dump.rs` is for differential testing against other solvers. Each input line
is whitespace-separated `key=value` pairs, one for each of `workers land space a lam b
eta g0 g1 k chi_max rho delta build_lag`. Each output line starts with `regime=<name>`,
then every output of `Eq1a::outputs` as `key=value` (every `Eq1a` field, including
`one_minus_x_star`, and each residual), or the non-interior regime's diagnostic. Floats
use Rust's `{:?}` formatting: the shortest digits that parse back to the same double,
always with a decimal point or an exponent (`1.0`, `-6.0005e-12`, `1e30`). A bad line,
including one that is not UTF-8, gives `error=<message>`. Every input line gives exactly
one output line. The loop is `oracle::dump::run`, which the unit tests drive on raw
bytes; if stdin cannot be read or stdout cannot be written or flushed, the example says
so on stderr and exits with status 1.

```sh
echo "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1" \
  | cargo run --release -p rustyecon-oracle --example dump
```

This prints `regime=Interior x_star=0.863150418162437 one_minus_x_star=0.136849581837563 …
u=1.0 … v=0.5434359606967785 … y=7.880605524972908 … n_a=1.3433818800977175 …`, which
is G1.

On 2026-09-25, 2001 random economies through the dump example agreed with laborformal's
`macro.py` on every regime and on every interior value within 9e-14 relative. That run
had J_b = 1 only and pooled `BoundaryNoMargin` with `NoInteriorAtZero`. The review of the
same day compared all four regimes on 3616 economies with J_b from 1 to 12, against a
macro.py patched for u; it found no disagreement except where the true |f| at a bracket
end is at most 3.8e-16, where the regime is not decidable in f64 (docs/unit-1a.md §4).
The largest value gaps are where 1 − x* is small: there both macro.py's brentq xtol and
the f64 conditioning of 1 − x* matter.

The dump is unit 1a's only. Units 1b and 1c have none, since there is no other
multi-category or multi-type solver to compare against (docs/unit-1b.md §9, docs/unit-1c.md
§9); their independent checks were the verifications' derivations.

## Regenerating the goldens

`goldens/generate.py` needs a Python with mpmath (1.3.0 was used; laborformal's analysis
venv has it, WSL's python3 does not). From `crates/oracle`:

```sh
python goldens/generate.py           # writes goldens/goldens.txt
python goldens/generate.py --check   # exits 1 if goldens.txt is not what it writes
```

On Windows, set `PYTHONIOENCODING=utf-8` first. The generator works at 70 digits and
asserts the published figures, the identities at 65 digits, the ρ = 0 nesting and each G8
regime as it goes. The constants in `tests/gate/goldens.rs` are `goldens.txt` rounded to
20 significant digits, and the gate test `goldens_file::constants_match_goldens_txt`
fails if the two disagree. After a change, update those constants by hand, with a
provenance comment each.

`goldens.txt`'s header records FNV-1a digests of `generate.py` and of the goldens below
it, and `goldens_file::goldens_txt_is_from_generate_py` recomputes both. So the gate,
even where mpmath is missing, fails if `goldens.txt` was edited by hand or not rewritten
after `generate.py` changed. It cannot prove that the values are what `generate.py`
computes: **run `generate.py --check` before committing any change to either file.**

Unit 1b's goldens work the same way:

```sh
python goldens/generate_1b.py           # writes goldens/goldens_1b.txt
python goldens/generate_1b.py --check   # exits 1 if goldens_1b.txt is not what it writes
```

`generate_1b.py` imports `generate.py` to assert that the category form of six 1a
instances equals 1a's solve, so `goldens_1b.txt` records the digests of both generators
and of its own goldens, and `c8_goldens_file` recomputes all three: a change to
`generate.py` means rerunning both. The constants in `tests/gate/goldens_1b.rs` are
`goldens_1b.txt` rounded to 20 significant digits (`c8_goldens_file::
constants_match_goldens_1b_txt`). On 2026-09-27 the oracle's f64 values matched all 245
numeric goldens of unit 1b away from an interior edge within 1.0e-15 relative. Of the 24
near-edge goldens, the three N are inputs, the 15 full-precision outputs match within
1.9e-16, and the six outputs made of the sliver by the edge miss by up to 7.5e-8, within
the bound of docs/unit-1b.md §5.4.

Unit 1c's goldens work the same way, with four digests (`generate_1c.py`, `generate_1b.py`,
`generate.py` and its own goldens), checked by `m8_goldens_file`:

```sh
python goldens/generate_1c.py           # writes goldens/goldens_1c.txt
python goldens/generate_1c.py --check   # exits 1 if goldens_1c.txt is not what it writes
```

It asserts as it goes that the one-type form of 1b's instances equals `generate_1b.py`'s solve
and M3's corners equal `generate.py`'s (within 6.1e-71), that check_dynamics' targets match the
JSON's doubles within 2e-15, every identity of docs/unit-1c.md §4 at 1e-65, the envelope
against a scan of 10^4 points, and f nonincreasing in every technique region. On 2026-09-27
the oracle's f64 values matched 191 of the 207 goldens it computes within 1.2e-15 relative
and 199 within 1e-14; the rest are where docs/unit-1c.md §5.6 says precision goes: the switch
points (up to 2.8e-14, from the cancellation in γ_i's closed form), a least pivot near 0
(1.7e-14), and the excess demand at M5m's switch, evaluated at the double below it where f is
steep (4.8e-13).

At the close (P1.7) all three `--check` passed, and the gate's golden comparisons were logged
again on WSL: the largest errors are as above, 1.0e-15 on 1b's goldens away from an edge (the
cells of `c6::cells_approach_the_line` are compared with the line's closed forms at the
midpoint rule's error, up to 3.5e-9, by design) and 4.8e-13 on 1c's, then 4.4e-13 on M5m's
tie share and 2.8e-14 on the switch points.

## Numerics

- The root is found by bisection on [1e-12, 1] until lo and hi are adjacent doubles.
  There is no tolerance. `BRACKET_LO` and `MAX_BISECTION_STEPS` are named constants
  with their reasons in `src/solve.rs`.
- u and the machine-wealth factor are computed by repeated squaring in this crate, so
  they are identical on every platform. u's error grows with the build lag, to about
  J_b·2.2e-16 relative (the `user_cost` doc).
- Every scale parameter (N, T, h, b, χ_max, and η, g0, g1) must lie in
  [`SCALE_FLOOR`, `SCALE_CEIL`] = [1e-30, 1e30], δ in [1e-30, 1], and λ and ρ in
  [0, 1e30]. Within these bounds the products and quotients the regime tests compare stay
  far from underflow and overflow. u and D are not bounded: a huge u can still overflow
  a price, and the solve then returns `SolveError::NonFinite`, never a wrong regime.
  D(1) = −∞ is `NotViable`.
- k must lie in [1e-30, `CURVATURE_CEIL`] = [1e-30, 1024]. Across one double of x, γ moves
  by less than max(k, 1)·2^-52 relative, 2.3e-13 at the ceiling, so γ stays resolved by
  the doubles x* is chosen from. At k = 1e20 it jumped from g0 to g0 + g1 across the last
  double below 1, and the solve returned a wrong `Interior`.
- An `Interior` result must clear the labour market: |N_a − n_S(x*)| (`res_labor`) at most
  `LABOR_RESIDUAL_NET` = 1e-9 of N_a, or the solve returns `SolveError::LaborNotCleared`.
  This is a safety net, not a tolerance. A root leaves at most about 1e-13 away from the
  viability edge; a jump in n_D − n_S leaves the jump. It trips on a schedule that breaks
  the continuity contract, and within about 5e-5 of the viability edge where the root is
  not resolved in f64 (docs/unit-1a.md §4 step 5).
- `funded` is `provider_baskets > 0`, so the two never disagree in f64.
- A negative zero in a, λ or ρ is stored as +0.0, so the same economy prints the same.
- At an exact f64 zero, the regime follows the spec's convention: D(1) = 0 is
  `NotViable`, f(1) = 0 `BoundaryNoMargin`, f(1e-12) = 0 `NoInteriorAtZero`. Within a
  few ulps of a boundary the regime is not decidable in f64. `NoInteriorAtZero` means
  f(1e-12) ≤ 0; a root can still lie in (0, 1e-12). The band where the regime is not
  decidable widens near the viability edge and with the build lag: u's error (J_b·2.2e-16)
  makes D(1) uncertain by about that much, and the prices carry it into f amplified by
  u(a + λγ)/D (docs/unit-1a.md §4 step 3).
- Near the viability edge the prices scale as 1/D, and x*'s representation reaches them
  amplified: at the double x*, the outputs are within about max(k, 1)·2^-53·(1 + uλγ/D)
  relative of the exact equilibrium, besides D's own rounding, 2^-53·(1 − u·a)/D. An
  `Interior` result within about 1e-5 of the edge (D(x*) ≲ 1e-5) can carry a labour
  residual up to the 1e-9 net (docs/unit-1a.md §4 step 4).
- `x_star` is a double in [1e-12, 1]; it is exactly 1.0 when the root is within half an
  ulp of 1. 1 − x* is carried separately, as `one_minus_x_star`, interpolated between
  the two doubles that bracket the root, and every output proportional to 1 − x*
  (final hours, N_a, participation, the labour share) uses it, so they keep full relative
  precision however close x* is to 1. In unit 1c this holds for a root, not for a tie at a
  switch near 1 (below).
- In unit 1b only the top segment carries its offset. Within a distance d of an interior
  edge of the task line, an output made mostly of the sliver between x* and the edge is
  good to about 2^-53·x*/d relative (hours) or 2^-53·(x* + 2J(x*)/γ(x*))/d (machine
  services, and K, M_s and interest when all machine use is in the sliver): 2.1e-8 on a
  category's hours at d = 1e-9, 1.9e-11 on K at d = 1e-6. Prices, v, x*, P_s, Y and N_a
  keep full precision. Interpolating an offset from each edge would not recover it
  (docs/unit-1b.md §5.4).
- D = 1 − u(a + λγ) is computed as (1 − u·a) − u·λγ with a fused multiply-add
  (`core::num::fma`, correctly rounded), and 1 − aδ likewise, so neither loses precision
  near the viability edge or as aδ → 1.
- Interest is computed as ρ·W_K, not (u − δ)·V_m·K, which cancels when ρ is small.
- Unit 1c's Leontief systems (the categories' I − A_cc, the machine block's I − Â and
  I − A^q, and the (O, V) system at each x) are solved by Gaussian elimination without
  pivoting, in index order. On these M-matrices it is stable, its pivots being positive is
  the productivity or viability test itself (the least pivot generalises 1a's D), and its
  fixed order makes one type repeat 1a's and 1b's operations bit for bit
  (`src/leontief.rs`; docs/unit-1c.md §5.1).
- A switch of the cheapest type is a closed-form γ_i; its point on the line is the largest
  double with γ(x) < γ_i, found by bisection, so γ is never inverted. γ_i carries the totals'
  rounding amplified by the cancellation in its numerator, up to 2.8e-14 on M5's switch.
  x_i is good in absolute terms: near x = 0 its relative error grows (1.6e-12 at 0.0042).
- A tie is evaluated at x_i, so every output carries γ_i's error through γ*: up to about
  2e-13 relative on x*, v and the prices on the second verification's 432 ties (7.5e-13
  near the viability edge). The split σ, and each type's services, builds, hours, land and
  wealth, carry it amplified by |f'|·x*/|jump| (the excess demand's slope over the jump of
  labour demand at the switch): up to 1.7e-10 there. A tie near x = 1 sets 1 − x* = 1.0 − x_i
  without interpolation, so 1 − x* and the outputs proportional to it are good only to about
  2^-53/(1 − x*) relative plus γ_i's error over γ'·(1 − x*): 3.2e-7 at a tie 1e-9 below 1
  (docs/unit-1c.md §5.6).
- With a switch of technique, a boundary regime is returned only when the excess demand
  changes side nowhere inside the bracket: f(1) ≥ 0 with a root or a tie below is
  `SolveError::MultipleEquilibria`, the boundary counted as one of them (docs/unit-1c.md
  §5.3). In unit 1c `NotViable` also covers a type no technique would use whose price
  recursion diverges (docs/unit-1c.md §12 item 15).
- The power x^k, ln(1 + z) and the fused multiply-add come from `core::num`, that is from
  the `libm` crate, so the outputs do not depend on the platform (docs/unit-1a.md §8).
  On 2026-09-26, 5000 random economies (every regime, J_b up to 12) gave byte-identical
  dump output on WSL and Windows. Before P1.1 they came from the platform libm, where glibc
  and MSVC differ in the last bits: the two platforms' outputs differed by up to 2.8e-14
  relative on the review's set, and by 1.1e-13 on x* (at x* = 9.7e-10, where the root is
  ill-conditioned) on the 5000. The move to libm changed no regime and no flag on that
  set, moved the interior outputs (residuals aside) by at most 7.0e-16 relative from the
  glibc build, and left the goldens' error maxima as they were (docs/unit-1a.md §6); the
  exact-tie input of G8 still ties. Cross-platform equality is recorded, not gated
  (ADDENDUM ruling 2).
