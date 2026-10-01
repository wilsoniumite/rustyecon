"""P2.4 (label run, 2026-10-01): the session's small plots, from the two waves' gathered runs, the
dial map, the B-D archive's CSVs and the diagnostics. Not a scorer. Every number plotted is in a
committed table or printout beside it (the dial map's CSV, the scorers' tables, diag/*.out).
Usage (WSL): python3 plots.py WAVE_A_RUNS.jsonl[.gz] BCD_RUNS.jsonl[.gz] DIALMAP.csv BCD_ARCHIVE REPO
Writes docs/probe/figs/families/{dialmap,i2_history}.png, figs/trap/dial_ticks.png,
figs/switch/corner.png and figs/free/{ct2_kicks,ct2_slow_root}.png."""
import csv
import gzip
import json
import math
import os
import re
import statistics
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

# the reference palette (dataviz skill, references/palette.md): surface, ink, categorical slots
# 1-4 (validated: CVD and normal-vision floors pass; slots 3-4 under 3:1 contrast, so every
# series also has a marker and a legend), and the status pair good / critical
SURF, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
S = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100"]
GOOD, CRIT, NONE = "#0ca30c", "#d03b3b", "#f0efec"
plt.rcParams.update({"figure.facecolor": SURF, "axes.facecolor": SURF, "axes.edgecolor": INK2,
                     "axes.labelcolor": INK, "xtick.color": INK2, "ytick.color": INK2,
                     "text.color": INK, "font.size": 9, "axes.grid": True, "grid.color": GRID,
                     "grid.linewidth": 0.6, "axes.spines.top": False, "axes.spines.right": False,
                     "lines.linewidth": 2.0, "legend.frameon": False})
SETTINGS = ([f"rate.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"buffer.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"adjust.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"tilt.*={v}" for v in ("0.05", "0.1", "0.25", "0.5", "1")])
COLS = SETTINGS + ["tilt.*=2"]


def short(s):
    k, v = s.split(".*=")
    return f"tilt {v}" if k == "tilt" else f"{k} ×{v}"


ORDER = ["iw1", "is1", "is2", "c1", "c1p", "c2", "c2p", "il1", "ct2"]
LABEL = {"iw1": "IW1, wall (GO)", "is1": "IS1, switch (GO)", "is2": "IS2, switch control",
         "c1": "C1, commons (GO at C2m)", "c1p": "C1P, paced (GO, margin)", "c2": "C2, commons (GO)",
         "c2p": "C2P, paced (GO, margin)", "il1": "IL1, idle land (GO)", "ct2": "CT2, two types (LOCAL)"}


def records(path):
    op = gzip.open if path.endswith(".gz") else open
    with op(path, "rt") as f:
        return [json.loads(line) for line in f if line.strip()]


def csv_rows(path):
    f = gzip.open(path + ".gz", "rt", newline="") if os.path.exists(path + ".gz") else open(path, newline="")
    with f:
        r = csv.reader(f)
        head = next(r)
        return head, list(r)


def dialmap_fig(dialmap, out):
    cell = {(r["inst"], r["setting"]): r for r in csv.DictReader(open(dialmap))}
    fig, ax = plt.subplots(figsize=(10.5, 4.0))
    ax.grid(False)
    for i, inst in enumerate(ORDER):
        for j, s in enumerate(COLS):
            r = cell.get((inst, s))
            if r is None:
                color, txt, tc = NONE, "", INK2
            elif r["holds"] == "yes":
                color, txt, tc = GOOD, "ok", "#ffffff"
            else:
                n = len([x for x in r["not_converged"].split("; ") if x])
                color, txt, tc = CRIT, f"{n} trap", "#ffffff"
            ax.add_patch(plt.Rectangle((j + 0.05, i + 0.07), 0.9, 0.86, color=color, lw=0))
            ax.text(j + 0.5, i + 0.5, txt, ha="center", va="center", fontsize=7, color=tc)
    ax.set_xlim(0, len(COLS))
    ax.set_ylim(len(ORDER), 0)
    ax.set_xticks([j + 0.5 for j in range(len(COLS))])
    ax.set_xticklabels([short(s) for s in COLS], rotation=60, ha="right", fontsize=8)
    ax.set_yticks([i + 0.5 for i in range(len(ORDER))])
    ax.set_yticklabels([LABEL[k] for k in ORDER], fontsize=8)
    for sp in ax.spines.values():
        sp.set_visible(False)
    ax.set_title("The dial neighbourhood, Tier 3 (and 3S at IW1, C1, C2) at each setting\n"
                 "ok: every run CONVERGED or VACUOUS; n trap: runs in the subsistence trap; blank: not run",
                 fontsize=9, loc="left")
    fig.tight_layout()
    fig.savefig(out, dpi=150)
    plt.close(fig)


def i2_history_fig(a_recs, reg_history, out):
    h = next(x for x in json.load(open(reg_history)) if x["inst"].lower() == "i2")
    e = next(r for r in a_recs if r.get("set") == "history" and r.get("inst") == "i2")
    wm = [w["end_Dh"] for w in h["windows"]]
    we = [w["end_Dh"] for w in e.get("windows", [])]
    fig, ax = plt.subplots(figsize=(7.5, 3.2))
    ax.plot(range(len(wm)), wm, color=S[0], label="mirror (registered)")
    ax.plot(range(len(we)), we, color=S[1], linestyle="none", marker="o", markersize=4, label="engine")
    ax.axhline(1.0, color=INK2, linewidth=1.0, linestyle="--")
    ax.text(len(wm) - 1, 1.15, "tolerance", ha="right", va="bottom", fontsize=8, color=INK2)
    ax.set_yscale("log")
    ax.set_xlabel("window (1,500 ticks each)")
    ax.set_ylabel("D̂ at the window's last tick")
    ax.set_title("Wave A, I2's history cycle(land.power,1500,80): D̂ at each window's end (O111)",
                 fontsize=9, loc="left")
    ax.legend(loc="lower right")
    fig.tight_layout()
    fig.savefig(out, dpi=150)
    plt.close(fig)


def dial_ticks_fig(bcd, out):
    def pick(inst, set_, s):
        return [r for r in bcd if r.get("set") == set_ and r.get("inst") == inst and r.get("setting") == s
                and r.get("cmd") == "run"]

    series = [("C1, the registered rule (E9's control)", "c1", "ctl-dial", S[0], "o"),
              ("C1P, paced (E10)", "c1p", "dial", S[1], "s"),
              ("C2P, paced (E10)", "c2p", "dial", S[2], "D")]
    fig, ax = plt.subplots(figsize=(9.5, 3.8))
    xs = list(range(len(COLS)))
    for k, (lab, inst, set_, col, mk) in enumerate(series):
        ys, xx = [], []
        for x, s in zip(xs, COLS):
            rs = pick(inst, "ctl-tilt2" if s == "tilt.*=2" else set_, s)
            conv = [r["in_tol_from"] for r in rs if r.get("class") == "CONVERGED"]
            if not conv:
                continue
            y = statistics.median(conv)
            xo = x + (k - 1) * 0.2
            xx.append(xo)
            ys.append(y)
            n = sum(1 for r in rs if r.get("class") == "DIVERGED")
            if n:
                ax.annotate(f"{n}", (xo, y), textcoords="offset points", xytext=(0, 7), ha="center",
                            fontsize=7, color=INK2)
        ax.plot(xx, ys, linestyle="none", marker=mk, markersize=5, color=col, label=lab)
    ax.set_xticks(xs)
    ax.set_xticklabels([short(s) for s in COLS], rotation=60, ha="right", fontsize=8)
    ax.set_ylabel("Tier 3 ticks to tolerance, median")
    ax.set_ylim(280, 840)
    ax.set_title("The pace at the dial neighbourhood: Tier 3's median ticks (CONVERGED runs); a number "
                 "above a mark counts its runs in the trap", fontsize=9, loc="left")
    ax.legend(loc="upper center", ncol=3, fontsize=8)
    fig.tight_layout()
    fig.savefig(out, dpi=150)
    plt.close(fig)


def corner_fig(archive, out):
    runs = [("sw_trained__0.5", "workers.trained", S[0], "trained, sw[trained]=0.5", "-"),
            ("RW_2_", "workers.trained", S[1], "trained, RW(2)", "--"),
            ("xxd2", "workers.trained", S[2], "trained, x*/2", ":"),
            ("sw_master__0.2", "workers.master", S[3], "master, sw[master]=0.2", "-")]
    fig, ax = plt.subplots(figsize=(7.5, 3.4))
    for fn, pop, col, lab, ls in runs:
        head, rows = csv_rows(os.path.join(archive, "switch", "is1", "corner", fn, fn + ".csv"))
        ia = head.index(f"pool_{pop}")
        t, a = [], []
        for row in rows:
            v = float(row[ia])
            if v > 0:
                t.append(int(row[0]))
                a.append(v)
        ax.plot(t, a, color=col, label=lab, linewidth=1.6, linestyle=ls)
    ax.axhline(1e-300, color=INK2, linewidth=1.0, linestyle="--")
    ax.text(0, 1e-300, "1e-300", va="bottom", fontsize=8, color=INK2)
    ax.set_yscale("log")
    ax.set_xlabel("tick")
    ax.set_ylabel("pool share a (log)")
    ax.set_title("IS1: a displaced type returns to its wall, its share falling by e^(k·g) a tick "
                 "to the subnormal stall (E4)", fontsize=9, loc="left")
    ax.legend(loc="upper right", fontsize=8)
    fig.tight_layout()
    fig.savefig(out, dpi=150)
    plt.close(fig)


def ct2_kicks_fig(bcd, mirror_out, out):
    order = ["hold", "land.mach=0.44@dated", "land.mach=0.36@dated", "land.mach=0.8@dated",
             "land.mach=0.2@dated", "b.food=0.66@dated", "b.food=0.54@dated", "b.food=1.2@dated",
             "b.food=0.3@dated", "exit.To=21.45@dated", "exit.To=17.55@dated", "exit.To=39@dated",
             "exit.To=9.75@dated"]
    lab = {"hold": "base", "b.food=1.2@dated": "b.food ×2\n(Crowded)", "exit.To=17.55@dated": "commons ×0.9\n(Crowded)",
           "exit.To=9.75@dated": "commons ×0.5\n(Enclosed)"}
    kicks = {r["names"][0]: r for r in bcd if r.get("key") == "free" and r.get("inst") == "ct2" and r.get("cmd") == "kick"}
    mirror, cur = {}, None
    for line in open(mirror_out):
        m = re.match(r"CT2 (\S+), kicked", line)
        if m:
            cur = m.group(1)
        m = re.match(r"\s+(\S+)\s+([+-])\s+tail (\S+)", line)
        if m and cur:
            mirror.setdefault(cur, []).append(float(m.group(3)))
    fig, ax = plt.subplots(figsize=(9.5, 3.9))
    for x, t in enumerate(order):
        g = [k["gain_tail"] for k in kicks[t]["each"]]
        ax.plot([x - 0.12] * len(g), g, linestyle="none", marker="o", markersize=4, color=S[0],
                label="engine, each kick (the wave's kick set)" if x == 0 else None)
        mg = mirror.get(t, [])
        if mg:
            ax.plot([x + 0.12] * len(mg), mg, linestyle="none", marker="D", markersize=4, mfc="none", color=S[1],
                    label="mirror, the same kicks run the harness's way (diagnostic)" if t == "b.food=1.2@dated" else None)
    ax.axhline(1e-3, color=CRIT, linewidth=1.2, linestyle="--")
    ax.text(len(order) - 0.5, 1.2e-3, "bar: tail gain 1e-3", ha="right", va="bottom", fontsize=8, color=CRIT)
    ax.set_yscale("log")
    ax.set_ylim(5e-8, 3)
    ax.set_xticks(range(len(order)))
    ax.set_xticklabels([lab.get(t, t.replace("@dated", "")) for t in order], rotation=50, ha="right", fontsize=7)
    ax.set_ylabel("gain in the tail (last tenth of H)")
    ax.set_title("CT2's kick sets (E3), each kick's gain in the tail. At the two Crowded targets the kicked runs\n"
                 "settle above the rounding floor; at the Enclosed target the commons' own kicks stay at 1",
                 fontsize=9, loc="left")
    ax.legend(loc="center left", fontsize=8)
    fig.tight_layout()
    fig.savefig(out, dpi=150)
    plt.close(fig)


def ct2_slow_root_fig(archive, out):
    runs = [("dial/rate.x_0.75", "rate ×0.75 (root 0.999884)", S[0]),
            ("dial/buffer.x_0.75", "buffer ×0.75 (0.999846)", S[1]),
            ("L", "C2m, the battery (0.999846)", S[2])]
    fig, ax = plt.subplots(figsize=(7.5, 3.4))
    for d, lab, col in runs:
        head, rows = csv_rows(os.path.join(archive, "free", "ct2", d, "b.food_1.2_genesis", "b.food_1.2_genesis.csv"))
        it, idh = head.index("tick"), head.index("dhat")
        ax.plot([int(r[it]) for r in rows], [float(r[idh]) for r in rows], color=col, label=lab, linewidth=1.6)
    ax.axhline(1e-9, color=INK2, linewidth=1.0, linestyle="--")
    ax.text(141000, 1.3e-9, "1e-9, the scorer's end band", ha="right", va="bottom", fontsize=8, color=INK2)
    ax.set_yscale("log")
    ax.set_xlabel("tick (L = 141,000)")
    ax.set_ylabel("D̂ (gap over the tolerance)")
    ax.set_title("CT2 b.food=1.2@genesis: D̂ still falls at the mirror's root at L (A2 part c)",
                 fontsize=9, loc="left")
    ax.legend(loc="upper right", fontsize=8)
    fig.tight_layout()
    fig.savefig(out, dpi=150)
    plt.close(fig)


def main(a_path, bcd_path, dialmap, archive, repo):
    figs = os.path.join(repo, "docs", "probe", "figs")
    for d in ("families", "trap", "switch", "free"):
        os.makedirs(os.path.join(figs, d), exist_ok=True)
    a = records(a_path)
    b = records(bcd_path)
    here = os.path.dirname(os.path.abspath(__file__))
    dialmap_fig(dialmap, os.path.join(figs, "families", "dialmap.png"))
    i2_history_fig(a, os.path.join(repo, "docs", "probe", "families", "registered", "history.json"),
                   os.path.join(figs, "families", "i2_history.png"))
    dial_ticks_fig(b, os.path.join(figs, "trap", "dial_ticks.png"))
    corner_fig(archive, os.path.join(figs, "switch", "corner.png"))
    ct2_kicks_fig(b, os.path.join(here, "diag", "mirror_kick.out"), os.path.join(figs, "free", "ct2_kicks.png"))
    ct2_slow_root_fig(archive, os.path.join(figs, "free", "ct2_slow_root.png"))
    print("plots written")


if __name__ == "__main__":
    main(*sys.argv[1:6])
