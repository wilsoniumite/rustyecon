"""build-commons: Tier 3S's mirror predictions in a line per set (from tier3s.jsonl)."""
import json
import statistics as s

rows = [json.loads(l) for l in open("/mnt/d/rustyecon-p23/build-commons/tier3s.jsonl")]
for n in ("C1", "C2"):
    for st in ("tier3s", "tier3sx10"):
        rs = [r for r in rows if r["name"] == n and r["set"] == st]
        tt = sorted(r["runner"]["ttol"] for r in rs)
        cl = {}
        for r in rs:
            cl[r["cls_engine_rule"]] = cl.get(r["cls_engine_rule"], 0) + 1
        print(n, st, len(rs), cl, "ticks to tol median", s.median(tt), "slowest", tt[-1],
              "dead ticks worst", max(r["runner"]["dead"] for r in rs),
              "lowest baskets", round(min(r["runner"]["baskets_trough"] for r in rs), 3),
              "first-year D-hat least", round(min(r["d0_first_year"] for r in rs), 1))
