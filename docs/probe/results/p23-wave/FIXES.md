# Fixes to the wave's machinery after its commit (decision 311)

Dated 2026-09-30, label `run`. `SHA256SUMS` records the files as committed before the wave
(P2.3.11). Each later fix is listed here with its sha256s; none changes a scored line.

| step | file | what | sha256 before | sha256 after |
|---|---|---|---|---|
| P2.3.14 | `score.py` | the commons' `verdicts.csv` counted the kick sets passed by run name where the records are keyed by file name, so its `kicks` column read 1/13 at C1 and C2; the E2 and E3 kick lines and the verdict used the file name and read 13/13. Scored again on the same `runs.jsonl`: `lines.csv` and every other table are byte for byte the same; only that column changes, to 13/13 | `75b66dffef92f1537d314dfa267bd4cc6fb361a06674cfb3450c32eddc2d3851` | `a30a2d462bd73100707a9dcde874b32a757b911bc893bb02289fc683b2f98486` |

The job runner's quoting and the dated-commons fix are in [rerun/](rerun/README.md) (P2.3.12,
P2.3.13); they changed no file of this directory.
