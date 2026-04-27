"""
Smoke tests for scenario load → edit → save → run.
Run with:  python tools/test_scenario.py
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from ron import Tagged
from scenario import (
    attach_region,
    get_pos,
    load_scenario,
    run_simulation,
    save_scenario,
    set_pos,
)


def test_load():
    gd = load_scenario("two_region_trade")
    regions = gd["regions"]
    assert len(regions) == 2, f"expected 2 regions, got {len(regions)}"
    assert regions[0]["name"] == "Manchester"
    assert len(gd["channels"]) == 1
    print("  load OK")
    return gd


def test_edit_save_roundtrip(gd):
    regions = gd["regions"]
    set_pos(regions[0], 0.0, 0.0)
    set_pos(regions[1], 1.732, 0.0)

    save_scenario("two_region_trade", gd)

    gd2 = load_scenario("two_region_trade")
    r = gd2["regions"]
    x0, y0 = get_pos(r[0], 0, 2)
    x1, y1 = get_pos(r[1], 1, 2)
    assert abs(x0 - 0.0) < 1e-5 and abs(y0 - 0.0) < 1e-5, f"r0 pos wrong: {x0}, {y0}"
    assert abs(x1 - 1.732) < 1e-3 and abs(y1 - 0.0) < 1e-5, f"r1 pos wrong: {x1}, {y1}"
    print("  edit/save/roundtrip OK")
    return gd2


def test_attach(gd):
    regions = gd["regions"]
    # Move region 1 to a default position first, then attach to region 0
    set_pos(regions[1], 99.0, 99.0)   # far away
    ok = attach_region(regions, 1, 0)
    assert ok, "attach_region returned False"
    x, y = get_pos(regions[1], 1, 2)
    # Should be roughly HEX_D away from region 0
    dist = (x**2 + y**2) ** 0.5
    import math
    assert abs(dist - math.sqrt(3)) < 0.01, f"attached distance wrong: {dist}"
    print("  attach OK")


def test_run():
    result = run_simulation("two_region_trade", ticks=10)
    if result.returncode != 0:
        print("  STDOUT:", result.stdout[:300])
        print("  STDERR:", result.stderr[:500])
        raise AssertionError("simulation failed")
    print("  run OK")


if __name__ == "__main__":
    print("test_load...")
    gd = test_load()
    print("test_edit_save_roundtrip...")
    gd = test_edit_save_roundtrip(gd)
    print("test_attach...")
    test_attach(gd)
    print("test_run...")
    test_run()
    print("\nAll tests passed.")
