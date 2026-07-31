"""Build a region corpus BY CONSTRUCTION from four consistency conditions — and
audit any existing tape against the same four.

    python tools/gen_regions.py --audit data/scenarios/lr_00 ...   # measure
    python tools/gen_regions.py --write                            # generate cr_*
    python tools/gen_regions.py --selftest                         # falsify the checks

WHY THIS EXISTS
---------------
`data/scenarios/lr_*` is a 24-cell FACTORIAL SEARCH GRID over labour x channel x
wheat x start_state. It was generated to LOOK FOR a stable region, and across
every configuration ever run exactly 5 of its 72 regions scored inside the 2.2x
band and all five were DEAD. No region has ever been both alive and in-band.

A search grid asks "is any of these stable?". It never asks whether any of them
is a world that COULD be stable. This tool asks that second question first, and
answers it from the tape alone, before a tick is run.

THE FOUR CONDITIONS, AND WHY EACH ONE IS A CONDITION AND NOT A PREFERENCE
-------------------------------------------------------------------------
Each is a statement about genesis that the engine cannot repair, because no
price can clear a market whose two sides differ by a factor no price appears in.

  A1  LABOUR BALANCE.  For every market node, the labour the desks would draw at
      their genesis capacity equals the hours the pops can offer.
      `sum_desks desired(labour) == sum_pops size * participation`

      Phase 4 measured `lr_00` running supply 0.41 against demand 73.7 and
      recorded that no price clears it. Both elasticities are zero there, so
      that is literally true: the price rule was chasing a crossing that does
      not exist. At genesis the same tape reads 2.07x (see `--audit`), and the
      collapse to 180x is the participation margin retreating from a market it
      could never have cleared.

  A2  GOODS BALANCE.  For every (node, good): what the desks produce or import
      equals what the desks consume as inputs plus what the pops eat at their
      GENESIS wealth tier.
      `sum_desks out_qty * chosen == sum_desks desired(in) + pop consumption`

      `lr_00` fails this twice over and in opposite directions: its mills make
      16 flour against 30 flour of pop demand (0.53x), while its services desks
      draw 50 labour to make a good the tier-0 basket does not contain at all
      (production/consumption = infinity). A market with zero demand and a
      market with zero supply are the two shapes the price rule handles worst —
      `imbalance = +-1` is a pin, not a signal.

  A3  MONEY.  Every owner holds the cash the kernel's own Rule 3 requires it to
      hold to transact at genesis prices:
          firm:  (b_cash/s + s) * outlay_per_tick   reserve + one activation
          pop:   b_cash * basket_cost / s           the reserve, so sigma_c = 0
      Below it a desk's budget is zero: it buys nothing, produces nothing, earns
      nothing, and can never climb out (the absorbing state recorded in
      src/kernel/mod.rs). This condition is KERNEL-SPECIFIC and is stated as
      such: the legacy layer sizes its holdings from `savings_target` and the
      dividend `reserve_multiple` instead. It is still the right condition to
      generate against, because the kernel is the arm under test.

  A4  PRICES.  Genesis relative prices equal the technology's implied relative
      prices — the Leontief labour-content vector, `p_g = lambda_g * p_labour`,
      which is the zero-profit vector every CapacityControl desk is trying to
      find. Solved by the SAME relaxation `tools/derive_parity.py` uses (it is
      imported, not re-implemented, so the generator and the scored benchmark
      cannot disagree), and the same vector `src/certify/technology.rs` scores
      B8 against.

      `lr_00` opens at wheat 0.35 / flour 0.60 against an implied 0.30 / 0.50 —
      1.17x and 1.20x. That is the mildest of its four defects by far, and
      saying so is part of the point: the price vector is the thing everyone
      looks at and it is not what is wrong with that corpus.

  A5  RULE WINDOW.  ADDED AFTER THE FIRST RUN, and it is not a condition on the
      world at all — it asks whether the kernel's OWN two rules can be satisfied
      at once. A pass-through desk (a channel operator) reads one inventory slot
      with two opposite meanings: Rule 1 posts stock above `b_out*flow`, Rule 3
      buys only below `s*flow`. A window exists iff `b_out < s`. It was found by
      building a world that satisfied A1-A4 EXACTLY and watching its channels
      stop trading on tick 2, which is the best argument available for building
      consistent worlds in the first place: a defect in the rules is only
      visible once the world stops being the obvious suspect.

WHAT THIS TOOL DOES *NOT* CLAIM
-------------------------------
Consistency is a NECESSARY condition, not a sufficient one. A world can satisfy
all four and still be dynamically unstable — indeed A1-A4 at exact equality make
genesis a FIXED POINT of the kernel's rules, and Phase 4's RESULT 2 already
established that the mechanism can hold a fixed point it cannot walk to. So the
corpus below is deliberately not four consistent worlds: it is four consistent
worlds, five worlds with a NAMED condition broken, and two mechanism probes, so
that the comparison is within one generator rather than between two corpora.

THE CONDITIONS ARE COUPLED, AND THE GENERATOR DECOUPLES THEM ON PURPOSE
-----------------------------------------------------------------------
Consumption drives capacity drives labour drives outlay drives cash, so a naive
single edit moves several readings at once and a run made from it is
uninterpretable. Every defect world below is therefore built so that its named
condition is the ONLY one that moves — cash is re-solved from each desk's own
outlay at each tape's own prices, capacity edits are compensated in the labour
market, and so on. `--selftest` asserts exactly that, on every defect world, in
both directions: the named condition reads its constructed ratio AND the other
three read 1.000. A check that fires on everything says nothing about anything.

The one world where a condition could not be isolated is `cr_10`, and the reason
is a result rather than an oversight: a rentier class has no consistent version.

ENCODING. Every read and write names utf-8 explicitly. A previous session let
cp1252 pick itself and corrupted 28 files.
"""

from __future__ import annotations

import argparse
import glob
import math
import os
import pathlib
import shutil
import sys
from dataclasses import dataclass, field

sys.path.insert(0, str(pathlib.Path(__file__).parent))

import ron  # noqa: E402  — the repo's RON parser, not a regex approximation
from derive_parity import HOME_PENALTY, labour_contents  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parent.parent
SCEN = REPO / "data" / "scenarios"

#: Two readings this close count as the same reading. Present so a condition that
#: holds exactly reports 1.000 rather than 0.9999999999999998, and so `--selftest`
#: can assert equality on a dyadic-rational construction without inventing a
#: tolerance for the arithmetic. It is NOT a pass band: the audit reports the
#: ratio and never converts it to a verdict.
EXACT = 1e-9


# ══════════════════════════════════════════════════════════════════════════════
# Mirrors of engine functions.
#
# Each of these reproduces a Rust function the audit's arithmetic depends on. A
# mirror that drifts from its original is worse than no audit at all, so each
# names the function it mirrors and each is exercised by `--selftest` against a
# tape whose answer is known.
# ══════════════════════════════════════════════════════════════════════════════


def desired(qty_per_unit: float, scaling: str, chosen: float, recipe_size: float) -> float:
    """`RecipeInput::desired` (src/types/recipe.rs)."""
    if scaling == "Fixed":
        return qty_per_unit * recipe_size
    # SemiVariable carries (floor, slope) and no corpus tape registers one; it is
    # rejected loudly rather than silently mis-audited as Variable.
    if scaling != "Variable":
        raise ValueError(f"unhandled input scaling {scaling!r}")
    return qty_per_unit * chosen


def interpolated_qty(wealth: float, levels: list, n_cats: int) -> list:
    """`GameData::interpolated_qty` (src/state/game_data.rs).

    `levels` is [(tier:int, [qty per category])], sorted by tier.
    """
    if not levels or n_cats == 0:
        return [0.0] * n_cats
    wealth = max(wealth, 0.0)
    floor_tier = int(math.floor(wealth))
    frac = wealth - math.floor(wealth)
    lower = levels[0]
    for lv in levels:
        if lv[0] <= floor_tier:
            lower = lv
    upper = lower
    for lv in levels:
        if lv[0] > floor_tier:
            upper = lv
            break
    return [
        lower[1][i] + frac * (upper[1][i] - lower[1][i])
        for i in range(n_cats)
    ]


def logit_shares(weights: list, prices: list, beta: float) -> list:
    """`kernel::logit_shares` (src/kernel/mod.rs)."""
    n = len(weights)
    if n == 0:
        return []
    if n == 1:
        return [1.0]
    mean = sum(prices) / n
    if not mean > 0.0:
        return [1.0 / n] * n
    raw = [max(w, 0.0) * math.exp(-beta * (p / mean)) for w, p in zip(weights, prices)]
    total = sum(raw)
    return [r / total for r in raw] if total > 0.0 else [1.0 / n] * n


# ══════════════════════════════════════════════════════════════════════════════
# Reading a tape
# ══════════════════════════════════════════════════════════════════════════════


def _idx(v) -> int:
    """Unwrap the several shapes an id takes in the corpus: `(0)`, `RegionId(0)`,
    `Some((0))`, plain `0`. Each appears in at least one shipped tape."""
    while True:
        if isinstance(v, ron.Tagged):
            v = v.inner
        elif isinstance(v, tuple) and len(v) == 1:
            v = v[0]
        else:
            break
    if isinstance(v, bool) or not isinstance(v, int):
        raise ValueError(f"not an id: {v!r}")
    return v


def _opt(v):
    """`Some(x)` -> x, `None` -> None."""
    if v is None:
        return None
    if isinstance(v, ron.Tagged) and v.tag == "Some":
        return v.inner
    if isinstance(v, ron.Bare) and v.value == "None":
        return None
    return v


@dataclass
class Tape:
    """Everything the four conditions need, resolved out of the RON."""

    name: str
    goods: list                      # [name]
    shelf: dict                      # name -> Bare/Tagged
    recipes: dict                    # name -> {inputs, outputs, strategy}
    cats: list                       # [(cat_name, [(good, weight)])]
    levels: list                     # [(tier, [qty per cat])]
    nodes: dict                      # node_idx -> (region_idx|None, currency name|None)
    channels: dict                   # ch_idx -> (from_node, to_node)
    regions: dict                    # region_idx -> (name, node_idx)
    kernel: dict
    prices: dict                     # good name -> price
    instances: list                  # desks and dividend pointers alike
    pops: list
    labour: str | None
    staple: str | None


def read_tape(d: pathlib.Path) -> Tape:
    gd = ron.load(str(d / "game_data.ron"))
    st = ron.load(str(d / "starting_state.ron"))

    goods = [g["name"] for g in gd["goods"]]
    shelf = {g["name"]: g["shelf_life"] for g in gd["goods"]}

    recipes = {}
    for r in gd["recipes"]:
        strat = r.get("strategy")
        strat_name = strat.tag if isinstance(strat, ron.Tagged) else (
            strat.value if isinstance(strat, ron.Bare) else "CapacityControl"
        )
        recipes[r["name"]] = {
            "inputs": [
                (i["good"], float(i["qty_per_unit"]),
                 i["scaling"].value if isinstance(i["scaling"], ron.Bare) else str(i["scaling"]))
                for i in r.get("inputs", [])
            ],
            "outputs": [(o["good"], float(o["qty_per_unit"])) for o in r["outputs"]],
            "strategy": strat_name,
        }

    cats = [
        (c["name"], [(e["good"], float(e["weight"])) for e in c["entries"]])
        for c in gd.get("need_categories", [])
    ]
    levels = []
    for wl in gd.get("wealth_levels", []):
        needs = dict(wl["needs"])
        levels.append((int(wl["tier"]), [float(needs.get(c[0], 0.0)) for c in cats]))
    levels.sort(key=lambda t: t[0])

    nodes = {}
    for n in gd["market_nodes"]:
        reg = _opt(n.get("region"))
        cur = _opt(n.get("currency_good"))
        nodes[_idx(n["id"])] = (
            _idx(reg) if reg is not None else None,
            goods[_idx(cur)] if cur is not None else None,
        )
    channels = {
        _idx(c["id"]): (_idx(c["from"]), _idx(c["to"])) for c in gd.get("channels", [])
    }
    regions = {
        _idx(r["id"]): (r["name"], _idx(r["market_node"])) for r in gd["regions"]
    }
    kernel = {k: (v if not isinstance(v, ron.Bare) else v.value)
              for k, v in gd.get("kernel", {}).items()}

    prices = {k: float(v) for k, v in dict(st.get("prices", {})).items()}

    instances = []
    for b in st.get("buildings", []):
        ch = _opt(b.get("channel"))
        instances.append({
            "id": _idx(b["id"]),
            "region": _idx(b["region"]),
            "recipe": b["recipe"],
            "recipe_size": float(b["recipe_size"]),
            "chosen_size": float(b["chosen_size"]),
            "inv": {k: float(v) for k, v in dict(b.get("inventory", {})).items()},
            "channel": _idx(ch) if ch is not None else None,
            "extra": False,
        })
    for e in st.get("extra_recipe_instances", []):
        instances.append({
            "id": None,
            "region": None,
            "recipe": e["recipe"],
            "recipe_size": float(e["recipe_size"]),
            "chosen_size": float(e["recipe_size"]),
            "inv": {},
            "channel": None,
            "extra": True,
            "building": _idx(e["building"]),
        })

    pops = []
    for p in st.get("pop_groups", []):
        lg = _opt(p.get("labour_good"))
        par = _opt(p.get("parity"))
        part = _opt(p.get("participation"))
        pops.append({
            "id": _idx(p["id"]),
            "region": _idx(p["region"]),
            "size": float(p["size"]),
            "wealth": float(p["wealth"]),
            "inv": {k: float(v) for k, v in dict(p.get("inventory", {})).items()},
            "labour_good": lg,
            "parity": float(par) if par is not None else None,
            # `RawPopGroup::participation`: absent means 1.0.
            "participation": float(part) if part is not None else 1.0,
        })

    labour = staple = None
    cpath = d / "criteria.ron"
    if cpath.exists():
        cr = ron.load(str(cpath))
        labour = cr.get("labour_good")
        staple = cr.get("staple_good")
    if labour is None:
        labour = next((g for g in goods if "labour" in g.lower()), None)

    return Tape(d.name, goods, shelf, recipes, cats, levels, nodes, channels,
                regions, kernel, prices, instances, pops, labour, staple)


def buy_sell_nodes(tape: Tape, inst: dict) -> tuple:
    """`kernel::nodes`: a channel operator spans two nodes, everything else
    trades at its own region's node."""
    if inst["channel"] is not None:
        return tape.channels[inst["channel"]]
    home = tape.regions[inst["region"]][1]
    return (home, home)


def currencies(tape: Tape) -> set:
    return {c for _, c in tape.nodes.values() if c is not None}


def desks(tape: Tape) -> list:
    """Instances the kernel governs. `DividendPayout` is not a desk (Rule 3's
    overflow routing replaces it), so it draws no inputs and posts no output."""
    return [i for i in tape.instances
            if tape.recipes[i["recipe"]]["strategy"] != "DividendPayout"]


# ══════════════════════════════════════════════════════════════════════════════
# The four conditions
# ══════════════════════════════════════════════════════════════════════════════


def _ratio(num: float, den: float) -> float:
    """num/den, with the two degenerate cases kept apart.

    0/0 is 1.0: a market nobody supplies and nobody wants is BALANCED, not
    broken, and calling it a defect would make every tape fail on every good it
    does not use. x/0 is infinity: a market with a demand and no supply at all is
    the worst case this audit exists to find, and it must not be smoothed into a
    large finite number.
    """
    if den == 0.0:
        return 1.0 if num == 0.0 else math.inf
    return num / den


def pop_consumption(tape: Tape) -> dict:
    """(node, good) -> quantity the pops eat per tick at their GENESIS wealth."""
    beta = float(tape.kernel.get("beta", 1.0))
    out: dict = {}
    for p in tape.pops:
        node = tape.regions[p["region"]][1]
        per = interpolated_qty(p["wealth"], tape.levels, len(tape.cats))
        for ci, (_cname, entries) in enumerate(tape.cats):
            total = per[ci] * p["size"]
            if total == 0.0:
                continue
            weights = [w for _, w in entries]
            prices = [max(tape.prices.get(g, 1.0), 0.0) for g, _ in entries]
            for (g, _w), share in zip(entries, logit_shares(weights, prices, beta)):
                out[(node, g)] = out.get((node, g), 0.0) + share * total
    return out


def a1_labour(tape: Tape) -> dict:
    """Labour demanded at genesis capacity against labour offered, per node."""
    if tape.labour is None:
        return {}
    supply: dict = {}
    for p in tape.pops:
        if p["labour_good"] != tape.labour:
            continue
        node = tape.regions[p["region"]][1]
        supply[node] = supply.get(node, 0.0) + p["size"] * p["participation"]
    demand: dict = {}
    for inst in desks(tape):
        buy, _sell = buy_sell_nodes(tape, inst)
        for g, q, sc in tape.recipes[inst["recipe"]]["inputs"]:
            if g != tape.labour:
                continue
            demand[buy] = demand.get(buy, 0.0) + desired(
                q, sc, inst["chosen_size"], inst["recipe_size"])
    out = {}
    for node in sorted(set(supply) | set(demand)):
        s, dm = supply.get(node, 0.0), demand.get(node, 0.0)
        out[node] = {"supply": s, "demand": dm, "ratio": _ratio(dm, s)}
    return out


def a2_goods(tape: Tape) -> dict:
    """Produced-or-imported against consumed-or-exported, per (node, good).

    Labour is excluded because A1 reports it on its own terms, and currency
    because it is not produced. A pass-through recipe (transport: flour in,
    flour out) counts as a demand at its buy node and a supply at its sell node,
    which is exactly the flow it moves.
    """
    cur = currencies(tape)
    skip = cur | ({tape.labour} if tape.labour else set())
    supply: dict = {}
    demand: dict = {}
    for inst in desks(tape):
        buy, sell = buy_sell_nodes(tape, inst)
        r = tape.recipes[inst["recipe"]]
        for g, q in r["outputs"]:
            if g in skip:
                continue
            supply[(sell, g)] = supply.get((sell, g), 0.0) + q * inst["chosen_size"]
        for g, q, sc in r["inputs"]:
            if g in skip:
                continue
            demand[(buy, g)] = demand.get((buy, g), 0.0) + desired(
                q, sc, inst["chosen_size"], inst["recipe_size"])
    for (node, g), qty in pop_consumption(tape).items():
        if g in skip:
            continue
        demand[(node, g)] = demand.get((node, g), 0.0) + qty
    out = {}
    for key in sorted(set(supply) | set(demand)):
        s, dm = supply.get(key, 0.0), demand.get(key, 0.0)
        out[key] = {"supply": s, "demand": dm, "ratio": _ratio(s, dm)}
    return out


def a3_money(tape: Tape) -> dict:
    """Cash held against the cash Rule 3 needs, per owner.

    KERNEL-SPECIFIC, and deliberately so: `(b_cash/s + s) * outlay` is Rule 3's
    reserve plus one activation's purchases, and `b_cash * cost / s` is the
    consumption desk's reserve at `sigma_c = 0`. The legacy arm holds cash
    against `savings_target` and `reserve_multiple` instead, so a tape sized for
    one arm is not sized for the other. Reported per owner because the binding
    constraint is per owner: one starved desk is an absorbing state whatever the
    regional total says.
    """
    k = tape.kernel
    if "b_cash" not in k or "s" not in k:
        return {}
    b_cash, s = float(k["b_cash"]), float(k["s"])
    cur = currencies(tape)
    out = {}

    # Desks sharing one building inventory share one cash position — Rule 3 is
    # evaluated per inventory OWNER, not per desk, so the requirement is summed
    # the same way or a two-desk building reads as short when it is not.
    by_building: dict = {}
    for inst in desks(tape):
        if inst["extra"]:
            continue
        by_building.setdefault(inst["id"], []).append(inst)
    for bid, group in sorted(by_building.items()):
        buy, _ = buy_sell_nodes(tape, group[0])
        currency = tape.nodes[buy][1]
        if currency is None:
            continue
        outlay = 0.0
        for inst in group:
            for g, q, sc in tape.recipes[inst["recipe"]]["inputs"]:
                if g in cur:
                    continue
                outlay += desired(q, sc, inst["chosen_size"], inst["recipe_size"]) \
                    * tape.prices.get(g, 1.0)
        need = (b_cash / s + s) * outlay
        held = group[0]["inv"].get(currency, 0.0)
        out[f"building{bid}"] = {"held": held, "need": need, "ratio": _ratio(held, need)}

    beta = float(k.get("beta", 1.0))
    for p in tape.pops:
        node = tape.regions[p["region"]][1]
        currency = tape.nodes[node][1]
        if currency is None:
            continue
        per = interpolated_qty(p["wealth"], tape.levels, len(tape.cats))
        cost = 0.0
        for ci, (_cn, entries) in enumerate(tape.cats):
            total = per[ci] * p["size"]
            weights = [w for _, w in entries]
            prices = [max(tape.prices.get(g, 1.0), 0.0) for g, _ in entries]
            for (g, _w), share in zip(entries, logit_shares(weights, prices, beta)):
                cost += share * total * tape.prices.get(g, 1.0)
        need = b_cash * cost / s
        held = p["inv"].get(currency, 0.0)
        out[f"pop{p['id']}"] = {"held": held, "need": need, "ratio": _ratio(held, need)}
    return out


def a4_prices(tape: Tape) -> dict:
    """Genesis relative prices against the technology's implied vector.

    The solver is `derive_parity.labour_contents`, imported rather than copied,
    so this reading and the `parity` written into the same tape cannot come from
    two different Leontief solves. `src/certify/technology.rs` is the third
    implementation and B8 scores against it; all three carry the same four rules
    (currency skipped, pass-through skipped, relaxation not topological because
    the graph has cycles, cheapest route wins).
    """
    if tape.labour is None:
        return {}
    cur = currencies(tape)
    recs = [
        (name, [(g, q) for g, q, _sc in r["inputs"]], list(r["outputs"]))
        for name, r in tape.recipes.items()
    ]
    content = labour_contents(tape.goods, recs, cur, tape.labour)
    p_labour = tape.prices.get(tape.labour, 1.0)
    out = {}
    for g in tape.goods:
        if g in cur or g == tape.labour:
            continue
        if g not in content:
            out[g] = {"implied": None, "price": tape.prices.get(g, 1.0), "ratio": math.inf,
                      "note": "no recipe route reaches labour"}
            continue
        implied = content[g] * p_labour
        out[g] = {"implied": implied, "price": tape.prices.get(g, 1.0),
                  "ratio": _ratio(tape.prices.get(g, 1.0), implied), "note": ""}
    return out


def a5_rule_window(tape: Tape) -> dict:
    """Is there ANY stock level at which a desk both posts and buys?

    ADDED 2026-07-31, AFTER `cr_01` DEADLOCKED. This condition is not about the
    world at all — it is about whether the kernel's own two rules can be
    satisfied at once — and it was found by building a world that satisfied
    A1-A4 exactly and watching its channels stop trading on tick 2.

    A PASS-THROUGH desk (a channel operator: flour in at one node, flour out at
    another) reads ONE inventory slot with two opposite meanings:

        Rule 1 posts   stock - b_out * chosen * q_out      > 0
        Rule 3 buys    s * desired_in(chosen) - stock      > 0

    so a window exists only where `b_out * q_out < s * q_in`. The ratio below is
    `s*q_in / (b_out*q_out)`: above 1 there is a stock level that does both,
    at or below 1 there is NONE and the desk is deadlocked whatever it holds.
    Stocked, it posts until it hits the band and then never buys again; empty,
    it never posts at all and its destination node has a market with zero supply
    for ever.

    MEASURED, on `cr_01`'s wheat channel, wheat cleared at the buying node over
    12 ticks: s=1 -> 0.00, s=2 -> 18.77, s=3 -> 93.32, s=4 -> 86.77. The
    boundary is exactly where the arithmetic says it is.

    IT ALSO EXPLAINS A DEFECT ALREADY IN THE RECORD. Phase 4 traced lr's
    collapse to "flour has zero posted supply from tick 0 under both rules —
    that part is pre-existing" and never said why. lr's six `flour_transport`
    desks open with no flour, so they post nothing; the importing node's flour
    imbalance is pinned at +1 and the price marks up by the full alpha for ever.
    Seeding them WOULD NOT HAVE HELPED at any level, because lr's window
    (s=4, b_out=2) is `2*flow < stock < 4*flow` and a desk that starts anywhere
    else in that band walks out of it. Only desks with a pass-through good are
    affected; a mill's wheat-in and flour-out live in different slots.

    Non-pass-through worlds are unconstrained and report `inf`.
    """
    k = tape.kernel
    if "b_out" not in k or "s" not in k:
        return {}
    b_out, s = float(k["b_out"]), float(k["s"])
    out = {}
    for inst in desks(tape):
        r = tape.recipes[inst["recipe"]]
        ins = {g: (q, sc) for g, q, sc in r["inputs"]}
        for g, q_out in r["outputs"]:
            if g not in ins:
                continue
            q_in, sc = ins[g]
            buy = s * desired(q_in, sc, inst["chosen_size"], inst["recipe_size"])
            post = b_out * inst["chosen_size"] * q_out
            key = f"{inst['recipe']}#{inst['id']}/{g}"
            out[key] = {"buy_below": buy, "post_above": post, "ratio": _ratio(buy, post)}
    return out


@dataclass
class Audit:
    name: str
    a1: dict
    a2: dict
    a3: dict
    a4: dict
    a5: dict

    @staticmethod
    def _worst(ratios) -> float:
        """The reading furthest from 1.0 in LOG space, so 0.5x and 2x are the
        same size of defect. A defect that halves a flow is not milder than one
        that doubles it, and an arithmetic distance would say it was."""
        worst = 1.0
        for r in ratios:
            if not math.isfinite(r) or r <= 0.0:
                return math.inf
            if abs(math.log(r)) > abs(math.log(worst)):
                worst = r
        return worst

    def summary(self) -> dict:
        """A1-A4 are ratios whose TARGET IS 1.0 and whose worst reading is the
        one furthest from it in log space. A5 is not that shape: it is a strict
        inequality with no ideal value, so its summary is the MINIMUM over
        desks and it is read as "> 1 or the desk is deadlocked". Mixing the two
        conventions in one column would have made `inf` mean "catastrophe" in
        four places and "unconstrained" in the fifth."""
        return {
            "A1_labour": self._worst(v["ratio"] for v in self.a1.values()),
            "A2_goods": self._worst(v["ratio"] for v in self.a2.values()),
            "A3_money": self._worst(v["ratio"] for v in self.a3.values()),
            "A4_prices": self._worst(v["ratio"] for v in self.a4.values()),
            "A5_window": min((v["ratio"] for v in self.a5.values()), default=math.inf),
        }

    def clean(self, which: str) -> bool:
        r = self.summary()[which]
        if which == "A5_window":
            return r > 1.0
        return math.isfinite(r) and abs(r - 1.0) <= EXACT


def audit(d: pathlib.Path) -> Audit:
    t = read_tape(d)
    return Audit(t.name, a1_labour(t), a2_goods(t), a3_money(t), a4_prices(t),
                 a5_rule_window(t))


def fmt(r: float) -> str:
    if not math.isfinite(r):
        return "inf" if r > 0 else "-inf"
    if r == 0.0:
        return "0"
    return f"{r:.4g}"


def report(a: Audit, tape: Tape, verbose: bool = False) -> str:
    s = a.summary()
    lines = [f"{a.name:16s}  A1 labour {fmt(s['A1_labour']):>10s}   "
             f"A2 goods {fmt(s['A2_goods']):>10s}   "
             f"A3 money {fmt(s['A3_money']):>10s}   "
             f"A4 prices {fmt(s['A4_prices']):>10s}   "
             f"A5 window {fmt(s['A5_window']):>7s}"
             + ("" if s["A5_window"] > 1.0 else "  DEADLOCK")]
    if not verbose:
        return lines[0]
    for node, v in sorted(a.a1.items()):
        rn = next((r[0] for r in tape.regions.values() if r[1] == node), f"node{node}")
        lines.append(f"    A1 {rn:12s} demand {v['demand']:10.4g} / supply "
                     f"{v['supply']:10.4g} = {fmt(v['ratio'])}")
    for (node, g), v in sorted(a.a2.items(), key=lambda kv: (kv[0][0], kv[0][1])):
        rn = next((r[0] for r in tape.regions.values() if r[1] == node), f"node{node}")
        if abs(v["ratio"] - 1.0) > EXACT:
            lines.append(f"    A2 {rn:12s} {g:10s} supply {v['supply']:10.4g} / demand "
                         f"{v['demand']:10.4g} = {fmt(v['ratio'])}")
    for owner, v in sorted(a.a3.items()):
        if abs(v["ratio"] - 1.0) > EXACT:
            lines.append(f"    A3 {owner:16s} held {v['held']:10.4g} / need "
                         f"{v['need']:10.4g} = {fmt(v['ratio'])}")
    for g, v in sorted(a.a4.items()):
        imp = "n/a" if v["implied"] is None else f"{v['implied']:.6g}"
        lines.append(f"    A4 {g:12s} price {v['price']:10.6g} / implied {imp:>10s} "
                     f"= {fmt(v['ratio'])}  {v['note']}")
    for key, v in sorted(a.a5.items()):
        lines.append(f"    A5 {key:22s} buys below {v['buy_below']:9.4g}, posts above "
                     f"{v['post_above']:9.4g} = {fmt(v['ratio'])}"
                     + ("" if v["ratio"] > 1.0 else "   NO STOCK LEVEL DOES BOTH"))
    return "\n".join(lines)


# ══════════════════════════════════════════════════════════════════════════════
# The generator
# ══════════════════════════════════════════════════════════════════════════════
#
# Everything below SOLVES for the numbers the four conditions require, from a
# handful of free parameters, and then writes them. Nothing here is a number
# somebody chose to make a run look good; each is the unique value a condition
# forces, and the derivation is written into each tape's own header so the tape
# is auditable without this file.
#
# THE ARITHMETIC, ONCE, IN GENERAL FORM
#
#   technology       a_wf labour -> 1 wheat;  1 wheat + a_gm labour -> 1 flour;
#                    a_ls labour -> 1 services
#   contents         lam_w = a_wf;  lam_f = a_wf + a_gm;  lam_s = a_ls
#   prices (A4)      p = lam with p_labour = 1  (times `money`, which is a pure
#                    redenomination and must leave every real series alone)
#   basket           at genesis wealth w0, f flour and v services per unit of pop
#   A1 <=> f*lam_f + v*lam_s == 1
#         i.e. THE GENESIS BASKET EMBODIES EXACTLY ONE UNIT OF LABOUR PER HEAD.
#         This is the whole condition, and it is worth saying in words: with the
#         zero-profit price vector, nominal consumption per head equals the wage,
#         so A1 and A4 together are the circular flow closing. That is why the
#         basket table is the thing this generator solves, not the capacity.
#   capacities (A2)  K_mill = K_farm = f*H;  K_serv = v*H
#   stocks           (1 + b_out) * K for storables, so Rule 1 posts exactly K;
#                    K for the non-storable, whose offer is one tick's flow
#   cash (A3)        (b_cash/s + s) * outlay for firms;  b_cash * cost / s for pops
# ══════════════════════════════════════════════════════════════════════════════

# Technology, shared by every cr_* world so the corpus varies one thing at a time.
A_WF = 0.25   # labour per wheat
A_GM = 0.25   # labour per flour, on top of one wheat
A_LS = 1.0    # labour per unit of services
LAM = {"wheat": A_WF, "flour": A_WF + A_GM, "services": A_LS, "labour": 1.0}

# Kernel dials: the corpus's generation-4 values, copied verbatim from lr_00's
# `game_data.ron` and NOT retuned. `s` is the one this generator varies, and it
# varies it in a scenario whose header says why (cr_09).
KERNEL = {
    "s": 1, "eta_up": 0.04, "eta_dn": 0.05, "dead": 0.05, "b_out": 2.0,
    "b_cash": 6.5, "beta": 1.0, "epsilon": 0.01,
    "phi": 0.6180339887498949, "fill_alpha": 0.25, "supply_rule": "inelastic",
}

#: Genesis utilisation. Interior on purpose: a desk at `chosen == recipe_size`
#: has nowhere to expand to, so Rule 2's upward branch would be untestable and
#: `BldUtil` would sit on its ceiling. 0.8 is the value `solv_chain` registers.
UTIL = 0.8

#: Genesis wealth, interior between the two registered tiers for the same reason:
#: at a corner the consumption desk's Rule 2 can only move one way.
W0 = 0.5


@dataclass
class Spec:
    """One world. Free parameters only; everything else is solved."""

    name: str
    header: str
    sizes: list                       # pop size per region
    basket: dict                      # tier -> (flour, services) per unit of pop
    trade: bool = False
    s: int = 1
    money: float = 1.0                # pure redenomination of prices and cash
    participation: float = 1.0
    cash_mult: float = 1.0            # A3 break
    price_override: dict = field(default_factory=dict)   # A4 break
    capacity_mult: dict = field(default_factory=dict)    # A2 break, per recipe
    rentiers: bool = False


REGION_NAMES = ["Ashby", "Brindle", "Corwen"]


def basket_at(basket: dict, w: float) -> tuple:
    lo, hi = basket[0], basket[1]
    return (lo[0] + w * (hi[0] - lo[0]), lo[1] + w * (hi[1] - lo[1]))


def check_spec(spec: Spec) -> None:
    """Refuse to emit a world whose free parameters cannot satisfy A1.

    A1 is the one condition that constrains the FREE parameters rather than the
    solved ones: capacity, stock and cash all follow from the basket, but the
    basket itself has to embody exactly one unit of labour per head or no choice
    of capacity can balance the labour market. Checked here, before any file is
    written, because a tape that cannot be consistent should not exist.
    """
    f, v = basket_at(spec.basket, W0)
    embodied = f * LAM["flour"] + v * LAM["services"]
    if abs(embodied - 1.0) > 1e-12:
        raise SystemExit(
            f"{spec.name}: the genesis basket embodies {embodied!r} labour per head, "
            f"not 1.0 — A1 cannot hold at any capacity. Fix the basket, not the desks."
        )


def parity_of(spec: Spec) -> float:
    """`derive_parity`'s rule, applied to this world's own tier-0 row.

    Recomputed here rather than shelled out to, so a generated tape is complete
    the moment it is written; `tools/derive_parity.py --write` on the result is a
    no-op and `--selftest` checks that it is.
    """
    f0, v0 = spec.basket[0]
    embodied = f0 * LAM["flour"] + v0 * LAM["services"]
    if embodied <= 0.0:
        raise SystemExit(f"{spec.name}: tier-0 basket embodies no labour")
    return HOME_PENALTY / embodied


@dataclass
class Desk:
    bid: int
    region: int
    recipe: str
    chosen: float
    size: float
    inv: dict
    channel: int | None = None


def solve(spec: Spec) -> dict:
    """Capacities, stocks and cash that satisfy A1-A4 for this spec."""
    check_spec(spec)
    m = spec.money
    prices = {g: LAM[g] * m for g in ("wheat", "flour", "services")}
    prices["labour"] = 1.0 * m
    prices["GBP"] = 1.0
    prices.update({g: p * m for g, p in spec.price_override.items()})

    f, v = basket_at(spec.basket, W0)
    b_out = KERNEL["b_out"]
    b_cash, s = KERNEL["b_cash"], float(spec.s)

    def cap(recipe: str, base: float) -> float:
        return base * spec.capacity_mult.get(recipe, 1.0)

    desks_: list = []
    pops: list = []
    recipes = recipe_defs(spec)
    counter = [0]

    def add(region: int, recipe: str, chosen: float, stock: dict,
            channel: int | None = None) -> None:
        """One desk. `recipe_size = chosen / UTIL` ALWAYS, so genesis utilisation
        is the same free parameter in every world and no tape can ship a desk
        whose chosen size exceeds the cap Rule 2 clamps it to — which is what
        holding `recipe_size` fixed while scaling `chosen` would have produced in
        cr_05."""
        inv = {"GBP": 0.0}
        inv.update(stock)
        desks_.append(Desk(counter[0], region, recipe, chosen, chosen / UTIL, inv, channel))
        counter[0] += 1

    if spec.trade:
        # Specialisation: region 0 mills and does not farm, region 1 farms and
        # does not mill, region 2 is autarkic. Two costless channels move the
        # wheat one way and the flour the other.
        #
        # WHY THE CHANNELS CARRY NO LABOUR, which is a real limitation and not a
        # simplification for tidiness: `RawSimState.prices` is `HashMap<String,
        # f64>` and `NameResolver::prices` writes ONE price per good to EVERY
        # node. A tape therefore cannot state a per-node genesis price. Any
        # transport with a cost has a zero-profit wedge — `p_to = p_from +
        # a_tr * w` — so a world with a priced channel CANNOT open on its own
        # equilibrium vector in this format; it opens exactly `a_tr * w` below it
        # on every importing node, and every transport desk opens at a loss of
        # exactly that. That is a defect of `lr_*` too: its six `flour_transport`
        # desks each open with revenue-minus-cost of `-0.1 * w`.
        f1 = f * spec.sizes[1]
        milled = f * (spec.sizes[0] + spec.sizes[1])
        # region 0: the mill for both, sized to the pair's whole flour need
        add(0, "grain_mill", milled, {"flour": (1 + b_out) * milled})
        # region 1: the farm for both
        add(1, "wheat_farm", milled, {"wheat": (1 + b_out) * milled})
        # services, one desk per region — never traded, `Local` movement
        for r in range(3):
            ks = cap("local_services", v * spec.sizes[r])
            add(r, "local_services", ks, {"services": ks})
        # region 2 keeps its own chain
        k2 = f * spec.sizes[2]
        add(2, "wheat_farm", k2, {"wheat": (1 + b_out) * k2})
        add(2, "grain_mill", k2, {"flour": (1 + b_out) * k2})
        # channel 0: wheat, node 1 -> node 0, carrying the mill's whole input
        add(1, "wheat_haul", milled, {"wheat": (1 + b_out) * milled}, channel=0)
        # channel 1: flour, node 0 -> node 1, carrying region 1's whole basket
        add(0, "flour_haul", f1, {"flour": (1 + b_out) * f1}, channel=1)
    else:
        for r, H in enumerate(spec.sizes):
            kf = cap("wheat_farm", f * H)
            km = cap("grain_mill", f * H)
            ks = cap("local_services", v * H)
            add(r, "wheat_farm", kf, {"wheat": (1 + b_out) * kf})
            add(r, "grain_mill", km, {"flour": (1 + b_out) * km})
            add(r, "local_services", ks, {"services": ks})

    # A3: every desk's cash is solved from its own outlay at THIS TAPE'S OWN
    # prices, so a world with deliberately wrong prices (cr_07) still opens with
    # the cash those wrong prices demand and A3 stays clean while A4 alone fires.
    for d in desks_:
        outlay = sum(
            desired(q, sc, d.chosen, d.size) * prices[g]
            for g, q, sc in recipes[d.recipe]["inputs"] if g != "GBP"
        )
        d.inv["GBP"] = (b_cash / s + s) * outlay * spec.cash_mult

    # Pops. The worker pop's cash is Rule 3's reserve at sigma_c = 0 exactly.
    pid = 0
    basket_cost_unit = f * prices["flour"] + v * prices["services"]
    for r, H in enumerate(spec.sizes):
        pops.append({
            "id": pid, "region": r, "size": H, "wealth": W0,
            "gbp": b_cash * (basket_cost_unit * H) / s * spec.cash_mult,
            "labour": True, "parity": parity_of(spec),
            "participation": spec.participation,
        }); pid += 1
    if spec.rentiers:
        for r in range(len(spec.sizes)):
            size = RENTIER_SIZE[r]
            pops.append({
                "id": pid, "region": r, "size": size, "wealth": W0,
                "gbp": b_cash * (basket_cost_unit * size) / s * spec.cash_mult,
                "labour": False, "parity": None, "participation": 1.0,
            }); pid += 1

    return {"prices": prices, "desks": desks_, "pops": pops, "recipes": recipes,
            "f": f, "v": v}


#: The rentier pops' sizes, one per region. Deliberately lr's own numbers
#: (0.999 / 1.101 / 0.9) so `cr_10` is testing lr's SHAPE and not a size somebody
#: invented for the occasion.
RENTIER_SIZE = [0.999, 1.101, 0.9]

#: (from_node, to_node) for the trade world's two channels.
CHANNEL_DEFS = [(1, 0), (0, 1)]


def recipe_defs(spec: Spec) -> dict:
    r = {
        "wheat_farm": {
            "inputs": [("labour", A_WF, "Variable")],
            "outputs": [("wheat", 1.0)],
        },
        "grain_mill": {
            "inputs": [("wheat", 1.0, "Variable"), ("labour", A_GM, "Variable")],
            "outputs": [("flour", 1.0)],
        },
        "local_services": {
            "inputs": [("labour", A_LS, "Variable")],
            "outputs": [("services", 1.0)],
        },
        "dividend": {
            "inputs": [("GBP", 1.0, "Variable")],
            "outputs": [("GBP", 1.0)],
        },
    }
    if spec.trade:
        r["wheat_haul"] = {"inputs": [("wheat", 1.0, "Variable")],
                           "outputs": [("wheat", 1.0)]}
        r["flour_haul"] = {"inputs": [("flour", 1.0, "Variable")],
                           "outputs": [("flour", 1.0)]}
    return r


# ── writing ───────────────────────────────────────────────────────────────────


def n(x) -> str:
    """A float that round-trips. `repr` on a Python float is the shortest string
    that reads back to the same f64, which is the same guarantee Rust's `{:e}`
    gives and the one Python's `{:e}` does NOT (it truncates to 6 dp)."""
    x = float(x)
    r = repr(x)
    return r if ("." in r or "e" in r or "E" in r) else r + ".0"


def write_text(path: pathlib.Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


def emit_game_data(spec: Spec, sol: dict) -> str:
    recipes = sol["recipes"]
    lines = [spec.header.rstrip(), "("]
    lines.append("    // Kernel dials: lr_00's generation-4 values, copied verbatim and NOT")
    lines.append("    // retuned (R2, R6). `s` is the only one any cr_* world varies.")
    lines.append("    kernel: (")
    for k in ("s", "eta_up", "eta_dn", "dead", "b_out", "b_cash", "beta",
              "epsilon", "phi", "fill_alpha"):
        val = spec.s if k == "s" else KERNEL[k]
        lines.append(f"        {k}: {val if k == 's' else n(val)},")
    lines.append(f"        supply_rule: {KERNEL['supply_rule']},")
    lines.append("    ),")
    lines.append("    goods: [")
    for gname, alpha, shelf, move in (
        ("wheat", 0.1, "Indefinite", "Physical"),
        ("flour", 0.1, "Indefinite", "Physical"),
        ("GBP", 0.0, "Indefinite", "Financial"),
        ("labour", 0.05, "Instant", "Local"),
        ("services", 0.1, "Ticks(1)", "Local"),
    ):
        lines.append(f'        ( name: "{gname}", alpha: {n(alpha)}, shelf_life: {shelf}, '
                     f"movement_type: {move}, divisible: true, storage_cost_per_tick: 0.0 ),")
    lines.append("    ],")
    lines.append("    recipes: [")
    for rname, r in recipes.items():
        ins = ", ".join(
            f'( good: "{g}", qty_per_unit: {n(q)}, scaling: {sc} )' for g, q, sc in r["inputs"])
        outs = ", ".join(f'( good: "{g}", qty_per_unit: {n(q)} )' for g, q in r["outputs"])
        strat = ("DividendPayout( reserve_multiple: 26.0 )" if rname == "dividend"
                 else "CapacityControl")
        lines.append("        (")
        lines.append(f'            name: "{rname}",')
        lines.append(f"            inputs:  [ {ins} ],")
        lines.append(f"            outputs: [ {outs} ],")
        lines.append("            reversible: false,")
        lines.append(f"            strategy: {strat},")
        lines.append("        ),")
    lines.append("    ],")
    lines.append("    need_categories: [")
    lines.append('        ( name: "staple_food",     entries: [ ( good: "flour",    weight: 1.0 ) ] ),')
    lines.append('        ( name: "services_demand", entries: [ ( good: "services", weight: 1.0 ) ] ),')
    lines.append("    ],")
    lines.append("    // TWO tiers, not a hundred. The ladder is piecewise linear in `wealth`,")
    lines.append("    // so two rows already give a continuous basket over the whole range the")
    lines.append("    // consumption desk can reach; more rows would be more numbers to")
    lines.append("    // register without another degree of freedom (R2).")
    lines.append("    wealth_levels: [")
    for tier in (0, 1):
        bf, bv = spec.basket[tier]
        needs = [f'"staple_food": {n(bf)}']
        if bv != 0.0:
            needs.append(f'"services_demand": {n(bv)}')
        lines.append(f"        ( tier: {tier}, needs: {{ {', '.join(needs)} }} ),")
    lines.append("    ],")
    lines.append("    market_nodes: [")
    for i in range(3):
        lines.append(f"        ( id: MarketNodeId({i}), tier: Regional, "
                     f"region: Some(RegionId({i})), currency_good: Some(GoodId(2)) ),")
    lines.append("    ],")
    if spec.trade:
        lines.append("    channels: [")
        for i, (a, b) in enumerate(CHANNEL_DEFS):
            lines.append(f"        ( id: ChannelId({i}), from: MarketNodeId({a}), "
                         f"to: MarketNodeId({b}), channel_type: Trade ),")
        lines.append("    ],")
    else:
        lines.append("    channels: [],")
    lines.append("    regions: [")
    for i, name in enumerate(REGION_NAMES):
        lines.append(f'        ( id: RegionId({i}), name: "{name}", market_node: MarketNodeId({i}) ),')
    lines.append("    ],")
    lines.append(")")
    return "\n".join(lines) + "\n"


def emit_state(spec: Spec, sol: dict) -> str:
    lines = [state_header(spec, sol).rstrip(), "(", "    tick: 0,"]
    pr = ", ".join(f'"{g}": {n(p)}' for g, p in sol["prices"].items())
    lines.append(f"    prices: {{ {pr} }},")
    lines.append("    buildings: [")
    for d in sol["desks"]:
        inv = ", ".join(f'"{g}": {n(q)}' for g, q in d.inv.items() if q != 0.0)
        ch = "None" if d.channel is None else f"Some(({d.channel}))"
        lines.append("        (")
        lines.append(f"            id: ({d.bid}), region: ({d.region}), recipe: \"{d.recipe}\",")
        lines.append(f"            recipe_size: {n(d.size)}, chosen_size: {n(d.chosen)}, "
                     f"efficiency: 1.0,")
        lines.append(f"            inventory: {{ {inv} }},")
        lines.append(f"            channel: {ch},")
        lines.append("        ),")
    lines.append("    ],")
    lines.append("    pop_groups: [")
    for p in sol["pops"]:
        lines.append("        (")
        lines.append(f"            id: ({p['id']}), region: ({p['region']}), "
                     f"size: {n(p['size'])}, wealth: {n(p['wealth'])},")
        lines.append(f"            inventory: {{ \"GBP\": {n(p['gbp'])} }},")
        lines.append("            savings_target: 0.9,")
        if p["labour"]:
            lines.append('            labour_good: Some("labour"),')
            lines.append("            // Subsistence baskets per unit of labour: the pop's outside")
            lines.append("            // option. Derived from technology by tools/derive_parity.py's")
            lines.append("            // rule, never from prices, because RealWage is scored (R6).")
            lines.append(f"            parity: Some({n(p['parity'])}),")
            if p["participation"] != 1.0:
                lines.append(f"            participation: Some({n(p['participation'])}),")
        lines.append("        ),")
    lines.append("    ],")
    if spec.trade:
        # `ChannelState.occupant` is a RecipeInstanceId, not a BuildingId. They
        # coincide only because the loader creates one instance per building in
        # tape order; the entries are emitted in CHANNEL index order rather than
        # desk order so the mapping is explicit rather than incidental.
        lines.append("    channels: [")
        occ = {d.channel: d.bid for d in sol["desks"] if d.channel is not None}
        for ci in range(len(CHANNEL_DEFS)):
            lines.append(f"        ( regulatory_factor: 1.0, occupant: Some(({occ[ci]})) ),")
        lines.append("    ],")
    lines.append("    // One claim-holder pointer per desk, so Rule 3's overflow has somewhere")
    lines.append("    // to go from every firm. At the zero-profit vector the overflow is")
    lines.append("    // identically zero, so these move nothing until something moves first.")
    lines.append("    extra_recipe_instances: [")
    holders = [p["id"] for p in sol["pops"] if not p["labour"]] or None
    for d in sol["desks"]:
        if holders:
            target = next(p["id"] for p in sol["pops"]
                          if not p["labour"] and p["region"] == d.region)
        else:
            target = next(p["id"] for p in sol["pops"] if p["region"] == d.region)
        lines.append(f'        ( recipe: "dividend", building: ({d.bid}), '
                     f"output_pop: ({target}), recipe_size: 10000.0 ),")
    lines.append("    ],")
    lines.append(")")
    return "\n".join(lines) + "\n"


def state_header(spec: Spec, sol: dict) -> str:
    a = []
    f, v = sol["f"], sol["v"]
    a.append(f"// Genesis for {spec.name}. SOLVED, not chosen: every number below is the")
    a.append("// unique value one of the four consistency conditions requires at this")
    a.append("// world's free parameters. tools/gen_regions.py --audit re-derives all four")
    a.append("// from these files alone and prints the ratios.")
    a.append("//")
    a.append(f"//   basket at wealth {n(W0)}   {n(f)} flour + {n(v)} services per unit of pop")
    a.append(f"//   embodied labour       {n(f)}*{n(LAM['flour'])} + {n(v)}*{n(LAM['services'])} "
             f"= {n(f * LAM['flour'] + v * LAM['services'])} per head   (A1)")
    a.append(f"//   prices                labour {n(sol['prices']['labour'])}, "
             f"wheat {n(sol['prices']['wheat'])}, flour {n(sol['prices']['flour'])}, "
             f"services {n(sol['prices']['services'])}   (A4)")
    a.append(f"//   stocks                (1 + b_out) * chosen = {n(1 + KERNEL['b_out'])} * chosen, "
             "so Rule 1 posts exactly one tick's flow")
    a.append(f"//   firm cash             (b_cash/s + s) * outlay = "
             f"{n(KERNEL['b_cash'] / spec.s + spec.s)} * outlay   (A3)")
    a.append(f"//   pop cash              b_cash * basket_cost / s, so sigma_c = 0 exactly   (A3)")
    return "\n".join(a)


def emit(spec: Spec, out: pathlib.Path) -> None:
    sol = solve(spec)
    d = out / spec.name
    d.mkdir(parents=True, exist_ok=True)
    write_text(d / "game_data.ron", emit_game_data(spec, sol))
    write_text(d / "starting_state.ron", emit_state(spec, sol))
    # The criteria file is copied BYTE FOR BYTE from the registered baseline. A
    # corpus scored against a criteria file of its own making would be a corpus
    # scored against a bar chosen after the world (R6), and the whole comparison
    # against lr_* depends on both being read by one generation-4 file.
    shutil.copyfile(SCEN / "lr_00" / "criteria.ron", d / "criteria.ron")
    write_text(d / "events.ron",
               "// No tape. A world built to sit at a known point must be left alone:\n"
               "// anything that then happens is the engine's doing, not an event's.\n"
               "(\n    events: [],\n    recurring: [],\n)\n")


# ══════════════════════════════════════════════════════════════════════════════
# The corpus
# ══════════════════════════════════════════════════════════════════════════════
#
# Eleven worlds in three classes. The classes are the design: comparing a
# consistent corpus against `lr_*` compares two DIFFERENT WORLDS and could not
# separate "consistency helped" from "these happen to be nicer tapes". The five
# defect worlds are the control that can — each is `cr_00` with a named condition
# broken and nothing else touched, so the difference between cr_00 and cr_04 is
# attributable in a way the difference between cr_00 and lr_00 never is.
#
#   CONSISTENT   cr_00 base   cr_01 trade   cr_02 services-heavy   cr_03 2x money
#   DEFECT       cr_04 A1     cr_05 A2      cr_06 A3               cr_07 A4
#                cr_08 all four at once
#   PROBE        cr_09 s=4    cr_10 rentiers

BASE_SIZES = [32.0, 24.0, 40.0]

#: flour and services per unit of pop, at tiers 0 and 1. Solved so that
#: `f*lam_flour + v*lam_services == 1` at W0 — see `check_spec`.
BASE_BASKET = {0: (1.0, 0.0), 1: (1.5, 0.75)}
#: Same condition, a different split: a quarter of the basket's labour in flour
#: and three quarters in services, where the base is 5/8 and 3/8.
SERVICES_BASKET = {0: (0.5, 0.25), 1: (0.5, 1.25)}


def _h(title: str, body: str) -> str:
    return f"// {title}\n//\n" + "\n".join(f"// {ln}" if ln else "//"
                                           for ln in body.strip().split("\n")) + "\n"


SPECS = [
    Spec(
        name="cr_00", sizes=BASE_SIZES, basket=BASE_BASKET,
        header=_h(
            "cr_00 — the base world: three autarkic regions, all four conditions exact.",
            """
THE FREE PARAMETERS, and they are the only things chosen here: three pop sizes
(32 / 24 / 40), the technology coefficients, the two basket rows, and genesis
utilisation 0.8. Everything else — every capacity, every stock, every cash
balance, every price — is SOLVED. tools/gen_regions.py holds the derivation.

WHAT THIS WORLD IS. Per unit of pop the basket is 1.25 flour + 0.375 services at
the genesis wealth 0.5, which embodies 1.25*0.5 + 0.375*1.0 = 1.0 unit of labour
— so one head's consumption takes exactly one head's hours, and A1 holds at
every capacity that meets the demand. At the zero-profit price vector (labour 1,
wheat 0.25, flour 0.5, services 1) that same statement is the circular flow
closing in money: each pop spends exactly its wage.

Region 0 (pop 32): farm 40, mill 40, services 12; labour 10 + 10 + 12 = 32.
Region 1 (pop 24): farm 30, mill 30, services  9; labour 7.5 + 7.5 + 9 = 24.
Region 2 (pop 40): farm 50, mill 50, services 15; labour 12.5 + 12.5 + 15 = 40.

THREE AUTARKIC REGIONS RATHER THAN ONE, on purpose: the corpus's failure classes
are scored per region and the A/B's unit is a region, so a one-region world would
have made this corpus a third the size of the baseline it is compared to. The
regions differ in size, so they are not three copies of one draw.

WHAT IT IS EXPECTED TO DO, WRITTEN BEFORE THE RUN. A1-A4 at exact equality make
genesis a FIXED POINT of the kernel's rules: every fill is 1, every margin is 0,
every imbalance is 0, sigma_c is 0, and pi is at its corner. So the kernel arm
should not move at all, and if it does the defect is in the engine or in this
derivation, not in the economy. THAT IS A NECESSARY CONDITION AND NOT AN
INTERESTING ONE — Phase 4's RESULT 2 already established that the mechanism can
hold a fixed point it cannot walk to. The interesting readings are the legacy arm
here (which has no reason to hold it) and the five defect worlds.

>>> MEASURED, AND THE PASS IS WORTH LESS THAN IT LOOKS. Recorded here rather
>>> than in a notebook, because anyone reading this tape's green certificate
>>> needs this paragraph with it.
>>>
>>> The kernel holds cr_00 exactly: 8/8 batteries, 3/3 regions, B8 1.000x with
>>> sd 0.000, `LevelRange` 1.000. The legacy arm cannot: band 4.4e5 on all three
>>> regions, none of them alive. So the fixed point is real and only one arm can
>>> sit on it.
>>>
>>> THERE IS NO BASIN AROUND IT. THERE IS A DEAD BAND. Scaling the genesis flour
>>> price by x and re-solving the cash (so A1-A5 stay exact except A4):
>>>
>>>     x1.00  x1.01  x1.02  x1.05  |  x1.10   x1.20   x1.50   x2.00
>>>     PASS   PASS   PASS   PASS   |  3.4e9   1.0e29  6.3e9   5.9e26   worst band
>>>
>>> and inside the passing region B8 reads 1.010x, 1.020x, 1.050x too dear with
>>> **sd 0.000** — THE DISPLACED PRICE NEVER MOVES. Those are full-PASS
>>> certificates on a world that is permanently and exactly 5% wrong. Nothing
>>> converges; the price freezes wherever it starts and the criteria call that
>>> stability.
>>>
>>> The edge is not empirical, it is `dead`. The mill's relative margin is
>>> (p_f - 0.5)/((p_f + 0.5)/2), which reaches dead = 0.05 at p_f = 1.025/1.95,
>>> i.e. x1.051282. Bisected AFTER deriving it: x1.0512 PASS, x1.0514 FAIL.
>>>
>>> So cr_00's green certificate says exactly one thing — the fixed point exists
>>> and the kernel can sit on it — and it must not be read as saying the
>>> mechanism finds prices. Phase 4's RESULT 2 said the mechanism "cannot walk
>>> to" a fixed point one factor of two away. This is sharper: it cannot walk to
>>> one 6% away, and inside 5% it does not walk at all.
"""),
    ),
    Spec(
        name="cr_01", sizes=[32.0, 32.0, 40.0], basket=BASE_BASKET, trade=True,
        header=_h(
            "cr_01 — the same conditions with REGIONAL SPECIALISATION and trade.",
            """
Region 0 mills and does not farm; region 1 farms and does not mill; region 2 is
autarkic. Two channels carry the wheat one way and the flour the other.

THE LABOUR CONDITION FORCES H0 = H1, and that is the whole design. Region 0's
hours go to milling both regions' flour plus its own services; region 1's go to
growing both regions' wheat plus its own services. Writing A1 for each and using
a_wf = a_gm gives 0.25*(F0+F1) = 0.625*H0 and the same with H1, so the two pops
must be the same size. A trade pattern is not free: it is a constraint on who can
live where, and this is the smallest world in which that shows up.

THE CHANNELS CARRY NO LABOUR, AND THAT IS A LIMITATION OF THE TAPE FORMAT, NOT A
SIMPLIFICATION. `RawSimState.prices` is one price per good, written to every
node by `NameResolver::prices`, so a tape cannot state a per-node genesis price.
A priced transport has a zero-profit wedge p_to = p_from + a_tr*w, so a world
with one CANNOT open on its own equilibrium vector in this format: it opens
exactly a_tr*w low on every importing node and every transport desk opens at a
loss of exactly that. lr_* has six such desks per scenario, each opening at
-0.1*w. A costless channel is the only trade structure this format can start
consistent, so that is what this world uses, and the limitation is recorded here
rather than worked around silently.

PRE-REGISTERED PREDICTION: the specialised regions have no local substitute, so
this world's failure mode if it has one should be a TRANSMISSION failure — a
price move in one region reaching the other through the channel — and not the
starvation modes cr_04/cr_05 are built to produce.

>>> THAT PREDICTION IS WRONG, AND IT IS MARKED IN PLACE RATHER THAN REWRITTEN
>>> (R14). What happened was not a transmission failure but a DEADLOCK, and it
>>> is a defect of the KERNEL'S RULES, not of this world. See `a5_rule_window`
>>> in tools/gen_regions.py, which was written because of this run.
>>>
>>> A pass-through desk reads ONE inventory slot with two opposite meanings:
>>> Rule 1 posts stock above `b_out*chosen*q_out` while Rule 3 buys only below
>>> `s*q_in*chosen`. With b_out = 2 and s = 1 the second bound is BELOW the
>>> first, so no stock level does both. Stocked to post — which is what this
>>> generator solved for — the haul sells down to its band and then never buys
>>> again. MEASURED, wheat cleared at the node the haul BUYS at, over 12 ticks:
>>>     s=1  0.00      s=2  18.77      s=3  93.32      s=4  86.77
>>> The boundary is exactly `b_out < s`, where the arithmetic puts it.
>>>
>>> THE TAPE IS LEFT AT s = 1 DELIBERATELY. Bumping it to 3 would hide the
>>> finding behind a dial, and the world is more useful as it is: A1-A4 all read
>>> 1.000 and its AUTARKIC region (Corwen) passes the full criteria under the
>>> kernel while its two traded regions die. One tape, one difference, and the
>>> difference is the channel.
>>>
>>> IT ALSO EXPLAINS SOMETHING ALREADY IN THE RECORD. Phase 4 traced lr's
>>> collapse to "flour has zero posted supply from tick 0 — that part is
>>> pre-existing" without saying why. lr's transports open with no flour, so
>>> they post nothing and the importing node's imbalance pins at +1 for ever.
>>> lr's window IS open (s=4, b_out=2), so seeding them would have helped there
>>> — which is a concrete, cheap experiment this finding hands to Phase 5.
""")),
    Spec(
        name="cr_02", sizes=BASE_SIZES, basket=SERVICES_BASKET,
        header=_h(
            "cr_02 — consistent, with the basket's labour moved into SERVICES.",
            """
Same technology, same conditions, a different split of the same one unit of
labour per head: 0.5 flour + 0.75 services at genesis wealth, so a quarter of
the basket's embodied labour is in the storable good and three quarters in the
non-storable one. The base world is 5/8 and 3/8.

WHY THIS IS THE SECOND FREE-PARAMETER AXIS AND NOT, SAY, ANOTHER SIZE VECTOR.
Services are `Ticks(1)`, so `GameData::storable` is false for them and Rule 1
posts `min(flow, stock)` — a VERTICAL supply curve, price elasticity exactly
0.000 under every supply rule, which `examples/elasticity_probe.rs` measured and
`price-responsive-supply.md` §2.3 predicted. Moving weight into services moves
this world's share of markets that have no supply response at all, which is the
one structural knob Phase 4 identified and never swept.

PRE-REGISTERED PREDICTION: if the elasticity story is right, cr_02 is harder than
cr_00 for both arms, and harder by more under the kernel than under legacy.
""")),
    Spec(
        name="cr_03", sizes=BASE_SIZES, basket=BASE_BASKET, money=2.0,
        header=_h(
            "cr_03 — cr_00 REDENOMINATED: every price and every cash balance doubled.",
            """
The control, not a scenario. This world's real economy is cr_00's exactly: same
pops, same capacities, same stocks, same baskets, same technology. Only the unit
of account differs. R12 and the money-neutrality result (PLAN, RESULT 4) require
every real series to be BIT-IDENTICAL to cr_00's and every price and currency
balance to be exactly 2x.

IT IS HERE TO CATCH THIS GENERATOR, not the engine. If any number emitted above
had been a currency amount that did not scale — a cash floor, a reserve written
as an absolute, a price used where a ratio was meant — this world and cr_00 would
diverge and the divergence would be the generator's fault. `solv_1g_money_2x`
makes the same check against a hand-written tape; this one makes it against a
solved one.

ONE KNOWN ABSOLUTE CONSTANT EXISTS in the engine: `overflow > 1e-12` in
`rule_3_buy_and_route`. It cannot bind here, because at a zero-profit fixed point
the overflow is identically zero at either denomination.
""")),
    Spec(
        name="cr_04", sizes=BASE_SIZES, basket=BASE_BASKET, participation=0.45,
        header=_h(
            "cr_04 — DEFECT A1: the labour market cannot balance. Nothing else moves.",
            """
cr_00 with the pops' genesis `participation` set to 0.45 and NOT ONE OTHER
NUMBER CHANGED. Capacities, stocks, cash, prices, baskets: identical files apart
from that field. So labour demand is cr_00's and labour supply is 0.45 of it —
a mismatch of 2.222x, against the 2.07x `lr_00` opens with and the 180x Phase 4
measured after its margin had retreated.

THIS IS THE ONE DEFECT THIS CORPUS CAN ISOLATE PERFECTLY, and it is isolated on
purpose: `participation` is the only registered field that moves labour supply
without moving consumption, capacity, outlay or price. `--audit` should read
A1 = 2.222 and A2 = A3 = A4 = 1.000 exactly. If it reads anything else on the
other three, this file has a bug.

WHY 0.45 AND NOT lr's 2.07x EXACTLY: 1/2.07 = 0.483 is a number chosen to
reproduce a measurement, and 0.45 is a round free parameter. The distinction
matters because a defect sized to match a target is a defect that has been tuned
(R6). The audit reports the ratio either way.

PRE-REGISTERED PREDICTION: labour is `Instant` and its posted supply is `pi*H` —
price elasticity exactly zero — so NO WAGE CLEARS THIS MARKET, which is Phase 4's
central finding reproduced on purpose in a world that is otherwise perfect.
Rule 3's buyers should ration, every desk should be driven toward 0.45 of its
scale, and the wage should run to a corner. The prediction is that this world is
UNSALVAGEABLE rather than merely unstable, and that no arm and no price rule
rescues it.
""")),
    Spec(
        name="cr_05", sizes=BASE_SIZES, basket=BASE_BASKET,
        capacity_mult={"wheat_farm": 0.5, "grain_mill": 0.5, "local_services": 1.8333333333333333},
        header=_h(
            "cr_05 — DEFECT A2: the goods balance broken, with A1 HELD.",
            """
cr_00 with the flour chain halved and the services desks enlarged to take up
exactly the labour the chain released, so the labour market still balances to the
last hour while the goods markets do not:

    farm and mill   chosen x 0.5    flour supply 0.5x its demand
    services        chosen x 1.8333 services supply 1.8333x its demand
    labour          0.25*20 + 0.25*20 + 22 = 32 = supply, unchanged

WHY IT IS BUILT THIS WAY. The four conditions are COUPLED — consumption drives
capacity drives labour drives outlay drives cash — so a naive "halve the mill"
would have moved A1 and A3 too and the run would have been uninterpretable.
Holding A1 fixed while breaking A2 is the only way to ask what a goods imbalance
does on its own. A3 stays exact because every desk's cash is solved from ITS OWN
outlay at its own scale, so the enlarged services desks open with the larger
balance they need and the halved chain with the smaller one; A4 is a price
condition and quantities cannot touch it. Measured, not asserted:
`--audit` reads A1 = A3 = A4 = 1.000 and A2 = 0.5.

This is `lr_00`'s shape: it mills 16 flour against 30 flour of demand (0.53x)
while its services desks draw 50 labour to make a good its tier-0 basket does not
contain at all.

PRE-REGISTERED PREDICTION: the flour market is short and the services market is
long, and the price rule's response is asymmetric between them — flour is
storable and marks up against a real shortage, services are `Ticks(1)` and expire
unsold, so the services desks should be the ones that die.
""")),
    Spec(
        name="cr_06", sizes=BASE_SIZES, basket=BASE_BASKET, cash_mult=0.125,
        header=_h(
            "cr_06 — DEFECT A3: every owner opens with an eighth of the cash it needs.",
            """
cr_00 with every currency balance — firms and pops alike — multiplied by 0.125,
and nothing else touched. The real economy is unchanged; only the money stock is
wrong. `--audit` should read A3 = 0.125 and A1 = A2 = A4 = 1.000 exactly: the
second perfectly isolated defect in this corpus.

WHY IT MATTERS AND WHY THE FACTOR IS BELOW ONE. Rule 3's budget is
`max(cash - reserve, 0)` with `reserve = b_cash * outlay / s`. At 0.125 of the
solved balance every desk sits below its reserve, so its budget is exactly zero:
it buys nothing, produces nothing, earns nothing, and cannot climb out. That
ABSORBING STATE is recorded in `src/kernel/mod.rs`'s header as a known property
of the design that the corpus's numbers happen not to reach, and it is recorded
there rather than guarded because a guard would hide it. This world reaches it on
purpose, so the phase map has one tape that starts inside it.

PRE-REGISTERED PREDICTION: instant death under the kernel arm, and a slower one
under legacy, whose spending is sized from `savings_target` rather than from a
reserve band and so has no hard zero.
""")),
    Spec(
        name="cr_07", sizes=BASE_SIZES, basket=BASE_BASKET,
        price_override={"wheat": 0.35, "flour": 0.6},
        header=_h(
            "cr_07 — DEFECT A4: genesis prices off the technology's implied vector.",
            """
cr_00 opened at `lr_00`'s literal genesis prices, wheat 0.35 and flour 0.60,
against this technology's implied 0.25 and 0.50 — so 1.4x and 1.2x too dear
against labour. Those two numbers are lr's own and were not chosen here; using
the baseline corpus's actual opening vector is what keeps this from being a
defect sized to produce an effect (R6).

A3 IS RE-SOLVED AT THE WRONG PRICES, deliberately. Outlays and basket costs are
priced, so leaving the cash at cr_00's values would have broken A3 as well and
confounded the two. Every cash balance here is the one Rule 3 requires AT THIS
TAPE'S OWN PRICES, so `--audit` reads A3 = 1.000 and only A4 fires. A1 and A2 are
quantity conditions and are untouched by construction.

WHAT THIS ISOLATES. Every CapacityControl desk is trying to find the zero-profit
vector; here it starts 1.2-1.4x away from it, in the direction that gives the
mill and the farm a positive paper margin at genesis. Rule 2 gates expansion on
margin, so the pre-registered prediction is that BOTH chains expand into a labour
market that cannot supply them, and the world's first move is an inflation rather
than a shortage.
""")),
    Spec(
        name="cr_08", sizes=BASE_SIZES, basket=BASE_BASKET, participation=0.45,
        cash_mult=0.125, price_override={"wheat": 0.35, "flour": 0.6},
        capacity_mult={"wheat_farm": 0.5, "grain_mill": 0.5,
                       "local_services": 1.8333333333333333},
        header=_h(
            "cr_08 — ALL FOUR DEFECTS AT ONCE: an lr-shaped world from this generator.",
            """
cr_04's participation, cr_05's capacities, cr_06's cash multiplier and cr_07's
prices, in one tape. It exists so the corpus contains a world that is broken the
way `lr_*` is broken but shares cr_00's technology, basket, region sizes and
criteria file — which is the only fair way to ask whether the difference between
the two corpora is CONSISTENCY or is just two different sets of worlds.

If cr_08 behaves like lr and cr_00 does not, the difference is the conditions. If
cr_08 behaves like cr_00, the conditions are not what separates them and this
whole exercise has produced a negative result, which is the more valuable of the
two answers and is to be reported as such.
""")),
    Spec(
        name="cr_09", sizes=BASE_SIZES, basket=BASE_BASKET, s=4,
        header=_h(
            "cr_09 — PROBE: cr_00 with the stagger period s = 4, lr's registered value.",
            """
Cash is re-solved for s = 4 — `(b_cash/s + s) * outlay = 5.625 * outlay` for
firms, `b_cash * cost / 4` for pops — so A3 holds. A1, A2 and A4 are untouched.
Every other number is cr_00's.

A PRE-REGISTERED PREDICTION, AND IT IS A PREDICTION OF FAILURE. Reading the
engine rather than running it: `rule_3_buy_and_route` fires only on a desk's
phase tick and buys `desired * s` — S ticks of inputs at once — while
`production::run` consumes `desired * 1` every tick. That is fine for a storable
input. `labour` is `ShelfLife::Instant`, so `initial_life()` is `Some(0)` and
`spoil_lots` removes the lot at the END OF THE TICK IT ARRIVED IN. Therefore, at
s = 4:

    a desk buys 4 ticks of labour, produces with 1, and has the other 3
    DESTROYED the same tick; on the next three ticks it is off-phase, holds no
    labour, and produces nothing at all.

If that reading is right the desk pays 4x the labour bill for 1x the output and
sits idle three ticks in four, and NO tape can be consistent at s > 1 while any
input is non-storable — the condition A1 is written against (per-tick demand)
is not the quantity Rule 3 actually buys. cr_09 against cr_00 is the measurement
that settles it, and if it comes out the other way this paragraph is wrong and
should be marked so in place rather than deleted (R14).

>>> MEASURED, AND THE PREDICTION IS HALF WRONG. Marked in place (R14).
>>>
>>> The collapse is real and it is the worst in this corpus: all three regions
>>> DEAD, bands 1.8e42 to 9.6e43 against cr_00's 1.000, worse than every one of
>>> the five NAMED defect worlds. Labour cleared at Ashby over 40 ticks is
>>> 373.8 against cr_00's 1280.0 — 29% of the throughput of the identical
>>> economy at s = 1.
>>>
>>> BUT SPOILAGE IS NOT THE BINDING MECHANISM, and the falsification test says
>>> so. Re-run with labour's `shelf_life` changed from `Instant` to `Ticks(8)`,
>>> so a purchased hour survives the whole activation period: the world still
>>> dies, all three regions still DEAD, and labour cleared FALLS slightly, to
>>> 329.2. (Flour cleared does roughly double, 245 -> 505, so the waste is real
>>> — it is just not what kills it.)
>>>
>>> THE ACTUAL MECHANISM IS THE ONE THE PROBE EXPOSED: a non-storable input's
>>> supply is a PER-TICK FLOW and Rule 3 asks for S ticks of it in a single
>>> tick. Region 0's three desks sit on inventory indices 0,1,2, so exactly one
>>> of them buys on each of three ticks in four and NONE buys on the fourth.
>>> Measured labour demand at Ashby, ticks 1-4: 40, 0, 48, 40, against a flat
>>> supply of 32. Clearing is min(supply, demand) per tick, so the cycle takes
>>> 32+0+32+32 = 96 of the 128 hours the economy needs, at genesis, before any
>>> dynamics — and the rationing compounds from there down to 29%.
>>>
>>> Making the hours keep does not help because the pop's hours on a desk's
>>> OFF ticks are never bought at all. It is the lumping, not the perishing.
>>>
>>> CONSEQUENCE, and it is a real one for the phase map: `s` is not a free
>>> stability dial. Against a flow-supplied input it is a throughput divisor,
>>> and lr's registered s = 4 imposes it on every scenario in the baseline
>>> corpus. The two `s`-sensitive findings here point OPPOSITE ways — A5 needs
>>> `s > b_out` for a channel to function, this needs `s = 1` for a labour
>>> market to clear — so with b_out = 2 the two cannot be satisfied at once and
>>> the corpus has no admissible `s` while both a channel and a labour market
>>> are present. That is a design contradiction, not a tuning problem.
""")),
    Spec(
        name="cr_10", sizes=BASE_SIZES, basket=BASE_BASKET, rentiers=True,
        header=_h(
            "cr_10 — PROBE: a rentier pop per region, lr's shape, and why it cannot balance.",
            """
cr_00 plus one non-labouring pop per region at lr's own sizes (0.999 / 1.101 /
0.9), holding the cash Rule 3 sizes for its basket and receiving every desk's
overflow through the dividend pointers. Capacities are NOT enlarged to feed
them, and that is the finding rather than an oversight:

    A rentier consumes a basket that embodies labour. At the zero-profit price
    vector there is NO profit, so the overflow that funds it is identically
    zero — its income stream does not exist. A2 therefore breaks by exactly
    their consumption (`--audit` reads 0.956, the worst region being Brindle,
    where 1.101 rentiers sit against 24 workers). A1 does NOT break, and the
    reason is worth stating: rentiers demand no labour and supply none, so the
    labour market is cr_00's untouched. THE DEFECT IS PURELY ONE OF GOODS AND
    INCOME. Enlarging the capacity to feed them would move the defect into A1
    instead; it cannot remove it from both.

A CLASS THAT CONSUMES WITHOUT WORKING IS CONSISTENT ONLY IN A WORLD WITH A
SURPLUS, and a zero-profit world has none by definition. `lr_*` carries exactly
this shape — three elite pops at wealth 10.0 funded by `DividendPayout` — and it
is worth being explicit that this is a MODELLING gap and not an arithmetic one:
rent, land and the claim register are Phase 6-7 work, and until they exist a
rentier is a pop with a cash balance and a countdown.

PRE-REGISTERED PREDICTION: the rentiers exhaust their cash in `b_cash / s` = 6.5
ticks and then buy nothing, and POP_DESTITUTION trips on `RealIncome`, which is
cash-per-head over all pops the metric counts as employed — and `is_employed` is
true for every raw pop the loader reads, whether or not it registers a
`labour_good`, so the rentiers are inside that average.
""")),
]


# ══════════════════════════════════════════════════════════════════════════════
# Falsification
# ══════════════════════════════════════════════════════════════════════════════


def selftest() -> int:
    """Show every check FIRING on a defect before trusting it to report clean.

    A guard that cannot fail is not a guard. Each condition below is checked
    three ways: it reads exactly 1.000 on a world built to satisfy it, it reads
    the CONSTRUCTED ratio on a world built to break it, and — the part that
    matters — the OTHER THREE still read 1.000 on that same broken world. A check
    that fires on everything says nothing about anything.
    """
    import tempfile

    failures = []

    def check(label: str, cond: bool, detail: str = "") -> None:
        print(f"  {'PASS' if cond else 'FAIL'}  {label}" + (f"   {detail}" if detail else ""))
        if not cond:
            failures.append(label)

    with tempfile.TemporaryDirectory() as tmp:
        out = pathlib.Path(tmp)
        for spec in SPECS:
            emit(spec, out)
        auds = {s.name: audit(out / s.name) for s in SPECS}

        def shown(a: Audit) -> str:
            return " ".join(f"{k}={fmt(v)}" for k, v in a.summary().items())

        # 1. The consistent worlds read exactly 1.000 on all four conditions
        #    about the WORLD. `cr_01` is listed here and NOT in the A5 group on
        #    purpose: it is a world whose economy is exactly consistent and whose
        #    channels the kernel's own rules cannot operate. Keeping those two
        #    statements apart is the whole reason A5 is a separate check.
        for name in ("cr_00", "cr_01", "cr_02", "cr_03", "cr_09"):
            check(f"{name}: A1-A4 exact",
                  all(auds[name].clean(k) for k in
                      ("A1_labour", "A2_goods", "A3_money", "A4_prices")),
                  shown(auds[name]))
        for name in ("cr_00", "cr_02", "cr_03", "cr_09"):
            check(f"{name}: A5 unconstrained (no pass-through desk)",
                  auds[name].clean("A5_window"), shown(auds[name]))

        # 2. Each defect world fires the condition it was built to break…
        expect = {
            "cr_04": ("A1_labour", 1.0 / 0.45),
            "cr_06": ("A3_money", 0.125),
            "cr_07": ("A4_prices", 1.4),
        }
        for name, (which, ratio) in expect.items():
            got = auds[name].summary()[which]
            check(f"{name}: {which} fires at the constructed ratio",
                  abs(got - ratio) <= 1e-9, f"{fmt(got)} vs {fmt(ratio)}")

        # 3. …and does NOT disturb the other three. This is the falsification
        #    control: without it, a check that always fires would pass step 2.
        for name, (which, _r) in expect.items():
            others = [k for k in auds[name].summary() if k != which]
            check(f"{name}: every other condition stays clean",
                  all(auds[name].clean(k) for k in others), shown(auds[name]))

        # 4. cr_05 breaks A2 while HOLDING A1 — the coupled case, stated
        #    explicitly because the label claims something the audit must confirm.
        s5 = auds["cr_05"].summary()
        check("cr_05: A2 fires while A1 is held exact",
              abs(s5["A1_labour"] - 1.0) <= EXACT and abs(s5["A2_goods"] - 0.5) <= 1e-9,
              " ".join(f"{k}={fmt(v)}" for k, v in s5.items()))

        # 5. cr_10's rentiers break A2 by EXACTLY their own consumption, and
        #    break NOTHING ELSE. The A1 half of this was written the other way
        #    round first — "rentiers break A1 and A2" — and the audit said no:
        #    a pop that neither supplies nor demands labour leaves the labour
        #    market alone. Kept here as the record of a prediction the
        #    instrument corrected, which is what the instrument is for.
        s10 = auds["cr_10"].summary()
        # Brindle is the worst region: the smallest pop carries the largest
        # rentier, so its goods ratio is the furthest from 1 in log terms.
        expect_a2 = BASE_SIZES[1] / (BASE_SIZES[1] + RENTIER_SIZE[1])
        check("cr_10: A2 fires at exactly the rentiers' share; A1/A3/A4 exact",
              abs(s10["A2_goods"] - expect_a2) <= 1e-9
              and all(abs(s10[k] - 1.0) <= EXACT
                      for k in ("A1_labour", "A3_money", "A4_prices")),
              " ".join(f"{k}={fmt(v)}" for k, v in s10.items())
              + f"  (expected A2 {expect_a2:.6g})")

        # 6. The 2x world is a pure redenomination: identical real numbers.
        t0, t3 = read_tape(out / "cr_00"), read_tape(out / "cr_03")
        same_real = all(
            a["chosen_size"] == b["chosen_size"] and a["recipe_size"] == b["recipe_size"]
            and {k: v for k, v in a["inv"].items() if k != "GBP"}
            == {k: v for k, v in b["inv"].items() if k != "GBP"}
            for a, b in zip(t0.instances, t3.instances)
        )
        doubled = all(
            abs(t3.prices[g] - 2.0 * t0.prices[g]) <= 0.0
            for g in t0.prices if g != "GBP"
        ) and all(
            abs(b["inv"].get("GBP", 0.0) - 2.0 * a["inv"].get("GBP", 0.0)) <= 0.0
            for a, b in zip(t0.instances, t3.instances)
        )
        check("cr_03: real numbers identical to cr_00, currency exactly 2x",
              same_real and doubled)

        # 7. The generator agrees with tools/derive_parity.py on `parity`.
        #    Two Leontief solves that disagree would put a different outside
        #    option in the tape from the one the docs derive.
        import derive_parity
        for name in ("cr_00", "cr_02"):
            par, _note = derive_parity.derive(out / name)
            written = next(p["parity"] for p in read_tape(out / name).pops
                           if p["parity"] is not None)
            check(f"{name}: written parity == derive_parity.py's",
                  par is not None and abs(par - written) <= 1e-12, f"{par} vs {written}")

        # 8. A5 fires on the world it was written for, is silent where no desk
        #    has a pass-through good, and — the falsification that matters — its
        #    verdict FLIPS on the one dial the derivation says decides it. If it
        #    said "deadlock" at every `s` it would be measuring nothing.
        check("cr_01 (s=1, b_out=2): A5 says no stock level both posts and buys",
              auds["cr_01"].summary()["A5_window"] <= 1.0,
              f"A5={fmt(auds['cr_01'].summary()['A5_window'])}")
        check("cr_00 (no channels): A5 unconstrained",
              math.isinf(auds["cr_00"].summary()["A5_window"]))
        for s_val, want_open in ((1, False), (2, False), (3, True), (4, True)):
            probe = out / f"_a5_probe_s{s_val}"
            shutil.rmtree(probe, ignore_errors=True)
            shutil.copytree(out / "cr_01", probe)
            gp = probe / "game_data.ron"
            txt = gp.read_text(encoding="utf-8")
            gp.write_text(txt.replace("        s: 1,", f"        s: {s_val},"),
                          encoding="utf-8")
            got = audit(probe).summary()["A5_window"]
            check(f"cr_01 at s={s_val}: A5 window {'opens' if want_open else 'stays shut'}",
                  (got > 1.0) == want_open, f"A5={fmt(got)}")

        # 9. The audit finds the DEFECTS ALREADY KNOWN TO EXIST in lr_00. This is
        #    the check that stops the whole instrument being circular: it is
        #    calibrated on a world it did not build, whose failures Phase 4
        #    measured independently.
        if (SCEN / "lr_00").exists():
            lr = audit(SCEN / "lr_00")
            s = lr.summary()
            check("lr_00: A1 fires (Phase 4 measured this market as unclearable)",
                  s["A1_labour"] > 1.5, f"A1={fmt(s['A1_labour'])}")
            check("lr_00: A2 fires (mills short of demand, services made for nobody)",
                  not (math.isfinite(s["A2_goods"]) and abs(s["A2_goods"] - 1.0) <= EXACT),
                  f"A2={fmt(s['A2_goods'])}")
            check("lr_00: A4 fires (genesis 0.35/0.60 against implied 0.30/0.50)",
                  abs(s["A4_prices"] - 1.0) > 0.1, f"A4={fmt(s['A4_prices'])}")
            # lr's registered s=4 keeps its channels' window open by a factor of
            # two. Its transports still never post, for the different reason
            # recorded on `a5_rule_window` — which is why A5 passing there is a
            # real reading and not the check failing to notice a live defect.
            check("lr_00: A5 window open at the registered s=4",
                  s["A5_window"] > 1.0, f"A5={fmt(s['A5_window'])}")
            lr1 = SCEN.parent.parent  # unused; kept explicit that the probe is a copy
            probe = out / "_a5_lr_s1"
            shutil.rmtree(probe, ignore_errors=True)
            shutil.copytree(SCEN / "lr_00", probe)
            gp = probe / "game_data.ron"
            gp.write_text(gp.read_text(encoding="utf-8").replace("        s: 4,", "        s: 1,"),
                          encoding="utf-8")
            check("lr_00 forced to s=1: A5 fires — the same tape, the one dial",
                  audit(probe).summary()["A5_window"] <= 1.0,
                  f"A5={fmt(audit(probe).summary()['A5_window'])}")

    print()
    if failures:
        print(f"{len(failures)} FAILED: " + ", ".join(failures))
        return 1
    print("all checks passed")
    return 0


# ══════════════════════════════════════════════════════════════════════════════


def main() -> int:
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except (AttributeError, OSError):
        pass
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--audit", nargs="*", help="scenario dirs (or globs) to audit")
    ap.add_argument("--write", action="store_true", help="write the cr_* corpus")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("-v", "--verbose", action="store_true")
    args = ap.parse_args()

    if args.selftest:
        return selftest()

    if args.write:
        for spec in SPECS:
            emit(spec, SCEN)
            a = audit(SCEN / spec.name)
            print(report(a, read_tape(SCEN / spec.name), args.verbose))
        return 0

    if args.audit is not None:
        targets = []
        for pat in (args.audit or [str(SCEN / "*")]):
            targets += [pathlib.Path(p) for p in sorted(glob.glob(pat)) if os.path.isdir(p)]
        for d in targets:
            if not (d / "game_data.ron").exists():
                continue
            try:
                print(report(audit(d), read_tape(d), args.verbose))
            except Exception as exc:  # a tape this tool cannot read is a finding
                print(f"{d.name:16s}  UNREADABLE  {exc}")
        return 0

    ap.print_help()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
