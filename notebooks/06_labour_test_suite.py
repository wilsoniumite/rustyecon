"""
Labour test suite generator.

Generates 24 scenario variants from multi_region by varying four dimensions:
  labour_supply: A (oversupply all), B (undersupply all), C (mixed by region)
  channel_size:  L (large x2),       S (small x0.5)
  wheat_supply:  B (balanced),        U (unbalanced — heavy imbalance)
  start_state:   G (good prices),     I (imbalanced prices)

Writes to data/scenarios/lr_{id}/ and a manifest to data/scenarios/lr_manifest.csv.

PLOTS = False  (no output files; copy to .ipynb to see graphs)
"""

import copy
import csv
import shutil
import sys
from itertools import product
from pathlib import Path

_root = next(p for p in [Path.cwd(), *Path.cwd().parents] if (p / "Cargo.toml").exists())
sys.path.insert(0, str(_root / "tools"))

import ron
from scenario import (
    SCENARIOS_DIR,
    load_scenario,
    save_scenario,
    load_starting_state,
    save_starting_state,
    load_events,
    save_events,
)

PLOTS = False
BASE = "multi_region"

# ── Dimension definitions ─────────────────────────────────────────────────────

# Labour supply: multiply each pop's size by these per-region factors.
# Regions are indexed 0=Manchester, 1=Leeds, 2=Birmingham.
LABOUR_SUPPLY = {
    "A": {0: 3.0,  1: 3.0,  2: 3.0},   # oversupply everywhere
    "B": {0: 0.4,  1: 0.4,  2: 0.4},   # undersupply everywhere
    "C": {0: 3.0,  1: 1.0,  2: 0.4},   # mixed: Manchester over, Leeds balanced, Birmingham under
}

# Channel size: multiply flour_transport recipe_size by this factor.
CHANNEL_SIZE = {
    "L": 2.0,   # oversized channels
    "S": 0.5,   # undersized channels
}

# Wheat supply: multiplier on wheat_farm recipe_size per region (0=Manchester, 1=Leeds, 2=Birmingham).
# "B" keeps baseline sizes (roughly matched to grain mills).
# "U" creates a regional mismatch: surplus capacity in Mcr/Bham, shortage in Leeds.
WHEAT_SUPPLY = {
    "B": {0: 1.0,  1: 1.0,  2: 1.0 },  # balanced (baseline)
    "U": {0: 2.5,  1: 0.2,  2: 2.0 },  # unbalanced
}

# Starting prices: {good_name: price} applied to every node.
START_STATE = {
    "G": {"wheat": 0.35, "flour": 0.6, "GBP": 1.0, "labour": 1.0},  # all buildings profitable from tick 1
    "I": {"wheat": 2.0,  "flour": 0.2, "GBP": 1.0, "labour": 0.5},  # prices inverted: farms/mills unprofitable
}

# ── Helpers ───────────────────────────────────────────────────────────────────

def _region_of(pop: dict) -> int:
    r = pop["region"]
    return r[0] if isinstance(r, tuple) else int(r)

def _region_of_building(b: dict) -> int:
    r = b["region"]
    return r[0] if isinstance(r, tuple) else int(r)

def _is_channel_building(b: dict) -> bool:
    ch = b.get("channel")
    return ch is not None and not (isinstance(ch, ron.Bare) and ch.value == "None")

def _recipe_name(b: dict) -> str:
    return b.get("recipe", "")

# ── Generator ─────────────────────────────────────────────────────────────────

def generate():
    base_gd = load_scenario(BASE)
    base_ss = load_starting_state(BASE)
    base_ev = load_events(BASE)

    manifest_rows = []
    scenario_id = 0

    for ls, cs, ws, st in product(
        sorted(LABOUR_SUPPLY), sorted(CHANNEL_SIZE),
        sorted(WHEAT_SUPPLY),  sorted(START_STATE),
    ):
        name = f"lr_{scenario_id:02d}"
        label = f"labour={ls} channel={cs} wheat={ws} start={st}"
        print(f"  {name}: {label}")

        ss = copy.deepcopy(base_ss)

        # Labour supply: scale pop sizes by per-region factor.
        labour_factors = LABOUR_SUPPLY[ls]
        for pop in ss["pop_groups"]:
            region = _region_of(pop)
            pop["size"] = round(pop["size"] * labour_factors.get(region, 1.0), 4)
            # Scale starting GBP proportionally so each pop starts with the same buffer.
            if "GBP" in pop["inventory"]:
                pop["inventory"]["GBP"] = round(
                    pop["inventory"]["GBP"] * labour_factors.get(region, 1.0), 4)

        # Channel size: scale flour_transport recipe_sizes.
        ch_factor = CHANNEL_SIZE[cs]
        for b in ss["buildings"]:
            if _is_channel_building(b):
                b["recipe_size"] = round(b["recipe_size"] * ch_factor, 4)

        # Wheat supply: scale wheat_farm recipe_sizes per region.
        wheat_factors = WHEAT_SUPPLY[ws]
        for b in ss["buildings"]:
            if _recipe_name(b) == "wheat_farm":
                region = _region_of_building(b)
                b["recipe_size"] = round(b["recipe_size"] * wheat_factors.get(region, 1.0), 4)

        # Starting prices: flat RonMap applied to all nodes.
        ss["prices"] = ron.RonMap(START_STATE[st])

        # Write scenario.
        dest = SCENARIOS_DIR / name
        dest.mkdir(parents=True, exist_ok=True)
        save_scenario(name, copy.deepcopy(base_gd))
        save_starting_state(name, ss)
        save_events(name, copy.deepcopy(base_ev))
        # Copy dashboard if present (gives a starting visualisation layout).
        src_dash = SCENARIOS_DIR / BASE / "dashboard.json"
        if src_dash.exists():
            shutil.copy(src_dash, dest / "dashboard.json")

        manifest_rows.append({
            "id": scenario_id,
            "name": name,
            "labour_supply": ls,
            "channel_size": cs,
            "wheat_supply": ws,
            "start_state": st,
            "label": label,
        })
        scenario_id += 1

    # Write manifest.
    manifest_path = SCENARIOS_DIR / "lr_manifest.csv"
    with open(manifest_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=manifest_rows[0].keys())
        writer.writeheader()
        writer.writerows(manifest_rows)

    print(f"\n{scenario_id} scenarios written to {SCENARIOS_DIR}")
    print(f"Manifest: {manifest_path}")

if __name__ == "__main__":
    print(f"Generating from base scenario: {BASE}")
    generate()
