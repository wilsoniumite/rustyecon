# Fetch family B of the Breakpoint B pre-look: rents and wages beyond the BoE set,
# and population.
#
# Dated 2026-09-26. Branch spine-eyeball.
#
# What this family fetches:
#   1. Clark's files on his UC Davis page: the 2015 macro-aggregates spreadsheet
#      (national income, land rents, population, wages, cost of living, 1209-1869),
#      the 2014 wages spreadsheet, the 2002 farm-price spreadsheet (1500-1914), the
#      2010 macro-aggregates paper (working version, Macroagg2009.pdf) and the 2002
#      farmland-rent paper (rentereh.pdf, Table 8 runs to 1912).
#   2. The GPIH copy of Clark's prices and wages, 1209-1914 (nominal wages to 1914).
#   3. Files of the BNS archive (Harvard Dataverse doi:10.7910/DVN/5EXFLU,
#      version 2.2) beyond family A's list as first written: BNS's own dataset.R (how
#      their series were built), the Feinstein (1988) digitisation, the BNS copies of
#      Clark's two spreadsheets (to test byte identity with Clark's site), and two
#      further workbooks listed in their README. Family A later also fetched dataset.R
#      and the two Clark copies; the bytes are identical (same sha256), so the overlap
#      is harmless. For the rest of the archive (hw19, clark07, clark10, Broadberry,
#      MalthusFigures, malthus_data) family B reads family A's bytes in place, checked
#      against family A's pins (extract_family_b.py), and does not download them.
#   4. Reachability records for sources that have no public file: Feinstein (1998)
#      full text and Wrigley-Schofield (1981).
#
# Rules (docs/PLAN.md R11): pinned URLs; every byte checked against a pinned sha256
# (and, for Dataverse, against the MD5 the archive publishes). A mismatch stops the
# run: the new bytes are kept beside the old as *.mismatch and nothing is replaced.
# TLS is always verified; never verify=False. The UC Davis faculty host sometimes
# serves an expired certificate from one back end: up to three attempts per URL, and
# every attempt is recorded in the manifest.
#
# Idempotent: a file already on disk with the pinned sha256 is not downloaded again.
# Its first retrieval record is kept in the sidecar <file>.fetch.json.
#
# Raw files go to D:/rustyecon-spine/raw/family_b (override: SPINE_RAW). None of
# them may be redistributed except BNS's own CC0 code; the manifest says which.
# The manifest is written next to this script: manifest_family_b.tsv.
#
# Run: D:/rustyecon-spine/venv/Scripts/python.exe data/spine/fetch_family_b.py

# %% imports and places
import datetime
import hashlib
import json
import os
import pathlib
import sys
import time

import requests

HERE = pathlib.Path(__file__).resolve().parent
RAW = pathlib.Path(os.environ.get("SPINE_RAW", "D:/rustyecon-spine/raw")) / "family_b"
RAW.mkdir(parents=True, exist_ok=True)
MANIFEST = HERE / "manifest_family_b.tsv"
TODAY = datetime.date.today().isoformat()
UA = "rustyecon-spine-fetch/0.1 (public research data; family B)"
TRIES = 3

# %% licences, as each source states them (read 2026-09-26)
LIC_CLARK = (
    "none stated on Clark's data page or in the file; posted for public download; "
    "copyright stays with the author"
)
LIC_CLARK_PAPER = "none stated; author's working-paper PDF; copyright stays with the author (and publisher)"
LIC_GPIH = (
    "none stated on the GPIH site or in the file; the file says 'Provided by Gregory "
    "Clark, 10 April 2006', re-formatted by Peter Lindert"
)
LIC_BNS_CC0 = "CC0 1.0 (Dataverse dataset licence, version 2.2, released 2025-04-16)"
LIC_BNS_DIGI = (
    LIC_BNS_CC0 + "; a BNS digitisation of a published table: the numbers are the "
    "cited author's (flag them)"
)
LIC_BNS_THIRD = (
    "third-party original deposited inside a CC0 archive; the depositor cannot "
    "relicense it: treat as the original owner's terms"
)

# %% the pinned sources
# key, url, local name, pinned sha256 (None until the first fetch), licence,
# redistribute raw (yes/no/flag), derived series in repo (yes/no/flag), note
UCD = "https://faculty.econ.ucdavis.edu/faculty/gclark/"
DIRECT = [
    dict(key="clark_nni2015", url=UCD + "English%20Data/England%20NNI%20-%20Clark%20-%202015.xlsx",
         name="clark/England NNI - Clark - 2015.xlsx",
         sha256="214ae3b58c2da845b408cb18587ba1f02304a65c4fc609b0b2f6f9aa4320b090",
         licence=LIC_CLARK, redistribute="no", derived="no",
         note="Clark (2010) macro aggregates, 2015 revision: population, wages, land rents, NNI, cost of living, 1209-1869"),
    dict(key="clark_wages2014", url=UCD + "English%20Data/Wages%202014.xlsx",
         name="clark/Wages 2014.xlsx",
         sha256="8959af72be11af1d574ebf69c9c31784bf385dd7a925f22aaa323e7d72a37781",
         licence=LIC_CLARK, redistribute="no", derived="no",
         note="Clark nominal and real day wages (farm, building labourer, craftsman), 1209-1869"),
    dict(key="clark_farm2002", url=UCD + "English%20Data/farm2002.xls",
         name="clark/farm2002.xls",
         sha256="5eec71f854f51781027bde13db42c88e9c4c46918a44a1e42d49c9f0a0489e4c",
         licence=LIC_CLARK, redistribute="no", derived="no",
         note="Clark (2004) prices of 22 farm products and farm price indices, 1500-1914; not used by the eyeball"),
    dict(key="clark_macroagg_pdf", url=UCD + "papers/Macroagg2009.pdf",
         name="clark/Macroagg2009.pdf",
         sha256="b4995bc7ae8ee1d7d59932c2efa199cbb6e9cd85103beea54bc9c23759ea1b92",
         licence=LIC_CLARK_PAPER, redistribute="no", derived="no",
         note="Clark (2010) working version; Table 13 checks the land share"),
    dict(key="clark_rent2002_pdf", url=UCD + "papers/rentereh.pdf",
         name="clark/rentereh.pdf",
         sha256="79fa99cb3fcc883b61d29357dea4542af0fbc736fbae856762a80578c57ae12f",
         licence=LIC_CLARK_PAPER, redistribute="no", derived="no",
         note="Clark (2002) farmland rents, working version; the two tables numbered 8 run to 1912"),
    dict(key="clark_datapage", url=UCD + "data.html", name="clark/data.html", sha256=None,
         licence="the licence record itself", redistribute="no", derived="no",
         note="kept as the licence and provenance record; a web page, so its hash is recorded, not pinned"),
    dict(key="gpih_clark_england", url="https://gpih.ucdavis.edu/files/England_1209-1914_%28Clark%29.xls",
         name="gpih/England_1209-1914_(Clark).xls",
         sha256="072c338b729e5c063473a9d8f6df203fe9f0537dcdbb113b098cf5bcdd3630d7",
         licence=LIC_GPIH, redistribute="no", derived="no",
         note="Clark's prices and wages as of 2006; craft and building-labourer wages run to 1914"),
]

BNS_VERSION_API = (
    "https://dataverse.harvard.edu/api/datasets/:persistentId/versions/2.2"
    "?persistentId=doi:10.7910/DVN/5EXFLU"
)
BNS_FILE_API = "https://dataverse.harvard.edu/api/access/datafile/{id}"
BNS = [
    dict(key="bns_dataset_R", dir="bns_replication/code", file="dataset.R",
         sha256="e939f08ebde19688977fdce29c6f5d36a10c977e524ad8a2745eb275cdd40a8b",
         licence=LIC_BNS_CC0, redistribute="yes", derived="yes",
         note="BNS code that builds malthus_data.csv; the record of their constructions"),
    dict(key="bns_feinstein88", dir="bns_replication/data/raw", file="feinstein88.xlsx",
         sha256="72ccf51a52f4bf9b340a1aee2115a1615f167eb4842f9a9a75449e0da715d106", licence=LIC_BNS_DIGI + " (Feinstein 1988, table VIII cols 4 and 7)",
         redistribute="flag", derived="flag",
         note="the only Feinstein file outside the BoE set: capital stock, not wages"),
    dict(key="bns_clark_nni_copy", dir="bns_replication/data/raw", file="England NNI - Clark - 2015.xlsx",
         sha256="214ae3b58c2da845b408cb18587ba1f02304a65c4fc609b0b2f6f9aa4320b090",
         licence=LIC_BNS_THIRD + " (Clark)", redistribute="no", derived="no",
         note="BNS downloaded it 2024-09-12; byte identity with Clark's site is the check"),
    dict(key="bns_clark_wages_copy", dir="bns_replication/data/raw", file="Wages 2014.xlsx",
         sha256="8959af72be11af1d574ebf69c9c31784bf385dd7a925f22aaa323e7d72a37781",
         licence=LIC_BNS_THIRD + " (Clark)", redistribute="no", derived="no",
         note="BNS downloaded it 2024-09-12; byte identity with Clark's site is the check"),
    dict(key="bns_howgrowthbegan", dir="bns_replication/data/raw", file="HowGrowthBeganFiguresTables.xlsx",
         sha256="375f2baafc42c312027c75c1105ee411157febfb96c4d4070260af7681fe49c4", licence=LIC_BNS_CC0 + "; Steinsson (2024) figure workbook; compiles others' series (flag them)",
         redistribute="flag", derived="flag",
         note="inspected for population and wage series; see the extract notes"),
    dict(key="bns_gdp_efficiency", dir="bns_replication/data/raw", file="GDP-Efficiency 2023.xlsx",
         sha256="0d7c31df0c9d47cb81218a37932751f6e9f8bf5ca3c7f51bbb7bc688c46ad1ab", licence=LIC_BNS_THIRD + " (Clark, private correspondence 2023-02-18)",
         redistribute="no", derived="no",
         note="Clark workbook shared privately with BNS; inspected only"),
]

# Sources with no public file. Each is probed and recorded; nothing stands in for it.
REACH = [
    dict(key="feinstein98_fulltext", url="https://doi.org/10.1017/S0022050700021100",
         licence="journal copyright (Journal of Economic History 58(3), 1998)",
         expect="article page", note="Feinstein (1998) 'Pessimism perpetuated'"),
    dict(key="ws1981_ukda_4491", url="http://doi.org/10.5255/UKDA-SN-4491-1",
         licence="UK Data Service terms; not read: the catalogue page renders by script and DataCite lists no rights",
         expect="catalogue page", note="Wrigley-Schofield (1981) input: parish counts for 404 parishes, not the population series"),
]

# %% helpers
SESSION = requests.Session()
SESSION.headers["User-Agent"] = UA


def get(url, stream=False):
    """GET with TLS verified, up to TRIES attempts. Returns (response or None, attempts log)."""
    log = []
    for i in range(1, TRIES + 1):
        try:
            r = SESSION.get(url, stream=stream, timeout=120, allow_redirects=True)
            log.append(f"try{i}:{r.status_code}")
            if r.status_code == 200:
                return r, log
        except requests.exceptions.SSLError as e:
            msg = "certificate has expired" if "expired" in str(e) else "SSLError"
            log.append(f"try{i}:SSLError({msg})")
        except requests.RequestException as e:
            log.append(f"try{i}:{type(e).__name__}")
        time.sleep(2 * i)
    return None, log


def digest(path, algo):
    h = hashlib.new(algo)
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def fetch(src, url, md5=None):
    """Download src to RAW/src['name'] unless present with the pinned sha256."""
    path = RAW / src["name"]
    path.parent.mkdir(parents=True, exist_ok=True)
    side = path.with_name(path.name + ".fetch.json")
    pin = src.get("sha256")
    if path.exists() and side.exists() and pin and digest(path, "sha256") == pin:
        rec = json.loads(side.read_text(encoding="utf-8"))
        rec["status"] = "cached"
        return rec
    r, log = get(url, stream=True)
    if r is None:
        return dict(key=src["key"], url=url, final_url="", retrieved=TODAY, bytes="", sha256="",
                    md5_check="", attempts=";".join(log), status="BLOCKED", local=str(path))
    tmp = path.with_name(path.name + ".part")
    with open(tmp, "wb") as f:
        for chunk in r.iter_content(1 << 20):
            f.write(chunk)
    sha = digest(tmp, "sha256")
    md5_check = ""
    if md5:
        got = digest(tmp, "md5")
        md5_check = "ok" if got == md5 else f"MISMATCH dataverse={md5} got={got}"
    bad = None
    if pin and sha != pin:
        bad = path.with_name(path.name + ".mismatch")
        tmp.replace(bad)
        status = f"SHA256 MISMATCH pinned={pin} got={sha}; kept as {bad.name}; nothing replaced"
    elif md5_check.startswith("MISMATCH"):
        bad = path.with_name(path.name + ".mismatch")
        tmp.replace(bad)
        status = f"MD5 {md5_check}; kept as {bad.name}"
    else:
        tmp.replace(path)
        status = "fetched; sha256 matches pin" if pin else "fetched; no pin: sha256 recorded"
    rec = dict(key=src["key"], url=url, final_url=r.url, retrieved=TODAY,
               bytes=(bad or path).stat().st_size, sha256=sha,
               md5_check=md5_check, attempts=";".join(log), status=status, local=str(path),
               last_modified=r.headers.get("Last-Modified", ""))
    if bad is None:
        side.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    return rec


# %% direct URLs
rows = []
for src in DIRECT:
    rec = fetch(src, src["url"])
    rows.append((src, rec))
    print(src["key"], "|", rec["status"], "|", rec.get("bytes"), "|", rec.get("attempts"))

# %% the BNS archive: resolve ids from the pinned version record
r, log = get(BNS_VERSION_API)
if r is None:
    print("BNS version API BLOCKED", log)
    for src in BNS:
        rows.append((src, dict(key=src["key"], url=BNS_VERSION_API, status="BLOCKED (version API)",
                               attempts=";".join(log))))
else:
    ver = r.json()["data"]
    (RAW / "bns").mkdir(exist_ok=True)
    (RAW / "bns" / "dataverse_version_2.2.json").write_text(json.dumps(ver, indent=1), encoding="utf-8")
    assert (ver["versionNumber"], ver["versionMinorNumber"]) == (2, 2), "not version 2.2"
    assert ver["license"]["rightsIdentifier"] == "CC0-1.0", ver["license"]
    index = {(f.get("directoryLabel", ""), f["dataFile"]["filename"]): f["dataFile"] for f in ver["files"]}
    for src in BNS:
        df = index.get((src["dir"], src["file"]))
        if df is None:
            rows.append((src, dict(key=src["key"], url="", status="NOT IN VERSION 2.2", attempts="")))
            print(src["key"], "NOT IN VERSION 2.2")
            continue
        url = BNS_FILE_API.format(id=df["id"])
        src = dict(src, name="bns/" + df["filename"])
        md5 = df["checksum"]["value"] if df.get("checksum", {}).get("type") == "MD5" else None
        rec = fetch(src, url, md5=md5)
        rec["dataverse_id"] = df["id"]
        rows.append((src, rec))
        print(src["key"], df["id"], "|", rec["status"], "|", rec.get("bytes"), "|", rec.get("md5_check"),
              "|", rec.get("attempts"))

# %% reachability of the sources with no public file (recorded, never substituted)
for src in REACH:
    rr, log = get(src["url"])
    body = rr.text if rr is not None else ""
    if src["key"] == "feinstein98_fulltext":
        gated = rr is not None and "/article/abs/" in rr.url and "Get access" in body
        status = ("BLOCKED: abstract page only; full text behind 'Get access' (paywall). "
                  "Values reach us only through the BoE file (family A)") if gated or rr is None else \
            "UNEXPECTED: page reachable without the access gate; inspect by hand"
    else:
        status = ("NOT FETCHED: the DOI resolves to the UK Data Service catalogue (study 4491, "
                  "'Parish Register Aggregate Analyses, 1662-1811; 404 Data'). Downloads there go "
                  "through a user account; none is created or used here. It holds parish counts, "
                  "the input to Wrigley-Schofield, not their population series") if rr is not None else "BLOCKED"
    rec = dict(key=src["key"], url=src["url"], final_url=rr.url if rr is not None else "", retrieved=TODAY,
               bytes="", sha256="", md5_check="", attempts=";".join(log), status=status, local="",
               last_modified="")
    rows.append((dict(src, redistribute="no", derived="no"), rec))
    print(src["key"], "|", status, "|", ";".join(log))

# %% manifest
cols = ["key", "family", "url", "final_url", "retrieved", "bytes", "sha256", "md5_check", "licence",
        "redistribute_raw", "derived_in_repo", "status", "attempts", "last_modified", "local_path", "note"]
with open(MANIFEST, "w", encoding="utf-8", newline="\n") as f:
    f.write("\t".join(cols) + "\n")
    for src, rec in rows:
        final = rec.get("final_url", "")
        if "X-Amz-" in final:  # signed S3 redirect: the signature expires; keep the stable part
            final = final.split("?")[0]
        vals = [src["key"], "B", rec.get("url", ""), final, rec.get("retrieved", ""),
                str(rec.get("bytes", "")), rec.get("sha256", ""), rec.get("md5_check", ""), src["licence"],
                src["redistribute"], src["derived"], rec.get("status", ""), rec.get("attempts", ""),
                rec.get("last_modified", ""),
                ("$SPINE_RAW/" + pathlib.Path(rec["local"]).relative_to(RAW.parent).as_posix()) if rec.get("local") else "",
                src["note"]]
        f.write("\t".join(v.replace("\t", " ").replace("\n", " ") for v in vals) + "\n")
print("manifest:", MANIFEST)

# %% stop and report
bad = [s["key"] + ": " + rec["status"] for s, rec in rows
       if rec["status"].startswith(("BLOCKED", "SHA256", "MD5", "NOT IN", "UNEXPECTED"))
       and s["key"] not in {x["key"] for x in REACH}]
if bad:
    print("STOP AND REPORT:")
    for b in bad:
        print("  " + b)
    sys.exit(1)
