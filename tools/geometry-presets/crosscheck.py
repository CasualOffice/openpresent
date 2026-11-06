#!/usr/bin/env python3
"""Compare evaluated presets with LibreOffice's independently shipped copy.

Not part of CI: it needs a LibreOffice install. LibreOffice ships its own compiled
presets in filter/oox-drawingml-cs-presets. This evaluates those equations at the
same size and compares path vertices, arc parameters, glue points, text frames and
handle positions with this engine's output, from

    cargo run -q -p casual-slide-geometry --example dump_geometry -- W H > dump.jsonl
    python3 tools/geometry-presets/crosscheck.py dump.jsonl W H [preset-file]

Agreement here says two readings of the same source agree. It says nothing about
PowerPoint, which neither has been compared with.
"""
import json
import math
import pathlib
import re
import sys

DEFAULT = "/Applications/LibreOffice.app/Contents/Resources/filter/oox-drawingml-cs-presets"
PARAM = re.compile(r"Value = \(any\) \{ \((?:long|double)\) (-?[0-9.eE+-]+) \}, Type = \(short\) (\d+)")
SEGMENT = re.compile(r"Command = \(short\) (\d+), Count = \(short\) (\d+)")
ADJ = re.compile(r'Value = \(any\) \{ \((?:long|double)\) (-?[0-9.eE+-]+) \}')
TOKENS = re.compile(r"\s*(\$\d+|\?\d+|\d+\.?\d*(?:[eE][+-]?\d+)?|[A-Za-z_][A-Za-z_0-9]*|[-+*/(),])")
TOLERANCE = 0.51  # EMU. Both sides evaluate in doubles; this only absorbs rounding.


def split_presets(path):
    presets, name, lines = {}, None, []
    for line in pathlib.Path(path).read_text(encoding="utf8").split("\n"):
        match = re.match(r"/\* (\S+) \*/$", line)
        if match:
            name = match.group(1)
            presets[name] = {}
        elif name:
            lines.append(line)
            if len(lines) == 2:
                presets[name][lines[0]] = lines[1]
                lines = []
    return presets


class Evaluator:
    """LibreOffice's custom shape expressions: a small recursive-descent reader."""

    def __init__(self, equations, adjusts, w, h):
        self.equations, self.adjusts, self.w, self.h = equations, adjusts, w, h
        self.memo = {}

    def equation(self, index):
        if index not in self.memo:
            saved = getattr(self, "pos", None), getattr(self, "toks", None)
            self.toks = TOKENS.findall(self.equations[index])
            self.pos = 0
            self.memo[index] = self.expr()
            self.pos, self.toks = saved
        return self.memo[index]

    def peek(self):
        return self.toks[self.pos] if self.pos < len(self.toks) else None

    def take(self):
        self.pos += 1
        return self.toks[self.pos - 1]

    def expr(self):
        value = self.term()
        while self.peek() in ("+", "-"):
            value = value + self.term() if self.take() == "+" else value - self.term()
        return value

    def term(self):
        value = self.unary()
        while self.peek() in ("*", "/"):
            if self.take() == "*":
                value *= self.unary()
            else:
                d = self.unary()
                value = value / d if d else 0.0
        return value

    def unary(self):
        if self.peek() == "-":
            self.take()
            return -self.unary()
        return self.atom()

    def args(self):
        self.take()  # (
        out = [self.expr()]
        while self.peek() == ",":
            self.take()
            out.append(self.expr())
        self.take()  # )
        return out

    def atom(self):
        token = self.take()
        if token == "(":
            value = self.expr()
            self.take()
            return value
        if token.startswith("$"):
            return self.adjusts[int(token[1:])]
        if token.startswith("?"):
            return self.equation(int(token[1:]))
        if token == "logwidth" or token == "width":
            return self.w
        if token == "logheight" or token == "height":
            return self.h
        if token == "pi":
            return math.pi
        if token in ("left", "top"):
            return 0.0
        if token == "right":
            return self.w
        if token == "bottom":
            return self.h
        if token == "hc":
            return self.w / 2
        if token == "vc":
            return self.h / 2
        if token[0].isalpha():
            a = self.args()
            return {
                "if": lambda: a[1] if a[0] > 0 else a[2],
                "sin": lambda: math.sin(a[0]),
                "cos": lambda: math.cos(a[0]),
                "tan": lambda: math.tan(a[0]),
                "atan2": lambda: math.atan2(a[0], a[1]),
                "atan": lambda: math.atan(a[0]),
                "sqrt": lambda: math.sqrt(max(a[0], 0)),
                "abs": lambda: abs(a[0]),
                "min": lambda: min(a),
                "max": lambda: max(a),
            }[token]()
        return float(token)

    def param(self, value, kind):
        if kind == 0:
            return value
        if kind == 1:
            return self.equation(int(value))
        if kind == 2:
            return self.adjusts[int(value)]
        return {3: 0.0, 4: 0.0, 5: self.w, 6: self.h}.get(kind, value)


def section(text, name):
    start = text.find(f'Name = "{name}"')
    if start < 0:
        return ""
    end = text.find('State = (com.sun.star.beans.PropertyState)', start)
    return text[start:end]


def pairs(block, evaluator):
    values = [evaluator.param(float(v), int(k)) for v, k in PARAM.findall(block)]
    return [(values[i], values[i + 1]) for i in range(0, len(values) - 1, 2)]


def lo_tokens(segments, coordinates):
    """Flatten LibreOffice segments to a comparable list, ignoring fill commands."""
    out, cursor = [], 0
    for command, count in segments:
        if command == 1:
            for _ in range(count):
                out.append(("M", coordinates[cursor]))
                cursor += 1
        elif command == 2:
            for _ in range(count):
                out.append(("L", coordinates[cursor]))
                cursor += 1
        elif command == 3:
            for _ in range(count):
                out.append(("C", coordinates[cursor], coordinates[cursor + 1], coordinates[cursor + 2]))
                cursor += 3
        elif command == 16:
            for _ in range(count):
                out.append(("Q", coordinates[cursor], coordinates[cursor + 1]))
                cursor += 2
        elif command == 17:
            for _ in range(count):
                out.append(("A", coordinates[cursor], coordinates[cursor + 1]))
                cursor += 2
        elif command == 4:
            out.append(("Z",))
        elif command in (5, 6, 7, 18, 19, 20, 21):
            continue
        else:
            raise ValueError(f"segment command {command}")
    return out


def my_tokens(paths):
    out = []
    for path in paths:
        for segment in path["segments"]:
            kind = segment[0]
            if kind == "A":
                out.append(("A", tuple(segment[1]), (segment[2][0] / 60000, segment[2][1] / 60000)))
            else:
                out.append((kind, *[tuple(p) for p in segment[1:]]))
    return out


def close(a, b, tolerance=TOLERANCE):
    return abs(a - b) <= tolerance


def same(mine, theirs):
    if mine[0] != theirs[0] or len(mine) != len(theirs):
        return False
    for p, q in zip(mine[1:], theirs[1:]):
        if not (close(p[0], q[0]) and close(p[1], q[1])):
            return False
    return True


def main():
    dump, w, h = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
    presets = split_presets(sys.argv[4] if len(sys.argv) > 4 else DEFAULT)
    results = {"paths": [], "sites": [], "rect": [], "handles": []}
    missing, compared = [], 0
    checked = {}  # sections compared for fewer presets than all of them
    checked["rect"] = 0
    for line in pathlib.Path(dump).read_text(encoding="utf8").splitlines():
        mine = json.loads(line)
        name = mine["name"]
        if name not in presets:
            missing.append(name)
            continue
        compared += 1
        lo = presets[name]
        adjusts = [float(v) for v in ADJ.findall(lo.get("AdjustmentValues", ""))]
        equations = re.findall(r'"([^"]*)"', lo.get("Equations", ""))
        path = lo.get("Path", "")
        ev = Evaluator(equations, adjusts, w, h)
        coords = pairs(section(path, "Coordinates"), ev)
        segs = [(int(a), int(b)) for a, b in SEGMENT.findall(section(path, "Segments"))]
        theirs = lo_tokens(segs, coords)
        ours = my_tokens(mine["paths"])
        # Arc angles arrive in degrees on both sides of the comparison.
        if len(ours) != len(theirs) or not all(same(a, b) for a, b in zip(ours, theirs)):
            bad = next(
                (i for i, (a, b) in enumerate(zip(ours, theirs)) if not same(a, b)),
                min(len(ours), len(theirs)),
            )
            results["paths"].append(
                f"{name}: token {bad}/{len(ours)} vs {len(theirs)}: "
                f"{ours[bad] if bad < len(ours) else None} vs {theirs[bad] if bad < len(theirs) else None}"
            )
        glue = pairs(section(path, "GluePoints"), ev)
        if len(glue) != len(mine["sites"]) or not all(
            close(a[0], b[0]) and close(a[1], b[1]) for a, b in zip(mine["sites"], glue)
        ):
            results["sites"].append(f"{name}: {mine['sites']} vs {glue}")
        frames = pairs(section(path, "TextFrames"), ev)
        if len(frames) >= 2:
            checked["rect"] += 1
            r = mine["rect"]
            want = [frames[0][0], frames[0][1], frames[1][0], frames[1][1]]
            if not all(close(a, b) for a, b in zip(r, want)):
                results["rect"].append(f"{name}: {r} vs {want}")
        spots = [
            tuple(ev.param(float(v), int(k)) for v, k in PARAM.findall(block))
            for block in re.findall(r'Name = "Position".*?State =', lo.get("Handles", ""))
        ]
        if len(spots) != len(mine["handles"]) or not all(
            close(a[0], b[0]) and close(a[1], b[1]) for a, b in zip(mine["handles"], spots)
        ):
            results["handles"].append(f"{name}: {mine['handles']} vs {spots}")
    print(f"compared {compared}; not in LibreOffice's table: {missing}")
    for key, items in results.items():
        total = checked.get(key, compared)
        print(f"{key}: {total - len(items)} of {total} agree, {len(items)} differ")
        for item in items:
            print("  " + item[:400])


if __name__ == "__main__":
    main()
