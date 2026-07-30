"""
Labour stability test suite runner and analyser.

Runs all 24 lr_XX scenarios plus the multi_region baseline for TICKS ticks,
writing Parquet telemetry, then computes per-region stability metrics and
issues.

PLOTS = False  (set True in .ipynb to see charts)
"""

import shutil
import copy, csv, subprocess, sys
from pathlib import Path
from dataclasses import dataclass, field

# The report uses PASS/FAIL glyphs; on a Windows console stdout defaults to
# cp1252 and the whole run dies at print time, after all the work is done.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

_root = next(p for p in [Path.cwd(), *Path.cwd().parents] if (p / "Cargo.toml").exists())
sys.path.insert(0, str(_root / "tools"))

import numpy as np
import pandas as pd

from scenario import SCENARIOS_DIR, REPO_ROOT, run_simulation
from readers import load_scenario_results, has_results, sampling_every, ScenarioResults

# ── Settings ──────────────────────────────────────────────────────────────────

PLOTS       = False
TICKS       = 1000
# Sampling this suite assumes. rolling_cv and the window slices below index by
# row position, so a subsampled run would silently rescale every window.
TELEMETRY_EVERY = 1
TRANSIENT   = 150         # ticks to discard as startup noise
ANALYSIS_END = TICKS
ANALYSIS_MID = TRANSIENT + (ANALYSIS_END - TRANSIENT) // 2   # ~575

# Output dirs
OUT_BASE    = REPO_ROOT / "tmp" / "stability_suite"
MANIFEST    = SCENARIOS_DIR / "lr_manifest.csv"

# Thresholds for issue detection
T_DEAD_VELOCITY    = 0.003   # velocity below this for >80% of window → DEAD
T_DEAD_EMPLOY      = 0.05    # employment below this for >80% of window → DEAD
T_UNSTABLE_CV      = 0.12    # rolling-10 CV above this sustained → UNSTABLE
T_LEVEL_RANGE      = 2.2     # p95/p05 band wider than this → DRIFTING
T_SWING_OSC        = 1.5     # detrended wobble grows by more than this → SWINGING
T_DRAIN_FRAC       = 0.82    # buildings hold >82% of regional GBP → DRAIN
T_DRAIN_SLOPE      = 0.0     # positive slope in 2nd half confirms drain trending up
T_DESTITUTION      = 0.8     # employed pop GBP < this × flour_cost × 1 week → DESTITUTION
T_DEAD_BLD_UTIL    = 0.05    # chosen/recipe < 5% for >60% of window → dead building
T_DEAD_BLD_FRAC    = 0.60

# ── Scenario list ─────────────────────────────────────────────────────────────

def scenario_list():
    names = ["multi_region"]
    if MANIFEST.exists():
        with open(MANIFEST) as f:
            rows = list(csv.DictReader(f))
        names += [r["name"] for r in rows]
        labels = {"multi_region": "baseline"} | {r["name"]: r["label"] for r in rows}
    else:
        labels = {"multi_region": "baseline"}
    return names, labels

# ── Run simulations ──────────────────────────────────────────────────

def run_all(names):
    OUT_BASE.mkdir(parents=True, exist_ok=True)
    print("Building binary…")
    subprocess.run(["cargo", "build"], cwd=str(REPO_ROOT),
                   capture_output=True, check=True)

    for name in names:
        out = OUT_BASE / name
        shutil.rmtree(out, ignore_errors=True)  # clear old results
        out.mkdir(parents=True, exist_ok=True)
        print(f"  {name}: running {TICKS} ticks…", end=" ", flush=True)
        # --record writes telemetry.parquet + manifest.json; --certify also
        # emits the engine's own per-region series alongside them.
        r = run_simulation(name, ticks=TICKS, output_dir=str(out),
                           record=True, certify=True, build=False)
        if not r.ok:
            print(f"ERROR (exit {r.returncode})\n{r.stderr[-500:]}")
        else:
            print("done (certificate FAIL)" if r.certificate_failed else "done")

# ── Load results ──────────────────────────────────────────────────────────────

def load_all(names):
    results = {}
    for name in names:
        out = OUT_BASE / name
        if not has_results(out):
            print(f"  {name}: no telemetry, skipping")
            continue
        # rolling_cv and the window slices below index by row position, so a
        # subsampled run would rescale every window without saying so.
        every = sampling_every(out)
        if every != TELEMETRY_EVERY:
            raise SystemExit(
                f"{name}: telemetry sampled every {every} ticks, but this suite's "
                f"windows assume every {TELEMETRY_EVERY}. Re-run it.")
        # No game_data needed: the manifest carries the dimension tables.
        results[name] = load_scenario_results(str(out))
    return results

# ── Metric helpers ────────────────────────────────────────────────────────────

def _node_for_region(res, region_id):
    return next((n for n, r in res.node_to_region.items() if r == region_id), None)

def _good_id(res, name):
    return next((k for k, v in res.good_name.items() if v == name), None)

def _currency_id(res, node_id):
    return res.node_currency.get(node_id)

def rolling_cv(series, window=10):
    """Mean coefficient of variation over a rolling window."""
    s = series.dropna()
    if len(s) < window * 2:
        return np.nan
    rm = s.rolling(window).mean().abs()
    rs = s.rolling(window).std()
    cv = (rs / rm.replace(0, np.nan)).dropna()
    return float(cv.mean())

def _log_fit(series):
    """OLS fit of ln(value) against tick over strictly positive samples.

    Mirrors certify::verdict::log_fit. Non-positive samples are dropped rather
    than clamped — a metric at exactly zero has no logarithm, and inventing one
    would put a fabricated point into the fit.
    """
    s = series.dropna()
    s = s[s > 0]
    if len(s) < 3:
        return None
    x = s.index.to_numpy(dtype=float)
    y = np.log(s.to_numpy(dtype=float))
    den = float(((x - x.mean()) ** 2).sum())
    if den <= 0:
        return None
    slope = float(((x - x.mean()) * (y - y.mean())).sum() / den)
    if not np.isfinite(slope):
        return None
    return slope, pd.Series(y - (y.mean() + slope * (x - x.mean())), index=s.index)


def level_range(series):
    """Ratio of the 95th to the 5th percentile of the level over the window.

    Mirrors certify::verdict::level_range. Asks "did the metric stay put", and is
    deliberately blind to the SHAPE of an excursion — a level that went somewhere
    and came back is as unstable as one that went and stayed. Percentiles rather
    than extremes so one freak sample cannot decide a verdict.

    Replaces the retired `log_drift`, an OLS trend fit that answered "what
    monotone trend best fits" instead: a fit through a V is flat, so a
    millionfold collapse-and-recovery scored 1.05 and passed clean.
    """
    s = series.dropna()
    s = s[s > 0]
    if len(s) < 3:
        return np.nan
    lo, hi = np.percentile(s.to_numpy(dtype=float), [5, 95])
    if lo <= 0:
        return np.inf
    return float(hi / lo)


def residual_damping(series, mid):
    """IQR(2nd half)/IQR(1st half) of the DETRENDED relative residual.

    Mirrors certify::verdict::residual_damping. Above 1 the oscillation is
    growing. Removing the log-trend first is the whole point: the retired
    `damping_ratio` read the raw level, where standard deviation scales with the
    level, so it scored a monotone collapse as "settling" (0.149) and a healthy
    stationary economy as "swinging" (1.009). See data/scenarios/*/criteria.ron
    generation 3 and tests/test_09_detectors.rs.
    """
    FLAT_REL = 1e-9
    fit = _log_fit(series)
    if fit is None:
        return np.nan
    res = fit[1]
    first, second = res[res.index < mid], res[res.index >= mid]
    # Interquartile range, not std: with std a single tick out of 851 could flip
    # this class (lr_01/Leeds moved 2.18 -> 0.73 on one sample's removal).
    q = lambda z: float(np.subtract(*np.percentile(z.to_numpy(dtype=float), [75, 25])))
    s1 = q(first)  if len(first)  > 5 else np.nan
    s2 = q(second) if len(second) > 5 else np.nan
    if np.isnan(s1) or np.isnan(s2):
        return np.nan
    if s1 < FLAT_REL:
        return 0.0 if s2 < FLAT_REL else np.inf
    return s2 / s1


def damping_ratio(series, mid):
    """RETIRED — std(second half) / std(first half) of the raw level.

    Kept only so a reader comparing against certificates registered before
    2026-07-31 can reproduce them. It is not a settling test: standard deviation
    is homogeneous of degree one, so the statistic falls whenever the level
    falls. Use `level_range` + `residual_damping`.
    """
    s = series.dropna()
    first  = s[s.index < mid]
    second = s[s.index >= mid]
    s1 = float(first.std())  if len(first)  > 5 else np.nan
    s2 = float(second.std()) if len(second) > 5 else np.nan
    # np.isnan, not `is np.nan`: a computed NaN is never the np.nan singleton,
    # so the identity check silently never fired and broken metrics passed.
    if np.isnan(s1) or np.isnan(s2):
        return np.nan   # uncomputable → fail-closed at the caller
    if s1 < 1e-12:
        return 0.0      # first half already flat: genuinely settled
    return s2 / s1

def analysis_slice(series):
    return series[(series.index >= TRANSIENT) & (series.index <= ANALYSIS_END)]

def frac_below(series, thresh):
    s = analysis_slice(series).dropna()
    if len(s) == 0: return np.nan
    return float((s < thresh).mean())

def linear_slope(series):
    s = series.dropna()
    if len(s) < 3: return 0.0
    x = np.arange(len(s), dtype=float)
    return float(np.polyfit(x, s.values, 1)[0])

# ── Per-region metrics ────────────────────────────────────────────────────────

def compute_metrics(res: ScenarioResults, region_id: int) -> dict:
    node = _node_for_region(res, region_id)
    if node is None:
        return {}

    cid  = _currency_id(res, node)
    labour_gid = _good_id(res, "labour")
    flour_gid  = _good_id(res, "flour")
    wheat_gid  = _good_id(res, "wheat")
    nc_goods = [g for g in res.good_name if g != cid]

    # ── Prices at this node ───────────────────────────────────────────────────
    node_prices = res.prices[res.prices["node_id"] == node].copy()
    node_prices["cleared"] = node_prices[["supply", "demand"]].min(axis=1)
    node_prices["txn_value"] = node_prices["cleared"] * node_prices["price"]

    nc_prices = node_prices[node_prices["good_id"].isin(nc_goods)]
    txn_by_tick = nc_prices.groupby("tick")["txn_value"].sum()
    trade_vol   = nc_prices.groupby("tick")["cleared"].sum()

    # ── Currency holdings ─────────────────────────────────────────────────────
    region_blds = res.buildings[res.buildings["region_id"] == region_id]["building_id"].unique()
    region_pops = res.pops[res.pops["region_id"] == region_id]["pop_id"].unique()

    if cid is not None:
        inv = res.inventories[res.inventories["good_id"] == cid].copy()
        inv_bld = inv[(inv["entity_type"] == "building") &
                      (inv["entity_id"].isin(region_blds))]
        inv_pop = inv[(inv["entity_type"] == "pop") &
                      (inv["entity_id"].isin(region_pops))]
        bld_gbp = inv_bld.groupby("tick")["qty"].sum()
        pop_gbp = inv_pop.groupby("tick")["qty"].sum()
        total_gbp = bld_gbp.add(pop_gbp, fill_value=0)
    else:
        total_gbp = pd.Series(dtype=float)
        bld_gbp   = pd.Series(dtype=float)
        pop_gbp   = pd.Series(dtype=float)

    # Velocity = transaction value / total GBP
    velocity = txn_by_tick / total_gbp.replace(0, np.nan)

    # GBP concentration: buildings' share
    concentration = bld_gbp / total_gbp.replace(0, np.nan)

    # ── Employment ────────────────────────────────────────────────────────────
    region_pop_df = res.pops[res.pops["region_id"] == region_id].copy()
    emp_df   = region_pop_df[region_pop_df["is_employed"] == True]
    unemp_df = region_pop_df[region_pop_df["is_employed"] == False]
    emp_size   = emp_df.groupby("tick")["size"].sum()
    total_size = region_pop_df.groupby("tick")["size"].sum()
    employment = emp_size / total_size.replace(0, np.nan)

    # ── Real wage (labour price / flour price) ────────────────────────────────
    labour_p = (node_prices[node_prices["good_id"] == labour_gid]
                .groupby("tick")["price"].mean() if labour_gid is not None
                else pd.Series(dtype=float))
    flour_p  = (node_prices[node_prices["good_id"] == flour_gid]
                .groupby("tick")["price"].mean() if flour_gid is not None
                else pd.Series(dtype=float))
    real_wage = labour_p / flour_p.replace(0, np.nan)

    # ── Building utilisation (non-channel buildings in this region) ───────────
    region_bld_df = res.buildings[
        (res.buildings["region_id"] == region_id) &
        (res.buildings["channel_id"] == -1)
    ].copy()
    region_bld_df["util"] = (region_bld_df["chosen_size"] /
                              region_bld_df["recipe_size"].replace(0, np.nan)).clip(0, 1)
    bld_util = region_bld_df.groupby("tick")["util"].mean()

    # ── Employed pop real income (GBP per employed capita / flour cost) ───────
    emp_pops_inv = (res.inventories[
        (res.inventories["entity_type"] == "pop") &
        (res.inventories["entity_id"].isin(
            emp_df["pop_id"].unique() if "pop_id" in emp_df else []))
    ] if cid is not None else pd.DataFrame())

    if cid is not None and not emp_pops_inv.empty:
        emp_gbp = (emp_pops_inv[emp_pops_inv["good_id"] == cid]
                   .groupby("tick")["qty"].sum())
        emp_total_size = emp_df.groupby("tick")["size"].sum()
        gbp_per_cap = emp_gbp / emp_total_size.replace(0, np.nan)
        real_income = gbp_per_cap / flour_p.replace(0, np.nan)
    else:
        real_income = pd.Series(dtype=float)

    return dict(
        velocity=velocity,
        trade_vol=trade_vol,
        employment=employment,
        real_wage=real_wage,
        concentration=concentration,
        bld_util=bld_util,
        real_income=real_income,
        bld_gbp=bld_gbp,
        pop_gbp=pop_gbp,
        flour_price=flour_p,
        labour_price=labour_p,
        region_bld_df=region_bld_df,
    )

# ── Issue detection ───────────────────────────────────────────────────────────

@dataclass
class RegionResult:
    region_name: str
    issues: list = field(default_factory=list)
    metrics: dict = field(default_factory=dict)

    def passed(self):
        fatal = {"DEAD", "UNSTABLE", "DRIFTING", "SWINGING", "CURRENCY_DRAIN", "POP_DESTITUTION"}
        return not any(i.split("(")[0] in fatal for i in self.issues)

def detect_issues(m: dict, res: ScenarioResults) -> list:
    issues = []

    vel = analysis_slice(m["velocity"])
    emp = analysis_slice(m["employment"])
    util = analysis_slice(m["bld_util"])
    conc = analysis_slice(m["concentration"])
    rw   = analysis_slice(m["real_wage"])
    ri   = analysis_slice(m["real_income"])

    # ── DEAD ─────────────────────────────────────────────────────────────────
    if vel.empty or (frac_below(m["velocity"], T_DEAD_VELOCITY) > 0.80):
        issues.append("DEAD")
        return issues   # no point checking further

    if not emp.empty and frac_below(m["employment"], T_DEAD_EMPLOY) > 0.80:
        issues.append("DEAD(employment)")

    # ── UNSTABLE: short-term oscillation ─────────────────────────────────────
    for label, series in [("velocity", m["velocity"]),
                           ("employment", m["employment"]),
                           ("real_wage", m["real_wage"])]:
        s = analysis_slice(series)
        if s.empty: continue
        cv = rolling_cv(s, window=10)
        # Fail-closed: a non-empty series whose CV cannot be computed is an issue,
        # not a silent pass (the old `cv is not np.nan` guard never fired).
        if np.isnan(cv) or cv > T_UNSTABLE_CV:
            issues.append(f"UNSTABLE({label} CV={cv:.2f})")

    # ── DRIFTING: the level does not stay put ─────────────────────────────────
    for label, series in [("velocity", m["velocity"]),
                           ("real_wage", m["real_wage"])]:
        s = analysis_slice(series)
        if s.empty: continue
        d = level_range(s)
        # Fail-closed: a range that cannot be computed is an issue, not a pass.
        if np.isnan(d) or d > T_LEVEL_RANGE:
            issues.append(f"DRIFTING({label} range={d:.2f}x)")

    # ── SWINGING: the wobble grows ────────────────────────────────────────────
    for label, series in [("velocity", m["velocity"]),
                           ("real_wage", m["real_wage"])]:
        s = analysis_slice(series)
        if s.empty: continue
        osc = residual_damping(s, ANALYSIS_MID)
        # Fail-closed: an uncomputable ratio counts as not-settling.
        if np.isnan(osc) or osc > T_SWING_OSC:
            issues.append(f"SWINGING({label} osc={osc:.2f})")

    # ── CURRENCY_DRAIN ────────────────────────────────────────────────────────
    if not conc.empty:
        second_half = conc[conc.index >= ANALYSIS_MID].dropna()
        mean_conc = float(conc.mean())
        slope = linear_slope(second_half) if len(second_half) > 5 else 0.0
        if mean_conc > T_DRAIN_FRAC and slope > T_DRAIN_SLOPE:
            issues.append(f"CURRENCY_DRAIN({mean_conc:.2f} bld share, slope={slope:.4f})")

    # ── POP_DESTITUTION ───────────────────────────────────────────────────────
    if not ri.empty:
        frac_dest = frac_below(m["real_income"], T_DESTITUTION)
        # Fail-closed: if the destitution fraction can't be computed over a
        # non-empty income series, flag it rather than pass silently.
        if np.isnan(frac_dest) or frac_dest > 0.50:
            issues.append(f"POP_DESTITUTION(frac={frac_dest:.2f})")

    # ── DEAD_BUILDING (warning) ───────────────────────────────────────────────
    bld_df = m.get("region_bld_df", pd.DataFrame())
    if not bld_df.empty:
        for bid, grp in bld_df.groupby("building_id"):
            a = grp[grp["tick"] >= TRANSIENT]
            if len(a) == 0: continue
            frac_dead = float((a["util"] < T_DEAD_BLD_UTIL).mean())
            if frac_dead > T_DEAD_BLD_FRAC:
                rname = res.recipe_name.get(int(a["recipe_id"].iloc[0]), f"bld{bid}")
                issues.append(f"DEAD_BUILDING({rname})")

    return issues

# ── Summary stats for printing ────────────────────────────────────────────────

def summary_stats(m: dict) -> dict:
    def _mean(series):
        s = analysis_slice(series).dropna()
        return float(s.mean()) if len(s) > 0 else float("nan")
    return {
        "vel_mean":  round(_mean(m["velocity"]),   3),
        "emp_mean":  round(_mean(m["employment"]), 3),
        "rw_mean":   round(_mean(m["real_wage"]),  3),
        "conc_mean": round(_mean(m["concentration"]), 3),
        "util_mean": round(_mean(m["bld_util"]),   3),
    }

# ── Main analysis ─────────────────────────────────────────────────────────────

def analyse_all(all_results: dict, labels: dict):
    scenario_rows = []

    for name, res in all_results.items():
        label = labels.get(name, name)
        regions = sorted(r for r in res.node_to_region.values() if r is not None)
        region_results = []

        for rid in regions:
            rname = res.region_name.get(rid, f"r{rid}")
            m = compute_metrics(res, rid)
            if not m:
                region_results.append(RegionResult(rname, ["NO_DATA"]))
                continue
            issues = detect_issues(m, res)
            rr = RegionResult(rname, issues, summary_stats(m))
            region_results.append(rr)

        passing = sum(1 for rr in region_results if rr.passed())
        total   = len(region_results)
        scenario_rows.append((name, label, passing, total, region_results))

    return scenario_rows

# ── Print report ──────────────────────────────────────────────────────────────

def print_report(rows):
    print("\n" + "=" * 72)
    print("STABILITY SUITE RESULTS")
    print("=" * 72)

    issue_counts = {}
    for name, label, passing, total, region_results in rows:
        score = f"{passing}/{total}"
        status = "✓" if passing == total else "✗"
        print(f"\n{status} {name:20s}  [{score}]  ({label})")
        for rr in region_results:
            flag = "PASS" if rr.passed() else "FAIL"
            issues_str = ", ".join(rr.issues) if rr.issues else "-"
            stats = rr.metrics
            print(f"    {rr.region_name:12s} {flag}  vel={stats.get('vel_mean','?'):.3f}"
                  f"  emp={stats.get('emp_mean','?'):.2f}"
                  f"  rw={stats.get('rw_mean','?'):.2f}"
                  f"  conc={stats.get('conc_mean','?'):.2f}"
                  f"  util={stats.get('util_mean','?'):.2f}")
            if rr.issues:
                print(f"               ISSUES: {issues_str}")
            for i in rr.issues:
                issue_counts[i.split("(")[0]] = issue_counts.get(i.split("(")[0], 0) + 1

    print("\n" + "-" * 72)
    print("SUMMARY")
    all_passing = sum(p == t for _, _, p, t, _ in rows)
    total_scenarios = len(rows)
    region_totals = sum(t for _, _, _, t, _ in rows)
    region_passing = sum(p for _, _, p, _, _ in rows)
    print(f"  Scenarios: {all_passing}/{total_scenarios} all-regions-pass")
    print(f"  Regions:   {region_passing}/{region_totals} passed")
    print(f"  Issue frequency:")
    for issue, count in sorted(issue_counts.items(), key=lambda x: -x[1]):
        print(f"    {issue:30s} {count}")
    print("=" * 72)

# ── Plots ─────────────────────────────────────────────────────────────────────

def make_plots(rows, all_results):
    import matplotlib.pyplot as plt
    import matplotlib.gridspec as gridspec

    names   = [r[0] for r in rows]
    passing = [r[2] / max(r[3], 1) for r in rows]
    labels  = [r[1][:30] for r in rows]

    # ── 1. Pass rate bar chart ────────────────────────────────────────────────
    fig, ax = plt.subplots(figsize=(14, 5))
    colours = ["#2ca02c" if p == 1.0 else "#ff7f0e" if p > 0 else "#d62728"
               for p in passing]
    ax.bar(range(len(names)), passing, color=colours)
    ax.set_xticks(range(len(names)))
    ax.set_xticklabels(names, rotation=45, ha="right", fontsize=8)
    ax.set_ylabel("Fraction of regions passing")
    ax.set_title("Stability pass rate by scenario")
    ax.axhline(1.0, color="green", linestyle="--", alpha=0.4)
    plt.tight_layout()
    plt.savefig(_root / "tmp" / "stability_pass_rate.png", dpi=120)
    plt.close()

    # ── 2. Heatmap of key metrics across scenarios ────────────────────────────
    metric_names = ["vel_mean", "emp_mean", "rw_mean", "conc_mean", "util_mean"]
    matrix = []
    for name, label, passing, total, region_results in rows:
        row_vals = []
        for mn in metric_names:
            vals = [rr.metrics.get(mn, np.nan) for rr in region_results
                    if not np.isnan(rr.metrics.get(mn, np.nan))]
            row_vals.append(np.nanmean(vals) if vals else np.nan)
        matrix.append(row_vals)
    matrix = np.array(matrix, dtype=float)

    fig, ax = plt.subplots(figsize=(9, max(6, len(names) * 0.3)))
    im = ax.imshow(matrix, aspect="auto", cmap="RdYlGn")
    ax.set_xticks(range(len(metric_names)))
    ax.set_xticklabels(["velocity", "employment", "real wage",
                         "bld conc", "bld util"], fontsize=9)
    ax.set_yticks(range(len(names)))
    ax.set_yticklabels(names, fontsize=8)
    plt.colorbar(im, ax=ax, fraction=0.03, pad=0.02)
    ax.set_title("Mean metric values (analysis window, all regions avg)")
    plt.tight_layout()
    plt.savefig(_root / "tmp" / "stability_heatmap.png", dpi=120)
    plt.close()

    # ── 3. Time series for multi_region baseline ──────────────────────────────
    if "multi_region" in all_results:
        res = all_results["multi_region"]
        regions = sorted(r for r in res.node_to_region.values() if r is not None)
        fig, axes = plt.subplots(5, len(regions), figsize=(5 * len(regions), 12),
                                  sharex=True)
        if len(regions) == 1:
            axes = [[a] for a in axes]

        for col, rid in enumerate(regions):
            rname = res.region_name.get(rid, f"r{rid}")
            m = compute_metrics(res, rid)
            if not m: continue

            for row, (label, key) in enumerate([
                ("Velocity", "velocity"),
                ("Employment", "employment"),
                ("Real wage", "real_wage"),
                ("Bld conc.", "concentration"),
                ("Bld util.", "bld_util"),
            ]):
                ax = axes[row][col]
                s = m[key].dropna()
                ax.plot(s.index, s.values, lw=1)
                ax.axvline(TRANSIENT, color="grey", linestyle="--", alpha=0.5, lw=0.8)
                ax.axvline(ANALYSIS_MID, color="grey", linestyle=":", alpha=0.5, lw=0.8)
                if col == 0:
                    ax.set_ylabel(label, fontsize=8)
                if row == 0:
                    ax.set_title(rname, fontsize=9)

        fig.suptitle("multi_region baseline — metric time series", fontsize=11)
        plt.tight_layout()
        plt.savefig(_root / "tmp" / "stability_baseline_timeseries.png", dpi=120)
        plt.close()

    print(f"\nPlots saved to {_root / 'tmp'}/")

# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    names, labels = scenario_list()
    print(f"Suite: {len(names)} scenarios, {TICKS} ticks each")

    run_all(names)

    print("\nLoading results…")
    all_results = load_all(names)
    print(f"  Loaded {len(all_results)} scenarios")

    print("\nAnalysing…")
    rows = analyse_all(all_results, labels)

    print_report(rows)

    if PLOTS:
        make_plots(rows, all_results)
