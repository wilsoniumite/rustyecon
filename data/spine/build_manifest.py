"""Merge the family manifests into one: data/spine/MANIFEST.tsv.

Dated 2026-09-26. Branch spine-eyeball.

The three fetch families each wrote a manifest in their own column layout
(manifest_family_{a,b,c}.tsv). This script lays them side by side in one table with one
set of columns, and adds the sheet's own fetch (fetch_sheet.py). It changes no value: a
column a family did not write stays empty. The family manifests remain the record.

Columns: family, key, url, retrieved, bytes, sha256, licence, redistribute_raw,
derived_in_repo, status.

Run from the repository root, in the spine's venv (docs/spine/DATA_NOTES.md):
    python data/spine/build_manifest.py
"""

import json
import os

import pandas as pd

HERE = os.path.dirname(os.path.abspath(__file__))
# The spine's cache: $SPINE_ROOT, else data/spine/.cache/ beside this script, which git ignores.
SPINE = os.environ.get("SPINE_ROOT") or os.path.join(HERE, ".cache")
RAW_SHEET = os.environ.get("SPINE_RAW_SHEET", f"{SPINE}/raw/sheet")
COLS = ["family", "key", "url", "retrieved", "bytes", "sha256", "licence", "redistribute_raw",
        "derived_in_repo", "status"]

rows = []
for fam in ["a", "b"]:
    d = pd.read_csv(os.path.join(HERE, f"manifest_family_{fam}.tsv"), sep="\t", dtype=str)
    rows.append(d.assign(family=fam.upper())[COLS])
c = pd.read_csv(os.path.join(HERE, "manifest_family_c.tsv"), sep="\t", dtype=str)
c = c.rename(columns={"redistribute": "redistribute_raw"}).assign(family="C", derived_in_repo="")
rows.append(c[COLS])

side = os.path.join(RAW_SHEET, "gpih", "Allen_London_South_Eng_1259-1914.xlsx.fetch.json")
if os.path.exists(side):
    j = json.load(open(side, encoding="utf-8"))
    rows.append(pd.DataFrame([dict(
        family="sheet", key="gpih_allen_london_south_eng", url=j["url"], retrieved=j["retrieved"],
        bytes=str(j["bytes"]), sha256=j["sha256"],
        licence="none stated on the GPIH site or in the file; Allen's spreadsheet, mirrored by GPIH",
        redistribute_raw="no", derived_in_repo="no",
        status="fetched; sha256 matches the probe pin; used only to check the BoE carry of Allen (A47 AG-AK, BH)",
    )]))

m = pd.concat(rows, ignore_index=True).fillna("")
m.to_csv(os.path.join(HERE, "MANIFEST.tsv"), sep="\t", index=False, lineterminator="\n")
print(len(m), "rows ->", os.path.join(HERE, "MANIFEST.tsv"))
