"""
ScenarioResults: parse RON checkpoints into DataFrames.

Usage (standalone / notebook):
    import sys; sys.path.insert(0, "tools")
    from readers import load_scenario_results
    from scenario import load_scenario, SCENARIOS_DIR

    gd  = load_scenario("supply_chain")
    res = load_scenario_results("tmp/supply_chain", gd)

    res.prices.head()
    res.price_index().plot()
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

import ron
from ron import Tagged, Bare

if TYPE_CHECKING:
    pass


# ── ID helpers ────────────────────────────────────────────────────────────────

def _int_id(v) -> int:
    """Extract integer from a Tagged, 1-tuple, or plain int, e.g. BuildingId(0), (0,), or 0."""
    if isinstance(v, Tagged):
        return _int_id(v.inner)
    if isinstance(v, tuple) and len(v) == 1:
        return int(v[0])
    if v is None:
        return -1
    return int(v)


def _bare_str(v) -> str:
    return v.value if isinstance(v, Bare) else str(v)


# ── game_data helpers ─────────────────────────────────────────────────────────

def _index_game_data(gd: dict) -> tuple[dict, dict, dict, dict, int | None, dict]:
    """
    Returns (good_name, recipe_name, node_to_region, region_name, currency_good_id, channel_to_regions).
    All keyed by integer id.
    """
    goods    = gd.get("goods", [])
    recipes  = gd.get("recipes", [])
    nodes    = gd.get("market_nodes", [])
    regions  = gd.get("regions", [])
    channels = gd.get("channels", [])

    good_name   = {i: g.get("name", f"good{i}") for i, g in enumerate(goods)}
    recipe_name = {i: r.get("name", f"recipe{i}") for i, r in enumerate(recipes)}
    region_name = {_int_id(r.get("id", i)): r.get("name", f"region{i}")
                   for i, r in enumerate(regions)}

    node_to_region: dict[int, int | None] = {}
    for i, n in enumerate(nodes):
        nid = _int_id(n.get("id", i))
        reg = n.get("region")
        if isinstance(reg, Tagged) and reg.tag == "Some":
            node_to_region[nid] = _int_id(reg.inner)
        else:
            node_to_region[nid] = None

    name_to_good_id = {v: k for k, v in good_name.items()}
    cg = gd.get("currency_good")
    currency_good_id: int | None = None
    if isinstance(cg, Tagged) and cg.tag == "Some":
        inner = cg.inner
        if isinstance(inner, str):
            currency_good_id = name_to_good_id.get(inner)
        else:
            currency_good_id = _int_id(inner)
    elif isinstance(cg, str):
        currency_good_id = name_to_good_id.get(cg)
    elif isinstance(cg, int):
        currency_good_id = cg

    channel_to_regions = {}
    for i, c in enumerate(channels):
        cid = _int_id(c.get("id", i))
        from_m_node = _int_id(c.get("from", -1))
        to_m_node = _int_id(c.get("to", -1))
        r1 = node_to_region.get(from_m_node)
        r2 = node_to_region.get(to_m_node)
        channel_to_regions[cid] = (r1, r2)

    print(channel_to_regions)
    return good_name, recipe_name, node_to_region, region_name, currency_good_id, channel_to_regions


# ── single checkpoint parser ──────────────────────────────────────────────────

def _parse_checkpoint(text: str) -> dict:
    return ron.parse(text)


def _extract_prices(state: dict) -> list[dict]:
    tick       = int(state["tick"])
    num_goods  = int(state["num_goods"])
    prices     = state.get("prices", [])
    supply     = state.get("supply", [])
    demand     = state.get("demand", [])
    rows = []
    for i, p in enumerate(prices):
        node_id = i // num_goods
        good_id = i % num_goods
        s = float(supply[i]) if i < len(supply) else 0.0
        d = float(demand[i]) if i < len(demand) else 0.0
        if s <= 0.0 and d <= 0.0:
            imb = 0.0
        else:
            imb = (d - s) / max(s, d)
        rows.append({
            "tick": tick, "node_id": node_id, "good_id": good_id,
            "price": float(p), "supply": s, "demand": d, "imbalance": imb,
        })
    return rows


def _extract_buildings(state: dict) -> tuple[list[dict], list[dict]]:
    tick = int(state["tick"])
    bld_rows: list[dict] = []
    inv_rows: list[dict] = []
    for b in state.get("buildings", []):
        bid = _int_id(b.get("id", 0))
        bld_rows.append({
            "tick":             tick,
            "building_id":      bid,
            "region_id":        _int_id(b.get("region", 0)),
            "recipe_id":        _int_id(b.get("recipe", 0)),
            "recipe_size":      float(b.get("recipe_size", 0.0)),
            "chosen_size":      float(b.get("chosen_size", 0.0)),
            "efficiency":       float(b.get("efficiency", 1.0)),
            "balance":          float(b.get("balance", 0.0)),
            "last_margin":      float(b.get("last_margin", 0.0)),
            "last_throughput":  float(b.get("last_throughput", 0.0)),
            "channel_id":      _int_id(b.get("channel", -1)),
        })
        for entry in (b.get("inventory") or []):
            if isinstance(entry, (tuple, list)) and len(entry) == 2:
                gid, qty = _int_id(entry[0]), float(entry[1])
                inv_rows.append({
                    "tick": tick, "entity_type": "building",
                    "entity_id": bid, "good_id": gid, "qty": qty,
                })
    return bld_rows, inv_rows


def _extract_pops(state: dict) -> tuple[list[dict], list[dict]]:
    tick = int(state["tick"])
    pop_rows: list[dict] = []
    inv_rows: list[dict] = []
    for p in state.get("pop_groups", []):
        pid = _int_id(p.get("id", 0))
        pop_rows.append({
            "tick":           tick,
            "pop_id":         pid,
            "region_id":      _int_id(p.get("region", 0)),
            "size":           float(p.get("size", 0.0)),
            "wealth":         float(p.get("wealth", 0.0)),
            "savings_target": float(p.get("savings_target", 0.0)),
        })
        for entry in (p.get("inventory") or []):
            if isinstance(entry, (tuple, list)) and len(entry) == 2:
                gid, qty = _int_id(entry[0]), float(entry[1])
                inv_rows.append({
                    "tick": tick, "entity_type": "pop",
                    "entity_id": pid, "good_id": gid, "qty": qty,
                })
    return pop_rows, inv_rows


# ── ScenarioResults ───────────────────────────────────────────────────────────

@dataclass
class ScenarioResults:
    # Lookup tables (from game_data)
    good_name:       dict[int, str]       # good_id  -> name
    recipe_name:     dict[int, str]       # recipe_id -> name
    region_name:     dict[int, str]       # region_id -> name
    node_to_region:  dict[int, int|None]  # node_id  -> region_id or None
    channel_to_regions: dict[int, tuple[int, int]]  # channel_id -> (region_id1, region_id2)
    currency_good_id: int | None

    # Time series (empty DataFrames with correct columns if no data)
    prices:      pd.DataFrame   # tick, node_id, good_id, price, supply, demand, imbalance
    buildings:   pd.DataFrame   # tick, building_id, region_id, recipe_id, recipe_size, chosen_size, efficiency, balance, last_margin, last_throughput, channel_id
    pops:        pd.DataFrame   # tick, pop_id, region_id, size, wealth, savings_target
    inventories: pd.DataFrame   # tick, entity_type, entity_id, good_id, qty

    def good_names_non_currency(self) -> list[int]:
        return [i for i in self.good_name if i != self.currency_good_id]

    def regional_node_ids(self) -> list[int]:
        """Node IDs that correspond to a region (excludes national/world nodes)."""
        return [n for n, r in self.node_to_region.items() if r is not None]

    def _filter_nodes(self, df: pd.DataFrame, region_id: int | None, node_id: int | None) -> pd.DataFrame:
        """Filter a prices-shaped DataFrame to the right node(s)."""
        if node_id is not None:
            return df[df["node_id"] == node_id]
        if region_id is not None:
            matching = [n for n, r in self.node_to_region.items() if r == region_id]
            return df[df["node_id"].isin(matching)]
        # No filter specified — use all regional nodes
        return df[df["node_id"].isin(self.regional_node_ids())]

    # ── derived series ────────────────────────────────────────────────────────

    def price_index(self, good_ids: list[int] | None = None) -> pd.Series:
        """
        Equal-weighted geometric mean of (price / base_price) for selected goods,
        averaged across all regional nodes.
        base_price = mean price at the earliest tick in the data.
        Returns a Series indexed by tick, named 'price_index'.
        """
        if self.prices.empty:
            return pd.Series(dtype=float, name="price_index")

        gids = good_ids if good_ids is not None else self.good_names_non_currency()
        if not gids:
            return pd.Series(dtype=float, name="price_index")

        sub = self._filter_nodes(self.prices, None, None)
        sub = sub[sub["good_id"].isin(gids)].copy()

        # Mean price across all regional nodes, per (tick, good)
        sub = sub.groupby(["tick", "good_id"])["price"].mean().reset_index()

        base_tick = sub["tick"].min()
        base_prices = (
            sub[sub["tick"] == base_tick]
            .set_index("good_id")["price"]
            .to_dict()
        )

        ticks = sorted(sub["tick"].unique())
        idx_vals = []
        for t in ticks:
            row = sub[sub["tick"] == t].set_index("good_id")["price"]
            log_sum = sum(
                math.log(max(row.get(g, 1e-12) / max(base_prices.get(g, 1.0), 1e-12), 1e-12))
                for g in gids
            )
            idx_vals.append(math.exp(log_sum / len(gids)))

        return pd.Series(idx_vals, index=ticks, name="price_index")

    def yoy_inflation(self, ticks_per_year: int = 52) -> pd.Series:
        """YoY % change in price_index. Returns Series indexed by tick."""
        pi = self.price_index()
        if pi.empty:
            return pd.Series(dtype=float, name="yoy_inflation_pct")
        shifted = pi.shift(ticks_per_year)
        result = ((pi / shifted) - 1.0) * 100.0
        result.name = "yoy_inflation_pct"
        return result
    
    def get_good_id(self, name_or_id: str | int) -> int | None:
        if isinstance(name_or_id, int):
            return name_or_id
        inv = {v: k for k, v in self.good_name.items()}
        try:
            return inv[name_or_id]
        except KeyError:
            raise ValueError(f"Unknown good name: {name_or_id}")

    def prices_for(
        self,
        good_id_or_name: int | str,
        region_id: int | None = None,
        node_id: int | None = None,
    ) -> pd.Series:
        """
        Price series for a good. If region_id and node_id are both None,
        returns the mean across all regional nodes.
        """
        good_id = self.get_good_id(good_id_or_name)
        df = self._filter_nodes(self.prices, region_id, node_id)
        df = df[df["good_id"] == good_id]
        return df.groupby("tick")["price"].mean()

    def volume_for(
        self,
        good_id_or_name: int | str,
        kind: str = "supply",   # "supply", "demand", or "imbalance"
        region_id: int | None = None,
        node_id: int | None = None,
    ) -> pd.Series:
        """
        Supply, demand, or imbalance series for a good. Mean across regional
        nodes when region_id and node_id are both None.
        """
        good_id = self.get_good_id(good_id_or_name)
        df = self._filter_nodes(self.prices, region_id, node_id)
        df = df[df["good_id"] == good_id]
        return df.groupby("tick")[kind].mean()

    def building_series(self, building_id: int, col: str) -> pd.Series:
        df = self.buildings[self.buildings["building_id"] == building_id]
        return df.set_index("tick")[col]

    def pop_wealth_by_region(self, region_id: int | None = None) -> pd.Series:
        """Mean pop wealth per tick, optionally filtered to one region."""
        df = self.pops.copy()
        if region_id is not None:
            df = df[df["region_id"] == region_id]
        return df.groupby("tick")["wealth"].mean()

    def inventory_for(
        self,
        entity_type: str,
        entity_id: int,
        good_id_or_name: int | str,
    ) -> pd.Series:
        good_id = self.get_good_id(good_id_or_name)
        df = self.inventories[
            (self.inventories["entity_type"] == entity_type) &
            (self.inventories["entity_id"] == entity_id) &
            (self.inventories["good_id"] == good_id)
        ]
        return df.set_index("tick")["qty"]


# ── loader ────────────────────────────────────────────────────────────────────

def load_scenario_results(
    output_dir: str | Path,
    game_data: dict | str | Path,
) -> ScenarioResults:
    """
    Parse all tick_*.ron files in output_dir into a ScenarioResults.

    game_data: either a parsed game_data dict (from ron.load / load_scenario)
               or a path to game_data.ron.
    """
    output_dir = Path(output_dir)

    if not isinstance(game_data, dict):
        game_data = ron.load(str(game_data))

    good_name, recipe_name, node_to_region, region_name, currency_good_id, channel_to_regions = (
        _index_game_data(game_data)
    )

    ron_files = sorted(output_dir.glob("tick_*.ron"))
    if not ron_files:
        # Return empty results
        empty_price = pd.DataFrame(columns=["tick","node_id","good_id","price","supply","demand","imbalance"])
        empty_bld   = pd.DataFrame(columns=["tick","building_id","region_id","recipe_id","recipe_size","chosen_size","efficiency","balance","last_margin","last_throughput"])
        empty_pop   = pd.DataFrame(columns=["tick","pop_id","region_id","size","wealth","savings_target"])
        empty_inv   = pd.DataFrame(columns=["tick","entity_type","entity_id","good_id","qty"])
        return ScenarioResults(
            good_name=good_name, recipe_name=recipe_name,
            region_name=region_name, node_to_region=node_to_region,
            currency_good_id=currency_good_id,
            channel_to_regions=channel_to_regions,
            prices=empty_price, buildings=empty_bld, pops=empty_pop, inventories=empty_inv,
        )

    price_rows, bld_rows, pop_rows, inv_rows = [], [], [], []

    for path in ron_files:
        text = path.read_text(encoding="utf-8")
        state = _parse_checkpoint(text)

        price_rows.extend(_extract_prices(state))

        b_rows, b_inv = _extract_buildings(state)
        bld_rows.extend(b_rows)
        inv_rows.extend(b_inv)

        p_rows, p_inv = _extract_pops(state)
        pop_rows.extend(p_rows)
        inv_rows.extend(p_inv)

    prices_df      = pd.DataFrame(price_rows)
    buildings_df   = pd.DataFrame(bld_rows)
    pops_df        = pd.DataFrame(pop_rows)
    inventories_df = pd.DataFrame(inv_rows)

    return ScenarioResults(
        good_name=good_name,
        recipe_name=recipe_name,
        region_name=region_name,
        node_to_region=node_to_region,
        currency_good_id=currency_good_id,
        channel_to_regions=channel_to_regions,
        prices=prices_df,
        buildings=buildings_df,
        pops=pops_df,
        inventories=inventories_df,
    )
