"""
Minimal RON parser/writer for rustyecon's game_data.ron schema.

Supported value types and their Python representations:
  RON None          -> None
  RON true/false    -> bool
  RON integer       -> int
  RON float         -> float
  RON "string"      -> str
  RON bare ident    -> Bare("name")   e.g. Regional, Physical, Variable
  RON TypeName(x)   -> Tagged("TypeName", x)   e.g. MarketNodeId(0), Some([1,2])
  RON [...]         -> list
  RON (a, b)        -> tuple          unnamed tuple
  RON (k: v, ...)   -> dict           named struct
  RON {"k": v, ...} -> RonMap         hashmap
"""

import re
from dataclasses import dataclass
from typing import Any


@dataclass
class Bare:
    """A bare RON identifier or enum variant, e.g. Regional, Physical."""
    value: str

    def __repr__(self):
        return self.value

    def __hash__(self):
        return hash(self.value)

    def __eq__(self, other):
        return isinstance(other, Bare) and self.value == other.value


@dataclass
class Tagged:
    """A RON tagged value, e.g. MarketNodeId(0), Some([1.0, 2.0])."""
    tag: str
    inner: Any

    def __repr__(self):
        return f"{self.tag}({self.inner!r})"


class RonMap(dict):
    """A RON hashmap {\"key\": value}, distinct from a named struct (k: v)."""
    pass


# ── Tokenizer ─────────────────────────────────────────────────────────────────

class _Parser:
    def __init__(self, text: str):
        self.text = re.sub(r"//[^\n]*", "", text)
        self.pos = 0

    def _skip(self):
        while self.pos < len(self.text) and self.text[self.pos] in " \t\n\r":
            self.pos += 1

    def _peek(self) -> str | None:
        self._skip()
        return self.text[self.pos] if self.pos < len(self.text) else None

    def _consume(self, expected: str):
        self._skip()
        if not self.text.startswith(expected, self.pos):
            ctx = self.text[self.pos:self.pos + 20]
            raise ValueError(f"Expected {expected!r}, got {ctx!r}")
        self.pos += len(expected)

    def _read_string(self) -> str:
        self._skip()
        assert self.text[self.pos] == '"'
        self.pos += 1
        buf = []
        while self.pos < len(self.text) and self.text[self.pos] != '"':
            if self.text[self.pos] == "\\" and self.pos + 1 < len(self.text):
                self.pos += 1
                buf.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}.get(
                    self.text[self.pos], self.text[self.pos]
                ))
            else:
                buf.append(self.text[self.pos])
            self.pos += 1
        self.pos += 1  # closing "
        return "".join(buf)

    def _read_number(self) -> int | float:
        self._skip()
        m = re.match(r"-?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?", self.text[self.pos:])
        if not m:
            raise ValueError(f"Expected number at {self.text[self.pos:self.pos+10]!r}")
        s = m.group(0)
        self.pos += len(s)
        return float(s) if ("." in s or "e" in s.lower()) else int(s)

    def _read_ident(self) -> str:
        self._skip()
        m = re.match(r"[A-Za-z_][A-Za-z0-9_]*", self.text[self.pos:])
        if not m:
            raise ValueError(f"Expected ident at {self.text[self.pos:self.pos+10]!r}")
        s = m.group(0)
        self.pos += len(s)
        return s

    def value(self) -> Any:
        c = self._peek()

        if c == '"':
            return self._read_string()

        if c == "[":
            return self._read_array()

        if c == "(":
            return self._read_paren()

        if c == "{":
            return self._read_map()

        if c is not None and (c.isdigit() or c == "-"):
            return self._read_number()

        if c is not None and (c.isalpha() or c == "_"):
            ident = self._read_ident()
            if ident == "true":
                return True
            if ident == "false":
                return False
            if ident == "None":
                return None
            # Check for tagged: TypeName(...)
            self._skip()
            if self.pos < len(self.text) and self.text[self.pos] == "(":
                # inside_tag=True: single-element parens unwrap (e.g. MarketNodeId(0) -> Tagged("MarketNodeId", 0))
                inner = self._read_paren(inside_tag=True)
                return Tagged(ident, inner)
            return Bare(ident)

        raise ValueError(f"Unexpected char {c!r} at pos {self.pos}")

    def _read_array(self) -> list:
        self._consume("[")
        items = []
        while self._peek() != "]":
            items.append(self.value())
            if self._peek() == ",":
                self._consume(",")
        self._consume("]")
        return items

    def _read_map(self) -> RonMap:
        self._consume("{")
        result = RonMap()
        while self._peek() != "}":
            key = self.value()
            self._consume(":")
            val = self.value()
            result[key] = val
            if self._peek() == ",":
                self._consume(",")
        self._consume("}")
        return result

    def _read_paren(self, inside_tag: bool = False) -> Any:
        """
        ( ... ) — named struct, unnamed tuple, or single-wrap.

        inside_tag=True  (called from a TypeName(...) context):
            single-element parens are unwrapped, e.g. MarketNodeId(0) -> Tagged("MarketNodeId", 0).
        inside_tag=False (bare value context):
            single-element scalar parens are kept as a 1-tuple so they round-trip
            as (0), which is what the Rust ron crate emits for newtype IDs.
        """
        self._consume("(")
        self._skip()
        if self._peek() == ")":
            self._consume(")")
            return {}

        # Look ahead: named struct starts with ident then ':'
        saved = self.pos
        try:
            ident = self._read_ident()
            self._skip()
            if self._peek() == ":":
                self._consume(":")
                fields = {ident: self.value()}
                while self._peek() == ",":
                    self._consume(",")
                    self._skip()
                    if self._peek() == ")":
                        break
                    k = self._read_ident()
                    self._consume(":")
                    fields[k] = self.value()
                self._consume(")")
                return fields
        except Exception:
            pass
        self.pos = saved

        # Unnamed tuple or single value
        items = []
        while self._peek() != ")":
            items.append(self.value())
            if self._peek() == ",":
                self._consume(",")
        self._consume(")")

        if len(items) != 1:
            return tuple(items)
        # Single element: unwrap only if inside a tag call OR the item is not a plain scalar.
        # Plain scalars in bare context stay as 1-tuples so they write back as (n).
        item = items[0]
        if inside_tag or not isinstance(item, (int, float)):
            return item
        return (item,)


# ── Public API ────────────────────────────────────────────────────────────────

def parse(text: str) -> Any:
    return _Parser(text).value()


def load(path: str) -> Any:
    with open(path, encoding="utf-8") as f:
        return parse(f.read())


# ── Writer ────────────────────────────────────────────────────────────────────

def _simple(v: Any) -> bool:
    """True if value fits on one line."""
    if isinstance(v, (bool, int, float, str, Bare, type(None))):
        return True
    if isinstance(v, Tagged):
        return _simple(v.inner)
    if isinstance(v, (list, tuple)) and len(v) <= 4 and all(_simple(i) for i in v):
        return True
    return False


def _w(v: Any, depth: int = 0) -> str:
    pad = "    " * depth
    ipad = "    " * (depth + 1)

    if v is None:
        return "None"
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, float):
        s = repr(v)
        return s if ("." in s or "e" in s.lower()) else s + ".0"
    if isinstance(v, str):
        escaped = v.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(v, Bare):
        return v.value
    if isinstance(v, Tagged):
        inner = v.inner
        if isinstance(inner, dict) and inner:
            # Enum variant with named fields — inline directly: Foo(a: 1, b: 2)
            lines = [f"{ipad}{k}: {_w(val, depth + 1)}" for k, val in inner.items()]
            return f"{v.tag}(\n" + ",\n".join(lines) + ",\n" + pad + ")"
        if isinstance(inner, dict):
            return f"{v.tag}()"
        inner_s = _w(inner, depth)
        return f"{v.tag}({inner_s})"
    if isinstance(v, tuple):
        parts = [_w(i, depth) for i in v]
        return "(" + ", ".join(parts) + ")"
    if isinstance(v, list):
        if not v:
            return "[]"
        if _simple(v):
            return "[" + ", ".join(_w(i, depth) for i in v) + "]"
        lines = [f"{ipad}{_w(i, depth + 1)}" for i in v]
        return "[\n" + ",\n".join(lines) + ",\n" + pad + "]"
    if isinstance(v, RonMap):
        if not v:
            return "{}"
        items = [f"{ipad}{_w(k, depth+1)}: {_w(val, depth+1)}" for k, val in v.items()]
        return "{\n" + ",\n".join(items) + ",\n" + pad + "}"
    if isinstance(v, dict):
        if not v:
            return "()"
        lines = [f"{ipad}{k}: {_w(val, depth + 1)}" for k, val in v.items()]
        return "(\n" + ",\n".join(lines) + ",\n" + pad + ")"

    raise TypeError(f"Cannot write RON for {type(v)}: {v!r}")


def dumps(v: Any) -> str:
    return _w(v, 0) + "\n"


def dump(v: Any, path: str):
    with open(path, "w", encoding="utf-8") as f:
        f.write(dumps(v))
