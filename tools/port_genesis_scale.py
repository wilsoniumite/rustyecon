"""Give every building a genesis scale, so a kernel run does not start from nothing.

THE PROBLEM. The corpus tapes register `chosen_size: 0.0` for every building.
That is harmless under the legacy agent, whose PD controller computes a target
from demand and overwrites the field on tick 1 — genesis scale is very nearly
unobservable there. Under the desk kernel `scale` IS the state variable, moved
only by small multiplicative nudges, so a tape saying 0.0 means every desk opens
at the clamp floor `epsilon * size` = 1% of nameplate and needs

    ln(100) / ln(1 + eta_up/2)  ~= 230 activations  ~= 920 ticks

to climb back. The scored window is ticks 150..1000. Measured on lr_00: nothing
produces, flour goes 0.6 -> ~10,000, wheat and labour go to 0.

THE RULE, STATED BEFORE IT WAS APPLIED (METHODOLOGY R6). A building's genesis
scale is its own `recipe_size` — nameplate. Three reasons it is not a tuned
choice:

  * It is the only other distinguished value in the tape. The choice is between
    "this world has never produced anything" (0.0) and "this world was running
    before tick 0" (nameplate), and every other thing in these tapes says the
    latter: pops hold cash, buildings hold cash, prices are not 1.0.
  * It is uniform. One rule, every building, every scenario — not a per-scenario
    number picked until something passed.
  * It cannot flatter a scored series by construction. Starting at nameplate is
    the value the kernel must adjust DOWN from if it is wrong, so if capacity is
    excessive the run shows a collapse it would otherwise have hidden.

WHAT IT DOES TO THE OTHER ARM. Legacy's first tick reads `last_throughput`,
which the loader initialises to 0.0 regardless, and blends 0.2 * last_throughput
with 0.8 * target — so this moves legacy by at most one tick of transient inside
a window that discards the first 150. The A/B is reported both with and without
the port rather than either being asserted to be the fair one.

    python tools/port_genesis_scale.py [--write]
"""

from __future__ import annotations

import glob
import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent


def port(path: pathlib.Path, write: bool):
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")
    out, changed, size = [], 0, None
    for line in lines:
        m = re.match(r"(\s*)recipe_size:\s*([0-9][0-9.eE+-]*)\s*,", line)
        if m:
            size = m.group(2)
            out.append(line)
            continue
        m2 = re.match(r"(\s*)chosen_size:\s*([0-9][0-9.eE+-]*)\s*,", line)
        if m2 and size is not None and float(m2.group(2)) == 0.0:
            out.append(f"{m2.group(1)}chosen_size: {size},")
            changed += 1
            size = None
            continue
        out.append(line)
    if write and changed:
        path.write_text("\n".join(out), encoding="utf-8")
    return changed


def main() -> int:
    write = "--write" in sys.argv
    total = 0
    for f in sorted(glob.glob(str(REPO / "data" / "scenarios" / "*" / "starting_state.ron"))):
        p = pathlib.Path(f)
        n = port(p, write)
        total += n
        print(f"  {p.parent.name:14s} {n} buildings")
    print(f"{'wrote' if write else 'would write'} {total} genesis scales")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
