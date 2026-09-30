"""D2.4 (label run, 2026-09-30): the results' small plots, docs/demo/v2/figs/. Reported, never
scored: they draw what score.py scored, engine beside mirror.

- fig1_ticks_to_tolerance.png: every battery run's ticks to tolerance, the engine's against the
  mirror's (years), and the relative difference's histogram with the 10% bar.
- fig2_long_run_dhat.png: each county's median and largest D-hat over the long run, the engine's
  against the mirror's, with the 5% band.
- fig3_long_run_heads.png: the herd against its equilibrium over 1750-1901 in six counties, the
  engine's every 13th tick and the mirror's.
- fig4_kicks.png: each county-date's kick-set g a year against the mirror's base local growth.

Colours: the dataviz skill's reference palette, slots 1 and 2 (engine blue, mirror orange), on a
light surface; text in ink tokens, never the series colour.

usage: python3 plots.py RESULTS GATHERED MIRROR_LONG_JSON REG FIGS
"""
import csv
import json
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

RES, G, MLONG, REG, FIGS = sys.argv[1:6]
os.makedirs(FIGS, exist_ok=True)
ENG, MIR = "#2a78d6", "#eb6834"
SURF, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
plt.rcParams.update({"figure.facecolor": SURF, "axes.facecolor": SURF, "axes.edgecolor": INK2,
                     "axes.labelcolor": INK, "xtick.color": INK2, "ytick.color": INK2,
                     "text.color": INK, "font.size": 8, "axes.grid": True, "grid.color": GRID,
                     "grid.linewidth": 0.6, "axes.spines.top": False,
                     "axes.spines.right": False, "legend.frameon": False})
NAMES = {r["county"]: r["name"] for r in csv.DictReader(open(os.path.join(REG,
                                                                         "battery-county-dates.csv")))}


def save(fig, name):
    fig.savefig(os.path.join(FIGS, name), dpi=110)
    plt.close(fig)


# fig1: ticks to tolerance
eng_runs = list(csv.DictReader(open(os.path.join(RES, "battery-runs.csv"))))
reg_runs = list(csv.DictReader(open(os.path.join(REG, "battery-runs.csv"))))
xs, ys, rel = [], [], []
for re_, rm in zip(eng_runs, reg_runs):
    assert (re_["county"], re_["year"], re_["run"]) == (rm["county"], rm["year"], rm["run"])
    if re_["ticks_to_tol"] and rm["ticks_to_tol"]:
        e, m = float(re_["ticks_to_tol"]), float(rm["ticks_to_tol"])
        xs.append(m / 52)
        ys.append(e / 52)
        rel.append(100 * (e / m - 1) if m > 0 else 0.0)
fig, ax = plt.subplots(1, 2, figsize=(7.2, 3.0))
ax[0].scatter(xs, ys, s=2, color=ENG, alpha=0.25, linewidths=0)
hi = max(xs + ys) * 1.05
ax[0].plot([0, hi], [0, hi], color=INK2, linewidth=0.8)
ax[0].set_xlabel("the mirror's years to tolerance")
ax[0].set_ylabel("the engine's years to tolerance")
ax[0].set_title(f"{len(xs):,} battery runs", fontsize=8, color=INK)
lo_r, hi_r = min(rel + [-12]), max(rel + [12])
ax[1].hist(rel, bins=80, range=(lo_r, hi_r), color=ENG, log=True)
for v in (-10, 10):
    ax[1].axvline(v, color=INK2, linewidth=0.8, linestyle="--")
ax[1].set_xlabel("engine over mirror − 1, per cent (bars at ±10%)")
ax[1].set_ylabel("runs")
fig.tight_layout()
save(fig, "fig1_ticks_to_tolerance.png")

# fig2: the long run's D-hat by county
lr = list(csv.DictReader(open(os.path.join(RES, "long-run.csv"))))
fig, ax = plt.subplots(1, 2, figsize=(7.2, 3.0))
for a, c, what in ((ax[0], "dhat_median", "median"), (ax[1], "dhat_max", "largest")):
    m = [float(r[f"{c}_m"]) for r in lr]
    e = [float(r[f"{c}_e"]) for r in lr]
    top = max(m + e) * 1.08
    a.fill_between([0, top], [0, 0.95 * top], [0, 1.05 * top], color=GRID, linewidth=0)
    a.plot([0, top], [0, top], color=INK2, linewidth=0.8)
    a.scatter(m, e, s=10, color=ENG, edgecolors=SURF, linewidths=0.6)
    a.set_xlabel(f"the mirror's {what} D-hat, 1750-1901")
    a.set_ylabel(f"the engine's {what} D-hat")
    a.set_title(f"{what.capitalize()} D-hat by county (93; grey band ±5%)", fontsize=8)
fig.tight_layout()
save(fig, "fig2_long_run_dhat.png")

# fig3: the herd against its equilibrium
SIX = ["county.lan", "county.lks", "county.gla", "county.dur", "county.sry", "county.sut"]
eng = {}
for r in csv.DictReader(open(os.path.join(G, "series.csv"))):
    if r["county"] in SIX:
        eng.setdefault(r["county"], []).append((int(r["tick"]), float(r["heads_over_oracle"])))
mir = {r["key"]: r["series"] for r in json.load(open(MLONG)) if r["key"] in SIX}
fig, axs = plt.subplots(2, 3, figsize=(7.2, 4.0), sharex=True, sharey=True)
for a, k in zip(axs.flat, SIX):
    t, v = zip(*eng[k])
    a.plot([1750 + x / 52 for x in t], v, color=ENG, linewidth=1.4, label="engine")
    ms = mir[k]
    a.plot([1750 + s[0] / 52 for s in ms], [s[5] for s in ms], color=MIR, linewidth=1.0,
           linestyle=(0, (3, 2)), label="mirror")
    a.axhline(1.0, color=INK2, linewidth=0.6)
    a.set_title(NAMES[k], fontsize=8)
axs[0][0].set_ylabel("heads / equilibrium")
axs[1][0].set_ylabel("heads / equilibrium")
axs[1][2].legend(loc="lower right", fontsize=7)
fig.tight_layout()
save(fig, "fig3_long_run_heads.png")

# fig4: the kick sets' slowest mode against the mirror's local growth
k = {(r["county"], r["year"]): r for r in csv.DictReader(open(os.path.join(G, "kicks.tsv")),
                                                          delimiter="\t")}
gm = {(r["county"], r["year"]): float(r["growth_per_year"])
      for r in csv.DictReader(open(os.path.join(REG, "growth.csv"))) if r["b_factor"] == "1.0"}
pts = [(gm[cd], float(k[cd]["g_year"])) for cd in gm if cd in k and k[cd]["g_year"] not in ("-", "")]
fig, ax = plt.subplots(figsize=(3.6, 3.0))
x, y = zip(*pts)
lo, hi = min(x + y) - 0.003, max(x + y) + 0.003
ax.plot([lo, hi], [lo, hi], color=INK2, linewidth=0.8)
ax.scatter(x, y, s=6, color=ENG, edgecolors=SURF, linewidths=0.4)
ax.set_xlim(lo, hi)
ax.set_ylim(lo, hi)
ax.set_xlabel("the mirror's base growth a year")
ax.set_ylabel("the engine's kick-set g a year")
ax.set_title(f"{len(pts)} county-dates: the kick envelope meets\n"
             "the rounding floor before the slowest mode leads", fontsize=7)
fig.tight_layout()
save(fig, "fig4_kicks.png")
print("wrote", sorted(os.listdir(FIGS)))
