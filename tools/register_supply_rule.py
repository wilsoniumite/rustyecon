"""Register `kernel.supply_rule` in every scenario tape (METHODOLOGY R2).

The field lands at `inelastic` — the SHIPPED behaviour — in all 33 tapes, not at
the new `reservation` rule, and that is the point rather than caution: R10 says a
change ships when the certified suite says it is no worse, and the A/B has not
been run when this script executes. Flipping the tapes is a decision taken on the
receipt, not on the way to producing it. `--supply-rule reservation` runs the new
mechanism meanwhile, and the effective value is folded into the run identity.

Encoding is passed explicitly on every read and every write. A previous session
corrupted 28 files by letting cp1252 pick itself on a Windows box; several of
these tapes carry non-ASCII in their headers.

    python tools/register_supply_rule.py            # apply
    python tools/register_supply_rule.py --check    # verify, touch nothing
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TAPES = sorted(ROOT.glob("data/scenarios/*/game_data.ron"))

# Anchored on `fill_alpha:` because it is the last dial in every kernel block and
# because anchoring on `kernel: (` would need brace matching through comments.
# `\r?$` because the corpus is mixed and `$` in MULTILINE matches before the
# `\n` of a `\r\n`, leaving the `\r` unmatched: without it this regex silently
# finds nothing in the five CRLF tapes and the script reports success.
FILL_ALPHA = re.compile(r"^([ \t]*)fill_alpha:[^,\r\n]+,[ \t]*(?=\r?$)", re.MULTILINE)

FIELD = "supply_rule"
VALUE = "inelastic"


def patch(text: str, eol: str) -> str:
    if re.search(rf"^\s*{FIELD}\s*:", text, re.MULTILINE):
        return text
    matches = list(FILL_ALPHA.finditer(text))
    if len(matches) != 1:
        raise SystemExit(f"expected exactly one fill_alpha line, found {len(matches)}")
    m = matches[0]
    indent = m.group(1)
    lines = [
        "// Which Rule 1 the desks post under. `inelastic` is the rule",
        "// shipped through Phase 4 (posted supply has price elasticity",
        "// exactly zero); `reservation` divides the band by the desk's",
        "// own markup, so b_out becomes the supply curve's slope.",
        "// Registered here, swept with --supply-rule.",
        f"{FIELD}: {VALUE},",
    ]
    insert = eol + eol.join(indent + line for line in lines)
    return text[: m.end()] + insert + text[m.end() :]


def main() -> int:
    check = "--check" in sys.argv
    if not TAPES:
        raise SystemExit("no tapes found")
    missing, changed = [], []
    for tape in TAPES:
        # newline="" on both legs: the corpus is mixed — the solv_* tapes are
        # CRLF and the rest are LF — and Python's default text mode would
        # rewrite every LF tape to CRLF, turning a six-line insertion into a
        # 33-file whitespace churn that hides it. Line endings are preserved
        # exactly as found; only the inserted block uses "\n", matched below.
        with open(tape, "r", encoding="utf-8", newline="") as fh:
            text = fh.read()
        eol = "\r\n" if "\r\n" in text else "\n"
        patched = patch(text, eol)
        if patched == text:
            if not re.search(rf"^\s*{FIELD}\s*:", text, re.MULTILINE):
                missing.append(tape)
            continue
        if check:
            missing.append(tape)
            continue
        with open(tape, "w", encoding="utf-8", newline="") as fh:
            fh.write(patched)
        changed.append(tape)

    print(f"{len(TAPES)} tapes, {len(changed)} patched, {len(missing)} still missing {FIELD}")
    for tape in missing:
        print(f"  MISSING {tape.relative_to(ROOT)}")
    return 1 if missing else 0


if __name__ == "__main__":
    raise SystemExit(main())
