# Extract and validate family B's series for the Breakpoint B pre-look.
#
# Dated 2026-09-26. Branch spine-eyeball. Nothing here is fitted.
#
# Reads only checked bytes:
#   - family B's raw files (fetch_family_b.py), checked against the pins below;
#   - family A's copies of the BNS digitisations and the Bank of England file, read
#     in place and checked against family A's pins (copied below from
#     manifest_family_a.tsv of 2026-09-26). Family B never downloads them twice.
#   - the second, hand-typed entry of Clark (2002) Table 8, typed from the rendered
#     PDF page into D:/rustyecon-spine/digitised/family_b/.
#
# Writes tidy CSVs, one per series, to D:/rustyecon-spine/series/family_b/ with the
# columns: series, year, period, freq, value, unit, source, file, sheet, column, kind,
# note. kind is "source" (the number as the source prints it) or "construction" (our
# arithmetic, named in the series id and the note). Also writes:
#   series_index_family_b.tsv   one row per series: what it is, coverage, licence flag
#   validation_family_b.tsv     one row per check: PASS / NOTE / FLAG / FAIL, and detail
#   crosscheck_*.csv            the tables behind the cross-source checks
#
# Nothing is averaged across sources and nothing is spliced. Where two sources cover
# the same years the differences are reported, not reconciled.
#
# Run: D:/rustyecon-spine/venv/Scripts/python.exe data/spine/extract_family_b.py

# %% imports and places
import hashlib
import os
import pathlib
import re
import sys
import warnings

import numpy as np
import pandas as pd
from pypdf import PdfReader

warnings.filterwarnings("ignore")
SPINE = pathlib.Path(os.environ.get("SPINE_ROOT", "D:/rustyecon-spine"))
RAWB = SPINE / "raw" / "family_b"
RAWA = SPINE / "raw" / "family_a"
DIGI = SPINE / "digitised" / "family_b"
OUT = SPINE / "series" / "family_b"
OUT.mkdir(parents=True, exist_ok=True)

# %% input files and their pins
INPUTS = {
    # family B (fetch_family_b.py)
    "nni15": (RAWB / "clark/England NNI - Clark - 2015.xlsx",
              "214ae3b58c2da845b408cb18587ba1f02304a65c4fc609b0b2f6f9aa4320b090"),
    "wages14": (RAWB / "clark/Wages 2014.xlsx",
                "8959af72be11af1d574ebf69c9c31784bf385dd7a925f22aaa323e7d72a37781"),
    "rent02": (RAWB / "clark/rentereh.pdf",
               "79fa99cb3fcc883b61d29357dea4542af0fbc736fbae856762a80578c57ae12f"),
    "macro09": (RAWB / "clark/Macroagg2009.pdf",
                "b4995bc7ae8ee1d7d59932c2efa199cbb6e9cd85103beea54bc9c23759ea1b92"),
    "gpih06": (RAWB / "gpih/England_1209-1914_(Clark).xls",
               "072c338b729e5c063473a9d8f6df203fe9f0537dcdbb113b098cf5bcdd3630d7"),
    "eff23": (RAWB / "bns/GDP-Efficiency 2023.xlsx",
              "0d7c31df0c9d47cb81218a37932751f6e9f8bf5ca3c7f51bbb7bc688c46ad1ab"),
    "datasetR": (RAWB / "bns/dataset.R",
                 "e939f08ebde19688977fdce29c6f5d36a10c977e524ad8a2745eb275cdd40a8b"),
    "fein88": (RAWB / "bns/feinstein88.xlsx",
               "72ccf51a52f4bf9b340a1aee2115a1615f167eb4842f9a9a75449e0da715d106"),
    # family A's bytes, read in place
    "hw19": (RAWA / "bns/hw19.xlsx", "6af7f532724e037b78c10307a0c3d124be73e233f417d73439977ba311cb4244"),
    "broad15": (RAWA / "bns/broadberry_etal15.xlsx",
                "3e33f232b7436f009cfe5f2daf19a6c9f963674308b772d0b91b1e1d740493bf"),
    "clark07": (RAWA / "bns/clark07.xlsx", "04a41d6c32670b2a1623020ee022b4233a38589f6c8c36afccfc29d619313da3"),
    "clark10": (RAWA / "bns/clark10.xlsx", "668927fd801888c136e7bc869c701c675f4e953c9e623338050d6440553bcf21"),
    "mfig": (RAWA / "bns/MalthusFigures.xlsx", "b2ea55eb66848a2126bc2e1668774cfc73d1114092c3bb75b1aaa7388c022dff"),
    "mdata": (RAWA / "bns/malthus_data.csv", "437a9c7640709ad683d9bccc8b9a444efe51c5cbf7076e19876fb2cef81f304a"),
    "boe": (RAWA / "boe/a-millennium-of-macroeconomic-data-for-the-uk.xlsx",
            "4c23dd392a498691eac92659aec283fb43f28118bd80511dc87fc595974195eb"),
    # the hand-typed second entry (no pin: it is ours; its hash is reported)
    "t8a_e2": (DIGI / "clark2002_table8a_entry2.csv", None),
    "t8b_e2": (DIGI / "clark2002_table8b_entry2.csv", None),
}

CHECKS = []  # (id, criterion, series, what, result, detail)


def check(cid, crit, series, what, result, detail):
    CHECKS.append((cid, crit, series, what, result, str(detail).replace("\t", " ").replace("\n", " ")))
    print(f"{cid} [{result}] {what}: {str(detail)[:220]}")


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


missing, bad = [], []
for k, (p, pin) in INPUTS.items():
    if not p.exists():
        missing.append(str(p))
        continue
    got = sha(p)
    if pin and got != pin:
        bad.append(f"{p.name}: pinned {pin[:12]} got {got[:12]}")
    if pin is None:
        check("V00", "all", k, f"hash of our own digitisation {p.name}", "NOTE", f"sha256 {got}")
if missing or bad:
    check("V01", "all", "inputs", "every input present and matching its pin", "FAIL",
          f"missing={missing} mismatched={bad}")
    print("STOP AND REPORT: inputs missing or changed")
    sys.exit(1)
check("V01", "all", "inputs", "every input present and matching its pin", "PASS", f"{len(INPUTS)} files")

# %% the tidy writer
SERIES = {}  # id -> dataframe
INDEX = []


def emit(sid, df, unit, source, file, sheet, column, kind, note, crit, redistribute="no", desc=""):
    """df has columns year, period, freq, value. Drops missing values. Records the index row."""
    d = df.dropna(subset=["value"]).copy()
    d["series"] = sid
    d["unit"] = unit
    d["source"] = source
    d["file"] = file
    d["sheet"] = sheet
    d["column"] = column
    d["kind"] = kind
    d["note"] = note
    d = d[["series", "year", "period", "freq", "value", "unit", "source", "file", "sheet", "column", "kind", "note"]]
    d.to_csv(OUT / f"{sid}.csv", index=False, float_format="%.10g")
    SERIES[sid] = d
    INDEX.append(dict(series=sid, criterion=crit, kind=kind, freq=",".join(sorted(d["freq"].unique())),
                      first=int(d["year"].min()), last=int(d["year"].max()), n=len(d), unit=unit,
                      source=source, file=file, sheet=sheet, column=column,
                      derived_in_repo=redistribute, description=desc or note))


def annual(years, values):
    y = pd.Series(years).astype(int).values
    return pd.DataFrame(dict(year=y, period=[str(v) for v in y], freq="annual", value=pd.to_numeric(values).values))


def decadal(decades, values):
    y = pd.Series(decades).astype(int).values
    return pd.DataFrame(dict(year=y, period=[f"{v}-{v + 9}" for v in y], freq="decade",
                             value=pd.to_numeric(values, errors="coerce").values))


# %% Clark, Wages 2014 (1209-1869): nominal and real day wages, cost of living
f_w = "clark/Wages 2014.xlsx"
wa = pd.read_excel(INPUTS["wages14"][0], "Annual", header=None)
assert wa.iloc[0, 0] == "Year" and wa.iloc[1, 0] == 1200 and wa.iloc[670, 0] == 1869, "Wages 2014 layout moved"
wa = wa.iloc[1:671].reset_index(drop=True)
wa.columns = ["year", "farm", "coal", "bldglab", "craft", "_f", "col", "_h", "r_farm", "r_bldglab", "r_craft"]
wa = wa.apply(pd.to_numeric, errors="coerce")
assert (wa["year"].diff().dropna() == 1).all()
wd = pd.read_excel(INPUTS["wages14"][0], "Decadal", header=None)
assert wd.iloc[0, 0] == "Decade" and wd.iloc[1, 0] == 1200 and wd.iloc[67, 0] == 1860
wd = wd.iloc[1:68].reset_index(drop=True)
wd.columns = wa.columns.str.replace("year", "decade")
wd = wd.apply(pd.to_numeric, errors="coerce")

# V36: a printed zero with no nominal wage behind it is a spreadsheet artefact, not a wage
zero_art = {}
for nom, real in [("farm", "r_farm"), ("bldglab", "r_bldglab"), ("craft", "r_craft")]:
    bad_ = wa.loc[(wa[real] == 0) & wa[nom].isna(), "year"].astype(int).tolist()
    zero_art[real] = bad_
    for y_ in bad_:
        dec_ = y_ // 10 * 10
        grp = wa[(wa["year"] // 10 * 10) == dec_]
        printed = wd.loc[wd["decade"] == dec_, real].iloc[0]
        check("V36", "C1", f"clark14_real_{nom}", f"real wage printed as 0 in {y_} where the nominal wage is blank",
              "FLAG", f"dropped from the annual series (named here, not silent). The Decadal sheet's {dec_}s value "
              f"{printed:.2f} is the mean including that zero; without it the mean is {grp.loc[grp[real] > 0, real].mean():.2f}. "
              f"The decadal series is kept as printed and carries this note. BNS w_{nom}_10 uses the decadal sheet")
        wa.loc[wa["year"] == y_, real] = np.nan
SRC_W = "Clark, Wages 2014.xlsx (UC Davis data page; documented in Clark 2010, Research in Economic History 27)"
for col, letter, name, what in [("farm", "B", "farm", "Farm Laborers, d/day"),
                                ("bldglab", "D", "bldglab", "Building Laborers, d/day"),
                                ("craft", "E", "craft", "Building Craftsmen, d/day")]:
    emit(f"clark14_nominal_{name}_annual", annual(wa["year"], wa[col]), "pence per day", SRC_W, f_w, "Annual",
         letter, "source", what, "C1,C2")
emit("clark14_col_annual", annual(wa["year"], wa["col"]), "index, 1860s=100", SRC_W, f_w, "Annual", "G", "source",
     "Cost of Living (1860s=100)", "C1,C2")
for col, letter, name, what in [("r_farm", "I", "farm", "Real Farm Wage (1860s=100)"),
                                ("r_bldglab", "J", "bldglab", "Real Building Laborer Wage (1860s=100)"),
                                ("r_craft", "K", "craft", "Real Building Craftsman Wage (1860s=100)")]:
    znote = "".join(f"; {y_} printed as 0 with no nominal wage: dropped (V36)" for y_ in zero_art[col])
    emit(f"clark14_real_{name}_annual", annual(wa["year"], wa[col]), "index, 1860s=100", SRC_W, f_w, "Annual",
         letter, "source", what + "; Clark's own deflation. Name: Clark real day wage" + znote, "C1,C2")
    znote = "".join(f"; the {y_ // 10 * 10}s value includes a spurious 0 for {y_} (V36)" for y_ in zero_art[col])
    emit(f"clark14_real_{name}_decade", decadal(wd["decade"], wd[col]), "index, 1860s=100", SRC_W, f_w, "Decadal",
         letter, "source", what + "; Clark's decadal sheet. Name: Clark real day wage" + znote, "C1,C2")
emit("clark14_col_decade", decadal(wd["decade"], wd["col"]), "index, 1860s=100", SRC_W, f_w, "Decadal", "G",
     "source", "Cost of Living (1860s=100), decadal sheet", "C1,C2")

# V04: Clark's real wage is nominal over his COL, rebased: I/(B/G) should be one constant per series
for nom, real in [("farm", "r_farm"), ("bldglab", "r_bldglab"), ("craft", "r_craft")]:
    k = (wa[real] / (wa[nom] / wa["col"])).dropna()
    check("V04", "C1,C2", f"clark14_real_{nom}_annual", "real wage = nominal / COL x constant (internal)",
          "PASS" if k.std() / k.mean() < 1e-6 else "FLAG",
          f"constant {k.mean():.6g}, relative spread {k.std() / k.mean():.2e}, n={len(k)}")

# V07: how the decadal sheet relates to the annual sheet (on the values as printed, zero included)
wa_printed = pd.read_excel(INPUTS["wages14"][0], "Annual", header=None).iloc[1:671].reset_index(drop=True)
wa_printed.columns = wa.columns[:11]
wa_printed = wa_printed.apply(pd.to_numeric, errors="coerce")
wa_printed["decade"] = (wa_printed["year"] // 10) * 10
wa["decade"] = (wa["year"] // 10) * 10
for col in ["farm", "col", "r_farm", "r_bldglab"]:
    m = wa_printed.groupby("decade")[col].mean()
    j = pd.concat([m.rename("annual_mean"), wd.set_index("decade")[col].rename("decadal")], axis=1).dropna()
    rel = (j["decadal"] / j["annual_mean"] - 1).abs()
    check("V07", "C1,C2", f"clark14 {col}", "Decadal sheet vs mean of the annual values in the decade",
          "PASS" if rel.max() < 1e-6 else "NOTE",
          f"max rel diff {rel.max():.3g} (decade {int(rel.idxmax())}); median {rel.median():.3g}. "
          + ("Decadal = annual mean" if rel.max() < 1e-6 else
             "Decadal is not the plain annual mean in some decades (Clark's own aggregation); both kept, labelled"))

# V31: are the early annual wages truly annual? Repeated values mark quinquennial or decadal blocks
for col in ["farm", "bldglab", "craft", "col"]:
    s = wa.set_index("year")[col]
    rep = (s == s.shift(1)) & s.notna()
    cg = (np.log(s).diff().diff().abs() < 1e-9) & ~rep
    by = pd.DataFrame(dict(rep=rep, cg=cg, n=s.notna())).groupby((s.index // 100) * 100).agg(
        rep=("rep", "mean"), cg=("cg", "mean"), n=("n", "sum"))
    check("V31", "C1", f"clark14 {col} annual", "annual values repeating the previous year / on a constant-growth line, by century",
          "NOTE", "; ".join(f"{c}s: n={int(r.n)} repeat {r.rep:.0%} const-growth {r.cg:.0%}" for c, r in by.iterrows() if c < 1900))

# %% Clark, England NNI 2015 (1209-1869): population, wages, rents, NNI, prices
f_n = "clark/England NNI - Clark - 2015.xlsx"
na = pd.read_excel(INPUTS["nni15"][0], "Annual", header=None)
cols = ["year", "pop_ew", "pop", "sh_farm", "w_farm", "w_nonfarm", "w_avg", "days", "wagebill", "rents",
        "houserents", "othcap", "allcap", "indtax", "nni", "_p", "px_de", "px_gdp", "px_col", "_t", "real_nni",
        "real_nni_n"]
na.columns = cols
# Excel rows 3-663 hold the annual block 1209-1869; rows 669-735 hold decade midpoints
blk = na.iloc[2:663].copy()
blk = blk.apply(pd.to_numeric, errors="coerce")
yrs = blk["year"].values
ok_block = (yrs[0] == 1209 and yrs[-1] == 1869 and (np.diff(yrs) == 1).all())
second = pd.to_numeric(na.iloc[663:, 0], errors="coerce").dropna()
check("V03", "all", "clark15 Annual sheet", "annual block is Excel rows 3-663 (1209-1869, consecutive); a second block below is excluded",
      "PASS" if ok_block else "FAIL",
      f"annual block {int(yrs[0])}-{int(yrs[-1])}, {len(yrs)} rows; second block {len(second)} rows "
      f"from {second.iloc[0]} to {second.iloc[-1]} (decade midpoints such as {second.iloc[1]}); not parsed as years")
if not ok_block:
    sys.exit(1)
nd = pd.read_excel(INPUTS["nni15"][0], "Decadal", header=None)
nd.columns = [c if c != "year" else "decade" for c in cols]
assert nd.iloc[2, 0] == 1200 and nd.iloc[68, 0] == 1860
nd = nd.iloc[2:69].apply(pd.to_numeric, errors="coerce").reset_index(drop=True)

SRC_N = "Clark, England NNI - Clark - 2015.xlsx (UC Davis data page; documented in Clark 2010, Research in Economic History 27)"
STEP = ("; before 1700 the annual values repeat one value per decade (a step function, see V32), so this is "
        "decadal information")
for c, letter, sid, unit, what in [
        ("pop", "C", "pop_england", "millions", "Pop England" + STEP + "; the 1590s repeat the 1580s value (V23)"),
        ("w_avg", "G", "wage_avg_male", "pence per day", "Male average Wage"),
        ("rents", "J", "land_rents", "GBP million", "Land rents" + STEP.replace("1700", "1800")),
        ("indtax", "N", "indirect_taxes", "GBP million", "Indirect Taxes"),
        ("nni", "O", "nni", "GBP million", "Net National Income"),
        ("px_col", "S", "col", "index, 1860s=100", "Price Index - Cost of Living")]:
    emit(f"clark15_{sid}_annual", annual(blk["year"], blk[c]), unit, SRC_N, f_n, "Annual (rows 3-663)", letter,
         "source", what, "C1" if c in ("pop", "w_avg", "px_col") else "C3")
    emit(f"clark15_{sid}_decade", decadal(nd["decade"], nd[c]), unit, SRC_N, f_n, "Decadal", letter, "source",
         what + ", decadal sheet", "C1" if c in ("pop", "w_avg", "px_col") else "C3")
pe = blk[["year", "pop_ew"]].dropna()
emit("clark15_pop_ew_census", annual(pe["year"], pe["pop_ew"]), "millions", SRC_N, f_n, "Annual (rows 3-663)", "B",
     "source", "Pop E&W: census years only (England and Wales, not England)", "C1")

# constructions on the 2015 spreadsheet (named, never presented as Clark's)
emit("cons_clark15_avgwage_over_col_annual", annual(blk["year"], 100 * blk["w_avg"] / blk["px_col"]),
     "pence per day at 1860s prices", SRC_N, f_n, "Annual (rows 3-663)", "100*G/S", "construction",
     "Construction: Clark average wage / Clark COL (G/S x 100)", "C1,C2")
emit("cons_clark15_avgwage_over_col_decade", decadal(nd["decade"], 100 * nd["w_avg"] / nd["px_col"]),
     "pence per day at 1860s prices", SRC_N, f_n, "Decadal", "100*G/S", "construction",
     "Construction: Clark average wage / Clark COL (G/S x 100), ratio of the decadal-sheet values", "C1,C2")
emit("cons_clark15_landshare_nni_annual", annual(blk["year"], blk["rents"] / blk["nni"]), "share", SRC_N, f_n,
     "Annual (rows 3-663)", "J/O", "construction", "Construction: Clark land rents / Clark NNI (J/O)", "C3")
emit("cons_clark15_landshare_nni_decade", decadal(nd["decade"], nd["rents"] / nd["nni"]), "share", SRC_N, f_n,
     "Decadal", "J/O", "construction", "Construction: Clark land rents / Clark NNI (J/O), decadal sheet", "C3")
emit("cons_clark15_landshare_netindtax_annual", annual(blk["year"], blk["rents"] / (blk["nni"] - blk["indtax"])),
     "share", SRC_N, f_n, "Annual (rows 3-663)", "J/(O-N)", "construction",
     "Construction: Clark land rents / (NNI - indirect taxes), the definition Clark (2010) states for Table 13",
     "C3")
emit("cons_clark15_landshare_netindtax_decade", decadal(nd["decade"], nd["rents"] / (nd["nni"] - nd["indtax"])),
     "share", SRC_N, f_n, "Decadal", "J/(O-N)", "construction",
     "Construction: Clark land rents / (NNI - indirect taxes), decadal sheet; Clark (2010) Table 13 definition",
     "C3")

# V05/V06: the two Clark spreadsheets agree where they carry the same variable
j = blk.set_index("year").join(wa.set_index("year"), rsuffix="_w")
d_col = (j["px_col"] / j["col"] - 1).abs().dropna()
d_farm = (j["w_farm"] / j["farm"] - 1).abs().dropna()
check("V05", "C1", "clark15 S vs clark14 G", "COL in the NNI file = COL in the wages file",
      "PASS" if d_col.max() < 1e-9 else "NOTE",
      f"max rel diff {d_col.max():.3g}, n={len(d_col)}; years differing by >1e-6: "
      f"{len(d_col[d_col > 1e-6])} ({', '.join(str(int(y)) for y in d_col[d_col > 1e-6].index[:12])}"
      f"{' ...' if len(d_col[d_col > 1e-6]) > 12 else ''}); NNI COL years {j['px_col'].notna().sum()}, wages COL years {j['col'].notna().sum()}")
check("V06", "C1", "clark15 E vs clark14 B", "male farm wage in the NNI file = farm wage in the wages file",
      "PASS" if d_farm.max() < 1e-9 else "NOTE",
      f"max rel diff {d_farm.max():.3g}, n={len(d_farm)}; NNI has {j['w_farm'].notna().sum()} farm-wage years, wages file {j['farm'].notna().sum()}")

# V08: the NNI decadal sheet against annual means
blk["decade"] = (blk["year"] // 10) * 10
for c in ["pop", "rents", "nni", "w_avg", "px_col"]:
    m = blk.groupby("decade")[c].mean()
    jj = pd.concat([m.rename("am"), nd.set_index("decade")[c].rename("dec")], axis=1).dropna()
    rel = (jj["dec"] / jj["am"] - 1).abs()
    check("V08", "C1,C3", f"clark15 {c}", "Decadal sheet vs mean of the annual block",
          "PASS" if rel.max() < 1e-6 else "NOTE",
          f"max rel diff {rel.max():.3g} (decade {int(rel.idxmax())}), median {rel.median():.3g}")

# V32: step functions in the annual NNI block (rents, population)
for c in ["rents", "pop", "nni", "w_avg"]:
    s = blk.set_index("year")[c]
    rep = (s == s.shift(1)) & s.notna()
    lg = np.log(s)
    const_growth = (lg.diff().diff().abs() < 1e-9) & ~rep
    by = pd.DataFrame(dict(rep=rep, cg=const_growth)).groupby((s.index // 100) * 100).mean()
    check("V32", "C1,C3", f"clark15 {c} annual", "share of years repeating the previous value / on a constant-growth line, by century",
          "NOTE", "; ".join(f"{int(c_)}s: repeat {r.rep:.0%}, const-growth {r.cg:.0%}" for c_, r in by.iterrows()))
# the Black Death window
bd = blk.set_index("year").loc[1340:1360, ["pop", "rents", "w_avg"]]
check("V33", "C1", "clark15 1340-1360", "annual population, rents and wage through the Black Death, as printed",
      "NOTE", "; ".join(f"{int(y)}: pop {r['pop']:.3f} rents {r['rents']:.3f} w {r['w_avg']:.3f}" for y, r in bd.iterrows()))

# %% GPIH copy of Clark's prices and wages, 1209-1914 (2006 vintage)
f_g = "gpih/England_1209-1914_(Clark).xls"
g = pd.read_excel(INPUTS["gpih06"][0], "main data sheet", header=None)
L = {"CD": 81, "CE": 82, "CF": 83}  # zero-based column index
for letter, j_ in L.items():
    assert str(g.iloc[5, j_]).startswith("Wage") and g.iloc[6, j_] == 1 and g.iloc[8, j_] == "day", g.iloc[4:9, j_].tolist()
gy = pd.to_numeric(g.iloc[10:, 0], errors="coerce")
gg = pd.DataFrame({"year": gy, "farm": pd.to_numeric(g.iloc[10:, 81], errors="coerce"),
                   "craft": pd.to_numeric(g.iloc[10:, 82], errors="coerce"),
                   "bldglab": pd.to_numeric(g.iloc[10:, 83], errors="coerce")}).dropna(subset=["year"])
gg["year"] = gg["year"].astype(int)
assert gg["year"].iloc[0] == 1209 and gg["year"].iloc[-1] == 1914
SRC_G = ("GPIH copy of Clark's English prices and wages, 1209-1914: 'Provided by Gregory Clark, 10 April 2006', "
         "re-formatted by Peter Lindert")
for c, letter, what in [("farm", "CD", "Wage, farm"), ("craft", "CE", "Wage, craft"), ("bldglab", "CF", "Wage, bldg laborer")]:
    emit(f"gpih_clark06_nominal_{c}_annual", annual(gg["year"], gg[c]), "pence per day", SRC_G, f_g,
         "main data sheet", letter, "source", what + " (2006 vintage of Clark's data)", "C2")

# V09: 2006 vintage (GPIH) against the 2014 vintage (Wages 2014), 1209-1869
vint = gg.set_index("year").join(wa.set_index("year")[["farm", "craft", "bldglab"]], rsuffix="_14")
rows = []
for c in ["farm", "craft", "bldglab"]:
    x = vint[[c, c + "_14"]].dropna()
    r = x[c] / x[c + "_14"] - 1
    dec = r.groupby((r.index // 10) * 10).mean()
    rows.append(pd.DataFrame({"series": c, "decade": dec.index, "mean_rel_diff_2006_vs_2014": dec.values}))
    big = dec[dec.abs() > 0.05]
    check("V09", "C2", f"gpih_clark06_nominal_{c} vs clark14_nominal_{c}",
          "2006 vintage vs 2014 vintage, same years (vintage revision, not averaged)",
          "PASS" if r.abs().max() < 1e-6 else "NOTE",
          f"n={len(x)}; identical years {int((r.abs() < 1e-9).sum())}; median |rel diff| {r.abs().median():.3g}; "
          f"1850-69 mean rel diff {r.loc[1850:1869].mean():+.3f}; decades with |mean diff|>5%: "
          f"{', '.join(f'{int(k)}s {v:+.2f}' for k, v in big.items()) or 'none'}")
pd.concat(rows).to_csv(OUT / "crosscheck_clark_vintages_2006_vs_2014.csv", index=False, float_format="%.6g")

# V10: the 1869/1870 seam inside the 2006 vintage, and coverage after 1869
for c in ["craft", "bldglab", "farm"]:
    s = gg.set_index("year")[c]
    post = s.loc[1870:1914].dropna()
    dl = np.log(s).diff()
    typical = dl.loc[1800:1869].abs().median()
    check("V10", "C2", f"gpih_clark06_nominal_{c}_annual", "coverage after 1869 and the 1869-70 step",
          "NOTE", f"{len(post)} values 1870-1914 ({post.index.min() if len(post) else '-'}-{post.index.max() if len(post) else '-'}); "
          f"log step 1869->70 {dl.get(1870, np.nan):+.3f} vs median |annual step| 1800-69 {typical:.3f}")
# V38: steps and repeated values in the nominal wages, 1850-1914 (the escape window's last stretch)
for c in ["craft", "bldglab"]:
    s = gg.set_index("year")[c].loc[1850:1914]
    dl = np.log(s).diff()
    big = dl[dl.abs() > 0.10]
    rep_ = s[(s == s.shift(1))].index.tolist()
    check("V38", "C2", f"gpih_clark06_nominal_{c}_annual", "steps above 10% and repeated values, 1850-1914", "NOTE",
          f"steps: {', '.join(f'{int(y)} {v:+.3f}' for y, v in big.items()) or 'none'}; years repeating the "
          f"previous value: {rep_ or 'none'}. Read as printed; not smoothed")
# V11: no Clark deflator after 1869 in these files
px_to_1914 = [str(g.iloc[5, j_]) for j_ in range(1, 81)
              if pd.to_numeric(g.iloc[10:, j_], errors="coerce").set_axis(gy.values).loc[1870:1914].notna().sum() > 0]
check("V11", "C2", "Clark real wage after 1869", "is a Clark cost-of-living index available after 1869?", "FLAG",
      f"No. Clark's COL (Wages 2014 G, NNI 2015 S) ends 1869. The GPIH file carries only these prices after 1869: "
      f"{', '.join(px_to_1914)}. farm2002.xls has farm-product prices to 1914, not a COL. A real Clark wage after "
      f"1869 needs another source's deflator: that would be a named cross-source construction; not built here")

# %% Clark (2002) Table 8, both tables: entry 1 from the PDF text layer, entry 2 typed from the page image
pdf = PdfReader(INPUTS["rent02"][0])
num = r"(-?\d+\.\d+|\d+)"
t8a_rows = []
for pg in (26, 27):
    for line in pdf.pages[pg].extract_text().splitlines():
        m = re.match(r"^\s*(\d{4}-\d{2})\s+(.*)$", line)
        if not m:
            continue
        vals = re.findall(num, m.group(2))
        if not vals:  # a label whose numbers the text layer puts on a later line (1910-12)
            continue
        if len(vals) == 6:
            t8a_rows.append([m.group(1)] + vals)
        elif len(vals) == 5:  # the North is blank (1500-39, 1560-79): see the table note
            t8a_rows.append([m.group(1), vals[0], vals[1], ""] + vals[2:])
        else:
            raise ValueError(f"Table 8a row not parsed: {line!r}")
# 1910-12's numbers sit on the line after the label in the text layer
txt28 = pdf.pages[27].extract_text()
m = re.search(r"1910-12\s*\n\s*\n?\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)", txt28)
if m and not any(r[0] == "1910-12" for r in t8a_rows):
    t8a_rows.append(["1910-12"] + list(m.groups()))
e1a = pd.DataFrame(t8a_rows, columns=["period_printed", "england", "se_pct", "north", "midlands", "south_east", "south_west"])
t8b_rows = []
for line in pdf.pages[42].extract_text().splitlines():
    m = re.match(r"^\s*(\d{4}-\d{2})\s+(.*)$", line)
    if m:
        vals = re.findall(num, m.group(2))
        assert len(vals) == 7, line
        t8b_rows.append([m.group(1)] + vals)
e1b = pd.DataFrame(t8b_rows, columns=["period_printed", "population_m", "share_males_ag", "males_ag_000",
                                      "farm_wages_gbp_m", "land_rents_local_taxes_gbp_m", "capital_payments_gbp_m",
                                      "net_farm_output_gbp_m"])
e1a.to_csv(DIGI / "clark2002_table8a_entry1_textlayer.csv", index=False)
e1b.to_csv(DIGI / "clark2002_table8b_entry1_textlayer.csv", index=False)
e2a = pd.read_csv(INPUTS["t8a_e2"][0], dtype=str).fillna("")
e2b = pd.read_csv(INPUTS["t8b_e2"][0], dtype=str).fillna("")
for name, e1, e2 in [("Table 8 (rent per acre by region)", e1a, e2a), ("Table 8 (net agricultural output)", e1b, e2b)]:
    e1 = e1.fillna("").astype(str)
    same_shape = e1.shape == e2.shape and list(e1.columns) == list(e2.columns)
    diffs = []
    if same_shape:
        for i in range(len(e1)):
            for c in e1.columns:
                a, b = e1.iat[i, e1.columns.get_loc(c)], e2.iat[i, e2.columns.get_loc(c)]
                if a != b and not (a and b and c != "period_printed" and float(a) == float(b)):
                    diffs.append(f"{e1.iat[i, 0]}/{c}: text={a!r} typed={b!r}")
    check("V14", "C3", f"clark02 {name}", "double entry: PDF text layer vs hand-typed from the page image",
          "PASS" if same_shape and not diffs else "FAIL",
          f"{len(e1)} rows x {e1.shape[1]} cols; cell differences: {len(diffs)} {diffs[:5]}")
    if not same_shape or diffs:
        print("STOP AND REPORT: the two entries of", name, "disagree")
        sys.exit(1)


def period_years(p):
    """'1500-39' -> (1500, 1539). The printed '1800-84' in Table 8a is 1800-04 (see check V15)."""
    a, b = p.split("-")
    a = int(a)
    b = int(a // 100 * 100 + int(b)) if len(b) == 2 else int(b)
    if p == "1800-84":
        b = 1804
    return a, b


t8a = e2a.copy()
t8b = e2b.copy()
for t in (t8a, t8b):
    t["y0"], t["y1"] = zip(*t["period_printed"].map(period_years))
    for c in t.columns[1:-2]:
        t[c] = pd.to_numeric(t[c], errors="coerce")
check("V15", "C3", "clark02 Table 8a", "period labels read as printed; one printed typo resolved by sequence",
      "NOTE", "Table 8a prints '1800-84' between 1795-99 and 1805-09; read as 1800-04 (typo in the source, also in the image)")
f_r = "clark/rentereh.pdf"
SRC_R = ("Clark (2002), 'Farmland rental values and agrarian history: England and Wales, 1500-1912', working "
         "version (rentereh.pdf, 'forthcoming ERoEH Dec 2002'); not the published article")


def periodic(t, col):
    return pd.DataFrame(dict(year=t["y0"], period=[f"{a}-{b}" for a, b in zip(t["y0"], t["y1"])], freq="period",
                             value=t[col]))


for col, sid, what in [("england", "england", "Average of England"), ("north", "north", "North"),
                       ("midlands", "midlands", "Midlands"), ("south_east", "south_east", "South East"),
                       ("south_west", "south_west", "South West")]:
    emit(f"clark02_t8a_rent_per_acre_{sid}", periodic(t8a, col), "GBP per acre, nominal", SRC_R, f_r,
         "p.27-28, Table 8: Land Rental Values by Period and Region", what, "source",
         f"{what}; double-entered (text layer + typed). "
         + ("The England average for 1500-39 and 1560-79 uses the three southern regions only, adjusted (table note)"
            if sid == "england" else ""), "C3")
emit("clark02_t8a_rent_se_pct", periodic(t8a, "se_pct"), "percent of level, relative to 1820-4", SRC_R, f_r,
     "p.27-28, Table 8", "Approximate Standard Error (%)", "source", "sampling error of the England average", "C3")
for col, sid, unit, what in [
        ("land_rents_local_taxes_gbp_m", "land_rents_and_local_taxes", "GBP million per year, nominal",
         "Total land rents and local taxes (includes local taxes; England)"),
        ("population_m", "population", "millions", "Population (m.)"),
        ("farm_wages_gbp_m", "farm_wages", "GBP million per year", "Farm Wages"),
        ("capital_payments_gbp_m", "capital_payments", "GBP million per year", "Assumed Capital Payments"),
        ("net_farm_output_gbp_m", "net_farm_output", "GBP million per year", "Nominal Net Farm Output")]:
    emit(f"clark02_t8b_{sid}", periodic(t8b, col), unit, SRC_R, f_r,
         "p.43, Table 8: Net Agricultural Output, England, 1500-1912", what, "source",
         what + "; double-entered (text layer + typed)", "C3")

# V16: Table 8b adds up (wages + rents and taxes + capital = output, to rounding)
res = t8b["farm_wages_gbp_m"] + t8b["land_rents_local_taxes_gbp_m"] + t8b["capital_payments_gbp_m"] - t8b["net_farm_output_gbp_m"]
check("V16", "C3", "clark02 Table 8b", "farm wages + rents and taxes + capital = net farm output (to rounding)",
      "PASS" if res.abs().max() <= 0.15 + 1e-9 else "NOTE",
      "adds up to rounding (|r|<=0.15) in the early periods; elsewhere output exceeds the three printed columns: "
      + ", ".join(f"{p_} {r_:+.1f}" for p_, r_ in zip(t8b['period_printed'], res) if abs(r_) > 0.15 + 1e-9)
      + ". Both entries agree, so this is the table as printed: the output column holds an item the three columns "
        "do not show. The rents column is used as printed")
# V17: England average lies within the regional range
lo = t8a[["north", "midlands", "south_east", "south_west"]].min(axis=1)
hi = t8a[["north", "midlands", "south_east", "south_west"]].max(axis=1)
out_rng = t8a.loc[(t8a["england"] < lo - 1e-9) | (t8a["england"] > hi + 1e-9), "period_printed"].tolist()
check("V17", "C3", "clark02 Table 8a", "England average within the regional min-max",
      "PASS" if not out_rng else "NOTE", f"outside the range: {out_rng or 'none'}")

# %% Clark (2010) Table 13: the working-paper PDF, the BNS digitisation, and the 2015 spreadsheet
m9 = PdfReader(INPUTS["macro09"][0])
t13 = []
for pg in (64, 65):
    for line in m9.pages[pg].extract_text().splitlines():
        mm = re.match(r"^\s*(1[2-8]\d0)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*$", line)
        if mm:
            t13.append([int(mm.group(1))] + [float(v) for v in mm.groups()[1:]])
t13 = pd.DataFrame(t13, columns=["decade", "nni", "wage", "land", "capital", "farm", "opw"]).set_index("decade")
assert list(t13.index) == list(range(1200, 1861, 10)), t13.index
emit("clark10pdf_t13_land_share_decade", decadal(t13.index, t13["land"]), "share",
     "Clark, 'The macroeconomic aggregates for England, 1209-1869', working version Macroagg2009.pdf (Nov 2009), Table 13",
     "clark/Macroagg2009.pdf", "p.65-66, Table 13", "Land Share in National Income", "source",
     "land rents / (NNI - indirect taxes), per Clark's text; working-paper version, parsed from the text layer",
     "C3")
bns13 = pd.read_excel(INPUTS["clark10"][0], "clark10_t13").set_index("Decade")
emit("bns_clark10_t13_land_share_decade", decadal(bns13.index, bns13["Land"]), "share",
     "BNS digitisation of Clark (2010) Table 13 (clark10.xlsx, CC0 as deposited; numbers are Clark's)",
     "family_a:bns/clark10.xlsx", "clark10_t13", "D (Land)", "source",
     "published-article version as digitised by BNS; BNS use it as land_share", "C3", redistribute="flag")

lsh = pd.DataFrame({
    "pdf_2009_T13": t13["land"],
    "bns_digitised_2010_T13": bns13["Land"],
    "clark15_J_over_O": (nd.set_index("decade")["rents"] / nd.set_index("decade")["nni"]),
    "clark15_J_over_O_minus_N": (nd.set_index("decade")["rents"] / (nd.set_index("decade")["nni"] - nd.set_index("decade")["indtax"])),
})
e23 = pd.read_excel(INPUTS["eff23"][0], "Efficiency", header=None)
e23y = pd.to_numeric(e23.iloc[2:, 0], errors="coerce")
e23a = pd.DataFrame({"year": e23y, "share_land": pd.to_numeric(e23.iloc[2:, 12], errors="coerce")})
e23a = e23a[(e23a["year"] == e23a["year"].round()) & e23a["year"].between(1209, 1869)].dropna()
assert e23.iloc[1, 12] == "share land", e23.iloc[1, 12]
lsh["clark23_share_land_ndp_decade_mean"] = e23a.groupby((e23a["year"] // 10) * 10)["share_land"].mean()
lsh.index.name = "decade"
lsh.to_csv(OUT / "crosscheck_land_share_constructions.csv", float_format="%.5f")
d_pdf_bns = (lsh["pdf_2009_T13"] - lsh["bns_digitised_2010_T13"]).abs()
check("V13", "C3", "Clark 2010 Table 13 land share", "working-paper PDF (2009) vs BNS digitisation of the published table",
      "PASS" if d_pdf_bns.max() < 0.0005 else "NOTE",
      f"max |diff| {d_pdf_bns.max():.4f} ({int(d_pdf_bns.idxmax())}s); decades differing: {int((d_pdf_bns > 0.0005).sum())}")
nni_diff = (t13["nni"] / bns13["NNI"] - 1).abs()
check("V13b", "C3", "Clark 2010 Table 13 NNI", "working-paper PDF vs BNS digitisation, NNI column",
      "PASS" if nni_diff.max() < 0.002 else "NOTE", f"max rel diff {nni_diff.max():.4f} ({int(nni_diff.idxmax())}s)")
for c in ["clark15_J_over_O", "clark15_J_over_O_minus_N", "clark23_share_land_ndp_decade_mean"]:
    dd = (lsh[c] - lsh["bns_digitised_2010_T13"]).dropna()
    check("V12", "C3", c, "land-share construction vs Clark (2010) Table 13 (published, BNS digitisation)", "NOTE",
          f"mean diff {dd.mean():+.4f}, max |diff| {dd.abs().max():.4f} ({int(dd.abs().idxmax())}s); "
          f"at 1750/1800/1850/1860: {', '.join(f'{lsh.loc[y, c]:.3f}' for y in (1750, 1800, 1850, 1860))} "
          f"vs T13 {', '.join(f'{lsh.loc[y, 'bns_digitised_2010_T13']:.3f}' for y in (1750, 1800, 1850, 1860))}")
emit("clark23_share_land_ndp_annual", annual(e23a["year"], e23a["share_land"]), "share of NDP",
     "Clark, GDP-Efficiency 2023.xlsx, shared privately with BNS (2023-02-18), deposited in the BNS archive",
     "bns/GDP-Efficiency 2023.xlsx", "Efficiency", "M (share land)", "source",
     "'Share farmland in NDP' (2023 vintage); third-party file, cross-check only", "C3")
# the 2023 workbook labels its land-rent column s./acre; the numbers equal the 2015 file's GBP m land rents
e23r = pd.Series(pd.to_numeric(e23.iloc[2:, 2], errors="coerce").values, index=e23y.values)
e23r = e23r[~e23r.index.duplicated()].loc[1209:1869].dropna()
ratio23 = e23r / blk.set_index("year")["rents"].reindex(e23r.index)
n23 = pd.to_numeric(pd.read_excel(INPUTS["eff23"][0], "nni", header=None).iloc[10:671, 1], errors="coerce").values
check("V29", "C1,C3", "clark23 'Land rent' vs clark15 J", "Clark's 2023 workbook against the 2015 spreadsheet",
      "FLAG", f"from 1500 the 2023 column equals the 2015 land rents in every year (max rel diff "
      f"{(ratio23.loc[1500:] - 1).abs().max():.1g}) though it is labelled 's./acre' and the 2015 column 'GBP m'. "
      f"Before 1500 the 2023 rents are lower: ratio 2023/2015 {ratio23.loc[:1499].min():.2f}-{ratio23.loc[:1499].max():.2f} "
      f"(1209: {e23r.loc[1209]:.2f} vs {blk.set_index('year')['rents'].loc[1209]:.2f}). The 2023 NNI also differs "
      f"(1209: {n23[0]:.2f} vs {blk['nni'].iloc[0]:.2f}; 1869: {n23[-1]:.1f} vs {blk['nni'].iloc[-1]:.1f}). Clark revised "
      f"medieval rents after 2015; his newest land share is far lower before 1500 (V12). Private file: cross-check only")

# %% BNS files: Humphries-Weisdorf (2019), Broadberry benchmarks, Clark (2007), MalthusFigures, malthus_data
hw = pd.read_excel(INPUTS["hw19"][0], "hw19_tabA2", header=None).iloc[2:61]
hw.columns = ["label", "n_obs", "cash", "benefits", "inc_annual", "real_annual", "day_pay", "inc_day", "real_day",
              "cpi_resp", "cpi_bare", "gdp_pc"]
hw["decade"] = hw["label"].str[:4].astype(int)
assert (hw["decade"].diff().dropna() == 10).all() and hw["decade"].iloc[0] == 1260 and hw["decade"].iloc[-1] == 1840
for c in hw.columns[1:-1]:
    hw[c] = pd.to_numeric(hw[c], errors="coerce")
SRC_HW = ("Humphries and Weisdorf (2019), 'Unreal wages', EJ 129, Table A2, as digitised by BNS "
          "(hw19.xlsx, CC0 as deposited; the numbers are HW's)")
emit("bns_hw19_real_income_annual_contracts_decade", decadal(hw["decade"], hw["real_annual"]),
     "ratio: implied annual income / annual basket cost (HW 2019)", SRC_HW, "family_a:bns/hw19.xlsx", "hw19_tabA2",
     "F (Real income, annual wages)", "source",
     "annual-contract real income; HW label decades '1260-70' for the 1260s", "C1", redistribute="flag")
emit("bns_hw19_real_income_day_wages_decade", decadal(hw["decade"], hw["real_day"]),
     "ratio: day pay x 250 / annual basket cost (HW 2019)", SRC_HW, "family_a:bns/hw19.xlsx", "hw19_tabA2",
     "I (Real income, day wages)", "source", "day-wage real income on HW's basket", "C1", redistribute="flag")
emit("bns_hw19_implied_income_annual_contracts_decade", decadal(hw["decade"], hw["inc_annual"]),
     "pence per year", SRC_HW, "family_a:bns/hw19.xlsx", "hw19_tabA2", "E (Implied income)", "source",
     "annual-contract income, cash plus implied benefits", "C1", redistribute="flag")
basket_a = hw["inc_annual"] / hw["real_annual"]
basket_d = hw["inc_day"] / hw["real_day"]
rel = (basket_a / basket_d - 1).abs()
check("V25", "C1", "bns_hw19 real incomes", "both real-income columns share one deflator (E/F = H/I)",
      "PASS" if rel.max() < 0.03 else "FLAG",
      f"implied basket cost {basket_a.iloc[0]:.0f} d/yr (1260s) to {basket_a.iloc[-1]:.0f} (1840s); max rel gap "
      f"between the two implied deflators {rel.max():.3f} (rounding of 2-decimal ratios); H = day pay x 250: "
      f"max rel gap {(hw['inc_day'] / (250 * hw['day_pay']) - 1).abs().max():.3f}")
# HW value the in-kind benefits of an annual contract at the basket cost. So col F is
# 1 + cash / basket by construction, and only the cash part can move. Named here; the
# cash share is kept as its own series (our arithmetic) so the sheet can show it.
ben_ratio = hw["benefits"] / basket_a
cash_ratio = hw["cash"] / basket_a
check("V25b", "C1", "bns_hw19 annual-contract real income", "implied benefits = basket cost, so F = 1 + cash/basket",
      "NOTE",
      f"implied benefits / implied basket cost (E/F) = {ben_ratio.min():.3f}-{ben_ratio.max():.3f} in all "
      f"{ben_ratio.notna().sum()} decades: F is 1 + cash/basket by construction. cash/basket "
      f"{cash_ratio.loc[hw['decade'].between(1260, 1360)].min():.2f}-{cash_ratio.loc[hw['decade'].between(1260, 1360)].max():.2f}"
      f" in the 1260s-1360s, {cash_ratio.iloc[-1]:.2f} in the 1840s; max |F - (1 + cash/basket)| "
      f"{(hw['real_annual'] - 1 - cash_ratio).abs().max():.3f}")
emit("bns_hw19_cash_annual_contracts_decade", decadal(hw["decade"], hw["cash"]),
     "pence per year", SRC_HW, "family_a:bns/hw19.xlsx", "hw19_tabA2", "C (Est. cash)", "source",
     "estimated cash pay of an annual contract", "C1", redistribute="flag")
emit("cons_hw19_cash_over_basket_decade", decadal(hw["decade"], cash_ratio),
     "ratio: cash pay / annual basket cost (basket = E/F)", SRC_HW, "family_a:bns/hw19.xlsx", "hw19_tabA2",
     "C / (E / F)", "construction",
     "our arithmetic: the part of HW's annual-contract real income that is not fixed at 1 by valuing benefits at "
     "the basket cost (F = 1 + this, to rounding)", "C1", redistribute="flag")

br = pd.read_excel(INPUTS["broad15"][0])
emit("bns_broadberry15_pop_benchmarks", pd.DataFrame(dict(year=br["year"].astype(int), period=br["year"].astype(str),
                                                          freq="benchmark", value=br["broad_pop_lf"])),
     "millions", "Broadberry et al. (2015), population benchmarks, as digitised by BNS (sheet bckol15_tab1.06)",
     "family_a:bns/broadberry_etal15.xlsx", "bckol15_tab1.06", "B (broad_pop_lf)", "source",
     "England population at benchmark years only; the only non-interpolated Broadberry points before 1541", "C1",
     redistribute="flag")
c7 = pd.read_excel(INPUTS["clark07"][0])
c7["decade"] = c7["Decade"].str[:4].astype(int)
best = [c for c in c7.columns if "Best" in c][0]
emit("bns_clark07_t9_pop_best_decade", decadal(c7["decade"], c7[best]), "millions",
     "Clark (2007), 'The long march of history', Table 9, as digitised by BNS (clark07.xlsx)",
     "family_a:bns/clark07.xlsx", "clark07_t9", "G ('Best' population estimate)", "source",
     "best estimate from the MPL and sample communities, 1250s-1590s", "C1", redistribute="flag")

mf = pd.read_excel(INPUTS["mfig"][0], "Wages and Population", header=None).iloc[2:]
mf = pd.DataFrame({"decade": pd.to_numeric(mf[0], errors="coerce"), "pop10": pd.to_numeric(mf[1], errors="coerce")}).dropna()
emit("bns_steinsson_clark10_pop_decade", decadal(mf["decade"], mf["pop10"]), "millions",
     "Steinsson (2021) lecture file MalthusFigures.xlsx in the BNS archive: Clark (2010) population, with two "
     "corrections (1590 and 1200) per its Readme", "family_a:bns/MalthusFigures.xlsx", "Wages and Population",
     "B (Pop England in millions)", "source", "BNS's pop_10 and, from 1540, their 'wrigley_pop'", "C1",
     redistribute="flag")
vr = pd.read_excel(INPUTS["mfig"][0], "Birth and Death Rates", header=None).iloc[1:]
assert str(pd.read_excel(INPUTS["mfig"][0], "Birth and Death Rates", header=None).iloc[0, 1]).startswith("Crude Death Rates W&S")
vr = pd.DataFrame({"year": pd.to_numeric(vr[0], errors="coerce"), "cdr": pd.to_numeric(vr[1], errors="coerce"),
                   "cdr_hmd": pd.to_numeric(vr[2], errors="coerce"), "cbr": pd.to_numeric(vr[3], errors="coerce"),
                   "cbr_hmd": pd.to_numeric(vr[4], errors="coerce")}).dropna(subset=["year"])
SRC_WS = ("Wrigley and Schofield (1981), Table A3.3, annual crude rates by back projection, digitised by I. Mico "
          "Millan (2017) in Steinsson's MalthusFigures.xlsx (BNS archive)")
emit("bns_ws1981_cbr_annual", annual(vr["year"], vr["cbr"]), "births per 1000", SRC_WS,
     "family_a:bns/MalthusFigures.xlsx", "Birth and Death Rates", "D (Crude Birth Rates W&S)", "source",
     "the only public copy of Wrigley-Schofield (1981) numbers found: vital rates, not population", "C1",
     redistribute="flag")
emit("bns_ws1981_cdr_annual", annual(vr["year"], vr["cdr"]), "deaths per 1000", SRC_WS,
     "family_a:bns/MalthusFigures.xlsx", "Birth and Death Rates", "B (Crude Death Rates W&S)", "source",
     "as above", "C1", redistribute="flag")
ov = vr.set_index("year").loc[1841:1871, ["cbr", "cbr_hmd", "cdr", "cdr_hmd"]].dropna()
check("V28", "C1", "bns_ws1981 crude rates", "W&S (1981) vs HMD (England and Wales) where both exist", "NOTE",
      f"overlap {int(ov.index.min())}-{int(ov.index.max())}: mean CBR W&S-HMD {(ov['cbr'] - ov['cbr_hmd']).mean():+.2f}, "
      f"mean CDR W&S-HMD {(ov['cdr'] - ov['cdr_hmd']).mean():+.2f} per 1000 (different geography: England vs E&W)")

md = pd.read_csv(INPUTS["mdata"][0])
x = md[md["decade"] >= 1540][["decade", "pop_10", "wrigley_pop"]].dropna()
check("V22", "C1", "BNS malthus_data wrigley_pop", "is BNS 'wrigley_pop' an independent Wrigley series?", "FLAG",
      f"No. dataset.R sets wrigley_pop = pop_10 for decades >= 1540 (pop_10 = Clark 2010 population via Steinsson's "
      f"file). Equal in {int((x['pop_10'] == x['wrigley_pop']).sum())} of {len(x)} decades. It is Clark's population "
      f"under a Wrigley name; not used as Wrigley")
x = md[["decade", "land_share"]].dropna().set_index("decade")["land_share"]
check("V24", "C3", "BNS malthus_data land_share", "= Clark (2010) Table 13 land share (BNS digitisation)",
      "PASS" if (x - bns13["Land"].reindex(x.index)).abs().max() < 1e-9 else "NOTE",
      f"max |diff| {(x - bns13['Land'].reindex(x.index)).abs().max():.3g}, n={len(x)}")
cmp = pd.concat([mf.set_index("decade")["pop10"], nd.set_index("decade")["pop"]], axis=1, keys=["steinsson", "clark15"]).dropna()
dif = cmp[(cmp["steinsson"] / cmp["clark15"] - 1).abs() > 1e-6]
c1590 = blk.set_index("year")["pop"]
check("V23", "C1", "Steinsson pop_10 vs Clark 2015 decadal pop", "where the lecture file departs from the 2015 spreadsheet",
      "FLAG", "; ".join(f"{int(k)}: {r.steinsson:.3f} vs {r.clark15:.3f}" for k, r in dif.iterrows())
      + f". In the 2015 annual block 1590-1599 repeat the 1580s value ({c1590.loc[1589]:.4f}); the Readme of "
        "Steinsson's file says 1590 was corrected after correspondence with Clark and 1200 from Clark (2010) Table 7. "
        "The 2015 series is kept as printed and flagged; the corrected one is bns_steinsson_clark10_pop_decade")

# %% Bank of England file (family A's bytes): population and England nominal GDP, for checks and C3's denominator
boe = pd.read_excel(INPUTS["boe"][0], sheet_name=["A2. Pop of Eng & GB 1086-1870", "A18. Population 1680+",
                                                   "A21. GDP per capita 1086+"], header=None)
a2 = boe["A2. Pop of Eng & GB 1086-1870"]
assert a2.iloc[7, 0] == "Year" and str(a2.iloc[7, 1]).startswith("Population of England")
a2 = pd.Series(pd.to_numeric(a2.iloc[8:, 1], errors="coerce").values, index=pd.to_numeric(a2.iloc[8:, 0], errors="coerce").values).dropna()
a18 = boe["A18. Population 1680+"]
assert str(a18.iloc[4, 10]).startswith("Wrigley (1997)") and str(a18.iloc[4, 25]).startswith("Wrigley (1997), Broadberry")
a18y = pd.to_numeric(a18.iloc[5:, 0], errors="coerce").values
a18k = pd.Series(pd.to_numeric(a18.iloc[5:, 10], errors="coerce").values, index=a18y).dropna() / 1000
a18z = pd.Series(pd.to_numeric(a18.iloc[5:, 25], errors="coerce").values, index=a18y).dropna() / 1000
a21 = boe["A21. GDP per capita 1086+"]
assert a21.iloc[3, 1] == "England" and a21.iloc[4, 1] == "Nominal GDP at market prices" and a21.iloc[4, 3] == "Nominal GDP at factor cost"
a21y = pd.to_numeric(a21.iloc[5:, 0], errors="coerce").values
gdp_mp = pd.Series(pd.to_numeric(a21.iloc[5:, 1], errors="coerce").values, index=a21y).dropna()
gdp_fc = pd.Series(pd.to_numeric(a21.iloc[5:, 3], errors="coerce").values, index=a21y).dropna()
gb_nom = pd.Series(pd.to_numeric(a21.iloc[5:, 12], errors="coerce").values, index=a21y).dropna()
uk_nom = pd.Series(pd.to_numeric(a21.iloc[5:, 26], errors="coerce").values, index=a21y).dropna()

# V20: Broadberry benchmarks (BNS) vs BoE A2 at the benchmark years
bb = br.set_index("year")["broad_pop_lf"]
d20 = (bb / a2.reindex(bb.index) - 1)
check("V20", "C1", "Broadberry benchmarks vs BoE A2 col B", "benchmark values equal the BoE annual series at those years",
      "PASS" if d20.abs().max() < 0.005 else "NOTE",
      "; ".join(f"{int(k)}: {v:+.3%}" for k, v in d20.items() if abs(v) >= 0.005) or f"all within 0.5% (max {d20.abs().max():.3%})")
# V21: BoE A2 between benchmarks is constant-growth interpolation
la = np.log(a2)
cg = (la.diff().diff().abs() < 1e-6)
share_cg = cg.loc[1086:1540].mean()
check("V21", "C1", "BoE A2 col B (Broadberry annual)", "annual values between benchmarks lie on constant-growth lines",
      "NOTE", f"{share_cg:.0%} of years 1086-1540 have zero second log-difference (interpolated); 1348-1351: "
      + ", ".join(f"{int(y)} {a2[y]:.3f}" for y in range(1347, 1353)) + ". Use benchmarks or decadal points only")

# V19: population across sources at decade starts
pop = pd.DataFrame({
    "clark15_C_decadal_sheet": nd.set_index("decade")["pop"],
    "clark15_C_annual_at_decade_start": blk.set_index("year")["pop"],
    "steinsson_pop10": mf.set_index("decade")["pop10"],
    "clark07_best": c7.set_index("decade")[best],
    "boe_A2_broadberry": a2,
    "boe_A18_K_wrigley97_census": a18k,
    "boe_A18_Z_wrigley97_broadberry": a18z,
    "clark02_t8b_population_period_start": t8b.set_index("y0")["population_m"],
}).loc[1200:1910]
pop = pop[pop.index % 10 == 0]
pop.index.name = "year"
pop.to_csv(OUT / "crosscheck_population_sources.csv", float_format="%.4f")
for c, ref in [("clark15_C_annual_at_decade_start", "boe_A18_K_wrigley97_census"),
               ("boe_A2_broadberry", "boe_A18_K_wrigley97_census"),
               ("clark07_best", "boe_A2_broadberry")]:
    for lo_, hi_ in [(1250, 1530), (1540, 1590), (1600, 1750), (1760, 1860), (1870, 1910)]:
        r = (pop[c] / pop[ref] - 1).loc[lo_:hi_].dropna()
        if len(r):
            check("V19", "C1", f"{c} vs {ref}", f"population, decade starts {lo_}-{hi_} (annual values, not decade means)",
                  "NOTE", f"n={len(r)}; mean {r.mean():+.2%}, min {r.min():+.2%} ({int(r.idxmin())}), "
                  f"max {r.max():+.2%} ({int(r.idxmax())})")
k_vs_z = (a18k / a18z - 1).dropna()
check("V19b", "C1", "BoE A18 K vs Z", "the two BoE England columns", "NOTE",
      f"identical in {int((k_vs_z.abs() < 1e-9).sum())} of {len(k_vs_z)} common years; max |diff| {k_vs_z.abs().max():.3%} "
      f"({int(k_vs_z.abs().idxmax())})")
bd_pop = pop.loc[1300:1400, ["clark15_C_annual_at_decade_start", "boe_A2_broadberry", "clark07_best"]]
check("V19c", "C1", "population around the Black Death", "decade starts 1300-1400, three sources", "NOTE",
      "; ".join(f"{int(y)}: " + "/".join(f"{v:.2f}" for v in r.values) for y, r in bd_pop.iterrows())
      + " (Clark15 / BoE-Broadberry / Clark07 best)")

# V27: what is 'England' nominal GDP in BoE A21 after 1870?
shr = (gdp_mp / uk_nom).loc[1801:1920].dropna()
shr_gb = (gdp_mp / gb_nom).loc[1700:1920].dropna()
check("V27", "C3", "BoE A21 England nominal GDP", "England GDP as a share of GB and UK(pre-1922) nominal GDP", "FLAG",
      f"England/GB: 1700 {shr_gb.get(1700, np.nan):.3f}, 1800 {shr_gb.get(1800, np.nan):.3f}, 1850 {shr_gb.get(1850, np.nan):.3f}, "
      f"1870 {shr_gb.get(1870, np.nan):.3f}, 1880 {shr_gb.get(1880, np.nan):.3f}, 1900 {shr_gb.get(1900, np.nan):.3f}, "
      f"1912 {shr_gb.get(1912, np.nan):.3f}; year-to-year sd of the share 1871-1912 {shr_gb.loc[1871:1912].diff().std():.4f}. "
      f"How the Bank builds England's GDP is not stated on the sheet: family A to document")

# %% construction: Clark (2002) land rents and local taxes over BoE England nominal GDP, period means
rows = []
for _, r in t8b.iterrows():
    yrs_ = range(int(r["y0"]), int(r["y1"]) + 1)
    fc = gdp_fc.reindex(yrs_)
    mp = gdp_mp.reindex(yrs_)
    rows.append(dict(y0=int(r["y0"]), y1=int(r["y1"]), rents_taxes=r["land_rents_local_taxes_gbp_m"],
                     gdp_fc_mean=fc.mean(), gdp_fc_n=int(fc.notna().sum()), gdp_mp_mean=mp.mean(),
                     n_years=len(yrs_)))
c3 = pd.DataFrame(rows)
c3["share_fc"] = c3["rents_taxes"] / c3["gdp_fc_mean"]
c3["share_mp"] = c3["rents_taxes"] / c3["gdp_mp_mean"]
assert (c3["gdp_fc_n"] == c3["n_years"]).all(), "BoE GDP missing inside a period"
per = pd.DataFrame(dict(year=c3["y0"], period=[f"{a}-{b}" for a, b in zip(c3["y0"], c3["y1"])], freq="period"))
SRC_C3 = "Clark (2002) Table 8 (net agricultural output) over Bank of England A21 (family A's bytes)"
emit("cons_clark02_rents_taxes_over_boe_england_gdp_fc_period", per.assign(value=c3["share_fc"]), "share",
     SRC_C3, "clark/rentereh.pdf + family_a:boe", "p.43 Table 8 / A21", "rents+local taxes / mean(A21 col D)",
     "construction",
     "Construction: Clark (2002) total land rents AND LOCAL TAXES, England, over the period mean of BoE A21 England "
     "nominal GDP at factor cost. Numerator includes local taxes; denominator is GDP, not NNI; after 1870 the BoE "
     "England GDP is the Bank's construction (V27)", "C3")
emit("cons_clark02_rents_taxes_over_boe_england_gdp_mp_period", per.assign(value=c3["share_mp"]), "share",
     SRC_C3, "clark/rentereh.pdf + family_a:boe", "p.43 Table 8 / A21", "rents+local taxes / mean(A21 col B)",
     "construction", "Construction: as the factor-cost variant, over BoE A21 England nominal GDP at market prices",
     "C3")
# V18: the overlap with Clark 2015 (1500-1869): same years, two constructions, reported side by side
ov_rows = []
for _, r in c3.iterrows():
    if r["y0"] > 1869:
        continue
    yrs_ = list(range(int(r["y0"]), int(min(r["y1"], 1869)) + 1))
    b_ = blk.set_index("year").reindex(yrs_)
    ov_rows.append(dict(period=f"{int(r['y0'])}-{int(r['y1'])}", clark02_rents_taxes=r["rents_taxes"],
                        clark15_land_rents_mean=b_["rents"].mean(), ratio_02_over_15=r["rents_taxes"] / b_["rents"].mean(),
                        clark02_pop=t8b.loc[t8b["y0"] == r["y0"], "population_m"].iloc[0],
                        clark15_pop_mean=b_["pop"].mean(),
                        cons_02_over_boe_gdp_fc=r["share_fc"], cons_15_J_over_O=(b_["rents"] / b_["nni"]).mean(),
                        cons_15_J_over_O_minus_N=(b_["rents"] / (b_["nni"] - b_["indtax"])).mean()))
ovd = pd.DataFrame(ov_rows)
ovd.to_csv(OUT / "crosscheck_c3_clark02_vs_clark15.csv", index=False, float_format="%.4f")
check("V18", "C3", "clark02 rents+taxes vs clark15 land rents", "same years, 1500-1869: numerator ratio and the two shares",
      "NOTE", f"ratio 2002/2015 numerator: median {ovd['ratio_02_over_15'].median():.2f} (range {ovd['ratio_02_over_15'].min():.2f}-"
      f"{ovd['ratio_02_over_15'].max():.2f}); 1860-69: 2002-cons {ovd.iloc[-1]['cons_02_over_boe_gdp_fc']:.3f} vs "
      f"J/O {ovd.iloc[-1]['cons_15_J_over_O']:.3f}. The two C3 constructions differ in level; they are not spliced")
post = c3[c3["y0"] >= 1860][["y0", "y1", "rents_taxes", "gdp_fc_mean", "share_fc"]]
check("V34", "C3", "cons_clark02 share after 1860", "the only direct land-share route past 1869, as computed", "NOTE",
      "; ".join(f"{int(r.y0)}-{int(r.y1)}: {r.rents_taxes:.1f}/{r.gdp_fc_mean:.0f} = {r.share_fc:.3f}" for r in post.itertuples()))
pk = [a18k.reindex(range(int(r.y0), int(r.y1) + 1)).mean() for r in c3.itertuples()]
p02 = (t8b["population_m"].values / np.array(pk) - 1)
check("V18c", "C1", "clark02 population vs BoE A18 K (Wrigley 1997 / census)", "period means, 1500-1912", "NOTE",
      "; ".join(f"{a}-{b}: {v:+.1%}" for a, b, v in zip(t8b["y0"], t8b["y1"], p02) if a in (1500, 1540, 1600, 1650, 1700, 1750, 1800, 1850, 1870, 1900, 1910)))
pop_ov = (ovd["clark02_pop"] / ovd["clark15_pop_mean"] - 1)
check("V18b", "C1", "clark02 population vs clark15 population", "period means, 1500-1869", "NOTE",
      f"mean {pop_ov.mean():+.2%}, max |diff| {pop_ov.abs().max():.2%} ({ovd.loc[pop_ov.abs().idxmax(), 'period']})")

# %% Feinstein and Wrigley-Schofield: what exists outside the BoE file
f88 = pd.read_excel(INPUTS["fein88"][0])
check("V30", "C2", "Feinstein outside the BoE file", "what the BNS archive carries", "FLAG",
      f"feinstein88.xlsx = Feinstein (1988) Table VIII capital stock (gross, net), decades {int(f88['decade'].min())}-"
      f"{int(f88['decade'].max())}; no wages or prices. Feinstein (1998) real wages reach us only through BoE A47/A48 "
      f"(family A); the JEH article is paywalled (manifest: feinstein98_fulltext). Not extracted")
check("V35", "C1", "Wrigley-Schofield (1981) population", "a public copy of the 1981 population series?", "FLAG",
      "None found. The book is not online; UK Data Service study 4491 holds the 404-parish counts (the input), behind a "
      "user account, not attempted. Public W&S numbers found: the annual crude birth and death rates (Table A3.3) in "
      "MalthusFigures.xlsx, extracted. Population of England from the Wrigley line: BoE A18 K/Z (Wrigley et al. 1997, "
      "family A) and Clark's own series. BNS 'wrigley_pop' is Clark's series renamed (V22)")

# V37: zeros and negatives in every emitted source series
for sid, d in SERIES.items():
    if d["kind"].iloc[0] != "source":
        continue
    z = d.loc[d["value"] <= 0, "year"].tolist()
    if z and not sid.startswith(("clark15_indirect_taxes", "clark02_t8a_rent_se_pct")):
        check("V37", "all", sid, "zero or negative values in a source series", "FLAG", f"years {z[:10]}")
check("V37", "all", "all source series", "zero or negative values (indirect taxes and standard errors may be 0)",
      "PASS" if not any(c[0] == "V37" for c in CHECKS) else "NOTE", "scan done")

# %% index and validation outputs
pd.DataFrame(INDEX).to_csv(OUT / "series_index_family_b.tsv", sep="\t", index=False)
pd.DataFrame(CHECKS, columns=["check", "criterion", "series", "what", "result", "detail"]).to_csv(
    OUT / "validation_family_b.tsv", sep="\t", index=False)
print(f"{len(SERIES)} series, {len(CHECKS)} checks -> {OUT}")
fails = [c for c in CHECKS if c[4] == "FAIL"]
if fails:
    print("STOP AND REPORT:", fails)
    sys.exit(1)
