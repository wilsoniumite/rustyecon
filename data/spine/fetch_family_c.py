"""Family C fetch: land share and factor incomes, 19th century (Breakpoint B pre-look).

Dated 2026-09-26. Label fetch-3.

Sources: Clark's 2015 spreadsheet and the working papers behind it (2010, 2002), the BNS
digitisation of Clark (2010), the Bank of England millennium file, Stamp (1916) from
archive.org (public domain), and two documentation PDFs (Allen 2009; the Bogart note on the
enclosure shapefile).

What it does. It downloads each pinned file into $SPINE_ROOT/raw/family_c/,
checks its sha256 against the pin, writes a sidecar <file>.fetch.json, and writes the
family manifest data/spine/manifest_family_c.tsv. It is idempotent: a file already on
disk with the pinned hash is not downloaded again.

Rules it keeps. TLS is always verified (the UC Davis host sometimes serves an expired
certificate; the script retries, up to three attempts, and records each one). A hash
mismatch stops the script: the new bytes are kept beside the old as <file>.new and
nothing is overwritten. A source that cannot be reached is recorded as blocked. No other
source is fetched in its place (R11).

Raw files stay out of the repository. The licence column says why.

The spine's cache: $SPINE_ROOT, else data/spine/.cache/ beside this script, which git ignores.

Run from the repository root, in the spine's venv (docs/spine/DATA_NOTES.md):
    python data/spine/fetch_family_c.py
"""

import hashlib
import json
import os
import sys
import time

import requests

RETRIEVED = "2026-09-26"
HERE = os.path.dirname(os.path.abspath(__file__))
# The spine's cache: $SPINE_ROOT, else data/spine/.cache/ beside this script, which git ignores.
SPINE = os.environ.get("SPINE_ROOT") or os.path.join(HERE, ".cache")
RAW = os.environ.get("SPINE_RAW", f"{SPINE}/raw/family_c")
MANIFEST = os.path.join(HERE, "manifest_family_c.tsv")
UA = "rustyecon-spine-fetch/0.1 (public-data research; contact via github.com/wilsontomass)"

CLARK = "https://faculty.econ.ucdavis.edu/faculty/gclark"
BNS_DOI = "doi:10.7910/DVN/5EXFLU"
DV = "https://dataverse.harvard.edu"

LIC_CLARK = "none stated on the author's page; academic copyright assumed"
LIC_BOE = ("BoE website terms (https://www.bankofengland.co.uk/legal): personal or internal "
           "non-commercial use; re-use needs the Bank's permission; third-party series stay "
           "with their owners")
LIC_BNS = "CC0 1.0 (Harvard Dataverse, dataset version 2.2); third-party numbers flagged"

# Each row: key, url (or dataverse filename), local name, pinned sha256, licence,
# redistribute raw (yes/no), what we use it for.
SOURCES = [
    dict(key="clark_nni2015",
         url=CLARK + "/English%20Data/England%20NNI%20-%20Clark%20-%202015.xlsx",
         local="clark/England NNI - Clark - 2015.xlsx",
         sha256="214ae3b58c2da845b408cb18587ba1f02304a65c4fc609b0b2f6f9aa4320b090",
         licence=LIC_CLARK, redistribute="no",
         use="factor incomes 1209-1869: wage bill, land rents, capital income, indirect taxes, NNI"),
    dict(key="clark_macroagg2010_pdf",
         url=CLARK + "/papers/Macroagg2009.pdf",
         local="clark/Macroagg2009.pdf",
         sha256="b4995bc7ae8ee1d7d59932c2efa199cbb6e9cd85103beea54bc9c23759ea1b92",
         licence=LIC_CLARK, redistribute="no",
         use="Clark (2010) text: definitions of land rents and NNI; Table 13 factor shares"),
    dict(key="clark_rent2002_pdf",
         url=CLARK + "/papers/rentereh.pdf",
         local="clark/rentereh.pdf",
         sha256="79fa99cb3fcc883b61d29357dea4542af0fbc736fbae856762a80578c57ae12f",
         licence=LIC_CLARK, redistribute="no",
         use="Clark (2002) working paper: second Table 8, total land rents and local taxes, 1500-1912"),
    dict(key="bns_clark10",
         dataverse=(BNS_DOI, "clark10.xlsx"),
         local="bns/clark10.xlsx",
         sha256="668927fd801888c136e7bc869c701c675f4e953c9e623338050d6440553bcf21",
         licence=LIC_BNS, redistribute="flag (a CC0 digitisation of Clark 2010 tables; the numbers are Clark's; not committed)",
         use="Clark (2010) Table 13 factor shares by decade, as digitised by BNS"),
    dict(key="boe_millennium",
         url=("https://www.bankofengland.co.uk/-/media/boe/files/statistics/research-datasets/"
              "a-millennium-of-macroeconomic-data-for-the-uk.xlsx"),
         local="boe/a-millennium-of-macroeconomic-data-for-the-uk.xlsx",
         sha256="4c23dd392a498691eac92659aec283fb43f28118bd80511dc87fc595974195eb",
         licence=LIC_BOE, redistribute="no",
         use="A17 rent (land and buildings) and GDP(I) 1855-1920; A21 England nominal GDP"),
    dict(key="allen_engelspause_pdf",
         url="https://www.nuff.ox.ac.uk/Users/Allen/engelspause.pdf",
         local="allen/engelspause.pdf",
         sha256="97f2c976cb36005521cc8109acebcda860fce46150bd2e0fce939139eb64a4dd",
         licence="none stated; journal copyright (EEH 2009) on the published version",
         redistribute="no",
         use="documentation only: Allen's rent share 1770-1913 is printed as a figure (Fig. 2), no table"),
    dict(key="bogart_enclosure_note",
         url="https://sites.socsci.uci.edu/~dbogart/enclosuredatanote_may12017.pdf",
         local="turner/enclosuredatanote_may12017.pdf",
         sha256="57566dd9b8b30e7a92403c4b760ce9de67c0812b7d9d99a5614fe11268a78c66",
         licence="none stated", redistribute="no",
         use="documentation only: describes the Tate (1978, ed. Turner) parish shapefile; no public download"),
    dict(key="stamp1916_pdf",
         url="https://archive.org/download/britishincomespr00stamuoft/britishincomespr00stamuoft.pdf",
         local="stamp/britishincomespr00stamuoft.pdf",
         sha256="a14150629e5f2a249d93bdf586b53099057378e6e2351b27636581c55d4951a9", sha1="7bb9e53f0f5d7abb99c0117ac70e67836b7a5d26",
         licence="public domain (Stamp d. 1941; archive.org: NOT_IN_COPYRIGHT); scan by "
                 "University of Toronto (Robarts)",
         redistribute="yes (public domain)",
         use="Stamp (1916) Table A4 p.49: Schedule A 'Lands (including tithes)', E&W, Scotland, "
             "Ireland, UK, 1842-3 to 1913-14; the source Clark cites for land rents after 1842"),
    dict(key="stamp1916_djvu_txt",
         url="https://archive.org/download/britishincomespr00stamuoft/britishincomespr00stamuoft_djvu.txt",
         local="stamp/britishincomespr00stamuoft_djvu.txt",
         sha256="8de5e6c16aded02e8d8d4fc8124e4ae2edd6bcb1b4094bc29e4af43eb88c6693", sha1="86a983d6716612a08b72434258e57f1ba494618a",
         licence="public domain OCR of the above", redistribute="yes (public domain)",
         use="OCR text layer: entry A for Table A4"),
]

# Sources probed and found blocked or absent. Recorded, never replaced.
BLOCKED = [
    dict(key="allen_site", url="https://robert-c-allen.net/",
         status="blocked: WordPress.com private-site login wall (HTTP 200, no data)"),
    dict(key="campop_enclosures",
         url="https://www.campop.geog.cam.ac.uk/research/projects/transport/data/enclosures.html",
         status="no download: the page shows a map image (enclosure.png) and no data link"),
    dict(key="bahs_chapman1987", url="https://www.bahs.org.uk/AGHR/ARTICLES/35n1a3.pdf",
         status="404 on 2026-09-26 (search-engine link to AgHR 35(1); archive moved)"),
    dict(key="bahs_walton1990", url="https://bahs.org.uk/AGHR/ARTICLES/38n1a6.pdf",
         status="404 on 2026-09-26 (search-engine link to AgHR 38(1); archive moved)"),
    dict(key="turner_county_acreage", url="(none found)",
         status="not found: no public copy of Turner (1980) county acreage or of the Tate-Turner "
                "digitisation (Clark 2001; Satchell-Bogart-Shaw-Taylor shapefile); Harvard "
                "Dataverse and Zenodo searched; HRV (QJE 2021, doi:10.7910/DVN/YZO7DV, CC0) has "
                "only a parish dummy 'Parliamentary Enclosure 1750-1840', not acreage"),
    dict(key="allen2009_rent_share", url="https://www.nuff.ox.ac.uk/Users/Allen/engelspause.pdf",
         status="figure only: Allen (2009) Fig. 2 plots land's share 1770-1913 (GB); no table, no "
                "data file reachable (his site is a login wall). Not digitised from the figure"),
    dict(key="broadberry2015_factor_shares", url="(none found)",
         status="not found: Broadberry et al. (2015) is output-side; no rent or factor-share "
                "series in the BoE carrier (A2-A7) or the BNS digitisation; the book is 403 at CUP"),
]


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def get(url, attempts_log, stream=True):
    """GET with verified TLS, up to three attempts. Returns the response or raises."""
    last = None
    for i in range(1, 4):
        try:
            r = requests.get(url, headers={"User-Agent": UA}, timeout=120, stream=stream,
                             allow_redirects=True)
            attempts_log.append(f"try{i}:{r.status_code}")
            if r.status_code == 200:
                return r
            last = RuntimeError(f"HTTP {r.status_code}")
        except requests.exceptions.SSLError as e:
            attempts_log.append(f"try{i}:ssl-error({str(e)[:80]})")
            last = e
        except requests.exceptions.RequestException as e:
            attempts_log.append(f"try{i}:{type(e).__name__}")
            last = e
        time.sleep(3 * i)
    raise RuntimeError(f"failed after 3 attempts: {last}")


def dataverse_file(doi, filename, attempts_log):
    """Resolve a file id by name from the dataset's latest version. Never hard-coded."""
    r = get(f"{DV}/api/datasets/:persistentId/?persistentId={doi}", attempts_log, stream=False)
    v = r.json()["data"]["latestVersion"]
    for f in v["files"]:
        df = f["dataFile"]
        if df.get("filename") == filename:
            return (f"{DV}/api/access/datafile/{df['id']}", df.get("md5"),
                    f"{v.get('versionNumber')}.{v.get('versionMinorNumber')}")
    raise RuntimeError(f"{filename} not in {doi}")


def fetch(src):
    dest = os.path.normpath(os.path.join(RAW, src["local"]))
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    attempts = []
    rec = dict(key=src["key"], retrieved=RETRIEVED, licence=src["licence"],
               redistribute=src["redistribute"], use=src["use"], local=dest)
    md5_expected = None
    url = src.get("url")
    if "dataverse" in src:
        url, md5_expected, dv_version = dataverse_file(*src["dataverse"], attempts)
        rec["dataverse_version"] = dv_version
    rec["url"] = url
    pin = src.get("sha256")

    if os.path.exists(dest) and pin and sha256_of(dest) == pin:
        rec.update(bytes=os.path.getsize(dest), sha256=pin, status="on disk from an earlier run; sha256 matches pin",
                   attempts="none")
        side = dest + ".fetch.json"
        if os.path.exists(side):
            old = json.load(open(side, encoding="utf-8"))
            for k in ("final_url", "last_modified", "md5_check", "sha1_check"):
                if k in old:
                    rec[k] = old[k]
        return rec

    r = get(url, attempts)
    tmp = dest + ".part"
    h = hashlib.sha256()
    md5 = hashlib.md5()
    sha1 = hashlib.sha1()
    n = 0
    with open(tmp, "wb") as f:
        for chunk in r.iter_content(1 << 20):
            f.write(chunk)
            h.update(chunk)
            md5.update(chunk)
            sha1.update(chunk)
            n += len(chunk)
    got = h.hexdigest()
    rec.update(final_url=r.url, last_modified=r.headers.get("Last-Modified", ""), bytes=n,
               sha256=got, attempts=";".join(attempts))
    if md5_expected:
        rec["md5_check"] = "ok" if md5.hexdigest() == md5_expected else "MISMATCH"
    if src.get("sha1"):  # archive.org publishes sha1 per file
        rec["sha1_check"] = "ok" if sha1.hexdigest() == src["sha1"] else "MISMATCH"
    if "MISMATCH" in (rec.get("md5_check", ""), rec.get("sha1_check", "")):
        os.replace(tmp, dest + ".new")
        rec["status"] = "STOP: host checksum (md5/sha1) does not match the bytes; kept as .new"
    elif pin and got != pin:
        os.replace(tmp, dest + ".new")
        rec["status"] = f"STOP: sha256 {got} differs from pin {pin}; kept as .new"
    else:
        os.replace(tmp, dest)
        rec["status"] = "fetched; sha256 matches pin" if pin else "fetched; no pin yet (record it)"
    with open(dest + ".fetch.json", "w", encoding="utf-8") as f:
        json.dump(rec, f, indent=1)
    return rec


def main():
    rows, stop = [], False
    for src in SOURCES:
        try:
            rec = fetch(src)
        except Exception as e:  # recorded, not replaced
            rec = dict(key=src["key"], url=src.get("url", str(src.get("dataverse"))),
                       retrieved=RETRIEVED, bytes="", sha256="", licence=src["licence"],
                       redistribute=src["redistribute"], use=src["use"],
                       status=f"BLOCKED: {e}")
        if rec["status"].startswith(("STOP", "BLOCKED")):
            stop = True
        print(f"{rec['key']:26s} {str(rec.get('bytes','')):>10s} {rec['status']}")
        rows.append(rec)
    for b in BLOCKED:
        rows.append(dict(key=b["key"], url=b["url"], retrieved=RETRIEVED, bytes="", sha256="",
                         licence="", redistribute="", use="", status=b["status"]))
    cols = ["key", "url", "retrieved", "bytes", "sha256", "last_modified", "licence",
            "redistribute", "use", "status"]
    with open(MANIFEST, "w", encoding="utf-8", newline="\n") as f:
        f.write("\t".join(cols) + "\n")
        for r in rows:
            f.write("\t".join(str(r.get(c, "")).replace("\t", " ") for c in cols) + "\n")
    print(f"manifest: {MANIFEST}")
    return 1 if stop else 0


if __name__ == "__main__":
    sys.exit(main())
