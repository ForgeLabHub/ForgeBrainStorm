#!/usr/bin/env python3
"""Превращает отчёт `pip install --dry-run --report` в закреплённый список версий.

Зачем: requirements.txt задаёт диапазоны, и две установки в разное время дают разные
версии. Переносимость требует фиксированного набора.

    python -m pip install --dry-run --ignore-installed --report report.json -r requirements.txt
    python tools/make_lock.py report.json > requirements.lock.txt
"""
from __future__ import annotations

import json
import sys


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    with open(argv[1], encoding="utf-8") as handle:
        report = json.load(handle)
    packages = sorted(
        (item["metadata"]["name"].lower(), item["metadata"]["version"])
        for item in report["install"]
    )
    print("# Зафиксированные версии, разрешённые из requirements.txt")
    print("# Сгенерировано tools/make_lock.py — не редактировать вручную.")
    print("# Обновление: make lock, затем make test, затем коммит.")
    print()
    for name, version in packages:
        print(f"{name}=={version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
