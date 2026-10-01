"""P2.4 (label run, 2026-10-01): the dial-neighbourhood map across the session's instances, from
the two waves' gathered runs (wave A's and the B-D waves'). Not a scorer: it tabulates classes the
scorers already scored, per instance and dial setting, and writes dialmap.csv and a printout.

For each instance and each of the 17 settings (every price rate, buffer and technique rate x 0.75,
0.9, 1.1, 1.25; every tilt at 0.05, 0.1, 0.25, 0.5, 1), Tier 3 at that setting (and Tier 3S where
wave A ran it, at C1, C2 and IW1): the count of each class and the runs that are neither
CONVERGED nor VACUOUS, by name. A setting holds where every run is CONVERGED or VACUOUS: the
verdict battery's Tier-3 criterion at that setting (MARKETS-SPEC §7.9). Beside them: IS1's and
IS2's switch rate alone (`rate.switch.*`, four settings), C1P's and C2P's Tier 3 at every tilt 2
(the trap's controls beyond the edge), and wave A's base kick set at each setting (reported there).
C1 is wave A's run; the trap's wave ran the same 731 runs again (E9), byte for byte (r1check.py).

Usage (WSL): python3 dialmap.py WAVE_A_RUNS.jsonl[.gz] BCD_RUNS.jsonl[.gz] OUT.csv"""
import csv
import gzip
import json
import sys
from collections import OrderedDict, defaultdict

SETTINGS = ([f"rate.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"buffer.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"adjust.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"tilt.*={v}" for v in ("0.05", "0.1", "0.25", "0.5", "1")])
EXTRA = [f"rate.switch.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")] + ["tilt.*=2"]
ORDER = ["iw1", "is1", "is2", "c1", "c1p", "c2", "c2p", "il1", "ct2"]
WHO = {"iw1": "wave A", "c1": "wave A", "c2": "wave A", "c1p": "trap E10, E9", "c2p": "trap E10, E9",
       "is1": "switch E7", "is2": "switch E8 (control)", "il1": "free E4", "ct2": "free E4"}


def records(path):
    op = gzip.open if path.endswith(".gz") else open
    with op(path, "rt") as f:
        for line in f:
            if line.strip():
                yield json.loads(line)


def main(a_path, bcd_path, out_path):
    cells = defaultdict(lambda: {"t3": defaultdict(int), "t3s": defaultdict(int), "bad": [], "kick": None})
    for r in records(a_path):
        if r.get("set") == "dial" and r.get("cmd") == "run":
            c = cells[(r["inst"], r["setting"])]
            k = "t3s" if r.get("first_year") else "t3"
            cls = r.get("class", "missing")
            c[k][cls] += 1
            if cls not in ("CONVERGED", "VACUOUS"):
                c["bad"].append(f"{r['names'][0]}{' (3S)' if r.get('first_year') else ''}: {cls}")
        elif r.get("set") == "dial_kick":
            cells[(r["inst"], r["setting"])]["kick"] = r.get("passed")
    for r in records(bcd_path):
        s = r.get("set")
        if r.get("cmd") != "run" or s not in ("dial", "nbhd", "swrate", "ctl-tilt2"):
            continue
        c = cells[(r["inst"], r["setting"])]
        cls = r.get("class", "missing")
        c["t3"][cls] += 1
        if cls not in ("CONVERGED", "VACUOUS"):
            c["bad"].append(f"{r['names'][0]}: {cls}")
    rows = []
    for inst in ORDER:
        for s in SETTINGS + EXTRA:
            if (inst, s) not in cells:
                continue
            c = cells[(inst, s)]
            n3s = sum(c["t3s"].values())
            rows.append(OrderedDict(
                inst=inst, source=WHO[inst], setting=s,
                tier3=json.dumps(dict(sorted(c["t3"].items()))), tier3_runs=sum(c["t3"].values()),
                tier3s=json.dumps(dict(sorted(c["t3s"].items()))) if n3s else "-",
                holds="yes" if not c["bad"] else "no", not_converged="; ".join(sorted(c["bad"])) or "-",
                base_kick="-" if c["kick"] is None else ("PASS" if c["kick"] else "FAIL")))
    with open(out_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]), lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
    print("Tier 3 (and 3S at C1, C2, IW1) at each setting: ok = every run CONVERGED or VACUOUS, "
          "else the count of runs that are not")
    print(f"{'':6}" + "".join(f"{s.split('.')[0][:3]}{s.split('=')[1]:>5} " for s in SETTINGS))
    for inst in ORDER:
        line = []
        for s in SETTINGS:
            c = cells.get((inst, s))
            line.append("." if c is None else ("ok" if not c["bad"] else str(len(c["bad"]))))
        print(f"{inst:6}" + "".join(f"{x:>8} " for x in line))
    for inst in ("is1", "is2"):
        cs = [cells.get((inst, s)) for s in EXTRA[:4]]
        print(f"{inst} rate.switch.* x0.75, 0.9, 1.1, 1.25: " + " ".join(
            "not run" if c is None else ("ok" if not c["bad"] else str(len(c["bad"]))) for c in cs))
    for inst in ("c1p", "c2p"):
        c = cells.get((inst, "tilt.*=2"))
        print(f"{inst} tilt.*=2: {json.dumps(dict(sorted(c['t3'].items())))}; " + ("; ".join(c["bad"]) or "-"))


if __name__ == "__main__":
    main(*sys.argv[1:4])
