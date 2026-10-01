"""run (2026-10-01): every job of the committed B-D list ran exactly once, under its own name, in
the wave's window, and runs.jsonl holds one record a job plus the two envelope records."""
import datetime as dt
import json
import os
import shlex
import sys

JOBS = "/mnt/d/rustyecon-wt/p24/docs/probe/results/p24-wave/jobs.txt"
ROOT = "/root/scratch/p24-bcd-runs"
W = "/root/scratch/p24-bcd"


def ts(s):
    return dt.datetime.strptime(s.strip(), "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc).timestamp()


start, end = ts(open(f"{W}/start.txt").read()), ts(open(f"{W}/end.txt").read())
lines = [l for l in open(JOBS).read().split("\n") if l.strip()]
dirs, names, cmds = [], [], []
for l in lines:
    a = shlex.split(l)
    assert a[0].endswith("/job.sh"), a[0]
    dirs.append(a[1]); cmds.append(a[2])
    rest = a[3:]
    nm, it = [], iter(rest)
    for x in it:
        if x in ("--inst", "--ticks", "--csv", "--every", "--horizon", "--set", "--one-sided", "--tpy"):
            next(it)
        elif x.startswith("--"):
            pass
        else:
            nm.append(x)
    names.append(nm)
print(f"job lines {len(lines)}; distinct dirs {len(set(dirs))}; distinct lines {len(set(lines))}")
# every directory holding an exit file, and every directory holding any file
exits, leaf = set(), set()
for d, sub, files in os.walk(ROOT):
    if files:
        leaf.add(d)
    if "exit" in files:
        exits.add(d)
js = set(dirs)
print(f"dirs with an exit file {len(exits)}; dirs with any file {len(leaf)}")
print(f"jobs without an exit file {len(js - exits)}; exit files of no job {len(exits - js)}; "
      f"dirs with files but no exit {len(leaf - exits)}")
bad_rc, early, late, mism, unread = [], [], [], [], []
mt = []
for d, c, nm in zip(dirs, cmds, names):
    p = os.path.join(d, "exit")
    rc, secs = open(p).read().split()
    if rc != "0":
        bad_rc.append(d)
    m = os.path.getmtime(p)
    mt.append(m)
    # the exit file is written once the job ends; its start is m - secs
    if m - float(secs) < start - 1:
        early.append(d)
    if m > end + 1:
        late.append(d)
    # the run's own name in its output
    if c == "run":
        s = open(os.path.join(d, "summary.tsv")).read().splitlines()
        got = s[1].split("\t")[0] if len(s) > 1 else None
        if got != nm[0]:
            mism.append((d, nm[0], got))
    elif c == "kick":
        o = open(os.path.join(d, "out.txt")).read().splitlines()
        got = o[1].split("\t")[0] if len(o) > 1 else None
        if got != nm[0]:
            mism.append((d, nm[0], got))
    else:
        if not os.path.getsize(os.path.join(d, "out.txt")):
            unread.append(d)
print(f"nonzero exits {len(bad_rc)}; started before the wave {len(early)}; ended after it {len(late)}")
print(f"exit files written {dt.datetime.fromtimestamp(min(mt), dt.timezone.utc):%FT%TZ} .. "
      f"{dt.datetime.fromtimestamp(max(mt), dt.timezone.utc):%FT%TZ}; wave {open(f'{W}/start.txt').read().strip()} .. {open(f'{W}/end.txt').read().strip()}")
print(f"runs and kick sets whose output names another run {len(mism)}; other jobs with an empty out.txt {len(unread)}")
for x in mism[:10]:
    print("  ", x)
print("commands:", {c: cmds.count(c) for c in sorted(set(cmds))})
# runs.jsonl
recs = [json.loads(l) for l in open(f"{W}/runs.jsonl")]
rels = [r["rel"] for r in recs]
want = [os.path.relpath(d, ROOT) for d in dirs]
env = [r for r in recs if r.get("cmd") == "envelope"]
print(f"runs.jsonl {len(recs)} records; the first {len(want)} are the jobs in the list's order: "
      f"{rels[:len(want)] == want}; the rest: {[(r['rel'], r['cmd']) for r in recs[len(want):]]}")
print(f"records with a nonzero rc, a missing file or a gather error: "
      f"{sum(1 for r in recs if r.get('rc') != 0 or 'missing' in r or 'gather_error' in r)}")
