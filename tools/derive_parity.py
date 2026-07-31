"""Derive each scenario's `parity` from its technology and register it per pop.

`parity` is the pop's outside option in **subsistence baskets per unit of
labour** (kernel.md, "Price formation"; pops.md). It drives the participation
margin `sigma_pi = (w/P_basket - parity)/parity`, which kernel.md calls the
load-bearing mechanism for the labour market.

THE RULE, STATED BEFORE IT WAS APPLIED (METHODOLOGY R6)
-------------------------------------------------------
It would be easy, and wrong, to set parity from the genesis real wage — that
makes sigma_pi = 0 at t=0 and looks tidy. `RealWage` is one of the two metrics
the A/B's band is computed on, so choosing parity from prices would set part of
a scored series from a dial. Parity is therefore derived from *technology*,
which is in the tape and is not a scored series:

    labour_content(g)  solved from the recipe graph: the labour embodied in one
                       unit of g, direct plus indirect. Labour itself is 1.0.
    basket             the tier-0 wealth level — subsistence, by definition the
                       bottom of the registered basket table.
    market_rate        1 / (labour embodied in one basket)
                       = how many baskets an hour buys at market productivity.
    parity             market_rate * HOME_PENALTY

HOME_PENALTY = 0.5: a pop producing for itself, without the market's
specialisation or its capital, achieves half of market productivity. It is a
structural constant of the world, not a fitted one, and it is stated here rather
than tuned later. Two things follow from the value that are worth being explicit
about: at 1.0 no wage could ever clear the margin (the market cannot pay labour
its whole product and still cover other inputs) so participation would collapse
to the floor; at 0.0 the margin is inert and labour posts unconditionally, which
is the legacy behaviour this replaces. Half sits between those corners and
corresponds to a labour share of one half.

Where a recipe's inputs cannot be resolved to labour at all — a good produced by
nothing, an import — its labour content is left undefined and it is skipped in
the basket, with a warning. A basket with NO resolvable content is an error
rather than a silent parity of infinity.

    python tools/derive_parity.py [--write]
"""

from __future__ import annotations

import glob
import math
import os
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))

import ron  # noqa: E402  — the repo's RON parser, not a regex approximation

REPO = pathlib.Path(__file__).resolve().parent.parent
HOME_PENALTY = 0.5


def labour_contents(goods: list, recipes: list, currency: set, labour: str) -> dict:
    """Direct + indirect labour embodied in one unit of each good.

    Solved by relaxation rather than matrix inversion: the recipe graph has
    cycles (transport takes flour and makes flour), so a naive topological pass
    would not terminate. Iterating to a fixed point handles both, and the cap
    keeps a genuinely circular definition from spinning forever.
    """
    content = {labour: 1.0}
    for _ in range(200):
        changed = False
        for _name, ins, outs in recipes:
            for og, oq in outs:
                if oq <= 0 or og == labour or og in currency:
                    continue
                # A good on both sides is passing through, not being made.
                if any(ig == og for ig, _ in ins):
                    continue
                total, ok = 0.0, True
                for ig, iq in ins:
                    if ig in currency:
                        continue
                    if ig not in content:
                        ok = False
                        break
                    total += content[ig] * iq
                if not ok:
                    continue
                value = total / oq
                if og not in content or abs(content[og] - value) > 1e-12:
                    # Cheapest route wins: the outside option is what the pop
                    # could actually achieve, not the worst way of achieving it.
                    if og not in content or value < content[og]:
                        content[og] = value
                        changed = True
        if not changed:
            break
    return content


def derive(scen_dir: pathlib.Path):
    gd = ron.load(str(scen_dir / "game_data.ron"))
    goods = [g["name"] for g in gd["goods"]]
    recipes = [
        (
            r["name"],
            [(i["good"], float(i["qty_per_unit"])) for i in r.get("inputs", [])],
            [(o["good"], float(o["qty_per_unit"])) for o in r["outputs"]],
        )
        for r in gd["recipes"]
    ]
    cats = {
        c["name"]: [(e["good"], float(e["weight"])) for e in c["entries"]]
        for c in gd.get("need_categories", [])
    }
    tier0 = {}
    for wl in gd.get("wealth_levels", []):
        if int(wl["tier"]) == 0:
            tier0 = {k: float(v) for k, v in dict(wl["needs"]).items()}
            break

    # The currency is whatever the market nodes name as such, not a guess from
    # the good's spelling.
    currency = set()
    for node in gd.get("market_nodes", []):
        cg = node.get("currency_good")
        if isinstance(cg, ron.Tagged) and cg.tag == "Some":
            idx = cg.inner
            if isinstance(idx, ron.Tagged):
                idx = idx.inner
            if isinstance(idx, int) and 0 <= idx < len(goods):
                currency.add(goods[idx])

    labour = next((g for g in goods if "labour" in g.lower()), None)
    if labour is None:
        return None, "no labour good: this world has no labour market to have a margin in"
    if not tier0 or not cats:
        return None, "no registered basket"

    content = labour_contents(goods, recipes, currency, labour)

    embodied, missing = 0.0, []
    for cat, qty in tier0.items():
        entries = cats.get(cat, [])
        total_w = sum(w for _, w in entries) or 1.0
        for g, w in entries:
            if g not in content:
                missing.append(g)
                continue
            embodied += qty * (w / total_w) * content[g]
    if embodied <= 0.0:
        return None, f"basket has no resolvable labour content (missing {missing})"
    parity = HOME_PENALTY / embodied
    return parity, (f"basket embodies {embodied:.4f} labour; market rate "
                    f"{1/embodied:.4f} baskets/labour; parity {parity:.4f}"
                    + (f"; SKIPPED {missing}" if missing else ""))


def write_parity(scen_dir: pathlib.Path, parity: float) -> int:
    p = scen_dir / "starting_state.ron"
    text = p.read_text(encoding="utf-8")
    if "parity:" in text:
        return 0
    # Inserted beside the field that already says what the pop supplies. Two
    # layouts exist in the corpus — one field per line, and whole pops on one
    # line — so both are handled rather than one being silently skipped, which
    # is how tracer_2r came back reporting "0 pops" the first time.
    lines, out, n = text.split("\n"), [], 0
    banner = [
        "// Subsistence baskets per unit of labour: the pop's outside option.",
        "// Derived from technology by tools/derive_parity.py, never from",
        "// prices, because RealWage is a scored series (R6).",
    ]
    for line in lines:
        m = re.match(r"(\s*)labour_good:\s*(.*)$", line)
        if m and m.group(2).rstrip().endswith(","):
            # One field per line: the value ends the line.
            out.append(line)
            indent = m.group(1)
            out.extend(f"{indent}{b}" for b in banner)
            out.append(f"{indent}parity: Some({parity!r}),")
            n += 1
        elif "labour_good:" in line:
            # Inline pop: splice the field in after labour_good's value.
            new, count = re.subn(
                r"(labour_good:\s*(?:None|Some\(\"[^\"]*\"\)))",
                lambda mm: f"{mm.group(1)}, parity: Some({parity!r})",
                line,
            )
            out.append(new)
            n += count
        else:
            out.append(line)
    p.write_text("\n".join(out), encoding="utf-8")
    return n


def main() -> int:
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except (AttributeError, OSError):
        pass
    write = "--write" in sys.argv
    dirs = sorted(pathlib.Path(d) for d in glob.glob(str(REPO / "data" / "scenarios" / "*"))
                  if os.path.isdir(d))
    failed = 0
    for d in dirs:
        parity, note = derive(d)
        if parity is None:
            print(f"  {d.name:14s} SKIP  {note}")
            continue
        n = write_parity(d, parity) if write else 0
        print(f"  {d.name:14s} {note}" + (f"  -> {n} pops" if write else ""))
        if not math.isfinite(parity) or parity <= 0:
            failed += 1
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
