#!/usr/bin/env python3
"""The chart modules are what tools/chart-codegen/generate.py writes, and nothing else.

    python3 tools/check-chart-codegen.py [ROOT]
"""

import importlib.util
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent


def check(root):
    spec = importlib.util.spec_from_file_location("chart_codegen", HERE / "chart-codegen/generate.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    root = pathlib.Path(root)
    problems = []
    outputs = module.outputs()
    for path, text in outputs.items():
        target = root / path
        if not target.is_file() or target.read_text(encoding="utf8") != text:
            problems.append(f"{path}: differs from what tools/chart-codegen/generate.py writes")
    return problems, f"chart code: {len(outputs)} generated files, each as the schemas give it"


if __name__ == "__main__":
    found, summary = check(sys.argv[1] if len(sys.argv) > 1 else HERE.parent)
    for problem in found:
        print(problem, file=sys.stderr)
    print(summary if not found else "chart code differs")
    sys.exit(1 if found else 0)
