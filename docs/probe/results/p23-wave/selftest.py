"""P2.3.11 (label run, 2026-09-30): the scorer's self-test, before the wave. It writes the
registered mirrors' own outputs as gather.py writes the engine's (runs.jsonl's records: the summary
line's fields, the stats lines, the CSV's last row, the history windows, the kick sets, the
elasticity probe and the oracle's point), then score.py scores them. Every line the mirrors' files
feed must pass; the lines that need a per-tick CSV the registered files do not hold (the thresholds'
return, the shares' decay and their end values, the end gaps) are missing, and so are the lines
whose registered values are the frame's text rather than a file (E6's 30,000-tick share). The
wall's point is built from the frame's registered oracle points (points.jsonl), so E9 checks
score.py's formulas for the edges against the frame's 50-digit ones.
Usage (WSL): python3 selftest.py OUT.jsonl"""
import csv
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
REG_W = os.path.join(REPO, "docs/probe/wall/registered")
REG_C = os.path.join(REPO, "docs/probe/commons/registered")
T3S_C = os.path.join(REPO, "docs/probe/results/commons/tier3s/tier3s.jsonl")
sys.path.insert(0, HERE)
import score as S  # noqa: E402

recs = []


def wall_rec(inst, set_, row, ticks):
    f = lambda k: None if row[k] == "" else float(row[k])
    i = lambda k: None if row[k] == "" else int(row[k])
    run = row["run"]
    cs = ticks // 4 if "@dated" in run else 0
    stats = {"wall.worst_buyer_fill|-": f("worst_fill"), "wall.depth_min|-": f("depth_min"),
             "wall.breach_ticks|-": i("breach_ticks"),
             "wall.first_breach|-": None if row["first_breach"] == "" else i("first_breach") + cs}
    for c in S.WALL_CATS:
        stats[f"wall.share_max|{c}"] = f(f"smax_{c}")
    for p, a in S.WALL_POPS.items():
        stats[f"wall.participation_hi|{a}"] = f(f"F_hi_{p}")
        stats[f"wall.saturated_ticks|{a}"] = i(f"sat_{p}")
    for m in S.WALL_MARKETS:
        stats[f"dead.below_floor|{m}"] = i(f"dead_{m}")
    return {"rel": f"wall/{inst}/{set_}/{S.fname(run)}", "cmd": "run", "inst": inst, "set": set_,
            "tpy": int(float(row["tpy"])), "ticks": ticks, "run": run, "class": row["cls"],
            "in_tol_from": i("ticks_to_tol"), "peak_dhat": f("peak_Dhat"), "dead": i("dead"),
            "low_baskets": f("baskets_trough"), "no_basket_ticks": i("baskets_none"),
            "transfer_short": f("transfer_short"), "last": 0.0, "stats": stats, "clock_start": cs,
            "rc": 0}


def tsv(name):
    with open(os.path.join(REG_W, f"runs_{name}.tsv"), newline="") as fh:
        return list(csv.DictReader(fh, delimiter="\t"))


for name, set_, inst, L in (("battery", "L", "iw1", 22000), ("tier3s", "tier3s", "iw1", 22000),
                            ("stocks", "stocks", "iw1", 22000), ("joint2", "joint2", "iw1", 22000),
                            ("joint4", "joint4", "iw1", 22000), ("basin", "basin", "iw1", 22000),
                            ("tpy12", "tpy12", "iw1", 20000), ("tpy365", "tpy365", "iw1", 164000),
                            ("hold", "hold", "iw1", 22000), ("tilt1", "tilt1", "iw1", 22000),
                            ("line", "L", "ic1", 25000), ("tier3s_line", "tier3s", "ic1", 25000)):
    for row in tsv(name):
        recs.append(wall_rec(inst, set_, row, L))
        if inst == "iw1" and ((name == "battery" and row["tier"] == "3") or name == "tier3s"):
            recs.append(wall_rec(inst, "10L", row, 220000))
for tpy, L, gap in ((52, 22000, 3.3e-16), (12, 20000, 1.6e-15), (365, 164000, 4.4e-16)):
    recs.append({"rel": f"wall/iw1/modea/{tpy}", "cmd": "run", "inst": "iw1", "set": "modea", "tpy": tpy,
                 "ticks": L, "run": "hold", "class": "CONVERGED", "mode_a": "PASS",
                 "peak_dhat": gap / 1e-3, "last": gap / 1e-3, "stats": {}})
    recs.append({"rel": f"wall/iw1/info/el{tpy}", "cmd": "elasticity", "inst": "iw1", "set": "info",
                 "tpy": tpy, "L": L, "markets": {"goods": {"tau": 106.4}}})
hl = json.load(open(os.path.join(REG_W, "history_land.json")))
recs.append({"rel": "wall/iw1/history/cycle_land.mach_1500_80_", "cmd": "run", "inst": "iw1",
             "set": "history", "class": "CONVERGED", "last": 0.0, "stats": {},
             "windows": [dict(w) for w in hl]})
loc = json.load(open(os.path.join(REG_W, "local.json")))
by = {}
for k in loc:
    if k["kind"] == "kick":
        by.setdefault(None if k["shock"] is None else f"{k['shock'][0]}={k['shock'][1]}@dated", []).append(k)
for tgt, ks in by.items():
    name = "hold" if tgt is None else tgt
    recs.append({"rel": f"wall/iw1/kick/{S.fname(name)}", "cmd": "kick", "inst": "iw1", "set": "kick",
                 "passed": all(k["gain_tail"] <= 1e-3 for k in ks),
                 "gain_tail": max(k["gain_tail"] for k in ks), "gain_peak": max(k["gain_peak"] for k in ks),
                 "kicks": len(ks)})
recs.append({"rel": "wall/iw1/env/@envelope", "cmd": "envelope", "inst": "iw1", "set": "env",
             "kicks": [dict(run=f"{k['market']}{k['sign']}", rate=k["rate"]) for k in by[None]]})
pts = []
for line in open(os.path.join(REG_W, "points.jsonl")):
    d = json.loads(line)
    if d.get("regime") != "Interior" or d["tpy"] != 52 or d["tag"].startswith("eps"):
        continue
    w = d["workers"]
    pts.append({"inst": "iw1", "label": d["tag"], "v": d["v"], "P_s": d["p_s"], "Y": d["y"],
                "n_D": d["n_pool"], "reserved wages": [w[1]["wage"], w[2]["wage"]],
                "hours": [w[0]["hours"], w[1]["reserved_hours"], w[2]["reserved_hours"]],
                "type prices": [d["types"][0]["price"]]})
recs.append({"rel": "wall/iw1/info/point", "cmd": "point", "inst": "iw1", "set": "info", "points": pts})

# ---- the commons
inv = {"C1": "c1", "C2": "c2", "C1/chi025": "c1n"}


def commons_rec(r, set_):
    rr = r.get("runner", r)
    inst = inv[r["name"]]
    run = r["run"] if "runner" in r else S.c_engine_name(r["run"])
    L = r["L"]
    cs = L // 4 if "@dated" in run else 0
    stats = {}
    for m, v in (rr.get("trough") or {}).items():
        stats[f"trough.cleared|{m}"] = v
    rt = dict(rr.get("regime_ticks", {}))
    if rr.get("stopped_early") and rr.get("regime_end"):
        rt[rr["regime_end"]] = rt.get(rr["regime_end"], 0) + L - rr["ticks_run"]
    for k in ("Commons", "Crowded", "Enclosed"):
        stats[f"commons.regime_ticks|{k}"] = rt.get(k, 0)
    stats["commons.regime_ticks|Split"] = 0
    stats["commons.regime_ticks|Unused"] = 0
    stats["commons.switches|-"] = rr.get("regime_switches")
    stats["commons.regime_end|-"] = rr.get("regime_end")
    stats["commons.regime_star|-"] = rr.get("regime_star")
    stats["commons.provider_coin_low|-"] = rr.get("provider_coin_low")
    stats["commons.tp_max|-"] = rr.get("tp_high")
    err = rr.get("err") or ""
    rt_tick = int(err.split("runaway at t=")[1]) if "runaway at t=" in err else None
    return {"rel": f"commons/{inst}/{set_}/{S.fname(run)}", "cmd": "run", "inst": inst, "set": set_,
            "tpy": int(rr.get("tpy", r.get("tpy", 52.0)) if rr.get("tpy") else 52), "ticks": L,
            "run": run, "class": r.get("cls_engine_rule") or rr["cls"], "in_tol_from": rr.get("ttol"),
            "peak_dhat": rr.get("peak"), "dead": rr.get("dead"), "low_baskets": rr.get("baskets_trough"),
            "no_basket_ticks": rr.get("baskets_none"), "worst_fill": rr.get("worst_fill"),
            "transfer_short": rr.get("transfer_short"), "transfer_short_ticks": rr.get("transfer_ticks"),
            "last": 0.0, "stats": stats, "end": {"ro_over_r": rr.get("ro_end")}, "clock_start": cs,
            "runaway_tick": rt_tick, "rc": 0}


for f in ("battery_v3.jsonl", "families_v2.jsonl", "tpy_v2.jsonl", "negctl.jsonl"):
    for line in open(os.path.join(REG_C, f)):
        r = json.loads(line)
        if f == "families_v2.jsonl" and r["set"] == "tpy12":
            continue
        s = {"battery": "L", "tier3x10": "10L", "negctl": "L"}.get(r["set"], r["set"])
        recs.append(commons_rec(r, s))
for line in open(T3S_C):
    r = json.loads(line)
    recs.append(commons_rec(r, "tier3s" if r["set"] == "tier3s" else "tier3s10L"))
pred = json.load(open(os.path.join(REG_C, "predictions.json")))
elas = json.load(open(os.path.join(REG_C, "elasticity.json")))
for c, N in (("c1", "C1"), ("c2", "C2")):
    for tpy in (52, 12, 365):
        ma = next(x for x in pred["mode_a"] if x["name"] == N and x["tpy"] == float(tpy))
        recs.append({"rel": f"commons/{c}/modea/{tpy}", "cmd": "run", "inst": c, "set": "modea", "tpy": tpy,
                     "run": "hold", "class": "CONVERGED", "mode_a": "PASS", "peak_dhat": ma["worst"] / 1e-3,
                     "last": ma["worst"] / 1e-3, "stats": {}})
        recs.append({"rel": f"commons/{c}/info/el{tpy}", "cmd": "elasticity", "inst": c, "set": "info",
                     "tpy": tpy, "L": elas[f"{N} {tpy}"]["L"], "markets": {}})
    for t in ["hold"] + S.COMMONS_TARGETS[c] + ["hold-12", "hold-365"]:
        recs.append({"rel": f"commons/{c}/kick/{S.fname(t)}", "cmd": "kick", "inst": c, "set": "kick",
                     "passed": True, "gain_tail": 1e-5, "gain_peak": 1.0, "kicks": 14})
hist = json.load(open(os.path.join(REG_C, "history.json")))
for h in hist:
    c = inv[h["name"]]
    run = "cycle(land.mach,1500,80)" if h["coef"] == "land.mach" else "cycle(commons,1500,80)"
    rt = None
    if h["err"]:
        w5, t5 = int(h["err"].split("window ")[1].split(",")[0]), int(h["err"].split("t=")[1])
        rt = w5 * 1500 + t5
    wins = [dict(k=w["w"], end_Dh=w["end_Dh"], ttol=w["ttol"], dead=w["dead"],
                 in_tol_at_end=w["end_Dh"] is not None and w["end_Dh"] <= 1) for w in h["windows"]]
    recs.append({"rel": f"commons/{c}/history/{S.fname(run)}", "cmd": "run", "inst": c, "set": "history",
                 "run": run, "class": "DIVERGED" if rt else "CONVERGED", "runaway_tick": rt, "last": 0.0,
                 "stats": {}, "windows": wins})

with open(sys.argv[1], "w", newline="\n") as fh:
    for r in recs:
        fh.write(json.dumps(r, sort_keys=True) + "\n")
print(f"{len(recs)} records")
