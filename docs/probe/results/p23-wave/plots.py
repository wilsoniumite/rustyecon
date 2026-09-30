"""P2.3.12 (label run, 2026-09-30): the wave's small plots, from the scorer's tables and the
archived runs. Usage (WSL): python3 plots.py ARCHIVE REPO
ARCHIVE is D:/rustyecon-p23/runs (as /mnt/d/...), REPO the worktree. Writes
docs/probe/figs/wall/*.png and docs/probe/figs/commons/*.png."""
import csv
import gzip
import io
import math
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

SURF, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
S1, S2, S3 = "#2a78d6", "#eb6834", "#1baf7a"
plt.rcParams.update({"figure.facecolor": SURF, "axes.facecolor": SURF, "axes.edgecolor": INK2,
                     "axes.labelcolor": INK, "xtick.color": INK2, "ytick.color": INK2,
                     "text.color": INK, "font.size": 9, "axes.grid": True, "grid.color": GRID,
                     "grid.linewidth": 0.6, "axes.spines.top": False, "axes.spines.right": False,
                     "lines.linewidth": 1.6, "legend.frameon": False})


def table(path):
    with open(path, newline="") as f:
        return list(csv.DictReader(f))


def series(path, cols, lo=0, hi=None):
    with gzip.open(path + ".gz") if os.path.exists(path + ".gz") else open(path, "rb") as fb:
        r = csv.reader(io.TextIOWrapper(fb, newline=""))
        head = next(r)
        idx = {c: head.index(c) for c in cols if c in head}
        idx.update({c: [i for i, h in enumerate(head) if h.startswith(c[:-1])] for c in cols if c.endswith("*")})
        out = {c: [] for c in ["tick"] + cols}
        for row in r:
            t = int(row[0])
            if t < lo:
                continue
            if hi is not None and t > hi:
                break
            out["tick"].append(t)
            for c in cols:
                i = idx[c]
                out[c].append(sum(float(row[j]) for j in i) if isinstance(i, list) else float(row[i]))
    return out


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return None


def scatter(rows, groups, path, title):
    fig, ax = plt.subplots(figsize=(5.2, 4.4))
    lo, hi = math.inf, 0
    for i, (label, color, pick) in enumerate(groups):
        xs, ys = [], []
        for r in rows:
            m, e = num(r.get("ttol_mirror")), num(r.get("ttol_engine"))
            if m and e and pick(r) and r.get("cls_mirror") == r.get("cls_engine") == "CONVERGED":
                xs.append(m)
                ys.append(e)
        if xs:
            lo, hi = min(lo, min(xs + ys)), max(hi, max(xs + ys))
            # the first group on top: the verdict's runs are not hidden under the families
            ax.scatter(xs, ys, s=14 + 10 * (len(groups) - 1 - i), color=color,
                       label=f"{label} ({len(xs)})", edgecolors=SURF, linewidths=0.6,
                       zorder=3 + len(groups) - i)
    ax.plot([lo * 0.9, hi * 1.1], [lo * 0.9, hi * 1.1], color=INK2, lw=0.8, ls="--", zorder=2)
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("ticks to tolerance, mirror (registered)")
    ax.set_ylabel("ticks to tolerance, engine")
    ax.set_title(title, loc="left", fontsize=10)
    ax.legend(loc="upper left", fontsize=8)
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)


def main(archive, repo):
    fw = os.path.join(repo, "docs/probe/figs/wall")
    fc = os.path.join(repo, "docs/probe/figs/commons")
    os.makedirs(fw, exist_ok=True)
    os.makedirs(fc, exist_ok=True)
    res = os.path.join(repo, "docs/probe/results")
    wr = table(os.path.join(res, "wall/battery.csv"))
    scatter(wr, [("battery, Tier 3S, 10·L", S1, lambda r: r["set"] in ("battery", "tier3s", "battery at 10L", "tier3s at 10L")),
                 ("families at 52 a year, IC1", S2, lambda r: r["set"] in ("stocks", "joint2", "joint4", "basin", "hold", "tilt1", "line", "tier3s_line")),
                 ("12 and 365 a year", S3, lambda r: r["set"] in ("tpy12", "tpy365"))],
            os.path.join(fw, "fig1_ttol_engine_vs_mirror.png"),
            "IW1 and IC1: ticks to tolerance, every CONVERGED run")
    cr = table(os.path.join(res, "commons/runs.csv"))
    scatter(cr, [("battery, Tier 3S, 10·L", S1, lambda r: r["set"] in ("L", "10L", "tier3s", "tier3s10L") and r["inst"] != "c1n"),
                 ("families at 52 a year, C1N", S2, lambda r: r["set"] in ("stocks", "enclose", "joint2", "joint4", "basin", "hold", "tilt1") or r["inst"] == "c1n"),
                 ("12 and 365 a year", S3, lambda r: r["set"] in ("tpy12", "tpy365"))],
            os.path.join(fc, "fig1_ttol_engine_vs_mirror.png"),
            "C1, C2 and C1N: ticks to tolerance, every CONVERGED run")
    # the wall's one breach: JB(0.5)
    s = series(os.path.join(archive, "wall/iw1/L/JB_0.5_/JB_0.5_.csv"),
               ["depth", "s_planned_services", "s_planned_goods"], 0, 250)
    fig, (a, b) = plt.subplots(1, 2, figsize=(8.4, 3.2))
    a.plot(s["tick"], s["depth"], color=S1)
    a.axhline(0, color=INK2, lw=0.8)
    a.set_title("depth ln(θw/(p_mach·γ(1)))", loc="left", fontsize=9)
    a.set_xlabel("tick")
    b.plot(s["tick"], s["s_planned_services"], color=S1, label="services")
    b.plot(s["tick"], s["s_planned_goods"], color=S2, ls="--", label="goods")
    b.set_title("each desk's human share", loc="left", fontsize=9)
    b.set_xlabel("tick")
    b.legend()
    fig.suptitle("IW1, JB(0.5): the one breach of the wall in the battery", x=0.01, ha="left", fontsize=10)
    fig.tight_layout()
    fig.savefig(os.path.join(fw, "fig2_jb05_breach.png"), dpi=150)
    plt.close(fig)
    # the wall's cost shocks: baskets over Y*
    fig, ax = plt.subplots(figsize=(6.4, 3.4))
    for run, color, label, ls in (("land.mach_0.8_genesis", S1, "land.mach × 2", "-"),
                                  ("tail.services_0.2_genesis", S2, "tail.services × 2", "--"),
                                  ("res.services.trained_0.08_genesis", S3, "res.services.trained × 2", ":")):
        s = series(os.path.join(archive, f"wall/iw1/L/{run}/{run}.csv"), ["baskets_*", "y.services_star"], 0, 800)
        ax.plot(s["tick"], [b / y for b, y in zip(s["baskets_*"], s["y.services_star"])], color=color,
                ls=ls, label=label)
    ax.set_ylim(0, 1.1)
    ax.set_xlabel("tick")
    ax.set_ylabel("baskets eaten over the new Y*")
    ax.set_title("IW1: the cost shocks at genesis", loc="left", fontsize=10)
    ax.legend(loc="lower right")
    fig.tight_layout()
    fig.savefig(os.path.join(fw, "fig3_cost_shocks.png"), dpi=150)
    plt.close(fig)
    # the commons: the trap, C1's land.mach history, window 5
    run = "cycle_land.mach_1500_80_"
    s = series(os.path.join(archive, f"commons/c1/history/{run}/{run}.csv"),
               ["part_workers", "p.food", "p.labour"], 7300, 7800)
    fig, (a, b) = plt.subplots(1, 2, figsize=(8.4, 3.2))
    a.plot(s["tick"], s["part_workers"], color=S1)
    a.axvline(7500, color=INK2, lw=0.8, ls="--")
    a.set_title("the workers' participation", loc="left", fontsize=9)
    a.set_xlabel("tick of the run (window 5 from 7,500)")
    b.plot(s["tick"], [0.3 * f / w for f, w in zip(s["p.food"], s["p.labour"])], color=S2)
    b.axvline(7500, color=INK2, lw=0.8, ls="--")
    b.set_title("the exit's value over the wage, p_food·s₀/w", loc="left", fontsize=9)
    b.set_xlabel("tick of the run")
    fig.suptitle("C1, cycle(land.mach,1500,80): the subsistence trap in window 5", x=0.01, ha="left", fontsize=10)
    fig.tight_layout()
    fig.savefig(os.path.join(fc, "fig2_trap_history.png"), dpi=150)
    plt.close(fig)
    # the commons' regimes: C1's commons history, the first ten windows
    run = "cycle_commons_1500_80_"
    s = series(os.path.join(archive, f"commons/c1/history/{run}/{run}.csv"),
               ["part_workers", "ro_over_r"], 0, 15000)
    fig, (a, b) = plt.subplots(2, 1, figsize=(7.2, 4.4), sharex=True)
    a.plot(s["tick"], s["ro_over_r"], color=S1, lw=1.0)
    a.set_title("the commons' shadow rent over r (0 with room, 1 when plots spill)", loc="left", fontsize=9)
    b.plot(s["tick"], s["part_workers"], color=S2, lw=1.0)
    b.set_title("the workers' participation", loc="left", fontsize=9)
    b.set_xlabel("tick (the commons × 1.1, × 0.9, × 2, × 0.5, × 1 every 1,500 ticks)")
    fig.suptitle("C1, cycle(commons,1500,80): the rule through the three regimes", x=0.01, ha="left", fontsize=10)
    fig.tight_layout()
    fig.savefig(os.path.join(fc, "fig3_commons_history.png"), dpi=150)
    plt.close(fig)
    print("plots written")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
