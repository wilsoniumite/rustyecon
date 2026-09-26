"""Fetch for the eyeball sheet: the one file the sheet adds to the three families.

Dated 2026-09-26. Branch spine-eyeball.

The file is the GPIH copy of Allen's spreadsheet of wages and prices for London and
southern England, 1259-1914 (it says it was downloaded from an Oxford address on
29 October 2013). The eyeball sheet plots Allen's wages and CPI as the Bank of England
file carries them (A47 AG-AK, BH). This file is used only to check that carry, year by
year (data/spine/eyeball.py). Allen's own site is a private login wall; this copy is a
mirror and is named as one. It is not substituted for Allen's published welfare ratios,
which were not reached.

TLS is verified. Up to three attempts; attempts are recorded. The bytes must match the
pin taken by the probe of the same day (sources plan, section 8); a mismatch keeps the new
bytes beside the old as <file>.mismatch and exits 1. The raw file stays outside the
repository (no licence is stated).

Run: D:/rustyecon-spine/venv/Scripts/python.exe data/spine/fetch_sheet.py
"""

import hashlib
import json
import os
import sys
import time

import requests

RETRIEVED = "2026-09-26"
RAW = os.environ.get("SPINE_RAW", "D:/rustyecon-spine/raw/sheet")
UA = "rustyecon-spine-fetch/0.1 (public-data research; contact via github.com/wilsontomass)"
URL = "https://gpih.ucdavis.edu/files/Allen_London_South_Eng_1259-1914.xlsx"
LOCAL = os.path.join(RAW, "gpih", "Allen_London_South_Eng_1259-1914.xlsx")
PIN = "a8d1b729f86a73498193aea526f1da1d2b82c8098c6d566a6636cf8fe2256827"


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


os.makedirs(os.path.dirname(LOCAL), exist_ok=True)
attempts = []
if os.path.exists(LOCAL) and sha256(LOCAL) == PIN:
    attempts.append("cached")
else:
    body = None
    for i in range(3):
        try:
            r = requests.get(URL, headers={"User-Agent": UA}, timeout=120)
            attempts.append(f"try{i + 1}:{r.status_code}")
            if r.status_code == 200:
                body, last_mod = r.content, r.headers.get("Last-Modified", "")
                break
        except requests.RequestException as e:
            attempts.append(f"try{i + 1}:{type(e).__name__}")
        time.sleep(2)
    if body is None:
        print("BLOCKED", URL, attempts)
        sys.exit(1)
    got = hashlib.sha256(body).hexdigest()
    if got != PIN:
        with open(LOCAL + ".mismatch", "wb") as f:
            f.write(body)
        print("STOP: sha256 mismatch", got, "pin", PIN)
        sys.exit(1)
    with open(LOCAL, "wb") as f:
        f.write(body)
    with open(LOCAL + ".fetch.json", "w", encoding="utf-8") as f:
        json.dump(dict(url=URL, retrieved=RETRIEVED, bytes=len(body), sha256=got,
                       last_modified=last_mod, attempts=attempts), f, indent=1)
print("ok", LOCAL, os.path.getsize(LOCAL), "bytes;", ",".join(attempts))
