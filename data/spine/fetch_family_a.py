# Fetch family A of the Breakpoint B pre-look: the carriers.
#
# Dated 2026-09-26. Branch spine-eyeball.
#
# Two carriers:
#   1. Bank of England, "A millennium of macroeconomic data for the UK", v3.1 (xlsx).
#   2. The Bouscasse-Nakamura-Steinsson replication archive, Harvard Dataverse,
#      doi:10.7910/DVN/5EXFLU, version 2.2 (CC0 1.0 at dataset level). File ids are
#      resolved from the pinned version's API record, never hard-coded.
#   Plus the BNS erratum and online appendix from the first author's site.
#
# Rules (docs/PLAN.md R11): pinned URLs; every byte checked against a pinned sha256
# (and, for Dataverse, against the MD5 the archive publishes). A mismatch stops the
# run: the new bytes are kept beside the old as *.mismatch and nothing is replaced.
# TLS is always verified. Up to three attempts per URL; attempts are recorded.
#
# Idempotent: a file already on disk with the pinned sha256 is not downloaded again.
# Its first retrieval date is kept in the sidecar <file>.fetch.json.
#
# Raw files go to $SPINE_ROOT/raw/family_a (override: SPINE_RAW). They stay out
# of the repository unless the manifest says the licence allows redistribution.
# The manifest is written next to this script: manifest_family_a.tsv.
#
# Run from the repository root, in the spine's venv (docs/spine/DATA_NOTES.md):
#   python data/spine/fetch_family_a.py

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
# The spine's cache: $SPINE_ROOT, else data/spine/.cache/ beside this script, which git ignores.
SPINE = pathlib.Path(os.environ.get("SPINE_ROOT") or HERE / ".cache")
RAW = pathlib.Path(os.environ.get("SPINE_RAW", SPINE / "raw")) / "family_a"
RAW.mkdir(parents=True, exist_ok=True)
MANIFEST = HERE / "manifest_family_a.tsv"
TODAY = datetime.date.today().isoformat()
UA = "rustyecon-spine-fetch/0.1 (public research data; family A)"
TRIES = 3

# %% licences, as each source states them (read 2026-09-26)
LIC_BOE = (
    "Bank of England website terms (https://www.bankofengland.co.uk/legal): copyright "
    "the Bank; download for personal or internal non-commercial use; re-use needs the "
    "Bank's permission; third-party material (Broadberry, Clark, Feinstein, Mitchell, "
    "others) stays with its owners. Not the OGL."
)
LIC_BNS_CC0 = "CC0 1.0 (Dataverse dataset licence, version 2.2, released 2025-04-16)"
LIC_BNS_DIGI = (
    LIC_BNS_CC0 + "; a BNS digitisation of published tables: the numbers are the "
    "cited authors' (flag them)"
)
LIC_BNS_THIRD = (
    "third-party original deposited inside a CC0 archive; the depositor cannot "
    "relicense it: treat as the original owner's terms"
)
LIC_SITE = "none stated (author's personal site)"

# %% the pinned sources
# key, url, local name, pinned sha256 (None until first fetch), licence,
# redistribute raw (yes/no/flag), derived series in repo (yes/no/flag), note
BOE_URL = (
    "https://www.bankofengland.co.uk/-/media/boe/files/statistics/research-datasets/"
    "a-millennium-of-macroeconomic-data-for-the-uk.xlsx"
)
BNS_VERSION_API = (
    "https://dataverse.harvard.edu/api/datasets/:persistentId/versions/2.2"
    "?persistentId=doi:10.7910/DVN/5EXFLU"
)
BNS_FILE_API = "https://dataverse.harvard.edu/api/access/datafile/{id}"

DIRECT = [
    dict(key="boe_millennium", url=BOE_URL, name="boe/a-millennium-of-macroeconomic-data-for-the-uk.xlsx",
         sha256="4c23dd392a498691eac92659aec283fb43f28118bd80511dc87fc595974195eb",
         licence=LIC_BOE, redistribute="no", derived="no",
         note="v3.1 as served by the Bank today; Last-Modified 2024-09-26"),
    dict(key="boe_legal_page", url="https://www.bankofengland.co.uk/legal", name="boe/legal.html",
         sha256=None, licence="the licence record itself", redistribute="no", derived="no",
         note="kept as the licence record; a web page, so its hash is recorded, not pinned"),
    dict(key="bns_site_erratum", url="https://paul-bouscasse.github.io/files/bns_malthus_erratum.pdf",
         name="bns_site/bns_malthus_erratum.pdf",
         sha256="0cd43282803a0751585d29a589da2be4bab036623ee9b52d5fe70becfbc48054",
         licence=LIC_SITE, redistribute="no", derived="no", note="erratum, June 2026"),
    dict(key="bns_site_appendix", url="https://paul-bouscasse.github.io/files/bns_malthus_appendix.pdf",
         name="bns_site/bns_malthus_appendix.pdf", sha256="8184e00a42fe2da5c40b518338a918fe29d2397a52aa51721325cd8809828a50",
         licence=LIC_SITE, redistribute="no", derived="no", note="online appendix"),
    dict(key="bns_site_estimates", url="https://paul-bouscasse.github.io/files/bns_estimates.xlsx",
         name="bns_site/bns_estimates.xlsx",
         sha256="ee663daab37899984b9148ef7825045f8a999f9620cacab31062470f45e7009d",
         licence=LIC_SITE + "; same bytes as the CC0 deposit (checked by sha256)",
         redistribute="via the CC0 deposit", derived="yes", note="compared with the deposit copy"),
]

# Dataverse files: (directoryLabel, filename) -> licence class. ?format=original for
# the one ingested tabular file.
BNS = [
    dict(key="bns_boe_copy", dir="bns_replication/data/raw", file="a-millennium-of-macroeconomic-data-for-the-uk.xlsx",
         sha256="70db9463379d5c06a65a67497f1810b08c22bebe89014be3e036d6d3fb81b6b4",
         licence=LIC_BNS_THIRD + " (Bank of England)", redistribute="no", derived="no",
         note="BNS downloaded it 2024-09-13 (README); the vintage diff runs against the Bank's copy"),
    dict(key="bns_readme", dir="bns_replication", file="README.pdf",
         sha256="d6e5166159e802edc74c9f7a1715f06f2ab4dee5324d1f6ab4b8b2dfd417907e",
         licence=LIC_BNS_CC0, redistribute="yes", derived="yes", note="provenance of every raw file (Appendix A)"),
    dict(key="bns_estimates", dir="bns_replication/output", file="bns_estimates.xlsx",
         sha256="ee663daab37899984b9148ef7825045f8a999f9620cacab31062470f45e7009d",
         licence=LIC_BNS_CC0, redistribute="yes", derived="yes", note="model output, not raw data"),
    dict(key="bns_malthus_data", dir="bns_replication/data/transformed", file="malthus_data.tab", original=True,
         sha256="437a9c7640709ad683d9bccc8b9a444efe51c5cbf7076e19876fb2cef81f304a",
         licence=LIC_BNS_CC0 + "; a BNS compilation of third-party series (Clark's Wages 2014 and his 2007 and 2010 "
         "population, the Bank's Broadberry population, Allen 2007, Humphries-Weisdorf 2019, Clark 2010 Table 13, "
         "Steinsson's MalthusFigures), built by dataset.R: the numbers are the cited authors' (flag them)",
         redistribute="flag", derived="flag",
         note="BNS's assembled estimation input; fetched as the original CSV; not committed (third-party numbers)"),
    dict(key="bns_allen07", dir="bns_replication/data/raw", file="allen07.xlsx",
         sha256="431564977567246e7cebfbf6692086d64520c1f453b1dd9490c07b9ab0b5f46b",
         licence=LIC_BNS_DIGI + " (Allen 2007, appendices I-II)", redistribute="flag", derived="flag",
         note="digitisation, not Allen's file"),
    dict(key="bns_clark10", dir="bns_replication/data/raw", file="clark10.xlsx",
         sha256="668927fd801888c136e7bc869c701c675f4e953c9e623338050d6440553bcf21",
         licence=LIC_BNS_DIGI + " (Clark 2010, tables 1, 13, 33)", redistribute="flag", derived="flag",
         note="digitisation"),
    dict(key="bns_clark07", dir="bns_replication/data/raw", file="clark07.xlsx", sha256="04a41d6c32670b2a1623020ee022b4233a38589f6c8c36afccfc29d619313da3",
         licence=LIC_BNS_DIGI + " (Clark 2007, table 9)", redistribute="flag", derived="flag",
         note="digitisation"),
    dict(key="bns_hw19", dir="bns_replication/data/raw", file="hw19.xlsx",
         sha256="6af7f532724e037b78c10307a0c3d124be73e233f417d73439977ba311cb4244",
         licence=LIC_BNS_DIGI + " (Humphries-Weisdorf 2019, tables A2-A3)", redistribute="flag", derived="flag",
         note="digitisation"),
    dict(key="bns_broadberry15", dir="bns_replication/data/raw", file="broadberry_etal15.xlsx",
         sha256="3e33f232b7436f009cfe5f2daf19a6c9f963674308b772d0b91b1e1d740493bf",
         licence=LIC_BNS_DIGI + " (Broadberry et al. 2015)", redistribute="flag", derived="flag",
         note="digitisation"),
    dict(key="bns_dataset_R", dir="bns_replication/code", file="dataset.R", sha256="e939f08ebde19688977fdce29c6f5d36a10c977e524ad8a2745eb275cdd40a8b",
         licence=LIC_BNS_CC0, redistribute="yes", derived="yes",
         note="BNS code that builds malthus_data.csv: the construction of every column"),
    dict(key="bns_clark_nni2015", dir="bns_replication/data/raw", file="England NNI - Clark - 2015.xlsx",
         sha256="214ae3b58c2da845b408cb18587ba1f02304a65c4fc609b0b2f6f9aa4320b090",
         licence=LIC_BNS_THIRD + " (Gregory Clark)", redistribute="no", derived="no",
         note="BNS copy (downloaded 2024-09-12 per README); family B fetches Clark's own copy; used here only to cross-check BoE A47/A48/A9 columns that cite Clark"),
    dict(key="bns_clark_wages2014", dir="bns_replication/data/raw", file="Wages 2014.xlsx",
         sha256="8959af72be11af1d574ebf69c9c31784bf385dd7a925f22aaa323e7d72a37781",
         licence=LIC_BNS_THIRD + " (Gregory Clark)", redistribute="no", derived="no",
         note="BNS copy; cross-check only (as above)"),
    dict(key="bns_malthusfigures", dir="bns_replication/data/raw", file="MalthusFigures.xlsx", sha256="b2ea55eb66848a2126bc2e1668774cfc73d1114092c3bb75b1aaa7388c022dff",
         licence=LIC_BNS_CC0 + "; Steinsson's lecture-note compilation of others' series (flag them)",
         redistribute="flag", derived="flag", note="Steinsson (2021) lecture notes, per the BNS README"),
]

# %% helpers
SESSION = requests.Session()
SESSION.headers["User-Agent"] = UA


def get(url, stream=False):
    """GET with TLS verified, up to TRIES attempts. Returns (response, attempts log)."""
    log = []
    for i in range(1, TRIES + 1):
        try:
            r = SESSION.get(url, stream=stream, timeout=120, allow_redirects=True)
            log.append(f"try{i}:{r.status_code}")
            if r.status_code == 200:
                return r, log
        except requests.RequestException as e:
            log.append(f"try{i}:{type(e).__name__}")
        time.sleep(2 * i)
    return None, log


def clean(url):
    """Drop the query of a pre-signed storage redirect: it carries a signature, and it expires."""
    return url.split("?")[0] + " (signed query dropped)" if "amazonaws.com" in url and "?" in url else url


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
        rec["final_url"] = clean(rec.get("final_url", ""))
        side.write_text(json.dumps(rec, indent=1), encoding="utf-8")
        rec["status"] = "cached; sha256 matches pin"
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
    status = "fetched; sha256 matches pin"
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
        if not pin:
            status = "fetched; unpinned (web page): sha256 recorded"
    rec = dict(key=src["key"], url=url, final_url=clean(r.url), retrieved=TODAY,
               bytes=(bad if "MISMATCH" in status else path).stat().st_size, sha256=sha,
               md5_check=md5_check, attempts=";".join(log), status=status, local=str(path),
               last_modified=r.headers.get("Last-Modified", ""))
    if "MISMATCH" not in status:
        side.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    return rec


# %% direct URLs
rows = []
for src in DIRECT:
    rec = fetch(src, src["url"])
    rows.append((src, rec))
    print(src["key"], rec["status"], rec.get("bytes"), rec.get("attempts"))

# %% the BNS archive: resolve ids from the pinned version record
r, log = get(BNS_VERSION_API)
if r is None:
    print("BNS version API BLOCKED", log)
    rows.append((dict(key="bns_version_api", licence=LIC_BNS_CC0, redistribute="-", derived="-",
                      note="the pinned version record; without it no file id resolves"),
                 dict(key="bns_version_api", url=BNS_VERSION_API, status="BLOCKED", attempts=";".join(log))))
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
        url = BNS_FILE_API.format(id=df["id"]) + ("?format=original" if src.get("original") else "")
        name = df.get("originalFileName") if src.get("original") else df["filename"]
        src = dict(src, name="bns/" + name)
        md5 = df["checksum"]["value"] if df.get("checksum", {}).get("type") == "MD5" else None
        rec = fetch(src, url, md5=md5)
        rec["dataverse_id"] = df["id"]
        rows.append((src, rec))
        print(src["key"], df["id"], rec["status"], rec.get("bytes"), rec.get("md5_check"), rec.get("attempts"))

# %% manifest
cols = ["key", "family", "url", "final_url", "retrieved", "bytes", "sha256", "md5_check", "licence",
        "redistribute_raw", "derived_in_repo", "status", "attempts", "last_modified", "local_path", "note"]
with open(MANIFEST, "w", encoding="utf-8", newline="\n") as f:
    f.write("\t".join(cols) + "\n")
    for src, rec in rows:
        vals = [src["key"], "A", rec.get("url", ""), rec.get("final_url", ""), rec.get("retrieved", ""),
                str(rec.get("bytes", "")), rec.get("sha256", ""), rec.get("md5_check", ""), src["licence"],
                src["redistribute"], src["derived"], rec.get("status", ""), rec.get("attempts", ""),
                rec.get("last_modified", ""),
                ("$SPINE_RAW/" + pathlib.Path(rec["local"]).relative_to(RAW.parent).as_posix()) if rec.get("local") else "",
                src["note"]]
        f.write("\t".join(v.replace("\t", " ").replace("\n", " ") for v in vals) + "\n")
print("manifest:", MANIFEST)

# %% stop and report
bad = [s["key"] + ": " + rec["status"] for s, rec in rows
       if rec["status"].startswith(("BLOCKED", "SHA256", "MD5", "NOT IN"))]
if bad:
    print("STOP AND REPORT:")
    for b in bad:
        print("  " + b)
    sys.exit(1)
