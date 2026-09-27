#!/usr/bin/env python3
"""The atlas build (docs/GUI.md §6; data/atlas/README.md). Written 2026-09-27.

Reads two pinned sources, checks their SHA-256, and writes `gb.atlas.ron` (regions, adjacency,
sites and shared-border geometry in British National Grid metres) and `preview.png` beside this
script. The run is offline once the cache holds the sources, and the same inputs and the pinned
package versions (requirements.txt) give the same bytes.

  1. HCBP Definition B (Historic Counties Trust), UK, OSGB, simplified: the 92 historic counties
     of the Historic Counties Standard, detached parts assigned to their parent counties.
  2. OpenStreetMap's boundary=traditional relations for the three ridings of Yorkshire, fetched
     from Overpass as of a fixed date. Overpass's reply carries the server's own timestamp, so
     the pin is on a canonical extract of the reply (ways, roles and coordinates), not its bytes.

What it does: Yorkshire is split along OSM's riding lines (faces of the noded linework, each
face to the riding it overlaps most); Ross-shire and Cromartyshire are joined as Ross and
Cromarty (HCS §4.4; Chapman ROC); areas, centroids, label points, shared-border lengths and sea
frontage are measured on the unsimplified coverage; the coverage is simplified with its
topology kept (shapely.coverage_simplify), snapped to a grid, and cut into arcs, each border
stored once with the region on each side.

Run it in a venv made from requirements.txt (WSL, Python 3.10):

    python3 -m venv /mnt/d/rustyecon-demo/venv-atlas
    /mnt/d/rustyecon-demo/venv-atlas/bin/pip install -r data/atlas/requirements.txt
    /mnt/d/rustyecon-demo/venv-atlas/bin/python data/atlas/build_atlas.py \\
        --cache /mnt/d/rustyecon-demo/atlas/cache

Without --cache the sources are kept in data/atlas/.cache/, which git ignores.
"""
import argparse
import hashlib
import io
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from collections import defaultdict

import numpy as np
import shapefile
import shapely
from pyproj import Geod, Transformer
from shapely.geometry import LineString, MultiPolygon, Point, Polygon
from shapely.ops import polylabel, transform

HERE = os.path.dirname(os.path.abspath(__file__))
UA = "rustyecon-atlas-build/1 (data/atlas/build_atlas.py)"

# ---------------------------------------------------------------------------------------------
# The pinned sources.

HCBP = {
    "key": "hcbp-b",
    "title": "Historic County Borders Project, UK Definition B, OSGB, simplified",
    "publisher": "Historic Counties Trust",
    "release": "2026-09-25",
    "url": "https://www.county-borders.co.uk/UKDefinitionB_OSGB_Simplified.zip",
    "file": "UKDefinitionB_OSGB_Simplified.zip",
    "sha256": "764c2097b9fee7cf6e069edcf2db82becf5ac5e6ecbf00ffd15644009810d8dc",
    "licence": "free for all personal, educational, non-commercial and commercial use;"
               " acknowledgement requested",
}
OSM_DATE = "2026-09-25T17:35:44Z"
OSM_RIDINGS = {13390974: "wry", 13391138: "nry", 17849894: "ery"}
OSM = {
    "key": "osm-ridings",
    "title": "OpenStreetMap boundary=traditional relations 13390974, 13391138 and 17849894"
             " (the West, North and East Ridings of Yorkshire), via Overpass",
    "publisher": "OpenStreetMap contributors",
    "release": OSM_DATE,
    "url": "https://overpass-api.de/api/interpreter",
    "query": f'[out:json][timeout:180][date:"{OSM_DATE}"];'
             f'relation(id:{",".join(str(i) for i in sorted(OSM_RIDINGS))});out geom;',
    "file": "osm_ridings_20260925T173544Z.json",
    # SHA-256 of canonical_ridings(reply), not of the reply's bytes (see the docstring).
    "sha256": "9103c50bbec1146467e957953031a3d77da12423a8114188b8486ae261a226b7",
    "licence": "ODbL 1.0",
}

# ---------------------------------------------------------------------------------------------
# The regions. HCS code -> (Chapman code, country). Keys are county.<chapman>, lower case
# (D7). Yorkshire (YRK) becomes its three ridings; Ross-shire (RSS) and Cromartyshire (CRT)
# become Ross and Cromarty (ROC), which HCS §4.4 allows "for many practical purposes" and which
# is the only Chapman code the two have. Monmouthshire's country is Wales, as in Chapman's list.

HCS = {
    # England
    "BED": ("BDF", "England"), "BER": ("BRK", "England"), "BUC": ("BKM", "England"),
    "CMB": ("CAM", "England"), "CHE": ("CHS", "England"), "CNW": ("CON", "England"),
    "CUM": ("CUL", "England"), "DRB": ("DBY", "England"), "DVN": ("DEV", "England"),
    "DRS": ("DOR", "England"), "DRH": ("DUR", "England"), "ESE": ("ESS", "England"),
    "GLC": ("GLS", "England"), "HMP": ("HAM", "England"), "HRF": ("HEF", "England"),
    "HTF": ("HRT", "England"), "HNT": ("HUN", "England"), "KNT": ("KEN", "England"),
    "LCS": ("LAN", "England"), "LCR": ("LEI", "England"), "LNC": ("LIN", "England"),
    "MSX": ("MDX", "England"), "NRF": ("NFK", "England"), "NHP": ("NTH", "England"),
    "NHB": ("NBL", "England"), "NOT": ("NTT", "England"), "OXD": ("OXF", "England"),
    "RTL": ("RUT", "England"), "SHP": ("SAL", "England"), "SMS": ("SOM", "England"),
    "STF": ("STS", "England"), "SFF": ("SFK", "England"), "SUR": ("SRY", "England"),
    "SUS": ("SSX", "England"), "WRW": ("WAR", "England"), "WML": ("WES", "England"),
    "WTS": ("WIL", "England"), "WRC": ("WOR", "England"),
    # Wales
    "AGL": ("AGY", "Wales"), "BRN": ("BRE", "Wales"), "CRN": ("CAE", "Wales"),
    "CRD": ("CGN", "Wales"), "CRM": ("CMN", "Wales"), "DBH": ("DEN", "Wales"),
    "FLT": ("FLN", "Wales"), "GLM": ("GLA", "Wales"), "MRN": ("MER", "Wales"),
    "MNM": ("MON", "Wales"), "MTG": ("MGY", "Wales"), "PMB": ("PEM", "Wales"),
    "RDN": ("RAD", "Wales"),
    # Scotland
    "ABN": ("ABD", "Scotland"), "ANG": ("ANS", "Scotland"), "ARG": ("ARL", "Scotland"),
    "AYS": ("AYR", "Scotland"), "BNF": ("BAN", "Scotland"), "BRW": ("BEW", "Scotland"),
    "BTE": ("BUT", "Scotland"), "CTN": ("CAI", "Scotland"), "CLM": ("CLK", "Scotland"),
    "DMF": ("DFS", "Scotland"), "DUN": ("DNB", "Scotland"), "ELT": ("ELN", "Scotland"),
    "FFE": ("FIF", "Scotland"), "INS": ("INV", "Scotland"), "KNC": ("KCD", "Scotland"),
    "KNR": ("KRS", "Scotland"), "KCB": ("KKD", "Scotland"), "LNK": ("LKS", "Scotland"),
    "MLT": ("MLN", "Scotland"), "MOY": ("MOR", "Scotland"), "NRN": ("NAI", "Scotland"),
    "ORN": ("OKI", "Scotland"), "PBS": ("PEE", "Scotland"), "PRT": ("PER", "Scotland"),
    "RNF": ("RFW", "Scotland"), "RSS": ("ROC", "Scotland"), "CRT": ("ROC", "Scotland"),
    "RXB": ("ROX", "Scotland"), "SKK": ("SEL", "Scotland"), "SHT": ("SHI", "Scotland"),
    "STL": ("STI", "Scotland"), "SRL": ("SUT", "Scotland"), "WLT": ("WLN", "Scotland"),
    "WGT": ("WIG", "Scotland"),
    # Northern Ireland: the same HCBP file and terms cover the six counties.
    "ANM": ("ANT", "NorthernIreland"), "ARH": ("ARM", "NorthernIreland"),
    "DWN": ("DOW", "NorthernIreland"), "FRM": ("FER", "NorthernIreland"),
    "LDR": ("LDY", "NorthernIreland"), "TYN": ("TYR", "NorthernIreland"),
    # Yorkshire: split below.
    "YRK": (None, "England"),
}
RIDING_NAMES = {"ery": "East Riding of Yorkshire", "nry": "North Riding of Yorkshire",
                "wry": "West Riding of Yorkshire"}
JOINED_NAMES = {"ROC": "Ross and Cromarty"}

# Major seaports c. 1750-1850, for the `port` flag and the atlas's sites. An illustrative list
# chosen for the demo world from general knowledge, not from a source; positions (WGS84) are
# approximate, to about a kilometre. Each names the county the build must find it in.
PORTS = [
    ("london", "London (the Pool)", 51.5075, -0.0790, "MDX"),
    ("bristol", "Bristol", 51.4540, -2.5960, "GLS"),
    ("liverpool", "Liverpool", 53.4040, -2.9920, "LAN"),
    ("hull", "Hull", 53.7430, -0.3310, "ERY"),
    ("whitby", "Whitby", 54.4860, -0.6150, "NRY"),
    ("newcastle", "Newcastle upon Tyne", 54.9680, -1.6090, "NBL"),
    ("sunderland", "Sunderland", 54.9060, -1.3800, "DUR"),
    ("whitehaven", "Whitehaven", 54.5490, -3.5930, "CUL"),
    ("yarmouth", "Great Yarmouth", 52.6080, 1.7300, "NFK"),
    ("kings_lynn", "King's Lynn", 52.7540, 0.3950, "NFK"),
    ("ipswich", "Ipswich", 52.0540, 1.1590, "SFK"),
    ("harwich", "Harwich", 51.9440, 1.2870, "ESS"),
    ("dover", "Dover", 51.1230, 1.3150, "KEN"),
    ("portsmouth", "Portsmouth", 50.7980, -1.1070, "HAM"),
    ("southampton", "Southampton", 50.8980, -1.4040, "HAM"),
    ("plymouth", "Plymouth", 50.3680, -4.1420, "DEV"),
    ("falmouth", "Falmouth", 50.1530, -5.0700, "CON"),
    ("swansea", "Swansea", 51.6180, -3.9390, "GLA"),
    ("milford", "Milford Haven", 51.7120, -5.0400, "PEM"),
    ("glasgow", "Glasgow", 55.8570, -4.2560, "LKS"),
    ("greenock", "Greenock", 55.9480, -4.7640, "RFW"),
    ("leith", "Leith", 55.9760, -3.1720, "MLN"),
    ("dundee", "Dundee", 56.4600, -2.9700, "ANS"),
    ("aberdeen", "Aberdeen", 57.1440, -2.0940, "ABD"),
    ("inverness", "Inverness", 57.4800, -4.2250, "INV"),
    ("belfast", "Belfast", 54.6010, -5.9270, "ANT"),
    ("londonderry", "Londonderry", 54.9970, -7.3190, "LDY"),
]

# ---------------------------------------------------------------------------------------------
# Fetching and pinning.


def sha256(b):
    return hashlib.sha256(b).hexdigest()


def fetch(url, data=None, tries=5):
    """GET, or POST `data`. Overpass sheds load with 429, 503 and 504; those are retried after
    a growing pause."""
    for k in range(tries):
        req = urllib.request.Request(url, data=data, headers={"User-Agent": UA, "Accept": "*/*"})
        try:
            with urllib.request.urlopen(req, timeout=600) as r:
                return r.read()
        except urllib.error.HTTPError as e:
            if e.code not in (429, 503, 504) or k == tries - 1:
                raise
            wait = 30 * (k + 1)
            print(f"  {url}: HTTP {e.code}; retrying in {wait} s")
            time.sleep(wait)
    raise AssertionError("unreachable")


def save(path, b):
    with open(path + ".part", "wb") as f:
        f.write(b)
    os.replace(path + ".part", path)


def canonical_ridings(reply):
    """The part of an Overpass reply the build reads, as canonical JSON bytes."""
    out = []
    for e in reply["elements"]:
        if e["type"] != "relation" or e["id"] not in OSM_RIDINGS:
            continue
        ways = [[m["ref"], m.get("role", ""),
                 [[round(p["lon"], 7), round(p["lat"], 7)] for p in m["geometry"]]]
                for m in e["members"] if m["type"] == "way" and "geometry" in m]
        out.append({"id": e["id"], "name": e["tags"]["name"], "ways": ways})
    out.sort(key=lambda r: r["id"])
    return json.dumps(out, separators=(",", ":"), sort_keys=True).encode()


def get_hcbp(cache):
    path = os.path.join(cache, HCBP["file"])
    if not os.path.exists(path):
        print(f"fetching {HCBP['url']}")
        save(path, fetch(HCBP["url"]))
    b = open(path, "rb").read()
    if sha256(b) != HCBP["sha256"]:
        sys.exit(f"{path}: SHA-256 {sha256(b)} is not the pinned {HCBP['sha256']}. HCBP"
                 " republishes at the same URL; a new release needs a new pin and a rebuild.")
    return b


def get_osm(cache):
    path = os.path.join(cache, OSM["file"])
    if not os.path.exists(path):
        print(f"fetching {OSM['url']} (attic query as of {OSM_DATE})")
        save(path, fetch(OSM["url"], urllib.parse.urlencode({"data": OSM["query"]}).encode()))
    canon = canonical_ridings(json.loads(open(path, "rb").read().decode("utf-8")))
    if sha256(canon) != OSM["sha256"]:
        sys.exit(f"{path}: canonical SHA-256 {sha256(canon)} is not the pinned {OSM['sha256']}")
    return json.loads(canon)


# ---------------------------------------------------------------------------------------------
# Geometry helpers.


def parts(g):
    if g.is_empty:
        return []
    return list(g.geoms) if g.geom_type == "MultiPolygon" else [g]


def nverts(g):
    return int(shapely.get_num_coordinates(g))


def load_hcbp(zbytes):
    z = zipfile.ZipFile(io.BytesIO(zbytes))
    r = shapefile.Reader(shp=io.BytesIO(z.read("UKDefinitionB.shp")),
                         shx=io.BytesIO(z.read("UKDefinitionB.shx")),
                         dbf=io.BytesIO(z.read("UKDefinitionB.dbf")))
    out = {}
    for sr in r.iterShapeRecords():
        rec = sr.record.as_dict()
        out[rec["HCS_CODE"]] = (rec, shapely.geometry.shape(sr.shape.__geo_interface__))
    return out


def riding_polygons(canon):
    to_bng = Transformer.from_crs("EPSG:4326", "EPSG:27700", always_xy=True)
    out = {}
    for rel in canon:
        lines = [LineString(w[2]) for w in rel["ways"] if w[1] in ("outer", "")]
        poly = shapely.union_all(list(shapely.polygonize(lines).geoms))
        out[OSM_RIDINGS[rel["id"]]] = transform(to_bng.transform, poly)
    return out


def split_yorkshire(hc, rid):
    """Yorkshire's faces, cut by OSM's riding lines, each to the riding it overlaps most.

    Yorkshire's neighbours are rebuilt from the same noded linework, so a node where a riding
    line meets the county border is a vertex on both sides and the coverage stays valid."""
    Y = hc["YRK"]
    touch = sorted(c for c in hc if c != "YRK" and hc[c].intersects(Y))
    rid_lines = shapely.union_all([g.boundary for g in rid.values()]).intersection(Y)
    noded = shapely.union_all([hc[c].boundary for c in ["YRK"] + touch] + [rid_lines])
    faces = list(shapely.polygonize(list(shapely.get_parts(noded))).geoms)
    shapely.prepare(Y)
    for c in touch:
        shapely.prepare(hc[c])
    assign = defaultdict(list)
    for f in faces:
        pt = f.representative_point()
        if Y.contains(pt):
            areas = {k: g.intersection(f).area for k, g in sorted(rid.items())}
            best = max(areas, key=lambda k: (areas[k], k))
            if areas[best] == 0.0:
                best = min(sorted(rid), key=lambda k: rid[k].distance(pt))
            assign[best].append(f)
            continue
        hit = next((c for c in touch if hc[c].contains(pt)), None)
        if hit is not None:
            assign[hit].append(f)
    ridings = {k: shapely.coverage_union_all(assign[k]) for k in sorted(rid)}
    ridings = merge_riding_fragments(ridings)
    got = sum(g.area for g in ridings.values())
    assert abs(got - Y.area) < 1.0, f"ridings cover {got/1e6:.3f} km2 of Yorkshire's {Y.area/1e6:.3f}"
    rebuilt = {}
    for c in touch:
        g = shapely.coverage_union_all(assign[c])
        assert abs(g.area - hc[c].area) < 1.0, f"{c} changed area rebuilding it"
        rebuilt[c] = g
    return ridings, rebuilt, len(faces)


def merge_riding_fragments(ridings, max_km2=5.0):
    """A small piece of one riding that touches none of its own riding's other pieces but
    borders another riding is a sliver of the split (OSM's outer line and HCBP's differ); it
    goes to the riding it shares the longest border with."""
    moved = []
    changed = True
    while changed:
        changed = False
        for k in sorted(ridings):
            ps = sorted(parts(ridings[k]), key=lambda p: -p.area)
            for p in ps[1:]:
                if p.area > max_km2 * 1e6:
                    continue
                shared = {o: p.boundary.intersection(ridings[o].boundary).length
                          for o in sorted(ridings) if o != k}
                best = max(shared, key=lambda o: (shared[o], o))
                own = sum(p.boundary.intersection(q.boundary).length for q in ps if q is not p)
                if shared[best] > 0.0 and own == 0.0:
                    ridings[k] = shapely.coverage_union_all([q for q in ps if q is not p])
                    ridings[best] = shapely.coverage_union_all(parts(ridings[best]) + [p])
                    moved.append((k, best, p.area / 1e6))
                    changed = True
                    break
            if changed:
                break
    for k, best, a in moved:
        print(f"  riding sliver {a:.3f} km2 moved from {k} to {best}")
    return ridings


def orient(p):
    return shapely.geometry.polygon.orient(p, 1.0)  # exterior CCW, holes CW


# ---------------------------------------------------------------------------------------------
# Arcs: each ring cut at junctions (vertices with other than two distinct neighbours), each
# piece stored once, forward for the region on its left. A ring with no junction is one closed
# arc starting at its least vertex, so the two rings that share it agree.


def ring_coords(ring, q):
    xy = np.asarray(ring.coords)[:-1]
    pts = [(int(round(x / q)), int(round(y / q))) for x, y in xy]
    out = []
    for p in pts:
        if not out or out[-1] != p:
            out.append(p)
    while len(out) > 1 and out[-1] == out[0]:
        out.pop()
    return out


def build_arcs(shapes):
    """shapes: per region, a list of parts, each a list of rings (lists of grid points, open,
    exterior CCW then holes CW). Returns arcs [(left, right, points)] and per region, per part,
    per ring, the signed arc references (i forward, -1-i reversed)."""
    nbrs = defaultdict(set)
    for reg in shapes:
        for part in reg:
            for ring in part:
                n = len(ring)
                for i in range(n):
                    a, b = ring[i], ring[(i + 1) % n]
                    nbrs[a].add(b)
                    nbrs[b].add(a)
    junction = {v for v, s in nbrs.items() if len(s) != 2}

    def pieces(ring):
        idx = [i for i, p in enumerate(ring) if p in junction]
        if not idx:
            m = min(range(len(ring)), key=lambda i: ring[i])
            seq = ring[m:] + ring[:m]
            return [tuple(seq + [seq[0]])]
        s = idx[0]
        seq = ring[s:] + ring[:s]
        out, cur = [], [seq[0]]
        for p in seq[1:] + [seq[0]]:
            cur.append(p)
            if p in junction:
                out.append(tuple(cur))
                cur = [p]
        return out

    index = {}  # canonical tuple -> arc id
    arcs = []  # [left, right, pts]
    refs = []
    for r, reg in enumerate(shapes):
        rparts = []
        for part in reg:
            rrings = []
            for ring in part:
                rr = []
                for pc in pieces(ring):
                    rev = tuple(reversed(pc))
                    # A valid coverage has one region on each side of an edge, so a piece seen
                    # forward before is an error, and one seen backward before has no right yet.
                    assert pc not in index, "an arc with the same region side twice"
                    if rev in index:
                        i = index[rev]
                        assert arcs[i][1] is None, "an arc used backward twice"
                        arcs[i][1] = r
                        rr.append(-1 - i)
                    else:
                        i = len(arcs)
                        index[pc] = i
                        arcs.append([r, None, pc])
                        rr.append(i)
                rrings.append(rr)
            rparts.append(rrings)
        refs.append(rparts)
    return arcs, refs


# ---------------------------------------------------------------------------------------------
# Writing RON.


def ron_str(s):
    return json.dumps(s, ensure_ascii=False)  # a JSON string literal is a RON string literal


def fmt1(v):
    return f"{v:.1f}"


HEADER = """\
// The rustyecon atlas: the historic counties of the United Kingdom, with Yorkshire in its three
// ridings. Written by data/atlas/build_atlas.py; do not edit by hand. Read by
// crates/worldgen/src/atlas.rs, which records this file's FNV-1a 64 digest.
//
// Licence: Open Database License 1.0 (data/atlas/LICENSE). Contains data from the Historic
// County Borders Project (https://www.county-borders.co.uk), Historic Counties Trust, and
// OpenStreetMap data (c) OpenStreetMap contributors (https://www.openstreetmap.org/copyright).
// See data/atlas/ATTRIBUTION.
//
// Coordinates are British National Grid (EPSG:27700) metres. Region centroids and label points
// are whole metres. Arc points are in units of `quantum_m`: the first pair absolute, each later
// pair the difference from the one before. A shape's ring lists arc references: i is arc i as
// stored, -1 - i is arc i reversed. Exterior rings run anticlockwise and holes clockwise, so an
// arc's `left` region lies on its left as stored; `right` is the region on the other side, or
// None where the other side is sea or unmapped water. Areas, border lengths and sea frontage
// are measured on the unsimplified coverage, in kilometres and square kilometres; `coastal` is
// a sea frontage of 1 km or more. `port` marks a region holding one of the `sites`, an
// illustrative list of major seaports c. 1750-1850 chosen for the demo world, not a source.
"""


def write_ron(path, meta, regions, sites, shapes_refs, arcs):
    L = [HEADER, "("]
    L.append(f"    schema: {meta['schema']},")
    L.append(f"    crs: {ron_str(meta['crs'])},")
    L.append(f"    quantum_m: {meta['quantum_m']},")
    L.append(f"    tolerance_m: {meta['tolerance_m']},")
    L.append("    sources: [")
    for s in meta["sources"]:
        L.append("        (")
        for k in ("key", "title", "publisher", "release", "url", "sha256", "licence"):
            L.append(f"            {k}: {ron_str(s[k])},")
        L.append("        ),")
    L.append("    ],")
    L.append("    regions: [")
    for r in regions:
        L.append("        (")
        L.append(f"            key: {ron_str(r['key'])},")
        L.append(f"            chapman: {ron_str(r['chapman'])},")
        L.append(f"            hcs: [{', '.join(ron_str(h) for h in r['hcs'])}],")
        L.append(f"            hcs_number: [{', '.join(str(n) for n in r['hcs_number'])}],")
        L.append(f"            name: {ron_str(r['name'])},")
        L.append(f"            country: {r['country']},")
        L.append(f"            area_km2: {fmt1(r['area_km2'])},")
        L.append(f"            centroid: ({r['centroid'][0]}, {r['centroid'][1]}),")
        L.append(f"            label: ({r['label'][0]}, {r['label'][1]}),")
        L.append(f"            sea_km: {fmt1(r['sea_km'])},")
        L.append(f"            coastal: {'true' if r['coastal'] else 'false'},")
        L.append(f"            port: {'true' if r['port'] else 'false'},")
        L.append("            neighbours: [")
        for k, km in r["neighbours"]:
            L.append(f"                ({ron_str(k)}, {fmt1(km)}),")
        L.append("            ],")
        L.append("        ),")
    L.append("    ],")
    L.append("    sites: [")
    for s in sites:
        L.append(f"        (key: {ron_str(s['key'])}, kind: Port, name: {ron_str(s['name'])},"
                 f" region: {ron_str(s['region'])}, at: ({s['at'][0]}, {s['at'][1]})),")
    L.append("    ],")
    L.append("    shapes: [")
    for key, rparts in shapes_refs:
        ps = ", ".join("[" + ", ".join("[" + ", ".join(str(i) for i in ring) + "]"
                                       for ring in part) + "]" for part in rparts)
        L.append(f"        ({ron_str(key)}, [{ps}]),")
    L.append("    ],")
    L.append("    arcs: [")
    for left, right, pts in arcs:
        flat = [pts[0][0], pts[0][1]]
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            flat += [x1 - x0, y1 - y0]
        rs = "None" if right is None else f"Some({right})"
        L.append(f"        ({left}, {rs}, [{','.join(str(v) for v in flat)}]),")
    L.append("    ],")
    L.append(")")
    text = "\n".join(L) + "\n"
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    return text.encode("utf-8")


# ---------------------------------------------------------------------------------------------
# The preview.


def preview(path, regions, simp, sites, tol):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.collections import PatchCollection
    from matplotlib.patches import PathPatch
    from matplotlib.path import Path

    # Six light tints, one per colour class of a DSatur colouring of the adjacency graph, so
    # neighbours differ. They mark identity only: no order, no good or bad.
    tints = ["#dce6f2", "#f4ddd0", "#d6ede3", "#f3e6c7", "#e6def0", "#e4e4df"]
    keys = [r["key"] for r in regions]
    adj = [{keys.index(k) for k, _ in r["neighbours"]} for r in regions]
    colour = {}
    while len(colour) < len(regions):
        i = max((j for j in range(len(regions)) if j not in colour),
                key=lambda j: (len({colour[n] for n in adj[j] if n in colour}), len(adj[j]),
                               keys[j]))
        used = {colour[n] for n in adj[i] if n in colour}
        colour[i] = next(c for c in range(len(tints)) if c not in used)

    x0, x1, y0, y1 = -10000, 670000, 0, 1230000
    fig = plt.figure(figsize=(7.0, 7.0 * (y1 - y0) / (x1 - x0)), dpi=150)
    ax = fig.add_axes((0.0, 0.0, 1.0, 1.0))
    fig.patch.set_facecolor("#fbfbfa")
    ax.set_facecolor("#fbfbfa")
    patches, faces = [], []
    for i, g in enumerate(simp):
        for p in parts(g):
            verts, codes = [], []
            for ring in [p.exterior] + list(p.interiors):
                xy = np.asarray(ring.coords)
                verts += xy.tolist()
                codes += [Path.MOVETO] + [Path.LINETO] * (len(xy) - 2) + [Path.CLOSEPOLY]
            patches.append(PathPatch(Path(verts, codes)))
            faces.append(tints[colour[i]])
    ax.add_collection(PatchCollection(patches, facecolors=faces, edgecolors="#6b6a66",
                                      linewidths=0.25))
    for r in regions:
        ax.text(r["label"][0], r["label"][1], r["chapman"], fontsize=4.2, ha="center",
                va="center", color="#2b2b29")
    xs = [s["at"][0] for s in sites]
    ys = [s["at"][1] for s in sites]
    ax.scatter(xs, ys, s=5, color="#2b2b29", zorder=3, linewidths=0)
    ax.set_xlim(x0, x1)
    ax.set_ylim(y0, y1)
    ax.set_aspect("equal")
    ax.axis("off")
    n = len(regions)
    fig.text(0.03, 0.985, f"The rustyecon atlas: {n} historic counties and ridings of the United"
             f" Kingdom,\nBritish National Grid, simplified at {tol:g} m. Dots: the {len(sites)}"
             " ports behind\nthe port flag. Labels: Chapman codes (keys are county.<code>).",
             fontsize=6.5, color="#2b2b29", va="top")
    fig.text(0.03, 0.944, "Contains data from the Historic County Borders Project"
             " (county-borders.co.uk)\nand OpenStreetMap data (c) OpenStreetMap contributors."
             " Database licence: ODbL 1.0.", fontsize=4.8, color="#52514e", va="top")
    fig.savefig(path, facecolor=fig.get_facecolor(), metadata={"Software": None})
    plt.close(fig)


# ---------------------------------------------------------------------------------------------


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--cache", default=os.path.join(HERE, ".cache"))
    ap.add_argument("--tolerance", type=float, default=100.0, help="metres (coverage_simplify)")
    ap.add_argument("--quantum", type=int, default=1, help="grid, metres")
    ap.add_argument("--min-island-km2", type=float, default=0.02,
                    help="drop islands (parts bordering no other region) smaller than this")
    ap.add_argument("--out", default=HERE)
    a = ap.parse_args()
    os.makedirs(a.cache, exist_ok=True)

    hc_rec = load_hcbp(get_hcbp(a.cache))
    assert sorted(hc_rec) == sorted(HCS), "HCBP's counties are not the 92 of the Standard"
    hc = {k: v[1] for k, v in hc_rec.items()}
    rid = riding_polygons(get_osm(a.cache))
    print(f"HCBP: {len(hc)} counties, {sum(nverts(g) for g in hc.values())} vertices;"
          f" OSM ridings: {sorted(rid)}")
    assert shapely.coverage_is_valid(list(hc.values())), "HCBP's coverage is not valid"

    ridings, rebuilt, nfaces = split_yorkshire(hc, rid)
    print(f"Yorkshire: {nfaces} faces; ridings (km2):"
          f" { {k: round(g.area / 1e6, 1) for k, g in ridings.items()} };"
          f" rebuilt neighbours: {sorted(rebuilt)}")

    # The regions, before simplification.
    geo = {}  # chapman -> geometry
    meta = {}  # chapman -> (name, country, [hcs], [hcs numbers])
    for h, (chap, country) in sorted(HCS.items()):
        if h == "YRK":
            continue
        g = rebuilt.get(h, hc[h])
        rec = hc_rec[h][0]
        if chap in geo:
            geo[chap] = shapely.coverage_union_all([geo[chap], g])
            m = meta[chap]
            meta[chap] = (JOINED_NAMES[chap], country, m[2] + [h], m[3] + [rec["HCS_NUMBER"]])
        else:
            geo[chap] = g
            meta[chap] = (rec["NAME"], country, [h], [rec["HCS_NUMBER"]])
    for k, g in ridings.items():
        geo[k.upper()] = g
        meta[k.upper()] = (RIDING_NAMES[k], "England", ["YRK"], [hc_rec["YRK"][0]["HCS_NUMBER"]])
    chaps = sorted(geo, key=lambda c: c.lower())
    geoms = [geo[c] for c in chaps]
    assert shapely.coverage_is_valid(geoms), "the regions' coverage is not valid"
    print(f"regions: {len(chaps)}; parts {sum(len(parts(g)) for g in geoms)};"
          f" vertices {sum(nverts(g) for g in geoms)}")
    # Areas are planar, in the grid; say how far that is from the ellipsoid's.
    to_ll = Transformer.from_crs("EPSG:27700", "EPSG:4326", always_xy=True)
    geod = Geod(ellps="WGS84")
    rel = [abs(g.area / abs(geod.geometry_area_perimeter(transform(to_ll.transform, g))[0]) - 1.0)
           for g in geoms]
    print(f"planar areas against WGS84 ellipsoidal areas: largest difference {max(rel):.2%}"
          f" ({chaps[int(np.argmax(rel))]}), median {float(np.median(rel)):.2%}")

    # Measures on the unsimplified coverage.
    land = shapely.coverage_union_all(geoms)
    sea_line = shapely.union_all([p.exterior for p in parts(land)])
    holes = [h for p in parts(land) for h in p.interiors]
    print(f"land: {len(parts(land))} polygons; {len(holes)} enclosed gaps"
          f" ({sum(Polygon(h).area for h in holes) / 1e6:.2f} km2), not counted as sea")
    tree = shapely.STRtree(geoms)
    bounds = [g.boundary for g in geoms]
    shared = defaultdict(dict)
    for i, j in zip(*tree.query(geoms, predicate="intersects")):
        if i >= j:
            continue
        km = bounds[i].intersection(bounds[j]).length / 1000.0
        if km > 0.0:
            shared[i][j] = km
            shared[j][i] = km
    wgs = Transformer.from_crs("EPSG:4326", "EPSG:27700", always_xy=True)
    site_rows = []
    for key, name, lat, lon, chap in PORTS:
        x, y = wgs.transform(lon, lat)
        pt = Point(x, y)
        d = [g.distance(pt) for g in geoms]
        i = int(np.argmin(d))
        assert chaps[i] == chap and d[i] < 3000.0, f"port {key} falls in {chaps[i]} ({d[i]:.0f} m)"
        site_rows.append({"key": f"port.{key}", "name": name, "region": f"county.{chap.lower()}",
                          "at": (int(round(x)), int(round(y)))})
    port_regions = {s["region"] for s in site_rows}

    regions = []
    for i, c in enumerate(chaps):
        g = geoms[i]
        name, country, hcs, nums = meta[c]
        largest = max(parts(g), key=lambda p: p.area)
        lab = polylabel(largest, tolerance=50.0)
        sea_km = g.boundary.intersection(sea_line).length / 1000.0
        key = f"county.{c.lower()}"
        regions.append({
            "key": key, "chapman": c, "hcs": hcs, "hcs_number": nums, "name": name,
            "country": country, "area_km2": g.area / 1e6,
            "centroid": (int(round(g.centroid.x)), int(round(g.centroid.y))),
            "label": (int(round(lab.x)), int(round(lab.y))),
            "sea_km": sea_km, "coastal": sea_km >= 1.0, "port": key in port_regions,
            "neighbours": [(f"county.{chaps[j].lower()}", shared[i][j]) for j in sorted(shared[i])
                           if shared[i][j] >= 0.05],
        })
    for r in regions:
        if r["port"] and not r["coastal"]:
            print(f"  river port, no sea frontage: {r['key']}")

    # Simplify with the topology kept, snap to the grid, drop specks.
    simp = list(shapely.coverage_simplify(geoms, a.tolerance, simplify_boundary=True))
    assert shapely.coverage_is_valid(simp), "coverage_simplify broke the coverage"
    q = a.quantum
    snapped = [shapely.remove_repeated_points(shapely.set_precision(g, float(q), mode="pointwise"))
               for g in simp]
    bad = [chaps[i] for i, g in enumerate(snapped) if not g.is_valid or g.is_empty]
    assert not bad, f"snapping made invalid shapes: {bad}"
    assert shapely.coverage_is_valid(snapped), "snapping broke the coverage"
    print(f"simplified at {a.tolerance:g} m, snapped to {q} m: vertices"
          f" {sum(nverts(g) for g in snapped)}")

    # Islands (parts bordering no other region) under the threshold are dropped.
    shapes, kept_geoms, dropped = [], [], 0
    others = shapely.STRtree(snapped)
    for i, g in enumerate(snapped):
        keep = []
        for p in parts(g):
            if p.area < a.min_island_km2 * 1e6:
                near = [j for j in others.query(p, predicate="intersects") if j != i]
                if not near:
                    dropped += 1
                    continue
            keep.append(orient(p))
        kept_geoms.append(MultiPolygon(keep) if len(keep) > 1 else keep[0])
        shapes.append([[ring_coords(p.exterior, q)] + [ring_coords(h, q) for h in p.interiors]
                       for p in keep])
    print(f"islands under {a.min_island_km2} km2 dropped: {dropped}; parts kept"
          f" {sum(len(reg) for reg in shapes)}, holes"
          f" {sum(len(p) - 1 for reg in shapes for p in reg)}")
    for reg in shapes:
        for part in reg:
            for ring in part:
                assert len(ring) >= 3
    drift = [abs(g.area / geoms[i].area - 1.0) for i, g in enumerate(kept_geoms)]
    print(f"drawn areas against measured: largest difference {max(drift):.2%}"
          f" ({chaps[int(np.argmax(drift))]}), median {float(np.median(drift)):.3%}")

    arcs, refs = build_arcs(shapes)
    ring_verts = sum(len(r) for reg in shapes for p in reg for r in p)
    arc_pts = sum(len(x[2]) for x in arcs)
    # Borders the simplification kept, against the measured adjacency.
    arc_pairs = {tuple(sorted((l, r))) for l, r, _ in arcs if r is not None}
    adj_pairs = {tuple(sorted((i, chaps.index(k[7:].upper())))) for i, r in enumerate(regions)
                 for k, _ in r["neighbours"]}
    lost = sorted(adj_pairs - arc_pairs)
    extra = sorted(arc_pairs - adj_pairs)
    assert not extra, f"arcs between regions the measure calls apart: {extra}"
    print(f"arcs {len(arcs)}; ring vertices {ring_verts}; stored arc points {arc_pts};"
          f" adjacent pairs {len(adj_pairs)}, of which drawn as a border {len(adj_pairs) - len(lost)}")
    for i, j in lost:
        km = shared[i][j]
        print(f"  border too short to survive simplification: {chaps[i]}-{chaps[j]} {km:.2f} km")

    meta_out = {"schema": 1, "crs": "EPSG:27700", "quantum_m": q,
                "tolerance_m": int(a.tolerance) if a.tolerance == int(a.tolerance) else a.tolerance,
                "sources": [HCBP, OSM]}
    text = write_ron(os.path.join(a.out, "gb.atlas.ron"), meta_out, regions, site_rows,
                     [(r["key"], refs[i]) for i, r in enumerate(regions)],
                     [(l, r, pts) for l, r, pts in arcs])
    fnv = 0xCBF29CE484222325
    for b in text:
        fnv = ((fnv ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    print(f"gb.atlas.ron: {len(text)} bytes; sha256 {sha256(text)}; fnv1a64 {fnv:#018x}")
    preview(os.path.join(a.out, "preview.png"), regions, kept_geoms, site_rows, a.tolerance)
    png = open(os.path.join(a.out, "preview.png"), "rb").read()
    print(f"preview.png: {len(png)} bytes; sha256 {sha256(png)}")
    by_country = defaultdict(int)
    for r in regions:
        by_country[r["country"]] += 1
    print(f"regions by country: {dict(sorted(by_country.items()))};"
          f" coastal {sum(r['coastal'] for r in regions)}; port {sum(r['port'] for r in regions)}")


if __name__ == "__main__":
    main()
