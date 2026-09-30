# Fixes to the wave's machinery after its commit (decision 311)

Dated 2026-09-30, label `run`. `SHA256SUMS` records the files as committed before the wave
(P2.3.11). Each later fix is listed here with its sha256s; none changes a scored line.

| step | file | what | sha256 before | sha256 after |
|---|---|---|---|---|
| P2.3.14 | `score.py` | the commons' `verdicts.csv` counted the kick sets passed by run name where the records are keyed by file name, so its `kicks` column read 1/13 at C1 and C2; the E2 and E3 kick lines and the verdict used the file name and read 13/13. Scored again on the same `runs.jsonl`: `lines.csv` and every other table are byte for byte the same; only that column changes, to 13/13 | `75b66dffef92f1537d314dfa267bd4cc6fb361a06674cfb3450c32eddc2d3851` | `a30a2d462bd73100707a9dcde874b32a757b911bc893bb02289fc683b2f98486` |
| P2.3.16 | `score.py` | label `fix-report`, after the wave and its reviews, disclosed: a `--amended` flag reads the wall's A2 (E6's end share against its own tick length's stall, 2, 10 or 70 subnormal ulps; `stall`) and the commons' A3 (runaway ticks against the mirror on the harness's reference, from `../commons/diag/runaway_ref.out`; r_o at a Crowded end against the oracle's `ro_star`). Without the flag: `lines.csv` (sha256 `ef87b114…b9`), all 16 tables and the printout are byte for byte as committed. With it (`score-amended.out`): 1,183 lines change their registered value or band, 124 of them from fail to pass, none from pass to fail; 0 lines fail; IW1, C1 and C2 GO; no refutation | `a30a2d462bd73100707a9dcde874b32a757b911bc893bb02289fc683b2f98486` | `49d17e8dd4028667342c2cbd196bf17a801ecd397a2ba3aeb15e33ebd430addc` |

The job runner's quoting and the dated-commons fix are in [rerun/](rerun/README.md) (P2.3.12,
P2.3.13); they changed no file of this directory.

## Files added after the wave (P2.3.15)

- `BIN.sha256`: the two binaries' sha256 and the wave's `runs.jsonl`'s.
- `score.out`: `score.py`'s printout on `runs.jsonl` (as fixed at P2.3.14).
- `lines.csv.gz`: every line the scorer read, 59,788 (step, instance, set, run, what, registered,
  engine, band, status).
- `plots.py`: the small plots in `docs/probe/figs/wall/` and `docs/probe/figs/commons/`, from the
  tables and the archived runs. Not a scorer; written after the scoring.

## Files added after the reviews (P2.3.16)

- `score-amended.out`: `score.py --amended`'s printout on the same `runs.jsonl` (sha256
  `f9fc7416…ea79`, from `D:/rustyecon-p23/runs/runs.jsonl.gz`). The re-run of both readings, and
  the line-by-line comparison, are `D:/rustyecon-p23/fix-report/rescore.sh` and `rescore/`.

**A build hazard the measurement review found** (not a fault of this wave: both binaries equal
fresh builds). Cargo keys a workspace member by its path relative to the workspace root, so a
target directory shared by two checkouts can reuse the other's artifacts when the sources' mtimes
are older than the outputs, as after `git archive` or checking out an older commit; the binary
or test is then silently stale. On Windows a clone's `commons_dated_shocks_load_and_fire` failed
with the pre-fix error in a target shared with a `198554a` export, and passed in a fresh one. So:
one `CARGO_TARGET_DIR` per checkout and commit for any binary a scored run or a gate uses, and
each scored binary's sha256 recorded against a fresh build, as `BIN.sha256` does.

**Added at P2.3.17 (the recheck).** `selftest.py` feeds the registered mirrors' records to the
scorer as if they were the engine's. Without `--amended` it passes as the README says; with
`--amended` 118 lines fail by construction: the mirror's own undisplaced runaway ticks, and its
early-stopped r_o set against the oracle's. So a later wave that reuses this registration under
A3 has no passing self-test until `selftest.py` reads `diag/runaway_ref.out` and the oracle's r_o,
or the mirror is run to L. A3's runaway reading also leaves seven lines as registered (three
tilt-1 runs, two C1N dated runs and C1N's two genesis parameter shocks); each passes with the
engine's tick equal to the registered one.
