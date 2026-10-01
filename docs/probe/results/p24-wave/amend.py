"""P2.4 (label run, 2026-10-01): the B-D waves' failed lines re-read under the dated amendments
written AFTER the waves were scored: the switch's A1 (../switch/registration-A1.md) and the free
step's A2 (../free/registration-A2.md). Not a scorer and not a change to one: the committed scorers
(SHA256SUMS) and their outputs stand as the record. This reads the same runs.jsonl, the scorers'
lines.csv (committed gzipped beside each registration) and, for A2's third part, each run's CSV
from the archive, and writes ../<key>/amended.csv, every line an amendment re-reads, with its
status as scored and on the amendment, and a printout with the tallies and the refutation
criteria read again.

- Switch A1 (E5): a pooled pop's end share read at the run's CSV's last row (its last tick,
  every double as the harness writes it), not at stats.tsv's `switch.end`, printed to 7
  significant digits; the band stays 1e-10 relative to a*. The walled lines are as scored (the
  README's reading 4, declared before the wave).
- Free A2 (a): the regime at the end against the oracle's regime (`regime_star`), which is the
  registration's own reading from the commons' posted price (SPEC §9.2, OF6), not the mirror's
  `regime_end`, read from bids against offers.
- Free A2 (b): a trap run's runaway tick against the mirror's tick with the bound held to the
  displaced genesis prices, the harness's reference (SPEC §9.3; diag/runaway_ref.json); the
  band stays 5%.
- Free A2 (c): a CONVERGED run whose end D-hat is above 1e-9 rests off the oracle's point only if
  it has stopped falling: over the CSV's last 40 rows its D-hat falls on every row, at a root a
  tick within the registration's PL band (1e-4 below 0.999, 2e-5 above; SPEC §9.3) of the
  mirror's largest root at that target and dial (lin_dial05.json, lin_all05.json).
- The kick sets and CT2's verdict are not re-read.

Usage (WSL): python3 amend.py RUNS.jsonl ARCHIVE_ROOT"""
import csv
import gzip
import io
import json
import math
import os
import sys
from collections import Counter, OrderedDict

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import score_free as SF  # noqa: E402
import score_switch as SS  # noqa: E402
from common import REPO, read_recs  # noqa: E402
from gather import file_name, opener  # noqa: E402

RES = os.path.join(REPO, "docs", "probe", "results")
FREG = os.path.join(REPO, "docs", "probe", "free", "registered")


def write_rows(path, rows):
    """As common.write_rows, with LF line ends (the repository's)."""
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]), lineterminator="\n")
        w.writeheader()
        w.writerows(rows)


def lines_of(key):
    with gzip.open(os.path.join(RES, key, "lines.csv.gz"), "rt", newline="") as f:
        return list(csv.DictReader(f))


def setting_of(row, key):
    g = row["group"]
    if key == "switch":
        return g.split(" ", 1)[1] if row["set"] in ("nbhd", "swrate") else None
    return g.split(" ", 1)[1] if row["set"] == "dial" else None


def switch_a1(recs):
    E = SS.engine_index(recs)
    out, n_ref = [], 0
    for r in lines_of("switch"):
        if r["what"] != SS.END_LINES[1]:
            continue
        pop = SS.POPS[r["band"].split(":")[0]]
        e = E[(r["inst"], r["set"], setting_of(r, "switch"))][r["run"]]
        end = e["end"]
        last = int(end["tick"]) == e["clock_start"] + e["ticks"] - 1
        a = float(r["registered"])
        v = end[f"pool_{pop}"]
        rel = abs(v - a) / a
        ok = last and rel <= 1e-10
        n_ref += rel > 1e-9 and r["status"] != "reported"
        out.append(OrderedDict(amendment="switch A1", step=r["step"], inst=r["inst"], set=r["set"],
                               group=r["group"], run=r["run"], what=r["what"], registered=r["registered"],
                               engine_as_scored=r["engine"], status_as_scored=r["status"],
                               engine_on_amendment=repr(v), detail=f"|a/a* - 1| {rel:.3e} at tick {int(end['tick'])}" + (
                                   ", within 1e-10" if ok else ", beyond 1e-10"),
                               status_on_amendment=("reported" if r["status"] == "reported" else
                                                    ("pass" if ok else "fail"))))
    return out, n_ref


def dial_label(setting):
    if setting is None:
        return "C2m"
    k, f = setting.split(".*=")
    return f"tilt {f}" if k == "tilt" else f"{k} x{f}"


def target_label(inst, run):
    if "=" not in run or "@" not in run:
        return "base"
    cv = run.split("@")[0]
    c, v = cv.split("=")
    for coef, vals in SF.COEF[inst]:
        if coef == c and v in vals:
            return f"{c} {SF.FACTORS[vals.index(v)]}"
    return None


def free_a2(recs, archive):
    R, _ = SF.registered()
    E = SF.engine_index(recs)
    ref = {(x["inst"], x["run"]): x for x in map(json.loads, open(os.path.join(HERE, "diag", "runaway_ref.json")))}
    lin = {}
    for fn in ("lin_dial05.json", "lin_all05.json"):
        for x in json.load(open(os.path.join(FREG, fn))):
            lin.setdefault((x["name"].split("/")[0].lower(), x.get("dials", "C2m"), x["target"].split(" (")[0]), x["PL"])
    out = []
    for r in lines_of("free"):
        what, inst, s = r["what"], r["inst"], setting_of(r, "free")
        base = OrderedDict(step=r["step"], inst=inst, set=r["set"], group=r["group"], run=r["run"], what=what,
                           registered=r["registered"], engine_as_scored=r["engine"], status_as_scored=r["status"])
        if what == "the regime at the end":
            m = R[(inst, r["set"], s)][r["run"]][1]
            star = m.get("regime_star")
            ok = r["engine"] == star
            out.append(OrderedDict(amendment="free A2 (a)", **base, engine_on_amendment=r["engine"],
                                   detail=f"the oracle's regime {star}; the mirror's bids-against-offers readout {m.get('regime_end')}",
                                   status_on_amendment="pass" if ok else "fail"))
        elif what == "runaway tick":
            x = ref[(inst, r["run"])]
            reg, eng = x["tick_displaced"], int(float(r["engine"]))
            ok = abs(eng - reg) <= 0.05 * reg
            out.append(OrderedDict(amendment="free A2 (b)", **base, engine_on_amendment=r["engine"],
                                   detail=f"the mirror's tick on the harness's reference {reg} (registered {x['tick_undisplaced']})",
                                   status_on_amendment="pass" if ok else "fail"))
        elif what == "end D-hat" and r["status"] == "fail":
            e = E[(inst, r["set"], s)][r["run"]]
            f = opener(os.path.join(archive, e["rel"], file_name(e["run"]) + ".csv"))
            with f:
                rows = list(csv.reader(f))
            h = rows[0]
            it, idh = h.index("tick"), h.index("dhat")
            tail = [(int(x[it]), float(x[idh])) for x in rows[1:]][-40:]
            mono = all(b[1] < a[1] for a, b in zip(tail, tail[1:]))
            root = math.exp(math.log(tail[-1][1] / tail[0][1]) / (tail[-1][0] - tail[0][0]))
            pl = lin.get((inst, dial_label(s), target_label(inst, r["run"])))
            band = 1e-4 if pl is not None and pl < 0.999 else 2e-5
            ok = mono and pl is not None and abs(root - pl) <= band
            proj = pl ** (e["ticks"] - e["in_tol_from"]) if pl else None
            out.append(OrderedDict(amendment="free A2 (c)", **base, engine_on_amendment=r["engine"],
                                   detail=(f"D-hat falls on each of the last 40 rows: {mono}; root {root:.8f} against the "
                                           f"mirror's {pl:.8f} (|gap| {abs(root - pl):.1e}, band {band:g}); "
                                           f"the mirror's root from tolerance "
                                           f"(tick {e['in_tol_from']}) to L gives {proj:.2e}"),
                                   status_on_amendment="pass" if ok else "fail"))
    return out


def main(runs, archive):
    recs = read_recs(runs)
    sw, n_ref = switch_a1(recs)
    fr = free_a2(recs, archive)
    write_rows(os.path.join(RES, "switch", "amended.csv"), sw)
    write_rows(os.path.join(RES, "free", "amended.csv"), fr)
    for key, rows in (("switch", sw), ("free", fr)):
        print(f"{key}: {len(rows)} lines re-read")
        c = Counter((r["amendment"], r["status_as_scored"], r["status_on_amendment"]) for r in rows)
        for k in sorted(c):
            print(f"  {k[0]}: as scored {k[1]}, on the amendment {k[2]}: {c[k]}")
        worse = [r for r in rows if r["status_as_scored"] == "pass" and r["status_on_amendment"] != "pass"]
        print(f"  lines that passed as scored and fail on the amendment: {len(worse)}")
    # the scored lines after the amendments, and the refutations read again
    for key, rows in (("switch", sw), ("free", fr)):
        amended = {(r["step"], r["inst"], r["set"], r["group"], r["run"], r["what"], r["registered"]):
                   r["status_on_amendment"] for r in rows}
        st, still = Counter(), Counter()
        for r in lines_of(key):
            s = amended.get((r["step"], r["inst"], r["set"], r["group"], r["run"], r["what"], r["registered"]), r["status"])
            st[s] += 1
            if s in ("fail", "missing"):
                still[(r["step"], r["inst"], r["set"], r["what"])] += 1
        print(f"{key} after the amendments: " + ", ".join(f"{k} {v}" for k, v in sorted(st.items())))
        for k in sorted(still):
            print(f"  still failing: {k[0]} {k[1]} {k[2]}: {k[3]}: {still[k]}")
    print("switch refutations read again: 0 switch pops ending on the wrong side of their switch (as scored); "
          f"{n_ref} pooled pops more than 1e-9 relative from a* at the CSV's last row")
    off = [r for r in fr if r["amendment"] == "free A2 (c)" and r["status_on_amendment"] != "pass"]
    print(f"free refutations read again: {len(off)} CONVERGED runs resting off the oracle's point; "
          "the 2 failing kick sets stand (CT2 b.food=1.2@dated, exit.To=9.75@dated), and with them "
          "CT2's verdict, LOCAL")


if __name__ == "__main__":
    main(*sys.argv[1:3])
