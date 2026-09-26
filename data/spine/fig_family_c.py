"""Family C diagnostic figure: land's share of income, every construction on one axis.

Dated 2026-09-26. A check on the C3 criterion (land's exit), not the eyeball sheet. It
plots the tidy CSVs written by extract_family_c.py and saves a PNG beside them
($SPINE_ROOT/series/family_c/). Nothing is fitted, averaged or spliced.

Run from the repository root, in the spine's venv (docs/spine/DATA_NOTES.md):
    python data/spine/fig_family_c.py
"""

# %% setup
import os

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import pandas as pd

# The spine's cache: $SPINE_ROOT, else data/spine/.cache/ beside this script, which git ignores.
SPINE = os.environ.get("SPINE_ROOT") or os.path.join(os.path.dirname(os.path.abspath(__file__)), ".cache")
OUT = os.environ.get("SPINE_SERIES", f"{SPINE}/series/family_c")
S1, S2, S3, S4 = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"  # validated, adjacent pairs
INK, INK2, GRID, SURF = "#0b0b0b", "#52514e", "#e4e3df", "#fcfcfb"


def rd(f, s):
    d = pd.read_csv(f"{OUT}/{f}")
    return d[d.series == s].sort_values("year")


def steps(ax, d, color, lw=2.0, **kw):
    """Draw period means as horizontal segments over their own years."""
    for i, r in enumerate(d.itertuples()):
        ax.plot([r.year, r.year_end + 1], [r.value, r.value], color=color, lw=lw,
                solid_capstyle="butt", label=kw.get("label") if i == 0 else None)


# %% data
c15 = rd("clark2015_land_share_annual.csv", "clark2015_land_share_of_factor_income")
c15 = c15[c15.year >= 1600]
t34 = rd("clark2010_t34.csv", "clark2010_t34_land_share")
t34 = t34[t34.year <= 1910]
c02 = rd("constr_clark2002_over_boe.csv", "constr_clark2002_rents_over_boe_england_gdp_fc")
stp = rd("constr_stamp_over_boe.csv", "constr_stamp_lands_gb_over_boe_gb_gdp")
a17 = rd("boe_a17_rent.csv", "boe_a17_rent_share_upper_bound")

# %% plot
fig, ax = plt.subplots(figsize=(10, 5.6), dpi=150)
fig.patch.set_facecolor(SURF)
ax.set_facecolor(SURF)
ax.plot(a17.year, a17.value, color=INK2, lw=1.5, ls=(0, (4, 3)),
        label="BoE A17 rent of land AND buildings / GDP(I), UK (upper bound, not a land share)")
ax.plot(c15.year, c15.value, color=S1, lw=1.6,
        label="Clark 2015: land rents / (NNI - indirect taxes), England, annual")
steps(ax, c02, S3, label="Constructed: Clark 2002 T8 rents / BoE England GDP (factor cost), decadal")
ax.plot(stp.year, stp.value, color=S4, lw=2.0,
        label="Constructed: Stamp Sch. A lands / BoE GB GDP, Great Britain, annual")
steps(ax, t34, S2, lw=2.6, label="Clark 2010 Table 34: land rent share, England, decadal")

ax.axvline(1869.5, color=INK2, lw=0.8)
ax.text(1872, 0.225, "1869: Clark's England\nspreadsheet ends", fontsize=8, color=INK2, va="top")
lab = dict(fontsize=8.5, color=INK, va="center")
ax.text(1612, 0.262, "Clark 2015", **lab)
ax.text(1702, 0.232, "Clark 2002 / BoE", **lab)
ax.text(1872, 0.010, "Stamp / BoE (GB)", **lab)
ax.text(1886, 0.066, "Clark T34", **lab)
ax.text(1856, 0.165, "A17 upper bound (UK)", fontsize=8.5, color=INK2, va="center")

ax.set_xlim(1600, 1921)
ax.set_ylim(0, 0.30)
ax.set_ylabel("share of income", color=INK2)
ax.set_title("Land's share of income, England and Britain, 1600-1920: every reachable construction",
             loc="left", fontsize=11, color=INK)
ax.grid(axis="y", color=GRID, lw=0.8)
for sp in ["top", "right"]:
    ax.spines[sp].set_visible(False)
for sp in ["left", "bottom"]:
    ax.spines[sp].set_color(GRID)
ax.tick_params(colors=INK2, labelsize=9)
ax.legend(loc="lower left", fontsize=7.5, frameon=False, labelcolor=INK)
fig.text(0.01, 0.01, "Sources: Clark (2015 spreadsheet; 2010 WP T34; 2002 WP T8), Stamp (1916) "
         "T. A4, BoE millennium v3.1 (A17, A21). Decadal values drawn over their own years. "
         "Nothing spliced. 2026-09-26.", fontsize=6.5, color=INK2)
fig.tight_layout(rect=(0, 0.03, 1, 1))
fig.savefig(f"{OUT}/fig_c3_land_share.png", facecolor=SURF)
print(f"{OUT}/fig_c3_land_share.png")
