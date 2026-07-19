"""
Regression checks for the Parquet analysis path (v2 Phase 2).

Run against a directory produced by `rustyecon <scenario> --record --certify`:

    python tools/test_readers.py tmp/stability_suite/lr_00

The shape assertions matter less than the last one. The Phase 2 gate is "no RON
parsing in the analysis path", and the way that gate quietly rots is someone
reaching for a checkpoint parser again because it is familiar. So the parser is
booby-trapped and the load re-run: if anything in readers.py ever calls it, this
fails loudly rather than merely getting slower.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

DEFAULT_DIR = "tmp/stability_suite/lr_00"

EXPECTED_COLUMNS = {
    "prices": ["tick", "node_id", "good_id", "price", "supply", "demand", "imbalance"],
    "buildings": [
        "tick", "building_id", "region_id", "recipe_id", "recipe_size", "chosen_size",
        "efficiency", "balance", "last_margin", "last_throughput", "channel_id",
    ],
    "pops": ["tick", "pop_id", "region_id", "size", "wealth", "savings_target", "is_employed"],
    "inventories": ["tick", "entity_type", "entity_id", "good_id", "qty"],
}


def check(out_dir: str) -> int:
    from readers import has_results, load_region_metrics, load_scenario_results

    if not has_results(out_dir):
        print(f"FAIL: no telemetry in {out_dir}. Run the scenario with --record first.")
        return 1

    failures: list[str] = []

    # The gate, checked first so that a violation is reported as a violation
    # rather than as whatever incidental error it happens to raise.
    import ron

    _real_parse, _real_load = ron.parse, ron.load

    def _boom(*_a, **_k):
        raise AssertionError("the analysis path parsed RON")

    ron.parse, ron.load = _boom, _boom
    try:
        load_scenario_results(out_dir)
    except AssertionError as e:
        failures.append(str(e))
    finally:
        ron.parse, ron.load = _real_parse, _real_load

    res = load_scenario_results(out_dir)

    for name, cols in EXPECTED_COLUMNS.items():
        df = getattr(res, name)
        if list(df.columns) != cols:
            failures.append(f"{name}: columns {list(df.columns)} != {cols}")
        if df.empty:
            failures.append(f"{name}: empty")
    if res.pops["is_employed"].dtype != bool:
        failures.append(f"pops.is_employed dtype is {res.pops['is_employed'].dtype}, want bool")

    # The dimension tables have to come from the manifest, not from a scenario.
    if not res.good_name:
        failures.append("good_name is empty; the manifest carries no goods")
    if not res.regional_node_ids():
        failures.append("no regional nodes resolved from the manifest")

    # The derived API is what tools/app.py and 05_supply_chain_sim call.
    for label, series in [
        ("price_index", res.price_index()),
        ("yoy_inflation", res.yoy_inflation()),
        ("pop_wealth_by_region", res.pop_wealth_by_region()),
    ]:
        if series.empty:
            failures.append(f"{label}() returned an empty series")

    rm = load_region_metrics(out_dir)
    if rm.empty:
        print("  note: no region series (run was not certified) - skipping that check")
    elif "velocity" not in rm.columns:
        failures.append("region metrics present but missing velocity")

    for f in failures:
        print(f"FAIL: {f}")
    if not failures:
        print(f"ok: {out_dir}")
        print(f"  prices={res.prices.shape} buildings={res.buildings.shape} "
              f"pops={res.pops.shape} inventories={res.inventories.shape}")
        print("  no RON parsed")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(check(sys.argv[1] if len(sys.argv) > 1 else DEFAULT_DIR))
