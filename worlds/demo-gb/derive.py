"""Derive the illustrative demo world's tables from its rough facts (docs/demo/WORLD.md), 2026-09-27.

    python worlds/demo-gb/derive.py            # writes regions.csv and history.csv beside this file
    python worlds/demo-gb/derive.py --check    # exits 1 if either file differs from what it would write

Reads counties.csv (rough facts and 0-1 tags per historic county) and writes:
- regions.csv: each county's instance of the Appendix B economy at 1750-01-01, per year where the
  tape registers a flow, with the tags the history's weights read and a basis note;
- history.csv: the dated ramps, 1750-1901, that move the instance.

Standard library only. Every number here is Assumed("illustrative demo, 2026-09-27: ..."): the rules
below are chosen to give a feel for the period and to keep every county, in every year, inside the
region where the probe's roles converge (docs/demo/WORLD.md sections 2-4 give the reasons and the
check). Nothing from this world may be scored or cited as research (R4, R5).
"""

import csv
import io
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TPY = 52  # ticks a year; the tape registers flows per year and the roles read them per tick

# --- The base instance: SSRN Appendix B's structure, with 1750-ish levels (WORLD.md section 2).
BASE = {
    "eta": 2.0,   # machines' task cost scale: twice Appendix B's, so fewer tasks are automated in 1750
    "g0": 0.2,    # Appendix B's schedule shape, unchanged
    "g1": 0.8,
    "k": 1.0,
    "a": 0.3,     # Appendix B's own input
    "lam": 0.03,  # an organic economy's machine services: little labour ...
    "b": 0.8,     # ... and much land (fodder, wood, water sites): twice Appendix B's b
    "space": 0.15,  # h, land services per basket: a small direct claim on land
    "chi_max": 1.0,
}
N_PER_THOUSAND = 1.0 / 25.0  # potential hours a tick per thousand people (only a scale: constant returns)
RHO0 = 0.45    # N/T a tick at the reference density
THETA = 0.25   # density compression: N/T = RHO0 * clamp((d / d_ref)^THETA, CLAMP)
CLAMP = (0.7, 1.3)
UPLAND_QUALITY = 0.7  # an upland acre counts (1 - 0.7 * upland) of a lowland acre
PORT_SITES = 0.4  # a harbour is a site: a port county's land services are (1 + 0.4 * port) times

# --- The history (WORLD.md section 4).
MU0, MU1 = 0.5, 0.5  # site services follow population: T grows by (N'/N)^(MU0 + MU1 * max(metro, port/2))
GRID = 12  # steps_per_year: the grid on which the compiler looks for a 1% move (WORLD.md section 4.3)


def fnum(x):
    """A number as the tables write it: 6 significant figures, no trailing zeros."""
    return f"{x:.6g}"


def read_counties():
    with open(os.path.join(HERE, "counties.csv"), newline="", encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    for r in rows:
        for k in ("area_km2", "pop1750_k", "pop1801_k", "pop1851_k", "pop1901_k", "upland", "coal",
                  "textile", "eng", "metro", "port", "canal", "openfield", "improver", "fen", "mining",
                  "slate", "highland", "speen", "kelp"):
            r[k] = float(r[k])
        r["pop1841_k"] = float(r["pop1841_k"]) if r["pop1841_k"] else None
        r["textile_from"] = int(r["textile_from"]) if r["textile_from"] else None
    return rows


def genesis(rows):
    """Each county's 1750 instance (WORLD.md section 3.2)."""
    eff = {r["key"]: r["area_km2"] * (1.0 - UPLAND_QUALITY * r["upland"]) for r in rows}
    dens = {r["key"]: r["pop1750_k"] / eff[r["key"]] for r in rows}
    logs = sorted(math.log(d) for d in dens.values())
    d_ref = math.exp(logs[len(logs) // 2])  # the median county's effective density
    out = []
    for r in rows:
        c = r["key"]
        ratio = RHO0 * min(max((dens[c] / d_ref) ** THETA, CLAMP[0]), CLAMP[1])
        n_tick = r["pop1750_k"] * N_PER_THOUSAND
        t_tick = n_tick / ratio * (1.0 + PORT_SITES * r["port"])
        up, coal, eng, metro = r["upland"], r["coal"], r["eng"], r["metro"]
        g = {
            "workers": n_tick * TPY,
            "land": t_tick * TPY,
            "space": BASE["space"] * (1.0 + up) * (1.0 + 0.25 * metro),
            "eta": BASE["eta"],
            "g0": BASE["g0"],
            "g1": BASE["g1"],
            "k": BASE["k"],
            "a": BASE["a"],
            "lam": BASE["lam"] * (1.0 - 0.2 * coal) * (1.0 - 0.1 * eng),
            "b": BASE["b"] * (1.0 - 0.3 * coal),
            "chi_max": BASE["chi_max"] * (1.0 + 0.3 * up),
        }
        nt = ratio / (1.0 + PORT_SITES * r["port"])
        why = [f"pop c.1750 ~{r['pop1750_k']:.0f}k on {r['area_km2']:.0f} km2 (upland {up:g})",
               f"N/T {nt:.3f} a tick (density {dens[c] / d_ref:.2f}x the median; "
               f"compressed ^{THETA} within {CLAMP})"]
        if up:
            why.append(f"upland: h x{1 + up:.2f}; chi x{1 + 0.3 * up:.2f}")
        if coal or eng:
            lam_x = (1 - 0.2 * coal) * (1 - 0.1 * eng)
            why.append(f"coal {coal:g} / eng {eng:g}: b x{1 - 0.3 * coal:.2f}; lam x{lam_x:.2f}")
        if metro:
            why.append(f"metropolitan {metro:g}: h x{1 + 0.25 * metro:.2f}")
        if r["port"]:
            why.append(f"port {r['port']:g}: T x{1 + PORT_SITES * r['port']:.2f}")
        g["why"] = "; ".join(why)
        out.append((r, g))
    return out, d_ref


# The ramps. Each is (key, start, end, regions, param, to, steps_per_year, weight, lever, note), with
# `to` the factor reached at `end` by a county of weight 1 (a county of weight w reaches to^w); the
# `from` column of these rows is 1. Selectors: "all", "nation:a+b", "tag:t" (tag > 0), "kind:k1+k2"
# (textile kind), or county keys separated by spaces; a start of 0 is the county's textile_from.
GROUP_RAMPS = [
    # --- Rents via T: enclosure and improvement of land (Midland open fields; commons and wastes).
    ("enclosure.open-fields.1", 1760, 1780, "tag:openfield", "land", 1.06, 12, "openfield", "rents (T)",
     "first wave of parliamentary enclosure: Midland open fields consolidated"),
    ("enclosure.wastes.2", 1793, 1815, "nation:england+wales", "land", 1.08, 12, "waste", "rents (T)",
     "war-time wave: commons and wastes enclosed; weight = upland share outside the Highlands"),
    ("enclosure.open-fields.2", 1793, 1815, "tag:openfield", "land", 1.04, 12, "openfield", "rents (T)",
     "war-time wave: the rest of the open fields"),
    ("improvement.scotland", 1760, 1830, "nation:scotland", "land", 1.15, 12, "improver", "rents (T)",
     "Lowland improvement: runrig ended, farms consolidated, limed and drained"),
    ("potato.ground", 1750, 1841, "nation:ireland", "land", 1.40, 12, "", "rents (T)",
     "potato ground: bog edge and hill land taken into tillage as Ulster's population grows"),
    ("drainage.fens", 1820, 1850, "tag:fen", "land", 1.15, 12, "fen", "rents (T)",
     "steam pumping drains the Fens"),
    ("clearances.sheep", 1780, 1855, "tag:highland", "land", 1.06, 12, "highland", "rents (T)",
     "Highland clearances: townships cleared for sheep walks (people leave through N)"),
    ("kelp.boom", 1765, 1810, "tag:kelp", "land", 1.10, 12, "kelp", "rents (T)",
     "kelp burned on the shore for alkali: a mineral-like rent"),
    ("kelp.bust", 1815, 1830, "tag:kelp", "land", 1 / 1.10, 12, "kelp", "rents (T)",
     "kelp collapses after the war (Spanish barilla, then Leblanc soda)"),
    ("mines.metal.boom", 1750, 1860, "tag:mining", "land", 1.30, 12, "mining", "rents (T)",
     "copper, tin and lead: mineral rents rise (Cornwall, Parys Mountain, the Peak)"),
    ("mines.metal.bust", 1866, 1885, "tag:mining", "land", 0.80, 12, "mining", "rents (T)",
     "copper and tin collapse; lead declines"),
    ("mines.slate", 1780, 1900, "tag:slate", "land", 1.60, 12, "slate", "rents (T)",
     "slate quarries: Penrhyn, Dinorwic, Ffestiniog roof the towns"),
    ("coal.mineral.1", 1750, 1830, "tag:coal", "land", 1.30, 12, "coal", "rents (T)",
     "coal's mineral rent: output grows on the old fields (Tyne, Wear, Lancashire, the West Riding)"),
    ("coal.mineral.2", 1830, 1870, "tag:coal", "land", 1.60, 12, "coal", "rents (T)",
     "coal's mineral rent: the railway age opens the fields"),
    ("coal.mineral.3", 1870, 1901, "tag:coal", "land", 1.30, 12, "coal", "rents (T)",
     "coal's mineral rent: steam coal (South Wales, Durham) for ships and export"),
    # --- Rents via h: less land per basket.
    ("improvement.rotation", 1750, 1840, "nation:england+wales", "space", 0.85, 12, "improver", "rents (h)",
     "Norfolk four-course rotation, turnips and clover: a basket needs less land"),
    ("improvement.high-farming", 1840, 1875, "all", "space", 0.90, 12, "lowland", "rents (h)",
     "high farming: tile drainage, guano and superphosphate; weight = lowland share"),
    # --- Rents via b and recursive automation via a: coal and steam make machine services cheaper.
    ("steam.coalfields", 1770, 1830, "tag:coal", "b", 0.70, 12, "coal", "rents (b)",
     "Watt's engine and coke iron: machine services need less land (fuel and fodder) on the coalfields"),
    ("canals", 1760, 1830, "tag:canal", "b", 0.90, 12, "canal", "rents (b)",
     "canals carry coal to the works: machine services need less land"),
    ("railways.gb", 1835, 1875, "nation:england+wales+scotland", "b", 0.75, 12, "offcoal", "rents (b)",
     "railways carry coal everywhere: the fall is largest away from the coalfields (weight 1 - coal/2)"),
    ("railways.ireland", 1845, 1885, "nation:ireland", "b", 0.80, 12, "offcoal", "rents (b)",
     "Ulster's railways, later and thinner"),
    ("railways.own-input", 1835, 1875, "all", "a", 0.90, 12, "", "recursive (a)",
     "cheaper carriage of materials: machine services need fewer machine services"),
    # --- Recursive automation via lam: machines that make machines.
    ("machine-tools", 1800, 1870, "tag:eng", "lam", 0.70, 12, "eng", "recursive (lam)",
     "Maudslay, Nasmyth and Whitworth: machine tools cut the labour in making machines"),
    ("engineering.spread", 1850, 1901, "all", "lam", 0.80, 12, "", "recursive (lam)",
     "engineering spreads: every county's machines take less labour to make"),
    # --- Task automation via eta: the schedule shifts, fastest in the factory-textile districts.
    ("textile.cotton", 0, 1835, "kind:cotton", "eta", 0.60, 12, "textile", "task (eta)",
     "cotton: jenny, water frame, mule, then the power loom; starts at the county's textile_from"),
    ("textile.wool", 0, 1850, "kind:wool", "eta", 0.70, 12, "textile", "task (eta)",
     "wool and worsted: scribbling and spinning mills, then worsted power looms"),
    ("textile.linen", 0, 1870, "kind:linen", "eta", 0.70, 12, "textile", "task (eta)",
     "wet-spun flax and jute: Dundee, Belfast, Dunfermline"),
    ("textile.hosiery", 0, 1880, "kind:hosiery", "eta", 0.80, 12, "textile", "task (eta)",
     "hosiery and lace move into steam-powered factories"),
    ("textile.other", 0, 1880, "kind:silk+pottery+ribbons+carpets+boots", "eta", 0.85, 12, "textile",
     "task (eta)", "silk, ribbons, carpets, pottery and boots: later and partial mechanisation"),
    ("threshing", 1790, 1830, "tag:improver", "eta", 0.95, 12, "improver", "task (eta)",
     "threshing machines in the arable counties (broken in the Swing riots of 1830)"),
    ("mechanisation.general", 1830, 1901, "all", "eta", 0.90, 12, "", "task (eta)",
     "general mechanisation: reapers, engineering, printing, food processing"),
    # --- Exit via chi_max: the value of dependence against work.
    ("poor-law.speenhamland", 1795, 1800, "tag:speen", "chi_max", 1.08, 12, "speen", "exit (chi)",
     "Speenhamland allowances in the southern and eastern arable counties: dependence worth more"),
    ("poor-law.1834", 1834, 1840, "nation:england+wales", "chi_max", 0.90, 12, "", "exit (chi)",
     "Poor Law Amendment Act: the workhouse test, less eligibility"),
    ("poor-law.ireland", 1838, 1845, "nation:ireland", "chi_max", 0.95, 12, "", "exit (chi)",
     "Irish Poor Law: workhouse unions"),
    ("poor-law.scotland", 1845, 1850, "nation:scotland", "chi_max", 1.03, 12, "", "exit (chi)",
     "Poor Law (Scotland) Act: parochial boards, some relief for the able-bodied in need"),
]


def weight(r, name):
    if name == "":
        return 1.0
    if name == "waste":
        return r["upland"] * (1.0 - r["highland"])
    if name == "lowland":
        return 1.0 - r["upland"]
    if name == "offcoal":
        return 1.0 - 0.5 * r["coal"]
    return r[name]


def selected(r, sel):
    if sel == "all":
        return True
    kind, _, arg = sel.partition(":")
    if kind == "nation":
        return r["nation"] in arg.split("+")
    if kind == "tag":
        return r[arg] > 0
    if kind == "kind":
        return r["textile_kind"] in arg.split("+") and r["textile"] > 0
    raise ValueError(sel)


def population_rows(r, g):
    """N's path through the benchmarks, geometric between them (mode path, absolute per year)."""
    pts = [(1750, r["pop1750_k"]), (1801, r["pop1801_k"])]
    if r["pop1841_k"] is not None:
        pts.append((1841, r["pop1841_k"]))
    pts += [(1851, r["pop1851_k"]), (1901, r["pop1901_k"])]
    rows = []
    short = r["chapman"].lower()
    for (y0, p0), (y1, p1) in zip(pts, pts[1:]):
        steps = GRID
        n0, n1 = p0 * N_PER_THOUSAND * TPY, p1 * N_PER_THOUSAND * TPY
        rows.append(dict(key=f"population.{short}.{y0}", start=y0, end=y1, regions=r["key"], param="workers",
                         mode="path", frm=n0, to=n1, steps=steps, weight="", lever="scarcity (N)",
                         note=f"population ~{p0:.0f}k to ~{p1:.0f}k (rough benchmarks)"))
        mu = MU0 + MU1 * max(r["metro"], r["port"] / 2)
        f = (p1 / p0) ** mu
        rows.append(dict(key=f"sites.{short}.{y0}", start=y0, end=y1, regions=r["key"], param="land",
                         mode="scale", frm=1.0, to=f, steps=steps, weight="", lever="rents (T)",
                         note=f"site services follow population: (N'/N)^{mu:.2f} (towns, harbours, works)"))
    return rows


def main():
    rows = read_counties()
    gen, d_ref = genesis(rows)
    regions_fields = ["key", "chapman", "hcs", "name", "nation", "workers", "land", "space", "eta", "g0", "g1",
                      "k", "a", "lam", "b", "chi_max", "upland", "coal", "textile", "textile_kind",
                      "textile_from", "eng", "metro", "port", "canal", "openfield", "improver", "fen", "mining",
                      "slate", "highland", "speen", "kelp", "categories", "machine_types", "carriers", "why"]
    reg = []
    for r, g in gen:
        row = {k: r[k] for k in ("key", "chapman", "hcs", "name", "nation", "textile_kind")}
        for k in ("workers", "land", "space", "eta", "g0", "g1", "k", "a", "lam", "b", "chi_max"):
            row[k] = fnum(g[k])
        for k in ("upland", "coal", "textile", "eng", "metro", "port", "canal", "openfield", "improver", "fen",
                  "mining", "slate", "highland", "speen", "kelp"):
            row[k] = fnum(r[k])
        row["textile_from"] = r["textile_from"] or ""
        row["categories"] = "good"
        # The one machine type, the horse (docs/demo/WORLD-V2.md §9.1; decision 327): v1 compiles it
        # as its flow machine, and stage v2a.1 as the durable good of machine_types.csv.
        row["machine_types"] = "horse"
        row["carriers"] = ""
        row["why"] = g["why"]
        reg.append(row)
    hist = []
    for (key, start, end, sel, param, to, steps, wname, lever, note) in GROUP_RAMPS:
        hist.append(dict(key=key, start=start if start else "textile_from", end=end, regions=sel, param=param,
                         mode="scale", frm=1.0, to=to, steps=steps, weight=wname, lever=lever, note=note))
    for r, g in gen:
        hist += population_rows(r, g)
    hist_fields = ["key", "start", "end", "regions", "param", "mode", "from", "to", "steps_per_year", "weight",
                   "lever", "note"]
    out = {}
    buf = io.StringIO()
    w = csv.DictWriter(buf, fieldnames=regions_fields, lineterminator="\n")
    w.writeheader()
    w.writerows(reg)
    out["regions.csv"] = buf.getvalue()
    buf = io.StringIO()
    w = csv.writer(buf, lineterminator="\n")
    w.writerow(hist_fields)
    for h in hist:
        w.writerow([h["key"], h["start"], h["end"], h["regions"], h["param"], h["mode"], fnum(h["frm"]),
                    fnum(h["to"]), h["steps"], h["weight"], h["lever"], h["note"]])
    out["history.csv"] = buf.getvalue()
    check = "--check" in sys.argv[1:]
    bad = False
    for name, text in out.items():
        path = os.path.join(HERE, name)
        if check:
            old = None
            if os.path.exists(path):
                with open(path, newline="", encoding="utf-8") as f:
                    old = f.read()
            if old != text:
                print(f"{name} differs from what derive.py writes")
                bad = True
        else:
            with open(path, "w", newline="", encoding="utf-8") as f:
                f.write(text)
    if check:
        sys.exit(1 if bad else 0)
    print(f"regions.csv: {len(reg)} counties; history.csv: {len(hist)} ramps; "
          f"median effective density {1000 * d_ref:.1f}/km2")


if __name__ == "__main__":
    main()
