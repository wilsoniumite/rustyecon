"""The kernel-vs-legacy A/B, computed from two sets of run certificates.

Implements the decision rule pre-registered in `results/ab/preregistration.md`
(2026-07-31, METHODOLOGY R6). Read that file first: this module is the
mechanism, the pre-registration is the commitment, and the commitment was made
before the kernel arm existed.

The rule has no tunable constants. Every threshold in it is either zero (a
median may not rise) or a comparison between the two arms; the only registered
number is TIE_TOL, which decides when two floating-point readings count as the
same reading. A rule with dials could be turned until the answer came out right,
and this one has none to turn.

    python tools/ab.py <legacy_results_dir> <kernel_results_dir> [-o receipt.md]
"""

from __future__ import annotations

import argparse
import glob
import json
import math
import os
import sys
from dataclasses import dataclass, field

# ── the registered rule ───────────────────────────────────────────────────────

#: Fatal classes that say the economy is *not running*, as opposed to running
#: badly. Taken from the scenario criteria.ron: a region with no transaction
#: velocity, no employment, or destitute pops is not an economy whose price
#: bands mean anything. UNSTABLE / DRIFTING / SWINGING / CURRENCY_DRAIN are
#: instability, not death, and a region can be alive and still fail them.
LIVENESS_CLASSES = frozenset({"DEAD", "DEAD_EMPLOYMENT", "POP_DESTITUTION"})

#: The class whose statistic is the A/B's continuous measure. DRIFTING carries
#: LevelRange — the 95th/5th-percentile band of the metric over the scored
#: window — which is both continuous and exactly what the corpus fails on.
BAND_CLASS = "DRIFTING"
BAND_STAT = "range"

#: Two readings this close in log-space are the same reading. Present so a
#: bit-identical rerun reports TIE rather than a coin-flip of rounding noise.
TIE_TOL = 1e-9


# ── reading certificates ──────────────────────────────────────────────────────


@dataclass
class Region:
    scenario: str
    region: str
    live: bool
    band: float
    passed: bool
    tripped: set = field(default_factory=set)

    @property
    def key(self) -> tuple:
        return (self.scenario, self.region)


@dataclass
class Arm:
    name: str
    regions: dict
    tape_shas: dict
    criteria: set

    @property
    def live(self) -> list:
        return [r for r in self.regions.values() if r.live]

    @property
    def passing(self) -> list:
        return [r for r in self.regions.values() if r.passed]


def load_arm(name: str, results_dir: str, only: str = "lr_") -> Arm:
    """Read one arm's certificates into per-region readings."""
    regions, shas, criteria = {}, {}, set()
    paths = sorted(glob.glob(os.path.join(results_dir, "*", "certificate.json")))
    for path in paths:
        cert = json.load(open(path, encoding="utf-8"))
        scenario = cert["scenario"]
        if only and not scenario.startswith(only):
            continue
        stability = cert.get("stability")
        if stability is None:
            raise SystemExit(
                f"{path}: no stability report — the scenario registered no criteria, "
                "so it cannot take part in the A/B"
            )
        shas[scenario] = cert["tape_sha"]
        criteria.add((stability["criteria_date"], stability["criteria_version"],
                      tuple(stability["window"])))
        for rv in stability["regions"]:
            ms = rv.get("measurements")
            if ms is None:
                raise SystemExit(
                    f"{path}: certificate predates the measurement table (P4.0a); "
                    "re-run this arm before comparing"
                )
            r = Region(
                scenario=scenario,
                region=rv["region"],
                # Liveness is read from `tripped`, not `counted`: a DEAD region
                # short-circuits, so POP_DESTITUTION never *counts* against it,
                # and reading `counted` would make liveness depend on scoring
                # order rather than on the series. It is a property of the
                # economy, so it is measured the same way in every region.
                live=not any(m["tripped"] and m["class"] in LIVENESS_CLASSES for m in ms),
                band=band_of(ms),
                passed=rv["passed"],
                tripped={m["class"] for m in ms if m["tripped"]},
            )
            if r.key in regions:
                raise SystemExit(f"{path}: duplicate region {r.key}")
            regions[r.key] = r
    if not regions:
        raise SystemExit(f"{results_dir}: no certificates matching '{only}*'")
    return Arm(name=name, regions=regions, tape_shas=shas, criteria=criteria)


def band_of(measurements: list) -> float:
    """The region's worst band: the max over the band class's metrics.

    Max rather than mean, because the region's verdict is decided by its worst
    metric and an average would let a well-behaved one pay for a runaway one.
    A missing or NaN reading is unbounded, not absent — the criteria are
    fail-closed and so is this.
    """
    vals = [
        float(m["value"])
        for m in measurements
        if m["class"] == BAND_CLASS and m["stat"] == BAND_STAT
    ]
    if not vals:
        return math.inf
    vals = [math.inf if math.isnan(v) or v <= 0.0 else v for v in vals]
    return max(vals)


# ── the comparison ────────────────────────────────────────────────────────────


def log_delta(kernel: float, legacy: float) -> float:
    """log(band_kernel) - log(band_legacy), with the unbounded cases kept.

    Both infinite is a tie: two arms that both blew up did not differ. One
    infinite is a total win or loss, and is returned as an infinity so the
    median and the sign counts both handle it without a fabricated finite value.
    """
    ki, li = math.isinf(kernel), math.isinf(legacy)
    if ki and li:
        return 0.0
    if ki:
        return math.inf
    if li:
        return -math.inf
    return math.log(kernel) - math.log(legacy)


def median(xs: list) -> float:
    """Median that tolerates infinities (statistics.median averages the middle
    pair on even counts, and inf - inf is NaN)."""
    if not xs:
        return float("nan")
    s = sorted(xs)
    n = len(s)
    if n % 2:
        return s[n // 2]
    a, b = s[n // 2 - 1], s[n // 2]
    if a == b:
        return a
    if math.isinf(a) or math.isinf(b):
        # Straddling an infinity: report the finite side rather than NaN, which
        # would read as "uncomputable" when the answer is in fact decided.
        return b if math.isinf(a) else a
    return (a + b) / 2.0


def sign_counts(deltas: list) -> tuple:
    wins = sum(1 for d in deltas if d < -TIE_TOL)
    losses = sum(1 for d in deltas if d > TIE_TOL)
    return wins, losses, len(deltas) - wins - losses


@dataclass
class Gate:
    id: str
    label: str
    passed: bool
    detail: str
    strict: bool = False  # strictly better, as opposed to merely not worse


def compare(legacy: Arm, kernel: Arm) -> dict:
    """Apply the pre-registered rule. Returns the receipt as data."""
    # Identical scenarios is a precondition, not a courtesy. R10 requires the
    # A/B to be under identical scenarios; checking the tape fingerprints is
    # what makes that claim auditable instead of asserted.
    problems = []
    if set(legacy.regions) != set(kernel.regions):
        only_l = sorted(set(legacy.regions) - set(kernel.regions))
        only_k = sorted(set(kernel.regions) - set(legacy.regions))
        problems.append(f"region sets differ: legacy-only {only_l}, kernel-only {only_k}")
    for scen, sha in legacy.tape_shas.items():
        other = kernel.tape_shas.get(scen)
        if other is not None and other != sha:
            problems.append(f"{scen}: tape_sha differs ({sha} vs {other}) — not the same scenario")
    if legacy.criteria != kernel.criteria:
        problems.append(f"criteria differ: {sorted(legacy.criteria)} vs {sorted(kernel.criteria)}")
    if problems:
        return {"verdict": "INVALID", "problems": problems}

    keys = sorted(legacy.regions)
    all_deltas = [log_delta(kernel.regions[k].band, legacy.regions[k].band) for k in keys]

    both_live = [k for k in keys if legacy.regions[k].live and kernel.regions[k].live]
    live_deltas = [log_delta(kernel.regions[k].band, legacy.regions[k].band) for k in both_live]

    killed = [k for k in keys if legacy.regions[k].live and not kernel.regions[k].live]
    revived = [k for k in keys if not legacy.regions[k].live and kernel.regions[k].live]

    n_l, n_k = len(legacy.live), len(kernel.live)
    g1 = Gate(
        "G1",
        "liveness may not regress",
        passed=(n_k >= n_l and not killed),
        detail=f"live regions {n_l} -> {n_k}; {len(revived)} revived, {len(killed)} killed"
        + (f" ({', '.join('/'.join(k) for k in killed[:6])})" if killed else ""),
        strict=(n_k > n_l),
    )

    lw, ll, lt = sign_counts(live_deltas)
    lmed = median(live_deltas)
    g2 = Gate(
        "G2",
        "band may not widen where both arms keep the economy alive",
        passed=(len(both_live) == 0 or (lmed <= TIE_TOL and lw >= ll)),
        detail=f"n={len(both_live)}, median dlog={fmt(lmed)} "
        f"(x{fmt(math.exp(lmed)) if math.isfinite(lmed) else lmed}), "
        f"{lw} tighter / {ll} wider / {lt} unchanged",
        strict=(lmed < -TIE_TOL),
    )

    aw, al, at = sign_counts(all_deltas)
    amed = median(all_deltas)
    g3 = Gate(
        "G3",
        "band may not widen across the corpus",
        passed=(amed <= TIE_TOL),
        detail=f"n={len(keys)}, median dlog={fmt(amed)} "
        f"(x{fmt(math.exp(amed)) if math.isfinite(amed) else amed}), "
        f"{aw} tighter / {al} wider / {at} unchanged",
        strict=(amed < -TIE_TOL),
    )

    gates = [g1, g2, g3]
    if all(g.passed for g in gates):
        verdict = "WIN" if any(g.strict for g in gates) else "TIE"
    else:
        verdict = "LOSS"

    # Per-scenario medians: the 72 regions are 24 scenarios x 3, so regions
    # inside a scenario share a world and are not independent draws. The
    # scenario-level aggregate is the more defensible unit and is reported
    # alongside, though neither gates the decision.
    by_scen = {}
    for k, d in zip(keys, all_deltas):
        by_scen.setdefault(k[0], []).append(d)
    scen_med = {s: median(v) for s, v in by_scen.items()}
    sw, sl, st = sign_counts(list(scen_med.values()))

    return {
        "verdict": verdict,
        "gate_met": verdict in ("WIN", "TIE"),
        "gates": [g.__dict__ for g in gates],
        "regions": len(keys),
        "passing": {"legacy": len(legacy.passing), "kernel": len(kernel.passing)},
        "live": {"legacy": n_l, "kernel": n_k},
        "revived": ["/".join(k) for k in revived],
        "killed": ["/".join(k) for k in killed],
        "bands": {
            "legacy": band_summary([legacy.regions[k].band for k in keys]),
            "kernel": band_summary([kernel.regions[k].band for k in keys]),
            "legacy_live": band_summary([r.band for r in legacy.live]),
            "kernel_live": band_summary([r.band for r in kernel.live]),
        },
        "classes": {
            "legacy": class_counts(legacy),
            "kernel": class_counts(kernel),
        },
        "scenario_level": {
            "median_dlog": median(list(scen_med.values())),
            "tighter": sw, "wider": sl, "unchanged": st,
        },
        "per_region": [
            {
                "scenario": k[0], "region": k[1],
                "legacy_band": legacy.regions[k].band, "kernel_band": kernel.regions[k].band,
                "dlog": d,
                "legacy_live": legacy.regions[k].live, "kernel_live": kernel.regions[k].live,
            }
            for k, d in zip(keys, all_deltas)
        ],
    }


def band_summary(bands: list) -> dict:
    fin = sorted(b for b in bands if math.isfinite(b))
    return {
        "n": len(bands),
        "nonfinite": len(bands) - len(fin),
        "min": fin[0] if fin else None,
        "median": median(bands),
        "max": fin[-1] if fin else None,
        "within_2_2": sum(1 for b in bands if b <= 2.2),
    }


def class_counts(arm: Arm) -> dict:
    counts = {}
    for r in arm.regions.values():
        for c in r.tripped:
            counts[c] = counts.get(c, 0) + 1
    return dict(sorted(counts.items(), key=lambda kv: -kv[1]))


def fmt(v) -> str:
    if v is None:
        return "-"
    if isinstance(v, float):
        if math.isinf(v):
            return "inf" if v > 0 else "-inf"
        if math.isnan(v):
            return "NaN"
        return f"{v:.4g}"
    return str(v)


# ── the receipt ───────────────────────────────────────────────────────────────


def receipt(result: dict, legacy_dir: str, kernel_dir: str) -> str:
    if result["verdict"] == "INVALID":
        lines = ["# A/B receipt — INVALID", "",
                 "The two arms are not comparable, so no verdict was computed:", ""]
        lines += [f"- {p}" for p in result["problems"]]
        return "\n".join(lines) + "\n"

    out = [
        f"# A/B receipt — {result['verdict']}",
        "",
        f"Gate {'MET' if result['gate_met'] else 'NOT MET'}. "
        f"Legacy `{legacy_dir}` vs kernel `{kernel_dir}`, {result['regions']} regions, "
        "identical scenarios (tape fingerprints checked) under one criteria file.",
        "",
        "Decision rule pre-registered in `results/ab/preregistration.md` before the",
        "kernel arm existed (METHODOLOGY R6). All three gates are necessary.",
        "",
        "| gate | what it forbids | result | reading |",
        "|---|---|---|---|",
    ]
    for g in result["gates"]:
        out.append(
            f"| {g['id']} | {g['label']} | {'PASS' if g['passed'] else '**FAIL**'} | {g['detail']} |"
        )

    out += [
        "",
        "## What moved",
        "",
        "| | legacy | kernel |",
        "|---|---|---|",
        f"| regions passing the full criteria | {result['passing']['legacy']} | {result['passing']['kernel']} |",
        f"| regions alive | {result['live']['legacy']} | {result['live']['kernel']} |",
    ]
    for label, key in [("all regions", ""), ("live regions", "_live")]:
        lb, kb = result["bands"]["legacy" + key], result["bands"]["kernel" + key]
        out.append(f"| median band, {label} | {fmt(lb['median'])}x | {fmt(kb['median'])}x |")
        out.append(f"| worst band, {label} | {fmt(lb['max'])}x | {fmt(kb['max'])}x |")
        out.append(f"| bands within 2.2x, {label} | {lb['within_2_2']}/{lb['n']} | {kb['within_2_2']}/{kb['n']} |")

    out += ["", "Failure classes tripped, by region count:", "", "| class | legacy | kernel |", "|---|---|---|"]
    for c in sorted(set(result["classes"]["legacy"]) | set(result["classes"]["kernel"])):
        out.append(f"| {c} | {result['classes']['legacy'].get(c, 0)} | {result['classes']['kernel'].get(c, 0)} |")

    s = result["scenario_level"]
    out += [
        "",
        "## Caveat on the units",
        "",
        f"The {result['regions']} regions are 24 scenarios of 3 regions each, so regions inside",
        "one scenario share a world and are not independent draws. No p-value is",
        "reported, because the assumption it would need is false here. The",
        "scenario-level aggregate is the more defensible unit and is reported for",
        "comparison, though the pre-registered gates are the region-level ones:",
        "",
        f"- per-scenario median dlog: {fmt(s['median_dlog'])} "
        f"({s['tighter']} scenarios tighter / {s['wider']} wider / {s['unchanged']} unchanged)",
    ]
    if result["killed"]:
        out += ["", "**Regions the kernel killed that legacy kept alive:** "
                + ", ".join(result["killed"])]
    if result["revived"]:
        out += ["", f"Regions the kernel revived ({len(result['revived'])}): "
                + ", ".join(result["revived"])]
    return "\n".join(out) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("legacy")
    ap.add_argument("kernel")
    ap.add_argument("-o", "--out", help="write the receipt here as well as to stdout")
    ap.add_argument("--json", help="write the full result as JSON")
    ap.add_argument("--only", default="lr_", help="scenario name prefix (default lr_)")
    args = ap.parse_args()

    # The console codepage is cp1252 on Windows and the receipt is not ASCII.
    # Phase 3.5 corrupted all 28 criteria files by letting a default encoding
    # pick itself; every write here names utf-8, and so does this one.
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except (AttributeError, OSError):
        pass

    legacy = load_arm("legacy", args.legacy, args.only)
    kernel = load_arm("kernel", args.kernel, args.only)
    result = compare(legacy, kernel)
    text = receipt(result, args.legacy, args.kernel)
    print(text)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(text)
    if args.json:
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump(result, f, indent=1, default=str)
    return 0 if result.get("gate_met") else 1


if __name__ == "__main__":
    raise SystemExit(main())
