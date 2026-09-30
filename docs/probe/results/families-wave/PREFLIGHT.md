# A load check of every job, after P2.4.2 and before the wave

Dated 2026-09-30, label `families`. P2.3's wave lost 54 jobs to a tape that no test had loaded
(P2.3.12) and 703 to the runner's quoting (P2.3.13). So after committing this directory, and
before the wave, every line of `jobs.txt` was run once with its lengths cut to 8 ticks: `--ticks
8` (a dated shock then fires at tick 2), `--horizon 8` for the kick sets, `--every 1`, into
`/root/scratch/p24-families-preflight/`, through `run.sh`'s `xargs -d '\n' -P 46` and a copy of
`job.sh` naming the `a483ed0` build (`/root/scratch/target-p24/release/markets`, sha256
`be3266ab…42e9`).

- All 6,443 jobs exited 0: every run name parsed, every tape loaded (the dated shocks, the
  `cycle` histories' 80 dated changes, every `--set` of the 17 dial settings and the five map
  cells, `--one-sided hold`, `--first-year`), and every kick set ran.
- `gather.py` read all 6,443 directories with no missing file and no gather error, each set
  where `score.py` looks for it (stocks, joint2, joint4, basin, history, hold, tilt1, map,
  map_modea, map_kick, dial, dial_kick).

Nothing at 8 ticks is a result: no class, tick to tolerance or kick of these runs was read or
kept, and the directory was deleted. No file of this directory changed.
