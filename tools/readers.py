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

def _index_game_data(gd: dict) -> tuple[dict, dict, dict, dict, dict, dict]:
    """
    Returns (good_name, recipe_name, node_to_region, region_name, node_currency, channel_to_regions).
    All keyed by integer id.
    node_currency: node_id -> good_id of that node's currency, or None.
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

    # Currency is per market node: build a node_id -> good_id map.
    node_currency: dict[int, int | None] = {}
    for i, n in enumerate(nodes):
        nid = _int_id(n.get("id", i))
        cg = n.get("currency_good")
        if isinstance(cg, Tagged) and cg.tag == "Some":
            node_currency[nid] = _int_id(cg.inner)
        elif isinstance(cg, int):
            node_currency[nid] = cg
        else:
            node_currency[nid] = None

    channel_to_regions = {}
    for i, c in enumerate(channels):
        cid = _int_id(c.get("id", i))
        from_m_node = _int_id(c.get("from", -1))
        to_m_node = _int_id(c.get("to", -1))
        r1 = node_to_region.get(from_m_node)
        r2 = node_to_region.get(to_m_node)
        channel_to_regions[cid] = (r1, r2)

    print(channel_to_regions)
    return good_name, recipe_name, node_to_region, region_name, node_currency, channel_to_regions


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

    recipe_instances = state.get("recipe_instances")
    if recipe_instances:
        # NEW FORMAT: buildings are slim; recipe_instances carry production data.
        # Top-level inventories list is indexed by InventoryId.
        inv_list = state.get("inventories", [])

        # Build inv_id -> (building_id, region_id) lookup
        inv_id_to_bld: dict[int, tuple[int, int]] = {}
        for b in state.get("buildings", []):
            bid = _int_id(b.get("id", 0))
            rid = _int_id(b.get("region", 0))
            inv_id = _int_id(b.get("inventory", 0))
            inv_id_to_bld[inv_id] = (bid, rid)

        # Production rows — only CapacityControl instances
        for ri in recipe_instances:
            ss = ri.get("strategy_state")
            if not (isinstance(ss, Tagged) and ss.tag == "CapacityControl"):
                continue
            input_inv_id = _int_id(ri.get("input_inv", 0))
            bld_info = inv_id_to_bld.get(input_inv_id)
            if bld_info is None:
                continue
            bld_id, region_id = bld_info
            cc = ss.inner if isinstance(ss.inner, dict) else {}
            bld_rows.append({
                "tick":             tick,
                "building_id":      bld_id,
                "region_id":        region_id,
                "recipe_id":        _int_id(ri.get("recipe", 0)),
                "recipe_size":      float(ri.get("recipe_size", 0.0)),
                "chosen_size":      float(ri.get("chosen_size", 0.0)),
                "efficiency":       float(cc.get("efficiency", 1.0)),
                "balance":          float(cc.get("balance", 0.0)),
                "last_margin":      float(cc.get("last_margin", 0.0)),
                "last_throughput":  float(cc.get("last_throughput", 0.0)),
                "channel_id":       _int_id(ri.get("channel")),
            })

        # Inventory rows — one per building, looked up by InventoryId
        for b in state.get("buildings", []):
            bid = _int_id(b.get("id", 0))
            inv_id = _int_id(b.get("inventory", 0))
            for entry in (inv_list[inv_id] if inv_id < len(inv_list) else []):
                if isinstance(entry, (tuple, list)) and len(entry) == 2:
                    gid, qty = _int_id(entry[0]), float(entry[1])
                    inv_rows.append({
                        "tick": tick, "entity_type": "building",
                        "entity_id": bid, "good_id": gid, "qty": qty,
                    })
    else:
        # OLD FORMAT: buildings carry inline recipe data and inventory
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
                "channel_id":       _int_id(b.get("channel", -1)),
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
    inv_list = state.get("inventories", [])  # present in new format
    for p in state.get("pop_groups", []):
        pid = _int_id(p.get("id", 0))
        pop_rows.append({
            "tick":           tick,
            "pop_id":         pid,
            "region_id":      _int_id(p.get("region", 0)),
            "size":           float(p.get("size", 0.0)),
            "wealth":         float(p.get("wealth", 0.0)),
            "savings_target": float(p.get("savings_target", 0.0)),
            "is_employed":    bool(p.get("is_employed", True)),
        })
        inv_field = p.get("inventory")
        if isinstance(inv_field, tuple):
            # NEW FORMAT: InventoryId reference
            inv_id = _int_id(inv_field)
            entries = inv_list[inv_id] if inv_id < len(inv_list) else []
        else:
            # OLD FORMAT: inline list of (good_id, qty)
            entries = inv_field or []
        for entry in entries:
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
    node_currency:   dict[int, int | None]  # node_id -> currency good_id or None

    # Time series (empty DataFrames with correct columns if no data)
    prices:      pd.DataFrame   # tick, node_id, good_id, price, supply, demand, imbalance
    buildings:   pd.DataFrame   # tick, building_id, region_id, recipe_id, recipe_size, chosen_size, efficiency, balance, last_margin, last_throughput, channel_id
    pops:        pd.DataFrame   # tick, pop_id, region_id, size, wealth, savings_target
    inventories: pd.DataFrame   # tick, entity_type, entity_id, good_id, qty

    def currency_good_ids(self) -> set[int]:
        """Set of all good_ids used as a currency on any node."""
        return {gid for gid in self.node_currency.values() if gid is not None}

    def currency_for_node(self, node_id: int) -> int | None:
        """Currency good_id for a specific node, or None."""
        return self.node_currency.get(node_id)

    def good_names_non_currency(self) -> list[int]:
        currencies = self.currency_good_ids()
        return [i for i in self.good_name if i not in currencies]

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

    good_name, recipe_name, node_to_region, region_name, node_currency, channel_to_regions = (
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
            node_currency=node_currency,
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
        node_currency=node_currency,
        channel_to_regions=channel_to_regions,
        prices=prices_df,
        buildings=buildings_df,
        pops=pops_df,
        inventories=inventories_df,
    )
