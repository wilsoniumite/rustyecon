"""
ScenarioResults: load a run's Parquet telemetry into DataFrames.

v2 Phase 2 rewrote the *input* plumbing only. Every attribute and method of
ScenarioResults is unchanged, because the notebooks and tools/app.py are built
on it; what changed is where the data comes from. Previously this module parsed
one RON checkpoint per tick with a hand-written recursive-descent parser whose
tokeniser re-slices the remaining file on every token, making load quadratic in
file size — a 25-scenario suite meant 25,000 of those parses. The engine now
writes a single Parquet file plus a manifest, and this module pivots the tidy
long table into the same four frames.

The manifest carries the dimension tables, so no game_data.ron is read here
either. That is the point of the phase gate: nothing in the analysis path
parses RON.

Usage (standalone / notebook):
    import sys; sys.path.insert(0, "tools")
    from readers import load_scenario_results

    res = load_scenario_results("tmp/supply_chain")

    res.prices.head()
    res.price_index().plot()
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import pandas as pd

TELEMETRY_FILE = "telemetry.parquet"
MANIFEST_FILE = "manifest.json"

# Manifest layouts this reader understands. A run written by a newer engine is
# refused rather than half-read: silently ignoring an unknown schema is how you
# get an analysis of columns that no longer mean what they used to.
SUPPORTED_SCHEMA_VERSIONS = (1,)

# Columns of each frame, in order. Declared once so the empty and populated
# cases cannot drift apart — they did before, and the empty frames were missing
# `channel_id` and `is_employed`, so a run with no data raised KeyError in the
# stability suite instead of yielding empty results.
PRICE_COLS = ["tick", "node_id", "good_id", "price", "supply", "demand", "imbalance"]
BUILDING_COLS = [
    "tick", "building_id", "region_id", "recipe_id", "recipe_size", "chosen_size",
    "efficiency", "balance", "last_margin", "last_throughput", "channel_id",
]
POP_COLS = ["tick", "pop_id", "region_id", "size", "wealth", "savings_target", "is_employed"]
INV_COLS = ["tick", "entity_type", "entity_id", "good_id", "qty"]

# Integer-valued columns. The long table stores every value as a float (one
# `value` column serves all metrics), so these are cast back on the way out —
# downstream code indexes and groups by them and expects ints.
_INT_COLS = {
    "tick", "node_id", "good_id", "building_id", "region_id", "recipe_id",
    "channel_id", "pop_id", "entity_id",
}


def has_results(output_dir: str | Path) -> bool:
    """True if output_dir holds a complete telemetry pair."""
    d = Path(output_dir)
    return (d / TELEMETRY_FILE).is_file() and (d / MANIFEST_FILE).is_file()


def sampling_every(output_dir: str | Path) -> int:
    """Ticks between telemetry samples for this run (1 = every tick).

    Anything that indexes by row position rather than tick value - rolling
    windows, positional shifts - is only correct when this is 1, and should
    say so rather than quietly rescaling its window.
    """
    return int(_read_manifest(Path(output_dir))["telemetry"].get("every", 1))


def _read_manifest(output_dir: Path) -> dict:
    path = output_dir / MANIFEST_FILE
    if not path.is_file():
        raise FileNotFoundError(
            f"no {MANIFEST_FILE} in {output_dir}. Run the engine with --record; "
            "a run without telemetry cannot be analysed."
        )
    m = json.loads(path.read_text(encoding="utf-8"))
    version = m.get("schema_version")
    if version not in SUPPORTED_SCHEMA_VERSIONS:
        raise ValueError(
            f"{path} declares schema_version {version!r}; this reader supports "
            f"{SUPPORTED_SCHEMA_VERSIONS}. Re-run the scenario or update readers.py."
        )
    return m


def _dimensions(m: dict) -> tuple[dict, dict, dict, dict, dict, dict]:
    """Build the six lookup tables from the manifest's dimension tables.

    Returns (good_name, recipe_name, node_to_region, region_name,
    node_currency, channel_to_regions) — the same six the RON reader derived
    from game_data.ron, keyed the same way.
    """
    good_name = {int(g["id"]): g["name"] for g in m.get("goods", [])}
    recipe_name = {int(r["id"]): r["name"] for r in m.get("recipes", [])}
    region_name = {int(r["id"]): r["name"] for r in m.get("regions", [])}

    node_to_region: dict[int, int | None] = {}
    node_currency: dict[int, int | None] = {}
    for n in m.get("market_nodes", []):
        nid = int(n["id"])
        # None for national and world nodes, which serve no single region.
        node_to_region[nid] = None if n.get("region") is None else int(n["region"])
        cg = n.get("currency_good")
        node_currency[nid] = None if cg is None else int(cg)

    channel_to_regions: dict[int, tuple[int | None, int | None]] = {}
    for c in m.get("channels", []):
        cid = int(c["id"])
        channel_to_regions[cid] = (
            node_to_region.get(int(c["from_node"])),
            node_to_region.get(int(c["to_node"])),
        )

    return good_name, recipe_name, node_to_region, region_name, node_currency, channel_to_regions


def _empty(cols: list[str]) -> pd.DataFrame:
    """An empty frame with the right columns *and* the right dtypes.

    Dtypes matter here: an object-dtype empty column silently changes the result
    of a downstream groupby or comparison, so an empty run would not behave like
    a small one.
    """
    df = pd.DataFrame({c: pd.Series(dtype=_dtype_for(c)) for c in cols})
    return df[cols]


def _dtype_for(col: str) -> str:
    if col in _INT_COLS:
        return "int64"
    if col == "is_employed":
        return "bool"
    if col == "entity_type":
        return "object"
    return "float64"


def _pivot(long: pd.DataFrame, keys: list[str], rename: dict[str, str],
           cols: list[str]) -> pd.DataFrame:
    """Pivot a slice of the tidy long table into one wide frame.

    `keys` are the long-table columns that identify a row of the result;
    `metric` becomes the columns and `value` the cells.
    """
    if long.empty:
        return _empty(cols)
    wide = long.pivot(index=keys, columns="metric", values="value").reset_index()
    wide.columns.name = None
    wide = wide.rename(columns=rename)
    for c in cols:
        if c not in wide.columns:
            # A metric the engine did not emit for this run. Left as NaN rather
            # than filled: absent is not zero.
            wide[c] = np.nan
    wide = wide[cols]
    for c in cols:
        dt = _dtype_for(c)
        if dt == "float64":
            continue
        if wide[c].isna().any():
            # A metric the engine emitted for some rows of this frame but not
            # others. Casting would raise and filling would invent data, so the
            # column stays float and the gap stays visible as NaN - which is
            # what the comment above promises.
            continue
        wide[c] = wide[c].astype(dt)
    return wide


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
    pops:        pd.DataFrame   # tick, pop_id, region_id, size, wealth, savings_target, is_employed
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
        """YoY % change in price_index. Returns Series indexed by tick.

        Aligned by *tick value*, not by row position. Telemetry can be
        subsampled (--telemetry-every), and a positional shift would then
        compare against whatever row happens to sit N places back: on a run
        sampled every 5 ticks, 52 rows is 260 ticks, which silently returns a
        number of the wrong sign and magnitude rather than an error.

        Where the tick a year earlier was not sampled the result is NaN. Under
        the fail-closed policy that is the honest answer - the comparison was
        not computable - and it is what the certificate treats as a failure
        rather than a pass. No nearest-neighbour fallback: silently comparing
        against a tick 3 short of a year is a different measurement wearing
        this one's name.

        That does mean a run sampled every 5 ticks returns all-NaN at the
        default lag of 52, because no multiple of 5 is 52 apart from another.
        The fix is a lag the sampling divides: on an every=5 run,
        `yoy_inflation(ticks_per_year=50)` agrees with the fully-sampled run
        exactly, to the bit.
        """
        pi = self.price_index()
        if pi.empty:
            return pd.Series(dtype=float, name="yoy_inflation_pct")
        prior = pi.reindex(pi.index - ticks_per_year)
        prior.index = pi.index
        result = ((pi / prior) - 1.0) * 100.0
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

def _require_unique(long: pd.DataFrame, keys: list[str], what: str) -> None:
    """Fail loudly if a pivot key repeats within a tick.

    `pivot` would raise anyway, and `pivot_table` would quietly average the
    collision away. Neither is a useful answer, so say what collided: a repeated
    key means two entities are being reported under one identity, and averaging
    them would produce a series for an entity that does not exist.
    """
    dupes = long.duplicated(subset=keys + ["metric"])
    if bool(dupes.any()):
        sample = long[dupes][keys + ["metric"]].head(3).to_dict("records")
        raise ValueError(
            f"{what}: {int(dupes.sum())} duplicate rows for keys {keys}, e.g. {sample}. "
            "Two entities share an identity in the telemetry; they cannot be pivoted "
            "into one series without inventing a number."
        )


def load_scenario_results(
    output_dir: str | Path,
    game_data: dict | str | Path | None = None,
) -> ScenarioResults:
    """
    Load output_dir's Parquet telemetry into a ScenarioResults.

    game_data is accepted and ignored. The manifest carries the dimension tables
    this loader used to derive from game_data.ron; the parameter is kept so
    existing callers (notebooks, tools/app.py) keep working unchanged.

    Raises FileNotFoundError if the directory holds no telemetry. That is
    deliberate: a run that cannot be analysed must not be returned as a run that
    showed nothing.
    """
    output_dir = Path(output_dir)
    manifest = _read_manifest(output_dir)
    (good_name, recipe_name, node_to_region, region_name,
     node_currency, channel_to_regions) = _dimensions(manifest)

    tpath = output_dir / manifest["telemetry"].get("file", TELEMETRY_FILE)
    if not tpath.is_file():
        raise FileNotFoundError(
            f"{MANIFEST_FILE} in {output_dir} points at {tpath.name}, which is missing."
        )

    df = pd.read_parquet(tpath, columns=["tick", "region", "entity_kind", "id", "good", "metric", "value"])

    # ── markets: one row per (tick, node, good) ──
    mk = df[df["entity_kind"] == "market"]
    if mk.empty:
        prices = _empty(PRICE_COLS)
    else:
        mk = mk[["tick", "id", "good", "metric", "value"]].copy()
        mk["good"] = mk["good"].astype("int64")
        _require_unique(mk, ["tick", "id", "good"], "market telemetry")
        raw = [c for c in PRICE_COLS if c != "imbalance"]
        prices = _pivot(mk, ["tick", "id", "good"],
                        {"id": "node_id", "good": "good_id"}, raw)
        # Derived, not stored: the engine emits supply and demand, and keeping a
        # third copy of the same information invites the copies to disagree.
        # Same definition as the RON reader used: 0.0 when nothing was posted.
        den = np.maximum(prices["supply"], prices["demand"])
        prices["imbalance"] = np.where(
            den > 0, (prices["demand"] - prices["supply"]) / den, 0.0
        )
        prices = prices[PRICE_COLS]

    # ── production: one row per (tick, building) ──
    bd = df[(df["entity_kind"] == "building") & (df["metric"] != "inventory")]
    if bd.empty:
        buildings = _empty(BUILDING_COLS)
    else:
        bd = bd[["tick", "id", "region", "metric", "value"]].copy()
        bd["region"] = bd["region"].astype("int64")
        _require_unique(bd, ["tick", "id", "region"], "building telemetry")
        buildings = _pivot(bd, ["tick", "id", "region"],
                           {"id": "building_id", "region": "region_id"}, BUILDING_COLS)

    # ── pops: one row per (tick, pop) ──
    pp = df[(df["entity_kind"] == "pop") & (df["metric"] != "inventory")]
    if pp.empty:
        pops = _empty(POP_COLS)
    else:
        pp = pp[["tick", "id", "region", "metric", "value"]].copy()
        pp["region"] = pp["region"].astype("int64")
        _require_unique(pp, ["tick", "id", "region"], "pop telemetry")
        pops = _pivot(pp, ["tick", "id", "region"],
                      {"id": "pop_id", "region": "region_id"}, POP_COLS)

    # ── inventories: already long; entity_kind is the old entity_type ──
    iv = df[df["metric"] == "inventory"]
    if iv.empty:
        inventories = _empty(INV_COLS)
    else:
        inventories = pd.DataFrame({
            "tick": iv["tick"].astype("int64").to_numpy(),
            "entity_type": iv["entity_kind"].astype("object").to_numpy(),
            "entity_id": iv["id"].astype("int64").to_numpy(),
            "good_id": iv["good"].astype("int64").to_numpy(),
            "qty": iv["value"].astype("float64").to_numpy(),
        })[INV_COLS]
        # Constructing from a numpy object array lets pandas re-infer a string
        # dtype, which would not match the empty path. Pin it.
        inventories["entity_type"] = inventories["entity_type"].astype(
            _dtype_for("entity_type"))

    return ScenarioResults(
        good_name=good_name,
        recipe_name=recipe_name,
        region_name=region_name,
        node_to_region=node_to_region,
        node_currency=node_currency,
        channel_to_regions=channel_to_regions,
        prices=prices,
        buildings=buildings,
        pops=pops,
        inventories=inventories,
    )


def load_region_metrics(output_dir: str | Path) -> pd.DataFrame:
    """The per-region series the engine scored itself against, as a wide frame.

    Not part of the ScenarioResults API: these are computed in-engine (Phase 1)
    and were never reconstructable from checkpoints, so nothing downstream
    expects them yet. Exposed because analysis that wants to agree with the
    certificate should read the engine's own numbers rather than recompute them.

    Columns: tick, region_id, then one per metric. Empty if the run was not
    certified — the collector only runs under --certify.
    """
    output_dir = Path(output_dir)
    manifest = _read_manifest(output_dir)
    df = pd.read_parquet(output_dir / manifest["telemetry"].get("file", TELEMETRY_FILE))
    rg = df[df["entity_kind"] == "region"]
    if rg.empty:
        return pd.DataFrame({"tick": pd.Series(dtype="int64"),
                             "region_id": pd.Series(dtype="int64")})
    rg = rg[["tick", "id", "metric", "value"]].copy()
    wide = rg.pivot(index=["tick", "id"], columns="metric", values="value").reset_index()
    wide.columns.name = None
    return wide.rename(columns={"id": "region_id"})
