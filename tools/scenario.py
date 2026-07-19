"""
Non-Streamlit scenario I/O and simulation runner.
Import this from app.py or standalone test scripts.
"""

import math
import subprocess
from dataclasses import dataclass
from pathlib import Path

import ron
from ron import Tagged

REPO_ROOT = Path(__file__).parent.parent
SCENARIOS_DIR = REPO_ROOT / "data" / "scenarios"
BINARY = REPO_ROOT / "target" / "debug" / "rustyecon"

HEX_D = math.sqrt(3)   # center-to-center distance (circumradius = 1.0)
SNAP_EPS = HEX_D * 0.35
NEIGHBOR_ANGLES = [i * math.pi / 3 for i in range(6)]


# ── Scenario I/O ──────────────────────────────────────────────────────────────

def list_scenarios() -> list[str]:
    if not SCENARIOS_DIR.exists():
        return []
    return sorted(p.name for p in SCENARIOS_DIR.iterdir() if p.is_dir())


def load_scenario(name: str) -> dict:
    return ron.load(str(SCENARIOS_DIR / name / "game_data.ron"))


def save_scenario(name: str, gd: dict):
    ron.dump(gd, str(SCENARIOS_DIR / name / "game_data.ron"))


def load_starting_state(name: str) -> dict:
    return ron.load(str(SCENARIOS_DIR / name / "starting_state.ron"))


def save_starting_state(name: str, ss: dict):
    ron.dump(ss, str(SCENARIOS_DIR / name / "starting_state.ron"))


def load_events(name: str) -> dict:
    return ron.load(str(SCENARIOS_DIR / name / "events.ron"))


def save_events(name: str, ev: dict):
    ron.dump(ev, str(SCENARIOS_DIR / name / "events.ron"))


# ── Run ───────────────────────────────────────────────────────────────────────

# Engine exit codes. A certified run exits 1 when its verdict is FAIL: the
# certificate and telemetry are both whole, the economy simply did not pass.
# That is the engine working as designed (verdict-first, fail-closed) and it is
# the expected outcome for every scenario in the corpus today, so a caller that
# treats it as a crash never sees its own results. Rust panics exit 101.
EXIT_OK = 0
EXIT_CERTIFICATE_FAIL = 1
EXIT_CERTIFICATE_WRITE_FAILED = 2
EXIT_TELEMETRY_FAILED = 3


@dataclass
class RunResult:
    returncode: int
    stdout: str
    stderr: str

    @property
    def ok(self) -> bool:
        """The run finished and its artifacts are trustworthy.

        True for a FAIL verdict. False for a telemetry or certificate write
        failure, and for a crash - in those cases the output on disk is
        incomplete and must not be analysed.
        """
        return self.returncode in (EXIT_OK, EXIT_CERTIFICATE_FAIL)

    @property
    def certificate_failed(self) -> bool:
        return self.returncode == EXIT_CERTIFICATE_FAIL


def run_simulation(
    name: str,
    ticks: int = 100,
    output_dir: str | None = None,
    record: bool = False,
    certify: bool = False,
    results_dir: str | None = None,
    telemetry_every: int = 1,
    human_save_every: int = 0,
    build: bool = True,
    timeout: int = 300,
) -> RunResult:
    if output_dir is None:
        output_dir = str(REPO_ROOT / "tmp" / f"out_{name}")
    Path(output_dir).mkdir(parents=True, exist_ok=True)

    scenario_path = str(SCENARIOS_DIR / name)
    extra = []
    if record:
        extra.append("--record")
        if telemetry_every != 1:
            extra += ["--telemetry-every", str(telemetry_every)]
    if certify:
        extra.append("--certify")
        # Certificates default to repo-root results/, which is committed
        # evidence (R5). A batch or exploratory run must not silently rewrite
        # it, so unless a caller asks for somewhere specific, certificates land
        # beside the run's own telemetry. Regenerating the committed verdicts
        # is then a deliberate act with an explicit --results.
        extra += ["--results", results_dir or str(Path(output_dir) / "results")]
    if human_save_every > 0:
        extra += ["--human-save", str(human_save_every)]

    if build:
        cmd = ["cargo", "run", "--", scenario_path,
               "--ticks", str(ticks), "--output", output_dir] + extra
    else:
        cmd = [str(BINARY), scenario_path,
               "--ticks", str(ticks), "--output", output_dir] + extra

    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=timeout,
        cwd=str(REPO_ROOT),
    )
    return RunResult(result.returncode, result.stdout, result.stderr)


# ── Position helpers ──────────────────────────────────────────────────────────

def _default_pos(idx: int, total: int) -> tuple[float, float]:
    if total == 1:
        return 0.0, 0.0
    angle = 2 * math.pi * idx / total
    radius = HEX_D * max(1, total // 2)
    return radius * math.cos(angle), radius * math.sin(angle)


def get_pos(region: dict, idx: int, total: int) -> tuple[float, float]:
    p = region.get("position")
    if isinstance(p, Tagged) and p.tag == "Some":
        inner = p.inner
        if isinstance(inner, (list, tuple)) and len(inner) == 2:
            return float(inner[0]), float(inner[1])
    return _default_pos(idx, total)


def set_pos(region: dict, x: float, y: float):
    region["position"] = Tagged("Some", (float(x), float(y)))


# ── Hex attach ────────────────────────────────────────────────────────────────

def neighbor_slots(cx: float, cy: float) -> list[tuple[float, float]]:
    return [(cx + HEX_D * math.cos(a), cy + HEX_D * math.sin(a))
            for a in NEIGHBOR_ANGLES]


def attach_region(regions: list, src_idx: int, tgt_idx: int) -> bool:
    """
    Snap regions[src_idx] to the best free side of regions[tgt_idx].
    Prefers the side facing src's current position. Returns True on success.
    """
    n = len(regions)
    tx, ty = get_pos(regions[tgt_idx], tgt_idx, n)
    sx_old, sy_old = get_pos(regions[src_idx], src_idx, n)

    pref = math.atan2(sy_old - ty, sx_old - tx)
    slots = neighbor_slots(tx, ty)

    occupied: set[int] = set()
    for i, r in enumerate(regions):
        if i in (src_idx, tgt_idx):
            continue
        rx, ry = get_pos(r, i, n)
        for s_idx, (slx, sly) in enumerate(slots):
            if abs(rx - slx) < SNAP_EPS and abs(ry - sly) < SNAP_EPS:
                occupied.add(s_idx)

    best, best_diff = None, float("inf")
    for s_idx, (slx, sly) in enumerate(slots):
        if s_idx in occupied:
            continue
        diff = abs(math.atan2(
            math.sin(pref - NEIGHBOR_ANGLES[s_idx]),
            math.cos(pref - NEIGHBOR_ANGLES[s_idx]),
        ))
        if diff < best_diff:
            best_diff, best = diff, s_idx

    if best is not None:
        set_pos(regions[src_idx], *slots[best])
        return True
    return False
