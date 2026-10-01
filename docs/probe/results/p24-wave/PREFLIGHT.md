# A load check of every job, after P2.4.14 and before the B–D waves

Dated 2026-09-30, label `run`. As wave A's (`../families-wave/PREFLIGHT.md`): P2.3's wave lost 54
jobs to a tape no test had loaded and 703 to the runner's quoting. So after committing this
directory at P2.4.14 (`2b68736`), and before any job of the three waves, every line of `jobs.txt`
was run once with its lengths cut to 8 ticks: `--ticks 8` (a dated shock then fires at tick 2),
`--horizon 8` for the kick sets, `--every 1`, into `/root/scratch/p24-bcd-pf/runs/`, three at a
time beside wave A, through `xargs -d '\n'` and a copy of `job.sh`, on the wave's binary.

- **The binary.** Built in WSL release from a `git archive` export of `2b68736` in its own target
  directory (`/root/scratch/p24-exportB-target`), sha256
  `aa97c626d6e266c9241f2ceb52b0f33b43352247cb6ac7c1f57ebd63f30488d4`: the free E0's binary, byte
  for byte. `make_jobs.py` with `MARKETS=` that binary writes the committed `jobs.txt` byte for
  byte.
- **All 10,194 jobs exited 0**: every run name parsed, every tape loaded (the dated shocks, the
  `cycle` histories, every `--set` of the 17 dial settings, `rate.switch.*`, `tilt.*=2`,
  `--one-sided hold`, `--first-year`, the `+`-joined envelope runs at tail.services 0.11), and every
  kick set, elasticity probe and point ran.
- **`gather.py` read all of them**, 10,196 records (the jobs and the two envelope records) with no
  missing file and no gather error, each where its scorer looks for it; the three scorers ran on
  those records to their end without an exception. Their lines at 8 ticks are no result and were
  not read, but for one: the point jobs run no tick and so ran in full, and while checking that
  the point's fields parse, IS1's tail.services 0.11 line was printed (a\* 0.007031052173720907,
  switch_mp's 0.007031052173720732). No other number of the preflight was read or kept.

The preflight's directory stays in scratch until the wave's gather, then is deleted. No file of
this directory changed but this one.
