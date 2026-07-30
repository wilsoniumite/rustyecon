"""Falsification tests for the A/B comparison (v2 Phase 4).

    python tools/test_ab.py

A comparison rule that cannot report LOSS is not a test, it is a rubber stamp.
The corpus was at 0/72 passing regions when this rule was written, which means
the obvious statistic — the pass count — could not distinguish an improved
kernel from an unchanged one, and the replacement has to be shown to
distinguish *something*. So each case below builds a kernel arm out of the real
legacy certificates, perturbs one thing, and asserts the rule notices.

The two that matter most:

  * an unchanged arm must report TIE, not a coin-flip of rounding noise;
  * an arm that wins by killing the economy must report LOSS. A dead region has
    flat metrics and therefore a perfect band, so a rule that read the band
    alone would rank collapse above every real improvement. That is the exact
    failure the liveness gate exists for, and it is asserted here rather than
    argued for in a comment.

Runs against `results/` as the legacy arm, so it also checks the shipped
certificates are the shape the tool expects.
"""

import copy
import glob
import json
import math
import os
import shutil
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import ab  # noqa: E402

REPO = Path(__file__).resolve().parent.parent
LEGACY = str(REPO / "results")

FAILURES = []


def check(name, cond, detail=""):
    if cond:
        print(f"  ok   {name}")
    else:
        print(f"  FAIL {name} {detail}")
        FAILURES.append(name)


def make_arm(tmp, mutate=None, only="lr_"):
    """Copy the legacy certificates into a fresh dir, optionally perturbed."""
    out = Path(tmp)
    for path in sorted(glob.glob(os.path.join(LEGACY, "*", "certificate.json"))):
        cert = json.load(open(path, encoding="utf-8"))
        if only and not cert["scenario"].startswith(only):
            continue
        cert = copy.deepcopy(cert)
        if mutate:
            mutate(cert)
        d = out / cert["scenario"]
        d.mkdir(parents=True, exist_ok=True)
        json.dump(cert, open(d / "certificate.json", "w", encoding="utf-8"), indent=1)
    return str(out)


def render(v):
    """Match the engine's rendering precision.

    Python's `{:e}` defaults to six decimal places; Rust's prints the shortest
    form that round-trips. Writing a fixture with the Python default silently
    rounds every band to seven significant figures, which showed up here as a
    "halve every band" case whose measured median was log(0.5) + 3e-8.
    """
    return f"{v:.17e}"


def scale_bands(factor):
    def f(cert):
        for r in cert["stability"]["regions"]:
            for m in r["measurements"]:
                if m["class"] == ab.BAND_CLASS and m["stat"] == ab.BAND_STAT:
                    v = float(m["value"])
                    if math.isfinite(v):
                        m["value"] = render(v * factor)
    return f


def kill_everything(cert):
    """The adversary: collapse every economy, so every band is perfect."""
    for r in cert["stability"]["regions"]:
        r["passed"] = False
        for m in r["measurements"]:
            if m["class"] == ab.BAND_CLASS and m["stat"] == ab.BAND_STAT:
                m["value"] = render(1.0)  # a flat metric: the best band there is
                m["tripped"] = False
            if m["class"] == "DEAD":
                m["tripped"] = True
                m["counted"] = True


def kill_one_live_region(cert):
    """Improve every band, but let one living region die for it."""
    scale_bands(0.5)(cert)
    for r in cert["stability"]["regions"]:
        alive = not any(m["tripped"] and m["class"] in ab.LIVENESS_CLASSES
                        for m in r["measurements"])
        if alive:
            for m in r["measurements"]:
                if m["class"] == "DEAD":
                    m["tripped"] = True
            return


def revive_everything(cert):
    for r in cert["stability"]["regions"]:
        for m in r["measurements"]:
            if m["class"] in ab.LIVENESS_CLASSES:
                m["tripped"] = False
                m["counted"] = False


def run(mutate, label, only="lr_"):
    with tempfile.TemporaryDirectory() as tmp:
        kernel_dir = make_arm(tmp, mutate, only)
        legacy = ab.load_arm("legacy", LEGACY, only)
        kernel = ab.load_arm("kernel", kernel_dir, only)
        result = ab.compare(legacy, kernel)
        # The receipt has to render for every verdict, including the ones nobody
        # wants to read; a formatter that crashes on LOSS hides the LOSS.
        text = ab.receipt(result, LEGACY, kernel_dir)
        assert label in text or result["verdict"] == "INVALID", text[:200]
        return result, text


def gate(result, gid):
    return next(g for g in result["gates"] if g["id"] == gid)


def main():
    print("A/B falsification tests")

    # 1. The null. An unchanged arm must be a TIE, and the gate must be met.
    r, _ = run(None, "TIE")
    check("an identical arm reports TIE", r["verdict"] == "TIE", r["verdict"])
    check("a TIE meets the gate", r["gate_met"])
    check("all three gates pass on a tie", all(g["passed"] for g in r["gates"]))
    check("no gate claims a strict improvement", not any(g["strict"] for g in r["gates"]))

    # 2. The adversary the liveness gate exists for. Every band is now a perfect
    #    1.0 — the best reading the statistic can take — bought by killing every
    #    economy. If this passes, the A/B rewards collapse.
    r, _ = run(kill_everything, "LOSS")
    check("collapsing every economy LOSES", r["verdict"] == "LOSS", r["verdict"])
    check("  and it is G1 that catches it", not gate(r, "G1")["passed"])
    check("  while the band gate was fooled", gate(r, "G3")["passed"],
          "G3 should be fooled by flat metrics — that is why G1 is necessary")

    # 3. Bands widen: a straightforward regression.
    r, _ = run(scale_bands(2.0), "LOSS")
    check("doubling every band LOSES", r["verdict"] == "LOSS", r["verdict"])
    check("  caught by both band gates", not gate(r, "G2")["passed"] and not gate(r, "G3")["passed"])

    # 4. Bands tighten everywhere: a straightforward win.
    r, _ = run(scale_bands(0.5), "WIN")
    check("halving every band WINS", r["verdict"] == "WIN", r["verdict"])
    check("  and the median moves by the right amount",
          abs(gate(r, "G3")["detail"].count("-0.6931")) >= 0 and
          abs(math.log(0.5) - r["scenario_level"]["median_dlog"]) < 1e-9,
          r["scenario_level"]["median_dlog"])

    # 5. Reviving regions is a win — the improvement the kernel is meant to make.
    r, _ = run(revive_everything, "WIN")
    check("reviving dead regions WINS", r["verdict"] == "WIN", r["verdict"])
    check("  on the liveness gate", gate(r, "G1")["strict"])

    # 6. A trade the rule refuses: better bands everywhere, one region killed.
    r, _ = run(kill_one_live_region, "LOSS")
    check("killing a live region LOSES even with better bands",
          r["verdict"] == "LOSS", r["verdict"])
    check("  and the receipt names the casualty", len(r["killed"]) >= 1, r["killed"])

    # 7. Different scenarios are not an A/B. R10 says identical scenarios, and
    #    the tape fingerprint is what makes that checkable.
    def retape(cert):
        cert["tape_sha"] = "0" * 16
    r, _ = run(retape, "INVALID")
    check("a changed tape is INVALID, not a verdict", r["verdict"] == "INVALID", r["verdict"])
    check("  and it is not silently treated as a pass", not r.get("gate_met"))

    # 8. Certificates from before the measurement table must be refused rather
    #    than scored as if every band were missing.
    def strip(cert):
        for reg in cert["stability"]["regions"]:
            reg.pop("measurements")
    try:
        run(strip, "")
        check("a certificate with no measurements is refused", False, "no error raised")
    except SystemExit as e:
        check("a certificate with no measurements is refused", "re-run" in str(e), str(e))

    # 9. The arithmetic on unbounded readings.
    check("both arms unbounded is a tie", ab.log_delta(math.inf, math.inf) == 0.0)
    check("kernel unbounded is a total loss", ab.log_delta(math.inf, 5.0) == math.inf)
    check("legacy unbounded is a total win", ab.log_delta(5.0, math.inf) == -math.inf)
    check("a NaN band is unbounded, not absent",
          ab.band_of([{"class": "DRIFTING", "stat": "range", "value": "NaN"}]) == math.inf)
    check("a missing band is unbounded", ab.band_of([]) == math.inf)
    check("median straddling an infinity is decided, not NaN",
          ab.median([-1.0, math.inf]) == -1.0, ab.median([-1.0, math.inf]))

    print()
    if FAILURES:
        print(f"{len(FAILURES)} FAILED: {', '.join(FAILURES)}")
        return 1
    print("all A/B falsification tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
